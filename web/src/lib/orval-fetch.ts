import { requestEnvelope, type RequestOptions } from "./http-client";

/** Generated Orval functions all use the same authenticated Axios transport. */
export const customFetch = <T>(url: string, options: RequestOptions = {}): Promise<T> =>
  requestEnvelope<T>(url, options);

export default customFetch;
