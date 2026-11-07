-- 20261106000003_records_source_gokz_top
--
-- 允许一次性导入 gokz.top 的历史成绩。
--
-- 背景：生产库里的 4884 条记录全部来自 GlobalAPI 的 WR 订阅，没有任何普通完成
-- 记录，导致「完成地图数 / 成绩历史」严重偏低。2026-11-07 用
-- `bin/import_gokz_top` 从 gokz.top 全量补了一次，导入来源需要与 kzgo/全局 API
-- 区分开，便于日后审计或按来源清理。

ALTER TABLE records DROP CONSTRAINT ck_records_source;

ALTER TABLE records ADD CONSTRAINT ck_records_source CHECK (
    source IN (
        'plugin',
        'admin',
        'globalapi_import',
        'cs2kz_import',
        'legacy_import',
        'gokz_top_import'
    )
);
