import type { ApiEnvelope, ApiErrorEnvelope } from '@cngokz/shared-types';

import { ApiClientError } from './errors';

/**
 * Structural view of an openapi-fetch result for endpoints that answer with the
 * unified envelope. Both success and error payloads are optional because
 * openapi-fetch discriminates them by HTTP status.
 */
export interface EnvelopeResult<T> {
  data?: ApiEnvelope<T>;
  error?: ApiErrorEnvelope;
  response: Response;
}

/** Unwraps `{ success, data }` or throws a normalized {@link ApiClientError}. */
export function unwrapResponse<T>(result: EnvelopeResult<T>): T {
  const { data, error, response } = result;

  if (data?.success) {
    return data.data;
  }

  if (error) {
    throw ApiClientError.fromEnvelope(error, response.status);
  }

  throw new ApiClientError(`Unexpected API response (HTTP ${response.status})`, response.status, 'unexpected_response');
}

/** Same as {@link unwrapResponse} but for promises, mapping transport errors. */
export async function request<T>(pending: Promise<EnvelopeResult<T>>): Promise<T> {
  let result: EnvelopeResult<T>;
  try {
    result = await pending;
  } catch (cause) {
    throw new ApiClientError('Unable to reach the API', 0, 'network_error', undefined, cause);
  }
  return unwrapResponse(result);
}
