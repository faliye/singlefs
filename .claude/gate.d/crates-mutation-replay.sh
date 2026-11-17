#!/usr/bin/env bash
# task-server-shards: *   经统一任务服务交的一律分片：服务拆成计划、各片、合并（下面「分片」）；--selftest、--list-items、--help、-h 服务不分
# 显卡（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）：卡只归 GPU 批处理验证服务，这一道不领卡、不挑卡、不清场，文件头不写要卡的声明；
#   环境里有 GPU_CARDS_BY_PRIORITY 就原样留给 scripts/mutation/crates-mutation-rows.py 起的 cargo，要卡的格怎么跑照它的文件头。
# task-server-compiles: yes
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
#   task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh [--item …] [项目根]
#                                      整张表（或点名的几行）：服务照文件头第三行分片，计划、各片、合并三步各起这一道一次（下面「分片」）
#   bash .claude/gate.d/crates-mutation-replay.sh [项目根]                                不是服务分片起的（gate.sh 整轮、手跑）：只核账，库里没有键相同记录的行不现判、整道判红（下面「分片」）
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
#   GATE_MUTATION_ROW_SELECTION=<文件>（一行一个变异表行号，只由分片那一套 scripts/mutation/mutation-shard-run.sh 的 shard 那一步设）：只判这些行（它们所在的基线批照跑），照常写判定库记录，
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
#   先按编译配置（cargo 参数在 `--` 之前的那几个词）、再按 crate 与改的文件归堆派（增量编译的命中率才稳，少在测试目标之间来回重编），活动态领（等分之下拿到几十秒重测试的那一片会独自拖住整道），上一次用时 ≥ GATE_MUTATION_LONG_ROW_SECONDS（默认 300）秒的先派。
#   工作进程数取 ① 每台默认重活线程数（设了 GATE_MUTATION_WORKERS 就顶掉 ①）、② 这一趟要现跑的条数、③ 编译目录所在的盘放得下几个工作进程的两份编译目录（下面「磁盘余量」）、
#   ④ 这一台能给的内存（仓根多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项，减去起这一道的父进程自己的用量）÷ 那个编译并行度下每个工作进程开满时的实测峰值的小者、至少 1：
#   按核开满、不超这一台的内存，不按服务给的上限收（这一台那一项取不到时除的是这一条拿到的上限）。这一条拿到的上限（TASK_SERVER_MEMORY_CAP_BYTES，没有读多机配置那一项，
#   再没有读自己 scope 的 memory.max；memory.max 是有限值就是硬顶）另当硬顶：要开的工作进程 × 峰值加父进程自己装不下，派活之前整道按撞内存上限退 250（见「上限、限时与卡住」），
#   服务放大上限重排，开满时量到的峰值记下之后下一次照它给。
#   每个工作进程的峰值由门禁自己量：每个工作进程里一条采样线程量它整棵进程树（cargo、rustc、测试、临时 prune-server）的 RssAnon + RssShmem + VmPTE 加总，
#   按这一格点名的包与编译并行度记进多机配置 NFS_SHARED_ROOTS 点名的共享根下的运行记录（worker-peaks/<包>/<编译并行度>/，两台同一份；判样本根时不记）；④ 排在最后、按 ①②③ 的小者当天花板，
#   每试一个进程数 n 就照 n 算编译并行度、取这一趟要现跑的行点名的包在那个编译并行度下最新几份实测的最大，满足 n × 峰值 ≤ 除的那个数的最大 n 才开；只有更低编译并行度的实测（或记录读不了）取保守值 = 除的那个数本身，只开 1 个；
#   从没量过的包不进这一项，派活时它的格单独跑、量到峰值之后同时在跑的格数按 ⌊这一条拿到的上限 ÷ 量到的峰值⌋ 放开（crates-mutation-rows.py 文件头「内存上限」的「探针」；判样本根时照保守值只开 1 个）。
#   判法在 worker_count_for，内存那一项在 memory_bound_for（crates-mutation-rows.py 文件头「编译并行度」「内存上限」，口径与内存包装记的「回收不掉的用量」差在哪也写在那里），
#   头一行与收尾一行写上限、每个工作进程的峰值与取法、个数与合计；弄坏开关 GATE_MUTATION_BREAK=memory-cap-ignored、memory-max-not-hard-cap（取到变量或多机配置的值就不拿 memory.max 当硬顶）、
#   worker-peaks-unkeyed-by-package、worker-peaks-unkeyed-by-jobs、worker-peaks-window-unbounded、jobs-feedback-ignored、unmeasured-worker-peak-guessed、unreadable-worker-peaks-guessed、worker-tree-self-only、worker-peaks-unrecorded、
#   sample-root-peaks-recorded（样本根上照样写峰值记录）、new-package-run-single-worker（从没量过的包照保守值整趟只开 1 个）、
#   unmeasured-package-not-probed-alone（没有实测的包的格不单独跑），
#   自证在 crates-mutation-rows.py --selftest（selftest_memory_cells、selftest_worker_peak_cells）；本道样本不判这一条。
#   判完的格每条立刻原子写进判定库（工作进程里 Library.write：同目录临时文件、fsync、改名），整组被硬杀之后重跑同一条命令，判过的格照键复用、只现判剩下的；
#   自证 selftest_killed_run_resume_cell 起子进程跑到一半对整组 SIGKILL 再重跑；弄坏开关 GATE_MUTATION_BREAK=records-written-at-end（记录攒到进程池收尾才写）下这一格必须判错。
#   重跑拿不到复用的另一个原因：crates-mutation-rows.py 里任何一处模块级赋值、import、类头、带缺省值的 def 头（连同自证用的 FAKE_CARGO 这类字符串常量）改了，判法单元指纹就变，整库的行键跟着全变（文件头「判法单元」），不是没写。
#   线程数按工作进程数压：每个 cargo 的编译并行度 CARGO_BUILD_JOBS 取 max(1, 每台默认重活线程数 ÷ 工作进程数)（GATE_MUTATION_CARGO_JOBS 顶掉它，要正整数；与 ④ 试进程数时同一个函数 cargo_jobs_for）。
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
# 编译目录（自己用完自己删，不跨轮复用）：GATE_MUTATION_TARGET_DIR 设了用它（调用方给的，跑完不删）；没设而环境里给了 CARGO_TARGET_DIR
#   （文件头的会编声明「# task-server-compiles: yes」让统一任务服务给这一次尝试一份）用它底下的 singlefs-crates-mutation-target，同样跑完不删；都没设时共用模块在这一趟自己的运行目录
#   （$TMPDIR 下的 singlefs-mutation-replay-*）里建 build/singlefs-crates-mutation-target，分片各加 -w<片号>、记覆盖的再加 -coverage、先编一份依赖的 -seed 几份都在那个 build 里，
#   正常结束、判红、被 TERM / HUP 停都删掉（删之前核没有进程开着里面的文件，有就不删、整道判红）。
#   只在 crates-mutation-rows.py 的 default_shared_target_directory 与 run_pool 里算，这里不另算（它文件头「编译目录」）。
#   片号锁、调用方给的编译目录上多出来的 -w<片号> 回收照旧（同一段）；自证在 crates-mutation-rows.py --selftest，
#   弄坏开关 GATE_MUTATION_BREAK=extratargetskept、inusetargetremoved、targetdefaulttmp、runtargetkept、runtargetopenfilesunchecked（本道样本不判这一条）。
#   先编一份依赖（crates-mutation-rows.py 文件头「先编一份依赖」）：开 2 个以上工作进程、要现跑的行选的包够得着外部依赖时，派活之前在 <共用目录>-seed（插桩的 -seed-coverage，
#   源码副本 -seed-source 跨轮留着、按内容同步，锁 -seed.lock）里按组各编一次，工作进程起来时只把外部依赖的产物拷进自己的两份编译目录，工作区的包照旧各编各的
#   （编出来的二进制里编死了 CARGO_MANIFEST_DIR，拷过去 cargo 照样判 Fresh、跑的时候读先编那一份的文件）；回收不碰这几份。先编那一份里摸过的包有产物是 fresh 整道判红。
#   自证与弄坏开关 compile-per-worker、workspaceunitsshared、seedmtimekept、seedfreshunchecked 在 crates-mutation-rows.py --selftest；本道样本不判这一条。
# 调试信息与 release 增量：这一道起的每条 cargo 都带 CARGO_PROFILE_{DEV,TEST,RELEASE,BENCH}_DEBUG=false（cargo test 用 test 档、承 dev；cargo test --release 用 release 档）
#   与 CARGO_PROFILE_RELEASE_INCREMENTAL=true，顶掉仓根 Cargo.toml 的 [profile.release] 与调用方的同名变量；编译档的 debug、incremental 不进行键，别的构建档取值照进
#   （crates-mutation-rows.py 文件头「调试信息与 release 增量」）。
#   自证与弄坏开关 debuginfo-kept、release-incremental-unset、profile-debug-keyed、profile-settings-unkeyed 在 crates-mutation-rows.py --selftest；本道样本不判这一条。
#   片号锁的自证同在那里：弄坏开关 GATE_MUTATION_BREAK=local-shard-numbering（两次并发调用领到的片号重叠）那一格必须判错
#   （直接起自证时同义名 CRATES_MUTATION_ROWS_BREAK）；本道样本不判这一条。
# 崩溃判定（crates-mutation-rows.py 文件头「变异复跑的崩溃判定」）：变异行的判定照行键经 prune-server 写进真服务里那一份判定库（下面「复用」）；
#   起 cargo test 真跑测试的每一次（基线的批、变异、记覆盖的插桩跑）共用模块另起一份它自己的临时 prune-server，走判定库的 验证包用例在变异版上判出的崩溃结论
#   只落进那一份、跑完停服务删目录，不读不写真服务里的崩溃判定；这里不设库的变量、不建本地库。
#   临时服务的二进制由这一格交下去（prune_server_binary_for_mutation_runs）：PRUNE_SERVER_BINARY 设了（绝对路径、可执行）照原样交，不是的这一格判红、一条都不判；
#   没设先照这一道所在仓的 scripts/hosts/rocksdb-prebuilt.py entry 判预编库的取值，再在这一格自己的临时目录里编一份（直接起，线程是服务套的 capped.sh 设好的变量，
#   cargo build --release --offline --manifest-path <这一道所在的仓>/prune-server/Cargo.toml --bin prune-server，与 code-tooling.sh 的 code_tooling_prune_server_binary 同一种编法），
#   设成 PRUNE_SERVER_BINARY 交给共用模块（它认这个变量、不再自己编），编译目录跑完删；判不过、编不出这一格判红、一条都不判。
#   没设时先取这一台的存放目录（prune_server_store_directory：~/.local/share/singlefs/prune-server-binaries，每台各一份、不进共享根；SINGLEFS_PRUNE_SERVER_STORE 只给自证换目录）里按键存下的那一份：
#   键 = 这一道所在仓的 prune-server/ 源码（不含 target/）、仓根 .cargo/config.toml、rustc 与 cargo 的版本、影响编出来什么的环境变量（prune_server_store_key）的 sha256 前 24 位；
#   有就用、不编、不判 RocksDB 预编库的取值（不编 cargo；片开跑时 mutation-shard-run.sh 已判过）；没有就持锁（同一台上几片同时起只一片编、别的等它编完取它）、判预编库的取值、编、存进去（prune_server_store_install），之后只留最新的 4 个键目录。
#   存不进去、HOME 没设、弄坏开关 GATE_MUTATION_BREAK=prune-server-store-ignored 都退回每一片自己编、跑完删。这一条没有门禁样本：样本经外层包装拿 PRUNE_SERVER_BINARY，不走这一步；
#   证法是真跑两片（第二片的任务日志里有「没编」那一句），弄坏开关下第二片照旧编。
#   --list-items 不起服务、不取二进制。本道样本不判这一条（样本照样经这一步取二进制）。
# 临时目录：每条变异与每批基线各一个 TMPDIR，结束时不论退出码（含被信号杀掉的测试进程）都整个删掉：被杀的测试 Drop 守卫不跑，镜像会堆在里面；
#   全部判完总目录下还剩哪一条的目录，整道判红。
#
# 上限、限时与卡住（records/2026-10-30-编译绕过内存总上限.md）：每条 cargo test 直接起，不套内存包装、不设上限、不限时；
#   这一道整条经 task-server/task submit 交，内存上限由统一任务服务按记录给、撞了放大重排，死循环的变异由服务按「日志与 CPU 都不涨」停整条任务。
#   撞内存上限（任务 7498 那一类）：服务给的上限装不下要开的工作进程量到的峰值时，共用模块派活之前退 250（crates-mutation-rows.py 文件头「内存上限」的「撞内存上限」），
#   这一格照 250 退、整道也照 250 退（格汇总把红格并成 1，这里另记一个标记、汇总之后照它退 250），服务照错误类型表 memory-cap-hit 那一行加额度重排；
#   分片的各片（GATE_MUTATION_ROW_SELECTION）不经格汇总，模块的 250 原样交出去。
#   底座读数：共用模块往 stderr 打一行 MUTATION_BASE_READING（父进程自己的用量、先编一份依赖那一步父进程整棵树的峰值），读法在它文件头「内存上限」的「底座读数」。
#   不判「内存撞顶」「超时」「显存不够」：显存不够而红的照抓到记；GPU 设备丢失（点名的测试自己那一段输出里、或所有段之外有行首 GPU_DEVICE_LOST 的行）另算，记本次未跑（下面「本次未跑」）。判法在 crates-mutation-rows.py 文件头同名一段。
#
# 基线（判法见 crates-mutation-rows.py 文件头「基线」）：点名的测试在没改坏的副本上本来就红时，改坏之后照样红、记「抓到」什么都没证明，
#   所以派变异之前先跑基线，不是恰好一个 ok 的行记「基线不绿」、整道判红。「计数：」那一行的格式不动（mutation-triage 照栏读），基线另起一行「基线：…」报。
# 复用：每行一条判定，行键是被改坏的函数单元（种类、指纹；所在文件与限定名不进键）、原文、替换文、点名测试执行过的单元集合指纹、判「抓没抓到」的判法单元指纹、工具链、解析出的依赖版本，
#   不带变异名；没有管整库的版本号、没有版本目录；一份库，两台判出的都经 prune-server 写进同一份（下面「分片」）。
#   库名固定（mutation-units 族，种类 singlefs-crates-mutation-library），库根不进键，派到哪台、判的是哪棵树都写同一份；取不到库名、服务连不上整道退出（连不上退 75），不现判；
#   成功句末尾点名库在哪。判出的不是「抓到」（没红、无效、基线不绿……）照行键盖掉原有的「抓到」，下一趟现判。
#   一行的基线结局不随这一趟选了哪几行而变（只看这一行自己的过滤词选中的结果行）：点名单跑与整张表对同一行判同一个结论。
#   判法单元连同 judge_row、judge_batch、run_cargo_command 这几段收闭包；逐样与弄坏开关在 crates-mutation-rows.py 文件头「判法单元」「行键」「判定库」「复用判据」「基线」。
#   没有整道全绿标记，不问 stage-must-run.sh。点名测试那一段这里取它的单元闭包键（入口 = 那条测试函数，比执行过的单元宽，不漏）；
#   执行过的单元另记在覆盖记录里（-C instrument-coverage、llvm-profdata、llvm-cov，工具不对整道判红），基线 ok 那一条按它复用、不按整树快照。
#   键对不上的现判，按栏报数（行新加、判法单元变了、被改坏的单元变了、点名测试的闭包变了、原文或替换文变了、工具链或依赖变了、上次没判成抓到、一律现判），
#   stderr 逐行写变了哪几个单元；算不出键的那一行拒算（点名的测试认不出单元、闭包里有认不出的形态），每趟照跑。成功句末尾报「现判 N、复用 M、拒算 R、本次未跑 K」。
#   撞键就拒算：表里两行以上算出同一把行键（两份内容相同的函数各改一处、点名同一条测试），这几行拒算、照跑、不复用，
#   stderr 逐行与成功句末尾报「拒算（行键撞键：与第 N 行同键）」；--item 与分片只判选中的行时，撞键照整张表判。
#   判法与弄坏开关 row-key-collision-unrefused 在 crates-mutation-rows.py 文件头「行键」，自证在它的 --selftest；本道样本不判这一条。
#   键、库的存放、复用判据写在 crates-mutation-rows.py 文件头「复用」「判法单元」「行键」「判定库」「复用判据」。只读看这一趟会怎么判：python3 scripts/mutation/crates-mutation-rows.py plan <根>；
#   --list-items 末行报键对不上的条数。只有「抓到」与基线 ok 复用。
# 分片（一律分片；机器、上限与每片派到哪台归 task-server，这一道自己不连第二台）：文件头第三行「# task-server-shards: *」让服务把经它交的这一道拆成三步，
#   每步都照原命令（连同 --item）起这一道一次，环境里 TASK_SERVER_SHARD_PHASE 说是哪一步（task-server/task_server_lib.py 文件头「分片」）；
#   这一道照它转给 scripts/mutation/mutation-shard-run.sh 的同一步（协议在它文件头）：
#   plan   库里没有键相同判定的行（点名 --item 的只在点名的行里，转成 --only-rows）按内存定片数：每片预计内存（每个工作进程的峰值 × 这一片开的工作进程数 + 父进程）
#          不超过多机配置 SLICE_MEMORY_CAP_BY_HOST 各台里最小那一项的 2/3，片数不超过服务交的 TASK_SERVER_SHARD_MAXIMUM，行少到一片装得下就只一片
#          （mutation-shard-run.sh 文件头「三步」plan 的 ④）；各片之间按上一次现跑的用时均分、都不钉台，计划不报内存量（一片的上限由服务按记录给）；拒算与锚点失效的行不进各片；
#          --list 的一趟写 0 片
#   shard  照 GATE_MUTATION_ROW_SELECTION 起这一道只判自己那一堆（选行模式，不再分），判定经 prune-server 写进同一份库
#   merge  先核各片判的行按此刻的行键在服务里读得到（一片判的行一行都读不到就判红、点名那一片），再在这一步照下面「核账」判整张表（或点名的几行），
#          stdout 是这一道的结论
#   核账（不是服务分片起的这一道、与 merge 那一步，GATE_MUTATION_SHARDED_ONLY=1 交给共用模块）：键相同的「抓到」复用；拒算的（没有键）、复用哨兵抽中的、
#          键相同而上次没判成抓到的照判；库里没有键相同记录的行这一趟不现判、整道判红，出路是经服务交这一道（crates-mutation-rows.py run_stage 里那一段）。
#          gate.sh 整轮里这一道就是核账：判过的照键复用，不重判。样本根（.crates-mutation-replay-sample-tools）不设它，样本照现判。
#   各片的工作进程数照上面「并发」④ 按这一台能给的内存开满。
# 本次未跑（判法在 crates-mutation-rows.py 文件头「本次未跑」）：点名的测试打 NOT_RUN_NO_GPU_ADAPTER 后返回成功的那一条不算抓到、不算没红，整道不因它判红；
#   test= 对不上测试名的（崩溃枚举用例判段不跑时打 test=crash_state_enumeration::<项名>）同样记本次未跑，不记没红（弄坏开关 not-run-test-name-only，
#   自证在 crates-mutation-rows.py --selftest「本次未跑认项名」那一格；本道样本不判这一条）。
#   设备丢失同走这一路：点名的测试 FAILED，而 libtest 给它交出的那一段输出（「---- <全名> stdout ----」）里、或所有段之外（实时打印的部分、「test result:」之后：
#   没带输出捕获的线程打的行，归属不明）有一行行首是 `GPU_DEVICE_LOST ` 的行
#   （`GPU_DEVICE_LOST test=<当前线程名> card=<卡名> reason=<一句>`，test= 可以是 main、unnamed-thread、全名，不比对）的那一条，
#   红的是设备丢失、不是断言，记本次未跑；标记行只在别的用例的那一段里、FAILED 而都没有这一行（含显存不够）照记抓到；点名的测试 ok 而带这一行照 ok 的判法。
#   弄坏开关 device-lost-ignored 与 device-lost 样本在下面；逐样自证在 crates-mutation-rows.py --selftest「设备丢失」那几格。
#   只有本次未跑的这一格退 77。模块写进 GATE_MUTATION_NOT_RUN_RECORD，这里转进 $GATE_NOT_RUN_FILE（分片的各片不转，由合并那一步的核账再报）。
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
#   GATE_MUTATION_BREAK=device-lost-ignored  不认 GPU_DEVICE_LOST 行：点名的测试 FAILED 且自己那一段里、或所有段之外带那一行的也记抓到
#       device-lost（输出里找不到「计数：抓到 2 条、没红 0 条、无效 0 条、锚点失效 0 行（共 4 条）」）
#   GATE_MUTATION_BREAK=sample-service-unchecked  样本根不核在不在临时 prune-server 底下，裸跑照判、判定写进真服务
#       sample-root-bare-red（期望退出 2，实测 1）
#   GATE_MUTATION_BREAK=sample-cargo-log-unset  样本档换上假 cargo 时不设 FAKE_CARGO_LOG（假 cargo 每起一次先往它追加一行，没设就抛 KeyError 退 1，基线一行都绿不了）
#       red（输出里找不到「计数：抓到 0 条、没红 1 条、无效 1 条、锚点失效 0 行（共 4 条）」）、
#       green（期望退出 0，实测 1）、not-run（期望退出 0，实测 1）
#   复用、续跑、只改一行只跑那一行、单元键的每一样（controls 路的 S34、S30、N33、N34、N45，被改坏的单元、点名测试的闭包、V5 那一类常量）、
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
#   点名的测试要上卡的行不另套清场、这一道不领卡（卡归 GPU 批处理验证服务，文件头「显卡」；crates-mutation-rows.py 文件头「带 gpu 特性的格不另套清场」）：
#   上卡的特性是 GPU_TEST_FEATURES 里的两个，验证包的 gpu 与
#   产品 GPU 包 singlefs-background-gpu 真显卡对拍的 tests-on-a-real-gpu；点名的测试没卡时打 NOT_RUN_NO_GPU_ADAPTER 后返回成功，记本次未跑（同上「本次未跑」）。
#   弄坏开关 GATE_MUTATION_BREAK=product-gpu-feature-unquiesced 让 tests-on-a-real-gpu 不当上卡的特性认，crates-mutation-rows.py --selftest 里
#   「产品 GPU 包（--features tests-on-a-real-gpu）的行：…」两格必须判错；本道样本不判这一条（样本用假 cargo、没有显卡）。
#   开 GPU 时崩溃判段那几行补 gpu 特性（配 GPU 必用 GPU，不降级；crates-mutation-rows.py 文件头同名一段）：多机配置写了 ENABLE_GPU=1 时，
#   点名测试的单元闭包够得着 crash_state_enumeration 或 crash_amplification 的选路函数、cargo 参数没开 gpu 的行，这一趟补上 gpu 特性起（照上面不另套清场、交多机配置），
#   键照补了之后的特性算；变异表那一行不动，stderr 一行点名补了哪几行。弄坏开关 GATE_MUTATION_BREAK=gpu-route-feature-unadded（不补），
#   自证在 crates-mutation-rows.py --selftest「开 GPU 时点名测试走崩溃判段的那一行补 gpu 特性」那一格；本道样本不判这一条（样本 crate 里没有那两处选路函数）。
#   带 gpu 特性、真跑测试的 cargo test（基线的批、变异、记覆盖的插桩跑）的 libtest 参数收尾带 --test-threads 1（.claude/rules/verification.md 里 GPU 那一条），
#   点名的测试目标全在放开名单里的带每台默认重活线程数（放开名单是 crates-mutation-rows.py 的 GPU_TEST_THREADS_RELEASED_TARGETS 加验证包登记的只经卡池取卡的测试目标，gpu_released_test_targets 照被判那棵树现读）；行里自己已写 --test-threads 的不重复补；线程参数不进行键
#   （crates-mutation-rows.py 文件头「带 gpu 特性的格测试参数收尾带 --test-threads」）。弄坏开关 GATE_MUTATION_BREAK=gpu-test-threads-unadded（不补），
#   自证在 crates-mutation-rows.py --selftest「带 gpu 的行收尾带 --test-threads 1、放开名单里的目标带登记值、不带 gpu 的行不带」那一格；本道样本不判这一条（样本用假 cargo、没有显卡）。
# 按项机读行（统一任务服务按项先算后排读它；一项 = 变异名，同名的几行合成一项；格式与算法在 crates-mutation-rows.py 文件头「按项机读行」）：
#   --list-items --with-keys   stdout，每项一行（次序是变异表的次序，行数等于逐行列出的项数，条数之和等于末行合计的数）：
#       ITEM_PLAN item=<项名> to_judge=<要现判的条数：键对不上与拒算> reusable=<能复用的条数：键对得上>
#   --list-items --with-keys --item …   stdout，ITEM_PLAN 之后另打（整表不打）：每项一行、次序同变异表，末了一行收尾：
#       ITEM_CARD item=<项名> card=<0|1>        1 是点名的行里有一行要卡或没证明不要卡，0 是证明了不要卡
#       ITEM_CARD_DONE items=<项数>
#     判法：点名的行里有一行 cargo 参数开了上卡的特性（crates-mutation-rows.py 的 gpu_test_run）就整条都是 1；点名的行都没开、而多机配置写了 ENABLE_GPU=1 时再算
#     开 GPU 时崩溃判段那几行补 gpu 特性的行（plan_context 的 gpu_route_lines，上面「开 GPU 时崩溃判段那几行补 gpu 特性」），补了的那项是 1，其余是 0。
#     算失败（读不了表、选行不对、键与闭包算不出）一行都不打、原因在 stderr、退出码仍照 ITEM_PLAN 那一步；统一任务服务读不到这些行就保持要卡，
#     读到每一项都是 0 才把这条任务改成不要卡、不分卡（task-server/task_server_lib.py 文件头「资源记录」预计用时末段）。
#   真跑（整张表、--item、分片的各片同样）  stderr，每项跑完一行（带挂钟，所以不进 stdout）：
#       ITEM_DONE item=<项名> seconds=<这一项自己的挂钟秒数> judged=<现判条数> reused=<复用条数> exit=<这一项的退出码：0 抓到、77 本次未跑、1 其余>
#   弄坏开关 GATE_MUTATION_BREAK=item-plan-reusable-zero、item-plan-selection-ignored（带 --item 也算整表）、item-done-reused-silent、item-done-seconds-whole-run，
#   自证在 crates-mutation-rows.py --selftest（selftest_item_lines_cells 五格）；本道样本不判这几条。
# 样本（fixtures/crates-mutation-replay.sh/；各放 .gate-cells 点名 crates-mutation-rows，setup.sh 在样本目录里 git init；
#   样本根放 .crates-mutation-replay-sample-tools/（标记，里面一个说明文件）：起 cargo 的是 crates-mutation-rows.py sample-tools 现写的假 cargo 与假 llvm 工具，
#   不编译；样本 crate 里 `// TEST <名字> NEEDS|READS|FILES|NOTRUN …` 与 `// COMPILE-ERROR-IF <原文>` 几行说每条测试、每处替换文的红绿（写法见 FAKE_CARGO 头上的说明）；
#   假 cargo 每起一次先往 $FAKE_CARGO_LOG 追加一行（FAKE_CARGO 头上那一句，没设它就退 1），格里把它设成放假工具那个临时目录里的 cargo-calls.log，随那个目录删；
#   只在样本根上进样本档：根不是本阶段所在的仓、也没设 GATE_STAGED_FROM）：
#   red      只改注释的一行（原文不落在任何单元上，拒算）照跑记「没红」；替换文编不过的一行记「无效」；
#            点名的测试读 fixture-data/answer.txt（样本仓里有、不在拷贝范围里：测试里路径逐段拼，拷贝范围的闭包只认 ../ 起头的串字面量，认不出它）的一行与只写名字、两个模块各有一个同名用例的一行都报「基线不绿」
#            （基线那一行报 2 行 ok、2 行不绿）；「临时目录」那一行报 2 条都删掉了；计数行里没红、无效各数各的；
#            只改注释的那一行与同名用例的那一行拒算，stderr 逐行写为什么
#   工作进程数不在样本里判：样本根上的工作进程不写峰值记录（crates-mutation-rows.py 的 worker_peaks_recorded_for），有内存上限时开几个随样本读到的运行记录里
#   有没有实测而变（没有就照「从没量过」只开 1 个），没有内存上限时照条数开；开几个的判法在 crates-mutation-rows.py --selftest（selftest_memory_cells、selftest_worker_peak_cells）。
#   green    五条正常变异照旧红在点名的测试上，其中一条改 litmus/sample.litmus（拷贝范围要含 litmus/），一条替换文空着、删掉 add_twice 里第二次相加；
#            基线那一行报 5 行都 ok，「临时目录」那一行报 5 条都删掉了，stderr 头一行报复用 0 条；假 cargo 与假 llvm 工具记覆盖：
#            5 行 4 条点名测试（两行同点 adds_two_and_three_to_five），「覆盖：」那一行报记了 4 份、没记成 0 份；按行键：5 行各一把键、库是空的，
#            「复用：」那一行报现判 5 条、复用 0 条、拒算 0 条，5 条都在「行新加」那一栏，stderr 逐行写现判的原因
#   not-run  一条点名的测试只打 NOT_RUN_NO_GPU_ADAPTER 那一行就返回、一条正常变异：报本次未跑 1 条、抓到 1 条、这一格绿，成功句末尾报本次未跑 1 条
#   device-lost  四条变异：一条正常；一条点名的测试被改坏之后 FAILED、它自己那一段输出里有 GPU_DEVICE_LOST 行（test=main；报本次未跑、点名卡名与那一句）；
#            一条点名的测试被改坏之后 FAILED、它自己那一段只有断言、GPU_DEVICE_LOST 行在别的用例的那一段里（照记抓到）；
#            一条点名的测试被改坏之后 FAILED、它自己那一段只有断言、GPU_DEVICE_LOST 行在所有段之外（实时打印的部分；报本次未跑、写明在段外）：
#            计数行抓到 2 条、没红 0 条，成功句报抓到 2 条、本次未跑 2 条，这一格绿
#   sample-root-bare-red  样本根放 .sample-environment-bare（造出不在临时 prune-server 底下的样子）：退 2、一行都不判，点名没有临时 prune-server 并给包装写法
#            （下面「样本根只在临时 prune-server 底下判」；red、green、not-run 都在临时服务底下被喂，它们就是这一判的绿侧）
# 样本根只在临时 prune-server 底下判（mutation_sample_service_check，在格汇总之前）：样本根上的判定库没有命名空间，不在临时服务底下就写进真服务，
#   prune-server 的二进制也不由包装交下来（PRUNE_SERVER_BINARY），这一格就在样本根里现编一份（cwd 是样本根，读不到仓根 .cargo/config.toml 的 BINDGEN_EXTRA_CLANG_ARGS，
#   librocksdb-sys 编不过，整格在「编不出 prune-server」那一步红、一行都没判）。所以样本根上开跑先调客户端的 selftest_service_problem（只看环境、不连服务），
#   不在临时服务底下退 2，拒绝说明给整条写法：python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh .claude/gate.d --item crates-mutation-replay.sh。
#   真仓、--staged 的临时树（GATE_STAGED_FROM）与 --list-items 不判这一条。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
if ! declare -F preflight >/dev/null; then
  echo "  ✗ crates-mutation-replay.sh：source .claude/scripts/preflight.sh 没成（上面一行报了找不到），preflight 没有定义、参数已被清空，拒绝往下跑（不拦的话 --check 与 --list 会变成全跑）"
  echo "     → 怎么办：这棵树缺 .claude/scripts/preflight.sh：建副本时连 .claude/scripts 与 .claude/singlefs-ai-sop 一起拷（scripts/hosts/minimal-copy.py 逐条加 --with .claude/scripts=… 与 --with .claude/singlefs-ai-sop=…），或回主工作区跑"
  exit 2
fi
STAGE_REPOSITORY="$(cd "$(dirname "$0")/../.." && pwd)"
source "$(dirname "${BASH_SOURCE[0]}")/lib/stage-cells.sh"
stage_cell crates-mutation-rows cell_crates_mutation_rows "crates/mutations.tsv 每条改坏一处、点名的测试都红，基线在没改坏的副本上绿" \
  "照上面逐条的判定改：没红的补测试或改变异，锚点失效的改原文，基线不绿的先让点名的测试在没改坏的副本上绿；只重跑某几行：task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --item <变异名>（服务分片判）；报「库里没有键相同的记录」的，经服务交这一道（文件头「分片」）"
# 分片（文件头「分片」）：一片起的这一道（选行模式）不再分；--list 的一趟服务照样分，计划写 0 片，合并那一步照常打格名
shard_phase="${TASK_SERVER_SHARD_PHASE:-}"
if [[ -n "${GATE_MUTATION_ROW_SELECTION:-}" ]]; then shard_phase=""; fi
if [[ "$shard_phase" == plan ]] && grep -qx -e '--list' <<<"$(printf '%s\n' "$@")"; then
  python3 -c 'import json, os, sys; temporary = os.path.join(sys.argv[1], "plan.json.partial"); json.dump({"shards": [], "note": "--list 不判，不分"}, open(temporary, "w")); os.replace(temporary, os.path.join(sys.argv[1], "plan.json"))' \
    "${TASK_SERVER_SHARD_PLAN_DIRECTORY:?计划那一步要有 TASK_SERVER_SHARD_PLAN_DIRECTORY}" || exit 1
  echo "  · --list 不判：计划写 0 片，合并那一步打格名"
  exit 0
fi
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
# 单设一个 trap 会盖掉另一个。
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
prune_server_store_lock=""
# 这一台存编好的 prune-server 的目录（每台各一份、不进共享根：两台的系统库版本不一定相同，二进制不互用）；SINGLEFS_PRUNE_SERVER_STORE 只给自证换目录；
# HOME 没设、弄坏开关 GATE_MUTATION_BREAK=prune-server-store-ignored 交空（不存不取、每一片照旧自己编）
prune_server_store_directory() {
  [[ ",${GATE_MUTATION_BREAK:-}," == *",prune-server-store-ignored,"* ]] && return 0
  if [[ -n "${SINGLEFS_PRUNE_SERVER_STORE:-}" ]]; then
    printf '%s\n' "$SINGLEFS_PRUNE_SERVER_STORE"
  elif [[ -n "${HOME:-}" ]]; then
    printf '%s\n' "$HOME/.local/share/singlefs/prune-server-binaries"
  fi
  return 0
}
# 键：这一道所在仓的 prune-server/ 源码（不含 target/）、仓根 .cargo/config.toml、rustc 与 cargo 的版本、影响编出来什么的环境变量（RUSTFLAGS、CARGO_PROFILE_*、RocksDB 预编库的取值这一类）合起来的 sha256 前 24 位
prune_server_store_key() {
  (
    cd "$STAGE_REPOSITORY" || exit 1
    {
      find prune-server -path prune-server/target -prune -o -type f -exec sha256sum {} + | LC_ALL=C sort -k2
      if [[ -f .cargo/config.toml ]]; then sha256sum .cargo/config.toml; fi
      rustc -vV
      cargo -V
      env | LC_ALL=C sort | grep -E '^(RUSTFLAGS|CARGO_ENCODED_RUSTFLAGS|CARGO_PROFILE_[A-Z_]*|CARGO_BUILD_TARGET|BINDGEN_EXTRA_CLANG_ARGS|ROCKSDB_[A-Z_]*|SNAPPY_[A-Z_]*|CC|CXX|CFLAGS|CXXFLAGS)=' || true
    } | sha256sum | cut -c1-24
  )
}
# <存放目录> <键> <编好的二进制>：拷进存放目录里的临时目录、整个改名放到 <存放目录>/<键>/（别的进程先放好了就取先放好的）；放好之后键目录只留修改时刻最新的 4 个
prune_server_store_install() {
  local store="$1" key="$2" built="$3" partial old
  partial="$(mktemp -d "$store/.partial.XXXXXX")" || return 1
  if ! cp -- "$built" "$partial/prune-server" || ! chmod +x "$partial/prune-server"; then
    rm -rf -- "${partial:?}"
    return 1
  fi
  if ! mv -T -- "$partial" "$store/$key" 2>/dev/null; then
    rm -rf -- "${partial:?}"
    [[ -x "$store/$key/prune-server" ]] || return 1
  fi
  while IFS= read -r old; do
    [[ -n "$old" ]] && rm -rf -- "${old:?}"
  done < <(find "$store" -mindepth 1 -maxdepth 1 -type d -regextype posix-extended -regex '.*/[0-9a-f]{24}' -printf '%T@ %p\n' | sort -rn | tail -n +5 | cut -d' ' -f2-)
  return 0
}
prune_server_store_unlock() {
  if [[ -n "${prune_server_store_lock:-}" ]]; then
    exec {prune_server_store_lock}>&-
    prune_server_store_lock=""
  fi
}
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
  local store key cached
  store="$(prune_server_store_directory)"
  key=""
  if [[ -n "$store" ]]; then
    key="$(prune_server_store_key)" || key=""
  fi
  if [[ -n "$key" ]]; then
    cached="$store/$key/prune-server"
    # 同一台上几片同时起、都没存时只一片编（持锁），别的等它编完取它；锁每 60 秒报一次在等谁，别让任务日志满十分钟不涨
    if mkdir -p -- "$store" 2>/dev/null && command -v flock >/dev/null && exec {prune_server_store_lock}>"$store/.build.lock"; then
      until flock -w 60 "$prune_server_store_lock"; do
        echo "  · 这一台上另一片在编 prune-server（键 $key），等它编完取它（每 60 秒报一次）"
      done
    fi
    if [[ -x "$cached" ]]; then
      touch -- "$store/$key" 2>/dev/null || true
      prune_server_store_unlock
      PRUNE_SERVER_BINARY="$cached"
      export PRUNE_SERVER_BINARY
      echo "  · 变异复跑的临时 prune-server 用这一台按键 $key 存下的 $PRUNE_SERVER_BINARY（没编：同一份源码、编译器与预编库取值这一台只编一次，存放目录文件头「崩溃判定」）"
      return 0
    fi
  fi
  prune_server_build_directory="$(mktemp -d "${TMPDIR:-/tmp}/crates-mutation-prune-server.XXXXXX")" || {
    prune_server_store_unlock
    echo "  ✗ 建不了编 prune-server 的临时目录，一行都没判"
    echo "     → 怎么办：看 \${TMPDIR:-/tmp} 满没满、有没有写权限，修好再跑（不要自己指二进制，这一格自己编一份）。"
    return 1
  }
  if ! python3 "$STAGE_REPOSITORY/scripts/hosts/rocksdb-prebuilt.py" entry --root "$STAGE_REPOSITORY" >"$prune_server_build_directory/rocksdb-entry.txt" 2>&1; then
    sed 's/^/     /' "$prune_server_build_directory/rocksdb-entry.txt" | tail -n 8
    echo "  ✗ 编 prune-server 之前 RocksDB 预编库的取值没判过（上面是 rocksdb-prebuilt.py entry 的输出尾部），没编、一行都没判"
    echo "     → 怎么办：照上面那几行的出路修（开新会话，或命令前 eval \"\$(python3 scripts/hosts/rocksdb-prebuilt.py env)\"），再跑。"
    prune_server_store_unlock
    return 1
  fi
  # 在这一道所在的仓根里编：cargo 从起它的目录往上找 .cargo/config.toml（[env] BINDGEN_EXTRA_CLANG_ARGS，librocksdb-sys 靠它），被判的根是别处时读不到
  if ! (cd "$STAGE_REPOSITORY" && CARGO_TARGET_DIR="$prune_server_build_directory/target" \
      cargo build --release --offline --manifest-path "$STAGE_REPOSITORY/prune-server/Cargo.toml" --bin prune-server >"$prune_server_build_directory/build.log" 2>&1) \
      || [[ ! -x "$prune_server_build_directory/target/release/prune-server" ]]; then
    echo "  ✗ 编不出 prune-server（cargo build --release --offline --manifest-path prune-server/Cargo.toml --bin prune-server）：$(tail -n 3 "$prune_server_build_directory/build.log" | tr '\n' ' ' | clip_text 300)"
    echo "     → 怎么办：照报错修 prune-server/（归工具实现员），再跑这一格。"
    prune_server_store_unlock
    return 1
  fi
  PRUNE_SERVER_BINARY="$prune_server_build_directory/target/release/prune-server"
  # 编好的二进制存进这一台的存放目录（键同上），放进去了就用存下的那一份（编译目录照旧跑完删）；存不进去用编译目录里这一份、不判红
  if [[ -n "$key" ]] && prune_server_store_install "$store" "$key" "$PRUNE_SERVER_BINARY"; then
    PRUNE_SERVER_BINARY="$store/$key/prune-server"
    echo "  · 变异复跑的临时 prune-server 用这一格编的、按键 $key 存下的 $PRUNE_SERVER_BINARY（这一台之后的片取它、不再编）"
  else
    echo "  · 变异复跑的临时 prune-server 用这一格编的 $PRUNE_SERVER_BINARY（跑完删）"
  fi
  prune_server_store_unlock
  export PRUNE_SERVER_BINARY
}
item_selection_file=""
if ((${#named_mutations[@]})); then
  if [[ -n "${GATE_MUTATION_ROW_SELECTION:-}" ]]; then
    echo "  ✗ --item 与环境里的 GATE_MUTATION_ROW_SELECTION 一起给了：两份选行不知道听哪一份"
    echo "     → 怎么办：点名几行只用 --item；GATE_MUTATION_ROW_SELECTION 只由分片那一套 scripts/mutation/mutation-shard-run.sh 的 shard 那一步设。"
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

# 分片的三步（文件头「分片」）：计划与各片转给分片那一套、照它的退出码退；合并先转给它核各片判的行，过了再往下照核账判（下面的格）
SHARD_SCRIPT="$STAGE_REPOSITORY/scripts/mutation/mutation-shard-run.sh"
if ((! list_items)) && [[ -n "$shard_phase" ]]; then
  case "$shard_phase" in
    plan)
      if [[ -n "$item_selection_file" ]]; then
        env -u GATE_MUTATION_ROW_SELECTION bash "$SHARD_SCRIPT" "$ROOT" --only-rows "$item_selection_file"
      else
        env -u GATE_MUTATION_ROW_SELECTION bash "$SHARD_SCRIPT" "$ROOT"
      fi
      exit $? ;;
    shard)
      env -u GATE_MUTATION_ROW_SELECTION bash "$SHARD_SCRIPT" "$ROOT"
      exit $? ;;
    merge)
      env -u GATE_MUTATION_ROW_SELECTION bash "$SHARD_SCRIPT" "$ROOT" || exit 1
      echo "  · 合并：各片判的行核过，下面照核账判（键对不上的行不现判，文件头「分片」）" ;;
    *)
      echo "  ✗ TASK_SERVER_SHARD_PHASE=$shard_phase 认不出（只认 plan、shard、merge）"
      echo "     → 怎么办：这个变量只由 task-server 设；手跑时去掉它"
      exit 2 ;;
  esac
fi

# 点名的几项要不要卡（文件头「按项机读行」的 ITEM_CARD，统一任务服务在估算那一步读）：每项一行 ITEM_CARD、末了一行 ITEM_CARD_DONE，打在 stdout。
# 点名的行里有一行 cargo 参数开了上卡的特性（共用模块的 gpu_test_run）就整条都是 1（没证明的写 1，不再算下去）；一行都没开、而多机配置开了 GPU（ENABLE_GPU=1）时，
# 再照共用模块的 plan_context 算开 GPU 时崩溃判段那几行补 gpu 特性的行（gpu_route_lines），补了的那一项是 1，其余是 0。算失败（读不了表、选行不对、
# 键与闭包算不出、任何异常）一行都不打，原因打在 stderr：服务照没有这些行办，保持要卡。
item_cards_lines() {
  python3 - "$ROOT" "$STAGE_REPOSITORY/scripts/mutation/crates-mutation-rows.py" "$item_selection_file" <<'PY' || echo "  ! ITEM_CARD 没算出来（原因在上面）：服务按要卡办" >&2
import contextlib
import importlib.util
import os
import shlex
import sys


def item_cards(root, module_path, selection_path):
    specification = importlib.util.spec_from_file_location("crates_mutation_rows", module_path)
    module = importlib.util.module_from_spec(specification)
    sys.modules[specification.name] = module
    specification.loader.exec_module(module)
    environment = dict(os.environ)
    rows, _malformed = module.read_table(os.path.join(root, module.TABLE), environment)
    selected, problems = module.read_selection(selection_path, rows)
    if problems:
        raise RuntimeError("选行文件不对：" + "；".join(problems))
    chosen = [row for row in rows if row.line_number in selected]
    cards = {}
    for row in chosen:
        try:
            on_card = module.gpu_test_run(["cargo", "test", *shlex.split(row.arguments)])
        except ValueError:
            on_card = True   # 引号不配对：认不出就当要卡
        cards[row.name] = max(cards.get(row.name, 0), int(on_card))
    if any(cards.values()):
        cards = {name: 1 for name in cards}   # 有一项已经要卡，整条都要卡：别的项不再花时间证明，没证明的写 1
    elif cards and module.gpu_route_enabled(environment)[0]:
        with contextlib.redirect_stdout(sys.stderr):
            context = module.plan_context(root, environment, selection_path)
        if context is None:
            raise RuntimeError("键与闭包没算出来，开 GPU 时哪几行要补 gpu 特性无从知道")
        routed = {line_number for line_number, _feature in context["key_context"].gpu_route_lines}
        for row in chosen:
            if row.line_number in routed:
                cards[row.name] = 1
    if not cards:
        raise RuntimeError("点名的行一行都没读到")
    return cards


try:
    result = item_cards(sys.argv[1], sys.argv[2], sys.argv[3])
except Exception as error:
    print(f"  ✗ ITEM_CARD：{type(error).__name__}: {error}", file=sys.stderr)
    print("     → 怎么办：照上面一句改 --item 点名的行或 crates/mutations.tsv 里那几行（选行文件对不上、读不了表）；改完重交，服务这一次按要卡办", file=sys.stderr)
    sys.exit(1)
for name, value in result.items():
    print(f"ITEM_CARD item={name} card={value}")
print(f"ITEM_CARD_DONE items={len(result)}")
PY
}

if ((list_items)); then
  # 一行一条变异（# 起头的行与空行不算）：变异名<制表符>第几行、改哪个文件；只有共用模块里一份。--with-keys 另给每项一行 ITEM_PLAN；
  # 带 --item 的只列、只算点名那几行：选行文件（上面 --item 换成的）交给 --rows。不 exec：退出时 EXIT trap 删选行文件
  list_items_arguments=(--list-items)
  if ((with_keys)); then list_items_arguments+=(--with-keys); fi
  if [[ -n "$item_selection_file" ]]; then list_items_arguments+=(--rows "$item_selection_file"); fi
  python3 "$STAGE_REPOSITORY/scripts/mutation/crates-mutation-rows.py" "${list_items_arguments[@]}" "$ROOT"
  list_items_exit=$?
  if ((list_items_exit == 0)) && ((with_keys)) && [[ -n "$item_selection_file" ]]; then item_cards_lines; fi
  exit "$list_items_exit"
fi

# ── crates-mutation-rows 格 ──
cell_crates_mutation_rows() {
# 整道没有复用标记：复用落在每一行的行键上（共用模块 run_stage 逐行查判定库），这里不问 stage-must-run.sh、不写整道标记。
# 前提读的是本阶段所在那一份仓的登记表：判别力样本的目录里没有登记表，前提照样按真表判。
python3 "$STAGE_REPOSITORY/scripts/gate/admission.py" gate-preconditions "$STAGE_REPOSITORY" "$(basename "$0")" || exit 1
# 编译目录不在这里算（文件头「编译目录」）：GATE_MUTATION_TARGET_DIR、CARGO_TARGET_DIR 设了照原样传，都没设由共用模块在这一趟的运行目录里建、跑完删
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
  # 假 cargo 每起一次先往 FAKE_CARGO_LOG 追加一行，没设就退 1（基线、变异一条都判不了）；弄坏开关 sample-cargo-log-unset 下不设
  if [[ ",${GATE_MUTATION_BREAK:-}," != *",sample-cargo-log-unset,"* ]]; then
    FAKE_CARGO_LOG="$sample_tools_directory/cargo-calls.log"
    export FAKE_CARGO_LOG
  fi
  echo "  ! 样本档：$PWD/.crates-mutation-replay-sample-tools 在，起 cargo 的是 $sample_tools_directory 里的假 cargo 与假 llvm 工具（不编译）；这一趟不算真跑过变异表"
fi
# 核账（文件头「分片」）：不是一片（选行模式）、不是样本根的一趟，键对不上的行不现判；一片照现判
if [[ -n "${GATE_MUTATION_ROW_SELECTION:-}" && -z "$item_selection_file" ]] || [[ -n "$sample_tools_directory" ]]; then
  unset GATE_MUTATION_SHARDED_ONLY
else
  GATE_MUTATION_SHARDED_ONLY=1
  export GATE_MUTATION_SHARDED_ONLY
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
# 撞内存上限（文件头「上限、限时与卡住」）：留标记，格汇总之后整道照 250 退
if [[ "$module_exit" == 250 && -n "${memory_cap_hit_directory:-}" ]]; then : > "$memory_cap_hit_directory/memory-cap-hit"; fi
unset GATE_MUTATION_NOT_RUN_RECORD
not_run_count="$(grep -c . "$not_run_record" 2>/dev/null)" || not_run_count=0
# 分片的各片（GATE_MUTATION_ROW_SELECTION 不是 --item 换成的）不转：分出去的行由合并那一步的核账再报
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
  echo "  ✓ 点名的 $selected_count 条变异里抓到 $((selected_count - not_run_count)) 条、本次未跑 $not_run_count 条（逐条见上；只判了这几行，不是都抓到、也不是整道结论）"  # gate-lint:skip-listed 本次未跑的那几条由 scripts/mutation/crates-mutation-rows.py 在模块输出里逐条打出，这一行只报数
  exit 0
fi
if [[ -n "$item_selection_file" && "$module_exit" == 3 ]]; then
  selected_count="$(sort -u "$item_selection_file" | grep -c .)"
  echo "  ✓ 点名的 $selected_count 条变异都抓到了（只判了这几行，不是整道结论；现判、复用、拒算各几条见上面「复用：」那一行）"
  exit 0
fi
exit "$module_exit"
}

# 样本根只在临时 prune-server 底下判（文件头同名一段）。样本根的认法与格里换假工具那一处相同；.sample-environment-bare 在样本根上时按「环境里什么都没设」答
mutation_sample_service_check() {
  [[ -d "$PWD/.crates-mutation-replay-sample-tools" && -z "${GATE_STAGED_FROM:-}" && "$(pwd -P)" != "$(cd "$STAGE_REPOSITORY" && pwd -P)" ]] || return 0
  [[ ",${GATE_MUTATION_BREAK:-}," == *",sample-service-unchecked,"* ]] && return 0
  local service_problem
  if ! service_problem="$(python3 -c 'import os, sys; sys.path.insert(0, sys.argv[1]); import prune_server_client; print(prune_server_client.selftest_service_problem({} if os.path.isfile(sys.argv[2]) else None))' \
        "$STAGE_REPOSITORY/scripts/prune-server" "$PWD/.sample-environment-bare" 2>&1)" || [[ -n "$service_problem" ]]; then
    echo "  ✗ 判的根是样本（$PWD），不在临时 prune-server 底下：${service_problem:-读不了 scripts/prune-server/prune_server_client.py}；样本的判定会写进真服务，一行都没判"
    echo "     → 怎么办：样本档整条套临时服务起（交任务服务时前面加 task-server/task submit --）：python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh .claude/gate.d --item crates-mutation-replay.sh"
    echo "       包装交下 PRUNE_SERVER_BINARY，这一格不再自己编 prune-server；单跑某一份样本根时，把 bash .claude/gate.d/crates-mutation-replay.sh <样本根> 接在同一个包装的 -- 后面。"
    exit 2
  fi
}
mutation_sample_service_check

# 分片的各片不经格汇总，照 mutation-shard-run.sh 的约定退（文件头 GATE_MUTATION_ROW_SELECTION 那一条）
if [[ -n "${GATE_MUTATION_ROW_SELECTION:-}" && -z "$item_selection_file" ]]; then
  ( cell_crates_mutation_rows )
  exit $?
fi
# 格汇总把撞内存上限的那一格（退 250）也并成红、整道退 1；撞了就照 250 退，服务按 memory-cap-hit 加额度重排（文件头「上限、限时与卡住」）
memory_cap_hit_directory="$(mktemp -d)" || { echo "  ✗ 建不了放撞内存上限标记的临时目录，一行都没判"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
( stage_cells_run "$ROOT" )
stage_exit=$?
if [[ -e "$memory_cap_hit_directory/memory-cap-hit" ]]; then
  rm -rf -- "${memory_cap_hit_directory:?}"
  echo "  ✗ 撞内存上限：服务给的上限装不下一个工作进程的峰值（上面那一格的红句），这一道照统一任务服务错误类型表里 memory-cap-hit 那一行退 250"
  echo "     → 怎么办：经 task-server/task submit 交的由服务加额度重排，不用改命令；手跑的给更大的内存上限再跑。"
  exit 250
fi
rm -rf -- "${memory_cap_hit_directory:?}"
exit "$stage_exit"
