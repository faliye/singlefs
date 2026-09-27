#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 它调的 research/scripts/test-environment-check.py 读内核日志、SMART 这类宿主信息，读不到的那一类它自己记「没查成」
# gate-stage: 跑完测试之后机器还干不干净（残留测试设备、临时目录残留、宿主盘）
#
# 判据：跑 `research/scripts/test-environment-check.py check`，它判 10 类——残留的 loop / dm 设备、
# 残留挂载、带 singlefs 的 qemu 进程、`/tmp` 下的残留镜像与目录（白名单之外的）、宿主盘只读挂载、
# 剩余空间、ext4 的 errors_count、内核日志里落在宿主盘栈上的块设备错误、SMART（要 --smart 才跑）。
# 这一阶段读它汇总行里的「判红 N 类」与「没查成 N 类」：判红不是 0 就红，出路是它自己打印的 `clean` 计划；
# 没查成不是 0 也红（那几类这一次没看，不是看过了没问题）。真跑那一支还核脚本的退出码与汇总对得上
# （它的约定：1 有判红、3 只有没查成、0 都查过都干净），对不上判红。
# 汇总行读不到（脚本没跑成、输出被截断）也红——分不出「查过了没问题」与「压根没查成」时不许报绿。
#
# 为什么：用户 2026-09-19 定「每一轮的里程碑都应该要有一个干净的环境，不应该受到上一轮的干扰；
# 每次里程碑完结后要检查挂载的盘有没有异常」，2026-09-20 再定「改了代码重新 mkfs、新的虚拟环境、
# 新的镜像、重新测试，门禁和崩溃全部重放重跑」。写成一句提醒拦不住，所以做成会红的阶段（C396 的落点）。
# 2026-09-20 接上这一阶段时现查：`/tmp` 里躺着 34 个当天跑测试留下的镜像（创建者进程都已不在），
# 而 `FileBackedBlockDevice` 当时用的是 open_or_create——进程号被系统复用时会默默接着用旧镜像。
#
# 射程：它查的是这台机器此刻的样子，不查「这一轮有没有重新 mkfs」——那一条由用例自己的排他创建守着。
# SMART 默认不跑（要 smartctl 与权限），阶段里也不给 --smart：没跑的它会逐个列名。
#
# 样本：被判目录里没有 `research/scripts/test-environment-check.py` 而有 `test-environment-check-output.log` 时，
# 不跑脚本、只判那份录好的输出（`.claude/gate.d/fixtures/77-test-environment.sh/` 的红绿样本走这一支）。
# 脚本与录好的输出都没有：判红，不当跳过——它随仓走，不在就是本该在仓里的东西不在（与 57、70、73 同一条）。
#
#   bash .claude/gate.d/77-test-environment.sh [项目根]
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级；路径写错或没有权限进去时这一道什么都没判。"; exit 2; }

SCRIPT="research/scripts/test-environment-check.py"
log="$(mktemp)"
trap 'rm -f "$log"' EXIT

# 本次门禁的开跑时刻（epoch 秒）：晚于它才出现的临时条目是这一次跑自己产生的，单列一档不判红（C448）。
# 取法是往上找祖先里那个 gate.sh 进程、读它的启动时刻——门禁没有导出开跑时刻的环境变量，而这一阶段
# 排在最后，用「本阶段开跑时刻」当分界会把前面每个阶段留下的都算进去，方向正好反了。
# 单跑这一阶段（祖先里没有 gate.sh）时算不出来，就不给分界、照旧全判。
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
  echo "     → 怎么办：从 git 找回它（git log --diff-filter=D -- $SCRIPT 找删它的提交）；挪了地方就改这一阶段的 SCRIPT。"
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
  echo "     → 怎么办：单跑 python3 $SCRIPT check 看它停在哪；改过它的输出格式就把这一阶段的取法一起改。"
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
  echo "               查不成就是这一次没看，不是看过了没问题——这一阶段不替它报绿。"
  failed=1
fi
if [[ -n "$script_rc" ]]; then
  expected_rc=0
  if [[ "$red_classes" != 0 ]]; then expected_rc=1; elif [[ "$unchecked_classes" != 0 ]]; then expected_rc=3; fi
  if [[ "$script_rc" != "$expected_rc" ]]; then
    tail -5 "$log"
    echo "  ✗ 环境检查退出码是 $script_rc，而它的汇总行说该是 $expected_rc（判红 $red_classes 类、没查成 $unchecked_classes 类）"
    echo "     → 怎么办：单跑 python3 $SCRIPT check; echo \$? 看它在哪一步出的错；退出码与汇总对不上时，这一阶段信不过那一行汇总。"
    failed=1
  fi
fi
if ((failed)); then
  exit 1
fi
tail -1 "$log"
