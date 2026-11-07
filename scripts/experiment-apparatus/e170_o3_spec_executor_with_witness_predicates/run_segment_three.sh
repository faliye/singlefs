#!/usr/bin/env bash
# admission: always 整个脚本每次都起：哪一步已有判定、键对不对得上由 prune-server 里的按步判定现答（scripts/experiment-apparatus/long_run_item_verdicts.py）
# run-condition: command python3 sha256sum
# E170 第 2 次重跑第三段（`research/prompts/e170-r2-prereg.md` 5.8 第三段那一行、第九节）：变异。全是 Python，不编 Rust。
# 做：输入核对（导出目录 manifest、第二段产物的 sha256）、v1 / v2 冻结行开头核对、变异驱动一趟（v1 的 X1–X20 与 v2 的 X22–X26、X28–X31 单测层，
# X22–X26、X28–X31 批层，XC3–XC7）、冻结行收尾核对。全部写进同一份产物 research/results/e170-r2-o3-spec-executor-<日期标签>-s3-mutations.out，
# 已存在就不覆盖（noclobber）。前台跑得完则前台跑；超过 240 秒的交主 agent 用 run_in_background 起。
# 跑法（在仓根，整条命令交 task-server，内存额度与线程由服务定）：
#   task-server/task submit -- \
#     bash scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/run_segment_three.sh <日期标签> <工作进程数> <草稿目录> [产物目录]
#   bash scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/run_segment_three.sh --selftest   自证（下面「自证」）
# 按步判定（.claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」）：两步各一条判定（inputs：输入核对与开头冻结核对；mutation-driver：变异驱动追加的那一段），
#   在 prune-server 的 apparatus-items 族、种类 e170_o3_spec_executor_with_witness_predicates（long_run_item_verdicts.py 文件头「判定库」）。
#   inputs 的键 = crc32c_castagnoli.py 与 v1、v2 两份 executor_main.py 的单元闭包、导出目录与第二段产物的内容、两份冻结参照、v1 与 v2 冻结的每个文件；逐字节可比（compare=exact）。
#   mutation-driver 的键 = mutation_driver_third_segment.py 的单元闭包（admission.py 的 compute_unit_key 算）、它读的两张变异表与第二段产物的内容、v1 与 v2 冻结的每个文件；
#   带计时（compare=timing）。路径不进键。键相同、库里判绿的步把库里那一段原样接进产物、不起程序（mutation-driver 那一步的 name=step 行带 reused=true，inputs 照旧没有 name=step 行）；判红（退非 0）的照同一把键记红撤绿、下一趟现判。
#   开头的 name=config 行与收尾的冻结核对每趟照写。末尾报「现判 N、复用 M、拒算 R」（LONG_RUN_ITEMS 那一行）。
#   退出码：0 跑完（步的成败记在产物的 name=step 行）；1 复用哨兵红或键在跑的中途变了；75 prune-server 不可用；2 用法错。
# 自证（--selftest）：经 long_run_item_verdicts.py 的复用格跑那六格（reuse-three-questions、key-independent-of-directory、reuse-sentinel 与哨兵的三种红）；样板树里 v1、v2 的
#   executor_main.py、crc32c_castagnoli.py、mutation_driver_third_segment.py 与两份冻结参照是假的，inputs、mutation-driver 两步的入口换成假树里两个 Python 函数（它文件头「复用格」）。
#   只在临时 prune-server 上跑（long_run_item_verdicts.py selftest-service，即客户端 selftest_service_problem；不在就退 2），起法：
#     python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/run_segment_three.sh --selftest
# 弄坏开关 E170_SEGMENT_THREE_BREAK=<项，逗号分隔>（只在 --selftest 起的那几趟里认：自证设 E170_SEGMENT_THREE_SELFTEST_MARK 指到它建的标记文件，别处设了退 2）：
#   E170_SEGMENT_THREE_BREAK=reuse-ignored  库里判绿的步也照起程序现判（不认 plan 答的 reuse）
#       「reuse-three-questions」「key-independent-of-directory」「reuse-sentinel」「forced-still-reused-red」
#   E170_SEGMENT_THREE_BREAK=finish-ignored  不认 long_run_item_verdicts.py finish 的退出码（复用哨兵红、键中途变了照退 0）
#       「sentinel-rejudged-red」「forced-verdict-differs-red」「forced-still-reused-red」
# 复用那几格另认 long_run_item_verdicts.py 的弄坏开关 LONG_RUN_ITEMS_BREAK（它文件头；打开后本脚本的自证红在它点名的同名格）。
set -u -o pipefail -o noclobber
source "$(dirname "${BASH_SOURCE[0]}")/../../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
export PYTHONDONTWRITEBYTECODE=1
SCRIPT_KEY="scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/run_segment_three.sh"
ITEM_VERDICTS="scripts/experiment-apparatus/long_run_item_verdicts.py"   # 按步判定的读写（文件头「按步判定」）
LIBRARY_KIND="e170_o3_spec_executor_with_witness_predicates"            # 判定库的种类：装置（执行器那一版）的目录名
V1_RELATIVE="scripts/experiment-apparatus/e170_second_run_o3_spec_executor"
V2_RELATIVE="scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates"
V1_FREEZE_RELATIVE="research/results/e170-r1-o3-spec-executor-2026-10-03-s1-executor.out"
V2_FREEZE_RELATIVE="research/results/e170-r2-o3-spec-executor-2026-10-04-s1-executor.out"

# 弄坏开关只在自证起的那几趟里认（标记文件在、内容对得上）
e170_break_is() {
  [[ ",${E170_SEGMENT_THREE_BREAK:-}," == *",$1,"* && -n "${E170_SEGMENT_THREE_SELFTEST_MARK:-}" ]] \
    && [[ "$(cat -- "$E170_SEGMENT_THREE_SELFTEST_MARK" 2> /dev/null)" == e170-segment-three-selftest ]]
}
if [[ -n "${E170_SEGMENT_THREE_BREAK:-}" && "${1:-}" != --selftest ]] \
    && [[ -z "${E170_SEGMENT_THREE_SELFTEST_MARK:-}" || "$(cat -- "${E170_SEGMENT_THREE_SELFTEST_MARK}" 2> /dev/null)" != e170-segment-three-selftest ]]; then
  echo "  ✗ E170_SEGMENT_THREE_BREAK 只给 --selftest 证红用，这一趟不是自证起的" >&2
  echo "  → 怎么办：平常跑时不设它" >&2
  exit 2
fi

# 自证：样板树里是本脚本的拷贝、v1 与 v2 的假执行器、假 crc32c、假变异驱动、两份假冻结参照、指回真仓的 .claude 与 long_run_item_verdicts.py；复用那六格经 reuse-cells 跑。
selftest() {
  local work tree reuse_output reuse_red
  python3 "$ROOT/$ITEM_VERDICTS" selftest-service || return 2
  work=$(mktemp -d) || return 1
  trap 'rm -rf -- "$work"' RETURN
  echo e170-segment-three-selftest > "$work/mark"
  tree="$work/template"
  mkdir -p -- "$tree/$V1_RELATIVE" "$tree/$V2_RELATIVE" "$tree/research/results"
  cp -- "$SCRIPT_PATH" "$tree/$SCRIPT_KEY"
  ln -s -- "$ROOT/$ITEM_VERDICTS" "$tree/$ITEM_VERDICTS"
  ln -s -- "$ROOT/.claude" "$tree/.claude"
  printf '%s\n' '"""复用格的假执行器：freeze_lines 交一行（本脚本的 freeze_check 照它算）。"""' '' '' 'def freeze_lines(emit):' \
    '    emit("name=freeze_seed sha256_prefix16=selftest")' > "$tree/$V1_RELATIVE/executor_main.py"
  cp -- "$tree/$V1_RELATIVE/executor_main.py" "$tree/$V2_RELATIVE/executor_main.py"
  printf '%s\n' '"""复用格的假 crc32c：一律交 0。"""' '' '' 'def crc32c(data):' '    return 0' > "$tree/$V2_RELATIVE/crc32c_castagnoli.py"
  printf '%s\n' '"""复用格的假变异驱动：往 --output 追加一行。"""' 'import sys' '' 'with open(sys.argv[sys.argv.index("--output") + 1], "a", encoding="utf-8") as handle:' \
    '    handle.write("name=selftest_mutation_driver\n")' > "$tree/$V2_RELATIVE/mutation_driver_third_segment.py"
  : > "$tree/$V1_FREEZE_RELATIVE"
  : > "$tree/$V2_FREEZE_RELATIVE"
  # 一趟一个新标签、新的草稿目录与产物目录（产物已在就不覆盖）：经 bash 拿进程号拼
  reuse_output=$(python3 "$ROOT/$ITEM_VERDICTS" reuse-cells --script "$SCRIPT_KEY" --template "$tree" --items inputs,mutation-driver --work "$work/reuse" --green-exit 0 -- \
    env E170_SEGMENT_THREE_SELFTEST_MARK="$work/mark" bash -c 'mkdir -p "$2/scratch-$$" "$2/results-$$" && exec bash "$1" "selftest$$" 1 "$2/scratch-$$" "$2/results-$$"' \
    run-segment-three-selftest "{tree}/$SCRIPT_KEY" "{tree}/.." 2>&1)
  echo "$reuse_output"
  reuse_red=$(sed -n 's/^long_run_item_verdicts 复用格.*：6 格，红 \([0-9]*\) 格$/\1/p' <<< "$reuse_output")
  if [[ -z "$reuse_red" ]]; then echo "  ✗ 复用那六格没跑完（没有「long_run_item_verdicts 复用格」那一行），按六格红算"; echo "     → 怎么办：看上面 reuse-cells 打的输出里第一处 ✗ 或 Python 报错（复用格起不来多半是 prune-server 不可用或 long_run_item_verdicts.py 抛了异常），修好之后在临时 prune-server 底下单独起本脚本的 --selftest（文件头「用法」那一条）到它退 0"; reuse_red=6; fi
  echo "run_segment_three.sh 自证：6 格，红 $reuse_red 格"
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
RESULTS_DIRECTORY="${4:-$ROOT/research/results}"  # 第 4 个参数只给试跑用（产物写到草稿目录）；第二段产物一律从 research/results 读
INPUT_PRODUCTS_DIRECTORY="$ROOT/research/results"
PRODUCT="$RESULTS_DIRECTORY/e170-r2-o3-spec-executor-$TAG-s3-mutations.out"
V1_FREEZE="$ROOT/$V1_FREEZE_RELATIVE"
V2_FREEZE="$ROOT/$V2_FREEZE_RELATIVE"
STREAMS="new_pool_file_creation fixed_script_through_e_then_unmount"
DUMP_DIRECTORIES=("$TARGET/e170-dumps" "$TARGET/e170-second-run-dumps" "$TARGET/e171-dumps/e170-bad-states-copy-s2" "$TARGET/e171-dumps/baseline-run1"
  "$TARGET/e171-dumps/rows-s4-f4/row-242/dump" "$TARGET/e171-dumps/rows-s4-f4/row-269/dump" "$TARGET/e171-dumps/rows-s4-f4/row-270/dump")

# ── 按步判定的请求（long_run_item_verdicts.py 文件头「请求文件」）──
frozen_file_requests() { # <项> <执行器目录>：那个目录里进冻结的每个文件（取法同 executor_main.freeze_lines：除 comparator*、mutation_driver*、witness_reference*、run_* 与 .pyc）
  local file
  while IFS= read -r file; do printf '@files\t%s\t%s\n' "$1" "$file"; done < <(find "$2" -maxdepth 1 -type f ! -name 'comparator*' ! -name 'mutation_driver*' \
    ! -name 'witness_reference*' ! -name 'run_*' ! -name '*.pyc' 2> /dev/null | sort)
}
input_product_requests() { # <项>：第二段产物（比对器与参照谓词的输入）
  local file
  for file in "$INPUT_PRODUCTS_DIRECTORY"/e170-r2-o3-spec-executor-2026-10-04-s2-*.out; do printf '@files\t%s\t%s\n' "$1" "$file"; done
}
segment_requests() {
  local directory
  printf 'inputs\t%s/crc32c_castagnoli.py\tline:program=inputs\tline:compare=exact\ninputs\t%s/executor_main.py\ninputs\t%s/executor_main.py\n' "$V2_RELATIVE" "$V1_RELATIVE" "$V2_RELATIVE"
  for directory in "${DUMP_DIRECTORIES[@]}"; do printf '@files\tinputs\t%s\n' "$directory"; done
  printf '@input\tinputs\tv1-freeze\t%s\n@input\tinputs\tv2-freeze\t%s\n' "$V1_FREEZE" "$V2_FREEZE"
  input_product_requests inputs
  frozen_file_requests inputs "$V1"; frozen_file_requests inputs "$V2"
  printf 'mutation-driver\t%s/mutation_driver_third_segment.py\tline:program=mutation-driver-third-segment\tline:compare=timing\n' "$V2_RELATIVE"
  printf '@input\tmutation-driver\tv1-table\t%s\n@input\tmutation-driver\tv2-table\t%s\n' \
    "$ROOT/scripts/mutation/tables/e170_second_run_o3_spec_executor.tsv" "$ROOT/scripts/mutation/tables/e170_o3_spec_executor_with_witness_predicates.tsv"
  input_product_requests mutation-driver
  frozen_file_requests mutation-driver "$V1"; frozen_file_requests mutation-driver "$V2"
}

library_failure_exit() { # <哪一步> <退出码>：75（prune-server 不可用）与 2 照交，别的按 1 交
  echo "  ✗ 按步判定的 $1 没成（long_run_item_verdicts.py 退 $2）" >&2
  echo "  → 怎么办：退 75 是 prune-server 不可用：python3 prune-server/control check 看服务，修好之后重交同一条命令（报给主 agent）；别的照上面它打的出路改" >&2
  if (($2 == 75 || $2 == 2)); then exit "$2"; fi
  exit 1
}

freeze_check() {  # <名> <执行器目录> <带冻结行的产物> <when>：冻结行此刻照算一遍，与第一段（v1 是第 1 次重跑第一段）产物里的逐行比（V8）
  (cd "$2" && python3 -c "
import sys; sys.path.insert(0, '.')
import executor_main
now = []; executor_main.freeze_lines(now.append)
reference = [line.rstrip('\n') for line in open('$3') if line.startswith(('name=freeze ', 'name=freeze_seed '))]
print(f'name=freeze_check executor=$1 when=$4 identical={str(now == reference and len(reference) > 0).lower()} lines_now={len(now)} reference_lines={len(reference)} seed_now={now[-1].split(\"=\")[-1]}')")
}

inputs() {  # V2：用到的导出目录照 manifest.tsv 的 sha256: 行核；第二段产物（比对器与参照谓词的输入）逐个记 sha256；B8–B10；冻结行开头核对
  for directory in "${DUMP_DIRECTORIES[@]}"; do
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

STATE="$(mktemp -d "$SCRATCH/items-XXXXXX")" || exit 1
trap 'rm -rf -- "$STATE"' EXIT
segment_requests > "$STATE/requests.tsv"
python3 "$ROOT/$ITEM_VERDICTS" plan "$STATE/plan" --script "$SCRIPT_KEY" --kind "$LIBRARY_KIND" --root "$ROOT" --requests "$STATE/requests.tsv" \
  || library_failure_exit plan "$?"
reused() { # <项>：plan 答 reuse 且没开 reuse-ignored
  [[ "$(awk -F'\t' -v item="$1" '$1 == item {print $2}' "$STATE/plan/decisions.tsv")" == reuse ]] && ! e170_break_is reuse-ignored
}
record_step() { # <项> <退出码> <这一步那一段>
  local red=()
  (($2 == 0)) || red=(--red)
  python3 "$ROOT/$ITEM_VERDICTS" record "$STATE/plan" "$1" "$2" "$3" ${red[@]+"${red[@]}"} || library_failure_exit "record（步 $1）" "$?"
}

echo "name=config segment=3 tag=$TAG workers=$WORKERS scratch=$SCRATCH results_directory=$RESULTS_DIRECTORY" > "$PRODUCT" || exit 1
if reused inputs; then   # 产物里输入核对那几行照旧没有 name=step 行（复用了没有，看末尾 LONG_RUN_ITEMS 前面那几行）
  cat -- "$STATE/plan/outputs/inputs.out" >> "$PRODUCT"
else
  inputs > "$STATE/inputs.out"; inputs_exit=$?
  cat -- "$STATE/inputs.out" >> "$PRODUCT"
  record_step inputs "$inputs_exit" "$STATE/inputs.out"
fi
mkdir -p "$SCRATCH/driver-scratch"
if reused mutation-driver; then
  cat -- "$STATE/plan/outputs/mutation-driver.out" >> "$PRODUCT"
  echo "name=step step=mutation_driver exit=$(cat -- "$STATE/plan/outputs/mutation-driver.exit") reused=true" >> "$PRODUCT"
else
  before=$(wc -c < "$PRODUCT")
  (cd "$V2" && python3 mutation_driver_third_segment.py --scratch "$SCRATCH/driver-scratch" --output "$PRODUCT" --workers "$WORKERS" \
     --results-directory "$INPUT_PRODUCTS_DIRECTORY" > "$SCRATCH/driver.log" 2>&1)
  driver_exit=$?
  tail -c "+$((before + 1))" -- "$PRODUCT" > "$STATE/driver.out"   # 驱动这一趟追加的那一段，就是这一步的产物
  echo "name=step step=mutation_driver exit=$driver_exit" >> "$PRODUCT"
  record_step mutation-driver "$driver_exit" "$STATE/driver.out"
fi
{ freeze_check v1 "$V1" "$V1_FREEZE" end; freeze_check v2 "$V2" "$V2_FREEZE" end; } >> "$PRODUCT"
echo "name=complete failed_steps=$(grep -c 'name=step .* exit=[1-9]' "$PRODUCT")" >> "$PRODUCT"
finish_exit=0
python3 "$ROOT/$ITEM_VERDICTS" finish "$STATE/plan" > "$STATE/finish.out" || finish_exit=$?
cat -- "$STATE/finish.out"
((finish_exit <= 1)) || library_failure_exit finish "$finish_exit"
! e170_break_is finish-ignored || finish_exit=0
((finish_exit == 0)) || { echo "  ✗ 按步判定收尾判红（复用哨兵或键在跑的中途变了，见上面 ✗ 那几行）" >&2; echo "  → 怎么办：照上面那几行的出路办，再起同一条命令（新的日期标签）" >&2; exit 1; }
exit 0
