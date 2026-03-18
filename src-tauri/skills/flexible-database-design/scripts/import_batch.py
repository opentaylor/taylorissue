#!/usr/bin/env python3
"""
批量导入脚本 - 从 JSON/CSV 文件批量归档
"""

import argparse
import csv
import json
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from flexible_db import FlexibleDatabase


def load_json(path: str) -> list:
    """从 JSON 文件加载，支持 [{"content":"...","extracted":{}}] 或 {"items":[...]}"""
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
    if isinstance(data, list):
        return data
    if isinstance(data, dict) and "items" in data:
        return data["items"]
    return [data]


def load_csv(path: str) -> list:
    """从 CSV 加载，需含 content 列，可选 extracted（JSON 字符串）"""
    rows = []
    with open(path, "r", encoding="utf-8", newline="") as f:
        reader = csv.DictReader(f)
        for r in reader:
            item = {"content": r.get("content") or r.get("raw_content", "").strip()}
            if r.get("extracted"):
                try:
                    item["extracted"] = json.loads(r["extracted"])
                except json.JSONDecodeError:
                    pass
            if r.get("source"):
                item["source"] = r["source"]
            rows.append(item)
    return rows


def main():
    parser = argparse.ArgumentParser(
        description="批量导入到灵活数据库",
        epilog="""
示例:
  python import_batch.py data.json
  python import_batch.py data.csv --format csv
  python import_batch.py data.json --source "导入" --no-skip-dup
        """,
    )
    parser.add_argument("file", help="JSON 或 CSV 文件路径")
    parser.add_argument("--format", "-f", choices=["json", "csv", "auto"], default="auto", help="格式，auto 按扩展名推断")
    parser.add_argument("--source", "-s", default="import", help="默认来源")
    parser.add_argument("--no-skip-dup", action="store_true", help="不跳过重复内容")

    args = parser.parse_args()

    fmt = args.format
    if fmt == "auto":
        fmt = "json" if args.file.lower().endswith(".json") else "csv"

    if fmt == "json":
        items = load_json(args.file)
    else:
        items = load_csv(args.file)

    if not items:
        print("[WARN] 无数据")
        sys.exit(0)

    db = FlexibleDatabase()
    ok, fail = db.import_batch(
        items,
        source=args.source,
        skip_duplicates=not args.no_skip_dup,
    )
    db.close()

    print(f"[OK] 导入完成: 成功 {ok}, 跳过/失败 {fail}")
    sys.exit(0 if fail == 0 else 1)


if __name__ == "__main__":
    main()
