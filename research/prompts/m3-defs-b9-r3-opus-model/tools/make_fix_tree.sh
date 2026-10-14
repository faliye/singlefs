#!/usr/bin/env bash
# 试改树：拿给的副本 cp -al 一份，只把 .claude/hooks/lib_abandoned_rounds.py 换成断开硬链接的拷贝再改（改法 F-Q4a，只在本腿模型上量过、被攻过零轮）：
#   compared_form 去掉改名表新旧名与日期之后，再去掉剩下的空全角括号「（）」——doc-consolidate.py migrate 删日期时（history-consolidate.py 的 strip_same_date）把「（<日期>）」连括号一起删。
# 用法：make_fix_tree.sh <副本根> <试改树>
set -euo pipefail
source_root="$1" tree="$2"
rm -rf "$tree"; cp -al "$source_root" "$tree"
lib="$tree/.claude/hooks/lib_abandoned_rounds.py"
cp --remove-destination "$source_root/.claude/hooks/lib_abandoned_rounds.py" "$lib"
python3 - "$lib" <<'PY'
import sys
path = sys.argv[1]
text = open(path, encoding="utf-8").read()
old = '    return UNCOMPARED_CHARACTERS.sub("", removed.sub("", text))\n'
new = '    return UNCOMPARED_CHARACTERS.sub("", removed.sub("", text)).replace("（）", "")\n'
assert text.count(old) == 1, "compared_form 那一行对不上（这份试改只对第三轮冻结副本那一版）"
open(path, "w", encoding="utf-8").write(text.replace(old, new))
PY
echo "$tree"
