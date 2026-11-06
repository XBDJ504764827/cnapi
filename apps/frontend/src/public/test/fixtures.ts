const now = '2026-11-04T12:00:00Z';

export const modeRef = { id: 1, code: 'vanilla', display_name: 'Vanilla' };
export const styleRef = { id: 1, code: 'normal', display_name: 'Normal' };
export const pageMeta = { page: 1, page_size: 20, total: 1, total_pages: 1 };

export const modesFixture = {
  items: [
    { id: 1, code: 'vanilla', display_name: 'Vanilla', description: 'GOKZ vanilla' },
    { id: 2, code: 'simple', display_name: 'Simple KZ', description: 'GOKZ simple' },
  ],
};

export const statisticsFixture = {
  players: 3,
  maps: 2,
  courses: 3,
  records: 10,
  world_records: 6,
  servers_online: 1,
  generated_at: now,
};

export const recordFixture = {
  id: 1,
  player: { account_id: 1, steam_id64: '76561198000000001', display_name: 'Alpha' },
  map: { id: 1, name: 'kz_fixture_alpha', slug: 'kz_fixture_alpha' },
  course: { id: 1, stage: 0, name: 'Main' },
  mode: modeRef,
  style: styleRef,
  tier: 4,
  time_type: 'pro',
  teleports: 0,
  time_ns: 12500000000,
  time_seconds: '12.500000000',
  rank: 1,
  is_world_record: true,
  replay_available: true,
  points: null,
  created_at: now,
};

export const leaderboardFixture = (items: unknown[] = []) => ({
  mode: modeRef,
  time_type: 'all',
  ranking_metric: 'completed_maps',
  points_available: false,
  items,
  meta: { page: 1, page_size: 10, total: items.length, total_pages: items.length ? 1 : 0 },
});

export const playerSummaryFixture = {
  rank: 1,
  account_id: 1,
  steam_id64: '76561198000000001',
  display_name: 'Alpha',
  country: 'SE',
  completed_maps: 2,
  wr_count: 3,
  total_records: 5,
  last_activity_at: now,
  points: null,
};

export const playerOverallFixture = (scoped: boolean) => ({
  scoped,
  global_rank: 1,
  completed_maps: 2,
  total_maps: 2,
  completion_rate: 1,
  wr_count: 3,
  pro_count: 2,
  tp_count: 1,
  total_records: 5,
  points: null,
  rating: null,
});

export const playerDetailFixture = {
  profile: {
    account_id: 1,
    steam_id64: '76561198000000001',
    display_name: 'Alpha',
    country: 'SE',
    steam_profile_url: 'https://steamcommunity.com/profiles/76561198000000001',
    first_record_at: now,
    last_record_at: now,
  },
  overall: playerOverallFixture(false),
  modes: [
    {
      mode: modeRef,
      rank: 1,
      completed_maps: 2,
      total_maps: 2,
      completion_rate: 1,
      wr_count: 3,
      pro_count: 2,
      tp_count: 1,
      total_records: 5,
      points: null,
    },
  ],
  tiers: [
    {
      tier: 4,
      map_count: 1,
      completed_count: 1,
      uncompleted_count: 0,
      completion_rate: 1,
      points: null,
      average_points: null,
    },
  ],
  scope: {
    mode: modeRef,
    style: { id: 0, code: 'all', display_name: '全部 Style' },
    time_type: 'all',
    completion_definition: '主线课程（stage 0）存在已通过成绩；Bonus 成绩不计入完成数',
  },
  scoped: playerOverallFixture(true),
  ranking_metric: 'completed_maps',
  points_available: false,
};

export const playerMapsFixture = {
  account: { account_id: 1, steam_id64: '76561198000000001', display_name: 'Alpha' },
  mode: modeRef,
  style: styleRef,
  time_type: 'all',
  items: [
    {
      map_id: 1,
      name: 'kz_fixture_alpha',
      slug: 'kz_fixture_alpha',
      tier: 4,
      completed: true,
      best_time_ns: 12500000000,
      best_time_seconds: '12.500',
      best_rank: 1,
      best_time_type: 'pro',
      last_record_at: now,
      bonus_total: 1,
      bonus_completed: 1,
      points: null,
    },
  ],
  meta: pageMeta,
};

export const mapSummaryFixture = {
  id: 1,
  name: 'kz_fixture_alpha',
  slug: 'kz_fixture_alpha',
  workshop_id: 111111111,
  description: 'Alpha fixture map',
  tier: 4,
  courses: 2,
  completed_players: 3,
  records_count: 5,
  wr_time_ns: 12500000000,
  wr_time_seconds: '12.500',
  wr_player_name: 'Alpha',
  created_at: now,
};

export const mapDetailFixture = {
  map: mapSummaryFixture,
  supported_modes: [modeRef],
  courses: [{ id: 1, stage: 0, name: 'Main', nub_tier: 3, pro_tier: 4, completed_players: 3, records_count: 5 }],
  statistics: {
    total_players: 3,
    completed_players: 3,
    pro_completed_players: 2,
    nub_completed_players: 2,
    completion_rate: 1,
    record_count: 5,
    records_by_mode: [{ mode: modeRef, records: 5 }],
    records_by_time_type: [
      { time_type: 'pro', records: 3 },
      { time_type: 'nub', records: 2 },
    ],
  },
  authors: null,
};

export const compareFixture = {
  left: {
    profile: playerDetailFixture.profile,
    overall: playerOverallFixture(false),
    modes: playerDetailFixture.modes,
  },
  right: {
    profile: { ...playerDetailFixture.profile, account_id: 2, steam_id64: '76561198000000002', display_name: 'Bravo' },
    overall: playerOverallFixture(false),
    modes: playerDetailFixture.modes,
  },
  mode: modeRef,
  time_type: 'all',
  tiers: [{ tier: 4, total_maps: 1, left_completed: 1, right_completed: 1, left_points: null, right_points: null }],
  filter: 'all',
  maps: [
    {
      map_id: 1,
      name: 'kz_fixture_alpha',
      slug: 'kz_fixture_alpha',
      tier: 4,
      left: {
        completed: true,
        time_ns: 12500000000,
        time_seconds: '12.500',
        rank: 1,
        points: null,
        last_record_at: now,
      },
      right: {
        completed: true,
        time_ns: 13000000000,
        time_seconds: '13.000',
        rank: 2,
        points: null,
        last_record_at: now,
      },
      status: 'both',
      time_diff_ns: -500000000,
      left_points_diff: null,
    },
  ],
  meta: pageMeta,
  ranking_metric: 'completed_maps',
  points_available: false,
};

export const searchFixture = {
  query: 'Alpha',
  players: [{ account_id: 1, steam_id64: '76561198000000001', display_name: 'Alpha' }],
  maps: [{ id: 1, name: 'kz_fixture_alpha', slug: 'kz_fixture_alpha' }],
};

export const playerPointsFixture = {
  account_id: 1,
  steam_id64: '76561198000000001',
  modes: [
    { code: 'KZT', display_name: 'KZTimer', pro_points: 1000, nub_points: 500, total_points: 1500, filter_count: 2 },
    { code: 'SKZ', display_name: 'Simple KZ', pro_points: 800, nub_points: 0, total_points: 800, filter_count: 1 },
    { code: 'VNL', display_name: 'Vanilla', pro_points: 2000, nub_points: 1000, total_points: 3000, filter_count: 3 },
  ],
};

export const playerFilterPointsFixture = {
  account_id: 1,
  items: [
    {
      filter_id: 1,
      map_id: 1,
      map_name: 'kz_points_alpha',
      map_slug: 'kz_points_alpha',
      course_id: 1,
      course_name: 'Main',
      stage: 0,
      mode: { id: 3, code: 'KZT', display_name: 'KZTimer' },
      record_type: 'pro',
      tier: 5,
      t9_label: false,
      pb_record_id: 1,
      pb_time_seconds: '10.000000000',
      wr_time_seconds: '10.000000000',
      rank: 1,
      points: 1000,
      is_world_record: true,
      calculated_at: now,
    },
  ],
  meta: pageMeta,
};

export const pointsLeaderboardFixture = {
  mode: {
    code: 'KZT',
    display_name: 'KZTimer',
    mode: { id: 3, code: 'kztimer', display_name: 'KZTimer' },
    t9_enabled: false,
  },
  items: [
    {
      rank: 1,
      account_id: 1,
      steam_id64: '76561198000000001',
      display_name: 'Alpha',
      pro_points: 1000,
      nub_points: 500,
      total_points: 1500,
      filter_count: 2,
    },
  ],
  meta: pageMeta,
};
