#!/usr/bin/env bash
# multi-host-run.sh 的自证 multi_host_selftest（在临时目录里搭一棵小仓，「第二台」是本机上的另一个目录，逐个场景起驱动并核机读行、清理回执与退出码）。
# 被 multi-host-run.sh source 的函数库，只定义 multi_host_selftest 一个函数，不单独调；
#   自证读的全局变量与函数（multi_host_script_directory、MULTI_HOST_SELFTEST_MARK_CONTENT、PEER_HOST_SELFTEST_MARK_CONTENT、multi_host_break_is、multi_host_step 一类）都在 multi-host-run.sh 与 peer-host-lib.sh 里定义，
#   source 它的那一刻之后才有。

# ── 自证 ──────────────────────────────────────────────────────────────────────
# 在临时目录里搭一棵小仓（本仓的 .claude/scripts、scripts、规范副本的符号链接；git init、提交一次、整理成包，场景照它拷），
# 「第二台」是本机上的另一个目录（SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1）：场景配置 BUILD_ROOT_BY_HOST 里第二台那一项是 <自证目录>/peer-homes/<场景名>（不在共享根下），
# 场景目录 <自证目录>/<场景名> 整个是这一场景的共享根（NFS_SHARED_ROOTS），小仓、配置、工作根与共享数据根都在它下面；驱动在小仓里起（这一刻的工作目录是小仓的根）。
# 场景里起的驱动 PATH 最前面放假 rustup、rustc、cargo（rustup component list --installed 打一份写死的组件表，rustc -Vv、cargo -V 打写死的版本，别的用法退 1）：
#   两台的工具链与组件照比，不起真工具链（peer-host-lib.sh 文件头「工具链比对」）；场景的 HOME 换成自证自己的 home/、不带 CARGO_HOME 与 RUSTUP_HOME，
#   自证不靠调用方的家目录找工具链：本机直接起、调用方把 HOME 指到别处（服务在第二台起、HOME 是共享根下的一个目录）都照样判。
# 命令都经真的 run-with-memory-cap.sh（上限 256M）。逐格（弄坏开关见文件头，各让对应那一格判错）：
#   Ⓐ 不带 --peer ⇒ 用法错（退 2，说本机的任务经 task-server 交），命令没跑、第二台一个目录都没建
#   peer-host-lib.sh 的只供测试的开关：照场景配置调 peer_host_load_configuration 两遍，去掉 PEER_HOST_SELFTEST_MARK 那一遍 peer_host_is_this_machine 是 0、带着是 1；
#      PATH 最前面放连得上的假 ssh（判法认这个开关也要同一份标记，去掉标记那一遍它照连第二台）
#   Ⓑ --peer 跑成：退 0，在第二台同一个绝对路径（小仓的根）里跑、HEAD 与本机相同；经仓里的 run-with-memory-cap.sh 跑；命令退 7 ⇒ 驱动退 7；
#      --memory-cap 比 SLICE_MEMORY_CAP_BY_HOST 里第二台那一项大 ⇒ 拒（退 2）、命令没跑
#   peer-host-lib.sh 的按台的三个键（那份文件头「按台的三个键」）：一份写在自证临时目录里的配置（这一台那一项写在头一项、值里带 =）直接调 peer_host_load_by_host
#      ⇒ 另一台取另一台那一项、这一台取这一台那一项，值里的 = 照留；另一台那一项没写 ⇒ 退 1、点名键与「第二台」、不打配置里的值
#   标签（文件头 ⑤ 标签撞了与查标签时连不上那一句）：第二台 builds/ 下已经有同名目录 ⇒ 用法错（退 2），命令没跑、那个目录还在；
#      查标签时第二台连不上（ssh 自己退 255）⇒ 退 255、说连不上，不当用法错，命令没跑、builds/ 下没建这一趟的目录。「第二台」是本机的目录、不经 ssh：
#      第二台的 PATH 最前面（场景的 peer-bin）放假 bash，-c 的命令以查标签那一句起头、点名标签 label-unreachable 的退 255，别的转给真 bash
#   Ⓒ 第二台的 rustc -Vv 不同、或第二台缺 clippy 组件 ⇒ 拒（退 255，说两台的工具链不同），命令没跑、第二台没建这一趟的目录；
#      离线依赖（peer-host-lib.sh 文件头「离线依赖」）：小仓树根放一份 Cargo.toml，第二台的 PATH 最前面放假 cargo（metadata 记下目录与参数；场景里有 metadata-missing
#      就照 cargo 的原话报 no matching package named `vstd`、退 101；-V 这类转给假工具链的 cargo，工具链照比）：报缺 ⇒ 退 255、说第二台缺依赖 vstd 与 cargo fetch 的同步办法、
#      命令没跑、没建这一趟的目录；不报缺 ⇒ 照跑、退 0，假 cargo 记下在小仓的根跑过 metadata --offline；
#      缺的是 path 依赖（小仓的成员 crates/demo 写 served = { path = "../../served" }、served/ 不在，假 cargo 照 cargo 的原话报取不到 served）⇒ 退 255、
#      说树里缺 path 依赖 served、两台都编不了，不说第二台缺依赖，命令没跑
#   Ⓔ Ⓑ 那几趟跑完（含退 7 那一趟）第二台 runs/ 与 builds/ 下什么都不剩
#   Ⓕ 第二台那条命令在跑时给驱动（只给它自己，kill "$!"）发 TERM ⇒ 120 秒内退 143，第二台那条命令不在了，runs/ 与 builds/ 下什么都不剩；
#      第二台那条命令只握 0、1、2（不握驱动那一头的管道：握着的话，起它那一步要等它跑完才回来）
# 场景的多机配置要过判法，按台的三个键必写（crash-point-replay-shard-configuration-check.sh 文件头）：
#   127.0.0.1（这一台）那一项是这一台的 cargo 目录（假工具链所在的目录）、<自证目录>/this-host-build-root 与 slice 此刻的总上限，192.0.2.61（「第二台」）那一项是
#   场景的 peer-bin、<自证目录>/peer-homes/<场景名> 与 1G（--memory-cap 的顶）。「第二台」其实在本机跑：它那一层内存包装照这一台网卡上的地址认出自己是 127.0.0.1、
#   取 127.0.0.1 那一项定 slice 的总上限、与 slice 此刻的 MemoryMax 不同就改（run-with-memory-cap.sh 文件头「总上限」②），所以照 slice 此刻的 MemoryMax 写那个数，包装不改它。
#   「第二台」那一趟的环境只给最少几样，包装用默认的 slice：读不到、不是 MiB 的整数倍的，自证在建场景之前就停（改不改 slice 判不了）
#   场景配置 场景的多机配置过得了判法（--emit-assignments 退 0），SLICE_MEMORY_CAP_BY_HOST 里这一台那一项照 slice 此刻的总上限写：这一格红了就停，后面经 --peer 的格同因全红
#     （「双机分片的本地配置判不过」）
#   Ⓖ 输出（驱动的标准输出与标准错误）里没有配置里的主机名与第二台的目录，而命令打的那两样被换成「第二台」「第二台的目录」
#   Ⓗ 配置里 ENABLE_ACROSS_MACHINES=0 ⇒ --peer 拒（退 255，说在配置里写 ENABLE_ACROSS_MACHINES=1），命令哪台都没跑
#   Ⓘ 弄坏开关设在自证之外（起驱动时去掉自证的标记，带 --peer）⇒ 拒（退 2），命令没跑；只供测试的 SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 设在自证之外同样拒（退 2）
#   Ⓚ --peer 跑 bash -c 'exit 3'，第二台的 PATH 最前面（场景的 peer-bin）放假 journalctl（打一行 NVRM … NV_ERR_NO_MEMORY），第二台那一层内存包装照 PATH 取到它 ⇒ 包装退 245，
#      驱动原样退 245、标准错误里报「显存不够」与「不算结果」
#   Ⓜ --peer 命令里 env CARGO_TARGET_DIR=../shared-target、bash -c 脚本里 export CARGO_TARGET_DIR=/var/tmp/shared-target ⇒ 都是用法错（退 2），命令没跑
#   Ⓠ 编译目录：--peer 时第二台那条命令拿到的 CARGO_TARGET_DIR 是这一趟的本地目录下的 target、
#      MULTI_HOST_RUN_PEER_SCRATCH 是这一趟的本地目录，小仓里没建 target，跑完那个目录不在
#   Ⓡ 删之前核开着的文件：命令记下这一趟的本地目录后等放行文件，自证在那里起一个 sleep 开着一份文件再放行 ⇒ 驱动不删、报出那个进程、退 255
#   Ⓣ 自证的细节行按字截（cut_characters）：200 个汉字截到 161 个字，截出来是合法的 UTF-8
#   Ⓞ RocksDB 预编库的取值（文件头 ⑤）：第二台的 PATH 前面放不带 entry 认的标记的假 cargo（-V 转给假工具链的 cargo，别的建一份「跑了」的文件），四个取值在本机当第二台的环境里都没有，
#      小仓的 scripts/experiment-apparatus/Cargo.toml 依赖 rocksdb：-- cargo build ⇒ 退 78、假 cargo 没起、接回驱动标准错误的输出里有 rocksdb-prebuilt entry 的红句；-- bash -c 'touch …' ⇒ 不判、退 0。
#   Ⓤ 两台用同一份多机配置（文件头 ⑤「带到第二台的环境」）：第二台那条命令记下的 SINGLEFS_MULTI_HOST_CONFIG 是场景那一份（同一个绝对路径、sha256 相同），
#      第二台专用目录下不留常驻的那一份
#   Ⓦ 两台对称：第二台的 PATH 最前面放假 ssh（被起一次记一行）；第二台那条命令里单跑判法、带 --emit-assignments 跑判法（带着只供测试的「第二台是本机」与它的标记，
#      命令里点到的两份标记照内容另写在这一场景的共享根下，
#      同这一侧），这一侧照同一份场景配置也跑这两趟 ⇒ 两边退出码相同、--emit-assignments 打的逐字相同；第二台那条命令里再起 multi-host-run.sh --peer
#      （带着自证的标记与调用方的弄坏开关）⇒ 退 255、说交给 task-server；假 ssh 一次没被起
#   Ⓥ 任务里面再带 --peer（文件头 ①）：起驱动时去掉服务给最外层那一次的标记 ⇒ 退 255、说交给 task-server，命令没跑、第二台没建这一趟的目录
#   Ⓧ 驱动正经 gpu-quiesce-run.sh 起（GPU_QUIESCE_HOLDER 指的登记文件名字以驱动的父进程号起头）⇒ 第二台拿到 GPU_QUIESCE_ORIGIN_READ_BACK=1；进程号对不上的登记文件 ⇒ 没设
#   Ⓩ 共享根（文件头 ③）：--peer 命令里有共享根外的绝对路径（<自证目录>/outside-shared 下，整个词、bash -c 词里各一趟）⇒ 退 255、点名那个路径，命令没跑、没建这一趟的目录；
#      在 outside-shared 里起 ⇒ 退 255、说工作目录；命令里只有共享根下的路径与 /dev/null ⇒ 照跑、退 0；
#      直接问 multi_host_outside_shared_path（共享根是一个空目录，每一例放在 bash -c 后面）：cp "${X:?}/release/a" /w/x 只取 /w/x，cd ./scripts && ls ../a/b、
#      ls $(pwd)/target 与 echo a:/opt/z 一个都不取，cp /opt/x y、OUT=/opt/y true、tool --out=/opt/v、cat </opt/u 照取；
#      --item 'crash-case-replay（接的是 /）' 与 bash -c 'echo "（接的是 /）" > /dev/null' 一个都不取，--item 'x（接的是 /）' /opt/outside-word 只取 /opt/outside-word
#   Ⓢ 共享根交给第二台：第二台那条命令看到的 SINGLEFS_WORK_ROOT、SINGLEFS_DATA_ROOT 是场景那两个的真路径；工作根、多机配置放到 outside-shared 下 ⇒ 各退 255、点名是哪一样；
#      配置 BUILD_ROOT_BY_HOST 里第二台那一项改到场景目录（共享根）下 ⇒ 退 255、说这一趟的本地目录不许落在 NFS 上，命令都没跑
#   Ⓦ 分段计时：--peer 跑成的那一趟里 peer-command、remove-build 两步都有开始与结束两行
#   双机驱动（照 crash-amplification-two-hosts.sh 的 two_hosts_peer_build_then_run_script：env 加 bash -c 包着先编后跑两步 cargo）经 --peer 真跑
#      （第二台的 cargo 是假的、记下它所在的目录与参数）⇒ 退 0、假 cargo 起了两次、都在本机小仓那棵树里起（同一个绝对路径）
#   接回（文件头 ⑤「跑着时接回」）：第二台那条命令打三行，每打一行空转一阵 CPU、再等那一行出现在驱动的标准错误文件里（至多 8 秒），记下等到了没有；
#      再打半行、空转、过 3 秒补完，最后一截不以换行收尾 ⇒ 三行都在跑着时等到了；驱动标准错误里 follow- 起头的行正好是那五行、同序不重；
#      MULTI_HOST_RUN_PEER_CPU 每一轮一行、poll= 从 1 起连号、usage_usec 不减、末行至少 1000000（空转了四阵、每阵至少 1 秒）、至少一轮 read=yes
#   此刻内存（文件头 ⑤「此刻内存」）：MULTI_HOST_RUN_PEER_MEMORY 每一轮一行、紧跟在那一轮的 CPU 行之后、poll= 与 CPU 那一行相同、整行对上固定格式、至少一轮 read=yes 且 current_bytes 大于 0；
#      再跑一趟空转 3 秒、每一轮读 memory.stat 的文件名换成不在的（MULTI_HOST_RUN_SELFTEST_CURRENT_FILE）、退出时读的也换成不在的 ⇒ 退 0、
#      MEMORY 行与 CPU 行一样多（至少一行）、每一行都是 current_bytes=0 peak_bytes=0 read=no
#   等 IO（文件头 ⑤「等 IO」）：接回那一趟每一轮一行 MULTI_HOST_RUN_PEER_IO_WAIT、在那一轮的 CPU 行之后、下一轮 CPU 行之前，poll= 与 CPU 那一行相同、整行对上固定格式、
#      至少一轮 read=yes；上面读不到 memory.stat 那一趟另把数的状态换成 S（MULTI_HOST_RUN_SELFTEST_BLOCKED_STATES=S：空转的命令底下总有在睡的父进程）⇒ 至少一轮 read=yes 且 blocked 大于 0
#   峰值（文件头 ⑤「峰值」）：接回那一趟结束时恰好一行 MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=<正整数>、在最后一行 CPU 之后；上面读不到 memory.stat 那一趟
#      （每一轮与退出时读的都换成不在的）⇒ 退 0、至少一轮 read=yes、不打那一行；再跑一趟 0.1 秒就退的、只把每一轮读的换成不在的 ⇒ 照那一层退出时读的打恰好一行
#      页缓存撑满上限（文件头 ⑤「峰值」）：每一轮读的目录换成假 cgroup（MULTI_HOST_RUN_SELFTEST_MEMORY_DIRECTORY：memory.stat 里 anon 1G、file 40G、kernel 650M、shmem 0，
#      memory.current 与 memory.peak 是三项之和），空转 3 秒 ⇒ 退 0、read=yes 的每一行 MEMORY 都是 current_bytes 与 peak_bytes 等于 1G、至少一行，峰值行是 bytes=1G
#      短命令（文件头 ⑤「峰值」）：每一轮读的目录换成假 cgroup（anon 18 MiB）、命令真用 128 MiB ⇒ 峰值行不小于 100 MiB；同一个假 cgroup、退出时读的文件名换成不在的 ⇒ 不打峰值行
#   内存明细（文件头 ⑤「内存最高那一刻的明细」）：页缓存那一趟每一轮一行 MULTI_HOST_RUN_PEER_MEMORY_STAT、读数是假 cgroup 写的；接回那一趟至少一轮 read=yes
#   显存（文件头 ⑤「显存」）：不要卡的一趟每一轮 used_mib=0 read=no；PATH 上放假 nvidia-smi、环境里有 GPU_CARDS_BY_PRIORITY 的一趟至少一轮 used_mib=2048 read=yes（1 号进程的 9999 不算）
#   实际在哪张卡（文件头 ⑤「实际在哪几张卡」）：同一个假 nvidia-smi，要卡的一趟与不要卡的一趟都至少一轮 on_cards=2:<这一趟的进程号>:2048（1 号进程在卡 0 上不算）
#   编译目录占盘（文件头 ⑤「编译目录占盘」）：Ⓠ --peer 那一趟往编译目录里写了产物 ⇒ 恰好一行 MULTI_HOST_RUN_PEER_BUILD_PEAK bytes=<正整数>；
#      命令自己删了编译目录 ⇒ 退 0、不打那一行
#   收拾（文件头 ⑥ 与 --recover-cleanup 那一段）：标签都带这一份自证自己的进程号（并行跑的几份自证各用各的标签）。
#      先按 scope 收：--peer 命令 setsid 出去一个清了环境的 sleep、记下它的进程号，再只给那一层内存包装（RUN_WITH_MEMORY_CAP_OUTER_UNIT 的单元名里带它的进程号）发 KILL
#        ⇒ 驱动退 137、sleep 不在了、⑥ 那一行写停 scope 时里面还有至少 1 个、第二台 runs/ 与 builds/ 下什么都不剩
#      临时目录（文件头 ⑤「这一条命令自己的临时目录」、⑥「那一层留下的临时目录」）：上面那一趟的命令先记下它拿到的 TMPDIR ⇒ 在 MULTI_HOST_PEER_DEFAULT_TEMPORARY_ROOT 下直接一层、
#        以 MULTI_HOST_PEER_TEMPORARY_PREFIX 起名、全路径不超过 MULTI_HOST_PEER_TEMPORARY_PATH_LIMIT 字节；⑥ 之后那一份不在了、⑥ 打了「删了第二台那一层没收拾的临时目录」那一行。
#        另一趟命令把这一趟的本地目录里那份记录改写成场景里的一个目录，再只 KILL 那一层内存包装 ⇒ 那个目录还在、⑥ 打了「没删第二台那一层记下的临时目录」那一行、
#        第二台 runs/ 与 builds/ 下什么都不剩（那一层建的那一份由这一格照命令记下的路径删）
#      setsid 出去的后代：--peer 命令带着这一趟的标签（env TASK_SERVER_ATTEMPT_LABEL=<标签>）经 systemd-run --user --scope 起一个跑出那一层 scope、setsid 出去的 bash（记下自己的进程号、exec sleep 300），
#        等它记下就退 0 ⇒ 驱动退 0、那一个不在了、⑥ 那一行写「删之前停了这一趟留下的 1 个进程」、第二台 runs/ 与 builds/ 下什么都不剩
#      --recover-cleanup 两格：驱动环境里给清理回执的路径（TASK_SERVER_PEER_CLEANUP_FILE），命令记下自己的进程号、等放行文件；自证只给自己起的驱动发 KILL
#        （它不收尾、清理回执留在 pending），再放行：命令 KILL 起它的那个 bash（进程号文件里那个号，它没写退出回执）。
#        后代都已不在：命令接着退 0；等这一趟的会话里一个进程都不剩，起一次 --recover-cleanup ⇒ 退 0、stderr 里「补写了退出回执「143 stopped already-gone」」、
#          清理回执是 confirmed、第二台 runs/ 与 builds/ 下什么都不剩
#        后代还活着：命令接着 exec sleep 300；等起的那个 bash 不在了，起一次 --recover-cleanup ⇒ 退 0、停了至少 1 个（PEER_HOST_LEFTOVERS stopped=）、
#          补写的退出回执是 143 stopped TERM 或 137 stopped KILL、命令不在了、清理回执是 confirmed、什么都不剩
# 自证在临时目录里放标记文件、export MULTI_HOST_RUN_SELFTEST_MARK 指向它：场景里起的驱动认得出是自证起的，调用方设的弄坏开关才生效

multi_host_selftest() {
  local repository_root work template sop_source checked=0 failures=0
  repository_root="$(cd "$multi_host_script_directory/../.." && pwd)"
  work="$(mktemp -d)"
  multi_host_selftest_work="$work"
  printf '%s\n' "$MULTI_HOST_SELFTEST_MARK_CONTENT" > "$work/selftest-mark"
  export MULTI_HOST_RUN_SELFTEST_MARK="$work/selftest-mark"
  # peer-host-lib.sh 只在它的标记在时认只供测试的「第二台是本机」（那份文件头「只供测试的开关」）：场景里起的驱动照环境继承这一份
  printf '%s\n' "$PEER_HOST_SELFTEST_MARK_CONTENT" > "$work/peer-host-selftest-mark"
  export PEER_HOST_SELFTEST_MARK="$work/peer-host-selftest-mark"
  # 内存包装只在它自证的标记在时认 RUN_WITH_MEMORY_CAP_BREAK（run-with-memory-cap.sh 文件头「弄坏开关只在自证里认」）：调用方设的开关原样带给场景里的包装，
  # 这里照包装文件头建同一种标记、export；随自证的临时目录删
  printf '%s\n' "run-with-memory-cap.sh --selftest 起的包装" > "$work/memory-cap-selftest-mark"
  export RUN_WITH_MEMORY_CAP_SELFTEST_MARK="$work/memory-cap-selftest-mark"
  trap 'echo "  · 自证的临时目录（$(du -sh "$multi_host_selftest_work" 2>/dev/null | cut -f1)）删掉"; rm -rf -- "${multi_host_selftest_work:?}"' EXIT
  expect() { # expect <格名> <判据的退出码：0 为过> <不过时的细节>
    checked=$((checked + 1))
    if [[ "$2" == 0 ]]; then
      echo "  ✓ $1"
    else
      echo "  ✗ $1：$3"  # gate-lint:detail
      failures=$((failures + 1))
    fi
  }
  if ! sop_source="$(peer_host_sop_source "$repository_root" "$repository_root")"; then
    echo "  ✗ multi-host-run.sh 自证搭不起小仓：本仓与它的主仓里都没有规范副本 .claude/singlefs-ai-sop/"
    echo "     → 怎么办：在主仓里装好它（bash .claude/singlefs-ai-sop/install.sh）再跑"
    return 1
  fi
  local outside_shared="$work/outside-shared"   # 不在任何场景的共享根下（Ⓩ、Ⓢ）
  mkdir -p "$outside_shared" "$work/peer-homes" "$work/home"

  # 小仓的模板
  template="$work/template"
  mkdir -p "$template/.claude/hooks" "$template/research"
  rsync -a --exclude __pycache__ "$repository_root/.claude/scripts/" "$template/.claude/scripts/"
  cp -- "$repository_root/.claude/hooks/lib_work_root.py" "$template/.claude/hooks/lib_work_root.py"   # ③ 取工作根要它
  rsync -a --exclude __pycache__ --exclude /experiment-apparatus/target/ "$repository_root/scripts/" "$template/scripts/"   # 装置的编译目录几十 G，自证用不到
  ln -s "$sop_source" "$template/.claude/singlefs-ai-sop"
  printf '/.claude/singlefs-ai-sop\n__pycache__/\n/multi-host.env\ntarget/\n' > "$template/.gitignore"
  selftest_commit() { # <仓> <git commit 的参数…>：自证里的提交（身份、钩子、签名都不读调用方的配置）
    local repository="$1"
    shift
    git -C "$repository" -c user.name=multi-host-run-selftest -c user.email=multi-host-run-selftest@localhost.invalid -c core.hooksPath=/dev/null -c commit.gpgSign=false \
      commit -q --no-verify "$@"
  }
  # 小仓的模板提交一次、整理成包：场景照它拷、带着这一个提交（Ⓑ 核第二台读到的 HEAD 与本机相同）
  if ! { git -c init.defaultBranch=master -C "$template" init -q && git -C "$template" add -A && selftest_commit "$template" -m 'multi-host-run 自证的小仓' \
           && git -C "$template" repack -adq; }; then
    echo "  ✗ multi-host-run.sh 自证搭不起小仓：模板 git init、提交或整理成包失败"
    echo "     → 怎么办：看 git 在不在、自证的临时目录可不可写、盘满没满"
    return 1
  fi
  # 假 rustup（peer-host-lib.sh 文件头「工具链比对」）：场景里起的驱动 PATH 最前面放它，本机与第二台的 rustup component list --installed 都打同目录那一份组件表，
  #   登记过的自证不起真 rustup（真的那一份在 cargo 的 bin 目录里，scripts/gate/selftest-process-guard.py 认它）
  local fake_rustup_directory="$work/fake-rustup"
  mkdir -p "$fake_rustup_directory"
  printf '%s\n' cargo-x86_64-unknown-linux-gnu clippy-x86_64-unknown-linux-gnu rust-std-x86_64-unknown-linux-gnu rustc-x86_64-unknown-linux-gnu rustfmt-x86_64-unknown-linux-gnu \
    > "$fake_rustup_directory/installed-components"
  cat > "$fake_rustup_directory/rustup" <<'FAKE_RUSTUP'
#!/usr/bin/env bash
# multi-host-run.sh 自证的假 rustup：component list --installed 打同目录 installed-components 那一份，别的用法退 1
if [[ "$*" == "component list --installed" ]]; then cat "$(dirname "$0")/installed-components"; exit 0; fi
exit 1
FAKE_RUSTUP
  chmod +x "$fake_rustup_directory/rustup"
  # 假 rustc、cargo（自证段说明里「假 rustup、rustc、cargo」那一句）：toolchain_rustc、toolchain_cargo 是场景里各份假 cargo 的 -V 与自证自己问 host 那一步用的那一份；
  #   弄坏开关 selftest-caller-toolchain 下不放，两个都取 PATH 上调用方的那一份
  local toolchain_rustc="$fake_rustup_directory/rustc" toolchain_cargo="$fake_rustup_directory/cargo"
  if multi_host_break_is selftest-caller-toolchain; then
    toolchain_rustc="$(command -v rustc)"
    toolchain_cargo="$(command -v cargo)"
  else
    cat > "$toolchain_rustc" <<'FAKE_RUSTC'
#!/usr/bin/env bash
# multi-host-run.sh 自证的假 rustc：-Vv / -vV 打写死的版本（两台照比），别的用法退 1
case "$*" in
  -Vv | -vV) printf '%s\n' 'rustc 0.0.0-multi-host-run-selftest' 'binary: rustc' 'commit-hash: multi-host-run-selftest' 'commit-date: multi-host-run-selftest' \
               'host: x86_64-unknown-linux-gnu' 'release: 0.0.0-multi-host-run-selftest'; exit 0 ;;
esac
exit 1
FAKE_RUSTC
    cat > "$toolchain_cargo" <<'FAKE_CARGO'
#!/usr/bin/env bash
# multi-host-run.sh 自证的假 cargo：-V / --version 打写死的版本（两台照比），别的用法退 1
case "$*" in
  -V | --version) echo 'cargo 0.0.0-multi-host-run-selftest'; exit 0 ;;
esac
exit 1
FAKE_CARGO
    chmod +x "$toolchain_rustc" "$toolchain_cargo"
  fi

  # 场景配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项（文件头「场景的多机配置要过判法」那几句）：「第二台」那一趟的包装读的默认 slice 的 MemoryMax 换成 MiB
  local scene_slice_memory_cap="" slice_cgroup slice_max=""
  slice_cgroup="$(systemctl --user show -p ControlGroup --value singlefs-heavy.slice 2>/dev/null)"
  [[ -n "$slice_cgroup" ]] && slice_max="$(cat "/sys/fs/cgroup$slice_cgroup/memory.max" 2>/dev/null)"
  if ! [[ "$slice_max" =~ ^[1-9][0-9]*$ ]] || (( slice_max % 1048576 != 0 )); then
    echo "  ✗ multi-host-run.sh 自证搭不起场景配置：判法要场景的多机配置写 SLICE_MEMORY_CAP_BY_HOST，这一台那一项要与内存包装用的 slice 此刻的总上限相同（不然包装会改 slice），读到的是 singlefs-heavy.slice 的 memory.max「${slice_max:-读不到}」"
    echo "     → 怎么办：slice 还没起或没有总上限的，先经 task-server 跑一条命令（包装照仓根多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项设上）"
    return 1
  fi
  scene_slice_memory_cap="$((slice_max / 1048576))M"
  local scene_this_cargo_directory scene_this_build_root="$work/this-host-build-root"
  scene_this_cargo_directory="$(dirname "$toolchain_cargo")"

  prepare_scene() { # <场景名> [配置里 ENABLE_ACROSS_MACHINES 那一行，默认 ENABLE_ACROSS_MACHINES=1]：repo（模板那一份，带着它那一个提交）、peer-bin、配置；
    # 「第二台」的专用目录 <自证目录>/peer-homes/<场景名>（不在这一场景的共享根下）
    local scene="$work/$1" peer_home="$work/peer-homes/$1"
    mkdir -p "$scene/peer-bin" "$peer_home"
    rsync -a "$template/" "$scene/repo/"
    {
      echo "HOSTS=127.0.0.1 192.0.2.61"   # 主机数组：127.0.0.1 在每一台的网卡上，是自己；192.0.2.61（文档用的地址段）是「第二台」
      # 按台的三个键（文件头「场景的多机配置要过判法」那几句）：192.0.2.61 那一项是「第二台」的，127.0.0.1 那一项是这一台的
      echo "BUILD_ROOT_BY_HOST=127.0.0.1=$scene_this_build_root 192.0.2.61=$peer_home"
      echo "CARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=$scene_this_cargo_directory 192.0.2.61=$scene/peer-bin"
      # 弄坏开关 scene-slice-cap-unwritten 下不写（判法判不过：「场景配置」那一格红、自证停在那里）
      multi_host_break_is scene-slice-cap-unwritten || echo "SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=$scene_slice_memory_cap 192.0.2.61=1G"
      echo "${2-ENABLE_ACROSS_MACHINES=1}"
      echo "ENABLE_GPU=0"   # 两个开关都要明确写（配置判法不兜底）
      echo "NFS_SHARED_ROOTS=$scene"   # 整个场景目录是这一场景的共享根（配置判法要它是本机已在的目录：上面 mkdir 过）
      echo "SHARED_DATA_ROOT=$scene/data-root"   # 共享数据根在这一场景的共享根下，与起驱动时设的 SINGLEFS_DATA_ROOT 同一个
      echo "QUIESCE_STOP_COMMAND="
      echo "QUIESCE_STOPPED_CHECK_COMMAND="
      echo "QUIESCE_START_COMMAND="
      echo "QUIESCE_STARTED_CHECK_COMMAND="
    } > "$scene/multi-host.env"
  }
  scene_command_for() { # <场景名>：scene_command 设成在这个场景里起驱动的前缀（弄坏开关从调用方的环境里留着）；
    # 驱动的临时目录落在自证的临时目录下，弄坏开关留下的随自证删；末两个词是 bash <驱动>（要另加环境的格插在它们前面）
    local scene="$work/$1"
    mkdir -p "$work/scene-tmp"
    # 清场的握手变量与这一趟的本地目录不从调用方带进来（自证在第二台上跑时调用方的环境里有 GPU_QUIESCE_ORIGIN_READ_BACK、MULTI_HOST_RUN_PEER_SCRATCH）；要的格自己给。
    # 场景里的驱动都当服务直接起的那一次（文件头 ①）：倒数第三个词是服务给最外层那一次的标记（Ⓥ 去掉它起）；末两个词是 bash <驱动>。
    # 服务起的任务环境里带着服务给的 CARGO_TARGET_DIR：清掉，驱动这一侧的编译目录不混进场景（第二台那条命令拿到的是 ⑤ 给的那一个）。
    # HOME 指到自证自己的 home/：「第二台」那一侧经 env -i 只拿到 HOME，它的内存包装照 HOME 取默认峰值表，调用方的 HOME 不可写（服务起的第二台条件）时包装往
    #   标准错误打「峰值没记进」，混进接回的输出；CARGO_HOME、RUSTUP_HOME 摘掉：工具链只用 PATH 最前面那几个假的（自证段说明），不靠调用方的家目录
    scene_command=(env -u RUN_WITH_MEMORY_CAP_KEY -u GPU_QUIESCE_HOLDER -u GPU_QUIESCE_ORIGIN_READ_BACK
      -u MULTI_HOST_RUN_PEER_SCRATCH -u MULTI_HOST_RUN_ADMITTED_BY_TASK_SERVER -u CARGO_TARGET_DIR -u CARGO_HOME -u RUSTUP_HOME HOME="$work/home"
      SINGLEFS_MULTI_HOST_CONFIG="$scene/multi-host.env" SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 PATH="$fake_rustup_directory:$PATH"
      MULTI_HOST_RUN_POLL_SECONDS=1 RUN_WITH_MEMORY_CAP_PEAKS="$scene/peaks.tsv" TMPDIR="$work/scene-tmp"
      SINGLEFS_WORK_ROOT="$scene/work-root" SINGLEFS_DATA_ROOT="$scene/data-root"
      MULTI_HOST_RUN_ADMITTED_BY_TASK_SERVER=selftest-task.1 bash "$scene/repo/scripts/hosts/multi-host-run.sh")
  }
  run_in() { # <场景名> <这一趟的名字> <驱动的参数…>：在小仓的根里起驱动；stdout、stderr、退出码各写场景目录一个文件
    local scene="$work/$1" run_name="$2" exit_code
    shift 2
    scene_command_for "${scene##*/}"
    if ( cd "$scene/repo" && exec "${scene_command[@]}" "$@" ) > "$scene/$run_name.out" 2> "$scene/$run_name.err" < /dev/null; then exit_code=0; else exit_code=$?; fi
    echo "$exit_code" > "$scene/$run_name.exit"
  }
  exit_of() { cat "$work/$1/$2.exit" 2>/dev/null; }
  tail_of() { tail -n 3 "$work/$1/$2.err" 2>/dev/null | tr '\n' '|'; }
  repository_of() { (cd "$work/$1/repo" && pwd -P); }   # <场景名>：小仓的真路径（第二台那条命令在它里面起）
  cut_characters() { # <字数>：每行只留前这么多个字（按 UTF-8 的字截：GNU cut -c 按字节截，会把汉字截成半个，细节行里留下解不成 UTF-8 的字节）；
    # 弄坏开关 detail-cut-by-bytes 下按字节截（Ⓣ 那一格红）
    if multi_host_break_is detail-cut-by-bytes; then cut -c "1-$1"; return; fi
    python3 -c 'import sys; limit = int(sys.argv[1]); sys.stdout.buffer.write(b"".join((line.decode("utf-8", "replace").rstrip("\n")[:limit] + "\n").encode("utf-8") for line in sys.stdin.buffer))' "$1"
  }
  peer_runs_left() { # <场景名>：第二台 runs/ 与 builds/ 下留着的条目数
    find "$work/peer-homes/$1/runs" "$work/peer-homes/$1/builds" -mindepth 1 -maxdepth 1 2>/dev/null | wc -l
  }

  peer_driver_cells() { # 双机驱动那一格（自证文件头「双机驱动」）；整份自证在末尾调
    local scene driver_repository driver_calls
    prepare_scene peer-driver
    scene="$work/peer-driver"
    # 写法照 crash-amplification-two-hosts.sh 的 two_hosts_peer_build_then_run_script：env 加 bash -c 包着先编后跑两步 cargo；第二台在共享的那棵树里、照同一个绝对路径起。
    #   第二台的 cargo 是假的（带 rocksdb-prebuilt entry 认的标记、-V 转给假工具链的 cargo），记下两次起它时所在的目录与参数
    local build_then_run_script='count="$1"; shift; build=("${@:1:$count}"); shift "$count"; "${build[@]}" || exit "$?"; "$@"'
    printf '%s\n' '#!/usr/bin/env bash' '# rocksdb-prebuilt-entry: selftest-fake-cargo' '# 自证用的 cargo（双机驱动那一格）：-V / --version 转给假工具链的 cargo（两台工具链照比），别的把所在的目录与参数记一行、退 0' \
      "case \"\${1:-}\" in -V|--version) exec $(printf '%q' "$toolchain_cargo") \"\$@\" ;; esac" \
      "printf '%s %s\n' \"\$PWD\" \"\$*\" >> $(printf '%q' "$scene/cargo-calls")" > "$scene/peer-bin/cargo"
    chmod +x "$scene/peer-bin/cargo"
    run_in peer-driver driver-e2e --peer --memory-cap 256M --label driver-e2e -- \
      env SINGLEFS_CRASH_AMPLIFICATION_SHARE=1/2 bash -c "$build_then_run_script" peer-build-then-run 3 cargo test --no-run cargo test --test crash_amplification_sample
    driver_calls="$(cat "$scene/cargo-calls" 2>/dev/null | tr '\n' '|')"
    driver_repository="$(repository_of peer-driver)"
    expect "双机驱动那种起法经 --peer 真跑一趟（在共享的树里、照同一个绝对路径起）⇒ 退 0、第二台的假 cargo 先编后跑起了两次、都在本机那棵树里起" \
      "$([[ "$(exit_of peer-driver driver-e2e)" == 0 \
         && "$driver_calls" == "$driver_repository test --no-run|$driver_repository test --test crash_amplification_sample|" ]]; echo $?)" \
      "退 $(exit_of peer-driver driver-e2e)；假 cargo 收到「$driver_calls」（目录应当是 $driver_repository）；stderr 尾部：$(tail_of peer-driver driver-e2e | cut_characters 300)"
  }

  # 场景配置（文件头「场景的多机配置要过判法」那几句）：判法判不过时后面经 --peer 的格同因全红，这一格红了就停
  local configuration_scene="$work/scene-configuration" configuration_check_exit=0
  prepare_scene scene-configuration
  env SINGLEFS_MULTI_HOST_CONFIG="$configuration_scene/multi-host.env" SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 \
    bash "$configuration_scene/repo/scripts/crash/crash-point-replay-shard-configuration-check.sh" --emit-assignments "$configuration_scene/repo" \
    > "$configuration_scene/check.out" 2>&1 || configuration_check_exit=$?
  expect "场景配置 场景的多机配置过得了判法（--emit-assignments 退 0），SLICE_MEMORY_CAP_BY_HOST 里这一台那一项照 slice 此刻的总上限写" \
    "$([[ "$configuration_check_exit" == 0 ]] && grep -qxF "SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=$scene_slice_memory_cap 192.0.2.61=1G" "$configuration_scene/multi-host.env"; echo $?)" \
    "判法退 $configuration_check_exit；场景配置里$(grep -q '^SLICE_MEMORY_CAP_BY_HOST=' "$configuration_scene/multi-host.env" && echo "写的 $(grep '^SLICE_MEMORY_CAP_BY_HOST=' "$configuration_scene/multi-host.env")" || echo 没写 SLICE_MEMORY_CAP_BY_HOST)（slice 此刻 $scene_slice_memory_cap）；判法的输出：$(grep '✗' "$configuration_scene/check.out" | tr '\n' '|' | cut_characters 300)"  # gate-lint:detail
  if (( failures > 0 )); then
    echo "  ✗ multi-host-run.sh 自证停在场景配置那一格（共 $checked 格）：判法判不过场景的多机配置，后面经 --peer 的格同因全红，不再跑"  # gate-lint:summary
    echo "     → 怎么办：设了 MULTI_HOST_RUN_BREAK 的，这就是要的结果；没设的，单跑 bash scripts/crash/crash-point-replay-shard-configuration-check.sh --selftest 看判法要哪些键，照它改 prepare_scene 写的场景配置"
    return 1
  fi

  # peer-host-lib.sh 的只供测试的开关只在自证的标记在时认（那份文件头「只供测试的开关」）：照场景配置读两遍，一遍去掉标记、一遍带着。
  #   判法认这个开关也要同一份标记（crash-point-replay-shard-configuration-check.sh 文件头「只供测试的开关」）：去掉标记那一遍判法照连第二台，
  #   PATH 最前面放一份连得上的假 ssh（被起一次记一行、退 0），判法过得了，只看 peer-host-lib.sh 自己认不认
  local lib_probe='source "$1"; peer_host_load_configuration "$2" "$3" >/dev/null 2>&1; echo "$?:$peer_host_is_this_machine"' lib_unmarked lib_marked
  local lib_probe_bin="$configuration_scene/lib-probe-bin"
  mkdir -p "$lib_probe_bin"
  printf '%s\n' '#!/usr/bin/env bash' '# multi-host-run.sh 自证 peer-host-lib.sh 那一格的假 ssh：被起一次记一行，照连得上退 0' \
    "printf '%s\n' \"\$*\" >> $(printf '%q' "$configuration_scene/lib-probe-ssh.log")" 'exit 0' > "$lib_probe_bin/ssh"
  chmod +x "$lib_probe_bin/ssh"
  lib_unmarked="$(env -u PEER_HOST_SELFTEST_MARK PATH="$lib_probe_bin:$PATH" SINGLEFS_MULTI_HOST_CONFIG="$configuration_scene/multi-host.env" \
    SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 bash -c "$lib_probe" lib-probe "$configuration_scene/repo/scripts/hosts/peer-host-lib.sh" \
    "$configuration_scene/repo/scripts/crash/crash-point-replay-shard-configuration-check.sh" "$configuration_scene/repo" 2>/dev/null)"
  lib_marked="$(env PATH="$lib_probe_bin:$PATH" SINGLEFS_MULTI_HOST_CONFIG="$configuration_scene/multi-host.env" \
    SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 bash -c "$lib_probe" lib-probe "$configuration_scene/repo/scripts/hosts/peer-host-lib.sh" \
    "$configuration_scene/repo/scripts/crash/crash-point-replay-shard-configuration-check.sh" "$configuration_scene/repo" 2>/dev/null)"
  expect "peer-host-lib.sh：只供测试的 SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 没有自证的标记 ⇒ 不认（peer_host_is_this_machine 是 0）；有标记 ⇒ 认（1）" \
    "$([[ "$lib_unmarked" == 0:0 && "$lib_marked" == 0:1 ]]; echo $?)" \
    "没标记读到 ${lib_unmarked:-空}、有标记读到 ${lib_marked:-空}（peer_host_load_configuration 的退出码:peer_host_is_this_machine）；两遍里判法共起假 ssh $(cat "$configuration_scene/lib-probe-ssh.log" 2>/dev/null | wc -l) 次"

  # peer-host-lib.sh 的按台的三个键（那份文件头「按台的三个键」）：直接调 peer_host_load_by_host，不经判法（判法过了才走到它，这里只核它按地址取那一项）。
  #   一份这一台那一项写在头一项、第二台的编译根里带 = 的配置；一份 CARGO_BIN_DIRECTORY_BY_HOST 只写了这一台那一项的
  local by_host_probe='source "$1"; CRASH_POINT_REPLAY_SHARD_CONFIGURATION_FILE="$2"; THIS_HOST_ADDRESS=127.0.0.1; PEER_HOST_ADDRESS=192.0.2.61
if peer_host_load_by_host > "$3" 2>&1; then echo "0 cargo=$PEER_HOST_CARGO_BIN_DIRECTORY root=$PEER_HOST_BUILD_ROOT cap=$PEER_HOST_SLICE_CAP this-cargo=$THIS_HOST_CARGO_BIN_DIRECTORY this-root=$THIS_HOST_BUILD_ROOT"; else echo "$?"; fi'
  local by_host_complete by_host_missing
  printf '%s\n' "CARGO_BIN_DIRECTORY_BY_HOST='127.0.0.1=/selftest/this-cargo 192.0.2.61=/selftest/peer-cargo'" \
    'BUILD_ROOT_BY_HOST=127.0.0.1=/selftest/this-root 192.0.2.61=/selftest/peer-root=with-equals' 'SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=3G 192.0.2.61=1G' \
    > "$configuration_scene/by-host-complete.env"
  printf '%s\n' 'CARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=/selftest/this-cargo' 'BUILD_ROOT_BY_HOST=127.0.0.1=/selftest/this-root 192.0.2.61=/selftest/peer-root' \
    'SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=3G 192.0.2.61=1G' > "$configuration_scene/by-host-missing.env"
  by_host_complete="$(bash -c "$by_host_probe" by-host-probe "$configuration_scene/repo/scripts/hosts/peer-host-lib.sh" "$configuration_scene/by-host-complete.env" \
    "$configuration_scene/by-host-complete.out")"
  by_host_missing="$(bash -c "$by_host_probe" by-host-probe "$configuration_scene/repo/scripts/hosts/peer-host-lib.sh" "$configuration_scene/by-host-missing.env" \
    "$configuration_scene/by-host-missing.out")"
  expect "peer-host-lib.sh：按台的三个键各台取各台那一项（头一项不是另一台也照地址取）；另一台那一项没写 ⇒ 载不了、退 1，打的话不带配置里的值" \
    "$([[ "$by_host_complete" == "0 cargo=/selftest/peer-cargo root=/selftest/peer-root=with-equals cap=1G this-cargo=/selftest/this-cargo this-root=/selftest/this-root" \
         && "$by_host_missing" == 1 ]] && grep -q 'CARGO_BIN_DIRECTORY_BY_HOST 里没有第二台那一项' "$configuration_scene/by-host-missing.out" \
         && ! grep -qE '/selftest/|127[.]0[.]0[.]1|192[.]0[.]2[.]61' "$configuration_scene/by-host-missing.out"; echo $?)" \
    "另一台取到的：${by_host_complete:-空}；少一项那一份退 ${by_host_missing:-空}、打的是：$(tr '\n' '|' < "$configuration_scene/by-host-missing.out" 2>/dev/null | cut_characters 300)"

  # Ⓐ 不带 --peer（文件头「不带 --peer」）：用法错，命令哪台都没跑
  local scene
  prepare_scene no-peer
  scene="$work/no-peer"
  run_in no-peer run --memory-cap 256M -- bash -c "touch $(printf '%q' "$scene/ran")"
  expect "Ⓐ 不带 --peer ⇒ 用法错（退 2，说本机的任务经 task-server 交），命令没跑、第二台一个目录都没建" \
    "$([[ "$(exit_of no-peer run)" == 2 && ! -e "$scene/ran" && "$(peer_runs_left no-peer)" == 0 ]] && grep -q '不带 --peer：这里只把命令挪到第二台跑，本机的任务经 task-server 交' "$scene/run.err"; echo $?)" \
    "退 $(exit_of no-peer run)，命令$([[ -e "$scene/ran" ]] && echo 跑了 || echo 没跑)；stderr 尾部：$(tail_of no-peer run)"

  # Ⓑ --peer 跑成、退出码带回、上限不许比 SLICE_MEMORY_CAP_BY_HOST 里第二台那一项大
  prepare_scene peer
  scene="$work/peer"
  # 记 cgroup 那一句写进场景目录（共享根下）的小脚本：--peer 的命令里不出现 /proc 这类共享根外的绝对路径（文件头 ③；两台相同的系统路径只认
  #   MULTI_HOST_HOST_INDEPENDENT_PATHS 那几样，task-server 的 HOST_INDEPENDENT_PATHS 同一组）
  printf '%s\n' 'cat /proc/self/cgroup > "$1"' > "$scene/record-cgroup.sh"
  run_in peer run --peer --memory-cap 256M --label peer-ok -- bash -c "pwd -P > $(printf '%q' "$scene/where"); bash $(printf '%q' "$scene/record-cgroup.sh") $(printf '%q' "$scene/cgroup"); \
git rev-parse -q --verify HEAD > $(printf '%q' "$scene/head"); printf '%s\n' \"\$MULTI_HOST_RUN_PEER_SCRATCH\"; echo 192.0.2.61"
  local peer_local_head peer_repository
  peer_local_head="$(git -C "$scene/repo" rev-parse -q --verify HEAD 2>/dev/null)"
  peer_repository="$(repository_of peer)"
  expect "Ⓑ --peer 跑成：退 0，在第二台同一个绝对路径（这一刻的工作目录，共享的那棵树）里跑，HEAD 与本机相同" \
    "$([[ "$(exit_of peer run)" == 0 && "$(cat "$scene/where" 2>/dev/null)" == "$peer_repository" && -n "$peer_local_head" && "$(cat "$scene/head" 2>/dev/null)" == "$peer_local_head" ]]; echo $?)" \
    "退 $(exit_of peer run)，在「$(cat "$scene/where" 2>/dev/null)」跑（应当是 $peer_repository），HEAD $(cat "$scene/head" 2>/dev/null || echo 没有)（本机 ${peer_local_head:-没有}）；stderr 尾部：$(tail_of peer run)"
  expect "Ⓑ --peer 经仓里的 run-with-memory-cap.sh 跑（cgroup 在 singlefs-memory-cap- 的 scope 里）" \
    "$(grep -q 'singlefs-memory-cap-' "$scene/cgroup" 2>/dev/null; echo $?)" \
    "命令记下的 cgroup：$(tr '\n' ' ' < "$scene/cgroup" 2>/dev/null)"
  run_in peer seven --peer --memory-cap 256M --label peer-seven -- bash -c 'exit 7'
  expect "Ⓑ --peer 退出码原样带回：第二台那条命令退 7 ⇒ 驱动退 7" \
    "$([[ "$(exit_of peer seven)" == 7 ]]; echo $?)" "退 $(exit_of peer seven)；stderr 尾部：$(tail_of peer seven)"
  run_in peer too-big --peer --memory-cap 2G --label peer-too-big -- bash -c "touch $(printf '%q' "$scene/too-big-ran")"
  expect "Ⓑ --memory-cap 2G 比多机配置 SLICE_MEMORY_CAP_BY_HOST 里第二台那一项 1G 大 ⇒ 拒（退 2），命令没跑" \
    "$([[ "$(exit_of peer too-big)" == 2 && ! -e "$scene/too-big-ran" ]] && grep -q 'SLICE_MEMORY_CAP_BY_HOST 里第二台那一项 1G 大' "$scene/too-big.err"; echo $?)" \
    "退 $(exit_of peer too-big)，命令$([[ -e "$scene/too-big-ran" ]] && echo 跑了 || echo 没跑)；stderr 尾部：$(tail_of peer too-big)"

  # 标签（文件头 ⑤ 标签撞了与查标签时连不上那一句）：已经有同名目录的是用法错；查标签时 ssh 自己退 255 的是第二台没跑成
  prepare_scene label
  scene="$work/label"
  local label_home="$work/peer-homes/label"
  mkdir -p "$label_home/builds/label-taken"
  printf '别的调用的本地目录\n' > "$label_home/builds/label-taken/kept.txt"
  run_in label taken --peer --memory-cap 256M --label label-taken -- bash -c "touch $(printf '%q' "$scene/taken-ran")"
  expect "标签 --peer 的 --label 在第二台 builds/ 下已经有同名目录 ⇒ 用法错（退 2），命令没跑、那个目录还在" \
    "$([[ "$(exit_of label taken)" == 2 && ! -e "$scene/taken-ran" && -f "$label_home/builds/label-taken/kept.txt" ]] && grep -q 'builds/ 下已经有 label-taken' "$scene/taken.err"; echo $?)" \
    "退 $(exit_of label taken)，命令$([[ -e "$scene/taken-ran" ]] && echo 跑了 || echo 没跑)，那个目录$([[ -f "$label_home/builds/label-taken/kept.txt" ]] && echo 还在 || echo 没了)；stderr 尾部：$(tail_of label taken | cut_characters 300)"
  local real_bash
  real_bash="$(command -v bash)"
  {
    printf '#!%s\n' "$real_bash"
    echo '# multi-host-run.sh 自证「标签」那一格的假 bash：查标签那一句点名 label-unreachable 的退 255（当作 ssh 自己连不上），别的原样转给真 bash'
    echo 'if [[ "${1:-}" == -c && "${2:-}" == "test ! -e "* && "${2:-}" == *"/builds/label-unreachable"* ]]; then exit 255; fi'
    printf 'exec %q "$@"\n' "$real_bash"
  } > "$scene/peer-bin/bash"
  chmod +x "$scene/peer-bin/bash"
  run_in label unreachable --peer --memory-cap 256M --label label-unreachable -- bash -c "touch $(printf '%q' "$scene/unreachable-ran")"
  expect "标签 --peer 查标签时第二台连不上（ssh 自己退 255）⇒ 退 255、说连不上，不当用法错，命令没跑、builds/ 下没建这一趟的目录" \
    "$([[ "$(exit_of label unreachable)" == 255 && ! -e "$scene/unreachable-ran" && ! -e "$label_home/builds/label-unreachable" ]] \
       && grep -q '第二台连不上（ssh 退 255）' "$scene/unreachable.err" && ! grep -q '用法错' "$scene/unreachable.err"; echo $?)" \
    "退 $(exit_of label unreachable)，命令$([[ -e "$scene/unreachable-ran" ]] && echo 跑了 || echo 没跑)，builds/ 下$([[ -e "$label_home/builds/label-unreachable" ]] && echo 建了 || echo 没建)这一趟的目录；stderr 尾部：$(tail_of label unreachable | cut_characters 300)"

  # Ⓒ 工具链不同
  prepare_scene toolchain
  scene="$work/toolchain"
  printf '#!/usr/bin/env bash\nprintf "rustc 0.0.0-other\\nbinary: rustc\\ncommit-hash: 0000\\nhost: other-host\\n"\n' > "$scene/peer-bin/rustc"
  chmod +x "$scene/peer-bin/rustc"
  run_in toolchain run --peer --memory-cap 256M -- bash -c "touch $(printf '%q' "$scene/ran")"
  expect "Ⓒ 第二台的 rustc -Vv 不同 ⇒ 拒（退 255，说两台的工具链不同），命令没跑、第二台没建这一趟的目录" \
    "$([[ "$(exit_of toolchain run)" == 255 && ! -e "$scene/ran" && "$(peer_runs_left toolchain)" == 0 ]] && grep -q '两台的工具链不同' "$scene/run.err"; echo $?)" \
    "退 $(exit_of toolchain run)，命令$([[ -e "$scene/ran" ]] && echo 跑了 || echo 没跑)；stderr 尾部：$(tail_of toolchain run)"

  # Ⓒ 第二台缺组件：第二台的 PATH 最前面放假 rustup，component list --installed 打本机那一份去掉 clippy 那一行（rustfmt 那一行与本机逐字相同）
  prepare_scene components
  scene="$work/components"
  grep -v '^clippy' "$fake_rustup_directory/installed-components" > "$scene/peer-components"
  cat > "$scene/peer-bin/rustup" <<'FAKE_RUSTUP'
#!/usr/bin/env bash
# 自证用的 rustup（Ⓒ 第二台缺组件）：component list --installed 打场景目录里那一份（本机的去掉 clippy 那一行），别的用法退 1
scene_directory="$(cd "$(dirname "$0")/.." && pwd)"
if [[ "$*" == "component list --installed" ]]; then cat "$scene_directory/peer-components"; exit 0; fi
exit 1
FAKE_RUSTUP
  chmod +x "$scene/peer-bin/rustup"
  run_in components run --peer --memory-cap 256M --label components-missing -- bash -c "touch $(printf '%q' "$scene/ran")"
  expect "Ⓒ 第二台缺 clippy 组件（rustup component list --installed 里没有那一行）⇒ 拒（退 255，说两台的工具链不同、第二台缺组件 clippy），命令没跑、第二台没建这一趟的目录" \
    "$([[ "$(exit_of components run)" == 255 && ! -e "$scene/ran" && "$(peer_runs_left components)" == 0 ]] && grep -q '两台的工具链不同：第二台缺组件 clippy' "$scene/run.err"; echo $?)" \
    "退 $(exit_of components run)，命令$([[ -e "$scene/ran" ]] && echo 跑了 || echo 没跑)；stderr 尾部：$(tail_of components run)"

  # Ⓒ 离线依赖：小仓树根放一份 Cargo.toml，第二台的 PATH 最前面放假 cargo。有 metadata-missing 那一趟照 cargo 的原话报缺 vstd、退 101；去掉它再跑一趟照跑
  prepare_scene offline
  scene="$work/offline"
  printf '[workspace]\nmembers = []\n' > "$scene/repo/Cargo.toml"
  printf '%s\n' "$toolchain_cargo" > "$scene/toolchain-cargo"
  cat > "$scene/peer-bin/cargo" <<'FAKE_CARGO'
#!/usr/bin/env bash
# 自证用的 cargo（Ⓒ 离线依赖）：metadata 记下目录与参数，场景目录里有 metadata-missing 就照 cargo 的原话报缺 vstd、退 101；
# 有 metadata-path-missing 就照 cargo 的原话（path 依赖的目录被工作区 exclude、又不在的那种）报取不到 path 依赖 served、退 101；别的用法转给假工具链的 cargo（-V 照比工具链）
scene_directory="$(cd "$(dirname "$0")/.." && pwd)"
if [[ "${1:-}" == metadata ]]; then
  printf '%s %s\n' "$PWD" "$*" >> "$scene_directory/metadata-calls"
  if [[ -e "$scene_directory/metadata-path-missing" ]]; then
    printf '%s\n' "error: failed to get \`served\` as a dependency of package \`demo v0.1.0 ($PWD/crates/demo)\`" '' 'Caused by:' \
      '  failed to load source for dependency `served`' '' 'Caused by:' "  unable to update $PWD/served" '' 'Caused by:' \
      "  failed to read \`$PWD/served/Cargo.toml\`" '' 'Caused by:' '  No such file or directory (os error 2)' >&2
    exit 101
  fi
  if [[ -e "$scene_directory/metadata-missing" ]]; then
    printf '%s\n' 'error: no matching package named `vstd` found' 'location searched: registry `crates-io`' \
      "required by package \`singlefs-replay-prefix v0.1.0 ($PWD/crates/singlefs-replay-prefix)\`" \
      "As a reminder, you're using offline mode (--offline) which can sometimes cause surprising resolution failures," >&2
    exit 101
  fi
  exit 0
fi
exec "$(cat "$scene_directory/toolchain-cargo")" "$@"
FAKE_CARGO
  chmod +x "$scene/peer-bin/cargo"
  : > "$scene/metadata-missing"
  run_in offline missing --peer --memory-cap 256M --label offline-missing -- bash -c "touch $(printf '%q' "$scene/missing-ran")"
  expect "Ⓒ 第二台那里 cargo metadata --offline 报缺依赖 vstd ⇒ 拒（退 255，说第二台缺依赖 vstd、带同步办法），命令没跑、第二台没建这一趟的目录" \
    "$([[ "$(exit_of offline missing)" == 255 && ! -e "$scene/missing-ran" && "$(peer_runs_left offline)" == 0 ]] \
       && grep -q '第二台缺依赖 vstd' "$scene/missing.err" && grep -q 'cargo fetch' "$scene/missing.err"; echo $?)" \
    "退 $(exit_of offline missing)，命令$([[ -e "$scene/missing-ran" ]] && echo 跑了 || echo 没跑)，第二台留着 $(peer_runs_left offline) 条；stderr 尾部：$(tail_of offline missing)"
  rm -f -- "$scene/metadata-missing" "$scene/metadata-calls"
  run_in offline ok --peer --memory-cap 256M --label offline-ok -- bash -c "touch $(printf '%q' "$scene/ok-ran")"
  expect "Ⓒ 第二台那里 cargo metadata --offline 过了 ⇒ 照跑、退 0，假 cargo 记下在工作目录所在那棵树的根跑过 metadata --offline" \
    "$([[ "$(exit_of offline ok)" == 0 && -e "$scene/ok-ran" ]] \
       && grep -qxF "$(git -C "$scene/repo" rev-parse --show-toplevel) metadata --offline --format-version 1 --filter-platform $("$toolchain_rustc" -vV | sed -n 's/^host: //p')" "$scene/metadata-calls"; echo $?)" \
    "退 $(exit_of offline ok)，metadata 那一步记下的：$(cat "$scene/metadata-calls" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of offline ok)"
  # Ⓒ 离线依赖里缺的是 path 依赖：小仓里一个成员 crates/demo 写 served = { path = "../../served" }，served/ 不在；假 cargo 照 cargo 的原话报取不到 served
  mkdir -p "$scene/repo/crates/demo"
  printf '[package]\nname = "demo"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\nserved = { path = "../../served" }\n' > "$scene/repo/crates/demo/Cargo.toml"
  : > "$scene/metadata-path-missing"
  run_in offline pathmissing --peer --memory-cap 256M --label offline-path-missing -- bash -c "touch $(printf '%q' "$scene/path-missing-ran")"
  expect "Ⓒ 第二台那里 cargo metadata --offline 报缺的 served 在树里的 Cargo.toml 里写成 path 依赖、目录不在 ⇒ 拒（退 255，说树里缺 path 依赖 served、两台都编不了），不报第二台缺依赖，命令没跑" \
    "$([[ "$(exit_of offline pathmissing)" == 255 && ! -e "$scene/path-missing-ran" && "$(peer_runs_left offline)" == 0 ]] \
       && grep -q '树里缺 path 依赖 served：crates/demo/Cargo.toml 里 served 写成 path 依赖' "$scene/pathmissing.err" && grep -q '两台都编不了' "$scene/pathmissing.err" \
       && ! grep -q '第二台缺依赖' "$scene/pathmissing.err"; echo $?)" \
    "退 $(exit_of offline pathmissing)，命令$([[ -e "$scene/path-missing-ran" ]] && echo 跑了 || echo 没跑)；stderr 尾部：$(tail_of offline pathmissing)"
  rm -f -- "$scene/metadata-path-missing"

  # Ⓔ 跑完删第二台这一趟的本地目录
  expect "Ⓔ Ⓑ 那几趟（含退 7 的那一趟）跑完，第二台 runs/ 与 builds/ 下什么都不剩" \
    "$([[ "$(peer_runs_left peer)" == 0 && -d "$work/peer-homes/peer/runs" ]]; echo $?)" \
    "Ⓑ 那边留着：$(ls -A "$work/peer-homes/peer/runs" "$work/peer-homes/peer/builds" 2>/dev/null | tr '\n' ' ')"

  # Ⓕ 第二台那条命令在跑时收到 TERM
  prepare_scene signalled
  scene="$work/signalled"
  local driver_pid signalled_exit waited sleeper_pid="" sleeper_alive=0
  scene_command_for signalled
  ( cd "$scene/repo" && exec "${scene_command[@]}" --peer --memory-cap 256M --label signalled \
      -- bash -c "echo \$\$ > $(printf '%q' "$scene/sleeper.pid"); exec sleep 300" ) > "$scene/run.out" 2> "$scene/run.err" < /dev/null &
  driver_pid=$!
  for (( waited = 0; waited < 600; waited++ )); do
    [[ -s "$scene/sleeper.pid" ]] && break
    sleep 0.1
  done
  sleeper_pid="$(cat "$scene/sleeper.pid" 2>/dev/null)"
  # 第二台那条命令握着的 0、1、2 之外的 fd（脱离会话起的那一份不该握驱动那一头的管道）
  local sleeper_fd_path sleeper_fd extra_fds="" driver_stuck=0
  if [[ -n "$sleeper_pid" ]]; then
    for sleeper_fd_path in "/proc/$sleeper_pid/fd"/*; do
      sleeper_fd="${sleeper_fd_path##*/}"
      if [[ "$sleeper_fd" =~ ^[0-9]+$ ]] && (( sleeper_fd > 2 )); then extra_fds+="$sleeper_fd -> $(readlink "$sleeper_fd_path")；"; fi
    done
  fi
  kill -TERM "$driver_pid"
  # 驱动收到 TERM 要当场收尾：至多等 120 秒（第二台那条 sleep 300 自己走完之前）；还没退就记下、停它（只停这一格自己起的那个进程号）
  for (( waited = 0; waited < 1200; waited++ )); do
    peer_host_process_is_alive "$driver_pid" || break
    sleep 0.1
  done
  if peer_host_process_is_alive "$driver_pid"; then
    driver_stuck=1
    kill -KILL "$driver_pid"
  fi
  if wait "$driver_pid"; then signalled_exit=0; else signalled_exit=$?; fi
  if [[ -n "$sleeper_pid" ]] && peer_host_process_is_alive "$sleeper_pid"; then
    sleeper_alive=1
    kill -TERM "$sleeper_pid"  # 弄坏开关下没停掉的那一条：只停这一格自己起的那个进程号
  fi
  expect "Ⓕ 第二台那条命令在跑时收到 TERM ⇒ 退 143，第二台那条命令不在了，runs/ 与 builds/ 下什么都不剩" \
    "$([[ "$driver_stuck" == 0 && "$signalled_exit" == 143 && -n "$sleeper_pid" && "$sleeper_alive" == 0 && "$(peer_runs_left signalled)" == 0 ]]; echo $?)" \
    "$([[ "$driver_stuck" == 1 ]] && echo "驱动收到 TERM 之后 120 秒还没退（发了 KILL）；")退 $signalled_exit，第二台那条命令的进程号「$sleeper_pid」$([[ "$sleeper_alive" == 1 ]] && echo 还活着 || echo 不在了)；留着 $(peer_runs_left signalled) 条；stderr 尾部：$(tail -n 4 "$scene/run.err" | tr '\n' '|')"
  expect "Ⓕ 第二台那条命令只握 0、1、2，不握驱动那一头的管道" \
    "$([[ -n "$sleeper_pid" && -z "$extra_fds" ]]; echo $?)" \
    "第二台那条命令的进程号「$sleeper_pid」；多握着的：${extra_fds:-（没读到）}"

  # Ⓖ 输出里没有配置里的主机与目录
  local all_outputs=("$work/peer/run.out" "$work/peer/run.err" "$work/peer/seven.err" "$work/signalled/run.err")
  expect "Ⓖ 输出里没有配置里的主机名与第二台的目录，命令打的那两样换成了「第二台」「第二台的目录」" \
    "$(! grep -qE '192[.]0[.]2[.]61|peer-homes|peer-bin' "${all_outputs[@]}" && grep -qx '第二台' "$work/peer/run.err" && grep -q '^第二台的目录/builds/peer-ok$' "$work/peer/run.err"; echo $?)" \
    "有的：$(grep -lE '192[.]0[.]2[.]61|peer-homes|peer-bin' "${all_outputs[@]}" 2>/dev/null | sed "s|^$work/||" | tr '\n' ' ')；命令的输出（接回在驱动的标准错误里）：$(grep '^第二台' "$work/peer/run.err" 2>/dev/null | tr '\n' '|')"

  # Ⓗ 双机开关没开
  prepare_scene switched-off "ENABLE_ACROSS_MACHINES=0"
  scene="$work/switched-off"
  run_in switched-off run --peer --memory-cap 256M -- bash -c "touch $(printf '%q' "$scene/ran")"
  expect "Ⓗ 配置里 ENABLE_ACROSS_MACHINES=0 ⇒ --peer 拒（退 255，出路写 ENABLE_ACROSS_MACHINES=1），命令哪台都没跑、第二台没建目录" \
    "$([[ "$(exit_of switched-off run)" == 255 && ! -e "$scene/ran" && "$(peer_runs_left switched-off)" == 0 ]] && grep -q 'ENABLE_ACROSS_MACHINES=1' "$scene/run.err"; echo $?)" \
    "退 $(exit_of switched-off run)，命令$([[ -e "$scene/ran" ]] && echo 跑了 || echo 没跑)；stderr 尾部：$(tail_of switched-off run)"

  # Ⓘ 弄坏开关设在自证之外：起驱动时去掉自证的标记，弄坏开关照调用方的再加一项 peer-exit-dropped
  prepare_scene outside
  scene="$work/outside"
  scene_command_for outside
  local outside_exit
  if ( cd "$scene/repo" && exec env -u MULTI_HOST_RUN_SELFTEST_MARK MULTI_HOST_RUN_BREAK="${MULTI_HOST_RUN_BREAK:+$MULTI_HOST_RUN_BREAK,}peer-exit-dropped" "${scene_command[@]}" \
       --peer --memory-cap 256M --label outside-break -- bash -c "touch $(printf '%q' "$scene/ran")" ) > "$scene/run.out" 2> "$scene/run.err" < /dev/null; then
    outside_exit=0
  else
    outside_exit=$?
  fi
  expect "Ⓘ MULTI_HOST_RUN_BREAK 设在 --selftest 之外（不带自证的标记）⇒ 拒（退 2，说设在 --selftest 之外），命令没跑" \
    "$([[ "$outside_exit" == 2 && ! -e "$scene/ran" ]] && grep -q 'MULTI_HOST_RUN_BREAK=.* 设在 --selftest 之外' "$scene/run.err"; echo $?)" \
    "退 $outside_exit，命令$([[ -e "$scene/ran" ]] && echo 跑了 || echo 没跑)；stderr 尾部：$(tail -n 3 "$scene/run.err" 2>/dev/null | tr '\n' '|')"
  # Ⓘ 只供测试的「第二台是本机」设在自证之外（文件头「只供测试的开关」）：去掉自证的标记与调用方的两个弄坏开关起驱动（scene_command 里设着它；
  #   MULTI_HOST_CONFIGURATION_BREAK 设在自证之外时驱动先拒它，轮不到「第二台是本机」那一道）。
  #   弄坏开关 this-machine-peer-honored-anywhere 下照它起（另加 break-honored-anywhere：没有标记时弄坏开关要它才生效）
  local outside_peer_exit outside_peer_break=()
  if multi_host_break_is this-machine-peer-honored-anywhere; then outside_peer_break=(MULTI_HOST_RUN_BREAK=break-honored-anywhere,this-machine-peer-honored-anywhere); fi
  if ( cd "$scene/repo" && exec env -u MULTI_HOST_RUN_SELFTEST_MARK -u MULTI_HOST_RUN_BREAK -u MULTI_HOST_CONFIGURATION_BREAK ${outside_peer_break[@]+"${outside_peer_break[@]}"} "${scene_command[@]}" \
       --peer --memory-cap 256M --label outside-peer-variable -- bash -c "touch $(printf '%q' "$scene/peer-variable-ran")" ) > "$scene/peer-variable.out" 2> "$scene/peer-variable.err" < /dev/null; then
    outside_peer_exit=0
  else
    outside_peer_exit=$?
  fi
  expect "Ⓘ 只供测试的 SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 设在 --selftest 之外（不带自证的标记）⇒ 拒（退 2，说只在 --selftest 里认），命令没跑" \
    "$([[ "$outside_peer_exit" == 2 && ! -e "$scene/peer-variable-ran" ]] && grep -q 'SHARD_PEER_IS_THIS_MACHINE=1 设在 --selftest 之外' "$scene/peer-variable.err"; echo $?)" \
    "退 $outside_peer_exit，命令$([[ -e "$scene/peer-variable-ran" ]] && echo 跑了 || echo 没跑)；stderr 尾部：$(tail -n 3 "$scene/peer-variable.err" 2>/dev/null | tr '\n' '|')"

  # Ⓚ 第二台那一层内存包装认出显存不够（假 journalctl 放进第二台 PATH 最前面的 peer-bin，包装照 PATH 取到它：-n 1 那一问打一行，查窗口那一问打 NVRM … NV_ERR_NO_MEMORY）
  prepare_scene gpu-memory
  scene="$work/gpu-memory"
  printf '%s\n' '#!/usr/bin/env bash' '# 自证用的假 journalctl：问最近一条打一行，问起止之间打一行显卡驱动报的显存不够' \
    'case " $* " in *" -n 1 "*) echo "host kernel: probe" ;; *) echo "host kernel: NVRM: GPU0 nvAssertOkFailedNoLog: Out of memory [NV_ERR_NO_MEMORY] (0x00000051)" ;; esac' \
    > "$scene/peer-bin/journalctl"
  chmod +x "$scene/peer-bin/journalctl"
  run_in gpu-memory run --peer --memory-cap 256M --label gpu-memory -- bash -c 'exit 3'
  expect "Ⓚ --peer：第二台的内存包装认出显存不够 ⇒ 原样退 245、报「显存不够」、说那一次的输出不算结果" \
    "$([[ "$(exit_of gpu-memory run)" == 245 ]] && grep -q '退 245：显存不够' "$scene/run.err" && grep -q '不算结果' "$scene/run.err"; echo $?)" \
    "退 $(exit_of gpu-memory run)，stderr 尾部：$(tail_of gpu-memory run)"

  # Ⓜ --peer 命令里写了 CARGO_TARGET_DIR=（上面「逐格」那一段的 Ⓜ）
  local driver_exit=""
  prepare_scene command-target
  scene="$work/command-target"
  run_in command-target assignment --peer --memory-cap 256M --label target-assignment -- env CARGO_TARGET_DIR=../shared-target bash -c "touch $(printf '%q' "$scene/assignment-ran")"
  run_in command-target script --peer --memory-cap 256M --label target-script -- bash -c "export CARGO_TARGET_DIR=/var/tmp/shared-target; touch $(printf '%q' "$scene/script-ran")"
  expect "Ⓜ --peer 命令里写了 CARGO_TARGET_DIR=（env 的赋值、bash -c 的脚本两种写法）⇒ 用法错（退 2），命令没跑" \
    "$([[ "$(exit_of command-target assignment)" == 2 && "$(exit_of command-target script)" == 2 && ! -e "$scene/assignment-ran" && ! -e "$scene/script-ran" ]] \
       && grep -q '命令里写了 CARGO_TARGET_DIR=' "$scene/assignment.err"; echo $?)" \
    "env 那一趟退 $(exit_of command-target assignment)，bash -c 那一趟退 $(exit_of command-target script)；stderr 尾部：$(tail_of command-target assignment)"

  wait_driver() { # <进程号> <秒>：等这一格自己起的驱动退出，至多等这么久，还在就只给它发 KILL；退出码放 driver_exit
    local driver_rounds
    for (( driver_rounds = 0; driver_rounds < $2 * 10; driver_rounds++ )); do peer_host_process_is_alive "$1" || break; sleep 0.1; done
    if peer_host_process_is_alive "$1"; then kill -KILL "$1"; fi
    if wait "$1"; then driver_exit=0; else driver_exit=$?; fi
  }
  wait_for_file() { # <文件> <秒> [<进程号>]：等文件出现，至多等这么久；给了进程号，它退了就不再等
    local file_rounds
    for (( file_rounds = 0; file_rounds < $2 * 10; file_rounds++ )); do
      [[ -s "$1" ]] && return 0
      if [[ -n "${3:-}" ]] && ! peer_host_process_is_alive "$3"; then break; fi
      sleep 0.1
    done
    [[ -s "$1" ]]
  }
  # Ⓠ --peer：第二台那条命令拿到的编译目录在第二台这一趟的本地目录下（不在共享根下），小仓里不建 target，跑完那个目录删了
  prepare_scene peer-target
  scene="$work/peer-target"
  local peer_target_home="$work/peer-homes/peer-target"
  run_in peer-target peer --peer --memory-cap 256M --label target-in-tree -- bash -c 'printf "%s\n" "${CARGO_TARGET_DIR-没设}" > "$0.target"; printf "%s\n" "${MULTI_HOST_RUN_PEER_SCRATCH-没设}" > "$0.scratch"
target_directory="${CARGO_TARGET_DIR:-target}"; mkdir -p "$target_directory/debug" && echo built > "$target_directory/debug/artifact"' "$scene/peer"
  expect "Ⓠ --peer：第二台那条命令拿到的 CARGO_TARGET_DIR 是第二台这一趟的本地目录下的 target（不在共享根下），共享的树里没建 target，跑完那个目录删了" \
    "$([[ "$(exit_of peer-target peer)" == 0 && "$(cat "$scene/peer.target" 2>/dev/null)" == "$peer_target_home/builds/target-in-tree/target" \
       && "$(cat "$scene/peer.scratch" 2>/dev/null)" == "$peer_target_home/builds/target-in-tree" && ! -e "$scene/repo/target" \
       && ! -e "$peer_target_home/builds/target-in-tree" && "$(peer_runs_left peer-target)" == 0 ]]; echo $?)" \
    "退 $(exit_of peer-target peer)，第二台那条命令拿到「$(cat "$scene/peer.target" 2>/dev/null)」、本地目录「$(cat "$scene/peer.scratch" 2>/dev/null)」，小仓里$([[ -e "$scene/repo/target" ]] && echo 建了 || echo 没建) target，那个目录$([[ -e "$peer_target_home/builds/target-in-tree" ]] && echo 还在 || echo 不在)；stderr 尾部：$(tail_of peer-target peer)"
  # 编译目录占盘（文件头 ⑤「编译目录占盘」）：上面那一趟往编译目录里写了一份产物，结束时恰好一行 MULTI_HOST_RUN_PEER_BUILD_PEAK bytes=<正整数>；
  #   再跑一趟、命令自己把编译目录删掉（量不到）⇒ 退 0、不打那一行
  local build_peak_problems removed_build_peak_lines
  build_peak_problems="$(awk '/^MULTI_HOST_RUN_PEER_BUILD_PEAK/ { lines++; text = $0 }
    END { if (lines != 1) print "占盘行 " (lines + 0) " 行"; else if (text !~ /^MULTI_HOST_RUN_PEER_BUILD_PEAK bytes=[1-9][0-9]*$/) print "格式不对：" text }' "$scene/peer.err" 2>/dev/null)"
  expect "编译目录占盘 --peer：第二台那条命令结束时打恰好一行 MULTI_HOST_RUN_PEER_BUILD_PEAK bytes=<正整数>（这一趟本地编译目录删之前 du -s -b 量的）" \
    "$([[ "$(exit_of peer-target peer)" == 0 && -z "$build_peak_problems" ]]; echo $?)" \
    "${build_peak_problems:-无}；stderr 尾部：$(tail_of peer-target peer)"
  run_in peer-target build-removed --peer --memory-cap 256M --label target-removed -- bash -c 'removed="$MULTI_HOST_RUN_PEER_SCRATCH/target"; rm -rf -- "${removed:?}"'   # 照这一趟的本地目录删（不靠 CARGO_TARGET_DIR：开关 peer-build-in-shared-tree 下它没设）；词里不写 }/ 起头的一截（文件头 ③ 的认法会把它当绝对路径）
  removed_build_peak_lines="$(grep -c '^MULTI_HOST_RUN_PEER_BUILD_PEAK' "$scene/build-removed.err" 2>/dev/null)" || true
  expect "编译目录占盘 --peer：命令自己删了编译目录（量不到）⇒ 退 0、不打 MULTI_HOST_RUN_PEER_BUILD_PEAK 那一行" \
    "$([[ "$(exit_of peer-target build-removed)" == 0 && "${removed_build_peak_lines:-0}" == 0 ]]; echo $?)" \
    "退 $(exit_of peer-target build-removed)，占盘行 ${removed_build_peak_lines:-0} 行：$(grep '^MULTI_HOST_RUN_PEER_BUILD_PEAK' "$scene/build-removed.err" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of peer-target build-removed)"
  # 留着的编译目录（文件头「留着的编译目录」）：带 --keep-build-directory 的一趟往编译目录里写一份产物 ⇒ 退 0、builds/keep-first 留着、恰好一行 MULTI_HOST_RUN_PEER_BUILD_KEPT label=keep-first、
  #   runs/ 下不剩；下一趟 --reuse-build-directory keep-first（不带 --keep-build-directory）⇒ 退 0、拿到的 CARGO_TARGET_DIR 是 builds/keep-first/target、读得到上一趟的产物，
  #   跑完连它删了；另留一份 keep-removed 再 --remove-build-directory keep-removed ⇒ 退 0、删了；点名不在的 keep-missing ⇒ 退 255、命令没跑
  prepare_scene peer-keep
  scene="$work/peer-keep"
  local keep_home="$work/peer-homes/peer-keep" keep_lines
  run_in peer-keep kept --peer --memory-cap 256M --label keep-first --keep-build-directory -- bash -c 'mkdir -p "$CARGO_TARGET_DIR/debug" && echo built-once > "$CARGO_TARGET_DIR/debug/artifact"'
  keep_lines="$(grep -c '^MULTI_HOST_RUN_PEER_BUILD_KEPT label=keep-first$' "$scene/kept.err" 2>/dev/null)" || true
  local kept_present=no runs_after_keep
  [[ -f "$keep_home/builds/keep-first/target/debug/artifact" ]] && kept_present=yes
  runs_after_keep="$(find "$keep_home/runs" -mindepth 1 -maxdepth 1 2>/dev/null | wc -l)"
  run_in peer-keep reused --peer --memory-cap 256M --label keep-second --reuse-build-directory keep-first -- bash -c 'printf "%s\n" "$CARGO_TARGET_DIR" > "$0.target"; cat "$CARGO_TARGET_DIR/debug/artifact" > "$0.artifact"' "$scene/reused"
  run_in peer-keep kept-again --peer --memory-cap 256M --label keep-removed --keep-build-directory -- bash -c 'true'
  local kept_again_present=no
  [[ -d "$keep_home/builds/keep-removed/target" ]] && kept_again_present=yes
  run_in peer-keep removed --remove-build-directory keep-removed
  run_in peer-keep missing --peer --memory-cap 256M --label keep-third --reuse-build-directory keep-missing -- bash -c "touch $(printf '%q' "$scene/missing-ran")"
  expect "留着 --keep-build-directory 跑完留着这一趟的编译目录、打 MULTI_HOST_RUN_PEER_BUILD_KEPT；下一趟 --reuse-build-directory 接着用的就是它，跑完连它删了；--remove-build-directory 删留着的那一份；点名不在的退 255、命令没跑" \
    "$([[ "$(exit_of peer-keep kept)" == 0 && "$kept_present" == yes && "${keep_lines:-0}" == 1 && "$runs_after_keep" == 0 \
       && "$(exit_of peer-keep reused)" == 0 && "$(cat "$scene/reused.target" 2>/dev/null)" == "$keep_home/builds/keep-first/target" \
       && "$(cat "$scene/reused.artifact" 2>/dev/null)" == built-once && ! -e "$keep_home/builds/keep-first" && ! -e "$keep_home/builds/keep-second" \
       && "$(exit_of peer-keep kept-again)" == 0 && "$kept_again_present" == yes && "$(exit_of peer-keep removed)" == 0 && ! -e "$keep_home/builds/keep-removed" \
       && "$(exit_of peer-keep missing)" == 255 && ! -e "$scene/missing-ran" && "$(peer_runs_left peer-keep)" == 0 ]]; echo $?)" \
    "留着的那一趟退 $(exit_of peer-keep kept)、留着那一份 $([[ "$kept_present" == yes ]] && echo 在 || echo 不在)、MULTI_HOST_RUN_PEER_BUILD_KEPT ${keep_lines:-0} 行、runs/ 下剩 $runs_after_keep 个；接着用的那一趟退 $(exit_of peer-keep reused)、拿到「$(cat "$scene/reused.target" 2>/dev/null)」、读到「$(cat "$scene/reused.artifact" 2>/dev/null)」、keep-first 跑完$([[ -e "$keep_home/builds/keep-first" ]] && echo 还在 || echo 删了)；--remove-build-directory 退 $(exit_of peer-keep removed)、keep-removed $([[ -e "$keep_home/builds/keep-removed" ]] && echo 还在 || echo 删了)；点名不在的退 $(exit_of peer-keep missing)、命令$([[ -e "$scene/missing-ran" ]] && echo 跑了 || echo 没跑)；stderr 尾部：$(tail_of peer-keep kept)"
  # Ⓡ 删之前核开着的文件：命令记下编译目录（本机）或这一趟的本地目录（--peer）后等放行文件；自证在那里起一个 sleep 开着一份文件（只记它自己的进程号），再放行。
  #   驱动该不删、报出那个进程、退 255；之后只停那一个进程号，手删留下的那一份
  prepare_scene holders
  scene="$work/holders"
  # 只给自己起的那一个 sleep 的进程号发一次终止信号，不在循环里发信号
  selftest_holder_case() { # <这一趟的名字>
    local holder_case="$1" holder_driver holder_sleep_pid holder_where
    scene_command_for holders
    ( cd "$scene/repo" && exec "${scene_command[@]}" --peer --memory-cap 256M --label holders-peer -- bash -c 'printf "%s\n" "$MULTI_HOST_RUN_PEER_SCRATCH" > "$0.where"; until test -e "$0.go"; do sleep 0.1; done' "$scene/$holder_case" ) \
      > "$scene/$holder_case.out" 2> "$scene/$holder_case.err" < /dev/null &
    holder_driver=$!
    holder_sleep_pid=""
    holder_where=""
    if wait_for_file "$scene/$holder_case.where" 180 "$holder_driver"; then
      holder_where="$(cat "$scene/$holder_case.where")"
      sleep 60 > "$holder_where/held-by-selftest" 2>&1 < /dev/null &
      holder_sleep_pid=$!
      # 放行之前等那个 sleep 真开着那份文件（它的 1 号 fd 指过去），不然驱动删的时候它可能还没开
      for (( waited = 0; waited < 100; waited++ )); do
        [[ "$(readlink "/proc/$holder_sleep_pid/fd/1" 2>/dev/null)" == "$holder_where/held-by-selftest" ]] && break
        sleep 0.1
      done
    fi
    : > "$scene/$holder_case.go"
    wait_driver "$holder_driver" 120
    echo "$driver_exit" > "$scene/$holder_case.exit"
    [[ -n "$holder_where" && -d "$holder_where" ]] && echo yes > "$scene/$holder_case.kept"
    [[ -z "$holder_sleep_pid" ]] || { kill "$holder_sleep_pid" 2>/dev/null; wait "$holder_sleep_pid" 2>/dev/null; }
  }
  selftest_holder_case peer
  rm -rf -- "${work:?}/peer-homes/holders/runs" "${work:?}/peer-homes/holders/builds"
  expect "Ⓡ --peer：第二台那条命令退了、还有进程开着这一趟本地目录里的文件 ⇒ ⑥ 不删、报出那个进程、退 255" \
    "$([[ "$(exit_of holders peer)" == 255 && -e "$scene/peer.kept" ]] && grep -q '这一趟的本地目录里还有进程开着文件' "$scene/peer.err" && grep -q '（sleep 60）' "$scene/peer.err"; echo $?)" \
    "退 $(exit_of holders peer)，这一趟的本地目录「$(cat "$scene/peer.where" 2>/dev/null)」$([[ -e "$scene/peer.kept" ]] && echo 还在 || echo 删了)；stderr 尾部：$(tail_of holders peer)"

  # Ⓞ 第二台起含 cargo 的命令之前判第二台的 RocksDB 预编库取值（文件头 ⑤「RocksDB 预编库的取值」）：第二台的 PATH 前面放不带 entry 认的标记的假 cargo
  #   （-V 转给假工具链的 cargo，工具链照比；别的就建一份「跑了」的文件），本机当第二台时环境照 ssh 那样只给最少几样、四个取值都没有，小仓的 scripts/experiment-apparatus/Cargo.toml
  #   依赖 rocksdb ⇒ 退 78、假 cargo 没起、输出里有 rocksdb-prebuilt entry 的红句；同一场景里命令不含 cargo 的照跑、退 0
  prepare_scene rocksdb
  scene="$work/rocksdb"
  printf '%s\n' '#!/usr/bin/env bash' '# 自证用的 cargo（Ⓞ）：不带 entry 认的标记；-V / --version 转给假工具链的 cargo（两台工具链照比），别的建一份「跑了」的文件、退 0' \
    "case \"\${1:-}\" in -V|--version) exec $(printf '%q' "$toolchain_cargo") \"\$@\" ;; esac" ": > $(printf '%q' "$scene/cargo-ran")" > "$scene/peer-bin/cargo"
  chmod +x "$scene/peer-bin/cargo"
  run_in rocksdb refused --peer --memory-cap 256M --label rocksdb-refused -- cargo build
  run_in rocksdb plain --peer --memory-cap 256M --label rocksdb-plain -- bash -c "touch $(printf '%q' "$scene/plain-ran")"
  expect "Ⓞ 第二台起含 cargo 的命令之前判第二台的取值：取值缺了 ⇒ 退 78、假 cargo 没起；不含 cargo 的命令不判、照跑退 0" \
    "$([[ "$(exit_of rocksdb refused)" == 78 && ! -e "$scene/cargo-ran" && "$(exit_of rocksdb plain)" == 0 && -e "$scene/plain-ran" ]] \
       && grep -q 'rocksdb-prebuilt entry：' "$scene/refused.err"; echo $?)" \
    "含 cargo 的退 $(exit_of rocksdb refused)、假 cargo $([[ -e "$scene/cargo-ran" ]] && echo 起了 || echo 没起)，不含的退 $(exit_of rocksdb plain)；接回的 rocksdb-prebuilt 那几行：$(grep 'rocksdb-prebuilt' "$scene/refused.err" 2>/dev/null | tail -n 3 | tr '\n' '|')；stderr 尾部：$(tail_of rocksdb refused)"

  # Ⓣ 自证的细节行按字截（cut_characters）：一行 200 个汉字截到 161 个字，截出来要是合法的 UTF-8、正好 161 个字
  #   （按字节截到 161 字节落在一个汉字的中间，留下半个字；细节行这样截过的开关趟输出解不成 UTF-8）。细节里只报字节数与能不能解，不把截出来的字节原样打出来
  local cut_result cut_judged
  cut_result="$(printf '第二台在用%.0s' {1..40} | cut_characters 161)"
  cut_judged="$(printf '%s' "$cut_result" | python3 -c 'import sys
data = sys.stdin.buffer.read()
try:
    text = data.decode("utf-8")
except UnicodeDecodeError as error:
    print(f"{len(data)} 字节，第 {error.start} 字节起解不成 UTF-8")
else:
    print(f"{len(data)} 字节，{len(text)} 个字")')"
  expect "Ⓣ 自证的细节行按字截：比上限长的一行截到上限那么多个字，截出来是合法的 UTF-8" \
    "$([[ "$cut_judged" == *"，161 个字" ]]; echo $?)" \
    "200 个字截到 161：截出来 ${cut_judged:-（没读到）}"

  # Ⓤ 两台用同一份多机配置（文件头 ⑤「带到第二台的环境」；scripts/hosts/peer-host-lib.sh 文件头「两台用同一份多机配置」）：--peer 一趟，第二台那条命令记下它看到的
  #   SINGLEFS_MULTI_HOST_CONFIG 与那份文件的 sha256；要是场景那一份（同一个绝对路径）、sha256 相同，第二台专用目录下不留常驻的那一份
  local scene_sum seen_line standing_sum
  local seen_command='printf "configuration=%s sum=%s origin=%s\n" "${SINGLEFS_MULTI_HOST_CONFIG-没设}" "$(if test -f "${SINGLEFS_MULTI_HOST_CONFIG:-}"; then sha256sum < "$SINGLEFS_MULTI_HOST_CONFIG" | cut -d " " -f 1; else echo 不在; fi)" "${GPU_QUIESCE_ORIGIN_READ_BACK-没设}" > "$0"'
  prepare_scene carry
  scene="$work/carry"
  run_in carry run --peer --memory-cap 256M --label carry -- bash -c "$seen_command" "$scene/seen"
  scene_sum="$(sha256sum < "$scene/multi-host.env" | cut -d' ' -f1)"
  seen_line="$(cat "$scene/seen" 2>/dev/null)"
  standing_sum="$(sha256sum < "$work/peer-homes/carry/multi-host.env" 2>/dev/null | cut -d' ' -f1)"
  expect "Ⓤ --peer：第二台那条命令读的是本机这一份多机配置（同一个绝对路径、sha256 相同），第二台的专用目录里不留常驻的那一份" \
    "$([[ "$(exit_of carry run)" == 0 && "$seen_line" == "configuration=$scene/multi-host.env sum=$scene_sum origin=没设" && -z "$standing_sum" ]]; echo $?)" \
    "退 $(exit_of carry run)；第二台那条命令看到的：${seen_line:-（没记下）}；本机这一份 ${scene_sum:0:12}，常驻那一份 ${standing_sum:-不在}；stderr 尾部：$(tail_of carry run)"

  # Ⓦ 两台对称（文件头 ①；scripts/crash/crash-point-replay-shard-configuration-check.sh 文件头「两台对称」）：第二台的 PATH 最前面（CARGO_BIN_DIRECTORY_BY_HOST 里第二台那一项）放假 ssh，
  #   被起一次就往场景目录记一行、退 255。第二台那条命令照本机这一份配置单跑判法、带 --emit-assignments 跑判法（带着只供测试的「第二台是本机」与 peer-host-lib.sh 的标记，
  #   同这一侧的驱动），再起 multi-host-run.sh --peer（带着自证的标记与调用方的弄坏开关：里层照收时它往配置里的另一台 ssh）；
  #   第二台那条命令的环境照 ssh 那样只给最少几样，判法的弄坏开关 MULTI_HOST_CONFIGURATION_BREAK 不带过去：那条命令由 setsid nohup 起、起它的 shell 一退它就归 init，
  #   判法脚本沿祖先找不到 --selftest，一见设着的弄坏开关就退 2（这一格判的是两侧结局相同，弄坏开关归 Ⓗ 那一格）。这一侧在小仓的根里照同一份场景配置、驱动的那一份环境也跑那两趟判法，两边逐项比。
  #   命令里点到的两份标记放进这一场景的共享根（③ 拒命令里共享根外的路径）：照自证那两份的内容另写一份
  prepare_scene peer-side
  scene="$work/peer-side"
  printf '%s\n' "$PEER_HOST_SELFTEST_MARK_CONTENT" > "$scene/peer-host-selftest-mark"
  printf '%s\n' "$MULTI_HOST_SELFTEST_MARK_CONTENT" > "$scene/selftest-mark"
  printf '%s\n' '#!/usr/bin/env bash' '# multi-host-run.sh 自证 Ⓦ 的假 ssh：第二台上被起一次就记一行，照连不上退 255' "printf '%s\n' \"\$*\" >> $(printf '%q' "$scene/ssh.log")" 'exit 255' \
    > "$scene/peer-bin/ssh"
  chmod +x "$scene/peer-bin/ssh"
  local peer_side_test="SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 PEER_HOST_SELFTEST_MARK=$(printf '%q' "$scene/peer-host-selftest-mark")"
  local nested_marks="MULTI_HOST_RUN_SELFTEST_MARK=$(printf '%q' "$scene/selftest-mark") MULTI_HOST_RUN_BREAK=$(printf '%q' "${MULTI_HOST_RUN_BREAK:-}")"
  run_in peer-side run --peer --memory-cap 256M --label peer-side -- bash -c "
env $peer_side_test bash scripts/crash/crash-point-replay-shard-configuration-check.sh . > \"\$0.check\" 2>&1; echo \$? > \"\$0.check-exit\"
env $peer_side_test bash scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-assignments . > \"\$0.emit\" 2>&1; echo \$? > \"\$0.emit-exit\"
env $nested_marks bash scripts/hosts/multi-host-run.sh --peer -- true > \"\$0.nested\" 2>&1; echo \$? > \"\$0.nested-exit\"
cat $(printf '%q' "$scene/ssh.log") 2>/dev/null | wc -l > \"\$0.ssh-count\"" "$scene/peer"
  # 这一侧：驱动的那一份环境（scene_command 去掉末三个词：服务给的标记与 bash <驱动>）
  local this_side_check_exit=0 this_side_emit_exit=0
  local this_side_environment=("${scene_command[@]:0:${#scene_command[@]}-3}")
  ( cd "$scene/repo" && exec "${this_side_environment[@]}" bash scripts/crash/crash-point-replay-shard-configuration-check.sh . ) > "$scene/this.check" 2>&1 < /dev/null \
    || this_side_check_exit=$?
  ( cd "$scene/repo" && exec "${this_side_environment[@]}" bash scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-assignments . ) > "$scene/this.emit" 2>&1 < /dev/null \
    || this_side_emit_exit=$?
  expect "Ⓦ 第二台那条命令里判同一份场景配置，结局（退出码与 --emit-assignments 打的赋值）与这一侧判的相同，再起 multi-host-run.sh --peer 照拒（退 255），一次都没往别的台 ssh" \
    "$([[ "$(exit_of peer-side run)" == 0 && "$this_side_check_exit" == 0 && "$(cat "$scene/peer.check-exit" 2>/dev/null)" == "$this_side_check_exit" \
       && "$this_side_emit_exit" == 0 && "$(cat "$scene/peer.emit-exit" 2>/dev/null)" == "$this_side_emit_exit" && "$(cat "$scene/peer.nested-exit" 2>/dev/null)" == 255 \
       && "$(cat "$scene/peer.ssh-count" 2>/dev/null)" == 0 ]] && grep -q '^PEER_HOST_ADDRESS=' "$scene/this.emit" && cmp -s "$scene/this.emit" "$scene/peer.emit" \
       && grep -q '把这条命令交给 task-server' "$scene/peer.nested"; echo $?)" \
    "第二台那里判法退 $(cat "$scene/peer.check-exit" 2>/dev/null)、emit 退 $(cat "$scene/peer.emit-exit" 2>/dev/null)；这一侧判法退 $this_side_check_exit、emit 退 $this_side_emit_exit；emit 两边$(cmp -s "$scene/this.emit" "$scene/peer.emit" && echo 逐字相同 || echo 不同)；里层 --peer 退 $(cat "$scene/peer.nested-exit" 2>/dev/null)（$(tail -n 2 "$scene/peer.nested" 2>/dev/null | tr '\n' ' ' | cut_characters 120)）；ssh 记了 $(cat "$scene/peer.ssh-count" 2>/dev/null || echo 没数) 次；stderr 尾部：$(tail_of peer-side run)"

  # Ⓥ 任务里面再带 --peer（文件头 ①）：起驱动时去掉服务给最外层那一次的标记（scene_command 倒数第三个词）⇒ 拒、退 255、说交给 task-server，命令没跑、第二台没建这一趟的目录
  prepare_scene nested
  scene="$work/nested"
  scene_command_for nested
  local nested_exit
  if ( cd "$scene/repo" && exec "${scene_command[@]:0:${#scene_command[@]}-3}" "${scene_command[@]: -2}" \
       --peer --memory-cap 256M --label nested-local -- bash -c "touch $(printf '%q' "$scene/ran")" ) > "$scene/run.out" 2> "$scene/run.err" < /dev/null; then
    nested_exit=0
  else
    nested_exit=$?
  fi
  expect "Ⓥ 任务里面再带 --peer（这一台：起它的不是服务的尝试单元，没有最外层那一次的标记）⇒ 拒（退 255，出路写把这条命令交给 task-server），命令没跑、第二台没建这一趟的目录" \
    "$([[ "$nested_exit" == 255 && ! -e "$scene/ran" && "$(peer_runs_left nested)" == 0 ]] && grep -q '把这条命令交给 task-server' "$scene/run.err"; echo $?)" \
    "退 $nested_exit，命令$([[ -e "$scene/ran" ]] && echo 跑了 || echo 没跑)；stderr 尾部：$(tail -n 3 "$scene/run.err" 2>/dev/null | tr '\n' '|' | cut_characters 300)"

  # Ⓧ 起活这一侧清没清场：驱动正经 gpu-quiesce-run.sh 起（GPU_QUIESCE_HOLDER 指的登记文件在、名字以驱动的父进程号——这个自证的进程——加「-」起头）⇒ 第二台那条命令拿到
  #   GPU_QUIESCE_ORIGIN_READ_BACK=1；登记文件名字的进程号对不上 ⇒ 没设。驱动在这个进程的直接子进程里起（cd 之后 exec），父进程号就是它
  prepare_scene origin-quiesce
  scene="$work/origin-quiesce"
  mkdir -p "$scene/holders"
  : > "$scene/holders/$BASHPID-1"
  : > "$scene/holders/1-1"
  scene_command_for origin-quiesce
  local origin_command=("${scene_command[@]:0:${#scene_command[@]}-2}")
  local origin_parent="$BASHPID"
  ( cd "$scene/repo" && exec "${origin_command[@]}" GPU_QUIESCE_HOLDER="$scene/holders/$origin_parent-1" "${scene_command[@]: -2}" --peer --memory-cap 256M --label origin-held -- bash -c "$seen_command" "$scene/held.seen" ) \
    > "$scene/held.out" 2> "$scene/held.err" < /dev/null
  echo "$?" > "$scene/held.exit"
  ( cd "$scene/repo" && exec "${origin_command[@]}" GPU_QUIESCE_HOLDER="$scene/holders/1-1" "${scene_command[@]: -2}" --peer --memory-cap 256M --label origin-other -- bash -c "$seen_command" "$scene/other.seen" ) \
    > "$scene/other.out" 2> "$scene/other.err" < /dev/null
  echo "$?" > "$scene/other.exit"
  expect "Ⓧ 驱动正经 gpu-quiesce-run.sh 起（登记文件在、名字以驱动的父进程号起头）⇒ 第二台那条命令拿到 GPU_QUIESCE_ORIGIN_READ_BACK=1；不是 ⇒ 没设" \
    "$([[ "$(exit_of origin-quiesce held)" == 0 && "$(exit_of origin-quiesce other)" == 0 && "$(sed -n 's/.* origin=//p' "$scene/held.seen" 2>/dev/null)" == 1 \
       && "$(sed -n 's/.* origin=//p' "$scene/other.seen" 2>/dev/null)" == 没设 ]]; echo $?)" \
    "那一趟退 $(exit_of origin-quiesce held)、看到 origin=$(sed -n 's/.* origin=//p' "$scene/held.seen" 2>/dev/null)；不是那一趟退 $(exit_of origin-quiesce other)、看到 origin=$(sed -n 's/.* origin=//p' "$scene/other.seen" 2>/dev/null)"

  # Ⓩ 共享根（文件头 ③）：命令里有共享根外的绝对路径（整个词、bash -c 词里各一趟）、工作目录在共享根外 ⇒ 退 255、点名；共享根下的与 /dev/null 照跑
  prepare_scene outside-paths
  scene="$work/outside-paths"
  run_in outside-paths word --peer --memory-cap 256M --label outside-word -- bash -c 'touch "$0"' "$outside_shared/word-ran"
  run_in outside-paths quoted --peer --memory-cap 256M --label outside-quoted -- bash -c "touch $(printf '%q' "$outside_shared/quoted-ran")"
  expect "Ⓩ --peer 命令里有共享根外的绝对路径（整个词、bash -c 词里各一趟）⇒ 退 255、点名那个路径，命令没跑、第二台没建这一趟的目录" \
    "$([[ "$(exit_of outside-paths word)" == 255 && "$(exit_of outside-paths quoted)" == 255 && ! -e "$outside_shared/word-ran" && ! -e "$outside_shared/quoted-ran" \
       && "$(peer_runs_left outside-paths)" == 0 ]] && grep -qF "命令里点到的 $outside_shared/word-ran 不在多机配置 NFS_SHARED_ROOTS" "$scene/word.err" \
       && grep -qF "命令里点到的 $outside_shared/quoted-ran 不在多机配置 NFS_SHARED_ROOTS" "$scene/quoted.err"; echo $?)" \
    "整个词那一趟退 $(exit_of outside-paths word)，bash -c 那一趟退 $(exit_of outside-paths quoted)，命令$([[ -e "$outside_shared/word-ran" || -e "$outside_shared/quoted-ran" ]] && echo 跑了 || echo 没跑)；stderr 尾部：$(tail_of outside-paths word | cut_characters 300)"
  scene_command_for outside-paths
  ( cd "$outside_shared" && exec "${scene_command[@]}" --peer --memory-cap 256M --label outside-cwd -- bash -c "touch $(printf '%q' "$scene/cwd-ran")" ) \
    > "$scene/cwd.out" 2> "$scene/cwd.err" < /dev/null
  echo "$?" > "$scene/cwd.exit"
  expect "Ⓩ --peer 在共享根外的工作目录里起 ⇒ 退 255、说工作目录不在共享根下，命令没跑" \
    "$([[ "$(exit_of outside-paths cwd)" == 255 && ! -e "$scene/cwd-ran" ]] && grep -q '这一刻的工作目录 .* 不在多机配置 NFS_SHARED_ROOTS' "$scene/cwd.err"; echo $?)" \
    "退 $(exit_of outside-paths cwd)，命令$([[ -e "$scene/cwd-ran" ]] && echo 跑了 || echo 没跑)；stderr 尾部：$(tail_of outside-paths cwd | cut_characters 300)"
  run_in outside-paths inside --peer --memory-cap 256M --label inside -- bash -c "touch $(printf '%q' "$scene/inside-ran") > /dev/null" "$scene/repo/.gitignore"
  expect "Ⓩ --peer 命令里的路径都在共享根下或是两台相同的系统路径（/dev/null）⇒ 照跑、退 0" \
    "$([[ "$(exit_of outside-paths inside)" == 0 && -e "$scene/inside-ran" ]]; echo $?)" \
    "退 $(exit_of outside-paths inside)，命令$([[ -e "$scene/inside-ran" ]] && echo 跑了 || echo 没跑)；stderr 尾部：$(tail_of outside-paths inside | cut_characters 300)"
  # Ⓩ 词里的绝对路径怎么取（文件头 ③，与 task-server/task_server.py 的 find_outside_shared_path 同一个读法）：直接问 multi_host_outside_shared_path，共享根是一个空目录；
  #   每一例「<bash -c 后面那一个脚本词>|<要打的那个路径，空是一个都不取>」
  local expansion_shared="$work/expansion-shared" expansion_case expansion_word expansion_wanted expansion_got expansion_wrong=""
  mkdir -p "$expansion_shared"
  for expansion_case in 'cp "${X:?}/release/a" /w/x|/w/x' 'cd ./scripts && ls ../a/b|' 'ls $(pwd)/target|' 'cp /opt/x y|/opt/x' 'OUT=/opt/y true|/opt/y' \
      'echo a:/opt/z|' 'tool --out=/opt/v|/opt/v' 'cat </opt/u|/opt/u'; do
    expansion_word="${expansion_case%|*}"
    expansion_wanted="${expansion_case##*|}"
    expansion_got="$(NFS_SHARED_ROOTS="$expansion_shared" multi_host_outside_shared_path bash -c "$expansion_word")" || expansion_got="（python3 没跑成）"
    [[ "$expansion_got" == "$expansion_wanted" ]] || expansion_wrong+="「$expansion_word」取到「$expansion_got」（要「$expansion_wanted」）；"
  done
  expect "Ⓩ bash -c 脚本词照 shell 词切：跟在 }、) 后面的 /…（\${…}、\$(…) 的尾巴）、./x、../x 与 a:/x 里夹着的 / 不当绝对路径，词头的、= 后面的与挨着 < 的照取" \
    "$([[ -z "$expansion_wrong" ]]; echo $?)" "$expansion_wrong"
  # Ⓩ 参数值中间夹着的 /（文件头 ③）：--item 的值「…（接的是 /）」、bash -c 脚本里引号中的「（接的是 /）」不当绝对路径；同一条里共享根外的整个词照取。
  #   每一例「<要打的那个路径，空是一个都不取>」对一组写死的词
  local substring_got substring_wrong=""
  substring_got="$(NFS_SHARED_ROOTS="$expansion_shared" multi_host_outside_shared_path bash .claude/gate.d/crash-point-replay.sh --item 'crash-case-replay（接的是 /）' "$expansion_shared")" \
    || substring_got="（python3 没跑成）"
  [[ -z "$substring_got" ]] || substring_wrong+="--item 的值「crash-case-replay（接的是 /）」取到「$substring_got」（要一个都不取）；"
  substring_got="$(NFS_SHARED_ROOTS="$expansion_shared" multi_host_outside_shared_path bash -c 'echo "（接的是 /）" > /dev/null')" || substring_got="（python3 没跑成）"
  [[ -z "$substring_got" ]] || substring_wrong+="bash -c 引号里的「（接的是 /）」取到「$substring_got」（要一个都不取）；"
  substring_got="$(NFS_SHARED_ROOTS="$expansion_shared" multi_host_outside_shared_path --item 'x（接的是 /）' /opt/outside-word)" || substring_got="（python3 没跑成）"
  [[ "$substring_got" == /opt/outside-word ]] || substring_wrong+="--item 'x（接的是 /）' /opt/outside-word 取到「$substring_got」（要「/opt/outside-word」）；"
  expect "Ⓩ 参数值中间夹着的 /（--item 的值、bash -c 引号里的「（接的是 /）」）不当绝对路径，同一条里共享根外的整个词照取" \
    "$([[ -z "$substring_wrong" ]]; echo $?)" "$substring_wrong"

  # Ⓢ 共享根交给第二台：第二台那条命令记下它看到的两个共享根；工作根、多机配置放到共享根外各一趟；配置 BUILD_ROOT_BY_HOST 里第二台那一项落在共享根下一趟
  prepare_scene shared-roots
  scene="$work/shared-roots"
  run_in shared-roots seen --peer --memory-cap 256M --label roots-seen -- bash -c 'printf "work=%s data=%s\n" "${SINGLEFS_WORK_ROOT-没设}" "${SINGLEFS_DATA_ROOT-没设}" > "$0"' "$scene/roots-seen"
  local roots_expected
  roots_expected="work=$(realpath -m -- "$scene/work-root") data=$(realpath -m -- "$scene/data-root")"
  expect "Ⓢ --peer：第二台那条命令拿到本机的工作根与共享数据根（SINGLEFS_WORK_ROOT、SINGLEFS_DATA_ROOT 照本机的真路径）" \
    "$([[ "$(exit_of shared-roots seen)" == 0 && "$(cat "$scene/roots-seen" 2>/dev/null)" == "$roots_expected" ]]; echo $?)" \
    "退 $(exit_of shared-roots seen)；第二台那条命令看到的：$(cat "$scene/roots-seen" 2>/dev/null || echo 没记下)（应当是 $roots_expected）；stderr 尾部：$(tail_of shared-roots seen | cut_characters 300)"
  cp -- "$scene/multi-host.env" "$outside_shared/multi-host.env"
  scene_command_for shared-roots
  ( cd "$scene/repo" && exec "${scene_command[@]:0:${#scene_command[@]}-2}" SINGLEFS_WORK_ROOT="$outside_shared/work-root" "${scene_command[@]: -2}" \
      --peer --memory-cap 256M --label roots-work -- bash -c "touch $(printf '%q' "$scene/work-ran")" ) > "$scene/work.out" 2> "$scene/work.err" < /dev/null
  echo "$?" > "$scene/work.exit"
  ( cd "$scene/repo" && exec "${scene_command[@]:0:${#scene_command[@]}-2}" SINGLEFS_MULTI_HOST_CONFIG="$outside_shared/multi-host.env" "${scene_command[@]: -2}" \
      --peer --memory-cap 256M --label roots-configuration -- bash -c "touch $(printf '%q' "$scene/configuration-ran")" ) > "$scene/configuration.out" 2> "$scene/configuration.err" < /dev/null
  echo "$?" > "$scene/configuration.exit"
  expect "Ⓢ --peer：工作根、多机配置不在共享根下 ⇒ 各退 255、点名是哪一样，命令没跑" \
    "$([[ "$(exit_of shared-roots work)" == 255 && "$(exit_of shared-roots configuration)" == 255 && ! -e "$scene/work-ran" && ! -e "$scene/configuration-ran" ]] \
       && grep -q '工作根（SINGLEFS_WORK_ROOT）.* 不在多机配置 NFS_SHARED_ROOTS' "$scene/work.err" && grep -q '本机这一份多机配置 .* 不在多机配置 NFS_SHARED_ROOTS' "$scene/configuration.err"; echo $?)" \
    "工作根那一趟退 $(exit_of shared-roots work)，多机配置那一趟退 $(exit_of shared-roots configuration)；stderr 尾部：$(tail_of shared-roots work | cut_characters 200)|$(tail_of shared-roots configuration | cut_characters 200)"
  prepare_scene peer-directory-shared
  scene="$work/peer-directory-shared"
  mkdir -p "$scene/peer-home-on-shared-root"
  sed -i "s|^BUILD_ROOT_BY_HOST=.*|BUILD_ROOT_BY_HOST=127.0.0.1=$scene_this_build_root 192.0.2.61=$scene/peer-home-on-shared-root|" "$scene/multi-host.env"
  run_in peer-directory-shared run --peer --memory-cap 256M --label peer-directory-shared -- bash -c "touch $(printf '%q' "$scene/ran")"
  expect "Ⓢ --peer：多机配置 BUILD_ROOT_BY_HOST 里第二台那一项落在共享根下 ⇒ 退 255、说这一趟的本地目录不许落在共享根下，命令没跑" \
    "$([[ "$(exit_of peer-directory-shared run)" == 255 && ! -e "$scene/ran" ]] && grep -q 'BUILD_ROOT_BY_HOST 里第二台那一项落在共享根下' "$scene/run.err"; echo $?)" \
    "退 $(exit_of peer-directory-shared run)，命令$([[ -e "$scene/ran" ]] && echo 跑了 || echo 没跑)；stderr 尾部：$(tail_of peer-directory-shared run | cut_characters 300)"

  # Ⓦ 分段计时（文件头「分段计时」）：--peer 跑成的一趟里两步都有开始与结束两行
  local missing_steps="" step_name
  prepare_scene steps
  scene="$work/steps"
  run_in steps run --peer --memory-cap 256M --label steps-run -- bash -c 'exit 0'
  for step_name in peer-command remove-build; do
    if ! grep -qE "^MULTI_HOST_RUN_STEP step=$step_name event=begin elapsed_seconds=[0-9]+\.[0-9]\$" "$scene/run.err" \
       || ! grep -qE "^MULTI_HOST_RUN_STEP step=$step_name event=end elapsed_seconds=[0-9]+\.[0-9] seconds=[0-9]+\.[0-9] exit=0\$" "$scene/run.err"; then
      missing_steps+="${missing_steps:+、}$step_name"
    fi
  done
  expect "Ⓦ --peer 跑成：第二台那条命令、删这一趟的本地目录每一步都打了开始与结束两行，结束行的秒数是非负数" \
    "$([[ "$(exit_of steps run)" == 0 && -z "$missing_steps" ]]; echo $?)" \
    "退 $(exit_of steps run)；缺的：${missing_steps:-无}；stderr 尾部：$(tail_of steps run)"

  # 接回（文件头 ⑤「跑着时接回」）：第二台那条命令分几次打行，每打一行先空转一阵 CPU，再等那一行出现在本机驱动的标准错误里（至多 8 秒）才往下，记下等到了没有；
  #   末了打半行、空转一阵、过 3 秒补完，最后一截不以换行收尾。本机标准错误里那几行要逐行同序、不重不漏、接成整行；CPU 那一行每一轮一行、数不减
  prepare_scene follow
  scene="$work/follow"
  local follow_lines follow_expected="follow-line-1|follow-line-2|follow-line-3|follow-part-joined|follow-tail|" cpu_problems
  # 整条命令跑着的时候另有一个子 shell 一直空转：每一轮数线程那一刻要读得到状态 R 的线程（只靠下面几段 2 秒的空转，采样那一刻常落在 sleep 上、可跑线程数是 0）
  local follow_command='spin() { local spin_until=$((SECONDS + 2)); while (( SECONDS < spin_until )); do :; done; }
( while :; do :; done ) &
busy_process=$!
trap "kill $busy_process 2>/dev/null" EXIT
for line_number in 1 2 3; do
  printf "follow-line-%s\n" "$line_number"
  spin
  for (( wait_round = 0; wait_round < 80; wait_round++ )); do grep -qx "follow-line-$line_number" "$0" && break; sleep 0.1; done
  if grep -qx "follow-line-$line_number" "$0"; then echo "seen-$line_number" >> "$1"; else echo "missed-$line_number" >> "$1"; fi
done
printf "follow-part-"
spin
sleep 3
printf "joined\n"
printf "follow-tail"'
  run_in follow run --peer --memory-cap 256M --label follow -- bash -c "$follow_command" "$scene/run.err" "$scene/seen"
  follow_lines="$(grep '^follow-' "$scene/run.err" 2>/dev/null | tr '\n' '|')"
  expect "接回 第二台那条命令打的每一行在它还跑着的时候就出现在本机标准错误里" \
    "$([[ "$(exit_of follow run)" == 0 && "$(tr '\n' ' ' < "$scene/seen" 2>/dev/null)" == "seen-1 seen-2 seen-3 " ]]; echo $?)" \
    "退 $(exit_of follow run)；等到了没有：$(tr '\n' ' ' < "$scene/seen" 2>/dev/null)；stderr 尾部：$(tail_of follow run)"
  expect "接回 本机标准错误里第二台那条命令打的行逐行同序、不重不漏，半行与不以换行收尾的末行都接成整行" \
    "$([[ "$(exit_of follow run)" == 0 && "$follow_lines" == "$follow_expected" ]]; echo $?)" \
    "退 $(exit_of follow run)；读到的：${follow_lines:-（一行都没有）}；要的：$follow_expected"
  cpu_problems="$(awk '
    /MULTI_HOST_RUN_PEER_CPU poll=/ {
      if ($0 !~ /^MULTI_HOST_RUN_PEER_CPU poll=[0-9]+ usage_usec=[0-9]+ read=(yes|no)$/) { print "格式不对：" $0; next }
      lines++
      split($2, poll_field, "="); split($3, usage_field, "=")
      if (poll_field[2] + 0 != lines) print "轮数断号：读到 poll=" poll_field[2] "，照行数该是 " lines
      if (usage_field[2] + 0 < last_usage) print "usage_usec 减了：" last_usage " → " usage_field[2]
      last_usage = usage_field[2] + 0
      if ($4 == "read=yes") read_rounds++
    }
    END {
      if (lines < 3) print "只有 " (lines + 0) " 行"
      if (last_usage < 1000000) print "末行 usage_usec=" (last_usage + 0) "，不到 1000000"
      if (read_rounds == 0) print "一轮都没读到（read=yes 一行都没有）"
    }' "$scene/run.err" 2>/dev/null | tr '\n' ' ')"
  expect "接回 每一轮一行 MULTI_HOST_RUN_PEER_CPU poll=<从 1 起连着的号> usage_usec=<数> read=<yes|no>，usage_usec 一轮比一轮不减、末行至少 1000000、有一轮读到了" \
    "$([[ "$(exit_of follow run)" == 0 && -z "$cpu_problems" ]]; echo $?)" \
    "${cpu_problems:-无}；stderr 尾部：$(tail_of follow run)"
  # 此刻内存（文件头 ⑤「此刻内存」）：同一趟里每一轮 CPU 行的下一行就是 MULTI_HOST_RUN_PEER_MEMORY，整行对上固定格式、轮数与那一行 CPU 相同，至少一轮 read=yes 且 current_bytes 大于 0
  local memory_problems
  memory_problems="$(awk '
    /MULTI_HOST_RUN_PEER_CPU poll=/ {
      if (waiting) print "第 " cpu_line " 行的 CPU 后面没跟 MEMORY"
      waiting = 1; cpu_line = NR
      split($0, poll_parts, "poll="); split(poll_parts[2], poll_number, " "); cpu_poll = poll_number[1]
    }
    /MULTI_HOST_RUN_PEER_MEMORY poll=/ {
      if ($0 !~ /^MULTI_HOST_RUN_PEER_MEMORY poll=[0-9]+ current_bytes=[0-9]+ peak_bytes=[0-9]+ read=(yes|no)$/) { print "格式不对：" $0; waiting = 0; next }
      lines++
      split($2, poll_field, "="); split($3, current_field, "="); split($5, read_field, "=")
      if (!waiting || NR != cpu_line + 1) print "第 " NR " 行的 MEMORY 前一行不是 CPU"
      else if (poll_field[2] != cpu_poll) print "轮数对不上：CPU 是 poll=" cpu_poll "，MEMORY 是 poll=" poll_field[2]
      waiting = 0
      if (read_field[2] == "yes" && current_field[2] + 0 > 0) good_rounds++
    }
    END {
      if (waiting) print "第 " cpu_line " 行的 CPU 后面没跟 MEMORY"
      if (lines == 0) print "只有 0 行 MEMORY"
      else if (good_rounds == 0) print "一轮都没有 read=yes 且 current_bytes 大于 0"
    }' "$scene/run.err" 2>/dev/null | tr '\n' ' ')"
  expect "接回 每一轮一行 MULTI_HOST_RUN_PEER_MEMORY poll=<与 CPU 那一行同一个轮数> current_bytes=<数> peak_bytes=<数> read=<yes|no>，紧跟每行 CPU 之后、至少一轮 read=yes 且 current_bytes 大于 0" \
    "$([[ "$(exit_of follow run)" == 0 && -z "$memory_problems" ]]; echo $?)" \
    "${memory_problems:-无}：$(grep -m 3 'MULTI_HOST_RUN_PEER_MEMORY poll=' "$scene/run.err" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of follow run)"
  # 等 IO（文件头 ⑤「等 IO」）：同一趟里每一轮 CPU 行之后、下一轮 CPU 行之前恰好一行 MULTI_HOST_RUN_PEER_IO_WAIT，整行对上固定格式、轮数与那一行 CPU 相同，至少一轮 read=yes
  #   （弄坏开关 peer-io-wait-unreported 下一行都没有）
  local io_wait_problems
  io_wait_problems="$(awk '
    /MULTI_HOST_RUN_PEER_CPU poll=/ {
      if (waiting) print "第 " cpu_line " 行的 CPU 那一轮没有 IO_WAIT"
      waiting = 1; cpu_line = NR
      split($0, poll_parts, "poll="); split(poll_parts[2], poll_number, " "); cpu_poll = poll_number[1]
    }
    /MULTI_HOST_RUN_PEER_IO_WAIT poll=/ {
      if ($0 !~ /^MULTI_HOST_RUN_PEER_IO_WAIT poll=[0-9]+ blocked=[0-9]+ read=(yes|no)$/) { print "格式不对：" $0; waiting = 0; next }
      lines++
      split($2, poll_field, "="); split($4, read_field, "=")
      if (!waiting) print "第 " NR " 行的 IO_WAIT 前面没有这一轮的 CPU"
      else if (poll_field[2] != cpu_poll) print "轮数对不上：CPU 是 poll=" cpu_poll "，IO_WAIT 是 poll=" poll_field[2]
      waiting = 0
      if (read_field[2] == "yes") read_rounds++
    }
    END {
      if (waiting) print "第 " cpu_line " 行的 CPU 那一轮没有 IO_WAIT"
      if (lines == 0) print "只有 0 行 IO_WAIT"
      else if (read_rounds == 0) print "一轮都没有 read=yes"
    }' "$scene/run.err" 2>/dev/null | tr '\n' ' ')"
  expect "等 IO 每一轮一行 MULTI_HOST_RUN_PEER_IO_WAIT poll=<与这一轮 CPU 那一行同一个轮数> blocked=<数> read=<yes|no>，在这一轮 CPU 之后、下一轮 CPU 之前，至少一轮 read=yes" \
    "$([[ "$(exit_of follow run)" == 0 && -z "$io_wait_problems" ]]; echo $?)" \
    "${io_wait_problems:-无}：$(grep -m 3 'MULTI_HOST_RUN_PEER_IO_WAIT poll=' "$scene/run.err" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of follow run)"
  # 线程（文件头 ⑤「线程」）：同一趟里每一轮 CPU 行之后、下一轮 CPU 行之前恰好一行 MULTI_HOST_RUN_PEER_THREADS，整行对上固定格式、轮数与那一行 CPU 相同；
  #   命令空转了几段（spin），至少一轮 read=yes 且 runnable、total 都大于 0（弄坏开关 peer-threads-unreported 下一行都没有，peer-threads-uncounted 下 runnable 都是 0）
  local thread_problems
  thread_problems="$(awk '
    /MULTI_HOST_RUN_PEER_CPU poll=/ {
      if (waiting) print "第 " cpu_line " 行的 CPU 那一轮没有 THREADS"
      waiting = 1; cpu_line = NR
      split($0, poll_parts, "poll="); split(poll_parts[2], poll_number, " "); cpu_poll = poll_number[1]
    }
    /MULTI_HOST_RUN_PEER_THREADS poll=/ {
      if ($0 !~ /^MULTI_HOST_RUN_PEER_THREADS poll=[0-9]+ runnable=[0-9]+ total=[0-9]+ top=[0-9]+:[A-Za-z0-9._-]+ read=(yes|no)$/) { print "格式不对：" $0; waiting = 0; next }
      lines++
      split($2, poll_field, "="); split($3, runnable_field, "="); split($4, total_field, "="); split($6, read_field, "=")
      if (!waiting) print "第 " NR " 行的 THREADS 前面没有这一轮的 CPU"
      else if (poll_field[2] != cpu_poll) print "轮数对不上：CPU 是 poll=" cpu_poll "，THREADS 是 poll=" poll_field[2]
      waiting = 0
      if (read_field[2] == "yes" && runnable_field[2] + 0 > 0 && total_field[2] + 0 > 0) counted_rounds++
    }
    END {
      if (waiting) print "第 " cpu_line " 行的 CPU 那一轮没有 THREADS"
      if (lines == 0) print "只有 0 行 THREADS"
      else if (counted_rounds == 0) print "一轮都没有 read=yes 且 runnable、total 都大于 0"
    }' "$scene/run.err" 2>/dev/null | tr '\n' ' ')"
  expect "线程 每一轮一行 MULTI_HOST_RUN_PEER_THREADS poll=<与这一轮 CPU 那一行同一个轮数> runnable=<数> total=<数> top=<数>:<名> read=<yes|no>，在这一轮 CPU 之后、下一轮 CPU 之前，至少一轮 read=yes 且 runnable、total 都大于 0" \
    "$([[ "$(exit_of follow run)" == 0 && -z "$thread_problems" ]]; echo $?)" \
    "${thread_problems:-无}：$(grep -m 3 'MULTI_HOST_RUN_PEER_THREADS poll=' "$scene/run.err" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of follow run)"
  # 显存读不到（文件头 ⑤「显存」）：接回那一趟不要卡 ⇒ 每一轮 CPU 之后照样有一行 MULTI_HOST_RUN_PEER_GPU used_mib=0 read=no，行数与 CPU 行一样多
  local plain_gpu_problems
  plain_gpu_problems="$(awk '
    /^MULTI_HOST_RUN_PEER_CPU poll=/ { cpu_lines++ }
    /^MULTI_HOST_RUN_PEER_GPU poll=/ { gpu_lines++; if ($0 !~ /^MULTI_HOST_RUN_PEER_GPU poll=[0-9]+ used_mib=0 read=no utilization=none on_cards=(empty|unread)$/) print "不是 used_mib=0 read=no utilization=none on_cards=empty 或 unread：" $0 }
    END { if (gpu_lines + 0 < 1 || gpu_lines != cpu_lines) print "GPU 行 " (gpu_lines + 0) " 行，CPU 行 " (cpu_lines + 0) " 行" }' "$scene/run.err" 2>/dev/null | tr '\n' ' ')"
  expect "显存 不要卡的一趟每一轮照打 MULTI_HOST_RUN_PEER_GPU …used_mib=0 read=no，每一轮 CPU 之后都有、不漏行" \
    "$([[ "$(exit_of follow run)" == 0 && -z "$plain_gpu_problems" ]]; echo $?)" \
    "${plain_gpu_problems:-无}：$(grep -m 3 'MULTI_HOST_RUN_PEER_GPU poll=' "$scene/run.err" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of follow run)"
  # 峰值（文件头 ⑤「峰值」）：上面那一趟经真的内存包装跑了十来秒，结束时恰好一行 MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=<正整数>、在最后一行 CPU 之后；
  #   再跑一趟、每一轮读的 memory.stat 与退出时读的文件名都换成不在的（MULTI_HOST_RUN_SELFTEST_CURRENT_FILE、MULTI_HOST_RUN_SELFTEST_EXIT_PEAK_FILE，只在自证里认）、
  #   空转 3 秒（有一轮 read=yes）⇒ 不打那一行；再跑一趟 0.1 秒就退的、只把每一轮读的换成不在的 ⇒ 照那一层退出时读的打恰好一行
  local peak_lines peak_problems unread_peak_lines unread_read_rounds
  peak_lines="$(grep '^MULTI_HOST_RUN_PEER_MEMORY_PEAK' "$scene/run.err" 2>/dev/null)"
  peak_problems="$(awk '/^MULTI_HOST_RUN_PEER_CPU poll=/ { last_cpu = NR } /^MULTI_HOST_RUN_PEER_MEMORY_PEAK/ { peaks++; peak_line = NR; peak_text = $0 }
    END {
      if (peaks != 1) print "峰值行 " (peaks + 0) " 行"
      else if (peak_text !~ /^MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=[1-9][0-9]*$/) print "格式不对：" peak_text
      else if (peak_line < last_cpu) print "峰值行在第 " peak_line " 行，最后一行 CPU 在第 " last_cpu " 行"
    }' "$scene/run.err" 2>/dev/null)"
  expect "峰值 第二台那条命令结束时打恰好一行 MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=<正整数>（那一层内存包装 cgroup 进程内存见过的最大），在最后一行 MULTI_HOST_RUN_PEER_CPU 之后" \
    "$([[ "$(exit_of follow run)" == 0 && -z "$peak_problems" ]]; echo $?)" \
    "${peak_problems:-无}：$(tr '\n' '|' <<<"$peak_lines")；stderr 尾部：$(tail_of follow run)"
  scene_command_for follow
  ( cd "$scene/repo" && exec "${scene_command[@]:0:${#scene_command[@]}-2}" \
      MULTI_HOST_RUN_SELFTEST_CURRENT_FILE=memory.stat-selftest-unreadable MULTI_HOST_RUN_SELFTEST_BLOCKED_STATES=S \
      MULTI_HOST_RUN_SELFTEST_EXIT_PEAK_FILE=exit-memory-peak-selftest-unreadable "${scene_command[@]: -2}" \
      --peer --memory-cap 256M --label follow-peak-unread -- bash -c 'spin_until=$((SECONDS + 3)); while (( SECONDS < spin_until )); do :; done' ) \
    > "$scene/peak-unread.out" 2> "$scene/peak-unread.err" < /dev/null
  echo "$?" > "$scene/peak-unread.exit"
  unread_peak_lines="$(grep -c '^MULTI_HOST_RUN_PEER_MEMORY_PEAK' "$scene/peak-unread.err" 2>/dev/null)" || true
  unread_read_rounds="$(grep -c '^MULTI_HOST_RUN_PEER_CPU poll=[0-9]* usage_usec=[0-9]* read=yes$' "$scene/peak-unread.err" 2>/dev/null)" || true
  expect "峰值 每一轮读不到 memory.stat、退出时读不到 memory.peak（两处读的文件名都换成不在的）⇒ 退 0、有一轮 read=yes、不打 MULTI_HOST_RUN_PEER_MEMORY_PEAK 那一行" \
    "$([[ "$(exit_of follow peak-unread)" == 0 && "${unread_read_rounds:-0}" -ge 1 && "${unread_peak_lines:-0}" == 0 ]]; echo $?)" \
    "退 $(exit_of follow peak-unread)，read=yes 的轮 ${unread_read_rounds:-0} 轮，峰值行 ${unread_peak_lines:-0} 行：$(grep '^MULTI_HOST_RUN_PEER_MEMORY_PEAK' "$scene/peak-unread.err" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of follow peak-unread)"
  # 此刻内存读不到（文件头 ⑤「此刻内存」）：同一趟里每一轮读 memory.stat 的文件名换成不在的 ⇒ 每一轮 CPU 之后照样有一行 MEMORY，
  #   两项写 0、read=no，行数与 CPU 行一样多（弄坏开关 peer-memory-unread-skipped 下漏行）
  local unread_memory_problems
  unread_memory_problems="$(awk '
    /MULTI_HOST_RUN_PEER_CPU poll=/ { cpu_lines++ }
    /MULTI_HOST_RUN_PEER_MEMORY poll=/ {
      memory_lines++
      if ($0 !~ /^MULTI_HOST_RUN_PEER_MEMORY poll=[0-9]+ current_bytes=0 peak_bytes=0 read=no$/) print "不是 current_bytes=0 peak_bytes=0 read=no：" $0
    }
    END { if (memory_lines + 0 < 1 || memory_lines != cpu_lines) print "MEMORY 行 " (memory_lines + 0) " 行，CPU 行 " (cpu_lines + 0) " 行" }' "$scene/peak-unread.err" 2>/dev/null | tr '\n' ' ')"
  expect "接回 读不到 memory.stat 的每一轮照打 MULTI_HOST_RUN_PEER_MEMORY …current_bytes=0 peak_bytes=0 read=no，每一轮 CPU 之后都有、不漏行" \
    "$([[ "$(exit_of follow peak-unread)" == 0 && -z "$unread_memory_problems" ]]; echo $?)" \
    "${unread_memory_problems:-无}：$(grep -m 3 'MULTI_HOST_RUN_PEER_MEMORY poll=' "$scene/peak-unread.err" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of follow peak-unread)"
  # 等 IO 数得到（文件头 ⑤「等 IO」）：同一趟数的状态取 S（MULTI_HOST_RUN_SELFTEST_BLOCKED_STATES=S）：空转的命令底下进程号那个 bash 在睡 ⇒ 至少一轮 read=yes 且 blocked 大于 0
  #   （弄坏开关 peer-io-wait-uncounted 下每一轮都写 blocked=0，peer-io-wait-unreported 下一行都没有）
  local counted_rounds
  counted_rounds="$(grep -c '^MULTI_HOST_RUN_PEER_IO_WAIT poll=[0-9]* blocked=[1-9][0-9]* read=yes$' "$scene/peak-unread.err" 2>/dev/null)" || true
  expect "等 IO 数的状态换成 S（在睡）⇒ 至少一轮 read=yes 且 blocked 大于 0" \
    "$([[ "$(exit_of follow peak-unread)" == 0 && "${counted_rounds:-0}" -ge 1 ]]; echo $?)" \
    "退 $(exit_of follow peak-unread)，blocked 大于 0 且 read=yes 的轮 ${counted_rounds:-0} 轮：$(grep -m 3 'MULTI_HOST_RUN_PEER_IO_WAIT poll=' "$scene/peak-unread.err" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of follow peak-unread)"
  # 命令跑 0.1 秒就退（比一轮短），每一轮读的换成不在的：峰值照第二台那一层内存包装退出时读的那一个打（弄坏开关 peak-read-only-by-sampling 下一行都不打）
  local short_peak_lines
  ( cd "$scene/repo" && exec "${scene_command[@]:0:${#scene_command[@]}-2}" MULTI_HOST_RUN_SELFTEST_CURRENT_FILE=memory.stat-selftest-unreadable "${scene_command[@]: -2}" \
      --peer --memory-cap 256M --label follow-peak-short -- sleep 0.1 ) > "$scene/peak-short.out" 2> "$scene/peak-short.err" < /dev/null
  echo "$?" > "$scene/peak-short.exit"
  short_peak_lines="$(grep '^MULTI_HOST_RUN_PEER_MEMORY_PEAK' "$scene/peak-short.err" 2>/dev/null)"
  expect "峰值 第二台那条命令跑 0.1 秒就退、每一轮都读不到 memory.stat ⇒ 退 0、照那一层内存包装退出时读的打恰好一行 MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=<正整数>" \
    "$([[ "$(exit_of follow peak-short)" == 0 && "$short_peak_lines" =~ ^MULTI_HOST_RUN_PEER_MEMORY_PEAK\ bytes=[1-9][0-9]*$ ]]; echo $?)" \
    "退 $(exit_of follow peak-short)，峰值行「$(tr '\n' '|' <<<"$short_peak_lines")」；stderr 尾部：$(tail_of follow peak-short)"
  # 页缓存撑满上限（文件头 ⑤「峰值」）：每一轮读进程内存的目录换成假 cgroup（MULTI_HOST_RUN_SELFTEST_MEMORY_DIRECTORY，只在自证里认）：
  #   memory.stat 里 anon 1G、file 40G、kernel 650M、shmem 0，memory.current 与 memory.peak 是三项之和（顶到上限的样子），空转 3 秒
  #   ⇒ 退 0、read=yes 的 MEMORY 行至少一行且每一行 current_bytes 与 peak_bytes 都是 anon 的 1G、峰值行恰好一行 bytes=1G（弄坏开关 peer-memory-with-page-cache 下照 memory.current、memory.peak 打）
  local page_cache_directory="$scene/page-cache-memory" page_cache_anon=1073741824 page_cache_file=42949672960 page_cache_kernel=681574400 page_cache_problems
  mkdir -p "$page_cache_directory"
  printf 'anon %s\nfile %s\nkernel %s\nshmem 0\n' "$page_cache_anon" "$page_cache_file" "$page_cache_kernel" > "$page_cache_directory/memory.stat"
  echo "$(( page_cache_anon + page_cache_file + page_cache_kernel ))" > "$page_cache_directory/memory.current"
  echo "$(( page_cache_anon + page_cache_file + page_cache_kernel ))" > "$page_cache_directory/memory.peak"
  ( cd "$scene/repo" && exec "${scene_command[@]:0:${#scene_command[@]}-2}" MULTI_HOST_RUN_SELFTEST_MEMORY_DIRECTORY="$page_cache_directory" "${scene_command[@]: -2}" \
      --peer --memory-cap 256M --label follow-page-cache -- bash -c 'spin_until=$((SECONDS + 3)); while (( SECONDS < spin_until )); do :; done' ) \
    > "$scene/page-cache.out" 2> "$scene/page-cache.err" < /dev/null
  echo "$?" > "$scene/page-cache.exit"
  page_cache_problems="$(awk -v wanted="$page_cache_anon" '
    /^MULTI_HOST_RUN_PEER_MEMORY poll=.* read=yes$/ {
      read_lines++
      split($3, current_field, "="); split($4, peak_field, "=")
      if (current_field[2] != wanted || peak_field[2] != wanted) print "不是 current_bytes=" wanted " peak_bytes=" wanted "：" $0
    }
    /^MULTI_HOST_RUN_PEER_MEMORY_PEAK/ { peaks++; peak_text = $0 }
    END {
      if (read_lines + 0 < 1) print "read=yes 的 MEMORY 行 0 行"
      if (peaks != 1 || peak_text != "MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=" wanted) print "峰值行 " (peaks + 0) " 行：" peak_text
    }' "$scene/page-cache.err" 2>/dev/null | tr '\n' ' ')"
  expect "峰值 页缓存撑满上限（假 cgroup：anon 1G、file 40G、memory.peak 顶到三项之和）⇒ 每一轮 current_bytes 与 peak_bytes 都是 anon 的 1G、峰值行 bytes=1G" \
    "$([[ "$(exit_of follow page-cache)" == 0 && -z "$page_cache_problems" ]]; echo $?)" \
    "退 $(exit_of follow page-cache)；${page_cache_problems:-无}：$(grep -m 3 '^MULTI_HOST_RUN_PEER_MEMORY' "$scene/page-cache.err" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of follow page-cache)"
  # 内存明细（文件头 ⑤「内存最高那一刻的明细」）：页缓存那一趟的假 cgroup 每一轮 CPU 之后一行 MULTI_HOST_RUN_PEER_MEMORY_STAT，读数就是它写的；接回那一趟（真的 scope）至少一轮 read=yes、current_bytes 大于 0
  local memory_stat_problems
  memory_stat_problems="$(awk -v current="$(( page_cache_anon + page_cache_file + page_cache_kernel ))" -v anon="$page_cache_anon" -v file="$page_cache_file" '
    /^MULTI_HOST_RUN_PEER_CPU poll=/ { cpu_lines++ }
    /^MULTI_HOST_RUN_PEER_MEMORY_STAT poll=/ {
      stat_lines++
      if ($0 !~ /^MULTI_HOST_RUN_PEER_MEMORY_STAT poll=[0-9]+ current_bytes=[0-9]+ anon=[0-9]+ file=[0-9]+ file_dirty=[0-9]+ file_writeback=[0-9]+ shmem=[0-9]+ read=(yes|no)$/) { print "格式不对：" $0; next }
      if ($NF == "read=yes") { read_lines++; if ($3 != "current_bytes=" current || $4 != "anon=" anon || $5 != "file=" file) print "读数不对：" $0 }
    }
    END { if (stat_lines + 0 < 1 || stat_lines != cpu_lines) print "MEMORY_STAT 行 " (stat_lines + 0) " 行，CPU 行 " (cpu_lines + 0) " 行"; if (read_lines + 0 < 1) print "read=yes 的 0 行" }' "$scene/page-cache.err" 2>/dev/null | tr '\n' ' ')"
  if ! grep -Eq '^MULTI_HOST_RUN_PEER_MEMORY_STAT poll=[0-9]+ current_bytes=[1-9][0-9]* .* read=yes$' "$scene/run.err" 2>/dev/null; then memory_stat_problems+="真的 scope 那一趟没有 read=yes 的行 "; fi
  expect "内存明细 每一轮一行 MULTI_HOST_RUN_PEER_MEMORY_STAT，假 cgroup 那一趟读数对得上，真的 scope 那一趟至少一轮 read=yes" \
    "$([[ "$(exit_of follow page-cache)" == 0 && -z "$memory_stat_problems" ]]; echo $?)" \
    "${memory_stat_problems:-无}：$(grep -m 3 'MULTI_HOST_RUN_PEER_MEMORY_STAT poll=' "$scene/page-cache.err" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of follow page-cache)"
  # 短命令按轮只读到外壳（文件头 ⑤「峰值」；任务 10045 这一族在第二台每次记 18 MiB、本机量到 556 MiB）：每一轮读的目录换成假 cgroup（anon 18 MiB），命令真用 128 MiB、再睡 2 秒
  #   ⇒ 峰值行照那一层内存包装退出时交的、不小于 100 MiB，不是外壳那个数（弄坏开关 peer-peak-only-when-unsampled、peak-read-only-by-sampling 下是 18 MiB）；
  #   再跑一趟退出时读的文件名换成不在的 ⇒ 按轮读到了也不打那一行（没量到，不进定额度）
  local shell_only_directory="$scene/shell-only-memory" shell_only_bytes=18874368 burst_peak_line burst_peak_value unread_exit_lines
  mkdir -p "$shell_only_directory"
  printf 'anon %s\nfile 0\nshmem 0\n' "$shell_only_bytes" > "$shell_only_directory/memory.stat"
  echo "$shell_only_bytes" > "$shell_only_directory/memory.current"
  ( cd "$scene/repo" && exec "${scene_command[@]:0:${#scene_command[@]}-2}" MULTI_HOST_RUN_SELFTEST_MEMORY_DIRECTORY="$shell_only_directory" "${scene_command[@]: -2}" \
      --peer --memory-cap 256M --label follow-peak-burst -- python3 -c 'import time; held = b"x" * (128 * 1024 * 1024); time.sleep(2); print(len(held))' ) \
    > "$scene/peak-burst.out" 2> "$scene/peak-burst.err" < /dev/null
  echo "$?" > "$scene/peak-burst.exit"
  burst_peak_line="$(grep '^MULTI_HOST_RUN_PEER_MEMORY_PEAK' "$scene/peak-burst.err" 2>/dev/null)"
  burst_peak_value="${burst_peak_line##*bytes=}"
  expect "峰值 短命令按轮只读到外壳的 18 MiB、真用量 128 MiB ⇒ 峰值行不是外壳那个数（不小于 100 MiB）" \
    "$([[ "$(exit_of follow peak-burst)" == 0 && "$burst_peak_value" =~ ^[1-9][0-9]*$ ]] && (( burst_peak_value >= 104857600 )); echo $?)" \
    "退 $(exit_of follow peak-burst)，峰值行「$(tr '\n' '|' <<<"$burst_peak_line")」；stderr 尾部：$(tail_of follow peak-burst)"
  ( cd "$scene/repo" && exec "${scene_command[@]:0:${#scene_command[@]}-2}" MULTI_HOST_RUN_SELFTEST_MEMORY_DIRECTORY="$shell_only_directory" MULTI_HOST_RUN_SELFTEST_EXIT_PEAK_FILE=exit-memory-peak-selftest-unreadable "${scene_command[@]: -2}" \
      --peer --memory-cap 256M --label follow-peak-exit-unread -- sleep 3 ) > "$scene/peak-exit-unread.out" 2> "$scene/peak-exit-unread.err" < /dev/null
  echo "$?" > "$scene/peak-exit-unread.exit"
  unread_exit_lines="$(grep -c '^MULTI_HOST_RUN_PEER_MEMORY_PEAK' "$scene/peak-exit-unread.err" 2>/dev/null)" || true
  expect "峰值 按轮读到了、退出时那一个读不到 ⇒ 不打 MULTI_HOST_RUN_PEER_MEMORY_PEAK 那一行（没量到）" \
    "$([[ "$(exit_of follow peak-exit-unread)" == 0 && "${unread_exit_lines:-0}" == 0 ]] && grep -q '^MULTI_HOST_RUN_PEER_MEMORY poll=[0-9]* current_bytes=[1-9][0-9]* .*read=yes$' "$scene/peak-exit-unread.err"; echo $?)" \
    "退 $(exit_of follow peak-exit-unread)，峰值行 ${unread_exit_lines:-0} 行：$(grep '^MULTI_HOST_RUN_PEER_MEMORY_PEAK' "$scene/peak-exit-unread.err" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of follow peak-exit-unread)"
  # 显存（文件头 ⑤「显存」）：PATH 最前面放假 nvidia-smi，打命令记下的进程号用 2048 MiB、1 号进程用 9999 MiB（不归这一趟）；这一趟要卡（环境里有 GPU_CARDS_BY_PRIORITY）
  #   ⇒ 至少一轮 used_mib=2048 read=yes，没有一轮不是 2048 的 read=yes 行（弄坏开关 peer-gpu-family-unfiltered 下算成 12047，peer-gpu-unreported 下一行都没有）；
  #   分到序号 0、2 两张，假 nvidia-smi 只认 -i 0,2、一张 40 一张 20 ⇒ 至少一轮 utilization=30（弄坏开关 peer-gpu-utilization-unread 下一直是 none，peer-gpu-unreported 下一行都没有）；
  #   命令记下的进程号在卡 2 上（uuid GPU-selftest-2）、1 号进程在卡 0 上 ⇒ 至少一轮 on_cards=2:<进程号>:2048，没有一轮列 1 号进程（弄坏开关 peer-card-locations-unreported 下一直 unread，
  #   peer-card-locations-unfiltered 下列 1 号进程，peer-gpu-unreported 下一行都没有）；不要卡的一趟同样（弄坏开关 peer-card-locations-plain-skipped 下一直 unread）
  local fake_smi_directory="$scene/fake-smi" gpu_pid_file="$scene/gpu-command.pid" gpu_problems
  mkdir -p "$fake_smi_directory"
  printf '%s\n' '#!/usr/bin/env bash' \
    '# multi-host-run.sh 自证的假 nvidia-smi：命令记下的进程号 2048 MiB，1 号进程 9999 MiB' \
    'case "$*" in' \
    '  *--query-gpu=utilization.gpu*) if [[ " $* " == *" -i 0,2 "* ]]; then printf "40\n20\n"; exit 0; fi; exit 9 ;;' \
    '  *--query-gpu=index,uuid*) printf "0, GPU-selftest-0\n2, GPU-selftest-2\n"; exit 0 ;;' \
    'esac' \
    "if [[ -s $(printf '%q' "$gpu_pid_file") ]]; then echo \"GPU-selftest-2, \$(cat $(printf '%q' "$gpu_pid_file")), 2048\"; fi" \
    'echo "GPU-selftest-0, 1, 9999"' > "$fake_smi_directory/nvidia-smi"
  chmod +x "$fake_smi_directory/nvidia-smi"
  ( cd "$scene/repo" && exec "${scene_command[@]:0:${#scene_command[@]}-2}" PATH="$fake_smi_directory:$fake_rustup_directory:$PATH" "GPU_CARDS_BY_PRIORITY=192.0.2.61:0:1G 192.0.2.61:2:1G" "${scene_command[@]: -2}" \
      --peer --memory-cap 256M --label follow-gpu -- bash -c 'echo "$$" > "$0"; sleep 3' "$gpu_pid_file" ) > "$scene/gpu.out" 2> "$scene/gpu.err" < /dev/null
  echo "$?" > "$scene/gpu.exit"
  gpu_problems="$(awk '
    /^MULTI_HOST_RUN_PEER_GPU poll=[0-9]+ used_mib=[0-9]+ read=yes utilization=([0-9]+|none) on_cards=[^ ]+$/ { read_lines++; if ($3 != "used_mib=2048" && $3 != "used_mib=0") print "不是 used_mib=2048：" $0; if ($3 == "used_mib=2048") counted++ }
    END { if (counted + 0 < 1) print "至少一轮 used_mib=2048 read=yes：实际 " (counted + 0) " 轮" }' "$scene/gpu.err" 2>/dev/null | tr '\n' ' ')"
  expect "显存 要卡的一趟每一轮一行 MULTI_HOST_RUN_PEER_GPU，只算这一趟进程树里的进程" \
    "$([[ "$(exit_of follow gpu)" == 0 && -z "$gpu_problems" ]]; echo $?)" \
    "退 $(exit_of follow gpu)；${gpu_problems:-无}：$(grep -m 3 'MULTI_HOST_RUN_PEER_GPU poll=' "$scene/gpu.err" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of follow gpu)"
  local utilization_problems
  utilization_problems="$(awk '
    /^MULTI_HOST_RUN_PEER_GPU poll=/ { if ($5 == "utilization=30") counted++; else if ($5 != "utilization=none") print "不是 utilization=30：" $0 }
    END { if (counted + 0 < 1) print "至少一轮 utilization=30：实际 " (counted + 0) " 轮" }' "$scene/gpu.err" 2>/dev/null | tr '\n' ' ')"
  expect "利用率 要卡的一趟每一轮 MULTI_HOST_RUN_PEER_GPU 带分到的那两张逐张平均的 utilization=" \
    "$([[ "$(exit_of follow gpu)" == 0 && -z "$utilization_problems" ]]; echo $?)" \
    "退 $(exit_of follow gpu)；${utilization_problems:-无}：$(grep -m 3 'MULTI_HOST_RUN_PEER_GPU poll=' "$scene/gpu.err" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of follow gpu)"
  card_location_problems() { # 读 $1 那一份 stderr：on_cards= 列的只有卡 2 上 2048 MiB 的那一个进程，至少一轮列了它（$2 给了的另要 used_mib=0 read=no）
    awk -v plain="${2:-}" '
      /^MULTI_HOST_RUN_PEER_GPU poll=/ {
        if (plain != "" && ($3 != "used_mib=0" || $4 != "read=no")) print "不是 used_mib=0 read=no：" $0
        if ($6 ~ /^on_cards=2:[0-9]+:2048$/) counted++
        else if ($6 != "on_cards=empty" && $6 != "on_cards=unread") print "不是这一趟的：" $0
      }
      END { if (counted + 0 < 1) print "至少一轮 on_cards=2：实际 " (counted + 0) " 轮" }' "$1" 2>/dev/null | tr '\n' ' '
  }
  local on_cards_problems plain_on_cards_problems
  on_cards_problems="$(card_location_problems "$scene/gpu.err")"
  expect "实际在哪张卡 要卡的一趟 on_cards= 只列这一趟进程树里的进程，序号照 uuid 对回来" \
    "$([[ "$(exit_of follow gpu)" == 0 && -z "$on_cards_problems" ]]; echo $?)" \
    "退 $(exit_of follow gpu)；${on_cards_problems:-无}：$(grep -m 3 'MULTI_HOST_RUN_PEER_GPU poll=' "$scene/gpu.err" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of follow gpu)"
  rm -f -- "$gpu_pid_file"
  ( cd "$scene/repo" && exec "${scene_command[@]:0:${#scene_command[@]}-2}" PATH="$fake_smi_directory:$fake_rustup_directory:$PATH" "${scene_command[@]: -2}" \
      --peer --memory-cap 256M --label follow-plain-card -- bash -c 'echo "$$" > "$0"; sleep 3' "$gpu_pid_file" ) > "$scene/plain-card.out" 2> "$scene/plain-card.err" < /dev/null
  echo "$?" > "$scene/plain-card.exit"
  plain_on_cards_problems="$(card_location_problems "$scene/plain-card.err" plain)"
  expect "实际在哪张卡 不要卡的一趟照读 on_cards=，used_mib=0 read=no 不变" \
    "$([[ "$(exit_of follow plain-card)" == 0 && -z "$plain_on_cards_problems" ]]; echo $?)" \
    "退 $(exit_of follow plain-card)；${plain_on_cards_problems:-无}：$(grep -m 3 'MULTI_HOST_RUN_PEER_GPU poll=' "$scene/plain-card.err" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of follow plain-card)"
  # 被外部信号杀掉（文件头 ⑤「被外部信号杀掉」）：PATH 最前面放假 journalctl，earlyoom 那一路打「sending SIGTERM to process <号>」，号取一个文件里写的那一个，内核那一路打一行不相干的；
  #   命令把自己的进程号（$$）写进文件、睡 3 秒、退 143 ⇒ 驱动退 143，标准错误里恰好一行 MULTI_HOST_RUN_PEER_EXTERNAL_KILL source=earlyoom pid=<那个进程号>
  #   （弄坏开关 peer-external-kill-unreported 下一行都没有）；日志点名的是 1 号进程（不是这一趟的）、同样退 143 ⇒ 没有这一行（弄坏开关 peer-external-kill-unfiltered 下有）；
  #   日志点名了这一趟的进程号而命令退 0 ⇒ 没有这一行；
  #   内核那一路打「Memory cgroup out of memory: Killed process <号> (」（内存组到了它自己的上限）点名这一趟的进程号、命令退 137 ⇒ 没有这一行（弄坏开关 peer-memory-cgroup-kill-reported 下有）
  local fake_journal_directory="$scene/fake-journal" other_journal_directory="$scene/other-journal" kill_pid_file="$scene/kill-command.pid" other_pid_file="$scene/other-killed.pid" kill_lines
  mkdir -p "$fake_journal_directory" "$other_journal_directory"
  echo 1 > "$other_pid_file"
  fake_journal() { # <目录> <读进程号的文件>：往目录里放假 journalctl（earlyoom 那一路点名文件里的进程号，内核那一路打一行不相干的）
    printf '%s\n' '#!/usr/bin/env bash' \
      '# multi-host-run.sh 自证的假 journalctl：-t earlyoom 点名一个进程号被杀，-k 打一行不相干的' \
      'case "$*" in' \
      "  *'-t earlyoom'*) if [[ -s $(printf '%q' "$2") ]]; then echo \"sending SIGTERM to process \$(cat $(printf '%q' "$2")) uid 1000 \\\"selftest\\\": oom_score 5\"; fi ;;" \
      '  *-k*) echo "kernel: unrelated line" ;;' \
      'esac' \
      'exit 0' > "$1/journalctl"
    chmod +x "$1/journalctl"
  }
  fake_journal "$fake_journal_directory" "$kill_pid_file"
  fake_journal "$other_journal_directory" "$other_pid_file"
  local cgroup_journal_directory="$scene/cgroup-journal" cgroup_pid_file="$scene/cgroup-kill-command.pid"
  mkdir -p "$cgroup_journal_directory"
  # 内存组到了它自己上限的假 journalctl：-t earlyoom 一行都不打，-k 打内核「Memory cgroup out of memory」那一行，点名文件里的进程号
  printf '%s\n' '#!/usr/bin/env bash' \
    '# multi-host-run.sh 自证的假 journalctl：-k 打内存组到了它自己上限的那一行，-t earlyoom 不打' \
    'case "$*" in' \
    "  *-k*) if [[ -s $(printf '%q' "$cgroup_pid_file") ]]; then echo \"Memory cgroup out of memory: Killed process \$(cat $(printf '%q' "$cgroup_pid_file")) (bash) total-vm:8000kB, anon-rss:4000kB\"; fi ;;" \
    'esac' \
    'exit 0' > "$cgroup_journal_directory/journalctl"
  chmod +x "$cgroup_journal_directory/journalctl"
  ( cd "$scene/repo" && exec "${scene_command[@]:0:${#scene_command[@]}-2}" PATH="$fake_journal_directory:$fake_rustup_directory:$PATH" "${scene_command[@]: -2}" \
      --peer --memory-cap 256M --label follow-kill -- bash -c 'echo "$$" > "$0"; sleep 3; exit 143' "$kill_pid_file" ) > "$scene/kill.out" 2> "$scene/kill.err" < /dev/null
  echo "$?" > "$scene/kill.exit"
  kill_lines="$(grep -c "^MULTI_HOST_RUN_PEER_EXTERNAL_KILL source=earlyoom pid=$(cat "$kill_pid_file" 2>/dev/null) line=sending SIGTERM" "$scene/kill.err" 2>/dev/null)" || true
  expect "被外部信号杀掉 日志点名了这一趟进程号、命令退 143 ⇒ 打 MULTI_HOST_RUN_PEER_EXTERNAL_KILL source=earlyoom pid=<号>" \
    "$([[ "$(exit_of follow kill)" == 143 && "${kill_lines:-0}" == 1 ]]; echo $?)" \
    "退 $(exit_of follow kill)，PEER_EXTERNAL_KILL 行 ${kill_lines:-0} 行：$(grep 'MULTI_HOST_RUN_PEER_EXTERNAL_KILL' "$scene/kill.err" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of follow kill)"
  ( cd "$scene/repo" && exec "${scene_command[@]:0:${#scene_command[@]}-2}" PATH="$other_journal_directory:$fake_rustup_directory:$PATH" "${scene_command[@]: -2}" \
      --peer --memory-cap 256M --label follow-kill-other -- bash -c 'echo "$$" > "$0"; sleep 3; exit 143' "$scene/other-command.pid" ) > "$scene/kill-other.out" 2> "$scene/kill-other.err" < /dev/null
  echo "$?" > "$scene/kill-other.exit"
  kill_lines="$(grep -c '^MULTI_HOST_RUN_PEER_EXTERNAL_KILL' "$scene/kill-other.err" 2>/dev/null)" || true
  expect "被外部信号杀掉 日志点名的是别的进程（1 号）、命令退 143 ⇒ 不打 MULTI_HOST_RUN_PEER_EXTERNAL_KILL" \
    "$([[ "$(exit_of follow kill-other)" == 143 && "${kill_lines:-0}" == 0 ]]; echo $?)" \
    "退 $(exit_of follow kill-other)，PEER_EXTERNAL_KILL 行 ${kill_lines:-0} 行：$(grep 'MULTI_HOST_RUN_PEER_EXTERNAL_KILL' "$scene/kill-other.err" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of follow kill-other)"
  ( cd "$scene/repo" && exec "${scene_command[@]:0:${#scene_command[@]}-2}" PATH="$fake_journal_directory:$fake_rustup_directory:$PATH" "${scene_command[@]: -2}" \
      --peer --memory-cap 256M --label follow-kill-clean -- bash -c 'echo "$$" > "$0"; sleep 3' "$kill_pid_file" ) > "$scene/kill-clean.out" 2> "$scene/kill-clean.err" < /dev/null
  echo "$?" > "$scene/kill-clean.exit"
  kill_lines="$(grep -c '^MULTI_HOST_RUN_PEER_EXTERNAL_KILL' "$scene/kill-clean.err" 2>/dev/null)" || true
  expect "被外部信号杀掉 日志点名了这一趟进程号而命令退 0 ⇒ 不打 MULTI_HOST_RUN_PEER_EXTERNAL_KILL" \
    "$([[ "$(exit_of follow kill-clean)" == 0 && "${kill_lines:-0}" == 0 ]]; echo $?)" \
    "退 $(exit_of follow kill-clean)，PEER_EXTERNAL_KILL 行 ${kill_lines:-0} 行：$(grep 'MULTI_HOST_RUN_PEER_EXTERNAL_KILL' "$scene/kill-clean.err" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of follow kill-clean)"
  ( cd "$scene/repo" && exec "${scene_command[@]:0:${#scene_command[@]}-2}" PATH="$cgroup_journal_directory:$fake_rustup_directory:$PATH" "${scene_command[@]: -2}" \
      --peer --memory-cap 256M --label follow-kill-cgroup -- bash -c 'echo "$$" > "$0"; sleep 3; exit 137' "$cgroup_pid_file" ) > "$scene/kill-cgroup.out" 2> "$scene/kill-cgroup.err" < /dev/null
  echo "$?" > "$scene/kill-cgroup.exit"
  kill_lines="$(grep -c '^MULTI_HOST_RUN_PEER_EXTERNAL_KILL' "$scene/kill-cgroup.err" 2>/dev/null)" || true
  expect "被外部信号杀掉 内核日志是 Memory cgroup out of memory 点名这一趟进程号、命令退 137 ⇒ 不打 MULTI_HOST_RUN_PEER_EXTERNAL_KILL" \
    "$([[ "$(exit_of follow kill-cgroup)" == 137 && "${kill_lines:-0}" == 0 && -s "$cgroup_pid_file" ]]; echo $?)" \
    "退 $(exit_of follow kill-cgroup)，PEER_EXTERNAL_KILL 行 ${kill_lines:-0} 行：$(grep 'MULTI_HOST_RUN_PEER_EXTERNAL_KILL' "$scene/kill-cgroup.err" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of follow kill-cgroup)"
  # 现场目录（文件头 ⑤「现场目录」）：驱动环境里给了 RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY（场景的共享根下），第二台那条命令往它的 TMPDIR 写一份文件、退 7
  #   ⇒ 驱动退 7，那一份由第二台那一层内存包装整个搬成这个目录（弄坏开关 peer-scene-not-passed 下留在第二台的临时根下：照它打的路径删掉，不留给别的趟）
  local scene_target="$scene/scene-of-peer-attempt" left_kept
  ( cd "$scene/repo" && exec "${scene_command[@]:0:${#scene_command[@]}-2}" RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY="$scene_target" "${scene_command[@]: -2}" \
      --peer --memory-cap 256M --label follow-scene -- bash -c ': > "$TMPDIR/left-by-peer-command"; exit 7' ) > "$scene/scene.out" 2> "$scene/scene.err" < /dev/null
  echo "$?" > "$scene/scene.exit"
  expect "现场目录 驱动环境里给了 RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY、第二台那条命令往 TMPDIR 写了文件又退 7 ⇒ 退 7、那一份由第二台那一层内存包装搬成这个目录" \
    "$([[ "$(exit_of follow scene)" == 7 && -e "$scene_target/left-by-peer-command" ]]; echo $?)" \
    "退 $(exit_of follow scene)，$scene_target 里「$(ls -A "$scene_target" 2>/dev/null | tr '\n' ' ')」；stderr 尾部：$(tail_of follow scene)"
  while IFS= read -r left_kept; do
    [[ "$left_kept" == "$MULTI_HOST_PEER_DEFAULT_TEMPORARY_ROOT/$MULTI_HOST_PEER_TEMPORARY_PREFIX"* && -d "$left_kept" ]] && rm -rf -- "${left_kept:?}"
  done < <(grep -o "留着看现场：$MULTI_HOST_PEER_DEFAULT_TEMPORARY_ROOT/$MULTI_HOST_PEER_TEMPORARY_PREFIX[^ ]*" "$scene/scene.err" 2>/dev/null | sed 's/^留着看现场：//')

  # 换字（文件头 ⑤「跑着时接回」）两格。
  #   坏字节：第二台那条命令的输出里夹一行不是合法 UTF-8 的字节（汉字被按字节截断，只剩两个字节），前后各一行带第二台地址的行 ⇒ 驱动退 0；本机标准错误里这三行同序都在，
  #   坏行的原字节一个不差，前后两行里的地址被换成「第二台」（私有的字照样被换掉）。弄坏开关 PEER_HOST_LIB_BREAK=redact-strict-utf8（换字照严格 UTF-8 读）下坏行起整块都不打。
  #   换字失败：自证的假 python3 放在 PATH 最前面，换字那一段（-c 的代码里有 PEER_HOST_REDACT_HOST）退 9，别的原样交给真的 python3 ⇒ 驱动照旧退 0、
  #   本机标准错误里至少一行「! 第二台那条命令的输出第 N 到 M 字节换字时退了 9，…」（输出可能分几轮接回，每一块各报一行）、没有没换字的地址与目录漏出来。
  #   弄坏开关 peer-redact-failure-unreported（悄悄丢掉）下一行都不报。
  local redact_bytes_lines redact_failure_lines redact_failure_leaks fake_python_directory="$work/fake-python"
  prepare_scene redact-bytes
  scene="$work/redact-bytes"
  run_in redact-bytes run --peer --memory-cap 256M --label redact-bytes -- bash -c 'printf "redact-before 192.0.2.61\n"; printf "redact-bad-\xe5\xbd\n"; printf "redact-after 192.0.2.61\n"'
  grep -a '^redact-' "$scene/run.err" > "$scene/redact-bytes.lines" 2>/dev/null
  printf 'redact-before 第二台\nredact-bad-\xe5\xbd\nredact-after 第二台\n' > "$scene/redact-bytes.expected"
  redact_bytes_lines="$(LC_ALL=C sed -n 'l' "$scene/redact-bytes.lines" 2>/dev/null | tr '\n' ' ')"
  expect "换字 第二台那条命令的输出里夹一行不是合法 UTF-8 的字节（按字节截断的汉字）⇒ 坏行前后的行都接回、坏行的原字节照样在、私有的地址照样被换掉" \
    "$([[ "$(exit_of redact-bytes run)" == 0 ]] && cmp -s "$scene/redact-bytes.lines" "$scene/redact-bytes.expected"; echo $?)" \
    "退 $(exit_of redact-bytes run)；读到的（sed l 的写法）：${redact_bytes_lines:-（一行都没有）}；stderr 尾部：$(tail_of redact-bytes run)"
  mkdir -p "$fake_python_directory"
  printf '%s\n' '#!/usr/bin/env bash' \
    '# multi-host-run.sh 自证的假 python3：换字那一段（-c 的代码里有 PEER_HOST_REDACT_HOST）退 9，别的原样交给真的 python3' \
    'if [[ "${1:-}" == -c && "${2:-}" == *PEER_HOST_REDACT_HOST* ]]; then exit 9; fi' \
    "exec $(printf '%q' "$(command -v python3)") \"\$@\"" > "$fake_python_directory/python3"
  chmod +x "$fake_python_directory/python3"
  prepare_scene redact-fails
  scene="$work/redact-fails"
  scene_command_for redact-fails
  ( cd "$scene/repo" && exec "${scene_command[@]:0:${#scene_command[@]}-2}" PATH="$fake_python_directory:$fake_rustup_directory:$PATH" "${scene_command[@]: -2}" \
      --peer --memory-cap 256M --label redact-fails -- bash -c 'printf "redact-lost 192.0.2.61\n"' ) > "$scene/redact-fails.out" 2> "$scene/redact-fails.err" < /dev/null
  echo "$?" > "$scene/redact-fails.exit"
  redact_failure_lines="$(grep -a -c '^  ! 第二台那条命令的输出第 [0-9]* 到 [0-9]* 字节换字时退了 9，' "$scene/redact-fails.err" 2>/dev/null)" || true
  redact_failure_leaks="$(grep -a -lE '192[.]0[.]2[.]61|peer-homes|peer-bin|redact-lost' "$scene/redact-fails.out" "$scene/redact-fails.err" 2>/dev/null | sed "s|^$work/||" | tr '\n' ' ')"
  expect "换字 换字那一步自己退非 0 ⇒ 驱动照旧退 0、本机标准错误里报这一块没打出来（退了几、第几到第几字节）、没有没换字的地址与目录漏出来" \
    "$([[ "$(exit_of redact-fails redact-fails)" == 0 && "${redact_failure_lines:-0}" -ge 1 && -z "$redact_failure_leaks" ]]; echo $?)" \
    "退 $(exit_of redact-fails redact-fails)；报的行 ${redact_failure_lines:-0} 行（要至少 1 行）；漏出私有字的文件：${redact_failure_leaks:-（没有）}；stderr 尾部：$(tail_of redact-fails redact-fails)"

  # 收拾（文件头 ⑥、--recover-cleanup 那一段；peer-host-lib.sh 文件头「这一趟留下的进程」「停下的退出回执」）三格，逐格写在上面自证段说明的「收拾」。
  #   标签带这一份自证自己的进程号：并行跑的几份自证各用各的标签，按标签停的那一步不碰别的自证留下的进程
  local leftover_label="leftover-$$" escaped_pid="" escaped_alive=0
  prepare_scene leftovers
  scene="$work/leftovers"
  # 后代经 systemd-run --user --scope 跑出那一层内存包装的 scope（包装命令退出之后停自己的 scope、⑥ 先停 scope，都停不到它），只剩标签找得到：照会话号与标签那一步收它
  run_in leftovers escaped --peer --memory-cap 256M --label "$leftover_label" -- env "TASK_SERVER_ATTEMPT_LABEL=$leftover_label" \
    bash -c 'systemd-run --user --scope --quiet --collect setsid bash -c '\''echo "$$" > "$0"; exec sleep 300'\'' "$0.pid" < /dev/null > /dev/null 2>&1 & for (( rounds = 0; rounds < 100; rounds++ )); do [[ -s "$0.pid" ]] && break; sleep 0.1; done; exit 0' "$scene/escaped"
  escaped_pid="$(cat "$scene/escaped.pid" 2>/dev/null)"
  if [[ "$escaped_pid" =~ ^[1-9][0-9]*$ ]] && peer_host_process_is_alive "$escaped_pid"; then
    escaped_alive=1
    kill "$escaped_pid"   # 弄坏开关下没停掉的那一个：只停这一格的命令记下的那个进程号
  fi
  expect "收拾 --peer：命令退了、后代自己 setsid 出去还在跑（环境里带这一趟的标签）⇒ ⑥ 删之前停掉它、⑥ 那一行写停了 1 个、退 0" \
    "$([[ "$(exit_of leftovers escaped)" == 0 && -n "$escaped_pid" && "$escaped_alive" == 0 && "$(peer_runs_left leftovers)" == 0 ]] \
       && grep -q '⑥ 删了第二台这一趟的本地目录.*删之前停了这一趟留下的 1 个进程' "$scene/escaped.err"; echo $?)" \
    "退 $(exit_of leftovers escaped)，setsid 出去的那一个（进程号「$escaped_pid」）$([[ "$escaped_alive" == 1 ]] && echo 还活着 || echo 不在了)；留着 $(peer_runs_left leftovers) 条；⑥ 那一行：$(grep '⑥' "$scene/escaped.err" 2>/dev/null | tr '\n' '|' | cut_characters 300)"
  # 先按 scope 收（文件头 ⑥）：命令 setsid 出去一个清了环境的 sleep（会话号、标签都对不上）、记下它的进程号，再只给那一层内存包装（单元名里带它的进程号）发 KILL：
  # 包装停不了自己的 scope，sleep 留在里面 ⇒ 驱动退 137（那一层被 KILL）、sleep 不在了、⑥ 那一行写停 scope 时里面还有至少 1 个、第二台 runs/ 与 builds/ 下什么都不剩。
  # 那一层被 KILL 不走它的 EXIT：它给命令建的临时目录在默认临时根下、路径记在这一趟的本地目录里，⑥ 照它删（下面「临时目录」那几格）；状态目录里的峰值文件这一格留下（推的，没量过）
  local scope_label="scope-leftover-$$" scope_leftover_pid="" scope_leftover_alive=0
  prepare_scene scope-leftover
  scene="$work/scope-leftover"
  run_in scope-leftover wrapper-killed --peer --memory-cap 256M --label "$scope_label" -- \
    bash -c 'printf "%s\n" "$TMPDIR" > "$0.tmpdir"; setsid env -i sleep 300 < /dev/null > /dev/null 2>&1 & echo "$!" > "$0.pid"; wrapper="${RUN_WITH_MEMORY_CAP_OUTER_UNIT#singlefs-memory-cap-}"; kill -KILL "${wrapper%%-*}"; exit 0' "$scene/scope-leftover"
  scope_leftover_pid="$(cat "$scene/scope-leftover.pid" 2>/dev/null)"
  if [[ "$scope_leftover_pid" =~ ^[1-9][0-9]*$ ]] && peer_host_process_is_alive "$scope_leftover_pid"; then
    scope_leftover_alive=1
    kill "$scope_leftover_pid"   # 弄坏开关下没停掉的那一个：只停这一格的命令记下的那个进程号
  fi
  # 临时目录（文件头 ⑤「这一条命令自己的临时目录」、⑥「那一层留下的临时目录」）：TMPDIR 短的那一格在前，弄坏开关 peer-temporary-root-in-scratch 红在它
  local killed_temporary killed_temporary_bytes temporary_own_pattern="^${MULTI_HOST_PEER_DEFAULT_TEMPORARY_ROOT}/${MULTI_HOST_PEER_TEMPORARY_PREFIX}[0-9]+-[A-Za-z0-9]+\$"
  killed_temporary="$(cat "$scene/scope-leftover.tmpdir" 2>/dev/null)"
  killed_temporary_bytes="$(printf '%s' "$killed_temporary" | wc -c)"
  expect "临时目录 第二台那条命令拿到的 TMPDIR ⇒ 在默认临时根下、以内存包装的前缀起名、全路径不超过 $MULTI_HOST_PEER_TEMPORARY_PATH_LIMIT 字节" \
    "$([[ "$killed_temporary" =~ $temporary_own_pattern ]] && (( killed_temporary_bytes <= MULTI_HOST_PEER_TEMPORARY_PATH_LIMIT )); echo $?)" \
    "TMPDIR「$killed_temporary」（$killed_temporary_bytes 字节）"
  expect "临时目录 第二台那一层内存包装被 KILL ⇒ ⑥ 照那一层记下的路径删了它留下的临时目录" \
    "$([[ "$killed_temporary" =~ $temporary_own_pattern && ! -e "$killed_temporary" ]] && grep -q '⑥ 删了第二台那一层没收拾的临时目录' "$scene/wrapper-killed.err"; echo $?)" \
    "TMPDIR「$killed_temporary」（$([[ -e "$killed_temporary" ]] && echo 还在 || echo 不在了)）；⑥ 那几行：$(grep '⑥' "$scene/wrapper-killed.err" 2>/dev/null | tr '\n' '|' | cut_characters 300)"
  if [[ "$killed_temporary" =~ $temporary_own_pattern ]]; then rm -rf -- "$killed_temporary"; fi   # 弄坏开关下留在默认临时根下的那一份：只删这一格的命令记下的那一个
  expect "收拾 --peer：那一层内存包装被 KILL、scope 里留着清了环境又换了会话号的进程 ⇒ ⑥ 先停那个 scope、那个进程不在了、⑥ 那一行写停 scope 时里面还有 1 个" \
    "$([[ "$(exit_of scope-leftover wrapper-killed)" == 137 && -n "$scope_leftover_pid" && "$scope_leftover_alive" == 0 && "$(peer_runs_left scope-leftover)" == 0 ]] \
       && grep -q '⑥ 删了第二台这一趟的本地目录.*scope 时里面还有 [1-9][0-9]* 个' "$scene/wrapper-killed.err"; echo $?)" \
    "退 $(exit_of scope-leftover wrapper-killed)，留在 scope 里的那一个（进程号「$scope_leftover_pid」）$([[ "$scope_leftover_alive" == 1 ]] && echo 还活着 || echo 不在了)；留着 $(peer_runs_left scope-leftover) 条；⑥ 那一行：$(grep '⑥' "$scene/wrapper-killed.err" 2>/dev/null | tr '\n' '|' | cut_characters 300)"
  # 记下的路径不是那一层起的那一份（文件头 ⑥「那一层留下的临时目录」）：命令把这一趟的本地目录里那份记录改写成场景里的一个目录，再只 KILL 那一层内存包装
  #   ⇒ 那个目录还在、⑥ 打「没删第二台那一层记下的临时目录」那一行、第二台 runs/ 与 builds/ 下什么都不剩；那一层建的那一份没人删，这里照命令记下的 TMPDIR 删
  local decoy_label="temporary-decoy-$$" decoy_temporary
  prepare_scene temporary-decoy
  scene="$work/temporary-decoy"
  mkdir -p "$scene/decoy-directory"
  run_in temporary-decoy decoy --peer --memory-cap 256M --label "$decoy_label" -- \
    bash -c 'printf "%s\n" "$TMPDIR" > "$0.tmpdir"; printf "%s\n" "$0-directory" > "$MULTI_HOST_RUN_PEER_SCRATCH/$1"; wrapper="${RUN_WITH_MEMORY_CAP_OUTER_UNIT#singlefs-memory-cap-}"; kill -KILL "${wrapper%%-*}"; exit 0' \
    "$scene/decoy" "$MULTI_HOST_PEER_TEMPORARY_RECORD_NAME"
  decoy_temporary="$(cat "$scene/decoy.tmpdir" 2>/dev/null)"
  expect "临时目录 记下的路径不是默认临时根下内存包装起的那一份 ⇒ ⑥ 不删、打一行" \
    "$([[ -d "$scene/decoy-directory" && "$(peer_runs_left temporary-decoy)" == 0 ]] && grep -q '⑥ 没删第二台那一层记下的临时目录' "$scene/decoy.err"; echo $?)" \
    "场景里那个目录$([[ -d "$scene/decoy-directory" ]] && echo 还在 || echo 不在了)；留着 $(peer_runs_left temporary-decoy) 条；⑥ 那几行：$(grep '⑥' "$scene/decoy.err" 2>/dev/null | tr '\n' '|' | cut_characters 300)"
  if [[ "$decoy_temporary" =~ $temporary_own_pattern ]]; then rm -rf -- "$decoy_temporary"; fi   # 那一层被 KILL 留下的那一份：只删这一格的命令记下的那一个
  selftest_recover_case() { # <场景名> <gone|alive>：起驱动（环境里给清理回执的路径）、只 KILL 这一格自己起的驱动、放行命令、等到位、起一次 --recover-cleanup；
    # 结局写场景目录里的 recover.exit、recover.out、recover.err，起之前退出回执在不在写 receipt-before，命令的进程号在 case.command
    local recover_scene="$work/$1" recover_label="$1-$$" recover_driver recover_rounds recover_session after_release='exit 0'
    [[ "$2" == alive ]] && after_release='exec sleep 300'
    prepare_scene "$1"
    scene_command_for "$1"
    ( cd "$recover_scene/repo" && exec "${scene_command[@]:0:${#scene_command[@]}-2}" TASK_SERVER_PEER_CLEANUP_FILE="$recover_scene/cleanup.json" "${scene_command[@]: -2}" \
        --peer --memory-cap 256M --label "$recover_label" -- bash -c 'echo "$$" > "$0.command"; home="$(dirname "$(dirname "$MULTI_HOST_RUN_PEER_SCRATCH")")"; until test -e "$0.go"; do sleep 0.1; done; wrapper_pid=$(cat "$home/runs/$(basename "$MULTI_HOST_RUN_PEER_SCRATCH").pid"); [[ "$wrapper_pid" =~ ^[0-9]+$ ]] && kill -KILL "$wrapper_pid"; '"$after_release" \
        "$recover_scene/case" ) > "$recover_scene/driver.out" 2> "$recover_scene/driver.err" < /dev/null &
    recover_driver=$!
    wait_for_file "$recover_scene/case.command" 180 "$recover_driver"
    kill -KILL "$recover_driver"   # 只停这一格自己起的驱动：它不收尾、不写退出回执、清理回执留在 pending
    wait "$recover_driver" 2>/dev/null
    recover_session="$(cat "$work/peer-homes/$1/runs/$recover_label.pid" 2>/dev/null)"
    : > "$recover_scene/case.go"
    # 等起的那个 bash 被命令 KILL 掉；后代都已不在那一格另等这一趟的会话里一个进程都不剩（至多 30 秒）
    for (( recover_rounds = 0; recover_rounds < 300; recover_rounds++ )); do
      if [[ "$recover_session" =~ ^[1-9][0-9]*$ ]] && ! peer_host_process_is_alive "$recover_session" \
         && { [[ "$2" == alive ]] || [[ -z "$(ps -o pid= -s "$recover_session" 2>/dev/null)" ]]; }; then break; fi
      sleep 0.1
    done
    [[ -e "$work/peer-homes/$1/runs/$recover_label.exit" ]] && echo present > "$recover_scene/receipt-before" || echo absent > "$recover_scene/receipt-before"
    ( cd "$recover_scene/repo" && exec "${scene_command[@]}" --recover-cleanup "$recover_scene/cleanup.json" ) > "$recover_scene/recover.out" 2> "$recover_scene/recover.err" < /dev/null
    echo "$?" > "$recover_scene/recover.exit"
  }
  recover_status_of() { python3 -c 'import json, sys; print(json.load(open(sys.argv[1], encoding="utf-8")).get("status", ""))' "$work/$1/cleanup.json" 2>/dev/null; }
  selftest_recover_case recover-gone gone
  expect "收拾 --recover-cleanup：起的那个 bash 被杀、没写退出回执、后代都已不在 ⇒ 一次就确认（退 0）、补写退出回执、清理回执记 confirmed、第二台 runs/ 与 builds/ 下什么都不剩" \
    "$([[ "$(exit_of recover-gone recover)" == 0 && "$(cat "$work/recover-gone/receipt-before" 2>/dev/null)" == absent && "$(recover_status_of recover-gone)" == confirmed \
       && "$(peer_runs_left recover-gone)" == 0 ]] && grep -q '补写了退出回执「143 stopped already-gone」' "$work/recover-gone/recover.err"; echo $?)" \
    "退 $(exit_of recover-gone recover)，起之前退出回执 $(cat "$work/recover-gone/receipt-before" 2>/dev/null)、清理回执「$(recover_status_of recover-gone)」、留着 $(peer_runs_left recover-gone) 条；stderr 尾部：$(tail_of recover-gone recover | cut_characters 300)"
  local recover_command_pid recover_command_alive=0
  selftest_recover_case recover-alive alive
  recover_command_pid="$(cat "$work/recover-alive/case.command" 2>/dev/null)"
  if [[ "$recover_command_pid" =~ ^[1-9][0-9]*$ ]] && peer_host_process_is_alive "$recover_command_pid"; then
    recover_command_alive=1
    kill "$recover_command_pid"   # 弄坏开关下没停掉的那一条：只停这一格的命令记下的那个进程号（它 exec 成了 sleep）
  fi
  expect "收拾 --recover-cleanup：起的那个 bash 被杀、没写退出回执、后代还活着 ⇒ 停掉它们之后确认（退 0）、补写退出回执、后代不在了、第二台 runs/ 与 builds/ 下什么都不剩" \
    "$([[ "$(exit_of recover-alive recover)" == 0 && "$(cat "$work/recover-alive/receipt-before" 2>/dev/null)" == absent && "$(recover_status_of recover-alive)" == confirmed \
       && -n "$recover_command_pid" && "$recover_command_alive" == 0 && "$(peer_runs_left recover-alive)" == 0 ]] \
       && grep -q '补写了退出回执「1[34][37] stopped \(TERM\|KILL\)」' "$work/recover-alive/recover.err" && grep -q 'PEER_HOST_LEFTOVERS stopped=[1-9]' "$work/recover-alive/recover.err"; echo $?)" \
    "退 $(exit_of recover-alive recover)，起之前退出回执 $(cat "$work/recover-alive/receipt-before" 2>/dev/null)、清理回执「$(recover_status_of recover-alive)」、命令（进程号「$recover_command_pid」）$([[ "$recover_command_alive" == 1 ]] && echo 还活着 || echo 不在了)、留着 $(peer_runs_left recover-alive) 条；stderr 尾部：$(tail_of recover-alive recover | cut_characters 300)"

  peer_driver_cells

  if (( failures > 0 )); then
    echo "  ✗ multi-host-run.sh 自证有 $failures 格判错（共 $checked 格）"  # gate-lint:summary
    echo "     → 怎么办：设了 MULTI_HOST_RUN_BREAK、MULTI_HOST_CONFIGURATION_BREAK 或 PEER_HOST_LIB_BREAK 的，这就是要的结果（那个开关把对应的一步弄坏了）；没设的，照上面那几格去看 scripts/hosts/multi-host-run.sh 与 peer-host-lib.sh"
    return 1
  fi
  echo "  ✓ multi-host-run.sh 自证 $checked 格都对（不带 --peer 就拒、只供测试的「第二台是本机」设在自证之外就拒、--peer 在第二台同一个绝对路径里跑且退出码带回、上限不超过 SLICE_MEMORY_CAP_BY_HOST 里第二台那一项、按台的三个键各台取各台那一项、标签撞了是用法错而查标签连不上退 255、工具链不同与缺离线依赖就拒（缺的是树里的 path 依赖报树里缺、不报第二台缺缓存）、跑完删第二台这一趟的本地目录、收到 TERM 先停第二台、输出不露主机与目录、双机没开就拒、弄坏开关设在自证之外就拒、包装认出显存不够时原样退 245 并报显存不够、--peer 命令里写 CARGO_TARGET_DIR 就拒、--peer 的编译目录在第二台本地这一趟的目录下且跑完删、删之前有进程开着里面的文件就不删并报出进程、第二台起 cargo 之前判 RocksDB 预编库取值、自证的细节行按字截、第二台读本机这一份多机配置、同一份配置在第二台判的结局与这一侧相同、任务里面再带 --peer 两台都拒、起活这一侧清过场才告诉第二台、共享根外的路径与工作目录就拒、工作根与共享数据根照本机的真路径交给第二台而不在共享根下就拒、第二台的编译根在共享根下就拒、--peer 每一步打开始与结束两行秒数、跑着时把第二台的新输出逐行接回且不重不漏、输出里夹不是合法 UTF-8 的字节照接回而私有的字照换、换字那一步退非 0 就报这一块没打出来、每一轮报一行不减的 CPU、结束时报一行那一层内存包装的进程内存峰值（只算 anon 加 shmem，页缓存撑满上限也不算；命令比一轮短照那一层退出时读的报）而读不到就不报、退非 0 的现场由第二台那一层搬进服务给的现场目录、删之前报一行编译目录的占盘而量不到就不报、⑥ 删之前先停那一层内存包装的 scope（包装被 KILL 时 scope 里留着的清了环境、换了会话号的进程也停掉）、第二台那条命令拿到的 TMPDIR 在默认临时根下且不超过 60 字节、那一层被 KILL 留下的临时目录由 ⑥ 照它记下的路径删而记下的不是它起的那一份就不删、再停后代自己 setsid 出去的进程并写停了几个、--recover-cleanup 在起的那个 bash 被杀又没留退出回执时照会话与标签核过就确认并补写回执（后代还活着的先停）、双机驱动先编后跑的起法经 --peer 两步都在本机那棵树里起）"
}
