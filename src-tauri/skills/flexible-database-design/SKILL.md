---
name: flexible-database-design
version: 1.0.0
description: |
  Guide agents and users to design and implement a "flexible database" on SQLite that can handle semi-structured, multi-source data.
  Typical scenarios: personal knowledge base, PDF/report archive, policy tracking, fragmented notes, custom form/questionnaire fields, multi-source data aggregation, event logging.
  When the user says things like "I want to build a knowledge base", "archive PDF reports", "search within report content", "collect policies or scattered information", this skill provides the end-to-end workflow.
  It includes: design principles, a three-layer model, agent workflows, schema templates, and reference Python scripts.
metadata:
  openclaw:
    emoji: 🧱
    requires:
      bins: ["python3"]
    install:
      - id: python
        kind: system
        bins: ["python3"]
---

# Flexible Database Design — 通用 Skill

一套可复用的「软 Schema」设计方法：主干硬、尾巴软，三层演进。用户安装后，Agent 可据此指导其**真正构建**出灵活数据库。

---

## 一、核心心法（3 条）

1. **主干硬、尾巴软** — 固定列只放：谁、何时、从哪来、类型；其余进 JSON/键值对。
2. **先全量保留，再按需提键** — 原始数据完整落库；需要查/统计时再写入键值对或业务表。
3. **分层演进** — 原始层 → 软字段层（JSON/键值对）→ 业务视图层；缺什么再补。

---

## 二、三层模型

| 层级 | 作用 | 典型做法 |
|------|------|----------|
| **原始层** | 不丢信息、可追溯 | 整条记录原样存，加哈希去重、来源、版本号 |
| **软字段层** | 灵活查询 | JSON 存结构化结果；键值对表按 key 查、聚合 |
| **业务视图层** | 高频查询、报表 | 物化表/视图，按需建索引 |

---

## 三、Agent 工作流（用户说「想做 XX」时执行）

### Step 1：Discovery（必做）

向用户确认以下信息，并记录到对话中：

| 问题 | 用途 |
|------|------|
| 你的数据主要来自哪里？（微信/网页/API/手动输入/多种） | 定 `source_type` 枚举 |
| 每条记录大概有哪些「永远会有」的信息？（如：时间、来源、类型） | 定主干字段 |
| 有哪些「可能经常变、不同来源不一样」的信息？ | 确认用 JSON/键值对 |
| 是否有按编号/文号等唯一标识查询的需求？ | 决定是否在视图中加入对应字段，并考虑快捷查询 |
| 需要全文搜索吗？ | 决定是否建 FTS |
| 内容语言？（中文为主 / 英文为主 / 混合） | 决定是否采用中文分词、FTS 策略 |
| 内容形态？（纯文本 / PDF / Excel / 网页 / 混合） | 决定归档前是否需要提取（如 pypdf） |
| 预期数据量？（百 / 千 / 万 / 十万级） | 决定 LIKE 回退是否可用、是否考虑外部搜索引擎 |

### Step 2：选择场景模板

根据用户描述，选择最接近的场景并适配：

| 场景 | 主干字段建议 | 软字段典型 key |
|------|--------------|----------------|
| **个人知识库 / 碎片收集** | id, created_at, source, content_type, raw_content | title, tags, url, project, deadline |
| **政策信息收集** | id, created_at, source, source_type | title, release_date, issuing_org, policy_type, url, policy_no, industry, subsidy_amount |
| **财务报表收集** | id, created_at, source, source_type | company, report_type, report_date, revenue, net_income, total_assets, roa, roe |
| **表单/问卷** | id, created_at, form_id, respondent_id, raw_response | 各题目 id 或题目名 |
| **PDF/报告知识库** | id, created_at, source, content_type, raw_content | report_title, report_type, period_start, period_end, file_path |
| **多源异构（如群消息聚合）** | id, created_at, source, sender, raw_content | data_type, items[], trend, 各业务字段 |

### Step 3：生成并落地 Schema

1. 复制 [references/schema_template.sql](references/schema_template.sql) 到用户项目。
2. 按 Discovery 结果做**最小修改**：
   - 调整 `source_type` 的 CHECK 枚举；
   - 如需 FTS，取消 `messages_fts` 相关注释；
   - 业务视图层参考 [references/view_examples.sql](references/view_examples.sql) 按需新增视图。**业务视图应覆盖高频查询字段**：政策类含 title、release_date、issuing_org、policy_type、url、policy_no；知识库类含 title、tags、url、project；财报类含 company、report_type、report_date、revenue、net_income；PDF/报告类含 report_title、report_type、period_start、period_end。
4. 在用户项目中创建 `data/` 目录，指定 db 路径（如 `data/flexible.db`）。

### 全文检索策略（当 Discovery 勾选「需要全文搜索」时）

| 内容语言 | 推荐方案 | 说明 |
|----------|----------|------|
| 英文为主 | FTS5 (unicode61) | 默认即可 |
| 中文为主 | FTS + LIKE 回退 | 长短语易漏检，需实现 `recall()` |
| 中文为主（数据量 &lt; 5000） | 同上 + 短词拆分 | 如「煤炭期货价格」→ 拆为「煤炭」「期货」「价格」分别查，取并集 |
| 中文为主（数据量 &gt; 5000） | 考虑 Meilisearch / jieba+FTS | SQLite FTS 中文能力有限 |

**实现要点**：查询层实现 `recall(keyword)`：先 FTS，无结果则 LIKE；中文可加短词拆分。`LIKE '%x%'` 无法用索引，数据量大时需评估性能。扫描件 PDF 无法提取正文，归档时需跳过或标记。

### Step 4：生成并适配脚本

1. 将 [scripts/](scripts/) 下脚本复制到用户项目 `scripts/`：**核心** flexible_db.py、archive_item.py、query_items.py；**可选** manage_item.py、import_batch.py、quick_validate.py、extractors/。
2. 修改 `flexible_db.py` 中的 `db_path` 指向用户的 db 路径。
3. 若用户有特殊「提取逻辑」（如用 LLM 从原文抽结构化数据），使用 `archive_item.py --llm-extract` 或配置 `FLEXIBLE_EXTRACTOR=extractors.dummy:extract`；可自定义 `extractors/` 下的实现。**抽取器字段应与场景匹配**：政策类建议 title、release_date、issuing_org、policy_type、url、policy_no；知识库类建议 title、tags、url、project；财报类建议 company、report_type、report_date、revenue、net_income（金额统一存「元」）；PDF/报告类建议 report_title、report_type、period_start、period_end。
4. **若内容为 PDF/文档**：归档前需提取正文（推荐 pypdf：`pip install pypdf`），扫描件无法提取时跳过。可选：将 PDF 复制到项目 `data/reports/` 统一管理，`file_path` 存相对路径；抽取器可从文件名解析 period、source、report_type。
5. 运行验证（`python` 或 `python3` 按环境选择）：

```bash
# 建表（首次运行脚本时会自动执行 schema）
python3 scripts/archive_item.py -c "测试第一条" -s "manual"

# 查询
python3 scripts/query_items.py --list
python3 scripts/query_items.py --field "tags" --value "工作"
python3 scripts/query_items.py --stats
```

### Step 5：验证清单

- [ ] 建表成功，无报错
- [ ] 能归档一条测试数据
- [ ] 能按分类/字段查询
- [ ] 全文搜索（若启用）可用；中文检索建议实现 FTS + LIKE 回退（如 `recall()`）
- [ ] **中文检索**：用 3–5 个中文短语（含 2–4 字短语）实测，确认能召回预期结果
- [ ] **数据量评估**：若预期 &gt; 5000 条，已评估 LIKE 性能或选用替代方案

---

## 四、场景速查（用户说想做 XX 时）

| 用户意图 | 推荐主干 | 推荐软字段 | 备注 |
|----------|----------|------------|------|
| 个人知识库 | id, created_at, source, content_type | title, tags, url, project | 碎片收集同此 |
| PDF/报告知识库 | id, created_at, source, content_type, file_path | report_title, report_type, period_start, period_end | 需提取正文；中文检索见「全文检索策略」 |
| 政策信息收集 | id, created_at, source, source_type | title, release_date, issuing_org, policy_type, url, policy_no, industry, subsidy_amount | 政府网站/新闻来源，文号常作查询键 |
| 财务报表收集 | id, created_at, source, source_type | company, report_type, report_date, revenue, net_income, total_assets, roa, roe | 金额统一存「元」；report_type→category |
| 表单/问卷 | id, created_at, form_id, respondent_id | 题目 id → 答案 | 可加 form_version |
| 多源消息聚合 | id, created_at, source, sender | data_type, items[], trend | 参考 agri-market-info |
| 埋点/事件 | id, created_at, event_name, user_id | properties JSON | 可加 event_version |

---

## 五、反模式

- 一上来穷举所有可能字段 → 改表成本高
- 软字段 key 随心所欲 → 约定 snake_case、命名空间
- 所有查询都扫 JSON → 高频条件应提键值对并建索引
- 没有原始层 → 无法回溯、补字段
- 中文全文检索只依赖 FTS5 → unicode61 对中文不友好，需 FTS + LIKE 回退或中文分词
- `LIKE '%x%'` 用于万级数据 → 全表扫描，应评估或改用外部搜索引擎
- PDF 只存路径不提取正文 → 无法全文检索，需在归档时提取并写入 raw_content

---

## 六、文件结构（本 Skill 包）

```
flexible-database-design/
├── SKILL.md                 # 本文件
├── README.md
├── references/
│   ├── schema_template.sql  # 通用建表模板
│   ├── view_examples.sql    # 业务视图示例
│   └── fulltext_chinese.md  # 可选：中文检索实现示例（短词拆分、recall 逻辑）
└── scripts/
    ├── flexible_db.py       # 数据库核心逻辑
    ├── archive_item.py     # 归档 CLI（支持 --llm-extract、--backup）
    ├── query_items.py      # 查询 CLI（支持 --export）
    ├── manage_item.py      # 管理 CLI（软删除、恢复、更新）
    ├── import_batch.py     # 批量导入
    ├── quick_validate.py   # 快速验证
    └── extractors/         # 抽取器（可替换为 LLM 实现）
```



---

## 工具声明（OpenClaw / ClawHub）

allowed-tools:
  - Bash
  - FileRead
  - FileWrite
  - Shell

---

**作者** | Mars Yang
**日期** | 2025-03-09
