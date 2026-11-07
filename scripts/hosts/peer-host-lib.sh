#!/usr/bin/env bash
# 双机分片共用的第二台那几段：读本地配置、在第二台上跑命令、比工具链、核离线依赖、找规范副本、清场（不复原）、
# 脱离 ssh 会话起一份活并轮询、按进程号连同后代停一棵进程树、按会话号与尝试标签停这一趟留下的进程、把输出里配置的主机与目录换成「第二台」。
# 被 source 的函数库，只定义函数与几个全局变量、不改 shell 选项，不单独调。往第二台起活的只有 scripts/hosts/multi-host-run.sh --peer（task-server 派到第二台的尝试都经它），
#   它调这里的几乎每一段。别的调用方只用其中一两段、不往第二台起活：scripts/mutation/mutation-shard-run.sh（crates 变异门禁的分片驱动，只由 task-server 按分片协议起）
#   只调 peer_host_load_configuration；scripts/hosts/host-dispatch.py（task-server 载入的判法库）起 bash -c 'source "$1"' 调 peer_host_load_configuration、peer_host_run；
#   scripts/hosts/gpu-card-health.py 调 peer_host_load_by_host、peer_host_run、peer_host_redact。
#
# 配置与崩溃点重放分片同一份：scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-assignments（只调）。配置里的主机、目录与清场命令是私有的：
# 这里的函数打出来的话一律写「第二台」，第二台上的命令的错误输出经 peer_host_redact 换掉再打。
#
# 两台用同一份多机配置：本机这一份（peer_host_load_configuration 读到的那一份）是唯一的源，
#   这一份落在 NFS 共享根下（仓根的 multi-host.env；多机配置的 NFS_SHARED_ROOTS），第二台照同一个绝对路径读它，不拷、第二台上不留常驻的那一份
#   （第二台不需要常驻配置，分任务就是在这里分；往第二台起活的入口 scripts/hosts/multi-host-run.sh --peer 核它在共享根下）；
#   第二台那条命令前面加 peer_host_carried_environment 打的几个赋值：SINGLEFS_MULTI_HOST_CONFIG=<本机这一份的绝对路径>，
#   起活这一侧照配置清过场、回读过了的另加 GPU_QUIESCE_ORIGIN_READ_BACK=1（第二台上的清场包装 scripts/crash/gpu-quiesce-run.sh 认它）。
#   两台对称（scripts/crash/crash-point-replay-shard-configuration-check.sh 文件头「两台对称」）：不带认台的角色变量，哪一台是自己只照 HOSTS 与这一台网卡上的地址认
#   （下面「主机数组」）；同一份配置在两台上判的结局相同，显卡先后里卡的台标签写 HOSTS 里那一台的地址（multi-host.env.example 的显卡两键），在哪台读都指同一张卡。
#   任务里面不再往别的台起活：第二台那条命令里再起 multi-host-run.sh --peer 照拒（那份文件头 ①「只收服务直接起的那一次」）。
#
# 工具链比对（peer_host_compare_toolchains，欠账 拆分提案第四十七节第 2 行）：两台 rustc -Vv 前三行与 host 行、cargo -V 逐字相同；
#   另比两台 rustup component list --installed 里 rustfmt、clippy 那两行（PEER_HOST_REQUIRED_COMPONENTS；合入后验证的 fmt、clippy 两步要它们）：
#   哪一台缺一样，按「两台的工具链不同」拒、点名哪一台缺哪个组件，出路是在那一台 rustup component add rustfmt clippy；两台都有而那两行不同（目标三元组不同）同样拒。
#   rustup component list 跑不了（PATH 上没有 rustup）也拒，不当作相同放过。
#   rustup 经 PATH 找（两台都是）：source 这份库的驱动在 --selftest 里把假的 rustup 放在 PATH 最前面（登记在 selftest-registry.tsv 的自证不起真 rustup：
#   scripts/gate/selftest-process-guard.py 认 cargo 的 bin 目录下的可执行文件，真 rustup 就在那里）：multi-host-run.sh 的自证放一份写死组件表的；
#   别的调用方都不调 peer_host_compare_toolchains。
# 离线依赖（peer_host_check_offline_dependencies，欠账 拆分提案第五十三节「同一天另一处」）：调用方起命令之前，照工作目录所在那棵树的根（共享根下，两台同一个绝对路径）调：
#   树根有 Cargo.toml 的，在第二台那里空跑 cargo metadata --offline --filter-platform <第二台 rustc -vV 的 host>（只解析依赖、不编，至多 PEER_HOST_METADATA_SECONDS 秒；
#   不带 --filter-platform 时别的平台才用的依赖也要下载，本机在本仓工作区上量到它报缺 android_system_properties，带上就过）：
#   退非 0 时从 cargo 的报错里认缺的依赖（no matching package named `X`、failed to get `X` as a dependency、failed to download `X`、
#   failed to select a version for the requirement `X …`、failed to load manifest for dependency `X`），先分出 path 依赖（peer_host_missing_path_dependencies）：
#   X 在树里的 Cargo.toml 里写成 path、那个目录里没有 Cargo.toml 的，是树里缺了那个目录（拷副本时漏拷了 path 依赖，例：只拷 Cargo.toml、Cargo.lock、crates/ 的副本缺 prune-server/，
#   验证包经 path 依赖用它），本机 cargo metadata 同样退 101，打「树里缺 path 依赖 <路径>，两台都编不了」与出路（把那个目录补进树），不报第二台缺缓存；
#   别的打「第二台缺依赖 X」与同步办法（本机 cargo fetch 之后把本机 cargo 缓存的 registry/ 与 git/ rsync 到第二台同一位置）；两种都有的两种都打；都退 1。
#   认不出缺哪个的打 cargo 报错的末几行（换掉私有的字），到点没跑完的说多半在等第二台 cargo 的包缓存锁，都退 1。树根没有 Cargo.toml 的不判（打一句）。
#
# 主机数组（多机配置的 HOSTS：各台有线直连段的 IP，空格分隔；判法与写法在 scripts/crash/crash-point-replay-shard-configuration-check.sh 文件头「主机数组」）：
#   哪一项是自己、第二台是哪一项只在这里取（peer_host_resolve_addresses；判法 source 这份库调它，python 一侧 scripts/hosts/multi_host_addresses.py 的 resolve 经判法取），
#   照这一台网卡上的地址（peer_host_this_machine_addresses：ip -o addr show 的地址栏去掉前缀长度）认，不写死主机名：恰好一项命中的是自己（THIS_HOST_ADDRESS），
#   不是自己的各项依次进 PEER_HOST_OTHER_ADDRESSES，头一项是第二台（PEER_HOST_ADDRESS）。ssh 直接连那个地址，用户名归各台自己的 ~/.ssh/config。
# 按台的三个键（多机配置的 CARGO_BIN_DIRECTORY_BY_HOST、BUILD_ROOT_BY_HOST、SLICE_MEMORY_CAP_BY_HOST，写法与意思在仓根 multi-host.env.example；
#   python 一侧的切法与判法是 scripts/hosts/multi_host_addresses.py 的 by_host_values）：值是空白分隔的 <地址>=<值>，每项从第一个 = 切；整份判（三个都必写、
#   HOSTS 里每一台恰好一项）归配置判法。这里判法过了之后照判法打的 CRASH_POINT_REPLAY_SHARD_CONFIGURATION_FILE 读那三行（peer_host_configured_value，读法同判法：
#   以「键=」起头的那一行、去一层成对引号），按地址取那一台那一项（peer_host_value_for_host）：经 ssh 去的另一台取 PEER_HOST_ADDRESS 那一项，这一台取 THIS_HOST_ADDRESS 那一项
#   （peer_host_load_by_host）；要的那一项没有就载不了、退 1，打的话只点名键与哪一台（「第二台」「本机」），不打值。
# 全局变量（peer_host_load_configuration 设）：HOSTS THIS_HOST_ADDRESS PEER_HOST_ADDRESS
#   PEER_HOST_CARGO_BIN_DIRECTORY PEER_HOST_BUILD_ROOT PEER_HOST_SLICE_CAP（另一台那一项：cargo 所在目录、本地盘上每一趟编译目录的根、slice 的总上限）
#   THIS_HOST_CARGO_BIN_DIRECTORY THIS_HOST_BUILD_ROOT（这一台那一项，只给 peer_host_redact 换字）
#   QUIESCE_STOP_COMMAND QUIESCE_STOPPED_CHECK_COMMAND QUIESCE_START_COMMAND QUIESCE_STARTED_CHECK_COMMAND、CRASH_POINT_REPLAY_SHARD_CONFIGURATION_FILE（本机这一份的路径）、peer_host_is_this_machine；
#   peer_host_quiesced（peer_host_quiesce 清过场置 1，peer_host_quiesce_left 报过之后置 0）；peer_host_quiesce_read_back（peer_host_quiesce 回读过了置 1）；
#   peer_host_toolchain_text（peer_host_compare_toolchains 设）；
#   peer_host_detached_outcome（peer_host_wait_detached 设：exit <码>、vanished、unreachable、never-started）。
# 这一趟留下的进程（peer_host_attempt_leftovers、peer_host_stop_attempt_leftovers，经 declare -f 带到第二台上跑）：
#   peer_host_start_detached 起的那个 bash 经 setsid 起，它就是会话首进程，会话号就是进程号文件里那个号；命令的后代都在这个会话里，
#   自己再 setsid 出去的不在，但环境里带着服务给这一次尝试的 TASK_SERVER_ATTEMPT_LABEL=<尝试标签>（task-server 给的 --label 与这个标签是同一个，task_server.py 的 PeerMachine.command）。
#   这一趟留下的 = 会话号是那个号的进程 ∪ 环境里带这一趟标签的进程（只读本用户读得到的 /proc/<号>/environ）；不算僵尸、扫的那个 shell 自己与它的祖先。
#   清了环境、又自己 setsid 出去的（会话号与标签都对不上）照会话号与标签找不到：multi-host-run.sh 先停这一趟内存包装的 scope
#   （peer_host_stop_attempt_scope，同样经 declare -f 带到第二台上跑；包装起 scope 之前把单元名写进 runs/<标签>.scope），会话号与标签那一步只当补充。
#   那个号此刻是一个启动时刻对不上的进程的：号被重用了，内核在原会话还有进程时不重用这个号，所以原会话已经空了，不按会话找。读不了 /proc 的不按已停处理。
# 停下的退出回执（peer_host_stop_detached）：起的那个 bash 被连同后代一起停的，它来不及把退出码写进退出码文件，没有回执就永远确认不了停没停。
#   所以停的那一方停完、核过这一趟留下的进程一个不剩之后，退出码文件还没有的，由它写一行「<退出码> stopped <最后发的信号>」：
#   最后发的是 TERM 写 143、要 KILL 才停下的写 137；核的时候一个都不用停（全都已经不在）的写「143 stopped already-gone」（服务停尝试发的是 TERM）。
#   读退出码文件的一方只取头一个词（peer_host_wait_detached）。
# peer_host_wait_detached 的第四个参数（可不给）是一个函数名：每一轮看完结局之后调它一次（参数 round），等到结局之后再调一次（参数 final）；
#   调用方借它在第二台跑着时接回新输出、报 CPU（multi-host-run.sh 的 multi_host_follow_peer）。不给的只等结局。
# 只供测试的开关 SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1（照崩溃点重放分片）：「第二台」是本机上的另一个目录，命令在本机跑，
#   环境照 ssh 那样只给最少几样（HOME、USER、LOGNAME、LANG、XDG_RUNTIME_DIR、DBUS_SESSION_BUS_ADDRESS、CARGO_HOME、RUSTUP_HOME 与 PATH），本机别的环境变量不漏过去。
#   只在调用方的 --selftest 里认：PEER_HOST_SELFTEST_MARK 指的文件在、第一行是 PEER_HOST_SELFTEST_MARK_CONTENT 那一行（自证建、export）才认；
#   没有标记时 peer_host_load_configuration 一律把 peer_host_is_this_machine 置 0（平常跑漏进来的这个变量改不了 peer_host_run 连哪一台）。
#   建标记的调用方：scripts/hosts/multi-host-run.sh 的 --selftest。
# 弄坏开关 PEER_HOST_LIB_BREAK=<项>（只给证红用，逗号分隔可给几个；打开时调用方的自证退 1）。这一份没有自证，红在 scripts/hosts/multi-host-run.sh 的 --selftest 里，
#   失败行是「✗ <格名>：<细节>」。别的调用方的自证下面各开关一律不红：mutation-shard-run.sh 只调 peer_host_load_configuration，shard 那一步起门禁时还把这个变量清掉
#   （mutation_shard_task_server_shard 的 stage_environment：env -u PEER_HOST_LIB_BREAK）；host-dispatch.py 只调 peer_host_load_configuration、peer_host_run，不碰各开关弄坏的那几步；
#   gpu-card-health.py 起子进程时不带 PEER_HOST_LIB_ 起头的变量。
# 逐项写开关名、弄坏哪一处、打开后 multi-host-run.sh 自证的哪几格红（项名是格名；括号里抄细节的句首，随跑变的路径写成 …）：
#   PEER_HOST_LIB_BREAK=this-machine-unmarked  peer_host_load_configuration 没有自证的标记也认 SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1
#       「peer-host-lib.sh：只供测试的 SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 没有自证的标记 ⇒ 不认（peer_host_is_this_machine 是 0）；有标记 ⇒ 认（1）」
#         （「没标记读到 0:1、有标记读到 0:1」）
#   PEER_HOST_LIB_BREAK=no-redact  peer_host_redact 原样转出、不换私有的字
#       「Ⓖ 输出里没有配置里的主机名与第二台的目录，命令打的那两样换成了「第二台」「第二台的目录」」（「有的：peer/run.err」）  history-kept: 自证格名原文引用，按原文匹配
#       连带红的两格：「换字 第二台那条命令的输出里夹一行不是合法 UTF-8 的字节…」（地址没换）、「换字 换字那一步自己退非 0…」（没报，地址漏出来）
#   PEER_HOST_LIB_BREAK=redact-strict-utf8  peer_host_redact 读标准输入、写标准输出按严格 UTF-8：一行坏字节让它退出，那一块输出都不打
#       「换字 第二台那条命令的输出里夹一行不是合法 UTF-8 的字节…」（「读到的（sed l 的写法）：…」）
#   PEER_HOST_LIB_BREAK=detached-keeps-caller-fds  peer_host_start_detached 起的那一份不关继承来的 fd
#       「Ⓕ 第二台那条命令只握 0、1、2，不握驱动那一头的管道」（「多握着的：63 -> pipe:[…]」）
#       与 no-redact 一起开时红三格（在用核数那三格删了之后没复跑）：Ⓕ 第二台那条命令在跑时收到 TERM（起第二台那条命令那一步要等它跑完才回来，收到 TERM 之后 120 秒还没退：「驱动收到 TERM 之后 120 秒还没退」）、
#         Ⓕ 只握 0、1、2，不握驱动那一头的管道，Ⓖ 输出里没有配置里的主机名与第二台的目录
#   PEER_HOST_LIB_BREAK=restore-after-run  peer_host_quiesce_left 跑配置的 QUIESCE_START_COMMAND 复原
#       哪一份自证都不红：没有调用方调 peer_host_quiesce、peer_host_quiesce_left（multi-host-run.sh 不清场，清场归 task-server）
#   PEER_HOST_LIB_BREAK=configuration-not-passed  peer_host_carried_environment 不给第二台那条命令设 SINGLEFS_MULTI_HOST_CONFIG（第二台照它自己的主工作树根去找，找到的不是本机这一份）
#       「Ⓤ --peer：第二台那条命令读的是本机这一份多机配置（同一个绝对路径、sha256 相同），第二台的专用目录里不留常驻的那一份」
#         （「退 0；第二台那条命令看到的：configuration=没设 sum=不在 origin=没设；…」）、
#       「Ⓦ 第二台那条命令里判同一份场景配置，结局（退出码与 --emit-assignments 打的赋值）与这一侧判的相同，再起 multi-host-run.sh --peer 照拒（退 255），一次都没往别的台 ssh」
#         （「第二台那里判法退 1、emit 退 1；这一侧判法退 0、emit 退 0 …」：第二台照它自己的主工作树根找不到配置）
#   PEER_HOST_LIB_BREAK=origin-read-back-always  peer_host_carried_environment 不看起活这一侧清没清场，一律给第二台设 GPU_QUIESCE_ORIGIN_READ_BACK=1
#       「Ⓤ --peer：第二台那条命令读的是本机这一份多机配置…」（「…origin=1」：这一格的驱动不经清场包装起）
#         与「Ⓧ 驱动正经 gpu-quiesce-run.sh 起（登记文件在、名字以驱动的父进程号起头）⇒ 第二台那条命令拿到 GPU_QUIESCE_ORIGIN_READ_BACK=1；不是 ⇒ 没设」（「…不是那一趟退 0、看到 origin=1」）
#   PEER_HOST_LIB_BREAK=components-uncompared  peer_host_compare_toolchains 不比两台的 rustfmt、clippy 组件
#       「Ⓒ 第二台缺 clippy 组件（rustup component list --installed 里没有那一行）⇒ 拒（退 255，说两台的工具链不同、第二台缺组件 clippy），命令没跑、第二台没建这一趟的目录」
#         （「退 0，命令跑了」）
#   PEER_HOST_LIB_BREAK=offline-dependencies-unchecked  peer_host_check_offline_dependencies 不在第二台空跑 cargo metadata --offline
#       「Ⓒ 第二台那里 cargo metadata --offline 报缺依赖 vstd ⇒ 拒（退 255，说第二台缺依赖 vstd、带同步办法），命令没跑、第二台没建这一趟的目录」
#         （「退 0，命令跑了」）
#         与「Ⓒ 第二台那里 cargo metadata --offline 过了 ⇒ 照跑、退 0，假 cargo 记下在工作目录所在那棵树的根跑过 metadata --offline」（「退 0，metadata 那一步记下的：」后面是空的）
#         与「Ⓒ 第二台那里 cargo metadata --offline 报缺的 served 在树里的 Cargo.toml 里写成 path 依赖、目录不在 ⇒ 拒（退 255，说树里缺 path 依赖 served、两台都编不了），不报第二台缺依赖，命令没跑」（「退 0，命令跑了」）
#   PEER_HOST_LIB_BREAK=path-dependencies-as-peer-cache  peer_host_check_offline_dependencies 不分 path 依赖，缺的一律报第二台缺依赖与同步 cargo 缓存
#       「Ⓒ 第二台那里 cargo metadata --offline 报缺的 served 在树里的 Cargo.toml 里写成 path 依赖、目录不在 ⇒ 拒（退 255，说树里缺 path 依赖 served、两台都编不了），不报第二台缺依赖，命令没跑」
#         （「退 255，…第二台缺依赖 served…」：stderr 里是第二台缺依赖，没有树里缺 path 依赖）
#   PEER_HOST_LIB_BREAK=by-host-first-entry  peer_host_value_for_host 不看地址，一律取头一项（写死次序的样子）
#       「peer-host-lib.sh：按台的三个键各台取各台那一项（头一项不是另一台也照地址取）；另一台那一项没写 ⇒ 载不了、退 1，打的话不带配置里的值」
#         （「另一台取到的：0 cargo=/selftest/this-cargo …」：取成了头一项那一台的）；连带红的格没逐个核（经 --peer 的格取成了这一台那一项的编译根与上限）
# 不设第二台共用编译目录（自己用完自己删）：第二台每一趟在它本地盘上这一趟的目录里编（multi-host-run.sh 文件头 ⑤），⑥ 连同编译目录一起删。

peer_host_is_this_machine=0
peer_host_quiesced=0
peer_host_quiesce_read_back=0
peer_host_toolchain_text=""
peer_host_detached_outcome=""
PEER_HOST_REQUIRED_COMPONENTS=(rustfmt clippy)                   # 两台都要装的 rustup 组件（文件头「工具链比对」）
PEER_HOST_METADATA_SECONDS=300                                   # 第二台空跑 cargo metadata --offline 至多等几秒（文件头「离线依赖」）
PEER_HOST_SELFTEST_MARK_CONTENT="peer-host-lib.sh 的调用方自证"    # 只供测试的开关只在这份标记在时认（文件头「只供测试的开关」）
PEER_HOST_CARGO_BIN_DIRECTORY="" PEER_HOST_BUILD_ROOT="" PEER_HOST_SLICE_CAP="" THIS_HOST_CARGO_BIN_DIRECTORY="" THIS_HOST_BUILD_ROOT=""

peer_host_inside_selftest() { # 调用方的 --selftest 起的：PEER_HOST_SELFTEST_MARK 指的文件在、第一行对得上（文件头「只供测试的开关」）
  [[ -n "${PEER_HOST_SELFTEST_MARK:-}" && -f "$PEER_HOST_SELFTEST_MARK" ]] \
    && [[ "$(head -n 1 -- "$PEER_HOST_SELFTEST_MARK")" == "$PEER_HOST_SELFTEST_MARK_CONTENT" ]]
}

peer_host_this_machine_addresses() { # 打这一台网卡上的地址，一行一个（ip -o addr show 第四栏去掉 /前缀长度；文件头「主机数组」）；ip 跑不起来退 1
  local listing
  listing="$(ip -o addr show 2>/dev/null)" || return 1
  awk '{ split($4, part, "/"); if (part[1] != "") print part[1] }' <<<"$listing"
}

peer_host_resolve_addresses() { # <HOSTS 的值> [<这一台的地址，空白隔开>]：照这一台网卡上的地址认主机数组里哪一项是自己（文件头「主机数组」）。
  # 设 peer_host_self_matches（命中几项）、THIS_HOST_ADDRESS、PEER_HOST_ADDRESS（不是自己的头一项）、PEER_HOST_OTHER_ADDRESSES（不是自己的各项，次序同数组）；
  # 恰好命中一项退 0；命中 0 项或不止一项退 1（三个地址变量置空）；读不出这一台的地址退 2。第二个参数只给判法的自证换掉这一台的地址用
  local listing item
  local -a items=()
  local -A local_address=()
  THIS_HOST_ADDRESS="" PEER_HOST_ADDRESS="" PEER_HOST_OTHER_ADDRESSES=() peer_host_self_matches=0
  if [[ -n "${2:-}" ]]; then listing="$(tr -s '[:space:]' '\n' <<<"$2")"; else listing="$(peer_host_this_machine_addresses)" || return 2; fi
  while IFS= read -r item; do [[ -n "$item" ]] && local_address[$item]=1; done <<<"$listing"
  read -r -a items <<<"$1"
  for item in "${items[@]}"; do
    if [[ -n "${local_address[$item]+set}" ]]; then
      peer_host_self_matches=$((peer_host_self_matches + 1))
      THIS_HOST_ADDRESS="$item"
    else
      PEER_HOST_OTHER_ADDRESSES+=("$item")
    fi
  done
  if (( peer_host_self_matches != 1 )); then THIS_HOST_ADDRESS="" PEER_HOST_OTHER_ADDRESSES=(); return 1; fi
  PEER_HOST_ADDRESS="${PEER_HOST_OTHER_ADDRESSES[0]:-}"
  return 0
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
  peer_host_load_by_host || return 1
  # 只供测试的开关只在调用方的自证里认（文件头「只供测试的开关」）；弄坏开关 this-machine-unmarked 下没有标记也认
  peer_host_is_this_machine=0
  if peer_host_inside_selftest || [[ ",${PEER_HOST_LIB_BREAK:-}," == *,this-machine-unmarked,* ]]; then
    peer_host_is_this_machine="${SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE:-0}"
  fi
}

peer_host_configured_value() { # <配置文件> <键>：打以「键=」起头的最后一行的值，去一层成对引号（读法同配置判法；文件头「按台的三个键」）；没有这一行或读不了退 1
  local line value="" found=1
  [[ -r "$1" ]] || return 1
  while IFS= read -r line || [[ -n "$line" ]]; do
    if [[ "$line" == "$2="* ]]; then value="${line#*=}"; found=0; fi
  done < "$1"
  (( found == 0 )) || return 1
  if [[ "$value" =~ ^\'(.*)\'$ || "$value" =~ ^\"(.*)\"$ ]]; then value="${BASH_REMATCH[1]}"; fi
  printf '%s' "$value"
}

peer_host_value_for_host() { # <按台的键的整值> <地址>：打那一台那一项的值（每项从第一个 = 切；文件头「按台的三个键」）；没有那一项退 1。
  # 弄坏开关 by-host-first-entry 下不看地址、取头一项
  local item
  local -a items=()
  read -r -a items <<<"$1"
  for item in ${items[@]+"${items[@]}"}; do
    [[ "$item" == *=* ]] || continue
    if [[ "${item%%=*}" == "$2" || ",${PEER_HOST_LIB_BREAK:-}," == *,by-host-first-entry,* ]]; then printf '%s' "${item#*=}"; return 0; fi
  done
  return 1
}

peer_host_load_by_host() { # 照 CRASH_POINT_REPLAY_SHARD_CONFIGURATION_FILE 那一份的三个按台的键，设另一台（PEER_HOST_ADDRESS）与这一台（THIS_HOST_ADDRESS）那一项（文件头「按台的三个键」）。
  # 要的那一项没有（键没写、少了那一台）打一行原因（只点名键与哪一台，不打配置里的值）与出路，退 1
  local key text value side address variable entry
  local -A by_host_text=()
  PEER_HOST_CARGO_BIN_DIRECTORY="" PEER_HOST_BUILD_ROOT="" PEER_HOST_SLICE_CAP="" THIS_HOST_CARGO_BIN_DIRECTORY="" THIS_HOST_BUILD_ROOT=""
  for key in CARGO_BIN_DIRECTORY_BY_HOST BUILD_ROOT_BY_HOST SLICE_MEMORY_CAP_BY_HOST; do
    if ! text="$(peer_host_configured_value "${CRASH_POINT_REPLAY_SHARD_CONFIGURATION_FILE:-}" "$key")"; then
      echo "  ✗ 多机配置里读不到按台的键 $key（三个按台的键都要写）"
      echo "     → 怎么办：照仓根 multi-host.env.example 写 $key=<地址>=<值> <地址>=<值>（HOSTS 里每一台恰好一项）；单跑 bash scripts/crash/crash-point-replay-shard-configuration-check.sh 看判法怎么说"
      return 1
    fi
    by_host_text[$key]="$text"
  done
  for entry in PEER_HOST_CARGO_BIN_DIRECTORY:CARGO_BIN_DIRECTORY_BY_HOST:peer PEER_HOST_BUILD_ROOT:BUILD_ROOT_BY_HOST:peer PEER_HOST_SLICE_CAP:SLICE_MEMORY_CAP_BY_HOST:peer \
      THIS_HOST_CARGO_BIN_DIRECTORY:CARGO_BIN_DIRECTORY_BY_HOST:this THIS_HOST_BUILD_ROOT:BUILD_ROOT_BY_HOST:this; do
    IFS=: read -r variable key side <<<"$entry"
    if [[ "$side" == peer ]]; then address="${PEER_HOST_ADDRESS:-}"; else address="${THIS_HOST_ADDRESS:-}"; fi
    if ! value="$(peer_host_value_for_host "${by_host_text[$key]}" "$address")"; then
      echo "  ✗ 多机配置的 $key 里没有$([[ "$side" == peer ]] && echo 第二台 || echo 本机)那一项（HOSTS 里每一台恰好一项）"
      echo "     → 怎么办：照仓根 multi-host.env.example 补上那一台的 <地址>=<值>；单跑 bash scripts/crash/crash-point-replay-shard-configuration-check.sh 看判法怎么说"
      return 1
    fi
    printf -v "$variable" '%s' "$value"
  done
}

peer_host_redact() { # 标准输入原样转到标准输出，配置里两台的编译根、cargo 目录与地址换成「第二台的目录」「第二台的 cargo 目录」「第二台」「本机的目录」「本机的 cargo 目录」「本机」
  if [[ ",${PEER_HOST_LIB_BREAK:-}," == *,no-redact,* ]]; then cat; return; fi
  PEER_HOST_REDACT_HOST="${PEER_HOST_ADDRESS:-}" PEER_HOST_REDACT_THIS="${THIS_HOST_ADDRESS:-}" PEER_HOST_REDACT_DIRECTORY="${PEER_HOST_BUILD_ROOT:-}" PEER_HOST_REDACT_CARGO="${PEER_HOST_CARGO_BIN_DIRECTORY:-}" \
    PEER_HOST_REDACT_THIS_DIRECTORY="${THIS_HOST_BUILD_ROOT:-}" PEER_HOST_REDACT_THIS_CARGO="${THIS_HOST_CARGO_BIN_DIRECTORY:-}" \
    PEER_HOST_REDACT_DECODE_ERRORS="$([[ ",${PEER_HOST_LIB_BREAK:-}," == *,redact-strict-utf8,* ]] && echo strict || echo surrogateescape)" \
    python3 -c '
import io, os, re, sys
decode_errors = os.environ["PEER_HOST_REDACT_DECODE_ERRORS"]   # surrogateescape：不是合法 UTF-8 的字节原样过，不让一行坏字节吞掉整块输出；strict 只在弄坏开关 redact-strict-utf8 下
source = io.TextIOWrapper(sys.stdin.buffer, encoding="utf-8", errors=decode_errors, newline="")
sink = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors=decode_errors, newline="")
replacements = [(os.environ["PEER_HOST_REDACT_DIRECTORY"], "第二台的目录"), (os.environ["PEER_HOST_REDACT_CARGO"], "第二台的 cargo 目录"),
                (os.environ["PEER_HOST_REDACT_THIS_DIRECTORY"], "本机的目录"), (os.environ["PEER_HOST_REDACT_THIS_CARGO"], "本机的 cargo 目录")]
replacements = sorted((pair for pair in replacements if pair[0]), key=lambda pair: -len(pair[0]))
hosts = [(os.environ[name], shown) for name, shown in (("PEER_HOST_REDACT_HOST", "第二台"), ("PEER_HOST_REDACT_THIS", "本机")) if os.environ[name]]
host_patterns = [(re.compile(r"(?<![\w.:-])" + re.escape(host) + r"(?![\w:-]|[.][0-9])"), shown) for host, shown in hosts]
for line in source:
    for secret, shown in replacements:
        line = line.replace(secret, shown)
    for host_pattern, shown in host_patterns:
        line = host_pattern.sub(shown, line)
    sink.write(line)
    sink.flush()'
}

peer_host_run() { # <目录> <命令文本>：在第二台的这个目录里 bash -c 跑（PATH 前面加 PEER_HOST_CARGO_BIN_DIRECTORY：按台的 CARGO_BIN_DIRECTORY_BY_HOST 里第二台那一项），
  # 标准错误换掉私有的字再转出；退出码是那条命令的
  if [[ "$peer_host_is_this_machine" == 1 ]]; then
    # 和 ssh 一样等标准输出关上才返回（经 cat 转出）：留在后台、还握着输出的进程会让它一直等
    (cd "$1" && env -i HOME="$HOME" USER="${USER:-}" LOGNAME="${LOGNAME:-}" LANG="${LANG:-C.UTF-8}" XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-}" \
        DBUS_SESSION_BUS_ADDRESS="${DBUS_SESSION_BUS_ADDRESS:-}" ${CARGO_HOME:+CARGO_HOME="$CARGO_HOME"} ${RUSTUP_HOME:+RUSTUP_HOME="$RUSTUP_HOME"} \
        PATH="$PEER_HOST_CARGO_BIN_DIRECTORY:$PATH" bash -c "$2") 2> >(peer_host_redact >&2) | cat
    return "${PIPESTATUS[0]}"
  fi
  ssh -o BatchMode=yes "$PEER_HOST_ADDRESS" "cd $(printf '%q' "$1") && PATH=$(printf '%q' "$PEER_HOST_CARGO_BIN_DIRECTORY"):\$PATH bash -c $(printf '%q' "$2")" \
    </dev/null 2> >(peer_host_redact >&2)
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
    echo "     → 怎么办：看多机配置 CARGO_BIN_DIRECTORY_BY_HOST 里第二台那一项的目录下有没有 rustup、ssh 连不连得上"
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
    echo "     → 怎么办：看多机配置 CARGO_BIN_DIRECTORY_BY_HOST 里第二台那一项的目录下有没有 rustc 与 cargo、ssh 连不连得上"
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

peer_host_missing_path_dependencies() { # <那棵树的根> <缺的依赖名，「、」隔开> <cargo 的报错>：把缺的依赖分成两种打出来（文件头「离线依赖」）：
  # 在 Cargo.toml 里写成 path、那个目录里又没有 Cargo.toml 的，一行一个「path<TAB><名><TAB><目录，树里的写相对树根的路径><TAB><写它的那份 Cargo.toml，同上>」，次序照缺的次序；
  # 最后一行「rest<TAB><别的缺的，「、」隔开，没有就空>」。
  # 读的 Cargo.toml：报错里「as a dependency of package `… (<目录>)`」「failed to load manifest for workspace member `<目录>`」点名的包，加树根与往下两层的（不进 target/）；
  # 每份的 [dependencies]、[dev-dependencies]、[build-dependencies]、[target.*.…] 与 [workspace.dependencies]，键或 package = 是那个名字、带 path 的那一项，path 照那份 Cargo.toml 所在目录解。
  # 树在共享根下、两台同一份，在本机读；读不了、解不开的那份跳过
  python3 - "$1" "$2" "$3" <<'PY_PATH_DEPENDENCIES'
import glob, os, re, sys, tomllib
tree, wanted, errors = os.path.realpath(sys.argv[1]), [name for name in sys.argv[2].split("、") if name], sys.argv[3]
directories = re.findall(r"as a dependency of package `[^`(]*\(([^)`]+)\)`", errors) + re.findall(r"failed to load manifest for workspace member `([^`]+)`", errors)
manifests = [os.path.join(directory, "Cargo.toml") for directory in directories] + [os.path.join(tree, "Cargo.toml")]
manifests += sorted(path for depth in ("*", "*/*") for path in glob.glob(os.path.join(tree, depth, "Cargo.toml")) if "target" not in os.path.relpath(path, tree).split(os.sep))
def shown(path):
    relative = os.path.relpath(path, tree)
    return path if relative == ".." or relative.startswith("../") else relative
def dependency_tables(document):
    sections = ("dependencies", "dev-dependencies", "build-dependencies")
    yield from (document.get(section, {}) for section in sections)
    for target in document.get("target", {}).values():
        yield from (target.get(section, {}) for section in sections)
    yield document.get("workspace", {}).get("dependencies", {})
found, seen = {}, set()
for manifest in manifests:
    manifest = os.path.realpath(manifest)
    if manifest in seen:
        continue
    seen.add(manifest)
    try:
        with open(manifest, "rb") as handle:
            document = tomllib.load(handle)
    except (OSError, tomllib.TOMLDecodeError):
        continue
    for table in dependency_tables(document):
        for key, value in (table.items() if isinstance(table, dict) else ()):
            if not isinstance(value, dict) or not isinstance(value.get("path"), str):
                continue
            name = key if key in wanted else value.get("package", key)   # 改了名的依赖：cargo 报的是键还是包名都认
            if name not in wanted or name in found:
                continue
            directory = os.path.normpath(os.path.join(os.path.dirname(manifest), value["path"]))
            if not os.path.isfile(os.path.join(directory, "Cargo.toml")):
                found[name] = (directory, manifest)
for name in wanted:
    if name in found:
        print(f"path\t{name}\t{shown(found[name][0])}\t{shown(found[name][1])}")
print("rest\t" + "、".join(name for name in wanted if name not in found))
PY_PATH_DEPENDENCIES
}

peer_host_check_offline_dependencies() { # <那棵树的根>（共享根下，两台同一个绝对路径）：树根有 Cargo.toml 的，在第二台那里空跑 cargo metadata --offline，过了退 0；
  # 缺的依赖在 Cargo.toml 里写成 path、树里没有那个目录的打「树里缺 path 依赖 <路径>，两台都编不了」与出路（把那个目录补进树）；别的缺离线依赖打「第二台缺依赖 <名>」与同步办法；
  # 认不出缺哪个、到点没跑完打原因与出路；这几种都退 1。树根没有 Cargo.toml 的打一句、退 0（文件头「离线依赖」）。
  # 第二台的报错经 peer_host_run 换掉私有的字。弄坏开关 offline-dependencies-unchecked 下不跑、退 0；
  # path-dependencies-as-peer-cache 下不认 path 依赖，缺的一律报第二台缺依赖
  local tree_root="$1" errors status missing classified="" kind name directory manifest path_reported=0
  if [[ ",${PEER_HOST_LIB_BREAK:-}," == *,offline-dependencies-unchecked,* ]]; then
    echo "  ! 弄坏开关 PEER_HOST_LIB_BREAK=offline-dependencies-unchecked：不在第二台空跑 cargo metadata --offline"
    return 0
  fi
  if ! peer_host_run "$tree_root" "test -f Cargo.toml" >/dev/null 2>&1; then
    echo "  · 那棵树的树根没有 Cargo.toml：不核离线依赖"
    return 0
  fi
  # --filter-platform 第二台的 host：不带它 cargo metadata 连别的平台才用的依赖（android_system_properties 这类）也要下载，缓存里本来就没有，次次误报
  if errors="$(peer_host_run "$tree_root" "timeout $PEER_HOST_METADATA_SECONDS cargo metadata --offline --format-version 1 --filter-platform \"\$(rustc -vV | sed -n 's/^host: //p')\" >/dev/null" 2>&1)"; then
    echo "  · 第二台那里 cargo metadata --offline 过了：离线依赖齐"
    return 0
  else
    status=$?
  fi
  missing="$(grep -oE 'no matching package named `[^`]+`|failed to get `[^`]+` as a dependency|failed to load manifest for dependency `[^`]+`|failed to download `[^`]+`|failed to select a version for the requirement `[^`]+' <<<"$errors" \
    | grep -oE '`[^` =]+' | tr -d '`' | awk '!seen[$0]++ { printf "%s%s", (joined++ ? "、" : ""), $0 }')"
  if [[ -n "$missing" && ",${PEER_HOST_LIB_BREAK:-}," == *,path-dependencies-as-peer-cache,* ]]; then
    echo "  ! 弄坏开关 PEER_HOST_LIB_BREAK=path-dependencies-as-peer-cache：不认 path 依赖，缺的一律当第二台缺缓存报"
  elif [[ -n "$missing" ]]; then
    if ! classified="$(peer_host_missing_path_dependencies "$tree_root" "$missing" "$errors")"; then
      echo "  ! 认不了缺的依赖里哪些是 path 依赖（读 Cargo.toml 的那一步退非 0），照第二台缺缓存报"
      classified=""
    fi
  fi
  while IFS=$'\t' read -r kind name directory manifest; do
    if [[ "$kind" == path ]]; then
      echo "  ✗ 树里缺 path 依赖 $directory：$manifest 里 $name 写成 path 依赖，那个目录里没有 Cargo.toml，两台都编不了（不是第二台缺缓存），这一趟不跑"
      path_reported=1
    elif [[ "$kind" == rest ]]; then
      missing="$name"
    fi
  done <<<"$classified"
  if (( path_reported )); then
    echo "     → 怎么办：把上面那几个目录补进这棵树（仓副本照 scripts/hosts/lib_repository_copy.py 的 copy_closure 拷，它连 path 依赖指到的目录一起拷；手拷的副本补拷那几个目录，不带 target），再跑同一条命令；不用同步 cargo 缓存"
    if [[ -z "$missing" ]]; then return 1; fi
  fi
  if [[ -n "$missing" ]]; then
    echo "  ✗ 第二台缺依赖 $missing：在第二台那里 cargo metadata --offline 解析不出（第二台的 cargo 缓存里没有，命令跑到 cargo 那一步会退 101），这一趟不跑"
    echo "     → 怎么办：同步第二台的 cargo 缓存：本机仓根先跑 cargo fetch（把依赖拉进本机的 cargo 缓存，默认 ~/.cargo），再把本机 cargo 缓存里的 registry/ 与 git/ 两个目录 rsync -a 到第二台 cargo 缓存的同一位置（不带 --delete）；第二台连得上网的，也可以在第二台那份仓里跑一次 cargo fetch。同步完再跑同一条命令"
    return 1
  fi
  if (( status == 124 )); then
    echo "  ✗ 第二台那里 cargo metadata --offline $PEER_HOST_METADATA_SECONDS 秒没跑完（多半在等第二台 cargo 的包缓存锁），这一趟不跑"
    echo "     → 怎么办：看第二台有没有别的 cargo 正在拉依赖或编译，等它放锁再跑同一条命令"
    return 1
  fi
  echo "  ✗ 第二台那里 cargo metadata --offline 退 $status，从报错里认不出缺哪个依赖，这一趟不跑；cargo 报的末几行（换掉了私有的字）："
  printf '%s\n' "$errors" | tail -n 8 | sed 's/^/      /'   # gate-lint:detail
  echo "     → 怎么办：照上面 cargo 报的修；缺依赖的同步办法是本机 cargo fetch 之后把本机 cargo 缓存的 registry/ 与 git/ rsync -a 到第二台同一位置；修好再跑同一条命令"
  return 1
}

peer_host_sop_source() { # <树根> <驱动所在的仓根>：打规范副本目录（multi-host-run.sh 的自证搭小仓时链过去）——树里有就用树里的，没有用驱动所在仓的，再没有用那份仓的 git common-dir 所在的主仓的；都没有退 1
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

peer_host_origin_quiesce_holder() { # 这个驱动正经 scripts/crash/gpu-quiesce-run.sh 起（那一层照配置清了场、回读过了）：GPU_QUIESCE_HOLDER 指的登记文件在、
  # 文件名以这个驱动的父进程号加「-」起头（gpu-quiesce-run.sh 文件头「给调用方与派发器的接口」）
  [[ -n "${GPU_QUIESCE_HOLDER:-}" && -e "$GPU_QUIESCE_HOLDER" && "${GPU_QUIESCE_HOLDER##*/}" == "$PPID-"* ]]
}

peer_host_carried_environment() { # 打第二台那条命令前面要加的环境赋值（printf %q 过，空格隔开，末尾带一个空格；文件头「两台用同一份多机配置」）：
  # 读到了本机这一份配置的（peer_host_load_configuration 设了 CRASH_POINT_REPLAY_SHARD_CONFIGURATION_FILE）SINGLEFS_MULTI_HOST_CONFIG=<它的绝对路径>（共享根下，第二台读同一份）；
  # 起活这一侧照配置清过场、回读过了的（这一趟 peer_host_quiesce 回读过了，或 peer_host_origin_quiesce_holder）GPU_QUIESCE_ORIGIN_READ_BACK=1。不带认台的角色变量（两台对称）。
  # 弄坏开关 origin-read-back-always 下一律带 GPU_QUIESCE_ORIGIN_READ_BACK=1；configuration-not-passed 下不带 SINGLEFS_MULTI_HOST_CONFIG
  local text=""
  if [[ -n "${CRASH_POINT_REPLAY_SHARD_CONFIGURATION_FILE:-}" && ",${PEER_HOST_LIB_BREAK:-}," != *,configuration-not-passed,* ]]; then
    text+="SINGLEFS_MULTI_HOST_CONFIG=$(printf '%q' "$CRASH_POINT_REPLAY_SHARD_CONFIGURATION_FILE") "
  fi
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
  # 弄坏开关 restore-after-run 下跑配置的复原命令（只给证红用）
  (( peer_host_quiesced )) || return 0
  peer_host_quiesced=0
  if [[ ",${PEER_HOST_LIB_BREAK:-}," == *,restore-after-run,* ]]; then
    bash -c "${QUIESCE_START_COMMAND:-}" >/dev/null 2>&1
    echo "  ! 弄坏开关 PEER_HOST_LIB_BREAK=restore-after-run：照配置跑了复原命令"
    return 0
  fi
  echo "  · 清了场、没复原：清场停下的服务一直停着；要复原手动跑私有复原脚本（配置的 QUIESCE_START_COMMAND），再跑 QUIESCE_STARTED_CHECK_COMMAND 回读到它退 0"
}

# 下面六个函数也经 declare -f 带到第二台上跑：只用 bash、ps、sleep、tr 与 /proc，不引这六个之外的函数；
#   peer_host_stop_process_tree 与 peer_host_stop_attempt_leftovers 把这一次最后发的信号记进 peer_host_stop_signal（none、TERM、KILL，只往重里改），起它们的那一段先置 none
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
  [[ "${peer_host_stop_signal:-none}" == KILL ]] || peer_host_stop_signal=TERM
  for pid in "${pids[@]}"; do peer_host_signal_one CONT "$pid"; done
  for (( waited = 0; waited < 300; waited++ )); do
    survivors=()
    for pid in "${pids[@]}"; do peer_host_process_is_alive "$pid" && survivors+=("$pid"); done
    (( ${#survivors[@]} == 0 )) && break
    sleep 0.1
  done
  (( ${#survivors[@]} == 0 )) || peer_host_stop_signal=KILL
  for pid in ${survivors[@]+"${survivors[@]}"}; do peer_host_signal_one KILL "$pid"; done
  sleep 0.5
  survivors=()
  for pid in "${pids[@]}"; do peer_host_process_is_alive "$pid" && survivors+=("$pid"); done
  if (( ${#survivors[@]} > 0 )); then echo "进程 $1 那一棵里 ${survivors[*]} 发了 TERM、KILL 还在"; return 1; fi
  echo "进程 $1 连同 $(( ${#pids[@]} - 1 )) 个后代都停了"
}
peer_host_attempt_leftovers() { # <会话号> <会话首进程的启动时刻> <尝试标签>：打这一趟还留在这一台上的进程号，一行一个（文件头「这一趟留下的进程」）：
  # 会话号是 <会话号> 的，与环境里带 TASK_SERVER_ATTEMPT_LABEL=<尝试标签> 的；<会话号> 那个进程此刻在、启动时刻对不上（号被重用了）或启动时刻没给的，不按会话找；
  # 会话号或标签给空的那一样不找。不列僵尸、自己与自己的祖先。只用 bash 内建读 /proc（扫的时候不起子进程）。读不了 /proc（列不出进程目录、读不了自己的 environ）退 2
  local session="$1" leader_start="$2" label="$3" entry pid stat item current by_session=0
  local -a fields=() environment=()
  local -A skipped=()
  [[ -r /proc/self/environ && -d "/proc/$BASHPID" ]] || return 2
  current="$BASHPID"
  while [[ "$current" =~ ^[1-9][0-9]*$ ]] && (( current > 1 )) && [[ -z "${skipped[$current]+set}" ]]; do
    skipped[$current]=1
    read -r stat 2>/dev/null < "/proc/$current/stat" || break
    read -r -a fields <<<"${stat##*) }"
    current="${fields[1]:-0}"
  done
  if [[ "$session" =~ ^[1-9][0-9]*$ ]] && (( session > 1 )); then
    by_session=1
    if read -r stat 2>/dev/null < "/proc/$session/stat"; then
      read -r -a fields <<<"${stat##*) }"
      [[ -n "$leader_start" && "${fields[19]:-}" == "$leader_start" ]] || by_session=0
    fi
  fi
  for entry in /proc/[0-9]*; do
    pid="${entry#/proc/}"
    [[ -z "${skipped[$pid]+set}" ]] || continue
    read -r stat 2>/dev/null < "$entry/stat" || continue
    read -r -a fields <<<"${stat##*) }"
    [[ "${fields[0]:-}" != Z ]] || continue
    if (( by_session )) && [[ "${fields[3]:-}" == "$session" ]]; then echo "$pid"; continue; fi
    [[ -n "$label" ]] || continue
    mapfile -d '' -t environment 2>/dev/null < "$entry/environ" || continue
    for item in ${environment[@]+"${environment[@]}"}; do
      if [[ "$item" == "TASK_SERVER_ATTEMPT_LABEL=$label" ]]; then echo "$pid"; break; fi
    done
  done
  return 0
}
peer_host_stop_attempt_leftovers() { # <会话号> <会话首进程的启动时刻> <尝试标签>：停这一趟留下的进程（peer_host_attempt_leftovers 列的；文件头「这一趟留下的进程」）：
  # 一遍：逐个冻住（STOP），冻住之后再列、新列出的接着冻，至多 20 轮到列不出新的为止；从后往前逐个 TERM、放开（CONT），至多等 30 秒，还在的逐个 KILL。
  # 收到 TERM 收尾时又起的进程（内存包装停自己的 scope 这类）由下一遍收，至多三遍、到一遍里列不出新的为止；停完再列一遍。
  # 打一行「PEER_HOST_LEFTOVERS stopped=<停下了几个> signal=<最后发的：none、TERM、KILL> remaining=<再列一遍还有几个>」，还有的另打一行
  # 「PEER_HOST_LEFTOVERS_REMAINING <进程号>（<命令行>）…」；一个不剩退 0，还有剩的退 1，读不了 /proc 打「PEER_HOST_LEFTOVERS unreadable」退 2
  local pid pass round index waited status new stopped=0
  local -a listed=() batch=() survivors=()
  local -A seen=()
  for (( pass = 0; pass < 3; pass++ )); do
    batch=()
    for (( round = 0; round < 20; round++ )); do
      mapfile -t listed < <(peer_host_attempt_leftovers "$1" "$2" "$3"; echo "status=$?")
      status="${listed[${#listed[@]} - 1]#status=}"
      unset 'listed[${#listed[@]} - 1]'
      if [[ "$status" != 0 ]]; then echo "PEER_HOST_LEFTOVERS unreadable"; return 2; fi
      new=0
      for pid in ${listed[@]+"${listed[@]}"}; do
        [[ -z "${seen[$pid]+set}" ]] || continue
        seen[$pid]=1
        batch+=("$pid")
        new=1
        peer_host_signal_one STOP "$pid"
      done
      (( new )) || break
    done
    (( ${#batch[@]} > 0 )) || break
    for (( index = ${#batch[@]} - 1; index >= 0; index-- )); do peer_host_signal_one TERM "${batch[index]}"; done
    [[ "${peer_host_stop_signal:-none}" == KILL ]] || peer_host_stop_signal=TERM
    for pid in "${batch[@]}"; do peer_host_signal_one CONT "$pid"; done
    for (( waited = 0; waited < 300; waited++ )); do
      survivors=()
      for pid in "${batch[@]}"; do peer_host_process_is_alive "$pid" && survivors+=("$pid"); done
      (( ${#survivors[@]} == 0 )) && break
      sleep 0.1
    done
    if (( ${#survivors[@]} > 0 )); then
      peer_host_stop_signal=KILL
      for pid in "${survivors[@]}"; do peer_host_signal_one KILL "$pid"; done
      sleep 0.5
    fi
    for pid in "${batch[@]}"; do peer_host_process_is_alive "$pid" || stopped=$((stopped + 1)); done
  done
  mapfile -t listed < <(peer_host_attempt_leftovers "$1" "$2" "$3"; echo "status=$?")
  status="${listed[${#listed[@]} - 1]#status=}"
  unset 'listed[${#listed[@]} - 1]'
  if [[ "$status" != 0 ]]; then echo "PEER_HOST_LEFTOVERS unreadable"; return 2; fi
  echo "PEER_HOST_LEFTOVERS stopped=$stopped signal=${peer_host_stop_signal:-none} remaining=${#listed[@]}"
  (( ${#listed[@]} == 0 )) && return 0
  printf 'PEER_HOST_LEFTOVERS_REMAINING'
  for pid in "${listed[@]}"; do printf ' %s（%s）' "$pid" "$(tr '\0' ' ' < "/proc/$pid/cmdline" 2>/dev/null)"; done
  echo
  return 1
}

peer_host_stop_attempt_scope() { # <scope 单元名文件>：停这一趟内存包装的 scope（文件头「这一趟留下的进程」；multi-host-run.sh 文件头 ⑥），经 declare -f 带到第二台上跑。
  # 文件不在或是空的打「PEER_HOST_SCOPE none」退 0（这一趟没经内存包装、或包装还没起 scope）；名字不是这一层包装起的样子（singlefs-memory-cap-<号>-<号>）打
  # 「PEER_HOST_SCOPE unrecognized」退 2。停之前读一次 cgroup.procs（目录是 /sys/fs/cgroup 接上 ControlGroup），systemctl --user stop 之后回读 is-active 的输出与
  # cgroup.procs 的内容（.claude/rules/service-stop-readback.md；目录已不在算空）：打一行「PEER_HOST_SCOPE unit=<名> before=<停之前几个> state=<is-active 的输出> remaining=<停之后几个>」，
  # 还有的每个另打一行「PEER_HOST_SCOPE_REMAINING <进程号> <命令行前 120 字>」；is-active 是 inactive 或 failed 而一个不剩退 0，不然退 1。只停这一个单元，不按名字挑别的
  local unit="" group="" before="" after="" state pid command_line
  IFS= read -r unit 2>/dev/null < "$1" || unit="${unit:-}"
  if [[ -z "$unit" ]]; then echo "PEER_HOST_SCOPE none"; return 0; fi
  if [[ ! "$unit" =~ ^singlefs-memory-cap-[0-9]+-[0-9]+$ ]]; then echo "PEER_HOST_SCOPE unrecognized"; return 2; fi
  group="$(systemctl --user show -p ControlGroup --value "$unit.scope" 2>/dev/null)"
  [[ -n "$group" ]] && group="/sys/fs/cgroup$group"
  [[ -n "$group" && -d "$group" ]] && before="$(cat -- "$group/cgroup.procs" 2>/dev/null)"
  systemctl --user stop "$unit.scope" >/dev/null 2>&1   # 判的是下面回读的输出（单元已收尾时 stop 报找不到单元）
  state="$(systemctl --user is-active "$unit.scope" 2>/dev/null)"
  [[ -n "$group" && -d "$group" ]] && after="$(cat -- "$group/cgroup.procs" 2>/dev/null)"
  echo "PEER_HOST_SCOPE unit=$unit before=$(grep -c . <<<"$before") state=${state:-空} remaining=$(grep -c . <<<"$after")"
  if [[ ( "$state" == inactive || "$state" == failed ) && -z "$after" ]]; then return 0; fi
  for pid in $after; do
    command_line="$(tr '\0' ' ' < "/proc/$pid/cmdline" 2>/dev/null)"
    echo "PEER_HOST_SCOPE_REMAINING $pid ${command_line:0:120}"
  done
  return 1
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
  detached_text="${close_text}awk '{print \$22}' /proc/\$\$/stat > $(printf '%q' "$pid_file.start") || exit 1; echo \$\$ > $(printf '%q' "$pid_file"); $command_text > $(printf '%q' "$output_file") 2>&1; echo \$? > $(printf '%q' "$exit_file")"
  # 只让 setsid nohup 那一条进后台（它的输入输出都重定向了）：整串放进后台的话，后台那个子进程握着 ssh 的输出通道，ssh 要等它跑完才返回
  peer_host_run "$directory" "rm -f -- $(printf '%q' "$pid_file") $(printf '%q' "$exit_file") $(printf '%q' "$output_file") || exit 1; setsid nohup bash -c $(printf '%q' "$detached_text") < /dev/null > /dev/null 2>&1 &"
}

peer_host_wait_detached() { # <进程号文件> <退出码文件> <轮询秒数> [<每一轮之后调的函数>]：等第二台那一份跑完，结局写进 peer_host_detached_outcome：
  # exit <码>（退出码文件头一个词；停下的那一方补写的回执后面另有两个词，文件头「停下的退出回执」）；vanished（进程没了、没留退出码，连续 3 次）；unreachable（连续 60 次 ssh 不通）；never-started（连续 60 次既没有进程号文件也没有退出码）。
  # 给了函数的，每一轮看完之后调它（参数 round，最后那一轮也调），等到结局之后再调一次（参数 final）；它的退出码不看（文件头「peer_host_wait_detached 的第四个参数」）
  local pid_file="$1" exit_file="$2" poll_seconds="$3" follow_function="${4:-}" poll unreachable_rounds=0 vanished_rounds=0 starting_rounds=0 previous=""
  peer_host_detached_outcome=""
  while [[ -z "$peer_host_detached_outcome" ]]; do
    if poll="$(peer_host_run / "if [[ -s $(printf '%q' "$exit_file") ]]; then read -r peer_host_exit_code _ < $(printf '%q' "$exit_file"); echo \"exit \$peer_host_exit_code\"; elif [[ -s $(printf '%q' "$pid_file") && -d /proc/\$(cat $(printf '%q' "$pid_file")) ]]; then echo running; elif [[ -s $(printf '%q' "$pid_file") ]]; then echo vanished; else echo starting; fi" 2>/dev/null)"; then
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
    if [[ -n "$follow_function" ]]; then "$follow_function" round; fi
    [[ -n "$peer_host_detached_outcome" ]] || sleep "$poll_seconds"
  done
  if [[ -n "$follow_function" ]]; then "$follow_function" final; fi
}

peer_host_stop_detached() { # <目录> <进程号文件> <退出码文件> <尝试标签> [keep]：停那一份连同这一趟留下的进程，停干净了补写退出回执（文件头「停下的退出回执」）；
  # 打一句结局，没停下退 1。起的那个 bash 还在（进程号文件里的号、启动时刻对得上、工作目录是这个目录）：照进程号连同后代停（peer_host_stop_process_tree）；
  # 不在（或号被重用了）而退出码文件里有回执的照收；都要再照会话号与尝试标签停这一趟留下的进程（peer_host_stop_attempt_leftovers）、停完再核。
  # 两样都停干净了、退出码文件还没有的，写一行「<退出码> stopped <最后发的信号>」再退 0。不按已停处理、退 1 的：进程号文件里读不出号或启动时刻（按会话核不了）、
  # 读不了 /proc、还有停不掉的、回执写不进。第五个参数写 keep 的（multi-host-run.sh 的弄坏开关 orphan-session-unverified）：起的那个 bash 不在而没有回执就不核、退 1
  peer_host_run "$1" "$(declare -f peer_host_signal_one peer_host_process_is_alive peer_host_process_tree peer_host_stop_process_tree peer_host_attempt_leftovers peer_host_stop_attempt_leftovers)
exit_file=$(printf '%q' "$3")
peer_host_stop_signal=none
for ((pid_rounds = 0; pid_rounds < 100; pid_rounds++)); do [[ -s $(printf '%q' "$2") ]] && break; sleep 0.1; done
detached_pid=\"\$(cat $(printf '%q' "$2") 2>/dev/null)\"
detached_start=\"\$(cat $(printf '%q' "$2.start") 2>/dev/null)\"
current_start=\"\$(awk '{print \$22}' /proc/\$detached_pid/stat 2>/dev/null)\"
if [[ \"\$detached_pid\" =~ ^[1-9][0-9]*$ && \"\$detached_pid\" != 1 && -n \"\$detached_start\" && \"\$current_start\" == \"\$detached_start\" && \"\$(readlink /proc/\$detached_pid/cwd 2>/dev/null)\" == \"\$(pwd -P)\" ]]; then
  peer_host_stop_process_tree \"\$detached_pid\" || exit 1
elif [[ -s \"\$exit_file\" ]]; then echo '已收到退出回执，接着核这一趟留下的进程'
elif [[ $(printf '%q' "${5:-}") == keep ]]; then echo '父进程无法确认且没有退出回执：保留这一趟，不按已停处理'; exit 1
elif [[ ! \"\$detached_pid\" =~ ^[1-9][0-9]*$ || \"\$detached_pid\" == 1 || -z \"\$detached_start\" ]]; then
  echo '进程号文件里读不出起的那个 bash 的号或启动时刻、又没有退出回执：按会话核不了，保留这一趟，不按已停处理'; exit 1
else echo \"起的那个 bash（进程号 \$detached_pid）不在了、没有退出回执：照会话号与尝试标签核这一趟留下的进程\"; fi
peer_host_stop_attempt_leftovers \"\$detached_pid\" \"\$detached_start\" $(printf '%q' "$4")
case \$? in
  0) ;;
  2) echo '读不了 /proc：核不了这一趟留下的进程，保留这一趟，不按已停处理'; exit 1 ;;
  *) echo '这一趟留下的进程还有停不掉的（上面 PEER_HOST_LEFTOVERS_REMAINING 那一行）：保留这一趟，不按已停处理'; exit 1 ;;
esac
[[ -s \"\$exit_file\" ]] && exit 0
case \"\$peer_host_stop_signal\" in KILL) receipt='137 stopped KILL' ;; TERM) receipt='143 stopped TERM' ;; *) receipt='143 stopped already-gone' ;; esac
printf '%s\n' \"\$receipt\" > \"\$exit_file\" || { echo '停干净了，可退出回执写不进：保留这一趟，不按已停处理'; exit 1; }
echo \"补写了退出回执「\$receipt」\""
}

# 这份库不单独调（只被 source）。直接执行时设着 PEER_HOST_LIB_BREAK 一律拒，不管值是什么：弄坏开关只在 source 它的驱动的 --selftest 里认（文件头「弄坏开关」）。
if [[ "${BASH_SOURCE[0]}" == "$0" && -n "${PEER_HOST_LIB_BREAK:-}" ]]; then
  echo "  ✗ PEER_HOST_LIB_BREAK=${PEER_HOST_LIB_BREAK} 是弄坏开关，只在 source 这份库的驱动的自证（multi-host-run.sh 的 --selftest）里认，直接执行设了就拒，这一趟什么都没做" >&2
  echo "  → 怎么办：去掉 PEER_HOST_LIB_BREAK 再跑；要证红就跑 PEER_HOST_LIB_BREAK=<项> bash scripts/hosts/multi-host-run.sh --selftest（项与红的格写在文件头「弄坏开关」一段）" >&2
  exit 2
fi
