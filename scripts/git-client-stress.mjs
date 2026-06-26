#!/usr/bin/env node
import { spawn, spawnSync } from 'node:child_process';
import { mkdirSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const defaults = {
  baseUrl: '',
  port: '18080',
  project: 'gity-blackbox',
  build: false,
  noCompose: false,
  workers: '4',
  iterations: '24',
  mode: 'mixed',
  clonePercent: '20',
  fetchPercent: '45',
  pushPercent: '25',
  lsRemotePercent: '10',
  timeoutMs: '60000',
  workdir: '.tmp-perf/git-client',
  keepWorkdir: false,
  seedFiles: '8',
  projectUrl: '',
  token: '',
  tokenUsername: 'git-perf',
};

const args = parseArgs(process.argv.slice(2));
const config = normalizeConfig({ ...defaults, ...args });
const composeFile = path.join(root, 'docker-compose.blackbox.yaml');
const baseUrl = config.baseUrl || `http://127.0.0.1:${config.port}`;
const runID = uniqueKey('gitperf');
const workRoot = path.resolve(root, config.workdir, runID);
const bypassGitProxy = shouldBypassGitProxy(baseUrl, config.projectUrl);

if (!hasCommand('git')) fail('git executable was not found in PATH.');

if (!config.noCompose) {
  const composeArgs = ['compose', '-p', config.project, '-f', composeFile, 'up', '-d'];
  if (config.build) composeArgs.push('--build');
  run('docker', composeArgs, {
    cwd: root,
    env: { ...process.env, GITY_BLACKBOX_WEB_PORT: config.port },
  });
  await waitForBlackbox(baseUrl, config.project, composeFile);
}

mkdirSync(workRoot, { recursive: true });
const timings = new Map();
const errors = [];
const context = await prepareContext();

console.log(`Git client stress target: ${sanitize(context.authenticatedUrl, context.token)}`);
console.log(`workload mode=${config.mode} workers=${config.workers} iterations=${config.iterations}`);

const workerDirs = await prepareWorkerClones(context);
let nextIteration = 0;
await Promise.all(workerDirs.map((dir, index) => runWorker(index, dir, context)));

printSummary();
if (!config.keepWorkdir) safeRemove(workRoot, workRoot);
if (errors.length > 0) process.exit(1);

async function prepareContext() {
  if (config.projectUrl && config.token) {
    return {
      token: config.token,
      tokenUsername: config.tokenUsername,
      authenticatedUrl: withCredentials(config.projectUrl, config.tokenUsername, config.token),
    };
  }

  await fetchOK(`${baseUrl}/api/health`);
  const session = await api('POST', '/auth/login', null, {
    username: `${runID}-owner`,
    password: 'password',
  });
  const accessToken = stringField(session, 'token');
  const userID = numericField(session, 'user_id', 'id');
  if (!accessToken || userID <= 0) fail(`login response did not include token/user_id: ${JSON.stringify(session)}`);

  let organization = asArray(await api('GET', '/orgs', accessToken))[0];
  if (!organization) {
    organization = await api('POST', '/orgs', accessToken, {
      key: `${runID}-org`,
      path_key: `${runID}-org`,
      owner_user_id: userID,
      name: `Git Perf ${runID}`,
      description: 'Created by git client stress setup',
      visibility: 'private',
    });
  }
  const organizationID = stringField(organization, 'id');
  if (!organizationID) fail(`organization response did not include id: ${JSON.stringify(organization)}`);

  const project = await api('POST', '/projects', accessToken, {
    organization_id: organizationID,
    key: `${runID}-project`,
    path_key: `${runID}-project`,
    name: `Git Perf ${runID}`,
    description: 'Project created by git client stress setup',
    visibility: 'private',
    default_branch: 'main',
  });
  const projectID = stringField(project, 'id');
  const cloneUrl = stringField(project, 'clone_http_url') || `${baseUrl}/${stringField(project, 'full_path')}.git`;
  if (!projectID || !cloneUrl) fail(`project response did not include id/clone url: ${JSON.stringify(project)}`);

  const credential = await api('POST', `/projects/${projectID}/access-tokens`, accessToken, {
    name: `${runID}-git-token`,
    username: config.tokenUsername,
    scopes: ['read_repository', 'write_repository'],
    created_by_user_id: userID,
    expires_at: '',
  });
  const gitToken = stringField(credential, 'token');
  if (!gitToken) fail(`project access token response did not include token: ${JSON.stringify(credential)}`);

  const authenticatedUrl = withCredentials(cloneUrl, config.tokenUsername, gitToken);
  await seedInitialRepository(authenticatedUrl, gitToken);
  return { token: gitToken, tokenUsername: config.tokenUsername, authenticatedUrl };
}

async function seedInitialRepository(remoteUrl, token) {
  const seedDir = path.join(workRoot, 'seed');
  mkdirSync(seedDir, { recursive: true });
  await timed('setup.init', () => git(['init', '-b', 'main'], seedDir, token));
  await git(['config', 'user.name', 'Gity Perf'], seedDir, token);
  await git(['config', 'user.email', 'perf@gity.local'], seedDir, token);
  writeFileSync(path.join(seedDir, 'README.md'), `# ${runID}\n\nGit client stress seed repository.\n`);
  mkdirSync(path.join(seedDir, 'src'), { recursive: true });
  for (let index = 0; index < config.seedFiles; index += 1) {
    writeFileSync(path.join(seedDir, 'src', `file-${index}.txt`), `seed ${runID} ${index}\n`);
  }
  await git(['add', '.'], seedDir, token);
  await timed('setup.commit', () => git(['commit', '-m', 'Seed git client stress repository'], seedDir, token));
  await git(['remote', 'add', 'origin', remoteUrl], seedDir, token);
  await timed('setup.push', () => git(['push', 'origin', 'HEAD:refs/heads/main'], seedDir, token));
}

async function prepareWorkerClones(context) {
  const dirs = [];
  for (let index = 0; index < config.workers; index += 1) {
    const dir = path.join(workRoot, `worker-${index}`);
    await timed('setup.clone', () => git(['clone', '--quiet', '--branch', 'main', context.authenticatedUrl, dir], workRoot, context.token));
    await git(['config', 'user.name', `Gity Perf ${index}`], dir, context.token);
    await git(['config', 'user.email', `perf-${index}@gity.local`], dir, context.token);
    dirs.push(dir);
  }
  return dirs;
}

async function runWorker(workerIndex, cwd, context) {
  for (;;) {
    const iteration = nextIteration;
    nextIteration += 1;
    if (iteration >= config.iterations) return;
    const operation = chooseOperation(iteration);
    try {
      await runOperation(operation, workerIndex, iteration, cwd, context);
    } catch (error) {
      errors.push({ operation, workerIndex, iteration, message: String(error.message || error) });
      console.error(`[${operation}] worker=${workerIndex} iteration=${iteration} failed: ${sanitize(String(error.message || error), context.token)}`);
    }
  }
}

async function runOperation(operation, workerIndex, iteration, cwd, context) {
  switch (operation) {
    case 'ls-remote':
      await timed('ls-remote', () => git(['ls-remote', '--heads', context.authenticatedUrl], workRoot, context.token));
      return;
    case 'clone':
      await cloneOnce(workerIndex, iteration, context);
      return;
    case 'fetch':
      await timed('fetch', () => git(['fetch', '--prune', 'origin'], cwd, context.token));
      return;
    case 'push':
      await pushOnce(workerIndex, iteration, cwd, context);
      return;
    default:
      throw new Error(`unsupported operation: ${operation}`);
  }
}

async function cloneOnce(workerIndex, iteration, context) {
  const cloneDir = path.join(workRoot, `clone-${workerIndex}-${iteration}`);
  try {
    await timed('clone', () => git(['clone', '--quiet', '--depth', '1', context.authenticatedUrl, cloneDir], workRoot, context.token));
  } finally {
    if (!config.keepWorkdir) safeRemove(workRoot, cloneDir);
  }
}

async function pushOnce(workerIndex, iteration, cwd, context) {
  const branch = `perf/${runID}/${workerIndex}-${iteration}`;
  await git(['checkout', '--quiet', '-B', branch, 'origin/main'], cwd, context.token);
  const dir = path.join(cwd, 'perf-output', String(workerIndex));
  mkdirSync(dir, { recursive: true });
  writeFileSync(path.join(dir, `${iteration}.txt`), `worker=${workerIndex}\niteration=${iteration}\nrun=${runID}\n`);
  await git(['add', '.'], cwd, context.token);
  await git(['commit', '-m', `perf push ${workerIndex}-${iteration}`], cwd, context.token);
  await timed('push', () => git(['push', 'origin', `HEAD:refs/heads/${branch}`], cwd, context.token));
}

function chooseOperation(iteration) {
  if (config.mode !== 'mixed') return config.mode;
  const weights = [
    ['clone', config.clonePercent],
    ['fetch', config.fetchPercent],
    ['push', config.pushPercent],
    ['ls-remote', config.lsRemotePercent],
  ];
  const total = weights.reduce((sum, [, weight]) => sum + weight, 0);
  let cursor = (iteration * 9973) % total;
  for (const [operation, weight] of weights) {
    if (cursor < weight) return operation;
    cursor -= weight;
  }
  return 'fetch';
}

async function timed(name, fn) {
  const started = performance.now();
  try {
    await fn();
    record(name, performance.now() - started, true);
  } catch (error) {
    record(name, performance.now() - started, false);
    throw error;
  }
}

function record(name, duration, ok) {
  const stat = timings.get(name) || { count: 0, failed: 0, durations: [] };
  stat.count += 1;
  if (!ok) stat.failed += 1;
  stat.durations.push(duration);
  timings.set(name, stat);
}

function printSummary() {
  console.log('\nGit client stress summary');
  const rows = [...timings.entries()].sort(([left], [right]) => left.localeCompare(right));
  for (const [name, stat] of rows) {
    const durations = stat.durations.slice().sort((a, b) => a - b);
    const avg = durations.reduce((sum, value) => sum + value, 0) / Math.max(durations.length, 1);
    console.log(`${name.padEnd(14)} count=${String(stat.count).padStart(4)} failed=${String(stat.failed).padStart(3)} avg=${formatMs(avg)} p50=${formatMs(percentile(durations, 0.5))} p95=${formatMs(percentile(durations, 0.95))} max=${formatMs(durations[durations.length - 1] || 0)}`);
  }
  if (errors.length > 0) {
    console.log(`\nFailures: ${errors.length}`);
  }
}

function percentile(values, rank) {
  if (values.length === 0) return 0;
  const index = Math.min(values.length - 1, Math.ceil(values.length * rank) - 1);
  return values[index];
}

function formatMs(value) {
  return `${value.toFixed(1)}ms`;
}

async function api(method, route, token, body) {
  const response = await fetch(`${baseUrl}/api/v1${route}`, {
    method,
    headers: {
      ...(body ? { 'Content-Type': 'application/json' } : {}),
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body: body ? JSON.stringify(body) : undefined,
  });
  const text = await response.text();
  if (!response.ok) throw new Error(`${method} ${route} returned ${response.status}: ${text.slice(0, 500)}`);
  return unwrapJSON(text);
}

async function fetchOK(url) {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`${url} returned ${response.status}`);
}

function unwrapJSON(text) {
  const parsed = text ? JSON.parse(text) : null;
  return unwrap(parsed);
}

function unwrap(value) {
  let current = value;
  for (let index = 0; index < 4; index += 1) {
    if (!current || typeof current !== 'object') return current;
    if ('data' in current && ('code' in current || 'message' in current)) {
      current = current.data;
      continue;
    }
    if ('body' in current) {
      current = current.body;
      continue;
    }
    if ('Body' in current) {
      current = current.Body;
      continue;
    }
    return current;
  }
  return current;
}

function asArray(value) {
  if (Array.isArray(value)) return value;
  if (!value || typeof value !== 'object') return [];
  for (const key of ['items', 'organizations', 'projects']) {
    if (Array.isArray(value[key])) return value[key];
  }
  return [];
}

function stringField(value, ...names) {
  if (!value || typeof value !== 'object') return '';
  for (const name of names) {
    if (value[name] !== undefined && value[name] !== null && value[name] !== '') return String(value[name]);
  }
  return '';
}

function numericField(value, ...names) {
  const parsed = Number(stringField(value, ...names));
  return Number.isFinite(parsed) ? parsed : 0;
}

function withCredentials(rawUrl, username, token) {
  const url = new URL(rawUrl);
  url.username = username;
  url.password = token;
  return url.toString();
}

function git(args, cwd, token) {
  const gitArgs = bypassGitProxy ? ['-c', 'http.proxy=', '-c', 'https.proxy=', '-c', 'credential.helper=', ...args] : args;
  return spawnCommand('git', gitArgs, cwd, token);
}

function spawnCommand(command, args, cwd, token) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      cwd,
      shell: false,
      env: {
        ...process.env,
        ...(bypassGitProxy ? gitProxyBypassEnv() : {}),
        GIT_TERMINAL_PROMPT: '0',
        GCM_INTERACTIVE: 'Never',
      },
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    let stdout = '';
    let stderr = '';
    const timer = setTimeout(() => child.kill('SIGKILL'), config.timeoutMs);
    child.stdout.on('data', (chunk) => { stdout += chunk.toString(); });
    child.stderr.on('data', (chunk) => { stderr += chunk.toString(); });
    child.on('error', (error) => {
      clearTimeout(timer);
      reject(error);
    });
    child.on('exit', (code, signal) => {
      clearTimeout(timer);
      if (code === 0) {
        resolve({ stdout, stderr });
        return;
      }
      reject(new Error(`${command} ${sanitize(args.join(' '), token)} exited code=${code ?? 'null'} signal=${signal ?? 'null'}\n${sanitize(stderr || stdout, token).slice(0, 2000)}`));
    });
  });
}

function safeRemove(rootDir, target) {
  const resolvedRoot = path.resolve(rootDir);
  const resolvedTarget = path.resolve(target);
  if (resolvedTarget !== resolvedRoot && !resolvedTarget.startsWith(resolvedRoot + path.sep)) {
    throw new Error(`refusing to remove path outside work root: ${resolvedTarget}`);
  }
  rmSync(resolvedTarget, { recursive: true, force: true });
}

function sanitize(value, token) {
  if (!token) return value;
  return value.split(token).join('<token>');
}

function shouldBypassGitProxy(...urls) {
  return urls.filter(Boolean).some((value) => {
    try {
      const host = new URL(value).hostname.toLowerCase();
      return host === '127.0.0.1' || host === 'localhost' || host === '::1' || host === '[::1]' || host === 'host.docker.internal';
    } catch {
      return false;
    }
  });
}

function gitProxyBypassEnv() {
  const entries = ['127.0.0.1', 'localhost', '::1', 'host.docker.internal'];
  const noProxy = mergeNoProxy(process.env.NO_PROXY || process.env.no_proxy || '', entries);
  return { NO_PROXY: noProxy, no_proxy: noProxy };
}

function mergeNoProxy(existing, required) {
  const values = new Set();
  for (const value of String(existing).split(',')) {
    const trimmed = value.trim();
    if (trimmed) values.add(trimmed);
  }
  for (const value of required) values.add(value);
  return [...values].join(',');
}

async function waitForBlackbox(url, project, compose) {
  for (let attempt = 0; attempt < 60; attempt += 1) {
    try {
      await fetchOK(`${url}/api/health`);
      await fetchOK(`${url}/`);
      return;
    } catch (error) {
      if (attempt === 59) {
        spawnSync('docker', ['compose', '-p', project, '-f', compose, 'logs'], { cwd: root, stdio: 'inherit' });
        throw error;
      }
      await delay(2000);
    }
  }
}

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function hasCommand(command) {
  const probe = process.platform === 'win32' ? 'where' : 'command';
  const args = process.platform === 'win32' ? [command] : ['-v', command];
  return spawnSync(probe, args, { stdio: 'ignore', shell: process.platform !== 'win32' }).status === 0;
}

function run(command, args, options) {
  const result = spawnSync(command, args, { ...options, stdio: 'inherit' });
  if (result.status !== 0) process.exit(result.status ?? 1);
}

function normalizeConfig(value) {
  const mode = String(value.mode).toLowerCase();
  const allowedModes = new Set(['mixed', 'clone', 'fetch', 'push', 'ls-remote']);
  if (!allowedModes.has(mode)) fail(`unsupported mode: ${value.mode}`);
  const normalized = {
    ...value,
    mode,
    workers: positiveInt(value.workers, 'workers'),
    iterations: positiveInt(value.iterations, 'iterations'),
    clonePercent: nonNegativeInt(value.clonePercent, 'clone-percent'),
    fetchPercent: nonNegativeInt(value.fetchPercent, 'fetch-percent'),
    pushPercent: nonNegativeInt(value.pushPercent, 'push-percent'),
    lsRemotePercent: nonNegativeInt(value.lsRemotePercent, 'ls-remote-percent'),
    timeoutMs: positiveInt(value.timeoutMs, 'timeout-ms'),
    seedFiles: nonNegativeInt(value.seedFiles, 'seed-files'),
  };
  if (normalized.mode === 'mixed') {
    const total = normalized.clonePercent + normalized.fetchPercent + normalized.pushPercent + normalized.lsRemotePercent;
    if (total <= 0) fail('mixed mode requires at least one positive operation percentage.');
  }
  return normalized;
}

function positiveInt(value, name) {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0) fail(`${name} must be a positive integer.`);
  return parsed;
}

function nonNegativeInt(value, name) {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed < 0) fail(`${name} must be a non-negative integer.`);
  return parsed;
}

function parseArgs(values) {
  const parsed = {};
  for (let index = 0; index < values.length; index += 1) {
    const arg = values[index];
    if (arg === '--help' || arg === '-h') {
      printHelp();
      process.exit(0);
    }
    if (arg === '--build') {
      parsed.build = true;
      continue;
    }
    if (arg === '--no-compose') {
      parsed.noCompose = true;
      continue;
    }
    if (arg === '--keep-workdir') {
      parsed.keepWorkdir = true;
      continue;
    }
    if (!arg.startsWith('--')) fail(`unknown argument: ${arg}`);
    const inline = arg.indexOf('=');
    const rawKey = inline >= 0 ? arg.slice(2, inline) : arg.slice(2);
    const key = toCamel(rawKey);
    const value = inline >= 0 ? arg.slice(inline + 1) : values[++index];
    if (!value) fail(`missing value for ${arg}`);
    parsed[key] = value;
  }
  return parsed;
}

function toCamel(value) {
  return value.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
}

function uniqueKey(prefix) {
  return `${prefix}-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}

function fail(message) {
  console.error(message);
  process.exit(1);
}

function printHelp() {
  console.log(`Usage: node scripts/git-client-stress.mjs [options]

Options:
  --build                     Rebuild blackbox containers before running.
  --no-compose                Do not start docker compose; run against --base-url.
  --base-url <url>            Existing Gity gateway URL, default http://127.0.0.1:18080.
  --port <port>               Blackbox web port, default 18080.
  --project <name>            Docker Compose project name, default gity-blackbox.
  --workers <count>           Concurrent git clients, default 4.
  --iterations <count>        Total measured git operations, default 24.
  --mode <mode>               mixed, clone, fetch, push, or ls-remote. Default mixed.
  --clone-percent <pct>       Mixed workload clone weight, default 20.
  --fetch-percent <pct>       Mixed workload fetch weight, default 45.
  --push-percent <pct>        Mixed workload push weight, default 25.
  --ls-remote-percent <pct>   Mixed workload ls-remote weight, default 10.
  --timeout-ms <ms>           Per git command timeout, default 60000.
  --workdir <path>            Local temp workdir, default .tmp-perf/git-client.
  --keep-workdir              Keep local git worktrees after the run.
  --seed-files <count>        Number of seed files in the initial repository, default 8.
  --project-url <url>         Reuse an existing repository URL instead of seeding by API.
  --token <token>             Token for --project-url. Password part of Basic auth.
  --token-username <name>     Basic auth username, default git-perf.
`);
}
