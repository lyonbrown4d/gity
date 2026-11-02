import crypto from 'k6/crypto';
import encoding from 'k6/encoding';
import { api, authenticate, sha } from './git-support.js';
import { int64, stringField, uniqueKey, unwrapJSON } from './api-support.js';

export function seed(vus, sharedBranch) {
  const marker = uniqueKey('k6-git');
  const data = { marker, token: '', outsiderToken: '', projectID: '', projectPath: '' };
  try {
    const account = authenticate('/auth/register', {
      username: `${marker}-owner`, password: encoding.b64encode(crypto.randomBytes(32), 'rawurl'),
    });
    data.token = String(account.token);
    const outsider = authenticate('/auth/register', {
      username: `${marker}-outsider`, password: encoding.b64encode(crypto.randomBytes(32), 'rawurl'),
    });
    data.outsiderToken = String(outsider.token);
    const project = unwrapJSON(api('POST', '/projects', data.token, 'POST setup/project', {
      organization_id: int64(account.organization_id), key: marker, name: `Git API load ${marker}`,
      visibility: 'private', default_branch: 'main',
    }));
    data.projectID = stringField(project, 'id');
    data.projectPath = stringField(project, 'full_path');
    if (!/^\d+$/.test(data.projectID) || !data.projectPath) throw new Error('isolated Git project fields missing');
    commit(data, 'README.md', `# ${marker}\nGit API performance fixture.\n`);
    // 768 KiB random bytes become exactly 1 MiB ASCII. No CI YAML is created.
    const payload = encoding.b64encode(crypto.randomBytes(768 * 1024));
    data.payloadSize = payload.length;
    data.payloadSha256 = crypto.sha256(payload, 'hex');
    data.seedSha = commit(data, 'fixtures/payload.txt', payload);
    const names = sharedBranch ? ['k6-shared'] : Array.from({ length: vus }, (_, i) => `k6-vu-${i + 1}`);
    for (const name of names) {
      const branch = unwrapJSON(api('POST', `/projects/${data.projectID}/repository/branches`, data.token,
        'POST setup/branch', { name, source_ref: 'main' }));
      if (!branch || branch.name !== name || branch.last_commit_sha !== data.seedSha) throw new Error('VU branch did not preserve seed SHA');
    }
    data.sharedBranch = sharedBranch;
    return data;
  } catch (error) {
    cleanup(data);
    throw error;
  }
}

function commit(data, file, content) {
  const value = unwrapJSON(api('POST', `/projects/${data.projectID}/file-commits`, data.token,
    'POST setup/file-commits', { branch_name: 'main', file_path: file, content, message: 'Seed Git API load fixture' }));
  if (!value || !sha(value.commit_sha)) throw new Error('seed file commit SHA missing');
  return value.commit_sha;
}

export function cleanup(data) {
  if (!data) return;
  if (data.projectID && data.projectPath && data.token) {
    api('DELETE', `/projects/${data.projectID}`, data.token, 'DELETE teardown/project', { confirmation: data.projectPath });
  }
  for (const token of [data.token, data.outsiderToken]) {
    if (token) api('POST', '/auth/logout', token, 'POST teardown/logout', {});
  }
}
