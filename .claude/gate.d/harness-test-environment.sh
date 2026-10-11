#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 它调的 scripts/hosts/test-environment-check.py 读内核日志、SMART 这类宿主信息，读不到的那一类它自己记「没查成」
# gate-stage: 跑完测试之后机器还干不干净（残留测试设备、临时目录残留、宿主盘）
# gate-category: harness 类
# gate-similar: harness-model-differential-and-scenarios.sh 同属 harness 类，它跑 crates/singlefs-harness 的测试并判模型对拍；这一道不跑测试，判的是测试跑完之后留在机器上的东西
# gate-similar: code-tooling.sh 它的 research-script-selftests 格只跑 test-environment-check.py 的 --selftest（判别力），不看这台机器此刻的样子；它的 stage-order 格只判这一道的文件名排在跑测试的阶段之后，不读机器；这里在真机上判
# gate-similar: checker-tier-layer0-replay.sh 它跑崩溃点重放时自己建镜像、自己收拾，判的是重放的结局；机器上残不残留、宿主盘好不好，由这一道在全部跑测试的阶段之后统一判
# gate-similar: checker-tier-qemu-device-streams.sh 它起 QEMU、判设备流的结局；带 singlefs 的 qemu 进程跑完还在不在，由这一道判
# gate-similar: gate.sh 上游「跑完没留下临时文件」那一段在全部阶段跑完之后判 gate.sh 发给这一轮的 TMPDIR 里剩下的每一项（名字不限 singlefs-*）；它判绿时这一轮的 TMPDIR 下没有剩东西，这里对那个目录的 singlefs-* 也必然判绿，所以在 gate.sh 里时临时目录残留那一类不再扫它。它看不见的 /tmp 与别处的 singlefs-*、loop / dm / 挂载、qemu 进程与宿主盘仍归这一道
#
# 格名表（--list 打它，--check machine-clean 只跑这一格，格名写错退 2）：
# gate-cell: machine-clean 跑完测试之后机器还干不干净
# 参数解析、样本根的 .gate-cells、汇总与退出码在共用库 lib/stage-cells.sh。
#
# 次序：这一道必须排在全部跑测试的阶段之后（gate.sh 按文件名 sort 起，它在 harness 类里字母序最后）：
# 残留设备、挂载、qemu 进程、宿主盘判的是这一道开跑那一刻的机器。新加或改名跑测试的阶段，名字要排在它前面；
# 次序由门禁 code-tooling 的 stage-order 格判。
#
# 判据：`scripts/hosts/test-environment-check.py check` 汇总行里「判红 N 类」不是 0 红，「没查成 N 类」不是 0 也红，
# 汇总行读不到也红；真跑时它的退出码（1 有判红、3 只有没查成、0 干净）与汇总对不上，红。
# 射程：只看机器此刻的样子，不查这一轮有没有重新 mkfs（那由用例的排他创建守）。不给 --smart，SMART 由它逐个列为没跑。
# 在 gate.sh 里（GATE_NOT_RUN_FILE 所在的目录正是此刻的 TMPDIR，gate.sh 把这一轮的临时目录发给各阶段的那一份）：把这个 TMPDIR 交给
# --gate-run-temporary-directory，临时目录残留那一类不扫它，由上游「跑完没留下临时文件」判；loop / dm / 挂载与宿主盘照旧算它。单跑时不交。
# GATE_CROSS_RUN_TMPDIR 设了（gate.sh 设它：开跑前的那个 TMPDIR）就交给 --cross-run-temporary-directory：临时目录残留那一类另扫它，
# 只判 singlefs-* 残留、判法与 /tmp 相同；不交的话它不是 /tmp 时从来没人扫。
#
# 样本（fixtures/harness-test-environment.sh/，每份的 .gate-cells 点名 machine-clean）：被判目录里没有那份脚本而有
# `test-environment-check-output.log` 时，只判这份录好的输出：
#   machine-clean-red    判红 1 类（临时目录残留）、没查成 1 类
#   machine-clean-green  10 类都查过、判红 0 类
#   machine-clean-gate-run-green  放一份只核参数的替身 scripts/hosts/test-environment-check.py：在 gate.sh 里该带
#                        --gate-run-temporary-directory <TMPDIR>、单跑时不带，对上了报判红 0 类，判绿
#   machine-clean-cross-run-green  放一份只核参数的替身：GATE_CROSS_RUN_TMPDIR 设了该带 --cross-run-temporary-directory <它>、没设不带，
#                        对上了报判红 0 类，判绿
# 脚本与录好的输出都没有：判红，不当跳过。
# 弄坏开关归共用库：STAGE_CELLS_BREAK=red-swallowed 让 machine-clean-red 退 0。
# 这一道自己的弄坏开关 HARNESS_TEST_ENVIRONMENT_BREAK=gate-run-scanned：在 gate.sh 里也不交 --gate-run-temporary-directory。
#   照 gate.sh 设好 TMPDIR 与 GATE_NOT_RUN_FILE（GATE_NOT_RUN_FILE 指到 TMPDIR 下）喂样本时 machine-clean-gate-run-green 判红、退 1；
#   单跑时这一道本来就不交，那份样本照样判绿，证不了红。
# 另一个 HARNESS_TEST_ENVIRONMENT_BREAK=cross-run-unscanned：GATE_CROSS_RUN_TMPDIR 设了也不交 --cross-run-temporary-directory。
#   设好 GATE_CROSS_RUN_TMPDIR 喂样本时 machine-clean-cross-run-green 判红、退 1；没设时这一道本来就不交，那份样本照样判绿。
#
#   bash .claude/gate.d/harness-test-environment.sh [项目根]
#   bash .claude/gate.d/harness-test-environment.sh --list
#   bash .claude/gate.d/harness-test-environment.sh --check machine-clean [项目根]
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_DIR="$(cd "$(dirname "$0")" && pwd)"
STAGE_NAME="$(basename "$0")"
source "$STAGE_DIR/lib/stage-cells.sh"

stage_cell machine-clean cell_machine_clean "跑完测试之后机器还干不干净" \
  "残留那几类先跑 python3 scripts/hosts/test-environment-check.py clean 看计划，确认没有正在跑的活在用它们之后加 --yes 再跑；宿主盘那几类（只读挂载、剩余空间、ext4 错误计数、内核日志）要人看；没查成的按它每一类后面那句「怎么办」补上再跑"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

root_argument=""
while (($#)); do
  case "$1" in
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check machine-clean 与一个项目根；格名用 bash .claude/gate.d/$STAGE_NAME --list 看。"
      exit 2 ;;
    *) root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$(cd "$STAGE_DIR/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级；路径写错或没有权限进去时这一道什么都没判。"; exit 2; }

SCRIPT="scripts/hosts/test-environment-check.py"

# 祖先 gate.sh 的启动时刻（epoch 秒）：晚于它的临时条目是这一趟门禁自己产生的，单列不判红。
# 不用本道开跑时刻：这一道排最后，那样会把前面各阶段的残留都放过。单跑（祖先里没有 gate.sh）不给分界、全判。
gate_started_epoch() {
  local boot_time clock_ticks pid parent start_ticks command_line
  boot_time="$(awk "/^btime/{print \$2}" /proc/stat 2>/dev/null)"
  clock_ticks="$(getconf CLK_TCK 2>/dev/null)"
  [[ -n "$boot_time" && -n "$clock_ticks" && "$clock_ticks" -gt 0 ]] || return 1
  pid=$$
  while [[ "$pid" != 1 && -r "/proc/$pid/stat" ]]; do
    # /proc/<pid>/stat 的第二个字段是可执行文件名、可能带空格，去掉「… ) 」之后 $2 才是父进程号、$20 是启动时刻
    parent="$(sed "s/.*) //" "/proc/$pid/stat" 2>/dev/null | awk "{print \$2}")"
    [[ -n "$parent" && "$parent" != 0 && -r "/proc/$parent/cmdline" ]] || return 1
    # 先把命令行落到变量再判：pipefail 下管道以 grep -q 收尾，命中时前段吃 SIGPIPE、整体返回 141，会被读成没命中
    command_line="$(tr "\\0" " " < "/proc/$parent/cmdline" 2>/dev/null)"
    if [[ "$command_line" == *gate.sh* ]]; then
      start_ticks="$(sed "s/.*) //" "/proc/$parent/stat" 2>/dev/null | awk "{print \$20}")"
      [[ -n "$start_ticks" ]] || return 1
      echo "$(( boot_time + start_ticks / clock_ticks ))"
      return 0
    fi
    pid="$parent"
  done
  return 1
}

# ── machine-clean ──
cell_machine_clean() {
  local script_rc started summary red_classes unchecked_summary unchecked_classes failed expected_rc
  log="$(mktemp)"
  trap 'rm -f "$log"' EXIT

  # 在 gate.sh 里：这一轮的 TMPDIR 交给 --gate-run-temporary-directory（文件头「射程」一段）
  local gate_run_arguments=()
  if [[ -n "${GATE_NOT_RUN_FILE:-}" && -n "${TMPDIR:-}" && "${HARNESS_TEST_ENVIRONMENT_BREAK:-}" != gate-run-scanned ]] \
    && [[ "$(realpath -m -- "$(dirname -- "$GATE_NOT_RUN_FILE")")" == "$(realpath -m -- "$TMPDIR")" ]]; then
    gate_run_arguments=(--gate-run-temporary-directory "$TMPDIR")
  fi
  # gate.sh 开跑前的 TMPDIR（GATE_CROSS_RUN_TMPDIR，跨轮复用的缓存放那里）：不是 /tmp 时不点名就从来不扫，点名交给 --cross-run-temporary-directory
  # （临时目录残留这一类另扫它，判法照旧；与 /tmp、这一轮的 TMPDIR 是同一处时脚本只扫一次）
  if [[ -n "${GATE_CROSS_RUN_TMPDIR:-}" && "${HARNESS_TEST_ENVIRONMENT_BREAK:-}" != cross-run-unscanned ]]; then
    gate_run_arguments+=(--cross-run-temporary-directory "$GATE_CROSS_RUN_TMPDIR")
  fi
  # 脚本的退出码要留着：它在「判红 0 类、没查成 ≥1 类」时退 3，吞掉它就只剩汇总行里的判红数可读
  script_rc=""
  if [[ -f "$SCRIPT" ]]; then
    if started="$(gate_started_epoch)"; then
      if python3 "$SCRIPT" check --produced-after "$started" ${gate_run_arguments[@]+"${gate_run_arguments[@]}"} >"$log" 2>&1; then script_rc=0; else script_rc=$?; fi
    else
      if python3 "$SCRIPT" check ${gate_run_arguments[@]+"${gate_run_arguments[@]}"} >"$log" 2>&1; then script_rc=0; else script_rc=$?; fi
    fi
  elif [[ -f test-environment-check-output.log ]]; then
    cp test-environment-check-output.log "$log"
  else
    echo "  ✗ 没有 $SCRIPT：它随仓走，不在就是被删了或挪了，机器干不干净这一轮没人查"
    echo "     → 怎么办：从 git 找回它（git log --diff-filter=D -- $SCRIPT 找删它的提交）；挪了地方就改这一道的 SCRIPT。"
    exit 1
  fi

  summary="$(grep -o '判红 [0-9]* 类' "$log" | tail -1)"
  red_classes="${summary#判红 }"
  red_classes="${red_classes% 类}"
  unchecked_summary="$(grep -o '没查成 [0-9]* 类' "$log" | tail -1)"
  unchecked_classes="${unchecked_summary#没查成 }"
  unchecked_classes="${unchecked_classes% 类}"
  if [[ -z "$summary" || -z "$unchecked_summary" ]]; then
    tail -12 "$log"
    echo "  ✗ 环境检查没报出完整的汇总行（「查了 N 类；✓ … 判红 N 类 … 没查成 N 类」），分不出「查过了没问题」与「压根没查成」"
    echo "     → 怎么办：单跑 python3 $SCRIPT check 看它停在哪；改过它的输出格式就把这一道的取法一起改。"
    exit 1
  fi
  failed=0
  if [[ "$red_classes" != 0 ]]; then
    grep -E '✗|→|汇总：' "$log" | head -40
    echo "  ✗ 机器上还有测试残留或宿主盘异常（判红 $red_classes 类，上面是逐类的判定）"
    echo "     → 怎么办：先跑 python3 $SCRIPT clean 看计划，确认没有正在跑的活在用它们之后加 --yes 再跑一次；"
    echo "               宿主盘那几类（只读挂载、剩余空间、ext4 错误计数、内核日志）不是 clean 能修的，要人看。"
    failed=1
  fi
  if [[ "$unchecked_classes" != 0 ]]; then
    grep -E '^ *!|→|汇总：' "$log" | head -40
    echo "  ✗ 环境检查有 $unchecked_classes 类没查成，分不出机器那几样干不干净（上面「!」开头的是哪几类、为什么）"
    echo "     → 怎么办：按每一类后面那句「怎么办」补上它缺的（权限、工具、读得到内核日志）再跑；"
    echo "               查不成就是这一次没看，不是看过了没问题——这一道不替它报绿。"
    failed=1
  fi
  if [[ -n "$script_rc" ]]; then
    expected_rc=0
    if [[ "$red_classes" != 0 ]]; then expected_rc=1; elif [[ "$unchecked_classes" != 0 ]]; then expected_rc=3; fi
    if [[ "$script_rc" != "$expected_rc" ]]; then
      tail -5 "$log"
      echo "  ✗ 环境检查退出码是 $script_rc，而它的汇总行说该是 $expected_rc（判红 $red_classes 类、没查成 $unchecked_classes 类）"
      echo "     → 怎么办：单跑 python3 $SCRIPT check; echo \$? 看它在哪一步出的错；退出码与汇总对不上时，这一道信不过那一行汇总。"
      failed=1
    fi
  fi
  if ((failed)); then
    exit 1
  fi
  tail -1 "$log"
}

stage_cells_run "$ROOT"
