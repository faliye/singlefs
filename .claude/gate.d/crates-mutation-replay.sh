#!/usr/bin/env bash
# task-server-cargo-family: -p singlefs-verification --features gpu
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 要的工具与设备登记在 stage-inputs.tsv 本阶段那一行第三列，由 scripts/gate/admission.py gate-preconditions 在阶段里判，没齐判红，不交给 gate.sh 预判
# gate-stage: crates 变异表复跑（每条改坏一处、点名的测试必须红）
# gate-fixture: yes
# gate-category: checker-tier 类
# gate-similar: code-source-discipline.sh 它的实验与变异源码纪律那一格经同一份判法（scripts/mutation/crates-mutation-rows.py static-check）静态判变异表成形与拷贝范围，不改坏、不起 cargo；这里真改坏每一处、真跑点名的测试（起 cargo），两边共用判法模块、不共用阶段
# gate-similar: research-build-and-replay.sh 它的复跑那一格也按输入复用、也逐条跑编出来的代码，但跑的是 research 的实验二进制、比的是产物；这里改坏的是 crates 的源码、判的是点名的测试红不红
#
# gate-cell: crates-mutation-rows crates/mutations.tsv 每条改坏一处、点名的测试都红，基线在没改坏的副本上绿
# 参数解析、格汇总与退出码经共用库 lib/stage-cells.sh（写法在它的文件头）。
#   bash .claude/gate.d/crates-mutation-replay.sh [项目根]                                整张表
#   bash .claude/gate.d/crates-mutation-replay.sh --list                                  逐行打格名与判什么，不起 cargo
#   bash .claude/gate.d/crates-mutation-replay.sh --check crates-mutation-rows [项目根]     只跑这一格（格名写错退 2）
#   bash .claude/gate.d/crates-mutation-replay.sh --list-items [项目根]                    逐行打变异表里可点名的变异名、行号与改哪个文件，不起 cargo；末了报这一趟键对不上（要现判）的条数
#                                                                                                      与锚点失效的行（逐行点名），再一行报牵连面（打的是 crates-mutation-rows.py --list-items，同一份）
#   bash .claude/gate.d/crates-mutation-replay.sh --list-items --with-keys [项目根]          另在逐行列出之后、末行合计之前，每项打一行 ITEM_PLAN（下面「按项机读行」），不起 cargo；键没算出来退 1
#   bash .claude/gate.d/crates-mutation-replay.sh --list-items --with-keys --item <行号或变异名> [--item …] [项目根]
#                                                                                                      同上，只列、只算点名那几行（--item 换成选行文件交给 crates-mutation-rows.py 的 --rows）：成本随点名的行数走，不随整表走；
#                                                                                                      --item 不跟 --with-keys 一起给 --list-items 退 2
#   bash .claude/gate.d/crates-mutation-replay.sh --item <行号或变异名> [--item …] [项目根]   只判点名的几行（crates-mutation-rows.py --item 交到这里）
#   --item：写变异表里成形的一行的行号，或变异名原样（名字里的逗号不拆、同名的几行都判）；不在表里列出可点名的、退 2。换成行号走共用模块的选行模式
#     （照常写判定库记录、照复用判据复用：点名单跑与整张跑时那一行的判定相同）。
#     选中的全抓到这一格绿（模块退 3，这里换成 0，成功行写明只判了这几行）；有本次未跑的，成功行写明抓到几条、本次未跑几条；全是本次未跑的退 77。
#   GATE_MUTATION_ROW_SELECTION=<文件>（一行一个变异表行号，只由分片驱动 scripts/mutation/mutation-shard-run.sh 的 shard 那一步设）：只判这些行（它们所在的基线批照跑），照常写判定库记录，
#     末行写明「只判了选中的 N 行，不是整道结论」；不经格汇总，选中的全抓到退 3、有红退 1
#     （汇总把 0、77 之外的码并成 1，驱动就分不出这两样）。
#
# 判据（判法、派活、判定库与覆盖都在 scripts/mutation/crates-mutation-rows.py；拷贝范围、判法单元、行键、判定库放哪、复用判据、弄坏开关见它的文件头）：
#   crates/mutations.tsv 每行六段（变异名、文件、原文、替换文、cargo test 参数、必须红的测试名），替换文可以空（删掉原文），其余五段不许空；
#   必须红的测试名是一个用例名，只有 \n 一种反斜杠转义，文件落在每片拷的范围（stage-inputs.tsv 本阶段那一行）之内：
#   这几样派活之前判（row_problems，门禁 code-source-discipline 经 static-check 静态判同一份），有一行不过整道判红、一个工作进程都不起；
#   原文在文件里恰好命中一次：派活之前按行判，不是的那一行记「锚点失效」（不跑、不入库、不复用，「计数：」里「锚点失效 N 行」一栏），别的行照判，整道判红
#   （共用模块文件头「锚点失效」；--item 与分片同样按行）；
#   点名的测试在没改坏的副本上恰好对上一个用例、结果是 ok（基线）；改坏那一处之后它必须红。一条不过整道红。
# 牵连面（只报数、不判红，共用模块文件头「牵连面」）：成功句之后与 --list-items 末了各一行「牵连最大的函数单元：… 进了 N 行的闭包（前 5：…）；一次改动最坏牵连 N 行」。
# 为什么：show-me-test.md「新增的测试必须先证明它会红」要变异清单进仓、交给门禁反复复跑；每处三方打中之后的改法留一条「改回去它就红」的变异，撤回时这里先响。
#
# 并发（派活与进程数的判法在 crates-mutation-rows.py 的 worker_count_for 与 dispatch_order）：每片各一份源码副本与一个编译目录（共用一份会被 cargo 的文件锁重新串行化）；
#   按 crate 归堆派（增量编译的命中率才稳），活动态领（等分之下拿到几十秒重测试的那一片会独自拖住整道），上一次用时 ≥ GATE_MUTATION_LONG_ROW_SECONDS（默认 300）秒的先派。
#   工作进程数取 ① 每台默认重活线程数（设了 GATE_MUTATION_WORKERS 就顶掉 ①）、② 这一趟要现跑的条数、③ 编译目录所在的盘放得下几个工作进程的两份编译目录（下面「磁盘余量」）、
#   ④ 这一条拿到的内存上限 ÷ 那个编译并行度下每个工作进程的实测峰值（上限取 TASK_SERVER_MEMORY_CAP_BYTES，没有读仓根多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项，再没有读自己 scope 的 memory.max；
#   memory.max 是有限值就是硬顶（前两级取到的值比它大取它，两者取小），是 max 或读不到不收窄、前两级也没有这一项不收）的小者、至少 1；撞内存上限由服务放大重排、从头跑，所以不能按线程数开满。
#   每个工作进程的峰值由门禁自己量：每个工作进程里一条采样线程量它整棵进程树（cargo、rustc、测试、临时 prune-server）的 RssAnon + RssShmem + VmPTE 加总，
#   按编译并行度记进多机配置 NFS_SHARED_ROOTS 点名的共享根下的运行记录（worker-peaks/<编译并行度>/，两台同一份；判样本根时不记）；④ 排在最后、按 ①②③ 的小者当天花板，
#   每试一个进程数 n 就照 n 算编译并行度、取那个编译并行度下最新几份实测的最大，满足 n × 峰值 ≤ 上限的最大 n 才开；没有实测（或记录读不了）取保守值 = 上限本身，只开 1 个。
#   判法在 worker_count_for，内存那一项在 memory_bound_for（crates-mutation-rows.py 文件头「编译并行度」「内存上限」，口径与内存包装记的「回收不掉的用量」差在哪也写在那里），
#   头一行与收尾一行写上限、每个工作进程的峰值与取法、个数与合计；弄坏开关 GATE_MUTATION_BREAK=memory-cap-ignored、memory-max-not-hard-cap（取到变量或多机配置的值就不拿 memory.max 当硬顶）、
#   worker-peaks-unkeyed-by-jobs、worker-peaks-window-unbounded、jobs-feedback-ignored、unmeasured-worker-peak-guessed、unreadable-worker-peaks-guessed、worker-tree-self-only、worker-peaks-unrecorded、
#   sample-root-peaks-recorded（样本根上照样写峰值记录），
#   自证在 crates-mutation-rows.py --selftest（selftest_memory_cells、selftest_worker_peak_cells）；本道样本不判这一条。
#   判完的格每条立刻原子写进判定库（工作进程里 Library.write：同目录临时文件、fsync、改名），整组被硬杀之后重跑同一条命令，判过的格照键复用、只现判剩下的；
#   自证 selftest_killed_run_resume_cell 起子进程跑到一半对整组 SIGKILL 再重跑；弄坏开关 GATE_MUTATION_BREAK=records-written-at-end（记录攒到进程池收尾才写）下这一格必须判错。
#   重跑拿不到复用的另一个原因：crates-mutation-rows.py 里任何一处模块级赋值、import、类头、带缺省值的 def 头（连同自证用的 FAKE_CARGO 这类字符串常量）改了，判法单元指纹就变，整库的行键跟着全变（文件头「判法单元」），不是没写。
#   线程数按工作进程数压：每个 cargo 的编译并行度 CARGO_BUILD_JOBS 取 max(1, 核数 ÷ 工作进程数)（GATE_MUTATION_CARGO_JOBS 顶掉它，要正整数；与 ④ 试进程数时同一个函数 cargo_jobs_for）。
#   磁盘余量（工作进程数的 ③，crates-mutation-rows.py 的 disk_bound_for）：每个工作进程最多占两份编译目录（普通、插桩），一份涨满按下面「编译目录上限」算；
#   派活之前按目标盘余量（外层给了 GATE_MUTATION_DISK_FREE 用它，不然读编译目录所在的盘）核「片号 1…n 各两份还能涨的量 + 备用 GATE_MUTATION_DISK_RESERVE（默认 30G）」，
#   放不下的 n 不开，头一行写磁盘卡在哪一项；余量没读到（两样都读不到）没有这一项，头一行写明原因。自证与弄坏开关 disknotchecked 在 crates-mutation-rows.py --selftest
#   （「磁盘余量放不下 16 份」等四格）；本道样本不判这一条。
#   编译目录上限（GATE_MUTATION_TARGET_DIR_CAP，默认 64G，trim_target_directory）：每个工作进程每判完一格量一次自己那份编译目录（判基线与变异的普通那份、判覆盖的插桩那份）的占盘，
#   超过上限就在两格之间清掉底下的增量产物（<目录>/<档>/incremental），依赖与测试二进制不动，清完仍超线的只报；弄坏开关 targetcapignored，自证同在 crates-mutation-rows.py --selftest
#   （「编译目录超过上限」等三格）；本道样本不判这一条。
#   被测试自己起的线程不压：RUST_TEST_THREADS 与各测试读的线程数变量调用方设了什么就原样透传给 cargo test。
#   ⚠️ stdout 里的判定与进程数无关：按变异表的行号排序再打，GATE_MUTATION_WORKERS=1 与 =8 逐字相同（并发之后重新证红的那一半）；进度、前移只进 stderr；
#   复用了多少进 stdout 的「复用：」「覆盖：」两行、抓到那一行末尾的「（现判：<栏>）」「（复用）」或「（拒算）」与成功句末尾「；这一趟现判 …」，比判定时去掉这几处。
#
# 编译目录（用户 2026-10-23 定案「自己用完自己删」，不跨轮复用）：GATE_MUTATION_TARGET_DIR 设了用它（调用方给的，跑完不删）；没设时共用模块在这一趟自己的运行目录
#   （$TMPDIR 下的 singlefs-mutation-replay-*）里建 build/singlefs-crates-mutation-target，分片各加 -w<片号>、记覆盖的再加 -coverage、先编一份依赖的 -seed 几份都在那个 build 里，
#   正常结束、判红、被 TERM / HUP 停都删掉（删之前核没有进程开着里面的文件，有就不删、整道判红）。
#   只在 crates-mutation-rows.py 的 default_shared_target_directory 与 run_pool 里算，这里不另算（它文件头「编译目录」）。
#   片号锁、调用方给的编译目录上多出来的 -w<片号> 回收照旧（同一段）；自证在 crates-mutation-rows.py --selftest，
#   弄坏开关 GATE_MUTATION_BREAK=extratargetskept、inusetargetremoved、targetdefaulttmp、runtargetkept、runtargetopenfilesunchecked（本道样本不判这一条）。
#   先编一份依赖（crates-mutation-rows.py 文件头「先编一份依赖」）：开 2 个以上工作进程、要现跑的行选的包够得着外部依赖时，派活之前在 <共用目录>-seed（插桩的 -seed-coverage，
#   源码副本 -seed-source 跨轮留着、按内容同步，锁 -seed.lock）里按组各编一次，工作进程起来时只把外部依赖的产物拷进自己的两份编译目录，工作区的包照旧各编各的
#   （编出来的二进制里编死了 CARGO_MANIFEST_DIR，拷过去 cargo 照样判 Fresh、跑的时候读先编那一份的文件）；回收不碰这几份。先编那一份里摸过的包有产物是 fresh 整道判红。
#   自证与弄坏开关 compile-per-worker、workspaceunitsshared、seedmtimekept、seedfreshunchecked 在 crates-mutation-rows.py --selftest；本道样本不判这一条。
# 调试信息：这一道起的每条 cargo 都带 CARGO_PROFILE_{DEV,TEST,RELEASE,BENCH}_DEBUG=false（cargo test 用 test 档、承 dev；cargo test --release 用 release 档），
#   顶掉仓根 Cargo.toml 的 [profile.release] debug = true 与调用方的同名变量；取值进行键（工具链行），改了整张表现判一次（crates-mutation-rows.py 文件头「调试信息」）。
#   自证与弄坏开关 debuginfo-kept、debuginfo-unkeyed 在 crates-mutation-rows.py --selftest；本道样本不判这一条。
#   片号锁的自证同在那里：弄坏开关 GATE_MUTATION_BREAK=local-shard-numbering（两次并发调用领到的片号重叠）那一格必须判错
#   （直接起自证时同义名 CRATES_MUTATION_ROWS_BREAK）；本道样本不判这一条。
# 崩溃判定（crates-mutation-rows.py 文件头「变异复跑的崩溃判定」）：变异行的判定照行键经 prune-server 写进真服务里那一份判定库（下面「复用」）；
#   起 cargo test 真跑测试的每一次（基线的批、变异、记覆盖的插桩跑）共用模块另起一份它自己的临时 prune-server，走判定库的 验证包用例在变异版上判出的崩溃结论
#   只落进那一份、跑完停服务删目录，不读不写真服务里的崩溃判定；这里不设库的变量、不建本地库。
#   临时服务的二进制由这一格交下去（prune_server_binary_for_mutation_runs）：PRUNE_SERVER_BINARY 设了（绝对路径、可执行）照原样交，不是的这一格判红、一条都不判；
#   没设先照这一道所在仓的 scripts/hosts/rocksdb-prebuilt.py entry 判预编库的取值，再在这一格自己的临时目录里编一份（直接起，线程是服务套的 capped.sh 设好的变量，
#   cargo build --release --offline --manifest-path <这一道所在的仓>/prune-server/Cargo.toml --bin prune-server，与 code-tooling.sh 的 code_tooling_prune_server_binary 同一种编法），
#   设成 PRUNE_SERVER_BINARY 交给共用模块（它认这个变量、不再自己编），跑完连同编译目录删；判不过、编不出这一格判红、一条都不判。
#   --list-items 不起服务、不取二进制。本道样本不判这一条（样本照样经这一步取二进制）。
# 临时目录：每条变异与每批基线各一个 TMPDIR，结束时不论退出码（含被信号杀掉的测试进程）都整个删掉：被杀的测试 Drop 守卫不跑，镜像会堆在里面；
#   全部判完总目录下还剩哪一条的目录，整道判红。
#
# 上限、限时与卡住（records/2026-10-30-编译绕过内存总上限.md SC02，用户定「照字面全删」「两样都交服务」）：每条 cargo test 直接起，不套内存包装、不设上限、不限时；
#   这一道整条经 task-server/task submit 交，内存上限由统一任务服务按记录给、撞了放大重排，死循环的变异由服务按「日志与 CPU 都不涨」停整条任务。
#   不判「内存撞顶」「超时」「显存不够」：显存不够而红的照抓到记。判法在 crates-mutation-rows.py 文件头同名一段。
#
# 基线（判法见 crates-mutation-rows.py 文件头「基线」）：点名的测试在没改坏的副本上本来就红时，改坏之后照样红、记「抓到」什么都没证明，
#   所以派变异之前先跑基线，不是恰好一个 ok 的行记「基线不绿」、整道判红。「计数：」那一行的格式不动（mutation-triage 照栏读），基线另起一行「基线：…」报。
# 复用：每行一条判定，行键是被改坏的函数单元（种类、指纹；所在文件与限定名不进键）、原文、替换文、点名测试执行过的单元集合指纹、判「抓没抓到」的判法单元指纹、工具链、解析出的依赖版本，
#   不带变异名；没有管整库的版本号、没有版本目录；一份库，两台判出的都经 prune-server 写进同一份（分片合并只核两片判的行按此刻的行键在服务里读得到）。
#   库名固定（mutation-units 族，种类 singlefs-crates-mutation-library），库根不进键，派到哪台、判的是哪棵树都写同一份；取不到库名、服务连不上整道退出（连不上退 75），不现判；
#   成功句末尾点名库在哪。判出的不是「抓到」（没红、无效、基线不绿……）照行键盖掉原有的「抓到」，下一趟现判。
#   一行的基线结局不随这一趟选了哪几行而变（只看这一行自己的过滤词选中的结果行）：点名单跑与整张表对同一行判同一个结论。
#   判法单元连同 judge_row、judge_batch、run_cargo_command 这几段收闭包；逐样与弄坏开关在 crates-mutation-rows.py 文件头「判法单元」「行键」「判定库」「复用判据」「基线」。
#   没有整道全绿标记，不问 stage-must-run.sh。点名测试那一段这里取它的单元闭包键（入口 = 那条测试函数，比执行过的单元宽，不漏）；
#   执行过的单元另记在覆盖记录里（-C instrument-coverage、llvm-profdata、llvm-cov，工具不对整道判红），基线 ok 那一条按它复用、不按整树快照。
#   键对不上的现判，按栏报数（行新加、判法单元变了、被改坏的单元变了、点名测试的闭包变了、原文或替换文变了、工具链或依赖变了、上次没判成抓到、一律现判），
#   stderr 逐行写变了哪几个单元；算不出键的那一行拒算（点名的测试认不出单元、闭包里有认不出的形态），每趟照跑。成功句末尾报「现判 N、复用 M、拒算 R、本次未跑 K」。
#   撞键就拒算（用户 2026-10-29 定）：表里两行以上算出同一把行键（两份内容相同的函数各改一处、点名同一条测试），这几行拒算、照跑、不复用，
#   stderr 逐行与成功句末尾报「拒算（行键撞键：与第 N 行同键）」；--item 与分片只判选中的行时，撞键照整张表判。
#   判法与弄坏开关 row-key-collision-unrefused 在 crates-mutation-rows.py 文件头「行键」，自证在它的 --selftest；本道样本不判这一条。
#   键、库的存放、复用判据写在 crates-mutation-rows.py 文件头「复用」「判法单元」「行键」「判定库」「复用判据」。只读看这一趟会怎么判：python3 scripts/mutation/crates-mutation-rows.py plan <根>；
#   --list-items 末行报键对不上的条数。只有「抓到」与基线 ok 复用。
# 分片：这一道不自己分两台、不调分片驱动（机器与分片归 task-server，收尾批 GT02）。要两台分着判的，另经 task-server 提交
#   scripts/mutation/mutation-shard-run.sh（服务按分片协议拆成计划、各片、合并：各片照 GATE_MUTATION_ROW_SELECTION 起这一道只判自己那一堆、判定写进 prune-server 里同一份库，
#   合并按此刻的行键核两片判的行在服务里读得到，步骤在它文件头），之后照常经 task-server 起这一道整张判：命中记录的复用、缺的现跑，stdout 与不分片逐字相同。
# 本次未跑（判法在 crates-mutation-rows.py 文件头「本次未跑」）：点名的测试打 NOT_RUN_NO_GPU_ADAPTER 后返回成功的那一条不算抓到、不算没红，整道不因它判红；
#   test= 对不上测试名的（崩溃枚举用例判段不跑时打 test=crash_state_enumeration::<项名>）同样记本次未跑，不记没红（弄坏开关 not-run-test-name-only，
#   自证在 crates-mutation-rows.py --selftest「本次未跑认项名」那一格；本道样本不判这一条）。
#   只有本次未跑的这一格退 77。模块写进 GATE_MUTATION_NOT_RUN_RECORD，这里转进 $GATE_NOT_RUN_FILE（分片驱动起的各片不转，由之后整张那一趟再报）。
#
# 弄坏开关（只给证红用）。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（fixtures/crates-mutation-replay.sh/ 下的目录名，
# 括号里是 stage-selftest.sh 报那份样本判错的第一行的句首；样本档登记为 samples:.crates-mutation-replay-sample-tools（假工具标记目录），句首都是逐个打开开关实跑样本档、照原样输出写的）：
#   GATE_MUTATION_BREAK=keepmutationtmp  拿掉删临时目录那一步
#       red（输出里找不到「临时目录：2 条变异各一个自己的 TMPDIR，每条结束时不论退出码都整个删掉了」）、
#       green（期望退出 0，实测 1）、not-run（期望退出 0，实测 1）
#   GATE_MUTATION_BREAK=nobaseline  跳过基线，读副本外文件的那一行误判成抓到
#       red（输出里找不到「计数：抓到 0 条、没红 1 条、无效 1 条、锚点失效 0 行（共 4 条）」）、
#       green（输出里找不到「基线：没改坏的副本上点名的测试恰好对上一个用例、结果是 ok 的 5 行，基线不绿 0 行…」）、
#       not-run（输出里找不到「基线：没改坏的副本上点名的测试恰好对上一个用例、结果是 ok 的 2 行，基线不绿 0 行…」）
#   GATE_MUTATION_BREAK=emptyreplacementrefused  替换文空着也判不成形
#       green（期望退出 0，实测 1）
#   GATE_MUTATION_BREAK=notrunasmissed  把本次未跑当没红记
#       not-run（期望退出 0，实测 1）
#   复用、续跑、只改一行只跑那一行、单元键的每一样（第二轮复查 controls 路的 S34、S30、N33、N34、N45，被改坏的单元、点名测试的闭包、V5 那一类常量）、
#   基线按执行过的单元复用、判法单元指纹、久的先派、选行这几样要连跑好几趟，在 scripts/mutation/crates-mutation-rows.py --selftest 里
#   用假 cargo、假 llvm 工具跑（门禁 code-tooling 替它复跑），弄坏开关 baseline-by-tree-snapshot、judge-units-whole-file、
#   mutated-unit-ignored、test-closure-ignored、rowkeywithouttoolchain、coveragenotrecorded、llvmversionunchecked、timingsignored、nobaseline、
#   emptyreplacementrefused、cargoconfignotcopied、broken-pool-looked-up-late 下各有一格必须判错（逐个开关红在哪几格写在它文件头「弄坏开关」）。
#   行里已带 --exact 的变异记覆盖、点名的测试在这一组特性下不编：弄坏开关 exactduplicated，以及 scripts/mutation/lib_mutation_units.py 里的
#   closure-error-masked（同样从 GATE_MUTATION_BREAK 认；按路径载入的 lib_function_units.py 不登记进 sys.modules，--list-items 末尾的真错被 PicklingError 盖住）各有格必须判错，
#   都在 crates-mutation-rows.py --selftest；本道样本不判这几条。
#   拷贝范围含 .cargo/（.cargo/config.toml 的 [env] BINDGEN_EXTRA_CLANG_ARGS，点名的测试编到 librocksdb-sys（验证包经 prune-server 依赖它）的行靠它）也在那里判：
#   假 cargo 见 --features gpu 而副本里读不到那个变量就照构建脚本报 stdbool.h 找不到、退 101，记「基线不绿」；弄坏开关 GATE_MUTATION_BREAK=cargoconfignotcopied
#   拷副本时跳过 .cargo/，那一格（「拷贝范围含 .cargo/：点名的测试要编 gpu 特性的那一行基线绿、变异抓到」）必须判错。本道样本不判这一条（样本用假 cargo）。
#   点名的测试要上卡的行不另套清场（crates-mutation-rows.py 文件头「带 gpu 特性的格不另套清场」：清场与分卡归统一任务服务，服务在最外层给整条门禁套一次）：上卡的特性是 GPU_TEST_FEATURES 里的两个，验证包的 gpu 与
#   产品 GPU 包 singlefs-background-gpu 真显卡对拍的 tests-on-a-real-gpu；点名的测试没卡时打 NOT_RUN_NO_GPU_ADAPTER 后返回成功，记本次未跑（同上「本次未跑」）。
#   弄坏开关 GATE_MUTATION_BREAK=product-gpu-feature-unquiesced 让 tests-on-a-real-gpu 不当上卡的特性认，crates-mutation-rows.py --selftest 里
#   「产品 GPU 包（--features tests-on-a-real-gpu）的行：…」两格必须判错；本道样本不判这一条（样本用假 cargo、没有显卡）。
#   开 GPU 时崩溃判段那几行补 gpu 特性（用户 2026-10-13 定「配 GPU 必用 GPU，不降级」；crates-mutation-rows.py 文件头同名一段）：多机配置写了 ENABLE_GPU=1 时，
#   点名测试的单元闭包够得着 crash_state_enumeration 或 crash_amplification 的选路函数、cargo 参数没开 gpu 的行，这一趟补上 gpu 特性起（照上面不另套清场、交多机配置），
#   键照补了之后的特性算；变异表那一行不动，stderr 一行点名补了哪几行。弄坏开关 GATE_MUTATION_BREAK=gpu-route-feature-unadded（不补），
#   自证在 crates-mutation-rows.py --selftest「开 GPU 时点名测试走崩溃判段的那一行补 gpu 特性」那一格；本道样本不判这一条（样本 crate 里没有那两处选路函数）。
#   带 gpu 特性、真跑测试的 cargo test（基线的批、变异、记覆盖的插桩跑）的 libtest 参数收尾带 --test-threads 1（.claude/rules/verification.md 里 GPU 那一条），
#   点名的测试目标全在 scripts/gate/admission.py 的 GPU_TEST_THREADS_RELEASED_TARGETS 里的带每台默认重活线程数；行里自己已写 --test-threads 的不重复补；线程参数不进行键
#   （crates-mutation-rows.py 文件头「带 gpu 特性的格测试参数收尾带 --test-threads」）。弄坏开关 GATE_MUTATION_BREAK=gpu-test-threads-unadded（不补），
#   自证在 crates-mutation-rows.py --selftest「带 gpu 的行收尾带 --test-threads 1、放开名单里的目标带登记值、不带 gpu 的行不带」那一格；本道样本不判这一条（样本用假 cargo、没有显卡）。
# 按项机读行（统一任务服务按项先算后排读它；一项 = 变异名，同名的几行合成一项；格式与算法在 crates-mutation-rows.py 文件头「按项机读行」）：
#   --list-items --with-keys   stdout，每项一行（次序是变异表的次序，行数等于逐行列出的项数，条数之和等于末行合计的数）：
#       ITEM_PLAN item=<项名> to_judge=<要现判的条数：键对不上与拒算> reusable=<能复用的条数：键对得上>
#   真跑（整张表、--item、分片的各片同样）  stderr，每项跑完一行（带挂钟，所以不进 stdout）：
#       ITEM_DONE item=<项名> seconds=<这一项自己的挂钟秒数> judged=<现判条数> reused=<复用条数> exit=<这一项的退出码：0 抓到、77 本次未跑、1 其余>
#   弄坏开关 GATE_MUTATION_BREAK=item-plan-reusable-zero、item-plan-selection-ignored（带 --item 也算整表）、item-done-reused-silent、item-done-seconds-whole-run，
#   自证在 crates-mutation-rows.py --selftest（selftest_item_lines_cells 五格）；本道样本不判这几条。
# 样本（fixtures/crates-mutation-replay.sh/；各放 .gate-cells 点名 crates-mutation-rows，setup.sh 在样本目录里 git init；
#   样本根放 .crates-mutation-replay-sample-tools/（标记，里面一个说明文件）：起 cargo 的是 crates-mutation-rows.py sample-tools 现写的假 cargo 与假 llvm 工具，
#   不编译；样本 crate 里 `// TEST <名字> NEEDS|READS|FILES|NOTRUN …` 与 `// COMPILE-ERROR-IF <原文>` 几行说每条测试、每处替换文的红绿（写法见 FAKE_CARGO 头上的说明）；
#   只在样本根上进样本档：根不是本阶段所在的仓、也没设 GATE_STAGED_FROM）：
#   red      只改注释的一行（原文不落在任何单元上，拒算）照跑记「没红」；替换文编不过的一行记「无效」；
#            点名的测试读 fixture-data/answer.txt（样本仓里有、不在拷贝范围里）的一行与只写名字、两个模块各有一个同名用例的一行都报「基线不绿」
#            （基线那一行报 2 行 ok、2 行不绿）；「临时目录」那一行报 2 条都删掉了；计数行里没红、无效各数各的；
#            只改注释的那一行与同名用例的那一行拒算，stderr 逐行写为什么
#   工作进程数不在样本里判：样本根上的工作进程不写峰值记录（crates-mutation-rows.py 的 worker_peaks_recorded_for），有内存上限时开几个随样本读到的运行记录里
#   有没有实测而变（没有就照「从没量过」只开 1 个），没有内存上限时照条数开；开几个的判法在 crates-mutation-rows.py --selftest（selftest_memory_cells、selftest_worker_peak_cells）。
#   green    五条正常变异照旧红在点名的测试上，其中一条改 litmus/sample.litmus（拷贝范围要含 litmus/），一条替换文空着、删掉 add_twice 里第二次相加；
#            基线那一行报 5 行都 ok，「临时目录」那一行报 5 条都删掉了，stderr 头一行报复用 0 条；假 cargo 与假 llvm 工具记覆盖：
#            5 行 4 条点名测试（两行同点 adds_two_and_three_to_five），「覆盖：」那一行报记了 4 份、没记成 0 份；按行键：5 行各一把键、库是空的，
#            「复用：」那一行报现判 5 条、复用 0 条、拒算 0 条，5 条都在「行新加」那一栏，stderr 逐行写现判的原因
#   not-run  一条点名的测试只打 NOT_RUN_NO_GPU_ADAPTER 那一行就返回、一条正常变异：报本次未跑 1 条、抓到 1 条、这一格绿，成功句末尾报本次未跑 1 条
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_REPOSITORY="$(cd "$(dirname "$0")/../.." && pwd)"
source "$(dirname "${BASH_SOURCE[0]}")/lib/stage-cells.sh"
stage_cell crates-mutation-rows cell_crates_mutation_rows "crates/mutations.tsv 每条改坏一处、点名的测试都红，基线在没改坏的副本上绿" \
  "照上面逐条的判定改：没红的补测试或改变异，锚点失效的改原文，基线不绿的先让点名的测试在没改坏的副本上绿；只重跑某几行：bash .claude/gate.d/crates-mutation-replay.sh --item <变异名>"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

list_items=0; with_keys=0; root_argument=""; named_mutations=()
while (($#)); do
  case "$1" in
    --list-items) list_items=1; shift ;;
    --with-keys) with_keys=1; shift ;;
    --item)
      if (($# < 2)) || [[ -z "$2" ]]; then
        echo "  ✗ --item 后面缺行号或变异名"
        echo "     → 怎么办：写成 --item <行号或变异名>（变异名是变异表第一列，原样照抄，名字里的逗号不拆），几行就给几次；可点的用 --list-items 看。"
        exit 2
      fi
      named_mutations+=("$2"); shift 2 ;;
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check crates-mutation-rows、--list-items [--with-keys]、--item <行号或变异名> 与一个项目根；缩小范围用 --item <行号或变异名>（--list-items 列可点的项）。"
      exit 2 ;;
    *) root_argument="$1"; shift ;;
  esac
done
if ((list_items)) && ((! with_keys)) && ((${#named_mutations[@]})); then
  echo "  ✗ --list-items 带 --item 要同时带 --with-keys（只列、只算点名那几行的 ITEM_PLAN）"
  echo "     → 怎么办：bash .claude/gate.d/crates-mutation-replay.sh --list-items --with-keys --item <行号或变异名> [项目根]；列整表不给 --item"
  exit 2
fi
if ((with_keys)) && ((! list_items)); then
  echo "  ✗ --with-keys 要和 --list-items 一起给"
  echo "     → 怎么办：bash .claude/gate.d/crates-mutation-replay.sh --list-items --with-keys [项目根]"
  exit 2
fi
ROOT="${root_argument:-$STAGE_REPOSITORY}"
cd "$ROOT" 2>/dev/null || exit 2
MUTATION_TABLE="crates/mutations.tsv"

# 临时文件的清理只有这一处：--item 的选行文件与本次未跑的记录文件放在同一个 EXIT trap 里删；格在子 shell 里跑、各自要设一次同一个 trap，
# 单设一个 trap 会盖掉另一个（改前 --item 的选行文件就靠父进程那一个 trap，格里再设一个就是两个各管一半）。
gate_temporary_files_remove() {
  local leftover=()
  if [[ -n "${not_run_record:-}" ]]; then leftover+=("$not_run_record"); fi
  if [[ -n "${item_selection_file:-}" ]]; then leftover+=("$item_selection_file"); fi
  if ((${#leftover[@]})); then rm -f -- "${leftover[@]}"; fi
  if [[ -n "${prune_server_build_directory:-}" ]]; then rm -rf -- "${prune_server_build_directory:?}"; fi
  if [[ -n "${sample_tools_directory:-}" ]]; then rm -rf -- "${sample_tools_directory:?}"; fi
}
# prune_server_binary_for_mutation_runs：变异复跑起临时 prune-server 用的二进制交下去（文件头「崩溃判定」）：PRUNE_SERVER_BINARY 设了核它是可执行文件的绝对路径；
# 没设在 prune_server_build_directory（这一格自己的临时目录，gate_temporary_files_remove 收尾删）里编一份、导出 PRUNE_SERVER_BINARY。取不到返回 1，为什么与出路已打出
prune_server_build_directory=""
prune_server_binary_for_mutation_runs() {
  if [[ -n "${PRUNE_SERVER_BINARY:-}" ]]; then
    if [[ "$PRUNE_SERVER_BINARY" != /* || ! -x "$PRUNE_SERVER_BINARY" ]]; then
      echo "  ✗ PRUNE_SERVER_BINARY=$PRUNE_SERVER_BINARY 不是可执行文件的绝对路径：变异复跑起不了各自的临时 prune-server，一行都没判"
      echo "     → 怎么办：改成编好的 prune-server 的绝对路径，或 unset 它让这一格自己编一份（文件头「崩溃判定」）。"
      return 1
    fi
    echo "  · 变异复跑的临时 prune-server 用 PRUNE_SERVER_BINARY=$PRUNE_SERVER_BINARY"
    return 0
  fi
  prune_server_build_directory="$(mktemp -d "${TMPDIR:-/tmp}/crates-mutation-prune-server.XXXXXX")" || {
    echo "  ✗ 建不了编 prune-server 的临时目录，一行都没判"
    echo "     → 怎么办：看 \${TMPDIR:-/tmp} 满没满、有没有写权限，修好再跑（不要自己指二进制，这一格自己编一份）。"
    return 1
  }
  if ! python3 "$STAGE_REPOSITORY/scripts/hosts/rocksdb-prebuilt.py" entry --root "$STAGE_REPOSITORY" >"$prune_server_build_directory/rocksdb-entry.txt" 2>&1; then
    sed 's/^/     /' "$prune_server_build_directory/rocksdb-entry.txt" | tail -n 8
    echo "  ✗ 编 prune-server 之前 RocksDB 预编库的取值没判过（上面是 rocksdb-prebuilt.py entry 的输出尾部），没编、一行都没判"
    echo "     → 怎么办：照上面那几行的出路修（开新会话，或命令前 eval \"\$(python3 scripts/hosts/rocksdb-prebuilt.py env)\"），再跑。"
    return 1
  fi
  if ! CARGO_TARGET_DIR="$prune_server_build_directory/target" \
      cargo build --release --offline --manifest-path "$STAGE_REPOSITORY/prune-server/Cargo.toml" --bin prune-server >"$prune_server_build_directory/build.log" 2>&1 \
      || [[ ! -x "$prune_server_build_directory/target/release/prune-server" ]]; then
    echo "  ✗ 编不出 prune-server（cargo build --release --offline --manifest-path prune-server/Cargo.toml --bin prune-server）：$(tail -n 3 "$prune_server_build_directory/build.log" | tr '\n' ' ' | cut -c1-300)"
    echo "     → 怎么办：照报错修 prune-server/（归工具实现员），再跑这一格。"
    return 1
  fi
  PRUNE_SERVER_BINARY="$prune_server_build_directory/target/release/prune-server"
  export PRUNE_SERVER_BINARY
  echo "  · 变异复跑的临时 prune-server 用这一格编的 $PRUNE_SERVER_BINARY（跑完删）"
}
item_selection_file=""
if ((${#named_mutations[@]})); then
  if [[ -n "${GATE_MUTATION_ROW_SELECTION:-}" ]]; then
    echo "  ✗ --item 与环境里的 GATE_MUTATION_ROW_SELECTION 一起给了：两份选行不知道听哪一份"
    echo "     → 怎么办：手跑只用 --item；GATE_MUTATION_ROW_SELECTION 只由分片驱动 scripts/mutation/mutation-shard-run.sh 的 shard 那一步设。"
    exit 2
  fi
  [[ -f "$MUTATION_TABLE" ]] || { echo "  ✗ 没有 $ROOT/$MUTATION_TABLE：--item 点名的变异无从找行"; echo "     → 怎么办：在项目根跑，或把项目根作为参数传进来。"; exit 1; }
  item_selection_file="$(mktemp)" || { echo "  ✗ 建不了选行文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
  trap gate_temporary_files_remove EXIT
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
    echo "     → 怎么办：写行号或变异名，照下面列的（bash .claude/gate.d/crates-mutation-replay.sh --list-items 打的同一份）原样写："
    python3 "$STAGE_REPOSITORY/scripts/mutation/crates-mutation-rows.py" --list-items "$ROOT" | sed 's/^/       /'
    exit 2
  fi
  GATE_MUTATION_ROW_SELECTION="$item_selection_file"
  export GATE_MUTATION_ROW_SELECTION
fi

if ((list_items)); then
  # 一行一条变异（# 起头的行与空行不算）：变异名<制表符>第几行、改哪个文件；只有共用模块里一份。--with-keys 另给每项一行 ITEM_PLAN；
  # 带 --item 的只列、只算点名那几行：选行文件（上面 --item 换成的）交给 --rows。不 exec：退出时 EXIT trap 删选行文件
  list_items_arguments=(--list-items)
  if ((with_keys)); then list_items_arguments+=(--with-keys); fi
  if [[ -n "$item_selection_file" ]]; then list_items_arguments+=(--rows "$item_selection_file"); fi
  python3 "$STAGE_REPOSITORY/scripts/mutation/crates-mutation-rows.py" "${list_items_arguments[@]}" "$ROOT"
  exit $?
fi

# ── crates-mutation-rows 格 ──
cell_crates_mutation_rows() {
# 整道没有复用标记：复用落在每一行的行键上（共用模块 run_stage 逐行查判定库），这里不问 stage-must-run.sh、不写整道标记。
# 前提读的是本阶段所在那一份仓的登记表：判别力样本的目录里没有登记表，前提照样按真表判。
python3 "$STAGE_REPOSITORY/scripts/gate/admission.py" gate-preconditions "$STAGE_REPOSITORY" "$(basename "$0")" || exit 1
# 编译目录不在这里算（文件头「编译目录」）：GATE_MUTATION_TARGET_DIR 设了照原样传，没设由共用模块在这一趟的运行目录里建、跑完删
MUTATION_ROWS_MODULE="$STAGE_REPOSITORY/scripts/mutation/crates-mutation-rows.py"
if [[ ! -f "$MUTATION_ROWS_MODULE" ]]; then
  echo "  ✗ 找不到 $MUTATION_ROWS_MODULE：这一道的判法、派活与判定库都在它里面，一条都判不了"
  echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一道按红记。"
  exit 1
fi
# 判法都在共用模块的 run_stage 里，这里 import 它、交出被判的仓根。
# 模块按名字登记进 sys.modules：进程池的工作进程靠这个名字找回要跑的函数。
# 本次未跑的那几条，模块逐条写进这一格自己的记录，跑完转进 $GATE_NOT_RUN_FILE。
not_run_record="$(mktemp)" || { echo "  ✗ 建不了本次未跑的记录文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
trap gate_temporary_files_remove EXIT
prune_server_binary_for_mutation_runs || exit 1
GATE_MUTATION_NOT_RUN_RECORD="$not_run_record"
export GATE_MUTATION_NOT_RUN_RECORD
# 样本档（文件头「样本」）：样本根上有 .crates-mutation-replay-sample-tools/、这个根不是本阶段所在的仓、也没设 GATE_STAGED_FROM 时，起 cargo 的是共用模块现写的假 cargo 与假 llvm 工具。
# prune-server 的二进制上面已经取好（PRUNE_SERVER_BINARY），PATH 上换了假 cargo 也不影响它
sample_tools_directory=""
if [[ -d "$PWD/.crates-mutation-replay-sample-tools" && -z "${GATE_STAGED_FROM:-}" && "$(pwd -P)" != "$(cd "$STAGE_REPOSITORY" && pwd -P)" ]]; then
  sample_tools_directory="$(mktemp -d "${TMPDIR:-/tmp}/crates-mutation-sample-tools.XXXXXX")" || {
    echo "  ✗ 建不了放样本档假工具的临时目录，一行都没判"
    echo "     → 怎么办：看 \${TMPDIR:-/tmp} 满没满、有没有写权限，修好再跑。"
    exit 1
  }
  python3 "$MUTATION_ROWS_MODULE" sample-tools "$sample_tools_directory" || exit 1
  PATH="$sample_tools_directory:$PATH"
  GATE_MUTATION_LLVM_TOOLS="$sample_tools_directory"
  export PATH GATE_MUTATION_LLVM_TOOLS
  echo "  ! 样本档：$PWD/.crates-mutation-replay-sample-tools 在，起 cargo 的是 $sample_tools_directory 里的假 cargo 与假 llvm 工具（不编译）；这一趟不算真跑过变异表"
fi
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
# 分片驱动起的各片（GATE_MUTATION_ROW_SELECTION 不是 --item 换成的）不转：分出去的行由之后整张那一趟再判、再报
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

# 分片驱动起的各片不经格汇总，照驱动的约定退（文件头 GATE_MUTATION_ROW_SELECTION 那一条）
if [[ -n "${GATE_MUTATION_ROW_SELECTION:-}" && -z "$item_selection_file" ]]; then
  ( cell_crates_mutation_rows )
  exit $?
fi
stage_cells_run "$ROOT"
