#!/usr/bin/env bash
# E170 第 2 次重跑第三段（`research/prompts/e170-r2-prereg.md` 5.8 第三段那一行、第九节）：变异。全是 Python，不编 Rust。
# 做：输入核对（导出目录 manifest、第二段产物的 sha256）、v1 / v2 冻结行开头核对、变异驱动一趟（v1 的 X1–X20 与 v2 的 X22–X26、X28–X31 单测层，
# X22–X26、X28–X31 批层，XC3–XC7）、冻结行收尾核对。全部写进同一份产物 research/results/e170-r2-o3-spec-executor-<日期标签>-s3-mutations.out，
# 已存在就不覆盖（noclobber）。前台跑得完则前台跑；超过 240 秒的交主 agent 用 run_in_background 起。
# 跑法（在仓根，整条脚本经两层包装）：
#   bash scripts/hosts/capped.sh <线程> bash scripts/hosts/run-with-memory-cap.sh <上限> \
#     bash scripts/experiments/e170_o3_spec_executor_with_witness_predicates/run_segment_three.sh <日期标签> <工作进程数> <草稿目录> [产物目录]
set -u -o pipefail -o noclobber
TAG="${1:?日期标签}"
WORKERS="${2:?工作进程数}"
SCRATCH="${3:?草稿目录}"
ROOT="$(cd "$(dirname "$0")/../../.. && pwd)"
export PYTHONDONTWRITEBYTECODE=1
V1="$ROOT/scripts/experiments/e170_second_run_o3_spec_executor"
V2="$ROOT/scripts/experiments/e170_o3_spec_executor_with_witness_predicates"
TARGET="$ROOT/scripts/experiments/target"
RESULTS_DIRECTORY="${4:-$ROOT/research/results}"  # 第 4 个参数只给试跑用（产物写到草稿目录）；第二段产物一律从 research/results 读
INPUT_PRODUCTS_DIRECTORY="$ROOT/research/results"
PRODUCT="$RESULTS_DIRECTORY/e170-r2-o3-spec-executor-$TAG-s3-mutations.out"
V1_FREEZE="$ROOT/research/results/e170-r1-o3-spec-executor-2026-10-03-s1-executor.out"
V2_FREEZE="$ROOT/research/results/e170-r2-o3-spec-executor-2026-10-04-s1-executor.out"
STREAMS="new_pool_file_creation fixed_script_through_e_then_unmount"

freeze_check() {  # <名> <执行器目录> <带冻结行的产物> <when>：冻结行此刻照算一遍，与第一段（v1 是第 1 次重跑第一段）产物里的逐行比（V8）
  (cd "$2" && python3 -c "
import sys; sys.path.insert(0, '.')
import executor_main
now = []; executor_main.freeze_lines(now.append)
reference = [line.rstrip('\n') for line in open('$3') if line.startswith(('name=freeze ', 'name=freeze_seed '))]
print(f'name=freeze_check executor=$1 when=$4 identical={str(now == reference and len(reference) > 0).lower()} lines_now={len(now)} reference_lines={len(reference)} seed_now={now[-1].split(\"=\")[-1]}')")
}

inputs() {  # V2：用到的导出目录照 manifest.tsv 的 sha256: 行核；第二段产物（比对器与参照谓词的输入）逐个记 sha256；B8–B10；冻结行开头核对
  for directory in "$TARGET/e170-dumps" "$TARGET/e170-second-run-dumps" "$TARGET/e171-dumps/e170-bad-states-copy-s2" "$TARGET/e171-dumps/baseline-run1" \
      "$TARGET/e171-dumps/rows-s4-f4/row-242/dump" "$TARGET/e171-dumps/rows-s4-f4/row-269/dump" "$TARGET/e171-dumps/rows-s4-f4/row-270/dump"; do
    for stream in $STREAMS; do
      local checked
      checked="$(cd "$directory/$stream" && grep '^sha256:' manifest.tsv | awk -F'\t' '{sub(/^sha256:/, "", $1); print $2 "  " $1}' | sha256sum -c 2>&1)"
      echo "name=manifest_check dir=${directory#"$ROOT"/}/$stream ok=$(grep -c ': OK$' <<< "$checked") failed=$(grep -vc ': OK$' <<< "$checked")"
    done
  done
  for file in "$INPUT_PRODUCTS_DIRECTORY"/e170-r2-o3-spec-executor-2026-10-04-s2-*.out; do
    echo "name=input_product file=$(basename "$file") sha256=$(sha256sum "$file" | cut -d' ' -f1)"
  done
  (cd "$V2" && python3 -c "
import crc32c_castagnoli as c
values = {'B8_ascii_123456789': (c.crc32c(b'123456789'), 0xE3069283), 'B9_zero4096': (c.crc32c(bytes(4096)), 0x98F94189), 'B10_zero512': (c.crc32c(bytes(512)), 0x30FCEDC0)}
for name, (got, want) in values.items(): print(f'name=crc_anchor id={name} got=0x{got:08X} want=0x{want:08X} holds={str(got == want).lower()}')")
  freeze_check v1 "$V1" "$V1_FREEZE" start
  freeze_check v2 "$V2" "$V2_FREEZE" start
}

{ echo "name=config segment=3 tag=$TAG workers=$WORKERS scratch=$SCRATCH results_directory=$RESULTS_DIRECTORY"; inputs; } > "$PRODUCT"
mkdir -p "$SCRATCH/driver-scratch"
(cd "$V2" && python3 mutation_driver_third_segment.py --scratch "$SCRATCH/driver-scratch" --output "$PRODUCT" --workers "$WORKERS" \
   --results-directory "$INPUT_PRODUCTS_DIRECTORY" > "$SCRATCH/driver.log" 2>&1)
echo "name=step step=mutation_driver exit=$?" >> "$PRODUCT"
{ freeze_check v1 "$V1" "$V1_FREEZE" end; freeze_check v2 "$V2" "$V2_FREEZE" end; } >> "$PRODUCT"
echo "name=complete failed_steps=$(grep -c 'name=step .* exit=[1-9]' "$PRODUCT")" >> "$PRODUCT"
