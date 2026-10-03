#!/usr/bin/env bash
# 模型 V3（写满盘 / 日志不可写）：不经 systemd，把 detached-run.sh 里单元那个 bash 的代码（三段 RUNNER）原样抠出来在本进程里跑。
#   甲：命令正常退 3，但它退出之前登记目录变成写不进（模型：chmod a-w；盘满时 printf 写不进、&& mv 不跑，走同一条路，是推的）
#       ⇒ 登记里没有 exit；再用替身 systemctl（单元已不在）跑 list，看它把原因报成什么。
#   乙：--log 指到写不进的目录里的文件（start 只核目录在不在）⇒ 命令一行都没跑，登记里 exit=1、没有 pid；
#       start 回读那一段（第 407–419 行）看到 exit 就打「✓ 起了…命令进程（已经跑完）」、退 0。
# 用法：bash model-v3-runner-writes-fail.sh <detached-run.sh 的路径>
set -u
script="$1"; here="$(cd "$(dirname "$0")" && pwd)"; source "$here/shim-lib.sh"
work="$(mktemp -d "${TMPDIR:-/tmp}/opusr2-v3w.XXXXXX")"; trap 'chmod -R u+w "$work"; rm -rf -- "$work"' EXIT
runner="$(awk '/^read -r -d .. DETACHED_RUN_RUNNER_(HEAD|EXIT) <</{f=1;next} /^RUNNER$/{f=0} f' "$script")"$'\n''exit "$rc"'
opusr2_shim "$work/shim"; state="$work/state"; mkdir -p "$state"
echo "== 甲"
opusr2_record "$state" opusr2v3a; rec="$state/opusr2v3a"
bash -c "$runner" detached-run-runner "$rec" "$rec/log" bash -c 'chmod a-w "$0"; echo 命令的最后一行; exit 3' "$rec" 2> "$work/a.err"
echo "单元那个 bash 退 $?；登记里：$(ls "$rec" | tr '\n' ' ')；它的报错：$(head -c 160 "$work/a.err" | tr '\n' ' ')"
PATH="$work/shim/bin:$PATH" DETACHED_RUN_STATE_DIRECTORY="$state" bash "$script" list 2>&1 | grep -E '没留退出码|已结束' | cut -c1-150
echo "== 乙"
mkdir -p "$work/ro"; chmod 555 "$work/ro"
opusr2_record "$state" opusr2v3b; rec="$state/opusr2v3b"
bash -c "$runner" detached-run-runner "$rec" "$work/ro/run.log" touch "$work/marker-b" 2> "$work/b.err"
echo "单元那个 bash 退 $?；登记里 pid：$(cat "$rec/pid" 2>/dev/null || echo 没有)  exit：$(cat "$rec/exit")；命令跑没跑（marker）：$([[ -e "$work/marker-b" ]] && echo 跑了 || echo 没跑)；它的报错：$(head -c 160 "$work/b.err" | tr '\n' ' ')"
pid=""; [[ -f "$rec/pid" ]] && pid="$(cat "$rec/pid")"
if [[ -z "$pid" && ! -f "$rec/exit" ]]; then echo "start 回读：判没起成、退 1"; else echo "start 回读（照第 414–419 行）：✓ 起了…命令进程 ${pid:-（已经跑完）}，退 0"; fi
