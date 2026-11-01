import http from 'k6/http';
import { group, sleep } from 'k6';
import { request, expect, unwrapJSON, asArray, nestedStringField, nestedField, stringField, numericField, uniqueKey, int64 } from './api-support.js';
import { createFile, identityAndProjectReads, repositoryReads, collaborationReads, contentRegistryReads, ciRunnerReads, lfsAuditReads, lightWrites } from './api-groups.js';
import { CI_CONFIG } from './api-config.js';

const BASE_URL = (__ENV.GITY_K6_BASE_URL || __ENV.K6_BASE_URL || 'http://127.0.0.1:18080').replace(/\/+$/, '');
const VUS = Number(__ENV.GITY_K6_VUS || __ENV.K6_VUS || 8);
const DURATION = __ENV.GITY_K6_DURATION || __ENV.K6_DURATION || '30s';
const P95_MS = Number(__ENV.GITY_K6_P95_MS || __ENV.K6_P95_MS || 1000);
const WRITE_PERCENT = Number(__ENV.GITY_K6_WRITE_PERCENT || __ENV.K6_WRITE_PERCENT || 12);


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
  const userID = stringField(session, 'user_id', 'id');
  if (!token || !/^\d+$/.test(userID)) {
    throw new Error(`login did not return token/user_id: ${JSON.stringify(session)}`);
  }

  let organization = asArray(unwrapJSON(request('GET', '/orgs', token, 'GET /api/v1/orgs')))[0];
  if (!organization) {
    organization = unwrapJSON(request('POST', '/orgs', token, 'POST /api/v1/orgs', {
      key: `${marker}-org`,
      path_key: `${marker}-org`,
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
    organization_id: int64(organizationID),
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
    author_user_id: int64(userID),
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
    author_user_id: int64(userID),
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
