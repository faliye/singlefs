#!/usr/bin/env bash
# 崩溃点重放双机分片的本地配置：在不在、键齐不齐、写得对不对、
# 第二台连不连得上、它上面有没有 cargo，双机与 GPU 两个开关开没开。门禁 checker-tier-crash-point-replay --full 拿它定走不走分片；scripts/crash/crash-point-replay-shard-run.sh 拿它当运行条件；
# 门禁 checker-tier-crates-mutation-replay 的双机驱动 scripts/mutation/mutation-shard-run.sh 与通用跨机脚本 scripts/hosts/multi-host-run.sh --peer 读同一份。
# PEER_MEMORY_CAP 是第二台那一片的内存上限（驱动脚本在第二台上经 scripts/hosts/run-with-memory-cap.sh 起那一片；本机那一片由起 checker-tier-crash-point-replay 的那一层包装管），
# 写法照 run-with-memory-cap.sh：正整数加 K / M / G / T，单位必写。
#
#   crash-point-replay-shard-configuration-check.sh [<仓根>]
#       退 0 双机：开（stdout 一句：双机与 GPU 开没开、配置在哪、第二台是谁）；退 3 双机：关（配置判得过，但 ENABLE_ACROSS_MACHINES 不是 1：
#       调用方照单机跑；stdout 一句「双机：关」与怎么打开，不连第二台）；退 1 判不过（stdout 一句原因，下一行是出路）。
#   crash-point-replay-shard-configuration-check.sh --emit-assignments [<仓根>]
#       判法同上；双机开着时 stdout 改打配置里每个键的 bash 赋值（printf %q，调用方 eval；两个开关键没写的打成 0），不打那一句；
#       双机关着时照样打「双机：关」那一句、退 3，不打赋值。
#   crash-point-replay-shard-configuration-check.sh --emit-hosts [<仓根>]
#       判法同上，只是不连任何一台（PEER_SSH_HOST 也不连）：清单里每一台连不连得上由调用方逐台报（scripts/hosts/host-resources.py 报「没读到」），
#       一台连不上不算整份配置坏。双机开着时 stdout 打 --emit-assignments 那几行，另打机器清单（下面「机器清单」）解出来的每一台：
#       MULTI_HOST_COUNT=<除本机外几台>，以及第 2 台起每台的 MULTI_HOST_<号>_SSH_HOST、MULTI_HOST_<号>_REPOSITORY_DIRECTORY、
#       MULTI_HOST_<号>_CARGO_BIN_DIRECTORY、MULTI_HOST_<号>_MEMORY_CAP（没写的打空串）、MULTI_HOST_<号>_IS_PEER（这台就是 PEER_SSH_HOST 打 1）。
#
# 两个开关键只在这一份里定义（别处照这里读，不另写一份）：
#   ENABLE_ACROSS_MACHINES  双机：写 1 才开；没写或写 0 算关（用户 2026-09-28 定：不设参数不主动开）。关着时不连第二台、退 3。
#   ENABLE_GPU              GPU：写 1 算开，没写或写 0 算关；这里只判写法、报开没开，用不用它归各调用方。
#   两个键只许写 0 或 1，写别的值判不过（退 1，说清是哪个键）。别的键缺一个都判不过，与开关开没开无关。
#
# 显卡的两个键（可以不写；用户 2026-10-10 定多机配置合成一份，私有显卡配置并进这一份）：GPU_CARDS_BY_PRIORITY、GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED。
#   这里只认这两个键名、不判值（值的写法与判法在 scripts/crash/crash-amplification-lib.sh 文件头「配置」与 crates/singlefs-checker-tier/src/gpu_card_configuration.rs），
#   --emit-assignments、--emit-hosts 不打它们。退役的 GPU_COMPUTATION_CLASSES 写进来照不认得的键判不过（开了 GPU 就三类全走 GPU，不再有正表）。
#
# 机器清单（可以不写；用户 2026-10-06 定「这样就不用限制多少台机器了」，键的定义只在这一份里）：
#   HOSTS                        除本机外的每一台的 ssh 别名，空格隔开，按先后编号：本机是第 1 台，HOSTS 里第一个是第 2 台，依次往下。
#                                没写或写空串时清单就是 PEER_SSH_HOST 一台（老用法）。别名只许字母、数字与 . _ @ -，不许重复。
#   HOST_<号>_REPOSITORY_DIRECTORY、HOST_<号>_CARGO_BIN_DIRECTORY   第 <号> 台上的目录（绝对路径），意思同 PEER_ 那两个；
#   HOST_<号>_MEMORY_CAP          第 <号> 台的内存上限，写法同 PEER_MEMORY_CAP。三个都可以不写；<号> 从 2 起、不带前导 0、不超过清单的台数。
#                                别名就是 PEER_SSH_HOST 的那一台，没写的按 PEER_ 那三个取。
#   崩溃点重放分片、变异分片与 multi-host-run.sh --peer 照旧只读 PEER_*（两台的老用法不变）；清单只给按台查资源的 scripts/hosts/host-resources.py 读（经 --emit-hosts）。
#
# 配置文件：${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}。主工作树的根按 git common-dir 取（在 HEAD + 暂存区的
# 临时 worktree 里跑也读主工作树那一份）；文件 git 忽略、不进仓，模板是仓根的 multi-host.env.example。
# 写法：一行一个 KEY=值（值不做 shell 展开；两头成对的单引号或双引号去掉一层），# 起头的行与空行不算；认不出的行、不认得的键、
# 缺键都判不能分片。八个必写键的意思见 multi-host.env.example；两个开关键见上。
# 清场一对（QUIESCE_STOP_COMMAND、QUIESCE_STOPPED_CHECK_COMMAND）与复原一对（QUIESCE_START_COMMAND、QUIESCE_STARTED_CHECK_COMMAND）各自要么都写、要么都空；
#   两对之间不要求成对：各脚本清场之后都不复原，复原那一对只给用户手动复原用，写不写都判得过。
# 弄坏开关 MULTI_HOST_CONFIGURATION_BREAK=<项>（只给证红用；打开时调用方的自证退 1）。这一份没有自证，红在两份调用方的 --selftest 里：
#   scripts/crash/crash-point-replay-shard-run.sh 与 scripts/hosts/multi-host-run.sh，两份的失败行都是「✗ <格名>：<细节>」。
# 逐项写开关名、弄坏哪一处、打开后哪一份自证的哪几格红（项名是格名，前面写是哪一份；括号里抄细节的句首，随跑变的路径写成 …）：
#   MULTI_HOST_CONFIGURATION_BREAK=switch-off-is-on  ENABLE_ACROSS_MACHINES 没写或写 0 也按开判
#       crash-point-replay-shard-run.sh 的「⑭ 写了 ENABLE_ACROSS_MACHINES=1 ⇒ 退 0、双机：开，GPU：关，赋值里 ENABLE_GPU=0；没写开关 ⇒ 判法报「双机：关」退 3（不是判不过），
#         --emit-assignments 同样退 3、不打赋值」（「写了 1：退 0 / emit 退 0（…）；没写：退 0 / emit 退 0（双机：开，…）」）
#       multi-host-run.sh 的「Ⓗ 配置里没写 ENABLE_ACROSS_MACHINES ⇒ --peer 拒（退 255，出路写 ENABLE_ACROSS_MACHINES=1），命令哪台都没跑、第二台没建目录」（「退 0，命令跑了」）
#   MULTI_HOST_CONFIGURATION_BREAK=switch-value-unchecked  两个开关键写了 0、1 之外的值也不报错
#       crash-point-replay-shard-run.sh 的「⑮ 开关写了 0、1 之外的值（ENABLE_ACROSS_MACHINES=yes、ENABLE_GPU=2）⇒ 判不过（退 1），说出是哪个键」
#         （「ENABLE_ACROSS_MACHINES=yes 退 3（双机：关（…）），ENABLE_GPU=2 退 0（双机：开，…）」）；multi-host-run.sh 的自证不红（没有写别的值的那一格）
#   MULTI_HOST_CONFIGURATION_BREAK=hosts-key-unknown  HOSTS 与 HOST_<号>_* 当不认得的键（改前的样子）
#       scripts/hosts/host-resources.py --selftest 的「hosts-list」「fits-on-host」两格（「…只报了 本机」「…实际结论「跑不起来：…」」）：配置判不过，只剩本机
#   MULTI_HOST_CONFIGURATION_BREAK=hosts-reach-refuses  --emit-hosts 也逐台 ssh，清单里有一台连不上就判整份配置坏
#       scripts/hosts/host-resources.py --selftest 的「hosts-list」「peer-unreachable」「host-timeout」三格（「…只报了 本机」「第二台应当没读到…」「时限 1 秒…」）
#   这两个开关打开时 crash-point-replay-shard-run.sh 与 multi-host-run.sh 的自证不红（它们的配置里没有 HOSTS，也不用 --emit-hosts）。
#   MULTI_HOST_CONFIGURATION_BREAK=gpu-card-keys-unknown  显卡的两个键当不认得的键（并进多机配置之前的样子）
#       scripts/crash/crash-amplification-item.sh --selftest 的第 22 格（「多机配置写了显卡的两个键却判不过…」）；crash-point-replay-shard-run.sh 与 multi-host-run.sh 的自证不红（它们的配置里没有这两个键）
# 只供测试的开关：SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 时不连第二台、不查它的 cargo（crash-point-replay-shard-run.sh --selftest 用，
# 那时「第二台」是本机上的另一个目录）。
#
# admission: always 判的是这一刻配置文件与第二台的样子，每次调都要现判
# run-condition: command git bash
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
crash_point_replay_shard_check_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

CRASH_POINT_REPLAY_SHARD_CONFIGURATION_KEYS=(PEER_SSH_HOST PEER_REPOSITORY_DIRECTORY PEER_CARGO_BIN_DIRECTORY PEER_MEMORY_CAP
  QUIESCE_STOP_COMMAND QUIESCE_STOPPED_CHECK_COMMAND QUIESCE_START_COMMAND QUIESCE_STARTED_CHECK_COMMAND)
MULTI_HOST_SWITCH_KEYS=(ENABLE_ACROSS_MACHINES ENABLE_GPU)  # 可以不写，不写算 0
MULTI_HOST_GPU_CARD_KEYS=(GPU_CARDS_BY_PRIORITY GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED)  # 可以不写，这里不判值（文件头「显卡的两个键」）
MULTI_HOST_MACHINE_KEY_PATTERN='^HOST_([1-9][0-9]*)_(REPOSITORY_DIRECTORY|CARGO_BIN_DIRECTORY|MEMORY_CAP)$'  # 机器清单里每台可选配的键（文件头「机器清单」）
configuration_break="${MULTI_HOST_CONFIGURATION_BREAK:-}"

emit_assignments=0
emit_hosts=0
if [[ "${1:-}" == --emit-assignments ]]; then emit_assignments=1; shift
elif [[ "${1:-}" == --emit-hosts ]]; then emit_hosts=1; shift; fi
repository_root="${1:-$crash_point_replay_shard_check_script_directory/../..}"

refuse() { # refuse <原因>：打原因与出路，退 1
  echo "  ✗ 双机分片不能用：$1"
  echo "     → 怎么办：照仓根 multi-host.env.example 建本地配置 multi-host.env（git 忽略），或设 SINGLEFS_MULTI_HOST_CONFIG 指到它；第二台连不上先修 ssh 免密登录"
  exit 1
}

if [[ -n "${SINGLEFS_MULTI_HOST_CONFIG:-}" ]]; then
  configuration_file="$SINGLEFS_MULTI_HOST_CONFIG"
else
  if ! common_directory="$(git -C "$repository_root" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" || [[ -z "$common_directory" ]]; then
    refuse "$repository_root 不是 git 工作树，找不到主工作树的根（配置默认在那里）"
  fi
  configuration_file="$(dirname "$common_directory")/multi-host.env"
fi
[[ -f "$configuration_file" ]] || refuse "没有配置文件 $configuration_file"

declare -A configuration=()
line_number=0
while IFS= read -r configuration_line || [[ -n "$configuration_line" ]]; do
  line_number=$((line_number + 1))
  [[ "$configuration_line" =~ ^[[:space:]]*(#.*)?$ ]] && continue
  [[ "$configuration_line" =~ ^([A-Z][A-Z0-9_]*)=(.*)$ ]] || refuse "$configuration_file 第 $line_number 行认不出（要是 KEY=值）"
  configuration_key="${BASH_REMATCH[1]}"
  configuration_value="${BASH_REMATCH[2]}"
  if [[ "$configuration_value" =~ ^\'(.*)\'$ || "$configuration_value" =~ ^\"(.*)\"$ ]]; then configuration_value="${BASH_REMATCH[1]}"; fi
  known_key=0
  for expected_key in "${CRASH_POINT_REPLAY_SHARD_CONFIGURATION_KEYS[@]}" "${MULTI_HOST_SWITCH_KEYS[@]}"; do [[ "$expected_key" == "$configuration_key" ]] && known_key=1; done
  if [[ "$configuration_break" != gpu-card-keys-unknown ]]; then
    for expected_key in "${MULTI_HOST_GPU_CARD_KEYS[@]}"; do [[ "$expected_key" == "$configuration_key" ]] && known_key=1; done
  fi
  if [[ "$configuration_break" != hosts-key-unknown ]] && [[ "$configuration_key" == HOSTS || "$configuration_key" =~ $MULTI_HOST_MACHINE_KEY_PATTERN ]]; then known_key=1; fi
  (( known_key )) || refuse "$configuration_file 第 $line_number 行的键 $configuration_key 不认得（认的是 ${CRASH_POINT_REPLAY_SHARD_CONFIGURATION_KEYS[*]} ${MULTI_HOST_SWITCH_KEYS[*]} ${MULTI_HOST_GPU_CARD_KEYS[*]}，与机器清单 HOSTS、HOST_<号>_REPOSITORY_DIRECTORY、HOST_<号>_CARGO_BIN_DIRECTORY、HOST_<号>_MEMORY_CAP）"
  [[ -z "${configuration[$configuration_key]+set}" ]] || refuse "$configuration_file 里 $configuration_key 写了两遍"
  configuration[$configuration_key]="$configuration_value"
done < "$configuration_file"
missing_keys=()
for expected_key in "${CRASH_POINT_REPLAY_SHARD_CONFIGURATION_KEYS[@]}"; do
  [[ -n "${configuration[$expected_key]+set}" ]] || missing_keys+=("$expected_key")
done
(( ${#missing_keys[@]} == 0 )) || refuse "$configuration_file 缺键 ${missing_keys[*]}（不清场、不手动复原的那一对写成空串，键照样要写）"
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

# 机器清单（文件头「机器清单」）：HOSTS 没写或写空串就只有 PEER_SSH_HOST 一台；每台可选配的键编号要落在清单里
machine_aliases=()
if [[ -n "${configuration[HOSTS]-}" ]]; then read -r -a machine_aliases <<< "${configuration[HOSTS]}"; fi
(( ${#machine_aliases[@]} > 0 )) || machine_aliases=("${configuration[PEER_SSH_HOST]}")
declare -A machine_alias_seen=()
for machine_alias in "${machine_aliases[@]}"; do
  [[ "$machine_alias" =~ ^[A-Za-z0-9][A-Za-z0-9._@-]*$ ]] || refuse "HOSTS 里有一个别名不像 ssh 别名（只许字母、数字与 . _ @ -）"
  [[ -z "${machine_alias_seen[$machine_alias]+set}" ]] || refuse "HOSTS 里同一个别名写了两遍"
  machine_alias_seen[$machine_alias]=1
done
for configuration_key in "${!configuration[@]}"; do
  [[ "$configuration_key" =~ $MULTI_HOST_MACHINE_KEY_PATTERN ]] || continue
  machine_number="${BASH_REMATCH[1]}"
  machine_field="${BASH_REMATCH[2]}"
  if (( machine_number < 2 || machine_number > ${#machine_aliases[@]} + 1 )); then
    refuse "$configuration_key 指的是第 $machine_number 台，清单里除本机外只有 ${#machine_aliases[@]} 台（编号从 2 起：本机是第 1 台，HOSTS 里第一个是第 2 台）"
  fi
  machine_value="${configuration[$configuration_key]}"
  [[ -n "$machine_value" ]] || continue
  if [[ "$machine_field" == MEMORY_CAP ]]; then
    [[ "$machine_value" =~ ^[1-9][0-9]*[KMGT]$ ]] || refuse "$configuration_key 要写正整数加 K / M / G / T（例 24G，单位必写），读到「$machine_value」"
  else
    [[ "$machine_value" == /* ]] || refuse "$configuration_key 要写第 $machine_number 台上的绝对路径，读到「$machine_value」"
  fi
done

# 两个开关：没写算 0；只许 0 或 1
for switch_key in "${MULTI_HOST_SWITCH_KEYS[@]}"; do
  [[ -n "${configuration[$switch_key]+set}" ]] || configuration[$switch_key]=0
  if [[ "${configuration[$switch_key]}" != 0 && "${configuration[$switch_key]}" != 1 && "$configuration_break" != switch-value-unchecked ]]; then
    refuse "$configuration_file 里 $switch_key 只许写 0 或 1（不写算 0），读到「${configuration[$switch_key]}」"
  fi
done
gpu_text="GPU：关"
[[ "${configuration[ENABLE_GPU]}" == 1 ]] && gpu_text="GPU：开"
if [[ "${configuration[ENABLE_ACROSS_MACHINES]}" != 1 && "$configuration_break" != switch-off-is-on ]]; then
  # 双机关着：配置判得过，不连第二台；调用方照单机跑
  echo "双机：关（配置 $configuration_file 里 ENABLE_ACROSS_MACHINES 没写或不是 1；要开就在里面写一行 ENABLE_ACROSS_MACHINES=1），${gpu_text}"
  exit 3
fi

peer_description="第二台 ${configuration[PEER_SSH_HOST]}"
if (( emit_hosts )); then
  # 不连任何一台：连不连得上由调用方逐台报（文件头 --emit-hosts）
  if [[ "$configuration_break" == hosts-reach-refuses ]]; then
    for machine_alias in "${machine_aliases[@]}"; do
      ssh -o BatchMode=yes -o ConnectTimeout=10 "$machine_alias" true </dev/null >/dev/null 2>&1 || refuse "清单里有一台连不上"
    done
  fi
elif [[ "${SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE:-0}" == 1 ]]; then
  peer_description="第二台是本机（只供测试的开关 SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1）"
else
  ssh -o BatchMode=yes -o ConnectTimeout=10 "${configuration[PEER_SSH_HOST]}" true </dev/null >/dev/null 2>&1 \
    || refuse "ssh -o BatchMode=yes ${configuration[PEER_SSH_HOST]} true 连不上"
  ssh -o BatchMode=yes -o ConnectTimeout=10 "${configuration[PEER_SSH_HOST]}" "test -x $(printf '%q' "${configuration[PEER_CARGO_BIN_DIRECTORY]}")/cargo" </dev/null >/dev/null 2>&1 \
    || refuse "第二台上没有 ${configuration[PEER_CARGO_BIN_DIRECTORY]}/cargo（PEER_CARGO_BIN_DIRECTORY 要指到它的 ~/.cargo/bin）"
fi

if (( emit_assignments || emit_hosts )); then
  for expected_key in "${CRASH_POINT_REPLAY_SHARD_CONFIGURATION_KEYS[@]}" "${MULTI_HOST_SWITCH_KEYS[@]}"; do
    printf '%s=%q\n' "$expected_key" "${configuration[$expected_key]}"
  done
  printf 'CRASH_POINT_REPLAY_SHARD_CONFIGURATION_FILE=%q\n' "$configuration_file"
  if (( emit_hosts )); then
    printf 'MULTI_HOST_COUNT=%q\n' "${#machine_aliases[@]}"
    for machine_index in "${!machine_aliases[@]}"; do
      machine_number=$((machine_index + 2))
      machine_alias="${machine_aliases[machine_index]}"
      machine_is_peer=0
      [[ "$machine_alias" == "${configuration[PEER_SSH_HOST]}" ]] && machine_is_peer=1
      printf 'MULTI_HOST_%s_SSH_HOST=%q\n' "$machine_number" "$machine_alias"
      for machine_field in REPOSITORY_DIRECTORY CARGO_BIN_DIRECTORY MEMORY_CAP; do
        machine_value="${configuration[HOST_${machine_number}_${machine_field}]-}"
        if [[ -z "$machine_value" ]] && (( machine_is_peer )); then machine_value="${configuration[PEER_${machine_field}]}"; fi
        printf 'MULTI_HOST_%s_%s=%q\n' "$machine_number" "$machine_field" "$machine_value"
      done
      printf 'MULTI_HOST_%s_IS_PEER=%q\n' "$machine_number" "$machine_is_peer"
    done
  fi
else
  echo "双机：开，${gpu_text}；配置 $configuration_file，$peer_description"
fi
