import http from 'k6/http';
import { check, group, sleep } from 'k6';

const BASE_URL = (__ENV.GITY_K6_BASE_URL || __ENV.K6_BASE_URL || 'http://127.0.0.1:18080').replace(/\/+$/, '');
const API_URL = `${BASE_URL}/api/v1`;
const VUS = Number(__ENV.GITY_K6_VUS || __ENV.K6_VUS || 8);
const DURATION = __ENV.GITY_K6_DURATION || __ENV.K6_DURATION || '30s';
const P95_MS = Number(__ENV.GITY_K6_P95_MS || __ENV.K6_P95_MS || 1000);
const WRITE_PERCENT = Number(__ENV.GITY_K6_WRITE_PERCENT || __ENV.K6_WRITE_PERCENT || 12);
const DEBUG_FAILURES = (__ENV.GITY_K6_DEBUG_FAILURES || __ENV.K6_DEBUG_FAILURES || '').toLowerCase() === 'true';

const CI_CONFIG = `pipeline {
  name = "perf"
}

stage smoke {
  tags = ["linux"]
  run {
    shell("echo perf")
  }
}
`;

export const options = {
  scenarios: {
    common_api: {
      executor: 'constant-vus',
      vus: VUS,
      duration: DURATION,
      gracefulStop: '10s',
    },
  },
  thresholds: {
    checks: ['rate>0.95'],
    http_req_failed: ['rate<0.05'],
    http_req_duration: [`p(95)<${P95_MS}`, `p(99)<${Math.max(P95_MS * 2, 1500)}`],
  },
  userAgent: 'gity-k6-common-api/1.0',
};

export function setup() {
  const marker = uniqueKey('perf');
  expect(http.get(`${BASE_URL}/api/health`, { tags: { name: 'GET /api/health' } }), 'GET /api/health', [200]);

  const session = unwrapJSON(request('POST', '/auth/login', null, 'POST /api/v1/auth/login', {
    username: `${marker}-owner`,
    password: 'password',
  }));
  const token = stringField(session, 'token');
  const username = stringField(session, 'username') || `${marker}-owner`;
  const userID = numericField(session, 'user_id', 'id');
  if (!token || userID <= 0) {
    throw new Error(`login did not return token/user_id: ${JSON.stringify(session)}`);
  }

  let organization = asArray(unwrapJSON(request('GET', '/orgs', token, 'GET /api/v1/orgs')))[0];
  if (!organization) {
    organization = unwrapJSON(request('POST', '/orgs', token, 'POST /api/v1/orgs', {
      key: `${marker}-org`,
      path_key: `${marker}-org`,
      owner_user_id: userID,
      name: `Perf ${marker}`,
      description: 'Created by k6 performance setup',
      visibility: 'private',
    }));
  }
  const organizationID = stringField(organization, 'id');
  if (!organizationID) {
    throw new Error(`organization was not available: ${JSON.stringify(organization)}`);
  }

  const project = unwrapJSON(request('POST', '/projects', token, 'POST /api/v1/projects', {
    organization_id: organizationID,
    key: `${marker}-project`,
    path_key: `${marker}-project`,
    name: `Perf ${marker}`,
    description: 'Project created by k6 performance setup',
    visibility: 'private',
    default_branch: 'main',
  }));
  const projectID = stringField(project, 'id');
  const defaultBranch = stringField(project, 'default_branch') || 'main';
  if (!projectID) {
    throw new Error(`project was not created: ${JSON.stringify(project)}`);
  }


  createFile(token, projectID, defaultBranch, 'README.md', `# ${marker}\n\nCreated by k6.\n`, username);
  createFile(token, projectID, defaultBranch, 'src/app.go', 'package main\n\nfunc main() {}\n', username);

  const issue = unwrapJSON(request('POST', `/projects/${projectID}/issues`, token, 'POST /api/v1/projects/:id/issues', {
    title: `Perf issue ${marker}`,
    description: 'Issue seeded by k6 performance setup',
  }));
  const issueIID = numericField(issue, 'number', 'iid') || 1;
  request('POST', `/projects/${projectID}/issues/${issueIID}/comments`, token, 'POST /api/v1/projects/:id/issues/:iid/comments', {
    body: 'Seed comment from k6 setup',
  });

  request('PATCH', `/projects/${projectID}/issues/${issueIID}/labels`, token, 'PATCH /api/v1/projects/:id/issues/:iid/labels', {
    labels: [{ name: 'performance', color: '#0ea5e9' }],
  });

  const mrBranch = `${marker}-feature`;
  request('POST', `/projects/${projectID}/repository/branches`, token, 'POST /api/v1/projects/:id/repository/branches', {
    name: mrBranch,
    source_ref: defaultBranch,
  });
  createFile(token, projectID, mrBranch, 'src/feature.go', 'package main\n\nfunc feature() string { return "perf" }\n', username);
  const mergeRequest = unwrapJSON(request('POST', `/projects/${projectID}/merge-requests`, token, 'POST /api/v1/projects/:id/merge-requests', {
    title: `Perf MR ${marker}`,
    description: 'Merge request seeded by k6 performance setup',
    source_branch: mrBranch,
    target_branch: defaultBranch,
    author_user_id: userID,
  }));
  const mergeIID = numericField(mergeRequest, 'iid', 'number') || 1;
  request('POST', `/projects/${projectID}/merge-requests/${mergeIID}/comments`, token, 'POST /api/v1/projects/:id/merge-requests/:iid/comments', {
    body: 'Seed MR comment from k6 setup',
  });


  request('POST', `/projects/${projectID}/wiki/pages`, token, 'POST /api/v1/projects/:id/wiki/pages', {
    slug: 'home',
    title: 'Home',
    content: `# ${marker}\n\nWiki page seeded by k6.\n`,
    format: 'markdown',
    author_user_id: userID,
  });

  const packageFile = unwrapJSON(request('POST', `/projects/${projectID}/packages/files`, token, 'POST /api/v1/projects/:id/packages/files', {
    type: 'generic',
    name: `${marker}-generic`,
    version: '0.1.0',
    file_name: 'artifact.txt',
    file_path: 'artifact.txt',
    content_type: 'text/plain',
    content_base64: 'cGVyZiBhcnRpZmFjdAo=',
  }));
  const packageFileID = stringField(packageFile, 'id');
  request('PUT', `/projects/${projectID}/packages/generic/${marker}-binary/0.1.0/data.txt`, token, 'PUT /api/v1/projects/:id/packages/generic/:name/:version/:file', 'generic package payload', [200, 201], {
    'Content-Type': 'application/octet-stream',
  });

  request('PATCH', `/projects/${projectID}/ci/variables`, token, 'PATCH /api/v1/projects/:id/ci/variables', {
    key: 'PERF_TOKEN',
    value: 'masked-secret',
    masked: true,
    protected: false,
  });

  const runnerRegistration = unwrapJSON(request('POST', `/projects/${projectID}/runners`, token, 'POST /api/v1/projects/:id/runners', {
    name: `${marker}-runner`,
    description: 'Runner seeded by k6 performance setup',
    tags: 'linux,perf',
  }));
  const runnerToken = stringField(runnerRegistration, 'token');

  const pipeline = unwrapJSON(request('POST', `/projects/${projectID}/pipelines`, token, 'POST /api/v1/projects/:id/pipelines', {
    source: 'api',
    ref_name: defaultBranch,
    commit_sha: '',
    config_source: 'inline',
    config_content: CI_CONFIG,
  }));
  const pipelineID = nestedStringField(pipeline, ['pipeline', 'id']) || stringField(pipeline, 'id');
  const pipelineJobs = asArray(nestedField(pipeline, ['jobs']));
  const pipelineJobID = pipelineJobs.length > 0 ? nestedStringField(pipelineJobs[0], ['project_job', 'id']) : '';

  const manualJob = unwrapJSON(request('POST', `/projects/${projectID}/jobs`, token, 'POST /api/v1/projects/:id/jobs', {
    kind: 'noop',
    payload: '{}',
    max_attempts: 1,
    run_after: new Date().toISOString(),
  }));
  const manualJobID = stringField(manualJob, 'id');

  const packages = asArray(unwrapJSON(request('GET', `/projects/${projectID}/packages`, token, 'GET /api/v1/projects/:id/packages')));
  const packageID = packages.length > 0 ? stringField(packages[0], 'id') : '';

  return {
    baseURL: BASE_URL,
    token,
    username,
    userID,
    organizationID,
    projectID,
    defaultBranch,
    issueIID,
    mergeIID,
    wikiSlug: 'home',
    packageID,
    packageFileID,
    genericPackageName: `${marker}-binary`,
    pipelineID,
    pipelineJobID,
    manualJobID,
    runnerToken,
  };
}

export default function (data) {
  const roll = Math.random() * 100;
  if (roll < 16) {
    group('identity and project', () => identityAndProjectReads(data));
  } else if (roll < 34) {
    group('repository', () => repositoryReads(data));
  } else if (roll < 50) {
    group('issues and merge requests', () => collaborationReads(data));
  } else if (roll < 64) {
    group('wiki and packages', () => contentRegistryReads(data));
  } else if (roll < 78) {
    group('ci and runner', () => ciRunnerReads(data));
  } else if (roll < 88) {
    group('lfs and audit', () => lfsAuditReads(data));
  } else if (roll < 88 + WRITE_PERCENT) {
    group('light writes', () => lightWrites(data));
  } else {
    expect(http.get(`${data.baseURL}/api/health`, { tags: { name: 'GET /api/health' } }), 'GET /api/health', [200]);
  }
  sleep(0.1 + Math.random() * 0.7);
}

function identityAndProjectReads(data) {
  request('GET', '/users/me', data.token, 'GET /api/v1/users/me');
  request('GET', '/users', data.token, 'GET /api/v1/users');
  request('GET', '/orgs', data.token, 'GET /api/v1/orgs');
  request('GET', `/orgs/${data.organizationID}`, data.token, 'GET /api/v1/orgs/:id');
  request('GET', `/orgs/${data.organizationID}/members`, data.token, 'GET /api/v1/orgs/:id/members');
  request('GET', '/projects', data.token, 'GET /api/v1/projects');
  request('GET', `/projects/${data.projectID}`, data.token, 'GET /api/v1/projects/:id');
  request('GET', `/projects/${data.projectID}/permissions`, data.token, 'GET /api/v1/projects/:id/permissions');
  request('GET', `/projects/${data.projectID}/members`, data.token, 'GET /api/v1/projects/:id/members');
}

function repositoryReads(data) {
  const branch = encodeURIComponent(data.defaultBranch);
  request('GET', `/projects/${data.projectID}/repository/branches`, data.token, 'GET /api/v1/projects/:id/repository/branches');
  request('GET', `/projects/${data.projectID}/repository/branch-protections`, data.token, 'GET /api/v1/projects/:id/repository/branch-protections');
  request('GET', `/projects/${data.projectID}/repository/commits?branch_name=${branch}&limit=20`, data.token, 'GET /api/v1/projects/:id/repository/commits');
  request('GET', `/projects/${data.projectID}/repository/tree?branch_name=${branch}&limit=50`, data.token, 'GET /api/v1/projects/:id/repository/tree');
  request('GET', `/projects/${data.projectID}/repository/blob?branch_name=${branch}&path=README.md`, data.token, 'GET /api/v1/projects/:id/repository/blob');
  request('GET', `/projects/${data.projectID}/repository/readme?branch_name=${branch}`, data.token, 'GET /api/v1/projects/:id/repository/readme');
  request('GET', `/projects/${data.projectID}/repository/search?branch_name=${branch}&query=perf&limit=20`, data.token, 'GET /api/v1/projects/:id/repository/search');
  request('GET', `/projects/${data.projectID}/languages?branch_name=${branch}`, data.token, 'GET /api/v1/projects/:id/languages');
}

function collaborationReads(data) {
  request('GET', `/projects/${data.projectID}/issues`, data.token, 'GET /api/v1/projects/:id/issues');
  request('GET', `/projects/${data.projectID}/issues/${data.issueIID}`, data.token, 'GET /api/v1/projects/:id/issues/:iid');
  request('GET', `/projects/${data.projectID}/issues/${data.issueIID}/comments`, data.token, 'GET /api/v1/projects/:id/issues/:iid/comments');
  request('GET', `/projects/${data.projectID}/issues/${data.issueIID}/assignees`, data.token, 'GET /api/v1/projects/:id/issues/:iid/assignees');
  request('GET', `/projects/${data.projectID}/issues/${data.issueIID}/labels`, data.token, 'GET /api/v1/projects/:id/issues/:iid/labels');
  request('GET', `/projects/${data.projectID}/merge-requests`, data.token, 'GET /api/v1/projects/:id/merge-requests');
  request('GET', `/projects/${data.projectID}/merge-requests/${data.mergeIID}`, data.token, 'GET /api/v1/projects/:id/merge-requests/:iid');
  request('GET', `/projects/${data.projectID}/merge-requests/${data.mergeIID}/diff`, data.token, 'GET /api/v1/projects/:id/merge-requests/:iid/diff');
  request('GET', `/projects/${data.projectID}/merge-requests/${data.mergeIID}/checks`, data.token, 'GET /api/v1/projects/:id/merge-requests/:iid/checks');
  request('GET', `/projects/${data.projectID}/merge-requests/${data.mergeIID}/participants`, data.token, 'GET /api/v1/projects/:id/merge-requests/:iid/participants');
  request('GET', `/projects/${data.projectID}/merge-requests/${data.mergeIID}/comments`, data.token, 'GET /api/v1/projects/:id/merge-requests/:iid/comments');
  request('GET', `/projects/${data.projectID}/merge-requests/${data.mergeIID}/approvals`, data.token, 'GET /api/v1/projects/:id/merge-requests/:iid/approvals');
  request('GET', `/projects/${data.projectID}/merge-request-approval-rules`, data.token, 'GET /api/v1/projects/:id/merge-request-approval-rules');
}

function contentRegistryReads(data) {
  request('GET', `/projects/${data.projectID}/wiki/pages`, data.token, 'GET /api/v1/projects/:id/wiki/pages');
  request('GET', `/projects/${data.projectID}/wiki/pages/${encodeURIComponent(data.wikiSlug)}`, data.token, 'GET /api/v1/projects/:id/wiki/pages/:slug');
  request('GET', `/projects/${data.projectID}/packages`, data.token, 'GET /api/v1/projects/:id/packages');
  if (data.packageID) request('GET', `/projects/${data.projectID}/packages/${data.packageID}`, data.token, 'GET /api/v1/projects/:id/packages/:package_id');
  if (data.packageFileID) {
    request('GET', `/projects/${data.projectID}/packages/files/${data.packageFileID}`, data.token, 'GET /api/v1/projects/:id/packages/files/:file_id');
    request('GET', `/projects/${data.projectID}/packages/files/${data.packageFileID}/download`, data.token, 'GET /api/v1/projects/:id/packages/files/:file_id/download');
  }
  request('GET', `/projects/${data.projectID}/packages/generic/${data.genericPackageName}/0.1.0/data.txt`, data.token, 'GET /api/v1/projects/:id/packages/generic/:name/:version/:file');
  request('GET', `/projects/${data.projectID}/packages/pypi/simple`, data.token, 'GET /api/v1/projects/:id/packages/pypi/simple');
  request('GET', `/projects/${data.projectID}/packages/nuget/index.json`, data.token, 'GET /api/v1/projects/:id/packages/nuget/index.json');
}

function ciRunnerReads(data) {
  request('GET', `/projects/${data.projectID}/pipelines`, data.token, 'GET /api/v1/projects/:id/pipelines');
  if (data.pipelineID) {
    request('GET', `/projects/${data.projectID}/pipelines/${data.pipelineID}`, data.token, 'GET /api/v1/projects/:id/pipelines/:pipeline_id');
    request('GET', `/projects/${data.projectID}/pipelines/${data.pipelineID}/jobs`, data.token, 'GET /api/v1/projects/:id/pipelines/:pipeline_id/jobs');
  }
  request('GET', `/projects/${data.projectID}/jobs`, data.token, 'GET /api/v1/projects/:id/jobs');
  const jobID = data.pipelineJobID || data.manualJobID;
  if (jobID) {
    request('GET', `/projects/${data.projectID}/jobs/${jobID}`, data.token, 'GET /api/v1/projects/:id/jobs/:job_id');
    request('GET', `/projects/${data.projectID}/jobs/${jobID}/trace?offset=0&limit=4096`, data.token, 'GET /api/v1/projects/:id/jobs/:job_id/trace');
    request('GET', `/projects/${data.projectID}/jobs/${jobID}/artifacts`, data.token, 'GET /api/v1/projects/:id/jobs/:job_id/artifacts');
  }
  request('GET', `/projects/${data.projectID}/runners`, data.token, 'GET /api/v1/projects/:id/runners');
  request('GET', `/projects/${data.projectID}/ci/variables`, data.token, 'GET /api/v1/projects/:id/ci/variables');
  if (data.runnerToken) {
    request('POST', '/runners/heartbeat', null, 'POST /api/v1/runners/heartbeat', { token: data.runnerToken });
    request('POST', '/runners/jobs/claim', null, 'POST /api/v1/runners/jobs/claim', { token: data.runnerToken, lease_seconds: 30 });
  }
}

function lfsAuditReads(data) {
  request('GET', `/projects/${data.projectID}/lfs/objects?limit=20`, data.token, 'GET /api/v1/projects/:id/lfs/objects');
  request('GET', `/projects/${data.projectID}/lfs/locks?limit=20`, data.token, 'GET /api/v1/projects/:id/lfs/locks');
  request('GET', `/projects/${data.projectID}/audit-events?limit=20`, data.token, 'GET /api/v1/projects/:id/audit-events');
}

function lightWrites(data) {
  const suffix = uniqueKey(`vu${__VU}-it${__ITER}`);
  request('POST', `/projects/${data.projectID}/issues`, data.token, 'POST /api/v1/projects/:id/issues', {
    title: `Perf write ${suffix}`,
    description: 'Issue created by k6 light write workload',
  });
  request('POST', `/projects/${data.projectID}/issues/${data.issueIID}/comments`, data.token, 'POST /api/v1/projects/:id/issues/:iid/comments', { body: `Perf comment ${suffix}` });
  request('PATCH', `/projects/${data.projectID}/wiki/pages/${encodeURIComponent(data.wikiSlug)}`, data.token, 'PATCH /api/v1/projects/:id/wiki/pages/:slug', {
    title: 'Home',
    content: `# Home\n\nUpdated by ${suffix}.\n`,
    editor_user_id: data.userID,
  });
  request('POST', `/projects/${data.projectID}/ci/lint`, data.token, 'POST /api/v1/projects/:id/ci/lint', { config_content: CI_CONFIG });
  request('POST', `/projects/${data.projectID}/lfs/locks`, data.token, 'POST /api/v1/projects/:id/lfs/locks', { path: `assets/${suffix}.bin` });
}

function createFile(token, projectID, branchName, filePath, content, username) {
  request('POST', `/projects/${projectID}/repository/files`, token, 'POST /api/v1/projects/:id/repository/files', {
    branch_name: branchName,
    path: filePath,
    content,
    message: `Add ${filePath}`,
    author_name: username,
    author_email: `${username}@local.gity`,
  });
}

function request(method, path, token, name, body, expectedStatuses = [200, 201, 204], extraHeaders = {}) {
  const requestBody = body == null ? null : typeof body === 'string' ? body : JSON.stringify(body);
  const headers = Object.assign({}, extraHeaders);
  if (!headers['Content-Type'] && body != null) headers['Content-Type'] = 'application/json';
  if (token) headers.Authorization = `Bearer ${token}`;
  const response = http.request(method, `${API_URL}${path}`, requestBody, { headers, tags: { name } });
  expect(response, name, expectedStatuses);
  return response;
}

function expect(response, name, expectedStatuses) {
  const ok = expectedStatuses.indexOf(response.status) >= 0;
  check(response, { [`${name} status`]: () => ok });
  if (!ok && DEBUG_FAILURES) console.error(`${name} failed with ${response.status}: ${String(response.body || '').slice(0, 500)}`);
}

function unwrapJSON(response) {
  return unwrap(parseJSON(response));
}

function parseJSON(response) {
  try {
    return response.json();
  } catch (_) {
    return null;
  }
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
  for (const key of ['items', 'projects', 'organizations', 'users', 'packages', 'jobs', 'pipelines', 'runners', 'variables', 'rules']) {
    if (Array.isArray(value[key])) return value[key];
  }
  return [];
}

function nestedField(value, path) {
  let current = value;
  for (const key of path) {
    if (!current || typeof current !== 'object') return undefined;
    current = current[key];
  }
  return current;
}

function nestedStringField(value, path) {
  const item = nestedField(value, path);
  return item === undefined || item === null ? '' : String(item);
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

function uniqueKey(prefix) {
  return `${prefix}-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}
