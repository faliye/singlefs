#!/usr/bin/env bash
# admission: always 整个脚本每次都起：哪一步已有判定、键对不对得上由 prune-server 里的按步判定现答（scripts/experiment-apparatus/long_run_item_verdicts.py）
# run-condition: command python3 sha256sum git
# E170 第 2 次重跑第二段 Q5 那一半（`research/prompts/e170-r2-prereg.md` 12.8 分段表「第二段」、修订 2 的 12.13–12.15）：
# Z 族输入核对（含第一段乙没核的 impl-verdicts.tsv）、v2-487 判 Z1 批 1–3 / Z3 / Z4 / Z5 批 1–3、Q5-pc（v2 原样判 Z1 / Z5 批 1–2、Z3、Z4）、
# Q5-e 第一项、Q5-d 与 Q4-fmt-487、照抄核对（Q5-i、Q5-pc3、XC8 / XC9）、Q5 比对器（含 B4-487、B15、B17、Q5-pc2）两趟。全是 Python，不编 Rust。
# 跑法（在仓根，整条命令交 task-server，内存额度与线程由服务定）：
#   task-server/task submit -- \
#     bash scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/run_segment_two_q5.sh <日期标签> <工作进程数> <空的草稿目录> [产物目录]
#   bash scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/run_segment_two_q5.sh --selftest   自证（下面「自证」）
# 产物：research/results/e170-r2-o3-spec-executor-<日期标签>-s2-q5-*.out 与 -s2b-known-bad-recopy*.out，已存在就不覆盖（noclobber；Python 一侧用 open(…, "x")）。
# 每一步的退出码记进 -s2-q5-run-status.out；一步失败不吞，照记、接着跑后面不依赖它的步。
# 按步判定（.claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」）：每一步一条判定，在 prune-server 的 apparatus-items 族、种类
#   e170_o3_spec_executor_with_witness_predicates_journal_header_487（long_run_item_verdicts.py 文件头「判定库」）。键 = 这一步起的 Python 程序的单元闭包（执行器、坏镜像判法、
#   路径计数、格式常量、照抄核对、Q5 比对器；admission.py 的 compute_unit_key 算）、解释器版本、这一步的参数（标签、批、新记的欠一句这类；路径不进）、
#   它读的导出目录与入库产物、快照与工作区里那几份源码的内容（@files、@input），读的上一步产物经 @after 进那一步的键；
#   照抄前的装置（git show 那一次提交的）、Q5-pc3 的复制件与判镜像那一份冻结参照在开跑前照旧现做，经 @input 按内容进键；
#   复制件与变异两步的做法写在本脚本里，本脚本整份经 @input 进那两步的键。输入核对逐字节可比（compare=exact），别的步带计时与进程号（compare=timing）。
#   绿 = 这一步的退出码是它该退的那一个（Q5-pc3 那一步是 3，别的是 0）。键相同、库里判绿的步把库里那一份产物写到这一趟的产物名上、不起程序，
#   -s2-q5-run-status.out 那一行带 reused=true；判红的照同一把键记红撤绿、下一趟现判。末尾报「现判 N、复用 M、拒算 R」（LONG_RUN_ITEMS 那一行）。
#   退出码：0 跑完（步的成败记在 -s2-q5-run-status.out）；1 复用哨兵红或键在跑的中途变了；75 prune-server 不可用；2 用法错。
# 自证（--selftest）：经 long_run_item_verdicts.py 的复用格跑那六格（reuse-three-questions、key-independent-of-directory、reuse-sentinel 与哨兵的三种红）；样板树里 v2-487 与 v2 的
#   executor_main.py 是假执行器（照 --output 写两行），别的程序不在（那几步拒算、照跑照失败）；v2-487-z1、pc-v2-z1 两步的入口换成假树里两个 Python 函数（它文件头「复用格」）。
#   只在临时 prune-server 上跑（long_run_item_verdicts.py selftest-service，即客户端 selftest_service_problem；不在就退 2），起法：
#     python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/run_segment_two_q5.sh --selftest
# 弄坏开关 E170_SEGMENT_TWO_Q5_BREAK=<项，逗号分隔>（只在 --selftest 起的那几趟里认：自证设 E170_SEGMENT_TWO_Q5_SELFTEST_MARK 指到它建的标记文件，别处设了退 2）：
#   E170_SEGMENT_TWO_Q5_BREAK=reuse-ignored  库里判绿的步也照起程序现判（不认 plan 答的 reuse）
#       「reuse-three-questions」「key-independent-of-directory」「reuse-sentinel」「forced-still-reused-red」
#   E170_SEGMENT_TWO_Q5_BREAK=finish-ignored  不认 long_run_item_verdicts.py finish 的退出码（复用哨兵红、键中途变了照退 0）
#       「sentinel-rejudged-red」「forced-verdict-differs-red」「forced-still-reused-red」
# 复用那几格另认 long_run_item_verdicts.py 的弄坏开关 LONG_RUN_ITEMS_BREAK（它文件头；打开后本脚本的自证红在它点名的同名格）。
set -u -o pipefail -o noclobber
source "$(dirname "${BASH_SOURCE[0]}")/../../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
export PYTHONDONTWRITEBYTECODE=1
SCRIPT_KEY="scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/run_segment_two_q5.sh"
ITEM_VERDICTS="scripts/experiment-apparatus/long_run_item_verdicts.py"   # 按步判定的读写（文件头「按步判定」）
LIBRARY_KIND="e170_o3_spec_executor_with_witness_predicates_journal_header_487"   # 判定库的种类：装置（执行器那一版）的目录名
V2_RELATIVE="scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates"
V487_RELATIVE="scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487"
V1487_RELATIVE="scripts/experiment-apparatus/e170_second_run_o3_spec_executor_journal_header_487"

# 弄坏开关只在自证起的那几趟里认（标记文件在、内容对得上）
e170_break_is() {
  [[ ",${E170_SEGMENT_TWO_Q5_BREAK:-}," == *",$1,"* && -n "${E170_SEGMENT_TWO_Q5_SELFTEST_MARK:-}" ]] \
    && [[ "$(cat -- "$E170_SEGMENT_TWO_Q5_SELFTEST_MARK" 2> /dev/null)" == e170-segment-two-q5-selftest ]]
}
if [[ -n "${E170_SEGMENT_TWO_Q5_BREAK:-}" && "${1:-}" != --selftest ]] \
    && [[ -z "${E170_SEGMENT_TWO_Q5_SELFTEST_MARK:-}" || "$(cat -- "${E170_SEGMENT_TWO_Q5_SELFTEST_MARK}" 2> /dev/null)" != e170-segment-two-q5-selftest ]]; then
  echo "  ✗ E170_SEGMENT_TWO_Q5_BREAK 只给 --selftest 证红用，这一趟不是自证起的" >&2
  echo "  → 怎么办：平常跑时不设它" >&2
  exit 2
fi

# 自证：样板树里是本脚本的拷贝、v2-487 与 v2 的假执行器、指回真仓的 .claude 与 long_run_item_verdicts.py；复用那六格经 long_run_item_verdicts.py reuse-cells 跑。
selftest() {
  local work reuse_output reuse_red
  python3 "$ROOT/$ITEM_VERDICTS" selftest-service || return 2
  work=$(mktemp -d) || return 1
  trap 'rm -rf -- "$work"' RETURN
  echo e170-segment-two-q5-selftest > "$work/mark"
  mkdir -p -- "$work/template/$V487_RELATIVE" "$work/template/$V2_RELATIVE"
  cp -- "$SCRIPT_PATH" "$work/template/$SCRIPT_KEY"
  ln -s -- "$ROOT/$ITEM_VERDICTS" "$work/template/$ITEM_VERDICTS"
  ln -s -- "$ROOT/.claude" "$work/template/.claude"
  cat > "$work/template/$V487_RELATIVE/executor_main.py" <<'FAKE'
"""复用格的假执行器：照 --output 写两行；freeze_lines 交一行（本脚本的 freeze_check 照它算）。"""
import sys


def freeze_lines(emit):
    emit("name=freeze_seed sha256_prefix16=selftest")


if __name__ == "__main__":
    with open(sys.argv[sys.argv.index("--output") + 1], "x", encoding="utf-8") as handle:
        handle.write("name=selftest_executor\nname=complete\n")
FAKE
  cp -- "$work/template/$V487_RELATIVE/executor_main.py" "$work/template/$V2_RELATIVE/executor_main.py"
  # 一趟一个新标签、新的草稿目录与产物目录（产物已在就不覆盖）：经 bash 拿进程号拼
  reuse_output=$(python3 "$ROOT/$ITEM_VERDICTS" reuse-cells --script "$SCRIPT_KEY" --template "$work/template" --items v2-487-z1,pc-v2-z1 --work "$work/reuse" --green-exit 0 -- \
    env E170_SEGMENT_TWO_Q5_SELFTEST_MARK="$work/mark" GIT_CEILING_DIRECTORIES="$work" bash -c 'mkdir -p "$2/scratch-$$" "$2/results-$$" && exec bash "$1" "selftest$$" 1 "$2/scratch-$$" "$2/results-$$"' \
    run-segment-two-q5-selftest "{tree}/$SCRIPT_KEY" "{tree}/.." 2>&1)
  echo "$reuse_output"
  reuse_red=$(sed -n 's/^long_run_item_verdicts 复用格.*：6 格，红 \([0-9]*\) 格$/\1/p' <<< "$reuse_output")
  if [[ -z "$reuse_red" ]]; then echo "  ✗ 复用那六格没跑完（没有「long_run_item_verdicts 复用格」那一行），按六格红算"; echo "     → 怎么办：看上面 reuse-cells 打的输出里第一处 ✗ 或 Python 报错（复用格起不来多半是 prune-server 不可用或 long_run_item_verdicts.py 抛了异常），修好之后在临时 prune-server 底下单独起本脚本的 --selftest（文件头「用法」那一条）到它退 0"; reuse_red=6; fi
  echo "run_segment_two_q5.sh 自证：6 格，红 $reuse_red 格"
  ((reuse_red == 0))
}
if [[ "${1:-}" == --selftest ]]; then
  selftest
  exit $?
fi

TAG="${1:?日期标签}"
WORKERS="${2:?工作进程数}"
SCRATCH="${3:?草稿目录}"
V2="$ROOT/$V2_RELATIVE"
V487="$ROOT/$V487_RELATIVE"
V1487="$ROOT/$V1487_RELATIVE"
RESULTS_DIRECTORY="${4:-$ROOT/research/results}"  # 第 4 个参数只给试跑用（指到草稿目录）；正式那一趟不给
P="$RESULTS_DIRECTORY/e170-r2-o3-spec-executor-$TAG-s2-q5"
PB="$RESULTS_DIRECTORY/e170-r2-o3-spec-executor-$TAG-s2b-known-bad-recopy"
S1B="$ROOT/research/results/e170-r2-o3-spec-executor-2026-10-04-s1b-executor.out"
V2_FREEZE="$ROOT/research/results/e170-r2-o3-spec-executor-2026-10-04-s1-executor.out"
EXPORT_PRODUCT="$ROOT/research/results/e170-r2-o3-spec-executor-2026-10-04-s2b-today-export-r3.out"
S2="$ROOT/research/results/e170-r2-o3-spec-executor-2026-10-04-s2"  # 第二段 Q1–Q3 那一趟：v2 在 311 冻结导出上的判定（Q5 的参照）
TABLE="$ROOT/scripts/mutation/tables/e170_o3_spec_executor_with_witness_predicates_journal_header_487.tsv"
Z="$ROOT/scripts/experiment-apparatus/target/e170-r2-today-dumps"
Z1="$Z/e170-dump"; Z1R="$Z/e170-dump-repeat"; Z3="$Z/e170-bad-states-second-run"; Z4="$Z/e170-bad-states-copy-s2"; ZKB="$Z/known-bad-s2"; Z5="$Z/e171-baseline-run1"
I2="$ROOT/scripts/experiment-apparatus/target/e170-second-run-dumps"; I4="$ROOT/scripts/experiment-apparatus/target/e171-dumps/e170-bad-states-copy-s2"; IKB="$ROOT/scripts/experiment-apparatus/target/e171-dumps/known-bad-s2"
TREE="$ROOT/scripts/experiment-apparatus/target/e170-r2-today-tree"
DEVICE="crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs"; TEST_FILE="crates/singlefs-harness/tests/checker_known_bad_images.rs"
IMAGE_RS="crates/singlefs-checker/src/image.rs"; BEFORE_COMMIT=13ca0a19
FORMAT_LIBRARY_FILE="crates/singlefs-format/src/lib.rs"; FORMAT_JOURNAL_FILE="crates/singlefs-core/src/journal.rs"   # comparator_q5_format_constants.py 读的那几份
FORMAT_DECISION_FILE=".claude/kb/decisions/23-journal的角色与格式.md"; FORMAT_LAYOUT_FILE=".claude/kb/layout/01-first-txn.md"
STREAMS="new_pool_file_creation fixed_script_through_e_then_unmount"
STATUS="$P-run-status.out"

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

# 开跑前现做的三样（它们的内容经 @input 进键，所以 plan 之前做）：判镜像那一份冻结参照、照抄前的装置、Q5-pc3 的复制件；另取格式补丁新记的欠一句
grep -E '^name=freeze(_seed)? ' "$S1B" | grep -v ' arm=' > "$SCRATCH/v2-487-freeze-reference.out"  # 判镜像那一份只认不带 arm= 的一组
NEW_GAPS="$(comm -13 <(cut -f1 "$V2/spec-gaps.tsv" | sort) <(cut -f1 "$V487/spec-gaps.tsv" | sort) | paste -sd, -)"  # 格式补丁新记的欠一句
git -C "$ROOT" show "$BEFORE_COMMIT:$DEVICE" > "$SCRATCH/e171-before-$BEFORE_COMMIT.rs"
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
pc3_copy > "$SCRATCH/pc3-copy.out"; PC3_COPY_EXIT=$?

# ── 按步判定的请求（long_run_item_verdicts.py 文件头「请求文件」）──
frozen_file_requests() { # <项> <执行器目录>：那个目录里进冻结的每个文件（取法同 executor_main.freeze_lines：除 comparator*、mutation_driver*、witness_reference*、run_* 与 .pyc）
  local file
  while IFS= read -r file; do printf '@files\t%s\t%s\n' "$1" "$file"; done < <(find "$2" -maxdepth 1 -type f ! -name 'comparator*' ! -name 'mutation_driver*' \
    ! -name 'witness_reference*' ! -name 'run_*' ! -name '*.pyc' 2> /dev/null | sort)
}
executor_request() { # <项> <版本目录（相对仓根）> <导出目录> <批>
  printf '%s\t%s/executor_main.py\tline:program=executor\tline:batches=%s\tline:compare=timing\n@files\t%s\t%s\n' "$1" "$2" "$4" "$1" "$3"
}
images_request() { # <项> <版本目录（相对仓根）> <冻结参照> <输入名> <导出目录> [已知坏镜像目录]
  printf '%s\t%s/comparator_bad_image_judge.py\tline:program=bad-image-judge\tline:input-label=%s\tline:known-bad=%s\tline:compare=timing\n' \
    "$1" "$V2_RELATIVE" "$4" "$( (($# >= 6)) && echo yes || echo no)"
  printf '%s\t%s/executor_main.py\n@files\t%s\t%s\n@input\t%s\tfreeze-reference\t%s\n' "$1" "$2" "$1" "$5" "$1" "$3"
  if (($# >= 6)); then printf '@files\t%s\t%s\n' "$1" "$6"; fi
}
recopy_request() { # <项> <照抄后的装置>：run_known_bad_recopy_check.py 一组五份
  printf '%s\t%s/run_known_bad_recopy_check.py::known_bad_recopy_report\tline:program=known-bad-recopy\tline:compare=timing\n' "$1" "$V487_RELATIVE"
  printf '@input\t%s\told-device\t%s\n@input\t%s\tnew-device\t%s\n@input\t%s\ttest-file\t%s\n@input\t%s\ti4-known-bad-o2\t%s\n@input\t%s\timage-rs\t%s\n' \
    "$1" "$SCRATCH/e171-before-$BEFORE_COMMIT.rs" "$1" "$2" "$1" "$ROOT/$TEST_FILE" "$1" "$IKB/known-bad-o2.tsv" "$1" "$ROOT/$IMAGE_RS"
}
comparator_request() { # <项> <冻结参照> <今天那一边的四步…>：Q5 比对器
  local name="$1" freeze="$2" upstream directory
  shift 2
  printf '%s\t%s/comparator_q5_same_shape.py\tline:program=q5-same-shape\tline:new-gaps=%s\tline:compare=timing\n@input\t%s\texpected-freeze\t%s\n' \
    "$name" "$V487_RELATIVE" "${NEW_GAPS:-none}" "$name" "$freeze"
  for upstream in "$@" record-paths known-bad-recopy; do printf '@after\t%s\t%s\n' "$name" "$upstream"; done
  for directory in "$Z1" "$Z3" "$Z4" "$ZKB" "$Z5" "$I2" "$I4" "$IKB"; do printf '@files\t%s\t%s\n' "$name" "$directory"; done
  printf '@input\t%s\ts2-i2\t%s\n@input\t%s\ts2-i1\t%s\n@input\t%s\ts2-i3\t%s\n@input\t%s\ts2-i4\t%s\n@input\t%s\ts2-s4b\t%s\n' \
    "$name" "$S2-v2-i2.out" "$name" "$S2-v2-i1.out" "$name" "$S2-v2-i3-images.out" "$name" "$S2-v2-i4-images.out" "$name" "$S2-v2-e171-s4-baseline.out"
}
segment_requests() {
  local file pair
  printf 'inputs\t%s/crc32c_castagnoli.py\tline:program=inputs\tline:compare=exact\n' "$V487_RELATIVE"
  printf 'inputs\t%s/executor_main.py\ninputs\t%s/executor_main.py\ninputs\t%s/executor_main.py\n' "$V487_RELATIVE" "$V1487_RELATIVE" "$V2_RELATIVE"
  for file in "$Z1" "$Z1R" "$Z3" "$Z5" "$Z4" "$ZKB"; do printf '@files\tinputs\t%s\n' "$file"; done
  printf '@input\tinputs\texport-product\t%s\n@input\tinputs\ts1b\t%s\n@input\tinputs\tv2-freeze\t%s\n' "$EXPORT_PRODUCT" "$S1B" "$V2_FREEZE"
  for file in "$DEVICE" "$TEST_FILE" "$IMAGE_RS"; do printf '@input\tinputs\tsnapshot-%s\t%s\n@input\tinputs\tworking-%s\t%s\n' "${file##*/}" "$TREE/$file" "${file##*/}" "$ROOT/$file"; done
  frozen_file_requests inputs "$V487"; frozen_file_requests inputs "$V1487"; frozen_file_requests inputs "$V2"
  executor_request v2-487-z1 "$V487_RELATIVE" "$Z1" 1,2,3
  executor_request v2-487-z5 "$V487_RELATIVE" "$Z5" 1,2,3
  images_request v2-487-z3-images "$V487_RELATIVE" "$SCRATCH/v2-487-freeze-reference.out" Z3 "$Z3"
  images_request v2-487-z4-images "$V487_RELATIVE" "$SCRATCH/v2-487-freeze-reference.out" Z4 "$Z4" "$ZKB"
  executor_request pc-v2-z1 "$V2_RELATIVE" "$Z1" 1,2
  executor_request pc-v2-z5 "$V2_RELATIVE" "$Z5" 1,2
  images_request pc-v2-z3-images "$V2_RELATIVE" "$V2_FREEZE" Z3 "$Z3"
  images_request pc-v2-z4-images "$V2_RELATIVE" "$V2_FREEZE" Z4 "$Z4" "$ZKB"
  printf 'record-paths\t%s/run_q5_record_path_counts.py::spec_lines\tline:program=record-paths\tline:executor=%s\tline:compare=timing\n' "$V487_RELATIVE" "${V487##*/}"
  for file in "$Z1" "$Z5" "$Z3" "$Z4" "$ZKB"; do printf '@files\trecord-paths\t%s\n' "$file"; done
  printf 'format-constants\t%s/comparator_q5_format_constants.py\tline:program=format-constants\tline:compare=timing\n' "$V487_RELATIVE"
  for file in "$FORMAT_LIBRARY_FILE" "$FORMAT_JOURNAL_FILE"; do printf '@input\tformat-constants\tsnapshot-%s\t%s\n@input\tformat-constants\tworking-%s\t%s\n' "${file##*/}" "$TREE/$file" "${file##*/}" "$ROOT/$file"; done
  printf '@input\tformat-constants\tdecision\t%s\n@input\tformat-constants\tlayout\t%s\n' "$ROOT/$FORMAT_DECISION_FILE" "$ROOT/$FORMAT_LAYOUT_FILE"
  frozen_file_requests format-constants "$V487"
  recopy_request known-bad-recopy "$ROOT/$DEVICE"
  recopy_request known-bad-recopy-pc3 "$SCRATCH/pc3-device.rs"
  printf '@input\tknown-bad-recopy-pc3\tpc3-copy-lines\t%s\n@input\tknown-bad-recopy-pc3\tthis-script\t%s\n' "$SCRATCH/pc3-copy.out" "$ROOT/$SCRIPT_KEY"
  printf 'known-bad-recopy-mutations\t%s/run_known_bad_recopy_check.py::known_bad_recopy_report\tline:program=known-bad-recopy-mutations\tline:compare=timing\n' "$V487_RELATIVE"
  printf '@after\tknown-bad-recopy-mutations\tknown-bad-recopy\n@after\tknown-bad-recopy-mutations\tknown-bad-recopy-pc3\n'
  printf '@input\tknown-bad-recopy-mutations\ttable\t%s\n@input\tknown-bad-recopy-mutations\trecopy-check-text\t%s/run_known_bad_recopy_check.py\n' "$TABLE" "$V487"
  printf '@input\tknown-bad-recopy-mutations\tthis-script\t%s\n@input\tknown-bad-recopy-mutations\tpc3-device\t%s\n' "$ROOT/$SCRIPT_KEY" "$SCRATCH/pc3-device.rs"
  for pair in old-device:"$SCRATCH/e171-before-$BEFORE_COMMIT.rs" new-device:"$ROOT/$DEVICE" test-file:"$ROOT/$TEST_FILE" i4-known-bad-o2:"$IKB/known-bad-o2.tsv" image-rs:"$ROOT/$IMAGE_RS"; do
    printf '@input\tknown-bad-recopy-mutations\t%s\t%s\n' "${pair%%:*}" "${pair#*:}"
  done
  comparator_request comparator "$SCRATCH/v2-487-freeze-reference.out" v2-487-z1 v2-487-z3-images v2-487-z4-images v2-487-z5
  comparator_request pc-comparator "$V2_FREEZE" pc-v2-z1 pc-v2-z3-images pc-v2-z4-images pc-v2-z5
}

library_failure_exit() { # <哪一步> <退出码>：75（prune-server 不可用）与 2 照交，别的按 1 交
  echo "  ✗ 按步判定的 $1 没成（long_run_item_verdicts.py 退 $2）" >&2
  echo "  → 怎么办：退 75 是 prune-server 不可用：python3 prune-server/control check 看服务，修好之后重交同一条命令（报给主 agent）；别的照上面它打的出路改" >&2
  if (($2 == 75 || $2 == 2)); then exit "$2"; fi
  exit 1
}

STATE="$(mktemp -d "$SCRATCH/items-XXXXXX")" || exit 1
trap 'rm -rf -- "$STATE"' EXIT
segment_requests > "$STATE/requests.tsv"
python3 "$ROOT/$ITEM_VERDICTS" plan "$STATE/plan" --script "$SCRIPT_KEY" --kind "$LIBRARY_KIND" --root "$ROOT" --requests "$STATE/requests.tsv" \
  || library_failure_exit plan "$?"
decision_of() { awk -F'\t' -v item="$1" '$1 == item {print $2}' "$STATE/plan/decisions.tsv"; }

: > "$STATUS"
echo "name=new_gaps identifiers=${NEW_GAPS:-none}" >> "$STATUS"
# 一步（文件头「按步判定」）：库里判绿就把那一份写到产物名上、不起命令；不然起命令、记判定（退出码不是 <绿的退出码> 记红）
step_expecting() { # <绿的退出码> <项> <产物> <命令…>
  local green="$1" name="$2" output="$3" code red=()
  shift 3
  if [[ "$(decision_of "$name")" == reuse ]] && ! e170_break_is reuse-ignored; then
    if [[ -e "$output" ]]; then
      echo "  ✗ 步 $name 复用库里那一份，可产物 $output 已在，不覆盖" >&2
      echo "  → 怎么办：产物名带日期标签、已在的不覆盖：换一个新的日期标签再起同一条命令（判绿的步仍复用库里那一份，写到新标签的产物名上）；旧产物不动" >&2
      echo "name=step step=$name exit=2 reused=true" >> "$STATUS"
      return 0
    fi
    cp -- "$STATE/plan/outputs/$name.out" "$output"
    echo "name=step step=$name exit=$(cat -- "$STATE/plan/outputs/$name.exit") reused=true" >> "$STATUS"
    return 0
  fi
  "$@"; code=$?
  echo "name=step step=$name exit=$code" >> "$STATUS"
  ((code == green)) || red=(--red)
  python3 "$ROOT/$ITEM_VERDICTS" record "$STATE/plan" "$name" "$code" "$output" ${red[@]+"${red[@]}"} > /dev/null \
    || library_failure_exit "record（步 $name）" "$?"
  return 0
}
step() { # <项> <产物> <命令…>
  step_expecting 0 "$@"
}
capture() { # <产物> <命令…>：命令的标准输出就是这一步的产物
  local output="$1"
  shift
  "$@" > "$output"
}

step inputs "$P-inputs.out" capture "$P-inputs.out" inputs

run_executor() {  # <版本：v487|v2> <导出目录> <批> <产物>；版本按名字取 V487 / V2，执行器在那一版的目录里起
  local version_directory
  case "$1" in
    v487) version_directory="$V487" ;;
    v2) version_directory="$V2" ;;
    *) echo "  ✗ run_executor：认不出的版本 $1" >&2; echo "  → 怎么办：写 v487 或 v2" >&2; return 2 ;;
  esac
  (cd "$version_directory" && python3 executor_main.py --dumps "$2" --batches "$3" --workers "$WORKERS" --output "$4")
}
judge_images() {  # <执行器目录> <冻结参照> <输入名> <导出目录> <产物> [已知坏镜像目录；不给就是这个输入没有已知坏镜像（Z3）]
  local known=(--known-bad-not-applicable); [ $# -ge 6 ] && known=(--known-bad-dir "$6")
  (cd "$V2" && python3 comparator_bad_image_judge.py --executor-directory "$1" --input-label "$3" --dumps "$4" "${known[@]}" \
     --freeze-reference "$2" --output "$5")
}
step v2-487-z1 "$P-v2-487-z1.out" run_executor v487 "$Z1" 1,2,3 "$P-v2-487-z1.out"
step v2-487-z5 "$P-v2-487-z5.out" run_executor v487 "$Z5" 1,2,3 "$P-v2-487-z5.out"
step v2-487-z3-images "$P-v2-487-z3-images.out" judge_images "$V487" "$SCRATCH/v2-487-freeze-reference.out" Z3 "$Z3" "$P-v2-487-z3-images.out"
step v2-487-z4-images "$P-v2-487-z4-images.out" judge_images "$V487" "$SCRATCH/v2-487-freeze-reference.out" Z4 "$Z4" "$P-v2-487-z4-images.out" "$ZKB"
step pc-v2-z1 "$P-pc-v2-z1.out" run_executor v2 "$Z1" 1,2 "$P-pc-v2-z1.out"
step pc-v2-z5 "$P-pc-v2-z5.out" run_executor v2 "$Z5" 1,2 "$P-pc-v2-z5.out"
step pc-v2-z3-images "$P-pc-v2-z3-images.out" judge_images "$V2" "$V2_FREEZE" Z3 "$Z3" "$P-pc-v2-z3-images.out"
step pc-v2-z4-images "$P-pc-v2-z4-images.out" judge_images "$V2" "$V2_FREEZE" Z4 "$Z4" "$P-pc-v2-z4-images.out" "$ZKB"
step record-paths "$P-record-paths.out" bash -c "cd '$V487' && python3 run_q5_record_path_counts.py --executor-directory '$V487' --states Z1:'$Z1':1,2,3 \
  --states Z5:'$Z5':1,2,3 --images Z3:'$Z3' --images Z4:'$Z4':'$ZKB' --output '$P-record-paths.out'"
step format-constants "$P-format-constants.out" bash -c "cd '$V487' && python3 comparator_q5_format_constants.py --executor-directory '$V487' --tree '$TREE' \
  --working-tree '$ROOT' --output '$P-format-constants.out'"

recopy_check() {  # <照抄后的装置> [变异过的核对脚本]：读法 18、19 的照抄核对；核对脚本缺省是 V487 里的 run_known_bad_recopy_check.py，
  # 变异那一步给它在草稿里改出来的那一份；照抄前的装置取 git 里那一次提交（12.12 第 2 条，开跑前已取出）
  (cd "$ROOT" && python3 "${2:-$V487/run_known_bad_recopy_check.py}" "$SCRATCH/e171-before-$BEFORE_COMMIT.rs" "$1" "$TEST_FILE" "${IKB#"$ROOT"/}/known-bad-o2.tsv" "$IMAGE_RS")
}
step known-bad-recopy "$PB.out" capture "$PB.out" recopy_check "$DEVICE"
pc3_run() { cat -- "$SCRATCH/pc3-copy.out" && ((PC3_COPY_EXIT == 0)) && recopy_check "$SCRATCH/pc3-device.rs"; }
step_expecting 3 known-bad-recopy-pc3 "$PB-pc3.out" capture "$PB-pc3.out" pc3_run  # 期望退 3（复制件上照抄前后不同恰 1），不算失败的步
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
step known-bad-recopy-mutations "$PB-mutations.out" capture "$PB-mutations.out" recopy_mutations

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
step comparator "$P-comparator.out" compare "$P-comparator.out" "$SCRATCH/v2-487-freeze-reference.out" "$P-v2-487" \
  --record-paths "$P-record-paths.out" --anchors-b15 Z3-I3,Z4-I4 --b17 "$PB.out:$ZKB" --positive-control-row-input Z3-I3 --scratch "$SCRATCH"
step pc-comparator "$P-pc-comparator.out" compare "$P-pc-comparator.out" "$V2_FREEZE" "$P-pc-v2" \
  --record-paths "$P-record-paths.out" --anchors-b15 Z3-I3,Z4-I4 --b17 "$PB.out:$ZKB" --positive-control-row-input Z3-I3 --scratch "$SCRATCH"

freeze_check v2_487 "$V487" "$S1B" - end >> "$STATUS"
freeze_check v1_487 "$V1487" "$S1B" v1_487 end >> "$STATUS"
freeze_check v2 "$V2" "$V2_FREEZE" - end >> "$STATUS"
echo "name=complete failed_steps=$(grep -v 'step=known-bad-recopy-pc3 ' "$STATUS" | grep -c 'exit=[1-9]') expected_nonzero_step=known-bad-recopy-pc3" >> "$STATUS"
finish_exit=0
python3 "$ROOT/$ITEM_VERDICTS" finish "$STATE/plan" > "$STATE/finish.out" || finish_exit=$?
cat -- "$STATE/finish.out"
((finish_exit <= 1)) || library_failure_exit finish "$finish_exit"
! e170_break_is finish-ignored || finish_exit=0
((finish_exit == 0)) || { echo "  ✗ 按步判定收尾判红（复用哨兵或键在跑的中途变了，见上面 ✗ 那几行）" >&2; echo "  → 怎么办：照上面那几行的出路办，再起同一条命令（新的日期标签）" >&2; exit 1; }
exit 0
