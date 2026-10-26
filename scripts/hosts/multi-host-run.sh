#!/usr/bin/env bash
# 把一件确定性的活挪到第二台跑（用户 2026-09-28 定）：默认在本机仓根跑，带 --peer 才把这棵树拷到第二台、在那边跑、取回点名的产物。
#
#   multi-host-run.sh [--peer] [--memory-cap <上限>] [--label <标签>] [--fetch <树里的相对路径>]… [--send <本机目录> <树里的相对路径>]… -- <命令与参数…>
#   multi-host-run.sh --selftest
#   --send 只在 --peer 时有意义：④ 拷完树、提交之后，把点名的本机目录 rsync 到第二台那棵树里的相对路径下（例如上一次跑积累的判定库，
#   让第二台复用已核过的块）；不带 --peer 给 --send 是用法错（退 2）。
#
# 不带 --peer：不读多机配置，在本机仓根（这份脚本所在的树）经 scripts/hosts/run-with-memory-cap.sh <上限> 跑那条命令，
#   上限默认 8G（照 scripts/experiment-apparatus/replay.sh 的 REPLAY_MEMORY_CAP 默认值）。
# 带 --peer：
#   ① 配置经 scripts/hosts/peer-host-lib.sh 的 peer_host_load_configuration 读（判法是 scripts/crash/crash-point-replay-shard-configuration-check.sh，
#      配置 ${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}，模板是仓根 multi-host.env.example）；判不过退 255。
#      读之前先单跑一次判法看双机开关（ENABLE_ACROSS_MACHINES，定义在那份判法里；判法退 3 是「双机：关」）：没开就拒、退 255，
#      出路是在配置里写一行 ENABLE_ACROSS_MACHINES=1，不偷偷退回本机跑（用户 2026-09-28 定：双机要参数开启）。
#      这一台就是第二台（SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE=peer：别的驱动带着配置把活挪过来的）时判法同样退 3，照样拒、退 255，说这一台是第二台：
#      第二台上不再往「第二台」起活（scripts/hosts/peer-host-lib.sh 文件头「两台用同一份多机配置」）。
#   ② 上限默认取配置的 PEER_MEMORY_CAP；--memory-cap 给的比它大就拒（退 2）。
#   ③ 两台工具链比一遍（peer_host_compare_toolchains：rustc -Vv 前三行与 host 行、cargo -V，另比 rustup component list --installed 里 rustfmt、clippy 那两行），
#      不同就拒、说清差在哪，哪一台缺 rustfmt 或 clippy 按工具链不同拒、点名缺的组件，都退 255（scripts/hosts/peer-host-lib.sh 文件头「工具链比对」）。
#   ④ 树拷到第二台 <PEER_REPOSITORY_DIRECTORY>/runs/<标签>/（新建目录，标签已在就拒、退 2）：拷的是 git 列得出的文件
#      （git ls-files -co --exclude-standard：跟踪的与没被忽略的新文件，所以不带 .git、target、本地配置 multi-host.env 与别的被忽略的文件；
#      树不是 git 仓（主 agent 与实现员的副本）时整棵树拷过去，不带 .git、target、target-*、multi-host.env），
#      rsync 不带 --delete；规范副本另拷（peer_host_sop_source：树里有拷树里的，没有拷主仓的），拷完回读 scripts/preflight.py；
#      拷的途中消失的文件：没跟踪的临时文件（文件名带 .tmp.，别的会话改文件时先写的那一份）列文件时就排掉、不拷；rsync 退 24（some files vanished）而它报消失的
#      全是没跟踪的文件（git ls-files -c 列不出的）时照跑、打一句消失了哪几个；有被跟踪的文件消失、认不出是哪个文件、树不是 git 仓的，照旧判失败、退 255
#      （2026-10-21 合入后验证第一趟拷树时别的会话的 scripts/kb/kb-spec-check.py.tmp.… 在拷的途中消失，整趟退 255：records/2026-10-18-里程碑三收尾与近期欠账.md 第 853 行）；
#      在副本里 git init、git config gc.auto 0、git add -A、提交一次（给 preflight 的输入指纹用）。gc.auto 0 关掉那棵树的自动整理：一万多个文件的松散对象过了 git 默认的门槛，
#      不关的话提交之后 git 在后台起 gc --auto（连同它起的 repack、pack-objects），⑦ 删树时它们还开着树里的文件，删不掉、整趟退 255。
#      提交之后把本机这一份多机配置（① 读到的那一份）原样带过去（peer_host_carry_configuration）：放进那棵树的树根 multi-host.env 与第二台专用目录下常驻的那一份，
#      两份都回读 sha256 与本机的比，对不上就拒、退 255（命令没跑，⑦ 照删这一趟的树）。
#      带过去之后核离线依赖（peer_host_check_offline_dependencies，peer-host-lib.sh 文件头「离线依赖」）：树根有 Cargo.toml 的，在第二台那棵树里空跑
#      cargo metadata --offline，缺依赖就报「第二台缺依赖 X」与同步办法、拒、退 255（命令没跑，⑦ 照删这一趟的树）；树根没有 Cargo.toml 的不核。
#      第二台上拷树之前还没有清单，所以这一样不放进 ③。
#   ⑤ 在副本的树根经副本里的 run-with-memory-cap.sh <上限> 跑那条命令，脱离 ssh 会话起（peer_host_start_detached）：
#      进程号、退出码、输出（标准输出与标准错误合成一份）各写第二台 runs/<标签>.pid / .exit / .log，每 MULTI_HOST_RUN_POLL_SECONDS（默认 5）秒
#      经 ssh 看一次（peer_host_wait_detached）；跑完输出拷回、经 peer_host_redact 打到标准输出。
#      带到第二台的环境：RUN_WITH_MEMORY_CAP_RESERVE 设了就带；本机别的环境变量不带（真的第二台经 ssh 也拿不到），要设写成 -- env 变量=值 <命令>。
#      带到第二台的还有 SINGLEFS_SERIAL_DECLARATIONS=<第二台 runs/<标签>.serial>：那条命令里的程序当场往它写串行段的声明（写法在 scripts/hosts/core_usage.py 文件头「声明」），
#      与 peer_host_role_environment 打的几样：SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE=peer、SINGLEFS_MULTI_HOST_CONFIG=<④ 带过去的那一份>，
#      这个驱动正经 scripts/crash/gpu-quiesce-run.sh 起（起活这一侧照配置清了场、回读过了）时另加 GPU_QUIESCE_ORIGIN_READ_BACK=1（第二台上的清场包装认它，见那一份文件头「第二台」）。
#      在用核数：第二台那条命令在跑时每 MULTI_HOST_RUN_CORE_SECONDS（默认 300，只许写不超过 300 的正整数）秒经 ssh 在那棵树里跑一次
#      scripts/hosts/core_usage.py sample（那条命令整棵进程树的 utime + stime 与每条线程的，连同 .serial 的原文），样本存本机，再在本机跑 core_usage.py judge：
#      打一行「第二台在用 X 核（登记 N）」；最近一个窗口（600 秒）里在用核数与同时在算的线程都低于登记数的四分之一的，打 ✗ 一行带出路、
#      往告警记录（core_usage.py 文件头「告警记录」，看门狗 agent-watch.py 读它叫醒主 agent）追加一条，告警进程号是这个驱动的进程号；
#      声明了串行段的照报、标出秒数与理由，累计超过整趟一半的另注一句。窗口、阈值、同时在算的线程怎么算都在 core_usage.py 文件头。
#      判红不停那条命令：处置由主 agent 定。第二台那条命令的轮询经 peer-host-lib.sh 的 PEER_HOST_WAIT_HOOK 调这一步。
#      编译目录（用户 2026-10-23 定案「自己用完自己删」，不跨趟复用）：第二台那条命令在这一趟的树里编（命令不设 CARGO_TARGET_DIR 时 cargo 用树里的 target），
#      ⑦ 连同树删。不再接跨趟共用的编译目录：MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY 设了（哪种模式都算）是用法错（退 2，命令没跑）；--peer 时命令的词里
#      （连同 bash -c 的脚本）有 CARGO_TARGET_DIR= 指到树外（值以 / 或 .. 开头）也是用法错（multi_host_outside_target_in）。
#      RocksDB 预编库的取值：命令里有 cargo（有一个词是 cargo、以 /cargo 结尾，或 bash -c 的脚本里有单独的 cargo 这个词；multi_host_command_starts_cargo）时，
#      第二台起它之前先在那棵树根跑 python3 scripts/hosts/rocksdb-prebuilt.py entry --root .（判第二台装了、ssh 起的这条命令拿到的取值与第二台的预编库一致），
#      判红退 78、命令不跑，红句随输出拷回；命令整条包在 bash -c '<判> && exec "$@"' 里交给上面几样。树里没有依赖 rocksdb 的清单、第二台 PATH 上是带标记的假 cargo，entry 报不判。
#      不含 cargo 的命令不判。本机模式不判（起 cargo 的调用方在本机自己判）。
#      命令里的路径：第二台那棵树不在本机仓根那个路径上，本机仓根的绝对路径在那边不存在（命令会退 127）。--peer 开跑前（读配置、拷树之前）见 -- 之后的命令里有以本机仓根
#      （这份脚本所在的树，符号链接解开前后两种写法都认）起头的参数——整词是仓根，或以「仓根/」起头——就拒、退 2，什么都没拷到第二台、命令没跑，出路是改成树内相对路径（仓根写 .），
#      命令在第二台那棵树的树根起。只认整词：--root=<仓根> 这类带前缀的词、bash -c 脚本里写的路径不拦；仓根之外的绝对路径、名字只是以仓根起头的路径（<仓根>-旁边）不拦。
#      不带 --peer 的本机跑法不判。run-with-memory-cap.sh 排不上时打的「改用 … multi-host-run.sh --peer …」已经把这类参数改成相对路径。
#   ⑥ --fetch 点名的路径（文件或目录，相对树根，不许绝对路径与 ..）取回本机取回目录；不带 --peer 时从本机仓根拷过去，两种一样放。
#      取回目录：${MULTI_HOST_RUN_FETCH_DIRECTORY:-<工作根>/multi-host-run/<标签>}，路径在里面照原样；已有东西就拒（退 2）。
#      工作根只在 .claude/hooks/lib_work_root.py 里定义（SINGLEFS_WORK_ROOT，默认 ~/.local/share/singlefs/work，开机不清）；
#      没设 MULTI_HOST_RUN_FETCH_DIRECTORY、又给了 --fetch 而那份库跑不起来，命令不跑、退 255。
#   ⑦ 删第二台这一趟的树（连同命令在树里编出来的 target）与 .pid / .exit / .log / .serial；④ 之后在哪一步失败退出，这一步都挂在 EXIT 上照做。
#      删之前在第二台核没有进程开着树里的文件（那棵树里的 scripts/hosts/run_build_directory.py holders；树里没有这份的打一句、照删）：
#      有就是那条命令的子进程没收干净，不删、逐个报进程（换掉私有的字）、驱动退 255（命令退的不是 0 时照命令的）。删不成同样退 255。
# 本机模式的编译目录：调用方没设 CARGO_TARGET_DIR 时，这一趟在自己的临时目录里建 <临时目录>/target、export 给那条命令，正常结束、出错、被 TERM / INT / HUP 停
#   都删掉（scripts/hosts/run_build_directory.py remove：删之前核本机没有进程开着里面的文件，有就不删、报出进程、连同临时目录留着、退 255）；调用方设了的照它、不删。
# 收到 TERM / INT / HUP（退 143 / 130 / 129）：先停那条命令——本机的从它的进程号起、第二台的从它在第二台上记下的进程号起（peer_host_stop_detached），
#   第二台的停下之后先取回 --fetch 点名的路径（写到一半的产物不随树一起丢），
#   连同全部后代冻住、从最底层起 TERM、放开，至多等 30 秒，还在的 KILL——再删第二台这一趟的树，做完才退。
# 退出码：那条命令的退出码（245 与 250–254 是内存包装自己的结局，含义见 run-with-memory-cap.sh 文件头「退出码」，那一次的输出不算结果）；
#   78 带 --peer、命令里有 cargo 而第二台的 RocksDB 预编库取值判红（⑤，命令没跑）；
#   2 用法错（含 --peer 的命令里有以本机仓根起头的参数），或弄坏开关 MULTI_HOST_RUN_BREAK 设在 --selftest 之外（命令一行都没跑）；255 这个脚本自己没跑成（配置、工具链（含缺 rustfmt、clippy 组件）、拷树、第二台缺离线依赖、起不来、等不到结局，或命令退 0 而点名取回的路径不在）。
#   命令退非 0 时点名的路径不在，只提一句，退出码照旧是命令的。
# 资源释放广播（用户 2026-10-23 定「叫醒并重判，不自动挪」，records/2026-09-16-subagent拆分提案.md 第九十九节；发与收都在 scripts/hosts/resource_release_broadcast.py）：
#   带 --peer：本机这一侧等到第二台那条命令结束（⑤ 等到退出码，或它没了又没留退出码），或收到 TERM / INT / HUP 停下了它之后，由本机代发一条 peer-run-finished
#   （multi_host_broadcast_peer_finished）：host=第二台、标签是这一趟的 --label、等到了退出码就带 exit；emit --skip-peer：写本机事件目录、推给多机配置里除第二台之外的机器
#   （第二台那边的内存包装跑完自己会发一条 memory-released，写在第二台自己的事件目录里）。不带 --peer 不发（本机的内存包装跑完自己发）。
#   第二台是本机上的另一个目录（只供测试的开关 SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1）时不发，除非显式给了 RESOURCE_BROADCAST_DIRECTORY：那不是真的第二台。
#   推不出去的那几行写进本机事件目录的 push-problems.log（emit --problems-to-log），广播那一步的输出一行都不进驱动的输出（那条命令的输出给调用方读）；
#   出了什么事都不改驱动的退出码，至多晚退那份库的 PUSH_SECONDS（4 秒）加起 python 与读配置的零点几秒。
# 驱动打的每一行都不写配置里第二台的主机、别名与目录：第二台的输出与错误输出经 peer_host_redact 换成「第二台」「第二台的目录」再打。
# 共用库 scripts/hosts/peer-host-lib.sh 只调它的对外函数，另设它认的 PEER_HOST_WAIT_HOOK（⑤ 的在用核数）。
#
# gate-similar: crash-point-replay-shard-run.sh 它把一条登记了分片的崩溃枚举用例切成两片、两台各跑一片再 merge 并写 checker-tier-crash-point-replay 的全绿标记，步骤与输入指纹都绑在那一类用例上；它按内容进三条用例的输入指纹（admission.py 的 SHARD_DRIVER_FILES），并进来改一行就废三格标记
# gate-similar: mutation-shard-run.sh 它是 checker-tier-crates-mutation-replay 专用的变异表分行驱动：按用时分两堆、两台各跑一份 checker-tier-crates-mutation-replay、拷回判定库记录核过再导入，判定归 checker-tier-crates-mutation-replay；这里只挪一条任意命令，不分行、不导记录，两份共用的第二台那几段已经抽在 peer-host-lib.sh
#
# 弄坏开关 MULTI_HOST_RUN_BREAK=<项>（只给证红用，逗号分隔可给几个；打开时自证退 1）。只在 --selftest 起的驱动里生效：自证在它的临时目录里放一份标记文件，
#   路径经 MULTI_HOST_RUN_SELFTEST_MARK 带给它起的驱动，文件在、内容是 MULTI_HOST_SELFTEST_MARK_CONTENT 那一行才认开关（驱动读完就 unset，不带给命令）；
#   别处设了它就拒、退 2，命令一行都没跑。逐项写开关名、弄坏哪一处、打开后自证哪几格红
# （项名是自证 expect 的格名，一行一项；失败行是「✗ <格名>：<细节>」，括号里抄细节的句首，随跑变的路径、进程号写成 …）：
#   MULTI_HOST_RUN_BREAK=local-reads-configuration  不带 --peer 也读配置（自证的配置是坏的，判不过、退 255）
#       Ⓐ 默认本机跑：配置是坏的、调用方的会话里已有一条日常的在跑也照跑、退 0、在本机仓根跑、第二台一个目录都没建（「退 255，在「」跑」）
#       Ⓐ 默认本机跑经 run-with-memory-cap.sh（cgroup 在 singlefs-memory-cap- 的 scope 里）（「命令记下的 cgroup：」后面是空的）
#       Ⓐ 默认本机跑 --fetch：取回的与仓根那一份逐字相同（「cmp: …/local/repo/out/local.txt: No such file or directory」）
#       连带红的格：「✗ Ⓙ 没设 MULTI_HOST_RUN_FETCH_DIRECTORY ⇒ 取回到工作根下 multi-host-run/<标签>/，不…」、「✗ Ⓙ 设了 MULTI_HOST_RUN_FETCH_DIRECTORY ⇒ 取回到它点名的目录，工作根下没有这一趟：设了变量那一趟退 2…」
#   MULTI_HOST_RUN_BREAK=local-without-memory-cap  不带 --peer 时不经 run-with-memory-cap.sh 跑
#       Ⓐ 默认本机跑经 run-with-memory-cap.sh（cgroup 在 singlefs-memory-cap- 的 scope 里）（「命令记下的 cgroup：0::/user.slice/…」）
#       连带红的格：「✗ Ⓚ 本机跑、内存包装认出显存不够 ⇒ 原样退 245、报「显存不够」、说那一次的输出不算结果」
#   MULTI_HOST_RUN_BREAK=peer-exit-dropped  第二台那条命令退几都退 0
#       Ⓑ --peer 退出码原样带回：第二台那条命令退 7 ⇒ 驱动退 7（「退 0」）
#       Ⓞ 第二台起含 cargo 的命令之前判第二台的取值：取值缺了 ⇒ 退 78、假 cargo 没起；不含 cargo 的命令不判、照跑退 0（「含 cargo 的退 0、假 cargo 没起…」）
#       连带红的格：「✗ Ⓢ --peer 跑完（命令退 3）⇒ 本机代发一条 peer-run-finished…」、「✗ Ⓢ 资源释放广播那份库出事 ⇒ 驱动照退命令的 3、输出里没有广播那一步的行」
#   MULTI_HOST_RUN_BREAK=peer-without-memory-cap  第二台那条命令不经副本里的 run-with-memory-cap.sh 跑
#       Ⓑ --peer 经副本里的 run-with-memory-cap.sh 跑（cgroup 在 singlefs-memory-cap- 的 scope 里）（「命令记下的 cgroup：0::/user.slice/…」）
#   MULTI_HOST_RUN_BREAK=no-peer-cap-limit  --memory-cap 比配置的 PEER_MEMORY_CAP 大也照跑
#       Ⓑ --memory-cap 2G 比配置的 PEER_MEMORY_CAP 1G 大 ⇒ 拒（退 2），命令没跑（「退 0，命令跑了」）
#   MULTI_HOST_RUN_BREAK=no-toolchain-check  不比两台工具链
#       Ⓒ 第二台的 rustc -Vv 不同 ⇒ 拒（退 255，说两台的工具链不同），命令没跑、第二台没建这一趟的目录（「退 0，命令跑了」）
#   MULTI_HOST_RUN_BREAK=fetch-altered  取回之后往每个取回的文件末尾加一个字节
#       Ⓐ 默认本机跑 --fetch：取回的与仓根那一份逐字相同（「cmp: EOF on …」）
#       Ⓓ --fetch 取回的一个文件、一个目录与第二台上算的 sha256 逐字相同（「退 0；核对：out/random.bin: FAILED」）
#       连带红的格：「✗ Ⓙ 没设 MULTI_HOST_RUN_FETCH_DIRECTORY ⇒ 取回到工作根下 multi-host-run/<标签>/，不…」、「✗ Ⓙ 设了 MULTI_HOST_RUN_FETCH_DIRECTORY ⇒ 取回到它点名的目录，工作根下没有这一趟：设了变量那一趟退 0…」
#   MULTI_HOST_RUN_BREAK=no-peer-cleanup  跑完（成功、失败都算）不删第二台这一趟的树
#       Ⓔ Ⓑ Ⓓ 那几趟（含退 7 的那一趟）跑完，第二台 runs/ 下什么都不剩（「Ⓑ 那边留着：peer-ok peer-ok.exit …」）
#       Ⓕ 第二台那条命令在跑时收到 TERM ⇒ 退 143，第二台那条命令不在了，runs/ 下什么都不剩（「退 143，第二台那条命令的进程号「…」不在了；runs/ 下留着 3 条」）
#       Ⓠ --peer：命令在那棵树里建的 target 跑完连同树删、第二台那条命令不带本机的 CARGO_TARGET_DIR（「…runs/ 下留着 4 条」）
#       Ⓡ --peer：第二台那条命令退了、还有进程开着那棵树里的文件 ⇒ ⑦ 不删、报出那个进程、退 255（『退 0，树还在』）
#       Ⓥ 带到第二台的配置与本机这一份 sha256 不同（假 rsync 拷完往那一份末尾加一行）⇒ 拒（退 255，说两台的配置不同），命令没跑、第二台这一趟的树删了（「退 255，命令没跑，runs/ 下留着 1 条」）
#       连带红的格：「✗ Ⓡ --peer：第二台那条命令退了、还有进程开着那棵树里的文件 ⇒ ⑦ 不删、报出那个进程、退 255」
#   MULTI_HOST_RUN_BREAK=no-peer-stop-on-signal  收到 TERM 不停第二台那条命令
#       Ⓕ 第二台那条命令在跑时收到 TERM ⇒ 退 143，第二台那条命令不在了，runs/ 下什么都不剩（「退 143，第二台那条命令的进程号「…」还活着」）
#   MULTI_HOST_RUN_BREAK=no-redact  第二台的输出不经 peer_host_redact 就打
#       Ⓖ 输出里没有配置里的主机名与第二台的目录，命令打的那两样换成了「第二台」「第二台的目录」（「有的：peer/run.out」）
#   MULTI_HOST_RUN_BREAK=switch-off-falls-back-to-local  配置里双机没开时退回本机跑
#       Ⓗ 配置里没写 ENABLE_ACROSS_MACHINES ⇒ --peer 拒（退 255，出路写 ENABLE_ACROSS_MACHINES=1），命令哪台都没跑、第二台没建目录（「退 0，命令跑了」）
#   内存包装的弄坏开关 RUN_WITH_MEMORY_CAP_BREAK=sessiononejob（装回 2026-10-06 撤了的「同一个会话同时只许一条」）也带进这份自证
#   （包装只在它自证的标记在时认这个开关：自证照包装文件头「弄坏开关只在自证里认」建一份标记文件、export RUN_WITH_MEMORY_CAP_SELFTEST_MARK）：
#     Ⓐ 在占着一条的同一个会话里起，被装回的那道拒绝挡下（退 252）
#       Ⓐ 默认本机跑：配置是坏的、调用方的会话里已有一条日常的在跑也照跑、退 0、在本机仓根跑、第二台一个目录都没建（「退 252，在「」跑」）
#       Ⓐ 默认本机跑经 run-with-memory-cap.sh（cgroup 在 singlefs-memory-cap- 的 scope 里）（「命令记下的 cgroup：」后面是空的）
#       Ⓐ 默认本机跑 --fetch：取回的与仓根那一份逐字相同（「cmp: …/local/repo/out/local.txt: No such file or directory」）
#   MULTI_HOST_RUN_BREAK=copy-requires-git  树不是 git 仓时照旧只认 git ls-files（改前的样子：列不出文件就拒）
#       Ⓛ 不是 git 仓的树照样拷到第二台跑：退 0，普通文件拷过去了，target 与本地配置没拷（「退 …，标记 …，target …，配置 …」）
#   MULTI_HOST_RUN_BREAK=sync-ignored-files  树整个拷过去（只排除 .git、target），被 git 忽略的文件也拷
#       Ⓑ --peer 跑成：退 0，在第二台 runs/<标签>/ 那棵副本里跑（有规范副本、提交过一次），被 git 忽略的 .env 没拷过去（「退 0，在「…/runs/peer-ok」跑，提交 …，.env 拷过去了」）
#       连带红的格：「✗ Ⓛ 不是 git 仓的树照样拷到第二台跑」、「✗ Ⓞ 拷树：没跟踪的临时文件 scripts/kb/sample.py.tmp.… 不拷，没跟踪的 untracked-note.txt…」
#   MULTI_HOST_RUN_BREAK=tmp-files-copied  没跟踪的临时文件（文件名带 .tmp.）照改前也拷
#       Ⓞ 拷树：没跟踪的临时文件 scripts/kb/sample.py.tmp.4242 不拷，没跟踪的 untracked-note.txt 在拷的途中消失（rsync 退 24）⇒ 照跑、退 0、报消失了哪个（「退 0，命令跑了，临时文件拷过去了」）
#   MULTI_HOST_RUN_BREAK=vanished-untracked-fails  rsync 退 24 一律判失败（改前的样子）
#       Ⓞ 拷树：没跟踪的临时文件 scripts/kb/sample.py.tmp.4242 不拷，没跟踪的 untracked-note.txt 在拷的途中消失（rsync 退 24）⇒ 照跑、退 0、报消失了哪个（「退 255，命令没跑」）
#       连带红的格：「✗ Ⓟ 拷树：被跟踪的 payload/input.txt 在拷的途中消失（rsync 退 24）⇒ 判失败…」
#   MULTI_HOST_RUN_BREAK=vanished-tracked-tolerated  rsync 退 24 时被跟踪的文件消失了也放过
#       Ⓟ 拷树：被跟踪的 payload/input.txt 在拷的途中消失（rsync 退 24）⇒ 判失败（退 255，说被跟踪的文件消失了），命令没跑（「退 0，命令跑了」）
#   MULTI_HOST_RUN_BREAK=break-honored-anywhere  弄坏开关在 --selftest 之外也认
#       Ⓘ MULTI_HOST_RUN_BREAK 设在 --selftest 之外（不带自证的标记）⇒ 拒（退 2，说设在 --selftest 之外），命令没跑（「退 0，命令跑了」）
#   MULTI_HOST_RUN_BREAK=fetch-under-tmp  没设 MULTI_HOST_RUN_FETCH_DIRECTORY 时取回目录退回 ${TMPDIR:-/tmp}/singlefs-multi-host-run/<标签>（改前的默认）
#       Ⓙ 没设 MULTI_HOST_RUN_FETCH_DIRECTORY ⇒ 取回到工作根下 multi-host-run/<标签>/，不落进 $TMPDIR；设了它 ⇒ 取回到它点名的目录、工作根下没有这一趟
#         （「默认那一趟退 0，工作根下没有取回的文件；$TMPDIR 下有」）
#   MULTI_HOST_RUN_BREAK=gpu-memory-unexplained  那条命令退 245（内存包装认出的显存不够）时不报「显存不够」
#       Ⓚ 本机跑、内存包装认出显存不够 ⇒ 原样退 245、报「显存不够」、说那一次的输出不算结果（「退 245，stderr 尾部：…」）
#   MULTI_HOST_RUN_BREAK=core-usage-unwatched  第二台那条命令在跑时不取样、不判在用核数（改前的样子：只报 running）
#       Ⓝ 第二台只起 1 条线程忙等 ⇒ 打「第二台在用 X 核（登记 N）」与 ✗ 一行带出路、写告警记录（带会话号与来源）（「退 0；在用核数那几行：；告警记录 0 条」）
#       Ⓝ 第二台起登记数那么多条线程忙等 ⇒ 打「第二台在用 X 核（登记 N）」、不打 ✗、不写告警记录（「退 0；在用核数那几行：；…」）
#       Ⓝ 第二台 1 条线程忙等、程序先声明了串行 ⇒ 照报 ✗，标出理由（「退 0；✗ 那几行：」）
#   MULTI_HOST_RUN_BREAK=peer-rocksdb-unchecked  第二台起含 cargo 的命令之前不判第二台的 RocksDB 预编库取值（改前的样子）
#       Ⓞ 第二台起含 cargo 的命令之前判第二台的取值：取值缺了 ⇒ 退 78、假 cargo 没起；不含 cargo 的命令不判、照跑退 0（「含 cargo 的退 0、假 cargo 起了…」）
#   MULTI_HOST_RUN_BREAK=outside-target-accepted  不拒 MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY 与 --peer 命令里指到树外的 CARGO_TARGET_DIR（改前的样子：跨趟共用编译目录）
#       Ⓜ 设了 MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY ⇒ 用法错（退 2），命令没跑（「退 0，命令跑了」）
#       Ⓜ --peer 命令里 CARGO_TARGET_DIR 指到树外（env 的赋值、bash -c 的脚本两种写法）⇒ 用法错（退 2），命令没跑（「env 那一趟退 0，bash -c 那一趟退 0」）
#   MULTI_HOST_RUN_BREAK=peer-absolute-path-accepted  --peer 的命令里有以本机仓根起头的参数也照跑（改前的样子：路径带到第二台，那边没有这条路径，命令退 127）
#       Ⓩ --peer 命令里有以本机仓根起头的参数（仓根本身、仓根下的文件各一趟）⇒ 用法错（退 2，出路写树内相对路径），第二台 runs/ 下什么都没建、命令没跑（「仓根本身那一趟退 0」）
#   MULTI_HOST_RUN_BREAK=local-absolute-path-refused  不带 --peer 的本机跑，命令里有以本机仓根起头的参数也拒（把只该管 --peer 的那道拒绝铺到本机模式）
#       Ⓩ 不带 --peer 的本机跑带仓根下的绝对路径 ⇒ 照跑、退 0（「本机那一趟退 2，文件不在」）
#   MULTI_HOST_RUN_BREAK=local-target-kept  本机模式不删这一趟自己建的编译目录（改前的样子：命令在仓根照仓根的 target 编、跨趟留着）
#       Ⓠ 本机跑：没设 CARGO_TARGET_DIR ⇒ 命令拿到的是这一趟自己建的编译目录，正常退 0、出错退 7、被 TERM 停（退 143）三种结局跑完都删了（「…还在」）
#       Ⓡ 本机跑：命令退了、还有进程开着这一趟编译目录里的文件 ⇒ 不删、报出那个进程、退 255（「退 0，编译目录还在」）
#   MULTI_HOST_RUN_BREAK=open-files-unchecked  删这一趟的编译目录（本机）与第二台这一趟的树之前不核有没有进程开着里面的文件
#       Ⓡ 本机跑：命令退了、还有进程开着这一趟编译目录里的文件 ⇒ 不删、报出那个进程、退 255（「退 0，编译目录删了」）
#       Ⓡ --peer：第二台那条命令退了、还有进程开着那棵树里的文件 ⇒ ⑦ 不删、报出那个进程、退 255（『退 0，树删了』）
#       连带红的格：「✗ Ⓡ --peer：第二台那条命令退了、还有进程开着那棵树里的文件 ⇒ ⑦ 不删、报出那个进程、退 255」
#   MULTI_HOST_RUN_BREAK=serial-declarations-unset  第二台那条命令不带 SINGLEFS_SERIAL_DECLARATIONS（程序声明不了串行段）
#       Ⓝ 第二台 1 条线程忙等、程序先声明了串行 ⇒ 照报 ✗，标出理由（「退 2；✗ 那几行：」：没设变量，declare-serial 拒了）
#   MULTI_HOST_RUN_BREAK=broadcast-none  第二台那一趟跑完不发资源释放广播（改前的样子）
#       Ⓢ --peer 跑完（命令退 3）⇒ 本机代发一条 peer-run-finished（host=第二台、标签、exit=3），驱动照退 3（『退 3，事件 0 条』）
#       连带红的格：「✗ Ⓢ --peer 跑完（命令退 3）⇒ 本机代发一条 peer-run-finished（host=第二台、标签、exit=3），驱动照…」
#   MULTI_HOST_RUN_BREAK=broadcast-test-world  第二台是本机上的另一个目录（只供测试的开关）而没给 RESOURCE_BROADCAST_DIRECTORY 也发
#       Ⓢ 第二台是本机上的另一个目录而没给 RESOURCE_BROADCAST_DIRECTORY ⇒ 不发（「默认事件目录里有 1 条 peer-run-finished」）
#   MULTI_HOST_RUN_BREAK=broadcast-exit-swallowed  广播那一步出事时照它的退出码退、输出照打
#       Ⓢ 资源释放广播那份库出事 ⇒ 驱动照退命令的 3、输出里没有广播那一步的行（『退 1，stderr 里有自证造的崩溃』）
#       连带红的格：「✗ Ⓢ 资源释放广播那份库出事 ⇒ 驱动照退命令的 3、输出里没有广播那一步的行」
#   MULTI_HOST_RUN_BREAK=detail-cut-by-bytes  自证的细节行照改前按字节截（cut -c），汉字截成半个
#       「✗ Ⓣ 自证的细节行按字截：比上限长的一行截到上限那么多个字，截出来是合法的 UTF-8」（细节「161 字节，第 159 字节起解不成 UTF-8」）
#   MULTI_HOST_RUN_BREAK=peer-gc-auto-kept  ④ 第二台那棵树 git init 之后不设 gc.auto 0（改前的样子：提交之后 git 在后台起 gc --auto）
#       Ⓑ ④ 第二台那棵树关了自动整理：那条命令在树里读到 git config gc.auto 为 0（『git config gc.auto 读到「」』）
# 判法的开关 MULTI_HOST_CONFIGURATION_BREAK 也带进这份自证，打开后红哪几格写在 scripts/crash/crash-point-replay-shard-configuration-check.sh 文件头；
# 共用库的开关 PEER_HOST_LIB_BREAK 同样带进来，打开后红哪几格写在 scripts/hosts/peer-host-lib.sh 文件头。
# 只供测试的开关：SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1（照崩溃点重放分片）：「第二台」是本机上的另一个目录，命令在本机跑、环境照 ssh 那样只给最少几样。
#
# admission: always 每次调都在这一刻这棵树上跑一条调用方给的命令；要不要重跑由调用方判，不在这里
# run-condition: command bash git rsync python3
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
multi_host_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=peer-host-lib.sh
source "$multi_host_script_directory/peer-host-lib.sh"

MULTI_HOST_LOCAL_DEFAULT_MEMORY_CAP=8G

multi_host_break_is() { [[ ",${MULTI_HOST_RUN_BREAK:-}," == *",$1,"* ]]; }

MULTI_HOST_SELFTEST_MARK_CONTENT="multi-host-run.sh --selftest 起的驱动"
multi_host_inside_selftest() { # 这个驱动是不是 --selftest 起的：MULTI_HOST_RUN_SELFTEST_MARK 指的文件在、第一行对得上（文件头「弄坏开关」那一段）
  [[ -n "${MULTI_HOST_RUN_SELFTEST_MARK:-}" && -f "$MULTI_HOST_RUN_SELFTEST_MARK" ]] \
    && [[ "$(head -n 1 -- "$MULTI_HOST_RUN_SELFTEST_MARK")" == "$MULTI_HOST_SELFTEST_MARK_CONTENT" ]]
}

multi_host_usage_error() { # <原因>：用法错，命令一行都没跑，退 2
  echo "  ✗ multi-host-run.sh 用法错：$1" >&2
  echo "     → 怎么办：multi-host-run.sh [--peer] [--memory-cap <上限>] [--label <标签>] [--fetch <树里的相对路径>]… [--send <本机目录> <树里的相对路径>]… -- <命令与参数…>，或 --selftest；上限写正整数加 K / M / G / T，标签只用字母、数字、点、减号与下划线；--send 只配 --peer" >&2
  exit 2
}

multi_host_vanished_untracked_only() { # <树根> <是不是 git 仓 1/0> <rsync 的错误输出>：rsync 报「file has vanished」的路径全是没跟踪的文件退 0（文件头 ④「拷的途中消失的文件」）
  # multi_host_vanished_paths 设成消失了的那几个相对路径；退 1 时 multi_host_vanished_reason 写为什么不放过。弄坏开关 vanished-untracked-fails 一律退 1、
  # vanished-tracked-tolerated 被跟踪的消失了也退 0
  local tree_root="$1" tree_is_git="$2" errors="$3" line vanished resolved_root tracked_hit=() unknown_hit=()
  resolved_root="$(cd "$tree_root" 2>/dev/null && pwd -P)"
  multi_host_vanished_paths=()
  multi_host_vanished_reason=""
  while IFS= read -r line; do
    vanished="${line#*file has vanished: \"}"
    vanished="${vanished%\"*}"
    vanished="${vanished#"$tree_root"/}"
    [[ -n "$resolved_root" ]] && vanished="${vanished#"$resolved_root"/}"
    multi_host_vanished_paths+=("$vanished")
    # 去不掉树根前缀的（认不出是树里哪个文件）照被跟踪的算：宁可判失败重跑，不放过一个被跟踪的文件
    if [[ "$vanished" == /* ]]; then unknown_hit+=("$vanished")
    elif [[ "$tree_is_git" != 1 || -n "${tracked_paths[$vanished]:-}" ]]; then tracked_hit+=("$vanished"); fi
  done < <(grep -F 'file has vanished: "' "$errors" 2>/dev/null)
  if multi_host_break_is vanished-untracked-fails; then
    multi_host_vanished_reason="弄坏开关 vanished-untracked-fails：消失了就判失败"
    return 1
  fi
  if ((${#multi_host_vanished_paths[@]} == 0)); then
    multi_host_vanished_reason="错误输出里认不出哪个文件消失了"
    return 1
  fi
  if ((${#tracked_hit[@]} + ${#unknown_hit[@]})) && ! multi_host_break_is vanished-tracked-tolerated; then
    if ((${#unknown_hit[@]})); then
      multi_host_vanished_reason="认不出是树里哪个文件的消失了：${unknown_hit[*]}"
    elif [[ "$tree_is_git" == 1 ]]; then
      multi_host_vanished_reason="被跟踪的文件在拷的途中消失了：${tracked_hit[*]}"
    else
      multi_host_vanished_reason="这棵树不是 git 仓，分不出消失的是不是临时文件：${tracked_hit[*]}"
    fi
    return 1
  fi
  return 0
}

multi_host_fail() { # <原因> <出路>：这个脚本自己没跑成，退 255（EXIT 上删第二台这一趟的树）
  echo "  ✗ multi-host-run.sh：$1" >&2
  echo "     → 怎么办：$2" >&2
  exit 255
}

multi_host_cap_bytes() { # <上限>：正整数加 K / M / G / T，打成字节数
  local number="${1%[KMGT]}" power
  case "${1: -1}" in K) power=1 ;; M) power=2 ;; G) power=3 ;; *) power=4 ;; esac
  echo $(( number * 1024 ** power ))
}

multi_host_redact() { # 第二台的输出：经 peer_host_redact 换掉私有的字再打；弄坏开关 no-redact 下原样打
  if multi_host_break_is no-redact; then cat; else peer_host_redact; fi
}

multi_host_remove_peer_run() { # ⑦ 删第二台这一趟的树与 .pid / .exit / .log，回读不在了才算删了；删之前核没有进程开着树里的文件。没删成置 multi_host_cleanup_failed=1
  peer_run_pending=0
  if multi_host_break_is no-peer-cleanup; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=no-peer-cleanup：第二台 runs/$run_label 不删" >&2
    return 0
  fi
  local removal_text holder_text holder_status checker="$peer_run_directory/scripts/hosts/run_build_directory.py"
  # 开着树里文件的进程（文件头 ⑦）：那棵树里的 run_build_directory.py holders，退 3 是有；树里没有这份退 4（照删、打一句）。弄坏开关 open-files-unchecked 不核
  if multi_host_break_is open-files-unchecked; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=open-files-unchecked：删第二台这一趟的树之前不核开着的文件" >&2
  else
    holder_text="$(peer_host_run / "if test -f $(printf '%q' "$checker"); then python3 $(printf '%q' "$checker") holders $(printf '%q' "$peer_run_directory"); else exit 4; fi" 2>/dev/null)"
    holder_status=$?
    if (( holder_status == 3 )); then
      multi_host_cleanup_failed=1
      echo "  ✗ 第二台 runs/$run_label 这一趟的树里还有进程开着文件，没删（那条命令的子进程没收干净）：" >&2
      printf '%s\n' "$holder_text" | sed 's/^/      /' | multi_host_redact >&2   # gate-lint:detail
      echo "     → 怎么办：到第二台照上面的进程号逐个 python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号>，再手动删 runs/ 下的 $run_label 与 $run_label.pid / .exit / .log / .serial；修那条命令，让它停自己的子进程之后再退" >&2
      return 0
    elif (( holder_status == 4 )); then
      echo "  ! 第二台那棵树里没有 scripts/hosts/run_build_directory.py：删之前没核开着的文件" >&2
    fi
  fi
  removal_text="rm -rf -- $(printf '%q' "${peer_run_directory:?}") $(printf '%q' "${peer_pid_file:?}") $(printf '%q' "${peer_exit_file:?}") $(printf '%q' "${peer_output_file:?}") $(printf '%q' "${peer_serial_file:?}")"
  if peer_host_run / "$removal_text && test ! -e $(printf '%q' "$peer_run_directory")" >/dev/null; then
    echo "  · ⑦ 删了第二台 runs/$run_label 这一趟的树与进程号、退出码、输出、串行声明四个文件（回读不在了）" >&2
  else
    multi_host_cleanup_failed=1
    echo "  ✗ 第二台 runs/$run_label 这一趟的树没删掉" >&2
    echo "     → 怎么办：到第二台 PEER_REPOSITORY_DIRECTORY 下的 runs/ 里手动删 $run_label 与 $run_label.pid / .exit / .log / .serial" >&2
  fi
}

multi_host_on_exit() { # 删第二台这一趟的树（⑦）、本机这一趟的编译目录与临时目录；有一样没删成而本来要退 0 的改退 255（文件头 ⑦、「本机模式的编译目录」）
  local exit_code=$? keep_scratch=0
  trap '' TERM INT HUP PIPE
  if (( peer_run_pending )); then multi_host_remove_peer_run; fi
  if [[ -n "${multi_host_local_target:-}" ]]; then
    if multi_host_break_is local-target-kept; then
      echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=local-target-kept：这一趟的编译目录 $multi_host_local_target 不删" >&2
      keep_scratch=1
    elif ! python3 "$multi_host_script_directory/run_build_directory.py" remove "$multi_host_local_target" \
           $(multi_host_break_is open-files-unchecked && echo --skip-open-check) >&2; then
      multi_host_cleanup_failed=1
      keep_scratch=1
    fi
  fi
  if (( keep_scratch )); then
    echo "  · 这一趟的临时目录 $multi_host_scratch 留着（里面的编译目录没删）" >&2
  elif [[ -n "${multi_host_scratch:-}" ]]; then
    rm -rf -- "${multi_host_scratch:?}"
  fi
  if (( multi_host_cleanup_failed )) && (( exit_code == 0 )); then exit 255; fi
}

multi_host_broadcast_peer_finished() { # <第二台那条命令的退出码，没等到写空>：本机代发一条 peer-run-finished（文件头「资源释放广播」）；
  #   输出不进驱动的输出，退出码不交回（弄坏开关 broadcast-exit-swallowed 下照它退、照打）
  local -a exit_option=()
  multi_host_break_is broadcast-none && return 0
  if [[ "${SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE:-}" == 1 && -z "${RESOURCE_BROADCAST_DIRECTORY:-}" ]] && ! multi_host_break_is broadcast-test-world; then
    return 0
  fi
  [[ "$1" =~ ^[0-9]+$ ]] && exit_option=(--exit-code "$1")
  if multi_host_break_is broadcast-exit-swallowed; then
    python3 "$multi_host_script_directory/resource_release_broadcast.py" emit --kind peer-run-finished --label "$run_label" --host 第二台 \
      ${exit_option[@]+"${exit_option[@]}"} --skip-peer --problems-to-log </dev/null || exit $?
    return 0
  fi
  python3 "$multi_host_script_directory/resource_release_broadcast.py" emit --kind peer-run-finished --label "$run_label" --host 第二台 \
    ${exit_option[@]+"${exit_option[@]}"} --skip-peer --problems-to-log </dev/null >/dev/null 2>&1
  return 0
}

multi_host_stop_and_exit() { # <信号名> <退出码>：先停那条命令，再 exit（EXIT 上删第二台这一趟的树）
  local stop_output
  trap '' TERM INT HUP PIPE
  echo "  ! 收到 $1：先停那条命令，再删第二台这一趟的树，做完再退" >&2
  if [[ -n "$local_command_process" ]]; then
    if stop_output="$(peer_host_stop_process_tree "$local_command_process")"; then
      echo "  · 本机那条命令：$stop_output" >&2
    else
      echo "  ✗ 本机那条命令没停下：$stop_output" >&2
      echo "     → 怎么办：照上面列的进程号逐个 python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号>" >&2
    fi
  fi
  if (( peer_command_started )); then
    if multi_host_break_is no-peer-stop-on-signal; then
      echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=no-peer-stop-on-signal：第二台那条命令不停" >&2
    elif stop_output="$(peer_host_stop_detached "$peer_run_directory" "$peer_pid_file")"; then
      echo "  · 第二台那条命令：$stop_output" >&2
      multi_host_broadcast_peer_finished ""   # 第二台那一趟停下了：本机代发（文件头「资源释放广播」）
    else
      echo "  ✗ 第二台那条命令没停下：$stop_output" >&2
      echo "     → 怎么办：到第二台 runs/$run_label.pid 里的进程号，连同后代逐个 python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号>" >&2
    fi
    # 停下之后先把 --fetch 点名的路径取回来再删树：那条命令写到一半的产物（崩溃放量判到一半的库）不随树一起丢
    if (( peer_run_pending )) && (( ${#fetch_paths[@]} > 0 )); then multi_host_fetch; fi
  fi
  exit "$2"
}

multi_host_fetch() { # ⑥ 取回 --fetch 点名的路径：本机模式从仓根拷，--peer 从第二台这一趟的树拷。缺的路径写进 fetch_missing
  local relative_path destination
  fetch_missing=()
  (( ${#fetch_paths[@]} > 0 )) || return 0
  for relative_path in "${fetch_paths[@]}"; do
    destination="$fetch_directory/$relative_path"
    mkdir -p -- "$(dirname "$destination")" || multi_host_fail "建不了取回目录 $(dirname "$destination")" "看取回目录 $fetch_directory 的上一级可不可写、盘满没满，或把 MULTI_HOST_RUN_FETCH_DIRECTORY 指到可写的空目录"
    if (( ! peer_mode )); then
      if [[ -d "$tree_root/$relative_path" ]]; then
        { mkdir -p -- "$destination" && cp -a -- "$tree_root/$relative_path/." "$destination/"; } || fetch_missing+=("$relative_path")
      elif [[ -e "$tree_root/$relative_path" ]]; then
        cp -a -- "$tree_root/$relative_path" "$destination" || fetch_missing+=("$relative_path")
      else
        fetch_missing+=("$relative_path")
      fi
    elif peer_host_run "$peer_run_directory" "test -d $(printf '%q' "$relative_path")" >/dev/null 2>&1; then
      peer_host_copy "$(peer_host_path "$peer_run_directory/$relative_path")/" "$destination/" || fetch_missing+=("$relative_path")
    elif peer_host_run "$peer_run_directory" "test -e $(printf '%q' "$relative_path")" >/dev/null 2>&1; then
      peer_host_copy "$(peer_host_path "$peer_run_directory/$relative_path")" "$destination" || fetch_missing+=("$relative_path")
    else
      fetch_missing+=("$relative_path")
    fi
  done
  if multi_host_break_is fetch-altered; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=fetch-altered：取回的每个文件末尾加一个字节" >&2
    find "$fetch_directory" -type f -exec sh -c 'printf x >> "$1"' sh {} \;
  fi
  echo "  · ⑥ 取回 $(( ${#fetch_paths[@]} - ${#fetch_missing[@]} )) 条（共点名 ${#fetch_paths[@]} 条）到 $fetch_directory" >&2
}

multi_host_finish() { # <命令的退出码>：缺了取回的路径、包装自己的结局各提一句，按文件头「退出码」那一段退
  local command_exit="$1"
  case "$command_exit" in
    245)
      if ! multi_host_break_is gpu-memory-unexplained; then
        echo "  ! 那条命令退 245：显存不够（内存包装认出它起止之间内核报了 NV_ERR_NO_MEMORY），那一次的输出不算结果；nvidia-smi 看谁在用卡，等它跑完再重跑" >&2
      fi
      ;;
    25[0-4]) echo "  ! 那条命令退 $command_exit：内存包装 run-with-memory-cap.sh 自己的结局（文件头「退出码」），那一次的输出不算结果" >&2 ;;
  esac
  if (( ${#fetch_missing[@]} > 0 )); then
    if [[ "$command_exit" == 0 ]]; then
      multi_host_fail "那条命令退 0，点名取回的 ${fetch_missing[*]} 却不在" "看命令是不是真把它们写在树根下这几条相对路径上（--fetch 相对树根写）"
    fi
    echo "  ! 点名取回的 ${fetch_missing[*]} 不在（那条命令退 $command_exit，多半没写到那一步）" >&2
  fi
  exit "$command_exit"
}

multi_host_core_check() { # 第二台那条命令在跑时每轮询到一次 running 调一次（peer-host-lib.sh 的 PEER_HOST_WAIT_HOOK）：
  # 到了 MULTI_HOST_RUN_CORE_SECONDS 就在第二台取一次样、在本机判一次（文件头 ⑤「在用核数」）；取不到只提一句，不影响等它跑完
  (( SECONDS >= core_next_check )) || return 0
  core_next_check=$(( SECONDS + core_seconds ))
  if multi_host_break_is core-usage-unwatched; then return 0; fi
  if ! peer_host_run "$peer_run_directory" "python3 scripts/hosts/core_usage.py sample --pid-file $(printf '%q' "$peer_pid_file") --declarations $(printf '%q' "$peer_serial_file")" \
       > "$multi_host_scratch/core-sample" 2>/dev/null; then
    echo "  ! 第二台那条命令在用几个核这一次没取到（ssh 或取样没成），下一次再取" >&2
    return 0
  fi
  python3 "$multi_host_script_directory/core_usage.py" judge --incoming "$multi_host_scratch/core-sample" --series "$multi_host_scratch/core-series" \
    --declarations "$multi_host_scratch/core-declarations" --subject 第二台 --alert-pid "$$" --source "multi-host-run.sh --peer" --label "$run_label" \
    --next-step "第二台那一趟在用的核太少：停下它（python3 .claude/singlefs-ai-sop/scripts/proc.py stop $$，驱动收到 TERM 先停第二台那条命令、取回点名的产物、再删第二台的树；驱动在 detached-run.sh 起的单元里就用 detached-run.sh stop <名>），照 .claude/rules/test-script-discipline.md「编译与算法都要快」把装置里互相独立的单位分到登记的线程数上再跑；真要串行的段由程序当场往 \$SINGLEFS_SERIAL_DECLARATIONS 写「<秒>\\t此刻串行：<理由>」，核过理由成立就 --ack 进程:$$ 接着跑" \
    2>&1 | multi_host_redact >&2
  return 0
}

multi_host_local_main() { # 不带 --peer：在本机仓根经 run-with-memory-cap.sh 跑，不读配置；调用方没设 CARGO_TARGET_DIR 时用这一趟自己建的编译目录（文件头「本机模式的编译目录」）
  local command_exit wrapper=(bash "$tree_root/scripts/hosts/run-with-memory-cap.sh" "$memory_cap")
  if [[ -z "${CARGO_TARGET_DIR:-}" ]]; then
    mkdir -- "$multi_host_scratch/target" || multi_host_fail "建不了这一趟的编译目录 $multi_host_scratch/target" "看 \${TMPDIR:-/tmp} 可不可写、盘满没满"
    multi_host_local_target="$multi_host_scratch/target"
    export CARGO_TARGET_DIR="$multi_host_local_target"
    echo "  · 这一趟的编译目录 $multi_host_local_target（这里建的，跑完删）" >&2
  fi
  if multi_host_break_is local-reads-configuration; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=local-reads-configuration：不带 --peer 也读配置" >&2
    peer_host_load_configuration "$multi_host_script_directory/../crash/crash-point-replay-shard-configuration-check.sh" "$tree_root" >&2 || exit 255
  fi
  if multi_host_break_is local-without-memory-cap; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=local-without-memory-cap：不经 run-with-memory-cap.sh 跑" >&2
    wrapper=()
  fi
  echo "  · 本机仓根跑（不跨机，不读多机配置）：经 run-with-memory-cap.sh $memory_cap 起" >&2
  ( cd "$tree_root" && exec ${wrapper[@]+"${wrapper[@]}"} "$@" ) <&0 &
  local_command_process=$!
  if wait "$local_command_process"; then command_exit=0; else command_exit=$?; fi
  local_command_process=""
  echo "  · 那条命令退 $command_exit" >&2
  multi_host_fetch
  multi_host_finish "$command_exit"
}

multi_host_outside_target_in() { # <命令与参数…>：有一个词（连同 bash -c 的脚本）里写了 CARGO_TARGET_DIR= 指到树外（值以 / 或 .. 开头，可带引号）交回 0（文件头 ⑤「编译目录」）
  local word pattern='(^|[[:space:];&|(`])(export[[:space:]]+)?CARGO_TARGET_DIR=["'"'"']?(/|\.\.)'
  for word in "$@"; do
    [[ "$word" =~ $pattern ]] && return 0
  done
  return 1
}

multi_host_local_root_path_in() { # <命令与参数…>：有一个词是本机仓根、或以「仓根/」起头，交回 0，multi_host_local_root_word 设成那个词（文件头 ⑤「命令里的路径」）
  local word root
  local -a roots=()
  roots+=("$(cd "$multi_host_script_directory/../.." && pwd)")
  roots+=("$(cd "$multi_host_script_directory/../.." && pwd -P)")
  multi_host_local_root_word=""
  for word in "$@"; do
    for root in "${roots[@]}"; do
      if [[ "$word" == "$root" || "$word" == "$root"/* ]]; then
        multi_host_local_root_word="$word"
        return 0
      fi
    done
  done
  return 1
}

multi_host_command_starts_cargo() { # <命令与参数…>：有一个词是 cargo、以 /cargo 结尾，或词里（bash -c 的脚本）有单独的 cargo 这个词，交回 0（文件头 ⑤「RocksDB 预编库的取值」）
  local word pattern='(^|[[:space:];&|(`])cargo([[:space:];&|)`]|$)'
  for word in "$@"; do
    [[ "$word" == cargo || "$word" == */cargo ]] && return 0
    [[ "$word" =~ $pattern ]] && return 0
  done
  return 1
}

multi_host_peer_main() { # 带 --peer：步骤 ①–⑦ 见文件头
  local cap_limit_bytes files_list sop_source listed_path command_text reserve_text="" command_exit switch_exit
  # ① 配置：双机开关先判（判法退 3 是「双机：关」）；关着就拒，不退回本机
  if bash "$multi_host_script_directory/../crash/crash-point-replay-shard-configuration-check.sh" "$tree_root" >/dev/null 2>&1; then switch_exit=0; else switch_exit=$?; fi
  if [[ "$switch_exit" == 3 ]]; then
    if multi_host_break_is switch-off-falls-back-to-local; then
      echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=switch-off-falls-back-to-local：双机关着，退回本机跑" >&2
      return 0
    fi
    if peer_host_this_machine_is_peer; then
      multi_host_fail "带了 --peer，这一台却就是第二台（SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE=peer：别的驱动带着配置把活挪过来的）：第二台上不再往「第二台」起活，也不退回本机跑" \
        "这条命令在第二台上去掉 --peer 就在这一台跑；要再分给别的机器，回起活的那一台去分"
    fi
    multi_host_fail "带了 --peer，配置里的双机开关却没开（ENABLE_ACROSS_MACHINES 没写或不是 1）：不去第二台，也不退回本机跑" \
      "要跨机就在配置（${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}）里写一行 ENABLE_ACROSS_MACHINES=1；不跨机就去掉 --peer"
  fi
  peer_host_load_configuration "$multi_host_script_directory/../crash/crash-point-replay-shard-configuration-check.sh" "$tree_root" >&2 || exit 255
  if [[ "$peer_host_is_this_machine" == 1 ]]; then
    echo "  ! 第二台是本机上的另一个目录（只供测试的开关 SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1）" >&2
  fi
  # ② 上限
  if [[ -z "$memory_cap" ]]; then
    memory_cap="$PEER_MEMORY_CAP"
  else
    cap_limit_bytes="$(multi_host_cap_bytes "$PEER_MEMORY_CAP")"
    if (( $(multi_host_cap_bytes "$memory_cap") > cap_limit_bytes )); then
      if multi_host_break_is no-peer-cap-limit; then
        echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=no-peer-cap-limit：--memory-cap $memory_cap 比配置的 PEER_MEMORY_CAP $PEER_MEMORY_CAP 大也照跑" >&2
      else
        multi_host_usage_error "--memory-cap $memory_cap 比配置的 PEER_MEMORY_CAP $PEER_MEMORY_CAP 大（第二台上的上限不许超过它）"
      fi
    fi
  fi
  # ③ 工具链
  if multi_host_break_is no-toolchain-check; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=no-toolchain-check：不比两台工具链" >&2
  else
    peer_host_compare_toolchains >&2 || exit 255
    echo "  · ③ 两台的工具链相同（${peer_host_toolchain_text}）" >&2
  fi
  # ④ 树拷到第二台
  sop_source="$(peer_host_sop_source "$tree_root" "$tree_root")" \
    || multi_host_fail "树里与它的主仓里都没有规范副本 .claude/singlefs-ai-sop/（要有 scripts/preflight.py）：第二台那棵树的脚本一步都判不了准入" "在主仓里装好它（bash .claude/singlefs-ai-sop/install.sh）"
  local peer_runs="$PEER_REPOSITORY_DIRECTORY/runs"
  peer_run_directory="$peer_runs/$run_label"
  peer_pid_file="$peer_runs/$run_label.pid"
  peer_exit_file="$peer_runs/$run_label.exit"
  peer_output_file="$peer_runs/$run_label.log"
  peer_serial_file="$peer_runs/$run_label.serial"
  if ! peer_host_run / "test ! -e $(printf '%q' "$peer_run_directory") && test ! -e $(printf '%q' "$peer_pid_file") && test ! -e $(printf '%q' "$peer_exit_file") && test ! -e $(printf '%q' "$peer_output_file") && test ! -e $(printf '%q' "$peer_serial_file")" >/dev/null; then
    multi_host_usage_error "第二台 runs/ 下已经有 $run_label（或它的 .pid / .exit / .log / .serial），或第二台连不上"
  fi
  peer_host_run / "mkdir -p $(printf '%q' "$peer_run_directory")" >/dev/null \
    || multi_host_fail "第二台建不了 runs/$run_label" "看配置的 PEER_REPOSITORY_DIRECTORY 可不可写、盘满没满"
  peer_run_pending=1
  files_list="$multi_host_scratch/files"
  local tree_is_git=0
  if git -C "$tree_root" rev-parse --is-inside-work-tree >/dev/null 2>&1 || multi_host_break_is copy-requires-git; then
    tree_is_git=1
    git -C "$tree_root" ls-files -co --exclude-standard -z --deduplicate > "$multi_host_scratch/listed" \
      || multi_host_fail "git ls-files 列不出这棵树的文件" "在 git 工作树里跑（树根是这份脚本所在的仓）"
  else
    # 不是 git 仓的树（主 agent 与实现员的副本）：整棵树拷过去，照样不带 .git、target、target-*、本地配置与规范副本（规范副本另拷）
    ( cd "$tree_root" && find . -mindepth 1 \( -name .git -o -name target -o -name 'target-*' \) -prune -o \( -type f -o -type l \) -printf '%P\0' ) > "$multi_host_scratch/listed" \
      || multi_host_fail "列不出这棵树的文件" "看树根 $tree_root 读不读得了"
  fi
  # 跟踪着的文件（git ls-files -c）：拷之前排掉没跟踪的临时文件、拷的途中消失了的判不判失败都按它分（文件头 ④「拷的途中消失的文件」）；不是 git 仓的树一份都没有
  local -A tracked_paths=()
  local tracked_path temporary_skipped=0
  if [[ "$tree_is_git" == 1 ]]; then
    while IFS= read -r -d '' tracked_path; do tracked_paths[$tracked_path]=1; done \
      < <(git -C "$tree_root" ls-files -c -z --deduplicate 2>/dev/null)
  fi
  : > "$files_list"
  while IFS= read -r -d '' listed_path; do
    case "$listed_path" in multi-host.env|.claude/singlefs-ai-sop|.claude/singlefs-ai-sop/*) continue ;; esac
    # 没跟踪的临时文件（文件名带 .tmp.：别的会话改文件时先写的那一份，再 mv 换上）不拷：拷的途中它多半就没了。弄坏开关 tmp-files-copied 照改前也拷
    if [[ "${listed_path##*/}" == *.tmp.* && -z "${tracked_paths[$listed_path]:-}" ]] && ! multi_host_break_is tmp-files-copied; then
      temporary_skipped=$((temporary_skipped + 1))
      continue
    fi
    if [[ -e "$tree_root/$listed_path" || -L "$tree_root/$listed_path" ]]; then printf '%s\0' "$listed_path" >> "$files_list"; fi
  done < "$multi_host_scratch/listed"
  if multi_host_break_is sync-ignored-files; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=sync-ignored-files：树整个拷过去，被 git 忽略的文件也拷" >&2
    printf '%s\0' . > "$files_list"
  fi
  # 错误输出经管道落一份文件（peer_host_copy 的换字那一支跑完、管道才收尾，读得到全文）；rsync 的退出码落另一份（管道里的赋值传不回来）
  rm -f -- "$multi_host_scratch/tree-copy.status"
  # （外层那一组先把原来的标准输出存进 3 号，里层把标准错误接进管道、标准输出照旧走 3 号）
  { { peer_host_copy --exclude target --exclude .git --exclude /.claude/singlefs-ai-sop --from0 --files-from="$files_list" -r "$tree_root/" "$(peer_host_path "$peer_run_directory")/" 2>&1 1>&3 3>&-
      echo "$?" > "$multi_host_scratch/tree-copy.status"; } | tee "$multi_host_scratch/tree-copy.err" >&2; } 3>&1
  local tree_copy_status
  tree_copy_status="$(cat "$multi_host_scratch/tree-copy.status" 2>/dev/null)"
  if [[ "$tree_copy_status" == 24 ]] && multi_host_vanished_untracked_only "$tree_root" "$tree_is_git" "$multi_host_scratch/tree-copy.err"; then
    echo "  ! ④ 拷树时 ${#multi_host_vanished_paths[@]} 个没跟踪的文件在拷的途中消失了（rsync 退 24）：${multi_host_vanished_paths[*]}；它们不在 git 里，照跑" >&2
  elif [[ "$tree_copy_status" != 0 ]]; then
    multi_host_fail "树 rsync 到第二台失败（rsync 退 ${tree_copy_status:-?}${multi_host_vanished_reason:+；$multi_host_vanished_reason}）" \
      "退 24 是拷的途中有文件消失了：被跟踪的文件消失说明有人正在改这棵树，等它改完再跑；别的退出码看第二台盘满没满、配置的 PEER_REPOSITORY_DIRECTORY 可不可写"
  fi
  peer_host_copy "$sop_source/" "$(peer_host_path "$peer_run_directory/.claude/singlefs-ai-sop")/" \
    || multi_host_fail "规范副本 rsync 到第二台失败" "看第二台盘满没满"
  # init 之后先关那棵树的自动整理（文件头 ④「gc.auto 0」）：不关的话提交之后 git 在后台起 gc --auto，⑦ 删树时它还开着树里的文件
  local gc_auto_off="git config gc.auto 0 && "
  if multi_host_break_is peer-gc-auto-kept; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=peer-gc-auto-kept：第二台那棵树 git init 之后不设 gc.auto 0" >&2
    gc_auto_off=""
  fi
  peer_host_run "$peer_run_directory" "test -f .claude/singlefs-ai-sop/scripts/preflight.py && git init -q . && ${gc_auto_off}git add -A && git -c user.name=multi-host-run -c user.email=multi-host-run@localhost.invalid commit -q --no-verify --no-gpg-sign -m multi-host-run" >/dev/null \
    || multi_host_fail "第二台那棵树回读不到规范副本的 scripts/preflight.py，或 git init、提交失败" "看第二台那棵树可不可写、装没装 git"
  echo "  · ④ 树拷到第二台 runs/$run_label（git 列得出的 $(tr -cd '\0' < "$files_list" | wc -c) 个文件，没跟踪的临时文件排掉 $temporary_skipped 个，规范副本另拷），在那里 git init 并提交一次" >&2
  # ④ 之后：本机这一份多机配置原样带到那棵树的树根与第二台专用目录下常驻那一份，回读 sha256 对不上就拒（文件头 ④；scripts/hosts/peer-host-lib.sh）
  peer_host_carry_configuration "$peer_run_directory" >&2 || exit 255
  # ④ 之后：树根有 Cargo.toml 的，在第二台那棵树里空跑 cargo metadata --offline，缺离线依赖就拒（文件头 ④；peer-host-lib.sh 文件头「离线依赖」）
  peer_host_check_offline_dependencies "$peer_run_directory" >&2 || exit 255
  # ④ 之后：--send 点名的本机目录送到那棵树里（提交之后送：它不是树的一部分，例如上一次跑积累的判定库，让第二台复用已核过的块）
  for send_pair in ${send_pairs[@]+"${send_pairs[@]}"}; do
    peer_host_copy "${send_pair%%|*}/" "$(peer_host_path "$peer_run_directory/${send_pair#*|}")/" \
      || multi_host_fail "--send 的目录 ${send_pair%%|*} rsync 到第二台失败" "看第二台盘满没满、配置的 PEER_REPOSITORY_DIRECTORY 可不可写"
    echo "  · ④ 之后送到第二台那棵树的 ${send_pair#*|}：$(du -sh -- "${send_pair%%|*}" | cut -f1)" >&2
  done
  # ⑤ 在第二台跑
  # 命令里有 cargo（文件头 ⑤「RocksDB 预编库的取值」）：第二台起它之前先在那棵树根判第二台的取值（scripts/hosts/rocksdb-prebuilt.py entry），判红退 78、命令不跑
  if multi_host_command_starts_cargo "$@"; then
    if multi_host_break_is peer-rocksdb-unchecked; then
      echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=peer-rocksdb-unchecked：第二台起含 cargo 的命令之前不判 RocksDB 预编库的取值" >&2
    else
      set -- bash -c 'python3 scripts/hosts/rocksdb-prebuilt.py entry --root . || { echo "  ✗ 第二台起 cargo 之前 RocksDB 预编库的取值没判过（上面 rocksdb-prebuilt.py entry 的红句），命令没跑（退 78）"; echo "  → 怎么办：在第二台上照上面的出路修（第二台没装就在那边跑 python3 scripts/hosts/rocksdb-prebuilt.py install；装了而 ssh 起的命令拿不到取值，看第二台 ~/.bashrc 最前面那一段），再跑同一条命令"; exit 78; }; exec "$@"' \
        multi-host-run-rocksdb-entry "$@"
      echo "  · ⑤ 命令里有 cargo：第二台起它之前先判第二台的 RocksDB 预编库取值（scripts/hosts/rocksdb-prebuilt.py entry），判红退 78、命令不跑" >&2
    fi
  fi
  if [[ -n "${RUN_WITH_MEMORY_CAP_RESERVE:-}" ]]; then reserve_text="RUN_WITH_MEMORY_CAP_RESERVE=$(printf '%q' "$RUN_WITH_MEMORY_CAP_RESERVE") "; fi
  command_text="${reserve_text}bash scripts/hosts/run-with-memory-cap.sh $(printf '%q' "$memory_cap") $(printf '%q ' "$@")"
  if multi_host_break_is peer-without-memory-cap; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=peer-without-memory-cap：第二台那条命令不经 run-with-memory-cap.sh 跑" >&2
    command_text="$(printf '%q ' "$@")"
  fi
  # 程序当场声明串行段的文件（文件头 ⑤「在用核数」）
  if multi_host_break_is serial-declarations-unset; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=serial-declarations-unset：第二台那条命令不带 SINGLEFS_SERIAL_DECLARATIONS" >&2
  else
    command_text="SINGLEFS_SERIAL_DECLARATIONS=$(printf '%q' "$peer_serial_file") $command_text"
  fi
  # 第二台认得自己是第二台、读的是 ④ 带过去的那一份配置、起活这一侧清没清场（文件头 ⑤「带到第二台的环境」）
  command_text="$(peer_host_role_environment)$command_text"
  peer_command_started=1   # 先置上：起的那一刻收到信号，也按第二台上记下的进程号去停
  peer_host_start_detached "$peer_run_directory" "$peer_pid_file" "$peer_exit_file" "$peer_output_file" "$command_text" >/dev/null \
    || multi_host_fail "第二台那条命令起不来" "看 ssh 连不连得上、第二台那棵树在不在"
  echo "  · ⑤ 第二台开跑（脱离 ssh 会话起，经副本里的 run-with-memory-cap.sh $memory_cap），每 $poll_seconds 秒看一次" >&2
  core_next_check=$(( SECONDS + core_seconds ))
  PEER_HOST_WAIT_HOOK=multi_host_core_check
  peer_host_wait_detached "$peer_pid_file" "$peer_exit_file" "$poll_seconds" >&2
  PEER_HOST_WAIT_HOOK=""
  peer_command_started=0
  if peer_host_copy "$(peer_host_path "$peer_output_file")" "$multi_host_scratch/peer.log" 2>/dev/null; then
    multi_host_redact < "$multi_host_scratch/peer.log"
  else
    echo "  ! 第二台那条命令的输出没拷回来" >&2
  fi
  case "$peer_host_detached_outcome" in
    "exit "*) command_exit="${peer_host_detached_outcome#exit }" ;;
    vanished) multi_host_broadcast_peer_finished ""   # 那条命令没了（没留退出码）：照样算跑完，本机代发（文件头「资源释放广播」）
      multi_host_fail "第二台那条命令没等到退出码：$peer_host_detached_outcome（vanished 进程没了又没留退出码；unreachable 连续 60 次 ssh 不通；never-started 60 次都没起来）" "看上面拷回的输出；ssh 到第二台 runs/ 下看 $run_label.log" ;;
    *) multi_host_fail "第二台那条命令没等到退出码：$peer_host_detached_outcome（vanished 进程没了又没留退出码；unreachable 连续 60 次 ssh 不通；never-started 60 次都没起来）" "看上面拷回的输出；ssh 到第二台 runs/ 下看 $run_label.log" ;;
  esac
  echo "  · 第二台那条命令退 $command_exit" >&2
  multi_host_broadcast_peer_finished "$command_exit"   # 第二台那一趟跑完：本机代发（文件头「资源释放广播」）
  if multi_host_break_is peer-exit-dropped; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=peer-exit-dropped：退出码不带回，一律退 0" >&2
    command_exit=0
  fi
  # ⑥ 取回、⑦ 删第二台这一趟的树
  multi_host_fetch
  multi_host_remove_peer_run
  multi_host_finish "$command_exit"
}

multi_host_main() {
  peer_mode=0; memory_cap=""; run_label=""; fetch_paths=(); fetch_missing=(); send_pairs=()
  local separator_seen=0 fetch_path send_pair
  while (( $# > 0 )); do
    case "$1" in
      --peer) peer_mode=1; shift ;;
      --memory-cap) (( $# >= 2 )) || multi_host_usage_error "--memory-cap 后面没给上限"; memory_cap="$2"; shift 2 ;;
      --label) (( $# >= 2 )) || multi_host_usage_error "--label 后面没给标签"; run_label="$2"; shift 2 ;;
      --fetch) (( $# >= 2 )) || multi_host_usage_error "--fetch 后面没给路径"; fetch_paths+=("$2"); shift 2 ;;
      --send) (( $# >= 3 )) || multi_host_usage_error "--send 后面要给 <本机目录> <树里的相对路径>"; send_pairs+=("$2|$3"); shift 3 ;;
      --) separator_seen=1; shift; break ;;
      *) multi_host_usage_error "认不出的参数「$1」（命令要写在 -- 后面）" ;;
    esac
  done
  (( separator_seen && $# > 0 )) || multi_host_usage_error "-- 后面没有命令"
  if [[ -n "$memory_cap" && ! "$memory_cap" =~ ^[1-9][0-9]*[KMGT]$ ]]; then
    multi_host_usage_error "--memory-cap 读到「$memory_cap」，要写正整数加 K / M / G / T（单位必写，例 8G）"
  fi
  [[ -n "$run_label" ]] || run_label="multi-host-$(date -u +%Y%m%dT%H%M%SZ)-$$"
  [[ "$run_label" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ ]] || multi_host_usage_error "--label 读到「$run_label」"
  for fetch_path in ${fetch_paths[@]+"${fetch_paths[@]}"}; do
    if [[ -z "$fetch_path" || "$fetch_path" == /* || "$fetch_path" =~ (^|/)\.\.(/|$) ]]; then
      multi_host_usage_error "--fetch 读到「$fetch_path」：要写树里的相对路径，不许绝对路径与 .."
    fi
  done
  for send_pair in ${send_pairs[@]+"${send_pairs[@]}"}; do
    (( peer_mode )) || multi_host_usage_error "--send 只在 --peer 时有意义（本机模式命令就在仓根跑，没有第二棵树可送）"
    local send_source="${send_pair%%|*}" send_relative="${send_pair#*|}"
    [[ -d "$send_source" ]] || multi_host_usage_error "--send 读到「$send_source」：要是本机已有的目录"
    if [[ -z "$send_relative" || "$send_relative" == /* || "$send_relative" =~ (^|/)\.\.(/|$) ]]; then
      multi_host_usage_error "--send 的落点读到「$send_relative」：要写树里的相对路径，不许绝对路径与 .."
    fi
  done
  # 不接跨趟共用的编译目录（文件头 ⑤「编译目录」）：设了 MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY、--peer 命令里 CARGO_TARGET_DIR 指到树外，都是用法错；
  # 弄坏开关 outside-target-accepted 下照跑（那个变量读完 unset、不起作用）
  if [[ -n "${MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY:-}" ]] && ! multi_host_break_is outside-target-accepted; then
    echo "  ✗ multi-host-run.sh 用法错：设了 MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY=$MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY：跨趟共用的编译目录不再接（每一趟自己的编译目录跑完删），命令没跑" >&2
    echo "     → 怎么办：调用方去掉这个变量，命令里也不设指到树外的 CARGO_TARGET_DIR；第二台那条命令在这一趟的树里编，⑦ 连同树删" >&2
    exit 2
  fi
  unset MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY
  if (( peer_mode )) && multi_host_outside_target_in "$@" && ! multi_host_break_is outside-target-accepted; then
    echo "  ✗ multi-host-run.sh 用法错：--peer 的命令里 CARGO_TARGET_DIR 指到树外（值以 / 或 .. 开头）：那是跨趟共用的编译目录，⑦ 删树时删不到它，命令没跑" >&2
    echo "     → 怎么办：命令里去掉这个赋值（或指到树里的相对路径），让第二台在这一趟的树里编、跑完连同树删" >&2
    exit 2
  fi
  # 命令里的路径（文件头 ⑤）：--peer 的命令里有以本机仓根起头的参数，第二台那棵树不在这个路径上，拒；弄坏开关 peer-absolute-path-accepted 下照跑
  #   弄坏开关 local-absolute-path-refused 下不带 --peer 的本机跑也判（本机仓根下的路径在本机存在，不该拦）
  if { (( peer_mode )) || multi_host_break_is local-absolute-path-refused; } && multi_host_local_root_path_in "$@" && ! multi_host_break_is peer-absolute-path-accepted; then
    echo "  ✗ multi-host-run.sh 用法错：--peer 的命令里有以本机仓根起头的参数「$multi_host_local_root_word」：第二台那棵树不在这个路径上，那边没有这条路径，命令会退 127；什么都没拷到第二台，命令没跑" >&2
    echo "     → 怎么办：把命令里以本机仓根起头的路径改成树内相对路径（仓根本身写 .），例如 bash .claude/gate.d/x.sh --check a .；命令在第二台那棵树的树根起" >&2
    exit 2
  fi
  # 取回目录（文件头 ⑥）：没设 MULTI_HOST_RUN_FETCH_DIRECTORY 就落在工作根下
  fetch_directory=""
  if [[ -n "${MULTI_HOST_RUN_FETCH_DIRECTORY:-}" ]]; then
    fetch_directory="$MULTI_HOST_RUN_FETCH_DIRECTORY"
  elif multi_host_break_is fetch-under-tmp; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=fetch-under-tmp：默认取回目录退回 \${TMPDIR:-/tmp} 下" >&2
    fetch_directory="${TMPDIR:-/tmp}/singlefs-multi-host-run/$run_label"
  elif multi_host_work_root="$(python3 "$multi_host_script_directory/../../.claude/hooks/lib_work_root.py")" && [[ -n "$multi_host_work_root" ]]; then
    fetch_directory="$multi_host_work_root/multi-host-run/$run_label"
  elif (( ${#fetch_paths[@]} > 0 )); then
    multi_host_fail "取不到工作根（.claude/hooks/lib_work_root.py 跑不起来），--fetch 的取回目录没处放：命令没跑" \
      "恢复那份库，或设 MULTI_HOST_RUN_FETCH_DIRECTORY 指到开机不清的可写空目录"
  fi
  if (( ${#fetch_paths[@]} > 0 )) && [[ -e "$fetch_directory" ]] && [[ -n "$(ls -A -- "$fetch_directory" 2>/dev/null)" ]]; then
    multi_host_usage_error "取回目录 $fetch_directory 里已经有东西（换一个 --label，或删掉它、或把 MULTI_HOST_RUN_FETCH_DIRECTORY 指到空目录）"
  fi
  poll_seconds="${MULTI_HOST_RUN_POLL_SECONDS:-5}"
  [[ "$poll_seconds" =~ ^[1-9][0-9]*$ ]] || multi_host_usage_error "MULTI_HOST_RUN_POLL_SECONDS 读到「$poll_seconds」，要写正整数秒"
  core_seconds="${MULTI_HOST_RUN_CORE_SECONDS:-300}"
  if [[ ! "$core_seconds" =~ ^[1-9][0-9]*$ ]] || (( core_seconds > 300 )); then
    multi_host_usage_error "MULTI_HOST_RUN_CORE_SECONDS 读到「$core_seconds」，要写不超过 300 的正整数秒（只许往短里改，第二台在用几个核不许调成不看）"
  fi
  core_next_check=0
  tree_root="$(cd "$multi_host_script_directory/../.." && pwd)"
  local_command_process=""
  peer_command_started=0
  peer_run_pending=0
  multi_host_cleanup_failed=0
  multi_host_local_target=""
  multi_host_scratch="$(mktemp -d)"
  trap multi_host_on_exit EXIT
  trap 'multi_host_stop_and_exit TERM 143' TERM
  trap 'multi_host_stop_and_exit INT 130' INT
  trap 'multi_host_stop_and_exit HUP 129' HUP
  if (( peer_mode )); then
    multi_host_peer_main "$@"
    peer_mode=0   # 只有弄坏开关 switch-off-falls-back-to-local 走得到这里
  fi
  [[ -n "$memory_cap" ]] || memory_cap="$MULTI_HOST_LOCAL_DEFAULT_MEMORY_CAP"
  multi_host_local_main "$@"
}

# ── 自证 ──────────────────────────────────────────────────────────────────────
# 在临时目录里搭一棵小仓（本仓的 .claude/scripts、scripts、规范副本的符号链接、一个被 git 忽略的 .env），
# 「第二台」是本机上的另一个目录（SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1），配置写在场景目录里（SINGLEFS_MULTI_HOST_CONFIG）。
# 命令都经真的 run-with-memory-cap.sh（上限 256M）。逐格（弄坏开关见文件头，各让对应那一格判错）：
#   Ⓐ 默认本机跑：配置是坏的、调用方的会话里已有一条日常的在跑（自证用假会话号经真包装占一条 16M 的 sleep，Ⓐ 带同一个会话号起）也照跑、退 0、
#      在本机仓根跑、第二台一个目录都没建；经 run-with-memory-cap.sh（cgroup 在 singlefs-memory-cap- 的 scope 里）；
#      --fetch 取回的文件与仓根那一份逐字相同
#   Ⓑ --peer 跑成：退 0，在第二台 runs/<标签>/ 那棵副本里跑（副本里有规范副本、提交过一次），被 git 忽略的 .env 没拷过去；经副本里的 run-with-memory-cap.sh 跑；
#      命令退 7 ⇒ 驱动退 7；--memory-cap 比 PEER_MEMORY_CAP 大 ⇒ 拒（退 2）、命令没跑
#   Ⓒ 第二台的 rustc -Vv 不同 ⇒ 拒（退 255，说两台的工具链不同），命令没跑、第二台没建这一趟的目录
#      第二台的 PATH 最前面放假 rustup（component list --installed 打本机那一份去掉 clippy 那一行）⇒ 拒（退 255，说两台的工具链不同、第二台缺组件 clippy），命令没跑、没建目录
#      离线依赖（peer-host-lib.sh 文件头「离线依赖」）：小仓树根放一份 Cargo.toml，第二台的 PATH 最前面放假 cargo（metadata 记下目录与参数；场景里有 metadata-missing
#      就照 cargo 的原话报 no matching package named `vstd`、退 101；-V 这类转给真 cargo，工具链照比）：报缺 ⇒ 退 255、说第二台缺依赖 vstd 与 cargo fetch 的同步办法、
#      命令没跑、这一趟的树删了；不报缺 ⇒ 照跑、退 0，假 cargo 记下在第二台这一趟的树根跑过 metadata --offline
#   Ⓓ --fetch 一个文件、一个目录：取回的逐字相同（与命令在第二台上算的 sha256 对）
#   Ⓔ Ⓑ Ⓓ 那几趟跑完（含退 7 那一趟）第二台 runs/ 下什么都不剩
#   Ⓕ 第二台那条命令在跑时给驱动（只给它自己，kill "$!"）发 TERM ⇒ 120 秒内退 143，第二台那条命令不在了，runs/ 下什么都不剩；
#      第二台那条命令只握 0、1、2（不握驱动那一头的管道：握着的话，起它那一步要等它跑完才回来）
# 场景里起驱动一律清掉 SINGLEFS_HEAVY_TESTS；会话号 CLAUDE_CODE_SESSION_ID 照调用方给的带下去：内存包装 2026-10-06 撤了「同一个会话同时只许一条」，
#   Ⓐ 在占着一条的同一个会话里照跑，就是核那道拒绝没再出现
#   Ⓖ 输出（驱动的标准输出与标准错误）里没有配置里的主机名与第二台的目录，而命令打的那两样换成了「第二台」「第二台的目录」
#   Ⓗ 配置里没写 ENABLE_ACROSS_MACHINES ⇒ --peer 拒（退 255，说在配置里写 ENABLE_ACROSS_MACHINES=1），命令哪台都没跑
#   Ⓘ 弄坏开关设在自证之外（起驱动时去掉自证的标记）⇒ 拒（退 2），命令没跑
#   Ⓙ 本机跑 --fetch，起驱动时去掉 MULTI_HOST_RUN_FETCH_DIRECTORY、SINGLEFS_WORK_ROOT 与 TMPDIR 各指进场景目录 ⇒ 取回到工作根下 multi-host-run/<标签>/、
#      $TMPDIR 下没有；再设 MULTI_HOST_RUN_FETCH_DIRECTORY 跑一趟 ⇒ 取回到它点名的目录，工作根下没有那一趟（小仓里另拷了 .claude/hooks/lib_work_root.py）
#   Ⓚ 本机跑 bash -c 'exit 3'，内存包装的 journalctl 换成假的（RUN_WITH_MEMORY_CAP_JOURNALCTL，打一行 NVRM … NV_ERR_NO_MEMORY）⇒ 包装退 245，
#      驱动原样退 245、标准错误里报「显存不够」与「不算结果」
#   Ⓜ 不接跨趟共用的编译目录（文件头 ⑤「编译目录」）：设了 MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY ⇒ 用法错（退 2），命令没跑；
#      --peer 命令里 env CARGO_TARGET_DIR=../shared-target、bash -c 脚本里 export CARGO_TARGET_DIR=/var/tmp/shared-target ⇒ 都是用法错（退 2），命令没跑
#   Ⓠ 本机跑的编译目录（文件头「本机模式的编译目录」）：命令记下拿到的 CARGO_TARGET_DIR、往里写一份产物；正常退 0、出错退 7、被 TERM 停（退 143）跑完那个目录都不在，
#      stderr 有删了它那一行；调用方设了 CARGO_TARGET_DIR 的照它编、跑完不删；--peer 时命令在树里建的 target 跑完连同树删，第二台那条命令不带本机的 CARGO_TARGET_DIR
#   Ⓡ 删之前核开着的文件：命令记下编译目录（本机）或树根（--peer）后等放行文件，自证在那里起一个 sleep 开着一份文件再放行 ⇒ 驱动不删、报出那个进程、退 255
#   Ⓞ Ⓟ 拷的途中消失的文件（文件头 ④）：PATH 最前面放假 rsync（照真的拷，拷树那一趟拷完照 rsync 的原话报 SELFTEST_VANISHED 里的路径消失了、退 24），
#      两个场景都把 payload/input.txt 加进索引（被跟踪），另放没跟踪的 scripts/kb/sample.py.tmp.4242 与 untracked-note.txt：
#      Ⓞ 报 untracked-note.txt 消失 ⇒ 照跑、退 0、打出消失了哪个，临时文件没拷过去；Ⓟ 报 payload/input.txt 消失 ⇒ 退 255、说被跟踪的文件消失了，命令没跑
#   Ⓝ 在用核数（文件头 ⑤）：MULTI_HOST_RUN_CORE_SECONDS=1、SINGLEFS_CORE_USAGE_WINDOW_SECONDS=2、告警记录写场景目录，第二台那条命令是 core_usage.py busy-load，
#      这一场景的 core_usage.py 换成假的（busy-load 只记线程数与起点、睡够秒数，sample 照记下的线程数报样本，judge 与 declare-serial 交给真的），结论不随此刻负载：
#      1 条线程忙 6 秒 ⇒ 打「第二台在用 X 核（登记 N）」与 ✗ 一行带出路、告警记录带会话号与来源；登记数那么多条线程忙 6 秒 ⇒ 打那一行、不打 ✗、不写告警记录；
#      先 declare-serial 再 1 条线程忙 6 秒 ⇒ ✗ 那一行标出声明的理由
#   Ⓣ 自证的细节行按字截（cut_characters）：200 个汉字截到 161 个字，截出来是合法的 UTF-8
#   Ⓞ RocksDB 预编库的取值（文件头 ⑤）：第二台的 PATH 前面放不带 entry 认的标记的假 cargo（-V 转给真 cargo，别的建一份「跑了」的文件），四个取值在本机当第二台的环境里都没有，
#      那棵树的 scripts/experiment-apparatus/Cargo.toml 依赖 rocksdb：-- cargo build ⇒ 退 78、假 cargo 没起、拷回的输出里有 rocksdb-prebuilt entry 的红句；-- bash -c 'touch …' ⇒ 不判、退 0。
#   Ⓤ 两台用同一份多机配置（文件头 ④ ⑤）：第二台那条命令记下的角色是 peer、SINGLEFS_MULTI_HOST_CONFIG 是这一趟树根下那一份、sha256 与场景的配置相同，第二台专用目录下常驻那一份也相同
#   Ⓥ PATH 最前面放假 rsync（目标是 …/multi-host.env 的那一趟拷完往末尾加一行）⇒ 两台的配置不同：退 255、命令没跑、这一趟的树删了
#   Ⓦ 第二台的 PATH 最前面放假 ssh（被起一次记一行）：第二台那条命令在角色 peer 之下单跑判法退 3、--emit-assignments 退 3 不打赋值、里层 multi-host-run.sh --peer 退 255，假 ssh 一次没被起；
#      角色写成 Peer ⇒ 判法退 1、点名这个变量
#   Ⓧ 驱动正经 gpu-quiesce-run.sh 起（GPU_QUIESCE_HOLDER 指的登记文件名字以驱动的父进程号起头）⇒ 第二台拿到 GPU_QUIESCE_ORIGIN_READ_BACK=1；进程号对不上的登记文件 ⇒ 没设
#   Ⓨ 判法报两台配置一不一致：Ⓤ 跑过之后报一致、场景的配置改了之后报不一致、没跑过 --peer 的场景报第二台还没有常驻那一份
#   Ⓩ 命令里的路径（文件头 ⑤「命令里的路径」）：--peer 的命令里有一个词是本机仓根、或是仓根下的文件 ⇒ 退 2、说以本机仓根起头与树内相对路径，命令没跑、第二台 runs/ 下什么都没建；
#      命令里只有仓根之外的路径、名字只是以仓根起头的路径（<仓根>-旁边） ⇒ 照跑、退 0；不带 --peer 的本机跑带仓根下的绝对路径 ⇒ 照跑、退 0
# 自证在临时目录里放标记文件、export MULTI_HOST_RUN_SELFTEST_MARK 指向它：场景里起的驱动认得出是自证起的，调用方设的弄坏开关才生效

multi_host_selftest() {
  local repository_root work template sop_source checked=0 failures=0
  repository_root="$(cd "$multi_host_script_directory/../.." && pwd)"
  work="$(mktemp -d)"
  multi_host_selftest_work="$work"
  printf '%s\n' "$MULTI_HOST_SELFTEST_MARK_CONTENT" > "$work/selftest-mark"
  export MULTI_HOST_RUN_SELFTEST_MARK="$work/selftest-mark"
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

  # 小仓的模板
  template="$work/template"
  mkdir -p "$template/.claude/hooks" "$template/research" "$template/payload"
  rsync -a --exclude __pycache__ "$repository_root/.claude/scripts/" "$template/.claude/scripts/"
  cp -- "$repository_root/.claude/hooks/lib_work_root.py" "$template/.claude/hooks/lib_work_root.py"   # 取回目录的默认值要它（Ⓙ）
  cp -- "$repository_root/.claude/heavy-thread-default" "$template/.claude/heavy-thread-default"   # core_usage.py judge 取登记数要它（Ⓝ）
  rsync -a --exclude __pycache__ --exclude /experiment-apparatus/target/ "$repository_root/scripts/" "$template/scripts/"   # 装置的编译目录几十 G，自证用不到
  ln -s "$sop_source" "$template/.claude/singlefs-ai-sop"
  printf '/.claude/singlefs-ai-sop\n__pycache__/\n/multi-host.env\n/.env\ntarget/\n' > "$template/.gitignore"
  printf '多机脚本自证的输入\n' > "$template/payload/input.txt"
  printf 'SELFTEST_SECRET=被忽略的文件不许拷到第二台\n' > "$template/.env"

  prepare_scene() { # <场景名> [配置里 ENABLE_ACROSS_MACHINES 那一行，默认 ENABLE_ACROSS_MACHINES=1]：repo（git init 过）、peer-home、peer-bin、配置
    local scene="$work/$1"
    mkdir -p "$scene/peer-home" "$scene/peer-bin"
    rsync -a "$template/" "$scene/repo/"
    git -C "$scene/repo" init -q
    {
      echo "PEER_SSH_HOST=selftest-multi-host-peer-name"
      echo "PEER_REPOSITORY_DIRECTORY=$scene/peer-home"
      echo "PEER_CARGO_BIN_DIRECTORY=$scene/peer-bin"
      echo "PEER_MEMORY_CAP=1G"
      echo "${2-ENABLE_ACROSS_MACHINES=1}"
      echo "QUIESCE_STOP_COMMAND="
      echo "QUIESCE_STOPPED_CHECK_COMMAND="
      echo "QUIESCE_START_COMMAND="
      echo "QUIESCE_STARTED_CHECK_COMMAND="
    } > "$scene/multi-host.env"
  }
  scene_command_for() { # <场景名>：scene_command 设成在这个场景里起驱动的前缀（弄坏开关从调用方的环境里留着；SINGLEFS_HEAVY_TESTS 清掉，见上面的说明）；
    # 驱动的临时目录（本机那一趟的编译目录在里面）落在自证的临时目录下，弄坏开关留下的随自证删
    local scene="$work/$1"
    mkdir -p "$work/scene-tmp"
    # 角色与清场的握手变量不从调用方带进来（自证在第二台上跑时调用方的环境里有 SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE=peer）；要的格自己给
    scene_command=(env -u SINGLEFS_HEAVY_TESTS -u RUN_WITH_MEMORY_CAP_RESERVE -u RUN_WITH_MEMORY_CAP_KEY -u SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE -u GPU_QUIESCE_HOLDER -u GPU_QUIESCE_ORIGIN_READ_BACK
      SINGLEFS_MULTI_HOST_CONFIG="$scene/multi-host.env" SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1
      MULTI_HOST_RUN_POLL_SECONDS=1 MULTI_HOST_RUN_FETCH_DIRECTORY="$scene/fetch" RUN_WITH_MEMORY_CAP_PEAKS="$scene/peaks.tsv" TMPDIR="$work/scene-tmp"
      bash "$scene/repo/scripts/hosts/multi-host-run.sh")
  }
  run_in() { # <场景名> <这一趟的名字> <驱动的参数…>：stdout、stderr、退出码各写场景目录一个文件
    local scene="$work/$1" run_name="$2" exit_code
    shift 2
    scene_command_for "${scene##*/}"
    if "${scene_command[@]}" "$@" > "$scene/$run_name.out" 2> "$scene/$run_name.err" < /dev/null; then exit_code=0; else exit_code=$?; fi
    echo "$exit_code" > "$scene/$run_name.exit"
  }
  exit_of() { cat "$work/$1/$2.exit" 2>/dev/null; }
  tail_of() { tail -n 3 "$work/$1/$2.err" 2>/dev/null | tr '\n' '|'; }
  cut_characters() { # <字数>：每行只留前这么多个字（按 UTF-8 的字截：GNU cut -c 按字节截，会把汉字截成半个，细节行里留下解不成 UTF-8 的字节）；
    # 弄坏开关 detail-cut-by-bytes 下照改前按字节截（Ⓣ 那一格红）
    if multi_host_break_is detail-cut-by-bytes; then cut -c "1-$1"; return; fi
    python3 -c 'import sys; limit = int(sys.argv[1]); sys.stdout.buffer.write(b"".join((line.decode("utf-8", "replace").rstrip("\n")[:limit] + "\n").encode("utf-8") for line in sys.stdin.buffer))' "$1"
  }
  peer_runs_left() { # <场景名>：第二台 runs/ 下留着的条目数
    find "$work/$1/peer-home/runs" -mindepth 1 -maxdepth 1 2>/dev/null | wc -l
  }

  # Ⓐ 默认本机跑：配置是坏的（判法一读就拒），不带 --peer 照跑。调用方的会话里已有一条日常的在跑：自证用假会话号经真包装占一条
  #   （16M 的 sleep，明写等待上限，slice 一时放不下就排队），Ⓐ 带同一个会话号起，照跑（内存包装的弄坏开关 sessiononejob 装回那道拒绝时 Ⓐ 退 252）
  local scene holder_pid holder_sleep="" holder_rounds fake_session="multi-host-run-selftest-$$"
  prepare_scene local
  scene="$work/local"
  env -u SINGLEFS_HEAVY_TESTS CLAUDE_CODE_SESSION_ID="$fake_session" RUN_WITH_MEMORY_CAP_WAIT_SECONDS=600 RUN_WITH_MEMORY_CAP_PEAKS="$scene/peaks.tsv" \
    RUN_WITH_MEMORY_CAP_KEY="multi-host-run.sh 自证 Ⓐ：同一个会话里先占着的一条" \
    bash "$template/scripts/hosts/run-with-memory-cap.sh" 16M bash -c 'echo "$$" > "$0"; exec sleep 300' "$scene/holder.pid" > "$scene/holder.out" 2>&1 &
  holder_pid=$!
  for (( holder_rounds = 0; holder_rounds < 6100; holder_rounds++ )); do
    [[ -s "$scene/holder.pid" ]] && break
    peer_host_process_is_alive "$holder_pid" || break
    sleep 0.1
  done
  holder_sleep="$(cat "$scene/holder.pid" 2>/dev/null)"
  printf '这一行不是 KEY=值：默认本机跑读了它就会被拒\n' > "$scene/multi-host.env"
  CLAUDE_CODE_SESSION_ID="$fake_session" run_in local run --memory-cap 256M --fetch out/local.txt -- bash -c "pwd > $(printf '%q' "$scene/where"); cat /proc/self/cgroup > $(printf '%q' "$scene/cgroup"); mkdir -p out && printf '本机产物\n' > out/local.txt"
  # 占位那一条只停它自己记下的 sleep（核过进程名），包装随之收尾
  if [[ -n "$holder_sleep" && "$(cat "/proc/$holder_sleep/comm" 2>/dev/null)" == sleep ]]; then kill "$holder_sleep"; fi
  wait "$holder_pid"
  expect "Ⓐ 默认本机跑：配置是坏的、调用方的会话里已有一条日常的在跑也照跑、退 0、在本机仓根跑、第二台一个目录都没建" \
    "$([[ -n "$holder_sleep" && "$(exit_of local run)" == 0 && "$(cat "$scene/where" 2>/dev/null)" == "$scene/repo" && ! -e "$scene/peer-home/runs" ]]; echo $?)" \
    "退 $(exit_of local run)，在「$(cat "$scene/where" 2>/dev/null)」跑；占位那一条$([[ -n "$holder_sleep" ]] && echo "在跑（sleep $holder_sleep）" || echo "没起来：$(tail -n 2 "$scene/holder.out" 2>/dev/null | tr '\n' '|')")；stderr 尾部：$(tail_of local run)"
  expect "Ⓐ 默认本机跑经 run-with-memory-cap.sh（cgroup 在 singlefs-memory-cap- 的 scope 里）" \
    "$(grep -q 'singlefs-memory-cap-' "$scene/cgroup" 2>/dev/null; echo $?)" \
    "命令记下的 cgroup：$(tr '\n' ' ' < "$scene/cgroup" 2>/dev/null)"
  expect "Ⓐ 默认本机跑 --fetch：取回的与仓根那一份逐字相同" \
    "$(cmp -s "$scene/repo/out/local.txt" "$scene/fetch/out/local.txt"; echo $?)" \
    "$(cmp "$scene/repo/out/local.txt" "$scene/fetch/out/local.txt" 2>&1 | head -1)"

  # Ⓑ --peer 跑成、退出码带回、上限不许比 PEER_MEMORY_CAP 大
  prepare_scene peer
  scene="$work/peer"
  run_in peer run --peer --memory-cap 256M --label peer-ok -- bash -c "pwd > $(printf '%q' "$scene/where"); cat /proc/self/cgroup > $(printf '%q' "$scene/cgroup"); \
if test -e .env; then echo synced > $(printf '%q' "$scene/leaked"); fi; test -f .claude/singlefs-ai-sop/scripts/preflight.py && git rev-parse -q --verify HEAD > $(printf '%q' "$scene/head"); \
git config --get gc.auto > $(printf '%q' "$scene/gc-auto"); \
pwd; echo selftest-multi-host-peer-name"
  expect "Ⓑ --peer 跑成：退 0，在第二台 runs/<标签>/ 那棵副本里跑（有规范副本、提交过一次），被 git 忽略的 .env 没拷过去" \
    "$([[ "$(exit_of peer run)" == 0 && "$(cat "$scene/where" 2>/dev/null)" == "$scene/peer-home/runs/peer-ok" && -s "$scene/head" && ! -e "$scene/leaked" ]]; echo $?)" \
    "退 $(exit_of peer run)，在「$(cat "$scene/where" 2>/dev/null)」跑，提交 $(cat "$scene/head" 2>/dev/null || echo 没有)，.env $([[ -e "$scene/leaked" ]] && echo 拷过去了 || echo 没拷)；stderr 尾部：$(tail_of peer run)"
  expect "Ⓑ --peer 经副本里的 run-with-memory-cap.sh 跑（cgroup 在 singlefs-memory-cap- 的 scope 里）" \
    "$(grep -q 'singlefs-memory-cap-' "$scene/cgroup" 2>/dev/null; echo $?)" \
    "命令记下的 cgroup：$(tr '\n' ' ' < "$scene/cgroup" 2>/dev/null)"
  expect "Ⓑ ④ 第二台那棵树关了自动整理：那条命令在树里读到 git config gc.auto 为 0" \
    "$([[ "$(exit_of peer run)" == 0 && "$(cat "$scene/gc-auto" 2>/dev/null)" == 0 ]]; echo $?)" \
    "git config gc.auto 读到「$(cat "$scene/gc-auto" 2>/dev/null)」（没设就是空的），退 $(exit_of peer run)；stderr 尾部：$(tail_of peer run)"
  run_in peer seven --peer --memory-cap 256M --label peer-seven -- bash -c 'exit 7'
  expect "Ⓑ --peer 退出码原样带回：第二台那条命令退 7 ⇒ 驱动退 7" \
    "$([[ "$(exit_of peer seven)" == 7 ]]; echo $?)" "退 $(exit_of peer seven)；stderr 尾部：$(tail_of peer seven)"
  run_in peer too-big --peer --memory-cap 2G --label peer-too-big -- bash -c "touch $(printf '%q' "$scene/too-big-ran")"
  expect "Ⓑ --memory-cap 2G 比配置的 PEER_MEMORY_CAP 1G 大 ⇒ 拒（退 2），命令没跑" \
    "$([[ "$(exit_of peer too-big)" == 2 && ! -e "$scene/too-big-ran" ]] && grep -q 'PEER_MEMORY_CAP' "$scene/too-big.err"; echo $?)" \
    "退 $(exit_of peer too-big)，命令$([[ -e "$scene/too-big-ran" ]] && echo 跑了 || echo 没跑)；stderr 尾部：$(tail_of peer too-big)"

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
  rustup component list --installed > "$scene/local-components" 2>/dev/null
  grep -v '^clippy' "$scene/local-components" > "$scene/peer-components"
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

  # Ⓒ 离线依赖：小仓树根放一份 Cargo.toml（没跟踪、没被忽略：照样拷过去），第二台的 PATH 最前面放假 cargo。
  #   有 metadata-missing 那一趟照 cargo 的原话报缺 vstd、退 101；去掉它再跑一趟照跑
  prepare_scene offline
  scene="$work/offline"
  printf '[workspace]\nmembers = []\n' > "$scene/repo/Cargo.toml"
  command -v cargo > "$scene/real-cargo"
  cat > "$scene/peer-bin/cargo" <<'FAKE_CARGO'
#!/usr/bin/env bash
# 自证用的 cargo（Ⓒ 离线依赖）：metadata 记下目录与参数，场景目录里有 metadata-missing 就照 cargo 的原话报缺 vstd、退 101；别的用法转给真 cargo（-V 照比工具链）
scene_directory="$(cd "$(dirname "$0")/.." && pwd)"
if [[ "${1:-}" == metadata ]]; then
  printf '%s %s\n' "$PWD" "$*" >> "$scene_directory/metadata-calls"
  if [[ -e "$scene_directory/metadata-missing" ]]; then
    printf '%s\n' 'error: no matching package named `vstd` found' 'location searched: registry `crates-io`' \
      "required by package \`singlefs-replay-prefix v0.1.0 ($PWD/crates/singlefs-replay-prefix)\`" \
      "As a reminder, you're using offline mode (--offline) which can sometimes cause surprising resolution failures," >&2
    exit 101
  fi
  exit 0
fi
exec "$(cat "$scene_directory/real-cargo")" "$@"
FAKE_CARGO
  chmod +x "$scene/peer-bin/cargo"
  : > "$scene/metadata-missing"
  run_in offline missing --peer --memory-cap 256M --label offline-missing -- bash -c "touch $(printf '%q' "$scene/missing-ran")"
  expect "Ⓒ 第二台那棵树 cargo metadata --offline 报缺依赖 vstd ⇒ 拒（退 255，说第二台缺依赖 vstd、带同步办法），命令没跑、第二台这一趟的树删了" \
    "$([[ "$(exit_of offline missing)" == 255 && ! -e "$scene/missing-ran" && "$(peer_runs_left offline)" == 0 ]] \
       && grep -q '第二台缺依赖 vstd' "$scene/missing.err" && grep -q 'cargo fetch' "$scene/missing.err"; echo $?)" \
    "退 $(exit_of offline missing)，命令$([[ -e "$scene/missing-ran" ]] && echo 跑了 || echo 没跑)，runs/ 下留着 $(peer_runs_left offline) 条；stderr 尾部：$(tail_of offline missing)"
  rm -f -- "$scene/metadata-missing" "$scene/metadata-calls"
  run_in offline ok --peer --memory-cap 256M --label offline-ok -- bash -c "touch $(printf '%q' "$scene/ok-ran")"
  expect "Ⓒ 第二台那棵树 cargo metadata --offline 过了 ⇒ 照跑、退 0，假 cargo 记下在那棵树的树根跑过 metadata --offline" \
    "$([[ "$(exit_of offline ok)" == 0 && -e "$scene/ok-ran" ]] \
       && grep -qxF "$scene/peer-home/runs/offline-ok metadata --offline --format-version 1 --filter-platform $(rustc -vV | sed -n 's/^host: //p')" "$scene/metadata-calls"; echo $?)" \
    "退 $(exit_of offline ok)，metadata 那一步记下的：$(cat "$scene/metadata-calls" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of offline ok)"

  # Ⓓ --fetch 一个文件、一个目录
  prepare_scene fetch
  scene="$work/fetch"
  run_in fetch run --peer --memory-cap 256M --fetch out/random.bin --fetch out/tree -- bash -c "mkdir -p out/tree/deeper && head -c 200000 /dev/urandom > out/random.bin \
&& printf '取回的目录\n' > out/tree/deeper/a.txt && sha256sum out/random.bin out/tree/deeper/a.txt > $(printf '%q' "$scene/sums")"
  expect "Ⓓ --fetch 取回的一个文件、一个目录与第二台上算的 sha256 逐字相同" \
    "$([[ "$(exit_of fetch run)" == 0 && -s "$scene/sums" ]] && (cd "$scene/fetch" && sha256sum --quiet -c "$scene/sums") >/dev/null 2>&1; echo $?)" \
    "退 $(exit_of fetch run)；核对：$( (cd "$scene/fetch" 2>/dev/null && sha256sum -c "$scene/sums" 2>&1) | tr '\n' '|')；stderr 尾部：$(tail_of fetch run)"

  # Ⓛ 不是 git 仓的树（副本）：整棵树拷过去跑，target 与本地配置不拷
  prepare_scene not-git
  scene="$work/not-git"
  rm -rf -- "${scene:?}/repo/.git"
  echo copied > "$scene/repo/copy-marker.txt"
  mkdir -p "$scene/repo/target" && echo no > "$scene/repo/target/must-not-copy"
  echo 'SELFTEST_SECRET=本地配置不许拷' > "$scene/repo/multi-host.env"
  # 树根的 multi-host.env 是 ④ 带过去的那一份（场景的配置）；树里那份写着 SELFTEST_SECRET 的本地配置不许拷过去
  run_in not-git run --peer --memory-cap 256M --label peer-not-git -- bash -c "cat copy-marker.txt > $(printf '%q' "$scene/marker"); \
if test -e target; then echo yes > $(printf '%q' "$scene/target-copied"); fi; if grep -q SELFTEST_SECRET multi-host.env 2>/dev/null; then echo yes > $(printf '%q' "$scene/env-copied"); fi"
  expect "Ⓛ 不是 git 仓的树照样拷到第二台跑：退 0，普通文件拷过去了，target 与本地配置没拷" \
    "$([[ "$(exit_of not-git run)" == 0 && "$(cat "$scene/marker" 2>/dev/null)" == copied && ! -e "$scene/target-copied" && ! -e "$scene/env-copied" ]]; echo $?)" \
    "退 $(exit_of not-git run)，标记「$(cat "$scene/marker" 2>/dev/null)」，target $([[ -e "$scene/target-copied" ]] && echo 拷过去了 || echo 没拷)，配置 $([[ -e "$scene/env-copied" ]] && echo 拷过去了 || echo 没拷)；stderr 尾部：$(tail_of not-git run)"
  # Ⓞ Ⓟ 拷的途中消失的文件（文件头 ④）：假 rsync 放在 PATH 最前面——照真的拷，拷树那一趟（参数里有 --files-from=）拷完照 rsync 的原话报
  #   SELFTEST_VANISHED 里的几个相对路径「消失了」、退 24（真 rsync 退 24 要文件恰好在拷的途中消失，自证里造不稳）
  local fake_rsync_directory="$work/fake-rsync" real_rsync
  real_rsync="$(command -v rsync)"
  mkdir -p "$fake_rsync_directory"
  cat > "$fake_rsync_directory/rsync" <<'FAKE_RSYNC'
#!/usr/bin/env bash
# multi-host-run.sh 自证的假 rsync：照真的拷；拷树那一趟拷完照 rsync 的原话报 SELFTEST_VANISHED 里的几个相对路径消失了、退 24
"$SELFTEST_REAL_RSYNC" "$@" || exit
case " $* " in *" --files-from="*) ;; *) exit 0 ;; esac
[[ -n "${SELFTEST_VANISHED:-}" ]] || exit 0
for vanished in $SELFTEST_VANISHED; do printf 'file has vanished: "%s/%s"\n' "$SELFTEST_TREE_ROOT" "$vanished" >&2; done
echo "rsync warning: some files vanished before they could be transferred (code 24) at main.c(1338) [sender=3.2.7]" >&2
exit 24
FAKE_RSYNC
  chmod +x "$fake_rsync_directory/rsync"
  local vanish_case
  for vanish_case in untracked tracked; do
    prepare_scene "vanish-$vanish_case"
    scene="$work/vanish-$vanish_case"
    git -C "$scene/repo" add payload/input.txt   # 进了索引就是被跟踪的（git ls-files -c 列得出）
    mkdir -p "$scene/repo/scripts/kb"
    printf '别的会话正在写、还没换上的一份\n' > "$scene/repo/scripts/kb/sample.py.tmp.4242"
    printf '没跟踪的笔记\n' > "$scene/repo/untracked-note.txt"
    PATH="$fake_rsync_directory:$PATH" SELFTEST_REAL_RSYNC="$real_rsync" SELFTEST_TREE_ROOT="$scene/repo" \
      SELFTEST_VANISHED="$([[ "$vanish_case" == untracked ]] && echo untracked-note.txt || echo payload/input.txt)" \
      run_in "vanish-$vanish_case" run --peer --memory-cap 256M --label "vanish-$vanish_case" -- bash -c \
      "if test -e scripts/kb/sample.py.tmp.4242; then echo yes > $(printf '%q' "$scene/temporary-copied"); fi; touch $(printf '%q' "$scene/ran")"
  done
  scene="$work/vanish-untracked"
  expect "Ⓞ 拷树：没跟踪的临时文件 scripts/kb/sample.py.tmp.4242 不拷，没跟踪的 untracked-note.txt 在拷的途中消失（rsync 退 24）⇒ 照跑、退 0、报消失了哪个" \
    "$([[ "$(exit_of vanish-untracked run)" == 0 && -e "$scene/ran" && ! -e "$scene/temporary-copied" ]] \
      && grep -qF '没跟踪的文件在拷的途中消失了（rsync 退 24）：untracked-note.txt' "$scene/run.err"; echo $?)" \
    "退 $(exit_of vanish-untracked run)，命令$([[ -e "$scene/ran" ]] && echo 跑了 || echo 没跑)，临时文件$([[ -e "$scene/temporary-copied" ]] && echo 拷过去了 || echo 没拷)；stderr 尾部：$(tail_of vanish-untracked run)"
  scene="$work/vanish-tracked"
  expect "Ⓟ 拷树：被跟踪的 payload/input.txt 在拷的途中消失（rsync 退 24）⇒ 判失败（退 255，说被跟踪的文件消失了），命令没跑" \
    "$([[ "$(exit_of vanish-tracked run)" == 255 && ! -e "$scene/ran" ]] && grep -qF '被跟踪的文件在拷的途中消失了：payload/input.txt' "$scene/run.err"; echo $?)" \
    "退 $(exit_of vanish-tracked run)，命令$([[ -e "$scene/ran" ]] && echo 跑了 || echo 没跑)；stderr 尾部：$(tail_of vanish-tracked run)"
  # Ⓔ 跑完删第二台这一趟的目录
  expect "Ⓔ Ⓑ Ⓓ 那几趟（含退 7 的那一趟）跑完，第二台 runs/ 下什么都不剩" \
    "$([[ "$(peer_runs_left peer)" == 0 && "$(peer_runs_left fetch)" == 0 && -d "$work/peer/peer-home/runs" ]]; echo $?)" \
    "Ⓑ 那边留着：$(ls -A "$work/peer/peer-home/runs" 2>/dev/null | tr '\n' ' ')；Ⓓ 那边留着：$(ls -A "$work/fetch/peer-home/runs" 2>/dev/null | tr '\n' ' ')"

  # Ⓕ 第二台那条命令在跑时收到 TERM
  prepare_scene signalled
  scene="$work/signalled"
  local driver_pid signalled_exit waited sleeper_pid="" sleeper_alive=0
  scene_command_for signalled
  "${scene_command[@]}" --peer --memory-cap 256M --label signalled -- bash -c "echo \$\$ > $(printf '%q' "$scene/sleeper.pid"); exec sleep 300" \
    > "$scene/run.out" 2> "$scene/run.err" < /dev/null &
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
  expect "Ⓕ 第二台那条命令在跑时收到 TERM ⇒ 退 143，第二台那条命令不在了，runs/ 下什么都不剩" \
    "$([[ "$driver_stuck" == 0 && "$signalled_exit" == 143 && -n "$sleeper_pid" && "$sleeper_alive" == 0 && "$(peer_runs_left signalled)" == 0 ]]; echo $?)" \
    "$([[ "$driver_stuck" == 1 ]] && echo "驱动收到 TERM 之后 120 秒还没退（发了 KILL）；")退 $signalled_exit，第二台那条命令的进程号「$sleeper_pid」$([[ "$sleeper_alive" == 1 ]] && echo 还活着 || echo 不在了)；runs/ 下留着 $(peer_runs_left signalled) 条；stderr 尾部：$(tail -n 4 "$scene/run.err" | tr '\n' '|')"
  expect "Ⓕ 第二台那条命令只握 0、1、2，不握驱动那一头的管道" \
    "$([[ -n "$sleeper_pid" && -z "$extra_fds" ]]; echo $?)" \
    "第二台那条命令的进程号「$sleeper_pid」；多握着的：${extra_fds:-（没读到）}"

  # Ⓖ 输出里没有配置里的主机与目录
  local all_outputs=("$work/peer/run.out" "$work/peer/run.err" "$work/peer/seven.err" "$work/fetch/run.err" "$work/signalled/run.err")
  expect "Ⓖ 输出里没有配置里的主机名与第二台的目录，命令打的那两样换成了「第二台」「第二台的目录」" \
    "$(! grep -qE 'selftest-multi-host-peer-name|peer-home|peer-bin' "${all_outputs[@]}" && grep -qx '第二台' "$work/peer/run.out" && grep -q '^第二台的目录/runs/peer-ok$' "$work/peer/run.out"; echo $?)" \
    "有的：$(grep -lE 'selftest-multi-host-peer-name|peer-home|peer-bin' "${all_outputs[@]}" 2>/dev/null | sed "s|^$work/||" | tr '\n' ' ')；命令的输出：$(tr '\n' '|' < "$work/peer/run.out")"

  # Ⓗ 双机开关没开
  prepare_scene switched-off ""
  scene="$work/switched-off"
  run_in switched-off run --peer --memory-cap 256M -- bash -c "touch $(printf '%q' "$scene/ran")"
  expect "Ⓗ 配置里没写 ENABLE_ACROSS_MACHINES ⇒ --peer 拒（退 255，出路写 ENABLE_ACROSS_MACHINES=1），命令哪台都没跑、第二台没建目录" \
    "$([[ "$(exit_of switched-off run)" == 255 && ! -e "$scene/ran" && ! -e "$scene/peer-home/runs" ]] && grep -q 'ENABLE_ACROSS_MACHINES=1' "$scene/run.err"; echo $?)" \
    "退 $(exit_of switched-off run)，命令$([[ -e "$scene/ran" ]] && echo 跑了 || echo 没跑)；stderr 尾部：$(tail_of switched-off run)"

  # Ⓘ 弄坏开关设在自证之外：起驱动时去掉自证的标记，弄坏开关照调用方的再加一项 peer-exit-dropped
  prepare_scene outside
  scene="$work/outside"
  scene_command_for outside
  local outside_exit
  if env -u MULTI_HOST_RUN_SELFTEST_MARK MULTI_HOST_RUN_BREAK="${MULTI_HOST_RUN_BREAK:+$MULTI_HOST_RUN_BREAK,}peer-exit-dropped" "${scene_command[@]}" \
       --memory-cap 256M -- bash -c "touch $(printf '%q' "$scene/ran")" > "$scene/run.out" 2> "$scene/run.err" < /dev/null; then
    outside_exit=0
  else
    outside_exit=$?
  fi
  expect "Ⓘ MULTI_HOST_RUN_BREAK 设在 --selftest 之外（不带自证的标记）⇒ 拒（退 2，说设在 --selftest 之外），命令没跑" \
    "$([[ "$outside_exit" == 2 && ! -e "$scene/ran" ]] && grep -q '设在 --selftest 之外' "$scene/run.err"; echo $?)" \
    "退 $outside_exit，命令$([[ -e "$scene/ran" ]] && echo 跑了 || echo 没跑)；stderr 尾部：$(tail -n 3 "$scene/run.err" 2>/dev/null | tr '\n' '|')"

  # Ⓙ 没设 MULTI_HOST_RUN_FETCH_DIRECTORY：取回到工作根下（SINGLEFS_WORK_ROOT 指进场景目录，TMPDIR 也指进来看有没有落进去）；设了它照它
  prepare_scene work-root-fetch
  scene="$work/work-root-fetch"
  mkdir -p "$scene/tmp"
  local default_fetch_exit override_fetch_exit
  local -a fetch_scene_command=(env -u SINGLEFS_HEAVY_TESTS -u RUN_WITH_MEMORY_CAP_RESERVE -u RUN_WITH_MEMORY_CAP_KEY -u MULTI_HOST_RUN_FETCH_DIRECTORY
    SINGLEFS_MULTI_HOST_CONFIG="$scene/multi-host.env" SINGLEFS_WORK_ROOT="$scene/work-root" TMPDIR="$scene/tmp" RUN_WITH_MEMORY_CAP_PEAKS="$scene/peaks.tsv")
  if "${fetch_scene_command[@]}" bash "$scene/repo/scripts/hosts/multi-host-run.sh" --memory-cap 256M --label default-fetch --fetch out/default.txt -- \
       bash -c "mkdir -p out && printf '默认取回目录\n' > out/default.txt" > "$scene/default.out" 2> "$scene/default.err" < /dev/null; then
    default_fetch_exit=0
  else
    default_fetch_exit=$?
  fi
  expect "Ⓙ 没设 MULTI_HOST_RUN_FETCH_DIRECTORY ⇒ 取回到工作根下 multi-host-run/<标签>/，不落进 \$TMPDIR" \
    "$([[ "$default_fetch_exit" == 0 && ! -e "$scene/tmp/singlefs-multi-host-run" ]] && cmp -s "$scene/repo/out/default.txt" "$scene/work-root/multi-host-run/default-fetch/out/default.txt"; echo $?)" \
    "默认那一趟退 $default_fetch_exit，工作根下$([[ -e "$scene/work-root/multi-host-run/default-fetch/out/default.txt" ]] && echo 有 || echo 没有)取回的文件；\$TMPDIR 下$([[ -e "$scene/tmp/singlefs-multi-host-run" ]] && echo 有 || echo 没有)；stderr 尾部：$(tail -n 3 "$scene/default.err" 2>/dev/null | tr '\n' '|')"
  if "${fetch_scene_command[@]}" MULTI_HOST_RUN_FETCH_DIRECTORY="$scene/override" bash "$scene/repo/scripts/hosts/multi-host-run.sh" --memory-cap 256M --label override-fetch \
       --fetch out/override.txt -- bash -c "mkdir -p out && printf '点名的取回目录\n' > out/override.txt" > "$scene/override.out" 2> "$scene/override.err" < /dev/null; then
    override_fetch_exit=0
  else
    override_fetch_exit=$?
  fi
  expect "Ⓙ 设了 MULTI_HOST_RUN_FETCH_DIRECTORY ⇒ 取回到它点名的目录，工作根下没有这一趟" \
    "$([[ "$override_fetch_exit" == 0 && ! -e "$scene/work-root/multi-host-run/override-fetch" ]] && cmp -s "$scene/repo/out/override.txt" "$scene/override/out/override.txt"; echo $?)" \
    "设了变量那一趟退 $override_fetch_exit，点名的目录下$([[ -e "$scene/override/out/override.txt" ]] && echo 有 || echo 没有)取回的文件，工作根下$([[ -e "$scene/work-root/multi-host-run/override-fetch" ]] && echo 有 || echo 没有)这一趟；stderr 尾部：$(tail -n 3 "$scene/override.err" 2>/dev/null | tr '\n' '|')"

  # Ⓚ 内存包装认出显存不够（假 journalctl：-n 1 那一问打一行，查窗口那一问打 NVRM … NV_ERR_NO_MEMORY）
  prepare_scene gpu-memory
  scene="$work/gpu-memory"
  mkdir -p "$scene/fake-journal"
  printf '%s\n' '#!/usr/bin/env bash' '# 自证用的假 journalctl：问最近一条打一行，问起止之间打一行显卡驱动报的显存不够' \
    'case " $* " in *" -n 1 "*) echo "host kernel: probe" ;; *) echo "host kernel: NVRM: GPU0 nvAssertOkFailedNoLog: Out of memory [NV_ERR_NO_MEMORY] (0x00000051)" ;; esac' \
    > "$scene/fake-journal/journalctl"
  chmod +x "$scene/fake-journal/journalctl"
  RUN_WITH_MEMORY_CAP_JOURNALCTL="$scene/fake-journal/journalctl" run_in gpu-memory run --memory-cap 256M -- bash -c 'exit 3'
  expect "Ⓚ 本机跑、内存包装认出显存不够 ⇒ 原样退 245、报「显存不够」、说那一次的输出不算结果" \
    "$([[ "$(exit_of gpu-memory run)" == 245 ]] && grep -q '退 245：显存不够' "$scene/run.err" && grep -q '不算结果' "$scene/run.err"; echo $?)" \
    "退 $(exit_of gpu-memory run)，stderr 尾部：$(tail_of gpu-memory run)"

  # Ⓜ 不接跨趟共用的编译目录（上面「逐格」那一段的 Ⓜ）
  local real_cargo driver_exit=""
  real_cargo="$(command -v cargo)"
  prepare_scene shared-refused
  scene="$work/shared-refused"
  MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY=../shared-target run_in shared-refused variable --peer --memory-cap 256M --label shared-variable -- bash -c "touch $(printf '%q' "$scene/variable-ran")"
  expect "Ⓜ 设了 MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY ⇒ 用法错（退 2），命令没跑" \
    "$([[ "$(exit_of shared-refused variable)" == 2 && ! -e "$scene/variable-ran" ]] && grep -q 'MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY=../shared-target' "$scene/variable.err"; echo $?)" \
    "退 $(exit_of shared-refused variable)，命令$([[ -e "$scene/variable-ran" ]] && echo 跑了 || echo 没跑)；stderr 尾部：$(tail_of shared-refused variable)"
  run_in shared-refused assignment --peer --memory-cap 256M --label shared-assignment -- env CARGO_TARGET_DIR=../shared-target bash -c "touch $(printf '%q' "$scene/assignment-ran")"
  run_in shared-refused script --peer --memory-cap 256M --label shared-script -- bash -c "export CARGO_TARGET_DIR=/var/tmp/shared-target; touch $(printf '%q' "$scene/script-ran")"
  expect "Ⓜ --peer 命令里 CARGO_TARGET_DIR 指到树外（env 的赋值、bash -c 的脚本两种写法）⇒ 用法错（退 2），命令没跑" \
    "$([[ "$(exit_of shared-refused assignment)" == 2 && "$(exit_of shared-refused script)" == 2 && ! -e "$scene/assignment-ran" && ! -e "$scene/script-ran" ]] \
       && grep -q 'CARGO_TARGET_DIR 指到树外' "$scene/assignment.err"; echo $?)" \
    "env 那一趟退 $(exit_of shared-refused assignment)，bash -c 那一趟退 $(exit_of shared-refused script)；stderr 尾部：$(tail_of shared-refused assignment)"

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
  # Ⓠ 本机跑的编译目录（文件头「本机模式的编译目录」）：命令记下它拿到的 CARGO_TARGET_DIR、往里写一份产物；正常退 0、出错退 7、被 TERM 停三种结局跑完那个目录都不在；
  #   调用方设了 CARGO_TARGET_DIR 的照它、跑完不删
  prepare_scene local-target
  scene="$work/local-target"
  local target_command='printf "%s\n" "${CARGO_TARGET_DIR:-没设}" > "$0"; mkdir -p "$CARGO_TARGET_DIR/debug" && echo built > "$CARGO_TARGET_DIR/debug/artifact"'
  local local_target_driver target_problems="" target_case recorded
  run_in local-target ok --memory-cap 256M -- bash -c "$target_command" "$scene/ok.target"
  run_in local-target failed --memory-cap 256M -- bash -c "$target_command; exit 7" "$scene/failed.target"
  scene_command_for local-target
  "${scene_command[@]}" --memory-cap 256M -- bash -c "$target_command; echo \$\$ > $(printf '%q' "$scene/term.pid"); exec sleep 300" "$scene/term.target" \
    > "$scene/term.out" 2> "$scene/term.err" < /dev/null &
  local_target_driver=$!
  wait_for_file "$scene/term.pid" 60 "$local_target_driver"
  kill -TERM "$local_target_driver"
  wait_driver "$local_target_driver" 120
  echo "$driver_exit" > "$scene/term.exit"
  for target_case in ok:0 failed:7 term:143; do
    recorded="$(cat "$scene/${target_case%%:*}.target" 2>/dev/null)"
    if [[ "$(exit_of local-target "${target_case%%:*}")" != "${target_case#*:}" || -z "$recorded" || "$recorded" == 没设 || "$recorded" == "$scene/repo/target" || -e "$recorded" ]] \
       || ! grep -qF "删了这一趟的编译目录 $recorded" "$scene/${target_case%%:*}.err"; then
      target_problems+="${target_case%%:*} 那一趟退 $(exit_of local-target "${target_case%%:*}")（要 ${target_case#*:}），编译目录「${recorded:-没记下}」$([[ -n "$recorded" && -e "$recorded" ]] && echo 还在 || echo 不在)；"
    fi
  done
  expect "Ⓠ 本机跑：没设 CARGO_TARGET_DIR ⇒ 命令拿到的是这一趟自己建的编译目录，正常退 0、出错退 7、被 TERM 停（退 143）三种结局跑完都删了" \
    "$([[ -z "$target_problems" ]]; echo $?)" "${target_problems}stderr 尾部：$(tail_of local-target term)"
  scene_command_for local-target
  "${scene_command[@]:0:${#scene_command[@]}-2}" CARGO_TARGET_DIR="$scene/caller-target" "${scene_command[@]: -2}" --memory-cap 256M -- bash -c "$target_command" "$scene/caller.target" \
    > "$scene/caller.out" 2> "$scene/caller.err" < /dev/null
  echo "$?" > "$scene/caller.exit"
  expect "Ⓠ 本机跑：调用方设了 CARGO_TARGET_DIR ⇒ 命令照它编，跑完不删" \
    "$([[ "$(exit_of local-target caller)" == 0 && "$(cat "$scene/caller.target" 2>/dev/null)" == "$scene/caller-target" && -f "$scene/caller-target/debug/artifact" ]]; echo $?)" \
    "退 $(exit_of local-target caller)，命令拿到「$(cat "$scene/caller.target" 2>/dev/null)」，$([[ -f "$scene/caller-target/debug/artifact" ]] && echo 产物还在 || echo 产物不在)；stderr 尾部：$(tail_of local-target caller)"
  # Ⓠ --peer：命令在那棵树里建的 target 跑完连同树删，第二台那条命令拿不到本机的 CARGO_TARGET_DIR
  run_in local-target peer --peer --memory-cap 256M --label target-in-tree -- bash -c "printf '%s\n' \"\${CARGO_TARGET_DIR-没设}\" > $(printf '%q' "$scene/peer.target"); mkdir -p target/debug && echo built > target/debug/artifact"
  expect "Ⓠ --peer：命令在那棵树里建的 target 跑完连同树删、第二台那条命令不带本机的 CARGO_TARGET_DIR" \
    "$([[ "$(exit_of local-target peer)" == 0 && "$(cat "$scene/peer.target" 2>/dev/null)" == 没设 && "$(peer_runs_left local-target)" == 0 ]]; echo $?)" \
    "退 $(exit_of local-target peer)，第二台那条命令拿到「$(cat "$scene/peer.target" 2>/dev/null)」，runs/ 下留着 $(peer_runs_left local-target) 条；stderr 尾部：$(tail_of local-target peer)"
  # Ⓡ 删之前核开着的文件：命令记下编译目录（本机）或树根（--peer）后等放行文件；自证在那里起一个 sleep 开着一份文件（只记它自己的进程号），再放行。
  #   驱动该不删、报出那个进程、退 255；之后只停那一个进程号，手删留下的那一份
  prepare_scene holders
  scene="$work/holders"
  # 本机、--peer 两趟逐条调用：每趟只给自己起的那一个 sleep 的进程号发一次终止信号，不在循环里发信号
  selftest_holder_case() { # <local|peer>
    local holder_case="$1" holder_driver holder_sleep_pid holder_where
    scene_command_for holders
    if [[ "$holder_case" == local ]]; then
      "${scene_command[@]}" --memory-cap 256M -- bash -c 'printf "%s\n" "$CARGO_TARGET_DIR" > "$0.where"; until test -e "$0.go"; do sleep 0.1; done' "$scene/$holder_case" \
        > "$scene/$holder_case.out" 2> "$scene/$holder_case.err" < /dev/null &
    else
      "${scene_command[@]}" --peer --memory-cap 256M --label holders-peer -- bash -c 'pwd > "$0.where"; until test -e "$0.go"; do sleep 0.1; done' "$scene/$holder_case" \
        > "$scene/$holder_case.out" 2> "$scene/$holder_case.err" < /dev/null &
    fi
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
  selftest_holder_case local
  selftest_holder_case peer
  rm -rf -- "${scene:?}/peer-home/runs"
  expect "Ⓡ 本机跑：命令退了、还有进程开着这一趟编译目录里的文件 ⇒ 不删、报出那个进程、退 255" \
    "$([[ "$(exit_of holders local)" == 255 && -e "$scene/local.kept" ]] && grep -q '还有 1 个进程开着里面的文件' "$scene/local.err" && grep -q '（sleep 60）' "$scene/local.err"; echo $?)" \
    "退 $(exit_of holders local)，编译目录$([[ -e "$scene/local.kept" ]] && echo 还在 || echo 删了)；stderr 尾部：$(tail_of holders local)"
  expect "Ⓡ --peer：第二台那条命令退了、还有进程开着那棵树里的文件 ⇒ ⑦ 不删、报出那个进程、退 255" \
    "$([[ "$(exit_of holders peer)" == 255 && -e "$scene/peer.kept" ]] && grep -q '这一趟的树里还有进程开着文件' "$scene/peer.err" && grep -q '（sleep 60）' "$scene/peer.err"; echo $?)" \
    "退 $(exit_of holders peer)，树「$(cat "$scene/peer.where" 2>/dev/null)」$([[ -e "$scene/peer.kept" ]] && echo 还在 || echo 删了)；stderr 尾部：$(tail_of holders peer)"
  # 本机那一趟留下的临时目录（驱动没删、打了路径）：记下的编译目录是 <临时目录>/target 才删它的上一层
  local leftover_target leftover_scratch
  leftover_target="$(cat "$scene/local.where" 2>/dev/null)"
  leftover_scratch="${leftover_target%/target}"
  if [[ "${leftover_target##*/}" == target && -d "$leftover_target" ]]; then rm -rf -- "${leftover_scratch:?}"; fi

  # Ⓝ 第二台在用几个核（文件头 ⑤「在用核数」）：每 1 秒取一次、窗口 2 秒，告警记录写场景目录。被取样的负载换成假的、不随此刻负载
  #   （真负载 core_usage.py busy-load 在本机忙的时候量出来低于阈值，登记数那一趟时红时不红）：这一场景的小仓里 core_usage.py 挪成 core_usage_real.py，
  #   原处放一份假的：busy-load 不占 CPU，只往树根记下线程数与起点、睡够秒数；sample 照记下的线程数报「每条线程从起点起一直在算」的样本；
  #   judge、declare-serial 交给真的那一份。驱动在第二台取样、在本机判、打行、记告警那几步与判法照真的走
  local core_threads core_scene core_red_mark=$'\xe2\x9c\x97'   # 在用核数告警那一行开头的叉号；下面拼要找的整行用它，字面量不写进这些检查行里
  core_threads="$(python3 "$repository_root/scripts/hosts/heavy_threads.py" --print 2>/dev/null)"
  prepare_scene cores
  core_scene="$work/cores"
  mv -- "$core_scene/repo/scripts/hosts/core_usage.py" "$core_scene/repo/scripts/hosts/core_usage_real.py"
  cat > "$core_scene/repo/scripts/hosts/core_usage.py" <<'FAKE_CORE_USAGE'
#!/usr/bin/env python3
# multi-host-run.sh 自证 Ⓝ 的假 core_usage.py：busy-load 不占 CPU，往当前目录（那棵树的根）记下线程数与起点再睡够秒数；
# sample 照记下的线程数报「每条线程从起点起一直在算」的样本，与此刻本机多忙无关；别的子命令交给同目录的 core_usage_real.py
import json, os, sys, time
arguments = sys.argv[1:]
state = ".selftest-busy-load"
if arguments[:1] == ["busy-load"] and len(arguments) == 3:
    with open(state, "w", encoding="utf-8") as handle:
        handle.write(f"{int(arguments[1])} {time.time():.3f}\n")
    time.sleep(float(arguments[2]))
    sys.exit(0)
if arguments[:1] == ["sample"]:
    options = dict(zip(arguments[1::2], arguments[2::2]))
    try:
        with open(options["--pid-file"], encoding="utf-8") as handle:
            root = int(handle.read().split()[0])
        with open(state, encoding="utf-8") as handle:
            threads_text, started_text = handle.read().split()
    except (OSError, KeyError, ValueError, IndexError):
        sys.exit(1)   # 负载还没起来：驱动报这一次没取到、下一次再取
    threads, started, now = int(threads_text), float(started_text), time.time()
    elapsed = max(0.0, now - started)
    sample = {"root": root, "time": round(now, 3), "started": round(started, 2), "cpu": round(threads * elapsed, 2), "processes": 1, "threads": threads,
              "peer_drivers": 0, "thread_cpu": {str(index): round(elapsed, 2) for index in range(threads)}}
    declarations = options.get("--declarations", "")
    if declarations and os.path.isfile(declarations):
        with open(declarations, encoding="utf-8", errors="replace") as handle:
            sample["declarations_text"] = handle.read()
    print(json.dumps(sample, ensure_ascii=False, separators=(",", ":")))
    sys.exit(0)
os.execv(sys.executable, [sys.executable, os.path.join(os.path.dirname(os.path.realpath(__file__)), "core_usage_real.py"), *arguments])
FAKE_CORE_USAGE
  core_run() { # <这一趟的名字> <命令…>：在 cores 场景里起一趟 --peer
    MULTI_HOST_RUN_CORE_SECONDS=1 SINGLEFS_CORE_USAGE_WINDOW_SECONDS=2 SINGLEFS_CORE_USAGE_ALERTS="$core_scene/$1.alerts" CLAUDE_CODE_SESSION_ID=multi-host-run-selftest-cores \
      run_in cores "$1" --peer --memory-cap 256M --label "cores-$1" -- "${@:2}"
  }
  core_alert_count() { if [[ -f "$core_scene/$1.alerts" ]]; then wc -l < "$core_scene/$1.alerts"; else echo 0; fi; }
  core_run one python3 scripts/hosts/core_usage.py busy-load 1 6
  core_run many python3 scripts/hosts/core_usage.py busy-load "$core_threads" 6
  core_run declared bash -c 'python3 scripts/hosts/core_usage.py declare-serial 计时窗口必须单线程跑完 && exec python3 scripts/hosts/core_usage.py busy-load 1 6'
  expect "Ⓝ 第二台只起 1 条线程忙等 ⇒ 打「第二台在用 X 核（登记 N）」与 ✗ 一行带出路、写告警记录（带会话号与来源）" \
    "$([[ "$(exit_of cores one)" == 0 && -n "$core_threads" ]] && grep -q "✗ 第二台在用 [0-9.]* 核（登记 $core_threads）" "$core_scene/one.err" \
       && grep -q '→ 怎么办：第二台那一趟在用的核太少' "$core_scene/one.err" \
       && grep -q '"session_id":"multi-host-run-selftest-cores"' "$core_scene/one.alerts" && grep -q '"source":"multi-host-run.sh --peer"' "$core_scene/one.alerts"; echo $?)" \
    "退 $(exit_of cores one)；在用核数那几行：$(grep '第二台在用' "$core_scene/one.err" 2>/dev/null | cut_characters 160 | tr '\n' '|')；告警记录 $(core_alert_count one) 条"
  expect "Ⓝ 第二台起登记数那么多条线程忙等 ⇒ 打「第二台在用 X 核（登记 N）」、不打 ✗、不写告警记录" \
    "$([[ "$(exit_of cores many)" == 0 && ! -e "$core_scene/many.alerts" ]] && grep -q "第二台在用 [0-9.]* 核（登记 $core_threads）：最近" "$core_scene/many.err" \
       && ! grep -q "${core_red_mark} 第二台在用" "$core_scene/many.err"; echo $?)" \
    "退 $(exit_of cores many)；在用核数那几行：$(grep '第二台在用' "$core_scene/many.err" 2>/dev/null | cut_characters 160 | tr '\n' '|')；告警记录 $(core_alert_count many) 条"
  expect "Ⓝ 第二台 1 条线程忙等、程序先声明了串行 ⇒ 照报 ✗，标出理由" \
    "$([[ "$(exit_of cores declared)" == 0 ]] && grep -q "${core_red_mark} 第二台在用.*声明了串行 [0-9]* 秒：计时窗口必须单线程跑完" "$core_scene/declared.err"; echo $?)" \
    "退 $(exit_of cores declared)；✗ 那几行：$(grep '✗ 第二台在用' "$core_scene/declared.err" 2>/dev/null | cut_characters 200 | tr '\n' '|')；stderr 尾部：$(tail_of cores declared)"

  # Ⓞ 第二台起含 cargo 的命令之前判第二台的 RocksDB 预编库取值（文件头 ⑤「RocksDB 预编库的取值」）：第二台的 PATH 前面放不带 entry 认的标记的假 cargo
  #   （-V 转给真 cargo，工具链照比；别的就建一份「跑了」的文件），本机当第二台时环境照 ssh 那样只给最少几样、四个取值都没有，那棵树的 scripts/experiment-apparatus/Cargo.toml
  #   依赖 rocksdb ⇒ 退 78、假 cargo 没起、输出里有 rocksdb-prebuilt entry 的红句；同一场景里命令不含 cargo 的照跑、退 0
  prepare_scene rocksdb
  scene="$work/rocksdb"
  printf '%s\n' '#!/usr/bin/env bash' '# 自证用的 cargo（Ⓞ）：不带 entry 认的标记；-V / --version 转给真 cargo（两台工具链照比），别的建一份「跑了」的文件、退 0' \
    "case \"\${1:-}\" in -V|--version) exec $(printf '%q' "$real_cargo") \"\$@\" ;; esac" ": > $(printf '%q' "$scene/cargo-ran")" > "$scene/peer-bin/cargo"
  chmod +x "$scene/peer-bin/cargo"
  run_in rocksdb refused --peer --memory-cap 256M --label rocksdb-refused -- cargo build
  run_in rocksdb plain --peer --memory-cap 256M --label rocksdb-plain -- bash -c "touch $(printf '%q' "$scene/plain-ran")"
  expect "Ⓞ 第二台起含 cargo 的命令之前判第二台的取值：取值缺了 ⇒ 退 78、假 cargo 没起；不含 cargo 的命令不判、照跑退 0" \
    "$([[ "$(exit_of rocksdb refused)" == 78 && ! -e "$scene/cargo-ran" && "$(exit_of rocksdb plain)" == 0 && -e "$scene/plain-ran" ]] \
       && grep -q 'rocksdb-prebuilt entry：' "$scene/refused.out"; echo $?)" \
    "含 cargo 的退 $(exit_of rocksdb refused)、假 cargo $([[ -e "$scene/cargo-ran" ]] && echo 起了 || echo 没起)，不含的退 $(exit_of rocksdb plain)；输出尾部：$(tail -n 3 "$scene/refused.out" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of rocksdb refused)"

  # Ⓢ 资源释放广播（文件头「资源释放广播」）：事件目录给这一场景自己的、不推；读事件只用 cat 与 grep
  prepare_scene broadcast
  scene="$work/broadcast"
  printf '%s\n' "resource_release_broadcast.py 自证的标记" > "$scene/broadcast-mark"
  RESOURCE_BROADCAST_DIRECTORY="$scene/events" RESOURCE_BROADCAST_PUSH=off \
    run_in broadcast sent --peer --memory-cap 256M --label broadcast-sent -- bash -c 'exit 3'
  RESOURCE_BROADCAST_DIRECTORY= XDG_RUNTIME_DIR="$scene/runtime" RESOURCE_BROADCAST_PUSH=off \
    run_in broadcast test-world --peer --memory-cap 256M --label broadcast-test-world -- true
  RESOURCE_BROADCAST_DIRECTORY="$scene/events-fault" RESOURCE_BROADCAST_PUSH=off RESOURCE_BROADCAST_SELFTEST_FAULT=crash RESOURCE_BROADCAST_SELFTEST_MARK="$scene/broadcast-mark" \
    run_in broadcast fault --peer --memory-cap 256M --label broadcast-fault -- bash -c 'exit 3'
  local broadcast_sent broadcast_test_world
  broadcast_sent="$(cat "$scene/events"/*.event 2>/dev/null | grep -cE '^seq=[0-9]+ kind=peer-run-finished host=第二台 source=local label=broadcast-sent exit=3 ')"
  broadcast_test_world="$(cat "$scene/runtime/singlefs-resource-events"/*.event 2>/dev/null | grep -c ' kind=peer-run-finished ')"
  expect "Ⓢ --peer 跑完（命令退 3）⇒ 本机代发一条 peer-run-finished（host=第二台、标签、exit=3），驱动照退 3" \
    "$([[ "$(exit_of broadcast sent)" == 3 && "$broadcast_sent" == 1 ]]; echo $?)" \
    "退 $(exit_of broadcast sent)，合的事件 ${broadcast_sent:-0} 条：$(cat "$scene/events"/*.event 2>/dev/null | tr '\n' '|')"
  expect "Ⓢ 第二台是本机上的另一个目录而没给 RESOURCE_BROADCAST_DIRECTORY ⇒ 不发" \
    "$([[ "$(exit_of broadcast test-world)" == 0 && "$broadcast_test_world" == 0 ]]; echo $?)" \
    "退 $(exit_of broadcast test-world)，默认事件目录里有 ${broadcast_test_world:-0} 条 peer-run-finished"
  expect "Ⓢ 资源释放广播那份库出事 ⇒ 驱动照退命令的 3、输出里没有广播那一步的行" \
    "$([[ "$(exit_of broadcast fault)" == 3 ]] && ! grep -qE '自证造的崩溃|没广播到|resource_release_broadcast' "$scene/fault.err" "$scene/fault.out"; echo $?)" \
    "退 $(exit_of broadcast fault)；stderr 尾部：$(tail_of broadcast fault)"

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

  # Ⓤ 两台用同一份多机配置（文件头 ④、⑤；scripts/hosts/peer-host-lib.sh 文件头「两台用同一份多机配置」）：--peer 一趟，第二台那条命令记下它看到的角色、
  #   SINGLEFS_MULTI_HOST_CONFIG 与那份文件的 sha256；要是这一趟的树根下那一份、与场景的配置 sha256 相同，第二台专用目录下常驻那一份也相同
  local scene_sum seen_line standing_sum
  local seen_command='printf "role=%s configuration=%s sum=%s origin=%s\n" "${SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE-没设}" "${SINGLEFS_MULTI_HOST_CONFIG-没设}" "$(if test -f "${SINGLEFS_MULTI_HOST_CONFIG:-}"; then sha256sum < "$SINGLEFS_MULTI_HOST_CONFIG" | cut -d " " -f 1; else echo 不在; fi)" "${GPU_QUIESCE_ORIGIN_READ_BACK-没设}" > "$0"'
  prepare_scene carry
  scene="$work/carry"
  run_in carry run --peer --memory-cap 256M --label carry -- bash -c "$seen_command" "$scene/seen"
  scene_sum="$(sha256sum < "$scene/multi-host.env" | cut -d' ' -f1)"
  seen_line="$(cat "$scene/seen" 2>/dev/null)"
  standing_sum="$(sha256sum < "$scene/peer-home/multi-host.env" 2>/dev/null | cut -d' ' -f1)"
  expect "Ⓤ --peer 把本机这一份多机配置原样带到第二台这一趟的树根（sha256 相同），第二台那条命令认得自己是第二台，第二台的专用目录里常驻那一份也相同" \
    "$([[ "$(exit_of carry run)" == 0 && "$seen_line" == "role=peer configuration=$scene/peer-home/runs/carry/multi-host.env sum=$scene_sum origin=没设" && "$standing_sum" == "$scene_sum" ]]; echo $?)" \
    "退 $(exit_of carry run)；第二台那条命令看到的：${seen_line:-（没记下）}；本机这一份 ${scene_sum:0:12}，常驻那一份 ${standing_sum:-不在}；stderr 尾部：$(tail_of carry run)"

  # Ⓥ 两台的配置不同：假 rsync 放在 PATH 最前面，照真的拷，拷的目标是 …/multi-host.env 的那一趟拷完往它末尾加一行（造「带过去的那一份与本机的不同」）
  local differing_rsync_directory="$work/differing-rsync"
  mkdir -p "$differing_rsync_directory"
  cat > "$differing_rsync_directory/rsync" <<'DIFFERING_RSYNC'
#!/usr/bin/env bash
# multi-host-run.sh 自证 Ⓥ 的假 rsync：照真的拷；目标是 …/multi-host.env 时拷完往它末尾加一行，带到第二台的那一份就与本机的不同
"$SELFTEST_REAL_RSYNC" "$@" || exit
target="${!#}"
if [[ "$target" == */multi-host.env ]]; then printf '# 自证造的不同：第二台那一份多了这一行\n' >> "$target"; fi
DIFFERING_RSYNC
  chmod +x "$differing_rsync_directory/rsync"
  prepare_scene carry-differs
  scene="$work/carry-differs"
  PATH="$differing_rsync_directory:$PATH" SELFTEST_REAL_RSYNC="$real_rsync" \
    run_in carry-differs run --peer --memory-cap 256M --label carry-differs -- bash -c "touch $(printf '%q' "$scene/ran")"
  expect "Ⓥ 带到第二台的配置与本机这一份 sha256 不同（假 rsync 拷完往那一份末尾加一行）⇒ 拒（退 255，说两台的配置不同），命令没跑、第二台这一趟的树删了" \
    "$([[ "$(exit_of carry-differs run)" == 255 && ! -e "$scene/ran" && "$(peer_runs_left carry-differs)" == 0 ]] && grep -q '两台的多机配置不同' "$scene/run.err"; echo $?)" \
    "退 $(exit_of carry-differs run)，命令$([[ -e "$scene/ran" ]] && echo 跑了 || echo 没跑)，runs/ 下留着 $(peer_runs_left carry-differs) 条；stderr 尾部：$(tail_of carry-differs run)"

  # Ⓦ 第二台上照角色换读：第二台的 PATH 最前面（PEER_CARGO_BIN_DIRECTORY）放假 ssh，被起一次就往场景目录记一行、退 255。第二台那条命令在带过去的配置、角色 peer 之下
  #   单跑判法、带 --emit-assignments 跑判法、再起 multi-host-run.sh --peer；判法的弄坏开关随命令带过去（第二台那条命令的环境照 ssh 那样只给最少几样）。
  #   另一趟把角色写成 Peer（不是 local 也不是 peer）单跑判法 ⇒ 判不过、退 1、点名这个变量
  prepare_scene peer-side
  scene="$work/peer-side"
  printf '%s\n' '#!/usr/bin/env bash' '# multi-host-run.sh 自证 Ⓦ 的假 ssh：第二台上被起一次就记一行，照连不上退 255' "printf '%s\n' \"\$*\" >> $(printf '%q' "$scene/ssh.log")" 'exit 255' \
    > "$scene/peer-bin/ssh"
  chmod +x "$scene/peer-bin/ssh"
  local peer_side_break="MULTI_HOST_CONFIGURATION_BREAK=$(printf '%q' "${MULTI_HOST_CONFIGURATION_BREAK:-}")"
  run_in peer-side run --peer --memory-cap 256M --label peer-side -- bash -c "
env $peer_side_break bash scripts/crash/crash-point-replay-shard-configuration-check.sh . > \"\$0.check\" 2>&1; echo \$? > \"\$0.check-exit\"
env $peer_side_break bash scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-assignments . > \"\$0.emit\" 2>&1; echo \$? > \"\$0.emit-exit\"
bash scripts/hosts/multi-host-run.sh --peer -- true > \"\$0.nested\" 2>&1; echo \$? > \"\$0.nested-exit\"
cat $(printf '%q' "$scene/ssh.log") 2>/dev/null | wc -l > \"\$0.ssh-count\"
env $peer_side_break SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE=Peer bash scripts/crash/crash-point-replay-shard-configuration-check.sh . > \"\$0.bad-role\" 2>&1; echo \$? > \"\$0.bad-role-exit\"" "$scene/peer"
  expect "Ⓦ 第二台上（带过去的配置、角色 peer）判法报「双机：关」退 3、不打赋值，再起 multi-host-run.sh --peer 拒（退 255），一次都没往「第二台」ssh" \
    "$([[ "$(exit_of peer-side run)" == 0 && "$(cat "$scene/peer.check-exit" 2>/dev/null)" == 3 && "$(cat "$scene/peer.emit-exit" 2>/dev/null)" == 3 && "$(cat "$scene/peer.nested-exit" 2>/dev/null)" == 255 && "$(cat "$scene/peer.ssh-count" 2>/dev/null)" == 0 ]] \
       && grep -q '这一台是第二台' "$scene/peer.check" && ! grep -q '^PEER_SSH_HOST=' "$scene/peer.emit" && grep -q '这一台却就是第二台' "$scene/peer.nested"; echo $?)" \
    "判法退 $(cat "$scene/peer.check-exit" 2>/dev/null)（$(tr '\n' ' ' < "$scene/peer.check" 2>/dev/null | cut_characters 80)），emit 退 $(cat "$scene/peer.emit-exit" 2>/dev/null)，里层 --peer 退 $(cat "$scene/peer.nested-exit" 2>/dev/null)；ssh 记了 $(cat "$scene/peer.ssh-count" 2>/dev/null || echo 没数) 次；stderr 尾部：$(tail_of peer-side run)"
  expect "Ⓦ 角色写了 local、peer 之外的值（Peer）⇒ 判法判不过（退 1），说是这个变量" \
    "$([[ "$(cat "$scene/peer.bad-role-exit" 2>/dev/null)" == 1 ]] && grep -q 'SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE 只许写 local 或 peer' "$scene/peer.bad-role"; echo $?)" \
    "判法退 $(cat "$scene/peer.bad-role-exit" 2>/dev/null)；输出：$(tr '\n' ' ' < "$scene/peer.bad-role" 2>/dev/null | cut_characters 80)"

  # Ⓧ 起活这一侧清没清场：驱动正经 gpu-quiesce-run.sh 起（GPU_QUIESCE_HOLDER 指的登记文件在、名字以驱动的父进程号——这个自证的进程——加「-」起头）⇒ 第二台那条命令拿到
  #   GPU_QUIESCE_ORIGIN_READ_BACK=1；登记文件名字的进程号对不上 ⇒ 没设
  prepare_scene origin-quiesce
  scene="$work/origin-quiesce"
  mkdir -p "$scene/holders"
  : > "$scene/holders/$BASHPID-1"
  : > "$scene/holders/1-1"
  scene_command_for origin-quiesce
  local origin_command=("${scene_command[@]:0:${#scene_command[@]}-2}")
  "${origin_command[@]}" GPU_QUIESCE_HOLDER="$scene/holders/$BASHPID-1" "${scene_command[@]: -2}" --peer --memory-cap 256M --label origin-held -- bash -c "$seen_command" "$scene/held.seen" \
    > "$scene/held.out" 2> "$scene/held.err" < /dev/null
  echo "$?" > "$scene/held.exit"
  "${origin_command[@]}" GPU_QUIESCE_HOLDER="$scene/holders/1-1" "${scene_command[@]: -2}" --peer --memory-cap 256M --label origin-other -- bash -c "$seen_command" "$scene/other.seen" \
    > "$scene/other.out" 2> "$scene/other.err" < /dev/null
  echo "$?" > "$scene/other.exit"
  expect "Ⓧ 驱动正经 gpu-quiesce-run.sh 起（登记文件在、名字以驱动的父进程号起头）⇒ 第二台那条命令拿到 GPU_QUIESCE_ORIGIN_READ_BACK=1；不是 ⇒ 没设" \
    "$([[ "$(exit_of origin-quiesce held)" == 0 && "$(exit_of origin-quiesce other)" == 0 && "$(sed -n 's/.* origin=//p' "$scene/held.seen" 2>/dev/null)" == 1 \
       && "$(sed -n 's/.* origin=//p' "$scene/other.seen" 2>/dev/null)" == 没设 ]]; echo $?)" \
    "那一趟退 $(exit_of origin-quiesce held)、看到 origin=$(sed -n 's/.* origin=//p' "$scene/held.seen" 2>/dev/null)；不是那一趟退 $(exit_of origin-quiesce other)、看到 origin=$(sed -n 's/.* origin=//p' "$scene/other.seen" 2>/dev/null)"

  # Ⓨ 判法报两台配置一不一致（判法文件头「两台配置一不一致」）：Ⓤ 跑过一趟（第二台专用目录下常驻那一份与场景的配置相同）⇒ 报一致；场景的配置末尾加一行 ⇒ 报不一致；
  #   没跑过 --peer 的场景（第二台没有常驻那一份）⇒ 报还没有
  local consistent_text differing_text missing_text
  check_in() { # <场景名>：在这个场景里单跑判法（本机当第二台），打它的输出
    env -u SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE SINGLEFS_MULTI_HOST_CONFIG="$work/$1/multi-host.env" SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 \
      bash "$work/$1/repo/scripts/crash/crash-point-replay-shard-configuration-check.sh" "$work/$1/repo" 2>&1
  }
  consistent_text="$(check_in carry)"
  printf '# 本机这一份改过：多了这一行\n' >> "$work/carry/multi-host.env"
  differing_text="$(check_in carry)"
  prepare_scene check-fresh
  missing_text="$(check_in check-fresh)"
  expect "Ⓨ 判法报两台配置一不一致：跑过一趟之后报一致、本机这一份改了之后报不一致、第二台没有常驻那一份时报还没有" \
    "$([[ "$consistent_text" == *"两台配置一致"* && "$differing_text" == *"两台配置不一致"* && "$missing_text" == *"第二台还没有常驻的那一份"* ]]; echo $?)" \
    "跑过一趟之后：${consistent_text##*；}；改了之后：${differing_text##*；}；没有常驻那一份：${missing_text##*；}"

  # Ⓩ 命令里的路径（文件头 ⑤「命令里的路径」）：--peer 的命令里有一个词是本机仓根（这个场景的 repo）、或是仓根下的文件 ⇒ 退 2，命令没跑、第二台 runs/ 下什么都没建；
  #   仓根之外的路径、名字只是以仓根起头的路径（repo-beside-ran）⇒ 照跑、退 0；不带 --peer 的本机跑带仓根下的绝对路径 ⇒ 照跑、退 0
  local absolute_runs_left
  prepare_scene absolute-refused
  scene="$work/absolute-refused"
  run_in absolute-refused root --peer --memory-cap 256M --label absolute-root -- bash -c 'touch "$1/root-ran"' absolute-root "$scene/repo"
  run_in absolute-refused file --peer --memory-cap 256M --label absolute-file -- bash -c 'touch "$0"' "$scene/repo/file-ran"
  absolute_runs_left="$(peer_runs_left absolute-refused)"
  expect "Ⓩ --peer 的命令里有以本机仓根起头的参数（仓根本身、仓根下的文件各一趟）⇒ 用法错（退 2，说以本机仓根起头、出路写树内相对路径），第二台 runs/ 下什么都没建、命令没跑" \
    "$([[ "$(exit_of absolute-refused root)" == 2 && "$(exit_of absolute-refused file)" == 2 && ! -e "$scene/repo/root-ran" && ! -e "$scene/repo/file-ran" && "$absolute_runs_left" == 0 ]] \
       && grep -q '以本机仓根起头的参数' "$scene/root.err" && grep -q '改成树内相对路径' "$scene/root.err" && grep -q '以本机仓根起头的参数' "$scene/file.err"; echo $?)" \
    "仓根本身那一趟退 $(exit_of absolute-refused root)，仓根下的文件那一趟退 $(exit_of absolute-refused file)；命令$([[ -e "$scene/repo/root-ran" || -e "$scene/repo/file-ran" ]] && echo 跑了 || echo 没跑)；第二台 runs/ 下留着 $absolute_runs_left 条；stderr 尾部：$(tail_of absolute-refused root)"
  run_in absolute-refused outside --peer --memory-cap 256M --label absolute-outside -- bash -c 'touch "$0"' "$scene/repo-beside-ran"
  run_in absolute-refused local --memory-cap 256M --label absolute-local -- bash -c 'touch "$0"' "$scene/repo/local-ran"
  expect "Ⓩ 命令里只有仓根之外的路径、名字只是以仓根起头的路径（repo-beside-ran）⇒ --peer 照跑、退 0；不带 --peer 的本机跑带仓根下的绝对路径 ⇒ 照跑、退 0" \
    "$([[ "$(exit_of absolute-refused outside)" == 0 && -e "$scene/repo-beside-ran" && "$(exit_of absolute-refused local)" == 0 && -e "$scene/repo/local-ran" ]]; echo $?)" \
    "--peer 那一趟退 $(exit_of absolute-refused outside)，文件$([[ -e "$scene/repo-beside-ran" ]] && echo 在 || echo 不在)；本机那一趟退 $(exit_of absolute-refused local)，文件$([[ -e "$scene/repo/local-ran" ]] && echo 在 || echo 不在)；stderr 尾部：$(tail_of absolute-refused outside)|$(tail_of absolute-refused local)"

  if (( failures > 0 )); then
    echo "  ✗ multi-host-run.sh 自证有 $failures 格判错（共 $checked 格）"  # gate-lint:summary
    echo "     → 怎么办：设了 MULTI_HOST_RUN_BREAK、MULTI_HOST_CONFIGURATION_BREAK 或 PEER_HOST_LIB_BREAK 的，这就是要的结果（那个开关把对应的一步弄坏了）；没设的，照上面那几格去看 scripts/hosts/multi-host-run.sh 与 peer-host-lib.sh"
    return 1
  fi
  echo "  ✓ multi-host-run.sh 自证 $checked 格都对（默认本机跑不读配置、--peer 跑成且退出码带回、第二台那棵树关了自动整理、上限不超过 PEER_MEMORY_CAP、工具链不同就拒、取回逐字相同、跑完删第二台的目录、收到 TERM 先停第二台、输出不露主机与目录、双机没开就拒、弄坏开关设在自证之外就拒、取回目录默认在工作根下而设了变量照变量、拷树排掉没跟踪的临时文件、只消失没跟踪的文件照跑而被跟踪的消失了判失败、包装认出显存不够时原样退 245 并报显存不够、设了跨趟共用编译目录的变量或 --peer 命令里 CARGO_TARGET_DIR 指到树外就拒、本机跑没设 CARGO_TARGET_DIR 时用这一趟自己建的编译目录且正常出错被 TERM 三种结局都删而调用方设的不删、--peer 时树里的 target 连同树删、删之前有进程开着里面的文件就不删并报出进程、第二台只用 1 条线程报在用核数过少并写告警记录而登记数那么多条线程不报、声明了串行段照报并标出理由、--peer 跑完本机代发 peer-run-finished 而测试开关下不发、那份库出事时退出码照旧、自证的细节行按字截、--peer 把本机这一份多机配置原样带到第二台并核 sha256、两台配置不同就拒、第二台认得自己是第二台而不再往第二台起活、起活这一侧清过场才告诉第二台、判法报两台配置一不一致）"
}

if [[ "${1:-}" == --selftest ]]; then
  multi_host_selftest
  exit $?
fi
if [[ -n "${MULTI_HOST_RUN_BREAK:-}" ]] && ! multi_host_break_is break-honored-anywhere && ! multi_host_inside_selftest; then
  echo "  ✗ MULTI_HOST_RUN_BREAK=$MULTI_HOST_RUN_BREAK 设在 --selftest 之外：弄坏开关只在自证起的驱动里生效，这一次什么都没做（退 2，命令一行都没跑）" >&2
  echo "     → 怎么办：去掉它再跑（env -u MULTI_HOST_RUN_BREAK bash scripts/hosts/multi-host-run.sh …）；要证红就跑 MULTI_HOST_RUN_BREAK=<项> bash scripts/hosts/multi-host-run.sh --selftest" >&2
  exit 2
fi
unset MULTI_HOST_RUN_SELFTEST_MARK
multi_host_main "$@"
