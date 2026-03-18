-- 业务视图层示例
-- 按需复制到项目并执行，或作为参考自行扩展
-- 依赖 schema_template.sql 中的 records、dynamic_data 表

-- ============================================
-- 示例 1：按 record_category 聚合统计
-- ============================================
CREATE VIEW IF NOT EXISTS v_category_summary AS
SELECT
    record_category AS category,
    COUNT(DISTINCT record_id) AS record_count,
    MIN(data_date) AS min_date,
    MAX(data_date) AS max_date
FROM dynamic_data
WHERE record_category IS NOT NULL
GROUP BY record_category;

-- ============================================
-- 示例 2：按 data_date 汇总（含 extracted 关键字段）
-- ============================================
CREATE VIEW IF NOT EXISTS v_daily_records AS
SELECT
    r.record_id,
    r.source,
    r.created_at,
    r.raw_content,
    r.extracted,
    d.data_date
FROM records r
LEFT JOIN (
    SELECT record_id, MIN(data_date) AS data_date
    FROM dynamic_data
    WHERE data_date IS NOT NULL
    GROUP BY record_id
) d ON r.record_id = d.record_id
WHERE r.is_deleted = 0;

-- ============================================
-- 示例 3：知识库风格 - title/tags 扁平视图（需有 title 字段）
-- 注意：tags 为 JSON 数组时 field_value 存为 ["a","b"] 字符串，应用层需 json.loads(tags) 解析
-- ============================================
CREATE VIEW IF NOT EXISTS v_knowledge_flat AS
SELECT
    r.record_id,
    r.source,
    r.created_at,
    r.raw_content,
    MAX(CASE WHEN d.field_name = 'title' THEN d.field_value END) AS title,
    MAX(CASE WHEN d.field_name = 'project' THEN d.field_value END) AS project,
    GROUP_CONCAT(DISTINCT CASE WHEN d.field_name = 'tags' THEN d.field_value END) AS tags
FROM records r
LEFT JOIN dynamic_data d ON r.record_id = d.record_id
WHERE r.is_deleted = 0
GROUP BY r.record_id, r.source, r.created_at, r.raw_content;
