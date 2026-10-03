"""从模型 run.py 里取「只凭语法切函数」那一段（strip 到 digest），不跑它的 main。"""
src = open("/home/fy5090/code/singlefs/research/prompts/tooling-broad-mutation-prune-r2-opus-model/run.py", encoding="utf-8").read()
import re, hashlib
ITEM_AT_LINE_START = re.compile(r"^[ \t]*(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?(impl|trait)\b", re.M)
exec(src[src.index("def strip("):src.index("# ---------- 树、测试、覆盖")])
