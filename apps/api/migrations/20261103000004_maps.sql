-- 0004_maps
--
-- Map identity, versions, courses (stages), supported modes and per-mode tiers.
--
-- Source findings:
--   * GOKZ local DB: Maps(MapID, Name UNIQUE, LastPlayed, Created) and
--     MapCourses(MapID, Course) with UNIQUE(MapID, Course).
--   * GlobalAPI v2 APIMap: id, name, filesize, validated, difficulty (tier),
--     created_on, updated_on, approved_by_steamid64. Difficulty is per map.
--   * cs2kz Maps: workshop_id, name, description, state (-1 invalid / 0 testing /
--     1 approved), vpk_checksum BINARY(16), approved_at. Courses are separate
--     rows; CourseFilters hold mode + nub_tier/pro_tier + filter state.
--
-- Design decisions:
--   * `course_tiers` merges cs2kz per-mode tiers with GlobalAPI's per-map
--     difficulty and keeps NUB/PRO tiers separate, because GP (GOKZ/GlobalAPI)
--     ranks NUB and PRO runs independently.
--   * `map_versions` stores the content identity used by validation: GlobalAPI
--     validates by file size, cs2kz by a 16-byte VPK checksum. Both are kept as
--     optional columns instead of guessing one canonical algorithm.

CREATE TABLE maps (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL,
    workshop_id BIGINT,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'in_testing',
    approved_at TIMESTAMPTZ,
    approved_by_account_id BIGINT REFERENCES accounts (id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    CONSTRAINT ck_maps_name CHECK (char_length(name) BETWEEN 1 AND 128),
    CONSTRAINT ck_maps_status CHECK (status IN ('in_testing', 'approved', 'disabled')),
    CONSTRAINT ck_maps_workshop_id CHECK (workshop_id IS NULL OR workshop_id > 0)
);

COMMENT ON TABLE maps IS 'KZ maps (CS:GO and CS2), soft deleted via deleted_at';
COMMENT ON COLUMN maps.name IS 'Map file name without .bsp, e.g. kz_bhop_example';
COMMENT ON COLUMN maps.workshop_id IS 'Steam Workshop published file id (CS2); NULL for CS:GO-only legacy maps';
COMMENT ON COLUMN maps.status IS 'in_testing|approved|disabled — merges cs2kz MapState (-1/0/1) and GlobalAPI validated flag';

CREATE UNIQUE INDEX uq_maps_name_active ON maps (lower(name)) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX uq_maps_workshop_id ON maps (workshop_id) WHERE workshop_id IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX idx_maps_status ON maps (status, updated_at DESC) WHERE deleted_at IS NULL;

CREATE TRIGGER trg_maps_updated_at
    BEFORE UPDATE ON maps
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ---------------------------------------------------------------------------
-- map_versions
-- ---------------------------------------------------------------------------
CREATE TABLE map_versions (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    map_id BIGINT NOT NULL REFERENCES maps (id) ON DELETE CASCADE,
    version_label TEXT,
    file_size_bytes BIGINT,
    checksum BYTEA,
    checksum_algorithm TEXT,
    is_current BOOLEAN NOT NULL DEFAULT FALSE,
    created_by_account_id BIGINT REFERENCES accounts (id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ck_map_versions_file_size CHECK (file_size_bytes IS NULL OR file_size_bytes > 0),
    CONSTRAINT ck_map_versions_checksum CHECK (
        (checksum IS NULL AND checksum_algorithm IS NULL)
        OR (checksum IS NOT NULL AND checksum_algorithm IS NOT NULL)
    )
);

COMMENT ON TABLE map_versions IS 'Content versions of a map used for validation and change tracking';
COMMENT ON COLUMN map_versions.file_size_bytes IS 'GlobalAPI APIMap.filesize (bytes) and GOKZ local file size validation';
COMMENT ON COLUMN map_versions.checksum IS 'Raw content checksum; cs2kz vpk_checksum is 16 bytes, algorithm not documented upstream';
COMMENT ON COLUMN map_versions.checksum_algorithm IS 'Name of the checksum algorithm, e.g. md5 or sha256';

CREATE UNIQUE INDEX uq_map_versions_current ON map_versions (map_id) WHERE is_current;
CREATE UNIQUE INDEX uq_map_versions_checksum
    ON map_versions (map_id, checksum_algorithm, checksum)
    WHERE checksum IS NOT NULL;
CREATE INDEX idx_map_versions_map ON map_versions (map_id, created_at DESC);

-- ---------------------------------------------------------------------------
-- map_courses
-- ---------------------------------------------------------------------------
CREATE TABLE map_courses (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    map_id BIGINT NOT NULL REFERENCES maps (id) ON DELETE CASCADE,
    stage SMALLINT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ck_map_courses_stage CHECK (stage >= 0 AND stage < 100),
    CONSTRAINT ck_map_courses_name CHECK (char_length(name) BETWEEN 1 AND 64),
    CONSTRAINT uq_map_courses_map_stage UNIQUE (map_id, stage),
    -- Target of the composite foreign key on records: guarantees that the
    -- record map_id always equals the map of the referenced course.
    CONSTRAINT uq_map_courses_id_map UNIQUE (id, map_id)
);

COMMENT ON TABLE map_courses IS 'Courses/stages of a map: 0 = main course, 1..99 = bonus stages (GOKZ naming)';
COMMENT ON COLUMN map_courses.stage IS 'GOKZ course number (0 = main, 1.. = bonus); GOKZ_MAX_COURSES = 100';
COMMENT ON COLUMN map_courses.name IS 'Display name; cs2kz course names are free-form, GOKZ derives them from the stage';

CREATE INDEX idx_map_courses_map ON map_courses (map_id, stage);

CREATE TRIGGER trg_map_courses_updated_at
    BEFORE UPDATE ON map_courses
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ---------------------------------------------------------------------------
-- map_modes
-- ---------------------------------------------------------------------------
-- Explicitly records which movement modes a map supports. Absence means "not
-- known / not supported"; tier data alone is not used for this decision because
-- unranked courses exist too.
CREATE TABLE map_modes (
    map_id BIGINT NOT NULL REFERENCES maps (id) ON DELETE CASCADE,
    mode_id SMALLINT NOT NULL REFERENCES movement_modes (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (map_id, mode_id)
);

COMMENT ON TABLE map_modes IS 'Movement modes supported by a map (presence = supported)';

CREATE INDEX idx_map_modes_mode ON map_modes (mode_id, map_id);

-- ---------------------------------------------------------------------------
-- course_tiers
-- ---------------------------------------------------------------------------
CREATE TABLE course_tiers (
    course_id BIGINT NOT NULL REFERENCES map_courses (id) ON DELETE CASCADE,
    mode_id SMALLINT NOT NULL REFERENCES movement_modes (id) ON DELETE CASCADE,
    nub_tier SMALLINT,
    pro_tier SMALLINT,
    status TEXT NOT NULL DEFAULT 'unranked',
    notes TEXT,
    reviewed_by_account_id BIGINT REFERENCES accounts (id) ON DELETE SET NULL,
    reviewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (course_id, mode_id),
    CONSTRAINT ck_course_tiers_nub_tier CHECK (nub_tier IS NULL OR nub_tier BETWEEN 1 AND 10),
    CONSTRAINT ck_course_tiers_pro_tier CHECK (pro_tier IS NULL OR pro_tier BETWEEN 1 AND 10),
    CONSTRAINT ck_course_tiers_status CHECK (status IN ('unranked', 'pending', 'ranked'))
);

COMMENT ON TABLE course_tiers IS 'Per-course, per-mode difficulty tiers and ranking review state';
COMMENT ON COLUMN course_tiers.nub_tier IS 'Tier for NUB leaderboard (cs2kz CourseFilters.nub_tier, 1-10)';
COMMENT ON COLUMN course_tiers.pro_tier IS 'Tier for PRO leaderboard (cs2kz CourseFilters.pro_tier, 1-10)';
COMMENT ON COLUMN course_tiers.status IS 'unranked|pending|ranked — mirrors cs2kz CourseFilterState (-1/0/1)';

CREATE INDEX idx_course_tiers_mode_tier ON course_tiers (mode_id, nub_tier, pro_tier);

CREATE TRIGGER trg_course_tiers_updated_at
    BEFORE UPDATE ON course_tiers
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
