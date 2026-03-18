#!/usr/bin/env python3
"""
Flexible Database - 通用灵活数据库核心逻辑
支持：原始层 + JSON 软字段 + 键值对查询
"""

import logging
import sqlite3
import json
import hashlib
import os
from datetime import datetime
from typing import List, Dict, Optional, Tuple, Any

logger = logging.getLogger(__name__)


def _escape_like(value: str) -> str:
    """转义 LIKE 中的 % 和 _"""
    return value.replace("\\", "\\\\").replace("%", "\\%").replace("_", "\\_")


def _resolve_db_path(db_path: Optional[str] = None) -> str:
    """解析数据库路径：参数 > 环境变量 > 默认值"""
    if db_path:
        return db_path
    env_path = os.environ.get("FLEXIBLE_DB_PATH")
    if env_path:
        return env_path
    base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    return os.path.join(base_dir, "data", "flexible.db")


class FlexibleDatabase:
    """通用灵活数据库 - 不绑定具体业务"""

    def __init__(self, db_path: str = None):
        self.db_path = _resolve_db_path(db_path)
        db_dir = os.path.dirname(self.db_path)
        if db_dir:
            os.makedirs(db_dir, exist_ok=True)

        self.conn = sqlite3.connect(self.db_path, timeout=30.0)
        self.conn.row_factory = sqlite3.Row
        self.conn.execute("PRAGMA journal_mode=WAL")
        self._init_db()

    def __enter__(self) -> "FlexibleDatabase":
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        self.close()

    def _init_db(self):
        """执行 schema 初始化"""
        schema_path = os.environ.get("FLEXIBLE_DB_SCHEMA")
        if not schema_path:
            schema_path = os.path.join(
                os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                "references",
                "schema_template.sql",
            )

        if os.path.exists(schema_path):
            with open(schema_path, "r", encoding="utf-8") as f:
                self.conn.executescript(f.read())
            self.conn.commit()

        if os.environ.get("FLEXIBLE_DB_FTS", "").lower() in ("1", "true", "yes"):
            self._init_fts()

    def _init_fts(self) -> None:
        """初始化 FTS5 全文检索（需 SQLite 3.9+）"""
        self.conn.executescript("""
            CREATE VIRTUAL TABLE IF NOT EXISTS records_fts USING fts5(
                content = 'records',
                content_rowid = 'id',
                raw_content
            );
            CREATE TRIGGER IF NOT EXISTS records_fts_insert AFTER INSERT ON records BEGIN
                INSERT INTO records_fts(rowid, raw_content) VALUES (new.id, new.raw_content);
            END;
            CREATE TRIGGER IF NOT EXISTS records_fts_update AFTER UPDATE ON records BEGIN
                INSERT INTO records_fts(records_fts, rowid, raw_content) VALUES('delete', old.id, old.raw_content);
                INSERT INTO records_fts(rowid, raw_content) VALUES (new.id, new.raw_content);
            END;
            CREATE TRIGGER IF NOT EXISTS records_fts_delete AFTER DELETE ON records BEGIN
                INSERT INTO records_fts(records_fts, rowid, raw_content) VALUES('delete', old.id, old.raw_content);
            END;
        """)
        self.conn.commit()

    def _generate_record_id(self, content: str, source: str, created_at: str) -> str:
        return hashlib.md5(f"{content}|{source}|{created_at}".encode()).hexdigest()

    def _generate_content_hash(self, content: str) -> str:
        return hashlib.md5(content.encode()).hexdigest()

    def _flatten_to_dynamic(
        self,
        record_id: str,
        category: str,
        data: Dict,
        prefix: str = "",
        data_date: str = None,
    ) -> None:
        """将 JSON 递归展平为键值对写入 dynamic_data，批量 INSERT 提升性能"""
        rows: List[tuple] = []
        for key, value in data.items():
            if key.startswith("_") or value is None:
                continue
            full_key = f"{prefix}{key}" if prefix else key

            if isinstance(value, (int, float)):
                rows.append((record_id, category, full_key, str(value), "number", float(value), data_date))
            elif isinstance(value, bool):
                rows.append((record_id, category, full_key, "true" if value else "false", "boolean", None, data_date))
            elif isinstance(value, (list, dict)):
                rows.append((record_id, category, full_key, json.dumps(value, ensure_ascii=False), "json", None, data_date))
            elif isinstance(value, str):
                rows.append((record_id, category, full_key, value, "text", None, data_date))

        if rows:
            self.conn.executemany(
                """INSERT INTO dynamic_data 
                   (record_id, record_category, field_name, field_value, field_type, numeric_value, data_date)
                   VALUES (?, ?, ?, ?, ?, ?, ?)""",
                rows,
            )

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
        """
        归档一条记录。
        extracted_data 为 None 时，仅存原文；有则存 JSON 并展平到 dynamic_data。
        """
        try:
            created_at = datetime.now().isoformat()
            record_id = self._generate_record_id(content, source, created_at)
            content_hash = self._generate_content_hash(content)

            if not skip_duplicate_check:
                cur = self.conn.execute(
                    "SELECT record_id FROM records WHERE raw_content_hash = ?", (content_hash,)
                )
                if cur.fetchone():
                    logger.debug("重复内容已跳过: %s", record_id)
                    return False, f"重复内容，已跳过: {record_id}"

            category = "unknown"
            if extracted_data:
                # 支持多场景字段映射：report_type→category, report_date→data_date
                category = (
                    extracted_data.get("data_type")
                    or extracted_data.get("category")
                    or extracted_data.get("report_type")
                    or "general"
                )
                data_date = (
                    extracted_data.get("date")
                    or extracted_data.get("report_date")
                    or created_at[:10]
                )
            else:
                data_date = created_at[:10]

            self.conn.execute(
                """INSERT INTO records 
                   (record_id, source, source_type, content_type, raw_content, raw_content_hash, extracted, confidence_score)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    record_id,
                    source,
                    source_type,
                    content_type,
                    content,
                    content_hash,
                    json.dumps(extracted_data, ensure_ascii=False) if extracted_data else None,
                    confidence,
                ),
            )

            if extracted_data:
                self._flatten_to_dynamic(record_id, category, extracted_data, data_date=data_date)

            if _batch_commit:
                self.conn.commit()
            logger.debug("归档成功: %s", record_id)
            return True, record_id

        except sqlite3.IntegrityError as e:
            self.conn.rollback()
            logger.warning("归档失败(IntegrityError): %s", e)
            return False, str(e)
        except sqlite3.OperationalError as e:
            self.conn.rollback()
            logger.warning("归档失败(OperationalError): %s", e)
            return False, str(e)
        except sqlite3.Error as e:
            self.conn.rollback()
            logger.warning("归档失败(sqlite3.Error): %s", e)
            return False, str(e)

    def import_batch(
        self,
        items: List[Dict],
        source: str = "import",
        source_type: str = "import",
        skip_duplicates: bool = True,
    ) -> Tuple[int, int]:
        """
        批量导入。items 每项需含 content，可选 extracted、source 等。
        返回 (成功数, 失败数)。
        """
        ok, fail = 0, 0
        try:
            for item in items:
                content = item.get("content") or item.get("raw_content")
                if not content:
                    fail += 1
                    continue
                extracted = item.get("extracted")
                src = item.get("source", source)
                src_type = item.get("source_type", source_type)
                success, _ = self.archive_item(
                    content=content,
                    source=src,
                    source_type=src_type,
                    extracted_data=extracted,
                    skip_duplicate_check=not skip_duplicates,
                    _batch_commit=False,
                )
                if success:
                    ok += 1
                else:
                    fail += 1
            self.conn.commit()
        except Exception:
            self.conn.rollback()
            raise
        return ok, fail

    def query_dynamic(
        self,
        category: str = None,
        field_name: str = None,
        field_value: str = None,
        limit: int = 50,
        offset: int = 0,
        exact_match: bool = False,
    ) -> List[Dict]:
        """
        按分类或字段查询 dynamic_data，并关联原始记录。
        exact_match=True 时 field_value 精确匹配；否则 LIKE 模糊匹配（%/_ 会转义）。
        """
        conditions, params = [], []
        if category:
            conditions.append("d.record_category = ?")
            params.append(category)
        if field_name:
            conditions.append("d.field_name = ?")
            params.append(field_name)
        if field_value:
            if exact_match:
                conditions.append("d.field_value = ?")
                params.append(field_value)
            else:
                esc = _escape_like(field_value)
                conditions.append("d.field_value LIKE ? ESCAPE '\\'")
                params.append(f"%{esc}%")

        where = " AND ".join(conditions) if conditions else "1=1"
        params.extend([limit, offset])

        cur = self.conn.execute(
            f"""SELECT d.*, r.raw_content, r.source, r.created_at
                FROM dynamic_data d
                JOIN records r ON d.record_id = r.record_id
                WHERE {where} AND r.is_deleted = 0
                ORDER BY r.created_at DESC
                LIMIT ? OFFSET ?""",
            params,
        )
        return [dict(row) for row in cur.fetchall()]

    def list_all(self, limit: int = 30, offset: int = 0) -> List[Dict]:
        """列出最近记录"""
        cur = self.conn.execute(
            """SELECT record_id, source, content_type, raw_content, created_at, extracted
               FROM records WHERE is_deleted = 0 ORDER BY created_at DESC LIMIT ? OFFSET ?""",
            (limit, offset),
        )
        return [dict(row) for row in cur.fetchall()]

    def get_categories(self) -> List[str]:
        """获取所有 record_category"""
        cur = self.conn.execute(
            "SELECT DISTINCT record_category FROM dynamic_data WHERE record_category IS NOT NULL"
        )
        return [r[0] for r in cur.fetchall()]

    def get_field_names(self) -> List[str]:
        """获取所有 field_name"""
        cur = self.conn.execute("SELECT DISTINCT field_name FROM dynamic_data")
        return [r[0] for r in cur.fetchall()]

    def update_extracted(
        self, record_id: str, extracted_data: Dict
    ) -> Tuple[bool, Any]:
        """
        更新已有记录的 extracted，并同步刷新 dynamic_data。
        策略：先删 dynamic_data 中该 record_id 的键值对，再插入新数据。
        """
        try:
            cur = self.conn.execute(
                "SELECT id, created_at FROM records WHERE record_id = ? AND is_deleted = 0",
                (record_id,),
            )
            row = cur.fetchone()
            if not row:
                return False, f"记录不存在或已删除: {record_id}"

            self.conn.execute("DELETE FROM dynamic_data WHERE record_id = ?", (record_id,))
            self.conn.execute(
                "UPDATE records SET extracted = ? WHERE record_id = ?",
                (json.dumps(extracted_data, ensure_ascii=False), record_id),
            )

            category = (
                extracted_data.get("data_type")
                or extracted_data.get("category")
                or "general"
            )
            data_date = extracted_data.get("date") or datetime.now().isoformat()[:10]
            self._flatten_to_dynamic(
                record_id, category, extracted_data, data_date=data_date
            )

            self.conn.commit()
            return True, record_id

        except sqlite3.Error as e:
            self.conn.rollback()
            return False, str(e)

    def soft_delete(self, record_id: str) -> Tuple[bool, str]:
        """软删除一条记录"""
        try:
            cur = self.conn.execute(
                "UPDATE records SET is_deleted = 1 WHERE record_id = ?", (record_id,)
            )
            if cur.rowcount == 0:
                return False, f"记录不存在: {record_id}"
            self.conn.commit()
            return True, record_id
        except sqlite3.Error as e:
            self.conn.rollback()
            return False, str(e)

    def restore(self, record_id: str) -> Tuple[bool, str]:
        """恢复已软删除的记录"""
        try:
            cur = self.conn.execute(
                "UPDATE records SET is_deleted = 0 WHERE record_id = ?", (record_id,)
            )
            if cur.rowcount == 0:
                return False, f"记录不存在: {record_id}"
            self.conn.commit()
            return True, record_id
        except sqlite3.Error as e:
            self.conn.rollback()
            return False, str(e)

    def search_fulltext(self, query: str, limit: int = 20) -> List[Dict]:
        """
        全文检索 raw_content。需设置 FLEXIBLE_DB_FTS=1 启用 FTS 后可用。
        未启用时返回空列表。
        """
        try:
            cur = self.conn.execute(
                """SELECT r.record_id, r.raw_content, r.source, r.created_at
                   FROM records_fts f
                   JOIN records r ON r.id = f.rowid
                   WHERE records_fts MATCH ? AND r.is_deleted = 0
                   ORDER BY rank
                   LIMIT ?""",
                (query, limit),
            )
            return [dict(row) for row in cur.fetchall()]
        except sqlite3.OperationalError:
            return []

    def recall(self, query: str, limit: int = 5) -> List[Dict]:
        """
        召回相关记录。FTS 启用时用全文检索；否则用 raw_content LIKE 模糊匹配。
        """
        fts_rows = self.search_fulltext(query, limit)
        if fts_rows:
            return fts_rows

        esc = _escape_like(query)
        cur = self.conn.execute(
            """SELECT record_id, raw_content, source, created_at
               FROM records WHERE is_deleted = 0 AND raw_content LIKE ? ESCAPE '\\'
               ORDER BY created_at DESC LIMIT ?""",
            (f"%{esc}%", limit),
        )
        return [dict(row) for row in cur.fetchall()]

    def get_stats(self) -> Dict[str, Any]:
        """获取统计信息：总记录数、分类统计等"""
        cur = self.conn.execute(
            "SELECT COUNT(*) FROM records WHERE is_deleted = 0"
        )
        total = cur.fetchone()[0]

        cur = self.conn.execute(
            """SELECT record_category AS category, COUNT(DISTINCT record_id) AS cnt
               FROM dynamic_data WHERE record_category IS NOT NULL
               GROUP BY record_category"""
        )
        by_category = {r["category"]: r["cnt"] for r in cur.fetchall()}

        return {"total_records": total, "by_category": by_category}

    def close(self) -> None:
        self.conn.close()
