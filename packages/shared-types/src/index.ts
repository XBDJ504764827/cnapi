/**
 * Types shared by the web app and the API client.
 *
 * `./generated/api-schema` is produced by `pnpm gen:api` from the backend
 * OpenAPI document (docs/openapi.json) and is the single source of truth for
 * request/response shapes. Do not edit it by hand.
 */
export type { components, operations, paths } from './generated/api-schema';

export * from './api-contract';
