"""
抽取器模块 - 从 raw_content 提取结构化 JSON

接口约定：extractor(content: str) -> dict
返回的 dict 将作为 extracted 写入 records，并展平到 dynamic_data。

使用方式：
  - 环境变量 FLEXIBLE_EXTRACTOR=module:function
  - archive_item.py --llm-extract 时自动调用
"""

import logging
import os

from .dummy import extract as dummy_extract

__all__ = ["dummy_extract", "load_extractor"]
logger = logging.getLogger(__name__)


def load_extractor(spec: str = None):
    """
    加载抽取函数。spec 格式: module.path:function_name
    例如: extractors.dummy:extract
    未指定时从环境变量 FLEXIBLE_EXTRACTOR 读取。
    """
    spec = spec or os.environ.get("FLEXIBLE_EXTRACTOR") or "extractors.dummy:extract"
    if ":" not in spec:
        return None
    mod_path, func_name = spec.rsplit(":", 1)
    try:
        mod = __import__(mod_path, fromlist=[func_name])
        return getattr(mod, func_name)
    except (ImportError, AttributeError) as e:
        logger.warning("抽取器加载失败 %s: %s", spec, e)
        return None
