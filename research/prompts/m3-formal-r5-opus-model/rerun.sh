#!/usr/bin/env bash
# m3-formal-r5 攻方腿原型的复跑：在冻结副本的一份拷贝上加一个 crate（r5probe）、在拷贝的 recovery.rs 里加一个只在副本里的变异开关，
# 编 release，逐件跑（每件 8 线程，串行）。日志写进草稿目录，与模型目录里的同名日志比计数行。
# 用法：bash rerun.sh <冻结副本 tree 目录> <草稿目录> [件名…]
# 件名：scan u3-rb u3-ow u5-um u5-fms u5-s1 u5-s5 u5-rbmut（不给就全跑；u3-ow 约 15 分钟、u5-fms 约 8 分钟）
set -euo pipefail
frozen="$1"; scratch="$2"; shift 2
items=("$@"); [ ${#items[@]} -eq 0 ] && items=(scan u3-rb u5-s1 u5-s5 u5-rbmut u5-um u5-fms u3-ow)
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
W="$repo/scripts/hosts/run-with-memory-cap.sh"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="复跑 m3-formal-r5 攻方腿原型 r5probe"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="复跑 m3-formal-r5 攻方腿原型，峰值表按整条命令记"
mkdir -p "$scratch"
rsync -a --exclude target --exclude .git "$frozen/" "$scratch/tree/"
mkdir -p "$scratch/tree/crates/r5probe/src"
cp "$here/Cargo.toml" "$scratch/tree/crates/r5probe/Cargo.toml"
cp "$here/main.rs" "$scratch/tree/crates/r5probe/src/main.rs"
python3 "$here/apply-mutation-switch.py" "$scratch/tree"
cd "$scratch/tree"
CARGO_TARGET_DIR="$scratch/target" nice -n 19 bash "$W" 3700M bash "$repo/scripts/hosts/capped.sh" 8 cargo build --offline --release -p r5probe
cd "$scratch"
P=./target/release/r5probe
run() { local log="$1"; shift; local rc=0; nice -n 19 bash "$W" 1G "$@" > "$log" 2>&1 || rc=$?; echo "$log exit=$rc"; }
for item in "${items[@]}"; do
  case "$item" in
    scan) run scan1-r2.log $P scan rb rbm rbm3 um
          run scan-fm256.log $P scan fm256-4 fm256-8 fm256-12 fm256-16 fm256-20 fm256-24 fm256-28 fm256-32 fm256-40 fm256-48
          run scan-fmm.log $P scan fmm256-6 fmm256-8 fmm256-10 fmm256-12 fmm256-14 fmm256-16 fmm256-20 fmm256-24 fms256-12 ;;
    u3-rb) run u3s2-rb-clean2.log env R5_STAGE2_ALL=1 $P u3s2 rb 8
           run u3s2-rb-mut2.log env R5_STAGE2_ALL=1 R5_MUTATION=keep_tree_table $P u3s2 rb 8 ;;
    u3-ow) for k in 0 1 2 3 4 5; do from=$((k*200000)); to=$(((k+1)*200000)); [ $to -gt 1048588 ] && to=1048588
             run "u3s2-ow-keep_tree_table-$k.log" env R5_MUTATION=keep_tree_table $P u3s2 ow 8 "$to" "$from"; done
           run u3s2-ow-clean-5.log $P u3s2 ow 8 1048588 1000000 ;;
    u5-um) run u5-um-r2.log $P u5 um 8 ;;
    u5-fms) run u5-fms256-12-r2-0.log $P u5 fms256-12 8 11000 0
            run u5-fms256-12-r2-1.log $P u5 fms256-12 8 22000 11000
            run u5-fms256-12-r2-2.log $P u5 fms256-12 8 32840 22000 ;;
    u5-s1) run u5-s1-r2.log $P u5 s1 8 ;;
    u5-s5) run u5-s5-r2.log $P u5 s5 8 ;;
    u5-rbmut) run u5-rb-mut.log env R5_MUTATION=keep_tree_table $P u5 rb 8 ;;
    *) echo "未知件 $item"; exit 2 ;;
  esac
done
