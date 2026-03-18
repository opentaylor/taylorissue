#!/usr/bin/env python3
"""
查询脚本 - 从灵活数据库按分类/字段/全文查询
"""

import argparse
import csv
import io
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from flexible_db import FlexibleDatabase


def list_all(limit: int = 30, offset: int = 0):
    """列出最近记录"""
    db = FlexibleDatabase()
    rows = db.list_all(limit, offset)
    print(f"\n[LIST] 最近 {len(rows)} 条记录")
    print("=" * 60)
    for r in rows:
        content = (r["raw_content"] or "")[:60]
        if len(r["raw_content"] or "") > 60:
            content += "..."
        print(f"  {r['created_at'][:19]} | {r['source']} | {content}")
    db.close()


def query_by_category(category: str, limit: int = 20, offset: int = 0):
    """按分类查询"""
    db = FlexibleDatabase()
    rows = db.query_dynamic(category=category, limit=limit, offset=offset)
    print(f"\n[CAT] 分类: {category} ({len(rows)} 条)")
    print("=" * 60)
    for r in rows:
        val = f"{r['field_value']}" + (f" {r['unit']}" if r["unit"] else "")
        print(f"  {r['created_at'][:10]} | {r['field_name']}: {val}")
        if r["raw_content"]:
            print(f"    原文: {(r['raw_content'][:50])}...")
    db.close()


def query_by_field(field_name: str, value: str = None, limit: int = 20, offset: int = 0, exact_match: bool = False):
    """按字段名查询（可带值筛选）"""
    db = FlexibleDatabase()
    rows = db.query_dynamic(field_name=field_name, field_value=value, limit=limit, offset=offset, exact_match=exact_match)
    print(f"\n[FIELD] 字段: {field_name}" + (f" ~ {value}" if value else "") + f" ({len(rows)} 条)")
    print("=" * 60)
    for r in rows:
        print(f"  {r['created_at'][:10]} | {r['field_value']} {r['unit'] or ''} | {r['source']}")
    db.close()


def show_stats():
    """输出总记录数、按分类统计"""
    db = FlexibleDatabase()
    stats = db.get_stats()
    db.close()
    print("\n[STATS] 数据概况")
    print("=" * 60)
    print(f"  总记录数: {stats['total_records']}")
    if stats.get("by_category"):
        print("  按分类:")
        for cat, cnt in sorted(stats["by_category"].items(), key=lambda x: -x[1]):
            print(f"    - {cat}: {cnt}")
    else:
        print("  按分类: (暂无)")


def show_schema(limit_cats: int = 20):
    """显示所有分类和字段名（探索数据结构）"""
    db = FlexibleDatabase()
    cats = db.get_categories()
    fields = db.get_field_names()
    print("\n[SCHEMA] 数据分类:")
    for c in (cats or ["(暂无)"]):
        print(f"   - {c}")
    print("\n[SCHEMA] 字段名:")
    for f in sorted(set(fields or ["(暂无)"]))[:limit_cats]:
        print(f"   - {f}")
    db.close()


def search_fulltext(query: str, limit: int = 20):
    """全文检索（需 FLEXIBLE_DB_FTS=1）"""
    db = FlexibleDatabase()
    rows = db.search_fulltext(query, limit)
    print(f"\n[FTS] 全文检索: {query} ({len(rows)} 条)")
    print("=" * 60)
    for r in rows:
        content = (r["raw_content"] or "")[:60]
        if len(r["raw_content"] or "") > 60:
            content += "..."
        print(f"  {r['created_at'][:19]} | {r['source']} | {content}")
    db.close()


def export_data(fmt: str, limit: int, output_path: str = None):
    """导出为 JSON 或 CSV"""
    db = FlexibleDatabase()
    rows = db.list_all(limit=limit)
    db.close()

    if fmt == "json":
        out = json.dumps(rows, ensure_ascii=False, indent=2)
    else:
        if not rows:
            out = "record_id,source,content_type,raw_content,created_at,extracted\n"
        else:
            buf = io.StringIO()
            w = csv.DictWriter(buf, fieldnames=["record_id", "source", "content_type", "raw_content", "created_at", "extracted"], extrasaction="ignore")
            w.writeheader()
            w.writerows(rows)
            out = buf.getvalue()

    if output_path:
        with open(output_path, "w", encoding="utf-8") as f:
            f.write(out)
        print(f"[OK] 已导出至 {output_path}")
    else:
        print(out)


def main():
    parser = argparse.ArgumentParser(description="查询灵活数据库")
    parser.add_argument("--list", "-l", action="store_true", help="列出最近记录")
    parser.add_argument("--category", "-c", help="按分类查询")
    parser.add_argument("--field", "-f", help="按字段名查询")
    parser.add_argument("--value", "-v", help="字段值筛选（配合 --field）")
    parser.add_argument("--search", help="全文检索（需 FLEXIBLE_DB_FTS=1）")
    parser.add_argument("--schema", "-s", action="store_true", help="显示所有分类和字段")
    parser.add_argument("--stats", action="store_true", help="显示总记录数、按分类统计")
    parser.add_argument("--limit", "-n", type=int, default=20, help="返回条数")
    parser.add_argument("--offset", "-o", type=int, default=0, help="分页偏移")
    parser.add_argument("--export", "-E", choices=["json", "csv"], help="导出为 JSON 或 CSV")
    parser.add_argument("--output", "-O", help="导出文件路径（配合 --export）")
    parser.add_argument("--exact", action="store_true", help="字段值精确匹配（配合 --field --value）")

    args = parser.parse_args()

    if args.export:
        export_data(args.export, args.limit, args.output)
        return

    if args.stats:
        show_stats()
        return

    if args.list:
        list_all(args.limit, args.offset)
    elif args.category:
        query_by_category(args.category, args.limit, args.offset)
    elif args.field:
        query_by_field(args.field, args.value, args.limit, args.offset, args.exact)
    elif args.search:
        search_fulltext(args.search, args.limit)
    elif args.schema:
        show_schema()
    else:
        # 默认：显示 schema 提示
        print("用法: --list | --category 分类 | --field 字段名 [--value 值] | --search 关键词 | --schema | --stats")
        show_schema(10)


if __name__ == "__main__":
    main()
