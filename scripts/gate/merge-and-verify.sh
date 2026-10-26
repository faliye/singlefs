#!/usr/bin/env bash
# admission: always 每一批补丁都要打一次、验一次，上一次的结论不替这一次作保
# run-condition: command git cargo
# 一批实现员交回之后的「合入后验证」：打全部补丁、编一次、跑补丁改到的包（与依赖它们的包）的 harness 档测试二进制、fmt、clippy 与 build，
# 再跑 checker 档快档里单元键对不上的测试函数（下面「checker 档快档」），红的列出来。替掉派一个实现员做这件活。
#
#   merge-and-verify.sh [--memory <上限>] [--checker-quick-memory <上限>] [--threads <N>] [--skip-apply] [--copy] [--all-packages] [--peer] <补丁目录> …
#     --threads 缺省取登记的重活默认并行度（.claude/heavy-thread-default，scripts/hosts/heavy-threads.sh），不看忙闲。
#     --checker-quick-memory checker 档快档那一步单独的内存上限（下面「checker 档快档」的「内存上限」一段）；没给取峰值表里这一步的近几次峰值乘 1.5，表里没有取推的 36G，都不比 --memory 小。
#     --all-packages 测试步照旧跑工作区成员减掉 checker 档包的全部包，不按补丁挑。
#     --peer 编译与测试交给第二台（下面「--peer」一段）。
#   merge-and-verify.sh --selftest                     # 用假的 apply-writer-patch 与假 cargo 走一遍；MERGE_AND_VERIFY_BREAK=ignore-red 时必须判红
#
# --copy：不往这份脚本所在的仓里打，自己建一份副本在副本里打、编、验，收尾删掉它自己建的副本与编译目录（成功、失败、被 INT / TERM 停都删）：
#   副本 <日志目录>/repo（scripts/hosts/source-stamp.py sync 拷：不拷 .git 与任意一层的 target、target-*，拷过去的 mtime 是此刻），
#   补丁经 apply-writer-patch.py --root <副本> 打；编译目录照下面「这一趟的编译目录」。
#   日志目录留着（日志小，汇总行指着它）。删完打一行「删了副本 …（共 …）」；删不成退 1、写明哪一份没删成。
#   要留现场：MERGE_AND_VERIFY_KEEP_COPY=1，收尾不删，打出副本与编译目录的路径和删法，看完自己删（command-safety.md「测试镜像一律放临时目录」：
#   要留着看现场由显式的开关打开、留下时打印路径）。不带 --copy 时照旧打进这份脚本所在的仓（合入主工作区就是这一种），不建副本。
#   副本不带 .git，按 git common-dir 找的两样在副本上取不到（records/2026-10-23-subagent拆分提案.md 第八十二节 ①②）：建副本之前从要验的树（MERGE_AND_VERIFY_TREE，
#   默认这份脚本所在的仓）取，打一行、写进 <日志目录>/copy-environment（一行一条「名字=值」，值经 printf %q，可以 source）：
#     GATE_MUTATION_RECORDS_HOME=<git common-dir>：在副本上起门禁 checker-tier-crates-mutation-replay 的轻型变异要带它（判定库在 git common-dir 下，
#       scripts/mutation/crates-mutation-rows.py 不设就取被判那棵树的 git common-dir，副本上取不到就一条都不跑）；
#     SINGLEFS_MULTI_HOST_CONFIG=<多机配置的绝对路径>：调用方设了照它（相对路径按起它时的当前目录转成绝对路径），没设取 git common-dir 上一层下的 multi-host.env
#       （找法同 scripts/crash/crash-amplification-lib.sh 的 crash_amplification_multi_host_configuration_file）；带 --peer 时交给第二台那一趟的 multi-host-run.sh
#       （副本里的那一份按副本的 git common-dir 找配置，找不到就退 255）。
#   要验的树不是 git 工作树（取不到 git common-dir）时 GATE_MUTATION_RECORDS_HOME 写不出、多机配置只在调用方设了时写：打一行「算不出」，不记红（打补丁与验证不靠它们）。
# 这一趟的编译目录（用户 2026-10-23 定案「自己用完自己删」，不留给以后的趟复用）：带不带 --copy、带不带 --peer 都一样，调用方没设 CARGO_TARGET_DIR 时
#   建 <日志目录>/target、export 成 CARGO_TARGET_DIR，四条 cargo 与源码戳都用它；正常结束、红了退出、被 INT / TERM 停都在 EXIT trap 里删掉
#   （scripts/hosts/run_build_directory.py remove：删之前核本机没有进程开着里面的文件，有就是这一趟的子进程没收干净，不删、逐个报进程、退 1）。
#   <日志目录>/target 已经有东西：不往里编也不删它，退 2。调用方设了 CARGO_TARGET_DIR 就照它编、收尾不删（不是这里建的，归调用方删）。
#   --peer 时第二台那一侧不交共用编译目录：multi-host-run.sh --peer 在第二台这一趟的树里编，⑦ 连同树里的编译目录一起删（它文件头 ⑤ ⑦）。
#
# 为什么：2026-09-20 起 8 天里「合入后验证」派了 3 个实现员、平均 14.3M 折算，是实现员里最贵的一种；它做的前三步（打补丁、编、逐个跑测试）
# 是 apply-writer-patch.py 加几条命令，只有「红了改钉值」要判断——那一步红了才派实现员。
# 做法：每个补丁目录经 scripts/gate/apply-writer-patch.py 打（退 2 就整批停）；编之前经 scripts/hosts/source-stamp.py refresh 刷一次源码戳：
# 调用方给的编译目录（CARGO_TARGET_DIR）可能是编过的，cargo 按 mtime 判本地包新不新鲜，被 rsync -a 铺进来的新内容带着旧 mtime 会被当成已编
# （records/2026-09-30-里程碑三开工第一轮.md「GPU 与 CPU 不一致调查员交回」那一行）；refresh 把内容或 mtime 与上一次不一样的源文件 touch 到此刻，
# 编译目录里有东西而没有戳（头一次）就全 touch、重编一遍工作区的本地包。刷不成就不编、退 1。手工往副本里拷树用 source-stamp.py sync <源树> <副本>。
# 然后经 run-with-memory-cap.sh 与 capped.sh 跑 test、fmt、clippy、build 四条 cargo，与上游 .claude/singlefs-ai-sop/scripts/check.sh 同一套判据：
#   test   `cargo test --offline --no-fail-fast -p <包>…`：包是工作区成员（树根 Cargo.toml 的 [workspace] members，逐个读成员 Cargo.toml 的包名）
#          减掉 checker 档包 singlefs-checker-tier（它的快档测试函数归下面「checker 档快档」那一步，全量归提交时的主 agent）；check.sh 跑的是 `cargo test --all`，差的只有这一个包。
#          成员里没有 singlefs-checker-tier（改了名、挪了）、成员写成通配、读不出包名，都判红、不编：排不掉它就宁可不跑。
#          退 0 而 test result 行里一条测试都没数到，也判红（check.sh「一条测试都没跑」那一条）。
#          测试只跑改到的包：上面那几个包里只留这一批补丁改到的包与工作区里依赖它们的包（成员 Cargo.toml 的 [dependencies]、[dev-dependencies]、
#          [build-dependencies] 与 [target.*] 下的这三张表里写了它，一层层往上传）。改到的包按给的补丁目录里 crates.patch 改过的路径算（路径往上找到的
#          第一份带 [package] 的 Cargo.toml，scripts/gate/lib_rebuilt_packages.py 的 package_of，与重编核对同一份；包里改过的不论是不是 .rs 都算），
#          --skip-apply 时照样读给的补丁目录。没挑上的包逐个报「本次未跑：补丁没改到」，成功句里逐个列名；一个包都没挑上（补丁只改了 checker 档包、
#          或没有 crates.patch）时测试这一步不跑、报「测试本次未跑」，不记红也不算测过。照旧全跑的：带 --all-packages；没给补丁目录；
#          补丁改到工作区根的 Cargo.toml、Cargo.lock 或 .cargo/；补丁改到不落在任何包里的路径（测试运行时可能读它，判不出归哪个包）。
#          harness 档按测试函数单元键入库复用（只跑键对不上的测试）归 .claude/kb/decisions/13-验证路线.md 已定项 17 的下一步，这里还是按包挑。
#          挑好的包不交给一条 cargo test 一个测试二进制接一个地跑（用例少的二进制跑的时候核就空着：2026-10-21 第二台只用了约 5 核、登记 31，
#          records/2026-10-18-里程碑三收尾与近期欠账.md 第 780 行），交给 scripts/gate/test-targets-parallel.py --jobs <线程上限>：先 --no-run 一次编好，
#          再按测试目标各起一条 cargo test -p <包> <--lib / --test 名 / --bin 名>、同时至多线程上限那么多条，每条的 RUST_TEST_THREADS 分摊，
#          各目标的输出按派活的次序回读进 test.log，派出与收回的条数对不上判红；文档测试最后一条 cargo test --doc 一起跑。跑的用例与 cargo test -p … 同一批，
#          FAILED 行、test result 行与 Compiling 行照旧落进 test.log，下面的判法不变。
#          慢档一起跑（用户 2026-10-26 弹窗定「整批慢档都带上」，拆分提案第五十一节 ①② 与第六十三节 ①）：这一步交给测试二进制的参数是 --include-ignored，
#          挑上的包里标 #[ignore] 的 harness 慢档用例与快档一起跑（改了 IMPLEMENTED_INVARIANTS 时要跑的那几个逐条循环判定用例就在里面，不另列）；
#          checker 档包照旧不进这一步。成功句报慢档跑了几条：test-targets-parallel.py 末行的 slow=（它文件头 ⑤：逐个目标先 --list --ignored 数一次）；
#          一条 cargo test 逐个二进制跑的那一种（自证与 --peer 的假驱动）数不出，写「条数没数」。慢档整批的挂钟没量过（推的：单条 60 秒以上，.claude/rules/verification.md）。
#          只剩没跑完的几条与文档测试那一段，它往 SINGLEFS_SERIAL_DECLARATIONS
#          声明串行（本机跑时这里把它设成 <日志目录>/serial-declarations，调用方设了照调用方的；看门狗按这个进程整棵树合计判核数，见 scripts/agents/agent-watch.py
#          文件头「长跑在用核数过少」）。--peer 时第二台照同一条起（副本里的 scripts/gate/test-targets-parallel.py）。
#          MERGE_AND_VERIFY_TEST_TARGETS=serial（只给自证）退回一条 cargo test --offline --no-fail-fast -p …；自证的别的格都这么设，只有「按测试目标并行起」那一格不设。
#   fmt    `cargo fmt --all -- --check`；clippy `cargo clippy --offline --all-targets --all-features -- -D warnings` 加编码纪律那张 lint 表；build `cargo build --offline --all-targets`。
#   lint 表只在 check.sh 一处定义：每次开跑从它的 `CODE_DISCIPLINE_LINTS=(` … `)` 那一段现读（一条一对 `-D clippy::<名>`），不在这里另抄一份；
#   同时核 check.sh 里 fmt、clippy、build、test 四条 cargo 的写法（CHECK_SH_COMMANDS）还是这里照着对齐的那一种。读不出表、表的形状不对、
#   哪一条写法变了，都判红、一个补丁都不打：上游改了判据，先回来对齐这里。
# 日志落 $MERGE_AND_VERIFY_LOG_DIRECTORY（默认工作根下的 merge-and-verify/<日期>-<进程号>，工作根只在 .claude/hooks/lib_work_root.py 里定义）。
# cargo test 那一趟内存包装退 245（显存不够，run-with-memory-cap.sh 文件头「退出码」）：这一趟的测试结果（连同 FAILED 行）不算数，记一项红、
#   报「显存不够」、不列 FAILED 也不给只重跑它们的命令（不是代码红），出路是看谁在用卡、等它跑完重跑这一条。退 250–254 照旧报包装自己的结局。
# 改过的包都重编了没有（.claude/kb/milestone/03-third-txn.md「待优化：流程、门禁与装置上踩过的坑」第 1 行；判法在 scripts/gate/lib_rebuilt_packages.py，这里不另写）：
#   改过的包从给的补丁目录里的 crates.patch 算（--skip-apply 时照样读给的补丁目录，只是不打；打没打先核，见「--skip-apply 先核补丁打过没有」）。编之前（刷源码戳之前）把补丁改过、落在包里的文件刷到此刻，
#   编完核 test 与 build 两趟的编译输出：每个改过的包都要有「Compiling <包名> v…」行，缺一个记一项红，出路是刷源码修改时刻或换干净编译目录整趟重跑。
#   只核包里改过的 .rs；包里改过的 .rs 一个都不在编译目录的 dep-info 里的包不核（这几趟编译不读它们，例如 checker 档包 gpu、verdict-store 下才编的模块）。
#   没给补丁目录、或补丁里没有要核的包：报「重编核对本次未判」，不记红也不算核过。
# --skip-apply 先核补丁打过没有：拿锁、进树之后，编之前，对给的每个补丁目录里的 crates.patch 在要验的树上跑 `git apply --check --reverse`
#   （反着打得上才算这份已经打在树上）；有一份核不过、或给的补丁目录不在，就退 1、逐份点名、一个都不编，出路是没打过的那几份去掉 --skip-apply 让这里打。
#   没有 crates.patch 的目录（只有 mutations-append.tsv 的）不核，报「本次未核」；不带 --skip-apply 时不核（这里自己打）。
#   罩不住的：同一批里两份补丁改到同一处、后一份又改了前一份改过的行时，前一份反着核不过，会误判红；判红的说明里写明这种可能，出路是在仓副本里倒序逐份 git apply --reverse 人工核。
# 主工作区合入锁（同一张表第 20 行；判法与锁文件在 scripts/gate/lib_workspace_lock.py 的文件头，这里不另写）：不带 --copy 时，打补丁与编之前先拿锁，
#   持有者是这个进程（$$），被别的合入占着就排队、每过一段时间报一行谁占着，持有者已死的判失效、报出来再接管；退出时（成功、红、被 INT / TERM 停）放掉。
#   锁按工作树分开：在 git worktree 加出来的树里跑时拿那棵树自己那一把，不排在主工作树的合入后面。
#   拿着锁时 export SINGLEFS_WORKSPACE_LOCK_HOLDER=$$，它起的 apply-writer-patch.py 认出是在持有者之下、不另拿。拿不成退 1、一个补丁都没打、没编。
#   带 --copy 时在自己的副本里打与编，不拿这一把。
# MERGE_AND_VERIFY_TREE（只给自证）：刷源码戳与编译在这棵树里（默认这份脚本所在的仓根）；打补丁照旧是 apply-writer-patch.py 自己的仓根。
# MERGE_AND_VERIFY_CHECK_SH（只给自证）：从哪一份 check.sh 读 lint 表与四条写法（默认这份脚本所在仓的 .claude/singlefs-ai-sop/scripts/check.sh）。
# --peer（用户 2026-10-20「第二台机器不能跑吗」「去找一下可用的电脑」，records/2026-09-16-subagent拆分提案.md 第七十五节）：打补丁、刷源码戳、拿合入锁、挑包照旧在本机
#   （不带 --copy 时就是主工作区），test、fmt、clippy、build 四条 cargo 一趟交给 scripts/hosts/multi-host-run.sh --peer：它把要验的树拷到第二台，
#   在那边的副本里逐步经副本里的 run-with-memory-cap.sh <上限> 与 capped.sh <N> 跑（四条命令与本机跑的是同一份数组），每步的输出与退出码写进副本里的
#   PEER_LOG_DIRECTORY，--fetch 取回 <日志目录>/peer-fetched/ 下；再拷成 <日志目录>/<步>.log，判法、出路与汇总同本机。multi-host-run.sh 的输出落 <日志目录>/peer.log。
#   第二台的编译目录是第二台这一趟那棵副本里的 target（不交 MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY，不跨趟复用），multi-host-run.sh 的 ⑦ 跑完连同副本删掉。
#   上限 --memory 不能比第二台配置的 PEER_MEMORY_CAP 大（multi-host-run.sh 拒、退 2）。第二台那一趟没跑成（multi-host-run.sh 退非 0）判红、退 1、一步都不判。
#   第二台的 rustfmt 与 clippy（2026-10-21 第二台 1.98.0 工具链没装这两个组件，fmt、clippy 两步退「is not installed for the toolchain」、被记成红）：
#   第二台那一趟在 fmt、clippy 两步各自之前先在副本的树根里问一句 cargo fmt --version、cargo clippy --version（PEER_STEPS_SCRIPT），答不出就不起这一步：那一步的日志写
#   「本次未跑」与 --version 的原话，留一个 <步>.not-run 标记，退出码文件写 77；本机这一侧看到标记就打「<步> 本次未跑：第二台的工具链没装 <组件> 组件」，出路写
#   rustup component add rustfmt clippy（原话里点名了工具链的带 --toolchain <名>）与「或去掉 --peer 在本机跑」，不记红，汇总行末尾写「本次未跑 N 步」。测试与 build 照跑照判。
#   选「本次未跑」而不是把 fmt、clippy 留在本机跑：clippy 要把整个工作区再编一遍，正是本机内存排不上才改去第二台的那种活，留在本机还是会排不上（退 252），
#   而 fmt、clippy 的结论随工具链版本变，两台各跑一半，结论口径分不清（与「不自动改去第二台」同一条理由）。「本次未跑」不记成绿也不记成红，汇总行里点名，由人装好组件重跑。
#   改过的包重编了没有不核（编译目录在第二台），报「重编核对本次未判」。不带 --peer 时不自动改去第二台：两台的结论口径分不清的时候不混用，
#   本机排不上时只在出路里写带 --peer 重跑的整条命令（下一段）。MERGE_AND_VERIFY_MULTI_HOST_RUN 换掉 multi-host-run.sh（只给自证）。
# 本机排不上（内存包装退 252：排满等待上限、或等多久都放不下）：那一步记一项红，打包装现查别的机器的那一行（run-with-memory-cap.sh 文件头「排队时去找别的机器」），
#   出路是带 --peer 重跑的整条命令（补丁这一趟已打进要验的树的，重跑带 --skip-apply 只验不打；补丁目录写绝对路径）；后面几步不再排（每一步都要再等满排队上限），
#   逐步报「本次未跑」。
# 退出码：0 全绿；1 有红（测试、fmt、clippy、build、apply 退 1、显存不够、源码戳刷不成、check.sh 读不出、工作区成员排不掉 checker 档包、
#   改过的包没重编、主工作区合入锁没拿成、本机内存排不上、--peer 时第二台那一趟没跑成、
#   不带 --peer 时起 cargo 之前 RocksDB 预编库的取值判红（下面「RocksDB 预编库的取值」）、checker 档快档红或起它的命令没带前缀（下面「checker 档快档」））；
#   2 补丁核不过（apply 退 2）、用法错；77 其余都绿、只有 checker 档快档那一步撞了自己的内存上限（本次未跑，下面「内存上限」一段；有红时仍退 1）。
#   checker 档快档因 --peer、--copy、补丁没碰而本次未跑、测试本次未跑都不改退出码，汇总行里点名。
# RocksDB 预编库的取值：clippy --all-features 与 build --all-targets 编工作区全部成员，连同 checker 档包（verdict-store 默认开、编到 librocksdb-sys）。
#   不带 --peer 时，补丁打完、刷完源码戳之后、四条 cargo 之前跑 python3 scripts/hosts/rocksdb-prebuilt.py entry --root <要编的树>（这台装了、此刻进程环境的取值对得上），
#   判红退 1、一条 cargo 都不起。带 --peer 时四条都在第二台跑，第二台的取值由 multi-host-run.sh --peer 在第二台上起含 cargo 的命令之前判（它文件头 ⑤）。
#   entry 的跳过办法：树里没有依赖 rocksdb 的清单（自证的样本工作区就是这样），或 PATH 上的 cargo 是带「# rocksdb-prebuilt-entry: selftest-fake-cargo」那一行的假 cargo。
# checker 档快档（records/2026-09-16-subagent拆分提案.md 第九十节 ㉓，用户 2026-10-23 定案「合入后验证跑受影响的快档」）：test、fmt、clippy、build 与重编核对之后，
#   起门禁 .claude/gate.d/checker-tier-crash-point-replay.sh 快档里按条复用测试函数的两格（--check crash-case-replay,feature-gated-tests <要验的树>）。
#   键与复用全在那一道里现读（每条测试函数的单元键、判定库、只跑键对不上的、判绿照键记库，它文件头「复用」），这里不另算一份键；它两趟打的「按条复用」那一行里的
#   现判、复用、拒算加总进成功句（读不到那一行不算过）。经内存包装与线程上限起（run_capped：日志 <日志目录>/checker-quick.log 与同名 .problems.log），
#   环境里的 MERGE_AND_VERIFY_BREAK 不交给它；内存上限是它自己的（下面「内存上限」一段）。起它的命令要带 SINGLEFS_HEAVY_TESTS=user-request（提交时 =commit）：没带就不起、记一项红，出路写带前缀重跑的写法。
#   补丁没碰会改快档单元键的东西时不起、报「checker 档快档本次未跑」与原因，不记红、不算过：给了补丁目录，crates.patch 改到的路径都落在包里，
#   这些包都不是 checker 档包 singlefs-checker-tier、也不在它的依赖闭包里（成员 Cargo.toml 三张依赖表连同 [target.*] 下的，一层层往下找），
#   补丁目录里也没有非空的 mutations-append.tsv（往 crates/mutations.tsv 追加的行）。照跑的：没给补丁目录；改到工作区根的 Cargo.toml、Cargo.lock 或 .cargo/；
#   改到不落在任何包里的路径；追加了变异行；改到 checker 档包或它依赖的包；读不了补丁或成员清单（判法是 select_test_packages 那段 python 的 checker_verdict）。
#   另两种本次未跑：带 --peer（编译在第二台，判定库在本机仓的 git 目录里，这一步不交过去）、带 --copy（副本没有 git 目录，判定库读不到）；出路写单跑那两格的整条命令。
#   那一道退 0 是过；退 3 只在交给它的根是它的样本根（有 .crash-point-replay-sample-tools/quick-tier，自证用）时算过，别处退 3 判红；退 77 报本次未跑；
#   内存包装退 252 照「本机排不上」、退 245 报显存不够；别的退出码判红：逐条列出日志里 FAILED 的测试函数、它报的「红的 …」（各自变了的单元）与红的格，
#   出路写红的测试函数并进写段的下一批，与只重跑那几格的整条命令（带前缀与范围声明；键相同的照复用，判绿记过库的不重跑，只现判红的那几条与改后键变了的）。
#   MERGE_AND_VERIFY_CHECKER_QUICK_STAGE 换掉那一道、MERGE_AND_VERIFY_CHECKER_QUICK_ROOT 换掉交给它的根（默认是要验的树；都只给自证）。
# 内存上限（checker 档快档那一步单独一个上限，不跟 test、fmt、clippy、build 共用 --memory；2026-10-23 第一次真跑 --memory 24G，这一步撞了上限，
#   records/2026-09-16-subagent拆分提案.md 第九十节 ㉟「合入后验证新那一步：第一次真跑的验收、内存上限」）：
#   --checker-quick-memory <上限> 给了就用它（写法同 --memory：正整数加 K / M / G / T），不看峰值表；没给时缺省这样取，成功句、开跑那一行与汇总行都写出上限与取法：
#   峰值表（scripts/hosts/run-with-memory-cap.sh 文件头「峰值表」）里有这一步那条命令的记录：取近几次里最大的峰值乘 scripts/hosts/memory_peaks_lib.py 的 CAP_FACTOR（1.5）、
#   向上取整到整数 GiB，写「量过的」；近几次里有一次撞顶，表里记的是那一次的上限、真峰值不小于它，取法里写明。读表、取键、算「近几次最大」都经 memory_peaks_lib
#   （command_key、recorded_peak），这里不另写一份；键是包装收到的那条命令：bash <capped.sh> <线程数> env -u MERGE_AND_VERIFY_BREAK bash <那一道> --check <两格> <根>，跟 run_capped 起它的是同一份数组。
#   表里没有就取推的 36G，写「推的」：2026-10-23 第一次真跑 24G 撞了上限（峰值表记 24G:hit，真峰值没量到、不小于 24G），乘 1.5；之后按 36G 量过：
#   records/2026-10-22-子agent结果落地审计.md 第 366 行那一趟（run-with-memory-cap.sh 36G 单跑那一道的 feature-gated-tests）峰值表记 36.0 GiB、clean、没被杀，
#   峰值正好等于上限（含页缓存），只说明 36G 下跑完了、够不够没量到不撞顶的一趟；records/2026-10-24-checker审计复核修复.md 第 101 行快档两格第六趟按 36G 退 0。
#   三种取法都不比 --memory 小（算出来的比 --memory 小就取 --memory）、不超过本机 slice 总上限的整数 GiB（本机约 40 GiB；超过了包装当场退 252，等多久都放不下）。
#   这一步撞了自己的内存上限（包装退 250，日志里有「撞了内存上限」）：结果不算数，不记代码红、不算过，报「本次未跑：checker 档快档撞了内存上限 <上限>」，
#   出路是只重跑这一步、上限加大一档的整条命令（带 SINGLEFS_HEAVY_TESTS 与 SINGLEFS_RUN_SCOPE）；一档 = 上限乘 CAP_FACTOR 向上取整到整数 GiB，不超过本机 slice 总上限的整数 GiB，
#   已到总上限的写没有更大的一档与降线程上限的出路。整条退 77（其余有红时仍退 1）。退 250 而日志里没有那一句的照旧按红判。
#   MERGE_AND_VERIFY_SLICE_TOTAL（只给自证）顶掉本机 slice 总上限的读法；RUN_WITH_MEMORY_CAP_PEAKS 换峰值表（与包装读的是同一个变量）。
# 自证经内存包装起（records/2026-10-23-subagent拆分提案.md 第一百二十二节 ⑩）：外层包装给它起的命令两个标记（RUN_WITH_MEMORY_CAP_OUTER_UNIT、RUN_WITH_MEMORY_CAP_OUTER_CAP）
#   与它自己建的临时目录（TMPDIR=/tmp/singlefs-run-with-memory-cap-<包装进程号>-…，scripts/hosts/run-with-memory-cap.sh 文件头「临时目录」）。自证开跑之前摘掉两个标记；
#   TMPDIR 是外层包装建的那一份时也不用它，临时目录照不经包装那样建在 /tmp 下（跑完照删）。为什么：自证各格的路径都在临时目录下，包装那一份路径长，
#   快档那一步取峰值表的键（命令原文）跟着变长，超过 memory_peaks_lib 的 KEY_LENGTH_LIMIT 被截；加键换算之前那一份库读表时不截表里的键，
#   经 run-with-memory-cap.sh 4G 跑「峰值表里这一步近几次最大 10 GiB」「峰值表算出来的比 --memory 小」两格查不到表里那一行、取了推的 36G、判红。
#   「快档那一步：内存上限 经内存包装起……」那一格在路径长过 KEY_LENGTH_LIMIT 的树上带两个标记再验一次，取的上限要与不经包装那一格相同。
# 弄坏开关 MERGE_AND_VERIFY_BREAK=<项>（只给证红用；打开时自证退 1）。逐项写开关名、弄坏哪一处、打开后自证哪几项红
# （自证的三例没有标签，项名直接写那条「✗ 自检：」失败行的句首，括号里写那一行接着报的）：
#   MERGE_AND_VERIFY_BREAK=logs-single  每一步的日志只写一份全量的（改前的样子：不经 two-logs.sh，没有 .problems.log）
#       「一条测试红时每一步的日志应当出两份（test.log 全量；test.problems.log 带行号收那条 FAILED）」
#   MERGE_AND_VERIFY_BREAK=tests-one-binary-at-a-time  测试那一步照改前一条 cargo test 一个测试二进制接一个地跑，不经 test-targets-parallel.py
#       「测试那一步应当经 test-targets-parallel.py 按测试目标并行起」（实际假 cargo 只收到一条 test --offline --no-fail-fast -p demo）；
#       连带「测试那一步应当带 --include-ignored 把慢档一起跑」（并行起那一格只收到一条 cargo test，没有 --lib、--test alpha 那两条，成功句也没有慢档条数）
#   MERGE_AND_VERIFY_BREAK=ignore-red  verify() 抓到 FAILED 行、cargo test 退非 0 都不记红，一律报测试全绿
#       「一条测试红时应当退 1、列出那条测试与只重跑它的命令」（实际退出码 0，报『✓ 测试全绿（0 个测试二进制）』）；
#       连带「--copy 测试红时也应当删掉副本与编译目录」（那一格的测试红被吞掉，退 0 不是要的 1）
#       连带红的格：「✗ 自检：一条测试红时每一步的日志」、「✗ 自检：--peer 时编译与测试交给第二台（一趟 multi-host-run.sh --peer、不交共用编译目录，test、fmt…」、
#       「✗ 自检：测试那一步…按测试目标并行起」（红的那一趟要退 1）
#   SINGLEFS_WORK_ROOT_BREAK=new-root-unknown（lib_work_root.py 的开关：工作根退回旧根 /tmp/claude-<uid>）  没设日志目录时日志落进旧根
#       「没设日志目录时日志应当落在工作根下」（跑完删掉旧根下多出的那个 merge-and-verify/<日期>-<进程号>）
#   MERGE_AND_VERIFY_BREAK=no-source-stamp  编之前不刷源码戳
#       「往编过的副本里 rsync -a 铺新内容（旧 mtime）之后再验，应当重编改过的 walk.rs、测试全绿」（实际退出码 1，假 cargo 报 Fresh、那一条 FAILED）；
#       连带「--copy 全绿时应当在副本里打补丁、收尾删掉副本与编译目录」（那一格还核源码戳记在 <日志目录>/target 下，不刷就没有那一行）
#   MERGE_AND_VERIFY_BREAK=gpu-memory-as-test-red  cargo test 那一趟包装退 245 也照 FAILED 行报测试红
#       「cargo test 那一趟包装退 245 时应当报显存不够、退 1，不列 FAILED、不给重跑命令」（实际报「红的测试 1 条」与重跑命令）
#   MERGE_AND_VERIFY_BREAK=drop-discipline-lint  从 check.sh 读到的 lint 表丢掉最后一条再交给 clippy
#       「clippy 应当带上 check.sh 的 lint 表里每一条」（实际少一条 -D clippy::shadow_unrelated 这类的）
#   MERGE_AND_VERIFY_BREAK=lint-table-optional  check.sh 读不出 lint 表、或四条写法对不上时只提一句、照常往下验
#       「check.sh 里读不出 lint 表时应当退 1、不编」与「check.sh 的 clippy 那一行变了时应当退 1、不编」（实际退 0、照常编）
#   MERGE_AND_VERIFY_BREAK=skip-build  不跑 cargo build --offline --all-targets
#       「应当跑 cargo build --offline --all-targets」（实际假 cargo 没收到 build）
#       连带红的格：「✗ 自检：--peer 时第二台的工具链没装 rustfmt 与 clippy」、「✗ 自检：--peer 时编译与测试交给第二台（一趟 multi-host-run.sh --peer、不交共用编译目录，test、fmt…」、「✗ 自检：本机内存排不上（包装退 252）时」
#   MERGE_AND_VERIFY_BREAK=fixed-harness-packages  cargo test 退回写死的四个包 singlefs-format、singlefs-core、singlefs-harness、singlefs-checker
#       「cargo test 应当挑工作区成员减掉 checker 档包」（实际收到的是写死的四个包，样本树的成员 demo 不在里面）；
#       「工作区成员里没有 singlefs-checker-tier 时应当退 1、不编」也红（写死的表不读成员，照编）；
#       按补丁挑包的那几格与别的带补丁目录的格也红（写死的四个包都不在样本工作区里，按补丁挑出来一个不剩，测试本次未跑）
#       连带红的格：「✗ 自检：--copy 测试红时也」、「✗ 自检：--peer 时取回的日志全绿」、「✗ 自检：--peer 时第二台的工具链没装 rustfmt 与 clippy」、「✗ 自检：--peer 时编译与测试交给第二台（一趟 multi-host-run.sh --peer、不交共用编译目录，test、fmt…」、「✗ 自检：一条测试红时」、「✗ 自检：一条测试红时每一步的日志」、「✗ 自检：全绿时」、「✗ 自检：带 --all-packages：cargo test」、「✗ 自检：本机内存排不上（包装退 252）时」、「✗ 自检：没设日志目录时日志」、「✗ 自检：测试那一步」、「✗ 自检：编译目录里的产物比源码新（共用编译目录）、补丁改过的 demo 没有 Compiling 行」、「✗ 自检：补丁改到 alpha 与 beta 两个包：cargo test 只该挑这两个、报 demo 与 delta 本次未跑」、「✗ 自检：补丁改到 demo：cargo test」、「✗ 自检：补丁改到工作区根的 Cargo.lock：cargo test」
#   MERGE_AND_VERIFY_BREAK=no-checker-tier-guard  工作区成员里找不到 singlefs-checker-tier 也照挑全部成员
#       「工作区成员里没有 singlefs-checker-tier 时应当退 1、不编」（实际退 0，改了名的 checker 档包被跑进来）
#   MERGE_AND_VERIFY_BREAK=zero-tests-green  cargo test 退 0 而一条测试都没数到时照报全绿
#       「cargo test 退 0 却一条测试都没跑时应当退 1」（实际退 0）
#   MERGE_AND_VERIFY_BREAK=keep-copy  --copy 收尾不删副本与编译目录、也不打路径（改前的样子：副本留在工作根下没人删）
#       「--copy 全绿时应当在副本里打补丁、收尾删掉副本与编译目录」与「--copy 测试红时也应当删掉副本与编译目录」（实际两份都还在）；
#       连带「MERGE_AND_VERIFY_KEEP_COPY=1 时应当留着副本与编译目录、打出路径」（开关在留现场那一句之前就退出，路径不打）
#   MERGE_AND_VERIFY_BREAK=cleanup-only-on-success  --copy 只在退 0 时删副本与编译目录，红了留着
#       「--copy 测试红时也应当删掉副本与编译目录」（实际退 1、两份都还在）
#   MERGE_AND_VERIFY_BREAK=skip-rebuild-check  编完不核改过的包有没有 Compiling 行
#       「编译目录里的产物比源码新（共用编译目录）、补丁改过的 demo 没有 Compiling 行：应当退 1、点名 demo」（实际退 0）
#       连带：「头一次编、补丁改过的 demo 有 Compiling 行：应当退 0、报都重编」「编过之后 --skip-apply 再验同一批补丁：改过的 demo 应当重编、退 0」（不核就不报「都重编了」，判这一句的两格跟着红）
#       连带红的格：「✗ 自检：--peer 时编译与测试交给第二台（一趟 multi-host-run.sh --peer、不交共用编译目录，test、fmt…」
#   MERGE_AND_VERIFY_BREAK=no-touch-patched  编之前不把补丁改过的文件刷到此刻（刷源码戳照刷）
#       「编过之后 --skip-apply 再验同一批补丁：改过的 demo 应当重编、退 0」（实际源码戳没变、没有 Compiling demo，退 1）
#   MERGE_AND_VERIFY_BREAK=ignore-workspace-lock  不带 --copy 时不拿主工作区合入锁
#       「两个合入同时起：后起的应当排队、报出前者的进程号、前者编完放锁后才编」（实际两趟的 cargo test 叠在一起）与
#       「锁文件里的持有者已死：应当报出来、接管、照常验完、放锁后记录清空」（实际没报接管）
#   MERGE_AND_VERIFY_BREAK=all-packages-always  测试步不按补丁挑包，照旧跑全部（改前的样子）
#       「补丁改到 alpha 与 beta 两个包：cargo test 只该挑这两个、报 demo 与 delta 本次未跑」（实际收到四个包）；
#       连带「补丁改到 demo：…」「补丁只改到 checker 档包：…」（照旧全跑）与「补丁改到工作区根的 Cargo.lock：…」（挑法那一句写的是开关，不是 Cargo.lock）
#   MERGE_AND_VERIFY_BREAK=lock-changes-narrowed  补丁改到工作区根的 Cargo.toml、Cargo.lock 或 .cargo/ 时不照旧全跑（当成没改到包）
#       「补丁改到工作区根的 Cargo.lock：cargo test 应当照旧挑全部四个包」（实际测试本次未跑）
#   MERGE_AND_VERIFY_BREAK=dependents-dropped  只挑补丁改到的包，不带上工作区里依赖它们的包
#       「补丁改到 demo：cargo test 应当挑 demo 与依赖它的 delta」（实际只收到 -p demo）
#   MERGE_AND_VERIFY_BREAK=skip-apply-unchecked  --skip-apply 时不核补丁打过没有（改前的样子）
#       「--skip-apply 给的补丁没打在树上：应当退 1、点名那份补丁、不编」（实际退 0、照常编，在没打补丁的树上验出绿）；
#       连带「补丁改到 alpha 与 beta 两个包：…」（那一格另核『核过 1 份补丁都已打在树上』这一行，不核就不打）
#   MERGE_AND_VERIFY_BREAK=peer-ignored  认了 --peer 也照旧在本机跑（改前的样子：没有这个选项）
#       「--peer 时编译与测试交给第二台」与「--peer 时取回的日志全绿」（实际假 multi-host-run.sh 一次都没被调、四步在本机跑）、
#       「--peer 时第二台那一趟没跑成」（实际在本机跑完，没报没跑成）；
#       连带「checker 档快档那一步：带 --copy 与带 --peer 时应当报本次未跑」（--peer 那一趟在本机跑，checker 档快档那一步照往下走）、
#       「--copy --peer 时第二台那一趟读得到主仓的多机配置」（假 multi-host-run.sh 一次都没被调）
#       连带红的格：「✗ 自检：--peer 时第二台的工具链没装 rustfmt 与 clippy」
#   MERGE_AND_VERIFY_BREAK=peer-failure-unreported  第二台那一趟没跑成（multi-host-run.sh 退非 0）时不判红、照往下判各步
#       「--peer 时第二台那一趟没跑成」（实际没报「第二台那一趟没跑成」，各步按没取回的结果判）
#   MERGE_AND_VERIFY_BREAK=peer-components-unchecked  第二台那一趟不先核 rustfmt 与 clippy 装没装，fmt、clippy 照起（改前的样子：组件没装的被记成红）
#       「--peer 时第二台的工具链没装 rustfmt 与 clippy，应当不起 fmt、clippy、报本次未跑与 rustup component add 的出路、不记红」（实际退 1，打 ✗ cargo fmt --check 红、✗ clippy 红）
#   MERGE_AND_VERIFY_BREAK=no-peer-hint  本机排不上时出路不写带 --peer 重跑的命令（改前的样子）
#       「本机内存排不上（包装退 252）时」（实际出路里没有 merge-and-verify.sh --peer）
#   MERGE_AND_VERIFY_BREAK=refused-keeps-going  本机一步排不上之后后面几步照排
#       「本机内存排不上（包装退 252）时」（实际假包装被调了 4 次，后面三步各排一次）
#   MERGE_AND_VERIFY_BREAK=keep-run-target  收尾不删这一趟自己建的编译目录 <日志目录>/target（改前的样子：不带 --copy 时编进树根的 target、跨趟留着）
#       「跑完运行目录里不剩它建的编译目录」（正常、出错、被 TERM 三种结局，实际 <日志目录>/target 还在）；
#       连带「--copy 全绿时应当在副本里打补丁、收尾删掉副本与编译目录」与「--copy 测试红时也应当删掉副本与编译目录」（编译目录还在）、
#       「这一趟的编译目录里还有进程开着文件：应当不删、报出那个进程、退 1」（不删也不核，退 0）
#   MERGE_AND_VERIFY_BREAK=run-target-open-files-unchecked  删这一趟的编译目录之前不核有没有进程开着里面的文件（run_build_directory.py remove --skip-open-check）
#       「这一趟的编译目录里还有进程开着文件：应当不删、报出那个进程、退 1」（实际删了、退 0）
#   MERGE_AND_VERIFY_BREAK=rocksdb-entry-unchecked  本机起四条 cargo 之前不调 scripts/hosts/rocksdb-prebuilt.py entry（改前的样子）
#       「起 cargo 之前 RocksDB 预编库的取值缺了……应当退 1、一条 cargo 都不起」（实际假 cargo 收到四条）
#   MERGE_AND_VERIFY_BREAK=copy-records-home-unreported  --copy 建副本时不打、不写 GATE_MUTATION_RECORDS_HOME（改前的样子）
#       「--copy 时打出并写下判定库位置与多机配置」（实际输出与 <日志目录>/copy-environment 里都没有 GATE_MUTATION_RECORDS_HOME=…）
#   MERGE_AND_VERIFY_BREAK=copy-peer-configuration-dropped  --copy --peer 不把要验的树的多机配置交给第二台那一趟（改前的样子）
#       「--copy --peer 时第二台那一趟读得到主仓的多机配置」（实际假 multi-host-run.sh 收到的 SINGLEFS_MULTI_HOST_CONFIG 是空的，按副本找不到配置退 255）
#   MERGE_AND_VERIFY_BREAK=slow-tier-skipped  测试那一步不带 --include-ignored（改前的样子：harness 慢档不跑）
#       「测试那一步应当带 --include-ignored 把慢档一起跑」（实际假 cargo 收到的测试命令没有 -- --include-ignored，成功句没有慢档条数）
#   MERGE_AND_VERIFY_BREAK=quick-memory-key-truncated-raw  checker 档快档那一步取峰值表的键时先把命令原文截到 KEY_LENGTH_LIMIT 再换算（截的是原文，换算之后比表里那一行短）
#       「checker 档快档那一步：内存上限 经内存包装起」（实际取了推的 36G：路径长的那一格查不到表里那一行；路径短的几格键不超长，照绿）
#   checker 档快档那一步（文件头「checker 档快档」）的七个开关，点名的格都以「checker 档快档那一步：」起头：
#   MERGE_AND_VERIFY_BREAK=checker-quick-skipped  不跑 checker 档快档那一步（改前的样子）
#       「checker 档快档那一步：补丁改到 checker 档包、改了 4 条快档测试函数闭包里的单元……」「checker 档快档那一步：键都对上时应当现判 0……」
#       「checker 档快档那一步：样本里 test_t 判 FAILED 时应当退 1……」「checker 档快档那一步：补丁改到 checker 档包依赖的 demo、起它的命令没带 SINGLEFS_HEAVY_TESTS……」
#       「checker 档快档那一步：补丁只改到不在 checker 档包依赖闭包里的 alpha、beta……」「checker 档快档那一步：那一道退 3 而根不是它的样本根……」
#       「checker 档快档那一步：带 --copy 与带 --peer 时应当报本次未跑」（七格都报不出那一步的行）
#       连带红的格：「✗ 自检：checker 档快档那一步：内存上限 --checker-quick-memory 给了 20G」、「✗ 自检：checker 档快档那一步：内存上限 峰值表里这一步近几次最大 10 GiB（其中一次撞了顶），没给上限时」、「✗ 自检：checker 档快档那一步：内存上限 峰值表算出来的比 --memory 小时」、「✗ 自检：checker 档快档那一步：内存上限 峰值表里没有这一步时」、「✗ 自检：checker 档快档那一步：撞内存上限（包装退 250，日志里有「撞了内存上限」）」、「✗ 自检：checker 档快档那一步：撞内存上限而上限已是本机总上限 64G 时」、「✗ 自检：checker 档快档那一步：撞内存上限 退 250 而日志里没有「撞了内存上限」时」、「✗ 自检：checker 档快档那一步：内存上限 经内存包装起」
#   MERGE_AND_VERIFY_BREAK=checker-quick-elsewhere-run  带 --copy、--peer 时不报本次未跑，照往下走
#       「checker 档快档那一步：带 --copy 与带 --peer 时应当报本次未跑」（实际 --copy 那一趟报的是补丁没碰）
#   MERGE_AND_VERIFY_BREAK=checker-quick-always-run  不看补丁碰没碰会改快档单元键的东西，一律照跑
#       「checker 档快档那一步：补丁只改到不在 checker 档包依赖闭包里的 alpha、beta……」（实际没带前缀被拒、退 1）；
#       连带「本机内存排不上（包装退 252）时……」（那一步也报一行「本次未跑：test 那一步本机内存排不上」，数对不上）
#   MERGE_AND_VERIFY_BREAK=checker-quick-prefix-unchecked  起它的命令没带 SINGLEFS_HEAVY_TESTS 也照起
#       「checker 档快档那一步：补丁改到 checker 档包依赖的 demo、起它的命令没带 SINGLEFS_HEAVY_TESTS……」（实际那一道被起 1 次、退 0）
#   MERGE_AND_VERIFY_BREAK=checker-quick-dependencies-ignored  依赖闭包只算 checker 档包自己，不往下找它依赖的包
#       「checker 档快档那一步：补丁改到 checker 档包依赖的 demo、起它的命令没带 SINGLEFS_HEAVY_TESTS……」（实际报补丁没碰、本次未跑、退 0）
#   MERGE_AND_VERIFY_BREAK=checker-quick-sample-exit-anywhere  那一道退 3 不看根是不是样本根，一律算过
#       「checker 档快档那一步：那一道退 3 而根不是它的样本根……」（实际退 0）
#   MERGE_AND_VERIFY_BREAK=checker-quick-red-ignored  那一道红了也报过
#       「checker 档快档那一步：样本里 test_t 判 FAILED 时应当退 1……」与「checker 档快档那一步：那一道退 3 而根不是它的样本根……」（实际都退 0）
#       连带红的格：「✗ 自检：checker 档快档那一步：撞内存上限 退 250 而日志里没有「撞了内存上限」时」
#   checker 档快档那一步内存上限（文件头「内存上限」）的六个开关，点名的格都以「checker 档快档那一步：内存上限」或「checker 档快档那一步：撞内存上限」起头（项名后面接的是失败行句首的后半）：
#   MERGE_AND_VERIFY_BREAK=quick-memory-shared  这一步照改前与别的步共用 --memory，连 --checker-quick-memory 也不认
#       「内存上限 --checker-quick-memory 给了 20G，应当只有这一步收到 20G……」「内存上限 峰值表里这一步近几次最大 10 GiB……」「内存上限 峰值表算出来的比 --memory 小时应当取 --memory 8G……」
#       （这一格实际也收到 8G，红在输出里没有『比 --memory 8G 小，取 --memory』那句取法）「内存上限 峰值表里没有这一步时应当取推的 36G……」
#       「撞内存上限（包装退 250，……）应当不记红、报「本次未跑：checker 档快档撞了内存上限 36G」……」「撞内存上限而上限已是本机总上限 64G 时……」
#       「内存上限 经内存包装起……」（实际快档那一趟都收到 --memory 的 8G）
#   MERGE_AND_VERIFY_BREAK=quick-memory-table-ignored  没给上限时不读峰值表，一律取推的 36G
#       「内存上限 峰值表里这一步近几次最大 10 GiB……」「内存上限 峰值表算出来的比 --memory 小时应当取 --memory 8G……」「内存上限 经内存包装起……」（实际收到 36G）
#   MERGE_AND_VERIFY_BREAK=quick-memory-below-memory  算出来的比 --memory 小也照用，不垫到 --memory
#       「内存上限 峰值表算出来的比 --memory 小时应当取 --memory 8G……」（实际收到 2G）
#   MERGE_AND_VERIFY_BREAK=quick-memory-oom-as-red  这一步撞了自己的内存上限也照改前记代码红（出路写单跑看细节）
#       「撞内存上限（包装退 250，……）应当不记红……」「撞内存上限而上限已是本机总上限 64G 时……」（实际退 1）
#   MERGE_AND_VERIFY_BREAK=quick-memory-oom-exits-zero  撞了内存上限报本次未跑，整条却退 0
#       「撞内存上限（包装退 250，……）应当不记红……」「撞内存上限而上限已是本机总上限 64G 时……」（实际退 0，要的是 77）
#   MERGE_AND_VERIFY_BREAK=quick-memory-same-tier  出路里的上限不加大一档，原样重跑
#       「撞内存上限（包装退 250，……）应当不记红……」（实际命令里是 36G，要的是 54G）、「撞内存上限而上限已是本机总上限 64G 时……」（实际照样打出重跑命令）
#   本库 scripts/gate/lib_rebuilt_packages.py 的 REBUILT_PACKAGES_BREAK=any-compiling-line 也让「编译目录里的产物比源码新……」那一格红（另一个包有 Compiling 行就当 demo 也重编了）；
#   它的 REBUILT_PACKAGES_BREAK=absolute-dep-info-only 让「头一次编、补丁改过的 demo 有 Compiling 行……」「编过之后 --skip-apply 再验同一批补丁……」
#     「编译目录里的产物比源码新（共用编译目录）、补丁改过的 demo 没有 Compiling 行……」三格红（假 cargo 写的 dep-info 是相对路径，只拿绝对路径比就一个都对不上，
#     三格都报「重编核对本次未判」、不报都重编了也不点名 demo；拆分提案第一百零八节 ③）；
#   scripts/gate/lib_workspace_lock.py 的 WORKSPACE_LOCK_BREAK=trust-dead-holder 让「锁文件里的持有者已死……」那一格红（排队等死掉的持有者，到 SINGLEFS_WORKSPACE_LOCK_GIVE_UP_SECONDS 退 1），
#     连带「--peer 时编译与测试交给第二台……」「--peer 时取回的日志全绿……」「--peer 时第二台那一趟没跑成……」「--peer 时第二台的工具链没装 rustfmt 与 clippy……」
#     「本机内存排不上（包装退 252）时……」「checker 档快档那一步：带 --copy 与带 --peer 时应当报本次未跑」六格；
#   它的 WORKSPACE_LOCK_BREAK=no-wait 让「两个合入同时起……」与「锁文件里的持有者已死……」两格红（活着的持有者也照拿、不排队；死的不报接管）；
#   它的 WORKSPACE_LOCK_BREAK=lock-in-common-dir 让「主工作树的锁被占着时在 git worktree 加出来的树里验……」那一格红（锁文件照改前放 git common-dir，排到点退 1）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../.." && pwd)"
source "$here/../hosts/heavy-threads.sh"
APPLY="${MERGE_AND_VERIFY_APPLY:-$here/apply-writer-patch.py}"
WRAPPER="${MERGE_AND_VERIFY_WRAPPER:-$here/../hosts/run-with-memory-cap.sh}"
CAPPED="${MERGE_AND_VERIFY_CAPPED:-$here/../hosts/capped.sh}"
TWO_LOGS="${MERGE_AND_VERIFY_TWO_LOGS:-$here/../hosts/two-logs.sh}"   # 每一步的日志出两份（run_capped）
TEST_TARGETS_PARALLEL="$here/test-targets-parallel.py"   # 测试那一步按测试目标并行起（文件头 test 一段）；--peer 时用第二台副本里同一相对路径的那一份
STAMP="$here/../hosts/source-stamp.py"
REBUILT="$here/lib_rebuilt_packages.py"          # 改过的包重编了没有（文件头）
WORKSPACE_LOCK="$here/lib_workspace_lock.py"     # 主工作区合入锁（文件头）
CHECK_PATCH_DIRS=()                              # 算改过的包用的补丁目录（绝对路径；--skip-apply 时也留着）
GPU_MEMORY_HIT_EXIT=245   # run-with-memory-cap.sh：显存不够（命令退了非 0、起止之间内核报了 NV_ERR_NO_MEMORY），结果不算数
MEMORY_ADMISSION_REFUSED_EXIT=252   # run-with-memory-cap.sh：内存不够排不上，命令一行没跑（文件头「本机排不上」）
MEMORY_CAP_HIT_EXIT=250   # run-with-memory-cap.sh：撞了这一条自己的内存上限（文件头「退出码」）
NOT_RUN_EXIT=77           # 其余都绿、只有 checker 档快档那一步撞了自己的内存上限：本次未跑（文件头「内存上限」）
PEER_MODE=0                                       # --peer（文件头「--peer」）
PEER_RESULTS=""                                   # --peer：第二台那一趟取回的各步日志与退出码所在目录
PEER_LOG_DIRECTORY=merge-and-verify-peer-logs     # 第二台那棵副本里放各步日志与退出码的目录（相对树根），--fetch 取回
RUN_BUILD_DIRECTORY="$here/../hosts/run_build_directory.py"   # 删这一趟自己建的编译目录（文件头「这一趟的编译目录」）
CHECKER_QUICK_STAGE="${MERGE_AND_VERIFY_CHECKER_QUICK_STAGE:-$root/.claude/gate.d/checker-tier-crash-point-replay.sh}"   # checker 档快档那一步起的门禁（文件头「checker 档快档」；换它只给自证）
CHECKER_QUICK_CELLS="crash-case-replay,feature-gated-tests"   # 那一道快档里按条复用测试函数的两格
CHECKER_QUICK_AFFECTED=""; CHECKER_QUICK_REASON=""   # select_test_packages 填：补丁碰没碰会改快档单元键的东西（1 碰了、0 没碰、空 算不出）与一句原因
CHECKER_QUICK_STATE=""                            # checker_quick_step 填：汇总行里的一句
CHECKER_QUICK_MEMORY_GIVEN=""                     # --checker-quick-memory 给的上限（空是没给，缺省取法见文件头「内存上限」）
CHECKER_QUICK_MEMORY=""; CHECKER_QUICK_MEMORY_HOW=""   # checker_quick_resolve_memory 填：这一步用的上限与取法
CHECKER_QUICK_MEMORY_INFERRED=36G                 # 峰值表里没有这一步时取的数。推的：2026-10-23 第一次真跑 24G 撞了上限，乘 CAP_FACTOR 1.5；按这个数量过的那两趟见文件头「内存上限」
CHECKER_QUICK_MEMORY_HIT=0                        # 这一步撞了自己的内存上限：本次未跑，整条退 NOT_RUN_EXIT
CHECKER_QUICK_NEXT_MEMORY=""; CHECKER_QUICK_NEXT_NOTE=""   # checker_quick_next_memory 填：加大一档的上限（"-" 是没有更大的一档）与一句说明
CHECKER_QUICK_WORDS=()                            # checker_quick_step 填：run_capped 起那一道的命令词（取峰值表的键也用它，文件头「内存上限」）
LOCAL_REFUSED_STEP=""                             # 本机第一步排不上的那一步（文件头「本机排不上」）
PEER_NOT_RUN_STEPS=()                             # --peer：第二台工具链缺组件、这一趟没起的步（文件头「--peer」第二台的 rustfmt 与 clippy）
COPY_RECORDS_HOME=""; COPY_MULTI_HOST_CONFIG=""   # --copy：要验的树的 git common-dir 与多机配置（文件头 --copy「副本不带 .git」；copy_environment 填）
SLOW_TIER_ARGUMENTS=(--include-ignored)           # 测试那一步交给测试二进制的参数：harness 慢档一起跑（文件头 test 一段「慢档一起跑」）
# 弄坏开关 slow-tier-skipped：照改前不带，慢档不跑
[[ "${MERGE_AND_VERIFY_BREAK:-}" != "slow-tier-skipped" ]] || SLOW_TIER_ARGUMENTS=()
# 第二台那一趟在副本的树根里跑的：逐步经副本里的内存包装与 capped.sh 跑，每步的输出与退出码写进 <日志目录>/<步>.log、.exit；参数是 <上限> <N> <日志目录> <核组件 1 或 0> 再接 <步名> <命令…> :: …
# 核组件（文件头「--peer」第二台的 rustfmt 与 clippy）：cargo fmt / cargo clippy 这两步之前先在副本的树根里问一句 cargo <子命令> --version，答不出（组件没装，rustup 报
# is not installed for the toolchain）就不跑这一步：<步>.log 写本次未跑与 --version 的原话，<步>.not-run 留一个标记，<步>.exit 写 77，接着跑下一步
PEER_STEPS_SCRIPT='memory="$1" threads="$2" logs="$3" probe="$4"
shift 4
mkdir -p -- "$logs" || exit 70
while (($#)); do
  step="$1"
  shift
  words=()
  while (($#)) && [[ "$1" != "::" ]]; do words+=("$1"); shift; done
  (($#)) && shift
  if [[ "$probe" == 1 && "${words[0]-}" == cargo && ( "${words[1]-}" == fmt || "${words[1]-}" == clippy ) ]] \
     && ! cargo "${words[1]}" --version > "$logs/$step.probe" 2>&1; then
    { printf "本次未跑：第二台的工具链没装 cargo %s 要的 rustup 组件，这一步没起；cargo %s --version 的原话：\n" "${words[1]}" "${words[1]}"; cat "$logs/$step.probe"; } > "$logs/$step.log"
    : > "$logs/$step.not-run"
    printf "%s\n" 77 > "$logs/$step.exit"
    continue
  fi
  bash scripts/hosts/run-with-memory-cap.sh "$memory" bash scripts/hosts/capped.sh "$threads" "${words[@]}" > "$logs/$step.log" 2>&1
  printf "%s\n" "$?" > "$logs/$step.exit"
done'
CHECK_SH="${MERGE_AND_VERIFY_CHECK_SH:-$root/.claude/singlefs-ai-sop/scripts/check.sh}"
CHECKER_TIER_PACKAGE=singlefs-checker-tier   # checker 档包（.claude/rules/verification.md）：这里的 cargo test 不挑它，它的快档测试函数归 checker_quick_step，全量归提交时的主 agent
# check.sh 里四条 cargo 的写法：每条在它里面恰好出现一次，这里的 test、fmt、clippy、build 照它们对齐（test 只少一个 checker 档包）。
CHECK_SH_COMMANDS=(
  'run_cargo fmt --all -- --check '
  'run_cargo clippy --all-targets --all-features -- -D warnings "${CODE_DISCIPLINE_LINTS[@]}"'
  'run_cargo build --all-targets '
  'run_cargo test --all 2>&1'
)

die() { echo "  ✗ $1"; echo "  → 怎么办：$2"; exit "${3:-2}"; }

# 收尾（EXIT trap，正常结束、红了退出、被 INT / TERM 停都走这里）：不带 --copy 时放掉主工作区合入锁（文件头「主工作区合入锁」；放不成只报一行，退出码照旧）；
# --copy 时删掉这里建的副本；删掉这一趟自己建的编译目录（文件头「这一趟的编译目录」）。MERGE_AND_VERIFY_KEEP_COPY=1 时副本与编译目录都留着、打路径。删不成退 1。
COPY_TREE=""; RUN_TARGET=""; WORKSPACE_LOCK_HELD=0
finish() {
  local rc=$? kept_size
  trap - EXIT
  cd / || true
  if ((WORKSPACE_LOCK_HELD)); then
    python3 "$WORKSPACE_LOCK" release --root "$tree" --holder-pid "$$" || echo "  ! 放主工作区合入锁没成（上面一行是原因）：看 $WORKSPACE_LOCK status --root $tree"
  fi
  [[ -n "$COPY_TREE" || -n "$RUN_TARGET" ]] || exit "$rc"
  if [[ -n "$COPY_TREE" ]] && { [[ "${MERGE_AND_VERIFY_BREAK:-}" == "keep-copy" ]] || { [[ "${MERGE_AND_VERIFY_BREAK:-}" == "cleanup-only-on-success" ]] && ((rc != 0)); }; }; then
    exit "$rc"
  fi
  if [[ "${MERGE_AND_VERIFY_KEEP_COPY:-}" == 1 ]]; then
    echo "  ! 留着${COPY_TREE:+副本 $COPY_TREE}${COPY_TREE:+${RUN_TARGET:+ 与}}${RUN_TARGET:+编译目录 $RUN_TARGET}（MERGE_AND_VERIFY_KEEP_COPY=1）"
    echo "  → 看完自己删：rm -rf --${COPY_TREE:+ '$COPY_TREE'}${RUN_TARGET:+ '$RUN_TARGET'}"
    exit "$rc"
  fi
  if [[ -n "$COPY_TREE" ]]; then
    kept_size="$(du -sh -- "$COPY_TREE" 2>/dev/null | cut -f1)"
    rm -rf -- "${COPY_TREE:?}"
    if [[ -e "$COPY_TREE" ]]; then
      echo "  ✗ 副本 $COPY_TREE 没删成"
      echo "  → 怎么办：看谁还开着里面的文件（python3 $RUN_BUILD_DIRECTORY holders '$COPY_TREE'），停了再 rm -rf -- 那一份"
      rc=1
    else
      echo "  删了副本 $COPY_TREE（共 ${kept_size:-?}；日志留在 $(dirname "$COPY_TREE")）"
    fi
  fi
  # 弄坏开关 keep-run-target：不删这一趟的编译目录；run-target-open-files-unchecked：删之前不核开着的文件
  if [[ -n "$RUN_TARGET" && "${MERGE_AND_VERIFY_BREAK:-}" != "keep-run-target" ]]; then
    python3 "$RUN_BUILD_DIRECTORY" remove "$RUN_TARGET" $([[ "${MERGE_AND_VERIFY_BREAK:-}" == "run-target-open-files-unchecked" ]] && echo --skip-open-check) || rc=1
  fi
  exit "$rc"
}

# check_patches_applied <日志目录>：--skip-apply 时核给的每份 crates.patch 都已打在要验的树上（文件头「--skip-apply 先核补丁打过没有」）；核不过退 1、不编。
check_patches_applied() {
  local logs="$1" dir patch checked_count=0 unapplied_text=""
  local -a unapplied=()
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "skip-apply-unchecked" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=skip-apply-unchecked：--skip-apply 时不核补丁打过没有"; return 0
  fi
  if ((${#CHECK_PATCH_DIRS[@]} == 0)); then echo "  - 补丁打没打本次未核：带了 --skip-apply 而没给补丁目录"; return 0; fi
  mkdir -p "$logs" || die "建不了日志目录 $logs" "看工作根在不在、可不可写" 1
  for dir in "${CHECK_PATCH_DIRS[@]}"; do
    if [[ ! -d "$dir" ]]; then unapplied+=("$dir（目录不在）"); continue; fi
    patch="$dir/crates.patch"
    [[ -s "$patch" ]] || continue
    checked_count=$((checked_count+1))
    if ! git -C "$tree" apply --check --reverse "$patch" > "$logs/reverse-check-$(basename "$dir").log" 2>&1; then
      unapplied+=("$dir（日志 $logs/reverse-check-$(basename "$dir").log）")
    fi
  done
  if ((${#unapplied[@]})); then
    unapplied_text="$(printf '%s、' "${unapplied[@]}")"
    die "带了 --skip-apply，给的补丁有 ${#unapplied[@]} 份没打在要验的树 $tree 上（crates.patch 反着核不过：git apply --check --reverse）：${unapplied_text%、}；一个都没编" \
      "没打过的那几份去掉 --skip-apply 让这里打（打过的与没打过的分两趟）；同一批里两份补丁改到同一处时前一份也会反着核不过，在仓副本里倒序逐份 git apply --reverse 人工核" 1
  fi
  if ((checked_count == 0)); then echo "  - 补丁打没打本次未核：给的 ${#CHECK_PATCH_DIRS[@]} 个补丁目录里都没有 crates.patch"; return 0; fi
  echo "  核过 $checked_count 份补丁都已打在树上（git apply --check --reverse）"
}

# read_check_sh <check.sh>：现读编码纪律那张 lint 表进 DISCIPLINE_LINTS，核四条 cargo 写法；读不出、对不上返回 1，原因进 CHECK_SH_PROBLEM。
read_check_sh() {
  local file="$1" line in_table=0 i command_text hits
  local -a words=()
  DISCIPLINE_LINTS=(); CHECK_SH_PROBLEM=""
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
  for command_text in "${CHECK_SH_COMMANDS[@]}"; do
    hits="$(grep -cF -- "$command_text" "$file")"
    if [[ "$hits" != 1 ]]; then CHECK_SH_PROBLEM="$file 里「$command_text」出现 ${hits:-0} 次，不是 1 次：上游改了这一条 cargo 的写法"; return 1; fi
  done
  return 0
}

# test_packages <树>：工作区成员减掉 checker 档包，逐个写成「-p <包名>」进 TEST_PACKAGES；排不掉 checker 档包、读不出返回 1，原因进 TEST_PACKAGES_PROBLEM。
test_packages() {
  local listing
  TEST_PACKAGES=(); TEST_PACKAGES_PROBLEM=""
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "fixed-harness-packages" ]]; then
    TEST_PACKAGES=(-p singlefs-format -p singlefs-core -p singlefs-harness -p singlefs-checker); return 0
  fi
  if ! listing="$(python3 - "$1" "$CHECKER_TIER_PACKAGE" "${MERGE_AND_VERIFY_BREAK:-}" <<'PY_MEMBERS' 2>&1
import os, sys, tomllib
tree, excluded, broken = sys.argv[1], sys.argv[2], sys.argv[3]
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
if excluded not in names and broken != "no-checker-tier-guard":
    fail(f"工作区成员（{'、'.join(names)}）里没有 {excluded}：checker 档包改了名或挪了，排不掉它")
kept = [name for name in names if name != excluded]
if not kept:
    fail(f"工作区成员减掉 {excluded} 之后一个包都不剩")
print("\n".join(kept))
PY_MEMBERS
  )"; then
    TEST_PACKAGES_PROBLEM="$listing"; return 1
  fi
  local name
  while IFS= read -r name; do [[ -n "$name" ]] && TEST_PACKAGES+=(-p "$name"); done <<< "$listing"
  return 0
}

# select_test_packages <树>：TEST_PACKAGES 里只留这一批补丁改到的包与工作区里依赖它们的包（文件头「测试只跑改到的包」）；照旧全跑的原样留着。
# 没挑上的包名进 UNTESTED_PACKAGES，怎么挑的一句进 TEST_SELECTION_NOTE；读不了补丁或成员清单返回 1，原因进 TEST_SELECTION_PROBLEM。
# 给了补丁目录就同一趟判 checker 档快档那一步起不起（文件头「checker 档快档」），进 CHECKER_QUICK_AFFECTED 与 CHECKER_QUICK_REASON；带 --all-packages 时照判、读不了不算错。
select_test_packages() {
  local listing kind name candidates=() position selected=1 checker_line
  UNTESTED_PACKAGES=(); TEST_SELECTION_NOTE=""; TEST_SELECTION_PROBLEM=""
  CHECKER_QUICK_AFFECTED=""; CHECKER_QUICK_REASON="没给补丁目录，算不出这一批补丁碰没碰 checker 档包，照跑"
  for ((position = 1; position < ${#TEST_PACKAGES[@]}; position += 2)); do candidates+=("${TEST_PACKAGES[position]}"); done
  if ((${#CHECK_PATCH_DIRS[@]})); then
    if ! listing="$(python3 - "$1" "$REBUILT" "${MERGE_AND_VERIFY_BREAK:-}" "${candidates[*]}" "$CHECKER_TIER_PACKAGE" "${CHECK_PATCH_DIRS[@]}" <<'PY_SELECT' 2>&1
import importlib.util, os, sys, tomllib
tree, library, broken, candidate_text, checker, *patch_directories = sys.argv[1:]
candidates = candidate_text.split()
specification = importlib.util.spec_from_file_location("lib_rebuilt_packages_for_merge_and_verify", library)
rebuilt = importlib.util.module_from_spec(specification)
specification.loader.exec_module(rebuilt)
try:
    paths = rebuilt.collect_paths(patch_directories, [])
except OSError as error:
    print(f"读不了补丁：{error}")
    sys.exit(1)
WORKSPACE_WIDE = ("Cargo.toml", "Cargo.lock")
def workspace_wide(path):
    return path in WORKSPACE_WIDE or path == ".cargo" or path.startswith(".cargo/")
def load(path):
    with open(path, "rb") as handle:
        return tomllib.load(handle)
manifests, manifest_problem = {}, None
try:
    members = load(os.path.join(tree, "Cargo.toml")).get("workspace", {}).get("members") or []
    for member in members:
        manifest = load(os.path.join(tree, member, "Cargo.toml"))
        manifests[manifest["package"]["name"]] = manifest
except (OSError, KeyError, TypeError, tomllib.TOMLDecodeError) as error:
    manifests, manifest_problem = {}, f"读不了工作区成员的 Cargo.toml：{error!r}"
def dependency_names(manifest):
    """一个成员在三张依赖表（连同 [target.*] 下的）里写的包名（改了名的照 package = 认）。"""
    tables = [manifest.get(key) for key in ("dependencies", "dev-dependencies", "build-dependencies")]
    for target in (manifest.get("target") or {}).values():
        if isinstance(target, dict):
            tables += [target.get(key) for key in ("dependencies", "dev-dependencies", "build-dependencies")]
    for table in tables:
        for key, value in (table or {}).items():
            yield value.get("package", key) if isinstance(value, dict) else key
depends_on = {name: set(dependency_names(manifest)) & set(manifests) for name, manifest in manifests.items()}
def checker_verdict():
    """checker 档快档那一步起不起（文件头「checker 档快档」）→ (affected 或 unaffected, 一句原因)。"""
    for path in paths:
        if workspace_wide(path):
            return "affected", f"补丁改到工作区根的 {path}"
    owners, loose = set(), []
    for path in paths:
        name = rebuilt.package_of(tree, path)
        if name is None:
            loose.append(path)
        else:
            owners.add(name)
    if loose:
        return "affected", f"补丁改到不落在任何包里的 {'、'.join(loose[:3])}（checker 档的测试运行时可能读它）"
    appended = [directory for directory in patch_directories if os.path.isfile(os.path.join(directory, "mutations-append.tsv"))
                and os.path.getsize(os.path.join(directory, "mutations-append.tsv")) > 0]
    if appended:
        return "affected", f"补丁往 crates/mutations.tsv 追加了行（{appended[0]} 里的 mutations-append.tsv；checker 档的测试运行时可能读它）"
    if manifest_problem or checker not in manifests:
        return "affected", f"算不出 checker 档包 {checker} 依赖哪些包（{manifest_problem or '工作区成员里没有它'}）"
    closure, frontier = {checker}, [checker]
    while frontier and broken != "checker-quick-dependencies-ignored":
        for needed in sorted(depends_on.get(frontier.pop(), ())):
            if needed not in closure:
                closure.add(needed)
                frontier.append(needed)
    hit = sorted(owners & closure)
    if hit:
        return "affected", f"补丁改到 checker 档包或它依赖的包：{'、'.join(hit)}"
    if not owners:
        return "unaffected", "这一批补丁目录里没有改到文件的 crates.patch"
    return "unaffected", f"补丁只改到 {'、'.join(sorted(owners))}，都不是 checker 档包 {checker}、也不在它的依赖闭包里，快档测试函数的单元键不会因这一批变"
print("CHECKER\t%s\t%s" % checker_verdict())
changed, outside = [], []
for path in paths:
    if workspace_wide(path):
        if broken == "lock-changes-narrowed":
            continue
        print(f"ALL\t补丁改到工作区根的 {path}")
        sys.exit(0)
    name = rebuilt.package_of(tree, path)
    if name is None:
        outside.append(path)
    elif name not in changed:
        changed.append(name)
if outside:
    print(f"ALL\t补丁改到不落在任何包里的 {'、'.join(outside[:3])}{' 等 ' + str(len(outside)) + ' 处' if len(outside) > 3 else ''}（测试运行时可能读它，判不出归哪个包）")
    sys.exit(0)
if manifest_problem:
    print(manifest_problem)
    sys.exit(1)
affected = set(changed)
dependents = []
while broken != "dependents-dropped":
    grown = sorted(name for name, needed in depends_on.items() if name not in affected and needed & affected)
    if not grown:
        break
    affected |= set(grown)
    dependents += grown
for name in changed:
    print(f"CHANGED\t{name}")
for name in dependents:
    print(f"DEPENDENT\t{name}")
for name in candidates:
    print(f"{'RUN' if name in affected else 'SKIP'}\t{name}")
PY_SELECT
    )"; then
      selected=0
    fi
    if checker_line="$(grep -m 1 $'^CHECKER\t' <<< "$listing")"; then
      if [[ "$(cut -f2 <<< "$checker_line")" == unaffected ]]; then CHECKER_QUICK_AFFECTED=0; else CHECKER_QUICK_AFFECTED=1; fi
      CHECKER_QUICK_REASON="$(cut -f3- <<< "$checker_line")"
    else
      CHECKER_QUICK_REASON="算不出这一批补丁碰没碰 checker 档包（$(tail -n 1 <<< "$listing")），照跑"
    fi
    listing="$(grep -v $'^CHECKER\t' <<< "$listing")"
    if ((selected == 0 && ALL_PACKAGES == 0)) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != "all-packages-always" ]]; then
      TEST_SELECTION_PROBLEM="$listing"; return 1
    fi
  fi
  if ((ALL_PACKAGES)); then TEST_SELECTION_NOTE="带了 --all-packages，照旧全跑"; return 0; fi
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "all-packages-always" ]]; then TEST_SELECTION_NOTE="弄坏开关 MERGE_AND_VERIFY_BREAK=all-packages-always，照旧全跑"; return 0; fi
  if ((${#CHECK_PATCH_DIRS[@]} == 0)); then TEST_SELECTION_NOTE="没给补丁目录，算不出改到的包，照旧全跑"; return 0; fi
  if grep -q $'^ALL\t' <<< "$listing"; then
    TEST_SELECTION_NOTE="$(sed -n 's/^ALL\t//p' <<< "$listing" | head -n 1)，照旧全跑"; return 0
  fi
  local changed_text="" dependent_text=""
  TEST_PACKAGES=()
  while IFS=$'\t' read -r kind name; do
    case "$kind" in
      RUN) TEST_PACKAGES+=(-p "$name") ;;
      SKIP) UNTESTED_PACKAGES+=("$name") ;;
      CHANGED) changed_text+="$name、" ;;
      DEPENDENT) dependent_text+="$name、" ;;
    esac
  done <<< "$listing"
  TEST_SELECTION_NOTE="补丁改到的包：${changed_text:-一个都没有}"
  TEST_SELECTION_NOTE="${TEST_SELECTION_NOTE%、}"
  [[ -z "$dependent_text" ]] || TEST_SELECTION_NOTE+="；另带上依赖它们的：${dependent_text%、}"
  return 0
}

run_capped() {   # $1 日志文件，其余是命令；经内存包装与线程上限（WRAPPER 为 none 时直接跑，自检用）。--peer 时不起，拷第二台那一趟取回的（文件头「--peer」）。上限是 --memory，RUN_CAPPED_MEMORY 设了用它（checker 档快档那一步，文件头「内存上限」）
  local log="$1" step rc memory="${RUN_CAPPED_MEMORY:-$MEMORY}"
  shift
  step="$(basename "$log" .log)"
  if ((PEER_MODE)); then
    rc="$(cat "$PEER_RESULTS/$step.exit" 2>/dev/null)"
    [[ "$rc" =~ ^[0-9]+$ ]] || { echo "第二台那一趟没取回 $step 这一步的退出码（$PEER_RESULTS/$step.exit）" > "$log"; return 255; }
    cp -- "$PEER_RESULTS/$step.log" "$log" 2>/dev/null || echo "第二台那一趟没取回 $step 这一步的日志" > "$log"
    bash "$TWO_LOGS" --from-log "${log%.log}" >/dev/null 2>&1   # 取回的是第二台那一步的全量日志：本机现筛出同名的 .problems.log
    return "$rc"
  fi
  # 前面一步本机排不上（文件头「本机排不上」）：这一步不再排
  if [[ -n "$LOCAL_REFUSED_STEP" && "${MERGE_AND_VERIFY_BREAK:-}" != "refused-keeps-going" ]]; then
    echo "本次未跑：$LOCAL_REFUSED_STEP 那一步本机内存排不上，这一步不再排" > "$log"
    return "$MEMORY_ADMISSION_REFUSED_EXIT"
  fi
  # 每一步的输出出两份日志（scripts/hosts/two-logs.sh --quiet：$log 是全量，同名的 .problems.log 只收错误与警告，行前带全量日志里的行号；不转打，汇总行点名它们）。
  # 弄坏开关 logs-single：照改前只写一份全量日志
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "logs-single" ]]; then
    if [[ "$WRAPPER" == "none" ]]; then "$@" > "$log" 2>&1; rc=$?
    else bash "$WRAPPER" "$memory" bash "$CAPPED" "$THREADS" "$@" > "$log" 2>&1; rc=$?; fi
  elif [[ "$WRAPPER" == "none" ]]; then bash "$TWO_LOGS" --quiet "${log%.log}" -- "$@"; rc=$?
  else bash "$TWO_LOGS" --quiet "${log%.log}" -- bash "$WRAPPER" "$memory" bash "$CAPPED" "$THREADS" "$@"; rc=$?; fi
  if ((rc == MEMORY_ADMISSION_REFUSED_EXIT)) && [[ -z "$LOCAL_REFUSED_STEP" ]]; then LOCAL_REFUSED_STEP="$step"; fi
  return "$rc"
}

# peer_rerun_command：带 --peer 重跑这一趟的整条命令（文件头「本机排不上」）
peer_rerun_command() {
  local text
  local -a words=(bash "$root/scripts/gate/merge-and-verify.sh" --peer --memory "$MEMORY" --threads "$THREADS")
  ((copy_mode)) && words+=(--copy)
  ((ALL_PACKAGES)) && words+=(--all-packages)
  if ((copy_mode == 0 || skip_apply == 1)); then words+=(--skip-apply); fi   # 补丁这一趟已打进要验的树：重跑只验不打
  words+=(${CHECK_PATCH_DIRS[@]+"${CHECK_PATCH_DIRS[@]}"})
  text="$(printf '%q ' "${words[@]}")"
  printf '%s' "${text% }"
}

# refused_step <步名> <退出码>：这一步是本机内存排不上（包装退 252）就打出来、交 0，不是交 1。第一步排不上的那一步打 ✗ 与出路（调用方记一项红），
# 后面几步（run_capped 没再排）打「本次未跑」
refused_step() {
  local step="$1" hint
  [[ "$2" == "$MEMORY_ADMISSION_REFUSED_EXIT" ]] || return 1
  if [[ "$LOCAL_REFUSED_STEP" != "$step" ]]; then
    echo "  - $step 本次未跑：$LOCAL_REFUSED_STEP 那一步本机内存排不上，后面几步不再排（每一步都要再等满排队上限）"
    return 0
  fi
  hint="$(grep -E '^  → 第二台此刻放得下|没查到别的机器：|现查别的机器：' "$logs/$step.log" 2>/dev/null | tail -n 1)"
  echo "  ✗ $step 那一趟本机内存排不上（内存包装退 $MEMORY_ADMISSION_REFUSED_EXIT：排满了等待上限、或等多久都放不下），一行没编（日志 $logs/$step.log）"
  [[ -z "$hint" ]] || echo "    内存包装现查别的机器：${hint#"${hint%%[![:space:]]*}"}"
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "no-peer-hint" ]]; then
    echo "  → 怎么办：等占着内存的那几条跑完再跑"
  else
    echo "  → 怎么办：编译与测试交给第二台重跑（打补丁照旧在本机，判法与汇总同本机）：$(peer_rerun_command)"
  fi
  return 0
}

# peer_step_not_run <步名>：--peer 时第二台那一趟在这一步之前核过 rustup 组件、没装，没起这一步（PEER_STEPS_SCRIPT 留的 <步>.not-run 标记）
peer_step_not_run() { ((PEER_MODE)) && [[ -f "$PEER_RESULTS/$1.not-run" ]]; }

# report_peer_step_not_run <步名> <组件名>：报本次未跑（不记红）与出路——在第二台装组件，或去掉 --peer 在本机跑
report_peer_step_not_run() {
  local step="$1" component="$2" toolchain said
  said="$(grep -m 1 'not installed' "$logs/$step.log" 2>/dev/null)"
  toolchain="$(sed -n "s/.*not installed for the toolchain '\([^']*\)'.*/\1/p" "$logs/$step.log" 2>/dev/null | head -n 1)"
  PEER_NOT_RUN_STEPS+=("$step")
  echo "  - $step 本次未跑：第二台的工具链没装 $component 组件，这一步没起，不记红（第二台上 cargo $step --version 答：${said:-见 $logs/$step.log}）"
  echo "  → 怎么办：在第二台上装好再带 --peer 重跑：rustup component add rustfmt clippy${toolchain:+ --toolchain $toolchain}；或去掉 --peer 在本机跑 fmt 与 clippy"
}

# copy_environment <要验的树> <日志目录>：--copy 建副本之前取要验的树的 git common-dir 与多机配置（文件头 --copy「副本不带 .git」），填 COPY_RECORDS_HOME、
# COPY_MULTI_HOST_CONFIG，打一行、写进 <日志目录>/copy-environment。取不到不记红，只打「算不出」。
copy_environment() {
  local source="$1" logs="$2" common="" file="$2/copy-environment" text=""
  if common="$(git -C "$source" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" && [[ -n "$common" ]]; then COPY_RECORDS_HOME="$common"; fi
  if [[ -n "${SINGLEFS_MULTI_HOST_CONFIG:-}" ]]; then
    COPY_MULTI_HOST_CONFIG="$SINGLEFS_MULTI_HOST_CONFIG"
    [[ "$COPY_MULTI_HOST_CONFIG" == /* ]] || COPY_MULTI_HOST_CONFIG="$PWD/$COPY_MULTI_HOST_CONFIG"
  elif [[ -n "$COPY_RECORDS_HOME" ]]; then
    COPY_MULTI_HOST_CONFIG="$(dirname "$COPY_RECORDS_HOME")/multi-host.env"
  fi
  # 弄坏开关 copy-records-home-unreported：照改前不打、不写判定库位置
  if [[ -n "$COPY_RECORDS_HOME" && "${MERGE_AND_VERIFY_BREAK:-}" != "copy-records-home-unreported" ]]; then
    printf 'GATE_MUTATION_RECORDS_HOME=%q\n' "$COPY_RECORDS_HOME" >> "$file"
    text+="在副本上起门禁 checker-tier-crates-mutation-replay 的轻型变异要带 GATE_MUTATION_RECORDS_HOME=$COPY_RECORDS_HOME（判定库在要验的树的 git common-dir 下）；"
  elif [[ -z "$COPY_RECORDS_HOME" ]]; then
    text+="算不出判定库位置（$source 不是 git 工作树，取不到 git common-dir），在副本上起轻型变异要自己带 GATE_MUTATION_RECORDS_HOME；"
  fi
  if [[ -n "$COPY_MULTI_HOST_CONFIG" ]]; then
    printf 'SINGLEFS_MULTI_HOST_CONFIG=%q\n' "$COPY_MULTI_HOST_CONFIG" >> "$file"
    text+="多机配置 SINGLEFS_MULTI_HOST_CONFIG=$COPY_MULTI_HOST_CONFIG（带 --peer 时交给第二台那一趟）；"
  else
    text+="算不出多机配置（没设 SINGLEFS_MULTI_HOST_CONFIG、也取不到 git common-dir），带 --peer 时第二台那一趟按副本找、会找不到；"
  fi
  echo "  副本不带 .git：${text%；}$([[ -s "$file" ]] && echo "；写进了 $file（source 它再起）")"
}

# run_peer_steps <日志目录> <步名> <命令…> :: …：--peer 时一趟 multi-host-run.sh --peer 在第二台的副本里逐步跑，取回各步的日志与退出码（文件头「--peer」）；
# 没跑成判红退 1
run_peer_steps() {
  local logs="$1" driver label rc fetched probe_components=1
  local -a configuration_setting=()
  shift
  # --copy 时把要验的树的多机配置交过去（文件头 --copy「副本不带 .git」）；弄坏开关 copy-peer-configuration-dropped：照改前不交
  if ((copy_mode)) && [[ -n "$COPY_MULTI_HOST_CONFIG" && "${MERGE_AND_VERIFY_BREAK:-}" != "copy-peer-configuration-dropped" ]]; then
    configuration_setting=(SINGLEFS_MULTI_HOST_CONFIG="$COPY_MULTI_HOST_CONFIG")
  fi
  # 弄坏开关 peer-components-unchecked：第二台那一趟不先核 rustfmt 与 clippy 装没装，fmt、clippy 照起（改前的样子：组件没装的被记成红）
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != "peer-components-unchecked" ]] || probe_components=0
  driver="${MERGE_AND_VERIFY_MULTI_HOST_RUN:-$tree/scripts/hosts/multi-host-run.sh}"
  label="merge-and-verify-$(date +%s)-$$"
  fetched="$logs/peer-fetched"
  [[ ! -e "$fetched" ]] || die "取回目录 $fetched 已经有东西，不往里取" "换一个日志目录（不设 MERGE_AND_VERIFY_LOG_DIRECTORY 就是新的 <日期>-<进程号>）" 1
  echo "  编译与测试交给第二台（--peer）：multi-host-run.sh --peer 把 $tree 拷过去，在那边的副本里逐步跑（内存上限 $MEMORY、线程上限 $THREADS，编译目录是第二台这一趟那棵副本里的、跑完连同副本删）…"
  env -u MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY -u CARGO_TARGET_DIR ${configuration_setting[@]+"${configuration_setting[@]}"} MULTI_HOST_RUN_FETCH_DIRECTORY="$fetched" \
    bash "$driver" --peer --memory-cap "$MEMORY" --label "$label" --fetch "$PEER_LOG_DIRECTORY" -- \
    bash -c "$PEER_STEPS_SCRIPT" merge-and-verify-peer-steps "$MEMORY" "$THREADS" "$PEER_LOG_DIRECTORY" "$probe_components" "$@" > "$logs/peer.log" 2>&1
  rc=$?
  PEER_RESULTS="$fetched/$PEER_LOG_DIRECTORY"
  if ((rc != 0)) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != "peer-failure-unreported" ]]; then
    die "第二台那一趟没跑成（multi-host-run.sh 退 $rc，日志 $logs/peer.log：$(tail -n 2 "$logs/peer.log" | tr '\n' ' ')），一步都没判" \
      "照那份日志修（多机配置与双机开关、两台工具链、--memory 不超过第二台的 PEER_MEMORY_CAP、第二台的盘）再带 --peer 重跑；或去掉 --peer 在本机跑" 1
  fi
  echo "  第二台那一趟跑完，取回 $(find "$PEER_RESULTS" -name '*.exit' 2>/dev/null | wc -l) 步的日志与退出码（$PEER_RESULTS），照本机同一套判"
}

# checker 档快档那一步的内存上限（文件头「内存上限」）：读峰值表、算整数 GiB、读本机 slice 总上限都经 scripts/hosts/memory_peaks_lib.py，这里只拼起来。
#   default：参数 <库目录> default <峰值表路径，空是默认那一份，- 是不读> <--memory 的写法> <推的数> <命令词…> ⇒ 打一行「<上限>\t<取法>」
#   next：参数 <库目录> next <上限> ⇒ 打一行「<加大一档的上限，- 是没有更大的一档>\t<说明>」
read -r -d '' CHECKER_QUICK_MEMORY_PYTHON <<'PY' || true
import math
import os
import sys

sys.dont_write_bytecode = True
GIBIBYTE = 1024 ** 3
sys.path.insert(0, sys.argv[1])
import memory_peaks_lib as peaks


def whole_gibibytes(byte_count):
    return f"{max(1, math.ceil(byte_count / GIBIBYTE))}G"


def slice_total():
    given = os.environ.get("MERGE_AND_VERIFY_SLICE_TOTAL", "")
    if given:
        return peaks.size_bytes(given)
    try:
        return peaks.headroom(os.environ)["slice_total"]
    except (OSError, ValueError, KeyError):
        return None


if sys.argv[2] == "default":
    table, floor_text, inferred_text = sys.argv[3:6]
    floor, inferred = peaks.size_bytes(floor_text), peaks.size_bytes(inferred_text)
    if floor is None or inferred is None:
        sys.exit(f"上限写法认不出：--memory {floor_text!r}、推的数 {inferred_text!r}")
    key = peaks.command_key(sys.argv[6:], os.environ.get("RUN_WITH_MEMORY_CAP_KEY"))
    if os.environ.get("MERGE_AND_VERIFY_BREAK") == "quick-memory-key-truncated-raw":
        # 弄坏开关：先把命令原文截到 KEY_LENGTH_LIMIT 再换算（截的是原文，换算之后比表里那一行短；路径长的那一格对不上，文件头「自证经内存包装起」）
        key = peaks.normalized_key(" ".join(sys.argv[6:])[:peaks.KEY_LENGTH_LIMIT])
    peak, hit = (None, False) if table == "-" else peaks.recorded_peak(table or peaks.default_peak_table(), key)
    if peak is None:
        wanted = inferred
        how = f"推的：峰值表里没有这一步的记录，取 {inferred_text}（文件头「内存上限」写了依据）"
    else:
        wanted = math.ceil(peak * peaks.CAP_FACTOR)
        flagged = "（其中有一次撞了顶，真峰值不小于它）" if hit else ""
        how = f"量过的：峰值表近几次实测最大 {peaks.human(peak)}{flagged}，乘 {peaks.CAP_FACTOR:g} 留余量"
    cap = peaks.size_bytes(whole_gibibytes(wanted))
    total = slice_total()
    if total and cap > total:
        cap = total // GIBIBYTE * GIBIBYTE
        how += f"；超过本机 slice 总上限 {peaks.human(total)}，取总上限里的整数 GiB"
    if cap < floor:
        cap = floor
        how += f"；比 --memory {floor_text} 小，取 --memory"
    print(f"{floor_text if cap == floor else whole_gibibytes(cap)}\t{how}")
elif sys.argv[2] == "next":
    current = peaks.size_bytes(sys.argv[3])
    if current is None:
        sys.exit(f"上限写法认不出：{sys.argv[3]!r}")
    total = slice_total()
    wanted = math.ceil(current * peaks.CAP_FACTOR / GIBIBYTE) * GIBIBYTE
    if total:
        wanted = min(wanted, total // GIBIBYTE * GIBIBYTE)
    if wanted <= current:
        print(f"-\t这一步的上限 {sys.argv[3]} 已到本机 slice 总上限 {peaks.human(total or current)} 的整数 GiB，没有更大的一档")
    else:
        print(f"{whole_gibibytes(wanted)}\t乘 {peaks.CAP_FACTOR:g} 向上取整到整数 GiB" + ("，不超过本机 slice 总上限" if total else ""))
else:
    sys.exit(f"认不出子命令 {sys.argv[2]!r}")
PY

# checker_quick_resolve_memory：填 CHECKER_QUICK_MEMORY 与 CHECKER_QUICK_MEMORY_HOW（已填过不再算；先于任何 $(…) 调一次，子 shell 里读到的才是填好的）。
#   取法（文件头「内存上限」）：--checker-quick-memory 给的 ⇒ 峰值表里这一步的近几次最大峰值乘 CAP_FACTOR ⇒ 表里没有取推的数；算不出（python3 报错）退回 --memory 并写明
checker_quick_resolve_memory() {
  local out table="${RUN_WITH_MEMORY_CAP_PEAKS:-}" floor_text="$MEMORY"
  [[ -z "$CHECKER_QUICK_MEMORY" ]] || return 0
  # 弄坏开关 quick-memory-shared：这一步照改前与别的步共用 --memory（连 --checker-quick-memory 也不认）；quick-memory-table-ignored：不读峰值表，一律取推的数；quick-memory-below-memory：算出来的比 --memory 小也照用
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "quick-memory-shared" ]]; then
    CHECKER_QUICK_MEMORY="$MEMORY"; CHECKER_QUICK_MEMORY_HOW="弄坏开关 quick-memory-shared：照改前与别的步共用 --memory"; return 0
  fi
  if [[ -n "$CHECKER_QUICK_MEMORY_GIVEN" ]]; then
    CHECKER_QUICK_MEMORY="$CHECKER_QUICK_MEMORY_GIVEN"; CHECKER_QUICK_MEMORY_HOW="--checker-quick-memory 给的"; return 0
  fi
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != "quick-memory-table-ignored" ]] || table="-"
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != "quick-memory-below-memory" ]] || floor_text="1K"
  if out="$(python3 -B -c "$CHECKER_QUICK_MEMORY_PYTHON" "$here/../hosts" default "$table" "$floor_text" "$CHECKER_QUICK_MEMORY_INFERRED" bash "$CAPPED" "$THREADS" ${CHECKER_QUICK_WORDS[@]+"${CHECKER_QUICK_WORDS[@]}"} 2>&1)" \
     && [[ "$out" == *$'\t'* ]]; then
    IFS=$'\t' read -r CHECKER_QUICK_MEMORY CHECKER_QUICK_MEMORY_HOW <<< "$out"
  else
    CHECKER_QUICK_MEMORY="$MEMORY"; CHECKER_QUICK_MEMORY_HOW="取法算不出（$(tail -n 1 <<< "$out")），退回 --memory $MEMORY"
  fi
}

# checker_quick_next_memory：这一步撞了上限之后，填 CHECKER_QUICK_NEXT_MEMORY（加大一档的上限，- 是没有更大的一档）与 CHECKER_QUICK_NEXT_NOTE
checker_quick_next_memory() {
  local out
  # 弄坏开关 quick-memory-same-tier：出路里的上限不加大
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "quick-memory-same-tier" ]]; then
    CHECKER_QUICK_NEXT_MEMORY="$CHECKER_QUICK_MEMORY"; CHECKER_QUICK_NEXT_NOTE="弄坏开关 quick-memory-same-tier：不加大"; return 0
  fi
  if out="$(python3 -B -c "$CHECKER_QUICK_MEMORY_PYTHON" "$here/../hosts" next "$CHECKER_QUICK_MEMORY" 2>&1)" && [[ "$out" == *$'\t'* ]]; then
    IFS=$'\t' read -r CHECKER_QUICK_NEXT_MEMORY CHECKER_QUICK_NEXT_NOTE <<< "$out"
  else
    CHECKER_QUICK_NEXT_MEMORY="-"; CHECKER_QUICK_NEXT_NOTE="下一档算不出（$(tail -n 1 <<< "$out")）"
  fi
}

# checker_quick_rerun_command <逗号隔开的格名> <根> [上限]：只重跑 checker 档快档那几格的整条命令（文件头「checker 档快档」）：前缀与范围声明在前，上限缺省是这一步用的那个，
# 包装次序照 .claude/agent-common.md「跑编译出来的代码经内存包装」那一条（capped.sh、run-with-memory-cap.sh、命令）
checker_quick_rerun_command() {
  local cells="$1" count text heavy="${SINGLEFS_HEAVY_TESTS:-}" memory="${3:-${CHECKER_QUICK_MEMORY:-$MEMORY}}" why="${4:-合入后验证这几格红了，修完只重跑它们：键相同的照复用，只现判红的那几条与改后键变了的}"
  [[ "$heavy" == user-request || "$heavy" == commit ]] || heavy=user-request
  count="$(tr ',' '\n' <<< "$cells" | grep -c .)"
  text="$(printf '%q ' bash "$root/scripts/hosts/capped.sh" "$THREADS" bash "$root/scripts/hosts/run-with-memory-cap.sh" "$memory" bash "$CHECKER_QUICK_STAGE" --check "$cells" "$2")"
  printf "SINGLEFS_HEAVY_TESTS=%s SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 checker-tier-crash-point-replay.sh 的 %s 格; 范围=%s 格; 为什么=%s' %s" \
    "$heavy" "${cells//,/、}" "$count" "$why" "${text% }"
}

# checker_quick_step <日志目录>：checker 档快档那一步（文件头「checker 档快档」）。过与本次未跑交 0、红交 1；汇总行里的一句进 CHECKER_QUICK_STATE
checker_quick_step() {
  local logs="$1" quick_root="${MERGE_AND_VERIFY_CHECKER_QUICK_ROOT:-$tree}" heavy="${SINGLEFS_HEAVY_TESTS:-}" rc counts judged reused refused passes red_cells name
  local log="$logs/checker-quick.log"
  local -a red_tests=()
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "checker-quick-skipped" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=checker-quick-skipped：不跑 checker 档快档那一步"; return 0
  fi
  # 起那一道的命令词（run_capped 收到的）与这一步的内存上限（文件头「内存上限」）；上限先算好，下面各处出路里的命令才带得上
  CHECKER_QUICK_WORDS=(env -u MERGE_AND_VERIFY_BREAK bash "$CHECKER_QUICK_STAGE" --check "$CHECKER_QUICK_CELLS" "$quick_root")
  checker_quick_resolve_memory
  # 弄坏开关 checker-quick-elsewhere-run：--peer 与 --copy 时照样往下走
  if ((PEER_MODE)) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != "checker-quick-elsewhere-run" ]]; then
    CHECKER_QUICK_STATE="本次未跑（--peer）"
    echo "  - checker 档快档本次未跑：带了 --peer，编译与测试在第二台，checker 档快档按条复用的判定库在本机仓的 git 目录里，这一步不交过去（不记红、不算过）"
    echo "  → 怎么办：要验这一步就去掉 --peer、带 --skip-apply 在本机重跑，或单跑那两格：$(checker_quick_rerun_command "$CHECKER_QUICK_CELLS" "$quick_root")"
    return 0
  fi
  if ((copy_mode)) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != "checker-quick-elsewhere-run" ]]; then
    CHECKER_QUICK_STATE="本次未跑（--copy）"
    echo "  - checker 档快档本次未跑：带了 --copy，副本 $tree 没有 git 目录（source-stamp.py sync 不拷 .git），快档按条复用的判定库读不到（不记红、不算过）"
    echo "  → 怎么办：补丁打进主工作区之后不带 --copy 重跑（打过的带 --skip-apply），或在主工作区单跑那两格：$(checker_quick_rerun_command "$CHECKER_QUICK_CELLS" "$root")"
    return 0
  fi
  # 弄坏开关 checker-quick-always-run：不看补丁碰没碰，一律照跑
  if [[ "$CHECKER_QUICK_AFFECTED" == 0 && "${MERGE_AND_VERIFY_BREAK:-}" != "checker-quick-always-run" ]]; then
    CHECKER_QUICK_STATE="本次未跑（补丁没碰 checker 档包与它的依赖）"
    echo "  - checker 档快档本次未跑：$CHECKER_QUICK_REASON（不记红、不算过）"
    return 0
  fi
  # 弄坏开关 checker-quick-prefix-unchecked：没带前缀也照起
  if [[ "$heavy" != user-request && "$heavy" != commit && "${MERGE_AND_VERIFY_BREAK:-}" != "checker-quick-prefix-unchecked" ]]; then
    CHECKER_QUICK_STATE="红（起它的命令没带前缀，没起）"
    echo "  ✗ checker 档快档那一步要起它的命令带 SINGLEFS_HEAVY_TESTS=user-request（提交时 =commit），这一趟是「${heavy:-没设}」，没起（$CHECKER_QUICK_REASON）"
    echo "  → 怎么办：整条 merge-and-verify.sh 命令前面加 SINGLEFS_HEAVY_TESTS=user-request 重跑（补丁这一趟已打进树的带 --skip-apply）；或只补这一步：$(checker_quick_rerun_command "$CHECKER_QUICK_CELLS" "$quick_root")"
    return 1
  fi
  echo "  checker 档快档（$CHECKER_QUICK_REASON）：起 $(basename "$CHECKER_QUICK_STAGE") --check $CHECKER_QUICK_CELLS，只现判单元键对不上的测试函数、键相同的复用（内存上限 $CHECKER_QUICK_MEMORY：$CHECKER_QUICK_MEMORY_HOW；线程上限 $THREADS）…"
  rm -f -- "$log"
  RUN_CAPPED_MEMORY="$CHECKER_QUICK_MEMORY" run_capped "$log" "${CHECKER_QUICK_WORDS[@]}"; rc=$?
  if refused_step checker-quick "$rc"; then
    [[ "$LOCAL_REFUSED_STEP" == checker-quick ]] || { CHECKER_QUICK_STATE="本次未跑（前面一步本机内存排不上）"; return 0; }
    CHECKER_QUICK_STATE="红（本机内存排不上）"; return 1
  fi
  if ((rc == GPU_MEMORY_HIT_EXIT)); then
    CHECKER_QUICK_STATE="红（显存不够）"
    echo "  ✗ checker 档快档那一趟显存不够（内存包装退 $GPU_MEMORY_HIT_EXIT），结果不算数（日志 $log）"
    echo "  → 怎么办：nvidia-smi 看谁在用卡，等它跑完再只补这一步：$(checker_quick_rerun_command "$CHECKER_QUICK_CELLS" "$quick_root")"
    return 1
  fi
  # 撞了这一步自己的内存上限（包装退 250、日志里有「撞了内存上限」）：结果不算数，不是代码红。弄坏开关 quick-memory-oom-as-red：照改前当红往下判
  if ((rc == MEMORY_CAP_HIT_EXIT)) && grep -q '撞了内存上限' "$log" 2>/dev/null && [[ "${MERGE_AND_VERIFY_BREAK:-}" != "quick-memory-oom-as-red" ]]; then
    CHECKER_QUICK_STATE="本次未跑（撞了内存上限 $CHECKER_QUICK_MEMORY）"; CHECKER_QUICK_MEMORY_HIT=1
    checker_quick_next_memory
    echo "  - 本次未跑：checker 档快档撞了内存上限 $CHECKER_QUICK_MEMORY（内存包装退 $rc；上限取法：$CHECKER_QUICK_MEMORY_HOW；日志 $log、错误警告 $logs/checker-quick.problems.log）：这一趟的结果不算数，不记代码红、不算过，整条退 $NOT_RUN_EXIT"
    if [[ "$CHECKER_QUICK_NEXT_MEMORY" == "-" ]]; then
      echo "  → 怎么办：$CHECKER_QUICK_NEXT_NOTE；降线程上限（--threads）后只重跑这一步，或交主 agent 判断；不是代码红，别照日志改测试"
    else
      echo "  → 怎么办：只重跑这一步、上限加大一档（$CHECKER_QUICK_MEMORY → $CHECKER_QUICK_NEXT_MEMORY：$CHECKER_QUICK_NEXT_NOTE）：$(checker_quick_rerun_command "$CHECKER_QUICK_CELLS" "$quick_root" "$CHECKER_QUICK_NEXT_MEMORY" "合入后验证 checker 档快档撞了内存上限 $CHECKER_QUICK_MEMORY，加大一档上限只重跑这一步：键相同的照复用，只现判没复用的")"
    fi
    return 0
  fi
  if ((rc == 77)); then
    CHECKER_QUICK_STATE="本次未跑（那一道退 77）"
    echo "  - checker 档快档本次未跑：$(basename "$CHECKER_QUICK_STAGE") 退 77（$(grep -E '^ *!' "$log" | tail -n 1 | sed 's/^ *//')），不记红、不算过"
    return 0
  fi
  counts="$(sed -n 's/^ *· \(快档\|feature-gated-tests \)按条复用：.*现判 \([0-9]*\)、复用 \([0-9]*\)、拒算 \([0-9]*\).*/\2 \3 \4/p' "$log")"
  read -r judged reused refused passes < <(awk 'NF { judged += $1; reused += $2; refused += $3; passes++ } END { print judged + 0, reused + 0, refused + 0, passes + 0 }' <<< "$counts")
  # 弄坏开关 checker-quick-sample-exit-anywhere：退 3 不看根是不是样本根一律算过；checker-quick-red-ignored：那一道红了也报过
  if { ((rc == 0)) || { ((rc == 3)) && { [[ -e "$quick_root/.crash-point-replay-sample-tools/quick-tier" ]] || [[ "${MERGE_AND_VERIFY_BREAK:-}" == "checker-quick-sample-exit-anywhere" ]]; }; }; } && ((passes > 0)) \
     || [[ "${MERGE_AND_VERIFY_BREAK:-}" == "checker-quick-red-ignored" ]]; then
    CHECKER_QUICK_STATE="过（现判 $judged、复用 $reused、内存上限 $CHECKER_QUICK_MEMORY）"
    echo "  ✓ checker 档快档过（$(basename "$CHECKER_QUICK_STAGE") 的 ${CHECKER_QUICK_CELLS//,/、} 两格退 $rc$( ((rc == 3)) && echo '，样本档：假 cargo 走的流程')；现判 $judged、复用 $reused、拒算 $refused，读到 $passes 趟按条复用的那一行；内存上限 $CHECKER_QUICK_MEMORY，$CHECKER_QUICK_MEMORY_HOW；日志 $log）"
    grep -m 1 '^ *· 逐格：' "$log" | sed 's/^ */    /'
    return 0
  fi
  mapfile -t red_tests < <(sed -n 's/^ *test \(.*\) \.\.\. FAILED$/\1/p' "$log" | LC_ALL=C sort -u)
  red_cells="$(sed -n 's/.*✗ 快档有 [0-9]* 格判红：\(.*\)（各自的出路在上面）.*/\1/p' "$log" | tail -n 1)"
  CHECKER_QUICK_STATE="红（${#red_tests[@]} 条测试函数）"
  echo "  ✗ checker 档快档红（$(basename "$CHECKER_QUICK_STAGE") 退 $rc$( ((rc == 3)) && echo '：退 3 只在样本根上算过，交给它的根不是样本根')$( ((passes == 0)) && echo '，日志里读不到按条复用那一行（现判、复用多少条判不了）')；红的格：${red_cells:-认不出}；日志 $log、错误警告 $logs/checker-quick.problems.log）：FAILED 的测试函数 ${#red_tests[@]} 条"
  for name in ${red_tests[@]+"${red_tests[@]}"}; do echo "      $name"; done   # gate-lint:detail
  grep -E '^ *红的 |^ *error: test failed, to rerun pass' "$log" | sed 's/^ *//' | LC_ALL=C sort -u | head -n 40 | sed 's/^/      /'   # gate-lint:detail
  ((${#red_tests[@]})) || grep -E '^ *✗' "$log" | head -n 5 | sed 's/^ */      /'   # gate-lint:detail
  echo "  → 怎么办：FAILED 的那几条测试函数并进写段的下一批派 implementation-writer 改；改完只重跑那几格：$(checker_quick_rerun_command "$(tr ' ' ',' <<< "${red_cells:-${CHECKER_QUICK_CELLS//,/ }}")" "$quick_root")（键相同的照复用：判绿记过库的不重跑，只现判上面红的这几条与改后键变了的）"
  return 1
}

verify() {
  local logs="$1"; shift
  local -a dirs=("$@")
  local dir rc applied=0 warned=0 failed_tests reds=0 test_count untested_text slow_count slow_text=""
  local -a clippy_lints=() patch_arguments=() compile_logs=() test_command=() fmt_command=() clippy_command=() build_command=() peer_steps=() slow_tail=()
  mkdir -p "$logs"
  rm -f -- "$logs/test.log" "$logs/build.log"   # 重编核对只认这一趟的编译输出（日志目录可能是调用方给的旧目录）
  for dir in ${CHECK_PATCH_DIRS[@]+"${CHECK_PATCH_DIRS[@]}"}; do patch_arguments+=(--patch-dir "$dir"); done
  if ! read_check_sh "$CHECK_SH"; then
    if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "lint-table-optional" ]]; then
      echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=lint-table-optional：$CHECK_SH_PROBLEM，照常往下验"
    else
      die "check.sh 读不出、或与这里对不上：$CHECK_SH_PROBLEM；一个补丁都没打、没编" \
        "上游 check.sh 的 lint 表或那四条 cargo 写法变了：先照它改这份脚本的 CHECK_SH_COMMANDS 与 verify()，让合入后验证与提交时的判据仍是同一套" 1
    fi
  fi
  clippy_lints=(${DISCIPLINE_LINTS[@]+"${DISCIPLINE_LINTS[@]}"})
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "drop-discipline-lint" && ${#clippy_lints[@]} -ge 2 ]]; then
    clippy_lints=("${clippy_lints[@]:0:${#clippy_lints[@]}-2}")
  fi
  for dir in "${dirs[@]}"; do
    [[ -d "$dir" ]] || die "补丁目录不在：$dir" "给实现员交回的补丁目录（里面有 crates.patch 或 mutations-append.tsv）"
    python3 "$APPLY" "$dir" ${COPY_TREE:+--root "$COPY_TREE"} > "$logs/apply-$(basename "$dir").log" 2>&1; rc=$?
    case "$rc" in
      0) applied=$((applied+1)); echo "  打上：$dir" ;;
      1) applied=$((applied+1)); warned=$((warned+1)); echo "  打上、但 code-source-discipline 红：$dir（日志 $logs/apply-$(basename "$dir").log）" ;;
      *) die "补丁核不过、一个字没改：$dir（apply-writer-patch.py 退 $rc，日志 $logs/apply-$(basename "$dir").log）" "按那份日志给的下一步让实现员改补丁再交" 2 ;;
    esac
  done
  # 补丁改过、落在包里的文件刷到此刻（文件头「改过的包都重编了没有」）：再验同一批补丁时源码戳没变，不刷就不会重编
  if ((${#patch_arguments[@]} > 0)); then
    if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "no-touch-patched" ]]; then
      echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=no-touch-patched：编之前不把补丁改过的文件刷到此刻"
    elif python3 "$REBUILT" touch --tree "$tree" "${patch_arguments[@]}" > "$logs/touch-patched.log" 2>&1; then
      echo "$(tail -1 "$logs/touch-patched.log")"
    else
      die "编之前没把补丁改过的文件刷到此刻，没编（日志 $logs/touch-patched.log）" "照那份日志修好再跑（补丁目录在不在、树可不可写）" 1
    fi
  fi
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "no-source-stamp" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=no-source-stamp：编之前不刷源码戳"
  elif python3 "$STAMP" refresh "$tree" > "$logs/source-stamp.log" 2>&1; then
    echo "  源码戳：$(grep '^SOURCE_STAMP ' "$logs/source-stamp.log" | tail -1)"
  else
    die "编之前没刷成源码戳，没编（日志 $logs/source-stamp.log）" "照那份日志修好再跑；刷不成就别复用这个编译目录（设 CARGO_TARGET_DIR 换一个新的）" 1
  fi
  test_packages "$tree" || die "挑不出要测的包：$TEST_PACKAGES_PROBLEM；没编" \
    "工作区成员要逐个写出目录、各有 [package] name，checker 档包 $CHECKER_TIER_PACKAGE 要在成员里（这里靠它的名字把它排掉）；改了名就同步改这份脚本的 CHECKER_TIER_PACKAGE" 1
  select_test_packages "$tree" || die "算不出这一批补丁改到哪几个包：$TEST_SELECTION_PROBLEM；没编" \
    "看补丁目录在不在、crates.patch 与工作区成员的 Cargo.toml 读不读得了；要照旧全跑就加 --all-packages" 1
  untested_text=""
  if ((${#UNTESTED_PACKAGES[@]})); then untested_text="$(printf '%s、' "${UNTESTED_PACKAGES[@]}")"; untested_text="；本次未跑：补丁没改到 ${#UNTESTED_PACKAGES[@]} 个包：${untested_text%、}"; fi
  # 四条 cargo 只写这一份：本机逐条经包装跑，--peer 时一趟交给第二台跑（文件头「--peer」）
  # 测试二进制的参数：--include-ignored，harness 慢档一起跑（文件头 test 一段「慢档一起跑」）
  ((${#SLOW_TIER_ARGUMENTS[@]} == 0)) || slow_tail=(-- "${SLOW_TIER_ARGUMENTS[@]}")
  if [[ "${MERGE_AND_VERIFY_TEST_TARGETS:-}" == serial || "${MERGE_AND_VERIFY_BREAK:-}" == "tests-one-binary-at-a-time" ]]; then
    test_command=(cargo test --offline --no-fail-fast ${TEST_PACKAGES[@]+"${TEST_PACKAGES[@]}"} ${slow_tail[@]+"${slow_tail[@]}"})
  else
    # 按测试目标并行起（文件头 test 一段）；--peer 时第二台那棵副本里同一相对路径的那一份
    test_command=(python3 "$( ((PEER_MODE)) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != "peer-ignored" ]] && echo scripts/gate/test-targets-parallel.py || echo "$TEST_TARGETS_PARALLEL")"
      --jobs "$THREADS" -- --offline --no-fail-fast ${TEST_PACKAGES[@]+"${TEST_PACKAGES[@]}"} ${slow_tail[@]+"${slow_tail[@]}"})
    # 只剩没跑完的几条与文档测试那一段，它往这里声明串行；调用方设了照调用方的
    [[ -n "${SINGLEFS_SERIAL_DECLARATIONS:-}" ]] || export SINGLEFS_SERIAL_DECLARATIONS="$logs/serial-declarations"
  fi
  fmt_command=(cargo fmt --all -- --check)
  clippy_command=(cargo clippy --offline --all-targets --all-features -- -D warnings ${clippy_lints[@]+"${clippy_lints[@]}"})
  build_command=(cargo build --offline --all-targets)
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "peer-ignored" ]]; then PEER_MODE=0; fi
  # RocksDB 预编库的取值（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」）：clippy --all-features 与 build --all-targets 编工作区全部成员，连同 checker 档包（编到 librocksdb-sys）。
  #   本机起四条 cargo 之前判一次这台装了、此刻进程环境的取值对得上（补丁打完之后判：补丁改了 Cargo.lock，键跟着变），判红退 1、一条 cargo 都不起。
  #   --peer 时四条都在第二台跑，第二台的取值由 multi-host-run.sh --peer 在第二台上判，本机不判。树里没有依赖 rocksdb 的清单（自证的样本工作区）、
  #   PATH 上是带标记的假 cargo，entry 报不判。弄坏开关 rocksdb-entry-unchecked 下不判（自证「起 cargo 之前 RocksDB 预编库的取值缺了」那一格判错）
  if ((PEER_MODE == 0)) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != "rocksdb-entry-unchecked" ]] && ! python3 "$root/scripts/hosts/rocksdb-prebuilt.py" entry --root "$tree"; then
    die "起 cargo 之前 RocksDB 预编库的取值没判过（上面 rocksdb-prebuilt.py entry 的红句），一条 cargo 都没起" \
      "照上面的出路修（开新会话，或命令前 eval \"\$(python3 scripts/hosts/rocksdb-prebuilt.py env)\"）再跑同一条命令；本机没装而第二台装了的，加 --peer 交给第二台" 1
  fi
  if ((PEER_MODE)); then
    ((${#TEST_PACKAGES[@]} == 0)) || peer_steps+=(test "${test_command[@]}" ::)
    peer_steps+=(fmt "${fmt_command[@]}" :: clippy "${clippy_command[@]}" ::)
    [[ "${MERGE_AND_VERIFY_BREAK:-}" == "skip-build" ]] || peer_steps+=(build "${build_command[@]}" ::)
    run_peer_steps "$logs" "${peer_steps[@]}"
  fi
  rc=0; test_count=0; failed_tests=""
  if ((${#TEST_PACKAGES[@]})); then
    echo "  编译并跑 ${TEST_PACKAGES[*]} 的全部测试二进制（工作区成员减掉 $CHECKER_TIER_PACKAGE，${TEST_SELECTION_NOTE}${untested_text}；$([[ "${test_command[0]}" == cargo ]] && echo '一条 cargo test 逐个二进制跑' || echo '按测试目标并行起')$( ((${#slow_tail[@]})) && echo '；带 --include-ignored，harness 慢档一起跑')；内存上限 $MEMORY、线程上限 $THREADS）…"
    run_capped "$logs/test.log" "${test_command[@]}"; rc=$?
    test_count="$(awk 'match($0, /^test result: [A-Za-z]+\. [0-9]+ passed; [0-9]+ failed/) { split($0, f, " "); passed += f[4]; failed += f[6] } END { print passed + failed + 0 }' "$logs/test.log")"
    failed_tests="$(sed -n 's/^test \(.*\) \.\.\. FAILED$/\1/p' "$logs/test.log")"
    # 慢档跑了几条（文件头 test 一段「慢档一起跑」）：test-targets-parallel.py 末行的 slow=
    if ((${#slow_tail[@]})); then
      slow_count="$(sed -n 's/^TEST_TARGETS_PARALLEL .* slow=\([0-9?-]*\)$/\1/p' "$logs/test.log" | tail -n 1)"
      case "$slow_count" in
        [0-9]*) slow_text="，其中慢档 $slow_count 条" ;;
        "?") slow_text="，慢档带上了、条数数不出（列慢档的那一趟退了非 0）" ;;
        *) slow_text="，慢档带上了、条数没数（一条 cargo test 逐个二进制跑时数不出）" ;;
      esac
    fi
  fi
  if ((${#TEST_PACKAGES[@]} == 0)); then
    echo "  - 测试本次未跑：这一批补丁一个要测的包都没改到（${TEST_SELECTION_NOTE}；checker 档包 $CHECKER_TIER_PACKAGE 归提交时）${untested_text}"
  elif refused_step test "$rc"; then
    [[ "$LOCAL_REFUSED_STEP" != test ]] || reds=$((reds+1))
  elif [[ $rc -eq $GPU_MEMORY_HIT_EXIT && "${MERGE_AND_VERIFY_BREAK:-}" != "gpu-memory-as-test-red" ]]; then
    reds=$((reds+1))
    echo "  ✗ cargo test 那一趟显存不够（内存包装退 $GPU_MEMORY_HIT_EXIT：起止之间内核报了 NV_ERR_NO_MEMORY），这一趟的测试结果不算数（日志 $logs/test.log、错误警告 $logs/test.problems.log）"
    echo "  → 怎么办：nvidia-smi 看谁在用卡，等它跑完再重跑这一条 merge-and-verify.sh；不是代码红，别照日志里的 FAILED 改钉值"
  elif [[ -n "$failed_tests" && "${MERGE_AND_VERIFY_BREAK:-}" != "ignore-red" ]]; then
    reds=$((reds+1))
    echo "  ✗ 红的测试 $(printf '%s\n' "$failed_tests" | wc -l) 条："
    printf '%s\n' "$failed_tests" | sed 's/^/      /'   # gate-lint:detail
    echo "  → 怎么办：只重跑红的这几条：python3 scripts/gate/rerun-failed-tests.py $logs/test.log -p <包>；照实现员报告里的算法改钉值，改完再只跑这几条（错误警告日志：$logs/test.problems.log，行前带全量日志里的行号）"
  elif [[ $rc -ne 0 && "${MERGE_AND_VERIFY_BREAK:-}" != "ignore-red" ]]; then
    reds=$((reds+1))
    echo "  ✗ cargo test 退 $rc 而一条 FAILED 都没抓到（编不过、或包装退 250–254）：$(tail -3 "$logs/test.log" | tr '\n' ' ')"
    echo "  → 怎么办：看 $logs/test.problems.log（错误警告，行前带全量日志里的行号）与 $logs/test.log（全量）；退 250–254 是内存包装自己的结局，按 run-with-memory-cap.sh 文件头处理"
  elif [[ "$test_count" == 0 && "${MERGE_AND_VERIFY_BREAK:-}" != "zero-tests-green" ]]; then
    reds=$((reds+1))
    echo "  ✗ cargo test 退 0，却一条测试都没跑（test result 行里 passed 与 failed 加起来是 0；日志 $logs/test.log）"
    echo "  → 怎么办：看那份日志里编了哪些包、有没有 test result 行；一条都没跑的「全绿」与跑过了长得一样（check.sh 同一条）"
  else
    echo "  ✓ 测试全绿（$(grep -c '^test result: ok' "$logs/test.log") 个测试二进制，共 $test_count 条测试${slow_text}；跑了 $((${#TEST_PACKAGES[@]} / 2)) 个包${untested_text}）"
  fi
  run_capped "$logs/fmt.log" "${fmt_command[@]}"; rc=$?
  if peer_step_not_run fmt; then report_peer_step_not_run fmt rustfmt
  elif refused_step fmt "$rc"; then [[ "$LOCAL_REFUSED_STEP" != fmt ]] || reds=$((reds+1))
  elif [[ $rc -ne 0 ]]; then reds=$((reds+1)); echo "  ✗ cargo fmt --check 红（日志 $logs/fmt.log、错误警告 $logs/fmt.problems.log）"; echo "  → 怎么办：cargo fmt --all，再看 diff 只动了格式"; else echo "  ✓ fmt 过"; fi
  run_capped "$logs/clippy.log" "${clippy_command[@]}"; rc=$?
  if peer_step_not_run clippy; then report_peer_step_not_run clippy clippy
  elif refused_step clippy "$rc"; then [[ "$LOCAL_REFUSED_STEP" != clippy ]] || reds=$((reds+1))
  elif [[ $rc -ne 0 ]]; then reds=$((reds+1)); echo "  ✗ clippy 红（日志 $logs/clippy.log、错误警告 $logs/clippy.problems.log）"; echo "  → 怎么办：按日志逐条改，不加 allow 绕过"; else echo "  ✓ clippy 过（带 check.sh 的编码纪律 lint $((${#clippy_lints[@]} / 2)) 条）"; fi
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "skip-build" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=skip-build：不跑 cargo build"
  else
    run_capped "$logs/build.log" "${build_command[@]}"; rc=$?
    if refused_step build "$rc"; then [[ "$LOCAL_REFUSED_STEP" != build ]] || reds=$((reds+1))
    elif [[ $rc -ne 0 ]]; then reds=$((reds+1)); echo "  ✗ cargo build --all-targets 红（日志 $logs/build.log、错误警告 $logs/build.problems.log）"; echo "  → 怎么办：从日志里第一条 rustc 报错改起，后面的多半是它的连锁反应"; else echo "  ✓ build 过"; fi
  fi
  # 改过的包这一趟都重编了没有（文件头）：test 与 build 两趟的编译输出合起来核
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "skip-rebuild-check" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=skip-rebuild-check：不核改过的包重编了没有"
  elif ((PEER_MODE)); then
    echo "  - 重编核对本次未判：编译在第二台（--peer），编译目录不在本机"
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
  # checker 档快档：编译与测试之后（文件头「checker 档快档」）
  checker_quick_step "$logs" || reds=$((reds+1))
  echo "  汇总：打上 ${applied} 个补丁目录（其中 code-source-discipline 红 $warned 个），验证红 $reds 项${PEER_NOT_RUN_STEPS[0]+，本次未跑 ${#PEER_NOT_RUN_STEPS[@]} 步（${PEER_NOT_RUN_STEPS[*]}：第二台工具链缺组件，不算绿，装好再带 --peer 重跑）}${CHECKER_QUICK_STATE:+，checker 档快档 $CHECKER_QUICK_STATE}，日志在 $logs"
  if [[ $reds -ne 0 || $warned -ne 0 ]]; then return 1; fi
  # 其余都绿、只有 checker 档快档那一步撞了自己的内存上限：本次未跑，不算全绿（文件头「内存上限」）。弄坏开关 quick-memory-oom-exits-zero：照退 0
  if ((CHECKER_QUICK_MEMORY_HIT)) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != "quick-memory-oom-exits-zero" ]]; then return "$NOT_RUN_EXIT"; fi
  return 0
}

selftest() {
  local scratch fake_cargo_dir failures=0 checked=0 out rc expected_lints expected_lint_count check_case
  unset MERGE_AND_VERIFY_KEEP_COPY   # 留现场那一格自己设；调用方设了不能带进别的格
  unset SINGLEFS_MULTI_HOST_CONFIG   # --copy 那几格从要验的树取多机配置（文件头 --copy「副本不带 .git」）；调用方设了的不带进来
  # 经内存包装起时摘掉外层包装的环境（文件头「自证经内存包装起」）：TMPDIR 是外层包装建的那一份时不用它，临时目录照不经包装那样建在 /tmp 下；两个标记摘掉
  if [[ -n "${RUN_WITH_MEMORY_CAP_OUTER_UNIT:-}" && "${TMPDIR:-}" == */singlefs-run-with-memory-cap-* ]]; then unset TMPDIR; fi
  unset RUN_WITH_MEMORY_CAP_OUTER_UNIT RUN_WITH_MEMORY_CAP_OUTER_CAP
  # 测试那一步：别的格照改前一条 cargo test（它们核假 cargo 收到的那一条），只有「按测试目标并行起」那一格不设它（文件头 test 一段）
  export MERGE_AND_VERIFY_TEST_TARGETS=serial
  # 核挑包的那几格只看 -p 那几个包：测试二进制的参数（-- --include-ignored，文件头 test 一段「慢档一起跑」）去掉再比，带没带它归「慢档一起跑」那一格
  test_calls() { sed 's/ -- --include-ignored$//' "$1" 2>/dev/null; }
  scratch="$(mktemp -d)"; fake_cargo_dir="$scratch/bin"; mkdir -p "$fake_cargo_dir" "$scratch/patch-a" "$scratch/patch-b"
  # checker 档快档那一步的内存上限（文件头「内存上限」）：峰值表指到一份不存在的（调用方本机的表不进自证），本机 slice 总上限读成 64G（不看这台的机器、不起 systemctl）；各格要表、要别的总上限的自己再设
  unset RUN_WITH_MEMORY_CAP_KEY
  export RUN_WITH_MEMORY_CAP_PEAKS="$scratch/no-memory-peaks.tsv" MERGE_AND_VERIFY_SLICE_TOTAL=64G
  # patch-a 改的是 demo 里的一份数据文件（不是 .rs：重编核对不核它，测试照挑 demo）
  printf -- '--- a/crates/demo/src/table.txt\n+++ b/crates/demo/src/table.txt\n@@ -1 +1 @@\n-1\n+2\n' > "$scratch/patch-a/crates.patch"
  # 主工作区合入锁：每一格的锁文件都是临时目录里这一份（不碰仓里 git 目录那一把；按工作树分开那一格另在临时仓里用默认位置）；排队时每 0.3 秒报一次、每 0.05 秒看一次；调用方的持有者变量不带进来
  # 排队最多等 30 秒：锁永远拿不到（弄坏开关、或哪一格没放锁）时到点退 1、那一格红，不挂住
  export SINGLEFS_WORKSPACE_LOCK_FILE="$scratch/main-workspace.lock" SINGLEFS_WORKSPACE_LOCK_REPORT_SECONDS=0.3 SINGLEFS_WORKSPACE_LOCK_POLL_SECONDS=0.05 SINGLEFS_WORKSPACE_LOCK_GIVE_UP_SECONDS=30
  unset SINGLEFS_WORKSPACE_LOCK_HOLDER
  # checker 档快档那一步（文件头「checker 档快档」）：别的格都换上假的那一道、带上前缀；那一步自己的几格另换回真的门禁，交它的样本根（假 cargo）
  cat > "$scratch/fake-checker-quick.sh" <<'FAKE'
#!/usr/bin/env bash
# 假的 checker-tier-crash-point-replay：参数记进 FAKE_CHECKER_QUICK_CALLS（设了的话），照真的那一道打一行按条复用与一行逐格，退 FAKE_CHECKER_QUICK_EXIT（没设是 0）
[[ -z "${FAKE_CHECKER_QUICK_CALLS:-}" ]] || printf '%s\n' "$*" >> "$FAKE_CHECKER_QUICK_CALLS"
echo "  · 快档按条复用：checker 档包 2 条编得进的测试函数，现判 0、复用 2、拒算 0；这一组特性下不编 0 条"
echo "  · 逐格：crash-case-replay：绿；feature-gated-tests：无对象可判"
exit "${FAKE_CHECKER_QUICK_EXIT:-0}"
FAKE
  export SINGLEFS_HEAVY_TESTS=user-request MERGE_AND_VERIFY_CHECKER_QUICK_STAGE="$scratch/fake-checker-quick.sh"
  unset MERGE_AND_VERIFY_CHECKER_QUICK_ROOT FAKE_CHECKER_QUICK_CALLS FAKE_CHECKER_QUICK_EXIT
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
        if [[ " $* " == *" --list "* ]]; then echo "tests::slow_case: test"; exit 0; fi   # test-targets-parallel.py 数慢档那一趟：每个目标列一条
        [[ "${FAKE_CARGO_MODE:-}" == "empty" ]] && exit 0
        printf 'running 2 tests\ntest tests::adds ... ok\ntest tests::subtracts_in_%s ... FAILED\n\ntest result: FAILED. 1 passed; 1 failed\n' "${FAKE_CARGO_MODE:-x}"; [[ "${FAKE_CARGO_MODE:-}" == "green" ]] && exit 0 || exit 101 ;;
  fmt|clippy) exit 0 ;;
esac
FAKE
  [[ "${FAKE_CARGO_MODE:-}" == "" ]] || true
  chmod +x "$fake_cargo_dir/cargo"
  # 刷源码戳与编译都在临时树里（MERGE_AND_VERIFY_TREE），不碰本仓的源文件与编译目录
  mkdir -p "$scratch/tree/crates/demo/src"; printf 'pub fn f() {}\n' > "$scratch/tree/crates/demo/src/lib.rs"
  # 样本工作区两个成员：demo 与 checker 档包；cargo test 只该挑 demo
  write_workspace() {   # $1 树，$2 checker 档那个成员的包名
    mkdir -p "$1/crates/demo" "$1/crates/singlefs-checker-tier"
    printf '[workspace]\nmembers = ["crates/demo", "crates/singlefs-checker-tier"]\n' > "$1/Cargo.toml"
    printf '[package]\nname = "demo"\n' > "$1/crates/demo/Cargo.toml"
    printf '[package]\nname = "%s"\n' "$2" > "$1/crates/singlefs-checker-tier/Cargo.toml"
  }
  write_workspace "$scratch/tree" singlefs-checker-tier
  run_case() { PATH="$fake_cargo_dir:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-$1" \
    MERGE_AND_VERIFY_TREE="$scratch/tree" env -u CARGO_TARGET_DIR bash "$0" "${@:2}" > "$out" 2>&1; rc=$?; }
  out="$scratch/out1.txt"; run_case one "$scratch/patch-a"; checked=$((checked+1))
  if [[ $rc -ne 1 ]] || ! grep -q 'subtracts_in_x' "$out" || ! grep -q 'rerun-failed-tests.py' "$out"; then
    echo "  ✗ 自检：一条测试红时应当退 1、列出那条测试与只重跑它的命令，实际退出码 $rc：$(tail -4 "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 每一步的日志出两份（run_capped 经 scripts/hosts/two-logs.sh）：test.log 是全量，test.problems.log 带行号收那条 FAILED，红的出路点名错误警告日志的路径
  checked=$((checked+1))
  if [[ ! -f "$scratch/logs-one/test.log" ]] || ! grep -qE '^[0-9]+:test tests::subtracts_in_x \.\.\. FAILED$' "$scratch/logs-one/test.problems.log" 2>/dev/null \
     || ! grep -qF "错误警告日志：$scratch/logs-one/test.problems.log" "$out"; then
    echo "  ✗ 自检：一条测试红时每一步的日志应当出两份（test.log 全量；test.problems.log 带行号收那条 FAILED），红的出路点名错误警告日志的路径，实际日志目录里有 $(ls "$scratch/logs-one" 2>/dev/null | tr '\n' ' ')：$(tail -3 "$scratch/logs-one/test.problems.log" 2>&1 | tr '\n' '|')"; failures=1; fi   # gate-lint:detail
  out="$scratch/out2.txt"; run_case two "$scratch/patch-a" "$scratch/patch-b"; checked=$((checked+1))
  if [[ $rc -ne 2 ]] || ! grep -q '补丁核不过' "$out"; then
    echo "  ✗ 自检：apply 退 2 时应当整批停、退 2，实际 $rc：$(tail -2 "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # --copy 测试红（假 cargo 这时还报一条 FAILED）：退 1，自己建的副本与编译目录照样删掉
  out="$scratch/out-copy-red.txt"; run_case copy-red --copy "$scratch/patch-a"; checked=$((checked+1))
  if [[ $rc -ne 1 ]] || [[ -e "$scratch/logs-copy-red/repo" || -e "$scratch/logs-copy-red/target" ]] || ! grep -q '删了副本' "$out"; then
    echo "  ✗ 自检：--copy 测试红时也应当删掉副本与编译目录，实际退出码 $rc，日志目录里还有 $(ls "$scratch/logs-copy-red" 2>/dev/null | tr '\n' ' ')：$(grep -E '✗|删了|建副本' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 起 cargo 之前判 RocksDB 预编库的取值（文件头「RocksDB 预编库的取值」）：样本工作区的 checker 档包依赖 rocksdb、进程环境里四个取值是空的，
  #   PATH 上的假 cargo 不带 entry 认的标记 ⇒ 退 1、假 cargo 一次都没起、输出里有 rocksdb-prebuilt entry 的红句（别的格样本工作区的 checker 档包不依赖 rocksdb，entry 报不判）
  local rocksdb_tree="$scratch/tree-rocksdb"
  mkdir -p "$rocksdb_tree/crates/demo/src"; printf 'pub fn f() {}\n' > "$rocksdb_tree/crates/demo/src/lib.rs"
  write_workspace "$rocksdb_tree" singlefs-checker-tier
  printf '[dependencies]\nrocksdb = { version = "=0.25.0", optional = true }\n' >> "$rocksdb_tree/crates/singlefs-checker-tier/Cargo.toml"
  out="$scratch/out-rocksdb.txt"; : > "$scratch/calls-rocksdb.txt"; checked=$((checked+1))
  PATH="$fake_cargo_dir:$PATH" FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-rocksdb.txt" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-rocksdb" MERGE_AND_VERIFY_TREE="$rocksdb_tree" ROCKSDB_LIB_DIR= ROCKSDB_STATIC= SNAPPY_LIB_DIR= SNAPPY_STATIC= \
    env -u CARGO_TARGET_DIR bash "$0" --skip-apply > "$out" 2>&1; rc=$?
  if [[ $rc -ne 1 || -s "$scratch/calls-rocksdb.txt" ]] || ! grep -qF 'rocksdb-prebuilt entry：' "$out"; then
    echo "  ✗ 自检：起 cargo 之前 RocksDB 预编库的取值缺了（checker 档包依赖 rocksdb、进程环境里没有取值）应当退 1、一条 cargo 都不起，实际退出码 $rc、假 cargo 收到 $(wc -l < "$scratch/calls-rocksdb.txt") 条：$(tail -3 "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 假 cargo 全绿的一趟：把 FAILED 行换掉
  sed -i.bak 's/ \.\.\. FAILED/ ... ok/; s/FAILED\. 1 passed; 1 failed/ok. 2 passed; 0 failed/' "$fake_cargo_dir/cargo" && rm -f "$fake_cargo_dir/cargo.bak"
  out="$scratch/out3.txt"; FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-three.txt" run_case three "$scratch/patch-a"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -q '测试全绿' "$out"; then
    echo "  ✗ 自检：全绿时应当退 0 并说全绿，实际 $rc：$(tail -3 "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # --copy 全绿：补丁打进副本（apply 收到 --root <日志目录>/repo）、源码戳记在 <日志目录>/target，收尾两份都删掉、打一行删了什么
  out="$scratch/out-copy-green.txt"; FAKE_CARGO_MODE=green run_case copy-green --copy "$scratch/patch-a"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || [[ -e "$scratch/logs-copy-green/repo" || -e "$scratch/logs-copy-green/target" ]] || ! grep -q '删了副本' "$out" \
     || ! grep -qxF "root=$scratch/logs-copy-green/repo" "$scratch/logs-copy-green/apply-patch-a.log" 2>/dev/null \
     || ! grep -qF "stamp=$scratch/logs-copy-green/target/" "$out"; then
    echo "  ✗ 自检：--copy 全绿时应当在副本里打补丁、收尾删掉副本与编译目录，实际退出码 $rc，日志目录里还有 $(ls "$scratch/logs-copy-green" 2>/dev/null | tr '\n' ' ')，apply 收到 $(cat "$scratch/logs-copy-green/apply-patch-a.log" 2>/dev/null)：$(grep -E '✗|删了|源码戳' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # MERGE_AND_VERIFY_KEEP_COPY=1：留着副本与编译目录、打出路径
  out="$scratch/out-copy-kept.txt"; FAKE_CARGO_MODE=green MERGE_AND_VERIFY_KEEP_COPY=1 run_case copy-kept --copy "$scratch/patch-a"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || [[ ! -d "$scratch/logs-copy-kept/repo" ]] || ! grep -qF "留着副本 $scratch/logs-copy-kept/repo 与编译目录 $scratch/logs-copy-kept/target" "$out"; then
    echo "  ✗ 自检：MERGE_AND_VERIFY_KEEP_COPY=1 时应当留着副本与编译目录、打出路径，实际退出码 $rc：$(grep -E '✗|留着|删了' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 与 check.sh 同一套判据：lint 表在这里另取一遍（sed 取那一段、grep 数条数），与脚本里 read_check_sh 不是同一段代码
  expected_lints="$(sed -n '/^CODE_DISCIPLINE_LINTS=(/,/^)/p' "$CHECK_SH" | grep -oE -- '-[DWAF] clippy::[a-z0-9_]+' | tr '\n' ' ')"
  expected_lints="${expected_lints% }"
  expected_lint_count="$(sed -n '/^CODE_DISCIPLINE_LINTS=(/,/^)/p' "$CHECK_SH" | grep -cE -- '-[DWAF] clippy::')"
  checked=$((checked+1))
  if [[ "$expected_lint_count" -lt 1 ]] || ! grep -qxF -- "clippy --offline --all-targets --all-features -- -D warnings $expected_lints" "$scratch/calls-three.txt"; then
    echo "  ✗ 自检：clippy 应当带上 check.sh 的 lint 表里每一条（$CHECK_SH 里数到 $expected_lint_count 条），假 cargo 收到的是：$(grep '^clippy' "$scratch/calls-three.txt")"; failures=1; fi   # gate-lint:detail
  checked=$((checked+1))
  if ! grep -qxF -- "build --offline --all-targets" "$scratch/calls-three.txt"; then
    echo "  ✗ 自检：应当跑 cargo build --offline --all-targets（check.sh 的构建那一条），假 cargo 收到的是：$(tr '\n' '|' < "$scratch/calls-three.txt")"; failures=1; fi   # gate-lint:detail
  checked=$((checked+1))
  if ! grep -qxF -- "test --offline --no-fail-fast -p demo" <<< "$(test_calls "$scratch/calls-three.txt")"; then
    echo "  ✗ 自检：cargo test 应当挑工作区成员减掉 checker 档包（样本里只剩 -p demo），假 cargo 收到的是：$(grep '^test' "$scratch/calls-three.txt")"; failures=1; fi   # gate-lint:detail
  # 测试那一步按测试目标并行起（文件头 test 一段）：这一格不设 MERGE_AND_VERIFY_TEST_TARGETS，假 cargo 的 metadata 与 --no-run 打录好的两个测试目标（demo 的 lib 与 test alpha）；
  # 全绿那一趟两个目标各起一条、汇总数到 2 个测试二进制 4 条测试，红的那一趟两条 FAILED 都列出来、退 1
  printf '%s\n' '{"packages":[{"id":"demo-id","name":"demo","targets":[{"kind":["lib"],"name":"demo","doctest":false},{"kind":["test"],"name":"alpha","doctest":false}]}],"workspace_members":["demo-id"]}' \
    > "$scratch/fake-metadata.json"
  printf '%s\n' '{"reason":"compiler-artifact","package_id":"demo-id","target":{"kind":["lib"],"name":"demo"},"profile":{"test":true},"executable":"/selftest/demo"}' \
    '{"reason":"compiler-artifact","package_id":"demo-id","target":{"kind":["test"],"name":"alpha"},"profile":{"test":true},"executable":"/selftest/alpha"}' > "$scratch/fake-artifacts.jsonl"
  out="$scratch/out-targets.txt"; MERGE_AND_VERIFY_TEST_TARGETS= FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-targets.txt" \
    FAKE_CARGO_METADATA="$scratch/fake-metadata.json" FAKE_CARGO_ARTIFACTS="$scratch/fake-artifacts.jsonl" run_case targets "$scratch/patch-a"; checked=$((checked+1))
  local targets_green_rc=$rc targets_red_out="$scratch/out-targets-red.txt" targets_red_rc
  # 红的那一趟要假 cargo 照最早那一份打一条 FAILED（全绿那一格之前已把它换成全 ok），跑完换回全 ok
  sed -i.bak 's/subtracts_in_%s \.\.\. ok/subtracts_in_%s ... FAILED/; s/ok\. 2 passed; 0 failed/FAILED. 1 passed; 1 failed/' "$fake_cargo_dir/cargo" && rm -f "$fake_cargo_dir/cargo.bak"
  out="$targets_red_out"; MERGE_AND_VERIFY_TEST_TARGETS= FAKE_CARGO_CALLS="$scratch/calls-targets-red.txt" \
    FAKE_CARGO_METADATA="$scratch/fake-metadata.json" FAKE_CARGO_ARTIFACTS="$scratch/fake-artifacts.jsonl" run_case targets-red "$scratch/patch-a"; targets_red_rc=$rc
  sed -i.bak 's/ \.\.\. FAILED/ ... ok/; s/FAILED\. 1 passed; 1 failed/ok. 2 passed; 0 failed/' "$fake_cargo_dir/cargo" && rm -f "$fake_cargo_dir/cargo.bak"
  out="$scratch/out-targets.txt"
  if [[ $targets_green_rc -ne 0 ]] || ! grep -qxF -- "test --offline --no-fail-fast -p demo --no-run --message-format=json-render-diagnostics" "$scratch/calls-targets.txt" \
     || ! grep -qxF -- "test --offline --no-fail-fast -p demo --lib" <<< "$(test_calls "$scratch/calls-targets.txt")" || ! grep -qxF -- "test --offline --no-fail-fast -p demo --test alpha" <<< "$(test_calls "$scratch/calls-targets.txt")" \
     || ! grep -q '✓ 测试全绿（2 个测试二进制，共 4 条测试' "$out" || ! grep -q '^TEST_TARGETS_PARALLEL dispatched=2 collected=2 ' "$scratch/logs-targets/test.log" \
     || [[ $targets_red_rc -ne 1 ]] || ! grep -q '红的测试 2 条' "$targets_red_out"; then
    echo "  ✗ 自检：测试那一步应当经 test-targets-parallel.py 按测试目标并行起（先 --no-run 编、两个目标各一条、汇总 2 个测试二进制 4 条测试；红的那一趟列出 2 条、退 1），假 cargo 收到的是：$(grep '^test' "$scratch/calls-targets.txt" 2>/dev/null | tr '\n' '|')；全绿那一趟：$(grep -E '✓ 测试|✗' "$out" | head -2 | tr '\n' ' ')；红的那一趟退 $targets_red_rc：$(grep -E '红的测试|✓ 测试' "$targets_red_out" | head -1)"; failures=1; fi   # gate-lint:detail
  # 慢档一起跑（文件头 test 一段「慢档一起跑」）：一条 cargo test 那一种（全绿那一格）与按测试目标并行起那一种（上一格）收到的都以 -- --include-ignored 收尾，
  # checker 档包照旧不进；并行起那一趟的成功句报慢档条数（假 cargo 遇到 --list 每个目标列一条，两个目标共 2 条）
  checked=$((checked+1))
  if ! grep -qxF -- "test --offline --no-fail-fast -p demo -- --include-ignored" "$scratch/calls-three.txt" \
     || ! grep -qxF -- "test --offline --no-fail-fast -p demo --lib -- --include-ignored" "$scratch/calls-targets.txt" \
     || ! grep -qxF -- "test --offline --no-fail-fast -p demo --test alpha -- --include-ignored" "$scratch/calls-targets.txt" \
     || grep -qE '^test .*-p singlefs-checker-tier' "$scratch/calls-three.txt" "$scratch/calls-targets.txt" \
     || ! grep -qF '共 4 条测试，其中慢档 2 条；' "$scratch/out-targets.txt"; then
    echo "  ✗ 自检：测试那一步应当带 --include-ignored 把慢档一起跑（一条 cargo test 与按测试目标并行起两种都以 -- --include-ignored 收尾）、checker 档包照旧不进、并行起那一趟的成功句报慢档 2 条，假 cargo 收到的是：$(grep -h '^test' "$scratch/calls-three.txt" "$scratch/calls-targets.txt" 2>/dev/null | tr '\n' '|')；成功句：$(grep '✓ 测试' "$scratch/out-targets.txt" | head -1)"; failures=1; fi   # gate-lint:detail
  # 测试只跑改到的包（文件头）：样本工作区五个成员 demo、alpha、beta、delta（依赖 demo）与 checker 档包；补丁改的都是包里的数据文件（重编核对不核）
  local narrow_tree="$scratch/narrow-tree" narrow_member
  mkdir -p "$narrow_tree/crates/singlefs-checker-tier"
  printf '[workspace]\nmembers = ["crates/demo", "crates/alpha", "crates/beta", "crates/delta", "crates/singlefs-checker-tier"]\n' > "$narrow_tree/Cargo.toml"
  for narrow_member in demo alpha beta delta; do
    mkdir -p "$narrow_tree/crates/$narrow_member/src" "$narrow_tree/crates/$narrow_member/tests"; printf 'pub fn f() {}\n' > "$narrow_tree/crates/$narrow_member/src/lib.rs"
    printf '[package]\nname = "%s"\n' "$narrow_member" > "$narrow_tree/crates/$narrow_member/Cargo.toml"
  done
  printf '\n[dev-dependencies]\ndemo = { path = "../demo" }\n' >> "$narrow_tree/crates/delta/Cargo.toml"
  printf '[package]\nname = "singlefs-checker-tier"\n' > "$narrow_tree/crates/singlefs-checker-tier/Cargo.toml"
  narrow_patch() {   # $1 补丁目录，其余是改到的路径；树里那几份写成打过之后的内容（--skip-apply 先核补丁打过没有）
    mkdir -p "$1"; local narrow_path; : > "$1/crates.patch"
    for narrow_path in "${@:2}"; do printf -- '--- a/%s\n+++ b/%s\n@@ -1 +1 @@\n-1\n+2\n' "$narrow_path" "$narrow_path" >> "$1/crates.patch"
      mkdir -p "$(dirname "$narrow_tree/$narrow_path")"; printf '2\n' > "$narrow_tree/$narrow_path"; done
  }
  narrow_patch "$scratch/narrow-two" crates/alpha/src/table.txt crates/beta/tests/data.txt
  narrow_patch "$scratch/narrow-demo" crates/demo/src/table.txt
  narrow_patch "$scratch/narrow-lock" Cargo.lock
  narrow_case() {   # $1 标签，其余是参数
    FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-narrow-$1.txt" PATH="$fake_cargo_dir:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none \
      MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-narrow-$1" MERGE_AND_VERIFY_TREE="$narrow_tree" env -u CARGO_TARGET_DIR bash "$0" "${@:2}" > "$out" 2>&1; rc=$?
  }
  out="$scratch/out-narrow-two.txt"; narrow_case two --skip-apply "$scratch/narrow-two"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -qxF -- "test --offline --no-fail-fast -p alpha -p beta" <<< "$(test_calls "$scratch/calls-narrow-two.txt")" || ! grep -qF '核过 1 份补丁都已打在树上' "$out" \
     || ! grep -qF '本次未跑：补丁没改到 2 个包：demo、delta' "$out" || ! grep -q '✓ 测试全绿.*跑了 2 个包；本次未跑：补丁没改到 2 个包：demo、delta' "$out"; then
    echo "  ✗ 自检：补丁改到 alpha 与 beta 两个包：cargo test 只该挑这两个、报 demo 与 delta 本次未跑，实际退出码 $rc，假 cargo 收到 $(grep '^test' "$scratch/calls-narrow-two.txt" 2>/dev/null)：$(grep -E '✓ 测试|本次未跑|✗' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  out="$scratch/out-narrow-demo.txt"; narrow_case demo --skip-apply "$scratch/narrow-demo"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -qxF -- "test --offline --no-fail-fast -p demo -p delta" <<< "$(test_calls "$scratch/calls-narrow-demo.txt")" || ! grep -qF '另带上依赖它们的：delta' "$out"; then
    echo "  ✗ 自检：补丁改到 demo：cargo test 应当挑 demo 与依赖它的 delta，实际退出码 $rc，假 cargo 收到 $(grep '^test' "$scratch/calls-narrow-demo.txt" 2>/dev/null)：$(grep -E '编译并跑|✗' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  out="$scratch/out-narrow-lock.txt"; narrow_case lock --skip-apply "$scratch/narrow-lock"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -qxF -- "test --offline --no-fail-fast -p demo -p alpha -p beta -p delta" <<< "$(test_calls "$scratch/calls-narrow-lock.txt")" || ! grep -qF '补丁改到工作区根的 Cargo.lock，照旧全跑' "$out"; then
    echo "  ✗ 自检：补丁改到工作区根的 Cargo.lock：cargo test 应当照旧挑全部四个包，实际退出码 $rc，假 cargo 收到 $(grep '^test' "$scratch/calls-narrow-lock.txt" 2>/dev/null)：$(grep -E '编译并跑|本次未跑|✗' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  # --skip-apply 给的补丁没打在树上（树里那一份还是改前的 1）：退 1、点名那份补丁、不编
  narrow_patch "$scratch/narrow-unapplied" crates/alpha/src/unapplied.txt
  printf '1\n' > "$narrow_tree/crates/alpha/src/unapplied.txt"
  out="$scratch/out-narrow-unapplied.txt"; narrow_case unapplied --skip-apply "$scratch/narrow-unapplied"; checked=$((checked+1))
  rm -f "$narrow_tree/crates/alpha/src/unapplied.txt"
  if [[ $rc -ne 1 ]] || ! grep -qF "没打在要验的树" "$out" || ! grep -qF "$scratch/narrow-unapplied" "$out" || [[ -s "$scratch/calls-narrow-unapplied.txt" ]]; then
    echo "  ✗ 自检：--skip-apply 给的补丁没打在树上：应当退 1、点名那份补丁、不编，实际退出码 $rc，假 cargo 收到 $(cat "$scratch/calls-narrow-unapplied.txt" 2>/dev/null | wc -l) 条：$(grep -E '✗|核过|本次未核' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  out="$scratch/out-narrow-all.txt"; narrow_case all --skip-apply --all-packages "$scratch/narrow-two"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -qxF -- "test --offline --no-fail-fast -p demo -p alpha -p beta -p delta" <<< "$(test_calls "$scratch/calls-narrow-all.txt")" || ! grep -qF '带了 --all-packages，照旧全跑' "$out"; then
    echo "  ✗ 自检：带 --all-packages：cargo test 应当照旧挑全部四个包，实际退出码 $rc，假 cargo 收到 $(grep '^test' "$scratch/calls-narrow-all.txt" 2>/dev/null)"; failures=1; fi   # gate-lint:detail
  narrow_patch "$scratch/narrow-checker" crates/singlefs-checker-tier/src/table.txt
  mkdir -p "$narrow_tree/crates/singlefs-checker-tier/src"
  out="$scratch/out-narrow-checker.txt"; narrow_case checker --skip-apply "$scratch/narrow-checker"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || grep -q '^test' "$scratch/calls-narrow-checker.txt" 2>/dev/null || ! grep -qF '测试本次未跑：这一批补丁一个要测的包都没改到' "$out"; then
    echo "  ✗ 自检：补丁只改到 checker 档包：测试这一步应当不跑、报测试本次未跑、退 0，实际退出码 $rc，假 cargo 收到 $(grep '^test' "$scratch/calls-narrow-checker.txt" 2>/dev/null)：$(grep -E '本次未跑|✗' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  # check.sh 读不出 lint 表、或 clippy 那一行变了：退 1，一个补丁都不打、不编
  sed '/^CODE_DISCIPLINE_LINTS=(/,/^)/d' "$CHECK_SH" > "$scratch/check-no-table.sh"
  sed 's/-- -D warnings "\${CODE_DISCIPLINE_LINTS\[@\]}"/-- -D warnings -W clippy::pedantic "${CODE_DISCIPLINE_LINTS[@]}"/' "$CHECK_SH" > "$scratch/check-clippy-changed.sh"
  for check_case in no-table clippy-changed; do
    out="$scratch/out-check-$check_case.txt"; checked=$((checked+1))
    FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-check-$check_case.txt" MERGE_AND_VERIFY_CHECK_SH="$scratch/check-$check_case.sh" run_case "check-$check_case" --skip-apply
    if [[ $rc -ne 1 ]] || ! grep -q 'check.sh 读不出、或与这里对不上' "$out" || [[ -s "$scratch/calls-check-$check_case.txt" ]]; then
      echo "  ✗ 自检：check.sh $([[ $check_case == no-table ]] && echo '里读不出 lint 表' || echo '的 clippy 那一行变了')时应当退 1、不编，实际退出码 $rc，假 cargo 收到 $(cat "$scratch/calls-check-$check_case.txt" 2>/dev/null | wc -l) 条：$(grep -E '✗|!' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  done
  # 工作区成员里没有 singlefs-checker-tier（样本里那个成员改了名）：排不掉它，退 1、不编
  write_workspace "$scratch/tree-renamed" singlefs-checker-tier-renamed
  mkdir -p "$scratch/tree-renamed/crates/demo/src"; printf 'pub fn f() {}\n' > "$scratch/tree-renamed/crates/demo/src/lib.rs"
  out="$scratch/out-renamed.txt"; checked=$((checked+1))
  PATH="$fake_cargo_dir:$PATH" FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-renamed.txt" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-renamed" MERGE_AND_VERIFY_TREE="$scratch/tree-renamed" env -u CARGO_TARGET_DIR bash "$0" --skip-apply > "$out" 2>&1; rc=$?
  if [[ $rc -ne 1 ]] || ! grep -q '排不掉它' "$out" || grep -q '^test' "$scratch/calls-renamed.txt" 2>/dev/null; then
    echo "  ✗ 自检：工作区成员里没有 singlefs-checker-tier 时应当退 1、不编，实际退出码 $rc：$(grep '^test' "$scratch/calls-renamed.txt" 2>/dev/null | tr '\n' ' ')$(grep -E '✗' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  # cargo test 退 0 而一条测试都没跑（假 cargo 什么都不打）：退 1
  out="$scratch/out-empty.txt"; checked=$((checked+1)); FAKE_CARGO_MODE=empty run_case empty --skip-apply
  if [[ $rc -ne 1 ]] || ! grep -q '一条测试都没跑' "$out"; then
    echo "  ✗ 自检：cargo test 退 0 却一条测试都没跑时应当退 1，实际退出码 $rc：$(grep -E '✓ 测试|✗' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 没设日志目录：日志落在工作根下（SINGLEFS_WORK_ROOT 指进临时目录）
  out="$scratch/out4.txt"; checked=$((checked+1))
  PATH="$fake_cargo_dir:$PATH" FAKE_CARGO_MODE=green MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none SINGLEFS_WORK_ROOT="$scratch/work-root" \
    MERGE_AND_VERIFY_TREE="$scratch/tree" env -u MERGE_AND_VERIFY_LOG_DIRECTORY -u CARGO_TARGET_DIR bash "$0" "$scratch/patch-a" > "$out" 2>&1; rc=$?
  if [[ $rc -ne 0 ]] || ! compgen -G "$scratch/work-root/merge-and-verify/*/test.log" > /dev/null; then
    echo "  ✗ 自检：没设日志目录时日志应当落在工作根下（$scratch/work-root/merge-and-verify/<日期>-<进程号>/），实际退出码 $rc：$(grep '日志在' "$out" | tail -1)"; failures=1; fi   # gate-lint:detail
  # 换了源码却没重编（调用方给的编译目录 <树>/target 两趟共用，收尾不删）：临时副本先验一趟（编过），再从源树 rsync -a 铺进新的 walk.rs（mtime 比编译目录里的 dep-info 旧，取 2020 年初的 epoch 秒）；
  # 假 cargo 照 mtime 判新不新鲜（source-stamp.py --write-selftest-cargo 写的那一份），没刷源码戳就认 walk.rs 已编、那一条 FAILED
  local stale_tree="$scratch/stale-tree" stale_source="$scratch/stale-source" stale_bin="$scratch/stale-bin"
  mkdir -p "$stale_tree/crates/demo/src" "$stale_source/crates/demo/src" "$stale_bin"
  write_workspace "$stale_tree" singlefs-checker-tier
  python3 "$STAMP" --write-selftest-cargo "$stale_bin/cargo"
  printf 'pub mod walk;\n' > "$stale_tree/crates/demo/src/lib.rs"
  printf 'pub fn walk() -> u32 { 1 }\n' > "$stale_tree/crates/demo/src/walk.rs"
  touch -d @1577836800 "$stale_tree/crates/demo/src/lib.rs" "$stale_tree/crates/demo/src/walk.rs"
  cp -p "$stale_tree/crates/demo/src/lib.rs" "$stale_source/crates/demo/src/lib.rs"
  printf 'pub fn walk() -> u32 { 22 }\n' > "$stale_source/crates/demo/src/walk.rs"
  touch -d @1577836900 "$stale_source/crates/demo/src/walk.rs"
  stale_case() { PATH="$stale_bin:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-$1" \
    MERGE_AND_VERIFY_TREE="$stale_tree" env CARGO_TARGET_DIR="$stale_tree/target" bash "$0" --skip-apply > "$out" 2>&1; rc=$?; }
  out="$scratch/out5-first.txt"; stale_case five-first
  rsync -a "$stale_source/" "$stale_tree/"
  out="$scratch/out5.txt"; stale_case five; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -q 'Compiling crates/demo/src/walk.rs' "$scratch/logs-five/test.log" 2>/dev/null || ! grep -q '测试全绿' "$out"; then
    echo "  ✗ 自检：往编过的副本里 rsync -a 铺新内容（旧 mtime）之后再验，应当重编改过的 walk.rs、测试全绿，实际退出码 $rc：$(grep -E 'Fresh|Compiling|FAILED' "$scratch/logs-five/test.log" 2>/dev/null | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # cargo test 那一趟内存包装退 245（显存不够）：假包装打一条 FAILED 再退 245
  cat > "$scratch/fake-wrapper-245.sh" <<'FAKE'
#!/usr/bin/env bash
# 假内存包装：cargo test 那一趟打一条 FAILED、退 245（显存不够）；fmt、clippy 照退 0
for word in "$@"; do
  if [[ "$word" == test ]]; then
    printf 'test tests::on_gpu ... FAILED\n\ntest result: FAILED. 0 passed; 1 failed\nrun-with-memory-cap: 显存不够（假包装），退 245\n'
    exit 245
  fi
done
exit 0
FAKE
  out="$scratch/out6.txt"; checked=$((checked+1))
  PATH="$fake_cargo_dir:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER="$scratch/fake-wrapper-245.sh" MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-six" \
    MERGE_AND_VERIFY_TREE="$scratch/tree" env -u CARGO_TARGET_DIR bash "$0" --skip-apply > "$out" 2>&1; rc=$?
  if [[ $rc -ne 1 ]] || ! grep -q '显存不够' "$out" || grep -q 'rerun-failed-tests.py' "$out" || grep -q '红的测试' "$out"; then
    echo "  ✗ 自检：cargo test 那一趟包装退 245 时应当报显存不够、退 1，不列 FAILED、不给重跑命令，实际退出码 $rc：$(grep -E '✗|→' "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 改过的包重编了没有（文件头；调用方给的编译目录 <树>/target 几趟共用，收尾不删）：假 cargo 按包、按 mtime 判新不新鲜，重编时照真 cargo 打「Compiling <包名> v…」（lib_rebuilt_packages.py --write-selftest-cargo）；
  # 补丁目录 rebuilt-patch 的 crates.patch 改的是 crates/demo/src/walk.rs（树里已经是改过的样子，--skip-apply 只验不打）
  local rebuilt_tree="$scratch/rebuilt-tree" rebuilt_bin="$scratch/rebuilt-bin" rebuilt_patch="$scratch/rebuilt-patch"
  mkdir -p "$rebuilt_tree/crates/demo/src" "$rebuilt_bin" "$rebuilt_patch"
  write_workspace "$rebuilt_tree" singlefs-checker-tier
  printf 'pub mod walk;\n' > "$rebuilt_tree/crates/demo/src/lib.rs"
  printf 'pub fn walk() -> u32 { 22 }\n' > "$rebuilt_tree/crates/demo/src/walk.rs"
  printf 'diff --git a/crates/demo/src/walk.rs b/crates/demo/src/walk.rs\n--- a/crates/demo/src/walk.rs\n+++ b/crates/demo/src/walk.rs\n@@ -1 +1 @@\n-pub fn walk() -> u32 { 1 }\n+pub fn walk() -> u32 { 22 }\n' \
    > "$rebuilt_patch/crates.patch"
  python3 "$REBUILT" --write-selftest-cargo "$rebuilt_bin/cargo"
  rebuilt_case() { PATH="$rebuilt_bin:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-$1" \
    MERGE_AND_VERIFY_TREE="$rebuilt_tree" env CARGO_TARGET_DIR="$rebuilt_tree/target" bash "$0" --skip-apply "$rebuilt_patch" > "$out" 2>&1; rc=$?; }
  # 绿样本：头一次编（编译目录是空的），demo 有 Compiling 行
  out="$scratch/out-rebuilt-first.txt"; rebuilt_case rebuilt-first; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -q '改过的包这一趟都重编了（核了 1 个包：demo）' "$out"; then
    echo "  ✗ 自检：头一次编、补丁改过的 demo 有 Compiling 行：应当退 0、报都重编了，实际退出码 $rc：$(grep -E '✗|✓|重编' "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 编过之后再验同一批补丁：源码戳没变（refresh 一个都不 touch），要靠编之前把补丁改过的文件刷到此刻，demo 才重编
  out="$scratch/out-rebuilt-rerun.txt"; rebuilt_case rebuilt-rerun; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -q '改过的包这一趟都重编了（核了 1 个包：demo）' "$out"; then
    echo "  ✗ 自检：编过之后 --skip-apply 再验同一批补丁：改过的 demo 应当重编、退 0，实际退出码 $rc：$(grep -E '✗|✓|重编|刷到此刻' "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 红样本：编译目录里 demo 的产物比源码新（另一份副本共用这个编译目录、后编的那种），源码的修改时刻比它旧；
  # checker 档那个包的文件刷到此刻、照常重编（有一行别的 Compiling），demo 没有 Compiling 行，测试照样全绿
  touch -d '+1 day' "$rebuilt_tree/target/debug/.fake/demo.d"
  touch "$rebuilt_tree/crates/singlefs-checker-tier/Cargo.toml"
  out="$scratch/out-rebuilt-stale.txt"; rebuilt_case rebuilt-stale; checked=$((checked+1))
  if [[ $rc -ne 1 ]] || ! grep -q '改过的包这一趟没重编：demo' "$out" || ! grep -q 'Compiling singlefs-checker-tier' "$scratch/logs-rebuilt-stale/test.log" 2>/dev/null; then
    echo "  ✗ 自检：编译目录里的产物比源码新（共用编译目录）、补丁改过的 demo 没有 Compiling 行：应当退 1、点名 demo，实际退出码 $rc：$(grep -E '✗|✓|重编' "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 跑完运行目录里不剩它建的编译目录（文件头「这一趟的编译目录」）：不带 --copy、没设 CARGO_TARGET_DIR 的三种结局——全绿退 0（out3 那一趟）、测试红退 1（out1 那一趟）、
  # 被 TERM 停（假 cargo test 睡 2 秒，时间线里出现它开始之后给脚本发 TERM，脚本等假 cargo 退出后走 EXIT trap，退 143）——<日志目录>/target 都不在、输出里有删了它那一行；
  # 调用方设了 CARGO_TARGET_DIR 的（上面重编核对那几趟）照它编、收尾不删
  local term_pid term_rc term_attempt removal_problems=""
  [[ ! -e "$scratch/logs-three/target" ]] && grep -qF "删了这一趟的编译目录 $scratch/logs-three/target" "$scratch/out3.txt" || removal_problems+="全绿那一趟（out3）还剩 $(ls -d "$scratch/logs-three/target" 2>/dev/null || echo 没剩、但没打删了那一行)；"
  [[ ! -e "$scratch/logs-one/target" ]] && grep -qF "删了这一趟的编译目录 $scratch/logs-one/target" "$scratch/out1.txt" || removal_problems+="测试红那一趟（out1）还剩 $(ls -d "$scratch/logs-one/target" 2>/dev/null || echo 没剩、但没打删了那一行)；"
  PATH="$rebuilt_bin:$PATH" FAKE_CARGO_TIMELINE="$scratch/term-timeline.txt" FAKE_CARGO_TAG=T FAKE_CARGO_SLEEP=2 MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-term" MERGE_AND_VERIFY_TREE="$rebuilt_tree" env -u CARGO_TARGET_DIR bash "$0" --skip-apply > "$scratch/out-term.txt" 2>&1 &
  term_pid=$!
  for term_attempt in $(seq 1 200); do grep -q '^start T ' "$scratch/term-timeline.txt" 2>/dev/null && break; kill -0 "$term_pid" 2>/dev/null || break; sleep 0.05; done
  kill -TERM "$term_pid" 2>/dev/null
  if wait "$term_pid"; then term_rc=0; else term_rc=$?; fi
  ((term_rc == 143)) && [[ ! -e "$scratch/logs-term/target" ]] && grep -qF "删了这一趟的编译目录 $scratch/logs-term/target" "$scratch/out-term.txt" \
    || removal_problems+="被 TERM 停的那一趟退 $term_rc（要 143），还剩 $(ls -d "$scratch/logs-term/target" 2>/dev/null || echo 没剩)：$(tail -2 "$scratch/out-term.txt" | tr '\n' ' ')；"
  [[ -d "$rebuilt_tree/target/debug" ]] && ! grep -q '删了这一趟的编译目录' "$scratch/out-rebuilt-first.txt" || removal_problems+="调用方设的 CARGO_TARGET_DIR 收尾被删了或打了删了那一行；"
  checked=$((checked+1))
  if [[ -n "$removal_problems" ]]; then
    echo "  ✗ 自检：跑完运行目录里不剩它建的编译目录（正常、出错、被 TERM 三种结局都删 <日志目录>/target，调用方设的 CARGO_TARGET_DIR 不删），实际：$removal_problems"; failures=1; fi   # gate-lint:detail
  # 这一趟的编译目录里还有进程开着文件（假 cargo test 留下一个后台 sleep 开着 <编译目录>/held，进程号写进 holder-pid）：不删、报出那个进程、退 1；看完只停那一个进程号
  local holder_bin="$scratch/holder-bin" holder_pid
  mkdir -p "$holder_bin"
  printf '#!/usr/bin/env bash\nif [[ "$1" == test ]]; then mkdir -p "$CARGO_TARGET_DIR"; sleep 30 > "$CARGO_TARGET_DIR/held" 2>&1 < /dev/null & echo "$!" > "$FAKE_HOLDER_PID"; fi\nexec python3 %q "$@"\n' "$rebuilt_bin/cargo" > "$holder_bin/cargo"
  chmod +x "$holder_bin/cargo"
  out="$scratch/out-holder.txt"; checked=$((checked+1))
  PATH="$holder_bin:$PATH" FAKE_HOLDER_PID="$scratch/holder-pid" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-holder" MERGE_AND_VERIFY_TREE="$rebuilt_tree" env -u CARGO_TARGET_DIR bash "$0" --skip-apply > "$out" 2>&1; rc=$?
  holder_pid="$(cat "$scratch/holder-pid" 2>/dev/null)"
  if [[ $rc -ne 1 || ! -d "$scratch/logs-holder/target" ]] || ! grep -q "还有 1 个进程开着里面的文件" "$out" || ! grep -q "进程 ${holder_pid:-没有}（sleep 30）" "$out"; then
    echo "  ✗ 自检：这一趟的编译目录里还有进程开着文件：应当不删、报出那个进程、退 1，实际退出码 $rc，编译目录$([[ -d "$scratch/logs-holder/target" ]] && echo 还在 || echo 删了)：$(grep -E '✗|删了|进程' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  [[ ! "$holder_pid" =~ ^[0-9]+$ ]] || kill "$holder_pid" 2>/dev/null
  # 主工作区合入锁（文件头）：两个合入同时起。A 的 cargo test 睡 1.5 秒；锁记录里出现 A 的进程号之后起 B；
  # B 要报「在排队」与 A 的进程号，B 的 cargo test 要在 A 的结束之后才开始
  local lock_tree="$scratch/lock-tree" timeline="$scratch/lock-timeline.txt" a_pid b_pid a_rc b_rc attempt holder_pid=""
  mkdir -p "$lock_tree/crates/demo/src"; write_workspace "$lock_tree" singlefs-checker-tier; printf 'pub fn f() {}\n' > "$lock_tree/crates/demo/src/lib.rs"
  PATH="$rebuilt_bin:$PATH" FAKE_CARGO_TIMELINE="$timeline" FAKE_CARGO_TAG=A FAKE_CARGO_SLEEP=1.5 MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-lock-a" MERGE_AND_VERIFY_TREE="$lock_tree" env -u CARGO_TARGET_DIR bash "$0" --skip-apply > "$scratch/out-lock-a.txt" 2>&1 &
  a_pid=$!
  for attempt in $(seq 1 200); do
    holder_pid="$(sed -n 's/^pid=//p' "$SINGLEFS_WORKSPACE_LOCK_FILE" 2>/dev/null)"
    [[ -z "$holder_pid" ]] || break
    kill -0 "$a_pid" 2>/dev/null || break   # A 已经跑完（不认锁时它根本不记），不再等
    sleep 0.05
  done
  PATH="$rebuilt_bin:$PATH" FAKE_CARGO_TIMELINE="$timeline" FAKE_CARGO_TAG=B MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-lock-b" MERGE_AND_VERIFY_TREE="$lock_tree" env -u CARGO_TARGET_DIR bash "$0" --skip-apply > "$scratch/out-lock-b.txt" 2>&1 &
  b_pid=$!
  if wait "$a_pid"; then a_rc=0; else a_rc=$?; fi
  if wait "$b_pid"; then b_rc=0; else b_rc=$?; fi
  local a_end b_start
  a_end="$(awk '$1 == "end" && $2 == "A" { print $3 }' "$timeline" 2>/dev/null)"
  b_start="$(awk '$1 == "start" && $2 == "B" { print $3 }' "$timeline" 2>/dev/null)"
  checked=$((checked+1))
  if [[ $a_rc -ne 0 || $b_rc -ne 0 || -z "$a_end" || -z "$b_start" || "$holder_pid" != "$a_pid" ]] || ((b_start <= a_end)) \
     || ! grep -q "在排队.*进程 $a_pid" "$scratch/out-lock-b.txt"; then
    lock_order_detail="A 的 cargo test 结束于 ${a_end:-没有}、B 的开始于 ${b_start:-没有}，B 报的：$(grep -E '排队|拿到|✗' "$scratch/out-lock-b.txt" | head -3 | tr '\n' ' ')"
    echo "  ✗ 自检：两个合入同时起：后起的应当排队、报出前者的进程号、前者编完放锁后才编，实际 A 退 $a_rc（进程 $a_pid，锁记录里 ${holder_pid:-没有}）、B 退 $b_rc， $lock_order_detail"; failures=1; fi   # gate-lint:detail
  # 锁文件里的持有者已死（记一个已经退出的进程）：报出来、接管、照常验完，放锁后记录清空
  local dead_pid
  bash -c 'exit 0' & dead_pid=$!
  wait "$dead_pid"
  printf 'pid=%s\nstart=1\nsession=死掉的会话\ncommand=merge-and-verify.sh 假的\nsince=%s\n' "$dead_pid" "$(date +%s)" > "$SINGLEFS_WORKSPACE_LOCK_FILE"
  out="$scratch/out-lock-dead.txt"; checked=$((checked+1))
  FAKE_CARGO_MODE=green SINGLEFS_WORKSPACE_LOCK_GIVE_UP_SECONDS=5 run_case lock-dead --skip-apply
  if [[ $rc -ne 0 ]] || ! grep -q '已死.*死掉的会话.*接管' "$out" || grep -q '^pid=' "$SINGLEFS_WORKSPACE_LOCK_FILE"; then
    echo "  ✗ 自检：锁文件里的持有者已死：应当报出来、接管、照常验完、放锁后记录清空，实际退出码 $rc：$(grep -E '锁|✗' "$out" | head -3 | tr '\n' ' ')；锁文件 $(tr '\n' ' ' < "$SINGLEFS_WORKSPACE_LOCK_FILE")"; failures=1; fi   # gate-lint:detail
  # 每棵工作树各一把（lib_workspace_lock.py 文件头「锁文件」）：临时仓的主工作树的锁被一个活着的假合入占着（它拿到锁之后 exec sleep，进程号不变），
  # 在它 git worktree 加出来的树里验不排队、照常验完；这一格不设 SINGLEFS_WORKSPACE_LOCK_FILE，锁文件在各自的 git 目录里。放弃时限 3 秒
  # （弄坏开关 WORKSPACE_LOCK_BREAK=lock-in-common-dir 下排到点退 1、这一格红，不挂住）；用完只停自己起的那一个进程号
  local wt_main="$scratch/wt-main" wt_linked="$scratch/wt-linked" wt_holder wt_attempt
  mkdir -p "$wt_main/crates/demo/src"; write_workspace "$wt_main" singlefs-checker-tier; printf 'pub fn f() {}\n' > "$wt_main/crates/demo/src/lib.rs"
  { git init -q "$wt_main" && git -C "$wt_main" add -A && git -C "$wt_main" -c user.name=s -c user.email=s@s commit -qm base && git -C "$wt_main" worktree add -q "$wt_linked"; } > /dev/null 2>&1
  env -u SINGLEFS_WORKSPACE_LOCK_FILE bash -c 'python3 "$0" acquire --root "$1" --holder-pid $$ --command "假合入 主工作树" > /dev/null || exit 1; : > "$2"; exec sleep 30' \
    "$WORKSPACE_LOCK" "$wt_main" "$scratch/wt-held" &
  wt_holder=$!
  for wt_attempt in $(seq 1 200); do [[ -e "$scratch/wt-held" ]] && break; kill -0 "$wt_holder" 2>/dev/null || break; sleep 0.05; done
  out="$scratch/out-lock-per-tree.txt"; checked=$((checked+1))
  PATH="$rebuilt_bin:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-lock-per-tree" \
    MERGE_AND_VERIFY_TREE="$wt_linked" SINGLEFS_WORKSPACE_LOCK_GIVE_UP_SECONDS=3 env -u CARGO_TARGET_DIR -u SINGLEFS_WORKSPACE_LOCK_FILE bash "$0" --skip-apply > "$out" 2>&1; rc=$?
  kill "$wt_holder" 2>/dev/null
  wait "$wt_holder" 2>/dev/null
  if [[ ! -e "$scratch/wt-held" || $rc -ne 0 ]] || grep -q '在排队' "$out"; then
    echo "  ✗ 自检：主工作树的锁被占着时在 git worktree 加出来的树里验：应当不排队、照常验完退 0，实际$([[ -e "$scratch/wt-held" ]] || echo '主工作树的假合入没拿到锁、')退出码 $rc：$(grep -E '锁|排队|✗' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # --peer（文件头「--peer」）：假 multi-host-run.sh 把 -- 后面那条放在 <临时目录>/peer-tree（当第二台的副本：假的 run-with-memory-cap.sh 与 capped.sh 只剥掉上限与线程数）里跑，
  # 把 --fetch 点名的目录拷进 MULTI_HOST_RUN_FETCH_DIRECTORY；PATH 前面的假 cargo 照最早那一份（FAKE_CARGO_MODE=green 时全 ok、退 0，否则一条 FAILED、退 101）
  local peer_tree="$scratch/peer-tree" peer_bin="$scratch/peer-bin" fake_multi_host_run="$scratch/fake-multi-host-run.sh" peer_calls="$scratch/peer-calls.txt" peer_cargo_calls
  mkdir -p "$peer_tree/scripts/hosts" "$peer_bin"
  printf '#!/usr/bin/env bash\n# 假内存包装：剥掉上限照跑\nshift\nexec "$@"\n' > "$peer_tree/scripts/hosts/run-with-memory-cap.sh"
  printf '#!/usr/bin/env bash\n# 假 capped.sh：剥掉线程数照跑\nshift\nexec "$@"\n' > "$peer_tree/scripts/hosts/capped.sh"
  cat > "$peer_bin/cargo" <<'FAKE'
#!/usr/bin/env bash
[[ -z "${FAKE_CARGO_CALLS:-}" ]] || printf '%s\t%s\n' "$PWD" "$*" >> "$FAKE_CARGO_CALLS"
case "$1" in
  test) if [[ "${FAKE_CARGO_MODE:-}" == green ]]; then printf 'running 2 tests\ntest tests::adds ... ok\ntest tests::subtracts ... ok\n\ntest result: ok. 2 passed; 0 failed\n'; exit 0; fi
        printf 'running 2 tests\ntest tests::adds ... ok\ntest tests::subtracts_on_peer ... FAILED\n\ntest result: FAILED. 1 passed; 1 failed\n'; exit 101 ;;
  fmt|clippy) if [[ -n "${FAKE_CARGO_MISSING_COMPONENTS:-}" ]]; then   # 第二台的工具链没装 rustfmt 与 clippy：rustup 的代理这样报、退 1
                echo "error: 'cargo-$1' is not installed for the toolchain '1.98.0-x86_64-unknown-linux-gnu'" >&2; exit 1; fi ;;
esac
exit 0
FAKE
  cat > "$fake_multi_host_run" <<'FAKE'
#!/usr/bin/env bash
# 假 multi-host-run.sh：参数、共用编译目录与多机配置记进 FAKE_PEER_CALLS；不带 --peer 退 2；FAKE_PEER_FAIL 设了照没跑成退 255；
# FAKE_PEER_REQUIRE_CONFIG 设了而 SINGLEFS_MULTI_HOST_CONFIG 指不到文件，照真的那样当副本里找不到配置、退 255；
# 否则在 FAKE_PEER_TREE 里跑 -- 后面那条，把 --fetch 点名的目录拷进 MULTI_HOST_RUN_FETCH_DIRECTORY，退出码照那条命令的
printf 'args %s\nshared %s\nconfig %s\n' "$*" "${MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY:-}" "${SINGLEFS_MULTI_HOST_CONFIG:-}" >> "$FAKE_PEER_CALLS"
fetch="" peer=0
while (($#)) && [[ "$1" != "--" ]]; do
  case "$1" in
    --peer) peer=1; shift ;;
    --fetch) fetch="$2"; shift 2 ;;
    *) shift ;;
  esac
done
shift
((peer)) || exit 2
if [[ -n "${FAKE_PEER_FAIL:-}" ]]; then echo "  ✗ 假的第二台没跑成" >&2; exit 255; fi
if [[ -n "${FAKE_PEER_REQUIRE_CONFIG:-}" && ! -f "${SINGLEFS_MULTI_HOST_CONFIG:-/nonexistent}" ]]; then echo "  ✗ 假的第二台：找不到多机配置（副本不是 git 工作树）" >&2; exit 255; fi
rm -rf -- "${FAKE_PEER_TREE:?}/${fetch:?}"
(cd "$FAKE_PEER_TREE" && "$@"); rc=$?
mkdir -p -- "$MULTI_HOST_RUN_FETCH_DIRECTORY" && cp -a -- "$FAKE_PEER_TREE/$fetch" "$MULTI_HOST_RUN_FETCH_DIRECTORY/"
exit "$rc"
FAKE
  chmod +x "$peer_bin/cargo" "$fake_multi_host_run" "$peer_tree/scripts/hosts/run-with-memory-cap.sh" "$peer_tree/scripts/hosts/capped.sh"
  peer_case() { # peer_case <名> [变量=值…] -- <参数…>：照 run_case 那样跑一趟，换上假 multi-host-run.sh 与 peer-bin 的假 cargo，两边各记一份调用
    local name="$1"
    local -a assignments=()
    shift
    while [[ "$1" != -- ]]; do assignments+=("$1"); shift; done
    shift
    : > "$peer_calls"
    : > "$scratch/peer-cargo-$name.txt"
    out="$scratch/out-$name.txt"
    env ${assignments[@]+"${assignments[@]}"} PATH="$peer_bin:$PATH" FAKE_CARGO_CALLS="$scratch/peer-cargo-$name.txt" FAKE_PEER_CALLS="$peer_calls" FAKE_PEER_TREE="$peer_tree" \
      MERGE_AND_VERIFY_MULTI_HOST_RUN="$fake_multi_host_run" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none \
      MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-$name" MERGE_AND_VERIFY_TREE="$scratch/tree" env -u CARGO_TARGET_DIR bash "$0" "$@" > "$out" 2>&1; rc=$?
  }
  # 取回的日志里一条测试红：照本机同一套判（退 1、列出那条测试与只重跑它的命令）；四步都在第二台的副本里跑，不交共用编译目录，重编核对本次未判
  checked=$((checked+1)); peer_case peer-red -- --peer "$scratch/patch-a"
  peer_cargo_calls="$(cat "$scratch/peer-cargo-peer-red.txt" 2>/dev/null)"
  if [[ $rc -ne 1 ]] || ! grep -q 'subtracts_on_peer' "$out" || ! grep -q 'rerun-failed-tests.py' "$out" \
     || ! grep -q '^args --peer .*--fetch merge-and-verify-peer-logs -- bash -c' "$peer_calls" || ! grep -qx 'shared ' "$peer_calls" \
     || ! grep -q '重编核对本次未判：编译在第二台' "$out" || [[ "$(grep -v -- ' --version$' <<<"$peer_cargo_calls" | grep -cE "^$peer_tree	(test|fmt|clippy|build) ")" != 4 ]]; then
    echo "  ✗ 自检：--peer 时编译与测试交给第二台（一趟 multi-host-run.sh --peer、不交共用编译目录，test、fmt、clippy、build 四步都在第二台那棵副本里跑），取回的日志里一条测试红应当照本机同一套判：退 1、列出那条测试与只重跑它的命令，实际退出码 $rc；multi-host-run.sh 收到：$(tr '\n' ' ' < "$peer_calls")；假 cargo 收到：$(tr '\n' '|' <<<"$peer_cargo_calls")；输出：$(grep -E '✗|✓|第二台|重编' "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 取回的日志全绿：退 0、报测试全绿
  checked=$((checked+1)); peer_case peer-green FAKE_CARGO_MODE=green -- --peer "$scratch/patch-a"
  if [[ $rc -ne 0 ]] || ! grep -q '✓ 测试全绿（1 个测试二进制，共 2 条测试' "$out" || ! grep -q '^args --peer ' "$peer_calls"; then
    echo "  ✗ 自检：--peer 时取回的日志全绿应当退 0、报测试全绿（1 个测试二进制、2 条测试），实际退出码 $rc；multi-host-run.sh 收到：$(tr '\n' ' ' < "$peer_calls")；输出：$(grep -E '✗|✓|第二台' "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 第二台那一趟没跑成（multi-host-run.sh 退 255）：退 1、报没跑成，一步都不判
  checked=$((checked+1)); peer_case peer-failed FAKE_PEER_FAIL=1 -- --peer "$scratch/patch-a"
  if [[ $rc -ne 1 ]] || ! grep -q '第二台那一趟没跑成（multi-host-run.sh 退 255' "$out" || grep -q '✓' "$out"; then
    echo "  ✗ 自检：--peer 时第二台那一趟没跑成（multi-host-run.sh 退 255）应当退 1、报没跑成、一步都不判，实际退出码 $rc：$(grep -E '✗|✓|第二台' "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 第二台的工具链没装 rustfmt 与 clippy（文件头「--peer」）：fmt、clippy 两步之前先核组件，没装就不起这两步、报本次未跑与 rustup component add 的出路，不记红；测试与 build 照跑照判
  checked=$((checked+1)); peer_case peer-no-components FAKE_CARGO_MODE=green FAKE_CARGO_MISSING_COMPONENTS=1 -- --peer "$scratch/patch-a"
  peer_cargo_calls="$(cat "$scratch/peer-cargo-peer-no-components.txt" 2>/dev/null)"
  if [[ $rc -ne 0 ]] || grep -q '✗' "$out" || ! grep -q 'fmt 本次未跑：第二台的工具链没装 rustfmt 组件' "$out" || ! grep -q 'clippy 本次未跑：第二台的工具链没装 clippy 组件' "$out" \
     || ! grep -qF 'rustup component add rustfmt clippy --toolchain 1.98.0-x86_64-unknown-linux-gnu' "$out" || ! grep -qF '本次未跑 2 步（fmt clippy：' "$out" \
     || ! grep -q '✓ 测试全绿' "$out" || ! grep -q '✓ build 过' "$out" || grep -qE "	(fmt --all|clippy --offline)" <<<"$peer_cargo_calls"; then
    echo "  ✗ 自检：--peer 时第二台的工具链没装 rustfmt 与 clippy，应当不起 fmt、clippy、报本次未跑与 rustup component add 的出路、不记红（退 0，没有 ✗），测试与 build 照判，实际退出码 $rc；假 cargo 收到：$(tr '\n' '|' <<<"$peer_cargo_calls")；输出：$(grep -E '✗|✓|本次未跑|怎么办' "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # --copy 时副本不带 .git（文件头 --copy「副本不带 .git」）：要验的树是一个临时 git 仓（根下放一份 multi-host.env），带 --copy --peer 验；
  # 假 multi-host-run.sh 设了 FAKE_PEER_REQUIRE_CONFIG，SINGLEFS_MULTI_HOST_CONFIG 指不到文件就照真的那样退 255（副本里找不到配置）
  local copy_source="$scratch/copy-source" copy_common
  mkdir -p "$copy_source/crates/demo/src"; write_workspace "$copy_source" singlefs-checker-tier; printf 'pub fn f() {}\n' > "$copy_source/crates/demo/src/lib.rs"
  printf 'ENABLE_ACROSS_MACHINES=1\n' > "$copy_source/multi-host.env"
  { git init -q "$copy_source" && git -C "$copy_source" add -A && git -C "$copy_source" -c user.name=s -c user.email=s@s commit -qm base; } > /dev/null 2>&1
  copy_common="$(git -C "$copy_source" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)"
  : > "$peer_calls"
  out="$scratch/out-copy-peer.txt"
  PATH="$peer_bin:$PATH" FAKE_CARGO_MODE=green FAKE_PEER_CALLS="$peer_calls" FAKE_PEER_TREE="$peer_tree" FAKE_PEER_REQUIRE_CONFIG=1 \
    MERGE_AND_VERIFY_MULTI_HOST_RUN="$fake_multi_host_run" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-copy-peer" MERGE_AND_VERIFY_TREE="$copy_source" env -u CARGO_TARGET_DIR bash "$0" --copy --peer "$scratch/patch-a" > "$out" 2>&1; rc=$?
  checked=$((checked+1))
  if [[ -z "$copy_common" ]] || ! grep -qF "GATE_MUTATION_RECORDS_HOME=$copy_common（" "$out" \
     || ! grep -qxF "GATE_MUTATION_RECORDS_HOME=$(printf '%q' "$copy_common")" "$scratch/logs-copy-peer/copy-environment" 2>/dev/null; then
    echo "  ✗ 自检：--copy 时打出并写下判定库位置与多机配置（GATE_MUTATION_RECORDS_HOME=<要验的树的 git common-dir>，写进 <日志目录>/copy-environment），实际 git common-dir「${copy_common:-取不到}」，输出：$(grep -E '副本不带|✗' "$out" | head -2 | tr '\n' ' ')；copy-environment：$(tr '\n' ' ' < "$scratch/logs-copy-peer/copy-environment" 2>/dev/null)"; failures=1; fi   # gate-lint:detail
  checked=$((checked+1))
  if [[ $rc -ne 0 || -z "$copy_common" ]] || ! grep -qxF "config $(dirname "$copy_common")/multi-host.env" "$peer_calls" || grep -qF '第二台那一趟没跑成' "$out"; then
    echo "  ✗ 自检：--copy --peer 时第二台那一趟读得到主仓的多机配置（假 multi-host-run.sh 收到 SINGLEFS_MULTI_HOST_CONFIG=<要验的树>/multi-host.env），实际退出码 $rc；multi-host-run.sh 收到：$(tr '\n' ' ' < "$peer_calls")；输出：$(grep -E '✗|第二台' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 本机排不上（文件头「本机排不上」）：假包装照真包装排满退 252 时那样打两行（含现查别的机器那一行），命令不跑
  cat > "$scratch/fake-wrapper-refused.sh" <<'FAKE'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$FAKE_WRAPPER_CALLS"
echo "  ✗ 内存不够排不上：等了 3600 秒（假包装）" >&2
echo "  → 第二台此刻放得下（可用 43.0 GiB、要 8.0 GiB）：改用 bash scripts/hosts/multi-host-run.sh --peer --memory-cap 8G --label moved-from-queue-1 -- cargo test（假包装）" >&2
exit 252
FAKE
  : > "$scratch/wrapper-calls.txt"
  out="$scratch/out-refused.txt"; checked=$((checked+1))
  PATH="$peer_bin:$PATH" FAKE_WRAPPER_CALLS="$scratch/wrapper-calls.txt" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER="$scratch/fake-wrapper-refused.sh" \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-refused" MERGE_AND_VERIFY_TREE="$scratch/tree" env -u CARGO_TARGET_DIR bash "$0" "$scratch/patch-a" > "$out" 2>&1; rc=$?
  if [[ $rc -ne 1 ]] || ! grep -q '✗ test 那一趟本机内存排不上' "$out" || ! grep -qF "merge-and-verify.sh --peer --memory 8G" "$out" \
     || ! grep -qF -- "--skip-apply $scratch/patch-a" "$out" || ! grep -q '内存包装现查别的机器：→ 第二台此刻放得下' "$out" \
     || [[ "$(grep -c . "$scratch/wrapper-calls.txt")" != 1 || "$(grep -c '本次未跑：test 那一步本机内存排不上' "$out")" != 3 ]]; then
    echo "  ✗ 自检：本机内存排不上（包装退 252）时应当退 1、报那一步排不上与包装现查别的机器那一行、出路写带 --peer 重跑的整条命令（补丁已打进树，带 --skip-apply 与补丁目录），后面三步不再排、报本次未跑，实际退出码 $rc，假包装被调 $(grep -c . "$scratch/wrapper-calls.txt") 次：$(grep -E '✗|→|本次未跑|现查' "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # checker 档快档那一步（文件头「checker 档快档」）：前三格换回真的门禁 checker-tier-crash-point-replay，交它的样本根（门禁样本 unit-key-samples-green / -red 照它的 setup.sh 搭好：
  #   上一版记进判定库，这一版改了 4 条快档测试函数闭包里的单元；假 cargo 照收到的 --skip 只打要现判的结果行，-red 那一份让 test_t 判 FAILED）；
  #   合入的树是测试只挑包那几格的 narrow-tree，补丁 narrow-checker 只改 checker 档包（测试本次未跑）
  local quick_fixtures="$root/.claude/gate.d/fixtures/checker-tier-crash-point-replay.sh" quick_calls="$scratch/checker-quick-calls.txt" quick_sample_name
  rm -f -- "$SINGLEFS_WORKSPACE_LOCK_FILE"   # 锁那几格在弄坏开关下可能留下死掉的持有者：这几格各自从没有持有者起，不排到放弃时限
  for quick_sample_name in green red; do
    cp -a "$quick_fixtures/unit-key-samples-$quick_sample_name" "$scratch/quick-$quick_sample_name" \
      && (cd "$scratch/quick-$quick_sample_name" && bash ./setup.sh) > "$scratch/quick-$quick_sample_name.setup.log" 2>&1 \
      || echo "  ! 自检：门禁样本 unit-key-samples-$quick_sample_name 搭不起来（$(tail -n 1 "$scratch/quick-$quick_sample_name.setup.log" 2>/dev/null)），用它的格跟着红"
  done
  # 改了 4 条快档测试函数闭包里的单元：只现判那 4 条，复用的 4 条经 --skip 不跑
  out="$scratch/out-quick-judge.txt"; checked=$((checked+1))
  MERGE_AND_VERIFY_CHECKER_QUICK_STAGE= MERGE_AND_VERIFY_CHECKER_QUICK_ROOT="$scratch/quick-green" narrow_case quick-judge --skip-apply "$scratch/narrow-checker"
  if [[ $rc -ne 0 ]] || ! grep -q '✓ checker 档快档过.*现判 4、复用 4、拒算 0' "$out" || ! grep -qF '补丁改到 checker 档包或它依赖的包：singlefs-checker-tier' "$out" \
     || ! grep -qF 'sample-cargo-arguments: test --release --no-fail-fast -p singlefs-checker-tier --lib --tests -- --nocapture --exact --skip f1_first --skip f1_last --skip h3_test --skip test_v' "$scratch/logs-narrow-quick-judge/checker-quick.log" 2>/dev/null; then
    echo "  ✗ 自检：checker 档快档那一步：补丁改到 checker 档包、改了 4 条快档测试函数闭包里的单元，应当只现判那 4 条（另 4 条经 --skip 复用、不跑）、退 0，实际退出码 $rc：$(grep -E 'checker 档快档|^  ✗' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 同一个样本根再验一次：键都对上，现判 0、报复用 8 条，那一道一条 cargo 都没起
  out="$scratch/out-quick-reuse.txt"; checked=$((checked+1))
  MERGE_AND_VERIFY_CHECKER_QUICK_STAGE= MERGE_AND_VERIFY_CHECKER_QUICK_ROOT="$scratch/quick-green" narrow_case quick-reuse --skip-apply "$scratch/narrow-checker"
  if [[ $rc -ne 0 ]] || ! grep -q '✓ checker 档快档过.*现判 0、复用 8、拒算 0' "$out" || grep -q 'sample-cargo-arguments' "$scratch/logs-narrow-quick-reuse/checker-quick.log" 2>/dev/null \
     || ! grep -qF '全部复用，这一趟没起 cargo' "$scratch/logs-narrow-quick-reuse/checker-quick.log" 2>/dev/null; then
    echo "  ✗ 自检：checker 档快档那一步：键都对上时应当现判 0、报复用 8 条、不起 cargo、退 0，实际退出码 $rc：$(grep -E 'checker 档快档|^  ✗' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 红了：退 1，逐条列出红的测试函数与它变了的单元，出路写只重跑红的那一格的整条命令（带前缀与范围声明）
  out="$scratch/out-quick-red.txt"; checked=$((checked+1))
  MERGE_AND_VERIFY_CHECKER_QUICK_STAGE= MERGE_AND_VERIFY_CHECKER_QUICK_ROOT="$scratch/quick-red" narrow_case quick-red --skip-apply "$scratch/narrow-checker"
  if [[ $rc -ne 1 ]] || ! grep -q '✗ checker 档快档红.*红的格：crash-case-replay.*FAILED 的测试函数 1 条' "$out" || ! grep -qx '      test_t' "$out" \
     || ! grep -qF '红的 crates/singlefs-checker-tier/tests/setting.rs::test_t：' "$out" || ! grep -qF '并进写段的下一批' "$out" \
     || ! grep -qF "SINGLEFS_HEAVY_TESTS=user-request SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 checker-tier-crash-point-replay.sh 的 crash-case-replay 格; 范围=1 格;" "$out" \
     || ! grep -qF -- "--check crash-case-replay $scratch/quick-red" "$out"; then
    echo "  ✗ 自检：checker 档快档那一步：样本里 test_t 判 FAILED 时应当退 1、列出 test_t 与它变了的单元、出路写只重跑 crash-case-replay 那一格的整条命令，实际退出码 $rc：$(grep -E 'checker 档快档|^      |→' "$out" | head -5 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 没带前缀：补丁改到 checker 档包依赖的 demo（这一格起 narrow-tree 的 checker 档包依赖 demo），这一步不起、记红、给出路
  printf '\n[dependencies]\ndemo = { path = "../demo" }\n' >> "$narrow_tree/crates/singlefs-checker-tier/Cargo.toml"
  out="$scratch/out-quick-no-prefix.txt"; : > "$quick_calls"; checked=$((checked+1))
  SINGLEFS_HEAVY_TESTS= FAKE_CHECKER_QUICK_CALLS="$quick_calls" narrow_case quick-no-prefix --skip-apply "$scratch/narrow-demo"
  if [[ $rc -ne 1 ]] || [[ -s "$quick_calls" ]] || ! grep -qF '✗ checker 档快档那一步要起它的命令带 SINGLEFS_HEAVY_TESTS=user-request' "$out" \
     || ! grep -qF '补丁改到 checker 档包或它依赖的包：demo' "$out" || ! grep -qF '→ 怎么办：整条 merge-and-verify.sh 命令前面加 SINGLEFS_HEAVY_TESTS=user-request' "$out"; then
    echo "  ✗ 自检：checker 档快档那一步：补丁改到 checker 档包依赖的 demo、起它的命令没带 SINGLEFS_HEAVY_TESTS 时应当不起那一道、退 1、写明出路，实际退出码 $rc，那一道被起 $(grep -c . "$quick_calls") 次：$(grep -E 'checker 档快档|→' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 补丁没碰 checker 档包与它的依赖闭包（只改 alpha、beta）：不带前缀也不拒，报本次未跑与原因，那一道不起
  out="$scratch/out-quick-unaffected.txt"; : > "$quick_calls"; checked=$((checked+1))
  SINGLEFS_HEAVY_TESTS= FAKE_CHECKER_QUICK_CALLS="$quick_calls" narrow_case quick-unaffected --skip-apply "$scratch/narrow-two"
  if [[ $rc -ne 0 ]] || [[ -s "$quick_calls" ]] || ! grep -qF '  - checker 档快档本次未跑：补丁只改到 alpha、beta，都不是 checker 档包 singlefs-checker-tier、也不在它的依赖闭包里' "$out"; then
    echo "  ✗ 自检：checker 档快档那一步：补丁只改到不在 checker 档包依赖闭包里的 alpha、beta 时应当报本次未跑与原因、不起那一道、退 0，实际退出码 $rc，那一道被起 $(grep -c . "$quick_calls") 次：$(grep -E 'checker 档快档' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 那一道退 3 而交给它的根不是样本根（这里是 narrow-tree）：判红
  out="$scratch/out-quick-exit-three.txt"; checked=$((checked+1))
  FAKE_CHECKER_QUICK_EXIT=3 narrow_case quick-exit-three --skip-apply "$scratch/narrow-checker"
  if [[ $rc -ne 1 ]] || ! grep -qF '退 3：退 3 只在样本根上算过，交给它的根不是样本根' "$out"; then
    echo "  ✗ 自检：checker 档快档那一步：那一道退 3 而根不是它的样本根时应当判红、退 1，实际退出码 $rc：$(grep -E 'checker 档快档' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # checker 档快档那一步的内存上限（文件头「内存上限」）：假包装把收到的上限记进 FAKE_WRAPPER_CALLS（每步一行「<上限>\t<命令>」）；设了 FAKE_WRAPPER_HIT 时，
  #   起快档那一道的那一趟照真包装撞上限那样往 stderr 打「撞了内存上限 <上限>」、退 250（FAKE_WRAPPER_HIT=silent 只退 250、不打那一句），别的步剥掉上限照跑。
  #   合入的树是 narrow-tree，补丁 narrow-checker 只改 checker 档包（测试本次未跑，fmt、clippy、build、快档四趟经假包装）；线程固定 5，好拼峰值表的键
  cat > "$scratch/fake-wrapper-record.sh" <<'FAKE'
#!/usr/bin/env bash
cap="$1"; shift
printf '%s\t%s\n' "$cap" "$*" >> "$FAKE_WRAPPER_CALLS"
if [[ -n "${FAKE_WRAPPER_HIT:-}" && " $* " == *fake-checker-quick.sh* ]]; then
  [[ "$FAKE_WRAPPER_HIT" == silent ]] || echo "run-with-memory-cap: 撞了内存上限 $cap（scope 假的 Result=oom-kill，命令退出码 143）" >&2
  exit 250
fi
exec "$@"
FAKE
  memory_case() {   # $1 标签，其余是参数：narrow_case 换成记上限的假包装
    : > "$scratch/wrapper-memory-$1.txt"
    FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-memory-$1.txt" FAKE_WRAPPER_CALLS="$scratch/wrapper-memory-$1.txt" PATH="$fake_cargo_dir:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" \
      MERGE_AND_VERIFY_WRAPPER="$scratch/fake-wrapper-record.sh" MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-memory-$1" MERGE_AND_VERIFY_TREE="$narrow_tree" env -u CARGO_TARGET_DIR bash "$0" "${@:2}" > "$out" 2>&1; rc=$?
  }
  local quick_key peaks_table="$scratch/peaks-quick.tsv" quick_wrapper_calls quick_other_caps quick_given_cap
  # 峰值表里这一步那条命令的键：包装收到的是 bash <capped.sh> <线程数> env -u MERGE_AND_VERIFY_BREAK bash <那一道> --check <两格> <根>（run_capped 起它的那一条）
  quick_key="bash $CAPPED 5 env -u MERGE_AND_VERIFY_BREAK bash $scratch/fake-checker-quick.sh --check $CHECKER_QUICK_CELLS $narrow_tree"
  # 单独上限生效：--memory 8G、--checker-quick-memory 20G ⇒ 只有起快档那一趟收到 20G，别的步收到 8G
  out="$scratch/out-quick-memory-given.txt"; checked=$((checked+1))
  memory_case given --skip-apply --memory 8G --checker-quick-memory 20G --threads 5 "$scratch/narrow-checker"
  quick_wrapper_calls="$(grep -c "fake-checker-quick.sh" "$scratch/wrapper-memory-given.txt")"
  quick_given_cap="$(grep "fake-checker-quick.sh" "$scratch/wrapper-memory-given.txt" | cut -f1 | tr '\n' ' ')"
  quick_other_caps="$(grep -v "fake-checker-quick.sh" "$scratch/wrapper-memory-given.txt" | cut -f1 | sort -u | tr '\n' ' ')"
  if [[ $rc -ne 0 || "$quick_wrapper_calls" != 1 || "$quick_given_cap" != "20G " || "$quick_other_caps" != "8G " ]] || ! grep -qF '内存上限 20G，--checker-quick-memory 给的' "$out" \
     || ! grep -qF '内存上限 20G）' "$out"; then
    echo "  ✗ 自检：checker 档快档那一步：内存上限 --checker-quick-memory 给了 20G，应当只有这一步收到 20G、别的步收到 --memory 8G、成功句写出上限与取法，实际退出码 $rc，快档那一趟收到「${quick_given_cap}」，别的步收到「${quick_other_caps}」：$(grep -E 'checker 档快档' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 没给：峰值表里有这一步的记录（近几次里最大的 10 GiB、其中一次撞了顶）⇒ 乘 1.5 取 15G，写「量过的」与撞顶
  printf '10737418240\t2026-10-23T00:00:00Z\t16G\t%s\t5\t8589934592:clean,10737418240:hit,1073741824:clean\n' "$quick_key" > "$peaks_table"  # clock-times:allow 峰值测试桩时间戳
  out="$scratch/out-quick-memory-table.txt"; checked=$((checked+1))
  RUN_WITH_MEMORY_CAP_PEAKS="$peaks_table" memory_case table --skip-apply --memory 8G --threads 5 "$scratch/narrow-checker"
  quick_given_cap="$(grep "fake-checker-quick.sh" "$scratch/wrapper-memory-table.txt" | cut -f1 | tr '\n' ' ')"
  if [[ $rc -ne 0 || "$quick_given_cap" != "15G " ]] || ! grep -qF '内存上限 15G，量过的：峰值表近几次实测最大 10.0 GiB（其中有一次撞了顶，真峰值不小于它），乘 1.5 留余量' "$out"; then
    echo "  ✗ 自检：checker 档快档那一步：内存上限 峰值表里这一步近几次最大 10 GiB（其中一次撞了顶），没给上限时应当取 15G（乘 1.5）、写明量过的与撞顶，实际退出码 $rc，快档那一趟收到「${quick_given_cap}」：$(grep -E '内存上限' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 经内存包装起（文件头「自证经内存包装起」）：外层包装的两个标记，加上它的临时目录让路径变长、键超过 memory_peaks_lib 的 KEY_LENGTH_LIMIT（这里把合入的树拷到长路径下）；
  # 表里照样记这一步那条命令原文的峰值，取的上限要与上一格（不经包装、路径短）相同：15G、量过的
  local long_tree long_key long_limit
  long_tree="$scratch/singlefs-run-with-memory-cap-$$-selftest/$(printf 'p%.0s' $(seq 1 160))/narrow-tree"
  mkdir -p "$(dirname "$long_tree")" && cp -a "$narrow_tree" "$long_tree"
  long_key="bash $CAPPED 5 env -u MERGE_AND_VERIFY_BREAK bash $scratch/fake-checker-quick.sh --check $CHECKER_QUICK_CELLS $long_tree"
  long_limit="$(python3 -B -c 'import sys; sys.path.insert(0, sys.argv[1]); import memory_peaks_lib as peaks; print(peaks.KEY_LENGTH_LIMIT)' "$here/../hosts" 2>/dev/null)"
  printf '10737418240\t2026-10-23T00:00:00Z\t16G\t%s\t5\t8589934592:clean,10737418240:hit,1073741824:clean\n' "$long_key" > "$peaks_table"  # clock-times:allow 峰值测试桩时间戳
  out="$scratch/out-quick-memory-wrapped.txt"; : > "$scratch/wrapper-memory-wrapped.txt"; checked=$((checked+1))
  RUN_WITH_MEMORY_CAP_OUTER_CAP=4G RUN_WITH_MEMORY_CAP_OUTER_UNIT=singlefs-memory-cap-merge-and-verify-selftest RUN_WITH_MEMORY_CAP_PEAKS="$peaks_table" \
    FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-memory-wrapped.txt" FAKE_WRAPPER_CALLS="$scratch/wrapper-memory-wrapped.txt" PATH="$fake_cargo_dir:$PATH" \
    MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER="$scratch/fake-wrapper-record.sh" MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-memory-wrapped" \
    MERGE_AND_VERIFY_TREE="$long_tree" env -u CARGO_TARGET_DIR bash "$0" --skip-apply --memory 8G --threads 5 "$scratch/narrow-checker" > "$out" 2>&1; rc=$?
  quick_given_cap="$(grep "fake-checker-quick.sh" "$scratch/wrapper-memory-wrapped.txt" | cut -f1 | tr '\n' ' ')"
  if [[ ! "$long_limit" =~ ^[0-9]+$ ]] || ((${#long_key} <= ${long_limit:-0})) || [[ $rc -ne 0 || "$quick_given_cap" != "15G " ]] \
     || ! grep -qF '内存上限 15G，量过的：峰值表近几次实测最大 10.0 GiB（其中有一次撞了顶，真峰值不小于它），乘 1.5 留余量' "$out"; then
    echo "  ✗ 自检：checker 档快档那一步：内存上限 经内存包装起（外层包装的两个标记、路径长到键超过 KEY_LENGTH_LIMIT ${long_limit:-读不出}，这一格的键 ${#long_key} 个字）应当与不经包装那一格一样取 15G、写明量过的，实际退出码 $rc，快档那一趟收到「${quick_given_cap}」：$(grep -E '内存上限' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 峰值表算出来的比 --memory 小（1 GiB × 1.5 取整 2G，--memory 8G）⇒ 取 --memory，不比它小
  printf '1073741824\t2026-10-23T00:00:00Z\t16G\t%s\t5\t1073741824:clean\n' "$quick_key" > "$peaks_table"  # clock-times:allow 峰值测试桩时间戳
  out="$scratch/out-quick-memory-floor.txt"; checked=$((checked+1))
  RUN_WITH_MEMORY_CAP_PEAKS="$peaks_table" memory_case floor --skip-apply --memory 8G --threads 5 "$scratch/narrow-checker"
  quick_given_cap="$(grep "fake-checker-quick.sh" "$scratch/wrapper-memory-floor.txt" | cut -f1 | tr '\n' ' ')"
  if [[ $rc -ne 0 || "$quick_given_cap" != "8G " ]] || ! grep -qF '比 --memory 8G 小，取 --memory' "$out"; then
    echo "  ✗ 自检：checker 档快档那一步：内存上限 峰值表算出来的比 --memory 小时应当取 --memory 8G，不比它小，实际退出码 $rc，快档那一趟收到「${quick_given_cap}」：$(grep -E '内存上限' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 峰值表里没有这一步 ⇒ 取推的 36G，写「推的」
  out="$scratch/out-quick-memory-inferred.txt"; checked=$((checked+1))
  memory_case inferred --skip-apply --memory 8G --threads 5 "$scratch/narrow-checker"
  quick_given_cap="$(grep "fake-checker-quick.sh" "$scratch/wrapper-memory-inferred.txt" | cut -f1 | tr '\n' ' ')"
  if [[ $rc -ne 0 || "$quick_given_cap" != "36G " ]] || ! grep -qF '内存上限 36G，推的：峰值表里没有这一步的记录，取 36G' "$out"; then
    echo "  ✗ 自检：checker 档快档那一步：内存上限 峰值表里没有这一步时应当取推的 36G、写明是推的，实际退出码 $rc，快档那一趟收到「${quick_given_cap}」：$(grep -E '内存上限' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 撞了上限（包装退 250、日志里有「撞了内存上限 36G」）：不记代码红，报本次未跑，出路是只重跑这一步、上限加大一档（36G × 1.5 = 54G，总上限 64G 放得下）的整条命令，整条退 77
  out="$scratch/out-quick-memory-hit.txt"; checked=$((checked+1))
  FAKE_WRAPPER_HIT=1 memory_case hit --skip-apply --memory 8G --threads 5 "$scratch/narrow-checker"
  if [[ $rc -ne 77 ]] || ! grep -qF '本次未跑：checker 档快档撞了内存上限 36G' "$out" || grep -qF '✗' "$out" || grep -q 'FAILED' "$out" \
     || ! grep -qF "SINGLEFS_HEAVY_TESTS=user-request SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 checker-tier-crash-point-replay.sh 的 crash-case-replay、feature-gated-tests 格;" "$out" \
     || ! grep -qF -- "run-with-memory-cap.sh 54G bash $scratch/fake-checker-quick.sh --check crash-case-replay\\,feature-gated-tests $narrow_tree" "$out" \
     || ! grep -qF 'checker 档快档 本次未跑（撞了内存上限 36G）' "$out"; then
    echo "  ✗ 自检：checker 档快档那一步：撞内存上限（包装退 250，日志里有「撞了内存上限」）应当不记红、报「本次未跑：checker 档快档撞了内存上限 36G」、出路写上限加大一档（54G）的整条命令、整条退 77，实际退出码 $rc：$(grep -E 'checker 档快档|→|✗' "$out" | head -4 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 撞了上限而上限已经是本机 slice 总上限（64G）：没有更大的一档，出路写降线程上限，不打重跑命令；照样不记红、退 77
  out="$scratch/out-quick-memory-hit-top.txt"; checked=$((checked+1))
  FAKE_WRAPPER_HIT=1 memory_case hit-top --skip-apply --memory 8G --checker-quick-memory 64G --threads 5 "$scratch/narrow-checker"
  if [[ $rc -ne 77 ]] || ! grep -qF '本次未跑：checker 档快档撞了内存上限 64G' "$out" || ! grep -qF '没有更大的一档' "$out" || grep -qF '✗' "$out" || grep -qF 'SINGLEFS_RUN_SCOPE' "$out"; then
    echo "  ✗ 自检：checker 档快档那一步：撞内存上限而上限已是本机总上限 64G 时应当报没有更大的一档、不打重跑命令、不记红、退 77，实际退出码 $rc：$(grep -E 'checker 档快档|→|✗' "$out" | head -4 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 退 250 而日志里没有「撞了内存上限」（不是包装报的撞顶）：照旧按红判，退 1
  out="$scratch/out-quick-memory-silent.txt"; checked=$((checked+1))
  FAKE_WRAPPER_HIT=silent memory_case silent --skip-apply --memory 8G --threads 5 "$scratch/narrow-checker"
  if [[ $rc -ne 1 ]] || ! grep -qF '✗ checker 档快档红' "$out" || grep -qF '本次未跑：checker 档快档撞了内存上限' "$out"; then
    echo "  ✗ 自检：checker 档快档那一步：撞内存上限 退 250 而日志里没有「撞了内存上限」时应当照旧按红判、退 1，实际退出码 $rc：$(grep -E 'checker 档快档' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # --copy 与 --peer：报本次未跑与单跑那两格的命令（读上面 copy-green、peer-green 两格的输出）
  checked=$((checked+1))
  if ! grep -qF '  - checker 档快档本次未跑：带了 --copy' "$scratch/out-copy-green.txt" || ! grep -qF '  - checker 档快档本次未跑：带了 --peer' "$scratch/out-peer-green.txt"; then
    echo "  ✗ 自检：checker 档快档那一步：带 --copy 与带 --peer 时应当报本次未跑，实际 --copy 那一趟：$(grep 'checker 档快档' "$scratch/out-copy-green.txt" | head -1)；--peer 那一趟：$(grep 'checker 档快档' "$scratch/out-peer-green.txt" | head -1)"; failures=1; fi   # gate-lint:detail
  rm -rf "${scratch:?}"
  if [[ $failures -ne 0 ]]; then
    echo "    → 怎么办：看 verify() 抓 FAILED 行与退出码、编之前刷源码戳的那几处；MERGE_AND_VERIFY_BREAK 设着的话这里本来就该红"
    return 1
  fi
  echo "  ✓ 自检通过（查了 $checked 种）：测试红时退 1 并列出红的与重跑命令、补丁核不过整批停退 2、全绿退 0、没设日志目录时日志落在工作根下、"
  echo "    往编过的副本里 rsync -a 铺旧 mtime 的新内容之后照样重编、包装退 245 报显存不够不报测试红、"
  echo "    clippy 带上 check.sh 的 $expected_lint_count 条编码纪律 lint、跑 build、cargo test 挑工作区成员减掉 checker 档包、测试那一步经 test-targets-parallel.py 按测试目标并行起、"
  echo "    check.sh 读不出 lint 表或 clippy 那一行变了退 1 不编、成员里没有 checker 档包退 1 不编、一条测试都没跑退 1、"
  echo "    --copy 在自己建的副本里打补丁与编、全绿与测试红收尾都删掉副本与编译目录、MERGE_AND_VERIFY_KEEP_COPY=1 留着并打出路径、"
  echo "    补丁改过的包头一次编与再验同一批补丁都有 Compiling 行、编译目录里的产物比源码新时点名没重编的包退 1、"
  echo "    两个合入同时起后起的排队并报出前者、前者放锁后才编、锁文件里的持有者已死时报出来并接管、主工作树的锁被占着时 git worktree 加出来的树里照常验、"
  echo "    测试只挑补丁改到的包（改两个包只跑两个、没改到的逐个报本次未跑）并带上依赖它们的包、改到 Cargo.lock 与带 --all-packages 照旧全跑、只改到 checker 档包时测试本次未跑、"
  echo "    --skip-apply 时补丁反着核得过才往下验、没打在树上的退 1 不编、"
  echo "    --peer 时四步一趟交给第二台、取回的日志按本机同一套判红绿、第二台没跑成退 1、第二台工具链没装 rustfmt 或 clippy 时那一步报本次未跑与 rustup component add 的出路而不记红、"
  echo "    本机排不上时出路写带 --peer 重跑的整条命令且后面几步不再排、"
  echo "    跑完运行目录里不剩它建的编译目录（全绿、测试红、被 TERM 停三种结局都删，调用方设的 CARGO_TARGET_DIR 不删）、编译目录里还有进程开着文件时不删并报出那个进程、"
  echo "    checker 档快档那一步照门禁 checker-tier-crash-point-replay 的样本根只现判闭包里改了单元的 4 条、键都对上时现判 0 并报复用 8 条、红了退 1 并列出红的测试函数与只重跑那一格的命令、"
  echo "    没带 SINGLEFS_HEAVY_TESTS 时不起并给出路、补丁没碰 checker 档包与它的依赖闭包时报本次未跑、那一道退 3 只在样本根上算过、--copy 与 --peer 时报本次未跑、"
  echo "    checker 档快档那一步有自己的内存上限（--checker-quick-memory 给的只用在这一步，没给取峰值表近几次最大乘 1.5、表里没有取推的 36G，都不比 --memory 小）、"
  echo "    这一步撞了自己的内存上限（包装退 250 且日志里有那一句）时不记红、报本次未跑、出路写上限加大一档的整条命令、已到本机总上限的写没有更大的一档、整条退 77，退 250 而没有那一句的照旧判红、"
  echo "    经内存包装起（两个标记、路径长到键超过 KEY_LENGTH_LIMIT）与不经包装取的内存上限相同、测试那一步带 --include-ignored 把慢档一起跑并报慢档条数、"
  echo "    --copy 时打出并写下判定库位置与多机配置、--copy --peer 时第二台那一趟读得到主仓的多机配置"
  return 0
}

if [[ "${1:-}" == "--selftest" ]]; then selftest; exit $?; fi
MEMORY="${REPLAY_MEMORY_CAP:-8G}"; THREADS="$(heavy_thread_default)"; skip_apply=0; copy_mode=0; ALL_PACKAGES=0; dirs=(); invocation="merge-and-verify.sh $*"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --memory) MEMORY="${2:?}"; shift 2 ;;
    --checker-quick-memory)
      CHECKER_QUICK_MEMORY_GIVEN="${2:?}"
      [[ "$CHECKER_QUICK_MEMORY_GIVEN" =~ ^[1-9][0-9]*[KMGT]$ ]] || die "--checker-quick-memory 的上限 $CHECKER_QUICK_MEMORY_GIVEN 写法不对" "正整数加 K / M / G / T（1024 进制），单位必写，例 36G"
      shift 2 ;;
    --threads) THREADS="${2:?}"; shift 2 ;;
    --skip-apply) skip_apply=1; shift ;;
    --copy) copy_mode=1; shift ;;
    --all-packages) ALL_PACKAGES=1; shift ;;
    --peer) PEER_MODE=1; shift ;;
    --*) die "认不出参数 $1" "只认 --memory <上限>、--checker-quick-memory <上限>、--threads <N>、--skip-apply、--copy、--all-packages、--peer" ;;
    *) dirs+=("$1"); shift ;;
  esac
done
[[ ${#dirs[@]} -gt 0 || $skip_apply -eq 1 ]] || die "没有给补丁目录" "merge-and-verify.sh [--memory 16G] [--checker-quick-memory 36G] [--threads N] <补丁目录> …；只验不打用 --skip-apply"
if [[ -n "${MERGE_AND_VERIFY_LOG_DIRECTORY:-}" ]]; then logs="$MERGE_AND_VERIFY_LOG_DIRECTORY"
elif work_root="$(python3 "$root/.claude/hooks/lib_work_root.py")"; then logs="$work_root/merge-and-verify/$(TZ=Asia/Tokyo date +%F)-$$"
else die "取不到工作根（.claude/hooks/lib_work_root.py 跑不起来）" "恢复那份库，或设 MERGE_AND_VERIFY_LOG_DIRECTORY 指到日志目录"; fi
tree="${MERGE_AND_VERIFY_TREE:-$root}"
# 这一趟的编译目录（文件头）与 --copy 的副本：是这里建的才登记进 RUN_TARGET / COPY_TREE，收尾的 EXIT trap 只删登记了的
[[ ! -e "$logs/repo" || copy_mode -eq 0 ]] || die "副本位置 $logs/repo 已经有东西，不往里铺、也不删它" "换一个日志目录（MERGE_AND_VERIFY_LOG_DIRECTORY 不设就是新的 <日期>-<进程号>），或先看清那里是什么"
[[ -n "${CARGO_TARGET_DIR:-}" || ! -e "$logs/target" ]] || die "这一趟的编译目录位置 $logs/target 已经有东西，不往里编、也不删它" "换一个日志目录（不设 MERGE_AND_VERIFY_LOG_DIRECTORY 就是新的 <日期>-<进程号>），或设 CARGO_TARGET_DIR 指到要用的编译目录（那一份收尾不删）"
mkdir -p "$logs" || die "建不了日志目录 $logs" "看工作根在不在、可不可写"
trap finish EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
if [[ -z "${CARGO_TARGET_DIR:-}" ]]; then
  RUN_TARGET="$logs/target"
  mkdir -- "$RUN_TARGET" || die "建不了这一趟的编译目录 $RUN_TARGET" "看日志目录 $logs 可不可写、盘满没满"
  CARGO_TARGET_DIR="$RUN_TARGET"; export CARGO_TARGET_DIR
  echo "  这一趟的编译目录：$RUN_TARGET（这里建的，收尾删）"
else
  echo "  编译目录：$CARGO_TARGET_DIR（调用方设的 CARGO_TARGET_DIR，收尾不删）"
fi
if ((copy_mode)); then
  COPY_TREE="$logs/repo"
  echo "  建副本：$tree → $COPY_TREE（编译目录 $CARGO_TARGET_DIR$([[ -n "$RUN_TARGET" ]] && echo '，收尾删' || echo '，调用方设的，收尾不删')）"
  python3 "$STAMP" sync "$tree" "$COPY_TREE" --target-dir "$CARGO_TARGET_DIR" > "$logs/copy-sync.log" 2>&1 \
    || die "建副本没成（日志 $logs/copy-sync.log）" "照那份日志修（rsync 在不在、盘满没满）再跑；建了一半的副本收尾照删" 1
  copy_environment "$tree" "$logs"   # 副本不带 .git：判定库位置与多机配置从要验的树取（文件头 --copy）
  tree="$COPY_TREE"
fi
# 主工作区合入锁（文件头）：不带 --copy 时拿，EXIT trap 放
if ((copy_mode == 0)); then
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "ignore-workspace-lock" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=ignore-workspace-lock：不拿主工作区合入锁"
  else
    python3 "$WORKSPACE_LOCK" acquire --root "$tree" --holder-pid "$$" --command "$invocation" \
      || die "主工作区合入锁没拿成（上面一行是原因），一个补丁都没打、没编" "照上面那一行的出路办；要在别处验就加 --copy 在自己的副本里打与编" 1
    WORKSPACE_LOCK_HELD=1   # 收尾的 EXIT trap（finish）放
    export SINGLEFS_WORKSPACE_LOCK_HOLDER="$$"
  fi
fi
# 算改过的包用的补丁目录（文件头「改过的包都重编了没有」）：进树之前转成绝对路径，--skip-apply 时也留着
for dir in ${dirs[@]+"${dirs[@]}"}; do
  if [[ -d "$dir" ]]; then CHECK_PATCH_DIRS+=("$(cd "$dir" && pwd)"); else CHECK_PATCH_DIRS+=("$dir"); fi
done
cd "$tree" || die "进不去要编的树 $tree" "在仓里跑；MERGE_AND_VERIFY_TREE 只给自证用，平时不设"
if [[ $skip_apply -eq 1 ]]; then dirs=(); check_patches_applied "$logs"; fi
verify "$logs" "${dirs[@]}"
