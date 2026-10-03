#!/bin/bash
# m3-root-ring-code-r2 攻方腿复跑：rebuild-and-run.sh <草稿目录> <批…>。冻结副本拷进 <草稿目录>/tree、放进原型 crate、编 release（build.sh）；
# 改法 A 的副本在 <草稿目录>/fixA/tree（冻结副本加 fixA.diff）。批：e2 | e2-sweep | fh | sir | avc | fz（三批随机历史，各 10 件）。
set -euo pipefail
scratch=$1; shift
model=$(cd "$(dirname "$0")/.." && pwd)
bash "$model/scripts/build.sh" "$scratch"
if [ ! -d "$scratch/fixA/tree" ]; then
  mkdir -p "$scratch/fixA" && rsync -a --exclude target "$scratch/tree/" "$scratch/fixA/tree/"
  (cd "$scratch/fixA/tree" && patch -p0 -s crates/singlefs-core/src/mount.rs < "$model/fixA.diff")
fi
bash "$model/scripts/build.sh" "$scratch/fixA"
run() { bash "$model/scripts/run.sh" "$@"; }
for what in "$@"; do
  case "$what" in
    e2) for m in flip clean none; do run "$scratch" "e2-s4-p20-b2-$m" e2_reread_flip 4 20 2 "$m"; done
        FLIP_ONLY_AT_ANCHOR=1 run "$scratch" e2-s4-p20-b2-flip-once e2_reread_flip 4 20 2 flip
        for m in flip clean; do ROLLBACK_AT_ANCHOR=1 FLIP_ONLY_AT_ANCHOR=1 run "$scratch" "e2-rb-$m" e2_reread_flip 4 20 2 "$m"; done ;;
    e2-sweep) for s in 4 6; do p=$((3*s+8)); for b in 1 2 4; do for c in 3000 20000 60000; do for m in flip clean; do
        PROTO_CONTENT_BYTES=$c FLIP_ONLY_AT_ANCHOR=1 run "$scratch" "sw-s$s-b$b-c$c-$m" e2_reread_flip "$s" "$p" "$b" "$m"
        [ "$c" = 60000 ] || PROTO_CONTENT_BYTES=$c FLIP_ONLY_AT_ANCHOR=1 run "$scratch/fixA" "fa-s$s-b$b-c$c-$m" e2_reread_flip "$s" "$p" "$b" "$m"
        done; done; done; done ;;
    fh) for t in "$scratch" "$scratch/fixA"; do for h in 9 10; do for m in mount none; do run "$t" "fh-s4-h$h-$m" front_hole_eio 4 "$h" "$m"; done; done; done ;;
    sir) for c in row zero; do run "$scratch" "sir-$c" skipped_instance_rows 4 8 "$c"; done ;;
    avc) run "$scratch" avc-mount applied_version_carrier 4 mount ;;
    fz) bash "$model/scripts/fuzz-batch.sh" "$scratch" 1 4 100 10 30
        FUZZ_P_ACTS=35:45:55 bash "$model/scripts/fuzz-batch.sh" "$scratch" 2 4 1000 10 25
        bash "$model/scripts/fuzz-batch.sh" "$scratch" 3 6 2000 10 20
        FUZZ_P_ACTS=35:45:55 bash "$model/scripts/fuzz-batch.sh" "$scratch" 4 4 1000 10 25
        FUZZ_ANY_TARGET=1 FUZZ_P_ACTS=35:45:55 bash "$model/scripts/fuzz-batch.sh" "$scratch" 5 4 5000 10 25 ;;
    *) echo "不认识的批：$what" >&2; exit 2 ;;
  esac
done
