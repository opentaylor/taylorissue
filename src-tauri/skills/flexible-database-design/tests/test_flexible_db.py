"""FlexibleDatabase 单元测试（使用 unittest，无需 pytest）"""
import os
import sys
import tempfile
import unittest

# 确保 scripts 在 path 中
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "scripts"))
from flexible_db import FlexibleDatabase, _resolve_db_path


def _mkdb():
    fd, path = tempfile.mkstemp(suffix=".db")
    os.close(fd)
    return path


class TestResolveDbPath(unittest.TestCase):
    def test_explicit_path(self):
        self.assertEqual(_resolve_db_path("/tmp/x.db"), "/tmp/x.db")

    def test_env_override(self):
        os.environ["FLEXIBLE_DB_PATH"] = "/env/path.db"
        try:
            self.assertEqual(_resolve_db_path(), "/env/path.db")
        finally:
            os.environ.pop("FLEXIBLE_DB_PATH", None)

    def test_explicit_over_env(self):
        os.environ["FLEXIBLE_DB_PATH"] = "/env/path.db"
        try:
            self.assertEqual(_resolve_db_path("/explicit.db"), "/explicit.db")
        finally:
            os.environ.pop("FLEXIBLE_DB_PATH", None)


class TestArchiveItem(unittest.TestCase):
    def setUp(self):
        self.db_path = _mkdb()
        self.db = FlexibleDatabase(self.db_path)

    def tearDown(self):
        self.db.close()
        try:
            os.unlink(self.db_path)
        except OSError:
            pass

    def test_archive_plain(self):
        ok, rid = self.db.archive_item("测试内容", source="test")
        self.assertTrue(ok)
        self.assertIsInstance(rid, str)
        self.assertEqual(len(rid), 32)

    def test_archive_with_extracted(self):
        ok, rid = self.db.archive_item(
            "内容",
            source="test",
            extracted_data={"title": "标题", "tags": ["a", "b"]},
        )
        self.assertTrue(ok)
        rows = self.db.query_dynamic(field_name="title", field_value="标题")
        self.assertGreaterEqual(len(rows), 1)

    def test_duplicate_skip(self):
        ok1, _ = self.db.archive_item("重复测试", source="test")
        ok2, msg = self.db.archive_item("重复测试", source="test")
        self.assertTrue(ok1)
        self.assertFalse(ok2)
        self.assertIn("重复", str(msg))

    def test_skip_duplicate_check(self):
        ok1, _ = self.db.archive_item("同内容", source="test", skip_duplicate_check=True)
        ok2, _ = self.db.archive_item("同内容", source="test", skip_duplicate_check=True)
        self.assertTrue(ok1)
        self.assertTrue(ok2)


class TestQueryDynamic(unittest.TestCase):
    def setUp(self):
        self.db_path = _mkdb()
        self.db = FlexibleDatabase(self.db_path)

    def tearDown(self):
        self.db.close()
        try:
            os.unlink(self.db_path)
        except OSError:
            pass

    def test_list_all(self):
        self.db.archive_item("a", source="s1")
        self.db.archive_item("b", source="s2")
        rows = self.db.list_all(limit=10)
        self.assertGreaterEqual(len(rows), 2)

    def test_offset(self):
        self.db.archive_item("o1", source="s")
        self.db.archive_item("o2", source="s")
        rows = self.db.list_all(limit=1, offset=0)
        self.assertEqual(len(rows), 1)
        rows2 = self.db.list_all(limit=1, offset=1)
        self.assertEqual(len(rows2), 1)
        self.assertNotEqual(rows[0]["record_id"], rows2[0]["record_id"])

    def test_exact_match(self):
        self.db.archive_item("x", source="s", extracted_data={"code": "100%"})
        rows = self.db.query_dynamic(field_name="code", field_value="100%", exact_match=True)
        self.assertGreaterEqual(len(rows), 1)
        rows2 = self.db.query_dynamic(field_name="code", field_value="100%x", exact_match=True)
        self.assertEqual(len(rows2), 0)

    def test_like_escape(self):
        """含 % 的模糊查询应转义：搜 "10%" 不应匹配 "100%" """
        self.db.archive_item("pct", source="s", extracted_data={"pct": "100%"})
        rows = self.db.query_dynamic(field_name="pct", field_value="100%", exact_match=False)
        self.assertGreaterEqual(len(rows), 1)
        # 未转义时 "10%" 会匹配 "100%"（% 匹配 0）；转义后 "10%" 为字面，不匹配
        rows2 = self.db.query_dynamic(field_name="pct", field_value="10%", exact_match=False)
        self.assertEqual(len(rows2), 0)


class TestSoftDeleteRestore(unittest.TestCase):
    def setUp(self):
        self.db_path = _mkdb()
        self.db = FlexibleDatabase(self.db_path)

    def tearDown(self):
        self.db.close()
        try:
            os.unlink(self.db_path)
        except OSError:
            pass

    def test_soft_delete_hides_from_list(self):
        ok, rid = self.db.archive_item("待删", source="test")
        self.assertTrue(ok)
        self.db.soft_delete(rid)
        rows = self.db.list_all(limit=10)
        self.assertFalse(any(r["record_id"] == rid for r in rows))

    def test_restore(self):
        ok, rid = self.db.archive_item("待恢复", source="test")
        self.db.soft_delete(rid)
        ok_r, _ = self.db.restore(rid)
        self.assertTrue(ok_r)
        rows = self.db.list_all(limit=10)
        self.assertTrue(any(r["record_id"] == rid for r in rows))


class TestUpdateExtracted(unittest.TestCase):
    def setUp(self):
        self.db_path = _mkdb()
        self.db = FlexibleDatabase(self.db_path)

    def tearDown(self):
        self.db.close()
        try:
            os.unlink(self.db_path)
        except OSError:
            pass

    def test_update_refreshes_dynamic(self):
        ok, rid = self.db.archive_item("u", source="s", extracted_data={"k": "v1"})
        self.assertTrue(ok)
        self.db.update_extracted(rid, {"k": "v2", "x": "y"})
        rows = self.db.query_dynamic(field_name="k", field_value="v2")
        self.assertGreaterEqual(len(rows), 1)
        rows_x = self.db.query_dynamic(field_name="x", field_value="y")
        self.assertGreaterEqual(len(rows_x), 1)


class TestContextManager(unittest.TestCase):
    def setUp(self):
        self.db_path = _mkdb()

    def tearDown(self):
        try:
            os.unlink(self.db_path)
        except OSError:
            pass

    def test_with_statement(self):
        from flexible_db import FlexibleDatabase
        with FlexibleDatabase(self.db_path) as db:
            ok, _ = db.archive_item("ctx_test", source="test")
            self.assertTrue(ok)
        # 验证数据已写入
        db2 = FlexibleDatabase(self.db_path)
        rows = db2.list_all(limit=5)
        db2.close()
        self.assertTrue(any("ctx_test" in (r.get("raw_content") or "") for r in rows))


class TestRecallAndStats(unittest.TestCase):
    def setUp(self):
        self.db_path = _mkdb()
        self.db = FlexibleDatabase(self.db_path)
        self.db.archive_item("召回测试内容", source="s", extracted_data={"title": "测试"})
        self.db.archive_item("无关内容", source="s")

    def tearDown(self):
        self.db.close()
        try:
            os.unlink(self.db_path)
        except OSError:
            pass

    def test_recall_like_fallback(self):
        rows = self.db.recall("召回测试", limit=5)
        self.assertGreaterEqual(len(rows), 1)
        self.assertIn("召回测试", rows[0]["raw_content"])

    def test_get_stats(self):
        stats = self.db.get_stats()
        self.assertIn("total_records", stats)
        self.assertIn("by_category", stats)
        self.assertGreaterEqual(stats["total_records"], 2)


class TestImportBatch(unittest.TestCase):
    def setUp(self):
        self.db_path = _mkdb()
        self.db = FlexibleDatabase(self.db_path)

    def tearDown(self):
        self.db.close()
        try:
            os.unlink(self.db_path)
        except OSError:
            pass

    def test_batch_import(self):
        items = [
            {"content": "b1", "extracted": {"t": "1"}},
            {"content": "b2", "extracted": {"t": "2"}},
        ]
        ok, fail = self.db.import_batch(items)
        self.assertEqual(ok, 2)
        self.assertEqual(fail, 0)

    def test_batch_skip_empty(self):
        items = [{"content": "x"}, {"content": ""}, {"content": "y"}]
        ok, fail = self.db.import_batch(items)
        self.assertEqual(ok, 2)
        self.assertEqual(fail, 1)


if __name__ == "__main__":
    unittest.main()
