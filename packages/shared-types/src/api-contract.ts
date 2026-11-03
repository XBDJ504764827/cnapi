import type { components } from './generated/api-schema';

/**
 * Envelope returned by every successful API call.
 * Mirrors `ApiResponse<T>` on the backend.
 */
export interface ApiEnvelope<T> {
  success: boolean;
  data: T;
  message: string;
}

/**
 * Envelope returned by every failed API call.
 * Mirrors `ApiErrorBody` on the backend.
 */
export interface ApiErrorEnvelope {
  /** Always `false`; typed as `boolean` because the generated schema models the field generically. */
  success: boolean;
  error: {
    code: ErrorCode;
    details?: unknown;
  };
  message: string;
}

export type ErrorCode = components['schemas']['ErrorCode'];

/** Response payloads (the part inside `data`). */
export type HealthPayload = components['schemas']['HealthResponse'];
export type LivenessPayload = components['schemas']['LivenessResponse'];
export type ReadinessPayload = components['schemas']['ReadinessResponse'];
export type DependencyStatusPayload = components['schemas']['DependencyStatusDto'];
export type SystemInfoPayload = components['schemas']['SystemInfoResponse'];
export type SystemStatusPayload = components['schemas']['SystemStatusResponse'];
export type SystemEventPayload = components['schemas']['SystemEventResponse'];
export type SystemEventsPayload = components['schemas']['SystemEventsResponse'];

/** `development` | `test` | `production`. */
export type SystemEnvironment = SystemInfoPayload['environment'];

/** Stable API version served under `/api/<version>`. */
export const API_VERSION = 'v1' as const;
