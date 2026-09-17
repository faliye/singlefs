#!/usr/bin/env bash
# alloc-basis-r3 Opus 攻方腿：在草稿目录里拷 crates 副本、打补丁、编译探针、跑全部场景。
# 副本上的数不是入库装置上的数（.claude/rules/three-way-inference.md「一条腿在副本装置上量出来的数」）。
# 用法（从仓根跑）：bash research/prompts/alloc-basis-r3-opus-model/run-probe.sh [产物目录名，默认 outputs-rerun]
#   产物写进本目录下的那个目录；是 outputs-rerun 时跑完与 outputs/ 逐个 cmp。
set -euo pipefail
repo="$(git rev-parse --show-toplevel)"
model="$repo/research/prompts/alloc-basis-r3-opus-model"
draft="${ALLOC_BASIS_R3_DRAFT:-/tmp/claude-1000/alloc-basis-r3-opus/rerun}"
out_name="${1:-outputs-rerun}"
out="$model/$out_name"
rm -rf "${out:?}"
mkdir -p "$draft" "$out/g8"
expected=(
  "66c9eb1dc842a5147b700c33b5b18a1deba2a964 crates/singlefs-core/src/allocator.rs"
  "b228b7f4a5ebe159911fbf451eb4508b4962d60e crates/singlefs-core/src/mount.rs"
  "6dd2ef9a01801b325d16a517bd5b1c9e57c2288f crates/singlefs-core/src/transaction.rs"
  "ef4da6108cf23366a69d9916160b36a9f52bf546 crates/singlefs-core/src/recovery.rs"
  "1caa43f0b33d93d990b378cf46b5f3d479b86651 crates/singlefs-checker/src/walk.rs"
  "bea51e1665fc51674f58b26b97bd7b98c43ffcf8 crates/singlefs-checker/src/image.rs"
)
for flavour in vis patched; do
  copy="$draft/copy-$flavour"
  rm -rf "${copy:?}"
  mkdir -p "$copy"
  rsync -a --exclude target --exclude .git "$repo/Cargo.toml" "$repo/Cargo.lock" "$repo/crates" "$copy/"
  for line in "${expected[@]}"; do
    file="${line#* }"
    got="$(git hash-object --no-filters "$copy/$file")"
    stored="$model/crates-at-report/$(basename "$file")"
    if [ "$got $file" != "$line" ] && [ -f "$stored" ] && [ "$(git hash-object --no-filters "$stored") $file" = "$line" ]; then
      echo "· $file 在仓里已被改过（$got），换回 crates-at-report/ 里报告读的那一版"
      cp "$stored" "$copy/$file"
      got="$(git hash-object --no-filters "$copy/$file")"
    fi
    if [ "$got $file" != "$line" ]; then
      echo "✗ $file 的哈希是 $got，不是报告读的那一版"
      echo "→ crates 在报告之后改过、crates-at-report/ 里也没有那一版：按新版本重判，或把那一版放进 crates-at-report/"
      exit 3
    fi
  done
  if [ "$flavour" = vis ]; then
    python3 "$model/patch-copy.py" "$copy" --visibility-only
  else
    python3 "$model/patch-copy.py" "$copy"
  fi
  cp "$model/probe.rs" "$copy/crates/singlefs-harness/src/bin/alloc_basis_r3_probe.rs"
  (cd "$copy" && CARGO_TARGET_DIR="$draft/target-$flavour" nice -n 19 cargo build --release --bin alloc_basis_r3_probe)
done
vis="$draft/target-vis/release/alloc_basis_r3_probe"
patched="$draft/target-patched/release/alloc_basis_r3_probe"
run() {
  local name="$1"; shift; local binary="$1"; shift
  env "$@" nice -n 19 "$binary" "${SCENARIO[@]}" > "$out/$name.txt" 2>&1
  tail -1 "$out/$name.txt" | grep -q PROBE-COMPLETE || { echo "✗ $name 没跑完"; echo "→ 看 $out/$name.txt 末尾的 panic"; exit 4; }
}
# 自证：补丁在环境变量全不设时不改行为（与只改可见性的那份逐字节相同）。
for spec in "milestone-turnover 30 28" "rollback-turnover 5 33" "z2-crash-between fkou" "two-rollbacks 1 4 40" "rollback-after-raise 30 46"; do
  read -r -a SCENARIO <<< "$spec"
  name="selfcheck-$(echo "$spec" | tr ' ' '_')"
  run "$name-vis" "$vis" X=1
  run "$name-patched" "$patched" X=1
  cmp "$out/$name-vis.txt" "$out/$name-patched.txt" || { echo "✗ 打过补丁的副本在环境变量全不设时与对照副本不一致：$spec"; echo "→ patch-copy.py 的某处替换改了默认行为"; exit 5; }
done
# Z1′ / Z4′：回退之后接着转环（四种分配器形态）与它的后果。
SCENARIO=(rollback-turnover 5 33); run h1-T0 "$patched" X=1
SCENARIO=(rollback-turnover 5 33); run h1-T1 "$patched" SFPROBE_T1=1
SCENARIO=(rollback-turnover 5 33); run h1-T1-G5turnover "$patched" SFPROBE_T1=1 SFPROBE_G5_TURNOVER=1
SCENARIO=(rollback-turnover 5 33); run h1-T1-conservative "$patched" SFPROBE_T1=1 SFPROBE_CONSERVATIVE=1
SCENARIO=(milestone-turnover 30 28); run h1m-T1-crash28 "$patched" SFPROBE_T1=1
SCENARIO=(milestone-turnover 30 28); run h1m-T0-crash28 "$patched" X=1
SCENARIO=(rollback-turnover-fallback 5 29); run h1f-T0 "$patched" X=1
SCENARIO=(rollback-turnover-fallback 5 29); run h1f-T1 "$patched" SFPROBE_T1=1
SCENARIO=(rollback-turnover-fallback 5 29); run h1f-T1-G5turnover "$patched" SFPROBE_T1=1 SFPROBE_G5_TURNOVER=1
# Z4′：两次回退（更早 / 更晚）、回退之前抬过 F。
for arm in "T0|X=1" "T1|SFPROBE_T1=1" "T1-G5turnover|SFPROBE_T1=1 SFPROBE_G5_TURNOVER=1"; do
  name="${arm%%|*}"; read -r -a envs <<< "${arm#*|}"
  SCENARIO=(two-rollbacks 1 4 40); run "h4-earlier-$name" "$patched" "${envs[@]}"
  SCENARIO=(two-rollbacks 2 11 40); run "h4-later-$name" "$patched" "${envs[@]}"
  SCENARIO=(rollback-after-raise 30 46); run "h4c-$name" "$patched" "${envs[@]}"
done
# Z2′：两个 F 的四组、坏镜像「扣住位被撤掉」、推进一格重发、X8-A。
SCENARIO=(z2-crash-between fkou); run z2-Fkou "$patched" X=1
SCENARIO=(z2-crash-between fkou); run z2-Fkou-prime "$patched" SFPROBE_CHECKER_FLOOR=effective
SCENARIO=(z2-crash-between g7); run z2-G7 "$patched" SFPROBE_T1=1 SFPROBE_CHECKER_FLOOR=effective
SCENARIO=(z2-crash-between g7); run z2-G7-prime "$patched" SFPROBE_T1=1
SCENARIO=(z2-crash-between fkou-nohold); run z2-badimage-hold-removed-checkerNewestF "$patched" X=1
SCENARIO=(z2-crash-between fkou-nohold); run z2-badimage-hold-removed-checkerEffective "$patched" SFPROBE_CHECKER_FLOOR=effective
SCENARIO=(z2-advance-one fkou cap); run z2adv-Fkou-cap "$patched" X=1
SCENARIO=(z2-advance-one fkou cap); run z2adv-Fkou-cap-checkerEffective "$patched" SFPROBE_CHECKER_FLOOR=effective
SCENARIO=(z2-advance-one fkou nocap); run z2adv-Fkou-nocap "$patched" X=1
SCENARIO=(z2-advance-one g7 cap); run z2adv-G7-cap "$patched" SFPROBE_T1=1 SFPROBE_CHECKER_FLOOR=effective
SCENARIO=(z2-advance-one g7 nocap); run z2adv-G7-nocap "$patched" SFPROBE_T1=1 SFPROBE_CHECKER_FLOOR=effective
SCENARIO=(x8a); run x8a "$patched" X=1
# Z1′：推进一格重发造洞（甲-T1 记账 + 甲-b′ 的读数）、多根共享单元。
SCENARIO=(hole-advance-one 26 38 56); run h5-hole-advance-one-T1 "$patched" SFPROBE_T1=1
SCENARIO=(shared-units 2 3 30 70); run h7-shared-2-3-T1-G7-G8 "$patched" SFPROBE_T1=1 SFPROBE_CHECKER_FLOOR=effective SFPROBE_G8=1
SCENARIO=(shared-units 1 1 30 70); run h7-shared-1-1-T1-G7-G8 "$patched" SFPROBE_T1=1 SFPROBE_CHECKER_FLOOR=effective SFPROBE_G8=1
# Z5′：活单元记成已释放的坏镜像与对照；G8 / G8′ 在合法历史上的判定。
SCENARIO=(z5 31); run h6-z5-bug-at5-T1-G8 "$patched" SFPROBE_T1=1 SFPROBE_G8=1 SFPROBE_BUG_RELEASE_CARRIED_INSTANCE_TABLE_AT=5
SCENARIO=(z5 31); run h6-z5-control-T1-G8 "$patched" SFPROBE_T1=1 SFPROBE_G8=1
SCENARIO=(rollback-turnover 5 40); run g8/h1 "$patched" SFPROBE_G8=1 SFPROBE_T1=1 SFPROBE_G5_TURNOVER=1 SFPROBE_CHECKER_FLOOR=effective
SCENARIO=(two-rollbacks 1 4 40); run g8/h4-earlier "$patched" SFPROBE_G8=1 SFPROBE_T1=1 SFPROBE_G5_TURNOVER=1 SFPROBE_CHECKER_FLOOR=effective
SCENARIO=(two-rollbacks 2 11 40); run g8/h4-later "$patched" SFPROBE_G8=1 SFPROBE_T1=1 SFPROBE_G5_TURNOVER=1 SFPROBE_CHECKER_FLOOR=effective
SCENARIO=(rollback-after-raise 30 46); run g8/h4c "$patched" SFPROBE_G8=1 SFPROBE_T1=1 SFPROBE_G5_TURNOVER=1 SFPROBE_CHECKER_FLOOR=effective
SCENARIO=(z2-crash-between g7); run g8/z2-g7 "$patched" SFPROBE_G8=1 SFPROBE_T1=1 SFPROBE_CHECKER_FLOOR=effective
SCENARIO=(hole-advance-one 26 38 56); run g8/h5 "$patched" SFPROBE_G8=1 SFPROBE_T1=1
SCENARIO=(z2-crash-between fkou); run g8/z2-fkou-writer-checkerNewestF "$patched" SFPROBE_G8=1
SCENARIO=(milestone-turnover 30); run g8/h1m-T0 "$patched" SFPROBE_G8=1
# 表：两种历史逐次发布（甲-T1 + G5，主 agent 倾向的组合）。
for name in h4-earlier-T1 h4-later-T1 h4c-T1 h1-T1; do
  python3 "$model/extract-tables.py" "$out/$name.txt" > "$out/table-$name.md"
done
(cd "$out" && find . -name '*.txt' -o -name '*.md' | LC_ALL=C sort | xargs sha256sum > SHA256SUMS)
if [ "$out_name" = outputs-rerun ] && [ -d "$model/outputs" ]; then
  (cd "$out" && find . -name '*.txt' -o -name '*.md' | LC_ALL=C sort) | while read -r produced; do
    cmp "$out/$produced" "$model/outputs/$produced" || { echo "✗ 复跑的 $produced 与 outputs/ 里的不同"; echo "→ 看 diff，确认是不是 crates 或探针改过"; exit 6; }
  done
  echo "复跑与 outputs/ 逐个相同"
fi
echo "RUN-PROBE-COMPLETE：产物在 $out"
