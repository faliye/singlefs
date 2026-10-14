#!/usr/bin/env bash
# 几条零散的现查：C-2 登记读不到时 ⑮ 放行；C-4 SendMessage 钩子不看线程上限；C-7 ⑥ 不收旧日期行时的说明；C-9 冻结表里四条旧行的轮有没有快照目录。
# 用法：adhoc.sh <冻结副本根> <make_gate_trees.sh 造的树根> <草稿目录（要不存在）>
set -euo pipefail
frozen="${1:?冻结副本根}"; trees="${2:?树根}"; work="${3:?草稿目录}"
[[ ! -e "$work" ]] || { echo "  ✗ $work 已存在" >&2; exit 2; }
mkdir -p "$work"
cut40() { python3 -I -c 'import sys; [print(line.rstrip("\n")[:150]) for line in sys.stdin]'; }
printf '重型测试：不跑\n开工先读 `.claude/agent-common.md`\n草稿目录：%s/x/\n报告：%s/x/r.md\n预计挂钟：2 小时内\n线程上限：16\n' "$work" "$work" > "$work/p-cap16.md"
echo "--- C-2 登记读不到：线程上限：16 的 clerk 提示"
(set +e; cd / && RUNNER_DISPATCH_GUARD_HEAVY_THREADS_LIBRARY=/nonexistent/heavy_threads.py bash "$frozen/.claude/hooks/runner-dispatch-guard.sh" --check-prompt "$work/p-cap16.md" --type clerk --root "$frozen" 2>&1 | grep -E '^(!|  ✓|✗ 派)' | cut40; echo "退出=${PIPESTATUS[0]}")
echo "--- C-4 continuation-guard.sh 里的「线程」"
(cd "$frozen" && grep -c '线程' .claude/hooks/continuation-guard.sh || true)
echo "--- C-7 legacy-sourceless 树：派发闸 ⑥ 的说明行"
legacy="$trees/legacy-sourceless"; mkdir -p "$legacy/.claude"; [[ -d "$legacy/.claude/agents" ]] || cp -r "$frozen/.claude/agents" "$legacy/.claude/agents"
printf '重型测试：不跑\n草稿目录：%s/y/\n报告：%s/y/r.md\n条款：x\n要动的 crates 文件：crates/foo/src/lib.rs\n出口：x\n预计挂钟：2 小时内\n线程上限：31\n' "$work" "$work" > "$work/p-legacy.md"
(set +e; cd / && bash "$frozen/.claude/hooks/runner-dispatch-guard.sh" --check-prompt "$work/p-legacy.md" --type implementation-writer --root "$legacy" 2>&1 | grep -E '^(!|✗ 派)' | cut40; echo "退出=${PIPESTATUS[0]}")
echo "--- C-9 冻结表里四条旧行的轮：快照目录 / 正文 / 判决"
for round in c196-r1 d16-item1-r2 c364-r3 smalldata-ttime-r1 m3-batch63-code-r1; do
  printf '%s snapshot:%s body:%s verdict:%s\n' "$round" "$([[ -d "$frozen/research/prompts/$round-snapshot" ]] && echo yes || echo no)" "$([[ -f "$frozen/research/prompts/_$round-body.md" ]] && echo yes || echo no)" "$([[ -e "$frozen/research/prompts/$round-main-verification.md" ]] && echo yes || echo no)"
done
echo "--- C-3 dispatch-prompt.py 打的骨架里的线程上限与预计挂钟四行（新冻结副本；OLD_FROZEN 给了的再打旧副本）"
skeleton_lines() { # <副本根> <类型>
  (cd / && python3 -I "$1/scripts/agents/dispatch-prompt.py" "$2" 2>&1 | grep -E '^(线程上限|预计挂钟|分片|续跑|多机)：' | python3 -I -c 'import sys; [print(line.rstrip("\n")[:60]) for line in sys.stdin]') || true
}
for kind in clerk three-way-forward three-way-defense; do
  echo "[新] $kind"; skeleton_lines "$frozen" "$kind"
  if [[ -n "${OLD_FROZEN:-}" ]]; then echo "[旧] $kind"; skeleton_lines "$OLD_FROZEN" "$kind"; fi
done
