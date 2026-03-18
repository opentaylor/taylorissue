# Flexible Database Design — 通用 Skill

<p align="center">
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-blue.svg" alt="Platform">
  <img src="https://img.shields.io/badge/python-3.6+-green.svg" alt="Python">
  <img src="https://img.shields.io/badge/license-MIT-yellow.svg" alt="License">
</p>

Reusable "soft schema" design methodology + ready-to-use implementation. Lets AI agents guide you to build flexible databases for personal knowledge bases, fragment collection, forms, and multi-source aggregation. Compatible with Claude Skills, Cursor, OpenClaw, Cherry Studio, and other AI workbenches.
可复用的「柔性 Schema」设计心法 + 可落地的实现包。用户安装后，Agent 可据此指导其**真正构建**出灵活数据库（个人知识库、碎片收集、表单、多源聚合等）。

## 内容

| 文件 | 说明 |
|------|------|
| **SKILL.md** | 心法 + Agent 工作流 + 场景速查，兼容 Claude Skills 规范 |
| **references/schema_template.sql** | 通用建表模板，按场景微调 |
| **scripts/flexible_db.py** | 数据库核心逻辑 |
| **scripts/archive_item.py** | 归档 CLI（支持 `--backup`、`-F` 从文件读取） |
| **scripts/query_items.py** | 查询 CLI（支持分页、全文检索） |
| **scripts/manage_item.py** | 管理 CLI（软删除、恢复、更新） |
| **scripts/import_batch.py** | 批量导入（JSON/CSV） |
| **scripts/quick_validate.py** | 快速验证脚本 |
| **scripts/extractors/** | 抽取器（默认 dummy，可替换为 LLM） |
| **references/view_examples.sql** | 业务视图示例 |

## 安装与使用

### 方式一：作为 Skill 使用（多平台）

本 Skill 遵循 **Claude Skills 规范**（SKILL.md + YAML frontmatter），兼容多种 AI IDE 与智能体平台。将本目录复制到对应路径即可：

| 平台 | 全局路径（所有项目可用） | 项目路径（仅当前项目） | 备注 |
|------|--------------------------|------------------------|------|
| **Claude Code** | `~/.claude/skills/flexible-database-design/` | `.claude/skills/flexible-database-design/` | Anthropic 官方 IDE |
| **Cursor** | `~/.cursor/skills/flexible-database-design/` | `.cursor/skills/flexible-database-design/` | 需在 AGENTS.md 注册 |
| **Windsurf** | `~/.claude/skills/flexible-database-design/` | `.claude/skills/flexible-database-design/` | 兼容 Claude Projects |
| **Continue** | `~/.claude/skills/flexible-database-design/` | `.claude/skills/flexible-database-design/` | 兼容 Claude Skills |
| **Cherry Studio** | — | `.claude/skills/flexible-database-design/` | 需在 Skill 管理器中注册 |
| **OpenClaw** | 配置 `load.extraDirs` 指向本目录 | — | 见 SKILL.md 场景速查 |

安装后，在对话中说「我想做个个人知识库」「收集政策/碎片信息」等，Agent 会按 SKILL.md 中的工作流引导你。

> **说明**：本 Skill 需 Agent 具备执行 Python 脚本的能力（终端权限），脚本运行需 Python 3 环境。

### 方式二：直接复制到项目落地

1. 将 `references/` 和 `scripts/` 复制到你的项目根目录，保持目录结构
2. 运行（`python` 或 `python3` 按环境选择）：

```bash
# 首次运行会自动建表
python3 scripts/archive_item.py -c "测试第一条" -s "manual"

# 查询
python3 scripts/query_items.py --list
python3 scripts/query_items.py --schema
python3 scripts/query_items.py --stats
```

3. 通过环境变量 `FLEXIBLE_DB_PATH` 指定 db 路径，或保持默认 `data/flexible.db`

### 环境变量

| 变量 | 说明 |
|------|------|
| `FLEXIBLE_DB_PATH` | 数据库文件路径 |
| `FLEXIBLE_DB_SCHEMA` | Schema 文件路径（可选） |
| `FLEXIBLE_DB_FTS` | 设为 `1` 启用全文检索（需 SQLite 3.9+） |

## 快速验证

```bash
# 一键验证核心流程
python3 scripts/quick_validate.py

# 单元测试
python3 -m unittest discover -s tests -v
```

## 使用示例

```bash
# 归档（可选 --backup 归档前备份；-F 从文件读取长文本）
python3 scripts/archive_item.py -c "软 Schema 设计心法" -s "笔记" \
  -e '{"title":"心法","tags":["数据库","设计"],"project":"学习"}'
python3 scripts/archive_item.py -F report.txt -s "file" -e '{"title":"年报摘要"}'

# 查询与分页
python3 scripts/query_items.py --list
python3 scripts/query_items.py --field "tags" --value "数据库" --offset 0
python3 scripts/query_items.py --stats   # 总记录数、按分类统计
# 精确匹配（含 %/_ 时避免通配符）
python3 scripts/query_items.py --field "code" --value "100%" --exact

# 全文检索（需 FLEXIBLE_DB_FTS=1）
FLEXIBLE_DB_FTS=1 python3 scripts/query_items.py --search "心法"

# 管理：软删除、恢复、更新
python3 scripts/manage_item.py --delete <record_id>
python3 scripts/manage_item.py --restore <record_id>
python3 scripts/manage_item.py --update <record_id> -e '{"title":"新标题"}'

# 批量导入与导出
python3 scripts/import_batch.py data.json
python3 scripts/query_items.py --export json --output export.json
python3 scripts/query_items.py --export csv --output export.csv
```

## 常见问题（FAQ）

**Q: 如何更换数据库路径？**  
A: 设置环境变量 `FLEXIBLE_DB_PATH`，或在代码中传入 `FlexibleDatabase(db_path="...")`。

**Q: 全文检索不生效？**  
A: 需设置 `FLEXIBLE_DB_FTS=1`，且 SQLite 版本 ≥ 3.9。新建库时设置；已有库需执行迁移回填 FTS。

**Q: 字段值含 `%` 或 `_` 时查询异常？**  
A: 使用 `--exact` 或 `query_dynamic(exact_match=True)` 进行精确匹配；模糊查询已自动转义。

**Q: 如何接入自己的 LLM 抽取？**  
A: 实现 `(content: str) -> dict` 函数，配置 `FLEXIBLE_EXTRACTOR=your_module:your_func`，或使用 `--extractor` 参数。

---

## 应用案例

| 案例 | 说明 |
|------|------|
| [财务报表场景说明](docs/examples/财务报表场景说明.md) | 财报收集、派生指标视图、PDF 归档、单文件多记录策略 |
| [外汇数据库场景适配](docs/examples/外汇数据库场景适配.md) | 结构化时间序列场景：单表 + 元数据表，按场景裁剪而非照搬三层模型 |
| [问卷收集场景适配](docs/examples/问卷收集场景适配.md) | 表单/问卷场景：答案展平、form_version、按题目统计 |
| [多源消息聚合场景适配](docs/examples/多源消息聚合场景适配.md) | 多源金融时序：国债+外汇、联合视图、按日期对齐 |

本 Skill 提供通用框架，各应用在此基础上扩展业务逻辑。

---

## 性能建议

| 场景 | 建议 |
|------|------|
| 批量导入 | 使用 `import_batch()`，单事务、避免逐条 commit |
| 高频按 category+field 查询 | 已建 `idx_dynamic_cat_field` 复合索引 |
| 全文检索 | 启用 FTS5，仅对新插入数据自动索引；已有数据需迁移回填 |
| 大数据量分页 | 使用 `offset` 或游标；万级以上可考虑 `created_at` 范围分页 |

---

## 文档索引

| 文档 | 说明 |
|------|------|
| [SKILL.md](SKILL.md) | 心法 + Agent 工作流 + 场景速查 |
| [docs/API.md](docs/API.md) | FlexibleDatabase 方法说明 |
| [docs/examples/财务报表场景说明.md](docs/examples/财务报表场景说明.md) | 财报收集、货币约定、派生指标、PDF 归档、单文件多记录 |
| [docs/examples/外汇数据库场景适配.md](docs/examples/外汇数据库场景适配.md) | 结构化时间序列：单表 + 元数据表，场景适配案例 |
| [docs/examples/问卷收集场景适配.md](docs/examples/问卷收集场景适配.md) | 表单/问卷：答案展平、按题目统计 |
| [docs/examples/多源消息聚合场景适配.md](docs/examples/多源消息聚合场景适配.md) | 多源金融时序：国债+外汇、联合视图 |
| [references/fulltext_chinese.md](references/fulltext_chinese.md) | 中文全文检索实现示例（短词拆分、recall 逻辑） |
| [references/migrations/](references/migrations/) | Schema 迁移示例 |

## License

MIT
