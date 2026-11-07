#!/usr/bin/env bash
# admission: always 每一批补丁都要打一次、验一次，上一次的结论不替这一次作保
# run-condition: command git cargo rsync
# 一批实现员交回之后的「合入后验证」：开跑那一刻把要验的树拷成一份快照，在快照上打全部补丁、编一次、只跑补丁改到的测试目标与变异行点名的测试目标（下面「挑测试目标」）、
# fmt、clippy 与 build，再把跑到的验证包目标里登记的崩溃枚举用例与挂特性的测试文件逐项交门禁 crash-point-replay（下面「验证包那一步」），红的列出来。
# 不跑整个包，也不起门禁的整格：判不出改到哪个测试目标的那几种测试那一步不跑、整条退 3（下面「挑测试目标」）。替掉派一个实现员做这件活。
#
#   merge-and-verify.sh [--skip-apply] [--crash-expand-up-to <n>] [--item <包>:<目标根文件>]… <补丁目录> …
#   merge-and-verify.sh --list-items [--with-keys] [同一趟的别的参数…]   逐行列这一趟测试那一步可点的项，不编、不跑（下面「逐项点名与按条复用」）
#     起法：task-server/task submit -- bash scripts/gate/merge-and-verify.sh …（一整条一个任务；内存上限、线程、机器、显卡都由服务定，命令里不写）。
#     --item <项>（给几次取并集）在挑出的测试目标里只跑点名的那几个（下面「逐项点名与按条复用」）；fmt、clippy、build、重编核对照旧，验证包那一步只交点名的验证包目标里的。
#     测试那一步的并行度取 heavy_thread_default（scripts/hosts/heavy-threads.sh；经服务起时是服务套的 capped.sh 设的 SINGLEFS_THREAD_CAP）。
#     --crash-expand-up-to <n> 把崩溃放量与崩溃枚举用例的展开深度缩到不超过 n（变量 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO，只往小缩，大于引擎常量的不放宽），
#     只管验证包目标的 cargo test 与验证包那一步（下面「验证包那一步」）；环境里已设了这个变量的当作给了同一个值；不是非负整数退 2、一样都不做；不给就是整个域。
#   merge-and-verify.sh --selftest                     # 用假的 apply-writer-patch 与假 cargo 走一遍；MERGE_AND_VERIFY_BREAK=ignore-red 时必须判红
#     验证包那一步换上假的那一道（照真的那一道的写法答 --list-items、打按条复用与崩溃状态块那几行）；逐项与复用那几格经 admission.py 查库、记库（判定库在 prune-server 里）：只在临时服务底下跑，
#     裸跑开跑前调客户端 selftest_service_problem、退 2、一格都不跑。起法：python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service
#     --server-binary <prune-server 二进制> -- bash scripts/gate/merge-and-verify.sh --selftest
#
# 快照（不拿锁）：只有这一种跑法，主工作区一个字节不写。
#   开跑那一刻把要验的树（主工作区；补丁已打在上面的带 --skip-apply）拷进 <日志目录>/repo（日志目录在工作根下，第二台同一路径读得到）：
#   rsync --recursive --links，不拷任意一层的 .git、target、target-* 与 __pycache__，带工作区里的全部源码与未提交改动；不保留属性（权限位照建文件时的缺省、
#   不带属主属组：有的临时目录不认保留属性），不保留修改时刻（拷过去的 mtime 是拷的那一刻：调用方给的编译目录编过也照新内容重编，
#   .claude/rules/test-script-discipline.md「编译与算法都要快」）。拷完在快照根 git init（不加文件、不提交）：读写判定库的脚本只认
#   <被判那棵树的 git common-dir>/<库名> 这个名字，库在 prune-server 里、键里不带哪棵仓（scripts/crash/lib_verdict_library.py 的 library_directory），
#   不是 git 工作树就退 2。拷不成、git init 不成退 1、一个补丁都不打。快照放进了要验的树里面（日志目录指进了树里）就退 2、什么都不拷。
#   之后打补丁（apply-writer-patch.py --root <快照>）、编、跑测试、验证包（起的是快照里那一份门禁、交给它的根是快照）、算复用键全对着快照：
#   主工作区随后怎么改都不影响这一趟的结论与写进判定库的键。编译目录照下面「这一趟的编译目录」。
#   收尾删掉快照（成功、失败、被 INT / TERM 停都删）；日志目录留着（日志小，汇总行指着它）。删完打一行「删了快照 …（共 …）」；删不成退 1、写明没删成。
#   要留现场：MERGE_AND_VERIFY_KEEP_COPY=1，收尾不删，打出快照与编译目录的路径和删法，看完自己删（command-safety.md「测试镜像一律放临时目录」：
#   要留着看现场由显式的开关打开、留下时打印路径）。
# 这一趟的编译目录（自己用完自己删，不留给以后的趟复用）：调用方没设 CARGO_TARGET_DIR 时
#   在这一台自己的 ${TMPDIR:-/tmp} 下新建 merge-and-verify-target.<随机>、export 成 CARGO_TARGET_DIR，四条 cargo 都用它；不建在日志目录下：
#   日志目录在工作根下，工作根两台共享（第二台经 NFS 写回本机的盘），编译目录有上百 G（records/2026-10-31-task-server巡检.md 问题 72 之二）。日志照旧在工作根下。
#   正常结束、红了退出、被 INT / TERM 停都在 EXIT trap 里删掉，而且是 trap 的第一步、在删快照之前（那几步写满盘、出错都挡不住它；删不成退 1，照接着删快照）
#   （scripts/hosts/run_build_directory.py remove：删之前核本机没有进程开着里面的文件，有就是这一趟的子进程没收干净，不删、逐个报进程、退 1）。
#   建不成退 2。调用方设了 CARGO_TARGET_DIR（统一任务服务给的编译目录）就照它编、收尾不删（不是这里建的，归调用方删）。
#
# 为什么：「合入后验证」做的前三步（打补丁、编、逐个跑测试）
# 是 apply-writer-patch.py 加几条命令，只有「红了改钉值」要判断——那一步红了才派实现员。
# 做法：每个补丁目录经 scripts/gate/apply-writer-patch.py --root <快照> 打（退 2 就整批停）。调用方给的编译目录（CARGO_TARGET_DIR）可能是编过的，
# cargo 按 mtime 判本地包新不新鲜，带着旧 mtime 的新内容会被当成已编：
# 快照不保留修改时刻（文件头「快照」），里面每个文件都比编译目录里的产物新，不另刷源码戳。
# 然后在本机直接跑 test、fmt、clippy、build 四步 cargo（clippy 按特性组合各一条；不套内存包装与 capped.sh，这一整条的上限与线程由起它的服务定），与上游 .claude/singlefs-ai-sop/scripts/check.sh 同一套判据：
#   test   只跑挑出的测试目标（下面「挑测试目标」），一个目标一项，一个包一条 `cargo test --offline --no-fail-fast -p <包> <点到的 --lib / --test 名 / --bin 名…>`，几个包的几条依次跑
#          （日志 test-<序号>.log，接成一份 test.log 与 test.problems.log）；不带上依赖它们的包，也不跑包里没点到的目标。check.sh 跑的是 `cargo test --all`，这里只跑其中点到的目标。
#          验证包 singlefs-verification 的目标带 --release 跑（与门禁 crash-point-replay 的 crash-case-replay 格同一档、同一种类的判定，下面「逐项点名与按条复用」），
#          登记成崩溃枚举用例与放量分项的用例函数经 --skip 排掉（崩溃枚举用例交验证包那一步）；验证包的计划算不出时这一包不起（登记的用例排不掉），按条复用失灵照判红。
#          工作区成员（树根 Cargo.toml 的 [workspace] members，逐个读成员 Cargo.toml 的包名）里没有 singlefs-verification（改了名、挪了）、成员写成通配、读不出包名，都判红、不编：
#          认不出验证包就排不掉登记的崩溃枚举用例。
#          退 0 而 test result 行里一条测试都没数到、按条复用也一条都没复用，判红（check.sh「一条测试都没跑」那一条）。挑出的目标一个都没有（补丁只改了要特性的目标、只删了测试目标、
#          只往变异表里删行）时测试这一步不跑、报「测试本次未跑」，不记红也不算测过。
#          每一条交给 scripts/gate/test-targets-parallel.py --jobs <线程上限>（用例少的二进制跑的时候核就空着：第二台上只用到约 5 核、登记 31，
#          records/2026-10-18-里程碑三收尾与近期欠账.md 第 780 行）：先 --no-run 一次编好，再按测试目标各起一条 cargo test -p <包> <挑法>、同时至多线程上限那么多条，
#          每条的 RUST_TEST_THREADS 分摊，各目标的输出按派活的次序回读，派出与收回的条数对不上判红。跑的用例与 cargo test -p … <挑法> 同一批，
#          FAILED 行、test result 行与 Compiling 行照旧落进 test.log，下面的判法不变。
#          MERGE_AND_VERIFY_TEST_TARGETS=serial（只给自证）退回一条 cargo test --offline --no-fail-fast -p …；自证的别的格都这么设，只有「按测试目标并行起」那一格不设。
# 挑测试目标（测试那一步与验证包那一步都照它；判法在 MERGE_AND_VERIFY_ITEMS_PROGRAM 的 targets 一段）：读给的每个补丁目录里 crates.patch 改到的路径（新旧两侧，
#   scripts/gate/lib_rebuilt_packages.py 的 collect_paths）与 mutations-append.tsv、mutations-replacements.tsv 的行；--skip-apply 时照样读给的补丁目录，不打。
#   路径往上找到的第一份带 [package] 的 Cargo.toml 是它的包（lib_rebuilt_packages.py 的 package_of），包里的相对路径照这几条落到测试目标
#   （目标清单是 scripts/gate/rerun-failed-tests.py 的 test_targets_of_package：cargo test 不带特性会跑的那几个）：
#     正是一个测试目标的根文件（tests/<目标>.rs、tests/<目标>/main.rs、src/bin/<名>.rs、src/main.rs、[[test]] / [[bin]] 写的 path）→ 那个目标；
#     tests/<子目录>/ 下的 → tests/<子目录>/main.rs 那个目标；没有它，就是 tests/ 顶层里写了 mod <子目录>（或 #[path = "<子目录>/…"]）的那几个目标；
#     src/bin/<名>/ 下的 → src/bin/<名>/main.rs 那个目标；
#     src/ 下别的（不论是不是 .rs）→ 库目标（[lib] 的 path，缺省 src/lib.rs）；包里没有库目标的落到 src/main.rs 那个目标；
#     写了 required-features 的 [[test]] / [[bin]]（cargo test 不带特性不编）记「要特性」：产品 GPU 包的交验证包那一步（真卡对拍那一格），别的包报本次未跑；
#     测试目标根文件那一类的路径在树里已经没有（补丁删了这个目标）：报一句、不跑。
#   变异行第五段（cargo test 的参数）里 -p <包> 与 --lib、--test <名>、--bin <名>、--example <名>、--bench <名>、--doc 点名的目标 → 那个目标
#     （`--` 之后的筛选词只报一句：跑的是整个目标，键相同的测试函数照复用）。
#   判不出的这几种：测试那一步与验证包那一步都不跑、整条退 3（fmt、clippy、build 与重编核对照跑，有红照退 1），逐条列原因，
#     出路写「整包归提交时的整轮门禁；提交之外要整包跑先弹窗问用户」：
#     没给补丁目录；改到工作区根的 Cargo.toml、Cargo.lock 或 .cargo/；改到不落在任何包里的路径；落在包里而照上面几条落不到测试目标的
#     （包的 Cargo.toml、build.rs、tests/ 顶层的数据文件这类）；变异行只点了包、点了一类目标（--tests、--bins、--examples、--benches、--all-targets、--workspace、--all）、
#     没点包，或点的目标在包里找不到。
#   读不了补丁、成员清单或目标清单退 1。挑出的项、各自为什么挑上（补丁改到哪个文件、哪一条变异行点名）逐项打一行。
#   fmt    此刻是 `cargo fmt --all -- --check`；build 此刻是 `cargo build --offline --all-targets`（参数照 check.sh 那一行现读，下面「cargo 命令现读」）。
#   clippy 照 check.sh 按特性组合逐个跑：每个组合一条 `cargo clippy --offline --all-targets <特性参数> -- -D warnings` 加编码纪律那张 lint 表
#          （组合 `default` 不带特性参数，别的照组合里的词带上），日志 <日志目录>/clippy-<组合>.log（组合去掉开头的 -、别的非字母数字换成 -）；
#          任一组合红整步红、红句点名是哪几个组合，过的那句列出跑了的组合。
#   lint 表与特性组合都只在 check.sh 一处定义：每次开跑从它的 `CODE_DISCIPLINE_LINTS=(` … `)` 那一段现读 lint 表（一条一对 `-D clippy::<名>`），
#   从行首那一行 `CLIPPY_FEATURE_COMBINATIONS=("…" "…")` 现读缺省的特性组合（每个组合是 `default` 或以 - 起头的特性参数），不在这里另抄一份；
#   缺省组合读出来不到两个判红（check.sh 退回了单趟 clippy，是上游改了判据）。
#   同时核 check.sh 里 clippy 按组合循环的三行结构（CHECK_SH_STRUCTURE_LINES）还是这里照着对齐的那一种。
#   cargo 命令现读：fmt、clippy、build、test 四条 cargo 命令的写法不在这里抄表，每次开跑从 check.sh 现读（read_cargo_command）：命令位置上写 cargo <子命令>
#   或 run_cargo <子命令>（前面可带 if）的行，每个子命令恰好一条；子命令之后到第一个 |、;、\ 之前的词（2>&1 不算）就是它的参数，这里的四步照这些词拼
#   （cargo 子命令后接 --offline；test 步另按包、按测试目标挑，所以那一行只认 --all 或 --workspace 加不带值的 --标志，带 --lib、--doc 这类收窄目标的标志判红；
#   clippy 那一行里 ${clippy_feature_arguments[@]+…} 换成这个组合的特性参数、"${CODE_DISCIPLINE_LINTS[@]}" 换成 lint 表，各恰好一次；别的带 $、引号、括号的词一律不认）。
#   所以上游只换包装（run_cargo 与 cargo 来回换）、加删标志（--locked 这类）时这里不用改；找不到那一条、写了不止一条、有不认的词才判红，
#   红句点名是哪一条命令、第几行、哪个词，出路写往 read_cargo_command 或 read_check_sh 里补哪一种识别，不是笼统的「写法变了」。
#   不退回「读不出就用写死的」：那样合入后验证跑的就不一定是 check.sh 跑的了（用户要的是同一套判据）。
#   check.sh 认项目根的特性组合登记（上游 lib.sh 的 CLIPPY_FEATURE_COMBINATIONS_FILE，登记了就只跑登记的）：这里不读登记，要验的树里有这份登记就判红、不编，
#   出路是先把登记的读法接进这里。读不出表或组合、形状不对、结构行或哪条 cargo 命令读不出，都判红、一个补丁都不打：上游改了判据，先回来对齐这里。
# 日志落 $MERGE_AND_VERIFY_LOG_DIRECTORY（默认工作根下的 merge-and-verify/<日期>-<进程号>，工作根只在 .claude/hooks/lib_work_root.py 里定义）。
# 改过的包都重编了没有（.claude/kb/milestone/03-third-txn.md「待优化：流程、门禁与装置上踩过的坑」第 1 行；判法在 scripts/gate/lib_rebuilt_packages.py，这里不另写）：
#   改过的包从给的补丁目录里的 crates.patch 算（--skip-apply 时照样读给的补丁目录，只是不打；打没打先核，见「--skip-apply 先核补丁打过没有」）。
#   编完核 test 与 build 两趟的编译输出：每个改过的包都要有「Compiling <包名> v…」行，缺一个记一项红（编译目录里的产物比快照的拷入时刻还新，例如别的趟往同一个编译目录里后编过），
#   出路是换干净编译目录整趟重跑。
#   只核包里改过的 .rs；包里改过的 .rs 一个都不在编译目录的 dep-info 里的包不核（这几趟编译不读它们，例如 验证包 gpu 特性下才编的模块）。
#   没给补丁目录、或补丁里没有要核的包：报「重编核对本次未判」，不记红也不算核过。
# --skip-apply 先核补丁打过没有：建好快照、进快照之后，编之前，在 ${TMPDIR:-/tmp} 下建一份临时副本（git init 的空仓，只拷要验的树里给的那几份 crates.patch 点到的文件），
#   照给的补丁目录的次序倒着逐份 `git apply --reverse`（后打的先倒）：全部倒得掉才算都已打在树上，同一批两份补丁改到同一行也判得对（reverse_check_in_order）。
#   有一份倒不掉、或给的补丁目录不在、或临时副本建不成，就退 1、点名那一份（倒不掉的连同它前面没往下核的份数）、一个都不编，
#   出路是没打过的那几份去掉 --skip-apply 让这里打、次序给错了的照打的次序重给。临时副本用完就删，快照一个字节不动。
#   没有 crates.patch 的目录（只有 mutations-append.tsv 的）不核，报「本次未核」；不带 --skip-apply 时不核（这里自己打）。
# 门禁 code-source-discipline 的 mutation-tables 格：打补丁脚本（apply-writer-patch.py）不在它自己的进程里起这一格，打上了就退 0、末尾打出要另交的命令
#   （它文件头「mutation-tables 那一格」）；这里也不起它。补丁都打上之后打一行「· code-source-discipline 的 mutation-tables 格本次未跑」，接着写补丁打进
#   主工作区之后另交的整条命令（--cwd 与根都是要验的树，带范围声明），汇总行照写「mutation-tables 格本次未跑」；本次未跑不改退出码，也不算判过。
#   打补丁脚本退 1（它不该退 1：只有它的弄坏开关 mutation-tables-in-process 下才在进程里判这一格）照判红、退 1，点名那份日志。
# MERGE_AND_VERIFY_TREE（只给自证）：要验的树（默认这份脚本所在的仓根），快照从它拷。
# MERGE_AND_VERIFY_CHECK_SH（只给自证）：从哪一份 check.sh 读 lint 表、特性组合、clippy 循环的结构行与四条 cargo 命令（默认这份脚本所在仓的 .claude/singlefs-ai-sop/scripts/check.sh）。
# 上限、线程、机器（records/2026-10-30-编译绕过内存总上限.md 收尾批 M03、M06：脚本里层的包装与上限一律删，一整条一个任务，上限、线程、机器由服务定）：
#   这里没有对应的参数，认不出的参数一律退 2、一样都不做（服务提交时拒这类参数，这里不另拒）。
#   撞内存上限、显存不够、排不上都是服务那一层的结局（服务放大重排、记错误类型），这里不另认包装的退出码。
# 退出码：0 全绿；1 有红（测试、fmt、clippy、build、apply 退 1、快照拷不成或 git init 不成、check.sh 读不出、工作区成员里认不出 验证包、读不了补丁或目标清单、
#   改过的包没重编、起 cargo 之前 RocksDB 预编库的取值判红（下面「RocksDB 预编库的取值」）、验证包那一步红（下面「验证包那一步」））；
#   2 补丁核不过（apply 退 2）、用法错（含认不出的参数、--item 的项写错）、快照放进了要验的树里面。验证包那一步本次未跑、测试本次未跑都不改退出码，汇总行里点名。
#   3 判不出改到哪个测试目标（下面「挑测试目标」那几种）：测试那一步与验证包那一步没跑，别的步照跑、没有红；有红退 1。
#     出路：整包归提交时的整轮门禁；提交之外要整包跑先弹窗问用户。
#   75 按条复用那一层连不上 prune-server（admission.py plan-failure 判不可用），别的没有红；有红退 1，判不出测试目标退 3。
# RocksDB 预编库的取值：clippy --all-features 与 build --all-targets 编工作区全部成员，连同 验证包（编到 librocksdb-sys）。
#   补丁打完之后、四条 cargo 之前跑 python3 scripts/hosts/rocksdb-prebuilt.py entry --root <快照>（这台装了、此刻进程环境的取值对得上），
#   判红退 1、一条 cargo 都不起。
#   entry 的跳过办法：树里没有依赖 rocksdb 的清单（自证的样本工作区就是这样），或 PATH 上的 cargo 是带「# rocksdb-prebuilt-entry: selftest-fake-cargo」那一行的假 cargo。
# 验证包那一步（records/2026-09-16-subagent拆分提案.md 第九十节 ㉓；按块、按条都照单元键复用，不起门禁的整格）：test、fmt、clippy、build 与重编核对之后，
#   这一趟跑到的验证包目标里要另走门禁的几样，逐样交快照里那一份门禁 <快照>/.claude/gate.d/crash-point-replay.sh（交给它的根是快照），一样一次、一次一份日志：
#     登记的崩溃枚举用例：先起一次那一道的 --list-items（不起 cargo；日志 verification-list-items.log）读它现列的「crash-case:<用例名><TAB><test=>」，
#       test= 的测试目标是这一趟跑到的验证包目标的 → --check crash-case-replay --crash-cases-only --item crash-case:<用例名>…
#       （只走按块那一路：块照块键复用、缺的当场判；不起包里按条复用那一趟，那几个目标的别的测试函数在测试那一步跑过）；日志 verification-crash-cases.log；
#     挂特性的测试文件：同一次 --list-items 读的「feature-gated:<文件名>」是这一趟跑到的 tests/<文件名> 目标的、跑到了验证包库目标时的 feature-gated:lib →
#       --check feature-gated-tests --item feature-gated:<文件名>…（只编、只跑点名的目标，特性照整格那一组，按条复用）；日志 verification-feature-gated.log；
#     产品 GPU 包的真卡对拍：挑测试目标记了「要特性」、包名是那一道里 PRODUCT_GPU_PACKAGE_NAME="<包名>" 那一行现读的那个包（此刻 singlefs-background-gpu；读不出这一行的
#       照起并点名读不出）→ --check product-gpu-real-card-tests（那一格不收点名，跑的是那个包真卡对拍的几个目标，键相同的照复用）；日志 verification-product-gpu.log；
#       点了 --item 时不起、报本次未跑。别的包要特性的目标报本次未跑（归提交时）。
#   三样都没有就不起那一道，报「验证包那一步本次未跑」与原因（不记红、不算过）。放量分项（scripts/crash/crash-amplification.sh 的 ITEMS）经 --skip 排掉之后这里不起，归提交时。
#   每一次：退 0 是过；退 3 只在交给它的根是它的样本根（有 .crash-point-replay-sample-tools 目录，自证用）时算过，别处退 3 判红；退 77 报本次未跑；别的退出码判红。
#     过了还要读到那一次的行：崩溃枚举用例那一次「✓ 崩溃状态块：现判 …、复用 …、拒算 …」，另两样「按条复用：…现判 N、复用 M、拒算 R」；读不到不算过。现判、复用、拒算加总进汇总行。
#     那一道的 --list-items 退非 0：验证包那一步判红，点名那份日志。
#   红了逐条列出日志里 FAILED 的测试函数、它报的「红的 …」（各自变了的单元）与 ITEM_DONE 里 exit=1 的项，出路写红的测试函数并进写段的下一批，
#     与只重跑红的那几项的整条命令（task-server/task submit -- … 交，范围声明写在 task-server/task 前面：种类 items，范围数是 --check 的格数加 --item 的项数；
#     ITEM_DONE 里认不出红在哪一项的，重跑那一次点的全部项）；那条命令起的是主工作区那一份门禁、交给它的根是主工作区（快照收尾就删，修完在主工作区上重跑）。
#   在这一整条里面直接起（run_step：两份日志），环境里的 MERGE_AND_VERIFY_BREAK 不交给它。起不起这一步不看环境变量：交给 task-server 的命令什么前缀都不写，
#     什么时候跑由规矩管（`.claude/main-agent.md`「写与验分两段」验证段第 1 步）。
#   --crash-expand-up-to <n>（环境里已设了 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO 的当作给了同一个值）把这个变量交给门禁那几次与验证包目标的 cargo test（连同算键的请求），
#     缩崩溃域（只往小缩）：缩过域的那一趟汇总里写「缩过域、不算门禁通过」，出路里只重跑的命令也带同一个变量（env 接在 submit -- 之后）。n 不是非负整数退 2、一样都不做。
#   MERGE_AND_VERIFY_VERIFICATION_STAGE 换掉那一道（默认是快照里那一份；出路那条命令里也写它）、MERGE_AND_VERIFY_VERIFICATION_ROOT 换掉交给它的根
#   （默认是快照；出路那条命令里也写它）；都只给自证。
# 逐项点名与按条复用（测试那一步；.claude/rules/test-script-discipline.md「入口脚本：不带参数就是不指定范围」「判定可复用：按输入存，按前缀剪」）：
#   项：挑出的测试目标，一个目标一项，写成 <包>:<目标根文件>（目标根文件从包目录起，就是 cargo 段头 `Running …` 里的源文件，
#     例 singlefs-verification:tests/model_differential.rs、singlefs-core:src/lib.rs），文档测试 <包>:doc（只有变异行点了 --doc 才有）。项的写法、挑目标的参数、日志里一段归哪一项，
#     都照 scripts/gate/rerun-failed-tests.py 文件头「项的写法」（test_targets_of_package、section_item；这里经 importlib 调，不另写一份）。
#   --list-items：逐行打「<项><TAB>挑目标的参数」，不编、不跑、不拷快照、不打补丁、不建日志目录与编译目录；项照这一趟的挑法（给的补丁目录），
#     读的是要验的树此刻的样子（补丁没打：补丁新加的测试目标这里列不出，真跑时照打了补丁的快照认，点得到）。退 0；判不出测试目标退 3（同真跑，逐条打原因），读不了退 1。
#     另带 --with-keys 时每项再打一行 ITEM_PLAN item=<项> to_judge=<要现判的条数> reusable=<能复用的条数>（键在要验的树此刻的样子上算，经 admission.py test-plan、连 prune-server；
#     文档测试一项一件、算要现判；一个 #[test] 都没有的包两样都是 0；算不成的那一包不打它的 ITEM_PLAN，另打一行「算不出」）；同时给了 --item 的只给点名的项打。
#   --item <项>：只跑点名的项；写错（不在这一趟挑出的项里）退 2、列出可点的项（真跑在打了补丁的快照上认）。不点就是挑出的全部项。
#     每个包一条（test-targets-parallel.py 或自证的 serial 一条 cargo test，`-p <包>` 只带这个包点到的 --lib / --test / --bin / --example / --bench），点了 <包>:doc 的另起一条
#     cargo test --offline --no-fail-fast -p <包>… --doc。点名单跑一项与它在整趟里跑的判定相同：同一套键、同一份库、同一个判法。
#   按条复用：每条测试函数一条判定（unit-verdicts 族，prune-server 那一份库；库名照 admission.py 文件头「判定库」，库根不进键）。种类两样：
#     验证包之外的包是 merge-and-verify.sh:test-functions（dev 档：不交 --profile，cfg 带 debug_assertions）；验证包是 crash-point-replay.sh:test-functions，
#     请求照 admission.py test-functions-request crash-point-replay.sh 打的整份交（--profile release，与门禁 crash-point-replay 的 crash-case-replay 格同一种类：
#     这里判绿记下的那一格照键复用，那一格记下的这里照键复用），另加 --exclude-registered（登记的崩溃枚举用例与放量分项记 registered、不判、经 --skip 排掉）。
#     键由 admission.py test-plan 算（它文件头「单元键」）：测试函数的单元闭包、闭包真引到的 cfg 原子与这一次编译的真值、
#     工具链、解析出的闭包真引到的依赖版本、判法 lib_libtest_result_lines.py::recordable_results 的闭包（judge:）、改判定域的环境变量（admission.py test-functions-request 照这一刻的环境打的
#     variable:）。路径、名字、行号、提交与树不进键。每个要测、跑到的项所在的包一份计划（$logs/test-plan.<包>.out 与 .plan）；
#     包目录下一个 .rs 里都没有测试属性（#[test]、#[<路径>::test]）的不起 test-plan，计划是空的（没有测试函数可复用）。
#     键相同、库里不红的经 cargo test -- --exact --skip <名> 跳过（名单 admission.py plan-skip-names，各包只算点到的目标）；
#     键对不上与拒算的照跑。跑完照 rerun-failed-tests.py 的 section_item 把 test.log 逐段归到项与包（认不出、拿不准的段不记库，打一行），每个包那几段交 lib_libtest_result_lines.py recordable
#     逐条认结局、admission.py plan-green-names 取判绿可记的名字；FAILED 的测试函数按（目标根文件，libtest 全名）对到计划里要现判的那一行交 --red-names，与判绿的同一次 verdict-record 记进库：
#     红的照同一把键记红、撤掉这把键原有的绿，下一趟现判（记红撤绿）。文档测试与拒算的不复用、不入库（每趟照跑）。
#     逐条结局写进 $logs/test-functions.tsv：「<名字><TAB><项><TAB><现判|复用|拒算><TAB><绿|红|没跑到>」（现判 = 这一趟日志里有它的结果行）。
#     复用哨兵照 admission.py（test-plan 抽、verdict-record 判；被判的树下有 .unit-verdict-sample/ 的不抽）。
#     按条复用失灵（test-plan 退非 0 或没打 PLAN、verdict-record 报哨兵红）：照 admission.py plan-failure 判，失灵的那一包一条都不跳、全部现判跑完（验证包不起，见 test 一段），测试这一步判红；
#     它判 prune-server 不可用（退 75）的，测试这一步不起 cargo、报不可用，别的步照跑，整条退 75（另有红退 1）。
#     成功句报「按条复用：现判 N、复用 M、拒算 R」（跑到的项里计划的条数）；cargo test 退 0 而跑到的与复用的加起来是 0 才判「一条测试都没跑」。
#   红了只重跑红的：出路打 python3 scripts/gate/rerun-failed-tests.py <test.log> --item <红的项>…（红的测试函数归不到项的另打 -p <包> 那一种）；
#     修完也可以只重跑那几项的合入后验证：照出路打的那一条（范围声明种类 items、范围是项数，task-server/task submit -- bash scripts/gate/merge-and-verify.sh --skip-apply --item <红的项>… <同一批补丁目录>；
#     判绿记过库的照复用，只现判上次红的与改后键变了的）。
#   机读行（写法同 scripts/crash/crash-point-replay.sh 文件头「输出」，task-server/task_server_lib.py 的 ITEM_PLAN_LINE、ITEM_DONE_LINE 认）：
#     ITEM_PLAN 见上面 --list-items；ITEM_DONE item=<项> seconds=<秒> judged=<现判的件数> reused=<复用的件数> exit=<0|1|77>，真跑测试那一步收尾每项一行（stdout）：
#     秒数是这一项那几段 libtest 汇总行「finished in <秒>s」之和（只算测试跑的时间，编译几项共用一份、不分摊），四舍五入成整数；件数照这一包的计划（文档测试一件现判）；
#     exit 1 是这一项有 FAILED 的测试函数（cargo test 退非 0 而一条 FAILED 都归不到项时，跑到的每一项都记 1），77 是日志里没有这一项那一段（没跑到），别的 0。
# 弄坏开关 MERGE_AND_VERIFY_BREAK=<项>（只给证红用；打开时自证退 1）。逐项写开关名、弄坏哪一处、打开后自证哪几项红
# （自证的三例没有标签，项名直接写那条「✗ 自检：」失败行的句首，括号里写那一行接着报的）：
#   MERGE_AND_VERIFY_BREAK=logs-single  每一步的日志只写一份全量的（不经 two-logs.sh，没有 .problems.log）
#       「一条测试红时每一步的日志应当出两份（test.log 全量；test.problems.log 带行号收那条 FAILED）」
#   MERGE_AND_VERIFY_BREAK=tests-one-binary-at-a-time  测试那一步一条 cargo test 一个测试二进制接一个地跑，不经 test-targets-parallel.py
#       「测试那一步应当经 test-targets-parallel.py 按测试目标并行起」（实际假 cargo 只收到一条 test --offline --no-fail-fast -p demo --lib --test alpha）
#   MERGE_AND_VERIFY_BREAK=ignore-red  verify() 抓到 FAILED 行、cargo test 退非 0 都不记红，一律报测试全绿
#       「一条测试红时应当退 1、列出那条测试与只重跑它的命令」（实际退出码 0，报『✓ 测试全绿（0 个测试二进制）』）；
#       连带「快照：测试红时也应当删掉快照与编译目录」（那一格的测试红被吞掉，退 0 不是要的 1）
#       连带红的格：「✗ 自检：一条测试红时每一步的日志」、
#       「✗ 自检：测试那一步…按测试目标并行起」（红的那一趟要退 1）、
#       「逐项与复用：记红撤绿」（FAKE_UNITS_RED 那一趟退 0）
#   SINGLEFS_WORK_ROOT_BREAK=new-root-unknown（lib_work_root.py 的开关：工作根退回旧根 /tmp/claude-<uid>）  没设日志目录时日志落进旧根
#       「没设日志目录时日志应当落在工作根下」「自检：编译目录不在工作根下」（后一格也不给日志目录、工作根指进 scratch，日志目录落进旧根，那一格核日志目录照旧在工作根下那一半判不过；
#       跑完删掉旧根下这两格多出的两个 merge-and-verify/<日期>-<进程号>）
#   MERGE_AND_VERIFY_BREAK=snapshot-keeps-times  快照带源树的修改时刻拷（rsync --times）
#       「快照：MERGE_AND_VERIFY_KEEP_COPY=1 时应当留着快照与编译目录、打出路径，快照里的修改时刻是拷的那一刻」（实际留下的快照里那份文件是源树的 2020 年的 mtime）；
#       连带「往编过的副本里 rsync -a 铺新内容（旧 mtime）之后再验，应当重编改过的 walk.rs、测试全绿」（实际退出码 1，假 cargo 报 Fresh、那一条 FAILED）、
#       「编过之后 --skip-apply 再验同一批补丁：改过的 demo 应当重编、退 0」（实际快照里 walk.rs 比上一趟的 dep-info 旧、没有 Compiling demo，退 1）
#   MERGE_AND_VERIFY_BREAK=verify-in-main-tree  不拷快照，打补丁、编、验、验证包都在要验的树（主工作区）上
#       「快照：跑着的时候主工作区里那份源码被改，结论与复用键应当照开跑时那一份算」（实际测试编的是改后的 source_edited_mid_run、验证包那一步起的是改后的那一份门禁）；
#       连带「快照：测试红时也应当删掉快照与编译目录」「快照：全绿时应当在快照里打补丁、收尾删掉快照与编译目录」（没有删了快照那一行，apply 收到的 --root 是要验的树）、
#       「快照：MERGE_AND_VERIFY_KEEP_COPY=1 时……」「快照：快照根应当 git init」（没有快照）、
#       「往编过的副本里 rsync -a 铺新内容（旧 mtime）之后再验……」「编过之后 --skip-apply 再验同一批补丁……」（树里是旧 mtime，假 cargo 报 Fresh）
#   MERGE_AND_VERIFY_BREAK=drop-discipline-lint  从 check.sh 读到的 lint 表丢掉最后一条再交给 clippy
#       「clippy 应当带上 check.sh 的 lint 表里每一条」（实际少一条 -D clippy::shadow_unrelated 这类的）
#   MERGE_AND_VERIFY_BREAK=lint-table-optional  check.sh 读不出 lint 表、特性组合、四条 cargo 命令，或结构行对不上时只提一句、照常往下验
#       「check.sh 里读不出 lint 表时应当退 1、不编」与「check.sh 的 clippy 那一行没带 lint 表时应当退 1、不编」（实际退 0、照常编）；
#       「check.sh 的 fmt 那一条换成认不出的包装时应当退 1、不编」与「check.sh 的 test 那一行带了收窄目标的标志 --lib 时应当退 1、不编」（同上）；
#       「check.sh 的缺省特性组合只剩一个时应当退 1、不编」与「check.sh 的 clippy 循环改成只取一个组合时应当退 1、不编」（同上）
#   MERGE_AND_VERIFY_BREAK=cargo-wrapper-fixed  读 check.sh 里 cargo 命令的识别只认 run_cargo <子命令> 那一种包装，不带包装的 cargo 一律当找不到（上游去掉 run_cargo 那一天整个挡住）
#       「cargo 命令现读：真 check.sh 同一趟 --skip-apply 应当退 0」（真 check.sh 此刻是不带包装的 cargo，读不出 fmt 那一条，退 1、不编）；
#       连带红的格：「✗ 自检：」（凡是经真 check.sh 往下验的格都停在读 check.sh 那一步，没有一格验得到后面）
#   MERGE_AND_VERIFY_BREAK=cargo-wrapper-unrecognized  读 cargo 命令的识别只认不带包装的 cargo，run_cargo <子命令> 当找不到（上游哪天改回包装函数就挡住）
#       「check.sh 的 cargo 命令包了 run_cargo 时应当照读、收到的命令与真 check.sh 相同」（样本里四条都是 run_cargo <子命令>，读不出，退 1、不编）
#   MERGE_AND_VERIFY_BREAK=cargo-arguments-fixed  四条 cargo 命令的参数用这里写死的，不用从 check.sh 现读
#       「check.sh 的四条 cargo 命令各加一个标志时应当照读、收到的命令带上它们」（实际收到的是不带 --verbose、-W clippy::pedantic、--locked 的）
#   MERGE_AND_VERIFY_BREAK=single-clippy-pass  clippy 那一步只跑 --all-features 一趟，从 check.sh 读到的特性组合不用
#       「clippy 应当照 check.sh 的特性组合逐个跑」（实际假 cargo 只收到 --all-features 那一条）；
#       「clippy 一个特性组合红时应当整步红、退 1、点名红的那个组合」（不带特性那一趟没跑，实际退 0）；
#       连带「check.sh 的四条 cargo 命令各加一个标志时应当照读、收到的命令带上它们」（每个组合的 clippy 都带 -W，实际只收到一条）
#   MERGE_AND_VERIFY_BREAK=clippy-combinations-unchecked  check.sh 的缺省特性组合读出不到两个也照收
#       「check.sh 的缺省特性组合只剩一个时应当退 1、不编」（实际退 0、只跑那一个组合）
#   MERGE_AND_VERIFY_BREAK=clippy-combination-red-uncounted  一个特性组合红时照打那一行，不记进整步的红
#       「clippy 一个特性组合红时应当整步红、退 1、点名红的那个组合」（实际退 0，没有整步红那一句）
#   MERGE_AND_VERIFY_BREAK=clippy-registration-ignored  要验的树里有 check.sh 认的特性组合登记也照读缺省组合往下验
#       「要验的树里有特性组合登记时应当退 1、不编」（实际退 0、照常编）
#   MERGE_AND_VERIFY_BREAK=skip-build  不跑 cargo build --offline --all-targets
#       「应当跑 cargo build --offline --all-targets」（实际假 cargo 没收到 build）；
#       连带「check.sh 的四条 cargo 命令各加一个标志时应当照读、收到的命令带上它们」（没有 build --locked 那一条）、
#       「判不出测试目标：」那四格（它们另核别的步照跑、假 cargo 收到 build）
#   MERGE_AND_VERIFY_BREAK=mutation-tables-claimed-judged  汇总写「其中 code-source-discipline 红 0 个」，不报 mutation-tables 格本次未跑、不打另交的命令
#       「打补丁之后应当报 code-source-discipline 的 mutation-tables 格本次未跑、打出补丁打进主工作区之后另交的整条命令」（实际汇总写红 0 个，没有那一行）
#   MERGE_AND_VERIFY_BREAK=no-verification-package-guard  工作区成员里找不到 singlefs-verification 也照挑全部成员
#       「工作区成员里没有 singlefs-verification 时应当退 1、不编」（实际退 0，改了名的 验证包被跑进来）
#   MERGE_AND_VERIFY_BREAK=zero-tests-green  cargo test 退 0 而一条测试都没数到时照报全绿
#       「cargo test 退 0 却一条测试都没跑时应当退 1」（实际退 0）
#   MERGE_AND_VERIFY_BREAK=keep-copy  收尾不删快照与编译目录、也不打路径（快照留在工作根下没人删）
#       「快照：全绿时应当在快照里打补丁、收尾删掉快照与编译目录」与「快照：测试红时也应当删掉快照与编译目录」（实际两份都还在）；
#       连带「快照：MERGE_AND_VERIFY_KEEP_COPY=1 时……」（开关在留现场那一句之前就退出，路径不打）、
#       「跑完运行目录里不剩它建的编译目录……」（全绿、测试红、被 TERM 停三趟的快照与编译目录都还在）、
#       「这一趟的编译目录里还有进程开着文件……」（不删也不核，退 0）、「收尾时前面一步写满盘（用开关演）编译目录照删……」（编译目录还在）；
#       「逐项与复用：reuse-three-questions」「逐项与复用：整趟每项一行 ITEM_DONE」「逐项与复用：记红撤绿」（同一个日志目录的下一趟碰上留下的快照，退 2）
#   MERGE_AND_VERIFY_BREAK=cleanup-only-on-success  只在退 0 时删快照与编译目录，红了留着
#       「快照：测试红时也应当删掉快照与编译目录」（实际退 1、两份都还在）；
#       连带「跑完运行目录里不剩它建的编译目录……」（测试红与被 TERM 停那两趟的快照与编译目录都还在）、
#       「逐项与复用：记红撤绿」（红的那一趟留下快照，下一趟退 2）
#   MERGE_AND_VERIFY_BREAK=skip-rebuild-check  编完不核改过的包有没有 Compiling 行
#       「编译目录里的产物比源码新（共用编译目录）、补丁改过的 demo 没有 Compiling 行：应当退 1、点名 demo」（实际退 0）
#       连带：「头一次编、补丁改过的 demo 有 Compiling 行：应当退 0、报都重编」「编过之后 --skip-apply 再验同一批补丁：改过的 demo 应当重编、退 0」（不核就不报「都重编了」，判这一句的两格跟着红）
#   MERGE_AND_VERIFY_BREAK=skip-apply-unchecked  --skip-apply 时不核补丁打过没有
#       「--skip-apply 给的补丁没打在树上：应当退 1、点名那份补丁、不编」（实际退 0、照常编，在没打补丁的树上验出绿）；
#       连带「补丁改到 alpha 的库与 beta 的测试目标 case：…」「--skip-apply 给的两份补丁改同一行、照打的次序给：…」与「临时目录不认保留属性时 --skip-apply 照样核得过补丁已打上、退 0」（这三格另核『核过 N 份补丁都已打在树上』这一行，不核就不打）
#   MERGE_AND_VERIFY_BREAK=reverse-check-each-alone  --skip-apply 在要验的树上逐份单独 git apply --check --reverse，不在临时副本里照次序倒着逐份倒
#       「--skip-apply 给的两份补丁改同一行、照打的次序给：应当判都已打上、退 0」（实际先打的那一份单独反着核不过，退 1）
#   MERGE_AND_VERIFY_BREAK=reverse-check-copy-preserves  --skip-apply 核补丁打没打的临时副本带 -p 拷（保留权限与属主）
#       「临时目录不认保留属性时 --skip-apply 照样核得过补丁已打上、退 0」（实际 cp 报 preserving permissions … Operation not supported，退 1）
#   MERGE_AND_VERIFY_BREAK=keep-run-target  收尾不删这一趟自己建的编译目录（编进树根的 target、跨趟留着）
#       「跑完运行目录里不剩它建的编译目录」（正常、出错、被 TERM 三种结局，实际编译目录还在）；
#       连带「快照：全绿时应当在快照里打补丁、收尾删掉快照与编译目录」与「快照：测试红时也应当删掉快照与编译目录」（编译目录还在）、
#       「这一趟的编译目录里还有进程开着文件：应当不删、报出那个进程、退 1」（不删也不核，退 0）、
#       「收尾时前面一步写满盘（用开关演）编译目录照删」（编译目录还在）
#   MERGE_AND_VERIFY_BREAK=target-under-work-root  编译目录建在 <日志目录>/target（日志目录在工作根下，派到第二台时上百 G 经 NFS 写回本机的盘）
#       「编译目录不在工作根下」（实际在 <工作根>/merge-and-verify/<日期>-<进程号>/target）
#   MERGE_AND_VERIFY_BREAK=cleanup-after-other-steps  EXIT trap 先删快照，最后才删编译目录（前面一步写满盘停住，编译目录就留下了）
#       「收尾时前面一步写满盘（用开关演）编译目录照删」（实际编译目录还在）
#   MERGE_AND_VERIFY_BREAK=run-target-open-files-unchecked  删这一趟的编译目录之前不核有没有进程开着里面的文件（run_build_directory.py remove --skip-open-check）
#       「这一趟的编译目录里还有进程开着文件：应当不删、报出那个进程、退 1」（实际删了、退 0）
#   MERGE_AND_VERIFY_BREAK=rocksdb-entry-unchecked  本机起四条 cargo 之前不调 scripts/hosts/rocksdb-prebuilt.py entry
#       「起 cargo 之前 RocksDB 预编库的取值缺了……应当退 1、一条 cargo 都不起」（实际假 cargo 收到四条）
#   MERGE_AND_VERIFY_BREAK=copy-not-git-initialized  拷完快照之后不在快照根 git init（快照不是 git 工作树，读写判定库的脚本在快照上退 2）
#       「快照：快照根应当 git init」（实际留下的快照取不到 git common-dir）；
#       连带「逐项与复用：reuse-three-questions」「逐项与复用：整趟每项一行 ITEM_DONE」「逐项与复用：点名单跑 alpha:tests/beta.rs」「逐项与复用：记红撤绿」
#       「逐项与复用：--list-items --with-keys 每项一行 ITEM_PLAN」（admission.py test-plan 在快照上退 2，按条复用失灵、测试那一步判红，一条都没记库）
#   MERGE_AND_VERIFY_BREAK=test-arguments-added  测试那一步的 cargo test 在 -- 之后多带一个参数（--show-output）
#       「测试那一步的 cargo test 在 -- 之后应当什么都不带」（实际假 cargo 收到的测试命令以 -- --show-output 收尾）；
#       连带逐字比假 cargo 收到的测试命令的各格：「cargo test 应当只起补丁改到的 demo 的库目标」「测试那一步应当经 test-targets-parallel.py 按测试目标并行起」
#       「补丁改到 alpha 的库与 beta 的测试目标 case：…」「补丁改到 alpha 的 src/lib.rs、变异行点名 alpha 的 case_a：…」「check.sh 的四条 cargo 命令各加一个标志时应当照读、收到的命令带上它们」
#       「验证包那一步：补丁只改 demo 一个函数、变异行点名验证包的 setting…」「验证包那一步：补丁改到验证包的库与挂特性的 gpu_pair…」（多了 -- 之后的参数就对不上）
#   挑测试目标（文件头「挑测试目标」）的开关：
#   MERGE_AND_VERIFY_BREAK=whole-package-tests  改到的包与工作区里依赖它们的包，每个目标都挑（照整包跑）
#       「补丁改到 alpha 的库与 beta 的测试目标 case：…」「补丁改到 alpha 的 src/lib.rs、变异行点名 alpha 的 case_a：…」（实际 alpha 与依赖它的 beta 的目标都起了）；
#       连带「验证包那一步：补丁只改 demo 一个函数、变异行点名验证包的 setting…」（另起了依赖 demo 的 delta、验证包的库，交了 feature-gated:lib）、
#       「验证包那一步：补丁改到验证包的库与挂特性的 gpu_pair…」（验证包那一条另带 --test setting）、
#       「验证包那一步：跑在第二台上时…」「验证包那一步：--crash-expand-up-to 6 …」「验证包那一步：不给 --crash-expand-up-to 时…」「验证包那一步：环境里已设展开深度变量时…」
#       （那一道多起了一次 feature-gated:lib：起的次数、记下的变量行数对不上）
#   MERGE_AND_VERIFY_BREAK=fallback-runs-whole-workspace  判不出测试目标的不拒，改成每个包的每个目标都挑（照旧全跑）
#       「判不出测试目标：」那四格（没给补丁目录、改 Cargo.lock、改不落在任何包里的路径、改到落不到目标的文件：实际退 0、假 cargo 收到各包的测试命令）
#   验证包那一步（文件头「验证包那一步」）的开关，点名的格都以「验证包那一步：」起头：
#   MERGE_AND_VERIFY_BREAK=verification-step-skipped  不跑 验证包那一步
#       「验证包那一步：补丁只改 demo 一个函数、变异行点名验证包的 setting…」「验证包那一步：补丁改到验证包的库与挂特性的 gpu_pair…」
#       「验证包那一步：按块那一路判红时…」「验证包那一步：补丁只改到 alpha、beta 时…」「验证包那一步：补丁改到产品 GPU 包的真卡对拍 real_card 时…」
#       「验证包那一步：那一道退 3 而根不是它的样本根…」「验证包那一步：那一道 --list-items 退 1 时…」「验证包那一步：跑在第二台上时…」
#       「验证包那一步：--crash-expand-up-to 6 …」「验证包那一步：不给 --crash-expand-up-to 时…」「验证包那一步：环境里已设展开深度变量时…」「验证包那一步：缩过域的那一趟红了…」
#       （那一道一次都没被起、报不出那一步的行）；连带「快照：跑着的时候主工作区里那份源码被改……」（日志里没有按块那一路的那一行）
#   MERGE_AND_VERIFY_BREAK=whole-verification-cells  有要交的就起那一道的三格整格（--check crash-case-replay,feature-gated-tests,product-gpu-real-card-tests，不带 --item）
#       「验证包那一步：补丁只改 demo 一个函数、变异行点名验证包的 setting…」（实际那一道收到的是三格整格）、
#       「验证包那一步：补丁改到验证包的库与挂特性的 gpu_pair…」「验证包那一步：补丁改到产品 GPU 包的真卡对拍 real_card 时…」（同上）、
#       「验证包那一步：按块那一路判红时…」（出路里只重跑的命令不带 --item crash-case:demo-case）、「验证包那一步：跑在第二台上时…」（报的是三格整格那一次，不是按块那一路）
#   MERGE_AND_VERIFY_BREAK=crash-cases-whole-cell  崩溃枚举用例那一次不带 --crash-cases-only（门禁照跑整个包的按条复用那一趟）
#       「验证包那一步：补丁只改 demo 一个函数、变异行点名验证包的 setting…」「验证包那一步：按块那一路判红时…」（实际收到与打出的都是 --check crash-case-replay --item …）
#   MERGE_AND_VERIFY_BREAK=feature-gated-unrouted  挂特性的测试文件一样都不交
#       「验证包那一步：补丁改到验证包的库与挂特性的 gpu_pair…」（实际那一道只被起了 --list-items 那一次）
#   MERGE_AND_VERIFY_BREAK=product-gpu-unrouted  产品 GPU 包要特性的目标也当别的包，报本次未跑、不交真卡对拍那一格
#       「验证包那一步：补丁改到产品 GPU 包的真卡对拍 real_card 时…」（实际那一道一次都没被起）
#   MERGE_AND_VERIFY_BREAK=verification-release-dropped  验证包那一条 cargo test 不带 --release
#       「验证包那一步：补丁只改 demo 一个函数、变异行点名验证包的 setting…」「验证包那一步：补丁改到验证包的库与挂特性的 gpu_pair…」（实际收到的验证包测试命令没有 --release）
#   MERGE_AND_VERIFY_BREAK=verification-step-sample-exit-anywhere  那一道退 3 不看根是不是样本根，一律算过
#       「验证包那一步：那一道退 3 而根不是它的样本根…」（实际退 0）
#   MERGE_AND_VERIFY_BREAK=unknown-option-accepted  认不出的选项也照收、不拒（当没给）
#       「给了认不出的选项 --never-used-option 应当退 2、点名它」（实际照往下验、退 0）；连带「给了 --all-packages（不认的参数）应当退 2、点名它、一样都不做」
#   MERGE_AND_VERIFY_BREAK=rerun-command-wrapped  只重跑验证包那一步红的那几项的出路套 capped.sh 与 run-with-memory-cap.sh <上限>，不经 task-server/task submit
#       「验证包那一步：按块那一路判红时…」（出路里没有 task-server/task submit --、带了 run-with-memory-cap.sh）；
#       连带「验证包那一步：缩过域的那一趟红了…」（出路里没有 task-server/task submit -- env …）
#   MERGE_AND_VERIFY_BREAK=rerun-whole-cells  只重跑的出路打三格整格（不带 --item）
#       「验证包那一步：按块那一路判红时…」（出路里没有 --check crash-case-replay --crash-cases-only --item crash-case:demo-case，范围也不是 2 项）
#   MERGE_AND_VERIFY_BREAK=verification-step-red-ignored  那一道红了也报过
#       「验证包那一步：按块那一路判红时…」与「验证包那一步：那一道退 3 而根不是它的样本根…」（实际都退 0）；
#       连带「验证包那一步：缩过域的那一趟红了…」（实际退 0、没有只重跑的出路）
#   MERGE_AND_VERIFY_BREAK=verification-step-failed-lines-ignored  数红的测试函数只认「test <名> ... FAILED」，不认那一道逐个红的测试目标抄下的「FAILED <名>」
#       「验证包那一步：日志里两种写法的红应当一起数、按名去重……」（实际只数出 1 条）
#   MERGE_AND_VERIFY_BREAK=expand-up-to-not-passed  给了 --crash-expand-up-to 不把变量交给那一道（汇总照写缩过域）
#       「验证包那一步：--crash-expand-up-to 6 应当把 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6 交给那一道、汇总写缩过域、不算门禁通过」（实际那一道收到的是 unset）
#   MERGE_AND_VERIFY_BREAK=scaled-run-unlabelled  缩过域的那一趟汇总里不写「缩过域、不算门禁通过」
#       「验证包那一步：--crash-expand-up-to 6 应当把 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6 交给那一道、汇总写缩过域、不算门禁通过」（实际汇总里没有那一句）；
#       连带「验证包那一步：环境里已设展开深度变量时当作给了同一个值」「验证包那一步：缩过域的那一趟红了…」
#   MERGE_AND_VERIFY_BREAK=rerun-command-unscaled  缩过域的那一趟红了，只重跑的命令不带变量
#       「验证包那一步：缩过域的那一趟红了…」
#   MERGE_AND_VERIFY_BREAK=expand-up-to-unchecked  --crash-expand-up-to 的值与环境里的变量不核是不是非负整数
#       「--crash-expand-up-to 不是非负整数时应当退 2、一样都不做」（参数写 abc 与环境里的变量写 abc 两格，实际照往下验）
#   逐项点名与按条复用（文件头「逐项点名与按条复用」）的开关，点名的格都以「逐项与复用：」起头：
#   MERGE_AND_VERIFY_BREAK=items-ignored  --item 点了也照整趟跑（全部项）
#       「逐项与复用：点名单跑 alpha:tests/beta.rs」（实际假 cargo 收到的测试命令不带 --test beta、alpha 的库也跑了）
#   MERGE_AND_VERIFY_BREAK=item-unknown-accepted  --item 写错的项悄悄丢掉、不退 2
#       「逐项与复用：--item 写错的项应当退 2」（实际一项都没点，照整趟跑、退 0）
#   MERGE_AND_VERIFY_BREAK=reuse-skips-dropped  键相同的测试函数不经 --skip 跳过（照跑，计划照算）
#       「逐项与复用：reuse-three-questions」（第二趟、闭包外改动那一趟复用的那几条都现判了，same-inputs 与 unrelated-change 两问 red）
#   MERGE_AND_VERIFY_BREAK=red-not-recorded  判红的测试函数不交 verdict-record --red-names（判绿的照记）
#       「逐项与复用：记红撤绿」（上一趟判红的 uses_helper 下一趟照键复用了，没现判）
#   MERGE_AND_VERIFY_BREAK=plan-failure-ignored  按条复用失灵（plan-failure 退 1）不判红，照旧全部现判跑完当绿
#       「逐项与复用：按条复用失灵（test-plan 退 1）应当判红」（实际退 0）
#   MERGE_AND_VERIFY_BREAK=plan-unavailable-run-anyway  test-plan 判 prune-server 不可用（plan-failure 退 75）时照全部现判起 cargo、不退 75
#       「逐项与复用：prune-server 不可用（test-plan 退 75）应当退 75、测试那一步不起 cargo」（实际退 0、假 cargo 收到测试命令）
#   MERGE_AND_VERIFY_BREAK=item-done-unreported  测试那一步收尾不打 ITEM_DONE
#       「逐项与复用：整趟每项一行 ITEM_DONE」与「逐项与复用：点名单跑 alpha:tests/beta.rs」（找不到 ITEM_DONE 行）
#   MERGE_AND_VERIFY_BREAK=item-plan-reusable-zero  --list-items --with-keys 的 ITEM_PLAN 里 reusable 一律 0（键相同的测试函数不数）
#       「逐项与复用：--list-items --with-keys 每项一行 ITEM_PLAN」（记过库之后 alpha:src/lib.rs 的 reusable 该是 2，实际 0）
#   本库 scripts/gate/lib_rebuilt_packages.py 的 REBUILT_PACKAGES_BREAK=any-compiling-line 也让「编译目录里的产物比源码新……」那一格红（另一个包有 Compiling 行就当 demo 也重编了）；
#   它的 REBUILT_PACKAGES_BREAK=absolute-dep-info-only 让「头一次编、补丁改过的 demo 有 Compiling 行……」「编过之后 --skip-apply 再验同一批补丁……」
#     「编译目录里的产物比源码新（共用编译目录）、补丁改过的 demo 没有 Compiling 行……」三格红（假 cargo 写的 dep-info 是相对路径，只拿绝对路径比就一个都对不上，
#     三格都报「重编核对本次未判」、不报都重编了也不点名 demo；拆分提案第一百零八节 ③）；
set -uo pipefail
# 弄坏开关只在自证里认（checks-owed.md C622），参数解析之前先过这一道：命令行第一个词是 --selftest，或祖先里有一个的命令行里带 --selftest（自证起的子进程）才放行；
# 否则下面这些变量有设了的，点名它、说它只在自证里认，退 2
#   MERGE_AND_VERIFY_SELFTEST_FULL_DISK 也只在自证里认：设了，EXIT trap 删完编译目录之后、删快照之前演一步写满盘、整条收尾停在那里（「收尾时前面一步写满盘」那一格用）
BREAK_VARIABLES=(MERGE_AND_VERIFY_BREAK MERGE_AND_VERIFY_SELFTEST_FULL_DISK)
inside_selftest() { # 往上数这个进程的祖先，有一个的命令行里带 --selftest 就退 0
  local pid="$PPID" stat words word fields
  while ((pid > 1)); do
    mapfile -d '' -t words 2> /dev/null < "/proc/$pid/cmdline" || return 1
    for word in "${words[@]}"; do [[ "$word" == --selftest ]] && return 0; done
    IFS= read -r stat 2> /dev/null < "/proc/$pid/stat" || return 1
    read -ra fields <<< "${stat##*) }"
    pid="${fields[1]:-0}"
  done
  return 1
}
settle_break_switches() { # settle_break_switches <命令行的词…>
  local name setting=()
  for name in "${BREAK_VARIABLES[@]}"; do [[ -n "${!name:-}" ]] && setting+=("$name"); done
  ((${#setting[@]} == 0)) && return 0
  [[ "${1:-}" == --selftest ]] && return 0
  inside_selftest && return 0
  echo "  ✗ ${setting[*]} 是弄坏开关，只在自证（--selftest 本身与它起的子进程）里认；平常跑设了就拒，这一次什么都没做" >&2
  echo "     → 怎么办：unset ${setting[*]} 再来；要看开关红在哪一项，跑 ${setting[0]}=<项> bash scripts/gate/merge-and-verify.sh --selftest" >&2
  exit 2
}
settle_break_switches "$@"
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../.." && pwd)"
source "$here/../hosts/heavy-threads.sh"
APPLY="${MERGE_AND_VERIFY_APPLY:-$here/apply-writer-patch.py}"
TWO_LOGS="${MERGE_AND_VERIFY_TWO_LOGS:-$here/../hosts/two-logs.sh}"   # 每一步的日志出两份（run_step）
TEST_TARGETS_PARALLEL="$here/test-targets-parallel.py"   # 测试那一步按测试目标并行起（文件头 test 一段）
STAMP="$here/../hosts/source-stamp.py"          # 自证里写按 mtime 判新不新鲜的假 cargo（--write-selftest-cargo）
REBUILT="$here/lib_rebuilt_packages.py"          # 改过的包重编了没有（文件头）
CHECK_PATCH_DIRS=()                              # 算改过的包用的补丁目录（绝对路径；--skip-apply 时也留着）
RUN_BUILD_DIRECTORY="$here/../hosts/run_build_directory.py"   # 删这一趟自己建的编译目录（文件头「这一趟的编译目录」）
VERIFICATION_STAGE_PATH=".claude/gate.d/crash-point-replay.sh"   # 验证包那一步起的门禁在树里的位置（文件头「验证包那一步」）
VERIFICATION_STAGE=""; VERIFICATION_RERUN_STAGE=""   # 建好快照之后填：起的是快照里那一份，出路那条命令写主工作区那一份（MERGE_AND_VERIFY_VERIFICATION_STAGE 换掉两样，只给自证）
SOURCE_TREE=""                                    # 要验的树（快照从它拷；出路那条命令交给门禁的根），参数解析之后填
# 验证包那一步点的三格与只走按块那一路的选项（文件头「验证包那一步」）；WHOLE_VERIFICATION_CELLS 只给弄坏开关 whole-verification-cells 与 rerun-whole-cells 用
CRASH_CASE_CELL="crash-case-replay"; FEATURE_GATED_CELL="feature-gated-tests"; PRODUCT_GPU_CELL="product-gpu-real-card-tests"
CRASH_CASES_ONLY_OPTION="--crash-cases-only"
WHOLE_VERIFICATION_CELLS="$CRASH_CASE_CELL,$FEATURE_GATED_CELL,$PRODUCT_GPU_CELL"
VERIFICATION_STATE=""; VERIFICATION_SCALED=""     # verification_step 填：汇总行里的一句；缩过域的那一趟置 1
CRASH_EXPAND_UP_TO=""                             # 缩崩溃域的展开深度上限（--crash-expand-up-to，或环境里已设的变量）；空就是整个域
CRASH_EXPAND_UP_TO_VARIABLE=SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO   # 交给那一道与验证包目标的 cargo test 的变量名（引擎读它，值只往小缩）
# 挑测试目标（文件头「挑测试目标」），select_test_targets 填
ALL_ITEM_LINES=()            # 挑出的项，一项一行「项<TAB>包<TAB>挑目标的参数」
ITEM_REASONS=()              # 一项一行「项<TAB>为什么挑上」
REQUIRED_FEATURE_TARGETS=()  # 要特性的目标，一行「包<TAB>目标名<TAB>特性<TAB>为什么」
TARGET_REFUSALS=()           # 判不出测试目标的原因；有一条就不跑测试那一步与验证包那一步、整条退 3
TARGET_NOTES=()              # 报一句的：变异行的筛选词、补丁删掉的目标
TARGET_PROBLEM=""            # 读不了补丁、成员清单或目标清单的原因
# 逐项点名与按条复用（文件头「逐项点名与按条复用」）
RERUN_FAILED_TESTS="$here/rerun-failed-tests.py"   # 项的清单与「日志里一段归哪一项」（它文件头「项的写法」），经 importlib 调
ADMISSION="$here/admission.py"                     # test-functions-request、test-plan、plan-skip-names、plan-green-names、verdict-record、plan-failure
LIBTEST_RESULT_LINES="$here/lib_libtest_result_lines.py"   # recordable：逐条认结局
LIST_ITEMS=0; LIST_WITH_KEYS=0; NAMED_ITEMS=()      # --list-items、--with-keys、--item 点名的项
RUN_ITEMS=()                                        # collect_test_items 填：这一趟测试那一步跑的项，一项一行「项<TAB>包<TAB>挑目标的参数」
TESTED_PACKAGES=()                                  # select_test_targets 填：挑出的项所在、包目录下有测试属性的包（只给它们起 test-plan）
TEST_FUNCTIONS_KIND=""; TEST_PLAN_ARGUMENTS=()      # plan_test_functions 填：验证包之外的判定种类与交给 test-plan 的参数（admission.py test-functions-request merge-and-verify.sh）
VERIFICATION_FUNCTIONS_KIND=""; VERIFICATION_PLAN_ARGUMENTS=()   # plan_test_functions 填：验证包的（test-functions-request crash-point-replay.sh，另加 --exclude-registered）
TESTS_REFUSED=0                                     # 判不出测试目标，测试那一步与验证包那一步没跑（文件头「退出码」3）
PLANNED_PACKAGES=(); FAILED_PLAN_PACKAGES=()        # 计划算成了的包、按条复用失灵的包
PLAN_UNAVAILABLE=0; PLAN_RED=0                      # prune-server 不可用、按条复用失灵（测试这一步判红）
PLAN_JUDGE=0; PLAN_REUSE=0; PLAN_REFUSED=0          # 跑到的项里计划的条数（成功句）
CHECK_SH="${MERGE_AND_VERIFY_CHECK_SH:-$root/.claude/singlefs-ai-sop/scripts/check.sh}"
VERIFICATION_PACKAGE=singlefs-verification   # 验证包（.claude/rules/verification.md）：它的目标带 --release 跑、按门禁 crash-point-replay 的种类复用，登记的崩溃枚举用例归 verification_step
# check.sh 里 clippy 按特性组合循环的三行结构（不是 cargo 命令本身）：每行在它里面恰好出现一次，这里的 clippy 一步照它们对齐
# （组合 default 不带特性参数；登记文件那一行是 check.sh 认登记的地方，见文件头 clippy 一段）。
# fmt、clippy、build、test 四条 cargo 命令的写法不在这里抄第二份：开跑时从 check.sh 现读（read_cargo_command，文件头 clippy 一段之后的「cargo 命令现读」）。
CHECK_SH_STRUCTURE_LINES=(
  'CLIPPY_COMBINATIONS_FILE="$ROOT/$CLIPPY_FEATURE_COMBINATIONS_FILE"'
  'for clippy_combination in "${CLIPPY_FEATURE_COMBINATIONS[@]}"; do'
  '[[ "$clippy_combination" == default ]] || read -r -a clippy_feature_arguments <<< "$clippy_combination"'
)
# check.sh 的 clippy 那一行里认得的两个占位词（别的带 $、引号、括号的词一律不认）：特性参数、编码纪律 lint 表
CLIPPY_FEATURES_PLACEHOLDER='${clippy_feature_arguments[@]+"${clippy_feature_arguments[@]}"}'
CLIPPY_LINTS_PLACEHOLDER='"${CODE_DISCIPLINE_LINTS[@]}"'
# check.sh 的 test 那一行里不许带的收窄目标的标志：这里的测试步自己按包、按测试目标挑，check.sh 那一行收窄了就不是同一批用例
TEST_NARROWING_FLAGS=(--lib --bins --bin --tests --test --examples --example --benches --bench --doc --package --exclude)
# read_cargo_command 填 CARGO_COMMAND_WORDS（check.sh 那一行里子命令之后的词）；read_check_sh 分到：
CARGO_FMT_WORDS=(); CARGO_CLIPPY_WORDS=(); CARGO_BUILD_WORDS=(); CARGO_TEST_FLAGS=(); CARGO_COMMAND_WORDS=()   # CARGO_TEST_FLAGS：test 那一行除 --all / --workspace 之外带的标志
# check.sh 认的特性组合登记放在项目根下哪里：上游 lib.sh 的 CLIPPY_FEATURE_COMBINATIONS_FILE（check.sh source 的那一份），开跑时现读，不在这里另抄
CHECK_SH_LIBRARY="$root/.claude/singlefs-ai-sop/scripts/lib.sh"
CLIPPY_COMBINATIONS=(); CLIPPY_REGISTRATION=""   # read_check_sh 填：check.sh 的缺省特性组合与登记文件的相对路径

die() { echo "  ✗ $1"; echo "  → 怎么办：$2"; exit "${3:-2}"; }

# take_snapshot <要验的树> <快照>：文件头「快照」。快照目录这里新建（已有就退 1）；不拷任意一层的 .git、target、target-* 与 __pycache__；
# 不保留属性（不带 --perms、--owner、--group：权限位照建文件时的缺省，可执行位随源文件带过去）与修改时刻（不带 --times）。
# rsync 退 24（拷的时候有文件被别的会话删了）算拷成：那一刻已经没有它，快照里也就没有；别的非 0 退 1。弄坏开关 snapshot-keeps-times：带源树的修改时刻拷
take_snapshot() {
  local rc=0
  local -a times=(--no-times)
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != "snapshot-keeps-times" ]] || times=(--times)
  mkdir -- "$2" || return 1
  rsync --recursive --links "${times[@]}" --exclude .git --exclude 'target/' --exclude 'target-*/' --exclude '__pycache__/' -- "$1/" "$2/" || rc=$?
  ((rc == 24)) && echo "  ! rsync 退 24：拷的时候有文件不见了（别的会话删了它），快照里没有它们"
  ((rc == 0 || rc == 24))
}

# 收尾（EXIT trap，正常结束、红了退出、被 INT / TERM 停都走这里）：第一步删掉这一趟自己建的编译目录（文件头「这一趟的编译目录」），在别的任何一条命令之前，
# 删不成记退 1、照接着往下收；然后删掉这里建的快照（文件头「快照」）。
# MERGE_AND_VERIFY_KEEP_COPY=1 时快照与编译目录都留着、打路径。删不成退 1。
SNAPSHOT_TREE=""; RUN_TARGET=""
# 删这一趟自己建的编译目录（弄坏开关 keep-run-target：不删；run-target-open-files-unchecked：删之前不核开着的文件）
remove_run_target() {
  [[ -n "$RUN_TARGET" && "${MERGE_AND_VERIFY_BREAK:-}" != "keep-run-target" ]] || return 0
  python3 "$RUN_BUILD_DIRECTORY" remove "$RUN_TARGET" $([[ "${MERGE_AND_VERIFY_BREAK:-}" == "run-target-open-files-unchecked" ]] && echo --skip-open-check)
}
finish() {
  local rc=$? kept_size keep_both=0
  trap - EXIT
  # 快照与编译目录都留着的三种（弄坏开关 keep-copy、cleanup-only-on-success 红了的那一趟，MERGE_AND_VERIFY_KEEP_COPY=1）先判，只看变量、不起命令
  if [[ -n "$SNAPSHOT_TREE" ]] && { [[ "${MERGE_AND_VERIFY_BREAK:-}" == "keep-copy" ]] || { [[ "${MERGE_AND_VERIFY_BREAK:-}" == "cleanup-only-on-success" ]] && ((rc != 0)); }; }; then
    keep_both=1
  elif [[ "${MERGE_AND_VERIFY_KEEP_COPY:-}" == 1 ]]; then
    keep_both=2
  fi
  # 第一步删编译目录（前面一步写满盘时编译目录就删不成）；弄坏开关 cleanup-after-other-steps 挪到最后
  if ((keep_both == 0)) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != "cleanup-after-other-steps" ]]; then
    remove_run_target || rc=1
  fi
  cd / || true
  if [[ -n "${MERGE_AND_VERIFY_SELFTEST_FULL_DISK:-}" ]]; then   # 只给自证演：删编译目录之后、删快照那一步写满盘，整条收尾停在那里
    echo "  ✗ 收尾那一步写不进去：OSError: [Errno 28] No space left on device（自证演的，MERGE_AND_VERIFY_SELFTEST_FULL_DISK）"
    echo "  → 怎么办：这是自证造的局面；编译目录在这一步之前删过了才对"
    exit 1
  fi
  [[ -n "$SNAPSHOT_TREE" || -n "$RUN_TARGET" ]] || exit "$rc"
  if ((keep_both == 1)); then
    exit "$rc"
  fi
  if ((keep_both == 2)); then
    echo "  ! 留着${SNAPSHOT_TREE:+快照 $SNAPSHOT_TREE}${SNAPSHOT_TREE:+${RUN_TARGET:+ 与}}${RUN_TARGET:+编译目录 $RUN_TARGET}（MERGE_AND_VERIFY_KEEP_COPY=1）"
    echo "  → 看完自己删：rm -rf --${SNAPSHOT_TREE:+ '$SNAPSHOT_TREE'}${RUN_TARGET:+ '$RUN_TARGET'}"
    exit "$rc"
  fi
  if [[ -n "$SNAPSHOT_TREE" ]]; then
    kept_size="$(du -sh -- "$SNAPSHOT_TREE" 2>/dev/null | cut -f1)"
    rm -rf -- "${SNAPSHOT_TREE:?}"
    if [[ -e "$SNAPSHOT_TREE" ]]; then
      echo "  ✗ 快照 $SNAPSHOT_TREE 没删成"
      echo "  → 怎么办：看谁还开着里面的文件（python3 $RUN_BUILD_DIRECTORY holders '$SNAPSHOT_TREE'），停了再 rm -rf -- 那一份"
      rc=1
    else
      echo "  删了快照 $SNAPSHOT_TREE（共 ${kept_size:-?}；日志留在 $(dirname "$SNAPSHOT_TREE")）"
    fi
  fi
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "cleanup-after-other-steps" ]]; then
    remove_run_target || rc=1
  fi
  exit "$rc"
}

# check_patches_applied <日志目录>：--skip-apply 时核给的每份 crates.patch 都已打在要验的树上（文件头「--skip-apply 先核补丁打过没有」）；核不过退 1、不编。
check_patches_applied() {
  local logs="$1" dir patch checked_count=0 unapplied_text=""
  local -a unapplied=() present=()
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "skip-apply-unchecked" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=skip-apply-unchecked：--skip-apply 时不核补丁打过没有"; return 0
  fi
  if ((${#CHECK_PATCH_DIRS[@]} == 0)); then echo "  - 补丁打没打本次未核：带了 --skip-apply 而没给补丁目录"; return 0; fi
  mkdir -p "$logs" || die "建不了日志目录 $logs" "看工作根在不在、可不可写" 1
  for dir in "${CHECK_PATCH_DIRS[@]}"; do
    if [[ ! -d "$dir" ]]; then unapplied+=("$dir（目录不在）"); continue; fi
    patch="$dir/crates.patch"
    [[ -s "$patch" ]] || continue
    present+=("$dir")
  done
  if ((${#unapplied[@]})); then
    unapplied_text="$(printf '%s、' "${unapplied[@]}")"
    die "带了 --skip-apply，给的补丁目录有 ${#unapplied[@]} 个不在，核不了打没打在要验的树 $tree 上：${unapplied_text%、}；一个都没编" \
      "补丁目录写对再来；没打过的那几份去掉 --skip-apply 让这里打（打过的与没打过的分两趟）" 1
  fi
  checked_count=${#present[@]}
  if ((checked_count == 0)); then echo "  - 补丁打没打本次未核：给的 ${#CHECK_PATCH_DIRS[@]} 个补丁目录里都没有 crates.patch"; return 0; fi
  # 弄坏开关 reverse-check-each-alone：在要验的树上逐份单独 git apply --check --reverse（同一批两份补丁改到同一行时前一份反着核不过，误判没打）
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "reverse-check-each-alone" ]]; then
    for dir in "${present[@]}"; do
      git -C "$tree" apply --check --reverse "$dir/crates.patch" > "$logs/reverse-check-$(basename "$dir").log" 2>&1 \
        || unapplied+=("$dir（日志 $logs/reverse-check-$(basename "$dir").log）")
    done
    if ((${#unapplied[@]})); then
      unapplied_text="$(printf '%s、' "${unapplied[@]}")"
      die "带了 --skip-apply，给的补丁有 ${#unapplied[@]} 份没打在要验的树 $tree 上（弄坏开关 reverse-check-each-alone：逐份单独反着核）：${unapplied_text%、}；一个都没编" \
        "没打过的那几份去掉 --skip-apply 让这里打" 1
    fi
    echo "  核过 $checked_count 份补丁都已打在树上（弄坏开关 reverse-check-each-alone：逐份单独 git apply --check --reverse）"
    return 0
  fi
  reverse_check_in_order "$logs" "${present[@]}"
  case $? in
    0) ;;
    1) die "带了 --skip-apply，照给的次序倒着逐份 git apply --reverse，倒到 $REVERSE_CHECK_FAILED 倒不掉：这一份没打在要验的树 $tree 上，或给的次序与打的次序不同（它前面还有 $REVERSE_CHECK_SKIPPED 份没往下核）；一个都没编" \
         "没打过的那几份去掉 --skip-apply 让这里打（打过的与没打过的分两趟）；都打过而次序给错了的，照打的次序重给补丁目录" 1 ;;
    *) die "带了 --skip-apply，核补丁打没打用的临时副本没建成：$REVERSE_CHECK_FAILED；一个都没编" "看 ${TMPDIR:-/tmp} 在不在、可不可写、有没有空间，再重跑同一条" 1 ;;
  esac
  echo "  核过 $checked_count 份补丁都已打在树上（临时副本里照给的次序倒着逐份 git apply --reverse，全部倒得掉）"
}

# reverse_check_in_order <日志目录> <补丁目录>…：--skip-apply 核补丁打没打（文件头「--skip-apply 先核补丁打过没有」）。在 ${TMPDIR:-/tmp} 下新建一份临时副本
# （git init 的空仓，只拷要验的树里这几份 crates.patch 点到的文件），照给的次序倒着逐份 git apply --reverse：后打的先倒，同一批两份补丁改到同一行也倒得掉。
# 全部倒得掉退 0；有一份倒不掉退 1，那一份的补丁目录与日志进 REVERSE_CHECK_FAILED、它前面没往下核的份数进 REVERSE_CHECK_SKIPPED；副本建不成退 2（原因进 REVERSE_CHECK_FAILED）。
# 副本用完就删（三种结局都删）；要验的树一个字节不动。
reverse_check_in_order() {
  local logs="$1" copy dir path index log copy_log="$1/reverse-check-copy.log"
  local -a patch_dirs=("${@:2}") touched=()
  REVERSE_CHECK_FAILED=""; REVERSE_CHECK_SKIPPED=0
  if ! copy="$(mktemp -d "${TMPDIR:-/tmp}/merge-and-verify-reverse-check.XXXXXX")"; then
    REVERSE_CHECK_FAILED="${TMPDIR:-/tmp} 下建不了临时目录"; return 2
  fi
  # 补丁点到的路径（新旧两侧、Git 引号、重命名与拷贝）：与重编核对同一份读法（lib_rebuilt_packages.py 的 collect_paths），这里不另写
  if ! python3 - "$REBUILT" "${patch_dirs[@]}" > "$copy_log.paths" 2> "$copy_log" <<'PY_REVERSE_CHECK_PATHS'
import importlib.util, sys
library, *patch_directories = sys.argv[1:]
specification = importlib.util.spec_from_file_location("lib_rebuilt_packages_for_reverse_check", library)
rebuilt = importlib.util.module_from_spec(specification)
specification.loader.exec_module(rebuilt)
for path in rebuilt.collect_paths(patch_directories, []):
    print(path)
PY_REVERSE_CHECK_PATHS
  then
    REVERSE_CHECK_FAILED="读不出补丁点到的路径（日志 $copy_log）"; rm -rf -- "${copy:?}"; return 2
  fi
  mapfile -t touched < "$copy_log.paths"
  if ! git init -q "$copy" >> "$copy_log" 2>&1; then
    REVERSE_CHECK_FAILED="临时副本 $copy 里 git init 不成（日志 $copy_log）"; rm -rf -- "${copy:?}"; return 2
  fi
  for path in ${touched[@]+"${touched[@]}"}; do
    # 绝对路径与带 .. 的路径不拷（git apply 自己也拒它们，倒不掉照判）；树里没有的不拷（新建的文件倒着打要删它、不在就倒不掉）
    [[ "$path" != /* && "/$path/" != */../* ]] || continue
    [[ -e "$tree/$path" || -L "$tree/$path" ]] || continue
    # 不保留属性（--no-preserve=all）：有的临时目录不认保留权限，git apply 只看内容与可执行位，可执行位随建文件带过去
    local preserve="--no-preserve=all"; [[ "${MERGE_AND_VERIFY_BREAK:-}" == "reverse-check-copy-preserves" ]] && preserve="-p"
    if ! (cd "$tree" && cp --parents -P "$preserve" -- "$path" "$copy/") >> "$copy_log" 2>&1; then
      REVERSE_CHECK_FAILED="$path 拷不进临时副本 $copy（日志 $copy_log）"; rm -rf -- "${copy:?}"; return 2
    fi
  done
  for (( index = ${#patch_dirs[@]} - 1; index >= 0; index-- )); do
    dir="${patch_dirs[index]}"; log="$logs/reverse-check-$(basename "$dir").log"
    if ! git -C "$copy" apply --reverse "$dir/crates.patch" > "$log" 2>&1; then
      REVERSE_CHECK_FAILED="$dir（日志 $log）"; REVERSE_CHECK_SKIPPED=$index
      rm -rf -- "${copy:?}"; return 1
    fi
  done
  rm -rf -- "${copy:?}"
  return 0
}

# clippy_step_name <特性组合>：这个组合那一步的步名（日志 <日志目录>/<步名>.log）：clippy- 接组合去掉开头的 -、别的非字母数字换成 -
clippy_step_name() {
  local text="$1"
  while [[ "$text" == -* ]]; do text="${text#-}"; done
  printf 'clippy-%s' "${text//[^A-Za-z0-9]/-}"
}

# read_cargo_command <check.sh> <子命令>：文件头「cargo 命令现读」。check.sh 里命令位置上写 cargo <子命令> 或 run_cargo <子命令>（前面可带 if）的行恰好一条，
# 子命令之后到第一个 |、;、\ 之前的词（2>&1 不算）进 CARGO_COMMAND_WORDS。找不到、不止一条、词里带 $、引号、括号这类这里不会展开的写法
# （clippy 那一行的两个占位词除外）返回 1，原因进 CHECK_SH_PROBLEM，原因里写这一条哪一行、哪个词、怎么办。
# 弄坏开关 cargo-wrapper-fixed：只认 run_cargo 那一种包装；cargo-wrapper-unrecognized：只认不带包装的 cargo。
read_cargo_command() {
  local file="$1" subcommand="$2" line line_number=0 hits=0 hit_text="" hit_number=0 wrapper_pattern='run_cargo|cargo' word line_re
  local -a words=()
  CARGO_COMMAND_WORDS=()
  case "${MERGE_AND_VERIFY_BREAK:-}" in
    cargo-wrapper-fixed) wrapper_pattern='run_cargo' ;;
    cargo-wrapper-unrecognized) wrapper_pattern='cargo' ;;
    *) ;;
  esac
  line_re='^[[:space:]]*(if[[:space:]]+)?('"$wrapper_pattern"')[[:space:]]+'"$subcommand"'([[:space:]]+([^|;\\]*))?([|;\\].*)?$'
  while IFS= read -r line || [[ -n "$line" ]]; do
    line_number=$((line_number + 1))
    if [[ "$line" =~ $line_re ]]; then
      hits=$((hits + 1)); hit_number=$line_number; hit_text+="第 $line_number 行「${line#"${line%%[![:space:]]*}"}」；"
      read -r -a words <<< "${BASH_REMATCH[4]}"
    fi
  done < "$file"
  if ((hits == 0)); then
    CHECK_SH_PROBLEM="$file 里找不到 cargo $subcommand 那一条命令（命令位置上写 cargo $subcommand 或 run_cargo $subcommand 的行，数到 0 条）：上游把它换成了别的写法（另一个包装函数、存在变量里的命令名）；先在这份脚本的 read_cargo_command 里加这一种写法的识别，只加识别、不放宽核对"
    return 1
  fi
  if ((hits != 1)); then
    CHECK_SH_PROBLEM="$file 里 cargo $subcommand 命令写了 $hits 条（${hit_text%；}）：这里每一步只对一条，先决定上游多出来的那几条各对应这里的哪一步"
    return 1
  fi
  for word in ${words[@]+"${words[@]}"}; do
    [[ "$word" == '2>&1' ]] && continue
    if [[ "$subcommand" == clippy && ( "$word" == "$CLIPPY_FEATURES_PLACEHOLDER" || "$word" == "$CLIPPY_LINTS_PLACEHOLDER" ) ]]; then CARGO_COMMAND_WORDS+=("$word"); continue; fi
    case "$word" in
      *[\$\"\'\`\(\)\{\}\<\>\&\*\?\~]*)
        CHECK_SH_PROBLEM="$file 第 $hit_number 行的 cargo $subcommand 命令里有这里不会展开的词「$word」（带 \$、引号、括号这类）：先决定它在合入后验证里对应什么值，再在这份脚本的 read_cargo_command 里加这个词的处理"
        return 1 ;;
      *) ;;
    esac
    CARGO_COMMAND_WORDS+=("$word")
  done
  return 0
}

# read_check_sh <check.sh>：现读编码纪律那张 lint 表进 DISCIPLINE_LINTS、缺省的特性组合进 CLIPPY_COMBINATIONS、特性组合登记的相对路径进 CLIPPY_REGISTRATION，
# 现读 fmt、clippy、build、test 四条 cargo 命令的参数进 CARGO_FMT_WORDS、CARGO_CLIPPY_WORDS、CARGO_BUILD_WORDS、CARGO_TEST_FLAGS，
# 核 clippy 按组合循环的三行结构；读不出、对不上返回 1，原因进 CHECK_SH_PROBLEM。
read_check_sh() {
  local file="$1" line in_table=0 i command_text hits rest combination combination_lines step seen word narrowing features_count lints_count scope_count
  local combinations_line_re='^CLIPPY_FEATURE_COMBINATIONS=\((.*)\)[[:space:]]*$' quoted_word_re='^[[:space:]]*"([^"]*)"(.*)$'
  local -a words=() steps=()
  DISCIPLINE_LINTS=(); CLIPPY_COMBINATIONS=(); CLIPPY_REGISTRATION=""; CHECK_SH_PROBLEM=""
  CARGO_FMT_WORDS=(); CARGO_CLIPPY_WORDS=(); CARGO_BUILD_WORDS=(); CARGO_TEST_FLAGS=()
  [[ -f "$file" ]] || { CHECK_SH_PROBLEM="$file 不在"; return 1; }
  while IFS= read -r line || [[ -n "$line" ]]; do
    if ((in_table == 1)); then
      if [[ "$line" =~ ^\)[[:space:]]*$ ]]; then in_table=2; break; fi
      line="${line%%#*}"
      read -r -a words <<< "$line"
      DISCIPLINE_LINTS+=(${words[@]+"${words[@]}"})
    elif [[ "$line" =~ ^CODE_DISCIPLINE_LINTS=\([[:space:]]*$ ]]; then
      in_table=1
    fi
  done < "$file"
  if ((in_table != 2)); then CHECK_SH_PROBLEM="$file 里找不到 CODE_DISCIPLINE_LINTS=( … ) 那一段（起止各占一行）"; return 1; fi
  if ((${#DISCIPLINE_LINTS[@]} == 0 || ${#DISCIPLINE_LINTS[@]} % 2 != 0)); then
    CHECK_SH_PROBLEM="$file 的 CODE_DISCIPLINE_LINTS 读出 ${#DISCIPLINE_LINTS[@]} 个词，不是一条一对的「-D clippy::<名>」"; return 1
  fi
  for ((i = 0; i < ${#DISCIPLINE_LINTS[@]}; i += 2)); do
    if [[ ! "${DISCIPLINE_LINTS[i]}" =~ ^-[DWAF]$ || ! "${DISCIPLINE_LINTS[i+1]}" =~ ^clippy::[a-z0-9_]+$ ]]; then
      CHECK_SH_PROBLEM="$file 的 CODE_DISCIPLINE_LINTS 第 $((i / 2 + 1)) 条读成「${DISCIPLINE_LINTS[i]} ${DISCIPLINE_LINTS[i+1]}」，不是「-D clippy::<名>」的形状"; return 1
    fi
  done
  for command_text in "${CHECK_SH_STRUCTURE_LINES[@]}"; do
    hits="$(grep -cF -- "$command_text" "$file")"
    if [[ "$hits" != 1 ]]; then CHECK_SH_PROBLEM="$file 里「$command_text」出现 ${hits:-0} 次，不是 1 次：上游改了 clippy 按特性组合循环的写法，这一步要跟着改"; return 1; fi
  done
  # 四条 cargo 命令的参数现读（文件头「cargo 命令现读」）
  read_cargo_command "$file" fmt || return 1
  CARGO_FMT_WORDS=(${CARGO_COMMAND_WORDS[@]+"${CARGO_COMMAND_WORDS[@]}"})
  read_cargo_command "$file" build || return 1
  CARGO_BUILD_WORDS=(${CARGO_COMMAND_WORDS[@]+"${CARGO_COMMAND_WORDS[@]}"})
  read_cargo_command "$file" clippy || return 1
  CARGO_CLIPPY_WORDS=(${CARGO_COMMAND_WORDS[@]+"${CARGO_COMMAND_WORDS[@]}"})
  features_count=0; lints_count=0
  for word in ${CARGO_CLIPPY_WORDS[@]+"${CARGO_CLIPPY_WORDS[@]}"}; do
    [[ "$word" != "$CLIPPY_FEATURES_PLACEHOLDER" ]] || features_count=$((features_count + 1))
    [[ "$word" != "$CLIPPY_LINTS_PLACEHOLDER" ]] || lints_count=$((lints_count + 1))
  done
  if ((features_count != 1 || lints_count != 1)); then
    CHECK_SH_PROBLEM="$file 的 cargo clippy 命令里特性参数的占位词 $CLIPPY_FEATURES_PLACEHOLDER 出现 $features_count 次、编码纪律 lint 表的占位词 $CLIPPY_LINTS_PLACEHOLDER 出现 $lints_count 次，各要恰好 1 次：少了就是 clippy 没按特性组合跑、或没带 lint 表，与这里不是同一套"; return 1
  fi
  read_cargo_command "$file" test || return 1
  CARGO_TEST_FLAGS=(); scope_count=0
  for word in ${CARGO_COMMAND_WORDS[@]+"${CARGO_COMMAND_WORDS[@]}"}; do
    if [[ "$word" == --all || "$word" == --workspace ]]; then scope_count=$((scope_count + 1)); continue; fi
    for narrowing in "${TEST_NARROWING_FLAGS[@]}"; do
      if [[ "$word" == "$narrowing" ]]; then CHECK_SH_PROBLEM="$file 的 cargo test 命令带了收窄目标的标志 $word：这里的测试步自己按包、按测试目标挑，收窄了就不是同一批用例，先决定怎么对应"; return 1; fi
    done
    if [[ ! "$word" =~ ^--[a-z][a-z0-9-]*$ ]]; then
      CHECK_SH_PROBLEM="$file 的 cargo test 命令里有这里认不出的词「$word」（只认 --all 或 --workspace，加上不带值的 --标志）：先决定它在合入后验证里对应什么，再在这份脚本的 read_check_sh 里加"; return 1
    fi
    CARGO_TEST_FLAGS+=("$word")
  done
  if ((scope_count != 1)); then
    CHECK_SH_PROBLEM="$file 的 cargo test 命令里 --all 或 --workspace 出现 $scope_count 次，不是 1 次：不是整个工作区的测试，这里按「工作区成员减掉 验证包」挑包，对不上"; return 1
  fi
  # 缺省的特性组合（文件头 clippy 一段）：行首那一行 CLIPPY_FEATURE_COMBINATIONS=("…" …)，括号里只许双引号括起的词
  combination_lines="$(grep -c '^CLIPPY_FEATURE_COMBINATIONS=(' "$file")"
  if [[ "$combination_lines" != 1 ]]; then CHECK_SH_PROBLEM="$file 里行首的 CLIPPY_FEATURE_COMBINATIONS=( 出现 ${combination_lines:-0} 次，不是 1 次：上游改了缺省特性组合的写法"; return 1; fi
  line="$(grep '^CLIPPY_FEATURE_COMBINATIONS=(' "$file")"
  if [[ ! "$line" =~ $combinations_line_re ]]; then CHECK_SH_PROBLEM="$file 的缺省特性组合那一行认不出（要写在一行里、以右括号收尾）：$line"; return 1; fi
  rest="${BASH_REMATCH[1]}"
  while [[ "$rest" =~ $quoted_word_re ]]; do CLIPPY_COMBINATIONS+=("${BASH_REMATCH[1]}"); rest="${BASH_REMATCH[2]}"; done
  if [[ -n "${rest//[[:space:]]/}" ]]; then CHECK_SH_PROBLEM="$file 的缺省特性组合那一行有认不出的写法「${rest}」（每个组合用双引号括起）：$line"; return 1; fi
  for combination in ${CLIPPY_COMBINATIONS[@]+"${CLIPPY_COMBINATIONS[@]}"}; do
    if [[ "$combination" != default && "$combination" != -* ]]; then CHECK_SH_PROBLEM="$file 的缺省特性组合「$combination」认不出（只认 default 与以 - 起头的特性参数）"; return 1; fi
    step="$(clippy_step_name "$combination")"
    for seen in ${steps[@]+"${steps[@]}"}; do
      if [[ "$seen" == "$step" ]]; then CHECK_SH_PROBLEM="$file 的缺省特性组合里有两个组合的步名都是 $step（日志名会撞）"; return 1; fi
    done
    steps+=("$step")
  done
  # 弄坏开关 clippy-combinations-unchecked：不到两个也照收
  if ((${#CLIPPY_COMBINATIONS[@]} < 2)) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != "clippy-combinations-unchecked" ]]; then
    CHECK_SH_PROBLEM="$file 的缺省特性组合只读出 ${#CLIPPY_COMBINATIONS[@]} 个（${CLIPPY_COMBINATIONS[*]-}），不到两个：check.sh 退回了单趟 clippy"; return 1
  fi
  # 特性组合登记的相对路径：上游 lib.sh 的 CLIPPY_FEATURE_COMBINATIONS_FILE
  hits="$(grep -c '^CLIPPY_FEATURE_COMBINATIONS_FILE=' "$CHECK_SH_LIBRARY" 2>/dev/null)"
  CLIPPY_REGISTRATION="$(sed -n 's/^CLIPPY_FEATURE_COMBINATIONS_FILE=//p' "$CHECK_SH_LIBRARY" 2>/dev/null)"
  if [[ "$hits" != 1 || -z "$CLIPPY_REGISTRATION" ]]; then
    CHECK_SH_PROBLEM="$CHECK_SH_LIBRARY 里读不出唯一一行 CLIPPY_FEATURE_COMBINATIONS_FILE=<路径>（出现 ${hits:-0} 次）：算不出 check.sh 认的特性组合登记在哪"; return 1
  fi
  return 0
}

# verification_member_present <树>：工作区成员逐个写出目录、各有 [package] name，验证包在成员里（文件头 test 一段：认不出验证包就排不掉登记的崩溃枚举用例）；
#   不是返回 1，原因进 MEMBERS_PROBLEM。弄坏开关 no-verification-package-guard：成员里找不到验证包也照往下验
verification_member_present() {
  local listing
  MEMBERS_PROBLEM=""
  if ! listing="$(python3 - "$1" "$VERIFICATION_PACKAGE" "${MERGE_AND_VERIFY_BREAK:-}" <<'PY_MEMBERS' 2>&1
import os, sys, tomllib
tree, wanted, broken = sys.argv[1], sys.argv[2], sys.argv[3]
def fail(text):
    print(text); sys.exit(1)
try:
    with open(os.path.join(tree, "Cargo.toml"), "rb") as handle:
        members = tomllib.load(handle).get("workspace", {}).get("members")
except (OSError, tomllib.TOMLDecodeError) as error:
    fail(f"读不了 {tree}/Cargo.toml：{error}")
if not isinstance(members, list) or not members:
    fail(f"{tree}/Cargo.toml 没有 [workspace] members")
names = []
for member in members:
    if not isinstance(member, str) or any(mark in member for mark in "*?["):
        fail(f"成员「{member}」写成了通配或不是字符串，这里只认逐个写出的目录")
    try:
        with open(os.path.join(tree, member, "Cargo.toml"), "rb") as handle:
            name = tomllib.load(handle).get("package", {}).get("name")
    except (OSError, tomllib.TOMLDecodeError) as error:
        fail(f"读不了成员 {member} 的 Cargo.toml：{error}")
    if not isinstance(name, str) or not name:
        fail(f"成员 {member} 的 Cargo.toml 没有 [package] name")
    names.append(name)
if wanted not in names and broken != "no-verification-package-guard":
    fail(f"工作区成员（{'、'.join(names)}）里没有 {wanted}：验证包改了名或挪了，认不出它就排不掉登记的崩溃枚举用例")
PY_MEMBERS
  )"; then
    MEMBERS_PROBLEM="$listing"; return 1
  fi
  return 0
}

# select_test_targets <树>：挑测试目标（文件头「挑测试目标」，判法是 MERGE_AND_VERIFY_ITEMS_PROGRAM 的 targets 一段）：挑出的项进 ALL_ITEM_LINES 与 ITEM_REASONS，
#   包目录下有测试属性的进 TESTED_PACKAGES，要特性的进 REQUIRED_FEATURE_TARGETS，判不出的进 TARGET_REFUSALS，报一句的进 TARGET_NOTES；读不了返回 1、原因进 TARGET_PROBLEM。
#   弄坏开关 whole-package-tests、fallback-runs-whole-workspace 在 targets 那一段里认
select_test_targets() {
  local listing kind first second third fourth
  ALL_ITEM_LINES=(); ITEM_REASONS=(); TESTED_PACKAGES=(); REQUIRED_FEATURE_TARGETS=(); TARGET_REFUSALS=(); TARGET_NOTES=(); TARGET_PROBLEM=""
  if ! listing="$(python3 - targets "$1" "$RERUN_FAILED_TESTS" "$REBUILT" "${MERGE_AND_VERIFY_BREAK:-}" ${CHECK_PATCH_DIRS[@]+"${CHECK_PATCH_DIRS[@]}"} \
       <<<"$MERGE_AND_VERIFY_ITEMS_PROGRAM" 2>&1)"; then
    TARGET_PROBLEM="$(tail -n 3 <<<"$listing" | tr '\n' ' ')"; return 1
  fi
  while IFS=$'\t' read -r kind first second third fourth; do
    case "$kind" in
      REFUSE) TARGET_REFUSALS+=("$first") ;;
      ITEM) ALL_ITEM_LINES+=("$first"$'\t'"$second"$'\t'"$third"); ITEM_REASONS+=("$first"$'\t'"$fourth") ;;
      TESTED) TESTED_PACKAGES+=("$first") ;;
      REQUIRES) REQUIRED_FEATURE_TARGETS+=("$first"$'\t'"$second"$'\t'"$third"$'\t'"$fourth") ;;
      NOTE) TARGET_NOTES+=("$first") ;;
    esac
  done <<<"$listing"
  return 0
}

# report_target_selection：挑出的项与为什么、报一句的、要特性的逐行打（文件头「挑测试目标」）；判不出的不在这里打（verify 里打红句与出路）
report_target_selection() {
  local line
  local -a fields=()
  ((${#TARGET_REFUSALS[@]})) && return 0
  echo "  挑测试目标：${#ALL_ITEM_LINES[@]} 项（补丁改到的与变异行点名的；不跑整个包，也不带上依赖它们的包）"
  for line in ${ITEM_REASONS[@]+"${ITEM_REASONS[@]}"}; do printf '      %s：%s\n' "${line%%$'\t'*}" "${line#*$'\t'}"; done   # gate-lint:detail
  for line in ${REQUIRED_FEATURE_TARGETS[@]+"${REQUIRED_FEATURE_TARGETS[@]}"}; do
    IFS=$'\t' read -r -a fields <<<"$line"
    printf '      %s 的 %s 要特性 %s（%s）：测试那一步不跑，产品 GPU 包的交验证包那一步\n' "${fields[0]}" "${fields[1]}" "${fields[2]}" "${fields[3]}"   # gate-lint:detail
  done
  for line in ${TARGET_NOTES[@]+"${TARGET_NOTES[@]}"}; do printf '      · %s\n' "$line"; done   # gate-lint:detail
  return 0
}

run_step() {   # $1 日志文件，其余是命令：在这一整条里面直接起（不套内存包装与 capped.sh，上限与线程由起这一整条的服务定，文件头「上限、线程、机器」）
  local log="$1" rc
  shift
  # 每一步的输出出两份日志（scripts/hosts/two-logs.sh --quiet：$log 是全量，同名的 .problems.log 只收错误与警告，行前带全量日志里的行号；不转打，汇总行点名它们）。
  # 弄坏开关 logs-single：只写一份全量日志
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "logs-single" ]]; then "$@" > "$log" 2>&1; rc=$?
  else bash "$TWO_LOGS" --quiet "${log%.log}" -- "$@"; rc=$?; fi
  return "$rc"
}

# verification_rerun_command <交给门禁的参数…>：只重跑验证包那一步红的那几项的整条命令（文件头「验证包那一步」；起主工作区那一份门禁、交给它的根是 VERIFICATION_RERUN_ROOT）：
#   范围声明写在 task-server/task 前面（种类 items，范围数是 --check 点名的格数加 --item 的项数，同 .claude/hooks/run-scope-guard.sh 文件头「门禁阶段」），
#   命令经 task-server/task submit -- 交（不套包装、不写上限与线程，文件头「上限、线程、机器」）；缩过域的那一趟带同一个变量（env 接在 submit -- 之后：变量不在服务的白名单里，写在命令里原样跑）。
#   弄坏开关 rerun-command-unscaled：缩过域的那一趟也不带变量；rerun-command-wrapped：套 capped.sh 与内存包装、不经 task-server；rerun-whole-cells：打整格（三格、不带 --item）
verification_rerun_command() {
  local word cells="" previous="" cell_count item_count text why="合入后验证这几项红了，修完只重跑它们：键相同的照复用，只现判红的那几条与改后键变了的"
  local -a arguments=("$@") scale_words=() items=()
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "rerun-whole-cells" ]]; then arguments=(--check "$WHOLE_VERIFICATION_CELLS"); fi
  for word in "${arguments[@]}"; do
    [[ "$previous" == --check ]] && cells="$word"
    [[ "$previous" == --item ]] && items+=("$word")
    previous="$word"
  done
  if [[ -n "$CRASH_EXPAND_UP_TO" && "${MERGE_AND_VERIFY_BREAK:-}" != "rerun-command-unscaled" ]]; then
    scale_words=(env "$CRASH_EXPAND_UP_TO_VARIABLE=$CRASH_EXPAND_UP_TO")
    why+="（缩过域：展开深度上限 $CRASH_EXPAND_UP_TO，不算门禁通过）"
  fi
  cell_count="$(tr ',' '\n' <<< "$cells" | grep -c .)"; item_count=${#items[@]}
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "rerun-command-wrapped" ]]; then
    text="$(printf '%q ' bash "$root/scripts/hosts/capped.sh" "$THREADS" bash "$root/scripts/hosts/run-with-memory-cap.sh" 36G bash "$VERIFICATION_RERUN_STAGE" "${arguments[@]}" "$VERIFICATION_RERUN_ROOT")"
  else
    text="$(printf '%q ' "$root/task-server/task" submit -- ${scale_words[@]+"${scale_words[@]}"} bash "$VERIFICATION_RERUN_STAGE" "${arguments[@]}" "$VERIFICATION_RERUN_ROOT")"
  fi
  printf "SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 crash-point-replay.sh 的 %s 格%s; 范围=%s 项（%s 格加点名的 %s 项）; 为什么=%s' %s" \
    "${cells//,/、}" "$( ((item_count)) && printf '点名的 %s' "${items[*]}")" "$((cell_count + item_count))" "$cell_count" "$item_count" "$why" "${text% }"
}

# verification_red_test_names <日志>：那一道日志里红的测试函数名，逐行、去重。认两种写法：cargo 的「test <名> ... FAILED」，
# 与那一道逐个红的测试目标抄下的「FAILED <名>」（汇总行「FAILED. N passed」不算）；只认前一种会把几条红数成 1 条
# 弄坏开关 verification-step-failed-lines-ignored：只认前一种
verification_red_test_names() {
  {
    sed -n 's/^ *test \(.*\) \.\.\. FAILED$/\1/p' "$1"
    [[ "${MERGE_AND_VERIFY_BREAK:-}" == "verification-step-failed-lines-ignored" ]] || sed -n 's/^ *FAILED \([^ .][^ ]*\)$/\1/p' "$1"
  } | LC_ALL=C sort -u
}

# verification_routes <门禁 --list-items 的日志，没有写 ->：这一趟跑到的验证包目标里要另走门禁的几样（文件头「验证包那一步」）：
#   崩溃枚举用例进 VERIFICATION_CASE_ITEMS（crash-case:<用例名>），挂特性的进 VERIFICATION_FEATURE_ITEMS（feature-gated:<文件名>、feature-gated:lib）；
#   产品 GPU 包要特性的目标置 VERIFICATION_PRODUCT_GPU，别的包要特性的目标进 VERIFICATION_UNROUTED。
#   弄坏开关 feature-gated-unrouted：挂特性的一样都不交；product-gpu-unrouted：产品 GPU 包的也当别的包
verification_routes() {
  local listing="$1" line item rest package selector key value target product_gpu_package=""
  local -a targets=() fields=()
  VERIFICATION_CASE_ITEMS=(); VERIFICATION_FEATURE_ITEMS=(); VERIFICATION_PRODUCT_GPU=0; VERIFICATION_UNROUTED=(); VERIFICATION_PRODUCT_GPU_WHY=""
  for line in ${RUN_ITEMS[@]+"${RUN_ITEMS[@]}"}; do
    item="${line%%$'\t'*}"; rest="${line#*$'\t'}"; package="${rest%%$'\t'*}"; selector="${rest#*$'\t'}"
    [[ "$package" == "$VERIFICATION_PACKAGE" ]] && targets+=("$selector")
  done
  if [[ "$listing" != - ]]; then
    while IFS=$'\t' read -r key value; do
      case "$key" in
        crash-case:*)
          target="${value#*:}"; target="${target%%:*}"
          if word_in_list "--test $target" ${targets[@]+"${targets[@]}"} || { [[ "$target" == lib ]] && word_in_list --lib ${targets[@]+"${targets[@]}"}; }; then
            VERIFICATION_CASE_ITEMS+=("$key")
          fi ;;
        feature-gated:lib) word_in_list --lib ${targets[@]+"${targets[@]}"} && VERIFICATION_FEATURE_ITEMS+=("$key") ;;
        feature-gated:*.rs) target="${key#feature-gated:}"; word_in_list "--test ${target%.rs}" ${targets[@]+"${targets[@]}"} && VERIFICATION_FEATURE_ITEMS+=("$key") ;;
      esac
    done < "$listing"
  fi
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != feature-gated-unrouted ]] || VERIFICATION_FEATURE_ITEMS=()
  ((${#REQUIRED_FEATURE_TARGETS[@]})) || return 0
  product_gpu_package="$(sed -n 's/^PRODUCT_GPU_PACKAGE_NAME="\([^"$`]*\)".*/\1/p' "$VERIFICATION_STAGE" 2>/dev/null | head -n 1)"
  for line in "${REQUIRED_FEATURE_TARGETS[@]}"; do
    IFS=$'\t' read -r -a fields <<<"$line"
    if [[ "${MERGE_AND_VERIFY_BREAK:-}" != product-gpu-unrouted ]] && { [[ -z "$product_gpu_package" ]] || [[ "${fields[0]}" == "$product_gpu_package" ]]; }; then
      VERIFICATION_PRODUCT_GPU=1
      VERIFICATION_PRODUCT_GPU_WHY="${fields[0]} 的 ${fields[1]}（要特性 ${fields[2]}；${fields[3]}）$([[ -z "$product_gpu_package" ]] && echo "；$(basename "$VERIFICATION_STAGE") 里读不出 PRODUCT_GPU_PACKAGE_NAME=\"<包名>\" 那一行，照起")"
    else
      VERIFICATION_UNROUTED+=("${fields[0]} 的 ${fields[1]}（要特性 ${fields[2]}）")
    fi
  done
  return 0
}

# verification_call <说明> <过了要读到的行：blocks 或 functions> <日志> <交给门禁的参数…>：验证包那一步起门禁的一次（文件头「验证包那一步」），
#   根是 VERIFICATION_CALL_ROOT。过交 0（现判、复用、拒算加进 VERIFICATION_JUDGED、VERIFICATION_REUSED、VERIFICATION_REFUSED，崩溃状态块那一行进 VERIFICATION_BLOCKS），
#   本次未跑交 77，红交 1（打红的测试函数、红的项与只重跑红的那几项的命令）。
#   弄坏开关 verification-step-sample-exit-anywhere：退 3 不看根是不是样本根一律算过；verification-step-red-ignored：红了也报过
verification_call() {
  local label="$1" expect="$2" log="$3" rc counts judged reused refused passes block_text seen=0 name word previous=""
  shift 3
  local -a arguments=("$@") red_tests=() red_items=() named=() rerun_arguments=()
  echo "  验证包那一步（$label）：起 $(basename "$VERIFICATION_STAGE") ${arguments[*]}，只现判键对不上的、键相同的复用${CRASH_EXPAND_UP_TO:+；展开深度上限缩到 $CRASH_EXPAND_UP_TO（缩过域、不算门禁通过）}…"
  rm -f -- "$log"
  run_step "$log" env -u MERGE_AND_VERIFY_BREAK ${VERIFICATION_SCALE_WORDS[@]+"${VERIFICATION_SCALE_WORDS[@]}"} bash "$VERIFICATION_STAGE" "${arguments[@]}" "$VERIFICATION_CALL_ROOT"; rc=$?
  if ((rc == 77)); then
    VERIFICATION_NOT_RUN+=("$label")
    echo "  - 验证包那一步（$label）本次未跑：$(basename "$VERIFICATION_STAGE") 退 77（$(grep -E '^ *!' "$log" | tail -n 1 | sed 's/^ *//')），不记红、不算过"
    return 77
  fi
  counts="$(sed -nE 's/^ *· (feature-gated-tests |product-gpu-real-card-tests )?按条复用：.*现判 ([0-9]*)、复用 ([0-9]*)、拒算 ([0-9]*).*/\2 \3 \4/p' "$log")"
  read -r judged reused refused passes < <(awk 'NF { judged += $1; reused += $2; refused += $3; passes++ } END { print judged + 0, reused + 0, refused + 0, passes + 0 }' <<< "$counts")
  block_text="$(sed -nE 's/^ *✓ 崩溃状态块：现判 ([0-9]+)、复用 ([0-9]+)、拒算 ([0-9]+).*/现判 \1、复用 \2、拒算 \3/p' "$log" | head -n 1)"
  if [[ "$expect" == blocks ]]; then [[ -n "$block_text" ]] && seen=1; else ((passes > 0)) && seen=1; fi
  if { { ((rc == 0)) || { ((rc == 3)) && { [[ -d "$VERIFICATION_CALL_ROOT/.crash-point-replay-sample-tools" ]] || [[ "${MERGE_AND_VERIFY_BREAK:-}" == "verification-step-sample-exit-anywhere" ]]; }; }; } && ((seen)); } \
     || [[ "${MERGE_AND_VERIFY_BREAK:-}" == "verification-step-red-ignored" ]]; then
    VERIFICATION_JUDGED=$((VERIFICATION_JUDGED + judged)); VERIFICATION_REUSED=$((VERIFICATION_REUSED + reused)); VERIFICATION_REFUSED=$((VERIFICATION_REFUSED + refused))
    [[ -z "$block_text" ]] || VERIFICATION_BLOCKS="$block_text"
    echo "  ✓ 验证包那一步（$label）过（$(basename "$VERIFICATION_STAGE") 退 $rc$( ((rc == 3)) && echo '，样本档：假 cargo 走的流程')；$( [[ "$expect" == blocks ]] && echo "崩溃状态块 ${block_text:-读不到}" || echo "按条现判 $judged、复用 $reused、拒算 $refused，读到 $passes 趟按条复用的那一行")；日志 $log）"
    grep -m 1 '^ *· 逐格：' "$log" | sed 's/^ */    /'
    return 0
  fi
  mapfile -t red_tests < <(verification_red_test_names "$log")
  mapfile -t red_items < <(sed -nE 's/^ITEM_DONE item=([^ ]+) .* exit=1$/\1/p' "$log" | LC_ALL=C sort -u)
  for word in "${arguments[@]}"; do [[ "$previous" == --item ]] && named+=("$word"); previous="$word"; done
  # 只重跑红的：ITEM_DONE 里 exit=1 的项落在这一次点的项里就只点它们，认不出的点这一次的全部项（文件头「验证包那一步」）
  previous=""
  for word in "${arguments[@]}"; do
    if [[ "$word" == --item ]]; then previous=--item; continue; fi
    if [[ "$previous" == --item ]]; then previous=""; continue; fi
    rerun_arguments+=("$word")
  done
  for name in ${named[@]+"${named[@]}"}; do
    if ((${#red_items[@]} == 0)) || word_in_list "$name" "${red_items[@]}"; then rerun_arguments+=(--item "$name"); fi
  done
  echo "  ✗ 验证包那一步（$label）红（$(basename "$VERIFICATION_STAGE") 退 $rc$( ((rc == 3)) && echo '：退 3 只在样本根上算过，交给它的根不是样本根')$( ((seen == 0)) && echo "，日志里读不到$([[ "$expect" == blocks ]] && echo '崩溃状态块' || echo '按条复用')那一行（现判、复用多少判不了）")；红的项：${red_items[*]:-认不出}；日志 $log、错误警告 ${log%.log}.problems.log）：FAILED 的测试函数 ${#red_tests[@]} 条"
  for name in ${red_tests[@]+"${red_tests[@]}"}; do echo "      $name"; done   # gate-lint:detail
  grep -E '^ *红的 |^ *error: test failed, to rerun pass' "$log" | sed 's/^ *//' | LC_ALL=C sort -u | head -n 40 | sed 's/^/      /'   # gate-lint:detail
  ((${#red_tests[@]})) || grep -E '^ *✗' "$log" | head -n 5 | sed 's/^ */      /'   # gate-lint:detail
  echo "  → 怎么办：FAILED 的那几条测试函数并进写段的下一批派 implementation-writer 改；改完只重跑红的：$(verification_rerun_command "${rerun_arguments[@]}")（键相同的照复用：判绿记过库的不重跑，只现判上面红的这几条与改后键变了的）"
  return 1
}

# verification_step <日志目录>：验证包那一步（文件头「验证包那一步」）。过与本次未跑交 0、红交 1；汇总行里的一句进 VERIFICATION_STATE，缩过域的进 VERIFICATION_SCALED。
#   弄坏开关 verification-step-skipped：不跑；whole-verification-cells：有要交的就起那一道的三格整格；crash-cases-whole-cell：崩溃枚举用例那一次不带 --crash-cases-only；
#   expand-up-to-not-passed：缩过域也不把变量交给那一道；scaled-run-unlabelled：缩过域的那一趟汇总里不写
verification_step() {
  local logs="$1" listing="-" rc reds=0 item verification_targets=0 line joined
  local -a case_arguments=() feature_arguments=() parts=()
  VERIFICATION_CALL_ROOT="${MERGE_AND_VERIFY_VERIFICATION_ROOT:-$tree}"; VERIFICATION_RERUN_ROOT="${MERGE_AND_VERIFY_VERIFICATION_ROOT:-$SOURCE_TREE}"
  VERIFICATION_JUDGED=0; VERIFICATION_REUSED=0; VERIFICATION_REFUSED=0; VERIFICATION_BLOCKS=""; VERIFICATION_NOT_RUN=(); VERIFICATION_SCALE_WORDS=()
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "verification-step-skipped" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=verification-step-skipped：不跑 验证包那一步"; return 0
  fi
  if ((${#TARGET_REFUSALS[@]})); then VERIFICATION_STATE="没跑（判不出测试目标，同测试那一步）"; return 0; fi
  if [[ -n "$CRASH_EXPAND_UP_TO" && "${MERGE_AND_VERIFY_BREAK:-}" != "expand-up-to-not-passed" ]]; then
    VERIFICATION_SCALE_WORDS=("$CRASH_EXPAND_UP_TO_VARIABLE=$CRASH_EXPAND_UP_TO")
  fi
  for line in ${RUN_ITEMS[@]+"${RUN_ITEMS[@]}"}; do
    line="${line#*$'\t'}"; [[ "${line%%$'\t'*}" == "$VERIFICATION_PACKAGE" ]] && verification_targets=$((verification_targets + 1))
  done
  # 登记的崩溃枚举用例与挂特性的测试文件照那一道的 --list-items 现读（不起 cargo）
  if ((verification_targets)); then
    listing="$logs/verification-list-items.log"
    run_step "$listing" env -u MERGE_AND_VERIFY_BREAK bash "$VERIFICATION_STAGE" --list-items "$VERIFICATION_CALL_ROOT"; rc=$?
    if ((rc != 0)); then
      VERIFICATION_STATE="红（$(basename "$VERIFICATION_STAGE") --list-items 退 $rc）"
      echo "  ✗ 验证包那一步：$(basename "$VERIFICATION_STAGE") --list-items 退 $rc（日志 $listing、错误警告 ${listing%.log}.problems.log），读不出跑到的 $verification_targets 个验证包目标里登记的崩溃枚举用例与挂特性的测试文件"
      echo "  → 怎么办：照那份日志修（那一道的登记表、tests/ 里挂特性的文件读不读得了），再重跑同一条命令"
      return 1
    fi
  fi
  if item_mode && ((${#REQUIRED_FEATURE_TARGETS[@]})); then
    VERIFICATION_NOT_RUN+=("产品 GPU 包的真卡对拍（点了 --item）")
  fi
  verification_routes "$listing"
  ! item_mode || VERIFICATION_PRODUCT_GPU=0
  if ((${#VERIFICATION_CASE_ITEMS[@]} + ${#VERIFICATION_FEATURE_ITEMS[@]} + VERIFICATION_PRODUCT_GPU == 0)); then
    VERIFICATION_STATE="本次未跑（跑到的目标里没有登记的崩溃枚举用例与挂特性的测试文件，也没有产品 GPU 包的真卡对拍）"
    echo "  - 验证包那一步本次未跑：这一趟跑到的 $verification_targets 个验证包目标里没有登记的崩溃枚举用例与挂特性的测试文件，补丁也没改到产品 GPU 包的真卡对拍（不记红、不算过）${CRASH_EXPAND_UP_TO:+；给的展开深度上限 $CRASH_EXPAND_UP_TO 没交给门禁}${VERIFICATION_UNROUTED[*]:+；要特性、本次未跑的：${VERIFICATION_UNROUTED[*]}}"
    return 0
  fi
  for item in ${VERIFICATION_CASE_ITEMS[@]+"${VERIFICATION_CASE_ITEMS[@]}"}; do case_arguments+=(--item "$item"); done
  for item in ${VERIFICATION_FEATURE_ITEMS[@]+"${VERIFICATION_FEATURE_ITEMS[@]}"}; do feature_arguments+=(--item "$item"); done
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "whole-verification-cells" ]]; then
    verification_call "三格整格（弄坏开关 whole-verification-cells）" functions "$logs/verification-step.log" --check "$WHOLE_VERIFICATION_CELLS"
    (($? == 1)) && reds=$((reds + 1))
  else
    if ((${#case_arguments[@]})); then
      if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "crash-cases-whole-cell" ]]; then
        verification_call "登记的崩溃枚举用例" blocks "$logs/verification-crash-cases.log" --check "$CRASH_CASE_CELL" "${case_arguments[@]}"
      else
        verification_call "登记的崩溃枚举用例" blocks "$logs/verification-crash-cases.log" --check "$CRASH_CASE_CELL" "$CRASH_CASES_ONLY_OPTION" "${case_arguments[@]}"
      fi
      (($? == 1)) && reds=$((reds + 1))
    fi
    if ((${#feature_arguments[@]})); then
      verification_call "挂特性的测试文件" functions "$logs/verification-feature-gated.log" --check "$FEATURE_GATED_CELL" "${feature_arguments[@]}"
      (($? == 1)) && reds=$((reds + 1))
    fi
    if ((VERIFICATION_PRODUCT_GPU)); then
      echo "  · 产品 GPU 包的真卡对拍：$VERIFICATION_PRODUCT_GPU_WHY"
      verification_call "产品 GPU 包的真卡对拍" functions "$logs/verification-product-gpu.log" --check "$PRODUCT_GPU_CELL"
      (($? == 1)) && reds=$((reds + 1))
    fi
  fi
  # 缩过域的那一趟汇总里写明（弄坏开关 scaled-run-unlabelled：不写）
  if [[ -n "$CRASH_EXPAND_UP_TO" && "${MERGE_AND_VERIFY_BREAK:-}" != "scaled-run-unlabelled" ]]; then VERIFICATION_SCALED=1; fi
  ((${#VERIFICATION_UNROUTED[@]})) && VERIFICATION_NOT_RUN+=("要特性的 ${VERIFICATION_UNROUTED[*]}")
  parts=("按条现判 $VERIFICATION_JUDGED、复用 $VERIFICATION_REUSED")
  [[ -z "$VERIFICATION_BLOCKS" ]] || parts+=("崩溃状态块 $VERIFICATION_BLOCKS")
  ((${#VERIFICATION_NOT_RUN[@]})) && parts+=("本次未跑：${VERIFICATION_NOT_RUN[*]}")
  joined="$(printf '%s；' "${parts[@]}")"; joined="${joined%；}"
  if ((reds)); then VERIFICATION_STATE="红（$reds 次门禁判红；$joined）"; return 1; fi
  VERIFICATION_STATE="过（$joined）"
  return 0
}

# ── 挑测试目标、逐项点名与按条复用（文件头「挑测试目标」「逐项点名与按条复用」）────────────
# 用的 python 段（python3 - <一样> … <<<"$MERGE_AND_VERIFY_ITEMS_PROGRAM"），照第一个参数做一样：
#   targets <树> <rerun-failed-tests.py> <lib_rebuilt_packages.py> <弄坏开关> <补丁目录>…   挑测试目标（文件头「挑测试目标」）：判不出的每处一行「REFUSE<TAB>原因」，
#        挑出的每项一行「ITEM<TAB>项<TAB>包<TAB>挑目标的参数<TAB>为什么挑上」（照包在工作区里的次序、包里目标的次序），挑出的项所在而包目录下有测试属性的「TESTED<TAB>包」，
#        要特性的目标「REQUIRES<TAB>包<TAB>目标名<TAB>特性,特性<TAB>为什么」，报一句的「NOTE<TAB>一句」；读不了补丁、成员清单或目标清单退 1、打一句原因。
#        弄坏开关 whole-package-tests：改到的包与工作区里依赖它们的包，每个目标都挑（文档测试除外）；fallback-runs-whole-workspace：判不出的不报、改成每个包的每个目标都挑（文档测试除外）
#   split <树> <rerun-failed-tests.py> <日志> <输出目录> <项>…   日志逐段归到项（section_item）：每段一行
#        「SECTION<TAB>项（归不到写 -）<TAB>段头的源文件<TAB>finished in 秒数之和<TAB>FAILED 条数<TAB>结果行条数」，每条 FAILED 一行「FAILED<TAB>项<TAB>libtest 全名」，
#        段头之前的结果行打「LOOSE<TAB>条数」；归到项的段按包接进 <输出目录>/<包>.log
#   outcomes <rerun-failed-tests.py> <test-plan 的输出> <这个包那几段的日志，没有写 -> <包> <红的名字落进的文件> <项>…   这个包跑到的项里每条测试函数一行
#        「<名字><TAB><项><TAB><现判|复用|拒算><TAB><绿|红|没跑到>」（拒算的写拒算；日志里有结果行的写现判；别的照计划写复用或现判）；计划里要现判而 FAILED 的名字写进第五个参数
MERGE_AND_VERIFY_ITEMS_PROGRAM="$(cat <<'PY_ITEMS'
import importlib.util, os, re, shlex, sys
TEST_ATTRIBUTE = re.compile(r"#\[\s*(?:[A-Za-z_][A-Za-z0-9_]*::)*test\s*[\](]")
FINISHED = re.compile(r"finished in ([0-9]+(?:\.[0-9]+)?)s")
def load(path, name="rerun_failed_tests_for_merge_and_verify"):
    specification = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module
def has_test_attribute(directory):
    for base, directories, files in os.walk(directory):
        directories[:] = [name for name in directories if name not in ("target", ".git") and not name.startswith("target-")]
        for name in files:
            if not name.endswith(".rs"):
                continue
            try:
                with open(os.path.join(base, name), encoding="utf-8", errors="replace") as handle:
                    if TEST_ATTRIBUTE.search(handle.read()):
                        return True
            except OSError:
                return True
    return False
def sections_of(rerun, path):
    sections, loose, current = [], 0, None
    with open(path, encoding="utf-8", errors="replace") as handle:
        for raw in handle:
            line = raw.rstrip("\n")
            header, doctest = rerun.SECTION_HEADER.match(line), rerun.DOCTEST_HEADER.match(line)
            running = re.match(r"^\s*Running\s+(.+)", line)
            if header or doctest or running:
                source = header.group(1) if header else (f"doc-tests {doctest.group(1)}" if doctest else running.group(1))
                current = {"source": source, "artifact": header.group(2).strip() if header else "", "lines": [line]}
                sections.append(current)
            elif current is None:
                loose += rerun.libtest_result_lines.result_line(line) is not None
            else:
                current["lines"].append(line)
    return sections, loose
WORKSPACE_WIDE = ("Cargo.toml", "Cargo.lock")
SELECTOR_VALUE_OPTIONS = ("--test", "--bin", "--example", "--bench")
SELECTOR_FLAGS = ("--lib", "--doc")
WHOLE_CLASS_OPTIONS = ("--tests", "--bins", "--examples", "--benches", "--all-targets", "--workspace", "--all", "--exclude")
CARGO_VALUE_OPTIONS = ("--features", "-F", "--target", "--target-dir", "--profile", "-j", "--jobs", "--manifest-path", "--color", "--message-format", "--config", "-Z")
def workspace_wide(path):
    return path in WORKSPACE_WIDE or path == ".cargo" or path.startswith(".cargo/")
def target_root_form(relative):
    """测试目标根文件那一类的路径：tests/<目标>.rs、tests/<目标>/main.rs、src/bin/<名>.rs、src/bin/<名>/main.rs。"""
    parts = relative.split("/")
    return (len(parts) == 2 and parts[0] == "tests" and parts[1].endswith(".rs")) or (len(parts) == 3 and parts[0] == "tests" and parts[2] == "main.rs") \
        or (len(parts) == 3 and parts[:2] == ["src", "bin"] and parts[2].endswith(".rs")) or (len(parts) == 4 and parts[:2] == ["src", "bin"] and parts[3] == "main.rs")
def module_pattern(name):
    return re.compile(rf"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+{re.escape(name)}\s*;|#\[\s*path\s*=\s*\"{re.escape(name)}/", re.MULTILINE)
def mutation_target(words):
    """变异行第五段 → (包, [挑法…], `--` 之后的筛选词, 点了一类目标时那个选项)。"""
    cut = words.index("--") if "--" in words else len(words)
    head, filters = words[:cut], words[cut + 1:]
    package, selectors, whole_class, position = None, [], None, 0
    while position < len(head):
        word = head[position]
        option, _, value = word.partition("=")
        if word in ("-p", "--package"):
            package = head[position + 1] if position + 1 < len(head) else None
            position += 2
            continue
        if option == "--package" and value:
            package = value
        elif word.startswith("-p") and len(word) > 2:
            package = word[2:]
        elif word in SELECTOR_FLAGS:
            selectors.append([word])
        elif word in SELECTOR_VALUE_OPTIONS:
            selectors.append([word, head[position + 1] if position + 1 < len(head) else ""])
            position += 2
            continue
        elif option in SELECTOR_VALUE_OPTIONS and value:
            selectors.append([option, value])
        elif option in WHOLE_CLASS_OPTIONS:
            whole_class = word
        elif word in CARGO_VALUE_OPTIONS:
            position += 2
            continue
        position += 1
    return package, selectors, filters, whole_class
def mutation_rows(directory):
    for file_name in ("mutations-append.tsv", "mutations-replacements.tsv"):
        path = os.path.join(directory, file_name)
        if not os.path.isfile(path):
            continue
        with open(path, encoding="utf-8") as handle:
            for raw in handle:
                line = raw.rstrip("\n")
                if line.strip() and not line.startswith("#"):
                    yield file_name, line.split("\t")
def select_targets(tree, rerun, rebuilt, broken, patch_directories):
    packages = rerun.workspace_packages(tree)
    order = list(packages)
    targets_of = {name: rerun.test_targets_of_package(directory, manifest, name) for name, (directory, manifest) in packages.items()}
    selected, refusals, required, notes, touched = {}, [], {}, [], []
    def select(package, root, why):
        entry = selected.setdefault((package, root), [])
        if why not in entry:
            entry.append(why)
    def required_entry(package, relative=None, name=None):
        """写了 required-features 的 [[test]] / [[bin]]：按路径（relative）或按名字（name）找 → (目标名, 特性) 或 None。"""
        manifest = packages[package][1]
        for section, folder in (("test", "tests"), ("bin", "src/bin")):
            for entry in manifest.get(section, []):
                entry_name, features = entry.get("name"), entry.get("required-features")
                if not isinstance(entry_name, str) or not features:
                    continue
                path = entry.get("path", "src/main.rs" if section == "bin" and entry_name == package else f"{folder}/{entry_name}.rs")
                folder_root = os.path.dirname(path) if path.endswith("/main.rs") else None
                if (name is not None and entry_name == name) or (relative is not None and (relative == path or (folder_root and relative.startswith(folder_root + "/")))):
                    return entry_name, features
        return None
    def roots_of_path(package, relative):
        directory, manifest = packages[package]
        targets = targets_of[package]
        if relative in targets and relative != rerun.DOC_TARGET:
            return [relative]
        parts = relative.split("/")
        if parts[0] == "tests" and len(parts) >= 3:
            main = f"tests/{parts[1]}/main.rs"
            if main in targets:
                return [main]
            pattern, hits = module_pattern(parts[1]), []
            for root in targets:
                if root.startswith("tests/") and root.count("/") == 1:
                    try:
                        with open(os.path.join(directory, root), encoding="utf-8", errors="replace") as handle:
                            if pattern.search(handle.read()):
                                hits.append(root)
                    except OSError:
                        continue
            return hits
        if parts[:2] == ["src", "bin"]:
            main = f"src/bin/{parts[2]}/main.rs" if len(parts) >= 4 else None
            return [main] if main in targets else []
        if parts[0] == "src" and len(parts) >= 2:
            library = (manifest.get("lib") or {}).get("path", "src/lib.rs")
            for root in (library, "src/main.rs"):
                if root in targets:
                    return [root]
        return []
    if not patch_directories:
        refusals.append("没给补丁目录：算不出这一批改到哪几个测试目标")
    for path in rebuilt.collect_paths(patch_directories, []):
        if workspace_wide(path):
            refusals.append(f"补丁改到工作区根的 {path}（改的是整个工作区的编法，判不出改到哪个测试目标）")
            continue
        package = rebuilt.package_of(tree, path)
        if package not in packages:
            refusals.append(f"补丁改到不落在任何包里的 {path}（测试运行时可能读它，判不出归哪个测试目标）")
            continue
        touched.append(package)
        directory = packages[package][0]
        relative = os.path.relpath(os.path.join(tree, path), directory)
        feature = required_entry(package, relative=relative)
        if feature:
            required.setdefault((package, feature[0]), [",".join(feature[1]), []])[1].append(f"补丁改到 {path}")
            continue
        roots = roots_of_path(package, relative)
        for root in roots:
            select(package, root, f"补丁改到 {path}")
        if roots:
            continue
        if target_root_form(relative) and not os.path.lexists(os.path.join(directory, relative)):
            notes.append(f"补丁删掉了 {path}（{package} 的这个测试目标已经没有，不跑）")
            continue
        refusals.append(f"补丁改到 {package} 包里的 {relative}，判不出改到哪个测试目标（不是测试目标的根文件、不在 src/ 下，tests/ 下也没有测试目标 mod 它）")
    for directory in patch_directories:
        for file_name, columns in mutation_rows(directory):
            name, label = columns[0], f"变异行「{columns[0]}」（{os.path.basename(directory)}/{file_name}）"
            if len(columns) < 6:
                refusals.append(f"{label}不是六段，读不出它点名的测试目标")
                continue
            try:
                words = shlex.split(columns[4])
            except ValueError as error:
                refusals.append(f"{label}的 cargo test 参数切不开（{error}）")
                continue
            package, selectors, filters, whole_class = mutation_target(words)
            if whole_class:
                refusals.append(f"{label}的 cargo test 参数点了一类目标 {whole_class}，判不出是哪一个测试目标")
                continue
            if package not in packages:
                refusals.append(f"{label}的 cargo test 参数{'没点包' if package is None else f'点的包 {package} 不在工作区里'}")
                continue
            if not selectors:
                refusals.append(f"{label}的 cargo test 参数只点了包 {package}、没点测试目标（等于跑整个包）")
                continue
            touched.append(package)
            for selector in selectors:
                if selector == ["--doc"]:
                    root = rerun.DOC_TARGET if rerun.DOC_TARGET in targets_of[package] else None
                else:
                    root = next((key for key, found in targets_of[package].items() if key != rerun.DOC_TARGET and list(found.selector) == selector), None)
                feature = required_entry(package, name=selector[1]) if root is None and len(selector) == 2 else None
                if root:
                    select(package, root, f"变异行「{name}」点名")
                    if filters:
                        notes.append(f"变异行「{name}」在 {rerun.item_name(package, root)} 里的筛选词 {' '.join(filters)}：跑的是整个目标，键相同的测试函数照复用")
                elif feature:
                    required.setdefault((package, feature[0]), [",".join(feature[1]), []])[1].append(f"变异行「{name}」点名")
                else:
                    refusals.append(f"{label}点名的 {' '.join(selector)} 在包 {package} 里找不到")
    if broken == "whole-package-tests":
        depends_on = {}
        for name, (directory, manifest) in packages.items():
            tables = [manifest.get(key) for key in ("dependencies", "dev-dependencies", "build-dependencies")]
            for target in (manifest.get("target") or {}).values():
                if isinstance(target, dict):
                    tables += [target.get(key) for key in ("dependencies", "dev-dependencies", "build-dependencies")]
            depends_on[name] = {value.get("package", key) if isinstance(value, dict) else key for table in tables for key, value in (table or {}).items()}
        widened = set(touched)
        while True:
            grown = {name for name, needed in depends_on.items() if name not in widened and needed & widened}
            if not grown:
                break
            widened |= grown
        for package in widened:
            for root in targets_of[package]:
                if root != rerun.DOC_TARGET:
                    select(package, root, "弄坏开关 whole-package-tests：改到的包与依赖它们的包整包挑")
    if refusals and broken == "fallback-runs-whole-workspace":
        notes.append(f"弄坏开关 fallback-runs-whole-workspace：判不出的 {len(refusals)} 处不报，每个包的每个目标都挑")
        refusals = []
        for package in order:
            for root in targets_of[package]:
                if root != rerun.DOC_TARGET:
                    select(package, root, "弄坏开关 fallback-runs-whole-workspace")
    for reason in refusals:
        print(f"REFUSE\t{reason}")
    if refusals:
        return
    position = {package: index for index, package in enumerate(order)}
    ranks = {(package, root): rank for package in order for rank, root in enumerate(targets_of[package])}
    for package, root in sorted(selected, key=lambda key: (position[key[0]], ranks[key])):
        found = targets_of[package][root]
        print(f"ITEM\t{rerun.item_name(package, root)}\t{package}\t{' '.join(found.selector)}\t{'；'.join(selected[(package, root)])}")
    for package in sorted({package for package, _root in selected}, key=position.get):
        if has_test_attribute(packages[package][0]):
            print(f"TESTED\t{package}")
    for (package, name), (features, reasons) in required.items():
        print(f"REQUIRES\t{package}\t{name}\t{features}\t{'；'.join(dict.fromkeys(reasons))}")
    for note in dict.fromkeys(notes):
        print(f"NOTE\t{note}")
mode = sys.argv[1]
if mode == "targets":
    tree, rerun, rebuilt, broken = sys.argv[2], load(sys.argv[3]), load(sys.argv[4], "lib_rebuilt_packages_for_merge_and_verify"), sys.argv[5]
    try:
        select_targets(tree, rerun, rebuilt, broken, sys.argv[6:])
    except (OSError, rerun.PackageNotFound) as error:
        print(f"读不了补丁、成员清单或目标清单：{error}")
        sys.exit(1)
elif mode == "split":
    tree, rerun, log, output = sys.argv[2], load(sys.argv[3]), sys.argv[4], sys.argv[5]
    packages = rerun.workspace_packages(tree)
    targets = {name: rerun.test_targets_of_package(directory, manifest, name) for name, (directory, manifest) in packages.items()}
    sections, loose = sections_of(rerun, log)
    os.makedirs(output, exist_ok=True)
    for section in sections:
        item = rerun.section_item(section["source"], section["artifact"], targets)
        seconds = sum(float(found.group(1)) for line in section["lines"] for found in [FINISHED.search(line)]
                      if found and rerun.libtest_result_lines.summary_line(line) is not None)
        failed = [name for name in (rerun.failed_name_of(line) for line in section["lines"]) if name is not None]
        results = sum(rerun.libtest_result_lines.result_line(line) is not None for line in section["lines"])
        shown = item if item else "-"
        print(f"SECTION\t{shown}\t{section['source']}\t{seconds}\t{len(failed)}\t{results}")
        for name in failed:
            print(f"FAILED\t{shown}\t{name}")
        if item:
            with open(os.path.join(output, rerun.split_item(item)[0] + ".log"), "a", encoding="utf-8") as handle:
                handle.write("\n".join(section["lines"]) + "\n")
    if loose:
        print(f"LOOSE\t{loose}")
elif mode == "outcomes":
    rerun, plan_output, package_log, package, red_file = load(sys.argv[2]), sys.argv[3], sys.argv[4], sys.argv[5], sys.argv[6]
    items, results, root = set(sys.argv[7:]), {}, None
    if package_log != "-" and os.path.isfile(package_log):
        with open(package_log, encoding="utf-8", errors="replace") as handle:
            for raw in handle:
                line = raw.rstrip("\n")
                header = rerun.SECTION_HEADER.match(line)
                if header or rerun.DOCTEST_HEADER.match(line):
                    root = header.group(1) if header else None
                    continue
                parsed = rerun.libtest_result_lines.result_line(line)
                if root and parsed is not None and parsed.outcome in ("ok", "FAILED"):
                    red = parsed.outcome == "FAILED" or results.get((root, parsed.full_name)) == "红"
                    results[(root, parsed.full_name)] = "红" if red else "绿"
    reds = []
    with open(plan_output, encoding="utf-8", errors="surrogateescape") as handle:
        for raw in handle:
            columns = raw.rstrip("\n").split("\t")
            if len(columns) < 7 or columns[1] not in ("judge", "reuse", "refused") or rerun.item_name(package, columns[4]) not in items:
                continue
            outcome = results.get((columns[4], columns[5]))
            state = "拒算" if columns[1] == "refused" else "现判" if outcome or columns[1] == "judge" else "复用"
            print(f"{columns[0]}\t{rerun.item_name(package, columns[4])}\t{state}\t{outcome or '没跑到'}")
            if outcome == "红" and columns[1] == "judge":
                reds.append(columns[0])
    with open(red_file, "w", encoding="utf-8") as handle:
        handle.write("".join(f"{name}\n" for name in reds))
PY_ITEMS
)"

# word_in_list <词> <词…>：第一个词在后面那几个里退 0
word_in_list() {
  local wanted="$1" candidate
  shift
  for candidate in "$@"; do [[ "$candidate" == "$wanted" ]] && return 0; done
  return 1
}

# item_mode：这一趟点了项（文件头「逐项点名与按条复用」）退 0。弄坏开关 items-ignored 下点了也当没点
item_mode() { ((${#NAMED_ITEMS[@]})) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != items-ignored ]]; }

# collect_test_items：挑出的项（ALL_ITEM_LINES，select_test_targets 填）里，点了 --item 的只留点名的进 RUN_ITEMS，写错的进 UNKNOWN_ITEMS；没点的全留。
#   弄坏开关 item-unknown-accepted 下写错的悄悄丢掉（丢完一项不剩就当没点）；items-ignored 下点了也全留
collect_test_items() {
  local item line
  local -a known=() kept_names=()
  RUN_ITEMS=(); UNKNOWN_ITEMS=()
  for line in ${ALL_ITEM_LINES[@]+"${ALL_ITEM_LINES[@]}"}; do known+=("${line%%$'\t'*}"); done
  for item in ${NAMED_ITEMS[@]+"${NAMED_ITEMS[@]}"}; do
    word_in_list "$item" ${known[@]+"${known[@]}"} || UNKNOWN_ITEMS+=("$item")
  done
  if ((${#UNKNOWN_ITEMS[@]})) && [[ "${MERGE_AND_VERIFY_BREAK:-}" == item-unknown-accepted ]]; then
    for item in "${NAMED_ITEMS[@]}"; do word_in_list "$item" "${UNKNOWN_ITEMS[@]}" || kept_names+=("$item"); done
    NAMED_ITEMS=(${kept_names[@]+"${kept_names[@]}"}); UNKNOWN_ITEMS=()
  fi
  if ! item_mode; then RUN_ITEMS=(${ALL_ITEM_LINES[@]+"${ALL_ITEM_LINES[@]}"}); return 0; fi
  for line in ${ALL_ITEM_LINES[@]+"${ALL_ITEM_LINES[@]}"}; do
    word_in_list "${line%%$'\t'*}" "${NAMED_ITEMS[@]}" && RUN_ITEMS+=("$line")
  done
  return 0
}

# run_item_roots <包>：RUN_ITEMS 里这个包的项的目标根文件（文档测试那一项不算），一行一个
run_item_roots() {
  local line item rest
  for line in ${RUN_ITEMS[@]+"${RUN_ITEMS[@]}"}; do
    item="${line%%$'\t'*}"; rest="${line#*$'\t'}"
    [[ "${rest%%$'\t'*}" == "$1" && "${item#"$1":}" != doc ]] && printf '%s\n' "${item#"$1":}"
  done
  return 0
}

# read_test_functions_request <阶段文件名> <去掉 --profile：1 或 0> [<交给 env 的赋值>…]：按条复用的请求（admission.py test-functions-request；同门禁 crash-point-replay 按条复用的取法）
#   → REQUEST_KIND 与 REQUEST_ARGUMENTS；交不出就打 ✗、两样都空。赋值照写进起它的环境（改判定域的变量要与交给 cargo 的同一份）
read_test_functions_request() {
  local stage_name="$1" strip_profile="$2" request_output tag value skip_next=0
  shift 2
  REQUEST_KIND=""; REQUEST_ARGUMENTS=()
  if request_output="$(env "$@" python3 "$ADMISSION" test-functions-request "$stage_name" "$root" 2>&1)"; then
    while IFS=$'\t' read -r tag value; do
      case "$tag" in
        kind) REQUEST_KIND="$value" ;;
        argument)
          if ((skip_next)); then skip_next=0
          elif ((strip_profile)) && [[ "$value" == --profile ]]; then skip_next=1
          else REQUEST_ARGUMENTS+=("$value"); fi ;;
      esac
    done <<<"$request_output"
  fi
  if [[ -z "$REQUEST_KIND" ]] || ((${#REQUEST_ARGUMENTS[@]} == 0)); then
    echo "  ✗ 按条复用：admission.py test-functions-request $stage_name 交不出请求（$(tr '\n' ' ' <<<"$request_output" | cut -c1-300)），用它的包一条都不跳、全部现判"
    echo "  → 怎么办：单独跑 python3 $ADMISSION test-functions-request $stage_name $root，看它为什么没打出 kind 与 argument 两种行（读不了阶段文件、种类登记缺了这类）；修好它再重跑合入后验证，按条复用才接得上"
    REQUEST_KIND=""; REQUEST_ARGUMENTS=()
  fi
  return 0
}

# package_kind <包>：这个包的按条复用种类（文件头「逐项点名与按条复用」：验证包是 VERIFICATION_FUNCTIONS_KIND，别的是 TEST_FUNCTIONS_KIND）
package_kind() { if [[ "$1" == "$VERIFICATION_PACKAGE" ]]; then printf '%s' "$VERIFICATION_FUNCTIONS_KIND"; else printf '%s' "$TEST_FUNCTIONS_KIND"; fi; }

# plan_test_functions <树>：跑到的项所在、有测试属性的包各起一次 admission.py test-plan（文件头「逐项点名与按条复用」的按条复用），计划放 PLAN_DIRECTORY 下
#   （test-plan.<包>.out 与 .plan）；算成的进 PLANNED_PACKAGES，失灵的进 FAILED_PLAN_PACKAGES 并置 PLAN_RED，判 prune-server 不可用的置 PLAN_UNAVAILABLE；
#   跑到的项里计划的条数进 PLAN_JUDGE、PLAN_REUSE、PLAN_REFUSED。验证包用 crash-point-replay.sh 的请求（--profile release 照留）另加 --exclude-registered，
#   缩过域时请求在带那个变量的环境里起。弄坏开关 plan-failure-ignored 下失灵不置 PLAN_RED；plan-unavailable-run-anyway 下不可用当失灵、照全部现判跑
plan_test_functions() {
  local tree_root="$1" line package plan_exit verdict_exit out failure_text judge reuse refused kind
  local -a packages=() plan_arguments=() scale_assignments=()
  PLANNED_PACKAGES=(); FAILED_PLAN_PACKAGES=(); PLAN_UNAVAILABLE=0; PLAN_RED=0; PLAN_JUDGE=0; PLAN_REUSE=0; PLAN_REFUSED=0
  TEST_FUNCTIONS_KIND=""; TEST_PLAN_ARGUMENTS=(); VERIFICATION_FUNCTIONS_KIND=""; VERIFICATION_PLAN_ARGUMENTS=()
  for line in ${RUN_ITEMS[@]+"${RUN_ITEMS[@]}"}; do
    line="${line#*$'\t'}"; package="${line%%$'\t'*}"
    if word_in_list "$package" ${TESTED_PACKAGES[@]+"${TESTED_PACKAGES[@]}"} && ! word_in_list "$package" ${packages[@]+"${packages[@]}"}; then packages+=("$package"); fi
  done
  ((${#packages[@]})) || return 0
  for package in "${packages[@]}"; do
    if [[ "$package" == "$VERIFICATION_PACKAGE" ]]; then
      [[ -z "$CRASH_EXPAND_UP_TO" ]] || scale_assignments=("$CRASH_EXPAND_UP_TO_VARIABLE=$CRASH_EXPAND_UP_TO")
      read_test_functions_request crash-point-replay.sh 0 ${scale_assignments[@]+"${scale_assignments[@]}"}
      VERIFICATION_FUNCTIONS_KIND="$REQUEST_KIND"; VERIFICATION_PLAN_ARGUMENTS=(${REQUEST_ARGUMENTS[@]+"${REQUEST_ARGUMENTS[@]}"})
      ((${#VERIFICATION_PLAN_ARGUMENTS[@]} == 0)) || VERIFICATION_PLAN_ARGUMENTS+=(--exclude-registered)
    elif [[ -z "$TEST_FUNCTIONS_KIND" ]]; then
      read_test_functions_request merge-and-verify.sh 1
      TEST_FUNCTIONS_KIND="$REQUEST_KIND"; TEST_PLAN_ARGUMENTS=(${REQUEST_ARGUMENTS[@]+"${REQUEST_ARGUMENTS[@]}"})
    fi
  done
  for package in "${packages[@]}"; do
    kind="$(package_kind "$package")"
    if [[ "$package" == "$VERIFICATION_PACKAGE" ]]; then plan_arguments=(${VERIFICATION_PLAN_ARGUMENTS[@]+"${VERIFICATION_PLAN_ARGUMENTS[@]}"})
    else plan_arguments=(${TEST_PLAN_ARGUMENTS[@]+"${TEST_PLAN_ARGUMENTS[@]}"}); fi
    if [[ -z "$kind" ]] || ((${#plan_arguments[@]} == 0)); then
      FAILED_PLAN_PACKAGES+=("$package")
      [[ "${MERGE_AND_VERIFY_BREAK:-}" == plan-failure-ignored ]] || PLAN_RED=1
      continue
    fi
    out="$PLAN_DIRECTORY/test-plan.$package.out"
    plan_exit=0
    python3 "$ADMISSION" test-plan "$tree_root" "$kind" "$package" "$PLAN_DIRECTORY/test-plan.$package.plan" "${plan_arguments[@]}" >"$out" 2>"$out.err" || plan_exit=$?
    if ((plan_exit == 0)) && grep -q '^PLAN ' "$out"; then
      PLANNED_PACKAGES+=("$package")
      read -r judge reuse refused < <(awk -F'\t' -v roots="$(run_item_roots "$package" | tr '\n' ' ')" 'BEGIN { n = split(roots, list, " "); for (i = 1; i <= n; i++) keep[list[i]] = 1 }
        NF >= 7 && ($5 in keep) { if ($2 == "judge") judge++; else if ($2 == "reuse") reuse++; else if ($2 == "refused") refused++ } END { print judge + 0, reuse + 0, refused + 0 }' "$out")
      PLAN_JUDGE=$((PLAN_JUDGE + judge)); PLAN_REUSE=$((PLAN_REUSE + reuse)); PLAN_REFUSED=$((PLAN_REFUSED + refused))
      continue
    fi
    # 失灵与不可用的判法只在 admission.py plan-failure（它文件头「按条复用失灵」）；它打的红句照转
    verdict_exit=0
    failure_text="$(python3 "$ADMISSION" plan-failure "merge-and-verify.sh 测试那一步的包 $package" test-plan "$plan_exit" "$out" "$out.err" 2>&1)" || verdict_exit=$?
    [[ -z "$failure_text" ]] || printf '%s\n' "$failure_text" | sed 's/^ */  /'
    if ((verdict_exit == 75)) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != plan-unavailable-run-anyway ]]; then
      PLAN_UNAVAILABLE=1
    else
      FAILED_PLAN_PACKAGES+=("$package")
      if ((verdict_exit == 1)) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != plan-failure-ignored ]]; then PLAN_RED=1; fi
    fi
  done
  echo "  · 按条复用（种类 ${TEST_FUNCTIONS_KIND:-}${VERIFICATION_FUNCTIONS_KIND:+${TEST_FUNCTIONS_KIND:+、}$VERIFICATION_FUNCTIONS_KIND（验证包）}）：${#packages[@]} 个包起了 test-plan，算成 ${#PLANNED_PACKAGES[@]} 个${FAILED_PLAN_PACKAGES[*]:+、失灵 ${#FAILED_PLAN_PACKAGES[@]} 个（${FAILED_PLAN_PACKAGES[*]}，一条都不跳）}$( ((PLAN_UNAVAILABLE)) && echo '、prune-server 不可用')；跑到的项里现判 $PLAN_JUDGE、复用 $PLAN_REUSE、拒算 $PLAN_REFUSED"
  return 0
}

# skip_words_of <包>：键相同、库里不红的测试函数（与验证包登记的用例）的 --exact --skip <名>…（admission.py plan-skip-names）进 SKIP_WORDS；
#   只算这一包跑到的目标，这一包的计划没算成就一条都不跳。弄坏开关 reuse-skips-dropped 下一条都不跳
skip_words_of() {
  local package="$1" names="" name
  local -a roots=() root_options=()
  SKIP_WORDS=()
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != reuse-skips-dropped ]] || return 0
  word_in_list "$package" ${PLANNED_PACKAGES[@]+"${PLANNED_PACKAGES[@]}"} || return 0
  mapfile -t roots < <(run_item_roots "$package")
  for name in ${roots[@]+"${roots[@]}"}; do root_options+=(--target-root "$name"); done
  ((${#root_options[@]})) || return 0
  names="$(python3 "$ADMISSION" plan-skip-names "$PLAN_DIRECTORY/test-plan.$package.out" "${root_options[@]}" 2>&1)" || names=""
  while IFS= read -r name; do [[ -n "$name" ]] && SKIP_WORDS+=(--skip "$name"); done <<<"$names"
  ((${#SKIP_WORDS[@]})) && SKIP_WORDS=(--exact "${SKIP_WORDS[@]}")
  return 0
}

# run_item_test_commands <日志目录> <测试的标志…>：测试那一步（文件头 test 一段与「逐项点名与按条复用」）：每个包一条只带这个包点到的目标，点了文档测试的另起一条 --doc；
#   验证包那一条带 --release（缩过域时带那个变量起），计划算不出时不起（登记的用例排不掉）。依次跑进 test-<序号>.log，跑完接成 test.log 与 test.problems.log。
#   交头一条退非 0 的退出码。弄坏开关 verification-release-dropped：验证包那一条不带 --release
run_item_test_commands() {
  local logs="$1" line package selector index=0 rc=0 step_rc
  shift
  local -a flags=("$@") packages=() doc_packages=() selectors=() selector_words=() command=() tail_words=() logs_written=() package_flags=() package_environment=()
  for line in "${RUN_ITEMS[@]}"; do
    line="${line#*$'\t'}"; package="${line%%$'\t'*}"
    word_in_list "$package" ${packages[@]+"${packages[@]}"} || packages+=("$package")
  done
  for package in ${packages[@]+"${packages[@]}"}; do
    selectors=()
    for line in "${RUN_ITEMS[@]}"; do
      line="${line#*$'\t'}"
      [[ "${line%%$'\t'*}" == "$package" ]] || continue
      selector="${line#*$'\t'}"
      if [[ "$selector" == --doc ]]; then word_in_list "$package" ${doc_packages[@]+"${doc_packages[@]}"} || doc_packages+=("$package"); continue; fi
      read -r -a selector_words <<<"$selector"; selectors+=("${selector_words[@]}")
    done
    ((${#selectors[@]})) || continue
    package_flags=(${flags[@]+"${flags[@]}"}); package_environment=()
    if [[ "$package" == "$VERIFICATION_PACKAGE" ]]; then
      if word_in_list "$package" ${FAILED_PLAN_PACKAGES[@]+"${FAILED_PLAN_PACKAGES[@]}"}; then
        echo "  ✗ 测试那一步：验证包 $package 的计划算不出（上面那一句），登记的崩溃枚举用例与放量分项排不掉，这一包的 ${selectors[*]} 不起"
        echo "  → 怎么办：照上面 admission.py plan-failure 那一句修好按条复用，再重跑同一条命令"
        continue
      fi
      [[ "${MERGE_AND_VERIFY_BREAK:-}" == "verification-release-dropped" ]] || package_flags=(--release ${flags[@]+"${flags[@]}"})
      [[ -z "$CRASH_EXPAND_UP_TO" ]] || package_environment=(env "$CRASH_EXPAND_UP_TO_VARIABLE=$CRASH_EXPAND_UP_TO")
    fi
    skip_words_of "$package"
    tail_words=()
    [[ "${MERGE_AND_VERIFY_BREAK:-}" != "test-arguments-added" ]] || tail_words+=(--show-output)
    tail_words+=(${SKIP_WORDS[@]+"${SKIP_WORDS[@]}"})
    if [[ "${MERGE_AND_VERIFY_TEST_TARGETS:-}" == serial || "${MERGE_AND_VERIFY_BREAK:-}" == "tests-one-binary-at-a-time" ]]; then
      command=(${package_environment[@]+"${package_environment[@]}"} cargo test --offline --no-fail-fast ${package_flags[@]+"${package_flags[@]}"} -p "$package" "${selectors[@]}")
    else
      command=(${package_environment[@]+"${package_environment[@]}"} python3 "$TEST_TARGETS_PARALLEL" --jobs "$THREADS" -- --offline --no-fail-fast ${package_flags[@]+"${package_flags[@]}"} -p "$package" "${selectors[@]}")
    fi
    ((${#tail_words[@]})) && command+=(-- "${tail_words[@]}")
    index=$((index + 1))
    echo "  · 测试第 $index 条：-p $package ${package_flags[*]} ${selectors[*]}（跳过键相同的 $(( ${#SKIP_WORDS[@]} / 2 )) 条）"
    run_step "$logs/test-$index.log" "${command[@]}"; step_rc=$?
    logs_written+=("$logs/test-$index")
    ((rc != 0)) || rc="$step_rc"
  done
  if ((${#doc_packages[@]})); then
    command=(cargo test --offline --no-fail-fast ${flags[@]+"${flags[@]}"})
    for package in "${doc_packages[@]}"; do command+=(-p "$package"); done
    command+=(--doc)
    index=$((index + 1))
    echo "  · 测试第 $index 条：文档测试 ${doc_packages[*]}（不复用）"
    run_step "$logs/test-$index.log" "${command[@]}"; step_rc=$?
    logs_written+=("$logs/test-$index")
    ((rc != 0)) || rc="$step_rc"
  fi
  : > "$logs/test.log"; : > "$logs/test.problems.log"
  for line in ${logs_written[@]+"${logs_written[@]}"}; do
    cat -- "$line.log" >> "$logs/test.log" 2>/dev/null
    [[ ! -f "$line.problems.log" ]] || { printf '== %s.log（行号是那一份里的）\n' "$line"; cat -- "$line.problems.log"; } >> "$logs/test.problems.log"
  done
  return "$rc"
}

# record_test_functions <日志目录> <树>：测试那一步跑完照键记库（文件头「逐项点名与按条复用」的按条复用）：test.log 逐段归到项与包（<日志目录>/test-sections.tsv、test-split/<包>.log），
#   算成计划的每个包：逐条结局并进 test-functions.tsv，lib_libtest_result_lines.py recordable 认结局、plan-green-names 取判绿可记的、计划里要现判而 FAILED 的交 --red-names，
#   同一次 verdict-record 记进库（记红撤绿）。verdict-record 报哨兵红的经 plan-failure 置 PLAN_RED，判不可用的置 PLAN_UNAVAILABLE。弄坏开关 red-not-recorded 下不交 --red-names
record_test_functions() {
  local logs="$1" tree_root="$2" line package item unattributed loose record_exit verdict_exit failure_text package_log kind how
  local split_directory="$logs/test-split"
  local -a items=()
  : > "$logs/test-functions.tsv"; : > "$logs/test-sections.tsv"
  [[ -s "$logs/test.log" ]] || return 0
  rm -rf -- "$split_directory"
  if ! python3 - split "$tree_root" "$RERUN_FAILED_TESTS" "$logs/test.log" "$split_directory" <<<"$MERGE_AND_VERIFY_ITEMS_PROGRAM" > "$logs/test-sections.tsv" 2>&1; then
    echo "  ! 按条复用：test.log 逐段归不到项（$(tail -n 2 "$logs/test-sections.tsv" | tr '\n' ' ')），这一趟一条都不记库，下一趟照现判"
    : > "$logs/test-sections.tsv"
    return 0
  fi
  unattributed="$(awk -F'\t' '$1 == "SECTION" && $2 == "-"' "$logs/test-sections.tsv" | grep -c .)"
  loose="$(awk -F'\t' '$1 == "LOOSE" { print $2 }' "$logs/test-sections.tsv")"
  if ((unattributed > 0 || ${loose:-0} > 0)); then
    echo "  ! 按条复用：test.log 里 $unattributed 段认不出、拿不准属于哪一项，${loose:-0} 条结果行在段头之前（rerun-failed-tests.py 的 section_item），这几条不记库、下一趟照现判"
  fi
  for package in ${PLANNED_PACKAGES[@]+"${PLANNED_PACKAGES[@]}"}; do
    items=()
    for line in ${RUN_ITEMS[@]+"${RUN_ITEMS[@]}"}; do
      item="${line%%$'\t'*}"; line="${line#*$'\t'}"
      [[ "${line%%$'\t'*}" == "$package" ]] && items+=("$item")
    done
    package_log="$split_directory/$package.log"; [[ -f "$package_log" ]] || package_log=-
    python3 - outcomes "$RERUN_FAILED_TESTS" "$PLAN_DIRECTORY/test-plan.$package.out" "$package_log" "$package" "$PLAN_DIRECTORY/test-plan.$package.red" "${items[@]}" \
      <<<"$MERGE_AND_VERIFY_ITEMS_PROGRAM" >> "$logs/test-functions.tsv" 2>&1 || : > "$PLAN_DIRECTORY/test-plan.$package.red"
    [[ "${MERGE_AND_VERIFY_BREAK:-}" != red-not-recorded ]] || : > "$PLAN_DIRECTORY/test-plan.$package.red"
    : > "$PLAN_DIRECTORY/test-plan.$package.green"
    if [[ "$package_log" != - ]] && python3 "$LIBTEST_RESULT_LINES" recordable "$package_log" > "$PLAN_DIRECTORY/test-plan.$package.accounting" 2>&1; then
      python3 "$ADMISSION" plan-green-names "$PLAN_DIRECTORY/test-plan.$package.out" "$PLAN_DIRECTORY/test-plan.$package.accounting" "$PLAN_DIRECTORY/test-plan.$package.withheld" \
        > "$PLAN_DIRECTORY/test-plan.$package.green" 2>&1 || : > "$PLAN_DIRECTORY/test-plan.$package.green"
    fi
    record_exit=0; kind="$(package_kind "$package")"; how="合入后验证测试那一步（cargo test，dev 档）"
    [[ "$package" != "$VERIFICATION_PACKAGE" ]] || how="合入后验证测试那一步（cargo test --release，验证包点到的目标，登记的崩溃枚举用例与放量分项经 --skip 排掉）"
    python3 "$ADMISSION" verdict-record "$tree_root" "$kind" "$PLAN_DIRECTORY/test-plan.$package.plan" "$PLAN_DIRECTORY/test-plan.$package.green" \
      --how "$how" --red-names "$PLAN_DIRECTORY/test-plan.$package.red" > "$PLAN_DIRECTORY/test-plan.$package.record" 2>&1 || record_exit=$?
    if ((record_exit == 0)); then
      echo "  · 按条复用记库（$package）：判绿 $(grep -c . "$PLAN_DIRECTORY/test-plan.$package.green") 条、判红 $(grep -c . "$PLAN_DIRECTORY/test-plan.$package.red") 条（红的照同一把键记红、撤掉原有的绿，下一趟现判）；$(grep '^RECORD ' "$PLAN_DIRECTORY/test-plan.$package.record" | tail -n 1)"
      continue
    fi
    verdict_exit=0
    # 记红没成（admission.py 文件头「按条复用失灵」）：带 --red 交，它判失灵、撤掉判红那几把键原有的绿；判红的名单空着的照只记绿那一支办
    failure_text="$(python3 "$ADMISSION" plan-failure "merge-and-verify.sh 测试那一步的包 $package" verdict-record "$record_exit" \
      --red "$tree_root" "$kind" "$PLAN_DIRECTORY/test-plan.$package.plan" "$PLAN_DIRECTORY/test-plan.$package.red" "$PLAN_DIRECTORY/test-plan.$package.record" 2>&1)" \
      || verdict_exit=$?
    [[ -z "$failure_text" ]] || printf '%s\n' "$failure_text" | sed 's/^ */  /'
    case "$verdict_exit" in
      0) echo "  ! 按条复用（$package）：判定没记进库（verdict-record 退 $record_exit：$(tail -n 2 "$PLAN_DIRECTORY/test-plan.$package.record" | tr '\n' ' ')），下一趟照判" ;;
      75) PLAN_UNAVAILABLE=1 ;;
      *) [[ "${MERGE_AND_VERIFY_BREAK:-}" == plan-failure-ignored ]] || PLAN_RED=1 ;;
    esac
  done
  return 0
}

# report_items_done <cargo test 的退出码>：测试那一步收尾每项一行 ITEM_DONE（文件头「逐项点名与按条复用」的机读行），读 test-sections.tsv 与各包的计划。
#   弄坏开关 item-done-unreported 下一行都不打
report_items_done() {
  local test_rc="$1" line item package seconds sections failed results judged reused exit_code any_failed=0 sections_file="$PLAN_DIRECTORY/test-sections.tsv"
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != item-done-unreported ]] || return 0
  [[ -f "$sections_file" ]] || : > "$sections_file"
  awk -F'\t' '$1 == "FAILED" && $2 != "-" { found = 1 } END { exit found ? 0 : 1 }' "$sections_file" && any_failed=1
  for line in ${RUN_ITEMS[@]+"${RUN_ITEMS[@]}"}; do
    item="${line%%$'\t'*}"; line="${line#*$'\t'}"; package="${line%%$'\t'*}"
    read -r sections seconds failed results < <(awk -F'\t' -v item="$item" '$1 == "SECTION" && $2 == item { count++; seconds += $4; failed += $5; results += $6 }
      END { printf "%d %d %d %d\n", count, seconds + 0.5, failed, results }' "$sections_file")
    if [[ "$item" == "$package:doc" ]]; then
      judged=1; reused=0
    elif word_in_list "$package" ${PLANNED_PACKAGES[@]+"${PLANNED_PACKAGES[@]}"}; then
      read -r judged reused < <(awk -F'\t' -v root="${item#"$package":}" 'NF >= 7 && $5 == root { if ($2 == "judge" || $2 == "refused") judge++; else if ($2 == "reuse") reuse++ }
        END { print judge + 0, reuse + 0 }' "$PLAN_DIRECTORY/test-plan.$package.out")
    else
      judged="$results"; reused=0
    fi
    exit_code=0
    if ((failed > 0)) || { ((test_rc != 0)) && ((any_failed == 0)); }; then exit_code=1
    elif ((sections == 0)); then exit_code=77; fi
    printf 'ITEM_DONE item=%s seconds=%s judged=%s reused=%s exit=%s\n' "$item" "$seconds" "$judged" "$reused" "$exit_code"
  done
  return 0
}

# list_items_and_exit：--list-items（文件头「逐项点名与按条复用」）：照这一趟的挑法挑测试目标、列项，带 --with-keys 另每项一行 ITEM_PLAN；不编、不跑、不拷快照。
#   判不出测试目标的逐条打、退 3。弄坏开关 item-plan-reusable-zero 下 ITEM_PLAN 的 reusable 一律 0
list_items_and_exit() {
  local tree_root="$SOURCE_TREE" line item package judged reused
  VERIFICATION_STAGE="${MERGE_AND_VERIFY_VERIFICATION_STAGE:-$tree_root/$VERIFICATION_STAGE_PATH}"
  verification_member_present "$tree_root" || die "认不出验证包：$MEMBERS_PROBLEM" "同真跑那一句：工作区成员要逐个写出目录、各有 [package] name，验证包 $VERIFICATION_PACKAGE 要在成员里" 1
  select_test_targets "$tree_root" || die "挑不出测试目标：$TARGET_PROBLEM" "看补丁目录在不在、crates.patch、变异行与工作区成员的 Cargo.toml 读不读得了" 1
  if ((${#TARGET_REFUSALS[@]})); then
    for line in "${TARGET_REFUSALS[@]}"; do echo "  ✗ 判不出测试目标：$line"; done   # gate-lint:detail
    die "这一趟有 ${#TARGET_REFUSALS[@]} 处判不出改到哪个测试目标（上面逐条），列不出项" "整包归提交时的整轮门禁；提交之外要整包跑先弹窗问用户" 3
  fi
  collect_test_items
  for line in ${ALL_ITEM_LINES[@]+"${ALL_ITEM_LINES[@]}"}; do
    item="${line%%$'\t'*}"; line="${line#*$'\t'}"
    printf '%s\t%s\n' "$item" "${line#*$'\t'}"
  done
  if ((${#UNKNOWN_ITEMS[@]})); then
    die "--item 点名的项不在这一趟可点的项里：${UNKNOWN_ITEMS[*]}（可点的项是上面列的第一列，一共 ${#ALL_ITEM_LINES[@]} 项）" "项照上面列的第一列写（<包>:<目标根文件> 或 <包>:doc）" 2
  fi
  ((LIST_WITH_KEYS)) || exit 0
  PLAN_DIRECTORY="$(mktemp -d "${TMPDIR:-/tmp}/merge-and-verify-list-items.XXXXXX")" || die "在 ${TMPDIR:-/tmp} 下建不了放计划的临时目录" "看 ${TMPDIR:-/tmp} 在不在、可不可写" 1
  trap 'rm -rf -- "${PLAN_DIRECTORY:?}"' EXIT
  plan_test_functions "$tree_root" >&2
  for line in ${RUN_ITEMS[@]+"${RUN_ITEMS[@]}"}; do
    item="${line%%$'\t'*}"; line="${line#*$'\t'}"; package="${line%%$'\t'*}"
    if [[ "$item" == "$package:doc" ]]; then judged=1; reused=0
    elif word_in_list "$package" ${PLANNED_PACKAGES[@]+"${PLANNED_PACKAGES[@]}"}; then
      read -r judged reused < <(awk -F'\t' -v root="${item#"$package":}" 'NF >= 7 && $5 == root { if ($2 == "judge" || $2 == "refused") judge++; else if ($2 == "reuse") reuse++ }
        END { print judge + 0, reuse + 0 }' "$PLAN_DIRECTORY/test-plan.$package.out")
    elif word_in_list "$package" ${TESTED_PACKAGES[@]+"${TESTED_PACKAGES[@]}"}; then
      printf '%s\t按项的计划算不出（这个包的 test-plan 失灵或 prune-server 不可用，上面那一句），不打 ITEM_PLAN\n' "$item"
      continue
    else judged=0; reused=0; fi
    [[ "${MERGE_AND_VERIFY_BREAK:-}" != item-plan-reusable-zero ]] || reused=0
    printf 'ITEM_PLAN item=%s to_judge=%s reusable=%s\n' "$item" "$judged" "$reused"
  done
  exit 0
}

# cargo_command_for <子命令> <参数词…>：拼一条 cargo 命令进 CARGO_BUILT_COMMAND：cargo <子命令> --offline 接 check.sh 那一行读来的参数词（读来的 --offline 不重复带）
cargo_command_for() {
  local subcommand="$1" word; shift
  CARGO_BUILT_COMMAND=(cargo "$subcommand" --offline)
  for word in "$@"; do [[ "$word" == --offline ]] || CARGO_BUILT_COMMAND+=("$word"); done
}

# clippy_command_for <特性组合>：照 check.sh 那一行（CARGO_CLIPPY_WORDS）拼这个组合的 clippy 命令进 clippy_command、步名进 clippy_step（都是 verify() 的局部变量；文件头 clippy 一段）。
# 两个占位词换成值：特性参数——组合 default 不带，别的按空白切成词带上；lint 表——verify() 的 clippy_lints
clippy_command_for() {
  local word
  local -a feature_arguments=() clippy_words=()
  [[ "$1" == default ]] || read -r -a feature_arguments <<< "$1"
  clippy_step="$(clippy_step_name "$1")"
  for word in ${CARGO_CLIPPY_WORDS[@]+"${CARGO_CLIPPY_WORDS[@]}"}; do
    case "$word" in
      "$CLIPPY_FEATURES_PLACEHOLDER") clippy_words+=(${feature_arguments[@]+"${feature_arguments[@]}"}) ;;
      "$CLIPPY_LINTS_PLACEHOLDER") clippy_words+=(${clippy_lints[@]+"${clippy_lints[@]}"}) ;;
      *) clippy_words+=("$word") ;;
    esac
  done
  cargo_command_for clippy ${clippy_words[@]+"${clippy_words[@]}"}
  clippy_command=("${CARGO_BUILT_COMMAND[@]}")
}

verify() {
  local logs="$1"; shift
  local -a dirs=("$@")
  local dir rc applied=0 warned=0 failed_tests reds=0 test_count combination clippy_step="" word
  MUTATION_TABLES_STATE=""
  local -a test_flags=() clippy_lints=() patch_arguments=() compile_logs=() fmt_command=() clippy_command=() build_command=() red_items=()
  local -a clippy_red_combinations=() clippy_green_combinations=()
  mkdir -p "$logs"
  rm -f -- "$logs/test.log" "$logs/build.log"   # 重编核对只认这一趟的编译输出（日志目录可能是调用方给的旧目录）
  for dir in ${CHECK_PATCH_DIRS[@]+"${CHECK_PATCH_DIRS[@]}"}; do patch_arguments+=(--patch-dir "$dir"); done
  if ! read_check_sh "$CHECK_SH"; then
    if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "lint-table-optional" ]]; then
      echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=lint-table-optional：$CHECK_SH_PROBLEM，照常往下验"
    else
      die "check.sh 读不出、或与这里对不上：$CHECK_SH_PROBLEM；一个补丁都没打、没编" \
        "上面一句点名了是 check.sh 的哪一处（lint 表、缺省特性组合、clippy 循环的结构行，或 fmt、clippy、build、test 哪一条 cargo 命令的哪一行哪个词），照它在 read_check_sh、read_cargo_command 里补上这一种写法的识别（cargo 命令只换包装函数名、加删标志不用改这里：现读）；先判得上游这一改是想让提交时跑的事变了，还是只换了写法，再补，让合入后验证与提交时的判据仍是同一套" 1
    fi
  fi
  # 弄坏开关 cargo-arguments-fixed：四条 cargo 命令的参数用这里写死的，不用从 check.sh 读来的
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "cargo-arguments-fixed" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=cargo-arguments-fixed：四条 cargo 命令的参数用写死的，不用 check.sh 里读来的"
    CARGO_FMT_WORDS=(--all -- --check); CARGO_BUILD_WORDS=(--all-targets); CARGO_TEST_FLAGS=()
    CARGO_CLIPPY_WORDS=(--all-targets "$CLIPPY_FEATURES_PLACEHOLDER" -- -D warnings "$CLIPPY_LINTS_PLACEHOLDER")
  fi
  # check.sh 认的特性组合登记（文件头 clippy 一段）：这里不读，要验的树里有就判红、不编。弄坏开关 clippy-registration-ignored：照读缺省组合往下验
  if [[ -n "$CLIPPY_REGISTRATION" && -e "$tree/$CLIPPY_REGISTRATION" ]]; then
    if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "clippy-registration-ignored" ]]; then
      echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=clippy-registration-ignored：要验的树里有特性组合登记 $CLIPPY_REGISTRATION，照读缺省组合往下验"
    else
      die "要验的树里有 check.sh 认的特性组合登记 $tree/$CLIPPY_REGISTRATION：check.sh 只跑登记的组合，这里不读登记，跑的组合会与它不同；一个补丁都没打、没编" \
        "先把登记的读法（check.sh 里 CLIPPY_COMBINATIONS_FILE 那一段）接进这份脚本的 read_check_sh，让合入后验证与提交时跑同一套组合；或删掉那份登记、回到 check.sh 的缺省组合" 1
    fi
  fi
  # 弄坏开关 single-clippy-pass：clippy 只跑 --all-features 一趟
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "single-clippy-pass" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=single-clippy-pass：clippy 只跑 --all-features 一趟，不用 check.sh 的特性组合（${CLIPPY_COMBINATIONS[*]-}）"
    CLIPPY_COMBINATIONS=("--all-features")
  fi
  clippy_lints=(${DISCIPLINE_LINTS[@]+"${DISCIPLINE_LINTS[@]}"})
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "drop-discipline-lint" && ${#clippy_lints[@]} -ge 2 ]]; then
    clippy_lints=("${clippy_lints[@]:0:${#clippy_lints[@]}-2}")
  fi
  for dir in "${dirs[@]}"; do
    [[ -d "$dir" ]] || die "补丁目录不在：$dir" "给实现员交回的补丁目录（里面有 crates.patch 或 mutations-append.tsv）"
    python3 "$APPLY" "$dir" --root "$tree" > "$logs/apply-$(basename "$dir").log" 2>&1; rc=$?
    case "$rc" in
      0) applied=$((applied+1)); echo "  打上：$dir" ;;
      1) applied=$((applied+1)); warned=$((warned+1))
         echo "  ✗ 打上了，但打补丁脚本退 1：$dir（它不在本进程里判 mutation-tables，不该退 1；日志 $logs/apply-$(basename "$dir").log）"
         echo "  → 怎么办：看那份日志里它退 1 的原因（环境里设着它的弄坏开关就去掉），修好之后整趟重跑" ;;
      *) die "补丁核不过、一个字没改：$dir（apply-writer-patch.py 退 $rc，日志 $logs/apply-$(basename "$dir").log）" "按那份日志给的下一步让实现员改补丁再交" 2 ;;
    esac
  done
  if ((applied)) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != "mutation-tables-claimed-judged" ]]; then
    MUTATION_TABLES_STATE="code-source-discipline 的 mutation-tables 格本次未跑（打补丁脚本不在本进程里起它）"
    echo "  · $MUTATION_TABLES_STATE；补丁打进主工作区之后另交："
    echo "    SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-source-discipline.sh 的 mutation-tables 格; 范围=1 格; 为什么=合入后验证没在本进程里跑这一格，补丁打进主工作区之后补判' task-server/task submit --cwd $(printf '%q' "$SOURCE_TREE") -- bash .claude/gate.d/code-source-discipline.sh --check mutation-tables $(printf '%q' "$SOURCE_TREE")"
  fi
  verification_member_present "$tree" || die "认不出验证包：$MEMBERS_PROBLEM；没编" \
    "工作区成员要逐个写出目录、各有 [package] name，验证包 $VERIFICATION_PACKAGE 要在成员里（这里靠它的名字给它的目标带 --release、排掉登记的崩溃枚举用例）；改了名就同步改这份脚本的 VERIFICATION_PACKAGE" 1
  select_test_targets "$tree" || die "挑不出测试目标：$TARGET_PROBLEM；没编" \
    "看补丁目录在不在、crates.patch、变异行与工作区成员的 Cargo.toml 读不读得了" 1
  report_target_selection
  # 逐项点名（文件头「逐项点名与按条复用」）：认 --item（在打了补丁的快照上认，写错退 2、一条 cargo 都不起）；判不出测试目标的这一趟不跑测试，不认
  collect_test_items
  if ((${#UNKNOWN_ITEMS[@]})) && ((${#TARGET_REFUSALS[@]} == 0)); then
    echo "  这一趟可点的项（${#ALL_ITEM_LINES[@]} 项）："
    for word in ${ALL_ITEM_LINES[@]+"${ALL_ITEM_LINES[@]}"}; do printf '       %s\n' "${word%%$'\t'*}"; done   # gate-lint:detail
    die "--item 点名的项不在这一趟可点的项里：${UNKNOWN_ITEMS[*]}；没编" \
      "项照上面列的写（<包>:<目标根文件> 或 <包>:doc；bash scripts/gate/merge-and-verify.sh --list-items <同一批补丁目录> 也列）" 2
  fi
  # 四步 cargo 只写这一份（clippy 每个特性组合一条），逐条直接跑
  # test 那一行读来的标志里 --offline、--no-fail-fast 这里本来就带，不重复
  for word in ${CARGO_TEST_FLAGS[@]+"${CARGO_TEST_FLAGS[@]}"}; do
    [[ "$word" == --offline || "$word" == --no-fail-fast ]] || test_flags+=("$word")
  done
  fmt_command=(cargo fmt ${CARGO_FMT_WORDS[@]+"${CARGO_FMT_WORDS[@]}"})   # cargo fmt 不认 --offline，不经 cargo_command_for
  # clippy 每个特性组合一条，clippy_command_for 现拼（文件头 clippy 一段）
  cargo_command_for build ${CARGO_BUILD_WORDS[@]+"${CARGO_BUILD_WORDS[@]}"}
  build_command=("${CARGO_BUILT_COMMAND[@]}")
  # RocksDB 预编库的取值（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」）：clippy --all-features 与 build --all-targets 编工作区全部成员，连同 验证包（编到 librocksdb-sys）。
  #   起四条 cargo 之前判一次这台装了、此刻进程环境的取值对得上（补丁打完之后判：补丁改了 Cargo.lock，键跟着变），判红退 1、一条 cargo 都不起。
  #   树里没有依赖 rocksdb 的清单（自证的样本工作区）、
  #   PATH 上是带标记的假 cargo，entry 报不判。弄坏开关 rocksdb-entry-unchecked 下不判（自证「起 cargo 之前 RocksDB 预编库的取值缺了」那一格判错）
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" != "rocksdb-entry-unchecked" ]] && ! python3 "$root/scripts/hosts/rocksdb-prebuilt.py" entry --root "$tree"; then
    die "起 cargo 之前 RocksDB 预编库的取值没判过（上面 rocksdb-prebuilt.py entry 的红句），一条 cargo 都没起" \
      "照上面的出路修（开新会话，或命令前 eval \"\$(python3 scripts/hosts/rocksdb-prebuilt.py env)\"）再跑同一条命令" 1
  fi
  # 按条复用（文件头「逐项点名与按条复用」）：跑到的项所在、有测试属性的包各算一份计划（admission.py test-plan 要 cargo -V 与 rustc -vV，放在预编库的取值判过之后）
  PLAN_DIRECTORY="$logs"
  ((${#TARGET_REFUSALS[@]} || ${#RUN_ITEMS[@]} == 0)) || plan_test_functions "$tree"
  # 测试二进制的参数：不带（标 #[ignore] 的只剩要 NVIDIA 卡、量显存与量速度、要补丁表副本那几条，带参数把它们起起来不对），只带按条复用跳过键相同的 --exact --skip <名>…
  #   （每个包各算，run_item_test_commands；弄坏开关 test-arguments-added：在 -- 之后多带一个参数）
  rc=0; test_count=0; failed_tests=""
  if ((${#TARGET_REFUSALS[@]} == 0)) && ((${#RUN_ITEMS[@]})) && ((PLAN_UNAVAILABLE == 0)); then
    echo "  编译并跑挑出的 ${#RUN_ITEMS[@]} 项（$(printf '%s\n' "${RUN_ITEMS[@]}" | cut -f1 | tr '\n' ' ' | sed 's/ $//')$(item_mode && echo "；--item 点名的")；一个包一条；并行度 $THREADS）…"
    run_item_test_commands "$logs" ${test_flags[@]+"${test_flags[@]}"}; rc=$?
    test_count="$(awk 'match($0, /^test result: [A-Za-z]+\. [0-9]+ passed; [0-9]+ failed/) { split($0, f, " "); passed += f[4]; failed += f[6] } END { print passed + failed + 0 }' "$logs/test.log")"
    failed_tests="$(sed -n 's/^test \(.*\) \.\.\. FAILED$/\1/p' "$logs/test.log")"
    # 按条复用照键记库、逐项打 ITEM_DONE（文件头「逐项点名与按条复用」）
    record_test_functions "$logs" "$tree"
    report_items_done "$rc"
  fi
  if ((${#TARGET_REFUSALS[@]})); then
    TESTS_REFUSED=1
    echo "  ✗ 测试那一步与验证包那一步没跑：${#TARGET_REFUSALS[@]} 处判不出改到哪个测试目标（下面逐条），合入后验证不跑整个包"
    for word in "${TARGET_REFUSALS[@]}"; do echo "      $word"; done   # gate-lint:detail
    echo "  → 怎么办：整包归提交时的整轮门禁；提交之外要整包跑先弹窗问用户。补丁本该只改到测试目标的，照上面那几处改（工作区级的改动拆出去、变异行点名 --lib 或 --test <目标>）再交；别的步照跑，整条退 3"
  elif ((${#RUN_ITEMS[@]} == 0)); then
    echo "  - 测试本次未跑：这一批补丁没点到要跑的测试目标（要特性的、删掉的见上面挑测试目标那几行）"
  elif ((PLAN_UNAVAILABLE)) && [[ ! -f "$logs/test.log" ]]; then
    echo "  - 测试本次未跑：按条复用那一层连不上 prune-server（上面 admission.py plan-failure 那一句），一条 cargo test 都没起（不记红、不算测过；整条退 75）"
  elif [[ -n "$failed_tests" && "${MERGE_AND_VERIFY_BREAK:-}" != "ignore-red" ]]; then
    reds=$((reds+1))
    echo "  ✗ 红的测试 $(printf '%s\n' "$failed_tests" | wc -l) 条："
    printf '%s\n' "$failed_tests" | sed 's/^/      /'   # gate-lint:detail
    # 红了只重跑红的（文件头「逐项点名与按条复用」）：红的测试函数归到了项的点名那几项，归不到的照旧 -p <包>
    mapfile -t red_items < <(awk -F'\t' '$1 == "FAILED" && $2 != "-" { print $2 }' "$logs/test-sections.tsv" 2>/dev/null | LC_ALL=C sort -u)
    if ((${#red_items[@]})); then
      echo "  → 怎么办：只重跑红的这几条：python3 scripts/gate/rerun-failed-tests.py $logs/test.log$(printf ' --item %s' "${red_items[@]}")（红的 ${#red_items[@]} 项）；照实现员报告里的算法改钉值，改完再只跑这几条，或只重跑这几项的合入后验证：$(merge_rerun_command "${red_items[@]}")（判绿记过库的照复用）（错误警告日志：$logs/test.problems.log，行前带全量日志里的行号）"
    else
      echo "  → 怎么办：只重跑红的这几条：python3 scripts/gate/rerun-failed-tests.py $logs/test.log -p <包>；照实现员报告里的算法改钉值，改完再只跑这几条（错误警告日志：$logs/test.problems.log，行前带全量日志里的行号）"
    fi
  elif [[ $rc -ne 0 && "${MERGE_AND_VERIFY_BREAK:-}" != "ignore-red" ]]; then
    reds=$((reds+1))
    echo "  ✗ cargo test 退 $rc 而一条 FAILED 都没抓到（编不过）：$(tail -3 "$logs/test.log" | tr '\n' ' ')"
    echo "  → 怎么办：看 $logs/test.problems.log（错误警告，行前带全量日志里的行号）与 $logs/test.log（全量）"
  elif [[ "$((test_count + PLAN_REUSE))" == 0 && "${MERGE_AND_VERIFY_BREAK:-}" != "zero-tests-green" ]]; then
    reds=$((reds+1))
    echo "  ✗ cargo test 退 0，却一条测试都没跑（test result 行里 passed 与 failed 加起来是 0，按条复用也一条都没复用；日志 $logs/test.log）"
    echo "  → 怎么办：看那份日志里编了哪些包、有没有 test result 行；一条都没跑的「全绿」与跑过了长得一样（check.sh 同一条）"
  else
    echo "  ✓ 测试全绿（$(grep -c '^test result: ok' "$logs/test.log") 个测试二进制，共 $test_count 条测试；跑了 ${#RUN_ITEMS[@]} 项（$(printf '%s\n' "${RUN_ITEMS[@]}" | cut -f2 | LC_ALL=C sort -u | grep -c .) 个包）$(item_mode && echo '，--item 点名的')；按条复用：现判 $PLAN_JUDGE、复用 $PLAN_REUSE、拒算 $PLAN_REFUSED）"
  fi
  run_step "$logs/fmt.log" "${fmt_command[@]}"; rc=$?
  if [[ $rc -ne 0 ]]; then reds=$((reds+1)); echo "  ✗ cargo fmt --check 红（日志 $logs/fmt.log、错误警告 $logs/fmt.problems.log）"; echo "  → 怎么办：cargo fmt --all，再看 diff 只动了格式"; else echo "  ✓ fmt 过"; fi
  # clippy 照 check.sh 的特性组合逐个跑（文件头 clippy 一段）：任一组合红整步红、点名是哪几个组合
  for combination in "${CLIPPY_COMBINATIONS[@]}"; do
    clippy_command_for "$combination"
    run_step "$logs/$clippy_step.log" "${clippy_command[@]}"; rc=$?
    if [[ $rc -ne 0 ]]; then
      echo "  ✗ clippy 特性组合 $combination 红（退 $rc；日志 $logs/$clippy_step.log、错误警告 $logs/$clippy_step.problems.log）"   # gate-lint:detail
      # 弄坏开关 clippy-combination-red-uncounted：照打上面那一行，不记进整步的红
      [[ "${MERGE_AND_VERIFY_BREAK:-}" == "clippy-combination-red-uncounted" ]] || clippy_red_combinations+=("$combination")
    else
      clippy_green_combinations+=("$combination")
    fi
  done
  if ((${#clippy_red_combinations[@]})); then
    reds=$((reds+1))
    echo "  ✗ clippy 红：${#CLIPPY_COMBINATIONS[@]} 个特性组合里 ${#clippy_red_combinations[@]} 个红（${clippy_red_combinations[*]}），日志逐个在上面"
    echo "  → 怎么办：按红的那几个组合的日志逐条改（只在不带特性那一趟红的，多是只在 #[cfg(feature)] 圈起来那一边用到的代码），不加 allow 绕过"
  elif ((${#clippy_green_combinations[@]} == ${#CLIPPY_COMBINATIONS[@]})); then
    echo "  ✓ clippy 过（${#CLIPPY_COMBINATIONS[@]} 个特性组合：${CLIPPY_COMBINATIONS[*]}；每个带 check.sh 的编码纪律 lint $((${#clippy_lints[@]} / 2)) 条）"
  fi
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "skip-build" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=skip-build：不跑 cargo build"
  else
    run_step "$logs/build.log" "${build_command[@]}"; rc=$?
    if [[ $rc -ne 0 ]]; then reds=$((reds+1)); echo "  ✗ cargo build --all-targets 红（日志 $logs/build.log、错误警告 $logs/build.problems.log）"; echo "  → 怎么办：从日志里第一条 rustc 报错改起，后面的多半是它的连锁反应"; else echo "  ✓ build 过"; fi
  fi
  # 改过的包这一趟都重编了没有（文件头）：test 与 build 两趟的编译输出合起来核
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "skip-rebuild-check" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=skip-rebuild-check：不核改过的包重编了没有"
  elif ((${#patch_arguments[@]} == 0)); then
    echo "  - 重编核对本次未判：没给补丁目录，算不出改过的包"
  elif [[ ! -f "$logs/test.log" && ! -f "$logs/build.log" ]]; then
    echo "  - 重编核对本次未判：这一趟测试与 build 都没编（测试本次未跑、build 没跑），没有编译输出可核"
  else
    compile_logs=()
    [[ ! -f "$logs/test.log" ]] || compile_logs+=(--log "$logs/test.log")   # 测试本次未跑时没有这一份
    [[ ! -f "$logs/build.log" ]] || compile_logs+=(--log "$logs/build.log")
    python3 "$REBUILT" check --tree "$tree" --target-dir "${CARGO_TARGET_DIR:-$tree/target}" "${compile_logs[@]}" "${patch_arguments[@]}" > "$logs/rebuilt.log" 2>&1; rc=$?
    sed 's/^/  /' "$logs/rebuilt.log"
    case "$rc" in
      0|77) ;;
      1) reds=$((reds+1)) ;;
      *) reds=$((reds+1)); echo "  ✗ 重编核对判不了（lib_rebuilt_packages.py 退 $rc，日志 $logs/rebuilt.log）"
         echo "  → 怎么办：照上面那一行修（编译日志在不在、补丁目录读不读得了）再整趟重跑；判不了不算绿" ;;
    esac
  fi
  # 验证包那一步：编译与测试之后（文件头「验证包那一步」）
  verification_step "$logs" || reds=$((reds+1))
  local applied_note="其中打补丁脚本退 1 的 $warned 个"
  [[ "${MERGE_AND_VERIFY_BREAK:-}" == "mutation-tables-claimed-judged" ]] && applied_note="其中 code-source-discipline 红 $warned 个"
  echo "  汇总：打上 ${applied} 个补丁目录（${applied_note}${MUTATION_TABLES_STATE:+；$MUTATION_TABLES_STATE}），验证红 $reds 项$( ((TESTS_REFUSED)) && echo '，测试那一步与验证包那一步没跑（判不出测试目标，整包归提交时的整轮门禁）')${VERIFICATION_STATE:+，验证包那一步 $VERIFICATION_STATE}${VERIFICATION_SCALED:+，缩过域、不算门禁通过（$CRASH_EXPAND_UP_TO_VARIABLE=$CRASH_EXPAND_UP_TO，只往小缩）}$( ((PLAN_UNAVAILABLE)) && echo '，测试那一步按条复用连不上 prune-server（不是红，也不算通过）')，日志在 $logs"
  if [[ $reds -ne 0 || $warned -ne 0 ]]; then return 1; fi
  ((TESTS_REFUSED == 0)) || return 3   # 文件头「退出码」3：判不出测试目标，测试那一步与验证包那一步没跑
  ((PLAN_UNAVAILABLE == 0)) || return 75   # 文件头「退出码」75：按条复用那一层连不上 prune-server，别的没有红
  return 0
}

# merge_rerun_command <项…>：只重跑这几项的合入后验证的整条命令（文件头「逐项点名与按条复用」）：范围声明种类 items、范围是项数，带 --skip-apply 与这一趟的补丁目录
merge_rerun_command() {
  local item text
  local -a words=("$root/task-server/task" submit -- bash "$root/scripts/gate/merge-and-verify.sh" --skip-apply)
  for item in "$@"; do words+=(--item "$item"); done
  words+=(${CHECK_PATCH_DIRS[@]+"${CHECK_PATCH_DIRS[@]}"})
  text="$(printf '%q ' "${words[@]}")"
  printf "SINGLEFS_RUN_SCOPE='种类=items; 跑什么=合入后验证点名的 %s; 范围=%s 项; 为什么=合入后验证这几项红了，修完只重跑它们：键相同的照复用，只现判红的那几条与改后键变了的' %s" "$*" "$#" "${text% }"
}

selftest() {
  local scratch fake_cargo_dir failures=0 checked=0 out rc expected_lints expected_lint_count check_case check_case_text combination clippy_expected clippy_received clippy_unreported
  local baseline_calls flagged_calls flagged_clippy_count
  local -a expected_combinations=()
  local service_problem
  # 逐项与复用那几格经 admission.py 写判定库（文件头 --selftest 那一行）：不是临时 prune-server 就一格都不跑，不写真服务
  if ! service_problem="$(python3 -c 'import sys; sys.path.insert(0, sys.argv[1]); import prune_server_client; print(prune_server_client.selftest_service_problem())' "$(dirname "${BASH_SOURCE[0]}")/../prune-server" 2>&1)" || [[ -n "$service_problem" ]]; then
    echo "  ✗ merge-and-verify.sh 自证没跑：${service_problem:-读不了 scripts/prune-server/prune_server_client.py}"
    echo "     → 怎么办：照 scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service 那一条，在临时服务底下起这一份自证（带 --server-binary 就不编）"
    return 2
  fi
  unset MERGE_AND_VERIFY_KEEP_COPY   # 留现场那一格自己设；调用方设了不能带进别的格
  unset SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO   # 缩崩溃域的格自己设；调用方环境里有的不能带进别的格
  # 测试那一步：别的格一条 cargo test（它们核假 cargo 收到的那一条），只有「按测试目标并行起」那一格不设它（文件头 test 一段）
  export MERGE_AND_VERIFY_TEST_TARGETS=serial
  scratch="$(mktemp -d)"; fake_cargo_dir="$scratch/bin"; mkdir -p "$fake_cargo_dir" "$scratch/patch-a" "$scratch/patch-b"
  # 各格没设 CARGO_TARGET_DIR 时编译目录建在 TMPDIR 下（文件头「这一趟的编译目录」）：TMPDIR 指进 scratch，留下的随 scratch 一起删
  export TMPDIR="$scratch/tmp"; mkdir -p "$TMPDIR"
  # 这一趟建的编译目录在哪：取它开跑时打的那一行（「这一趟的编译目录：<路径>（这里建的，收尾删）」）
  run_target_of() { sed -n 's/^  这一趟的编译目录：\(.*\)（这里建的，收尾删）$/\1/p' "$1" 2>/dev/null | head -1; }
  # patch-a 改的是 demo 里的一份数据文件（不是 .rs：重编核对不核它，测试照挑 demo）
  printf -- '--- a/crates/demo/src/table.txt\n+++ b/crates/demo/src/table.txt\n@@ -1 +1 @@\n-1\n+2\n' > "$scratch/patch-a/crates.patch"
  # 验证包那一步（文件头「验证包那一步」）：各格都换上假的那一道，它照真的那一道的写法答 --list-items、打按条复用与崩溃状态块那几行
  cat > "$scratch/fake-verification-step.sh" <<'FAKE'
#!/usr/bin/env bash
# 假的 门禁 crash-point-replay：--list-items 照真的那一道的写法列一条登记的崩溃枚举用例（测试目标 setting）与挂特性的两项（库、gpu_pair.rs），退 FAKE_VERIFICATION_LIST_EXIT（没设是 0）；
#   别的起法照真的那一道打一行按条复用、一行崩溃状态块与一行逐格，FAKE_VERIFICATION_EXIT 不是 0 时另打一条 FAILED 与点名的每项一行 exit=1 的 ITEM_DONE，退它（没设是 0）。
#   每次的参数记进 FAKE_VERIFICATION_CALLS，不是 --list-items 的那几次把展开深度变量记进 FAKE_VERIFICATION_ENVIRONMENT（都是设了的话）
[[ -z "${FAKE_VERIFICATION_CALLS:-}" ]] || printf '%s\n' "$*" >> "$FAKE_VERIFICATION_CALLS"
# 照真的那一道写它的产品 GPU 包那一格守哪个包：合入后验证照这一行现读（文件头「验证包那一步」）
PRODUCT_GPU_PACKAGE_NAME="singlefs-background-gpu"
if [[ "$1" == --list-items ]]; then
  printf 'crash-case:demo-case\tsinglefs-verification:setting:case_function\n'
  printf 'feature-gated:lib\t--lib\n'
  printf 'feature-gated:gpu_pair.rs\twhole-file gpu\n'
  exit "${FAKE_VERIFICATION_LIST_EXIT:-0}"
fi
[[ -z "${FAKE_VERIFICATION_ENVIRONMENT:-}" ]] || printf '%s\n' "${SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO:-unset}" >> "$FAKE_VERIFICATION_ENVIRONMENT"
echo "  · 按条复用：验证包 2 条编得进的测试函数，现判 0、复用 2、拒算 0；这一组特性下不编 0 条"
echo "  ✓ 崩溃状态块：现判 3、复用 5、拒算 0、本次未跑 0"
echo "  · 逐格：crash-case-replay：绿；feature-gated-tests：无对象可判"
if [[ "${FAKE_VERIFICATION_EXIT:-0}" != 0 ]]; then
  echo "test case_function ... FAILED"
  previous=""
  for word in "$@"; do [[ "$previous" == --item ]] && echo "ITEM_DONE item=$word seconds=1 judged=1 reused=0 exit=1"; previous="$word"; done
fi
exit "${FAKE_VERIFICATION_EXIT:-0}"
FAKE
  export MERGE_AND_VERIFY_VERIFICATION_STAGE="$scratch/fake-verification-step.sh"
  unset MERGE_AND_VERIFY_VERIFICATION_ROOT FAKE_VERIFICATION_CALLS FAKE_VERIFICATION_EXIT FAKE_VERIFICATION_ENVIRONMENT FAKE_VERIFICATION_LIST_EXIT
  # 只有变异行、没有 crates.patch 的补丁目录（--skip-apply 时不核打没打）：变异行点名 demo 的库目标，--skip-apply 的各格拿它挑测试目标（文件头「挑测试目标」）
  mkdir -p "$scratch/patch-demo-lib"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'demo 的库（自证样本）' crates/demo/src/lib.rs 'pub fn f() {}' 'pub fn f() { }' '-p demo --lib' f > "$scratch/patch-demo-lib/mutations-append.tsv"
  # patch-targets 改 demo 的库里一份数据文件与测试目标 alpha 的根文件（「按测试目标并行起」那一格用）
  mkdir -p "$scratch/patch-targets"
  printf -- '--- a/crates/demo/src/table.txt\n+++ b/crates/demo/src/table.txt\n@@ -1 +1 @@\n-1\n+2\n--- a/crates/demo/tests/alpha.rs\n+++ b/crates/demo/tests/alpha.rs\n@@ -1 +1 @@\n-1\n+2\n' \
    > "$scratch/patch-targets/crates.patch"
  # 自证自己经 task-server 派到第二台时环境里带着第二台那一趟的几样（scripts/hosts/multi-host-run.sh 文件头 ⑤「带到第二台的环境」：本台的多机配置、起活那一方的回读握手、
  #   这一趟的本地目录）：各格照本机跑，只有「跑在第二台上」那一格自己设那几样；多机配置不带进来（逐项与复用那几格起 admission.py，
  #   本台那一份的路径不漏进它们起的子进程）。两台对称，没有认台的角色变量
  unset SINGLEFS_MULTI_HOST_CONFIG GPU_QUIESCE_ORIGIN_READ_BACK MULTI_HOST_RUN_PEER_SCRATCH
  cat > "$scratch/fake-apply.py" <<'FAKE'
import sys
if "--root" in sys.argv:
    print("root=" + sys.argv[sys.argv.index("--root") + 1])
sys.exit(2 if sys.argv[1].endswith("patch-b") else 0)
FAKE
  cat > "$fake_cargo_dir/cargo" <<'FAKE'
#!/usr/bin/env bash
[[ -z "${FAKE_CARGO_CALLS:-}" ]] || printf '%s\n' "$*" >> "$FAKE_CARGO_CALLS"
case "$1" in
  metadata) cat "${FAKE_CARGO_METADATA:?假 cargo 没给 FAKE_CARGO_METADATA}"; exit 0 ;;
  test) if [[ " $* " == *" --no-run "* ]]; then cat "${FAKE_CARGO_ARTIFACTS:?假 cargo 没给 FAKE_CARGO_ARTIFACTS}"; exit 0; fi
        [[ "${FAKE_CARGO_MODE:-}" == "empty" ]] && exit 0
        printf 'running 2 tests\ntest tests::adds ... ok\ntest tests::subtracts_in_%s ... FAILED\n\ntest result: FAILED. 1 passed; 1 failed\n' "${FAKE_CARGO_MODE:-x}"; [[ "${FAKE_CARGO_MODE:-}" == "green" ]] && exit 0 || exit 101 ;;
  fmt) exit 0 ;;
  clippy) # FAKE_CARGO_CLIPPY_RED 设了：特性组合（--all-targets 与 -- 之间的词，没有就是 default）等于它的那一趟打一条告警、退 101
          all="$*"; combination="${all#clippy --offline --all-targets}"; combination="${combination%% -- *}"; combination="${combination# }"
          if [[ -n "${FAKE_CARGO_CLIPPY_RED:-}" && "${combination:-default}" == "$FAKE_CARGO_CLIPPY_RED" ]]; then
            echo "error: function \`only_used_under_a_feature\` is never used"; exit 101
          fi
          exit 0 ;;
esac
FAKE
  [[ "${FAKE_CARGO_MODE:-}" == "" ]] || true
  chmod +x "$fake_cargo_dir/cargo"
  # 快照从临时树拷（MERGE_AND_VERIFY_TREE），不碰本仓的源文件与编译目录
  mkdir -p "$scratch/tree/crates/demo/src"; printf 'pub fn f() {}\n' > "$scratch/tree/crates/demo/src/lib.rs"
  # 样本工作区两个成员：demo 与 验证包；补丁 patch-a 只改到 demo 的库，cargo test 只该起 demo --lib
  write_workspace() {   # $1 树，$2 验证包那个成员的包名
    mkdir -p "$1/crates/demo" "$1/crates/singlefs-verification"
    printf '[workspace]\nmembers = ["crates/demo", "crates/singlefs-verification"]\n' > "$1/Cargo.toml"
    printf '[package]\nname = "demo"\n' > "$1/crates/demo/Cargo.toml"
    printf '[package]\nname = "%s"\n' "$2" > "$1/crates/singlefs-verification/Cargo.toml"
  }
  write_workspace "$scratch/tree" singlefs-verification
  run_case() { PATH="$fake_cargo_dir:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-$1" \
    MERGE_AND_VERIFY_TREE="$scratch/tree" env -u CARGO_TARGET_DIR bash "$0" "${@:2}" > "$out" 2>&1; rc=$?; }
  out="$scratch/out1.txt"; run_case one "$scratch/patch-a"; checked=$((checked+1))
  if [[ $rc -ne 1 ]] || ! grep -q 'subtracts_in_x' "$out" || ! grep -q 'rerun-failed-tests.py' "$out"; then
    echo "  ✗ 自检：一条测试红时应当退 1、列出那条测试与只重跑它的命令，实际退出码 $rc：$(tail -4 "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 每一步的日志出两份（run_step 经 scripts/hosts/two-logs.sh）：test.log 是全量，test.problems.log 带行号收那条 FAILED，红的出路点名错误警告日志的路径
  checked=$((checked+1))
  if [[ ! -f "$scratch/logs-one/test.log" ]] || ! grep -qE '^[0-9]+:test tests::subtracts_in_x \.\.\. FAILED$' "$scratch/logs-one/test.problems.log" 2>/dev/null \
     || ! grep -qF "错误警告日志：$scratch/logs-one/test.problems.log" "$out"; then
    echo "  ✗ 自检：一条测试红时每一步的日志应当出两份（test.log 全量；test.problems.log 带行号收那条 FAILED），红的出路点名错误警告日志的路径，实际日志目录里有 $(ls "$scratch/logs-one" 2>/dev/null | tr '\n' ' ')：$(tail -3 "$scratch/logs-one/test.problems.log" 2>&1 | tr '\n' '|')"; failures=1; fi   # gate-lint:detail
  # 打补丁之后 mutation-tables 那一格（文件头「门禁 code-source-discipline 的 mutation-tables 格」）：报本次未跑、打出另交的整条命令，汇总不说判过
  checked=$((checked+1))
  if ! grep -qF '· code-source-discipline 的 mutation-tables 格本次未跑' "$scratch/out1.txt" \
     || ! grep -qF "task-server/task submit --cwd $scratch/tree -- bash .claude/gate.d/code-source-discipline.sh --check mutation-tables $scratch/tree" "$scratch/out1.txt" \
     || grep -qF 'code-source-discipline 红 0 个' "$scratch/out1.txt"; then
    echo "  ✗ 自检：打补丁之后应当报 code-source-discipline 的 mutation-tables 格本次未跑、打出补丁打进主工作区之后另交的整条命令，实际：$(grep -E 'mutation-tables|汇总' "$scratch/out1.txt" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  out="$scratch/out2.txt"; run_case two "$scratch/patch-a" "$scratch/patch-b"; checked=$((checked+1))
  if [[ $rc -ne 2 ]] || ! grep -q '补丁核不过' "$out"; then
    echo "  ✗ 自检：apply 退 2 时应当整批停、退 2，实际 $rc：$(tail -2 "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 快照（文件头「快照」）测试红（假 cargo 这时还报一条 FAILED）：退 1，自己拷的快照与建的编译目录照样删掉
  out="$scratch/out-copy-red.txt"; run_case copy-red "$scratch/patch-a"; checked=$((checked+1))
  local copy_red_target; copy_red_target="$(run_target_of "$out")"
  if [[ $rc -ne 1 ]] || [[ -e "$scratch/logs-copy-red/repo" || -z "$copy_red_target" || -e "$copy_red_target" ]] || ! grep -q '删了快照' "$out"; then
    echo "  ✗ 自检：快照：测试红时也应当删掉快照与编译目录，实际退出码 $rc，日志目录里还有 $(ls "$scratch/logs-copy-red" 2>/dev/null | tr '\n' ' ')：$(grep -E '✗|删了|拷好快照' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 日志目录指进了要验的树里面：快照会把自己拷进自己，退 2、什么都不拷（这一处拒绝不配弄坏开关：放开它 rsync 会往快照里一层层拷快照）
  out="$scratch/out-snapshot-inside.txt"; : > "$scratch/calls-snapshot-inside.txt"; checked=$((checked+1))
  PATH="$fake_cargo_dir:$PATH" FAKE_CARGO_CALLS="$scratch/calls-snapshot-inside.txt" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/tree/logs-inside" \
    MERGE_AND_VERIFY_TREE="$scratch/tree" env -u CARGO_TARGET_DIR bash "$0" --skip-apply > "$out" 2>&1; rc=$?
  if [[ $rc -ne 2 || -s "$scratch/calls-snapshot-inside.txt" || -e "$scratch/tree/logs-inside/repo" ]] || ! grep -qF '快照会把自己拷进自己' "$out"; then
    echo "  ✗ 自检：日志目录在要验的树里面时应当退 2、什么都不拷，实际退出码 $rc：$(grep -E '✗|拷好快照' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  rm -rf -- "${scratch:?}/tree/logs-inside"
  # 起 cargo 之前判 RocksDB 预编库的取值（文件头「RocksDB 预编库的取值」）：样本工作区的 验证包依赖 rocksdb、进程环境里四个取值是空的，
  #   PATH 上的假 cargo 不带 entry 认的标记 ⇒ 退 1、假 cargo 一次都没起、输出里有 rocksdb-prebuilt entry 的红句（别的格样本工作区的 验证包不依赖 rocksdb，entry 报不判）
  local rocksdb_tree="$scratch/tree-rocksdb"
  mkdir -p "$rocksdb_tree/crates/demo/src"; printf 'pub fn f() {}\n' > "$rocksdb_tree/crates/demo/src/lib.rs"
  write_workspace "$rocksdb_tree" singlefs-verification
  printf '[dependencies]\nrocksdb = { version = "=0.25.0", optional = true }\n' >> "$rocksdb_tree/crates/singlefs-verification/Cargo.toml"
  out="$scratch/out-rocksdb.txt"; : > "$scratch/calls-rocksdb.txt"; checked=$((checked+1))
  PATH="$fake_cargo_dir:$PATH" FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-rocksdb.txt" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-rocksdb" MERGE_AND_VERIFY_TREE="$rocksdb_tree" ROCKSDB_LIB_DIR= ROCKSDB_STATIC= SNAPPY_LIB_DIR= SNAPPY_STATIC= \
    env -u CARGO_TARGET_DIR bash "$0" --skip-apply > "$out" 2>&1; rc=$?
  if [[ $rc -ne 1 || -s "$scratch/calls-rocksdb.txt" ]] || ! grep -qF 'rocksdb-prebuilt entry：' "$out"; then
    echo "  ✗ 自检：起 cargo 之前 RocksDB 预编库的取值缺了（验证包依赖 rocksdb、进程环境里没有取值）应当退 1、一条 cargo 都不起，实际退出码 $rc、假 cargo 收到 $(wc -l < "$scratch/calls-rocksdb.txt") 条：$(tail -3 "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 假 cargo 全绿的一趟：把 FAILED 行换掉
  sed -i.bak 's/ \.\.\. FAILED/ ... ok/; s/FAILED\. 1 passed; 1 failed/ok. 2 passed; 0 failed/' "$fake_cargo_dir/cargo" && rm -f "$fake_cargo_dir/cargo.bak"
  out="$scratch/out3.txt"; FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-three.txt" run_case three "$scratch/patch-a"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -q '测试全绿' "$out"; then
    echo "  ✗ 自检：全绿时应当退 0 并说全绿，实际 $rc：$(tail -3 "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 快照全绿：补丁打进快照（apply 收到 --root <日志目录>/repo），收尾快照与编译目录都删掉、打一行删了什么
  out="$scratch/out-copy-green.txt"; FAKE_CARGO_MODE=green run_case copy-green "$scratch/patch-a"; checked=$((checked+1))
  local copy_green_target; copy_green_target="$(run_target_of "$out")"
  if [[ $rc -ne 0 ]] || [[ -e "$scratch/logs-copy-green/repo" || -z "$copy_green_target" || -e "$copy_green_target" ]] || ! grep -q '删了快照' "$out" \
     || ! grep -qxF "root=$scratch/logs-copy-green/repo" "$scratch/logs-copy-green/apply-patch-a.log" 2>/dev/null; then
    echo "  ✗ 自检：快照：全绿时应当在快照里打补丁、收尾删掉快照与编译目录，实际退出码 $rc，日志目录里还有 $(ls "$scratch/logs-copy-green" 2>/dev/null | tr '\n' ' ')，apply 收到 $(cat "$scratch/logs-copy-green/apply-patch-a.log" 2>/dev/null)：$(grep -E '✗|删了|拷好快照' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # MERGE_AND_VERIFY_KEEP_COPY=1：留着快照与编译目录、打出路径；源树里那份 lib.rs 的修改时刻先拨回 2020 年初，留下的快照里那一份要是拷的那一刻（不保留修改时刻）
  touch -d @1577836800 "$scratch/tree/crates/demo/src/lib.rs"
  out="$scratch/out-copy-kept.txt"; FAKE_CARGO_MODE=green MERGE_AND_VERIFY_KEEP_COPY=1 run_case copy-kept "$scratch/patch-a"; checked=$((checked+1))
  local copy_kept_target kept_mtime; copy_kept_target="$(run_target_of "$out")"
  kept_mtime="$(stat -c %Y "$scratch/logs-copy-kept/repo/crates/demo/src/lib.rs" 2>/dev/null)"
  if [[ $rc -ne 0 ]] || [[ ! -d "$scratch/logs-copy-kept/repo" || -z "$copy_kept_target" || ! -d "$copy_kept_target" ]] \
     || ! grep -qF "留着快照 $scratch/logs-copy-kept/repo 与编译目录 $copy_kept_target" "$out" || [[ ! "$kept_mtime" =~ ^[0-9]+$ ]] || ((kept_mtime <= 1577836800)); then
    echo "  ✗ 自检：快照：MERGE_AND_VERIFY_KEEP_COPY=1 时应当留着快照与编译目录、打出路径，快照里的修改时刻是拷的那一刻，实际退出码 $rc，快照里 lib.rs 的 mtime「${kept_mtime:-取不到}」（源树的是 1577836800）：$(grep -E '✗|留着|删了' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 快照根 git init（文件头「快照」）：读上面留着的那一份快照，它的 git common-dir 是 <快照>/.git
  local kept_copy_common=""; checked=$((checked+1))
  kept_copy_common="$(git -C "$scratch/logs-copy-kept/repo" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" || kept_copy_common=""
  if [[ "$kept_copy_common" != "$(cd "$scratch/logs-copy-kept" 2>/dev/null && pwd -P)/repo/.git" ]]; then
    echo "  ✗ 自检：快照：快照根应当 git init（留下的快照 $scratch/logs-copy-kept/repo 的 git common-dir 该是它自己的 .git），实际取到「${kept_copy_common:-取不到}」"; failures=1; fi   # gate-lint:detail
  # 与 check.sh 同一套判据：lint 表在这里另取一遍（sed 取那一段、grep 数条数），与脚本里 read_check_sh 不是同一段代码
  expected_lints="$(sed -n '/^CODE_DISCIPLINE_LINTS=(/,/^)/p' "$CHECK_SH" | grep -oE -- '-[DWAF] clippy::[a-z0-9_]+' | tr '\n' ' ')"
  expected_lints="${expected_lints% }"
  expected_lint_count="$(sed -n '/^CODE_DISCIPLINE_LINTS=(/,/^)/p' "$CHECK_SH" | grep -cE -- '-[DWAF] clippy::')"
  # 特性组合同样在这里另取一遍（sed 取行首那一行、grep 抠双引号里的词），步名照文件头的规则另算一遍
  while IFS= read -r combination; do expected_combinations+=("$combination"); done \
    < <(sed -n 's/^CLIPPY_FEATURE_COMBINATIONS=(\(.*\))[[:space:]]*$/\1/p' "$CHECK_SH" | grep -oE '"[^"]*"' | tr -d '"')
  checked=$((checked+1))
  clippy_received="$(grep '^clippy' "$scratch/calls-three.txt")"
  if [[ "$expected_lint_count" -lt 1 || -z "$clippy_received" ]] || grep -vqE -- " -- -D warnings $expected_lints\$" <<< "$clippy_received"; then
    echo "  ✗ 自检：clippy 应当带上 check.sh 的 lint 表里每一条（$CHECK_SH 里数到 $expected_lint_count 条，每个特性组合那一条都带），假 cargo 收到的是：$(tr '\n' '|' <<< "$clippy_received")"; failures=1; fi   # gate-lint:detail
  # clippy 照 check.sh 的特性组合逐个跑：每个组合一条、次序同 check.sh，default 不带特性参数（只比 -- 之前那一段，lint 表归上一格）
  checked=$((checked+1))
  clippy_expected=""
  for combination in ${expected_combinations[@]+"${expected_combinations[@]}"}; do
    clippy_expected+="clippy --offline --all-targets$([[ "$combination" == default ]] || printf ' %s' "$combination")|"
  done
  clippy_received="$(sed 's/ -- .*//' <<< "$clippy_received" | tr '\n' '|')"
  if ((${#expected_combinations[@]} < 2)) || [[ "$clippy_received" != "$clippy_expected" ]]; then
    echo "  ✗ 自检：clippy 应当照 check.sh 的特性组合逐个跑（$CHECK_SH 里读到 ${#expected_combinations[@]} 个：${expected_combinations[*]-}；要收到 $clippy_expected），假 cargo 收到的是：$clippy_received"; failures=1; fi   # gate-lint:detail
  # 一个特性组合红（假 cargo 在 default 那一趟打告警、退 101）：整步红、退 1、点名 default，别的组合照跑
  out="$scratch/out-clippy-red.txt"; checked=$((checked+1))
  FAKE_CARGO_MODE=green FAKE_CARGO_CLIPPY_RED=default FAKE_CARGO_CALLS="$scratch/calls-clippy-red.txt" run_case clippy-red --skip-apply
  if [[ $rc -ne 1 ]] || ! grep -qF '✗ clippy 特性组合 default 红' "$out" || ! grep -qF "✗ clippy 红：${#expected_combinations[@]} 个特性组合里 1 个红（default）" "$out" \
     || grep -qF '✓ clippy 过' "$out" || [[ "$(grep -c '^clippy' "$scratch/calls-clippy-red.txt" 2>/dev/null)" != "${#expected_combinations[@]}" ]] \
     || ! grep -qF 'only_used_under_a_feature' "$scratch/logs-clippy-red/clippy-default.log" 2>/dev/null; then
    echo "  ✗ 自检：clippy 一个特性组合红时应当整步红、退 1、点名红的那个组合（default，日志 clippy-default.log），别的组合照跑，实际退出码 $rc，假 cargo 收到 $(grep -c '^clippy' "$scratch/calls-clippy-red.txt" 2>/dev/null) 条 clippy：$(grep -E 'clippy' "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 要验的树里有 check.sh 认的特性组合登记（上游 lib.sh 的 CLIPPY_FEATURE_COMBINATIONS_FILE，此刻是 .claude/clippy-feature-combinations）：退 1、不编
  mkdir -p "$scratch/tree/.claude" && printf -- '--all-features  # 自证样本\n' > "$scratch/tree/.claude/clippy-feature-combinations"
  out="$scratch/out-clippy-registered.txt"; checked=$((checked+1))
  FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-clippy-registered.txt" run_case clippy-registered --skip-apply
  rm -rf -- "${scratch:?}/tree/.claude"
  if [[ $rc -ne 1 ]] || ! grep -qF '要验的树里有 check.sh 认的特性组合登记' "$out" || [[ -s "$scratch/calls-clippy-registered.txt" ]]; then
    echo "  ✗ 自检：要验的树里有特性组合登记时应当退 1、不编，实际退出码 $rc，假 cargo 收到 $(cat "$scratch/calls-clippy-registered.txt" 2>/dev/null | wc -l) 条：$(grep -E '✗|!' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  checked=$((checked+1))
  if ! grep -qxF -- "build --offline --all-targets" "$scratch/calls-three.txt"; then
    echo "  ✗ 自检：应当跑 cargo build --offline --all-targets（check.sh 的构建那一条），假 cargo 收到的是：$(tr '\n' '|' < "$scratch/calls-three.txt")"; failures=1; fi   # gate-lint:detail
  checked=$((checked+1))
  if [[ "$(grep '^test' "$scratch/calls-three.txt" | tr '\n' '|')" != "test --offline --no-fail-fast -p demo --lib|" ]]; then
    echo "  ✗ 自检：cargo test 应当只起补丁改到的 demo 的库目标（-p demo --lib），假 cargo 收到的是：$(grep '^test' "$scratch/calls-three.txt" | tr '\n' '|')"; failures=1; fi   # gate-lint:detail
  # 测试那一步按测试目标并行起（文件头 test 一段）：这一格不设 MERGE_AND_VERIFY_TEST_TARGETS，假 cargo 的 metadata 与 --no-run 打录好的两个测试目标（demo 的 lib 与 test alpha）；
  # 全绿那一趟两个目标各起一条、汇总数到 2 个测试二进制 4 条测试，红的那一趟两条 FAILED 都列出来、退 1
  printf '%s\n' '{"packages":[{"id":"demo-id","name":"demo","targets":[{"kind":["lib"],"name":"demo","doctest":false},{"kind":["test"],"name":"alpha","doctest":false}]}],"workspace_members":["demo-id"]}' \
    > "$scratch/fake-metadata.json"
  printf '%s\n' '{"reason":"compiler-artifact","package_id":"demo-id","target":{"kind":["lib"],"name":"demo"},"profile":{"test":true},"executable":"/selftest/demo"}' \
    '{"reason":"compiler-artifact","package_id":"demo-id","target":{"kind":["test"],"name":"alpha"},"profile":{"test":true},"executable":"/selftest/alpha"}' > "$scratch/fake-artifacts.jsonl"
  mkdir -p "$scratch/tree/crates/demo/tests"; printf 'fn main() {}\n' > "$scratch/tree/crates/demo/tests/alpha.rs"   # 这两格之后删掉：别的格的 demo 只有库目标
  out="$scratch/out-targets.txt"; MERGE_AND_VERIFY_TEST_TARGETS= FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-targets.txt" \
    FAKE_CARGO_METADATA="$scratch/fake-metadata.json" FAKE_CARGO_ARTIFACTS="$scratch/fake-artifacts.jsonl" run_case targets "$scratch/patch-targets"; checked=$((checked+1))
  local targets_green_rc=$rc targets_red_out="$scratch/out-targets-red.txt" targets_red_rc
  # 红的那一趟要假 cargo 照最早那一份打一条 FAILED（全绿那一格之前已把它换成全 ok），跑完换回全 ok
  sed -i.bak 's/subtracts_in_%s \.\.\. ok/subtracts_in_%s ... FAILED/; s/ok\. 2 passed; 0 failed/FAILED. 1 passed; 1 failed/' "$fake_cargo_dir/cargo" && rm -f "$fake_cargo_dir/cargo.bak"
  out="$targets_red_out"; MERGE_AND_VERIFY_TEST_TARGETS= FAKE_CARGO_CALLS="$scratch/calls-targets-red.txt" \
    FAKE_CARGO_METADATA="$scratch/fake-metadata.json" FAKE_CARGO_ARTIFACTS="$scratch/fake-artifacts.jsonl" run_case targets-red "$scratch/patch-targets"; targets_red_rc=$rc
  rm -rf -- "${scratch:?}/tree/crates/demo/tests"
  sed -i.bak 's/ \.\.\. FAILED/ ... ok/; s/FAILED\. 1 passed; 1 failed/ok. 2 passed; 0 failed/' "$fake_cargo_dir/cargo" && rm -f "$fake_cargo_dir/cargo.bak"
  out="$scratch/out-targets.txt"
  if [[ $targets_green_rc -ne 0 ]] || ! grep -qxF -- "test --offline --no-fail-fast --lib --test alpha -p demo --no-run --message-format=json-render-diagnostics" "$scratch/calls-targets.txt" \
     || ! grep -qxF -- "test --offline --no-fail-fast -p demo --lib" < "$scratch/calls-targets.txt" || ! grep -qxF -- "test --offline --no-fail-fast -p demo --test alpha" < "$scratch/calls-targets.txt" \
     || ! grep -q '✓ 测试全绿（2 个测试二进制，共 4 条测试' "$out" || ! grep -q '^TEST_TARGETS_PARALLEL dispatched=2 collected=2 ' "$scratch/logs-targets/test.log" \
     || [[ $targets_red_rc -ne 1 ]] || ! grep -q '红的测试 2 条' "$targets_red_out"; then
    echo "  ✗ 自检：测试那一步应当经 test-targets-parallel.py 按测试目标并行起（先 --no-run 编、两个目标各一条、汇总 2 个测试二进制 4 条测试；红的那一趟列出 2 条、退 1），假 cargo 收到的是：$(grep '^test' "$scratch/calls-targets.txt" 2>/dev/null | tr '\n' '|')；全绿那一趟：$(grep -E '✓ 测试|✗' "$out" | head -2 | tr '\n' ' ')；红的那一趟退 $targets_red_rc：$(grep -E '红的测试|✓ 测试' "$targets_red_out" | head -1)"; failures=1; fi   # gate-lint:detail
  # 测试那一步的 cargo test 在 -- 之后什么都不带（文件头 test 一段）：一条 cargo test 那一种（全绿那一格）与按测试目标并行起那一种（上一格）收到的命令里都没有 -- 之后的参数，验证包照旧不进，成功句不报标 #[ignore] 的条数
  checked=$((checked+1))
  if grep -qE '^test .* -- ' "$scratch/calls-three.txt" "$scratch/calls-targets.txt" \
     || grep -qE '^test .*-p singlefs-verification' "$scratch/calls-three.txt" "$scratch/calls-targets.txt" \
     || ! grep -qF '共 4 条测试；跑了 2 项（1 个包）' "$scratch/out-targets.txt"; then
    echo "  ✗ 自检：测试那一步的 cargo test 在 -- 之后应当什么都不带（一条 cargo test 与按测试目标并行起两种都一样）、验证包照旧不进、成功句不报标 #[ignore] 的条数，假 cargo 收到的是：$(grep -h '^test' "$scratch/calls-three.txt" "$scratch/calls-targets.txt" 2>/dev/null | tr '\n' '|')；成功句：$(grep '✓ 测试' "$scratch/out-targets.txt" | head -1)"; failures=1; fi   # gate-lint:detail
  # 挑测试目标（文件头「挑测试目标」）：样本工作区成员 demo、alpha、beta（dev 依赖 alpha）、delta（dev 依赖 demo）、验证包与产品 GPU 包；
  #   alpha 有测试目标 case_a、beta 有 case，验证包有库与 setting、gpu_pair 两个测试目标，产品 GPU 包的 real_card 写了 required-features
  local narrow_tree="$scratch/narrow-tree" narrow_member
  mkdir -p "$narrow_tree"
  printf '[workspace]\nmembers = ["crates/demo", "crates/alpha", "crates/beta", "crates/delta", "crates/singlefs-verification", "crates/singlefs-background-gpu"]\n' > "$narrow_tree/Cargo.toml"
  for narrow_member in demo alpha beta delta singlefs-verification singlefs-background-gpu; do
    mkdir -p "$narrow_tree/crates/$narrow_member/src" "$narrow_tree/crates/$narrow_member/tests"; printf 'pub fn f() {}\n' > "$narrow_tree/crates/$narrow_member/src/lib.rs"
    printf '[package]\nname = "%s"\n' "$narrow_member" > "$narrow_tree/crates/$narrow_member/Cargo.toml"
  done
  printf '\n[dev-dependencies]\nalpha = { path = "../alpha" }\n' >> "$narrow_tree/crates/beta/Cargo.toml"
  printf '\n[dev-dependencies]\ndemo = { path = "../demo" }\n' >> "$narrow_tree/crates/delta/Cargo.toml"
  printf '\n[[test]]\nname = "real_card"\nrequired-features = ["tests-on-a-real-gpu"]\n' >> "$narrow_tree/crates/singlefs-background-gpu/Cargo.toml"
  printf 'fn main() {}\n' > "$narrow_tree/crates/alpha/tests/case_a.rs"
  printf 'fn main() {}\n' > "$narrow_tree/crates/singlefs-verification/tests/setting.rs"
  narrow_patch() {   # $1 补丁目录，其余是改到的路径；树里那几份写成打过之后的内容（--skip-apply 先核补丁打过没有）
    mkdir -p "$1"; local narrow_path; : > "$1/crates.patch"
    for narrow_path in "${@:2}"; do printf -- '--- a/%s\n+++ b/%s\n@@ -1 +1 @@\n-1\n+2\n' "$narrow_path" "$narrow_path" >> "$1/crates.patch"
      mkdir -p "$(dirname "$narrow_tree/$narrow_path")"; printf '2\n' > "$narrow_tree/$narrow_path"; done
  }
  narrow_case() {   # $1 标签，其余是参数
    FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-narrow-$1.txt" PATH="$fake_cargo_dir:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" \
      MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-narrow-$1" MERGE_AND_VERIFY_TREE="$narrow_tree" env -u CARGO_TARGET_DIR bash "$0" "${@:2}" > "$out" 2>&1; rc=$?
  }
  narrow_tests() { grep '^test' "$scratch/calls-narrow-$1.txt" 2>/dev/null | tr '\n' '|'; }   # <标签>：假 cargo 收到的测试命令，| 连起来
  # 补丁改到 alpha 的库（src 下的数据文件）与 beta 的测试目标 case：只起这两个目标，各一条（不带上依赖 alpha 的 beta 的别的目标、也不跑 demo、delta）
  narrow_patch "$scratch/narrow-two" crates/alpha/src/table.txt crates/beta/tests/case.rs
  out="$scratch/out-narrow-two.txt"; narrow_case two --skip-apply "$scratch/narrow-two"; checked=$((checked+1))
  if [[ $rc -ne 0 || "$(narrow_tests two)" != "test --offline --no-fail-fast -p alpha --lib|test --offline --no-fail-fast -p beta --test case|" ]] \
     || ! grep -qF '核过 1 份补丁都已打在树上' "$out" || ! grep -q '✓ 测试全绿.*跑了 2 项（2 个包）' "$out"; then
    echo "  ✗ 自检：补丁改到 alpha 的库与 beta 的测试目标 case：测试那一步只该起 alpha --lib 与 beta --test case 两条，实际退出码 $rc，假 cargo 收到 $(narrow_tests two)：$(grep -E '✓ 测试|✗' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 补丁改到 alpha 的 src/lib.rs、变异行点名 alpha 的测试目标 case_a（带筛选词）：只起 alpha 一条 --lib --test case_a，依赖 alpha 的 beta 一个目标都不起
  narrow_patch "$scratch/narrow-alpha" crates/alpha/src/lib.rs
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'alpha 的 case_a（自证样本）' crates/alpha/src/lib.rs 2 3 '-p alpha --test case_a -- a_filter' a_filter > "$scratch/narrow-alpha/mutations-append.tsv"
  out="$scratch/out-narrow-alpha.txt"; narrow_case alpha --skip-apply "$scratch/narrow-alpha"; checked=$((checked+1))
  if [[ $rc -ne 0 || "$(narrow_tests alpha)" != "test --offline --no-fail-fast -p alpha --lib --test case_a|" ]] || ! grep -qF '筛选词 a_filter' "$out"; then
    echo "  ✗ 自检：补丁改到 alpha 的 src/lib.rs、变异行点名 alpha 的 case_a：测试那一步只该起 alpha --lib 与 --test case_a，不起依赖 alpha 的 beta，实际退出码 $rc，假 cargo 收到 $(narrow_tests alpha)：$(grep -E '✓ 测试|筛选词|✗' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 判不出测试目标的四种（文件头「挑测试目标」）：测试那一步不跑、别的步照跑（build 照起）、整条退 3、逐条报原因与整包归提交时的出路
  narrow_patch "$scratch/narrow-lock" Cargo.lock
  narrow_patch "$scratch/narrow-loose" docs/notes.txt
  narrow_patch "$scratch/narrow-unmapped" crates/alpha/tests/fixture.txt
  local refusal_label refusal_text
  for refusal_label in none lock loose unmapped; do
    out="$scratch/out-narrow-refuse-$refusal_label.txt"; checked=$((checked+1))
    case "$refusal_label" in
      none) refusal_text='没给补丁目录'; narrow_case refuse-none --skip-apply ;;
      lock) refusal_text='补丁改到工作区根的 Cargo.lock'; narrow_case refuse-lock --skip-apply "$scratch/narrow-lock" ;;
      loose) refusal_text='补丁改到不落在任何包里的 docs/notes.txt'; narrow_case refuse-loose --skip-apply "$scratch/narrow-loose" ;;
      unmapped) refusal_text='补丁改到 alpha 包里的 tests/fixture.txt，判不出改到哪个测试目标'; narrow_case refuse-unmapped --skip-apply "$scratch/narrow-unmapped" ;;
    esac
    if [[ $rc -ne 3 || -n "$(narrow_tests "refuse-$refusal_label")" ]] || ! grep -qF -- "$refusal_text" "$out" || ! grep -qF '整包归提交时的整轮门禁；提交之外要整包跑先弹窗问用户' "$out" \
       || ! grep -qxF -- "build --offline --all-targets" "$scratch/calls-narrow-refuse-$refusal_label.txt" 2>/dev/null; then
      echo "  ✗ 自检：判不出测试目标：$refusal_text 时测试那一步不跑、别的步照跑、退 3，实际退出码 $rc，假 cargo 收到的测试命令 $(narrow_tests "refuse-$refusal_label")：$(grep -E '判不出|✗' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  done
  rm -rf -- "${narrow_tree:?}/docs" "$narrow_tree/crates/alpha/tests/fixture.txt"
  # --all-packages 不是认得的参数（不留整包跑的选项）：给了就退 2、点名它、一样都不做
  out="$scratch/out-narrow-all-packages.txt"; narrow_case all-packages --skip-apply --all-packages "$scratch/narrow-two"; checked=$((checked+1))
  if [[ $rc -ne 2 || -s "$scratch/calls-narrow-all-packages.txt" ]] || ! grep -qF '认不出参数 --all-packages' "$out"; then
    echo "  ✗ 自检：给了 --all-packages（不认的参数）应当退 2、点名它、一样都不做，实际退出码 $rc，假 cargo 收到 $(grep -c . "$scratch/calls-narrow-all-packages.txt" 2>/dev/null) 条"; failures=1; fi   # gate-lint:detail
  # --skip-apply 给的补丁没打在树上（树里那一份还是补丁前的 1）：退 1、点名那份补丁、不编
  narrow_patch "$scratch/narrow-unapplied" crates/alpha/src/unapplied.txt
  printf '1\n' > "$narrow_tree/crates/alpha/src/unapplied.txt"
  out="$scratch/out-narrow-unapplied.txt"; narrow_case unapplied --skip-apply "$scratch/narrow-unapplied"; checked=$((checked+1))
  rm -f "$narrow_tree/crates/alpha/src/unapplied.txt"
  if [[ $rc -ne 1 ]] || ! grep -qF "没打在要验的树" "$out" || ! grep -qF "$scratch/narrow-unapplied" "$out" || [[ -s "$scratch/calls-narrow-unapplied.txt" ]]; then
    echo "  ✗ 自检：--skip-apply 给的补丁没打在树上：应当退 1、点名那份补丁、不编，实际退出码 $rc，假 cargo 收到 $(cat "$scratch/calls-narrow-unapplied.txt" 2>/dev/null | wc -l) 条：$(grep -E '✗|核过|本次未核' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # --skip-apply 给的两份补丁改同一行（先打的 1 改成 2，后打的 2 改成 3，树里是两份都打过之后的 3），照打的次序给：临时副本里倒着逐份倒得掉，判都已打上、退 0、树不动
  #   （弄坏开关 reverse-check-each-alone 下先打的那一份单独反着核不过，这一格红）
  mkdir -p "$scratch/narrow-stacked-first" "$scratch/narrow-stacked-second"
  printf -- '--- a/crates/alpha/src/stacked.txt\n+++ b/crates/alpha/src/stacked.txt\n@@ -1 +1 @@\n-1\n+2\n' > "$scratch/narrow-stacked-first/crates.patch"
  printf -- '--- a/crates/alpha/src/stacked.txt\n+++ b/crates/alpha/src/stacked.txt\n@@ -1 +1 @@\n-2\n+3\n' > "$scratch/narrow-stacked-second/crates.patch"
  printf '3\n' > "$narrow_tree/crates/alpha/src/stacked.txt"
  out="$scratch/out-narrow-stacked.txt"; narrow_case stacked --skip-apply "$scratch/narrow-stacked-first" "$scratch/narrow-stacked-second"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -qF '核过 2 份补丁都已打在树上（临时副本里照给的次序倒着逐份 git apply --reverse' "$out" || [[ "$(cat "$narrow_tree/crates/alpha/src/stacked.txt")" != 3 ]]; then
    echo "  ✗ 自检：--skip-apply 给的两份补丁改同一行、照打的次序给：应当判都已打上、退 0、要验的树不动，实际退出码 $rc，树里那一份是「$(cat "$narrow_tree/crates/alpha/src/stacked.txt" 2>/dev/null)」：$(grep -E '✗|核过|本次未核' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  rm -f "$narrow_tree/crates/alpha/src/stacked.txt"
  # 临时目录不认保留属性（假 cp 见 -p、-a、--preserve 带权限就照那种文件系统报 Operation not supported）：--skip-apply 照样核得过、退 0
  #   （弄坏开关 reverse-check-copy-preserves 下带 -p 拷，这一格红）
  mkdir -p "$scratch/fake-cp-no-preserve"; real_cp="$(command -v cp)"
  printf '#!/usr/bin/env bash\nfor a in "$@"; do case "$a" in -p|-a|-*p*|--preserve|--preserve=*mode*|--preserve=all|--archive) [[ "$a" == --no-preserve* || "$a" == --parents ]] && continue; echo "cp: preserving permissions for ‘${@: -1}’: Operation not supported" >&2; exit 1;; esac; done\nexec %q "$@"\n' "$real_cp" > "$scratch/fake-cp-no-preserve/cp"
  chmod +x "$scratch/fake-cp-no-preserve/cp"
  narrow_patch "$scratch/narrow-no-preserve" crates/alpha/src/no-preserve.txt
  out="$scratch/out-narrow-no-preserve.txt"; PATH="$scratch/fake-cp-no-preserve:$PATH" narrow_case no-preserve --skip-apply "$scratch/narrow-no-preserve"; checked=$((checked+1))
  rm -f "$narrow_tree/crates/alpha/src/no-preserve.txt"
  if [[ $rc -ne 0 ]] || ! grep -qF '核过 1 份补丁都已打在树上' "$out"; then
    echo "  ✗ 自检：临时目录不认保留属性时 --skip-apply 照样核得过补丁已打上、退 0，实际退出码 $rc：$(grep -E '✗|核过|拷不进' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # check.sh 读不出 lint 表、clippy 那一行没带 lint 表、fmt 那一条换成认不出的包装、test 那一行带了收窄目标的标志、缺省特性组合只剩一个（照它 CHECK_BREAK=single-clippy-pass 的写法）、
  # 循环改成只取一个组合：退 1，一个补丁都不打、不编
  sed '/^CODE_DISCIPLINE_LINTS=(/,/^)/d' "$CHECK_SH" > "$scratch/check-no-table.sh"
  sed 's/ "\${CODE_DISCIPLINE_LINTS\[@\]}" \\$/ \\/' "$CHECK_SH" > "$scratch/check-clippy-lints-dropped.sh"
  sed 's/^\(run_cargo\|cargo\) fmt /"${cargo_command[@]}" fmt /' "$CHECK_SH" > "$scratch/check-fmt-unknown-wrapper.sh"
  sed 's/\(cargo test --all\) 2>&1/\1 --lib 2>\&1/' "$CHECK_SH" > "$scratch/check-test-narrowed.sh"
  sed 's/^CLIPPY_FEATURE_COMBINATIONS=(.*)[[:space:]]*$/CLIPPY_FEATURE_COMBINATIONS=("--all-features")/' "$CHECK_SH" > "$scratch/check-single-combination.sh"
  sed 's/for clippy_combination in "\${CLIPPY_FEATURE_COMBINATIONS\[@\]}"; do/for clippy_combination in "${CLIPPY_FEATURE_COMBINATIONS[@]:0:1}"; do/' "$CHECK_SH" > "$scratch/check-combination-loop-one.sh"
  for check_case in no-table clippy-lints-dropped fmt-unknown-wrapper test-narrowed single-combination combination-loop-one; do
    case "$check_case" in
      no-table) check_case_text='里读不出 lint 表' ;;
      clippy-lints-dropped) check_case_text='的 clippy 那一行没带 lint 表' ;;
      fmt-unknown-wrapper) check_case_text='的 fmt 那一条换成认不出的包装' ;;
      test-narrowed) check_case_text='的 test 那一行带了收窄目标的标志 --lib ' ;;   # 结尾留一个空格：拼上「时应当」后 --lib 与「时」之间要有空格，与文件头点名的原文一致
      single-combination) check_case_text='的缺省特性组合只剩一个' ;;
      combination-loop-one) check_case_text='的 clippy 循环改成只取一个组合' ;;
    esac
    out="$scratch/out-check-$check_case.txt"; checked=$((checked+1))
    if cmp -s "$CHECK_SH" "$scratch/check-$check_case.sh"; then   # 样本造不出（check.sh 里没有要改的那一行）：这一格不算判过
      echo "  ✗ 自检：check.sh ${check_case_text}时应当退 1、不编，样本没造出来（sed 在 $CHECK_SH 里一处都没改到）"; failures=1; continue; fi   # gate-lint:detail
    FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-check-$check_case.txt" MERGE_AND_VERIFY_CHECK_SH="$scratch/check-$check_case.sh" run_case "check-$check_case" --skip-apply
    if [[ $rc -ne 1 ]] || ! grep -q 'check.sh 读不出、或与这里对不上' "$out" || [[ -s "$scratch/calls-check-$check_case.txt" ]]; then
      echo "  ✗ 自检：check.sh ${check_case_text}时应当退 1、不编，实际退出码 $rc，假 cargo 收到 $(cat "$scratch/calls-check-$check_case.txt" 2>/dev/null | wc -l) 条：$(grep -E '✗|!' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  done
  # cargo 命令现读（文件头「cargo 命令现读」）：check.sh 只换包装、或给命令加标志时，不判红，照读到的写法跑。基线是真 check.sh 同一趟 --skip-apply 收到的假 cargo 命令
  out="$scratch/out-read-baseline.txt"; checked=$((checked+1))
  FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-read-baseline.txt" run_case read-baseline --skip-apply "$scratch/patch-demo-lib"
  baseline_calls="$(sort "$scratch/calls-read-baseline.txt" 2>/dev/null)"
  if [[ $rc -ne 0 || -z "$baseline_calls" ]]; then
    echo "  ✗ 自检：cargo 命令现读：真 check.sh 同一趟 --skip-apply 应当退 0、假 cargo 收到命令（后面两格拿它当基线），实际退出码 $rc：$(grep -E '✗' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 包装：命令位置上写成 run_cargo <子命令>（check.sh 此刻不是这样就改成这样，已经是这样就不变），收到的命令要与基线逐条相同
  sed 's/^\([[:space:]]*\(if \)\?\)\(run_cargo\|cargo\) \(fmt\|clippy\|build\|test\) /\1run_cargo \4 /' "$CHECK_SH" > "$scratch/check-wrapper-run-cargo.sh"
  out="$scratch/out-wrapper-run-cargo.txt"; checked=$((checked+1))
  if [[ "$(grep -cE '^[[:space:]]*(if[[:space:]]+)?run_cargo[[:space:]]+(fmt|clippy|build|test)[[:space:]]' "$scratch/check-wrapper-run-cargo.sh")" != 4 ]]; then
    echo "  ✗ 自检：check.sh 的 cargo 命令包了 run_cargo 时应当照读、收到的命令与真 check.sh 相同，样本没造出来（sed 之后 $CHECK_SH 里命令位置上的 run_cargo <子命令> 不是 4 条）"; failures=1   # gate-lint:detail
  else
    FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-wrapper-run-cargo.txt" MERGE_AND_VERIFY_CHECK_SH="$scratch/check-wrapper-run-cargo.sh" run_case wrapper-run-cargo --skip-apply "$scratch/patch-demo-lib"
    if [[ $rc -ne 0 || "$(sort "$scratch/calls-wrapper-run-cargo.txt" 2>/dev/null)" != "$baseline_calls" ]]; then
      echo "  ✗ 自检：check.sh 的 cargo 命令包了 run_cargo 时应当照读、收到的命令与真 check.sh 相同，实际退出码 $rc，假 cargo 收到 $(wc -l < "$scratch/calls-wrapper-run-cargo.txt" 2>/dev/null) 条（基线 $(wc -l <<< "$baseline_calls") 条）：$(grep -E '✗|!' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  fi
  # 加标志：四条命令各加一个标志（fmt --verbose、clippy -W clippy::pedantic、build --locked、test --locked），收到的命令要带上它们（读来的，不是这里写死的）
  sed -e 's/cargo fmt --all -- --check/cargo fmt --all --verbose -- --check/' \
      -e 's/-- -D warnings "\${CODE_DISCIPLINE_LINTS\[@\]}"/-- -D warnings -W clippy::pedantic "${CODE_DISCIPLINE_LINTS[@]}"/' \
      -e 's/cargo build --all-targets /cargo build --all-targets --locked /' \
      -e 's/\(cargo test --all\) 2>&1/\1 --locked 2>\&1/' "$CHECK_SH" > "$scratch/check-flags-added.sh"
  out="$scratch/out-flags-added.txt"; checked=$((checked+1))
  if [[ "$(diff "$CHECK_SH" "$scratch/check-flags-added.sh" | grep -c '^>')" != 4 ]]; then
    echo "  ✗ 自检：check.sh 的四条 cargo 命令各加一个标志时应当照读、收到的命令带上它们，样本没造出来（sed 在 $CHECK_SH 里没改到四行）"; failures=1   # gate-lint:detail
  else
    FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-flags-added.txt" MERGE_AND_VERIFY_CHECK_SH="$scratch/check-flags-added.sh" run_case flags-added --skip-apply "$scratch/patch-demo-lib"
    flagged_calls="$(cat "$scratch/calls-flags-added.txt")"
    flagged_clippy_count="$(grep -cF -- ' -- -D warnings -W clippy::pedantic -D ' <<< "$flagged_calls")"
    if [[ $rc -ne 0 ]] || ! grep -qxF -- "fmt --all --verbose -- --check" <<< "$flagged_calls" || ! grep -qxF -- "build --offline --all-targets --locked" <<< "$flagged_calls" \
       || ! grep -qxF -- "test --offline --no-fail-fast --locked -p demo --lib" <<< "$flagged_calls" || [[ "$flagged_clippy_count" != "${#expected_combinations[@]}" ]]; then
      echo "  ✗ 自检：check.sh 的四条 cargo 命令各加一个标志时应当照读、收到的命令带上它们（fmt --verbose、build --locked、test --locked、每个组合的 clippy 都带 -W clippy::pedantic），实际退出码 $rc，clippy 带上的有 $flagged_clippy_count 条（要 ${#expected_combinations[@]} 条），假 cargo 收到：$(tr '\n' '|' <<< "$flagged_calls")"; failures=1; fi   # gate-lint:detail
  fi
  # 工作区成员里没有 singlefs-verification（样本里那个成员改了名）：认不出验证包，退 1、不编
  write_workspace "$scratch/tree-renamed" singlefs-verification-renamed
  mkdir -p "$scratch/tree-renamed/crates/demo/src"; printf 'pub fn f() {}\n' > "$scratch/tree-renamed/crates/demo/src/lib.rs"
  out="$scratch/out-renamed.txt"; checked=$((checked+1))
  PATH="$fake_cargo_dir:$PATH" FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-renamed.txt" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-renamed" MERGE_AND_VERIFY_TREE="$scratch/tree-renamed" env -u CARGO_TARGET_DIR bash "$0" --skip-apply > "$out" 2>&1; rc=$?
  if [[ $rc -ne 1 ]] || ! grep -q '认不出它就排不掉登记的崩溃枚举用例' "$out" || grep -q '^test' "$scratch/calls-renamed.txt" 2>/dev/null; then
    echo "  ✗ 自检：工作区成员里没有 singlefs-verification 时应当退 1、不编，实际退出码 $rc：$(grep '^test' "$scratch/calls-renamed.txt" 2>/dev/null | tr '\n' ' ')$(grep -E '✗' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  # cargo test 退 0 而一条测试都没跑（假 cargo 什么都不打）：退 1
  out="$scratch/out-empty.txt"; checked=$((checked+1)); FAKE_CARGO_MODE=empty run_case empty --skip-apply "$scratch/patch-demo-lib"
  if [[ $rc -ne 1 ]] || ! grep -q '一条测试都没跑' "$out"; then
    echo "  ✗ 自检：cargo test 退 0 却一条测试都没跑时应当退 1，实际退出码 $rc：$(grep -E '✓ 测试|✗' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 没设日志目录：日志落在工作根下（SINGLEFS_WORK_ROOT 指进临时目录）
  out="$scratch/out4.txt"; checked=$((checked+1))
  PATH="$fake_cargo_dir:$PATH" FAKE_CARGO_MODE=green MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" SINGLEFS_WORK_ROOT="$scratch/work-root" \
    MERGE_AND_VERIFY_TREE="$scratch/tree" env -u MERGE_AND_VERIFY_LOG_DIRECTORY -u CARGO_TARGET_DIR bash "$0" "$scratch/patch-a" > "$out" 2>&1; rc=$?
  if [[ $rc -ne 0 ]] || ! compgen -G "$scratch/work-root/merge-and-verify/*/test.log" > /dev/null; then
    echo "  ✗ 自检：没设日志目录时日志应当落在工作根下（$scratch/work-root/merge-and-verify/<日期>-<进程号>/），实际退出码 $rc：$(grep '日志在' "$out" | tail -1)"; failures=1; fi   # gate-lint:detail
  # 换了源码却没重编（调用方给的编译目录 <树>/target 两趟共用，收尾不删；快照不拷它）：临时树先验一趟（编过），再从源树 rsync -a 铺进新的 walk.rs（mtime 比编译目录里的 dep-info 旧，取 2020 年初的 epoch 秒）；
  # 假 cargo 照 mtime 判新不新鲜（source-stamp.py --write-selftest-cargo 写的那一份）：快照里的修改时刻是拷的那一刻（文件头「快照」），walk.rs 照新内容重编；
  # 快照带着源树的旧 mtime（弄坏开关 snapshot-keeps-times）就认 walk.rs 已编、那一条 FAILED
  local stale_tree="$scratch/stale-tree" stale_source="$scratch/stale-source" stale_bin="$scratch/stale-bin"
  mkdir -p "$stale_tree/crates/demo/src" "$stale_source/crates/demo/src" "$stale_bin"
  write_workspace "$stale_tree" singlefs-verification
  python3 "$STAMP" --write-selftest-cargo "$stale_bin/cargo"
  printf 'pub mod walk;\n' > "$stale_tree/crates/demo/src/lib.rs"
  printf 'pub fn walk() -> u32 { 1 }\n' > "$stale_tree/crates/demo/src/walk.rs"
  touch -d @1577836800 "$stale_tree/crates/demo/src/lib.rs" "$stale_tree/crates/demo/src/walk.rs"
  cp --preserve=timestamps "$stale_tree/crates/demo/src/lib.rs" "$stale_source/crates/demo/src/lib.rs"
  printf 'pub fn walk() -> u32 { 22 }\n' > "$stale_source/crates/demo/src/walk.rs"
  touch -d @1577836900 "$stale_source/crates/demo/src/walk.rs"
  stale_case() { PATH="$stale_bin:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-$1" \
    MERGE_AND_VERIFY_TREE="$stale_tree" env CARGO_TARGET_DIR="$stale_tree/target" bash "$0" --skip-apply "$scratch/patch-demo-lib" > "$out" 2>&1; rc=$?; }
  out="$scratch/out5-first.txt"; stale_case five-first
  rsync -a "$stale_source/" "$stale_tree/"
  out="$scratch/out5.txt"; stale_case five; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -q 'Compiling crates/demo/src/walk.rs' "$scratch/logs-five/test.log" 2>/dev/null || ! grep -q '测试全绿' "$out"; then
    echo "  ✗ 自检：往编过的副本里 rsync -a 铺新内容（旧 mtime）之后再验，应当重编改过的 walk.rs、测试全绿，实际退出码 $rc：$(grep -E 'Fresh|Compiling|FAILED' "$scratch/logs-five/test.log" 2>/dev/null | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 改过的包重编了没有（文件头；调用方给的编译目录 <树>/target 几趟共用，收尾不删）：假 cargo 按包、按 mtime 判新不新鲜，重编时照真 cargo 打「Compiling <包名> v…」（lib_rebuilt_packages.py --write-selftest-cargo）；
  # 补丁目录 rebuilt-patch 的 crates.patch 改的是 crates/demo/src/walk.rs（树里已经是改过的样子，--skip-apply 只验不打）
  local rebuilt_tree="$scratch/rebuilt-tree" rebuilt_bin="$scratch/rebuilt-bin" rebuilt_patch="$scratch/rebuilt-patch"
  mkdir -p "$rebuilt_tree/crates/demo/src" "$rebuilt_bin" "$rebuilt_patch"
  write_workspace "$rebuilt_tree" singlefs-verification
  printf 'pub mod walk;\n' > "$rebuilt_tree/crates/demo/src/lib.rs"
  printf 'pub fn walk() -> u32 { 22 }\n' > "$rebuilt_tree/crates/demo/src/walk.rs"
  printf 'diff --git a/crates/demo/src/walk.rs b/crates/demo/src/walk.rs\n--- a/crates/demo/src/walk.rs\n+++ b/crates/demo/src/walk.rs\n@@ -1 +1 @@\n-pub fn walk() -> u32 { 1 }\n+pub fn walk() -> u32 { 22 }\n' \
    > "$rebuilt_patch/crates.patch"
  python3 "$REBUILT" --write-selftest-cargo "$rebuilt_bin/cargo"
  rebuilt_case() { PATH="$rebuilt_bin:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-$1" \
    MERGE_AND_VERIFY_TREE="$rebuilt_tree" env CARGO_TARGET_DIR="$rebuilt_tree/target" bash "$0" --skip-apply "$rebuilt_patch" > "$out" 2>&1; rc=$?; }
  # 绿样本：头一次编（编译目录是空的），demo 有 Compiling 行
  out="$scratch/out-rebuilt-first.txt"; rebuilt_case rebuilt-first; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -q '改过的包这一趟都重编了（核了 1 个包：demo）' "$out"; then
    echo "  ✗ 自检：头一次编、补丁改过的 demo 有 Compiling 行：应当退 0、报都重编了，实际退出码 $rc：$(grep -E '✗|✓|重编' "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 编过之后再验同一批补丁：快照里的修改时刻是拷的那一刻，比上一趟的 dep-info 新，demo 照样重编（弄坏开关 snapshot-keeps-times 下快照带着源树的旧 mtime，demo 不重编）
  out="$scratch/out-rebuilt-rerun.txt"; rebuilt_case rebuilt-rerun; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -q '改过的包这一趟都重编了（核了 1 个包：demo）' "$out"; then
    echo "  ✗ 自检：编过之后 --skip-apply 再验同一批补丁：改过的 demo 应当重编、退 0，实际退出码 $rc：$(grep -E '✗|✓|重编|刷到此刻' "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 红样本：编译目录里 demo 的产物比快照的拷入时刻还新（别的趟共用这个编译目录、后编的那种）；
  # 验证包那个包的产物是上一趟编的、比快照旧，照常重编（有一行别的 Compiling），demo 没有 Compiling 行，测试照样全绿
  touch -d '+1 day' "$rebuilt_tree/target/debug/.fake/demo.d"
  touch "$rebuilt_tree/crates/singlefs-verification/Cargo.toml"
  out="$scratch/out-rebuilt-stale.txt"; rebuilt_case rebuilt-stale; checked=$((checked+1))
  if [[ $rc -ne 1 ]] || ! grep -q '改过的包这一趟没重编：demo' "$out" || ! grep -q 'Compiling singlefs-verification' "$scratch/logs-rebuilt-stale/test.log" 2>/dev/null; then
    echo "  ✗ 自检：编译目录里的产物比源码新（共用编译目录）、补丁改过的 demo 没有 Compiling 行：应当退 1、点名 demo，实际退出码 $rc：$(grep -E '✗|✓|重编' "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 跑完运行目录里不剩它建的编译目录与快照（文件头「这一趟的编译目录」「快照」）：没设 CARGO_TARGET_DIR 的三种结局——全绿退 0（out3 那一趟）、测试红退 1（out1 那一趟）、
  # 被 TERM 停（假 cargo test 睡 2 秒，时间线里出现它开始之后给脚本发 TERM，脚本等假 cargo 退出后走 EXIT trap，退 143）——它开跑时打的编译目录都不在、输出里有删了它那一行；
  # 调用方设了 CARGO_TARGET_DIR 的（上面重编核对那几趟）照它编、收尾不删
  local term_pid term_rc term_attempt removal_problems="" three_target one_target term_target
  three_target="$(run_target_of "$scratch/out3.txt")"; one_target="$(run_target_of "$scratch/out1.txt")"
  [[ -n "$three_target" && ! -e "$three_target" ]] && grep -qF "删了这一趟的编译目录 $three_target" "$scratch/out3.txt" || removal_problems+="全绿那一趟（out3）还剩 $(ls -d "${three_target:-没打编译目录那一行}" 2>/dev/null || echo 没剩、但没打删了那一行)；"
  [[ -n "$one_target" && ! -e "$one_target" ]] && grep -qF "删了这一趟的编译目录 $one_target" "$scratch/out1.txt" || removal_problems+="测试红那一趟（out1）还剩 $(ls -d "${one_target:-没打编译目录那一行}" 2>/dev/null || echo 没剩、但没打删了那一行)；"
  PATH="$rebuilt_bin:$PATH" FAKE_CARGO_TIMELINE="$scratch/term-timeline.txt" FAKE_CARGO_TAG=T FAKE_CARGO_SLEEP=2 MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-term" MERGE_AND_VERIFY_TREE="$rebuilt_tree" env -u CARGO_TARGET_DIR bash "$0" --skip-apply "$rebuilt_patch" > "$scratch/out-term.txt" 2>&1 &
  term_pid=$!
  for term_attempt in $(seq 1 200); do grep -q '^start T ' "$scratch/term-timeline.txt" 2>/dev/null && break; kill -0 "$term_pid" 2>/dev/null || break; sleep 0.05; done
  kill -TERM "$term_pid" 2>/dev/null
  if wait "$term_pid"; then term_rc=0; else term_rc=$?; fi
  term_target="$(run_target_of "$scratch/out-term.txt")"
  ((term_rc == 143)) && [[ -n "$term_target" && ! -e "$term_target" ]] && grep -qF "删了这一趟的编译目录 $term_target" "$scratch/out-term.txt" \
    || removal_problems+="被 TERM 停的那一趟退 $term_rc（要 143），还剩 $(ls -d "${term_target:-没打编译目录那一行}" 2>/dev/null || echo 没剩)：$(tail -2 "$scratch/out-term.txt" | tr '\n' ' ')；"
  [[ -d "$rebuilt_tree/target/debug" ]] && ! grep -q '删了这一趟的编译目录' "$scratch/out-rebuilt-first.txt" || removal_problems+="调用方设的 CARGO_TARGET_DIR 收尾被删了或打了删了那一行；"
  local snapshot_label
  for snapshot_label in three one term; do
    [[ ! -e "$scratch/logs-$snapshot_label/repo" ]] || removal_problems+="$snapshot_label 那一趟的快照 $scratch/logs-$snapshot_label/repo 还在；"
  done
  checked=$((checked+1))
  if [[ -n "$removal_problems" ]]; then
    echo "  ✗ 自检：跑完运行目录里不剩它建的编译目录与快照（正常、出错、被 TERM 三种结局都删它开跑时打的那一份与快照，调用方设的 CARGO_TARGET_DIR 不删），实际：$removal_problems"; failures=1; fi   # gate-lint:detail
  # 这一趟的编译目录里还有进程开着文件（假 cargo test 留下一个后台 sleep 开着 <编译目录>/held，进程号写进 holder-pid）：不删、报出那个进程、退 1；看完只停那一个进程号
  local holder_bin="$scratch/holder-bin" holder_pid
  mkdir -p "$holder_bin"
  printf '#!/usr/bin/env bash\nif [[ "$1" == test ]]; then mkdir -p "$CARGO_TARGET_DIR"; sleep 30 > "$CARGO_TARGET_DIR/held" 2>&1 < /dev/null & echo "$!" > "$FAKE_HOLDER_PID"; fi\nexec python3 %q "$@"\n' "$rebuilt_bin/cargo" > "$holder_bin/cargo"
  chmod +x "$holder_bin/cargo"
  out="$scratch/out-holder.txt"; checked=$((checked+1))
  PATH="$holder_bin:$PATH" FAKE_HOLDER_PID="$scratch/holder-pid" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-holder" MERGE_AND_VERIFY_TREE="$rebuilt_tree" env -u CARGO_TARGET_DIR bash "$0" --skip-apply "$rebuilt_patch" > "$out" 2>&1; rc=$?
  holder_pid="$(cat "$scratch/holder-pid" 2>/dev/null)"
  local holder_target; holder_target="$(run_target_of "$out")"
  if [[ $rc -ne 1 || -z "$holder_target" || ! -d "$holder_target" ]] || ! grep -q "还有 1 个进程开着里面的文件" "$out" || ! grep -q "进程 ${holder_pid:-没有}（sleep 30）" "$out"; then
    echo "  ✗ 自检：这一趟的编译目录里还有进程开着文件：应当不删、报出那个进程、退 1，实际退出码 $rc，编译目录$([[ -n "$holder_target" && -d "$holder_target" ]] && echo 还在 || echo 删了)：$(grep -E '✗|删了|进程' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  [[ ! "$holder_pid" =~ ^[0-9]+$ ]] || kill "$holder_pid" 2>/dev/null
  # 编译目录不在工作根下（文件头「这一趟的编译目录」）：不给日志目录、工作根指进 scratch，编译目录要是 TMPDIR 下这一趟新建的那一份，日志目录照旧建在工作根下
  local placement_target placement_logs physical_temporary physical_work_root
  mkdir -p "$scratch/work-root"
  physical_temporary="$(cd "$TMPDIR" && pwd -P)"; physical_work_root="$(cd "$scratch/work-root" && pwd -P)"
  out="$scratch/out-placement.txt"; checked=$((checked+1))
  PATH="$rebuilt_bin:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" SINGLEFS_WORK_ROOT="$scratch/work-root" MERGE_AND_VERIFY_TREE="$rebuilt_tree" \
    env -u CARGO_TARGET_DIR -u MERGE_AND_VERIFY_LOG_DIRECTORY bash "$0" --skip-apply > "$out" 2>&1; rc=$?
  placement_target="$(run_target_of "$out")"
  placement_logs="$(ls -d "$scratch/work-root/merge-and-verify/"*/ 2>/dev/null | head -1)"
  if [[ -z "$placement_target" || "$placement_target" == "$scratch/work-root/"* || "$placement_target" == "$physical_work_root/"* \
        || "$placement_target" != "$physical_temporary/"* || -z "$placement_logs" ]]; then
    echo "  ✗ 自检：编译目录不在工作根下：应当建在 TMPDIR（$physical_temporary）下这一趟新建的一份、日志目录照旧在工作根下，实际退出码 $rc，编译目录「$placement_target」，日志目录「$placement_logs」"; failures=1; fi   # gate-lint:detail
  # 收尾时前面一步写满盘（MERGE_AND_VERIFY_SELFTEST_FULL_DISK 演：删快照那一步写不进去、整条收尾停在那里）：编译目录在那之前删过，照删
  local full_disk_target
  out="$scratch/out-full-disk.txt"; checked=$((checked+1))
  PATH="$rebuilt_bin:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_SELFTEST_FULL_DISK=1 \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-full-disk" MERGE_AND_VERIFY_TREE="$rebuilt_tree" env -u CARGO_TARGET_DIR bash "$0" --skip-apply > "$out" 2>&1; rc=$?
  full_disk_target="$(run_target_of "$out")"
  if [[ $rc -ne 1 || -z "$full_disk_target" || -e "$full_disk_target" ]] || ! grep -qF 'No space left on device' "$out"; then
    echo "  ✗ 自检：收尾时前面一步写满盘（用开关演）编译目录照删：应当退 1、编译目录不在，实际退出码 $rc，编译目录「$full_disk_target」$([[ -n "$full_disk_target" && -e "$full_disk_target" ]] && echo 还在 || echo 不在)"; failures=1; fi   # gate-lint:detail
  # 认不出的选项（文件头「上限、线程、机器」）：给了 --never-used-option 就退 2、点名它，一样都不做（假 cargo 一条都没收到）
  out="$scratch/out-unknown-option.txt"; : > "$scratch/calls-unknown-option.txt"; checked=$((checked+1))
  PATH="$fake_cargo_dir:$PATH" FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-unknown-option.txt" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-unknown-option" MERGE_AND_VERIFY_TREE="$scratch/tree" env -u CARGO_TARGET_DIR bash "$0" --skip-apply --never-used-option > "$out" 2>&1; rc=$?
  if [[ $rc -ne 2 ]] || [[ -s "$scratch/calls-unknown-option.txt" ]] || ! grep -qF '认不出参数 --never-used-option' "$out"; then
    echo "  ✗ 自检：给了认不出的选项 --never-used-option 应当退 2、点名它，一样都不做，实际退出码 $rc，假 cargo 收到 $(grep -c . "$scratch/calls-unknown-option.txt" 2>/dev/null) 条：$(grep -E '✗|→' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 验证包那一步（文件头「验证包那一步」）：假的那一道答 --list-items（登记的崩溃枚举用例 demo-case 在测试目标 setting 里，挂特性的是库与 gpu_pair.rs），
  #   别的起法打按条复用与崩溃状态块那几行；合入的树是挑测试目标那几格的 narrow-tree，那一道收到的参数记进 verification_calls
  local verification_calls="$scratch/verification-step-calls.txt" verification_environment="$scratch/verification-step-environment.txt"
  verification_starts() { grep -c -- "^$1" "$verification_calls" 2>/dev/null; }   # <参数开头>：那一道被这样起了几次
  # 补丁只改 demo 一个函数（库里一份数据文件）、变异行点名验证包的 setting：测试那一步只起 demo --lib 与验证包 --release --test setting，
  #   验证包那一步只起一次 --list-items 与一次按块那一路（--check crash-case-replay --crash-cases-only --item crash-case:demo-case），不起门禁的整格
  narrow_patch "$scratch/narrow-core" crates/demo/src/table.txt
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' '验证包的 setting（自证样本）' crates/demo/src/lib.rs 2 3 '-p singlefs-verification --release --test setting -- case_filter --exact' case_filter \
    > "$scratch/narrow-core/mutations-append.tsv"
  out="$scratch/out-verification-core.txt"; : > "$verification_calls"; checked=$((checked+1))
  FAKE_VERIFICATION_CALLS="$verification_calls" narrow_case verification-core --skip-apply "$scratch/narrow-core"
  if [[ $rc -ne 0 || "$(narrow_tests verification-core)" != "test --offline --no-fail-fast -p demo --lib|test --offline --no-fail-fast --release -p singlefs-verification --test setting|" ]] \
     || [[ "$(grep -c . "$verification_calls")" -ne 2 || "$(verification_starts '--list-items ')" -ne 1 ]] \
     || [[ "$(verification_starts '--check crash-case-replay --crash-cases-only --item crash-case:demo-case ')" -ne 1 ]] \
     || grep -qF 'crash-case-replay,' "$verification_calls" || ! grep -qF '✓ 验证包那一步（登记的崩溃枚举用例）过' "$out"; then
    echo "  ✗ 自检：验证包那一步：补丁只改 demo 一个函数、变异行点名验证包的 setting，应当只起 demo --lib、验证包 --release --test setting 与按块那一路的 crash-case:demo-case，不起整格，实际退出码 $rc，假 cargo 收到 $(narrow_tests verification-core)，那一道收到 $(tr '\n' '|' < "$verification_calls")：$(grep -E '验证包那一步|✗' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 补丁改到验证包的库与挂特性的 gpu_pair：测试那一步验证包一条 --release --lib --test gpu_pair，验证包那一步只交 feature-gated:lib 与 feature-gated:gpu_pair.rs（不起按块那一路）
  narrow_patch "$scratch/narrow-features" crates/singlefs-verification/src/table.txt crates/singlefs-verification/tests/gpu_pair.rs
  out="$scratch/out-verification-features.txt"; : > "$verification_calls"; checked=$((checked+1))
  FAKE_VERIFICATION_CALLS="$verification_calls" narrow_case verification-features --skip-apply "$scratch/narrow-features"
  if [[ $rc -ne 0 || "$(narrow_tests verification-features)" != "test --offline --no-fail-fast --release -p singlefs-verification --lib --test gpu_pair|" ]] \
     || [[ "$(grep -c . "$verification_calls")" -ne 2 || "$(verification_starts '--check feature-gated-tests --item feature-gated:lib --item feature-gated:gpu_pair.rs ')" -ne 1 ]] \
     || ! grep -qF '✓ 验证包那一步（挂特性的测试文件）过' "$out"; then
    echo "  ✗ 自检：验证包那一步：补丁改到验证包的库与挂特性的 gpu_pair，应当只交 feature-gated:lib 与 feature-gated:gpu_pair.rs，实际退出码 $rc，假 cargo 收到 $(narrow_tests verification-features)，那一道收到 $(tr '\n' '|' < "$verification_calls")"; failures=1; fi   # gate-lint:detail
  # 按块那一路红（假的那一道打 case_function FAILED 与 crash-case:demo-case 的 ITEM_DONE exit=1）：退 1、列出红的测试函数，出路只重跑红的那一项（种类 items、范围 2 项：1 格加 1 项）
  out="$scratch/out-verification-red.txt"; checked=$((checked+1))
  FAKE_VERIFICATION_EXIT=1 narrow_case verification-red --skip-apply "$scratch/narrow-core"
  if [[ $rc -ne 1 ]] || ! grep -qF '✗ 验证包那一步（登记的崩溃枚举用例）红' "$out" || ! grep -qx '      case_function' "$out" \
     || ! grep -qF "SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 crash-point-replay.sh 的 crash-case-replay 格点名的 crash-case:demo-case; 范围=2 项（1 格加点名的 1 项）;" "$out" \
     || ! grep -qF -- "--check crash-case-replay --crash-cases-only --item crash-case:demo-case" "$out" || ! grep -qF "task-server/task submit -- bash" "$out" \
     || grep -qF 'run-with-memory-cap.sh' "$out" || ! grep -qF '并进写段的下一批' "$out"; then
    echo "  ✗ 自检：验证包那一步：按块那一路判红时应当退 1、列出红的测试函数与只重跑红的那一项的命令，实际退出码 $rc：$(grep -E '验证包那一步|^      |→' "$out" | head -4 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 数红的测试函数（verification_red_test_names）：「test <名> ... FAILED」与「FAILED <名>」两种写法一起数、按名去重，汇总行「FAILED. N passed」不算
  checked=$((checked+1))
  printf '    test a_red ... FAILED
      FAILED a_red
      FAILED tests::b_red
test result: FAILED. 6 passed; 1 failed
    红的测试目标 x（汇总行 FAILED. 6 passed; 1 failed）：
' > "$scratch/red-names.log"
  out="$(verification_red_test_names "$scratch/red-names.log" | tr '
' ' ')"
  if [[ "$out" != "a_red tests::b_red " ]]; then
    echo "  ✗ 自检：验证包那一步：日志里两种写法的红应当一起数、按名去重（a_red、tests::b_red 两条），实际：$out"; failures=1; fi   # gate-lint:detail
  # 补丁只改 alpha、beta（跑到的目标里没有验证包的，也没有要特性的）：报本次未跑与原因，那一道一次都不起
  out="$scratch/out-verification-unaffected.txt"; : > "$verification_calls"; checked=$((checked+1))
  FAKE_VERIFICATION_CALLS="$verification_calls" narrow_case verification-unaffected --skip-apply "$scratch/narrow-two"
  if [[ $rc -ne 0 || -s "$verification_calls" ]] || ! grep -qF '  - 验证包那一步本次未跑：' "$out"; then
    echo "  ✗ 自检：验证包那一步：补丁只改到 alpha、beta 时应当报本次未跑、不起那一道，实际退出码 $rc，那一道被起 $(grep -c . "$verification_calls") 次：$(grep -E '验证包' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 补丁改到产品 GPU 包写了 required-features 的真卡对拍 real_card：测试那一步不跑它，验证包那一步起一次 --check product-gpu-real-card-tests（那一格不收点名）
  narrow_patch "$scratch/narrow-product-gpu" crates/singlefs-background-gpu/tests/real_card.rs
  out="$scratch/out-verification-product-gpu.txt"; : > "$verification_calls"; checked=$((checked+1))
  FAKE_VERIFICATION_CALLS="$verification_calls" narrow_case verification-product-gpu --skip-apply "$scratch/narrow-product-gpu"
  if [[ $rc -ne 0 || -n "$(narrow_tests verification-product-gpu)" || "$(grep -c . "$verification_calls")" -ne 1 || "$(verification_starts '--check product-gpu-real-card-tests ')" -ne 1 ]] \
     || ! grep -qF '产品 GPU 包的真卡对拍：singlefs-background-gpu 的 real_card' "$out"; then
    echo "  ✗ 自检：验证包那一步：补丁改到产品 GPU 包的真卡对拍 real_card 时应当起 --check product-gpu-real-card-tests、测试那一步不跑它，实际退出码 $rc，假 cargo 收到 $(narrow_tests verification-product-gpu)，那一道收到 $(tr '\n' '|' < "$verification_calls")"; failures=1; fi   # gate-lint:detail
  # 那一道退 3 而交给它的根不是样本根（这里是快照）：判红
  out="$scratch/out-verification-exit-three.txt"; checked=$((checked+1))
  FAKE_VERIFICATION_EXIT=3 narrow_case verification-exit-three --skip-apply "$scratch/narrow-core"
  if [[ $rc -ne 1 ]] || ! grep -qF '退 3：退 3 只在样本根上算过，交给它的根不是样本根' "$out"; then
    echo "  ✗ 自检：验证包那一步：那一道退 3 而根不是它的样本根时应当判红、退 1，实际退出码 $rc：$(grep -E '验证包' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 那一道的 --list-items 退 1：读不出登记的用例与挂特性的文件，判红
  out="$scratch/out-verification-list-red.txt"; checked=$((checked+1))
  FAKE_VERIFICATION_LIST_EXIT=1 narrow_case verification-list-red --skip-apply "$scratch/narrow-core"
  if [[ $rc -ne 1 ]] || ! grep -qF -- '--list-items 退 1' "$out"; then
    echo "  ✗ 自检：验证包那一步：那一道 --list-items 退 1 时应当判红、退 1，实际退出码 $rc：$(grep -E '验证包' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 跑在第二台上（task-server 经 multi-host-run.sh --peer 派来的那一趟：带着起活那一方的回读握手与这一趟的本地目录，两台对称、没有认台的变量）：照起那一道、报过、退 0
  out="$scratch/out-verification-peer.txt"; : > "$verification_calls"; checked=$((checked+1))
  GPU_QUIESCE_ORIGIN_READ_BACK=1 MULTI_HOST_RUN_PEER_SCRATCH="$scratch/peer-scratch" FAKE_VERIFICATION_CALLS="$verification_calls" narrow_case verification-peer --skip-apply "$scratch/narrow-core"
  if [[ $rc -ne 0 || "$(verification_starts '--check ')" -ne 1 ]] || ! grep -qF '✓ 验证包那一步（登记的崩溃枚举用例）过' "$out"; then
    echo "  ✗ 自检：验证包那一步：跑在第二台上时应当照起那一道、报过、退 0，实际退出码 $rc，那一道被起 $(grep -c . "$verification_calls") 次：$(grep -E '验证包|→' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 缩崩溃域（文件头「验证包那一步」）：--crash-expand-up-to 6 把 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6 交给那一道、汇总写缩过域、不算门禁通过、把崩溃状态块那一行的现判与复用写进汇总；
  #   不给就是整个域：那一道收到的是 unset、汇总不写缩过域；环境里已设的变量当作给了同一个值；值不是非负整数退 2、一样都不做；缩过域的那一趟红了，只重跑的命令带同一个变量
  #   （弄坏开关 expand-up-to-not-passed、scaled-run-unlabelled、rerun-command-unscaled、expand-up-to-unchecked 下对应的格红）
  out="$scratch/out-verification-scaled.txt"; : > "$verification_environment"; checked=$((checked+1))
  FAKE_VERIFICATION_ENVIRONMENT="$verification_environment" narrow_case verification-scaled --skip-apply --crash-expand-up-to 6 "$scratch/narrow-core"
  if [[ $rc -ne 0 ]] || [[ "$(cat "$verification_environment")" != 6 ]] || ! grep -q '汇总：.*缩过域、不算门禁通过（SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6，只往小缩）' "$out" \
     || ! grep -q '汇总：.*崩溃状态块 现判 3、复用 5、拒算 0' "$out"; then
    echo "  ✗ 自检：验证包那一步：--crash-expand-up-to 6 应当把 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6 交给那一道、汇总写缩过域、不算门禁通过、把崩溃状态块的现判与复用写进汇总，实际退出码 $rc，那一道收到的变量「$(tr '\n' ' ' < "$verification_environment")」：$(grep '汇总：' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  out="$scratch/out-verification-unscaled.txt"; : > "$verification_environment"; checked=$((checked+1))
  FAKE_VERIFICATION_ENVIRONMENT="$verification_environment" narrow_case verification-unscaled --skip-apply "$scratch/narrow-core"
  if [[ $rc -ne 0 ]] || [[ "$(cat "$verification_environment")" != unset ]] || grep -qF '缩过域' "$out"; then
    echo "  ✗ 自检：验证包那一步：不给 --crash-expand-up-to 时那一道收到的变量应当是 unset、汇总不写缩过域，实际退出码 $rc，那一道收到的变量「$(tr '\n' ' ' < "$verification_environment")」：$(grep '汇总：' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  out="$scratch/out-verification-scaled-environment.txt"; : > "$verification_environment"; checked=$((checked+1))
  SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=4 FAKE_VERIFICATION_ENVIRONMENT="$verification_environment" narrow_case verification-scaled-environment --skip-apply "$scratch/narrow-core"
  if [[ $rc -ne 0 ]] || [[ "$(cat "$verification_environment")" != 4 ]] || ! grep -q '汇总：.*缩过域、不算门禁通过（SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=4，只往小缩）' "$out"; then
    echo "  ✗ 自检：验证包那一步：环境里已设展开深度变量时当作给了同一个值（那一道收到 4、汇总写缩过域），实际退出码 $rc，那一道收到的变量「$(tr '\n' ' ' < "$verification_environment")」：$(grep '汇总：' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  out="$scratch/out-verification-bad-argument.txt"; checked=$((checked+1))
  narrow_case verification-bad-argument --skip-apply --crash-expand-up-to abc "$scratch/narrow-core"
  if [[ $rc -ne 2 ]] || [[ -s "$scratch/calls-narrow-verification-bad-argument.txt" ]] || ! grep -qF '不是非负整数' "$out"; then
    echo "  ✗ 自检：--crash-expand-up-to 不是非负整数时应当退 2、一样都不做（参数写 abc），实际退出码 $rc，假 cargo 收到 $(grep -c . "$scratch/calls-narrow-verification-bad-argument.txt" 2>/dev/null) 条：$(grep -E '✗' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  out="$scratch/out-verification-bad-environment.txt"; checked=$((checked+1))
  SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=abc narrow_case verification-bad-environment --skip-apply "$scratch/narrow-core"
  if [[ $rc -ne 2 ]] || [[ -s "$scratch/calls-narrow-verification-bad-environment.txt" ]] || ! grep -qF '不是非负整数' "$out"; then
    echo "  ✗ 自检：--crash-expand-up-to 不是非负整数时应当退 2、一样都不做（环境里的变量写 abc），实际退出码 $rc，假 cargo 收到 $(grep -c . "$scratch/calls-narrow-verification-bad-environment.txt" 2>/dev/null) 条：$(grep -E '✗' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  out="$scratch/out-verification-missing-value.txt"; checked=$((checked+1))
  narrow_case verification-missing-value --skip-apply "$scratch/narrow-core" --crash-expand-up-to
  if [[ $rc -ne 2 ]] || [[ -s "$scratch/calls-narrow-verification-missing-value.txt" ]] || ! grep -qF '后面要跟一个非负整数' "$out"; then
    echo "  ✗ 自检：--crash-expand-up-to 后面没有值时应当退 2、一样都不做，实际退出码 $rc，假 cargo 收到 $(grep -c . "$scratch/calls-narrow-verification-missing-value.txt" 2>/dev/null) 条：$(grep -E '✗' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  out="$scratch/out-verification-scaled-red.txt"; checked=$((checked+1))
  FAKE_VERIFICATION_EXIT=1 narrow_case verification-scaled-red --skip-apply --crash-expand-up-to 6 "$scratch/narrow-core"
  if [[ $rc -ne 1 ]] || ! grep -qF -- 'task-server/task submit -- env SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6 bash' "$out" || ! grep -q '汇总：.*缩过域、不算门禁通过' "$out"; then
    echo "  ✗ 自检：验证包那一步：缩过域的那一趟红了，只重跑的命令应当带同一个变量（task-server/task submit -- env SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6 bash …）、汇总写缩过域，实际退出码 $rc：$(grep -E '→ 怎么办|汇总：' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 跑着的时候主工作区（要验的树）里那份源码被改（文件头「快照」）：假 cargo 的 test 一开跑就把要验的树里的 lib.rs 与那一道门禁改掉，
  #   再照它此刻所在目录里的 lib.rs 报一条测试；补丁目录只有变异行（点名 demo 的库与验证包的 setting），验证包那一步照默认起快照里那一份门禁
  #   （这一格不设 MERGE_AND_VERIFY_VERIFICATION_STAGE），那一份答 --list-items（setting 里登记了一条崩溃枚举用例）、按块那一路报它收到的根里 lib.rs 的内容
  #   （当成复用键算的是哪一份源码）。测试的结论与复用的键都要照开跑时那一份（source_at_start）；
  #   弄坏开关 verify-in-main-tree 下测试编的是改后的那一份、验证包那一步起的是改后的那一份门禁，这一格红
  local edited_tree="$scratch/edited-tree" edited_bin="$scratch/edited-bin"
  mkdir -p "$edited_tree/crates/demo/src" "$edited_tree/crates/singlefs-verification/tests" "$edited_tree/.claude/gate.d" "$edited_bin" "$scratch/edited-patch"
  write_workspace "$edited_tree" singlefs-verification
  printf 'source_at_start\n' > "$edited_tree/crates/demo/src/lib.rs"
  printf 'fn main() {}\n' > "$edited_tree/crates/singlefs-verification/tests/setting.rs"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'demo 的库（自证样本）' crates/demo/src/lib.rs a b '-p demo --lib' f '验证包的 setting（自证样本）' crates/demo/src/lib.rs a b '-p singlefs-verification --test setting' f \
    > "$scratch/edited-patch/mutations-append.tsv"
  cat > "$edited_tree/.claude/gate.d/crash-point-replay.sh" <<'FAKE'
#!/usr/bin/env bash
# 开跑时那一份假门禁：--list-items 列 setting 里登记的一条崩溃枚举用例；按块那一路报它收到的根（最后一个参数）里 lib.rs 的内容
if [[ "$1" == --list-items ]]; then printf 'crash-case:edited-case\tsinglefs-verification:setting:case_function\n'; exit 0; fi
echo "  ✓ 崩溃状态块：现判 1、复用 0、拒算 0（键照 $(cat "${@: -1}/crates/demo/src/lib.rs") 算，开跑时那一份门禁）"
FAKE
  cat > "$edited_bin/cargo" <<'FAKE'
#!/usr/bin/env bash
# 假 cargo：test 一开跑就改要验的树（FAKE_EDITED_TREE）里的 lib.rs 与那一道门禁，再照此刻所在目录里的 lib.rs 报一条测试；别的子命令退 0
[[ "$1" == test ]] || exit 0
printf 'source_edited_mid_run\n' > "$FAKE_EDITED_TREE/crates/demo/src/lib.rs"
printf '%s\n' '#!/usr/bin/env bash' 'echo "  ✓ 崩溃状态块：现判 1、复用 0、拒算 0（键照主工作区改后的那一份门禁算）"' > "$FAKE_EDITED_TREE/.claude/gate.d/crash-point-replay.sh"
printf 'running 1 test\ntest tests::compiled_%s ... ok\n\ntest result: ok. 1 passed; 0 failed\n' "$(cat crates/demo/src/lib.rs)"
FAKE
  chmod +x "$edited_bin/cargo"
  out="$scratch/out-edited.txt"; checked=$((checked+1))
  PATH="$edited_bin:$PATH" FAKE_EDITED_TREE="$edited_tree" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-edited" \
    MERGE_AND_VERIFY_TREE="$edited_tree" env -u CARGO_TARGET_DIR -u MERGE_AND_VERIFY_VERIFICATION_STAGE bash "$0" --skip-apply "$scratch/edited-patch" > "$out" 2>&1; rc=$?
  if [[ $rc -ne 0 || "$(cat "$edited_tree/crates/demo/src/lib.rs" 2>/dev/null)" != source_edited_mid_run ]] \
     || ! grep -qxF 'test tests::compiled_source_at_start ... ok' "$scratch/logs-edited/test.log" 2>/dev/null \
     || ! cat "$scratch/logs-edited/"verification-*.log 2>/dev/null | grep -qF '键照 source_at_start 算，开跑时那一份门禁'; then
    echo "  ✗ 自检：快照：跑着的时候主工作区里那份源码被改，结论与复用键应当照开跑时那一份算（测试编 source_at_start、验证包那一步起快照里那一份门禁、键照 source_at_start 算），实际退出码 $rc，测试：$(grep '^test tests::' "$scratch/logs-edited/test.log" 2>/dev/null | tr '\n' ' ')，验证包那一步：$(cat "$scratch/logs-edited/"verification-*.log 2>/dev/null | grep '崩溃状态块' | head -1)"; failures=1; fi   # gate-lint:detail
  # ── 逐项点名与按条复用（文件头「逐项点名与按条复用」），点名的格都以「逐项与复用：」起头 ─────────
  #   样本工作区的 alpha 有三条测试函数（库里 tests::plain、tests::uses_helper，tests/beta.rs 里 beta_case），uses_helper 调 helper、spare 谁都不调；
  #   假 cargo 照收到的 --lib / --test / --doc 与 -- 之后的 --exact --skip 打 alpha 的几段（段头、结果行、汇总行带 finished in 0.00s），FAKE_UNITS_RED 点名的那一条打 FAILED。
  #   样本树下有 .unit-verdict-sample/（admission.py 不抽复用哨兵：条数钉死）。每一格各拷一份样本树、各自 git init（判定库在临时服务里按库目录分命名空间，几格互不串）；
  #   同一格里的几趟用同一个日志目录（快照每趟落在同一条路径上，判定库是同一份）
  local units_template="$scratch/units-template" units_bin="$scratch/bin-units" units_calls="" units_logs whole_done_line named_done_line affected_name reuse_check_output reuse_check_rc
  local keys_tree units_round
  mkdir -p "$units_template/crates/alpha/src" "$units_template/crates/alpha/tests" "$units_template/crates/singlefs-verification" "$units_template/.unit-verdict-sample" "$units_bin"
  printf '[workspace]\nmembers = ["crates/alpha", "crates/singlefs-verification"]\n' > "$units_template/Cargo.toml"
  printf '[package]\nname = "alpha"\nversion = "0.1.0"\nedition = "2021"\n' > "$units_template/crates/alpha/Cargo.toml"
  printf '[package]\nname = "singlefs-verification"\nversion = "0.1.0"\nedition = "2021"\n' > "$units_template/crates/singlefs-verification/Cargo.toml"
  cat > "$units_template/crates/alpha/src/lib.rs" <<'RUST'
pub fn helper() -> u32 {
    1
}

pub fn spare() -> u32 {
    7
}

#[cfg(test)]
mod tests {
    #[test]
    fn uses_helper() {
        assert!(super::helper() > 0);
    }

    #[test]
    fn plain() {}
}
RUST
  printf '#[test]\nfn beta_case() {}\n' > "$units_template/crates/alpha/tests/beta.rs"
  # 每一趟都带的补丁目录：只有变异行，点名 alpha 的库、测试目标 beta 与文档测试（挑测试目标照它挑出这三项，文件头「挑测试目标」）
  local units_patch="$scratch/units-patch"
  mkdir -p "$units_patch"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'alpha 的库（自证样本）' crates/alpha/src/lib.rs 1 2 '-p alpha --lib -- tests' uses_helper \
    'alpha 的 beta（自证样本）' crates/alpha/tests/beta.rs a b '-p alpha --test beta -- beta_case' beta_case 'alpha 的文档测试（自证样本）' crates/alpha/src/lib.rs 1 2 '-p alpha --doc' doc \
    > "$units_patch/mutations-append.tsv"
  cat > "$units_bin/cargo" <<'FAKE'
#!/usr/bin/env bash
# 假 cargo（按条复用那几格）：-V 打版本；test 照收到的 --lib / --test <名> / --doc（一个都没点就是全部）与 -- 之后的 --exact --skip <名> 打 alpha 的几段，
# FAKE_UNITS_RED=<libtest 全名> 那一条打 FAILED、退 101；别的子命令退 0。每次起都把收到的参数记进 FAKE_CARGO_CALLS（设了的话）
[[ -z "${FAKE_CARGO_CALLS:-}" ]] || printf '%s\n' "$*" >> "$FAKE_CARGO_CALLS"
case "$1" in
  -V) echo "cargo 0.0.0-merge-and-verify-selftest"; exit 0 ;;
  test) ;;
  *) exit 0 ;;
esac
lib=0; doc=0; tests=" "; skips=" "; selected=0; after=0; previous=""
for word in "$@"; do
  if ((after)); then [[ "$previous" == --skip ]] && skips+="$word "; previous="$word"; continue; fi
  case "$word" in
    --) after=1 ;;
    --lib) lib=1; selected=1 ;;
    --doc) doc=1; selected=1 ;;
  esac
  [[ "$previous" == --test ]] && { tests+="$word "; selected=1; }
  previous="$word"
done
((selected)) || { lib=1; doc=1; tests=" beta "; }
red=0
section() {   # <段头> <测试名…>
  local header="$1" name ran=0 filtered=0 failed=0
  shift
  printf '     %s\n\n' "$header"
  for name in "$@"; do if [[ "$skips" == *" $name "* ]]; then filtered=$((filtered + 1)); else ran=$((ran + 1)); fi; done
  echo "running $ran tests"
  for name in "$@"; do
    [[ "$skips" != *" $name "* ]] || continue
    if [[ "$name" == "${FAKE_UNITS_RED:-}" ]]; then echo "test $name ... FAILED"; failed=$((failed + 1)); red=1; else echo "test $name ... ok"; fi
  done
  printf '\ntest result: %s. %d passed; %d failed; 0 ignored; 0 measured; %d filtered out; finished in 0.00s\n\n' "$( ((failed)) && echo FAILED || echo ok)" $((ran - failed)) "$failed" "$filtered"
}
((lib == 0)) || section "Running unittests src/lib.rs (target/debug/deps/alpha-0123456789abcdef)" tests::plain tests::uses_helper
[[ "$tests" != *" beta "* ]] || section "Running tests/beta.rs (target/debug/deps/beta-0123456789abcdef)" beta_case
((doc == 0)) || section "Doc-tests alpha"
exit $((red ? 101 : 0))
FAKE
  cat > "$units_bin/rustc" <<'FAKE'
#!/usr/bin/env bash
# 假 rustc（按条复用那几格）：-V、-vV 打版本与 host（admission.py 的单元键要工具链那几行），别的退 0
case "${1:-}" in
  -V) echo "rustc 0.0.0-merge-and-verify-selftest" ;;
  -vV) printf '%s\n' "rustc 0.0.0-merge-and-verify-selftest" "host: x86_64-unknown-linux-gnu" "release: 0.0.0" ;;
esac
exit 0
FAKE
  chmod +x "$units_bin/cargo" "$units_bin/rustc"
  units_tree() { cp -r -- "$units_template" "$scratch/$1" && git init -q "$scratch/$1"; }   # <格名>：拷一份样本树、git init
  units_case() {   # <样本树> <日志目录名> <参数…>：起一趟（参数后面接上 units_patch），输出进 $out、退出码进 rc、假 cargo 收到的参数进 $units_calls
    local units_tree_root="$1" units_logs_name="$2"
    shift 2
    PATH="$units_bin:$PATH" FAKE_CARGO_CALLS="$units_calls" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/$units_logs_name" \
      MERGE_AND_VERIFY_TREE="$units_tree_root" env -u CARGO_TARGET_DIR bash "$0" "$@" "$units_patch" > "$out" 2>&1; rc=$?
  }
  # 列项：--list-items 逐行打「项<TAB>挑目标的参数」、退 0，不编不跑（假 cargo 一次都没起）、不建日志目录
  units_tree units-list
  out="$scratch/out-units-list.txt"; units_calls="$scratch/calls-units-list.txt"; : > "$units_calls"; checked=$((checked+1))
  units_case "$scratch/units-list" logs-units-list --list-items
  if [[ $rc -ne 0 || -s "$units_calls" || -e "$scratch/logs-units-list" ]] || ! grep -qxF $'alpha:src/lib.rs\t--lib' "$out" || ! grep -qxF $'alpha:tests/beta.rs\t--test beta' "$out" \
     || ! grep -qxF $'alpha:doc\t--doc' "$out" || [[ "$(grep -c . "$out")" -ne 3 ]]; then
    echo "  ✗ 自检：逐项与复用：--list-items 逐行列这一趟的项（alpha:src/lib.rs、alpha:tests/beta.rs、alpha:doc 三行）、退 0、假 cargo 一次都没起、不建日志目录，实际退出码 $rc，假 cargo 收到 $(grep -c . "$units_calls") 条：$(tr '\n' '|' < "$out" | cut -c1-300)"; failures=1; fi   # gate-lint:detail
  # 项写错：退 2、列出这一趟可点的项、一条测试都不起
  out="$scratch/out-units-unknown.txt"; units_calls="$scratch/calls-units-unknown.txt"; : > "$units_calls"; checked=$((checked+1))
  units_case "$scratch/units-list" logs-units-unknown --skip-apply --item alpha:tests/missing.rs
  if [[ $rc -ne 2 ]] || grep -q '^test ' "$units_calls" || ! grep -qF -- '--item 点名的项不在这一趟可点的项里：alpha:tests/missing.rs' "$out" || ! grep -qxF '       alpha:tests/beta.rs' "$out"; then
    echo "  ✗ 自检：逐项与复用：--item 写错的项应当退 2、列出可点的项、一条测试都不起，实际退出码 $rc，假 cargo 收到的测试命令 $(grep -c '^test ' "$units_calls") 条：$(grep -E '✗|alpha:' "$out" | head -4 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 复用三问（判法 scripts/gate/lib_reuse_checks.py three-questions）：同一份输入跑四趟——头一趟、同一份输入、改闭包外的 spare 之后、改闭包里的 helper 之后；
  #   每趟的逐条结局取 test-functions.tsv（名字与现判 / 复用），affected 是 uses_helper（只有它的闭包含 helper）。头两趟另核每项一行 ITEM_DONE
  units_tree units-reuse; units_calls="$scratch/calls-units-reuse.txt"; : > "$units_calls"
  local -a reuse_rounds_rc=()
  for units_round in first second unrelated unit-change; do
    case "$units_round" in
      unrelated) sed -i 's/^    7$/    8/' "$scratch/units-reuse/crates/alpha/src/lib.rs" ;;
      unit-change) sed -i 's/^    1$/    2/' "$scratch/units-reuse/crates/alpha/src/lib.rs" ;;
    esac
    out="$scratch/out-units-reuse-$units_round.txt"
    units_case "$scratch/units-reuse" logs-units-reuse --skip-apply
    reuse_rounds_rc+=("$rc")
    cut -f1,3 "$scratch/logs-units-reuse/test-functions.tsv" > "$scratch/units-reuse-$units_round.tsv" 2>/dev/null || : > "$scratch/units-reuse-$units_round.tsv"
  done
  affected_name="$(awk -F'\t' 'NF >= 7 && $6 == "tests::uses_helper" { print $1; exit }' "$scratch/logs-units-reuse/test-plan.alpha.out" 2>/dev/null)"
  checked=$((checked+1))
  reuse_check_rc=0
  reuse_check_output="$(python3 "$here/lib_reuse_checks.py" three-questions --script scripts/gate/merge-and-verify.sh --first "$scratch/units-reuse-first.tsv" \
    --second "$scratch/units-reuse-second.tsv" --unrelated "$scratch/units-reuse-unrelated.tsv" --unit-change "$scratch/units-reuse-unit-change.tsv" --affected "${affected_name:-tests::uses_helper}" 2>&1)" || reuse_check_rc=$?
  grep '^REUSE_SELFTEST ' <<<"$reuse_check_output"
  if [[ "${reuse_rounds_rc[*]}" != "0 0 0 0" || $reuse_check_rc -ne 0 || -z "$affected_name" ]]; then
    echo "  ✗ 自检：逐项与复用：reuse-three-questions（四趟退出码 ${reuse_rounds_rc[*]}、affected「${affected_name:-取不到}」；lib_reuse_checks.py three-questions 退 $reuse_check_rc：$(grep -E '✗|REUSE_SELFTEST' <<<"$reuse_check_output" | tr '\n' ' ' | cut -c1-400)）"; failures=1; fi   # gate-lint:detail
  checked=$((checked+1))
  if ! grep -qxF 'ITEM_DONE item=alpha:src/lib.rs seconds=0 judged=2 reused=0 exit=0' "$scratch/out-units-reuse-first.txt" \
     || ! grep -qxF 'ITEM_DONE item=alpha:tests/beta.rs seconds=0 judged=1 reused=0 exit=0' "$scratch/out-units-reuse-first.txt" \
     || ! grep -qxF 'ITEM_DONE item=alpha:doc seconds=0 judged=1 reused=0 exit=0' "$scratch/out-units-reuse-first.txt" \
     || ! grep -qxF 'ITEM_DONE item=alpha:src/lib.rs seconds=0 judged=0 reused=2 exit=0' "$scratch/out-units-reuse-second.txt" \
     || ! grep -qF '按条复用：现判 0、复用 3、拒算 0' "$scratch/out-units-reuse-second.txt"; then
    echo "  ✗ 自检：逐项与复用：整趟每项一行 ITEM_DONE（头一趟三项各现判、第二趟库里那两条复用），成功句报「按条复用：现判、复用、拒算」，实际头一趟 $(grep '^ITEM_DONE ' "$scratch/out-units-reuse-first.txt" | tr '\n' '|')，第二趟 $(grep -E '^ITEM_DONE |按条复用' "$scratch/out-units-reuse-second.txt" | tr '\n' '|' | cut -c1-400)"; failures=1; fi   # gate-lint:detail
  # 点名单跑一项与整趟里同一项判定相同：两份新拷的样本树（各自一份库）一趟整趟、一趟只点 alpha:tests/beta.rs；点名那一趟只起 --test beta、不跑库与文档测试，
  #   它的 ITEM_DONE 与整趟里那一项的逐字相同，beta_case 的结局相同
  units_tree units-whole; units_tree units-named
  out="$scratch/out-units-whole.txt"; units_calls="$scratch/calls-units-whole.txt"; : > "$units_calls"
  units_case "$scratch/units-whole" logs-units-whole --skip-apply
  whole_done_line="$(grep '^ITEM_DONE item=alpha:tests/beta.rs ' "$out")"
  out="$scratch/out-units-named.txt"; units_calls="$scratch/calls-units-named.txt"; : > "$units_calls"; checked=$((checked+1))
  units_case "$scratch/units-named" logs-units-named --skip-apply --item alpha:tests/beta.rs
  named_done_line="$(grep '^ITEM_DONE item=alpha:tests/beta.rs ' "$out")"
  if [[ $rc -ne 0 || -z "$named_done_line" || "$named_done_line" != "$whole_done_line" ]] || [[ "$(grep -c '^test ' "$units_calls")" -ne 1 ]] \
     || ! grep -qE '^test --offline --no-fail-fast -p alpha --test beta( |$)' "$units_calls" || grep -q '^ITEM_DONE item=alpha:src/lib.rs ' "$out" \
     || [[ "$(awk -F'\t' '$2 == "alpha:tests/beta.rs" { print $3, $4 }' "$scratch/logs-units-named/test-functions.tsv" 2>/dev/null)" \
           != "$(awk -F'\t' '$2 == "alpha:tests/beta.rs" { print $3, $4 }' "$scratch/logs-units-whole/test-functions.tsv" 2>/dev/null)" ]]; then
    echo "  ✗ 自检：逐项与复用：点名单跑 alpha:tests/beta.rs 应当只起 --test beta、它的 ITEM_DONE 与整趟里那一项相同，实际退出码 $rc，假 cargo 收到的测试命令 $(grep '^test ' "$units_calls" | tr '\n' '|')，点名「$named_done_line」、整趟「$whole_done_line」"; failures=1; fi   # gate-lint:detail
  # 记红撤绿：FAKE_UNITS_RED=tests::uses_helper 那一趟红（退 1、出路点名红的项 alpha:src/lib.rs），下一趟（假 cargo 不再打 FAILED、源码不动）
  #   uses_helper 照计划现判、原因以「上次判红」起头，结局绿
  units_tree units-red; units_calls="$scratch/calls-units-red.txt"; : > "$units_calls"
  out="$scratch/out-units-red.txt"; checked=$((checked+1))
  FAKE_UNITS_RED=tests::uses_helper units_case "$scratch/units-red" logs-units-red --skip-apply
  local red_round_rc="$rc"
  out="$scratch/out-units-red-next.txt"
  units_case "$scratch/units-red" logs-units-red --skip-apply
  if [[ "$red_round_rc" -ne 1 || $rc -ne 0 ]] || ! grep -qF "rerun-failed-tests.py $scratch/logs-units-red/test.log --item alpha:src/lib.rs" "$scratch/out-units-red.txt" \
     || ! awk -F'\t' 'NF >= 7 && $6 == "tests::uses_helper" && $2 == "judge" && index($7, "上次判红") == 1 { found = 1 } END { exit found ? 0 : 1 }' "$scratch/logs-units-red/test-plan.alpha.out" 2>/dev/null \
     || [[ "$(awk -F'\t' '$1 ~ /uses_helper$/ { print $3, $4 }' "$scratch/logs-units-red/test-functions.tsv" 2>/dev/null)" != "现判 绿" ]]; then
    echo "  ✗ 自检：逐项与复用：记红撤绿：上一趟判红的 uses_helper 下一趟应当照计划现判（原因「上次判红」）、判绿，红的那一趟退 1、出路点名 --item alpha:src/lib.rs，实际两趟退出码 $red_round_rc $rc，下一趟计划里那一条「$(awk -F'\t' 'NF >= 7 && $6 == "tests::uses_helper"' "$scratch/logs-units-red/test-plan.alpha.out" 2>/dev/null | cut -f2,7 | tr '\t' ' ')」"; failures=1; fi   # gate-lint:detail
  # 每项一行 ITEM_PLAN：整趟一次（留着快照，MERGE_AND_VERIFY_KEEP_COPY=1）记进库，再对判的那棵树 --list-items --with-keys：库与文档测试各项照计划报
  #   （没有快照时就是要验的树本身）
  units_tree units-keys; units_calls="$scratch/calls-units-keys.txt"; : > "$units_calls"
  out="$scratch/out-units-keys.txt"; checked=$((checked+1))
  MERGE_AND_VERIFY_KEEP_COPY=1 units_case "$scratch/units-keys" logs-units-keys --skip-apply
  local keys_round_rc="$rc"
  keys_tree="$scratch/logs-units-keys/repo"; [[ -d "$keys_tree" ]] || keys_tree="$scratch/units-keys"
  out="$scratch/out-units-keys-list.txt"
  units_case "$keys_tree" logs-units-keys-list --list-items --with-keys
  if [[ "$keys_round_rc" -ne 0 || $rc -ne 0 ]] || ! grep -qxF 'ITEM_PLAN item=alpha:src/lib.rs to_judge=0 reusable=2' "$out" \
     || ! grep -qxF 'ITEM_PLAN item=alpha:tests/beta.rs to_judge=0 reusable=1' "$out" || ! grep -qxF 'ITEM_PLAN item=alpha:doc to_judge=1 reusable=0' "$out"; then
    echo "  ✗ 自检：逐项与复用：--list-items --with-keys 每项一行 ITEM_PLAN（记过库之后 alpha:src/lib.rs 0 与 2、alpha:tests/beta.rs 0 与 1、alpha:doc 1 与 0），实际两趟退出码 $keys_round_rc $rc：$(grep -E '^ITEM_PLAN|✗|算不出' "$out" | tr '\n' '|' | cut -c1-400)"; failures=1; fi   # gate-lint:detail
  # 按条复用失灵：样本树的 .unit-verdict-sample/plan-failure 写 exit-1（admission.py test-plan 不算键、退 1）：测试那一步照旧全部现判跑完、判红退 1
  units_tree units-plan-failure; printf 'exit-1\n' > "$scratch/units-plan-failure/.unit-verdict-sample/plan-failure"
  units_calls="$scratch/calls-units-plan-failure.txt"; : > "$units_calls"
  out="$scratch/out-units-plan-failure.txt"; checked=$((checked+1))
  units_case "$scratch/units-plan-failure" logs-units-plan-failure --skip-apply
  if [[ $rc -ne 1 ]] || ! grep -qF '✗ 测试那一步按条复用失灵' "$out" || ! grep -qE '^test --offline --no-fail-fast -p alpha( |$)' "$units_calls" || grep -q -- '--skip' "$units_calls"; then
    echo "  ✗ 自检：逐项与复用：按条复用失灵（test-plan 退 1）应当判红、退 1，测试照旧全部现判跑完（不带 --skip），实际退出码 $rc，假 cargo 收到的测试命令 $(grep '^test ' "$units_calls" | tr '\n' '|')：$(grep -E '✗' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # prune-server 不可用：plan-failure 写 exit-75（test-plan 不连服务、退 75）：测试那一步不起 cargo、报不可用，整条退 75
  units_tree units-unavailable; printf 'exit-75\n' > "$scratch/units-unavailable/.unit-verdict-sample/plan-failure"
  units_calls="$scratch/calls-units-unavailable.txt"; : > "$units_calls"
  out="$scratch/out-units-unavailable.txt"; checked=$((checked+1))
  units_case "$scratch/units-unavailable" logs-units-unavailable --skip-apply
  if [[ $rc -ne 75 ]] || grep -q '^test ' "$units_calls" || ! grep -qF '测试本次未跑：按条复用那一层连不上 prune-server' "$out"; then
    echo "  ✗ 自检：逐项与复用：prune-server 不可用（test-plan 退 75）应当退 75、测试那一步不起 cargo，实际退出码 $rc，假 cargo 收到的测试命令 $(grep -c '^test ' "$units_calls") 条：$(grep -E '✗|测试' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  rm -rf "${scratch:?}"
  if [[ $failures -ne 0 ]]; then
    echo "    → 怎么办：看 verify() 抓 FAILED 行与退出码、拷快照（take_snapshot）与收尾（finish）那几处；MERGE_AND_VERIFY_BREAK 设着的话这里本来就该红"
    return 1
  fi
  echo "  ✓ 自检通过（查了 $checked 种）：测试红时退 1 并列出红的与重跑命令、补丁核不过整批停退 2、全绿退 0、没设日志目录时日志落在工作根下、"
  echo "    往编过的副本里 rsync -a 铺旧 mtime 的新内容之后照样重编、给了认不出的选项退 2 一样都不做、"
  echo "    clippy 照 check.sh 的 ${#expected_combinations[@]} 个特性组合（${expected_combinations[*]-}）逐个跑、每个带上 check.sh 的 $expected_lint_count 条编码纪律 lint、一个组合红整步红并点名那个组合、"
  echo "    跑 build、cargo test 只起补丁改到的 demo 的库目标、测试那一步经 test-targets-parallel.py 按测试目标并行起、"
  echo "    四条 cargo 命令从 check.sh 现读（只换包装 run_cargo 与 cargo 来回换、各加一个标志都照读到的写法跑）、"
  echo "    check.sh 读不出 lint 表、clippy 那一行没带 lint 表、fmt 那一条换成认不出的包装、test 那一行带收窄目标的标志、缺省特性组合只剩一个、循环改成只取一个组合都退 1 不编、要验的树里有特性组合登记退 1 不编、"
  echo "    成员里没有 验证包退 1 不编、一条测试都没跑退 1、"
  echo "    开跑时拷快照、在快照里打补丁与编、快照根 git init、快照里的修改时刻是拷的那一刻、全绿与测试红收尾都删掉快照与编译目录、MERGE_AND_VERIFY_KEEP_COPY=1 留着并打出路径、"
  echo "    跑着的时候主工作区里那份源码被改、测试与 验证包那一步的键照开跑时那一份算、日志目录在要验的树里面退 2 不拷、"
  echo "    补丁改过的包头一次编与再验同一批补丁都有 Compiling 行、编译目录里的产物比源码新时点名没重编的包退 1、"
  echo "    只跑补丁改到的与变异行点名的测试目标（不带依赖它们的包、不跑包里没点到的目标）、判不出测试目标的四种（没给补丁目录、改 Cargo.lock、改不落在包里的路径、改到落不到目标的文件）测试不跑退 3、--all-packages 退 2、"
  echo "    --skip-apply 时补丁反着核得过才往下验、没打在树上的退 1 不编、"
  echo "    跑完运行目录里不剩它建的编译目录与快照（全绿、测试红、被 TERM 停三种结局都删，调用方设的 CARGO_TARGET_DIR 不删）、编译目录里还有进程开着文件时不删并报出那个进程、"
  echo "    编译目录建在 TMPDIR 下这一趟新建的一份、不在工作根下（日志照旧在工作根下）、收尾时前面一步写满盘编译目录照删（EXIT trap 第一步就删它）、"
  echo "    验证包那一步只把跑到的验证包目标里登记的崩溃枚举用例交按块那一路、挂特性的文件交 feature-gated 点名、产品 GPU 包的真卡对拍交那一格，不起整格、红了退 1 并只重跑红的那一项（经 task-server/task submit 交）、"
  echo "    跑到的目标里没有要另走门禁的就不起那一道、那一道退 3 只在样本根上算过、--list-items 退非 0 判红、跑在第二台上时照起那一道、"
  echo "    给了 --crash-expand-up-to（或环境里已设展开深度变量）把变量交给 验证包那一步、汇总写缩过域、不算门禁通过，缩过域的那一趟红了出路里只重跑的命令带同一个变量，值不是非负整数退 2 一样都不做、"
  echo "    测试那一步的 cargo test 在 -- 之后什么都不带（标 #[ignore] 的硬件与量数用例不起）、成功句不报标 #[ignore] 的条数、"
  echo "    逐项与复用：--list-items 列项不编不跑、--item 写错退 2、点名单跑一项与整趟里同一项判定相同、每条测试函数按单元键复用（reuse-three-questions 三问都过）、每项一行 ITEM_DONE 与 ITEM_PLAN、"
  echo "    判红的照同一把键记红下一趟现判、按条复用失灵判红、prune-server 不可用退 75 不起 cargo"
  return 0
}

if [[ "${1:-}" == "--selftest" ]]; then selftest; exit $?; fi
THREADS="$(heavy_thread_default)" || exit 2; skip_apply=0; dirs=(); crash_expand_up_to_argument=""; crash_expand_up_to_given=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-apply) skip_apply=1; shift ;;
    --crash-expand-up-to)
      # 文件头「验证包那一步」：缩崩溃域的展开深度上限，值是非负整数，下面统一核
      [[ $# -ge 2 ]] || die "--crash-expand-up-to 后面要跟一个非负整数（展开深度上限，只往小缩），一样都没做" "写成 --crash-expand-up-to 6 这样的两个词"
      crash_expand_up_to_argument="$2"; crash_expand_up_to_given=1; shift 2 ;;
    # 逐项点名（文件头「逐项点名与按条复用」）
    --list-items) LIST_ITEMS=1; shift ;;
    --with-keys) LIST_WITH_KEYS=1; shift ;;
    --item)
      [[ $# -ge 2 && -n "$2" && "$2" != -* ]] || die "--item 后面要跟一项（<包>:<目标根文件> 或 <包>:doc），一样都没做" "写成 --item singlefs-core:src/lib.rs 这样的两个词；可点的项用 --list-items 看"
      NAMED_ITEMS+=("$2"); shift 2 ;;
    --item=*)
      [[ -n "${1#--item=}" ]] || die "--item= 后面要跟一项（<包>:<目标根文件> 或 <包>:doc），一样都没做" "写成 --item=singlefs-core:src/lib.rs；可点的项用 --list-items 看"
      NAMED_ITEMS+=("${1#--item=}"); shift ;;
    --*)
      # 弄坏开关 unknown-option-accepted：认不出的选项也照收、当没给（不拒）
      if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "unknown-option-accepted" ]]; then shift; continue; fi
      die "认不出参数 $1" "只认 --skip-apply、--crash-expand-up-to <n>、--item <项>、--list-items [--with-keys]（整包跑归提交时的整轮门禁；提交之外要整包跑先弹窗问用户）" ;;
    *) dirs+=("$1"); shift ;;
  esac
done
((LIST_WITH_KEYS == 0 || LIST_ITEMS)) || die "--with-keys 只跟 --list-items 一起用，一样都没做" "写成 --list-items --with-keys [同一趟的别的参数…]"
# 缩崩溃域（文件头「验证包那一步」）：参数给了取参数，没给取环境里已设的变量；值要是非负整数（弄坏开关 expand-up-to-unchecked：不核），写坏退 2、一样都不做
if ((crash_expand_up_to_given)); then CRASH_EXPAND_UP_TO="$crash_expand_up_to_argument"; crash_expand_up_to_origin="--crash-expand-up-to"
elif [[ -n "${SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO+x}" ]]; then CRASH_EXPAND_UP_TO="$SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO"; crash_expand_up_to_origin="环境变量 $CRASH_EXPAND_UP_TO_VARIABLE"; crash_expand_up_to_given=1
fi
if ((crash_expand_up_to_given)) && [[ ! "$CRASH_EXPAND_UP_TO" =~ ^[0-9]+$ ]] && [[ "${MERGE_AND_VERIFY_BREAK:-}" != "expand-up-to-unchecked" ]]; then
  die "$crash_expand_up_to_origin 的值「$CRASH_EXPAND_UP_TO」不是非负整数（展开深度上限，只往小缩），一样都没做" "写成 --crash-expand-up-to 6 这样；不缩就别给，也别在环境里设 $CRASH_EXPAND_UP_TO_VARIABLE"
fi
SOURCE_TREE="$(cd "${MERGE_AND_VERIFY_TREE:-$root}" && pwd)" || die "进不去要验的树 ${MERGE_AND_VERIFY_TREE:-$root}" "在仓里跑；MERGE_AND_VERIFY_TREE 只给自证用，平时不设"
tree="$SOURCE_TREE"
# 补丁目录转成绝对路径（之后进快照里干活，相对路径就指错了）；算改过的包也用这一份（文件头「改过的包都重编了没有」，--skip-apply 时也留着）
absolute_dirs=()
for dir in ${dirs[@]+"${dirs[@]}"}; do
  if [[ -d "$dir" ]]; then absolute_dirs+=("$(cd "$dir" && pwd)"); else absolute_dirs+=("$dir"); fi
done
dirs=(${absolute_dirs[@]+"${absolute_dirs[@]}"}); CHECK_PATCH_DIRS=(${absolute_dirs[@]+"${absolute_dirs[@]}"})
# --list-items（文件头「逐项点名与按条复用」）：不建日志目录、不拷快照、不编，列完就退
((LIST_ITEMS == 0)) || list_items_and_exit
[[ ${#dirs[@]} -gt 0 || $skip_apply -eq 1 ]] || die "没有给补丁目录" "task-server/task submit -- bash scripts/gate/merge-and-verify.sh <补丁目录> …；只验不打用 --skip-apply"
if [[ -n "${MERGE_AND_VERIFY_LOG_DIRECTORY:-}" ]]; then logs="$MERGE_AND_VERIFY_LOG_DIRECTORY"
elif work_root="$(python3 "$root/.claude/hooks/lib_work_root.py")"; then logs="$work_root/merge-and-verify/$(TZ=Asia/Tokyo date +%F)-$$"
else die "取不到工作根（.claude/hooks/lib_work_root.py 跑不起来）" "恢复那份库，或设 MERGE_AND_VERIFY_LOG_DIRECTORY 指到日志目录"; fi
# 这一趟的编译目录（文件头）与快照：是这里建的才登记进 RUN_TARGET / SNAPSHOT_TREE，收尾的 EXIT trap 只删登记了的
[[ ! -e "$logs/repo" ]] || die "快照位置 $logs/repo 已经有东西，不往里铺、也不删它" "换一个日志目录（MERGE_AND_VERIFY_LOG_DIRECTORY 不设就是新的 <日期>-<进程号>），或先看清那里是什么"
mkdir -p "$logs" && logs="$(cd "$logs" && pwd)" || die "建不了日志目录 $logs" "看工作根在不在、可不可写"
[[ "$(cd "$logs" && pwd -P)/" != "$(cd "$SOURCE_TREE" && pwd -P)/"* ]] || die "日志目录 $logs 在要验的树 $SOURCE_TREE 里面：快照会把自己拷进自己，什么都没拷" \
  "MERGE_AND_VERIFY_LOG_DIRECTORY 指到树外（不设就是工作根下的 merge-and-verify/<日期>-<进程号>）"
trap finish EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
# 快照（文件头「快照」）：开跑这一刻拷，之后打补丁、编、验都在它上面。弄坏开关 verify-in-main-tree：不拷、都在要验的树上
if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "verify-in-main-tree" ]]; then
  echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=verify-in-main-tree：不拷快照，打补丁、编、验都在要验的树 $tree 上"
else
  SNAPSHOT_TREE="$logs/repo"
  take_snapshot "$SOURCE_TREE" "$SNAPSHOT_TREE" > "$logs/snapshot.log" 2>&1 \
    || die "拷快照没成（日志 $logs/snapshot.log），一个补丁都没打、没编" "照那份日志修（rsync 在不在、工作根盘满没满）再跑；拷了一半的快照收尾照删" 1
  # 快照根 git init（文件头「快照」）：判定库按 <git common-dir>/<库名> 取名，不是 git 工作树的读写判定库的脚本退 2。弄坏开关 copy-not-git-initialized：不 init
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" != "copy-not-git-initialized" ]]; then
    git init -q "$SNAPSHOT_TREE" > "$logs/snapshot-git-init.log" 2>&1 \
      || die "快照根 git init 没成（日志 $logs/snapshot-git-init.log），一个补丁都没打、没编" "照那份日志修（git 在不在、快照可不可写）再跑；拷了的快照收尾照删" 1
  fi
  echo "  拷好快照：$SOURCE_TREE → $SNAPSHOT_TREE（$(du -sh -- "$SNAPSHOT_TREE" 2>/dev/null | cut -f1)；不带 .git 与编译目录，修改时刻是此刻，收尾删）"
  tree="$SNAPSHOT_TREE"
fi
# 验证包那一步（文件头「验证包那一步」）：起快照里那一份门禁，出路那条命令写主工作区那一份
VERIFICATION_STAGE="${MERGE_AND_VERIFY_VERIFICATION_STAGE:-$tree/$VERIFICATION_STAGE_PATH}"
VERIFICATION_RERUN_STAGE="${MERGE_AND_VERIFY_VERIFICATION_STAGE:-$root/$VERIFICATION_STAGE_PATH}"
if [[ -z "${CARGO_TARGET_DIR:-}" ]]; then
  # 编译目录放这一台自己的临时目录下、这一趟新建（文件头「这一趟的编译目录」），不放日志目录：日志目录在工作根下，工作根两台共享（第二台经 NFS 写回本机盘）
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "target-under-work-root" ]]; then   # 弄坏开关：建在 <日志目录>/target
    RUN_TARGET="$logs/target"
    mkdir -- "$RUN_TARGET" || die "建不了这一趟的编译目录 $RUN_TARGET" "看日志目录 $logs 可不可写、盘满没满"
  else
    RUN_TARGET="$(mktemp -d "${TMPDIR:-/tmp}/merge-and-verify-target.XXXXXXXX")" && RUN_TARGET="$(cd "$RUN_TARGET" && pwd -P)" \
      || die "在 ${TMPDIR:-/tmp} 下建不了这一趟的编译目录" "看 ${TMPDIR:-/tmp} 在不在、可不可写、盘满没满；要用现成的编译目录就设 CARGO_TARGET_DIR（那一份收尾不删）"
  fi
  CARGO_TARGET_DIR="$RUN_TARGET"; export CARGO_TARGET_DIR
  echo "  这一趟的编译目录：$RUN_TARGET（这里建的，收尾删）"
else
  echo "  编译目录：$CARGO_TARGET_DIR（调用方设的 CARGO_TARGET_DIR，收尾不删）"
fi
cd "$tree" || die "进不去要编的树 $tree" "看快照 $tree 在不在（拷完就进，进不去多半是盘满或被删）" 1
if [[ $skip_apply -eq 1 ]]; then dirs=(); check_patches_applied "$logs"; fi
verify "$logs" ${dirs[@]+"${dirs[@]}"}
