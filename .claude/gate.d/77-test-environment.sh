#!/usr/bin/env bash
# gate-stage: 跑完测试之后机器还干不干净（残留测试设备、临时目录残留、宿主盘）
#
# 判据：跑 `research/scripts/test-environment-check.py check`，它判 10 类——残留的 loop / dm 设备、
# 残留挂载、带 singlefs 的 qemu 进程、`/tmp` 下的残留镜像与目录（白名单之外的）、宿主盘只读挂载、
# 剩余空间、ext4 的 errors_count、内核日志里落在宿主盘栈上的块设备错误、SMART（要 --smart 才跑）。
# 这一阶段读它汇总行里的「判红 N 类」：N 不是 0 就红，出路是它自己打印的 `clean` 计划。
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
#
#   bash .claude/gate.d/77-test-environment.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2

SCRIPT="research/scripts/test-environment-check.py"
log="$(mktemp)"

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

if [[ -f "$SCRIPT" ]]; then
  if started="$(gate_started_epoch)"; then
    python3 "$SCRIPT" check --produced-after "$started" >"$log" 2>&1 || true
  else
    python3 "$SCRIPT" check >"$log" 2>&1 || true
  fi
elif [[ -f test-environment-check-output.log ]]; then
  cp test-environment-check-output.log "$log"
else
  rm -f "$log"
  echo "  ! 没有 $SCRIPT，本阶段跳过"
  exit 77
fi

summary="$(grep -o '判红 [0-9]* 类' "$log" | tail -1)"
red_classes="${summary#判红 }"
red_classes="${red_classes% 类}"
if [[ -z "$summary" ]]; then
  tail -12 "$log"
  rm -f "$log"
  echo "  ✗ 环境检查没报出汇总行（「查了 N 类；✓ … 判红 N 类」），分不出「查过了没问题」与「压根没查成」"
  echo "     → 怎么办：单跑 python3 $SCRIPT check 看它停在哪；改过它的输出格式就把这一阶段的取法一起改。"
  exit 1
fi
if [[ "$red_classes" != 0 ]]; then
  grep -E '✗|→|汇总：' "$log" | head -40
  rm -f "$log"
  echo "  ✗ 机器上还有测试残留或宿主盘异常（判红 $red_classes 类，上面是逐类的判定）"
  echo "     → 怎么办：先跑 python3 $SCRIPT clean 看计划，确认没有正在跑的活在用它们之后加 --yes 再跑一次；"
  echo "               宿主盘那几类（只读挂载、剩余空间、ext4 错误计数、内核日志）不是 clean 能修的，要人看。"
  exit 1
fi
tail -1 "$log"
rm -f "$log"
