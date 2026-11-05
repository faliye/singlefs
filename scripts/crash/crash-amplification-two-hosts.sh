#!/usr/bin/env bash
# admission: always 每次调都在这一刻的树上跑：经 task-server 分片时各片判自己那一份、直接写进 prune-server，合并那一步核两片判的块按此刻的键在服务里读得到，不分片时本机整份判；上一次的结论不替这一次作保，块的复用在判定库（prune-server）里
# run-condition: command cargo python3
# 崩溃放量一条用例分两片判（里程碑二「增补 4」第一、四、十一项）。分不分、哪一片去哪台机器、上限、线程、分卡与清场都由 task-server 管
# （records/2026-10-30-编译绕过内存总上限.md 收尾批 #5：「把以前分散到各个任务中的脚本启动和判定 统一收起来。 统一管理。」）；这一份只照服务的分片协议跑交给它的那一步，不自己连第二台：
#
#   crash-amplification-two-hosts.sh <测试目标> [--test-filter <用例函数名>] [--label <标签>] [--given-states] [--skip <测试函数名>]…
#   crash-amplification-two-hosts.sh --selftest
#       假的本机那一趟（换掉共用库的 crash_amplification_run_on_this_host，判定库演成格目录里的一份假服务）与假的 task-server（照分片协议逐步起这一份：计划、读 plan.json 照片数与 pin 起各片、
#       两片分别演成在两台上起（自证换掉多机配置判法认的这一台网卡地址）、合并）走一遍；不编、不起 cargo、不连第二台、不上卡。只在临时 prune-server 上跑（scripts/prune-server/prune_server_client_selftest.py --with-temporary-service 起，
#       共用库的 crash_amplification_selftest_service_problem 判），不是就退 2、一格都不跑。末尾一轮（开关轮）从下面「弄坏开关」那张表现读开关与点名的格，逐个打开再跑一遍 --selftest，核红的格与点名的逐格相等。
#   测试目标是 crates/singlefs-verification/tests/ 下一条走 crash_amplification::run_from_environment 的用例（例 crash_amplification_fixed_script_through_rollback），
#   或经给定状态引擎判的随机历史用例（--given-states）。
#   --test-filter：只跑测试目标里点名的那一条用例（cargo test 的 --exact）。
#   --skip <测试函数名>（可给几次）：这几条测试函数不跑，原样接在每一趟 cargo test 的参数后面（--exact 与逐条 --skip；--test-filter 已带 --exact 的不再加第二个）。
#   --given-states：测试目标经给定状态引擎判（`crash_amplification::GivenStatesEngine`：每张表在引擎里先写进库、再从库里读回来判），这里不起录入段，引擎每一趟先录再判。
#   认不出的选项按用法错退 2。
#
# 经 task-server 分片（task-server/task_server_lib.py 文件头「分片」，SHARDED_ENTRIES 登记了这一份；提交 task-server/task submit -- bash scripts/crash/crash-amplification-two-hosts.sh <测试目标> …）：
#   服务拆成计划、各片、合并几个子任务，这一份照环境里的 TASK_SERVER_SHARD_PHASE 只跑其中一步（各变量的写法在那份文件头）：
#   plan   往计划目录（TASK_SERVER_SHARD_PLAN_DIRECTORY）写：label（这一件的标签，各片与合并都照它）、source-digest（开跑时被测源码的摘要，下面「源码中途变了」）、
#          segments-registry/<文件名>（调用方设了 SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY 时那份按段登记文件的快照，各片照它）与 plan.json：两片——
#          第 0 片 {"label": …}：SHARE=0/2；第 1 片 {"label": …}：SHARE=1/2。两片都不钉台（片里没有 pin 键：records/2026-10-31-task-server巡检.md 问题 94，写了 pin 服务拒），
#          各由服务照记录挑机器；两片判的块都直接写进 prune-server 里同一份判定库（下面「判定库」），不拷库、不取回、不导入。
#          不写 memory_floor（一片的上限由服务按记录给，records/2026-10-30-编译绕过内存总上限.md 收尾批 M03）。
#          这一步不编、不起 cargo、不交显卡。
#   shard  照 TASK_SERVER_SHARD=<片号>/<片数> 只跑自己那一片（SINGLEFS_CRASH_AMPLIFICATION_SHARE=<片号>/<片数>），判的块直接写进 prune-server：
#          第 0 片：单独起（选段没设成 judge、不带 --given-states）先在这一台跑录入段 0/2（清单、流程、事实表与判表整条录，观察者只录这一片领的块，与判段同一个分法；
#            crash-amplification-lib.sh 的 crash_amplification_record_on_this_host），录成了记下观察者版本（<产物目录>/observer-version，下面「观察者版本」）、再判 0/2；
#          第 1 片：
#            单独起时不设选段，引擎先录它领的那一份观察者再判（第 0 片只录了它自己那一份观察者）；选段交过来就是 judge 的（调用方已经整份录过）只判；
#          两片都带计划目录里那份按段登记文件（有的话）；每一片照多机配置判法 --emit-addresses 认这一台的地址（不看派到哪一台、不读 TASK_SERVER_SHARD_HOST），
#            地址在多机配置的 GPU_KERNEL_SERIAL_COMPILE_HOSTS 里的另设 SINGLEFS_GPU_KERNELS_ONE_AT_A_TIME=1（判器内核一个一个编：一起编在那一台上撞过内存上限），
#            没写、或这一台不在里面就一起编；判法判不过（配置不在、写坏）这一片退 78、一趟都不起。两台照同一份配置、同一条规则定，结局只随这一台的地址变。
#            用哪张卡由用例照多机配置判法 --emit-addresses 认这一台，这里不设角色变量。日志写进产物目录（record.log、share.log）；计划目录与产物目录在共享工作根下（task-server 给的，两台同一个绝对路径），本机读得到。
#   merge  读 TASK_SERVER_SHARD_OUTPUTS（每片一行「片号<TAB>台<TAB>本机产物目录」）：各片产物目录里的日志与观察者版本拷进 <家目录>/runs/<标签>/shard-<片号>/ 留着；
#          产物表里没有第 0 片、或除第 0 片之外不是恰好一片就退 1；先核源码中途变没变（下面「源码中途变了」），再在本机起核账那一趟：SHARE=0/1（整份），
#          单独起时选段设成 judge（第 0 片录过），--given-states 不设，日志 <家目录>/runs/<标签>/merge.log。两片判的块按此刻的键（块键与判法单元指纹）在服务里都读得到，
#          这一趟就全部复用、一块不现判（CRASH_AMPLIFICATION 行的 blocks_checked_here= 与崩溃枚举用例 CRASH_CASE_BLOCKS 行的 fresh= 加起来是 0），用例断言对账为空、违例为空，它退 0 这一件才算全绿。
#          现判数大于 0：那几块按此刻的键在服务里读不到（有一片写进了别的库：连的不是同一份 prune-server，或判的不是这一棵树），退 1、点名两片的台与日志
#          （这一趟已经把它们判进服务，修好之后重投就是全部复用）；那一趟一行计数行都没打的同样退 1（判不了读没读到）；之后核观察者版本。弄坏开关 merge-blocks-unchecked：不核现判数。
#   有一片红了（cargo test 退非 0）：那一片照日志分类之后的码退（下面「退出码」），服务不起合并、整件照那一片的结局记；它判过的块已经写进服务，下一次照键复用。
# 不经 task-server 分片（环境里没有 TASK_SERVER_SHARD_PHASE：直接起，或在服务起的别的任务里调到这里）：一律本机整份跑，不分、不连第二台：
#   单独起先录入段 0/1、再判 0/1（日志 <家目录>/runs/<标签>/record.log、single.log），带 --given-states 只判 0/1。TASK_SERVER_SHARD_PHASE 是别的值退 2、什么都不起。
# 判定库：崩溃判定只在 prune-server 里那一份库（crash-amplification-lib.sh 文件头「判定库」）：本机那几趟都不拼库目录、不送库的变量，引擎经客户端直接读写；
#   连哪一处照客户端的取法（本机 unix socket；派到第二台的那一片没有 socket，走多机配置的 PRUNE_SERVER_TCP_ADDRESS）。服务连不上时那一趟退 75，这里照原样退。
#   运行目录 <家目录>/runs/<标签>/ 只放日志，家目录 ${SINGLEFS_CRASH_AMPLIFICATION_HOME:-<工作根>/crash-amplification}（共用库 crash_amplification_home_directory：
#   工作根在多机配置 NFS_SHARED_ROOTS 点名的共享根里，两台同一绝对路径，不经 $HOME；各片与合并写进同一份，本机读得到第二台那一片的日志）。
# 跑法：配置（crash-amplification-lib.sh 文件头「配置」那一份多机配置）ENABLE_GPU=1 就带 gpu 特性编，卡是 task-server 挑好、清过场经环境 GPU_CARDS_BY_PRIORITY 交下来的那一张
#   （共用库 crash_amplification_prepare 从环境取；开了 GPU 而环境里没有退 78、一趟都不起；计划那一步不用卡）。不读多机配置里的显卡先后、不自己清场。
#   走 GPU 的计算类不送（引擎开了 GPU 就三类全走 GPU）。
#   GPU 对拍的测试各占卡的显存，不并行起：开了 GPU 时每一趟 cargo test 的参数带 --test-threads 1，没开的不加。
# 往引擎送的变量（crash-point-replay.sh 文件头「往引擎送的变量」）：SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES 从调用方环境继承（服务的各步起的是同一条命令）；
#   SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY 计划那一步拷一份快照进计划目录，各片照那一份（调用方给的路径可能不在共享根下，派到第二台的那一片读不到），合并与不分片的那一趟从调用方环境继承。
# 线程：本机那几趟两步 cargo 直接起、不套包装，编译与测试线程照服务交下来的 CARGO_BUILD_JOBS、RUST_TEST_THREADS（crash-amplification-lib.sh 文件头「包装、内存上限与线程」）；
#   引擎的核对线程照共用库「线程数的缺省」（登记的重活默认并行度，SINGLEFS_THREAD_CAP 设了取它）。
# 观察者版本（第 0 片录入段之后记下、合并那一步核账那一趟之后对一遍）：挂了观察者的用例，录入段的块按观察者版本存、判段按同一把版本读（crash_amplification.rs 的 observer_versions_of_the_keys）；
#   录入段到核账那一趟之间测试二进制重编、判器单元变了，核账那一趟读不到录入段录的块，用例报「录入段那一块不在库里：先跑录入段」——叫人去补录入段，其实录入段是好的。
#   第 0 片从 record.log 里 cargo 打的 Running … (<测试二进制>) 那一行找到二进制，读出编进去的判器键表里观察者那几组（observer_code:…、observer_conclusion_composition、
#   judgement_of_the_crash_images）与工具链行取 sha256，写进 <产物目录>/observer-version；合并那一步对 merge.log 指到的二进制再取一遍：对不上而核账那一趟红了退 78，
#   出路写「两趟之间判器单元变了，重跑整件」；对不上而判绿只打一行说明；没记下、日志里找不到二进制、二进制里读不出表都不核，打一行说明。弄坏开关 observer-version-unchecked：不核。
# 源码中途变了（计划那一步记下、合并那一步核账之前再核）：各片与核账那一趟各自编，计划之后别的会话往树里打了补丁，核账那一趟编出来的二进制与两片判的不是同一份源码，核账那一趟的红不是判定红。
#   被测源码的摘要：树根下 Cargo.toml、Cargo.lock、.cargo/、crates/ 里每个普通文件的相对路径与内容 sha256，排好序再取 sha256（不进 .git、target、target-*、__pycache__；
#   与 scripts/hosts/source-stamp.py 的源文件同一批）。对不上（或那时算不出）打 CRASH_AMPLIFICATION_NOT_RUN step=merge … reason=跑的过程中源码变了，整件重跑、退 77（本次未跑，不是红）；
#   核账那一趟不起（不写 merge.log：崩溃点重放分片包装 crash-point-replay-shard-run.sh 只把 merge.log 当核账那一趟的日志抄走）；两片判完的块已经在服务里，重投时键没变的照复用。
#   计划那一步算不出摘要的打一行说明、合并之前不核。弄坏开关 source-unchecked：不核。
# 输出：各趟的 CRASH_AMPLIFICATION、CRASH_AMPLIFICATION_GPU_CARD、CRASH_AMPLIFICATION_GPU_TOTAL 行原样打出；核账那一趟的行尾加 merge_check_run=true（那一趟读的块都是两片判过、
#   写进服务的，复用数不算它的）。判段没跑的那一种（配置写了 ENABLE_GPU=1 而二进制没编进 gpu 特性：引擎每一趟判段打一行
#   CRASH_AMPLIFICATION_NOT_RUN step=judge … flow=<流> reason=…、录入段照录、退 0）只从核账那一趟（不分片时从那一趟）转打一次。
# 被停：TERM / INT 照退 143 / 130（本机那一趟收到 TERM 先由共用库转给它起的 cargo、等它退出，crash-amplification-lib.sh 文件头「被停」）。
# 运行目录的归属记录：建 <家目录>/runs/<标签>/ 之后照 scripts/hosts/run-build-directory.sh 的 run_directory_owner_record 写 .owner（会话外跑的不写、已有的不动），
#   完工清理闸（.claude/hooks/completion-cleanup-guard.sh）在这一趟的进程不在了之后照它认归属。
# 上次红的清单（scripts/gate/lib_reuse_checks.py 文件头「三」「测试目标」）：<数据根>/last-runs/scripts__crash__crash-amplification-two-hosts.sh.tsv（SINGLEFS_LAST_RUNS_DIRECTORY 盖过），
#   红的测试名一行一个：「<测试目标>」，带 --test-filter 的「<测试目标>::<用例函数>」；退 0 是绿、退 2 / 77 / 130 / 143 照留，别的是红。不分片的那一趟、本机的各片与合并都经 EXIT 钩子
#   two_hosts_on_exit 写（计划那一步与各片不写：各片的结局由合并那一步汇成整件的结局）；测试目标的源文件不在、又没显式设 SINGLEFS_LAST_RUNS_DIRECTORY 的不写。
# 退出码：0 那一步成了（核账那一趟、或不分片的那一趟退 0）；2 用法错（参数认不得、带值的选项缺值、TASK_SERVER_SHARD_PHASE 认不出、各步缺服务给的变量）；
#   1 合并时产物表里没有第 0 片、或除第 0 片之外不是恰好一片，核账那一趟现判了块（两片判的块有按此刻的键在服务里读不到的）或一行计数行都没打，
#     计划那一步写不成计划、拷不成按段登记文件，建不了运行目录、产物目录；
#   77 本次未跑：合并之前核出源码中途变了（上面「源码中途变了」）；
#   78 运行条件不满足：开了 GPU 而环境里没有 task-server 交的卡；一片认不出这一台（多机配置判法 --emit-addresses 判不过）；哪一趟的日志里有卡用不了的原话、给定状态表的判器借不到显存那一行（crash-amplification-lib.sh 的
#      crash_amplification_failure_kind 判 device）、这份二进制跑不了这一趟（结论版本与树对不上、开了 GPU 而没编进 gpu 特性；同一个函数判 build）；本机起 cargo 之前 RocksDB 预编库的取值没判过
#      （共用库 crash_amplification_run_on_this_host 里 scripts/hosts/rocksdb-prebuilt.py entry）；观察者版本变了而核账那一趟红了；
#   75 prune-server 不可用（引擎读写判定库连不上服务，那一趟原样退）；248 算法上判不了（同一个函数判 algorithm）；其余是那一趟 cargo test 的退出码，原样。
# 跑 验证包用例：只在提交时或用户要求时由主 agent 经 task-server/task submit 交；--selftest 不起 cargo。
#
# 只供测试（--selftest 给它起的每一趟里层设；平常跑时设了也照认）：
#   CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_RUN=<脚本>      换掉本机那一趟（调法同 crash-amplification-lib.sh 的 crash_amplification_run_on_this_host：
#                                                       <脚本> <share> <日志> [<附加环境 键=值>…]，自己写日志；环境照真的那一趟组）；这一趟的 cargo test 参数
#                                                       （共用库拼的 cargo_test_arguments，空格连起来）放进它环境里的 CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_TEST_ARGUMENTS
#   CRASH_AMPLIFICATION_TWO_HOSTS_SOURCE_ROOT=<目录>   被测源码的摘要算这棵树，不算本脚本所在的仓根（--selftest 给每一格设成自己造的小树，不因别的会话正改着真仓的 crates/ 退 77）
#   CRASH_AMPLIFICATION_TWO_HOSTS_SWITCH_TABLE=<文件>   --selftest 末尾的开关轮从这份文件的文件头读开关表（不设读本脚本自己的），代码里认的开关照旧从本脚本取。
# 弄坏开关 CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=<项>（几个用逗号连；只在 --selftest 里认：平常跑设了它、或设了下面共用库的 CRASH_AMPLIFICATION_ITEM_BREAK，在参数解析之前退 2、点名它只在自证里认；
# 自证设 CRASH_AMPLIFICATION_TWO_HOSTS_SELFTEST_MARK 指到它建的标记文件，自证起的每一趟里层认这份标记、都带着调用方设的值）。
# 逐项写开关名、弄坏哪一处、打开后自证哪几格红（判错的那一格打「✗ 自证 <格名>：」）。
# 写法：一行「#   <变量>=<开关名>  <弄坏哪一处>」，下一行起以「#       」开头写点名的格（、隔开；括号里是说明，开关轮不读，括号外的每一段都算一格）。
# --selftest 末尾的开关轮从这一段与下面「共用库」那一段现读（crash-amplification-lib.sh 的 crash_amplification_read_switch_table），判法是同一份库的 crash_amplification_switch_wheel：
# 表里的双机开关与代码里认的要相等、表里的共用库开关要是共用库里认的，每个逐个打开、要退 1、红的格与点名的逐格相等；共用库认而表里没列的开关也逐个打开，要退 0、一格不红。
# 并行度 CRASH_AMPLIFICATION_SELFTEST_JOBS（没给取 SINGLEFS_THREAD_CAP，再没给取登记的重活默认并行度）。
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=plan-single-shard  计划照改前只写一片（整件）
#       plan-writes-two-shards、shards-run-own-share、merge-checks-both-shards-readable（只有第 0 片、领 0/1，产物表里没有第 1 片，退 1）、judge-only-when-caller-recorded、registry-from-plan（第 1 片没起）、
#       serial-compile-follows-the-address（第 1 片没起，两片的一个一个编看不全）、serial-compile-unjudged-exits-78（第 1 片没起，没有它那个 78）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=first-shard-pinned  第 0 片照改前写 pin=local（服务拒：片里写了 pin 键）
#       plan-writes-two-shards
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=same-share-on-both-shards  第 1 片也领 0/2（两片判同一半）
#       shards-run-own-share、merge-checks-both-shards-readable（3、5 两块没人录也没人判，核账那一趟报先跑录入段、退 101）、judge-only-when-caller-recorded（第 1 片领的是 0/2）、
#       serial-compile-follows-the-address（第 1 片领的是 0/2）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=serial-compile-environment-unset  这一台在 GPU_KERNEL_SERIAL_COMPILE_HOSTS 里的那一片也不设 SINGLEFS_GPU_KERNELS_ONE_AT_A_TIME=1
#       shards-run-own-share、serial-compile-follows-the-address
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=serial-compile-on-every-host  每一片都设 SINGLEFS_GPU_KERNELS_ONE_AT_A_TIME=1（不看这一台在不在 GPU_KERNEL_SERIAL_COMPILE_HOSTS 里）
#       shards-run-own-share、serial-compile-follows-the-address
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=serial-compile-unjudged-runs  多机配置判法认不出这一台也照一起编往下跑（不退 78）
#       serial-compile-unjudged-exits-78
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=record-skipped-when-not-judging  单独起（选段没设成 judge）也不先在本机跑录入段
#       shards-run-own-share（第 0 片只起一趟、不设选段）、merge-checks-both-shards-readable（核账那一趟没设成只判）、single-host-without-phase、crash-case-shaped-records
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=record-for-given-states  带了 --given-states 也先起录入段
#       given-states-not-recorded、serial-compile-follows-the-address（带 --given-states 跑的两片各多起一趟录入段）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=other-shard-judges-only  单独起时第 1 片也照改前设 SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge（只判，它那一份观察者没人录）
#       shards-run-own-share（第 1 片报先跑录入段、退 101）、merge-checks-both-shards-readable（合并没起）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=merge-does-nothing  合并那一步照改前什么都不做、退 0
#       merge-checks-both-shards-readable、merge-shard-row-missing-refused、merge-unreadable-blocks-refused、merge-red-kept、source-change-not-run、observer-version-changed-red、judge-not-run-forwarded
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=merge-blocks-unchecked  合并不核核账那一趟现判了几块、打没打计数行，一律当两片判的块都读得到
#       merge-unreadable-blocks-refused（三种现场都退 0，不退 1）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=merge-failure-ignored  核账那一趟退非 0（对账不为空或有红）也往下走
#       merge-red-kept（退 1：那一趟没打计数行，不是原样的 101）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=no-merge-mark  核账那一趟转打时不加 merge_check_run=true
#       merge-checks-both-shards-readable
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=shard-logs-not-kept  合并不把各片产物目录里的日志拷进运行目录
#       merge-checks-both-shards-readable
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=judge-not-run-dropped  核账那一趟打的 CRASH_AMPLIFICATION_NOT_RUN step=judge 行不转打（入口认不出这一项判段没跑）
#       judge-not-run-forwarded
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=source-unchecked  合并那一步核账之前不核被测源码的摘要（改前的样子：源码中途变了照样按新源码重编、起核账那一趟）
#       source-change-not-run
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=observer-version-unchecked  核账那一趟之后不核观察者版本（改前的样子：版本变了落成假用例「先跑录入段」那一句，退核账那一趟的码）
#       observer-version-changed-red（退 101，不退 78）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=skip-not-passed-to-cargo  收了 --skip 也不接到 cargo test 的参数后面
#       skip-names-passed
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=gpu-test-threads-unset  开了 GPU 照改前不给 cargo test 参数加 --test-threads 1
#       service-card-passed
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=segments-registry-not-planned  计划那一步不拷按段登记文件，各片照调用方环境里的路径（派到第二台的那一片读不到）
#       registry-from-plan
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=unknown-phase-runs-single  TASK_SERVER_SHARD_PHASE 认不出也当没设、本机整份跑
#       phase-unknown-refused
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=shard-failure-ignored  一片那一趟退非 0 也报这一片成了（退 0）
#       shard-card-unusable、shard-given-states-run-condition
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=prepare-failure-ignored  crash_amplification_prepare 交回非 0（开了 GPU 而环境里没有服务交的卡）也照跑
#       service-card-missing-refused
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=last-run-unwritten  跑完不写上次红的清单（改前的样子）
#       last-run-list
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=run-directory-owner-unrecorded  建运行目录之后不写归属记录 .owner（改前的样子：起它的进程不在了，完工清理闸认不出这一趟归哪个会话）
#       run-directory-owner-recorded
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=usage-exits-1  用法错退 1（不退 2）
#       usage-refused
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=wheel-ignores-cells  开关轮只比退出码、不比红的格（crash_amplification_switch_wheel 的第三个参数、crash_amplification_switch_wheel_verdict 的第五个参数给 1）
#       switch-wheel-judges（造出来的输出上删一格、多一格、格名写错、没列的开关红了都判成对上）
# 共用库 crash-amplification-lib.sh 的开关 CRASH_AMPLIFICATION_ITEM_BREAK（开关表在 crash-amplification-item.sh 文件头）打开时这里也红（共用库认而这里没列的，开关轮要它一格不红）：
#   CRASH_AMPLIFICATION_ITEM_BREAK=device-unusable-as-run-failure  日志里卡用不上的原话不认，落成普通失败（不退 78）
#       shard-card-unusable（退 101，不退 78）
#   CRASH_AMPLIFICATION_ITEM_BREAK=algorithm-refusal-as-run-failure  日志里算法判不了的原话不认，落成普通失败（不退 248）
#       single-host-algorithm-refused（退 101，不退 248）
#   CRASH_AMPLIFICATION_ITEM_BREAK=given-states-run-condition-as-run-failure  给定状态表的判器借不到显存那一行（CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET）不认
#       shard-given-states-run-condition（退 101，不退 78）
#   CRASH_AMPLIFICATION_ITEM_BREAK=cards-read-from-configuration  显卡先后照改前从多机配置读（不看环境里 task-server 交的那一张）
#       service-card-passed（调用记录里 cards= 是多机配置里的 127.0.0.1:0:8G）、service-card-missing-refused（多机配置里写着卡，照跑、退 0）
#   CRASH_AMPLIFICATION_ITEM_BREAK=gpu-card-missing-unrefused  开了 GPU 而环境里没有服务交的卡也照跑
#       service-card-missing-refused
#   CRASH_AMPLIFICATION_ITEM_BREAK=library-marker-not-accepted  录入段照改前只认 CRASH_AMPLIFICATION_RECORDED 行，不认崩溃枚举用例经判定库两段跑时打的 CRASH_LIBRARY_RECORDED 行
#       crash-case-shaped-records（退 1、一趟判段都不起）
#   CRASH_AMPLIFICATION_ITEM_BREAK=this-host-service-threads-dropped  本机那一趟两步 cargo 起之前清掉服务交下来的 CARGO_BUILD_JOBS、RUST_TEST_THREADS，吃 cargo 自己的缺省
#       this-host-cargo-direct（两步 cargo 收到的 CARGO_BUILD_JOBS、RUST_TEST_THREADS 是 unset）
#
# gate-similar: crash-point-replay-shard-run.sh 它把门禁交的一条崩溃枚举用例转给这里同一步、合并之后把核账那一趟的日志抄进门禁给的日志；分两片、合并时的核账归这里
# gate-similar: mutation-shard-run.sh 它照同一份服务分片协议把变异表按用时分两堆、各片跑门禁 crates-mutation-replay；分的是变异行，不是崩溃状态块
# gate-similar: crash-amplification-item.sh 它排派活计划、本机一台跑一个分项；不调这里
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

two_hosts_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
two_hosts_script="$two_hosts_script_directory/$(basename "${BASH_SOURCE[0]}")"
repository_root="$(cd "$two_hosts_script_directory/../.." && pwd)"
source "$two_hosts_script_directory/crash-amplification-lib.sh"
source "$two_hosts_script_directory/../hosts/run-build-directory.sh"   # 只用它的 run_directory_owner_record：运行目录的归属记录

# 弄坏开关只在自证里认：是不是自证起的子进程，看 CRASH_AMPLIFICATION_TWO_HOSTS_SELFTEST_MARK 指到的文件在不在、内容对不对（只有 run_selftest 建它、设它）
two_hosts_selftest_mark_content="crash-amplification-two-hosts.sh --selftest 起的子进程"
two_hosts_under_selftest=0
if [[ -n "${CRASH_AMPLIFICATION_TWO_HOSTS_SELFTEST_MARK:-}" && -f "${CRASH_AMPLIFICATION_TWO_HOSTS_SELFTEST_MARK:-}" ]] \
   && [[ "$(cat -- "$CRASH_AMPLIFICATION_TWO_HOSTS_SELFTEST_MARK")" == "$two_hosts_selftest_mark_content" ]]; then
  two_hosts_under_selftest=1
fi

two_hosts_break_is() { [[ ",${CRASH_AMPLIFICATION_TWO_HOSTS_BREAK:-}," == *",$1,"* ]]; }
# 观察者版本（文件头「观察者版本」）：录入段之后记下、核账那一趟对一遍。版本取自这一趟 cargo test 在日志里打的 Running … (<测试二进制>) 那一行指到的二进制：
# 二进制里编进去一份判器键表（crates/singlefs-verification/src/crash_identity.rs 的 COMPILED_JUDGING_UNITS_KEYS_TEXT，构建脚本写的 judging_units_keys.tsv，表头
# 「# singlefs entry group keys」），观察者版本由其中观察者代码那一组（observer_code:…）、观察者结论拼块那一组（observer_conclusion_composition）、池级 checker 那一组
# （judgement_of_the_crash_images）的键与工具链行拼出来（crash_amplification.rs 的 observer_versions_of_the_keys）；这里把这几组的行取出来取 sha256。
# 另读的输入（extra_inputs_digest）是跑起来才算的，二进制里没有，不在这一份摘要里。
two_hosts_observer_version_of_binary() { # <测试二进制>：打「<摘要> groups=<组数> tables=<份数> binary=<路径>」一行；读不出打原因到 stderr、退 1
  python3 -I - "$1" <<'OBSERVER_VERSION_PYTHON'
import hashlib
import mmap
import sys

path = sys.argv[1]
start = b"# singlefs entry group keys\ntoolchain\t"
recognized = (b"toolchain\t", b"group\t", b"refusal\t", b"not-compiled-entry\t", b"unit\t", b"watched\t")
relevant_groups = (b"observer_conclusion_composition", b"judgement_of_the_crash_images")
try:
    with open(path, "rb") as handle:
        view = mmap.mmap(handle.fileno(), 0, access=mmap.ACCESS_READ)
except (OSError, ValueError) as error:
    print(f"读不了测试二进制 {path}：{error}", file=sys.stderr)
    sys.exit(1)
selected, tables, at = [], 0, view.find(start)
while at >= 0:
    tables += 1
    view.seek(at)
    view.readline()
    while True:
        line = view.readline()
        if not line.endswith(b"\n") or not line.startswith(recognized):
            break
        fields = line.rstrip(b"\n").split(b"\t")
        if line.startswith(b"toolchain\t"):
            selected.append(line)
        elif line.startswith(b"group\t") and len(fields) > 1 and (fields[1].startswith(b"observer_code:") or fields[1] in relevant_groups):
            selected.append(line)
    at = view.find(start, at + 1)
if tables == 0:
    print(f"测试二进制 {path} 里找不到编进去的判器键表（# singlefs entry group keys）", file=sys.stderr)
    sys.exit(1)
groups = sum(1 for line in selected if line.startswith(b"group\t"))
if groups == 0:
    print(f"测试二进制 {path} 的判器键表里没有观察者那几组（observer_code:…、observer_conclusion_composition、judgement_of_the_crash_images）", file=sys.stderr)
    sys.exit(1)
print(f"{hashlib.sha256(b''.join(selected)).hexdigest()} groups={groups} tables={tables} binary={path}")
OBSERVER_VERSION_PYTHON
}
two_hosts_observer_version_of_log() { # <日志>：日志里最后一行 Running … (<测试二进制>) 指到的二进制的观察者版本；相对路径按树根算
  local binary
  binary="$(grep -E '^[[:space:]]+Running .* \(.+\)$' "$1" 2>/dev/null | tail -n 1 | sed -E 's/^.*\((.+)\)$/\1/')"
  if [[ -z "$binary" ]]; then echo "日志 $1 里没有 cargo 打的 Running … (<测试二进制>) 那一行（日志不在，或这一趟没经 cargo test）" >&2; return 1; fi
  [[ "$binary" == /* ]] || binary="$repository_root/$binary"
  two_hosts_observer_version_of_binary "$binary"
}
# 源码中途变了（文件头「源码中途变了」）：被测源码的摘要用共用库的 crash_amplification_source_digest（crash-amplification-lib.sh 文件头「源码摘要」），这里不另写一份

# 录入段成了之后记下观察者版本（文件头「观察者版本」）；取不到打一行说明、合并那一步不核
two_hosts_note_observer_version() { # <录入段的日志> <记到哪个文件>
  local version
  rm -f -- "$2"
  if version="$(two_hosts_observer_version_of_log "$1" 2>&1)"; then
    printf '%s\n' "$version" >"$2"
    echo "  · 录入段之后记下观察者版本 ${version%% *}（$2）"
  else
    echo "  ! 观察者版本取不到，合并那一步核账之后不核版本：$version" >&2
  fi
}
# 核账那一趟的二进制里观察者版本和第 0 片录入段之后记下的对不上：退 0（置 observer_version_recorded、observer_version_now）；没记下、没法核、没变都退 1。
# 弄坏开关 observer-version-unchecked：不核（永远退 1）
observer_version_recorded="" observer_version_now=""
two_hosts_observer_version_changed() { # <记下的那个文件> <核账那一趟的日志>
  two_hosts_break_is observer-version-unchecked && return 1
  [[ -s "$1" ]] || return 1
  observer_version_recorded="$(<"$1")"
  if ! observer_version_now="$(two_hosts_observer_version_of_log "$2" 2>&1)"; then
    echo "  ! 核账那一趟的观察者版本取不到，没法核：$observer_version_now" >&2
    return 1
  fi
  [[ "${observer_version_recorded%% *}" != "${observer_version_now%% *}" ]]
}
# 摘要算哪棵树：本脚本所在的仓根；只供测试的 CRASH_AMPLIFICATION_TWO_HOSTS_SOURCE_ROOT 换成别的树（文件头「只供测试」）
two_hosts_source_root="${CRASH_AMPLIFICATION_TWO_HOSTS_SOURCE_ROOT:-$repository_root}"
# 合并那一步核账之前再算一次：计划那一步记下了、此刻对不上或算不出，退 0（置 source_digest_recorded、source_digest_now）；没记下、对得上退 1。
# 弄坏开关 source-unchecked：不核（永远退 1）
source_digest_recorded="" source_digest_now=""
two_hosts_source_changed() { # <计划目录里记下的那个文件>
  two_hosts_break_is source-unchecked && return 1
  [[ -s "$1" ]] || return 1
  source_digest_recorded="$(<"$1")"
  if ! source_digest_now="$(crash_amplification_source_digest "$two_hosts_source_root" 2>&1)"; then
    source_digest_now="算不出（$source_digest_now）"
    return 0
  fi
  [[ "${source_digest_recorded%% *}" != "${source_digest_now%% *}" ]]
}

# 只供测试（文件头「只供测试」）：换掉本机那一趟。环境照真的那一趟组（crash_amplification_environment_for_this_host），只把起 cargo 换成假的
if [[ -n "${CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_RUN:-}" ]]; then
  crash_amplification_run_on_this_host() {
    crash_amplification_environment_for_this_host "$1" "${@:3}"
    env "${this_host_environment[@]}" CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_TEST_ARGUMENTS="${cargo_test_arguments[*]}" bash "$CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_RUN" "$@"
  }
fi

usage() { # [<哪里不对>]
  echo "  ✗ ${1:+$1；}用法：$0 <测试目标> [--test-filter <用例函数名>] [--label <标签>] [--given-states] [--skip <测试函数名>]…，或 --selftest" >&2
  echo "  → 怎么办：测试目标写 crates/singlefs-verification/tests/ 下的文件名（不带 .rs），例 crash_amplification_fixed_script_through_rollback；带值的选项后面紧跟它的值" >&2
  if two_hosts_break_is usage-exits-1; then exit 1; fi
  exit 2
}
require_value() { # <选项> <余下的参数个数（含选项自己）> [<值>]：后面没给值、给的是空串或以 - 起头按用法错退 2
  if (( $2 >= 2 )) && [[ -n "${3:-}" && "${3:-}" != -* ]]; then return 0; fi
  usage "$1 后面缺值"
}

# 一趟红了（文件头「退出码」）：照日志分类之后打 ✗ 与出路、退分类之后的码
two_hosts_refuse_failed_run() { # <那一趟的退出码> <日志> <哪一趟的一句>
  local code
  code="$(crash_amplification_exit_code_of_a_failed_run "$1" "$2")"
  echo "  ✗ $3退 $1；日志 $2" >&2
  grep -h -A 3 'panicked' "$2" 2>/dev/null | head -8 | sed 's/^/    /' >&2
  if [[ "$code" == 78 || "$code" == 248 ]]; then
    crash_amplification_explain_exit_code "$code"
  else
    echo "  → 怎么办：看那份日志里 test result 与 panicked 那几行；判过的块已经在 prune-server 里，修好之后重投同一条命令接着判" >&2
  fi
  exit "$code"
}

# 本机那几趟共用的准备（文件头「跑法」）：共用库拼参数、取服务交的卡；开了 GPU 每一趟 cargo test 带 --test-threads 1；--skip 的名单接在后面
two_hosts_prepare() {
  local skip_name
  crash_amplification_prepare || two_hosts_break_is prepare-failure-ignored || exit 78
  if (( gpu_switch )) && ! two_hosts_break_is gpu-test-threads-unset; then cargo_test_arguments+=(--test-threads 1); fi
  # --test-filter 已带 --exact 的不再加第二个（libtest 的 --exact 只认一次，给两次整趟用法错）
  if (( ${#skip_names[@]} )) && ! two_hosts_break_is skip-not-passed-to-cargo; then
    [[ " ${cargo_test_arguments[*]} " == *" --exact "* ]] || cargo_test_arguments+=(--exact)
    for skip_name in "${skip_names[@]}"; do cargo_test_arguments+=(--skip "$skip_name"); done
  fi
}

two_hosts_open_run_directory() { # <运行目录>：建好、写归属记录（文件头「运行目录的归属记录」；弄坏开关 run-directory-owner-unrecorded 照改前不写）
  if ! mkdir -p -- "$1"; then
    echo "  ✗ 建不了运行目录 $1" >&2
    echo "  → 怎么办：看 $crash_amplification_home 可不可写，或把 SINGLEFS_CRASH_AMPLIFICATION_HOME 指到可写的目录" >&2
    exit 1
  fi
  two_hosts_break_is run-directory-owner-unrecorded || run_directory_owner_record "$1" '双机驱动这一趟的运行目录'
}

# 转打核账那一趟的行（文件头「输出」）：CRASH_AMPLIFICATION 行尾加 merge_check_run=true，判段没跑的那一行只从这里转一次
two_hosts_print_merge_lines() { # <核账那一趟的日志>
  local merge_mark=" merge_check_run=true"
  two_hosts_break_is no-merge-mark && merge_mark=""
  grep -h '^CRASH_AMPLIFICATION ' "$1" 2>/dev/null | sed "s/\$/$merge_mark/"
  two_hosts_break_is judge-not-run-dropped && return 0
  grep -h '^CRASH_AMPLIFICATION_NOT_RUN step=judge ' "$1" 2>/dev/null
}

# 自证：假的本机那一趟（换掉共用库的 crash_amplification_run_on_this_host）加一份假的 task-server（run_phases 照分片协议逐步起这一份），每一格起几趟里层 bash <本脚本>，
# 看退出码、转打的行、调用记录与假服务里的块。假的用例：一共 6 块，SHARE=k/n 领序号除以 n 余 k 的那几块；判定库演成格目录里的一个目录（假的 prune-server：blocks 判过的块、
# records 录过观察者的块，一行一块的序号；同一格里各片与合并连的是同一个，就像两台连同一份服务），已判的块记复用、不重判。各格各用自己的格目录与标签（运行目录），不读别的格留下的东西。
# 只在临时 prune-server 上跑（文件头「用法」--selftest；共用库 crash_amplification_selftest_service_problem 判），不是就退 2、一格都不跑。
run_selftest() {
  local work failures=0 cells=0 step_exit=0 phase_exits="" scene="" service_problem
  service_problem="$(crash_amplification_selftest_service_problem)"
  if [[ -n "$service_problem" ]]; then
    echo "  ✗ crash-amplification-two-hosts.sh 自证没跑：$service_problem" >&2
    echo "     → 怎么办：照 scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service 那一条，在临时服务底下起这一份自证（带 --server-binary 就不编）" >&2
    exit 2
  fi
  work="$(mktemp -d "${TMPDIR:-/tmp}/crash-amplification-two-hosts-selftest.XXXXXX")" || {
    echo "  ✗ --selftest：建不了临时目录（TMPDIR=${TMPDIR:-/tmp}）" >&2
    echo "  → 怎么办：看 TMPDIR 可不可写" >&2
    exit 2
  }
  trap "rm -rf -- '$work'" EXIT   # 现在展开：退出时局部变量 work 不一定还在
  # 自证起的子进程的标记：只有这里建、这里设（文件头「弄坏开关」）；里层认它才认弄坏开关
  printf '%s\n' "$two_hosts_selftest_mark_content" >"$work/mark"
  export CRASH_AMPLIFICATION_TWO_HOSTS_SELFTEST_MARK="$work/mark"
  cat >"$work/case.sh" <<'STUB'
#!/usr/bin/env bash
# 假的本机那一趟（自证用；调法 <share> <日志> [<附加环境 键=值>…]，环境由驱动照真的那一趟组好）：判定库是 STUB_SERVICE 指的目录（假的 prune-server）。
#   录入段（STAGE=record）把领的块录进 records、打录入行就退；STAGE 没设先录再判；STAGE=judge 只判，领的块没录过就照真引擎报「先跑录入段」、退 101。
#   判段：领的块在 blocks 里就复用，不在就现判、写进 blocks。整份（n=1）那一趟对账：6 块有没判的退 101。
#   STUB_FAILURE=<哪一种趟：record|judge>:<怎么坏：device|algorithm|given-states|plain|judge-not-run|silent> 照点名演那一趟（silent：一行计数行都不打、退 0）；
#   STUB_CASE_SHAPE=crash-case 照崩溃点重放用例的样子（录入段只打 CRASH_LIBRARY_RECORDED；
#   判段打 CRASH_CASE_BLOCKS 行、不打 CRASH_AMPLIFICATION 行）；STUB_BINARY_<RECORD|JUDGE> 照 cargo 打那一行 Running … (<二进制>)。
#   每一趟往 STUB_CALLS 记一行：step= kind= share= stage= kernels= cards=[…] registry=[…] args=[…]
set -uo pipefail
share="${SINGLEFS_CRASH_AMPLIFICATION_SHARE:?}" service="${STUB_SERVICE:?}" log="$2"
stage="${SINGLEFS_CRASH_AMPLIFICATION_STAGE-unset}"
kind=judge
[[ "$stage" == record ]] && kind=record
arguments="${CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_TEST_ARGUMENTS:-}"
echo "step=${STUB_STEP:-main} kind=$kind share=$share stage=$stage kernels=${SINGLEFS_GPU_KERNELS_ONE_AT_A_TIME:-unset} cards=[${GPU_CARDS_BY_PRIORITY-unset}] registry=[${SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY-unset}] args=[$arguments]" >>"${STUB_CALLS:?}"
exec >"$log" 2>&1
running_variable="STUB_BINARY_${kind^^}"
[[ -z "${!running_variable:-}" ]] || echo "     Running tests/stub.rs (${!running_variable})"
crash_case=0
[[ "${STUB_CASE_SHAPE:-amplification}" == crash-case ]] && crash_case=1
failure="${STUB_FAILURE:-}"
if [[ -n "$failure" && "${failure%%:*}" == "$kind" ]]; then
  case "${failure#*:}" in
    device) echo "GPU_CARD_NOT_USABLE index=0：配置点名了这张卡，本机却用不了它"; exit 101 ;;
    algorithm) echo "卡 0 判不了点 p 块内第 5 个状态（恢复内核第 1 字 7、checker 内核第 3 字 9：表越过了内核的容量）"; exit 101 ;;
    given-states) echo "CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET reason=假的：池里没有一张卡借得出显存"; exit 101 ;;
    plain) echo "test result: FAILED. 0 passed; 1 failed"; exit 101 ;;
    silent) echo "test result: ok. 1 passed; 0 failed"; exit 0 ;;
    judge-not-run) echo "CRASH_AMPLIFICATION_NOT_RUN step=judge test=crash_amplification::run_from_environment flow=stub reason=假的：这份二进制没有编进 gpu 特性" ;;
  esac
fi
mkdir -p -- "$service"
: >>"$service/blocks"; : >>"$service/records"
share_index="${share%/*}" share_count="${share#*/}"
owned=()
for block in 0 1 2 3 4 5; do (( block % share_count == share_index )) && owned+=("$block"); done
has() { grep -qx -- "$2" "$1" 2>/dev/null; }
if [[ "$kind" == record || "$stage" == unset ]]; then
  for block in "${owned[@]}"; do has "$service/records" "$block" || echo "$block" >>"$service/records"; done
  if (( crash_case )); then
    echo "CRASH_LIBRARY_RECORDED class=crash_state_enumeration item=stub::full manifest=0000000000000000"
  else
    echo "CRASH_AMPLIFICATION_RECORDED flow=stub blocks=${#owned[@]} exhaustive=true"
  fi
  [[ "$kind" == record ]] && exit 0
fi
checked=0 reused=0
for block in "${owned[@]}"; do
  if has "$service/blocks" "$block"; then reused=$((reused + 1)); continue; fi
  if ! has "$service/records" "$block"; then echo "thread 'stub' panicked at tests/stub.rs:1:1: 录入段那一块不在库里：先跑录入段（块 $block）"; exit 101; fi
  echo "$block" >>"$service/blocks"
  checked=$((checked + 1))
done
unjudged=0
for block in 0 1 2 3 4 5; do has "$service/blocks" "$block" || unjudged=$((unjudged + 1)); done
if (( crash_case )); then
  echo "CRASH_CASE_BLOCKS blocks=${#owned[@]} reused=$reused fresh=$checked red=0 refused=0"
else
  echo "CRASH_AMPLIFICATION share=$share states_checked_here=$checked blocks_checked_here=$checked reused=$reused unjudged_blocks=$unjudged red=0"
fi
if (( share_count == 1 && unjudged > 0 )); then
  echo "thread 'stub' panicked at tests/stub.rs:1:1: 对账不为空：$unjudged 块没人判"
  exit 101
fi
exit 0
STUB
  # 两份假的测试二进制（观察者版本那几格）：编进去的判器键表里观察者那一组的键不同（录入段那一趟的、重编之后的）
  printf '# singlefs entry group keys\ntoolchain\tselftest\ngroup\tobserver_code:stub\tkey-at-the-record\n' >"$work/binary-at-the-record"
  printf '# singlefs entry group keys\ntoolchain\tselftest\ngroup\tobserver_code:stub\tkey-after-a-rebuild\n' >"$work/binary-after-a-rebuild"
  # 被测源码的摘要（文件头「源码中途变了」）：各格默认算这一棵造出来的小树，不读真仓的 crates/（别的会话正改着它）
  mkdir -p "$work/source-tree/crates/stub/src" "$work/remote-bin"
  printf '[workspace]\n' >"$work/source-tree/Cargo.toml"
  printf '// 自证用的被测源码\n' >"$work/source-tree/crates/stub/src/lib.rs"
  # 不分片那一格把这两份放在 PATH 最前面：连第二台要用的 ssh、rsync 被起一次就往 STUB_REMOTE_CALLS 记一笔
  local remote
  for remote in ssh rsync; do
    printf '#!/usr/bin/env bash\necho "%s $*" >>"${STUB_REMOTE_CALLS:-/dev/null}"\nexit 1\n' "$remote" >"$work/remote-bin/$remote"
    chmod +x "$work/remote-bin/$remote"
  done
  printf '%s\n' 'ENABLE_GPU=1' 'GPU_CARDS_BY_PRIORITY="127.0.0.1:0:8G"' >"$work/multi-host-gpu-on.env"
  # 里层默认读的多机配置：写全键（各片经多机配置判法 --emit-addresses 认这一台，整份要判得过），两个开关都关；主机数组写 127.0.0.1（每一台的网卡上都有）
  #   与文档用的地址段 192.0.2.66（不在任何一台上）；内核一个一个编的台写 192.0.2.66。另两份只换内核一个一个编的台：写 127.0.0.1、不写
  printf '%s\n' "HOSTS=127.0.0.1 192.0.2.66" "CARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=/selftest/this-cargo 192.0.2.66=/selftest/other-cargo" \
    "BUILD_ROOT_BY_HOST=127.0.0.1=/selftest/this-builds 192.0.2.66=/selftest/other-builds" "SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=2G 192.0.2.66=2G" \
    QUIESCE_STOP_COMMAND= QUIESCE_STOPPED_CHECK_COMMAND= QUIESCE_START_COMMAND= QUIESCE_STARTED_CHECK_COMMAND= ENABLE_ACROSS_MACHINES=0 ENABLE_GPU=0 \
    "NFS_SHARED_ROOTS=$work" >"$work/multi-host-without-serial.env"
  { cat "$work/multi-host-without-serial.env"; echo "GPU_KERNEL_SERIAL_COMPILE_HOSTS=192.0.2.66"; } >"$work/multi-host.env"
  { cat "$work/multi-host-without-serial.env"; echo "GPU_KERNEL_SERIAL_COMPILE_HOSTS=127.0.0.1"; } >"$work/multi-host-serial-here.env"
  printf '自证用的按段登记文件\n' >"$work/segments.tsv"
  # 里层读的多机配置指到上面那一份（两个开关都关），运行目录（日志）在临时目录下，上次红的清单写这里
  export SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" SINGLEFS_CRASH_AMPLIFICATION_HOME="$work/home" \
    SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_RUN="$work/case.sh"

  scene_new() { scene="$work/$1"; mkdir -p "$scene"; : >"$scene/calls"; }   # <格名>：格目录放进 scene
  service_with() { # <假服务的目录> <判过的块，空格隔开> <录过的块，空格隔开>
    local block
    mkdir -p "$1"; : >"$1/blocks"; : >"$1/records"
    for block in $2; do echo "$block" >>"$1/blocks"; done
    for block in $3; do echo "$block" >>"$1/records"; done
  }
  blocks_of() { sort -n "$1/blocks" 2>/dev/null | tr '\n' ' '; }   # <假服务的目录>
  calls_of() { grep "^step=$2 " "$1/calls" 2>/dev/null; }          # <格目录> <步名>
  # 起一趟里层：在 <起的目录> 里起本脚本，输出落 <格目录>/<步名>.output、调用记录落 <格目录>/calls，退出码放 step_exit；
  #   假服务是 <格目录>/service（各片与合并同一个），环境里另给 STUB_SERVICE 的照给的（演那一趟连的是别的服务）
  driver_step() { # <格目录> <步名> <起的目录> [<环境 键=值>…] -- <参数…>
    local step_scene="$1" step="$2" start="$3" environment=()
    shift 3
    while (($# > 0)) && [[ "$1" != -- ]]; do environment+=("$1"); shift; done
    shift
    ( cd "$start" && exec env -u SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE -u SINGLEFS_CRASH_AMPLIFICATION_STAGE \
        -u SINGLEFS_GPU_KERNELS_ONE_AT_A_TIME -u GPU_CARDS_BY_PRIORITY \
        -u SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY -u SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES -u SINGLEFS_CRASH_LIBRARY_RUN_KIND \
        -u TASK_SERVER_SHARD_PHASE -u TASK_SERVER_SHARD -u TASK_SERVER_SHARD_PLAN_DIRECTORY -u TASK_SERVER_SHARD_OUTPUT_DIRECTORY -u TASK_SERVER_SHARD_OUTPUTS -u TASK_SERVER_SHARD_HOST \
        -u CONFIGURATION_SELFTEST_LOCAL_ADDRESSES -u SINGLEFS_THREAD_CAP -u WORK_DIRECTORY_OWNER_SESSION -u STUB_REMOTE_CALLS \
        STUB_FAILURE= STUB_CASE_SHAPE= STUB_BINARY_RECORD= STUB_BINARY_JUDGE= STUB_STEP="$step" STUB_CALLS="$step_scene/calls" STUB_SERVICE="$step_scene/service" \
        CRASH_AMPLIFICATION_TWO_HOSTS_SOURCE_ROOT="$work/source-tree" "${environment[@]+"${environment[@]}"}" \
        bash "$two_hosts_script" "$@" ) >"$step_scene/$step.output" 2>&1
    step_exit=$?
  }
  # 假的 task-server（文件头「经 task-server 分片」，写法照 task-server/task_server_lib.py 文件头「分片」）：在本机起计划那一步；读 plan.json 照片数起各片——
  #   两个目录变量都交绝对路径（计划目录 <格目录>/plan，产物目录 <格目录>/output-<片号>：服务把它们放在共享工作根下，两台同一个绝对路径）；
  #   第 0 片演成在 127.0.0.1 那一台起、第 1 片演成在 192.0.2.66 那一台起（同一个目录、同一棵树里起，不拷不取回）：多机配置判法认的这一台网卡地址经
  #   CONFIGURATION_SELFTEST_LOCAL_ADDRESSES 换成那一台的（判法只在自证里认它），TASK_SERVER_SHARD_HOST 照服务给那一台的地址；每片记一份 <格目录>/shard-<片号>.where
  #   （host= plan=<它读的计划目录> output=<它写的产物目录>），各片跑完把假服务里判过的块记进 <格目录>/blocks-after-shards；到 merge 为止且各片都退 0，才写 TASK_SERVER_SHARD_OUTPUTS
  #   （<格目录>/outputs.tsv）、在本机起合并。phase_exits 是「plan=<码> shard-0=<码> … merge=<码>」（没起的不写）
  run_phases() { # <格目录> <到哪一步为止：shards|merge> [<环境 键=值>…] -- <参数…>
    local phase_scene="$1" until="$2" environment=() count pins index host plan_used output_used rows="" failed=0 shard_hosts=(127.0.0.1 192.0.2.66)
    shift 2
    while (($# > 0)) && [[ "$1" != -- ]]; do environment+=("$1"); shift; done
    shift
    phase_exits=""
    mkdir -p "$phase_scene/plan"
    driver_step "$phase_scene" plan "$phase_scene" "${environment[@]+"${environment[@]}"}" TASK_SERVER_SHARD_PHASE=plan TASK_SERVER_SHARD_PLAN_DIRECTORY="$phase_scene/plan" -- "$@"
    phase_exits="plan=$step_exit"
    (( step_exit == 0 )) || return 0
    read -r count pins <<<"$(python3 -c 'import json, sys
plan = json.load(open(sys.argv[1], encoding="utf-8"))
print(len(plan["shards"]), ",".join(shard.get("pin") or "free" for shard in plan["shards"]))' "$phase_scene/plan/plan.json" 2>/dev/null)"
    [[ "$count" =~ ^[0-9]+$ ]] || { phase_exits+=" plan-unreadable"; return 0; }
    for ((index = 0; index < count; index++)); do
      host="${shard_hosts[index % 2]}"
      plan_used="$phase_scene/plan" output_used="$phase_scene/output-$index"
      driver_step "$phase_scene" "shard-$index" "$phase_scene" "${environment[@]+"${environment[@]}"}" TASK_SERVER_SHARD_PHASE=shard TASK_SERVER_SHARD="$index/$count" TASK_SERVER_SHARD_HOST="$host" \
        CONFIGURATION_SELFTEST_LOCAL_ADDRESSES="$host" \
        TASK_SERVER_SHARD_PLAN_DIRECTORY="$plan_used" TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$output_used" -- "$@"
      phase_exits+=" shard-$index=$step_exit"
      printf 'host=%s plan=%s output=%s\n' "$host" "$plan_used" "$output_used" >"$phase_scene/shard-$index.where"
      rows+="$index"$'\t'"$host"$'\t'"$output_used"$'\n'
      (( step_exit == 0 )) || failed=1
    done
    blocks_of "$phase_scene/service" >"$phase_scene/blocks-after-shards"
    [[ "$until" == merge ]] && (( ! failed )) || return 0
    printf '%s' "$rows" >"$phase_scene/outputs.tsv"
    driver_step "$phase_scene" merge "$phase_scene" "${environment[@]+"${environment[@]}"}" TASK_SERVER_SHARD_PHASE=merge TASK_SERVER_SHARD_PLAN_DIRECTORY="$phase_scene/plan" \
      TASK_SERVER_SHARD_OUTPUTS="$phase_scene/outputs.tsv" -- "$@"
    phase_exits+=" merge=$step_exit"
  }
  where_of() { sed -n "s/.* $3=\([^ ]*\).*/\1/p; s/^$3=\([^ ]*\).*/\1/p" "$1/shard-$2.where" 2>/dev/null | head -n 1; }   # <格目录> <片号> <host|plan|output>
  # 合并那几格自己造的计划目录与两片的产物：计划目录只写 label；两片的产物目录空着；假服务里第 0 片判过 0、2、4，第 1 片判过的块照给的（写 - 是它一块都没判进这份服务），
  #   6 块都录过；TASK_SERVER_SHARD_OUTPUTS 两行（第三个参数写 only-first 时只有第 0 片那一行）
  merge_scene() { # <格名> <第 1 片判过的块，空格隔开；写 - 是一块都没判进这份服务> [only-first]
    local other_blocks="$2"
    scene_new "$1"
    mkdir -p "$scene/plan" "$scene/output-0" "$scene/output-1"
    printf '%s\n' "$1" >"$scene/plan/label"
    [[ "$other_blocks" == - ]] && other_blocks=""
    service_with "$scene/service" "0 2 4 $other_blocks" "0 1 2 3 4 5"
    if [[ "${3:-}" == only-first ]]; then
      printf '0\tlocal\t%s\n' "$scene/output-0" >"$scene/outputs.tsv"
    else
      printf '0\tlocal\t%s\n1\tpeer\t%s\n' "$scene/output-0" "$scene/output-1" >"$scene/outputs.tsv"
    fi
  }
  merge_in() { # <格目录> [<环境 键=值>…] [-- <参数…>]：起合并那一步（参数不给是 some-target）
    local merge_scene_directory="$1" environment=() arguments=(some-target)
    shift
    while (($# > 0)) && [[ "$1" != -- ]]; do environment+=("$1"); shift; done
    if (($# > 0)); then shift; arguments=("$@"); fi
    driver_step "$merge_scene_directory" merge "$merge_scene_directory" TASK_SERVER_SHARD_PHASE=merge TASK_SERVER_SHARD_PLAN_DIRECTORY="$merge_scene_directory/plan" \
      TASK_SERVER_SHARD_OUTPUTS="$merge_scene_directory/outputs.tsv" "${environment[@]+"${environment[@]}"}" -- "${arguments[@]}"
  }
  cell_failed() { # <格名> <判错的一句> <出路> [<格目录>]
    local failed_scene="${4:-$work/$1}" output
    echo "  ✗ 自证 $1：$2（调用记录「$(tr '\n' ';' <"$failed_scene/calls" 2>/dev/null)」）" >&2
    echo "     → 怎么办：$3" >&2
    for output in "$failed_scene"/*.output; do
      [[ -f "$output" ]] || continue
      echo "       | ${output##*/}：" >&2
      tail -n 4 "$output" | sed 's/^/       |   /' >&2
    done
    failures=$((failures + 1))
  }
  # selftest-refused-outside-temporary-service（文件头「用法」--selftest）：摘掉临时服务的标记再起一份自证 ⇒ 退 2、说自证没跑，一格都不跑
  scene_new selftest-refused-outside-temporary-service
  env -u PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET bash "$two_hosts_script" --selftest >"$scene/bare.output" 2>&1; step_exit=$?
  cells=$((cells + 1))
  if [[ "$step_exit" != 2 ]] || ! grep -q 'crash-amplification-two-hosts.sh 自证没跑：' "$scene/bare.output" || grep -q '✗ 自证 ' "$scene/bare.output"; then
    cell_failed selftest-refused-outside-temporary-service "不在临时 prune-server 底下起的自证应当退 2、说自证没跑、一格都不跑（退 $step_exit）" \
      "看 run_selftest 开头调共用库 crash_amplification_selftest_service_problem 那一段"
  fi

  # plan-writes-two-shards（文件头「经 task-server 分片」plan）：只起计划那一步：退 0、一趟本机那一趟都没起；
  #   plan.json 两片，都不钉（片里没有 pin 键），都不带 memory_floor；label 是这一格的标签；source-digest 是「<64 位摘要> files=<数>」；计划目录里只有这几样、没有库
  scene_new plan-writes-two-shards
  mkdir -p "$scene/plan"
  driver_step "$scene" plan "$scene" TASK_SERVER_SHARD_PHASE=plan TASK_SERVER_SHARD_PLAN_DIRECTORY="$scene/plan" -- some-target --label plan-writes-two-shards
  local plan_view plan_entries
  plan_view="$(python3 -c 'import json, sys
plan = json.load(open(sys.argv[1], encoding="utf-8"))
print(" ".join(("pin=" + shard["pin"] if "pin" in shard else "unpinned") + ":" + str("memory_floor" in shard) for shard in plan["shards"]))' "$scene/plan/plan.json" 2>&1)"
  plan_entries="$(cd "$scene/plan" && printf '%s ' *)"
  cells=$((cells + 1))
  if [[ "$step_exit" != 0 || "$plan_view" != "unpinned:False unpinned:False" || "$(cat "$scene/plan/label" 2>/dev/null)" != plan-writes-two-shards || -s "$scene/calls" ]] \
     || ! grep -qE '^[0-9a-f]{64} files=[0-9]+$' "$scene/plan/source-digest" 2>/dev/null || [[ "$plan_entries" != "label plan.json source-digest " ]]; then
    cell_failed plan-writes-two-shards "计划那一步应当退 0、不起本机那一趟，plan.json 两片（都不钉、片里没有 pin 键，都不带 memory_floor），记下标签与源码摘要、计划目录里没有别的（退 $step_exit；plan.json「$plan_view」；计划目录里「$plan_entries」）" \
      "看 two_hosts_plan：计划目录里只写 label、source-digest、segments-registry/（调用方给了按段登记文件才有）与两片的 plan.json"
  fi

  # 两片与合并（同一个场景 two-shards）：假服务里已有块 1（判过、录过）；第 0 片带录入段那一趟的二进制（观察者版本）；照假的 task-server 起计划、两片、合并
  scene_new two-shards
  service_with "$scene/service" "1" "1"
  run_phases "$scene" merge STUB_BINARY_RECORD="$work/binary-at-the-record" -- some-target --label two-shards
  # shards-run-own-share：两片都退 0；第 0 片先录 0/2 再只判 0/2；第 1 片领 1/2、不设选段（先录它那一份观察者再判）；
  #   默认配置的内核一个一个编的台是 192.0.2.66：第 1 片（在那一台）内核一个一个编，第 0 片（在 127.0.0.1）一起编；
  #   两片连的是同一份服务：第 1 片照服务里的块 1 复用、现判 3、5；第 0 片记下观察者版本；两片跑完服务里 6 块都在（各片直接写进服务）
  local shard_host shard_kernels="unset" first_kernels="unset" shard_problems=""
  shard_host="$(where_of "$scene" 1 host)"
  [[ "$shard_host" == 192.0.2.66 ]] && shard_kernels=1
  [[ "$(where_of "$scene" 0 host)" == 192.0.2.66 ]] && first_kernels=1
  [[ "$phase_exits" == "plan=0 shard-0=0 shard-1=0"* ]] || shard_problems+="各步退「$phase_exits」；"
  [[ "$(calls_of "$scene" shard-0 | sed -n 1p)" == "step=shard-0 kind=record share=0/2 stage=record kernels=$first_kernels "* \
     && "$(calls_of "$scene" shard-0 | sed -n 2p)" == "step=shard-0 kind=judge share=0/2 stage=judge kernels=$first_kernels "* \
     && "$(calls_of "$scene" shard-0 | wc -l)" == 2 ]] || shard_problems+="第 0 片的调用不是「先录 0/2、再只判 0/2」；"
  [[ "$(calls_of "$scene" shard-1 | wc -l)" == 1 \
     && "$(calls_of "$scene" shard-1)" == "step=shard-1 kind=judge share=1/2 stage=unset kernels=$shard_kernels "* ]] \
    || shard_problems+="第 1 片（$shard_host）的调用不是「领 1/2、不设选段、内核一个一个编 $shard_kernels」；"
  grep -q '^CRASH_AMPLIFICATION share=1/2 states_checked_here=2 blocks_checked_here=2 reused=1 ' "$scene/shard-1.output" || shard_problems+="第 1 片没照服务里的块 1 复用、现判 3 与 5；"
  grep -qE '^[0-9a-f]{64} groups=1 tables=1 binary=' "$scene/output-0/observer-version" 2>/dev/null || shard_problems+="第 0 片没记下观察者版本；"
  [[ "$(cat "$scene/blocks-after-shards" 2>/dev/null)" == "0 1 2 3 4 5 " ]] || shard_problems+="两片跑完服务里的块是「$(cat "$scene/blocks-after-shards" 2>/dev/null)」（要 6 块都在：各片直接写进服务）；"
  cells=$((cells + 1))
  [[ -z "$shard_problems" ]] || cell_failed shards-run-own-share "$shard_problems" \
    "看 two_hosts_shard：两片都直接写服务、不建库；第 0 片先录再只判；第 1 片领自己的份、不设选段；这一台在 GPU_KERNEL_SERIAL_COMPILE_HOSTS 里的设内核一个一个编" "$scene"
  # merge-checks-both-shards-readable：合并退 0；只起一趟核账：领 0/1、只判；两片判的块按此刻的键全读得到、一块不现判；恰好一行带 merge_check_run=true（现判 0、对账为空）；
  #   第 1 片的日志拷进运行目录
  local merge_problems=""
  [[ "$phase_exits" == "plan=0 shard-0=0 shard-1=0 merge=0" ]] || merge_problems+="各步退「$phase_exits」；"
  [[ "$(calls_of "$scene" merge | wc -l)" == 1 && "$(calls_of "$scene" merge)" == "step=merge kind=judge share=0/1 stage=judge "* ]] \
    || merge_problems+="合并的调用不是恰好一趟「领 0/1、只判」；"
  [[ "$(grep -c '^CRASH_AMPLIFICATION .* merge_check_run=true$' "$scene/merge.output" 2>/dev/null)" == 1 ]] \
     && grep -q '^CRASH_AMPLIFICATION share=0/1 states_checked_here=0 blocks_checked_here=0 reused=6 unjudged_blocks=0 .*merge_check_run=true$' "$scene/merge.output" \
    || merge_problems+="没有恰好一行带 merge_check_run=true、现判 0、复用 6、对账为空的行；"
  [[ -f "$work/home/runs/two-shards/shard-1/share.log" && -f "$work/home/runs/two-shards/merge.log" ]] || merge_problems+="第 1 片的日志没拷进运行目录 runs/two-shards/shard-1/、或核账那一趟的 merge.log 不在；"
  cells=$((cells + 1))
  [[ -z "$merge_problems" ]] || cell_failed merge-checks-both-shards-readable "$merge_problems" \
    "看 two_hosts_merge：读 TASK_SERVER_SHARD_OUTPUTS、拷各片日志、起核账那一趟（SHARE=0/1）、核现判数、经 two_hosts_print_merge_lines 转打" "$scene"
  # merge-shard-row-missing-refused：产物表里只有第 0 片那一行 ⇒ 合并退 1、一趟都没起、输出点名第 0 片之外的片数
  merge_scene merge-shard-row-missing-refused "1 3 5" only-first
  merge_in "$scene"
  cells=$((cells + 1))
  if [[ "$step_exit" != 1 || -s "$scene/calls" ]] || ! grep -q '除第 0 片之外有 0 片' "$scene/merge.output"; then
    cell_failed merge-shard-row-missing-refused "产物表里只有第 0 片，合并应当退 1、一趟都不起（退 $step_exit）" "看 two_hosts_merge 里 other_count 不是 1 那一段"
  fi
  # merge-unreadable-blocks-refused（文件头「经 task-server 分片」merge）：三种现场，合并都退 1、点名读不到或判不了：
  #   第 1 片判的块没进这份服务（演它写进了别的库）⇒ 核账那一趟现判了 1、3、5（崩溃放量那一种的 blocks_checked_here=3）；
  #   照崩溃点重放用例的样子（点名用例）同样没进 ⇒ CRASH_CASE_BLOCKS 行 fresh=3；核账那一趟一行计数行都没打（silent）⇒ 判不了
  local unreadable_problems="" unreadable_shape
  for unreadable_shape in amplification crash-case silent; do
    merge_scene "merge-unreadable-blocks-refused-$unreadable_shape" "$([[ "$unreadable_shape" == silent ]] && echo "1 3 5" || echo -)"
    case "$unreadable_shape" in
      amplification) merge_in "$scene" ;;
      crash-case) merge_in "$scene" STUB_CASE_SHAPE=crash-case -- some-target --test-filter the_full_case ;;
      silent) merge_in "$scene" STUB_FAILURE=judge:silent ;;
    esac
    if [[ "$unreadable_shape" == silent ]]; then
      { [[ "$step_exit" == 1 ]] && grep -q '一行计数行都没打' "$scene/merge.output"; } || unreadable_problems+="核账那一趟一行计数行都没打，合并退 $step_exit（要 1、说判不了）；"
    else
      { [[ "$step_exit" == 1 ]] && grep -q '核账那一趟现判了 3 块' "$scene/merge.output"; } || unreadable_problems+="$unreadable_shape 那一种第 1 片的块读不到，合并退 $step_exit（要 1、说现判了 3 块）；"
    fi
  done
  cells=$((cells + 1))
  [[ -z "$unreadable_problems" ]] || cell_failed merge-unreadable-blocks-refused "$unreadable_problems" \
    "看 two_hosts_merge 末尾 two_hosts_blocks_judged_in_the_check 那一段：现判数大于 0、或一行计数行都没有，都退 1" "$work/merge-unreadable-blocks-refused-amplification"
  # merge-red-kept：核账那一趟红了（退 101）⇒ 合并原样退 101
  merge_scene merge-red-kept "1 3 5"
  merge_in "$scene" STUB_FAILURE=judge:plain
  cells=$((cells + 1))
  [[ "$step_exit" == 101 ]] || cell_failed merge-red-kept "核账那一趟退 101，合并应当原样退 101（退 $step_exit）" "看 two_hosts_merge 里 check_exit 不是 0 那一段"
  # source-change-not-run（文件头「源码中途变了」）：计划时记下这一格那棵小树的摘要，合并之前往树里追加一行 ⇒ 打 CRASH_AMPLIFICATION_NOT_RUN step=merge、退 77，
  #   核账那一趟没起（一趟都没调、merge.log 不在）
  merge_scene source-change-not-run "1 3 5"
  mkdir -p "$scene/source-tree/crates/stub/src"
  printf '[workspace]\n' >"$scene/source-tree/Cargo.toml"; printf '// 自证用的被测源码\n' >"$scene/source-tree/crates/stub/src/lib.rs"
  crash_amplification_source_digest "$scene/source-tree" >"$scene/plan/source-digest" 2>/dev/null
  printf '// 计划之后打进来的补丁\n' >>"$scene/source-tree/crates/stub/src/lib.rs"
  merge_in "$scene" CRASH_AMPLIFICATION_TWO_HOSTS_SOURCE_ROOT="$scene/source-tree"
  cells=$((cells + 1))
  if [[ "$step_exit" != 77 || -s "$scene/calls" || -e "$work/home/runs/source-change-not-run/merge.log" ]] \
     || ! grep -q '^CRASH_AMPLIFICATION_NOT_RUN step=merge test=crash_amplification::run_from_environment reason=跑的过程中源码变了' "$scene/merge.output"; then
    cell_failed source-change-not-run "计划之后源码变了，合并应当打 step=merge 那一行、退 77、核账那一趟不起（退 $step_exit）" \
      "看 two_hosts_merge 里 two_hosts_source_changed 那一段"
  fi
  # observer-version-changed-red（文件头「观察者版本」）：第 0 片记下的观察者版本与核账那一趟的二进制对不上、核账那一趟红了 ⇒ 退 78、说两趟之间判器单元变了
  merge_scene observer-version-changed-red "1 3 5"
  printf '%064d groups=1 tables=1 binary=recorded-by-the-shard\n' 0 >"$scene/output-0/observer-version"
  merge_in "$scene" STUB_BINARY_JUDGE="$work/binary-after-a-rebuild" STUB_FAILURE=judge:plain
  cells=$((cells + 1))
  if [[ "$step_exit" != 78 ]] || ! grep -q '判器单元变了' "$scene/merge.output"; then
    cell_failed observer-version-changed-red "观察者版本变了而核账那一趟红了，合并应当退 78、说两趟之间判器单元变了（退 $step_exit）" "看 two_hosts_merge 里 two_hosts_observer_version_changed 那一段"
  fi
  # judge-not-run-forwarded（文件头「输出」）：核账那一趟打了一行 CRASH_AMPLIFICATION_NOT_RUN step=judge、照常退 0 ⇒ 合并退 0、那一行恰好转打一次
  merge_scene judge-not-run-forwarded "1 3 5"
  merge_in "$scene" STUB_FAILURE=judge:judge-not-run
  cells=$((cells + 1))
  if [[ "$step_exit" != 0 || "$(grep -c '^CRASH_AMPLIFICATION_NOT_RUN step=judge ' "$scene/merge.output")" != 1 ]]; then
    cell_failed judge-not-run-forwarded "核账那一趟判段没跑（打了 CRASH_AMPLIFICATION_NOT_RUN step=judge），那一行应当恰好转打一次、合并退 0（退 $step_exit）" "看 two_hosts_print_merge_lines"
  fi

  # single-host-without-phase（文件头「不经 task-server 分片」）：环境里没有 TASK_SERVER_SHARD_PHASE ⇒ 本机整份：先录 0/1、再只判 0/1，退 0，转打 share=0/1 那一行；
  #   一次都不连第二台（PATH 最前面的 ssh、rsync 一次都没被起），不领 0/2、1/2
  scene_new single-host-without-phase
  driver_step "$scene" main "$scene" PATH="$work/remote-bin:$PATH" STUB_REMOTE_CALLS="$scene/remote-calls" -- some-target --label single-host-without-phase
  cells=$((cells + 1))
  if [[ "$step_exit" != 0 || "$(wc -l <"$scene/calls")" != 2 || -e "$scene/remote-calls" ]] \
     || [[ "$(sed -n 1p "$scene/calls")" != "step=main kind=record share=0/1 stage=record "* || "$(sed -n 2p "$scene/calls")" != "step=main kind=judge share=0/1 stage=judge "* ]] \
     || ! grep -q '^CRASH_AMPLIFICATION share=0/1 ' "$scene/main.output"; then
    cell_failed single-host-without-phase "没有 TASK_SERVER_SHARD_PHASE 时应当本机整份（先录 0/1、再只判 0/1）、退 0、一次都不连第二台（退 $step_exit；连第二台的记录「$(tr '\n' ';' <"$scene/remote-calls" 2>/dev/null)」）" \
      "看主流程 case 里空串那一支与 two_hosts_single"
  fi
  # phase-unknown-refused：TASK_SERVER_SHARD_PHASE=bogus ⇒ 退 2、一趟都没起
  scene_new phase-unknown-refused
  driver_step "$scene" main "$scene" TASK_SERVER_SHARD_PHASE=bogus -- some-target --label phase-unknown-refused
  cells=$((cells + 1))
  [[ "$step_exit" == 2 && ! -s "$scene/calls" ]] || cell_failed phase-unknown-refused "TASK_SERVER_SHARD_PHASE 认不出应当退 2、一趟都不起（退 $step_exit）" "看主流程认 TASK_SERVER_SHARD_PHASE 那一段"
  # shard-card-unusable、shard-given-states-run-condition：第 0 片那一趟的日志里有卡用不了的原话 / 给定状态表借不到显存那一行 ⇒ 这一片退 78
  local failing_cell failing_setting
  for failing_cell in shard-card-unusable shard-given-states-run-condition; do
    scene_new "$failing_cell"
    mkdir -p "$scene/plan"; printf '%s\n' "$failing_cell" >"$scene/plan/label"
    if [[ "$failing_cell" == shard-card-unusable ]]; then failing_setting=(STUB_FAILURE=judge:device SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge); else failing_setting=(STUB_FAILURE=judge:given-states); fi
    driver_step "$scene" shard-0 "$scene" "${failing_setting[@]}" TASK_SERVER_SHARD_PHASE=shard TASK_SERVER_SHARD=0/2 \
      TASK_SERVER_SHARD_PLAN_DIRECTORY="$scene/plan" TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$scene/output-0" -- some-target \
      $([[ "$failing_cell" == shard-given-states-run-condition ]] && echo --given-states)
    cells=$((cells + 1))
    [[ "$step_exit" == 78 ]] || cell_failed "$failing_cell" "这一片那一趟的日志里有运行条件不满足的原话，应当退 78（退 $step_exit）" \
      "看 two_hosts_shard 末尾与 crash-amplification-lib.sh 的 crash_amplification_failure_kind 的 device 那一组"
  done
  # single-host-algorithm-refused：本机整份那一趟的日志里是算法判不了的原话 ⇒ 退 248
  scene_new single-host-algorithm-refused
  driver_step "$scene" main "$scene" STUB_FAILURE=judge:algorithm -- some-target --given-states
  cells=$((cells + 1))
  [[ "$step_exit" == 248 ]] || cell_failed single-host-algorithm-refused "那一趟算法判不了，应当退 248（退 $step_exit）" "看 two_hosts_refuse_failed_run 与 crash_amplification_failure_kind 的 algorithm 那一组"
  # given-states-not-recorded：带 --given-states ⇒ 不起录入段，只起一趟、不设选段
  scene_new given-states-not-recorded
  driver_step "$scene" main "$scene" -- some-target --given-states
  cells=$((cells + 1))
  [[ "$step_exit" == 0 && "$(wc -l <"$scene/calls")" == 1 && "$(cat "$scene/calls")" == "step=main kind=judge share=0/1 stage=unset "* ]] \
    || cell_failed given-states-not-recorded "带 --given-states 应当只起一趟 0/1、不设选段（退 $step_exit）" "看 record_wanted 那一句"
  # judge-only-when-caller-recorded：调用方把选段设成 judge（整份录过，假服务里 6 块都录过）⇒ 两片都只判、一趟录入段都不起；第 0 片领 0/2、第 1 片领 1/2
  scene_new judge-only-when-caller-recorded
  service_with "$scene/service" "" "0 1 2 3 4 5"
  run_phases "$scene" shards SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge -- some-target --label judge-only-when-caller-recorded
  cells=$((cells + 1))
  if [[ "$phase_exits" != "plan=0 shard-0=0 shard-1=0" || "$(wc -l <"$scene/calls")" != 2 ]] \
     || [[ "$(calls_of "$scene" shard-0)" != "step=shard-0 kind=judge share=0/2 stage=judge "* || "$(calls_of "$scene" shard-1)" != "step=shard-1 kind=judge share=1/2 stage=judge "* ]]; then
    cell_failed judge-only-when-caller-recorded "选段交过来就是 judge，两片都应当只判自己那一份、不起录入段（各步退「$phase_exits」）" "看 two_hosts_shard 里 record_wanted 为 0 的那两支" "$scene"
  fi
  # serial-compile-follows-the-address（文件头「经 task-server 分片」shard）：两片判同一套规则，只看这一台的地址在不在 GPU_KERNEL_SERIAL_COMPILE_HOSTS 里：
  #   写 127.0.0.1 ⇒ 第 0 片（在 127.0.0.1）一个一个编、第 1 片（在 192.0.2.66）一起编；不写 ⇒ 两片都一起编（默认配置写 192.0.2.66 的那一半在 shards-run-own-share）
  local serial_problems="" serial_scene serial_expected serial_configuration
  for serial_scene in serial-here:multi-host-serial-here:1,unset serial-unwritten:multi-host-without-serial:unset,unset; do
    IFS=: read -r serial_scene serial_configuration serial_expected <<<"$serial_scene"
    scene_new "serial-compile-follows-the-address-$serial_scene"
    run_phases "$scene" shards SINGLEFS_MULTI_HOST_CONFIG="$work/$serial_configuration.env" -- some-target --label "$serial_scene" --given-states
    [[ "$phase_exits" == "plan=0 shard-0=0 shard-1=0" \
       && "$(calls_of "$scene" shard-0)" == "step=shard-0 kind=judge share=0/2 stage=unset kernels=${serial_expected%,*} "* \
       && "$(calls_of "$scene" shard-1)" == "step=shard-1 kind=judge share=1/2 stage=unset kernels=${serial_expected#*,} "* ]] \
      || serial_problems+="$serial_configuration.env：各步退「$phase_exits」，两片的内核一个一个编应当依次是 ${serial_expected}（调用「$(tr '\n' ';' <"$scene/calls")」）；"
  done
  cells=$((cells + 1))
  [[ -z "$serial_problems" ]] || cell_failed serial-compile-follows-the-address "$serial_problems" \
    "看 two_hosts_shard 与 two_hosts_serial_compile_here：照 --emit-addresses 打的 THIS_HOST_ADDRESS 在不在 GPU_KERNEL_SERIAL_COMPILE_HOSTS 里定，不看派到哪一台" "$scene"
  # serial-compile-unjudged-exits-78：多机配置不在（判法认不出这一台）⇒ 两片都退 78、一趟都不起
  scene_new serial-compile-unjudged-exits-78
  run_phases "$scene" shards SINGLEFS_MULTI_HOST_CONFIG="$work/absent-multi-host.env" -- some-target --label serial-compile-unjudged --given-states
  cells=$((cells + 1))
  [[ "$phase_exits" == "plan=0 shard-0=78 shard-1=78" && -z "$(calls_of "$scene" shard-0)$(calls_of "$scene" shard-1)" ]] \
    || cell_failed serial-compile-unjudged-exits-78 "多机配置判法认不出这一台，两片都应当退 78、一趟都不起（各步退「$phase_exits」）" \
      "看 two_hosts_serial_compile_here 判法退非 0 那一支与 two_hosts_shard 退 78 那一句" "$scene"
  # registry-from-plan（文件头「往引擎送的变量」）：调用方设了按段登记文件 ⇒ 计划目录里一份它的快照，两片都照它们各自读的计划目录里那一份（不是调用方的路径）
  scene_new registry-from-plan
  run_phases "$scene" shards SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY="$work/segments.tsv" -- some-target --label registry-from-plan --given-states
  local registry_problems="" registry_index registry_line
  cmp -s "$work/segments.tsv" "$scene/plan/segments-registry/segments.tsv" || registry_problems+="计划目录里没有那份按段登记文件的快照；"
  for registry_index in 0 1; do
    registry_line="$(calls_of "$scene" "shard-$registry_index")"
    [[ -n "$registry_line" && -z "$(grep -vF "registry=[$(where_of "$scene" "$registry_index" plan)/segments-registry/segments.tsv]" <<<"$registry_line")" ]] \
      || registry_problems+="第 $registry_index 片收到的按段登记不是它读的计划目录里那一份（「$registry_line」）；"
  done
  cells=$((cells + 1))
  [[ -z "$registry_problems" ]] || cell_failed registry-from-plan "$registry_problems" "看 two_hosts_plan 拷 segments-registry 与 two_hosts_shard 读它那一段" "$scene"
  # skip-names-passed：点名用例、给两个 --skip ⇒ 每一趟 cargo test 参数里恰好一个 --exact、跳过名单原样接在后面
  scene_new skip-names-passed
  driver_step "$scene" main "$scene" -- some-target --given-states --test-filter the_case --skip the_slow_case --skip the_flaky_case
  local skip_line skip_wrong=""
  while IFS= read -r skip_line; do
    [[ "$skip_line" == *"--exact the_case --skip the_slow_case --skip the_flaky_case]"* && "$(grep -o -- '--exact' <<<"$skip_line" | wc -l)" == 1 ]] || skip_wrong+="「$skip_line」；"
  done <"$scene/calls"
  cells=$((cells + 1))
  [[ "$step_exit" == 0 && -s "$scene/calls" && -z "$skip_wrong" ]] || cell_failed skip-names-passed "每一趟 cargo test 的参数都应当带恰好一个 --exact 与原样的跳过名单（退 $step_exit；$skip_wrong）" \
    "看 two_hosts_prepare 接 --skip 那一段与 crash-amplification-lib.sh 的 crash_amplification_prepare 里 --exact 那一句"
  # crash-case-shaped-records：照崩溃点重放用例的样子造的用例（录入段只打 CRASH_LIBRARY_RECORDED）、点名用例 ⇒ 录入段真跑、那一行算录入成了并转打、再只判，退 0
  scene_new crash-case-shaped-records
  driver_step "$scene" main "$scene" STUB_CASE_SHAPE=crash-case -- some-target --test-filter the_full_case
  cells=$((cells + 1))
  if [[ "$step_exit" != 0 || "$(wc -l <"$scene/calls")" != 2 || "$(sed -n 1p "$scene/calls")" != "step=main kind=record "* || "$(sed -n 2p "$scene/calls")" != "step=main kind=judge share=0/1 stage=judge "* ]] \
     || ! grep -qxF "CRASH_LIBRARY_RECORDED class=crash_state_enumeration item=stub::full manifest=0000000000000000" "$scene/main.output"; then
    cell_failed crash-case-shaped-records "照崩溃点重放用例的样子造的用例：录入段应当真跑、CRASH_LIBRARY_RECORDED 那一行算录入成了并转打、再只判，退 0（退 $step_exit）" \
      "看 crash-amplification-lib.sh 的 crash_amplification_prepare 与 crash_amplification_record_on_this_host 认的录入行"
  fi
  # service-card-passed（文件头「跑法」）：多机配置开了 GPU、写着 127.0.0.1:0:8G，环境里是 task-server 交的 127.0.0.1:1:20G ⇒ 每一趟收到的卡是服务交的那一张、
  #   cargo test 参数带 --test-threads 1，退 0
  scene_new service-card-passed
  driver_step "$scene" main "$scene" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on.env" GPU_CARDS_BY_PRIORITY=127.0.0.1:1:20G \
    -- some-target --given-states
  cells=$((cells + 1))
  if [[ "$step_exit" != 0 || ! -s "$scene/calls" ]] || grep -vF ' cards=[127.0.0.1:1:20G] ' "$scene/calls" | grep -q . || grep -v -- '--test-threads 1' "$scene/calls" | grep -q .; then
    cell_failed service-card-passed "开了 GPU：每一趟应当收到环境里服务交的那一张卡（127.0.0.1:1:20G）、cargo test 带 --test-threads 1（退 $step_exit）" \
      "看 crash-amplification-lib.sh 的 crash_amplification_prepare 取卡那一段、crash_amplification_environment_for_this_host 与 two_hosts_prepare"
  fi
  # service-card-missing-refused：开了 GPU、环境里没有服务交的卡 ⇒ 退 78、一趟都没起、说是 task-server 交的显卡
  scene_new service-card-missing-refused
  driver_step "$scene" main "$scene" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on.env" -- some-target --given-states
  cells=$((cells + 1))
  if [[ "$step_exit" != 78 || -s "$scene/calls" ]] || ! grep -q 'task-server 交的显卡' "$scene/main.output"; then
    cell_failed service-card-missing-refused "开了 GPU 而环境里没有服务交的卡，应当退 78、一趟都不起（退 $step_exit）" "看 crash-amplification-lib.sh 的 crash_amplification_prepare 里 gpu_cards_by_priority 为空那一支"
  fi
  # last-run-list（文件头「上次红的清单」）：清单写这一格自己的目录；整个目标红 ⇒ 清单 beta-target；点名用例绿 ⇒ 不动；点名用例红 ⇒ 添 beta-target::one_case；整个目标绿 ⇒ 全换掉（清单空）
  scene_new last-run-list
  local last_run_step last_run_target last_run_filter last_run_setting last_run_seen="" last_run_index=0
  for last_run_step in "beta-target::STUB_FAILURE=judge:plain" "beta-target:one_case:" "beta-target:one_case:STUB_FAILURE=judge:plain" "beta-target::"; do
    IFS=: read -r last_run_target last_run_filter last_run_setting <<<"$last_run_step"
    last_run_index=$((last_run_index + 1))
    driver_step "$scene" "run-$last_run_index" "$scene" SINGLEFS_LAST_RUNS_DIRECTORY="$scene/last-runs" ${last_run_setting:+"$last_run_setting"} \
      -- "$last_run_target" ${last_run_filter:+--test-filter "$last_run_filter"} --given-states
    last_run_seen+="$step_exit[$(sort "$scene/last-runs/scripts__crash__crash-amplification-two-hosts.sh.tsv" 2>/dev/null | tr '\n' ',')]"
  done
  cells=$((cells + 1))
  [[ "$last_run_seen" == "101[beta-target,]0[beta-target,]101[beta-target,beta-target::one_case,]0[]" ]] \
    || cell_failed last-run-list "四趟之后的退出码与清单应当依次是 101「beta-target」、0「beta-target」、101「beta-target、beta-target::one_case」、0「空」，实际 $last_run_seen" \
      "看 two_hosts_on_exit（经 scripts/gate/lib_reuse_checks.py last-run-target）"
  # run-directory-owner-recorded（文件头「运行目录的归属记录」）：会话号给了 ⇒ 运行目录里有 .owner、第一行是那个会话号
  scene_new run-directory-owner-recorded
  driver_step "$scene" main "$scene" WORK_DIRECTORY_OWNER_SESSION=selftest-session -- some-target --given-states --label run-directory-owner-recorded
  cells=$((cells + 1))
  [[ "$step_exit" == 0 && "$(head -n 1 "$work/home/runs/run-directory-owner-recorded/.owner" 2>/dev/null)" == "session: selftest-session" ]] \
    || cell_failed run-directory-owner-recorded "建运行目录之后应当写 .owner、第一行 session: selftest-session（退 $step_exit）" "看 two_hosts_open_run_directory"
  # usage-refused：认不出的选项（用一个从没用过的名字）与缺值、缺测试目标都按用法错退 2，一趟都没起
  scene_new usage-refused
  local usage_case usage_wrong="" usage_index=0
  for usage_case in "some-target --never-used-option" "some-target --never-used-option any" "some-target --label" "--given-states"; do
    usage_index=$((usage_index + 1))
    # shellcheck disable=SC2086  # 每一种写法照空格拆成参数
    driver_step "$scene" "case-$usage_index" "$scene" -- $usage_case
    [[ "$step_exit" == 2 ]] || usage_wrong+="「$usage_case」退 $step_exit；"
  done
  cells=$((cells + 1))
  [[ -z "$usage_wrong" && ! -s "$scene/calls" ]] || cell_failed usage-refused "用法错应当退 2、一趟都不起：$usage_wrong" "看参数解析那一段与 usage"

  # ── 本机那一趟两步 cargo 直接起（crash-amplification-lib.sh 文件头「包装、内存上限与线程」）：另起一个 bash 只 source 共用库，调真的 crash_amplification_run_on_this_host；
  #   仓根换成格目录下的空树（再套一层仓里的包装就起不来、交回非 0），PATH 最前面的假 cargo 把收到的 CARGO_BUILD_JOBS、RUST_TEST_THREADS 与只编还是跑记进调用记录；
  #   这里设 CARGO_BUILD_JOBS=5、RUST_TEST_THREADS=4 当服务交的值
  scene_new this-host-cargo-direct
  mkdir -p "$scene/root" "$scene/bin"
  printf '%s\n' '#!/usr/bin/env bash' 'step=test; [[ " $* " == *" --no-run "* ]] && step=build' \
    'echo "jobs=${CARGO_BUILD_JOBS-unset} tests=${RUST_TEST_THREADS-unset} step=$step" >>"${STUB_CALLS:?}"' >"$scene/bin/cargo"
  chmod +x "$scene/bin/cargo"
  env -u SINGLEFS_THREAD_CAP STUB_CALLS="$scene/calls" CARGO_BUILD_JOBS=5 RUST_TEST_THREADS=4 PATH="$scene/bin:$PATH" \
    bash -c 'repository_root="$1"; source "$2" || exit 3; common_environment=(); gpu_cards_by_priority=""
             cargo_build_arguments=(test --no-run); cargo_test_arguments=(test)
             crash_amplification_run_on_this_host 0/1 "$3/run.log"' \
    this-host-run "$scene/root" "$two_hosts_script_directory/crash-amplification-lib.sh" "$scene" >"$scene/main.output" 2>&1
  step_exit=$?
  cells=$((cells + 1))
  [[ "$step_exit" == 0 && "$(tr '\n' ';' <"$scene/calls")" == "jobs=5 tests=4 step=build;jobs=5 tests=4 step=test;" ]] \
    || cell_failed this-host-cargo-direct "本机那一趟两步 cargo 应当直接起（不套包装）、原样收到服务交的 CARGO_BUILD_JOBS=5、RUST_TEST_THREADS=4、交回 0（退 $step_exit；调用记录「$(tr '\n' ';' <"$scene/calls")」）" \
      "看 crash_amplification_run_on_this_host：两步都是 env <本机环境> cargo …，不套包装、不清掉也不改写这两个变量"

  # ── 开关轮的判法（crash-amplification-lib.sh 的 crash_amplification_switch_wheel_verdict）：不起里层，拿一份造出来的输出（红在 cell-a、cell-b 两格）
  #   比几种点名：照实点名的、没列而一格不红的要判对上；删一格、多一格、格名写错、退出码不对、表里没写格、没列而红了的要判对不上
  local wheel_sample="$work/switch-wheel-judges" exit_only=0 verdicts
  two_hosts_break_is wheel-ignores-cells && exit_only=1
  mkdir -p "$wheel_sample"; : >"$wheel_sample/calls"; : >"$wheel_sample/empty"
  printf '%s\n' '  ✗ 自证 cell-b：假的' '     → 怎么办：假的' '  ✗ 自证 cell-a：假的' '  ✗ 自证 cell-a：又一次' >"$wheel_sample/red"
  verdict_of() { # <点名的格> <退出码> <造出来的输出：red|empty>
    if crash_amplification_switch_wheel_verdict "$1" "$2" "$wheel_sample/$3" '✗ 自证 ([a-z0-9-]+)：' "$exit_only" >/dev/null; then echo 对上; else echo 对不上; fi  # gate-lint:detail 判法读的模式串，不是拒绝
  }
  verdicts="照实点名=$(verdict_of 'cell-a cell-b' 1 red) 删一格=$(verdict_of cell-a 1 red) 多一格=$(verdict_of 'cell-a cell-b cell-c' 1 red)"
  verdicts+=" 格名写错=$(verdict_of 'cell-a cell-bb' 1 red) 退出码不对=$(verdict_of 'cell-a cell-b' 0 red) 没写格=$(verdict_of '' 1 red)"
  verdicts+=" 没列而一格不红=$(verdict_of - 0 empty) 没列而红了=$(verdict_of - 1 red)"
  cells=$((cells + 1))
  if [[ "$verdicts" != "照实点名=对上 删一格=对不上 多一格=对不上 格名写错=对不上 退出码不对=对不上 没写格=对不上 没列而一格不红=对上 没列而红了=对不上" ]]; then
    cell_failed switch-wheel-judges "开关轮的判法应当只认照实点名的（退 1、红的格逐格相等）与没列而一格不红的（退 0），别的都判对不上；实判「$verdicts」" \
      "看 crash-amplification-lib.sh 的 crash_amplification_switch_wheel_verdict：第五个参数是 0 时要逐格比，点名的格是空串一律对不上"
  fi

  if (( failures > 0 )); then
    echo "  ✗ --selftest：$cells 格里 $failures 格判错" >&2
    echo "     → 怎么办：照上面每一格给的出路改，再跑 bash $two_hosts_script --selftest" >&2
    exit 1
  fi
  if crash_amplification_switch_wheel_inner; then echo "  ✓ --selftest（开关轮里起的一份，不再起开关轮）：$cells 格都对"; exit 0; fi

  # 开关轮（文件头「弄坏开关」）：从文件头现读开关与点名的格、与代码里认的开关比齐，逐个打开再跑一遍 --selftest，红的格要与点名的逐格相等
  local table assignment named in_code in_library in_table_two="" in_table_item="" duplicated stray switch round_failures=0 listed_count unlisted_count=0
  switch_wheel_assignments=(); switch_wheel_expected=()
  local switch_table_source="${CRASH_AMPLIFICATION_TWO_HOSTS_SWITCH_TABLE:-$two_hosts_script}"
  [[ "$switch_table_source" == "$two_hosts_script" ]] || echo "  · 开关轮：开关表读自 $switch_table_source（CRASH_AMPLIFICATION_TWO_HOSTS_SWITCH_TABLE），代码里认的开关取自 $two_hosts_script"
  table="$(crash_amplification_read_switch_table "$switch_table_source" CRASH_AMPLIFICATION_TWO_HOSTS_BREAK CRASH_AMPLIFICATION_ITEM_BREAK)"
  while IFS=$'\t' read -r assignment named; do
    [[ -n "$assignment" ]] || continue
    switch_wheel_assignments+=("$assignment"); switch_wheel_expected+=("$named")
    case "$assignment" in
      CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=*) in_table_two+="${assignment#*=}"$'\n' ;;
      *) in_table_item+="${assignment#*=}"$'\n' ;;
    esac
  done <<<"$table"
  listed_count="${#switch_wheel_assignments[@]}"
  if (( listed_count == 0 )); then
    echo "  ✗ 开关轮：文件头「弄坏开关」那张表一个开关都没读到" >&2
    echo "     → 怎么办：表里一行「#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=<开关名>  <弄坏哪一处>」，下一行起以「#       」开头写点名的格（文件头「弄坏开关」一段的写法）" >&2
    exit 1
  fi
  duplicated="$(printf '%s\n' "${switch_wheel_assignments[@]}" | LC_ALL=C sort | uniq -d | tr '\n' ' ')"
  if [[ -n "$duplicated" ]]; then
    echo "  ✗ 开关轮：文件头表里同一个开关列了不止一次：$duplicated" >&2
    echo "     → 怎么办：一个开关只写一行，点名的格都写在它下面" >&2
    round_failures=$((round_failures + 1))
  fi
  in_code="$(grep -oE 'two_hosts_break_is [a-z0-9-]+' "$two_hosts_script" | awk '{ print $2 }' | LC_ALL=C sort -u)"
  in_table_two="$(printf '%s' "$in_table_two" | LC_ALL=C sort -u)"
  if [[ "$in_code" != "$in_table_two" ]]; then
    echo "  ✗ 开关轮：代码里 two_hosts_break_is 认的开关与文件头表里列的双机开关不一样（只在代码里：$(LC_ALL=C comm -23 <(printf '%s\n' "$in_code") <(printf '%s\n' "$in_table_two") | tr '\n' ' ')；只在表里：$(LC_ALL=C comm -13 <(printf '%s\n' "$in_code") <(printf '%s\n' "$in_table_two") | tr '\n' ' ')）" >&2
    echo "     → 怎么办：代码里每一处 two_hosts_break_is 认的开关都在表里写一行（连同点名的格），表里每一行都在代码里有对应的一处" >&2
    round_failures=$((round_failures + 1))
  fi
  in_library="$(grep -oE 'crash_amplification_item_break_is [a-z0-9-]+' "$two_hosts_script_directory/crash-amplification-lib.sh" | awk '{ print $2 }' | LC_ALL=C sort -u)"
  in_table_item="$(printf '%s' "$in_table_item" | LC_ALL=C sort -u)"
  stray="$(LC_ALL=C comm -13 <(printf '%s\n' "$in_library") <(printf '%s\n' "$in_table_item") | sed '/^$/d' | tr '\n' ' ')"
  if [[ -n "$stray" ]]; then
    echo "  ✗ 开关轮：文件头表里列的共用库开关，共用库里不认：$stray" >&2
    echo "     → 怎么办：照 crash-amplification-lib.sh 里 crash_amplification_item_break_is 认的名字写，不认的那一行删掉" >&2
    round_failures=$((round_failures + 1))
  fi
  # 共用库认而表里没列的：也逐个打开，要退 0、一格不红（点名写「-」）
  for switch in $(LC_ALL=C comm -23 <(printf '%s\n' "$in_library") <(printf '%s\n' "$in_table_item")); do
    switch_wheel_assignments+=("CRASH_AMPLIFICATION_ITEM_BREAK=$switch"); switch_wheel_expected+=(-); unlisted_count=$((unlisted_count + 1))
  done
  crash_amplification_switch_wheel "$work" '✗ 自证 ([a-z0-9-]+)：' "$exit_only" -- bash "$two_hosts_script" --selftest
  round_failures=$((round_failures + switch_wheel_failures))
  if (( round_failures > 0 )); then
    echo "  ✗ --selftest：$cells 格都对，开关轮有 $round_failures 处对不上" >&2
    echo "     → 怎么办：红的格与表里点名的不同，要么改那一处判法让开关红在点名的格，要么照实改文件头那一行；单看一个开关：CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=<开关> bash $two_hosts_script --selftest" >&2
    exit 1
  fi
  echo "  ✓ --selftest：不在临时 prune-server 底下起的自证退 2；经 task-server 分片的三步（计划写两片、都不钉台、都不带内存下限，记下标签、源码摘要与按段登记、不拷库；各片只判自己那一份、"
  echo "    直接写进同一份服务，第 0 片先录再只判、第 1 片先录再判、这一台在 GPU_KERNEL_SERIAL_COMPILE_HOSTS 里的那一片设内核一个一个编（两片同一条规则）、认不出这一台退 78；合并读 TASK_SERVER_SHARD_OUTPUTS、拷各片日志、起一趟 SHARE=0/1 的核账、按此刻的键读到两片判的全部块（现判 0）、那一行带 merge_check_run=true；"
  echo "    产物表里第 0 片之外不是恰好一片退 1、核账那一趟现判了块或一行计数行都没打退 1、核账红原样退、源码中途变了退 77 而不起核账、观察者版本变了而红退 78、判段没跑那一行转打一次）、不分片时本机整份跑且一次不连第二台、"
  echo "    TASK_SERVER_SHARD_PHASE 认不出退 2、卡用不了与给定状态表借不到显存退 78、算法判不了退 248、--given-states 不起录入段、选段交过来是 judge 两片都只判、"
  echo "    --skip 每一趟都带、崩溃点重放样子的用例录入段真跑、服务交的卡照收（多机配置里的不读）且开了 GPU 带 --test-threads 1、"
  echo "    开了 GPU 没有服务交的卡退 78、上次红的清单、运行目录的归属记录、认不出的选项退 2、本机那一趟两步 cargo 直接起而线程照服务交的、开关轮的判法，$cells 格都对；"
  echo "    开关轮 ${#switch_wheel_assignments[@]} 个开关（文件头现读：双机 $(grep -c . <<<"$in_table_two") 个、共用库 $(( listed_count - $(grep -c . <<<"$in_table_two") )) 个；共用库另有 $unlisted_count 个表里没列、要它一格不红）逐个打开，派出 ${#switch_wheel_assignments[@]} 个、收回 $switch_wheel_collected 个，各红在点名的格"
  exit 0
}

# 计划（文件头「经 task-server 分片」plan）：这一步不编、不起 cargo、不交显卡，不经 two_hosts_prepare
two_hosts_plan() {
  local plan_directory="${TASK_SERVER_SHARD_PLAN_DIRECTORY:-}" digest registry_name shard_layout=two first_pin=""
  if [[ -z "$plan_directory" || ! -d "$plan_directory" ]]; then
    echo "  ✗ 计划那一步没给在的计划目录（TASK_SERVER_SHARD_PLAN_DIRECTORY=${plan_directory:-没设}），什么都没写" >&2
    echo "  → 怎么办：这一步只由 task-server 起；不分片就去掉 TASK_SERVER_SHARD_PHASE" >&2
    exit 2
  fi
  plan_directory="$(cd "$plan_directory" && pwd)"
  if ! printf '%s\n' "$label" >"$plan_directory/label"; then
    echo "  ✗ 计划目录 $plan_directory 写不进标签" >&2
    echo "  → 怎么办：看那个目录可不可写" >&2
    exit 1
  fi
  rm -f -- "$plan_directory/source-digest"
  if digest="$(crash_amplification_source_digest "$two_hosts_source_root" 2>&1)"; then
    printf '%s\n' "$digest" >"$plan_directory/source-digest"
    echo "  · 计划：记下被测源码摘要 ${digest%% *}（${digest#* }，$two_hosts_source_root）"
  else
    echo "  ! 计划：算不出被测源码摘要，合并之前不核源码变没变：$digest" >&2
  fi
  # 按段登记文件的快照（文件头「往引擎送的变量」）；弄坏开关 segments-registry-not-planned 下不拷
  rm -rf -- "${plan_directory:?}/segments-registry"
  if [[ -n "${SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY:-}" ]] && ! two_hosts_break_is segments-registry-not-planned; then
    registry_name="$(basename -- "$SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY")"
    if ! { mkdir -p -- "$plan_directory/segments-registry" && cp -- "$SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY" "$plan_directory/segments-registry/$registry_name"; }; then
      echo "  ✗ 拷不了按段登记文件 $SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY 进计划目录" >&2
      echo "  → 怎么办：看那份文件在不在、$plan_directory 可不可写" >&2
      exit 1
    fi
  fi
  # 两片、都不钉（文件头「经 task-server 分片」plan：片里没有 pin 键）；弄坏开关 plan-single-shard 下照改前只写一片，first-shard-pinned 下第 0 片照改前写 pin=local
  two_hosts_break_is plan-single-shard && shard_layout=single
  two_hosts_break_is first-shard-pinned && first_pin=local
  if ! python3 - "$plan_directory" "$shard_layout" "$first_pin" "$test_target${test_filter:+::$test_filter}" <<'PLAN_PYTHON'; then
import json, os, sys
directory, layout, first_pin, item = sys.argv[1:]
if layout == "single":
    shards = [{"label": f"{item} 整件"}]
else:
    shards = [{"label": f"{item} 第 0 片 SHARE=0/2，判的块写进 prune-server"},
              {"label": f"{item} 第 1 片 SHARE=1/2，判的块写进 prune-server"}]
if first_pin:
    shards[0]["pin"] = first_pin
temporary = os.path.join(directory, "plan.json.partial")
with open(temporary, "w", encoding="utf-8") as handle:
    json.dump({"shards": shards, "note": "崩溃放量分两片（crash-amplification-two-hosts.sh 文件头「经 task-server 分片」）"}, handle, ensure_ascii=False)
os.replace(temporary, os.path.join(directory, "plan.json"))
PLAN_PYTHON
    echo "  ✗ 计划目录 $plan_directory 写不进 plan.json（上面是 python 的报错）" >&2
    echo "  → 怎么办：看那个目录可不可写" >&2
    exit 1
  fi
  echo "  · 计划：plan.json 写 $([[ "$shard_layout" == single ]] && echo 一片 || echo 两片)（都不钉台、判的块都直接写进 prune-server，合并那一步核它们按此刻的键读得到；标签 $label）"
  exit 0
}

# 这一台在 GPU_KERNEL_SERIAL_COMPILE_HOSTS 里的那一片另设的环境（文件头「经 task-server 分片」shard）：判器内核一个一个编（用哪张卡由用例照多机配置判法 --emit-addresses 认这一台，不在这里设）
#   （变量名与 crates/singlefs-verification/src/crash_judge_dispatch.rs 的 GPU_KERNELS_ONE_AT_A_TIME_ENVIRONMENT_VARIABLE 相同，那边的测试读行首是 serial_compile_environment=( 的这一行）
serial_compile_environment=( SINGLEFS_GPU_KERNELS_ONE_AT_A_TIME=1 )

# 这一台编判器内核要不要一个一个编（文件头「经 task-server 分片」shard）：照多机配置判法 --emit-addresses 认这一台的地址（共用库 crash_amplification_host_addresses），
#   在 GPU_KERNEL_SERIAL_COMPILE_HOSTS 里就要，不看派到哪一台。退 0 要、1 不要；认不出这一台退 2（共用库把判法的原话与出路打在标准错误上）
two_hosts_serial_compile_here() {
  local serial_host
  crash_amplification_host_addresses "$(crash_amplification_multi_host_configuration_file "$repository_root")" || return 2
  for serial_host in $crash_amplification_serial_compile_hosts; do
    [[ "$serial_host" == "$crash_amplification_this_host_address" ]] && return 0
  done
  return 1
}

# 一片（文件头「经 task-server 分片」shard）：判的块直接写进 prune-server（文件头「判定库」），不建库、不拷库
two_hosts_shard() {
  local plan_directory="${TASK_SERVER_SHARD_PLAN_DIRECTORY:-}" output="${TASK_SERVER_SHARD_OUTPUT_DIRECTORY:-}" position="${TASK_SERVER_SHARD:-}"
  local index count share registry run_exit host_name serial_exit=0
  if [[ -z "$plan_directory" || ! -d "$plan_directory" || -z "$output" || ! "$position" =~ ^([0-9]+)/([1-9][0-9]*)$ ]]; then
    echo "  ✗ 这一片缺 task-server 给的变量（计划目录 ${plan_directory:-没设}、产物目录 ${output:-没设}、片号 ${position:-没设}），一趟都没起" >&2
    echo "  → 怎么办：这一步只由 task-server 起；不分片就去掉 TASK_SERVER_SHARD_PHASE" >&2
    exit 2
  fi
  index="${BASH_REMATCH[1]}" count="${BASH_REMATCH[2]}"
  if (( index >= count )); then
    echo "  ✗ 片号 $position 不对（片号要小于片数），一趟都没起" >&2
    echo "  → 怎么办：这个变量只由 task-server 设" >&2
    exit 2
  fi
  if ! mkdir -p -- "$output"; then
    echo "  ✗ 建不了这一片的产物目录 $output" >&2
    echo "  → 怎么办：看它的上层目录可不可写" >&2
    exit 1
  fi
  # 两个目录给的是相对路径的照这一刻的目录换成绝对路径（日志与观察者版本写进产物目录，按段登记从计划目录读）
  output="$(cd "$output" && pwd)" plan_directory="$(cd "$plan_directory" && pwd)"
  [[ -s "$plan_directory/label" ]] && label="$(head -n 1 -- "$plan_directory/label")"
  share="$index/$count"
  (( index != 0 )) && two_hosts_break_is same-share-on-both-shards && share="0/$count"
  # 按段登记照计划目录里那一份（文件头「往引擎送的变量」）
  for registry in "$plan_directory"/segments-registry/*; do
    [[ -f "$registry" ]] && export SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY="$registry"
  done
  # 这一台在 GPU_KERNEL_SERIAL_COMPILE_HOSTS 里就设内核一个一个编（文件头「经 task-server 分片」shard，环境在 serial_compile_environment 那一行）；
  #   认不出这一台退 78。弄坏开关 serial-compile-unjudged-runs 下认不出照一起编往下跑，serial-compile-on-every-host 下每一片都设，serial-compile-environment-unset 下都不设
  two_hosts_serial_compile_here || serial_exit=$?
  if (( serial_exit == 2 )) && ! two_hosts_break_is serial-compile-unjudged-runs; then
    echo "  · 第 $position 片认不出这一台，定不了判器内核一个一个编还是一起编，一趟都没起（退 78）" >&2
    exit 78
  fi
  host_name="这一台 ${crash_amplification_this_host_address:-认不出}，内核一起编"
  if (( serial_exit == 0 )) || two_hosts_break_is serial-compile-on-every-host; then
    if ! two_hosts_break_is serial-compile-environment-unset; then
      export "${serial_compile_environment[@]}"
      host_name="这一台 ${crash_amplification_this_host_address:-认不出}，内核一个一个编"
    fi
  fi
  two_hosts_prepare
  if (( index == 0 )); then
    if (( record_wanted )); then
      crash_amplification_record_on_this_host "$output/record.log" "0/$count" || exit $?
      two_hosts_note_observer_version "$output/record.log" "$output/observer-version"
      export SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge
    fi
  else
    # 单独起：第 1 片的观察者那一份只有它自己录（第 0 片只录自己那一份），不设选段、引擎先录再判；弄坏开关 other-shard-judges-only 下照改前只判
    if (( record_wanted )); then
      if two_hosts_break_is other-shard-judges-only; then export SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge; else unset SINGLEFS_CRASH_AMPLIFICATION_STAGE; fi
    fi
  fi
  echo "  · 第 $position 片（$host_name）：SHARE=$share，判的块写进 prune-server，日志 $output/share.log"
  if crash_amplification_run_on_this_host "$share" "$output/share.log"; then run_exit=0; else run_exit=$?; fi
  grep -hE '^CRASH_AMPLIFICATION(_GPU_CARD|_GPU_TOTAL)? ' "$output/share.log" 2>/dev/null
  if (( run_exit != 0 )) && ! two_hosts_break_is shard-failure-ignored; then
    two_hosts_refuse_failed_run "$run_exit" "$output/share.log" "第 $position 片（$host_name）那一趟"
  fi
  echo "  ✓ 第 $position 片判完自己那一份（SHARE=$share，判的块都写进 prune-server）"
  exit 0
}

# 合并时把各片产物目录里的日志与观察者版本拷进运行目录留着（文件头「经 task-server 分片」merge）；弄坏开关 shard-logs-not-kept 下不拷
two_hosts_keep_shard_logs() { # <片号> <产物目录>
  local kept="$run_directory/shard-$1" file
  two_hosts_break_is shard-logs-not-kept && return 0
  mkdir -p -- "$kept" || return 0
  for file in "$2"/*.log "$2/observer-version"; do
    [[ -f "$file" ]] && cp -- "$file" "$kept/"
  done
  return 0
}

# 核账那一趟现判了几块（文件头「经 task-server 分片」merge）：打「<现判的块数> <计数行的行数>」——CRASH_AMPLIFICATION 行的 blocks_checked_here= 与
#   崩溃枚举用例 CRASH_CASE_BLOCKS 行的 fresh= 加起来；计数行一行都没有时行数是 0（判不了读没读到）
two_hosts_blocks_judged_in_the_check() { # <核账那一趟的日志>
  awk '
    $1 == "CRASH_AMPLIFICATION" || $1 == "CRASH_CASE_BLOCKS" {
      lines++
      for (i = 2; i <= NF; i++) {
        split($i, pair, "=")
        if (($1 == "CRASH_AMPLIFICATION" && pair[1] == "blocks_checked_here") || ($1 == "CRASH_CASE_BLOCKS" && pair[1] == "fresh")) judged += pair[2]
      }
    }
    END { printf "%d %d\n", judged, lines }' "$1" 2>/dev/null || echo "0 0"
}

# 合并（文件头「经 task-server 分片」merge）：两片判的块都已在 prune-server 里，这里只核它们按此刻的键读得到；不拷库、不导入。弄坏开关 merge-does-nothing 下照改前什么都不做
two_hosts_merge() {
  local plan_directory="${TASK_SERVER_SHARD_PLAN_DIRECTORY:-}" outputs="${TASK_SERVER_SHARD_OUTPUTS:-}" index host directory
  local first_directory="" first_host="" other_host="" other_count=0 check_exit judged_blocks counted_lines
  if two_hosts_break_is merge-does-nothing; then echo "  · 合并：没有要合的"; exit 0; fi
  if [[ -z "$outputs" || ! -f "$outputs" || -z "$plan_directory" || ! -d "$plan_directory" ]]; then
    echo "  ✗ 合并那一步缺 task-server 给的变量（各片产物表 ${outputs:-没设}、计划目录 ${plan_directory:-没设}），一趟都没起" >&2
    echo "  → 怎么办：这一步只由 task-server 起；不分片就去掉 TASK_SERVER_SHARD_PHASE" >&2
    exit 2
  fi
  [[ -s "$plan_directory/label" ]] && label="$(head -n 1 -- "$plan_directory/label")"
  run_directory="$crash_amplification_home/runs/$label"
  two_hosts_open_run_directory "$run_directory"
  while IFS=$'\t' read -r index host directory; do
    [[ -n "$index" ]] || continue
    if [[ "$index" == 0 ]]; then first_directory="$directory" first_host="$host"; else other_host="$host"; other_count=$((other_count + 1)); fi
    two_hosts_keep_shard_logs "$index" "$directory"
  done <"$outputs"
  # 产物表里没有第 0 片、或除第 0 片之外不是恰好一片（文件头「退出码」1）
  if [[ -z "$first_directory" ]] || (( other_count != 1 )); then
    echo "  ✗ 各片产物表 $outputs 里$([[ -z "$first_directory" ]] && echo "没有第 0 片那一行" || echo "除第 0 片之外有 $other_count 片")（计划写的是两片），核账那一趟没起" >&2
    echo "  → 怎么办：看计划那一步写的 plan.json 与服务的分片记录；重投这一件" >&2
    exit 1
  fi
  two_hosts_prepare
  # 核账那一趟只判（第 0 片录过；文件头「经 task-server 分片」merge）；--given-states 不设
  (( record_wanted )) && export SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge
  # 源码中途变了（文件头「源码中途变了」）：核账那一趟不起
  if two_hosts_source_changed "$plan_directory/source-digest"; then
    echo "CRASH_AMPLIFICATION_NOT_RUN step=merge test=crash_amplification::run_from_environment reason=跑的过程中源码变了，整件重跑（计划时 ${source_digest_recorded%% *}，核账之前 ${source_digest_now%% *}）"
    echo "  ! 本次未跑（这不是通过）：跑的过程中源码变了，整件重跑：计划时被测源码摘要 ${source_digest_recorded}，核账之前 ${source_digest_now}（$two_hosts_source_root 下 Cargo.toml、Cargo.lock、.cargo/、crates/）；核账那一趟没起，两片判完的块已经在 prune-server 里" >&2
    echo "  → 怎么办：等树里的改动停下（别的会话的补丁打完）再重投同一条命令，判过的块键没变的照服务里的复用" >&2
    exit 77
  fi
  if crash_amplification_run_on_this_host 0/1 "$run_directory/merge.log"; then check_exit=0; else check_exit=$?; fi
  two_hosts_print_merge_lines "$run_directory/merge.log"
  # 观察者版本（文件头「观察者版本」）：核账那一趟用的二进制和第 0 片录入段之后记下的对不上：红了不是漏录，退 78 叫人重跑整件；判绿了只说一声
  if two_hosts_observer_version_changed "$first_directory/observer-version" "$run_directory/merge.log"; then
    if (( check_exit != 0 )); then
      echo "  ✗ 录入段之后到核账那一趟之间，测试二进制里编进去的判器单元变了：观察者版本录入段 ${observer_version_recorded%% *}，核账那一趟 ${observer_version_now%% *}；核账那一趟退 $check_exit，读不到录入段按旧版本录的块，不是录入段漏录" >&2
      echo "  → 怎么办：两趟之间判器单元变了，重跑整件（同一条命令，从计划起，别只补录入段）；树在中途被改过的，等改动停下再投" >&2
      exit 78
    fi
    echo "  ! 录入段之后到核账那一趟之间观察者版本变了（录入段 ${observer_version_recorded%% *}，核账那一趟 ${observer_version_now%% *}），核账那一趟照样对账为空、0 红：结论按核账那一趟的二进制算" >&2
  fi
  if (( check_exit != 0 )) && ! two_hosts_break_is merge-failure-ignored; then
    two_hosts_refuse_failed_run "$check_exit" "$run_directory/merge.log" "核账那一趟（SHARE=0/1，读两片写进 prune-server 的块；对账不为空或有红）"
  fi
  # 两片判的块按此刻的键读得到（文件头「经 task-server 分片」merge）；弄坏开关 merge-blocks-unchecked 下不核
  if ! two_hosts_break_is merge-blocks-unchecked; then
    read -r judged_blocks counted_lines <<<"$(two_hosts_blocks_judged_in_the_check "$run_directory/merge.log")"
    if (( counted_lines == 0 )); then
      echo "  ✗ 核账那一趟退 $check_exit，日志 $run_directory/merge.log 里却一行计数行都没打（CRASH_AMPLIFICATION 与 CRASH_CASE_BLOCKS 都没有）：判不了两片判的块读没读到" >&2
      echo "  → 怎么办：看那份日志跑的是不是这一条用例、用例有没有提早返回；修好之后重投同一条命令" >&2
      exit 1
    fi
    if (( judged_blocks > 0 )); then
      echo "  ✗ 核账那一趟现判了 $judged_blocks 块：两片判的块里有按此刻的键在 prune-server 里读不到的（它们写进了别的库：两片连的不是同一份服务，或判的不是这一棵树）；第 0 片在 $first_host（日志 $run_directory/shard-0/），第 1 片在 $other_host（日志 $run_directory/shard-1/）" >&2
      echo "  → 怎么办：看两片日志里客户端连的是哪一处（本机 unix socket，第二台多机配置的 PRUNE_SERVER_TCP_ADDRESS）、环境里有没有改连哪一处的变量；读不到的块这一趟已经判进服务，修好之后重投同一条命令" >&2
      exit 1
    fi
  fi
  echo "  ✓ 两片各判各的那一份、都写进 prune-server；核账那一趟按此刻的键读到两片判的全部块（现判 0 块）、对账为空、0 红（日志在 $run_directory）"
  exit 0
}

# 不经 task-server 分片（文件头「不经 task-server 分片」）：本机整份
two_hosts_single() {
  local run_exit
  run_directory="$crash_amplification_home/runs/$label"
  two_hosts_open_run_directory "$run_directory"
  two_hosts_prepare
  if (( record_wanted )); then
    crash_amplification_record_on_this_host "$run_directory/record.log" || exit $?
    export SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge
  fi
  if crash_amplification_run_on_this_host 0/1 "$run_directory/single.log"; then run_exit=0; else run_exit=$?; fi
  grep -hE '^CRASH_AMPLIFICATION(_GPU_CARD|_GPU_TOTAL)? ' "$run_directory/single.log" 2>/dev/null
  two_hosts_break_is judge-not-run-dropped || grep -h '^CRASH_AMPLIFICATION_NOT_RUN step=judge ' "$run_directory/single.log" 2>/dev/null
  if (( run_exit != 0 )); then two_hosts_refuse_failed_run "$run_exit" "$run_directory/single.log" "本机整份那一趟"; fi
  echo "  ✓ 本机整份判完、对账为空、0 红（日志里 $(grep -cE '^CRASH_AMPLIFICATION ' "$run_directory/single.log" 2>/dev/null) 行计数行；没经 task-server 分片，一台跑；日志在 $run_directory）"
  exit 0
}

# 参数解析之前：弄坏开关只在自证里认。平常跑设了就退 2 并点名；自证起的子进程与命令行第一个参数就是 --selftest 的（开关轮起的那一趟）放过
if (( ! two_hosts_under_selftest )) && [[ "${1:-}" != --selftest ]]; then
  for break_variable in CRASH_AMPLIFICATION_TWO_HOSTS_BREAK CRASH_AMPLIFICATION_ITEM_BREAK; do
    if [[ -n "${!break_variable:-}" ]]; then
      echo "  ✗ $break_variable 只在自证里认（弄坏开关只给 --selftest 证红用），这一趟不是自证，一步都没做" >&2
      echo "     → 怎么办：去掉 $break_variable 再跑；要证红就跑 $break_variable=<项> bash $0 --selftest" >&2
      exit 2
    fi
  done
fi
test_target=""; label="crash-amplification-$(date -u +%Y%m%dT%H%M%SZ)-$$"; selftest=0; threads=""; test_filter=""; given_states=0; skip_names=()
while (($# > 0)); do
  case "$1" in
    --test-filter) require_value "$1" $# "${2:-}"; test_filter="$2"; shift 2 ;;
    --label) require_value "$1" $# "${2:-}"; label="$2"; shift 2 ;;
    --skip) require_value "$1" $# "${2:-}"; skip_names+=("$2"); shift 2 ;;
    --given-states) given_states=1; shift ;;
    --selftest) selftest=1; shift ;;
    -*) usage "$1 不认得" ;;
    *) [[ -z "$test_target" ]] || usage "测试目标给了不止一个"; test_target="$1"; shift ;;
  esac
done
if (( selftest )); then run_selftest; fi
[[ -n "$test_target" ]] || usage "缺测试目标"

# 走哪一步（文件头「经 task-server 分片」「不经 task-server 分片」）；弄坏开关 unknown-phase-runs-single 下认不出的当没设
phase="${TASK_SERVER_SHARD_PHASE:-}"
case "$phase" in
  plan|shard|merge|"") ;;
  *)
    if two_hosts_break_is unknown-phase-runs-single; then
      phase=""
    else
      echo "  ✗ TASK_SERVER_SHARD_PHASE=$phase 认不出（只认 plan、shard、merge），一趟都没起" >&2
      echo "  → 怎么办：这个变量只由 task-server 设；去掉它就是本机整份跑" >&2
      exit 2
    fi ;;
esac
crash_amplification_home="$(crash_amplification_home_directory)" || exit 2
# 两段（用户 2026-10-11 定）：单独起（选段没设成 judge）先录入段、之后的判段只从判定库（prune-server）里读；选段交过来就是 judge 的（调用方整份录过）只判；带 --given-states 的引擎自己先录再判。
#   弄坏开关 record-skipped-when-not-judging 下单独起也不录；record-for-given-states 下带 --given-states 也录
record_wanted=0
if [[ "${SINGLEFS_CRASH_AMPLIFICATION_STAGE:-}" != judge ]] && { (( ! given_states )) || two_hosts_break_is record-for-given-states; } \
   && ! two_hosts_break_is record-skipped-when-not-judging; then
  record_wanted=1
fi
run_directory=""
# 退出时（每一种退法）写上次红的清单（文件头「上次红的清单」）；计划那一步与各片不写。弄坏开关 last-run-unwritten 下不写
two_hosts_on_exit() { # <这一趟的退出码>
  [[ "$phase" == plan || "$phase" == shard ]] && return 0
  two_hosts_break_is last-run-unwritten && return 0
  python3 "$repository_root/scripts/gate/lib_reuse_checks.py" last-run-target scripts/crash/crash-amplification-two-hosts.sh "$test_target" "$1" \
    ${test_filter:+--test-filter "$test_filter"} --target-file "$repository_root/crates/singlefs-verification/tests/$test_target.rs" >/dev/null 2>&1 \
    || echo "  ! 上次红的清单没写成（scripts/gate/lib_reuse_checks.py last-run-target），这一趟的判定不受影响" >&2
}
trap 'two_hosts_on_exit "$?"' EXIT
trap 'exit 143' TERM
trap 'exit 130' INT
case "$phase" in
  plan) two_hosts_plan ;;
  shard) two_hosts_shard ;;
  merge) two_hosts_merge ;;
  *) two_hosts_single ;;
esac
