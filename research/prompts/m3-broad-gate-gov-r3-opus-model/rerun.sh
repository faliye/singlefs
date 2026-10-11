#!/usr/bin/env bash
# 攻方腿 m3-broad-gate-gov-r3 的复跑（只有轻的：门禁样本照「cd 进样本根、根参数写同一个目录」起、载入驱动的纯函数与 ensure_baseline、
# 真包装在自己的临时 slice（总上限 ≤ 256M）里起 python 与 sleep；不起驱动真跑、不起整道门禁、不跑拷贝上的自证——那几样在 main-agent-heavy.sh）。
#   bash rerun.sh <冻结副本树根> <草稿目录> [段…]     段：samples j2a j2b j1c j1d j1a j2c j3 fix（不给就全跑）；产出写进本目录 outputs/（同名覆盖）
set -uo pipefail
frozen="$1"; work="$2"; shift 2; here="$(cd "$(dirname "$0")" && pwd)"; out="$here/outputs"
export R3_ATTACK_SCRATCH="$work"
mkdir -p "$work/trees" "$out"; base="$work/trees/base"
if [[ ! -d "$base" ]]; then rsync -a --exclude target --exclude .git "$frozen/" "$base/"; fi
run_samples() { bash "$here/run-samples.sh" "$@" | tail -1; }
seg_samples() { run_samples "$base" "$work/out/base" > "$out/samples-base.txt"; }
seg_j2a() {
  : > "$out/j2a-hooks.txt"
  for h in baseline-rerun baseline-rerun-then-reescalate peak-overwrite; do
    { echo "== $h"; HOOK="$here/hook-$h.sh" run_samples "$base" "$work/out/j2a-$h" gate-verdict-escalated-green
      grep -E '登记表第|说第' "$work/out/j2a-$h/gate-verdict-escalated-green.txt" | sed "s|$work/[^ ]*||" ; } >> "$out/j2a-hooks.txt"
  done
  mkdir -p "$work/probe"; python3 "$here/j2a-baseline-probe.py" "$base/scripts/mutation/broad-mutation.py" "$work/probe" 2>&1 | grep -v '^  ·' > "$out/j2a-baseline-probe.txt"
}
seg_j2b() {
  rm -rf "${work:?}/trees/j2b"; mkdir -p "$work/trees/j2b"
  python3 "$here/j2b-gate-mutants.py" "$base" "$work/trees/j2b" > "$out/j2b-mutants.list" || return 1
  : > "$out/j2b-sweep.txt"
  for m in $(cut -f1 "$out/j2b-mutants.list"); do printf '%s\t' "$m" >> "$out/j2b-sweep.txt"; run_samples "$work/trees/j2b/$m" "$work/out/j2b-$m" >> "$out/j2b-sweep.txt"; done
  : > "$out/j2b-inputs.txt"
  for pair in i03-baseline-lie:g03-baseline-differs i05-not-larger:g05-not-larger i06m-memory-multiple:g06m-memory-multiple i07-no-trailer:g07-trailer \
              i07b-trailer-wrong:g07b-trailer-prefix i08-no-wrapper:g08-wrapper i09-columns:g09-columns i10-no-escalated-field:g10-escalated-field \
              i11-name-differs:g11-sequence-only i14-logs-elsewhere:g14-anywhere; do
    c=${pair%%:*}; g=${pair##*:}
    CASE=$c HOOK="$here/hook-j2b-case.sh" run_samples "$base" "$work/out/j2b-in-$c-base" gate-verdict-escalated-green >/dev/null
    CASE=$c HOOK="$here/hook-j2b-case.sh" run_samples "$work/trees/j2b/$g" "$work/out/j2b-in-$c-$g" gate-verdict-escalated-green >/dev/null
    b=$(sed -n 's/^exit=//p' "$work/out/j2b-in-$c-base/gate-verdict-escalated-green.txt"); m=$(sed -n 's/^exit=//p' "$work/out/j2b-in-$c-$g/gate-verdict-escalated-green.txt")
    why=$(grep -m1 -E '登记表第|明细|ESCALATED|说第' "$work/out/j2b-in-$c-base/gate-verdict-escalated-green.txt" | sed 's/^ *//' | cut -c1-160)
    printf '%s\t原样门禁 exit=%s\t%s exit=%s\t%s\n' "$c" "$b" "$g" "$m" "$why" >> "$out/j2b-inputs.txt"
  done
}
seg_j1c() {
  rm -rf "${work:?}/trees/j1c"; mkdir -p "$work/trees/j1c"
  python3 "$here/j1c-admission-mutants.py" "$base" "$work/trees/j1c" > "$out/j1c-mutants.list" || return 1
  : > "$out/j1c-sweep.txt"
  for m in $(cut -f1 "$out/j1c-mutants.list"); do printf '%s\t' "$m" >> "$out/j1c-sweep.txt"; run_samples "$work/trees/j1c/$m" "$work/out/j1c-$m" >> "$out/j1c-sweep.txt"; done
  rsync -a "$base/" "$work/trees/j1c/a0-original/"
  python3 "$here/j1c-env-gap.py" "$work/trees/j1c" "$work" a0-original $(cut -f1 "$out/j1c-mutants.list") > "$out/j1c-env-gap.txt" 2>&1
}
seg_j1d() { python3 "$here/j1d-rustup-toolchain.py" "$base/scripts/mutation/broad-mutation.py" "$base" nightly-x86_64-unknown-linux-gnu > "$out/j1d-rustup-toolchain.txt" 2>&1; }
seg_j1a() { { command -v cargo; PATH=/usr/bin:/bin run_samples "$base" "$work/out/j1a-nocargo" library-version-is-staged-tree-marker-check-clean-unmarked
             grep -E '✗|→|^exit=' "$work/out/j1a-nocargo/library-version-is-staged-tree-marker-check-clean-unmarked.txt"; } > "$out/j1a-marker-check-no-cargo.txt" 2>&1; }
seg_j2c() {
  bash "$here/j2c-wrapper-probe.sh" "$base" "$work" > "$out/j2c-wrapper-probe.txt" 2>&1
  bash "$here/j2c-over-slice.sh" "$base" "$work" > "$out/j2c-over-slice.txt" 2>&1
  bash "$here/j2c-collateral.sh" "$base" "$work" > "$out/j2c-collateral.txt" 2>&1
  bash "$here/j2c-pagecache.sh" "$base" "$work" > "$out/j2c-pagecache.txt" 2>&1
}
seg_j3() {
  mkdir -p "$work/j3"; awk 'NR>=399 && NR<=624' "$base/.claude/gate.d/checker-tier-broad-mutation.sh" > "$work/j3/gate_verdict.py"
  python3 "$here/j3-combos.py" "$base/scripts/mutation/broad-mutation.py" "$work/j3/gate_verdict.py" "$work/j3" 400 20261003 > "$out/j3-combos-400.txt" 2>&1
  python3 "$here/j3-combos.py" "$base/scripts/mutation/broad-mutation.py" "$work/j3/gate_verdict.py" "$work/j3" 5000 7 > "$out/j3-combos-5000.txt" 2>&1
  : > "$out/j3-switches.txt"
  for b in $(awk 'NR>=83 && NR<=111 && $1=="#" && $2 ~ /^[a-z][a-z-]+$/ {print $2}' "$base/.claude/gate.d/checker-tier-broad-mutation.sh"); do
    printf '%s\t' "$b" >> "$out/j3-switches.txt"; CHECKER_TIER_BROAD_MUTATION_BREAK=$b run_samples "$base" "$work/out/sw-$b" >> "$out/j3-switches.txt"
  done
}
seg_fix() {
  rm -rf "${work:?}/trees/fix"; mkdir -p "$work/trees/fix"; python3 "$here/j2a-fix-prototypes.py" "$base" "$work/trees/fix" >/dev/null || return 1
  { for f in F1 F2 F12; do echo "== $f 36 份样本"; run_samples "$work/trees/fix/$f" "$work/out/fix-$f"
      for h in baseline-rerun baseline-rerun-then-reescalate peak-overwrite; do printf '   %s：' "$h"
        HOOK="$here/hook-$h.sh" run_samples "$work/trees/fix/$f" "$work/out/fix-$f-$h" gate-verdict-escalated-green >/dev/null
        grep -m1 -E '^exit=' "$work/out/fix-$f-$h/gate-verdict-escalated-green.txt"; done; done; } > "$out/j2a-fix-prototypes.txt" 2>&1
}
for segment in ${@:-samples j2a j2b j1c j1d j1a j2c j3 fix}; do "seg_$segment" || echo "段 $segment 没跑成"; done
echo "done ${*:-all}"
