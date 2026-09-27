#!/usr/bin/env bash
# defs-gatebatch-m2-r2 云端攻方（Opus）探针复跑。被判的文件只读；小仓、副本都建在草稿目录。
#   bash rerun.sh <输出目录> [草稿目录]
# 输出目录里：k1-hook.log、k1-manifest.log、k3-judge.log、k4-cost.log、semantics.log（打在仓里被判的那一份上），
# fix-*.log（打在 make_fix_copy.py 建的改法副本上：副本上的数不算入库装置上的数），*.rc 是各自的退出码
# （探针：0 全成立、1 有攻击格打中、2 对照格破了）。
set -uo pipefail
export PYTHONDONTWRITEBYTECODE=1
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/../../.." && pwd)"
mkdir -p "${1:?输出目录}"; OUT="$(cd "$1" && pwd)"; SCRATCH="${2:-/tmp/claude-1000/defs-gatebatch-m2-r2-opus}"
mkdir -p "$SCRATCH"; cd "$REPO" || exit 1
probe() { local name="$1"; shift; nice -n 19 python3 "$@" > "$OUT/$name.log" 2>&1; echo "$?" > "$OUT/$name.rc"; echo "$name rc=$(cat "$OUT/$name.rc")"; }
probe k1-hook "$HERE/probe_k1_hook.py"
probe k1-manifest "$HERE/probe_k1_manifest.py"
probe k3-judge "$HERE/probe_k3_judge.py"
probe k4-cost "$HERE/probe_k4_cost.py"
nice -n 19 python3 research/scripts/admission.py --selftest > "$OUT/selftest-admission.log" 2>&1; echo "$?" > "$OUT/selftest-admission.rc"
nice -n 19 bash research/scripts/run-with-memory-cap.sh 4G bash "$HERE/verify_semantics.sh" "$SCRATCH" > "$OUT/semantics.log" 2>&1; echo "$?" > "$OUT/semantics.rc"
echo "semantics rc=$(cat "$OUT/semantics.rc")"
FIX="$SCRATCH/fixcopy"
rm -rf -- "${FIX:?}"; nice -n 19 rsync -a --exclude target --exclude .git "$REPO/" "$FIX/"
python3 "$HERE/make_fix_copy.py" "$FIX" > "$OUT/fix-build.log" 2>&1
cp "$FIX/admission.py.patch" "$FIX/lib_heavy_tests.py.patch" "$OUT/"
( cd "$FIX" && nice -n 19 python3 research/scripts/admission.py --selftest > "$OUT/fix-selftest-admission.log" 2>&1; echo "$?" > "$OUT/fix-selftest-admission.rc"
  nice -n 19 python3 .claude/hooks/lib_heavy_tests.py --selftest > "$OUT/fix-selftest-lib.log" 2>&1; echo "$?" > "$OUT/fix-selftest-lib.rc"
  nice -n 19 bash .claude/hooks/heavy-test-guard.sh --selftest > "$OUT/fix-selftest-guard.log" 2>&1; echo "$?" > "$OUT/fix-selftest-guard.rc" )
export K1_HOOK="$FIX/.claude/hooks/heavy-test-guard.sh" K1_ADMISSION="$FIX/research/scripts/admission.py"
probe fix-k1-hook "$HERE/probe_k1_hook.py"
probe fix-k1-manifest "$HERE/probe_k1_manifest.py"
for m in research/scripts/admission.py "$FIX/research/scripts/admission.py"; do
  for k in crash-case:layer0-first-stream crash-case:layer0-second-stream crash-case:floor-raise-pushed-by-the-session crash-case:c561-sigma-full; do
    printf '%s %s ' "${m/#$SCRATCH/<草稿>}" "$k"; nice -n 19 python3 "$m" crash-case-manifest . "$k" "$SCRATCH/real-manifest.out" --toolchain --build-environment
  done
done > "$OUT/real-repo-manifests.log" 2>&1
echo "done: $(ls "$OUT" | wc -l) 个文件"
