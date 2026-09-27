#!/usr/bin/env bash
# defs-gatebatch-m2-r3 云端攻方的探针复跑。被判的文件只读；小仓、镜像仓与小 crate 都建在草稿目录。
# 用法（在仓根）：bash research/prompts/defs-gatebatch-m2-r3-opus-model/rerun.sh [<输出目录>] [<草稿目录>]
#   默认输出目录是本目录的 outputs/，草稿目录 /tmp/claude-1000/defs-gatebatch-m2-r3-opus。
#   R3_ADMISSION / R3_HOOK 指到副本时攻副本（副本上的数不算入库装置上的数）。
# 重型测试一条都不跑：闸只喂 JSON；verify_semantics.sh 编、跑的是草稿目录里自己写的小 crate（经 run-with-memory-cap.sh 2G）；
# verify_systemd_run.sh 用 systemd-run --user --wait / --scope 起 printenv / printf。
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repository="$(cd "$here/../../.." && pwd)"
out="${1:-$here/outputs}"
export R3_SCRATCH="${2:-/tmp/claude-1000/defs-gatebatch-m2-r3-opus}"
export PYTHONDONTWRITEBYTECODE=1
mkdir -p "$out" "$R3_SCRATCH"
cd "$repository" || exit 1
for probe in l1-hook l1-p5 l1-attrs l1-manifest d2-trace marker-fields; do
  script="$here/probe_${probe//-/_}.py"
  nice -n 19 python3 "$script" > "$out/$probe.log" 2>&1
  echo "$probe rc=$? $(tail -1 "$out/$probe.log")"
done
nice -n 19 bash research/scripts/run-with-memory-cap.sh 2G bash "$here/verify_semantics.sh" "$R3_SCRATCH" > "$out/semantics.log" 2>&1
echo "semantics rc=$? $(grep -c '^SEM' "$out/semantics.log") 行 SEM"
bash "$here/verify_systemd_run.sh" > "$out/systemd-run.log" 2>&1
echo "systemd-run rc=$? $(grep -c '^SEM' "$out/systemd-run.log") 行 SEM"
# 攻方改法（副本上的数）：F1 F2 F3 F5 F10 套到 hooks 副本上再跑闸那两份探针，另跑副本的两份自证
fix_hook="$(bash "$here/make_fix_copy.sh" "$repository" "$R3_SCRATCH")" || { echo "make_fix_copy 失败"; exit 1; }
R3_HOOK="$fix_hook" nice -n 19 python3 "$here/probe_l1_hook.py" > "$out/fix-l1-hook.log" 2>&1; echo "fix-l1-hook rc=$? $(tail -1 "$out/fix-l1-hook.log")"
R3_HOOKS_SOURCE="$(dirname "$fix_hook")" nice -n 19 python3 "$here/probe_l1_p5.py" > "$out/fix-l1-p5.log" 2>&1; echo "fix-l1-p5 rc=$? $(tail -1 "$out/fix-l1-p5.log")"
nice -n 19 bash "$fix_hook" --selftest > "$out/fix-guard-selftest.log" 2>&1; echo "fix-guard-selftest rc=$? $(tail -1 "$out/fix-guard-selftest.log" | cut -c1-60)"
