#!/usr/bin/env bash
# 用法：run-samples.sh <树根> <输出目录> [样本名…]；照 stage-selftest.sh 的起法（拷进临时目录、setup.sh、cd 进样本根、根参数写同一个目录、--force）逐份起 <树根> 里的阶段。
# 环境里的 CHECKER_TIER_BROAD_MUTATION_BREAK 原样传给阶段。每份打一行：样本名 判对/判错 期望退出→实测 缺的 want 条数。
set -uo pipefail
tree="$1"; out="$2"; shift 2
stage="$tree/.claude/gate.d/checker-tier-broad-mutation.sh"
fx="$tree/.claude/gate.d/fixtures/checker-tier-broad-mutation.sh"
mkdir -p "$out"
if (($#)); then names=("$@"); else mapfile -t names < <(cd "$fx" && ls -1); fi
ok=0; bad=0; badlist=()
for n in "${names[@]}"; do
  d="$fx/$n"; work="$(mktemp -d /tmp/claude-1000/gov-r2-attack/work.XXXXXX)"
  cp -a "$d/." "$work/"
  if [[ -f "$work/setup.sh" ]]; then ( cd "$work" && env -u GATE_BASE -u GATE_STAGED_FROM -u GATE_DIFF_BASE bash setup.sh >/dev/null 2>&1 ) || echo "SETUP-FAIL $n"; fi
  got=0; o="$(cd "$work" && env -u GATE_BASE -u GATE_STAGED_FROM -u GATE_DIFF_BASE bash "$stage" "$work" --force 2>&1)" || got=$?
  rm -rf "${work:?}"
  printf '%s\nexit=%s\n' "$o" "$got" > "$out/$n.txt"
  want_exit="$(sed -n 's/^exit=//p' "$d/expect")"; miss=0
  while IFS= read -r w; do [[ -z "$w" ]] && continue; grep -qF -- "$w" <<<"$o" || miss=$((miss+1)); done < <(sed -n 's/^want=//p' "$d/expect")
  if [[ "$got" == "$want_exit" && $miss -eq 0 ]]; then ok=$((ok+1)); else bad=$((bad+1)); badlist+=("$n(期望${want_exit}→实测${got}，缺 want ${miss} 条)"); fi
done
echo "判对 $ok 个、判错 $bad 个：${badlist[*]:-无}"
