#!/usr/bin/env python3
"""
管理脚本 - 软删除、恢复、更新记录
"""

import argparse
import json
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from flexible_db import FlexibleDatabase


def soft_delete(record_id: str) -> bool:
    """软删除一条记录"""
    db = FlexibleDatabase()
    success, result = db.soft_delete(record_id)
    if success:
        print(f"[OK] 已软删除: {record_id}")
    else:
        print(f"[FAIL] {result}")
    db.close()
    return success


def restore(record_id: str) -> bool:
    """恢复已软删除的记录"""
    db = FlexibleDatabase()
    success, result = db.restore(record_id)
    if success:
        print(f"[OK] 已恢复: {record_id}")
    else:
        print(f"[FAIL] {result}")
    db.close()
    return success


def update_extracted(record_id: str, extracted_json: str) -> bool:
    """更新记录的 extracted 并同步 dynamic_data"""
    try:
        extracted = json.loads(extracted_json)
    except json.JSONDecodeError as e:
        print(f"[FAIL] JSON 解析失败: {e}")
        return False

    db = FlexibleDatabase()
    success, result = db.update_extracted(record_id, extracted)
    if success:
        print(f"[OK] 已更新: {record_id}")
    else:
        print(f"[FAIL] {result}")
    db.close()
    return success


def main():
    parser = argparse.ArgumentParser(
        description="管理灵活数据库记录（软删除、恢复、更新）",
        epilog="""
示例:
  python manage_item.py --delete <record_id>
  python manage_item.py --restore <record_id>
  python manage_item.py --update <record_id> -e '{"title":"新标题","tags":["a","b"]}'
        """,
    )
    parser.add_argument("--delete", "-d", metavar="RECORD_ID", help="软删除记录")
    parser.add_argument("--restore", "-r", metavar="RECORD_ID", help="恢复已删除记录")
    parser.add_argument("--update", "-u", metavar="RECORD_ID", help="更新 extracted")
    parser.add_argument("--extracted", "-e", help="结构化 JSON（配合 --update）")

    args = parser.parse_args()

    if args.delete:
        ok = soft_delete(args.delete)
    elif args.restore:
        ok = restore(args.restore)
    elif args.update:
        if not args.extracted:
            print("[FAIL] --update 需配合 --extracted 使用")
            sys.exit(1)
        ok = update_extracted(args.update, args.extracted)
    else:
        parser.print_help()
        sys.exit(0)

    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
