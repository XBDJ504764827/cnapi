-- 20261106000005_map_preview_image
--
-- 地图封面图。来源：gokz.top 的 workshop 预览图接口（307 到 Steam CDN），
-- 由一次性脚本回填，浏览器直连 images.steamusercontent.com 加载。
ALTER TABLE maps ADD COLUMN preview_image_url TEXT;

ALTER TABLE maps ADD CONSTRAINT ck_maps_preview_image_url
    CHECK (preview_image_url IS NULL OR char_length(preview_image_url) BETWEEN 8 AND 512);

COMMENT ON COLUMN maps.preview_image_url IS 'Workshop preview image URL (Steam CDN), filled by scripts/import-map-previews';
