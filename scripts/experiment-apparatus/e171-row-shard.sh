#!/usr/bin/env bash
# admission: always 每一次调都现查判定库：逐行驱动按行、共用前置按项各经 long_run_item_verdicts 照此刻的键取判过的，上一次的结论不替这一次作保
# run-condition: command python3 rsync cargo
# task-server-card-lease: on-demand
# task-server-compiles: yes
# task-server-shards: *   经统一任务服务交的一律分片：服务拆成计划、各片、合并三步（下面「分片」；task-server/task_server_lib.py 文件头「分片」）；--list-items 服务不分
# E171（三oracle合谋测试）整表那几行（scripts/experiment-apparatus/replay.sh 的 E171 第 7、9、10、11 行：s2、s4、扩表、补位）要的项与行，判进判定库只在这一份入口里做，
# 而且只在服务分片的三步里做（下面「分片」）；replay.sh 那四行只核账（e171_three_oracle_collusion/shared_steps.py 的 take 与 take-rows：库里有键相同、判绿的才取，
# 缺的点名这一份入口的整条命令，一行都不现判）。
#
#   task-server/task submit -- bash scripts/experiment-apparatus/e171-row-shard.sh --root <共用输出根> [--step8-slices <片数>] [--f4-slices <片数>]
#        [--list-items | --item <项>…]
#   在快照树或主工作区的根目录起（--repo-root 取此刻的目录；领卡客户端是这棵树里的 task-server/task）。
#
# 入口（.claude/rules/test-script-discipline.md「入口脚本：不带参数就是不指定范围」）：
#   不带 --item           全部项：计划把库里没有的独立项与表的行拆成片，合并做依赖前面的项、再核全部项与两张表每一行的账
#   --list-items          逐行打可点名的项：<项><TAB><要先有哪几项><TAB><做哪一段>，不起编译、不跑
#   --item <项>           只做点名的项，给几次取并集：计划只拆点到的独立项与点到的表区间，合并只做、只核点到的项；
#                         表的片写成 step8:<第几片>/<共几片>、f4:<第几片>/<共几片>，区间由 row_driver.py --print-shard-ranges <共几片> --shards-may-cut-groups 现算
#                         （列出来的是缺省片数 step8 8 片、f4 12 片，--step8-slices、--f4-slices 换缺省片数；这几个片数只管点名，计划的片照下面「分片」切）
#   项名写错、--item 后面缺项名：列出可点名的项，退 2
# 共用输出根：--root 给，要绝对路径（两台同一路径，多机配置 NFS_SHARED_ROOTS 里的共享根下；replay.sh 那四行的出路句写工作根下的 e171-row-shards）。
# 判定库：逐行驱动的每一行、共用前置的每一项各是一条判定，经 scripts/experiment-apparatus/long_run_item_verdicts.py 存进 prune-server 那一份库
#   （row_driver.py、shared_steps.py 文件头「判定库」）；各片判的行写进同一份库，补位与 Q3-核那几项的逐行驱动从库里取键相同、判绿的行，
#   共用输出根里的行目录与项目录只是这一趟的产物、不当复用来源。中途停下重交同一条命令，判过的从库里复用，只补没判的。
# 项（次序即依赖次序；合并里做的项开跑前核前置，没有的点名缺哪一项、退 4，一步都不做）：
#   prerequisites                   共用前置：基线两趟、E170 导出、已知坏镜像、关 O1 与关 O2 的基线、O3 判官与执行器（shared_steps.py 文件头）
#   positive-control:O1-对拍 / positive-control:O1-崩溃判官 / positive-control:O2   阳性对照的装置基线与只跑空变异的那一行
#   positive-control:O3             O3 阳性对照（要先有 prerequisites：库里有它）
#   empty-mutation                  s2 整表的空变异那一行（row-0；整表输出根 <根>/empty-mutation）
#   step8:<i>/<n>                   步长 8 那张表（115 行）的第 i 片，输出根 <根>/step8，编不过的行记编不过、不补位（--keep-compile-failures-as-they-are）
#   f4:<i>/<n>                      扩表（163 行）的第 i 片，输出根 <根>/f4，同上，补位跳过 step8 的名字
#   s2-substitutes                  s2 整表的补位（要先有 step8 每一行）：步长 8 的表、补位只跳过这张表的名字，<根>/s2-substitutes
#   substitutes                     补位那一行（第 11 行）的补位（要先有 step8 每一行）：补位跳过两张表的名字，<根>/substitutes
#   q3core:O1 / q3core:O2 / q3core:no-executor   Q3-核三趟（要先有 prerequisites 与 step8 每一行）：行由 comparator.py --print-q3-core-rows 在 <根>/step8 与
#                                   prerequisites 的 baseline-run1 上现算，加上 s2 那一趟写死的第 8 行；输出根 <根>/q3core-<O1|O2|no-executor>
#   两种补位分开放：补位挑的行跟着补位跳过的那几张表变，s2 与补位那一行各写自己的输出根（replay.sh 里各自点名）；补位换了行的结局不入库。
# 分片（经服务交时，服务照 TASK_SERVER_SHARD_PHASE 把这一份起三次；三次都从头走）：
#   计划  TASK_SERVER_SHARD_PHASE=plan：点到的项里互不依赖的照键查库——prerequisites 与三条不依赖前置的阳性对照（shared_steps.py fetch：库里有就解进共用输出根）、
#         empty-mutation 与两张表的行（shared_steps.py take-rows --check-only：只查不写）；库里已有键相同、判绿的项与行不进片。要现判的项一项一片；
#         两张表各自把缺的行按表里的次序切成连续的几片，每片至多 ROWS_PER_SLICE_AT_MOST（8）行、各片行数最多差 1（片数 = 缺的行数 ÷ 8 向上取整）。
#         总片数超过服务给的 TASK_SERVER_SHARD_MAXIMUM 时，项一片不并，两张表的每片行数从 8 起一行一行往上加，到「项的片数 + 两张表的片数」不超过上限为止
#         （项的片数本身就超过上限的不切，退 1、点名）。依赖前面的项（positive-control:O3、s2-substitutes、substitutes、q3core 三趟）不进片，归合并。
#         计划目录里写 shards.tsv（一片一行：<片的说明><TAB>item<TAB><项> 或 <片的说明><TAB>rows<TAB><step8|f4> <逗号连的序号>）与 plan.json（每片只写 label）。
#   各片  TASK_SERVER_SHARD_PHASE=shard：照 TASK_SERVER_SHARD=<片号>/<片数> 取 shards.tsv 的那一行，只做这一片：项照现判那一套做（shared_steps.py run、
#         empty-mutation 起逐行驱动只跑空变异），行起逐行驱动 --rows <这几行>（编不过的记编不过、不补位：--keep-compile-failures-as-they-are；扩表另带
#         --other-selected-rows 步长 8 那张表）；判定入库的写法不变（各自经 long_run_item_verdicts）。工作根是服务给的这一片的产物目录下的 work/，这一片做完就删；
#         做成了在产物目录写 shard.done。
#   合并  TASK_SERVER_SHARD_PHASE=merge：TASK_SERVER_SHARD_MISSING 不空、或 TASK_SERVER_SHARD_OUTPUTS 里缺哪一片的 shard.done，点名缺的片、退 1、什么都不做。
#         再把要的行从库里取进共用输出根（step8：点到它的片、或点到依赖它的项时；f4：点到它的片时），做依赖前面的项（positive-control:O3、s2-substitutes、
#         substitutes、q3core 三趟；做法与前置核对同各片），最后核账：点到的每一项——共用前置五项照键在库里（fetch，解进共用输出根）、empty-mutation 与两张表
#         点到的每一行取进共用输出根、两种补位与 Q3-核三趟点到的每一行在库里（take-rows --check-only）——都有键相同、判绿的那一条才算；缺的逐个打
#         E171_ROW_SHARD_MISSING item=<项> [row=<序号>] reason=<为什么>，那一项判没做成（退出码 1），一样都不现判。
#   没有 TASK_SERVER_SHARD_PHASE（不经服务起，只给自证的假驱动用）：这一份自己当假服务，照上面的协议逐步设这个变量，在一件里依次起计划、每一片、合并
#         （计划目录与各片产物目录在 <共用输出根>/.direct-run-* 下、跑完删；片数上限取 DIRECT_RUN_SHARD_MAXIMUM，同服务的 MAXIMUM_SHARDS），同一套代码、
#         不另有不分片的路；有一片没成就不起合并、退那一片的码。真装置的判行不走这条路：经服务交的一律分片（文件头第 6 行）。
# 各项的参数（线道 2 条、每条 2 线程，表与补位跳过的名字）与 replay.sh 那几行核账时取的行同键（键不带线道、线程与补位跳过的名字，row_driver.py 文件头「判定库」）；
#   只跑空变异的两项用 1 条线道。
# 卡（文件头的按需领卡声明，task-server/task_server_lib.py 文件头「GPU」）：服务起跑不分卡、不清场。多机配置开了 GPU（ENABLE_GPU=1，读法照 replay.sh：
#   scripts/crash/crash-amplification-lib.sh）时逐行驱动带 --lease-card-per-device-run、共用前置带 --device-features gpu，装置都分录入、判两段起：录入段不持卡，录入段退 0 才领一张卡起判段、判段跑完还
#   （e171_three_oracle_collusion/device_card_lease.py 文件头「装置分录入、判两段起」）；
#   没开 GPU 不领卡、共用前置带 --device-features none。编译目录：文件头的会编声明让服务给 CARGO_TARGET_DIR，共用前置照它编；逐行驱动的线道在各自的工作根里
#   （各片在服务给的产物目录下，合并做的项在 <根>/work-<项>-<随机串>，做完删）。
# 输出（都进标准错误）：计划每项一行 E171_ROW_SHARD_PLAN item=<项> in_library=yes|no、每张表一行 E171_ROW_SHARD_PLAN table=<表> rows=<点到几行> missing=<缺几行>，
#   末行 E171_ROW_SHARD_PLAN shards=<几片> item_shards=<几片> rows_per_slice=<每片至多几行> maximum=<上限>；各片一行 E171_ROW_SHARD_SLICE index=<片号>/<片数> exit=<码> label=<说明>；
#   合并每项一行 E171_ROW_SHARD_ITEM item=<项> exit=<退出码> missing=<缺几样> seconds=<秒>，末行 E171_ROW_SHARD_TOTAL items=<几项> passed=<几项> failed=<几项> failed_items=<…|->；
#   复用了多少看各项自己的那一行（逐行驱动与共用前置标准错误的 LONG_RUN_ITEMS … 现判 N、复用 M、拒算 R，共用前置的 E171_SHARED_STEP … taken=yes|no）。
# 退出码：0 点到的项都做完、核账都在；1 合并核账有缺、缺片、计划或片读不通；2 参数不对、项名写错、区间被驱动拒、服务的分片变量缺或认不出；
#   4 合并里做的项缺前置；78 多机配置读不出、开了 GPU 而没有领卡通道；
#   别的是没做成的那一步（计划查库、那一片、合并里次序最靠前的没做成的项）的退出码（row_driver.py：5 = 测量管子的缺陷，含装置输出里有 NOT_RUN、工具链缺失、
#   领卡没成；shared_steps.py：1 = 有一步退非 0，75 = prune-server 连不上）。
# 自证：python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e171-row-shard-selftest.sh [--break <项>]
#   （假逐行驱动、假 cargo、假领卡客户端，真的 shared_steps.py；共用前置的判定写进临时 prune-server）。
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

research="scripts/experiment-apparatus/e171_three_oracle_collusion"
driver="$research/row_driver.py"
steps="$research/shared_steps.py"
comparator="$research/comparator.py"
step8_table="$research/bad-implementations-step-8.tsv"
f4_table="$research/bad-implementations-step-4-only.tsv"
Q3_CORE_ROW_OF_THE_S2_RUN=8   # replay.sh 的 driver_e171_s2_comparator 三趟 Q3-核写死的那一行
ROWS_PER_SLICE_AT_MOST=8      # 计划切表的一片至多几行（文件头「分片」；量的：任务 15262 等每片 7 行本机约 900 秒，第二台约慢五倍，一片要在两小时内）
DIRECT_RUN_SHARD_MAXIMUM=64   # 不经服务起时当假服务给的片数上限（同 task-server/task_server_lib.py 的 MAXIMUM_SHARDS）
step8_slices=8
f4_slices=12

usage() {
  echo "用法：bash scripts/experiment-apparatus/e171-row-shard.sh [--root <共用输出根>] [--step8-slices <片数>] [--f4-slices <片数>] [--list-items | --item <项>…]" >&2
}

list_items() {
  local index
  printf '%s\t%s\t%s\n' prerequisites - "基线两趟、E170 导出、已知坏镜像、关 O1 与关 O2 的基线、O3 判官与执行器"
  printf '%s\t%s\t%s\n' "positive-control:O1-对拍" - "阳性对照 O1-对拍：装置基线一趟、逐行驱动只跑空变异一趟"
  printf '%s\t%s\t%s\n' "positive-control:O1-崩溃判官" - "阳性对照 O1-崩溃判官：装置基线一趟、逐行驱动只跑空变异一趟"
  printf '%s\t%s\t%s\n' "positive-control:O2" - "阳性对照 O2：装置基线一趟、逐行驱动只跑空变异一趟"
  printf '%s\t%s\t%s\n' "positive-control:O3" prerequisites "阳性对照 O3：逐行驱动只跑空变异，带 prerequisites 的 E170 bad-states 导出"
  printf '%s\t%s\t%s\n' empty-mutation - "s2 整表的空变异那一行（row-0）"
  for ((index = 1; index <= step8_slices; index++)); do printf '%s\t%s\t%s\n' "step8:$index/$step8_slices" - "步长 8 那张表的第 $index 片（共 $step8_slices 片）"; done
  for ((index = 1; index <= f4_slices; index++)); do printf '%s\t%s\t%s\n' "f4:$index/$f4_slices" - "扩表的第 $index 片（共 $f4_slices 片）"; done
  printf '%s\t%s\t%s\n' s2-substitutes "step8:*" "s2 整表编不过的行补位（只跳过步长 8 那张表的名字）"
  printf '%s\t%s\t%s\n' substitutes "step8:*" "补位那一行编不过的行补位（跳过两张表的名字）"
  printf '%s\t%s\t%s\n' "q3core:O1" "prerequisites,step8:*" "Q3-核关 O1 那一趟"
  printf '%s\t%s\t%s\n' "q3core:O2" "prerequisites,step8:*" "Q3-核关 O2 那一趟"
  printf '%s\t%s\t%s\n' "q3core:no-executor" "prerequisites,step8:*" "Q3-核驱动不跑执行器那一趟"
}

# 项在依赖次序里排第几（--item 给的几项照它排）；不认得的项打空串
item_rank() {
  case "$1" in
    prerequisites) echo 0 ;;
    "positive-control:O1-对拍") echo 1 ;;
    "positive-control:O1-崩溃判官") echo 2 ;;
    "positive-control:O2") echo 3 ;;
    "positive-control:O3") echo 4 ;;
    empty-mutation) echo 5 ;;
    step8:*|f4:*)
      if [[ "$1" =~ ^(step8|f4):([1-9][0-9]*)/([1-9][0-9]*)$ ]] && (( BASH_REMATCH[2] <= BASH_REMATCH[3] )); then
        if [[ "${BASH_REMATCH[1]}" == step8 ]]; then echo 6; else echo 7; fi
      fi ;;
    s2-substitutes) echo 8 ;;
    substitutes) echo 9 ;;
    "q3core:O1") echo 10 ;;
    "q3core:O2") echo 11 ;;
    "q3core:no-executor") echo 12 ;;
  esac
}

refuse_unknown_item() { # <写错的项名或空>
  if [[ -n "$1" ]]; then echo "  ✗ 认不得的项「$1」" >&2; else echo "  ✗ --item 后面缺项名" >&2; fi
  echo "  可点名的项（--list-items 打的项名）：" >&2
  list_items | cut -f1 | sed 's/^/    /' >&2
  usage
  exit 2
}

root=""
wanted=()
listing=0
while (($#)); do
  case "$1" in
    --root) [[ $# -ge 2 ]] || { usage; exit 2; }; root="$2"; shift 2 ;;
    --step8-slices) [[ "${2:-}" =~ ^[1-9][0-9]*$ ]] || { usage; exit 2; }; step8_slices="$2"; shift 2 ;;
    --f4-slices) [[ "${2:-}" =~ ^[1-9][0-9]*$ ]] || { usage; exit 2; }; f4_slices="$2"; shift 2 ;;
    --list-items) listing=1; shift ;;
    --item) [[ $# -ge 2 && -n "$2" && "$2" != --* ]] || refuse_unknown_item ""; [[ -n "$(item_rank "$2")" ]] || refuse_unknown_item "$2"; wanted+=("$2"); shift 2 ;;
    *) echo "  ✗ 认不得的参数「$1」" >&2; usage; exit 2 ;;
  esac
done
if (( listing )); then
  (( ${#wanted[@]} == 0 )) || { echo "  ✗ --list-items 不与 --item 一起给" >&2; usage; exit 2; }
  list_items
  exit 0
fi
[[ -n "$root" && "$root" == /* ]] || { echo "  ✗ 共用输出根要给绝对路径（--root，两台同一路径）" >&2; echo "  → 怎么办：加 --root <工作根>/e171-row-shards" >&2; usage; exit 2; }
[[ -f "$driver" && -f "$steps" ]] || { echo "  ✗ 不在快照树的根目录：找不到 $driver 或 $steps" >&2; echo "  → 怎么办：在快照树或主工作区的根目录起" >&2; exit 2; }
everything=0   # 不带 --item：全部项，计划与合并里两张表都按整张表查（文件头「分片」）
if (( ${#wanted[@]} == 0 )); then
  everything=1
  mapfile -t wanted < <(list_items | cut -f1)
fi
mapfile -t ordered < <(for item in "${wanted[@]}"; do printf '%s\t%s\n' "$(item_rank "$item")" "$item"; done | sort -t $'\t' -k1,1n -s | cut -f2 | awk '!seen[$0]++')

# 多机配置开没开 GPU（读法照 replay.sh 的 read_gpu_switch，共用库只 source 不抄）：打 gpu 或 none；读不出打 ✗ 与出路、返回 78
device_features() {
  local configuration_file switch
  # shellcheck source=/dev/null
  source scripts/crash/crash-amplification-lib.sh || { echo "  ✗ 读不了 scripts/crash/crash-amplification-lib.sh（读多机配置的共用库）" >&2; return 78; }
  configuration_file="$(crash_amplification_multi_host_configuration_file "$PWD")"
  if [[ ! -e "$configuration_file" ]]; then
    if [[ -n "${SINGLEFS_MULTI_HOST_CONFIG:-}" ]]; then
      echo "  ✗ SINGLEFS_MULTI_HOST_CONFIG 点名的多机配置 $configuration_file 不在，读不出 ENABLE_GPU；一项都没做" >&2
      echo "  → 怎么办：指到在的文件，或不设它（取主工作树根的 multi-host.env；那份不在就是没开 GPU）" >&2
      return 78
    fi
    echo none
    return 0
  fi
  switch="$(crash_amplification_configuration_value "$configuration_file" ENABLE_GPU)" || { echo "  ✗ 读多机配置 $configuration_file 的 ENABLE_GPU 出错" >&2; return 78; }
  case "$switch" in
    ""|0) echo none ;;
    1) echo gpu ;;
    *) echo "  ✗ 多机配置 $configuration_file 的 ENABLE_GPU 写成了「$switch」，只认 0、1；一项都没做" >&2; return 78 ;;
  esac
}
if ! features="$(device_features)"; then exit 78; fi
card_lease=()
[[ "$features" != gpu ]] || card_lease=(--lease-card-per-device-run)

# 单做一项、整轮核账的起法（出路句里用）：都经服务交
remake() { echo "task-server/task submit -- bash scripts/experiment-apparatus/e171-row-shard.sh --root $root --item $1"; }
remake_all() { echo "task-server/task submit -- bash scripts/experiment-apparatus/e171-row-shard.sh --root $root"; }

# 前置核对（项名，要先有的那一样）：没有就打 ✗ 与出路，返回 4
step8_rows_done() { # <要它的项>
  PYTHONDONTWRITEBYTECODE=1 python3 "$steps" rows-done --selected-rows "$step8_table" --rows-root "$root/step8" && return 0
  echo "  ✗ E171 整表项 $1：要先有 step8 每一行（合并从库里取进 $root/step8 的还缺行，见上一行 E171_ROWS_MISSING）；这一项一步都没做" >&2
  echo "  → 怎么办：经服务交不带 --item 的整条（计划把库里没有的行拆成片）：$(remake_all)，再做这一项" >&2
  return 4
}
prerequisites_done() { # <要它的项>
  PYTHONDONTWRITEBYTECODE=1 python3 "$steps" fetch prerequisites --root "$root" --repo-root "$PWD" --device-features "$features" && return 0
  echo "  ✗ E171 整表项 $1：要先有 prerequisites（库里没有，见上一行 E171_SHARED_STEP_MISSING）；这一项一步都没做" >&2
  echo "  → 怎么办：先做 $(remake prerequisites)，再做这一项" >&2
  return 4
}

# 逐行驱动现判（各片与合并里做的项）：<输出目录名> <工作根> <逐行驱动的其余参数…>；输出根 <根>/<输出目录名>，判过的行由逐行驱动自己从判定库取
run_rows() {
  local output="$1" work="$2"
  shift 2
  PYTHONDONTWRITEBYTECODE=1 python3 "$driver" --repo-root "$PWD" --work-root "$work" --output-root "$root/$output" \
    "$@" ${card_lease[@]+"${card_lease[@]}"}
}

# 逐行驱动只取（shared_steps.py take-rows；row_driver.py 文件头「只取」）：<输出目录名> <工作根> <日志前缀> <take-rows 的其余参数…>：
# 标准错误进 <日志前缀>、标准输出进 <日志前缀>.out；交它的退出码（0 都在、4 有缺、别的是查库出错）
take_rows() {
  local output="$1" work="$2" log="$3"
  shift 3
  if PYTHONDONTWRITEBYTECODE=1 python3 "$steps" take-rows --repo-root "$PWD" --work-root "$work" --output-root "$root/$output" "$@" >"$log.out" 2>"$log"; then
    return 0
  else
    return $?
  fi
}

# take-rows 那一份标准错误里缺的行：逐行打「<序号><TAB><为什么>」。E171_ROWS_TAKEN 那一行不在、或它报的缺行数与逐行点名的条数对不上（结果抓取的完整性闸），打 ✗、返回 1
rows_missing_in() { # <take-rows 的标准错误>
  local reported listed
  reported="$(sed -n 's/^E171_ROWS_TAKEN taken=[0-9]* missing=\([0-9]*\) .*$/\1/p' "$1")"
  listed="$(grep -c '^E171_ROW_NOT_IN_THE_LIBRARY row=' "$1" || true)"
  if [[ -z "$reported" || "$reported" != "$listed" ]]; then
    echo "  ✗ E171 只取的标准错误 $1 里缺行的条数对不上（E171_ROWS_TAKEN 报 ${reported:-没有这一行}，逐行点名 $listed 行）" >&2
    echo "  → 怎么办：看那份标准错误的末尾（只取那一趟多半中途出错退了），修好再交同一条命令" >&2
    return 1
  fi
  sed -n 's/^E171_ROW_NOT_IN_THE_LIBRARY row=\([0-9]*\) reason=\(.*\)$/\1\t\2/p' "$1"
}

# 计划查库：<日志前缀> <查库的命令…>：交那条命令的退出码（0 库里有、4 库里没有、别的是查库出错）；标准错误进 <日志前缀>
library_probe() {
  local log="$1"
  shift
  if PYTHONDONTWRITEBYTECODE=1 "$@" >"$log.out" 2>"$log"; then return 0; else return $?; fi
}

# 表的第几片的区间：<step8|f4> <第几片> <共几片>，打「<起>-<止>」（row_driver.py --print-shard-ranges <共几片> --shards-may-cut-groups 现算，只按行数均分）
slice_range() {
  local table="$1" index="$2" count="$3" selected ranges line
  if [[ "$table" == step8 ]]; then selected="$step8_table"; else selected="$f4_table"; fi
  # --print-shard-ranges 只读表打区间，驱动的 --repo-root、--work-root、--output-root 是必填（argparse 先查必填）；这里给的两个路径不会被建
  ranges="$(PYTHONDONTWRITEBYTECODE=1 python3 "$driver" --repo-root "$PWD" --work-root "$root/work-print-ranges" --output-root "$root/$table" \
    --selected-rows "$selected" --print-shard-ranges "$count" --shards-may-cut-groups)" || return 2
  line="$(sed -n "${index}p" <<<"$ranges")"
  if [[ ! "$line" =~ low=([0-9]+)\ high=([0-9]+) ]]; then
    echo "  ✗ 表 $table 切 $count 片，没有第 $index 片（打出的区间：$ranges）" >&2
    echo "  → 怎么办：照 --list-items 列的片点名" >&2
    return 2
  fi
  echo "${BASH_REMATCH[1]}-${BASH_REMATCH[2]}"
}

# 点到的项里依赖前面各片的那几样（合并做）、表的片
is_merge_item() { case "$1" in positive-control:O3|s2-substitutes|substitutes|q3core:*) return 0 ;; *) return 1 ;; esac; }
table_of_table_item() { case "$1" in step8:*) echo step8 ;; f4:*) echo f4 ;; esac; }

# 一张表这一趟要的几段（一行一段：- 是整张表，别的是 <起>-<止>）：不带 --item 是整张表；点到依赖 step8 的项时 step8 是整张表；别的照点到的片现算区间
table_parts() { # <step8|f4> <合并里依赖它的项算不算：yes|no>
  local table="$1" with_dependents="$2" item
  if (( everything )); then echo -; return 0; fi
  if [[ "$table" == step8 && "$with_dependents" == yes ]]; then
    for item in "${ordered[@]}"; do
      case "$item" in s2-substitutes|substitutes|q3core:*) echo -; return 0 ;; esac
    done
  fi
  for item in "${ordered[@]}"; do
    [[ "$(table_of_table_item "$item")" == "$table" ]] || continue
    [[ "$item" =~ ^(step8|f4):([0-9]+)/([0-9]+)$ ]]
    slice_range "$table" "${BASH_REMATCH[2]}" "${BASH_REMATCH[3]}" || return $?
  done
}

# 一张表缺的行切成几片：<缺的行数> <每片至多几行> 打片数
slice_count_of() { echo $(( ($1 + $2 - 1) / $2 )); }

# 计划（文件头「分片」计划）：<计划目录> <片数上限>
plan_phase() {
  local plan_directory="$1" maximum="$2" item table part code=0 selected rows_per_slice item_count index slice_count low high label nonempty_tables=0 log parts_text points
  local -a item_shards=() parts=() rows=() slice=() range_arguments=()
  local -A missing_count=()
  for item in "${ordered[@]}"; do
    log="$plan_directory/probe-${item//:/-}.err"
    case "$item" in
      prerequisites|positive-control:O1-对拍|positive-control:O1-崩溃判官|positive-control:O2)
        if library_probe "$log" python3 "$steps" fetch "$item" --root "$root" --repo-root "$PWD" --device-features "$features"; then code=0; else code=$?; fi ;;
      empty-mutation)
        if library_probe "$log" python3 "$steps" take-rows --repo-root "$PWD" --work-root "$plan_directory/work" \
             --output-root "$root/empty-mutation" --empty-mutation-only --check-only; then code=0; else code=$?; fi ;;
      *) continue ;;
    esac
    case "$code" in
      0) echo "E171_ROW_SHARD_PLAN item=$item in_library=yes" >&2 ;;
      4) echo "E171_ROW_SHARD_PLAN item=$item in_library=no" >&2; item_shards+=("$item") ;;
      *) cat "$log" >&2; echo "  ✗ E171 计划：查 $item 在不在库里那一步退 $code；计划没写" >&2; echo "  → 怎么办：照上面那几行的出路办，再交同一条命令" >&2; return "$code" ;;
    esac
  done
  for table in step8 f4; do
    if [[ "$table" == step8 ]]; then selected="$step8_table"; else selected="$f4_table"; fi
    : >"$plan_directory/missing-$table.tsv"
    parts_text="$(table_parts "$table" no)" || return $?
    parts=()
    [[ -z "$parts_text" ]] || mapfile -t parts <<<"$parts_text"
    points=0
    for part in ${parts[@]+"${parts[@]}"}; do
      range_arguments=()
      [[ "$part" == - ]] || range_arguments=(--row-range "$part")
      if library_probe "$plan_directory/$table.err" python3 "$steps" take-rows --repo-root "$PWD" --work-root "$plan_directory/work" --output-root "$root/$table" \
           --selected-rows "$selected" ${range_arguments[@]+"${range_arguments[@]}"} --check-only; then code=0; else code=$?; fi
      if [[ "$code" != 0 && "$code" != 4 ]]; then
        cat "$plan_directory/$table.err" >&2
        echo "  ✗ E171 计划：查表 $table（${part/#-/整张表}）的行在不在库里那一步退 $code；计划没写" >&2
        echo "  → 怎么办：照上面那几行的出路办，再交同一条命令" >&2
        return "$code"
      fi
      rows_missing_in "$plan_directory/$table.err" >>"$plan_directory/missing-$table.tsv" || return 1
      points=$(( points + $(sed -n 's/^E171_ROWS_TAKEN .* of=\([0-9]*\) .*$/\1/p' "$plan_directory/$table.err") ))
    done
    missing_count[$table]="$(cut -f1 "$plan_directory/missing-$table.tsv" | sort -n -u | grep -c . || true)"
    (( ${#parts[@]} == 0 )) || echo "E171_ROW_SHARD_PLAN table=$table rows=$points missing=${missing_count[$table]}" >&2
    (( ${missing_count[$table]} == 0 )) || nonempty_tables=$((nonempty_tables + 1))
  done
  item_count=${#item_shards[@]}
  if (( item_count + nonempty_tables > maximum )); then
    echo "  ✗ E171 计划：要现判的项 $item_count 项、缺行的表 $nonempty_tables 张，至少要 $((item_count + nonempty_tables)) 片，超过服务给的上限 $maximum 片；计划没写" >&2
    echo "  → 怎么办：用 --item 分几次交（每次点几项，--list-items 列可点的项）" >&2
    return 1
  fi
  rows_per_slice=$ROWS_PER_SLICE_AT_MOST
  while (( item_count + $(slice_count_of "${missing_count[step8]}" "$rows_per_slice") + $(slice_count_of "${missing_count[f4]}" "$rows_per_slice") > maximum )); do
    rows_per_slice=$((rows_per_slice + 1))
  done
  : >"$plan_directory/shards.tsv.partial"
  for item in ${item_shards[@]+"${item_shards[@]}"}; do
    printf '%s\t%s\t%s\n' "项 $item" item "$item" >>"$plan_directory/shards.tsv.partial"
  done
  for table in step8 f4; do
    mapfile -t rows < <(cut -f1 "$plan_directory/missing-$table.tsv" | sort -n -u)
    (( ${#rows[@]} > 0 )) || continue
    slice_count="$(slice_count_of "${#rows[@]}" "$rows_per_slice")"
    for ((index = 0; index < slice_count; index++)); do
      low=$(( ${#rows[@]} * index / slice_count ))
      high=$(( ${#rows[@]} * (index + 1) / slice_count ))
      slice=("${rows[@]:low:high-low}")
      label="表 $table 的 ${#slice[@]} 行（第 ${slice[0]} 到第 ${slice[-1]} 行里库里没有的）"
      printf '%s\t%s\t%s %s\n' "$label" rows "$table" "$(IFS=,; echo "${slice[*]}")" >>"$plan_directory/shards.tsv.partial"
    done
  done
  mv -f "$plan_directory/shards.tsv.partial" "$plan_directory/shards.tsv"
  python3 - "$plan_directory/shards.tsv" "$plan_directory/plan.json.partial" <<'PY' || return 1
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    labels = [line.split("\t", 1)[0] for line in handle if line.strip()]
with open(sys.argv[2], "w", encoding="utf-8") as handle:
    json.dump({"shards": [{"label": label} for label in labels], "note": "E171 整表那几行：库里没有的独立项一项一片，两张表缺的行按表里的次序切成连续的片"},
              handle, ensure_ascii=False)
PY
  mv -f "$plan_directory/plan.json.partial" "$plan_directory/plan.json"
  rm -rf "${plan_directory:?}/work"
  echo "E171_ROW_SHARD_PLAN shards=$(grep -c . "$plan_directory/shards.tsv" || true) item_shards=$item_count rows_per_slice=$rows_per_slice maximum=$maximum" >&2
}

# 一片（文件头「分片」各片）：<计划目录>
shard_phase() {
  local plan_directory="$1" spec="${TASK_SERVER_SHARD:-}" output="${TASK_SERVER_SHARD_OUTPUT_DIRECTORY:-}" index count line label kind value table rows code
  if [[ ! "$spec" =~ ^([0-9]+)/([1-9][0-9]*)$ || -z "$output" ]]; then
    echo "  ✗ 服务的 shard 那一步没给片号或产物目录（TASK_SERVER_SHARD=${spec:-没设}，TASK_SERVER_SHARD_OUTPUT_DIRECTORY=${output:-没设}）" >&2
    echo "  → 怎么办：这一步只由 task-server 起；直接跑不设 TASK_SERVER_SHARD_PHASE" >&2
    return 2
  fi
  index="${BASH_REMATCH[1]}"
  count="${BASH_REMATCH[2]}"
  if [[ ! -f "$plan_directory/shards.tsv" ]] || (( $(grep -c . "$plan_directory/shards.tsv" || true) != count )) || (( index >= count )); then
    echo "  ✗ 计划目录的 shards.tsv 不在、行数不是服务给的片数 $count，或片号 $index 不在里面（$plan_directory）" >&2
    echo "  → 怎么办：计划那一步没写全：看计划那一步的日志，重交这一件" >&2
    return 1
  fi
  line="$(sed -n "$((index + 1))p" "$plan_directory/shards.tsv")"
  IFS=$'\t' read -r label kind value <<<"$line"
  mkdir -p "$output/work"
  case "$kind" in
    item)
      case "$value" in
        prerequisites|positive-control:O1-对拍|positive-control:O1-崩溃判官|positive-control:O2)
          if PYTHONDONTWRITEBYTECODE=1 python3 "$steps" run "$value" --root "$root" --repo-root "$PWD" --device-features "$features"; then code=0; else code=$?; fi ;;
        empty-mutation)
          if run_rows empty-mutation "$output/work" --empty-mutation-only; then code=0; else code=$?; fi ;;
        *) echo "  ✗ shards.tsv 第 $((index + 1)) 行的项「$value」不是计划会拆的项" >&2; echo "  → 怎么办：重交这一件" >&2; code=1 ;;
      esac ;;
    rows)
      table="${value%% *}"
      rows="${value#* }"
      case "$table" in
        step8)
          if run_rows step8 "$output/work" --selected-rows "$step8_table" --rows "$rows" --lanes 2 --threads 2 --keep-compile-failures-as-they-are; then code=0; else code=$?; fi ;;
        f4)
          if run_rows f4 "$output/work" --selected-rows "$f4_table" --other-selected-rows "$step8_table" --rows "$rows" --lanes 2 --threads 2 \
               --keep-compile-failures-as-they-are; then code=0; else code=$?; fi ;;
        *) echo "  ✗ shards.tsv 第 $((index + 1)) 行的表「$table」认不出" >&2; echo "  → 怎么办：重交这一件" >&2; code=1 ;;
      esac ;;
    *) echo "  ✗ shards.tsv 第 $((index + 1)) 行的种类「$kind」认不出（认 item、rows）" >&2; echo "  → 怎么办：重交这一件" >&2; code=1 ;;
  esac
  rm -rf "${output:?}/work"
  echo "E171_ROW_SHARD_SLICE index=$index/$count exit=$code label=$label" >&2
  (( code != 0 )) || printf '%s\n' "$label" >"$output/shard.done"
  return "$code"
}

declare -A q3core_rows_of=()
run_q3core() { # <O1|O2|no-executor> <工作根>：算出的行记进 q3core_rows_of（核账用）
  local name="$1" work="$2" printed rows flag=(--disable-oracle "$1")
  [[ "$name" == no-executor ]] && flag=(--no-executor)
  prerequisites_done "q3core:$name" || return 4
  step8_rows_done "q3core:$name" || return 4
  printed="$(PYTHONDONTWRITEBYTECODE=1 python3 "$comparator" --print-q3-core-rows --rows-root "$root/step8" --baseline-dir "$root/prerequisites/dumps/baseline-run1")" || return 1
  if [[ ! "$printed" =~ q3_core_rows=([0-9,]*) ]]; then
    echo "  ✗ comparator.py --print-q3-core-rows 没打 q3_core_rows=：$printed" >&2
    echo "  → 怎么办：单起 comparator.py --print-q3-core-rows 看它打什么" >&2
    return 1
  fi
  rows="$(tr ',' '\n' <<<"${BASH_REMATCH[1]},$Q3_CORE_ROW_OF_THE_S2_RUN" | grep -v '^$' | sort -n -u | paste -sd, -)"
  q3core_rows_of[$name]="$rows"
  echo "E171_ROW_SHARD q3core=$name rows=$rows" >&2
  run_rows "q3core-$name" "$work" --rows "$rows" "${flag[@]}" --lanes 2 --threads 2
}

# 合并里做的项（依赖前面各片的那几样）：<项>；工作根是 <根>/work-<项>-<随机串>，做完删
run_dependent_item() {
  local work code
  work="$(mktemp -d "$root/work-${1//:/-}-XXXXXX")"
  case "$1" in
    positive-control:O3)
      if PYTHONDONTWRITEBYTECODE=1 python3 "$steps" run "$1" --root "$root" --repo-root "$PWD" --device-features "$features"; then code=0; else code=$?; fi ;;
    s2-substitutes)
      if step8_rows_done "$1" && run_rows s2-substitutes "$work" --selected-rows "$step8_table" --lanes 2 --threads 2; then code=0; else code=$?; fi ;;
    substitutes)
      if step8_rows_done "$1" && run_rows substitutes "$work" --selected-rows "$step8_table" --other-selected-rows "$f4_table" --lanes 2 --threads 2; then code=0; else code=$?; fi ;;
    q3core:*)
      if run_q3core "${1#q3core:}" "$work"; then code=0; else code=$?; fi ;;
  esac
  rm -rf "${work:?}"
  return "$code"
}

# 合并（文件头「分片」合并）：<计划目录>
merge_phase() {
  local plan_directory="$1" missing_shards="${TASK_SERVER_SHARD_MISSING-}" outputs="${TASK_SERVER_SHARD_OUTPUTS:-}" count index host directory
  local item table part code started item_missing item_missing_count low high passed=0 first_failure=0 step8_audit dependents_ran=no parts_text selected
  local missing_row missing_reason audited failed_list merge_work="${plan_directory:?}/merge-work" audit_source
  local -a parts=() failed_items=() range_arguments=() flag=()
  local -A shard_directory=() item_code=() item_seconds=()
  if [[ -n "$missing_shards" ]]; then
    echo "  ✗ 缺片：$missing_shards（服务说这几片没有产物）；不合并、不核账" >&2
    echo "  → 怎么办：同一条命令重交，库里有的项与行照键复用、只补缺的" >&2
    return 1
  fi
  if [[ ! -f "$plan_directory/shards.tsv" ]]; then
    echo "  ✗ 计划目录里没有 shards.tsv（$plan_directory）；不合并" >&2
    echo "  → 怎么办：计划那一步没写全：看计划那一步的日志，重交这一件" >&2
    return 1
  fi
  count="$(grep -c . "$plan_directory/shards.tsv" || true)"
  if [[ -n "$outputs" && -f "$outputs" ]]; then
    while IFS=$'\t' read -r index host directory; do shard_directory[$index]="$directory"; done <"$outputs"
  fi
  for ((index = 0; index < count; index++)); do
    if [[ ! -f "${shard_directory[$index]:-$plan_directory/no-such-output}/shard.done" ]]; then
      echo "  ✗ 第 $index 片没有产物（TASK_SERVER_SHARD_OUTPUTS 里没有它，或它的产物目录里没有 shard.done）；不合并、不核账" >&2
      echo "  → 怎么办：同一条命令重交，库里有的项与行照键复用、只补缺的" >&2
      return 1
    fi
  done
  # 要的行从库里取进共用输出根（step8：点到它的片或依赖它的项；f4：点到它的片），缺的记进 merge-missing-<表>.tsv（序号<TAB>为什么）
  for table in step8 f4; do
    : >"$plan_directory/merge-missing-$table.tsv"
    parts_text="$(table_parts "$table" yes)" || return $?
    parts=()
    [[ -z "$parts_text" ]] || mapfile -t parts <<<"$parts_text"
    for part in ${parts[@]+"${parts[@]}"}; do
      range_arguments=()
      selected="$step8_table"
      [[ "$table" == step8 ]] || selected="$f4_table"
      [[ "$part" == - ]] || range_arguments=(--row-range "$part")
      if take_rows "$table" "$merge_work" "$plan_directory/merge-$table.err" --selected-rows "$selected" ${range_arguments[@]+"${range_arguments[@]}"}; then code=0; else code=$?; fi
      if [[ "$code" != 0 && "$code" != 4 ]]; then
        cat "$plan_directory/merge-$table.err" >&2
        echo "  ✗ E171 合并：从库里取表 $table 的行那一步退 $code；不核账" >&2
        echo "  → 怎么办：照上面那几行的出路办，再交同一条命令" >&2
        return "$code"
      fi
      rows_missing_in "$plan_directory/merge-$table.err" >>"$plan_directory/merge-missing-$table.tsv" || return 1
    done
  done
  # 做依赖前面各片的项
  for item in "${ordered[@]}"; do
    is_merge_item "$item" || continue
    started=$SECONDS
    if run_dependent_item "$item"; then code=0; else code=$?; fi
    item_code[$item]="$code"
    item_seconds[$item]=$((SECONDS - started))
    echo "E171_ROW_SHARD_DONE item=$item exit=$code seconds=${item_seconds[$item]}" >&2
    case "$item" in s2-substitutes|substitutes) dependents_ran=yes ;; esac
  done
  # 两种补位做过之后 step8 的行照此刻的库再核一遍（补位那一趟可能判了 step8 里缺的行）
  step8_audit="$plan_directory/merge-missing-step8.tsv"
  if [[ "$dependents_ran" == yes ]]; then
    if take_rows step8 "$merge_work" "$plan_directory/audit-step8.err" --selected-rows "$step8_table" --check-only; then code=0; else code=$?; fi
    if [[ "$code" != 0 && "$code" != 4 ]]; then
      cat "$plan_directory/audit-step8.err" >&2
      echo "  ✗ E171 合并：核 step8 的行那一步退 $code" >&2
      echo "  → 怎么办：照上面那几行的出路办，再交同一条命令" >&2
      return "$code"
    fi
    step8_audit="$plan_directory/audit-step8.tsv"
    rows_missing_in "$plan_directory/audit-step8.err" >"$step8_audit" || return 1
  fi
  # 核账：点到的每一项照键在库里（文件头「分片」合并）
  item_missing="$plan_directory/item-missing.tsv"
  for item in "${ordered[@]}"; do
    : >"$item_missing"
    code="${item_code[$item]:-0}"
    audited=0
    case "$item" in
      prerequisites|positive-control:*)
        if PYTHONDONTWRITEBYTECODE=1 python3 "$steps" fetch "$item" --root "$root" --repo-root "$PWD" --device-features "$features" 2>"$plan_directory/audit-item.err"; then audited=0; else audited=$?; fi
        if (( audited == 4 )); then printf '%s\t%s\n' - "库里没有这棵树、这组特性下键相同、判绿的这一项" >"$item_missing"; fi ;;
      empty-mutation)
        if take_rows empty-mutation "$merge_work" "$plan_directory/audit-item.err" --empty-mutation-only; then audited=0; else audited=$?; fi
        if (( audited == 4 )); then rows_missing_in "$plan_directory/audit-item.err" >"$item_missing" || return 1; fi ;;
      step8:*|f4:*)
        table="$(table_of_table_item "$item")"
        [[ "$item" =~ ^(step8|f4):([0-9]+)/([0-9]+)$ ]]
        part="$(slice_range "$table" "${BASH_REMATCH[2]}" "${BASH_REMATCH[3]}")" || return $?
        low="${part%-*}"
        high="${part#*-}"
        if [[ "$table" == step8 ]]; then audit_source="$step8_audit"; else audit_source="$plan_directory/merge-missing-f4.tsv"; fi
        awk -F'\t' -v low="$low" -v high="$high" '$1 >= low && $1 <= high' "$audit_source" | sort -n -u >"$item_missing" ;;
      s2-substitutes|substitutes)
        sort -n -u "$step8_audit" >"$item_missing" ;;
      q3core:*)
        if [[ -z "${q3core_rows_of[${item#q3core:}]:-}" ]]; then
          printf '%s\t%s\n' - "这一趟没算出 Q3-核的行（这一项没做成，见上面 E171_ROW_SHARD_DONE）" >"$item_missing"
        else
          flag=(--disable-oracle "${item#q3core:}")
          [[ "$item" != q3core:no-executor ]] || flag=(--no-executor)
          if take_rows "q3core-${item#q3core:}" "$merge_work" "$plan_directory/audit-item.err" --rows "${q3core_rows_of[${item#q3core:}]}" "${flag[@]}" --check-only; then
            audited=0
          else
            audited=$?
          fi
          if (( audited == 4 )); then rows_missing_in "$plan_directory/audit-item.err" >"$item_missing" || return 1; fi
        fi ;;
    esac
    if (( audited != 0 && audited != 4 )); then
      cat "$plan_directory/audit-item.err" >&2
      (( code != 0 )) || code=$audited
    fi
    item_missing_count="$(grep -c . "$item_missing" || true)"   # 核账：这一项缺几样
    while IFS=$'\t' read -r missing_row missing_reason; do
      if [[ "$missing_row" == - ]]; then
        echo "E171_ROW_SHARD_MISSING item=$item reason=$missing_reason" >&2
      else
        echo "E171_ROW_SHARD_MISSING item=$item row=$missing_row reason=$missing_reason" >&2
      fi
    done <"$item_missing"
    (( code != 0 || item_missing_count == 0 )) || code=1
    echo "E171_ROW_SHARD_ITEM item=$item exit=$code missing=$item_missing_count seconds=${item_seconds[$item]:-0}" >&2
    if (( code == 0 )); then
      passed=$((passed + 1))
    else
      failed_items+=("$item")
      (( first_failure != 0 )) || first_failure=$code
    fi
  done
  rm -rf "${merge_work:?}"
  failed_list="$(IFS=,; echo "${failed_items[*]:-}")"
  echo "E171_ROW_SHARD_TOTAL items=${#ordered[@]} passed=$passed failed=${#failed_items[@]} failed_items=${failed_list:--}" >&2
  if (( first_failure != 0 )); then
    echo "  ✗ E171 合并：${#failed_items[@]} 项没做成或核账有缺（上面 E171_ROW_SHARD_MISSING 逐个点名），一样都没现判" >&2
    echo "  → 怎么办：经服务交不带 --item 的整条（计划把库里没有的项与行拆成片补判）：$(remake_all)" >&2
  fi
  return "$first_failure"
}

# 服务的三步之一：照 TASK_SERVER_SHARD_PHASE 起计划、一片或合并
dispatch_phase() {
  local phase="${TASK_SERVER_SHARD_PHASE:-}" plan_directory="${TASK_SERVER_SHARD_PLAN_DIRECTORY:-}"
  if [[ -z "$plan_directory" || ! -d "$plan_directory" ]]; then
    echo "  ✗ 服务的 $phase 那一步没给在的计划目录（TASK_SERVER_SHARD_PLAN_DIRECTORY=${plan_directory:-没设}）" >&2
    echo "  → 怎么办：这一步只由 task-server 起；直接跑不设 TASK_SERVER_SHARD_PHASE" >&2
    return 2
  fi
  case "$phase" in
    plan) plan_phase "$plan_directory" "${TASK_SERVER_SHARD_MAXIMUM:-$DIRECT_RUN_SHARD_MAXIMUM}" ;;
    shard) shard_phase "$plan_directory" ;;
    merge) merge_phase "$plan_directory" ;;
    *)
      echo "  ✗ TASK_SERVER_SHARD_PHASE=$phase 认不出（认 plan、shard、merge）" >&2
      echo "  → 怎么办：这个变量只由 task-server 设；直接跑不设它" >&2
      return 2 ;;
  esac
}

# 不经服务起（只给自证的假驱动用；文件头「分片」末一条）：自己当假服务，逐步设分片变量起计划、每一片、合并
direct_run() {
  local service index count code=0 failed=0
  service="$(mktemp -d "$root/.direct-run-XXXXXX")"
  mkdir -p "$service/plan"
  : >"$service/outputs.tsv"
  if (export TASK_SERVER_SHARD_PHASE=plan TASK_SERVER_SHARD_PLAN_DIRECTORY="$service/plan" TASK_SERVER_SHARD_MAXIMUM="$DIRECT_RUN_SHARD_MAXIMUM"; dispatch_phase); then
    count="$(grep -c . "$service/plan/shards.tsv" || true)"
    for ((index = 0; index < count; index++)); do
      if (export TASK_SERVER_SHARD_PHASE=shard TASK_SERVER_SHARD_PLAN_DIRECTORY="$service/plan" TASK_SERVER_SHARD="$index/$count" \
            TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$service/output-$index"; dispatch_phase); then
        printf '%s\t%s\t%s\n' "$index" direct "$service/output-$index" >>"$service/outputs.tsv"
      else
        code=$?
        (( failed != 0 )) || failed=$code
      fi
    done
    if (( failed != 0 )); then
      code=$failed
    elif (export TASK_SERVER_SHARD_PHASE=merge TASK_SERVER_SHARD_PLAN_DIRECTORY="$service/plan" TASK_SERVER_SHARD_OUTPUTS="$service/outputs.tsv" \
            TASK_SERVER_SHARD_MISSING=""; dispatch_phase); then
      code=0
    else
      code=$?
    fi
  else
    code=$?
  fi
  rm -rf "${service:?}"
  return "$code"
}

mkdir -p "$root"
if [[ -n "${TASK_SERVER_SHARD_PHASE:-}" ]]; then
  if dispatch_phase; then exit 0; else exit $?; fi
fi
if direct_run; then exit 0; else exit $?; fi
