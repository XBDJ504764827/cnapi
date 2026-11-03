-- 0008_replays
--
-- Replay object metadata. Binary data lives in object storage (Cloudflare R2 in
-- production), never in PostgreSQL.
--
-- Source findings:
--   * GOKZ replays (`gokz-replays`): binary replay files with format version 1
--     and 2 (see playback.sp); uploads happen after a record is accepted.
--   * GlobalAPI v2: POST /records/{recordId}/replay uploads the file for a
--     record, GET /records/{recordId}/replay and
--     GET /records/replay/{replayId} return it. No metadata table is exposed.
--   * cs2kz-api: replay bytes are stored in an S3-compatible bucket; the database
--     only tracks `Records.replay_available`, short-lived `ReplayUploadKeys`
--     (record_id, upload_key, expires_at) and a cleaner table. Object keys and
--     checksums are not stored in MySQL.
--
-- Design decisions:
--   * Metadata is persisted (unlike cs2kz) so storage lifecycle, deduplication
--     and integrity checks are possible without listing the bucket.
--   * `account_id` and `map_id` are copied from the record at upload time: they
--     make retention/cleanup and per-player replay listings possible without a
--     join, and keep attribution stable if a record is later corrected.
--   * Upload tickets (short-lived signed uploads) are NOT modelled in phase 2;
--     they are part of the phase 3 upload flow and can be added as a separate
--     table without touching this one.

CREATE TABLE replays (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    record_id BIGINT NOT NULL REFERENCES records (id) ON DELETE CASCADE,
    account_id BIGINT NOT NULL REFERENCES accounts (id),
    map_id BIGINT NOT NULL REFERENCES maps (id),
    storage_provider TEXT NOT NULL DEFAULT 'r2',
    bucket TEXT NOT NULL,
    object_key TEXT NOT NULL,
    content_type TEXT NOT NULL DEFAULT 'application/octet-stream',
    size_bytes BIGINT NOT NULL,
    checksum_sha256 BYTEA NOT NULL,
    replay_format_version SMALLINT,
    status TEXT NOT NULL DEFAULT 'pending',
    uploaded_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ck_replays_provider CHECK (storage_provider IN ('r2', 's3', 'local')),
    CONSTRAINT ck_replays_bucket CHECK (char_length(bucket) BETWEEN 1 AND 128),
    CONSTRAINT ck_replays_object_key CHECK (char_length(object_key) BETWEEN 1 AND 512),
    CONSTRAINT ck_replays_size CHECK (size_bytes > 0),
    CONSTRAINT ck_replays_checksum CHECK (octet_length(checksum_sha256) = 32),
    CONSTRAINT ck_replays_format CHECK (replay_format_version IS NULL OR replay_format_version > 0),
    CONSTRAINT ck_replays_status CHECK (status IN ('pending', 'available', 'failed', 'deleted')),
    CONSTRAINT ck_replays_uploaded_at CHECK (uploaded_at IS NULL OR uploaded_at >= created_at),
    CONSTRAINT ck_replays_expiry CHECK (expires_at IS NULL OR expires_at > created_at)
);

COMMENT ON TABLE replays IS 'Replay object metadata; bytes are stored in R2/S3 and referenced by object_key';
COMMENT ON COLUMN replays.object_key IS 'Object key inside the bucket (R2 object key in production)';
COMMENT ON COLUMN replays.checksum_sha256 IS 'SHA-256 of the stored file for integrity verification (32 bytes)';
COMMENT ON COLUMN replays.replay_format_version IS 'GOKZ replay format version observed in the file header (1 or 2)';
COMMENT ON COLUMN replays.expires_at IS 'Retention deadline for cleanup jobs; NULL = keep indefinitely';
COMMENT ON COLUMN replays.status IS 'pending -> available | failed; deleted marks logical deletion before object removal';

CREATE UNIQUE INDEX uq_replays_object ON replays (bucket, object_key);
CREATE INDEX idx_replays_record ON replays (record_id, created_at DESC);
CREATE INDEX idx_replays_account ON replays (account_id, created_at DESC);
CREATE INDEX idx_replays_map ON replays (map_id, created_at DESC);
CREATE INDEX idx_replays_retention ON replays (expires_at)
    WHERE deleted_at IS NULL AND status = 'available' AND expires_at IS NOT NULL;

CREATE TRIGGER trg_replays_updated_at
    BEFORE UPDATE ON replays
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
