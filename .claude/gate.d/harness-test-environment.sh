#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 它调的 research/scripts/test-environment-check.py 读内核日志、SMART 这类宿主信息，读不到的那一类它自己记「没查成」
# gate-stage: 跑完测试之后机器还干不干净（残留测试设备、临时目录残留、宿主盘）
# gate-category: harness 类
# gate-similar: harness-model-differential-and-scenarios.sh 同属 harness 类，它跑 crates/singlefs-harness 的测试并判模型对拍；这一道不跑测试，判的是测试跑完之后留在机器上的东西
# gate-similar: code-tooling.sh 它的 research-script-selftests 格只跑 test-environment-check.py 的 --selftest（判别力），不看这台机器此刻的样子；它的 stage-order 格只判这一道的文件名排在跑测试的阶段之后，不读机器；这里在真机上判
# gate-similar: 54-layer0-replay.sh 它跑崩溃点重放时自己建镜像、自己收拾，判的是重放的结局；机器上残不残留、宿主盘好不好，由这一道在全部跑测试的阶段之后统一判
# gate-similar: checker-tier-qemu-device-streams.sh 它起 QEMU、判设备流的结局；带 singlefs 的 qemu 进程跑完还在不在，由这一道判
#
# 一格。格名表（--list 打它，--check machine-clean 只跑这一格，格名写错退 2）：
# gate-cell: machine-clean 跑完测试之后机器还干不干净
# 参数解析、样本根的 .gate-cells、子 shell、汇总与退出码都经共用库 lib/stage-cells.sh（写法与判据在它的文件头），这里只登记格、解析项目根。
#
# 次序：这一道要排在全部跑测试的阶段之后。gate.sh 取门禁目录顶层的 *.sh、按文件名 sort 之后逐道起，
# 去编号之后以数字起头的 `54-layer0-replay.sh` 排最前，其余按类名 checker-* < code-* < doc-* < harness-*，
# 这一道在 harness 类里字母序也排最后（排在 harness-model-differential-and-scenarios.sh 之后；C 与 en_US.UTF-8 两种 locale 次序相同）。
# 依赖次序的是与时刻无关的几类：残留的 loop / dm 设备、残留挂载、带 singlefs 的 qemu 进程、宿主盘（只读挂载、剩余空间、ext4 错误计数、内核日志），
# 判的是这一道开跑那一刻的机器，排在它后面的阶段干的事这一趟看不见（攻方 research/prompts/gate-shrink-r1-opus-output.md 第 153 行）。
# 临时目录那一类不依赖次序：它用祖先 gate.sh 的开跑时刻当分界（见下面 gate_started_epoch），晚于分界的条目这一趟不判、下一趟判。
# 新加跑测试的阶段、改名时，名字要排在这一道前面；判次序的那一格在工具层自检那一道里，不在这里。
#
# 判据：跑 `research/scripts/test-environment-check.py check`，它判 10 类——残留的 loop / dm 设备、
# 残留挂载、带 singlefs 的 qemu 进程、`/tmp` 下的残留镜像与目录（白名单之外的）、宿主盘只读挂载、
# 剩余空间、ext4 的 errors_count、内核日志里落在宿主盘栈上的块设备错误、SMART（要 --smart 才跑）。
# 这一格读它汇总行里的「判红 N 类」与「没查成 N 类」：判红不是 0 就红，出路是它自己打印的 `clean` 计划；
# 没查成不是 0 也红（那几类这一次没看，不是看过了没问题）。真跑那一支还核脚本的退出码与汇总对得上
# （它的约定：1 有判红、3 只有没查成、0 都查过都干净），对不上判红。
# 汇总行读不到（脚本没跑成、输出被截断）也红——分不出「查过了没问题」与「压根没查成」时不许报绿。
#
# 为什么：用户 2026-09-19 定「每一轮的里程碑都应该要有一个干净的环境，不应该受到上一轮的干扰；
# 每次里程碑完结后要检查挂载的盘有没有异常」，2026-09-20 再定「改了代码重新 mkfs、新的虚拟环境、
# 新的镜像、重新测试，门禁和崩溃全部重放重跑」。写成一句提醒拦不住，所以做成会红的门禁（C396 的落点）。
# 2026-09-20 接上它时现查：`/tmp` 里躺着 34 个当天跑测试留下的镜像（创建者进程都已不在），
# 而 `FileBackedBlockDevice` 当时用的是 open_or_create——进程号被系统复用时会默默接着用旧镜像。
#
# 射程：它查的是这台机器此刻的样子，不查「这一轮有没有重新 mkfs」——那一条由用例自己的排他创建守着。
# SMART 默认不跑（要 smartctl 与权限），这里也不给 --smart：没跑的它会逐个列名。
#
# 样本：被判目录里没有 `research/scripts/test-environment-check.py` 而有 `test-environment-check-output.log` 时，
# 不跑脚本、只判那份录好的输出（fixtures/harness-test-environment.sh/ 的样本走这一支，每份的 .gate-cells 点名 machine-clean）：
#   machine-clean-red    判红 1 类（临时目录残留）、没查成 1 类
#   machine-clean-green  10 类都查过、判红 0 类
# 脚本与录好的输出都没有：判红，不当跳过——它随仓走，不在就是本该在仓里的东西不在。
# 汇总的弄坏开关归共用库（STAGE_CELLS_BREAK=red-swallowed 让 machine-clean-red 退 0）。
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
  "残留那几类先跑 python3 research/scripts/test-environment-check.py clean 看计划，确认没有正在跑的活在用它们之后加 --yes 再跑；宿主盘那几类（只读挂载、剩余空间、ext4 错误计数、内核日志）要人看；没查成的按它每一类后面那句「怎么办」补上再跑"
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

SCRIPT="research/scripts/test-environment-check.py"

# 本次门禁的开跑时刻（epoch 秒）：晚于它才出现的临时条目是这一次跑自己产生的，单列一档不判红（C448）。
# 取法是往上找祖先里那个 gate.sh 进程、读它的启动时刻——门禁没有导出开跑时刻的环境变量，而这一道
# 排在最后，用「本道开跑时刻」当分界会把前面每个阶段留下的都算进去，方向正好反了。
# 单跑这一道（祖先里没有 gate.sh）时算不出来，就不给分界、照旧全判。
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

  # 脚本的退出码要留着：它在「判红 0 类、没查成 ≥1 类」时退 3，吞掉它就只剩汇总行里的判红数可读
  script_rc=""
  if [[ -f "$SCRIPT" ]]; then
    if started="$(gate_started_epoch)"; then
      if python3 "$SCRIPT" check --produced-after "$started" >"$log" 2>&1; then script_rc=0; else script_rc=$?; fi
    else
      if python3 "$SCRIPT" check >"$log" 2>&1; then script_rc=0; else script_rc=$?; fi
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
