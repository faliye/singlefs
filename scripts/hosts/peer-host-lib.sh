#!/usr/bin/env bash
# 双机分片共用的第二台那几段：读本地配置、在第二台上跑命令、拷文件、比工具链、找规范副本、清场（不复原）、
# 脱离 ssh 会话起一份活并轮询、按进程号连同后代停一棵进程树、把输出里配置的主机与目录换成「第二台」。
# 被 source 的函数库，只定义函数与几个全局变量、不改 shell 选项，不单独调；scripts/mutation/mutation-shard-run.sh（门禁 checker-tier-crates-mutation-replay 的双机分片驱动）source 它。
# gate-overlap:copy-kept crash-point-replay-shard-run.sh 改它会让登记了 shard=across-machines 的三条崩溃枚举用例的全绿标记作废（admission.py 的 SHARD_DRIVER_FILES 按内容进输入指纹），KV 接入那一批再把它迁到这一份上
#
# 配置与崩溃点重放分片同一份：scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-assignments（只调）。配置里的主机、目录与清场命令是私有的：
# 这里的函数打出来的话一律写「第二台」，第二台上的命令与拷文件的错误输出经 peer_host_redact 换掉再打。
#
# 两台用同一份多机配置（用户 2026-10-23 定「两台的配置应该是一样的 不应该不同 这边改了那边就应该改」）：本机这一份（peer_host_load_configuration 读到的那一份）是唯一的源，
#   往第二台起活的每个入口（scripts/hosts/multi-host-run.sh --peer，经它的 scripts/crash/crash-amplification-two-hosts.sh 与崩溃点重放分片，scripts/mutation/mutation-shard-run.sh）
#   拷完树之后调 peer_host_carry_configuration，把它原样带到第二台这一趟的树根、回读 sha256 与本机的比，对不上就拒跑；第二台那条命令前面加 peer_host_role_environment 打的几个赋值：
#   SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE=peer（这个进程在哪台机器；Rust 那一侧读同一个变量，crates/singlefs-checker-tier/src/gpu_card_configuration.rs 的 HOST_ROLE_ENVIRONMENT_VARIABLE）、
#   SINGLEFS_MULTI_HOST_CONFIG=<带过去的那一份>，起活这一侧照配置清过场、回读过了的另加 GPU_QUIESCE_ORIGIN_READ_BACK=1。
#   第二台上照角色换读的几处：scripts/crash/crash-point-replay-shard-configuration-check.sh（角色是 peer 就报「双机：关」、退 3，PEER_SSH_HOST、PEER_*、HOSTS、
#   HOST_JUDGE_WEIGHTS 一样都不打，经它读配置的脚本在第二台上都不再往第二台起活）与 scripts/crash/gpu-quiesce-run.sh（清场四个键在起活那台跑、两台都管，第二台上不跑，
#   认 GPU_QUIESCE_ORIGIN_READ_BACK=1；显卡先后里 peer: 那几项是第二台自己的卡）。写法与依据在 multi-host.env.example 与这两份的文件头。
#
# 工具链比对（peer_host_compare_toolchains，欠账 拆分提案第四十七节第 2 行）：两台 rustc -Vv 前三行与 host 行、cargo -V 逐字相同；
#   另比两台 rustup component list --installed 里 rustfmt、clippy 那两行（PEER_HOST_REQUIRED_COMPONENTS；合入后验证的 fmt、clippy 两步要它们）：
#   哪一台缺一样，按「两台的工具链不同」拒、点名哪一台缺哪个组件，出路是在那一台 rustup component add rustfmt clippy；两台都有而那两行不同（目标三元组不同）同样拒。
#   rustup component list 跑不了（PATH 上没有 rustup）也拒，不当作相同放过。
# 离线依赖（peer_host_check_offline_dependencies，欠账 拆分提案第五十三节「同一天另一处」）：调用方把树拷到第二台之后、起命令之前调：
#   树根有 Cargo.toml 的，在第二台那棵树里空跑 cargo metadata --offline --filter-platform <第二台 rustc -vV 的 host>（只解析依赖、不编，至多 PEER_HOST_METADATA_SECONDS 秒；
#   不带 --filter-platform 时别的平台才用的依赖也要下载，本机 2026-10-26 在本仓工作区上量到它报缺 android_system_properties，带上就过）：
#   退非 0 时从 cargo 的报错里认缺的依赖（no matching package named `X`、failed to get `X` as a dependency、failed to download `X`、
#   failed to select a version for the requirement `X …`），打「第二台缺依赖 X」与同步办法（本机 cargo fetch 之后把本机 cargo 缓存的 registry/ 与 git/ rsync 到第二台同一位置），退 1；
#   认不出缺哪个的打 cargo 报错的末几行（换掉私有的字），到点没跑完的说多半在等第二台 cargo 的包缓存锁，都退 1。树根没有 Cargo.toml 的不判（打一句）。
#   第二台上这时还没有清单，所以不与工具链比对放在同一步：工具链在拷树之前比，离线依赖在拷树之后核，两样都在这一份里、都在起命令之前。
#
# 全局变量（peer_host_load_configuration 设）：PEER_SSH_HOST PEER_REPOSITORY_DIRECTORY PEER_CARGO_BIN_DIRECTORY PEER_MEMORY_CAP
#   QUIESCE_STOP_COMMAND QUIESCE_STOPPED_CHECK_COMMAND QUIESCE_START_COMMAND QUIESCE_STARTED_CHECK_COMMAND、CRASH_POINT_REPLAY_SHARD_CONFIGURATION_FILE（本机这一份的路径）、peer_host_is_this_machine；
#   peer_host_quiesced（peer_host_quiesce 清过场置 1，peer_host_quiesce_left 报过之后置 0）；peer_host_quiesce_read_back（peer_host_quiesce 回读过了置 1）；
#   peer_host_carried_configuration、peer_host_configuration_sha256（peer_host_carry_configuration 设：第二台上这一趟用的那一份的路径、本机这一份的 sha256）；
#   peer_host_toolchain_text（peer_host_compare_toolchains 设）；
#   peer_host_detached_outcome（peer_host_wait_detached 设：exit <码>、vanished、unreachable、never-started）。
# 调用方设的全局变量：PEER_HOST_WAIT_HOOK（函数名；peer_host_wait_detached 每轮询到一次 running 调它一次，不带参数，它的退出码不看；
#   multi-host-run.sh 用它每隔一段取第二台那条命令在用几个核，scripts/hosts/core_usage.py；没设什么都不调）。
# 只供测试的开关 SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1（照崩溃点重放分片）：「第二台」是本机上的另一个目录，命令在本机跑，
#   环境照 ssh 那样只给最少几样（HOME、USER、LOGNAME、LANG、XDG_RUNTIME_DIR、DBUS_SESSION_BUS_ADDRESS、CARGO_HOME、RUSTUP_HOME 与 PATH），本机别的环境变量不漏过去。
# 弄坏开关 PEER_HOST_LIB_BREAK=<项>（只给证红用，逗号分隔可给几个；打开时调用方的自证退 1）。这一份没有自证，红在两份 source 它的脚本的 --selftest 里：
#   scripts/mutation/mutation-shard-run.sh 与 scripts/hosts/multi-host-run.sh，两份的失败行都是「✗ <格名>：<细节>」。
# 逐项写开关名、弄坏哪一处、打开后哪一份自证的哪几格红（项名是格名，前面写是哪一份；括号里抄细节的句首，随跑变的路径写成 …）：
#   PEER_HOST_LIB_BREAK=no-redact  peer_host_redact 原样转出、不换私有的字
#       mutation-shard-run.sh 的「① 输出里没有配置的主机、第二台的目录与清场命令」（「…/singlefs-mutation-shard-logs/…/peer.log 里有」）
#       multi-host-run.sh 的「Ⓖ 输出里没有配置里的主机名与第二台的目录，命令打的那两样换成了「第二台」「第二台的目录」」（「有的：peer/run.out」）
#   PEER_HOST_LIB_BREAK=detached-keeps-caller-fds  peer_host_start_detached 起的那一份不关继承来的 fd（改前的写法）
#       multi-host-run.sh 的「Ⓕ 第二台那条命令只握 0、1、2，不握驱动那一头的管道」（「多握着的：63 -> pipe:[…]」）
#       与 no-redact 一起开时 multi-host-run.sh 的 Ⓕ 两格都红：起第二台那条命令那一步要等它跑完才回来，收到 TERM 之后 120 秒还没退（「驱动收到 TERM 之后 120 秒还没退」）
#   PEER_HOST_LIB_BREAK=restore-after-run  peer_host_quiesce_left 照改前跑配置的 QUIESCE_START_COMMAND 复原
#       mutation-shard-run.sh 的「① 照配置清了场、跑完没复原」（「stderr：…」）与「⑥ 两份都在跑时收到 TERM」（「退 143；…」）
#       multi-host-run.sh 的自证不红（它不清场）
#   PEER_HOST_LIB_BREAK=configuration-not-carried  peer_host_carry_configuration 不拷、不回读，第二台那条命令也不带 SINGLEFS_MULTI_HOST_CONFIG（改前的样子：第二台没有配置）
#       multi-host-run.sh 的「Ⓤ --peer 把本机这一份多机配置原样带到第二台这一趟的树根（sha256 相同），第二台那条命令认得自己是第二台，第二台的专用目录里常驻那一份也相同」
#         （「退 0；第二台那条命令看到的：role=peer configuration=没设 sum=不在 origin=没设；…」）、
#         「Ⓥ 带到第二台的配置与本机这一份 sha256 不同（假 rsync 拷完往那一份末尾加一行）⇒ 拒（退 255，说两台的配置不同），命令没跑、第二台这一趟的树删了」（「退 0，命令跑了…」：没拷就没得比）、
#         「Ⓦ 第二台上（带过去的配置、角色 peer）判法报「双机：关」退 3、不打赋值，再起 multi-host-run.sh --peer 拒（退 255），一次都没往「第二台」ssh」
#         （「判法退 1（  ✗ 双机分片不能用：没有配置文件 …」）、
#         「Ⓨ 判法报两台配置一不一致：跑过一趟之后报一致、本机这一份改了之后报不一致、第二台没有常驻那一份时报还没有」（「跑过一趟之后：第二台还没有常驻的那一份…」）
#       mutation-shard-run.sh 的「⑯ 第二台那一份读到的多机配置就是本机这一份（这一趟带过去的，sha256 相同）、认得自己是第二台，起活这一侧清过场；本机那一份照旧；第二台专用目录下常驻那一份也相同」
#         （「驱动退 …；第二台：role=peer configuration=没设 sum=不在 origin=1|…」），连带经 checker-tier-crates-mutation-replay 跑的 16 格（那份文件头这一条下面写了为什么）
#   PEER_HOST_LIB_BREAK=role-reversed  peer_host_role_environment 给第二台那条命令设 SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE=local（角色读反）
#       multi-host-run.sh 的「Ⓤ --peer 把本机这一份多机配置原样带到第二台这一趟的树根…」（「退 0；第二台那条命令看到的：role=local configuration=…」）
#         与「Ⓦ 第二台上（带过去的配置、角色 peer）判法报「双机：关」退 3、不打赋值，再起 multi-host-run.sh --peer 拒（退 255），一次都没往「第二台」ssh」
#         （「判法退 1（  ✗ 双机分片不能用：ssh -o BatchMode=yes … true 连不上 …），emit 退 1，里层 --peer 退 255；ssh 记了 4 次」）
#       mutation-shard-run.sh 的「⑯ 第二台那一份读到的多机配置就是本机这一份…」（「驱动退 …；第二台：role=local …」），连带经 checker-tier-crates-mutation-replay 跑的 16 格
#   PEER_HOST_LIB_BREAK=differing-configuration-accepted  peer_host_carry_configuration 回读到第二台那一份与本机的 sha256 不同也放行
#       multi-host-run.sh 的「Ⓥ 带到第二台的配置与本机这一份 sha256 不同（假 rsync 拷完往那一份末尾加一行）⇒ 拒（退 255，说两台的配置不同），命令没跑、第二台这一趟的树删了」（「退 0，命令跑了…」）
#       mutation-shard-run.sh 的自证只红经 checker-tier-crates-mutation-replay 跑的那 16 格（它没有造两台配置不同的那一格）
#   PEER_HOST_LIB_BREAK=origin-read-back-always  peer_host_role_environment 不看起活这一侧清没清场，一律给第二台设 GPU_QUIESCE_ORIGIN_READ_BACK=1
#       multi-host-run.sh 的「Ⓤ --peer 把本机这一份多机配置原样带到第二台这一趟的树根…」（「…origin=1」：这一格的驱动不经清场包装起）
#         与「Ⓧ 驱动正经 gpu-quiesce-run.sh 起（登记文件在、名字以驱动的父进程号起头）⇒ 第二台那条命令拿到 GPU_QUIESCE_ORIGIN_READ_BACK=1；不是 ⇒ 没设」（「…不是那一趟退 0、看到 origin=1」）
#       mutation-shard-run.sh 的自证只红经 checker-tier-crates-mutation-replay 跑的那 16 格（它的场景都照配置清了场）
#   PEER_HOST_LIB_BREAK=components-uncompared  peer_host_compare_toolchains 不比两台的 rustfmt、clippy 组件（改前的样子）
#       multi-host-run.sh 的「Ⓒ 第二台缺 clippy 组件（rustup component list --installed 里没有那一行）⇒ 拒（退 255，说两台的工具链不同、第二台缺组件 clippy），命令没跑、第二台没建这一趟的目录」
#         （「退 0，命令跑了」）
#       mutation-shard-run.sh 的自证不红（它没有造少组件的那一格）
#   PEER_HOST_LIB_BREAK=offline-dependencies-unchecked  peer_host_check_offline_dependencies 不在第二台空跑 cargo metadata --offline（改前的样子）
#       multi-host-run.sh 的「Ⓒ 第二台那棵树 cargo metadata --offline 报缺依赖 vstd ⇒ 拒（退 255，说第二台缺依赖 vstd、带同步办法），命令没跑、第二台这一趟的树删了」
#         （「退 0，命令跑了」）
#         与「Ⓒ 第二台那棵树 cargo metadata --offline 过了 ⇒ 照跑、退 0，假 cargo 记下在那棵树的树根跑过 metadata --offline」（「退 0，metadata 那一步记下的：」后面是空的）
#       mutation-shard-run.sh 的自证不红（它不调这个函数）
# 不设第二台共用编译目录（用户 2026-10-23 定案「自己用完自己删」）：第二台每一趟在自己那棵树里编，multi-host-run.sh 的 ⑦ 连同树里的编译目录一起删。

peer_host_is_this_machine=0
peer_host_quiesced=0
peer_host_quiesce_read_back=0
peer_host_carried_configuration=""
peer_host_configuration_sha256=""
peer_host_toolchain_text=""
peer_host_detached_outcome=""
PEER_HOST_ROLE_VARIABLE=SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE   # 这个进程在哪台机器：local 或 peer，没设是 local（文件头「两台用同一份多机配置」）
PEER_HOST_CARRIED_CONFIGURATION_NAME=multi-host.env              # 带到第二台的那一份放在这一趟的树根、与第二台专用目录下常驻那一份的文件名
PEER_HOST_REQUIRED_COMPONENTS=(rustfmt clippy)                   # 两台都要装的 rustup 组件（文件头「工具链比对」）
PEER_HOST_METADATA_SECONDS=300                                   # 第二台空跑 cargo metadata --offline 至多等几秒（文件头「离线依赖」）

peer_host_this_machine_is_peer() { # 这个进程是不是第二台（SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE=peer）
  [[ "${SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE:-local}" == peer ]]
}

peer_host_load_configuration() { # <配置判法脚本> <树根>：判得过就设配置的键，退 0；判法退 3（双机：关，配置没写 ENABLE_ACROSS_MACHINES=1）
  # 什么都不设、不打，退 3，由调用方报「双机：关」；判不过打一行原因（不含配置内容）与出路，退 1
  local assignments configuration_exit=0
  assignments="$(bash "$1" --emit-assignments "$2" 2>/dev/null)" || configuration_exit=$?
  if (( configuration_exit == 3 )); then return 3; fi
  if (( configuration_exit != 0 )); then
    echo "  ✗ 双机分片的本地配置判不过（原因不在这里打：配置里的主机与目录是私有的）"
    echo "     → 怎么办：单跑 bash scripts/crash/crash-point-replay-shard-configuration-check.sh 看原因；配置照仓根 multi-host.env.example 写"
    return 1
  fi
  eval "$assignments"
  peer_host_is_this_machine="${SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE:-0}"
}

peer_host_redact() { # 标准输入原样转到标准输出，配置里第二台的目录、cargo 目录与主机名换成「第二台的目录」「第二台的 cargo 目录」「第二台」
  if [[ ",${PEER_HOST_LIB_BREAK:-}," == *,no-redact,* ]]; then cat; return; fi
  PEER_HOST_REDACT_HOST="${PEER_SSH_HOST:-}" PEER_HOST_REDACT_DIRECTORY="${PEER_REPOSITORY_DIRECTORY:-}" PEER_HOST_REDACT_CARGO="${PEER_CARGO_BIN_DIRECTORY:-}" \
    python3 -c '
import os, re, sys
replacements = [(os.environ["PEER_HOST_REDACT_DIRECTORY"], "第二台的目录"), (os.environ["PEER_HOST_REDACT_CARGO"], "第二台的 cargo 目录")]
replacements = sorted((pair for pair in replacements if pair[0]), key=lambda pair: -len(pair[0]))
host = os.environ["PEER_HOST_REDACT_HOST"]
host_pattern = re.compile(r"(?<![\w.-])" + re.escape(host) + r"(?![\w-])") if host else None
for line in sys.stdin:
    for secret, shown in replacements:
        line = line.replace(secret, shown)
    if host_pattern:
        line = host_pattern.sub("第二台", line)
    sys.stdout.write(line)
    sys.stdout.flush()'
}

peer_host_run() { # <目录> <命令文本>：在第二台的这个目录里 bash -c 跑（PATH 前面加 PEER_CARGO_BIN_DIRECTORY），标准错误换掉私有的字再转出；退出码是那条命令的
  if [[ "$peer_host_is_this_machine" == 1 ]]; then
    # 和 ssh 一样等标准输出关上才返回（经 cat 转出）：留在后台、还握着输出的进程会让它一直等
    (cd "$1" && env -i HOME="$HOME" USER="${USER:-}" LOGNAME="${LOGNAME:-}" LANG="${LANG:-C.UTF-8}" XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-}" \
        DBUS_SESSION_BUS_ADDRESS="${DBUS_SESSION_BUS_ADDRESS:-}" ${CARGO_HOME:+CARGO_HOME="$CARGO_HOME"} ${RUSTUP_HOME:+RUSTUP_HOME="$RUSTUP_HOME"} \
        PATH="$PEER_CARGO_BIN_DIRECTORY:$PATH" bash -c "$2") 2> >(peer_host_redact >&2) | cat
    return "${PIPESTATUS[0]}"
  fi
  ssh -o BatchMode=yes "$PEER_SSH_HOST" "cd $(printf '%q' "$1") && PATH=$(printf '%q' "$PEER_CARGO_BIN_DIRECTORY"):\$PATH bash -c $(printf '%q' "$2")" \
    </dev/null 2> >(peer_host_redact >&2)
}

peer_host_copy() { # <rsync 的参数…>：rsync -a（不带 --delete），第二台那头的路径用 peer_host_path 写；错误输出换掉私有的字
  if [[ "$peer_host_is_this_machine" == 1 ]]; then
    rsync -a "$@" 2> >(peer_host_redact >&2)
  else
    rsync -a -e "ssh -o BatchMode=yes" "$@" 2> >(peer_host_redact >&2)
  fi
}

peer_host_path() { # <第二台上的路径>：rsync 的参数写法
  if [[ "$peer_host_is_this_machine" == 1 ]]; then printf '%s' "$1"; else printf '%s:%s' "$PEER_SSH_HOST" "$1"; fi
}

peer_host_component_lines() { # <rustup component list --installed 的输出>：打其中 PEER_HOST_REQUIRED_COMPONENTS 那几行（就是组件名、或组件名后接「-<目标>」），按组件的次序
  local component line
  for component in "${PEER_HOST_REQUIRED_COMPONENTS[@]}"; do
    while IFS= read -r line; do
      if [[ "$line" == "$component" || "$line" == "$component"-* ]]; then printf '%s\n' "$line"; fi
    done <<<"$1"
  done
}

peer_host_missing_components() { # <peer_host_component_lines 打的那几行>：打缺的组件名，用「、」隔开；都在打空
  local component line found joined=""
  for component in "${PEER_HOST_REQUIRED_COMPONENTS[@]}"; do
    found=0
    while IFS= read -r line; do
      if [[ "$line" == "$component" || "$line" == "$component"-* ]]; then found=1; fi
    done <<<"$1"
    (( found )) || joined+="${joined:+、}$component"
  done
  printf '%s' "$joined"
}

peer_host_compare_components() { # 两台 rustup component list --installed 里 rustfmt、clippy 那几行都在、逐字相同退 0；缺、不同或跑不了打原因与出路，退 1（文件头「工具链比对」）
  local component_command='rustup component list --installed' local_components peer_components local_missing peer_missing
  if ! local_components="$(bash -c "$component_command" 2>/dev/null)"; then
    echo "  ✗ 本机 rustup component list --installed 跑不了，比不了 rustfmt、clippy 组件"
    echo "     → 怎么办：看 PATH 里有没有 rustup（工具链要经 rustup 装）"
    return 1
  fi
  if ! peer_components="$(peer_host_run / "$component_command" 2>/dev/null)"; then
    echo "  ✗ 第二台 rustup component list --installed 跑不了，比不了 rustfmt、clippy 组件"
    echo "     → 怎么办：看配置的 PEER_CARGO_BIN_DIRECTORY 下有没有 rustup、ssh 连不连得上"
    return 1
  fi
  local_components="$(peer_host_component_lines "$local_components")"
  peer_components="$(peer_host_component_lines "$peer_components")"
  local_missing="$(peer_host_missing_components "$local_components")"
  peer_missing="$(peer_host_missing_components "$peer_components")"
  if [[ -n "$local_missing" ]]; then
    echo "  ✗ 两台的工具链不同：本机缺组件 $local_missing（rustup component list --installed 里没有那一行）"
    echo "     → 怎么办：在本机 rustup component add rustfmt clippy（合入后验证的 fmt、clippy 两步要它们），再跑"
    return 1
  fi
  if [[ -n "$peer_missing" ]]; then
    echo "  ✗ 两台的工具链不同：第二台缺组件 $peer_missing（rustup component list --installed 里没有那一行）"
    echo "     → 怎么办：在第二台上给与本机同一个工具链 rustup component add rustfmt clippy（合入后验证的 fmt、clippy 两步要它们），再跑"
    return 1
  fi
  if [[ "$local_components" != "$peer_components" ]]; then
    echo "  ✗ 两台的工具链不同：rustfmt、clippy 组件那几行本机「${local_components//$'\n'/；}」，第二台「${peer_components//$'\n'/；}」"
    echo "     → 怎么办：两台装同一个 rustup 工具链与组件（目标三元组相同）再跑"
    return 1
  fi
}

peer_host_compare_toolchains() { # 两台 rustc -Vv 前三行与 host 行、cargo -V 相同，rustfmt、clippy 组件两台都在、那几行相同，退 0（peer_host_toolchain_text 设成本机 rustc 那一行）；
  # 不同、缺组件或报不出打原因与出路，退 1（文件头「工具链比对」）。弄坏开关 components-uncompared 下不比组件
  local description_command='rustc -Vv | head -3 && rustc -Vv | grep "^host: " && cargo -V' local_toolchain peer_toolchain
  if ! local_toolchain="$(bash -c "$description_command" 2>/dev/null)"; then
    echo "  ✗ 本机 rustc / cargo 报不出版本"
    echo "     → 怎么办：看 PATH 里有没有 rustc 与 cargo"
    return 1
  fi
  if ! peer_toolchain="$(peer_host_run / "$description_command" 2>/dev/null)"; then
    echo "  ✗ 第二台 rustc / cargo 报不出版本"
    echo "     → 怎么办：看配置的 PEER_CARGO_BIN_DIRECTORY 下有没有 rustc 与 cargo、ssh 连不连得上"
    return 1
  fi
  if [[ "$local_toolchain" != "$peer_toolchain" ]]; then
    echo "  ✗ 两台的工具链不同：本机「${local_toolchain//$'\n'/；}」，第二台「${peer_toolchain//$'\n'/；}」"
    echo "     → 怎么办：两台装同一个 rustup 工具链（rustc -Vv 的 commit 相同）再跑"
    return 1
  fi
  if [[ ",${PEER_HOST_LIB_BREAK:-}," == *,components-uncompared,* ]]; then
    echo "  ! 弄坏开关 PEER_HOST_LIB_BREAK=components-uncompared：不比两台的 rustfmt、clippy 组件"
  else
    peer_host_compare_components || return 1
  fi
  peer_host_toolchain_text="${local_toolchain%%$'\n'*}"
}

peer_host_check_offline_dependencies() { # <第二台上这一趟的树根>：树根有 Cargo.toml 的，在那棵树里空跑 cargo metadata --offline，过了退 0；
  # 缺离线依赖打「第二台缺依赖 <名>」与同步办法，认不出缺哪个、到点没跑完打原因与出路，退 1；树根没有 Cargo.toml 的打一句、退 0（文件头「离线依赖」）。
  # 第二台的报错经 peer_host_run 换掉私有的字。弄坏开关 offline-dependencies-unchecked 下不跑、退 0（改前的样子）
  local tree_root="$1" errors status missing
  if [[ ",${PEER_HOST_LIB_BREAK:-}," == *,offline-dependencies-unchecked,* ]]; then
    echo "  ! 弄坏开关 PEER_HOST_LIB_BREAK=offline-dependencies-unchecked：不在第二台空跑 cargo metadata --offline"
    return 0
  fi
  if ! peer_host_run "$tree_root" "test -f Cargo.toml" >/dev/null 2>&1; then
    echo "  · 第二台那棵树的树根没有 Cargo.toml：不核离线依赖"
    return 0
  fi
  # --filter-platform 第二台的 host：不带它 cargo metadata 连别的平台才用的依赖（android_system_properties 这类）也要下载，缓存里本来就没有，次次误报
  if errors="$(peer_host_run "$tree_root" "timeout $PEER_HOST_METADATA_SECONDS cargo metadata --offline --format-version 1 --filter-platform \"\$(rustc -vV | sed -n 's/^host: //p')\" >/dev/null" 2>&1)"; then
    echo "  · 第二台那棵树 cargo metadata --offline 过了：离线依赖齐"
    return 0
  else
    status=$?
  fi
  missing="$(grep -oE 'no matching package named `[^`]+`|failed to get `[^`]+` as a dependency|failed to download `[^`]+`|failed to select a version for the requirement `[^`]+' <<<"$errors" \
    | grep -oE '`[^` =]+' | tr -d '`' | awk '!seen[$0]++ { printf "%s%s", (joined++ ? "、" : ""), $0 }')"
  if [[ -n "$missing" ]]; then
    echo "  ✗ 第二台缺依赖 $missing：在第二台那棵树里 cargo metadata --offline 解析不出（第二台的 cargo 缓存里没有，命令跑到 cargo 那一步会退 101），这一趟不跑"
    echo "     → 怎么办：同步第二台的 cargo 缓存：本机仓根先跑 cargo fetch（把依赖拉进本机的 cargo 缓存，默认 ~/.cargo），再把本机 cargo 缓存里的 registry/ 与 git/ 两个目录 rsync -a 到第二台 cargo 缓存的同一位置（不带 --delete）；第二台连得上网的，也可以在第二台那份仓里跑一次 cargo fetch。同步完再跑同一条命令"
    return 1
  fi
  if (( status == 124 )); then
    echo "  ✗ 第二台那棵树 cargo metadata --offline $PEER_HOST_METADATA_SECONDS 秒没跑完（多半在等第二台 cargo 的包缓存锁），这一趟不跑"
    echo "     → 怎么办：看第二台有没有别的 cargo 正在拉依赖或编译，等它放锁再跑同一条命令"
    return 1
  fi
  echo "  ✗ 第二台那棵树 cargo metadata --offline 退 $status，从报错里认不出缺哪个依赖，这一趟不跑；cargo 报的末几行（换掉了私有的字）："
  printf '%s\n' "$errors" | tail -n 8 | sed 's/^/      /'   # gate-lint:detail
  echo "     → 怎么办：照上面 cargo 报的修；缺依赖的同步办法是本机 cargo fetch 之后把本机 cargo 缓存的 registry/ 与 git/ rsync -a 到第二台同一位置；修好再跑同一条命令"
  return 1
}

peer_host_sop_source() { # <树根> <驱动所在的仓根>：打要拷到第二台的规范副本目录——树里有就用树里的，没有用驱动所在仓的，再没有用那份仓的 git common-dir 所在的主仓的；都没有退 1
  local candidate common_directory
  local candidates=("$1/.claude/singlefs-ai-sop" "$2/.claude/singlefs-ai-sop")
  if common_directory="$(git -C "$2" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" && [[ -n "$common_directory" ]]; then
    candidates+=("${common_directory%/*}/.claude/singlefs-ai-sop")
  fi
  for candidate in "${candidates[@]}"; do
    if [[ -f "$candidate/scripts/preflight.py" ]]; then printf '%s' "$candidate"; return 0; fi
  done
  return 1
}

peer_host_carry_configuration() { # <第二台上这一趟的树根>：本机这一份多机配置原样带过去（文件头「两台用同一份多机配置」）。
  # 拷到 <树根>/multi-host.env，再在第二台上原样放一份到 PEER_REPOSITORY_DIRECTORY/multi-host.env（常驻那一份：判法报两台配置一不一致时比它），两份都回读 sha256 与本机这一份比；
  # 都相同退 0，设 peer_host_carried_configuration、peer_host_configuration_sha256；拷不过去或对不上打原因与出路（不打配置的内容与路径），退 1。
  # 弄坏开关 configuration-not-carried 下不拷、不回读、不设（改前的样子）；differing-configuration-accepted 下对不上也放行
  local tree_root="$1" source_file="${CRASH_POINT_REPLAY_SHARD_CONFIGURATION_FILE:-}" local_sum carried standing staged read_back peer_sum peer_short
  local peer_sums=()
  peer_host_carried_configuration=""
  peer_host_configuration_sha256=""
  if [[ ",${PEER_HOST_LIB_BREAK:-}," == *,configuration-not-carried,* ]]; then
    echo "  ! 弄坏开关 PEER_HOST_LIB_BREAK=configuration-not-carried：本机的多机配置不带到第二台"
    return 0
  fi
  if [[ -z "$source_file" || ! -f "$source_file" ]]; then
    echo "  ✗ 本机这一份多机配置找不到（peer_host_load_configuration 没设 CRASH_POINT_REPLAY_SHARD_CONFIGURATION_FILE，或那份文件不在了），带不到第二台"
    echo "     → 怎么办：先调 peer_host_load_configuration；配置照仓根 multi-host.env.example 写"
    return 1
  fi
  local_sum="$(sha256sum < "$source_file")"
  local_sum="${local_sum%% *}"
  carried="$tree_root/$PEER_HOST_CARRIED_CONFIGURATION_NAME"
  standing="$PEER_REPOSITORY_DIRECTORY/$PEER_HOST_CARRIED_CONFIGURATION_NAME"
  staged="$standing.tmp.$$"
  if ! peer_host_copy --chmod=F600 "$source_file" "$(peer_host_path "$carried")"; then
    echo "  ✗ 本机的多机配置拷不到第二台这一趟的树根"
    echo "     → 怎么办：看第二台盘满没满、那棵树可不可写，再跑同一条命令"
    return 1
  fi
  # 常驻那一份先写同目录的临时文件再 mv 换上：别的一趟正在读它时读到的是整份旧的或整份新的
  if ! peer_host_run / "cp -- $(printf '%q' "$carried") $(printf '%q' "$staged") && chmod 600 -- $(printf '%q' "$staged") && mv -f -- $(printf '%q' "$staged") $(printf '%q' "$standing")"; then
    echo "  ✗ 第二台专用目录下常驻的那一份多机配置没换上"
    echo "     → 怎么办：看配置的 PEER_REPOSITORY_DIRECTORY 可不可写、盘满没满，再跑同一条命令"
    return 1
  fi
  read_back="$(peer_host_run / "sha256sum < $(printf '%q' "$carried") && sha256sum < $(printf '%q' "$standing")" 2>/dev/null)"
  mapfile -t peer_sums <<< "$read_back"
  for peer_sum in "${peer_sums[0]:-}" "${peer_sums[1]:-}"; do
    peer_short="${peer_sum%% *}"
    [[ "$peer_short" == "$local_sum" ]] && continue
    if [[ ",${PEER_HOST_LIB_BREAK:-}," == *,differing-configuration-accepted,* ]]; then
      echo "  ! 弄坏开关 PEER_HOST_LIB_BREAK=differing-configuration-accepted：第二台那一份与本机这一份 sha256 不同也放行"
      break
    fi
    echo "  ✗ 两台的多机配置不同：带到第二台的那一份回读出来的 sha256（${peer_short:0:12}…）与本机这一份（${local_sum:0:12}…）对不上，这一趟不跑"
    echo "     → 怎么办：本机这一份是唯一的源，每一趟都重新带过去、重新核；看第二台盘满没满、那棵树与专用目录可不可写，再跑同一条命令"
    return 1
  done
  peer_host_carried_configuration="$carried"
  peer_host_configuration_sha256="$local_sum"
  echo "  · 本机的多机配置原样带到第二台这一趟的树根与专用目录，回读 sha256 相同（${local_sum:0:12}…）"
}

peer_host_origin_quiesce_holder() { # 这个驱动正经 scripts/crash/gpu-quiesce-run.sh 起（那一层照配置清了场、回读过了）：GPU_QUIESCE_HOLDER 指的登记文件在、
  # 文件名以这个驱动的父进程号加「-」起头（gpu-quiesce-run.sh 文件头「给调用方与派发器的接口」）
  [[ -n "${GPU_QUIESCE_HOLDER:-}" && -e "$GPU_QUIESCE_HOLDER" && "${GPU_QUIESCE_HOLDER##*/}" == "$PPID-"* ]]
}

peer_host_role_environment() { # 打第二台那条命令前面要加的环境赋值（printf %q 过，空格隔开，末尾带一个空格；文件头「两台用同一份多机配置」）：
  # SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE=peer；带过去了配置的（peer_host_carry_configuration 设过）SINGLEFS_MULTI_HOST_CONFIG=<第二台上这一趟的那一份>；
  # 起活这一侧照配置清过场、回读过了的（这一趟 peer_host_quiesce 回读过了，或 peer_host_origin_quiesce_holder）GPU_QUIESCE_ORIGIN_READ_BACK=1。
  # 弄坏开关 role-reversed 下角色写成 local；origin-read-back-always 下一律带 GPU_QUIESCE_ORIGIN_READ_BACK=1
  local role=peer text
  if [[ ",${PEER_HOST_LIB_BREAK:-}," == *,role-reversed,* ]]; then role=local; fi
  text="$PEER_HOST_ROLE_VARIABLE=$role "
  if [[ -n "$peer_host_carried_configuration" ]]; then text+="SINGLEFS_MULTI_HOST_CONFIG=$(printf '%q' "$peer_host_carried_configuration") "; fi
  if (( peer_host_quiesce_read_back )) || peer_host_origin_quiesce_holder || [[ ",${PEER_HOST_LIB_BREAK:-}," == *,origin-read-back-always,* ]]; then
    text+="GPU_QUIESCE_ORIGIN_READ_BACK=1 "
  fi
  printf '%s' "$text"
}

peer_host_quiesce() { # 照配置清场（配置里写了才做），跑完回读；清场与回读的命令与输出都不打（里面有服务名）。回读没过退 1
  [[ -n "${QUIESCE_STOP_COMMAND:-}" ]] || return 0
  peer_host_quiesced=1
  bash -c "$QUIESCE_STOP_COMMAND" >/dev/null 2>&1
  if bash -c "$QUIESCE_STOPPED_CHECK_COMMAND" >/dev/null 2>&1; then
    peer_host_quiesce_read_back=1
    echo "  · 照配置清了场，回读过了"
    return 0
  fi
  echo "  ✗ 照配置清场之后回读没过（配置的 QUIESCE_STOPPED_CHECK_COMMAND 退非 0）"
  echo "     → 怎么办：手动跑配置里的清场命令，回读到它退 0 为止"
  return 1
}

peer_host_quiesce_left() { # 清过场就报一句「清了场、没复原」（调用方挂在 EXIT 上）：复原由用户手动做，这里不跑复原命令（命令不打，里面有服务名）。
  # 弄坏开关 restore-after-run 下照改前跑配置的复原命令（只给证红用）
  (( peer_host_quiesced )) || return 0
  peer_host_quiesced=0
  if [[ ",${PEER_HOST_LIB_BREAK:-}," == *,restore-after-run,* ]]; then
    bash -c "${QUIESCE_START_COMMAND:-}" >/dev/null 2>&1
    echo "  ! 弄坏开关 PEER_HOST_LIB_BREAK=restore-after-run：照配置跑了复原命令"
    return 0
  fi
  echo "  · 清了场、没复原：清场停下的服务一直停着；要复原手动跑私有复原脚本（配置的 QUIESCE_START_COMMAND），再跑 QUIESCE_STARTED_CHECK_COMMAND 回读到它退 0"
}

# 下面四个函数也经 declare -f 带到第二台上跑：只用 bash、ps 与 /proc，不引这四个之外的函数
peer_host_signal_one() { # <信号名> <进程号>：给一个写死的进程号发一个信号，一次一个（.claude/singlefs-ai-sop/rules/command-safety.md「pkill -f / killall 一律禁用」；
  # 停一棵树时逐个进程号调它，kill 不写进循环里）。进程号不是正整数、或是 0、1，不发、退 2；发不出去（进程已经没了）退 kill 的码
  [[ "$2" =~ ^[0-9]+$ ]] && (( $2 > 1 )) || return 2
  kill -"$1" "$2" 2>/dev/null
}
peer_host_process_is_alive() { # <进程号>：还在、且不是僵尸（僵尸算停了）
  [[ -d "/proc/$1" ]] && [[ "$(awk '{print $3}' "/proc/$1/stat" 2>/dev/null)" != Z ]]
}
peer_host_process_tree() { # <进程号>：打这个进程与它的全部后代的进程号，一行一个，父在子前（按 ps --ppid 一层层往下找）
  local queue=("$1") current child
  while (( ${#queue[@]} > 0 )); do
    current="${queue[0]}"
    queue=("${queue[@]:1}")
    [[ -d "/proc/$current" ]] || continue
    echo "$current"
    for child in $(ps -o pid= --ppid "$current" 2>/dev/null); do queue+=("$child"); done
  done
}
peer_host_stop_process_tree() { # <进程号>：这个进程连同全部后代，逐个按进程号冻住（STOP）、从最底层起 TERM、放开（CONT），至多等 30 秒，
  # 还在的逐个 KILL；回读一个不剩退 0，还有剩的打出来退 1。冻住之后再列一遍：列与冻之间新起的子进程也收进来
  local pids=() pid index waited survivors
  mapfile -t pids < <(peer_host_process_tree "$1")
  if (( ${#pids[@]} == 0 )); then echo "进程 $1 已经不在"; return 0; fi
  for pid in "${pids[@]}"; do peer_host_signal_one STOP "$pid"; done
  mapfile -t pids < <(peer_host_process_tree "$1")
  for pid in "${pids[@]}"; do peer_host_signal_one STOP "$pid"; done
  for (( index = ${#pids[@]} - 1; index >= 0; index-- )); do peer_host_signal_one TERM "${pids[index]}"; done
  for pid in "${pids[@]}"; do peer_host_signal_one CONT "$pid"; done
  for (( waited = 0; waited < 300; waited++ )); do
    survivors=()
    for pid in "${pids[@]}"; do peer_host_process_is_alive "$pid" && survivors+=("$pid"); done
    (( ${#survivors[@]} == 0 )) && break
    sleep 0.1
  done
  for pid in ${survivors[@]+"${survivors[@]}"}; do peer_host_signal_one KILL "$pid"; done
  sleep 0.5
  survivors=()
  for pid in "${pids[@]}"; do peer_host_process_is_alive "$pid" && survivors+=("$pid"); done
  if (( ${#survivors[@]} > 0 )); then echo "进程 $1 那一棵里 ${survivors[*]} 发了 TERM、KILL 还在"; return 1; fi
  echo "进程 $1 连同 $(( ${#pids[@]} - 1 )) 个后代都停了"
}

peer_host_start_detached() { # <目录> <进程号文件> <退出码文件> <输出文件> <命令文本>：在第二台的这个目录里脱离 ssh 会话起（setsid nohup），
  # 起的那个 bash 先把自己的进程号写进进程号文件，命令的输出写进输出文件，跑完把退出码写进退出码文件；ssh 马上返回
  local directory="$1" pid_file="$2" exit_file="$3" output_file="$4" command_text="$5"
  local detached_text close_text=""
  # 脱离会话起的那个 bash 先关掉 0、1、2 之外继承来的 fd，它与它起的命令只握 0、1、2：本机当第二台时，peer_host_run 给标准错误接的
  #   >(peer_host_redact) 那条管道（bash 的进程替换，fd 多是 63）会一路传下来；留着它，换字那个进程要等这份活跑完才读到结尾，
  #   peer_host_run 跟着等，起这一份就成了在前台等它跑完、驱动收到信号也收不了尾（换字是 python 时 bash 直接 exec 它、碰巧不等；
  #   PEER_HOST_LIB_BREAK=no-redact 换成 cat 就等到命令跑完）。ssh 起的第二台本来就只有这三个，关了也一样
  if [[ ",${PEER_HOST_LIB_BREAK:-}," != *,detached-keeps-caller-fds,* ]]; then
    close_text='for peer_host_fd_path in /proc/$$/fd/*; do peer_host_fd="${peer_host_fd_path##*/}"; if [[ "$peer_host_fd" =~ ^[0-9]+$ ]] && (( peer_host_fd > 2 )); then exec {peer_host_fd}>&-; fi; done; '
  fi
  detached_text="${close_text}echo \$\$ > $(printf '%q' "$pid_file"); $command_text > $(printf '%q' "$output_file") 2>&1; echo \$? > $(printf '%q' "$exit_file")"
  # 只让 setsid nohup 那一条进后台（它的输入输出都重定向了）：整串放进后台的话，后台那个子进程握着 ssh 的输出通道，ssh 要等它跑完才返回
  peer_host_run "$directory" "rm -f -- $(printf '%q' "$pid_file") $(printf '%q' "$exit_file") $(printf '%q' "$output_file") || exit 1; setsid nohup bash -c $(printf '%q' "$detached_text") < /dev/null > /dev/null 2>&1 &"
}

peer_host_wait_detached() { # <进程号文件> <退出码文件> <轮询秒数>：等第二台那一份跑完，结局写进 peer_host_detached_outcome：
  # exit <码>；vanished（进程没了、没留退出码，连续 3 次）；unreachable（连续 60 次 ssh 不通）；never-started（连续 60 次既没有进程号文件也没有退出码）
  local pid_file="$1" exit_file="$2" poll_seconds="$3" poll unreachable_rounds=0 vanished_rounds=0 starting_rounds=0 previous=""
  peer_host_detached_outcome=""
  while [[ -z "$peer_host_detached_outcome" ]]; do
    if poll="$(peer_host_run / "if [[ -s $(printf '%q' "$exit_file") ]]; then echo \"exit \$(cat $(printf '%q' "$exit_file"))\"; elif [[ -s $(printf '%q' "$pid_file") && -d /proc/\$(cat $(printf '%q' "$pid_file")) ]]; then echo running; elif [[ -s $(printf '%q' "$pid_file") ]]; then echo vanished; else echo starting; fi" 2>/dev/null)"; then
      unreachable_rounds=0
    else
      poll=unreachable; unreachable_rounds=$((unreachable_rounds + 1))
    fi
    case "$poll" in
      "exit "*) peer_host_detached_outcome="$poll" ;;
      vanished) vanished_rounds=$((vanished_rounds + 1)); (( vanished_rounds < 3 )) || peer_host_detached_outcome=vanished ;;
      unreachable) (( unreachable_rounds < 60 )) || peer_host_detached_outcome=unreachable ;;
      starting) starting_rounds=$((starting_rounds + 1)); (( starting_rounds < 60 )) || peer_host_detached_outcome=never-started ;;
      *) vanished_rounds=0 ;;
    esac
    if [[ "$poll" != "$previous" && -z "$peer_host_detached_outcome" ]]; then
      echo "  … 第二台那一份：$poll"
      previous="$poll"
    fi
    if [[ "$poll" == running && -n "${PEER_HOST_WAIT_HOOK:-}" ]]; then "$PEER_HOST_WAIT_HOOK" || :; fi
    [[ -n "$peer_host_detached_outcome" ]] || sleep "$poll_seconds"
  done
}

peer_host_stop_detached() { # <目录> <进程号文件>：按第二台上记下的进程号停那一份（连同后代）；进程号文件要等那一份起来才有，至多等 10 秒；
  # 核它的工作目录是这个目录（进程号没被别的进程重用）。打一句结局；没停下退 1
  peer_host_run "$1" "$(declare -f peer_host_signal_one peer_host_process_is_alive peer_host_process_tree peer_host_stop_process_tree)
for ((pid_rounds = 0; pid_rounds < 100; pid_rounds++)); do [[ -s $(printf '%q' "$2") ]] && break; sleep 0.1; done
detached_pid=\"\$(cat $(printf '%q' "$2") 2>/dev/null)\"
if [[ -z \"\$detached_pid\" ]]; then echo '没有进程号文件：那一份没起来'
elif [[ \"\$(readlink /proc/\$detached_pid/cwd 2>/dev/null)\" != \"\$(pwd -P)\" ]]; then echo \"进程号 \$detached_pid 已经不在（或不是这一趟的）\"
else peer_host_stop_process_tree \"\$detached_pid\"; fi"
}
