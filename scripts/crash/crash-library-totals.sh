#!/usr/bin/env bash
# admission: always 只读查询：答的是此刻判定库里的总账，库随时被别的任务写，上一次查到的数不替这一次作保
# run-condition: command bash cargo python3
# task-server-compiles: yes
# selftest-serial: 每格只起一趟本脚本、几十毫秒就完，并行省不下时间，十几格按次序跑
# 崩溃判定库按态数的总账（只读查询）：起一次只读读库程序 crash_library_reader keys（验证包 crates/singlefs-verification/src/bin/crash_library_reader.rs，
#   一遍扫全库、不写），读库程序打的行原样留在标准输出，后面接一段给人看的总账与末行成功句。
#
# 用法：
#   crash-library-totals.sh              查一次全库。读库程序与真跑编同一套特性：特性串由 lib_crash_point_replay_items.py reader-features 现算
#                                        （与 crash-point-replay.sh --list-items --with-keys 取同一处：多机配置开了 GPU 带 gpu，再加验证包 [dev-dependencies]
#                                        里带特性的依赖的 <依赖>/<特性>）；多机配置的取法同那份入口（crash-amplification-lib.sh 的
#                                        crash_amplification_multi_host_configuration_file：SINGLEFS_MULTI_HOST_CONFIG 设了照它，没设取主工作树根下的 multi-host.env），
#                                        文件在就以绝对路径交给读库程序（它照多机配置找 prune-server）。
#   crash-library-totals.sh --selftest   假 cargo 当读库程序、小仓里的验证包清单，走一遍各格（文件头「自证」）
#   别的参数一律退 2。--list-items 与 --item 同样退 2、打出路：这是只读查询、不判项——读库程序 keys 一遍扫全库，没有按项缩的读法；
#   按项的块账与要现判的态数归 bash scripts/crash/crash-point-replay.sh --list-items --with-keys [--item <项>]。
#   经 task-server/task submit -- bash scripts/crash/crash-library-totals.sh 交（会编读库程序，文件头 task-server-compiles 让服务给编译目录）。
#
# 读库程序的输出（写法以 crash_library_reader.rs 文件头「输出」那一段为准；行内制表符隔开、字段写成 <名>=<值>）：这里读三种行，别的行原样转出、不读：
#   CRASH_LIBRARY_KEYS<TAB>library=<库><TAB>recorded_blocks=<TAB>judged_under_the_current_key=<TAB>judged_only_under_another_key=<TAB>never_judged=      块数，恰好一行
#   CRASH_LIBRARY_STATES<TAB>recorded_states=<TAB>states_judged_under_the_current_key=<TAB>states_judged_only_under_another_key=<TAB>states_never_judged=   态数，恰好一行
#   CRASH_LIBRARY_STORED_KIND<TAB><种类><TAB>versions=<TAB>block_version_pairs=<TAB>pairs_recorded=<TAB>states_of_the_pairs_recorded=<TAB>pairs_not_recorded=     每种结论一行，至少一行
#   每把版本一行的 CRASH_LIBRARY_STORED_VERSION 只转出，不拿它求和。数一律是非负整数（至多 18 位）。
# 判法（totals_judged）：
#   缺行：没有 CRASH_LIBRARY_STATES 或一行 CRASH_LIBRARY_STORED_KIND 都没有（读库程序还没有按态的行），或没有 CRASH_LIBRARY_KEYS ⇒ 判红，不拿各版本行求和顶替、不报 0。
#   认不出：三种行的字段名、个数、次序对不上，数不是非负整数，CRASH_LIBRARY_KEYS 或 CRASH_LIBRARY_STATES 不止一行，同一种结论两行 ⇒ 判红。
#   完整性闸：states_judged_under_the_current_key + states_judged_only_under_another_key + states_never_judged 要等于 recorded_states；
#     每种结论 pairs_recorded + pairs_not_recorded 要等于 block_version_pairs；对不上判红。
#   判红时标准输出一个字都不打（读库程序的行转到标准错误给人看）；判过才把读库程序的行与总账一起打到标准输出。
# 总账（判过之后，接在读库程序的行后面）：
#   一段四行：录下的、当前键下判过的、只在别的键下判过的、一种判器的结论都没有的，各写态数、折成亿（四舍五入到两位小数）与块数（块数取 CRASH_LIBRARY_KEYS 那一行）；
#   按结论种类每种一行：几把键（versions）、块×键对数（block_version_pairs）、数得出态的对数（pairs_recorded）、态×键合计（states_of_the_pairs_recorded，带亿）、
#   读不出态数的对数（pairs_not_recorded），末了一行全部种类的合计；末行成功句「✓ 判定库总账：录下 N 态（…亿），当前键下判过 …；结论种类 K 种、键 M 把……」。
# 编译目录：照调用方的 CARGO_TARGET_DIR（经 task-server 交时服务按 --cwd 那一棵树给，编译目录归树、不归任务）；没设退 2、一个 cargo 都不起（出路是经 task-server/task submit 交）。
#   起 cargo 之前先判这台的 RocksDB 预编库取值（scripts/hosts/rocksdb-prebuilt.py entry；判红退 78）。
# 退出码：
#   0   总账打出、两道完整性闸都过
#   1   读库程序退非 0（75 除外）、输出缺行、认不出、完整性闸对不上，或读库程序的特性串算不出
#   2   用法错：参数认不得、给了 --list-items 或 --item、--selftest 夹了别的参数、平常跑时设了只供自证的变量、没有编译目录 CARGO_TARGET_DIR
#   75  prune-server 不可用（读库程序退 75），出路 python3 prune-server/control check
#   78  运行条件不满足：preflight 判的（文件头 run-condition），或起 cargo 之前 rocksdb-prebuilt.py entry 判红
# 只供自证（--selftest 起的子进程里才认；平常跑时有一个非空就在参数解析之前退 2 拒跑，--selftest 本身只认弄坏开关、转给它起的子进程）：
#   CRASH_LIBRARY_TOTALS_BREAK=<项>（下面那张表，几个用逗号连）；CRASH_LIBRARY_TOTALS_ROOT=<造出来的小仓>：验证包清单与 rocksdb-prebuilt.py entry 的仓根都取它。
#   「自证起的子进程」的标记：CRASH_LIBRARY_TOTALS_SELFTEST_MARK=<文件>，那份文件在、内容是 SELFTEST_MARK_CONTENT 那一行；只有 run_selftest 设它。
#   这几个变量读完就从环境里清掉，不传给读库程序。
# 自证（--selftest）：临时目录里一棵小仓（验证包清单的 [dev-dependencies] 带两个特性、没有 rocksdb 依赖，entry 报不判；rocksdb-entry-refused 那一格另一棵带 rocksdb 依赖的），
#   PATH 最前面放假 cargo（记一行参数、目录与交来的多机配置，把 STUB_READER_OUTPUT 那份文件当读库程序的输出、退 STUB_READER_EXIT），多机配置是临时目录里只写 ENABLE_GPU 的两份。
#   不编、不连 prune-server。每格失败打一行「✗ 自证 <格名>：…」与出路；格：
#     green                     真库量级的一份完整输出 ⇒ 退 0，读库程序的行原样都在标准输出，总账四行、每种结论一行与合计一行的数与折成的亿对得上，末行是成功句
#     reader-same-build         开了 GPU 的配置 ⇒ 读库程序带「gpu,<清单里带特性的依赖>」、在仓根起、收到多机配置的绝对路径；关了 GPU 的 ⇒ 不带 gpu
#     reader-lines-missing      只有块数那两种行（读库程序还没有按态的行）、有态数行而没有按种类的行 ⇒ 都退 1、点名「合入那份补丁之后再查」、标准输出是空的
#     states-sum-mismatch       三类态数之和比录下的多 1 ⇒ 退 1、点名三类态数之和、标准输出是空的
#     kind-pairs-mismatch       observer 那一种数得出态的加读不出的比块×键对数多 1 ⇒ 退 1、点名那一种、标准输出是空的
#     field-shapes-refused      CRASH_LIBRARY_STATES 少一个字段 ⇒ 退 1、点名认不出、标准输出是空的
#     reader-exit-reported      读库程序输出完整而退 75 ⇒ 退 75、点名 prune-server；退 3 ⇒ 退 1；两趟标准输出都是空的
#     build-directory-required  没有 CARGO_TARGET_DIR ⇒ 退 2、一个 cargo 都不起
#     rocksdb-entry-refused     验证包依赖 rocksdb、进程环境里四个预编库取值是空的 ⇒ 退 78、一个 cargo 都不起
#     items-arguments-refused   --list-items、--item x ⇒ 都退 2、点名没有项、一个 cargo 都不起
#     unknown-argument-refused  --no-such-option ⇒ 退 2、一个 cargo 都不起
# 弄坏开关 CRASH_LIBRARY_TOTALS_BREAK=<项>（只在自证起的子进程里认）：一行「#   CRASH_LIBRARY_TOTALS_BREAK=<开关名>  <弄坏哪一处>」，下一行起写点名的格与失败行句首：
#   CRASH_LIBRARY_TOTALS_BREAK=missing-lines-as-zero  读库程序没打按态的行与块数行时当 0 照算，不判红
#       reader-lines-missing（「✗ 自证 reader-lines-missing：」）
#   CRASH_LIBRARY_TOTALS_BREAK=states-sum-unchecked  不核三类态数之和等于录下的态数
#       states-sum-mismatch（「✗ 自证 states-sum-mismatch：」）
#   CRASH_LIBRARY_TOTALS_BREAK=kind-pairs-unchecked  不核每种结论数得出态的对数加读不出的等于块×键对数
#       kind-pairs-mismatch（「✗ 自证 kind-pairs-mismatch：」）
#   CRASH_LIBRARY_TOTALS_BREAK=field-shapes-unchecked  三种行的字段只按位置取值，不核名字、个数与是不是非负整数
#       field-shapes-refused（「✗ 自证 field-shapes-refused：」）
#   CRASH_LIBRARY_TOTALS_BREAK=output-before-judging  读库程序的行先打到标准输出再判（判红时标准输出不空）
#       reader-lines-missing（「✗ 自证 reader-lines-missing：」）、states-sum-mismatch（「✗ 自证 states-sum-mismatch：」）、
#       kind-pairs-mismatch（「✗ 自证 kind-pairs-mismatch：」）、field-shapes-refused（「✗ 自证 field-shapes-refused：」）、
#       reader-exit-reported（「✗ 自证 reader-exit-reported：」）
#   CRASH_LIBRARY_TOTALS_BREAK=reader-exit-ignored  读库程序退非 0 也照读它的输出往下判
#       reader-exit-reported（「✗ 自证 reader-exit-reported：」）
#   CRASH_LIBRARY_TOTALS_BREAK=build-directory-unchecked  没有 CARGO_TARGET_DIR 也照起 cargo
#       build-directory-required（「✗ 自证 build-directory-required：」）
#   CRASH_LIBRARY_TOTALS_BREAK=rocksdb-entry-unchecked  起 cargo 之前不调 rocksdb-prebuilt.py entry
#       rocksdb-entry-refused（「✗ 自证 rocksdb-entry-refused：」）
#   CRASH_LIBRARY_TOTALS_BREAK=items-arguments-accepted  --list-items 与 --item 丢掉、照查全库
#       items-arguments-refused（「✗ 自证 items-arguments-refused：」）
#   CRASH_LIBRARY_TOTALS_BREAK=unknown-argument-accepted  认不得的参数丢掉、照查全库
#       unknown-argument-refused（「✗ 自证 unknown-argument-refused：」）
#   CRASH_LIBRARY_TOTALS_BREAK=reader-features-dropped  读库程序那一趟 cargo run 不带特性
#       reader-same-build（「✗ 自证 reader-same-build：」）
#   CRASH_LIBRARY_TOTALS_BREAK=multi-host-configuration-dropped  不把多机配置交给读库程序
#       reader-same-build（「✗ 自证 reader-same-build：」）
#   CRASH_LIBRARY_TOTALS_BREAK=kind-lines-dropped  总账里不打按结论种类的每一行（只打合计）
#       green（「✗ 自证 green：」）
#   CRASH_LIBRARY_TOTALS_BREAK=hundred-million-truncated  折成亿时直接截掉两位小数之后的，不四舍五入
#       green（「✗ 自证 green：」）
#
# gate-similar: crash-point-replay.sh 它的 --list-items --with-keys 按块报键对不上的块、按项估要现判的态；这里只读一次全库出按态数的总账、核两道完整性闸，并进去要把只读查询塞进那份带分片与准入的入口
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

SELFTEST_MARK_CONTENT="crash-library-totals.sh --selftest 起的子进程"
TOTALS_STATES_PER_HUNDRED_MILLION=100000000   # 折成亿的除数
TOTALS_FIELD_DIGITS_MOST=18                   # 字段的数至多几位：bash 算术是 64 位有符号，18 位以内加起来不溢出

totals_script="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
totals_script_directory="$(dirname "$totals_script")"
source "$totals_script_directory/crash-amplification-lib.sh"

# 只供自证的变量（文件头「只供自证」）：读完就清掉
totals_break="${CRASH_LIBRARY_TOTALS_BREAK:-}"
totals_root_override="${CRASH_LIBRARY_TOTALS_ROOT:-}"
totals_selftest_mark="${CRASH_LIBRARY_TOTALS_SELFTEST_MARK:-}"
unset CRASH_LIBRARY_TOTALS_BREAK CRASH_LIBRARY_TOTALS_ROOT CRASH_LIBRARY_TOTALS_SELFTEST_MARK
totals_in_selftest_child=0
if [[ -n "$totals_selftest_mark" && -f "$totals_selftest_mark" && "$(cat -- "$totals_selftest_mark")" == "$SELFTEST_MARK_CONTENT" ]]; then
  totals_in_selftest_child=1
fi
if (( ! totals_in_selftest_child )) && [[ "${1-}" != --selftest ]] && [[ -n "$totals_break" || -n "$totals_root_override" ]]; then
  echo "  ✗ CRASH_LIBRARY_TOTALS_BREAK、CRASH_LIBRARY_TOTALS_ROOT 只在自证（--selftest 起的子进程）里认，平常跑设了不跑：BREAK=「$totals_break」ROOT=「$totals_root_override」" >&2
  echo "  → 怎么办：去掉这两个变量再交；要看弄坏开关的效果跑 CRASH_LIBRARY_TOTALS_BREAK=<项> bash scripts/crash/crash-library-totals.sh --selftest" >&2
  exit 2
fi
totals_break_is() { (( totals_in_selftest_child )) && [[ ",$totals_break," == *",$1,"* ]]; }
repository_root="$(cd "$totals_script_directory/../.." && pwd)"
if (( totals_in_selftest_child )) && [[ -n "$totals_root_override" ]]; then repository_root="$totals_root_override"; fi

usage_refused() { # <哪里错了>
  echo "  ✗ $1" >&2
  echo "  → 怎么办：照下面的用法写（文件头「用法」）" >&2
  echo "       bash scripts/crash/crash-library-totals.sh              查一次全库的按态数总账（经 task-server/task submit -- 交）" >&2
  echo "       bash scripts/crash/crash-library-totals.sh --selftest   假读库程序走一遍各格" >&2
  exit 2
}

items_refused() { # <给的参数…>
  echo "  ✗ 给了 $*：这条查询没有项——读库程序 keys 一遍扫全库，没有按项缩的读法，--list-items 与 --item 都不认" >&2
  echo "  → 怎么办：查全库的总账不带参数交；按项的块账与要现判的态数用 bash scripts/crash/crash-point-replay.sh --list-items --with-keys [--item <项>]" >&2
  exit 2
}

# 态数折成亿（四舍五入到两位小数）：<态数> → 「832.05」；弄坏开关 hundred-million-truncated 下直接截
in_hundred_millions() {
  local hundredths
  if totals_break_is hundred-million-truncated; then
    hundredths=$(( $1 / (TOTALS_STATES_PER_HUNDRED_MILLION / 100) ))
  else
    hundredths=$(( ($1 + TOTALS_STATES_PER_HUNDRED_MILLION / 200) / (TOTALS_STATES_PER_HUNDRED_MILLION / 100) ))
  fi
  printf '%d.%02d' $(( hundredths / 100 )) $(( hundredths % 100 ))
}

# shaped_fields <行> <字段写法…>：这一行行首标记之后制表符隔开的字段恰好是这几个、依次对得上 → 交回 0，各值依次放进 shaped_values；对不上交回 1。
#   字段写法：<名>（值是至多 TOTALS_FIELD_DIGITS_MOST 位的非负整数）、<名>=*（值是任意非空文本）、-（一段不带 = 的非空文本，结论种类名）。
#   弄坏开关 field-shapes-unchecked 下只按位置取值（去掉 = 之前的一段），不核
shaped_values=()
shaped_fields() {
  local line="$1" position=1 shape value digits_pattern
  local -a fields
  shift
  IFS=$'\t' read -r -a fields <<<"$line"
  shaped_values=()
  if totals_break_is field-shapes-unchecked; then
    for shape in "$@"; do value="${fields[position]-}"; shaped_values+=("${value#*=}"); position=$((position + 1)); done
    return 0
  fi
  (( ${#fields[@]} == $# + 1 )) || return 1
  digits_pattern="^[0-9]{1,$TOTALS_FIELD_DIGITS_MOST}\$"
  for shape in "$@"; do
    value="${fields[position]}"
    case "$shape" in
      -) [[ -n "$value" && "$value" != *=* ]] || return 1; shaped_values+=("$value") ;;
      *=\*) [[ "$value" == "${shape%\*}"?* ]] || return 1; shaped_values+=("${value#"${shape%\*}"}") ;;
      *) [[ "$value" == "$shape="* && "${value#"$shape"=}" =~ $digits_pattern ]] || return 1; shaped_values+=("${value#"$shape"=}") ;;
    esac
    position=$((position + 1))
  done
}

# 判法（文件头「判法」）：<读库程序标准输出的文件> <总账写到的文件>。判过交回 0、总账（连同末行成功句）写进第二个文件；判红交回 1、✗ 与出路打标准错误、第二个文件不写
totals_judged() {
  local reader_output="$1" totals_file="$2" line tag kind
  local keys_rows=0 states_rows=0 library="" recorded_blocks=0 blocks_current=0 blocks_another=0 blocks_never=0
  local recorded_states=0 states_current=0 states_another=0 states_never=0
  local -a problems=() missing=() kinds=() unbalanced=()
  local -A kind_versions=() kind_pairs=() kind_pairs_recorded=() kind_states=() kind_pairs_not_recorded=()
  while IFS= read -r line || [[ -n "$line" ]]; do
    tag="${line%%$'\t'*}"
    case "$tag" in
      CRASH_LIBRARY_KEYS)
        keys_rows=$((keys_rows + 1))
        if shaped_fields "$line" 'library=*' recorded_blocks judged_under_the_current_key judged_only_under_another_key never_judged; then
          library="${shaped_values[0]}"; recorded_blocks="${shaped_values[1]}"; blocks_current="${shaped_values[2]}"
          blocks_another="${shaped_values[3]}"; blocks_never="${shaped_values[4]}"
        else
          problems+=("CRASH_LIBRARY_KEYS 那一行认不出：${line//$'\t'/ }")
        fi ;;
      CRASH_LIBRARY_STATES)
        states_rows=$((states_rows + 1))
        if shaped_fields "$line" recorded_states states_judged_under_the_current_key states_judged_only_under_another_key states_never_judged; then
          recorded_states="${shaped_values[0]}"; states_current="${shaped_values[1]}"; states_another="${shaped_values[2]}"; states_never="${shaped_values[3]}"
        else
          problems+=("CRASH_LIBRARY_STATES 那一行认不出：${line//$'\t'/ }")
        fi ;;
      CRASH_LIBRARY_STORED_KIND)
        if ! shaped_fields "$line" - versions block_version_pairs pairs_recorded states_of_the_pairs_recorded pairs_not_recorded; then
          problems+=("CRASH_LIBRARY_STORED_KIND 那一行认不出：${line//$'\t'/ }")
          continue
        fi
        kind="${shaped_values[0]}"
        if [[ -n "${kind_versions[$kind]+set}" ]]; then
          problems+=("结论种类 $kind 有不止一行 CRASH_LIBRARY_STORED_KIND")
          continue
        fi
        kinds+=("$kind")
        kind_versions[$kind]="${shaped_values[1]}"; kind_pairs[$kind]="${shaped_values[2]}"; kind_pairs_recorded[$kind]="${shaped_values[3]}"
        kind_states[$kind]="${shaped_values[4]}"; kind_pairs_not_recorded[$kind]="${shaped_values[5]}" ;;
      *) ;;
    esac
  done <"$reader_output"
  (( keys_rows > 1 )) && problems+=("CRASH_LIBRARY_KEYS 有 $keys_rows 行（只该有一行）")
  (( states_rows > 1 )) && problems+=("CRASH_LIBRARY_STATES 有 $states_rows 行（只该有一行）")
  if (( ${#problems[@]} )); then
    printf '  ✗ %s\n' "${problems[@]}" >&2   # gate-lint:detail
    echo "  ✗ 读库程序的输出有 ${#problems[@]} 处认不出（写法与 crates/singlefs-verification/src/bin/crash_library_reader.rs 文件头「输出」那一段对不上）" >&2   # gate-lint:summary
    echo "  → 怎么办：看读库程序是不是换了写法，让这份脚本文件头「读库程序的输出」与那一段对上，再交同一条命令" >&2
    return 1
  fi
  (( keys_rows )) || missing+=(CRASH_LIBRARY_KEYS)
  (( states_rows )) || missing+=(CRASH_LIBRARY_STATES)
  (( ${#kinds[@]} )) || missing+=(CRASH_LIBRARY_STORED_KIND)
  if (( ${#missing[@]} )) && ! totals_break_is missing-lines-as-zero; then
    echo "  ✗ 读库程序的输出里没有 ${missing[*]}：读库程序还没有按态的行，合入那份补丁之后再查；不拿各版本行自己求和顶替、不报 0" >&2
    echo "  → 怎么办：合入给读库程序加 CRASH_LIBRARY_STATES 与每种结论一行 CRASH_LIBRARY_STORED_KIND 的那份补丁（写法在 crates/singlefs-verification/src/bin/crash_library_reader.rs 文件头「输出」那一段），再交同一条命令" >&2
    return 1
  fi
  if ! totals_break_is states-sum-unchecked && (( states_current + states_another + states_never != recorded_states )); then
    echo "  ✗ 三类态数之和对不上录下的态数：当前键下判过的 $states_current + 只在别的键下判过的 $states_another + 一种判器的结论都没有的 $states_never = $(( states_current + states_another + states_never ))，录下的 $recorded_states" >&2
    echo "  → 怎么办：读库程序数态的那一段有漏数或重数，看 crash_library_reader.rs 里 CRASH_LIBRARY_STATES 那一行怎么数的；这一趟的数不能用" >&2
    return 1
  fi
  for kind in ${kinds[@]+"${kinds[@]}"}; do
    if ! totals_break_is kind-pairs-unchecked && (( kind_pairs_recorded[$kind] + kind_pairs_not_recorded[$kind] != kind_pairs[$kind] )); then
      unbalanced+=("$kind：pairs_recorded + pairs_not_recorded = ${kind_pairs_recorded[$kind]} + ${kind_pairs_not_recorded[$kind]}，block_version_pairs = ${kind_pairs[$kind]}")
    fi
  done
  if (( ${#unbalanced[@]} )); then
    printf '  ✗ 结论种类 %s\n' "${unbalanced[@]}" >&2   # gate-lint:detail
    echo "  ✗ ${#unbalanced[@]} 种结论的数得出态的对数加读不出的对数对不上块×键对数" >&2   # gate-lint:summary
    echo "  → 怎么办：读库程序按种类数块×键对的那一段有漏数或重数，看 crash_library_reader.rs 里 CRASH_LIBRARY_STORED_KIND 那一行怎么数的；这一趟的数不能用" >&2
    return 1
  fi
  local versions_total=0 pairs_total=0 pairs_recorded_total=0 states_total=0 pairs_not_recorded_total=0
  {
    printf '判定库 %s 按态数的总账（块数取同一趟 CRASH_LIBRARY_KEYS 那一行）：\n' "$library"
    printf '  录下的：%s 态（%s 亿），%s 块\n' "$recorded_states" "$(in_hundred_millions "$recorded_states")" "$recorded_blocks"
    printf '  当前键下判过的：%s 态（%s 亿），%s 块\n' "$states_current" "$(in_hundred_millions "$states_current")" "$blocks_current"
    printf '  只在别的键下判过的：%s 态（%s 亿），%s 块\n' "$states_another" "$(in_hundred_millions "$states_another")" "$blocks_another"
    printf '  一种判器的结论都没有的：%s 态（%s 亿），%s 块\n' "$states_never" "$(in_hundred_millions "$states_never")" "$blocks_never"
    printf '按结论种类（每种一行 CRASH_LIBRARY_STORED_KIND）：\n'
    for kind in ${kinds[@]+"${kinds[@]}"}; do
      if ! totals_break_is kind-lines-dropped; then
        printf '  %s：%s 把键，块×键 %s 对，数得出态的 %s 对，态×键合计 %s 态（%s 亿），读不出态数的 %s 对\n' "$kind" "${kind_versions[$kind]}" "${kind_pairs[$kind]}" \
          "${kind_pairs_recorded[$kind]}" "${kind_states[$kind]}" "$(in_hundred_millions "${kind_states[$kind]}")" "${kind_pairs_not_recorded[$kind]}"
      fi
      versions_total=$((versions_total + kind_versions[$kind])); pairs_total=$((pairs_total + kind_pairs[$kind]))
      pairs_recorded_total=$((pairs_recorded_total + kind_pairs_recorded[$kind])); states_total=$((states_total + kind_states[$kind]))
      pairs_not_recorded_total=$((pairs_not_recorded_total + kind_pairs_not_recorded[$kind]))
    done
    printf '  合计：%s 把键，块×键 %s 对，数得出态的 %s 对，态×键合计 %s 态（%s 亿），读不出态数的 %s 对\n' "$versions_total" "$pairs_total" \
      "$pairs_recorded_total" "$states_total" "$(in_hundred_millions "$states_total")" "$pairs_not_recorded_total"
    printf '✓ 判定库总账：录下 %s 态（%s 亿），当前键下判过 %s 态（%s 亿）、只在别的键下判过 %s 态（%s 亿）、一种判器的结论都没有 %s 态（%s 亿）；结论种类 %s 种、键 %s 把，态×键合计 %s 态，读不出态数的 %s 对；三类态数之和等于录下的，%s 种结论的数得出态的对数加读不出的都等于块×键对数\n' \
      "$recorded_states" "$(in_hundred_millions "$recorded_states")" "$states_current" "$(in_hundred_millions "$states_current")" \
      "$states_another" "$(in_hundred_millions "$states_another")" "$states_never" "$(in_hundred_millions "$states_never")" \
      "${#kinds[@]}" "$versions_total" "$states_total" "$pairs_not_recorded_total" "${#kinds[@]}"
  } >"$totals_file"
}

# 查一次（文件头「用法」不带参数那一行）：起读库程序、判、判过才往标准输出打。退出码见文件头「退出码」
library_totals_query() {
  local configuration_file base_features="" reader_features reader_exit
  local -a reader_environment
  if [[ -z "${CARGO_TARGET_DIR:-}" ]] && ! totals_break_is build-directory-unchecked; then
    echo "  ✗ 没有编译目录 CARGO_TARGET_DIR：读库程序要编，编译目录归树、由服务给，不照仓根的 target 编" >&2
    echo "  → 怎么办：经 task-server/task submit -- bash scripts/crash/crash-library-totals.sh 交（服务按 --cwd 那一棵树给编译目录，编译目录归树、不归任务）" >&2
    exit 2
  fi
  if ! totals_break_is rocksdb-entry-unchecked && ! python3 "$totals_script_directory/../hosts/rocksdb-prebuilt.py" entry --root "$repository_root" >&2; then
    echo "  ✗ 这台的 RocksDB 预编库取值判红（上面那几行），一个 cargo 都没起" >&2
    echo "  → 怎么办：照上面 rocksdb-prebuilt entry 打的出路修，再交同一条命令" >&2
    exit 78
  fi
  configuration_file="$(crash_amplification_multi_host_configuration_file "$repository_root")"
  [[ "$(crash_amplification_configuration_value "$configuration_file" ENABLE_GPU)" == 1 ]] && base_features=gpu
  if ! reader_features="$(python3 "$totals_script_directory/lib_crash_point_replay_items.py" reader-features "$repository_root/crates/singlefs-verification/Cargo.toml" "$base_features")"; then
    echo "  ✗ 读库程序要带的特性算不出：读不了 $repository_root/crates/singlefs-verification/Cargo.toml 的 [dev-dependencies]（上面那一行）" >&2
    echo "  → 怎么办：看那份清单可不可读、写法对不对；读库程序与真跑要编同一套特性，键才对得上" >&2
    exit 1
  fi
  totals_break_is reader-features-dropped && reader_features=""
  # 多机配置在就以绝对路径交（读库程序照它找 prune-server）；弄坏开关 multi-host-configuration-dropped 下不交、连继承来的也清掉。env 的 -u 要写在「变量=值」之前
  reader_environment=(env)
  if [[ -f "$configuration_file" ]]; then
    if totals_break_is multi-host-configuration-dropped; then
      reader_environment+=(-u SINGLEFS_MULTI_HOST_CONFIG)
    else
      reader_environment+=("SINGLEFS_MULTI_HOST_CONFIG=$(cd "$(dirname "$configuration_file")" && pwd)/$(basename "$configuration_file")")
    fi
  fi
  reader_environment+=("BINDGEN_EXTRA_CLANG_ARGS=$CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS")
  # work 不写成局部变量：EXIT 上的 trap 在函数返回之后才跑，要读得到它
  work="$(mktemp -d "${TMPDIR:-/tmp}/crash-library-totals.XXXXXX")" || { echo "  ✗ 建不了临时目录" >&2; echo "  → 怎么办：看 \${TMPDIR:-/tmp} 可不可写" >&2; exit 1; }
  trap 'rm -rf -- "$work"' EXIT
  (cd "$repository_root" && "${reader_environment[@]}" cargo run --release --quiet --offline -p singlefs-verification ${reader_features:+--features "$reader_features"} \
    --bin crash_library_reader -- keys) >"$work/reader.out" 2>"$work/reader.err"
  reader_exit=$?
  cat -- "$work/reader.err" >&2
  totals_break_is output-before-judging && cat -- "$work/reader.out"
  if (( reader_exit != 0 )) && ! totals_break_is reader-exit-ignored; then
    echo "  读库程序标准输出的末 5 行（判红，不进标准输出）：" >&2
    tail -n 5 -- "$work/reader.out" | sed 's/^/    /' >&2
    if (( reader_exit == 75 )); then
      echo "  ✗ 读库程序 crash_library_reader keys 退 75：prune-server 连不上或读到一半停了，总账查不出" >&2
      echo "  → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再交同一条命令" >&2
      exit 75
    fi
    echo "  ✗ 读库程序 crash_library_reader keys 退 $reader_exit，总账查不出（退出码的含义在 crash_library_reader.rs 文件头「退出码」）" >&2
    echo "  → 怎么办：照上面读库程序打的那几行修（库不是这一版的布局就换一个新目录建库），再交同一条命令" >&2
    exit 1
  fi
  if ! totals_judged "$work/reader.out" "$work/totals"; then
    echo "  读库程序打的行（判红，不进标准输出）：" >&2
    sed 's/^/    /' -- "$work/reader.out" >&2
    exit 1
  fi
  totals_break_is output-before-judging || cat -- "$work/reader.out"
  cat -- "$work/totals"
}

# ── 自证（文件头「自证」）
run_selftest() {
  local cells=0 cells_judged=0 failures=0 calls stdout_file stderr_file exit_code root_directory
  # work 不写成局部变量：EXIT 上的 trap 在函数返回之后才跑，要读得到它
  work="$(mktemp -d "${TMPDIR:-/tmp}/crash-library-totals-selftest.XXXXXX")" || { echo "  ✗ 自证建不了临时目录" >&2; echo "  → 怎么办：看 \${TMPDIR:-/tmp} 可不可写" >&2; return 1; }
  trap 'rm -rf -- "$work"' EXIT
  work="$(cd "$work" && pwd)"
  calls="$work/calls"; stdout_file="$work/stdout"; stderr_file="$work/stderr"; root_directory="$work/root"
  printf '%s\n' "$SELFTEST_MARK_CONTENT" >"$work/mark"
  mkdir -p "$work/bin" "$work/target" "$root_directory/crates/singlefs-verification" "$work/root-rocksdb/crates/singlefs-verification"
  printf '%s\n' '[package]' 'name = "singlefs-verification"' 'version = "0.0.0"' '[dev-dependencies]' \
    'singlefs-core = { path = "../singlefs-core", features = ["lookup-work-counters", "test-only-publish-thresholds"] }' 'plain = "1"' \
    >"$root_directory/crates/singlefs-verification/Cargo.toml"
  printf '%s\n' '[package]' 'name = "singlefs-verification"' 'version = "0.0.0"' '[dependencies]' 'rocksdb = { version = "=0.25.0", optional = true }' \
    >"$work/root-rocksdb/crates/singlefs-verification/Cargo.toml"
  printf '%s\n' 'ENABLE_GPU=1' >"$work/multi-host-gpu-on.env"
  printf '%s\n' 'ENABLE_GPU=0' >"$work/multi-host-gpu-off.env"
  cat >"$work/bin/cargo" <<'FAKE_CARGO'
#!/usr/bin/env bash
# crash-library-totals.sh --selftest 的假 cargo：记一行参数、目录与交来的多机配置，把 STUB_READER_OUTPUT 那份文件当读库程序的输出，退 STUB_READER_EXIT
printf 'cargo %s\n' "$*" >>"$STUB_CALLS"
printf 'cwd=%s\n' "$PWD" >>"$STUB_CALLS"
printf 'SINGLEFS_MULTI_HOST_CONFIG=%s\n' "${SINGLEFS_MULTI_HOST_CONFIG-unset}" >>"$STUB_CALLS"
cat -- "$STUB_READER_OUTPUT"
exit "${STUB_READER_EXIT:-0}"
FAKE_CARGO
  chmod +x "$work/bin/cargo"
  # 一份完整的读库程序输出：态数取真库的量级（亿位、四舍五入与截断分得开），块数与版本行是造的小数
  printf '%s\n' \
    $'CRASH_LIBRARY_KEYS\tlibrary=prune-server unix:/run/singlefs-prune-server/server.sock\trecorded_blocks=6\tjudged_under_the_current_key=2\tjudged_only_under_another_key=3\tnever_judged=1' \
    $'CRASH_LIBRARY_KEYS_JUDGE\timage_judgement\tcurrent_key=0505050505050505' \
    $'CRASH_LIBRARY_STATES\trecorded_states=83204785477\tstates_judged_under_the_current_key=35893860\tstates_judged_only_under_another_key=5980821966\tstates_never_judged=77188069651' \
    $'CRASH_LIBRARY_STORED_VERSION\timage_judgement\tversion=0505050505050505\tagainst_this_binary=current\tblocks=2\tblocks_recorded=2\tstates_of_the_blocks_recorded=35893860' \
    $'CRASH_LIBRARY_STORED_VERSION\tobserver\tversion=0606060606060606\tagainst_this_binary=not_compared\tblocks=3\tblocks_recorded=3\tstates_of_the_blocks_recorded=5980821966' \
    $'CRASH_LIBRARY_STORED_KIND\timage_judgement\tversions=585\tblock_version_pairs=1433050\tpairs_recorded=1233567\tstates_of_the_pairs_recorded=10190635822\tpairs_not_recorded=199483' \
    $'CRASH_LIBRARY_STORED_KIND\tobserver\tversions=327\tblock_version_pairs=2000000\tpairs_recorded=2000000\tstates_of_the_pairs_recorded=16109305366\tpairs_not_recorded=0' \
    >"$work/reader-complete"
  grep -E '^CRASH_LIBRARY_KEYS(_JUDGE)?'$'\t' "$work/reader-complete" >"$work/reader-blocks-only"
  grep -v '^CRASH_LIBRARY_STORED_KIND' "$work/reader-complete" >"$work/reader-without-kinds"
  sed 's/states_never_judged=77188069651/states_never_judged=77188069652/' "$work/reader-complete" >"$work/reader-states-sum-off"
  sed 's/pairs_not_recorded=0$/pairs_not_recorded=1/' "$work/reader-complete" >"$work/reader-kind-pairs-off"
  sed $'s/\tstates_never_judged=77188069651$//' "$work/reader-complete" >"$work/reader-states-field-missing"

  # 起一趟本脚本（子进程认自证标记）：[<变量=值>…] -- <参数…>；输出进 stdout_file、stderr_file，退出码进 exit_code，调用记录进 calls
  totals_child() {
    local -a settings=()
    while (($#)) && [[ "$1" != -- ]]; do settings+=("$1"); shift; done
    shift
    : >"$calls"
    cells=$((cells + 1))
    env -u CRASH_AMPLIFICATION_ITEM_BREAK -u GPU_CARDS_BY_PRIORITY PATH="$work/bin:$PATH" STUB_CALLS="$calls" STUB_READER_OUTPUT="$work/reader-complete" \
      CARGO_TARGET_DIR="$work/target" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on.env" CRASH_LIBRARY_TOTALS_SELFTEST_MARK="$work/mark" \
      CRASH_LIBRARY_TOTALS_ROOT="$root_directory" CRASH_LIBRARY_TOTALS_BREAK="$totals_break" ${settings[@]+"${settings[@]}"} \
      bash "$totals_script" "$@" >"$stdout_file" 2>"$stderr_file"
    exit_code=$?
  }
  # 子进程标准错误的末几行（✗ 换成「[红]」：引原文不写失败行起头的那种形态）
  child_tail() { tail -n 3 -- "$stderr_file" | tr '\n' '|' | sed 's/✗/[红]/g' | crash_amplification_clip_text 400; }
  cell_failed() { # <格名> <判错的一句> <出路>
    echo "  ✗ 自证 $1：$2（退 $exit_code；标准错误末几行：$(child_tail)）" >&2
    echo "     → 怎么办：$3" >&2
    failures=$((failures + 1))
  }
  stdout_has_line() { grep -qxF -- "$1" "$stdout_file"; }
  stderr_has() { grep -qF -- "$1" "$stderr_file"; }
  calls_empty() { [[ ! -s "$calls" ]]; }
  local reader_line wrong expected_features expected sample

  cells_judged=$((cells_judged + 1))
  # green：完整输出 ⇒ 退 0，读库程序的行原样都在，总账的数与亿对得上，末行是成功句
  totals_child --
  wrong=""
  [[ "$exit_code" == 0 ]] || wrong+="退出码不是 0；"
  while IFS= read -r reader_line; do stdout_has_line "$reader_line" || wrong+="读库程序那一行没原样留在标准输出（${reader_line%%$'\t'*}）；"; done <"$work/reader-complete"
  for expected in '  录下的：83204785477 态（832.05 亿），6 块' '  当前键下判过的：35893860 态（0.36 亿），2 块' \
                  '  只在别的键下判过的：5980821966 态（59.81 亿），3 块' '  一种判器的结论都没有的：77188069651 态（771.88 亿），1 块' \
                  '  image_judgement：585 把键，块×键 1433050 对，数得出态的 1233567 对，态×键合计 10190635822 态（101.91 亿），读不出态数的 199483 对' \
                  '  observer：327 把键，块×键 2000000 对，数得出态的 2000000 对，态×键合计 16109305366 态（161.09 亿），读不出态数的 0 对' \
                  '  合计：912 把键，块×键 3433050 对，数得出态的 3233567 对，态×键合计 26299941188 态（263.00 亿），读不出态数的 199483 对'; do
    stdout_has_line "$expected" || wrong+="总账少了「$expected」；"
  done
  [[ "$(tail -n 1 -- "$stdout_file")" == '✓ 判定库总账：录下 83204785477 态（832.05 亿），当前键下判过 35893860 态（0.36 亿）'* ]] || wrong+="末行不是成功句；"
  [[ -n "$wrong" ]] && cell_failed green "完整的读库程序输出应当退 0、原样转出、总账的数与亿对得上：$wrong" "看 totals_judged 写总账那一段与 in_hundred_millions"

  cells_judged=$((cells_judged + 1))
  # reader-same-build：开了 GPU ⇒ 带 gpu 与清单里带特性的依赖、在仓根起、收到多机配置的绝对路径；关了 GPU ⇒ 不带 gpu
  wrong=""
  expected_features="singlefs-core/lookup-work-counters,singlefs-core/test-only-publish-thresholds"
  totals_child --
  grep -qxF -- "cargo run --release --quiet --offline -p singlefs-verification --features gpu,$expected_features --bin crash_library_reader -- keys" "$calls" \
    || wrong+="开了 GPU 的那一趟 cargo run 没带「gpu,$expected_features」；"
  grep -qxF -- "cwd=$root_directory" "$calls" || wrong+="读库程序不是在仓根起的；"
  grep -qxF -- "SINGLEFS_MULTI_HOST_CONFIG=$work/multi-host-gpu-on.env" "$calls" || wrong+="读库程序没收到多机配置的绝对路径；"
  totals_child SINGLEFS_MULTI_HOST_CONFIG="$work/bin/../multi-host-gpu-off.env" --
  grep -qxF -- "cargo run --release --quiet --offline -p singlefs-verification --features $expected_features --bin crash_library_reader -- keys" "$calls" \
    || wrong+="关了 GPU 的那一趟 cargo run 应当只带「$expected_features」；"
  grep -qxF -- "SINGLEFS_MULTI_HOST_CONFIG=$work/multi-host-gpu-off.env" "$calls" || wrong+="带 .. 的多机配置路径没折成绝对路径交给读库程序；"
  [[ -n "$wrong" ]] && cell_failed reader-same-build "读库程序应当与真跑编同一套特性、收到多机配置：$wrong" "看 library_totals_query 里算特性串与 reader_environment 那一段"

  cells_judged=$((cells_judged + 1))
  # reader-lines-missing：只有块数行、有态数行没有按种类的行 ⇒ 都退 1、点名合入补丁、标准输出空
  wrong=""
  for sample in reader-blocks-only reader-without-kinds; do
    totals_child STUB_READER_OUTPUT="$work/$sample" --
    { [[ "$exit_code" == 1 && ! -s "$stdout_file" ]] && stderr_has '读库程序还没有按态的行，合入那份补丁之后再查'; } || wrong+="$sample 那一份退 $exit_code、标准输出 $(wc -l <"$stdout_file") 行；"
  done
  [[ -n "$wrong" ]] && cell_failed reader-lines-missing "读库程序没打按态的行应当退 1、点名合入补丁、不拿版本行求和、标准输出空：$wrong" "看 totals_judged 里缺行那一判"

  cells_judged=$((cells_judged + 1))
  # states-sum-mismatch：三类态数之和比录下的多 1 ⇒ 退 1、点名、标准输出空
  totals_child STUB_READER_OUTPUT="$work/reader-states-sum-off" --
  if ! { [[ "$exit_code" == 1 && ! -s "$stdout_file" ]] && stderr_has '三类态数之和对不上录下的态数'; }; then
    cell_failed states-sum-mismatch "三类态数之和对不上录下的应当退 1、点名、标准输出空" "看 totals_judged 里三类态数之和那一道闸"
  fi

  cells_judged=$((cells_judged + 1))
  # kind-pairs-mismatch：observer 数得出态的加读不出的比块×键对数多 1 ⇒ 退 1、点名那一种、标准输出空
  totals_child STUB_READER_OUTPUT="$work/reader-kind-pairs-off" --
  if ! { [[ "$exit_code" == 1 && ! -s "$stdout_file" ]] && stderr_has '结论种类 observer：pairs_recorded + pairs_not_recorded = 2000000 + 1'; }; then
    cell_failed kind-pairs-mismatch "一种结论的对数对不上应当退 1、点名那一种、标准输出空" "看 totals_judged 里按种类核对数那一道闸"
  fi

  cells_judged=$((cells_judged + 1))
  # field-shapes-refused：CRASH_LIBRARY_STATES 少一个字段 ⇒ 退 1、点名认不出、标准输出空
  totals_child STUB_READER_OUTPUT="$work/reader-states-field-missing" --
  if ! { [[ "$exit_code" == 1 && ! -s "$stdout_file" ]] && stderr_has 'CRASH_LIBRARY_STATES 那一行认不出'; }; then
    cell_failed field-shapes-refused "字段写坏的态数行应当退 1、点名认不出、标准输出空" "看 shaped_fields 与 totals_judged 里 CRASH_LIBRARY_STATES 那一支"
  fi

  cells_judged=$((cells_judged + 1))
  # reader-exit-reported：输出完整而读库程序退 75 ⇒ 退 75、点名 prune-server；退 3 ⇒ 退 1；标准输出都空
  wrong=""
  totals_child STUB_READER_EXIT=75 --
  { [[ "$exit_code" == 75 && ! -s "$stdout_file" ]] && stderr_has 'prune-server'; } || wrong+="读库程序退 75 时这一趟退 $exit_code；"
  totals_child STUB_READER_EXIT=3 --
  [[ "$exit_code" == 1 && ! -s "$stdout_file" ]] || wrong+="读库程序退 3 时这一趟退 $exit_code、标准输出 $(wc -l <"$stdout_file") 行；"
  [[ -n "$wrong" ]] && cell_failed reader-exit-reported "读库程序退非 0 应当照实报、不出总账：$wrong" "看 library_totals_query 里读库程序退出码那一判"

  cells_judged=$((cells_judged + 1))
  # build-directory-required：没有 CARGO_TARGET_DIR ⇒ 退 2、不起 cargo
  totals_child CARGO_TARGET_DIR= --
  if ! { [[ "$exit_code" == 2 ]] && calls_empty && stderr_has 'CARGO_TARGET_DIR'; }; then
    cell_failed build-directory-required "没有编译目录应当退 2、一个 cargo 都不起" "看 library_totals_query 开头编译目录那一判"
  fi

  cells_judged=$((cells_judged + 1))
  # rocksdb-entry-refused：验证包依赖 rocksdb、四个预编库取值是空的 ⇒ 退 78、不起 cargo
  totals_child CRASH_LIBRARY_TOTALS_ROOT="$work/root-rocksdb" ROCKSDB_LIB_DIR= ROCKSDB_STATIC= SNAPPY_LIB_DIR= SNAPPY_STATIC= --
  if ! { [[ "$exit_code" == 78 ]] && calls_empty && stderr_has 'rocksdb-prebuilt entry：'; }; then
    cell_failed rocksdb-entry-refused "验证包依赖 rocksdb 而预编库取值是空的，应当在起 cargo 之前退 78" "看 library_totals_query 里 rocksdb-prebuilt.py entry 那一判"
  fi

  cells_judged=$((cells_judged + 1))
  # items-arguments-refused：--list-items、--item x ⇒ 都退 2、点名没有项、不起 cargo
  wrong=""
  totals_child -- --list-items
  { [[ "$exit_code" == 2 ]] && calls_empty && stderr_has '这条查询没有项'; } || wrong+="--list-items 退 $exit_code；"
  totals_child -- --item acquisition-barrier
  { [[ "$exit_code" == 2 ]] && calls_empty && stderr_has '这条查询没有项'; } || wrong+="--item 退 $exit_code；"
  [[ -n "$wrong" ]] && cell_failed items-arguments-refused "--list-items 与 --item 应当退 2、点名没有项、不起 cargo：$wrong" "看参数那一段 --list-items|--item 那一支"

  cells_judged=$((cells_judged + 1))
  # unknown-argument-refused：认不得的参数 ⇒ 退 2、不起 cargo
  totals_child -- --no-such-option
  if ! { [[ "$exit_code" == 2 ]] && calls_empty && stderr_has '认不出的参数'; }; then
    cell_failed unknown-argument-refused "认不得的参数应当退 2、不起 cargo" "看参数那一段认不出的那一支"
  fi

  if (( failures )); then
    echo "  ✗ crash-library-totals.sh --selftest：$failures 格红（共 $cells_judged 格，起了 $cells 趟本脚本）" >&2   # gate-lint:summary
    echo "  → 怎么办：照上面各格的出路修，再跑 bash scripts/crash/crash-library-totals.sh --selftest" >&2
    return 1
  fi
  echo "  ✓ crash-library-totals.sh --selftest：$cells_judged 格都对，起了 $cells 趟本脚本（文件头 selftest-serial：按次序跑）"
}

# ── 参数（文件头「用法」）
if (( $# )); then
  case "$1" in
    --selftest)
      (( $# == 1 )) || usage_refused "--selftest 不与别的参数一起给：$*"
      run_selftest
      exit $? ;;
    --list-items|--item)
      totals_break_is items-arguments-accepted || items_refused "$@" ;;
    *)
      totals_break_is unknown-argument-accepted || usage_refused "认不出的参数：$*" ;;
  esac
fi
library_totals_query
