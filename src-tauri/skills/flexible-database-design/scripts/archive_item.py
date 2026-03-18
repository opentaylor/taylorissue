#!/usr/bin/env python3
"""
归档脚本 - 将内容写入灵活数据库
支持：纯原文 / 原文 + 结构化 JSON / 归档前备份
"""

import argparse
import json
import shutil
import sys
import os
from datetime import datetime

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from flexible_db import FlexibleDatabase, _resolve_db_path

try:
    from extractors import load_extractor
except ImportError:
    load_extractor = None


def _do_backup(db_path: str) -> bool:
    """备份数据库文件"""
    suffix = datetime.now().strftime("%Y%m%d_%H%M%S")
    backup_path = f"{db_path}.bak.{suffix}"
    try:
        shutil.copy2(db_path, backup_path)
        print(f"[OK] 已备份至 {backup_path}")
        return True
    except Exception as e:
        print(f"[WARN] 备份失败: {e}")
        return False


def archive(
    content: str,
    source: str = "manual",
    source_type: str = "manual",
    extracted_json: str = None,
    confidence: float = 1.0,
    backup: bool = False,
    llm_extract: bool = False,
    extractor_spec: str = None,
):
    """归档一条记录"""
    db_path = _resolve_db_path()
    if backup and os.path.exists(db_path):
        _do_backup(db_path)

    db = FlexibleDatabase()

    extracted = None
    if extracted_json:
        try:
            extracted = json.loads(extracted_json)
        except json.JSONDecodeError as e:
            print(f"[WARN] JSON 解析失败: {e}")
            return False
    elif llm_extract and load_extractor:
        fn = load_extractor(extractor_spec)
        if fn:
            try:
                extracted = fn(content)
                if extracted:
                    print(f"[INFO] 抽取: {list(extracted.keys())}")
            except Exception as e:
                print(f"[WARN] 抽取失败: {e}")

    success, result = db.archive_item(
        content=content,
        source=source,
        source_type=source_type,
        extracted_data=extracted,
        confidence=confidence,
    )

    if success:
        print(f"[OK] 归档成功 | record_id: {result}")
        if extracted:
            print(f"   分类: {extracted.get('data_type') or extracted.get('category') or '-'}")
    else:
        print(f"[FAIL] 归档失败: {result}")

    db.close()
    return success


def main():
    parser = argparse.ArgumentParser(
        description="归档内容到灵活数据库（支持原文 + 结构化 JSON）",
        epilog="""
示例:
  # 仅存原文
  python archive_item.py -c "今天学到：软 Schema 设计" -s "笔记"

  # 原文 + 结构化数据（知识库/碎片）
  python archive_item.py -c "某想法" -s "灵感" -e '{"title":"想法","tags":["工作"],"project":"项目A"}'

  # 多源聚合风格
  python archive_item.py -c "某价格消息" -s "群A" -e '{"data_type":"价格","items":[{"name":"A","value":100}]}'

  # 从文件读取（长文本/PDF 全文）
  python archive_item.py -F report.txt -s "file" -e '{"title":"年报摘要"}'
        """,
    )
    parser.add_argument("--content", "-c", help="原始内容（与 -F 二选一）")
    parser.add_argument("--file", "-F", help="从文件读取内容，解决长文本/命令行长度限制")
    parser.add_argument("--source", "-s", default="manual", help="来源标识")
    parser.add_argument("--source-type", "-t", default="manual", help="来源类型")
    parser.add_argument("--extracted", "-e", help="结构化 JSON（可选）")
    parser.add_argument("--confidence", type=float, default=1.0, help="置信度 0-1")
    parser.add_argument("--backup", "-b", action="store_true", help="归档前备份数据库")
    parser.add_argument("--llm-extract", "-L", action="store_true", help="使用抽取器从原文生成 extracted（需配置 FLEXIBLE_EXTRACTOR 或默认 dummy）")
    parser.add_argument("--extractor", help="抽取器 module:function，覆盖环境变量")

    args = parser.parse_args()
    content = args.content
    if args.file:
        if not os.path.exists(args.file):
            print(f"[FAIL] 文件不存在: {args.file}")
            sys.exit(1)
        with open(args.file, "r", encoding="utf-8", errors="replace") as f:
            content = f.read()
    if not content:
        print("[FAIL] 请提供 -c 或 -F 指定内容")
        parser.print_help()
        sys.exit(1)
    archive(
        content=content,
        source=args.source,
        source_type=args.source_type,
        extracted_json=args.extracted,
        confidence=args.confidence,
        backup=args.backup,
        llm_extract=args.llm_extract,
        extractor_spec=args.extractor,
    )


if __name__ == "__main__":
    main()
