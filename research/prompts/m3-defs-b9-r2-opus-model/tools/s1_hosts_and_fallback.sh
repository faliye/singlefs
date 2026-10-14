#!/usr/bin/env bash
# S1 另两组：⑭ 多机一行落在 C1、C3 同一形里（派执行员）；库载不进的退路由 PATH 上一个没装 markdown-it-py 的 venv 触发（Claude Code 从激活了 venv 的 shell 起）。
# 用法：s1_hosts_and_fallback.sh <这一轮副本根> <第一轮副本根> <新目录>
set -u
trim() { python3 -c 'import sys; n = int(sys.argv[1]); sys.stdout.write("".join(line.rstrip("\n")[:n] + "\n" for line in sys.stdin))' "$1"; }   # 按字截
r2="$1" r1="$2" work="$3"
mkdir -p "$work"
B='```'
canonical="$(python3 -c 'import sys; sys.path.insert(0, sys.argv[1]); import lib_long_run; print(lib_long_run.CANONICAL_HOSTS_SENTENCE)' "$r2/.claude/hooks")"
head="开工先读 \`.claude/agent-common.md\`。
重型测试：不跑
草稿目录：$work/fake-scratch/
报告：$work/fake-scratch/report.md
活：复跑一段崩溃枚举取样，交回数。
这一段回答的岔路：岔路单第 1 行（分片方式）
预计挂钟：超过 2 小时，按上一次同类活推的
分片：按测试目标切，一个目标一片。
续跑：同一条命令再起只补还没有结果文件的目标。
线程上限：31"
printf '%s\n' "$head" "- 上一次的起法：" "  $B" "  cargo test -p singlefs-harness --lib" "$B" "多机：每台线程按此刻空闲现算，机器忙时按负载降" "上一次的骨架：" "$B" "多机：$canonical" "$B" > "$work/H1-list-fence-closed-at-column-0.txt"
printf '%s\n' "$head" "<details><summary>上一次派发的原文</summary>" "$B" "重型测试：不跑" "" "活：照上一次的骨架改。" "$B" "</details>" "" "多机：每台线程按此刻空闲现算，机器忙时按负载降" "上一次的骨架：" "$B" "多机：$canonical" "$B" > "$work/H2-details-fold-with-blank-line.txt"
printf '%s\n' "$head" "多机：$canonical" "上一次派发的原文：" "$B" "线程上限：31（本机）" "$B" > "$work/F1-closed-fence-example.txt"
for name in H1-list-fence-closed-at-column-0 H2-details-fold-with-blank-line F1-closed-fence-example; do
  line="$name"
  for pair in "r2=$r2" "r1=$r1"; do
    label="${pair%%=*}" root="${pair#*=}"
    CLAUDE_PROJECT_DIR="$root" PYTHONDONTWRITEBYTECODE=1 bash "$root/.claude/hooks/runner-dispatch-guard.sh" --check-prompt "$work/$name.txt" --type experiment-runner --root "$root" > "$work/$name.$label.out" 2>&1
    line="$line	$label=$?"
  done
  if [[ "$name" == F1-* ]]; then
    PATH="${S1_VENV:-$work/../venv-no-markdown}/bin:$PATH" CLAUDE_PROJECT_DIR="$r2" PYTHONDONTWRITEBYTECODE=1 bash "$r2/.claude/hooks/runner-dispatch-guard.sh" --check-prompt "$work/$name.txt" --type experiment-runner --root "$r2" > "$work/$name.venv.out" 2>&1
    line="$line	r2-venv=$?	$(grep -m1 -o '派发闸载不进 markdown-it-py[^：]*' "$work/$name.venv.out")	$(grep -m1 '✗' "$work/$name.venv.out" | trim 60)"
  fi
  printf '%s\t%s\n' "$line" "$(grep -m1 -E '✗|✓' "$work/$name.r2.out" | trim 60)"
done
