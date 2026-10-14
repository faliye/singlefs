#!/usr/bin/env bash
# S1：库载不进时的退路。用一个 PATH 前置的 python3 垫片（/usr/bin/python3 -S，不加载 dist-packages）让 import markdown_it 真的失败（不靠弄坏开关），
# 对三份提示各判一次，比正常 PATH 与垫片 PATH 下的结果与 stderr。用法：s1_library_missing.sh <b10 树> <垫片目录> <工作目录>
set -uo pipefail
tree="$1" shim="$2" work="$3"; mkdir -p "$work"
HEAD=$'重型测试：不跑\n开工先读 `.claude/agent-common.md`\n草稿目录：/home/fy5090/.local/share/singlefs/work/zz-nonexistent/\n报告：/home/fy5090/.local/share/singlefs/work/zz-nonexistent/r.md\n预计挂钟：2 小时内\n'
printf '%s' "$HEAD"$'线程上限：31\n上一次派发的原文：\n```\n线程上限：16（本机）\n```\n' > "$work/fenced-example.txt"
printf '%s' "$HEAD"$'线程上限：31\n上一次派发的原文：\n~~~\n线程上限：16（本机）\n~~~\n' > "$work/tilde-example.txt"
printf '%s' "$HEAD"$'线程上限：31\n' > "$work/plain.txt"
for name in fenced-example tilde-example plain; do
  for mode in normal shim; do
    if [[ $mode == shim ]]; then path="$shim:$PATH"; else path="$PATH"; fi
    out="$(PATH="$path" CLAUDE_PROJECT_DIR="$tree" PYTHONDONTWRITEBYTECODE=1 bash "$tree/.claude/hooks/runner-dispatch-guard.sh" --check-prompt "$work/$name.txt" --type clerk --root "$tree" 2>&1)"; code=$?
    refused="$(grep -o '会拒 [0-9]* 处（[^）]*）' <<<"$out" | head -1)"
    note="$(grep -c '载不进 markdown-it-py' <<<"$out")"
    printf '%s\tPATH=%s\t退出=%s\t%s\t载不进那句 stderr=%s 行\n' "$name" "$mode" "$code" "${refused:-（没有拒绝项）}" "$note"
  done
done
PATH="$shim:$PATH" python3 -c 'import sys; print("垫片 python3 的 sys.flags.no_site =", sys.flags.no_site)'
PATH="$shim:$PATH" python3 -c 'import markdown_it' 2>&1 | tail -1 || true
exit 0
