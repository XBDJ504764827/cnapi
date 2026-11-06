-- 0011: Points & Ranking System (phase 6)
--
-- A Filter is (map_id, course_id, mode, NUB/PRO). Filters are independent:
-- mode, course and record_type never share PB, WR, rank or points.
-- No existing table is dropped. `records` validity rules are unchanged:
-- valid = status='approved' AND deleted_at IS NULL.
-- T9 is a *historical label* on the first VNL completion, never a stored tier.

-- ---------------------------------------------------------------------------
-- Scoring modes: only KZT / SKZ / VNL participate in phase 6.
-- ---------------------------------------------------------------------------
CREATE TABLE scoring_modes (
    code TEXT PRIMARY KEY CHECK (code IN ('KZT', 'SKZ', 'VNL')),
    mode_id SMALLINT NOT NULL UNIQUE REFERENCES movement_modes (id),
    display_name TEXT NOT NULL,
    sort_order SMALLINT NOT NULL,
    t9_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO scoring_modes (code, mode_id, display_name, sort_order, t9_enabled) VALUES
    ('KZT', 3, 'KZTimer', 1, FALSE),  -- kztimer
    ('SKZ', 2, 'Simple KZ', 2, FALSE), -- simple
    ('VNL', 1, 'Vanilla', 3, TRUE);    -- vanilla

COMMENT ON TABLE scoring_modes IS 'Phase-6 scoring modes; classic is intentionally excluded';

-- ---------------------------------------------------------------------------
-- Authoritative tier per (map, course, mode). Not split by NUB/PRO.
-- ---------------------------------------------------------------------------
CREATE TABLE course_mode_tiers (
    map_id BIGINT NOT NULL REFERENCES maps (id) ON DELETE CASCADE,
    course_id BIGINT NOT NULL REFERENCES map_courses (id) ON DELETE CASCADE,
    mode_id SMALLINT NOT NULL REFERENCES movement_modes (id) ON DELETE CASCADE,
    tier SMALLINT NOT NULL CHECK (tier BETWEEN 1 AND 8),
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    first_completion_record_id BIGINT REFERENCES records (id) ON DELETE SET NULL,
    first_completion_account_id BIGINT REFERENCES accounts (id) ON DELETE SET NULL,
    first_completion_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (map_id, course_id, mode_id),
    CONSTRAINT fk_course_mode_tiers_course_map FOREIGN KEY (course_id, map_id)
        REFERENCES map_courses (id, map_id) ON DELETE CASCADE,
    CONSTRAINT ck_first_completion_consistency CHECK (
        (completed = FALSE AND first_completion_record_id IS NULL AND first_completion_at IS NULL)
        OR (completed = TRUE AND first_completion_record_id IS NOT NULL AND first_completion_at IS NOT NULL)
    )
);

CREATE INDEX idx_course_mode_tiers_mode_tier ON course_mode_tiers (mode_id, tier);
CREATE INDEX idx_course_mode_tiers_course ON course_mode_tiers (course_id, mode_id);

CREATE TRIGGER trg_course_mode_tiers_updated_at
    BEFORE UPDATE ON course_mode_tiers
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

COMMENT ON TABLE course_mode_tiers IS 'Authoritative tier[map][mode][course]; T9 is never stored here';

-- Seed from course_tiers (PRO tier preferred, else NUB tier) only for scoring
-- modes. Missing/unrated rows are intentionally not created. This does NOT
-- fabricate first-completion state: existing historical records are not used to
-- decide T9 history.
-- Existing completed courses keep their first valid record as the audit
-- anchor, but no historical T9 label is fabricated. A `completed=FALSE` row
-- can still trigger T9 for a future VNL first completion.
INSERT INTO course_mode_tiers
    (map_id, course_id, mode_id, tier, completed,
     first_completion_record_id, first_completion_account_id, first_completion_at)
SELECT c.map_id, c.id, sm.mode_id,
       COALESCE(ct.pro_tier, ct.nub_tier) AS tier,
       first.id IS NOT NULL AS completed,
       first.id, first.account_id, first.created_at
FROM map_courses c
JOIN scoring_modes sm ON TRUE
JOIN course_tiers ct ON ct.course_id = c.id AND ct.mode_id = sm.mode_id
LEFT JOIN LATERAL (
    SELECT r.id, r.account_id, r.created_at
      FROM records r
     WHERE r.course_id = c.id AND r.map_id = c.map_id AND r.mode_id = sm.mode_id
       AND r.status = 'approved' AND r.deleted_at IS NULL
     ORDER BY r.created_at ASC, r.id ASC
     LIMIT 1
) first ON TRUE
WHERE COALESCE(ct.pro_tier, ct.nub_tier) BETWEEN 1 AND 8
ON CONFLICT (map_id, course_id, mode_id) DO NOTHING;

-- ---------------------------------------------------------------------------
-- Filter identity: one row per (map, course, mode, NUB/PRO).
-- ---------------------------------------------------------------------------
CREATE TABLE scoring_filters (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    map_id BIGINT NOT NULL REFERENCES maps (id) ON DELETE CASCADE,
    course_id BIGINT NOT NULL,
    mode_id SMALLINT NOT NULL REFERENCES movement_modes (id) ON DELETE CASCADE,
    record_type TEXT NOT NULL CHECK (record_type IN ('nub', 'pro')),
    tier SMALLINT NOT NULL CHECK (tier BETWEEN 1 AND 8),
    dirty BOOLEAN NOT NULL DEFAULT TRUE,
    version BIGINT NOT NULL DEFAULT 0,
    last_recalculated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_scoring_filters_course_map FOREIGN KEY (course_id, map_id)
        REFERENCES map_courses (id, map_id) ON DELETE CASCADE,
    CONSTRAINT uq_scoring_filters UNIQUE (map_id, course_id, mode_id, record_type)
);

CREATE INDEX idx_scoring_filters_dirty ON scoring_filters (dirty, id) WHERE dirty;
CREATE INDEX idx_scoring_filters_mode ON scoring_filters (mode_id, record_type, tier);

CREATE TRIGGER trg_scoring_filters_updated_at
    BEFORE UPDATE ON scoring_filters
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

COMMENT ON TABLE scoring_filters IS 'Filter = map + course + mode + NUB/PRO; each has max 1000 points per player PB';

-- Seed existing filters from approved records, using the authoritative tier.
INSERT INTO scoring_filters (map_id, course_id, mode_id, record_type, tier, dirty)
SELECT DISTINCT r.map_id, r.course_id, r.mode_id, r.time_type, cmt.tier, TRUE
FROM records r
JOIN course_mode_tiers cmt
  ON cmt.map_id = r.map_id AND cmt.course_id = r.course_id AND cmt.mode_id = r.mode_id
WHERE r.status = 'approved' AND r.deleted_at IS NULL
ON CONFLICT (map_id, course_id, mode_id, record_type) DO NOTHING;

-- ---------------------------------------------------------------------------
-- Persisted player PB points per Filter.
-- ---------------------------------------------------------------------------
CREATE TABLE player_filter_points (
    filter_id BIGINT NOT NULL REFERENCES scoring_filters (id) ON DELETE CASCADE,
    account_id BIGINT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    map_id BIGINT NOT NULL REFERENCES maps (id) ON DELETE CASCADE,
    course_id BIGINT NOT NULL,
    mode_id SMALLINT NOT NULL REFERENCES movement_modes (id) ON DELETE CASCADE,
    record_type TEXT NOT NULL CHECK (record_type IN ('nub', 'pro')),
    pb_record_id BIGINT NOT NULL REFERENCES records (id) ON DELETE CASCADE,
    time_ns BIGINT NOT NULL CHECK (time_ns > 0),
    rank INTEGER NOT NULL CHECK (rank >= 1),
    tier SMALLINT NOT NULL CHECK (tier BETWEEN 1 AND 9),
    wr_time_ns BIGINT NOT NULL CHECK (wr_time_ns > 0),
    pb_count BIGINT NOT NULL CHECK (pb_count >= 0),
    points INTEGER NOT NULL CHECK (points BETWEEN 0 AND 1000),
    is_world_record BOOLEAN NOT NULL DEFAULT FALSE,
    calculated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (filter_id, account_id),
    CONSTRAINT fk_player_filter_points_course_map FOREIGN KEY (course_id, map_id)
        REFERENCES map_courses (id, map_id) ON DELETE CASCADE,
    CONSTRAINT ck_player_filter_points_time CHECK (time_ns >= wr_time_ns)
);

CREATE INDEX idx_player_filter_points_account ON player_filter_points (account_id, mode_id, record_type);
CREATE INDEX idx_player_filter_points_filter_rank ON player_filter_points (filter_id, rank);
CREATE INDEX idx_player_filter_points_mode_points ON player_filter_points (mode_id, points DESC);

COMMENT ON TABLE player_filter_points IS 'One PB score per player per Filter; rebuilt transactionally by recalculation';

-- ---------------------------------------------------------------------------
-- Player totals per scoring mode (PRO + NUB, never cross-mode).
-- ---------------------------------------------------------------------------
CREATE TABLE player_mode_points (
    mode_id SMALLINT NOT NULL REFERENCES movement_modes (id) ON DELETE CASCADE,
    account_id BIGINT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    pro_points INTEGER NOT NULL DEFAULT 0 CHECK (pro_points >= 0),
    nub_points INTEGER NOT NULL DEFAULT 0 CHECK (nub_points >= 0),
    total_points INTEGER NOT NULL DEFAULT 0 CHECK (total_points >= 0),
    filter_count INTEGER NOT NULL DEFAULT 0 CHECK (filter_count >= 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (mode_id, account_id),
    CONSTRAINT ck_player_mode_points_total CHECK (total_points = pro_points + nub_points)
);

CREATE INDEX idx_player_mode_points_rank ON player_mode_points (mode_id, total_points DESC, account_id);

CREATE TRIGGER trg_player_mode_points_updated_at
    BEFORE UPDATE ON player_mode_points
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ---------------------------------------------------------------------------
-- Recalculation tasks with claim/heartbeat/retry and an audit log.
-- ---------------------------------------------------------------------------
CREATE TABLE filter_recalc_tasks (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    filter_id BIGINT NOT NULL REFERENCES scoring_filters (id) ON DELETE CASCADE,
    mode_id SMALLINT NOT NULL REFERENCES movement_modes (id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'failed', 'done')),
    requested_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    worker_id TEXT,
    CONSTRAINT uq_filter_recalc_pending UNIQUE (filter_id, status),
    CONSTRAINT ck_task_timestamps CHECK (
        (status = 'pending' AND started_at IS NULL AND completed_at IS NULL)
        OR (status = 'running' AND started_at IS NOT NULL AND completed_at IS NULL)
        OR (status IN ('done','failed') AND started_at IS NOT NULL)
    )
);

CREATE INDEX idx_filter_recalc_status ON filter_recalc_tasks (status, requested_at);

CREATE TABLE filter_recalc_runs (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    filter_id BIGINT NOT NULL REFERENCES scoring_filters (id) ON DELETE CASCADE,
    task_id BIGINT REFERENCES filter_recalc_tasks (id) ON DELETE SET NULL,
    mode_id SMALLINT NOT NULL REFERENCES movement_modes (id) ON DELETE CASCADE,
    status TEXT NOT NULL CHECK (status IN ('success', 'failed')),
    pb_count BIGINT NOT NULL,
    duration_ms INTEGER NOT NULL,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_filter_recalc_runs_created ON filter_recalc_runs (created_at DESC);

-- ---------------------------------------------------------------------------
-- Historical T9 label (not a permanent tier, not a points lock).
-- ---------------------------------------------------------------------------
CREATE TABLE filter_first_completions (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    map_id BIGINT NOT NULL REFERENCES maps (id) ON DELETE CASCADE,
    course_id BIGINT NOT NULL,
    mode_id SMALLINT NOT NULL REFERENCES movement_modes (id) ON DELETE CASCADE,
    record_id BIGINT NOT NULL REFERENCES records (id) ON DELETE CASCADE,
    account_id BIGINT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (map_id, course_id, mode_id),
    CONSTRAINT fk_filter_first_completions_course_map FOREIGN KEY (course_id, map_id)
        REFERENCES map_courses (id, map_id) ON DELETE CASCADE,
    CONSTRAINT uq_first_completion_record UNIQUE (record_id)
);

-- ---------------------------------------------------------------------------
-- Record changes enqueue only the affected Filter (never other modes).
-- ---------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION enqueue_filter_recalc() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    v_filter_id BIGINT;
    v_tier SMALLINT;
    v_mode_id SMALLINT;
    v_map BIGINT;
    v_course BIGINT;
    v_type TEXT;
BEGIN
    IF (TG_OP = 'DELETE') THEN
        RETURN NULL;
    END IF;

    v_map := NEW.map_id;
    v_course := NEW.course_id;
    v_mode_id := NEW.mode_id;
    v_type := NEW.time_type;

    IF NEW.status <> 'approved' OR NEW.deleted_at IS NOT NULL THEN
        v_filter_id := NULL;
    ELSE
        SELECT id INTO v_filter_id
          FROM scoring_filters
         WHERE map_id = v_map AND course_id = v_course
           AND mode_id = v_mode_id AND record_type = v_type;

        IF v_filter_id IS NULL THEN
            SELECT tier INTO v_tier
              FROM course_mode_tiers
             WHERE map_id = v_map AND course_id = v_course AND mode_id = v_mode_id;

            IF v_tier IS NULL THEN
                RETURN NULL;
            END IF;

            INSERT INTO scoring_filters (map_id, course_id, mode_id, record_type, tier, dirty)
            VALUES (v_map, v_course, v_mode_id, v_type, v_tier, TRUE)
            ON CONFLICT (map_id, course_id, mode_id, record_type) DO UPDATE SET dirty = TRUE
            RETURNING id INTO v_filter_id;
        END IF;
    END IF;

    -- Mark the Filter (or the exact Filter derived from OLD on invalidation).
    IF v_filter_id IS NULL THEN
        SELECT id INTO v_filter_id
          FROM scoring_filters
         WHERE map_id = v_map AND course_id = v_course
           AND mode_id = v_mode_id AND record_type = v_type;

        IF v_filter_id IS NULL THEN
            RETURN NULL;
        END IF;
    END IF;

    UPDATE scoring_filters SET dirty = TRUE, updated_at = now() WHERE id = v_filter_id;

    INSERT INTO filter_recalc_tasks (filter_id, mode_id, status, requested_at)
    VALUES (v_filter_id, v_mode_id, 'pending', now())
    ON CONFLICT (filter_id, status) DO UPDATE SET requested_at = LEAST(filter_recalc_tasks.requested_at, EXCLUDED.requested_at);

    -- T9: only VNL, only when the course was never completed; record the
    -- historical label once, then immediately promote the course to T8.
    IF NEW.status = 'approved' AND NEW.deleted_at IS NULL
       AND EXISTS (SELECT 1 FROM scoring_modes WHERE mode_id = v_mode_id AND code = 'VNL' AND t9_enabled)
       AND EXISTS (
           SELECT 1 FROM course_mode_tiers
            WHERE map_id = v_map AND course_id = v_course AND mode_id = v_mode_id
              AND completed = FALSE
       )
    THEN
        INSERT INTO filter_first_completions (map_id, course_id, mode_id, record_id, account_id)
        VALUES (v_map, v_course, v_mode_id, NEW.id, NEW.account_id)
        ON CONFLICT (map_id, course_id, mode_id) DO NOTHING;

        IF FOUND THEN
            UPDATE course_mode_tiers
               SET tier = 8, completed = TRUE,
                   first_completion_record_id = NEW.id,
                   first_completion_account_id = NEW.account_id,
                   first_completion_at = NEW.created_at
             WHERE map_id = v_map AND course_id = v_course AND mode_id = v_mode_id;
        END IF;
    END IF;

    RETURN NULL;
END
$$;

CREATE TRIGGER trg_records_enqueue_recalc
    AFTER INSERT OR UPDATE OF status, deleted_at, time_ns, time_type, course_id, map_id, mode_id
    ON records
    FOR EACH ROW EXECUTE FUNCTION enqueue_filter_recalc();

COMMENT ON FUNCTION enqueue_filter_recalc() IS 'Marks exactly one Filter dirty and upserts a pending recalc task';

-- Tier changes must invalidate every Filter for that (map, course, mode).
CREATE OR REPLACE FUNCTION enqueue_tier_recalc() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    task RECORD;
BEGIN
    FOR task IN
        SELECT id, mode_id FROM scoring_filters
         WHERE map_id = NEW.map_id AND course_id = NEW.course_id AND mode_id = NEW.mode_id
    LOOP
        UPDATE scoring_filters SET dirty = TRUE, tier = NEW.tier, updated_at = now() WHERE id = task.id;
        INSERT INTO filter_recalc_tasks (filter_id, mode_id, status, requested_at)
        VALUES (task.id, task.mode_id, 'pending', now())
        ON CONFLICT (filter_id, status) DO UPDATE SET requested_at = LEAST(filter_recalc_tasks.requested_at, EXCLUDED.requested_at);
    END LOOP;
    RETURN NULL;
END
$$;

CREATE TRIGGER trg_course_mode_tiers_enqueue
    AFTER INSERT OR UPDATE OF tier, completed ON course_mode_tiers
    FOR EACH ROW EXECUTE FUNCTION enqueue_tier_recalc();
