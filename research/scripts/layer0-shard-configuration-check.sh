#!/usr/bin/env bash
# 层 0 崩溃重放双机分片（里程碑三第六项，.claude/kb/milestone/03-third-txn.md 第六节）的本地配置：在不在、键齐不齐、写得对不对、
# 第二台连不连得上、它上面有没有 cargo。门禁 54 号 --full 拿它定走不走分片；research/scripts/layer0-shard-run.sh 拿它当运行条件。
# PEER_MEMORY_CAP 是第二台那一片的内存上限（驱动脚本在第二台上经 research/scripts/run-with-memory-cap.sh 起那一片；本机那一片由起 54 号的那一层包装管），
# 写法照 run-with-memory-cap.sh：正整数加 K / M / G / T，单位必写。
#
#   layer0-shard-configuration-check.sh [<仓根>]
#       退 0 能分片（stdout 一句：配置在哪、第二台是谁）；退 1 不能（stdout 一句原因，下一行是出路）。
#   layer0-shard-configuration-check.sh --emit-assignments [<仓根>]
#       判法同上；能分片时 stdout 改打配置里每个键的 bash 赋值（printf %q，调用方 eval），不打那一句。
#
# 配置文件：${SINGLEFS_LAYER0_SHARD_CONFIG:-<主工作树的根>/layer0-shard.env}。主工作树的根按 git common-dir 取（在 HEAD + 暂存区的
# 临时 worktree 里跑也读主工作树那一份）；文件 git 忽略、不进仓，模板是仓根的 layer0-shard.env.example。
# 写法：一行一个 KEY=值（值不做 shell 展开；两头成对的单引号或双引号去掉一层），# 起头的行与空行不算；认不出的行、不认得的键、
# 缺键都判不能分片。键与各自的意思见 layer0-shard.env.example。
# 只供测试的开关：SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1 时不连第二台、不查它的 cargo（layer0-shard-run.sh --selftest 用，
# 那时「第二台」是本机上的另一个目录）。
#
# admission: always 判的是这一刻配置文件与第二台的样子，每次调都要现判
# run-condition: command git bash
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
layer0_shard_check_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

LAYER0_SHARD_CONFIGURATION_KEYS=(PEER_SSH_HOST PEER_REPOSITORY_DIRECTORY PEER_CARGO_BIN_DIRECTORY PEER_MEMORY_CAP
  QUIESCE_STOP_COMMAND QUIESCE_STOPPED_CHECK_COMMAND QUIESCE_START_COMMAND QUIESCE_STARTED_CHECK_COMMAND)

emit_assignments=0
if [[ "${1:-}" == --emit-assignments ]]; then emit_assignments=1; shift; fi
repository_root="${1:-$layer0_shard_check_script_directory/../..}"

refuse() { # refuse <原因>：打原因与出路，退 1
  echo "  ✗ 双机分片不能用：$1"
  echo "     → 怎么办：照仓根 layer0-shard.env.example 建本地配置 layer0-shard.env（git 忽略），或设 SINGLEFS_LAYER0_SHARD_CONFIG 指到它；第二台连不上先修 ssh 免密登录"
  exit 1
}

if [[ -n "${SINGLEFS_LAYER0_SHARD_CONFIG:-}" ]]; then
  configuration_file="$SINGLEFS_LAYER0_SHARD_CONFIG"
else
  if ! common_directory="$(git -C "$repository_root" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" || [[ -z "$common_directory" ]]; then
    refuse "$repository_root 不是 git 工作树，找不到主工作树的根（配置默认在那里）"
  fi
  configuration_file="$(dirname "$common_directory")/layer0-shard.env"
fi
[[ -f "$configuration_file" ]] || refuse "没有配置文件 $configuration_file"

declare -A configuration=()
line_number=0
while IFS= read -r configuration_line || [[ -n "$configuration_line" ]]; do
  line_number=$((line_number + 1))
  [[ "$configuration_line" =~ ^[[:space:]]*(#.*)?$ ]] && continue
  [[ "$configuration_line" =~ ^([A-Z_]+)=(.*)$ ]] || refuse "$configuration_file 第 $line_number 行认不出（要是 KEY=值）"
  configuration_key="${BASH_REMATCH[1]}"
  configuration_value="${BASH_REMATCH[2]}"
  if [[ "$configuration_value" =~ ^\'(.*)\'$ || "$configuration_value" =~ ^\"(.*)\"$ ]]; then configuration_value="${BASH_REMATCH[1]}"; fi
  known_key=0
  for expected_key in "${LAYER0_SHARD_CONFIGURATION_KEYS[@]}"; do [[ "$expected_key" == "$configuration_key" ]] && known_key=1; done
  (( known_key )) || refuse "$configuration_file 第 $line_number 行的键 $configuration_key 不认得（认的是 ${LAYER0_SHARD_CONFIGURATION_KEYS[*]}）"
  [[ -z "${configuration[$configuration_key]+set}" ]] || refuse "$configuration_file 里 $configuration_key 写了两遍"
  configuration[$configuration_key]="$configuration_value"
done < "$configuration_file"
missing_keys=()
for expected_key in "${LAYER0_SHARD_CONFIGURATION_KEYS[@]}"; do
  [[ -n "${configuration[$expected_key]+set}" ]] || missing_keys+=("$expected_key")
done
(( ${#missing_keys[@]} == 0 )) || refuse "$configuration_file 缺键 ${missing_keys[*]}（不清场的两对写成空串，键照样要写）"
[[ -n "${configuration[PEER_SSH_HOST]}" ]] || refuse "PEER_SSH_HOST 是空的"
for directory_key in PEER_REPOSITORY_DIRECTORY PEER_CARGO_BIN_DIRECTORY; do
  [[ "${configuration[$directory_key]}" == /* ]] || refuse "$directory_key 要写第二台上的绝对路径，读到「${configuration[$directory_key]}」"
done
[[ "${configuration[PEER_MEMORY_CAP]}" =~ ^[1-9][0-9]*[KMGT]$ ]] \
  || refuse "PEER_MEMORY_CAP 要写第二台那一片的内存上限（正整数加 K / M / G / T，例 24G，单位必写），读到「${configuration[PEER_MEMORY_CAP]}」"
for command_pair in "QUIESCE_STOP_COMMAND QUIESCE_STOPPED_CHECK_COMMAND" "QUIESCE_START_COMMAND QUIESCE_STARTED_CHECK_COMMAND"; do
  read -r action_key check_key <<< "$command_pair"
  if [[ -n "${configuration[$action_key]}" && -z "${configuration[$check_key]}" ]] || [[ -z "${configuration[$action_key]}" && -n "${configuration[$check_key]}" ]]; then
    refuse "$action_key 与 $check_key 要么都写、要么都空：改了状态的命令跑完要回读一次"
  fi
done
if [[ -n "${configuration[QUIESCE_STOP_COMMAND]}" && -z "${configuration[QUIESCE_START_COMMAND]}" ]]; then
  refuse "写了 QUIESCE_STOP_COMMAND 就要写 QUIESCE_START_COMMAND：清了场要复原"
fi

peer_description="第二台 ${configuration[PEER_SSH_HOST]}"
if [[ "${SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE:-0}" == 1 ]]; then
  peer_description="第二台是本机（只供测试的开关 SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1）"
else
  ssh -o BatchMode=yes -o ConnectTimeout=10 "${configuration[PEER_SSH_HOST]}" true </dev/null >/dev/null 2>&1 \
    || refuse "ssh -o BatchMode=yes ${configuration[PEER_SSH_HOST]} true 连不上"
  ssh -o BatchMode=yes -o ConnectTimeout=10 "${configuration[PEER_SSH_HOST]}" "test -x $(printf '%q' "${configuration[PEER_CARGO_BIN_DIRECTORY]}")/cargo" </dev/null >/dev/null 2>&1 \
    || refuse "第二台上没有 ${configuration[PEER_CARGO_BIN_DIRECTORY]}/cargo（PEER_CARGO_BIN_DIRECTORY 要指到它的 ~/.cargo/bin）"
fi

if (( emit_assignments )); then
  for expected_key in "${LAYER0_SHARD_CONFIGURATION_KEYS[@]}"; do
    printf '%s=%q\n' "$expected_key" "${configuration[$expected_key]}"
  done
  printf 'LAYER0_SHARD_CONFIGURATION_FILE=%q\n' "$configuration_file"
else
  echo "配置 $configuration_file，$peer_description"
fi
