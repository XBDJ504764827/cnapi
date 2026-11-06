-- 0002_kz_reference_data
--
-- Reference ("enum") tables for the KZ domain plus the shared updated_at
-- trigger. Values are taken from the upstream sources studied for phase 2:
--
--   * GOKZ 3.8.0 (KZGlobalTeam/gokz, commit 3a601e1)
--       - sourcemod/scripting/include/gokz/core.inc      (Mode/Style/TimeType)
--       - sourcemod/scripting/include/gokz/jumpstats.inc (JumpType)
--       - sourcemod/scripting/include/gokz/global.inc    (GlobalAPI mode names)
--   * cs2kz-api (KZGlobalTeam/cs2kz-api, commit c938925)
--       - crates/cs2kz/src/mode.rs, styles.rs, jumpstats/jump_type.rs
--   * csgo-legacy-globalapi-plugin (commit bec212c)
--       - GlobalAPI-Core.sp (global mode strings), include/GlobalAPI-Core.inc
--
-- Codes are stable identifiers used by the API and by import adapters. Numeric
-- ids are internal only and must not leak into the public API.

CREATE OR REPLACE FUNCTION set_updated_at() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END
$$;

COMMENT ON FUNCTION set_updated_at() IS 'Trigger function that maintains updated_at columns in UTC';

-- ---------------------------------------------------------------------------
-- movement_modes
-- ---------------------------------------------------------------------------
CREATE TABLE movement_modes (
    id SMALLINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ck_movement_modes_code CHECK (code ~ '^[a-z][a-z0-9_]*$')
);

COMMENT ON TABLE movement_modes IS 'KZ movement modes / rulesets (GOKZ Mode, GlobalAPI mode strings, cs2kz Mode)';
COMMENT ON COLUMN movement_modes.code IS 'Stable identifier, e.g. vanilla, simple, kztimer, classic';

INSERT INTO movement_modes (code, display_name, description) VALUES
    ('vanilla', 'Vanilla', 'GOKZ Mode_Vanilla (0); GlobalAPI mode kz_vanilla; cs2kz Mode::Vanilla (1).'),
    ('simple',  'Simple KZ', 'GOKZ Mode_SimpleKZ (1); GlobalAPI mode kz_simple. No cs2kz equivalent.'),
    ('kztimer', 'KZTimer', 'GOKZ Mode_KZTimer (2); GlobalAPI mode kz_timer. No cs2kz equivalent.'),
    ('classic', 'Classic', 'cs2kz Mode::Classic (2). Not present in GOKZ (CS:GO) or the legacy GlobalAPI mode list.');

-- ---------------------------------------------------------------------------
-- movement_styles
-- ---------------------------------------------------------------------------
CREATE TABLE movement_styles (
    id SMALLINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ck_movement_styles_code CHECK (code ~ '^[a-z][a-z0-9_-]*$')
);

COMMENT ON TABLE movement_styles IS 'Movement style dimension of a run/jumpstat';
COMMENT ON COLUMN movement_styles.code IS 'Stable identifier; only `normal` is confirmed by the studied sources';

-- GOKZ 3.8.0 defines exactly one style (`Style_Normal = 0`, STYLE_COUNT = 1).
-- Sideways / half-sideways / space styles appear in KZTimer ecosystems but were
-- NOT found in the studied sources, therefore they are intentionally not seeded.
-- cs2kz `Styles` is a bitmask of server settings (auto-bhop, legacy-jump,
-- auto-unduck), not a movement style; see docs/globalapi-compatibility.md.
INSERT INTO movement_styles (code, display_name, description) VALUES
    ('normal', 'Normal', 'GOKZ Style_Normal (0). The only movement style confirmed by the studied sources.');

-- ---------------------------------------------------------------------------
-- jump_types
-- ---------------------------------------------------------------------------
CREATE TABLE jump_types (
    id SMALLINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    gokz_code SMALLINT UNIQUE,
    cs2kz_code SMALLINT UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ck_jump_types_code CHECK (code ~ '^[a-z][a-z0-9-]*$')
);

COMMENT ON TABLE jump_types IS 'Jumpstat jump types';
COMMENT ON COLUMN jump_types.gokz_code IS 'Value of GOKZ JumpType_* enum (gokz/jumpstats.inc)';
COMMENT ON COLUMN jump_types.cs2kz_code IS 'Value of cs2kz JumpType enum (crates/cs2kz/src/jumpstats/jump_type.rs)';

-- GOKZ codes: LongJump=0 .. Invalid=11 (JUMPTYPE_COUNT = 12).
-- cs2kz codes: LongJump=0 .. Fall=7. LowpreBhop, LowpreWeirdJump, Other and
-- Invalid have no cs2kz equivalent.
INSERT INTO jump_types (code, display_name, gokz_code, cs2kz_code, description) VALUES
    ('long-jump',        'Long Jump',          0, 0, 'GOKZ JumpType_LongJump / cs2kz JumpType::LongJump'),
    ('bhop',             'Bunnyhop',           1, 1, 'GOKZ JumpType_Bhop / cs2kz JumpType::Bhop'),
    ('multi-bhop',       'Multi Bunnyhop',     2, 2, 'GOKZ JumpType_MultiBhop / cs2kz JumpType::MultiBhop'),
    ('weird-jump',       'Weird Jump',         3, 3, 'GOKZ JumpType_WeirdJump / cs2kz JumpType::WeirdJump'),
    ('ladder-jump',      'Ladder Jump',        4, 4, 'GOKZ JumpType_LadderJump / cs2kz JumpType::LadderJump'),
    ('ladderhop',        'Ladderhop',          5, 5, 'GOKZ JumpType_Ladderhop / cs2kz JumpType::Ladderhop'),
    ('jumpbug',          'Jumpbug',            6, 6, 'GOKZ JumpType_Jumpbug / cs2kz JumpType::Jumpbug'),
    ('lowpre-bhop',      'Lowpre Bunnyhop',    7, NULL, 'GOKZ only (JumpType_LowpreBhop)'),
    ('lowpre-weird-jump','Lowpre Weird Jump',  8, NULL, 'GOKZ only (JumpType_LowpreWeirdJump)'),
    ('fall',             'Fall',               9, 7, 'GOKZ JumpType_Fall / cs2kz JumpType::Fall'),
    ('other',            'Other',             10, NULL, 'GOKZ JumpType_Other'),
    ('invalid',          'Invalid',           11, NULL, 'GOKZ JumpType_Invalid (stored for completeness, not ranking)');
