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

/* Public portal payloads */
export type TimeType = components['schemas']['TimeType'];
export type PublicStatisticsPayload = components['schemas']['PublicStatistics'];
export type PublicModePayload = components['schemas']['PublicMode'];
export type PublicModesPayload = components['schemas']['PublicModesResponse'];
export type LeaderboardPayload = components['schemas']['LeaderboardResponse'];
export type LeaderboardEntryPayload = components['schemas']['LeaderboardEntry'];
export type RecordSummaryPayload = components['schemas']['RecordSummary'];
export type RecentRecordsPayload = components['schemas']['RecentRecordsResponse'];
export type RecordListPayload = components['schemas']['RecordListResponse'];
export type WorldRecordPayload = components['schemas']['WorldRecordEntry'];
export type ActivePlayersPayload = components['schemas']['ActivePlayersResponse'];
export type PlayerSummaryPayload = components['schemas']['PlayerSummary'];
export type PlayerListPayload = components['schemas']['PlayerListResponse'];
export type PlayerDetailPayload = components['schemas']['PlayerDetailResponse'];
export type PlayerProfilePayload = components['schemas']['PlayerProfileDto'];
export type PlayerOverallPayload = components['schemas']['PlayerOverallStats'];
export type PlayerModeStatsPayload = components['schemas']['PlayerModeStatsDto'];
export type TierProgressPayload = components['schemas']['TierProgressDto'];
export type PlayerMapProgressPayload = components['schemas']['PlayerMapProgress'];
export type PlayerMapsPayload = components['schemas']['PlayerMapsResponse'];
export type PlayerRecordsPayload = components['schemas']['PlayerRecordsResponse'];
export type ComparePayload = components['schemas']['CompareResponse'];
export type CompareMapPayload = components['schemas']['CompareMapRow'];
export type CompareBestPayload = components['schemas']['CompareBest'];
export type CompareTierPayload = components['schemas']['CompareTier'];
export type MapSummaryPayload = components['schemas']['MapSummary'];
export type MapListPayload = components['schemas']['MapListResponse'];
export type MapDetailPayload = components['schemas']['MapDetailResponse'];
export type MapStatisticsPayload = components['schemas']['MapStatistics'];
export type MapLeaderboardPayload = components['schemas']['MapLeaderboardResponse'];
export type MapCoursePayload = components['schemas']['MapCourseDto'];
export type ServerSummaryPayload = components['schemas']['ServerSummary'];
export type ServerListPayload = components['schemas']['ServerListResponse'];
export type SearchPayload = components['schemas']['SearchResponse'];
