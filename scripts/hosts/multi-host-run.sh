#!/usr/bin/env bash
# 把一件确定性的活挪到第二台跑：只在第二台跑，--peer 必写。仓、工作根与会话记录目录经 NFS 两台挂在同一绝对路径
#   （多机配置 NFS_SHARED_ROOTS）：
#   第二台在这一刻的工作目录、同一个绝对路径里直接起，不拷树、不送、不取回、不改写路径，写出的文件两台都读得到。
#
#   multi-host-run.sh --peer [--memory-cap <上限>] [--label <标签>] [--keep-build-directory] [--reuse-build-directory <上一次的标签>] -- <命令与参数…>
#   multi-host-run.sh --remove-build-directory <标签>
#   multi-host-run.sh --selftest
# 留着的编译目录（task-server 撞上限重排时第二台那一次接着编，与本机同一段：task-server/task_server_lib.py 文件头「编译目录」的重试复用）：
#   --keep-build-directory：第二台那条命令等到了退出码的，⑥ 只删 runs/ 下三个文件与这一层退出时峰值那一份，这一趟的本地目录 builds/<标签>/（连同编译目录）留着，
#     回读它还在之后往本机标准错误打一行「MULTI_HOST_RUN_PEER_BUILD_KEPT label=<留着的那一份的标签>」（task-server 照它记这一次的编译目录没删）；
#     没等到退出码的（中途失败、被信号停）照 ⑥ 删。删之前照旧核没有进程开着里面的文件。
#   --reuse-build-directory <上一次的标签>：这一趟不新建，接着用第二台 builds/<上一次的标签>/（CARGO_TARGET_DIR 是它下面的 target、MULTI_HOST_RUN_PEER_SCRATCH 是它）；
#     起之前在第二台核它的 target 在（不在退 255、命令没跑），删掉里面上一次那一层退出时写的峰值；runs/ 下三个文件照这一趟的 --label 起名。
#     不带 --keep-build-directory 的跑完照 ⑥ 连它一起删。
#   --remove-build-directory <标签>：只删第二台 builds/<标签>/（task-server 换台或有了结局时调）：核没有进程开着里面的文件、rm、回读不在了退 0（本来就不在也退 0），
#     有进程开着或核不成退 1，多机配置载不了、第二台连不上退 255。
#   弄坏开关 peer-build-not-kept 下带 --keep-build-directory 也照 ⑥ 删。
# 分段计时：每一步开始与结束各往标准错误打一行（秒数在这个进程里计，EPOCHREALTIME，一位小数），一行一样：
#   MULTI_HOST_RUN_STEP step=<步> event=begin elapsed_seconds=<自驱动开跑的秒数>
#   MULTI_HOST_RUN_STEP step=<步> event=end elapsed_seconds=<…> seconds=<这一步用的秒数> exit=<这一步的退出码；remove-build 是 0 删了、1 没删>
#   MULTI_HOST_RUN_STEP step=<步> event=skip elapsed_seconds=<…> reason=<为什么没做这一步>
#   带 --peer 的步：peer-command（⑤ 起第二台那条命令到等到结局）、remove-build（⑥ 删第二台这一趟的本地目录，连同编译目录与进程号、退出码、输出三个文件）。
#   第二台那条命令里编与跑各占几秒，由起它的调用方在命令里自己计（例 crash-amplification-two-hosts.sh 的 peer-build、peer-run 两步）。
#
# 不带 --peer 是用法错（退 2，命令没跑）：本机的任务经 task-server 交，服务在本机直接起 run-with-memory-cap.sh（task-server/task_server.py 文件头 ③「起」），
#   不经这里；这里只管服务派到第二台的那一趟。弄坏开关 peer-flag-unrequired 下不带也当带了 --peer 跑。
# 带 --peer：
#   ① 配置经 scripts/hosts/peer-host-lib.sh 的 peer_host_load_configuration 读（判法是 scripts/crash/crash-point-replay-shard-configuration-check.sh，
#      配置 ${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}，模板是仓根 multi-host.env.example）；判不过退 255。
#      读之前先单跑一次判法看双机开关（ENABLE_ACROSS_MACHINES，定义在那份判法里；判法退 3 是「双机：关」）：没开就拒、退 255，
#      出路是在配置里写一行 ENABLE_ACROSS_MACHINES=1，不偷偷退回本机跑（双机要参数开启）。
#      只收服务直接起的那一次（两台对称，不靠认台的角色变量）：task-server 起派第二台的尝试时只给最外层这一层设
#      MULTI_HOST_RUN_ADMITTED_BY_TASK_SERVER=<任务号>.<第几次尝试>（task-server/task_server.py 文件头 ③「环境」）；这里开头读一次就 unset，
#      本机这一侧往下起的子进程看不到它，第二台那条命令只拿 ⑤「带到第二台的环境」点名的几样，也看不到。没有它（任务里面的命令再带 --peer，在哪台都一样）
#      就拒、退 255，什么都没跑、哪台都没连，出路是把这条命令交给 task-server。判在用法之后、读配置之前。
#      弄坏开关 nested-peer-accepted 下没有它也照收；admission-mark-forwarded 下把它原样带进第二台那条命令。
#   ② 上限默认取多机配置 SLICE_MEMORY_CAP_BY_HOST 里第二台那一项（那一台 slice 的总上限；按第二台在 HOSTS 里的地址取，peer-host-lib.sh 文件头「按台的三个键」）；
#      --memory-cap 给的比它大就拒（退 2）。
#   ③ 共享根（连第二台之前判；判不过退 255、第二台上什么都没起。task-server 在提交时照同一个读法当场拒（task_server.py 的 find_outside_shared_path，task submit 退 3），
#      经服务派来的到不了这里）：
#      ① 读到的 NFS_SHARED_ROOTS（冒号分隔）列的根；下面几样都要落在某一个根下面（等于根或以「根/」起头；本机这一侧取真路径）：
#        这一刻的工作目录；
#        -- 之后命令的每个词里的绝对路径（multi_host_outside_shared_path；与 task-server/task_server.py 的 find_outside_shared_path 同一个读法，只认整个 shell 词是绝对路径的）：
#          以 / 起头的整个词（此刻在的、带空白的绝对路径照整个词）、「名字=绝对路径」「--选项=绝对路径」的值；bash、sh 这类后面带 -c（-ec、-lc 同）的脚本那一个词
#          照 shell 词切开，每个以 / 起头的 shell 词、「名字=」的值与挨着 ;|&<>( 写的下一个词；别的词中间夹着的 /（--item 的值「…（接的是 /）」这类）、
#          脚本里引号中空白后面的 /、跟在 }、) 后面的 /…（${…}、$(…) 展开之后的尾巴）、./x、../x 与 a:/x 里夹着的都不算；脚本引号不配对、切不开的照子串取（多拒不漏）；
#          两台相同的系统路径不判（MULTI_HOST_HOST_INDEPENDENT_PATHS：/dev/null、/dev/stdin、
#          /dev/stdout、/dev/stderr、/dev/fd/ 下、/usr/ 下、/bin/ 下）；相对路径照工作目录解，工作目录在共享根下它们也在；
#        本机这一份多机配置（第二台那条命令照同一个绝对路径读它，⑤「带到第二台的环境」）；
#        工作根与共享数据根（都经 .claude/hooks/lib_work_root.py 取：SINGLEFS_DATA_ROOT，没设取多机配置的 SHARED_DATA_ROOT，都没有那份库报错、这里退 255）：两个都照本机的真路径交给第二台那条命令
#          （第二台的家目录是另一个，不经 ~ 取）。
#      有一样不在：退 255，一句话点名是哪一样、值是什么、共享根有哪几个，出路是把它放进共享根、或多机配置加上它所在的根；不跨机就经 task-server 交、由服务挑台。
#      弄坏开关 outside-shared-paths-accepted 下不判工作目录与命令里的路径。
#   ④ 两台工具链比一遍（peer_host_compare_toolchains：rustc -Vv 前三行与 host 行、cargo -V，另比 rustup component list --installed 里 rustfmt、clippy 那两行），
#      不同就拒、说清差在哪，哪一台缺 rustfmt 或 clippy 按工具链不同拒、点名缺的组件，都退 255（scripts/hosts/peer-host-lib.sh 文件头「工具链比对」）。
#      离线依赖（peer_host_check_offline_dependencies，peer-host-lib.sh 文件头「离线依赖」）：工作目录所在 git 工作树的根（不是 git 仓的取工作目录）有 Cargo.toml 的，
#      在第二台那里空跑 cargo metadata --offline，缺依赖就报「第二台缺依赖 X」与同步办法、拒、退 255（命令没跑）；缺的是树里的 Cargo.toml 写成 path、目录又不在的，
#      报「树里缺 path 依赖 <路径>，两台都编不了」与出路（把那个目录补进树），同样退 255；没有 Cargo.toml 的不核。
#   ⑤ 在第二台跑：第二台的编译根是多机配置 BUILD_ROOT_BY_HOST 里第二台那一项（第二台本地盘上，按地址取）。这一趟在它下面新建一个目录 <编译根>/builds/<标签>/
#      （下面叫这一趟的本地目录），编译目录是它下面的 target；进程号、退出码、输出三个文件写第二台 <编译根>/runs/<标签>.pid / .exit / .log。编译根照字面落在共享根下就拒、退 255
#      （编译目录与这几份文件不许落在 NFS 上：两台各编各的、这一趟用完就删）。这一趟的本地目录或那三个文件已在是标签撞了，用法错退 2；
#      查的那一步 ssh 自己退 255 是第二台连不上，退 255、不当用法错。
#      第二台那条命令：env -C <这一刻的工作目录> 起（同一个绝对路径），环境里 CARGO_TARGET_DIR=<这一趟的本地目录>/target、
#      MULTI_HOST_RUN_PEER_SCRATCH=<这一趟的本地目录>（命令要在第二台本地盘上放临时东西的照它放，⑥ 连同编译目录删），外面套仓里的 run-with-memory-cap.sh <上限>
#      （同一个绝对路径；那一层只有这一条的上限与 slice 的总上限，不排队、不记账：排队与记账只在本机的 task-server；那一层只认服务起的那一层，
#      这里照 ① 读到的服务给这一次的标记给它设 RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER，它照旧从命令的环境里摘掉）。弄坏开关 peer-build-in-shared-tree 下不设
#      CARGO_TARGET_DIR（cargo 照默认在共享的树里编）；peer-runs-in-scratch 下不在这一刻的工作目录里起，在这一趟的本地目录里起。
#      编译目录（自己用完自己删，不跨趟复用）：--peer 时命令的词里（连同 bash -c 的脚本）写了 CARGO_TARGET_DIR= 是用法错（退 2，命令没跑；
#      multi_host_command_target_in）：编译目录由这里在第二台本地盘上建、⑥ 删。弄坏开关 command-target-accepted 下照跑。
#      脱离 ssh 会话起（peer_host_start_detached），每 MULTI_HOST_RUN_POLL_SECONDS（默认 5）秒经 ssh 看一次（peer_host_wait_detached，每一轮之后调 multi_host_follow_peer）。
#      跑着时接回（task-server 看本机这一次尝试的日志涨不涨、CPU 涨不涨判卡住，第二台的任务也要看得见）：每一轮多一次 ssh，读第二台输出文件此刻的字节数与
#      那条命令进程树里各 cgroup 的 cpu.stat usage_usec；输出比上一轮多出来的那一截取回来，到最后一个换行为止（半行留到下一轮，换字按整行换）经 peer_host_redact
#      接着打到本机标准错误；等到结局之后只补最后没接回的那一截（不以换行收尾的补一个换行），已接回的不再打。
#      每一轮（连同看到结局的那一轮）往本机标准错误打一行，格式固定：
#        MULTI_HOST_RUN_PEER_CPU poll=<轮数：从 1 起每一轮加 1> usage_usec=<累计微秒> read=<yes|no>
#      usage_usec 是第二台那条命令进程树里的进程所在、而起它的那个 bash（进程号文件里那一个）不在的每个 cgroup（经 run-with-memory-cap.sh 起的就是它的
#      singlefs-memory-cap-*.scope，命令里再套一层包装的另算一个）见过的最大 usage_usec 之和：scope 收尾、读不到的照它最后一次读到的数算，所以一轮比一轮不减；
#      read=yes 是这一轮读到了至少一个那样的 cgroup，no 是这一轮一个都没读到（还没起、已经收尾、ssh 不通、不经内存包装跑）。读这一行的一方（task-server）
#      峰值：每一轮另读那条命令进程树里头一个不在起它的 bash 那个 cgroup 里的 cgroup（父在子前数，就是第二台那一层内存包装 run-with-memory-cap.sh 的
#      singlefs-memory-cap-*.scope）进程用的内存：memory.stat 的 anon 加 shmem（同 scripts/hosts/memory_peaks_lib.py 的 process_memory_bytes、本机 task-server/attempt_runner.py
#      文件头 ③，两台同一种量；页缓存与内核挂在 scope 上的不算：编验证包时 anon 921M、file 22163M，memory.peak 顶到上限 50176M，页缓存撑满上限时内核回收它、
#      不因此杀进程），留见过的最大。等到退出码之后再经 ssh 读一次第二台那一层内存包装在命令退出之后读的那一个
#      （这里给那一层设 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT=<这一趟的本地目录>/exit-memory-peak，run-with-memory-cap.sh 文件头「task-server 给这一层的两样」：scope 收尾之前一定读到一次
#      scope 的 memory.peak），按轮一次都没读到（命令比一轮短）才照它；在「第二台那条命令退 N」那一句之前往本机标准错误打一行，格式固定：
#        MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=<十进制字节数>
#      两样都没有（不经内存包装跑、读不了）才不打这一行。没等到退出码的（第二台连不上、进程没了）不打。
#      读这一行的是 task-server（按它记第二台那一次的内存峰值）。弄坏开关 peer-memory-peak-unreported 下不打；peer-memory-peak-unread-printed 下没读到也打 bytes=0；
#      peak-read-only-by-sampling 下只靠按轮采（不给那一层设这个变量、不读退出时那一个）；peer-memory-with-page-cache 下每一轮读 memory.peak 与 memory.current
#      （含页缓存）、与退出时那一个取大的。
#      只供自证：MULTI_HOST_RUN_SELFTEST_EXIT_PEAK_FILE=<文件名> 把退出时读的那一份的文件名换成它（造「读不到」那几格），
#      MULTI_HOST_RUN_SELFTEST_MEMORY_DIRECTORY=<绝对路径> 把每一轮读 memory.stat（弄坏开关下 memory.peak、memory.current）的目录换成它（假 cgroup：造「页缓存撑满上限」那一格，
#      只收 /、字母、数字与 ._- 组成的绝对路径），只在 --selftest 起的驱动里认，别处设了不认、读完就 unset。
#      编译目录占盘：峰值那一行之后、⑥ 删之前，在第二台上量这一趟给命令的 CARGO_TARGET_DIR（builds/<标签>/target）的占盘（du -s -b，字节），往本机标准错误打一行，格式固定：
#        MULTI_HOST_RUN_PEER_BUILD_PEAK bytes=<十进制字节数>
#      量不到（目录不在了、du 没跑成、ssh 不通）就不打；读这一行的是 task-server。弄坏开关 peer-build-peak-unreported 下不打；peer-build-peak-unread-printed 下量不到也打 bytes=0。
#      编译段峰值：每一轮接回的那一截整行里找 cargo 编完打的那一行（MULTI_HOST_CARGO_FINISHED_PATTERN，与 task-server/task_server_lib.py 的 CARGO_FINISHED_PATTERN 同一个式子），
#      还没见到的这一轮读到的进程内存（上面「峰值」每一轮读的那一个）收进编译那一段的最大，见到那一行的那一轮与之后的不收（同本机 task-server/attempt_runner.py 文件头 ③）；
#      结局之后补接回的那一截里才有那一行的照样算见到。见到了那一行、那一行之前至少一轮读到了的，在 MULTI_HOST_RUN_PEER_MEMORY_PEAK 那一行之后打一行，格式固定：
#        MULTI_HOST_RUN_PEER_BUILD_PHASE_PEAK bytes=<十进制字节数>
#      没见到那一行（编不过、不是 cargo）或之前一轮都没读到的不打；读这一行的是 task-server（编译族只借它，task_server_lib.py 文件头「资源记录」的编译族）。
#      弄坏开关 peer-build-phase-peak-unreported 下不打。
#      此刻内存：就是上面「峰值」每一轮读的那一个（那一层内存包装的 scope 这一轮的 anon 加 shmem），每一轮紧跟在 MULTI_HOST_RUN_PEER_CPU 那一行之后往本机标准错误打一行，格式固定：
#        MULTI_HOST_RUN_PEER_MEMORY poll=<与紧挨着的 CPU 那一行同一个轮数> current_bytes=<这一轮读到的 anon 加 shmem，没读到写 0> peak_bytes=<见过的最大，没读到写 0> read=<yes|no>
#      read=yes 是这一轮读到了 memory.stat 的 anon 或 shmem，no 是没读到（还没起、已经收尾、ssh 不通、不经内存包装跑）：没读到也照打、两项写 0（peak_bytes 在前面几轮读到过的照写读到过的最大），不漏行。
#      只在跑着的每一轮打，等到结局之后的那一段不打。读这一行的是 task-server（第二台顶在内存上限上、日志不涨而 CPU 在涨的，按它判撞上限；read=no 时 current_bytes 不算数）。
#      两边的格式一起改：task-server/task_server.py 的 PEER_MEMORY_LINE 认 ^MULTI_HOST_RUN_PEER_MEMORY poll=([0-9]+) current_bytes=([0-9]+) peak_bytes=([0-9]+) read=(yes|no)$。
#      弄坏开关 peer-memory-unreported 下不打；peer-memory-unread-skipped 下没读到 current 的这一轮不打这一行（漏行）。
#      只供自证：MULTI_HOST_RUN_SELFTEST_CURRENT_FILE=<文件名> 把每一轮读 memory.stat 的文件名换成它（造「读不到」那一格），认的地方与读完 unset 同 MULTI_HOST_RUN_SELFTEST_EXIT_PEAK_FILE。
#      等 IO（task-server 判卡住之前看这一次的进程是不是卡在不可中断的 IO 等待上：本机根盘被写满时本机导出的 NFS 跟着慢，第二台上读写共享根的进程停在 D 状态，
#      日志不涨、CPU 也不涨，是在等 IO 不是死了）：同一次 ssh 里另数那条命令进程树里（连同进程号文件里那个 bash 自己）/proc/<进程号>/stat 状态字段是 D 的进程，
#      每一轮紧跟在 MULTI_HOST_RUN_PEER_MEMORY 那一行之后往本机标准错误打一行，格式固定：
#        MULTI_HOST_RUN_PEER_IO_WAIT poll=<与这一轮 CPU 那一行同一个轮数> blocked=<这一轮 D 状态的进程数，没读到写 0> read=<yes|no>
#      read=yes 是这一轮读到了进程树（进程号那个 bash 还在），no 是没读到（还没起、已经收尾、ssh 不通）：没读到也照打、blocked 写 0，不漏行；只在跑着的每一轮打。
#      读这一行的是 task-server（日志与 CPU 都满窗口不涨、这一个窗口里每一轮 blocked 都大于 0 且 read=yes 的不判卡住；read=no 时 blocked 不算数）。
#      两边的格式一起改：task-server/task_server.py 的 PEER_IO_WAIT_LINE 认 ^MULTI_HOST_RUN_PEER_IO_WAIT poll=([0-9]+) blocked=([0-9]+) read=(yes|no)$。
#      弄坏开关 peer-io-wait-unreported 下不打；peer-io-wait-uncounted 下读到了也写 blocked=0。
#      只供自证：MULTI_HOST_RUN_SELFTEST_BLOCKED_STATES=<大写字母…> 把数的状态从 D 换成它们（造「有进程在等」那一格：S 是在睡，空转的命令底下总有在睡的父进程），
#      只收大写字母，认的地方与读完 unset 同 MULTI_HOST_RUN_SELFTEST_EXIT_PEAK_FILE。
#      按 usage_usec 涨不涨判「在算」；这一行、MULTI_HOST_RUN_PEER_MEMORY 行、MULTI_HOST_RUN_PEER_IO_WAIT 行与 MULTI_HOST_RUN_STEP 行每一轮都打，判日志涨不涨时不算它们。
#      换字（peer_host_redact）读到不是合法 UTF-8 的字节（命令按字节截断了汉字、打了二进制）时原样放过那几个字节，其余照换，前后的行照打。
#      换字那一步自己退非 0（python3 起不来、被杀）时，那一块（第二台输出文件的第 N 到 M 字节）一个字也不往本机打（没换字的私有字不许漏出去），
#      本机标准错误打一行「! 第二台那条命令的输出第 N 到 M 字节换字时退了 <码>，这一块没打到这里、也不再重打…原样的输出在第二台 runs/ 下的 <标签>.log」，已接回的字节数照旧往前推（不重打）。
#      只报一行、不整条判错：这一步只管日志上显示什么，第二台那条命令的退出码、输出文件与峰值都没受影响，整条判错会让一趟编了几小时的活白跑、被服务重排。弄坏开关 peer-redact-failure-unreported 下不报这一行。
#      弄坏开关 peer-output-at-end-only 下跑着时不接回、等到结局才整份打；peer-output-resent 下结局时从头再打一遍；
#      peer-output-partial-lines 下不留半行、读到多少打多少；peer-cpu-unreported 下不打 CPU 那一行；peer-cpu-not-kept 下 usage_usec 只算这一轮读到的。
#      带到第二台的环境：本机的环境变量不带（真的第二台经 ssh 也拿不到），要设写成 -- env 变量=值 <命令>；只带 peer_host_carried_environment 打的几样：
#      SINGLEFS_MULTI_HOST_CONFIG=<本机这一份多机配置的绝对路径>（在共享根下，第二台读同一份，③；不带认台的角色变量，两台对称）；
#      这个驱动正经 scripts/crash/gpu-quiesce-run.sh 起（起活这一侧照配置清了场、回读过了）时另加 GPU_QUIESCE_ORIGIN_READ_BACK=1（第二台上的清场包装认它，见那一份文件头「第二台」）；
#      另带 SINGLEFS_WORK_ROOT、SINGLEFS_DATA_ROOT（③ 判过的本机真路径；弄坏开关 shared-roots-not-passed 下不带）与上面的 CARGO_TARGET_DIR、MULTI_HOST_RUN_PEER_SCRATCH。
#      现场目录：服务给这一次尝试的 RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY（共享工作根下，两台同一个绝对路径；task-server/task_server_lib.py 文件头「现场」）开头读了就 unset，
#      原样设给第二台那一层内存包装（不进命令）：那条命令退非 0 留下的临时目录由那一层直接搬进这个目录，本机照同一个路径读（弄坏开关 peer-scene-not-passed 下不设，
#      留在第二台的 /tmp 下）。服务给本机这一层的 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT 同样开头 unset（本机这一侧不套内存包装）。
#      在用核数不在这里取：每次尝试用了几个核由 task-server 记进它的 status（records/2026-10-30-编译绕过内存总上限.md「收尾批」）。
#      RocksDB 预编库的取值：命令里有 cargo（有一个词是 cargo、以 /cargo 结尾，或 bash -c 的脚本里有单独的 cargo 这个词；multi_host_command_starts_cargo）时，
#      第二台起它之前先跑 python3 <仓根>/scripts/hosts/rocksdb-prebuilt.py entry --root <④ 那棵树的根>（判第二台装了、ssh 起的这条命令拿到的取值与第二台的预编库一致），
#      判红退 78、命令不跑，红句随输出接回本机；命令整条包在 bash -c '<判> && exec "$@"' 里交给上面几样。不含 cargo 的命令不判。
#   ⑥ 命令结束之后（连同 ③ 以后在哪一步失败退出、被信号停，与 --recover-cleanup）删第二台这一趟的本地目录（连同编译目录）与 runs/ 下那三个文件，回读不在了才算删了：
#      删之前先在第二台停这一趟内存包装的 scope（「status=confirmed reason=processes-gone」之后那一层 scope 里仍可能挂着进程，它们清了环境、换了会话号，
#      照会话号与标签找不到）：⑤ 起第二台那条命令时给那一层内存包装设 RUN_WITH_MEMORY_CAP_UNIT_OUTPUT=runs/<标签>.scope（进程号文件旁边；run-with-memory-cap.sh 文件头
#      「multi-host-run.sh 给这一层的一样」），⑥ 照它 systemctl --user stop 那一个 scope、回读 is-active 的输出是 inactive 或 failed、cgroup.procs 是空的（peer-host-lib.sh 的
#      peer_host_stop_attempt_scope），⑥ 那一行另写停 scope 时里面还有几个；停了还有进程的不删、不打 ⑥、逐个报出留下的进程（换掉私有的字），清理回执记 pending、
#      原因 scope-processes-remain、scope_leftovers 记那几个进程（task-server 照它把 cleanup-pending 那一条提醒推给交任务的会话，修改意见是核实脚本有没有不等子进程就退），
#      驱动退 255（命令退的不是 0 时照命令的）；单元名认不出、ssh 没跑成的同样不删、记 pending。弄坏开关 peer-scope-unchecked 下不停 scope。
#      再停这一趟留下的进程（multi_host_stop_peer_leftovers：会话号是进程号文件里那个号的、环境里带 TASK_SERVER_ATTEMPT_LABEL=<这一趟的标签> 的，
#      后代自己 setsid 出去的照标签找得到；peer-host-lib.sh 文件头「这一趟留下的进程」；停了 scope 之后这一步只当补充），⑥ 那一行写停了几个；还有停不掉的、读不了 /proc 的不删、不打 ⑥、报出来、
#      清理回执记 pending（占用照留），驱动退 255（命令退的不是 0 时照命令的）。弄坏开关 leftovers-left-at-cleanup 下不停。
#      再在第二台核没有进程开着里面的文件（仓里 scripts/hosts/run_build_directory.py holders）：有就是那条命令的子进程没收干净，不删、逐个报进程（换掉私有的字）、
#      驱动退 255（命令退的不是 0 时照命令的）。删不成同样退 255。弄坏开关 no-peer-cleanup 下不删。
# 收到 TERM / INT / HUP（退 143 / 130 / 129）：先停那条命令——从它在第二台上记下的进程号起（peer_host_stop_detached），
#   连同全部后代冻住、从最底层起 TERM、放开，至多等 30 秒，还在的 KILL，再停这一趟留下的进程；停干净了而起的那个 bash 来不及写退出码的，由停的这一方补写退出回执
#   （peer-host-lib.sh 文件头「停下的退出回执」）——再删第二台这一趟的本地目录（⑥），做完才退。
# --recover-cleanup <清理回执>（只由 task-server 在本机这一层包装结束之后调，task_server.py 的 confirm_peer_cleanup）：回执还是 pending 的，照回执里记的标签与三个文件
#   在第二台再停一次、再删（⑥），停干净、删成、回执写成 confirmed 才退 0。起的那个 bash 不在、又没有退出回执的，照同一判法核这一趟留下的进程：一个不剩的当场判已停、
#   补写退出回执；还有的先停、停完再核（peer_host_stop_detached）。弄坏开关 orphan-session-unverified 下：起的那个 bash 不在而没有回执就不核、退 1、回执留在 pending。
# 退出码：那条命令的退出码（245 与 250–254 是内存包装自己的结局，含义见 run-with-memory-cap.sh 文件头「退出码」，那一次的输出不算结果）；
#   78 带 --peer、命令里有 cargo 而第二台的 RocksDB 预编库取值判红（⑤，命令没跑）；
#   2 用法错（含不带 --peer、命令里写了 CARGO_TARGET_DIR=、标签撞了），
#     或弄坏开关 MULTI_HOST_RUN_BREAK 设在 --selftest 之外（命令一行都没跑）；
#   255 这个脚本自己没跑成（不是服务直接起的那一次（①）、配置、判不了 --peer：工作目录、命令里的路径、多机配置、工作根或共享数据根不在共享根下（③），第二台的编译根（BUILD_ROOT_BY_HOST 里它那一项）在共享根下、
#      查标签时第二台连不上、工具链（含缺 rustfmt、clippy 组件）、第二台缺离线依赖、起不来、等不到结局、删不掉这一趟的本地目录）。
# 驱动打的每一行都不写配置里第二台的主机、别名与目录：第二台的输出与错误输出经 peer_host_redact 换成「第二台」「第二台的目录」再打。
# 共用库 scripts/hosts/peer-host-lib.sh 只调它的对外函数。
#
# gate-similar: crash-point-replay-shard-run.sh 它把一条登记了分片的崩溃枚举用例切成两片、两台各跑一片再 merge 并写 crash-point-replay 的全绿标记，步骤与输入指纹都绑在那一类用例上；它按内容进三条用例的输入指纹（admission.py 的 SHARD_DRIVER_FILES），并进来改一行就废三格标记
# gate-similar: mutation-shard-run.sh 它是 crates-mutation-replay 专用的变异表分行驱动：只由 task-server 按分片协议起，按用时分两堆、服务把两堆当两片各起一趟 crates-mutation-replay、合并时核过判定库记录再导入，它自己不连第二台（只经 peer-host-lib.sh 读多机配置），判定归 crates-mutation-replay；这里只挪一条任意命令，不分行、不导记录
#
# 弄坏开关 MULTI_HOST_RUN_BREAK=<项>（只给证红用，逗号分隔可给几个；打开时自证退 1）。只在 --selftest 起的驱动里生效：自证在它的临时目录里放一份标记文件，
#   路径经 MULTI_HOST_RUN_SELFTEST_MARK 带给它起的驱动，文件在、内容是 MULTI_HOST_SELFTEST_MARK_CONTENT 那一行才认开关（驱动读完就 unset，不带给命令）；
#   别处设了它就拒、退 2，命令一行都没跑。逐项写开关名、弄坏哪一处、打开后自证哪几格红
# （项名是自证 expect 的格名，一行一项；失败行是「✗ <格名>：<细节>」，括号里抄细节的句首，随跑变的路径、进程号写成 …）：
#   MULTI_HOST_RUN_BREAK=peer-flag-unrequired  不带 --peer 也当带了 --peer 跑（不拒）
#       Ⓐ 不带 --peer ⇒ 用法错（退 2，说本机的任务经 task-server 交），命令没跑、第二台一个目录都没建（「退 0，命令跑了」）
#   MULTI_HOST_RUN_BREAK=peer-exit-dropped  第二台那条命令退几都退 0
#       Ⓑ --peer 退出码原样带回：第二台那条命令退 7 ⇒ 驱动退 7（「退 0」）
#       Ⓞ 第二台起含 cargo 的命令之前判第二台的取值：取值缺了 ⇒ 退 78、假 cargo 没起；不含 cargo 的命令不判、照跑退 0（「含 cargo 的退 0、假 cargo 没起…」）
#   MULTI_HOST_RUN_BREAK=peer-without-memory-cap  第二台那条命令不经仓里的 run-with-memory-cap.sh 跑
#       Ⓑ --peer 经仓里的 run-with-memory-cap.sh 跑（cgroup 在 singlefs-memory-cap- 的 scope 里）（「命令记下的 cgroup：0::/user.slice/…」）
#       连带红的格：「✗ 接回 每一轮一行 MULTI_HOST_RUN_PEER_CPU …」（没有 scope：末行 usage_usec=0、一轮都没读到）、
#         「✗ Ⓚ --peer：第二台的内存包装认出显存不够 ⇒ 原样退 245、报「显存不够」、说那一次的输出不算结果」（不经包装：命令自己的退 3 原样带回）、
#         「✗ 峰值 第二台那条命令跑 0.1 秒就退、每一轮都读不到 memory.stat ⇒ …」（不经包装：退出时没人写峰值）、
#         「✗ 现场目录 驱动环境里给了 RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY、… ⇒ 退 7、那一份由第二台那一层内存包装搬成这个目录」（不经包装：没人搬）
#   MULTI_HOST_RUN_BREAK=no-peer-cap-limit  --memory-cap 比多机配置 SLICE_MEMORY_CAP_BY_HOST 里第二台那一项大也照跑
#       Ⓑ --memory-cap 2G 比多机配置 SLICE_MEMORY_CAP_BY_HOST 里第二台那一项 1G 大 ⇒ 拒（退 2），命令没跑（「退 0，命令跑了」）
#   MULTI_HOST_RUN_BREAK=no-toolchain-check  不比两台工具链（连同 rustfmt、clippy 组件）
#       Ⓒ 第二台的 rustc -Vv 不同 ⇒ 拒（退 255，说两台的工具链不同），命令没跑、第二台没建这一趟的目录（「退 0，命令跑了」）
#       Ⓒ 第二台缺 clippy 组件（rustup component list --installed 里没有那一行）⇒ 拒（退 255，说两台的工具链不同、第二台缺组件 clippy），命令没跑、第二台没建这一趟的目录（「退 0，命令跑了」）
#   MULTI_HOST_RUN_BREAK=no-peer-cleanup  跑完（成功、失败、被停都算）不删第二台这一趟的本地目录与 runs/ 下三个文件
#       Ⓔ Ⓑ 那几趟（含退 7 的那一趟）跑完，第二台 runs/ 与 builds/ 下什么都不剩（「Ⓑ 那边留着：…」）
#       Ⓕ 第二台那条命令在跑时收到 TERM ⇒ 退 143，第二台那条命令不在了，runs/ 与 builds/ 下什么都不剩（「退 143，第二台那条命令的进程号「…」不在了；留着 4 条」）
#       Ⓠ --peer：第二台那条命令拿到的 CARGO_TARGET_DIR 是第二台这一趟的本地目录下的 target（不在共享根下），共享的树里没建 target，跑完那个目录删了（「退 0，…那个目录还在」）
#       Ⓡ --peer：第二台那条命令退了、还有进程开着这一趟本地目录里的文件 ⇒ ⑥ 不删、报出那个进程、退 255（「退 0，这一趟的本地目录还在」）
#       Ⓦ --peer 跑成：第二台那条命令、删这一趟的本地目录每一步都打了开始与结束两行，结束行的秒数是非负数（「退 0；缺的：remove-build」）
#   MULTI_HOST_RUN_BREAK=no-peer-stop-on-signal  收到 TERM 不停第二台那条命令
#       Ⓕ 第二台那条命令在跑时收到 TERM ⇒ 退 143，第二台那条命令不在了，runs/ 与 builds/ 下什么都不剩（「退 143，第二台那条命令的进程号「…」还活着」）
#   MULTI_HOST_RUN_BREAK=no-redact  第二台的输出不经 peer_host_redact 就打
#       Ⓖ 输出里没有配置里的主机名与第二台的目录，命令打的那两样换成了「第二台」「第二台的目录」（「有的：peer/run.err」）  history-kept: 自证格名原文引用，按原文匹配
#       连带红的两格：「✗ 换字 第二台那条命令的输出里夹一行不是合法 UTF-8 的字节…」（地址没换）、「✗ 换字 换字那一步自己退非 0…」（没报，地址漏出来）
#   MULTI_HOST_RUN_BREAK=switch-off-falls-back-to-local  配置里双机没开时退回本机直接跑那条命令
#       Ⓗ 配置里 ENABLE_ACROSS_MACHINES=0 ⇒ --peer 拒（退 255，出路写 ENABLE_ACROSS_MACHINES=1），命令哪台都没跑、第二台没建目录（「退 0，命令跑了」）
#   MULTI_HOST_RUN_BREAK=break-honored-anywhere  弄坏开关在 --selftest 之外也认
#       Ⓘ MULTI_HOST_RUN_BREAK 设在 --selftest 之外（不带自证的标记）⇒ 拒（退 2，说 MULTI_HOST_RUN_BREAK 设在 --selftest 之外），命令没跑
#         （「退 2，命令没跑；stderr 尾部：…SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 设在 --selftest 之外…」：开关放过去之后被只供测试那一道拒，拒的不是这一道）
#   MULTI_HOST_RUN_BREAK=gpu-memory-unexplained  那条命令退 245（内存包装认出的显存不够）时不报「显存不够」
#       Ⓚ --peer：第二台的内存包装认出显存不够 ⇒ 原样退 245、报「显存不够」、说那一次的输出不算结果（「退 245，stderr 尾部：…」）
#   MULTI_HOST_RUN_BREAK=peer-rocksdb-unchecked  第二台起含 cargo 的命令之前不判第二台的 RocksDB 预编库取值
#       Ⓞ 第二台起含 cargo 的命令之前判第二台的取值：取值缺了 ⇒ 退 78、假 cargo 没起；不含 cargo 的命令不判、照跑退 0（「含 cargo 的退 0、假 cargo 起了…」）
#   MULTI_HOST_RUN_BREAK=command-target-accepted  不拒 --peer 命令里写的 CARGO_TARGET_DIR=（编译目录跟着命令走，可能落在共享根下、⑥ 删不到）
#       Ⓜ --peer 命令里写了 CARGO_TARGET_DIR=（env 的赋值、bash -c 的脚本两种写法）⇒ 用法错（退 2），命令没跑（「env 那一趟退 0，bash -c 那一趟退 …」）
#   MULTI_HOST_RUN_BREAK=this-machine-peer-honored-anywhere  SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 设在 --selftest 之外也认
#       Ⓘ 只供测试的 SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 设在 --selftest 之外（不带自证的标记）⇒ 拒（退 2，说只在 --selftest 里认），命令没跑（「退 0，命令跑了」）
#   MULTI_HOST_RUN_BREAK=open-files-unchecked  删第二台这一趟的本地目录之前不核有没有进程开着里面的文件
#       Ⓡ --peer：第二台那条命令退了、还有进程开着这一趟本地目录里的文件 ⇒ ⑥ 不删、报出那个进程、退 255（「退 0，这一趟的本地目录删了」）
#   MULTI_HOST_RUN_BREAK=detail-cut-by-bytes  自证的细节行按字节截（cut -c），汉字截成半个
#       Ⓣ 自证的细节行按字截：比上限长的一行截到上限那么多个字，截出来是合法的 UTF-8（「161 字节，第 159 字节起解不成 UTF-8」）
#   MULTI_HOST_RUN_BREAK=step-lines-unmarked  ⑤ 第二台那条命令那一步不打 MULTI_HOST_RUN_STEP 的开始与结束两行（文件头「分段计时」）
#       Ⓦ --peer 跑成：第二台那条命令、删这一趟的本地目录每一步都打了开始与结束两行，结束行的秒数是非负数（「退 0；缺的：peer-command」）
#   MULTI_HOST_RUN_BREAK=label-unreachable-as-usage-error  ⑤ 查标签那一步 ssh 自己退 255 也当标签已有、用法错退 2
#       标签 --peer 查标签时第二台连不上（ssh 自己退 255）⇒ 退 255、说连不上，不当用法错，命令没跑、builds/ 下没建这一趟的目录（「退 2，命令没跑，builds/ 下没建这一趟的目录」）
#   MULTI_HOST_RUN_BREAK=label-taken-as-unreachable  ⑤ 标签已有也当第二台连不上、退 255
#       标签 --peer 的 --label 在第二台 builds/ 下已经有同名目录 ⇒ 用法错（退 2），命令没跑、那个目录还在（「退 255，命令没跑，那个目录还在」）
#   MULTI_HOST_RUN_BREAK=scene-slice-cap-unwritten  自证的场景配置不写 SLICE_MEMORY_CAP_BY_HOST（判法判不过）；自证停在这一格，后面的格不跑
#       场景配置 场景的多机配置过得了判法（--emit-assignments 退 0），SLICE_MEMORY_CAP_BY_HOST 里这一台那一项照 slice 此刻的总上限写
#         （「判法退 1；场景配置里没写 SLICE_MEMORY_CAP_BY_HOST…」）
#   MULTI_HOST_RUN_BREAK=peer-output-at-end-only  ⑤ 跑着时不接回第二台的输出，等到结局才整份打
#       接回 第二台那条命令打的每一行在它还跑着的时候就出现在本机标准错误里（「missed-1 missed-2 missed-3」）
#   MULTI_HOST_RUN_BREAK=peer-output-resent  ⑤ 等到结局时从头再打一遍第二台的输出（跑着时接回过的又打一次）
#       接回 本机标准错误里第二台那条命令打的行逐行同序、不重不漏，半行与不以换行收尾的末行都接成整行（「读到的：follow-line-1|…|follow-line-1|…」）
#   MULTI_HOST_RUN_BREAK=peer-output-partial-lines  ⑤ 跑着时读到多少打多少，不留到最后一个换行（半行被别的行插断）
#       接回 本机标准错误里第二台那条命令打的行逐行同序、不重不漏…（「读到的：…|follow-part-MULTI_HOST_RUN_PEER_CPU poll=…」）
#       连带红的格：「✗ 接回 每一轮一行 MULTI_HOST_RUN_PEER_CPU …」（被插断的那一轮的 CPU 行不在行首：「格式不对：follow-part-MULTI_HOST_RUN_PEER_CPU …」）
#   MULTI_HOST_RUN_BREAK=peer-redact-failure-unreported  ⑤ 换字那一步退非 0 时不报这一块没打出来：悄悄丢掉，字节数照样往前推
#       换字 换字那一步自己退非 0 ⇒ 驱动照旧退 0、本机标准错误里报这一块没打出来、没有没换字的私有字（「报的行 0 行（要至少 1 行）」）
#   MULTI_HOST_RUN_BREAK=peer-cpu-unreported  ⑤ 每一轮不打 MULTI_HOST_RUN_PEER_CPU 那一行
#       接回 每一轮一行 MULTI_HOST_RUN_PEER_CPU poll=<从 1 起连着的号> usage_usec=<数> read=<yes|no>，usage_usec 一轮比一轮不减、末行至少 1000000、有一轮读到了（「只有 0 行 …」）
#       连带红的格：「✗ 接回 每一轮一行 MULTI_HOST_RUN_PEER_MEMORY …」「✗ 接回 读不到 memory.stat 的每一轮照打 …」（MEMORY 行没有 CPU 行在前：「第 … 行的 MEMORY 前一行不是 CPU」）
#         「✗ 峰值 每一轮读不到 memory.stat、退出时读不到 memory.peak …」（那一趟数 read=yes 的 CPU 行：「read=yes 的轮 0 轮」）
#         「✗ 等 IO 每一轮一行 MULTI_HOST_RUN_PEER_IO_WAIT poll=<与这一轮 CPU 那一行同一个轮数>」（IO_WAIT 行前面没有这一轮的 CPU 行：「第 … 行的 IO_WAIT 前面没有这一轮的 CPU」；推的，没复跑）
#   MULTI_HOST_RUN_BREAK=peer-cpu-not-kept  ⑤ usage_usec 只算这一轮读到的 cgroup，不留见过的最大（scope 收尾之后那一轮掉回 0）
#       接回 每一轮一行 MULTI_HOST_RUN_PEER_CPU …（「usage_usec 减了：… → 0 …」）
#   MULTI_HOST_RUN_BREAK=peer-memory-unreported  ⑤ 每一轮不打 MULTI_HOST_RUN_PEER_MEMORY 那一行
#       接回 每一轮一行 MULTI_HOST_RUN_PEER_MEMORY poll=… current_bytes=… peak_bytes=… read=…：紧跟每行 CPU 之后、轮数与那一行 CPU 相同、至少一轮 read=yes 且 current_bytes 大于 0（「只有 0 行 MEMORY …」）
#       连带红的格：「✗ 接回 读不到 memory.stat 的每一轮照打 …」（「MEMORY 行 0 行 …」）、「✗ 峰值 页缓存撑满上限 …」（「read=yes 的 MEMORY 行 0 行」；推的，没复跑）
#   MULTI_HOST_RUN_BREAK=peer-memory-unread-skipped  ⑤ 没读到 memory.stat 的那一轮不打 MULTI_HOST_RUN_PEER_MEMORY 那一行（漏行）
#       接回 读不到 memory.stat 的每一轮照打 MULTI_HOST_RUN_PEER_MEMORY …current_bytes=0 peak_bytes=0 read=no：每一轮 CPU 之后都有，不漏行（「MEMORY 行 0 行 …」）
#       连带红的格：「✗ 接回 每一轮一行 MULTI_HOST_RUN_PEER_MEMORY …」（经真的内存包装跑的那一趟，scope 还没起的头一轮与收尾之后的几轮也读不到 current，同样漏行：「第 … 行的 CPU 后面没跟 MEMORY」）
#   MULTI_HOST_RUN_BREAK=peer-io-wait-unreported  ⑤ 每一轮不打 MULTI_HOST_RUN_PEER_IO_WAIT 那一行（文件头 ⑤「等 IO」）
#       「✗ 等 IO 每一轮一行 MULTI_HOST_RUN_PEER_IO_WAIT poll=<与这一轮 CPU 那一行同一个轮数>」（「只有 0 行 IO_WAIT」）
#       「✗ 等 IO 数的状态换成 S（在睡）⇒ 至少一轮 read=yes 且 blocked 大于 0」（「blocked 大于 0 且 read=yes 的轮 0 轮」）
#   MULTI_HOST_RUN_BREAK=peer-io-wait-uncounted  ⑤ 读到了进程树也写 blocked=0（不数进程的状态；文件头 ⑤「等 IO」）
#       「✗ 等 IO 数的状态换成 S（在睡）⇒ 至少一轮 read=yes 且 blocked 大于 0」（「blocked 大于 0 且 read=yes 的轮 0 轮」）
#   MULTI_HOST_RUN_BREAK=peer-memory-peak-unreported  ⑤ 第二台那条命令结束时不打 MULTI_HOST_RUN_PEER_MEMORY_PEAK 那一行
#       峰值 第二台那条命令结束时打恰好一行 MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=<正整数>（那一层内存包装 cgroup 进程内存见过的最大），在最后一行 MULTI_HOST_RUN_PEER_CPU 之后（「峰值行 0 行」）
#       连带红的格：「✗ 峰值 第二台那条命令跑 0.1 秒就退、每一轮都读不到 memory.stat ⇒ …」（「峰值行「」」）、「✗ 峰值 页缓存撑满上限 …」（「峰值行 0 行：」；推的，没复跑）
#   MULTI_HOST_RUN_BREAK=peer-build-peak-unreported  ⑤ 第二台那条命令结束时不打 MULTI_HOST_RUN_PEER_BUILD_PEAK 那一行
#       编译目录占盘 --peer：第二台那条命令结束时打恰好一行 MULTI_HOST_RUN_PEER_BUILD_PEAK bytes=<正整数>（这一趟本地编译目录删之前 du -s -b 量的）（「占盘行 0 行」）
#   MULTI_HOST_RUN_BREAK=peer-build-peak-unread-printed  ⑤ 量不到编译目录的占盘也打一行 bytes=0
#       编译目录占盘 --peer：命令自己删了编译目录（量不到）⇒ 退 0、不打 MULTI_HOST_RUN_PEER_BUILD_PEAK 那一行（「占盘行 1 行：MULTI_HOST_RUN_PEER_BUILD_PEAK bytes=0」）
#   MULTI_HOST_RUN_BREAK=peer-build-phase-peak-unreported  ⑤ 第二台那条命令结束时不打 MULTI_HOST_RUN_PEER_BUILD_PHASE_PEAK 那一行（文件头 ⑤「编译段峰值」）
#       编译段峰值 接回那一趟第二台那条命令打了 cargo 编完那一行 ⇒ 恰好一行 MULTI_HOST_RUN_PEER_BUILD_PHASE_PEAK bytes=<正整数>、不大于峰值行的数（「编译段峰值行 0 行」）
#   MULTI_HOST_RUN_BREAK=peer-memory-peak-unread-printed  ⑤ 每一轮没读到 memory.stat、退出时没读到 memory.peak 也打一行 bytes=0
#       峰值 每一轮读不到 memory.stat、退出时读不到 memory.peak（两处读的文件名都换成不在的）⇒ 退 0、有一轮 read=yes、不打 MULTI_HOST_RUN_PEER_MEMORY_PEAK 那一行（「峰值行 1 行：MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=0」）
#   MULTI_HOST_RUN_BREAK=peak-read-only-by-sampling  ⑤ 只靠按轮采：不给第二台那一层内存包装设 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT、等到退出码之后不读退出时那一个
#       峰值 第二台那条命令跑 0.1 秒就退、每一轮都读不到 memory.stat ⇒ 退 0、照那一层内存包装退出时读的打恰好一行 MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=<正整数>（「峰值行「」」）
#   MULTI_HOST_RUN_BREAK=peer-memory-with-page-cache  ⑤ 每一轮读那一层内存包装 cgroup 的 memory.current 与 memory.peak（含页缓存），峰值与退出时读的取大的
#       峰值 页缓存撑满上限（假 cgroup：anon 1G、file 40G、memory.peak 顶到三项之和）⇒ 每一轮 current_bytes 与 peak_bytes 都是 anon 的 1G、峰值行 bytes=1G
#         （「不是 current_bytes=1073741824 peak_bytes=1073741824：…」）
#       连带红的格（推的，没复跑：这种读法不看 MULTI_HOST_RUN_SELFTEST_CURRENT_FILE）：「✗ 峰值 每一轮读不到 memory.stat、退出时读不到 memory.peak …」（「峰值行 1 行 …」）、
#         「✗ 接回 读不到 memory.stat 的每一轮照打 …」（「不是 current_bytes=0 peak_bytes=0 read=no：…」）
#   MULTI_HOST_RUN_BREAK=peer-scene-not-passed  ⑤ 不把服务给的 RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY 设给第二台那一层内存包装（现场留在第二台的 /tmp 下）
#       现场目录 驱动环境里给了 RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY、第二台那条命令往 TMPDIR 写了文件又退 7 ⇒ 退 7、那一份由第二台那一层内存包装搬成这个目录（「退 7，… 里「」」）
#   MULTI_HOST_RUN_BREAK=outside-shared-paths-accepted  ③ 不判工作目录与命令里的路径在不在共享根下（在第二台上照同一路径读写，读到的是它自己的那一份或什么都没有）
#       Ⓩ --peer 命令里有共享根外的绝对路径（整个词、bash -c 词里各一趟）⇒ 退 255、点名那个路径，命令没跑、第二台没建这一趟的目录（「整个词那一趟退 0…」）
#       Ⓩ --peer 在共享根外的工作目录里起 ⇒ 退 255、说工作目录不在共享根下，命令没跑（「退 0，命令跑了」）
#   MULTI_HOST_RUN_BREAK=expansion-tail-as-path  ③ bash -c 脚本词切出的 shell 词里跟在 )、} 后面的 /…（${…}、$(…) 展开之后的尾巴）也当绝对路径
#       Ⓩ bash -c 脚本词照 shell 词切：跟在 }、) 后面的 /…（${…}、$(…) 的尾巴）、./x、../x 与 a:/x 里夹着的 / 不当绝对路径…（「「cp "${X:?}/release/a" /w/x」取到「/release/a」…」）
#   MULTI_HOST_RUN_BREAK=peer-path-check-substring  ③ 每个带空白的词都按子串取绝对路径（参数值中间夹着的 /、a:/x 里的 / 也当路径）
#       Ⓩ 参数值中间夹着的 /（--item 的值、bash -c 引号里的「（接的是 /）」）不当绝对路径…（「--item 的值「crash-case-replay（接的是 /）」取到「/）」…」）
#       连带红的格：「✗ Ⓩ bash -c 脚本词照 shell 词切：…」（「「echo a:/opt/z」取到「/opt/z」…」）
#   MULTI_HOST_RUN_BREAK=shared-roots-not-passed  ⑤ 不给第二台那条命令设 SINGLEFS_WORK_ROOT、SINGLEFS_DATA_ROOT（第二台照它自己的家目录取，落在它本地盘上）
#       Ⓢ --peer：第二台那条命令拿到本机的工作根与共享数据根（SINGLEFS_WORK_ROOT、SINGLEFS_DATA_ROOT 照本机的真路径）（「第二台那条命令看到的：work=没设 data=没设」）
#   MULTI_HOST_RUN_BREAK=peer-directory-unchecked  ⑤ 不判第二台的编译根（BUILD_ROOT_BY_HOST 里它那一项）落不落在共享根下（编译目录落在 NFS 上）
#       Ⓢ --peer：多机配置 BUILD_ROOT_BY_HOST 里第二台那一项落在共享根下 ⇒ 退 255、说这一趟的本地目录不许落在共享根下，命令没跑（「退 0，命令跑了」）
#   MULTI_HOST_RUN_BREAK=peer-build-in-shared-tree  ⑤ 第二台那条命令不设 CARGO_TARGET_DIR（cargo 照默认在共享的树里编，两台编到同一个 target 里）
#       Ⓠ --peer：第二台那条命令拿到的 CARGO_TARGET_DIR 是第二台这一趟的本地目录下的 target（不在共享根下），共享的树里没建 target，跑完那个目录删了（「退 0，第二台那条命令拿到「没设」…」）
#   MULTI_HOST_RUN_BREAK=peer-runs-in-scratch  ⑤ 第二台那条命令不在这一刻的工作目录里起，在这一趟的本地目录里起（相对路径落到第二台本地盘上）
#       Ⓑ --peer 跑成：退 0，在第二台同一个绝对路径（这一刻的工作目录，共享的那棵树）里跑，HEAD 与本机相同（「退 0，在「…/builds/peer-ok」跑…」）
#       连带红的格：「✗ Ⓦ 第二台那条命令里判同一份场景配置，结局（退出码与 --emit-assignments 打的赋值）与这一侧判的相同，再起 multi-host-run.sh --peer 照拒（退 255），一次都没往别的台 ssh」
#         （相对路径的判法在本地目录里不在，退 127）、
#         「✗ 双机驱动那种起法经 --peer 真跑一趟（在共享的树里、照同一个绝对路径起）⇒ 退 0、第二台的假 cargo 先编后跑起了两次、都在本机那棵树里起」
#   MULTI_HOST_RUN_BREAK=selftest-caller-toolchain  自证不放假 rustc、cargo，场景里用 PATH 上调用方的那一份（场景的 HOME 是自证自己的 home/、不带 CARGO_HOME 与 RUSTUP_HOME，rustup 的代理找不到工具链）
#       经 --peer 走到 ④ 的 31 格都红，细节里 stderr 尾部是「本机 rustc / cargo 报不出版本」，例：
#       Ⓑ --peer 跑成：退 0，在第二台同一个绝对路径（这一刻的工作目录，共享的那棵树）里跑，HEAD 与本机相同（「退 255，在「」跑…」）
#       Ⓒ 第二台的 rustc -Vv 不同 ⇒ 拒（退 255，说两台的工具链不同），命令没跑、第二台没建这一趟的目录（「退 255，命令没跑；stderr 尾部：…」）
#       双机驱动那种起法经 --peer 真跑一趟（在共享的树里、照同一个绝对路径起）⇒ 退 0、第二台的假 cargo 先编后跑起了两次、都在本机那棵树里起（「退 255；假 cargo 收到「」…」）
#       PATH 上的 rustc、cargo 不是 rustup 的代理（直接装的工具链、不读 RUSTUP_HOME）时这一项不红
#   MULTI_HOST_RUN_BREAK=nested-peer-accepted  ① 环境里没有服务给最外层那一次的 MULTI_HOST_RUN_ADMITTED_BY_TASK_SERVER 也照收（任务里面再带 --peer 照往别的台起活）
#       Ⓥ 任务里面再带 --peer（这一台：起它的不是服务的尝试单元，没有最外层那一次的标记）⇒ 拒（退 255，出路写把这条命令交给 task-server），命令没跑、第二台没建这一趟的目录（「退 0，命令跑了」）
#       Ⓦ 第二台那条命令里判同一份场景配置，结局（退出码与 --emit-assignments 打的赋值）与这一侧判的相同，再起 multi-host-run.sh --peer 照拒（退 255），一次都没往别的台 ssh
#         （「…里层 --peer 退 255…ssh 记了 2 次」：里层照收，判法与读配置各往配置里的另一台 ssh 一次）
#   MULTI_HOST_RUN_BREAK=admission-mark-forwarded  ⑤ 把最外层那一次的 MULTI_HOST_RUN_ADMITTED_BY_TASK_SERVER 原样带进第二台那条命令（里层再带 --peer 也认成服务起的）
#       Ⓦ 第二台那条命令里判同一份场景配置…再起 multi-host-run.sh --peer 照拒（退 255），一次都没往别的台 ssh（「…里层 --peer 退 255…ssh 记了 2 次」）
#   MULTI_HOST_RUN_BREAK=peer-build-not-kept  ⑥ 带 --keep-build-directory 也照删这一趟的本地目录（第二台每一趟新建、跑完删）
#       留着 --keep-build-directory 跑完留着这一趟的编译目录、打 MULTI_HOST_RUN_PEER_BUILD_KEPT；下一趟 --reuse-build-directory 接着用的就是它（「…留着那一份 不在…」）
#   MULTI_HOST_RUN_BREAK=orphan-session-unverified  起的那个 bash 不在、又没有退出回执时不照会话号与尝试标签核，一律不按已停处理（peer_host_stop_detached 的 keep）
#       收拾 --recover-cleanup：起的那个 bash 被杀、没写退出回执、后代都已不在 ⇒ 一次就确认（退 0）、补写退出回执、清理回执记 confirmed、第二台 runs/ 与 builds/ 下什么都不剩（「退 1，…」）
#       收拾 --recover-cleanup：起的那个 bash 被杀、没写退出回执、后代还活着 ⇒ 停掉它们之后确认（退 0）、补写退出回执、后代不在了、第二台 runs/ 与 builds/ 下什么都不剩（「退 1，…」）
#   MULTI_HOST_RUN_BREAK=leftovers-left-at-cleanup  ⑥ 删之前不停这一趟留下的进程（后代自己 setsid 出去的那一棵没人收）
#       收拾 --peer：命令退了、后代自己 setsid 出去还在跑（环境里带这一趟的标签）⇒ ⑥ 删之前停掉它、⑥ 那一行写停了 1 个、退 0（「退 0，…还活着…」）
#   MULTI_HOST_RUN_BREAK=peer-scope-unchecked  ⑥ 不先停这一趟内存包装的 scope，只照会话号与标签找留下的进程
#       收拾 --peer：那一层内存包装被 KILL、scope 里留着清了环境又换了会话号的进程 ⇒ ⑥ 先停那个 scope、那个进程不在了、⑥ 那一行写停 scope 时里面还有 1 个（「退 137，…还活着…」）
# 判法的开关 MULTI_HOST_CONFIGURATION_BREAK 也带进这份自证，打开后红哪几格写在 scripts/crash/crash-point-replay-shard-configuration-check.sh 文件头；
# 共用库的开关 PEER_HOST_LIB_BREAK 同样带进来，打开后红哪几格写在 scripts/hosts/peer-host-lib.sh 文件头。
# 只供测试的开关：SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1（照崩溃点重放分片）：「第二台」是本机上的另一个目录，命令在本机跑、环境照 ssh 那样只给最少几样。
#   只在 --selftest 起的驱动里认（MULTI_HOST_RUN_SELFTEST_MARK 对得上；自证另 export peer-host-lib.sh 认的 PEER_HOST_SELFTEST_MARK）：别处设了就拒、退 2，命令一行都没跑。
#   弄坏开关 this-machine-peer-honored-anywhere 下别处设了也照认。
#
# admission: always 每次调都在这一刻这棵树上跑一条调用方给的命令；要不要重跑由调用方判，不在这里
# run-condition: command bash git rsync python3
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
multi_host_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=peer-host-lib.sh
source "$multi_host_script_directory/peer-host-lib.sh"

# 两台相同的系统路径（文件头 ③「共享根」）：命令里点到它们不判在不在共享根下；以 / 收尾的是整个目录
MULTI_HOST_HOST_INDEPENDENT_PATHS='/dev/null /dev/stdin /dev/stdout /dev/stderr /dev/fd/ /usr/ /bin/'
# 跑着时接回（文件头 ⑤「跑着时接回」）：已接回本机的字节数、看了几轮、各 cgroup 见过的最大 usage_usec（键是 cgroup 路径）；multi_host_peer_main 起第二台那条命令之前清零
multi_host_peer_forwarded_bytes=0
multi_host_peer_polls=0
declare -A multi_host_peer_cgroup_usage=()
# 峰值（文件头 ⑤「峰值」）：第二台那一层内存包装 cgroup 进程用的内存（anon 加 shmem）见过的最大（空串是一轮都没读到）
multi_host_peer_memory_peak=""
# 编译段峰值（文件头 ⑤「编译段峰值」）：见到 cargo 编完那一行之前每一轮读到的进程内存见过的最大（空串是一轮都没读到），见没见到那一行
multi_host_peer_build_phase_peak=""
multi_host_peer_build_finished=0
MULTI_HOST_CARGO_FINISHED_PATTERN='^[[:space:]]*Finished[[:space:]].*target\(s\) in '   # task-server/task_server_lib.py 的 CARGO_FINISHED_PATTERN 的 ERE 写法，两边一起改
multi_host_peer_current_file=memory.stat   # 每一轮读进程内存的文件名（文件头 ⑤「此刻内存」；只供自证换，文件头同一段）
multi_host_peer_memory_directory=""        # 每一轮读进程内存的目录，空是那一层内存包装的 cgroup（只供自证换成假 cgroup，文件头 ⑤「峰值」）
multi_host_peer_blocked_states=D   # 每一轮数进程树里哪几种状态的进程（文件头 ⑤「等 IO」；D 是不可中断的 IO 等待，只供自证换）
MULTI_HOST_PEER_EXIT_PEAK_NAME=exit-memory-peak   # 第二台那一层内存包装退出时写峰值的那一份，在这一趟的本地目录下（文件头 ⑤「峰值」）
multi_host_keep_build=0       # --keep-build-directory（文件头「留着的编译目录」）；--recover-cleanup、--remove-build-directory 不留
peer_command_exited=0         # 第二台那条命令等到了退出码：只有这样才照 --keep-build-directory 留着
multi_host_peer_exit_peak_file="$MULTI_HOST_PEER_EXIT_PEAK_NAME"   # 等到退出码之后读的那一份的文件名（只供自证换成不在的）

multi_host_break_is() { [[ ",${MULTI_HOST_RUN_BREAK:-}," == *",$1,"* ]]; }

# 分段计时（文件头「分段计时」）：秒数在这个进程里计，一位小数；一步一个开始时刻
multi_host_now_microseconds() {
  if [[ -n "${EPOCHREALTIME:-}" ]]; then printf '%s' "${EPOCHREALTIME/[.,]/}"; else date +%s%6N; fi
}
multi_host_seconds_text() { # <微秒数>：打成一位小数的秒
  local tenths=$(( ($1 + 50000) / 100000 ))
  printf '%d.%d' $(( tenths / 10 )) $(( tenths % 10 ))
}
multi_host_started_microseconds="$(multi_host_now_microseconds)"
declare -A multi_host_step_begun=()
multi_host_step() { # <步> <begin|end|skip> [<end 的退出码，或 skip 的理由>] [<end 另接的字段>]：往标准错误打一行 MULTI_HOST_RUN_STEP；弄坏开关 step-lines-unmarked 下第二台那条命令那一步不打
  local step="$1" event="$2" now elapsed fields
  if [[ "$step" == peer-command ]] && multi_host_break_is step-lines-unmarked; then return 0; fi
  now="$(multi_host_now_microseconds)"
  elapsed="$(multi_host_seconds_text $(( now - multi_host_started_microseconds )))"
  case "$event" in
    begin) multi_host_step_begun[$step]="$now"; fields="event=begin elapsed_seconds=$elapsed" ;;
    end) fields="event=end elapsed_seconds=$elapsed seconds=$(multi_host_seconds_text $(( now - ${multi_host_step_begun[$step]:-$now} ))) exit=${3:-0}${4:+ $4}" ;;
    *) fields="event=skip elapsed_seconds=$elapsed reason=${3:-none}" ;;
  esac
  echo "MULTI_HOST_RUN_STEP step=$step $fields" >&2
}

# 共享根外的路径（文件头 ③「共享根」）：第一个参数是 NFS_SHARED_ROOTS，第二个是两台相同的系统路径，第三个是逗号连的弄坏开关（只认 expansion-tail-as-path、
#   peer-path-check-substring，别的当没设），往后是要判的命令的词；打第一个落在共享根外的绝对路径（本机这一侧取真路径），都在打空串。
#   词的读法与 task-server/task_server.py 的 find_outside_shared_path 相同（shell_script_word_indexes、shell_word_paths 两段照抄在这里：这一层在本机起、不经服务，
#   不载服务的库）：只认整个 shell 词是绝对路径的。整个词以 / 起头的、「名字=绝对路径」「--选项=绝对路径」的值；bash / sh 这类的 -c 脚本那一个词照 shell 词切开，
#   每个以 / 起头的 shell 词、「名字=」的值与挨着 ;|&<>( 写的下一个词；别的词中间夹着的 /（--item 的值「…（接的是 /）」这类）不算，-c 脚本里引号中的整段是一个词，
#   空白后面的 / 同样不算；${…}、$(…) 展开之后的尾巴不算；-c 脚本引号不配对、切不开的照子串取（多拒不漏）。
MULTI_HOST_OUTSIDE_PATH_PROGRAM='import os, re, shlex, sys
roots = [os.path.realpath(root) for root in sys.argv[1].split(":") if root]
independent = sys.argv[2].split()
variants = sys.argv[3].split(",")
words = sys.argv[4:]
shells = ("bash", "sh", "dash", "zsh", "ksh")
options_with_value = ("-o", "+o", "-O", "+O", "--rcfile", "--init-file")
substring = re.compile(r"(?<![\w/}).])/[^\s\x27\x22`;|&()<>]+")
# 弄坏开关 expansion-tail-as-path 下跟在 )、} 后面的（展开之后的尾巴）也取
word_path = re.compile(r"(?:^|(?<=[=;|&<>()}]))/[^;|&()<>`]+" if "expansion-tail-as-path" in variants else r"(?:^|(?<=[=;|&<>(]))/[^;|&()<>`]+")

def script_indexes(command):
    indexes = set()
    for index, word in enumerate(command):
        if os.path.basename(word) not in shells:
            continue
        cursor, script = index + 1, False
        while cursor < len(command) and command[cursor][:1] in ("-", "+") and command[cursor] != "--":
            option = command[cursor]
            if option in options_with_value:
                cursor += 1
            elif not option.startswith("--") and "c" in option[1:]:
                script = True
            cursor += 1
        if cursor < len(command) and command[cursor] == "--":
            cursor += 1
        if script and cursor < len(command):
            indexes.add(cursor)
    return indexes

def script_paths(script):
    try:
        lexer = shlex.shlex(script, posix=True)
        lexer.whitespace_split = True
        lexer.commenters = ""
        pieces = list(lexer)
    except ValueError:
        return substring.findall(script)
    return [path for piece in pieces for path in word_path.findall(piece)]

def shared(path):
    real = os.path.realpath(path)
    return any(real == root or real.startswith(root.rstrip("/") + "/") for root in roots)

def host_independent(path):
    return any(path == entry or (entry.endswith("/") and path.startswith(entry)) for entry in independent)

scripts = script_indexes(words)
for index, word in enumerate(words):
    if word.startswith("/") and os.path.lexists(word):
        candidates = [word]
    elif "peer-path-check-substring" in variants and any(character.isspace() for character in word):
        candidates = substring.findall(word)
    elif index in scripts:
        candidates = script_paths(word)
    else:
        value = word if word.startswith("/") else word.partition("=")[2]
        candidates = [value] if value.startswith("/") else []
    for path in candidates:
        if not host_independent(path) and not shared(path):
            print(path)
            sys.exit(0)'
multi_host_outside_shared_path() { # <词…>：打第一个落在共享根外的绝对路径（文件头 ③）；都在共享根下打空串；python3 没跑成交回非 0
  local variants="" variant
  for variant in expansion-tail-as-path peer-path-check-substring; do
    if multi_host_break_is "$variant"; then variants+="${variants:+,}$variant"; fi
  done
  python3 -c "$MULTI_HOST_OUTSIDE_PATH_PROGRAM" "${NFS_SHARED_ROOTS:-}" "$MULTI_HOST_HOST_INDEPENDENT_PATHS" "$variants" "$@"
}
multi_host_require_shared() { # <是什么> <路径>：路径落在共享根外就退 255（文件头 ③「共享根」：判不了 --peer，第二台上什么都没起）
  local outside
  [[ "$2" == /* ]] || multi_host_fail "判不了 --peer：没取到$1的绝对路径（读到「$2」）：第二台上什么都没起" "照上面那一步的出路修（多机配置由 peer_host_load_configuration 读出它的路径）"
  outside="$(multi_host_outside_shared_path "$2")" \
    || multi_host_fail "判不了$1 $2 在不在共享根下（python3 没跑成）：第二台上什么都没起" "看 python3 在不在 PATH 上"
  [[ -z "$outside" ]] && return 0
  multi_host_fail "判不了 --peer：$1 $2 不在多机配置 NFS_SHARED_ROOTS 的任何一个根下（${NFS_SHARED_ROOTS:-一个根都没写}），第二台照同一个绝对路径读写，读不到本机这一份；第二台上什么都没起" \
    "把它放进共享根下，或多机配置的 NFS_SHARED_ROOTS 加上它所在的根（两台都挂在同一绝对路径，task-server/README.md「## NFS 共享」）；不跨机就经 task-server 交、由服务挑台"
}
multi_host_under_shared_root() { # <第二台上的路径>：照字面落在某一个共享根下交回 0（第二台上的路径，本机这一侧不取真路径）
  local root
  local -a roots=()
  IFS=: read -r -a roots <<<"${NFS_SHARED_ROOTS:-}"
  for root in ${roots[@]+"${roots[@]}"}; do
    [[ -n "$root" ]] || continue
    root="${root%/}"
    [[ "$1" == "$root" || "$1" == "$root"/* ]] && return 0
  done
  return 1
}

# 服务只给最外层那一次设的标记（文件头 ①「只收服务直接起的那一次」）：开头读一次、马上 unset，往下起的子进程（本机这一侧与第二台那条命令）都看不到
multi_host_admitted_by_task_server="${MULTI_HOST_RUN_ADMITTED_BY_TASK_SERVER:-}"
unset MULTI_HOST_RUN_ADMITTED_BY_TASK_SERVER
# 服务给这一次尝试的现场目录（文件头 ⑤「现场目录」）：开头读了就 unset，只设给第二台那一层内存包装；本机这一侧不套包装，退出时峰值那一份也 unset
multi_host_scene_directory="${RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY:-}"
unset RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY RUN_WITH_MEMORY_CAP_PEAK_OUTPUT

MULTI_HOST_SELFTEST_MARK_CONTENT="multi-host-run.sh --selftest 起的驱动"
multi_host_inside_selftest() { # 这个驱动是不是 --selftest 起的：MULTI_HOST_RUN_SELFTEST_MARK 指的文件在、第一行对得上（文件头「弄坏开关」那一段）
  [[ -n "${MULTI_HOST_RUN_SELFTEST_MARK:-}" && -f "$MULTI_HOST_RUN_SELFTEST_MARK" ]] \
    && [[ "$(head -n 1 -- "$MULTI_HOST_RUN_SELFTEST_MARK")" == "$MULTI_HOST_SELFTEST_MARK_CONTENT" ]]
}

multi_host_usage_error() { # <原因>：用法错，命令一行都没跑，退 2
  echo "  ✗ multi-host-run.sh 用法错：$1" >&2
  echo "     → 怎么办：multi-host-run.sh --peer [--memory-cap <上限>] [--label <标签>] [--keep-build-directory] [--reuse-build-directory <上一次的标签>] -- <命令与参数…>、--remove-build-directory <标签>，或 --selftest；本机的任务经 task-server 交（task-server/task submit -- <命令>），不经这里；上限写正整数加 K / M / G / T，标签只用字母、数字、点、减号与下划线；第二台在同一棵共享的树里跑，不送、不取回（产物写在共享根下两台都读得到）" >&2
  exit 2
}

multi_host_fail() { # <原因> <出路>：这个脚本自己没跑成，退 255（EXIT 上删第二台这一趟的本地目录）
  echo "  ✗ multi-host-run.sh：$1" >&2
  echo "     → 怎么办：$2" >&2
  exit 255
}

multi_host_judge_label_free() { # <查标签那条命令的退出码> <标签已有时的用法错原因>：0 往下走；255 是 ssh 自己连不上（被重置、超时），第二台没跑成、退 255；别的非 0 是标签已有，用法错退 2
  # 弄坏开关 label-unreachable-as-usage-error 下 255 也当用法错，label-taken-as-unreachable 下标签已有也当连不上
  local status="$1"
  if multi_host_break_is label-taken-as-unreachable && (( status != 0 )); then status=255; fi
  if multi_host_break_is label-unreachable-as-usage-error && (( status == 255 )); then status=1; fi
  (( status == 255 )) && multi_host_fail "第二台连不上（ssh 退 255）：命令没跑" "等第二台的 sshd 缓过来再交；服务把这一次记 peer-run-failed、重排"
  (( status == 0 )) || multi_host_usage_error "$2"
}

multi_host_cap_bytes() { # <上限>：正整数加 K / M / G / T，打成字节数
  local number="${1%[KMGT]}" power
  case "${1: -1}" in K) power=1 ;; M) power=2 ;; G) power=3 ;; *) power=4 ;; esac
  echo $(( number * 1024 ** power ))
}

multi_host_redact() { # 第二台的输出：经 peer_host_redact 换掉私有的字再打；弄坏开关 no-redact 下原样打
  if multi_host_break_is no-redact; then cat; else peer_host_redact; fi
}

multi_host_record_cleanup() { # <pending|confirmed> <原因> [<停了 scope 还留着的进程，一行一个、换过私有的字>]：先落本机尝试目录，再发日志；未知结局跨服务重启仍占卡。
  # 第三样给了的记进清理回执的 scope_leftovers（文件头 ⑥：task-server 照它推 cleanup-pending 给交任务的会话）
  local status="$1" reason="$2" leftovers="${3:-}"
  echo "MULTI_HOST_RUN_CLEANUP time_us=$(multi_host_now_microseconds) pid=$$ label=$run_label status=$status reason=$reason" >&2
  [[ -n "${TASK_SERVER_PEER_CLEANUP_FILE:-}" ]] || return 0
  python3 - "$TASK_SERVER_PEER_CLEANUP_FILE" "$status" "$reason" "$run_label" \
    "$peer_start_directory" "$peer_scratch" "$peer_pid_file" "$peer_exit_file" "$peer_output_file" "${peer_boot_id:-}" "${peer_processes_stopped:-0}" "$leftovers" <<'PY'
import json, os, sys, time
path, status, reason, label, directory, scratch, pid, exited, output, boot_id, stopped, leftovers = sys.argv[1:]
record = dict(status=status, reason=reason, label=label, directory=directory, scratch=scratch,
              pid_file=pid, exit_file=exited, output_file=output, boot_id=boot_id,
              processes_stopped=stopped == "1", updated_at=time.time())
if leftovers.strip():
    record["scope_leftovers"] = [line for line in leftovers.splitlines() if line.strip()]
temporary = path + ".partial"
with open(temporary, "w", encoding="utf-8") as handle:
    json.dump(record, handle)
    handle.flush()
    os.fsync(handle.fileno())
os.replace(temporary, path)
directory_fd = os.open(os.path.dirname(path) or ".", os.O_RDONLY | os.O_DIRECTORY)
try:
    os.fsync(directory_fd)
finally:
    os.close(directory_fd)
PY
}

multi_host_stop_pending_peer() {
  local stop_output
  (( peer_command_started )) || return 0
  if [[ -n "${peer_boot_id:-}" ]]; then
    local current_boot
    current_boot="$(peer_host_run / 'cat /proc/sys/kernel/random/boot_id')" || return 1
    if [[ -n "$current_boot" && "$current_boot" != "$peer_boot_id" ]]; then
      peer_command_started=0
      peer_processes_stopped=1
      multi_host_record_cleanup pending reboot-confirmed
      return $?  # 旧开机的全部进程已不在，仍须检查目录持有者。
    fi
  fi
  # 起的那个 bash 不在、又没有退出回执的照会话号与尝试标签核（peer-host-lib.sh 文件头「停下的退出回执」）；弄坏开关 orphan-session-unverified 下不核
  local unverified=""
  multi_host_break_is orphan-session-unverified && unverified=keep
  if stop_output="$(peer_host_stop_detached "$peer_start_directory" "$peer_pid_file" "$peer_exit_file" "$run_label" "$unverified")"; then
    echo "  · 第二台那条命令：$stop_output" >&2
    peer_command_started=0
    peer_processes_stopped=1
    multi_host_record_cleanup pending process-tree-stopped
    return $?
  fi
  echo "  ✗ 第二台那条命令尚未确认停止：$stop_output；保留进程号、退出码、日志和目录，继续占用该卡" >&2
  echo "     → 怎么办：照上面留下的进程号到第二台核它还在不在；停住之后再收这一片，没停住之前那张卡别交给别的任务" >&2
  multi_host_record_cleanup pending stop-unconfirmed
  return 1
}

multi_host_recover_cleanup() { # task-server 只在本机包装结束后调用；成功回执之前不释放资源。
  TASK_SERVER_PEER_CLEANUP_FILE="$1"
  tree_root="$(cd "$multi_host_script_directory/../.." && pwd)"
  local fields=()
  mapfile -d '' -t fields < <(python3 - "$1" <<'PY'
import json, sys
with open(sys.argv[1], encoding="utf-8") as handle:
    record = json.load(handle)
for key in ("status", "label", "directory", "scratch", "pid_file", "exit_file", "output_file", "boot_id"):
    value = record.get(key, "")
    if not isinstance(value, str) or "\0" in value:
        raise ValueError("invalid cleanup record")
    sys.stdout.buffer.write(value.encode() + b"\0")
sys.stdout.buffer.write(b"1\0" if record.get("processes_stopped") is True else b"0\0")
PY
  )
  (( ${#fields[@]} == 9 )) || return 1
  [[ "${fields[0]}" == confirmed || "${fields[0]}" == not-started ]] && return 0
  [[ "${fields[0]}" == pending ]] || return 1
  run_label="${fields[1]}"; peer_start_directory="${fields[2]}"; peer_scratch="${fields[3]}"
  peer_pid_file="${fields[4]}"; peer_exit_file="${fields[5]}"; peer_output_file="${fields[6]}"; peer_boot_id="${fields[7]}"
  peer_scope_file="${peer_pid_file%.pid}.scope"   # 文件头 ⑥：进程号文件旁边那一份 scope 单元名
  peer_host_load_configuration "$multi_host_script_directory/../crash/crash-point-replay-shard-configuration-check.sh" "$tree_root" >&2 || return 1
  [[ "$run_label" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ && "$peer_start_directory" == /* \
     && "$peer_scratch" =~ ^"${PEER_HOST_BUILD_ROOT%/}/builds/"[A-Za-z0-9][A-Za-z0-9._-]*$ \
     && "$peer_pid_file" == "${PEER_HOST_BUILD_ROOT%/}/runs/$run_label.pid" \
     && "$peer_exit_file" == "${PEER_HOST_BUILD_ROOT%/}/runs/$run_label.exit" \
     && "$peer_output_file" == "${PEER_HOST_BUILD_ROOT%/}/runs/$run_label.log" ]] || return 1
  peer_processes_stopped="${fields[8]}"
  peer_command_started=$(( 1 - peer_processes_stopped )); peer_run_pending=1; multi_host_cleanup_failed=0
  multi_host_stop_pending_peer || return 1
  multi_host_remove_peer_run
  (( multi_host_cleanup_failed == 0 ))
}

multi_host_remove_kept_build() { # <标签>：删第二台 builds/<标签>/（上一次带 --keep-build-directory 留下的；文件头「留着的编译目录」）：task-server 换台或有了结局时调。
  # 核没有进程开着里面的文件（有就不删、退 1）、rm、回读不在了才退 0；本来就不在的也退 0。配置载不了、第二台连不上退 255
  local label="$1" holder_text holder_status checker
  [[ "$label" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ ]] || multi_host_usage_error "--remove-build-directory 读到「$label」"
  tree_root="$(cd "$multi_host_script_directory/../.." && pwd)"
  checker="$tree_root/scripts/hosts/run_build_directory.py"
  peer_host_load_configuration "$multi_host_script_directory/../crash/crash-point-replay-shard-configuration-check.sh" "$tree_root" >&2 \
    || multi_host_fail "多机配置载不了：留着的 builds/$label 没删" "照上面判法的红句修多机配置，再跑 bash scripts/hosts/multi-host-run.sh --remove-build-directory $label"
  local scratch="${PEER_HOST_BUILD_ROOT%/}/builds/$label"
  holder_text="$(peer_host_run / "python3 $(printf '%q' "$checker") holders $(printf '%q' "$scratch")" 2>/dev/null)"
  holder_status=$?
  if (( holder_status == 255 )); then
    multi_host_fail "第二台连不上（ssh 退 255）：留着的 builds/$label 没删" "等第二台的 sshd 缓过来再跑 bash scripts/hosts/multi-host-run.sh --remove-build-directory $label"
  elif (( holder_status == 3 )); then
    echo "  ✗ 第二台留着的 builds/$label 里还有进程开着文件，没删：" >&2
    printf '%s\n' "$holder_text" | sed 's/^/      /' | multi_host_redact >&2   # gate-lint:detail
    echo "     → 怎么办：到第二台照上面的进程号逐个 python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号>，再跑 bash scripts/hosts/multi-host-run.sh --remove-build-directory $label" >&2
    return 1
  elif (( holder_status != 0 )); then
    echo "  ✗ 第二台上没核成开着 builds/$label 里文件的进程（run_build_directory.py holders 退 $holder_status），没删" >&2
    echo "     → 怎么办：在第二台上手跑 python3 scripts/hosts/run_build_directory.py holders <那个目录> 看原因，修好再跑 bash scripts/hosts/multi-host-run.sh --remove-build-directory $label" >&2
    return 1
  fi
  if peer_host_run / "rm -rf -- $(printf '%q' "${scratch:?}") && test ! -e $(printf '%q' "$scratch")" >/dev/null; then
    echo "  · 删了第二台留着的 builds/$label（连同编译目录，回读不在了）" >&2
    return 0
  fi
  echo "  ✗ 第二台留着的 builds/$label 没删掉" >&2
  echo "     → 怎么办：到第二台多机配置 BUILD_ROOT_BY_HOST 里它那一项的目录下手动删 builds/$label" >&2
  return 1
}

multi_host_stop_peer_scope() { # ⑥ 删之前先在第二台停这一趟内存包装的 scope（文件头 ⑥；peer-host-lib.sh 的 peer_host_stop_attempt_scope，照 runs/<标签>.scope 里的单元名）。
  # 停 scope 时里面还有几个记进 multi_host_scope_stopped；回读 inactive 或 failed 而一个不剩（单元名文件不在也算：这一趟没起 scope）退 0；
  # 停了还有进程的逐个报（换掉私有的字）、清理回执记 pending scope-processes-remain 并带上那几个进程、退 1；单元名认不出、ssh 没跑成的记 pending scope-unchecked、退 1（占用照留）。
  # 弄坏开关 peer-scope-unchecked 下不停（只照会话号与标签找）
  local scope_text scope_status scope_summary remaining_lines
  multi_host_scope_stopped=0
  if multi_host_break_is peer-scope-unchecked; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=peer-scope-unchecked：⑥ 不先停这一趟内存包装的 scope，只照会话号与标签找留下的进程" >&2
    return 0
  fi
  scope_text="$(peer_host_run / "$(declare -f peer_host_stop_attempt_scope)
peer_host_stop_attempt_scope $(printf '%q' "$peer_scope_file")" 2>/dev/null)"
  scope_status=$?
  scope_summary="$(grep -m 1 '^PEER_HOST_SCOPE ' <<<"$scope_text")"
  if (( scope_status == 0 )); then
    [[ "$scope_summary" =~ before=([0-9]+) ]] && multi_host_scope_stopped="${BASH_REMATCH[1]}"
    return 0
  fi
  if (( scope_status == 1 )); then
    remaining_lines="$(grep '^PEER_HOST_SCOPE_REMAINING ' <<<"$scope_text" | sed 's/^PEER_HOST_SCOPE_REMAINING //' | multi_host_redact)"
    multi_host_record_cleanup pending scope-processes-remain "$remaining_lines"
    echo "  ✗ 第二台这一趟内存包装的 scope 停了之后还留着进程（${scope_summary#PEER_HOST_SCOPE }），builds/$run_label 与 runs/ 下的文件没删、占用照留：" >&2
    printf '%s\n' "$remaining_lines" | sed 's/^/      /' >&2   # gate-lint:detail
    echo "     → 怎么办：这条命令退出后还留着子进程，核实脚本有没有不等子进程就退；到第二台照上面的进程号看它们为什么停不掉，停掉之后服务对账时的 --recover-cleanup 再核、再删" >&2
  else
    multi_host_record_cleanup pending scope-unchecked
    echo "  ✗ 在第二台停这一趟内存包装的 scope 那一步退 $scope_status（${scope_summary:-ssh 连不上，或那一步自己没跑成}）：builds/$run_label 与 runs/ 下的文件没删、占用照留" >&2
    echo "     → 怎么办：等第二台连得上、看 runs/$run_label.scope 里的单元名，服务对账时的 --recover-cleanup 再核、再删" >&2
  fi
  return 1
}

multi_host_stop_peer_leftovers() { # ⑥ 删之前在第二台停这一趟留下的进程（文件头 ⑥；peer-host-lib.sh 的 peer_host_stop_attempt_leftovers：会话号是进程号文件里那个号的、
  # 环境里带 TASK_SERVER_ATTEMPT_LABEL=<这一趟的标签> 的，后代自己 setsid 出去的照标签找得到）。停下了几个记进 multi_host_leftovers_stopped；
  # 一个不剩退 0；还有停不掉的、读不了 /proc、核的那一步跑不成的打原因与出路、记清理回执 pending、退 1（占用照留）。弄坏开关 leftovers-left-at-cleanup 下不停
  local leftover_text leftover_status summary
  multi_host_leftovers_stopped=0
  multi_host_stop_peer_scope || return 1   # 先停这一趟内存包装的 scope（文件头 ⑥）；会话号与标签那一步只当补充
  if multi_host_break_is leftovers-left-at-cleanup; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=leftovers-left-at-cleanup：⑥ 删之前不停这一趟留下的进程" >&2
    return 0
  fi
  leftover_text="$(peer_host_run / "$(declare -f peer_host_signal_one peer_host_process_is_alive peer_host_attempt_leftovers peer_host_stop_attempt_leftovers)
peer_host_stop_signal=none
session=\"\$(cat $(printf '%q' "$peer_pid_file") 2>/dev/null)\"
[[ \"\$session\" =~ ^[1-9][0-9]*\$ && \"\$session\" != 1 ]] || session=''
peer_host_stop_attempt_leftovers \"\$session\" \"\$(cat $(printf '%q' "$peer_pid_file.start") 2>/dev/null)\" $(printf '%q' "$run_label")" 2>/dev/null)"
  leftover_status=$?
  summary="$(grep -m 1 '^PEER_HOST_LEFTOVERS stopped=' <<<"$leftover_text")"
  if (( leftover_status == 0 )) && [[ "$summary" =~ stopped=([0-9]+) ]]; then
    multi_host_leftovers_stopped="${BASH_REMATCH[1]}"
    return 0
  fi
  if (( leftover_status == 1 )); then
    multi_host_record_cleanup pending leftovers-remain
    echo "  ✗ 第二台这一趟留下的进程还有停不掉的（发了 TERM、KILL 还在），builds/$run_label 与 runs/ 下三个文件没删、占用照留：" >&2
    printf '%s\n' "$leftover_text" | sed 's/^/      /' | multi_host_redact >&2   # gate-lint:detail
    echo "     → 怎么办：到第二台照上面的进程号看它们为什么停不掉（D 状态等 IO 的等 IO 缓过来），停掉之后服务对账时的 --recover-cleanup 再核、再删" >&2
  elif (( leftover_status == 2 )); then
    multi_host_record_cleanup pending leftovers-unreadable
    echo "  ✗ 第二台读不了 /proc，核不了这一趟留下的进程：builds/$run_label 与 runs/ 下三个文件没删、占用照留" >&2
    echo "     → 怎么办：看第二台 /proc 的挂载选项（hidepid 这类）；修好之后服务对账时的 --recover-cleanup 再核、再删" >&2
  else
    multi_host_record_cleanup pending leftovers-unchecked
    echo "  ✗ 在第二台核这一趟留下的进程那一步退 $leftover_status（ssh 连不上，或那一步自己没跑成）：builds/$run_label 与 runs/ 下三个文件没删、占用照留" >&2
    echo "     → 怎么办：等第二台连得上，服务对账时的 --recover-cleanup 再核、再删" >&2
  fi
  return 1
}

multi_host_remove_peer_run() { # ⑥ 删第二台这一趟的本地目录（连同编译目录）与 runs/ 下三个文件，回读不在了才算删了；删之前先停这一趟留下的进程（multi_host_stop_peer_leftovers），
  # 再核没有进程开着里面的文件。没删成置 multi_host_cleanup_failed=1
  local holder_text holder_status removal_status=1 checker="$tree_root/scripts/hosts/run_build_directory.py"
  peer_run_pending=0
  if multi_host_break_is no-peer-cleanup; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=no-peer-cleanup：第二台这一趟的本地目录 builds/$run_label 与 runs/ 下三个文件不删" >&2
    return 0
  fi
  multi_host_step remove-build begin
  if ! multi_host_stop_peer_leftovers; then
    multi_host_cleanup_failed=1
    multi_host_step remove-build end 1
    return 0
  fi
  # 开着里面文件的进程（文件头 ⑥）：仓里的 run_build_directory.py holders（同一个绝对路径），退 3 是有。弄坏开关 open-files-unchecked 不核
  if multi_host_break_is open-files-unchecked; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=open-files-unchecked：删第二台这一趟的本地目录之前不核开着的文件" >&2
  else
    holder_text="$(peer_host_run / "python3 $(printf '%q' "$checker") holders $(printf '%q' "$peer_scratch")" 2>/dev/null)"
    holder_status=$?
    if (( holder_status == 3 )); then
      multi_host_cleanup_failed=1
      multi_host_record_cleanup pending holders-remain
      echo "  ✗ 第二台 builds/$run_label 这一趟的本地目录里还有进程开着文件，没删（那条命令的子进程没收干净）：" >&2
      printf '%s\n' "$holder_text" | sed 's/^/      /' | multi_host_redact >&2   # gate-lint:detail
      echo "     → 怎么办：到第二台照上面的进程号逐个 python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号>，再手动删多机配置 BUILD_ROOT_BY_HOST 里第二台那一项的目录下的 builds/$run_label 与 runs/$run_label.pid / .exit / .log；修那条命令，让它停自己的子进程之后再退" >&2
      multi_host_step remove-build end 1
      return 0
    elif (( holder_status != 0 )); then
      multi_host_cleanup_failed=1
      multi_host_record_cleanup pending holders-unreadable
      echo "  ! 第二台上没核成开着的文件（run_build_directory.py holders 退 $holder_status），保留现场和占用" >&2
      return 0
    fi
  fi
  multi_host_record_cleanup confirmed processes-gone || { multi_host_cleanup_failed=1; return 0; }
  # 留着的编译目录（文件头「留着的编译目录」）：带 --keep-build-directory、命令等到了退出码的，只删 runs/ 下三个文件与这一层退出时峰值那一份，
  # 这一趟的本地目录（连同编译目录）留给下一次 --reuse-build-directory；回读它还在、三个文件不在了才算留下，打一行 MULTI_HOST_RUN_PEER_BUILD_KEPT。
  # 没等到退出码的（中途失败、被信号停）照删。弄坏开关 peer-build-not-kept 下一律照删
  if (( multi_host_keep_build && peer_command_exited )) && ! multi_host_break_is peer-build-not-kept; then
    if peer_host_run / "rm -f -- $(printf '%q' "$peer_scratch/$MULTI_HOST_PEER_EXIT_PEAK_NAME") $(printf '%q' "${peer_pid_file:?}") $(printf '%q' "$peer_pid_file.start") $(printf '%q' "${peer_exit_file:?}") $(printf '%q' "${peer_output_file:?}") $(printf '%q' "${peer_scope_file:?}") && test -d $(printf '%q' "$peer_scratch/target") && test ! -e $(printf '%q' "$peer_pid_file")" >/dev/null; then
      removal_status=0
      echo "  · ⑥ 留着第二台这一趟的本地目录 builds/${peer_scratch##*/}（连同编译目录，--keep-build-directory），删了 runs/ 下进程号、退出码、输出三个文件（回读过），删之前停了这一趟留下的 ${multi_host_leftovers_stopped:-0} 个进程，停这一趟内存包装的 scope 时里面还有 ${multi_host_scope_stopped:-0} 个" >&2
      echo "MULTI_HOST_RUN_PEER_BUILD_KEPT label=${peer_scratch##*/}" >&2
    else
      multi_host_cleanup_failed=1
      echo "  ✗ 第二台 runs/ 下 $run_label 的三个文件没删掉，或要留着的 builds/${peer_scratch##*/}/target 不在了" >&2
      echo "     → 怎么办：到第二台多机配置 BUILD_ROOT_BY_HOST 里它那一项的目录下手动删 builds/${peer_scratch##*/} 与 runs/$run_label.pid / .exit / .log" >&2
    fi
    multi_host_step remove-build end "$removal_status"
    return 0
  fi
  if peer_host_run / "rm -rf -- $(printf '%q' "${peer_scratch:?}") $(printf '%q' "${peer_pid_file:?}") $(printf '%q' "$peer_pid_file.start") $(printf '%q' "${peer_exit_file:?}") $(printf '%q' "${peer_output_file:?}") $(printf '%q' "${peer_scope_file:?}") && test ! -e $(printf '%q' "$peer_scratch")" >/dev/null; then
    removal_status=0
    # 这一行的头一段与「回读不在了」是 task-server 认第二台那一份删了的原文（task_server.py 的 PEER_TRIP_REMOVED_MARK、PEER_TRIP_READ_BACK_MARK），停了几个接在后面
    echo "  · ⑥ 删了第二台这一趟的本地目录 builds/$run_label（连同编译目录）与 runs/ 下进程号、退出码、输出三个文件（回读不在了），删之前停了这一趟留下的 ${multi_host_leftovers_stopped:-0} 个进程，停这一趟内存包装的 scope 时里面还有 ${multi_host_scope_stopped:-0} 个" >&2
  else
    multi_host_cleanup_failed=1
    echo "  ✗ 第二台这一趟的本地目录 builds/$run_label 没删掉" >&2
    echo "     → 怎么办：到第二台多机配置 BUILD_ROOT_BY_HOST 里它那一项的目录下手动删 builds/$run_label 与 runs/$run_label.pid / .exit / .log" >&2
  fi
  multi_host_step remove-build end "$removal_status"
}

multi_host_on_exit() { # 删第二台这一趟的本地目录（⑥）与本机这一趟的临时目录；第二台那一份没删成而本来要退 0 的改退 255（文件头 ⑥）
  local exit_code=$?
  trap '' TERM INT HUP PIPE
  if (( peer_run_pending )); then
    if [[ "${multi_host_skip_peer_stop:-0}" == 1 ]] || ! multi_host_stop_pending_peer; then
      multi_host_cleanup_failed=1
    else
      multi_host_remove_peer_run
    fi
  fi
  if [[ -n "${multi_host_scratch:-}" ]]; then rm -rf -- "${multi_host_scratch:?}"; fi
  if (( multi_host_cleanup_failed )) && (( exit_code == 0 )); then exit 255; fi
}

multi_host_stop_and_exit() { # <信号名> <退出码>：先停那条命令，再 exit（EXIT 上删第二台这一趟的本地目录）
  trap '' TERM INT HUP PIPE
  echo "  ! 收到 $1：先停那条命令，再删第二台这一趟的本地目录，做完再退" >&2
  if (( peer_command_started )); then
    if multi_host_break_is no-peer-stop-on-signal; then
      echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=no-peer-stop-on-signal：第二台那条命令不停" >&2
      multi_host_skip_peer_stop=1
    fi
  fi
  exit "$2"
}

multi_host_finish() { # <命令的退出码>：包装自己的结局提一句，按文件头「退出码」那一段退
  local command_exit="$1"
  case "$command_exit" in
    245)
      if ! multi_host_break_is gpu-memory-unexplained; then
        echo "  ! 那条命令退 245：显存不够（内存包装认出它起止之间内核报了 NV_ERR_NO_MEMORY），那一次的输出不算结果；nvidia-smi 看谁在用卡，等它跑完再重跑" >&2
      fi
      ;;
    25[0-4]) echo "  ! 那条命令退 $command_exit：内存包装 run-with-memory-cap.sh 自己的结局（文件头「退出码」），那一次的输出不算结果" >&2 ;;
  esac
  exit "$command_exit"
}

multi_host_command_target_in() { # <命令与参数…>：有一个词（连同 bash -c 的脚本）里写了 CARGO_TARGET_DIR= 交回 0（文件头 ⑤「编译目录」）
  local word pattern='(^|[[:space:];&|(`])(export[[:space:]]+)?CARGO_TARGET_DIR='
  for word in "$@"; do
    [[ "$word" =~ $pattern ]] && return 0
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

# 跑着时接回（文件头 ⑤「跑着时接回」）：在第二台上跑的那一段。参数（前面 set -- 交进来）是进程号文件、输出文件；peer_host_process_tree 随 declare -f 带过去。
#   打一行「size <输出文件此刻的字节数>」，再给进程号那个 bash 的进程树里、它自己所在的 cgroup 之外的每个 cgroup 打一行「cgroup <usage_usec> <cgroup 路径>」；
#   其中头一个（父在子前数，就是那一层内存包装的 scope）另打一行「current <进程内存> <cgroup 路径>」（文件头 ⑤「峰值」「此刻内存」）：进程内存是那个 cgroup 目录
#   （第五个参数非空时换成它，只供自证的假 cgroup）下第三个参数那个文件（memory.stat）里 anon 与 shmem 两行之和，两行都没有不打；
#   第六个参数是 1（弄坏开关 peer-memory-with-page-cache）时另打「peak <memory.peak 的数> <cgroup 路径>」、current 打 memory.current 的数，各自读不到不打；
#   进程树（连同进程号那个 bash 自己）读完之后打一行「blocked <状态是第四个参数里某个字母的进程数>」（文件头 ⑤「等 IO」；进程号那个 bash 不在了不打）
MULTI_HOST_PEER_FOLLOW_SCRIPT='pid_file="$1" output_file="$2" current_file="$3" blocked_states="${4:-D}" memory_directory="${5:-}" with_page_cache="${6:-0}" memory_reported=0 blocked=0
printf "size %s\n" "$(stat -c %s -- "$output_file" 2>/dev/null || echo 0)"
follow_pid="$(cat -- "$pid_file" 2>/dev/null)"
[[ "$follow_pid" =~ ^[0-9]+$ && -d "/proc/$follow_pid" ]] || exit 0
base_cgroup="$(sed -n "s/^0:://p" "/proc/$follow_pid/cgroup" 2>/dev/null)"
declare -A seen_cgroups=()
for tree_pid in $(peer_host_process_tree "$follow_pid"); do
  tree_state="$(sed -n "s/^.*) \([A-Z]\) .*$/\1/p" "/proc/$tree_pid/stat" 2>/dev/null)"
  if [[ -n "$tree_state" && "$blocked_states" == *"$tree_state"* ]]; then blocked=$(( blocked + 1 )); fi
  tree_cgroup="$(sed -n "s/^0:://p" "/proc/$tree_pid/cgroup" 2>/dev/null)"
  [[ -n "$tree_cgroup" && "$tree_cgroup" != "$base_cgroup" ]] || continue
  [[ -z "${seen_cgroups[$tree_cgroup]:-}" ]] || continue
  seen_cgroups[$tree_cgroup]=1
  usage="$(sed -n "s/^usage_usec //p" "/sys/fs/cgroup$tree_cgroup/cpu.stat" 2>/dev/null)"
  if [[ "$usage" =~ ^[0-9]+$ ]]; then printf "cgroup %s %s\n" "$usage" "$tree_cgroup"; fi
  (( memory_reported )) && continue
  memory_reported=1
  memory_root="${memory_directory:-/sys/fs/cgroup$tree_cgroup}"
  current=""
  if [[ "$with_page_cache" == 1 ]]; then
    peak="$(cat -- "$memory_root/memory.peak" 2>/dev/null)"
    if [[ "$peak" =~ ^[0-9]+$ ]]; then printf "peak %s %s\n" "$peak" "$tree_cgroup"; fi
    current="$(cat -- "$memory_root/memory.current" 2>/dev/null)"
  elif [[ -r "$memory_root/$current_file" ]]; then
    process_total=0 process_seen=0
    while read -r stat_name stat_value _; do
      case "$stat_name" in
        anon|shmem) [[ "$stat_value" =~ ^[0-9]+$ ]] && process_total=$(( process_total + stat_value )) && process_seen=1 ;;
      esac
    done 2>/dev/null < "$memory_root/$current_file"
    (( process_seen )) && current="$process_total"
  fi
  if [[ "$current" =~ ^[0-9]+$ ]]; then printf "current %s %s\n" "$current" "$tree_cgroup"; fi
done
printf "blocked %s\n" "$blocked"
exit 0'

multi_host_follow_peer() { # <round|final>：peer_host_wait_detached 每一轮之后（round）与等到结局之后（final）调（文件头 ⑤「跑着时接回」）。
  # round：一次 ssh 读输出文件的字节数、各 cgroup 的 usage_usec 与那一层内存包装 cgroup 进程用的内存（anon 加 shmem），多出来的那一截取回、到最后一个换行为止经 multi_host_redact 打到标准错误，
  #   再打一行 MULTI_HOST_RUN_PEER_CPU、紧跟一行 MULTI_HOST_RUN_PEER_MEMORY，再一行 MULTI_HOST_RUN_PEER_IO_WAIT；
  # final：已接回之后的全部取回打出（不以换行收尾的补一个换行），不打 CPU、内存与等 IO 那三行
  local phase="$1" state kind first rest size="" read_now=no total=0 cgroup usage_value fetch_now=0 start_byte range_text chunk_size complete partial_size redact_exit
  local current_now=0 current_read=no blocked_now=0 blocked_read=no with_page_cache=0 finished_now=0
  local chunk="$multi_host_scratch/peer-output.chunk"
  local -A this_round_usage=()
  # 弄坏开关 peer-memory-with-page-cache：每一轮读 memory.peak 与 memory.current（含页缓存；文件头 ⑤「峰值」）
  if multi_host_break_is peer-memory-with-page-cache; then with_page_cache=1; fi
  state="$(peer_host_run / "$(declare -f peer_host_process_tree)
set -- $(printf '%q' "$peer_pid_file") $(printf '%q' "$peer_output_file") $(printf '%q' "$multi_host_peer_current_file") $(printf '%q' "$multi_host_peer_blocked_states") $(printf '%q' "$multi_host_peer_memory_directory") $with_page_cache
$MULTI_HOST_PEER_FOLLOW_SCRIPT" 2>/dev/null)" || state=""
  while read -r kind first rest; do
    case "$kind" in
      size) [[ "$first" =~ ^[0-9]+$ ]] && size="$first" ;;
      cgroup)
        [[ "$first" =~ ^[0-9]+$ && -n "$rest" ]] || continue
        read_now=yes
        this_round_usage[$rest]="$first"
        if (( first > ${multi_host_peer_cgroup_usage[$rest]:-0} )); then multi_host_peer_cgroup_usage[$rest]="$first"; fi
        ;;
      peak)   # 只在弄坏开关 peer-memory-with-page-cache 下有（memory.peak）
        [[ "$first" =~ ^[0-9]+$ ]] || continue
        if [[ -z "$multi_host_peer_memory_peak" ]] || (( first > multi_host_peer_memory_peak )); then multi_host_peer_memory_peak="$first"; fi
        ;;
      current)   # 这一轮的进程内存；峰值是见过的最大（文件头 ⑤「峰值」），0 不算读到过峰值
        [[ "$first" =~ ^[0-9]+$ ]] || continue
        current_now="$first"
        current_read=yes
        if (( first > 0 && with_page_cache == 0 )) && { [[ -z "$multi_host_peer_memory_peak" ]] || (( first > multi_host_peer_memory_peak )); }; then
          multi_host_peer_memory_peak="$first"
        fi
        ;;
      blocked)
        [[ "$first" =~ ^[0-9]+$ ]] || continue
        blocked_now="$first"
        blocked_read=yes
        ;;
    esac
  done <<<"$state"
  # 新输出：round 只取 [已接回, size) 那一截（弄坏开关 peer-output-at-end-only 下 round 不取）；final 取已接回之后的全部（peer-output-resent 下从头取）
  if [[ "$phase" == final ]]; then
    fetch_now=1
  elif ! multi_host_break_is peer-output-at-end-only && [[ -n "$size" ]] && (( size > multi_host_peer_forwarded_bytes )); then
    fetch_now=1
  fi
  if (( fetch_now )); then
    start_byte="$multi_host_peer_forwarded_bytes"
    if [[ "$phase" == final ]] && multi_host_break_is peer-output-resent; then
      echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=peer-output-resent：结局时第二台的输出从头再打一遍" >&2
      start_byte=0
    fi
    range_text="tail -c +$(( start_byte + 1 )) -- $(printf '%q' "$peer_output_file")"
    [[ "$phase" == final ]] || range_text+=" | head -c $(( size - start_byte ))"
    if peer_host_run / "$range_text" > "$chunk" 2>/dev/null; then
      chunk_size="$(wc -c < "$chunk")"
      complete="$chunk_size"
      # round 只打到最后一个换行：半行留到下一轮（换字按整行换，也不让本机自己打的行插进半行里）；弄坏开关 peer-output-partial-lines 下读到多少打多少
      if [[ "$phase" == round ]] && ! multi_host_break_is peer-output-partial-lines && (( chunk_size > 0 )) && [[ -n "$(tail -c 1 -- "$chunk")" ]]; then
        partial_size="$(tail -n 1 -- "$chunk" | wc -c)"
        complete=$(( chunk_size - partial_size ))
      fi
      # 编译段峰值（文件头 ⑤「编译段峰值」）：这一截整行里有 cargo 编完那一行的，这一轮读到的不收（grep 单独判退出码，不接在管道后面）
      if (( complete > 0 && ! multi_host_peer_build_finished )) && grep -Eq -- "$MULTI_HOST_CARGO_FINISHED_PATTERN" < <(head -c "$complete" -- "$chunk"); then
        finished_now=1
      fi
      if (( complete > 0 )); then
        head -c "$complete" -- "$chunk" | multi_host_redact >&2
        redact_exit="${PIPESTATUS[1]}"   # 换字那一步自己的退出码（管道的退出码是最后一个非 0 的，head 被 SIGPIPE 也会带上）
        if (( redact_exit != 0 )); then
          # 换字失败的这一块不重打（重打还是失败，打了一半的会重复），字节数照下面往前推；退非 0 就报，不悄悄丢（文件头 ⑤「跑着时接回」）
          multi_host_break_is peer-redact-failure-unreported \
            || echo "  ! 第二台那条命令的输出第 $(( start_byte + 1 )) 到 $(( start_byte + complete )) 字节换字时退了 $redact_exit，这一块没打到这里、也不再重打（命令的退出码不受影响）：原样的输出在第二台 runs/ 下的 $run_label.log，ssh 上去看" >&2
        elif [[ "$phase" == final && -n "$(tail -c 1 -- "$chunk")" ]]; then
          echo >&2
        fi
      fi
      multi_host_peer_forwarded_bytes=$(( start_byte + complete ))
    elif [[ "$phase" == final ]]; then
      echo "  ! 第二台那条命令的输出接回了 $start_byte 字节，之后的没接回来：ssh 到第二台 runs/ 下看 $run_label.log" >&2
    fi
  fi
  if (( finished_now )); then
    multi_host_peer_build_finished=1
  elif [[ "$phase" == round ]] && (( ! multi_host_peer_build_finished && with_page_cache == 0 )) && [[ "$current_read" == yes ]] && (( current_now > 0 )) \
      && { [[ -z "$multi_host_peer_build_phase_peak" ]] || (( current_now > multi_host_peer_build_phase_peak )); }; then
    multi_host_peer_build_phase_peak="$current_now"
  fi
  [[ "$phase" == round ]] || return 0
  # 每一轮一行 CPU（文件头 ⑤ 的固定格式）：各 cgroup 见过的最大之和，一轮比一轮不减；弄坏开关 peer-cpu-not-kept 下只算这一轮读到的、peer-cpu-unreported 下不打
  multi_host_peer_polls=$(( multi_host_peer_polls + 1 ))
  if multi_host_break_is peer-cpu-not-kept; then
    for cgroup in "${!this_round_usage[@]}"; do usage_value="${this_round_usage[$cgroup]}"; total=$(( total + usage_value )); done
  else
    for cgroup in "${!multi_host_peer_cgroup_usage[@]}"; do usage_value="${multi_host_peer_cgroup_usage[$cgroup]}"; total=$(( total + usage_value )); done
  fi
  if ! multi_host_break_is peer-cpu-unreported; then
    echo "MULTI_HOST_RUN_PEER_CPU poll=$multi_host_peer_polls usage_usec=$total read=$read_now" >&2
  fi
  # 紧跟一行此刻内存（文件头 ⑤「此刻内存」）：同一轮、同一个轮数；没读到 current 写 0 与 read=no，峰值没读到写 0，不漏行；
  # 弄坏开关 peer-memory-unreported 下不打，peer-memory-unread-skipped 下没读到 current 的这一轮不打
  if ! multi_host_break_is peer-memory-unreported && ! { [[ "$current_read" == no ]] && multi_host_break_is peer-memory-unread-skipped; }; then
    echo "MULTI_HOST_RUN_PEER_MEMORY poll=$multi_host_peer_polls current_bytes=$current_now peak_bytes=${multi_host_peer_memory_peak:-0} read=$current_read" >&2
  fi
  # 再一行等 IO（文件头 ⑤「等 IO」）：同一轮、同一个轮数；进程树没读到写 blocked=0 read=no，不漏行；
  # 弄坏开关 peer-io-wait-unreported 下不打，peer-io-wait-uncounted 下读到了也写 blocked=0
  if multi_host_break_is peer-io-wait-unreported; then return 0; fi
  if multi_host_break_is peer-io-wait-uncounted; then blocked_now=0; fi
  echo "MULTI_HOST_RUN_PEER_IO_WAIT poll=$multi_host_peer_polls blocked=$blocked_now read=$blocked_read" >&2
}

multi_host_report_peer_memory_peak() { # 第二台那条命令等到退出码之后打一行那一层内存包装 cgroup 的峰值（文件头 ⑤「峰值」）：按轮采到的进程内存的最大，
  # 按轮一次都没采到才照那一层退出时读的 memory.peak，都没有就不打；
  # 弄坏开关 peer-memory-peak-unreported 下不打，peer-memory-peak-unread-printed 下没读到也打 bytes=0，peak-read-only-by-sampling 下不读退出时那一个，
  # peer-memory-with-page-cache 下按轮采到的（memory.peak）与退出时读的取大的
  local exit_peak=""
  if multi_host_break_is peer-memory-peak-unreported; then return 0; fi
  if ! multi_host_break_is peak-read-only-by-sampling && { [[ -z "$multi_host_peer_memory_peak" ]] || multi_host_break_is peer-memory-with-page-cache; }; then
    exit_peak="$(peer_host_run / "cat -- $(printf '%q' "$peer_scratch/$multi_host_peer_exit_peak_file") 2>/dev/null" 2>/dev/null)" || exit_peak=""
    if [[ "$exit_peak" =~ ^[1-9][0-9]*$ ]] && { [[ -z "$multi_host_peer_memory_peak" ]] || (( exit_peak > multi_host_peer_memory_peak )); }; then
      multi_host_peer_memory_peak="$exit_peak"
    fi
  fi
  if [[ -z "$multi_host_peer_memory_peak" ]]; then
    multi_host_break_is peer-memory-peak-unread-printed || return 0
    multi_host_peer_memory_peak=0
  fi
  echo "MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=$multi_host_peer_memory_peak" >&2
}

multi_host_report_peer_build_phase_peak() { # 第二台那条命令等到退出码之后打一行编译段峰值（文件头 ⑤「编译段峰值」）：见到了 cargo 编完那一行、
  # 那一行之前至少一轮读到了进程内存的才打；弄坏开关 peer-build-phase-peak-unreported 下不打
  if multi_host_break_is peer-build-phase-peak-unreported; then return 0; fi
  (( multi_host_peer_build_finished )) && [[ -n "$multi_host_peer_build_phase_peak" ]] || return 0
  echo "MULTI_HOST_RUN_PEER_BUILD_PHASE_PEAK bytes=$multi_host_peer_build_phase_peak" >&2
}

multi_host_report_peer_build_peak() { # 第二台那条命令等到退出码之后、⑥ 删之前量这一趟本地编译目录的占盘，打一行（文件头 ⑤「编译目录占盘」）：量不到就不打；
  # 弄坏开关 peer-build-peak-unreported 下不打，peer-build-peak-unread-printed 下量不到也打 bytes=0
  local measured
  if multi_host_break_is peer-build-peak-unreported; then return 0; fi
  measured="$(peer_host_run / "du -s -b -- $(printf '%q' "$peer_scratch/target") 2>/dev/null | cut -f1" 2>/dev/null)" || measured=""
  if [[ ! "$measured" =~ ^[0-9]+$ ]]; then
    multi_host_break_is peer-build-peak-unread-printed || return 0
    measured=0
  fi
  echo "MULTI_HOST_RUN_PEER_BUILD_PEAK bytes=$measured" >&2
}

multi_host_shared_root_text() { # 打交给第二台那条命令的两个共享根赋值（printf %q 过，末尾带一个空格；文件头 ⑤「带到第二台的环境」）；弄坏开关 shared-roots-not-passed 下打空串
  if multi_host_break_is shared-roots-not-passed; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=shared-roots-not-passed：第二台那条命令不带 SINGLEFS_WORK_ROOT、SINGLEFS_DATA_ROOT" >&2
    return 0
  fi
  printf 'SINGLEFS_WORK_ROOT=%q SINGLEFS_DATA_ROOT=%q ' "$multi_host_work_root" "$multi_host_data_root"
}

multi_host_peer_main() { # 带 --peer：步骤 ①–⑥ 见文件头
  local cap_limit_bytes command_text command_exit switch_exit outside_word tree_top build_text wrapper_text
  # ① 配置：双机开关先判（判法退 3 是「双机：关」）；关着就拒，不退回本机
  if bash "$multi_host_script_directory/../crash/crash-point-replay-shard-configuration-check.sh" "$tree_root" >/dev/null 2>&1; then switch_exit=0; else switch_exit=$?; fi
  if [[ "$switch_exit" == 3 ]]; then
    if multi_host_break_is switch-off-falls-back-to-local; then
      echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=switch-off-falls-back-to-local：双机关着，退回本机直接跑那条命令" >&2
      if ( cd "$tree_root" && exec "$@" ) < /dev/null; then command_exit=0; else command_exit=$?; fi
      multi_host_finish "$command_exit"
    fi
    multi_host_fail "带了 --peer，配置里的双机开关却没开（ENABLE_ACROSS_MACHINES 没写或不是 1）：不去第二台，也不退回本机跑" \
      "要跨机就在配置（${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}）里写一行 ENABLE_ACROSS_MACHINES=1；不跨机就经 task-server 交、由服务挑台"
  fi
  peer_host_load_configuration "$multi_host_script_directory/../crash/crash-point-replay-shard-configuration-check.sh" "$tree_root" >&2 || exit 255
  if [[ "$peer_host_is_this_machine" == 1 ]]; then
    echo "  ! 第二台是本机上的另一个目录（只供测试的开关 SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1）" >&2
  fi
  # ② 上限：顶是多机配置 SLICE_MEMORY_CAP_BY_HOST 里第二台那一项（peer-host-lib.sh 按地址取成 PEER_HOST_SLICE_CAP）
  if [[ -z "$memory_cap" ]]; then
    memory_cap="$PEER_HOST_SLICE_CAP"
  else
    cap_limit_bytes="$(multi_host_cap_bytes "$PEER_HOST_SLICE_CAP")"
    if (( $(multi_host_cap_bytes "$memory_cap") > cap_limit_bytes )); then
      if multi_host_break_is no-peer-cap-limit; then
        echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=no-peer-cap-limit：--memory-cap $memory_cap 比多机配置 SLICE_MEMORY_CAP_BY_HOST 里第二台那一项 $PEER_HOST_SLICE_CAP 大也照跑" >&2
      else
        multi_host_usage_error "--memory-cap $memory_cap 比多机配置 SLICE_MEMORY_CAP_BY_HOST 里第二台那一项 $PEER_HOST_SLICE_CAP 大（第二台上的上限不许超过它那一台 slice 的总上限）"
      fi
    fi
  fi
  # ③ 共享根：工作目录、命令里的路径、多机配置、工作根与共享数据根都要落在 NFS_SHARED_ROOTS 某一个根下（第二台照同一个绝对路径读写）
  if multi_host_break_is outside-shared-paths-accepted; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=outside-shared-paths-accepted：不判工作目录与命令里的路径在不在共享根下" >&2
  else
    multi_host_require_shared "这一刻的工作目录" "$PWD"
    outside_word="$(multi_host_outside_shared_path "$@")" \
      || multi_host_fail "判不了命令里的路径在不在共享根下（python3 没跑成）：第二台上什么都没起" "看 python3 在不在 PATH 上"
    if [[ -n "$outside_word" ]]; then
      multi_host_fail "判不了 --peer：命令里点到的 $outside_word 不在多机配置 NFS_SHARED_ROOTS 的任何一个根下（${NFS_SHARED_ROOTS:-一个根都没写}），第二台照同一个绝对路径读写，读不到本机那一份、写出的本机也读不到；第二台上什么都没起" \
        "把这个路径换到共享根下（工作根 SINGLEFS_WORK_ROOT 在共享根下），或多机配置的 NFS_SHARED_ROOTS 加上它所在的根；不跨机就经 task-server 交、由服务挑台"
    fi
  fi
  multi_host_require_shared "本机这一份多机配置" "${CRASH_POINT_REPLAY_SHARD_CONFIGURATION_FILE:-}"
  multi_host_work_root="$(python3 "$tree_root/.claude/hooks/lib_work_root.py")" && [[ "$multi_host_work_root" == /* ]] \
    || multi_host_fail "取不到工作根（.claude/hooks/lib_work_root.py 跑不起来）：第二台上什么都没起" "修好那份库，或设 SINGLEFS_WORK_ROOT 指到共享根下的工作根"
  multi_host_work_root="$(realpath -m -- "$multi_host_work_root")"
  multi_host_data_root="$(python3 "$tree_root/.claude/hooks/lib_work_root.py" data-root)" && [[ "$multi_host_data_root" == /* ]] \
    || multi_host_fail "取不到共享数据根（.claude/hooks/lib_work_root.py data-root 没打出绝对路径：环境里没设 SINGLEFS_DATA_ROOT、多机配置里也没写 SHARED_DATA_ROOT，或那份库跑不起来）：第二台上什么都没起" \
      "在多机配置里写一行 SHARED_DATA_ROOT=<共享根下的数据根绝对路径>（multi-host.env.example），或设 SINGLEFS_DATA_ROOT"
  multi_host_data_root="$(realpath -m -- "$multi_host_data_root")"
  multi_host_require_shared "工作根（SINGLEFS_WORK_ROOT）" "$multi_host_work_root"
  multi_host_require_shared "共享数据根（SINGLEFS_DATA_ROOT）" "$multi_host_data_root"
  echo "  · ③ 工作目录 $PWD、命令里的路径、多机配置、工作根与共享数据根都在共享根下（${NFS_SHARED_ROOTS}）：第二台在同一个绝对路径里跑，不拷树、不送、不取回" >&2
  # ④ 工具链；离线依赖照工作目录所在那棵树
  if multi_host_break_is no-toolchain-check; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=no-toolchain-check：不比两台工具链" >&2
  else
    peer_host_compare_toolchains >&2 || exit 255
    echo "  · ④ 两台的工具链相同（${peer_host_toolchain_text}）" >&2
  fi
  tree_top="$(git -C "$PWD" rev-parse --show-toplevel 2>/dev/null)" || tree_top="$PWD"
  peer_host_check_offline_dependencies "$tree_top" >&2 || exit 255
  # ⑤ 第二台这一趟的本地目录：多机配置 BUILD_ROOT_BY_HOST 里第二台那一项（PEER_HOST_BUILD_ROOT）不许落在共享根下（弄坏开关 peer-directory-unchecked 下不判）
  local peer_runs="${PEER_HOST_BUILD_ROOT%/}/runs" peer_builds="${PEER_HOST_BUILD_ROOT%/}/builds" label_free_status=0
  if multi_host_break_is peer-directory-unchecked; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=peer-directory-unchecked：不判 BUILD_ROOT_BY_HOST 里第二台那一项落不落在共享根下" >&2
  elif multi_host_under_shared_root "$peer_builds"; then
    multi_host_fail "多机配置 BUILD_ROOT_BY_HOST 里第二台那一项落在共享根下：这一趟的本地目录（编译目录、进程号、退出码、输出三个文件）要在第二台本地盘上，不许落在 NFS 上；第二台上什么都没起" \
      "把多机配置 BUILD_ROOT_BY_HOST 里第二台那一项改成第二台本地盘上、不在 NFS_SHARED_ROOTS 任何一个根下的目录"
  fi
  peer_scratch="$peer_builds/${multi_host_reuse_label:-$run_label}"   # 文件头「留着的编译目录」：带 --reuse-build-directory 的接着用上一次留下的那一份
  peer_pid_file="$peer_runs/$run_label.pid"
  peer_exit_file="$peer_runs/$run_label.exit"
  peer_output_file="$peer_runs/$run_label.log"
  peer_scope_file="$peer_runs/$run_label.scope"   # 文件头 ⑥：那一层内存包装把 scope 单元名写进它，⑥ 先停这一个 scope
  local scratch_free_test="test ! -e $(printf '%q' "$peer_scratch") && "
  if [[ -n "$multi_host_reuse_label" ]]; then
    # 要接着用的那一份要在（它是上一次带 --keep-build-directory 留下的）；查的那一步 ssh 自己退 255 是连不上，别的非 0 是不在：退 255，服务记 peer-run-failed、下一次照新建的起
    local reuse_status=0
    peer_host_run / "test -d $(printf '%q' "$peer_scratch/target")" >/dev/null || reuse_status=$?
    (( reuse_status == 255 )) && multi_host_fail "第二台连不上（ssh 退 255）：命令没跑" "等第二台的 sshd 缓过来再交；服务把这一次记 peer-run-failed、重排"
    (( reuse_status == 0 )) || multi_host_fail "--reuse-build-directory 点名的上一次的编译目录 builds/$multi_host_reuse_label 在第二台上不在（被删了或上一次没留下）：命令没跑" \
      "服务把这一次记 peer-run-failed、重排时照新建的起；手跑的去掉 --reuse-build-directory 再跑"
    scratch_free_test=""
  fi
  peer_host_run / "${scratch_free_test}test ! -e $(printf '%q' "$peer_pid_file") && test ! -e $(printf '%q' "$peer_pid_file.start") && test ! -e $(printf '%q' "$peer_exit_file") && test ! -e $(printf '%q' "$peer_output_file") && test ! -e $(printf '%q' "$peer_scope_file")" >/dev/null \
    || label_free_status=$?
  multi_host_judge_label_free "$label_free_status" "第二台 builds/ 下已经有 $run_label（或 runs/ 下它的 .pid / .exit / .log）"
  # 接着用的那一份里上一次那一层内存包装退出时写的峰值删掉（这一次读的要是这一次写的；文件头 ⑤「峰值」）
  [[ -z "$multi_host_reuse_label" ]] || peer_host_run / "rm -f -- $(printf '%q' "$peer_scratch/$MULTI_HOST_PEER_EXIT_PEAK_NAME")" >/dev/null \
    || multi_host_fail "第二台删不掉接着用的编译目录里上一次的退出时峰值文件：命令没跑" "看多机配置 BUILD_ROOT_BY_HOST 里第二台那一项的目录可不可写"
  peer_host_run / "mkdir -p $(printf '%q' "$peer_runs") $(printf '%q' "$peer_scratch/target")" >/dev/null \
    || multi_host_fail "第二台建不了这一趟的本地目录 builds/$run_label" "看多机配置 BUILD_ROOT_BY_HOST 里第二台那一项的目录可不可写、盘满没满"
  peer_run_pending=1
  # 命令里有 cargo（文件头 ⑤「RocksDB 预编库的取值」）：第二台起它之前先判第二台的取值（scripts/hosts/rocksdb-prebuilt.py entry），判红退 78、命令不跑
  if multi_host_command_starts_cargo "$@"; then
    if multi_host_break_is peer-rocksdb-unchecked; then
      echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=peer-rocksdb-unchecked：第二台起含 cargo 的命令之前不判 RocksDB 预编库的取值" >&2
    else
      set -- bash -c 'python3 "$1" entry --root "$2" || { echo "  ✗ 第二台起 cargo 之前 RocksDB 预编库的取值没判过（上面 rocksdb-prebuilt.py entry 的红句），命令没跑（退 78）"; echo "  → 怎么办：在第二台上照上面的出路修（第二台没装就在那边跑 python3 scripts/hosts/rocksdb-prebuilt.py install；装了而 ssh 起的命令拿不到取值，看第二台 ~/.bashrc 最前面那一段），再跑同一条命令"; exit 78; }; shift 2; exec "$@"' \
        multi-host-run-rocksdb-entry "$tree_root/scripts/hosts/rocksdb-prebuilt.py" "$tree_top" "$@"
      echo "  · ⑤ 命令里有 cargo：第二台起它之前先判第二台的 RocksDB 预编库取值（scripts/hosts/rocksdb-prebuilt.py entry），判红退 78、命令不跑" >&2
    fi
  fi
  # ⑤ 第二台那条命令：在这一刻的工作目录里起（弄坏开关 peer-runs-in-scratch 下在这一趟的本地目录里起），编译目录在这一趟的本地目录下（peer-build-in-shared-tree 下不设）
  peer_start_directory="$PWD"
  if multi_host_break_is peer-runs-in-scratch; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=peer-runs-in-scratch：第二台那条命令在这一趟的本地目录里起，不在 $PWD" >&2
    peer_start_directory="$peer_scratch"
  fi
  build_text="CARGO_TARGET_DIR=$(printf '%q' "$peer_scratch/target") "
  if multi_host_break_is peer-build-in-shared-tree; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=peer-build-in-shared-tree：第二台那条命令不设 CARGO_TARGET_DIR" >&2
    build_text=""
  fi
  # 第二台这一层内存包装只认服务起的那一层（run-with-memory-cap.sh 文件头「只认服务直接起的那一层」）：照服务给这一次的标记给它设上
  # 另给那一层两样（run-with-memory-cap.sh 文件头「task-server 给这一层的两样」；它读了就 unset，不进命令）：退出时峰值写进这一趟的本地目录（文件头 ⑤「峰值」，
  # 弄坏开关 peak-read-only-by-sampling 下不给），服务给的现场目录原样带过去（文件头 ⑤「现场目录」，peer-scene-not-passed 下不给）
  local wrapper_extras=""
  multi_host_break_is peak-read-only-by-sampling || wrapper_extras+="RUN_WITH_MEMORY_CAP_PEAK_OUTPUT=$(printf '%q' "$peer_scratch/$MULTI_HOST_PEER_EXIT_PEAK_NAME") "
  if [[ -n "$multi_host_scene_directory" ]] && ! multi_host_break_is peer-scene-not-passed; then
    wrapper_extras+="RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY=$(printf '%q' "$multi_host_scene_directory") "
  fi
  wrapper_extras+="RUN_WITH_MEMORY_CAP_UNIT_OUTPUT=$(printf '%q' "$peer_scope_file") "   # 文件头 ⑥：那一层把 scope 单元名写进进程号文件旁边，⑥ 先停它
  wrapper_text="RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER=$(printf '%q' "$multi_host_admitted_by_task_server") ${wrapper_extras}bash $(printf '%q' "$tree_root/scripts/hosts/run-with-memory-cap.sh") $(printf '%q' "$memory_cap") "
  if multi_host_break_is peer-without-memory-cap; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=peer-without-memory-cap：第二台那条命令不经 run-with-memory-cap.sh 跑" >&2
    wrapper_text=""
  fi
  # 第二台读本机这一份配置、起活这一侧清没清场、两个共享根（文件头 ⑤「带到第二台的环境」）；弄坏开关 admission-mark-forwarded 下另把服务给最外层那一次的标记带过去
  local forwarded_admission=""
  if multi_host_break_is admission-mark-forwarded; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=admission-mark-forwarded：把 MULTI_HOST_RUN_ADMITTED_BY_TASK_SERVER 带进第二台那条命令" >&2
    forwarded_admission="MULTI_HOST_RUN_ADMITTED_BY_TASK_SERVER=$(printf '%q' "$multi_host_admitted_by_task_server") "
  fi
  command_text="$(peer_host_carried_environment)${forwarded_admission}$(multi_host_shared_root_text)${build_text}MULTI_HOST_RUN_PEER_SCRATCH=$(printf '%q' "$peer_scratch") env -C $(printf '%q' "$peer_start_directory") ${wrapper_text}$(printf '%q ' "$@")"
  multi_host_step peer-command begin
  peer_boot_id="$(peer_host_run / 'cat /proc/sys/kernel/random/boot_id')" \
    || multi_host_fail "起命令前读不到第二台的开机标识：命令没起" "检查第二台连接"
  multi_host_record_cleanup pending starting || multi_host_fail "清理状态写不进尝试目录：第二台命令没起" "检查尝试目录可写、磁盘余量"
  peer_command_started=1   # 先置上：起的那一刻收到信号，也按第二台上记下的进程号去停
  peer_host_start_detached "$peer_start_directory" "$peer_pid_file" "$peer_exit_file" "$peer_output_file" "$command_text" >/dev/null \
    || multi_host_fail "第二台那条命令起不来" "看 ssh 连不连得上、第二台上 $PWD 所在的那一个共享根挂着没有（在第二台 stat -f -c %T -- <根>/. 读到 nfs 才算挂着，task-server/README.md「## NFS 共享」；python3 task-server/control check 判同一项）"
  echo "  · ⑤ 第二台开跑（脱离 ssh 会话起，在同一个绝对路径 $peer_start_directory 里，经 run-with-memory-cap.sh $memory_cap，编译目录在第二台这一趟的本地目录 builds/$run_label 下），每 $poll_seconds 秒看一次：跑着时把它的新输出接回本机标准错误、每一轮报一行 MULTI_HOST_RUN_PEER_CPU、一行 MULTI_HOST_RUN_PEER_MEMORY 与一行 MULTI_HOST_RUN_PEER_IO_WAIT" >&2
  # 跑着时接回（文件头 ⑤「跑着时接回」）：每一轮之后与等到结局之后调 multi_host_follow_peer
  multi_host_peer_forwarded_bytes=0
  multi_host_peer_polls=0
  multi_host_peer_cgroup_usage=()
  multi_host_peer_memory_peak=""
  multi_host_peer_build_phase_peak=""
  multi_host_peer_build_finished=0
  peer_host_wait_detached "$peer_pid_file" "$peer_exit_file" "$poll_seconds" multi_host_follow_peer >&2
  case "$peer_host_detached_outcome" in
    "exit "*) command_exit="${peer_host_detached_outcome#exit }"; peer_command_started=0; peer_command_exited=1 ;;
    *) multi_host_fail "第二台那条命令没等到退出码：$peer_host_detached_outcome（vanished 进程没了又没留退出码；unreachable 连续 60 次 ssh 不通；never-started 60 次都没起来）" "看上面接回的输出；ssh 到第二台 runs/ 下看 $run_label.log" ;;
  esac
  multi_host_report_peer_memory_peak
  multi_host_report_peer_build_phase_peak
  multi_host_report_peer_build_peak
  echo "  · 第二台那条命令退 $command_exit" >&2
  multi_host_step peer-command end "$command_exit"
  if multi_host_break_is peer-exit-dropped; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=peer-exit-dropped：退出码不带回，一律退 0" >&2
    command_exit=0
  fi
  # ⑥ 删第二台这一趟的本地目录
  multi_host_remove_peer_run
  multi_host_finish "$command_exit"
}

multi_host_main() {
  peer_mode=0; memory_cap=""; run_label=""; multi_host_keep_build=0; multi_host_reuse_label=""
  local separator_seen=0
  while (( $# > 0 )); do
    case "$1" in
      --peer) peer_mode=1; shift ;;
      --memory-cap) (( $# >= 2 )) || multi_host_usage_error "--memory-cap 后面没给上限"; memory_cap="$2"; shift 2 ;;
      --label) (( $# >= 2 )) || multi_host_usage_error "--label 后面没给标签"; run_label="$2"; shift 2 ;;
      --keep-build-directory) multi_host_keep_build=1; shift ;;
      --reuse-build-directory) (( $# >= 2 )) || multi_host_usage_error "--reuse-build-directory 后面没给上一次的标签"; multi_host_reuse_label="$2"; shift 2 ;;
      --) separator_seen=1; shift; break ;;
      *) multi_host_usage_error "认不出的参数「$1」（命令要写在 -- 后面）" ;;
    esac
  done
  (( separator_seen && $# > 0 )) || multi_host_usage_error "-- 后面没有命令"
  # 只管第二台（文件头「不带 --peer」）：不带是用法错；弄坏开关 peer-flag-unrequired 下当带了跑
  if (( ! peer_mode )); then
    multi_host_break_is peer-flag-unrequired || multi_host_usage_error "不带 --peer：这里只把命令挪到第二台跑，本机的任务经 task-server 交（服务在本机直接起内存包装，task-server/task_server.py 文件头 ③「起」）"
    peer_mode=1
  fi
  if [[ -n "$memory_cap" && ! "$memory_cap" =~ ^[1-9][0-9]*[KMGT]$ ]]; then
    multi_host_usage_error "--memory-cap 读到「$memory_cap」，要写正整数加 K / M / G / T（单位必写，例 8G）"
  fi
  [[ -n "$run_label" ]] || run_label="multi-host-$(date -u +%Y%m%dT%H%M%SZ)-$$"
  [[ "$run_label" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ ]] || multi_host_usage_error "--label 读到「$run_label」"
  [[ -z "$multi_host_reuse_label" || "$multi_host_reuse_label" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ ]] || multi_host_usage_error "--reuse-build-directory 读到「$multi_host_reuse_label」"
  # 只收服务直接起的那一次（文件头 ①）：任务里面的命令再带 --peer（在哪台都一样）拒、退 255，什么都没跑；弄坏开关 nested-peer-accepted 下照收
  if [[ -z "$multi_host_admitted_by_task_server" ]]; then
    if multi_host_break_is nested-peer-accepted; then
      echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=nested-peer-accepted：不是服务直接起的那一次也照收" >&2
    else
      multi_host_fail "带了 --peer，起它的却不是 task-server 的尝试单元（环境里没有服务只给最外层那一次设的 MULTI_HOST_RUN_ADMITTED_BY_TASK_SERVER）：任务里面的命令（在哪台都一样）不再往别的台起活，这一趟什么都没跑、哪台都没连" \
        "把这条命令交给 task-server（task-server/task submit -- <命令>，不带 multi-host-run.sh 与 --peer），派哪台由服务定；任务里面的命令本来就套着 multi-host-run.sh --peer 的，去掉这一层"
    fi
  fi
  # 编译目录由这里在第二台本地盘上建（文件头 ⑤「编译目录」）：--peer 命令里写了 CARGO_TARGET_DIR= 是用法错；弄坏开关 command-target-accepted 下照跑
  if multi_host_command_target_in "$@" && ! multi_host_break_is command-target-accepted; then
    echo "  ✗ multi-host-run.sh 用法错：--peer 的命令里写了 CARGO_TARGET_DIR=：编译目录由 multi-host-run.sh 在第二台本地盘上这一趟新建、跑完删，命令里指的那一个删不到（落在共享根下就是两台编进同一个目录），命令没跑" >&2
    echo "     → 怎么办：命令里去掉这个赋值；第二台那条命令拿到的 CARGO_TARGET_DIR 是这一趟的本地目录下的 target" >&2
    exit 2
  fi
  poll_seconds="${MULTI_HOST_RUN_POLL_SECONDS:-5}"
  [[ "$poll_seconds" =~ ^[1-9][0-9]*$ ]] || multi_host_usage_error "MULTI_HOST_RUN_POLL_SECONDS 读到「$poll_seconds」，要写正整数秒"
  tree_root="$(cd "$multi_host_script_directory/../.." && pwd)"
  peer_command_started=0
  peer_command_exited=0
  peer_run_pending=0
  peer_scratch=""
  peer_start_directory=""
  peer_scope_file=""   # 第二台 runs/<标签>.scope：那一层内存包装写的 scope 单元名（文件头 ⑥）
  multi_host_cleanup_failed=0
  multi_host_scratch="$(mktemp -d)"
  trap multi_host_on_exit EXIT
  trap 'multi_host_stop_and_exit TERM 143' TERM
  trap 'multi_host_stop_and_exit INT 130' INT
  trap 'multi_host_stop_and_exit HUP 129' HUP
  multi_host_peer_main "$@"
}

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
#   编译段峰值（文件头 ⑤「编译段峰值」）：接回那一趟三行之后打一行 cargo 编完那一行 ⇒ 恰好一行 MULTI_HOST_RUN_PEER_BUILD_PHASE_PEAK bytes=<正整数>、不大于峰值行的数
#   编译目录占盘（文件头 ⑤「编译目录占盘」）：Ⓠ --peer 那一趟往编译目录里写了产物 ⇒ 恰好一行 MULTI_HOST_RUN_PEER_BUILD_PEAK bytes=<正整数>；
#      命令自己删了编译目录 ⇒ 退 0、不打那一行
#   收拾（文件头 ⑥ 与 --recover-cleanup 那一段）：标签都带这一份自证自己的进程号（并行跑的几份自证各用各的标签）。
#      先按 scope 收：--peer 命令 setsid 出去一个清了环境的 sleep、记下它的进程号，再只给那一层内存包装（RUN_WITH_MEMORY_CAP_OUTER_UNIT 的单元名里带它的进程号）发 KILL
#        ⇒ 驱动退 137、sleep 不在了、⑥ 那一行写停 scope 时里面还有至少 1 个、第二台 runs/ 与 builds/ 下什么都不剩
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
  # Ⓘ 只供测试的「第二台是本机」设在自证之外（文件头「只供测试的开关」）：去掉自证的标记与调用方的弄坏开关起驱动（scene_command 里设着它）。
  #   弄坏开关 this-machine-peer-honored-anywhere 下照它起（另加 break-honored-anywhere：没有标记时弄坏开关要它才生效）
  local outside_peer_exit outside_peer_break=()
  if multi_host_break_is this-machine-peer-honored-anywhere; then outside_peer_break=(MULTI_HOST_RUN_BREAK=break-honored-anywhere,this-machine-peer-honored-anywhere); fi
  if ( cd "$scene/repo" && exec env -u MULTI_HOST_RUN_SELFTEST_MARK -u MULTI_HOST_RUN_BREAK ${outside_peer_break[@]+"${outside_peer_break[@]}"} "${scene_command[@]}" \
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
  #   同这一侧的驱动），再起 multi-host-run.sh --peer（带着自证的标记与调用方的弄坏开关：里层照收时它往配置里的另一台 ssh）；判法的弄坏开关随命令带过去
  #   （第二台那条命令的环境照 ssh 那样只给最少几样）。这一侧在小仓的根里照同一份场景配置、驱动的那一份环境也跑那两趟判法，两边逐项比。
  #   命令里点到的两份标记放进这一场景的共享根（③ 拒命令里共享根外的路径）：照自证那两份的内容另写一份
  prepare_scene peer-side
  scene="$work/peer-side"
  printf '%s\n' "$PEER_HOST_SELFTEST_MARK_CONTENT" > "$scene/peer-host-selftest-mark"
  printf '%s\n' "$MULTI_HOST_SELFTEST_MARK_CONTENT" > "$scene/selftest-mark"
  printf '%s\n' '#!/usr/bin/env bash' '# multi-host-run.sh 自证 Ⓦ 的假 ssh：第二台上被起一次就记一行，照连不上退 255' "printf '%s\n' \"\$*\" >> $(printf '%q' "$scene/ssh.log")" 'exit 255' \
    > "$scene/peer-bin/ssh"
  chmod +x "$scene/peer-bin/ssh"
  local peer_side_break="MULTI_HOST_CONFIGURATION_BREAK=$(printf '%q' "${MULTI_HOST_CONFIGURATION_BREAK:-}")"
  local peer_side_test="SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 PEER_HOST_SELFTEST_MARK=$(printf '%q' "$scene/peer-host-selftest-mark")"
  local nested_marks="MULTI_HOST_RUN_SELFTEST_MARK=$(printf '%q' "$scene/selftest-mark") MULTI_HOST_RUN_BREAK=$(printf '%q' "${MULTI_HOST_RUN_BREAK:-}")"
  run_in peer-side run --peer --memory-cap 256M --label peer-side -- bash -c "
env $peer_side_break $peer_side_test bash scripts/crash/crash-point-replay-shard-configuration-check.sh . > \"\$0.check\" 2>&1; echo \$? > \"\$0.check-exit\"
env $peer_side_break $peer_side_test bash scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-assignments . > \"\$0.emit\" 2>&1; echo \$? > \"\$0.emit-exit\"
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
  local follow_command='spin() { local spin_until=$((SECONDS + 2)); while (( SECONDS < spin_until )); do :; done; }
for line_number in 1 2 3; do
  printf "follow-line-%s\n" "$line_number"
  spin
  for (( wait_round = 0; wait_round < 80; wait_round++ )); do grep -qx "follow-line-$line_number" "$0" && break; sleep 0.1; done
  if grep -qx "follow-line-$line_number" "$0"; then echo "seen-$line_number" >> "$1"; else echo "missed-$line_number" >> "$1"; fi
done
echo "    Finished \`release\` profile [optimized] target(s) in 6.00s"
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
  # 编译段峰值（文件头 ⑤「编译段峰值」）：那一趟三行之后打了 cargo 编完那一行 ⇒ 恰好一行 MULTI_HOST_RUN_PEER_BUILD_PHASE_PEAK bytes=<正整数>、不大于峰值行的数
  local phase_problems
  phase_problems="$(awk '/^MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=/ { split($2, field, "="); peak = field[2] + 0 }
    /^MULTI_HOST_RUN_PEER_BUILD_PHASE_PEAK/ { phases++; phase_text = $0; split($2, field, "="); phase = field[2] + 0 }
    END {
      if (phases != 1) print "编译段峰值行 " (phases + 0) " 行"
      else if (phase_text !~ /^MULTI_HOST_RUN_PEER_BUILD_PHASE_PEAK bytes=[1-9][0-9]*$/) print "格式不对：" phase_text
      else if (phase > peak) print "编译段峰值 " phase " 比峰值 " peak " 大"
    }' "$scene/run.err" 2>/dev/null)"
  expect "编译段峰值 接回那一趟第二台那条命令打了 cargo 编完那一行 ⇒ 恰好一行 MULTI_HOST_RUN_PEER_BUILD_PHASE_PEAK bytes=<正整数>、不大于峰值行的数" \
    "$([[ "$(exit_of follow run)" == 0 && -z "$phase_problems" ]]; echo $?)" \
    "${phase_problems:-无}：$(grep '^MULTI_HOST_RUN_PEER_BUILD_PHASE_PEAK' "$scene/run.err" 2>/dev/null | tr '\n' '|')；stderr 尾部：$(tail_of follow run)"
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
    [[ "$left_kept" == /tmp/singlefs-run-with-memory-cap-* && -d "$left_kept" ]] && rm -rf -- "${left_kept:?}"
  done < <(grep -o '留着看现场：/tmp/singlefs-run-with-memory-cap-[^ ]*' "$scene/scene.err" 2>/dev/null | sed 's/^留着看现场：//')

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
  # 那一层被 KILL 不走它的 EXIT：它在 /tmp 下的空临时目录与状态目录里的峰值文件这一格留下（推的，没量过）
  local scope_label="scope-leftover-$$" scope_leftover_pid="" scope_leftover_alive=0
  prepare_scene scope-leftover
  scene="$work/scope-leftover"
  run_in scope-leftover wrapper-killed --peer --memory-cap 256M --label "$scope_label" -- \
    bash -c 'setsid env -i sleep 300 < /dev/null > /dev/null 2>&1 & echo "$!" > "$0.pid"; wrapper="${RUN_WITH_MEMORY_CAP_OUTER_UNIT#singlefs-memory-cap-}"; kill -KILL "${wrapper%%-*}"; exit 0' "$scene/scope-leftover"
  scope_leftover_pid="$(cat "$scene/scope-leftover.pid" 2>/dev/null)"
  if [[ "$scope_leftover_pid" =~ ^[1-9][0-9]*$ ]] && peer_host_process_is_alive "$scope_leftover_pid"; then
    scope_leftover_alive=1
    kill "$scope_leftover_pid"   # 弄坏开关下没停掉的那一个：只停这一格的命令记下的那个进程号
  fi
  expect "收拾 --peer：那一层内存包装被 KILL、scope 里留着清了环境又换了会话号的进程 ⇒ ⑥ 先停那个 scope、那个进程不在了、⑥ 那一行写停 scope 时里面还有 1 个" \
    "$([[ "$(exit_of scope-leftover wrapper-killed)" == 137 && -n "$scope_leftover_pid" && "$scope_leftover_alive" == 0 && "$(peer_runs_left scope-leftover)" == 0 ]] \
       && grep -q '⑥ 删了第二台这一趟的本地目录.*scope 时里面还有 [1-9][0-9]* 个' "$scene/wrapper-killed.err"; echo $?)" \
    "退 $(exit_of scope-leftover wrapper-killed)，留在 scope 里的那一个（进程号「$scope_leftover_pid」）$([[ "$scope_leftover_alive" == 1 ]] && echo 还活着 || echo 不在了)；留着 $(peer_runs_left scope-leftover) 条；⑥ 那一行：$(grep '⑥' "$scene/wrapper-killed.err" 2>/dev/null | tr '\n' '|' | cut_characters 300)"
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
  echo "  ✓ multi-host-run.sh 自证 $checked 格都对（不带 --peer 就拒、只供测试的「第二台是本机」设在自证之外就拒、--peer 在第二台同一个绝对路径里跑且退出码带回、上限不超过 SLICE_MEMORY_CAP_BY_HOST 里第二台那一项、按台的三个键各台取各台那一项、标签撞了是用法错而查标签连不上退 255、工具链不同与缺离线依赖就拒（缺的是树里的 path 依赖报树里缺、不报第二台缺缓存）、跑完删第二台这一趟的本地目录、收到 TERM 先停第二台、输出不露主机与目录、双机没开就拒、弄坏开关设在自证之外就拒、包装认出显存不够时原样退 245 并报显存不够、--peer 命令里写 CARGO_TARGET_DIR 就拒、--peer 的编译目录在第二台本地这一趟的目录下且跑完删、删之前有进程开着里面的文件就不删并报出进程、第二台起 cargo 之前判 RocksDB 预编库取值、自证的细节行按字截、第二台读本机这一份多机配置、同一份配置在第二台判的结局与这一侧相同、任务里面再带 --peer 两台都拒、起活这一侧清过场才告诉第二台、共享根外的路径与工作目录就拒、工作根与共享数据根照本机的真路径交给第二台而不在共享根下就拒、第二台的编译根在共享根下就拒、--peer 每一步打开始与结束两行秒数、跑着时把第二台的新输出逐行接回且不重不漏、输出里夹不是合法 UTF-8 的字节照接回而私有的字照换、换字那一步退非 0 就报这一块没打出来、每一轮报一行不减的 CPU、结束时报一行那一层内存包装的进程内存峰值（只算 anon 加 shmem，页缓存撑满上限也不算；命令比一轮短照那一层退出时读的报）而读不到就不报、退非 0 的现场由第二台那一层搬进服务给的现场目录、删之前报一行编译目录的占盘而量不到就不报、⑥ 删之前先停那一层内存包装的 scope（包装被 KILL 时 scope 里留着的清了环境、换了会话号的进程也停掉）、再停后代自己 setsid 出去的进程并写停了几个、--recover-cleanup 在起的那个 bash 被杀又没留退出回执时照会话与标签核过就确认并补写回执（后代还活着的先停）、双机驱动先编后跑的起法经 --peer 两步都在本机那棵树里起）"
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
if [[ -n "${MULTI_HOST_CONFIGURATION_BREAK:-}" ]] && ! multi_host_inside_selftest; then
  echo "  ✗ MULTI_HOST_CONFIGURATION_BREAK=$MULTI_HOST_CONFIGURATION_BREAK 设在 --selftest 之外：弄坏开关只在自证里认，这一次什么都没做（退 2，命令一行都没跑）" >&2
  echo "     → 怎么办：从环境里去掉 MULTI_HOST_CONFIGURATION_BREAK 再起同一条命令；要证红就带着它起 --selftest" >&2
  exit 2
fi
# 只供自证（文件头 ⑤「峰值」「此刻内存」「等 IO」）：退出时读的那一份与每一轮读进程内存的文件名、读进程内存的目录（假 cgroup）、数的状态，
# 只在 --selftest 起的驱动里认、只收文件名（目录只收绝对路径）的字符，读完就 unset、不带给命令
if [[ "${MULTI_HOST_RUN_SELFTEST_EXIT_PEAK_FILE:-}" =~ ^[A-Za-z0-9._-]+$ ]] && multi_host_inside_selftest; then multi_host_peer_exit_peak_file="$MULTI_HOST_RUN_SELFTEST_EXIT_PEAK_FILE"; fi
if [[ "${MULTI_HOST_RUN_SELFTEST_CURRENT_FILE:-}" =~ ^[A-Za-z0-9._-]+$ ]] && multi_host_inside_selftest; then multi_host_peer_current_file="$MULTI_HOST_RUN_SELFTEST_CURRENT_FILE"; fi
if [[ "${MULTI_HOST_RUN_SELFTEST_MEMORY_DIRECTORY:-}" =~ ^/[A-Za-z0-9._/-]+$ ]] && multi_host_inside_selftest; then multi_host_peer_memory_directory="$MULTI_HOST_RUN_SELFTEST_MEMORY_DIRECTORY"; fi
if [[ "${MULTI_HOST_RUN_SELFTEST_BLOCKED_STATES:-}" =~ ^[A-Z]+$ ]] && multi_host_inside_selftest; then multi_host_peer_blocked_states="$MULTI_HOST_RUN_SELFTEST_BLOCKED_STATES"; fi
unset MULTI_HOST_RUN_SELFTEST_EXIT_PEAK_FILE MULTI_HOST_RUN_SELFTEST_CURRENT_FILE MULTI_HOST_RUN_SELFTEST_MEMORY_DIRECTORY MULTI_HOST_RUN_SELFTEST_BLOCKED_STATES
# 只供测试的 SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE（文件头「只供测试的开关」）：只在 --selftest 起的驱动里认，别处设了就拒；弄坏开关 this-machine-peer-honored-anywhere 下照认
if [[ -n "${SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE:-}" ]] && ! multi_host_inside_selftest && ! multi_host_break_is this-machine-peer-honored-anywhere; then
  echo "  ✗ SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=$SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE 设在 --selftest 之外：这个开关只供测试（「第二台」换成本机上的目录），只在 --selftest 里认；这一次什么都没做（退 2，命令一行都没跑）" >&2
  echo "     → 怎么办：去掉它再跑（env -u SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE bash scripts/hosts/multi-host-run.sh …）" >&2
  exit 2
fi
unset MULTI_HOST_RUN_SELFTEST_MARK
if [[ "${1:-}" == --recover-cleanup && $# == 2 ]]; then
  multi_host_recover_cleanup "$2"
  exit $?
fi
if [[ "${1:-}" == --remove-build-directory ]]; then
  (( $# == 2 )) || multi_host_usage_error "--remove-build-directory 后面要恰好一个标签"
  multi_host_remove_kept_build "$2"
  exit $?
fi
multi_host_main "$@"
