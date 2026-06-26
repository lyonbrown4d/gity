#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const defaults = {
  port: '18080',
  project: 'gity-blackbox',
  vus: '8',
  duration: '30s',
  p95: '1000',
  writePercent: '12',
  script: 'perf/k6/common-api.js',
  baseUrl: '',
  dockerBaseUrl: '',
  build: false,
  dockerK6: false,
  noCompose: false,
};

const args = parseArgs(process.argv.slice(2));
const config = { ...defaults, ...args };
const composeFile = path.join(root, 'docker-compose.blackbox.yaml');
const scriptPath = path.resolve(root, config.script);

if (!existsSync(scriptPath)) fail(`k6 script not found: ${scriptPath}`);
if (!config.noCompose && !existsSync(composeFile)) fail(`compose file not found: ${composeFile}`);

const hostBaseUrl = config.baseUrl || `http://127.0.0.1:${config.port}`;

if (!config.noCompose) {
  const composeArgs = ['compose', '-p', config.project, '-f', composeFile, 'up', '-d'];
  if (config.build) composeArgs.push('--build');
  run('docker', composeArgs, {
    cwd: root,
    env: { ...process.env, GITY_BLACKBOX_WEB_PORT: config.port },
  });
  await waitForBlackbox(hostBaseUrl, config.project, composeFile);
}

const env = {
  ...process.env,
  GITY_K6_BASE_URL: hostBaseUrl,
  GITY_K6_VUS: config.vus,
  GITY_K6_DURATION: config.duration,
  GITY_K6_P95_MS: config.p95,
  GITY_K6_WRITE_PERCENT: config.writePercent,
};

if (!config.dockerK6 && hasCommand('k6')) {
  console.log('Running k6 against ' + hostBaseUrl);
  run('k6', ['run', scriptPath], { cwd: root, env });
  process.exit(0);
}

const dockerBaseUrl = config.dockerBaseUrl || dockerReachableBaseUrl(hostBaseUrl, config.port);
const scriptRoot = path.dirname(scriptPath);
console.log('Running Docker k6 against ' + dockerBaseUrl);
run('docker', [
  'run',
  '--rm',
  '-i',
  '-e', `GITY_K6_BASE_URL=${dockerBaseUrl}`,
  '-e', `GITY_K6_VUS=${config.vus}`,
  '-e', `GITY_K6_DURATION=${config.duration}`,
  '-e', `GITY_K6_P95_MS=${config.p95}`,
  '-e', `GITY_K6_WRITE_PERCENT=${config.writePercent}`,
  '-v', `${scriptRoot}:/scripts:ro`,
  'grafana/k6',
  'run',
  `/scripts/${path.basename(scriptPath)}`,
], { cwd: root, env: process.env });

function dockerReachableBaseUrl(baseUrl, port) {
  try {
    const url = new URL(baseUrl);
    if (url.hostname === '127.0.0.1' || url.hostname === 'localhost' || url.hostname === '::1') {
      url.hostname = 'host.docker.internal';
      if (!url.port) url.port = port;
      return url.toString().replace(/\/$/, '');
    }
    return baseUrl;
  } catch (_) {
    return `http://host.docker.internal:${port}`;
  }
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
    if (arg === '--docker-k6') {
      parsed.dockerK6 = true;
      continue;
    }
    if (arg === '--no-compose') {
      parsed.noCompose = true;
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

async function waitForBlackbox(baseUrl, project, compose) {
  for (let attempt = 0; attempt < 60; attempt += 1) {
    try {
      await fetchOk(`${baseUrl}/api/health`);
      await fetchOk(`${baseUrl}/`);
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

async function fetchOk(url) {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`${url} returned ${response.status}`);
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

function fail(message) {
  console.error(message);
  process.exit(1);
}

function printHelp() {
  console.log(`Usage: node scripts/k6-blackbox.mjs [options]

Options:
  --build                   Rebuild blackbox containers before running.
  --docker-k6               Force grafana/k6 Docker image instead of local k6.
  --no-compose              Do not start docker compose; run against --base-url.
  --base-url <url>          Existing Gity gateway URL.
  --docker-base-url <url>   Docker-reachable gateway URL, default rewrites localhost to host.docker.internal.
  --port <port>             Blackbox web port, default 18080.
  --project <name>          Docker Compose project name, default gity-blackbox.
  --vus <count>             k6 constant VUs, default 8.
  --duration <duration>     k6 duration, default 30s.
  --p95 <ms>                p95 threshold in milliseconds, default 1000.
  --write-percent <pct>     Light write iteration percentage, default 12.
  --script <path>           k6 script path, default perf/k6/common-api.js.
`);
}
