# FlexibleDatabase API 文档

| 版本 | 修改日期 | 修改人 | 修改内容 |
|------|----------|--------|----------|
| 1.0 | 2025-03-07 | Mars | 创建文档 |

---

## 一、模块与初始化

### `flexible_db.FlexibleDatabase`

```python
from flexible_db import FlexibleDatabase

db = FlexibleDatabase(db_path=None)
# db_path 可选，默认从 FLEXIBLE_DB_PATH 或 data/flexible.db 解析
```

**初始化行为**：自动建表（执行 schema_template.sql）、可选启用 FTS（`FLEXIBLE_DB_FTS=1`）、启用 WAL 模式、连接超时 30 秒。

---

## 二、公开方法

### 1. archive_item

归档一条记录。

```python
def archive_item(
    self,
    content: str,
    source: str = "manual",
    source_type: str = "manual",
    content_type: str = "text",
    extracted_data: Dict = None,
    confidence: float = 1.0,
    skip_duplicate_check: bool = False,
    _batch_commit: bool = True,
) -> Tuple[bool, Any]:
```

| 参数 | 类型 | 说明 |
|------|------|------|
| content | str | 原始内容（必填） |
| source | str | 来源标识 |
| source_type | str | 来源类型 |
| content_type | str | 内容类型：text/link/image/file/mixed |
| extracted_data | dict | 结构化 JSON，会展平到 dynamic_data |
| confidence | float | 置信度 0-1 |
| skip_duplicate_check | bool | 跳过 raw_content_hash 去重 |
| _batch_commit | bool | 内部用，批量导入时=False |

**返回**：`(True, record_id)` 或 `(False, 错误信息)`

**示例**：

```python
ok, rid = db.archive_item("测试", source="manual")
ok, rid = db.archive_item("测试", extracted_data={"title": "标题", "tags": ["a"]})
```

---

### 2. import_batch

批量导入。

```python
def import_batch(
    self,
    items: List[Dict],
    source: str = "import",
    source_type: str = "import",
    skip_duplicates: bool = True,
) -> Tuple[int, int]:
```

| 参数 | 说明 |
|------|------|
| items | 每项需含 `content` 或 `raw_content`，可选 `extracted`、`source`、`source_type` |
| source | 默认来源 |
| skip_duplicates | 是否按 content hash 去重 |

**返回**：`(成功数, 失败数)`

---

### 3. query_dynamic

按分类或字段查询 dynamic_data。

```python
def query_dynamic(
    self,
    category: str = None,
    field_name: str = None,
    field_value: str = None,
    limit: int = 50,
    offset: int = 0,
    exact_match: bool = False,
) -> List[Dict]:
```

| 参数 | 说明 |
|------|------|
| category | record_category 筛选 |
| field_name | 字段名筛选 |
| field_value | 字段值筛选；exact_match=False 时 LIKE 模糊（%/_ 转义），True 时精确 |
| limit, offset | 分页 |
| exact_match | 字段值精确匹配 |

**返回**：`[{"record_id", "field_name", "field_value", "raw_content", "source", "created_at", ...}, ...]`

---

### 4. list_all

列出最近记录。

```python
def list_all(self, limit: int = 30, offset: int = 0) -> List[Dict]:
```

**返回**：`[{"record_id", "source", "content_type", "raw_content", "created_at", "extracted"}, ...]`

---

### 5. get_categories / get_field_names

探索数据结构。

```python
def get_categories(self) -> List[str]:
def get_field_names(self) -> List[str]:
```

---

### 6. update_extracted

更新记录的 extracted 并同步 dynamic_data。

```python
def update_extracted(self, record_id: str, extracted_data: Dict) -> Tuple[bool, Any]:
```

**返回**：`(True, record_id)` 或 `(False, 错误信息)`

---

### 7. soft_delete / restore

软删除与恢复。

```python
def soft_delete(self, record_id: str) -> Tuple[bool, str]:
def restore(self, record_id: str) -> Tuple[bool, str]:
```

---

### 8. search_fulltext

全文检索（需 FLEXIBLE_DB_FTS=1）。

```python
def search_fulltext(self, query: str, limit: int = 20) -> List[Dict]:
```

**返回**：未启用 FTS 时返回 `[]`。

---

### 9. recall

召回相关记录（FTS 优先，否则 LIKE 模糊匹配）。

```python
def recall(self, query: str, limit: int = 5) -> List[Dict]:
```

### 10. get_stats

获取统计信息。

```python
def get_stats(self) -> Dict[str, Any]:
```

返回 `{"total_records": int, "by_category": {str: int}}`。

### 11. 上下文管理器

```python
with FlexibleDatabase() as db:
    db.archive_item("测试", source="manual")
# 自动 close
```

### 12. close

关闭连接。

```python
def close(self) -> None:
```

---

## 三、辅助函数

### `_resolve_db_path(db_path=None) -> str`

解析数据库路径：参数 > 环境变量 `FLEXIBLE_DB_PATH` > 默认 `data/flexible.db`。

---

## 四、关联文档

- [README](../README.md)
