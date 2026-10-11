#!/usr/bin/env bash
# m3-formal-r6 攻方腿原型 r6probe 的复跑：冻结副本拷一份、打两份副本补丁（c651_patch_copy.py：C651 候选开关与 W1 开关；
# r6_patch_copy.py：回退返回早于持久的变异开关、把 r6probe 加进工作区）、放进原型、经内存包装编 release，再逐件跑。
# 每次调用不超过 20 万态；件里按序号段切成几次，每次一份日志。
# 用法：bash rerun.sh <冻结副本 tree 目录> <草稿目录> [件名…]
# 件名：sizes lo-rbm lo-rbm-mut tgt-fft tgt-rbm w1-off-A w1-on-A（不给就全跑）
set -euo pipefail
frozen="$1"; scratch="$2"; shift 2
items=("$@"); [ ${#items[@]} -eq 0 ] && items=(sizes lo-rbm lo-rbm-mut tgt-fft tgt-rbm w1-off-A w1-on-A)
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
W="$repo/scripts/hosts/run-with-memory-cap.sh"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="复跑 m3-formal-r6 攻方腿原型 r6probe"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="复跑 m3-formal-r6 攻方腿原型，峰值表按整条命令记"
mkdir -p "$scratch/runs"
rsync -a --exclude target --exclude .git "$frozen/" "$scratch/tree/"
cp -r "$repo/.cargo" "$scratch/tree/"
python3 "$here/c651_patch_copy.py" "$scratch/tree"
python3 "$here/r6_patch_copy.py" "$scratch/tree"
mkdir -p "$scratch/tree/crates/r6probe/src"
cp "$here/Cargo.toml" "$scratch/tree/crates/r6probe/Cargo.toml"
cp "$here/main.rs" "$scratch/tree/crates/r6probe/src/main.rs"
cd "$scratch/tree"
CARGO_TARGET_DIR="$scratch/target" nice -n 19 bash "$W" 3G bash "$repo/scripts/hosts/capped.sh" 5 cargo build --offline --release -p r6probe
cd "$scratch"
P=./target/release/r6probe
run() { local log="$1"; shift; local rc=0; nice -n 19 bash "$W" 2G "$@" > "runs/$log" 2>&1 || rc=$?; echo "$log exit=$rc"; }
A='W:x=65536:y=512:u=5:reuse=disk0:fsid=same'
for item in "${items[@]}"; do
  case "$item" in
    sizes) for h in rbm fft; do run "sizes-$h.log" $P sizes $h; done
           run sizes-rbm-mut.log env R6_MUTATION=rollback_not_durable $P sizes rbm ;;
    lo-rbm) run lo-rbm-clean-0.log $P lo rbm 5 12309 0
            run lo-rbm-clean-1.log $P lo rbm 5 24618 12309 ;;
    lo-rbm-mut) run lo-rbm-mut-0.log env R6_MUTATION=rollback_not_durable $P lo rbm 5 12338 0
                run lo-rbm-mut-1.log env R6_MUTATION=rollback_not_durable $P lo rbm 5 24677 12338 ;;
    tgt-fft) run tgt-fft.log $P tgt fft 5 ;;
    tgt-rbm) run tgt-rbm-0.log $P tgt rbm 5 8000 0
             run tgt-rbm-1.log $P tgt rbm 5 16000 8000
             run tgt-rbm-2.log $P tgt rbm 5 20300 16000
             run tgt-rbm-3.log $P tgt rbm 5 24618 20300 ;;
    w1-off-A) run w1-off-A-0.log env C651_CANDIDATE=search $P w1 "$A" 5 3500 0
              run w1-off-A-1.log env C651_CANDIDATE=search $P w1 "$A" 5 13850 3500 ;;
    w1-on-A) for k in 0 1 2 3 4 5 6 7 8; do from=$((k*10000)); to=$(((k+1)*10000)); [ $to -gt 82970 ] && to=82970
               run "w1-on-A-full-$k.log" env C651_CANDIDATE=search C651_MKFS_CLEARS=1 $P w1 "$A" 5 "$to" "$from"; done ;;
    *) echo "未知件 $item"; exit 2 ;;
  esac
done
