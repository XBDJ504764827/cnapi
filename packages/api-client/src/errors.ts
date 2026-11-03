import type { ApiErrorEnvelope, ErrorCode } from '@cngokz/shared-types';

/** Machine readable error codes added by the client on transport failures. */
export type ClientErrorCode = ErrorCode | 'network_error' | 'unexpected_response';

/**
 * Normalized error thrown by every API client method.
 *
 * The UI can branch on `code` and use `message` for display. `details` carries
 * sanitized server details (for example readiness checks).
 */
export class ApiClientError extends Error {
  readonly status: number;
  readonly code: ClientErrorCode;
  readonly details?: unknown;

  constructor(message: string, status: number, code: ClientErrorCode, details?: unknown, cause?: unknown) {
    super(message, { cause });
    this.name = 'ApiClientError';
    this.status = status;
    this.code = code;
    this.details = details;
  }

  static fromEnvelope(envelope: ApiErrorEnvelope, status: number): ApiClientError {
    return new ApiClientError(envelope.message, status, envelope.error.code, envelope.error.details);
  }
}

export function isApiClientError(error: unknown): error is ApiClientError {
  return error instanceof ApiClientError;
}
