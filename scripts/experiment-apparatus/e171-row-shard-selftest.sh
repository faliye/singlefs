#!/usr/bin/env bash
# admission: always 判的是此刻的 e171-row-shard.sh 与 shared_steps.py，上一次的结论不替这一次作保
# run-condition: command python3 sha256sum
# selftest-serial: 后几格读前一格在同一个共用输出根里留下的产物与事件记录（再起一趟取现成的、片的参数、领卡次序），格与格之间有先后
# e171-row-shard.sh（E171 整表那几行的分片入口）的自证：临时目录里搭一棵小树，放本仓的 e171-row-shard.sh、shared_steps.py、device_card_lease.py、frozen_o3.py、
# 读多机配置的共用库，别的换成假的：假逐行驱动（现判那一路按参数建行目录、写进假库、记一笔；只取那一路 take_rows_into_the_output_root 照假库取行、缺的逐行点名，
# 打的几种行同真驱动；shared_steps.py 引的那几个名字里，打包、算键用的几个从本仓真的 row_driver.py 转引）、
# 假 cargo（编只记一笔，跑装置照 --output-directory 建目录）、假领卡客户端、假 O3 判官、假执行器、假 comparator.py（--print-q3-core-rows 打固定两行）。
# 假库是 FAKE_ROW_LIBRARY 那个目录（一行一个文件，名字是这一行的键里会变的那几样），只在这一趟的临时目录里，替的是 prune-server 里逐行驱动那一族；
# 小树的步长 8 那张表 10 行（1–10）、扩表 4 行（11–14）。
# 不起编译、不起装置；几样往同一份事件记录里记，按先后核。共用前置的判定写进临时 prune-server（命名空间是这一趟自己的），五项的装置入口经
# LONG_RUN_ITEMS_SELFTEST_ENTRIES 换成小树里的假 Python 入口（long_run_item_verdicts.py plant-fake-entries），判法闭包照本仓（E171_REPOSITORY_ROOT 指回本仓）算。
# 不带分片变量起的那几格走入口自己的假服务（它文件头「分片」末一条）；sharded-through-a-fake-service 那一格由这里当假服务，照协议逐步设 TASK_SERVER_SHARD_PHASE 等变量。
#   python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e171-row-shard-selftest.sh [--selftest | --break <项>]
#   不在临时 prune-server 底下退 2、一格都不跑（long_run_item_verdicts.py selftest-service 判，即客户端 selftest_service_problem）。
# 格（每格打一行 GREEN <格> 或 RED <格>：<为什么>）：
#   listing              --list-items 退 0、列出的项与次序对（缺省片数 step8 8 片、f4 12 片；--step8-slices、--f4-slices 换片数），一个假程序都没起
#   unknown-item         --item 写错的项名、--item 后面缺项名：都退 2、列出可点名的项，一个假程序都没起
#   missing-prerequisite 空的共用输出根上点 q3core:O1、positive-control:O3、substitutes（三项都在合并里做）：都退 4、点名缺的那一项（prerequisites、step8），
#                        装置与逐行驱动一个都没起（只取那一路记的 rows_taken 不算）
#   named-equals-all     一个根上照 --list-items 的次序逐项点名做，另一个根上不带 --item 依次全做：两个根里的文件（工作根之外）逐个相同，两趟都退 0，
#                        依次全做那一趟每项一行 E171_ROW_SHARD_ITEM、总计 15 项全过
#   second-run-takes     做完之后照分片协议单起一片「项 prerequisites」（手写的 shards.tsv）：判定库里有，退 0、标准错误有 taken=yes、产物目录有 shard.done，
#                        假 cargo 与假领卡客户端一笔都没记
#                        这两格先核共用前置 prerequisites 的键在逐项点名、依次全做、单起那一片三趟里没变过（各趟标准错误 E171_SHARED_STEP item=prerequisites … key=）：
#                        判法闭包照本仓算，自证跑着的时候本仓里闭包碰到的源码被改了，库里早先记的那一条就对不上，后面要它的项退 4、再起的那一片现判；
#                        键变过的，两格都红在「键在这几趟里变过」、别的条件不再判（结局不算数，等改动定下来再起同一条命令）
#   slice-arguments      逐项点名 step8:2/2 那一趟判第 6–10 行的那一片的逐行驱动：工作根是那一片的产物目录下的 work、--rows 6,7,8,9,10、
#                        带 --keep-compile-failures-as-they-are 与 --lease-card-per-device-run
#   card-around-device   prerequisites 里每一次跑装置都分录入、判两段：录入段一张卡都不持（cards=none），判段夹在领卡与还卡之间、拿的是领到的那张，
#                        两段各 5 次；编的时候一张卡都不持，E170 导出装置不拿卡
#   no-reuse-from        依次全做那一趟的逐行驱动一次都不带 --reuse-from（补位与 Q3-核的行由逐行驱动自己从判定库取）；--reuse-rows-from 退 2
#   header               文件头前 60 行有按需领卡、会编与分片（task-server-shards: *）三句声明，没有起跑就分卡的 task-server-gpu-arguments
#   sharded-through-a-fake-service  这里当假服务，换一份空的假库与共用前置的命名空间，照 task-server/task_server_lib.py 文件头「分片」逐步设变量起分片入口：
#                        片数上限 7 时两张表各并成一片（步长 8 那张 10 行一片）、共 7 片；
#                        上限 64 时 8 片：四项共用前置与三条阳性对照里不依赖前置的、empty-mutation 各一片，步长 8 那张两片（1–5、6–10）、扩表一片（11–14），
#                        plan.json 8 个 label；每一片退 0、产物目录有 shard.done；合并退 0、总计 15 项全过；再起计划 0 片；
#                        合并时 TASK_SERVER_SHARD_MISSING=3 退 1、点名缺片 3；假库里删掉扩表第 12 行之后点 f4:1/2 的合并退非 0、点名 item=f4:1/2 row=12
# 弄坏开关（--break，把临时树里的副本改坏，自证该红在点名的那一格）：
#   dependency-check-dropped   前置核对一律当做完了          -> 红在 missing-prerequisite（连带 slice-arguments）
#   all-run-drops-an-item      不带 --item 依次全做时漏掉 q3core:no-executor   -> 红在 named-equals-all（连带 sharded-through-a-fake-service）
#   reuse-ignored              shared_steps.py 判定库答复用也照做一遍   -> 红在 second-run-takes（连带 named-equals-all：依次全做那一趟合并里重做已在库里的
#                              positive-control:O3，finish 判红退 1）
#   reuse-from-passed          s2-substitutes 给逐行驱动带 --reuse-from <根>/step8   -> 红在 no-reuse-from
#   item-name-unchecked        --item 不核项名                 -> 红在 unknown-item
#   shard-work-root-shared     表的片的工作根不在服务给的产物目录下，几片共用 <根>/work-step8   -> 红在 slice-arguments
#   lease-flag-dropped         开了 GPU 也不给逐行驱动 --lease-card-per-device-run   -> 红在 slice-arguments
#   card-at-start-header       文件头按需领卡那一句换成起跑就分卡的 task-server-gpu-arguments   -> 红在 header
#   record-stage-holds-a-card  device_card_lease.py 录入段之前就领卡、把卡串交给录入段   -> 红在 card-around-device
#   prerequisites-key-moved-midway  逐项点名做完、依次全做之前，改小树里假装置入口 alpha_detail 的返回值（prerequisites 的入口 measure_alpha 闭包里的一行，键跟着变）
#                              -> 红在 named-equals-all、second-run-takes（「键在这几趟里变过」）
#   shards-header-dropped      文件头的 task-server-shards 声明那一行删了   -> 红在 header
#   plan-ignores-the-library   计划查库一律当库里没有   -> 红在 sharded-through-a-fake-service（再起计划不是 0 片）
#   slices-over-eight-rows     计划切表时一片至多 16 行   -> 红在 sharded-through-a-fake-service（步长 8 那张 10 行切成一片）
#   shard-maximum-ignored      计划不照 TASK_SERVER_SHARD_MAXIMUM 并片   -> 红在 sharded-through-a-fake-service（上限 7 时出了 8 片）
#   audit-skipped              合并核账缺的一样都不算   -> 红在 sharded-through-a-fake-service（假库里缺第 12 行的合并退 0）
#   missing-shards-ignored     合并不看 TASK_SERVER_SHARD_MISSING   -> 红在 sharded-through-a-fake-service（缺片 3 的合并退 0）
# 退出码：0 全过；1 有格红；2 用法不对、弄坏开关没改动副本（prerequisites-key-moved-midway：没改动小树里的假装置入口）、不在临时 prune-server 底下。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repository="$(cd "$here/../.." && pwd)"
break_item=""
if [[ "${1:-}" == --selftest && $# -eq 1 ]]; then set --; fi   # 登记表与 runner 表的统一起法（<脚本> --selftest）与不带参数同义
if [[ "${1:-}" == --break && -n "${2:-}" && $# -eq 2 ]]; then break_item="$2"; elif [[ $# -gt 0 ]]; then echo "用法：$0 [--break <项>]" >&2; exit 2; fi
verdicts="$repository/scripts/experiment-apparatus/long_run_item_verdicts.py"
python3 "$verdicts" selftest-service || exit 2   # 共用前置的判定只写临时 prune-server
tmp="$(mktemp -d)"
trap 'rm -rf "${tmp:?}"' EXIT
tree="$tmp/tree"
research="$tree/scripts/experiment-apparatus/e171_three_oracle_collusion"
mkdir -p "$research" "$tree/scripts/crash" "$tree/scripts/hosts" "$tree/.claude/scripts" "$tree/task-server" "$tree/crates/core/src" \
  "$tree/scripts/experiment-apparatus/e170_second_run_o3_spec_executor_journal_header_496" "$tree/research/results" "$tmp/bin" "$tmp/channel"
shard="$tree/scripts/experiment-apparatus/e171-row-shard.sh"
steps="$research/shared_steps.py"
cp "$here/e171-row-shard.sh" "$shard"
cp "$here/e171_three_oracle_collusion/shared_steps.py" "$steps"
cp "$here/e171_three_oracle_collusion/device_card_lease.py" "$here/e171_three_oracle_collusion/frozen_o3.py" "$research/"
cp "$repository/scripts/crash/crash-amplification-lib.sh" "$tree/scripts/crash/"
cp "$repository/scripts/hosts/heavy-threads.sh" "$tree/scripts/hosts/"
python3 "$verdicts" plant-fake-entries "$tree" || exit 2
fake_entry="scripts/experiment-apparatus/long_run_reuse_fake_entries.py::measure_alpha"
printf '%s\t%s\n' prerequisites "$fake_entry" positive-control.O1-model-differential "$fake_entry" positive-control.O1-crash-judge "$fake_entry" \
  positive-control.O2 "$fake_entry" positive-control.O3 "$fake_entry" >"$tmp/entries.tsv"
printf '%s\n' 'preflight() { shift; PREFLIGHT_ARGUMENTS=("$@"); }   # 自证的小树里不判准入与运行条件' >"$tree/.claude/scripts/preflight.sh"
printf '[workspace]\n' >"$tree/Cargo.toml"
printf 'pub fn core() {}\n' >"$tree/crates/core/src/lib.rs"
printf '冻结行\n' >"$tree/research/results/e170-o3-spec-executor-journal-header-496-2026-11-09-freeze-executor-e171-r3.out"
for row_number in 1 2 3 4 5 6 7 8 9 10; do printf '%s\tstep8\n' "$row_number"; done >"$research/bad-implementations-step-8.tsv"
for row_number in 11 12 13 14; do printf '%s\tf4\n' "$row_number"; done >"$research/bad-implementations-step-4-only.tsv"
printf 'ENABLE_GPU=1\n' >"$tmp/multi-host.env"

cat >"$research/row_driver.py" <<'PYTHON'
"""假逐行驱动：被 shared_steps.py import 时给它引的那几个名字（打包、算键用的几个从本仓真的 row_driver.py 转引；只取那一路换成读假库的
take_rows_into_the_output_root）；当脚本起时按参数在输出根里写行目录，往事件记录里记一笔。假库是 FAKE_ROW_LIBRARY 那个目录：一行一个文件，
名字是这一行的键里会变的那几样（序号、关哪一条、跑不跑执行器、哪条阳性对照），现判那一路库里有就照它写、没有就写一份新的并存进去。"""
import importlib.util
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REAL_DIRECTORY = os.path.join(os.environ["E171_REPOSITORY_ROOT"], "scripts", "experiment-apparatus", "e171_three_oracle_collusion")
sys.path.append(REAL_DIRECTORY)
_specification = importlib.util.spec_from_file_location("e171_row_driver_of_the_repository", os.path.join(REAL_DIRECTORY, "row_driver.py"))
_real = importlib.util.module_from_spec(_specification)
sys.modules[_specification.name] = _real   # dataclasses 认类型时按模块名在 sys.modules 里找
_specification.loader.exec_module(_real)
pack_directory, unpack_directory, item_verdicts_module = _real.pack_directory, _real.unpack_directory, _real.item_verdicts_module
o3_freeze_digest, empty_mutation_line_of, parse_row_range = _real.o3_freeze_digest, _real.empty_mutation_line_of, _real.parse_row_range
BREAK_VARIABLE = _real.BREAK_VARIABLE
BREAK_ACCEPTED_IN_PROCESS = False
SYNCED_ENTRIES = ("Cargo.toml", "Cargo.lock", ".cargo", "crates")
DEVICE_PACKAGE = "singlefs-verification"
DEVICE_BINARY_NAME = "e171_three_oracle_collusion"
O3_CONTROL = "O3"
O3_CONTROL_ROW_NAME = "P1-F"
IMAGE_JUDGE_SCRIPT = os.path.join(HERE, "o3_bad_image_judge.py")


class Row:
    def __init__(self, number):
        self.number = number


def read_selected_rows(path):
    with open(path, encoding="utf-8") as handle:
        return [Row(int(line.split("\t", 1)[0])) for line in handle if line.strip()]


def path_dependency_entries(root):
    return []


def threads_per_lane(threads, lanes, environment=os.environ):
    return 1


def value(arguments, flag, default=None):
    return arguments[arguments.index(flag) + 1] if flag in arguments else default


def chosen_numbers(table, rows, row_range, empty_only, empty):
    """选行同真驱动的 choose_rows：只跑空变异是 row-0；--rows 取表里点到的；--row-range 取区间里的；都没给是整张表；--empty-mutation 另加 row-0。"""
    numbers = [row.number for row in read_selected_rows(table)]
    if empty_only:
        return [0]
    if rows is not None:
        wanted = {int(word) for word in rows.split(",") if word}
        chosen = [number for number in numbers if number in wanted]
    elif row_range is not None:
        low, high = (int(word) for word in row_range.split("-"))
        chosen = [number for number in numbers if low <= number <= high]
    else:
        chosen = numbers
    return ([0] if empty else []) + chosen


def library_path(number, disabled, no_executor, control):
    name = f"row-{number}.{disabled or 'all-oracles'}.{'no-executor' if no_executor else 'executor'}.{control or 'no-control'}"
    return os.path.join(os.environ["FAKE_ROW_LIBRARY"], name)


def write_row(output, number, content):
    row = os.path.join(output, f"row-{number}")
    os.makedirs(row, exist_ok=True)
    with open(os.path.join(row, "row-result.tsv"), "w", encoding="utf-8") as handle:
        handle.write(content)


def read_text(path):
    with open(path, encoding="utf-8") as handle:
        return handle.read()


def event(line):
    with open(os.environ["FAKE_EVENT_LOG"], "a", encoding="utf-8") as log:
        log.write(line + "\n")


def main(arguments):
    table = value(arguments, "--selected-rows", os.path.join(HERE, "bad-implementations-step-8.tsv"))
    numbers = [row.number for row in read_selected_rows(table)]
    if "--print-shard-ranges" in arguments:
        count, start = int(value(arguments, "--print-shard-ranges")), 0
        for index in range(count):
            end = len(numbers) * (index + 1) // count
            part, start = numbers[start:end], end
            print(f"shard_range index={index + 1} low={part[0]} high={part[-1]} rows={len(part)}")
        return 0
    chosen = chosen_numbers(table, value(arguments, "--rows"), value(arguments, "--row-range"), "--empty-mutation-only" in arguments,
                            "--empty-mutation" in arguments)
    output = value(arguments, "--output-root")
    flags = " ".join(word for word in arguments if word.startswith("--") and word not in ("--repo-root", "--work-root", "--output-root"))
    os.makedirs(os.environ["FAKE_ROW_LIBRARY"], exist_ok=True)
    for number in chosen:
        path = library_path(number, value(arguments, "--disable-oracle"), "--no-executor" in arguments, value(arguments, "--positive-control"))
        if not os.path.isfile(path):
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(f"row_number\t{number}\ntable\t{os.path.basename(table)}\nflags\t{flags}\n")
        write_row(output, number, read_text(path))
    event(f"row_driver work={os.path.basename(value(arguments, '--work-root'))} output={os.path.basename(output)} {flags} "
          f"range={value(arguments, '--row-range', '-')} rows={','.join(map(str, chosen))}")
    return 0


def take_rows_into_the_output_root(arguments, printer):
    """只取：假库里有的行写进输出根（check_only 不写），缺的逐行点名；打的几种行同真驱动（row_driver.py 文件头「只取」）。"""
    chosen = chosen_numbers(arguments.selected_rows, arguments.rows, arguments.row_range, arguments.empty_mutation_only, arguments.empty_mutation)
    taken, missing = 0, []
    for number in chosen:
        path = library_path(number, arguments.disable_oracle, arguments.no_executor, arguments.positive_control)
        if not os.path.isfile(path):
            missing.append((number, "库里没有这把键（假库）"))
            continue
        taken += 1
        if not arguments.check_only:
            write_row(arguments.output_root, number, read_text(path))
    for number, reason in missing:
        printer(f"E171_ROW_NOT_IN_THE_LIBRARY row={number} reason={reason}")
    printer(f"E171_ROWS_TAKEN taken={taken} missing={len(missing)} of={len(chosen)} check_only={'yes' if arguments.check_only else 'no'} "
            f"output_root={arguments.output_root}")
    event(f"rows_taken output={os.path.basename(arguments.output_root)} check_only={'yes' if arguments.check_only else 'no'} "
          f"rows={','.join(map(str, chosen))} missing={','.join(str(number) for number, _ in missing) or '-'}")
    return missing


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
PYTHON
cat >"$research/comparator.py" <<'PYTHON'
import sys
assert "--print-q3-core-rows" in sys.argv
print("q3_core_rows=2,4")
PYTHON
for writer in o3_bad_image_judge executor_main; do
  target="$research/$writer.py"
  [[ "$writer" == executor_main ]] && target="$tree/scripts/experiment-apparatus/e170_second_run_o3_spec_executor_journal_header_496/executor_main.py"
  cat >"$target" <<PYTHON
import os, sys
arguments = sys.argv[1:]
with open(os.environ["FAKE_EVENT_LOG"], "a", encoding="utf-8") as log:
    log.write("$writer " + arguments[0] + "\n")
with open(arguments[arguments.index("--output") + 1], "w", encoding="utf-8") as handle:
    handle.write("$writer " + " ".join(os.path.basename(word) for word in arguments) + "\n")
PYTHON
done
cat >"$tmp/bin/cargo" <<'BASH'
#!/usr/bin/env bash
case "${1:-}" in
  --version) echo "cargo 0.0.0 (selftest)"; exit 0 ;;
  build) echo "build" >>"$FAKE_EVENT_LOG"; exit 0 ;;
  run) ;;
  *) echo "假 cargo 只认 --version、build、run，收到：$*" >&2; exit 2 ;;
esac
binary=""; arguments=()
while (($#)); do
  if [[ "$1" == --bin ]]; then binary="$2"; shift 2; continue; fi
  if [[ "$1" == -- ]]; then shift; arguments=("$@"); break; fi
  shift
done
stage="-"
for ((index = 0; index < ${#arguments[@]}; index++)); do
  if [[ "${arguments[index]}" == --stage ]]; then stage="${arguments[index + 1]}"; fi
done
echo "run $binary ${arguments[0]} stage=$stage cards=${GPU_CARDS_BY_PRIORITY:-none}" >>"$FAKE_EVENT_LOG"
for ((index = 0; index < ${#arguments[@]}; index++)); do
  if [[ "${arguments[index]}" == --output-directory ]]; then mkdir -p "${arguments[index + 1]}/stream"; printf '%s\n' "${arguments[0]}" >"${arguments[index + 1]}/stream/verdicts.tsv"; fi
done
echo "E7RESULT name=fake ${arguments[0]}"
BASH
cat >"$tree/task-server/task" <<'BASH'
#!/usr/bin/env bash
log="${FAKE_EVENT_LOG:?}"
case "${1:-} ${2:-}" in
  "lease take")
    number=$(( $(cat "$log.count" 2>/dev/null || echo 0) + 1 ))
    echo "$number" >"$log.count"
    echo "take card-$number" >>"$log"
    echo "card-$number" ;;
  "lease return") echo "return ${3:-all}" >>"$log" ;;
  *) echo "假领卡客户端只认 lease take、lease return，收到：$*" >&2; exit 2 ;;
esac
BASH
chmod +x "$tmp/bin/cargo" "$tree/task-server/task" "$shard"

case "$break_item" in
  "") ;;
  dependency-check-dropped) sed -i 's/ && return 0$/; return 0/' "$shard" ;;
  all-run-drops-an-item) sed -i "s/mapfile -t wanted < <(list_items | cut -f1)/mapfile -t wanted < <(list_items | cut -f1 | sed '\$d')/" "$shard" ;;
  reuse-ignored) sed -i 's/if planned\[name\]\["decision"\] == verdicts.REUSE and/if False and/' "$steps" ;;
  reuse-from-passed) sed -i 's|run_rows s2-substitutes "$work" --selected-rows "$step8_table" --lanes|run_rows s2-substitutes "$work" --selected-rows "$step8_table" --reuse-from "$root/step8" --lanes|' "$shard" ;;
  item-name-unchecked) sed -i 's/ \[\[ -n "\$(item_rank "\$2")" \]\] || refuse_unknown_item "\$2";//' "$shard" ;;
  shard-work-root-shared) sed -i 's|run_rows step8 "$output/work"|run_rows step8 "$root/work-step8"|' "$shard" ;;
  lease-flag-dropped) sed -i 's/card_lease=(--lease-card-per-device-run)/card_lease=()/' "$shard" ;;
  card-at-start-header) sed -i 's/^# task-server-card-lease: on-demand$/# task-server-gpu-arguments: step8 f4/' "$shard" ;;
  record-stage-holds-a-card) sed -i 's/held = take() if broken(CARD_HELD_THROUGH_THE_RECORD_STAGE) else None/held = take()/' "$research/device_card_lease.py" ;;
  prerequisites-key-moved-midway) ;;   # 不改副本：在逐项点名与依次全做之间改小树里的假装置入口（named-equals-all 那一段）
  shards-header-dropped) sed -i '/^# task-server-shards: \*/d' "$shard" ;;
  plan-ignores-the-library) sed -i 's/; then return 0; else return \$?; fi$/; then return 4; else return $?; fi/' "$shard" ;;
  slices-over-eight-rows) sed -i 's/^ROWS_PER_SLICE_AT_MOST=8 /ROWS_PER_SLICE_AT_MOST=16 /' "$shard" ;;
  shard-maximum-ignored) sed -i 's/ > maximum )); do$/ > maximum * 1000 )); do/' "$shard" ;;
  audit-skipped) sed -i 's/^    item_missing_count="\$(grep -c \. "\$item_missing" || true)"   # 核账：这一项缺几样$/    item_missing_count=0/' "$shard" ;;
  missing-shards-ignored) sed -i 's/^  if \[\[ -n "\$missing_shards" \]\]; then$/  if false; then/' "$shard" ;;
  *) echo "不认得的弄坏开关 $break_item" >&2; exit 2 ;;
esac
if [[ -n "$break_item" && "$break_item" != prerequisites-key-moved-midway ]] && cmp -s "$shard" "$here/e171-row-shard.sh" \
  && cmp -s "$steps" "$here/e171_three_oracle_collusion/shared_steps.py" \
  && cmp -s "$research/device_card_lease.py" "$here/e171_three_oracle_collusion/device_card_lease.py"; then
  echo "弄坏开关 $break_item 没改动副本" >&2
  exit 2
fi

events="$tmp/events.log"
export FAKE_EVENT_LOG="$events" PATH="$tmp/bin:$PATH" SINGLEFS_MULTI_HOST_CONFIG="$tmp/multi-host.env" \
  TASK_SERVER_CARD_LEASE_DIRECTORY="$tmp/channel" TASK_SERVER_CARD_LEASE_TOKEN=selftest-token \
  E171_REPOSITORY_ROOT="$repository" LONG_RUN_ITEMS_LIBRARY_DIRECTORY="$tmp/library" LONG_RUN_ITEMS_SELFTEST_ENTRIES="$tmp/entries.tsv" \
  SINGLEFS_REUSE_SENTINEL_COUNT=1000 FAKE_ROW_LIBRARY="$tmp/row-library"
unset E171_SHARED_STEPS_BREAK E171_ROW_DRIVER_BREAK LONG_RUN_ITEMS_SELFTEST_FAULT LONG_RUN_ITEMS_SELFTEST_FAKE_JUDGE LONG_RUN_ITEMS_BREAK \
  SINGLEFS_REUSE_SENTINEL_SEED TASK_SERVER_SHARD_PHASE TASK_SERVER_SHARD_PLAN_DIRECTORY TASK_SERVER_SHARD \
  TASK_SERVER_SHARD_OUTPUT_DIRECTORY TASK_SERVER_SHARD_OUTPUTS TASK_SERVER_SHARD_MISSING TASK_SERVER_SHARD_MAXIMUM
fail=0
red() { echo "RED $1：$2"; fail=1; }
green() { echo "GREEN $1"; }
shard_run() { # <输出文件前缀> <参数…>：在小树的根上起 e171-row-shard.sh，标准输出、标准错误各写一份，打退出码
  local prefix="$1"
  shift
  (cd "$tree" && bash scripts/experiment-apparatus/e171-row-shard.sh "$@") >"$prefix.out" 2>"$prefix.err"
  echo $?
}
event_count() { if [[ -f "$events" ]]; then grep -c . "$events"; else echo 0; fi; }

# listing
: >"$events"
code="$(shard_run "$tmp/list-default" --list-items)"
code_small="$(shard_run "$tmp/list-small" --list-items --step8-slices 2 --f4-slices 2)"
expected_small="prerequisites
positive-control:O1-对拍
positive-control:O1-崩溃判官
positive-control:O2
positive-control:O3
empty-mutation
step8:1/2
step8:2/2
f4:1/2
f4:2/2
s2-substitutes
substitutes
q3core:O1
q3core:O2
q3core:no-executor"
if [[ "$code" != 0 || "$code_small" != 0 ]]; then red listing "退 $code / $code_small：$(tail -n 2 "$tmp/list-default.err" | tr '\n' ' ')"
elif [[ "$(cut -f1 "$tmp/list-small.out")" != "$expected_small" ]]; then red listing "片数 2、2 时列的项不对：$(cut -f1 "$tmp/list-small.out" | tr '\n' ' ')"
elif [[ "$(grep -c '^step8:[0-9]*/8	' "$tmp/list-default.out")" != 8 || "$(grep -c '^f4:[0-9]*/12	' "$tmp/list-default.out")" != 12 ]]; then red listing "缺省片数不是 step8 8 片、f4 12 片"
elif [[ "$(event_count)" != 0 ]]; then red listing "列项起了假程序：$(head -n 3 "$events" | tr '\n' ' ')"
else green listing; fi

# unknown-item
code="$(shard_run "$tmp/unknown" --root "$tmp/unused" --item nonsense)"
code_missing="$(shard_run "$tmp/no-name" --root "$tmp/unused" --item)"
if [[ "$code" != 2 || "$code_missing" != 2 ]]; then red unknown-item "写错的项名退 $code、缺项名退 $code_missing（都要 2）"
elif ! grep -q '^    prerequisites$' "$tmp/unknown.err" || ! grep -q '^    prerequisites$' "$tmp/no-name.err"; then red unknown-item "没列出可点名的项"
elif [[ "$(event_count)" != 0 ]]; then red unknown-item "起了假程序：$(head -n 3 "$events" | tr '\n' ' ')"
else green unknown-item; fi

# missing-prerequisite
: >"$events"
code_q3="$(shard_run "$tmp/missing-q3" --root "$tmp/empty-root" --item q3core:O1)"
code_o3="$(shard_run "$tmp/missing-o3" --root "$tmp/empty-root" --item positive-control:O3)"
code_substitutes="$(shard_run "$tmp/missing-substitutes" --root "$tmp/empty-root" --step8-slices 2 --item substitutes)"
if [[ "$code_q3" != 4 || "$code_o3" != 4 || "$code_substitutes" != 4 ]]; then red missing-prerequisite "q3core:O1 退 $code_q3、positive-control:O3 退 $code_o3、substitutes 退 $code_substitutes（都要 4）"
elif ! grep -q '要先有 prerequisites' "$tmp/missing-q3.err" || ! grep -q '要先有 prerequisites' "$tmp/missing-o3.err" || ! grep -q '要先有 step8' "$tmp/missing-substitutes.err"; then
  red missing-prerequisite "没点名缺的那一项：$(grep -h '✗' "$tmp"/missing-*.err | tr '\n' ' ')"
elif grep -qE '^(row_driver|build|run|take)' "$events"; then red missing-prerequisite "缺前置还起了：$(grep -E '^(row_driver|build|run|take)' "$events" | head -n 3 | tr '\n' ' ')"
else green missing-prerequisite; fi

# named-equals-all 与 second-run-takes：三趟（逐项点名、依次全做、单起「项 prerequisites」那一片）都跑完再判这两格，三趟里 prerequisites 打的键都收着（文件头「格」）
: >"$events"
: >"$tmp/named-runs.err"
named_failures=""
while IFS= read -r item; do
  code="$(shard_run "$tmp/named" --root "$tmp/named-root" --step8-slices 2 --f4-slices 2 --item "$item")"
  cat "$tmp/named.err" >>"$tmp/named-runs.err"
  [[ "$code" == 0 ]] || named_failures+="$item 退 $code；"
done < <(cut -f1 "$tmp/list-small.out")
cp "$events" "$tmp/named-events.log"
if [[ "$break_item" == prerequisites-key-moved-midway ]]; then
  fake_entries="$tree/scripts/experiment-apparatus/long_run_reuse_fake_entries.py"
  sed -i 's/^    return 1$/    return 3/' "$fake_entries"
  grep -qx '    return 3' "$fake_entries" || { echo "弄坏开关 $break_item 没改动小树里的假装置入口 $fake_entries" >&2; exit 2; }
fi
code_all="$(shard_run "$tmp/all" --root "$tmp/all-root" --step8-slices 2 --f4-slices 2)"
listing_of() { (cd "$1" && find . -path './work-*' -prune -o -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum); }
named_listing="$(listing_of "$tmp/named-root")"
all_listing="$(listing_of "$tmp/all-root")"
: >"$events"
mkdir -p "$tmp/second-plan"
printf '项 prerequisites\titem\tprerequisites\n' >"$tmp/second-plan/shards.tsv"
code_second="$(TASK_SERVER_SHARD_PHASE=shard TASK_SERVER_SHARD_PLAN_DIRECTORY="$tmp/second-plan" TASK_SERVER_SHARD=0/1 TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$tmp/second-output" \
  shard_run "$tmp/second" --root "$tmp/named-root" --item prerequisites)"
events_second="$(event_count)"
prerequisites_keys() { # <标准错误文件…>：这几份里共用前置 prerequisites 打的键（E171_SHARED_STEP item=prerequisites taken=… key=<前 12 位>），按先后去重
  sed -n 's/^E171_SHARED_STEP item=prerequisites taken=[a-z]* key=\([^ ]*\).*$/\1/p' "$@" | awk '!seen[$0]++'
}
keys_seen="$(prerequisites_keys "$tmp/named-runs.err" "$tmp/all.err" "$tmp/second.err")"
key_moved=""
key_count="$(grep -c . <<<"$keys_seen")"
if [[ "$key_count" -gt 1 ]]; then
  key_moved="键在这几趟里变过：共用前置 prerequisites 依次是 $(paste -sd' ' <<<"$keys_seen")（判法闭包照本仓算，自证跑着的时候本仓里闭包碰到的源码或小树里的装置入口被改了，"
  key_moved+="库里早先那一条对不上），这一格的结局不算数；→ 等改动定下来再起同一条命令"
fi

# named-equals-all
if [[ -n "$key_moved" ]]; then red named-equals-all "$key_moved"
elif [[ -n "$named_failures" || "$code_all" != 0 ]]; then red named-equals-all "逐项点名：${named_failures:-都退 0}；依次全做退 $code_all：$(tail -n 2 "$tmp/all.err" | tr '\n' ' ')"
elif [[ -z "$named_listing" || "$named_listing" != "$all_listing" ]]; then
  red named-equals-all "两个根的文件不同：$(diff <(echo "$named_listing") <(echo "$all_listing") | head -n 4 | tr '\n' ' ')"
elif [[ "$(grep -c '^E171_ROW_SHARD_ITEM ' "$tmp/all.err")" != 15 ]] || ! grep -q '^E171_ROW_SHARD_TOTAL items=15 passed=15 failed=0 failed_items=-$' "$tmp/all.err"; then
  red named-equals-all "依次全做没有每项一行与总计行：$(grep '^E171_ROW_SHARD_TOTAL' "$tmp/all.err")"
else green named-equals-all; fi

# second-run-takes
if [[ -n "$key_moved" ]]; then red second-run-takes "$key_moved"
elif [[ "$code_second" != 0 ]]; then red second-run-takes "退 $code_second：$(tail -n 2 "$tmp/second.err" | tr '\n' ' ')"
elif ! grep -q 'E171_SHARED_STEP item=prerequisites taken=yes' "$tmp/second.err"; then red second-run-takes "标准错误没有 taken=yes"
elif [[ ! -f "$tmp/second-output/shard.done" ]]; then red second-run-takes "那一片退 0 却没在产物目录写 shard.done"
elif [[ "$events_second" != 0 ]]; then red second-run-takes "再起一片又起了装置或领卡：$(head -n 3 "$events" | tr '\n' ' ')"
else green second-run-takes; fi

# slice-arguments
slice_line="$(grep '^row_driver .* output=step8 .* rows=6,7,8,9,10$' "$tmp/named-events.log" || true)"
if [[ -z "$slice_line" ]]; then red slice-arguments "没有判第 6–10 行的那一片逐行驱动：$(grep '^row_driver' "$tmp/named-events.log" | grep -m2 step8 | tr '\n' ' ')"
elif [[ "$slice_line" != "row_driver work=work "* ]]; then red slice-arguments "那一片的工作根不是服务给的产物目录下的 work：$slice_line"
elif [[ "$slice_line" != *"--rows"* || "$slice_line" != *"--keep-compile-failures-as-they-are"* || "$slice_line" != *"--lease-card-per-device-run"* ]]; then red slice-arguments "缺参数：$slice_line"
else green slice-arguments; fi

# card-around-device
held=""; records=0; judges=0; problem=""
while IFS= read -r event; do
  case "$event" in
    "take "*) [[ -z "$held" ]] || problem+="又领了一张：$event；"; held="${event#take }" ;;
    "return "*) [[ "${event#return }" == "$held" ]] || problem+="还的不是领到的：$event；"; held="" ;;
    build) [[ -z "$held" ]] || problem+="编的时候持着 $held；" ;;
    "run e171_three_oracle_collusion "*" stage=record "*) [[ -z "$held" && "$event" == *" cards=none" ]] || problem+="录入段持着卡：$event；"; records=$((records + 1)) ;;
    "run e171_three_oracle_collusion "*" stage=judge "*) [[ -n "$held" && "${event##*cards=}" == "$held" ]] || problem+="判段没拿领到的卡：$event；"; judges=$((judges + 1)) ;;
    "run e171_three_oracle_collusion "*) problem+="装置没分录入、判两段起：$event；" ;;
    "run e170_specification_executor_export "*) [[ -z "$held" && "$event" == *"cards=none" ]] || problem+="导出装置拿了卡：$event；" ;;
  esac
done < <(sed -n '1,/^executor_main /p' "$tmp/named-events.log")
if [[ -n "$problem" || "$records" != 5 || "$judges" != 5 ]]; then red card-around-device "prerequisites 里装置录入段 $records 次、判段 $judges 次（各要 5）；${problem:-}"
else green card-around-device; fi

# no-reuse-from
code="$(shard_run "$tmp/reuse" --root "$tmp/reuse-root" --reuse-rows-from "$tmp/named-root" --step8-slices 2 --item step8:1/2)"
reuse_lines="$(grep '^row_driver ' "$tmp/named-events.log" | grep -e '--reuse-from' || true)"
if [[ -n "$reuse_lines" ]]; then red no-reuse-from "逐行驱动带了 --reuse-from：$(head -n 2 <<<"$reuse_lines" | tr '\n' ' ')"
elif [[ "$code" != 2 ]]; then red no-reuse-from "--reuse-rows-from 退 $code（要 2：认不得的参数）"
else green no-reuse-from; fi

# header
header="$(head -n 60 "$shard")"
if ! grep -qx '# task-server-card-lease: on-demand' <<<"$header" || ! grep -qx '# task-server-compiles: yes' <<<"$header" \
  || ! grep -q '^# task-server-shards: \* ' <<<"$header" || grep -q '^# task-server-gpu-arguments:' <<<"$header"; then
  red header "文件头不是按需领卡、会编、一律分片：$(sed -n 4,6p "$shard" | tr '\n' ' ')"
else green header; fi

# sharded-through-a-fake-service：这里当假服务（task-server/task_server_lib.py 文件头「分片」），换一份空的假库与共用前置的命名空间
sharded_root="$tmp/sharded-root"
service="$tmp/service"
service_entry_arguments=()
service_step() { # <记号> <阶段> <计划目录> [变量=值…]：照分片协议设变量、在小树根上起分片入口这一步，标准输出与标准错误进 $tmp/sharded-<记号>.out|.err，打退出码
  local mark="$1" phase="$2" plan="$3"
  shift 3
  (cd "$tree" && export TASK_SERVER_SHARD_PHASE="$phase" TASK_SERVER_SHARD_PLAN_DIRECTORY="$plan" LONG_RUN_ITEMS_LIBRARY_DIRECTORY="$tmp/library-sharded" \
     FAKE_ROW_LIBRARY="$tmp/row-library-sharded" "$@" \
     && bash scripts/experiment-apparatus/e171-row-shard.sh --root "$sharded_root" --step8-slices 2 --f4-slices 2 ${service_entry_arguments[@]+"${service_entry_arguments[@]}"}) \
    >"$tmp/sharded-$mark.out" 2>"$tmp/sharded-$mark.err"
  echo $?
}
shards_of() { cut -f2,3 "$1/shards.tsv" 2>/dev/null; }   # 计划写的每一片：<种类><TAB><做什么>（shards.tsv 不在时空着，后面照「片不对」判）
mkdir -p "$service/plan-small" "$service/plan" "$service/plan-again" "$service/plan-audit"
: >"$events"
sharded_problem=""
expected_small="item	prerequisites
item	positive-control:O1-对拍
item	positive-control:O1-崩溃判官
item	positive-control:O2
item	empty-mutation
rows	step8 1,2,3,4,5,6,7,8,9,10
rows	f4 11,12,13,14"
code="$(service_step small plan "$service/plan-small" TASK_SERVER_SHARD_MAXIMUM=7)"
[[ "$code" == 0 && "$(shards_of "$service/plan-small")" == "$expected_small" ]] || sharded_problem+="上限 7 的计划退 $code、片是「$(shards_of "$service/plan-small" | tr '\n\t' '; ')」；"
expected_plan="item	prerequisites
item	positive-control:O1-对拍
item	positive-control:O1-崩溃判官
item	positive-control:O2
item	empty-mutation
rows	step8 1,2,3,4,5
rows	step8 6,7,8,9,10
rows	f4 11,12,13,14"
code="$(service_step plan plan "$service/plan" TASK_SERVER_SHARD_MAXIMUM=64)"
shard_count="$(grep -c . "$service/plan/shards.tsv" 2>/dev/null || true)"
if [[ "$code" != 0 || "$(shards_of "$service/plan")" != "$expected_plan" ]]; then
  sharded_problem+="上限 64 的计划退 $code、片是「$(shards_of "$service/plan" | tr '\n\t' '; ')」；"
elif [[ "$(grep -o '"label"' "$service/plan/plan.json" | grep -c .)" != 8 ]]; then
  sharded_problem+="plan.json 不是 8 个 label；"
fi
: >"$service/outputs.tsv"
for ((index = 0; index < ${shard_count:-0}; index++)); do
  code="$(service_step "shard-$index" shard "$service/plan" TASK_SERVER_SHARD="$index/$shard_count" TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$service/output-$index")"
  if [[ "$code" == 0 && -f "$service/output-$index/shard.done" ]]; then
    printf '%s\t127.0.0.1\t%s\n' "$index" "$service/output-$index" >>"$service/outputs.tsv"
  else
    sharded_problem+="第 $index 片退 $code（要 0、产物目录有 shard.done）：$(tail -n 2 "$tmp/sharded-shard-$index.err" | tr '\n' ' ')；"
  fi
done
code="$(service_step merge merge "$service/plan" TASK_SERVER_SHARD_OUTPUTS="$service/outputs.tsv" TASK_SERVER_SHARD_MISSING=)"
[[ "$code" == 0 ]] && grep -q '^E171_ROW_SHARD_TOTAL items=15 passed=15 failed=0 failed_items=-$' "$tmp/sharded-merge.err" \
  || sharded_problem+="合并退 $code（要 0、15 项全过）：$(grep -m3 -E '^E171_ROW_SHARD_(MISSING|TOTAL)' "$tmp/sharded-merge.err" | tr '\n' ' ')；"
code="$(service_step again plan "$service/plan-again" TASK_SERVER_SHARD_MAXIMUM=64)"
[[ "$code" == 0 && -f "$service/plan-again/shards.tsv" && ! -s "$service/plan-again/shards.tsv" ]] \
  || sharded_problem+="都判过之后再起计划退 $code、片是「$(shards_of "$service/plan-again" | tr '\n\t' '; ')」（要 0 片）；"
code="$(service_step missing-shard merge "$service/plan" TASK_SERVER_SHARD_OUTPUTS="$service/outputs.tsv" TASK_SERVER_SHARD_MISSING=3)"
[[ "$code" == 1 ]] && grep -q '缺片：3' "$tmp/sharded-missing-shard.err" || sharded_problem+="缺片 3 的合并退 $code（要 1、点名缺片 3）；"
rm -f "$tmp/row-library-sharded/row-12.all-oracles.executor.no-control"
: >"$service/plan-audit/shards.tsv"
service_entry_arguments=(--item f4:1/2)
code="$(service_step audit merge "$service/plan-audit" TASK_SERVER_SHARD_OUTPUTS="$service/outputs.tsv" TASK_SERVER_SHARD_MISSING=)"
service_entry_arguments=()
[[ "$code" != 0 ]] && grep -q '^E171_ROW_SHARD_MISSING item=f4:1/2 row=12 ' "$tmp/sharded-audit.err" \
  || sharded_problem+="假库里缺第 12 行时点 f4:1/2 的合并退 $code（要非 0、点名 item=f4:1/2 row=12）：$(grep -m2 '^E171_ROW_SHARD_' "$tmp/sharded-audit.err" | tr '\n' ' ')；"
if [[ -n "$sharded_problem" ]]; then red sharded-through-a-fake-service "$sharded_problem"; else green sharded-through-a-fake-service; fi

if (( fail == 0 )); then echo "GREEN e171-row-shard-selftest 10 格全过"; else echo "RED e171-row-shard-selftest 有格红（弄坏开关：${break_item:-无}）"; fi
exit $fail
