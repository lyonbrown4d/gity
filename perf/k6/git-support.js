import http from 'k6/http';
import { check } from 'k6';
import { Counter, Gauge, Rate, Trend } from 'k6/metrics';
import { request, unwrapJSON } from './api-support.js';

const endpoints = ['branches', 'commits', 'tree', 'blob', 'file_commits', 'write_verify', 'forbidden'];
const metrics = {};
export const unexpectedFailures = new Rate('git_api_unexpected_failure');
export const expectedConflicts = new Counter('git_api_expected_conflicts');
export const expectedForbidden = new Counter('git_api_expected_forbidden');
export const completedWrites = new Counter('git_api_completed_writes');
export const measurementStarted = new Gauge('git_api_measurement_started_unix_seconds');
export const measurementFinished = new Gauge('git_api_measurement_finished_unix_seconds');
for (const endpoint of endpoints) {
  metrics[endpoint] = {
    duration: new Trend(`git_api_${endpoint}_duration`, true),
    failure: new Rate(`git_api_${endpoint}_unexpected_failure`),
    requests: new Counter(`git_api_${endpoint}_requests`),
    successes: new Counter(`git_api_${endpoint}_successes`),
  };
}

export function initializeCounters() {
  expectedConflicts.add(0);
  expectedForbidden.add(0);
  completedWrites.add(0);
  for (const endpoint of endpoints) {
    metrics[endpoint].requests.add(0);
    metrics[endpoint].successes.add(0);
  }
}

// Only this script's JS runtime changes the callback. A matching custom rate
// still catches an unexpected 403/409 on an ordinary success-only endpoint.
http.setResponseCallback(http.expectedStatuses(200, 201, 204, 403, 409));

export function api(method, path, token, name, body = null, statuses = [200]) {
  return request(method, path, token, name, body, statuses);
}

// Authentication deliberately avoids the shared debug-body path. Session and
// password values are never printed, including a malformed success response.
export function authenticate(path, body) {
  const base = (__ENV.GITY_K6_BASE_URL || __ENV.K6_BASE_URL || 'http://127.0.0.1:18080').replace(/\/+$/, '');
  const response = http.post(`${base}/api/v1${path}`, JSON.stringify(body), {
    headers: { 'Content-Type': 'application/json' }, tags: { name: `POST ${path}` },
    responseCallback: http.expectedStatuses(200),
  });
  const ok = response.status === 200;
  check(response, { 'temporary account registration succeeded': () => ok });
  if (!ok) throw new Error(`temporary account registration failed (HTTP ${response.status})`);
  const value = unwrapJSON(response);
  if (!value || !value.token || !value.organization_id) throw new Error('temporary account session fields missing');
  return value;
}

export function measured(endpoint, method, path, token, body, validate, allowConflict = false) {
  const statuses = endpoint === 'forbidden' ? [403] : allowConflict ? [200, 409] : [200];
  const response = api(method, path, token, `${method} git/${endpoint}`, body, statuses);
  const expectedConflict = allowConflict && response.status === 409;
  const expectedDenial = endpoint === 'forbidden' && response.status === 403;
  let valid = statuses.indexOf(response.status) >= 0;
  if (valid && !expectedConflict && !expectedDenial) {
    try { valid = validate(unwrapJSON(response)); } catch (_) { valid = false; }
  }
  metrics[endpoint].duration.add(response.timings.duration);
  metrics[endpoint].failure.add(!valid);
  metrics[endpoint].requests.add(1);
  if (valid && !expectedConflict && !expectedDenial) metrics[endpoint].successes.add(1);
  unexpectedFailures.add(!valid);
  if (expectedConflict) expectedConflicts.add(1);
  if (expectedDenial) expectedForbidden.add(1);
  check(response, { [`git/${endpoint} contract`]: () => valid });
  return { response, value: valid ? unwrapJSON(response) : null, valid, expectedConflict };
}

export function thresholds(p95, writePercent) {
  const values = { checks: ['rate==1'], git_api_unexpected_failure: ['rate==0'], http_req_failed: ['rate==0'] };
  for (const endpoint of endpoints) {
    values[`git_api_${endpoint}_duration`] = [`p(95)<${p95}`];
  }
  if (writePercent > 0) values.git_api_completed_writes = ['count>0'];
  return values;
}

export function sha(value) { return typeof value === 'string' && /^[0-9a-f]{40}$/.test(value); }
