#!/usr/bin/env bash
# alloc-basis-r2 Opus 攻方腿：在草稿目录里拷 crates 副本、编译探针、跑全部场景，产物写进本目录的 outputs/。
# 副本上的数不是入库装置上的数（.claude/rules/three-way-inference.md「一条腿在副本装置上量出来的数」）。
# 用法：bash research/prompts/alloc-basis-r2-opus-model/run-probe.sh   （从仓根跑；约 25 分钟，大头是两次 legal-search）
# 2026-09-17 04:21 UTC 起仓里的 allocator.rs / mount.rs / recovery.rs 被另一个会话改过，脚本从 crates-at-report/ 换回报告读的那一版；
# 对新版本的三格复跑（newcrates-*.txt）不在这个脚本里，是手跑的，命令与哈希见报告「读的实现」那一段。
set -euo pipefail
repo="$(git rev-parse --show-toplevel)"
model="$repo/research/prompts/alloc-basis-r2-opus-model"
draft="${ALLOC_BASIS_R2_DRAFT:-/tmp/claude-1000/alloc-basis-r2-opus/rerun}"
out="$model/outputs-rerun"
mkdir -p "$draft" "$out"
expected=(
  "6bd429edf5ec6dc8701659169e988479918f2d4f crates/singlefs-core/src/allocator.rs"
  "34246932bbd7a85f4a0be11463f4d461e87cf10c crates/singlefs-core/src/mount.rs"
  "af342120f9060997751d95af8e4deb6eee652211 crates/singlefs-core/src/recovery.rs"
  "6dd2ef9a01801b325d16a517bd5b1c9e57c2288f crates/singlefs-core/src/transaction.rs"
  "6f539f57db4ece2971418799c06cf34eaffdf9c2 crates/singlefs-checker/src/walk.rs"
  "7e4292e7e1c54d678e862a38960b7d85121f4267 crates/singlefs-checker/src/image.rs"
)
for flavour in plain patched; do
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
  if [ "$flavour" = patched ]; then
    python3 "$model/patch-copy.py" "$copy"
  fi
  cp "$model/probe.rs" "$copy/crates/singlefs-harness/src/bin/alloc_basis_r2_probe.rs"
  (cd "$copy" && CARGO_TARGET_DIR="$draft/target-$flavour" nice -n 19 cargo build --release --bin alloc_basis_r2_probe)
done
plain="$draft/target-plain/release/alloc_basis_r2_probe"
patched="$draft/target-patched/release/alloc_basis_r2_probe"
run() { local name="$1"; shift; local binary="$1"; shift; env "$@" nice -n 19 "$binary" "${SCENARIO[@]}" > "$out/$name.txt" 2>&1; tail -1 "$out/$name.txt" | grep -q PROBE-COMPLETE || { echo "✗ $name 没跑完"; echo "→ 看 $out/$name.txt 末尾的 panic"; exit 4; }; }
SCENARIO=(turnover); run s1-turnover "$plain" X=1
SCENARIO=(turnover-remount26); run s1-turnover-remount26 "$plain" X=1
SCENARIO=(crash-between); run s3-crash-between "$plain" X=1
SCENARIO=(window-hit 14 1 11 1 3); run s4-window-hit-k14-e1 "$plain" X=1
SCENARIO=(legal-hit 1 6 8 1 1 3); run s4l-legal-hit-k1-1-k2-6-f8 "$plain" X=1
SCENARIO=(legal-hit 1 6 8 1 1 3); run p-s4l-legal-hit-k1-1-k2-6-f8-checkerF-effective "$patched" SFPROBE_CHECKER_FLOOR=effective
SCENARIO=(turnover); run p-s1-turnover "$patched" X=1
cmp "$out/p-s1-turnover.txt" "$out/s1-turnover.txt" || { echo "✗ 打过补丁的副本在环境变量全不设时与原版不一致"; echo "→ patch-copy.py 的某处替换改了默认行为"; exit 5; }
SCENARIO=(turnover); run p-s1-turnover-T1 "$patched" SFPROBE_T1=1
SCENARIO=(turnover-remount26); run p-s1-turnover-remount26-T1 "$patched" SFPROBE_T1=1
SCENARIO=(turnover); run p-s1-turnover-yiprime-noT1 "$patched" SFPROBE_ARM=yiprime SFPROBE_I52=rewritten
SCENARIO=(crash-between); run p-s3-crash-between-checkerF-effective "$patched" SFPROBE_CHECKER_FLOOR=effective
SCENARIO=(crash-between); run p-s3-crash-between-T1-Fsheng-checkerEff "$patched" SFPROBE_T1=1 SFPROBE_CHECKER_FLOOR=effective
SCENARIO=(crash-between); run p-s3-crash-between-yiprime-T1-Fsheng-checkerEff "$patched" SFPROBE_T1=1 SFPROBE_ARM=yiprime SFPROBE_I52=rewritten SFPROBE_CHECKER_FLOOR=effective
SCENARIO=(window-hit 14 1 11 1 3); run p-s4-window-hit-k14-e1-checkerF-effective "$patched" SFPROBE_CHECKER_FLOOR=effective
for arm in "T0|X=1" "T1|SFPROBE_T1=1" "yiprime-T1|SFPROBE_T1=1 SFPROBE_ARM=yiprime SFPROBE_I52=rewritten"; do
  name="${arm%%|*}"; read -r -a envs <<< "${arm#*|}"
  SCENARIO=(hole 26 52); run "p-s5-hole-$name" "$patched" "${envs[@]}"
done
SCENARIO=(hole 26 52); run p-s5-hole-yiprime-T0 "$patched" SFPROBE_ARM=yiprime SFPROBE_I52=rewritten
for cfg in "T0-legal|X=1" "T0-leak5|SFPROBE_LEAK_TXG=5" "T0-deferplus|SFPROBE_DEFER_PLUS=1" "T1-leak5|SFPROBE_T1=1 SFPROBE_LEAK_TXG=5" "T1-deferplus|SFPROBE_T1=1 SFPROBE_DEFER_PLUS=1" "yiprime-asListed-legal|SFPROBE_T1=1 SFPROBE_ARM=yiprime" "yiprime-I52-legal|SFPROBE_T1=1 SFPROBE_ARM=yiprime SFPROBE_I52=rewritten" "yiprime-I52-leak5|SFPROBE_T1=1 SFPROBE_ARM=yiprime SFPROBE_I52=rewritten SFPROBE_LEAK_TXG=5" "yiprime-I52-deferplus|SFPROBE_T1=1 SFPROBE_ARM=yiprime SFPROBE_I52=rewritten SFPROBE_DEFER_PLUS=1" "yiprime-asListed-deferplus|SFPROBE_T1=1 SFPROBE_ARM=yiprime SFPROBE_DEFER_PLUS=1"; do
  name="${cfg%%|*}"; read -r -a envs <<< "${cfg#*|}"
  SCENARIO=(short 6 5 fsheng); run "p-s6-$name" "$patched" "${envs[@]}"
done
for cfg in "T0-Fjin-legal|X=1" "T0-Fjin-leak5|SFPROBE_LEAK_TXG=5" "T0-Fjin-deferplus|SFPROBE_DEFER_PLUS=1"; do
  name="${cfg%%|*}"; read -r -a envs <<< "${cfg#*|}"
  SCENARIO=(short 6 5 fjin); run "p-s6-$name" "$patched" "${envs[@]}"
done
SCENARIO=(reuse-candidate); run p-s7-reuse-candidate-T0 "$patched" X=1
SCENARIO=(reuse-candidate); run p-s7-reuse-candidate-yiprime "$patched" SFPROBE_T1=1 SFPROBE_ARM=yiprime SFPROBE_I52=rewritten
SCENARIO=(reuse-candidate); run p-s7-reuse-candidate-T1-checkerEff "$patched" SFPROBE_T1=1 SFPROBE_CHECKER_FLOOR=effective
for arm in jiaa_weak jiaa_exact; do
  SCENARIO=(turnover); run "p-jiaa-$arm-turnover" "$patched" SFPROBE_T1=1 SFPROBE_ARM=$arm
  SCENARIO=(turnover); run "p-jiaa-$arm-turnover-noT1" "$patched" SFPROBE_ARM=$arm
  SCENARIO=(hole 26 52); run "p-jiaa-$arm-hole" "$patched" SFPROBE_T1=1 SFPROBE_ARM=$arm
  SCENARIO=(short 6 5 fsheng); run "p-jiaa-$arm-short-legal" "$patched" SFPROBE_T1=1 SFPROBE_ARM=$arm
  SCENARIO=(short 6 5 fsheng); run "p-jiaa-$arm-short-leak5" "$patched" SFPROBE_T1=1 SFPROBE_ARM=$arm SFPROBE_LEAK_TXG=5
  SCENARIO=(short 6 5 fsheng); run "p-jiaa-$arm-short-deferplus" "$patched" SFPROBE_T1=1 SFPROBE_ARM=$arm SFPROBE_DEFER_PLUS=1
done
SCENARIO=(window-search 48 4); run s2-window-search "$plain" X=1
SCENARIO=(legal-search 20 25); run s2l-legal-search "$plain" X=1
SCENARIO=(legal-search 8 16); run p-s2l-legal-search-T1-fsheng-8-16 "$patched" SFPROBE_T1=1 SFPROBE_RAISE=fsheng
(cd "$model" && export PYTHONDONTWRITEBYTECODE=1 && nice -n 19 python3 z3_gates.py --selftest > "$out/z3-selftest.txt" && nice -n 19 python3 z3_gates.py --verbose > "$out/z3-results.txt" && nice -n 19 python3 z3_gates.py --sweep > "$out/z3-sweep.txt")
(cd "$out" && sha256sum ./*.txt > SHA256SUMS)
echo "RUN-PROBE-COMPLETE：产物在 $out，与 outputs/ 逐个 cmp 即核对"
