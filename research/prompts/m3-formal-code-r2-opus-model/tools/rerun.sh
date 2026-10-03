#!/usr/bin/env bash
# m3-formal-code-r2 攻方腿原型的复跑。用法：bash rerun.sh <冻结副本 tree 目录> <草稿目录> [件名…]
# 件名：k3（k3slot、k3slot0、k3mkfs，几秒）、k2first（11 个窗口的第一次崩溃全量，合计约 25 分钟，按窗口一件）、
#       k2second（mkmnt 全部、mnt 每段第一个、fms12 序号 0 的二次崩溃全量，约 15 分钟）、k4（两个变异各跑 harness 档 108 个测试二进制，约 15 分钟）
# 每条命令经 run-with-memory-cap.sh、capped.sh 6；输出写 <草稿目录>/runs/，与模型目录 outputs/ 同名的比计数行（去掉 elapsed / 包装行 / e.g. 行）。
set -u
frozen="$1"; D="$2"; shift 2
items=("$@"); [ ${#items[@]} -eq 0 ] && items=(k3 k2first k2second k4)
here="$(cd "$(dirname "$0")/.." && pwd)"
repo="$(cd "$here/../../.." && pwd)"
W="$repo/research/scripts/run-with-memory-cap.sh"; C="$repo/research/scripts/capped.sh"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="复跑 m3-formal-code-r2 攻方原型，逐段历史的峰值不同"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="复跑 m3-formal-code-r2 攻方原型 r2probe"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS=600
mkdir -p "$D/runs"
rsync -a --exclude target --exclude .git "$frozen/" "$D/tree/"
mkdir -p "$D/tree/crates/r2probe/src"
cp "$here/prototypes/r2probe/Cargo.toml" "$D/tree/crates/r2probe/Cargo.toml"
cp "$here/prototypes/r2probe/src/main.rs" "$D/tree/crates/r2probe/src/main.rs"
grep -q '"crates/r2probe"' "$D/tree/Cargo.toml" || sed -i 's#    "crates/singlefs-reference-filesystem",#    "crates/singlefs-reference-filesystem",\n    "crates/r2probe",#' "$D/tree/Cargo.toml"
(cd "$D/tree" && CARGO_TARGET_DIR="$D/target" nice -n 19 bash "$W" 3G bash "$C" 6 cargo build --offline --release -p r2probe) > "$D/build.log" 2>&1 || { echo "build failed"; exit 1; }
P="$D/target/release/r2probe"
run() { local log="$1"; shift; nice -n 19 bash "$W" 3G bash "$C" 6 "$P" "$@" > "$D/runs/$log" 2>&1; echo "$log rc=$?"; }
for item in "${items[@]}"; do
  case "$item" in
    k3) run k3slot.log k3slot; run k3slot0.log k3slot0; run k3mkfs.log k3mkfs ;;
    k2first) for h in mnt rb rbm cold puw rfx3-1 rfx4-2 fm256m12 um; do run "k2-$h-first.log" k2 "$h" 6; done
             run k2-fms12-first.log k2 fms12 6 0 14000 none; run k2-fms12-first-b.log k2 fms12 6 14000 28733 none ;;
    k2second) run k2-mkmnt.log k2 mkmnt 6 0 33 all
              R2_SECOND_CAP=25000 R2_FIRST_PER_SEGMENT=1 run k2-mnt-second-seg1.log k2 mnt 6 0 20511 all
              run k2one-fms12-0-a.log k2one fms12 0 6 0 14000; run k2one-fms12-0-b2.log k2one fms12 0 6 14000 28733 ;;
    k4) rsync -a --exclude target --exclude .git "$frozen/" "$D/tree-mut/"; cp -r "$repo/litmus" "$D/tree-mut/litmus"
        for m in core checker; do
          f="$D/tree-mut/crates/singlefs-core/src/recovery.rs"; [ "$m" = checker ] && f="$D/tree-mut/crates/singlefs-checker/src/image.rs"
          cp "$frozen/crates/singlefs-core/src/recovery.rs" "$D/tree-mut/crates/singlefs-core/src/recovery.rs"
          cp "$frozen/crates/singlefs-checker/src/image.rs" "$D/tree-mut/crates/singlefs-checker/src/image.rs"
          patch "$f" < "$here/probes/mut-$m-borrow.patch"
          for list in k4-targets k4-rest; do
            args=(); while read -r t; do args+=(--test "$t"); done < "$here/outputs/$list.txt"
            (cd "$D/tree-mut" && CARGO_TARGET_DIR="$D/target-mut" nice -n 19 bash "$W" 3G bash "$C" 6 cargo test --offline -p singlefs-harness --no-fail-fast "${args[@]}") > "$D/runs/k4-$m-$list.log" 2>&1
            grep -E '^test result' "$D/runs/k4-$m-$list.log" | awk -v n="k4-$m-$list" '{p+=$4; f+=$6; i+=$8} END {print n" binaries="NR" passed="p" failed="f" ignored="i}'
          done
        done ;;
  esac
done
