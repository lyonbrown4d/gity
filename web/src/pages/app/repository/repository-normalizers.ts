import { z } from "zod";

export type RawRecord = Record<string, unknown>;

export const rawRecordSchema = z.record(z.string(), z.unknown());
export const rawRecordArraySchema = z.array(rawRecordSchema);

const stringValueSchema = z.preprocess((value) => (value === undefined || value === null ? "" : String(value)), z.string());
const optionalStringValueSchema = stringValueSchema.transform((value) => {
  const normalized = value.trim();
  return !normalized || normalized === "0001-01-01T00:00:00Z" ? null : normalized;
});
const numberValueSchema = z.preprocess((value) => {
  if (typeof value === "number" && Number.isFinite(value)) {
    return value;
  }
  const parsed = Number.parseInt(value === undefined || value === null ? "" : String(value), 10);
  return Number.isFinite(parsed) ? parsed : 0;
}, z.number());
const booleanValueSchema = z.preprocess((value) => {
  if (value === true || value === 1) {
    return true;
  }
  const normalized = value === undefined || value === null ? "" : String(value).trim().toLowerCase();
  return normalized === "1" || normalized === "true";
}, z.boolean());

export const isRecord = (value: unknown): value is RawRecord =>
  rawRecordSchema.safeParse(value).success;

export const resolveRecordArray = (value: unknown): RawRecord[] => {
  const result = rawRecordArraySchema.safeParse(value);
  return result.success ? result.data : [];
};

export const resolveBody = (payload: unknown): unknown => {
  if (!isRecord(payload)) {
    return payload;
  }
  return payload.body ?? payload.Body ?? payload;
};

export const resolveArrayPayload = <T = unknown>(payload: unknown): T[] => {
  const candidates = [payload, resolveBody(payload)];
  for (const candidate of candidates) {
    if (Array.isArray(candidate)) {
      return candidate as T[];
    }
    if (!isRecord(candidate)) {
      continue;
    }
    for (const key of ["data", "items", "body", "Body"]) {
      const nested = candidate[key];
      if (Array.isArray(nested)) {
        return nested as T[];
      }
    }
  }
  return [];
};

export const normalizeString = (value: unknown): string => stringValueSchema.parse(value);

export const normalizeOptionalString = (value: unknown): string | null => optionalStringValueSchema.parse(value);

export const normalizeNumber = (value: unknown): number => numberValueSchema.parse(value);

export const normalizeBoolean = (value: unknown): boolean => booleanValueSchema.parse(value);

export const normalizeStringArray = (value: unknown): string[] => {
  if (Array.isArray(value)) {
    return value.map((item) => normalizeString(item)).filter(Boolean);
  }
  const raw = normalizeString(value).trim();
  if (!raw) {
    return [];
  }
  try {
    const decoded: unknown = JSON.parse(raw);
    return Array.isArray(decoded) ? decoded.map((item) => normalizeString(item)).filter(Boolean) : [raw];
  } catch {
    return [raw];
  }
};

