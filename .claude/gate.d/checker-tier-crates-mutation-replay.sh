#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 要的工具与设备登记在 stage-inputs.tsv 本阶段那一行第三列，由 scripts/gate/admission.py gate-preconditions 在阶段里判，没齐判红，不交给 gate.sh 预判
# gate-stage: crates 变异表复跑（每条改坏一处、点名的测试必须红）
# gate-fixture: yes
# gate-category: checker-tier 类
# gate-similar: code-source-discipline.sh 它的实验与变异源码纪律那一格经同一份判法（scripts/mutation/crates-mutation-rows.py static-check）静态判变异表成形与拷贝范围，不改坏、不起 cargo；这里真改坏每一处、真跑点名的测试，是重型，两边共用判法模块、不共用阶段
# gate-similar: checker-tier-research-build-and-replay.sh 它的复跑那一格也按输入复用、也逐条跑编出来的代码，但跑的是 research 的实验二进制、比的是产物；这里改坏的是 crates 的源码、判的是点名的测试红不红
#
# gate-cell: crates-mutation-rows crates/mutations.tsv 每条改坏一处、点名的测试都红，基线在没改坏的副本上绿
# 参数解析、格汇总与退出码经共用库 lib/stage-cells.sh（写法在它的文件头）。
#   bash .claude/gate.d/checker-tier-crates-mutation-replay.sh [项目根]                                整张表
#   bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --list                                  逐行打格名与判什么，不起 cargo
#   bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --check crates-mutation-rows [项目根]     只跑这一格（格名写错退 2）
#   bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --list-items [项目根]                    逐行打变异表里可点名的变异名、行号与改哪个文件，不起 cargo；末了报这一趟键对不上（要现判）的条数
#                                                                                                      与锚点失效的行（逐行点名），再一行报牵连面（打的是 crates-mutation-rows.py --list-items，同一份）
#   bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --item <行号或变异名> [--item …] [项目根]   只判点名的几行（crates-mutation-rows.py --item 交到这里）
#   bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --quick [--item <行号或变异名> …] [项目根]   轻型：只判轻型变异登记表的行，再加点名的几行
#   --quick（轻型，.claude/rules/verification.md「两档：轻型与重型」）：核心部分是 .claude/gate.d/crates-mutation-light-rows.tsv 登记的行（改坏对象在 crates/singlefs-core/src/ 下的行里每个测试目标一行，
#     写法与挑法在它的表头），先经 scripts/mutation/crates-mutation-light-rows.py names 核那张表（核不过退 1、一行都不判），再连同 --item 点名的几行
#     （可变部分，取并集）照下面 --item 那一路判；轻型与整张表同一份判定库、同一套单元键复用（键对得上的复用、对不上的现判），不另设只写不读。
#     样本根放 .mutation-sample/quick 当成给了 --quick（样本没法带参数；被判的根是这道阶段所在的仓或设了 GATE_STAGED_FROM 时不认）。
#   --item：写变异表里成形的一行的行号，或变异名原样（名字里的逗号不拆、同名的几行都判）；不在表里列出可点名的、退 2。换成行号走共用模块的选行模式
#     （不调双机驱动，照常写判定库记录、照复用判据复用：点名单跑与整张跑时那一行的判定相同）。
#     选中的全抓到这一格绿（模块退 3，这里换成 0，成功行写明只判了这几行）；有本次未跑的，成功行写明抓到几条、本次未跑几条；全是本次未跑的退 77。
#   GATE_MUTATION_ROW_SELECTION=<文件>（一行一个变异表行号，只由双机分片的驱动设）：只判这些行（它们所在的基线批照跑），照常写判定库记录，
#     末行写明「只判了选中的 N 行，不是整道结论」；不经格汇总，选中的全抓到退 3、有红退 1
#     （汇总把 0、77 之外的码并成 1，驱动就分不出这两样）。
#
# 判据（判法、派活、判定库与覆盖都在 scripts/mutation/crates-mutation-rows.py；拷贝范围、判法单元、行键、判定库放哪、复用判据、弄坏开关见它的文件头）：
#   crates/mutations.tsv 每行六段（变异名、文件、原文、替换文、cargo test 参数、必须红的测试名），替换文可以空（删掉原文），其余五段不许空；
#   必须红的测试名是一个用例名，只有 \n 一种反斜杠转义，文件落在每片拷的范围（stage-inputs.tsv 本阶段那一行）之内：
#   这几样派活之前判（row_problems，门禁 code-source-discipline 经 static-check 静态判同一份），有一行不过整道判红、一个工作进程都不起；
#   原文在文件里恰好命中一次：派活之前按行判，不是的那一行记「锚点失效」（不跑、不入库、不复用，「计数：」里「锚点失效 N 行」一栏），别的行照判，整道判红
#   （共用模块文件头「锚点失效」；--item、--quick 与双机分片同样按行）；
#   点名的测试在没改坏的副本上恰好对上一个用例、结果是 ok（基线）；改坏那一处之后它必须红。一条不过整道红。
# 牵连面（只报数、不判红，共用模块文件头「牵连面」）：成功句之后与 --list-items 末了各一行「牵连最大的函数单元：… 进了 N 行的闭包（前 5：…）；一次改动最坏牵连 N 行」。
# 为什么：show-me-test.md「新增的测试必须先证明它会红」要变异清单进仓、交给门禁反复复跑；每处三方打中之后的改法留一条「改回去它就红」的变异，撤回时这里先响。
#
# 并发（派活与进程数的判法在 crates-mutation-rows.py 的 worker_count_for 与 dispatch_order）：每片各一份源码副本与一个编译目录（共用一份会被 cargo 的文件锁重新串行化）；
#   按 crate 归堆派（增量编译的命中率才稳），活动态领（等分之下拿到几十秒重测试的那一片会独自拖住整道），上一次用时 ≥ GATE_MUTATION_LONG_ROW_SECONDS（默认 300）秒或超时的先派。
#   工作进程数取 ① 核数的一半、至多 16（设了 GATE_MUTATION_WORKERS 就顶掉 ①）、② 这一趟要现跑的条数、③ 外层内存上限 ÷ 每条变异的默认上限（向下取整；
#   默认上限是没有逐行上限的那些行用的那个值，有一行写了 24G 不再把整趟压成 1 个）、④ 编译目录所在的盘放得下几个工作进程的两份编译目录（下面「磁盘余量」）的小者、至少 1。
#   ③ 的外层上限读环境变量 RUN_WITH_MEMORY_CAP_OUTER_CAP（run-with-memory-cap.sh 套在外层时给命令设的，例 `run-with-memory-cap.sh 24G bash 本阶段 --quick`）：
#   套在外层里的里层包装不排队，合起来超过外层上限时只有 slice 的总上限兜着，所以按外层收；没设或写法不认就是没套在外层里，没有 ③、
#   每条自己经包装排队（开跑那一刻看不见别的重活来抢，不按内存收），stderr 头一行写明「外层内存上限没读到」。外层不够一条的上限时按 1 个开。判法在 worker_count_for。
#   按额度排队（外层读得到时；读不到时整段不起作用，派活与不排队时一样）：派的每一格（基线的批、一条变异、一组覆盖）按它自己跑起来经包装给的上限（逐行上限写了取它，
#   没写取默认上限）从外层额度里扣，在跑的各格上限之和不超过外层上限；扣不下的那一格留在队里，后面扣得下的小格可以先派，一格被越过的次数到了工作进程数就停下、
#   等在跑的收完（不许饿死大格）；什么都没在跑时队首一定派，上限比整个外层还大的那一格单独起，由里层包装按用法错退 251 报出来。头一行写「逐行上限最大 X、按额度排队」。
#   判法在 crates-mutation-rows.py 的 pick_ready_job 与 run_pool，文件头「工作进程数」；自证与弄坏开关 largestrowmemory、rowmemoryunbudgeted、bypassunbounded 在它的 --selftest
#   （「外层 36G、一行 24G 加二十行 4G」那一格）；本道样本不判这一条。
#   线程数按工作进程数压：每个 cargo 的编译并行度 CARGO_BUILD_JOBS 取 max(1, 核数 ÷ 工作进程数)（GATE_MUTATION_CARGO_JOBS 顶掉它）。
#   先编那一趟（cargo test … --no-run：基线、变异的先编与插桩编译）的并行度另算：外层读不到时只按这一行的内存上限收：上限 ÷ 1.75 GiB（向下取整、至少 1）与上一句算出的取小者，
#   每条 4G 时至多 2、8G 至多 4（crates-mutation-rows.py 的 compile_jobs_for，系数 COMPILE_GIB_PER_JOB 是推的、只有一个读数；GATE_MUTATION_CARGO_JOBS 设了不收）；跑测试那一趟不收。
#   外层读得到时先编改从外层额度里此刻空着的那一部分算（compile_plan、OuterBudget）：想要的并行度（核数 ÷ 工作进程数）× 1.75 GiB 减去这一格自己的上限，向上面按额度排队的那本账借，
#   空着的 = 外层 − 在跑各格的上限之和 − 别的先编已借走的 − 给每个空着的工作进程接下一格留的默认上限，几个工作进程同时要时各分一份；借到多少，这一趟先编的包装内存上限就是
#   自己的上限加借到的，并行度随之升高（不低于只按自己上限收的，至多想要的那个数），编完就还；在跑各格的上限加借走的之和不超过外层。例：外层 36G、2 个工作进程在跑各 4G、核数 32，
#   每个先编借 14G，包装上限 18432M，并行度 10（只按自己的 4G 收是 2）；9 个工作进程各 4G 时额度已满、借不到，仍是 2。末了 stderr 报一行「额度账：…峰值」。
#   自证与弄坏开关 compilejobsownlimit（先编只按自己的上限收）在 crates-mutation-rows.py --selftest（「先编向外层额度借」等四格）；本道样本不判这一条。
#   磁盘余量（工作进程数的 ④，crates-mutation-rows.py 的 disk_bound_for）：每个工作进程最多占两份编译目录（普通、插桩），一份涨满按下面「编译目录上限」算；
#   派活之前按目标盘余量（外层给了 GATE_MUTATION_DISK_FREE 用它，不然读编译目录所在的盘）核「片号 1…n 各两份还能涨的量 + 备用 GATE_MUTATION_DISK_RESERVE（默认 30G）」，
#   放不下的 n 不开，头一行写磁盘卡在哪一项；余量没读到（两样都读不到）没有这一项，头一行写明原因。自证与弄坏开关 disknotchecked 在 crates-mutation-rows.py --selftest
#   （「磁盘余量放不下 9 份」等四格）；本道样本不判这一条。
#   编译目录上限（GATE_MUTATION_TARGET_DIR_CAP，默认 64G，trim_target_directory）：每个工作进程每判完一格量一次自己那份编译目录（判基线与变异的普通那份、判覆盖的插桩那份）的占盘，
#   超过上限就在两格之间清掉底下的增量产物（<目录>/<档>/incremental），依赖与测试二进制不动，清完仍超线的只报；弄坏开关 targetcapignored，自证同在 crates-mutation-rows.py --selftest
#   （「编译目录超过上限」等三格）；本道样本不判这一条。
#   被测试自己起的线程不压：RUST_TEST_THREADS 与各测试读的线程数变量调用方设了什么就原样透传给 cargo test。
#   ⚠️ stdout 里的判定与进程数无关：按变异表的行号排序再打，GATE_MUTATION_WORKERS=1 与 =8 逐字相同（并发之后重新证红的那一半）；进度、前移只进 stderr；
#   复用了多少进 stdout 的「复用：」「覆盖：」两行、抓到那一行末尾的「（现判：<栏>）」「（复用）」或「（拒算）」与成功句末尾「；这一趟现判 …」，比判定时去掉这几处。
#
# 编译目录：GATE_MUTATION_TARGET_DIR 设了用它；不然 GATE_CROSS_RUN_TMPDIR 设了放 $GATE_CROSS_RUN_TMPDIR/singlefs-crates-mutation-target；
#   都没设放 ${SINGLEFS_CROSS_RUN_CACHE_HOME:-~/.local/share/singlefs/cross-run-cache}/singlefs-crates-mutation-target，不落 /tmp
#   （只在 crates-mutation-rows.py 的 default_shared_target_directory 一处算，这里不另算）。分片各加 -w<片号>、各自冷编一次（prepare_shard），记覆盖的再加 -coverage。
#   片号跨调用经锁领（claim_worker_slot，C456）：共用编译目录那一层里每个片号一把锁文件（<共用目录>.slot 是第 1 片，其余 <共用目录>-w<片号>.slot），工作进程启动时从 1 往上
#   试拿排他 flock（不阻塞），拿到第一个空闲的就是它的片号，锁拿到进程退出；同一台机器上并发的两次调用（普通 gate.sh 与 --staged 的临时 worktree）因此拿到不相交的片号集合，
#   不再两边的第 2 片都指向同一个 -w2、被 cargo 的目录锁互相排队。缓存目录仍按片号跨轮复用（不按仓路径或会话号分目录：--staged 的临时 worktree 每趟都会冷编）。
#   放 GATE_CROSS_RUN_TMPDIR 不放 TMPDIR：门禁每轮给阶段的私有 TMPDIR 跑完就删，放进去每轮都要从头编。
#   编译目录数按这一趟的工作进程数复用：派活之前，片号大于这一趟工作进程数的 -w<片号>（连同 -coverage）回收，stderr 打一行删了几份、共多大；
#   有进程的工作目录、命令行、CARGO_TARGET_DIR 指到它、或它那一片的在用标记与片号锁被别的进程拿着的不删，逐份写明谁在用（crates-mutation-rows.py 文件头「编译目录」）。
#   回收与默认位置的自证在 crates-mutation-rows.py --selftest，弄坏开关 GATE_MUTATION_BREAK=extratargetskept、inusetargetremoved、targetdefaulttmp（本道样本不判这一条）。
#   片号锁的自证同在那里：弄坏开关 GATE_MUTATION_BREAK=local-shard-numbering（两次并发调用领到的片号重叠）那一格必须判错
#   （直接起自证时同义名 CRATES_MUTATION_ROWS_BREAK）；本道样本不判这一条。
# 崩溃判定库：变异版上走判定库的 checker 档用例不写真跑的长期库。起 cargo 时 SINGLEFS_CRASH_AMPLIFICATION_HOME 换成 GATE_MUTATION_CRASH_LIBRARY_HOME（设了、非空），
#   没设取真跑家目录（${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}）底下的 mutation-replay/（跨轮留着），调用方点名的单份库
#   SINGLEFS_CRASH_AMPLIFICATION_LIBRARY 清掉；只在 crates-mutation-rows.py 的 mutation_crash_library_environment 一处算（它文件头「变异复跑的判定库」），这里照原样
#   传调用方的环境，双机分片的驱动给第二台设 mutation-kept 下一份。自证在 crates-mutation-rows.py --selftest（弄坏开关 GATE_MUTATION_BREAK=sharedcrashlibrary）与
#   mutation-shard-run.sh --selftest（MUTATION_SHARD_RUN_BREAK=peer-crash-library-home-unset）；本道样本不判这一条。
# 崩溃判定库每片各一份：每个工作进程再用上面那个家目录底下的 shard-<片号>（crates-mutation-rows.py 的 crash_library_home_for_shard，片号经锁领、并发的几次调用不相交），
#   几个工作进程不共用一份库，开库等锁（crash_library.rs 的 DEFAULT_LOCK_WAIT_SECONDS，1800 秒）不与测试限时叠在一起；弄坏开关 GATE_MUTATION_BREAK=crashlibraryacrossshards，
#   自证在 crates-mutation-rows.py --selftest（「崩溃判定库每片各一份」「两个工作进程同时在跑」两格）；本道样本不判这一条。
# 临时目录：每条变异与每批基线各一个 TMPDIR，结束时不论退出码（含限时、撞顶被杀）都整个删掉：被杀的测试 Drop 守卫不跑，镜像会堆在里面；
#   全部判完总目录下还剩哪一条的目录，整道判红。
#
# 内存：每条 cargo test 经 scripts/hosts/run-with-memory-cap.sh（systemd 临时 scope，MemoryMax=<上限>、MemorySwapMax=0）跑，撞上限只杀这一条。
#   上限先到先用：变异表里单起一行「# 每条变异的内存上限：<上限>」、GATE_MUTATION_MEMORY_MAX、默认 4G。
#   逐行上限（判法在 scripts/mutation/crates-mutation-rows.py 文件头「逐行上限」）：核心部分的行装不下这一个数的，在 .claude/gate.d/crates-mutation-light-rows.tsv 里
#   单起一行「# 逐行上限：<变异名><TAB><内存上限或 -><TAB><限时秒或 ->[<TAB># 依据]」，那一行（连同它所在的那一批基线）拿它自己的上限与限时，别的行照上面那一个数；
#   取法定成逐行写，不按 scripts/hosts/host-resources.py 的峰值表现取（峰值表里撞过顶的键记的是那一次的上限）。上限与限时都不进行键。
#   默认 4G 的依据：本机 60 GiB、常驻约 12 GiB、同时可能 3 件重活，每件 16 GiB 里再分给几个工作进程；四个 crate 全部测试目标冷编的匿名内存峰值 0.60 GiB。
#   点名的测试跑起来要多少没量，按条记在 ~/.local/share/singlefs/memory-peaks.tsv。这些数只在本机成立，换机器要重算。
#   包装退 250 记「内存撞顶」，254 记「被总上限挤掉」，252 记「排不上没跑」，245 记「显存不够」（起止之间内核报了 NV_ERR_NO_MEMORY，
#   输出里的 FAILED 是被它弄红的）：都不算抓到也不算没红，单列、整道判红。分栏在 scripts/mutation/crates-mutation-rows.py 的 verdict_of_run
#   （245 那一栏的自证是它 --selftest 里「包装退 245（显存不够）记显存不够、不算抓到」那一格，弄坏开关 GATE_MUTATION_BREAK=gpumemoryasoutput）。
#   退 250 再分两栏（verdict_of_run、baseline_problem）：先编那一趟撞的单列「编译撞顶」（出路写编译并行度与逐行上限，不写「无界分配」，点名的测试一次都没起），
#   跑测试那一趟撞的照旧；「计数：」那一行两栏都算内存撞顶，格式不动；基线不绿的句子写明「那一批编译那一趟撞了内存上限」。弄坏开关 GATE_MUTATION_BREAK=memorycompilemerged，
#   自证在 crates-mutation-rows.py --selftest（「先编那一趟撞了内存上限」「测试那一趟撞了内存上限」两格）；本道样本不判这一条。
#   scope 或 slice 的总上限设不上（没有用户级 systemd、D-Bus 连不上）整道判红、一条都不跑，不退回无上限去跑。
# 限时：交给包装（RUN_WITH_MEMORY_CAP_TIME_LIMIT → scope 的 RuntimeMaxSec，从起跑算、不算排队）；不在包装外面套 timeout：排队的时间会被算成超时。
#   不含重编译：基线的一批与每条变异先在同一个内存上限里不限时跑一趟 cargo test <-- 之前那一段> --no-run，编成了再限时跑点名的测试；插桩编译同样不限时
#   （判法在 crates-mutation-rows.py 的 run_under_wrapper；弄坏开关 GATE_MUTATION_BREAK=timelimitincludescompile 照改前限时含重编译，红在它自证的「逐行上限」那一格）。
#   秒数先到先用：变异表里单起一行「# 每条变异的超时秒数：<秒>」、GATE_MUTATION_TIMEOUT、默认 1800。包装退 253 记「超时」：单列一栏，整道判红。
#   默认 1800 的依据：点名随机历史测试的变异在 16 个工作进程并发时一条超过 6–8 分钟（C457（被测试自己的线程池与外层进程池叠加超订阅）），约 4 倍留余地；
#   代价是一条真挂死的变异让一个工作进程多占 30 分钟。本机的数，换机器要重算。
#
# 基线（判法见 crates-mutation-rows.py 文件头「基线」）：点名的测试在没改坏的副本上本来就红时，改坏之后照样红、记「抓到」什么都没证明，
#   所以派变异之前先跑基线，不是恰好一个 ok 的行记「基线不绿」、整道判红。「计数：」那一行的格式不动（mutation-triage 照栏读），基线另起一行「基线：…」报。
# 复用：每行一条判定，行键是被改坏的函数单元指纹、原文、替换文、点名测试执行过的单元集合指纹、判「抓没抓到」的判法单元指纹、工具链、解析出的依赖版本，
#   不带变异名、路径、内存上限、限时；没有管整库的版本号、没有版本目录；一份库，第二台判出的经 crates-mutation-rows.py import 逐行核键并进本机那一份。
#   没有整道全绿标记，不问 stage-must-run.sh。点名测试那一段这里取它的单元闭包键（入口 = 那条测试函数，比执行过的单元宽，不漏）；
#   执行过的单元另记在覆盖记录里（-C instrument-coverage、llvm-profdata、llvm-cov，工具不对整道判红），基线 ok 那一条按它复用、不按整树快照。
#   键对不上的现判，按栏报数（行新加、判法单元变了、被改坏的单元变了、点名测试的闭包变了、原文或替换文变了、工具链或依赖变了、上次没判成抓到、一律现判），
#   stderr 逐行写变了哪几个单元；算不出键的那一行拒算（点名的测试认不出单元、闭包里有认不出的形态），每趟照跑。成功句末尾报「现判 N、复用 M、拒算 R、本次未跑 K」。
#   键、库的存放、复用判据写在 crates-mutation-rows.py 文件头「复用」「判法单元」「行键」「判定库」「复用判据」。只读看这一趟会怎么判：python3 scripts/mutation/crates-mutation-rows.py plan <根>；
#   --list-items 末行报键对不上的条数。只有「抓到」与基线 ok 复用；SINGLEFS_GATE_FULL=1 或 GATE_MUTATION_START_OVER=1 只写不读（现判的照键写回同一份库）。
# 双机分片：只在本地多机配置写了 ENABLE_ACROSS_MACHINES=1 时开（判法 scripts/crash/crash-point-replay-shard-configuration-check.sh：退 0 开、退 3 关、退 1 判不过），不认环境变量开。
#   整道跑、没设 SINGLEFS_GATE_FULL=1 与 GATE_MUTATION_START_OVER=1（这两样下最后那一趟不复用，分出去的白跑）、判法退 0 时，
#   先调 scripts/mutation/mutation-shard-run.sh 把没有作数记录的行分到两台各判一份（步骤与弄坏开关在它文件头），输出进 stderr；
#   驱动退几都照常整张判一遍：命中记录的复用、缺的现跑，stdout 与单机逐字相同。第二台判出的记录由驱动当场导入本机那一份库，第二台不留长期库。
# 本次未跑（判法在 crates-mutation-rows.py 文件头「本次未跑」）：点名的测试打 NOT_RUN_NO_GPU_ADAPTER 后返回成功的那一条不算抓到、不算没红，整道不因它判红；
#   只有本次未跑的这一格退 77。模块写进 GATE_MUTATION_NOT_RUN_RECORD，这里转进 $GATE_NOT_RUN_FILE（驱动起的那两份不转，由本机整张那一趟再报）。
#
# 弄坏开关（只给证红用）。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（fixtures/checker-tier-crates-mutation-replay.sh/ 下的目录名，
# 括号里是 stage-selftest.sh 报那份样本判错的第一行的句首；样本档登记为 samples-heavy，句首都是逐个打开开关实跑样本档、照原样输出写的）：
#   GATE_MUTATION_BREAK=keepmutationtmp  拿掉删临时目录那一步
#       red（输出里找不到「endless-walk：20 秒没跑完（超时），点名的测试 walking_to_ten_ends_at_ten 没来得及红；它的临时目录已整个删掉（删前留有 1 个文件」）、
#       green（期望退出 0，实测 1）、not-run（期望退出 0，实测 1）
#   GATE_MUTATION_BREAK=nobaseline  跳过基线，读副本外文件的那一行误判成抓到
#       red（输出里找不到「计数：抓到 0 条、没红 1 条、无效 1 条、内存撞顶 1 条、超时 1 条…」）、
#       green（输出里找不到「基线：没改坏的副本上点名的测试恰好对上一个用例、结果是 ok 的 5 行，基线不绿 0 行…」）、
#       not-run（输出里找不到「基线：没改坏的副本上点名的测试恰好对上一个用例、结果是 ok 的 2 行，基线不绿 0 行…」）
#   GATE_MUTATION_BREAK=emptyreplacementrefused  替换文空着也判不成形
#       green（期望退出 0，实测 1）
#   GATE_MUTATION_BREAK=notrunasmissed  把本次未跑当没红记
#       not-run（期望退出 0，实测 1）
#   GATE_MUTATION_BREAK=quicktableunchecked  --quick 不核轻型变异登记表，登记的名字直接当 --item 交下去
#       quick-red（期望退出 1，实测 2）
#   GATE_MUTATION_BREAK=quickrunsallrows  --quick 不读轻型变异登记表，整张表都判
#       quick-green（输出里找不到「点名的 1 条变异都抓到了」）
#   MUTATION_SHARD_RUN_BREAK 的几项只在 scripts/mutation/mutation-shard-run.sh --selftest 起的进程里认（那份自证放的标记文件 MUTATION_SHARD_RUN_SELFTEST_MARK，
#   内容与驱动的 MUTATION_SHARD_SELFTEST_MARK_CONTENT 同一行）；别处设了，这一格判红、什么都不判：
#   MUTATION_SHARD_RUN_BREAK=break-honored-anywhere  别处设了也认
#       本道样本不判错；红在 scripts/mutation/mutation-shard-run.sh --selftest 的一格（「✗ ⑩ MUTATION_SHARD_RUN_BREAK 设在 --selftest 之外…」）
#   MUTATION_SHARD_RUN_BREAK=no-final-pass  调完驱动就按驱动的退出码退，不再整张判
#       本道样本不开双机、不判错；红在 scripts/mutation/mutation-shard-run.sh --selftest 的五格（「✗ ① checker-tier-crates-mutation-replay 接驱动那一趟判绿…」、
#       「✗ ① 最后那一趟全部复用…」、「✗ ① 第二台判的行连同覆盖记录导入…」、「✗ ③ 第二台那一份判完 1 条就停…」、「✗ ④ 驱动拒了之后 checker-tier-crates-mutation-replay 在本机整张判…」）
#   MUTATION_SHARD_RUN_BREAK=environment-opens  GATE_MUTATION_ACROSS_MACHINES=1 照旧把双机打开、不看配置的开关
#       本道样本不判错；红在 scripts/mutation/mutation-shard-run.sh --selftest 的一格（「✗ 单机那一趟（配置没开双机、环境变量 GATE_MUTATION_ACROSS_MACHINES=1）判绿…」）
#   复用、续跑、只改一行只跑那一行、单元键的每一样（第二轮复查 controls 路的 S34、S30、N33、N34、N45，被改坏的单元、点名测试的闭包、V5 那一类常量）、
#   基线按执行过的单元复用、判法单元指纹、久的先派、选行、导入这几样要连跑好几趟，在 scripts/mutation/crates-mutation-rows.py --selftest 里
#   用假 cargo、假 llvm 工具跑（门禁 code-tooling 替它复跑），弄坏开关 row-key-with-limits（红在 N33 那一格）、baseline-by-tree-snapshot、judge-units-whole-file、
#   mutated-unit-ignored、test-closure-ignored、rowkeywithouttoolchain、coveragenotrecorded、llvmversionunchecked、timingsignored、importunchecked、nobaseline、
#   emptyreplacementrefused、cargoconfignotcopied、outermemoryignored 下各有一格必须判错（逐个开关红在哪几格写在它文件头「弄坏开关」）。
#   编译并行度按内存上限收、行里已带 --exact 的变异记覆盖、点名的测试在这一组特性下不编：弄坏开关 compilejobsbyworkers、exactduplicated，以及 scripts/mutation/lib_mutation_units.py 里的
#   closure-error-masked（同样从 GATE_MUTATION_BREAK 认；按路径载入的 lib_function_units.py 不登记进 sys.modules，--list-items 末尾的真错被 PicklingError 盖住）各有格必须判错，
#   都在 crates-mutation-rows.py --selftest；本道样本不判这几条。
#   拷贝范围含 .cargo/（.cargo/config.toml 的 [env] BINDGEN_EXTRA_CLANG_ARGS，点名的测试要编 gpu 或 verdict-store 特性的行靠它编 librocksdb-sys）也在那里判：
#   假 cargo 见 --features gpu 而副本里读不到那个变量就照构建脚本报 stdbool.h 找不到、退 101，记「基线不绿」；弄坏开关 GATE_MUTATION_BREAK=cargoconfignotcopied
#   拷副本时跳过 .cargo/，那一格（「拷贝范围含 .cargo/：点名的测试要编 gpu 特性的那一行基线绿、变异抓到」）必须判错。本道样本不判这一条（样本用真 cargo、不编 librocksdb-sys）。
# 样本（fixtures/checker-tier-crates-mutation-replay.sh/；各放 .gate-cells 点名 crates-mutation-rows，setup.sh 在样本目录里 git init）：
#   red      变异表把上限压到 512M、限时压到 20 秒：1 MiB 的缓冲改成 1536 MiB 的那条报「内存撞顶」；步长 1 改成 0、循环走不到上界的那条报「超时」，
#            它点名的测试先往 TMPDIR 写一个 4 KiB 的文件，判定里报「临时目录已整个删掉（删前留有 1 个文件」（0 个说明 TMPDIR 没传到测试进程）；
#            点名的测试读 fixture-data/answer.txt（样本仓里有、不在拷贝范围里）的一行与只写名字、两个模块各有一个同名用例的一行都报「基线不绿」
#            （基线那一行报 4 行 ok、2 行不绿）；「临时目录」那一行报 4 条都删掉了；计数行里没红、无效、内存撞顶、超时各数各的；
#            只改注释的那一行（原文不落在任何单元上）与同名用例的那一行拒算，stderr 逐行写为什么
#   green    同样压到 512M，留一行撤掉了的「# 给整机留的内存余量：1T」：工作进程数那一行报开 5 个、卡在「这一趟要现跑的条数」，不按内存收；
#            五条正常变异照旧红在点名的测试上，其中一条改 litmus/sample.litmus（拷贝范围要含 litmus/），一条替换文空着、删掉 add_twice 里第二次相加；
#            基线那一行报 5 行都 ok，「临时目录」那一行报 5 条都删掉了，stderr 头一行报复用 0 条；真 cargo 插桩、真 llvm 工具记覆盖：
#            5 行 4 条点名测试（两行同点 adds_two_and_three_to_five），「覆盖：」那一行报记了 4 份、没记成 0 份；按行键：5 行各一把键、库是空的，
#            「复用：」那一行报现判 5 条、复用 0 条、拒算 0 条，5 条都在「行新加」那一栏，stderr 逐行写现判的原因
#   not-run  一条点名的测试只打 NOT_RUN_NO_GPU_ADAPTER 那一行就返回、一条正常变异：报本次未跑 1 条、抓到 1 条、这一格绿，成功句末尾报本次未跑 1 条
#   quick-green  放 .mutation-sample/quick（当成 --quick）：变异表两行同一个测试目标，轻型登记表登记其中一行，只判那一行、抓到、这一格绿；
#            --quick 那一行报与整张表同一份判定库（不再设 GATE_MUTATION_START_OVER）
#   quick-red    放 .mutation-sample/quick：轻型登记表点名一个变异表里没有的变异名，核表就退 1、一行都不判（不起 cargo）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_REPOSITORY="$(cd "$(dirname "$0")/../.." && pwd)"
source "$(dirname "${BASH_SOURCE[0]}")/lib/stage-cells.sh"
stage_cell crates-mutation-rows cell_crates_mutation_rows "crates/mutations.tsv 每条改坏一处、点名的测试都红，基线在没改坏的副本上绿" \
  "照上面逐条的判定改：没红的补测试或改变异，锚点失效的改原文，基线不绿的先让点名的测试在没改坏的副本上绿；只重跑某几行：bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --item <变异名>"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

list_items=0; root_argument=""; named_mutations=(); quick=0
while (($#)); do
  case "$1" in
    --list-items) list_items=1; shift ;;
    --quick) quick=1; shift ;;
    --item)
      if (($# < 2)) || [[ -z "$2" ]]; then
        echo "  ✗ --item 后面缺行号或变异名"
        echo "     → 怎么办：写成 --item <行号或变异名>（变异名是变异表第一列，原样照抄，名字里的逗号不拆），几行就给几次；可点的用 --list-items 看。"
        exit 2
      fi
      named_mutations+=("$2"); shift 2 ;;
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check crates-mutation-rows、--list-items、--quick、--item <行号或变异名> 与一个项目根。"
      exit 2 ;;
    *) root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$STAGE_REPOSITORY}"
cd "$ROOT" 2>/dev/null || exit 2
MUTATION_TABLE="crates/mutations.tsv"
# 只供样本：.mutation-sample/quick 当成给了 --quick（被判的根是这道阶段所在的仓、或设了 GATE_STAGED_FROM 时不认）
if [[ "$(pwd -P)" != "$(cd "$STAGE_REPOSITORY" && pwd -P)" && -z "${GATE_STAGED_FROM:-}" && -f .mutation-sample/quick ]]; then quick=1; fi
# --quick（文件头「--quick」）：轻型变异登记表核过才取它的行，连同 --item 点名的几行取并集；与整张表同一份判定库、同一套单元键复用
if ((quick)) && ((list_items == 0)) && [[ ",${GATE_MUTATION_BREAK:-}," != *,quickrunsallrows,* ]]; then
  light_rows_command=(python3 "$STAGE_REPOSITORY/scripts/mutation/crates-mutation-light-rows.py" names "$ROOT")
  if light_rows_output="$("${light_rows_command[@]}")"; then light_rows_exit=0; else light_rows_exit=$?; fi
  if ((light_rows_exit != 0)) && [[ ",${GATE_MUTATION_BREAK:-}," != *,quicktableunchecked,* ]]; then
    printf '%s\n' "$light_rows_output"
    echo "  ✗ --quick：轻型变异登记表 .claude/gate.d/crates-mutation-light-rows.tsv 核不过（crates-mutation-light-rows.py names 退 $light_rows_exit），一行都没判"
    echo "     → 怎么办：照上面逐处列的改登记表（写法与挑法在它的表头），改完单跑 python3 scripts/mutation/crates-mutation-light-rows.py check 到它退 0，再跑 --quick。"
    exit 1
  fi
  if ((light_rows_exit != 0)); then
    light_rows_output="$(awk -F'\t' '!/^#/ && NF >= 2 { print $1 }' .claude/gate.d/crates-mutation-light-rows.tsv 2>/dev/null)"
  fi
  light_row_count=0
  while IFS= read -r light_row_name; do
    [[ -n "$light_row_name" ]] || continue
    named_mutations+=("$light_row_name"); light_row_count=$((light_row_count + 1))
  done <<<"$light_rows_output"
  if ((light_row_count == 0)); then
    echo "  ✗ --quick：轻型变异登记表一行都没有，核心部分是空的"
    echo "     → 怎么办：照 .claude/gate.d/crates-mutation-light-rows.tsv 表头的挑法给 crates/mutations.tsv 里改坏对象在 crates/singlefs-core/src/ 下的行里每个测试目标登记一行。"
    exit 1
  fi
  echo "  · --quick：轻型变异登记表 $light_row_count 行（核心部分）$( ((${#named_mutations[@]} > light_row_count)) && echo "，另加点名的 $((${#named_mutations[@]} - light_row_count)) 项" )；与整张表同一份判定库，键对得上的复用、对不上的现判"
fi

if ((list_items)); then
  # 一行一条变异（# 起头的行与空行不算）：变异名<制表符>第几行、改哪个文件；只有共用模块里一份
  exec python3 "$STAGE_REPOSITORY/scripts/mutation/crates-mutation-rows.py" --list-items "$ROOT"
fi

item_selection_file=""
if ((${#named_mutations[@]})); then
  if [[ -n "${GATE_MUTATION_ROW_SELECTION:-}" ]]; then
    echo "  ✗ --item 与环境里的 GATE_MUTATION_ROW_SELECTION 一起给了：两份选行不知道听哪一份"
    echo "     → 怎么办：手跑只用 --item；GATE_MUTATION_ROW_SELECTION 只由双机分片的驱动设。"
    exit 2
  fi
  [[ -f "$MUTATION_TABLE" ]] || { echo "  ✗ 没有 $ROOT/$MUTATION_TABLE：--item 点名的变异无从找行"; echo "     → 怎么办：在项目根跑，或把项目根作为参数传进来。"; exit 1; }
  item_selection_file="$(mktemp)" || { echo "  ✗ 建不了选行文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
  trap 'rm -f -- "${item_selection_file:?}"' EXIT
  unknown_mutations=()
  for named in "${named_mutations[@]}"; do
    # 行号：那一行要是成形的一条（六段、# 起头与空行不算）；变异名：同名的几行都选上
    if [[ "$named" =~ ^[1-9][0-9]*$ ]]; then
      line_numbers="$(awk -F'\t' -v wanted="$named" 'NR == wanted && !/^#/ && !/^[[:space:]]*$/ && NF == 6 { print NR }' "$MUTATION_TABLE")"
    else
      line_numbers="$(awk -F'\t' -v wanted="$named" '/^#/ || /^[[:space:]]*$/ { next } $1 == wanted { print NR }' "$MUTATION_TABLE")"
    fi
    if [[ -z "$line_numbers" ]]; then unknown_mutations+=("$named"); else printf '%s\n' "$line_numbers" >> "$item_selection_file"; fi
  done
  if ((${#unknown_mutations[@]})); then
    printf '  ✗ --item 点名的变异不在 %s 里（写的是行号就不是成形的一行）：%s\n' "$MUTATION_TABLE" "$(printf '「%s」' "${unknown_mutations[@]}")"
    echo "     → 怎么办：写行号或变异名，照下面列的（bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --list-items 打的同一份）原样写："
    python3 "$STAGE_REPOSITORY/scripts/mutation/crates-mutation-rows.py" --list-items "$ROOT" | sed 's/^/       /'
    exit 2
  fi
  GATE_MUTATION_ROW_SELECTION="$item_selection_file"
  export GATE_MUTATION_ROW_SELECTION
fi

# ── crates-mutation-rows 格 ──
cell_crates_mutation_rows() {
# 弄坏开关 MUTATION_SHARD_RUN_BREAK 只在 scripts/mutation/mutation-shard-run.sh --selftest 起的进程里认（文件头弄坏开关那一段）：别处设了就判红、什么都不判
if [[ -n "${MUTATION_SHARD_RUN_BREAK:-}" && ",${MUTATION_SHARD_RUN_BREAK}," != *,break-honored-anywhere,* ]] \
   && ! { [[ -n "${MUTATION_SHARD_RUN_SELFTEST_MARK:-}" && -f "$MUTATION_SHARD_RUN_SELFTEST_MARK" ]] \
          && [[ "$(head -n 1 -- "$MUTATION_SHARD_RUN_SELFTEST_MARK")" == "mutation-shard-run.sh --selftest 起的进程" ]]; }; then
  echo "  ✗ MUTATION_SHARD_RUN_BREAK=$MUTATION_SHARD_RUN_BREAK 设在 scripts/mutation/mutation-shard-run.sh --selftest 之外：弄坏开关只在那份自证起的进程里生效，这一趟什么都没判"
  echo "     → 怎么办：去掉它再跑（env -u MUTATION_SHARD_RUN_BREAK …）；要证红就跑 MUTATION_SHARD_RUN_BREAK=<项> bash scripts/mutation/mutation-shard-run.sh --selftest"
  exit 1
fi
# 整道没有复用标记：复用落在每一行的行键上（共用模块 run_stage 逐行查判定库），这里不问 stage-must-run.sh、不写整道标记。
# 前提读的是本阶段所在那一份仓的登记表：判别力样本的目录里没有登记表，前提照样按真表判。
python3 "$STAGE_REPOSITORY/scripts/gate/admission.py" gate-preconditions "$STAGE_REPOSITORY" "$(basename "$0")" || exit 1
# 编译目录不在这里算（文件头「编译目录」）：GATE_MUTATION_TARGET_DIR、GATE_CROSS_RUN_TMPDIR 设了照原样传，默认位置由共用模块定
# 带内存上限跑一条命令的包装在这份阶段所在的仓里（样本仓里没有 research/）
GATE_MUTATION_MEMORY_CAP_RUNNER="$STAGE_REPOSITORY/scripts/hosts/run-with-memory-cap.sh"
export GATE_MUTATION_MEMORY_CAP_RUNNER
MUTATION_ROWS_MODULE="$STAGE_REPOSITORY/scripts/mutation/crates-mutation-rows.py"
if [[ ! -f "$MUTATION_ROWS_MODULE" ]]; then
  echo "  ✗ 找不到 $MUTATION_ROWS_MODULE：这一道的判法、派活与判定库都在它里面，一条都判不了"
  echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一道按红记。"
  exit 1
fi
# 双机分片（文件头「双机分片」）：驱动与配置判法的输出只进 stderr，stdout 只有下面 run_stage 那一趟的判定。
if [[ -z "${GATE_MUTATION_ROW_SELECTION:-}" ]]; then
  across_machines_skipped=""
  across_machines_configuration_exit=0
  if [[ "${SINGLEFS_GATE_FULL:-}" == 1 || "${GATE_MUTATION_START_OVER:-}" == 1 ]]; then
    across_machines_skipped="SINGLEFS_GATE_FULL=1 或 GATE_MUTATION_START_OVER=1：这一趟不复用判定库记录，分出去判的行本机还要再跑一遍"
  else
    bash "$STAGE_REPOSITORY/scripts/crash/crash-point-replay-shard-configuration-check.sh" "$ROOT" >/dev/null 2>&1 || across_machines_configuration_exit=$?
    if [[ ",${MUTATION_SHARD_RUN_BREAK:-}," == *,environment-opens,* && "${GATE_MUTATION_ACROSS_MACHINES:-}" == 1 ]]; then
      echo "  ! 弄坏开关 MUTATION_SHARD_RUN_BREAK=environment-opens：GATE_MUTATION_ACROSS_MACHINES=1 照旧把双机打开，不看配置的开关" >&2
      across_machines_configuration_exit=0
    fi
    case "$across_machines_configuration_exit" in
      0) ;;
      3) across_machines_skipped="双机：关——本地多机配置没写 ENABLE_ACROSS_MACHINES=1；要开就在配置里写上（模板仓根 multi-host.env.example）" ;;
      *) across_machines_skipped="本地配置判不过；单跑 bash scripts/crash/crash-point-replay-shard-configuration-check.sh 看原因" ;;
    esac
  fi
  if [[ -n "$across_machines_skipped" ]]; then
    echo "  … 双机分片：没分（$across_machines_skipped），整张表在本机判" >&2
  else
    if bash "$STAGE_REPOSITORY/scripts/mutation/mutation-shard-run.sh" "$ROOT" >&2; then shard_driver_exit=0; else shard_driver_exit=$?; fi
    echo "  … 双机分片：驱动退 $shard_driver_exit；下面在本机整张判一遍（命中判定库记录的复用、缺的现跑）" >&2
    if [[ ",${MUTATION_SHARD_RUN_BREAK:-}," == *,no-final-pass,* ]]; then
      echo "  ! 弄坏开关 MUTATION_SHARD_RUN_BREAK=no-final-pass：调完驱动就退，不再整张判" >&2
      exit "$shard_driver_exit"
    fi
  fi
fi
# 判法都在共用模块的 run_stage 里，这里 import 它、交出被判的仓根。
# 模块按名字登记进 sys.modules：进程池的工作进程靠这个名字找回要跑的函数。
# 本次未跑的那几条，模块逐条写进这一格自己的记录，跑完转进 $GATE_NOT_RUN_FILE。
not_run_record="$(mktemp)" || { echo "  ✗ 建不了本次未跑的记录文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
trap 'rm -f -- "${not_run_record:?}"' EXIT
GATE_MUTATION_NOT_RUN_RECORD="$not_run_record"
export GATE_MUTATION_NOT_RUN_RECORD
python3 - "$ROOT" "$MUTATION_ROWS_MODULE" <<'PY'
import importlib.util
import sys

specification = importlib.util.spec_from_file_location("crates_mutation_rows", sys.argv[2])
module = importlib.util.module_from_spec(specification)
sys.modules[specification.name] = module
specification.loader.exec_module(module)
sys.exit(module.run_stage(sys.argv[1]))
PY
module_exit=$?
unset GATE_MUTATION_NOT_RUN_RECORD
not_run_count="$(grep -c . "$not_run_record" 2>/dev/null)" || not_run_count=0
# 双机分片的驱动起的那两份（GATE_MUTATION_ROW_SELECTION 不是 --item 换成的）不转：分出去的行由本机整张那一趟再判、再报
if ((not_run_count > 0)) && [[ -n "${GATE_NOT_RUN_FILE:-}" && ( -z "${GATE_MUTATION_ROW_SELECTION:-}" || -n "$item_selection_file" ) ]]; then
  cat "$not_run_record" >> "$GATE_NOT_RUN_FILE"
fi
# --item 点名的几行：选中的全抓到时模块退 3（选行模式给驱动看的约定），人点名的全抓到就是这一格绿；
# 选中的里有本次未跑的不说都抓到，全是本次未跑的退 77。
if [[ -n "$item_selection_file" && "$module_exit" == 3 && "$not_run_count" -gt 0 ]]; then
  selected_count="$(sort -u "$item_selection_file" | grep -c .)"
  if ((not_run_count >= selected_count)); then
    echo "  · 点名的 $selected_count 条变异全是本次未跑（逐条见上），一条都没判到被改坏的那一处：这一格记本次未跑，不是通过"
    exit 77
  fi
  echo "  ✓ 点名的 $selected_count 条变异里抓到 $((selected_count - not_run_count)) 条、本次未跑 $not_run_count 条（逐条见上；只判了这几行，不是都抓到、也不是整道结论）"
  exit 0
fi
if [[ -n "$item_selection_file" && "$module_exit" == 3 ]]; then
  selected_count="$(sort -u "$item_selection_file" | grep -c .)"
  echo "  ✓ 点名的 $selected_count 条变异都抓到了（只判了这几行，不是整道结论；现判、复用、拒算各几条见上面「复用：」那一行）"
  exit 0
fi
exit "$module_exit"
}

# 双机分片的驱动起的那两份不经格汇总，照驱动的约定退（文件头 GATE_MUTATION_ROW_SELECTION 那一条）
if [[ -n "${GATE_MUTATION_ROW_SELECTION:-}" && -z "$item_selection_file" ]]; then
  ( cell_crates_mutation_rows )
  exit $?
fi
stage_cells_run "$ROOT"
