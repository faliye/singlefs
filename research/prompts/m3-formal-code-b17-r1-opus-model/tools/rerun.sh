#!/usr/bin/env bash
# 复跑攻方腿（m3-formal-code-b17-r1，opus）的全部量：冻结树拷进 <工作目录>/repo、放进原型，经内存包装编、跑；
# 形 A（挂 k 次后第一个文件）、形 B（实例 2 里推 z 次零单元发布后第一个文件）、形 C（瞬时读坏掩码）、形 D（只数崩溃状态）、形 E（长度）；
# 打 walk-fix-arms.patch 后两条改法臂 f1 / f2 各跑形 A、形 B、现有两个 I-5.5 用例；A2c 变异下三条臂各跑形 A；还原后逐行证 rows.tsv 的 8 行变异。
# 用法：bash tools/rerun.sh <工作目录（不存在）> [线程数，默认 8]；总时长约 25 分钟（攻方副本上量的单次时长相加）。
# 跑完看 <工作目录>/out/summary.txt：每行一格，与报告「各格判定一览」里写的计数逐项对。
set -euo pipefail
WORK="$1"
THREADS="${2:-8}"
REPO_ROOT=/home/fy5090/code/singlefs
FROZEN=/tmp/claude-1000/m3-formal-code-b17-r1/tree
MODEL="$(cd "$(dirname "$0")/.." && pwd)"
CAP="$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh"
CAPPED="$REPO_ROOT/scripts/hosts/capped.sh"
if [ -e "$WORK" ]; then echo "工作目录已存在：$WORK" >&2; exit 2; fi
mkdir -p "$WORK/repo" "$WORK/out"
cp "$REPO_ROOT/Cargo.toml" "$REPO_ROOT/Cargo.lock" "$WORK/repo/"
rsync -a --exclude target --exclude .git "$FROZEN/crates" "$WORK/repo/"
(cd "$WORK/repo" && sha256sum -c --quiet "$REPO_ROOT/research/prompts/m3-formal-code-b17-r1-snapshot/crates-sha256.txt")
cp "$MODEL/opus_b17_first_data_unit_attack.rs" "$WORK/repo/crates/singlefs-harness/tests/"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="攻方原型扫几十格、镜像拷贝更多，峰值表的数不是这一份副本量的"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="三方轮复跑攻方腿的原型，与别的活各跑各的"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS=600
proto() { # proto <输出名> <过滤词>，环境变量由调用方前缀给
  (cd "$WORK/repo" && nice -n 19 bash "$CAP" 2G bash "$CAPPED" "$THREADS" cargo test --offline -p singlefs-harness \
    --test opus_b17_first_data_unit_attack -- "$2" --nocapture > "$WORK/out/$1.log" 2>&1)
}
binary() { # binary <输出名> <cargo test 的 --test 之后的参数…>
  local out="$1"; shift
  (cd "$WORK/repo" && nice -n 19 bash "$CAP" 2G bash "$CAPPED" "$THREADS" cargo test --offline -p singlefs-harness --test "$@" \
    > "$WORK/out/$out.log" 2>&1) || true
}
count() { # count <输出名> <name= 前缀>
  local f="$WORK/out/$1.log"
  echo "$1 cells=$(grep -c "^name=$2 " "$f") holds=$(grep "^name=$2 " "$f" | grep -c 'i55=holds') na=$(grep "^name=$2 " "$f" | grep -c 'i55=not_applicable') violated=$(grep "^name=$2 " "$f" | grep -c 'i55=VIOLATED') other_red=$(grep "^name=$2 " "$f" | grep 'red=\[' | grep -v 'red=\[\]' | grep -vc 'red=\["I-5.5"\]' || true) no_judge_line=$(grep "^name=$2 " "$f" | grep -vc 'red=\[' || true)"
}
OPUS_K_MAX=20 proto runA1 shape_a
OPUS_Z_MAX=40 proto runB1 shape_b
OPUS_S_BITS=1 proto runC0 shape_c
OPUS_ROOT_MASKS=singles OPUS_RECORD_MASKS=0x1 proto runC1-0x1 shape_c
OPUS_ROOT_MASKS=singles OPUS_RECORD_MASKS=ffffffffffffffff proto runC1-ffffffffffffffff shape_c
proto runD0 shape_d
proto runE shape_e
(cd "$WORK/repo" && patch -p1 < "$MODEL/walk-fix-arms.patch")
for arm in f1 f2; do
  OPUS_I55_ARM=$arm OPUS_K_MAX=12 proto "runA-$arm" shape_a
  OPUS_I55_ARM=$arm OPUS_Z_MAX=30 proto "runB-$arm" shape_b
  OPUS_I55_ARM=$arm binary "arm-$arm-pool_checker" pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement
  OPUS_I55_ARM=$arm binary "arm-$arm-checker_know" checker_known_bad_images -- the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target
done
python3 "$MODEL/apply_row.py" "$MODEL/rows.tsv" 2 "$WORK/repo" apply "$FROZEN"
for arm in orig f1 f2; do
  if [ "$arm" = orig ]; then OPUS_K_MAX=12 proto "runA-a2c-$arm" shape_a; else OPUS_I55_ARM=$arm OPUS_K_MAX=12 proto "runA-a2c-$arm" shape_a; fi
done
python3 "$MODEL/apply_row.py" "$MODEL/rows.tsv" 2 "$WORK/repo" revert "$FROZEN"
python3 "$MODEL/apply_row.py" "$MODEL/rows-h1.tsv" 1 "$WORK/repo" apply "$FROZEN"
for arm in orig f1 f2; do
  if [ "$arm" = orig ]; then OPUS_K_MAX=12 proto "runA-h1-$arm" shape_a; else OPUS_I55_ARM=$arm OPUS_K_MAX=12 proto "runA-h1-$arm" shape_a; fi
done
python3 "$MODEL/apply_row.py" "$MODEL/rows-h1.tsv" 1 "$WORK/repo" revert "$FROZEN"
cp "$FROZEN/crates/singlefs-checker/src/walk.rs" "$WORK/repo/crates/singlefs-checker/src/walk.rs"
for n in 1 2 3 4 5 6 7 8; do
  nice -n 19 bash "$CAP" 3G bash "$CAPPED" "$THREADS" bash "$MODEL/run_row.sh" "$n" "$WORK" > "$WORK/out/row$n.summary" 2>&1 || true
done
{
  count runA1 shape_a; count runB1 shape_b; count runA-f1 shape_a; count runA-f2 shape_a; count runB-f1 shape_b; count runB-f2 shape_b
  count runA-a2c-orig shape_a; count runA-a2c-f1 shape_a; count runA-a2c-f2 shape_a; count runE shape_e
  count runA-h1-orig shape_a; count runA-h1-f1 shape_a; count runA-h1-f2 shape_a
  for f in runC1-0x1 runC1-ffffffffffffffff; do count "$f" shape_c; done
  grep -h '^name=shape_d' "$WORK/out/runD0.log"
  for f in arm-f1-pool_checker arm-f1-checker_know arm-f2-pool_checker arm-f2-checker_know; do echo "$f $(grep -h 'test result' "$WORK/out/$f.log")"; done
  for n in 1 2 3 4 5 6 7 8; do grep -h "^row=$n cargo_rc" "$WORK/out/row$n.summary"; done
} > "$WORK/out/summary.txt"
cat "$WORK/out/summary.txt"
