#!/usr/bin/env bash
# Q1（第二轮 S2，判「站得住」）：dispatch-prompt.py 给 ⑮ 那九种类型与设计员各打一份骨架，照骨架写的「2 小时内删掉后三行」与「填完占位」填好，
# 再交 dispatch-prompt.py --check（起同一份派发闸 --check-prompt）判；另造三份「不起重活只写已判行」「要别的数写 N 加已判行」「要别的数只写已判行」看放拒。
# 不跑 --selftest。用法：q1_skeletons.sh <名>=<那一版副本根> <工作目录>
set -euo pipefail
label="${1%%=*}" root="${1#*=}" work="$2/$label"
rm -rf "$work"; mkdir -p "$work"
fill() {   # 骨架 → 填好：预计挂钟写 2 小时内、分片 / 续跑 / 多机三行删掉、别的占位填「已填」（照 dispatch-prompt.py 的 fill_for_guard 那一种填法）
  python3 -c 'import re, sys
out = []
for line in sys.stdin.read().split("\n"):
    if line.startswith("预计挂钟："): out.append("预计挂钟：2 小时内")
    elif not line.startswith(("分片：", "续跑：", "多机：")): out.append(re.sub(r"<填[^>]*>", "已填", line))
sys.stdout.write("\n".join(out))'
}
check() {   # 名 类型 提示文件
  local code=0 first
  ( cd "$root" && PYTHONDONTWRITEBYTECODE=1 python3 -B scripts/agents/dispatch-prompt.py --check "$3" --type "$2" --root "$root" ) > "$work/$1.out" 2>&1 || code=$?
  first="$(grep -m1 -E '✗|✓' "$work/$1.out" | python3 -c 'import sys; sys.stdout.write(sys.stdin.read().strip()[:110])')"
  printf '%s\t%s\t%s\t--check 退出=%s\t%s\n' "$label" "$1" "$2" "$code" "$first"
}
for type in experiment-designer implementation-writer investigator tooling-writer experiment-runner mutation-triage three-way-attack clerk three-way-forward three-way-defense; do
  ( cd "$root" && PYTHONDONTWRITEBYTECODE=1 python3 -B scripts/agents/dispatch-prompt.py "$type" ) > "$work/$type.skeleton" 2>&1 || true
  fill < "$work/$type.skeleton" | sed "s#已填#$work/scratch-$type/#" > "$work/$type.prompt"
  printf '%s\t%s\t骨架的线程上限行：%s\n' "$label" "$type" "$(grep -m1 '^线程上限' "$work/$type.skeleton" || echo 无)"
  check "骨架填好-$type" "$type" "$work/$type.prompt"
done
grep -v '^线程上限：' "$work/clerk.prompt" > "$work/clerk-valve-only.prompt"; echo '线程上限已判：这件活只读文件、整理输出，不编译、不跑测试' >> "$work/clerk-valve-only.prompt"
check clerk-不起重活只写已判行 clerk "$work/clerk-valve-only.prompt"
sed 's/^线程上限：.*/线程上限：16/' "$work/investigator.prompt" > "$work/investigator-user-16.prompt"; echo '线程上限已判：用户给了参数 16' >> "$work/investigator-user-16.prompt"
check investigator-要别的数写N加已判行 investigator "$work/investigator-user-16.prompt"
grep -v '^线程上限：' "$work/investigator.prompt" > "$work/investigator-valve-16.prompt"; echo '线程上限已判：用户给了参数 16' >> "$work/investigator-valve-16.prompt"
check investigator-要别的数只写已判行 investigator "$work/investigator-valve-16.prompt"
