import crypto from 'k6/crypto';
import { seed, cleanup } from './git-fixture.js';
import { measured, completedWrites, initializeCounters, measurementStarted, measurementFinished, thresholds, sha } from './git-support.js';

const VUS = Number(__ENV.GITY_K6_VUS || __ENV.K6_VUS || 8);
const DURATION = __ENV.GITY_K6_DURATION || __ENV.K6_DURATION || '60s';
const P95 = Number(__ENV.GITY_K6_P95_MS || __ENV.K6_P95_MS || 1000);
const WRITE_PERCENT = Number(__ENV.GITY_K6_WRITE_PERCENT || __ENV.K6_WRITE_PERCENT || 20);
const SHARED = (__ENV.GITY_K6_CONFLICTS || '').toLowerCase() === 'true';
if (!Number.isInteger(VUS) || VUS < 1 || !Number.isInteger(WRITE_PERCENT) || WRITE_PERCENT < 0 || WRITE_PERCENT > 100
    || !Number.isFinite(P95) || P95 <= 0) throw new Error('invalid Git API load parameters');

export const options = {
  scenarios: { git_api: { executor: 'constant-vus', vus: VUS, duration: DURATION, gracefulStop: '30s' } },
  thresholds: thresholds(P95, WRITE_PERCENT),
  summaryTrendStats: ['med', 'p(95)', 'p(99)', 'min', 'max'],
  systemTags: ['status', 'method', 'name', 'scenario', 'expected_response'],
  setupTimeout: '180s', teardownTimeout: '60s', userAgent: 'gity-k6-git-api/1.0',
};

export function setup() {
  const data = seed(VUS, SHARED);
  measurementStarted.add(Date.now() / 1000);
  return data;
}
export function teardown(data) {
  measurementFinished.add(Date.now() / 1000);
  cleanup(data);
}

export default function (data) {
  if (__ITER === 0) initializeCounters();
  if (Math.random() * 100 < WRITE_PERCENT) write(data);
  else read(data, Math.floor(Math.random() * 4));
  if (__ITER % 100 === 0) read(data, 4);
}

function read(data, choice) {
  const base = `/projects/${data.projectID}/repository`;
  if (choice === 0) {
    measured('branches', 'GET', `${base}/branches`, data.token, null,
      rows => Array.isArray(rows) && rows.some(row => row.name === 'main' && row.last_commit_sha === data.seedSha)
        && rows.every(row => sha(row.last_commit_sha)));
  } else if (choice === 1) {
    measured('commits', 'GET', `${base}/commits?ref_name=main&limit=20`, data.token, null,
      rows => Array.isArray(rows) && rows.length >= 2 && rows[0].commit_sha === data.seedSha
        && rows.every(row => sha(row.commit_sha)));
  } else if (choice === 2) {
    measured('tree', 'GET', `${base}/tree?ref_name=main&path=fixtures`, data.token, null,
      rows => Array.isArray(rows) && rows.some(row => row.path === 'fixtures/payload.txt'
        && row.kind === 'blob' && row.size === data.payloadSize));
  } else if (choice === 3) {
    measured('blob', 'GET', `${base}/blob?ref_name=main&path=fixtures%2Fpayload.txt`, data.token, null,
      value => value && value.encoding === 'utf-8' && value.size === data.payloadSize
        && crypto.sha256(value.content, 'hex') === data.payloadSha256);
  } else {
    measured('forbidden', 'GET', `${base}/blob?ref_name=main&path=README.md`, data.outsiderToken, null, () => false);
  }
}

function write(data) {
  const branch = data.sharedBranch ? 'k6-shared' : `k6-vu-${__VU}`;
  const file = `load/vu-${__VU}/iteration-${__ITER}.txt`;
  const content = `vu=${__VU}; iteration=${__ITER}; marker=${data.marker}\n`;
  const result = measured('file_commits', 'POST', `/projects/${data.projectID}/file-commits`, data.token,
    { branch_name: branch, file_path: file, content, message: `Git API load VU ${__VU} iteration ${__ITER}` },
    value => value && sha(value.commit_sha) && value.branch_name === branch && value.file_path === file, data.sharedBranch);
  if (!result.valid || result.expectedConflict) return;
  // Read by returned immutable SHA, so shared-branch advancement cannot make
  // another VU's successful commit appear to contain the wrong content.
  const readback = measured('write_verify', 'GET', `/projects/${data.projectID}/repository/blob?ref_name=${result.value.commit_sha}&path=${encodeURIComponent(file)}`,
    data.token, null, value => value && value.encoding === 'utf-8' && value.content === content);
  if (readback.valid) completedWrites.add(1);
}
