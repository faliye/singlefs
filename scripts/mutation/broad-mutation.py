#!/usr/bin/env python3
# admission: always 每次调都现算此刻源码上每条变异的单元键再查判定库；哪些条要跑由键与沿用判据定，上一次的结论不替这一次作保
# run-condition: command git python3
# task-server-shards: *   经统一任务服务交的一律分片：计划、两片、合并（下面「分片」）；--selftest、--list-items、--help、-h 服务不分
# task-server-compiles: yes
# broad-mutation-driver-signature: singlefs 广谱变异逐条驱动（改名或拷走的一份靠这一行认，别删）
"""广谱变异的逐条驱动（.claude/kb/checks-owed.md 的 C489（singlefs-core 的广谱变异对今天的代码没有结论）：一直跑、中断要续接不从头来、一条一条可点名；
剪枝的单元是函数，广谱变异一条一条按内容算，不按 HEAD、不按提交算）。

拆成的几份（本文件只留文件头、main、在运行目录里跑的入口与对各份的再导出；代码在同目录的这几份里，按这个次序往前 import、没有环；别处按本文件取名字，名字一个不少）：
  broad_mutation_base.py  标准库与仓内库的载入、全部模块级赋值（常量、表、正则、列名）、入口与拆出的各份模块的路径表。模块级可变状态（TEST_MODE、PRUNE_MODULE、REUSE_CHECKS_MODULE、selftest_fixtures）不在这一份：各在读写它的函数所在的那一份。
  broad_mutation_judging.py  选条、沿用、结论分类（Judgement）、GPU 一趟的判法（GpuJudging）、起一条变异的步、读命令行与自证变量的小函数（refuse、say、note、test_mode、break_is 这类）。只 import base。驱动（这一份与别的份）的代码不进键。
  broad_mutation_library.py  判定库（Library）、读表、取库目录与源码树、导出范围、清单的生成与缓存、规矩库与复用三问库的载入、变异补丁的施加与切分。
  broad_mutation_runner.py  步的跑法：按步取命令、跑一步（Runner）、libtest 与 nextest 结果的认法、工具版本、改坏与还原源码。
  broad_mutation_gpu.py  GPU 那一趟：GPU 结局的分类之外的部分（门控测试文件、编译参数、卡池取卡、判定行的读写、GpuPass、GPU 一趟的编译与跑）。
  broad_mutation_run.py  一次跑：选条、算键与读库的提速、汇总（summarise）、命令行的参数、跑全部项（run_items）、复用哨兵、分片的计划与计数。
  broad_mutation_selftest_a.py  自证（一）：Selftest 类与基础、状态、表、环境、同步、剪枝、复用、算键提速几组格。
  broad_mutation_selftest_b.py  自证（二）：基线、服务、编译目录、登记、GPU 等几组格，格函数表、串并行、run_selftest。
弄坏开关的代码在哪一份（开关表仍在下面各节，名字出现在哪一份的 raw_break / break_is 里，那一条开关的代码就在那一份）：
  broad_mutation_judging.py：accept-unknown-item、baseline-by-packages、baseline-refused-step-whole、carry-without-store-check、coverage-selection-dropped、gpu-all-items、gpu-not-run-as-missed、gpu-red-not-caught、gpu-test-threads-dropped、missed-carried、named-items-always-rerun、no-carry-without-arguments、own-libtest-failure-line、refused-carried、rerun-done、select-by-commit、stage-two-missed-as-caught、stage-two-not-by-binary、state-is-last-row、test-variables-anywhere、unknown-exit-stops、unviable-by-exit-code
  broad_mutation_library.py：break-guard-after-parsing、build-directory-in-home、exclude-re-dropped、interrupted-run-directory-kept、list-cache-unchecked、no-row-flush、read-only-volatile-refused、run-directory-kept、run-directory-open-files-unchecked、service-build-directory-ignored、test-variables-anywhere、unreachable-as-empty-library
  broad_mutation_runner.py：baseline-by-threads、doc-tests-skipped、no-environment-check、progress-never、stage-two-baseline-red-generic、stage-two-full-cargo-test、term-leaves-step-running、test-steps-configuration-unpassed
  broad_mutation_gpu.py：gpu-also-when-cpu-caught、gpu-builds-interleaved、gpu-carry-dropped
  broad_mutation_run.py：changed-units-batch-oldest-row、composition-write-unmemorised、interrupted-as-done、item-counts-dropped、key-cache-digest-unchecked、key-cache-sample-unchecked、no-environment-check、rocksdb-entry-unchecked、shard-seconds-without-locator、summary-gate-silent、summary-mismatch-silent、test-steps-configuration-unpassed、unknown-option-accepted、unreachable-as-empty-library
  broad_mutation_selftest_b.py：selftest-chain-result-dropped

  broad-mutation.py [选哪些条] [怎么跑]             不带选条参数就是剪枝：把 --source 给的工作树（默认本仓的工作区）换进库目录的专用树，由 cargo-mutants
                                                   生成此刻的清单，每条变异现算单元键过沿用判据（scripts/mutation/mutation-prune.py），沿用得了沿用、沿用不了现判；
                                                   成功那句报「现判 N、沿用 M、拒算 R」。重跑同一条命令就是续跑。
                                                   只经统一任务服务交（task-server/task submit -- python3 scripts/mutation/broad-mutation.py [选哪些条]），服务分两片判（下面「分片」）
  broad-mutation.py --refresh-list                 只换树、生成清单、把清单按单元记进库（不编不跑），之后 --list-items / --summary 在同一份源码上列得出全部条
  broad-mutation.py --list-items                   逐行打「序号 <TAB> 包 <TAB> 变异名 <TAB> 状态 <TAB> stale=<同一处变异键对不上的判定行数> <TAB> key=<此刻的键>」，不编不跑、不起 cargo；
                                                   状态是两趟合起来的（CPU 那一趟没红又要 GPU 那一趟的条取合起来的状态，见下「GPU 那一趟」）
  broad-mutation.py --summary                      六栏条数与提交门禁那一行（见下「输出」，六栏按两趟合起来的状态数，另报 GPU 那一趟要几条、复用几条、本次未跑几条）；不编不跑、不起 cargo，只读判定库
  broad-mutation.py --selftest                     自证（假清单、假测试命令、假 GPU，不起 cargo；判定库只写临时 prune-server：照 scripts/prune-server/prune_server_client.py
                                                   文件头「命令行」--with-temporary-service 那一条包一层起，裸跑退 2）；环境变量 BROAD_MUTATION_SELFTEST_CELLS=<组名,…> 只跑点名的几组格（开发时分段跑，
                                                   组名见 selftest_cell_functions，有的组接着前面组留下的库目录；不设就是全部，门禁与提交前跑的都不设）；
                                                   各组分成互不依赖的几串同时跑（下面「--selftest 的并行」），成功句报「并行 N 路」；
                                                   自证的夹具（假一步与假清单驱动的脚本、假源码树、计划表）在同目录的 lib_broad_mutation_selftest_fixtures.py，
                                                   只在 --selftest 开头载入

分片（判条的用法——剪枝跑、--item、--from、--package——经统一任务服务交的一律分片：文件头「# task-server-shards: *」，服务拆成三步、
  每步照原命令起这一份一次，环境里 TASK_SERVER_SHARD_PHASE 说是哪一步，变量的写法在 task-server/task_server_lib.py 文件头「分片」；shard_phase_main）：
  plan   拿库目录的锁、换树、生成清单，照原命令的选条参数算这一趟要现判的条（与剪枝跑同一份 select_items），按上一次现判的编译加测试秒数（没记过的按记过的中位数，
         一条都没记过按 1 秒）从长到短放进合计用时少的那一堆；选条参数罩得住而这一趟不现判 CPU 那一趟的条按序号单双分进两堆（GPU 那一趟与复用哨兵在各片自己那一堆里办）。
         每个不空的堆一片，写 <计划目录>/shard-<片号>.items（一行一个变异名）与 plan.json（只写 label，不钉台）；罩得住的一条都没有写 0 片。
  shard  照 TASK_SERVER_SHARD 读自己那一堆，只在那一堆里照剪枝跑判（CPU 那一趟、GPU 那一趟照卡、复用哨兵都在这一堆里）；库目录的锁、运行目录、日志与 tree.synced
         放 <库目录>/shards/shard-<片号>/（两片可在两台同时跑，判定照样写进服务里同一份库）；计划那一步写的变异名有一条在此刻清单里找不到，退 2（树在两步之间变了）。
         收尾往 TASK_SERVER_SHARD_OUTPUT_DIRECTORY 写 counts.json（现判、沿用、拒算与 GPU 那一趟的几个数）。
  merge  拿库目录的锁、换树、生成清单，照原命令的选条参数核账：CPU 那一趟还要现判的条（拒算的与复用哨兵抽中的除外）有一条就退 1、逐条点名，一条都不现判；
         GPU 那一趟还没有结论的条记本次未跑（BROAD_MUTATION_GPU_NOT_RUN reason=not-judged-in-shards），不在这一步跑；复用哨兵照抽。
         成功句接各片 counts.json 合起来的数；TASK_SERVER_SHARD_MISSING 点名的片（人手单独取消的）照缺报，那几片的条在核账里报没判。
  不是服务分片起的判条用法（没有 TASK_SERVER_SHARD_PHASE）退 2，出路写经服务交；--list-items、--summary、--refresh-list 照常（plan 那一步见到它们写 0 片）。

算键提速（不改键、不改状态；run_items、read_only_judgement 里装，函数在 speed_up_judgement 起的那一节，selftest_key_speed_cells 对拍）：
  键：每条变异的键先算齐（prefill_keys）再算状态；互不依赖的活按 --threads 并行（mutation-prune.py 的 prefill_records：fork 出子进程，内存放得下才开，
    路数取线程数、每路至少 8 条、cgroup 内存余量除以每路估计；按条的次序收，某一路没成就打一行说明退回串行）；同一个容器单元里的几条变异共用一次闭包键的计算
    （cache_closure_keys），组成清单惰性地现算（写库与报「变了哪个单元」时才用）。分片那一趟里计划那一步算完把每条的键写进计划目录的 key-cache.json，
    各片与合并那一步在摘要（专用树导出范围的文件摘要、全局那几行、驱动与规则库与它们用的两份库的摘要）对得上、抽查十二条现算相同时读它，不对就现算；
    只读的用法（--summary、--list-items）被判的是 --source 本身，不读也不写这份缓存。
  状态：不缓存、不并行（判定库在计划与各片之间会写进新判定），只在分片那一趟里先筛出自己那一堆再算。
  这一节的函数没改 Judgement 与 UnitTree 的方法：提速都是装在实例与规则库模块上的外挂，库里的判定不因它们作废。

选哪些条（不给 --item 就是此刻清单里沿用不了的全部条，按序号）：
  --item <序号或变异名>   可给多次，只跑点名的里面沿用不了的；沿用得了的跳过并逐条说一句。不与 --from / --package 同用
  --from <序号>           只要序号不小于它的
  --package <包>          只要这个包的
  序号是此刻这份清单里的次序（先 core 后 checker，同包里按文件、起止、替换文排），只当索引，不进任何键。
怎么跑：
  --source <目录>         被判的工作树（主工作区或一份副本），默认本仓根；驱动只读它，变异改的是库目录里的专用树
  --stage-two-eatmydata   第二段的文档测试与 验证包也经 eatmydata 跑（默认不经）
  --threads <N>           线程数，默认取 SINGLEFS_THREAD_CAP，没设取每台默认重活线程数（.claude/heavy-thread-default 的登记值）；测试步的 -j 照它，编译与 libtest 的并行度读统一任务服务交的 CARGO_BUILD_JOBS、RUST_TEST_THREADS（驱动不另套线程包装）
  --suspect-streak <K>    连续 K 条「无效」、或连续 K 条「测试那一步不到 --suspect-test-seconds 秒就红的抓到」，就在没改过的树上把那一步重跑一遍：
                          红了退 6 报「疑似环境坏了」并列出这 K 条，绿了接着跑。默认 8；--suspect-test-seconds 默认 0.2
  --stop-grace <秒>       被 TERM / INT 停时等在途那一步退出多久，默认 10，过了还在就 KILL
  --lock-wait <秒>        库目录的锁被上一趟留下的进程占着时等多久，默认 120
  --home <库目录>         默认 <数据根>/broad-mutation-units（数据根取 .claude/hooks/lib_work_root.py 的 shared_data_path，由多机配置定，不经 ~ 取）：
                          只放这一趟的运行目录、日志与锁，判定不在这里（见下「判定库」）；要写库的用法落在开机会清空的目录里拒（只读的几种不写库、不拒）
  只读的几种（--list-items、--summary）另认：--toolchain-from <文件>（rustc -Vv 的输出换成这份文件；只有弄坏开关 toolchain-version-keyed 下进键）、
  --tools-from <文件>（工具版本换成这份文件，一行一个）：提交门禁的样本用，真跑与写行的用法不认。

库目录（--home；一处，固定，不随趟、不随判法换目录、没有版本目录；判定不在这里，在下面「判定库」）：
  runs/run-<进程号>-<随机>/   这一趟的运行目录（自己用完自己删：编译生成物每趟新建、跑完删，判定库照留、按单元键复用）。
                   拿到库目录的锁之后建（Library.begin_run），正常结束、拒绝退出、被 TERM / INT 停都在 main 的收尾删掉（Library.end_run；
                   删之前经 scripts/hosts/run_build_directory.py 核本机没有进程开着里面的文件，有就不删、逐个报进程，这一趟别的都成了也退 9）。
                   被 KILL 的那一趟留下的，下一趟拿到锁之后先删（同样先核开着的文件；里面有在途记录的报一句被打断在哪一条）。
                   环境里给了 CARGO_TARGET_DIR（这个驱动没有自己指运行目录的变量，只认它；文件头的会编声明「# task-server-compiles: yes」让统一任务服务
                   给这一次尝试一份，撞上限重来时接着给同一个）：运行目录不建在 runs/ 下，
                   用 <那个目录>/broad-mutation-run/（Library.begin_run 的 service_build_directory），收尾不删（服务在任务有了结局时删）；
                   上一次尝试被停在半路留下的在途记录在里面的，报一句被打断在哪一条、删掉在途记录，换树把改过的那份文件照源码拷回（内容不同的才写），
                   编译目录与没变的源文件照留，重来的那一次只付增量编译。没给照上面在 runs/ 下新建、跑完删。
                   弄坏开关 service-build-directory-ignored，自证 selftest_service_build_directory_cells。里面四样：
    tree/          专用树：开跑把 --source 那棵树的导出范围逐文件拷进来（写的文件 mtime 取此刻），变异就地改这里的一个文件、判完拷回原件：
                   编译目录在这一趟里跨条复用，每条只付增量编译。导出范围取门禁 crates-mutation-replay 登记的
                   拷贝范围（scripts/mutation/crates-mutation-rows.py 的 copied_paths，去掉 research/ 下的）加 .cargo/
    target/、target-coverage/   CARGO_TARGET_DIR；scratch/ 是每一步的 TMPDIR（每一步开跑前清空），scratch-coverage/ 是覆盖那一步的产物
    inflight.json、inflight.original   在途记录：改文件之前先存原件、记下改的是哪一条，判完照它拷回（临时件 inflight.restore 在运行目录里，不在树里）
  tree.synced      上一次换树拷了几个文件、什么时候（只当记录，不进任何键）
  logs/            没过的那一步的输出（每条每步一份，后一趟同名的盖掉前一趟）
  lock             库目录的排他锁（见下「中断续接」）
判定库（.claude/rules/verification.md「判定库只有一份」，不留旧库的残留）：全在 prune-server 里那一份，读写只经
  scripts/prune-server/prune_server_client.py，本地一个文件都不建、不读。服务连不上、停了、忙到上限，客户端退 75（不当「库里没有」现判）；配置不对、回应解不开退 3。
  键 = broad-mutation[@命名空间]/broad-mutation-units/<表>/<条目>（客户端文件头「键空间」：库目录落在临时目录底下的另占一份命名空间，自证与门禁样本用；
  别处的不带，两台读写同一份）。各张表：
  verdicts/<次序>          判定（只追加）：一条一行，十六段（见下「一条一条跑」），键是那一条变异的单元键（见下「键」），另记结局、哪一段判的、抓到它的测试与它的测试键、
                           覆盖者键、编译与测试秒数、判定日期、跑法、序号与变异名与位置（只当索引）、拒算形态、备注
  gpu-verdicts/<次序>      GPU 那一趟的判定（只追加，十三段：键、序号、包、变异名、结局、抓到它的测试、测试键、覆盖者键、GPU 趟自己的键、测试秒、判定日期、跑法、备注）：
                           键是那条变异的单元键（与 verdicts/ 里 CPU 那一趟的同一把），GPU 那一趟的结局只记在这里，不进 verdicts/；沿用判据同 CPU 那一趟（见下「GPU 那一趟」）
  baselines/<次序>         基线（只追加）：一行五段：基线键、步名、结局（ok / exit-<码>）、时刻、线程数；
                           基线键按这一步真跑到的判定的单元闭包算（见下「基线」）；步名只当索引，不进键。段数不是五段的行不认（那一步照跑）
  baseline-entries/<次序>  按条记基线（只追加，见下「基线」）：一行六段：记录键、步名、线程数、kind（test / rest / binary）、二进制（只当索引）、时刻；
                           段数不是六段的行不认（那一份照跑）
  coverage/<次序>          覆盖记录（只追加）：一行七段「记录键、哪一段、二进制、用例、执行到的单元（种类:指纹，JSON）、profraw 份数、日期」；
                           记录键是那条测试的测试键（验证包逐个二进制的取那个二进制里各条测试的测试键），测试的闭包不变就照用，变了那几条重跑插桩
  lists/<清单的单元键>     清单按单元存：值是这个单元里的变异（JSON；mutation-prune.py 的 list_unit_key、list_entry），同一个单元键再写就盖掉；
                           被变异包里每一个单元都记（没有变异的记空表），--list-items 与 --summary 照它在此刻的源码上复原清单，不起 cargo
  compositions/<键>        每把键（变异键、测试键）的组成清单：单元行（种类、限定名、文件:起止、指纹）、全局那几行与 edition；
                           作废时照它报出变了的是哪几个单元（--list-items、--summary 的 NO_CONCLUSION 那一行）
  <次序> = <写入时刻的纳秒，二十位>-<进程号>-<这个进程里的序号>-<随机>（order_stamp）：一个进程里只增，按键升序扫回来就是写入的次序
    （同一把键的几行，状态取最后一个定论，见下「结局」）；一行一个条目，几个进程、两台同时追加不互相盖。
    服务回确认（WAL 同步之后）才算落了；没确认就退出的那一行这一趟丢了，那一条下一趟照现判。
  判定行不带校验列：库在服务里，不是谁都手改得了的本地文件；四族都只经客户端写，一样不带。

键（规则与弄坏开关在 scripts/mutation/mutation-prune.py 文件头；单元与闭包由 scripts/gate/lib_function_units.py 算）：
  变异键 = 容器单元（包住这条变异的最小单元）的单元闭包键（闭包里每个单元的种类与去注释指纹、数据单元摘要、入口目标的生效 cfg、
    各包的 edition）+ 变异在容器里的相对位置（去注释的词法单元下标）+ 原文 + 替换文。
  全局那几行（每把键里都有）平常是空的：键只留被改坏的那一处与测试的闭包，构建环境一样都不进。
    编译变量（RUSTFLAGS、RUSTDOCFLAGS、RUSTC 与几个 *_WRAPPER、CARGO_BUILD_*、CARGO_TARGET_*）与它们指的程序、cargo 配置的 build / target / env / unstable 解析值、
    清单的 lints、rustc 与 cargo 的版本（rustc -vV、cargo nextest / cargo-mutants / llvm-profdata 的版本、rust-toolchain 的值）与 Cargo.lock 里的依赖版本都不进，改它们库里的判定一条都不作废。
    Cargo.toml、.cargo/ 的原文不进（只改注释照样沿用）；排除表不进（加一条排除只让被排除的那几条测试不再算数：抓到它的那条测试进了排除表的现判）；
    --threads 不进（换了线程数照样沿用别的条，基线也照样沿用）；编译档（环境变量 CARGO_PROFILE_*、cargo 配置与 Cargo.toml 的 [profile]）与线程数（CARGO_BUILD_JOBS、[build] 的 jobs）、
    驱动与算键库（本驱动各份、mutation-prune.py、scripts/gate/lib_function_units*.py）的代码都不进；改它们库里的判定一条都不作废，只被改坏的那一处与抓到它的测试的闭包变才作废。
  测试键 = 抓到它那条测试（nextest 的「二进制 id 用例名」，libtest 的照日志里的 Running / Doc-tests 行认出二进制）的单元闭包键；
  覆盖者键 = 静态闭包里有容器单元的每一条测试（第一段的、验证包、被变异包的文档测试；标了 #[ignore] 的 cargo 不跑，不算；登记的崩溃枚举用例与放量分项不算）的测试键多重集。
沿用判据（mutation-prune.py 的 carry_decision）：判定库里键相同的那几行，抓到的另要测试键与此刻的相同、那条测试不在排除表里；
  没红的要覆盖者键相同才算数（--summary 与提交门禁按它定状态），
  但真跑一律不沿用、每趟现判（下面「记红、撤绿、下趟现判」）；无效的键相同就沿用；驱动错不是结论，每趟照选中现判。
  拒算（容器单元找不到、闭包里有 lib_function_units 认不出的形态、抓到它的测试认不出）的条不沿用、每趟现判，成功那句与 --summary 报条数；
  抓到它的测试认不出时那一行另记覆盖者键，--summary 照它核（提交门禁认它：判定的内容与此刻相同、能碰到它的测试一条没变）。

一条一条跑：每条变异一次独立的判定，判完立刻往判定库 verdicts/ 写一行、等服务回确认（驱动自己不经 eatmydata，只有测试命令经它）。一行十六段，制表符分隔：
  键、序号、包、变异名、结局、哪一段判的（one / two）、抓到它的第一条测试、测试键、覆盖者键、编译秒数、测试秒数、判定日期、跑法、位置、拒算形态、备注
  跑法：fresh（现判）。
  读判定库时段数不对的行不算判定（那几条照沿用不了选、续跑时重跑），--summary 把它们报成对不上（点名条目）、退 1。
  结局：caught 抓到、missed 没红（两段都绿）、unviable 无效（编译那一步编不过）、stage-one-only 只过了第一段、
        driver-error 驱动错（某一步退了认不出的码，不是这条变异的结局）。
        结局只认这五种：别的写法是认不出的结局，那一行不算判定（那一条照沿用不了选、点名时现判），--summary 报对不上、退 1；
        出路是查清那几个条目是谁经客户端写进来的（驱动只写这五种结局），用客户端把点到的条目改写成成形的行（服务没有删键）。
  一条的状态不取最后一行：键相同、沿用判据认的那几行里有 caught / missed / unviable 的取其中最后一行；
  都没有而有 stage-one-only 行的是 stage-one-only；只有 driver-error 行的是 driver-error，续跑时重试；一行都没有是 none。
  记红、撤绿、下趟现判（「判红」照提交门禁 broad-mutation 的 gate-verdict 格与 --summary 的 BROAD_MUTATION_GATE 行：missed 没红 > 0 判红，
    没结论（只过了第一段、驱动错、none）也判红但那是「没有定论」，不是一条定论；无效 unviable 与抓到都算有结论、不判红）：
    判红的定论是没红。同一把键的定论（caught / missed / unviable）里最后一个是 missed 的，红照键记着，前面同一把键的好结论（抓到、无效）撤掉，下一趟这一条现判、不沿用：
    不带选条参数的剪枝跑、点名的跑都现判一次，它也不进复用哨兵的候选；现判又出没红，下一趟还是现判（没红每趟现判，直到判出抓到或无效）。
    单独一行没红、与先抓到后来同一把键又判没红，一样处理。抓到、无效照键沿用。--summary、--list-items 与提交门禁不受影响（状态本来就取最后一个定论，没红照没红数）。
    判法在 Judgement.state（CPU 一趟）与 GpuJudging.state（GPU 一趟）里，只在真跑的选条（for_run）下撤绿。
    不放在 mutation-prune.py 的 carry_decision 里：carry_decision 判的是「这一行此刻还算不算数」，--summary、提交门禁也靠它定没红的状态，没红在那里一律不算数的话没红数会少，
    同键更早的抓到还会重新冒出来当状态。
    要 GPU 那一趟的条（CPU 没红、又有带 gpu 特性的测试够得着）红不红看合起来的状态：GPU 趟的真实状态是没红才算红，CPU 一趟的没红行才不沿用；GPU 趟抓到 / 无效 / 还没有结论 /
    没结论的，CPU 一趟的没红行照沿用（gpu_decided_keys 在选条之前交给 Judgement.state，GPU 趟没红的那一条两趟都现判）。
    弄坏开关 missed-carried 下没红照旧按覆盖者键沿用。
两段式：第一段每条只跑验证包之外三个包（singlefs-checker、singlefs-core、singlefs-format）的测试（eatmydata + cargo nextest，遇第一个失败即停）；第一段没红的先落一行 stage-one-only，再进第二段。
  第二段只补第一段没跑到的：被变异的两个包的文档测试（cargo test --doc）、验证包里的全部测试函数
  （cargo test --release -p singlefs-verification --lib --tests；登记成 crash-case: 的崩溃枚举用例与放量分项经 --skip 排掉，它们另走按块那一路，
  排除单与门禁按条那一趟的 --exclude-registered 是同一张：scripts/gate/admission.py 的 registered_test_functions，读法只在那里一份）；
  都没红才记 missed。标了 #[ignore] 的用例 cargo 不跑，驱动不算（只剩要显卡、量显存与量速度那几条）。第二段被打断的条，续跑时只补第二段。
GPU 那一趟（第二段补一趟带 gpu 特性的 验证包，出处 .claude/kb/checks-owed.md 的 C489（singlefs-core 的广谱变异对今天的代码没有结论）的 GPU 普查）：
  CPU 那一趟（上面的两段）全跑完之后，对「CPU 那一趟最后落的结论是没红、又有带 gpu 特性的测试够得着」的条另补一趟带 gpu 特性的 验证包（verification.md 里带特性那一趟）：
    cargo test --release -p singlefs-verification --features <tests/ 里 cfg 点到的特性> --lib / --test <挂特性的测试文件>… -- --nocapture --test-threads 1
    特性与挂特性的文件每趟从 验证包 tests/ 顶层每个 .rs 的 #[cfg(…)] / #![cfg(…)] 现读（gated_test_files，同门禁 crash-point-replay 的 feature-gated-tests 格），
    一个文件都不挂特性、或特性里没有 gpu：没有这一趟。先编后跑：要现判的各条逐条先编完（改坏、cargo test --no-run、拷回原件），
    编得过的把编译步日志里 cargo 报的每个测试可执行文件（「Executable <源文件> (<可执行文件>)」）硬链接进编译目录底下的 broad-mutation-gpu-stash/<序号>/
    （链不了就拷，记下大小与修改时刻；真跑而一个 Executable 行都没有、或报的文件不在，记驱动错）；再在没改过的树上编一次基线（编译目录里留下的是没改过的树的产物），
    之后才只跑测试步：基线，再逐条照原命令跑 cargo test，cargo 起测试二进制时经执行器（CARGO_TARGET_<host>_RUNNER 指 stash 里的 runner）
    换成这一条留下的那一份（BROAD_MUTATION_GPU_STASH 指目录；cargo 的工作目录、环境、起的次序、Running 行、第一个二进制红就停都照 cargo test，树没改、cargo 不再编）；
    全部跑完删掉 stash。留下的那一份在跑之前大小或修改时刻变了（后面的编译就地改写了它）、或执行器报 BROAD_MUTATION_GPU_STASH_MISSING（cargo 要起的这一条没留下）记驱动错。
    弄坏开关 gpu-builds-interleaved：每一条在跑它的测试步之前才编（基线先编先跑），自证 selftest_gpu_build_first_cells。
    编不过（无效）的条照 gpu_classify_build 判，基线测试步绿了才落行、不跑测试步；测试步直接起，不另套清场（见下面「不领卡」），
    GPU 测试各占卡的显存所以 --test-threads 1；BINDGEN_EXTRA_CLANG_ARGS 取 scripts/crash/crash-amplification-lib.sh
    的 CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS（与崩溃放量同一个值，编译产物才共用）；--threads 同 CPU 那一趟，每一步不设内存上限、不限时（见下「内存与限时」）。
  要不要这一趟：一条两趟里任一趟红算抓到，所以只给 CPU 那一趟没红（missed）的条跑；还要有一条这一趟跑得到的测试（验证包的库与挂特性的测试文件里不标 #[ignore] 的）带 gpu 特性的静态闭包
    含这条变异的容器单元、又真引到 gpu 原子（feature = "gpu" 带真值 true，闭包键里 cfg 原子的那一项）——改坏对象落在带 gpu 特性才编进来的代码、或判器单元闭包含它，都是这一条的特例；
    够不着它的、够得着却不引 gpu 的，这一趟帮不上，照旧只有 CPU 那一趟。要跑的目标取这些测试所在的目标（lib、挂特性的文件）。
  结局：编不过 unviable，测试步退 101（有测试没过）caught（抓到它的测试取第一条没过的；--nocapture 加 --test-threads 1 下结果行常被测试自己的输出打散，认不出就读收尾的 failures: 名单），
    全过且跑到了测试 missed；日志里有 NOT_RUN_NO_GPU_ADAPTER（GPU 对拍没找到卡，打一行就返回成功）是本次未跑：不落行、不算没红，
    BROAD_MUTATION_GPU_NOT_RUN 逐条报；退 0 而一个测试都没跑到记 driver-error（不算没红）；认不出的退出码记 driver-error。
  基线：这一趟要跑的目标在没改过的树上跑一遍（编译步排在逐条的编译之后、测试步排在逐条的测试步之前，各一次），不绿拒（退 6，基线不绿后面每一条抓到都不作数，
    编不过的条也不落行）；基线里就见到 NOT_RUN_NO_GPU_ADAPTER，整趟本次未跑；这一趟的基线不记库，每趟现跑。
  两趟的结局分开记：CPU 那一趟的行照旧只在 verdicts/，
    GPU 那一趟的行在 gpu-verdicts/；一条的合起来的状态（gpu_combined_state）：CPU 没红、要 GPU 趟的条，GPU 抓到 / 没红 / 无效就是定论，驱动错是没结论，
    库里还没有它的 GPU 结论是 none（理由 gpu-not-run，不算没红）；别的条就是 CPU 那一趟的状态。--summary 的六栏、MISSED / NO_CONCLUSION 明细、提交门禁那一行和 --list-items 的状态都按合起来的状态数。
  沿用：GPU 判定行的键是变异的单元键，另带 gpu_key（这一趟的写法、编进来的特性）；
    抓到的要抓到它的测试此刻带特性的测试键（它的单元闭包键）与行里的相同，没红的要覆盖者键（这一趟够得着它的那些测试各自测试键的多重集）相同才算数（真跑一律不沿用、每趟现判，见「记红、撤绿、下趟现判」），无效的键相同就沿用，
    驱动错不是结论；CPU 那一趟沿用不了重判过的条，GPU 那一趟照自己的键判；复用哨兵 GPU 趟另有一份（项名 gpu:<序号>）。
  分片的两片各在自己那一堆里跑 GPU 趟，判定照样写进服务里同一份 gpu-verdicts/；键相同、gpu_key 相同的行照常沿用，
    GPU 趟只现判没有结论的条（沿用判据的其余几问——测试键、覆盖者键——仍在用到时现核，核不过的那一条照现判）。
  不领卡（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）：卡与清场归 GPU 批处理验证服务，这一份不领卡、不挑卡、不清场；
    环境里有 GPU_CARDS_BY_PRIORITY 就原样交给 GPU 那一趟的每一步，没有不设。测试自己开不了卡时打 NOT_RUN_NO_GPU_ADAPTER（上面「结局」的本次未跑），
    要现判的条逐条打 BROAD_MUTATION_GPU_NOT_RUN（不落行、不算没红），不退回 CPU。自证 selftest_gpu_build_first_cells 核每一步收到的卡串就是环境里给的那一串。
  CPU 那一趟的前提（不归这一节）：mutation-prune.py 的 coverers_key 跳过默认 cfg 下不编的测试（tests/ 里整个文件 #![cfg(feature = "gpu")] 的，entry-cfg-inactive；
    它们不可能是 CPU 那一趟的覆盖者，不算拒、不进条数），CPU 那一趟没红的行的覆盖者键因此算得出，GPU 那一趟才选得到它们；别的拒因仍整条拒（状态 none，理由 coverer-refused）。
    弄坏开关 coverers-cfg-inactive-refused（BROAD_MUTATION_BREAK）：不跳过、照旧整条拒，红在 mutation-prune.py --selftest 的覆盖者键那一格；本驱动的 --selftest 不另设这一格。
中断续接：任何时刻被 TERM / KILL，重跑同一条命令：沿用得了的条一条都不重跑（带 --item 的一样）；被打断的那一条没有新行、只重跑它（或只补它的第二段）；
  被 KILL 的那一趟的运行目录（改过的树、在途记录、编译目录）下一趟拿到锁之后先删掉，这一趟在新建的运行目录里照源码换树（库目录「runs/」）。
  被 TERM / INT 停：在途那一步连同它的进程组一起停（给驱动自己起的那个子进程的组发 TERM，到 --stop-grace 秒还在就 KILL），不留着它跑完。
  被 KILL：留下的那一步还活着时占着库目录的锁（锁的描述符在驱动与这一步起的进程手里）：等 --lock-wait 秒，还占着就退 4、不跑。
基线：每一步按基线键先在没改过的树上跑一遍，全绿才开始跑变异（基线红了一律退 6）。
  取基线按（基线键, 步名）取最后一行：是绿的就沿用，不然重跑那一步、追加一行；行里记着跑它时的线程数（--threads），不进键、不挑行（弄坏开关 baseline-by-threads：只认同一个线程数的行）。
  基线键（Judgement.baseline_key，D13 已定项 17）= 这一步真跑到的每一条判定各自单元闭包键的多重集、这一步的命令（去掉线程数）、全局那几行：
    判定是测试函数（mutation-prune.py 的 test_entries 静态列：第一段那三个包、验证包 singlefs-verification 里不标 #[ignore] 的，
    去掉登记的崩溃枚举用例与放量分项）、这几个目标里的条目级宏调用（宏展开出来的测试静态列不出，把宏调用连同它的闭包算进去）、
    被变异两个包的文档测试；验证包带 --release，闭包照 cfg 去掉 debug_assertions 收。闭包键里有闭包单元的种类与去注释指纹、闭包真引到的
    cfg 原子、全局那几行与 edition；在这一步的 cfg 与特性下不编的判定不算这一步跑到的。不带步名、手写的版本号、包级与文件级指纹：
    改一个不在这一步任何判定闭包里的单元，基线照样沿用。
  拒算（照跑、不沿用、不退回包级）：编译步（编得过与否、用时随它编的那几个包里每一个单元变，按单元闭包切不开）、有一条判定的闭包拒算
    （读运行期拼出的路径、起自己这个可执行文件这一类，lib_function_units.py 认不全）、认不出跑哪些判定的步。拒算的步这一趟用 run_token 算一把
    这一趟独有的键：基线行照常记、这一趟里照常对得上，以后哪一趟都对不上。stdout 那一句写「基线键拒算（为什么）」。
  按条记基线（PER_ENTRY_TEST_STEPS 那两个测试步里，基线键只因为有判定的闭包拒算而算不出的；Judgement.baseline_entry_plan、Runner.ensure_entry_baseline）：
    不整步照跑。nextest 那一步一条测试一份记录（键 = 去掉线程数的命令、全局那几行、它自己的单元闭包键），有条目级宏调用的二进制另一份
    「宏调用展开的那几条」（键 = 那个二进制里宏调用的闭包键多重集）；验证包一个二进制一份记录（键 = 它里面闭包
    算得出的判定与宏调用的闭包键多重集；里面有拒算的，记的是 --exact --skip 掉它们的那一趟）。每份记录照（记录键、kind、线程数）在
    baseline-entries/ 找，找得到的沿用（不再跑）；找不到的与拒算的那几条在没改过的树上照跑（nextest 用 -E 只点它们；
    验证包拒算的那几条另起一趟 --exact 只跑它们），
    照跑的那一趟绿了，它点到的每份记录照键追加一行；宏调用的闭包拒算的二进制整个照跑。名字与二进制只当索引、不进键（rest 另按二进制找：改名只会重跑）。
    baselines/ 照常追加这一趟那一行（这一趟独有的键）。
    stdout 写「按条记基线：N 份记录沿用 M、照跑 K；拒算的 R 条每趟照跑（头几条）」。文档测试那一步照旧整步。
    今天的树上第一段测试步 3 条、验证包 15 条拒算（读 env!("CARGO_MANIFEST_DIR") 拼出的目录或运行期给的文件名、起自己这个可执行文件），
    lib_function_units.py 与 crates/singlefs-unit-graph 逐条同义，那几种形态两边都拒算，不在这里另认。
  每一趟的编译目录是新建的（库目录「runs/」）：头一回编（验环境与编译步的基线）从头编，之后各条在这一趟里只付增量编译；不照用上一趟编的产物。
  每次重起（这一趟没现跑基线的那一段）先在没改过的树上把编译那几步各跑一遍，红了退 6：基线绿过之后环境坏了要在落行之前发现。
  第二段基线红报成退 6 的拒绝，列红的测试与出路（修好之后重跑同一条命令；基线不绿，后面每一条抓到都不作数，所以第二段红着这一道过不去）。
  编译那一步退 101 而日志里没有带源码位置的编译错误（磁盘满、锁文件不符这一类），不记无效，记 driver-error 并当场在没改过的树上验一次编译。
多机配置交给测试（multi_host_configuration_for_tests）：测试在库目录里的专用树里起，那棵树不是 git 工作树，crates 那侧照 git common-dir 找不到多机配置就直接报错；
  所以真跑测试的每一步（步表里种类是 test 的：两段的测试步、覆盖那一步、GPU 那一趟的测试步；编译步不算）的环境里设 SINGLEFS_MULTI_HOST_CONFIG=<多机配置的绝对路径>。
  多机配置在哪：调用方设了 SINGLEFS_MULTI_HOST_CONFIG 照它（取成绝对路径），没设取 --source 那棵树的 git common-dir 上一层下的 multi-host.env。
  那份文件不在（或 --source 不是 git 工作树、又没设那个变量）的，真跑起任何 cargo 之前就拒、退 2，一步都不起（配置错，不当本次未跑）。
  弄坏开关 test-steps-configuration-unpassed 下不判、不设。
内存与限时：每一步命令直接起，不经线程包装、内存包装与清场包装，不设内存上限、不限时，
  起在自己新开的进程组里（组号就是驱动起的那个子进程的进程号）。整条驱动是统一任务服务的一个任务：内存上限由服务按记录给、撞了放大重排，
  卡住由服务判（task-server/task_server_lib.py 文件头结局表的 stalled），驱动不另判、不另重跑。一条变异把整趟撞上限或卡住、被服务停，
  就是驱动整趟被停（照「中断续接」）：在途那一条不落行、没有结论，不算抓到；服务重排之后同一条命令照常判它。一步退了认不出的码一律记 driver-error。
谁起、什么时候起：只归统一任务服务判（起 cargo 的与写判定库的用法都经 task-server/task submit 交），
  驱动自己不看环境变量另判；出路里给的命令一律写成 task-server/task submit -- <命令>（SUBMIT_PREFIX）。

按覆盖挑测试（C2；规则在 scripts/mutation/mutation-prune.py）：每趟在没改过的树上插桩列一遍测试（--coverage-step，驱动自己起；第一段逐条测试、
  验证包逐个二进制，E6），覆盖记录键对得上的那几条照用判定库 coverage/ 里的、对不上的才插桩跑；每条变异先编，再只跑执行到它的容器单元的测试
  （-E 按块切，避开单参数 131072 字节上限），遇第一个失败即停；零覆盖记没红（备注写零覆盖）。
  例外一律跑全部（E1）：容器不是 fn 单元的变异、常量上下文里用到的 const fn、起子进程的测试二进制、文档测试（第二段每条都跑）。
  覆盖要 llvm-profdata、llvm-cov：取 rustup 组件 llvm-tools（rustup component add llvm-tools；与编译器同一份 LLVM），装在别处给 --llvm-bin <目录>；它的版本进全局那几行。

输出：开跑一行 BROAD_MUTATION_PLAN list=<清单条数> judge=<现判> carried=<沿用> refused=<拒算> selected=<选中> threads=…，
      要现判的每条一行 BROAD_MUTATION_JUDGE sequence=… reason=…（沿用不了的理由；作废时另带 changed=<变了的单元>）；
      每条判完一行 BROAD_MUTATION_ITEM sequence=… outcome=… stage=… build_seconds=… test_seconds=… wall_seconds=… judged=1 red=<没红记 1，别的记 0> reused=0 name=…
      （入口脚本那一行的判了、红了、复用了多少：这一行只给现判的条打，一条一判，复用的条不打这一行、数在 BROAD_MUTATION_PLAN 的 carried=；C641）；
      末尾一行 BROAD_MUTATION_TOTAL selected=… ran=… judged=<现判> carried=<沿用> refused=<拒算> 各结局条数，成功那句报「现判 N、沿用 M、拒算 R」，后面接「GPU 趟（要 K 条）：现判 a、复用 b、本次未跑 c」；
      GPU 那一趟：BROAD_MUTATION_GPU_PLAN needed=<要这一趟的条数> judge=<现判> carried=<沿用> refused=<拒算> features=<特性>，现判的每条一行 BROAD_MUTATION_GPU_JUDGE sequence=… reason=… labels=<目标>，
      判完一行 BROAD_MUTATION_GPU_ITEM sequence=… cpu=<CPU 那一趟的结局> gpu=<GPU 那一趟的结局> combined=<合起来的状态> build_seconds=… test_seconds=… wall_seconds=… judged=1 red=<GPU 没红记 1> reused=0 name=…，
      本次未跑的每条一行 BROAD_MUTATION_GPU_NOT_RUN sequence=… reason=<no-adapter 等> name=…，末尾一行 BROAD_MUTATION_GPU_TOTAL needed=… judged=… carried=… not_run=… 各结局条数；
      --summary 打 BROAD_MUTATION_SUMMARY 一行（六栏与 refused=、unlisted_units=、gpu_needed= gpu_reused= gpu_not_run= gpu_without_conclusion=）与 MISSED、DRIVER_ERROR、REFUSED、UNLISTED 明细，
      BROAD_MUTATION_GATE 一行（total、concluded、missed、without_conclusion、refused、unlisted_units、gpu_needed、gpu_reused、gpu_not_run、gpu_without_conclusion、verdict）与 NO_CONCLUSION 明细；
      gpu_needed 是要 GPU 那一趟的条数（CPU 那一趟没红、又有带 gpu 特性的测试够得着），它 = gpu_reused（库里键相同、沿用判据认的 GPU 结论）+ gpu_not_run（还没有 GPU 结论：没卡、没来得及跑）+
      gpu_without_conclusion（GPU 那一趟驱动错）；后两种的条在 without_conclusion 里、NO_CONCLUSION 明细的理由写 gpu-…，不在 missed 里；
      NO_CONCLUSION 明细的理由后面接作废时变了的单元；
      一步跑着时每过 PROGRESS_INTERVAL_SECONDS 秒（300）往 stderr 打一行
      BROAD_MUTATION_PROGRESS sequence=<序号，基线是 baseline，验环境是 environment> step=<步名> elapsed_seconds=… compiled_crates=… tests_passed=… tests_failed=…
退出码：0 选中的条都判完；1 --summary 栏数对不上或判定库有对不上的行（段数、结局认不出）、读不了登记的崩溃枚举用例与放量分项、
  清单按单元存复原不回去、清单里有落不进任何单元的变异、--selftest 有格判错；
  2 用法错（项写错、库目录不合、缺工具、真跑起 cargo 之前 RocksDB 预编库的取值判红（scripts/hosts/rocksdb-prebuilt.py entry：这台没装、
    此刻进程环境的取值缺了或对不上；出路是开新会话，或命令前 eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"）、只供自证的变量出现在自证之外、--selftest 不在临时 prune-server 底下起）；
  3 判定库服务的配置不对、服务拒了帧或回应解不开（scripts/prune-server/prune_server_client.py 文件头「退出码」）；
  75 判定库服务不可用（连不上、停了、忙到上限）：不当「库里没有」现判，服务回来之后重跑同一条命令；
  4 锁被占着；5 在途记录对不上又换不了树；6 基线不绿（含 GPU 那一趟的基线）、或疑似环境坏了；
  9 这一趟别的都成了，运行目录没删掉（有进程开着里面的文件，或删了回读还在；逐个报进程，出路照那一句）；
  8 --summary 判提交门禁红（有没结论的条、有没红的条、或有单元没记清单；BROAD_MUTATION_GATE 行写 verdict=red）；
  143 被 TERM / INT 停（在途那一条不落行）。
自证失败行的转述：失败行（「✗ 自证：」那一行）转述驱动输出的尾部，截在哪由临时路径长短定；转述里的 ✗ 一律换成「〔转述〕」，截进来的半行不另成失败行；
  上一格判完以来这一格起的驱动在完整输出里报的 ✗ 行（如「✗ 复用失灵：…」）整行接在失败行那一行末尾「；这一格起的驱动报的失败行 N 行：…」
  （去掉数字之后相同的几行并成一种、注明同样的另几行，至多 SELFTEST_DRIVER_FAILURE_LINES_SHOWN 种，多的只报种数）。下面点名驱动报的那一句的，对的是失败行那一行。
--selftest 的并行（selftest_cell_chains、selftest_run_chains、selftest_collect_chains）：
  各组格分成几串：一串一个 fork 出来的子进程，用它自己的 Selftest 与临时目录 <临时目录>/chain-<头一组的函数名>/（假的一步记的调用、树副本、库目录、假清单与计划互不踩），
  串里照 selftest_cell_functions 的次序走；不同的串同时跑，至多每台默认重活线程数（scripts/hosts/heavy_threads.py 的 default_heavy_threads，设了 SINGLEFS_THREAD_CAP 照它；
  取不到打一行说明、一串一串跑）个。不按内存上限收：服务给的上限取这条命令最近几趟跑完的峰值，按上限收并行度就一直开不大，撞了上限由服务放大重排。
  要照次序接着跑、不能拆开的（selftest_cells_run_together）：selftest_basic_cells、selftest_state_cells、selftest_test_variable_cells 一串——后两组接着用前一组判出的库「first」。
  等各串都收回之后才在本进程里跑的（selftest_cells_after_chains）：selftest_multi_host_configuration_cells——它判的是各串起过的每一步收没收到多机配置，读各串记下的记录。
  每串的标准输出、标准错误各落一份文件，收齐之后按串的登记次序回放；格数与判错数按串的登记次序加；每串收回时往 stderr 打一行用时、CPU 秒数、峰值内存（ru_maxrss，含它等过的孙进程）。
  派出去几串就要收回几串：没留下结果的串逐个点名、记一格判错，整份判红（「✗ 自证：派出去的串与收回的结果对不上」）；串里抛了异常的同样记一格判错、带上异常的尾巴。
break-summary-rows：「✗ --selftest：… 格里 … 格判错」（--selftest 有格判错时 run_selftest 在各格失败行之外多打的汇总句；开关复跑认它是汇总行、不要求点名）
弄坏开关 BROAD_MUTATION_BREAK=<项>（逗号分隔，只给 --selftest 证红用，只在 --selftest 自己起的子进程里生效；selftest-chain-result-dropped 例外，在 --selftest 自己那一层认）。
逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest_cell_functions 里那几组格的函数名，括号里是那条「✗ 自证：」的开头；
每一项红时都另有汇总行「✗ --selftest：<格数> 格里 N 格判错」，退 1）：
  BROAD_MUTATION_BREAK=selftest-chain-result-dropped  --selftest 自己那一层认：各串收回之后，照登记次序排在最后的那一串的结果丢掉（像那一串的子进程没留下结果）
      红：「✗ 自证：派出去的串与收回的结果对不上」
  BROAD_MUTATION_BREAK=list-items-pipe-unhandled  --list-items 不照 SIGPIPE 惯例退（接 | head 时打 BrokenPipeError、退 120）
      红：「✗ 自证：--list-items 接 | head 时没照 SIGPIPE 惯例安静退」
  BROAD_MUTATION_BREAK=own-libtest-failure-line  libtest 结果行回到驱动自己那条正则取红的测试名（名字带上「 - should panic」），不经 lib_libtest_result_lines.py
      红：「✗ 自证：两段式的结局不对」、「✗ 自证：只补第二段不对」
  BROAD_MUTATION_BREAK=rerun-done  沿用得了的条照样重跑（不读判定库）
      红：「✗ 自证：单独一条没红（CPU 一趟）下一趟没现判、或抓到的、无效的没沿用」、「✗ 自证：先抓到、同一把键又判没红（CPU 一趟）下一趟没现判、或现判之后没红仍沿用」、「✗ 自证：现判又出没红的条（CPU 一趟）下一趟没接着现判」、「✗ 自证：同一条变异内容没变、判定库里有可沿用的行、所在文件在某提交之后改过别处，没照样沿」、「✗ 自证：抓到它的那条测试改了之后那几条没现判」、「✗ 自证：多登记一条之后别的条没照样沿用」、「✗ 自证：只改 Cargo.toml 注释之后别的条没照样沿用」、「✗ 自证：改了 RUSTFLAGS 之后判过的条没有照样沿用」、
      「✗ 自证：拒算的条没有每趟现判、或别的条没沿用」、「✗ 自证：只补第二段不对」、「✗ 自证：重起之后沿用得了的条或基线被重跑了」、
      「✗ 自证：点名沿用得了的条没跳过」、「✗ 自证：点名现判（自证变量 BROAD_MUTATION_REJUDGE_NAMED）没有重跑并追加新行」、「✗ 自证：抓到的条被后来的驱动错行盖掉了」、「✗ 自证：认不出的退出码没处理对」、
      「✗ 自证：工具链版本进了键」、「✗ 自证：两台各点名一半的结果没合对」
      另红：「✗ 自证：reuse-three-questions：」（① 第二趟全现判）、「✗ 自证：reuse-sentinel：」（这一格要的就是那一趟复用哨兵判红；跟着判错的是照常那一趟）
      另红：「✗ 复用失灵：…键相同却现判了」、「✗ 自证：有卡之后重跑同一条命令没把 GPU 那一趟补上」
  REUSE_CHECKS_BREAK=rejudged-sentinel-passes（共用库 scripts/gate/lib_reuse_checks.py 的开关，哨兵被现判不红）
      红：「✗ 自证：reuse-sentinel：」
  BROAD_MUTATION_BREAK=interrupted-as-done  被打断的那一条重起时照「已完成」落一行、不重跑
      红：「✗ 自证：被打断的第 3 条没有重跑、或被当成已完成」、「✗ 自证：两段式的结局不对」、「✗ 自证：汇总不对」
      另红：「✗ 提交门禁判红：…条沿用不了、也没有现判的结论」
  BROAD_MUTATION_BREAK=no-row-flush  判定行攒在内存里到收尾才写
      红：「✗ 自证：杀驱动之前，前两条的判定行没有都落盘」、「✗ 自证：重起之后沿用得了的条或基线被重跑了」
  BROAD_MUTATION_BREAK=stage-two-missed-as-caught  第二段没红记成抓到
      红：「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑不对」、「✗ 自证：单独一条没红（CPU 一趟）下一趟没现判、或抓到的、无效的没沿用」、「✗ 自证：同一条变异内容没变、判定库里有可沿用的行、所在文件在某提交之后改过别处，没照样沿用」、
      「✗ 自证：先抓到、同一把键又判没红（CPU 一趟）下一趟没现判、或现判之后没红仍沿用」、「✗ 自证：现判又出没红的条（CPU 一趟）下一趟没接着现判」、「✗ 自证：先抓到的第 6 条点名现判（自证变量）判成没红之后，汇总没按没红数、或这一趟现判的不是只有它」、「✗ 自证：reuse-three-questions」、「✗ 自证：reuse-sentinel」、
      「✗ 自证：只有 CPU 红：CPU 那一趟抓到的条也起了 GPU 那一趟、或两趟的结局没分开记」、「✗ 自证：GPU 那一趟的结局没有单独存进 GPU 判定表」、
      「✗ 自证：只有 GPU 红：GPU 那一趟红了没算抓到、抓到它的测试没认出来、或 --list-items 的状态不是两趟合起来的」、「✗ 自证：两趟都没红：没记成没红、或 GPU 那一趟跑了闭包够不着的条」、「✗ 自证：GPU 那一趟的命令不对」、
      「✗ 自证：没卡：没报本次未跑、记成了没红、或落了 GPU 行」、「✗ 自证：有卡之后重跑同一条命令没把 GPU 那一趟补上」、「✗ 自证：中途没卡：这一条之后的没停下、或落了行」、
      「✗ 自证：GPU 趟基线红没拒成「基线不绿」」、「✗ 自证：GPU 趟认不出的退出码 / 一个测试都没跑到不是没结论」、「✗ 自证：GPU 趟编不过没记成无效」、「✗ 自证：点名现判（自证变量）没有重跑 GPU 那一趟并追加新行」、
      「✗ 自证：BROAD_MUTATION_ITEM 每条一行没带判了、红了、复用了多少，或 red= 与结局对不上」、「✗ 自证：两段式的结局不对」、「✗ 自证：汇总不对」、
      「✗ 自证：GPU 趟没红的条下一趟没现判」、「✗ 自证：GPU 趟抓到的条（CPU 一趟没红）下一趟没沿用」、「✗ 自证：GPU 趟先抓到、同一把键又判没红的条下一趟没现判」、「✗ 自证：GPU 趟先编后跑：」
      另红：「✗ 提交门禁判红：…条沿用不了、也没有现判的结论」
  BROAD_MUTATION_BREAK=summary-mismatch-silent  汇总对不上（认不出的结局、段数不对的行、清单表复原不回去）不报
      红：「✗ 自证：判定库里有认不出的结局时汇总没判红」、「✗ 自证：清单按单元存的表被改坏了没判红」
  BROAD_MUTATION_BREAK=interrupted-run-directory-kept  开跑时不删上一趟被 KILL 留下的运行目录、不认它里面的在途记录
      红：「✗ 自证：被打断的第 3 条没有重跑、或被当成已完成」、「✗ 自证：重起跑完之后库目录里还剩上一趟被 KILL 留下的、或这一趟的运行目录」、「✗ 自证：上一趟被 KILL 留下的运行目录」、
      「✗ 自证：恢复原件半途被杀之后树里多出了文件、临时件不在运行目录里、或重跑没照常判完」
  BROAD_MUTATION_BREAK=accept-unknown-item  不认得的项悄悄略过
      红：「✗ 自证：不认得的项没拒」
  BROAD_MUTATION_BREAK=exclude-re-dropped  换树时不把 .cargo/mutants.toml 拷进专用树（cargo-mutants 读不到 exclude_re，等于忽略它）
      红：「✗ 自证：exclude_re 没排掉那一条、清单没整版重判、或别的条没沿用」
  BROAD_MUTATION_BREAK=item-counts-dropped  每条一行 BROAD_MUTATION_ITEM 不带 judged=、red=、reused=
      红：「✗ 自证：BROAD_MUTATION_ITEM 每条一行没带判了、红了、复用了多少」
  BROAD_MUTATION_BREAK=test-variables-anywhere  只供自证的变量隔一层进程、库目录在自证工作目录之外也认
      红：「✗ 自证：只供自证的变量指着自证工作目录之外的库时没拒」、「✗ 自证：父进程不是 --selftest 的进程带着假一步的变量时没拒」、「✗ 自证：父进程不是 --selftest 的进程只带着弄坏开关时没拒」
      另红：「✗ 自证：平常跑设了弄坏开关、带认不得的选项时没在参数解析之前拒并点名」
  BROAD_MUTATION_BREAK=break-guard-after-parsing  只供自证的变量（含这个弄坏开关）平常跑设了的拒排回参数解析之后：认不得的选项先被 argparse 当用法错拒，不点名变量
      红：「✗ 自证：平常跑设了弄坏开关、带认不得的选项时没在参数解析之前拒并点名」、「✗ 自证：父进程不是 --selftest 的进程带着假一步的变量时没拒」、「✗ 自证：父进程不是 --selftest 的进程只带着弄坏开关时没拒」
  BROAD_MUTATION_BREAK=missed-carried  没红（CPU 一趟、GPU 一趟）按覆盖者键相同照旧沿用、不现判
      红：「✗ 自证：单独一条没红（CPU 一趟）下一趟没现判、或抓到的、无效的没沿用」、「✗ 自证：同一条变异内容没变、判定库里有可沿用的行、所在文件在某提交之后改过别处，没照样沿用」、
      「✗ 自证：抓到它的那条测试改了之后那几条没现判」、「✗ 自证：先抓到、同一把键又判没红（CPU 一趟）下一趟没现判、或现判之后没红仍沿用」、「✗ 自证：现判又出没红的条（CPU 一趟）下一趟没接着现判」、
      「✗ 自证：GPU 趟没红的条下一趟没现判」、「✗ 自证：GPU 趟先抓到、同一把键又判没红的条下一趟没现判」
  BROAD_MUTATION_BREAK=state-is-last-row  一条的状态照旧取最后一行
      红：「✗ 自证：第二段的定论被后来的 stage-one-only 行降级了」、「✗ 自证：抓到的条被后来的驱动错行盖掉了」、「✗ 自证：汇总不对」
  BROAD_MUTATION_BREAK=named-items-always-rerun  点名的条沿用得了也重跑
      红：「✗ 自证：点名沿用得了的条没跳过」、「✗ 自证：点名现判（自证变量 BROAD_MUTATION_REJUDGE_NAMED）没有重跑并追加新行」、「✗ 自证：抓到的条被后来的驱动错行盖掉了」、「✗ 自证：只补第二段不对」
      另红：「✗ 自证：有卡之后重跑同一条命令没把 GPU 那一趟补上」、「✗ 自证：GPU 趟抓到的条（CPU 一趟没红）下一趟没沿用」
  BROAD_MUTATION_BREAK=no-environment-check  重起不验编译、连着无效或秒红也不查
      红：「✗ 自证：认不出的退出码没处理对」、「✗ 自证：基线绿过之后编译坏了，重起没发现」、「✗ 自证：连续无效而原件上编译照过时没有接着跑」、「✗ 自证：连续 3 条秒红的抓到、原件上那一步也红，没有停下报疑似环境坏了」
  BROAD_MUTATION_BREAK=unviable-by-exit-code  编译步退 101 一律记无效
      红：「✗ 自证：认不出的退出码没处理对」
  BROAD_MUTATION_BREAK=unknown-exit-stops  一步退认不出的码照旧整趟停
      红：「✗ 自证：认不出的退出码没处理对」、「✗ 自证：重起之后沿用得了的条或基线被重跑了」、「✗ 自证：BROAD_MUTATION_ITEM 每条一行没带判了、红了、复用了多少，或 red= 与结局对不上」、
      「✗ 自证：被打断的第 3 条没有重跑、或被当成已完成」、「✗ 自证：两段式的结局不对」、「✗ 自证：汇总不对」、「✗ 自证：第二段的步不对」、「✗ 自证：判定库里有认不出的结局时汇总没判红」、
      「✗ 自证：第二段的定论被后来的 stage-one-only 行降级了」、「✗ 自证：抓到的条被后来的驱动错行盖掉了」、
      「✗ 自证：恢复原件半途被杀之后树里多出了文件、临时件不在运行目录里、或重跑没照常判完」、「✗ 自证：两台各点名一半的结果没合对」
      另红：「✗ 第 3 条的 one-test 退 104（认不出的退出码）：不落行」；SELFTEST_PLAN 里第 3、6、8 条退 104，用默认计划跑到这几条的格跟着红（上面逐条列的就是这些格）
  BROAD_MUTATION_BREAK=stage-two-full-cargo-test  第二段照旧是整遍 cargo test（第一段那三个包与验证包）
      红：「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑」、「✗ 自证：两段式的结局不对」、「✗ 自证：汇总不对」、「✗ 自证：第二段的步不对」
      另红：「✗ 提交门禁判红：…条沿用不了、也没有现判的结论」、「✗ 自证：重起之后沿用得了的条或基线被重跑了」
  BROAD_MUTATION_BREAK=term-leaves-step-running  被 TERM 时只给驱动起的那一个进程发一次 TERM，不停它的进程组
      红：「✗ 自证：跑完运行目录里不剩它建的编译目录与树副本（被 TERM 停）」（分出来的那一个还握着这一趟树里的工作目录，运行目录删不掉）、「✗ 自证：被 TERM 停时在途那一步没有连同它的进程组一起停」（selftest_stop_cells：假的一步分出来的那一个照挂着）
  BROAD_MUTATION_BREAK=registered-not-skipped  第二段 验证包与 GPU 那一趟的测试步命令不接登记的崩溃枚举用例与放量分项的 --skip
      红：「✗ 自证：登记的崩溃枚举用例与放量分项没经 --skip 排掉」
  BROAD_MUTATION_BREAK=registered-exclusion-dropped  登记的崩溃枚举用例与放量分项不进排除单（验证包里对上的静态测试照算覆盖者与基线）
      红：「✗ 自证：登记的崩溃枚举用例没进排除单」
  BROAD_MUTATION_BREAK=attribute-skipped-run  标了 #[ignore] 的用例也算驱动真跑得到的测试
      红：「✗ 自证：标了 #[ignore] 的用例进了驱动真跑得到的测试」
  BROAD_MUTATION_BREAK=progress-never  一步跑着时进度间隔的判断永不触发、一行 BROAD_MUTATION_PROGRESS 都不打
      红：「✗ 自证：一步跑过进度间隔没有报进度」
  BROAD_MUTATION_BREAK=coverage-selection-dropped  剪枝：不按覆盖挑测试
      红：「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑」
  BROAD_MUTATION_BREAK=stage-two-not-by-binary  剪枝：第二段不按二进制挑
      红：「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑」
  BROAD_MUTATION_BREAK=doc-tests-skipped  剪枝：文档测试不跑
      红：「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑」、「✗ 自证：两段式的结局不对」、「✗ 自证：汇总不对」、「✗ 自证：第二段的步不对」
      另红：「✗ 提交门禁判红：…条沿用不了、也没有现判的结论」
  BROAD_MUTATION_BREAK=carry-without-store-check  判定库里键相同就沿用（抓到它的测试键、覆盖者键都不比）
      红：「✗ 自证：抓到它的那条测试改了之后那几条没现判」、「✗ 自证：第二段的定论被后来的 stage-one-only 行降级了」
      另红：「✗ 自证：reuse-three-questions：」
  BROAD_MUTATION_BREAK=summary-gate-silent  --summary 有没红或没结论也退 0
      红：「✗ 自证：先抓到的第 6 条点名现判（自证变量）判成没红之后，汇总没按没红数、或这一趟现判的不是只有它」、「✗ 自证：汇总不对」、「✗ 自证：认不出的退出码没处理对」、「✗ 自证：两台各点名一半的结果没合对」
      另红：「✗ 自证：两趟都没红：」、「✗ 自证：只有 CPU 红：」、「✗ 自证：没卡：」
  BROAD_MUTATION_BREAK=stage-two-baseline-red-generic  第二段基线红只报一句不给出路
      红：「✗ 自证：第二段基线红没报成带出路的拒绝」
      另红：「✗ 基线 two-test-verification 在没改过的树上退 101：基线不绿」
  BROAD_MUTATION_BREAK=test-steps-configuration-unpassed  真跑测试的步不往环境里设 SINGLEFS_MULTI_HOST_CONFIG、起 cargo 之前也不判多机配置在不在
      红：「✗ 自证：真跑测试的步没收到多机配置」、「✗ 自证：多机配置不在时没在起 cargo 之前拒」
  BROAD_MUTATION_BREAK=rocksdb-entry-unchecked  真跑起 cargo 之前不调 scripts/hosts/rocksdb-prebuilt.py entry
      红：「✗ 自证：被判的树里 验证包依赖 rocksdb 而进程环境里没有预编库的取值，没在起 cargo 之前拒」
      （entry 的跳过办法：被判的树里没有依赖 rocksdb 的清单（自证的 SELFTEST_SOURCES 就是这样），或 PATH 上的 cargo 是带
      『# rocksdb-prebuilt-entry: selftest-fake-cargo』那一行的假 cargo；见 scripts/hosts/rocksdb-prebuilt.py 文件头『entry』）
  BROAD_MUTATION_BREAK=no-carry-without-arguments  不带选条参数时不读判定库（全部现判）
      红：「✗ 自证：单独一条没红（CPU 一趟）下一趟没现判、或抓到的、无效的没沿用」、「✗ 自证：先抓到、同一把键又判没红（CPU 一趟）下一趟没现判、或现判之后没红仍沿用」、「✗ 自证：现判又出没红的条（CPU 一趟）下一趟没接着现判」、「✗ 自证：同一条变异内容没变、判定库里有可沿用的行、所在文件在某提交之后改过别处，没照样沿」、「✗ 自证：抓到它的那条测试改了之后那几条没现判」、「✗ 自证：多登记一条之后别的条没照样沿用」、「✗ 自证：只改 Cargo.toml 注释之后别的条没照样沿用」、「✗ 自证：改了 RUSTFLAGS 之后判过的条没有照样沿用」、
      「✗ 自证：拒算的条没有每趟现判、或别的条没沿用」、「✗ 自证：重起之后沿用得了的条或基线被重跑了」、
      「✗ 自证：认不出的退出码没处理对」、「✗ 自证：工具链版本进了键」、「✗ 自证：两台各点名一半的结果没合对」
      另红：「✗ 复用失灵：…键相同却现判了」、「✗ 自证：reuse-sentinel：」、「✗ 自证：reuse-three-questions：」
  BROAD_MUTATION_BREAK=select-by-commit  恢复按提交选条：被判那棵树里相对 HEAD 改过的文件里的条一律现判
      红：「✗ 自证：同一条变异内容没变、判定库里有可沿用的行、所在文件在某提交之后改过别处，没照样沿」、「✗ 自证：抓到它的那条测试改了之后那几条没现判」、
      「✗ 自证：先抓到、同一把键又判没红（CPU 一趟）下一趟没现判、或现判之后没红仍沿用」、「✗ 自证：现判又出没红的条（CPU 一趟）下一趟没接着现判」
  BROAD_MUTATION_BREAK=refused-carried  拒算的条与抓到它的测试认不出的条照沿用
      红：「✗ 自证：拒算的条没有每趟现判、或别的条没沿用」、「✗ 自证：第二段的定论被后来的 stage-one-only 行降级了」
  BROAD_MUTATION_BREAK=read-only-volatile-refused  只读的用法（--summary 这一类）也拒落在开机会清空的目录里的库：样本拷进 /tmp 再判时判不了
      红：「✗ 自证：库目录落在 /dev/shm 里时只读的 --summary 也拒了」
  BROAD_MUTATION_BREAK=list-cache-unchecked  清单按单元记进判定库 lists/ 之后不复原核对
      红：「✗ 自证：清单里有被变异包之外的一条时 --refresh-list 没拒」
  BROAD_MUTATION_BREAK=run-directory-kept  跑完（正常、拒绝、被 TERM / INT 停）不删这一趟的运行目录：编译目录与树跨趟留着
      红：「✗ 自证：跑完运行目录里不剩它建的编译目录与树副本（正常结束）」、「✗ 自证：跑完运行目录里不剩它建的编译目录与树副本（出错退出）」、「✗ 自证：跑完运行目录里不剩它建的编译目录与树副本（被 TERM 停）」、
      「✗ 自证：这一趟的运行目录里还有进程开着文件时应当不删、报出那个进程、退 9」、「✗ 自证：重起跑完之后库目录里还剩上一趟被 KILL 留下的、或这一趟的运行目录」、「✗ 自证：上一趟被 KILL 留下的运行目录」、
      「✗ 自证：恢复原件半途被杀之后树里多出了文件、临时件不在运行目录里、或重跑没照常判完」
  BROAD_MUTATION_BREAK=run-directory-open-files-unchecked  删这一趟的运行目录之前不核有没有进程开着里面的文件
      红：「✗ 自证：这一趟的运行目录里还有进程开着文件时应当不删、报出那个进程、退 9」
  BROAD_MUTATION_BREAK=build-directory-in-home  把专用树、编译目录、每一步的 TMPDIR 与在途记录放在库目录里、跨趟留着（不建 runs/ 下的运行目录）
      红：「✗ 自证：第二趟的编译目录不是新建的」、「✗ 自证：跑完运行目录里不剩它建的编译目录与树副本（正常结束）」、「✗ 自证：跑完运行目录里不剩它建的编译目录与树副本（出错退出）」、
      「✗ 自证：跑完运行目录里不剩它建的编译目录与树副本（被 TERM 停）」、「✗ 自证：这一趟的运行目录里还有进程开着文件时应当不删、报出那个进程、退 9」、「✗ 自证：驱动被杀之后它那一趟的运行目录里没有留着改过的树与在途记录」、
      「✗ 自证：被打断的第 3 条没有重跑、或被当成已完成」、「✗ 自证：重起跑完之后库目录里还剩上一趟被 KILL 留下的、或这一趟的运行目录」、「✗ 自证：上一趟被 KILL 留下的运行目录」、
      「✗ 自证：恢复原件半途被杀之后树里多出了文件、临时件不在运行目录里、或重跑没照常判完」、「✗ 自证：服务给的编译目录：」
  BROAD_MUTATION_BREAK=baseline-by-threads  基线只认同一个线程数的行（线程数回到基线的取法里）
      红：「✗ 自证：换了 --threads 之后基线没有沿用」
  BROAD_MUTATION_BREAK=baseline-by-packages  基线键取这一步编、跑的那几个包（连同路径依赖）里全部单元的指纹（包级），带手写的 form 与步名
      红：「✗ 自证：基线键没按这一步跑到的判定的单元闭包取」、「✗ 自证：基线没按条记：一步里收得全的测试没各自沿用、或拒算的那几条没照跑」
      另红：「✗ 自证：第二趟的编译目录不是新建的」
  BROAD_MUTATION_BREAK=baseline-refused-step-whole  不按条记基线：基线键拒算的测试步整步照跑（一条拒算整步照跑）
      红：「✗ 自证：基线没按条记：一步里收得全的测试没各自沿用、或拒算的那几条没照跑」
  BROAD_MUTATION_BREAK=gpu-red-not-caught  GPU 那一趟红了不算抓到（合起来仍记没红）
      红：「✗ 自证：只有 GPU 红：」
  BROAD_MUTATION_BREAK=gpu-all-items  不看闭包：每一条 CPU 那一趟没红的都跑 GPU 那一趟、目标取全部（闭包不引 gpu 的、没有测试够得着的也跑）
      红：「✗ 自证：只有 CPU 红：」、「✗ 自证：两趟都没红：」、「✗ 自证：GPU 那一趟的命令不对」、「✗ 自证：GPU 趟没红的条下一趟没现判」、「✗ 自证：GPU 趟先抓到、同一把键又判没红的条下一趟没现判」
  BROAD_MUTATION_BREAK=gpu-not-run-as-missed  没找到卡（NOT_RUN_NO_GPU_ADAPTER）记成没红
      红：「✗ 自证：没卡：」、「✗ 自证：中途没卡：」
  BROAD_MUTATION_BREAK=gpu-carry-dropped  GPU 那一趟沿用得了的行照样重跑
      红：「✗ 自证：GPU 趟抓到的条（CPU 一趟没红）下一趟没沿用」、「✗ 自证：点名现判（自证变量）没有重跑 GPU 那一趟并追加新行」、复用哨兵那一格（reuse-sentinel：GPU 趟沿用得了的被现判，复用失灵）
      另红：「✗ 自证：reuse-sentinel：」、「✗ 自证：reuse-three-questions：」
  BROAD_MUTATION_BREAK=gpu-test-threads-dropped  GPU 那一趟的测试步不加 --test-threads 1
      红：「✗ 自证：GPU 那一趟的命令不对」
  BROAD_MUTATION_BREAK=gpu-also-when-cpu-caught  CPU 那一趟抓到的条也跑 GPU 那一趟
      红：「✗ 自证：只有 CPU 红：」
      另红：「✗ 自证：reuse-three-questions：」、「✗ 自证：reuse-sentinel：」
  BROAD_MUTATION_BREAK=unreachable-as-empty-library  判定库服务连不上时读判定库交空表、照「库里没有」往下判（改错的样子：该退 75 的退成门禁判红）
      红：「✗ 自证：判定库服务连不上时 --summary 没退 75」
  BROAD_MUTATION_BREAK=unknown-option-accepted  认不出的选项悄悄略过（参数解析照 parse_known_args，不退 2）
      红：「✗ 自证：认不出的选项没拒」
  BROAD_MUTATION_BREAK=gpu-builds-interleaved  GPU 那一趟不先编齐：基线先编先跑，每一条在跑它的测试步之前才编（run_gpu_phase 里逐条编译那一段）
      红：「✗ 自证：GPU 趟先编后跑：」
  BROAD_MUTATION_BREAK=service-build-directory-ignored  环境里给了 CARGO_TARGET_DIR 也不用：同没给时一样在 runs/ 下新建运行目录、跑完删（Library.begin_run）
      红：「✗ 自证：服务给的编译目录：」
  算键与读库的提速（selftest_key_speed_cells，在自证进程里直接调函数，开关在那一组里临时打开认；红的都是那一组的格，汇总行同上）：
  BROAD_MUTATION_BREAK=key-cache-digest-unchecked  读算键缓存时不核摘要
      红：「✗ 自证：摘要对不上的算键缓存也读了」
  BROAD_MUTATION_BREAK=key-cache-sample-unchecked  读了算键缓存不抽查
      红：「✗ 自证：被改了一行键的算键缓存读了」
  BROAD_MUTATION_BREAK=shard-seconds-without-locator  计划分堆时键对不上就当没记过用时，不看同一处变异的行
      红：「✗ 自证：计划分堆没按同一处变异最后一行有定论的用时估」
  BROAD_MUTATION_BREAK=changed-units-batch-oldest-row  批量读上一回的组成清单时取同一处变异最早的一行
      红：「✗ 自证：批量读上一回的组成清单得的「变了的单元」与 Judgement.changed_units 逐条的不同」
  BROAD_MUTATION_BREAK=composition-write-unmemorised  库的写组成清单不记住写过的键
      红：「✗ 自证：库的写组成清单没记住写过的键」
  mutation-prune.py 里算键提速的四项（closure-key-cache-ignores-globals、closure-composition-empty、spawning-cache-ignores-packages、prefill-results-unordered）
  只在 mutation-prune.py --selftest 里认，这份 --selftest 不转发。
mutation-prune.py 文件头列的那几项（驱动在自证里把它们转给规则库）打开时这份 --selftest 不一定红，由 mutation-prune.py --selftest 判。
只供测试（--selftest 自己设；这个进程的父进程不是带 --selftest 的这份脚本、或库目录不在那一趟自证的工作目录里，设了任何一个都拒，退 2）：
  BROAD_MUTATION_STEP_DRIVER=1 每一步的真命令换成自证工作目录（BROAD_MUTATION_SELFTEST_WORK）里的 fake-step-driver
    （调法 fake-step-driver <步名> <序号> <树>，不要前缀；文件名写死，起它的命令是 lib_function_units.py 认得的根外程序形态）、
  BROAD_MUTATION_LIST_DRIVER=<脚本> 换掉 cargo mutants --list（调法 <脚本> <包>，打 JSON）、BROAD_MUTATION_EXPORT_EVERYTHING=1 导出范围换成 --source 整棵树、
  BROAD_MUTATION_TOOLCHAIN=<文字> 换掉 rustc -Vv 的输出、BROAD_MUTATION_KILL_AT=<点> 走到那一点给自己发 KILL、
  BROAD_MUTATION_ALLOW_VOLATILE_HOME=1 许库目录落在 /tmp 下、BROAD_MUTATION_REGISTERED=<目标:用例函数,…> 换掉登记的崩溃枚举用例与放量分项的读法（自证的假树里没有登记表）、
  BROAD_MUTATION_PROGRESS_INTERVAL=<秒> 换掉进度间隔、BROAD_MUTATION_TOOL_VERSIONS=<文字> 换掉工具版本、
  BROAD_MUTATION_STAGE_ONE_ONLY=1 每条只跑第一段、GPU 那一趟不起（造「第二段被打断」那种只过了第一段的现场；命令行没有这样的参数，真跑一律两段跑完再跑 GPU 那一趟）、
  BROAD_MUTATION_REJUDGE_NAMED=1 点名的条沿用得了也现判（GPU 那一趟也不读库）、追加新行、复用哨兵不抽（造「同一把键再判出另一个结局」的现场；命令行没有这样的参数，真跑的强制现判只走复用哨兵）。

gate-similar: crates-mutation-replay.sh 它按 crates/mutations.tsv 逐条改坏、点名的测试必须红，行是人写的、判的是「这条测试抓不抓得到」；这里的清单由 cargo-mutants 生成、判每条变异有没有任何测试抓到，导出范围取它登记的那一份
gate-similar: mutate.sh 它跑一张手写变异表对一个实验二进制，每条 cargo test 直接起、不设内存上限、不限时（整条 mutate.sh 是统一任务服务的一个任务）；这里每一步同样直接起，清单是生成的、判定按单元键落在库目录里沿用与续接
gate-similar: crash-amplification.sh 它是崩溃放量的分项驱动，--list-items / --item 与「重跑同一条命令就是续跑」的形态照它；判的对象不同（崩溃点状态对变异）
gate-similar: crash-point-replay.sh 它的 feature-gated-tests 格对崩溃枚举用例的状态块跑同一条带特性的 验证包命令、判的是「状态块有没有红」；这里同一条命令判的是「CPU 那一趟没红的变异有没有被带特性的测试抓到」，特性与文件的现读写法照它的
gate-similar: mutation-shard-run.sh 它把 crates/mutations.tsv 的行按用时分两片、两片判的都写进 prune-server 里同一份库，合并只按此刻的行键核两片判的行在服务里读得到；这里同样分两片（上面「分片」），分的是 cargo-mutants 清单的条，判的写进服务里族 broad-mutation 那一份（判定按写入次序只追加），表与判据都不同，不共用
"""
import argparse  # noqa: F401
import datetime  # noqa: F401
import difflib  # noqa: F401
import fcntl  # noqa: F401
import hashlib  # noqa: F401
import importlib.util  # noqa: F401
import json  # noqa: F401
import os
import re  # noqa: F401
import secrets  # noqa: F401
import shlex  # noqa: F401
import shutil  # noqa: F401
import signal  # noqa: F401
import subprocess  # noqa: F401
import sys
import tempfile  # noqa: F401
import time  # noqa: F401
import zoneinfo  # noqa: F401
from typing import NamedTuple  # noqa: F401

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))   # 拆出的各份与本文件同目录；按文件路径载入本文件的别处也靠这一行 import 得到它们
from broad_mutation_base import (SCRIPT_DIRECTORY, REPOSITORY, preflight, HeavyThreadDefaultError, default_heavy_threads, run_build_directory,
    SELF_PATH, _LIBTEST_RESULT_LINES_SPECIFICATION, libtest_result_lines, SCRIPT_KEY, MODULE_SOURCE_FILES, MODULE_SOURCE_PATHS, REAL_SELF_PATH,
    DEFAULT_HOME_ENTRY, DEFAULT_LOCK_WAIT_SECONDS, DEFAULT_STOP_GRACE_SECONDS, DEFAULT_SUSPECT_STREAK, DEFAULT_SUSPECT_TEST_SECONDS,
    PROGRESS_INTERVAL_SECONDS, LOCK_POLL_SECONDS, MUTATED_PACKAGES, STAGE_ONE_PACKAGES, VERIFICATION_PACKAGE, REGISTERED_REASON, ADMISSION,
    SUBMIT_PREFIX, VOLATILE_PREFIXES, VOLATILE_FILESYSTEMS, IGNORED_DIRECTORY_NAMES, VERDICT_COLUMNS, PRUNE_SERVER_CLIENT_PATH,
    PRUNE_SERVER_CLIENT_MODULE, LIBRARY_KIND, BASELINES_TABLE, BASELINE_ENTRIES_TABLE, COMPOSITIONS_TABLE, COVERAGE_TABLE, GPU_VERDICTS_TABLE,
    LISTS_TABLE, VERDICTS_TABLE, ORDER_STATE, BASELINE_COLUMNS, BASELINE_ENTRY_COLUMNS, COVERAGE_COLUMNS, PER_ENTRY_TEST_STEPS, EXIT_GATE_RED,
    DECISIVE_OUTCOMES, STAGE_ONE_ONLY, DRIVER_ERROR, NOT_RUN, ROW_OUTCOMES, SUMMARY_COLUMNS, SUMMARY_KEYS, OUTCOME_TEXT, NEXTEST_TEST_FAILED_EXIT,
    CARGO_FAILED_EXIT, EXIT_BASELINE_RED, EXIT_DIRTY_TREE, EXIT_INCONSISTENT, EXIT_LOCKED, EXIT_STOPPED, EXIT_USAGE, EXIT_RUN_DIRECTORY_KEPT,
    NEXTEST_FAILURE_LINE, LIBTEST_RUNNING_LINE, LIBTEST_DOCTESTS_LINE, COMPILING_LINE, NEXTEST_PASS_LINE, COMPILE_ERROR_LINE, COMPILE_ERROR_LOCATION,
    COULD_NOT_COMPILE_LINE, HUNK_HEADER, UNRECOGNISED_TEST, MUTATION_MARKER, BASELINE_SEQUENCE, ENVIRONMENT_SEQUENCE, SELFTEST_WORK_VARIABLE,
    SELFTEST_CELLS_VARIABLE, SELFTEST_STEP_DRIVER_NAME, TEST_ONLY_VARIABLES, SELFTEST_ANCESTOR_DEPTH, NAME_LOCATION_PREFIX, PRUNE_BREAKS,
    GPU_VERDICT_COLUMNS, GPU_ROW_OUTCOMES, GPU_FEATURE, GPU_TESTS_DIRECTORY, GPU_BUILD_STEP, GPU_TEST_STEP, GPU_BASELINE_SEQUENCE,
    GPU_NO_ADAPTER_MARK, GPU_NOT_RUN, BINDGEN_LIBRARY, GPU_ATTRIBUTE_START, GPU_FEATURE_NAME, GPU_PASSED_COUNT, ROCKSDB_PREBUILT_PATH,
    REGISTERED_CHILD)  # noqa: F401
from broad_mutation_judging import (Refusal, refuse, say, note, process_command_line, selftest_ancestor_distance, raw_break, test_mode,
    test_variable, stage_two_skipped, named_items_rejudged, break_is, kill_point, now_text, file_digest, fsync_directory, write_atomically,
    toolchain_text, ItemState, state_of_outcomes, today_text, prune_rules, global_lines_of, verdict_state, Judgement, composition_difference,
    offset_of, replace_span, registered_exclusions, libtest_failed_name, nextest_failed_name, log_shows_compile_error, tool_versions, llvm_bin,
    libtest_binary, first_failing_test, apply_mutation, restore_original, ItemRow, process_item, GpuOutcome, gated_test_files, gpu_pass_command,
    gpu_log_text, gpu_failing_test, gpu_classify_build, gpu_classify_test, gpu_combined_state, lines_digest, read_gpu_verdicts, GpuView,
    cpu_state_for_gpu, gpu_decided_keys, GpuJudging, changed_since_head, select_items)  # noqa: F401
from broad_mutation_library import (Stopped, print_refusal, path_is_under, refuse_outside_selftest, check_test_only_variables_before_parsing,
    check_test_only_variables, clean_field, TableRead, read_table, prune_server_client, order_stamp, LibraryRows, filesystem_type_of, acquire_lock,
    export_paths, remove_tree, Library, service_build_directory, default_home, resolve_home, resolve_source, files_in_tree, exported_files,
    sync_tree, canonical_items, generate_list, item_identity, record_list_cache, restore_list, reuse_checks_library, apply_unified_diff,
    raw_mutants_of_package, in_run_scope)  # noqa: F401
from broad_mutation_runner import (registered_functions, package_arguments, step_table, progress_line, Runner, libtest_outcome)  # noqa: F401
from broad_mutation_gpu import (GpuTotals, gpu_bindgen_arguments, record_gpu_verdict, refresh_verdict_rows, gpu_views, combined_states, GpuPass,
    plan_gpu_sentinel, gpu_stash_prepare, record_gpu_item, build_gpu_item, run_gpu_item, run_gpu_phase)  # noqa: F401
from broad_mutation_run import (key_identity, speed_up_judgement, memorise_compositions, memory_room_bytes, key_workers, key_cache_digest,
    write_key_cache, read_key_cache, key_view, prefill_keys, changed_units_of, item_states, stale_counts, read_only_inputs, read_only_judgement,
    list_items, summarise, ArgumentParser, default_threads, check_real_run_is_allowed, multi_host_configuration_for_tests,
    check_multi_host_configuration, check_rocksdb_entry, SuspectStreaks, stop_on_signal, coverage_step, parse_arguments, plan_line, run_items,
    gpu_totals_text, plan_reuse_sentinel, finish_reuse_sentinel, judging_usage, shard_plan_directory, write_shard_plan, shard_piles,
    shard_scope_names, write_shard_counts, shard_counts_text, shard_phase_main)  # noqa: F401
from broad_mutation_selftest_a import (selftest_mutants, process_is_running, write_sources, Selftest, wait_until, summary_field, plan_field,
    selftest_basic_cells, selftest_state_cells, selftest_table_cells, selftest_environment_cells, selftest_sync_cells, selftest_selections,
    selftest_source_copy, selftest_prune_cells, selftest_reuse_cells, selftest_global_cells, selftest_key_speed_cells, selftest_fixture_readers,
    selftest_refused_cells, selftest_baseline_threads_cells, selftest_baseline_key_cells, selftest_baseline_entry_cells, selftest_service_cells,
    selftest_fresh_build_cells, selftest_run_directory_cells, selftest_shared_library_cells, selftest_registered_cells, selftest_test_variable_cells,
    selftest_progress_cells, selftest_stop_cells, selftest_fixture_reader_cells)  # noqa: F401
from broad_mutation_selftest_b import (FIXTURE_MODULES, load_selftest_fixtures, gpu_item_lines, gpu_lines_of, selftest_gpu_cells,
    selftest_gpu_build_first_cells, selftest_service_build_directory_cells, run_selftest_cells, selftest_rocksdb_entry_cells,
    selftest_multi_host_configuration_cells, selftest_cell_functions, selftest_cells_run_together, selftest_cells_after_chains, selftest_cell_chains,
    selftest_chain_name, selftest_chain_jobs, selftest_chain_child, selftest_run_chains, selftest_collect_chains, run_selftest)  # noqa: F401


def main(argv):
    check_test_only_variables_before_parsing(argv)
    arguments = parse_arguments(argv)
    if arguments.selftest:
        exit_code, parallel_ways = run_selftest([name for name in os.environ.get(SELFTEST_CELLS_VARIABLE, "").split(",") if name] or None)
        if exit_code == 0:
            say(f"  ✓ --selftest 的并行：并行 {parallel_ways} 路（各串同时跑，至多每台默认重活线程数）")
        return exit_code
    if arguments.coverage_step:
        return coverage_step(arguments.coverage_step)
    phase_exit = shard_phase_main(arguments)   # 文件头「分片」
    if phase_exit is not None:
        return phase_exit
    read_only = arguments.list_items or arguments.summary
    library = Library(resolve_home(arguments.home, read_only))   # 里面先核只供自证的变量（check_test_only_variables），再核（要写库的用法）库目录落在哪
    lock_home = library.home
    if getattr(arguments, "shard_phase", "") == "shard":
        # 一片的锁、运行目录、日志与 tree.synced 放 <库目录>/shards/shard-<片号>/：两片可同时跑；判定库（服务里那一份）照库目录取，两片写同一份
        lock_home = os.path.join(library.home, "shards", f"shard-{os.environ.get('TASK_SERVER_SHARD', '').split('/')[0]}")
        library.runs = os.path.join(lock_home, "runs")
        library.tree_synced = os.path.join(lock_home, "tree.synced")
        library.logs = os.path.join(lock_home, "logs")
    source = arguments.source_root = resolve_source(arguments.source, library.home, read_only)
    if arguments.list_items:
        if not break_is("list-items-pipe-unhandled"):
            # 接 | head 时读的一头先关了：照 SIGPIPE 的惯例安静退出（被 SIGPIPE 停，shell 里退 141），不打 BrokenPipeError 的回溯
            signal.signal(signal.SIGPIPE, signal.SIG_DFL)
        list_items(library, arguments, source)
        return 0
    if arguments.summary:
        return summarise(library, arguments, source)
    check_real_run_is_allowed()
    check_rocksdb_entry(source)
    check_multi_host_configuration(source, arguments)
    signal.signal(signal.SIGTERM, stop_on_signal)
    signal.signal(signal.SIGINT, stop_on_signal)
    library.lock_descriptor = acquire_lock(lock_home, arguments.lock_wait)
    # 这一趟的运行目录（文件头「库目录」runs/）：专用树、编译目录与在途记录都在里面；正常结束、拒绝、被 TERM / INT 停都在 finally 里删
    arguments.interrupted = library.begin_run()
    exit_code = None
    try:
        exit_code = run_in_the_run_directory(library, arguments, source)
    finally:
        removed = library.end_run()
    if not removed:
        note(f"  ✗ 这一趟的运行目录没删掉（上面一句是为什么）；这一趟本身退 {exit_code}")
        note("  → 怎么办：照上面那句停掉还开着文件的进程、删掉那个目录；下一趟开跑时也会先删上一趟留下的运行目录")
        return exit_code or EXIT_RUN_DIRECTORY_KEPT
    return exit_code


def run_in_the_run_directory(library, arguments, source):
    """main 里拿到锁、建好这一趟的运行目录之后的那一段：换树、（--refresh-list）记清单，或逐条跑。"""
    sync_tree(library, source)
    if arguments.refresh_list:
        judgement = Judgement(library, library.tree, arguments, for_run=True)
        items = generate_list(library)
        written, units = record_list_cache(library, judgement, items)
        say(f"  ✓ 清单 {len(items)} 条，按单元记进判定库 lists/（{units} 个单元，这一趟新写 {written} 条）；不编不跑")
        return 0
    return run_items(library, arguments, source)


if __name__ == "__main__":
    preflight(__file__)
    try:
        sys.exit(main(sys.argv[1:]))
    except Refusal as refusal:
        print_refusal(refusal)
        sys.exit(refusal.code)
    except Stopped:
        print("  ✗ 被 TERM / INT 停了：在途那一条没落行，在途那一步已一起停掉", file=sys.stderr)
        print("  → 怎么办：重跑同一条命令接着跑（在途那一条在新换的树上重判；这一趟的运行目录没删掉的，下一趟开跑先删它）", file=sys.stderr)
        sys.exit(EXIT_STOPPED)
