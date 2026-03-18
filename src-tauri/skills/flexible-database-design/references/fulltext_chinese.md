# 中文全文检索实现示例

> 可选参考：当内容以中文为主、数据量 &lt; 5000 时，可采用 FTS + LIKE 回退 + 短词拆分。

---

## 一、recall 逻辑

```python
def recall(db, keyword: str, limit: int = 10) -> list:
    """先 FTS，无结果则 LIKE；中文可加短词拆分"""
    rows = db.search_fulltext(keyword, limit)
    if rows:
        return rows
    # 回退：LIKE 模糊匹配
    rows = db.query_by_like(keyword, limit)
    if rows:
        return rows
    # 可选：短词拆分（如「煤炭期货价格」→「煤炭」「期货」「价格」）
    if len(keyword) >= 4:
        terms = _split_chinese(keyword)  # 简单按 2 字切分或 jieba 分词
        seen = set()
        for term in terms:
            for r in db.query_by_like(term, limit):
                if r["record_id"] not in seen:
                    seen.add(r["record_id"])
                    rows.append(r)
    return rows[:limit]
```

---

## 二、短词拆分示例

| 策略 | 说明 | 适用 |
|------|------|------|
| **固定 2 字** | 「煤炭期货」→「煤炭」「炭期」「期货」 | 简单，易漏 |
| **jieba 分词** | `jieba.lcut("煤炭期货价格")` → `["煤炭","期货","价格"]` | 需 `pip install jieba` |
| **按标点/空格** | 仅对含空格的短语有效 | 英文或中英混合 |

---

## 三、LIKE 查询封装

```python
def query_by_like(db, keyword: str, limit: int) -> list:
    """raw_content 中 LIKE '%keyword%'，注意万级数据时全表扫描"""
    cur = db.conn.execute(
        "SELECT * FROM records WHERE is_deleted=0 AND raw_content LIKE ? LIMIT ?",
        (f"%{keyword}%", limit),
    )
    return [dict(r) for r in cur.fetchall()]
```

---

## 四、注意事项

- `LIKE '%x%'` 无法用索引，数据量 &gt; 5000 时需评估性能
- 扫描件 PDF 无法提取正文，归档时跳过或标记 `content_type=scan`
- 数据量 &gt; 5000 时，考虑 Meilisearch、Elasticsearch 或 jieba+FTS 外部方案
