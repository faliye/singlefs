#!/usr/bin/env bash
# 把一件确定性的活挪到第二台跑：只在第二台跑，--peer 必写。仓、工作根与会话记录目录经 NFS 两台挂在同一绝对路径
#   （多机配置 NFS_SHARED_ROOTS）：
#   第二台在这一刻的工作目录、同一个绝对路径里直接起，不拷树、不送、不取回、不改写路径，写出的文件两台都读得到。
#
#   multi-host-run.sh --peer [--memory-cap <上限>] [--label <标签>] [--keep-build-directory] [--reuse-build-directory <上一次的标签>] -- <命令与参数…>
#   multi-host-run.sh --remove-build-directory <标签>
#   multi-host-run.sh --selftest
# 留着的编译目录（task-server 给每棵树在第二台一组编译目录，同一棵树上后交的任务接着编，与本机同一段：task-server/task_server_lib.py 文件头「编译目录」）：
#   --keep-build-directory：第二台那条命令等到了退出码的，⑥ 只删 runs/ 下三个文件与这一层退出时峰值那一份，这一趟的本地目录 builds/<标签>/（连同编译目录）留着，
#     回读它还在之后往本机标准错误打一行「MULTI_HOST_RUN_PEER_BUILD_KEPT label=<留着的那一份的标签>」（task-server 照它记这一次的编译目录没删）；
#     没等到退出码的（中途失败、被信号停）照 ⑥ 删。删之前照旧核没有进程开着里面的文件。
#   --reuse-build-directory <上一次的标签>：这一趟不新建，接着用第二台 builds/<上一次的标签>/（CARGO_TARGET_DIR 是它下面的 target、MULTI_HOST_RUN_PEER_SCRATCH 是它）；
#     起之前在第二台核它的 target 在（不在退 255、命令没跑），删掉里面上一次那一层退出时写的峰值；runs/ 下三个文件照这一趟的 --label 起名。
#     不带 --keep-build-directory 的跑完照 ⑥ 连它一起删。
#   --remove-build-directory <标签>：只删第二台 builds/<标签>/（task-server 删组里那一个时调：那一棵树不在了）：核没有进程开着里面的文件、rm、回读不在了退 0（本来就不在也退 0），
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
#      （带 --reuse-build-directory 的接着用上一次留下的那一个，文件头「留着的编译目录」；下面叫这一趟的本地目录），编译目录是它下面的 target；进程号、退出码、输出三个文件写第二台 <编译根>/runs/<标签>.pid / .exit / .log。编译根照字面落在共享根下就拒、退 255
#      （编译目录与这几份文件不许落在 NFS 上：两台各编各的；不带 --keep-build-directory 的这一趟用完就删）。这一趟的本地目录或那三个文件已在是标签撞了，用法错退 2；
#      查的那一步 ssh 自己退 255 是第二台连不上，退 255、不当用法错。
#      第二台那条命令：env -C <这一刻的工作目录> 起（同一个绝对路径），环境里 CARGO_TARGET_DIR=<这一趟的本地目录>/target、
#      MULTI_HOST_RUN_PEER_SCRATCH=<这一趟的本地目录>（命令要在第二台本地盘上放临时东西的照它放，⑥ 连同编译目录删），外面套仓里的 run-with-memory-cap.sh <上限>
#      这一条命令自己的临时目录：与本机一样由那一层建在它默认的临时根下（MULTI_HOST_PEER_DEFAULT_TEMPORARY_ROOT，/tmp，同 run-with-memory-cap.sh 的 TEMPORARY_ROOT 默认值），
#      命令拿到的 TMPDIR 全路径不超过 MULTI_HOST_PEER_TEMPORARY_PATH_LIMIT（60）字节：测试在 TMPDIR 下绑 Unix 套接字，套接字路径上限 107 字节，建进这一趟的本地目录就超。
#      给那一层设 RUN_WITH_MEMORY_CAP_TEMPORARY_RECORD=<这一趟的本地目录>/temporary-directory（MULTI_HOST_PEER_TEMPORARY_RECORD_NAME；run-with-memory-cap.sh 文件头
#      「multi-host-run.sh 给这一层的两样」）：那一层把建的那一份的全路径写进去、退出时删掉这份文件；那一层被 KILL 时 EXIT 钩子不走、文件留着，⑥ 照它删留下的那一份。
#      弄坏开关 peer-temporary-root-in-scratch 下另给那一层设 RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT=<这一趟的本地目录>/temporary（TMPDIR 超长）；
#      peer-temporary-record-not-passed 下不设 RUN_WITH_MEMORY_CAP_TEMPORARY_RECORD（被 KILL 留下的那一份留在第二台 /tmp 下）
#      （同一个绝对路径；那一层只有这一条的上限与 slice 的总上限，不排队、不记账：排队与记账只在本机的 task-server；那一层只认服务起的那一层，
#      这里照 ① 读到的服务给这一次的标记给它设 RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER，它照旧从命令的环境里摘掉）。弄坏开关 peer-build-in-shared-tree 下不设
#      CARGO_TARGET_DIR（cargo 照默认在共享的树里编）；peer-runs-in-scratch 下不在这一刻的工作目录里起，在这一趟的本地目录里起。
#      编译目录（由这里给：不带 --keep-build-directory 的这一趟用完就删，带的留给 task-server 按树接着用）：--peer 时命令的词里（连同 bash -c 的脚本）写了 CARGO_TARGET_DIR= 是用法错（退 2，命令没跑；
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
#      不因此杀进程），留见过的最大。等到退出码之后再经 ssh 读一次第二台那一层内存包装退出时交的那一个
#      （这里给那一层设 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT=<这一趟的本地目录>/exit-memory-peak，run-with-memory-cap.sh 文件头「task-server 给这一层的两样」：扣了页缓存、每 0.1 秒采一次），
#      与按轮读的取大的（一轮 5 秒，十来秒的命令按轮只读得到外壳：任务 10045 这一族记 18 MiB，本机量到 556 MiB）；在「第二台那条命令退 N」那一句之前往本机标准错误打一行，格式固定：
#        MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=<十进制字节数>
#      退出时交的那一个读不到（不经内存包装跑、读不了）的不打这一行（没量到，不进定额度；按轮读的只有外壳那一截，不当峰值）。没等到退出码的（第二台连不上、进程没了）不打。
#      读这一行的是 task-server（按它记第二台那一次的内存峰值）。弄坏开关 peer-memory-peak-unreported 下不打；peer-memory-peak-unread-printed 下没读到也打 bytes=0；
#      peak-read-only-by-sampling 下只靠按轮采（不给那一层设这个变量、不读退出时那一个）；peer-peak-only-when-unsampled 下按轮读到了就不看退出时那一个；peer-memory-with-page-cache 下每一轮读 memory.peak 与 memory.current
#      （含页缓存）、与退出时那一个取大的。
#      只供自证：MULTI_HOST_RUN_SELFTEST_EXIT_PEAK_FILE=<文件名> 把退出时读的那一份的文件名换成它（造「读不到」那几格），
#      MULTI_HOST_RUN_SELFTEST_MEMORY_DIRECTORY=<绝对路径> 把每一轮读 memory.stat（弄坏开关下 memory.peak、memory.current）的目录换成它（假 cgroup：造「页缓存撑满上限」那一格，
#      只收 /、字母、数字与 ._- 组成的绝对路径），只在 --selftest 起的驱动里认，别处设了不认、读完就 unset。
#      编译目录占盘：峰值那一行之后、⑥ 删之前，在第二台上量这一趟给命令的 CARGO_TARGET_DIR（builds/<标签>/target）的占盘（du -s -b，字节），往本机标准错误打一行，格式固定：
#        MULTI_HOST_RUN_PEER_BUILD_PEAK bytes=<十进制字节数>
#      量不到（目录不在了、du 没跑成、ssh 不通）就不打；读这一行的是 task-server。弄坏开关 peer-build-peak-unreported 下不打；peer-build-peak-unread-printed 下量不到也打 bytes=0。
#      此刻内存：就是上面「峰值」每一轮读的那一个（那一层内存包装的 scope 这一轮的 anon 加 shmem），每一轮紧跟在 MULTI_HOST_RUN_PEER_CPU 那一行之后往本机标准错误打一行，格式固定：
#        MULTI_HOST_RUN_PEER_MEMORY poll=<与紧挨着的 CPU 那一行同一个轮数> current_bytes=<这一轮读到的 anon 加 shmem，没读到写 0> peak_bytes=<见过的最大，没读到写 0> read=<yes|no>
#      read=yes 是这一轮读到了 memory.stat 的 anon 或 shmem，no 是没读到（还没起、已经收尾、ssh 不通、不经内存包装跑）：没读到也照打、两项写 0（peak_bytes 在前面几轮读到过的照写读到过的最大），不漏行。
#      只在跑着的每一轮打，等到结局之后的那一段不打。读这一行的是 task-server（第二台顶在内存上限上、日志不涨而 CPU 在涨的，按它判撞上限；read=no 时 current_bytes 不算数）。
#      两边的格式一起改：task-server/task_server.py 的 PEER_MEMORY_LINE 认 ^MULTI_HOST_RUN_PEER_MEMORY poll=([0-9]+) current_bytes=([0-9]+) peak_bytes=([0-9]+) read=(yes|no)$。
#      弄坏开关 peer-memory-unreported 下不打；peer-memory-unread-skipped 下没读到 current 的这一轮不打这一行（漏行）。
#      只供自证：MULTI_HOST_RUN_SELFTEST_CURRENT_FILE=<文件名> 把每一轮读 memory.stat 的文件名换成它（造「读不到」那一格），认的地方与读完 unset 同 MULTI_HOST_RUN_SELFTEST_EXIT_PEAK_FILE。
#      内存最高那一刻的明细（task-server 写撞上限那一次的结局说明：进程被杀之后读 memory.stat 内存已释放，任务 10279 峰值 45.46 GiB、退出时读到 anon 311296）：同一次 ssh 里另读那一层内存包装的 memory.current
#      与 memory.stat 的五样，每一轮紧跟在 MULTI_HOST_RUN_PEER_MEMORY 那一行之后往本机标准错误打一行，格式固定：
#        MULTI_HOST_RUN_PEER_MEMORY_STAT poll=<轮数> current_bytes=<memory.current> anon=<字节> file=<字节> file_dirty=<字节> file_writeback=<字节> shmem=<字节> read=<yes|no>
#      没读到也照打、都写 0、read=no，不漏行；只在跑着的每一轮打。读这一行的是 task-server（留 current_bytes 最大的那一行）；两边的格式一起改：task-server/task_server.py 的 PEER_MEMORY_STAT_LINE。
#      弄坏开关 peer-memory-stat-unreported 下不打。
#      显存（任务记录里第二台的 gpu_memory_peak）与卡的利用率：这一趟要卡（本机环境里有 GPU_CARDS_BY_PRIORITY）的，同一次 ssh 里另起 timeout 10 nvidia-smi 读计算进程，把这一趟进程树里的进程用的显存相加，
#      再起一次 timeout 10 nvidia-smi -i <这一趟分到的序号，逗号隔开> --query-gpu=utilization.gpu 读分到的那几张的利用率，逐张平均（四舍五入到整数），
#      每一轮再打一行，格式固定：
#        MULTI_HOST_RUN_PEER_GPU poll=<轮数> used_mib=<MiB> read=<yes|no> utilization=<百分数|none> on_cards=<见下>
#      不要卡的、nvidia-smi 起不来或退非 0 的写 used_mib=0 read=no，不漏行；利用率读不到（不要卡、切不出序号、nvidia-smi 起不来或退非 0、有一行不是整数）写 utilization=none。只在跑着的每一轮打。
#      实际在哪几张卡（on_cards=；task-server 判 wrong-card）：要卡、不要卡的那一趟都读。同一次 ssh 里读的计算进程带卡的 uuid（--query-compute-apps=gpu_uuid,pid,used_memory），
#      只留这一趟进程树里的进程（与 used_mib 同一份进程树：进程号文件里那个 bash 往下，别的会话的进程与清场白名单不在里面）；有进程才再起一次 --query-gpu=index,uuid 把 uuid 对回序号。
#      写法：<序号>:<进程号>:<显存 MiB，读不到写 na>，几个用逗号接；empty 是读到了、这一趟一个都不在卡上；unread 是这一轮没读到（nvidia-smi 起不来或退非 0、进程树没读到、uuid 一个都对不回序号）。
#      读这一行的是 task-server（显存留最大，利用率逐行累计成这一次尝试的平均，on_cards= 判 wrong-card）；两边的格式一起改：task-server/task_server.py 的 PEER_GPU_LINE。
#      弄坏开关 peer-gpu-unreported 下不打，peer-gpu-family-unfiltered 下卡上每个进程都算，peer-gpu-utilization-unread 下利用率一直写 none，
#      peer-card-locations-unreported 下 on_cards= 一直写 unread，peer-card-locations-plain-skipped 下不要卡的那一趟不读，peer-card-locations-unfiltered 下 on_cards= 列卡上每个进程。
#      等 IO（task-server 判卡住之前看这一次的进程是不是卡在不可中断的 IO 等待上：本机根盘被写满时本机导出的 NFS 跟着慢，第二台上读写共享根的进程停在 D 状态，
#      日志不涨、CPU 也不涨，是在等 IO 不是死了）：同一次 ssh 里另数那条命令进程树里（连同进程号文件里那个 bash 自己）/proc/<进程号>/stat 状态字段是 D 的进程，
#      每一轮紧跟在 MULTI_HOST_RUN_PEER_MEMORY 那一行之后往本机标准错误打一行，格式固定：
#        MULTI_HOST_RUN_PEER_IO_WAIT poll=<与这一轮 CPU 那一行同一个轮数> blocked=<这一轮 D 状态的进程数，没读到写 0> read=<yes|no>
#      read=yes 是这一轮读到了进程树（进程号那个 bash 还在），no 是没读到（还没起、已经收尾、ssh 不通）：没读到也照打、blocked 写 0，不漏行；只在跑着的每一轮打。
#      读这一行的是 task-server（日志与 CPU 都满窗口不涨、这一个窗口里每一轮 blocked 都大于 0 且 read=yes 的不判卡住；read=no 时 blocked 不算数）。
#      两边的格式一起改：task-server/task_server.py 的 PEER_IO_WAIT_LINE 认 ^MULTI_HOST_RUN_PEER_IO_WAIT poll=([0-9]+) blocked=([0-9]+) read=(yes|no)$。
#      弄坏开关 peer-io-wait-unreported 下不打；peer-io-wait-uncounted 下读到了也写 blocked=0。
#      线程（task-server 的 CPU 闸照可跑线程数记：机器挤满时在用核数被压低；线程总数远超登记数的另推一条提醒；task-server/task_server.py 文件头 ②「CPU 闸」）：
#      同一次 ssh 里另数那条命令进程树里（连同进程号文件里那个 bash 自己）每个进程 /proc/<进程号>/task/<线程>/stat 的状态字段，状态 R 的算可跑的，
#      连同线程总数与占线程最多的那个进程（它的线程数与 /proc/<进程号>/comm，名字里字母、数字与 ._- 之外的字换成 _，读不到写 none），
#      每一轮在 MULTI_HOST_RUN_PEER_GPU 那一行之后往本机标准错误打一行，格式固定：
#        MULTI_HOST_RUN_PEER_THREADS poll=<与这一轮 CPU 那一行同一个轮数> runnable=<状态 R 的线程数> total=<线程总数> top=<线程数>:<进程名> read=<yes|no>
#      read=no 是这一轮没读到进程树（还没起、已经收尾、ssh 不通）：照打、写 runnable=0 total=0 top=0:none，不漏行；只在跑着的每一轮打。
#      两边的格式一起改：task-server/task_server.py 的 PEER_THREADS_LINE。弄坏开关 peer-threads-unreported 下不打；peer-threads-uncounted 下读到了也写 runnable=0。
#      被外部信号杀掉（task-server 的 killed-externally 对第二台的尝试也认；本机那一台的判法见 task-server/task_server.py 的 external_kill_note）：跑着时每一轮同一次 ssh 里另读这一趟
#      进程树里的全部进程号（进程号文件里那个 bash 往下，与 used_mib、on_cards= 同一份进程树）记在本机；第二台那条命令退非 0 之后（退出码 128 加信号号的，和构建工具报
#      「signal: 15, SIGTERM」而自己退 1 的，都在里面）另起一次 ssh，读第二台这一段时间（从命令起跑往前 60 秒）的 earlyoom 日志（journalctl -t earlyoom）与内核日志（journalctl -k）里
#      点名杀了这些进程号之一的头一行（「sending SIGTERM / SIGKILL to process <号>」「Killed process <号> (」，同 task_server_lib.py 的 EXTERNAL_KILL_LINE），往本机标准错误打一行，格式固定：
#        MULTI_HOST_RUN_PEER_EXTERNAL_KILL source=<earlyoom|kernel> pid=<被杀的进程号> line=<日志原行，换过私有的字、压成一行、至多 300 字节>
#      日志读不到、没有点名这一趟进程号的、命令退 0 的一律不打，不猜。内核那一行是「Memory cgroup out of memory」（某个内存组到了它自己的上限，不是整机内存紧张；
#      同 task-server/task_server_lib.py 的 MEMORY_CGROUP_KILL_TEXT）的不算被外部信号杀掉：跳过它接着找下一行，没有别的就不打。读这一行的是 task-server（第二台的尝试读到它，同本机那一路记 killed-externally 并同一上限重排）；
#      两边的格式一起改：task-server/task_server.py 的 PEER_EXTERNAL_KILL_LINE。这一行在命令退出之后打，不算日志在涨。
#      弄坏开关 peer-external-kill-unreported 下不打，peer-external-kill-unfiltered 下日志里任何一个进程被杀都算（不看是不是这一趟进程树里的），
#      peer-memory-cgroup-kill-reported 下「Memory cgroup out of memory」那一行也照打。
#      只供自证：MULTI_HOST_RUN_SELFTEST_BLOCKED_STATES=<大写字母…> 把数的状态从 D 换成它们（造「有进程在等」那一格：S 是在睡，空转的命令底下总有在睡的父进程），
#      只收大写字母，认的地方与读完 unset 同 MULTI_HOST_RUN_SELFTEST_EXIT_PEAK_FILE。
#      按 usage_usec 涨不涨判「在算」；这一行、MULTI_HOST_RUN_PEER_MEMORY 行、MULTI_HOST_RUN_PEER_MEMORY_STAT 行、MULTI_HOST_RUN_PEER_GPU 行、MULTI_HOST_RUN_PEER_THREADS 行、MULTI_HOST_RUN_PEER_IO_WAIT 行与 MULTI_HOST_RUN_STEP 行每一轮都打，判日志涨不涨时不算它们。
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
#      那一层留下的临时目录（multi_host_remove_peer_temporary）：再读这一趟的本地目录里的 temporary-directory（⑤「这一条命令自己的临时目录」）。文件不在是那一层走到了 EXIT，
#      没有要删的；文件在是那一层没走到 EXIT（被 KILL / HUP），照里面的路径删那一份、回读不在了，往本机标准错误打一行「⑥ 删了第二台那一层没收拾的临时目录 …」。
#      路径不是 MULTI_HOST_PEER_DEFAULT_TEMPORARY_ROOT 下直接一层、名字是 MULTI_HOST_PEER_TEMPORARY_PREFIX<进程号>-<后缀> 的那种，不删，打一行「⑥ 没删第二台那一层记下的临时目录 …」；
#      读不了、删不成照删不成办（退 255）。带 --keep-build-directory 留着这一趟的本地目录的同样照它删，连同删这份文件。弄坏开关 peer-temporary-record-unchecked 下不核路径、照删。
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
# gate-similar: crash-point-replay.sh 门禁照 task-server 分片协议把崩溃枚举用例的块分到各片判、合并时核账，它自己不连第二台；片派到第二台时服务经这一份起它
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
#       连带红的格：「✗ 显存 不要卡的一趟每一轮照打 …」「✗ 内存明细 每一轮一行 …」（行数与 CPU 行对不上：「… 行，CPU 行 0 行」）、「✗ 接回 每一轮一行 MULTI_HOST_RUN_PEER_MEMORY …」「✗ 接回 读不到 memory.stat 的每一轮照打 …」（MEMORY 行没有 CPU 行在前：「第 … 行的 MEMORY 前一行不是 CPU」）
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
#   MULTI_HOST_RUN_BREAK=peer-threads-unreported  ⑤ 每一轮不打 MULTI_HOST_RUN_PEER_THREADS 那一行（文件头 ⑤「线程」）
#       「✗ 线程 每一轮一行 MULTI_HOST_RUN_PEER_THREADS poll=<与这一轮 CPU 那一行同一个轮数>…」（「只有 0 行 THREADS」；推的，没复跑）
#   MULTI_HOST_RUN_BREAK=peer-threads-uncounted  ⑤ 读到了进程树也写 runnable=0（不数线程的状态；文件头 ⑤「线程」）
#       「✗ 线程 每一轮一行 MULTI_HOST_RUN_PEER_THREADS poll=<与这一轮 CPU 那一行同一个轮数>…」（「一轮都没有 read=yes 且 runnable、total 都大于 0」；推的，没复跑）
#   MULTI_HOST_RUN_BREAK=peer-memory-peak-unreported  ⑤ 第二台那条命令结束时不打 MULTI_HOST_RUN_PEER_MEMORY_PEAK 那一行
#       峰值 第二台那条命令结束时打恰好一行 MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=<正整数>（那一层内存包装 cgroup 进程内存见过的最大），在最后一行 MULTI_HOST_RUN_PEER_CPU 之后（「峰值行 0 行」）
#       连带红的格：「✗ 峰值 第二台那条命令跑 0.1 秒就退、每一轮都读不到 memory.stat ⇒ …」（「峰值行「」」）、「✗ 峰值 页缓存撑满上限 …」（「峰值行 0 行：」；推的，没复跑）
#   MULTI_HOST_RUN_BREAK=peer-build-peak-unreported  ⑤ 第二台那条命令结束时不打 MULTI_HOST_RUN_PEER_BUILD_PEAK 那一行
#       编译目录占盘 --peer：第二台那条命令结束时打恰好一行 MULTI_HOST_RUN_PEER_BUILD_PEAK bytes=<正整数>（这一趟本地编译目录删之前 du -s -b 量的）（「占盘行 0 行」）
#   MULTI_HOST_RUN_BREAK=peer-build-peak-unread-printed  ⑤ 量不到编译目录的占盘也打一行 bytes=0
#       编译目录占盘 --peer：命令自己删了编译目录（量不到）⇒ 退 0、不打 MULTI_HOST_RUN_PEER_BUILD_PEAK 那一行（「占盘行 1 行：MULTI_HOST_RUN_PEER_BUILD_PEAK bytes=0」）
#   MULTI_HOST_RUN_BREAK=peer-memory-peak-unread-printed  ⑤ 退出时没读到峰值也打一行 bytes=0
#       峰值 每一轮读不到 memory.stat、退出时读不到 memory.peak（两处读的文件名都换成不在的）⇒ 退 0、有一轮 read=yes、不打 MULTI_HOST_RUN_PEER_MEMORY_PEAK 那一行（「峰值行 1 行：MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=0」）
#       峰值 按轮读到了、退出时那一个读不到 ⇒ 不打 MULTI_HOST_RUN_PEER_MEMORY_PEAK 那一行（没量到）（「峰值行 1 行」）
#   MULTI_HOST_RUN_BREAK=peak-read-only-by-sampling  ⑤ 只靠按轮采：不给第二台那一层内存包装设 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT、等到退出码之后不读退出时那一个
#       峰值 第二台那条命令跑 0.1 秒就退、每一轮都读不到 memory.stat ⇒ 退 0、照那一层内存包装退出时读的打恰好一行 MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=<正整数>（「峰值行「」」）
#       连带红的格：「✗ 峰值 短命令按轮只读到外壳的 18 MiB …」（「峰值行 …bytes=18874368」）、「✗ 峰值 按轮读到了、退出时那一个读不到 …」（「峰值行 1 行」）
#   MULTI_HOST_RUN_BREAK=peer-peak-only-when-unsampled  ⑤ 按轮读到了就不看第二台那一层退出时交的那一个（短命令只记外壳的用量）
#       峰值 短命令按轮只读到外壳的 18 MiB、真用量 128 MiB ⇒ 峰值行不是外壳那个数（不小于 100 MiB）（「峰值行 …bytes=18874368」）
#       峰值 按轮读到了、退出时那一个读不到 ⇒ 不打 MULTI_HOST_RUN_PEER_MEMORY_PEAK 那一行（没量到）（「峰值行 1 行」）
#   MULTI_HOST_RUN_BREAK=peer-memory-stat-unreported  ⑤ 每一轮不打 MULTI_HOST_RUN_PEER_MEMORY_STAT 那一行（文件头 ⑤「内存最高那一刻的明细」）
#       内存明细 每一轮一行 MULTI_HOST_RUN_PEER_MEMORY_STAT，假 cgroup 那一趟读数对得上，真的 scope 那一趟至少一轮 read=yes（「MEMORY_STAT 行 0 行」）
#   MULTI_HOST_RUN_BREAK=peer-gpu-unreported  ⑤ 每一轮不打 MULTI_HOST_RUN_PEER_GPU 那一行（文件头 ⑤「显存」）
#       显存 要卡的一趟每一轮一行 MULTI_HOST_RUN_PEER_GPU，只算这一趟进程树里的进程（「至少一轮 used_mib=2048 read=yes：」）
#       显存 不要卡的一趟每一轮照打 MULTI_HOST_RUN_PEER_GPU …used_mib=0 read=no，每一轮 CPU 之后都有、不漏行（「GPU 行 0 行」）
#       利用率 要卡的一趟每一轮 MULTI_HOST_RUN_PEER_GPU 带分到的那两张逐张平均的 utilization=（「至少一轮 utilization=30：」）
#       实际在哪张卡 要卡的一趟 on_cards= 只列这一趟进程树里的进程，序号照 uuid 对回来（「至少一轮 on_cards=2：」）
#       实际在哪张卡 不要卡的一趟照读 on_cards=，used_mib=0 read=no 不变（「至少一轮 on_cards=2：」）
#   MULTI_HOST_RUN_BREAK=peer-gpu-family-unfiltered  ⑤ 卡上每个进程的显存都算进这一趟（别的任务的也算）
#       显存 要卡的一趟每一轮一行 MULTI_HOST_RUN_PEER_GPU，只算这一趟进程树里的进程（「不是 used_mib=2048：」）
#   MULTI_HOST_RUN_BREAK=peer-gpu-utilization-unread  ⑤ 卡的利用率不读，MULTI_HOST_RUN_PEER_GPU 那一行一直写 utilization=none（文件头 ⑤「显存」）
#       利用率 要卡的一趟每一轮 MULTI_HOST_RUN_PEER_GPU 带分到的那两张逐张平均的 utilization=（「至少一轮 utilization=30：」）
#   MULTI_HOST_RUN_BREAK=peer-card-locations-unreported  ⑤ on_cards= 一直写 unread，不读进程在哪张卡（文件头 ⑤「实际在哪几张卡」）
#       实际在哪张卡 要卡的一趟 on_cards= 只列这一趟进程树里的进程，序号照 uuid 对回来（「至少一轮 on_cards=2：」）
#       实际在哪张卡 不要卡的一趟照读 on_cards=，used_mib=0 read=no 不变（「至少一轮 on_cards=2：」）
#   MULTI_HOST_RUN_BREAK=peer-card-locations-plain-skipped  ⑤ 不要卡的那一趟不读进程在哪张卡（文件头 ⑤「实际在哪几张卡」）
#       实际在哪张卡 不要卡的一趟照读 on_cards=，used_mib=0 read=no 不变（「至少一轮 on_cards=2：」）
#   MULTI_HOST_RUN_BREAK=peer-external-kill-unreported  ⑤ 命令退非 0 之后不读第二台的 earlyoom 与内核日志、不打 MULTI_HOST_RUN_PEER_EXTERNAL_KILL（文件头 ⑤「被外部信号杀掉」）
#       被外部信号杀掉 日志点名了这一趟进程号、命令退 143 ⇒ 打 MULTI_HOST_RUN_PEER_EXTERNAL_KILL source=earlyoom pid=<号>（「PEER_EXTERNAL_KILL 行 0 行」）
#   MULTI_HOST_RUN_BREAK=peer-external-kill-unfiltered  ⑤ 日志里任何一个进程被杀都算，不看进程号是不是这一趟进程树里的（文件头 ⑤「被外部信号杀掉」）
#       被外部信号杀掉 日志点名的是别的进程、命令退 143 ⇒ 不打（「PEER_EXTERNAL_KILL 行 1 行」）
#   MULTI_HOST_RUN_BREAK=peer-memory-cgroup-kill-reported  ⑤ 内核「Memory cgroup out of memory」那一行（内存组到了它自己的上限）也当被外部信号杀掉打出来（文件头 ⑤「被外部信号杀掉」）
#       被外部信号杀掉 内核日志是 Memory cgroup out of memory 点名这一趟进程号、命令退 137 ⇒ 不打（「PEER_EXTERNAL_KILL 行 1 行」）
#   MULTI_HOST_RUN_BREAK=peer-card-locations-unfiltered  ⑤ on_cards= 列卡上每个进程，别的任务的也算（文件头 ⑤「实际在哪几张卡」）
#       实际在哪张卡 要卡的一趟 on_cards= 只列这一趟进程树里的进程，序号照 uuid 对回来（「不是这一趟的：」）
#       实际在哪张卡 不要卡的一趟照读 on_cards=，used_mib=0 read=no 不变（「不是这一趟的：」）
#   MULTI_HOST_RUN_BREAK=peer-temporary-root-in-scratch  ⑤ 另给第二台那一层内存包装设 RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT=<这一趟的本地目录>/temporary（命令拿到的 TMPDIR 超长）
#       临时目录 第二台那条命令拿到的 TMPDIR ⇒ 在默认临时根下、以内存包装的前缀起名、全路径不超过 60 字节（「TMPDIR「…/builds/…」」）；
#       临时目录 第二台那一层内存包装被 KILL ⇒ ⑥ 照那一层记下的路径删了它留下的临时目录 也红（记下的路径不在默认临时根下，⑥ 打「没删」那一行）
#   MULTI_HOST_RUN_BREAK=peer-temporary-record-not-passed  ⑤ 不给第二台那一层内存包装设 RUN_WITH_MEMORY_CAP_TEMPORARY_RECORD（包装被 KILL 留下的那一份没人删）
#       临时目录 第二台那一层内存包装被 KILL ⇒ ⑥ 照那一层记下的路径删了它留下的临时目录（「…（还在）…」）
#   MULTI_HOST_RUN_BREAK=peer-temporary-record-unchecked  ⑥ 不核那一层记下的路径是不是默认临时根下内存包装起的那一份，照删
#       临时目录 记下的路径不是默认临时根下内存包装起的那一份 ⇒ ⑥ 不删、打一行（「场景里那个目录不在了」）
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
# 判法的开关（变量 MULTI_HOST_CONFIGURATION_BREAK，只在 --selftest 里认，驱动读到它设在自证之外就退 2）也带进这份自证：驱动读多机配置的那一步照它弄坏的判法办。
#   下面只写这份自证红在哪一格的一条；别的值打开后红哪几格写在 scripts/crash/crash-point-replay-shard-configuration-check.sh 文件头：
#   MULTI_HOST_CONFIGURATION_BREAK=switch-off-is-on  判法把 ENABLE_ACROSS_MACHINES 没写或写 0 也按开判（驱动 ① 的双机开关那一判退 0，不退 3）
#       Ⓗ 配置里 ENABLE_ACROSS_MACHINES=0 ⇒ --peer 拒（退 255，出路写 ENABLE_ACROSS_MACHINES=1），命令哪台都没跑、第二台没建目录
#       只红这一格：Ⓘ「第二台是本机」设在自证之外那一格去掉它起驱动，Ⓦ 第二台那条命令不带它（那条命令的祖先里没有自证，判法脚本一见设着的它就退 2）
# 共用库的开关 PEER_HOST_LIB_BREAK 同样带进来，打开后红哪几格写在 scripts/hosts/peer-host-lib.sh 文件头。
# 只供测试的开关：SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1（照崩溃点重放分片）：「第二台」是本机上的另一个目录，命令在本机跑、环境照 ssh 那样只给最少几样。
#   只在 --selftest 起的驱动里认（MULTI_HOST_RUN_SELFTEST_MARK 对得上；自证另 export peer-host-lib.sh 认的 PEER_HOST_SELFTEST_MARK）：别处设了就拒、退 2，命令一行都没跑。
#   弄坏开关 this-machine-peer-honored-anywhere 下别处设了也照认。
#
# 分在两份文件里：自证 multi_host_selftest（连同它前面的场景说明）写在同目录 multi-host-run-selftest.sh 里，本文件在 --selftest 那一支里 source 它
#   （路径照本文件自己所在的目录 multi_host_script_directory 找，不照工作目录；第二台在同一个绝对路径上起本文件，同一个目录里就有它），别的调用不载入。
#   弄坏开关 detail-cut-by-bytes、scene-slice-cap-unwritten、selftest-caller-toolchain、this-machine-peer-honored-anywhere 在自证一侧的判断在 multi-host-run-selftest.sh 里，
#   别的开关的代码都在本文件，开关表仍写在本文件。
#
# admission: always 每次调都在这一刻这棵树上跑一条调用方给的命令；要不要重跑由调用方判，不在这里
# run-condition: command bash git rsync python3
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
multi_host_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=peer-host-lib.sh
source "$multi_host_script_directory/peer-host-lib.sh"
# clip_text <字节数>：标准输入每行取前 <字节数> 个字节，再丢掉被切在中间的半个字符，输出总是合法的 UTF-8（GNU 的 cut -c 与 cut -b 都按字节切，三个字节的汉字会被切断；给人看的失败行与日志摘录用它截）
clip_text() { cut -b "1-$1" | iconv -c -f UTF-8 -t UTF-8 2>/dev/null; }

# 两台相同的系统路径（文件头 ③「共享根」）：命令里点到它们不判在不在共享根下；以 / 收尾的是整个目录
MULTI_HOST_HOST_INDEPENDENT_PATHS='/dev/null /dev/stdin /dev/stdout /dev/stderr /dev/fd/ /usr/ /bin/'
# 跑着时接回（文件头 ⑤「跑着时接回」）：已接回本机的字节数、看了几轮、各 cgroup 见过的最大 usage_usec（键是 cgroup 路径）；multi_host_peer_main 起第二台那条命令之前清零
multi_host_peer_forwarded_bytes=0
multi_host_peer_polls=0
declare -A multi_host_peer_cgroup_usage=()
# 峰值（文件头 ⑤「峰值」）：第二台那一层内存包装 cgroup 进程用的内存（anon 加 shmem）见过的最大（空串是一轮都没读到）
multi_host_peer_memory_peak=""
# 被外部信号杀掉（文件头 ⑤「被外部信号杀掉」）：这一趟进程树里每一轮见过的进程号（键是进程号），与第二台那条命令起跑的本机时刻（秒）
declare -A multi_host_peer_tree_pids=()
multi_host_peer_started_at=0
multi_host_peer_current_file=memory.stat   # 每一轮读进程内存的文件名（文件头 ⑤「此刻内存」；只供自证换，文件头同一段）
multi_host_peer_memory_directory=""        # 每一轮读进程内存的目录，空是那一层内存包装的 cgroup（只供自证换成假 cgroup，文件头 ⑤「峰值」）
multi_host_peer_blocked_states=D   # 每一轮数进程树里哪几种状态的进程（文件头 ⑤「等 IO」；D 是不可中断的 IO 等待，只供自证换）
MULTI_HOST_PEER_EXIT_PEAK_NAME=exit-memory-peak   # 第二台那一层内存包装退出时写峰值的那一份，在这一趟的本地目录下（文件头 ⑤「峰值」）
# 这一条命令自己的临时目录（文件头 ⑤「这一条命令自己的临时目录」、⑥「那一层留下的临时目录」）：
MULTI_HOST_PEER_TEMPORARY_RECORD_NAME=temporary-directory   # 第二台那一层内存包装把给命令建的临时目录的全路径写进它，在这一趟的本地目录下
MULTI_HOST_PEER_DEFAULT_TEMPORARY_ROOT=/tmp   # 那一层建它的临时根：run-with-memory-cap.sh 的 TEMPORARY_ROOT 默认值；⑥ 只删直接在这下面的
MULTI_HOST_PEER_TEMPORARY_PREFIX=singlefs-run-with-memory-cap-   # 那一份的名字前缀：run-with-memory-cap.sh 的 begin_own_temporary_directory 起的名字；⑥ 只删这种名字的
MULTI_HOST_PEER_TEMPORARY_PATH_LIMIT=60   # 第二台那条命令拿到的 TMPDIR 全路径至多这么多字节（Unix 套接字路径上限 107 字节，测试还要在 TMPDIR 下接几层）
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

multi_host_remove_peer_temporary() { # ⑥ 照这一趟的本地目录里那一层内存包装记下的路径，删它没走到 EXIT 时留下的临时目录（文件头 ⑥「那一层留下的临时目录」）。
  # 记录文件不在没有要删的；路径不是默认临时根下直接一层、内存包装起的名字的不删、打一行；读不了、删不成置 multi_host_cleanup_failed=1。
  # 弄坏开关 peer-temporary-record-unchecked 下不核路径、照删
  local record="$peer_scratch/$MULTI_HOST_PEER_TEMPORARY_RECORD_NAME" recorded="" read_status=0
  local own_pattern="^${MULTI_HOST_PEER_DEFAULT_TEMPORARY_ROOT}/${MULTI_HOST_PEER_TEMPORARY_PREFIX}[0-9]+-[A-Za-z0-9]+\$"
  recorded="$(peer_host_run / "test -e $(printf '%q' "$record") || exit 3; cat -- $(printf '%q' "$record")")" || read_status=$?
  (( read_status == 3 )) && return 0
  if (( read_status != 0 )); then
    multi_host_cleanup_failed=1
    echo "  ✗ ⑥ 读不了第二台这一趟的本地目录里那一层记下的临时目录路径（builds/${peer_scratch##*/}/$MULTI_HOST_PEER_TEMPORARY_RECORD_NAME，退 $read_status），那一层留下的临时目录没删" >&2
    echo "     → 怎么办：看 ssh 连不连得上；到第二台读那份文件，照里面的路径（$MULTI_HOST_PEER_DEFAULT_TEMPORARY_ROOT 下、名字以 $MULTI_HOST_PEER_TEMPORARY_PREFIX 起头的）手动删" >&2
    return 0
  fi
  if multi_host_break_is peer-temporary-record-unchecked; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=peer-temporary-record-unchecked：⑥ 不核那一层记下的临时目录路径，照删" >&2
  elif [[ ! "$recorded" =~ $own_pattern ]]; then
    printf '  ! ⑥ 没删第二台那一层记下的临时目录「%s」：不是默认临时根 %s 下直接一层、名字是 %s<进程号>-<后缀> 的那一份\n' \
      "$recorded" "$MULTI_HOST_PEER_DEFAULT_TEMPORARY_ROOT" "$MULTI_HOST_PEER_TEMPORARY_PREFIX" | multi_host_redact >&2
    return 0
  fi
  if [[ -n "$recorded" ]] && peer_host_run / "rm -rf -- $(printf '%q' "$recorded") && test ! -e $(printf '%q' "$recorded")" >/dev/null; then
    echo "  · ⑥ 删了第二台那一层没收拾的临时目录 $recorded（回读不在了）" | multi_host_redact >&2
    return 0
  fi
  multi_host_cleanup_failed=1
  echo "  ✗ ⑥ 第二台那一层没收拾的临时目录「$recorded」没删掉" | multi_host_redact >&2
  echo "     → 怎么办：到第二台看那个目录里剩什么、有没有进程开着里面的文件（python3 scripts/hosts/run_build_directory.py holders <目录>），停掉再手动删" >&2
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
  multi_host_remove_peer_temporary   # 那一层没走到 EXIT 时留下的临时目录（文件头 ⑥「那一层留下的临时目录」）
  # 留着的编译目录（文件头「留着的编译目录」）：带 --keep-build-directory、命令等到了退出码的，只删 runs/ 下三个文件与这一层退出时峰值那一份，
  # 这一趟的本地目录（连同编译目录）留给下一次 --reuse-build-directory；回读它还在、三个文件不在了才算留下，打一行 MULTI_HOST_RUN_PEER_BUILD_KEPT。
  # 没等到退出码的（中途失败、被信号停）照删。弄坏开关 peer-build-not-kept 下一律照删
  if (( multi_host_keep_build && peer_command_exited )) && ! multi_host_break_is peer-build-not-kept; then
    if peer_host_run / "rm -f -- $(printf '%q' "$peer_scratch/$MULTI_HOST_PEER_EXIT_PEAK_NAME") $(printf '%q' "$peer_scratch/$MULTI_HOST_PEER_TEMPORARY_RECORD_NAME") $(printf '%q' "${peer_pid_file:?}") $(printf '%q' "$peer_pid_file.start") $(printf '%q' "${peer_exit_file:?}") $(printf '%q' "${peer_output_file:?}") $(printf '%q' "${peer_scope_file:?}") && test -d $(printf '%q' "$peer_scratch/target") && test ! -e $(printf '%q' "$peer_pid_file")" >/dev/null; then
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
#   同一个目录另打一行「stat <memory.current> <anon> <file> <file_dirty> <file_writeback> <shmem>」（文件头 ⑤「内存最高那一刻的明细」；memory.current 或第三个参数那个文件读不到不打，某一行没有写 0）；
#   第九个参数非空（这一趟分到的序号，逗号隔开）的另打一行「utilization <那几张逐张平均的利用率>」（文件头 ⑤「显存」；nvidia-smi 起不来、退非 0 或有一行不是整数不打），
#   第七个参数是 1（这一趟要卡）的另打一行「gpu <这一趟进程树里的进程用的显存之和>」（文件头 ⑤「显存」；nvidia-smi 起不来或退非 0 不打），第八个参数是 1（弄坏开关 peer-gpu-family-unfiltered）时不看进程树；
#   第十个参数是 1 的另打一行「cards <on_cards= 的值>」（文件头 ⑤「实际在哪几张卡」；读不到不打，由本机写 unread），第十一个参数是 1（弄坏开关 peer-card-locations-unfiltered）时不看进程树；
#   进程树（连同进程号那个 bash 自己）读完之后打一行「blocked <状态是第四个参数里某个字母的进程数>」（文件头 ⑤「等 IO」；进程号那个 bash 不在了不打）
MULTI_HOST_PEER_FOLLOW_SCRIPT='pid_file="$1" output_file="$2" current_file="$3" blocked_states="${4:-D}" memory_directory="${5:-}" with_page_cache="${6:-0}" query_gpu="${7:-0}" gpu_unfiltered="${8:-0}" gpu_indexes="${9:-}" query_cards="${10:-0}" cards_unfiltered="${11:-0}" memory_reported=0 blocked=0 threads_runnable=0 threads_total=0 threads_top=0 threads_top_name=
printf "size %s\n" "$(stat -c %s -- "$output_file" 2>/dev/null || echo 0)"
follow_pid="$(cat -- "$pid_file" 2>/dev/null)"
[[ "$follow_pid" =~ ^[0-9]+$ && -d "/proc/$follow_pid" ]] || exit 0
base_cgroup="$(sed -n "s/^0:://p" "/proc/$follow_pid/cgroup" 2>/dev/null)"
declare -A seen_cgroups=() tree_members=()
for tree_pid in $(peer_host_process_tree "$follow_pid"); do
  tree_members[$tree_pid]=1
  tree_state="$(sed -n "s/^.*) \([A-Z]\) .*$/\1/p" "/proc/$tree_pid/stat" 2>/dev/null)"
  if [[ -n "$tree_state" && "$blocked_states" == *"$tree_state"* ]]; then blocked=$(( blocked + 1 )); fi
  thread_count=0
  for thread_stat in "/proc/$tree_pid/task/"*/stat; do
    { read -r thread_line < "$thread_stat"; } 2>/dev/null || continue
    thread_rest="${thread_line##*) }"
    thread_count=$(( thread_count + 1 ))
    if [[ "${thread_rest%% *}" == R ]]; then threads_runnable=$(( threads_runnable + 1 )); fi
  done
  threads_total=$(( threads_total + thread_count ))
  if (( thread_count > threads_top )); then
    threads_top="$thread_count"
    { read -r threads_top_name < "/proc/$tree_pid/comm"; } 2>/dev/null || threads_top_name=""
  fi
  tree_cgroup="$(sed -n "s/^0:://p" "/proc/$tree_pid/cgroup" 2>/dev/null)"
  [[ -n "$tree_cgroup" && "$tree_cgroup" != "$base_cgroup" ]] || continue
  [[ -z "${seen_cgroups[$tree_cgroup]:-}" ]] || continue
  seen_cgroups[$tree_cgroup]=1
  usage="$(sed -n "s/^usage_usec //p" "/sys/fs/cgroup$tree_cgroup/cpu.stat" 2>/dev/null)"
  if [[ "$usage" =~ ^[0-9]+$ ]]; then printf "cgroup %s %s\n" "$usage" "$tree_cgroup"; fi
  (( memory_reported )) && continue
  memory_reported=1
  memory_root="${memory_directory:-/sys/fs/cgroup$tree_cgroup}"
  stat_current="$(cat -- "$memory_root/memory.current" 2>/dev/null)"
  if [[ "$stat_current" =~ ^[0-9]+$ && -r "$memory_root/$current_file" ]]; then
    stat_anon=0 stat_file=0 stat_dirty=0 stat_writeback=0 stat_shmem=0
    while read -r stat_name stat_value _; do
      [[ "$stat_value" =~ ^[0-9]+$ ]] || continue
      case "$stat_name" in
        anon) stat_anon="$stat_value" ;;
        file) stat_file="$stat_value" ;;
        file_dirty) stat_dirty="$stat_value" ;;
        file_writeback) stat_writeback="$stat_value" ;;
        shmem) stat_shmem="$stat_value" ;;
      esac
    done 2>/dev/null < "$memory_root/$current_file"
    printf "stat %s %s %s %s %s %s\n" "$stat_current" "$stat_anon" "$stat_file" "$stat_dirty" "$stat_writeback" "$stat_shmem"
  fi
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
threads_top_name="${threads_top_name//[^A-Za-z0-9._-]/_}"
printf "threads %s %s %s %s\n" "$threads_runnable" "$threads_total" "$threads_top" "${threads_top_name:-none}"
printf "tree %s\n" "${!tree_members[*]}"
if [[ "$query_gpu" == 1 || "$query_cards" == 1 ]] && gpu_apps="$(timeout 10 nvidia-smi --query-compute-apps=gpu_uuid,pid,used_memory --format=csv,noheader,nounits 2>/dev/null)"; then
  gpu_total=0
  own_apps=()
  while IFS=", " read -r gpu_uuid gpu_pid gpu_used _; do
    [[ "$gpu_pid" =~ ^[0-9]+$ ]] || continue
    if [[ "$cards_unfiltered" == 1 || -n "${tree_members[$gpu_pid]:-}" ]]; then own_apps+=("$gpu_uuid $gpu_pid $gpu_used"); fi
    [[ "$gpu_used" =~ ^[0-9]+$ ]] || continue
    if [[ "$gpu_unfiltered" == 1 || -n "${tree_members[$gpu_pid]:-}" ]]; then gpu_total=$(( gpu_total + gpu_used )); fi
  done <<<"$gpu_apps"
  if [[ "$query_gpu" == 1 ]]; then printf "gpu %s\n" "$gpu_total"; fi
  if [[ "$query_cards" == 1 ]] && (( ${#own_apps[@]} == 0 )); then
    printf "cards empty\n"
  elif [[ "$query_cards" == 1 ]] && card_rows="$(timeout 10 nvidia-smi --query-gpu=index,uuid --format=csv,noheader,nounits 2>/dev/null)"; then
    declare -A card_of_uuid=()
    while IFS=", " read -r card_index card_uuid _; do
      if [[ "$card_index" =~ ^[0-9]+$ && -n "$card_uuid" ]]; then card_of_uuid[$card_uuid]="$card_index"; fi
    done <<<"$card_rows"
    card_list=""
    for own_app in "${own_apps[@]}"; do
      read -r own_uuid own_pid own_used <<<"$own_app"
      [[ -n "${card_of_uuid[$own_uuid]:-}" ]] || continue
      [[ "$own_used" =~ ^[0-9]+$ ]] || own_used=na
      card_list+="${card_list:+,}${card_of_uuid[$own_uuid]}:$own_pid:$own_used"
    done
    if [[ -n "$card_list" ]]; then printf "cards %s\n" "$card_list"; fi
  fi
fi
if [[ -n "$gpu_indexes" ]] && gpu_rates="$(timeout 10 nvidia-smi -i "$gpu_indexes" --query-gpu=utilization.gpu --format=csv,noheader,nounits 2>/dev/null)"; then
  rate_total=0 rate_count=0 rate_unread=0
  while read -r gpu_rate; do
    [[ -n "$gpu_rate" ]] || continue
    if [[ "$gpu_rate" =~ ^[0-9]+$ ]]; then rate_total=$(( rate_total + gpu_rate )); rate_count=$(( rate_count + 1 )); else rate_unread=1; fi
  done <<<"$gpu_rates"
  if (( rate_count > 0 && ! rate_unread )); then printf "utilization %s\n" "$(( (rate_total + rate_count / 2) / rate_count ))"; fi
fi
exit 0'

multi_host_card_indexes() { # 环境里 GPU_CARDS_BY_PRIORITY 那几项的序号，逗号隔开（每项 <台标签>:<序号>:<额度>，从右边切两刀：台标签可以是 IPv6）；有一项切不出序号打空串
  local entry rest index joined=""
  for entry in ${GPU_CARDS_BY_PRIORITY:-}; do
    rest="${entry%:*}"
    index="${rest##*:}"
    [[ "$index" =~ ^[0-9]+$ && "$rest" != "$entry" ]] || return 0
    joined+="${joined:+,}$index"
  done
  printf '%s' "$joined"
}

multi_host_follow_peer() { # <round|final>：peer_host_wait_detached 每一轮之后（round）与等到结局之后（final）调（文件头 ⑤「跑着时接回」）。
  # round：一次 ssh 读输出文件的字节数、各 cgroup 的 usage_usec 与那一层内存包装 cgroup 进程用的内存（anon 加 shmem），多出来的那一截取回、到最后一个换行为止经 multi_host_redact 打到标准错误，
  #   再打一行 MULTI_HOST_RUN_PEER_CPU、紧跟一行 MULTI_HOST_RUN_PEER_MEMORY，再一行 MULTI_HOST_RUN_PEER_THREADS 与一行 MULTI_HOST_RUN_PEER_IO_WAIT；
  # final：已接回之后的全部取回打出（不以换行收尾的补一个换行），不打 CPU、内存与等 IO 那三行
  local phase="$1" state kind first rest size="" read_now=no total=0 cgroup usage_value fetch_now=0 start_byte range_text chunk_size complete partial_size redact_exit
  local threads_now="0 0 0:none" threads_read=no threads_total_now threads_top_now threads_name_now
  local tree_member current_now=0 current_read=no blocked_now=0 blocked_read=no with_page_cache=0 stat_now="" gpu_now=0 gpu_read=no query_gpu=0 gpu_unfiltered=0 gpu_indexes="" utilization_now=none query_cards=1 cards_unfiltered=0 on_cards_now=unread
  local chunk="$multi_host_scratch/peer-output.chunk"
  local -A this_round_usage=()
  # 弄坏开关 peer-memory-with-page-cache：每一轮读 memory.peak 与 memory.current（含页缓存；文件头 ⑤「峰值」）
  if multi_host_break_is peer-memory-with-page-cache; then with_page_cache=1; fi
  # 这一趟要卡的才读 nvidia-smi（文件头 ⑤「显存」）
  if [[ -n "${GPU_CARDS_BY_PRIORITY:-}" ]]; then query_gpu=1; fi
  if (( query_gpu )) && ! multi_host_break_is peer-gpu-utilization-unread; then gpu_indexes="$(multi_host_card_indexes)"; fi
  if multi_host_break_is peer-gpu-family-unfiltered; then gpu_unfiltered=1; fi
  # 实际在哪几张卡（文件头 ⑤「实际在哪几张卡」）：要卡、不要卡的那一趟都读
  if multi_host_break_is peer-card-locations-unreported || { multi_host_break_is peer-card-locations-plain-skipped && (( ! query_gpu )); }; then query_cards=0; fi
  if multi_host_break_is peer-card-locations-unfiltered; then cards_unfiltered=1; fi
  state="$(peer_host_run / "$(declare -f peer_host_process_tree)
set -- $(printf '%q' "$peer_pid_file") $(printf '%q' "$peer_output_file") $(printf '%q' "$multi_host_peer_current_file") $(printf '%q' "$multi_host_peer_blocked_states") $(printf '%q' "$multi_host_peer_memory_directory") $with_page_cache $query_gpu $gpu_unfiltered $(printf '%q' "$gpu_indexes") $query_cards $cards_unfiltered
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
      threads)   # 这一轮进程树里状态 R 的线程数、线程总数、占线程最多的进程（文件头 ⑤「线程」）
        read -r threads_total_now threads_top_now threads_name_now <<<"$rest"
        [[ "$first" =~ ^[0-9]+$ && "$threads_total_now" =~ ^[0-9]+$ && "$threads_top_now" =~ ^[0-9]+$ && "$threads_name_now" =~ ^[A-Za-z0-9._-]+$ ]] || continue
        threads_now="$first $threads_total_now $threads_top_now:$threads_name_now"
        threads_read=yes
        ;;
      tree)   # 这一轮进程树里的进程号（命令被信号终止时读第二台的日志用，文件头 ⑤「被外部信号杀掉」）
        [[ "$first${rest:+ $rest}" =~ ^[0-9]+(\ [0-9]+)*$ ]] || continue
        for tree_member in $first $rest; do multi_host_peer_tree_pids[$tree_member]=1; done
        ;;
      stat)   # memory.current 与 memory.stat 的五样（文件头 ⑤「内存最高那一刻的明细」）
        [[ "$first" =~ ^[0-9]+$ && "$rest" =~ ^[0-9]+\ [0-9]+\ [0-9]+\ [0-9]+\ [0-9]+$ ]] || continue
        stat_now="$first $rest"
        ;;
      gpu)   # 这一趟进程树里的进程用的显存之和（文件头 ⑤「显存」）
        [[ "$first" =~ ^[0-9]+$ ]] || continue
        gpu_now="$first"
        gpu_read=yes
        ;;
      utilization)   # 这一趟分到的那几张逐张平均的利用率（文件头 ⑤「显存」）
        [[ "$first" =~ ^[0-9]+$ ]] || continue
        utilization_now="$first"
        ;;
      cards)   # 这一趟进程树里的进程在哪几张卡上（文件头 ⑤「实际在哪几张卡」）
        [[ "$first" =~ ^(empty|[0-9]+:[0-9]+:([0-9]+|na)(,[0-9]+:[0-9]+:([0-9]+|na))*)$ ]] || continue
        on_cards_now="$first"
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
    range_text="tail -c +$(( start_byte + 1 )) -- $(printf '%q' "$peer_output_file")"   # shell-lint:bytes-ok 按字节偏移取第二台输出的续段，不是给人看的截断
    [[ "$phase" == final ]] || range_text+=" | head -c $(( size - start_byte ))"
    if peer_host_run / "$range_text" > "$chunk" 2>/dev/null; then
      chunk_size="$(wc -c < "$chunk")"
      complete="$chunk_size"
      # round 只打到最后一个换行：半行留到下一轮（换字按整行换，也不让本机自己打的行插进半行里）；弄坏开关 peer-output-partial-lines 下读到多少打多少
      if [[ "$phase" == round ]] && ! multi_host_break_is peer-output-partial-lines && (( chunk_size > 0 )) && [[ -n "$(tail -c 1 -- "$chunk")" ]]; then
        partial_size="$(tail -n 1 -- "$chunk" | wc -c)"
        complete=$(( chunk_size - partial_size ))
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
  # 再一行内存明细与一行显存（文件头 ⑤「内存最高那一刻的明细」「显存」）：同一轮、同一个轮数；没读到写 0 与 read=no，不漏行；
  # 弄坏开关 peer-memory-stat-unreported、peer-gpu-unreported 下各不打
  if ! multi_host_break_is peer-memory-stat-unreported; then
    local -a stat_values=(0 0 0 0 0 0)
    local stat_read=no
    if [[ -n "$stat_now" ]]; then read -r -a stat_values <<<"$stat_now"; stat_read=yes; fi
    echo "MULTI_HOST_RUN_PEER_MEMORY_STAT poll=$multi_host_peer_polls current_bytes=${stat_values[0]} anon=${stat_values[1]} file=${stat_values[2]} file_dirty=${stat_values[3]} file_writeback=${stat_values[4]} shmem=${stat_values[5]} read=$stat_read" >&2
  fi
  if ! multi_host_break_is peer-gpu-unreported; then
    echo "MULTI_HOST_RUN_PEER_GPU poll=$multi_host_peer_polls used_mib=$gpu_now read=$gpu_read utilization=$utilization_now on_cards=$on_cards_now" >&2
  fi
  # 再一行线程（文件头 ⑤「线程」）：同一轮、同一个轮数；进程树没读到写 runnable=0 total=0 top=0:none read=no，不漏行；
  # 弄坏开关 peer-threads-unreported 下不打，peer-threads-uncounted 下读到了也写 runnable=0
  if ! multi_host_break_is peer-threads-unreported; then
    local -a thread_values=(0 0 0:none)
    read -r -a thread_values <<<"$threads_now"
    if multi_host_break_is peer-threads-uncounted; then thread_values[0]=0; fi
    echo "MULTI_HOST_RUN_PEER_THREADS poll=$multi_host_peer_polls runnable=${thread_values[0]} total=${thread_values[1]} top=${thread_values[2]} read=$threads_read" >&2
  fi
  # 再一行等 IO（文件头 ⑤「等 IO」）：同一轮、同一个轮数；进程树没读到写 blocked=0 read=no，不漏行；
  # 弄坏开关 peer-io-wait-unreported 下不打，peer-io-wait-uncounted 下读到了也写 blocked=0
  if multi_host_break_is peer-io-wait-unreported; then return 0; fi
  if multi_host_break_is peer-io-wait-uncounted; then blocked_now=0; fi
  echo "MULTI_HOST_RUN_PEER_IO_WAIT poll=$multi_host_peer_polls blocked=$blocked_now read=$blocked_read" >&2
}

multi_host_report_peer_memory_peak() { # 第二台那条命令等到退出码之后打一行那一层内存包装 cgroup 的峰值（文件头 ⑤「峰值」）：按轮采到的进程内存与那一层退出时交的（扣页缓存）取大的，
  # 退出时交的那一个读不到的不打（没量到）；弄坏开关 peer-memory-peak-unreported 下不打，peer-memory-peak-unread-printed 下没量到也打 bytes=0，peak-read-only-by-sampling 下不读退出时那一个、只靠按轮采，
  # peer-peak-only-when-unsampled 下按轮采到了就不读退出时那一个，peer-memory-with-page-cache 下按轮采到的（memory.peak）与退出时读的取大的、读不到照按轮采到的
  local exit_peak=""
  if multi_host_break_is peer-memory-peak-unreported; then return 0; fi
  if ! multi_host_break_is peak-read-only-by-sampling && ! { multi_host_break_is peer-peak-only-when-unsampled && [[ -n "$multi_host_peer_memory_peak" ]]; }; then
    exit_peak="$(peer_host_run / "cat -- $(printf '%q' "$peer_scratch/$multi_host_peer_exit_peak_file") 2>/dev/null" 2>/dev/null)" || exit_peak=""
    if [[ "$exit_peak" =~ ^[1-9][0-9]*$ ]]; then
      if [[ -z "$multi_host_peer_memory_peak" ]] || (( exit_peak > multi_host_peer_memory_peak )); then multi_host_peer_memory_peak="$exit_peak"; fi
    elif ! multi_host_break_is peer-memory-with-page-cache; then
      multi_host_peer_memory_peak=""
    fi
  fi
  if [[ -z "$multi_host_peer_memory_peak" ]]; then
    multi_host_break_is peer-memory-peak-unread-printed || return 0
    multi_host_peer_memory_peak=0
  fi
  echo "MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=$multi_host_peer_memory_peak" >&2
}

# 被外部信号杀掉（文件头 ⑤「被外部信号杀掉」）：在第二台上跑的那一段。参数（前面 set -- 交进来）是往前看多少秒、认进程号的方式（filtered 只认这一趟进程树里的，any 是弄坏开关
# peer-external-kill-unfiltered）、内核「Memory cgroup out of memory」那一行算不算（skipped 不算，counted 是弄坏开关 peer-memory-cgroup-kill-reported）、
# 这一趟进程树里见过的进程号；读第二台 earlyoom（journalctl -t earlyoom）与内核（journalctl -k）这一段时间的日志，
# 头一条点名杀了这些进程号之一的打一行「<来源 earlyoom 或 kernel> <进程号> <原行>」；认杀进程的两种行同 task_server_lib.py 的 EXTERNAL_KILL_LINE（sending SIGTERM / SIGKILL to process、Killed process）
MULTI_HOST_PEER_EXTERNAL_KILL_SCRIPT='window="$1" mode="$2" cgroup_kills="$3"
shift 3
ids=" $* "
end_at="$(date +%s)"
begin_at=$(( end_at - window ))
for source in earlyoom kernel; do
  if [[ "$source" == earlyoom ]]; then selector=(-t earlyoom); else selector=(-k); fi
  while IFS= read -r line; do
    pid=""
    if [[ "$line" =~ sending\ SIG(TERM|KILL)\ to\ process\ ([0-9]+)([^0-9]|$) ]]; then pid="${BASH_REMATCH[2]}"
    elif [[ "$line" =~ Killed\ process\ ([0-9]+)\ \( ]]; then pid="${BASH_REMATCH[1]}"; fi
    [[ -n "$pid" ]] || continue
    [[ "$cgroup_kills" == counted || "$line" != *"Memory cgroup out of memory"* ]] || continue
    [[ "$mode" == any || "$ids" == *" $pid "* ]] || continue
    printf "%s %s %s\n" "$source" "$pid" "$line"
    exit 0
  done < <(timeout 15 journalctl --no-pager -q -o cat "--since=@$begin_at" "--until=@$(( end_at + 1 ))" "${selector[@]}" 2>/dev/null)
done'

multi_host_report_peer_external_kill() { # <命令的退出码>：第二台那条命令退非 0 之后，读第二台的 earlyoom 与内核日志里有没有点名这一趟进程树里的进程号被杀（文件头 ⑤「被外部信号杀掉」），
  # 有就打一行 MULTI_HOST_RUN_PEER_EXTERNAL_KILL source=<earlyoom|kernel> pid=<进程号> line=<原行，换过私有的字、压成一行>；task-server 读这一行记 killed-externally。读不到日志、没有对上的不打。
  # 内核「Memory cgroup out of memory」那一行（内存组到了它自己的上限）不算，跳过接着找；
  # 弄坏开关 peer-external-kill-unreported 下不打，peer-external-kill-unfiltered 下日志里任何一个进程被杀都算（不看进程号是不是这一趟的），peer-memory-cgroup-kill-reported 下内存组那一行也算
  local command_exit="$1" mode=filtered cgroup_kills=skipped window found source pid line
  [[ "$command_exit" != 0 ]] || return 0
  multi_host_break_is peer-external-kill-unreported && return 0
  multi_host_break_is peer-external-kill-unfiltered && mode=any
  multi_host_break_is peer-memory-cgroup-kill-reported && cgroup_kills=counted
  (( ${#multi_host_peer_tree_pids[@]} > 0 )) || [[ "$mode" == any ]] || return 0
  window=$(( $(date +%s) - multi_host_peer_started_at + 60 ))
  found="$(peer_host_run / "set -- $window $mode $cgroup_kills ${!multi_host_peer_tree_pids[*]}
$MULTI_HOST_PEER_EXTERNAL_KILL_SCRIPT" 2>/dev/null)" || return 0
  read -r source pid line <<<"$found"
  [[ "$source" =~ ^(earlyoom|kernel)$ && "$pid" =~ ^[0-9]+$ ]] || return 0
  echo "MULTI_HOST_RUN_PEER_EXTERNAL_KILL source=$source pid=$pid line=$(printf '%s' "$line" | multi_host_redact | tr '\n' ' ' | clip_text 300)" >&2
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
  # 这一条命令自己的临时目录（文件头 ⑤）：那一层照它默认的临时根建（TMPDIR 短），全路径写进这一趟的本地目录里的记录文件，那一层被 KILL 留下的由 ⑥ 照它删；
  # 弄坏开关 peer-temporary-record-not-passed 下不给记录文件，peer-temporary-root-in-scratch 下另把临时根设在这一趟的本地目录下
  multi_host_break_is peer-temporary-record-not-passed || wrapper_extras+="RUN_WITH_MEMORY_CAP_TEMPORARY_RECORD=$(printf '%q' "$peer_scratch/$MULTI_HOST_PEER_TEMPORARY_RECORD_NAME") "
  if multi_host_break_is peer-temporary-root-in-scratch; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=peer-temporary-root-in-scratch：第二台那一层内存包装的临时根设在这一趟的本地目录下" >&2
    peer_host_run / "mkdir -p $(printf '%q' "$peer_scratch/temporary")" >/dev/null \
      || multi_host_fail "第二台建不了这一趟的本地目录下的 temporary" "看多机配置 BUILD_ROOT_BY_HOST 里第二台那一项的目录可不可写、盘满没满"
    wrapper_extras+="RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT=$(printf '%q' "$peer_scratch/temporary") "
  fi
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
  echo "  · ⑤ 第二台开跑（脱离 ssh 会话起，在同一个绝对路径 $peer_start_directory 里，经 run-with-memory-cap.sh $memory_cap，编译目录在第二台这一趟的本地目录 builds/$run_label 下），每 $poll_seconds 秒看一次：跑着时把它的新输出接回本机标准错误、每一轮报一行 MULTI_HOST_RUN_PEER_CPU、一行 MULTI_HOST_RUN_PEER_MEMORY、一行 MULTI_HOST_RUN_PEER_THREADS 与一行 MULTI_HOST_RUN_PEER_IO_WAIT" >&2
  # 跑着时接回（文件头 ⑤「跑着时接回」）：每一轮之后与等到结局之后调 multi_host_follow_peer
  multi_host_peer_forwarded_bytes=0
  multi_host_peer_polls=0
  multi_host_peer_cgroup_usage=()
  multi_host_peer_memory_peak=""
  multi_host_peer_tree_pids=()
  multi_host_peer_started_at="$(date +%s)"
  peer_host_wait_detached "$peer_pid_file" "$peer_exit_file" "$poll_seconds" multi_host_follow_peer >&2
  case "$peer_host_detached_outcome" in
    "exit "*) command_exit="${peer_host_detached_outcome#exit }"; peer_command_started=0; peer_command_exited=1 ;;
    *) multi_host_fail "第二台那条命令没等到退出码：$peer_host_detached_outcome（vanished 进程没了又没留退出码；unreachable 连续 60 次 ssh 不通；never-started 60 次都没起来）" "看上面接回的输出；ssh 到第二台 runs/ 下看 $run_label.log" ;;
  esac
  multi_host_report_peer_memory_peak
  multi_host_report_peer_build_peak
  multi_host_report_peer_external_kill "$command_exit"
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


if [[ "${1:-}" == --selftest ]]; then
  source "$multi_host_script_directory/multi-host-run-selftest.sh"   # 自证只在这一支用，别的调用不载入
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
