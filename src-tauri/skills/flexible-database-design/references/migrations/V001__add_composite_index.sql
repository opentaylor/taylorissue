-- 添加复合索引（若从无此索引的旧版升级）
-- 执行前请备份 data/flexible.db

CREATE INDEX IF NOT EXISTS idx_dynamic_cat_field 
ON dynamic_data(record_category, field_name);
