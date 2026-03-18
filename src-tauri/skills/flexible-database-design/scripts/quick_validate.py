#!/usr/bin/env python3
"""
快速验证脚本 - 一键执行：建表 → 归档 → 查询 → 断言
用于 CI 或本地验证，确保核心流程可用。
"""

import os
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from flexible_db import FlexibleDatabase


def main():
    # 使用临时 db 避免污染正式数据
    with tempfile.TemporaryDirectory() as tmp:
        db_path = os.path.join(tmp, "validate.db")
        os.environ["FLEXIBLE_DB_PATH"] = db_path

        db = FlexibleDatabase()
        assert os.path.exists(db_path), "建表失败：db 文件未创建"

        # 1. 归档
        ok, rid = db.archive_item(
            content="quick_validate 测试",
            source="validate",
            extracted_data={"title": "验证", "tags": ["test"]},
        )
        assert ok, f"归档失败: {rid}"
        record_id = rid

        # 2. 查询
        rows = db.list_all(limit=5)
        assert len(rows) >= 1, "list_all 无结果"
        assert any(r["record_id"] == record_id for r in rows), "归档记录未在 list 中"

        rows = db.query_dynamic(field_name="tags", field_value="test", limit=5)
        assert len(rows) >= 1, "query_dynamic 无结果"

        # 3. 分页
        rows = db.list_all(limit=1, offset=0)
        assert len(rows) == 1, "分页 limit/offset 异常"

        # 4. 更新
        ok, result = db.update_extracted(
            record_id, {"title": "验证v2", "tags": ["test", "updated"]}
        )
        assert ok, f"更新失败: {result}"

        # 5. 软删除与恢复
        ok, _ = db.soft_delete(record_id)
        assert ok, "软删除失败"
        rows = db.list_all(limit=10)
        assert not any(r["record_id"] == record_id for r in rows), "软删除后仍可见"

        ok, _ = db.restore(record_id)
        assert ok, "恢复失败"
        rows = db.list_all(limit=10)
        assert any(r["record_id"] == record_id for r in rows), "恢复后不可见"

        db.close()

    print("[OK] quick_validate 全部通过")
    return 0


if __name__ == "__main__":
    sys.exit(main())
