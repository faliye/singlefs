#!/usr/bin/env bash
# m3-formal-code-b18-r1 攻方原型复跑（副本上的数：只改拷出来的副本，仓内与冻结副本一个字节不碰）。
# 用法：bash tools/rerun.sh <quick|sweep|rand|all> [冻结树根（含 crates/），默认 /tmp/claude-1000/m3-formal-code-b18-r1/tree] [工作目录，默认 ${TMPDIR:-/tmp}/b18-opus-rerun]
#   quick  t1（靶向第二截那条用例的输入）× 5 个开关、t3 种子 0（第三截判别力那条用例的输入）× 7 个开关 × {基线, 加 V242u}，再加 F3 原型两趟（约 3 分钟）
#   sweep  t3 种子 k×4096（k=1–42）三批：加 V242u、基线、加 V242u 与 F3 原型（约 9 分钟）
#   rand   BROAD 比重的随机历史 60 段（种子 k×4096，k=1–60，8 步、每段 16 个崩溃态）基线（约 3 分钟）
# 线程上限 B18_THREADS（默认 6）；每趟经 scripts/run-with-memory-cap.sh，上限 64M（原型单趟峰值约 24M）。
# 判结果：读工作目录 out/ 下各文件的 t1 / t3 / rand 行与 per-trial.txt；报告里引的行就是这些。
set -euo pipefail
MODEL=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
REPO=$(cd "$MODEL/../../.." && pwd)
mode=${1:-quick}
TREE=${2:-/tmp/claude-1000/m3-formal-code-b18-r1/tree}
WORK=${3:-${TMPDIR:-/tmp}/b18-opus-rerun}
THREADS=${B18_THREADS:-6}
CAP="bash $REPO/scripts/run-with-memory-cap.sh 64M"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="复跑攻方原型上限固定为六十四兆不随峰值表变"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="复跑攻方原型小批并行各件互不共享状态"

mkdir -p "$WORK"
rm -rf "${WORK:?}/copy" "${WORK:?}/out"
mkdir -p "$WORK/copy" "$WORK/out"
rsync -a "$TREE/crates" "$WORK/copy/"
cp "$REPO/Cargo.toml" "$REPO/Cargo.lock" "$WORK/copy/"
mkdir -p "$WORK/copy/.cargo"
cp "$REPO/.cargo/config.toml" "$WORK/copy/.cargo/"
python3 "$MODEL/tools/patch_copy.py" "$WORK/copy"
python3 "$MODEL/tools/patch_copy_2.py" "$WORK/copy"
python3 "$MODEL/tools/patch_copy_3.py" "$WORK/copy"
mkdir -p "$WORK/copy/crates/b18-attack-proto/src"
cp "$MODEL/proto/Cargo.toml" "$WORK/copy/crates/b18-attack-proto/Cargo.toml"
cp "$MODEL/proto/main.rs" "$WORK/copy/crates/b18-attack-proto/src/main.rs"
python3 - "$WORK/copy/Cargo.toml" <<'PY'
import sys, pathlib
p = pathlib.Path(sys.argv[1]); t = p.read_text()
old = '    "crates/singlefs-reference-filesystem",\n]'
assert t.count(old) == 1, "工作区成员表的末行对不上"
p.write_text(t.replace(old, '    "crates/singlefs-reference-filesystem",\n    "crates/b18-attack-proto",\n]'))
PY
(cd "$WORK/copy" && CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$REPO/scripts/capped.sh" "$THREADS" cargo build --offline -p b18-attack-proto)
B="$WORK/target/debug/b18-attack-proto"

# 一波最多 THREADS 件并行，每件的退出码写进 out/<名>.rc；收齐之后数一遍，件数对不上整波作废。
jobs_names=(); jobs_cmds=()
add_job() { jobs_names+=("$1"); jobs_cmds+=("$2"); }
run_jobs() {
  local i=0 total=${#jobs_names[@]}
  while [ "$i" -lt "$total" ]; do
    local wave=()
    local j
    for ((j = i; j < total && j < i + THREADS; j++)); do
      wave+=("$j")
      rm -f "$WORK/out/${jobs_names[$j]}.rc"
    done
    for j in "${wave[@]}"; do
      { bash -c "${jobs_cmds[$j]}" > "$WORK/out/${jobs_names[$j]}.out" 2> "$WORK/out/${jobs_names[$j]}.err"; echo "$?" > "$WORK/out/${jobs_names[$j]}.rc"; } &
    done
    wait # shell-lint:exit-collected 每件退出码写进 out/<名>.rc，下面逐个读
    for j in "${wave[@]}"; do
      if [ ! -f "$WORK/out/${jobs_names[$j]}.rc" ]; then echo "作废：${jobs_names[$j]} 没留下退出码"; exit 1; fi
      echo "${jobs_names[$j]} rc=$(cat "$WORK/out/${jobs_names[$j]}.rc")"
    done
    i=$((i + THREADS))
  done
  jobs_names=(); jobs_cmds=()
}

if [ "$mode" = quick ] || [ "$mode" = all ]; then
  for v in none s2_version_none s2_index_write s2_call_injected s2_old_judge; do
    add_job "t1-$v" "B18_VARIANT=$v nice -n 19 $CAP $B t1"
  done
  for v in none s2_version_none s2_index_write s3_call_injected s3_call_first_image s3_call_first_version s3_call_base_without_segment; do
    add_job "t3-$v" "B18_VARIANT=$v nice -n 19 $CAP $B t3 0 1"
    add_job "t3v242u-$v" "B18_VARIANT=$v B18_V242U=1 B18_TRACE=1 nice -n 19 $CAP $B t3 0 1"
  done
  add_job "t3-s3_decorrelate" "B18_VARIANT=s3_decorrelate nice -n 19 $CAP $B t3 0 1"
  add_job "t3v242u-s3_decorrelate" "B18_VARIANT=s3_decorrelate B18_V242U=1 nice -n 19 $CAP $B t3 0 1"
  run_jobs
  python3 "$MODEL/tools/summarize_trace.py" "$WORK/out/t3v242u-none.err" row_publish > "$WORK/out/seed0-row-publish-groups.txt"
fi
if [ "$mode" = sweep ] || [ "$mode" = all ]; then
  for k in 0 1 2 3 4 5; do
    a=$((1 + k * 7)); b=$((a + 7))
    add_job "vB-$k" "B18_V242U=1 B18_TRACE=1 nice -n 19 $CAP $B t3 $a $b 4096"
    add_job "bB-$k" "nice -n 19 $CAP $B t3 $a $b 4096"
    add_job "vC-$k" "B18_V242U=1 B18_VARIANT=s3_decorrelate nice -n 19 $CAP $B t3 $a $b 4096"
  done
  run_jobs
  python3 "$MODEL/tools/per_trial.py" "$WORK"/out/vB-*.err > "$WORK/out/per-trial.txt"
fi
if [ "$mode" = rand ] || [ "$mode" = all ]; then
  for k in 0 1 2 3 4 5; do
    a=$((1 + k * 10)); b=$((a + 10))
    add_job "rA-$k" "nice -n 19 $CAP $B rand $a $b 4096 8 16"
  done
  run_jobs
fi
echo "输出在 $WORK/out"
