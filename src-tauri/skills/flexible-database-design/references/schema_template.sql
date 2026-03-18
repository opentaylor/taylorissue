-- Flexible Database Schema Template
-- 通用灵活数据库建表模板，适用于：知识库、碎片收集、表单、多源聚合等
-- 使用前请根据 SKILL.md 中的场景速查调整 source_type、表名等

PRAGMA foreign_keys = ON;
PRAGMA encoding = 'UTF-8';

-- ============================================
-- 1. 原始层：records 主表
-- ============================================
CREATE TABLE IF NOT EXISTS records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id TEXT UNIQUE NOT NULL,       -- 唯一ID（MD5生成，用于去重）
    
    -- 主干字段（按场景调整）
    source TEXT,                         -- 来源标识（如：微信群名、表单ID、应用名）
    source_type TEXT DEFAULT 'manual',   -- 来源类型，按需扩展 CHECK
    -- 示例: CHECK(source_type IN ('manual', 'wechat', 'api', 'form', 'import'))
    
    content_type TEXT DEFAULT 'text',   -- 内容类型：text, link, image, file, mixed
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    -- 原始数据 100% 保留
    raw_content TEXT NOT NULL,
    raw_content_hash TEXT,              -- MD5 去重
    
    -- 软字段层：动态结构化数据（JSON）
    extracted JSON,
    /*
    extracted 示例（知识库/碎片）：
    {"title": "某想法", "tags": ["工作","灵感"], "url": "https://...", "project": "项目A"}
    
    extracted 示例（多源聚合）：
    {"data_type": "价格", "items": [{"name": "A", "value": 100, "unit": "元"}], "trend": "涨"}
    */
    
    -- 三件套
    confidence_score REAL DEFAULT 1.0,  -- 置信度 0-1
    is_deleted INTEGER DEFAULT 0
);

-- ============================================
-- 2. 软字段层：键值对表，支持按任意字段查询
-- ============================================
CREATE TABLE IF NOT EXISTS dynamic_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id TEXT NOT NULL,
    
    record_category TEXT,               -- 数据分类（如：知识、价格、表单答案）
    field_name TEXT NOT NULL,
    field_value TEXT,
    field_type TEXT DEFAULT 'text',     -- text, number, date, boolean, json
    
    unit TEXT,
    numeric_value REAL,                 -- 数值型时便于排序/聚合
    
    data_date TEXT,                     -- 若有业务日期
    
    FOREIGN KEY (record_id) REFERENCES records(record_id) ON DELETE CASCADE
);

-- ============================================
-- 3. 全文检索（可选）
-- 由 FLEXIBLE_DB_FTS=1 启用时 flexible_db 自动创建；或取消下方注释后直接建表
-- ============================================
-- CREATE VIRTUAL TABLE IF NOT EXISTS records_fts USING fts5(
--     content = 'records',
--     content_rowid = 'id',
--     raw_content
-- );
-- CREATE TRIGGER IF NOT EXISTS records_fts_insert AFTER INSERT ON records BEGIN
--   INSERT INTO records_fts(rowid, raw_content) VALUES (new.id, new.raw_content);
-- END;
-- CREATE TRIGGER IF NOT EXISTS records_fts_update AFTER UPDATE ON records BEGIN
--   INSERT INTO records_fts(records_fts, rowid, raw_content) VALUES('delete', old.id, old.raw_content);
--   INSERT INTO records_fts(rowid, raw_content) VALUES (new.id, new.raw_content);
-- END;
-- CREATE TRIGGER IF NOT EXISTS records_fts_delete AFTER DELETE ON records BEGIN
--   INSERT INTO records_fts(records_fts, rowid, raw_content) VALUES('delete', old.id, old.raw_content);
-- END;

-- ============================================
-- 4. 索引
-- ============================================
CREATE INDEX IF NOT EXISTS idx_records_created ON records(created_at);
CREATE INDEX IF NOT EXISTS idx_records_source ON records(source);
CREATE INDEX IF NOT EXISTS idx_records_type ON records(content_type);
CREATE INDEX IF NOT EXISTS idx_records_hash ON records(raw_content_hash);

CREATE INDEX IF NOT EXISTS idx_dynamic_record ON dynamic_data(record_id);
CREATE INDEX IF NOT EXISTS idx_dynamic_category ON dynamic_data(record_category);
CREATE INDEX IF NOT EXISTS idx_dynamic_field ON dynamic_data(field_name);
CREATE INDEX IF NOT EXISTS idx_dynamic_date ON dynamic_data(data_date);
CREATE INDEX IF NOT EXISTS idx_dynamic_cat_field ON dynamic_data(record_category, field_name);
