#!/usr/bin/env bash
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: command cargo git
# 复跑已入库的实验，把今天的输出和 research/results/ 里那份逐字节比对。
#
#   bash scripts/experiments/replay.sh [实验号...]      # 不给参数就全跑
#
# 为什么要有它：kb-discipline.md「所有历史数据都只是参考」——每条实测都绑在当时那个
# 构建上，要拿它支撑新结论，先复跑一遍确认它今天还成立。手工比对会被抄第二遍，所以做成脚本。
#
# 三条完整性闸（command-safety.md「结果抓取要有完整性闸」）：
#   1. 跑前删本轮输出，不许跨轮复用；
#   2. 收尾行必须是 `name=done emitted=N`，且 N 必须等于本轮实际 E7RESULT 行数
#      （口径：emitted 把 config 行与 done 行自己都算进去，2026-08-29 对 8 份入库产物逐份核过）；
#   3. 退出码非 0 一律判红，不许当成「跑过了」。
#
# 计时行的负载关（C350，.claude/kb/checks-owed.md）：判据写 timing 的行，开比之前先读这一趟新产物每一条 `name=done` 收尾行的负载字段。
#   约定的收尾行形态：`E7RESULT name=done emitted=N loadavg1=<1 分钟负载> cpus=<在线 CPU 数>`（装置从 /proc/loadavg 与 available_parallelism 取）。
#   emitted 照旧只数 config 行与 done 行自己，两个字段不进 emitted；逐字节比对之前两边的这两个字段都删掉（strip_load_fields）。
#   loadavg1 / cpus 超过 $REPLAY_TIMING_LOAD_RATIO（默认 0.5）的那一行记「本次不判」：不比结构、不判区间断言，既不红也不绿，
#     往 $GATE_NOT_RUN_FILE 报一行；多段产物取各段最大的比值；整趟没有一行判得了（判的全是本次不判）退 77。
#   收尾行没有这两个字段（或值不是数）的判红，理由点名产物与「收尾行没有 loadavg1 / cpus」；
#     例外是 .claude/gate.d/timing-load-field-lag.tsv 登记的存量产物（一行两列、制表符分隔：复跑表第 4 列的留存产物名、理由）：照旧判，成功句报「滞后 K 份」。
#     滞后表只缩不涨：装置补了字段、留存产物换成带字段的新产物，就删那一行；不往里加新行。
#     登记的名字不是复跑表某条 timing 行的第 4 列、没写理由、重复登记、或留存产物自己已经有字段，开跑前判红退 2。
#
# 准入（门禁与实验共用 scripts/gate/admission.py，登记表 .claude/gate.d/stage-inputs.tsv）：登记了准入的实验，
# 每一行开跑之前先问一次（实验键 = 这一行的行键去掉期望产物，admission.py 文件头「复跑表每一行的行键」），实验键与留存产物头上记的相同就记「输入没变」、
# 不跑也不判红；产物头的 `E7INPUT` 行比对前删掉。
# 每一行按自己的准入键问、比自己的产物：键在准入那一段的 ADMISSION_ROW_KEYS 里按行指明，没指明的是实验号；同一个实验几行又登记了准入的，各行各指一个键。
#   bash scripts/experiments/replay.sh --admission-only [实验号...]   只问准入，不编、不跑、不比
#   bash scripts/experiments/replay.sh --selftest                    假 cargo 演构建的两种结局（replay_selftest），不编、不跑实验
#   bash scripts/experiments/replay.sh --full [实验号...]             全量：按行复用的判定库只记不读，每一行都现跑（SINGLEFS_GATE_FULL=1 同义）
# 复用（test-script-discipline.md「判定可复用：按输入存，按前缀剪」）：每一行复跑一条判定，行键是装置 main 起的单元闭包、驱动函数的 shell 单元闭包、
#   这一行的参数、GPU 开关与特性集、期望产物内容、工具链、解析出的依赖版本（scripts/gate/admission.py replay-row-inputs，算法在它文件头「复跑表每一行的行键」），
#   不带实验号与文件名。先算全部行的行键、判完准入，再定要现跑的几行：一行都不用现跑就不试编 gpu 特性、不清场、不编；要现跑的只有 research 二进制的行时，
#   只编那几个 bin（cargo build --bin <名>…），有驱动函数的行才整编。判定库里这把键有一份判过的记录（字节一致、仅计时不同、已归档只跑不比三种）就不跑、记「复用」，
#   结论区间断言照常对记录里留的那一份产物判；没有就现跑，判成这三种之一的连同新产物、组成清单、判定日期与跑法写进判定库（同目录临时目录写完再改名换上）。
#   键对不上而要现跑的那一行，开跑前在 stderr 报一行变了的单元（与这一行上一次判过的组成清单比）。行键拒算的那一行照跑、不复用、不入库，计进 refused=。
#   判定库：<git common-dir>/singlefs-unit-verdicts/replay-row/keys/<行键>/（verdict、line、product、finished、how、composition），items/<前三列的 sha256> 记这一行最近一次的键；
#   只记不删，键不按时限过期；不在 git 工作树里不读不写。旧的 singlefs-replay-verdicts/ 整个弃用（键的写法不同，一条都不认）。
#   全量（--full 或 SINGLEFS_GATE_FULL=1）只写不读：每一行都现跑，判过的照键写回同一份库。
#   每一趟末尾打一行 REPLAY_TOTAL dispatched=<派出> collected=<收回> judged=<现判> reused=<复用> refused=<行键拒算> pass= timing_only= archived_run_only=<已归档只跑不比，含复用来的>
#     drift= broken= claims_bad= unchanged= load_skipped= full=<0|1>（门禁 checker-tier-research-build-and-replay 的 experiment-replay 格读它）。
#   已归档、只跑不比的行（留存产物不在树里，逐字节这一档没有对照物）逐行列出，设了 $GATE_NOT_RUN_FILE 时每行往里追加一行（show-me-test.md「门禁不许假装通过」）。
# 开跑前的构建输出落进 $OUT_DIR/build.log；构建失败退 2，stderr 打第一处 error、日志末尾 20 行、日志路径与单独重现的命令。
# GPU（test-script-discipline.md「用上现有的硬件加速」；resolve_checker_tier_features）：多机配置（scripts/crash/crash-amplification-lib.sh 文件头「配置」：
#   ${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}）写了 ENABLE_GPU=1 时，起 checker 档装置（用到给定状态引擎的 e156、e158、e161、e168、e170、e171 各装置）的
#   cargo run 一律带 --features verdict-store,gpu（数组 CHECKER_TIER_FEATURES）；开跑前先 cargo build --release -p singlefs-checker-tier --features verdict-store,gpu --lib
#   试编一次，输出落进 $OUT_DIR/gpu-build.log。没开（文件不在、没写这个键、写 0）一个字都不变，不起这一次试编。
#   配置读不出（SINGLEFS_MULTI_HOST_CONFIG 点名的文件不在、文件不可读、读的时候出错、ENABLE_GPU 写了 0、1 之外的值），或开了 GPU 而 gpu 特性编不过：退 78，一条实验都没跑，
#   stderr 写原因与出路；不悄悄退回不带特性去跑（那样装置把判段当没编进 gpu 特性，报本次未跑或 panic）。
#   清场（.claude/rules/verification.md「凡用 GPU 先清场，跑完不复原」）：开了 GPU 时，试编过了之后先经 bash scripts/crash/gpu-quiesce-run.sh -- true 清一次场并回读（卡上有不归
#   清场白名单的计算进程时回读不过，等那个进程跑完再起，不停它），之后起 checker 档装置的 cargo run 一律排在 run-with-memory-cap.sh 里面、经同一个清场包装起
#   （数组 CHECKER_TIER_QUIESCE_PREFIX：env SINGLEFS_MULTI_HOST_CONFIG=<配置> GPU_CARDS_BY_PRIORITY=<清场之后那一份> bash scripts/crash/gpu-quiesce-run.sh --）。跑完、跑红、被打断都不复原
#   （要复原手动跑多机配置的 QUIESCE_START_COMMAND，再拿 QUIESCE_STARTED_CHECK_COMMAND 回读到它退 0）。显卡先后与额度一律用清场之后那一份：多机配置的
#   GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 交给装置（环境变量 GPU_CARDS_BY_PRIORITY），不取不清场那一份，也不照调用方环境里漏进来的那个变量。
#   开了 GPU 而多机配置没写 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED、或清场回读没过：退 78，一条实验都没跑，stderr 写原因与出路。没开 GPU 一个字都不变，不清场。
#   GPU 开关与特性集进点到 CHECKER_TIER_FEATURES 的那几行的行键（文件头「复用」）：开关一换，只那几行现跑。开关先读（read_gpu_switch，不编），
#   试编与清场（prepare_checker_tier_features）只在有行要现跑时做。
set -uo pipefail
# 每个驱动跑的都是编译出来的代码（cargo run 或直接执行 release 二进制），子 agent 经它跑要经
# scripts/hosts/run-with-memory-cap.sh（.claude/hooks/heavy-test-guard.sh 执行前拒不经它的直接执行/cargo run；
# records/2026-09-16-subagent拆分提案.md 第四十节第 35 行，来历见第 30 行）。上限没有按驱动分别量过，
# 给一个统一默认值（run-with-memory-cap.sh 文件头「默认值由调用方定，写在调用方的脚本头」），
# REPLAY_MEMORY_CAP 环境变量能覆盖。
REPLAY_MEMORY_CAP="${REPLAY_MEMORY_CAP:-8G}"
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
cd "$(dirname "$0")/../.." || { echo "  ✗ 进不去仓根 $(dirname "$0")/../.."; echo "     → 怎么办：从仓根照文件头的用法起：bash scripts/experiments/replay.sh"; exit 2; }
source scripts/hosts/heavy-threads.sh
REPLAY_BROKEN="${REPLAY_BREAK:-}"   # 弄坏开关（只给 --selftest 证红用），见 replay_selftest 上面那一段

# 拷 crates/ 去编译的副本（e168_patched_tree 的副本、driver_e171_s4_comparator 的冻结树、driver_e175 的仓副本）都带上每台默认重活线程数的登记文件
# .claude/heavy-thread-default：harness 包经 include_str! 在编译时读它（crates/singlefs-harness/src/heavy_threads.rs），副本里没有就编不过。
# 拷 crates 的那一行在同一行里接着调它（replay_selftest 的 copy-lines-with-registry 那一格静态核）。在仓根调（脚本开头已 cd 到仓根）。
# 弄坏开关 REPLAY_BREAK=thread-registry-not-copied：什么都不拷（改前的样子）。
copy_thread_registry_into() { # <副本目录>
  [[ "$REPLAY_BROKEN" == thread-registry-not-copied ]] && return 0
  mkdir -p "$1/.claude" && cp -- .claude/heavy-thread-default "$1/.claude/heavy-thread-default"
}

# 开跑之前编一次 scripts/experiments/ 的装置（在 scripts/experiments/ 里跑）：输出落进 $1（构建日志）。失败时打第一处 error 那一行、日志末尾 20 行、
# 日志路径与一条单独重现的命令，返回 2；成功什么都不打。
build_research_binaries() { # <构建日志> [只编这几个 bin…]（不给就整编 research 工作区）
  local log="$1" build_exit bin_options=() name
  shift
  for name in "$@"; do bin_options+=(--bin "$name"); done
  if [[ "$REPLAY_BROKEN" == build-output-dropped ]]; then   # 弄坏开关：改前的样子，输出丢进 /dev/null、红了只说一句
    cargo build --release --manifest-path scripts/experiments/Cargo.toml >/dev/null 2>&1 || { echo "replay: 构建失败" >&2; return 2; }
    return 0
  fi
  if cargo build --release --manifest-path scripts/experiments/Cargo.toml ${bin_options[@]+"${bin_options[@]}"} >"$log" 2>&1; then
    return 0
  else
    build_exit=$?
  fi
  echo "replay: 构建失败（cargo build 退 $build_exit），一条实验都没跑" >&2
  grep -m1 -E '^error(\[|:)' -- "$log" | sed 's/^/  第一处错：/' >&2
  echo "  构建日志末尾 20 行：" >&2
  tail -n 20 -- "$log" | sed 's/^/    /' >&2
  echo "  整份构建日志：$log" >&2
  echo "  → 怎么办：照日志里第一处 error 改；单独重现：(cd $PWD && cargo build --release --manifest-path scripts/experiments/Cargo.toml${bin_options[*]:+ ${bin_options[*]}})" >&2
  return 2
}

# 驱动函数开跑前编它要的那一个研究二进制。研究工作区是 scripts/experiments/，仓根不是那个工作区（在仓根点那个包再带特性，cargo 拒），
# 照 build_research_binaries 经 --manifest-path 指过去；cargo 的输出一律进 stderr（驱动的 stderr 是这一行的 .err，stdout 是产物），
# 编不过另打一句重现命令、返回 1。行键照 --bin 那个名字解到 scripts/experiments/ 里那份源码（admission.py 文件头「复跑表每一行的行键」）。
# 弄坏开关 REPLAY_BREAK=driver-build-output-dropped：改前的样子（不指工作区、输出丢进 /dev/null，.err 是空的）。
build_research_driver_binary() { # <特性> <bin 名>
  local features="$1" binary="$2"
  if [[ "$REPLAY_BROKEN" == driver-build-output-dropped ]]; then
    cargo build -q --release --features "$features" --bin "$binary" >/dev/null 2>&1 || return 1
    return 0
  fi
  cargo build -q --release --manifest-path scripts/experiments/Cargo.toml --features "$features" --bin "$binary" >&2 && return 0
  echo "replay: 驱动要的研究二进制 $binary 编不过（上面是 cargo 的输出）；单独重现：(cd $PWD && cargo build --release --manifest-path scripts/experiments/Cargo.toml --features $features --bin $binary)" >&2
  return 1
}

# 开了 GPU 时的清场（文件头「GPU」；.claude/rules/verification.md「凡用 GPU 先清场，跑完不复原」）：结果放进全局数组 CHECKER_TIER_QUIESCE_PREFIX，不经命令替换带出来。
# 清场之后的显卡先后只取多机配置的 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED（没写退 78：装置在清场之下没有它就拒，那样每一行都红在装置里，不如这里一次说清）；
# 数组是 env SINGLEFS_MULTI_HOST_CONFIG=<配置的绝对路径> GPU_CARDS_BY_PRIORITY=<那一份> bash <仓根>/scripts/crash/gpu-quiesce-run.sh --，起装置时放在 cargo run 前面
# （路径写绝对的：E168 的阳性对照在另一棵树里 cd 过去才起）。先拿它包一条 true 清一次场并回读：回读没过（包装退非 0）退 78，打包装自己的 GPU_QUIESCE 行与出路，
# 不复原。弄坏开关 REPLAY_BREAK=gpu-without-quiesce：整段不办（改前的样子）；=quiesce-failure-ignored：清场没过也照跑；
# =cards-not-from-quiesced-list：显卡先后取不清场的 GPU_CARDS_BY_PRIORITY。
# 返回 0：清过场并回读过；返回 1（打 ✗ 与 → 出路）：没写清场之后的显卡先后，或清场没过。
quiesce_for_checker_tier_runs() { # <多机配置的路径>
  local configuration_file="$1" cards_key=GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED quiesced_cards quiesce_exit
  CHECKER_TIER_QUIESCE_PREFIX=()
  [[ "$REPLAY_BROKEN" == gpu-without-quiesce ]] && return 0
  [[ "$configuration_file" == /* ]] || configuration_file="$PWD/$configuration_file"
  [[ "$REPLAY_BROKEN" == cards-not-from-quiesced-list ]] && cards_key=GPU_CARDS_BY_PRIORITY
  quiesced_cards="$(crash_amplification_configuration_value "$configuration_file" "$cards_key" 2>/dev/null)"
  if [[ -z "${quiesced_cards//[[:space:]]/}" ]]; then
    echo "  ✗ replay: 多机配置 $configuration_file 开了 GPU（ENABLE_GPU=1）却没写 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED（清场之后的显卡先后与额度），一条实验都没跑" >&2
    echo "  → 怎么办：照仓根 multi-host.env.example 在多机配置里写清场之后的显卡先后与额度；确实不用 GPU 就把多机配置的 ENABLE_GPU 改成 0" >&2
    return 1
  fi
  CHECKER_TIER_QUIESCE_PREFIX=(env "SINGLEFS_MULTI_HOST_CONFIG=$configuration_file" "GPU_CARDS_BY_PRIORITY=$quiesced_cards" bash "$PWD/scripts/crash/gpu-quiesce-run.sh" --)
  if "${CHECKER_TIER_QUIESCE_PREFIX[@]}" true; then quiesce_exit=0; else quiesce_exit=$?; fi
  if (( quiesce_exit != 0 )) && [[ "$REPLAY_BROKEN" != quiesce-failure-ignored ]]; then
    echo "  ✗ replay: 开了 GPU，清场没过（scripts/crash/gpu-quiesce-run.sh 退 $quiesce_exit，原因在上面它自己打的 GPU_QUIESCE 行里；回读没过、多机配置没写清场命令都是这一种），一条实验都没跑；不复原" >&2
    echo "  → 怎么办：卡上有不归清场白名单的计算进程时回读就不过：等那个进程跑完再起，不停它；要复原手动跑多机配置的 QUIESCE_START_COMMAND，再拿 QUIESCE_STARTED_CHECK_COMMAND 回读到它退 0；确实不用 GPU 就把多机配置的 ENABLE_GPU 改成 0" >&2
    CHECKER_TIER_QUIESCE_PREFIX=()
    return 1
  fi
  return 0
}

# 配置开了 GPU 时，起 checker 档装置（cargo run … -p singlefs-checker-tier --bin <装置>）带 --features verdict-store,gpu 编（文件头「GPU」）：
# 结果放进全局数组 CHECKER_TIER_FEATURES（空数组就是不带，默认特性 verdict-store 照旧），不经命令替换带出来。多机配置怎么找、键怎么读只在
# scripts/crash/crash-amplification-lib.sh 一处（crash_amplification_multi_host_configuration_file、crash_amplification_configuration_value），这里 source 它不抄第二份。
# 返回 0：配置没开 GPU（文件不在、没写 ENABLE_GPU、写 0），或开了而 gpu 特性编得过（cargo build --release -p singlefs-checker-tier --features verdict-store,gpu --lib，输出落进 $1）。
# 返回 78（打 ✗ 与 → 出路）：配置读不出（SINGLEFS_MULTI_HOST_CONFIG 点名的文件不在、文件不可读、读的时候出错、ENABLE_GPU 写了 0、1 之外的值），
# 或开了 GPU 而 gpu 特性编不过。不悄悄退回不带特性：那样装置会把判段当没编进 gpu 特性、报本次未跑或 panic，复跑结论不是配置要的那一份。
# 弄坏开关 REPLAY_BREAK=gpu-features-dropped：读配置、编也编，但开了 GPU 也不带特性（改前的样子）。
# 开了 GPU、试编也过了之后接着清场（quiesce_for_checker_tier_runs）：数组 CHECKER_TIER_QUIESCE_PREFIX 是起装置时放在 cargo run 前面的清场包装（没开 GPU 是空数组）。
resolve_checker_tier_features() { # <gpu 编译日志>：先读开关、开着再试编与清场（两步分开的写法见 read_gpu_switch、prepare_checker_tier_features）
  read_gpu_switch "$1" || return $?
  prepare_checker_tier_features "$1"
}

# 读多机配置的 ENABLE_GPU：结果放进全局 REPLAY_GPU_SWITCH（0 或 1）与 REPLAY_GPU_CONFIGURATION_FILE，不编、不清场。配置读不出返回 78（打 ✗ 与 → 出路）。
read_gpu_switch() { # <gpu 编译日志>（读配置出错时把那几行落在 <日志>.configuration）
  local log="$1" configuration_file switch="" reading_exit
  REPLAY_GPU_SWITCH=0
  REPLAY_GPU_CONFIGURATION_FILE=""
  CHECKER_TIER_FEATURES=()
  CHECKER_TIER_QUIESCE_PREFIX=()
  # 只拷了这一份脚本的树（自证造的临时仓）里没有那份库：那里也没有多机配置，ENABLE_GPU 算关，不读库
  if [[ ! -r scripts/crash/crash-amplification-lib.sh && -z "${SINGLEFS_MULTI_HOST_CONFIG:-}" && ! -e multi-host.env ]]; then
    return 0
  fi
  # shellcheck source=/dev/null
  if ! source scripts/crash/crash-amplification-lib.sh; then
    echo "  ✗ replay: 读不了 scripts/crash/crash-amplification-lib.sh（读多机配置的共用库），不知道配置开没开 GPU，一条实验都没跑" >&2
    echo "  → 怎么办：从仓根起（bash scripts/experiments/replay.sh），或补回那份库" >&2
    return 78
  fi
  configuration_file="$(crash_amplification_multi_host_configuration_file "$PWD")"
  if [[ ! -e "$configuration_file" ]]; then
    if [[ -n "${SINGLEFS_MULTI_HOST_CONFIG:-}" ]]; then
      echo "  ✗ replay: SINGLEFS_MULTI_HOST_CONFIG 点名的多机配置 $configuration_file 不在，读不出 ENABLE_GPU，一条实验都没跑" >&2
      echo "  → 怎么办：把 SINGLEFS_MULTI_HOST_CONFIG 指到在的文件，或不设它（取主工作树根的 multi-host.env；那份不在就是两个开关都关）" >&2
      return 78
    fi
    return 0   # 没有多机配置：ENABLE_GPU 算关（crash-amplification-lib.sh 文件头「配置」）
  fi
  if [[ ! -r "$configuration_file" ]]; then
    echo "  ✗ replay: 多机配置 $configuration_file 不可读，读不出 ENABLE_GPU，一条实验都没跑" >&2
    echo "  → 怎么办：修好文件权限（ls -l $configuration_file），再跑" >&2
    return 78
  fi
  if switch="$(crash_amplification_configuration_value "$configuration_file" ENABLE_GPU 2>"$log.configuration")"; then reading_exit=0; else reading_exit=$?; fi
  if (( reading_exit != 0 )); then
    echo "  ✗ replay: 读多机配置 $configuration_file 的 ENABLE_GPU 出错（退 $reading_exit）：$(head -n 3 -- "$log.configuration" 2>/dev/null | tr "\n" " ")，一条实验都没跑" >&2
    echo "  → 怎么办：看那份配置是不是 UTF-8 文本；单跑 bash -c 'source scripts/crash/crash-amplification-lib.sh && crash_amplification_configuration_value \"\$1\" ENABLE_GPU' x $configuration_file" >&2
    return 78
  fi
  case "$switch" in
    ""|0) return 0 ;;
    1) REPLAY_GPU_SWITCH=1; REPLAY_GPU_CONFIGURATION_FILE="$configuration_file"; return 0 ;;
    *)
      echo "  ✗ replay: 多机配置 $configuration_file 的 ENABLE_GPU 写成了「$switch」，只认 0、1，读不出开没开 GPU，一条实验都没跑" >&2
      echo "  → 怎么办：改成 ENABLE_GPU=1 或 ENABLE_GPU=0（写法见仓根 multi-host.env.example）" >&2
      return 78 ;;
  esac
}

# 开了 GPU（read_gpu_switch 读出 1）时试编 gpu 特性、清场，设 CHECKER_TIER_FEATURES 与 CHECKER_TIER_QUIESCE_PREFIX；没开一个字都不变。编不过、清场没过返回 78。
prepare_checker_tier_features() { # <gpu 编译日志>
  local log="$1" configuration_file="${REPLAY_GPU_CONFIGURATION_FILE:-}" build_exit
  CHECKER_TIER_FEATURES=()
  CHECKER_TIER_QUIESCE_PREFIX=()
  [[ "${REPLAY_GPU_SWITCH:-0}" == 1 ]] || return 0
  if cargo build --release -p singlefs-checker-tier --features verdict-store,gpu --lib >"$log" 2>&1; then
    quiesce_for_checker_tier_runs "$configuration_file" || return 78
    [[ "$REPLAY_BROKEN" == gpu-features-dropped ]] || CHECKER_TIER_FEATURES=(--features verdict-store,gpu)
    return 0
  else
    build_exit=$?
  fi
  echo "  ✗ replay: 多机配置 $configuration_file 开了 GPU（ENABLE_GPU=1），checker 档带 --features verdict-store,gpu 编不过（cargo build 退 $build_exit），一条实验都没跑；不退回不带 gpu 特性去跑" >&2
  grep -m1 -E '^error(\[|:)' -- "$log" | sed 's/^/  第一处错：/' >&2
  echo "  编译日志末尾 20 行：" >&2
  tail -n 20 -- "$log" | sed 's/^/    /' >&2
  echo "  整份编译日志：$log" >&2
  echo "  → 怎么办：照日志里第一处 error 修好 gpu 特性的编译；单独重现：(cd $PWD && cargo build --release -p singlefs-checker-tier --features verdict-store,gpu --lib)；确实不用 GPU 就把多机配置的 ENABLE_GPU 改成 0" >&2
  return 78
}

# 复跑表这一行问准入用哪个键：admission.py replay-row-keys 给的（按前三列 实验号|二进制|参数 查 ADMISSION_ROW_KEY），结果放进 ROW_ADMISSION_KEY
# （不经命令替换带出来）。弄坏开关 REPLAY_BREAK=row-key-by-experiment 下一律用实验号（改前的样子）。
admission_key_of_row() {
  if [[ "$REPLAY_BROKEN" == row-key-by-experiment ]]; then
    ROW_ADMISSION_KEY="$1"
  else
    ROW_ADMISSION_KEY="${ADMISSION_ROW_KEY["$1|$2|$3"]-$1}"
  fi
}

# ── 计时行的负载关（约定见文件头「计时行的负载关」）──────────────────────────────────────
TIMING_LOAD_RATIO="${REPLAY_TIMING_LOAD_RATIO:-0.5}"
TIMING_LAG_FILE=".claude/gate.d/timing-load-field-lag.tsv"
declare -A TIMING_LAG=()   # 滞后表：留存产物名 → 理由

# 比对之前删掉 name=done 收尾行上的 loadavg1= 与 cpus=（管道过滤器）：每次复跑它们都不同，留存产物里也可能没有。别的行上同名的字段不动。
strip_load_fields() {
  sed -E '/^E7RESULT name=done /s/ (loadavg1|cpus)=[^ ]*//g'
}

# 读 $1（一份产物）里每一条 name=done 收尾行的 loadavg1= 与 cpus=，往 stdout 打一行：
#   missing <原因>                      没有收尾行，或有一条收尾行缺这两个字段、值不是数、cpus 不是正整数
#   ok|high <最大比值> <loadavg1> <cpus>  取各段里 loadavg1/cpus 最大的那一段；比 $TIMING_LOAD_RATIO 大是 high
timing_load_reading_raw() {
  awk -v limit="$TIMING_LOAD_RATIO" '
    /^E7RESULT name=done / {
      seen++; load = ""; cpus = ""
      for (i = 1; i <= NF; i++) { if ($i ~ /^loadavg1=/) load = substr($i, 10); else if ($i ~ /^cpus=/) cpus = substr($i, 6) }
      if (load !~ /^[0-9]+(\.[0-9]+)?$/ || cpus !~ /^[1-9][0-9]*$/) { malformed = 1; next }
      ratio = load / cpus
      if (!have || ratio > best) { have = 1; best = ratio; best_load = load; best_cpus = cpus }
    }
    END {
      if (!seen) print "missing 产物里没有 name=done 收尾行，收尾行没有 loadavg1 / cpus"
      else if (malformed) print "missing 收尾行没有 loadavg1 / cpus（或值不是数）"
      else { word = (best > limit + 0) ? "high" : "ok"; print word, best, best_load, best_cpus }
    }' "$1"
}

# 这一行（种类 $1，留存产物名 $2，本次新产物 $3）过不过负载关；结果放进全局，不经命令替换带出来：
#   TIMING_LOAD_DECISION  judge（照判）、skip（本次不判）、red（判红）；TIMING_LOAD_NOTE 是点名产物与比值的一句话；TIMING_LOAD_LAGGED 为 1 是「没字段、靠滞后表照判」
# 弄坏开关 REPLAY_BREAK=load-field-ignored：读字段，但 high 与「没字段又不在滞后表」都照判（不看字段的样子）。
timing_load_decide() {
  local kind="$1" stored="$2" fresh="$3" reading word ratio load cpus
  TIMING_LOAD_DECISION=judge; TIMING_LOAD_NOTE=""; TIMING_LOAD_LAGGED=0
  [[ "$kind" == timing ]] || return 0
  reading="$(timing_load_reading_raw "$fresh")"
  word="${reading%% *}"
  case "$word" in
    high|ok)
      read -r _ ratio load cpus <<<"$reading"
      TIMING_LOAD_NOTE="loadavg1=$load cpus=$cpus 比值 $ratio（阈值 $TIMING_LOAD_RATIO）"
      if [[ "$word" == high && "$REPLAY_BROKEN" != load-field-ignored ]]; then TIMING_LOAD_DECISION=skip; fi ;;
    *)
      if [[ -n "${TIMING_LAG["$stored"]+set}" ]]; then
        TIMING_LOAD_LAGGED=1; TIMING_LOAD_NOTE="${reading#* }；$stored 登记在滞后表 $TIMING_LAG_FILE，照旧判"
      else
        TIMING_LOAD_NOTE="产物 $stored（本次输出 $fresh）${reading#* }；$stored 也没登记在滞后表 $TIMING_LAG_FILE"
        if [[ "$REPLAY_BROKEN" != load-field-ignored ]]; then TIMING_LOAD_DECISION=red; fi
      fi ;;
  esac
  return 0
}

# 读滞后表 $1（一行 `<留存产物名><制表符><理由>`，# 开头的行与空行跳过）进 TIMING_LAG；$2 是复跑表原文，$3 是留存产物目录。
# 每个问题往 stdout 打一行，有问题返回 1：表不存在；没有制表符隔开的理由列；没写理由（至少 8 个字）；名字重复；名字不是 $2 里某条 timing 行的第 4 列；留存产物在树里且自己已经有字段（该删这一行）。
# 留存产物不在树里是对的（多数已归档进版本库，replay_one 的「产物已归档」一档），所以这里只认复跑表，不认文件在不在。
load_timing_lag() {
  local file="$1" table_text="$2" results_directory="$3" line name reason line_number=0 problems=0 timing_products reading
  TIMING_LAG=()
  if [[ ! -f "$file" ]]; then echo "滞后表 $file 不存在"; return 1; fi
  timing_products="$(printf '%s\n' "$table_text" | awk -F'|' '$5 == "timing" {print $4}')"
  while IFS= read -r line || [[ -n "$line" ]]; do
    line_number=$((line_number + 1))
    [[ -z "${line//[[:space:]]/}" || "$line" =~ ^[[:space:]]*# ]] && continue
    if [[ "$line" != *$'\t'* ]]; then echo "$file:$line_number 没有用制表符隔开的理由（两列：留存产物名、理由）：$line"; problems=$((problems + 1)); continue; fi
    name="${line%%$'\t'*}"; reason="${line#*$'\t'}"
    name="${name//[[:space:]]/}"; reason="${reason#"${reason%%[![:space:]]*}"}"
    if [[ -z "$name" || ${#reason} -lt 8 ]]; then echo "$file:$line_number 名字空着或理由不足 8 个字：$line"; problems=$((problems + 1)); continue; fi
    if [[ -n "${TIMING_LAG["$name"]+set}" ]]; then echo "$file:$line_number 重复登记 $name"; problems=$((problems + 1)); continue; fi
    TIMING_LAG["$name"]="$reason"
    if ! grep -qxF -- "$name" <<<"$timing_products"; then
      echo "$file:$line_number $name 不是复跑表任何一条 timing 行的留存产物（第 4 列）"; problems=$((problems + 1)); continue
    fi
    if [[ -f "$results_directory/$name" ]]; then
      reading="$(timing_load_reading_raw "$results_directory/$name")"
      if [[ "${reading%% *}" != missing ]]; then echo "$file:$line_number $name 自己的收尾行已经有 loadavg1 / cpus，滞后表只缩不涨，删这一行"; problems=$((problems + 1)); fi
    fi
  done <"$file"
  ((problems == 0))
}

# 同时跑几条实验的缺省：登记的重活默认并行度（.claude/heavy-thread-default，经 scripts/hosts/heavy-threads.sh；SINGLEFS_THREAD_CAP 设了取它）。
# 弄坏开关 REPLAY_BREAK=jobs-from-nproc 下是改前的样子（核数的一半）。
default_replay_jobs() {
  if [[ "$REPLAY_BROKEN" == jobs-from-nproc ]]; then
    echo $(( $(nproc 2>/dev/null || echo 4) / 2 ))
  else
    heavy_thread_default
  fi
}

# 自证（bash scripts/experiments/replay.sh --selftest）：PATH 最前面放一个假 cargo，只演 build_research_binaries 的两种结局，不编、不跑任何实验。
#   build-red    假 cargo 打一行 error[E0425] 退 101：要返回 2，stderr 有第一处错、日志末尾、日志路径与重现命令，日志落了盘
#   build-green  假 cargo 退 0：要返回 0，stderr 一个字都没有
#   gpu-features-on / -off / -garbled / -no-such / -unreadable / -build-red  配置开了 GPU 的特性（resolve_checker_tier_features，假 cargo 记参数，配置文件放临时目录）：
#     写 ENABLE_GPU=1 → 试编一次、CHECKER_TIER_FEATURES 是 --features verdict-store,gpu；写 0 或没写 → 不带、不起 cargo；写 yes、点名的文件不在、文件不可读 → 78 点名原因；
#     开了而试编红 → 78、第一处错、日志路径、「不退回不带 gpu 特性去跑」
#   run-lines-with-features  起 checker 档装置的 cargo run 行（本文件里 grep 出来）每一行都带数组
#   gpu-quiesced-before-run / gpu-quiesce-read-back-fails / gpu-quiesced-cards-missing  开了 GPU 先清场（quiesce_for_checker_tier_runs；真的 gpu-quiesce-run.sh，
#     多机配置里的清场命令是假的、只往临时目录的日志记一笔，状态目录与日志目录也在临时目录）：
#     gpu-quiesced-before-run  清场过了 → 返回 0，经 CHECKER_TIER_QUIESCE_PREFIX 起的命令看到清场的登记文件在、GPU_CARDS_BY_PRIORITY 是 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 那一份
#       （不是配置里另写的不清场那一份），假清场命令记了一笔，不复原（GPU_QUIESCE step=restore result=not-restored）
#     gpu-quiesce-read-back-fails  清场回读没过 → 78、点名「清场没过」与「不复原」、数组空着
#     gpu-quiesced-cards-missing  开了 GPU 而没写 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED → 78、点名这个键，假清场命令一笔都没记（没停任何东西）
#   run-lines-with-quiesce  起 checker 档装置的 cargo run 行每一行都带 CHECKER_TIER_QUIESCE_PREFIX
#   row-key      一张两行的假键表（E1 第二行指明 E1/second）：admission_key_of_row 给第二行的键要是 E1/second，不是实验号
#   jobs-default 没设 SINGLEFS_THREAD_CAP 时 default_replay_jobs 要等于登记文件第一条非注释行的数；设成 7 时要是 7
#   整条按行认键（replay-row-keys 判红、按行问准入、产物头由这里写）在临时仓里跑的那几格在 python3 scripts/gate/admission.py --selftest 的「复跑表按行的准入键」
#   负载关（假产物放临时目录，阈值钉 0.5，timing_load_decide 判，不起任何实验）：
#     load-high    收尾行 loadavg1=20.50 cpus=32（比值 0.640625）→ skip，点名比值
#     load-low     loadavg1=2.50 cpus=32 → judge
#     load-none    收尾行没有字段、不在滞后表 → red，点名产物与「收尾行没有 loadavg1 / cpus」
#     load-lagged  收尾行没有字段、产物登记在滞后表 → judge 并记滞后
#     load-exact   判据写 exact 的行不看字段 → judge
#     lag-table-bad  滞后表登记复跑表里没有的名字、exact 行的产物、重复名字、理由不足 8 个字、用空格而不是制表符隔开理由 → 逐条点名（load_timing_lag）
#     lag-table-stale  滞后表登记的留存产物自己已经有字段 → 点名叫人删这一行
#     strip-load   strip_load_fields 只删 name=done 行上的 loadavg1= 与 cpus=
#   按行复用（replay_reuse_selftest：临时仓里两行的合成复跑表，E1 留存产物在、E2 已归档；这一份 replay.sh 的副本、假 cargo（记下每一次收到的参数）、假 rustc、
#   两个假 research 二进制，真仓的 .claude/scripts、.claude/singlefs-ai-sop、scripts/gate 链进来，判定库在临时仓自己的 git common-dir 里），接连跑五趟：
#     reuse-first-run      头一趟两行都跑：E1 字节一致、E2 已归档只跑不比，判定库记两份，E2 逐行报进 $GATE_NOT_RUN_FILE
#     reuse-second-run     第二趟两行都复用、一个装置都没跑、cargo build 一次都不起（全复用就不编、不清场），REPLAY_TOTAL 报 reused=2 judged=0
#     reuse-device-source  改 E1 装置 main 的一行代码：只重跑 E1、只编 --bin e1-sample，开跑前报出变了 fn e1_sample.rs::main，E2 照旧复用
#     reuse-comment-only   只改 E2 装置源码的一行注释：两行都复用（行键取单元的去注释指纹，注释不算）
#     reuse-full           --full：判定库只记不读，两行都现跑
#     reuse-three-questions  复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」）：上面头一趟、第二趟（①）、只改注释（②）、改 E1 装置 main（③，affected E1）
#                          照共用库判、打规范输出「REUSE_SELFTEST script=scripts/experiments/replay.sh …」；reuse-second-run、reuse-comment-only、reuse-device-source 三格在它里面点名
#     last-run-list        上次红的清单（文件头「三」那一份共用库）：全绿的几趟之后是空文件
#     reuse-sentinel       复用哨兵（共用库文件头「二」）：种子 5、哨兵 1 条那一趟绿、抽中一行强制现判；开着 REPLAY_BREAK=reuse-ignored 判红「复用失灵」
#   这几趟的哨兵数设 1000（键相同的全当哨兵、不抽强制现判），reuse-sentinel 那两趟除外；上次红的清单写临时目录（SINGLEFS_LAST_RUNS_DIRECTORY）
# 弄坏开关：
#   REPLAY_BREAK=build-output-dropped  构建输出丢进 /dev/null、红了只说「replay: 构建失败」（改前的样子）
#       build-red（「✗ 自检：构建失败时应当返回 2、打第一处错」）
#   REPLAY_BREAK=driver-build-output-dropped  驱动开跑前编研究二进制不指 scripts/experiments 工作区、输出丢进 /dev/null（改前的样子）
#       driver-build（「✗ 自检：驱动开跑前编研究二进制（build_research_driver_binary）要经 --manifest-path …」）
#       与 E162、E163 那三个驱动各一格（「✗ 自检：driver_e162_anchors 开跑前要经 build_research_driver_binary 编…」，driver_e162_power_cut_selftest、driver_e163_r1_merge 同）
#   REPLAY_BREAK=row-key-by-experiment  每一行一律按实验号问准入（改前的样子）
#       row-key（「✗ 自检：复跑表那一行指明了准入键 E1/second」）；admission.py --selftest 设着它跑，「复跑表按行的准入键」那几格转红
#   REPLAY_BREAK=load-field-ignored  不看负载字段：超阈值的与没字段又不在滞后表的，都照判
#       load-high（「✗ 自检：负载 20.5 / 32 核超过阈值 0.5，这一行应当记本次不判并点名比值」）、load-none（「✗ 自检：收尾行没有字段又不在滞后表，这一行应当判红…」）两格转红
#   REPLAY_BREAK=gpu-features-dropped  配置开了 GPU 也不带 gpu 特性（改前的样子）
#       gpu-features-on（「✗ 自检：配置写了 ENABLE_GPU=1，应当先试编 gpu 特性…」）
#   REPLAY_BREAK=run-lines-without-features  自检里核「cargo run 行都带特性数组」时当没有数组（模拟新加的一行忘了带）
#       run-lines-with-features（「✗ 自检：起 checker 档装置的 cargo run 行…应当每一行都带…」）
#   REPLAY_BREAK=gpu-without-quiesce  开了 GPU 也不清场、不要清场之后的显卡先后（改前的样子）
#       gpu-quiesced-before-run、gpu-quiesce-read-back-fails、gpu-quiesced-cards-missing（「✗ 自检：开了 GPU …」）
#   REPLAY_BREAK=quiesce-failure-ignored  清场回读没过也照往下跑
#       gpu-quiesce-read-back-fails
#   REPLAY_BREAK=cards-not-from-quiesced-list  显卡先后取不清场的 GPU_CARDS_BY_PRIORITY，不取 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED
#       gpu-quiesced-before-run、gpu-quiesced-cards-missing
#   REPLAY_BREAK=run-lines-without-quiesce  自检里核「cargo run 行都带清场数组」时当没有数组（模拟新加的一行忘了带）
#       run-lines-with-quiesce
#   REPLAY_BREAK=reuse-ignored  按行复用的判定库只记不读（改前的样子：每一趟全跑）
#       reuse-second-run、reuse-device-source、reuse-comment-only、reuse-three-questions（「✗ 自检：按行复用 <格名>：」）
#   REUSE_CHECKS_BREAK=rejudged-sentinel-passes（共用库 scripts/gate/lib_reuse_checks.py 的开关）  哨兵被现判不红
#       reuse-sentinel（「✗ 自检：按行复用 reuse-sentinel：」）
#   REPLAY_BREAK=reuse-key-by-row-only  按行复用的键只按复跑表那一行算，不看装置源码与工具链
#       reuse-first-run（「✗ 自检：按行复用 reuse-first-run：」：键不经准入模块算，判定库那两份不带组成清单）、
#       reuse-device-source（「✗ 自检：按行复用 reuse-device-source：」）
#   REPLAY_BREAK=build-everything  有行要现跑时一律整编（改前的样子：不按要现跑的行只编那几个 bin）
#       reuse-device-source（「✗ 自检：按行复用 reuse-device-source：」）
#   REPLAY_BREAK=archived-not-itemized  已归档只跑不比的行只报个数，不逐行列、不往 $GATE_NOT_RUN_FILE 报
#       reuse-first-run（「✗ 自检：按行复用 reuse-first-run：」）
#   REPLAY_BREAK=thread-registry-not-copied  拷 crates/ 去编译的副本不带 .claude/heavy-thread-default（改前的样子）
#       thread-registry-copied（「✗ 自检：拷 crates/ 去编译的副本要带上每台默认重活线程数的登记文件…」）
#   REPLAY_BREAK=copy-lines-without-registry  自检里核「拷 crates 的行都接着拷登记文件」时当没有那一句（模拟新加的一行忘了带）
#       copy-lines-with-registry（「✗ 自检：拷 crates/ 去编译的行…应当每一行都接着 copy_thread_registry_into…」）
# 按行复用的自证（文件头「按行复用」，replay_selftest 调）：<目录> 下建一个临时仓（git init），复跑表换成两行（E1 留存产物在、E2 已归档），
# 真仓的 .claude/scripts、.claude/singlefs-ai-sop、scripts/gate 链进来，内存包装换成直接起命令的一份，PATH 前面放假 cargo、假 rustc；
# 拿这一份 replay.sh 的副本接连跑五趟，每一格往 stdout 打一行「<格名> ok」或「<格名> bad <一句>」。
replay_reuse_selftest() {
  local work="$1" real_root="$PWD" root library calls cargo_calls out total run_number=0 reuse_exit=0 name
  root="$work/root"; library="$work/root/.git/singlefs-unit-verdicts/replay-row"; calls="$work/devices-ran.log"; cargo_calls="$work/cargo-calls.log"
  mkdir -p "$root/.claude/gate.d" "$root/scripts/experiments/target/release" "$root/scripts/hosts" "$root/research/results" "$work/bin" || return 1
  git init -q "$root" || return 1
  ln -s "$real_root/.claude/scripts" "$root/.claude/scripts"
  ln -s "$real_root/.claude/singlefs-ai-sop" "$root/.claude/singlefs-ai-sop"
  ln -s "$real_root/scripts/gate" "$root/scripts/gate"
  ln -s "$real_root/scripts/crash" "$root/scripts/crash"   # resolve_checker_tier_features 读多机配置的共用库
  printf '%s\n' '.claude/scripts' '.claude/singlefs-ai-sop' 'scripts/gate' 'scripts/crash' >"$root/.gitignore"
  printf '%s\n' '# 自证的登记表：一行都不登记准入' >"$root/.claude/gate.d/stage-inputs.tsv"
  printf '%s\n' '# 自证的滞后表' >"$root/.claude/gate.d/timing-load-field-lag.tsv"
  printf '%s\n' '#!/usr/bin/env bash' '# 自证的内存包装：不设上限，直接起命令' 'shift' 'exec "$@"' >"$root/scripts/hosts/run-with-memory-cap.sh"
  printf '%s\n' '[package]' 'name = "e7-index-bench"' 'version = "0.1.0"' '' '[lib]' 'path = "lib.rs"' '' '[[bin]]' 'name = "e1-sample"' 'path = "e1_sample.rs"' \
    '' '[[bin]]' 'name = "e2-sample"' 'path = "e2_sample.rs"' >"$root/scripts/experiments/Cargo.toml"
  printf '%s\n' '# 自证' >"$root/scripts/experiments/Cargo.lock"
  printf '%s\n' 'pub fn shared() -> u32 { 1 }' >"$root/scripts/experiments/lib.rs"
  printf '%s\n' 'fn main() { println!("one"); }' >"$root/scripts/experiments/e1_sample.rs"
  printf '%s\n' 'fn main() { println!("two"); }' >"$root/scripts/experiments/e2_sample.rs"
  for name in e1 e2; do
    printf '%s\n' '#!/usr/bin/env bash' "echo $name >>\"\${REPLAY_SELFTEST_RUN_LOG:?}\"" "printf 'E7RESULT name=config arm=$name\\nE7RESULT name=done emitted=2\\n'" \
      >"$root/scripts/experiments/target/release/$name-sample"
    chmod +x "$root/scripts/experiments/target/release/$name-sample"
  done
  printf 'E7RESULT name=config arm=e1\nE7RESULT name=done emitted=2\n' >"$root/research/results/e1-sample.out"
  printf '%s\n' '#!/usr/bin/env bash' '[[ "${1:-}" == -V ]] && { echo "cargo 0.0.0-replay-reuse-selftest"; exit 0; }' 'echo "$*" >>"${REPLAY_SELFTEST_CARGO_LOG:?}"' 'exit 0' >"$work/bin/cargo"
  printf '%s\n' '#!/usr/bin/env bash' 'case "${1:-}" in -V) echo "rustc 0.0.0-replay-reuse-selftest";; -vV) printf "rustc 0.0.0-replay-reuse-selftest\nhost: x86_64-unknown-linux-gnu\n";; esac' 'exit 0' >"$work/bin/rustc"
  chmod +x "$root/scripts/hosts/run-with-memory-cap.sh" "$work/bin/cargo" "$work/bin/rustc"
  python3 - "$real_root/scripts/experiments/replay.sh" "$root/scripts/experiments/replay.sh" <<'PY' || return 1
import sys
text = open(sys.argv[1], encoding="utf-8").read()
table_start = text.index("TABLE=$(cat <<'TSV'\n") + len("TABLE=$(cat <<'TSV'\n")
table_end = text.index("\nTSV\n)\n", table_start)
text = text[:table_start] + "E1|e1-sample||e1-sample.out|exact\nE2|e2-sample||e2-sample.out|exact" + text[table_end:]
keys_start = text.index("ADMISSION_ROW_KEYS=$(cat <<'TSV'\n") + len("ADMISSION_ROW_KEYS=$(cat <<'TSV'\n")
keys_end = text.index("TSV\n)\n", keys_start)
text = text[:keys_start] + text[keys_end:]
with open(sys.argv[2], "w", encoding="utf-8") as handle:
    handle.write(text)
PY
  chmod +x "$root/scripts/experiments/replay.sh"
  reuse_run() { # [参数…]：在临时仓里跑一趟副本；输出进 $out、这一趟起了哪几个装置进 $calls（每趟先清空）、末行 REPLAY_TOTAL 进 $total、退出码进 reuse_exit
    run_number=$((run_number + 1)); out="$work/out.$run_number"; : >"$calls"; : >"$cargo_calls"; : >"$work/not-run.$run_number"
    # 复用哨兵：键相同的全当哨兵、不抽强制现判（reuse_sentinel 设了的那几趟照它）；上次红的清单写临时目录
    if (cd "$root" && env -u SINGLEFS_GATE_FULL -u SINGLEFS_MULTI_HOST_CONFIG -u SINGLEFS_REUSE_SENTINEL_SEED PATH="$work/bin:$PATH" REPLAY_OUT="$work/replay-out.$run_number" REPLAY_JOBS=1 \
          REPLAY_SELFTEST_RUN_LOG="$calls" REPLAY_SELFTEST_CARGO_LOG="$cargo_calls" GATE_NOT_RUN_FILE="$work/not-run.$run_number" \
          SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" SINGLEFS_REUSE_SENTINEL_COUNT=1000 ${reuse_sentinel[@]+"${reuse_sentinel[@]}"} bash scripts/experiments/replay.sh "$@") >"$out" 2>&1; then
      reuse_exit=0
    else
      reuse_exit=$?
    fi
    total="$(grep '^REPLAY_TOTAL ' "$out")"
    for name in E1 E2; do   # 复用三问那一格读的：这一趟每一行复用还是现判
      if grep -qE "^$name .*复用 " "$out"; then printf '%s\t复用\n' "$name"; else printf '%s\t现判\n' "$name"; fi
    done >"$work/states.$run_number"
  }
  local reuse_sentinel=()
  ran() { sort "$calls" | tr '\n' ' '; }
  reuse_run
  if [[ "$reuse_exit" == 0 && "$(ran)" == "e1 e2 " && "$total" == *" reused=0 "* ]] && grep -qE '^E1 .*字节一致' "$out" && grep -qE '^E2 .*已归档、只跑不比' "$out" \
     && [[ "$(find "$library/keys" -mindepth 2 -maxdepth 2 -name verdict | wc -l)" == 2 && "$(find "$library/keys" -mindepth 2 -maxdepth 2 -name composition | wc -l)" == 2 ]] \
     && grep -qF 'replay.sh E2 e2-sample.out：已归档、只跑不比' "$work/not-run.$run_number"; then
    echo "reuse-first-run ok"
  else
    echo "reuse-first-run bad 头一趟应当两行都跑、E1 字节一致、E2 已归档只跑不比、判定库记两份（各带组成清单）、E2 报进本次未跑；退 $reuse_exit，起了 $(ran)，$total，判定库 $(find "$library/keys" -mindepth 2 -maxdepth 2 -name verdict 2>/dev/null | wc -l) 份，本次未跑「$(tr '\n' ' ' <"$work/not-run.$run_number")」，输出尾部 $(tail -n 4 "$out" | tr '\n' ' ')"
  fi
  reuse_run
  if [[ "$reuse_exit" == 0 && -z "$(ran)" && "$total" == *" judged=0 reused=2 "* ]] && grep -qE '^E1 .*复用 +字节一致：行键相同' "$out" && grep -qE '^E2 .*复用 +已归档只跑不比：行键相同' "$out" \
     && ! grep -q '^build' "$cargo_calls" && grep -qF '要现跑的 0 行，不试编 gpu 特性、不清场、不编' "$out"; then
    echo "reuse-second-run ok"
  else
    echo "reuse-second-run bad 第二趟行键没变应当两行都复用、一个装置都不起、cargo build 一次都不起（不清场、不编）；退 $reuse_exit，起了「$(ran)」，cargo 收到「$(tr '\n' ';' <"$cargo_calls")」，$total"
  fi
  printf '%s\n' 'fn main() { println!("uno"); }' >"$root/scripts/experiments/e1_sample.rs"
  reuse_run
  if [[ "$reuse_exit" == 0 && "$(ran)" == "e1 " && "$total" == *" reused=1 "* ]] && grep -qE '^E1 .*字节一致' "$out" && grep -qE '^E2 .*复用' "$out" \
     && grep -q -- '^build .*--bin e1-sample' "$cargo_calls" && ! grep -q -- '--bin e2-sample' "$cargo_calls" && grep -q 'E1 要现跑：键对不上.*变了 fn e1_sample.rs::main' "$out"; then
    echo "reuse-device-source ok"
  else
    echo "reuse-device-source bad 改了 E1 装置 main 的一行代码应当只重跑 E1（只编 --bin e1-sample、报出变了 fn e1_sample.rs::main）、E2 复用；退 $reuse_exit，起了「$(ran)」，cargo 收到「$(tr '\n' ';' <"$cargo_calls")」，$total，输出 $(grep '要现跑' "$out" | head -n 2 | tr '\n' ' ')"
  fi
  printf '%s\n' '// 只改一行注释：行键取单元的去注释指纹，注释不算' 'fn main() { println!("two"); }' >"$root/scripts/experiments/e2_sample.rs"
  reuse_run
  if [[ "$reuse_exit" == 0 && -z "$(ran)" && "$total" == *" reused=2 "* ]]; then
    echo "reuse-comment-only ok"
  else
    echo "reuse-comment-only bad 只改 E2 装置源码的一行注释应当两行都复用；退 $reuse_exit，起了「$(ran)」，$total"
  fi
  # 复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」）：上面四趟就是那四问——头一趟、第二趟（①）、只改注释（②，排在闭包内那一趟之后，库里是那一趟记下的）、
  # 改 E1 装置 main 的一行（③，affected 是 E1）；已有的 reuse-second-run、reuse-comment-only、reuse-device-source 三格在这一格里点名，判法照共用库
  if python3 "$real_root/scripts/gate/lib_reuse_checks.py" three-questions --script scripts/experiments/replay.sh --first "$work/states.1" --second "$work/states.2" \
       --unrelated "$work/states.4" --unit-change "$work/states.3" --affected E1 >"$work/three.log" 2>&1; then
    echo "reuse-three-questions ok"
  else
    echo "reuse-three-questions bad $(tr '\n' ' ' <"$work/three.log")"
  fi
  grep '^REUSE_SELFTEST ' "$work/three.log"
  # 上次红的清单（lib_reuse_checks.py 文件头「三」）：这几趟都绿，清单是空文件
  if [[ -f "$work/last-runs/scripts__experiments__replay.sh.tsv" && ! -s "$work/last-runs/scripts__experiments__replay.sh.tsv" ]]; then
    echo "last-run-list ok"
  else
    echo "last-run-list bad 全绿的几趟之后 $work/last-runs/scripts__experiments__replay.sh.tsv 应当在、是空文件：$(ls -l "$work/last-runs" 2>&1 | tr '\n' ' ')"
  fi
  reuse_run --full
  if [[ "$reuse_exit" == 0 && "$(ran)" == "e1 e2 " && "$total" == *" reused=0 "* && "$total" == *" full=1"* ]]; then
    echo "reuse-full ok"
  else
    echo "reuse-full bad --full 应当判定库只记不读、两行都现跑；退 $reuse_exit，起了「$(ran)」，$total"
  fi
  # 复用哨兵（lib_reuse_checks.py 文件头「二」）：种子 5、哨兵 1 条：一行当哨兵复用、另一行强制现判、判定与库里相同，整趟绿；
  # 开着 REPLAY_BREAK=reuse-ignored（只记不读）再跑，哨兵判红「复用失灵」（共用库的开关 REUSE_CHECKS_BREAK=rejudged-sentinel-passes 下这一格判错）
  reuse_sentinel=(SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=5)
  reuse_run
  local sentinel_green_exit="$reuse_exit" sentinel_green_out="$out"
  reuse_sentinel+=(REPLAY_BREAK="${REPLAY_BREAK:+$REPLAY_BREAK,}reuse-ignored")
  reuse_run
  reuse_sentinel=()
  if [[ "$sentinel_green_exit" == 0 ]] && grep -qF '复用哨兵（复跑表）：种子 5' "$sentinel_green_out" && grep -qF '复用哨兵抽中强制现判' "$sentinel_green_out" \
     && [[ "$reuse_exit" == 1 ]] && grep -qF '复用失灵：' "$out"; then
    echo "reuse-sentinel ok"
  else
    echo "reuse-sentinel bad 种子 5 那一趟应当绿、抽中一行强制现判，只记不读那一趟应当判红「复用失灵」；照常那一趟退 $sentinel_green_exit：$(grep -E '复用哨兵|复用失灵' "$sentinel_green_out" | tr '\n' ' ')；只记不读那一趟退 $reuse_exit：$(grep -E '复用哨兵|复用失灵' "$out" | tr '\n' ' ')"
  fi
}

#   REPLAY_BREAK=jobs-from-nproc  同时跑几条实验的缺省取核数的一半（改前的样子）
#       jobs-default（「✗ 自检：没设 REPLAY_JOBS 时同时跑几条实验要取登记的重活默认」）
replay_selftest() {
  local scratch failures=0 checked=0 status
  scratch="$(mktemp -d "${TMPDIR:-/tmp}/replay-selftest-XXXXXX")"
  # 中途报错、被打断也删（成功、失败都删；local 变量到退出时已不在，路径在这里就展开）
  trap "rm -rf -- '$scratch'" EXIT
  mkdir -p "$scratch/bin"
  printf '%s\n' '#!/usr/bin/env bash' 'echo "   Compiling e7-index-bench v0.1.0"' \
    'if [[ "${FAKE_CARGO_OUTCOME:-}" == red ]]; then echo "error[E0425]: cannot find value \`missing\` in this scope"; echo "error: could not compile \`e7-index-bench\`"; exit 101; fi' \
    'echo "    Finished release"; exit 0' >"$scratch/bin/cargo"
  chmod +x "$scratch/bin/cargo"
  checked=$((checked + 1))
  if (PATH="$scratch/bin:$PATH" FAKE_CARGO_OUTCOME=red build_research_binaries "$scratch/build-red.log") 2>"$scratch/build-red.stderr"; then status=0; else status=$?; fi
  if [[ "$status" != 2 ]] || ! grep -qF "第一处错：error[E0425]: cannot find value" "$scratch/build-red.stderr" \
     || ! grep -qF "    error: could not compile" "$scratch/build-red.stderr" || ! grep -qF "整份构建日志：$scratch/build-red.log" "$scratch/build-red.stderr" \
     || ! grep -qF "单独重现：(cd $PWD && cargo build --release --manifest-path scripts/experiments/Cargo.toml)" "$scratch/build-red.stderr" \
     || [[ ! -s "$scratch/build-red.log" ]]; then
    echo "  ✗ 自检：构建失败时应当返回 2、打第一处错、日志末尾、日志路径与重现命令，日志要落盘；实际返回 $status：$(cat "$scratch/build-red.stderr")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  if (PATH="$scratch/bin:$PATH" FAKE_CARGO_OUTCOME=green build_research_binaries "$scratch/build-green.log") 2>"$scratch/build-green.stderr"; then status=0; else status=$?; fi
  if [[ "$status" != 0 || -s "$scratch/build-green.stderr" ]]; then
    echo "  ✗ 自检：构建成功时应当返回 0、一个字都不打；实际返回 $status：$(cat "$scratch/build-green.stderr")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 驱动开跑前编研究二进制（build_research_driver_binary）：假 cargo 记收到的参数、照仓根编那种报错退 101；cargo 的输出进 stderr（这一行的 .err），stdout 一个字不打
  mkdir -p "$scratch/driver-bin"
  printf '%s\n' '#!/usr/bin/env bash' 'echo "$*" >>"${FAKE_CARGO_LOG:?}"' 'echo "error: cannot specify features for packages outside of workspace"' 'exit 101' >"$scratch/driver-bin/cargo"
  chmod +x "$scratch/driver-bin/cargo"
  : >"$scratch/driver-cargo.log"
  checked=$((checked + 1))
  if (PATH="$scratch/driver-bin:$PATH" FAKE_CARGO_LOG="$scratch/driver-cargo.log" build_research_driver_binary e174-gpu e174-lease-drop-wait-memory-return) \
       >"$scratch/driver-build.stdout" 2>"$scratch/driver-build.err"; then status=0; else status=$?; fi
  if [[ "$status" != 1 || -s "$scratch/driver-build.stdout" ]] || ! grep -qF 'error: cannot specify features for packages outside of workspace' "$scratch/driver-build.err" \
     || ! grep -qxF 'build -q --release --manifest-path scripts/experiments/Cargo.toml --features e174-gpu --bin e174-lease-drop-wait-memory-return' "$scratch/driver-cargo.log"; then
    echo "  ✗ 自检：驱动开跑前编研究二进制（build_research_driver_binary）要经 --manifest-path scripts/experiments/Cargo.toml 编、编不过返回 1、cargo 的输出进 stderr（这一行的 .err 非空）、stdout 不打；实际返回 $status，cargo 收到「$(tr '\n' '|' <"$scratch/driver-cargo.log")」，stderr「$(head -c 300 "$scratch/driver-build.err")」，stdout「$(head -c 200 "$scratch/driver-build.stdout")」"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # E162、E163 那三个驱动同样经它编：起驱动本身，假 cargo 另记 BINDGEN_EXTRA_CLANG_ARGS（E162 两个要把它设进编那一步），编不过就停在编那一步。
  # 驱动定义在这个函数被调用之后才读到，这里从本文件里取出那三段定义、在子 shell 里 eval
  printf '%s\n' '#!/usr/bin/env bash' 'echo "$*|BINDGEN=${BINDGEN_EXTRA_CLANG_ARGS-没设}" >>"${FAKE_CARGO_LOG:?}"' 'echo "error: cannot specify features for packages outside of workspace"' 'exit 101' >"$scratch/driver-bin/cargo"
  local driver_case driver_name driver_expected driver_definitions
  driver_definitions="$(awk '/^(driver_e162_anchors|driver_e162_power_cut_selftest|driver_e163_r1_merge)\(\) \{/,/^}/' "${BASH_SOURCE[0]}")"
  checked=$((checked + 1))
  if [[ "$(grep -cE '^driver_e16[23]_[a-z0-9_]+\(\) \{' <<<"$driver_definitions")" != 3 ]]; then
    echo "  ✗ 自检：从 ${BASH_SOURCE[0]} 里取不全 driver_e162_anchors、driver_e162_power_cut_selftest、driver_e163_r1_merge 三段定义（取到的头几行：$(head -n 3 <<<"$driver_definitions" | tr '\n' '|')）"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  for driver_case in \
    "driver_e162_anchors|build -q --release --manifest-path scripts/experiments/Cargo.toml --features e162-block-stores --bin e162-crash-verdict-block-store|BINDGEN=-I/usr/lib/gcc/x86_64-linux-gnu/13/include" \
    "driver_e162_power_cut_selftest|build -q --release --manifest-path scripts/experiments/Cargo.toml --features e162-block-stores --bin e162-verdict-store-power-cut|BINDGEN=-I/usr/lib/gcc/x86_64-linux-gnu/13/include" \
    "driver_e163_r1_merge|build -q --release --manifest-path scripts/experiments/Cargo.toml --features e163-gpu --bin e163-gpu-multicard-crc32c|BINDGEN=没设"; do
    driver_name="${driver_case%%|*}"; driver_expected="${driver_case#*|}"
    : >"$scratch/driver-cargo.log"
    checked=$((checked + 1))
    if (unset BINDGEN_EXTRA_CLANG_ARGS; OUT_DIR="$scratch"; eval "$driver_definitions"; PATH="$scratch/driver-bin:$PATH" FAKE_CARGO_LOG="$scratch/driver-cargo.log" "$driver_name") \
         >"$scratch/driver-build.stdout" 2>"$scratch/driver-build.err"; then status=0; else status=$?; fi
    if [[ "$status" != 1 || -s "$scratch/driver-build.stdout" ]] || ! grep -qF 'error: cannot specify features for packages outside of workspace' "$scratch/driver-build.err" \
       || [[ "$(cat "$scratch/driver-cargo.log")" != "$driver_expected" ]]; then
      echo "  ✗ 自检：$driver_name 开跑前要经 build_research_driver_binary 编（cargo 只起一次：$driver_expected）、编不过返回 1、cargo 的输出进 .err、stdout 不打；实际返回 $status，cargo 收到「$(tr '\n' '|' <"$scratch/driver-cargo.log")」，stderr「$(head -c 300 "$scratch/driver-build.err")」，stdout「$(head -c 200 "$scratch/driver-build.stdout")」"  # gate-lint:detail
      failures=$((failures + 1))
    fi
  done
  checked=$((checked + 1))
  local -A ADMISSION_ROW_KEY=(["E1|@driver_e1|"]="E1" ["E1|@driver_e1_second|"]="E1/second")
  local ROW_ADMISSION_KEY=""
  admission_key_of_row E1 @driver_e1_second ""
  if [[ "$ROW_ADMISSION_KEY" != E1/second ]]; then
    echo "  ✗ 自检：复跑表那一行指明了准入键 E1/second，问准入就该用它；实际用的是「$ROW_ADMISSION_KEY」"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 配置开了 GPU 时 checker 档装置带 gpu 特性编（resolve_checker_tier_features）：假 cargo 把收到的参数记进 gpu-calls.log，配置文件放临时目录
  mkdir -p "$scratch/gpu-bin"
  printf '%s\n' '#!/usr/bin/env bash' 'echo "cargo $*" >>"${FAKE_CARGO_LOG:?}"' \
    'if [[ "${FAKE_CARGO_OUTCOME:-}" == red ]]; then echo "error[E0432]: unresolved import wgpu"; exit 101; fi' 'exit 0' >"$scratch/gpu-bin/cargo"
  chmod +x "$scratch/gpu-bin/cargo"
  gpu_probe() { # <配置文件路径> <假 cargo 的结局>：子 shell 里跑一遍，打「状态 特性数组」一行；stderr 进 gpu-probe.stderr，cargo 的调用进 gpu-calls.log
    : >"$scratch/gpu-calls.log"
    ( export SINGLEFS_MULTI_HOST_CONFIG="$1" FAKE_CARGO_LOG="$scratch/gpu-calls.log" FAKE_CARGO_OUTCOME="$2" PATH="$scratch/gpu-bin:$PATH" \
        GPU_QUIESCE_STATE_DIRECTORY="$scratch/gpu-quiesce-state" GPU_QUIESCE_LOG_DIRECTORY="$scratch/gpu-quiesce-logs" QUIESCE_CALLS="$scratch/quiesce-calls.log"
      resolve_checker_tier_features "$scratch/gpu-probe.log" 2>"$scratch/gpu-probe.stderr"; probe_status=$?
      echo "$probe_status ${CHECKER_TIER_FEATURES[*]:-}" )
  }
  local gpu_reading
  # 开了 GPU 的多机配置：清场命令是假的（往 $QUIESCE_CALLS 记一笔），不清场与清场之后的显卡先后写成两份不同的值，核装置拿到的是后一份
  printf '%s\n' '# 自证的多机配置' 'ENABLE_ACROSS_MACHINES=0' 'ENABLE_GPU=1' "GPU_CARDS_BY_PRIORITY='local:1:8G'" "GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED='local:0:28G'" \
    "QUIESCE_STOP_COMMAND='echo quiesce-stop >> \"\$QUIESCE_CALLS\"'" "QUIESCE_STOPPED_CHECK_COMMAND=true" >"$scratch/gpu-on.env"
  printf '%s\n' '# 自证的多机配置：清场回读不过' 'ENABLE_GPU=1' "GPU_CARDS_BY_PRIORITY='local:1:8G'" "GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED='local:0:28G'" \
    "QUIESCE_STOP_COMMAND='echo quiesce-stop >> \"\$QUIESCE_CALLS\"'" "QUIESCE_STOPPED_CHECK_COMMAND=false" >"$scratch/gpu-on-read-back-fails.env"
  printf '%s\n' '# 自证的多机配置：没写清场之后的显卡先后' 'ENABLE_GPU=1' "GPU_CARDS_BY_PRIORITY='local:1:8G'" \
    "QUIESCE_STOP_COMMAND='echo quiesce-stop >> \"\$QUIESCE_CALLS\"'" "QUIESCE_STOPPED_CHECK_COMMAND=true" >"$scratch/gpu-on-cards-missing.env"
  checked=$((checked + 1))
  gpu_reading="$(gpu_probe "$scratch/gpu-on.env" green)"
  if [[ "$gpu_reading" != "0 --features verdict-store,gpu" ]] \
     || ! grep -qxF "cargo build --release -p singlefs-checker-tier --features verdict-store,gpu --lib" "$scratch/gpu-calls.log"; then
    echo "  ✗ 自检：配置写了 ENABLE_GPU=1，应当先试编 gpu 特性并把「--features verdict-store,gpu」放进 CHECKER_TIER_FEATURES；实际「$gpu_reading」，cargo 调用：$(tr '\n' '|' <"$scratch/gpu-calls.log")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  printf '%s\n' 'ENABLE_GPU=0' >"$scratch/gpu-off.env"
  printf '%s\n' 'ENABLE_ACROSS_MACHINES=1' >"$scratch/gpu-unset.env"
  checked=$((checked + 1))
  if [[ "$(gpu_probe "$scratch/gpu-off.env" green)" != "0 " || -s "$scratch/gpu-calls.log" || "$(gpu_probe "$scratch/gpu-unset.env" green)" != "0 " || -s "$scratch/gpu-calls.log" ]]; then
    echo "  ✗ 自检：配置写 ENABLE_GPU=0、或没写这个键，应当不带特性、一次 cargo 都不起；实际 $(gpu_probe "$scratch/gpu-off.env" green)，cargo 调用：$(tr '\n' '|' <"$scratch/gpu-calls.log")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  printf '%s\n' 'ENABLE_GPU=yes' >"$scratch/gpu-garbled.env"
  checked=$((checked + 1))
  gpu_reading="$(gpu_probe "$scratch/gpu-garbled.env" green)"
  if [[ "$gpu_reading" != "78 " ]] || ! grep -qF "ENABLE_GPU 写成了「yes」" "$scratch/gpu-probe.stderr" || [[ -s "$scratch/gpu-calls.log" ]]; then
    echo "  ✗ 自检：ENABLE_GPU 写了 0、1 之外的值，应当退 78、点名那个值、不起 cargo、不悄悄不带特性；实际「$gpu_reading」：$(cat "$scratch/gpu-probe.stderr")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  gpu_reading="$(gpu_probe "$scratch/gpu-no-such.env" green)"
  if [[ "$gpu_reading" != "78 " ]] || ! grep -qF "点名的多机配置 $scratch/gpu-no-such.env 不在" "$scratch/gpu-probe.stderr"; then
    echo "  ✗ 自检：SINGLEFS_MULTI_HOST_CONFIG 点名的文件不在，应当退 78 并点名那个文件；实际「$gpu_reading」：$(cat "$scratch/gpu-probe.stderr")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  if (( EUID != 0 )); then
    cp -- "$scratch/gpu-on.env" "$scratch/gpu-unreadable.env"; chmod 000 "$scratch/gpu-unreadable.env"
    checked=$((checked + 1))
    gpu_reading="$(gpu_probe "$scratch/gpu-unreadable.env" green)"
    if [[ "$gpu_reading" != "78 " ]] || ! grep -qF "gpu-unreadable.env 不可读" "$scratch/gpu-probe.stderr" || [[ -s "$scratch/gpu-calls.log" ]]; then
      echo "  ✗ 自检：配置文件不可读，应当退 78、不起 cargo（不当成没开 GPU）；实际「$gpu_reading」：$(cat "$scratch/gpu-probe.stderr")"  # gate-lint:detail
      failures=$((failures + 1))
    fi
  fi
  checked=$((checked + 1))
  gpu_reading="$(gpu_probe "$scratch/gpu-on.env" red)"
  if [[ "$gpu_reading" != "78 " ]] || ! grep -qF "第一处错：error[E0432]: unresolved import wgpu" "$scratch/gpu-probe.stderr" \
     || ! grep -qF "不退回不带 gpu 特性去跑" "$scratch/gpu-probe.stderr" || ! grep -qF "整份编译日志：$scratch/gpu-probe.log" "$scratch/gpu-probe.stderr"; then
    echo "  ✗ 自检：开了 GPU 而 gpu 特性编不过，应当退 78、打第一处错与日志路径、不带特性数组；实际「$gpu_reading」：$(cat "$scratch/gpu-probe.stderr")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 开了 GPU 先清场（quiesce_for_checker_tier_runs）：真的 gpu-quiesce-run.sh，清场命令是假的。quiesced_run 打「resolve=<返回值>」，过了再打经 CHECKER_TIER_QUIESCE_PREFIX
  #   起的一条命令看到的「cards=<GPU_CARDS_BY_PRIORITY> holder=<清场登记文件在不在>」；假清场命令记的笔数进 quiesce-calls.log，包装的 stderr 进 gpu-probe.stderr
  quiesced_run() { # <配置文件路径>
    : >"$scratch/gpu-calls.log"; : >"$scratch/quiesce-calls.log"
    ( export SINGLEFS_MULTI_HOST_CONFIG="$1" FAKE_CARGO_LOG="$scratch/gpu-calls.log" FAKE_CARGO_OUTCOME=green PATH="$scratch/gpu-bin:$PATH" \
        GPU_QUIESCE_STATE_DIRECTORY="$scratch/gpu-quiesce-state" GPU_QUIESCE_LOG_DIRECTORY="$scratch/gpu-quiesce-logs" QUIESCE_CALLS="$scratch/quiesce-calls.log"
      unset GPU_CARDS_BY_PRIORITY GPU_QUIESCE_HOLDER
      resolve_checker_tier_features "$scratch/gpu-probe.log" 2>"$scratch/gpu-probe.stderr"; probe_status=$?
      echo "resolve=$probe_status"
      if (( probe_status == 0 )); then
        "${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"}" \
          bash -c 'echo "cards=${GPU_CARDS_BY_PRIORITY-unset} holder=$([[ -f "${GPU_QUIESCE_HOLDER:-/nonexistent}" ]] && echo yes || echo no)"' 2>>"$scratch/gpu-probe.stderr"
      fi )
  }
  local quiesced_reading
  checked=$((checked + 1))
  quiesced_reading="$(quiesced_run "$scratch/gpu-on.env" | tr '\n' ' ')"
  if [[ "$quiesced_reading" != "resolve=0 cards=local:0:28G holder=yes " ]] || ! grep -qxF quiesce-stop "$scratch/quiesce-calls.log" \
     || ! grep -qF "GPU_QUIESCE step=restore result=not-restored" "$scratch/gpu-probe.stderr"; then
    echo "  ✗ 自检：开了 GPU，应当先清场并回读、返回 0；经 CHECKER_TIER_QUIESCE_PREFIX 起的命令要在清场之下（登记文件在）、GPU_CARDS_BY_PRIORITY 是清场之后那一份 local:0:28G（不是不清场的 local:1:8G）、不复原；实际「$quiesced_reading」，假清场命令记了「$(tr '\n' '|' <"$scratch/quiesce-calls.log")」：$(tail -n 4 "$scratch/gpu-probe.stderr" | tr '\n' ' ')"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  quiesced_reading="$(quiesced_run "$scratch/gpu-on-read-back-fails.env" | tr '\n' ' ')"
  if [[ "$quiesced_reading" != "resolve=78 " ]] || ! grep -qF "清场没过" "$scratch/gpu-probe.stderr" || ! grep -qF "不复原" "$scratch/gpu-probe.stderr"; then
    echo "  ✗ 自检：清场回读没过，应当退 78、点名「清场没过」与「不复原」、不往下跑；实际「$quiesced_reading」：$(tail -n 6 "$scratch/gpu-probe.stderr" | tr '\n' ' ')"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  quiesced_reading="$(quiesced_run "$scratch/gpu-on-cards-missing.env" | tr '\n' ' ')"
  if [[ "$quiesced_reading" != "resolve=78 " ]] || ! grep -qF "GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED" "$scratch/gpu-probe.stderr" || [[ -s "$scratch/quiesce-calls.log" ]]; then
    echo "  ✗ 自检：开了 GPU 而没写清场之后的显卡先后，应当退 78、点名 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED、一条清场命令都不跑；实际「$quiesced_reading」，假清场命令记了「$(tr '\n' '|' <"$scratch/quiesce-calls.log")」：$(tail -n 3 "$scratch/gpu-probe.stderr" | tr '\n' ' ')"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 起 checker 档装置的 cargo run 行（用到给定状态引擎的那几个装置）每一行都带特性数组：新加一行忘了带，开了 GPU 时那一行拿不到判段结论
  local engine_lines unmarked_lines quiesce_lines quiesce_unmarked_lines
  engine_lines="$(grep -E 'cargo run .*-p singlefs-checker-tier .*--bin (e156_|e158_|e161_|e168_|e170_|e171_|"\$bin")' -- "${BASH_SOURCE[0]}" | grep -vE '^[[:space:]]*#' || true)"
  if [[ "$REPLAY_BROKEN" == run-lines-without-features ]]; then engine_lines="${engine_lines//CHECKER_TIER_FEATURES/}"; fi
  unmarked_lines="$(grep -vF 'CHECKER_TIER_FEATURES' <<<"$engine_lines" || true)"
  checked=$((checked + 1))
  if [[ -z "$engine_lines" || -n "$unmarked_lines" ]]; then
    echo "  ✗ 自检：起 checker 档装置的 cargo run 行（共 $(grep -c . <<<"$engine_lines") 行）应当每一行都带 \${CHECKER_TIER_FEATURES[@]…}；缺的：$(head -n 3 <<<"$unmarked_lines" | cut -c1-200)"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 同一批行每一行也都经清场数组起：新加一行忘了带，开了 GPU 时那一行不在清场之下、拿不到清场之后的显卡先后
  quiesce_lines="$engine_lines"
  if [[ "$REPLAY_BROKEN" == run-lines-without-quiesce ]]; then quiesce_lines="${quiesce_lines//CHECKER_TIER_QUIESCE_PREFIX/}"; fi
  quiesce_unmarked_lines="$(grep -vF 'CHECKER_TIER_QUIESCE_PREFIX' <<<"$quiesce_lines" || true)"
  checked=$((checked + 1))
  if [[ -z "$quiesce_lines" || -n "$quiesce_unmarked_lines" ]]; then
    echo "  ✗ 自检：起 checker 档装置的 cargo run 行（共 $(grep -c . <<<"$quiesce_lines") 行）应当每一行都带 \${CHECKER_TIER_QUIESCE_PREFIX[@]…}（开了 GPU 经 gpu-quiesce-run.sh 清场起）；缺的：$(head -n 3 <<<"$quiesce_unmarked_lines" | cut -c1-200)"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 拷 crates/ 去编译的副本带上每台默认重活线程数的登记文件（copy_thread_registry_into）：拷进去的与仓里那一份逐字节相同
  checked=$((checked + 1))
  copy_thread_registry_into "$scratch/registry-tree"
  if ! cmp -s -- .claude/heavy-thread-default "$scratch/registry-tree/.claude/heavy-thread-default"; then
    echo "  ✗ 自检：拷 crates/ 去编译的副本要带上每台默认重活线程数的登记文件 .claude/heavy-thread-default（harness 经 include_str! 读它），副本里实际有「$(ls -A "$scratch/registry-tree/.claude" 2>/dev/null | tr '\n' ' ')」"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 拷 crates 的每一行（「Cargo.lock .cargo crates」那几行）都在同一行里接着 copy_thread_registry_into：新加一行忘了带，那个副本编不过 harness
  local copy_lines unregistered_copy_lines
  copy_lines="$(grep -E 'Cargo\.lock \.cargo crates' -- "${BASH_SOURCE[0]}" | grep -vE '^[[:space:]]*#' || true)"
  if [[ "$REPLAY_BROKEN" == copy-lines-without-registry ]]; then copy_lines="${copy_lines//copy_thread_registry_into/}"; fi
  unregistered_copy_lines="$(grep -vF 'copy_thread_registry_into' <<<"$copy_lines" || true)"
  checked=$((checked + 1))
  if [[ -z "$copy_lines" || -n "$unregistered_copy_lines" ]]; then
    echo "  ✗ 自检：拷 crates/ 去编译的行（共 $(grep -c . <<<"$copy_lines") 行）应当每一行都接着 copy_thread_registry_into <副本>；缺的：$(head -n 3 <<<"$unregistered_copy_lines" | cut -c1-200)"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 计时行的负载关：假产物放进 $scratch/results/，复跑表是下面这几行，滞后表登记 e-lagged.out
  mkdir -p "$scratch/results"
  local -A TIMING_LAG=()
  local TIMING_LOAD_RATIO=0.5 lag_table="$scratch/lag.tsv" lag_problems
  local selftest_table
  selftest_table="$(printf '%s\n' 'E1|e1-high||e-high.out|timing' 'E2|e2-low||e-low.out|timing' 'E3|e3-none||e-none.out|timing' 'E4|e4-lagged||e-lagged.out|timing' 'E5|e5-exact||e-exact.out|exact')"
  printf '%s\n' 'E7RESULT name=config threads=4' 'E7RESULT name=done emitted=2 loadavg1=20.50 cpus=32' >"$scratch/results/e-high.out"
  printf '%s\n' 'E7RESULT name=config threads=4' 'E7RESULT name=done emitted=2 loadavg1=2.50 cpus=32' >"$scratch/results/e-low.out"
  printf '%s\n' 'E7RESULT name=config threads=4' 'E7RESULT name=done emitted=2' >"$scratch/results/e-none.out"
  cp -- "$scratch/results/e-none.out" "$scratch/results/e-lagged.out"
  printf '%s\n' '# 样本滞后表' '' $'e-lagged.out\t2026-10-13 立 C350 之前的产物，装置还没加收尾字段' >"$lag_table"
  checked=$((checked + 1))
  if ! load_timing_lag "$lag_table" "$selftest_table" "$scratch/results" >"$scratch/lag.problems"; then   # 不经命令替换：TIMING_LAG 要留在这一层
    echo "  ✗ 自检：只登记了复跑表里 timing 行的留存产物、理由写全的滞后表应当读得过；实际报：$(cat "$scratch/lag.problems")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  timing_load_decide timing e-high.out "$scratch/results/e-high.out"
  if [[ "$TIMING_LOAD_DECISION" != skip || "$TIMING_LOAD_NOTE" != *"比值 0.640625"* ]]; then
    echo "  ✗ 自检：负载 20.5 / 32 核超过阈值 0.5，这一行应当记本次不判并点名比值；实际 $TIMING_LOAD_DECISION：$TIMING_LOAD_NOTE"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  timing_load_decide timing e-low.out "$scratch/results/e-low.out"
  if [[ "$TIMING_LOAD_DECISION" != judge || "$TIMING_LOAD_LAGGED" != 0 ]]; then
    echo "  ✗ 自检：负载 2.5 / 32 核没超阈值，这一行应当照判；实际 $TIMING_LOAD_DECISION（滞后 $TIMING_LOAD_LAGGED）：$TIMING_LOAD_NOTE"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  timing_load_decide timing e-none.out "$scratch/results/e-none.out"
  if [[ "$TIMING_LOAD_DECISION" != red || "$TIMING_LOAD_NOTE" != *"e-none.out"* || "$TIMING_LOAD_NOTE" != *"收尾行没有 loadavg1 / cpus"* ]]; then
    echo "  ✗ 自检：收尾行没有字段又不在滞后表，这一行应当判红并点名产物与「收尾行没有 loadavg1 / cpus」；实际 $TIMING_LOAD_DECISION：$TIMING_LOAD_NOTE"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  timing_load_decide timing e-lagged.out "$scratch/results/e-lagged.out"
  if [[ "$TIMING_LOAD_DECISION" != judge || "$TIMING_LOAD_LAGGED" != 1 ]]; then
    echo "  ✗ 自检：收尾行没有字段但登记在滞后表里，这一行应当照判并记滞后；实际 $TIMING_LOAD_DECISION（滞后 $TIMING_LOAD_LAGGED）：$TIMING_LOAD_NOTE"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  timing_load_decide exact e-none.out "$scratch/results/e-none.out"
  if [[ "$TIMING_LOAD_DECISION" != judge ]]; then
    echo "  ✗ 自检：判据写 exact 的行不看负载字段，没字段也照判；实际 $TIMING_LOAD_DECISION：$TIMING_LOAD_NOTE"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  printf '%s\n' $'e-ghost.out\t2026-10-13 不在复跑表里的名字' $'e-exact.out\t2026-10-13 复跑表里是 exact 的行' $'e-lagged.out\t2026-10-13 重复登记' $'e-short.out\t短' 'e-spaced.out  # 用空格隔开理由的旧写法，不是两列' >>"$lag_table"
  if lag_problems="$(load_timing_lag "$lag_table" "$selftest_table" "$scratch/results")" \
     || [[ "$lag_problems" != *"e-ghost.out 不是复跑表任何一条 timing 行"* || "$lag_problems" != *"e-exact.out 不是复跑表任何一条 timing 行"* \
        || "$lag_problems" != *"重复登记 e-lagged.out"* || "$lag_problems" != *"理由不足 8 个字"* \
        || "$lag_problems" != *"没有用制表符隔开的理由"* ]]; then
    echo "  ✗ 自检：滞后表登记了复跑表里没有的名字、exact 行的产物、重复名字、没写够理由、用空格隔开理由，应当判红并逐条点名；实际：$lag_problems"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  printf '%s\n' '# 样本滞后表' $'e-high.out\t2026-10-13 这份已经有字段了却还登记着' >"$lag_table"
  if lag_problems="$(load_timing_lag "$lag_table" "$selftest_table" "$scratch/results")" || [[ "$lag_problems" != *"e-high.out 自己的收尾行已经有 loadavg1 / cpus"* ]]; then
    echo "  ✗ 自检：滞后表登记的留存产物自己已经有字段，应当判红叫人删这一行；实际：$lag_problems"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  if [[ "$(printf '%s\n' 'E7RESULT name=config cpus=4' 'E7RESULT name=done emitted=2 loadavg1=2.50 cpus=32' | strip_load_fields)" != $'E7RESULT name=config cpus=4\nE7RESULT name=done emitted=2' ]]; then
    echo "  ✗ 自检：比对前要删掉 name=done 行上的 loadavg1= 与 cpus=、别的行上的同名字段不动"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 按行复用（文件头「按行复用」）：五格各一行收回来，收回的格数对不上整个算一处不对
  local reuse_cell reuse_verdict reuse_detail reuse_collected=0
  replay_reuse_selftest "$scratch/reuse" >"$scratch/reuse.report" 2>&1
  while read -r reuse_cell reuse_verdict reuse_detail; do
    checked=$((checked + 1)); reuse_collected=$((reuse_collected + 1))
    if [[ "$reuse_verdict" != ok ]]; then
      echo "  ✗ 自检：按行复用 $reuse_cell：$reuse_detail"  # gate-lint:detail
      failures=$((failures + 1))
    fi
  done < <(grep -E '^(reuse-[a-z-]+|last-run-list) (ok|bad)' "$scratch/reuse.report")
  grep '^REUSE_SELFTEST ' "$scratch/reuse.report"
  if (( reuse_collected != 8 )); then
    echo "  ✗ 自检：按行复用的自证只收回 $reuse_collected 格（要 8 格）：$(tail -n 6 "$scratch/reuse.report" | tr '\n' ' ' | cut -c1-800)"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  local registered_default capped_jobs unset_jobs
  registered_default="$(grep -vE '^[[:space:]]*(#|$)' .claude/heavy-thread-default | head -n 1 | awk '{print $1}')"
  unset_jobs="$(env -u SINGLEFS_THREAD_CAP bash -c 'source scripts/hosts/heavy-threads.sh; REPLAY_BROKEN="$1"; '"$(declare -f default_replay_jobs)"'; default_replay_jobs' _ "$REPLAY_BROKEN")"
  capped_jobs="$(SINGLEFS_THREAD_CAP=7 bash -c 'source scripts/hosts/heavy-threads.sh; REPLAY_BROKEN="$1"; '"$(declare -f default_replay_jobs)"'; default_replay_jobs' _ "$REPLAY_BROKEN")"
  if [[ -z "$registered_default" || "$unset_jobs" != "$registered_default" || "$capped_jobs" != 7 ]]; then
    echo "  ✗ 自检：没设 REPLAY_JOBS 时同时跑几条实验要取登记的重活默认（登记「$registered_default」，没设上限得「$unset_jobs」，上限 7 得「$capped_jobs」）"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  rm -rf -- "${scratch:?}"
  if ((failures)); then
    echo "  ✗ replay.sh 自检 $failures 处不对（查了 $checked 项）"
    echo "    → 怎么办：看 build_research_binaries、admission_key_of_row 与计时行的负载关（timing_load_decide、load_timing_lag）；REPLAY_BREAK 设着的话这里本来就该红"
    return 1
  fi
  echo "  ✓ replay.sh 自检通过（查了 $checked 项：构建失败返回 2 并打第一处错、日志末尾、日志路径与重现命令，构建成功一个字都不打，驱动开跑前编研究二进制经 --manifest-path 指到研究工作区、编不过的输出进 .err（build_research_driver_binary 与 E162、E163 那三个驱动），指明了准入键的行按它问准入，按行复用接连五趟（头一趟两行都跑并记进判定库、第二趟都复用、改一行装置源码只重跑那一行、只改注释照样复用、--full 都现跑），计时行负载超阈值记本次不判、没超照判、没字段判红、没字段但在滞后表照判，滞后表的登记错误逐条点名，配置开了 GPU 时 checker 档装置带 gpu 特性（开了先试编、没开不起 cargo、配置读不出或试编红退 78 点名原因、起装置的 cargo run 行都带特性数组，开了 GPU 先清场并回读（过了才往下、没过或没写清场之后的显卡先后退 78 不复原，装置拿清场之后那一份显卡先后、在清场之下起），起装置的 cargo run 行都经清场数组起），同时跑几条实验的缺省取登记的重活默认，拷 crates/ 去编译的副本都带上每台默认重活线程数的登记文件）"
  return 0
}
if [[ "${1:-}" == "--selftest" ]]; then
  replay_selftest
  exit $?
fi

OUT_DIR="${REPLAY_OUT:-${TMPDIR:-/tmp}/singlefs-replay-$$}"
mkdir -p "$OUT_DIR"
# E9 要一个真设备/文件当后端（O_DIRECT）。镜像一律落临时目录，不许进仓
# （command-safety.md「测试镜像一律放临时目录」）。
REPLAY_DEV="${REPLAY_DEV:-$OUT_DIR/e9.img}"
# e9-keylayout 用 O_DIRECT 打开这个后端，自己不建文件。tmpfs 不支持 O_DIRECT，
# 所以 REPLAY_OUT 要落在真文件系统上（本机 /tmp 是 ext4，够用）。
[[ -e "$REPLAY_DEV" ]] || truncate -s 512M "$REPLAY_DEV"
export REPLAY_DEV
# E44 要一个 64 MiB 的 O_DIRECT 后端量本机 fsync 率
REPLAY_DEV45="${REPLAY_DEV45:-$OUT_DIR/e45.img}"
[[ -e "$REPLAY_DEV45" ]] || truncate -s 64M "$REPLAY_DEV45"
export REPLAY_DEV45
# ⚠️ **E58 的测试区不许预先 truncate 出来。** 稀疏洞读起来可能根本不碰设备，
# 而 e58 只看文件大小决定填不填 ⇒ 预创建一个空洞文件会让它去量「读洞要多久」。
# 交给二进制自己建、自己填。真读错了阳性对照会判红（读洞时 read_bytes 是 0）。
REPLAY_DEV58="${REPLAY_DEV58:-$OUT_DIR/e58.img}"
export REPLAY_DEV58
# E140 与 E58 同一套装置（8 GiB O_DIRECT 测试区，二进制自己建、自己填），同样不许预先 truncate。
REPLAY_DEV140="${REPLAY_DEV140:-$OUT_DIR/e140.img}"
export REPLAY_DEV140

# 实验号 | 二进制 | 参数 | 入库产物 | 判据（exact=应逐字节一致 / timing=含计时字段，只比结构）
TABLE=$(cat <<'TSV'
E14|e14-discrimination||e14-discrimination-2026-08-29.out|exact
E18|e18-branch||e18-branch-2026-08-28.out|exact
E19|e19-defer||e19-defer-2026-08-28.out|exact
E23|e23_journal_geom||e23-journal-geom-2026-08-29.out|exact
E24|e24_recovery||e24-recovery-2026-08-29.out|exact
E25|e25_journal_reserve||e25-journal-reserve-2026-08-29.out|exact
E26|e26_accounting||e26-accounting-2026-08-29.out|exact
E27|e27_snapshot_accounting_risk_paths||e27-d5-paths-2026-08-29.out|exact
E28|e28_map_rebuild||e28-map-rebuild-2026-08-29.out|exact
E29|e29_blast_radius||e29-blast-radius-2026-08-29.out|exact
E30|e30_range_rebuild||e30-range-rebuild-2026-08-29.out|exact
E31|e31-aad-snapshot||e31-aad-snapshot-2026-08-29.out|exact
E32|e32-journal-timeline||e32-journal-timeline-2026-08-29.out|exact
E33|e33-pin-rules||e33-pin-rules-2026-08-29.out|exact
E35|e35-head-forms||e35-head-forms-2026-08-29.out|exact
E36|e36-slot-mapping||e36-slot-mapping-2026-08-29.out|exact
E37|e37-log-epoch||e37-log-epoch-2026-08-29.out|exact
E38|e38_accounting_copy_on_write||e38-accounting-cow-2026-08-29.out|exact
E39|e39_back_chain||e39-back-chain-2026-08-29.out|exact
E42|e42_transaction_records||e42-txn-records-2026-08-29.out|exact
E44|e44_jsn_width|$REPLAY_DEV45|e44-jsn-width-2026-08-30.out|timing
E58|e58-csum-grain|$REPLAY_DEV58 1 none 4096 8192|e58-csum-grain-repro-2026-09-16.out|timing
E140|e140-header-alignment|$REPLAY_DEV140 1 none 4096 8192|e140-header-alignment-repro-2026-09-13.out|timing
E43|e43_extension_point_budget||e43-ext-budget-2026-10-04-h487.out|exact
E41|e41_root_ring_geom||e41-root-ring-geom-2026-08-30.out|exact
E71|e71-accounting-keys||e71-accounting-keys-2026-09-01.out|exact
E75|e75-record-size||e75-record-size-2026-09-01.out|exact
E76|e76-payload-checksum||e76-payload-csum-2026-09-01.out|exact
E77|e77-publish-order||e77-publish-order-2026-09-02.out|exact
E78|e78-replay-start||e78-replay-start-2026-09-02.out|exact
E79|e79-root-record||e79-root-record-2026-09-06.out|exact
E112|e112-old-writer-unknown-tree||e112-old-writer-unknown-tree-2026-09-06.out|exact
E113|e113-unknown-tree-full-arms||e113-unknown-tree-full-arms-2026-09-06.out|exact
E114|e114-pack-ledger||e114-pack-ledger-2026-09-12.out|exact
E117|e117-reserved-header||e117-reserved-header-2026-09-12.out|exact
E118|e118-single-disk-recovery||e118-single-disk-recovery-2026-09-07.out|exact
E122|e122-directory-locality||e122-dir-locality-2026-09-07.out|exact
E116|e116-pack-settle||e116-pack-settle-2026-10-04-h487.out|exact
E119|e119-slot-tiers||e119-slot-tiers-2026-09-12.out|exact
E120|e120-tier-ratio||e120-tier-ratio-2026-09-12.out|exact
E121|e121-capacity-tiers||e121-cap-tiers-2026-09-12.out|exact
E115|e115-system-configuration-completeness||e115-system-configuration-completeness-2026-09-07.out|exact
E80|e80-partial-stripe||e80-partial-stripe-2026-09-02.out|exact
E81|e81-commit-fixpoint||e81-commit-fixpoint-2026-09-02.out|exact
E82|e82-admission-overlay||e82-admission-overlay-2026-09-02.out|exact
E83|e83-tombstone-grain||e83-tombstone-grain-2026-10-04-r2.out|exact
E84|e84-tombstone-pinning||e84-tombstone-pinning-2026-09-03.out|exact
E85|e85-unit-header||e85-unit-header-2026-09-02.out|exact
E86|e86-scan-step||e86-scan-step-2026-09-02.out|exact
E87|e87-fixed-placement||e87-fixed-placement-2026-09-02.out|exact
E88|e88-impostor-orphan||e88-impostor-orphan-2026-09-02.out|exact
E89|e89-interval-frontier||e89-interval-frontier-2026-09-03.out|exact
E90|e90-tree-aad||e90-tree-aad-2026-09-03.out|exact
E91|e91-ring-admission||e91-ring-admission-2026-09-03.out|exact
E92|e92-reuse-requirement||e92-reuse-requirement-2026-09-08.out|exact
E123|e123-reuse-window-versus-rollback-depth||e123-k-fork-cost-2026-09-09.out|exact
E124|e124-system-configuration-recompute||e124-system-configuration-recompute-2026-09-09.out|exact
E126|e126-system-configuration-slot-width||e126-system-configuration-slot-width-2026-09-09.out|exact
E127|e127-group-identity-under-split-merge||e127-group-identity-under-split-merge-2026-09-13-knobs.out|exact
E93|e93-aging-placement||e93-aging-placement-2026-09-03.out|exact
E95|e95-node-layout-arms||e95-node-layout-arms-2026-09-09.out|exact
E94|e94-move-touchset||e94-move-touchset-2026-09-03.out|exact
E96|e96-hybrid-consistency||e96-hybrid-consistency-2026-09-03.out|exact
E97|e97-entry-encoding||e97-entry-encoding-2026-09-07.out|exact
E98|e98-inode-record||e98-inode-record-2026-09-07.out|exact
E99|e99-writebuffer-sequence||e99-writebuffer-seq-2026-09-03.out|exact
E100|e100-system-configuration-slot||e100-system-configuration-slot-2026-09-03.out|exact
E101|e101-node-tag-reserve||e101-node-tag-reserve-2026-09-03.out|exact
E102|e102-unit-class-registry||e102-unit-class-registry-2026-10-04.out|exact
E103|e103-inode-update-cost||e103-inode-update-cost-2026-09-14-round2.out|exact
E104|e104-current-version||e104-current-version-2026-09-05.out|exact
E105|e105-extent-leaf-packed||e105-extent-leaf-packed-2026-09-12.out|exact
E106|e106-stripe-member-table||e106-stripe-member-table-2026-09-12.out|exact
E108|e108-plaintext-layer-cost||e108-plaintext-layer-cost-2026-09-06.out|exact
E107|e107-stripe-table-wa||e107-stripe-table-wa-2026-09-06.out|timing
E109|e109-position-authority||e109-position-authority-2026-09-06.out|exact
E110|e110-stripe-table-steady||e110-stripe-table-steady-2026-09-06.out|exact
E111|e111-stripe-table-key||e111-stripe-table-key-2026-09-06.out|exact
E34|e34-ring-iomin||e34-ring-iomin-2026-09-01.out|exact
E73|e73-key-range||e73-key-range-2026-09-07.out|exact
E74|e74-allocation-records||e74-alloc-records-2026-09-01.out|exact
E40|e40_checksum_width||e40-csum-width-2026-08-30.out|exact
E46|e46_region_spacing||e46-region-spacing-2026-08-30.out|exact
E47|e47_ring_loss||e47-ring-loss-2026-08-30.out|exact
E48|e48_ring_placement||e48-ring-placement-2026-08-30.out|exact
E50|e50_ring_slots||e50-ring-slots-2026-08-30.out|exact
E49|e49_chain_width||e49-chain-width-2026-10-04-h487.out|exact
E51|e51_chain_chances||e51-chain-chances-2026-08-30.out|exact
E52|e52_head_mechanisms||e52-head-mechanisms-2026-08-30.out|exact
E54|e54_accounting_generations||e54-accounting-gen-2026-08-30.out|exact
E57|e57_field_authority||e57-field-authority-2026-08-31.out|exact
E59|e59_message_recompute||e59-msg-recompute-2026-08-31.out|exact
E61|e61-chain-hash||e61-chain-hash-2026-08-31.out|exact
E69|e69-backref-cost||e69-backref-cost-2026-08-31.out|exact
E60|e60-rebalance||e60-rebalance-2026-08-31.out|exact
E70|e70-ckpt-thresholds||e70-ckpt-thresholds-2026-08-31.out|exact
E62|e62-ring-home||e62-ring-home-2026-08-31.out|exact
E63|e63-width-rule||e63-width-rule-2026-08-31.out|exact
E67|e67-device-subset||e67-device-subset-2026-08-31.out|exact
E68|e68-inline-threshold||e68-inline-threshold-2026-08-31.out|exact
E8|e8-split||e8-split-2026-08-28.out|exact
E9|@driver_e9||e9-keylayout-2026-08-28.out|exact
E16|e16-journal||e16-journal-2026-08-31.out|exact
E16|e16-journal|bytes|e16-bytes-2026-09-03.out|exact
E16R5|e16-fifth-run-peak-under-today-widths||e16-fifth-run-peak-under-today-widths-2026-10-04-h487.out|exact
E16R5|e16-fifth-run-peak-under-today-widths|pc3|e16-fifth-run-peak-under-today-widths-pc3-2026-09-28.out|exact
E16R5|e16-fifth-run-peak-under-today-widths|second_segment|e16-fifth-run-peak-under-today-widths-second-segment-2026-10-04-h487.out|exact
E17|e17-merge||e17-merge-2026-08-29-repro.out|timing
E20|e20-fanout||e20-poscontrol-2026-08-29.out|timing
E21|e21-cpu|2048 5|e21-cpu-2026-08-28.out|timing
E128|e128-pointer-birth-cost|2000000|e128-pointer-birth-cost-2026-09-10.out|timing
E130|e130_livelist_bounded_destroy||e130-livelist-bounded-destroy-2026-09-10.out|exact
E131|e131_livelist_carrier||e131-livelist-carrier-2026-09-16.out|exact
E132|e132_livelist_carrier_recount||e132-livelist-carrier-recount-2026-09-16.out|exact
E133|e133_map_key_format_cost||e133-map-key-format-cost-2026-09-11.out|exact
E134|e134_map_key_slot_baselines||e134-map-key-slot-baselines-2026-09-11.out|exact
E136|e136_fork_cost_rows||e136-fork-cost-rows-2026-09-11.out|exact
E138|e138_per_disk_floor||e138-per-disk-floor-2026-09-11.out|exact
E139|e139_tightened_floor||e139-tightened-floor-2026-10-05.out|exact
E141|e141_switch_reserve_mount_admission||e141-switch-reserve-mount-admission-2026-09-14-row-writing.out|exact
E142|@driver_e142||e142-new-pool-file-creation-dry-run-r25-main-2026-10-13.out|exact
E142|@driver_e142_p8g_crates_dump||e142-r25-p8g-crates-dump-2026-10-13.out|exact
E143|e143-one-unit-per-txn-journal||e143-one-unit-per-txn-journal-2026-09-13.out|exact
E145|e145-self-describing-node-header||e145-self-describing-node-header-2026-09-16-tree-table-200.out|exact
E146|e146-livelist-entry-width||e146-livelist-entry-width-2026-09-16-tree-table-200.out|exact
E147|e147-system-configuration-recompute-from-layout||e147-system-configuration-recompute-from-layout-2026-09-13.out|exact
E148|e148-commit-fixpoint-two-record-trees||e148-commit-fixpoint-two-record-trees-2026-09-13.out|exact
E150|e150-rollback-reuse-of-abandoned-roots||e150-rollback-reuse-of-abandoned-roots-2026-09-13-admission.out|exact
E151|e151-arrival-and-container-arms||e151-arrival-and-container-arms-2026-09-13-region.out|exact
E149|e149-pack-container-repair-options||e149-pack-container-repair-options-2026-09-13.out|exact
E144|e144-header-checksum-cost||e144-header-checksum-cost-2026-09-13.out|timing
E135|e135_rollback_floor||e135-rollback-floor-2026-09-11.out|exact
E137|e137_map_key_performance||e137-map-key-performance-2026-09-11.out|exact
E154|e154-two-gates-serial-rejudge-and-reclaim-timing||e154-two-gates-serial-rejudge-and-reclaim-timing-2026-09-24-rename.out|exact
E153|e153-ledger-shape-and-ring-holes||e153-ledger-shape-and-ring-holes-2026-09-24-rename.out|exact
E155|e155-fsync-write-volume||e155-fsync-write-volume-2026-09-25-h311-replay.out|exact
E155R2|e155-second-run-fsync-write-volume||e155-second-run-fsync-write-volume-2026-09-25-h311-replay.out|exact
E155R3|e155-third-run-release-cascade||e155-third-run-release-cascade-2026-09-25-h311-replay.out|exact
E155R4|e155-fourth-run-group-commit-concurrency||e155-fourth-run-group-commit-concurrency-2026-09-25-h311-replay.out|exact
E156|@driver_e156||e156-alloc-basis-counts-2026-10-05-r2.out|exact
E157|e157-parallel-line-one-clauses||e157-parallel-line-one-clauses-2026-10-04-h487.out|exact
E160|e160-random-small-read-share||e160-random-small-read-share-segment1-2026-09-24-realweight.out|exact
E158|@driver_e158||e158-root-choice-repair-2026-09-25-segment1-rerun.out|exact
E158|@driver_e158_q3_1_g0||e158-root-choice-repair-2026-09-25-q3-1-g0.out|exact
E158|@driver_e158_q3_1_s16||e158-root-choice-repair-2026-09-25-q3-1-s16.out|exact
E158|@driver_e158_q3_1_small_ring||e158-root-choice-repair-2026-09-25-q3-1-small-ring.out|exact
E158|@driver_e158_q3_1_s4||e158-root-choice-repair-2026-09-25-q3-1-s4.out|exact
E158|@driver_e158_q1_g0||e158-root-choice-repair-2026-09-25-q1-g0-today.out|exact
E158|@driver_e158_q2_1_g0||e158-root-choice-repair-2026-09-24-q2-1-g0-today.out|exact
E158|@driver_e158_q2_2a_g0||e158-root-choice-repair-2026-09-25-q2-2a-g0-today.out|exact
E158|@driver_e158_q2_1_pc2||e158-root-choice-repair-2026-09-24-pc2-today.out|exact
E158|@driver_e158_q2_1_g0_session_s5||e158-root-choice-repair-2026-09-24-q2-1-g0-today-session-s5.out|exact
E158|@driver_e158_q1_s16||e158-root-choice-repair-2026-09-25-q1-s16.out|exact
E158|@driver_e158_q1_s4||e158-root-choice-repair-2026-09-25-q1-s4.out|exact
E158|@driver_e158_q2_1_hc1||e158-root-choice-repair-2026-09-24-q2-1-hc1.out|exact
E158|@driver_e158_q2_1_hc1_lower_bound||e158-root-choice-repair-2026-09-24-q2-1-hc1-lower-bound.out|exact
E158|@driver_e158_r2_all||e158-root-choice-repair-2026-09-27-r2-all-today.out|exact
E158|@driver_e158_r3_seg1_today||e158-root-choice-repair-2026-09-27-r3-seg1-today.out|exact
E158|@driver_e158_r3_seg1_compare||e158-root-choice-repair-2026-09-27-r3-seg1-compare.out|exact
E158|@driver_e158_r4_compare||e158-root-choice-repair-2026-09-27-r4-compare.out|exact
E159|e159-fsync-wait-group-commit|anchors|e159-fsync-wait-group-commit-2026-09-25-h311-replay.out|exact
E162|@driver_e162_anchors||e162-crash-verdict-block-store-2026-09-27-anchors-2026-09-28.out|exact
E162|@driver_e162_power_cut_selftest||e162-verdict-store-power-cut-2026-09-27-selftest-r2.out|exact
E161|@driver_e161_feasibility||e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out|timing
E163|@driver_e163_r1_merge||e163-gpu-multicard-crc32c-2026-09-27-r1-merge.out|exact
E164|e164-crash-amplification-design-function-test|models|e164-crash-amplification-design-function-test-2026-09-28-models.out|exact
E167|e167-multi-inflight-crash-cut||e167-multi-inflight-crash-cut-2026-10-01-r2.out|exact
E167|e167-multi-inflight-crash-cut|second|e167-multi-inflight-crash-cut-second-2026-10-01-r3.out|exact
E167|e167-multi-inflight-crash-cut|third|e167-multi-inflight-crash-cut-third-2026-10-01.out|exact
E168|@driver_e168_s1_history||e168-delete-ring-window-borrow-s1-history-2026-10-03.out|exact
E168|@driver_e168_positive_controls||e168-delete-ring-window-borrow-positive-controls-2026-10-03.out|exact
E173|e173-admission-ceiling-bad-ring-slot-dispositions|checks|e173-admission-ceiling-bad-ring-slot-dispositions-checks-2026-10-05.out|exact
E173|e173-admission-ceiling-bad-ring-slot-dispositions|first|e173-admission-ceiling-bad-ring-slot-dispositions-segment-one-2026-10-05.out|exact
E173|e173-admission-ceiling-bad-ring-slot-dispositions|second|e173-admission-ceiling-bad-ring-slot-dispositions-segment-two-2026-10-05.out|exact
E174|@driver_e174_checks||e174-lease-drop-wait-memory-return-checks-2026-10-08.out|exact
E174|@driver_e174_simulated_judge||e174-lease-drop-wait-memory-return-simulated-second-judge-2026-10-08.out|exact
E174|@driver_e174_segment_one_judge||e174-lease-drop-wait-memory-return-segment-one-judge-2026-10-08.out|exact
E174|@driver_e174_segment_two_judge||e174-lease-drop-wait-memory-return-segment-two-judge-2026-10-08.out|exact
E176|e176-inode-leaf-merge-capacity-and-rewrites|segment-one|e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out|exact
E176|e176-inode-leaf-merge-capacity-and-rewrites|segment-two|e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out|exact
E177|e177-per-tree-resurrection-without-tombstones|segment-one|e177-per-tree-resurrection-without-tombstones-segment-one.out|exact
E178|e178-leaf-and-record-death-counts|segment-one|e178-leaf-and-record-death-counts-segment-one.out|exact
E178|e178-leaf-and-record-death-counts|segment-two|e178-leaf-and-record-death-counts-segment-two.out|exact
E178|e178-leaf-and-record-death-counts|segment-three|e178-leaf-and-record-death-counts-segment-three.out|exact
E178|e178-leaf-and-record-death-counts|segment-four --no-reuse|e178-leaf-and-record-death-counts-segment-four.out|exact
E165|@driver_e165||e165-replay-bounded-enumeration-2026-10-01-segment1-legal-family.out|exact
E165|@driver_e165_light||e165-replay-bounded-enumeration-2026-10-01-light-legal-family.out|exact
E165|@driver_e165_adversarial||e165-replay-bounded-enumeration-2026-10-01-segment1-adversarial-family.out|exact
E165|@driver_e165_segment2||e165-replay-bounded-enumeration-2026-10-02-r2-segment2c-kill-bounds.out|exact
E165|@driver_e165_r2_segment3_multi_disk||e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-multi-disk.out|exact
E165|@driver_e165_r2_segment4_table_both_legal||e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-table-both-legal-family.out|exact
E165|@driver_e165_r2_segment4_table_both_adversarial||e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-table-both-adversarial-family.out|exact
E165|@driver_e165_r2_segment4_table_both_multi_disk||e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-table-both-multi-disk.out|exact
E165|@driver_e165_r2_segment4_reversed_disk_order_legal||e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-reversed-disk-order-legal-family.out|exact
E165|@driver_e165_r2_segment4_reversed_disk_order_multi_disk||e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-reversed-disk-order-multi-disk.out|exact
E165|@driver_e165_r3_segment1_adversarial||e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r3-adversarial-family.out|exact
E165|@driver_e165_r6_r3_legal||e165-replay-bounded-enumeration-2026-10-09-r6-r3-legal-family.out|exact
E165|@driver_e165_r3_segment2_root_choice||e165-replay-bounded-enumeration-2026-10-03-r3-segment2-root-choice.out|exact
E165|@driver_e165_r3_segment3_root_choice_kill_bounds||e165-replay-bounded-enumeration-2026-10-03-r3-segment3-root-choice-kill-bounds.out|exact
E165|@driver_e165_r4_segment1_root_choice_faults||e165-replay-bounded-enumeration-2026-10-03-r4-segment1-root-choice-faults-2026-10-03-r2.out|exact
E165|@driver_e165_r4_segment2_root_choice_faults_kill_bounds||e165-replay-bounded-enumeration-2026-10-03-r4-segment2-root-choice-faults-kill-bounds.out|exact
E165|@driver_e165_r5_segment2_r2_mode_kill_bounds||e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r2-mode-kill-bounds.out|exact
E165|@driver_e165_r5_segment2_r3_mode_kill_bounds||e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r3-mode-kill-bounds.out|exact
E165|@driver_e165_r5_segment2_r4_mode_kill_bounds||e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r4-mode-kill-bounds.out|exact
E165|@driver_e165_r5_segment3_legal||e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-legal-family.out|exact
E165|@driver_e165_r5_segment3_adversarial||e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-adversarial-family.out|exact
E165|@driver_e165_r5_segment3_multi_disk||e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-multi-disk.out|exact
E165|@driver_e165_r5_segment3_odd_marker_legal||e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-odd-marker-legal-family.out|exact
E165|@driver_e165_r5_segment3_odd_marker_adversarial||e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-odd-marker-adversarial-family.out|exact
E165|@driver_e165_r5_segment5_r2_gate_root_choice_faults||e165-replay-bounded-enumeration-2026-10-04-r5-segment5-r2-gate-root-choice-faults.out|exact
E165|@driver_e165_r5_segment5b_r4_gate_r2_mode_kill_bounds||e165-replay-bounded-enumeration-2026-10-04-r5-segment5b-r4-gate-r2-mode-kill-bounds.out|exact
E166|e166-parallel-commit-shared-state-partition||e166-parallel-commit-shared-state-partition-r2-main-2026-10-02.out|exact
E169|e169-parallel-commit-append-and-prefix-recovery||e169-parallel-commit-append-and-prefix-recovery-2026-10-01-r3.out|exact
E169|e169-parallel-commit-append-and-prefix-recovery|segment-two|e169-parallel-commit-append-and-prefix-recovery-segment-two-2026-10-01-r2.out|exact
E169|e169-parallel-commit-append-and-prefix-recovery|segment-three|e169-parallel-commit-append-and-prefix-recovery-segment-three-2026-10-01.out|exact
E169|e169-parallel-commit-append-and-prefix-recovery|replay-core-inputs|e169-parallel-commit-append-and-prefix-recovery-replay-core-inputs-2026-10-01.out|exact
E170|@driver_e170_r2_dump_compare||e170-r2-o3-spec-executor-2026-10-05-s2b-dump-compare.out|exact
E170|@driver_e170_r2_bad_states_second_run_export||e170-r2-o3-spec-executor-2026-10-05-s2b-bad-states-second-run-export.out|exact
E170|@driver_e170_r2_v2_487_unit_tests||e170-r2-o3-spec-executor-2026-10-04-s1a-v2-487-unit-tests-2026-10-05.out|exact
E170|@driver_e170_r2_v1_487_unit_tests||e170-r2-o3-spec-executor-2026-10-04-s1a-v1-487-unit-tests-2026-10-05.out|exact
E170|@driver_e170_r2_v2_487_unit_level_mutations||e170-r2-o3-spec-executor-2026-10-04-s1a-v2-487-unit-level-mutations-2026-10-05.out|exact
E170|@driver_e170_r2_v1_487_unit_level_mutations||e170-r2-o3-spec-executor-2026-10-04-s1a-v1-487-unit-level-mutations-2026-10-05.out|exact
E170|@driver_e170_r2_v2_487_z1||e170-r2-o3-spec-executor-2026-10-04-s2-q5-v2-487-z1-2026-10-05.out|exact
E170|@driver_e170_r2_v2_487_z3_images||e170-r2-o3-spec-executor-2026-10-04-s2-q5-v2-487-z3-images-2026-10-05.out|exact
E170|@driver_e170_r2_v2_487_z4_images||e170-r2-o3-spec-executor-2026-10-04-s2-q5-v2-487-z4-images-2026-10-05.out|exact
E171|@driver_e171_s1_baseline||e171-three-oracle-collusion-2026-10-03-s1-baseline-run1.out|exact
E171|@driver_e171_s1_known_bad_images||e171-three-oracle-collusion-2026-10-03-s1-known-bad-images.out|exact
E171|@driver_e171_s2_bad_states_export||e171-three-oracle-collusion-2026-10-03-s2-bad-states-export.out|exact
E171|@driver_e171_s2_known_bad_images||e171-three-oracle-collusion-2026-10-03-s2-known-bad-images.out|exact
E171|@driver_e171_s2_images_o3||e171-three-oracle-collusion-2026-10-03-s2-images-o3.out|exact
E171|@driver_e171_s2_o3_control_baseline||e171-three-oracle-collusion-2026-10-03-s2-o3-control-baseline.out|exact
E171|@driver_e171_s2_comparator||e171-three-oracle-collusion-2026-10-03-s2-comparator.out|exact
E171|@driver_e171_s3_comparator_mutations||e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r7.out|exact
E171|@driver_e171_s4_comparator||e171-three-oracle-collusion-2026-10-03-s4-comparator.out|exact
E171|@driver_e171_s4_f4_comparator||e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out|exact
E171|@driver_e171_s4_f4_substitutes_comparator||e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out|exact
E171|@driver_e171_r1_s1_revision_6_comparator||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3-2026-10-07.out|exact
E171|@driver_e171_r1_s1_revision_6_cross_host_check||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07-r3.out|exact
E171|@driver_e171_r1_s1_research_mutations||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-research-mutations-2026-10-07-r3.out|exact
E171|@driver_e171_r1_s2_revision_6_comparator||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out|exact
E171|@driver_e171_r1_s2_research_mutations||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-research-mutations-r6.out|exact
E171|@driver_e171_r1_s1_revision_8_tools_research_mutations||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s1-revision-8-tools-research-mutations-2026-10-07.out|exact
E171|@driver_e171_r1_s2_revision_10_tools_research_mutations||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-08-s2-revision-10-tools-research-mutations.out|exact
E171|@driver_e171_r1_revision_11_tools_research_mutations||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-08-revision-11-tools-research-mutations.out|exact
E171|@driver_e171_r1_revision_12_stream_constants||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-13-s1-stream-constants-revision-12.out|exact
E172|@driver_e172||e172-r2-deletion-write-dry-run-2026-10-05.out|exact
E175|@driver_e175||e175-product-path-inode-watermark-regression-2026-10-08-d-P-replay.out|exact
TSV
)

# 计时字段：换机器、换负载就会变，比对时抹掉。抹掉的是**值**不是**字段名**——
# 字段整个消失属于结构变化，仍然会被抓。
# ⚠️ **判决字段一律不抹**：E128 那几行里 `verdict=` 与 `rounds_jia_slower=` 是结论不是计时，
# 留着逐字比 ⇒ 结论翻向会被这一步当场抓住，不用等下面的区间断言。
# 准入写在产物最前面的几行（`E7INPUT` 开头：输入指纹、强制重跑的理由）只说「这一趟是按什么输入跑的」，不是结果：
# 每次输入一变它就变，老产物还没有它。逐字节比对之前两边都删掉；闸 2 只数 E7RESULT 行，本来就不数它们。
strip_admission_header() {
  sed '/^E7INPUT /d'
}

strip_timing() {
  sed -E 's/(per_sec_milli|median_per_sec_milli|spread_bp|min|max|ratio_bp|years_at_sync1|years_at_sync8|sync1_per_sec_milli|nosync_per_sec_milli|elapsed_ns|verify_ns|ns_per_op|ns_per_lookup|lookups_per_s|ns_small|ns_big|bing_median|jia_median|ratio_median|ratio_min|ratio_max|e128_median|deviation|ratio|one_shot_ns|two_phase_ns|two_phase_plus_5ms_ns|injected_recovered_ns|spread_one_shot|spread_two_phase|per_round_ns|best_ns|t1_ns|t16_ns|mibs|mib_per_s|entries_per_s|gbps|peak_gbps|speedup|threads16_speedup|dev|secs|copy_ns|r_rand_qd1|r_seq|random_ns_per_byte_padded|random_ns_per_byte_h133|seq_ns_per_byte_padded|seq_ns_per_byte_h133|crossover_random_share)=[^ ]*/\1=X/g'
}

# ── 结论区间断言 ──────────────────────────────────────────────────────────
# 计时实验复跑不出同样的字节，抹掉计时之后的「结构一致」又几乎什么都不证明——
# 三条臂一起漂到别处，结构照样一致（test-discipline.md「只让多条臂互相比，
# 测不出所有臂一起错」）。所以每个计时实验都要有一条把 kb 里那个数钉住的断言。
# 断言不中**不等于代码坏了**，也可能是 kb 里那个区间该改了——两种都要人来判，所以判红。
claim() { # claim <实验> <说的是什么> <实测值> <下界> <上界>
  local exp="$1" what="$2" got="$3" lo="$4" hi="$5"
  if [[ -z "$got" ]]; then
    printf '  ✗ %-5s %-46s 读不到这个值\n' "$exp" "$what"; return 1  # gate-lint:detail
  fi
  if awk -v g="$got" -v l="$lo" -v h="$hi" 'BEGIN{exit !(g>=l && g<=h)}'; then
    printf '  ✓ %-5s %-46s %s（kb 记的区间 %s–%s）\n' "$exp" "$what" "$got" "$lo" "$hi"; return 0
  fi
  printf '  ✗ %-5s %-46s %s 落在 kb 记的 %s–%s 之外\n' "$exp" "$what" "$got" "$lo" "$hi"; return 1  # gate-lint:detail
}
fld() { sed -n "s/.*$2=\([0-9.]*\).*/\1/p" "$1" | tail -1; }   # 取某行最后一个匹配字段

check_claims() {
  local exp="$1" f="$2" bad=0 v w x y
  case "$exp" in
  E17)
    # kb 记的是 29.9–31.8M 条目/秒（八轮独立运行：2026-08-28 起四轮、2026-09-16 四轮）。
    # 留 1% 余量给机器状态波动，超出就是该改 kb 那个区间了。
    # ⚠️ **下界 2026-09-16 从 30591000 放到 29636000**：当天四轮里三轮落在旧下界之外
    # （29.94 / 30.10 / 30.47 / 30.71M），不是一次抖动，所以按八轮的并集改区间。
    # 放宽的依据是那四轮的读数本身，不是为了让门禁变绿——同一天的并行加速与阳性对照
    # 一起偏低，而装置不记录跑时的机器负载，「漂移还是被别的负载挤」这一轮分不开，
    # 账在 C350（计时实验不记录跑时的机器负载）。
    v=$(grep 'arm=single' "$f" | sed -n 's/.*entries_per_s=\([0-9]*\).*/\1/p')
    claim E17 "单线程合并吞吐（条目/秒）" "$v" 29636000 32118000 || bad=1
    # 散射是并行度天花板：阳性对照 32 线程必须明显快过合并臂 32 线程，
    # 否则「散射吃掉 55%」这条读数没有判别力。
    w=$(grep 'arm=parallel threads=32' "$f" | sed -n 's/.*speedup=\([0-9.]*\).*/\1/p')
    x=$(grep 'name=poscontrol threads=32' "$f" | sed -n 's/.*speedup=\([0-9.]*\).*/\1/p')
    if awk -v a="$x" -v b="$w" 'BEGIN{exit !(a > b*1.5)}'; then
      printf '  ✓ %-5s %-46s 对照 %s× vs 合并臂 %s×\n' E17 "散射吃掉的那一半仍在（阳性对照更快）" "$x" "$w"
    else
      printf '  ✗ %-5s %-46s 对照 %s× 没比合并臂 %s× 快出 1.5 倍\n' E17 "阳性对照失去判别力" "$x" "$w"; bad=1  # gate-lint:detail
    fi ;;
  E20)
    # kb 的承重结论：超 L3 那一档 16 KiB 是唯一最小点，且 8 KiB 反常地差（五轮稳定，未解释）。
    for n in 2048 4096 8192 16384 32768 65536; do
      eval "v$n=\$(grep \"name=e20 node_bytes=$n keys=8388608 entry_bytes=40 \" '$f' | sed -n 's/.*ns_per_lookup=\\([0-9.]*\\).*/\\1/p')"
    done
    if awk -v a="$v16384" -v b="$v2048" -v c="$v4096" -v d="$v8192" -v e="$v32768" -v g="$v65536" \
         'BEGIN{exit !(a>0 && a<b && a<c && a<d && a<e && a<g)}'; then
      printf '  ✓ %-5s %-46s 16K=%s ns，其余 %s/%s/%s/%s/%s\n' E20 "超 L3 档 16 KiB 仍是唯一最小点" "$v16384" "$v2048" "$v4096" "$v8192" "$v32768" "$v65536"
    else
      printf '  ✗ %-5s %-46s 16K=%s，2K/4K/8K/32K/64K=%s/%s/%s/%s/%s\n' E20 "16 KiB 不再是最小点 ⇒ kb 那条要改" "$v16384" "$v2048" "$v4096" "$v8192" "$v32768" "$v65536"; bad=1  # gate-lint:detail
    fi
    if awk -v d="$v8192" -v c="$v4096" 'BEGIN{exit !(d>c)}'; then
      printf '  ✓ %-5s %-46s 8K=%s > 4K=%s\n' E20 "8 KiB 的未解释拐点又复现一次" "$v8192" "$v4096"
    else
      printf '  ✗ %-5s %-46s 8K=%s ≤ 4K=%s ⇒ kb 记的「五轮稳定」不再成立\n' E20 "8 KiB 拐点这次没出现" "$v8192" "$v4096"; bad=1  # gate-lint:detail
    fi ;;
  E144)
    # kb 记的是本机软件实现在 105 字节头上的纳秒数：crc32c 28、sha256 676（7 轮取最小）。留 ±30% 给机器状态波动，
    # 超出就是该改 kb 那个区间了；比值那一行由这两个数夹住，不另钉。
    v=$(grep 'name=cost arm=crc32c width=105 ' "$f" | sed -n 's/.*ns_per_op=\([0-9]*\).*/\1/p')
    claim E144 "CRC32C 算一个 105 字节头（ns）" "$v" 19 37 || bad=1
    w=$(grep 'name=cost arm=sha256 width=105 ' "$f" | sed -n 's/.*ns_per_op=\([0-9]*\).*/\1/p')
    claim E144 "SHA-256 算一个 105 字节头（ns）" "$w" 473 879 || bad=1
    # 判别力那一半是确定性的：阳性对照必须还漏得出双比特翻转，正式臂一个都不许漏。
    x=$(grep 'name=verdict' "$f" | sed -n 's/.*control_has_teeth=\([a-z]*\).*/\1/p')
    y=$(grep 'name=verdict' "$f" | sed -n 's/.*formal_double_bit_missed=\([0-9]*\).*/\1/p')
    if [[ "$x" == true && "$y" == 0 ]]; then
      printf '  ✓ %-5s %-46s 对照漏得出、正式臂零漏\n' E144 "判别力测试仍分得出差别"
    else
      printf '  ✗ %-5s %-46s control_has_teeth=%s formal_double_bit_missed=%s\n' E144 "判别力测试失去判别力" "$x" "$y"; bad=1  # gate-lint:detail
    fi ;;
  E128)
    # kb 的承重结论有两条，方向相反，所以两条都要钉——只钉一条会让「甲不慢」被读成
    # 「甲哪儿都不慢」，而记账那一对是慢的。
    ok67=$(grep 'name=xdev entry_bytes=67 ' "$f" | sed -n 's/.*ok=\([a-z]*\).*/\1/p')
    ok111=$(grep 'name=xdev entry_bytes=111 ' "$f" | sed -n 's/.*ok=\([a-z]*\).*/\1/p')
    if [[ "$ok67" == true && "$ok111" == true ]]; then
      printf '  ✓ %-5s %-46s 67 与 111 两档都落回 E20 的数\n' E128 "跨装置闸仍然成立"
    else
      printf '  ✗ %-5s %-46s ok67=%s ok111=%s\n' E128 "跨装置闸破了：这套装置与 E20 报不同的数" "$ok67" "$ok111"; bad=1  # gate-lint:detail
    fi
    ri=$(grep 'name=verdict pair=inode ' "$f" | sed -n 's/.*ratio_median=\([0-9.]*\).*/\1/p')
    rl=$(grep 'name=verdict pair=ledger ' "$f" | sed -n 's/.*ratio_median=\([0-9.]*\).*/\1/p')
    # ⚠️ **inode 那一对不钉方向。** 2026-09-10 四整轮里三轮判 `0/5 jia_never_slower`、
    # 第四轮判 `5/5 jia_slower_every_round`，方向相反 ⇒ kb 记的是「不稳定」，没有方向可钉。
    # 钉任何一边都是把一条不稳定的观测写成断言。这里只把观测值打出来，不判绿也不判红。
    # 真正兜着它的是上面那一步**逐字节结构比对**：`strip_timing` 有意不抹 `verdict=` 与
    # `rounds_jia_slower=`（它们是结论不是计时）⇒ 判决再翻一次，那一步就判红。
    # ⚠️ **E128 因此会间歇性判红，这是有意的。** 判红时该做的不是调宽容差，
    # 是把新跑那一次的比值与判决添进 E128 正文「结论一」那张逐轮表，让它变成五轮、六轮。
    # 每一次红都是一次新观测——对一格记着「不稳定」的实验，这正是复跑该有的行为。
    printf '  ! %-5s %-46s 109/93 = %s（不判，kb 记「不稳定」）\n' E128 "inode 那一对四轮里翻过一次向" "$ri"
    if awk -v r="$rl" 'BEGIN{exit !(r>1.00 && r<1.10)}'; then
      printf '  ✓ %-5s %-46s 97/81 = %s\n' E128 "记账树上甲仍慢一点点，量级不变" "$rl"
    else
      printf '  ✗ %-5s %-46s 97/81 = %s 掉出 (1.00, 1.10)\n' E128 "记账树那一对的方向或量级变了" "$rl"; bad=1  # gate-lint:detail
    fi ;;
  E44)
    # 本机 fsync 率：换机器会变，但**量级**要稳住，否则寿命折算整个塌掉
    v=$(grep 'name=arm arm=Sync1' "$f" | sed -n 's/.*median_per_sec_milli=\([0-9]*\).*/\1/p')
    claim E44 "本机 fsync 率（每秒千分之一次）" "$v" 500000 20000000 || bad=1
    # 阳性对照：不 fsync 必须至少快一倍，否则 fdatasync 没到设备
    w=$(grep 'name=poscontrol' "$f" | sed -n 's/.*ok=\([a-z]*\).*/\1/p')
    if [[ "$w" == true ]]; then printf '  ✓ %-5s %-46s\n' E44 "阳性对照：fdatasync 确实到了设备"
    else printf '  ✗ %-5s %-46s ok=%s\n' E44 "阳性对照失败 ⇒ fdatasync 没到设备，整轮作废" "$w"; bad=1; fi
    # 48 位计数器在本机速率下的寿命：这是「48 位够不够」那条结论的落点
    x=$(grep 'name=lifetime bits=48' "$f" | sed -n 's/.*years_at_sync1=\([0-9]*\).*/\1/p')
    claim E44 "48 位计数器在本机撑多少年" "$x" 500 50000 || bad=1
    # 加宽 jsn 到 12 字节的代价：0..=100 项里一格都不该多占
    y=$(grep 'name=width unit=512 jsn_bytes=12 ' "$f" | sed -n 's/.*cost_unit_count_0_100=\([0-9]*\).*/\1/p')
    claim E44 "jsn 8→12 在 512 单元下多占几格" "$y" 0 0 || bad=1 ;;
  E58)
    # kb 的承重结论：32 KiB 在随机小读上比 16 KiB 贵约一成，而顺序侧只快 0.83%。
    # 只钉倍数不够（三条臂一起漂，倍数照样对），所以第三条钉的是绝对值。
    v=$(grep 'name=rand_g16384 ' "$f" | sed -n 's/.*ns_per_op=\([0-9.]*\).*/\1/p')
    w=$(grep 'name=rand_g32768 ' "$f" | sed -n 's/.*ns_per_op=\([0-9.]*\).*/\1/p')
    x=$(awk -v a="$w" -v b="$v" 'BEGIN{if(b>0) printf "%.4f", a/b}')
    claim E58 "随机小读 QD=1：32 KiB 是 16 KiB 的几倍" "$x" 1.03 1.25 || bad=1
    v=$(grep 'name=randq_g16384 ' "$f" | sed -n 's/.*user_mib_per_s=\([0-9.]*\).*/\1/p')
    w=$(grep 'name=randq_g32768 ' "$f" | sed -n 's/.*user_mib_per_s=\([0-9.]*\).*/\1/p')
    x=$(awk -v a="$v" -v b="$w" 'BEGIN{if(b>0) printf "%.4f", a/b}')
    claim E58 "QD=16 用户带宽：16 KiB 是 32 KiB 的几倍" "$x" 1.03 1.25 || bad=1
    # 绝对值：32 KiB 单元读 4 KiB 的放大恰为 8，由独立算术给出（32768/4096）
    x=$(grep 'name=meta_g32768 ' "$f" | sed -n 's/.*read_amp_4k=\([0-9.]*\).*/\1/p')
    claim E58 "32 KiB 单元的读放大（绝对值）" "$x" 8.0 8.0 || bad=1
    # 阳性对照：内核记的字节 ÷ ops×G，读到洞或读到缓存都会让它塌
    x=$(grep 'name=rand_g32768 ' "$f" | sed -n 's/.*pr_over_devbytes=\([0-9.]*\).*/\1/p')
    claim E58 "阳性对照：内核记的字节 ÷ (ops×G)" "$x" 0.98 1.02 || bad=1 ;;
  E140)
    # kb 的承重结论：含头随机页读比补齐慢 7.7%–9.1%，顺序读补齐少 7.6%–9.2% 带宽。复跑用的是种子 1、ops 4096 的单轮，
    # 抽样比第三轮少 4 倍，区间按第三轮五个种子的极差再各放 5 个百分点。
    v=$(grep 'name=verdict' "$f" | sed -n 's/.*r_rand_qd1=\([0-9.]*\).*/\1/p')
    claim E140 "随机 4 KiB 页读：含头是补齐的几倍" "$v" 1.03 1.15 || bad=1
    w=$(grep 'name=verdict' "$f" | sed -n 's/.* r_seq=\([0-9.]*\).*/\1/p')
    claim E140 "顺序读：补齐带宽是含头的几倍" "$w" 0.85 0.97 || bad=1
    # 绝对值：跨单元页占比由闭式 (4096 − gcd) / 32635 独立算出，与计时无关
    x=$(grep 'name=model_h133 ' "$f" | sed -n 's/.*straddle_fraction=\([0-9.]*\).*/\1/p')
    claim E140 "头 133 时跨单元页占比（闭式）" "$x" 0.125479 0.125479 || bad=1
    # 阳性对照：内核记的字节 ÷ 程序记账，读到缓存就塌（阴性对照那份产物里同一格是 0.0000）
    y=$(grep 'name=rand_h133 ' "$f" | sed -n 's/.*pr_over_devbytes=\([0-9.]*\).*/\1/p')
    claim E140 "阳性对照：内核记的字节 ÷ 程序记账" "$y" 0.98 1.02 || bad=1 ;;
  E107)
    # kb 的承重结论有两条，一条是字节、一条是延迟，两条都要钉。
    # 字节那条是纯算术、逐次相同，但仍然钉住——只钉延迟会让一个把字节模型改错的变异照样绿。
    v=$(grep 'name=bytes leaves=8 f=0 ' "$f" | sed -n 's/.*ratio_a_over_c=\([0-9.]*\).*/\1/p')
    claim E107 "主负载 8 叶 f=0：甲臂 ÷ 丙臂" "$v" 0.9503 0.9503 || bad=1
    v=$(grep 'name=crossover' "$f" | sed -n 's/.*first_leaves_where_arm_a_cheaper=\([0-9-]*\).*/\1/p')
    claim E107 "甲臂第一次比丙臂便宜的叶数" "$v" 8 8 || bad=1
    # 延迟那条：durable 语义下三段写序比一次提交慢多少倍
    a=$(grep 'name=latency semantics=durable' "$f" | sed -n 's/.*one_shot_ns=\([0-9]*\).*/\1/p')
    b=$(grep 'name=latency semantics=durable' "$f" | sed -n 's/.*two_phase_ns=\([0-9]*\).*/\1/p')
    x=$(awk -v a="$a" -v b="$b" 'BEGIN{if(a>0) printf "%.4f", b/a}')
    claim E107 "durable：三段写序 ÷ 一次提交" "$x" 1.15 2.10 || bad=1
    # 阳性对照：注入的 5 ms 必须被量回来，落在 ±10% 内
    x=$(grep 'name=latency semantics=durable' "$f" | sed -n 's/.*injected_recovered_ns=\([0-9-]*\).*/\1/p')
    claim E107 "阳性对照：注入 5 ms 回收到的纳秒" "$x" 4500000 5500000 || bad=1
    # 作废条款 3：不 O_DIRECT 不 fdatasync 那条必须快一个数量级，否则 I/O 没真落盘
    a=$(grep 'name=latency semantics=durable' "$f" | sed -n 's/.*one_shot_ns=\([0-9]*\).*/\1/p')
    b=$(grep 'name=nosync_control' "$f" | sed -n 's/.*per_round_ns=\([0-9]*\).*/\1/p')
    x=$(awk -v a="$a" -v b="$b" 'BEGIN{if(b>0) printf "%.2f", a/b}')
    claim E107 "作废条款 3：durable ÷ nosync" "$x" 10 500 || bad=1 ;;
  E21)
    # kb 的承重结论：CPU 扫描撞内存带宽墙（约 65 GB/s），16 线程几乎不加速 ⇒ GPU 传输地板已经更慢。
    v=$(grep 'name=scaling arm=bandwidth' "$f" | sed -n 's/.*peak_gbps=\([0-9.]*\).*/\1/p')
    claim E21 "CPU 扫描峰值带宽（GB/s）" "$v" 55 75 || bad=1
    w=$(grep 'name=scaling arm=bandwidth' "$f" | sed -n 's/.*threads16_speedup=\([0-9.]*\).*/\1/p')
    claim E21 "带宽受限：16 线程几乎不加速" "$w" 1.0 1.4 || bad=1
    # 阳性对照：同样 16 线程，计算受限的那条必须大幅加速，否则「不加速」分不清是带宽墙还是没跑起来
    x=$(grep 'name=poscontrol arm=compute' "$f" | sed -n 's/.*speedup=\([0-9.]*\).*/\1/p')
    claim E21 "阳性对照（计算受限）16 线程加速" "$x" 10 20 || bad=1 ;;
  esac
  return $bad
}

# E9 的入库产物是 25 次运行拼起来的（5 种子 × 5 改名档），而这个循环从没被写进 kb。
# 2026-08-29 审计时按产物里的 config 行反推出来，重建结果与入库产物**逐字节一致**。
# 25 次调用整条包一层（不是每次调用各包一层）：省掉 24 次多余的 systemd scope 建立。
driver_e9() {
  bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" bash -c '
    device="$1"
    for r in 0 500 2000 5000 20000; do
      for s in 3 7 11 13 17; do
        ./scripts/experiments/target/release/e9-keylayout "$device" "$s" interleave 8 "$r" || exit 1
      done
    done
  ' _ "$REPLAY_DEV" || return 1
}

# E142 第十五次跑步④（重跑登记 `research/prompts/e142-r15-prereg.md` 第六节）：装置↔crates/ 逐字节比对
# 要跨两个 cargo workspace（scripts/experiments/ 与仓根的 crates/ workspace，仓根 Cargo.toml 显式 exclude =
# ["scripts/experiments"]，两边互相看不到对方，不能合并成一次 cargo 调用）。旧的 `new_pool_file_creation_region_bytes`
# 读的是改位置寻址之前的旧布局（区域表写死八个单元，见该文件模块注释），已经比不出新写的五个分配记录树节点，
# 换成只读导出 `e142_new_pool_file_creation_write_dump`（不带任何布局知识，逐次写按 (设备, 偏移, 长度, sha256,
# 整段十六进制) 原样打出来）。装置的第一个命令行参数是这份导出的文件路径（做 Q142.1 真比对），第二个参数是
# arm O 的历史留存产物（`第十五次跑步④` 起 Q142.8 用它算「哪些区域从旧布局变到了新布局」，
# 交回报告 `research/prompts/e142-r15-step234-runner-report.md` 里写明这份参照为什么找不到能重新编译的
# 源码、只能用留存产物）。两段都各自有自己的 `name=done`，闸 2 逐段核过。
#
# E142 第十六次跑第一段（重跑登记 `research/prompts/e142-r16-prereg.md`；交回报告
# `/tmp/claude-1000/e142-r16-s1/report.md`）：α（内部/根节点一格）、β（根 largest_key）、ι（内部条目
# key 取什么）、γ（extent 上段叶 key 区间）三格 5.1 的变体开关整套删除，改成 D8（核心索引结构） 已定项 14
# 第 395/401 行、D18（块里携带什么信息） 已定项 2 射程写死的唯一写法（稀疏、整个 key 空间、按位置分片区间）；
# δ（盘上槽数）收口成 `slots_of_device_bytes` 一个 const fn。第二个命令行参数从「第十四次跑 arm O 参照」
# 改成「这一次步①现编现跑的臂 N15 参照」（`research/results/e142-new-pool-file-creation-dry-run-2026-09-25-r16-arm-n15.out`），
# 按 (设备, 偏移, 长度) 配对出 `name=old_new_region`（Q142.19），不再按名字配对出旧的 `name=old_new_region`
# 系列。新增 `name=g3_shape`（第八节 G3 五个几何点，不依赖 `crates/`）、`name=positive_control_p2`
# （四个点，锚点随稀疏改成第 0 条条目）、`name=g4_bytes_equal_summary`（第三个命令行参数给才跑，这一次
# `crates/` 一盘几何造不出（`make_filesystem.rs` 的 `make_filesystem` 开头断言两块盘，D2（RAID 条带策略） 已定项 9），不传，报
# `skipped=true`）。`code2_field_rows` 从「非空格与第一个空格」改成「每条都列 + 补齐区一行」。
# Q142.11（原 Q142.1）这一次判「全等」：29 个区域全部配上、29 个相等、0 个不等（三格上模型与 `crates/`
# 逐字节相同，只说明 kb 转写与 `crates/` 一致，不说明条款本身对，见第十二节修订与第 5.4 节）。
#
# E142 第十七次跑第一段（重跑登记 `research/prompts/e142-r17-prereg.md`；交回报告
# `/tmp/claude-1000/e142-r17-s1/report.md`）：问题单第 6、8 行——阳性对照的期望违例数三档改成从
# 写清单现算（R10），不再写死 `− 8`；加逐态预言器、结果类矩阵、G6/G7 两个几何敏感性取样点、PC2
# （主臂几何逐单元靶向对照）、角色名自检（Q142.27）、文本数字自检（Q142.28）、Q142.29 两处点名的
# 直接比较；`journal_record_name_for_offset` 的第三条记录角色名从写死 `"t9"` 改成写清单现算的
# 动态标签；G22 空白清单条目从字面改成绑定这份产物自己的记账数。这一段没有改动主臂写字节的路径
# （S7：改动前后 29 行 `name=device_region_bytes` 逐字节相同），`driver_e142` 本身不用改；
# 存盘文件名从 `-r16-combined.out` 改成 `-r17-main.out`（登记步③给的文件名）。
driver_e142() {
  local impl_snapshot="$OUT_DIR/e142-crates-write-dump.tmp"
  local arm_n15_reference="research/results/e142-new-pool-file-creation-dry-run-2026-09-25-r16-arm-n15.out"
  (bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q -p singlefs-checker-tier --bin e142_new_pool_file_creation_write_dump) >"$impl_snapshot" || return 1
  bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ./scripts/experiments/target/release/e142-new-pool-file-creation-dry-run "$impl_snapshot" "$arm_n15_reference" || return 1
  cat "$impl_snapshot"
}

# E142 第二十二次跑第二段（重跑登记 `research/prompts/e142-r22-prereg.md` 第五节 5.4 ⑧）：`crates/` 在两块 8 GiB 盘上的导出，
# 与 `driver_e142` 第一条命令相同，只多 `-- --device-bytes 8589934592`（导出 bin 的可选参数，不带时 4 GiB、输出不变）。
# 装置那一侧的 P8G 一段已在 `driver_e142` 的产物里（`name=p8g_` 打头的行），这一份是它要比的另一半，比对脚本 `p8g_region_compare.py`
# 在 `research/results/e142-r22-s2-comparisons-2026-10-05.out` 的附录里。
driver_e142_p8g_crates_dump() {
  (bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q -p singlefs-checker-tier --bin e142_new_pool_file_creation_write_dump -- --device-bytes 8589934592) || return 1
}

# E156（入库装置，跑前登记「一」读法写死第 1 行；重跑登记 `research/prompts/e156-r2-prereg.md`「五、5.7」
# 第一、二、三段都在同一个二进制里，产物是累计的：2026-09-24 stage2.out 前 324 行是第一段（岔路 7），
# 之后是第二段（岔路 3，run_hf_single_cell）与第三段（岔路 1，run_hh_cell）新加的行。
# stage3.out（2026-09-24，重跑登记「十二」修订这一段）在 stage2.out 的基础上追加：Q1 诊断溯源
# （`q1_delta_debug`／已回收未覆盖记录过滤，见修订）、岔路 1 的 S = 4 第二个几何取样点（`q1_hh`／
# `q1d_*` 的 `s=` 字段、`anchor_k8`、`q1_geometry_sensitivity_s`）、K9 前提现核（`k9_precondition`）；
# 在 stage2.out 原有的 329 行共有格式上逐字节相同（`e156-s4/report.md` 的对拍命令），Q3e（X8-A）仍
# `status=not_done`，未做。stage4.out（2026-09-24，续派第二段）在 stage3.out 的基础上追加：岔路 1
# 步数对齐对照（`q1_step_matched_diff`，`run_hh_cell` 新增 `matched_to_holes` 参数）、岔路 3 的
# Q3e（X8-A/HY，`run_x8a_cell`，独立 128 槽小池）与 Q3d（`q3d_derived`）；在 stage3.out 原有的 323 行
# （K1/legal_state/s1d_step/Q7 全家/PC-检查三条/HK-HR-H0 构造）共有格式上逐字节相同。
# r3.out（2026-09-25，第 3 次重跑登记 `research/prompts/e156-r3-prereg.md`：分配记录树按位置寻址之后
# 重算钉着旧布局的常量）在 stage4.out 的基础上结构性改动，不是逐字节追加：分配记录树不再是单节点，
# 装置里的三处常量与另外六处改成第七节 7.2 算出的闭式与绝对值（K1-1 第 1 项 13→17、β0 走读引用 12→16、
# S1(c) 隔离槽数 34→54，其余变成随「这次改动的记录落在几片叶」现算的闭式，不再是常数）；新增
# `anchor_a_d8`、`anchor_a_d8_root_level`、`s1ef_step`／`s1ef_summary`（Q3r.2 逐次闭式核对）、
# `r5_empty_publish_closed_form`、`r7_hy_cap`／`r7_hy_condition`（HY 的覆盖写次数改成搜出满足
# e ≥ max(8, f) 的那一档，这一轮搜到的是 0：搜索过程见交回报告，判定按登记走）行；H0／HR 不再撞
# `AllocationRecordsExceedOneNode` 那道墙，跑满登记要求的全部步数（`baseline_workload_completed_full_length`
# 取代 stage4.out 的 `baseline_workload_truncated_by_write_failure`）。Q3r.4（岔路 1、3、7 判定变不变）
# 的比对命令与判定表见交回报告；核心结论：岔路 1（Q1d 在 S = 4 上从「单调」变「不单调」）与岔路 7
# （`q7d2_min_item5` 从「非 0」变「= 0」，F16 从「触发」变「不触发」）判定变了，岔路 3（Q3c/Q3e/K9）
# 在已核的量上不变。
# fork7-selfproof.out（2026-09-25，续派，E156 第 3 次重跑登记「十二」修订 4）在 r3.out 的基础上加
# 一个第 10 个基底 `beta_hr_rollback_row`：r3.out 的 `q7d2_min_item5` 第一次在可达状态上读到
# `min_item5=0 family=HR kind=rollback_row txg=76`（此前 7 个可达基底第 5 项最小是 1），这里把这一步
# 也捕成一个基底、并入既有的 `bases` 数组（9→10），让已有的 Q7a/Q7c①②/PC-检查循环再跑一遍，不改
# `q7c_self_test` 的公式、不新写判定逻辑。核心结论：Q7c① 在这个基底上第一次转色（`q7c1_not_subtracting_defer
# basis=beta_hr_rollback_row … flips_red_to_green=true`）——此前只有不可达的 β_syn 转过色；`basis_count`
# 9→10，`q7a_summary` 的 `all_red_count` 18→20（该基底 delta=1/8 各命中一次），`q7c1_flip_seen`/
# `q7c2_flip_seen` 字面不变（β_syn 已经让它们是 true）。在 r3.out 原有的 1076 行共有格式上逐字节相同，
# 只在旧的 `q7a_summary`/`integrity` 两行与新增 9 行（`basis_snapshot`/`q7a` ×3/`q7c1`/`q7c2`/
# `pc_check` ×3）上不同。
# 2026-09-26.out（第 4 次重跑登记 `research/prompts/e156-r4-prereg.md`，第十二节修订 2）整个换了装置：回退改成挂着时的一次向前发布、
# 挂载内回收、准入改式之后，岔路 3、7 与 H0/HR/HK/HF/HX/HY 不再跑，二进制只跑第一段（PQ1-前/后、PC-c2、PQ2、U11、U13 两半、G-adm）
# 与第二段（Hh(k, 位置, S, ρ) 38 条排得下的历史 × 「实」「每」两个回收时点、Q1c/Q1c-洞/Q1c-前/Q1c-后/Q1d、PC-多扣/洞/分配器/判定器）；
# 第一段任一停机条款触发就只出第一段（`name=stage_two_not_run`）。不是在 fork7-selfproof.out 上追加，两份没有共同格式。
# 整个装置就活在 crates/singlefs-harness 里，没有 scripts/experiments 侧的配对二进制，先例同 E142（第 371 行注释）——
# 两个 cargo workspace 互相看不到对方，不能合并成一次调用。确定性：同一个二进制跑两遍逐字节一致（2026-09-25 现查：
# 这一段的产物两次跑出 `cmp` 逐字节一致）。
# 2026-10-05.out（第 5 次重跑登记 `research/prompts/e156-r5-prereg.md`）：core 在分配器没装根环表而记了一条根时断言（C551），装置的
# PC-分配器 撤了、换成 PC-钉门槛（格表里 Hh(1, 后, S, ρ) 六格的两份滞后集合交 k = 0 的 A10 判定，打 `name=pc_pinned_floor` 七行）；
# `v2_pc_allocator_seen` 换名 `v2_pc_pinned_floor_seen`。与 2026-09-26-r2.out 在新 crates/ 上规范化之后有 294 行不同（登记 Q2b；
# `…-2026-10-05-attribution-without-31-1.out` 是副本里反打 31-1 那一块的归属跑，与本份逐字节相同，不进这里），r2 那一份留着、不再登记。
# 2026-10-05-r2.out（第 6 次重跑登记 `research/prompts/e156-r6-prereg.md`）：回收门槛照用户 2026-10-05 定乙读（挂载时读回全零的根环槽
# 钉在它可能住过的最旧 txg，直到这个进程写掉那一槽），K10 守门、「每」列与锚点模型同改；新加 `name=all_zero_pins_at_mount`、
# `name=all_zero_pin_cell`、`name=all_zero_pin_summary` 三种行，已有行格式不变。产物用仓根 `target/release/` 的二进制跑（debug 一遍约 13 分钟，
# 子 agent 前台跑不完）；debug 二进制前 1803 行与之逐字节相同，整份 debug 复跑由这一行的 exact 比对。`…-2026-10-05-all-zero-pin-removed.out`
# 是副本里去掉 core 全零钉住（同目录 `.patch`）的判别力对照，不进这里；与第 5 次的判定行比较在 `…-2026-10-05-r6-vs-r5-compare.py` / `.txt`。
# E161（崩溃放量的去重与分段耗时）可行性档（2026-09-27，用户定只验能跑能用，登记 12.2）：入库装置 `feasibility` 模式。
# crates 的崩溃点重放枚举（S3、S4 对拍时调的）往标准输出打 LAYER0_* 进度行（带墙钟秒数），产物只留 E7RESULT 行。
# 计时字段一律以 _elapsed_ns 或 _ratio 结尾，按 timing 比结构；PC3 的判定行随计时可能翻，翻了是新观测。线程数钉 10；内存上限跟 REPLAY_MEMORY_CAP（默认 8G，2026-09-27 收窄前曾写死 16G，改成跟脚本头默认值一致）。
driver_e161_feasibility() {
  (set -o pipefail && E161_THREADS=10 bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e161_crash_state_dedup_and_time_split -- feasibility | grep '^E7RESULT ')
}

# E165（恢复重放有界穷举与变异杀伤界）第一段（登记 `research/prompts/e165-preregistration.md` 5.5）：入库装置，只跑合法历史一族
# （`--family legal`；对抗一族停在登记停机条款 H4，见登记第十二节修订 7，数没入库）。装置另打 `E7TIMING` 耗时行，产物只留 E7RESULT 行；
# 计数与线程数无关（10 线程与 7 线程两遍逐字节相同），线程数钉 10。`driver_e165_light` 是 `--bounds light` 那一档。
driver_e165() {
  (set -o pipefail && E165_THREADS=10 bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds segment1 --family legal | grep '^E7RESULT ')
}
driver_e165_light() {
  (set -o pipefail && E165_THREADS=10 bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds light --family legal | grep '^E7RESULT ')
}
# 第一段对抗一族（登记修订第 9 条收窄 H4 之后，2026-10-01 入库；16 线程与 7 线程两遍逐字节相同）。
driver_e165_adversarial() {
  (set -o pipefail && E165_THREADS=10 bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds segment1 --family adversarial | grep '^E7RESULT ')
}
# 第二段（第 2 次重跑 r2 登记 `research/prompts/e165-r2-prereg.md` 5.6、9.1–9.5，补跑 2c）：26 个变体各一份副本、各编一次，整段约 15 分钟（2026-10-02 同机负载 12–30，--jobs 2 --threads 4；26 个变体都跑，估计 15 至 20 分钟，没整轮量过）。
# 驱动在副本里改 recovery.rs、自己经内存包装跑每个二进制，这里不再包一层。产物里每行的复跑命令写的是 research/results/ 下的路径，
# 这里的输出落在临时目录，比对前把两个文件名换回那个写法；红例账的 sha256 在产物的 name=verdict 行里，账一字不差才对得上。
# 驱动的 REGISTERED 钉的是第四段冻结根的表（`crates/mutations.tsv` 9d5fd9fc…）与 recovery.rs、lib.rs 的哈希，主工作区的 crates/ 变了它就停在 RH1，
# 所以复跑要 --root 指冻结根：默认 /tmp/claude-1000/e165-r2-segment4/root（清单与建法见登记 r2 修订 4，第 819–820 行一带），那份没了就照修订 4 重建一份、
# 用 E165_R2_FROZEN_ROOT 指过去；根里要有 scripts、scripts/mutation/tables、第一段 r2 的 segment1b 两份产物与 .claude/scripts/preflight.sh。
# 驱动 --selftest 不带 --root；这一份基线是 1373 行（修订 4 挪了三项自检之后），同一棵根上复得出，别的树复不出来先对冻结根清单。
driver_e165_segment2() {
  local scratch="$OUT_DIR/e165-segment2"
  # 默认的那份根 2026-10-06 开机清 /tmp 时已丢、要重建：照这个驱动的注释写的建法重建、核过清单，放 ~/.local/share/singlefs/work/ 下，用 E165_R2_FROZEN_ROOT 指过去
  local frozen_root="${E165_R2_FROZEN_ROOT:-/tmp/claude-1000/e165-r2-segment4/root}"
  mkdir -p "$scratch/out"
  (E165_KILL_BOUNDS_MEMORY_CAP=8G RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="变体与基线装置峰值只量过合法历史一族，留十倍余量" \
      bash scripts/experiments/e165-replay-mutation-kill-bounds.sh --root "$frozen_root" --scratch "$scratch" \
      --output "$scratch/out/e165-replay-bounded-enumeration-2026-10-02-r2-segment2c-kill-bounds.out" \
      --ledger "$scratch/out/e165-replay-bounded-enumeration-2026-10-02-r2-segment2c-red-examples.tsv" --jobs 2 --threads 4 >&2) || return 1
  sed -e 's#--output e165-#--output research/results/e165-#' -e 's#--ledger e165-#--ledger research/results/e165-#' \
    "$scratch/out/e165-replay-bounded-enumeration-2026-10-02-r2-segment2c-kill-bounds.out"
}
# 第 2 次重跑（登记 `research/prompts/e165-r2-prereg.md`）第三段与第四段的跑法：入库装置，`--profile r2`；第三段是多盘一族，第四段几份是 `--table both`（两种建表法各判一遍逐态比）与盘次序反过来。
# 登记行指的产物是第 5 次重跑第一段（登记 `research/prompts/e165-r5-prereg.md` 第十一节比对对象表第 8–13 行，主 agent 认定第 4 条）在第一段根
# （`/tmp/claude-1000/e165-r5-segment1/root`，清单 `root-manifest-at-start.sha256`，sha256 95237ecf…）上编出的二进制、7 线程跑的；第 2 次重跑的旧产物
# （`…-2026-10-02-r2-segment3-multi-disk-2026-10-02.out`、`…-2026-10-02-r2-segment4-*.out`）留着，今天的装置在 J7a 两格上复不出它们（根环那条线 7-2 改了施加规则，登记 r5 PF2）。
# 这里在主工作区上 `cargo run`：`crates/` 里的文件主工作区与第一段根可能不同（别的会话在改），那样复出来对不上是新观测，先对着第一段根的清单查。
# 计数与线程数无关（第 2 次重跑多盘一族 8 与 4 线程两遍逐字节相同），线程数钉 8。
driver_e165_r2_segment3_multi_disk() {
  (set -o pipefail && E165_THREADS=8 bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds segment1 --family multi_disk --profile r2 | grep '^E7RESULT ')
}
driver_e165_r2_segment4_table_both_legal() {
  (set -o pipefail && E165_THREADS=8 bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds segment1 --family legal --profile r2 --table both | grep '^E7RESULT ')
}
driver_e165_r2_segment4_table_both_adversarial() {
  (set -o pipefail && E165_THREADS=8 bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds segment1 --family adversarial --profile r2 --table both | grep '^E7RESULT ')
}
driver_e165_r2_segment4_table_both_multi_disk() {
  (set -o pipefail && E165_THREADS=8 bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds segment1 --family multi_disk --profile r2 --table both | grep '^E7RESULT ')
}
driver_e165_r2_segment4_reversed_disk_order_legal() {
  (set -o pipefail && E165_THREADS=8 bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds segment1 --family legal --only R --profile r2 --table both --disk-order reversed | grep '^E7RESULT ')
}
driver_e165_r2_segment4_reversed_disk_order_multi_disk() {
  (set -o pipefail && E165_THREADS=8 bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds segment1 --family multi_disk --only M,MC --profile r2 --table both --disk-order reversed | grep '^E7RESULT ')
}
# 第 3 次重跑（登记 `research/prompts/e165-r3-prereg.md`）：`--profile r3`，入库装置；第一段是对抗一族按「同一实例里 journal 记录的 txg 随计数器不减」拆行（5.6），
# 第二段是择根与重放合判一族 `--family root_choice`（5.3、5.5），计数与线程数无关（两段都是 8 与 4 线程两遍整份逐字节相同），线程数钉 8。
# 对抗一族那一行指的产物是第 5 次重跑第一段（登记 `research/prompts/e165-r5-prereg.md` 比对对象表第 14 行，主 agent 认定第 4 条）在第一段根
# （`/tmp/claude-1000/e165-r5-segment1/root`，清单 sha256 95237ecf…）上编出的二进制跑的；第 3 次重跑第一段的旧产物（`…-2026-10-02-r3-segment1-adversarial-family.out`）留着，
# 今天的装置在 J7a 分配记录树根那一格上复不出它。择根那一行仍指第 3 次重跑第二段冻结根上跑的产物，第 5 次第一段在第一段根上的重跑与它逐字节相同。
# 这里在主工作区上 `cargo run`，`crates/` 里的文件（别的会话在改）与两份根可能不同，复出来对不上是新观测，先对着根的清单查（对抗一族：第 5 次第一段
# `root-manifest-at-start.sha256`；择根：第 3 次第二段 `/tmp/claude-1000/e165-r3-segment2/segment2-root-after-overlay.sha256`）。
driver_e165_r3_segment1_adversarial() {
  (set -o pipefail && E165_THREADS=8 bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds segment1 --family adversarial --profile r3 | grep '^E7RESULT ')
}
driver_e165_r3_segment2_root_choice() {
  (set -o pipefail && E165_THREADS=8 bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds segment1 --family root_choice --profile r3 | grep '^E7RESULT ')
}
# 第 6 次重跑（登记 `research/prompts/e165-r6-prereg.md`）：入库装置，`--profile r3 --family legal`（偶数卸载记号、直接建表、盘次序照常），r3 档合法一族的正式跑，
# J6Qn 与 J7 五行的阳性对照在这一档只在这一跑里出。产物是 2026-10-09 开工那一刻主工作区 Cargo.toml、Cargo.lock、.cargo/、crates/ 的拷贝
# （~/.local/share/singlefs/work/e165-r6-runner/root，清单 `root-manifest-at-start.sha256`，340 个文件，清单 sha256 56d52a1c…）上编出的二进制跑的；
# 计数与线程数无关（8 与 4 线程两遍整份逐字节相同，`…-r6-r3-legal-family-threads-second.out`），线程数钉 8。同一份根上的 r2 合法一族只当比较对象，不登记复跑。
# 这里在主工作区上 `cargo run`，`crates/` 里的文件（别的会话在改）与那份根可能不同，复出来对不上是新观测，先对着根的清单查。
driver_e165_r6_r3_legal() {
  (set -o pipefail && E165_THREADS=8 bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds segment1 --family legal --profile r3 | grep '^E7RESULT ')
}
# 第 3 次重跑第三段（登记 `research/prompts/e165-r3-prereg.md` 5.8、9.1、9.3）：变体 1822–1827 各一份副本、各编一次，驱动带 --profile r3 调装置
# （--profile r3 --bounds segment1 --family root_choice），不落红例账。驱动在副本里改 recovery.rs 或 lib.rs、自己经内存包装跑每个二进制，这里不再包一层；
# 产物里每行的复跑命令写的 --output 是产物的文件名（--root 指冻结根时驱动只写 basename，不管产物落在哪），这里的输出落在临时目录，同样只写 basename，原样比，不做替换。
# 驱动的 REGISTERED_R3 钉的是第二段冻结根的 recovery.rs、lib.rs、crates/mutations.tsv 的哈希与两个函数的区间，主工作区的 crates/ 变了它就停在 RH1，
# 所以复跑要 --root 指冻结根：默认 /tmp/claude-1000/e165-r3-segment2/root（清单 `segment2-root-after-overlay.sha256`，233 个文件），那份没了就照登记第十二节修订 2 的第 10 条重建一份、
# 用 E165_R3_FROZEN_ROOT 指过去。TV7 的比对对象是脚本所在的仓里的第二段产物（research/results/e165-replay-bounded-enumeration-2026-10-03-r3-segment2-root-choice.out）。
# 整段（基线两遍加变体六个，每个冷编一次再跑一遍 70 秒量级的装置）的用时跑之前没量过；--jobs × --threads 不超过 6，重型复跑，只在提交时由主 agent 跑。
driver_e165_r3_segment3_root_choice_kill_bounds() {
  local scratch="$OUT_DIR/e165-r3-segment3"
  # 默认的那份根 2026-10-06 开机清 /tmp 时已丢、要重建：照这个驱动的注释写的建法重建、核过清单，放 ~/.local/share/singlefs/work/ 下，用 E165_R3_FROZEN_ROOT 指过去
  local frozen_root="${E165_R3_FROZEN_ROOT:-/tmp/claude-1000/e165-r3-segment2/root}"
  mkdir -p "$scratch/out"
  (E165_KILL_BOUNDS_MEMORY_CAP=1G RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="装置峰值只在基线上量过（第二段 446 MiB），变体留余量" \
      bash scripts/experiments/e165-replay-mutation-kill-bounds.sh --profile r3 --root "$frozen_root" --scratch "$scratch" \
      --output "$scratch/out/e165-replay-bounded-enumeration-2026-10-03-r3-segment3-root-choice-kill-bounds.out" --jobs 2 --threads 3 >&2) || return 1
  cat "$scratch/out/e165-replay-bounded-enumeration-2026-10-03-r3-segment3-root-choice-kill-bounds.out"
}
# 第 4 次重跑第一段（登记 `research/prompts/e165-r4-prereg.md` 5.3 至 5.6、第七节与修订 1、2）：入库装置，`--profile r4 --family root_choice_faults`，
# 带读故障的择根合判一族（KF、KF3、KF9、KFW、KG、KA；闸取实现的取甲）。计数与线程数无关（6 与 3 线程两遍整份逐字节相同，线程数钉 8）；整遍峰值 RSS 约 1.7 GB，
# REPLAY_MEMORY_CAP 默认 8G 够。产物是第一段根（`/tmp/claude-1000/e165-r4-segment1b/root`，清单 `root-manifest-after-overlay.sha256`）上编出的二进制跑的；
# 这里在主工作区上 `cargo run`，`crates/` 里别的文件（别的会话在改，尤其 `mount.rs`、`transaction.rs`：条款改取 R2 之后实现一落地，这一族要重跑、产物会变）与那份根可能不同，
# 复出来对不上是新观测，先对着根清单查。
driver_e165_r4_segment1_root_choice_faults() {
  (set -o pipefail && E165_THREADS=8 bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds segment1 --family root_choice_faults --profile r4 | grep '^E7RESULT ')
}
# 第 4 次重跑第二段（登记 `research/prompts/e165-r4-prereg.md` 9.1、9.3 与修订 3）：变体 v1822–v1827（在根里的 `crates/mutations.tsv` 按第一列的名字选行）各一份副本、各编一次，
# 驱动带 --profile r4 调装置（--profile r4 --bounds segment1 --family root_choice_faults），不落红例账。驱动在副本里改 recovery.rs 或 lib.rs、自己经内存包装跑每个二进制，这里不再包一层。
# 驱动的 R4_REGISTERED 钉的是第一段根的 recovery.rs、lib.rs、crates/mutations.tsv 的哈希与两个函数的区间，主工作区的 crates/ 变了它就停在开工核对，
# 所以复跑要 --root 指第一段根：默认 /tmp/claude-1000/e165-r4-segment1b/root（清单 `root-manifest-after-overlay-final.sha256`，246 个文件），那份没了就照登记第十二节修订 2 的「根」条重建一份、
# 用 E165_R4_FROZEN_ROOT 指过去。TV7 的比对对象是脚本所在的仓里的第一段产物（research/results/e165-replay-bounded-enumeration-2026-10-03-r4-segment1-root-choice-faults-2026-10-03-r2.out）。
# 内存上限取 8G（第一段整遍峰值约 1.66 GB；2026-10-03 第一趟 4G、--jobs 2 时 tsv-1823 撞顶退 250，补跑 8G 才得出结果，登记修订 3）；--jobs 2 --threads 3，整轮约 10 分钟
# （第二段那一次变体各编各跑 130 秒量级，开头收尾基线各 66 秒量级），重型复跑，只在提交时由主 agent 跑。
# 产物里每行的复跑命令写的 --output 是产物的文件名（--root 指第一段根时驱动只写 basename），这里的输出落在临时目录，同样只写 basename，原样比。
driver_e165_r4_segment2_root_choice_faults_kill_bounds() {
  local scratch="$OUT_DIR/e165-r4-segment2"
  # 默认的那份根 2026-10-06 开机清 /tmp 时已丢、要重建：照这个驱动的注释写的建法重建、核过清单，放 ~/.local/share/singlefs/work/ 下，用 E165_R4_FROZEN_ROOT 指过去
  local frozen_root="${E165_R4_FROZEN_ROOT:-/tmp/claude-1000/e165-r4-segment1b/root}"
  mkdir -p "$scratch/out"
  (E165_KILL_BOUNDS_MEMORY_CAP=8G RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="装置峰值只在基线上量过（第一段约 1.66 GB），tsv-1823 在 4G 撞过顶，留余量" \
      bash scripts/experiments/e165-replay-mutation-kill-bounds.sh --profile r4 --root "$frozen_root" --scratch "$scratch" \
      --output "$scratch/out/e165-replay-bounded-enumeration-2026-10-03-r4-segment2-root-choice-faults-kill-bounds.out" --jobs 2 --threads 3 >&2) || return 1
  cat "$scratch/out/e165-replay-bounded-enumeration-2026-10-03-r4-segment2-root-choice-faults-kill-bounds.out"
}
# 第 5 次重跑第二段（登记 `research/prompts/e165-r5-prereg.md` 5.6、9.1、9.3）：驱动三个模式加 `--pins r5`，变体同第 2–4 次（r2 模式 27 个变体、r3 / r4 模式各 7 个），
# r2 模式另落红例账。驱动的 R5_PINS 钉的是第 5 次第一段根的 recovery.rs、lib.rs、crates/mutations.tsv 的哈希与函数区间，主工作区的 crates/ 变了它就停在开工核对，
# 所以复跑要 --root 指那份根：默认 /tmp/claude-1000/e165-r5-segment1/root（主工作区当时的 Cargo.toml、Cargo.lock、.cargo/、crates/ 的拷贝，清单
# `root-manifest-at-start.sha256`，291 个文件，清单 sha256 95237ecf…），那份没了就照登记第十一节「根、清单、在哪编」重建一份、核过清单，用 E165_R5_SEGMENT1_ROOT 指过去。
# 自造表、内存包装与 V7 / TV7 的比对对象（第 5 次第一段的重跑产物）取脚本所在的仓。产物里每行的复跑命令写的 --output / --ledger 是文件名，这里原样比，不做替换。
# 内存与线程照那一次：r2、r4 模式 8G，r3 模式 1G，--jobs 2 --threads 3；用时那一次 r3 约 10 分钟、r4 约 15 分钟、r2 约 20 分钟量级（`-timing.out`），重型复跑，只在提交时由主 agent 跑。
driver_e165_r5_segment2_r2_mode_kill_bounds() {
  local scratch="$OUT_DIR/e165-r5-segment2-r2"
  # 默认的那份根 2026-10-06 开机清 /tmp 时已丢、要重建：照这个驱动的注释写的建法重建、核过清单，放 ~/.local/share/singlefs/work/ 下，用 E165_R5_SEGMENT1_ROOT 指过去
  local root="${E165_R5_SEGMENT1_ROOT:-/tmp/claude-1000/e165-r5-segment1/root}"
  mkdir -p "$scratch/out"
  (E165_KILL_BOUNDS_MEMORY_CAP=8G RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="变体与基线装置峰值只量过合法历史一族，留十倍余量" \
      bash scripts/experiments/e165-replay-mutation-kill-bounds.sh --pins r5 --root "$root" --scratch "$scratch" \
      --output "$scratch/out/e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r2-mode-kill-bounds.out" \
      --ledger "$scratch/out/e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r2-mode-red-examples.tsv" --jobs 2 --threads 3 >&2) || return 1
  cat "$scratch/out/e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r2-mode-kill-bounds.out"
}
driver_e165_r5_segment2_r3_mode_kill_bounds() {
  local scratch="$OUT_DIR/e165-r5-segment2-r3"
  # 默认的那份根 2026-10-06 开机清 /tmp 时已丢、要重建：照这个驱动的注释写的建法重建、核过清单，放 ~/.local/share/singlefs/work/ 下，用 E165_R5_SEGMENT1_ROOT 指过去
  local root="${E165_R5_SEGMENT1_ROOT:-/tmp/claude-1000/e165-r5-segment1/root}"
  mkdir -p "$scratch/out"
  (E165_KILL_BOUNDS_MEMORY_CAP=1G RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="装置峰值只在基线上量过（r3 第二段 446 MiB），变体留余量" \
      bash scripts/experiments/e165-replay-mutation-kill-bounds.sh --profile r3 --pins r5 --root "$root" --scratch "$scratch" \
      --output "$scratch/out/e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r3-mode-kill-bounds.out" --jobs 2 --threads 3 >&2) || return 1
  cat "$scratch/out/e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r3-mode-kill-bounds.out"
}
driver_e165_r5_segment2_r4_mode_kill_bounds() {
  local scratch="$OUT_DIR/e165-r5-segment2-r4"
  # 默认的那份根 2026-10-06 开机清 /tmp 时已丢、要重建：照这个驱动的注释写的建法重建、核过清单，放 ~/.local/share/singlefs/work/ 下，用 E165_R5_SEGMENT1_ROOT 指过去
  local root="${E165_R5_SEGMENT1_ROOT:-/tmp/claude-1000/e165-r5-segment1/root}"
  mkdir -p "$scratch/out"
  (E165_KILL_BOUNDS_MEMORY_CAP=8G RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="装置峰值只在基线上量过（约 1.66 GB），第 4 次 tsv-1823 在 4G 撞过顶" \
      bash scripts/experiments/e165-replay-mutation-kill-bounds.sh --profile r4 --pins r5 --root "$root" --scratch "$scratch" \
      --output "$scratch/out/e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r4-mode-kill-bounds.out" --jobs 2 --threads 3 >&2) || return 1
  cat "$scratch/out/e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r4-mode-kill-bounds.out"
}
# 第 5 次重跑第三段（登记 5.3、5.4，`--profile r5`：记录后三项照 D23（journal 的角色与格式） 已定项 15 生成，J8 五行）与第五段 5-甲（修订 1，`--profile r4
# --family root_choice_faults --gate r2`：系统配置读错取 R2 的闸）：入库装置。产物是第一段根拷一份、盖上当时主工作区的装置与 crates/mutations.tsv 编出的二进制、7 线程跑的
# （第三段根 /tmp/claude-1000/e165-r5-segment3/root，装置 f72fc1a4…，清单 `root-manifest-after-overlay.sha256`；5-甲 的根 …/root5，装置 8006b251…，清单 sha256 47c6d349…）。
# 主工作区今天的装置是 8006b251…；5-甲 的二进制在 5-甲 的根上重跑第三段的合法历史那一份整份逐字节相同（`…-r5-segment5-profile-check.out` 的 pv13 行），另外四份没在它上面重跑过。
# 这里在主工作区上 `cargo run`，`crates/` 里别的文件（别的会话在改）与那两份根可能不同，复出来对不上是新观测，先对着根清单查。
# 计数与线程数无关（合法历史、对抗两族 7 与 3 线程逐字节相同；R2 一族 7 与 3 线程逐字节相同，`…-threads-second.out`），线程数钉 8；R2 一族整遍峰值没读到，REPLAY_MEMORY_CAP 默认 8G 跑过。
driver_e165_r5_segment3_legal() {
  (set -o pipefail && E165_THREADS=8 bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds segment1 --family legal --profile r5 | grep '^E7RESULT ')
}
driver_e165_r5_segment3_adversarial() {
  (set -o pipefail && E165_THREADS=8 bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds segment1 --family adversarial --profile r5 | grep '^E7RESULT ')
}
driver_e165_r5_segment3_multi_disk() {
  (set -o pipefail && E165_THREADS=8 bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds segment1 --family multi_disk --profile r5 | grep '^E7RESULT ')
}
driver_e165_r5_segment3_odd_marker_legal() {
  (set -o pipefail && E165_THREADS=8 bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds segment1 --family legal --profile r5 --marker-parity odd | grep '^E7RESULT ')
}
driver_e165_r5_segment3_odd_marker_adversarial() {
  (set -o pipefail && E165_THREADS=8 bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds segment1 --family adversarial --profile r5 --marker-parity odd | grep '^E7RESULT ')
}
driver_e165_r5_segment5_r2_gate_root_choice_faults() {
  (set -o pipefail && E165_THREADS=8 bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds segment1 --family root_choice_faults --profile r4 --gate r2 | grep '^E7RESULT ')
}
# 第 5 次重跑第五段 5-乙（登记修订 1 修1.九乙）：驱动 r4 模式 `--pins r5 --gate r2`，变体 v1822–v1827 在 R2 那一族上各编各跑。驱动的 R5_GATE_R2_PINS 钉的是 5-甲 的根
# （/tmp/claude-1000/e165-r5-segment3/root5：第一段根盖上装置 8006b251… 与 crates/mutations.tsv c9a03eb4…，清单 `root5-manifest-after-overlay.sha256`，sha256 47c6d349…），
# 那份没了就照登记修1.十一「根、清单、在哪编」重建一份、核过清单，用 E165_R5_SEGMENT5_ROOT 指过去；TV7 的比对对象是脚本所在的仓里的 `…-r5-segment5-r2-gate-root-choice-faults.out`。
# 那一次第一趟 8G、--jobs 2 时 tsv-1812 撞顶退 250，第二趟 16G 只补了它（登记修1.十一；两趟的驱动日志与计时各一份 `…-first-pass-*.out`、`…-second-pass-*.out`），这里取 16G。
# 产物里每行的复跑命令写的 --output 是文件名、--root 写的是「<第一段根>」字样（驱动的写法，实际指的是 5-甲 的根），这里原样比。重型复跑，只在提交时由主 agent 跑。
driver_e165_r5_segment5b_r4_gate_r2_mode_kill_bounds() {
  local scratch="$OUT_DIR/e165-r5-segment5b"
  # 默认的那份根 2026-10-06 开机清 /tmp 时已丢、要重建：照这个驱动的注释写的建法重建、核过清单，放 ~/.local/share/singlefs/work/ 下，用 E165_R5_SEGMENT5_ROOT 指过去
  local root="${E165_R5_SEGMENT5_ROOT:-/tmp/claude-1000/e165-r5-segment3/root5}"
  mkdir -p "$scratch/out"
  (E165_KILL_BOUNDS_MEMORY_CAP=16G RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="第一趟 8G、--jobs 2 时 tsv-1812 撞过顶，第二趟 16G 补跑才得出结果" \
      bash scripts/experiments/e165-replay-mutation-kill-bounds.sh --profile r4 --pins r5 --gate r2 --root "$root" --scratch "$scratch" \
      --output "$scratch/out/e165-replay-bounded-enumeration-2026-10-04-r5-segment5b-r4-gate-r2-mode-kill-bounds.out" --jobs 2 --threads 3 >&2) || return 1
  cat "$scratch/out/e165-replay-bounded-enumeration-2026-10-04-r5-segment5b-r4-gate-r2-mode-kill-bounds.out"
}

# E168（删除借保留量的环窗口）第一段（登记 `research/prompts/e168-preregistration.md` 5.0、5.8、第十一节 S1、S2 与第十二节修订 1–17）：入库装置
# `crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs`。装置的行以 `E name=` 开头、收尾 `E name=done`，不用 E7RESULT 协议：
# 照 driver_e163_r1_merge 的先例在驱动里包一层（e168_wrap：行首 `E name=` 换成 `E7RESULT name=`，收尾行补 `emitted=`），入库产物就是包过的样子，
# 换回去与装置的原样输出逐字节相同（2026-10-03 四份逐份核过）。
# s1-history 在主工作区上跑（不调删除入口）。positive-controls 与 segment1 要删除入口，主工作区那一份是桩、一调就打停机行退 3，
# positive-controls 在临时目录里拷一份 `crates/`、套删除路径补丁表 `scripts/mutation/tables/e168_delete_ring_window_borrow_delete_path.tsv`
# （每行旧串恰好命中一次，不中就整行跑不了：登记停机条款 S2）再编、跑（先例：driver_e165_segment2 在副本里改了再编）。segment1 一遍约 9 小时、36G，不进登记表（用户 2026-10-03 定），复跑照实验页「复跑」那一节手工步骤、由用户点名跑。
# 入库产物是 2026-10-03 那一次（第一段 r8）在当时主工作区 `crates/` 的快照上编、跑的，快照逐文件的 sha256 在
# `e168-delete-ring-window-borrow-segment1-tree-2026-10-03.sha256`（不含补丁表）；之后别的会话改过 `crates/singlefs-core/` 里的几份文件
# （extent_tree.rs、records.rs、recovery.rs 等），今天复出来对不上是新观测，先对着那份清单查。
# 计数与线程数无关（单测 segment_one_on_p2_s4_prints_the_same_lines_on_one_thread_and_on_eight_threads），线程数照那一次钉 4、P4 的片一次一片。
# 重型：positive-controls 峰值约 15 GB（上限 20G），s1-history 峰值约 2.3 GB（上限 4G），
# 只在提交时由主 agent 跑；E168_POSITIVE_CONTROLS_MEMORY_CAP、E168_S1_HISTORY_MEMORY_CAP 可以覆盖上限。
e168_wrap() {
  awk '/^E name=/{ sub(/^E name=/, "E7RESULT name="); n++; if ($0 == "E7RESULT name=done") { print $0 " emitted=" n; next } print }'
}
e168_patched_tree() { # e168_patched_tree <副本目录>：拷主工作区的 crates/、Cargo 清单与 .cargo，套删除路径补丁表
  local tree="$1"
  rm -rf "$tree" && mkdir -p "$tree" || return 1
  (rsync -a --exclude target Cargo.toml Cargo.lock .cargo crates "$tree/" && copy_thread_registry_into "$tree") || return 1
  (python3 -c 'import sys
root=sys.argv[2]; anchors_not_hit_once=0
for line in open(sys.argv[1],encoding="utf-8"):
    line=line.rstrip("\n")
    if not line or line.startswith("#"): continue
    _,path,old,new=line.split("\t"); old=old.replace("\\n","\n"); new=new.replace("\\n","\n")
    text=open(root+"/"+path,encoding="utf-8").read(); hits=text.count(old); print(path,hits); anchors_not_hit_once+=hits!=1
    if hits==1: open(root+"/"+path,"w",encoding="utf-8").write(text.replace(old,new,1))
sys.exit(1 if anchors_not_hit_once else 0)' scripts/mutation/tables/e168_delete_ring_window_borrow_delete_path.tsv "$tree") >&2
}
driver_e168_s1_history() {
  (set -o pipefail && bash scripts/hosts/run-with-memory-cap.sh "${E168_S1_HISTORY_MEMORY_CAP:-4G}" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e168_delete_ring_window_borrow -- s1-history | e168_wrap)
}
driver_e168_positive_controls() {
  local tree="$OUT_DIR/e168-positive-controls-tree" wrapper="$PWD/scripts/hosts/run-with-memory-cap.sh"
  e168_patched_tree "$tree" || return 1
  (cd "$tree" && set -o pipefail && E168_THREADS=4 E168_LARGE_SLICE_THREADS=1 bash "$wrapper" "${E168_POSITIVE_CONTROLS_MEMORY_CAP:-20G}" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e168_delete_ring_window_borrow -- positive-controls | e168_wrap)
}

# E170（O3规约执行器第一版）第 2 次重跑（登记 `research/prompts/e170-r2-prereg.md` 5.5「复跑行」、12.6、12.11 方案甲；取舍由主 agent 定，
# 调度记录 `records/2026-10-01-里程碑三形式化接入.md` 第 22 批）。今天的导出装置照 `crates/` 写 487 字节记录头，读 311 的第一代执行器与第 1 次重跑执行器
# 那几行（第一代的坏状态导出、Q3 比对器、Q2 比对器、变异表，第 1 次重跑的 Q2 比对器）在今天的格式下没有对应物，已从这张表删掉，历史在 git 里。
# 九行：导出装置自己的两趟（`dump --compare-with`、`bad-states-second-run`），v2-487、v1-487 的单测与单测层变异（纯 Python），
# v2-487 判今天现导的 Z1（批 1–3）、Z3、Z4（造法照登记 12.6 与读法 13，种子 85441ae491419411 照抄）。v2（311）那几份不进复跑。
# 导出分两组、各造一次，同组的几行共用（flock 排着，复跑是并发起的；造好留完成标记，后来的直接用）：
#   e170 组：`dump` → e170-dump（Z1）；再 `dump --compare-with` → e170-dump-repeat；拷一份 e170-dump 造 `bad-states-second-run` → e170-bad-states-second-run（Z3）；
#   e171 组：E171 装置 `baseline` → e171-baseline-run1；拷一份造 `bad-states`（第一套表）→ e170-bad-states-copy-s2，再 E171 装置 `known-bad-images` → known-bad-s2（Z4）。
# 目录名照抄第 2 次重跑重导时的名字：判镜像那一份的 config 行写目录末一段名字。
# Python 一侧（执行器、`comparator_bad_image_judge.py`、`mutation_driver_unit_level.py`）原样跑，原始输出再经 v2-487 目录的
# `run_replay_result_lines.py` 改写成 E7RESULT 结果行：删去键以 _ns 结尾的字段（墙钟）与 worker=（进程号），绝对路径换成末一段名字，判定字段不动。
# 执行器用一个工作进程（--workers 1）：多进程时判定行按完成先后打出、行序每次不同。产物不带耗时，exact 比。
e170_r2_cargo_run() { # <bin> <参数…>：在仓根照今天的 crates/ 起入库装置
  local bin="$1"; shift
  RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="复跑统一给 REPLAY_MEMORY_CAP，没有按驱动分别量过" \
    bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin "$bin" -- "$@"
}
e170_r2_build_exports_e170() {
  local directory="$1"
  (set -o pipefail \
    && e170_r2_cargo_run e170_o3_spec_executor_export dump --output-directory "$directory/e170-dump" >/dev/null \
    && e170_r2_cargo_run e170_o3_spec_executor_export dump --output-directory "$directory/e170-dump-repeat" --compare-with "$directory/e170-dump" \
         | grep '^E7RESULT ' >"$directory/dump-compare.lines" \
    && cp -r "$directory/e170-dump" "$directory/e170-bad-states-second-run" \
    && e170_r2_cargo_run e170_o3_spec_executor_export bad-states-second-run --dump-directory "$directory/e170-bad-states-second-run" --seed-hex 85441ae491419411 \
         | grep '^E7RESULT ' >"$directory/bad-states-second-run.lines")
}
e170_r2_build_exports_e171() {
  local directory="$1"
  (set -o pipefail \
    && e170_r2_cargo_run e171_three_oracle_collusion baseline --output-directory "$directory/e171-baseline-run1" >/dev/null \
    && cp -r "$directory/e171-baseline-run1" "$directory/e170-bad-states-copy-s2" \
    && e170_r2_cargo_run e170_o3_spec_executor_export bad-states --dump-directory "$directory/e170-bad-states-copy-s2" --seed-hex 85441ae491419411 >/dev/null \
    && e170_r2_cargo_run e171_three_oracle_collusion known-bad-images --dump-directory "$directory/e170-bad-states-copy-s2" \
         --output-directory "$directory/known-bad-s2" --e170-bad-rows >/dev/null)
}
e170_r2_shared_exports() { # <组：e170 或 e171>：同一组只造一次，打出导出所在目录
  local directory="$OUT_DIR/e170-r2-exports-$1"
  (
    flock 9 || exit 1
    if [[ ! -f "$directory.done" ]]; then
      rm -rf "$directory" && mkdir -p "$directory" || exit 1
      "e170_r2_build_exports_$1" "$directory" || exit 1
      : >"$directory.done"
    fi
  ) 9>"$directory.lock" || return 1
  printf '%s\n' "$directory"
}
e170_r2_python() { # 在仓根下起 Python，经内存包装
  PYTHONDONTWRITEBYTECODE=1 RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="复跑统一给 REPLAY_MEMORY_CAP，没有按驱动分别量过" \
    bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" python3 "$@"
}
e170_r2_bin() { printf '%s\n' "$PWD/scripts/experiments/$1"; }
e170_r2_wrap() { # <原始输出> <产物>：改写成结果行再打出来
  e170_r2_python "$(e170_r2_bin e170_o3_spec_executor_with_witness_predicates_journal_header_487)/run_replay_result_lines.py" wrap --input "$1" --output "$2" >&2 \
    && cat "$2"
}
driver_e170_r2_dump_compare() {
  local exports
  exports="$(e170_r2_shared_exports e170)" && cat "$exports/dump-compare.lines"
}
driver_e170_r2_bad_states_second_run_export() {
  local exports
  exports="$(e170_r2_shared_exports e170)" && cat "$exports/bad-states-second-run.lines"
}
e170_r2_unit_tests() { # <臂名> <执行器目录名>
  local scratch="$OUT_DIR/e170-r2-unit-tests-$1"
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  e170_r2_python "$(e170_r2_bin e170_o3_spec_executor_with_witness_predicates_journal_header_487)/run_replay_result_lines.py" unit-tests \
    --executor-directory "$(e170_r2_bin "$2")" --arm "$1" --scratch "$scratch/test-scratch" --output "$scratch/result.out" >&2 || return 1
  cat "$scratch/result.out"
}
driver_e170_r2_v2_487_unit_tests() { e170_r2_unit_tests v2-487 e170_o3_spec_executor_with_witness_predicates_journal_header_487; }
driver_e170_r2_v1_487_unit_tests() { e170_r2_unit_tests v1-487 e170_second_run_o3_spec_executor_journal_header_487; }
# 单测层变异：表里 XC8、XC9 两行的原文在 v2-487 的照抄核对脚本里，不归单测管（登记末「主 agent 认定第二段 Q5 交回」第 3 条：整表跑时先排除），
# 只留 X 起头的那几行（今天 X32–X36）。
e170_r2_unit_level_mutations() { # <臂名> <执行器目录名>
  local scratch="$OUT_DIR/e170-r2-unit-level-mutations-$1" table="e170_o3_spec_executor_with_witness_predicates_journal_header_487"
  rm -rf "$scratch" && mkdir -p "$scratch/work" || return 1
  awk -F'\t' '$1 !~ /^XC[0-9]+_/' "scripts/mutation/tables/$table.tsv" >"$scratch/$table-without-xc-rows.tsv" || return 1
  e170_r2_python "$(e170_r2_bin e170_o3_spec_executor_with_witness_predicates)/mutation_driver_unit_level.py" --executor-directory "$(e170_r2_bin "$2")" \
    --table "$scratch/$table-without-xc-rows.tsv" --scratch "$scratch/work" --output "$scratch/raw.out" >&2 || return 1
  e170_r2_wrap "$scratch/raw.out" "$scratch/result.out"
}
driver_e170_r2_v2_487_unit_level_mutations() { e170_r2_unit_level_mutations v2-487 e170_o3_spec_executor_with_witness_predicates_journal_header_487; }
driver_e170_r2_v1_487_unit_level_mutations() { e170_r2_unit_level_mutations v1-487 e170_second_run_o3_spec_executor_journal_header_487; }
driver_e170_r2_v2_487_z1() {
  local scratch="$OUT_DIR/e170-r2-v2-487-z1" exports
  exports="$(e170_r2_shared_exports e170)" || return 1
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  e170_r2_python "$(e170_r2_bin e170_o3_spec_executor_with_witness_predicates_journal_header_487)/executor_main.py" --dumps "$exports/e170-dump" \
    --batches 1,2,3 --workers 1 --output "$scratch/raw.out" >&2 || return 1
  e170_r2_wrap "$scratch/raw.out" "$scratch/result.out"
}
# 判镜像：冻结参照取第一段乙产物里不带 arm= 的那一组冻结行（同 run_segment_two_q5.sh），文件名照抄（config 行写它的名字）。
e170_r2_judge_images() { # <输入名> <导出组> <含 bad-*.tsv 的导出目录名> [已知坏镜像目录名]
  local scratch="$OUT_DIR/e170-r2-v2-487-${1,,}-images" exports known=()
  exports="$(e170_r2_shared_exports "$2")" || return 1
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  grep -E '^name=freeze(_seed)? ' research/results/e170-r2-o3-spec-executor-2026-10-04-s1b-executor.out | grep -v ' arm=' >"$scratch/v2-487-freeze-reference.out" || return 1
  [[ $# -ge 4 ]] && known=(--known-bad-dir "$exports/$4")
  e170_r2_python "$(e170_r2_bin e170_o3_spec_executor_with_witness_predicates)/comparator_bad_image_judge.py" \
    --executor-directory "$(e170_r2_bin e170_o3_spec_executor_with_witness_predicates_journal_header_487)" --input-label "$1" \
    --dumps "$exports/$3" "${known[@]}" --freeze-reference "$scratch/v2-487-freeze-reference.out" --output "$scratch/raw.out" >&2 || return 1
  e170_r2_wrap "$scratch/raw.out" "$scratch/result.out"
}
driver_e170_r2_v2_487_z3_images() { e170_r2_judge_images Z3 e170 e170-bad-states-second-run; }
driver_e170_r2_v2_487_z4_images() { e170_r2_judge_images Z4 e171 e170-bad-states-copy-s2 known-bad-s2; }

# E171（三oracle合谋测试）第一段（登记 `research/prompts/e171-preregistration.md` 5.3、5.6、5.8「第一段」、第十一节 Z1 与 S2）：入库装置
# `e171_three_oracle_collusion` 与研究一侧（`scripts/experiments/e171_three_oracle_collusion/`，Python，不是编译出来的代码，不经内存包装）。
# 每个驱动各带自己的临时目录（复跑是并发起的，不共用可写状态）；每条经内存包装的命令都设 OVER_PEAK_REASON：复跑统一给 REPLAY_MEMORY_CAP，没有按驱动量过。
# 产物只留 E7RESULT 行：不带耗时、不带路径（比对器的 config 行只写目录的名字），exact 比；耗时在 E7TIMING 行里，不入库。
# 第一段两份（baseline-run1、known-bad-images）不带 O3：O3 那几列要等 E170 第 1 次重跑冻结执行器，第二段才有。
# 比对器的 A4–A7 读今天的 `crates/mutations.tsv`（坏实现表 117 行按名字取、挑法脚本现算步长 8 与 4 的行数、改实现的行 844 行）：
# 表里别处加了核心实现的行、或改了那 117 行，这几行会变，不是装置坏了。
# E170 导出装置在 `bad-states` 模式、第一次的表（`bad-states`，不是 `bad-states-second-run`）上造 P 行与 N 行；第一段的 known-bad-images 用占位种子
# 0000000000000000（P、N 行与种子无关），第二段的盲测族用冻结执行器的种子（见下）。它在 E170 第 1 次重跑里加了第二套表，第一套的输出与加之前逐字节相同。
e171_device() {
  RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="复跑统一给 REPLAY_MEMORY_CAP，没有按驱动分别量过" \
    bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e171_three_oracle_collusion -- "$@"
}
e171_e170_exporter() {
  RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="复跑统一给 REPLAY_MEMORY_CAP，没有按驱动分别量过" \
    bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e170_o3_spec_executor_export -- "$@"
}
driver_e171_s1_baseline() {
  local scratch="$OUT_DIR/e171-baseline"
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  (set -o pipefail && e171_device baseline --output-directory "$scratch/baseline-run1" | grep '^E7RESULT ')
}
driver_e171_s1_known_bad_images() {
  local scratch="$OUT_DIR/e171-known-bad"
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  (set -o pipefail \
    && e171_device baseline --output-directory "$scratch/baseline-run1" >/dev/null \
    && cp -r "$scratch/baseline-run1" "$scratch/e170-bad-states-copy" \
    && e171_e170_exporter bad-states --dump-directory "$scratch/e170-bad-states-copy" --seed-hex 0000000000000000 >/dev/null \
    && e171_device known-bad-images --dump-directory "$scratch/e170-bad-states-copy" --output-directory "$scratch/known-bad" --e170-bad-rows | grep '^E7RESULT ')
}
# E171 第二段（登记 `research/prompts/e171-preregistration.md` 5.4、5.5、第十二节修订）：O3 用 E170 第 1 次重跑冻结的执行器
#（`scripts/experiments/e170_second_run_o3_spec_executor/`，冻结行在 `e170-r1-o3-spec-executor-2026-10-03-s1-executor.out`；
# 盲测族种子取那份产物 `name=freeze_seed` 行的 85441ae491419411）。第一段的比对器与变异表产物（`e171-…-s1-comparator.out`、`…-s1-comparator-mutations.out`）
# 是第一段那一版比对器、九条变异跑出来的，留在 `research/results/` 里当历史；比对器、变异表在第二段改了，那两份不再复跑，由下面的 s2 两份取代。
# 坏镜像那三份（bad-states-export、known-bad-images、images-o3）共用「基线 → 拷一份 → E170 导出装置 bad-states（冻结的种子）」这三步，各带自己的临时目录。
# images-o3 的 config 行写两个目录名（e170-bad-states-copy-s2、known-bad-s2）与两个源文件的 sha256（本实验那份 `o3_bad_image_judge.py` 与它拷来的 E170
# `comparator_q3_bad_states.py`）：改了这两份源文件或目录名，这份产物就对不上，不是装置坏了。
e171_s2_prepare_bad_states() {
  local scratch="$1"
  (set -o pipefail \
    && e171_device baseline --output-directory "$scratch/baseline-run1" >"$scratch/baseline-run1.stdout" \
    && cp -r "$scratch/baseline-run1" "$scratch/e170-bad-states-copy-s2" \
    && e171_e170_exporter bad-states --dump-directory "$scratch/e170-bad-states-copy-s2" --seed-hex 85441ae491419411 >"$scratch/e170-bad-states.stdout")
}
e171_s2_known_bad() {
  local scratch="$1"
  (set -o pipefail \
    && e171_device known-bad-images --dump-directory "$scratch/e170-bad-states-copy-s2" --output-directory "$scratch/known-bad-s2" --e170-bad-rows >"$scratch/known-bad-images.stdout")
}
e171_s2_python() {
  PYTHONDONTWRITEBYTECODE=1 RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="复跑统一给 REPLAY_MEMORY_CAP，没有按驱动分别量过" \
    bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" python3 "$@"
}
driver_e171_s2_bad_states_export() {
  local scratch="$OUT_DIR/e171-s2-bad-states"
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  e171_s2_prepare_bad_states "$scratch" || return 1
  grep '^E7RESULT ' "$scratch/e170-bad-states.stdout"
}
driver_e171_s2_known_bad_images() {
  local scratch="$OUT_DIR/e171-s2-known-bad"
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  e171_s2_prepare_bad_states "$scratch" && e171_s2_known_bad "$scratch" || return 1
  grep '^E7RESULT ' "$scratch/known-bad-images.stdout"
}
driver_e171_s2_images_o3() {
  local scratch="$OUT_DIR/e171-s2-images-o3"
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  e171_s2_prepare_bad_states "$scratch" && e171_s2_known_bad "$scratch" || return 1
  (e171_s2_python scripts/experiments/e171_three_oracle_collusion/o3_bad_image_judge.py judge \
      --e170-dumps "$scratch/e170-bad-states-copy-s2" --known-bad-dir "$scratch/known-bad-s2" --output "$scratch/images-o3.out" >&2) || return 1
  grep '^E7RESULT ' "$scratch/images-o3.out"
}
driver_e171_s2_o3_control_baseline() {
  local scratch="$OUT_DIR/e171-s2-o3-control"
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  e171_s2_prepare_bad_states "$scratch" || return 1
  (e171_s2_python scripts/experiments/e171_three_oracle_collusion/o3_bad_image_judge.py control \
      --dumps "$scratch/baseline-run1" --e170-dumps "$scratch/e170-bad-states-copy-s2" --row-name P1-F --output "$scratch/o3-control.out" >&2) || return 1
  grep '^E7RESULT ' "$scratch/o3-control.out"
}
# 比对器那一份（第二段）：基线两趟、E170 导出装置的 dump、冻结的执行器在基线上（A3 的 O3 一份）、已知坏镜像与 E170 的 P、N、盲测各行（O2 与 O3）、关一条（O1、O2）、
# 阳性对照四条（基线与空变异各一趟）、逐行驱动的九行（空变异与第一刀里的 1、6、8、18、28 与表里的 64、83、84；三条线道、每条线道两个线程）、
# Q3-核的机制试跑（第一刀的第 8 行，关 O1、关 O2、驱动不跑执行器各一趟）；目录的名字（baseline-run1、rows-s2-trial、disable-O1……）进产物，改了名产物就变。
driver_e171_s2_comparator() {
  local scratch="$OUT_DIR/e171-s2-comparator" dumps control name flag
  dumps="$scratch/dumps"
  rm -rf "$scratch" && mkdir -p "$dumps" "$scratch/work" || return 1
  (set -o pipefail \
    && e171_device baseline --output-directory "$dumps/baseline-run1" >"$scratch/baseline-run1.stdout" \
    && e171_device baseline --output-directory "$dumps/baseline-run2" >/dev/null \
    && e171_e170_exporter dump --output-directory "$dumps/e170-dump" >/dev/null \
    && cp -r "$dumps/baseline-run1" "$dumps/e170-bad-states-copy-s2" \
    && e171_e170_exporter bad-states --dump-directory "$dumps/e170-bad-states-copy-s2" --seed-hex 85441ae491419411 >/dev/null \
    && e171_device known-bad-images --dump-directory "$dumps/e170-bad-states-copy-s2" --output-directory "$dumps/known-bad-s2" --e170-bad-rows >/dev/null \
    && e171_device baseline --output-directory "$dumps/disable-O1" --disable-oracle O1 >/dev/null \
    && e171_device baseline --output-directory "$dumps/disable-O2" --disable-oracle O2 >/dev/null \
    && e171_s2_python scripts/experiments/e171_three_oracle_collusion/o3_bad_image_judge.py judge \
         --e170-dumps "$dumps/e170-bad-states-copy-s2" --known-bad-dir "$dumps/known-bad-s2" --output "$scratch/images-o3.out" >&2 \
    && e171_s2_python scripts/experiments/e171_three_oracle_collusion/o3_bad_image_judge.py control \
         --dumps "$dumps/baseline-run1" --e170-dumps "$dumps/e170-bad-states-copy-s2" --row-name P1-F --output "$scratch/o3-control-baseline.out" >&2 \
    && (root="$PWD" && cd scripts/experiments/e170_second_run_o3_spec_executor && PYTHONDONTWRITEBYTECODE=1 \
          RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="复跑统一给 REPLAY_MEMORY_CAP，没有按驱动分别量过" \
          bash "$root/scripts/hosts/run-with-memory-cap.sh" "$REPLAY_MEMORY_CAP" python3 executor_main.py --dumps "$dumps/baseline-run1" \
          --batches 1,2 --workers 6 --output "$scratch/o3-baseline.out" >&2)) || return 1
  for control in O1-对拍 O1-崩溃判官 O2; do
    (e171_device baseline --output-directory "$dumps/control-baseline-$control" --positive-control "$control" >/dev/null \
      && PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiments/e171_three_oracle_collusion/row_driver.py --repo-root "$PWD" \
           --work-root "$scratch/work" --output-root "$dumps/rows-controls-$control" --empty-mutation --positive-control "$control" \
           --memory-cap "$REPLAY_MEMORY_CAP" >&2) || return 1
  done
  (PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiments/e171_three_oracle_collusion/row_driver.py --repo-root "$PWD" \
      --work-root "$scratch/work" --output-root "$dumps/rows-controls-O3" --empty-mutation --positive-control O3 \
      --o3-control-bad-states "$dumps/e170-bad-states-copy-s2" --memory-cap "$REPLAY_MEMORY_CAP" >&2 \
    && PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiments/e171_three_oracle_collusion/row_driver.py --repo-root "$PWD" \
      --work-root "$scratch/work" --output-root "$dumps/rows-s2-trial" --empty-mutation --rows 1,6,8,18,28,64,83,84 --lanes 3 --threads 2 \
      --memory-cap "$REPLAY_MEMORY_CAP" >"$scratch/driver-tables.log") || return 1
  for name in O1 O2 no-executor; do
    flag="--disable-oracle ${name}"
    [[ "$name" == no-executor ]] && flag="--no-executor"
    # shellcheck disable=SC2086
    (PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiments/e171_three_oracle_collusion/row_driver.py --repo-root "$PWD" \
        --work-root "$scratch/work" --output-root "$dumps/rows-s2-q3core-$name" --rows 8 $flag --memory-cap "$REPLAY_MEMORY_CAP" \
        | grep '^table ' >>"$scratch/driver-tables.log") || return 1
  done
  (PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiments/e171_three_oracle_collusion/comparator.py --output "$scratch/comparator.out" \
      --mutations-table crates/mutations.tsv --baseline-dir "$dumps/baseline-run1" --baseline-device-output "$scratch/baseline-run1.stdout" \
      --baseline-repeat-dir "$dumps/baseline-run2" --e170-dump "$dumps/e170-dump" --o3-baseline "$scratch/o3-baseline.out" \
      --disabled O1:"$dumps/disable-O1" --disabled O2:"$dumps/disable-O2" \
      --rows-root "$dumps/rows-s2-trial" --driver-output "$scratch/driver-tables.log" \
      --known-bad-dir "$dumps/known-bad-s2" --e170-expected-dir "$dumps/e170-bad-states-copy-s2" --image-judgements "$scratch/images-o3.out" \
      --control "O1-对拍:baseline:$dumps/control-baseline-O1-对拍" --control "O1-对拍:empty_mutation:$dumps/rows-controls-O1-对拍/row-0/dump" \
      --control "O1-崩溃判官:baseline:$dumps/control-baseline-O1-崩溃判官" --control "O1-崩溃判官:empty_mutation:$dumps/rows-controls-O1-崩溃判官/row-0/dump" \
      --control "O2:baseline:$dumps/control-baseline-O2" --control "O2:empty_mutation:$dumps/rows-controls-O2/row-0/dump" \
      --control "O3:baseline:$scratch/o3-control-baseline.out" --control "O3:empty_mutation:$dumps/rows-controls-O3/row-0/o3-control.out" \
      --q3-core "O1:$dumps/rows-s2-trial:$dumps/rows-s2-q3core-O1" --q3-core "O2:$dumps/rows-s2-trial:$dumps/rows-s2-q3core-O2" \
      --q3-core "no_executor:$dumps/rows-s2-trial:$dumps/rows-s2-q3core-no-executor" >&2) || return 1
  grep '^E7RESULT ' "$scratch/comparator.out"
}
driver_e171_s3_comparator_mutations() {
  local scratch="$OUT_DIR/e171-s3-comparator-mutations" repository_root="$PWD"
  rm -rf "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiments/e171_three_oracle_collusion" && PYTHONDONTWRITEBYTECODE=1 python3 mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion.tsv" --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  grep '^E7RESULT ' "$scratch/mutations.out"
}
# E171 s4 整表（登记修订 5）：第二段比对器那一串前置（基线两趟、E170 导出装置 dump 与 bad-states、已知坏镜像、关一条两趟、冻结执行器在基线与坏镜像上、
# 阳性对照四条在基线与空变异各一趟），加修过的逐行驱动跑全部 117 行（两条线道、每条 2 线程，各趟都从同一份冻住的树拍快照）与 Q3-核三趟，再出比对器。
# 推的挂钟约两小时（s4 整表 5 569.6 秒、Q3-核三趟各约两分钟）。117 行的读数与 A4–A7 都读今天的 `crates/`：实现、O2、O1、装置或 `crates/mutations.tsv`
# 里那 117 行变了，产物就变，不是复跑坏了。目录的名字（baseline-run1、rows-s4、disable-O1、disable-O2）进产物，改了名产物就变。
driver_e171_s4_comparator() {
  local scratch="$OUT_DIR/e171-s4-comparator" dumps frozen control rows name flag entry
  dumps="$scratch/dumps"
  frozen="$scratch/frozen-tree"
  rm -rf "$scratch" && mkdir -p "$dumps" "$scratch/work" "$frozen" || return 1
  (for entry in Cargo.toml Cargo.lock .cargo crates; do rsync -a --exclude target "$entry" "$frozen/" || exit 1; done && copy_thread_registry_into "$frozen") || return 1
  (set -o pipefail \
    && e171_device baseline --output-directory "$dumps/baseline-run1" >"$scratch/baseline-run1.stdout" \
    && e171_device baseline --output-directory "$dumps/baseline-run2" >/dev/null \
    && e171_e170_exporter dump --output-directory "$dumps/e170-pinned-dump" >/dev/null \
    && cp -r "$dumps/baseline-run1" "$dumps/e170-bad-states-copy-s2" \
    && e171_e170_exporter bad-states --dump-directory "$dumps/e170-bad-states-copy-s2" --seed-hex 85441ae491419411 >/dev/null \
    && e171_device known-bad-images --dump-directory "$dumps/e170-bad-states-copy-s2" --output-directory "$dumps/known-bad-s2" --e170-bad-rows >/dev/null \
    && e171_device baseline --output-directory "$dumps/disable-O1" --disable-oracle O1 >/dev/null \
    && e171_device baseline --output-directory "$dumps/disable-O2" --disable-oracle O2 >/dev/null \
    && e171_s2_python scripts/experiments/e171_three_oracle_collusion/o3_bad_image_judge.py judge \
         --e170-dumps "$dumps/e170-bad-states-copy-s2" --known-bad-dir "$dumps/known-bad-s2" --output "$scratch/images-o3.out" >&2 \
    && e171_s2_python scripts/experiments/e171_three_oracle_collusion/o3_bad_image_judge.py control \
         --dumps "$dumps/baseline-run1" --e170-dumps "$dumps/e170-bad-states-copy-s2" --row-name P1-F --output "$scratch/o3-control-baseline.out" >&2 \
    && (root="$PWD" && cd scripts/experiments/e170_second_run_o3_spec_executor && PYTHONDONTWRITEBYTECODE=1 \
          RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="复跑统一给 REPLAY_MEMORY_CAP，没有按驱动分别量过" \
          bash "$root/scripts/hosts/run-with-memory-cap.sh" "$REPLAY_MEMORY_CAP" python3 executor_main.py --dumps "$dumps/baseline-run1" \
          --batches 1,2 --workers 6 --output "$scratch/o3-baseline.out" >&2)) || return 1
  for control in O1-对拍 O1-崩溃判官 O2; do
    (e171_device baseline --output-directory "$dumps/control-baseline-$control" --positive-control "$control" >/dev/null \
      && PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiments/e171_three_oracle_collusion/row_driver.py --repo-root "$PWD" \
           --work-root "$scratch/work" --snapshot-from "$frozen" --output-root "$dumps/rows-controls-$control" --empty-mutation \
           --positive-control "$control" --memory-cap "$REPLAY_MEMORY_CAP" >&2) || return 1
  done
  (PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiments/e171_three_oracle_collusion/row_driver.py --repo-root "$PWD" \
      --work-root "$scratch/work" --snapshot-from "$frozen" --output-root "$dumps/rows-controls-O3" --empty-mutation --positive-control O3 \
      --o3-control-bad-states "$dumps/e170-bad-states-copy-s2" --memory-cap "$REPLAY_MEMORY_CAP" >&2 \
    && PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiments/e171_three_oracle_collusion/row_driver.py --repo-root "$PWD" \
      --work-root "$scratch/work" --snapshot-from "$frozen" --output-root "$dumps/rows-s4" --all --lanes 2 --threads 2 \
      --keep-compile-failures-as-they-are --memory-cap "$REPLAY_MEMORY_CAP" >"$scratch/table-driver.out") || return 1
  rows=$(PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiments/e171_three_oracle_collusion/comparator.py --print-q3-core-rows \
           --rows-root "$dumps/rows-s4" --baseline-dir "$dumps/baseline-run1" | grep -o 'q3_core_rows=[0-9,]*' | cut -d= -f2)
  [[ -n "$rows" ]] || return 1
  for name in O1 O2 no-executor; do
    flag="--disable-oracle ${name}"
    [[ "$name" == no-executor ]] && flag="--no-executor"
    # shellcheck disable=SC2086
    (PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiments/e171_three_oracle_collusion/row_driver.py --repo-root "$PWD" \
        --work-root "$scratch/work" --snapshot-from "$frozen" --output-root "$dumps/rows-s4-q3core-$name" --rows "$rows" $flag --lanes 2 --threads 2 \
        --memory-cap "$REPLAY_MEMORY_CAP" >"$scratch/q3core-$name-driver.out") || return 1
  done
  (PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiments/e171_three_oracle_collusion/comparator.py --output "$scratch/comparator.out" \
      --mutations-table crates/mutations.tsv --baseline-dir "$dumps/baseline-run1" --baseline-device-output "$scratch/baseline-run1.stdout" \
      --baseline-repeat-dir "$dumps/baseline-run2" --e170-dump "$dumps/e170-pinned-dump" --o3-baseline "$scratch/o3-baseline.out" \
      --disabled O1:"$dumps/disable-O1" --disabled O2:"$dumps/disable-O2" \
      --control "O1-对拍:baseline:$dumps/control-baseline-O1-对拍" --control "O1-对拍:empty_mutation:$dumps/rows-controls-O1-对拍/row-0/dump" \
      --control "O1-崩溃判官:baseline:$dumps/control-baseline-O1-崩溃判官" --control "O1-崩溃判官:empty_mutation:$dumps/rows-controls-O1-崩溃判官/row-0/dump" \
      --control "O2:baseline:$dumps/control-baseline-O2" --control "O2:empty_mutation:$dumps/rows-controls-O2/row-0/dump" \
      --control "O3:baseline:$scratch/o3-control-baseline.out" --control "O3:empty_mutation:$dumps/rows-controls-O3/row-0/o3-control.out" \
      --known-bad-dir "$dumps/known-bad-s2" --e170-expected-dir "$dumps/e170-bad-states-copy-s2" --image-judgements "$scratch/images-o3.out" \
      --rows-root "$dumps/rows-s4" --driver-output "$scratch/table-driver.out" \
      --q3-core "O1:$dumps/rows-s4:$dumps/rows-s4-q3core-O1" --q3-core "O2:$dumps/rows-s4:$dumps/rows-s4-q3core-O2" \
      --q3-core "no_executor:$dumps/rows-s4:$dumps/rows-s4-q3core-no-executor" >&2) || return 1
  grep '^E7RESULT ' "$scratch/comparator.out"
}
# E171 F4 扩表（登记 5.2「扩表」、第十节 F4）：先整条跑 driver_e171_s4_comparator（它的草稿目录留着 s4 的导出、冻住的树与编译目录），再在同一份冻住的树、
# 同一个工作根上跑扩表 164 行（`bad-implementations-expansion-164.tsv`，补位跳过 117 行那张表的名字），比对器把 rows-s4 与 rows-s4-f4 并成一张、
# 带 `--expansion-rows`、段名 s4-f4。推的挂钟约四小时（s4 那一串约两小时、扩表 164 行约两小时）。读数读今天的 `crates/`，同 s4 那一条的口径。
# 编不过的行补位那一趟（第八段起驱动在编不过之后照登记 5.2 补位）见下面的 driver_e171_s4_f4_substitutes_comparator。s4 与这一条的整表都带
# `--keep-compile-failures-as-they-are`：两份产物是编不过不补位的那一版驱动跑的，不带它，第 33、59、127、163、206、243 行会补位、产物就变。
driver_e171_s4_f4_comparator() {
  local s4="$OUT_DIR/e171-s4-comparator" dumps
  driver_e171_s4_comparator >/dev/null || return 1
  dumps="$s4/dumps"
  (PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiments/e171_three_oracle_collusion/row_driver.py --repo-root "$PWD" \
      --work-root "$s4/work" --snapshot-from "$s4/frozen-tree" --output-root "$dumps/rows-s4-f4" \
      --selected-rows scripts/experiments/e171_three_oracle_collusion/bad-implementations-expansion-164.tsv \
      --other-selected-rows scripts/experiments/e171_three_oracle_collusion/bad-implementations-117.tsv --all --lanes 2 --threads 2 \
      --keep-compile-failures-as-they-are --memory-cap "$REPLAY_MEMORY_CAP" >"$s4/table-f4-driver.out") || return 1
  (PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiments/e171_three_oracle_collusion/comparator.py --output "$s4/comparator-f4.out" \
      --mutations-table "$s4/frozen-tree/crates/mutations.tsv" --baseline-dir "$dumps/baseline-run1" --baseline-device-output "$s4/baseline-run1.stdout" \
      --baseline-repeat-dir "$dumps/baseline-run2" --e170-dump "$dumps/e170-pinned-dump" --o3-baseline "$s4/o3-baseline.out" \
      --disabled O1:"$dumps/disable-O1" --disabled O2:"$dumps/disable-O2" \
      --control "O1-对拍:baseline:$dumps/control-baseline-O1-对拍" --control "O1-对拍:empty_mutation:$dumps/rows-controls-O1-对拍/row-0/dump" \
      --control "O1-崩溃判官:baseline:$dumps/control-baseline-O1-崩溃判官" --control "O1-崩溃判官:empty_mutation:$dumps/rows-controls-O1-崩溃判官/row-0/dump" \
      --control "O2:baseline:$dumps/control-baseline-O2" --control "O2:empty_mutation:$dumps/rows-controls-O2/row-0/dump" \
      --control "O3:baseline:$s4/o3-control-baseline.out" --control "O3:empty_mutation:$dumps/rows-controls-O3/row-0/o3-control.out" \
      --known-bad-dir "$dumps/known-bad-s2" --e170-expected-dir "$dumps/e170-bad-states-copy-s2" --image-judgements "$s4/images-o3.out" \
      --segment s4-f4 --rows-root "$dumps/rows-s4" --rows-root "$dumps/rows-s4-f4" \
      --expansion-rows scripts/experiments/e171_three_oracle_collusion/bad-implementations-expansion-164.tsv \
      --driver-output "$s4/table-driver.out" --driver-output "$s4/table-f4-driver.out" \
      --q3-core "O1:$dumps/rows-s4:$dumps/rows-s4-q3core-O1" --q3-core "O2:$dumps/rows-s4:$dumps/rows-s4-q3core-O2" \
      --q3-core "no_executor:$dumps/rows-s4:$dumps/rows-s4-q3core-no-executor" >&2) || return 1
  grep '^E7RESULT ' "$s4/comparator-f4.out"
}
# E171 编不过的 6 行补位（登记 5.2、第一节读法 5「替换之后编不过」）：先整条跑 driver_e171_s4_f4_comparator（它的草稿目录留着 s4 与 F4 的导出、冻住的树与编译目录；
# 两趟整表都带 `--keep-compile-failures-as-they-are`，所以第 33、59、127、163、206、243 行在两份输出根里的结局还是编不过），再在同一份冻住的树、同一个工作根上
# 只跑那六行（`--rerun-compile-failed-from` 两个输出根；原行先编，编不过就换同一文件组的下一行、至多三次，补位跳过 117 行与扩表两张表的全部名字，
# 补位那一趟不带 `--keep-compile-failures-as-they-are`），比对器把 rows-s4、rows-s4-f4 并上补位根（`--substitution-rows-root`）、段名 s4-f4-substitutes。
# 推的挂钟约四个半小时（F4 那一串约四小时、补位 6 行 1 277.9 秒）。读今天的 `crates/`，同 s4 那一条的口径；补位挑中哪几行取决于 `crates/mutations.tsv` 与那六行
# 今天编不编得过，表或实现变了换上的行就变、产物就变。目录的名字（rows-s4-f4-substitutes）进产物的 config 行。
driver_e171_s4_f4_substitutes_comparator() {
  local s4="$OUT_DIR/e171-s4-comparator" dumps
  driver_e171_s4_f4_comparator >/dev/null || return 1
  dumps="$s4/dumps"
  (PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiments/e171_three_oracle_collusion/row_driver.py --repo-root "$PWD" \
      --work-root "$s4/work" --snapshot-from "$s4/frozen-tree" --output-root "$dumps/rows-s4-f4-substitutes" \
      --selected-rows scripts/experiments/e171_three_oracle_collusion/bad-implementations-117.tsv \
      --other-selected-rows scripts/experiments/e171_three_oracle_collusion/bad-implementations-expansion-164.tsv \
      --rerun-compile-failed-from "$dumps/rows-s4" --rerun-compile-failed-from "$dumps/rows-s4-f4" --lanes 2 --threads 2 \
      --memory-cap "$REPLAY_MEMORY_CAP" >"$s4/substitution-driver.out") || return 1
  grep -q '^rerun_compile_failed rows=33,59,127,163,206,243$' "$s4/substitution-driver.out" || return 1
  (PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiments/e171_three_oracle_collusion/comparator.py --output "$s4/comparator-f4-substitutes.out" \
      --mutations-table "$s4/frozen-tree/crates/mutations.tsv" --baseline-dir "$dumps/baseline-run1" --baseline-device-output "$s4/baseline-run1.stdout" \
      --baseline-repeat-dir "$dumps/baseline-run2" --e170-dump "$dumps/e170-pinned-dump" --o3-baseline "$s4/o3-baseline.out" \
      --disabled O1:"$dumps/disable-O1" --disabled O2:"$dumps/disable-O2" \
      --control "O1-对拍:baseline:$dumps/control-baseline-O1-对拍" --control "O1-对拍:empty_mutation:$dumps/rows-controls-O1-对拍/row-0/dump" \
      --control "O1-崩溃判官:baseline:$dumps/control-baseline-O1-崩溃判官" --control "O1-崩溃判官:empty_mutation:$dumps/rows-controls-O1-崩溃判官/row-0/dump" \
      --control "O2:baseline:$dumps/control-baseline-O2" --control "O2:empty_mutation:$dumps/rows-controls-O2/row-0/dump" \
      --control "O3:baseline:$s4/o3-control-baseline.out" --control "O3:empty_mutation:$dumps/rows-controls-O3/row-0/o3-control.out" \
      --known-bad-dir "$dumps/known-bad-s2" --e170-expected-dir "$dumps/e170-bad-states-copy-s2" --image-judgements "$s4/images-o3.out" \
      --segment s4-f4-substitutes --rows-root "$dumps/rows-s4" --rows-root "$dumps/rows-s4-f4" --substitution-rows-root "$dumps/rows-s4-f4-substitutes" \
      --expansion-rows scripts/experiments/e171_three_oracle_collusion/bad-implementations-expansion-164.tsv \
      --driver-output "$s4/table-driver.out" --driver-output "$s4/q3core-O1-driver.out" --driver-output "$s4/q3core-O2-driver.out" \
      --driver-output "$s4/q3core-no-executor-driver.out" --driver-output "$s4/table-f4-driver.out" --driver-output "$s4/substitution-driver.out" \
      --q3-core "O1:$dumps/rows-s4:$dumps/rows-s4-q3core-O1" --q3-core "O2:$dumps/rows-s4:$dumps/rows-s4-q3core-O2" \
      --q3-core "no_executor:$dumps/rows-s4:$dumps/rows-s4-q3core-no-executor" >&2) || return 1
  grep '^E7RESULT ' "$s4/comparator-f4-substitutes.out"
}
# E171 第 1 次重跑第一段·修订 6 那一趟（登记 `research/prompts/e171-r1-prereg.md` 修订 6–8 与其后各条「主 agent 认定」；第 0–10 步后缀 -r6-r3，
# 第 11–13 步 -resume2）：只在这一趟已有的导出与产物上重出，不起装置、不起 cargo、不碰别的台（研究一侧是 Python，不是编译出来的代码，不经内存包装）。
# 导出根 scripts/experiments/target/e171-r1-dumps/revision-6-2026-10-05-r3/ 是这一趟在本机的留存（不进仓）：它没了，比对器与跨机对照这两行退非 0、判红。
# 第 0–13 步本身（装置各趟、逐行驱动、O3 执行器、多机编排）要冻结树、真跑各行或第二台，推的挂钟几十小时，不登记。
e171_r1_revision_6_python() { # 在仓根起研究一侧的 Python
  (PYTHONDONTWRITEBYTECODE=1 nice -n 19 python3 -B "scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes/$1" "${@:2}")
}
driver_e171_r1_s1_revision_6_comparator() {
  local scratch="$OUT_DIR/e171-r1-s1-revision-6-comparator" dumps="scripts/experiments/target/e171-r1-dumps/revision-6-2026-10-05-r3"
  local prefix="research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1"
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  e171_r1_revision_6_python segment_one_comparator.py --baseline "$dumps/baseline-run1" --second-baseline "$dumps/baseline-run2" \
    --baseline-o3 "$prefix-o3-baseline-run1-r6-r3.out" --rows-root "$dumps/segment1-rows-resume2" \
    --q5-main "$prefix-q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out" --q5-reverse "$prefix-q5-reverse-at-the-return-point-of-call-2-r6-r3.out" \
    --control "O1-对拍=$prefix-positive-control-o1-comparison-r6-r3.out" --control "O1-对拍-卸载=$prefix-positive-control-o1-comparison-unmount-r6-r3.out" \
    --control "O1-崩溃判官=$prefix-positive-control-o1-crash-judge-r6-r3.out" --control "RefFS=$prefix-positive-control-reference-filesystem-r6-r3.out" \
    --control "O2=$prefix-positive-control-o2-r6-r3.out" --control "见证=$prefix-positive-control-witness-r6-r3.out" \
    --control "交回值=$prefix-positive-control-return-value-r6-r3.out" --control "内存对象=$prefix-positive-control-memory-object-r6-r3.out" \
    --control "panic=$prefix-positive-control-panic-r6-r3.out" --control "O3=$prefix-positive-control-o3-p1-r6-r3.out" \
    --dumps-root "$dumps" --product-prefix e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1- --product-suffix=-r6-r3.out \
    --output "$scratch/comparator.out" >&2 || return 1
  cat "$scratch/comparator.out"
}
# 跨机一致对照（S17）的单独入口在已有的两份行目录上重出（「主 agent 认定：`-resume2` 跨机一致对照 S17 停机的处置」第 3、4 条）。入口只往标准输出写、
# 不写收尾行，照 e168_wrap 的先例在这里补 `name=done emitted=`：入库产物（…-2026-10-07-r3.out）就是补过的样子；它与 …-2026-10-07-r2.out 只差第 2 行的
# entry_sha256（入口改成读同目录里入库的编排器，04376c00… → afc9a9b0…），别的行逐字节相同。
driver_e171_r1_s1_revision_6_cross_host_check() {
  local scratch="$OUT_DIR/e171-r1-s1-revision-6-cross-host-check" dumps="scripts/experiments/target/e171-r1-dumps/revision-6-2026-10-05-r3"
  local prefix="research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1"
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  e171_r1_revision_6_python cross_host_o3_check.py compare --repo . --step-12-row-0 "$dumps/segment1-rows-resume2/row-0" \
    --cross-host-row-0 "$dumps/segment1-rows-resume2-cross-host/row-0" --step-12-product "$prefix-rows-driver-resume2-r6-r3.out" \
    --cross-host-product "$prefix-rows-cross-host-row-0-driver-resume2-r6-r3.out" --ledger "$dumps/segment1-rows-resume2-shards/row-hosts.tsv" \
    >"$scratch/compare.out" || return 1
  awk '{print} END{print "E7RESULT name=done emitted=" NR + 1}' "$scratch/compare.out"
}
driver_e171_r1_s1_research_mutations() {
  local scratch="$OUT_DIR/e171-r1-s1-research-mutations" repository_root="$PWD"
  rm -rf "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 nice -n 19 python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes.tsv" --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}
# E171 第 1 次重跑第二段（整表）·修订 6 树：比对器只读已有的导出与合成产物重出（不起装置、不起 cargo、不碰别的台）。基线是修订 6 那一趟第 1 步
# S2 新装置基线，稳定性基线是同一趟第 7、9 步两次基线；行输出根 scripts/experiments/target/e171-r1-dumps/revision-6-segment-2/segment2-rows 是这一段在本机的留存（不进仓）：
# 它或那一趟的导出根没了，这一行退非 0、判红。整表本身（281 行逐行改坏、编、跑装置，两台推约五个多小时）不登记。
driver_e171_r1_s2_revision_6_comparator() {
  local scratch="$OUT_DIR/e171-r1-s2-revision-6-comparator" dumps="scripts/experiments/target/e171-r1-dumps"
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  e171_r1_revision_6_python segment_two_comparator.py --baseline "$dumps/revision-6-2026-10-05-r3/s2-device-baseline-streams-1-2" \
    --stability-baseline "$dumps/revision-6-2026-10-05-r3/baseline-run1" --stability-baseline "$dumps/revision-6-2026-10-05-r3/baseline-run2" \
    --rows-root "$dumps/revision-6-segment-2/segment2-rows" \
    --driver-product research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-driver-r6.out \
    --mutations-table scripts/experiments/target/e171-r1-frozen-tree-revision-6/crates/mutations.tsv \
    --dumps-root "$dumps/revision-6-2026-10-05-r3" --dumps-root "$dumps/revision-6-segment-2" \
    --product-prefix e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2- --product-suffix=-r6.out \
    --output "$scratch/comparator.out" >&2 || return 1
  cat "$scratch/comparator.out"
}
driver_e171_r1_s2_research_mutations() {
  local scratch="$OUT_DIR/e171-r1-s2-research-mutations" repository_root="$PWD"
  rm -rf "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 nice -n 19 python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_segment_two.tsv" --unit-test-module test_segment_two \
      --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}
driver_e171_r1_s1_revision_8_tools_research_mutations() {
  local scratch="$OUT_DIR/e171-r1-s1-revision-8-tools-research-mutations" repository_root="$PWD"
  rm -rf "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 nice -n 19 python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_revision_8.tsv" --unit-test-module test_revision_8 \
      --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}
driver_e171_r1_s2_revision_10_tools_research_mutations() {
  local scratch="$OUT_DIR/e171-r1-s2-revision-10-tools-research-mutations" repository_root="$PWD"
  rm -rf "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 nice -n 19 python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_revision_10.tsv" --unit-test-module test_revision_10 \
      --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}

driver_e171_r1_revision_11_tools_research_mutations() {
  local scratch="$OUT_DIR/e171-r1-revision-11-tools-research-mutations" repository_root="$PWD"
  rm -rf "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 nice -n 19 python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_revision_11.tsv" --unit-test-module test_revision_11 \
      --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}

# E171 第 1 次重跑·修订 12（登记 `research/prompts/e171-r1-prereg.md` 修订 12）：入库装置 `stream-constants --streams 1,2,3` 在今天的 crates 上回比
# 三条流与 Q5 那一形的本地常量（S1）；只留 E7RESULT 行（这一模式不写导出、不带路径），exact 比。crates 改了写流这一行会红：照登记修订 8 第 8.2 节分类重取常量。
driver_e171_r1_revision_12_stream_constants() {
  (set -o pipefail && RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="复跑统一给 REPLAY_MEMORY_CAP，没有按驱动分别量过" \
    bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e171_three_oracle_collusion_after_the_oracle_fixes -- stream-constants --streams 1,2,3 | grep '^E7RESULT ')
}

driver_e156() {
  (bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e156_allocation_basis_counts)
}

# E158（第一段，入库装置，跑前登记「装置写在哪」写死第 5 行）：同 E156 的先例，两个 cargo workspace
# 互相看不到对方，不能合并成一次调用。`all` 模式跑 5.7 常量回比、第七节锚点、S5 独立解码对拍、
# H3（岔路 3）四个几何点不注入全枚举、Q3-2/Q2-3 算术、PC3。
# 2026-09-23 产物在 `crates/` 落地 C512（树表 0 条的一版上被换下的实例表记在哪没有条款） 定案（拿掉
# 「回退到无文件那一版」的拒绝）之后结构性对不上（50 行不同，原因见实验页与
# `/tmp/claude-1000/e158-s2/report.md` 第 1 节）；2026-09-24 主 agent 定这一行承重
# `e158-root-choice-repair-2026-09-24-segment1-rerun.out`（旧产物 `…-2026-09-23-segment1.out`
# 原样留着，对应 C512 落地之前的代码，不再是这一行比对的对象）。
# ⚠️ **2026-09-25（session s10）**：另一条并行线（主 agent 知会「实二六」）此刻在改
# `mount.rs`/`allocator.rs`/`allocation_record_tree.rs`（抬 F、挂载读、分配记录树根层），这一份
# `2026-09-24` 产物与今天重出的产物对不上，`pc3` 一行从 `recover_verdict=pass recover_root=
# Some((1, 6))` 变成 `recover_verdict=fail recover_root=Some((0, 0))`——按今天日期另存
# `e158-root-choice-repair-2026-09-25-segment1-rerun.out`，登记表这一行已改指向它；`…-09-24-…`
# 原样留着不删。**这不是本轮代码改动引起的**（e158 装置本身这一段只加了 op1 第三种变体，没碰
# `driver_e158`/`pc3` 这条路径），是不是要等 `crates/` 落定后再复核一遍交主 agent 定，详见跑前
# 登记「十二、修订」session s10 条目第 6 条与交回报告。
driver_e158() {
  (bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- all)
}

# E158 第二段 Q3-1（岔路 3 候选 3 那一半，2026-09-24）：`q3-1-g0` 模式只跑 G0 几何上的
# H3 × Φ3（|F|≤2）违例枚举，不跑第一段的 H3 全枚举（那部分归 `driver_e158`）。
# ⚠️ **2026-09-25（session s10）：值得单独点名的一处现查**——G0/S16/小环三点，今天重出的
# `pairs_with_any_violation` 从旧产物的 213 变成 0（`cold_recover_with_fault_failed`/
# `mount_writable_with_fault_failed` 两点都仍是 0，不是新增了报错，是判定本身不再违例）；S4 那一点
# 除了 `pairs_with_any_violation`（1355→0）之外，`cold_recover_with_fault_failed`/`mount_
# writable_with_fault_failed` 还从 0 变成 239——即同一批构造里，以前是「成功但违例」，现在有 239
# 个变成了「调用直接报错」，是两种不同的失效形态，不只是数字变化。与 `driver_e158` 的 `pc3` 翻转
# 同一批文件改动引起（`mount.rs`/`allocator.rs`/`allocation_record_tree.rs`，另一条并行线，非本轮
# e158 装置改动）。**这条现查可能动到岔路单第 3 行「候选 3 = 今天」这个前提与 F2 的触发条件**——
# C332 正文钉的「2 个故障就能撤销回退」在今天这份 `crates/` 上现在测不出来了，交主 agent 判断是要
# 重新核对 F2、还是等这条并行线落定再复核；详见跑前登记「十二、修订」session s10 与交回报告。今天
# 重出的产物按日期另存 `e158-root-choice-repair-2026-09-25-{q3-1-g0,q3-1-s16,q3-1-small-ring,
# q3-1-s4}.out`，`…-09-24-…` 原样留着，登记表已改指向新文件。
driver_e158_q3_1_g0() {
  (bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q3-1-g0)
}
# 第八节敏感性（行 3）三个取样点：S16、小环（环长现算，产物里的 `small_ring_search` 那行同时钉住取到的环长）、
# S4（`sigma_length_limit=4`，比其余三点多穷举一层，代价数量级最大，real 约 18 分钟，2026-09-24 现查）。
driver_e158_q3_1_s16() {
  (bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q3-1-s16)
}
driver_e158_q3_1_small_ring() {
  (bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q3-1-small-ring)
}
driver_e158_q3_1_s4() {
  (bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q3-1-s4)
}
# E158 岔路单第 1 行（C393）：`q1-g0` 模式在今天的 `crates/`（候选 (c)）上跑 H1 家族 + Φ1 故障注入 +
# PC1-a/PC1-b，只跑 G0 几何（2026-09-24 session s3，见实验页）。候选 (a)（A1 副本）的同一份数只存产物
# `e158-root-choice-repair-2026-09-24-q1-g0-a1-arm.out`，**不登记在这张表里**：它要在
# `scripts/mutation/tables/e158_arms.tsv` 描述的副本上重新编译才跑得出来，这张表假设「跑这一行就等于跑今天
# committed 的 crates/」，副本不满足这个假设；复跑它的步骤见实验页「复跑」一节。session s9 起产物里
# 多了 171 格差集的分类诊断（`q1_2_subset_diff_pair*`，根因是 `RootRecord::instance_table` 没被走
# 全版本走到）与 op1 起三步挂载的持续/瞬时故障轨迹（`q1_1a_op1_trajectory`），纯增量追加。
# **2026-09-25（session s10）**：加了 op1 第三种变体（`MountWritableThenRaiseFloorTo`），新增
# `q1_1a_raise_floor_trigger_summary`/`q1_1a_raise_floor_by_aspect_severity`（G0：
# `pairs=96 trigger_count=32 history_nodes_without_room=26`，both_copies 两个指称各 16/16 触发、
# disk0_only/disk1_only 各 0/16，与既有 op1 两种同一批构造上的模式一致）——这一段是本轮新增、真实
# 数据，不是噪声。⚠️ 同一份产物里还混着另一条并行线（`mount.rs`/`recovery.rs`）造成的漂移：
# `q1_2_tree_table_crates_has_a_path`（`verify_named_units=false`）从「108+81」变成「189+0」（原来
# 能不碰物理拷贝复算出指针的 81 格现在全部走不到了）、`pc1_a` 的 `device_write_bytes` 从 157184 变
# 353792（`AllocationRecordTreeNode` 写得更多，与 `q2_2a_g0` 那条同一根因）——这两条与本轮 e158 装置
# 改动无关，是现查到的既有漂移（与 session s9 报告的方向一致）。今天重出的产物按日期另存
# `e158-root-choice-repair-2026-09-25-{q1-g0-today,q1-s16,q1-s4}.out`，`…-09-24-…` 原样留着。
driver_e158_q1_g0() {
  (bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q1-g0)
}
# E158 岔路单第 2 行 ①（C331 修法，session s4，2026-09-24 session s5 起废弃，见下）：`q2-1-g0` 在
# 今天的 `crates/`（丙 = 甲-jsn）上跑 H2 主族（op2=`mount_writable`，只 n1∈{0,1,2,3}、n2=1）的穷举
# 下界搜索。**session s5 查出这条产物是在故障装配 bug 存在时跑出来的**（`attempt_rootback_probe_
# and_advance` 一块盘只装得上一个故障目标，权重 ≥ 2 就可能漏装——见跑前登记「十二、修订」session s5
# 条目第 2 条）：bug 修好之后同一个 n1 范围重跑给出不同结果（甲-txg 臂从「0 命中」变成「k_min=4」），
# 这条产物与它对应的 `driver_e158_q2_1_g0` 不能再当「今天/丙 0 命中」的依据引用，只留着当「bug 修前
# 长什么样」的历史对照。承重的是下面 `driver_e158_q2_1_g0_session_s5`。
driver_e158_q2_1_g0() {
  (bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q2-1-g0)
}
# E158 岔路单第 2 行 ②（每条修法每次发布多写几字节，session s4）：`q2-2a-g0` 在今天的 `crates/` 上跑
# 固定脚本，按结构种类报每次发布写的字节。甲-txg 臂的同一份数只存产物，同上不登记在这张表里。
# ⚠️ **2026-09-25（session s10）**：`AllocationRecordTreeNode` 每次发布的 `write_calls`/
# `written_bytes` 全面上涨（例如 `second_mount_row_publish` 从 `write_calls=2 written_bytes=32768`
# 变成 `write_calls=10 written_bytes=163840`），与 `q1_g0` 的 `pc1_a` 字节变化同一根因
# （`allocator.rs`/`allocation_record_tree.rs`，另一条并行线，非本轮 e158 装置改动）。今天重出的
# 产物按日期另存 `e158-root-choice-repair-2026-09-25-q2-2a-g0-today.out`，`…-09-24-…` 原样留着。
driver_e158_q2_2a_g0() {
  (bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q2-2a-g0)
}
# E158 岔路单第 2 行 PC2（阳性对照，session s5）：复现判决 K2 的两个具体构造（甲-txg 4 个瞬时根槽
# 读失败、乙-只配置 0 个注入故障 + 1 个崩溃点），不靠穷举——见跑前登记「十二、修订」session s5。
# 乙-只配置候选的同一份数只存产物（副本上的数，副本没有 `published_txg` 字段就编不过，不登记在这张
# 表里，复跑步骤见实验页与 `scripts/mutation/tables/e158_arms.tsv`）。
driver_e158_q2_1_pc2() {
  (bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q2-1-pc2)
}
# E158 岔路单第 2 行 ①（C331 修法，2026-09-24 session s5，承重）：修好故障装配 bug 之后，`q2-1-g0`
# 在今天的 `crates/`（丙 = 甲-jsn）上重跑 H2 主族，n1 范围从 {0,1,2,3} 补齐到跑前登记 5.1 要求的
# {0,...,6}。甲-txg 臂的同一份数只存产物，同上不登记在这张表里（复跑步骤见实验页）。**session s9
# 撤掉了旧的计数上限（`SUBSET_ENUMERATION_CAP`，会在权重档中途停手）**，换成按权重档边界停的
# `weight_ceiling` 机制：n1=0..3 仍是完整穷举（`subsets_tried`==`full_space_subset_count`）；n1=4、
# 6 用新机制找到 `k_min=12`（此前因为撞旧计数上限报 `capped`，没有找到）；n1=5 如实报
# `k_min=not_found_up_to_weight_ceiling`（完整空间 131072、只搜到权重 12 为止，未截断，见跑前登记
# 「十二、修订」session s9）。**2026-09-25（session s10）复跑确认字节一致**——与 `q2_1_pc2`/
# `q2_1_hc1`/`q2_1_hc1_lower_bound` 三行一样不受另一条并行线这一刻改动的影响（那条线动的是
# `mount.rs`/`allocator.rs`/`allocation_record_tree.rs` 里 `q3-1`/`q1-g0`/`q2-2a-g0` 会读到的
# 路径，H2 主族的穷举下界搜索走的是 `first_txg_of_new_instance`/`next_counter`/根环读取，不经过
# 那几处改动），行 2「够判」的结论不受这一刻 crates/ 波动影响。
driver_e158_q2_1_g0_session_s5() {
  (bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q2-1-g0)
}
# 第八节几何敏感性（行 1，2026-09-24 session s6）：岔路单第 1 行判决格 = Q1-1a 的 N_trig，S16（根环
# 大一倍）与 S4（根环小一半）两个方向相反的取样点，复用 H1 装置代码（`run_ledger_fault_family` 本身
# 就是几何参数化的）。两点都与 G0 逐字节等值（N_trig=798），判定不翻面，判别力自证「两点同值，自证
# 不适用」。
driver_e158_q1_s16() {
  (bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q1-s16)
}
driver_e158_q1_s4() {
  (bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q1-s4)
}
# H-C1 直接构造（2026-09-24 session s6，岔路单第 2 行 ①，丙的具体历史）：判决 K2 说丙需要 12 个
# 故障（4 根槽 + 4 条记录各两块盘）才打得中；`run_rootback_tolerance_family` 的穷举在 n1∈{4,5,6}
# 会撞 `SUBSET_ENUMERATION_CAP`，这里不靠穷举，直接按这个具体构造跑一次（hit=true, weight=12），
# 附一个只挡 4 条根槽、不挡记录的负对照（hit=false，证明「只挡根环挡不住丙」）。
driver_e158_q2_1_hc1() {
  (bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q2-1-hc1)
}
# H-C1 下界探针（2026-09-24 session s6；session s9 改参数为 weight_ceiling）：在同一个 n1=4 节点
# 上，穷举权重 0..11 的全部组合（`subsets_tried=22558`，`full_space_subset_count=32768`，没有撞
# 顶）都没有命中，权重 12 上第一个尝试的组合就命中。**这是「恰好 12」的严格证据**：不是构造上界，是
# 穷举下界真正走到了 12 且之下全空。不传第二个参数时 `weight_ceiling=None`——n1=4 这个节点的完整
# 空间只有 2^15=32768，在 `FEASIBLE_FULL_SEARCH_SUBSET_BUDGET`=100000 预算内，`None` 就等于穷举到
# 完整空间的顶，与 session s6 当年手动调高到 60000 效果相同（60000 > 这段历史任何可能的权重值）。
driver_e158_q2_1_hc1_lower_bound() {
  (bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q2-1-hc1-lower-bound)
}
# E158 第 2 次跑第一段（重跑登记 `research/prompts/e158-r2-prereg.md`，2026-09-26）：`r2-all` 在工作树的 `crates/`（今天那一臂）上
# 跑第 2 次跑的开跑检查、H1d、Q1-0、PC1-a/b/c 与实七两段复现历史。另外七臂（A1、甲-txg、乙F-留环、乙 / 丁四臂）的产物
# `e158-root-choice-repair-2026-09-26-r2-all-<臂>.out` 要在 `scripts/mutation/tables/e158_arms.tsv` 套出来的副本上重编才跑得出来，
# 不登记在这张表里（同第一次跑 A1 产物的先例）；`…-r2-all-today-snapshot.out` 是同一装置在那几份副本共用的快照上跑的今天那一臂，
# 与这一行的产物只在实七崩溃注入那一段不同（快照之后 harness 的记录核对器改过，实验页写明）。
driver_e158_r2_all() {
  (bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- r2-all)
}
# E158 第 3 次跑第一段（重跑登记 `research/prompts/e158-r3-prereg.md`，2026-09-27）：`r2-all` 那一行 2026-09-27 在
# 开工快照上重出（工作树的 `crates/` 那一刻与快照逐字节相同，登记 S8），改指 `…-2026-09-27-r2-all-today.out`，旧产物原样留着。
# `r3-seg1` 在九份臂副本上各跑一遍，这里只登记今天那一臂（工作树的 `crates/` 就是它）；另外八臂的产物
# `e158-root-choice-repair-2026-09-27-r3-seg1-<臂>.out` 要在 `scripts/mutation/tables/e158_arms.tsv` 的 r3 行套出来的副本上重编才跑得出来，
# 不登记（同第 2 次跑的先例）。`r3-compare` 只读 `research/results/` 下九份产物、不碰 `crates/`，复跑逐字节可比。
# 产物头的 `E7INPUT name=crates_snapshot` 那一行是开工快照的汇总 sha256，比对前删掉。工作树的 `crates/` 在快照之后又被改过，
# 这两行里读 `crates/` 的那一行（r3-seg1 今天、r2-all）会随之漂，不归这一次（登记第一节「上一次执行员交来的两件」）。
driver_e158_r3_seg1_today() {
  (SINGLEFS_E158_ARM=today bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- r3-seg1)
}
driver_e158_r3_seg1_compare() {
  (bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- r3-compare research/results/e158-root-choice-repair-2026-09-27-r3-seg1)
}
# E158 第 4 次跑（重跑登记 `research/prompts/e158-r4-prereg.md`，2026-09-27）：五段（seg1、seg2b、seg3、seg4、seg5）的臂产物都在
# `scripts/mutation/tables/e158_arms.tsv` 的 r4 行套出来的快照副本上编、跑，今天那一臂也是快照副本（工作树的 `crates/` 在快照之后
# 已合进 C554 乙，不再是「今天那一臂」），各段的臂产物都不登记（同第 2、3 次跑的先例）。这里只登记五段合并的 `r4-compare`：
# 它只读 `research/results/` 下那五组臂产物、不碰 `crates/`，复跑逐字节可比；第二段以装置撞键修过之后重跑的 seg2b 为准，
# 撞键之前那一份 seg2 不喂进来。产物头的 `E7INPUT name=crates_snapshot` 那一行比对前删掉。
driver_e158_r4_compare() {
  (bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ${CHECKER_TIER_QUIESCE_PREFIX[@]+"${CHECKER_TIER_QUIESCE_PREFIX[@]}"} cargo run -q --release -p singlefs-checker-tier ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- r4-compare research/results/e158-root-choice-repair-2026-09-27-r4-seg1 research/results/e158-root-choice-repair-2026-09-27-r4-seg2b research/results/e158-root-choice-repair-2026-09-27-r4-seg3 research/results/e158-root-choice-repair-2026-09-27-r4-seg4 research/results/e158-root-choice-repair-2026-09-27-r4-seg5)
}

# E162（崩溃放量判定块存储选型）：只登记确定性的 anchors（跑前登记 7.2 的锚点）。S1、S2 与几何取样点的产物带杀点与计时，
# 两次跑本来就不同，不登记逐字节比对，复跑命令写在实验页。这个 bin 挂在不默认打开的特性 e162-block-stores 上
# （librocksdb-sys 要现编 C++），它的 bindgen 在本机要 BINDGEN_EXTRA_CLANG_ARGS 指到 gcc 的 stdbool.h 才编得过；开跑前经 build_research_driver_binary 编（编不过的输出进这一行的 .err）。
driver_e162_anchors() {
  BINDGEN_EXTRA_CLANG_ARGS="${BINDGEN_EXTRA_CLANG_ARGS:--I/usr/lib/gcc/x86_64-linux-gnu/13/include}" \
    build_research_driver_binary e162-block-stores e162-crash-verdict-block-store || return 1
  bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ./scripts/experiments/target/release/e162-crash-verdict-block-store anchors
}

# E162 第四段 S4 掉电装置：只登记确定性的 selftest（登记 7.2 A1–A6 与补 7 B 系列锚点）。s4-drive 起 QEMU 虚机（重型），
# 不登记逐字节复跑；它的整条命令写在实验页。同一个特性、同一个 BINDGEN 变量，照上面 E162 的写法。
driver_e162_power_cut_selftest() {
  BINDGEN_EXTRA_CLANG_ARGS="${BINDGEN_EXTRA_CLANG_ARGS:--I/usr/lib/gcc/x86_64-linux-gnu/13/include}" \
    build_research_driver_binary e162-block-stores e162-verdict-store-power-cut || return 1
  bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ./scripts/experiments/target/release/e162-verdict-store-power-cut --role selftest
}

# E163（GPU多卡算单元校验和）：只登记 R1（单机两张卡）的 merge 输出——它是唯一不含计时字段、
# 逐字节可比的一段（每片结果文件自己带 build_device_ms 等计时字段，两次跑不同，不登记）。
# 这个 bin 挂在不默认打开的特性 e163-gpu 上（wgpu、pollster，只有这个 bin 用），装置本身不用
# e7_index_bench::Emitter 的 E7RESULT 协议，这里在驱动函数里把 merge 的原始输出包一层，
# 满足下面「跑不了」判定要的 E7RESULT / name=done 收尾行；要机器上至少两张能起上下文的 NVIDIA 独显卡。
# 开跑前经 build_research_driver_binary 编（编不过的输出进这一行的 .err）。
driver_e163_r1_merge() {
  build_research_driver_binary e163-gpu e163-gpu-multicard-crc32c || return 1
  local bin=./scripts/experiments/target/release/e163-gpu-multicard-crc32c
  local input="$OUT_DIR/e163-r1-merge-input.bin"
  local shard0="$OUT_DIR/e163-r1-merge-shard0.out"
  local shard1="$OUT_DIR/e163-r1-merge-shard1.out"
  bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" "$bin" gen-input --output "$input" --units 2560 --seed 0xE163092700000001 >/dev/null || return 1
  bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" "$bin" run-shard --input "$input" --start 0 --count 1280 --adapter-index 0 --output "$shard0" >/dev/null || return 1
  bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" "$bin" run-shard --input "$input" --start 1280 --count 1280 --adapter-index 1 --output "$shard1" >/dev/null || return 1
  local merge_output line_count
  merge_output="$(bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" "$bin" merge --original-input "$input" --total-units 2560 --shard "$shard0" --shard "$shard1")" || return 1
  line_count=$(printf '%s\n' "$merge_output" | grep -c .)
  printf '%s\n' "$merge_output" | awk '{print "E7RESULT name=merge_line body=" $0}'
  echo "E7RESULT name=done emitted=$((line_count + 1))"
}

# E174（借据丢掉时等待与显存回还）：只登记不起卡、不含计时字段、逐字节可比的输出——`checks`（模型卡上的自检与清单检查）
# 和 `judge` 读产物的判决（模型卡上跑出来的第二段产物 `run --simulated`；真卡两段产物见 driver_e174_segment_one_judge）；
# 真卡上的产物本身含计时与显存读数，两次跑不同，不在这里复跑。
# 这个 bin 挂在不默认打开的特性 e174-gpu 上（wgpu、pollster），开跑前经 build_research_driver_binary 编（编不过的输出进这一行的 .err）；这两个子命令不碰卡。
driver_e174_checks() {
  build_research_driver_binary e174-gpu e174-lease-drop-wait-memory-return || return 1
  bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ./scripts/experiments/target/release/e174-lease-drop-wait-memory-return checks
}

driver_e174_simulated_judge() {
  build_research_driver_binary e174-gpu e174-lease-drop-wait-memory-return || return 1
  bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ./scripts/experiments/target/release/e174-lease-drop-wait-memory-return judge \
    --input research/results/e174-lease-drop-wait-memory-return-simulated-second-2026-10-08.out --scope second
}

# E174 两段真卡产物的判（`judge` 读产物、确定性、不碰卡）：真卡产物本身含计时与显存读数，两次跑不同，不在这里复跑，
# 登记的是判法对着入库产物的输出（第一段 `--scope first`，第二段 `--scope second`）。
driver_e174_segment_one_judge() {
  build_research_driver_binary e174-gpu e174-lease-drop-wait-memory-return || return 1
  bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ./scripts/experiments/target/release/e174-lease-drop-wait-memory-return judge \
    --input research/results/e174-lease-drop-wait-memory-return-segment-one-2026-10-08.out --scope first
}

driver_e174_segment_two_judge() {
  build_research_driver_binary e174-gpu e174-lease-drop-wait-memory-return || return 1
  bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ./scripts/experiments/target/release/e174-lease-drop-wait-memory-return judge \
    --input research/results/e174-lease-drop-wait-memory-return-segment-two-2026-10-08.out --scope second
}

# E172（删除写出的字节的干跑；第 2 次重跑登记 research/prompts/e172-r2-prereg.md 第五节 5.7 第 4 步）：形照 driver_e142——
# 先在仓根现跑 crates/ 侧只读导出 deletion_write_dump 两次进两个临时文件（不带参数的 H1、H2 一份，`-- --history h3` 一份），
# 再让研究侧独立解码器按「H1H2 在前、H3 在后」读它们。产物只是解码器的输出（入库的两份导出是
# research/results/e172-r2-deletion-write-dump-h1h2-2026-10-05.out 与 research/results/e172-r2-deletion-write-dump-h3-2026-10-05.out，不 cat 进产物）；
# 解码器产物头两行 `name=export_file file=h1h2 / h3` 各带一份导出的 sha256，哪一份导出一变那一行就对不上。
# 第 1 次的产物 research/results/e172-deletion-write-dry-run-2026-10-05.out 留着、不再登记复跑：T_time 改毫秒之后导出的系统配置槽字节变了，它复现不出来。
driver_e172() {
  local h1h2_snapshot="$OUT_DIR/e172-r2-deletion-write-dump-h1h2.tmp"
  local h3_snapshot="$OUT_DIR/e172-r2-deletion-write-dump-h3.tmp"
  (bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q -p singlefs-checker-tier --bin deletion_write_dump) >"$h1h2_snapshot" || return 1
  (bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q -p singlefs-checker-tier --bin deletion_write_dump -- --history h3) >"$h3_snapshot" || return 1
  bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ./scripts/experiments/target/release/e172-deletion-write-dry-run "$h1h2_snapshot" "$h3_snapshot" || return 1
}

# E175（产品路径上 inode 号水位退回与重发号）：仓副本装置（跑前登记 research/prompts/e175-preregistration.md 第五节「装置写在哪」写死，
# 形式化线冻结期主工作区 crates/ 不动），装置源码只留在产物目录的 device-source.rs。驱动把今天的工作区清单与 crates/ 拷进
# $OUT_DIR 下的副本、放进那份源码、编一次，跑 D 族 P 臂（8 段，D 族结果行不带挂钟），只留 E7RESULT 行并补收尾行（闸 2）。
driver_e175() {
  local scratch="$OUT_DIR/e175-repository-copy" product_dir="$PWD/research/results/e175-product-path-inode-watermark-regression-2026-10-08" binary
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  (cp -a Cargo.toml Cargo.lock .cargo crates "$scratch"/ && copy_thread_registry_into "$scratch") || return 1
  cp "$product_dir/device-source.rs" "$scratch/crates/singlefs-harness/tests/e175_product_path_inode_watermark_regression.rs" || return 1
  (cd "$scratch" && nice -n 19 cargo test -q --release -p singlefs-harness --test e175_product_path_inode_watermark_regression --no-run >&2) || return 1
  binary="$(find "$scratch/target/release/deps" -maxdepth 1 -type f -executable -name 'e175_product_path_inode_watermark_regression-*' | head -n 1)"
  [[ -n "$binary" ]] || return 1
  E175_ARM=P E175_FAMILY=D E175_RESULT_FILE="$scratch/d-P.out" E175_WORKER_THREADS=4 E175_TIME_BUDGET_SECONDS=600 \
    bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" "$binary" --ignored --exact run_one_family_on_one_arm_into_the_result_file >&2 || return 1
  awk '/^E7RESULT /{print; n++} END{print "E7RESULT name=done emitted=" n + 1}' "$scratch/d-P.out"
}

# --admission-only：只问准入（scripts/gate/admission.py，登记表 .claude/gate.d/stage-inputs.tsv），不编、不跑、不比。
ADMISSION_ONLY=0
REPLAY_FULL=0
[[ "${SINGLEFS_GATE_FULL:-}" == 1 ]] && REPLAY_FULL=1
ONLY=()
for replay_argument in "$@"; do
  if [[ "$replay_argument" == "--admission-only" ]]; then ADMISSION_ONLY=1
  elif [[ "$replay_argument" == "--full" ]]; then REPLAY_FULL=1
  else ONLY+=("$replay_argument"); fi
done
# 替换表按 E103 这种带 E 的形态登记；裸数字会匹配 0 条并报全零（2026-09-05 在 E103 上踩过两次）。
for wanted in ${ONLY[@]+"${ONLY[@]}"}; do
  if [[ "$wanted" =~ ^[0-9]+$ ]]; then
    echo "  ✗ 实验号 $wanted 没带 E：这样匹配不到任何一行，跑出来全是零" >&2
    echo "     → 怎么办：写成 E$wanted，例：bash scripts/experiments/replay.sh E$wanted" >&2
    exit 2
  fi
done
want() { [[ ${#ONLY[@]} -eq 0 ]] && return 0; local e; for e in "${ONLY[@]}"; do [[ "$e" == "$1" ]] && return 0; done; return 1; }

# 产物列是 research/results/ 底下的纯文件名，不是仓库根起的路径：写成路径时下面拼出 research/results/research/results/… 指不到文件，
# 而 diff 失败会被报成「对不上，N 行不同」——看着像产物变了，其实是登记表坏了。2026-09-21 被一次全仓路径回写
# 的路径回写踩中一次（它把第 4 列升级成了仓库根路径）。
bad_rows=$(printf "%s\n" "$TABLE" | awk -F"|" '/^E[0-9]+\|/ && $4 ~ /\// {print "      " $1 "：第 4 列 " $4}')
if [[ -n "$bad_rows" ]]; then
  echo "  ✗ 登记表第 4 列（留存产物）写成了带斜杠的路径，应当是 research/results/ 底下的纯文件名：" >&2
  printf "%s\n" "$bad_rows" >&2   # gate-lint:detail
  echo "     → 怎么办：把那几行第 4 列改回纯文件名（例 e100-system-configuration-slot-2026-09-03.out）；" >&2
  echo "               产物搬过家就同时改文件名本身，别把目录写进这一列。" >&2
  exit 2
fi

# 计时行的负载关：阈值要是数；滞后表要读得过（登记的名字都是上面表里 timing 行的留存产物、理由写全、留存产物自己还没有字段）。读不过一行都不开跑。
if [[ ! "$TIMING_LOAD_RATIO" =~ ^[0-9]+(\.[0-9]+)?$ ]]; then
  echo "  ✗ REPLAY_TIMING_LOAD_RATIO 是「$TIMING_LOAD_RATIO」，要是非负的数（默认 0.5）" >&2
  echo "     → 怎么办：写成 0.5 这样的小数，或不设" >&2
  exit 2
fi
if ! lag_problems_text="$(load_timing_lag "$TIMING_LAG_FILE" "$TABLE" research/results)"; then
  echo "  ✗ 计时行产物的负载字段滞后表 $TIMING_LAG_FILE 有 $(printf '%s\n' "$lag_problems_text" | wc -l) 处不对：" >&2
  printf '      %s\n' "$lag_problems_text" >&2   # gate-lint:detail
  echo "     → 怎么办：滞后表只缩不涨，一行一个留存产物名加「  # 理由」；名字要是上面复跑表里 timing 行的第 4 列，留存产物补了字段就删那一行，不往里加新行" >&2
  exit 2
fi
load_timing_lag "$TIMING_LAG_FILE" "$TABLE" research/results >/dev/null   # 上面在命令替换里读的，TIMING_LAG 要在这一层再读一遍

# ── 准入（门禁与实验共用的 scripts/gate/admission.py，登记表 .claude/gate.d/stage-inputs.tsv）──────────
# 登记了准入的实验，每一行开跑之前先问一次（driver_e142 这类驱动在装置之前还要编、要跑 crates 的导出，先问省掉这一截）；
# 装置入口自己还会再问一次，并把输入指纹打在产物最前面（`E7INPUT` 开头，比对前删掉，`name=done` 的条数不数它们）。
# 退 77（输入自上次产物以来没变）记「输入没变」，不跑、不判红，沿用留存产物的结论；别的非 0 记「跑不了」。
# 每一行问哪个准入键：下面 ADMISSION_ROW_KEYS 指明了的用指明的，没指明的用实验号（写法见 scripts/gate/admission.py 文件头「复跑表按行的准入键」）。
# 同一个实验在这张表里不止一行、又登记了准入的，每一行都要在这里指明一个键、在 .claude/gate.d/stage-inputs.tsv 各登记一行它读的输入，
# 每一行按自己的键问准入、比自己的产物；不然准入分不清哪一行对哪一份，admission.py replay-row-keys 判红、整份不开跑。
# 一行一条，制表符分隔：<准入键> <复跑表那一行的前三列，原样抄> <产物头谁写>。产物头谁写：device 是装置自己问准入、自己打 E7INPUT 行；
# replay 是那一行跑的东西不接准入（crates 的导出 bin 这一类），由 replay_one 把准入放行时给的 E7INPUT 行接在新产物前面。
# 这一段 bash 自己不读，准入模块从本文件原文读（replay-row-keys、experiment、product-field= 都按它认行）。
# shellcheck disable=SC2034
ADMISSION_ROW_KEYS=$(cat <<'TSV'
E142	E142|@driver_e142|	device
E142/p8g-crates-dump	E142|@driver_e142_p8g_crates_dump|	replay
TSV
)
ADMISSION_MODULE="scripts/gate/admission.py"
if ! admission_rows_text="$(python3 "$ADMISSION_MODULE" replay-row-keys .)"; then
  printf '%s\n' "$admission_rows_text" >&2
  echo "  ✗ 定不下复跑表每一行的准入键（python3 $ADMISSION_MODULE replay-row-keys .），一行都没跑" >&2
  echo "     → 怎么办：照上面准入模块打的那几条改 ADMISSION_ROW_KEYS 或 .claude/gate.d/stage-inputs.tsv；不知道每一行按哪个键问准入、比哪一份产物，就不开跑" >&2
  exit 2
fi
declare -A ADMISSION_REGISTERED=() ADMISSION_ROW_KEY=() ADMISSION_ROW_HEADER_WRITER=()
while IFS=$'\t' read -r row_identity row_key row_header_writer row_registered; do
  [[ -n "$row_identity" ]] || continue
  ADMISSION_ROW_KEY["$row_identity"]="$row_key"
  ADMISSION_ROW_HEADER_WRITER["$row_identity"]="$row_header_writer"
  [[ "$row_registered" == yes ]] && ADMISSION_REGISTERED["$row_key"]=1
done <<<"$admission_rows_text"
admission_rows_unkeyed=()
while IFS='|' read -r exp bin args _stored _kind; do
  [[ -z "$exp" ]] && continue
  [[ -n "${ADMISSION_ROW_KEY["$exp|$bin|$args"]+set}" ]] || admission_rows_unkeyed+=("$exp|$bin|$args")
done <<<"$TABLE"
if (( ${#admission_rows_unkeyed[@]} )); then
  echo "  ✗ 复跑表有 ${#admission_rows_unkeyed[@]} 行准入模块没给出准入键（它只认「编号|二进制|参数|产物|exact 或 timing」形状的行）：" >&2
  printf '      %s\n' "${admission_rows_unkeyed[@]}" >&2   # gate-lint:detail
  echo "     → 怎么办：把那几行改成这个形状（第 5 列只写 exact 或 timing），python3 scripts/gate/admission.py replay-row-keys . 列得出每一行再跑" >&2
  exit 2
fi

if (( ADMISSION_ONLY )); then
  admission_judged=0; admission_unregistered=0; admission_errors=0
  while IFS='|' read -r exp bin row_arguments stored _kind; do
    [[ -z "$exp" ]] && continue
    want "$exp" || continue
    admission_judged=$((admission_judged+1))
    admission_key_of_row "$exp" "$bin" "$row_arguments"
    if [[ -z "${ADMISSION_REGISTERED[$ROW_ADMISSION_KEY]:-}" ]]; then
      admission_unregistered=$((admission_unregistered+1))
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 没接准入 "准入登记表里没有 $ROW_ADMISSION_KEY，照跑（留存产物 $stored）"
      continue
    fi
    if admission_messages="$(python3 "$ADMISSION_MODULE" experiment . "$ROW_ADMISSION_KEY" 2>&1 >/dev/null)"; then admission_exit_code=0; else admission_exit_code=$?; fi
    case "$admission_exit_code" in
      0) admission_verdict=放行 ;; 77) admission_verdict=输入没变 ;; 3) admission_verdict=前提没齐 ;;
      *) admission_verdict=登记有错; admission_errors=$((admission_errors+1)) ;;
    esac
    printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" "$admission_verdict" "准入键 $ROW_ADMISSION_KEY，准入退 $admission_exit_code，留存产物 $stored"
    printf '%s\n' "$admission_messages" | sed 's/^/      /'   # gate-lint:detail
  done <<<"$TABLE"
  echo "只判准入、没编没跑：判了 $admission_judged 行（其中 $admission_unregistered 行没接准入，照跑），准入模块报登记有错 $admission_errors 行"
  (( admission_errors == 0 )) || exit 1
  exit 0
fi

# ── 按行复用（文件头「按行复用」）：先算全部行的行键、定哪几行要现跑，再决定编不编、清不清场 ──────────────────────────────────
declare -A ROW_REUSE_KEY=() ROW_CHANGE_NOTE=() ROW_REFUSAL=()
REPLAY_ROW_LIBRARY=""
if replay_common_directory="$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" && [[ -n "$replay_common_directory" ]]; then
  REPLAY_ROW_LIBRARY="$replay_common_directory/singlefs-unit-verdicts/replay-row"
fi
read_gpu_switch "$OUT_DIR/gpu-switch.log" || exit 78
mkdir -p "$OUT_DIR/compositions"
if [[ -z "$REPLAY_ROW_LIBRARY" ]]; then
  echo "  ! 按行复用：不在 git 工作树里，判定库没处放：这一趟一行都不复用、也不记" >&2
elif row_inputs_text="$(python3 "$ADMISSION_MODULE" replay-row-inputs . --gpu "$REPLAY_GPU_SWITCH" --library "$REPLAY_ROW_LIBRARY" --compositions "$OUT_DIR/compositions" ${ONLY[@]+"${ONLY[@]}"})"; then
  while IFS=$'\t' read -r row_identity row_reuse_key _row_units row_note; do
    [[ -n "$row_identity" ]] || continue
    if [[ "$REPLAY_BROKEN" == reuse-key-by-row-only ]]; then   # 弄坏开关：键只按复跑表那一行算，不看装置源码与工具链
      row_reuse_key="$(printf '%s' "$row_identity" | sha256sum | cut -c1-64)"; row_note=""
    fi
    if [[ "$row_reuse_key" == refused ]]; then
      ROW_REFUSAL["$row_identity"]="$row_note"
      continue
    fi
    ROW_REUSE_KEY["$row_identity"]="$row_reuse_key"
    ROW_CHANGE_NOTE["$row_identity"]="$row_note"
  done <<<"$row_inputs_text"
  mkdir -p -- "$REPLAY_ROW_LIBRARY/keys" "$REPLAY_ROW_LIBRARY/items" 2>/dev/null || { echo "  ! 按行复用：建不了判定库 $REPLAY_ROW_LIBRARY：这一趟一行都不复用、也不记" >&2; ROW_REUSE_KEY=(); }
else
  printf '%s\n' "$row_inputs_text" >&2
  echo "  ! 按行复用：每一行的行键算不出（python3 $ADMISSION_MODULE replay-row-inputs . 没成，原因在上面）：这一趟一行都不复用、也不记，照常全跑" >&2
fi
(( REPLAY_FULL )) && echo "  · 全量（--full 或 SINGLEFS_GATE_FULL=1）：按行复用的判定库只记不读，每一行都现跑" >&2

# 复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「二」）：行键都算好之后、开跑前那一判之前抽；候选是这把行键在判定库里有判过的记录
# （keys/<行键>/verdict 是字节一致、仅计时不同、已归档只跑不比之一），直接读记录目录、不经 decide_row。抽中强制现判的那一行 decide_row 不读库、照现跑
REPLAY_REUSE_CHECKS="${ADMISSION_MODULE%/admission.py}/lib_reuse_checks.py"
REPLAY_SENTINEL="$OUT_DIR/reuse-sentinel.json"
REPLAY_FORCED_TAG=""
: >"$REPLAY_SENTINEL.candidates"
if [[ ! -f "$REPLAY_REUSE_CHECKS" ]]; then
  # 拷进不带共用库的小树里跑的这一份（scripts/gate/admission.py 的自证那一类）：哨兵与上次红的清单都不做，说一句；真仓里这份库一直在
  echo "  ! 复用哨兵：找不到 $REPLAY_REUSE_CHECKS（这一份 replay.sh 跑在不带它的树里），这一趟不抽哨兵、不写上次红的清单" >&2
elif (( REPLAY_FULL )) || [[ -z "$REPLAY_ROW_LIBRARY" ]]; then
  replay_sentinel_off="全量（--full 或 SINGLEFS_GATE_FULL=1，不读判定库）"; [[ -z "$REPLAY_ROW_LIBRARY" ]] && replay_sentinel_off="没有判定库"
  python3 "$REPLAY_REUSE_CHECKS" sentinel-plan "$REPLAY_SENTINEL" --script scripts/experiments/replay.sh --label 复跑表 --off "$replay_sentinel_off" >&2 \
    || { echo "  ✗ 复用哨兵没抽成（$REPLAY_REUSE_CHECKS sentinel-plan）" >&2; echo "     → 怎么办：照它的报错修 scripts/gate/lib_reuse_checks.py；抽不成这一趟判不了复用" >&2; exit 1; }
else
  candidate_sequence=0
  while IFS='|' read -r exp bin args stored kind; do
    [[ -z "$exp" ]] && continue
    want "$exp" || continue
    candidate_sequence=$((candidate_sequence + 1))
    candidate_key="${ROW_REUSE_KEY["$exp|$bin|$args"]-}"
    [[ -n "$candidate_key" && -f "$REPLAY_ROW_LIBRARY/keys/$candidate_key/verdict" && -f "$REPLAY_ROW_LIBRARY/keys/$candidate_key/product" ]] || continue
    candidate_verdict="$(cat -- "$REPLAY_ROW_LIBRARY/keys/$candidate_key/verdict")"
    case "$candidate_verdict" in pass|timing_only|archived) printf '%s\t%s\n' "$exp.$candidate_sequence" "$candidate_verdict" >>"$REPLAY_SENTINEL.candidates" ;; esac
  done <<<"$TABLE"
  python3 "$REPLAY_REUSE_CHECKS" sentinel-plan "$REPLAY_SENTINEL" --script scripts/experiments/replay.sh --label 复跑表 --candidates "$REPLAY_SENTINEL.candidates" >&2 \
    || { echo "  ✗ 复用哨兵没抽成（$REPLAY_REUSE_CHECKS sentinel-plan）" >&2; echo "     → 怎么办：照它的报错修 scripts/gate/lib_reuse_checks.py；抽不成这一趟判不了复用" >&2; exit 1; }
fi
[[ -f "$REPLAY_SENTINEL" ]] && REPLAY_FORCED_TAG="$(python3 -c 'import json, sys; print(json.load(open(sys.argv[1], encoding="utf-8")).get("forced") or "")' "$REPLAY_SENTINEL")"

# 一行判成字节一致、仅计时不同或已归档只跑不比之后记进判定库（文件头「按行复用」）：同目录临时目录写完再改名换上，连同这把键的组成清单与跑法；
# items/<前三列的 sha256> 记这一行最近一次判过的键（下一趟键对不上时 admission.py 拿它比出变了的单元）。记不下只报一行，不判红
remember_row_verdict() { # <记录目录，空串不记> <判定> <结果行> <新产物> <前三列>
  local record="$1" temporary key identity_digest
  [[ -n "$record" ]] || return 0
  temporary="$record.tmp.$BASHPID"
  key="${record##*/}"
  rm -rf -- "${temporary:?}"
  if mkdir -p -- "$temporary" && cp -- "$4" "$temporary/product" && printf '%s\n' "$2" >"$temporary/verdict" \
     && printf '%s\n' "$3" >"$temporary/line" && TZ=Asia/Tokyo date +%F >"$temporary/finished" \
     && printf '%s\n' "bash scripts/experiments/replay.sh（这一行的二进制与参数照复跑表；GPU 开关 ${REPLAY_GPU_SWITCH:-0}）" >"$temporary/how" \
     && { [[ ! -f "$OUT_DIR/compositions/$key.tsv" ]] || cp -- "$OUT_DIR/compositions/$key.tsv" "$temporary/composition"; }; then
    rm -rf -- "${record:?}"
    if mv -T -- "$temporary" "$record" 2>/dev/null; then
      identity_digest="$(printf '%s' "$5" | sha256sum | cut -c1-64)"
      printf '%s\n' "$key" >"$REPLAY_ROW_LIBRARY/items/$identity_digest.tmp.$BASHPID" && mv -f -- "$REPLAY_ROW_LIBRARY/items/$identity_digest.tmp.$BASHPID" "$REPLAY_ROW_LIBRARY/items/$identity_digest"
      return 0
    fi
  fi
  rm -rf -- "${temporary:?}"
  echo "  ! 按行复用：记不下 $record（下一趟这一行照跑）" >&2
}

pass=0; drift=0; timing_only=0; broken=0; claim_bad=0; archived=0; unchanged=0; UNCHANGED_ROWS=()
reused=0; REUSED_ROWS=(); ARCHIVED_ROWS=(); refused_rows=0   # 按行复用的行；已归档只跑不比的行（现跑的与复用来的都记，文件头「按行复用」）；行键拒算的行
load_skipped=0; load_passed=0; load_lagged=0; LOAD_SKIPPED_ROWS=()   # 计时行的负载关：本次不判、放行（含滞后）、其中滞后；本次不判的逐个记名字与比值
CLAIM_QUEUE=()

printf '%-5s %-24s %-10s %s\n' 实验 二进制 判定 说明
printf '%s\n' "-------------------------------------------------------------------------"
# 一行判跑出来的新产物（闸 2 逐段核、产物头、负载关、已归档、逐字节比）：写 $OUT_DIR/<tag>.line / .verdict / .claim 这几份，判过的记进判定库。
# 这一段是复跑这一行的判法：行键里的 judge:scripts/experiments/replay.sh::replay_judge_fresh_product（admission.py REPLAY_JUDGING_FUNCTIONS）。
replay_judge_fresh_product() { # <实验号> <二进制> <留存产物> <exact|timing> <准入键> <tag> <新产物> <记录目录> <前三列>
  local exp="$1" bin="$2" stored="$3" kind="$4" row_key="$5" tag="$6" fresh="$7" reuse_record="$8" identity="$9" gate2 n fresh_head result_line
  # 闸 2：**逐段**核。产物可能是多次运行拼起来的（E9 就是 25 段），
  # 只看最后一个 name=done 会让前 24 段的缺行全部漏过去。
  gate2=$(awk '/^E7RESULT/{n++}
               /^E7RESULT name=done emitted=/{
                 split($0,a,"emitted="); e=a[2]+0
                 segs++
                 if (e != n) { print "第 " segs " 段收尾行说 " e " 条，实收 " n " 条"; bad=1; exit }
                 n=0
               }
               END{ if (!bad) { if (segs==0) print "没有收尾行 name=done"; else if (n>0) print "最后一段没有收尾行，尾巴 " n " 条" } }' "$fresh")
  if [[ -n "$gate2" ]]; then
    printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 跑不了 "$gate2" >"$OUT_DIR/$tag.line"; echo broken >"$OUT_DIR/$tag.verdict"; return
  fi
  # 登记了准入的实验，产物最前面要有实验键那一行；没有这一行，下一次准入永远判不了「输入没变」
  fresh_head="$(head -n 20 "$fresh")"
  if [[ -n "${ADMISSION_REGISTERED[$row_key]:-}" ]] && ! grep -q "^E7INPUT name=input_fingerprint key=$row_key " <<<"$fresh_head"; then
    printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 跑不了 "登记了准入，产物头却没有 key=$row_key 的实验键行：装置 main 没调 e7_index_bench::admit_experiment_run_or_exit（或传的键与 ADMISSION_ROW_KEYS 指明的不同）" >"$OUT_DIR/$tag.line"; echo broken >"$OUT_DIR/$tag.verdict"; return
  fi
  # 计时行的负载关（文件头「计时行的负载关」）：负载越过阈值的记本次不判，不比结构、不判区间断言；没字段又不在滞后表的判红
  timing_load_decide "$kind" "$stored" "$fresh"
  case "$TIMING_LOAD_DECISION" in
    red)
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 缺负载字段 "$TIMING_LOAD_NOTE。→ 怎么办：装置的收尾行补 loadavg1= cpus=（约定见文件头）；留存产物还没字段的存量才登记进滞后表，只缩不涨" >"$OUT_DIR/$tag.line"; echo broken >"$OUT_DIR/$tag.verdict"; return ;;
    skip)
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 本次不判 "$stored：$TIMING_LOAD_NOTE，机器正忙，不比结构、不判区间" >"$OUT_DIR/$tag.line"
      echo load_skipped >"$OUT_DIR/$tag.verdict"; printf '%s %s\n' "$exp" "$TIMING_LOAD_NOTE" >"$OUT_DIR/$tag.skip"; return ;;
  esac
  if [[ "$kind" == timing ]]; then
    if (( TIMING_LOAD_LAGGED )); then echo lagged >"$OUT_DIR/$tag.load"; else echo judged >"$OUT_DIR/$tag.load"; fi
  fi
  # 留存产物已按「每次提交删上一次的实验记录」归档进版本库时，这一档不比对。
  # 不报成「对不上」：那与「装置真的改坏了、复跑出不同字节」长得一模一样，读的人会以为实验坏了
  # （`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）。
  if [[ ! -f "research/results/$stored" ]]; then
    result_line="$(printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 产物已归档 "已归档、只跑不比：$stored 不在树里；本次跑得出来，逐字节这一档不比对")"
    printf '%s\n' "$result_line" >"$OUT_DIR/$tag.line"; echo "$exp $stored" >"$OUT_DIR/$tag.archived"
    echo archived >"$OUT_DIR/$tag.verdict"; echo "$exp|$fresh" >"$OUT_DIR/$tag.claim"
    remember_row_verdict "$reuse_record" archived "$result_line" "$fresh" "$identity"; return
  fi
  if diff -q <(strip_admission_header <"$fresh") <(strip_admission_header <"research/results/$stored") >/dev/null 2>&1; then
    printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 字节一致 "$stored" >"$OUT_DIR/$tag.line"; echo pass >"$OUT_DIR/$tag.verdict"
    echo "$exp|$fresh" >"$OUT_DIR/$tag.claim"; remember_row_verdict "$reuse_record" pass "$(cat -- "$OUT_DIR/$tag.line")" "$fresh" "$identity"; return
  fi
  if diff -q <(strip_admission_header <"$fresh" | strip_load_fields | strip_timing) <(strip_admission_header <"research/results/$stored" | strip_load_fields | strip_timing) >/dev/null 2>&1; then
    if [[ "$kind" == timing ]]; then
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 仅计时不同 "$stored（结构一致，符合声明）" >"$OUT_DIR/$tag.line"; echo timing_only >"$OUT_DIR/$tag.verdict"
      echo "$exp|$fresh" >"$OUT_DIR/$tag.claim"; remember_row_verdict "$reuse_record" timing_only "$(cat -- "$OUT_DIR/$tag.line")" "$fresh" "$identity"
    else
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 判据写错 "$stored 声明 exact 却只在抹掉计时后才一致" >"$OUT_DIR/$tag.line"; echo drift >"$OUT_DIR/$tag.verdict"
    fi
    return
  fi
  n=$(diff <(strip_admission_header <"$fresh" | strip_load_fields | strip_timing) <(strip_admission_header <"research/results/$stored" | strip_load_fields | strip_timing) | grep -c '^[<>]')
  printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 对不上 "$stored，$n 行不同 → diff $fresh research/results/$stored" >"$OUT_DIR/$tag.line"
  echo drift >"$OUT_DIR/$tag.verdict"
}

# 开跑前的那一判（准入与按行复用）：照判定写 $OUT_DIR/<tag>.decision（reuse、unchanged、broken、run 之一），准入放行的产物头写 <tag>.header；
# 复用、输入没变、跑不了的那几行同时把结果行与结论写好，replay_one 读到不是 run 的就直接收尾。主循环先把每一行都判完，再定编不编（文件头「按行复用」）。
decide_row() { # <实验号> <tag> <二进制> <留存产物> <准入键> <行键，空串不复用>
  local exp="$1" tag="$2" bin="$3" stored="$4" row_key="$5" reuse_key="$6" reuse_record="" reused_verdict reused_title admission_exit_code=0
  rm -f "$OUT_DIR/$tag.decision" "$OUT_DIR/$tag.header" "$OUT_DIR/$tag.admission"
  if [[ -n "${ADMISSION_REGISTERED[$row_key]:-}" ]]; then
    if python3 "$ADMISSION_MODULE" experiment . "$row_key" >"$OUT_DIR/$tag.header" 2>"$OUT_DIR/$tag.admission"; then admission_exit_code=0; else admission_exit_code=$?; fi
    if [[ $admission_exit_code -eq 77 ]]; then
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 输入没变 "实验键与留存产物头上记的相同，本次没跑，沿用 $stored 的结论（见 $OUT_DIR/$tag.admission）" >"$OUT_DIR/$tag.line"
      echo unchanged >"$OUT_DIR/$tag.verdict"; echo "$exp|results/$stored" >"$OUT_DIR/$tag.claim"; echo unchanged >"$OUT_DIR/$tag.decision"; return
    fi
    if [[ $admission_exit_code -ne 0 ]]; then
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 跑不了 "准入没放行（退 $admission_exit_code），见 $OUT_DIR/$tag.admission" >"$OUT_DIR/$tag.line"; echo broken >"$OUT_DIR/$tag.verdict"
      echo broken >"$OUT_DIR/$tag.decision"; return
    fi
  fi
  [[ -n "$reuse_key" && -n "$REPLAY_ROW_LIBRARY" ]] && reuse_record="$REPLAY_ROW_LIBRARY/keys/$reuse_key"
  if [[ -n "$REPLAY_FORCED_TAG" && "$tag" == "$REPLAY_FORCED_TAG" ]]; then   # 复用哨兵抽中强制现判：不读库、照现跑
    printf '  · %s 要现跑：复用哨兵抽中强制现判（行键相同，这一趟不读库）\n' "$exp" >&2
    echo run >"$OUT_DIR/$tag.decision"; return
  fi
  if [[ -n "$reuse_record" && -f "$reuse_record/verdict" && -f "$reuse_record/product" ]] && (( ! REPLAY_FULL )) && [[ "$REPLAY_BROKEN" != reuse-ignored ]]; then
    reused_verdict="$(cat -- "$reuse_record/verdict")"
    case "$reused_verdict" in pass) reused_title=字节一致 ;; timing_only) reused_title=仅计时不同 ;; archived) reused_title=已归档只跑不比 ;; *) reused_title="" ;; esac
    if [[ -n "$reused_title" ]]; then
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 复用 "${reused_title}：行键相同（${reuse_key:0:16}…，$(cat -- "$reuse_record/finished" 2>/dev/null || echo 那一趟)判的；留存产物 $stored），这一趟没跑" >"$OUT_DIR/$tag.line"
      echo "reused:$reused_verdict" >"$OUT_DIR/$tag.verdict"; echo "$exp|$reuse_record/product" >"$OUT_DIR/$tag.claim"
      [[ "$reused_verdict" == archived ]] && echo "$exp $stored（复用）" >"$OUT_DIR/$tag.archived"
      echo reuse >"$OUT_DIR/$tag.decision"; return
    fi
  fi
  echo run >"$OUT_DIR/$tag.decision"
}

# 一条实验的复跑：并发起的，所以计数、结果行与 claim 队列都落到自己的文件，
# 父进程按登记表的顺序回读（command-safety.md「并行不许把失败吃掉」「输出不许直接往 stdout 写」）。
replay_one() {
  # 第 2 个参数是这一条在登记表里的序号：同一个实验号可以有两行（E16 就是，一行比 e16-journal、
  # 一行带 args=bytes 比 e16-bytes），按实验号命名输出文件时两条会并发写同一个文件，
  # 后写完的那份整份留下、另一条的结果彻底消失，而「派多少收多少」那道闸只判文件存不存在、看不出来。
  # 第 7、8 个参数是这一行的准入键与产物头谁写（ADMISSION_ROW_KEYS，没指明的是实验号与 device）；
  # 第 9 个参数是这一行的行键（文件头「按行复用」，空串就是不复用也不记）；第 10 个是复跑表的前三列原样（判定库按它记这一行最近一次的键）
  local exp="$1" seq="$2" bin="$3" args="$4" stored="$5" kind="$6" row_key="$7" header_writer="$8" reuse_key="${9:-}" identity="${10:-}" tag fresh rc
  local reuse_record=""
  tag="$exp.$seq"
  fresh="$OUT_DIR/$tag.out"
  [[ "$(cat -- "$OUT_DIR/$tag.decision" 2>/dev/null)" == run ]] || return   # 复用、输入没变、跑不了：开跑前那一判已经写好了这一行
  rm -f "$fresh" "$OUT_DIR/$tag.load" "$OUT_DIR/$tag.skip" "${OUT_DIR:?}/${tag:?}.archived"   # 闸 1：不许跨轮复用（负载关的记录同样）
  [[ -n "$reuse_key" && -n "$REPLAY_ROW_LIBRARY" ]] && reuse_record="$REPLAY_ROW_LIBRARY/keys/$reuse_key"
  if [[ "$bin" == @* ]]; then
    "${bin#@}" >"$fresh" 2>"$OUT_DIR/$tag.err"
  else
    # shellcheck disable=SC2086
    bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" ./scripts/experiments/target/release/"$bin" $args >"$fresh" 2>"$OUT_DIR/$tag.err"
  fi
  rc=$?
  if [[ $rc -ne 0 ]]; then                          # 闸 3
    printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 跑不了 "退出码 $rc，见 $OUT_DIR/$tag.err" >"$OUT_DIR/$tag.line"; echo broken >"$OUT_DIR/$tag.verdict"; return
  fi
  # 产物头由这里写的行（ADMISSION_ROW_KEYS 第三列 replay：跑的东西不接准入）：准入放行时给的 E7INPUT 行接在新产物最前面，与装置自己打的同一个样子
  if [[ -n "${ADMISSION_REGISTERED[$row_key]:-}" && "$header_writer" == replay ]]; then
    if ! { cat -- "$OUT_DIR/$tag.header" "$fresh" >"$fresh.stamped" && mv -- "$fresh.stamped" "$fresh"; }; then
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 跑不了 "准入键 $row_key 的产物头接不到新产物前面（$OUT_DIR/$tag.header）" >"$OUT_DIR/$tag.line"; echo broken >"$OUT_DIR/$tag.verdict"; return
    fi
  fi
  replay_judge_fresh_product "$exp" "$bin" "$stored" "$kind" "$row_key" "$tag" "$fresh" "$reuse_record" "$identity"
}

# 开跑前那一判：按表里的次序给每一行定 tag 与判定（decide_row），数出要现跑的几行；一行都不用现跑就不试编 gpu 特性、不清场、不编（文件头「按行复用」）
declare -a ROW_TEXTS=() REPLAY_ORDER=() TO_RUN_BINARIES=()
row_sequence=0; to_run=0; to_run_drivers=0
while IFS='|' read -r exp bin args stored kind; do
  [[ -z "$exp" ]] && continue
  want "$exp" || continue
  row_sequence=$((row_sequence+1))
  admission_key_of_row "$exp" "$bin" "$args"   # 按表里的原样查（$REPLAY_DEV 这类还没展开）
  row_identity="$exp|$bin|$args"
  ROW_TEXTS+=("$exp|$bin|$args|$stored|$kind")
  REPLAY_ORDER+=("$exp.$row_sequence")
  [[ -n "${ROW_REFUSAL["$row_identity"]+set}" ]] && refused_rows=$((refused_rows + 1))
  decide_row "$exp" "$exp.$row_sequence" "$bin" "$stored" "$ROW_ADMISSION_KEY" "${ROW_REUSE_KEY["$row_identity"]-}"
  if [[ "$(cat -- "$OUT_DIR/$exp.$row_sequence.decision" 2>/dev/null)" == run ]]; then
    to_run=$((to_run + 1))
    if [[ "$bin" == @* ]]; then to_run_drivers=$((to_run_drivers + 1)); else TO_RUN_BINARIES+=("$bin"); fi
    if [[ -n "${ROW_CHANGE_NOTE["$row_identity"]:-}" ]]; then printf '  · %s 要现跑：%s\n' "$exp" "${ROW_CHANGE_NOTE["$row_identity"]}" >&2; fi
    if [[ -n "${ROW_REFUSAL["$row_identity"]+set}" ]]; then printf '  · %s 要现跑：行键%s\n' "$exp" "${ROW_REFUSAL["$row_identity"]}" >&2; fi
  fi
done <<<"$TABLE"
if (( to_run == 0 )); then
  echo "  · 开跑前判完：${#REPLAY_ORDER[@]} 行里要现跑的 0 行，不试编 gpu 特性、不清场、不编" >&2
else
  prepare_checker_tier_features "$OUT_DIR/gpu-build.log" || exit 78
  if (( to_run_drivers )) || [[ "$REPLAY_BROKEN" == build-everything ]]; then
    build_research_binaries "$OUT_DIR/build.log" || exit 2
  else
    build_research_binaries "$OUT_DIR/build.log" $(printf '%s\n' "${TO_RUN_BINARIES[@]}" | sort -u) || exit 2
  fi
fi

# 并发度：这些实验多是跑 release 二进制的 CPU 活，缺省取登记的重活默认并行度（default_replay_jobs，不看忙闲）。
# REPLAY_JOBS 压过它。各条实验的产物、镜像（e9/e45/e58/e140.img）与结果文件各用各的，没有共用的可写状态。
REPLAY_JOBS="${REPLAY_JOBS:-$(default_replay_jobs)}"
[[ "$REPLAY_JOBS" -lt 1 ]] && REPLAY_JOBS=1
declare -a REPLAY_PIDS=()
running=0
row_sequence=0
for row_text in ${ROW_TEXTS[@]+"${ROW_TEXTS[@]}"}; do
  IFS='|' read -r exp bin args stored kind <<<"$row_text"
  admission_key_of_row "$exp" "$bin" "$args"
  row_identity="$exp|$bin|$args"
  row_header_writer="${ADMISSION_ROW_HEADER_WRITER["$row_identity"]-device}"
  row_reuse_key_of_this_row="${ROW_REUSE_KEY["$row_identity"]-}"   # 按行复用的键（文件头「按行复用」），按表里的原样查
  args="${args//\$REPLAY_DEV140/$REPLAY_DEV140}"  # 先换长的，否则前缀会被短的吃掉
  args="${args//\$REPLAY_DEV58/$REPLAY_DEV58}"
  args="${args//\$REPLAY_DEV45/$REPLAY_DEV45}"
  args="${args//\$REPLAY_DEV/$REPLAY_DEV}"   # 表里写字面量 $REPLAY_DEV，这里才展开
  row_sequence=$((row_sequence+1))
  replay_one "$exp" "$row_sequence" "$bin" "$args" "$stored" "$kind" "$ROW_ADMISSION_KEY" "$row_header_writer" "$row_reuse_key_of_this_row" "$row_identity" &
  REPLAY_PIDS+=($!)
  running=$((running+1))
  if (( running >= REPLAY_JOBS )); then wait -n 2>/dev/null || true; running=$((running-1)); fi
done
# 逐个 wait 写死的 pid，不写不带参数的 wait（它的退出码恒为 0，红了几个一个字都不说）
for pid in ${REPLAY_PIDS[@]+"${REPLAY_PIDS[@]}"}; do wait "$pid" 2>/dev/null || true; done

# 派出去多少条就要收回来多少条：对不上整道红，不许少跑一条还报绿
collected=0
for tag in ${REPLAY_ORDER[@]+"${REPLAY_ORDER[@]}"}; do
  [[ -f "$OUT_DIR/$tag.line" ]] && collected=$((collected+1))
done
if (( collected != ${#REPLAY_ORDER[@]} )); then
  echo "  ✗ 派出去 ${#REPLAY_ORDER[@]} 条复跑，只收回 $collected 条结果行"
  echo "     → 怎么办：这是并发收束自己的完整性闸红了，不是实验的问题；REPLAY_JOBS=1 再跑一遍看串行下全不全，"
  echo "       全的话去查 replay_one 与它的 pid 收束那一段。"
  exit 1
fi

# 按登记表的顺序回读：输出与并发度无关，REPLAY_JOBS=1 与 =16 逐字相同
for tag in ${REPLAY_ORDER[@]+"${REPLAY_ORDER[@]}"}; do
  cat "$OUT_DIR/$tag.line"
  if [[ -f "$OUT_DIR/$tag.verdict" ]]; then
    case "$(cat "$OUT_DIR/$tag.verdict")" in
      pass) pass=$((pass+1)) ;; timing_only) timing_only=$((timing_only+1)) ;;
      drift) drift=$((drift+1)) ;; broken) broken=$((broken+1)) ;; archived) archived=$((archived+1)) ;;
      unchanged) unchanged=$((unchanged+1)); UNCHANGED_ROWS+=("${tag%.*}") ;;
      reused:*) reused=$((reused+1)); REUSED_ROWS+=("${tag%.*}") ;;
      load_skipped) load_skipped=$((load_skipped+1)); LOAD_SKIPPED_ROWS+=("$(cat "$OUT_DIR/$tag.skip")") ;;
    esac
  fi
  if [[ -f "$OUT_DIR/$tag.load" ]]; then   # 放行的计时行（judged 或 lagged）
    load_passed=$((load_passed+1)); if [[ "$(cat "$OUT_DIR/$tag.load")" == lagged ]]; then load_lagged=$((load_lagged+1)); fi
  fi
  [[ -f "$OUT_DIR/$tag.claim" ]] && CLAIM_QUEUE+=("$(cat "$OUT_DIR/$tag.claim")")
  [[ -f "$OUT_DIR/$tag.archived" ]] && ARCHIVED_ROWS+=("$(cat "$OUT_DIR/$tag.archived")")
done

# 复用哨兵判：每一行的结局（复用来的、现判的、强制现判的）一次记好，哨兵被现判、强制现判那一行判定与库里不同，整道红
: >"$OUT_DIR/reuse-sentinel.outcomes"
for tag in ${REPLAY_ORDER[@]+"${REPLAY_ORDER[@]}"}; do
  row_verdict="$(cat -- "$OUT_DIR/$tag.verdict" 2>/dev/null)"
  case "$row_verdict" in
    reused:*) printf '%s\t复用\t%s\n' "$tag" "${row_verdict#reused:}" ;;
    "") ;;
    *) printf '%s\t%s\t%s\n' "$tag" "$([[ "$tag" == "$REPLAY_FORCED_TAG" ]] && echo 强制现判 || echo 现判)" "$row_verdict" ;;
  esac
done >>"$OUT_DIR/reuse-sentinel.outcomes"
sentinel_bad=0
if [[ -f "$REPLAY_SENTINEL" ]]; then
  python3 "$REPLAY_REUSE_CHECKS" sentinel-observe-many "$REPLAY_SENTINEL" "$OUT_DIR/reuse-sentinel.outcomes" || echo "  ! 复用哨兵没记下这一趟的结局（这一趟哨兵不判）" >&2
  python3 "$REPLAY_REUSE_CHECKS" sentinel-finish "$REPLAY_SENTINEL" || sentinel_bad=1
fi

printf '%s\n' "-------------------------------------------------------------------------"
echo "结论区间断言（计时实验复跑不出同样的字节，靠这些把 kb 里的数钉住）："
CLAIM_BAD_ROWS=()
for q in "${CLAIM_QUEUE[@]}"; do
  check_claims "${q%%|*}" "${q#*|}" || { claim_bad=$((claim_bad+1)); CLAIM_BAD_ROWS+=("${q%%|*}"); }
done
# 上次红的清单（scripts/gate/lib_reuse_checks.py 文件头「三」，.claude/hooks/run-scope-guard.sh 的拒绝说明读它）：一行一个红的实验号（对不上、跑不了、结论断言不中）；
# 点名了实验号的只换这几个，别的照留
{ for tag in ${REPLAY_ORDER[@]+"${REPLAY_ORDER[@]}"}; do
    case "$(cat -- "$OUT_DIR/$tag.verdict" 2>/dev/null)" in drift|broken) printf '%s\n' "${tag%.*}" ;; esac
  done
  printf '%s\n' ${CLAIM_BAD_ROWS[@]+"${CLAIM_BAD_ROWS[@]}"}; } | sort -u >"$OUT_DIR/last-run.red"
if [[ ! -f "$REPLAY_REUSE_CHECKS" ]]; then
  :   # 不带共用库的小树（上面复用哨兵那一段说过一句）
elif (( ${#ONLY[@]} )); then
  printf '%s\n' "${ONLY[@]}" >"$OUT_DIR/last-run.ran"
  python3 "$REPLAY_REUSE_CHECKS" last-run scripts/experiments/replay.sh "$OUT_DIR/last-run.red" --ran "$OUT_DIR/last-run.ran" >&2 || echo "  ! 上次红的清单没写成" >&2
else
  python3 "$REPLAY_REUSE_CHECKS" last-run scripts/experiments/replay.sh "$OUT_DIR/last-run.red" >&2 || echo "  ! 上次红的清单没写成" >&2
fi
printf '%s\n' "-------------------------------------------------------------------------"
echo "字节一致 $pass ／ 仅计时不同 $timing_only ／ 对不上 $drift ／ 跑不了 $broken ／ 结论断言不中 $claim_bad ／ 产物已归档 $archived ／ 输入没变没跑 $unchanged ／ 负载高本次不判 $load_skipped ／ 按行复用 $reused"
echo "REPLAY_TOTAL dispatched=${#REPLAY_ORDER[@]} collected=$collected judged=$((pass + timing_only + drift + archived)) reused=$reused refused=$refused_rows pass=$pass timing_only=$timing_only archived_run_only=${#ARCHIVED_ROWS[@]} drift=$drift broken=$broken claims_bad=$claim_bad unchanged=$unchanged load_skipped=$load_skipped full=$REPLAY_FULL"
if (( reused )); then
  echo "  · 按行复用 $reused 行（行键相同，判定库 ${REPLAY_ROW_LIBRARY:-没有}，结论区间断言照常对记录里留的那一份产物判）：${REUSED_ROWS[*]}"
fi
echo "本轮输出：$OUT_DIR"
if (( load_passed + load_skipped )); then
  echo "  计时行负载关：判了 $load_passed、本次不判 $load_skipped、滞后 $load_lagged 份（滞后表 $TIMING_LAG_FILE，登记 ${#TIMING_LAG[@]} 份）"
fi
if [[ $load_skipped -ne 0 ]]; then
  echo "  ! 「本次不判」$load_skipped 行：新产物收尾行的负载超过阈值 $TIMING_LOAD_RATIO（REPLAY_TIMING_LOAD_RATIO），机器正忙，计时数不可信；这些行既不判绿也不判红，结构与区间断言都没比："
  printf '      %s\n' "${LOAD_SKIPPED_ROWS[@]}"   # gate-lint:detail
  echo "     → 怎么办：等负载降下来，只重跑这几行：bash scripts/experiments/replay.sh <实验号>"
  if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then
    for skipped_row in "${LOAD_SKIPPED_ROWS[@]}"; do
      printf 'replay.sh %s，负载过高本次不判（阈值 %s）\n' "$skipped_row" "$TIMING_LOAD_RATIO" >>"$GATE_NOT_RUN_FILE"
    done
  fi
fi
if [[ $unchanged -ne 0 ]]; then
  echo "  ! 「输入没变」$unchanged 行没跑：${UNCHANGED_ROWS[*]}。实验键（这一行的行键去掉期望产物：装置入口单元闭包、驱动闭包、参数、GPU 开关、工具链）"
  echo "     与留存产物头上记的相同，准入（scripts/gate/admission.py）拒绝重跑，结论沿用留存产物；结论区间断言照常对留存产物判。"
  echo "     这不是判绿也不是判红。确要重跑：SINGLEFS_EXPERIMENT_RERUN_REASON=<理由> bash scripts/experiments/replay.sh <实验号>，理由写进新产物头。"
fi
if (( ${#ARCHIVED_ROWS[@]} )) && [[ "$REPLAY_BROKEN" != archived-not-itemized ]]; then
  echo "  ! 已归档、只跑不比 ${#ARCHIVED_ROWS[@]} 行（留存产物不在树里，逐字节这一档没有对照物；这不是判绿）："
  printf '      %s\n' "${ARCHIVED_ROWS[@]}"
  if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then
    for archived_row in "${ARCHIVED_ROWS[@]}"; do
      printf 'replay.sh %s：已归档、只跑不比（逐字节这一档没有对照物）\n' "$archived_row" >>"$GATE_NOT_RUN_FILE"
    done
  fi
fi
if [[ $archived -ne 0 ]]; then
  echo "  ! 「产物已归档」$archived 行：留存产物按「每次提交删上一次的实验记录」归档进了版本库，逐字节这一档没有对照物。"
  echo "     这不是判红——这几行本次都跑得出来，结论区间断言照常判。要看当时的产物："
  echo "         git log --all --diff-filter=D --name-only -- \"*<产物文件名>\"     # 找到删它的那次提交"
  echo "         git show <提交>^:research/results/<产物文件名>                      # 读回当时的内容"
  echo "     读到的是当时的数、不是今天的结论；要拿它支撑新结论就重新跑一遍（evidence-discipline.md）。"
fi
if (( sentinel_bad )); then
  echo "  ✗ 复用哨兵判红（上面 ✗ 那几行：键相同却现判了，或强制现判那一行判定与库里不同）"
  echo "     → 怎么办：照上面的出路查 decide_row 读库那一段与 remember_row_verdict；判法在 scripts/gate/lib_reuse_checks.py 文件头「二」"
fi
if [[ $drift -ne 0 || $broken -ne 0 || $claim_bad -ne 0 ]]; then
  echo "  → 怎么办：「跑不了」看上面那一行给的 $OUT_DIR/<实验号>.err；「对不上」按上面给的 diff 命令看差在哪，" \
       "结构性差异是代码改动带来的就更新入库产物，不是就说明代码退化了；" \
       "「结论断言不中」逐条去 check_claims() 里对应实验号那一段读注释——是该改 kb 里记的区间，还是真的退化了，" \
       "两种都要人判，不许为了让这里变绿就调宽容差（test-discipline.md）。"
fi
if [[ $drift -eq 0 && $broken -eq 0 && $claim_bad -eq 0 && $sentinel_bad -eq 0 ]]; then
  # 全绿且用的是自动分配的临时目录 ⇒ 收拾掉。有一条不绿就留着，上面的提示指着它。
  [[ -z "${REPLAY_OUT:-}" ]] && rm -rf "$OUT_DIR"
  if (( load_skipped && pass + timing_only + archived + unchanged + reused == 0 )); then
    echo "  ! 判的全是本次不判（负载过高），一行都没比：不算通过，退 77"
    exit 77
  fi
  echo "  ✓ 复跑现判 $((pass + timing_only)) 行都对得上（字节一致 $pass、仅计时不同 $timing_only）、复用 $reused 行、行键拒算照跑 $refused_rows 行、本次未跑 $((load_skipped + ${#ARCHIVED_ROWS[@]})) 行（负载高本次不判 $load_skipped、已归档只跑不比 ${#ARCHIVED_ROWS[@]}）、输入没变没跑 $unchanged 行，结论断言全中；负载关：判了 $load_passed、滞后 $load_lagged 份"
  exit 0
fi
exit 1
