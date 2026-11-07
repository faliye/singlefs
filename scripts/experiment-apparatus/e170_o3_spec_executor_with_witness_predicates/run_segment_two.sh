#!/usr/bin/env bash
# admission: always 整个脚本每次都起：哪一步已有判定、键对不对得上由 prune-server 里的按步判定现答（scripts/experiment-apparatus/long_run_item_verdicts.py）
# run-condition: command python3 sha256sum
# E170 第 2 次重跑第二段（`research/prompts/e170-r2-prereg.md` 5.3、5.4、5.8 第二段那一行）：输入核对、v1 现跑与入库产物比、
# v2 判 I1–I4 / s4 三行 / 几何点、I5、参照谓词 W_ref 与 T_ref、X21 / X27 在 s4 行上各一趟、两份比对器。全是 Python，不编 Rust。
# 跑法（在仓根，整条命令交 task-server，内存额度与线程由服务定）：
#   task-server/task submit -- \
#     bash scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/run_segment_two.sh <日期标签> <工作进程数> <草稿目录> [产物目录]
#   bash scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/run_segment_two.sh --selftest   自证（下面「自证」）
# 产物：research/results/e170-r2-o3-spec-executor-<日期标签>-s2-*.out，已存在就不覆盖（noclobber，executor_main 自己也拒绝）。
# 每一步的退出码记进 -s2-run-status.out；一步失败不吞，照记、接着跑后面不依赖它的步（依赖它的那一步会因输入缺失自己失败、同样记下）。
# 按步判定（.claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」）：每一步一条判定，在 prune-server 的 apparatus-items 族、种类
#   e170_o3_spec_executor_with_witness_predicates（long_run_item_verdicts.py 文件头「判定库」）。键 = 这一步起的 Python 程序的单元闭包（执行器、比对器、参照谓词、变异驱动；
#   admission.py 的 compute_unit_key 算）、解释器版本、这一步的参数（标签、批、变异名这类；路径不进）、它读的导出目录与入库产物的内容（@files、@input）、
#   读的上一步产物经 @after 进那一步的键（上一步的键）；输入核对那一步另进 v1、v2 冻结的每个文件。输入核对逐字节可比（compare=exact），别的步带计时与进程号（compare=timing）。
#   键相同、库里判绿的步把库里那一份产物写到这一趟的产物名上、不起程序，-s2-run-status.out 那一行带 reused=true；判红（退非 0）的照同一把键记红撤绿、下一趟现判。
#   末尾报「现判 N、复用 M、拒算 R」（LONG_RUN_ITEMS 那一行）。退出码：0 跑完（步的成败记在 -s2-run-status.out）；1 复用哨兵红或键在跑的中途变了；75 prune-server 不可用；2 用法错。
# 自证（--selftest）：经 long_run_item_verdicts.py 的复用格跑那六格（reuse-three-questions、key-independent-of-directory、reuse-sentinel 与哨兵的三种红）；样板树里 v1、v2 的
#   executor_main.py 是假执行器（照 --output 写两行），别的程序不在（那几步拒算、照跑照失败）；v1-i1、v2-i1 两步的入口换成假树里两个 Python 函数（它文件头「复用格」）。
#   只在临时 prune-server 上跑（long_run_item_verdicts.py selftest-service，即客户端 selftest_service_problem；不在就退 2），起法：
#     python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/run_segment_two.sh --selftest
# 弄坏开关 E170_SEGMENT_TWO_BREAK=<项，逗号分隔>（只在 --selftest 起的那几趟里认：自证设 E170_SEGMENT_TWO_SELFTEST_MARK 指到它建的标记文件，别处设了退 2）：
#   E170_SEGMENT_TWO_BREAK=reuse-ignored  库里判绿的步也照起程序现判（不认 plan 答的 reuse）
#       「reuse-three-questions」「key-independent-of-directory」「reuse-sentinel」「forced-still-reused-red」
#   E170_SEGMENT_TWO_BREAK=finish-ignored  不认 long_run_item_verdicts.py finish 的退出码（复用哨兵红、键中途变了照退 0）
#       「sentinel-rejudged-red」「forced-verdict-differs-red」「forced-still-reused-red」
# 复用那几格另认 long_run_item_verdicts.py 的弄坏开关 LONG_RUN_ITEMS_BREAK（它文件头；打开后本脚本的自证红在它点名的同名格）。
set -u -o pipefail -o noclobber
source "$(dirname "${BASH_SOURCE[0]}")/../../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
export PYTHONDONTWRITEBYTECODE=1
SCRIPT_KEY="scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/run_segment_two.sh"
ITEM_VERDICTS="scripts/experiment-apparatus/long_run_item_verdicts.py"   # 按步判定的读写（文件头「按步判定」）
LIBRARY_KIND="e170_o3_spec_executor_with_witness_predicates"            # 判定库的种类：装置（执行器那一版）的目录名
V1_RELATIVE="scripts/experiment-apparatus/e170_second_run_o3_spec_executor"
V2_RELATIVE="scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates"

# 弄坏开关只在自证起的那几趟里认（标记文件在、内容对得上）
e170_break_is() {
  [[ ",${E170_SEGMENT_TWO_BREAK:-}," == *",$1,"* && -n "${E170_SEGMENT_TWO_SELFTEST_MARK:-}" ]] \
    && [[ "$(cat -- "$E170_SEGMENT_TWO_SELFTEST_MARK" 2> /dev/null)" == e170-segment-two-selftest ]]
}
if [[ -n "${E170_SEGMENT_TWO_BREAK:-}" && "${1:-}" != --selftest ]] \
    && [[ -z "${E170_SEGMENT_TWO_SELFTEST_MARK:-}" || "$(cat -- "${E170_SEGMENT_TWO_SELFTEST_MARK}" 2> /dev/null)" != e170-segment-two-selftest ]]; then
  echo "  ✗ E170_SEGMENT_TWO_BREAK 只给 --selftest 证红用，这一趟不是自证起的" >&2
  echo "  → 怎么办：平常跑时不设它" >&2
  exit 2
fi

# 自证：样板树里是本脚本的拷贝、v1 与 v2 的假执行器、指回真仓的 .claude 与 long_run_item_verdicts.py；复用那六格经 long_run_item_verdicts.py reuse-cells 跑。
selftest() {
  local work reuse_output reuse_red
  python3 "$ROOT/$ITEM_VERDICTS" selftest-service || return 2
  work=$(mktemp -d) || return 1
  trap 'rm -rf -- "$work"' RETURN
  echo e170-segment-two-selftest > "$work/mark"
  mkdir -p -- "$work/template/$V1_RELATIVE" "$work/template/$V2_RELATIVE"
  cp -- "$SCRIPT_PATH" "$work/template/$SCRIPT_KEY"
  ln -s -- "$ROOT/$ITEM_VERDICTS" "$work/template/$ITEM_VERDICTS"
  ln -s -- "$ROOT/.claude" "$work/template/.claude"
  cat > "$work/template/$V1_RELATIVE/executor_main.py" <<'FAKE'
"""复用格的假执行器：照 --output 写两行；freeze_lines 交一行（本脚本的 freeze_check 照它算）。"""
import sys


def freeze_lines(emit):
    emit("name=freeze_seed sha256_prefix16=selftest")


if __name__ == "__main__":
    with open(sys.argv[sys.argv.index("--output") + 1], "x", encoding="utf-8") as handle:
        handle.write("name=selftest_executor\nname=complete\n")
FAKE
  cp -- "$work/template/$V1_RELATIVE/executor_main.py" "$work/template/$V2_RELATIVE/executor_main.py"
  # i5 那两步在样板树里也要跑得绿（它们有键：入口是假执行器）：v1 的单测文件换成一条照过的假单测
  printf '%s\n' '"""复用格的假单测：i5 那两步在样板树里照过。"""' 'import unittest' '' '' 'class SelftestPlaceholder(unittest.TestCase):' \
    '    def test_placeholder_passes(self):' '        self.assertTrue(True)' > "$work/template/$V1_RELATIVE/test_executor_on_synthetic_pool.py"
  # 一趟一个新标签、新的草稿目录与产物目录（产物已在就不覆盖）：经 bash 拿进程号拼
  reuse_output=$(python3 "$ROOT/$ITEM_VERDICTS" reuse-cells --script "$SCRIPT_KEY" --template "$work/template" --items v1-i1,v2-i1 --work "$work/reuse" --green-exit 0 -- \
    env E170_SEGMENT_TWO_SELFTEST_MARK="$work/mark" bash -c 'mkdir -p "$2/scratch-$$" "$2/results-$$" && exec bash "$1" "selftest$$" 1 "$2/scratch-$$" "$2/results-$$"' \
    run-segment-two-selftest "{tree}/$SCRIPT_KEY" "{tree}/.." 2>&1)
  echo "$reuse_output"
  reuse_red=$(sed -n 's/^long_run_item_verdicts 复用格.*：6 格，红 \([0-9]*\) 格$/\1/p' <<< "$reuse_output")
  if [[ -z "$reuse_red" ]]; then echo "  ✗ 复用那六格没跑完（没有「long_run_item_verdicts 复用格」那一行），按六格红算"; echo "     → 怎么办：看上面 reuse-cells 打的输出里第一处 ✗ 或 Python 报错（复用格起不来多半是 prune-server 不可用或 long_run_item_verdicts.py 抛了异常），修好之后在临时 prune-server 底下单独起本脚本的 --selftest（文件头「用法」那一条）到它退 0"; reuse_red=6; fi
  echo "run_segment_two.sh 自证：6 格，红 $reuse_red 格"
  ((reuse_red == 0))
}
if [[ "${1:-}" == --selftest ]]; then
  selftest
  exit $?
fi

TAG="${1:?日期标签}"
WORKERS="${2:?工作进程数}"
SCRATCH="${3:?草稿目录}"
V1="$ROOT/$V1_RELATIVE"
V2="$ROOT/$V2_RELATIVE"
TARGET="$ROOT/scripts/experiment-apparatus/target"
RESULTS_DIRECTORY="${4:-$ROOT/research/results}"  # 第 4 个参数只给试跑用（指到草稿目录）；正式那一趟不给，写进 research/results
P="$RESULTS_DIRECTORY/e170-r2-o3-spec-executor-$TAG-s2"
V1_FREEZE="$ROOT/research/results/e170-r1-o3-spec-executor-2026-10-03-s1-executor.out"
V2_FREEZE="$ROOT/research/results/e170-r2-o3-spec-executor-2026-10-04-s1-executor.out"
TABLE="$ROOT/scripts/mutation/tables/e170_o3_spec_executor_with_witness_predicates.tsv"
I1="$TARGET/e170-dumps"; I2="$TARGET/e170-second-run-dumps"; I4="$TARGET/e171-dumps/e170-bad-states-copy-s2"
KNOWN_BAD="$TARGET/e171-dumps/known-bad-s2"; BASELINE="$TARGET/e171-dumps/baseline-run1"; ROWS="$TARGET/e171-dumps/rows-s4-f4"
STREAMS="new_pool_file_creation fixed_script_through_e_then_unmount"
STATUS="$P-run-status.out"

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
reference_request() { # <项> <states|images> <输入名> <导出目录> <上一步> [已知坏镜像目录]
  printf '%s\t%s/witness_reference.py\tline:program=witness-reference\tline:mode=%s\tline:input-label=%s\tline:compare=timing\n@files\t%s\t%s\n@after\t%s\t%s\n' \
    "$1" "$V2_RELATIVE" "$2" "$3" "$1" "$4" "$1" "$5"
  if (($# >= 6)); then printf '@files\t%s\t%s\n' "$1" "$6"; fi
}
after_requests() { # <项> <上一步>…
  local name="$1" upstream
  shift
  for upstream in "$@"; do printf '@after\t%s\t%s\n' "$name" "$upstream"; done
}
segment_requests() {
  local row file name helper
  printf 'inputs\t%s/crc32c_castagnoli.py\tline:program=inputs\tline:compare=exact\ninputs\t%s/executor_main.py\ninputs\t%s/executor_main.py\n' "$V2_RELATIVE" "$V1_RELATIVE" "$V2_RELATIVE"
  for file in "$I1" "$I2" "$I4" "$BASELINE" "$ROWS/row-242/dump" "$ROWS/row-269/dump" "$ROWS/row-270/dump" "$KNOWN_BAD"; do printf '@files\tinputs\t%s\n' "$file"; done
  printf '@input\tinputs\tv1-freeze\t%s\n@input\tinputs\tv2-freeze\t%s\n' "$V1_FREEZE" "$V2_FREEZE"
  frozen_file_requests inputs "$V1"; frozen_file_requests inputs "$V2"
  executor_request v1-i1 "$V1_RELATIVE" "$I1" 1,2
  executor_request v1-i2 "$V1_RELATIVE" "$I2" 1,2
  for row in 242 269 270; do executor_request "v1-row-$row" "$V1_RELATIVE" "$ROWS/row-$row/dump" 1,2; done
  executor_request v2-i1 "$V2_RELATIVE" "$I1" 1,2,3
  executor_request v2-i2 "$V2_RELATIVE" "$I2" 1,2,3
  for row in 242 269 270; do executor_request "v2-row-$row" "$V2_RELATIVE" "$ROWS/row-$row/dump" 1,2; done
  executor_request v2-e171-s4-baseline "$V2_RELATIVE" "$BASELINE" 1,2,3
  images_request v1-i3-images "$V1_RELATIVE" "$V1_FREEZE" I3 "$I2"
  images_request v1-i4-images "$V1_RELATIVE" "$V1_FREEZE" I4 "$I4" "$KNOWN_BAD"
  images_request v2-i3-images "$V2_RELATIVE" "$V2_FREEZE" I3 "$I2"
  images_request v2-i4-images "$V2_RELATIVE" "$V2_FREEZE" I4 "$I4" "$KNOWN_BAD"
  for helper in v2 v1; do
    name=i5; [[ "$helper" == v1 ]] && name=i5-with-v1-helper
    printf '%s\t%s/executor_main.py\tline:program=v1-unit-tests-on-v2\tline:helper=%s\tline:compare=timing\n@input\t%s\tv1-test-file\t%s/test_executor_on_synthetic_pool.py\n' \
      "$name" "$V2_RELATIVE" "$helper" "$name" "$V1"
    [[ "$helper" == v1 ]] && printf '@input\t%s\tv1-helper\t%s/synthetic_pool_for_tests.py\n' "$name" "$V1"
    while IFS= read -r file; do printf '@files\t%s\t%s\n' "$name" "$file"; done < <(find "$V2" -maxdepth 1 -type f ! -name 'comparator*' ! -name 'mutation_driver*' \
      ! -name 'witness_reference*' ! -name 'run_*' ! -name '*.pyc' 2> /dev/null | sort)
  done
  reference_request reference-i1 states I1 "$I1" v2-i1
  reference_request reference-i2 states I2 "$I2" v2-i2
  for row in 242 269 270; do reference_request "reference-row-$row" states "row-$row" "$ROWS/row-$row/dump" "v2-row-$row"; done
  reference_request reference-e171-s4-baseline states e171-s4-baseline "$BASELINE" v2-e171-s4-baseline
  reference_request reference-i3-images images I3 "$I2" v2-i3-images
  reference_request reference-i4-images images I4 "$I4" v2-i4-images "$KNOWN_BAD"
  for row in 242:X21 269:X21 270:X27; do
    name="$(tr 'A-Z' 'a-z' <<< "${row#*:}")-row-${row%%:*}"
    printf '%s\t%s/mutation_driver_on_rows.py\tline:program=mutation-on-rows\tline:mutation=%s\tline:batches=1,2\tline:compare=timing\n@input\t%s\ttable\t%s\n@files\t%s\t%s\n' \
      "$name" "$V2_RELATIVE" "${row#*:}" "$name" "$TABLE" "$name" "$ROWS/row-${row%%:*}/dump"
  done
  printf 'v1-against-stored\t%s/comparator_v1_against_stored.py\tline:program=v1-against-stored\tline:compare=timing\n' "$V2_RELATIVE"
  after_requests v1-against-stored v1-i1 v1-i2 v1-i3-images v1-i4-images v1-row-242 v1-row-269 v1-row-270
  printf '@input\tv1-against-stored\tstored-i1\t%s\n@input\tv1-against-stored\tstored-i2\t%s\n@input\tv1-against-stored\tstored-i3\t%s\n@input\tv1-against-stored\tstored-i4\t%s\n' \
    "$ROOT/research/results/e170-r1-o3-spec-executor-2026-10-03-s1-executor.out" "$ROOT/research/results/e170-r1-o3-spec-executor-2026-10-03-s2-executor.out" \
    "$ROOT/research/results/e170-r1-o3-spec-executor-2026-10-03-s3-bad-states.out" "$ROOT/research/results/e171-three-oracle-collusion-2026-10-03-s2-images-o3.out"
  for row in 242 269 270; do printf '@input\tv1-against-stored\tstored-row-%s\t%s\n' "$row" "$ROWS/row-$row/o3.out"; done
  printf 'v2-i1-against-segment-one\t%s/comparator_v1_against_stored.py\tline:program=v2-against-segment-one\tline:compare=timing\n@after\tv2-i1-against-segment-one\tv2-i1\n' "$V2_RELATIVE"
  printf '@input\tv2-i1-against-segment-one\tsegment-one\t%s\n' "$V2_FREEZE"
  printf 'comparator-q1\t%s/comparator_two_versions.py\tline:program=comparator-q1\tline:batches=1,2\tline:compare=timing\n' "$V2_RELATIVE"
  after_requests comparator-q1 v1-i1 v2-i1 v1-i2 v2-i2 v1-i3-images v2-i3-images v1-i4-images v2-i4-images i5
  for file in "$I2" "$I4" "$KNOWN_BAD"; do printf '@files\tcomparator-q1\t%s\n' "$file"; done
  printf 'comparator-q2-q3\t%s/comparator_witness_classes.py\tline:program=comparator-q2-q3\tline:compare=timing\n' "$V2_RELATIVE"
  after_requests comparator-q2-q3 v1-row-242 v2-row-242 reference-row-242 v1-row-269 v2-row-269 reference-row-269 v1-row-270 v2-row-270 reference-row-270 \
    x21-row-242 x21-row-269 x27-row-270 v2-i1 reference-i1 v2-i2 reference-i2 v2-i3-images reference-i3-images v2-i4-images reference-i4-images \
    v2-e171-s4-baseline reference-e171-s4-baseline
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
# 一步（文件头「按步判定」）：库里判绿就把那一份写到产物名上、不起命令；不然起命令、记判定（退非 0 记红）。<产物> 是这一步写的那一份
step() { # <项> <产物> <命令…>
  local name="$1" output="$2" code red=()
  shift 2
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
  ((code == 0)) || red=(--red)
  python3 "$ROOT/$ITEM_VERDICTS" record "$STATE/plan" "$name" "$code" "$output" ${red[@]+"${red[@]}"} > /dev/null \
    || library_failure_exit "record（步 $name）" "$?"
  return 0
}
capture() { # <产物> <命令…>：命令的标准输出就是这一步的产物
  local output="$1"
  shift
  "$@" > "$output"
}

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
step inputs "$P-inputs.out" capture "$P-inputs.out" inputs

run_executor() {  # <版本：v1|v2> <导出目录> <批> <产物>；版本按名字取 V1 / V2，执行器在那一版的目录里起
  local version_directory
  case "$1" in
    v1) version_directory="$V1" ;;
    v2) version_directory="$V2" ;;
    *) echo "  ✗ run_executor：认不出的版本 $1" >&2; echo "  → 怎么办：写 v1 或 v2" >&2; return 2 ;;
  esac
  (cd "$version_directory" && python3 executor_main.py --dumps "$2" --batches "$3" --workers "$WORKERS" --output "$4")
}
step v1-i1 "$P-v1-i1.out" run_executor v1 "$I1" 1,2 "$P-v1-i1.out"
step v1-i2 "$P-v1-i2.out" run_executor v1 "$I2" 1,2 "$P-v1-i2.out"
for row in 242 269 270; do step "v1-row-$row" "$P-v1-row-$row.out" run_executor v1 "$ROWS/row-$row/dump" 1,2 "$P-v1-row-$row.out"; done
step v2-i1 "$P-v2-i1.out" run_executor v2 "$I1" 1,2,3 "$P-v2-i1.out"
step v2-i2 "$P-v2-i2.out" run_executor v2 "$I2" 1,2,3 "$P-v2-i2.out"
for row in 242 269 270; do step "v2-row-$row" "$P-v2-row-$row.out" run_executor v2 "$ROWS/row-$row/dump" 1,2 "$P-v2-row-$row.out"; done
step v2-e171-s4-baseline "$P-v2-e171-s4-baseline.out" run_executor v2 "$BASELINE" 1,2,3 "$P-v2-e171-s4-baseline.out"

judge_images() {  # <v 名> <版本目录> <冻结参照> <输入名> <导出目录> [已知坏镜像目录；不给就是这个输入没有已知坏镜像（I3）]
  local known=(--known-bad-not-applicable); [ $# -ge 6 ] && known=(--known-bad-dir "$6")
  (cd "$V2" && python3 comparator_bad_image_judge.py --executor-directory "$2" --input-label "$4" --dumps "$5" "${known[@]}" \
     --freeze-reference "$3" --output "$P-$1-$(tr 'A-Z' 'a-z' <<< "$4")-images.out")
}
step v1-i3-images "$P-v1-i3-images.out" judge_images v1 "$V1" "$V1_FREEZE" I3 "$I2"
step v1-i4-images "$P-v1-i4-images.out" judge_images v1 "$V1" "$V1_FREEZE" I4 "$I4" "$KNOWN_BAD"
step v2-i3-images "$P-v2-i3-images.out" judge_images v2 "$V2" "$V2_FREEZE" I3 "$I2"
step v2-i4-images "$P-v2-i4-images.out" judge_images v2 "$V2" "$V2_FREEZE" I4 "$I4" "$KNOWN_BAD"

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
step i5 "$P-i5-v1-tests-on-v2.out" capture "$P-i5-v1-tests-on-v2.out" unit_tests_of_version_one_on_version_two i5-copy
step i5-with-v1-helper "$P-i5-v1-tests-and-helper-on-v2.out" capture "$P-i5-v1-tests-and-helper-on-v2.out" unit_tests_of_version_one_on_version_two i5-copy-with-v1-helper also_v1_helper

reference() {  # <states|images> <输入名> <导出目录> <v2 产物> <产物名> [已知坏镜像目录；images 不给就是这个输入没有已知坏镜像（I3）]
  local known=(); [ "$1" = images ] && known=(--known-bad-not-applicable); [ $# -ge 6 ] && known=(--known-bad-dir "$6")
  (cd "$V2" && python3 witness_reference.py "$1" --dumps "$3" "${known[@]}" --v2-output "$4" --input-label "$2" --output "$P-reference-$5.out")
}
step reference-i1 "$P-reference-i1.out" reference states I1 "$I1" "$P-v2-i1.out" i1
step reference-i2 "$P-reference-i2.out" reference states I2 "$I2" "$P-v2-i2.out" i2
for row in 242 269 270; do step "reference-row-$row" "$P-reference-row-$row.out" reference states "row-$row" "$ROWS/row-$row/dump" "$P-v2-row-$row.out" "row-$row"; done
step reference-e171-s4-baseline "$P-reference-e171-s4-baseline.out" reference states e171-s4-baseline "$BASELINE" "$P-v2-e171-s4-baseline.out" e171-s4-baseline
step reference-i3-images "$P-reference-i3-images.out" reference images I3 "$I2" "$P-v2-i3-images.out" i3-images
step reference-i4-images "$P-reference-i4-images.out" reference images I4 "$I4" "$P-v2-i4-images.out" i4-images "$KNOWN_BAD"

# 下面 mutated_run 与 comparator-q1 两处显式给 --batches 1,2：登记 research/prompts/e170-r2-prereg.md 5.3 写 I1、I2、第 270 行的态都是「两条流批 1、批 2」
# （第 53、54、67 行），第 631 行写「批 3 在 Q1、Q2 上不判、不跑」，所以登记的范围就是两批，不用缺省的三批。
mutated_run() {  # <变异> <行号>
  (cd "$V2" && python3 mutation_driver_on_rows.py --mutation "$1" --table "$TABLE" --scratch "$SCRATCH" --dumps "$ROWS/row-$2/dump" \
     --batches 1,2 --workers "$WORKERS" --output "$P-$(tr 'A-Z' 'a-z' <<< "$1")-row-$2.out")
}
step x21-row-242 "$P-x21-row-242.out" mutated_run X21 242 >> "$STATUS"
step x21-row-269 "$P-x21-row-269.out" mutated_run X21 269 >> "$STATUS"
step x27-row-270 "$P-x27-row-270.out" mutated_run X27 270 >> "$STATUS"

step v1-against-stored "$P-v1-against-stored.out" bash -c "cd '$V2' && python3 comparator_v1_against_stored.py \
  --pair I1:states:'$P-v1-i1.out':'$ROOT/research/results/e170-r1-o3-spec-executor-2026-10-03-s1-executor.out' \
  --pair I2:states:'$P-v1-i2.out':'$ROOT/research/results/e170-r1-o3-spec-executor-2026-10-03-s2-executor.out' \
  --pair I3:bad_states:'$P-v1-i3-images.out':'$ROOT/research/results/e170-r1-o3-spec-executor-2026-10-03-s3-bad-states.out' \
  --pair I4:images:'$P-v1-i4-images.out':'$ROOT/research/results/e171-three-oracle-collusion-2026-10-03-s2-images-o3.out' \
  --pair row-242:states:'$P-v1-row-242.out':'$ROWS/row-242/o3.out' --pair row-269:states:'$P-v1-row-269.out':'$ROWS/row-269/o3.out' \
  --pair row-270:states:'$P-v1-row-270.out':'$ROWS/row-270/o3.out' --output '$P-v1-against-stored.out'"
step v2-i1-against-segment-one "$P-v2-i1-against-segment-one.out" bash -c "cd '$V2' && python3 comparator_v1_against_stored.py \
  --pair I1-v2-batches-1-2:states:'$P-v2-i1.out':'$V2_FREEZE' --output '$P-v2-i1-against-segment-one.out'"

step comparator-q1 "$P-comparator-q1.out" bash -c "cd '$V2' && python3 comparator_two_versions.py \
  --pair I1:states:'$P-v1-i1.out':'$P-v2-i1.out' --pair I2:states:'$P-v1-i2.out':'$P-v2-i2.out' \
  --pair I3:images:'$P-v1-i3-images.out':'$P-v2-i3-images.out' --pair I4:images:'$P-v1-i4-images.out':'$P-v2-i4-images.out' \
  --batches 1,2 --names I3:'$I2' --names I4:'$I4':'$KNOWN_BAD' --unit-tests '$P-i5-v1-tests-on-v2.out' \
  --anchors '$P-v2-i1.out' --anchors '$P-v2-i2.out' --positive-control-input I2 --scratch '$SCRATCH' --output '$P-comparator-q1.out'"

step comparator-q2-q3 "$P-comparator-q2-q3.out" bash -c "cd '$V2' && python3 comparator_witness_classes.py \
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
finish_exit=0
python3 "$ROOT/$ITEM_VERDICTS" finish "$STATE/plan" > "$STATE/finish.out" || finish_exit=$?
cat -- "$STATE/finish.out"
((finish_exit <= 1)) || library_failure_exit finish "$finish_exit"
! e170_break_is finish-ignored || finish_exit=0
((finish_exit == 0)) || { echo "  ✗ 按步判定收尾判红（复用哨兵或键在跑的中途变了，见上面 ✗ 那几行）" >&2; echo "  → 怎么办：照上面那几行的出路办，再起同一条命令（新的日期标签）" >&2; exit 1; }
exit 0
