import http from 'k6/http';
import { check } from 'k6';

const API_URL = `${(__ENV.GITY_K6_BASE_URL || __ENV.K6_BASE_URL || 'http://127.0.0.1:18080').replace(/\/+$/, '')}/api/v1`;
const DEBUG_FAILURES = (__ENV.GITY_K6_DEBUG_FAILURES || __ENV.K6_DEBUG_FAILURES || '').toLowerCase() === 'true';

export function request(method, path, token, name, body, expectedStatuses = [200, 201, 204], extraHeaders = {}) {
  const requestBody = body == null ? null : typeof body === 'string' ? body : jsonBody(body);
  const headers = Object.assign({}, extraHeaders);
  if (!headers['Content-Type'] && body != null) headers['Content-Type'] = 'application/json';
  if (token) headers.Authorization = `Bearer ${token}`;
  const response = http.request(method, `${API_URL}${path}`, requestBody, { headers, tags: { name } });
  expect(response, name, expectedStatuses);
  return response;
}

export function int64(decimal) {
  const value = String(decimal);
  if (!/^-?\d+$/.test(value)) throw new Error('expected decimal int64 ID');
  return { __gityInt64: value };
}

function jsonBody(value) {
  return JSON.stringify(value, (_, item) => {
    if (item && typeof item === 'object' && item.__gityInt64 !== undefined) {
      return `__GITY_INT64_${item.__gityInt64}__`;
    }
    return item;
  }).replace(/"__GITY_INT64_(-?\d+)__"/g, '$1');
}

export function expect(response, name, expectedStatuses) {
  const ok = expectedStatuses.indexOf(response.status) >= 0;
  check(response, { [`${name} status`]: () => ok });
  if (!ok && DEBUG_FAILURES) console.error(`${name} failed with ${response.status}: ${String(response.body || '').slice(0, 500)}`);
}

export function unwrapJSON(response) {
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

export function asArray(value) {
  if (Array.isArray(value)) return value;
  if (!value || typeof value !== 'object') return [];
  for (const key of ['items', 'projects', 'organizations', 'users', 'packages', 'jobs', 'pipelines', 'runners', 'variables', 'rules']) {
    if (Array.isArray(value[key])) return value[key];
  }
  return [];
}

export function nestedField(value, path) {
  let current = value;
  for (const key of path) {
    if (!current || typeof current !== 'object') return undefined;
    current = current[key];
  }
  return current;
}

export function nestedStringField(value, path) {
  const item = nestedField(value, path);
  return item === undefined || item === null ? '' : String(item);
}

export function stringField(value, ...names) {
  if (!value || typeof value !== 'object') return '';
  for (const name of names) {
    if (value[name] !== undefined && value[name] !== null && value[name] !== '') return String(value[name]);
  }
  return '';
}

export function numericField(value, ...names) {
  const parsed = Number(stringField(value, ...names));
  return Number.isFinite(parsed) ? parsed : 0;
}

export function uniqueKey(prefix) {
  return `${prefix}-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}
