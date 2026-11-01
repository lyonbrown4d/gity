import { request, asArray, stringField, uniqueKey, int64 } from './api-support.js';
import { CI_CONFIG } from './api-config.js';

export function identityAndProjectReads(data) {
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

export function repositoryReads(data) {
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

export function collaborationReads(data) {
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

export function contentRegistryReads(data) {
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

export function ciRunnerReads(data) {
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

export function lfsAuditReads(data) {
  request('GET', `/projects/${data.projectID}/lfs/objects?limit=20`, data.token, 'GET /api/v1/projects/:id/lfs/objects');
  request('GET', `/projects/${data.projectID}/lfs/locks?limit=20`, data.token, 'GET /api/v1/projects/:id/lfs/locks');
  request('GET', `/projects/${data.projectID}/audit-events?limit=20`, data.token, 'GET /api/v1/projects/:id/audit-events');
}

export function lightWrites(data) {
  const suffix = uniqueKey(`vu${__VU}-it${__ITER}`);
  request('POST', `/projects/${data.projectID}/issues`, data.token, 'POST /api/v1/projects/:id/issues', {
    title: `Perf write ${suffix}`,
    description: 'Issue created by k6 light write workload',
  });
  request('POST', `/projects/${data.projectID}/issues/${data.issueIID}/comments`, data.token, 'POST /api/v1/projects/:id/issues/:iid/comments', { body: `Perf comment ${suffix}` });
  request('PATCH', `/projects/${data.projectID}/wiki/pages/${encodeURIComponent(data.wikiSlug)}`, data.token, 'PATCH /api/v1/projects/:id/wiki/pages/:slug', {
    title: 'Home',
    content: `# Home\n\nUpdated by ${suffix}.\n`,
    editor_user_id: int64(data.userID),
  });
  request('POST', `/projects/${data.projectID}/ci/lint`, data.token, 'POST /api/v1/projects/:id/ci/lint', { config_content: CI_CONFIG });
  request('POST', `/projects/${data.projectID}/lfs/locks`, data.token, 'POST /api/v1/projects/:id/lfs/locks', { path: `assets/${suffix}.bin` });
}

export function createFile(token, projectID, branchName, filePath, content, username) {
  request('POST', `/projects/${projectID}/repository/files`, token, 'POST /api/v1/projects/:id/repository/files', {
    branch_name: branchName,
    path: filePath,
    content,
    message: `Add ${filePath}`,
    author_name: username,
    author_email: `${username}@local.gity`,
  });
}
