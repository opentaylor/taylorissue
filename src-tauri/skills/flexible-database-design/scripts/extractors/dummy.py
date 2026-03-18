"""
占位抽取器 - 不调用 LLM，仅做简单规则提取

用于测试或作为「无 LLM」时的默认实现。
用户可替换为 OpenAI/Claude/本地模型等实现。
"""


def extract(content: str) -> dict:
    """
    从 content 提取结构化数据。
    本实现：若内容含「标题：xxx」「标签：a,b」等则提取，否则返回空 dict。
    用户可替换为 LLM 实现（如 OpenAI/Claude）。
    """
    result = {}
    for line in content.strip().split("\n"):
        line = line.strip()
        if "：" in line or ":" in line:
            sep = "：" if "：" in line else ":"
            k, _, v = line.partition(sep)
            k = k.strip().lower().replace(" ", "_")
            v = v.strip()
            if k in ("title", "标题"):
                result["title"] = v
            elif k in ("tags", "标签"):
                result["tags"] = [x.strip() for x in v.replace("，", ",").split(",") if x.strip()]
            elif k in ("project", "项目"):
                result["project"] = v
    return result
