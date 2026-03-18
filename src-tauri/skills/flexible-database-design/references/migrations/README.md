# 迁移指南

Schema 变更时，将增量 SQL 放入本目录，按版本号命名。

---

## 命名规范

`V{版本号}__{描述}.sql`

示例：`V001__add_composite_index.sql`

---

## 执行顺序

1. 备份 `data/flexible.db`
2. 按版本号顺序执行迁移 SQL
3. 可选：在 `records` 表或独立 `schema_version` 表中记录当前版本

---

## 示例：添加复合索引（已包含在 schema_template）

若从旧版升级，可单独执行：

```sql
-- V001__add_composite_index.sql
CREATE INDEX IF NOT EXISTS idx_dynamic_cat_field 
ON dynamic_data(record_category, field_name);
```

---

## 示例：启用 FTS

若已有数据，需先创建 FTS、触发器，再回填：

```sql
-- V002__enable_fts.sql（参考 flexible_db._init_fts）
CREATE VIRTUAL TABLE IF NOT EXISTS records_fts USING fts5(...);
-- 创建触发器后，回填：INSERT INTO records_fts(rowid, raw_content) SELECT id, raw_content FROM records;
```

---

## 回滚

迁移前务必备份。回滚需根据具体变更编写反向 SQL，或从备份恢复。
