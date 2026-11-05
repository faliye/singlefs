#!/usr/bin/env bash
# watchdog-task-half-r1 攻方腿的活探针：一条「日志一直在涨、CPU 一直占着」的任务，跑到记录用时的很多倍，服务停不停它、wait 叫不叫醒。
# 用法（主 agent 交两次，两次命令逐词相同，服务才把头一次的用时记成第二次的预计用时）：
#   task-server/task submit -- bash <模型目录>/overrun-probe.sh <状态目录>
#   头一次：状态目录里没有 quick-run-done，睡 5 秒、建 quick-run-done、退 0（服务记下这条命令约 5 秒跑完）；
#   第二次：有 quick-run-done，忙等 BUSY_SECONDS 秒（一个核占满），每 5 秒往标准输出打一行（日志一直在涨），到点退 0。
# 读法：第二次那一条在 1200 秒前后既不该被停（日志在涨，stalled 两条都要日志满 600 秒不涨），也不该有 TASK_SERVER_SUSPECT（CPU 在涨）；
#   改前的看门狗在已跑到预计 5 秒的 3 倍、且多出 300 秒（305 秒）时就报「跑得远超预计」。
# 只有本 shell 一个进程（忙等用 bash 自己的算术与 printf，不起子进程；头一次的 sleep 是唯一的子进程、5 秒退）。
set -euo pipefail
BUSY_SECONDS=1500
state="${1:?要给状态目录}"
mkdir -p "$state"
if [[ ! -e "$state/quick-run-done" ]]; then
  echo "PROBE phase=quick start"
  sleep 5
  : > "$state/quick-run-done"
  echo "PROBE phase=quick end"
  exit 0
fi
echo "PROBE phase=busy start busy_seconds=$BUSY_SECONDS"
end=$((SECONDS + BUSY_SECONDS))
next=$SECONDS
spins=0
while ((SECONDS < end)); do
  spins=$((spins + 1))
  if ((SECONDS >= next)); then
    printf 'PROBE tick elapsed=%s spins=%s\n' "$SECONDS" "$spins"
    next=$((SECONDS + 5))
  fi
done
echo "PROBE phase=busy end elapsed=$SECONDS"
