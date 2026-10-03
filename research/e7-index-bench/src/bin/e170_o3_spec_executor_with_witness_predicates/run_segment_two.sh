#!/usr/bin/env bash
# E170 第 2 次重跑第二段（`research/prompts/e170-r2-prereg.md` 5.3、5.4、5.8 第二段那一行）：输入核对、v1 现跑与入库产物比、
# v2 判 I1–I4 / s4 三行 / 几何点、I5、参照谓词 W_ref 与 T_ref、X21 / X27 在 s4 行上各一趟、两份比对器。全是 Python，不编 Rust。
# 跑法（在仓根，整条脚本经两层包装）：
#   bash research/scripts/capped.sh <线程> bash research/scripts/run-with-memory-cap.sh 8G \
#     bash research/e7-index-bench/src/bin/e170_o3_spec_executor_with_witness_predicates/run_segment_two.sh <日期标签> <工作进程数> <草稿目录> [产物目录]
# 产物：research/results/e170-r2-o3-spec-executor-<日期标签>-s2-*.out，已存在就不覆盖（noclobber，executor_main 自己也拒绝）。
# 每一步的退出码记进 -s2-run-status.out；一步失败不吞，照记、接着跑后面不依赖它的步（依赖它的那一步会因输入缺失自己失败、同样记下）。
set -u -o pipefail -o noclobber
TAG="${1:?日期标签}"
WORKERS="${2:?工作进程数}"
SCRATCH="${3:?草稿目录}"
ROOT="$(cd "$(dirname "$0")/../../../../.." && pwd)"
export PYTHONDONTWRITEBYTECODE=1
V1="$ROOT/research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor"
V2="$ROOT/research/e7-index-bench/src/bin/e170_o3_spec_executor_with_witness_predicates"
TARGET="$ROOT/research/target"
RESULTS_DIRECTORY="${4:-$ROOT/research/results}"  # 第 4 个参数只给试跑用（指到草稿目录）；正式那一趟不给，写进 research/results
P="$RESULTS_DIRECTORY/e170-r2-o3-spec-executor-$TAG-s2"
V1_FREEZE="$ROOT/research/results/e170-r1-o3-spec-executor-2026-10-03-s1-executor.out"
V2_FREEZE="$ROOT/research/results/e170-r2-o3-spec-executor-2026-10-04-s1-executor.out"
TABLE="$ROOT/research/mutations/e170_o3_spec_executor_with_witness_predicates.tsv"
I1="$TARGET/e170-dumps"; I2="$TARGET/e170-second-run-dumps"; I4="$TARGET/e171-dumps/e170-bad-states-copy-s2"
KNOWN_BAD="$TARGET/e171-dumps/known-bad-s2"; BASELINE="$TARGET/e171-dumps/baseline-run1"; ROWS="$TARGET/e171-dumps/rows-s4-f4"
STREAMS="new_pool_file_creation fixed_script_through_e_then_unmount"
STATUS="$P-run-status.out"
: > "$STATUS"
step() { local name="$1"; shift; "$@"; local code=$?; echo "name=step step=$name exit=$code" >> "$STATUS"; return 0; }

freeze_check() {  # <名> <执行器目录> <带冻结行的产物> <when>：冻结行此刻照算一遍，与第一段（v1 是第 1 次重跑第一段）产物里的逐行比（V8）
  (cd "$2" && python3 -c "
import sys; sys.path.insert(0, '.')
import executor_main
now = []; executor_main.freeze_lines(now.append)
reference = [line.rstrip('\n') for line in open('$3') if line.startswith(('name=freeze ', 'name=freeze_seed '))]
print(f'name=freeze_check executor=$1 when=$4 identical={str(now == reference and len(reference) > 0).lower()} lines_now={len(now)} reference_lines={len(reference)} seed_now={now[-1].split(\"=\")[-1]}')")
}

inputs() {  # 5.3：每个导出目录照 manifest.tsv 的 sha256: 行核；manifest 不列的坏状态文件逐个算 sha256（name=input_file）；B8–B10
  for directory in "$I1" "$I2" "$I4" "$BASELINE" "$ROWS/row-242/dump" "$ROWS/row-269/dump" "$ROWS/row-270/dump"; do
    for stream in $STREAMS; do
      local checked
      checked="$(cd "$directory/$stream" && grep '^sha256:' manifest.tsv | awk -F'\t' '{sub(/^sha256:/, "", $1); print $2 "  " $1}' | sha256sum -c 2>&1)"
      echo "name=manifest_check dir=${directory#"$ROOT"/}/$stream ok=$(grep -c ': OK$' <<< "$checked") failed=$(grep -vc ': OK$' <<< "$checked")"
    done
  done
  for directory in "$I2" "$I4"; do
    for stream in $STREAMS; do
      for file in bad-states.tsv bad-patches.bin bad-expected.tsv; do
        echo "name=input_file path=${directory#"$ROOT"/}/$stream/$file sha256=$(sha256sum "$directory/$stream/$file" | cut -d' ' -f1)"
      done
    done
  done
  for file in known-bad-states.tsv known-bad-patches.bin known-bad-o2.tsv; do
    echo "name=input_file path=${KNOWN_BAD#"$ROOT"/}/$file sha256=$(sha256sum "$KNOWN_BAD/$file" | cut -d' ' -f1)"
  done
  (cd "$V2" && python3 -c "
import crc32c_castagnoli as c
values = {'B8_ascii_123456789': (c.crc32c(b'123456789'), 0xE3069283), 'B9_zero4096': (c.crc32c(bytes(4096)), 0x98F94189), 'B10_zero512': (c.crc32c(bytes(512)), 0x30FCEDC0)}
for name, (got, want) in values.items(): print(f'name=crc_anchor id={name} got=0x{got:08X} want=0x{want:08X} holds={str(got == want).lower()}')")
  freeze_check v1 "$V1" "$V1_FREEZE" start
  freeze_check v2 "$V2" "$V2_FREEZE" start
}
step inputs inputs > "$P-inputs.out"

run_executor() {  # <版本目录> <导出目录> <批> <产物>
  (cd "$1" && python3 executor_main.py --dumps "$2" --batches "$3" --workers "$WORKERS" --output "$4")
}
step v1-i1 run_executor "$V1" "$I1" 1,2 "$P-v1-i1.out"
step v1-i2 run_executor "$V1" "$I2" 1,2 "$P-v1-i2.out"
for row in 242 269 270; do step "v1-row-$row" run_executor "$V1" "$ROWS/row-$row/dump" 1,2 "$P-v1-row-$row.out"; done
step v2-i1 run_executor "$V2" "$I1" 1,2,3 "$P-v2-i1.out"
step v2-i2 run_executor "$V2" "$I2" 1,2,3 "$P-v2-i2.out"
for row in 242 269 270; do step "v2-row-$row" run_executor "$V2" "$ROWS/row-$row/dump" 1,2 "$P-v2-row-$row.out"; done
step v2-e171-s4-baseline run_executor "$V2" "$BASELINE" 1,2,3 "$P-v2-e171-s4-baseline.out"

judge_images() {  # <v 名> <版本目录> <冻结参照> <输入名> <导出目录> [已知坏镜像目录]
  local known=(); [ $# -ge 6 ] && known=(--known-bad-dir "$6")
  (cd "$V2" && python3 comparator_bad_image_judge.py --executor-directory "$2" --input-label "$4" --dumps "$5" "${known[@]}" \
     --freeze-reference "$3" --output "$P-$1-$(tr 'A-Z' 'a-z' <<< "$4")-images.out")
}
step v1-i3-images judge_images v1 "$V1" "$V1_FREEZE" I3 "$I2"
step v1-i4-images judge_images v1 "$V1" "$V1_FREEZE" I4 "$I4" "$KNOWN_BAD"
step v2-i3-images judge_images v2 "$V2" "$V2_FREEZE" I3 "$I2"
step v2-i4-images judge_images v2 "$V2" "$V2_FREEZE" I4 "$I4" "$KNOWN_BAD"

unit_tests_of_version_one_on_version_two() {  # I5（读法 5、Q1-e）：v1 冻结的单测文件原样，对 v2 的模块跑；<副本名> [also_v1_helper]
  local copy="$SCRATCH/$1"
  mkdir "$copy" || return 3
  find "$V2" -maxdepth 1 -type f ! -name 'comparator*' ! -name 'mutation_driver*' ! -name 'witness_reference*' ! -name 'run_*' ! -name '*.pyc' \
    -exec cp {} "$copy/" \;
  cp "$V1/test_executor_on_synthetic_pool.py" "$copy/test_executor_on_synthetic_pool.py"
  [ "${2:-}" = also_v1_helper ] && cp "$V1/synthetic_pool_for_tests.py" "$copy/synthetic_pool_for_tests.py"
  echo "name=i5 copy=$1 command=python3_-m_unittest_-v_test_executor_on_synthetic_pool helper=${2:-v2_helper}"
  (cd "$copy" && sha256sum test_executor_on_synthetic_pool.py synthetic_pool_for_tests.py executor_core.py witness_predicates.py | sed 's/^/name=tested_file_sha256 /')
  mkdir "$copy/test-scratch"
  (cd "$copy" && E170_TEST_SCRATCH="$copy/test-scratch" python3 -m unittest -v test_executor_on_synthetic_pool 2>&1); local code=$?
  echo "name=i5_exit code=$code"
  rm -rf "${copy:?}"
  return $code
}
step i5 unit_tests_of_version_one_on_version_two i5-copy > "$P-i5-v1-tests-on-v2.out"
step i5-with-v1-helper unit_tests_of_version_one_on_version_two i5-copy-with-v1-helper also_v1_helper > "$P-i5-v1-tests-and-helper-on-v2.out"

reference() {  # <states|images> <输入名> <导出目录> <v2 产物> <产物名> [已知坏镜像目录]
  local known=(); [ $# -ge 6 ] && known=(--known-bad-dir "$6")
  (cd "$V2" && python3 witness_reference.py "$1" --dumps "$3" "${known[@]}" --v2-output "$4" --input-label "$2" --output "$P-reference-$5.out")
}
step reference-i1 reference states I1 "$I1" "$P-v2-i1.out" i1
step reference-i2 reference states I2 "$I2" "$P-v2-i2.out" i2
for row in 242 269 270; do step "reference-row-$row" reference states "row-$row" "$ROWS/row-$row/dump" "$P-v2-row-$row.out" "row-$row"; done
step reference-e171-s4-baseline reference states e171-s4-baseline "$BASELINE" "$P-v2-e171-s4-baseline.out" e171-s4-baseline
step reference-i3-images reference images I3 "$I2" "$P-v2-i3-images.out" i3-images
step reference-i4-images reference images I4 "$I4" "$P-v2-i4-images.out" i4-images "$KNOWN_BAD"

mutated_run() {  # <变异> <行号>
  (cd "$V2" && python3 mutation_driver_on_rows.py --mutation "$1" --table "$TABLE" --scratch "$SCRATCH" --dumps "$ROWS/row-$2/dump" \
     --batches 1,2 --workers "$WORKERS" --output "$P-$(tr 'A-Z' 'a-z' <<< "$1")-row-$2.out")
}
step x21-row-242 mutated_run X21 242 >> "$STATUS"
step x21-row-269 mutated_run X21 269 >> "$STATUS"
step x27-row-270 mutated_run X27 270 >> "$STATUS"

step v1-against-stored bash -c "cd '$V2' && python3 comparator_v1_against_stored.py \
  --pair I1:states:'$P-v1-i1.out':'$ROOT/research/results/e170-r1-o3-spec-executor-2026-10-03-s1-executor.out' \
  --pair I2:states:'$P-v1-i2.out':'$ROOT/research/results/e170-r1-o3-spec-executor-2026-10-03-s2-executor.out' \
  --pair I3:bad_states:'$P-v1-i3-images.out':'$ROOT/research/results/e170-r1-o3-spec-executor-2026-10-03-s3-bad-states.out' \
  --pair I4:images:'$P-v1-i4-images.out':'$ROOT/research/results/e171-three-oracle-collusion-2026-10-03-s2-images-o3.out' \
  --pair row-242:states:'$P-v1-row-242.out':'$ROWS/row-242/o3.out' --pair row-269:states:'$P-v1-row-269.out':'$ROWS/row-269/o3.out' \
  --pair row-270:states:'$P-v1-row-270.out':'$ROWS/row-270/o3.out' --output '$P-v1-against-stored.out'"
step v2-i1-against-segment-one bash -c "cd '$V2' && python3 comparator_v1_against_stored.py \
  --pair I1-v2-batches-1-2:states:'$P-v2-i1.out':'$V2_FREEZE' --output '$P-v2-i1-against-segment-one.out'"

step comparator-q1 bash -c "cd '$V2' && python3 comparator_two_versions.py \
  --pair I1:states:'$P-v1-i1.out':'$P-v2-i1.out' --pair I2:states:'$P-v1-i2.out':'$P-v2-i2.out' \
  --pair I3:images:'$P-v1-i3-images.out':'$P-v2-i3-images.out' --pair I4:images:'$P-v1-i4-images.out':'$P-v2-i4-images.out' \
  --batches 1,2 --names I3:'$I2' --names I4:'$I4':'$KNOWN_BAD' --unit-tests '$P-i5-v1-tests-on-v2.out' \
  --anchors '$P-v2-i1.out' --anchors '$P-v2-i2.out' --positive-control-input I2 --scratch '$SCRATCH' --output '$P-comparator-q1.out'"

step comparator-q2-q3 bash -c "cd '$V2' && python3 comparator_witness_classes.py \
  --row 242:'$P-v1-row-242.out':'$P-v2-row-242.out':'$P-reference-row-242.out' \
  --row 269:'$P-v1-row-269.out':'$P-v2-row-269.out':'$P-reference-row-269.out' \
  --row 270:'$P-v1-row-270.out':'$P-v2-row-270.out':'$P-reference-row-270.out' \
  --mutated 242:X21:'$P-x21-row-242.out' --mutated 269:X21:'$P-x21-row-269.out' --mutated 270:X27:'$P-x27-row-270.out' \
  --legal I1:previous_legal_history:states:1,2:'$P-v2-i1.out':'$P-reference-i1.out' \
  --legal I2:previous_legal_history:states:1,2:'$P-v2-i2.out':'$P-reference-i2.out' \
  --legal I3-base:previous_legal_history:images:-:'$P-v2-i3-images.out':'$P-reference-i3-images.out' \
  --legal I4-base:previous_legal_history:images:-:'$P-v2-i4-images.out':'$P-reference-i4-images.out' \
  --legal I1-batch-3:batch_three:states:3:'$P-v2-i1.out':'$P-reference-i1.out' \
  --legal I2-batch-3:batch_three:states:3:'$P-v2-i2.out':'$P-reference-i2.out' \
  --legal e171-s4-baseline:e171_s4_baseline:states:1,2,3:'$P-v2-e171-s4-baseline.out':'$P-reference-e171-s4-baseline.out' \
  --positive-control-legal-input I2 --scratch '$SCRATCH' --output '$P-comparator-q2-q3.out'"

freeze_check v1 "$V1" "$V1_FREEZE" end >> "$STATUS"
freeze_check v2 "$V2" "$V2_FREEZE" end >> "$STATUS"
echo "name=complete failed_steps=$(grep -c 'exit=[1-9]' "$STATUS")" >> "$STATUS"
