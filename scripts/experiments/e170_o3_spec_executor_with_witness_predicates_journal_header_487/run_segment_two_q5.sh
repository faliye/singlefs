#!/usr/bin/env bash
# E170 第 2 次重跑第二段 Q5 那一半（`research/prompts/e170-r2-prereg.md` 12.8 分段表「第二段」、修订 2 的 12.13–12.15）：
# Z 族输入核对（含第一段乙没核的 impl-verdicts.tsv）、v2-487 判 Z1 批 1–3 / Z3 / Z4 / Z5 批 1–3、Q5-pc（v2 原样判 Z1 / Z5 批 1–2、Z3、Z4）、
# Q5-e 第一项、Q5-d 与 Q4-fmt-487、照抄核对（Q5-i、Q5-pc3、XC8 / XC9）、Q5 比对器（含 B4-487、B15、B17、Q5-pc2）两趟。全是 Python，不编 Rust。
# 跑法（在仓根，整条脚本经两层包装）：
#   bash scripts/hosts/capped.sh <线程> bash scripts/hosts/run-with-memory-cap.sh 8G \
#     bash scripts/experiments/e170_o3_spec_executor_with_witness_predicates_journal_header_487/run_segment_two_q5.sh <日期标签> <工作进程数> <空的草稿目录> [产物目录]
# 产物：research/results/e170-r2-o3-spec-executor-<日期标签>-s2-q5-*.out 与 -s2b-known-bad-recopy*.out，已存在就不覆盖（noclobber；Python 一侧用 open(…, "x")）。
# 每一步的退出码记进 -s2-q5-run-status.out；一步失败不吞，照记、接着跑后面不依赖它的步。
set -u -o pipefail -o noclobber
TAG="${1:?日期标签}"
WORKERS="${2:?工作进程数}"
SCRATCH="${3:?草稿目录}"
ROOT="$(cd "$(dirname "$0")/../../.. && pwd)"
export PYTHONDONTWRITEBYTECODE=1
V2="$ROOT/scripts/experiments/e170_o3_spec_executor_with_witness_predicates"
V487="$ROOT/scripts/experiments/e170_o3_spec_executor_with_witness_predicates_journal_header_487"
V1487="$ROOT/scripts/experiments/e170_second_run_o3_spec_executor_journal_header_487"
RESULTS_DIRECTORY="${4:-$ROOT/research/results}"  # 第 4 个参数只给试跑用（指到草稿目录）；正式那一趟不给
P="$RESULTS_DIRECTORY/e170-r2-o3-spec-executor-$TAG-s2-q5"
PB="$RESULTS_DIRECTORY/e170-r2-o3-spec-executor-$TAG-s2b-known-bad-recopy"
S1B="$ROOT/research/results/e170-r2-o3-spec-executor-2026-10-04-s1b-executor.out"
V2_FREEZE="$ROOT/research/results/e170-r2-o3-spec-executor-2026-10-04-s1-executor.out"
EXPORT_PRODUCT="$ROOT/research/results/e170-r2-o3-spec-executor-2026-10-04-s2b-today-export-r3.out"
S2="$ROOT/research/results/e170-r2-o3-spec-executor-2026-10-04-s2"  # 第二段 Q1–Q3 那一趟：v2 在 311 冻结导出上的判定（Q5 的参照）
TABLE="$ROOT/scripts/mutation/tables/e170_o3_spec_executor_with_witness_predicates_journal_header_487.tsv"
Z="$ROOT/scripts/experiments/target/e170-r2-today-dumps"
Z1="$Z/e170-dump"; Z1R="$Z/e170-dump-repeat"; Z3="$Z/e170-bad-states-second-run"; Z4="$Z/e170-bad-states-copy-s2"; ZKB="$Z/known-bad-s2"; Z5="$Z/e171-baseline-run1"
I2="$ROOT/scripts/experiments/target/e170-second-run-dumps"; I4="$ROOT/scripts/experiments/target/e171-dumps/e170-bad-states-copy-s2"; IKB="$ROOT/scripts/experiments/target/e171-dumps/known-bad-s2"
TREE="$ROOT/scripts/experiments/target/e170-r2-today-tree"
DEVICE="crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs"; TEST_FILE="crates/singlefs-harness/tests/checker_known_bad_images.rs"
IMAGE_RS="crates/singlefs-checker/src/image.rs"; BEFORE_COMMIT=13ca0a19
STREAMS="new_pool_file_creation fixed_script_through_e_then_unmount"
STATUS="$P-run-status.out"
: > "$STATUS"
step() { local name="$1"; shift; "$@"; local code=$?; echo "name=step step=$name exit=$code" >> "$STATUS"; return 0; }

freeze_check() {  # <名> <执行器目录> <带冻结行的产物> <臂名或 -> <when>：冻结行此刻照算，与产物里那一组逐行比（V8 / V17）；臂名 - 取不带 arm= 的那一组
  (cd "$2" && python3 -c "
import sys; sys.path.insert(0, '.')
import executor_main
now = []; executor_main.freeze_lines(now.append)
arm = '$4'
reference = []
for line in open('$3', encoding='utf-8'):
    line = line.rstrip('\n').removeprefix('E7RESULT ')
    if not line.startswith(('name=freeze ', 'name=freeze_seed ')):
        continue
    if arm == '-' and ' arm=' not in line:
        reference.append(line)
    elif arm != '-' and f' arm={arm} ' in line:
        reference.append(line.replace(f' arm={arm} ', ' ', 1))
print(f'name=freeze_check executor=$1 group=$4 reference={\"$3\".split(\"/\")[-1]} when=$5 identical={str(now == reference and len(reference) > 0).lower()} lines_now={len(now)} reference_lines={len(reference)} seed_now={now[-1].split(\"=\")[-1]}')")
}

inputs() {  # 5.3 / 12.8：Z 族每个导出目录照 manifest.tsv 的 sha256: 行核（含 impl-verdicts.tsv，第一段乙那两项 not_checked 在这里补）；
            # manifest.tsv 自己的 sha256 对重导产物记的；manifest 不列的坏状态文件逐个记 sha256；用例文件与装置的快照 / 工作区 sha256；B8–B10；开头冻结
  local label directory stream checked
  for pair in "Z1:$Z1" "Z1r:$Z1R" "Z3:$Z3" "Z5:$Z5" "Z4:$Z4"; do
    label="${pair%%:*}"; directory="${pair#*:}"
    for stream in $STREAMS; do
      checked="$(cd "$directory/$stream" && grep '^sha256:' manifest.tsv | awk -F'\t' '{sub(/^sha256:/, "", $1); print $2 "  " $1}' | sha256sum -c 2>&1)"
      echo "name=manifest_check input=$label dir=${directory#"$ROOT"/}/$stream ok=$(grep -c ': OK$' <<< "$checked") failed=$(grep -vc ': OK$' <<< "$checked")"
      local listed actual recorded
      listed="$(awk -F'\t' '$1 == "sha256:impl-verdicts.tsv" {print $2}' "$directory/$stream/manifest.tsv")"
      actual="$(sha256sum "$directory/$stream/impl-verdicts.tsv" | cut -d' ' -f1)"
      echo "name=impl_verdicts_check input=$label stream=$stream manifest_lists=$listed actual=$actual equal=$([ "$listed" = "$actual" ] && echo true || echo false)"
      actual="$(sha256sum "$directory/$stream/manifest.tsv" | cut -d' ' -f1)"
      recorded="$(grep "^name=manifest_check directory=${directory##*/} stream=$stream " "$EXPORT_PRODUCT" | sed 's/.* manifest_sha256=\([0-9a-f]*\).*/\1/')"
      echo "name=manifest_file input=$label stream=$stream sha256=$actual export_product_records=${recorded:-not_recorded} equal=$([ "$recorded" = "$actual" ] && echo true || echo false)"
    done
  done
  for pair in "Z3:$Z3" "Z4:$Z4"; do
    label="${pair%%:*}"; directory="${pair#*:}"
    for stream in $STREAMS; do
      for file in bad-states.tsv bad-patches.bin bad-expected.tsv; do
        echo "name=input_file input=$label path=${directory#"$ROOT"/}/$stream/$file sha256=$(sha256sum "$directory/$stream/$file" | cut -d' ' -f1)"
      done
    done
  done
  for file in known-bad-states.tsv known-bad-patches.bin known-bad-o2.tsv; do
    echo "name=input_file input=Z4 path=${ZKB#"$ROOT"/}/$file sha256=$(sha256sum "$ZKB/$file" | cut -d' ' -f1)"
  done
  for file in "$DEVICE" "$TEST_FILE" "$IMAGE_RS"; do
    local snapshot working
    snapshot="$(sha256sum "$TREE/$file" | cut -d' ' -f1)"; working="$(sha256sum "$ROOT/$file" | cut -d' ' -f1)"
    echo "name=source_file path=$file snapshot_sha256=$snapshot working_tree_sha256=$working same=$([ "$snapshot" = "$working" ] && echo true || echo false)"
  done
  (cd "$V487" && python3 -c "
import crc32c_castagnoli as c
values = {'B8_ascii_123456789': (c.crc32c(b'123456789'), 0xE3069283), 'B9_zero4096': (c.crc32c(bytes(4096)), 0x98F94189), 'B10_zero512': (c.crc32c(bytes(512)), 0x30FCEDC0)}
for name, (got, want) in values.items(): print(f'name=crc_anchor id={name} got=0x{got:08X} want=0x{want:08X} holds={str(got == want).lower()}')")
  freeze_check v2_487 "$V487" "$S1B" - start
  freeze_check v2_487 "$V487" "$S1B" v2_487 start
  freeze_check v1_487 "$V1487" "$S1B" v1_487 start
  freeze_check v2 "$V2" "$V2_FREEZE" - start
}
step inputs inputs > "$P-inputs.out"
grep -E '^name=freeze(_seed)? ' "$S1B" | grep -v ' arm=' > "$SCRATCH/v2-487-freeze-reference.out"  # 判镜像那一份只认不带 arm= 的一组
NEW_GAPS="$(comm -13 <(cut -f1 "$V2/spec-gaps.tsv" | sort) <(cut -f1 "$V487/spec-gaps.tsv" | sort) | paste -sd, -)"  # 格式补丁新记的欠一句
echo "name=new_gaps identifiers=${NEW_GAPS:-none}" >> "$STATUS"

run_executor() {  # <版本目录> <导出目录> <批> <产物>
  (cd "$1" && python3 executor_main.py --dumps "$2" --batches "$3" --workers "$WORKERS" --output "$4")
}
judge_images() {  # <执行器目录> <冻结参照> <输入名> <导出目录> <产物> [已知坏镜像目录]
  local known=(); [ $# -ge 6 ] && known=(--known-bad-dir "$6")
  (cd "$V2" && python3 comparator_bad_image_judge.py --executor-directory "$1" --input-label "$3" --dumps "$4" "${known[@]}" \
     --freeze-reference "$2" --output "$5")
}
step v2-487-z1 run_executor "$V487" "$Z1" 1,2,3 "$P-v2-487-z1.out"
step v2-487-z5 run_executor "$V487" "$Z5" 1,2,3 "$P-v2-487-z5.out"
step v2-487-z3-images judge_images "$V487" "$SCRATCH/v2-487-freeze-reference.out" Z3 "$Z3" "$P-v2-487-z3-images.out"
step v2-487-z4-images judge_images "$V487" "$SCRATCH/v2-487-freeze-reference.out" Z4 "$Z4" "$P-v2-487-z4-images.out" "$ZKB"
step pc-v2-z1 run_executor "$V2" "$Z1" 1,2 "$P-pc-v2-z1.out"
step pc-v2-z5 run_executor "$V2" "$Z5" 1,2 "$P-pc-v2-z5.out"
step pc-v2-z3-images judge_images "$V2" "$V2_FREEZE" Z3 "$Z3" "$P-pc-v2-z3-images.out"
step pc-v2-z4-images judge_images "$V2" "$V2_FREEZE" Z4 "$Z4" "$P-pc-v2-z4-images.out" "$ZKB"
step record-paths bash -c "cd '$V487' && python3 run_q5_record_path_counts.py --executor-directory '$V487' --states Z1:'$Z1':1,2,3 \
  --states Z5:'$Z5':1,2,3 --images Z3:'$Z3' --images Z4:'$Z4':'$ZKB' --output '$P-record-paths.out'"
step format-constants bash -c "cd '$V487' && python3 comparator_q5_format_constants.py --executor-directory '$V487' --tree '$TREE' \
  --working-tree '$ROOT' --output '$P-format-constants.out'"

recopy_check() {  # <照抄后的装置> <核对脚本>：读法 18、19 的照抄核对；照抄前的装置取 git 里那一次提交（12.12 第 2 条）
  (cd "$ROOT" && python3 "$2" "$SCRATCH/e171-before-$BEFORE_COMMIT.rs" "$1" "$TEST_FILE" "${IKB#"$ROOT"/}/known-bad-o2.tsv" "$IMAGE_RS")
}
git -C "$ROOT" show "$BEFORE_COMMIT:$DEVICE" > "$SCRATCH/e171-before-$BEFORE_COMMIT.rs"
step known-bad-recopy recopy_check "$DEVICE" "$V487/run_known_bad_recopy_check.py" > "$PB.out"
pc3_copy() {  # Q5-pc3（12.14）：照抄后的装置复制一份，序号 5 那一条目标名之后第一个数加 1，目标名不动（重导准备第三趟草稿 make_pc3_copy.py 的同一取法）
  python3 - "$ROOT/$DEVICE" "$SCRATCH/pc3-device.rs" <<'PY'
import re, sys
source, destination = sys.argv[1:3]
lines = open(source, encoding="utf-8").read().split("\n")
start = next(i for i, line in enumerate(lines) if line.startswith("    fn known_bad_images(clean: &MemoryPool)"))
openings = [i for i in range(start, len(lines)) if lines[i] == "            ("]
target_line = openings[5] + 1
for index in range(target_line + 1, len(lines)):
    match = re.search(r"\b\d+\b", lines[index])
    if match and not lines[index].strip().startswith("//"):
        before = lines[index]
        lines[index] = before[:match.start()] + str(int(match.group(0)) + 1) + before[match.end():]
        print(f"name=pc3_copy entry=5 target={lines[target_line].strip()} line={index + 1} before={before.strip()!r} after={lines[index].strip()!r}")
        break
with open(destination, "x", encoding="utf-8") as handle:
    handle.write("\n".join(lines))
PY
}
pc3_run() { pc3_copy && recopy_check "$SCRATCH/pc3-device.rs" "$V487/run_known_bad_recopy_check.py"; }
step known-bad-recopy-pc3 pc3_run > "$PB-pc3.out"  # 期望退 3（复制件上照抄前后不同恰 1），不算失败的步
recopy_mutations() {  # XC8、XC9（12.14）：变异表里 XC 两行逐条在草稿副本上替换（原文在核对脚本里恰好命中 1 次），XC8 跑 pc3 复制件、XC9 跑真的照抄后装置
  # 抓到 = 退出码或判决行与同一输入上没变异那一趟（$PB-pc3.out / $PB.out 的末行）不同；12.14 点名的那一格（照抄前后不同）另报前后两个数
  local name anchor replacement input baseline_exit baseline_line hits code verdict
  while IFS=$'\t' read -r name anchor replacement; do
    case "$name" in
      XC8_*) input="$SCRATCH/pc3-device.rs"; baseline_exit=3; baseline_line="$(tail -1 "$PB-pc3.out")" ;;
      XC9_*) input="$DEVICE"; baseline_exit=0; baseline_line="$(tail -1 "$PB.out")" ;;
      *) continue ;;
    esac
    hits="$(grep -cF -- "$anchor" "$V487/run_known_bad_recopy_check.py")"
    echo "name=mutation_anchor mutation=$name file=run_known_bad_recopy_check.py hits=$hits"
    [ "$hits" = 1 ] || { echo "name=mutation_run mutation=$name result=invalid reason=anchor_hits_$hits"; continue; }
    ANCHOR="$anchor" REPLACEMENT="$replacement" python3 -c "
import os, sys
text = open(sys.argv[1], encoding='utf-8').read()
open(sys.argv[2], 'x', encoding='utf-8').write(text.replace(os.environ['ANCHOR'], os.environ['REPLACEMENT'], 1))" \
      "$V487/run_known_bad_recopy_check.py" "$SCRATCH/mutant-$name.py"
    recopy_check "$input" "$SCRATCH/mutant-$name.py" > "$SCRATCH/mutant-$name.out"; code=$?
    verdict="$(tail -1 "$SCRATCH/mutant-$name.out")"
    echo "name=mutation_verdict_line mutation=$name $verdict"
    echo "name=mutation_run mutation=$name input=${input##*/} exit=$code unmutated_exit=$baseline_exit" \
      "recopy_differences_unmutated=$(sed 's/.*recopy_differences=\([0-9]*\).*/\1/' <<< "$baseline_line")" \
      "recopy_differences_mutated=$(sed 's/.*recopy_differences=\([0-9]*\).*/\1/' <<< "$verdict")" \
      "result=$([ "$code" != "$baseline_exit" ] || [ "$verdict" != "$baseline_line" ] && echo caught || echo not_red)"
  done < "$TABLE"
}
step known-bad-recopy-mutations recopy_mutations > "$PB-mutations.out"

compare() {  # <产物> <冻结参照> <今天那一边的前缀> [额外参数...]：Q5 比对器
  local output="$1" freeze="$2" today="$3"; shift 3
  (cd "$V487" && python3 comparator_q5_same_shape.py \
    --pair "Z1-I2:states:judged:e170_control:$today-z1.out:$S2-v2-i2.out" \
    --pair "Z1-I1:states:attached:e170_control:$today-z1.out:$S2-v2-i1.out" \
    --pair "Z3-I3:images:judged:rows:$today-z3-images.out:$S2-v2-i3-images.out" \
    --pair "Z4-I4:images:judged:rows:$today-z4-images.out:$S2-v2-i4-images.out" \
    --pair "Z5-S4B:states:judged:e171_control:$today-z5.out:$S2-v2-e171-s4-baseline.out" \
    --today-names "Z1-I2:$Z1" --today-names "Z1-I1:$Z1" --today-names "Z3-I3:$Z3" --today-names "Z4-I4:$Z4:$ZKB" --today-names "Z5-S4B:$Z5" \
    --reference-names "Z3-I3:$I2" --reference-names "Z4-I4:$I4:$IKB" \
    --new-gaps "$NEW_GAPS" --e171-tree Z4-I4,Z5-S4B --expected-freeze "$freeze" --output "$output" "$@")
}
step comparator compare "$P-comparator.out" "$SCRATCH/v2-487-freeze-reference.out" "$P-v2-487" \
  --record-paths "$P-record-paths.out" --anchors-b15 Z3-I3,Z4-I4 --b17 "$PB.out:$ZKB" --positive-control-row-input Z3-I3 --scratch "$SCRATCH"
step pc-comparator compare "$P-pc-comparator.out" "$V2_FREEZE" "$P-pc-v2"

freeze_check v2_487 "$V487" "$S1B" - end >> "$STATUS"
freeze_check v1_487 "$V1487" "$S1B" v1_487 end >> "$STATUS"
freeze_check v2 "$V2" "$V2_FREEZE" - end >> "$STATUS"
echo "name=complete failed_steps=$(grep -v 'step=known-bad-recopy-pc3 ' "$STATUS" | grep -c 'exit=[1-9]') expected_nonzero_step=known-bad-recopy-pc3" >> "$STATUS"
