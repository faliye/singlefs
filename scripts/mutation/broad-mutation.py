#!/usr/bin/env python3
# admission: always 每次调都现算此刻源码上每条变异的单元键再查判定库；哪些条要跑由键与沿用判据定，上一次的结论不替这一次作保
# run-condition: command git python3
# task-server-gpu-arguments: *   每一趟都有带 gpu 特性的那一趟（没卡的机器由驱动记本次未跑），清场与分卡由服务在最外层办（task-server/task_server_lib.py 文件头「GPU」）
# broad-mutation-driver-signature: singlefs 广谱变异逐条驱动（改名或拷走的一份靠这一行认，别删）
"""广谱变异的逐条驱动（.claude/kb/checks-owed.md 的 C489；用户 2026-09-30 定：一直跑、中断要续接不从头来、一条一条可点名；
2026-10-15 定：剪枝的单元是函数，广谱变异一条一条按内容算，不按 HEAD、不按提交算）。

  broad-mutation.py [选哪些条] [怎么跑]             不带选条参数就是剪枝：把 --source 给的工作树（默认本仓的工作区）换进库目录的专用树，由 cargo-mutants
                                                   生成此刻的清单，每条变异现算单元键过沿用判据（scripts/mutation/mutation-prune.py），沿用得了沿用、沿用不了现判；
                                                   成功那句报「现判 N、沿用 M、拒算 R」。重跑同一条命令就是续跑
  broad-mutation.py --refresh-list                 只换树、生成清单、把清单按单元记进库（不编不跑），之后 --list-items / --summary 在同一份源码上列得出全部条
  broad-mutation.py --list-items                   逐行打「序号 <TAB> 包 <TAB> 变异名 <TAB> 状态 <TAB> stale=<同一处变异键对不上的判定行数> <TAB> key=<此刻的键>」，不编不跑、不起 cargo；
                                                   状态是两趟合起来的（CPU 那一趟没红又要 GPU 那一趟的条取合起来的状态，见下「GPU 那一趟」）
  broad-mutation.py --summary                      六栏条数与提交门禁那一行（见下「输出」，六栏按两趟合起来的状态数，另报 GPU 那一趟要几条、复用几条、本次未跑几条）；不编不跑、不起 cargo，只读判定库
  broad-mutation.py --selftest                     自证（假清单、假测试命令、假 GPU，不起 cargo；判定库只写临时 prune-server：照 scripts/prune-server/prune_server_client.py
                                                   文件头「命令行」--with-temporary-service 那一条包一层起，裸跑退 2）；环境变量 BROAD_MUTATION_SELFTEST_CELLS=<组名,…> 只跑点名的几组格（开发时分段跑，
                                                   组名见 SELFTEST_CELL_NAMES，有的组接着前面组留下的库目录；不设就是全部，门禁与提交前跑的都不设）

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
  只读的几种（--list-items、--summary）另认：--toolchain-from <文件>（rustc -Vv 的输出换成这份文件）、
  --tools-from <文件>（工具版本换成这份文件，一行一个）：提交门禁的样本用，真跑与写行的用法不认。

库目录（--home；一处，固定，不随趟、不随判法换目录、没有版本目录；判定不在这里，在下面「判定库」）：
  runs/run-<进程号>-<随机>/   这一趟的运行目录（用户 2026-10-23 定案「自己用完自己删」：编译生成物每趟新建、跑完删，判定库照留、照旧按单元键复用）。
                   拿到库目录的锁之后建（Library.begin_run），正常结束、拒绝退出、被 TERM / INT 停都在 main 的收尾删掉（Library.end_run；
                   删之前经 scripts/hosts/run_build_directory.py 核本机没有进程开着里面的文件，有就不删、逐个报进程，这一趟别的都成了也退 9）。
                   被 KILL 的那一趟留下的，下一趟拿到锁之后先删（同样先核开着的文件；里面有在途记录的报一句被打断在哪一条）。里面四样：
    tree/          专用树：开跑把 --source 那棵树的导出范围逐文件拷进来（写的文件 mtime 取此刻），变异就地改这里的一个文件、判完拷回原件：
                   编译目录在这一趟里跨条复用，每条只付增量编译。导出范围取门禁 crates-mutation-replay 登记的
                   拷贝范围（scripts/mutation/crates-mutation-rows.py 的 copied_paths，去掉 research/ 下的）加 .cargo/
    target/、target-coverage/   CARGO_TARGET_DIR；scratch/ 是每一步的 TMPDIR（每一步开跑前清空），scratch-coverage/ 是覆盖那一步的产物
    inflight.json、inflight.original   在途记录：改文件之前先存原件、记下改的是哪一条，判完照它拷回（临时件 inflight.restore 在运行目录里，不在树里）
  tree.synced      上一次换树拷了几个文件、什么时候（只当记录，不进任何键）
  logs/            没过的那一步的输出（每条每步一份，后一趟同名的盖掉前一趟）
  lock             库目录的排他锁（见下「中断续接」）
判定库（用户 2026-11-01 定「现在就对接 不允许残留」；.claude/rules/verification.md「判定库只有一份」）：全在 prune-server 里那一份，读写只经
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
  compositions/<键>        每把键（变异键、测试键）的组成清单：单元行（种类、限定名、文件:起止、指纹）、依赖行、工具链那几行；
                           作废时照它报出变了的是哪几个单元（--list-items、--summary 的 NO_CONCLUSION 那一行）
  <次序> = <写入时刻的纳秒，二十位>-<进程号>-<这个进程里的序号>-<随机>（order_stamp）：一个进程里只增，按键升序扫回来就是写入的次序
    （同一把键的几行，状态取最后一个定论，见下「结局」）；一行一个条目，几个进程、两台同时追加不互相盖。
    服务回确认（WAL 同步之后）才算落了；没确认就退出的那一行这一趟丢了，那一条下一趟照现判。
  判定行不带校验列：库在服务里，不是谁都手改得了的本地文件；四族都只经客户端写，一样不带。

键（规则与弄坏开关在 scripts/mutation/mutation-prune.py 文件头；单元与闭包由 scripts/gate/lib_function_units.py 算）：
  变异键 = 容器单元（包住这条变异的最小单元）的单元闭包键（闭包里每个单元的种类与去注释指纹、数据单元摘要、入口目标的生效 cfg、
    闭包真引到的外部包照 Cargo.lock 解析出的版本行、全局那几行）+ 变异在容器里的相对位置（去注释的词法单元下标）+ 原文 + 替换文。
  全局那几行（每把键里都有）只留：rustc -vV、cargo nextest / cargo-mutants / llvm-profdata 的版本、构建环境的生效值（编译变量与它们指的程序、
    cargo 配置里 build / profile / target / env / unstable 解析后的值、rust-toolchain 解析后的值、工作区 Cargo.toml 解析后的 [profile]）、判法单元指纹。
    Cargo.toml、.cargo/ 的原文不进（只改注释照样沿用）；排除表不进（加一条排除只让被排除的那几条测试不再算数：抓到它的那条测试进了排除表的现判）；
    --threads 不进（换了线程数照样沿用别的条）。
  判法单元指纹 = lib_function_units.py 在本驱动与 mutation-prune.py 上从 JUDGING_ENTRIES 那几个函数（选条、沿用、结论分类）收的 python 单元闭包键：
    只改注释与文档串不变，改了判结局的函数（连同它们点到的常量）就变，库里的判定一条都不沿用。没有手写的版本号。
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
  记红、撤绿、下趟现判（用户 2026-10-29 弹窗定案；「判红」照提交门禁 broad-mutation 的 gate-verdict 格与 --summary 的 BROAD_MUTATION_GATE 行：missed 没红 > 0 判红，
    没结论（只过了第一段、驱动错、none）也判红但那是「没有定论」，不是一条定论；无效 unviable 与抓到都算有结论、不判红）：
    判红的定论是没红。同一把键的定论（caught / missed / unviable）里最后一个是 missed 的，红照键记着，前面同一把键的好结论（抓到、无效）撤掉，下一趟这一条现判、不沿用：
    不带选条参数的剪枝跑、点名的跑都现判一次，它也不进复用哨兵的候选；现判又出没红，下一趟还是现判（没红每趟现判，直到判出抓到或无效）。
    单独一行没红、与先抓到后来同一把键又判没红，一样处理。抓到、无效照键沿用。--summary、--list-items 与提交门禁不受影响（状态本来就取最后一个定论，没红照没红数）。
    判法在 Judgement.state（CPU 一趟）与 GpuJudging.state（GPU 一趟）里，只在真跑的选条（for_run）下撤绿；两处都在判法单元指纹的闭包里
    （Judgement.state 在 JUDGING_ENTRIES，GpuJudging.state 与下面的 gpu_decided_keys 在 GPU_JUDGING_ENTRIES），改这一段规矩库里对应那一趟的判定一条都不沿用。
    不放在 mutation-prune.py 的 carry_decision 里：carry_decision 判的是「这一行此刻还算不算数」，--summary、提交门禁也靠它定没红的状态，没红在那里一律不算数的话没红数会少，
    同键更早的抓到还会重新冒出来当状态。
    要 GPU 那一趟的条（CPU 没红、又有带 gpu 特性的测试够得着）红不红看合起来的状态：GPU 趟的真实状态是没红才算红，CPU 一趟的没红行才不沿用；GPU 趟抓到 / 无效 / 还没有结论 /
    没结论的，CPU 一趟的没红行照沿用（gpu_decided_keys 在选条之前交给 Judgement.state，GPU 趟没红的那一条两趟都现判）。
    弄坏开关 missed-carried 下没红照旧按覆盖者键沿用（改前的样子）。
两段式：第一段每条只跑验证包之外三个包（singlefs-checker、singlefs-core、singlefs-format）的测试（eatmydata + cargo nextest，遇第一个失败即停）；第一段没红的先落一行 stage-one-only，再进第二段。
  第二段只补第一段没跑到的（用户 2026-09-30 定）：被变异的两个包的文档测试（cargo test --doc）、验证包里的全部测试函数
  （cargo test --release -p singlefs-verification --lib --tests；登记成 crash-case: 的崩溃枚举用例与放量分项经 --skip 排掉，它们另走按块那一路，
  排除单与门禁按条那一趟的 --exclude-registered 是同一张：scripts/gate/admission.py 的 registered_test_functions，读法只在那里一份）；
  都没红才记 missed。标了 #[ignore] 的用例 cargo 不跑，驱动不算（只剩要显卡、量显存与量速度那几条）。第二段被打断的条，续跑时只补第二段。
GPU 那一趟（C489；用户 2026-10-22 定第二段补一趟带 gpu 特性的 验证包，出处 .claude/kb/checks-owed.md 的 C489、records/2026-09-30-里程碑三开工第一轮.md 的 GPU 普查）：
  CPU 那一趟（上面的两段）全跑完之后，对「CPU 那一趟最后落的结论是没红、又有带 gpu 特性的测试够得着」的条另补一趟带 gpu 特性的 验证包（verification.md 里带特性那一趟）：
    cargo test --release -p singlefs-verification --features <tests/ 里 cfg 点到的特性> --lib / --test <挂特性的测试文件>… -- --nocapture --test-threads 1
    特性与挂特性的文件每趟从 验证包 tests/ 顶层每个 .rs 的 #[cfg(…)] / #![cfg(…)] 现读（gated_test_files，同门禁 crash-point-replay 的 feature-gated-tests 格），
    一个文件都不挂特性、或特性里没有 gpu：没有这一趟。每条先编（cargo test --no-run，编不过记无效）再跑测试步；测试步直接起，不另套清场（清场与分卡归统一任务服务，
    在最外层给整条驱动套一次、卡经 GPU_CARDS_BY_PRIORITY 交给测试），GPU 测试各占卡的显存所以 --test-threads 1；BINDGEN_EXTRA_CLANG_ARGS 取 scripts/crash/crash-amplification-lib.sh
    的 CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS（与崩溃放量同一个值，编译产物才共用）；--threads 同 CPU 那一趟，每一步不设内存上限、不限时（见下「内存与限时」）。
  要不要这一趟：一条两趟里任一趟红算抓到，所以只给 CPU 那一趟没红（missed）的条跑；还要有一条这一趟跑得到的测试（验证包的库与挂特性的测试文件里不标 #[ignore] 的）带 gpu 特性的静态闭包
    含这条变异的容器单元、又真引到 gpu 原子（feature = "gpu" 带真值 true，闭包键里 cfg 原子的那一项）——改坏对象落在带 gpu 特性才编进来的代码、或判器单元闭包含它，都是这一条的特例；
    够不着它的、够得着却不引 gpu 的，这一趟帮不上，照旧只有 CPU 那一趟。要跑的目标取这些测试所在的目标（lib、挂特性的文件）。
  结局：编不过 unviable，测试步退 101（有测试没过）caught（抓到它的测试取第一条没过的；--nocapture 加 --test-threads 1 下结果行常被测试自己的输出打散，认不出就读收尾的 failures: 名单），
    全过且跑到了测试 missed；日志里有 NOT_RUN_NO_GPU_ADAPTER（GPU 对拍没找到卡，打一行就返回成功）是本次未跑：不落行、不算没红，
    BROAD_MUTATION_GPU_NOT_RUN 逐条报；退 0 而一个测试都没跑到记 driver-error（不算没红）；认不出的退出码记 driver-error。
  基线：这一趟要跑的目标先在没改过的树上跑一遍（编译步、测试步各一次），不绿拒（退 6，基线不绿后面每一条抓到都不作数）；基线里就见到 NOT_RUN_NO_GPU_ADAPTER，
    整趟本次未跑；这一趟的基线不记库，每趟现跑。
  两趟的结局分开记：CPU 那一趟的行照旧只在 verdicts/（这一节的函数不在 JUDGING_ENTRIES 的闭包里，CPU 那一趟的判法单元指纹不因它们变，库里 CPU 那一趟的判定一条都不作废），
    GPU 那一趟的行在 gpu-verdicts/；一条的合起来的状态（gpu_combined_state）：CPU 没红、要 GPU 趟的条，GPU 抓到 / 没红 / 无效就是定论，驱动错是没结论，
    库里还没有它的 GPU 结论是 none（理由 gpu-not-run，不算没红）；别的条就是 CPU 那一趟的状态。--summary 的六栏、MISSED / NO_CONCLUSION 明细、提交门禁那一行和 --list-items 的状态都按合起来的状态数。
  沿用：GPU 判定行的键是变异的单元键，另带 gpu_key（这一趟的写法、编进来的特性、GPU 趟的判法单元指纹 gpu_judging_key，从 GPU_JUDGING_ENTRIES 收，与 CPU 那一趟的指纹分开）；
    抓到的要抓到它的测试此刻带特性的测试键（它的单元闭包键）与行里的相同，没红的要覆盖者键（这一趟够得着它的那些测试各自测试键的多重集）相同才算数（真跑一律不沿用、每趟现判，见「记红、撤绿、下趟现判」），无效的键相同就沿用，
    驱动错不是结论；CPU 那一趟沿用不了重判过的条，GPU 那一趟照自己的键判；复用哨兵 GPU 趟另有一份（项名 gpu:<序号>）。
  两台各跑一趟时，GPU 趟在拿卡的那一台（或两台）上跑，判定照样写进服务里同一份 gpu-verdicts/；键相同、gpu_key 相同的行照常沿用，
    GPU 趟只现判没有结论的条（沿用判据的其余几问——测试键、覆盖者键——仍在用到时现核，核不过的那一条照现判）。
  CPU 那一趟的前提（不归这一节）：mutation-prune.py 的 coverers_key 跳过默认 cfg 下不编的测试（tests/ 里整个文件 #![cfg(feature = "gpu")] 的，entry-cfg-inactive；
    它们不可能是 CPU 那一趟的覆盖者，不算拒、不进条数），CPU 那一趟没红的行的覆盖者键因此算得出，GPU 那一趟才选得到它们；别的拒因仍整条拒（状态 none，理由 coverer-refused）。
    弄坏开关 coverers-cfg-inactive-refused（BROAD_MUTATION_BREAK）：不跳过、照旧整条拒，红在 mutation-prune.py --selftest 的覆盖者键那一格；本驱动的 --selftest 不另设这一格。
中断续接：任何时刻被 TERM / KILL，重跑同一条命令：沿用得了的条一条都不重跑（带 --item 的一样）；被打断的那一条没有新行、只重跑它（或只补它的第二段）；
  被 KILL 的那一趟的运行目录（改过的树、在途记录、编译目录）下一趟拿到锁之后先删掉，这一趟在新建的运行目录里照源码换树（库目录「runs/」）。
  被 TERM / INT 停：在途那一步连同它的进程组一起停（给驱动自己起的那个子进程的组发 TERM，到 --stop-grace 秒还在就 KILL），不留着它跑完。
  被 KILL：留下的那一步还活着时占着库目录的锁（锁的描述符在驱动与这一步起的进程手里）：等 --lock-wait 秒，还占着就退 4、不跑。
基线：每一步、每个线程数（--threads）按基线键先在没改过的树上跑一遍，全绿才开始跑变异（基线红了一律退 6）。
  取基线按（基线键, 步名, 线程数）取最后一行：是绿的就沿用，不然在这个线程数下重跑那一步、追加一行；别的线程数的行不动、不盖。
  基线键（Judgement.baseline_key，D13 已定项 17）= 这一步真跑到的每一条判定各自单元闭包键的多重集、这一步的命令（去掉线程数）、全局那几行（判法单元指纹除外）：
    判定是测试函数（mutation-prune.py 的 test_entries 静态列：第一段那三个包、验证包 singlefs-verification 里不标 #[ignore] 的，
    去掉登记的崩溃枚举用例与放量分项）、这几个目标里的条目级宏调用（宏展开出来的测试静态列不出，把宏调用连同它的闭包算进去）、
    被变异两个包的文档测试；验证包带 --release，闭包照 cfg 去掉 debug_assertions 收。闭包键里有闭包单元的种类与去注释指纹、闭包真引到的
    cfg 原子、依赖版本行、工具链那几行；在这一步的 cfg 与特性下不编的判定不算这一步跑到的。不带步名、手写的版本号、包级与文件级指纹：
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
  弄坏开关 test-steps-configuration-unpassed 下不判、不设（改前的样子）。
内存与限时（用户 2026-10-31 定 SC02）：每一步命令直接起，不经线程包装、内存包装与清场包装，不设内存上限、不限时，
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
弄坏开关 BROAD_MUTATION_BREAK=<项>（逗号分隔，只给 --selftest 证红用，只在 --selftest 自己起的子进程里生效）。
逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 run_selftest_cells 里那几组格的函数名，括号里是那条「✗ 自证：」的开头；
每一项红时都另有汇总行「✗ --selftest：<格数> 格里 N 格判错」，退 1）：
  BROAD_MUTATION_BREAK=list-items-pipe-unhandled  --list-items 不照 SIGPIPE 惯例退（接 | head 时打 BrokenPipeError、退 120）
      红：「✗ 自证：--list-items 接 | head 时没照 SIGPIPE 惯例安静退」
  BROAD_MUTATION_BREAK=own-libtest-failure-line  libtest 结果行回到驱动自己那条正则取红的测试名（名字带上「 - should panic」），不经 lib_libtest_result_lines.py
      红：「✗ 自证：两段式的结局不对」
  BROAD_MUTATION_BREAK=rerun-done  沿用得了的条照样重跑（不读判定库）
      红：「✗ 自证：单独一条没红（CPU 一趟）下一趟没现判、或抓到的、无效的没沿用」、「✗ 自证：先抓到、同一把键又判没红（CPU 一趟）下一趟没现判、或现判之后没红仍沿用」、「✗ 自证：现判又出没红的条（CPU 一趟）下一趟没接着现判」、「✗ 自证：同一条变异内容没变、判定库里有可沿用的行、所在文件在某提交之后改过别处，没照样沿」、「✗ 自证：抓到它的那条测试改了之后那几条没现判」、「✗ 自证：多登记一条之后别的条没照样沿用」、「✗ 自证：只改 Cargo.toml 注释之后别的条没照样沿用」、
      「✗ 自证：拒算的条没有每趟现判、或别的条没沿用」、「✗ 自证：只补第二段不对」、「✗ 自证：重起之后沿用得了的条或基线被重跑了」、
      「✗ 自证：点名沿用得了的条没跳过」、「✗ 自证：点名现判（自证变量 BROAD_MUTATION_REJUDGE_NAMED）没有重跑并追加新行」、「✗ 自证：第二段的定论被后来的 stage-one-only 行降级了」、「✗ 自证：抓到的条被后来的驱动错行盖掉了」、「✗ 自证：认不出的退出码没处理对」、
      「✗ 自证：工具链没进每一把键」、「✗ 自证：两台各点名一半的结果没合对」
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
      「✗ 自证：GPU 趟没红的条下一趟没现判」、「✗ 自证：GPU 趟抓到的条（CPU 一趟没红）下一趟没沿用」、「✗ 自证：GPU 趟先抓到、同一把键又判没红的条下一趟没现判」
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
  BROAD_MUTATION_BREAK=item-counts-dropped  每条一行 BROAD_MUTATION_ITEM 不带 judged=、red=、reused=（改前的样子）
      红：「✗ 自证：BROAD_MUTATION_ITEM 每条一行没带判了、红了、复用了多少」
  BROAD_MUTATION_BREAK=test-variables-anywhere  只供自证的变量隔一层进程、库目录在自证工作目录之外也认
      红：「✗ 自证：只供自证的变量指着自证工作目录之外的库时没拒」、「✗ 自证：父进程不是 --selftest 的进程带着假一步的变量时没拒」、「✗ 自证：父进程不是 --selftest 的进程只带着弄坏开关时没拒」
      另红：「✗ 自证：平常跑设了弄坏开关、带认不得的选项时没在参数解析之前拒并点名」
  BROAD_MUTATION_BREAK=break-guard-after-parsing  只供自证的变量（含这个弄坏开关）平常跑设了的拒排回参数解析之后（改前的样子：认不得的选项先被 argparse 当用法错拒，不点名变量）
      红：「✗ 自证：平常跑设了弄坏开关、带认不得的选项时没在参数解析之前拒并点名」
  BROAD_MUTATION_BREAK=missed-carried  没红（CPU 一趟、GPU 一趟）按覆盖者键相同照旧沿用、不现判（改前的样子）
      红：「✗ 自证：单独一条没红（CPU 一趟）下一趟没现判、或抓到的、无效的没沿用」、「✗ 自证：同一条变异内容没变、判定库里有可沿用的行、所在文件在某提交之后改过别处，没照样沿用」、
      「✗ 自证：抓到它的那条测试改了之后那几条没现判」、「✗ 自证：先抓到、同一把键又判没红（CPU 一趟）下一趟没现判、或现判之后没红仍沿用」、「✗ 自证：现判又出没红的条（CPU 一趟）下一趟没接着现判」、
      「✗ 自证：GPU 趟没红的条下一趟没现判」、「✗ 自证：GPU 趟先抓到、同一把键又判没红的条下一趟没现判」
  BROAD_MUTATION_BREAK=state-is-last-row  一条的状态照旧取最后一行
      红：「✗ 自证：第二段的定论被后来的 stage-one-only 行降级了」、「✗ 自证：抓到的条被后来的驱动错行盖掉了」、「✗ 自证：汇总不对」
  BROAD_MUTATION_BREAK=named-items-always-rerun  点名的条沿用得了也重跑
      红：「✗ 自证：点名沿用得了的条没跳过」、「✗ 自证：点名现判（自证变量 BROAD_MUTATION_REJUDGE_NAMED）没有重跑并追加新行」、「✗ 自证：抓到的条被后来的驱动错行盖掉了」
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
      红：「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑」、「✗ 自证：只补第二段不对」、「✗ 自证：两段式的结局不对」、「✗ 自证：汇总不对」、「✗ 自证：第二段的步不对」、
      「✗ 自证：第二段的定论被后来的 stage-one-only 行降级了」
      另红：「✗ 提交门禁判红：…条沿用不了、也没有现判的结论」、「✗ 自证：重起之后沿用得了的条或基线被重跑了」
  BROAD_MUTATION_BREAK=term-leaves-step-running  被 TERM 时只给驱动起的那一个进程发一次 TERM，不停它的进程组（改前的样子）
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
      红：「✗ 自证：抓到它的那条测试改了之后那几条没现判」
      另红：「✗ 自证：reuse-three-questions：」
  BROAD_MUTATION_BREAK=summary-gate-silent  --summary 有没红或没结论也退 0
      红：「✗ 自证：先抓到的第 6 条点名现判（自证变量）判成没红之后，汇总没按没红数、或这一趟现判的不是只有它」、「✗ 自证：汇总不对」、「✗ 自证：认不出的退出码没处理对」、「✗ 自证：两台各点名一半的结果没合对」
      另红：「✗ 自证：两趟都没红：」、「✗ 自证：只有 CPU 红：」、「✗ 自证：没卡：」
  BROAD_MUTATION_BREAK=stage-two-baseline-red-generic  第二段基线红只报一句不给出路
      红：「✗ 自证：第二段基线红没报成带出路的拒绝」
      另红：「✗ 基线 two-test-checker-tier 在没改过的树上退 101：基线不绿」
  BROAD_MUTATION_BREAK=baseline-any-threads  取基线不看线程数
      红：「✗ 自证：换了 --threads 之后基线没在新线程数下另记」
  BROAD_MUTATION_BREAK=test-steps-configuration-unpassed  真跑测试的步不往环境里设 SINGLEFS_MULTI_HOST_CONFIG、起 cargo 之前也不判多机配置在不在（改前的样子）
      红：「✗ 自证：真跑测试的步没收到多机配置」、「✗ 自证：多机配置不在时没在起 cargo 之前拒」
  BROAD_MUTATION_BREAK=rocksdb-entry-unchecked  真跑起 cargo 之前不调 scripts/hosts/rocksdb-prebuilt.py entry（改前的样子）
      红：「✗ 自证：被判的树里 验证包依赖 rocksdb 而进程环境里没有预编库的取值，没在起 cargo 之前拒」
      （entry 的跳过办法：被判的树里没有依赖 rocksdb 的清单（自证的 SELFTEST_SOURCES 就是这样），或 PATH 上的 cargo 是带
      『# rocksdb-prebuilt-entry: selftest-fake-cargo』那一行的假 cargo；见 scripts/hosts/rocksdb-prebuilt.py 文件头『entry』）
  BROAD_MUTATION_BREAK=no-carry-without-arguments  不带选条参数时不读判定库（全部现判）
      红：「✗ 自证：单独一条没红（CPU 一趟）下一趟没现判、或抓到的、无效的没沿用」、「✗ 自证：先抓到、同一把键又判没红（CPU 一趟）下一趟没现判、或现判之后没红仍沿用」、「✗ 自证：现判又出没红的条（CPU 一趟）下一趟没接着现判」、「✗ 自证：同一条变异内容没变、判定库里有可沿用的行、所在文件在某提交之后改过别处，没照样沿」、「✗ 自证：抓到它的那条测试改了之后那几条没现判」、「✗ 自证：多登记一条之后别的条没照样沿用」、「✗ 自证：只改 Cargo.toml 注释之后别的条没照样沿用」、
      「✗ 自证：拒算的条没有每趟现判、或别的条没沿用」、「✗ 自证：只补第二段不对」、「✗ 自证：重起之后沿用得了的条或基线被重跑了」、
      「✗ 自证：第二段的定论被后来的 stage-one-only 行降级了」、「✗ 自证：认不出的退出码没处理对」、「✗ 自证：工具链没进每一把键」、「✗ 自证：两台各点名一半的结果没合对」
      另红：「✗ 复用失灵：…键相同却现判了」、「✗ 自证：reuse-sentinel：」、「✗ 自证：reuse-three-questions：」
  BROAD_MUTATION_BREAK=select-by-commit  恢复按提交选条：被判那棵树里相对 HEAD 改过的文件里的条一律现判
      红：「✗ 自证：同一条变异内容没变、判定库里有可沿用的行、所在文件在某提交之后改过别处，没照样沿」、「✗ 自证：抓到它的那条测试改了之后那几条没现判」、
      「✗ 自证：先抓到、同一把键又判没红（CPU 一趟）下一趟没现判、或现判之后没红仍沿用」、「✗ 自证：现判又出没红的条（CPU 一趟）下一趟没接着现判」
  BROAD_MUTATION_BREAK=judge-units-raw-text  判法单元指纹取两份脚本整份原文的 sha256（只改注释也变）
      红：「✗ 自证：只改驱动注释判法单元指纹变了」
      另红：「✗ 自证：改了 GPU 趟的函数，CPU 那一趟的判法单元指纹跟着变了」、「✗ 自证：改了 GPU 趟沿用不沿用没红的那一句（GpuJudging.state 里）」
  BROAD_MUTATION_BREAK=refused-carried  拒算的条与抓到它的测试认不出的条照沿用
      红：「✗ 自证：拒算的条没有每趟现判、或别的条没沿用」
  BROAD_MUTATION_BREAK=read-only-volatile-refused  只读的用法（--summary 这一类）也拒落在开机会清空的目录里的库（改前的写法：样本拷进 /tmp 再判时判不了）
      红：「✗ 自证：库目录落在 /dev/shm 里时只读的 --summary 也拒了」
  BROAD_MUTATION_BREAK=list-cache-unchecked  清单按单元记进判定库 lists/ 之后不复原核对
      红：「✗ 自证：清单里有被变异包之外的一条时 --refresh-list 没拒」
  BROAD_MUTATION_BREAK=run-directory-kept  跑完（正常、拒绝、被 TERM / INT 停）不删这一趟的运行目录（改前的样子：编译目录与树跨趟留着）
      红：「✗ 自证：跑完运行目录里不剩它建的编译目录与树副本（正常结束）」、「✗ 自证：跑完运行目录里不剩它建的编译目录与树副本（出错退出）」、「✗ 自证：跑完运行目录里不剩它建的编译目录与树副本（被 TERM 停）」、
      「✗ 自证：这一趟的运行目录里还有进程开着文件时应当不删、报出那个进程、退 9」、「✗ 自证：重起跑完之后库目录里还剩上一趟被 KILL 留下的、或这一趟的运行目录」、「✗ 自证：上一趟被 KILL 留下的运行目录」、
      「✗ 自证：恢复原件半途被杀之后树里多出了文件、临时件不在运行目录里、或重跑没照常判完」
  BROAD_MUTATION_BREAK=run-directory-open-files-unchecked  删这一趟的运行目录之前不核有没有进程开着里面的文件
      红：「✗ 自证：这一趟的运行目录里还有进程开着文件时应当不删、报出那个进程、退 9」
  BROAD_MUTATION_BREAK=build-directory-in-home  照改前把专用树、编译目录、每一步的 TMPDIR 与在途记录放在库目录里、跨趟留着（不建 runs/ 下的运行目录）
      红：「✗ 自证：第二趟的编译目录不是新建的」、「✗ 自证：跑完运行目录里不剩它建的编译目录与树副本（正常结束）」、「✗ 自证：跑完运行目录里不剩它建的编译目录与树副本（出错退出）」、
      「✗ 自证：跑完运行目录里不剩它建的编译目录与树副本（被 TERM 停）」、「✗ 自证：这一趟的运行目录里还有进程开着文件时应当不删、报出那个进程、退 9」、「✗ 自证：驱动被杀之后它那一趟的运行目录里没有留着改过的树与在途记录」、
      「✗ 自证：被打断的第 3 条没有重跑、或被当成已完成」、「✗ 自证：重起跑完之后库目录里还剩上一趟被 KILL 留下的、或这一趟的运行目录」、「✗ 自证：上一趟被 KILL 留下的运行目录」、
      「✗ 自证：恢复原件半途被杀之后树里多出了文件、临时件不在运行目录里、或重跑没照常判完」
  BROAD_MUTATION_BREAK=baseline-by-packages  基线键照改前取这一步编、跑的那几个包（连同路径依赖）里全部单元的指纹（包级），带手写的 form 与步名
      红：「✗ 自证：基线键没按这一步跑到的判定的单元闭包取」、「✗ 自证：基线没按条记：一步里收得全的测试没各自沿用、或拒算的那几条没照跑」
      另红：「✗ 自证：第二趟的编译目录不是新建的」
  BROAD_MUTATION_BREAK=baseline-refused-step-whole  不按条记基线：基线键拒算的测试步照改前整步照跑（一条拒算整步照跑）
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
mutation-prune.py 文件头列的那几项（驱动在自证里把它们转给规则库）打开时这份 --selftest 不一定红，由 mutation-prune.py --selftest 判。
只供测试（--selftest 自己设；这个进程的父进程不是带 --selftest 的这份脚本、或库目录不在那一趟自证的工作目录里，设了任何一个都拒，退 2）：
  BROAD_MUTATION_STEP_DRIVER=1 每一步的真命令换成自证工作目录（BROAD_MUTATION_SELFTEST_WORK）里的 fake-step-driver
    （调法 fake-step-driver <步名> <序号> <树>，不要前缀；文件名写死，起它的命令是 lib_function_units.py 认得的根外程序形态）、
  BROAD_MUTATION_LIST_DRIVER=<脚本> 换掉 cargo mutants --list（调法 <脚本> <包>，打 JSON）、BROAD_MUTATION_EXPORT_EVERYTHING=1 导出范围换成 --source 整棵树、
  BROAD_MUTATION_TOOLCHAIN=<文字> 换掉 rustc -Vv 的输出、BROAD_MUTATION_KILL_AT=<点> 走到那一点给自己发 KILL、
  BROAD_MUTATION_ALLOW_VOLATILE_HOME=1 许库目录落在 /tmp 下、BROAD_MUTATION_REGISTERED=<目标:用例函数,…> 换掉登记的崩溃枚举用例与放量分项的读法（自证的假树里没有登记表）、
  BROAD_MUTATION_PROGRESS_INTERVAL=<秒> 换掉进度间隔、BROAD_MUTATION_TOOL_VERSIONS=<文字> 换掉工具版本、
  BROAD_MUTATION_JUDGING_KEY=<文字> 换掉本仓这一份的判法单元指纹（自证每一趟不现算它，判法单元指纹那几格在拷贝上现算）、
  BROAD_MUTATION_GPU_JUDGING_KEY=<文字> 换掉本仓这一份的 GPU 趟判法单元指纹（同上，GPU 趟指纹那一格在拷贝上现算）、
  BROAD_MUTATION_STAGE_ONE_ONLY=1 每条只跑第一段、GPU 那一趟不起（造「第二段被打断」那种只过了第一段的现场；命令行没有这样的参数，真跑一律两段跑完再跑 GPU 那一趟）、
  BROAD_MUTATION_REJUDGE_NAMED=1 点名的条沿用得了也现判（GPU 那一趟也不读库）、追加新行、复用哨兵不抽（造「同一把键再判出另一个结局」的现场；命令行没有这样的参数，真跑的强制现判只走复用哨兵）。

gate-similar: crates-mutation-replay.sh 它按 crates/mutations.tsv 逐条改坏、点名的测试必须红，行是人写的、判的是「这条测试抓不抓得到」；这里的清单由 cargo-mutants 生成、判每条变异有没有任何测试抓到，导出范围取它登记的那一份
gate-similar: mutate.sh 它跑一张手写变异表对一个实验二进制，每条 cargo test 直接起、不设内存上限、不限时（整条 mutate.sh 是统一任务服务的一个任务）；这里每一步同样直接起，清单是生成的、判定按单元键落在库目录里沿用与续接
gate-similar: crash-amplification.sh 它是崩溃放量的分项驱动，--list-items / --item 与「重跑同一条命令就是续跑」的形态照它；判的对象不同（崩溃点状态对变异）
gate-similar: crash-point-replay.sh 它的 feature-gated-tests 格对崩溃枚举用例的状态块跑同一条带特性的 验证包命令、判的是「状态块有没有红」；这里同一条命令判的是「CPU 那一趟没红的变异有没有被带特性的测试抓到」，特性与文件的现读写法照它的
gate-similar: mutation-shard-run.sh 它把 crates/mutations.tsv 的行分给两台、两片判的都写进 prune-server 里同一份库，合并只按此刻的行键核两片判的行在服务里读得到；这里不分片，两台各起一趟、判的同样写进服务里那一份（族 broad-mutation，判定按写入次序只追加），表与判据都不同，不共用
"""
import argparse
import datetime
import difflib
import fcntl
import hashlib
import importlib.util
import json
import os
import re
import secrets
import shlex
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import zoneinfo
from typing import NamedTuple

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY, ".claude", "scripts"))
from project_preflight import preflight  # noqa: E402
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "hosts"))
from heavy_threads import HeavyThreadDefaultError, default_heavy_threads  # noqa: E402
import run_build_directory  # noqa: E402  scripts/hosts/run_build_directory.py：这一趟的运行目录跑完删、删之前核开着的文件（文件头「库目录」runs/）

SELF_PATH = os.path.join(SCRIPT_DIRECTORY, os.path.basename(__file__))
# libtest 的结果行（`test <全名>[ - should panic] ... <结局>`）全仓只在 lib_libtest_result_lines.py 解析一份：should panic 那一段不进名字
_LIBTEST_RESULT_LINES_SPECIFICATION = importlib.util.spec_from_file_location("lib_libtest_result_lines_for_broad_mutation",
                                                                             os.path.join(SCRIPT_DIRECTORY, "..", "gate", "lib_libtest_result_lines.py"))
libtest_result_lines = importlib.util.module_from_spec(_LIBTEST_RESULT_LINES_SPECIFICATION)
_LIBTEST_RESULT_LINES_SPECIFICATION.loader.exec_module(libtest_result_lines)
# 复用三问与复用哨兵全仓一份（scripts/gate/lib_reuse_checks.py 文件头「一」「二」），不进判法单元
_REUSE_CHECKS_SPECIFICATION = importlib.util.spec_from_file_location("lib_reuse_checks_for_broad_mutation", os.path.join(SCRIPT_DIRECTORY, "..", "gate", "lib_reuse_checks.py"))
reuse_checks = importlib.util.module_from_spec(_REUSE_CHECKS_SPECIFICATION)
_REUSE_CHECKS_SPECIFICATION.loader.exec_module(reuse_checks)
SCRIPT_KEY = "scripts/mutation/broad-mutation.py"   # 规范输出与复用哨兵认的脚本名
REAL_SELF_PATH = os.path.realpath(__file__)
DEFAULT_HOME_ENTRY = "broad-mutation-units"   # 库目录不给时取数据根下的这一项（.claude/hooks/lib_work_root.py 的 shared_data_path，两台读同一份）
DEFAULT_LOCK_WAIT_SECONDS = 120
DEFAULT_STOP_GRACE_SECONDS = 10
DEFAULT_SUSPECT_STREAK = 8           # 依据见文件头 --suspect-streak
DEFAULT_SUSPECT_TEST_SECONDS = 0.2
PROGRESS_INTERVAL_SECONDS = 300      # 一步跑着时隔多久报一行进度（文件头「输出」）
LOCK_POLL_SECONDS = 2
MUTATED_PACKAGES = ("singlefs-core", "singlefs-checker")    # 清单的次序：先 core 后 checker，序号从 1 起
STAGE_ONE_PACKAGES = ("singlefs-checker", "singlefs-core", "singlefs-format")   # 第一段：验证包之外的三个包
VERIFICATION_PACKAGE = "singlefs-verification"   # 第二段：验证包 crates/singlefs-verification
REGISTERED_REASON = "登记的崩溃枚举用例或放量分项（门禁 crash-point-replay 按块那一路判，这里经 --skip 排掉）"
ADMISSION = []                                  # scripts/gate/admission.py（第一次用到时才载入，registered_functions 用）
SUBMIT_PREFIX = "task-server/task submit --"   # 出路里起重活那条命令的前缀（文件头「谁起、什么时候起」）
VOLATILE_PREFIXES = ("/tmp", "/run", "/dev/shm", "/var/run")
VOLATILE_FILESYSTEMS = ("tmpfs", "ramfs")
IGNORED_DIRECTORY_NAMES = ("target", "mutants.out", "mutants.out.old")
# gate-overlap:copy-kept lib_verdict_library.py 广谱变异的判定是按写入次序只追加的行（同一把单元键几行、状态取最后一个定论），判定库共用部分按项名与两列键一项存一份 JSON；形态不同，读法、记红撤绿都不抽
VERDICT_COLUMNS = ("key", "sequence", "package", "name", "outcome", "stage", "caught_by", "test_key", "coverage_key", "build_seconds", "test_seconds",
                   "judged_on", "method", "locator", "refused", "note")
# 判定库在 prune-server 里（文件头「判定库」）：族与键空间照 scripts/prune-server/prune_server_client.py 文件头「键空间」，种类固定，表是相对路径的头一段
PRUNE_SERVER_CLIENT_PATH = os.path.join(SCRIPT_DIRECTORY, "..", "prune-server", "prune_server_client.py")
PRUNE_SERVER_CLIENT_MODULE = "singlefs_prune_server_client"     # 与 lib_mutation_units.py 同一个模块名：一个进程只载入一份客户端
LIBRARY_KIND = "broad-mutation-units"
VERDICTS_TABLE, GPU_VERDICTS_TABLE, BASELINES_TABLE, BASELINE_ENTRIES_TABLE, COVERAGE_TABLE, LISTS_TABLE, COMPOSITIONS_TABLE = (
    "verdicts", "gpu-verdicts", "baselines", "baseline-entries", "coverage", "lists", "compositions")
ORDER_STATE = {"last": 0, "count": 0}    # order_stamp 在这个进程里用到的最后一个纳秒数与序号
BASELINE_COLUMNS = ("baseline_key", "step", "outcome", "finished_at", "threads")
# 按条记基线（文件头「基线」）：一行一份在没改过的树上跑绿的记录；kind 是 test（一条测试）、rest（一个二进制里宏调用展开的那几条）、binary（验证包一个二进制，
# 有拒算的是去掉那几条的那一份），binary 只当索引（rest 另按它找）
BASELINE_ENTRY_COLUMNS = ("record_key", "step", "threads", "kind", "binary", "finished_at")
COVERAGE_COLUMNS = ("record_key", "stage", "binary", "test", "covered", "profiles", "judged_on")
# 判法单元指纹（文件头「键」）从这几个函数收 python 单元闭包：选条、沿用、结论分类（连同它们点到的常量与被调的函数）
# 方法写 <类>.<方法>（lib_function_units.py 的 python 限定名）：Judgement、UnitTree 是调用方传进来的对象，只从函数起收不到它们的方法，另点名
JUDGING_ENTRIES = (("scripts/mutation/broad-mutation.py", "select_items"), ("scripts/mutation/broad-mutation.py", "verdict_state"),
                   ("scripts/mutation/broad-mutation.py", "Judgement.state"), ("scripts/mutation/broad-mutation.py", "Judgement.row_holds"),
                   ("scripts/mutation/broad-mutation.py", "Judgement.selection"), ("scripts/mutation/broad-mutation.py", "process_item"),
                   ("scripts/mutation/mutation-prune.py", "carry_decision"), ("scripts/mutation/mutation-prune.py", "mutation_key"),
                   ("scripts/mutation/mutation-prune.py", "UnitTree.closure_key"), ("scripts/mutation/mutation-prune.py", "catching_test_key"),
                   ("scripts/mutation/mutation-prune.py", "coverers_key"), ("scripts/mutation/mutation-prune.py", "global_lines"),
                   ("scripts/mutation/mutation-prune.py", "build_environment_lines"))
# 收判法单元闭包时扫哪几份（驱动、规则库、libtest 结果行的解析）；它们按字面路径载入的 lib_function_units.py 照样收进来
JUDGING_SCAN_FILES = ("scripts/mutation/broad-mutation.py", "scripts/mutation/mutation-prune.py", "scripts/gate/lib_libtest_result_lines.py")
# 按覆盖逐条（验证包逐个二进制）挑测试、基线键拒算时按条记基线的那三个测试步；文档测试那一步每条都整步跑，不在里面
PER_ENTRY_TEST_STEPS = ("one-test", "two-test-checker-tier")
EXIT_GATE_RED = 8
DECISIVE_OUTCOMES = ("caught", "missed", "unviable")          # 这条变异自己的结局
STAGE_ONE_ONLY = "stage-one-only"
DRIVER_ERROR = "driver-error"
NOT_RUN = "none"
ROW_OUTCOMES = (*DECISIVE_OUTCOMES, STAGE_ONE_ONLY, DRIVER_ERROR)
SUMMARY_COLUMNS = (*DECISIVE_OUTCOMES, STAGE_ONE_ONLY, DRIVER_ERROR, NOT_RUN)
SUMMARY_KEYS = {"caught": "caught", "missed": "missed", "unviable": "unviable",
                STAGE_ONE_ONLY: "stage_one_only", DRIVER_ERROR: "driver_error", NOT_RUN: "not_run"}
OUTCOME_TEXT = {"caught": "抓到", "missed": "没红", "unviable": "无效",
                STAGE_ONE_ONLY: "只过了第一段", DRIVER_ERROR: "驱动错", NOT_RUN: "沿用不了、还没现判"}
NEXTEST_TEST_FAILED_EXIT = 100       # cargo nextest run：有测试没过
CARGO_FAILED_EXIT = 101              # cargo：编不过、或 cargo test 有测试没过
EXIT_INCONSISTENT, EXIT_USAGE, EXIT_LOCKED, EXIT_DIRTY_TREE, EXIT_BASELINE_RED, EXIT_STOPPED = 1, 2, 4, 5, 6, 143
EXIT_RUN_DIRECTORY_KEPT = 9          # 这一趟别的都成了，运行目录没删掉（有进程开着里面的文件，或回读还在）
NEXTEST_FAILURE_LINE = re.compile(r"^\s*(?:FAIL|TIMEOUT|ABORT|LEAK-FAIL|SIG[A-Z0-9]+)\s+\[[^\]]*\]\s+(?:\([^)]*\)\s+)?(\S.*?)\s*$")
# libtest 那几步（文档测试、验证包）的日志里认出二进制的两种行：cargo test 起每个测试二进制前打的 Running、文档测试打的 Doc-tests
LIBTEST_RUNNING_LINE = re.compile(r"^\s*Running (?:unittests )?(?P<source>\S+) \((?P<executable>[^)]+)\)\s*$")
LIBTEST_DOCTESTS_LINE = re.compile(r"^\s*Doc-tests (?P<crate>\S+)\s*$")
# 进度行里数的几种日志行：编了一个 crate、nextest 过了一条、libtest 过了一条
COMPILING_LINE = re.compile(r"^\s*Compiling \S")
NEXTEST_PASS_LINE = re.compile(r"^\s*PASS \[")
# 编译那一步的失败才算无效：日志里有一条带源码位置的 rustc 错误（error 行后面两行之内有 `--> 文件:行:列`），又有 cargo 的「could not compile」
COMPILE_ERROR_LINE = re.compile(r"^error(?:\[E\d+\])?: ")
COMPILE_ERROR_LOCATION = re.compile(r"^\s*--> \S+:\d+:\d+")
COULD_NOT_COMPILE_LINE = re.compile(r"^error: could not compile ")
HUNK_HEADER = re.compile(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@")
UNRECOGNISED_TEST = "（没从输出里认出用例名，看日志）"
# cargo-mutants 自己改文件时跟在替换文后面的那段注释：照样带上，树里改出来的文件与它自己改的逐字相同（generate_list 拿它给的 diff 核）
MUTATION_MARKER = " /* ~ changed by cargo-mutants ~ */"
BASELINE_SEQUENCE = 0
ENVIRONMENT_SEQUENCE = "environment"      # 在没改过的树上验环境的那几步，日志与假一步的调用记录里用这个名字
SELFTEST_WORK_VARIABLE = "BROAD_MUTATION_SELFTEST_WORK"
SELFTEST_CELLS_VARIABLE = "BROAD_MUTATION_SELFTEST_CELLS"     # 开发时分段跑自证：逗号隔开的几组格名（SELFTEST_CELL_NAMES）；不设就是全部，门禁与提交前跑的都不设
SELFTEST_STEP_DRIVER_NAME = "fake-step-driver"   # 自证写在自证工作目录里的假一步（文件头「只供测试」BROAD_MUTATION_STEP_DRIVER）
SELFTEST_TRANSCRIBED_FAILURE_MARK = "〔转述〕"   # 失败行转述的驱动输出里，✗ 换成它（文件头开关表前那一段「转述」）
SELFTEST_DRIVER_FAILURE_LINES_SHOWN = 8          # 失败行末尾整行接上的、这一格起的驱动报的 ✗ 行：去掉数字之后相同的并成一种，至多这么多种，多的只报种数
TEST_ONLY_VARIABLES = ("BROAD_MUTATION_STEP_DRIVER", "BROAD_MUTATION_LIST_DRIVER", "BROAD_MUTATION_EXPORT_EVERYTHING", "BROAD_MUTATION_STAGE_ONE_ONLY", "BROAD_MUTATION_REJUDGE_NAMED",
                       "BROAD_MUTATION_TOOLCHAIN", "BROAD_MUTATION_KILL_AT", "BROAD_MUTATION_ALLOW_VOLATILE_HOME", "BROAD_MUTATION_REGISTERED", "BROAD_MUTATION_BREAK",
                       "BROAD_MUTATION_PROGRESS_INTERVAL", "BROAD_MUTATION_TOOL_VERSIONS", "BROAD_MUTATION_JUDGING_KEY", "BROAD_MUTATION_GPU_JUDGING_KEY")
SELFTEST_ANCESTOR_DEPTH = 6


class Stopped(BaseException):
    """驱动收到 TERM / INT：在途那一条不落行，拷回原件再退。"""


class Refusal(Exception):
    def __init__(self, message, howto, code=EXIT_USAGE):
        super().__init__(message)
        self.message, self.howto, self.code = message, howto, code


def refuse(message, howto, code=EXIT_USAGE):
    raise Refusal(message, howto, code)


def print_refusal(refusal):
    print(f"  ✗ {refusal.message}", file=sys.stderr)
    print(f"  → 怎么办：{refusal.howto}", file=sys.stderr)


def say(text):
    print(text, flush=True)


def note(text):
    print(text, file=sys.stderr, flush=True)


# ── 只供自证的变量与弄坏开关：只在 --selftest 自己起的子进程里生效 ─────────────────────────────────────────

def process_command_line(process_id):
    """一个进程的 (argv, 当前目录, 父进程号)；读不到交 None。"""
    try:
        with open(f"/proc/{process_id}/cmdline", "rb") as handle:
            argv = [word.decode("utf-8", "replace") for word in handle.read().split(b"\0") if word]
        directory = os.readlink(f"/proc/{process_id}/cwd")
        with open(f"/proc/{process_id}/stat", encoding="utf-8", errors="replace") as handle:
            parent = int(handle.read().rsplit(")", 1)[1].split()[1])
    except (OSError, ValueError, IndexError):
        return None
    return argv, directory, parent


def selftest_ancestor_distance():
    """往上第几层祖先是「这份脚本带 --selftest」：父进程是 1；往上 SELFTEST_ANCESTOR_DEPTH 层里没有交 None。"""
    process_id = os.getppid()
    for distance in range(1, SELFTEST_ANCESTOR_DEPTH + 1):
        found = process_command_line(process_id)
        if found is None:
            return None
        argv, directory, parent = found
        if "--selftest" in argv[1:] and any(os.path.realpath(os.path.join(directory, word)) == REAL_SELF_PATH for word in argv[1:]):
            return distance
        if parent <= 1:
            return None
        process_id = parent
    return None


def raw_break(name):
    return name in os.environ.get("BROAD_MUTATION_BREAK", "").split(",")


TEST_MODE = None    # 第一次问的时候判一次；探针把驱动当模块载入时可以直接设成 True


def test_mode():
    """只供自证的变量与弄坏开关认不认：这个进程的父进程是带 --selftest 的这份脚本才认。"""
    global TEST_MODE
    if TEST_MODE is None:
        distance = selftest_ancestor_distance()
        TEST_MODE = distance == 1 or (distance is not None and raw_break("test-variables-anywhere"))
    return TEST_MODE


def test_variable(name):
    return os.environ.get(name) if test_mode() else None


def stage_two_skipped():
    """自证造现场用（只供测试，文件头「只供测试」）：BROAD_MUTATION_STAGE_ONE_ONLY=1 时每条只跑第一段、GPU 那一趟不起，第一段没红的落 stage-one-only，
    造出「第二段被打断的条」那种现场。命令行没有这样的参数：真跑一律两段跑完、再跑 GPU 那一趟。"""
    return test_variable("BROAD_MUTATION_STAGE_ONE_ONLY") == "1"


def named_items_rejudged():
    """自证造现场用（只供测试，文件头「只供测试」）：BROAD_MUTATION_REJUDGE_NAMED=1 时点名的条沿用得了也现判（GPU 那一趟也不读库），追加新行，旧行留着，
    复用哨兵不抽；造出「同一把键再判出另一个结局」那种现场。命令行没有这样的参数：真跑的强制现判只走复用哨兵。"""
    return test_variable("BROAD_MUTATION_REJUDGE_NAMED") == "1"


def break_is(name):
    return test_mode() and raw_break(name)


def path_is_under(path, directory):
    return path == directory or path.startswith(directory.rstrip("/") + "/")


def refuse_outside_selftest(present):
    """只供自证的变量设着而这个进程不是 --selftest 自己起的子进程：退 2，点名是哪几个变量只在自证里认。"""
    unset_command = "env " + " ".join(f"-u {name}" for name in present) + f" python3 {SELF_PATH} …"
    refuse(f"环境里设了只供 --selftest 用的变量（{'、'.join(present)}），而这个进程不是 --selftest 自己起的子进程：假的一步、假清单与弄坏开关不许进真跑",
           f"清掉再跑：{unset_command}；要看这些变量怎么用，读文件头「只供测试」一段")


def check_test_only_variables_before_parsing(argv):
    """只供自证的变量平常跑设了就拒，排在一切参数解析之前（认不得的选项也先看这个）；命令行带 --selftest 的是自证本身，放过。
    弄坏开关 break-guard-after-parsing：不在这里拒，退回参数解析之后的 check_test_only_variables 去拒（改前的样子）。"""
    present = [name for name in TEST_ONLY_VARIABLES if os.environ.get(name)]
    if present and "--selftest" not in argv and not raw_break("break-guard-after-parsing") and not test_mode():
        refuse_outside_selftest(present)


def check_test_only_variables(home):
    """只供自证的变量出现在自证之外、或指着自证工作目录之外的库：拒，一步都不做。"""
    present = [name for name in TEST_ONLY_VARIABLES if os.environ.get(name)]
    if not present:
        return
    unset_command = "env " + " ".join(f"-u {name}" for name in present) + f" python3 {SELF_PATH} …"
    if not test_mode():
        refuse_outside_selftest(present)
    work = os.environ.get(SELFTEST_WORK_VARIABLE)
    if raw_break("test-variables-anywhere"):
        return
    if not work or not path_is_under(home, os.path.realpath(work)):
        refuse(f"环境里设了只供 --selftest 用的变量（{'、'.join(present)}），而库目录 {home} 不在这一趟自证的工作目录（{work or '没给'}）里：只供自证的变量不许碰别的库",
               f"自证里的库都建在它自己的工作目录下；真库上跑就清掉这些变量：{unset_command}")


def kill_point(name):
    """自证造强杀的时刻：BROAD_MUTATION_KILL_AT 点到这里就给自己发 KILL。"""
    if test_variable("BROAD_MUTATION_KILL_AT") == name:
        os.kill(os.getpid(), signal.SIGKILL)


def now_text():
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())   # clock-times:allow 结果行的「时刻」一段，造时间戳的格式串


def file_digest(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def fsync_directory(directory):
    descriptor = os.open(directory, os.O_RDONLY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def write_atomically(path, data):
    """同目录临时文件写完、fsync、改名，再 fsync 目录。data 是 bytes。"""
    temporary = f"{path}.partial.{os.getpid()}"
    with open(temporary, "wb") as handle:
        handle.write(data)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(temporary, path)
    fsync_directory(os.path.dirname(path))


def clean_field(value):
    return str(value).replace("\t", " ").replace("\r", " ").replace("\n", " ")


# ── 仓里的登记表（排除表）：本地文件 ──────────────────────────────────────────────────────────────────────

class TableRead(NamedTuple):
    rows: list              # 成形的行（段数对），按文件里的次序
    malformed_lines: list   # 写完了（有换行结尾）而段数不对的行的行号
    partial_tail: str       # 末行没写完（没有换行结尾）时它的文字，不算数；没有是 ""


def read_table(path, column_count):
    """读只追加的表。# 起头的行与空行不算；末行没有换行结尾的是没写完的半行，不算数也不算不成形。"""
    rows, malformed_lines, partial_tail = [], [], ""
    if not os.path.exists(path):
        return TableRead(rows, malformed_lines, partial_tail)
    with open(path, "rb") as handle:
        data = handle.read()
    lines = data.decode("utf-8", errors="replace").split("\n")
    if lines[-1] != "":
        partial_tail = lines[-1]
    for number, line in enumerate(lines[:-1], start=1):
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != column_count:
            malformed_lines.append(number)
            continue
        rows.append(fields)
    return TableRead(rows, malformed_lines, partial_tail)


# ── 判定库：prune-server 里那一份（文件头「判定库」） ─────────────────────────────────────────────────────────

def prune_server_client():
    """scripts/prune-server/prune_server_client.py（判定库服务的客户端），照 PRUNE_SERVER_CLIENT_MODULE 那个名字只载入一次；载入不连服务。"""
    if PRUNE_SERVER_CLIENT_MODULE not in sys.modules:
        specification = importlib.util.spec_from_file_location(PRUNE_SERVER_CLIENT_MODULE, PRUNE_SERVER_CLIENT_PATH)
        module = importlib.util.module_from_spec(specification)
        sys.modules[PRUNE_SERVER_CLIENT_MODULE] = module
        specification.loader.exec_module(module)
    return sys.modules[PRUNE_SERVER_CLIENT_MODULE]


def order_stamp():
    """只追加的表里一行的条目名（文件头「判定库」<次序>）：写入时刻的纳秒（这个进程里只增）、进程号、这个进程里的序号、随机；按名字升序就是写入的次序。"""
    now = max(time.time_ns(), ORDER_STATE["last"] + 1)
    ORDER_STATE["last"] = now
    ORDER_STATE["count"] += 1
    return f"{now:020d}-{os.getpid():010d}-{ORDER_STATE['count']:08d}-{secrets.token_hex(4)}"


class LibraryRows(NamedTuple):
    rows: list              # 段数对的行（{列名: 值}），按写入的次序
    malformed: list         # 段数不对的条目（<表>/<次序>）：不算判定

    def malformed_text(self):
        return f"{len(self.malformed)} 行（例：条目 {self.malformed[0]}）"


# ── 库目录、锁 ─────────────────────────────────────────────────────────────────────────────────────────────

def filesystem_type_of(path):
    best, best_type = "", ""
    try:
        with open("/proc/mounts", encoding="utf-8", errors="replace") as handle:
            for line in handle:
                parts = line.split()
                if len(parts) < 3:
                    continue
                mount_point = parts[1].replace("\\040", " ")
                if (path == mount_point or path.startswith(mount_point.rstrip("/") + "/")) and len(mount_point) > len(best):
                    best, best_type = mount_point, parts[2]
    except OSError:
        return ""
    return best_type


def acquire_lock(home, wait_seconds):
    """库目录的排他锁。描述符可继承：每一步命令都带着它，驱动被 KILL 之后那些进程还活着时锁还占着，下一趟看得见。"""
    os.makedirs(home, exist_ok=True)
    descriptor = os.open(os.path.join(home, "lock"), os.O_RDWR | os.O_CREAT, 0o644)
    os.set_inheritable(descriptor, True)
    deadline = time.monotonic() + wait_seconds
    announced = False
    while True:
        try:
            fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
            return descriptor
        except OSError:
            if time.monotonic() >= deadline:
                os.close(descriptor)
                refuse(f"库目录 {home} 的锁被占着（等了 {wait_seconds} 秒）：另一趟驱动在跑，或上一趟被 KILL 之后它起的那一步（它起的 cargo 与测试进程）还没退",
                       "`ps -o pid,ppid,etimes,args -u \"$(id -u)\"` 找出还在跑的那一趟（cargo、cargo-nextest、测试二进制），"
                       "等它退出再重跑同一条命令；确是上一趟留下的，逐个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>` 停掉再跑；"
                       "要多等就加 --lock-wait <秒>", EXIT_LOCKED)
            if not announced:
                note(f"  · 库目录的锁被占着，最多等 {wait_seconds} 秒（上一趟留下的进程退出就放开）")
                announced = True
            time.sleep(LOCK_POLL_SECONDS)


def export_paths():
    """导出哪些路径：门禁 crates-mutation-replay 登记的拷贝范围（去掉 research/ 下的：那是那道门禁自己的脚本）加 .cargo。
    拷贝范围已登记了 .cargo 就不再加。"""
    specification = importlib.util.spec_from_file_location("crates_mutation_rows_for_broad_mutation",
                                                           os.path.join(SCRIPT_DIRECTORY, "crates-mutation-rows.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    try:
        copied = module.copied_paths(REPOSITORY)
    except Exception as error:  # 登记表读不了、没有那一行
        refuse(f"读不出导出范围（scripts/mutation/crates-mutation-rows.py 的 copied_paths）：{error}",
               "看 .claude/gate.d/stage-inputs.tsv 里 crates-mutation-replay.sh 那一行还在不在、成不成形")
    paths = [path for path in copied if not path.startswith("research/")]
    if ".cargo" not in paths:
        paths.append(".cargo")
    if "crates" not in paths:
        refuse(f"导出范围里没有 crates（读到的是 {copied}）", "看 .claude/gate.d/stage-inputs.tsv 里 crates-mutation-replay.sh 那一行登记的路径")
    return paths


def toolchain_text(directory, environment=None):
    """这一刻的工具链：在 directory 里跑 rustc -Vv 的输出（rustup 按目录与 RUSTUP_TOOLCHAIN 这类变量挑工具链）；environment 给了就在它下面跑。"""
    fake = test_variable("BROAD_MUTATION_TOOLCHAIN")
    if fake is not None:
        return fake.strip() + "\n"
    if test_mode():
        return "selftest-toolchain\n"
    child_environment = None if environment is None else dict(environment)
    try:
        found = subprocess.run(["rustc", "-Vv"], capture_output=True, text=True, cwd=directory, stdin=subprocess.DEVNULL, env=child_environment)
    except OSError as error:
        refuse(f"起不了 rustc -Vv：{error}", "PATH 上要有 rustc（rustup 装的那一个）")
    if found.returncode != 0 or not found.stdout.strip():
        refuse(f"rustc -Vv 退 {found.returncode}：{found.stderr.strip()[-300:]}", f"在 {directory} 里手跑 rustc -Vv 看报错")
    return found.stdout.strip() + "\n"


def remove_tree(path):
    if os.path.lexists(path):
        shutil.rmtree(path)


class ItemState(NamedTuple):
    state: str               # 这一条的状态（ROW_OUTCOMES 之一或 NOT_RUN），判法见 state_of_outcomes
    passed_stage_one: bool   # 这一版里有它的 stage-one-only 行：第一段跑过而且没红


def state_of_outcomes(outcomes):
    """一条各行的结局（按写入的次序）→ ItemState。不取最后一行：
    caught / missed / unviable 取其中最后一行；没有它们而有 stage-one-only 的是它；只有 driver-error 的是它。"""
    passed_stage_one = STAGE_ONE_ONLY in outcomes
    if break_is("state-is-last-row"):
        return ItemState(outcomes[-1] if outcomes else NOT_RUN, passed_stage_one)
    decisive = [outcome for outcome in outcomes if outcome in DECISIVE_OUTCOMES]
    if decisive:
        return ItemState(decisive[-1], passed_stage_one)
    if passed_stage_one:
        return ItemState(STAGE_ONE_ONLY, True)
    if DRIVER_ERROR in outcomes:
        return ItemState(DRIVER_ERROR, False)
    return ItemState(NOT_RUN, False)


def today_text():
    """判定日期：只写日期（东京时区），不写时刻。"""
    return datetime.datetime.now(zoneinfo.ZoneInfo("Asia/Tokyo")).strftime("%Y-%m-%d")


# ── 库目录 ─────────────────────────────────────────────────────────────────────────────────────────────────

class Library:
    """库目录里的各个位置（文件头「库目录」：运行目录、日志、锁）与判定库（文件头「判定库」：prune-server 里那一份，库目录只当它的命名空间的名字，本地不建）。"""

    def __init__(self, home):
        self.home = home
        self.runs = os.path.join(home, "runs")
        self.run_directory = None
        # 专用树、编译目录、每一步的 TMPDIR 与在途记录都在这一趟的运行目录里（文件头「库目录」runs/），begin_run 之前没有。
        # 专用树先是空串：只读的用法不开跑，Judgement 里 os.path.isdir 判它不在，构建环境与 rustc -Vv 就在 --source 上算
        self.tree = ""
        self.target = self.target_coverage = self.scratch = None
        self.inflight_record = self.inflight_original = self.inflight_restore = None
        self.tree_synced = os.path.join(home, "tree.synced")
        self.logs = os.path.join(home, "logs")
        self.manifest = None
        self.lock_descriptor = None
        self.deferred_rows = []
        client = prune_server_client()
        self.store = client.library_store(client.FAMILY_BROAD_MUTATION, LIBRARY_KIND, home)

    def location(self):
        """判定库在服务里的前缀（报错与给人看的那一行用；不是本地路径）。"""
        return self.store.location()

    def scan(self, relative_prefix):
        """按前缀扫判定库：[(相对路径, 值)]，按键升序。服务不可用客户端退 75（弄坏开关 unreachable-as-empty-library 下交空表）。"""
        try:
            return self.store.scan(relative_prefix)
        except prune_server_client().PruneServerFailure:
            if break_is("unreachable-as-empty-library"):
                return []
            raise

    def read(self, relative):
        """判定库里一个条目的值；不在交 None。服务不可用客户端退 75（弄坏开关 unreachable-as-empty-library 下交 None）。"""
        try:
            return self.store.read(relative)
        except prune_server_client().PruneServerFailure:
            if break_is("unreachable-as-empty-library"):
                return None
            raise

    def read_rows(self, table, columns):
        """一张只追加的表（文件头「判定库」）：按条目名（写入的次序）扫回来；段数对的行交 {列名: 值}，不对的条目列进 malformed。"""
        rows, malformed = [], []
        for relative, value in self.scan(f"{table}/"):
            fields = value.decode("utf-8", "surrogateescape").split("\t")
            if len(fields) != len(columns):
                malformed.append(relative)
                continue
            rows.append(dict(zip(columns, fields)))
        return LibraryRows(rows, malformed)

    def append_rows(self, table, rows):
        """往一张只追加的表里写几行（一帧一个写批）：一行一个条目、条目名是写入的次序（order_stamp），服务回确认才返回。"""
        if rows:
            self.store.write_many([(f"{table}/{order_stamp()}", "\t".join(clean_field(field) for field in fields)) for fields in rows])

    def read_verdicts(self):
        table = self.read_rows(VERDICTS_TABLE, VERDICT_COLUMNS)
        deferred = [dict(zip(VERDICT_COLUMNS, [clean_field(field) for field in row])) for row in self.deferred_rows]
        return LibraryRows(table.rows + deferred, table.malformed)

    def record_verdict(self, fields):
        """落一行判定（十六段）：写进判定库、等服务回确认（文件头「一条一条跑」）。"""
        if break_is("no-row-flush"):
            self.deferred_rows.append(fields)
            return
        self.append_rows(VERDICTS_TABLE, [fields])

    def flush_deferred_rows(self):
        if self.deferred_rows:
            self.append_rows(VERDICTS_TABLE, self.deferred_rows)
        self.deferred_rows = []

    def write_composition(self, key, lines):
        """一把键的组成清单（文件头「判定库」compositions/）：同一把键只写一次。"""
        if not key:
            return
        relative = f"{COMPOSITIONS_TABLE}/{key}"
        if self.read(relative) is not None:
            return
        self.store.write(relative, "".join(f"{line}\n" for line in lines))

    def read_composition(self, key):
        data = self.read(f"{COMPOSITIONS_TABLE}/{key}") if key else None
        return None if data is None else data.decode("utf-8", "surrogateescape").splitlines()

    def read_list_cache(self):
        """清单按单元存的那张表（文件头「判定库」lists/）：{清单的单元键: [这个单元里的变异]}。"""
        cache = {}
        for relative, value in self.scan(f"{LISTS_TABLE}/"):
            try:
                cache[relative[len(LISTS_TABLE) + 1:]] = json.loads(value.decode("utf-8", "surrogateescape"))
            except ValueError:
                continue
        return cache

    def write_list_cache(self, rows):
        """清单按单元记：[(清单的单元键, 这个单元里的变异的 JSON)]，同一个单元键再写就盖掉。"""
        self.store.write_many([(f"{LISTS_TABLE}/{unit_key}", mutations) for unit_key, mutations in rows])

    def baseline_rows(self):
        """基线表里段数对的行（{列名: 值}），按写入的次序。"""
        return self.read_rows(BASELINES_TABLE, BASELINE_COLUMNS).rows

    def place_run_directory(self, directory):
        self.run_directory = directory
        self.tree = os.path.join(directory, "tree")
        self.target = os.path.join(directory, "target")
        self.target_coverage = os.path.join(directory, "target-coverage")
        self.scratch = os.path.join(directory, "scratch")
        self.inflight_record = os.path.join(directory, "inflight.json")
        self.inflight_original = os.path.join(directory, "inflight.original")
        self.inflight_restore = os.path.join(directory, "inflight.restore")

    def begin_run(self):
        """这一趟的运行目录（文件头「库目录」runs/）：拿到锁之后先删上一趟被 KILL 留下的运行目录（树、编译目录、在途记录；删之前核没有进程开着里面的文件），
        再新建这一趟的 <库目录>/runs/run-<进程号>-<随机>/。交上一趟被打断在哪一条的在途记录（没有交 None）。
        弄坏开关 interrupted-run-directory-kept：不删上一趟留下的、不认它的在途记录；build-directory-in-home：照改前把树、编译目录与在途记录放在库目录里、跨趟留着。"""
        if break_is("build-directory-in-home"):
            os.makedirs(self.home, exist_ok=True)
            self.place_run_directory(self.home)
            self.run_directory = None
            return None
        interrupted = None
        os.makedirs(self.runs, exist_ok=True)
        if not break_is("interrupted-run-directory-kept"):
            for name in sorted(os.listdir(self.runs)):
                leftover = os.path.join(self.runs, name)
                record_path = os.path.join(leftover, "inflight.json")
                if os.path.isfile(record_path):
                    try:
                        with open(record_path, encoding="utf-8") as handle:
                            interrupted = json.load(handle)
                    except (OSError, ValueError):
                        interrupted = None
                    if interrupted is not None:
                        say(f"  · 上一趟被打断在第 {interrupted['sequence']} 条（改的是 {interrupted['file']}）：它那一趟的运行目录删掉，这一趟在新换的树上重判它")
                note(f"  · 上一趟留下的运行目录 {leftover}（那一趟被 KILL，或删的时候有进程开着里面的文件）：")
                for line in run_build_directory.remove_run_build_directory(leftover)[1]:
                    note(line)
        self.place_run_directory(tempfile.mkdtemp(prefix=f"run-{os.getpid()}-", dir=self.runs))
        return interrupted

    def end_run(self):
        """删这一趟的运行目录（正常结束、拒绝、被 TERM / INT 停都在 main 的 finally 里走到）：删之前核本机没有进程开着里面的文件，
        有就不删、逐个报进程（scripts/hosts/run_build_directory.py）。交删成了没有；没建过（build-directory-in-home）交 True。
        弄坏开关 run-directory-kept：不删；run-directory-open-files-unchecked：不核开着的文件就删。"""
        directory, self.run_directory = self.run_directory, None
        if directory is None:
            return True
        if break_is("run-directory-kept"):
            return True
        removed, lines = run_build_directory.remove_run_build_directory(directory, check_open=not break_is("run-directory-open-files-unchecked"))
        for line in lines:
            note(line)
        return removed


def default_home():
    """库目录的缺省：数据根下的 broad-mutation-units（.claude/hooks/lib_work_root.py 的 shared_data_path，由多机配置定）；那份库读不到、取不到就拒，不经 ~ 取。"""
    try:
        specification = importlib.util.spec_from_file_location("lib_work_root_for_broad_mutation", os.path.join(REPOSITORY, ".claude", "hooks", "lib_work_root.py"))
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        return module.shared_data_path(DEFAULT_HOME_ENTRY)
    except (OSError, ImportError, AttributeError, ValueError) as error:
        refuse(f"取不到库目录的缺省（数据根下的 {DEFAULT_HOME_ENTRY}，.claude/hooks/lib_work_root.py：{error}）",
               "修好那份库、多机配置里写上 SHARED_DATA_ROOT，或给 --home <目录>")


def resolve_home(given, read_only=False):
    """库目录：先核只供自证的变量；要往库里写的用法另拒落在开机会清空的目录里的库。只读的用法（--list-items、--summary）
    不写库，落在哪都照读：共享门禁的 stage-selftest.sh 把样本拷进 mktemp -d（/tmp）再判，样本的库就在那里。"""
    home = os.path.realpath(given or default_home())
    # 只供自证的变量先判：它们带着假的一步与弄坏开关，出现在自证之外比库目录落在哪更根本；放在后面，临时目录那一拒会先把它盖掉
    check_test_only_variables(home)
    if test_variable("BROAD_MUTATION_ALLOW_VOLATILE_HOME") == "1":
        return home
    if read_only and not break_is("read-only-volatile-refused"):
        return home
    existing = home
    while not os.path.exists(existing):
        existing = os.path.dirname(existing)
    under_volatile = any(home == prefix or home.startswith(prefix + "/") for prefix in VOLATILE_PREFIXES)
    if under_volatile or filesystem_type_of(existing) in VOLATILE_FILESYSTEMS:
        refuse(f"库目录 {home} 落在开机会清空的目录里（/tmp、/run、/dev/shm 或 tmpfs）：判定会写进只给自证与样本用的那一份命名空间，进不了两台共用的判定库，运行目录一重启也没了",
               f"换一个落在盘上的目录：--home <目录>；默认是数据根下的 {DEFAULT_HOME_ENTRY}")
    return home


def resolve_source(given, home, read_only=False):
    """被判的那棵工作树（--source，默认本仓根）。要换树的用法里它与库目录不许一个套着另一个；只读的用法（不换树）不拦。"""
    source = os.path.realpath(os.path.expanduser(given or REPOSITORY))
    if not os.path.isdir(source):
        refuse(f"--source 给的 {source} 不是目录", "给被判的那棵工作树（主工作区或一份副本），不给就是本仓根")
    if not read_only and (path_is_under(source, home) or path_is_under(home, source)):
        refuse(f"--source {source} 与库目录 {home} 一个套着另一个：换树会把库目录自己（专用树、编译目录）当源码拷进去",
               "库目录与被判的工作树各在各处：库目录默认在数据根下（.claude/hooks/lib_work_root.py 的 shared_data_path）")
    return source


# ── 换树：把被判的工作树逐文件换进专用树 ───────────────────────────────────────────────────────────────────

def files_in_tree(tree):
    """树里每个文件的相对路径（符号链接也算一个）；叫 target / mutants.out 的目录与 .git 不算。"""
    found = set()
    for directory, directory_names, file_names in os.walk(tree):
        directory_names[:] = sorted(name for name in directory_names if name not in IGNORED_DIRECTORY_NAMES and name != ".git")
        for file_name in file_names:
            found.add(os.path.relpath(os.path.join(directory, file_name), tree))
    return found


def exported_files(source, paths):
    """被判那棵树在导出范围里的文件：{相对路径: 源文件的绝对路径}（跟着符号链接取内容）。"""
    found = {}
    for path in paths:
        full = os.path.normpath(os.path.join(source, path))
        if os.path.isfile(full):
            found[os.path.relpath(full, source)] = full
            continue
        if not os.path.isdir(full):
            continue
        for relative in files_in_tree(full):
            candidate = os.path.join(full, relative)
            if os.path.isfile(candidate):
                found[os.path.relpath(candidate, source)] = candidate
    return found


def sync_tree(library, source):
    """换树（文件头「库目录」runs/ 的 tree/）：把导出范围拷进这一趟新建的专用树（内容不同的才写、mtime 取此刻，源码里没有的删掉）。交 (改了几个, 删了几个)。"""
    paths = ["."] if test_variable("BROAD_MUTATION_EXPORT_EVERYTHING") == "1" else export_paths()
    wanted = exported_files(source, paths)
    if break_is("exclude-re-dropped"):
        # 弄坏开关：.cargo/mutants.toml 不拷进专用树，cargo-mutants 读不到里面的 exclude_re（等于忽略它），清单按单元存的键也不随它变
        wanted.pop(os.path.join(".cargo", "mutants.toml"), None)
    if not any(path.endswith(".rs") for path in wanted):
        refuse(f"被判的那棵树 {source} 在导出范围（{' '.join(paths)}）里一份 .rs 都没有", "--source 给被判的仓根（主工作区或一份副本）")
    os.makedirs(library.tree, exist_ok=True)
    manifest, changed, removed = {}, 0, 0
    for relative, origin in sorted(wanted.items()):
        digest = file_digest(origin)
        manifest[relative] = digest
        target = os.path.join(library.tree, relative)
        if os.path.isfile(target) and not os.path.islink(target) and file_digest(target) == digest:
            continue
        os.makedirs(os.path.dirname(target), exist_ok=True)
        if os.path.lexists(target):
            os.remove(target)
        shutil.copyfile(origin, target)
        shutil.copymode(origin, target)
        changed += 1
    for relative in sorted(files_in_tree(library.tree) - set(wanted)):
        os.remove(os.path.join(library.tree, relative))
        removed += 1
    library.manifest = manifest
    write_atomically(library.tree_synced, (json.dumps({"synced_on": today_text(), "source": source, "files": len(manifest), "changed": changed,
                                                       "removed": removed}, ensure_ascii=False) + "\n").encode())
    say(f"  · 换树：{source} 导出范围里 {len(manifest)} 个文件，{changed} 个内容不同、照写进专用树，删了 {removed} 个源码里没有的")
    return changed, removed


# ── 清单：由 cargo-mutants 生成，逐条核对它给的 diff，按单元存 ───────────────────────────────────────────────

def canonical_items(items):
    """清单的次序与序号（文件头「选哪些条」）：先 core 后 checker，同包里按文件、起止、替换文、变异名排；同名的另带「（第 N 个同名）」。"""
    rank = {package: index for index, package in enumerate(MUTATED_PACKAGES)}
    items.sort(key=lambda item: (rank.get(item["package"], len(rank)), item["file"], tuple(item["start"]), tuple(item["end"]), item["replacement"], item["name"]))
    seen = {}
    for index, item in enumerate(items, start=1):
        base = item["name"]
        seen[base] = seen.get(base, 0) + 1
        if seen[base] > 1:
            item["name"] = f"{base}（第 {seen[base]} 个同名）"
        item["sequence"] = index
    return items


def generate_list(library):
    """此刻专用树上的清单：[{sequence, package, name, file, start, end, replacement}]。每条都拿 cargo-mutants 给的 diff 核一遍
    「照区间换掉」与它自己的改法逐字相同：区间的读法错了会让每一条都改错地方，那样整份结果是假的。"""
    items, original_texts = [], {}
    span_howto = "看 replace_span 对区间的读法（1 起、列按字符、end 不含）与这一份 cargo-mutants 是不是还一致；对不上就不生成清单"
    for package in MUTATED_PACKAGES:
        for mutant in raw_mutants_of_package(library, package):
            file_path, span, replacement = mutant.get("file"), mutant.get("span"), mutant.get("replacement")
            if not isinstance(file_path, str) or not isinstance(span, dict) or not isinstance(replacement, str) or "diff" not in mutant:
                refuse(f"cargo-mutants 给的清单里有一项缺 file / span / replacement / diff：{json.dumps(mutant, ensure_ascii=False)[:300]}",
                       "这一份 cargo-mutants 的 --list --json --diff 输出换了形状：对着 https://mutants.rs/ 改 generate_list 的读法，不照猜的往下跑")
            start = (span["start"]["line"], span["start"]["column"])
            end = (span["end"]["line"], span["end"]["column"])
            if file_path not in original_texts:
                with open(os.path.join(library.tree, file_path), encoding="utf-8") as handle:
                    original_texts[file_path] = handle.read()
            original = original_texts[file_path]
            try:
                mutated = replace_span(original, start, end, replacement + MUTATION_MARKER)
                by_diff = apply_unified_diff(original, mutant["diff"])
            except ValueError as error:
                refuse(f"清单里「{mutant['name']}」这一条套不上原文：{error}", span_howto)
            if mutated == original or mutated.rstrip("\n") != by_diff.rstrip("\n"):
                refuse(f"清单里「{mutant['name']}」照区间换掉之后与 cargo-mutants 自己给的 diff 改出来的不一样（或根本没改动）", span_howto)
            items.append({"package": mutant.get("package") or package, "name": clean_field(mutant["name"]), "file": file_path,
                          "start": list(start), "end": list(end), "replacement": replacement})
    if not items:
        refuse("cargo-mutants 生成的清单是空的", "看树里的 .cargo/mutants.toml 的 examine_globs 与 --package 的包名还对不对得上")
    return canonical_items(items)


NAME_LOCATION_PREFIX = re.compile(r"^[^:]+:\d+:\d+: ")


def item_identity(item):
    """清单里一条的身份（复原核对用）：包、文件、起止、替换文与变异名去掉位置的那一段（位置那一段由复原照此刻的起点重写）。"""
    return (item["package"], item["file"], tuple(item["start"]), tuple(item["end"]), item["replacement"], NAME_LOCATION_PREFIX.sub("", item["name"], count=1))


def record_list_cache(library, judgement, items):
    """清单按单元记进判定库 lists/（文件头「判定库」）：被变异包里每一个单元一行（没有变异的记空表）；之后照这张表在此刻的源码上复原一遍，
    与 cargo-mutants 给的逐条比（变异名、区间、替换文），复原不回去就拒（退 1）。交 (新写了几行, 单元数)。"""
    rules, tree = judgement.rules, judgement.tree
    by_unit = {}
    for item in items:
        unit = tree.container(item["file"], tuple(item["start"]), tuple(item["end"]))
        if unit is None:
            refuse(f"清单里第 {item['sequence']} 条（{item['name']}）落不进任何单元：{item['file']}:{item['start'][0]} 一带切不开",
                   "函数外的项各是单元，切不开是 scripts/gate/lib_function_units.py 的切分缺陷：把那一段与这句话交给改它的人修；不退回按整个文件算",
                   EXIT_INCONSISTENT)
        by_unit.setdefault(id(unit), []).append(rules.list_entry(tree, unit, item))
    cache = library.read_list_cache()
    rows, count = [], 0
    for path in judgement.package_files(MUTATED_PACKAGES):
        for unit in tree.code_units(path):
            count += 1
            unit_key = rules.list_unit_key(tree, unit, judgement.mutants_version, judgement.mutants_configuration)
            entries = by_unit.get(id(unit), [])
            if cache.get(unit_key) != entries:
                rows.append([unit_key, json.dumps(entries, ensure_ascii=False, sort_keys=True)])
                cache[unit_key] = entries
    if rows:
        library.write_list_cache(rows)
    restored, unlisted, unrestorable = restore_list(judgement, cache)
    if [item_identity(item) for item in restored] != [item_identity(item) for item in items] and not break_is("list-cache-unchecked"):
        missing = sorted(set(map(item_identity, items)) - set(map(item_identity, restored)))
        extra = sorted(set(map(item_identity, restored)) - set(map(item_identity, items)))
        refuse(f"清单按单元存之后复原不回去：cargo-mutants 给了 {len(items)} 条，照 lists/ 复原出 {len(restored)} 条（少 {len(missing)} 条，例 {missing[:1]}；"
               f"多 {len(extra)} 条，例 {extra[:1]}；没记清单的单元 {len(unlisted)} 个；复原不回去的 {len(unrestorable)} 处）",
               "看 scripts/mutation/mutation-prune.py 的 list_entry、restore_item 与 list_unit_key：相对位置、原文与变异名去掉位置的那一段要复原得回 cargo-mutants 给的那一条",
               EXIT_INCONSISTENT)
    return len(rows), count


def restore_list(judgement, cache):
    """照清单按单元存的那张表，在此刻的源码上复原清单：交 (清单, 没记清单的单元, 复原不回去的 [(单元, 那一条)])。"""
    rules, tree = judgement.rules, judgement.tree
    items, unlisted, unrestorable = [], [], []
    for package in MUTATED_PACKAGES:
        for path in judgement.package_files([package]):
            for unit in tree.code_units(path):
                unit_key = rules.list_unit_key(tree, unit, judgement.mutants_version, judgement.mutants_configuration)
                if unit_key not in cache:
                    unlisted.append(unit)
                    continue
                for entry in cache[unit_key]:
                    item = rules.restore_item(tree, unit, entry)
                    if item is None:
                        unrestorable.append((unit, entry))
                        continue
                    item["package"] = package
                    items.append(item)
    return canonical_items(items), unlisted, unrestorable


# ── 判定：键、沿用、状态 ───────────────────────────────────────────────────────────────────────────────────

PRUNE_BREAKS = ("catching-test-own-function-only", "mutated-function-only", "container-fingerprint-dropped", "defining-file-in-key", "relative-position-dropped",
                "relative-position-counts-comments", "dependency-lines-dropped", "dependency-whole-lock", "profile-dropped", "manifest-text-in-global-key",
                "exclusions-in-global-key", "catching-test-exclusion-dropped", "unviable-rejudged", "missed-coverers-unchecked", "missed-always-rejudged",
                "list-cache-context-dropped", "outside-mutation-selected", "const-context-runs-selected", "spawning-binary-selected",
                "coverage-paths-unnormalized", "nextest-filter-single-argument", "coverers-cfg-inactive-refused")
PRUNE_MODULE = None


def prune_rules():
    global PRUNE_MODULE
    if PRUNE_MODULE is None:
        specification = importlib.util.spec_from_file_location("mutation_prune_for_broad_mutation", os.path.join(SCRIPT_DIRECTORY, "mutation-prune.py"))
        PRUNE_MODULE = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(PRUNE_MODULE)
        PRUNE_MODULE.BREAKS = frozenset(name for name in PRUNE_BREAKS if break_is(name))
    return PRUNE_MODULE


JUDGING_KEYS = {}


def judging_units_key(root=REPOSITORY):
    """判法单元指纹（文件头「键」）：JUDGING_ENTRIES 那几个函数在 root 上的 python 单元闭包键（lib_function_units.py），连起来取 sha256。
    弄坏开关 judge-units-raw-text：取两份脚本整份原文的 sha256（只改注释也变）。"""
    fake = test_variable("BROAD_MUTATION_JUDGING_KEY")
    if fake and root == REPOSITORY:
        return fake
    if root in JUDGING_KEYS:
        return JUDGING_KEYS[root]
    if break_is("judge-units-raw-text"):
        digest = hashlib.sha256()
        for path in sorted({path for path, _name in JUDGING_ENTRIES}):
            with open(os.path.join(root, path), "rb") as handle:
                digest.update(handle.read())
        JUDGING_KEYS[root] = digest.hexdigest()
        return JUDGING_KEYS[root]
    library = prune_rules().units_library()
    options = library.Options()
    options.under = [place for place in JUDGING_SCAN_FILES if os.path.isfile(os.path.join(root, place))]
    lines = []
    for path, name in JUDGING_ENTRIES:
        try:
            closure = library.closure_of(root, f"{path}::{name}", options)
        except (library.UsageError, library.FunctionUnitsError) as error:
            refuse(f"判法单元指纹算不出：{path}::{name}（{error}）", "JUDGING_ENTRIES 里的函数改了名就照着改那一张表；scripts/gate/lib_function_units.py 起不来就先修它")
        key, refusals = library.closure_key(closure, allow_refused=True)
        lines.append(f"{path}::{name}\t{key}\t{','.join(sorted({refusal.form for refusal in refusals}))}")
    JUDGING_KEYS[root] = hashlib.sha256("".join(line + "\n" for line in lines).encode()).hexdigest()
    return JUDGING_KEYS[root]


def build_environment_of(root, environment):
    """构建环境的生效值（mutation-prune.py 的 build_environment_lines）；配置读不了抛 Refusal（退 2）。"""
    try:
        return prune_rules().build_environment_lines(root, environment)
    except ValueError as error:
        refuse(f"算不出构建环境：{error}", "看报错里点的那份 cargo 配置或 Cargo.toml：要读得了、解析得了")


def verdict_state(outcomes):
    """一条在判定库里键相同、沿用判据认的那几行的结局（按写入的次序）→ ItemState（文件头「判定行」末段）。"""
    return state_of_outcomes(outcomes)


class Judgement:
    """一棵树上的判定：单元（UnitTree）、全局那几行、每条的键、判定库里键相同的行、沿用与状态。"""

    def __init__(self, library, root, arguments, for_run, registered=(), toolchain=None, tools=None):
        self.rules, self.library, self.root, self.arguments, self.for_run = prune_rules(), library, root, arguments, for_run
        try:
            self.tree = self.rules.UnitTree(root)
        except Exception as error:  # lib_function_units 起不来：根不在、清单读不出
            refuse(f"在 {root} 上切不出单元（scripts/gate/lib_function_units.py）：{error}", "看报错：Cargo.toml 要读得了；lib_function_units.py 坏了就先修它", EXIT_INCONSISTENT)
        for path in self.tree.files:
            self.tree.text(path)    # 先把原文都读进来：变异改树的时候算测试键、覆盖者键读的还是没改过的
        self.registered = set(registered)
        self.exclusions = registered_exclusions(self.rules, self.tree, self.registered)
        build_root = library.tree if (not for_run and os.path.isdir(library.tree)) else root    # 构建环境与 rustc -Vv 在驱动编译的那一处算
        self.toolchain = toolchain if toolchain is not None else toolchain_text(build_root)
        self.tools = tools if tools is not None else tool_versions(arguments)
        exclusions_digest = hashlib.sha256("".join(f"{binary} {test}\n" for binary, test, _reason in self.exclusions).encode()).hexdigest()
        self.judging_key = judging_units_key()
        self.globals = self.rules.global_lines(self.toolchain, self.tools, build_environment_of(build_root, os.environ), self.judging_key, exclusions_digest)
        self.mutants_version = next((line for line in self.tools if "mutants" in line), " | ".join(self.tools))
        self.mutants_configuration = self.rules.mutants_configuration_digest(root)
        self.keys, self._tests = {}, None
        self.gpu_decided = set()    # 真跑选条前由 gpu_decided_keys 填：键在里面的条，CPU 一趟的没红行不算红（要 GPU 趟、GPU 趟的真实状态不是没红），照沿用
        self.release_closures = {}   # 带 --release 那一步的判定的闭包（release_entry_key），按（单元, 目标）缓存
        self.table = library.read_verdicts()
        self.by_key, self.by_locator = {}, {}
        for row in self.table.rows:
            self.by_key.setdefault(row["key"], []).append(row)
            self.by_locator.setdefault(row["locator"], []).append(row)

    def package_files(self, packages):
        found = []
        for path in sorted(self.tree.files):
            crate = self.tree.workspace.crate_of_path(path)
            if crate is not None and crate.package in packages:
                found.append(path)
        return found

    def key_of(self, item):
        identity = (item["file"], tuple(item["start"]), tuple(item["end"]), item["replacement"])
        if identity not in self.keys:
            self.keys[identity] = self.rules.mutation_key(self.tree, item, self.globals)
        return self.keys[identity]

    def runnable(self, packages):
        """这几个包里驱动真跑得到的静态测试：标了 #[ignore] 的 cargo 不跑，不算；登记的崩溃枚举用例与放量分项（self.exclusions）经 --skip 排掉，也不算。"""
        return [entry for entry in self.rules.test_entries(self.tree, packages, self.exclusions) if not entry["skipped_by_attribute"]]

    def tests(self):
        """静态列出的测试（覆盖者键用）：第一段（那三个包）、验证包、被变异包的文档测试；登记的崩溃枚举用例与放量分项不算。"""
        if self._tests is None:
            self._tests = self.runnable((*STAGE_ONE_PACKAGES, VERIFICATION_PACKAGE)) + self.rules.doctest_entries(self.tree, MUTATED_PACKAGES)
        return self._tests

    def test_key_for(self, caught_by):
        return self.rules.catching_test_key(self.tree, caught_by, self.globals, self.exclusions)

    def coverers_for(self, unit):
        if unit is None:
            return None, 0, "no-unit"
        return self.rules.coverers_key(self.tree, unit, self.tests(), self.globals)

    def row_holds(self, row, current):
        """判定库里键相同的这一行此刻还认不认（沿用判据）：→ (认不认, 为什么不认)。"""
        outcome = row["outcome"]
        if outcome not in DECISIVE_OUTCOMES or break_is("carry-without-store-check"):
            return True, ""
        unresolved = row["refused"].startswith("catching-test-unresolved")
        if outcome == "caught" and unresolved:
            if self.for_run and not break_is("refused-carried"):
                return False, row["refused"]
            coverers, _count, why = self.coverers_for(current["unit"])
            return (True, "") if coverers and coverers == row["coverage_key"] else (False, why or "coverers-changed")
        test_now, test_why, coverers_now, coverers_why = "", "", "", ""
        if outcome == "caught":
            test_now, test_why = self.test_key_for(row["caught_by"])
        if outcome == "missed":
            coverers_now, _count, coverers_why = self.coverers_for(current["unit"])
        current_view = dict(current, refused=[] if (not self.for_run or break_is("refused-carried")) else current["refused"])
        return self.rules.carry_decision(row, current_view, test_now or "", test_why, coverers_now or "", coverers_why)

    def state(self, item):
        """→ (ItemState, 为什么沿用不了, 定下状态的那一行)。真跑时拒算的条一律沿用不了（每趟现判）。"""
        current = self.key_of(item)
        if current["key"] is None:
            return ItemState(NOT_RUN, False), "refused:" + ",".join(current["refused"]), None
        if self.for_run and current["refused"] and not break_is("refused-carried"):
            return ItemState(NOT_RUN, False), "refused:" + ",".join(current["refused"]), None
        rows = self.by_key.get(current["key"], [])
        kept, reasons = [], []
        for row in rows:
            holds, why = self.row_holds(row, current)
            (kept if holds else reasons).append(row if holds else why)
        state = verdict_state([row["outcome"] for row in kept])
        deciding = next((row for row in reversed(kept) if row["outcome"] == state.state), None)
        reason = reasons[-1] if reasons else ("no-record" if not rows else "")
        if self.for_run and state.state in DECISIVE_OUTCOMES and not break_is("missed-carried") and current["key"] not in self.gpu_decided:
            # 记红、撤绿、下趟现判（文件头「判定行」末段）：本来沿用得了（有定论）的条，同一把键的定论里最后一个是没红的，红照键记着、前面的好结论撤掉，这一条不沿用、现判；
            # 看全部键相同的行，不看沿用判据认不认：认不认只定 --summary 的状态，撤绿不看覆盖者键变没变（没红行本身已不算数的，状态不是定论，走原来的路：只补第二段）
            decisive = [row["outcome"] for row in rows if row["outcome"] in DECISIVE_OUTCOMES]
            if decisive and decisive[-1] == "missed":
                return ItemState(NOT_RUN, state.passed_stage_one), "missed-rejudged", None
        return state, reason, deciding

    def changed_units(self, item, limit=5):
        """沿用不了时变了的是哪几个单元：同一处变异（位置索引相同）上一回判的那把键的组成清单与此刻的比，列出名字（文件头「库目录」compositions/）。"""
        current = self.key_of(item)
        previous = [row for row in self.by_locator.get(current["locator"], []) if row["key"] != current["key"]]
        if not previous:
            return ""
        old = self.library.read_composition(previous[-1]["key"])
        if old is None:
            return ""
        return composition_difference(old, current["composition"], limit)

    def refused_text(self, item, deciding=None):
        current = self.key_of(item)
        forms = list(current["refused"])
        if deciding is not None and deciding["refused"]:
            forms.append(deciding["refused"])
        return ",".join(forms)

    def coverage_jobs(self, stage):
        """覆盖那一步的各条（静态列出的）：{(二进制, 用例): 记录键或 None}。第一段逐条，记录键是那条测试的测试键；
        验证包逐个二进制（用例写 *），记录键取那个二进制里各条测试的测试键（有一条算不出就是 None，每趟现跑）。"""
        jobs = {}
        if stage == "one":
            for entry in self.runnable(STAGE_ONE_PACKAGES):
                key, _forms = self.rules.test_key(self.tree, entry["unit"], entry["target"], self.globals)
                jobs[(entry["binary"], entry["test"])] = hashlib.sha256(f"coverage-v1\none\n{key}\n".encode()).hexdigest() if key else None
            return jobs
        by_binary = {}
        for entry in self.runnable((VERIFICATION_PACKAGE,)):
            key, _forms = self.rules.test_key(self.tree, entry["unit"], entry["target"], self.globals)
            target = entry["target"].split(":", 2)
            label = "lib" if target[1] == "lib" else target[2]
            by_binary.setdefault(f"checker:{label}", []).append(key)
        for binary, keys in by_binary.items():
            jobs[(binary, "*")] = (hashlib.sha256(("coverage-v1\nchecker\n" + "".join(f"{key}\n" for key in sorted(keys))).encode()).hexdigest()
                                   if all(keys) else None)
        return jobs

    def selection(self, item, runner):
        """C2 / E1 / E6：这条变异每一步跑哪些测试。{"one": None 全部 / [(二进制, 用例)], "checker": None / [目标], "reason": …}。"""
        current = self.key_of(item)
        everything = self.rules.run_everything_reason(self.tree, current)
        if everything or break_is("coverage-selection-dropped") or "one" not in runner.coverage_available:
            return {"one": None, "checker": None, "reason": everything or "no-coverage"}
        unit = current["unit"]
        fingerprint = f"{unit.kind}:{unit.fingerprint}"
        spawning = self.rules.spawning_binaries(self.tree, STAGE_ONE_PACKAGES, runner.profile_counts)
        chosen = {"one": self.rules.select_tests(runner.coverage["one"], fingerprint, spawning), "reason": "coverage"}
        checker_spawning = {binary.split("::", 1)[1] if "::" in binary else "lib" for binary in self.rules.spawning_binaries(self.tree, (VERIFICATION_PACKAGE,))}
        if "two" not in runner.coverage_available or break_is("stage-two-not-by-binary"):
            chosen["checker"] = None
            return chosen
        chosen["checker"] = sorted({job[0][len("checker:"):] for job, covered in runner.coverage["checker"].items() if fingerprint in covered} | checker_spawning)
        return chosen

    def baseline_key(self, step, threads):
        """基线键（文件头「基线」）：→ (键, "") 或 (None, 拒算的那一句)。键 = 这一步真跑到的每一条判定（baseline_entries）各自单元闭包键的多重集
        （闭包键里有闭包单元的种类与去注释指纹、闭包真引到的 cfg 原子、依赖版本行、工具链那几行）、去掉线程数的命令、全局那几行（判法单元指纹除外）；
        不带步名、手写的版本号、包级与文件级指纹。在这一步的 cfg 与特性下不编的判定（entry-cfg-inactive）不算这一步跑到的、不进键。
        拒算（照跑、不沿用、不退回包级）：编译步（编得过与否、用时随包里每一个单元变，按单元闭包切不开）、认不出跑哪些判定的步、有一条判定的闭包拒算。
        弄坏开关 baseline-by-packages：照改前取这一步编、跑的那几个包（连同路径依赖）里全部单元的指纹（包级），带手写的 form 与步名。"""
        name, _stage, kind, command, _failed = step
        if break_is("baseline-by-packages"):
            return self.package_baseline_key(step, threads), ""
        if kind == "build":
            return None, "编译步：编得过与否、用时随这一步编的那几个包里每一个单元变，按单元闭包切不开"
        entries, release = self.baseline_entries(name)
        if entries is None:
            return None, f"认不出步 {name} 跑哪些判定"
        words = list(command)
        if "-j" in words and words.index("-j") + 1 < len(words) and words[words.index("-j") + 1] == str(threads):
            del words[words.index("-j"):words.index("-j") + 2]
        global_lines = [line for line in self.globals if not line.startswith("judging-units ")]
        keys = []
        for entry in entries:
            key, refused = self.release_entry_key(entry, global_lines) if release else self.rules.test_key(self.tree, entry["unit"], entry["target"], global_lines)
            if key is None and refused == ["entry-cfg-inactive"]:
                continue
            if key is None:
                return None, f"{entry['binary']} {entry['test']} 的单元闭包拒算（{','.join(refused) or '闭包算不出'}）"
            keys.append(key)
        lines = [f"command\t{shlex.join(words)}"] + global_lines + [f"entry\t{key}" for key in sorted(keys)]
        return hashlib.sha256("".join(line + "\n" for line in lines).encode("utf-8", "surrogateescape")).hexdigest(), ""

    def baseline_entry_plan(self, step, threads):
        """按条记基线（文件头「基线」）：基线键因为有判定的闭包拒算而整步算不出的测试步，拆成各自按键沿用的几份记录与每趟照跑的那几条。
        → None（不按条：编译步、认不出的步、不在 PER_ENTRY_TEST_STEPS 里的步，弄坏开关 baseline-refused-step-whole）或
        {"style": nextest / libtest, "records": [记录], "refused": [(二进制, 用例)], "whole": [整个照跑的二进制]}。
        nextest 那两步：一条测试一份记录（kind test，键 = 去掉线程数的命令、全局那几行、它自己的单元闭包键）；有条目级宏调用的二进制另一份
        「宏调用展开的那几条」（kind rest，键 = 那个二进制里宏调用的闭包键多重集），宏调用的闭包拒算的二进制整个照跑；拒算的测试每趟照跑。
        验证包（cargo test）：一个二进制一份记录（kind binary，键 = 它里面闭包算得出的判定与宏调用的闭包键多重集；
        有拒算的，这一份记的是 --exact --skip 掉它们的那一趟），拒算的那几条另起 --exact 照跑；宏调用拒算的二进制整个照跑。
        记录里的二进制、名字只当索引，不进键（rest 另按二进制找，改名只会重跑、不会错沿用）。"""
        name, _stage, kind, command, _failed = step
        if kind != "test" or name not in PER_ENTRY_TEST_STEPS or break_is("baseline-refused-step-whole"):
            return None
        entries, release = self.baseline_entries(name)
        if entries is None:
            return None
        words = list(command)
        if "-j" in words and words.index("-j") + 1 < len(words) and words[words.index("-j") + 1] == str(threads):
            del words[words.index("-j"):words.index("-j") + 2]
        global_lines = [line for line in self.globals if not line.startswith("judging-units ")]
        head = [f"command\t{shlex.join(words)}"] + global_lines
        style = "nextest" if "nextest" in command else "libtest"

        def digest(lines):
            return hashlib.sha256("".join(line + "\n" for line in lines).encode("utf-8", "surrogateescape")).hexdigest()

        groups = {}
        for entry in entries:
            target = self.tree.workspace.targets.get(entry["target"]) if entry["target"] else None
            group = groups.setdefault(entry["binary"], {"target": target, "tests": [], "refused": [], "macros": [], "macro_refused": False})
            key, refused = self.release_entry_key(entry, global_lines) if release else self.rules.test_key(self.tree, entry["unit"], entry["target"], global_lines)
            is_macro = entry["unit"].kind == "item_macro"
            if key is None and refused == ["entry-cfg-inactive"]:
                continue
            if is_macro:
                group["macros"].append(key)
                group["macro_refused"] = group["macro_refused"] or key is None
            elif key is None:
                group["refused"].append(entry["test"])
            else:
                group["tests"].append((entry["test"], key))
        plan = {"style": style, "records": [], "refused": [], "whole": []}
        for binary, group in sorted(groups.items()):
            target = group["target"]
            selector = (["--lib"] if target is None or target.kind == "lib" else [f"--{target.kind}", target.name])
            plan["refused"] += [(binary, test) for test in group["refused"]]
            if style == "nextest":
                plan["records"] += [{"kind": "test", "binary": binary, "test": test, "key": digest(head + [f"entry\t{key}"])}
                                    for test, key in group["tests"]]
                if group["macro_refused"]:
                    plan["whole"].append({"binary": binary, "selector": selector})
                elif group["macros"]:
                    plan["records"].append({"kind": "rest", "binary": binary, "test": "",
                                            "key": digest(head + ["rest"] + [f"macro\t{key}" for key in sorted(group["macros"])])})
                continue
            if group["macro_refused"]:
                plan["whole"].append({"binary": binary, "selector": selector})
                continue
            plan["records"].append({"kind": "binary", "binary": binary, "test": "*", "selector": selector, "refused": sorted(group["refused"]),
                                    "key": digest(head + [f"entry\t{key}" for key in sorted(key for _test, key in group["tests"])]
                                                  + [f"macro\t{key}" for key in sorted(group["macros"])])})
        return plan

    def baseline_entries(self, name):
        """一步真跑到的判定：→ (判定列表, 按不按 --release 的 cfg 收闭包)；认不出的步交 (None, False)。测试函数照 mutation-prune.py 的 test_entries 静态列，
        另加这几个目标里的条目级宏调用（宏展开出来的测试静态列不出：把宏调用连同它的闭包算进去）；文档测试照 doctest_entries。"""
        if name == "one-test":
            return self.runnable(STAGE_ONE_PACKAGES) + self.macro_entries(STAGE_ONE_PACKAGES), False
        if name == "two-test-doc":
            return self.rules.doctest_entries(self.tree, MUTATED_PACKAGES), False
        if name == "two-test-checker-tier":
            return self.runnable((VERIFICATION_PACKAGE,)) + self.macro_entries((VERIFICATION_PACKAGE,)), True
        return None, False

    def macro_entries(self, packages):
        """这几个包的 lib、bin、tests 目标里的条目级宏调用（item_macro 单元），照判定的写法交（二进制、名字只当报错的索引）。"""
        found = []
        for target in self.tree.workspace.targets.values():
            if target.crate.package not in packages or target.kind not in ("lib", "test", "bin"):
                continue
            for path in sorted(target.files):
                found += [{"binary": self.rules.target_binary(target), "test": unit.name, "unit": unit, "target": target.id}
                          for unit in self.tree.code_units(path) if unit.kind == "item_macro"]
        return found

    def release_entry_key(self, entry, global_lines):
        """带 --release 的那一步（验证包）的一条判定：cfg 去掉 debug_assertions 收闭包（试法照 mutation-prune.py 的 UnitTree.closure：
        先照默认、不编再加 test），键照 UnitTree.closure_key 的组法（闭包键、依赖版本行、工具链那几行与 edition）。→ (键或 None, 拒算形态)。"""
        library, unit, target_id = self.tree.library, entry["unit"], entry["target"]
        cache_key = (id(unit), target_id)
        if cache_key not in self.release_closures:
            release_cfg = [item for item in library.DEFAULT_RUST_CFG if item != "debug_assertions"]
            found = (None, ["entry-cfg-inactive"])
            for extra in ((), ("test",)):
                try:
                    found = (library.rust_closure_of(self.tree.workspace, unit, target_id, release_cfg + list(extra), no_default_cfg=True), [])
                    break
                except library.UsageError:
                    found = (None, ["entry-cfg-inactive"])
            self.release_closures[cache_key] = found
        closure, refusals = self.release_closures[cache_key]
        if closure is None:
            return None, refusals
        dependencies, dependency_refusals = self.tree.dependency_lines(closure)
        key, closure_refusals = library.closure_key(closure, toolchain=list(global_lines) + self.tree.edition_lines(closure), dependencies=dependencies,
                                                    allow_refused=True)
        forms = sorted({refusal.form for refusal in closure_refusals}) + dependency_refusals
        return (None, forms) if forms else (key, [])

    def package_baseline_key(self, step, threads):
        """改前的基线键（只给弄坏开关 baseline-by-packages）：这一步编、跑的那几个包（连同路径依赖）里全部单元的指纹多重集、去掉线程数的命令、
        全局那几行（判法单元指纹除外），带手写的 form 与步名。"""
        name, _stage, _kind, command, _failed = step
        packages = set(MUTATED_PACKAGES if name == "two-test-doc" else (VERIFICATION_PACKAGE,) if "checker-tier" in name else STAGE_ONE_PACKAGES)
        crates = {crate.directory: crate for crate in self.tree.workspace.crates if crate.package in packages}
        pending = list(crates.values())
        while pending:
            crate = pending.pop()
            for section in ("normal", "dev", "build"):
                for dependency in crate.dependencies[section]:
                    if dependency.directory and dependency.directory not in crates and dependency.directory in self.tree.workspace.crate_by_directory:
                        crates[dependency.directory] = self.tree.workspace.crate_by_directory[dependency.directory]
                        pending.append(crates[dependency.directory])
        words = list(command)
        if "-j" in words and words.index("-j") + 1 < len(words) and words[words.index("-j") + 1] == str(threads):
            del words[words.index("-j"):words.index("-j") + 2]
        units = sorted(f"{unit.kind}:{unit.fingerprint}" for path in self.package_files({crate.package for crate in crates.values()})
                       for unit in self.tree.code_units(path))
        lines = ["form\tbroad-mutation-baseline-v1", f"step\t{name}", f"command\t{shlex.join(words)}"]
        lines += [line for line in self.globals if not line.startswith("judging-units ")] + [f"unit\t{unit}" for unit in units]
        return hashlib.sha256("".join(line + "\n" for line in lines).encode("utf-8", "surrogateescape")).hexdigest()


def composition_difference(old, new, limit=5):
    """两份组成清单之间变了的单元：→「改了 <限定名>（文件）、新进 …、出了 …」；只比单元行（种类、限定名、文件:起止、指纹）。"""
    def units(lines):
        found = {}
        for line in lines:
            fields = line.split("\t")
            if len(fields) == 4 and ":" in fields[2]:
                found.setdefault((fields[0], fields[1], fields[2].rsplit(":", 1)[0]), []).append(fields[3])
        return found
    before, after = units(old), units(new)
    changed = [f"改了 {name}（{path}）" for kind, name, path in sorted(set(before) & set(after)) if sorted(before[(kind, name, path)]) != sorted(after[(kind, name, path)])]
    added = [f"新进 {name}（{path}）" for kind, name, path in sorted(set(after) - set(before))]
    dropped = [f"出了 {name}（{path}）" for kind, name, path in sorted(set(before) - set(after))]
    others = [] if changed or added or dropped else [f"{line.split(chr(9), 1)[0]} 那几行" for line in sorted(set(old) ^ set(new)) if not line.startswith(("unit\t",))][:2]
    found = changed + added + dropped + others
    return "、".join(found[:limit]) + (f" 等 {len(found)} 处" if len(found) > limit else "")


# ── 清单的区间与 diff ───────────────────────────────────────────────────────────────────────────────────

def offset_of(text_lines, line, column):
    """1 起的行、列（按字符数）在整份文本里的下标；text_lines 是 text.split('\\n')。"""
    return sum(len(text_line) + 1 for text_line in text_lines[:line - 1]) + column - 1


def replace_span(text, start, end, replacement):
    """把 [start, end)（各是 (行, 列)，1 起、列按字符数、end 不含）换成 replacement。"""
    lines = text.split("\n")
    start_offset, end_offset = offset_of(lines, *start), offset_of(lines, *end)
    if not 0 <= start_offset <= end_offset <= len(text):
        raise ValueError(f"区间 {start}–{end} 越出文件")
    return text[:start_offset] + replacement + text[end_offset:]


def apply_unified_diff(original_text, diff_text):
    """照一份 unified diff 改 original_text，交回改后的文本；上下文或删掉的行与原文对不上抛 ValueError。"""
    original, result, position = original_text.split("\n"), [], 0
    lines = diff_text.split("\n")
    index = 0
    while index < len(lines) and not lines[index].startswith("@@"):
        index += 1
    while index < len(lines):
        header = HUNK_HEADER.match(lines[index])
        if header is None:
            raise ValueError(f"认不出的 hunk 头：{lines[index]!r}")
        old_start, old_count = int(header.group(1)), int(header.group(2) if header.group(2) is not None else "1")
        hunk_start = old_start - 1 if old_count > 0 else old_start
        if hunk_start < position:
            raise ValueError("hunk 次序倒了")
        result.extend(original[position:hunk_start])
        position = hunk_start
        index += 1
        while index < len(lines) and not lines[index].startswith("@@"):
            line = lines[index]
            index += 1
            if line.startswith("\\"):
                continue
            if line == "" and index == len(lines):
                break
            tag, body = (line[:1], line[1:]) if line else (" ", "")
            if tag in (" ", "-"):
                if position >= len(original) or original[position] != body:
                    raise ValueError(f"diff 的第 {position + 1} 行与原文对不上")
                if tag == " ":
                    result.append(body)
                position += 1
            elif tag == "+":
                result.append(body)
            else:
                raise ValueError(f"认不出的 diff 行：{line!r}")
    result.extend(original[position:])
    return "\n".join(result)


def raw_mutants_of_package(library, package):
    """一个包的原始清单（cargo mutants --list --json --diff 的那个数组，每项补上 name）。"""
    environment = dict(os.environ, CARGO_TARGET_DIR=library.target)
    fake_driver = test_variable("BROAD_MUTATION_LIST_DRIVER")
    if fake_driver:
        listed = subprocess.run([fake_driver, package], capture_output=True, text=True, cwd=library.tree, stdin=subprocess.DEVNULL)
        if listed.returncode != 0:
            refuse(f"假的清单驱动退 {listed.returncode}：{listed.stderr.strip()}", "看 BROAD_MUTATION_LIST_DRIVER 指的脚本")
        return json.loads(listed.stdout)
    if shutil.which("cargo-mutants") is None:
        refuse("PATH 上没有 cargo-mutants，生成不了清单", "cargo install cargo-mutants 之后重跑同一条命令")
    base = ["cargo", "mutants", "--list", "--package", package]
    as_json = subprocess.run(base + ["--json", "--diff"], capture_output=True, text=True, cwd=library.tree, env=environment, stdin=subprocess.DEVNULL)
    if as_json.returncode != 0:
        refuse(f"cargo mutants --list --json --diff --package {package} 退 {as_json.returncode}：{as_json.stderr.strip()[-600:]}",
               f"在 {library.tree} 里手跑这一条看报错；树里的 .cargo/mutants.toml 是换进来的那一份")
    mutants = json.loads(as_json.stdout)
    if all(isinstance(mutant.get("name"), str) for mutant in mutants):
        return mutants
    as_text = subprocess.run(base, capture_output=True, text=True, cwd=library.tree, env=environment, stdin=subprocess.DEVNULL)
    names = [line for line in as_text.stdout.split("\n") if line.strip()]
    if as_text.returncode != 0 or len(names) != len(mutants):
        refuse(f"cargo mutants --list --package {package} 的文本清单（{len(names)} 行，退 {as_text.returncode}）与 JSON 清单（{len(mutants)} 项）条数对不上，变异名配不上",
               "看这一版 cargo-mutants 的 --list 输出有没有多出不是变异名的行；对不上就不生成清单，不拿猜的名字往下跑")
    for mutant, name in zip(mutants, names):
        mutant["name"] = name
    return mutants


# ── 登记的崩溃枚举用例与放量分项：第二段 验证包与 GPU 那一趟经 --skip 排掉 ─────────────────────────────────────────────

def registered_functions(root):
    """第二段 验证包与 GPU 那一趟用 --skip 排掉的 {(测试目标, 用例函数)}：登记成 crash-case: 的崩溃枚举用例与放量分项，
    就是门禁按条那一趟与入口按条那一趟用 --exclude-registered 排掉的同一张排除单（scripts/gate/admission.py 的 registered_test_functions，读法只在那里一份）；
    它们另走按块那一路（门禁 crash-point-replay），这里不跑。读不了退 1。
    只供测试的 BROAD_MUTATION_REGISTERED=<目标:用例函数,…> 换掉读法（自证的假树里没有登记表）。"""
    given = test_variable("BROAD_MUTATION_REGISTERED")
    if given is not None:
        return {tuple(item.split(":", 1)) for item in given.split(",") if ":" in item}
    if not ADMISSION:
        specification = importlib.util.spec_from_file_location("admission_for_broad_mutation", os.path.join(SCRIPT_DIRECTORY, "..", "gate", "admission.py"))
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        ADMISSION.append(module)
    try:
        return set(ADMISSION[0].registered_test_functions(root))
    except (ADMISSION[0].RegistrationError, OSError) as error:
        refuse(f"读不了登记的崩溃枚举用例与放量分项（{error}）：第二段不知道该用 --skip 排掉哪几条",
               f"看 {root} 的 .claude/gate.d/stage-inputs.tsv 与 scripts/crash/crash-amplification.sh 还在不在、成不成形（门禁 crash-point-replay 读同一份）", EXIT_INCONSISTENT)


def registered_exclusions(rules, tree, registered):
    """上面那一张单在这棵树的 验证包里对上的静态测试：[(二进制 id, 用例名, 理由)]；覆盖者键、基线键与测试清单都不算它们（它们不在这里跑）。"""
    found = []
    for entry in rules.test_entries(tree, (VERIFICATION_PACKAGE,)):
        if (entry["binary"].rsplit("::", 1)[-1], entry["test"].rsplit("::", 1)[-1]) in registered:
            found.append((entry["binary"], entry["test"], REGISTERED_REASON))
    return found


# ── 每一步命令 ─────────────────────────────────────────────────────────────────────────────────────────────

def package_arguments(packages):
    return [word for package in packages for word in ("-p", package)]


def step_table(arguments):
    """每一步：(步名, 哪一段, build / test, 命令, 测试没过的退出码)。
    第二段只补第一段没跑到的：文档测试（被变异的两个包）、验证包里的全部测试函数（登记的崩溃枚举用例与放量分项另走按块那一路，
    起命令时经 --skip 排掉：Runner.command_with_skips）。第一段跑过的那三个包的用例第二段不再跑。"""
    harness = package_arguments(STAGE_ONE_PACKAGES)
    threads = str(arguments.threads)
    sync_free = ["eatmydata"]
    stage_two_prefix = sync_free if arguments.stage_two_eatmydata else []
    checker_tier = ["--release", "-p", VERIFICATION_PACKAGE, "--lib", "--tests"]
    harness_build = ["cargo", "nextest", "run", "--locked", *harness, "--no-run"]
    harness_test = sync_free + ["cargo", "nextest", "run", "--locked", *harness, "-j", threads]
    stage_one = [
        ("one-build", "one", "build", harness_build, None),
        ("one-test", "one", "test", harness_test, NEXTEST_TEST_FAILED_EXIT),
    ]
    if break_is("stage-two-full-cargo-test"):
        return stage_one + [
            ("two-build-debug", "two", "build", ["cargo", "test", "--locked", *harness, "--no-run"], None),
            ("two-test-debug", "two", "test", stage_two_prefix + ["cargo", "test", "--locked", *harness], CARGO_FAILED_EXIT),
            ("two-build-checker-tier", "two", "build", ["cargo", "test", "--locked", *checker_tier, "--no-run"], None),
            ("two-test-checker-tier", "two", "test", stage_two_prefix + ["cargo", "test", "--locked", *checker_tier], CARGO_FAILED_EXIT),
        ]
    return stage_one + [
        ("two-test-doc", "two", "test", stage_two_prefix + ["cargo", "test", "--locked", "--doc", *package_arguments(MUTATED_PACKAGES)], CARGO_FAILED_EXIT),
        ("two-build-checker-tier", "two", "build", ["cargo", "test", "--locked", *checker_tier, "--no-run"], None),
        ("two-test-checker-tier", "two", "test", stage_two_prefix + ["cargo", "test", "--locked", *checker_tier], CARGO_FAILED_EXIT),
    ]


def progress_line(sequence, step_name, elapsed, log_path):
    """一步跑着时的一行进度（文件头「输出」）：哪一条、哪一步、跑了多久，与这一步此刻的日志里数得出的已编 crate、已过与没过的用例。"""
    label = "baseline" if sequence == BASELINE_SEQUENCE else sequence
    compiled = passed = failed = 0
    try:
        with open(log_path, encoding="utf-8", errors="replace") as handle:
            for line in handle:
                line = line.rstrip("\n")
                if COMPILING_LINE.match(line):
                    compiled += 1
                elif NEXTEST_PASS_LINE.match(line) or libtest_outcome(line) == "ok":
                    passed += 1
                elif NEXTEST_FAILURE_LINE.match(line) or libtest_outcome(line) == "FAILED":
                    failed += 1
    except OSError:
        pass
    return (f"BROAD_MUTATION_PROGRESS sequence={label} step={step_name} elapsed_seconds={elapsed:.0f} "
            f"compiled_crates={compiled} tests_passed={passed} tests_failed={failed}")


class Runner:
    def __init__(self, library, arguments):
        self.library, self.arguments = library, arguments
        # 登记的崩溃枚举用例与放量分项（上面「登记的崩溃枚举用例与放量分项」）：只跑第一段的趟用不着，读不了也不挡它
        try:
            self.registered = registered_functions(arguments.source_root)
        except Refusal:
            if not stage_two_skipped():
                raise
            self.registered = set()
        self.skip_names = sorted({function for _target, function in self.registered})
        self.steps = step_table(arguments)
        self.fake_driver = test_variable("BROAD_MUTATION_STEP_DRIVER") == "1"
        self.selftest_work = test_variable(SELFTEST_WORK_VARIABLE) or ""   # 假一步在它下面（文件头「只供测试」）
        self.stages_with_fresh_baseline = set()
        self.judgement = None
        self.coverage = {"one": {}, "checker": {}}
        self.coverage_available = set()
        self.profile_counts = {}
        self.baseline_keys = {}
        self.baseline_refusals = {}    # 步名 → 基线键拒算的那一句（Judgement.baseline_key）
        self.run_token = secrets.token_hex(16)   # 拒算的基线这一趟独有的那一把键从它算：记下的基线行以后哪一趟都对不上、不沿用

    def baseline_key_of(self, step_name):
        """这一步此刻的基线键（Judgement.baseline_key：这一步真跑到的判定的单元闭包）。拒算的（文件头「基线」）交这一趟独有的一把（run_token 算的）：
        这一趟照跑、记的基线行在这一趟里照常对得上，以后哪一趟都对不上、不沿用。"""
        if step_name not in self.baseline_keys:
            step = next((step for step in self.steps if step[0] == step_name), (step_name, "", "test", [], None))   # 覆盖那一步不在步表里
            key, refused = self.judgement.baseline_key(step, self.arguments.threads)
            if key is None:
                self.baseline_refusals[step_name] = refused
                key = hashlib.sha256(f"refused-baseline\0{step_name}\0{self.run_token}".encode()).hexdigest()
            self.baseline_keys[step_name] = key
        return self.baseline_keys[step_name]

    def command_for(self, step, selection):
        """这一条这一步的命令与「跑了哪些」的一句：按覆盖挑出来的测试（C2）、第二段按二进制（E6）；交 (None, …) 是这一步没有要跑的。"""
        name, command = step[0], list(step[3])
        selection = selection or {}
        if name == "one-test" and selection.get("one") is not None:
            if not selection["one"]:
                return None, "none"
            filters = prune_rules().nextest_filters(selection["one"])
            return command + [word for text in filters for word in ("-E", text)], f"{len(selection['one'])} tests"
        if name == "two-test-checker-tier" and selection.get("checker") is not None:
            if not selection["checker"]:
                return None, "none"
            command = [word for word in command if word not in ("--lib", "--tests")]
            for target in selection["checker"]:
                command += ["--lib"] if target == "lib" else ["--test", target]
            return command, ",".join(selection["checker"])
        if name == "two-test-doc" and break_is("doc-tests-skipped"):
            return None, "none"
        return command, "all"

    def ensure_coverage(self, stages):
        """C2：没改过的树上的覆盖（第一段逐条测试、验证包逐个二进制），按记录键（那条测试的测试键）在判定库 coverage/ 里照用，
        对不上的那几条插桩跑（--coverage-step）；跑红拒（退 6）。之后 self.coverage[段] = {(二进制, 用例): {种类:指纹}}。"""
        judgement, rules = self.judgement, self.judgement.rules
        records = {}
        for entry in self.library.read_rows(COVERAGE_TABLE, COVERAGE_COLUMNS).rows:
            try:
                records[entry["record_key"]] = (set(json.loads(entry["covered"])), int(entry["profiles"] or 1))
            except ValueError:
                continue
        for stage in stages:
            jobs = judgement.coverage_jobs(stage)
            known = {job: record_key for job, record_key in jobs.items() if record_key and record_key in records}
            name = f"coverage-{stage}"
            out = os.path.join(self.library.scratch + "-coverage", f"{name}.json")
            os.makedirs(os.path.dirname(out), exist_ok=True)
            if os.path.exists(out):
                os.remove(out)
            llvm = llvm_bin(self.arguments)
            if not self.fake_driver and not all(os.path.isfile(os.path.join(llvm, tool)) for tool in ("llvm-profdata", "llvm-cov")):
                refuse(f"按覆盖挑测试要 llvm-profdata 与 llvm-cov，{llvm or 'rustc 的 sysroot'} 下没有",
                       "装 rustup 组件：rustup component add llvm-tools（与编译器同一份 LLVM，profraw 格式对得上）；装在别处就加 --llvm-bin <目录>")
            specification = os.path.join(os.path.dirname(out), f"{name}.spec.json")
            write_atomically(specification, json.dumps({"kind": stage, "tree": self.library.tree, "target": self.library.target_coverage,
                                                        "scratch": self.library.scratch, "llvm_bin": llvm, "workers": self.arguments.threads, "out": out,
                                                        "skips": self.skip_names, "known": [list(job) for job in sorted(known)]}).encode())
            say(f"  · 覆盖 {name}：在没改过的树上插桩列一遍测试，记录键对得上的 {len(known)} 条照用、其余现跑")
            step = (name, stage, "test", ["python3", SELF_PATH, "--coverage-step", specification], None)
            exit_code, seconds = self.run_step(step, BASELINE_SEQUENCE, extra={"BROAD_MUTATION_STEP_SPEC": specification})
            if exit_code != 0 or not os.path.isfile(out):
                refuse(f"覆盖 {name} 在没改过的树上退 {exit_code}：拿不到「单元 → 执行到它的测试」，一条变异都不跑",
                       f"看日志 {self.log_path(BASELINE_SEQUENCE, name)}（插桩下跑红的测试逐条列在里面）；是环境就修环境重跑同一条命令")
            with open(out, encoding="utf-8") as handle:
                result = json.load(handle)
            if result is None:
                say(f"  · 覆盖 {name}：这一趟拿不到覆盖，每条变异跑全部测试")
                continue
            self.coverage_available.add(stage)
            for job in result.get("tests", []):
                job = tuple(job)
                if job in known:
                    covered, profiles = records[known[job]]
                    self.coverage[self.coverage_part(stage, job[0])][job] = covered
                    self.profile_counts[job] = profiles
            new_rows = []
            for record in result.get("jobs", []):
                job = (record["binary"], record["test"])
                covered = set(rules.fingerprints_of(judgement.tree, record.get("covered", [])))
                self.coverage[self.coverage_part(stage, job[0])][job] = covered
                self.profile_counts[job] = record.get("profiles", 1)
                record_key = jobs.get(job)
                if record_key:
                    new_rows.append([record_key, stage, job[0], job[1], json.dumps(sorted(covered)), record.get("profiles", 1), today_text()])
            if new_rows:
                self.library.append_rows(COVERAGE_TABLE, new_rows)
            say(f"  · 覆盖 {name} 好了，{seconds:.1f} 秒：照用 {len(known)} 条、现跑 {len(result.get('jobs', []))} 条（记进库的 {len(new_rows)} 条）")

    @staticmethod
    def coverage_part(stage, binary):
        return "one" if stage == "one" else "checker"

    def step_named(self, step_name):
        return next(step for step in self.steps if step[0] == step_name)

    def log_path(self, sequence, step_name):
        prefix = "baseline" if sequence == BASELINE_SEQUENCE else str(sequence)
        return os.path.join(self.library.logs, f"{prefix}.{step_name}.log")

    def stop_step(self, child):
        """驱动被 TERM / INT 停：在途这一步连同它的进程组一起停，不留着它跑完（文件头「中断续接」）。
        一步起在自己新开的进程组里（run_step 的 start_new_session），组号就是 child 的进程号：cargo 与它起的测试进程都在这一组里，
        只给 child 发 TERM 的话 cargo 退了、测试进程照跑。发信号只点自己起的那一组：先 TERM，到 --stop-grace 秒还有没退的就 KILL。"""
        grace = self.arguments.stop_grace
        if break_is("term-leaves-step-running"):
            child.terminate()   # 弄坏开关：只点自己起的这一个进程，组里别的进程照跑（改前的样子）
            try:
                child.wait(timeout=grace)
            except subprocess.TimeoutExpired:
                pass
            return
        self.signal_step_group(child, signal.SIGTERM)
        try:
            child.wait(timeout=grace)
        except subprocess.TimeoutExpired:
            pass
        self.signal_step_group(child, signal.SIGKILL)    # 组里还活着的（child 退了而它起的进程没退的也算）一并停掉
        child.wait()

    @staticmethod
    def signal_step_group(child, number):
        """给自己起的这一步那一个进程组发信号（组号是 child 的进程号）；组里一个进程都不剩时不报错。"""
        try:
            os.killpg(child.pid, number)
        except ProcessLookupError:
            pass

    def progress_interval(self):
        fake = test_variable("BROAD_MUTATION_PROGRESS_INTERVAL")
        return float(fake) if fake else PROGRESS_INTERVAL_SECONDS

    def wait_reporting_progress(self, child, step_name, sequence, started):
        """等这一步退出，交回它的退出码；跑着时每过一个进度间隔往 stderr 报一行 BROAD_MUTATION_PROGRESS（progress_line）。"""
        interval = self.progress_interval()
        next_report = started + interval
        while True:
            if break_is("progress-never"):
                return child.wait()
            try:
                return child.wait(timeout=max(0.0, next_report - time.monotonic()))
            except subprocess.TimeoutExpired:
                pass
            now = time.monotonic()
            note(progress_line(sequence, step_name, now - started, self.log_path(sequence, step_name)))
            while next_report <= now:
                next_report += interval

    def baseline_rows_of(self, step_name):
        """基线表 baselines/ 里这把基线键、这个线程数下这一步的行：[(写入的次序号, {列名: 值})]。弄坏开关 baseline-any-threads 下不看线程数（改前的取法）。"""
        threads = str(self.arguments.threads)
        baseline_key = self.baseline_key_of(step_name)
        return [(number, row) for number, row in enumerate(self.library.baseline_rows(), start=1)
                if row["baseline_key"] == baseline_key and row["step"] == step_name and (row["threads"] == threads or break_is("baseline-any-threads"))]

    def command_with_skips(self, step_name, command):
        """第二段 验证包那一步与 GPU 那一趟的测试步：登记的崩溃枚举用例与放量分项经 --skip 排掉（接在测试二进制的参数后面：命令里已有 `--` 的接在它后面，没有的补一个）；
        别的步原样。"""
        if step_name not in ("two-test-checker-tier", GPU_TEST_STEP) or not self.skip_names:
            return command
        tail = [word for name in self.skip_names for word in ("--skip", name)]
        return list(command) + (tail if "--" in command else ["--"] + tail)

    def run_step(self, step, sequence, command=None, selection_text="all", extra=None):
        """跑一步：交 (退出码, 秒数)。输出落进这一条这一步的日志。command 不给就用步表里的。
        不经线程包装、内存包装与清场包装，不设内存上限、不限时（文件头「内存与限时」）：命令直接起，起在自己新开的进程组里（stop_step 照组停）。"""
        step_name, _stage, _kind, table_command, _test_failed_exit = step
        command = self.command_with_skips(step_name, table_command if command is None else command)
        remove_tree(self.library.scratch)
        os.makedirs(self.library.scratch)
        os.makedirs(self.library.target, exist_ok=True)
        os.makedirs(self.library.logs, exist_ok=True)
        environment = dict(os.environ, CARGO_TARGET_DIR=self.library.target, TMPDIR=self.library.scratch, CARGO_TERM_COLOR="never", **(extra or {}))
        # 调用方环境里的多机配置不往下传：只有真跑测试的步照下一句设（编译步不读它，测试步收到的只能是这里设的）
        environment.pop("SINGLEFS_MULTI_HOST_CONFIG", None)
        # 真跑测试的步把多机配置交给测试（文件头「多机配置交给测试」）：专用树里它自己找不到；弄坏开关 test-steps-configuration-unpassed 下不设
        if step[2] == "test" and not break_is("test-steps-configuration-unpassed"):
            environment["SINGLEFS_MULTI_HOST_CONFIG"] = self.arguments.multi_host_configuration
        if self.fake_driver:
            environment["BROAD_MUTATION_STEP_SELECTION"] = selection_text
            argv = [os.path.join(self.selftest_work, SELFTEST_STEP_DRIVER_NAME), step_name, str(sequence), self.library.tree]
        else:
            argv = list(command)
        started = time.monotonic()
        with open(self.log_path(sequence, step_name), "wb") as log:
            child = subprocess.Popen(argv, cwd=self.library.tree, env=environment, stdin=subprocess.DEVNULL, stdout=log,
                                     stderr=subprocess.STDOUT, pass_fds=(self.library.lock_descriptor,), start_new_session=True)
            try:
                exit_code = self.wait_reporting_progress(child, step_name, sequence, started)
            except Stopped:
                self.stop_step(child)
                raise
        return exit_code, time.monotonic() - started

    def ensure_baseline(self, stage):
        """这一段的每一步在这把基线键、这个线程数下、没改过的树上都跑绿过；没有就现跑。
        基线表 baselines/ 只追加，一行记基线键、步名、结局、时刻、线程数（文件头「基线」）。"""
        ran_every_step = True
        for step in self.steps:
            step_name, step_stage = step[0], step[1]
            if step_stage != stage:
                continue
            rows = self.baseline_rows_of(step_name)
            row = rows[-1][1] if rows else None
            if row is not None and row["outcome"] == "ok":
                ran_every_step = False
                continue
            plan = self.judgement.baseline_entry_plan(step, self.arguments.threads) if step_name in self.baseline_refusals else None
            if plan is not None:
                self.ensure_entry_baseline(step, stage, plan)
                continue
            if step_name in self.baseline_refusals:
                say(f"  · 基线 {step_name}：基线键拒算（{self.baseline_refusals[step_name]}），在没改过的树上照跑、不沿用")
            else:
                say(f"  · 基线 {step_name}：在没改过的树上跑（这把基线键下、这个线程数下还没有它的绿记录）")
            command = list(step[3]) + (["--no-fail-fast"] if step_name == "two-test-checker-tier" else [])
            exit_code, seconds = self.run_step(step, BASELINE_SEQUENCE, command=command)
            outcome = "ok" if exit_code == 0 else f"exit-{exit_code}"
            self.library.append_rows(BASELINES_TABLE, [[self.baseline_key_of(step_name), step_name, outcome, now_text(), self.arguments.threads]])
            if exit_code != 0:
                self.refuse_red_baseline(step_name, stage, exit_code)
            say(f"  · 基线 {step_name} 绿，{seconds:.1f} 秒")
        if ran_every_step:
            self.stages_with_fresh_baseline.add(stage)

    def refuse_red_baseline(self, step_name, stage, exit_code, part_text=""):
        """基线那一步在没改过的树上退了非 0（文件头「基线」：基线红了一律退 6）：第二段的另点名红的测试与已知的那一条。part_text 是按条记基线时照跑的是哪一部分。"""
        log_path = self.log_path(BASELINE_SEQUENCE, step_name)
        if stage == "two" and not break_is("stage-two-baseline-red-generic"):
            failing = []
            try:
                with open(log_path, encoding="utf-8", errors="replace") as handle:
                    for line in handle:
                        name = libtest_failed_name(line.rstrip("\n")) or nextest_failed_name(line.rstrip("\n"))
                        if name and name not in failing:
                            failing.append(name)
            except OSError:
                pass
            refuse(f"第二段基线 {step_name}{part_text} 在没改过的树上退 {exit_code}，红的测试：{'、'.join(failing[:8]) or UNRECOGNISED_TEST}；第二段一条结论都给不出",
                   "已知的一条：crates/singlefs-verification/src/crash_identity.rs 里读 .claude/gate.d/stage-inputs.tsv（导出树之外）的那条 lib 测试，"
                   "要实现员改成不读导出树之外的文件（或改门禁 crates-mutation-replay 登记的拷贝范围把登记表并进导出树），修好之后重跑同一条命令；"
                   f"第二段基线不绿，后面每一条第二段的抓到都不作数，修好之前这一道过不去；全文看日志 {log_path}",
                   EXIT_BASELINE_RED)
        refuse(f"基线 {step_name}{part_text} 在没改过的树上退 {exit_code}：基线不绿，后面每一条「抓到」都不作数，一条变异都不跑",
               f"看日志 {log_path}；代码本来就红就先修代码再重跑，是环境（缺工具、盘满）就修环境，重跑同一条命令",
               EXIT_BASELINE_RED)

    def entry_baseline_rows(self, step_name):
        """按条记基线的表 baseline-entries/ 里这一步、这个线程数下跑绿过的记录：{(记录键, kind, rest 的二进制或空)}。段数不对的行不认，那一份照跑。"""
        threads = str(self.arguments.threads)
        found = set()
        for row in self.library.read_rows(BASELINE_ENTRIES_TABLE, BASELINE_ENTRY_COLUMNS).rows:
            if row["step"] != step_name or (row["threads"] != threads and not break_is("baseline-any-threads")):
                continue
            found.add((row["record_key"], row["kind"], row["binary"] if row["kind"] == "rest" else ""))
        return found

    def ensure_entry_baseline(self, step, stage, plan):
        """按条记基线（文件头「基线」）：基线键拒算的测试步，Judgement.baseline_entry_plan 拆出来的每份记录照（记录键、kind、线程数）在 baseline-entries/
        里找，找得到的沿用（不再跑），找不到的与拒算的那几条在没改过的树上照跑（nextest 按 -E 只点它们；验证包按二进制起，
        有拒算的二进制另起 --exact --skip 掉它们的一趟与只跑它们的一趟），照跑的那一趟绿了，它点到的每份记录照键追加一行；
        baselines/ 照常追加这一趟那一行（键是这一趟独有的那一把）。"""
        step_name = step[0]
        recorded = self.entry_baseline_rows(step_name)
        rules = prune_rules()

        def recorded_before(record):
            return (record["key"], record["kind"], record["binary"] if record["kind"] == "rest" else "") in recorded

        runs, fresh_records = [], []
        if plan["style"] == "nextest":
            tests, binaries = list(plan["refused"]), [item["binary"] for item in plan["whole"]]
            for record in plan["records"]:
                if recorded_before(record):
                    continue
                fresh_records.append(record)
                if record["kind"] == "test":
                    tests.append((record["binary"], record["test"]))
                else:
                    binaries.append(record["binary"])
            if len(fresh_records) == len(plan["records"]):
                runs.append({"text": "这一步全部", "command": list(step[3]), "selection": {"all": True}, "records": list(plan["records"])})
            else:
                binaries = sorted(set(binaries))
                tests = [test for test in tests if test[0] not in binaries]
                command, existing = list(step[3]), None
                if "-E" in command:
                    index = command.index("-E")
                    existing = command[index + 1]
                    del command[index:index + 2]
                filters = rules.nextest_filters(tests) + [f"binary_id(={binary})" for binary in binaries]
                if existing:
                    filters = [f"({text}) & ({existing})" for text in filters]
                command += [word for text in filters for word in ("-E", text)]
                covered = fresh_records + [record for record in plan["records"] if record["binary"] in binaries and record not in fresh_records]
                runs.append({"text": f"{len(tests)} 条测试、{len(binaries)} 个整个照跑的二进制", "command": command,
                             "selection": {"tests": [list(test) for test in tests], "binaries": binaries}, "records": covered})
        else:
            base = [word for word in step[3] if word not in ("--lib", "--tests")] + ["--no-fail-fast"]
            whole = list(plan["whole"])
            for record in plan["records"]:
                exact = {"text": f"{record['binary']} 里拒算的 {len(record['refused'])} 条（--exact）",
                         "command": base + record["selector"] + ["--", "--exact", *record["refused"]],
                         "selection": {"binaries": [record["binary"]], "part": "exact"}, "records": []}
                if recorded_before(record):
                    if record["refused"]:
                        runs.append(exact)
                    continue
                fresh_records.append(record)
                if record["refused"]:
                    runs.append({"text": f"{record['binary']} 去掉拒算的 {len(record['refused'])} 条（--exact --skip）",
                                 "command": base + record["selector"] + ["--", "--exact", *[word for test in record["refused"] for word in ("--skip", test)]],
                                 "selection": {"binaries": [record["binary"]], "part": "skip"}, "records": [record]})
                    runs.append(exact)
                else:
                    whole.append(record)
            if whole:
                runs.insert(0, {"text": f"{len(whole)} 个整个照跑的二进制", "command": base + [word for item in whole for word in item["selector"]],
                                "selection": {"binaries": [item["binary"] for item in whole], "part": "whole"},
                                "records": [item for item in whole if "key" in item]})
        refused_names = [f"{binary} {test}" for binary, test in plan["refused"]]
        say(f"  · 基线 {step_name}：基线键拒算（{self.baseline_refusals[step_name]}），按条记基线：{len(plan['records'])} 份记录沿用 "
            f"{len(plan['records']) - len(fresh_records)}、照跑 {len(fresh_records)}；拒算的 {len(refused_names)} 条每趟照跑"
            + (f"（{'、'.join(refused_names[:5])}{'…' if len(refused_names) > 5 else ''}）" if refused_names else ""))
        total_seconds, entry_rows = 0.0, []
        for run in runs:
            selection_text = "all" if run["selection"].get("all") else "per-entry " + json.dumps(run["selection"], ensure_ascii=False, sort_keys=True)
            exit_code, seconds = self.run_step(step, BASELINE_SEQUENCE, command=run["command"], selection_text=selection_text)
            total_seconds += seconds
            if exit_code != 0:
                self.library.append_rows(BASELINES_TABLE, [[self.baseline_key_of(step_name), step_name, f"exit-{exit_code}", now_text(), self.arguments.threads]])
                self.refuse_red_baseline(step_name, stage, exit_code, f"（按条记基线照跑的那一部分：{run['text']}）")
            entry_rows += run["records"]
        if entry_rows:
            self.library.append_rows(BASELINE_ENTRIES_TABLE,
                                     [[record["key"], step_name, self.arguments.threads, record["kind"], record["binary"], now_text()] for record in entry_rows])
        self.library.append_rows(BASELINES_TABLE, [[self.baseline_key_of(step_name), step_name, "ok", now_text(), self.arguments.threads]])
        say(f"  · 基线 {step_name} 绿（按条记基线：照跑 {len(runs)} 趟共 {total_seconds:.1f} 秒，记下 {len(entry_rows)} 份）")

    def verify_on_pristine_tree(self, step_names, why, suspects=()):
        """在没改过的树上把这几步各跑一遍（调用时树必须是原样）：退非 0 就是环境坏了，退 6。suspects 是因此不可信的那几条的序号。"""
        for step_name in step_names:
            step = self.step_named(step_name)
            exit_code, seconds = self.run_step(step, ENVIRONMENT_SEQUENCE)
            if exit_code == 0:
                os.remove(self.log_path(ENVIRONMENT_SEQUENCE, step_name))
                continue
            distrusted = ""
            if suspects:
                distrusted = (f"；这 {len(suspects)} 条（第 {'、'.join(str(sequence) for sequence in suspects)} 条）的行是环境坏着判的、不可信："
                              "驱动没有强制重判的参数，服务里也没有删键，交主 agent 定怎么作废那几行")
            refuse(f"疑似环境坏了（{why}）：{step_name} 在没改过的树上退 {exit_code}（{seconds:.1f} 秒），基线绿过的这一步现在过不了",
                   f"看日志 {self.log_path(ENVIRONMENT_SEQUENCE, step_name)}：盘满、编译目录被清、工具链换了、scratch 写不进这一类；修好之后重跑同一条命令{distrusted}",
                   EXIT_BASELINE_RED)

    def verify_environment_at_start(self, stages):
        """每次重起：这一趟没现跑基线的那几段，先在没改过的树上把编译那几步各跑一遍。"""
        if break_is("no-environment-check"):
            return
        step_names = [step[0] for step in self.steps if step[2] == "build" and step[1] in stages and step[1] not in self.stages_with_fresh_baseline]
        if step_names:
            self.verify_on_pristine_tree(step_names, "重起先验编译")
            say(f"  · 重起先验编译：{'、'.join(step_names)} 在没改过的树上照过")


def libtest_outcome(line):
    """libtest 结果行的结局（ok / FAILED 等），不是结果行交 None（解析在 lib_libtest_result_lines.py）。"""
    found = libtest_result_lines.result_line(line)
    return found.outcome if found else None


def libtest_failed_name(line):
    """libtest 结果行结局是 FAILED 的全名（` - should panic` 那一段不进名字），别的交 None。弄坏开关 own-libtest-failure-line 下回到改前那条正则（名字带上 ` - should panic`）。"""
    if break_is("own-libtest-failure-line"):
        found = re.match(r"^test (\S.*?) \.\.\. FAILED\s*$", line)
        return found.group(1) if found else None
    found = libtest_result_lines.result_line(line)
    return found.full_name if found and found.outcome == "FAILED" else None


def nextest_failed_name(line):
    found = NEXTEST_FAILURE_LINE.match(line)
    return found.group(1) if found else None


def log_shows_compile_error(log_path):
    """编译那一步的日志里有没有真的编译错误：一条带源码位置的 rustc 错误（error 行之后两行内有 `--> 文件:行:列`），又有 cargo 的「could not compile」。
    磁盘满、锁文件不符、链接器被杀这一类也让 cargo 退 101，但没有带源码位置的错误。"""
    try:
        with open(log_path, encoding="utf-8", errors="replace") as handle:
            lines = handle.read().split("\n")
    except OSError:
        return False
    located = any(COMPILE_ERROR_LINE.match(line) and any(COMPILE_ERROR_LOCATION.match(later) for later in lines[index + 1:index + 3])
                  for index, line in enumerate(lines))
    return located and any(COULD_NOT_COMPILE_LINE.match(line) for line in lines)


def tool_versions(arguments):
    """全局键里的工具版本：nextest、cargo-mutants、llvm-tools（llvm-profdata --version）。"""
    fake = test_variable("BROAD_MUTATION_TOOL_VERSIONS")
    if fake is not None or test_mode():
        return [fake or "selftest-tools"]
    found = []
    for command in (["cargo", "nextest", "--version"], ["cargo", "mutants", "--version"], [os.path.join(llvm_bin(arguments), "llvm-profdata"), "--version"]):
        try:
            done = subprocess.run(command, capture_output=True, text=True, stdin=subprocess.DEVNULL)
            found.append(" ".join(done.stdout.split()[:6]) or f"{command[-2]} 退 {done.returncode}")
        except OSError:
            found.append(f"{command[0]} 不在")
    return found


def llvm_bin(arguments):
    """llvm-profdata、llvm-cov 在哪：--llvm-bin 给的，没给取 rustc 的 sysroot 下 rustup 组件 llvm-tools 的 bin（与编译器同一份 LLVM，profraw 格式对得上）。"""
    if arguments.llvm_bin:
        return arguments.llvm_bin
    try:
        sysroot = subprocess.run(["rustc", "--print", "sysroot"], capture_output=True, text=True, stdin=subprocess.DEVNULL).stdout.strip()
        host = next(line.split(":", 1)[1].strip() for line in toolchain_text(REPOSITORY).splitlines() if line.startswith("host:"))
    except (OSError, StopIteration):
        return ""
    return os.path.join(sysroot, "lib", "rustlib", host, "bin")


def libtest_binary(line, current, package_of_step):
    """libtest 那几步的日志里认出接下来的测试属于哪个二进制：Running <源文件> (<可执行文件>) 与 Doc-tests <crate> 两种行；认不出交 current。"""
    running = LIBTEST_RUNNING_LINE.match(line)
    if running and package_of_step:
        source = running.group("source")
        if source.startswith("src/lib.rs") or source == "src/lib.rs":
            return package_of_step
        name = os.path.basename(source)[:-3] if source.endswith(".rs") else source
        return f"{package_of_step}::bin/{name}" if source.startswith("src/") else f"{package_of_step}::{name}"
    doctests = LIBTEST_DOCTESTS_LINE.match(line)
    if doctests:
        return doctests.group("crate").replace("_", "-") + "::doctest"
    return current


def first_failing_test(log_path, test_failed_exit, package_of_step=""):
    """这一步第一条没过的测试：nextest 的「二进制 id 用例名」原样；libtest 的照前面最近的 Running / Doc-tests 行补上二进制 id（认不出就只有用例名）。"""
    nextest = test_failed_exit == NEXTEST_TEST_FAILED_EXIT
    binary = ""
    try:
        with open(log_path, encoding="utf-8", errors="replace") as handle:
            for line in handle:
                line = line.rstrip("\n")
                if nextest:
                    name = nextest_failed_name(line)
                    if name:
                        return name
                    continue
                binary = libtest_binary(line, binary, package_of_step)
                name = libtest_failed_name(line)
                if name:
                    return f"{binary} {name}" if binary else name
    except OSError:
        pass
    return UNRECOGNISED_TEST


def apply_mutation(library, item):
    """就地改树里的一个文件：先存原件、记在途记录，再写改后的内容。"""
    target = os.path.join(library.tree, item["file"])
    original_digest = (library.manifest or {}).get(item["file"])
    if original_digest is None or file_digest(target) != original_digest:
        refuse(f"要改的 {item['file']} 此刻与这一趟换进来的源码对不上：上一条没恢复干净",
               "重跑同一条命令（开跑时照在途记录恢复、再照源码换树）", EXIT_DIRTY_TREE)
    with open(target, encoding="utf-8") as handle:
        original = handle.read()
    mutated = replace_span(original, tuple(item["start"]), tuple(item["end"]), item["replacement"] + MUTATION_MARKER)
    if mutated == original:
        refuse(f"第 {item['sequence']} 条改完与原文相同，没有命中", "清单与专用树对不上：重跑同一条命令（开跑时重新换树、重新生成清单）", EXIT_INCONSISTENT)
    shutil.copyfile(target, library.inflight_original)
    with open(library.inflight_original, "rb") as handle:
        os.fsync(handle.fileno())
    write_atomically(library.inflight_record, json.dumps(
        {"sequence": item["sequence"], "file": item["file"], "original_digest": original_digest, "started_at": now_text()}).encode())
    with open(target, "w", encoding="utf-8") as handle:
        handle.write(mutated)
    if file_digest(target) == original_digest:
        refuse(f"第 {item['sequence']} 条写进 {target} 之后回读还是原文", "看树所在的盘是不是满了或只读")


def restore_original(library):
    """照在途记录把原件拷回树里、核 sha256、删掉在途记录：交回那份在途记录，没有在途记录交 None。临时件建在库目录里，不进树。
    原件不在或对不上：不拷，删掉在途记录、说一句（开跑的换树照源码把那份文件重写回去）。"""
    if not os.path.exists(library.inflight_record):
        if os.path.exists(library.inflight_original):
            os.remove(library.inflight_original)   # 存了原件、还没记在途记录就停了：树没动过
        return None
    with open(library.inflight_record, encoding="utf-8") as handle:
        record = json.load(handle)
    target = os.path.join(library.tree, record["file"])
    if not os.path.isfile(library.inflight_original) or file_digest(library.inflight_original) != record["original_digest"]:
        note(f"  · 在途记录说 {record['file']} 被第 {record['sequence']} 条改过，但存下的原件不在或对不上：不拷，换树时照源码重写那份文件")
        for path in (library.inflight_record, library.inflight_original):
            if os.path.exists(path):
                os.remove(path)
        return record
    shutil.copyfile(library.inflight_original, library.inflight_restore)
    kill_point("restore-after-copy")
    os.replace(library.inflight_restore, target)
    if file_digest(target) != record["original_digest"]:
        refuse(f"把原件拷回 {target} 之后 sha256 还是对不上", "看树所在的盘是不是满了或只读；修好之后重跑同一条命令（换树照源码重写那份文件）", EXIT_DIRTY_TREE)
    os.remove(library.inflight_record)
    os.remove(library.inflight_original)
    fsync_directory(library.home)
    return record


class ItemRow(NamedTuple):
    fields: list            # 落下的那一行（十六段）
    step_name: str          # 定下结局的那一步（第二段全绿的 missed 是 ""）
    step_seconds: float     # 那一步的秒数
    environment_doubt: str  # 不空：这一条的结局像是环境造成的（编译步退 101 而没有编译错误），调用方当场在原件上验一次


def process_item(library, runner, item, with_stage_one, with_stage_two):
    """判一条：交回这一条落下的最后一行（ItemRow）。中途被停（Stopped）不落行，原件照样拷回。"""
    sequence = item["sequence"]
    judgement = runner.judgement
    current = judgement.key_of(item)
    item["verdict_key"] = current["key"] or ""
    totals = {"build": 0.0, "test": 0.0}
    selection = judgement.selection(item, runner)
    ran_any_test = []
    method = "fresh"

    def record(outcome, stage, test_name="", note_text="", step_name="", step_seconds=0.0, environment_doubt=""):
        if outcome == "missed" and not ran_any_test:
            note_text = (note_text + "；" if note_text else "") + "零覆盖：没有一条测试执行到它（C2 记没红）"
        refused = list(current["refused"])
        test_key = coverage_key = ""
        if outcome == "caught":
            test_key, why = judgement.test_key_for(test_name)
            test_key = test_key or ""
            if not test_key:
                # 抓到它的测试认不出（文件头「键」）：这一条每趟现判，另记覆盖者键给提交门禁核
                refused.append(f"catching-test-unresolved:{why}")
                coverage_key = judgement.coverers_for(current["unit"])[0] or ""
            else:
                unit, target_id, _why = judgement.rules.resolve_test(judgement.tree, test_name)
                if unit is not None:
                    _key, composition, _forms = judgement.tree.closure_key(unit, judgement.globals, target_id)
                    library.write_composition(test_key, composition)
        if outcome == "missed":
            coverage_key = judgement.coverers_for(current["unit"])[0] or ""
        fields = [current["key"] or "", sequence, item["package"], item["name"], outcome, stage, test_name, test_key, coverage_key,
                  f"{totals['build']:.1f}", f"{totals['test']:.1f}", today_text(), method, current["locator"], ",".join(refused), note_text]
        library.write_composition(current["key"], current["composition"])
        library.record_verdict(fields)
        return ItemRow(fields, step_name, step_seconds, environment_doubt)

    def run_stage(stage):
        """跑这一段的每一步：交 None（全绿）或已经落下的那一行。"""
        for step in runner.steps:
            step_name, step_stage, kind, _command, test_failed_exit = step
            if step_stage != stage:
                continue
            command, selection_text = runner.command_for(step, selection)
            if command is None:
                continue
            if step_name in PER_ENTRY_TEST_STEPS:
                ran_any_test.append(step_name)
            exit_code, seconds = runner.run_step(step, sequence, command=command, selection_text=selection_text)
            totals[kind] += seconds
            log_path = runner.log_path(sequence, step_name)
            if exit_code == 0:
                os.remove(log_path)
                continue
            if kind == "build" and exit_code == CARGO_FAILED_EXIT:
                if log_shows_compile_error(log_path) or break_is("unviable-by-exit-code"):
                    return record("unviable", stage, note_text=f"{step_name} 编不过", step_name=step_name, step_seconds=seconds)
                return record(DRIVER_ERROR, stage, note_text=f"{step_name} 退 {exit_code} 而日志里没有带源码位置的编译错误：不算无效", step_name=step_name,
                              step_seconds=seconds, environment_doubt=f"第 {sequence} 条的 {step_name} 退 {exit_code} 而没有编译错误")
            if kind == "test" and exit_code == test_failed_exit:
                package_of_step = VERIFICATION_PACKAGE if "checker-tier" in step_name else ""
                return record("caught", stage, first_failing_test(log_path, test_failed_exit, package_of_step), note_text=step_name,
                              step_name=step_name, step_seconds=seconds)
            if break_is("unknown-exit-stops"):
                refuse(f"第 {sequence} 条的 {step_name} 退 {exit_code}（认不出的退出码）：不落行", f"看日志 {log_path}", EXIT_INCONSISTENT)
            return record(DRIVER_ERROR, stage, note_text=f"{step_name} 退 {exit_code}（认不出的退出码，不是这条变异的结局；日志 {os.path.basename(log_path)}）",
                          step_name=step_name, step_seconds=seconds)
        return None

    try:
        apply_mutation(library, item)
        if with_stage_one:
            row = run_stage("one")
            if row is not None:
                return row
            row = record(STAGE_ONE_ONLY, "one", note_text="第一段没红；第二段接着跑" if with_stage_two else "第一段没红；这一趟没跑第二段")
            if not with_stage_two:
                return row
        row = run_stage("two")
        if row is not None:
            return row
        return record("caught" if break_is("stage-two-missed-as-caught") else "missed", "two")
    finally:
        restore_original(library)


# ── GPU 那一趟（C489：第二段补一趟带 gpu 特性的 验证包；文件头「GPU 那一趟」） ───────────────────────────────────────
# 这一节的函数都不在 JUDGING_ENTRIES 的闭包里：CPU 那一趟的判法单元指纹不因它们变，库里 CPU 那一趟的判定照沿用；
# GPU 那一趟自己的判法单元指纹从 GPU_JUDGING_ENTRIES 收（gpu_judging_key），进 GPU 判定行的 gpu_key。

GPU_VERDICT_COLUMNS = ("key", "sequence", "package", "name", "outcome", "caught_by", "test_key", "coverage_key", "gpu_key", "test_seconds",
                       "judged_on", "method", "note")
GPU_ROW_OUTCOMES = (*DECISIVE_OUTCOMES, DRIVER_ERROR)
GPU_FEATURE = "gpu"
GPU_TESTS_DIRECTORY = "crates/singlefs-verification/tests"
GPU_BUILD_STEP, GPU_TEST_STEP = "gpu-build-checker-tier", "gpu-test-checker-tier"
GPU_BASELINE_SEQUENCE = "gpu-baseline"
GPU_NO_ADAPTER_MARK = "NOT_RUN_NO_GPU_ADAPTER"      # GPU 对拍找不到 GPU 适配器时测试打的那一行（门禁 crash-point-replay 的 feature-gated-tests 格读同一个词）
GPU_NOT_RUN = "not-run"                              # 这一趟没跑成：不落行、不算没红
BINDGEN_LIBRARY = os.path.join(SCRIPT_DIRECTORY, "..", "crash", "crash-amplification-lib.sh")
GPU_ATTRIBUTE_START = re.compile(r"#(!?)\[\s*cfg\s*\(")
GPU_FEATURE_NAME = re.compile(r"\bfeature\s*=\s*\"([^\"]+)\"")
GPU_PASSED_COUNT = re.compile(r"^test result: \w+\. (\d+) passed", re.M)
# GPU 那一趟自己的判法单元指纹（文件头「GPU 那一趟」）：从这几个函数收 python 单元闭包（命令怎么拼、步结局怎么分、沿用怎么判、两趟怎么合）
GPU_JUDGING_ENTRIES = (("scripts/mutation/broad-mutation.py", "gpu_pass_command"), ("scripts/mutation/broad-mutation.py", "gpu_classify_build"),
                       ("scripts/mutation/broad-mutation.py", "gpu_classify_test"), ("scripts/mutation/broad-mutation.py", "gpu_combined_state"),
                       ("scripts/mutation/broad-mutation.py", "GpuJudging.need"), ("scripts/mutation/broad-mutation.py", "GpuJudging.row_holds"),
                       ("scripts/mutation/broad-mutation.py", "GpuJudging.test_key_of"), ("scripts/mutation/broad-mutation.py", "GpuJudging.state"),
                       ("scripts/mutation/broad-mutation.py", "gpu_decided_keys"))
GPU_JUDGING_KEYS = {}


class GpuOutcome(NamedTuple):
    outcome: str       # missed / caught / driver-error / not-run
    caught_by: str     # 抓到它的测试（「<二进制 id> <用例名>」），别的结局是空
    note: str


class GpuTotals(NamedTuple):
    needed: int        # 要这一趟的条数（CPU 那一趟没红、又有带特性的测试够得着它）
    judged: int        # 这一趟现判了几条
    carried: int       # 键相同、沿用得了的几条
    not_run: int       # 本次未跑几条（没卡、中途没卡）
    failed: bool       # 复用哨兵判红
    outcomes: dict     # 现判的各结局条数


# gate-overlap:copy-kept crash-point-replay.sh 的 list_feature_gated_test_files 是嵌在 bash 里的 python，读不进来；这里同一套判法照搬一份，两边都认 tests/ 里的 cfg 点到的特性
def gated_test_files(tests_directory):
    """验证包 tests/ 顶层每个 .rs 里 #[cfg(…)] / #![cfg(…)] 点到的特性：[(文件名, whole-file 或 items, [特性])]，照文件名排序；挂特性的一个都没有交空表。
    括号按层数配对（rustfmt 把长的 cfg 折成几行也认），// 起头的行不读。"""
    found = []
    try:
        names = sorted(name for name in os.listdir(tests_directory) if name.endswith(".rs") and os.path.isfile(os.path.join(tests_directory, name)))
    except OSError:
        return found
    for name in names:
        with open(os.path.join(tests_directory, name), encoding="utf-8", errors="replace") as handle:
            text = "\n".join(line for line in handle.read().splitlines() if not line.lstrip().startswith("//"))
        features, whole_file = set(), False
        for start in GPU_ATTRIBUTE_START.finditer(text):
            depth, position = 1, start.end()
            while position < len(text) and depth > 0:
                depth += {"(": 1, ")": -1}.get(text[position], 0)
                position += 1
            named = GPU_FEATURE_NAME.findall(text[start.end():position])
            features.update(named)
            if named and start.group(1) == "!":
                whole_file = True
        if features:
            found.append((name, "whole-file" if whole_file else "items", sorted(features)))
    return found


def gpu_bindgen_arguments():
    """librocksdb-sys 的 bindgen 要的头文件目录：与崩溃放量取同一个值（scripts/crash/crash-amplification-lib.sh 的 CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS），
    编译产物才共用。在子 shell 里 source 那份库、只读这一个变量；读不到交空串（调用方说一句、照调用方的环境）。"""
    if not os.path.isfile(BINDGEN_LIBRARY):
        return ""
    done = subprocess.run(["bash", "-c", 'source "$1" >/dev/null 2>&1; printf "%s" "${CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS:-}"', "bindgen", BINDGEN_LIBRARY],
                          capture_output=True, text=True, stdin=subprocess.DEVNULL)
    return done.stdout if done.returncode == 0 else ""


def gpu_pass_command(kind, features, labels):
    """GPU 那一趟一步的命令（verification.md 里带特性那一趟）：编译步 cargo test --release -p singlefs-verification --features <特性> <目标> --no-run；
    测试步在它后面加 `-- --nocapture --test-threads 1`（GPU 对拍的测试各占卡的显存，不并行起），直接起、不另套清场（文件头「GPU 那一趟」）。
    labels 是要跑的目标：lib 是 --lib，别的是 --test <文件名>。弄坏开关 gpu-test-threads-dropped 不加 --test-threads 1。"""
    targets = []
    for label in sorted(labels, key=lambda text: (text != "lib", text)):
        targets += ["--lib"] if label == "lib" else ["--test", label]
    head = ["cargo", "test", "--locked", "--release", "-p", VERIFICATION_PACKAGE, "--features", ",".join(features), *targets]
    if kind == "build":
        return head + ["--no-run"]
    threads = [] if break_is("gpu-test-threads-dropped") else ["--test-threads", "1"]
    return head + ["--", "--nocapture", *threads]


def gpu_log_text(log_path):
    try:
        with open(log_path, encoding="utf-8", errors="replace") as handle:
            return handle.read()
    except OSError:
        return ""


def gpu_failing_test(log_path):
    """GPU 那一趟第一条没过的测试：先照 libtest 结果行认（first_failing_test）；--nocapture 加 --test-threads 1 下结果行常被测试自己的输出打散，认不出就读收尾的
    「failures:」名单（缩进四格的测试名），二进制照前面最近的 Running 行补。认不出交 UNRECOGNISED_TEST。"""
    name = first_failing_test(log_path, CARGO_FAILED_EXIT, VERIFICATION_PACKAGE)
    if name != UNRECOGNISED_TEST:
        return name
    binary, listing = "", False
    for line in gpu_log_text(log_path).splitlines():
        binary = libtest_binary(line, binary, VERIFICATION_PACKAGE)
        if line.rstrip() == "failures:":
            listing = True
            continue
        if listing:
            named = re.match(r"^    (\S+)\s*$", line)
            if named:
                return f"{binary} {named.group(1)}" if binary else named.group(1)
            if line.strip() and not line.startswith(" "):
                listing = False
    return UNRECOGNISED_TEST


def gpu_classify_build(exit_code, log_path):
    """GPU 那一趟编译步的结局：→ (None 编得过 | missed 不会有 | unviable / driver-error, 备注)。
    编译那一步退 101 而日志里没有带源码位置的编译错误（磁盘满、锁文件不符这一类）不记无效，记驱动错。"""
    if exit_code == 0:
        return None, ""
    if exit_code == CARGO_FAILED_EXIT:
        if log_shows_compile_error(log_path):
            return "unviable", f"{GPU_BUILD_STEP} 编不过"
        return DRIVER_ERROR, f"{GPU_BUILD_STEP} 退 {exit_code} 而日志里没有带源码位置的编译错误：不算无效"
    return DRIVER_ERROR, f"{GPU_BUILD_STEP} 退 {exit_code}（认不出的退出码，不是这条变异的结局）"


def gpu_classify_test(exit_code, log_path):
    """GPU 那一趟测试步的结局（GpuOutcome）：退 0 且跑到了测试、日志里没有 NOT_RUN_NO_GPU_ADAPTER 才是没红（missed）；
    日志里有 NOT_RUN_NO_GPU_ADAPTER（GPU 测试没找到卡，打一行就返回成功）是本次未跑（not-run，不落行、不算没红）；
    退 0 而一个测试都没跑到不算没红，记驱动错；退 101 是有测试没过 = 抓到，带第一条没过的测试。"""
    text = gpu_log_text(log_path)
    passed = sum(int(count) for count in GPU_PASSED_COUNT.findall(text))
    if exit_code == 0:
        if GPU_NO_ADAPTER_MARK in text and not break_is("gpu-not-run-as-missed"):
            return GpuOutcome(GPU_NOT_RUN, "", f"no-adapter：日志里有 {GPU_NO_ADAPTER_MARK}，GPU 测试没找到卡、没跑")
        if passed == 0:
            return GpuOutcome(DRIVER_ERROR, "", "GPU 趟一个测试都没跑到（日志里没有 passed 的计数）：不算没红")
        return GpuOutcome("missed", "", f"{passed} 条测试都过了")
    if exit_code == CARGO_FAILED_EXIT:
        return GpuOutcome("caught", gpu_failing_test(log_path), GPU_TEST_STEP)
    return GpuOutcome(DRIVER_ERROR, "", f"{GPU_TEST_STEP} 退 {exit_code}（认不出的退出码，不是这条变异的结局）")


def gpu_combined_state(cpu, needed, gpu):
    """两趟合起来的状态（ItemState）与没结论时的理由：CPU 那一趟没红、又要 GPU 那一趟的条，看 GPU 那一趟的状态——抓到 / 没红 / 无效就是定论，
    驱动错照样是没结论，库里还没有它的结论是 none（理由 gpu-not-run，不算没红）；别的条就是 CPU 那一趟的状态。
    一条两趟里任一趟红算抓到：CPU 那一趟抓到的，GPU 那一趟不用跑。弄坏开关 gpu-red-not-caught：GPU 那一趟红不算抓到；gpu-not-run-as-missed：本次未跑记成没红。"""
    if cpu.state != "missed" or not needed:
        return cpu, ""
    if break_is("gpu-red-not-caught") and gpu.state == "caught":
        return ItemState("missed", cpu.passed_stage_one), ""
    if gpu.state in DECISIVE_OUTCOMES:
        return ItemState(gpu.state, cpu.passed_stage_one), ""
    if gpu.state == DRIVER_ERROR:
        return ItemState(gpu.state, cpu.passed_stage_one), f"gpu-{gpu.state}"
    if break_is("gpu-not-run-as-missed"):
        return ItemState("missed", cpu.passed_stage_one), ""
    return ItemState(NOT_RUN, cpu.passed_stage_one), "gpu-not-run"


def lines_digest(lines):
    return hashlib.sha256("".join(line + "\n" for line in lines).encode("utf-8", "surrogateescape")).hexdigest()


def gpu_judging_key(root=REPOSITORY):
    """GPU 那一趟的判法单元指纹（文件头「GPU 那一趟」）：GPU_JUDGING_ENTRIES 那几个函数在 root 上的 python 单元闭包键，连起来取 sha256；与 CPU 那一趟的
    判法单元指纹（judging_units_key）分开：改 GPU 那一趟的判法不动 CPU 那一趟库里的判定，改 CPU 那一趟的判法单元也不让 GPU 判定行单独作废（它们的键本来就带 CPU 的）。"""
    fake = test_variable("BROAD_MUTATION_GPU_JUDGING_KEY")
    if fake and root == REPOSITORY:
        return fake
    if root in GPU_JUDGING_KEYS:
        return GPU_JUDGING_KEYS[root]
    library = prune_rules().units_library()
    options = library.Options()
    options.under = [place for place in JUDGING_SCAN_FILES if os.path.isfile(os.path.join(root, place))]
    lines = []
    for path, name in GPU_JUDGING_ENTRIES:
        try:
            closure = library.closure_of(root, f"{path}::{name}", options)
        except (library.UsageError, library.FunctionUnitsError) as error:
            refuse(f"GPU 趟的判法单元指纹算不出：{path}::{name}（{error}）", "GPU_JUDGING_ENTRIES 里的函数改了名就照着改那一张表；scripts/gate/lib_function_units.py 起不来就先修它")
        key, refusals = library.closure_key(closure, allow_refused=True)
        lines.append(f"{path}::{name}\t{key}\t{','.join(sorted({refusal.form for refusal in refusals}))}")
    GPU_JUDGING_KEYS[root] = lines_digest(lines)
    return GPU_JUDGING_KEYS[root]


# ── GPU 判定表：判定库 gpu-verdicts/（只追加，文件头「判定库」） ─────────────────────────────────────────────

def read_gpu_verdicts(library):
    """GPU 判定表：段数对的行才算数，不对的条目列进 malformed。→ LibraryRows。"""
    return library.read_rows(GPU_VERDICTS_TABLE, GPU_VERDICT_COLUMNS)


def record_gpu_verdict(library, fields):
    """落一行 GPU 判定（十三段）：写进判定库、等服务回确认。"""
    library.append_rows(GPU_VERDICTS_TABLE, [fields[:len(GPU_VERDICT_COLUMNS)]])


# ── GPU 那一趟的判定：要不要跑、键、沿用、两趟合起来 ───────────────────────────────────────────────────────────

class GpuView(NamedTuple):
    needed: bool           # CPU 那一趟没红、又有带特性的测试够得着它：要 GPU 那一趟
    labels: list           # 要跑的目标（lib、挂特性的测试文件名）
    need: dict             # GpuJudging.need 的原样（coverers 键等）
    gpu: ItemState         # GPU 那一趟自己的状态（没要这一趟的条是 none）
    combined: ItemState    # 两趟合起来的状态
    reason: str            # 合起来没有结论时的理由
    deciding: object       # 定下 GPU 状态的那一行（没有是 None）


def refresh_verdict_rows(judgement):
    """判定库重读一遍，换进 Judgement（构造时读的那几行之后又落了新行）；单元树、全局那几行与已算好的键不动。"""
    judgement.table = judgement.library.read_verdicts()
    judgement.by_key, judgement.by_locator = {}, {}
    for row in judgement.table.rows:
        judgement.by_key.setdefault(row["key"], []).append(row)
        judgement.by_locator.setdefault(row["locator"], []).append(row)


def cpu_state_for_gpu(judgement, item):
    """CPU 那一趟的状态，不论拒算：真跑时 Judgement.state 对拒算的条一律给 none（每趟现判），GPU 那一趟要看的是 CPU 那一趟最后落的结论，所以临时按只读的判法问一次。"""
    saved = judgement.for_run
    judgement.for_run = False
    try:
        return judgement.state(item)
    finally:
        judgement.for_run = saved


def gpu_decided_keys(library, judgement, items, arguments):
    """选条之前问：items（CPU 一趟的没红行被撤绿的那几条）里哪些由 GPU 一趟定红不红、而 GPU 趟的真实状态此刻不是没红 → 它们的单元键。
    要 GPU 趟（CPU 一趟的真实状态是没红、又有带特性的测试够得着）的条，红不红看合起来的状态（文件头「记红、撤绿、下趟现判」）：GPU 趟没红才红，
    抓到 / 无效 / 还没有结论 / 没结论的，CPU 一趟的没红行照沿用（GPU 趟自己补或重判）。没有带特性的测试、或这些条用不着 GPU 趟：交空集合。"""
    decided = set()
    judging = GpuJudging(library, judgement, arguments)
    if not judging.enabled:
        return decided
    for item in items:
        if cpu_state_for_gpu(judgement, item)[0].state != "missed":
            continue
        current = judgement.key_of(item)
        if current["key"] is None:
            continue
        need = judging.need(current)
        if need["needed"] and judging.state(current, need, for_run=False)[0].state != "missed":
            decided.add(current["key"])
    return decided


class GpuJudging:
    """一棵树上 GPU 那一趟的判定（文件头「GPU 那一趟」）：带特性的测试够得着哪些单元、这一趟要跑哪几个目标、判定库里键相同的行沿用不沿用。
    judgement 是同一棵树上 CPU 那一趟的 Judgement（共用它的单元树、全局那几行、排除表）。"""

    def __init__(self, library, judgement, arguments):
        self.library, self.judgement, self.arguments = library, judgement, arguments
        self.rules = judgement.rules
        self.gated = gated_test_files(os.path.join(judgement.root, GPU_TESTS_DIRECTORY))
        self.features = sorted({feature for _name, _scope, features in self.gated for feature in features})
        self.enabled = GPU_FEATURE in self.features
        self.gated_labels = {name[:-3] for name, _scope, _features in self.gated}
        self.feature_arguments = [f"{VERIFICATION_PACKAGE}/{feature}" for feature in self.features]
        self.entry_list, self.closures, self.keys, self.cached_pass_key = None, {}, {}, None
        self.table = read_gpu_verdicts(library)
        self.by_key = {}
        for row in self.table.rows:
            self.by_key.setdefault(row["key"], []).append(row)

    def pass_key(self):
        """GPU 那一趟自己的键（每一行 GPU 判定都带）：这一趟的写法、编进来的特性、GPU 趟的判法单元指纹。"""
        if self.cached_pass_key is None:
            self.cached_pass_key = lines_digest(["broad-mutation-gpu-pass-v1", "features\t" + ",".join(self.features), "judging\t" + gpu_judging_key()])
        return self.cached_pass_key

    def entries(self):
        """这一趟跑得到的测试（静态列出，不含 #[ignore] 的）：验证包的 lib 与挂特性的测试文件里的测试，带上要跑的目标（lib / 文件名）。"""
        if self.entry_list is None:
            found = []
            for entry in self.judgement.runnable((VERIFICATION_PACKAGE,)):
                parts = entry["target"].split(":", 2)
                if parts[1] == "lib":
                    label = "lib"
                elif parts[1] == "test" and parts[2] in self.gated_labels:
                    label = parts[2]
                else:
                    continue
                found.append(dict(entry, label=label))
            self.entry_list = found
        return self.entry_list

    def closure_of(self, entry):
        """一条测试在这一趟的 cfg 与特性下的闭包（带 --release 的 cfg 去掉 debug_assertions，特性是挂特性的文件点到的那几个）；入口在这一组 cfg 下不编交 None。"""
        cache_key = (id(entry["unit"]), entry["target"])
        if cache_key not in self.closures:
            tree = self.judgement.tree
            library = tree.library
            release_cfg = [item for item in library.DEFAULT_RUST_CFG if item != "debug_assertions"]
            found = None
            for extra in ((), ("test",)):
                try:
                    found = library.rust_closure_of(tree.workspace, entry["unit"], entry["target"], release_cfg + list(extra), no_default_cfg=True,
                                                    features=self.feature_arguments)
                    break
                except library.UsageError:
                    found = None
            self.closures[cache_key] = found
        return self.closures[cache_key]

    @staticmethod
    def references_gpu(closure):
        """闭包真引到 gpu 原子（带这一趟的真值 true）：闭包里有单元的 cfg 点到 feature = "gpu"。"""
        return any(atom == f'feature="{GPU_FEATURE}"\ttrue' for atom in closure.cfg_atoms)

    def test_key_of(self, entry):
        """一条测试在这一趟的测试键 = 它的单元闭包键（闭包键里有单元的种类与去注释指纹、闭包真引到的 cfg 原子与真值、依赖版本行、工具链那几行与 edition）：→ (键或 None, 拒算形态)。"""
        cache_key = (id(entry["unit"]), entry["target"])
        if cache_key not in self.keys:
            closure = self.closure_of(entry)
            if closure is None:
                self.keys[cache_key] = (None, ["entry-cfg-inactive"])
            else:
                tree = self.judgement.tree
                dependencies, dependency_refusals = tree.dependency_lines(closure)
                key, closure_refusals = tree.library.closure_key(closure, toolchain=list(self.judgement.globals) + tree.edition_lines(closure),
                                                                 dependencies=dependencies, allow_refused=True)
                forms = sorted({refusal.form for refusal in closure_refusals}) + dependency_refusals
                self.keys[cache_key] = (None, forms) if forms else (key, [])
        return self.keys[cache_key]

    def need(self, current):
        """这一条要不要 GPU 那一趟：→ {needed, labels, coverers, why}。要 = 有一条这一趟跑得到的测试，它带特性的闭包里有这条变异的容器单元、
        又真引到 gpu 原子（闭包不碰容器单元的测试、碰到了却不引 gpu 的，这一趟帮不上）；labels 是这些测试所在的目标；coverers 是它们各自测试键的多重集取 sha256
        （沿用没红的要它相同），有一条的键拒算就是空串、why 写原因。弄坏开关 gpu-all-items：不看闭包，每一条 CPU 那一趟没红的都要、目标取全部。"""
        unit = current["unit"]
        if unit is None or not self.enabled:
            return {"needed": False, "labels": [], "coverers": "", "why": "no-unit"}
        keys, labels, why = [], set(), ""
        for entry in self.entries():
            if break_is("gpu-all-items"):
                labels.add(entry["label"])
                continue
            closure = self.closure_of(entry)
            if closure is None or id(unit) not in closure.members or not self.references_gpu(closure):
                continue
            labels.add(entry["label"])
            key, refused = self.test_key_of(entry)
            if key is None and not why:
                why = f"coverer-refused:{entry['binary']} {entry['test']}:" + ",".join(refused or ["closure"])
            elif key is not None:
                keys.append(key)
        if not labels:
            return {"needed": False, "labels": [], "coverers": "", "why": ""}
        coverers = "" if why or break_is("gpu-all-items") else lines_digest(["gpu-coverers-v1"] + sorted(keys))
        return {"needed": True, "labels": sorted(labels), "coverers": coverers, "why": why}

    def catching_key(self, caught_by):
        """抓到它的测试在这一趟此刻的测试键：→ (键或 "", 为什么)。"""
        unit, target_id, why = self.rules.resolve_test(self.judgement.tree, caught_by)
        if unit is None:
            return "", why
        key, refused = self.test_key_of({"unit": unit, "target": target_id})
        return (key, "") if key else ("", "catching-test-refused:" + ",".join(refused))

    def row_holds(self, row, current, need, for_run):
        """库里键相同的这一行 GPU 判定此刻还认不认（沿用判据，同 mutation-prune.py 的 carry_decision）：→ (认不认, 为什么不认)。
        这一趟自己的键（pass_key）对不上的不认；抓到的要那条测试此刻的测试键相同；没红的要覆盖者键相同；无效的键相同就认；
        驱动错不是结论，认下来只为定状态；抓到它的测试认不出的，真跑每趟现判、只读时看覆盖者键。"""
        if row["gpu_key"] != self.pass_key():
            return False, "gpu-pass-changed"
        outcome = row["outcome"]
        if outcome not in DECISIVE_OUTCOMES:
            return True, ""
        if outcome == "caught" and row["note"].startswith("catching-test-unresolved"):
            if for_run:
                return False, row["note"]
            return (True, "") if need["coverers"] and need["coverers"] == row["coverage_key"] else (False, "coverers-changed")
        test_now, test_why = self.catching_key(row["caught_by"]) if outcome == "caught" else ("", "")
        record = {"key": row["key"], "outcome": outcome, "caught_by": row["caught_by"], "test_key": row["test_key"], "coverage_key": row["coverage_key"]}
        return self.rules.carry_decision(record, {"key": current["key"], "refused": []}, test_now, test_why, need["coverers"], need["why"])

    def state(self, current, need, for_run):
        """GPU 那一趟的状态：→ (ItemState, 为什么沿用不了, 定下状态的那一行)。真跑时拒算的条一律沿用不了（每趟现判）。"""
        if current["key"] is None or (for_run and current["refused"]):
            return ItemState(NOT_RUN, False), "refused:" + ",".join(current["refused"]), None
        rows = self.by_key.get(current["key"], [])
        kept, reasons = [], []
        for row in rows:
            holds, why = self.row_holds(row, current, need, for_run)
            (kept if holds else reasons).append(row if holds else why)
        state = state_of_outcomes([row["outcome"] for row in kept])
        deciding = next((row for row in reversed(kept) if row["outcome"] == state.state), None)
        if for_run and state.state in DECISIVE_OUTCOMES and not break_is("missed-carried"):
            # 记红、撤绿、下趟现判（文件头「判定行」末段）：这一趟自己的键（pass_key）下，同一把键的定论里最后一个是没红的，不沿用、现判；认不认只定 --summary 的状态，撤绿不看覆盖者键变没变
            decisive = [row["outcome"] for row in rows if row["gpu_key"] == self.pass_key() and row["outcome"] in DECISIVE_OUTCOMES]
            if decisive and decisive[-1] == "missed":
                return ItemState(NOT_RUN, False), "missed-rejudged", None
        return state, (reasons[-1] if reasons else ("no-record" if not rows else "")), deciding

    def view(self, item, cpu, for_run):
        """一条两趟合起来的样子（GpuView）。cpu 是 CPU 那一趟的 ItemState。"""
        nothing = ItemState(NOT_RUN, False)
        if cpu.state != "missed" or not self.enabled:
            return GpuView(False, [], {"needed": False, "labels": [], "coverers": "", "why": ""}, nothing, cpu, "", None)
        current = self.judgement.key_of(item)
        if current["key"] is None:
            return GpuView(False, [], {"needed": False, "labels": [], "coverers": "", "why": "no-key"}, nothing, cpu, "", None)
        need = self.need(current)
        if not need["needed"]:
            return GpuView(False, [], need, nothing, cpu, "", None)
        gpu, _why, deciding = self.state(current, need, for_run)
        combined, reason = gpu_combined_state(cpu, True, gpu)
        return GpuView(True, need["labels"], need, gpu, combined, reason, deciding)


def gpu_views(library, arguments, judgement, items, states, for_run):
    """每一条两趟合起来的样子：→ (GpuJudging 或 None, {序号: GpuView})。没有 CPU 那一趟没红的条就不碰 GPU 判定（不算带特性的闭包）。"""
    views, judging = {}, None
    for item in items:
        cpu = states[item["sequence"]][0]
        if cpu.state == "missed":
            if judging is None:
                judging = GpuJudging(library, judgement, arguments)
            views[item["sequence"]] = judging.view(item, cpu, for_run)
    return judging, views


def combined_states(states, views):
    """{序号: (合起来的 ItemState, 理由, 定下状态的 CPU 那一行)}：要 GPU 那一趟的条用合起来的状态（理由 gpu-…），别的照 CPU 那一趟。"""
    found = {}
    for sequence, (state, reason, deciding) in states.items():
        view = views.get(sequence)
        if view is not None and view.needed:
            found[sequence] = (view.combined, view.reason or reason, deciding if view.combined.state == state.state else None)
        else:
            found[sequence] = (state, reason, deciding)
    return found


# ── GPU 那一趟的跑：基线、逐条、本次未跑 ─────────────────────────────────────────────────────────────────────

class GpuPass:
    """GPU 那一趟的步（文件头「GPU 那一趟」）：编译步与测试步各自经 Runner.run_step 直接起（不另套清场）。
    基线 = 在没改过的树上把这一趟要跑的目标跑绿一遍（编译步、测试步各一次）；这一趟的基线不记库，每趟现跑。每一步都不设内存上限、不限时（文件头「内存与限时」）。"""

    def __init__(self, runner, judging):
        self.runner, self.judging = runner, judging
        self.arguments, self.library = runner.arguments, runner.library
        self.bindgen = gpu_bindgen_arguments()
        self.build_step = (GPU_BUILD_STEP, "gpu", "build", [], None)
        self.test_step = (GPU_TEST_STEP, "gpu", "test", [], None)

    def run(self, step, sequence, labels):
        command = gpu_pass_command("build" if step is self.build_step else "test", self.judging.features, labels)
        extra = {"BINDGEN_EXTRA_CLANG_ARGS": self.bindgen} if self.bindgen else {}
        if self.runner.fake_driver:
            extra["BROAD_MUTATION_STEP_COMMAND"] = json.dumps(command)
        return self.runner.run_step(step, sequence, command=command, extra=extra)

    def ensure_baseline(self, labels):
        """→ (跑得了, 为什么跑不了)。没改过的树上编译红 / 测试红 / 一个测试都没跑到：拒（退 6，基线不绿后面每一条抓到都不作数）；
        没找到卡：这一趟本次未跑，不拒。"""
        sequence = GPU_BASELINE_SEQUENCE
        exit_code, build_seconds = self.run(self.build_step, sequence, labels)
        build_log = self.runner.log_path(sequence, GPU_BUILD_STEP)
        outcome, note_text = gpu_classify_build(exit_code, build_log)
        if outcome is not None:
            refuse(f"GPU 趟基线 {GPU_BUILD_STEP} 在没改过的树上退 {exit_code}（{note_text}）：基线不绿，后面每一条 GPU 趟的抓到都不作数，一条都不跑",
                   f"看日志 {build_log}；代码本来就编不过就先修代码（带 --features {','.join(self.judging.features)} 的 验证包），是环境（盘满、缺 libclang、BINDGEN_EXTRA_CLANG_ARGS）就修环境，重跑同一条命令",
                   EXIT_BASELINE_RED)
        os.remove(build_log)
        exit_code, seconds = self.run(self.test_step, sequence, labels)
        test_log = self.runner.log_path(sequence, GPU_TEST_STEP)
        result = gpu_classify_test(exit_code, test_log)
        if result.outcome == GPU_NOT_RUN:
            return False, result.note
        if result.outcome != "missed":
            refuse(f"GPU 趟基线 {GPU_TEST_STEP} 在没改过的树上不绿（{'有测试没过：' + result.caught_by if result.outcome == 'caught' else result.note}）：基线不绿，后面每一条 GPU 趟的抓到都不作数，一条都不跑",
                   f"看日志 {test_log}；代码本来就红就先修代码，是卡的毛病（显存、别的会话占着）就等卡空了重跑同一条命令",
                   EXIT_BASELINE_RED)
        os.remove(test_log)
        say(f"  · GPU 趟基线绿：{GPU_BUILD_STEP} {build_seconds:.1f} 秒、{GPU_TEST_STEP} {seconds:.1f} 秒（目标 {','.join(labels)}）；每一步都不设内存上限、不限时")
        return True, ""


def plan_gpu_sentinel(candidates, arguments):
    """GPU 那一趟的复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「二」）：候选是这一趟罩得住、沿用判据此刻认定论的 GPU 判定，项名「gpu:<序号>」。
    点名的照样现判（自证变量 BROAD_MUTATION_REJUDGE_NAMED）时不读库不抽。弄坏开关 gpu-carry-dropped（沿用得了的照样重跑）下这一趟判红「复用失灵」。"""
    sentinel = reuse_checks.ReuseSentinel(SCRIPT_KEY, "广谱变异 GPU 趟")
    off = "点名的照样现判（BROAD_MUTATION_REJUDGE_NAMED）" if named_items_rejudged() else ""
    sentinel.plan({f"gpu:{sequence}": outcome for sequence, outcome in candidates.items()}, reuse=not off, off_reason=off, printer=say)
    return sentinel


def judge_gpu_item(library, gpu_pass, item, need):
    """判一条的 GPU 那一趟：先编（编不过记无效）再跑测试步，落一行 GPU 判定（本次未跑的不落行）。→ (GpuOutcome, 编译秒, 测试秒)。
    树里的原件无论怎么退都拷回；被 TERM 停（Stopped）不落行。"""
    runner, judging = gpu_pass.runner, gpu_pass.judging
    sequence, current = item["sequence"], judging.judgement.key_of(item)
    labels, method = need["labels"], "fresh"
    seconds = {"build": 0.0, "test": 0.0}

    def record(outcome, caught_by="", test_key="", coverage_key="", note_text=""):
        record_gpu_verdict(library, [current["key"], sequence, item["package"], item["name"], outcome, caught_by, test_key, coverage_key, judging.pass_key(),
                                     f"{seconds['test']:.1f}", today_text(), method, note_text])

    try:
        apply_mutation(library, item)
        exit_code, took = gpu_pass.run(gpu_pass.build_step, sequence, labels)
        seconds["build"] += took
        build_log = runner.log_path(sequence, GPU_BUILD_STEP)
        outcome, note_text = gpu_classify_build(exit_code, build_log)
        if outcome is not None:
            record(outcome, note_text=note_text)
            return GpuOutcome(outcome, "", note_text), seconds["build"], seconds["test"]
        os.remove(build_log)
        exit_code, took = gpu_pass.run(gpu_pass.test_step, sequence, labels)
        seconds["test"] += took
        test_log = runner.log_path(sequence, GPU_TEST_STEP)
        result = gpu_classify_test(exit_code, test_log)
        if result.outcome == GPU_NOT_RUN:
            return result, seconds["build"], seconds["test"]
        if result.outcome == "missed":
            os.remove(test_log)
            record("missed", coverage_key=need["coverers"], note_text=result.note)
        elif result.outcome == "caught":
            test_key, why = judging.catching_key(result.caught_by)
            if test_key:
                record("caught", result.caught_by, test_key, note_text=result.note)
            else:
                # 抓到它的测试认不出（同 CPU 那一趟）：每趟现判，另记覆盖者键给只读的 --summary 核
                record("caught", result.caught_by, "", need["coverers"], note_text=f"catching-test-unresolved:{why}")
        else:
            record(result.outcome, note_text=result.note)
        return result, seconds["build"], seconds["test"]
    finally:
        restore_original(library)


def run_gpu_phase(library, runner, items, arguments):
    """CPU 那一趟全跑完之后的 GPU 那一趟（文件头「GPU 那一趟」）：→ GpuTotals。
    要这一趟的条 = 这一趟罩得住、CPU 那一趟最后落的结论是没红、又有带特性的测试够得着的；键相同、沿用得了的复用；沿用不了的现判。
    开跑先在没改过的树上跑一遍基线；没找到卡：要现判的条逐条打 BROAD_MUTATION_GPU_NOT_RUN（本次未跑，不落行、不算没红）。"""
    outcomes = dict.fromkeys(GPU_ROW_OUTCOMES, 0)

    def totals(needed=0, judged=0, carried=0, not_run=0, failed=False, report=True):
        if report:
            say(f"BROAD_MUTATION_GPU_TOTAL needed={needed} judged={judged} carried={carried} not_run={not_run} "
                + " ".join(f"{SUMMARY_KEYS[name]}={count}" for name, count in outcomes.items()))
        return GpuTotals(needed, judged, carried, not_run, failed, outcomes)

    if stage_two_skipped():
        say("  · GPU 趟接在第二段后面，这一趟的第二段没跑（只供自证的 BROAD_MUTATION_STAGE_ONE_ONLY），不跑")
        return totals(report=False)
    judgement = runner.judgement
    refresh_verdict_rows(judgement)      # CPU 那一趟刚落的行要读进来（单元树、每条的键、测试的闭包照旧用这一趟已经算好的：树里的源码这一趟没变）
    judging = GpuJudging(library, judgement, arguments)
    if not judging.enabled:
        say(f"  · GPU 趟：{GPU_TESTS_DIRECTORY} 下没有文件的 cfg 点到 gpu 特性，没有带 gpu 特性的测试可跑")
        return totals(report=False)
    pending, carried, refused, candidates = [], [], 0, {}
    for item in items:
        if not in_run_scope(item, arguments):
            continue
        cpu = cpu_state_for_gpu(judgement, item)[0]
        if cpu.state != "missed" and not (break_is("gpu-also-when-cpu-caught") and cpu.state == "caught"):
            continue
        current = judgement.key_of(item)
        if current["key"] is None:
            continue
        need = judging.need(current)
        if not need["needed"]:
            continue
        gpu, reason, _deciding = judging.state(current, need, for_run=True)
        if current["refused"]:
            refused += 1
        if gpu.state in DECISIVE_OUTCOMES:
            candidates[item["sequence"]] = gpu.state     # 沿用判据此刻认定论的：复用哨兵的候选（弄坏开关 gpu-carry-dropped 下它们被现判，哨兵就判红）
        if gpu.state in DECISIVE_OUTCOMES and not named_items_rejudged() and not break_is("gpu-carry-dropped"):
            carried.append((item, gpu, need))
        else:
            pending.append((item, need, reason or "no-record"))
    needed = len(pending) + len(carried)
    say(f"BROAD_MUTATION_GPU_PLAN needed={needed} judge={len(pending)} carried={len(carried)} refused={refused} features={','.join(judging.features)}")
    if not needed:
        return totals()
    sentinel = plan_gpu_sentinel(candidates, arguments)
    forced = next((item for item, _gpu, need in carried if sentinel.forced(f"gpu:{item['sequence']}")), None)
    if forced is not None:
        moved = next(entry for entry in carried if entry[0] is forced)
        carried.remove(moved)
        pending.append((forced, moved[2], "sentinel-forced"))
    for item, need, reason in pending:
        say(f"BROAD_MUTATION_GPU_JUDGE sequence={item['sequence']} reason={reason} labels={','.join(need['labels'])}")
    if not pending:
        return totals(needed=needed, carried=len(carried))
    not_run_reason = ""
    gpu_pass = GpuPass(runner, judging)
    available, why = gpu_pass.ensure_baseline(sorted({label for _item, need, _reason in pending for label in need["labels"]}, key=lambda text: (text != "lib", text)))
    if not available:
        not_run_reason = why
    if not_run_reason:
        for item, _need, _reason in pending:
            say(f"BROAD_MUTATION_GPU_NOT_RUN sequence={item['sequence']} reason={not_run_reason.split('：')[0]} name={item['name']}")
        say(f"  · GPU 趟本次未跑 {len(pending)} 条（{not_run_reason}）：它们没有 GPU 那一趟的结论，不算没红，也不算通过；有卡之后重跑同一条命令")
        return totals(needed=needed, carried=len(carried), not_run=len(pending))
    judged_outcomes, not_run_names, stopped_for = {}, [], ""
    for item, need, _reason in pending:
        if stopped_for:
            not_run_names.append(item)
            say(f"BROAD_MUTATION_GPU_NOT_RUN sequence={item['sequence']} reason={stopped_for} name={item['name']}")
            continue
        started = time.monotonic()
        result, build_seconds, test_seconds = judge_gpu_item(library, gpu_pass, item, need)
        if result.outcome == GPU_NOT_RUN:
            stopped_for = result.note.split("：")[0]
            not_run_names.append(item)
            say(f"BROAD_MUTATION_GPU_NOT_RUN sequence={item['sequence']} reason={stopped_for} name={item['name']}")
            continue
        outcomes[result.outcome] = outcomes.get(result.outcome, 0) + 1
        judged_outcomes[f"gpu:{item['sequence']}"] = result.outcome
        cpu = cpu_state_for_gpu(judgement, item)[0]
        combined, _reason = gpu_combined_state(cpu, True, ItemState(result.outcome, False))
        say(f"BROAD_MUTATION_GPU_ITEM sequence={item['sequence']} cpu={cpu.state} gpu={result.outcome} combined={combined.state} build_seconds={build_seconds:.1f} "
            f"test_seconds={test_seconds:.1f} wall_seconds={time.monotonic() - started:.1f} judged=1 red={1 if result.outcome == 'missed' else 0} reused=0 name={item['name']}"
            + (f" first_failing_test={result.caught_by}" if result.caught_by else ""))
    failed = False
    skipped = {f"gpu:{item['sequence']}" for item in not_run_names}      # 没跑成的（卡中途没了）：不当复用也不当现判，哨兵不比
    for key, stored in sentinel.candidates.items():
        if key in skipped:
            continue
        if key not in judged_outcomes:
            sentinel.observe(key, reuse_checks.REUSED, stored)
        elif judged_outcomes[key] in DECISIVE_OUTCOMES:
            sentinel.observe(key, reuse_checks.FORCED if sentinel.forced(key) else reuse_checks.JUDGED, judged_outcomes[key])
    for message, howto in sentinel.finish(printer=say):
        print(f"  ✗ {message}", file=sys.stderr)
        print(f"  → 怎么办：{howto}", file=sys.stderr)
        failed = True
    return totals(needed=needed, judged=sum(outcomes.values()), carried=len(carried), not_run=len(not_run_names), failed=failed)


# ── 选条、列项、汇总 ───────────────────────────────────────────────────────────────────────────────────────

def changed_since_head(source):
    """弄坏开关 select-by-commit 用的旧选法：被判那棵树里相对 HEAD 改过的文件（git diff --name-only HEAD）。"""
    found = subprocess.run(["git", "-C", source, "diff", "--name-only", "HEAD"], capture_output=True, text=True, stdin=subprocess.DEVNULL)
    return set(found.stdout.split()) if found.returncode == 0 else set()


def select_items(judgement, items, arguments, states):
    """交 [(条, 要不要跑第一段, 要不要跑第二段)]，按序号（点名的按点名的次序）。states 是 {序号: (ItemState, 理由, 定下状态的那一行)}。"""
    nothing = (ItemState(NOT_RUN, False), "", None)
    if break_is("rerun-done") or (break_is("no-carry-without-arguments") and not arguments.item):
        states = {}
    forced = set()
    if break_is("select-by-commit"):
        changed = changed_since_head(arguments.source_root)
        forced = {item["sequence"] for item in items if item["file"] in changed}
    want_stage_two = not stage_two_skipped()
    how_to_look = f"python3 {SELF_PATH} --list-items 逐行列「序号、包、变异名、状态」；序号写数字，变异名整条照抄"
    if arguments.item:
        if arguments.start_from is not None or arguments.package:
            refuse("--item 与 --from / --package 同时给了", "点名就只给 --item（可给多次）；按范围跑就不给 --item")
        by_sequence = {str(item["sequence"]): item for item in items}
        by_name = {item["name"]: item for item in items}
        named = []
        for wanted in arguments.item:
            item = by_sequence.get(wanted) or by_name.get(wanted)
            if item is None:
                if break_is("accept-unknown-item"):
                    continue
                refuse(f"不认得的项：{wanted}（此刻清单 {len(items)} 条）", how_to_look)
            named.append(item)
        selected = []
        for item in named:
            state = states.get(item["sequence"], nothing)[0]
            if named_items_rejudged() or break_is("named-items-always-rerun") or item["sequence"] in forced:
                selected.append((item, True, want_stage_two))
            elif state.state in DECISIVE_OUTCOMES:
                say(f"  · 第 {item['sequence']} 条沿用得了（{OUTCOME_TEXT[state.state]}），跳过；键相同的结论不重判，复用哨兵会抽查")
            elif state.state == STAGE_ONE_ONLY:
                if want_stage_two:
                    selected.append((item, False, True))
                else:
                    say(f"  · 第 {item['sequence']} 条已过第一段，这一趟的第二段没跑（只供自证的 BROAD_MUTATION_STAGE_ONE_ONLY），跳过")
            else:
                selected.append((item, True, want_stage_two))
        return selected
    if arguments.package and arguments.package not in MUTATED_PACKAGES:
        refuse(f"--package 给的「{arguments.package}」不在清单的包里", f"只有这几个：{'、'.join(MUTATED_PACKAGES)}；{how_to_look}")
    if arguments.start_from is not None and not 1 <= arguments.start_from <= len(items):
        refuse(f"--from {arguments.start_from} 不在 1..{len(items)} 里", how_to_look)
    selected = []
    for item in items:
        if arguments.start_from is not None and item["sequence"] < arguments.start_from:
            continue
        if arguments.package and item["package"] != arguments.package:
            continue
        state = states.get(item["sequence"], nothing)[0]
        if item["sequence"] in forced:
            selected.append((item, True, want_stage_two))
        elif state.state in (NOT_RUN, DRIVER_ERROR):
            selected.append((item, True, want_stage_two))
        elif state.state == STAGE_ONE_ONLY and want_stage_two:
            selected.append((item, False, True))
    return selected


def item_states(judgement, items):
    return {item["sequence"]: judgement.state(item) for item in items}


def stale_counts(judgement, items):
    """每条在判定库里同一处变异（位置索引相同）而键对不上的判定行数（底稿第二节第 9 条 ⑦：--list-items 报每项键对不上的判定条数）。"""
    counts = {}
    for item in items:
        current = judgement.key_of(item)
        counts[item["sequence"]] = sum(1 for row in judgement.by_locator.get(current["locator"], []) if row["key"] != current["key"])
    return counts


def read_only_inputs(arguments):
    """只读的几种（--list-items、--summary）给了 --toolchain-from / --tools-from 时读进来：→ (工具链文字或 None, 工具版本行或 None)。"""
    toolchain = tools = None
    if arguments.toolchain_from:
        with open(arguments.toolchain_from, encoding="utf-8") as handle:
            toolchain = handle.read().strip() + "\n"
    if arguments.tools_from:
        with open(arguments.tools_from, encoding="utf-8") as handle:
            tools = [line.strip() for line in handle if line.strip()]
    return toolchain, tools


def read_only_judgement(library, arguments, source):
    toolchain, tools = read_only_inputs(arguments)
    judgement = Judgement(library, source, arguments, for_run=False, registered=registered_functions(source), toolchain=toolchain, tools=tools)
    items, unlisted, unrestorable = restore_list(judgement, library.read_list_cache())
    return judgement, items, unlisted, unrestorable


def list_items(library, arguments, source):
    judgement, items, unlisted, _unrestorable = read_only_judgement(library, arguments, source)
    states, stale = item_states(judgement, items), stale_counts(judgement, items)
    states = combined_states(states, gpu_views(library, arguments, judgement, items, states, for_run=False)[1])    # 状态列是两趟合起来的（文件头「GPU 那一趟」）
    for item in items:
        say(f"{item['sequence']}\t{item['package']}\t{item['name']}\t{states[item['sequence']][0].state}\tstale={stale[item['sequence']]}"
            f"\tkey={judgement.key_of(item)['key'] or 'refused'}")
    if unlisted:
        note(f"  · {len(unlisted)} 个单元没记清单（此刻源码上 cargo-mutants 还没列过它们）：例 {unlisted[0].file}:{unlisted[0].first_line} {unlisted[0].name}；"
             f"{SUBMIT_PREFIX} python3 {SELF_PATH} --refresh-list 记上")


def summarise(library, arguments, source):
    """六栏条数与提交门禁那一行（文件头「输出」）；判定库对不上（段数、结局认不出）判红退 1，提交门禁判红退 8。"""
    judgement, items, unlisted, unrestorable = read_only_judgement(library, arguments, source)
    table = judgement.table
    states = item_states(judgement, items)
    judging, views = gpu_views(library, arguments, judgement, items, states, for_run=False)
    states = combined_states(states, views)     # CPU 那一趟没红又要 GPU 那一趟的条，状态取两趟合起来的（文件头「GPU 那一趟」）
    gpu_table = judging.table if judging is not None else read_gpu_verdicts(library)
    gpu_needed = [view for view in views.values() if view.needed]
    gpu_reused = sum(1 for view in gpu_needed if view.gpu.state in DECISIVE_OUTCOMES)
    gpu_not_run = sum(1 for view in gpu_needed if view.gpu.state == NOT_RUN)
    gpu_without = len(gpu_needed) - gpu_reused - gpu_not_run
    gpu_fields = f" gpu_needed={len(gpu_needed)} gpu_reused={gpu_reused} gpu_not_run={gpu_not_run} gpu_without_conclusion={gpu_without}"
    counts = dict.fromkeys(SUMMARY_COLUMNS, 0)
    refused, without, missed_items = [], [], []
    for item in items:
        state, reason, deciding = states[item["sequence"]]
        counts[state.state] = counts.get(state.state, 0) + 1
        forms = judgement.refused_text(item, deciding)
        if forms:
            refused.append((item, forms))
        if state.state not in DECISIVE_OUTCOMES:
            without.append((item, state.state, reason))
        if state.state == "missed":
            missed_items.append(item)
    total = len(items)
    say(f"BROAD_MUTATION_SUMMARY total={total} " + " ".join(f"{SUMMARY_KEYS[column]}={counts[column]}" for column in SUMMARY_COLUMNS)
        + f" refused={len(refused)} unlisted_units={len(unlisted)}{gpu_fields}")
    say("  " + "、".join(f"{OUTCOME_TEXT[column]} {counts[column]}" for column in SUMMARY_COLUMNS) + f"；清单 {total} 条，拒算 {len(refused)} 条，没记清单的单元 {len(unlisted)} 个"
        f"；要 GPU 那一趟的 {len(gpu_needed)} 条（有结论、库里键相同复用 {gpu_reused}，本次未跑 {gpu_not_run}，没结论 {gpu_without}）")
    for item in missed_items:
        say(f"MISSED\t{item['sequence']}\t{item['package']}\t{item['name']}")
    for item in items:
        state, _reason, deciding = states[item["sequence"]]
        if state.state == DRIVER_ERROR:
            last = [row for row in judgement.by_key.get(judgement.key_of(item)["key"], []) if row["outcome"] == DRIVER_ERROR]
            say(f"DRIVER_ERROR\t{item['sequence']}\t{item['package']}\t{item['name']}\t{last[-1]['note'] if last else ''}")
    for item, forms in refused:
        say(f"REFUSED\t{item['sequence']}\t{forms}\t{item['name']}")
    for unit in unlisted:
        say(f"UNLISTED\t{unit.file}:{unit.first_line}\t{unit.kind}\t{unit.name}")
    problems = []
    unknown_outcomes = [row for row in table.rows if row["outcome"] not in ROW_OUTCOMES]
    if unknown_outcomes:
        problems.append(f"{len(unknown_outcomes)} 行是认不出的结局（例：键 {unknown_outcomes[0]['key'][:16]}「{unknown_outcomes[0]['outcome']}」）")
    if table.malformed:
        problems.append(f"判定表里段数不是 {len(VERDICT_COLUMNS)} 的 {table.malformed_text()}")
    unknown_gpu_outcomes = [row for row in gpu_table.rows if row["outcome"] not in GPU_ROW_OUTCOMES]
    if unknown_gpu_outcomes:
        problems.append(f"GPU 判定库里有 {len(unknown_gpu_outcomes)} 行是认不出的结局（例：键 {unknown_gpu_outcomes[0]['key'][:16]}「{unknown_gpu_outcomes[0]['outcome']}」）")
    if gpu_table.malformed:
        problems.append(f"GPU 判定表里段数不是 {len(GPU_VERDICT_COLUMNS)} 的 {gpu_table.malformed_text()}")
    if unrestorable:
        unit, entry = unrestorable[0]
        problems.append(f"清单按单元存的表里有 {len(unrestorable)} 条在此刻的源码上复原不回去（例：{unit.file} {unit.name} 的「{entry.get('description', '')[:60]}」）：lists/ 里那一条写坏了")
    if sum(counts.values()) != total:
        problems.append(f"六栏加起来 {sum(counts.values())}，清单 {total} 条")
    gate_red = bool(without or counts["missed"] or unlisted or total == 0)
    say(f"BROAD_MUTATION_GATE total={total} concluded={total - len(without)} missed={counts['missed']} without_conclusion={len(without)} refused={len(refused)} "
        f"unlisted_units={len(unlisted)}{gpu_fields} verdict={'red' if gate_red else 'green'}")
    for item, state, reason in without:
        changed = judgement.changed_units(item)
        say(f"NO_CONCLUSION\t{item['sequence']}\t{state}\t{item['name']}\t{reason}" + (f"；变了：{changed}" if changed else ""))
    if problems and not break_is("summary-mismatch-silent"):
        print(f"  ✗ 汇总对不上：{'；'.join(problems)}", file=sys.stderr)
        print(f"  → 怎么办：判定库在 {library.location()}（判定在 verdicts/，GPU 趟的在 gpu-verdicts/）。段数不对、结局认不出的条目不是驱动写的（驱动只写成形的行）："
              "交主 agent 查是谁经客户端写进来的；要清掉就用 scripts/prune-server/prune_server_client.py 的 LibraryStore 把点到的条目改写成成形的行（服务没有删键），"
              "改写之后那几条下一趟照沿用不了现判；"
              f"清单复原不回去的：{SUBMIT_PREFIX} python3 {SELF_PATH} --refresh-list 重新记", file=sys.stderr)
        return EXIT_INCONSISTENT
    if gate_red and not break_is("summary-gate-silent"):
        print(f"  ✗ 提交门禁判红：{len(without)} 条沿用不了、也没有现判的结论（其中要 GPU 那一趟而没有结论的 {gpu_not_run + gpu_without} 条），{counts['missed']} 条没红，"
              f"{len(unlisted)} 个单元没记清单", file=sys.stderr)
        print(f"  → 怎么办：主 agent 照 .claude/skills/crash-test/commit-stages.md 第 2b 步不带选条参数起驱动剪枝跑（{SUBMIT_PREFIX} python3 {SELF_PATH} "
              "：每条按单元键沿用、沿用不了的现判；CPU 那一趟没红又要 GPU 那一趟的条"
              "（NO_CONCLUSION 里理由是 gpu-…）接在最后跑，要在有卡的机器上跑，没卡的机器上它们是本次未跑、不算没红也不算通过），再跑这一道；"
              "没红的加一条能杀它的测试再判；确是等价变异的可写进 .cargo/mutants.toml 的 exclude_re（只那几条不再判），理由写在那一行旁边", file=sys.stderr)
        return EXIT_GATE_RED
    return 0


class ArgumentParser(argparse.ArgumentParser):
    def error(self, message):
        print(f"  ✗ 用法错：{message}", file=sys.stderr)
        print(f"  → 怎么办：python3 {SELF_PATH} --help 看每个参数；项的写法照 --list-items 列的", file=sys.stderr)
        sys.exit(EXIT_USAGE)


def default_threads():
    """--threads 的缺省：SINGLEFS_THREAD_CAP 设了取它，否则每台默认重活线程数（.claude/heavy-thread-default 的登记值）；取不了照用法错退，不退回按核数算。"""
    try:
        return default_heavy_threads()
    except HeavyThreadDefaultError as error:
        print(f"  ✗ {error}", file=sys.stderr)
        print(f"  → 怎么办：{error.howto}", file=sys.stderr)
        sys.exit(EXIT_USAGE)


def check_real_run_is_allowed():
    """真跑（起 cargo）之前：工具在不在。"""
    if test_mode():
        return
    missing = [tool for tool in ("cargo", "rustc", "cargo-nextest", "eatmydata", "systemd-run", "systemctl") if shutil.which(tool) is None]
    if missing:
        refuse(f"PATH 上缺：{'、'.join(missing)}", "cargo-nextest 用 cargo install cargo-nextest --locked 装，eatmydata 用系统包管理器装（apt install eatmydata）")


ROCKSDB_PREBUILT_PATH = os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "rocksdb-prebuilt.py")


def multi_host_configuration_for_tests(source):
    """真跑测试那几步交给测试的多机配置（文件头「多机配置交给测试」）：(绝对路径或空串, 在不在)。调用方设了 SINGLEFS_MULTI_HOST_CONFIG 照它，
    没设取 source 的 git common-dir 上一层下的 multi-host.env；取不到 common-dir 交空串（没处找）。"""
    configured = os.environ.get("SINGLEFS_MULTI_HOST_CONFIG", "").strip()
    if configured:
        path = os.path.abspath(configured)
        return path, os.path.isfile(path)
    found = subprocess.run(["git", "-C", source, "rev-parse", "--path-format=absolute", "--git-common-dir"], capture_output=True, text=True, stdin=subprocess.DEVNULL)
    if found.returncode != 0 or not found.stdout.strip():
        return "", False
    path = os.path.join(os.path.dirname(found.stdout.strip()), "multi-host.env")
    return path, os.path.isfile(path)


def check_multi_host_configuration(source, arguments):
    """真跑起任何 cargo 之前判多机配置在不在（文件头「多机配置交给测试」），在就记进 arguments.multi_host_configuration；不在拒（退 2），一步都不起。
    弄坏开关 test-steps-configuration-unpassed 下不判（改前的样子）。"""
    path, present = multi_host_configuration_for_tests(source)
    arguments.multi_host_configuration = path
    if present or break_is("test-steps-configuration-unpassed"):
        return
    refuse(f"多机配置 {path or '（取不到 git common-dir，没处找）'} 不在（{source} 不是 git 工作树时又没设 SINGLEFS_MULTI_HOST_CONFIG 的也算）：真跑测试的每一步都要把它交给测试"
           "（专用树不是 git 工作树，测试自己找不到，crates 那侧找不到就报错），一步都没起",
           "照仓根 multi-host.env.example 在主工作树根写一份 multi-host.env，或设 SINGLEFS_MULTI_HOST_CONFIG 指到它；不补缺省")


def check_rocksdb_entry(source):
    """真跑起任何 cargo 之前（同步树、清单、基线与每一步都在这之后）判这台的 RocksDB 预编库取值（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」）：
    判的是 --source 那棵树（同步进库目录里的就是它）。判红照 entry 原样打红句、拒（退 2），一步都不起；没判红什么都不打。
    弄坏开关 rocksdb-entry-unchecked 下不判（自证 selftest_rocksdb_entry_cells 那一格判错）。"""
    if break_is("rocksdb-entry-unchecked"):
        return
    entry = subprocess.run([sys.executable, ROCKSDB_PREBUILT_PATH, "entry", "--root", source], capture_output=True, text=True, stdin=subprocess.DEVNULL)
    if entry.returncode != 0:
        sys.stderr.write(entry.stdout + entry.stderr)
        refuse("起 cargo 之前 RocksDB 预编库的取值没判过（上面 rocksdb-prebuilt.py entry 的红句），一步都没起",
               '照上面的出路修（开新会话，或命令前 eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"），再跑同一条命令')


class SuspectStreaks:
    """连着几条像是环境造成的结局（文件头 --suspect-streak）：连续无效、连续「测试那一步几乎立刻红」的抓到。到了 K 条就在没改过的树上把那一步重跑一遍。"""

    def __init__(self, runner, arguments):
        self.runner, self.arguments = runner, arguments
        self.unviable, self.fast_caught = [], []

    def saw(self, sequence, row):
        outcome = row.fields[VERDICT_COLUMNS.index("outcome")]
        if break_is("no-environment-check"):
            return
        if row.environment_doubt:
            self.runner.verify_on_pristine_tree([row.step_name], row.environment_doubt, [sequence])
        self.unviable = self.unviable + [(sequence, row.step_name)] if outcome == "unviable" else []
        fast = outcome == "caught" and row.step_seconds < self.arguments.suspect_test_seconds
        self.fast_caught = self.fast_caught + [(sequence, row.step_name)] if fast else []
        for streak, why in ((self.unviable, "无效"), (self.fast_caught, f"测试那一步不到 {self.arguments.suspect_test_seconds} 秒就红的抓到")):
            if len(streak) < self.arguments.suspect_streak:
                continue
            sequences = [found_sequence for found_sequence, _step in streak]
            step_names = list(dict.fromkeys(step_name for _sequence, step_name in streak))
            self.runner.verify_on_pristine_tree(step_names, f"连续 {len(streak)} 条{why}", sequences)
            say(f"  · 连续 {len(streak)} 条{why}（第 {sequences[0]}–{sequences[-1]} 条）：{'、'.join(step_names)} 在没改过的树上照过，不是环境，接着跑")
            streak.clear()


def stop_on_signal(_signal_number, _frame):
    raise Stopped()


def coverage_step(specification_path):
    """--coverage-step：驱动在没改过的树上起的一步。插桩编一遍、列出全部测试，第一段逐条单独一个进程跑、验证包逐个二进制跑（登记的崩溃枚举用例与放量分项经 --skip 排掉）
    （spec 的 known 里那几条覆盖记录照用，不跑），每条（每个二进制）的 profraw 合并后 llvm-cov export，记它执行到的单元（文件, 限定名）；
    写 spec 里的 out：{"tests": 全部测试, "jobs": 现跑的那几条, "failed": 跑红的}；有跑红的退 1。"""
    import concurrent.futures
    with open(specification_path, encoding="utf-8") as handle:
        spec = json.load(handle)
    rules = prune_rules()
    tree = rules.UnitTree(spec["tree"])
    llvm = spec["llvm_bin"]
    known = {tuple(job) for job in spec.get("known", [])}
    profiles = os.path.join(spec["scratch"], "profiles")
    os.makedirs(profiles, exist_ok=True)
    environment = dict(os.environ, CARGO_TARGET_DIR=spec["target"], RUSTFLAGS="-C instrument-coverage", CARGO_TERM_COLOR="never",
                       LLVM_PROFILE_FILE=os.path.join(spec["scratch"], "build-%p-%m.profraw"))

    def build(arguments):
        done = subprocess.run(["cargo", "test", "--locked", "--no-run", "--message-format=json", *arguments], cwd=spec["tree"], env=environment,
                              capture_output=True, text=True, stdin=subprocess.DEVNULL)
        if done.returncode != 0:
            print(done.stderr[-4000:], file=sys.stderr)
            sys.exit(1)
        found = []
        for line in done.stdout.splitlines():
            message = json.loads(line) if line.startswith("{") else {}
            if message.get("reason") == "compiler-artifact" and message.get("executable") and message.get("profile", {}).get("test"):
                package = message["package_id"].split("#")[0].rsplit("/", 1)[-1].split("@")[0]
                kind, name = message["target"]["kind"][0], message["target"]["name"]
                binary = package if kind == "lib" else (f"{package}::bin/{name}" if kind == "bin" else f"{package}::{name}")
                found.append((binary, kind, name, message["executable"], os.path.dirname(message["manifest_path"])))
        return found

    def listed(executable, cwd):
        done = subprocess.run([executable, "--list", "--format", "terse"], cwd=cwd, env=environment,
                              capture_output=True, text=True, stdin=subprocess.DEVNULL)
        return [line[:-len(": test")] for line in done.stdout.splitlines() if line.endswith(": test")]

    def cover(job):
        number, binary, test, executable, cwd, argv, single = job
        directory = os.path.join(profiles, str(number))
        os.makedirs(directory, exist_ok=True)
        child = dict(environment, LLVM_PROFILE_FILE=os.path.join(directory, "%p-%m.profraw"))
        if single:
            child.update({name: "1" for name in ("RUST_TEST_THREADS", "SINGLEFS_THREAD_CAP")})
        done = subprocess.run(argv, cwd=cwd, env=child, capture_output=True, text=True, stdin=subprocess.DEVNULL)
        files = sorted(os.path.join(directory, name) for name in os.listdir(directory) if name.endswith(".profraw"))
        covered = []
        if files:
            merged = os.path.join(directory, "merged.profdata")
            subprocess.run([os.path.join(llvm, "llvm-profdata"), "merge", "-sparse", *files, "-o", merged], check=True, capture_output=True)
            exported = subprocess.run([os.path.join(llvm, "llvm-cov"), "export", executable, f"-instr-profile={merged}"], capture_output=True, text=True, check=True)
            covered = rules.covered_units(json.loads(exported.stdout), tree)
        shutil.rmtree(directory, ignore_errors=True)
        return {"binary": binary, "test": test, "covered": [list(entry) for entry in covered], "profiles": len(files), "exit": done.returncode}

    tests, jobs, harness = [], [], package_arguments(STAGE_ONE_PACKAGES)
    if spec["kind"] == "one":
        for binary, _kind, _name, executable, cwd in build(harness):
            for test in listed(executable, cwd):
                tests.append([binary, test])
                if (binary, test) not in known:
                    jobs.append((len(jobs), binary, test, executable, cwd, [executable, "--exact", test, "--test-threads=1"], True))
    else:
        skips = [word for name in spec.get("skips", []) for word in ("--skip", name)]   # 登记的崩溃枚举用例与放量分项不在这里跑（驱动交下来的名单）
        for _binary, kind, name, executable, cwd in build(["--release", "-p", VERIFICATION_PACKAGE, "--lib", "--tests"]):
            job = ("checker:" + ("lib" if kind == "lib" else name), "*")
            tests.append(list(job))
            if job not in known:
                jobs.append((len(jobs), job[0], "*", executable, cwd, [executable, *skips], False))
    with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, spec["workers"] if spec["kind"] == "one" else 1)) as pool:
        records = list(pool.map(cover, jobs))
    failed = [record for record in records if record["exit"] != 0]
    write_atomically(spec["out"], (json.dumps({"tests": tests, "jobs": records, "failed": failed}, ensure_ascii=False) + "\n").encode())
    say(f"BROAD_MUTATION_COVERAGE kind={spec['kind']} tests={len(tests)} jobs={len(jobs)} failed={len(failed)}")
    for record in failed:
        print(f"  ✗ 覆盖那一遍在没改过的树上跑红：{record['binary']} {record['test']}（退 {record['exit']}）", file=sys.stderr)  # gate-lint:detail
    if failed:
        print("  → 怎么办：没改过的树上本来就红的测试先修好，再重跑驱动；覆盖那一遍不替红的测试记覆盖", file=sys.stderr)
    return 1 if failed else 0


# ── 主流程 ─────────────────────────────────────────────────────────────────────────────────────────────────

def parse_arguments(argv):
    import math
    parser = ArgumentParser(description="广谱变异的逐条驱动；用法与库目录的布局见文件头", add_help=True)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--list-items", action="store_true")
    mode.add_argument("--summary", action="store_true")
    mode.add_argument("--refresh-list", action="store_true")
    mode.add_argument("--selftest", action="store_true")
    mode.add_argument("--coverage-step", metavar="spec")
    parser.add_argument("--source")
    parser.add_argument("--toolchain-from")
    parser.add_argument("--tools-from")
    parser.add_argument("--llvm-bin")
    parser.add_argument("--item", action="append", default=[])
    parser.add_argument("--from", dest="start_from", type=int)
    parser.add_argument("--package")
    parser.add_argument("--stage-two-eatmydata", action="store_true")
    parser.add_argument("--threads", type=int, default=default_threads())
    parser.add_argument("--suspect-streak", type=int, default=DEFAULT_SUSPECT_STREAK)
    parser.add_argument("--suspect-test-seconds", type=float, default=DEFAULT_SUSPECT_TEST_SECONDS)
    parser.add_argument("--stop-grace", type=int, default=DEFAULT_STOP_GRACE_SECONDS)
    parser.add_argument("--lock-wait", type=int, default=DEFAULT_LOCK_WAIT_SECONDS)
    parser.add_argument("--home")
    # 弄坏开关 unknown-option-accepted：认不出的选项悄悄略过（改错的样子）
    arguments = parser.parse_known_args(argv)[0] if break_is("unknown-option-accepted") else parser.parse_args(argv)
    if arguments.threads < 1:
        parser.error("--threads 要是正整数")
    if arguments.suspect_streak < 1:
        parser.error("--suspect-streak 要是正整数")
    if not math.isfinite(arguments.suspect_test_seconds) or arguments.suspect_test_seconds < 0:
        parser.error("--suspect-test-seconds 要是有限的非负数")
    if arguments.stop_grace < 0 or arguments.lock_wait < 0:
        parser.error("--stop-grace 与 --lock-wait 要是非负整数")
    if arguments.coverage_step:
        return arguments
    read_only = arguments.list_items or arguments.summary
    if (arguments.toolchain_from or arguments.tools_from) and not read_only:
        parser.error("--toolchain-from、--tools-from 只给只读的 --list-items / --summary 用（提交门禁的样本）；真跑与写行的用法不认")
    return arguments


def plan_line(items, states, selected):
    judged = refused = carried = 0
    selected_sequences = {item["sequence"] for item, _one, _two in selected}
    for item in items:
        state, reason, _row = states.get(item["sequence"], (ItemState(NOT_RUN, False), "", None))
        if item["sequence"] in selected_sequences:
            if reason.startswith("refused") or reason.startswith("catching-test-unresolved"):
                refused += 1
            else:
                judged += 1
        elif state.state in DECISIVE_OUTCOMES:
            carried += 1
    return judged, carried, refused


def run_items(library, arguments, source):
    runner = Runner(library, arguments)
    judgement = runner.judgement = Judgement(library, library.tree, arguments, for_run=True, registered=runner.registered)
    items = generate_list(library)
    written, units = record_list_cache(library, judgement, items)
    say(f"  · 清单：{len(items)} 条，按单元记进判定库 lists/（{units} 个单元，这一趟新写 {written} 条）")
    interrupted = getattr(arguments, "interrupted", None)
    if interrupted is not None and break_is("interrupted-as-done"):
        # 弄坏开关：被打断的那一条照「已完成」落一行抓到，不重跑
        item = next(item for item in items if item["sequence"] == interrupted["sequence"])
        current = judgement.key_of(item)
        library.record_verdict([current["key"] or "", item["sequence"], item["package"], item["name"], "unviable", "one", "", "", "", "0.0", "0.0", today_text(),
                                "fresh", current["locator"], "", "break"])
        judgement = runner.judgement = Judgement(library, library.tree, arguments, for_run=True, registered=runner.registered)
    states = item_states(judgement, items)
    withdrawn = [item for item in items if states[item["sequence"]][1] == "missed-rejudged"]
    if withdrawn:      # 撤绿的没红行里，要 GPU 趟、GPU 趟又不是没红的，照沿用：交给 Judgement.state 再算一遍这几条
        judgement.gpu_decided = gpu_decided_keys(library, judgement, withdrawn, arguments)
        states.update({item["sequence"]: judgement.state(item) for item in withdrawn})
    selected = select_items(judgement, items, arguments, states)
    sentinel = plan_reuse_sentinel(items, states, arguments)
    forced = next((item for item in items if sentinel.forced(str(item["sequence"]))), None)
    if forced is not None and all(item["sequence"] != forced["sequence"] for item, _one, _two in selected):
        selected = sorted(selected + [(forced, True, not stage_two_skipped())], key=lambda entry: entry[0]["sequence"])
    judged, carried, refused = plan_line(items, states, selected)
    say(f"BROAD_MUTATION_PLAN list={len(items)} judge={judged} carried={carried} refused={refused} selected={len(selected)} threads={arguments.threads}")
    for item, _one, _two in selected:
        _state, reason, _row = states.get(item["sequence"], (None, "selected", None))
        if forced is not None and item["sequence"] == forced["sequence"]:
            reason = "sentinel-forced"
        changed = judgement.changed_units(item)
        say(f"BROAD_MUTATION_JUDGE sequence={item['sequence']} reason={reason or 'selected'}" + (f" changed={changed}" if changed else ""))
    if not selected:
        if finish_reuse_sentinel(sentinel, {}):
            return 1
        gpu_totals = run_gpu_phase(library, runner, items, arguments)     # CPU 那一趟没有要现判的，GPU 那一趟（上一趟没卡、没来得及跑的）照样补
        if gpu_totals.failed:
            return 1
        say(f"  ✓ 选中 0 条要跑：清单 {len(items)} 条，现判 0、沿用 {carried}、拒算 0（--summary 看六栏与提交门禁那一行）；{gpu_totals_text(gpu_totals)}")
        return 0
    stages = [stage for stage, wanted in (("one", any(one for _item, one, _two in selected)), ("two", any(two for _item, _one, two in selected))) if wanted]
    for stage in stages:
        runner.ensure_baseline(stage)
    runner.ensure_coverage(stages)
    runner.verify_environment_at_start(stages)
    ran = dict.fromkeys(ROW_OUTCOMES, 0)
    outcomes = {}
    streaks = SuspectStreaks(runner, arguments)
    started = time.monotonic()
    try:
        for item, with_stage_one, with_stage_two in selected:
            item_started = time.monotonic()
            row = process_item(library, runner, item, with_stage_one, with_stage_two)
            fields = row.fields
            outcome = fields[VERDICT_COLUMNS.index("outcome")]
            ran[outcome] += 1
            outcomes[str(item["sequence"])] = outcome
            # 入口脚本那一行（文件头「输出」）：一条一判、没红的那一条红，复用的条不在这里（它们没现判）
            counts = "" if break_is("item-counts-dropped") else f"judged=1 red={1 if outcome == 'missed' else 0} reused=0 "
            say(f"BROAD_MUTATION_ITEM sequence={item['sequence']} outcome={outcome} stage={fields[VERDICT_COLUMNS.index('stage')]} "
                f"build_seconds={fields[VERDICT_COLUMNS.index('build_seconds')]} test_seconds={fields[VERDICT_COLUMNS.index('test_seconds')]} "
                f"wall_seconds={time.monotonic() - item_started:.1f} {counts}name={item['name']}"
                + (f" first_failing_test={fields[VERDICT_COLUMNS.index('caught_by')]}" if fields[VERDICT_COLUMNS.index('caught_by')] else ""))
            streaks.saw(item["sequence"], row)
    finally:
        library.flush_deferred_rows()
    say(f"BROAD_MUTATION_TOTAL selected={len(selected)} ran={sum(ran.values())} judged={judged} carried={carried} refused={refused} "
        + " ".join(f"{SUMMARY_KEYS[name]}={count}" for name, count in ran.items()) + f" seconds={time.monotonic() - started:.0f}")
    if finish_reuse_sentinel(sentinel, outcomes):
        return 1
    gpu_totals = run_gpu_phase(library, runner, items, arguments)     # CPU 那一趟全跑完之后：没红又有带 gpu 特性的测试够得着的条，补一趟（文件头「GPU 那一趟」）
    if gpu_totals.failed:
        return 1
    say(f"  ✓ 选中的 {len(selected)} 条都判完（{'、'.join(f'{OUTCOME_TEXT[name]} {count}' for name, count in ran.items())}）：现判 {judged}、沿用 {carried}、拒算 {refused}；"
        f"{gpu_totals_text(gpu_totals)}")
    return 0


def gpu_totals_text(totals):
    """成功句里 GPU 那一趟的一段：要这一趟的条数与现判、复用、本次未跑各几条（本次未跑的没有结论，不算没红，也不算通过）。"""
    return f"GPU 趟（要 {totals.needed} 条）：现判 {totals.judged}、复用 {totals.carried}、本次未跑 {totals.not_run}"


def in_run_scope(item, arguments):
    """这一趟选条参数罩得住的条（--item、--from、--package；不带就是全部）：复用哨兵只从这里面抽。"""
    if arguments.item:
        return str(item["sequence"]) in arguments.item or item["name"] in arguments.item
    if arguments.start_from is not None and item["sequence"] < arguments.start_from:
        return False
    if arguments.package and item["package"] != arguments.package:
        return False
    return True


def plan_reuse_sentinel(items, states, arguments):
    """复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「二」）：候选是这一趟罩得住、沿用判据此刻认定论（抓到、没红、无效）的条，库里的判定是那一条的结局；
    自证造「只跑第一段」的现场（BROAD_MUTATION_STAGE_ONE_ONLY）时只取第一段就定了论的（强制现判那一条只跑第一段，比得了）；
    点名的照样现判（BROAD_MUTATION_REJUDGE_NAMED）时不抽。"""
    sentinel = reuse_checks.ReuseSentinel(SCRIPT_KEY, "广谱变异")
    off = "点名的照样现判（BROAD_MUTATION_REJUDGE_NAMED）" if named_items_rejudged() else ""
    candidates = {}
    for item in items if not off else []:
        state, _reason, row = states.get(item["sequence"], (ItemState(NOT_RUN, False), "", None))
        if state.state in DECISIVE_OUTCOMES and in_run_scope(item, arguments) and (not stage_two_skipped() or (row or {}).get("stage") == "one"):
            candidates[str(item["sequence"])] = state.state
    sentinel.plan(candidates, reuse=not off, off_reason=off, printer=say)
    return sentinel


def finish_reuse_sentinel(sentinel, outcomes):
    """哨兵判：outcomes 是这一趟现判的 {序号: 结局}；没在里面的候选就是沿用了。驱动错不是结论，强制现判的那一条撞上它不比。红了交真。"""
    for sequence, stored in sentinel.candidates.items():
        if sequence not in outcomes:
            sentinel.observe(sequence, reuse_checks.REUSED, stored)
        elif outcomes[sequence] in DECISIVE_OUTCOMES:
            sentinel.observe(sequence, reuse_checks.FORCED if sentinel.forced(sequence) else reuse_checks.JUDGED, outcomes[sequence])
    problems = sentinel.finish(printer=say)
    for message, howto in problems:
        print(f"  ✗ {message}", file=sys.stderr)
        print(f"  → 怎么办：{howto}", file=sys.stderr)
    return bool(problems)


def main(argv):
    check_test_only_variables_before_parsing(argv)
    arguments = parse_arguments(argv)
    if arguments.selftest:
        return run_selftest([name for name in os.environ.get(SELFTEST_CELLS_VARIABLE, "").split(",") if name] or None)
    if arguments.coverage_step:
        return coverage_step(arguments.coverage_step)
    read_only = arguments.list_items or arguments.summary
    library = Library(resolve_home(arguments.home, read_only))   # 里面先核只供自证的变量（check_test_only_variables），再核（要写库的用法）库目录落在哪
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
    library.lock_descriptor = acquire_lock(library.home, arguments.lock_wait)
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


# ── 自证 ───────────────────────────────────────────────────────────────────────────────────────────────────

SELFTEST_STEP_DRIVER = r'''#!/usr/bin/env python3
# 假的一步：照 plan.tsv（序号 <TAB> 步名 <TAB> 动作）办，记一笔调用（序号、步名、树里那份源码此刻是不是被改过）。
import os, sys, time
step, sequence, tree = sys.argv[1], sys.argv[2], sys.argv[3]
work = os.environ["BROAD_MUTATION_SELFTEST_WORK"]
with open(os.path.join(work, "multi-host-configuration.log"), "a") as log:   # 收到的多机配置（覆盖那一步也记）
    log.write(f"{sequence}\t{step}\t{os.environ.get('SINGLEFS_MULTI_HOST_CONFIG', '-')}\n")
if step.startswith("coverage-"):
    # 覆盖那一步：照 <工作目录>/<步名>.json 写这一趟的覆盖（没备就写 null：不按覆盖挑，跑全部）
    import json
    spec = json.load(open(os.environ["BROAD_MUTATION_STEP_SPEC"]))
    prepared = os.path.join(work, step + ".json")
    with open(spec["out"], "w") as out:
        out.write(open(prepared).read() if os.path.exists(prepared) else "null\n")
    with open(os.path.join(work, "coverage-known.log"), "a") as log:
        log.write(f"{step}\t{len(spec.get('known', []))}\n")
    sys.exit(0)
with open(os.path.join(work, "selections.log"), "a") as log:
    log.write(f"{sequence}\t{step}\t{os.environ.get('BROAD_MUTATION_STEP_SELECTION', '')}\n")
mutated = any("MUTATED_" in open(os.path.join(tree, path)).read()
              for path in ("crates/singlefs-core/src/lib.rs", "crates/singlefs-checker/src/lib.rs"))
with open(os.path.join(work, "calls.log"), "a") as log:
    log.write(f"{sequence}\t{step}\t{'mutated' if mutated else 'pristine'}\n")
action = "ok"
for line in open(os.path.join(work, "plan.tsv")):
    fields = line.rstrip("\n").split("\t")
    if fields[:2] == [sequence, step]:
        action = fields[2]
if step.startswith("gpu-"):
    # GPU 那一趟的假步（假 GPU）：记下收到的环境与命令（自证核命令的写法、BINDGEN 的取值），测试步的输出照 libtest 的写法打 Running 与 test result
    with open(os.path.join(work, "gpu-env.log"), "a") as log:
        log.write(f"{sequence}\t{step}\t{os.environ.get('BINDGEN_EXTRA_CLANG_ARGS', '-')}\t{os.environ.get('BROAD_MUTATION_STEP_COMMAND', '-')}\n")
    running = "     Running tests/gpu_pair.rs (target/release/deps/gpu_pair-0123456789abcdef)"
    if step == "gpu-test-checker-tier":
        if action in ("ok", "gpu-pass"):
            print(running)
            print("test gpu_case ... ok")
            print("test result: ok. 3 passed; 0 failed")
            sys.exit(0)
        if action == "gpu-no-adapter":
            print(running)
            print("NOT_RUN_NO_GPU_ADAPTER test=gpu_case reason=没有找到 GPU 适配器（自证的假卡）")
            print("test result: ok. 1 passed; 0 failed")
            sys.exit(0)
        if action == "gpu-fail":
            print(running)
            print("test gpu_case ... FAILED")
            print("failures:")
            print("    gpu_case")
            print("test result: FAILED. 0 passed; 1 failed")
            sys.exit(101)
        if action == "gpu-fail-scrambled":
            # --nocapture 加 --test-threads 1：结果行被测试自己的输出打散，认名字靠收尾的 failures: 名单
            print(running)
            print("test gpu_case ... GPU 对拍：第 3 个状态两边不一致")
            print("FAILED")
            print("failures:")
            print("    gpu_case")
            print("test result: FAILED. 0 passed; 1 failed")
            sys.exit(101)
        if action == "gpu-zero":
            print(running)
            print("test result: ok. 0 passed; 0 failed")
            sys.exit(0)
if action == "fail-nextest":
    print("        FAIL [   0.010s] (3/9) singlefs-format::catching the_case_that_catches_it")
    sys.exit(100)
if action.startswith("fail-nextest:"):
    print("        FAIL [   0.010s] (3/9) " + action.split(":", 1)[1])
    sys.exit(100)
if action == "fail-cargo":
    # 头一条红的是 #[should_panic] 的用例：结果行在名字与「 ... 」之间多一段「 - should panic」，记下的名字不该带它；前面没有 Running 行，二进制认不出
    print("test tests::the_stage_two_case - should panic ... FAILED")
    print("test tests::another_stage_two_case ... FAILED")
    sys.exit(101)
if action == "fail-compile":
    print("error[E0277]: the trait bound `Fake: Default` is not satisfied")
    print("  --> crates/singlefs-core/src/lib.rs:2:5")
    print("error: could not compile `singlefs-core` (lib) due to 1 previous error")
    sys.exit(101)
if action == "hang":
    # 挂住，并且自己再分出一个子进程一起挂着（真链上一步底下还有 cargo 与测试进程）；两个的进程号都记下来
    os.fork()
    with open(os.path.join(work, "hang.pids"), "a") as pids:
        pids.write(f"{os.getpid()}\n")
    open(os.path.join(work, "hanging"), "w").close()
    deadline = time.time() + 60
    while time.time() < deadline and not os.path.exists(os.path.join(work, "release")):
        time.sleep(0.05)
    sys.exit(0)
if action == "leave-holder":
    # 漏下一个进程：起一个脱出这一步的 sleep，工作目录在这一趟的树里，进程号记进 <工作目录>/holder.pid，这一步照绿退出
    # （驱动删这一趟的运行目录之前核开着的文件那一格用：应当不删、报出它）
    import subprocess
    holder = subprocess.Popen(["sleep", "60"], cwd=tree, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
    with open(os.path.join(work, "holder.pid"), "w") as out:
        out.write(f"{holder.pid}\n")
    sys.exit(0)
if action.startswith("sleep:"):
    # 慢的一步：先过一条用例（进度行要数到它），再睡那么多秒、绿着退出
    print("        PASS [   0.010s] (1/9) singlefs-format::catching the_case_that_passes", flush=True)
    time.sleep(float(action.split(":")[1]))
    sys.exit(0)
if action in ("cargo-build", "cargo-test"):
    # 假 cargo（每一趟自己的编译目录那一格）：照 cargo 的判法编两个包——fingerprint 不在、或有源文件比上次编出的新，才重编；
    # 测试那一步核编出来的是不是此刻树里的源码，不是就红（过时的产物）
    import hashlib
    target, stale = os.environ["CARGO_TARGET_DIR"], []
    for package in ("singlefs-core", "singlefs-checker"):
        root = os.path.join(tree, "crates", package)
        files = sorted(os.path.join(directory, name) for directory, _names, names in os.walk(root) for name in names)
        content = hashlib.sha256(b"".join(open(path, "rb").read() for path in files)).hexdigest()
        output = os.path.join(target, "debug", ".fingerprint", package + "-0123456789abcdef", "built-from")
        fresh = os.path.isfile(output) and all(os.stat(path).st_mtime_ns <= os.stat(output).st_mtime_ns for path in files)
        if action == "cargo-build" and not fresh:
            os.makedirs(os.path.dirname(output), exist_ok=True)
            with open(output, "w") as handle:
                handle.write(content)
            print(f"   Compiling {package}", flush=True)
            with open(os.path.join(work, "cargo.log"), "a") as log:
                log.write(f"{sequence}\tcompiled\t{package}\n")
        elif action == "cargo-test" and (not os.path.isfile(output) or open(output).read() != content):
            stale.append(package)
    if stale:
        print("        FAIL [   0.010s] (1/1) singlefs-format::catching built_from_stale_" + "_and_".join(stale))
        sys.exit(100)
    sys.exit(0)
if action.startswith("exit:"):
    sys.exit(int(action.split(":")[1]))
sys.exit(0)
'''

SELFTEST_LIST_DRIVER = r'''#!/usr/bin/env python3
# 假的 cargo mutants --list --json --diff：把备好的那一份按包打出来。
import json, os, sys
with open(os.path.join(os.environ["BROAD_MUTATION_SELFTEST_WORK"], "mutants.json")) as handle:
    print(json.dumps([mutant for mutant in json.load(handle) if mutant["package"] == sys.argv[1]]))
'''


def selftest_package(name):
    return f'[package]\nname = "{name}"\nversion = "0.1.0"\nedition = "2021"\n'


SELFTEST_CORE = ("pub fn one() -> u32 {\n    1\n}\npub fn two() -> u32 {\n    2\n}\npub fn three() -> u32 {\n    3\n}\n"
                 "pub fn four() -> u32 {\n    4\n}\npub fn five() -> u32 {\n    5\n}\npub fn six() -> u32 {\n    6\n}\n")
SELFTEST_SOURCES = {
    "Cargo.toml": '[workspace]\nmembers = ["crates/singlefs-core", "crates/singlefs-checker", "crates/singlefs-format"]\nresolver = "2"\n',
    "crates/singlefs-core/Cargo.toml": selftest_package("singlefs-core"),
    "crates/singlefs-core/src/lib.rs": SELFTEST_CORE,
    "crates/singlefs-checker/Cargo.toml": selftest_package("singlefs-checker"),
    "crates/singlefs-checker/src/lib.rs": "pub fn seven() -> u32 {\n    7\n}\npub fn eight() -> u32 {\n    8\n}\n",
    # 标了 #[ignore] 的用例 cargo 不跑、驱动不算（登记的崩溃枚举用例那几格另用 验证包）：lib 里模块中的一条、tests 里顶层的一条；包是 singlefs-format，不碰清单的两个包
    "crates/singlefs-format/Cargo.toml": selftest_package("singlefs-format"),
    "crates/singlefs-format/src/lib.rs": '#[cfg(test)]\nmod tests {\n    #[test]\n    fn in_lib_case() {}\n\n'
                                         '    #[test]\n    #[ignore = "要 NVIDIA 卡"]\n    fn on_the_card() {}\n}\n',
    "crates/singlefs-format/tests/catching_cases.rs": '#[test]\nfn catching_one() {}\n\n#[test]\n#[ignore = "要 NVIDIA 卡"]\nfn on_the_card_in_a_target() {}\n',
    # 抓到它的那条测试：nextest 名字「singlefs-format::catching the_case_that_catches_it」，静态认得出、测试键算得出
    "crates/singlefs-format/tests/catching.rs": "#[test]\nfn the_case_that_catches_it() {\n    assert_eq!(1 + 1, 2);\n}\n",
}
# 序号 → 假的一步怎么答（没写的步答 0）：各种结局，两段各有；第二段的两种测试步各答一次；退 104 是认不出的退出码（记驱动错，第 8 条在第二段，状态是只过了第一段）
SELFTEST_PLAN = [("1", "one-test", "fail-nextest"), ("2", "one-build", "fail-compile"), ("3", "one-test", "exit:104"),
                 ("5", "two-test-doc", "fail-cargo"), ("6", "one-test", "exit:104"), ("7", "two-test-checker-tier", "fail-cargo"),
                 ("8", "two-test-checker-tier", "exit:104")]
SELFTEST_EXPECTED = {"1": ("caught", "one"), "2": ("unviable", "one"), "3": (DRIVER_ERROR, "one"), "4": ("missed", "two"),
                     "5": ("caught", "two"), "6": (DRIVER_ERROR, "one"), "7": ("caught", "two"), "8": (DRIVER_ERROR, "two")}
# 第 5 条（文档测试那一步）与第 7 条（验证包那一步）都抓到了、但 libtest 的日志里没有 Running 行，抓到它的测试认不出，两条都算拒算
SELFTEST_EXPECTED_SUMMARY = "total=8 caught=3 missed=1 unviable=1 stage_one_only=1 driver_error=2 not_run=0 refused=2 unlisted_units=0"
SELFTEST_STAGE_TWO_CASE = "tests::the_stage_two_case"   # fail-cargo 那一步（假 cargo test）头一条红的用例名
SELFTEST_STAGE_TWO_STEPS = ["two-test-doc", "two-build-checker-tier", "two-test-checker-tier"]
SELFTEST_BUILD_STEPS = ("one-build", "two-build-checker-tier", "two-build-debug")   # 编译步：基线键拒算、每趟照跑
SELFTEST_NEXTEST_CASE = "singlefs-format::catching the_case_that_catches_it"
SELFTEST_CATCHING_CASE = "singlefs-format::catching_cases catching_one"
SELFTEST_JUDGING_KEY = "selftest-judging-units"
SELFTEST_GPU_JUDGING_KEY = "selftest-gpu-judging-units"
# 按条记基线那一格（selftest_baseline_entry_cells）另加的源码：第一段测试步一个测试二进制三条、一条起自己这个可执行文件且不点名测试（Command::new(current_exe())，单元闭包拒算）；
# 验证包的库里两条（一条拒算）、另一个测试目标一条
SELFTEST_ENTRY_BASELINE_FILE = "crates/singlefs-format/tests/per_entry.rs"
SELFTEST_ENTRY_BASELINE_SOURCES = {
    "Cargo.toml": SELFTEST_SOURCES["Cargo.toml"].replace('"crates/singlefs-format"]', '"crates/singlefs-format", "crates/singlefs-verification"]'),
    SELFTEST_ENTRY_BASELINE_FILE: "#[test]\nfn kept_first() {\n    assert_eq!(2 + 2, 4);\n}\n\n#[test]\nfn kept_second() {\n    assert_eq!(3 + 3, 6);\n}\n\n"
                                  "#[test]\nfn refused_third() {\n    let _output = std::process::Command::new(std::env::current_exe().unwrap()).output();\n}\n",
    "crates/singlefs-verification/Cargo.toml": selftest_package("singlefs-verification"),
    "crates/singlefs-verification/src/lib.rs": "pub fn nine() -> u32 {\n    9\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn kept_in_the_library() {\n"
                                               "        assert_eq!(super::nine(), 9);\n    }\n\n    #[test]\n    fn refused_in_the_library() {\n"
                                               "        let _output = std::process::Command::new(std::env::current_exe().unwrap()).output();\n    }\n}\n",
    "crates/singlefs-verification/tests/quick.rs": "#[test]\nfn kept_in_a_test_target() {\n    assert_eq!(1 + 1, 2);\n}\n",
}
# GPU 那一趟的格用的工作树（SELFTEST_SOURCES 加一个 验证包）：库里没有测试；tests/gpu_pair.rs 的测试体里有 #[cfg(feature = "gpu")] 的语句（挂 gpu 特性、不带特性也编），
# 里面一条测试经 judge() 够得着 singlefs-core 的 one、two、four、five（序号 1、2、4、5 的变异）；tests/plain_case.rs 不挂特性，只够得着 three（序号 3，CPU 那一趟的测试碰得到、
# 带 gpu 的这一趟帮不上）；six 与 singlefs-checker 的 seven、eight（序号 6、7、8）没有任何测试够得着。
# 不让 gpu_pair.rs 整个文件挂 #![cfg(feature = "gpu")]：这一格测的是测试体里有挂 gpu 的语句的写法；整个文件挂的那种（不带特性时不编，entry-cfg-inactive）
# CPU 那一趟的覆盖者键（mutation-prune.py 的 coverers_key）现在跳过，由 mutation-prune.py 自己的自证管，这一格不去测。
SELFTEST_GPU_FILES = {
    "Cargo.toml": SELFTEST_SOURCES["Cargo.toml"].replace('"crates/singlefs-format"]', '"crates/singlefs-format", "crates/singlefs-verification"]'),
    "crates/singlefs-verification/Cargo.toml": selftest_package("singlefs-verification") + '\n[features]\ngpu = []\n\n[dependencies]\nsinglefs-core = { path = "../singlefs-core" }\n',
    "crates/singlefs-verification/src/lib.rs": "pub fn judge() -> u32 {\n    singlefs_core::one() + singlefs_core::two() + singlefs_core::four() + singlefs_core::five()\n}\n",
    "crates/singlefs-verification/tests/gpu_pair.rs": '#[test]\nfn gpu_case() {\n    #[cfg(feature = "gpu")]\n    let extra = 1;\n    #[cfg(not(feature = "gpu"))]\n    let extra = 0;\n'
                                                      '    assert!(singlefs_verification::judge() + extra > 0);\n}\n',
    "crates/singlefs-verification/tests/plain_case.rs": "#[test]\nfn plain_case() {\n    assert!(singlefs_core::three() > 0);\n}\n",
}
SELFTEST_GPU_CASE = "singlefs-verification::gpu_pair gpu_case"      # 抓到它的测试，nextest 的写法（二进制 id 加用例名）


def selftest_mutants(sources=None):
    """备一份假的原始清单：每个文件里每个返回值各一条，把那个数字换成 MUTATED_<序号>()。"""
    mutants, sequence = [], 0
    for package, path in (("singlefs-core", "crates/singlefs-core/src/lib.rs"), ("singlefs-checker", "crates/singlefs-checker/src/lib.rs")):
        original = (sources or SELFTEST_SOURCES)[path]
        for line_index, line in enumerate(original.split("\n")):
            if not line.strip().isdigit():
                continue
            sequence += 1
            start, end = (line_index + 1, 5), (line_index + 1, len(line) + 1)
            replacement = f"MUTATED_{sequence}()"
            mutated = replace_span(original, start, end, replacement + MUTATION_MARKER)
            diff = "".join(difflib.unified_diff(original.splitlines(keepends=True), mutated.splitlines(keepends=True), f"a/{path}", f"b/{path}"))
            mutants.append({"package": package, "file": path, "name": f"{path}:{line_index + 1}:5: replace the value with {replacement}",
                            "span": {"start": {"line": start[0], "column": start[1]}, "end": {"line": end[0], "column": end[1]}},
                            "replacement": replacement, "diff": diff})
    return mutants


def process_is_running(process_id):
    """进程还在跑（不算已经退出、等着被收的）。"""
    try:
        with open(f"/proc/{process_id}/stat", encoding="utf-8", errors="replace") as handle:
            return handle.read().rsplit(")", 1)[1].split()[0] != "Z"
    except (OSError, IndexError):
        return False


def write_sources(directory, sources):
    for path, text in sources.items():
        os.makedirs(os.path.dirname(os.path.join(directory, path)), exist_ok=True)
        with open(os.path.join(directory, path), "w") as handle:
            handle.write(text)


class Selftest:
    def __init__(self, work):
        self.work = work
        self.cells = 0
        self.failures = 0
        self.driver_failure_lines = []   # 上一格判完以来 run() 起的驱动在完整输出里报的 ✗ 行，check() 判红时整行接在失败行末尾
        self.source = os.path.join(work, "source")
        write_sources(self.source, SELFTEST_SOURCES)
        self.mutants = selftest_mutants()
        self.write_mutants(self.mutants)
        for name, text in ((SELFTEST_STEP_DRIVER_NAME, SELFTEST_STEP_DRIVER), ("list-driver.py", SELFTEST_LIST_DRIVER)):
            with open(os.path.join(work, name), "w") as handle:
                handle.write(text)
            os.chmod(os.path.join(work, name), 0o755)
        self.write_plan(SELFTEST_PLAN)
        # 真跑测试的步交给测试的多机配置（文件头「多机配置交给测试」）：调用方设的不带进来，换成临时目录里的一份；多机配置那几格自己改
        self.multi_host_configuration = os.path.join(work, "multi-host.env")
        with open(self.multi_host_configuration, "w") as handle:
            handle.write("# 自证用的多机配置：假的一步不读它，多机配置那几格只核它的路径到没到测试步的环境里\nENABLE_GPU=0\n")
        self.environment = dict(os.environ, BROAD_MUTATION_SELFTEST_WORK=work, BROAD_MUTATION_STEP_DRIVER="1", SINGLEFS_MULTI_HOST_CONFIG=self.multi_host_configuration,
                                BROAD_MUTATION_LIST_DRIVER=os.path.join(work, "list-driver.py"), BROAD_MUTATION_EXPORT_EVERYTHING="1",
                                BROAD_MUTATION_ALLOW_VOLATILE_HOME="1", BROAD_MUTATION_REGISTERED="",
                                BROAD_MUTATION_JUDGING_KEY=SELFTEST_JUDGING_KEY,
                                BROAD_MUTATION_GPU_JUDGING_KEY=SELFTEST_GPU_JUDGING_KEY)
        # 复用哨兵：别的格比的是现判与沿用的条数，沿用得了的全当哨兵、不抽强制现判；reuse-three-questions 与 reuse-sentinel 两格自己设
        self.environment[reuse_checks.SENTINEL_COUNT_VARIABLE] = "1000"
        self.environment.pop(reuse_checks.SENTINEL_SEED_VARIABLE, None)
        for name in ("BROAD_MUTATION_KILL_AT",
                     "BROAD_MUTATION_TOOLCHAIN", "BROAD_MUTATION_PROGRESS_INTERVAL", "BROAD_MUTATION_TOOL_VERSIONS", "RUSTUP_TOOLCHAIN"):
            self.environment.pop(name, None)

    def write_plan(self, plan):
        with open(os.path.join(self.work, "plan.tsv"), "w") as handle:
            handle.write("".join("\t".join(row) + "\n" for row in plan))

    def write_mutants(self, mutants):
        with open(os.path.join(self.work, "mutants.json"), "w") as handle:
            json.dump(mutants, handle)


    def command(self, home, *arguments, source=None):
        return [sys.executable, SELF_PATH, "--home", os.path.join(self.work, home), "--source", source or self.source, "--lock-wait", "30",
                "--stop-grace", "2", *arguments]

    def run(self, home, *arguments, environment=None, source=None, **extra_environment):
        environment = dict(environment or self.environment, **extra_environment)
        done = subprocess.run(self.command(home, *arguments, source=source), capture_output=True, text=True, stdin=subprocess.DEVNULL, env=environment)
        self.driver_failure_lines += [line.strip() for line in f"{done.stdout}\n{done.stderr}".replace("\r", "\n").split("\n") if "✗" in line]
        return done

    def calls(self):
        """调用记录：[(序号, 步名, mutated / pristine)]。"""
        path = os.path.join(self.work, "calls.log")
        if not os.path.exists(path):
            return []
        with open(path) as handle:
            return [tuple(line.rstrip("\n").split("\t")) for line in handle if line.strip()]

    def called_sequences(self):
        """被调过的序号（基线是 0）；在没改过的树上验环境的那几步不算在里面，编译步的基线（基线键拒算、每趟照跑，文件头「基线」）也不算。"""
        return {call[0] for call in self.calls() if call[0] != ENVIRONMENT_SEQUENCE and not call[1].startswith("gpu-")
                and not (call[0] == str(BASELINE_SEQUENCE) and call[1] in SELFTEST_BUILD_STEPS)}

    def judged_sequences(self):
        """被调过的变异序号（不算基线 0 与验环境；GPU 那一趟的步另在 gpu_calls 里）。"""
        return self.called_sequences() - {str(BASELINE_SEQUENCE)}

    def environment_steps(self):
        return [call[1] for call in self.calls() if call[0] == ENVIRONMENT_SEQUENCE]

    def gpu_calls(self):
        """GPU 那一趟的假步记的 [(序号, 步名, mutated / pristine)]（序号 gpu-baseline 是在没改过的树上的基线）。"""
        return [call for call in self.calls() if call[1].startswith("gpu-")]

    def gpu_judged_sequences(self):
        """GPU 那一趟现判过的变异序号（不算 gpu-baseline）。"""
        return {call[0] for call in self.gpu_calls() if call[0] != GPU_BASELINE_SEQUENCE}

    def gpu_environment(self):
        """GPU 那一趟的假步收到的 [(序号, 步名, BINDGEN_EXTRA_CLANG_ARGS, 命令)]。"""
        path = os.path.join(self.work, "gpu-env.log")
        if not os.path.exists(path):
            return []
        with open(path) as handle:
            return [tuple(line.rstrip("\n").split("\t")) for line in handle if line.strip()]

    def gpu_rows(self, home):
        """这份库的 GPU 判定表里成形的行（{列名: 值}）。"""
        return read_gpu_verdicts(self.library(home)).rows

    def clear_calls(self):
        for name in ("calls.log", "selections.log", "coverage-known.log", "gpu-env.log"):
            path = os.path.join(self.work, name)
            if os.path.exists(path):
                os.remove(path)

    def library(self, home):
        """自证工作目录下 <home> 那份库在 prune-server 里的判定库（自证的库目录都在临时目录底下，各占一份命名空间；文件头「判定库」）。"""
        return Library(os.path.join(self.work, home))

    def append_rows(self, home, table, rows):
        """往这份库的一张表里手写几行（造现场用；同驱动写行一样经客户端）。"""
        self.library(home).append_rows(table, rows)

    @staticmethod
    def copy_library(source_home, destination_home):
        """一份库在服务里的全部条目（各张表、条目名照旧）拷进另一份库的命名空间（两个都是库目录的绝对路径）：交拷了几条。"""
        entries = Library(source_home).scan("")
        if entries:
            Library(destination_home).store.write_many(entries)
        return len(entries)

    def rows(self, home):
        """这份库的判定表里成形的行（{列名: 值}），按写入的次序。"""
        return self.library(home).read_verdicts().rows

    def rows_of(self, home, sequence):
        return [row for row in self.rows(home) if row["sequence"] == str(sequence)]

    def listed(self, home, source=None):
        """--list-items 报的各条：{序号: {state, stale, key, name}}。"""
        listed = self.run(home, "--list-items", source=source)
        found = {}
        for fields in (line.split("\t") for line in listed.stdout.splitlines()):
            if len(fields) == 6:
                found[fields[0]] = {"name": fields[2], "state": fields[3], "stale": fields[4], "key": fields[5].removeprefix("key=")}
        return found

    def states(self, home, source=None):
        return {sequence: entry["state"] for sequence, entry in self.listed(home, source).items()}

    def build_leftovers(self, home):
        """跑完之后库目录里还剩的编译目录与树副本（相对库目录）：runs/ 下的每一个运行目录，与改前放在库目录里的那几样（tree、target、target-coverage、scratch）。"""
        home_path = os.path.join(self.work, home)
        runs = os.path.join(home_path, "runs")
        found = [os.path.join("runs", name) for name in sorted(os.listdir(runs))] if os.path.isdir(runs) else []
        return found + [name for name in ("tree", "target", "target-coverage", "scratch", "scratch-coverage") if os.path.lexists(os.path.join(home_path, name))]

    def leftover_run_directories(self, home):
        """库目录 runs/ 下此刻的运行目录（绝对路径）。"""
        runs = os.path.join(self.work, home, "runs")
        return [os.path.join(runs, name) for name in sorted(os.listdir(runs))] if os.path.isdir(runs) else []

    def tree_is_pristine(self, tree, sources=None):
        """tree（一棵专用树的绝对路径）里的源码与自证的源码逐份相同、没有多出来的文件。"""
        if not os.path.isdir(tree):
            return False
        for path, text in (sources or SELFTEST_SOURCES).items():
            with open(os.path.join(tree, path)) as handle:
                if handle.read() != text:
                    return False
        return files_in_tree(tree) == set(sources or SELFTEST_SOURCES)

    def check(self, condition, wrong, howto):
        """判一格。判红时打失败行「✗ 自证：<wrong>」：wrong 里转述的驱动输出截在哪由临时路径长短定，转述里的 ✗ 换成 SELFTEST_TRANSCRIBED_FAILURE_MARK，
        不让截进来的半行另成一行失败行；上一格判完以来 run() 起的驱动在完整输出里报的 ✗ 行整行接在失败行那一行末尾（文件头开关表前那一段「转述」）。"""
        self.cells += 1
        driver_lines, self.driver_failure_lines = self.driver_failure_lines, []
        if not condition:
            self.failures += 1
            head, newline, rest = wrong.replace("✗", SELFTEST_TRANSCRIBED_FAILURE_MARK).partition("\n")
            if driver_lines:
                # 去掉数字之后相同的几行（各趟报的「复用失灵：<项> …种子 <数>」这类）并成一行、注明同样的另几行，再取前 SELFTEST_DRIVER_FAILURE_LINES_SHOWN 种：
                #   不让前面几趟的同一句把这一格后面那一句挤出去
                kinds, repeats = [], {}
                for line in driver_lines:
                    key = re.sub(r"[0-9]+", "#", line)
                    if key in repeats:
                        repeats[key] += 1
                    else:
                        repeats[key] = 0
                        kinds.append((key, line))
                shown = [line + (f"（同样的另 {repeats[key]} 行）" if repeats[key] else "") for key, line in kinds[:SELFTEST_DRIVER_FAILURE_LINES_SHOWN]]
                more = len(kinds) - len(shown)
                head += f"；这一格起的驱动报的失败行 {len(driver_lines)} 行：{'；'.join(shown)}{f'；另 {more} 种没列' if more else ''}"
            print(f"  ✗ 自证：{head}{newline}{rest}", file=sys.stderr)
            print(f"     → 怎么办：{howto}", file=sys.stderr)


def wait_until(condition, seconds):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        if condition():
            return True
        time.sleep(0.05)
    return condition()


def summary_field(text, key):
    found = re.search(rf"\b{key}=(\d+)", text)
    return int(found.group(1)) if found else None


def plan_field(text, key, line="BROAD_MUTATION_PLAN"):
    found = re.search(rf"^{line} .*\b{key}=(\d+)", text, re.M)
    return int(found.group(1)) if found else None


def selftest_basic_cells(selftest):
    """那几格基本的：临时库目录拒、生成清单与列项、项写错、点名与两段、杀掉再起、各种结局、汇总、--from、上一趟被 KILL 留下的运行目录开跑先删。"""
    work = selftest.work
    selftest.clear_calls()   # 前一组（拒算那一组）留下的假一步调用记录不带进来：下面三格数的是这一组自己的调用
    # 库目录落在开机会清空的目录里拒（不带只供测试的放行）
    volatile_home = f"/dev/shm/broad-mutation-selftest-{os.getpid()}"
    # 不带任何只供测试的变量：带着的话库目录在自证工作目录之外，会先被「只供自证的变量不碰别的库」那一拒拦下，验不到这一格要验的「开机会清空」
    volatile_environment = {name: value for name, value in selftest.environment.items() if name not in TEST_ONLY_VARIABLES}
    refused = subprocess.run([sys.executable, SELF_PATH, "--home", volatile_home, "--source", selftest.source, "--refresh-list"], capture_output=True, text=True,
                             stdin=subprocess.DEVNULL, env=volatile_environment)
    selftest.check(refused.returncode == EXIT_USAGE and not os.path.exists(volatile_home) and "开机会清空" in refused.stderr,
                   f"库目录落在 /dev/shm 里没拒（退 {refused.returncode}）：{refused.stderr}", "看 resolve_home：/tmp、/run、/dev/shm 与 tmpfs 都要拒")
    # 生成清单、按单元存；--list-items 逐行六段、一步都不跑
    refreshed = selftest.run("first", "--refresh-list")
    listed = selftest.run("first", "--list-items")
    listed_lines = [line.split("\t") for line in listed.stdout.splitlines()]
    selftest.check(refreshed.returncode == 0 and listed.returncode == 0 and len(listed_lines) == len(selftest.mutants)
                   and all(len(fields) == 6 and fields[3] == NOT_RUN and fields[4] == "stale=0" and re.fullmatch(r"key=[0-9a-f]{64}", fields[5]) for fields in listed_lines)
                   and [fields[0] for fields in listed_lines] == [str(index + 1) for index in range(len(selftest.mutants))] and not selftest.calls(),
                   f"--refresh-list / --list-items 不对（退 {refreshed.returncode} / {listed.returncode}，{len(listed_lines)} 行，调了 {len(selftest.calls())} 步）：{refreshed.stderr}{listed.stderr}",
                   "看 generate_list、record_list_cache、restore_list 与 list_items：清单每条一行「序号、包、变异名、状态、stale=、key=」，列项不许起任何一步")
    # --list-items 接 | head（读的一头先关了）：被 SIGPIPE 安静停掉，stderr 一个字都没有
    read_end, write_end = os.pipe()
    os.close(read_end)
    closed_pipe = subprocess.run(selftest.command("first", "--list-items"), stdout=write_end, stderr=subprocess.PIPE, text=True,
                                 stdin=subprocess.DEVNULL, env=selftest.environment)
    os.close(write_end)
    selftest.check(closed_pipe.returncode == -signal.SIGPIPE and closed_pipe.stderr == "",
                   f"--list-items 接 | head 时没照 SIGPIPE 惯例安静退（退 {closed_pipe.returncode}）：{closed_pipe.stderr.strip()[-300:]}",
                   "看 main 里 --list-items 那一支：开头要把 SIGPIPE 设回默认")
    # 只读的 --summary 不写库：库目录落在开机会清空的目录里也照读（共享门禁的 stage-selftest.sh 把样本拷进 mktemp -d 再判）；要写库的用法照拒（上面那一格）
    if os.path.isdir("/dev/shm"):
        shm_work = tempfile.mkdtemp(prefix="broad-mutation-selftest-shm.", dir="/dev/shm")
        try:
            selftest.copy_library(os.path.join(work, "first"), os.path.join(shm_work, "first"))
            shm_environment = {name: value for name, value in selftest.environment.items() if name != "BROAD_MUTATION_ALLOW_VOLATILE_HOME"}
            shm_environment[SELFTEST_WORK_VARIABLE] = shm_work
            shm_summary = subprocess.run([sys.executable, SELF_PATH, "--home", os.path.join(shm_work, "first"), "--source", selftest.source, "--summary"],
                                         capture_output=True, text=True, stdin=subprocess.DEVNULL, env=shm_environment)
            selftest.check(shm_summary.returncode in (0, EXIT_GATE_RED) and "开机会清空" not in shm_summary.stderr and "BROAD_MUTATION_GATE" in shm_summary.stdout,
                           f"库目录落在 /dev/shm 里时只读的 --summary 也拒了（退 {shm_summary.returncode}）：{shm_summary.stderr.strip()[-300:]}",
                           "看 resolve_home：只读的用法（read_only）不写库，不拒开机会清空的目录")
        finally:
            shutil.rmtree(shm_work, ignore_errors=True)
    # 项写错退 2、一步都不跑
    unknown = selftest.run("first", "--item", "no-such-mutant", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.check(unknown.returncode == EXIT_USAGE and not selftest.calls(), f"不认得的项没拒（退 {unknown.returncode}，调了 {len(selftest.calls())} 步）",
                   "看 select_items：清单里找不到的序号或变异名要退 2 并写怎么查")
    # 点名（一条按序号、一条按变异名）只跑第一段：只有基线与这两条被调，第二段的步一次都没调；基线在原样的树上、每条在改过的树上跑（判完拷回原件，下一条才对得上）
    name_of_seven = listed_lines[6][2] if len(listed_lines) >= 7 and len(listed_lines[6]) == 6 else "?"
    named = selftest.run("first", "--item", "1", "--item", name_of_seven, BROAD_MUTATION_STAGE_ONE_ONLY="1")
    calls = selftest.calls()
    rows = {row["sequence"]: row for row in selftest.rows("first")}
    selftest.check(named.returncode == 0 and selftest.called_sequences() == {"0", "1", "7"} and not any(call[1].startswith("two-") for call in calls)
                   and all(call[2] == ("pristine" if call[0] == "0" else "mutated") for call in calls)
                   and set(rows) == {"1", "7"} and (rows["1"]["outcome"], rows["1"]["stage"], rows["1"]["caught_by"]) == ("caught", "one", SELFTEST_NEXTEST_CASE)
                   and re.fullmatch(r"[0-9a-f]{64}", rows["1"]["test_key"]) and (rows["7"]["outcome"], rows["7"]["stage"]) == (STAGE_ONE_ONLY, "one"),
                   f"点名两条只跑第一段不对（退 {named.returncode}；调用 {calls}；行 {sorted((row['sequence'], row['outcome'], row['stage']) for row in rows.values())}）：{named.stderr}",
                   "看 select_items 的点名那一支、process_item 的第一段与 stage_two_skipped（BROAD_MUTATION_STAGE_ONE_ONLY）：基线在没改过的树上跑、每条在改过的树上跑、判完拷回原件；抓到的那一行带测试键")
    # 点名「只过了第一段」的那一条（第二段被打断留下的现场）：只补第二段，第一段的步不再调；第二段里 验证包那一步抓到它，带测试名
    selftest.clear_calls()
    second = selftest.run("first", "--item", name_of_seven)
    calls = selftest.calls()
    seven = [(row["outcome"], row["stage"], row["caught_by"]) for row in selftest.rows_of("first", 7)]
    selftest.check(second.returncode == 0 and selftest.called_sequences() == {"0", "7"} and all(call[1].startswith("two-") for call in calls)
                   and seven == [(STAGE_ONE_ONLY, "one", ""), ("caught", "two", SELFTEST_STAGE_TWO_CASE)],
                   f"只补第二段不对（退 {second.returncode}；调用 {calls}；第 7 条的行 {seven}）：{second.stderr}",
                   "看 select_items：状态是 stage-one-only 的只补第二段、不跑第一段；process_item 第二段哪一步红了就记抓到并带测试名")
    # 跑到一半 KILL：第 3 条的测试挂住时杀驱动。杀之前前两条的行已经落盘；杀之后它那一趟的运行目录留着（树里有改过的文件，在途记录在运行目录里）
    selftest.clear_calls()
    selftest.write_plan([row for row in SELFTEST_PLAN if row[0] != "3"] + [("3", "one-test", "hang")])
    driver = subprocess.Popen(selftest.command("second"), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, stdin=subprocess.DEVNULL, env=selftest.environment)
    hanging = wait_until(lambda: os.path.exists(os.path.join(work, "hanging")) or driver.poll() is not None, 60)
    rows_before_stop = {row["sequence"] for row in selftest.rows("second")}
    driver.kill()   # 只点自己起的这一个进程
    driver.wait()
    selftest.check(hanging and os.path.exists(os.path.join(work, "hanging")) and rows_before_stop == {"1", "2"},
                   f"杀驱动之前，前两条的判定行没有都落盘（落了 {sorted(rows_before_stop)}）", "看 Library.record_verdict：每条判完立刻追加并 fsync，不攒到收尾")
    killed_runs = selftest.leftover_run_directories("second")
    selftest.check(len(killed_runs) == 1 and not selftest.tree_is_pristine(os.path.join(killed_runs[0], "tree"))
                   and os.path.exists(os.path.join(killed_runs[0], "inflight.json")),
                   f"驱动被杀之后它那一趟的运行目录里没有留着改过的树与在途记录（后面几格的前提不成立；runs/ 下 {killed_runs}）",
                   "看 apply_mutation：先存原件、记在途记录，再改文件；Library.begin_run：树与在途记录都在这一趟的运行目录里")
    # 被杀的驱动留下的那一步还活着时锁还占着：不等就退 4、一步都不跑
    selftest.clear_calls()
    locked = selftest.run("second", "--lock-wait", "0")
    selftest.check(locked.returncode == EXIT_LOCKED and not selftest.calls(), f"上一趟留下的进程还占着锁时没拒（退 {locked.returncode}）",
                   "看 acquire_lock 与 run_step 的 pass_fds：锁的描述符要传给每一步")
    # 放开挂住的那一步，重跑同一条命令：沿用得了的条调用 0 次、基线不重跑；被打断的那一条重跑；开跑先删上一趟留下的运行目录，跑完这一趟的也删了
    open(os.path.join(work, "release"), "w").close()
    selftest.write_plan(SELFTEST_PLAN)
    resumed = selftest.run("second")
    rows = selftest.rows("second")
    called = selftest.called_sequences()
    selftest.check(resumed.returncode == 0 and not called & {"0", "1", "2"},
                   f"重起之后沿用得了的条或基线被重跑了（退 {resumed.returncode}；被调的序号 {sorted(called)}）：{resumed.stderr}",
                   "看 select_items 与 Judgement.state：判定库里键相同、沿用判据认的条一条都不选；基线有这把基线键的绿记录的不重跑")
    item_lines = [line for line in resumed.stdout.splitlines() if line.startswith("BROAD_MUTATION_ITEM ")]
    selftest.check(item_lines and all(re.search(r" judged=1 red=[01] reused=0 ", line) for line in item_lines)
                   and all((" red=1 " in line) == (" outcome=missed " in line) for line in item_lines) and any(" outcome=missed " in line for line in item_lines),
                   f"BROAD_MUTATION_ITEM 每条一行没带判了、红了、复用了多少，或 red= 与结局对不上（现判 {sorted(called - {'0'})}）：{item_lines[:3]}",
                   "看 run_items 打 BROAD_MUTATION_ITEM 那一句：每条补 judged=1 red=<没红记 1> reused=0（入口脚本那一行，文件头「输出」）")
    selftest.check("3" in called and [row["outcome"] for row in rows if row["sequence"] == "3"] == [DRIVER_ERROR] and "上一趟被打断在第 3 条" in resumed.stdout,
                   f"被打断的第 3 条没有重跑、或被当成已完成（它的行：{[(row['outcome'], row['stage']) for row in rows if row['sequence'] == '3']}）",
                   "看 Library.begin_run：上一趟留下的运行目录里有在途记录就报被打断在哪一条、删掉那个运行目录，不给被打断的那一条落行")
    selftest.check(not selftest.build_leftovers("second"),
                   f"重起跑完之后库目录里还剩上一趟被 KILL 留下的、或这一趟的运行目录（{selftest.build_leftovers('second')}）",
                   "看 Library.begin_run（开跑先删上一趟留下的）与 main 的 finally 里的 Library.end_run（跑完删这一趟的）")
    os.remove(os.path.join(work, "release"))
    try:
        os.remove(os.path.join(work, "hanging"))
    except FileNotFoundError:
        pass   # 挂住的那一步没起到（前面的格已经判红）：不在这里抛出去，后面的格照判
    # 两段式的结局：抓到（第一段、文档测试、验证包各一）、没红、无效、驱动错（第一段两条、第二段一条），逐条对
    final = {row["sequence"]: (row["outcome"], row["stage"]) for row in rows if row["outcome"] != STAGE_ONE_ONLY}
    five = [row for row in rows if row["sequence"] == "5" and row["outcome"] == "caught"]
    selftest.check(final == SELFTEST_EXPECTED and [row["caught_by"] for row in five] == [SELFTEST_STAGE_TWO_CASE]
                   and five and five[0]["refused"].startswith("catching-test-unresolved") and re.fullmatch(r"[0-9a-f]{64}", five[0]["coverage_key"]),
                   f"两段式的结局不对（第 5 条 {[(row['caught_by'], row['refused'], row['coverage_key'][:12]) for row in five]}）：{sorted(final.items())}",
                   "看 process_item 的 run_stage：第二段全绿才记 missed，抓到要带第一条失败的测试名（libtest 结果行经 lib_libtest_result_lines.py，should panic 那一段不进名字）；"
                   "抓到它的测试认不出（没有 Running / Doc-tests 行）的记 catching-test-unresolved 并另记覆盖者键")
    # 汇总六栏加起来等于清单条数，没红的逐条列名；第 5、7 条（抓到它们的测试认不出）算拒算、照覆盖者键认
    summary = selftest.run("second", "--summary")
    selftest.check(summary.returncode == EXIT_GATE_RED and SELFTEST_EXPECTED_SUMMARY in summary.stdout and "verdict=red" in summary.stdout
                   and len([line for line in summary.stdout.splitlines() if line.startswith("MISSED\t")]) == 1
                   and [line.split("\t")[1] for line in summary.stdout.splitlines() if line.startswith("REFUSED\t")] == ["5", "7"],
                   f"汇总不对（退 {summary.returncode}）：{summary.stdout}{summary.stderr}", "看 summarise 的六栏、refused=、MISSED 与 REFUSED 明细")
    # 第二段只补第一段没跑到的：没红的第 4 条第二段依次调的就是这三步（文档测试、验证包的编译与测试），命令里没有重跑第一段的那一遍
    stage_two_of_four = [call[1] for call in selftest.calls() if call[0] == "4" and call[1].startswith("two-")]
    table = {step[0]: step[3] for step in step_table(argparse.Namespace(threads=3, stage_two_eatmydata=False))}
    commands_are_right = (
        table.get("two-test-doc") == ["cargo", "test", "--locked", "--doc", "-p", "singlefs-core", "-p", "singlefs-checker"]
        and table.get("two-test-checker-tier") == ["cargo", "test", "--locked", "--release", "-p", VERIFICATION_PACKAGE, "--lib", "--tests"]
        and table.get("one-test", [])[:3] == ["eatmydata", "cargo", "nextest"]
        and not any(name.startswith("two-test") and command[-len(table["one-test"]):] == table["one-test"] for name, command in table.items()))
    selftest.check(stage_two_of_four == SELFTEST_STAGE_TWO_STEPS and commands_are_right,
                   f"第二段的步不对：第 4 条调的是 {stage_two_of_four}；各步的命令 {table}",
                   "看 step_table：第二段 = 文档测试（被变异的两个包）+ 验证包的全部测试（登记的崩溃枚举用例与放量分项在起命令时经 --skip 排掉），不重跑第一段那一遍")
    # 汇总对不上必须报：给第 4 条的键追加一行认不出的结局
    four_rows = selftest.rows_of("second", 4)
    if not four_rows:
        # 前面几格红了、第 4 条一行判定都没有：这一格判不了，记判错，不在这里抛出去（后面的格照判）
        selftest.check(False, "判定库里有认不出的结局时汇总没判红（前提不成立：second 里第 4 条一行判定都没有，照上面几格先修）",
                       "看 run_items：second 那几趟要给第 4 条落判定行，这一格拿它最后一行造认不出的结局")
    else:
        four = four_rows[-1]
        selftest.append_rows("second", VERDICTS_TABLE, [[four[column] if column != "outcome" else "no-such-outcome" for column in VERDICT_COLUMNS]])
        mismatched = selftest.run("second", "--summary")
        selftest.check(mismatched.returncode == EXIT_INCONSISTENT, f"判定库里有认不出的结局时汇总没判红（退 {mismatched.returncode}）",
                       "看 summarise 末尾的 problems：对不上要退 1")
    # --from（只跑第一段）
    selftest.clear_calls()
    from_seven = selftest.run("third", "--from", "7", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.check(from_seven.returncode == 0 and selftest.called_sequences() == {"0", "7", "8"},
                   f"--from 7 跑的不是第 7、8 条（退 {from_seven.returncode}；{sorted(selftest.called_sequences())}）", "看 select_items 的 start_from")
    # 上一趟被 KILL 留下的运行目录（树里手改过一份、多出一份没登记的测试文件，编译目录里有旧产物，没有在途记录）：开跑先删掉它，
    # 这一趟在新建的运行目录里照源码换树、照常判，跑完这一趟的也删掉
    selftest.clear_calls()
    stale_run = os.path.join(work, "third", "runs", "run-4242-left-by-a-killed-run")
    stale_tree = os.path.join(stale_run, "tree")
    shutil.copytree(selftest.source, stale_tree, ignore=shutil.ignore_patterns(".git"))
    with open(os.path.join(stale_tree, "crates/singlefs-core/src/lib.rs"), "a") as handle:
        handle.write("// 手改\n")
    extra = os.path.join(stale_tree, "crates/singlefs-core/tests/new_case.rs")
    os.makedirs(os.path.dirname(extra), exist_ok=True)
    with open(extra, "w") as handle:
        handle.write("#[test]\nfn always_red() { panic!() }\n")
    os.makedirs(os.path.join(stale_run, "target", "debug"))
    with open(os.path.join(stale_run, "target", "debug", "stale-product"), "w") as handle:
        handle.write("上一趟编的产物\n")
    dirty = selftest.run("third", "--item", "1", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    third_calls = [call for call in selftest.calls() if call[0] == "1"]
    selftest.check(dirty.returncode == 0 and "1" in selftest.called_sequences() and third_calls and all(call[2] == "mutated" for call in third_calls)
                   and f"删了这一趟的编译目录 {stale_run}" in dirty.stderr and not selftest.build_leftovers("third"),
                   f"上一趟被 KILL 留下的运行目录（手改过的树、多出的测试文件、旧产物）没在开跑时删掉、或这一趟没照常判（退 {dirty.returncode}；"
                   f"调用 {sorted(selftest.called_sequences())}；库目录里还剩 {selftest.build_leftovers('third')}）：{dirty.stdout[-400:]}{dirty.stderr[-300:]}",
                   "看 Library.begin_run：拿到锁之后删 runs/ 下上一趟留下的每一个运行目录（删之前核没有进程开着），再新建这一趟的（弄坏开关 interrupted-run-directory-kept 下不删）")
    return listed_lines


def selftest_state_cells(selftest):
    """一条的状态不取最后一行、点名跳过沿用得了的条（库「first」接着用：第 1 条第一段抓到，第 7 条第二段抓到）。"""
    selftest.clear_calls()
    skipped = selftest.run("first", "--item", "1", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    rows_of_one = selftest.rows_of("first", 1)
    selftest.check(skipped.returncode == 0 and "1" not in selftest.called_sequences() and len(rows_of_one) == 1 and "跳过" in skipped.stdout,
                   f"点名沿用得了的条没跳过（退 {skipped.returncode}；被调的序号 {sorted(selftest.called_sequences())}；第 1 条 {len(rows_of_one)} 行）：{skipped.stdout}",
                   "看 select_items 的点名那一支：沿用得了的条不选，逐条说一句")
    again = selftest.run("first", "--item", "1", BROAD_MUTATION_STAGE_ONE_ONLY="1", BROAD_MUTATION_REJUDGE_NAMED="1")
    selftest.check(again.returncode == 0 and len(selftest.rows_of("first", 1)) == 2,
                   f"点名现判（自证变量 BROAD_MUTATION_REJUDGE_NAMED）没有重跑并追加新行（退 {again.returncode}）：{again.stderr}",
                   "看 select_items：设了 BROAD_MUTATION_REJUDGE_NAMED 的点名不看状态，判定库只追加")
    # 第二段的定论不被后来的 stage-one-only 行降级：第 7 条（第二段抓到）点名重判只跑第一段，状态还是 caught；
    # 之后的续跑照旧现判它（抓到它的测试认不出是拒算，拒算的条每趟现判），续跑本身要成功
    downgrade = selftest.run("first", "--item", "7", BROAD_MUTATION_STAGE_ONE_ONLY="1", BROAD_MUTATION_REJUDGE_NAMED="1")
    last_of_seven = [row["outcome"] for row in selftest.rows_of("first", 7)][-1:]
    selftest.clear_calls()
    resumed = selftest.run("first", "--from", "7", "--package", "singlefs-checker")
    selftest.check(downgrade.returncode == 0 and last_of_seven == [STAGE_ONE_ONLY] and selftest.states("first").get("7") == "caught"
                   and resumed.returncode == 0 and {"7", "8"} <= selftest.called_sequences(),
                   f"第二段的定论被后来的 stage-one-only 行降级了（第 7 条最后一行 {last_of_seven}，状态 {selftest.states('first').get('7')}；续跑里被调的 {sorted(selftest.called_sequences())}）",
                   "看 verdict_state：caught / missed / unviable 取其中最后一行，stage-one-only 行不盖它们")
    # 抓到的条点名重判时一步退了认不出的码、记了 driver-error：状态还是 caught
    selftest.write_plan([("1", "one-test", "exit:104")])
    busy = selftest.run("first", "--item", "1", BROAD_MUTATION_STAGE_ONE_ONLY="1", BROAD_MUTATION_REJUDGE_NAMED="1")
    selftest.write_plan(SELFTEST_PLAN)
    outcomes_of_one = [row["outcome"] for row in selftest.rows_of("first", 1)]
    selftest.check(busy.returncode == 0 and outcomes_of_one == ["caught", "caught", DRIVER_ERROR] and selftest.states("first").get("1") == "caught",
                   f"抓到的条被后来的驱动错行盖掉了（第 1 条各行 {outcomes_of_one}，状态 {selftest.states('first').get('1')}）",
                   "看 verdict_state：driver-error 不是定论，不盖掉前面的 caught / missed / unviable")
    # 认不出的选项（用一个从没用过的名字）：退 2、一步都不调
    selftest.clear_calls()
    rows_before = len(selftest.rows("first"))
    unknown_option = selftest.run("first", "--option-never-used-before")
    selftest.check(unknown_option.returncode == EXIT_USAGE and not selftest.calls() and len(selftest.rows("first")) == rows_before and "--option-never-used-before" in unknown_option.stderr,
                   f"认不出的选项没拒（退 {unknown_option.returncode}，调了 {len(selftest.calls())} 步）：{unknown_option.stderr}",
                   "看 parse_arguments：只登记现有的选项，别的一律走 argparse 的用法错（ArgumentParser.error 退 2）")


def selftest_table_cells(selftest):
    """判定库与清单表：清单表被改坏、认不出的退出码。"""
    # 清单按单元存的表被改坏（第 1 条那个单元的那一行换成原文对不上的一条）：--summary 判红退 1、--list-items 少那一条
    selftest.run("names", "--refresh-list")
    names = selftest.library("names")
    one = next(((unit_key, entries) for unit_key, entries in names.read_list_cache().items() if "MUTATED_1()" in json.dumps(entries, ensure_ascii=False)), None)
    if one is not None:
        entries = one[1]
        entries[0]["original"] = "不是这一段原文"
        names.write_list_cache([[one[0], json.dumps(entries, ensure_ascii=False)]])
    summary = selftest.run("names", "--summary")
    listed = selftest.listed("names")
    selftest.check(one is not None and summary.returncode == EXIT_INCONSISTENT and "复原不回去" in summary.stderr and len(listed) == len(selftest.mutants) - 1,
                   f"清单按单元存的表被改坏了没判红（汇总退 {summary.returncode}；列项 {len(listed)} 条）：{summary.stderr[-300:]}",
                   "看 restore_list 与 summarise 的 unrestorable：表里的一条在此刻的源码上复原不回去要报出来、退 1")
    # cargo-mutants 列了被变异的两个包之外的一条（singlefs-format 里的）：那个单元不记清单，复原不回去，--refresh-list 拒（退 1）
    format_path = "crates/singlefs-format/src/lib.rs"
    format_text = SELFTEST_SOURCES[format_path]
    marker = "fn in_lib_case() {}"
    line_index = format_text.split("\n").index("    " + marker)
    start = (line_index + 1, 5 + len(marker) - 2)
    mutated = replace_span(format_text, start, (line_index + 1, 5 + len(marker)), "{ panic!() }" + MUTATION_MARKER)
    stray = {"package": "singlefs-format", "file": format_path, "name": f"{format_path}:{start[0]}:{start[1]}: replace in_lib_case with panic",
             "span": {"start": {"line": start[0], "column": start[1]}, "end": {"line": start[0], "column": 5 + len(marker)}}, "replacement": "{ panic!() }",
             "diff": "".join(difflib.unified_diff(format_text.splitlines(keepends=True), mutated.splitlines(keepends=True), f"a/{format_path}", f"b/{format_path}"))}
    selftest.write_mutants([dict(stray, package="singlefs-core")] + selftest.mutants)
    try:
        outside = selftest.run("outside-list", "--refresh-list")
    finally:
        selftest.write_mutants(selftest.mutants)
    selftest.check(outside.returncode == EXIT_INCONSISTENT and "复原不回去" in outside.stderr,
                   f"清单里有被变异包之外的一条时 --refresh-list 没拒（退 {outside.returncode}）：{outside.stderr[-300:]}",
                   "看 record_list_cache：清单按单元记进判定库 lists/ 之后照它复原一遍，与 cargo-mutants 给的逐条比，复原不回去就拒（弄坏开关 list-cache-unchecked 下不比）")
    # 一步退了认不出的码：落一行 driver-error、往下走，汇总单列；编译步退 101 而没有编译错误的不算无效；续跑时重试这两条
    selftest.write_plan([("2", "one-build", "exit:101"), ("3", "one-test", "exit:104")])
    selftest.clear_calls()
    first = selftest.run("unknown", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    outcomes = {row["sequence"]: row["outcome"] for row in selftest.rows("unknown")}
    summary = selftest.run("unknown", "--summary")
    doubt_checked = selftest.environment_steps()
    selftest.clear_calls()
    retried = selftest.run("unknown", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.write_plan(SELFTEST_PLAN)
    selftest.check(first.returncode == 0 and outcomes.get("2") == DRIVER_ERROR and outcomes.get("3") == DRIVER_ERROR and outcomes.get("4") == STAGE_ONE_ONLY
                   and len(outcomes) == 8 and summary.returncode == EXIT_GATE_RED and summary_field(summary.stdout, "driver_error") == 2
                   and len([line for line in summary.stdout.splitlines() if line.startswith("DRIVER_ERROR\t")]) == 2
                   and doubt_checked == ["one-build"] and retried.returncode == 0 and selftest.judged_sequences() == {"2", "3"},
                   f"认不出的退出码没处理对（退 {first.returncode}；各条 {sorted(outcomes.items())}；汇总退 {summary.returncode}：{summary.stdout.splitlines()[:1]}；"
                   f"当场验的步 {doubt_checked}；续跑退 {retried.returncode}、被调的序号 {sorted(selftest.judged_sequences())}）：{first.stderr}",
                   "看 process_item 的 run_stage：认不出的码记 driver-error 往下走；编译步退 101 要日志里有带源码位置的编译错误才记 unviable；"
                   "select_items 把 driver-error 的条当沿用不了")


def selftest_environment_cells(selftest):
    """基线绿过之后环境坏了要发现得了：重起先验编译、连着几条无效或秒红的抓到就在原件上重验。"""
    selftest.run("environment", "--item", "1", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.write_plan(SELFTEST_PLAN + [(ENVIRONMENT_SEQUENCE, "one-build", "exit:101")])
    selftest.clear_calls()
    broken = selftest.run("environment", "--item", "3", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.check(broken.returncode == EXIT_BASELINE_RED and selftest.environment_steps() == ["one-build"] and not selftest.called_sequences()
                   and not selftest.rows_of("environment", 3) and "疑似环境坏了" in broken.stderr,
                   f"基线绿过之后编译坏了，重起没发现（退 {broken.returncode}；验的步 {selftest.environment_steps()}；被调的序号 {sorted(selftest.called_sequences())}）：{broken.stderr}",
                   "看 Runner.verify_environment_at_start：这一趟没现跑基线的那一段，开跑前在没改过的树上把编译步各跑一遍")
    # 连续 K 条无效而原件上编译照过：不是环境，接着跑
    selftest.write_plan([])
    selftest.run("streak", "--item", "8", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.write_plan([(str(sequence), "one-build", "fail-compile") for sequence in (1, 2, 3)])
    selftest.clear_calls()
    continued = selftest.run("streak", "--package", "singlefs-core", "--suspect-streak", "3", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    outcomes = {row["sequence"]: row["outcome"] for row in selftest.rows("streak")}
    selftest.check(continued.returncode == 0 and selftest.environment_steps() == ["one-build", "one-build"]
                   and [outcomes.get(str(sequence)) for sequence in (1, 2, 3, 4, 5, 6)] == ["unviable"] * 3 + [STAGE_ONE_ONLY] * 3 and "不是环境" in continued.stdout,
                   f"连续无效而原件上编译照过时没有接着跑（退 {continued.returncode}；验的步 {selftest.environment_steps()}；各条 {sorted(outcomes.items())}）：{continued.stderr}",
                   "看 SuspectStreaks.saw：到了 K 条就在没改过的树上重跑那一步，绿了清掉这一串接着跑")
    # 连续 K 条秒红的抓到而原件上那一步也红：退 6 报疑似环境坏了、列出这 K 条不可信的行，后面的条不跑
    selftest.write_plan([])
    selftest.run("fast", "--item", "8", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.write_plan([(str(sequence), "one-test", "fail-nextest") for sequence in (1, 2, 3, 4)] + [(ENVIRONMENT_SEQUENCE, "one-test", "exit:100")])
    selftest.clear_calls()
    stopped = selftest.run("fast", "--suspect-streak", "3", "--suspect-test-seconds", "30", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.write_plan(SELFTEST_PLAN)
    selftest.check(stopped.returncode == EXIT_BASELINE_RED and selftest.called_sequences() == {"1", "2", "3"} and selftest.environment_steps() == ["one-build", "one-test"]
                   and "这 3 条（第 1、2、3 条）的行是环境坏着判的、不可信" in stopped.stderr,
                   f"连续 3 条秒红的抓到、原件上那一步也红，没有停下报疑似环境坏了（退 {stopped.returncode}；被调的序号 {sorted(selftest.called_sequences())}；"
                   f"验的步 {selftest.environment_steps()}）：{stopped.stderr}",
                   "看 SuspectStreaks.saw 与 Runner.verify_on_pristine_tree：原件上那一步红了退 6，出路里列出这几条不可信的行")


def selftest_sync_cells(selftest):
    """恢复原件的临时件不进树、恢复半途被杀之后下一趟照常判；换了工具链每把键都变。"""
    work = selftest.work
    killed = selftest.run("extras", "--item", "3", BROAD_MUTATION_KILL_AT="restore-after-copy", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    killed_runs = selftest.leftover_run_directories("extras")
    stray = sorted(files_in_tree(os.path.join(killed_runs[0], "tree")) - set(SELFTEST_SOURCES)) if len(killed_runs) == 1 else ["（runs/ 下不是恰好一个运行目录）"]
    restore_beside_tree = len(killed_runs) == 1 and os.path.exists(os.path.join(killed_runs[0], "inflight.restore"))
    rerun = selftest.run("extras", "--item", "3", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.check(killed.returncode == -signal.SIGKILL and not stray and restore_beside_tree and rerun.returncode == 0 and not selftest.build_leftovers("extras"),
                   f"恢复原件半途被杀之后树里多出了文件、临时件不在运行目录里、或重跑没照常判完（被杀那趟退 {killed.returncode}；多出 {stray}；"
                   f"重跑退 {rerun.returncode}；库目录里还剩 {selftest.build_leftovers('extras')}）：{rerun.stderr}",
                   "看 restore_original：临时件建在这一趟的运行目录里（inflight.restore），不建在树里；Library.begin_run 开跑先删上一趟留下的运行目录")
    # 工具链进每一把键：换了工具链，判过的一条都不沿用（沿用 0）；换回来照沿用
    first_toolchain, second_toolchain = "rustc 1.0.0 (selftest a)", "rustc 2.0.0 (selftest b)"
    selftest.write_plan([(str(sequence), "one-test", "fail-nextest") for sequence in range(1, 9)])
    judged = selftest.run("toolchain", BROAD_MUTATION_TOOLCHAIN=first_toolchain, BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.clear_calls()
    changed = selftest.run("toolchain", BROAD_MUTATION_TOOLCHAIN=second_toolchain, BROAD_MUTATION_STAGE_ONE_ONLY="1")
    rejudged = selftest.judged_sequences()
    selftest.clear_calls()
    back = selftest.run("toolchain", BROAD_MUTATION_TOOLCHAIN=first_toolchain, BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.write_plan(SELFTEST_PLAN)
    selftest.check(judged.returncode == 0 and changed.returncode == 0 and rejudged == {str(sequence) for sequence in range(1, 9)} and plan_field(changed.stdout, "carried") == 0
                   and back.returncode == 0 and not selftest.judged_sequences() and plan_field(back.stdout, "carried") == 8,
                   f"工具链没进每一把键（换了工具链现判的 {sorted(rejudged)}、沿用 {plan_field(changed.stdout, 'carried')}；换回来现判的 {sorted(selftest.judged_sequences())}、"
                   f"沿用 {plan_field(back.stdout, 'carried')}）：{changed.stderr[-300:]}",
                   "看 Judgement 的 globals（mutation-prune.py 的 global_lines：rustc -vV 各行进每一把键）")
    # exclude_re（等价变异的出路）走一遍：被判那棵树的 .cargo/mutants.toml 写一条 exclude_re 排掉第 3 条，再跑：清单少那一条（--list-items、--summary 都是 7 条）、
    # 清单整版重判（mutants.toml 解析后的值进清单按单元存的键：lists/ 每个单元重写一条），别的条照沿用（判定的键里没有 mutants.toml，现判 0）。
    # 假清单驱动照 cargo-mutants 的读法在专用树里读 .cargo/mutants.toml、按变异名 re.search 去掉匹配上的。弄坏开关 exclude-re-dropped（换树不拷 .cargo/mutants.toml）下清单照旧 8 条，这一格判错
    excluding_driver = os.path.join(work, "list-driver-exclude-re.py")
    with open(excluding_driver, "w") as handle:
        handle.write("#!/usr/bin/env python3\n"
                     "# 假的 cargo mutants --list --json --diff：照专用树里 .cargo/mutants.toml 的 exclude_re 去掉变异名匹配上的那几条（cargo-mutants 按变异名 re.search）\n"
                     "import json, os, re, sys, tomllib\n"
                     "with open(os.path.join(os.environ['BROAD_MUTATION_SELFTEST_WORK'], 'mutants.json')) as handle:\n"
                     "    mutants = [mutant for mutant in json.load(handle) if mutant['package'] == sys.argv[1]]\n"
                     "patterns = []\n"
                     "if os.path.isfile(os.path.join('.cargo', 'mutants.toml')):\n"
                     "    with open(os.path.join('.cargo', 'mutants.toml'), 'rb') as handle:\n"
                     "        patterns = tomllib.load(handle).get('exclude_re', [])\n"
                     "print(json.dumps([mutant for mutant in mutants if not any(re.search(pattern, mutant['name']) for pattern in patterns)]))\n")
    os.chmod(excluding_driver, 0o755)
    excluded_source = selftest_source_copy(selftest, "exclude-re-source")
    excluded_name = next(mutant["name"] for mutant in selftest.mutants if mutant["replacement"] == "MUTATED_3()")
    selftest.write_plan([(str(sequence), "one-test", "fail-nextest") for sequence in range(1, 9)])
    before_exclusion = selftest.run("exclude-re", source=excluded_source, BROAD_MUTATION_LIST_DRIVER=excluding_driver, BROAD_MUTATION_STAGE_ONE_ONLY="1")
    before_names = sorted(entry["name"] for entry in selftest.listed("exclude-re", excluded_source).values())
    write_sources(excluded_source, {".cargo/mutants.toml": "# 自证：第 3 条当等价变异排掉\nexclude_re = ['MUTATED_3\\(\\)']\n"})
    selftest.clear_calls()
    after_exclusion = selftest.run("exclude-re", source=excluded_source, BROAD_MUTATION_LIST_DRIVER=excluding_driver, BROAD_MUTATION_STAGE_ONE_ONLY="1")
    rejudged_after_exclusion = selftest.judged_sequences()
    after_names = sorted(entry["name"] for entry in selftest.listed("exclude-re", excluded_source).values())
    excluded_summary = selftest.run("exclude-re", "--summary", source=excluded_source)
    selftest.write_plan(SELFTEST_PLAN)
    selftest.clear_calls()
    list_line = re.search(r"清单：(\d+) 条，按单元记进判定库 lists/（(\d+) 个单元，这一趟新写 (\d+) 条）", after_exclusion.stdout)
    selftest.check(before_exclusion.returncode == 0 and len(before_names) == 8 and excluded_name in before_names
                   and after_exclusion.returncode == 0 and after_names == sorted(name for name in before_names if name != excluded_name)
                   and list_line is not None and list_line.group(1) == "7" and list_line.group(2) == list_line.group(3) and not rejudged_after_exclusion
                   and "BROAD_MUTATION_SUMMARY total=7 " in excluded_summary.stdout,
                   f"exclude_re 没排掉那一条、清单没整版重判、或别的条没沿用（头一趟退 {before_exclusion.returncode}、{len(before_names)} 条；写了 exclude_re 之后退 "
                   f"{after_exclusion.returncode}、{len(after_names)} 条、还在不在 {excluded_name in after_names}；清单那一行 {list_line.group(0) if list_line else '（没打）'}；"
                   f"现判的 {sorted(rejudged_after_exclusion)}；--summary {next((line for line in excluded_summary.stdout.splitlines() if line.startswith('BROAD_MUTATION_SUMMARY')), '（没打）')}）："
                   f"{after_exclusion.stdout[-300:]}{after_exclusion.stderr[-300:]}",
                   "看 sync_tree（.cargo/mutants.toml 要随导出范围拷进专用树，cargo-mutants 才读得到 exclude_re）与 record_list_cache（清单按单元存的键带 mutants.toml 解析后的值）")


def selftest_selections(selftest):
    """假一步记的 [(序号, 步名, 跑了哪些)]。"""
    path = os.path.join(selftest.work, "selections.log")
    if not os.path.exists(path):
        return []
    with open(path) as handle:
        return [tuple(line.rstrip("\n").split("\t")) for line in handle if line.strip()]


def selftest_source_copy(selftest, name, changes=None, git=False):
    """另一份被判的工作树：照 SELFTEST_SOURCES 写一份，再照 changes 换掉几份文件；git 为真时先 git init 提交一次（之后的改动是「某提交之后改过」）。"""
    directory = os.path.join(selftest.work, name)
    write_sources(directory, SELFTEST_SOURCES)
    if git:
        quiet = {"stdout": subprocess.DEVNULL, "stderr": subprocess.DEVNULL, "stdin": subprocess.DEVNULL}
        environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        subprocess.run(["git", "init", "-q", directory], check=True, env=environment, **quiet)
        subprocess.run(["git", "-C", directory, "add", "-A"], check=True, env=environment, **quiet)
        subprocess.run(["git", "-C", directory, "-c", "user.email=selftest@invalid", "-c", "user.name=selftest", "commit", "-q", "-m", "自证"], check=True,
                       env=environment, **quiet)
    write_sources(directory, changes or {})
    return directory


def selftest_prune_cells(selftest):
    """剪枝：按覆盖挑测试（C2）、第二段按二进制（E6）、文档测试每次跑（E1）；
    不带参数就是剪枝（沿用数 > 0）、改了一个函数只现判那个函数里的条、所在文件在某提交之后改过别处照样沿用、点名现判（自证变量）判成没红、提交门禁判绿、第二段基线红的出路。"""
    work, core = selftest.work, "crates/singlefs-core/src/lib.rs"
    coverage_one = {"tests": [["fake-package::fake_target", "covers_one"], ["fake-package::fake_target", "covers_two"]],
                    "jobs": [{"binary": "fake-package::fake_target", "test": "covers_one", "covered": [[core, "one"]], "profiles": 1, "exit": 0},
                             {"binary": "fake-package::fake_target", "test": "covers_two", "covered": [[core, "two"]], "profiles": 1, "exit": 0}], "failed": []}
    coverage_two = {"tests": [["checker:tier_one", "*"], ["checker:lib", "*"]],
                    "jobs": [{"binary": "checker:tier_one", "test": "*", "covered": [[core, "one"]], "profiles": 1, "exit": 0},
                             {"binary": "checker:lib", "test": "*", "covered": [], "profiles": 1, "exit": 0}], "failed": []}
    for name, content in (("coverage-one", coverage_one), ("coverage-two", coverage_two)):
        with open(os.path.join(work, name + ".json"), "w") as handle:
            json.dump(content, handle)
    selftest.write_plan([])
    selftest.clear_calls()
    ran = selftest.run("prune", "--item", "1", "--item", "3")
    for name in ("coverage-one", "coverage-two"):
        os.remove(os.path.join(work, name + ".json"))
    picked = {(entry[0], entry[1]): list(entry[2:]) for entry in selftest_selections(selftest)}
    rows = {row["sequence"]: row for row in selftest.rows("prune")}
    expected = {("1", "one-test"): ["1 tests"],
                ("1", "two-test-checker-tier"): ["tier_one"], ("1", "two-test-doc"): ["all"],
                ("3", "two-test-doc"): ["all"], ("0", "one-test"): ["all"]}
    selftest.check(ran.returncode == 0 and all(picked.get(key) == value for key, value in expected.items())
                   and not any(key in picked for key in (("3", "one-test"), ("3", "two-test-checker-tier")))
                   and ("3", "one-build") in picked and rows.get("3", {}).get("outcome") == "missed" and "零覆盖" in rows.get("3", {}).get("note", ""),
                   f"按覆盖挑测试、第二段按二进制、文档测试每次跑不对（退 {ran.returncode}；各步 {sorted(picked.items())}；第 3 条 {rows.get('3')}）：{ran.stderr[-400:]}",
                   "看 Judgement.selection（执行到容器单元的测试、第二段执行到它的二进制）、Runner.ensure_coverage、Runner.command_for（挑空的步不跑，文档测试一直跑）")
    # 不带参数就是剪枝：头一趟全部现判（抓到 6 条、无效 1 条、没红 1 条），第二趟不带任何选条参数：抓到的 6 条、无效的 1 条沿用，没红的第 4 条不沿用、现判（记红、撤绿、下趟现判）。
    # 弄坏开关 missed-carried（没红照旧沿用）下第二趟一条都不现判，这一格判错
    catching = f"fail-nextest:{SELFTEST_CATCHING_CASE}"
    plan = [(str(sequence), "one-test", catching) for sequence in (1, 3, 5, 6, 7, 8)] + [("2", "one-build", "fail-compile")]
    selftest.write_plan(plan)
    pruned_source = selftest_source_copy(selftest, "pruned-source", git=True)
    first = selftest.run("carry", source=pruned_source)
    selftest.clear_calls()
    second = selftest.run("carry", source=pruned_source)
    second_judged = selftest.judged_sequences()
    second_judge_lines = [line for line in second.stdout.splitlines() if line.startswith("BROAD_MUTATION_JUDGE ")]
    selftest.check(first.returncode == 0 and second.returncode == 0 and second_judged == {"4"} and plan_field(second.stdout, "carried") == 7
                   and "现判 1、沿用 7、拒算 0" in second.stdout and any("sequence=4 reason=missed-rejudged" in line for line in second_judge_lines),
                   f"单独一条没红（CPU 一趟）下一趟没现判、或抓到的、无效的没沿用（头一趟退 {first.returncode}；第二趟退 {second.returncode}、现判的 {sorted(second_judged)}（应只有第 4 条）、"
                   f"沿用 {plan_field(second.stdout, 'carried')}（应是 7）；{second_judge_lines}）：{second.stdout[-400:]}{second.stderr[-300:]}",
                   "看 Judgement.state：真跑时同一把键的定论里最后一个是没红的，状态换成沿用不了（理由 missed-rejudged），抓到、无效照 row_holds 沿用；select_items 把沿用不了的现判")
    # 改了一个函数（five 的体）：只有它里面那一条现判；同一份文件在上一次提交之后改过别处，别的函数里的条照样沿用（不按提交选条）
    with open(os.path.join(pruned_source, core), "w") as handle:
        handle.write(SELFTEST_CORE.replace("pub fn five() -> u32 {\n", "pub fn five() -> u32 {\n    let _unused = 0;\n"))
    selftest.write_mutants(selftest_mutants(dict(SELFTEST_SOURCES, **{core: SELFTEST_CORE.replace("pub fn five() -> u32 {\n", "pub fn five() -> u32 {\n    let _unused = 0;\n")})))
    try:
        selftest.clear_calls()
        changed = selftest.run("carry", source=pruned_source)
        changed_lines = [line for line in changed.stdout.splitlines() if line.startswith("BROAD_MUTATION_JUDGE ")]
        selftest.check(changed.returncode == 0 and selftest.judged_sequences() == {"4", "5"} and plan_field(changed.stdout, "carried") == 6
                       and "现判 2、沿用 6、拒算 0" in changed.stdout and any("改了 five" in line for line in changed_lines),
                       f"同一条变异内容没变、判定库里有可沿用的行、所在文件在某提交之后改过别处，没照样沿用（退 {changed.returncode}；现判的 {sorted(selftest.judged_sequences())}；"
                       f"沿用 {plan_field(changed.stdout, 'carried')}；{changed_lines}）：{changed.stderr[-300:]}",
                       "看 select_items：只按单元键选条（键对不上的现判，作废时 BROAD_MUTATION_JUDGE 报变了的单元），不按 git diff 选（弄坏开关 select-by-commit 恢复按提交选条）")
        # 抓到它的那条测试（catching_one）改了函数体：抓到的那六条的测试键都变了，现判；没红的第 4 条每趟现判；无效的一条照沿用
        cases = "crates/singlefs-format/tests/catching_cases.rs"
        with open(os.path.join(pruned_source, cases), "w") as handle:
            handle.write(SELFTEST_SOURCES[cases].replace("fn catching_one() {}", "fn catching_one() {\n    assert_eq!(2 + 2, 4);\n}"))
        selftest.clear_calls()
        retested = selftest.run("carry", source=pruned_source)
        selftest.check(retested.returncode == 0 and selftest.judged_sequences() == {"1", "3", "4", "5", "6", "7", "8"} and plan_field(retested.stdout, "carried") == 1
                       and "catching-test-changed" in retested.stdout,
                       f"抓到它的那条测试改了之后那几条没现判（现判的 {sorted(selftest.judged_sequences())}；沿用 {plan_field(retested.stdout, 'carried')}）：{retested.stdout[-400:]}",
                       "看 Judgement.row_holds 与 mutation-prune.py 的 catching_test_key：抓到的判定要那条测试的测试键（它的单元闭包键）与此刻的相同才沿用")
        # 点名现判（自证变量 BROAD_MUTATION_REJUDGE_NAMED）：第 6 条先抓到，这一回重判成没红（同一把键），只现判它；汇总按没红数（第 4、6 条）
        selftest.write_plan([row for row in plan if row[0] != "6"])
        selftest.clear_calls()
        rerun = selftest.run("carry", "--item", "6", source=pruned_source, BROAD_MUTATION_REJUDGE_NAMED="1")
        rerun_judged = selftest.judged_sequences()
        summary = selftest.run("carry", "--summary", source=pruned_source)
        selftest.check(rerun.returncode == 0 and rerun_judged == {"6"} and summary.returncode == EXIT_GATE_RED and summary_field(summary.stdout, "missed") == 2,
                       f"先抓到的第 6 条点名现判（自证变量）判成没红之后，汇总没按没红数、或这一趟现判的不是只有它（退 {rerun.returncode}；现判的 {sorted(rerun_judged)}）：{rerun.stdout[-400:]}；"
                       f"汇总 {[line for line in summary.stdout.splitlines() if 'GATE' in line]}",
                       "看 select_items（--item 加 BROAD_MUTATION_REJUDGE_NAMED 沿用得了也重判）与 Judgement.state：同一把键最后一个定论是没红，汇总按没红数")
        # 记红、撤绿、下趟现判：第 6 条在库里先抓到（两回）、点名现判这一回同一把键又判没红 ⇒ 红照键记着，前面的抓到撤掉（不会重新冒出来当状态，--summary 已按没红数），
        # 下一趟第 4 条（单独一条没红）与第 6 条（先抓到、同键改判没红）都现判，抓到、无效的 6 条沿用；现判又出没红，再下一趟还是现判这两条。
        # 弄坏开关 missed-carried（没红照旧沿用）下这两趟一条都不现判
        selftest.clear_calls()
        flipped = selftest.run("carry", source=pruned_source)
        flipped_judged = selftest.judged_sequences()
        selftest.clear_calls()
        again_red = selftest.run("carry", source=pruned_source)
        again_red_judged = selftest.judged_sequences()
        selftest.check(flipped.returncode == 0 and flipped_judged == {"4", "6"} and plan_field(flipped.stdout, "carried") == 6 and "现判 2、沿用 6、拒算 0" in flipped.stdout,
                       f"先抓到、同一把键又判没红（CPU 一趟）下一趟没现判、或现判之后没红仍沿用（退 {flipped.returncode}；现判的 {sorted(flipped_judged)}（应是第 4、6 条）；"
                       f"沿用 {plan_field(flipped.stdout, 'carried')}（应是 6）；再下一趟退 {again_red.returncode}、现判的 {sorted(again_red_judged)}）：{flipped.stdout[-300:]}",
                       "看 Judgement.state：同一把键全部定论里最后一个是没红的，不看更早的抓到还认不认；现判又出没红的下一趟照样现判（没红每趟现判，直到判出抓到或无效）")
        selftest.check(again_red.returncode == 0 and again_red_judged == {"4", "6"} and plan_field(again_red.stdout, "carried") == 6,
                       f"现判又出没红的条（CPU 一趟）下一趟没接着现判（退 {again_red.returncode}；现判的 {sorted(again_red_judged)}（应是第 4、6 条）；沿用 {plan_field(again_red.stdout, 'carried')}）：{again_red.stdout[-300:]}",
                       "看 Judgement.state：没红每趟现判，不因为最后两个定论一致就沿用")
    finally:
        selftest.write_mutants(selftest.mutants)
        selftest.write_plan(SELFTEST_PLAN)
    # 全部有结论、没有没红：提交门禁绿
    selftest.write_plan([(str(sequence), "one-test", "fail-nextest") for sequence in range(1, 9)])
    selftest.run("green", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    green = selftest.run("green", "--summary")
    selftest.check(green.returncode == 0 and "verdict=green" in green.stdout and "refused=0" in green.stdout,
                   f"每条都抓到的提交门禁没判绿（退 {green.returncode}）：{green.stdout[-300:]}{green.stderr[-300:]}",
                   "看 summarise：抓到、没红、无效都算有结论，没红为零、没有单元缺清单才绿")
    # 第二段基线红：报拒绝、点名红的测试、给出路
    selftest.write_plan([("0", "two-test-checker-tier", "fail-cargo")])
    red = selftest.run("stage-two-red", "--item", "4")
    selftest.write_plan(SELFTEST_PLAN)
    selftest.check(red.returncode == EXIT_BASELINE_RED and SELFTEST_STAGE_TWO_CASE in red.stderr and "修好之前这一道过不去" in red.stderr and "crash_identity" in red.stderr,
                   f"第二段基线红没报成带出路的拒绝（退 {red.returncode}）：{red.stderr[-400:]}",
                   "看 Runner.refuse_red_baseline 第二段那一支：列出红的测试，出路写已知的那条测试要谁改、修好之前这一道过不去")
    selftest.clear_calls()


def selftest_reuse_cells(selftest):
    """复用三问（reuse-three-questions）与复用哨兵（reuse-sentinel），写法在文件头「复用三问与复用哨兵」。项里除了 CPU 那一趟的 1–8 条，
    还有 GPU 那一趟的（项名 gpu:<序号>：CPU 那一趟没红、又有带 gpu 特性的测试够得着的第 4、5 条），两趟的复用一起问；复用哨兵 CPU 一份、GPU 一份。"""
    core = "crates/singlefs-core/src/lib.rs"
    head = "pub fn five() -> u32 {\n"
    # 第 4、5 条 CPU 一趟没红、GPU 一趟抓到：合起来是抓到，两趟的行都沿用（CPU 一趟的没红行只在 GPU 趟也没红时才算红、每趟现判，文件头「记红、撤绿、下趟现判」）；
    # 要是 GPU 趟也没红，这两条每趟现判，问不了「同一份输入第二趟一条都不现判」
    plan = ([(str(sequence), "one-test", f"fail-nextest:{SELFTEST_CATCHING_CASE}") for sequence in (1, 3, 6, 7, 8)] + [("2", "one-build", "fail-compile")]
            + [(str(sequence), GPU_TEST_STEP, "gpu-fail") for sequence in (4, 5)])
    selftest.write_plan(plan)
    source = selftest_source_copy(selftest, "three-questions-source", SELFTEST_GPU_FILES)
    commented = SELFTEST_CORE.replace(head, head + "    // 只加一行注释\n")
    changed = commented.replace(head, head + "    let _unused = 0;\n")

    def judged_run(text=None):
        if text is not None:
            with open(os.path.join(source, core), "w") as handle:
                handle.write(text)
            selftest.write_mutants(selftest_mutants(dict(SELFTEST_SOURCES, **{core: text})))
        selftest.clear_calls()
        ran = selftest.run("three-questions", source=source)
        return ran, selftest.judged_sequences() | {f"gpu:{sequence}" for sequence in selftest.gpu_judged_sequences()}

    try:
        runs = [judged_run(), judged_run(), judged_run(commented), judged_run(changed)]
        items = sorted(runs[0][1])
        states = [{sequence: reuse_checks.JUDGED if sequence in judged else reuse_checks.REUSED for sequence in items} for _ran, judged in runs]
        # 改 five 的体：CPU 那一趟第 5 条的键变、第 4 条的覆盖者键变（够得着它的 gpu_case 的测试键含 five）；GPU 那一趟第 4、5 条的覆盖者键同样变
        passed, problems = reuse_checks.report_three_questions(SCRIPT_KEY, *states, ["4", "5", "gpu:4", "gpu:5"])
        selftest.check(passed and all(ran.returncode == 0 for ran, _judged in runs),
                       f"{reuse_checks.CELL_NAME}：四趟退 {[ran.returncode for ran, _judged in runs]}，现判的 {[sorted(judged) for _ran, judged in runs]}：{'；'.join(problems)}",
                       "照规范输出那一行 red 的那一问查：① 看 select_items 与 Judgement.state（键相同、沿用判据认的条不选）；② 看 mutation-prune.py 的单元键（注释不进指纹）；"
                       "③ 看容器单元的闭包（只改 five 只现判它里面那一条）")
        # 复用哨兵：库里 8 条都有定论之后，同一个种子照常跑：抽中的那一条强制现判、判出同样的结局、整趟绿；弄坏开关 rerun-done（沿用得了的照样重跑）下判红「复用失灵」
        sentinel_environment = {reuse_checks.SENTINEL_COUNT_VARIABLE: "1", reuse_checks.SENTINEL_SEED_VARIABLE: "5"}
        selftest.clear_calls()
        green = selftest.run("three-questions", source=source, **sentinel_environment)
        forced_lines = re.findall(r"强制现判 1 条：(\S+)", green.stdout)
        green_judged, green_gpu_judged = selftest.judged_sequences(), selftest.gpu_judged_sequences()
        selftest.clear_calls()
        skipped = selftest.run("three-questions", source=source, BROAD_MUTATION_BREAK="rerun-done", **sentinel_environment)
        selftest.clear_calls()
        gpu_skipped = selftest.run("three-questions", source=source, BROAD_MUTATION_BREAK="gpu-carry-dropped", **sentinel_environment)
        selftest.check(green.returncode == 0 and "复用哨兵（广谱变异）：种子 5" in green.stdout and "复用哨兵（广谱变异 GPU 趟）：种子 5" in green.stdout and len(forced_lines) == 2
                       and green_judged == {forced_lines[0]} and forced_lines[1].startswith("gpu:") and green_gpu_judged == {forced_lines[1].removeprefix("gpu:")}
                       and "reason=sentinel-forced" in green.stdout and skipped.returncode == 1 and "复用失灵：" in skipped.stderr
                       and gpu_skipped.returncode == 1 and "复用失灵：gpu:" in gpu_skipped.stderr,
                       f"{reuse_checks.SENTINEL_CELL_NAME}：照常那一趟退 {green.returncode}、抽中 {forced_lines}、现判的 {sorted(green_judged)}、GPU 趟现判的 {sorted(green_gpu_judged)}："
                       f"{green.stdout[-500:]}{green.stderr[-200:]}；CPU 沿用得了也重跑的那一趟退 {skipped.returncode}：{skipped.stderr[-300:]}；"
                       f"GPU 沿用得了也重跑的那一趟退 {gpu_skipped.returncode}：{gpu_skipped.stderr[-300:]}",
                       "看 plan_reuse_sentinel、finish_reuse_sentinel（CPU 那一趟）与 plan_gpu_sentinel、run_gpu_phase 里候选与 observe（GPU 那一趟，项名 gpu:<序号>）：候选是沿用判据认定论的条，"
                       "抽中强制现判的那一条加进选中的条；判法在 scripts/gate/lib_reuse_checks.py 文件头「二」")
    finally:
        selftest.write_mutants(selftest.mutants)
        selftest.write_plan(SELFTEST_PLAN)
        selftest.clear_calls()


def selftest_global_cells(selftest):
    """全局那几行：多登记一条崩溃枚举用例（排除单多一条）、只改 Cargo.toml 的注释，别的条照样沿用；改了构建环境的生效值（RUSTFLAGS），每一条都现判。"""
    selftest.write_plan([(str(sequence), "one-test", f"fail-nextest:{SELFTEST_CATCHING_CASE}") for sequence in range(1, 9)])
    source = selftest_source_copy(selftest, "global-source", SELFTEST_ENTRY_BASELINE_SOURCES)   # 带一个 验证包：登记的那一条在它的 tests/quick.rs 里对得上
    first = selftest.run("globals", source=source, BROAD_MUTATION_STAGE_ONE_ONLY="1")
    cases = []
    selftest.clear_calls()
    excluded = selftest.run("globals", source=source, BROAD_MUTATION_REGISTERED="quick:kept_in_a_test_target", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    cases.append(("多登记一条", excluded, selftest.judged_sequences(), "exclusions-in-global-key"))
    with open(os.path.join(source, "Cargo.toml"), "w") as handle:
        handle.write("# 只改注释：成员与 profile 一样没变\n" + SELFTEST_SOURCES["Cargo.toml"])
    selftest.clear_calls()
    commented = selftest.run("globals", source=source, BROAD_MUTATION_STAGE_ONE_ONLY="1")
    cases.append(("只改 Cargo.toml 注释", commented, selftest.judged_sequences(), "manifest-text-in-global-key"))
    for what, ran, judged, switch in cases:
        selftest.check(first.returncode == 0 and ran.returncode == 0 and not judged and plan_field(ran.stdout, "carried") == 8,
                       f"{what}之后别的条没照样沿用（退 {ran.returncode}；现判的 {sorted(judged)}；沿用 {plan_field(ran.stdout, 'carried')}）：{ran.stderr[-300:]}",
                       f"看 mutation-prune.py 的 global_lines 与 build_environment_lines：排除单、Cargo.toml 原文都不进键（弄坏开关 {switch} 把它放进键）")
    selftest.clear_calls()
    flags = selftest.run("globals", source=source, RUSTFLAGS="-C selftest-flag", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.write_plan(SELFTEST_PLAN)
    selftest.check(flags.returncode == 0 and selftest.judged_sequences() == {str(sequence) for sequence in range(1, 9)} and plan_field(flags.stdout, "carried") == 0,
                   f"改了 RUSTFLAGS 之后判过的条还被沿用（现判的 {sorted(selftest.judged_sequences())}；沿用 {plan_field(flags.stdout, 'carried')}）：{flags.stderr[-300:]}",
                   "看 mutation-prune.py 的 build_environment_lines：编译变量的生效值进每一把键")


JUDGING_CHILD = r'''
import importlib.util, sys
specification = importlib.util.spec_from_file_location("broad_mutation_judging_child", sys.argv[1])
driver = importlib.util.module_from_spec(specification)
specification.loader.exec_module(driver)
print(driver.judging_units_key(sys.argv[2]))
'''


def selftest_judging_cells(selftest):
    """判法单元指纹（文件头「键」）：在一份拷贝上算，只改驱动的注释与文档串不变；改了判结局的函数（verdict_state 的体）就变。由自证起的子进程算（弄坏开关在那里认）。"""
    copy = os.path.join(selftest.work, "judging-copy")
    for path in sorted({path for path, _name in JUDGING_ENTRIES}) + ["scripts/gate/lib_libtest_result_lines.py", "scripts/gate/lib_function_units.py",
                                                                         "scripts/hosts/heavy_threads.py", ".claude/scripts/project_preflight.py"]:
        if os.path.isfile(os.path.join(REPOSITORY, path)):
            os.makedirs(os.path.dirname(os.path.join(copy, path)), exist_ok=True)
            shutil.copyfile(os.path.join(REPOSITORY, path), os.path.join(copy, path))
    driver_copy = os.path.join(copy, "scripts/mutation/broad-mutation.py")

    def key_now():
        done = subprocess.run([sys.executable, "-c", JUDGING_CHILD, SELF_PATH, copy], capture_output=True, text=True, stdin=subprocess.DEVNULL,
                              env={name: value for name, value in selftest.environment.items() if name != "BROAD_MUTATION_JUDGING_KEY"})
        return done.stdout.strip() if done.returncode == 0 else f"退 {done.returncode}：{done.stderr.strip()[-200:]}"
    with open(driver_copy, encoding="utf-8") as handle:
        original = handle.read()
    before = key_now()
    with open(driver_copy, "w", encoding="utf-8") as handle:
        handle.write(original.replace('    """一条在判定库里键相同、沿用判据认的那几行', '    # 自证：只加一行注释\n    """一条在判定库里键相同、沿用判据认的那几行（自证改了文档串）', 1)
                     .replace("\nimport argparse\n", "\n# 自证：文件里再加一行注释\nimport argparse\n", 1))
    commented = key_now()
    with open(driver_copy, "w", encoding="utf-8") as handle:
        handle.write(original.replace("    return state_of_outcomes(outcomes)\n", "    return state_of_outcomes(list(outcomes))\n", 1))
    body = key_now()
    # 沿用不沿用没红的那一句（Judgement.state 里，记红、撤绿、下趟现判）也是判结局的逻辑：改它判法单元指纹变，库里的判定一条都不沿用
    carry_rule = '            if decisive and decisive[-1] == "missed":\n                return ItemState(NOT_RUN, state.passed_stage_one), "missed-rejudged", None\n'
    if original.count(carry_rule) == 1:
        with open(driver_copy, "w", encoding="utf-8") as handle:
            handle.write(original.replace(carry_rule, carry_rule.replace('decisive[-1] == "missed"', 'decisive[-1] != "caught"'), 1))
        rule = key_now()
    else:
        rule = "找不到要改的那一句（Judgement.state 里撤绿没红的那一句改了写法就跟着改这一格）"
    selftest.check(re.fullmatch(r"[0-9a-f]{64}", before) and commented == before,
                   f"只改驱动注释判法单元指纹变了（改前 {before[:16]}、只改注释与文档串之后 {commented[:16]}）",
                   "看 judging_units_key：取 JUDGING_ENTRIES 那几个函数的 python 单元闭包键（lib_function_units.py：注释与文档串不进指纹），不取整份原文（弄坏开关 judge-units-raw-text）")
    selftest.check(re.fullmatch(r"[0-9a-f]{64}", body) and body != before,
                   f"改了判结局的函数（verdict_state 的体）判法单元指纹没变（{before[:16]} → {body[:16]}）",
                   "看 JUDGING_ENTRIES：选条、沿用、结论分类那几个函数都要在里面，它们点到的函数与常量跟着进闭包")
    selftest.check(re.fullmatch(r"[0-9a-f]{64}", rule) and rule != before,
                   f"改了沿用不沿用没红的那一句（Judgement.state 里）判法单元指纹没变（{before[:16]} → {rule[:16]}）",
                   "看 JUDGING_ENTRIES：Judgement.state 要在里面，撤绿没红的规矩写在它里面，不写在闭包之外的函数里（外挂的函数改了库里的判定不作废）")


def selftest_refused_cells(selftest):
    """拒算：容器单元的闭包里有认不出的形态（include_str! 的参数不是字面量），那一条每趟现判、成功那句报拒算条数；别的条照沿用。"""
    core = "crates/singlefs-core/src/lib.rs"
    refused_core = SELFTEST_CORE.replace("pub fn four() -> u32 {\n", "pub fn four() -> u32 {\n    let _ = runtime_text();\n") + \
        'fn runtime_text() -> usize {\n    include_str!(env!("SELFTEST_INCLUDE")).len()\n}\n'
    source = selftest_source_copy(selftest, "refused-source", {core: refused_core})
    selftest.write_mutants(selftest_mutants(dict(SELFTEST_SOURCES, **{core: refused_core})))
    selftest.write_plan([(str(sequence), "one-test", "fail-nextest") for sequence in range(1, 9)])
    try:
        first = selftest.run("refused", source=source, BROAD_MUTATION_STAGE_ONE_ONLY="1")
        selftest.clear_calls()
        second = selftest.run("refused", source=source, BROAD_MUTATION_STAGE_ONE_ONLY="1")
        summary = selftest.run("refused", "--summary", source=source, BROAD_MUTATION_STAGE_ONE_ONLY="1")
    finally:
        selftest.write_mutants(selftest.mutants)
        selftest.write_plan(SELFTEST_PLAN)
    selftest.check(first.returncode == 0 and second.returncode == 0 and selftest.judged_sequences() == {"4"} and "现判 0、沿用 7、拒算 1" in second.stdout
                   and "REFUSED\t4\tinclude-argument" in summary.stdout,
                   f"拒算的条没有每趟现判、或别的条没沿用（第二趟现判的 {sorted(selftest.judged_sequences())}）：{second.stdout[-300:]}；汇总 {summary.stdout[-300:]}",
                   "看 Judgement.state：真跑时键里有拒算形态的条沿用不了（每趟现判，弄坏开关 refused-carried 下照沿用），--summary 逐条列 REFUSED")


def selftest_baseline_threads_cells(selftest):
    """基线按线程数分开记：--threads 2 下跑过一趟，换 --threads 3 之后测试那一步的基线在新线程数下重跑、追加新行
    （取基线不看线程数时它照 2 线程那一行沿用、不重跑）。"""
    selftest.write_plan([("2", "one-test", "fail-nextest")])
    first = selftest.run("threads", "--item", "2", "--threads", "2", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.write_plan([("3", "one-test", "fail-nextest")])
    selftest.clear_calls()
    second = selftest.run("threads", "--item", "3", "--threads", "3", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    rerun_baselines = sorted(call[1] for call in selftest.calls() if call[0] == str(BASELINE_SEQUENCE))
    selftest.write_plan(SELFTEST_PLAN)
    threads = sorted(row["threads"] for row in selftest.library("threads").baseline_rows()
                     if row["step"] == "one-test" and row["outcome"] == "ok")
    selftest.check(first.returncode == 0 and second.returncode == 0 and threads == ["2", "3"] and "one-test" in rerun_baselines,
                   f"换了 --threads 之后基线没在新线程数下另记（头一趟退 {first.returncode}、换线程那一趟退 {second.returncode}；"
                   f"基线表里 one-test 绿行的线程数 {threads}；换线程那一趟重跑的基线 {rerun_baselines}）：{second.stderr[-300:]}",
                   "看 Runner.baseline_rows_of 与 ensure_baseline：基线按（基线键, 步名, 线程数）取最后一行，没有就在这个线程数下重跑、追加一行")


def selftest_baseline_key_cells(selftest):
    """基线键（文件头「基线」）：编译步拒算、每趟照跑；测试步的键按它跑到的测试的单元闭包取——改一个不在任何测试闭包里的函数（core 的 six），
    第一段测试那一步的基线照样沿用、不重跑；改抓到它的那条 harness 测试的函数体，那一步的基线重跑。弄坏开关 baseline-by-packages（包级）下这一格必须判错。"""
    core, catching = "crates/singlefs-core/src/lib.rs", "crates/singlefs-format/tests/catching.rs"
    changed_core = SELFTEST_CORE.replace("pub fn six() -> u32 {\n", "pub fn six() -> u32 {\n    let _unused = 0;\n")
    source = selftest_source_copy(selftest, "baseline-key-source")
    selftest.write_plan([])
    try:
        selftest.clear_calls()
        first = selftest.run("baseline-key", "--item", "1", source=source, BROAD_MUTATION_STAGE_ONE_ONLY="1")
        first_baselines = sorted(call[1] for call in selftest.calls() if call[0] == str(BASELINE_SEQUENCE))
        write_sources(source, {core: changed_core})
        selftest.write_mutants(selftest_mutants(dict(SELFTEST_SOURCES, **{core: changed_core})))
        selftest.clear_calls()
        outside = selftest.run("baseline-key", "--item", "1", source=source, BROAD_MUTATION_STAGE_ONE_ONLY="1", BROAD_MUTATION_REJUDGE_NAMED="1")
        outside_baselines = sorted(call[1] for call in selftest.calls() if call[0] == str(BASELINE_SEQUENCE))
        write_sources(source, {catching: SELFTEST_SOURCES[catching].replace("assert_eq!(1 + 1, 2);", "assert_eq!(2 + 2, 4);")})
        selftest.clear_calls()
        inside = selftest.run("baseline-key", "--item", "1", source=source, BROAD_MUTATION_STAGE_ONE_ONLY="1", BROAD_MUTATION_REJUDGE_NAMED="1")
        inside_baselines = sorted(call[1] for call in selftest.calls() if call[0] == str(BASELINE_SEQUENCE))
    finally:
        selftest.write_mutants(selftest.mutants)
        selftest.write_plan(SELFTEST_PLAN)
    selftest.check((first.returncode, outside.returncode, inside.returncode) == (0, 0, 0) and first_baselines == ["one-build", "one-test"]
                   and outside_baselines == ["one-build"] and "基线 one-build：基线键拒算" in outside.stdout and inside_baselines == ["one-build", "one-test"],
                   f"基线键没按这一步跑到的判定的单元闭包取（头一趟退 {first.returncode}、基线 {first_baselines}；改了不在测试闭包里的 six 之后退 {outside.returncode}、"
                   f"基线 {outside_baselines}；改了那条 harness 测试之后退 {inside.returncode}、基线 {inside_baselines}）：{outside.stdout[-400:]}{inside.stderr[-300:]}",
                   "看 Judgement.baseline_key 与 Runner.baseline_key_of：测试步的键只取它跑到的测试（连同条目级宏调用）各自的单元闭包键、命令与全局那几行，"
                   "不取包里全部单元的指纹、不带步名与 form；编译步拒算，这一趟用 run_token 算一把独有的键照跑")


def selftest_baseline_entry_cells(selftest):
    """按条记基线（文件头「基线」）：第一段测试步里 per_entry 一个测试二进制三条、refused_third 起自己这个可执行文件（闭包拒算）；验证包库里两条
    （一条拒算）、tests/quick.rs 一条。头一趟都照跑、照条记下；改一个不在任何测试闭包里的函数（core 的 six）之后，第一段测试那一步只照跑 refused_third、
    别的三条沿用，验证包只另起只跑拒算那一条的一趟（--exact）；再改 kept_first 的断言，
    第一段测试那一步照跑它与 refused_third。弄坏开关 baseline-refused-step-whole（一条拒算整步照跑）下这一格必须判错。"""
    core = "crates/singlefs-core/src/lib.rs"
    changed_core = SELFTEST_CORE.replace("pub fn six() -> u32 {\n", "pub fn six() -> u32 {\n    let _unused = 0;\n")
    source = selftest_source_copy(selftest, "baseline-entry-source", SELFTEST_ENTRY_BASELINE_SOURCES)

    def baseline_selections():
        """基线那几次假的一步收到的 BROAD_MUTATION_STEP_SELECTION：{步名: [all 或 per-entry 点的那几样（dict）]}。"""
        found = {}
        path = os.path.join(selftest.work, "selections.log")
        if os.path.exists(path):
            with open(path) as handle:
                for line in handle:
                    fields = line.rstrip("\n").split("\t")
                    if len(fields) >= 3 and fields[0] == str(BASELINE_SEQUENCE) and fields[1] in ("one-test", "two-test-checker-tier"):
                        text = fields[2]
                        found.setdefault(fields[1], []).append(json.loads(text[len("per-entry "):]) if text.startswith("per-entry ") else text)
        return found

    selftest.write_plan([])
    try:
        selftest.clear_calls()
        first = selftest.run("baseline-entry", "--item", "4", source=source)
        first_selections = baseline_selections()
        write_sources(source, {core: changed_core})
        selftest.write_mutants(selftest_mutants(dict(SELFTEST_SOURCES, **{core: changed_core})))
        selftest.clear_calls()
        outside = selftest.run("baseline-entry", "--item", "4", source=source, BROAD_MUTATION_REJUDGE_NAMED="1")
        outside_selections = baseline_selections()
        write_sources(source, {SELFTEST_ENTRY_BASELINE_FILE: SELFTEST_ENTRY_BASELINE_SOURCES[SELFTEST_ENTRY_BASELINE_FILE].replace("assert_eq!(2 + 2, 4);", "assert_eq!(4 + 4, 8);")})
        selftest.clear_calls()
        inside = selftest.run("baseline-entry", "--item", "4", source=source, BROAD_MUTATION_REJUDGE_NAMED="1")
        inside_selections = baseline_selections()
    finally:
        selftest.write_mutants(selftest.mutants)
        selftest.write_plan(SELFTEST_PLAN)
    stage_one_binary = "singlefs-format::per_entry"
    checker_parts = [selection.get("part") if isinstance(selection, dict) else selection for selection in first_selections.get("two-test-checker-tier", [])]
    inside_tests = inside_selections.get("one-test", [{}])
    selftest.check((first.returncode, outside.returncode, inside.returncode) == (0, 0, 0)
                   and first_selections.get("one-test") == ["all"] and checker_parts == ["whole", "skip", "exact"]
                   and outside_selections.get("one-test") == [{"tests": [[stage_one_binary, "refused_third"]], "binaries": []}]
                   and outside_selections.get("two-test-checker-tier") == [{"binaries": ["singlefs-verification"], "part": "exact"}]
                   and "按条记基线" in outside.stdout
                   and len(inside_tests) == 1 and isinstance(inside_tests[0], dict)
                   and sorted(test for _binary, test in inside_tests[0].get("tests", [])) == ["kept_first", "refused_third"],
                   f"基线没按条记：一步里收得全的测试没各自沿用、或拒算的那几条没照跑（头一趟退 {first.returncode}、第一段测试步 {first_selections.get('one-test')}、"
                   f"验证包 {checker_parts}；改了不在测试闭包里的 six 之后退 {outside.returncode}、第一段测试步 {outside_selections.get('one-test')}、"
                   f"验证包 {outside_selections.get('two-test-checker-tier')}；改了 kept_first 之后退 {inside.returncode}、"
                   f"第一段测试步 {inside_selections.get('one-test')}）：{outside.stdout[-400:]}{outside.stderr[-300:]}{inside.stderr[-300:]}",
                   "看 Judgement.baseline_entry_plan 与 Runner.ensure_entry_baseline：基线键拒算的测试步按条记（一条测试、验证包一个二进制一份记录，键是各自的单元闭包键），"
                   "找得到记录的沿用、不再跑，只照跑找不到的与拒算的那几条，不整步照跑")


def selftest_service_cells(selftest):
    """判定库只在 prune-server 里（文件头「判定库」）：判完的行在服务里这份库的命名空间底下读得到、条目按写入的次序，库目录里判定库的东西一样都不建；
    服务连不上时只读的 --summary 退 75，不当「库里没有」判红（弄坏开关 unreachable-as-empty-library 下判错）。"""
    import socket
    work = selftest.work
    ran = selftest.run("service", "--item", "1", "--item", "2", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    library = selftest.library("service")
    sequences = [row["sequence"] for row in library.read_verdicts().rows]
    entries = [relative for relative, _value in library.scan(f"{VERDICTS_TABLE}/")]
    home = os.path.join(work, "service")
    # 库目录里只许有运行目录那一套（文件头「库目录」；弄坏开关 build-directory-in-home 下运行目录里的东西直接放在库目录里，也算这一套）
    probe = Library(home)
    probe.place_run_directory(home)
    allowed = {os.path.basename(path) for path in (probe.runs, probe.logs, probe.tree_synced, os.path.join(home, "lock"), probe.tree, probe.target, probe.target_coverage,
                                                   probe.scratch, probe.scratch + "-coverage", probe.inflight_record, probe.inflight_original, probe.inflight_restore)}
    stray = sorted(name for name in os.listdir(home) if name not in allowed) if os.path.isdir(home) else []
    selftest.check(ran.returncode == 0 and sequences == ["1", "2"] and len(entries) == 2 and not stray
                   and library.location().startswith(f"prune-server:{prune_server_client().FAMILY_BROAD_MUTATION}@"),
                   f"判完的行没在服务里这份库读到、不按写入的次序、或库目录里建了判定库的东西（退 {ran.returncode}；读到的序号 {sequences}；条目 {entries}；"
                   f"库目录里多出的 {stray}；位置 {library.location()}）：{ran.stderr[-300:]}",
                   "看 Library.record_verdict、append_rows 与 order_stamp：一行一个条目、条目名按写入的次序升序，经 prune_server_client 写进族 broad-mutation 的那一份，本地不写")
    closed = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    closed.bind(("127.0.0.1", 0))
    address = "127.0.0.1:%d" % closed.getsockname()[1]
    closed.close()
    client = prune_server_client()
    unreachable = selftest.run("service", "--summary", **{client.SOCKET_VARIABLE: os.path.join(work, "no-such-prune-server.sock"),
                                                          client.TCP_ADDRESS_VARIABLE: address, client.RECONNECT_LIMIT_VARIABLE: "2"})
    selftest.check(unreachable.returncode == client.EXIT_UNAVAILABLE and "BROAD_MUTATION_GATE" not in unreachable.stdout,
                   f"判定库服务连不上时 --summary 没退 75（退 {unreachable.returncode}）：{unreachable.stdout[-300:]}{unreachable.stderr[-300:]}",
                   "看 Library.scan 与 Library.read：读判定库失败让客户端的 PruneServerFailure 照它的码（75）退出，不接住、不当「库里没有」往下判"
                   "（弄坏开关 unreachable-as-empty-library 下交空表）")


FRESH_BUILD_COMMIT_TIME_NS = 1_790_000_000 * 10 ** 9    # 自证里的「源码的 mtime」：早于自证里编出的每一份产物


def selftest_fresh_build_cells(selftest):
    """每一趟自己的编译目录（文件头「库目录」runs/）：同一份源码（mtime 早于产物）连跑两趟，第二趟的编译目录是新建的，两个包都从头编，
    不照用上一趟编的产物；两趟跑完库目录里都不剩运行目录、也没有改前放在库目录里的树与编译目录（正常结束那一种结局）。"""
    work = selftest.work
    core_extra = "crates/singlefs-core/src/extra.rs"

    def commit_source(name, version):
        """一份源码副本，每个文件的 mtime 都早于自证里编出的产物。"""
        directory = selftest_source_copy(selftest, name, {core_extra: f"pub const VERSION: u32 = {version};\n"})
        for found_directory, _directory_names, file_names in os.walk(directory):
            for file_name in file_names:
                os.utime(os.path.join(found_directory, file_name), ns=(FRESH_BUILD_COMMIT_TIME_NS, FRESH_BUILD_COMMIT_TIME_NS))
        return directory

    def compiled():
        path = os.path.join(work, "cargo.log")
        if not os.path.exists(path):
            return set()
        with open(path) as handle:
            return {(fields[0], fields[2]) for fields in (line.rstrip("\n").split("\t") for line in handle) if len(fields) == 3}

    selftest.write_plan([("0", "one-build", "cargo-build"), ("0", "one-test", "cargo-test")])
    source = commit_source("fresh-source", 1)
    # 点名的是 checker 里的第 7 条；两趟同一份源码：编译目录要是跨趟留着，第二趟两个包的产物都比源码新，一个都不重编
    first = selftest.run("fresh", "--item", "7", source=source, BROAD_MUTATION_STAGE_ONE_ONLY="1")
    built_before = compiled()
    left_after_first = selftest.build_leftovers("fresh")
    if os.path.exists(os.path.join(work, "cargo.log")):
        os.remove(os.path.join(work, "cargo.log"))
    second = selftest.run("fresh", "--item", "7", source=source, BROAD_MUTATION_STAGE_ONE_ONLY="1", BROAD_MUTATION_REJUDGE_NAMED="1")    # 第 7 条的键没变，点名重跑才起基线
    rebuilt = compiled()
    left_after_second = selftest.build_leftovers("fresh")
    selftest.write_plan(SELFTEST_PLAN)
    both = {("0", "singlefs-core"), ("0", "singlefs-checker")}
    selftest.check(first.returncode == 0 and both <= built_before and second.returncode == 0 and both <= rebuilt,
                   f"第二趟的编译目录不是新建的：照用了上一趟编的产物（第一趟退 {first.returncode}、编过 {sorted(built_before)}；第二趟退 {second.returncode}、编过 {sorted(rebuilt)}）："
                   f"{second.stdout[-300:]}{second.stderr[-400:]}",
                   "看 Library.begin_run：每一趟在 runs/ 下新建自己的运行目录，CARGO_TARGET_DIR 是它里面的 target（弄坏开关 build-directory-in-home 下照改前放在库目录里、跨趟留着）")
    selftest.check(first.returncode == 0 and second.returncode == 0 and not left_after_first and not left_after_second,
                   f"跑完运行目录里不剩它建的编译目录与树副本（正常结束）：第一趟之后还剩 {left_after_first}，第二趟之后还剩 {left_after_second}",
                   "看 main 的 finally 里的 Library.end_run：正常结束也删这一趟的运行目录（删之前核没有进程开着里面的文件）")


def selftest_run_directory_cells(selftest):
    """这一趟的运行目录跑完删（文件头「库目录」runs/）：出错退出（基线红，退 6）那一种结局也删；这一趟漏下一个进程开着运行目录里的文件时不删、报出那个进程、退 9。
    正常结束那一种在 selftest_fresh_build_cells，被 TERM 停那一种在 selftest_stop_cells，上一趟被 KILL 留下的在 selftest_basic_cells。"""
    work = selftest.work
    selftest.write_plan([("0", "one-test", "exit:100")])
    failed = selftest.run("own-error", "--item", "1", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    left = selftest.build_leftovers("own-error")
    selftest.check(failed.returncode == EXIT_BASELINE_RED and not left,
                   f"跑完运行目录里不剩它建的编译目录与树副本（出错退出）：基线红那一趟退 {failed.returncode}（应当退 6），库目录里还剩 {left}：{failed.stderr[-400:]}",
                   "看 main 的 finally 里的 Library.end_run：拒绝（Refusal）退出也删这一趟的运行目录")
    holder_file = os.path.join(work, "holder.pid")
    if os.path.exists(holder_file):
        os.remove(holder_file)
    selftest.write_plan([("1", "one-test", "leave-holder")])
    held = selftest.run("own-holder", "--item", "1", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.write_plan(SELFTEST_PLAN)
    holder = int(open(holder_file).read().split()[0]) if os.path.exists(holder_file) else None
    kept = selftest.leftover_run_directories("own-holder")
    reported = holder is not None and f"进程 {holder}（" in held.stderr
    if holder is not None and process_is_running(holder):
        os.kill(holder, signal.SIGTERM)   # 只点假的一步记下的那一个进程号
        wait_until(lambda: not process_is_running(holder), 10)
    for directory in kept:
        shutil.rmtree(directory, ignore_errors=True)
    selftest.check(held.returncode == EXIT_RUN_DIRECTORY_KEPT and len(kept) == 1 and reported,
                   f"这一趟的运行目录里还有进程开着文件时应当不删、报出那个进程、退 {EXIT_RUN_DIRECTORY_KEPT}（退 {held.returncode}；留着的 {kept}；漏下的进程 {holder}"
                   f"{'报了' if reported else '没报'}）：{held.stderr[-600:]}",
                   "看 Library.end_run：删之前经 scripts/hosts/run_build_directory.py 核本机有没有进程开着里面的文件（弄坏开关 run-directory-open-files-unchecked 下不核）")


def selftest_shared_library_cells(selftest):
    """两台各点名一半的条（单号一台、双号一台）判的都写进服务里同一份判定库（文件头「判定库」）：两趟各起一趟写进同一份，之后汇总是合起来的六栏，
    不点名的剪枝跑只现判驱动错的那两条（3、6）。自证的库目录在临时目录底下、各占一份命名空间，两趟用同一个库目录当那同一份库。"""
    selftest.run("shared", "--item", "1", "--item", "3", "--item", "5", "--item", "7", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.run("shared", "--item", "2", "--item", "4", "--item", "6", "--item", "8", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    summary = selftest.run("shared", "--summary")
    selftest.clear_calls()
    nothing_left = selftest.run("shared", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.check(summary.returncode == EXIT_GATE_RED and "caught=1 missed=0 unviable=1 stage_one_only=4 driver_error=2 not_run=0" in summary.stdout
                   and nothing_left.returncode == 0 and selftest.judged_sequences() == {"3", "6"},
                   f"两台各点名一半的结果没合对（汇总退 {summary.returncode}：{summary.stdout.splitlines()[:1]}；之后剪枝跑退 {nothing_left.returncode}、"
                   f"现判的序号 {sorted(selftest.judged_sequences())}（只该是驱动错的 3、6））：{summary.stderr[-300:]}{nothing_left.stderr[-300:]}",
                   "看 Library.append_rows 与 read_verdicts：两趟判的行都写进同一份判定库，状态照全部的行算")


REGISTERED_CHILD = r'''
import importlib.util, json, sys, types
specification = importlib.util.spec_from_file_location("broad_mutation_registered_child", sys.argv[1])
driver = importlib.util.module_from_spec(specification)
specification.loader.exec_module(driver)
driver.TEST_MODE = True     # 弄坏开关在这个进程里认（探针把驱动当模块载入）
rules = driver.prune_rules()
tree = rules.UnitTree(sys.argv[2])
quick = [(entry["binary"], entry["test"]) for entry in rules.test_entries(tree, (driver.VERIFICATION_PACKAGE,)) if entry["test"].endswith("kept_in_a_test_target")]
exclusions = driver.registered_exclusions(rules, tree, {("quick", "kept_in_a_test_target")})
nothing = driver.registered_exclusions(rules, tree, {("quick", "no_such_case")})
stub = types.SimpleNamespace(rules=rules, tree=tree, exclusions=[])
runnable = sorted(entry["test"] for entry in driver.Judgement.runnable(stub, ("singlefs-format",)))
runner = types.SimpleNamespace(skip_names=["alpha_case", "tool_case"])
base = ["cargo", "test", "--locked", "--release", "-p", driver.VERIFICATION_PACKAGE, "--lib", "--tests", "--no-fail-fast"]
commands = {"plain": driver.Runner.command_with_skips(runner, "two-test-checker-tier", base),
            "exact": driver.Runner.command_with_skips(runner, "two-test-checker-tier", base + ["--", "--exact", "a_case"]),
            "other": driver.Runner.command_with_skips(runner, "one-test", ["cargo", "nextest"]),
            "gpu": driver.Runner.command_with_skips(runner, driver.GPU_TEST_STEP, driver.gpu_pass_command("test", ["gpu"], ["lib"])),
            "none": driver.Runner.command_with_skips(types.SimpleNamespace(skip_names=[]), "two-test-checker-tier", base)}
print(json.dumps({"quick": quick, "exclusions": exclusions, "nothing": nothing, "runnable": runnable, "commands": commands, "base": base}))
'''


def selftest_registered_cells(selftest):
    """登记的崩溃枚举用例与放量分项（文件头「两段式」）和标了 #[ignore] 的用例，由自证起的子进程载入驱动直接问（弄坏开关在那里认）：
    ① 第二段 验证包与 GPU 那一趟的测试步命令接 --skip（命令里已有 `--` 的接在它后面，没有的补一个；别的步原样）；
    ② 这棵树 验证包里对上的静态测试进排除单，没对上的不进；
    ③ 标了 #[ignore] 的用例不算驱动真跑得到的测试。"""
    tree = selftest_source_copy(selftest, "registered-source", SELFTEST_ENTRY_BASELINE_SOURCES)
    done = subprocess.run([sys.executable, "-c", REGISTERED_CHILD, SELF_PATH, tree], capture_output=True, text=True, stdin=subprocess.DEVNULL, env=selftest.environment)
    if done.returncode != 0:
        for title in ("登记的崩溃枚举用例与放量分项没经 --skip 排掉：第二段 验证包与 GPU 那一趟的测试步命令不对", "登记的崩溃枚举用例没进排除单：验证包里对上的静态测试没被排掉",
                      "标了 #[ignore] 的用例进了驱动真跑得到的测试"):
            selftest.check(False, f"{title}（子进程退 {done.returncode}）：{done.stderr[-300:]}", "看 REGISTERED_CHILD 载入驱动后问的那几个函数")
        return
    found = json.loads(done.stdout)
    skips = ["--skip", "alpha_case", "--skip", "tool_case"]
    commands, base = found["commands"], found["base"]
    selftest.check(commands["plain"] == base + ["--"] + skips and commands["exact"] == base + ["--", "--exact", "a_case"] + skips and commands["other"] == ["cargo", "nextest"]
                   and commands["gpu"][-len(skips):] == skips and "--nocapture" in commands["gpu"] and commands["none"] == base,
                   f"登记的崩溃枚举用例与放量分项没经 --skip 排掉：第二段 验证包与 GPU 那一趟的测试步命令不对（{commands}）",
                   "看 Runner.command_with_skips：两个测试步（two-test-checker-tier、GPU 的测试步）接 --skip，命令里已有 `--` 的接在后面，没有的补一个，别的步和没有登记时原样")
    expected = [[binary, test, driver_reason] for (binary, test), driver_reason in ((pair, REGISTERED_REASON) for pair in found["quick"])]
    selftest.check(found["quick"] and found["exclusions"] == expected and found["nothing"] == [],
                   f"登记的崩溃枚举用例没进排除单：验证包里对上的静态测试没被排掉（对上的 {found['quick']}；排除单 {found['exclusions']}；没对上的 {found['nothing']}）",
                   "看 registered_exclusions：（测试目标, 用例函数）与 验证包里静态列出的测试逐个比，对上的进排除单")
    selftest.check(found["runnable"] == ["catching_one", "in_lib_case", "tests::in_lib_case"][1:2] + ["tests::in_lib_case"][:0] or found["runnable"] == sorted(["in_lib_case", "catching_one"]) or
                   (found["runnable"] and not any("on_the_card" in name for name in found["runnable"]) and any("in_lib_case" in name for name in found["runnable"])
                    and any("catching_one" in name for name in found["runnable"])),
                   f"标了 #[ignore] 的用例进了驱动真跑得到的测试（{found['runnable']}）",
                   "看 Judgement.runnable：标了 #[ignore] 的 cargo 不跑，不算驱动真跑得到的测试")


def selftest_test_variable_cells(selftest):
    """只供自证的变量：库目录在自证工作目录之外拒；隔了一层别的进程（父进程不是带 --selftest 的这份脚本）拒。"""
    outside = tempfile.mkdtemp(prefix="broad-mutation-selftest-outside.")
    try:
        elsewhere = subprocess.run([sys.executable, SELF_PATH, "--home", os.path.join(outside, "home"), "--source", selftest.source, "--refresh-list"],
                                   capture_output=True, text=True, stdin=subprocess.DEVNULL, env=selftest.environment)
        created = os.path.exists(os.path.join(outside, "home"))
    finally:
        shutil.rmtree(outside, ignore_errors=True)
    selftest.check(elsewhere.returncode == EXIT_USAGE and not created and "自证的工作目录" in elsewhere.stderr,
                   f"只供自证的变量指着自证工作目录之外的库时没拒（退 {elsewhere.returncode}，库目录{'建了' if created else '没建'}）：{elsewhere.stderr}",
                   "看 check_test_only_variables：设了只供自证的变量，库目录就得在 BROAD_MUTATION_SELFTEST_WORK 底下")
    selftest.clear_calls()
    relay = "import subprocess, sys; sys.exit(subprocess.run(sys.argv[1:]).returncode)"
    relayed = subprocess.run([sys.executable, "-c", relay, *selftest.command("first", "--item", "2")], capture_output=True, text=True,
                             stdin=subprocess.DEVNULL, env=selftest.environment)
    selftest.check(relayed.returncode == EXIT_USAGE and not selftest.calls() and "不是 --selftest 自己起的子进程" in relayed.stderr
                   and selftest.states("first").get("2") == NOT_RUN,
                   f"父进程不是 --selftest 的进程带着假一步的变量时没拒（退 {relayed.returncode}，调了 {len(selftest.calls())} 步，第 2 条状态 {selftest.states('first').get('2')}）：{relayed.stderr}",
                   "看 test_mode 与 resolve_home：只供自证的变量与弄坏开关只在父进程是带 --selftest 的这份脚本时认，这一判要排在「库目录不在临时目录」之前")
    selftest.clear_calls()
    rows_before = len(selftest.rows("first"))
    break_only = {name: value for name, value in selftest.environment.items() if name not in TEST_ONLY_VARIABLES}
    break_only["BROAD_MUTATION_BREAK"] = ",".join(part for part in (os.environ.get("BROAD_MUTATION_BREAK", ""), "rerun-done") if part)
    relayed_break = subprocess.run([sys.executable, "-c", relay, *selftest.command("first", "--item", "2")], capture_output=True, text=True,
                                   stdin=subprocess.DEVNULL, env=break_only)
    selftest.check(relayed_break.returncode == EXIT_USAGE and not selftest.calls() and "不是 --selftest 自己起的子进程" in relayed_break.stderr
                   and len(selftest.rows("first")) == rows_before,
                   f"父进程不是 --selftest 的进程只带着弄坏开关时没拒（退 {relayed_break.returncode}，调了 {len(selftest.calls())} 步，判定行 {rows_before} → {len(selftest.rows('first'))}）：{relayed_break.stderr}",
                   "看 check_test_only_variables：TEST_ONLY_VARIABLES 里要有 BROAD_MUTATION_BREAK，非自证的进程带着它就拒")
    # 平常跑设了弄坏开关、再带一个认不得的选项（门禁 code-tooling 的 break-switch-outside-selftest 格就这样探）：要在参数解析之前拒并点名变量，不是被 argparse 当用法错拒
    probed = subprocess.run([sys.executable, "-c", relay, sys.executable, SELF_PATH, "--force", "--break-guard-probe-unknown-option"], capture_output=True, text=True,
                            stdin=subprocess.DEVNULL, env=break_only)
    selftest.check(probed.returncode == EXIT_USAGE and "BROAD_MUTATION_BREAK" in probed.stderr and "不是 --selftest 自己起的子进程" in probed.stderr,
                   f"平常跑设了弄坏开关、带认不得的选项时没在参数解析之前拒并点名（退 {probed.returncode}）：{probed.stderr}",
                   "看 main 开头的 check_test_only_variables_before_parsing：它要排在 parse_arguments 之前")


def selftest_progress_cells(selftest):
    """一步跑过进度间隔要往 stderr 报进度：间隔换成 0.3 秒、第 1 条的测试那一步跑 1.5 秒，要有点名这一条这一步的行，其中有一行数到了日志里已过的那一条用例。"""
    selftest.write_plan([("1", "one-test", "sleep:1.5")])
    ran = selftest.run("progress", "--item", "1", BROAD_MUTATION_PROGRESS_INTERVAL="0.3", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.write_plan(SELFTEST_PLAN)
    lines = [line for line in ran.stderr.splitlines() if line.startswith("BROAD_MUTATION_PROGRESS sequence=1 step=one-test ")]
    selftest.check(ran.returncode == 0 and lines and any(" tests_passed=1 tests_failed=0" in line for line in lines),
                   f"一步跑过进度间隔没有报进度（退 {ran.returncode}；点名第 1 条 one-test 的进度行 {lines[:3]}）：{ran.stderr[-600:]}",
                   "看 Runner.wait_reporting_progress 与 progress_line：一步跑着时每过 PROGRESS_INTERVAL_SECONDS 秒往 stderr 打一行，数日志里已过的用例")


def selftest_stop_cells(selftest):
    """被 TERM 停：在途那一步连同它的进程组一起停（文件头「中断续接」）。假的一步挂住时自己再分出一个子进程一起挂着（真链上一步底下还有 cargo 与测试进程），
    驱动被 TERM 之后两个都要停；只给驱动起的那一个进程发 TERM 的话（弄坏开关 term-leaves-step-running）分出来的那一个照挂着，这一格判错。"""
    work = selftest.work
    for name in ("release", "hanging", "hang.pids"):
        if os.path.exists(os.path.join(work, name)):
            os.remove(os.path.join(work, name))
    selftest.write_plan([("1", "one-test", "hang")])
    pid_file = os.path.join(work, "hang.pids")
    stage_one_only_environment = dict(selftest.environment, BROAD_MUTATION_STAGE_ONE_ONLY="1")
    driver = subprocess.Popen(selftest.command("stop", "--item", "1"), stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                              text=True, stdin=subprocess.DEVNULL, env=stage_one_only_environment)
    hanging = wait_until(lambda: os.path.exists(os.path.join(work, "hanging")) or driver.poll() is not None, 90)
    wait_until(lambda: os.path.exists(pid_file) and len(open(pid_file).read().split()) >= 2, 5)
    driver.terminate()   # 只点自己起的这一个进程
    try:
        output, errors = driver.communicate(timeout=60)
    except subprocess.TimeoutExpired:
        driver.kill()    # 只点自己起的这一个进程
        output, errors = driver.communicate()
    step_processes = [int(word) for word in open(pid_file).read().split()] if os.path.exists(pid_file) else []
    all_stopped = wait_until(lambda: not any(process_is_running(process_id) for process_id in step_processes), 10)
    left_after_term = selftest.build_leftovers("stop")
    selftest.check(driver.returncode == EXIT_STOPPED and not left_after_term,
                   f"跑完运行目录里不剩它建的编译目录与树副本（被 TERM 停）：驱动退 {driver.returncode}，库目录里还剩 {left_after_term}：{errors[-600:]}",
                   "看 main 的 finally：被 TERM / INT 停（Stopped）也走 Library.end_run，在途那一步连同它的进程组停下之后删这一趟的运行目录")
    selftest.write_plan(SELFTEST_PLAN)
    selftest.clear_calls()
    rerun = subprocess.run(selftest.command("stop", "--item", "1", "--lock-wait", "0"), capture_output=True, text=True,
                           stdin=subprocess.DEVNULL, env=stage_one_only_environment)
    open(os.path.join(work, "release"), "w").close()   # 停不下的那一步（弄坏开关下）见到它就退，不留到自证收尾
    wait_until(lambda: not any(process_is_running(process_id) for process_id in step_processes), 10)
    rows_of_one = [row["outcome"] for row in selftest.rows_of("stop", 1)]
    selftest.check(hanging and os.path.exists(os.path.join(work, "hanging")) and driver.returncode == EXIT_STOPPED and len(step_processes) >= 2 and all_stopped
                   and rerun.returncode == 0 and rows_of_one == ["caught"],
                   f"被 TERM 停时在途那一步没有连同它的进程组一起停（那一步{'挂住了' if hanging else '没起来'}；驱动退 {driver.returncode}；那一步的进程 {step_processes}、"
                   f"{'都停了' if all_stopped else '还有活着的'}；紧接着重跑退 {rerun.returncode}、第 1 条各行 {rows_of_one}）：{errors[-600:]}{rerun.stderr[-300:]}",
                   "看 Runner.run_step（start_new_session：一步起在自己新开的进程组里）与 Runner.stop_step（给那一组发 TERM，到 --stop-grace 秒还有活着的就 KILL）")


def gpu_item_lines(output, prefix="BROAD_MUTATION_GPU_ITEM"):
    """输出里每条一行的 GPU 趟结论：{序号: 那一行}。"""
    found = {}
    for line in output.splitlines():
        if line.startswith(prefix + " "):
            found[int(re.search(r"sequence=(\d+)", line).group(1))] = line
    return found


def gpu_lines_of(output, prefix):
    """输出里以 prefix 起头的行，每行拆成 {键: 值}（制表符分段的明细行拆不了，交原样）。"""
    return [dict(token.split("=", 1) for token in line.split()[1:] if "=" in token) for line in output.splitlines() if line.startswith(prefix + " ")]


def selftest_gpu_cells(selftest):
    """GPU 那一趟（C489）：只有 CPU 红、只有 GPU 红、两趟都没红、没卡，各一格；GPU 只给 CPU 没红又有带 gpu 特性的测试够得着的条跑；
    命令的写法（特性、目标、不另套清场、--test-threads 1、BINDGEN）；认不出的退出码 / 无效 / 一个测试都没跑到；中途没卡、基线红；复用与点名现判（自证变量）；
    第二段没跑（自证变量）时 GPU 趟不起；--list-items 的状态是两趟合起来的。假的一步当假 GPU（SELFTEST_STEP_DRIVER 的 gpu- 那几步）。"""
    source = selftest_source_copy(selftest, "gpu-source", SELFTEST_GPU_FILES)
    covered = {"1", "2", "4", "5"}
    two = ("--item", "1", "--item", "2")

    def gpu_run(home, plan, *arguments, **environment):
        selftest.write_plan(plan)
        selftest.clear_calls()
        return selftest.run(home, *arguments, source=source, **environment)

    def summary_of(home):
        return selftest.run(home, "--summary", source=source)

    def gate_fields(text):
        found = gpu_lines_of(text, "BROAD_MUTATION_GATE")
        return found[0] if found else {}

    try:
        # ① 只有 CPU 红：第 1 条 CPU 那一趟（第一段）抓到，GPU 那一趟不起；第 2 条 CPU 没红、GPU 也没红 ⇒ 没红
        ran = gpu_run("gpu-cpu-red", [("1", "one-test", "fail-nextest")], *two)
        cpu_rows = {row["sequence"]: (row["outcome"], row["stage"]) for row in selftest.rows("gpu-cpu-red")}
        gpu_rows = [(row["sequence"], row["outcome"]) for row in selftest.gpu_rows("gpu-cpu-red")]
        items = gpu_item_lines(ran.stdout)
        summary = summary_of("gpu-cpu-red")
        selftest.check(ran.returncode == 0 and cpu_rows == {"1": ("caught", "one"), "2": ("missed", "two")} and gpu_rows == [("2", "missed")]
                       and selftest.gpu_judged_sequences() == {"2"} and set(items) == {2} and " cpu=missed gpu=missed combined=missed " in items[2]
                       and " judged=1 red=1 reused=0 " in items[2] and "BROAD_MUTATION_GPU_PLAN needed=1 judge=1 carried=0 " in ran.stdout
                       and "GPU 趟（要 1 条）：现判 1、复用 0、本次未跑 0" in ran.stdout
                       and summary_field(summary.stdout, "caught") == 1 and summary_field(summary.stdout, "missed") == 1 and summary.returncode == EXIT_GATE_RED
                       and (gate_fields(summary.stdout).get("gpu_needed"), gate_fields(summary.stdout).get("gpu_reused")) == ("1", "1"),
                       f"只有 CPU 红：CPU 那一趟抓到的条也起了 GPU 那一趟、或两趟的结局没分开记（退 {ran.returncode}；CPU 行 {cpu_rows}；GPU 行 {gpu_rows}；GPU 趟现判过 "
                       f"{sorted(selftest.gpu_judged_sequences())}）：{ran.stdout[-500:]}{ran.stderr[-300:]}",
                       "看 run_gpu_phase：只给 CPU 那一趟落的结论是没红的条起 GPU 那一趟（一条两趟里任一趟红算抓到，CPU 红了就不用跑）；GPU 的结局落 gpu-verdicts/，"
                       "不进 verdicts/，BROAD_MUTATION_GPU_ITEM 一行分开写 cpu= 与 gpu=")
        selftest.check(bool(selftest.gpu_rows("gpu-cpu-red"))
                       and {(row["sequence"], row["outcome"]) for row in selftest.rows("gpu-cpu-red")} == {("1", "caught"), ("2", STAGE_ONE_ONLY), ("2", "missed")},
                       "GPU 那一趟的结局没有单独存进 GPU 判定表（判定表 verdicts/ 里只该有 CPU 那一趟的三行）", "看 record_gpu_verdict：GPU 趟的判定行落 gpu-verdicts/，CPU 那一趟的 verdicts/ 一行不改")

        # ② 只有 GPU 红：CPU 那一趟都没红；第 1 条 GPU 的结果行被 --nocapture 打散（认名字靠 failures: 名单），第 2 条是整齐的结果行 ⇒ 合起来都是抓到
        ran = gpu_run("gpu-gpu-red", [("1", "gpu-test-checker-tier", "gpu-fail-scrambled"), ("2", "gpu-test-checker-tier", "gpu-fail")], *two)
        gpu_rows = {row["sequence"]: row for row in selftest.gpu_rows("gpu-gpu-red")}
        items = gpu_item_lines(ran.stdout)
        summary = summary_of("gpu-gpu-red")
        listed = selftest.states("gpu-gpu-red", source)
        selftest.check(ran.returncode == 0 and set(gpu_rows) == {"1", "2"} and all(row["outcome"] == "caught" and row["caught_by"] == SELFTEST_GPU_CASE
                                                                                  and re.fullmatch(r"[0-9a-f]{64}", row["test_key"]) for row in gpu_rows.values())
                       and all(" cpu=missed gpu=caught combined=caught " in line and " red=0 " in line for line in items.values()) and len(items) == 2
                       and summary_field(summary.stdout, "caught") == 2 and summary_field(summary.stdout, "missed") == 0
                       and (listed.get("1"), listed.get("2")) == ("caught", "caught"),
                       f"只有 GPU 红：GPU 那一趟红了没算抓到、抓到它的测试没认出来、或 --list-items 的状态不是两趟合起来的（退 {ran.returncode}；GPU 行 "
                       f"{[(sequence, row['outcome'], row['caught_by']) for sequence, row in sorted(gpu_rows.items())]}；--list-items {listed}）：{ran.stdout[-400:]}{ran.stderr[-300:]}",
                       "看 gpu_combined_state（GPU 抓到就是抓到）、gpu_failing_test（--nocapture 下结果行被打散，认 failures: 名单）、GpuJudging.catching_key 与 list_items 的 combined_states")

        # ③ 两趟都没红：CPU 没红、GPU 也没红 ⇒ 没红（汇总的没红栏）；GPU 那一趟只给有带特性的测试够得着的 4 条里 CPU 没红的跑（全量跑：8 条 CPU 都没红）
        ran = gpu_run("gpu-neither", [])
        gpu_rows = {row["sequence"]: row for row in selftest.gpu_rows("gpu-neither")}
        summary = summary_of("gpu-neither")
        missed_lines = [line.split("\t")[1] for line in summary.stdout.splitlines() if line.startswith("MISSED\t")]
        environment = selftest.gpu_environment()
        selftest.check(ran.returncode == 0 and set(gpu_rows) == covered and all(row["outcome"] == "missed" and re.fullmatch(r"[0-9a-f]{64}", row["coverage_key"])
                                                                                  for row in gpu_rows.values())
                       and selftest.gpu_judged_sequences() == covered and summary_field(summary.stdout, "missed") == 8 and sorted(missed_lines) == sorted(str(n) for n in range(1, 9))
                       and (gate_fields(summary.stdout).get("gpu_needed"), gate_fields(summary.stdout).get("gpu_reused")) == ("4", "4") and summary.returncode == EXIT_GATE_RED
                       and gpu_lines_of(ran.stdout, "BROAD_MUTATION_GPU_TOTAL")[0].get("judged") == "4",
                       f"两趟都没红：没记成没红、或 GPU 那一趟跑了闭包够不着的条（GPU 行 {sorted(gpu_rows)}；GPU 趟现判过 {sorted(selftest.gpu_judged_sequences())}；"
                       f"汇总没红 {summary_field(summary.stdout, 'missed')}、MISSED {missed_lines}）：{ran.stdout[-400:]}{ran.stderr[-300:]}",
                       "看 GpuJudging.need：只给「有一条跑得到的测试，它带特性的闭包含容器单元、又真引到 gpu 原子」的条跑 GPU 那一趟；没红的 CPU + GPU 两趟都没红才记没红，汇总与门禁行照合起来的状态数")
        # 命令的写法：特性是 tests/ 里 cfg 点到的、目标是够得着的那几个、测试步直接起（不另套清场）并加 --nocapture --test-threads 1、BINDGEN_EXTRA_CLANG_ARGS 取崩溃放量那一份
        bindgen = gpu_bindgen_arguments()
        build_commands = [json.loads(entry[3]) for entry in environment if entry[1] == GPU_BUILD_STEP]
        test_commands = [json.loads(entry[3]) for entry in environment if entry[1] == GPU_TEST_STEP]
        right_test = ["cargo", "test", "--locked", "--release", "-p", VERIFICATION_PACKAGE, "--features", "gpu", "--test", "gpu_pair",
                      "--", "--nocapture", "--test-threads", "1"]
        right_build = ["cargo", "test", "--locked", "--release", "-p", VERIFICATION_PACKAGE, "--features", "gpu", "--test", "gpu_pair", "--no-run"]
        selftest.check(bindgen and len(environment) == 2 * (1 + len(covered)) and build_commands == [right_build] * (1 + len(covered)) and test_commands == [right_test] * (1 + len(covered))
                       and all(entry[2] == bindgen for entry in environment),
                       f"GPU 那一趟的命令不对（BINDGEN {bindgen[:40]!r}；收到 {len(environment)} 步；编译步 {build_commands[:1]}；测试步 {test_commands[:1]}）",
                       "看 gpu_pass_command：编译步 cargo test --locked --release -p singlefs-verification --features <特性> <目标> --no-run；测试步同一条直接起（不套清场包装），"
                       "后面加 -- --nocapture --test-threads 1；BINDGEN_EXTRA_CLANG_ARGS 取 gpu_bindgen_arguments（crash-amplification-lib.sh 的那一份）")

        # ④ 没卡：基线那一趟日志里有 NOT_RUN_NO_GPU_ADAPTER ⇒ 两条逐条报本次未跑、不落 GPU 行、不记没红；之后有卡了重跑同一条命令补上
        ran = gpu_run("gpu-no-card", [("gpu-baseline", GPU_TEST_STEP, "gpu-no-adapter")], *two)
        summary = summary_of("gpu-no-card")
        not_run = gpu_lines_of(ran.stdout, "BROAD_MUTATION_GPU_NOT_RUN")
        no_conclusion = [line.split("\t") for line in summary.stdout.splitlines() if line.startswith("NO_CONCLUSION\t")]
        gate = gate_fields(summary.stdout)
        selftest.check(ran.returncode == 0 and not selftest.gpu_rows("gpu-no-card") and sorted(entry["sequence"] for entry in not_run) == ["1", "2"]
                       and all(entry["reason"] == "no-adapter" for entry in not_run) and selftest.gpu_judged_sequences() == set()
                       and "GPU 趟（要 2 条）：现判 0、复用 0、本次未跑 2" in ran.stdout and summary_field(summary.stdout, "missed") == 0
                       and (gate.get("gpu_needed"), gate.get("gpu_reused"), gate.get("gpu_not_run")) == ("2", "0", "2")
                       and {entry[1] for entry in no_conclusion if entry[-1].startswith("gpu-not-run")} == {"1", "2"} and summary.returncode == EXIT_GATE_RED,
                       f"没卡：没报本次未跑、记成了没红、或落了 GPU 行（退 {ran.returncode}；本次未跑 {not_run}；GPU 行 {selftest.gpu_rows('gpu-no-card')}；汇总没红 {summary_field(summary.stdout, 'missed')}）："
                       f"{ran.stdout[-500:]}{ran.stderr[-300:]}",
                       "看 gpu_classify_test（日志里有 NOT_RUN_NO_GPU_ADAPTER 是 not-run，不是没红）与 GpuPass.ensure_baseline、run_gpu_phase 的 BROAD_MUTATION_GPU_NOT_RUN：本次未跑的不落行、不进没红栏，"
                       "汇总里它们是 none（理由 gpu-not-run），门禁行的 gpu_not_run= 数得出")
        # 有卡了：同一条命令 CPU 那一趟沿用跳过，GPU 那一趟补上
        selftest.write_plan([])
        selftest.clear_calls()
        again = selftest.run("gpu-no-card", *two, source=source)
        selftest.check(again.returncode == 0 and {row["sequence"]: row["outcome"] for row in selftest.gpu_rows("gpu-no-card")} == {"1": "missed", "2": "missed"}
                       and "GPU 趟（要 2 条）：现判 2、复用 0、本次未跑 0" in again.stdout,
                       f"有卡之后重跑同一条命令没把 GPU 那一趟补上（退 {again.returncode}）：{again.stdout[-400:]}{again.stderr[-300:]}",
                       "看 run_items 里「选中 0 条要跑」那一支：CPU 那一趟没有要现判的，也要接着跑 run_gpu_phase")

        # 中途没卡（基线有、第 1 条测试步打了 NOT_RUN_NO_GPU_ADAPTER）：这一条和后面的都本次未跑，已落的行不动
        ran = gpu_run("gpu-card-gone", [("1", GPU_TEST_STEP, "gpu-no-adapter")], *two)
        not_run = gpu_lines_of(ran.stdout, "BROAD_MUTATION_GPU_NOT_RUN")
        selftest.check(ran.returncode == 0 and not selftest.gpu_rows("gpu-card-gone") and sorted(entry["sequence"] for entry in not_run) == ["1", "2"]
                       and selftest.gpu_judged_sequences() == {"1"},
                       f"中途没卡：这一条之后的没停下、或落了行（GPU 行 {selftest.gpu_rows('gpu-card-gone')}；本次未跑 {not_run}；GPU 趟现判过 {sorted(selftest.gpu_judged_sequences())}）",
                       "看 run_gpu_phase 的 stopped_for：测试步见到 NOT_RUN_NO_GPU_ADAPTER，这一条不落行、后面的条不再起，逐条报本次未跑；树里的原件照拷回")

        # 基线红：没改过的树上 GPU 那一趟就不绿 ⇒ 拒（退 6）、一条 GPU 行都不落、树回到原样；CPU 那一趟已落的行在
        ran = gpu_run("gpu-baseline-red", [("gpu-baseline", GPU_TEST_STEP, "gpu-fail")], *two)
        selftest.check(ran.returncode == EXIT_BASELINE_RED and "GPU 趟基线" in ran.stderr and not selftest.gpu_rows("gpu-baseline-red") and {(row["sequence"], row["outcome"]) for row in selftest.rows("gpu-baseline-red")} == {("1", STAGE_ONE_ONLY), ("1", "missed"), ("2", STAGE_ONE_ONLY), ("2", "missed")}
                       and selftest.gpu_judged_sequences() == set(),
                       f"GPU 趟基线红没拒成「基线不绿」（退 {ran.returncode}）：{ran.stderr[-400:]}", "看 GpuPass.ensure_baseline：没改过的树上编译红、测试红、一个测试都没跑到，退 6 并写出路")

        # 认不出的退出码、一个测试都没跑到、编不过：没结论的（驱动错）不当定论，编不过是无效
        ran = gpu_run("gpu-without-conclusion", [("1", GPU_TEST_STEP, "exit:104"), ("2", GPU_TEST_STEP, "gpu-zero")], *two)
        gpu_rows = {row["sequence"]: row["outcome"] for row in selftest.gpu_rows("gpu-without-conclusion")}
        summary = summary_of("gpu-without-conclusion")
        reasons = {line.split("\t")[1]: line.split("\t")[-1] for line in summary.stdout.splitlines() if line.startswith("NO_CONCLUSION\t")}
        selftest.check(ran.returncode == 0 and gpu_rows == {"1": DRIVER_ERROR, "2": DRIVER_ERROR} and reasons.get("1") == f"gpu-{DRIVER_ERROR}" and reasons.get("2") == f"gpu-{DRIVER_ERROR}"
                       and gate_fields(summary.stdout).get("gpu_without_conclusion") == "2" and summary_field(summary.stdout, "missed") == 0,
                       f"GPU 趟认不出的退出码 / 一个测试都没跑到不是没结论（GPU 行 {gpu_rows}；理由 {reasons}）：{ran.stdout[-300:]}{ran.stderr[-300:]}",
                       "看 gpu_classify_test：认不出的退出码、退 0 而一个测试都没跑到都是驱动错，不是没红；汇总里它们是没结论（gpu_without_conclusion=）")
        ran = gpu_run("gpu-unviable", [("1", GPU_BUILD_STEP, "fail-compile")], *two)
        gpu_rows = {row["sequence"]: row["outcome"] for row in selftest.gpu_rows("gpu-unviable")}
        selftest.check(ran.returncode == 0 and gpu_rows == {"1": "unviable", "2": "missed"} and selftest.states("gpu-unviable", source).get("1") == "unviable",
                       f"GPU 趟编不过没记成无效（GPU 行 {gpu_rows}）：{ran.stdout[-300:]}{ran.stderr[-300:]}", "看 gpu_classify_build 与 judge_gpu_item：编译步退 101 又有带源码位置的编译错误才是无效")

        # 记红、撤绿、下趟现判（GPU 一趟）：gpu-neither 里 CPU 一趟、GPU 一趟都没红，同一条命令再跑，没红的每趟现判（CPU 八条、GPU 趟四条都重判，一条都不沿用）；
        # 弄坏开关 missed-carried（没红照旧沿用）下一条都不现判
        selftest.write_plan([])
        selftest.clear_calls()
        reused = selftest.run("gpu-neither", source=source)
        selftest.check(reused.returncode == 0 and selftest.gpu_judged_sequences() == covered and selftest.judged_sequences() == {str(number) for number in range(1, 9)}
                       and "BROAD_MUTATION_GPU_PLAN needed=4 judge=4 carried=0 " in reused.stdout
                       and "GPU 趟（要 4 条）：现判 4、复用 0、本次未跑 0" in reused.stdout and len(selftest.gpu_rows("gpu-neither")) == 8,
                       f"GPU 趟没红的条下一趟没现判（CPU 一趟现判的 {sorted(selftest.judged_sequences())}、GPU 趟现判的 {sorted(selftest.gpu_judged_sequences())}，应是 CPU 八条、GPU 趟 {sorted(covered)}）："
                       f"{reused.stdout[-400:]}{reused.stderr[-300:]}",
                       "看 GpuJudging.state：真跑时这一趟自己的键下同一把键的定论里最后一个是没红的，状态换成沿用不了（理由 missed-rejudged）；Judgement.state 对 CPU 一趟同样（GPU 趟也没红的条 CPU 一趟的没红行不沿用）")
        # 抓到照沿用、点名现判（自证变量）：gpu-gpu-red 里 CPU 一趟没红、GPU 一趟抓到（合起来是抓到）：同一条命令再跑，CPU 一趟的没红行与 GPU 趟的抓到都沿用，一步都不起；
        # 点名现判第 2 条（BROAD_MUTATION_REJUDGE_NAMED）：GPU 趟追加一行（这一回没红）
        selftest.write_plan([])
        selftest.clear_calls()
        carried_green = selftest.run("gpu-gpu-red", *two, source=source)
        selftest.check(carried_green.returncode == 0 and not selftest.gpu_calls() and not selftest.judged_sequences()
                       and "BROAD_MUTATION_GPU_PLAN needed=2 judge=0 carried=2 " in carried_green.stdout
                       and "GPU 趟（要 2 条）：现判 0、复用 2、本次未跑 0" in carried_green.stdout and len(selftest.gpu_rows("gpu-gpu-red")) == 2,
                       f"GPU 趟抓到的条（CPU 一趟没红）下一趟没沿用（CPU 一趟现判的 {sorted(selftest.judged_sequences())}、GPU 步 {selftest.gpu_calls()[:3]}）：{carried_green.stdout[-400:]}{carried_green.stderr[-300:]}",
                       "看 GpuJudging.state 与 row_holds：键相同、gpu_key 相同、抓到的测试键相同才沿用；Judgement.state：要 GPU 趟、GPU 趟的真实状态不是没红的条（gpu_decided_keys），CPU 一趟的没红行照沿用；run_gpu_phase 里沿用得了的不选")
        rerun = selftest.run("gpu-gpu-red", "--item", "2", source=source, BROAD_MUTATION_REJUDGE_NAMED="1")
        selftest.check(rerun.returncode == 0 and selftest.gpu_judged_sequences() == {"2"} and len([row for row in selftest.gpu_rows("gpu-gpu-red") if row["sequence"] == "2"]) == 2,
                       f"点名现判（自证变量）没有重跑 GPU 那一趟并追加新行（GPU 步 {selftest.gpu_calls()}）：{rerun.stdout[-300:]}{rerun.stderr[-300:]}",
                       "看 run_gpu_phase：设了 BROAD_MUTATION_REJUDGE_NAMED 时沿用得了的也现判，GPU 判定库只追加")
        # 先抓到、同一把键又判没红（GPU 一趟）：第 2 条的 GPU 行是抓到、没红两行，最后一个定论是没红 ⇒ 红照键记着、前面的抓到撤掉：下一趟 CPU 一趟、GPU 一趟都现判
        selftest.clear_calls()
        flipped_gpu = selftest.run("gpu-gpu-red", "--item", "2", source=source)
        selftest.check(flipped_gpu.returncode == 0 and selftest.gpu_judged_sequences() == {"2"} and selftest.judged_sequences() == {"2"}
                       and "BROAD_MUTATION_GPU_PLAN needed=1 judge=1 carried=0 " in flipped_gpu.stdout,
                       f"GPU 趟先抓到、同一把键又判没红的条下一趟没现判（CPU 一趟现判的 {sorted(selftest.judged_sequences())}、GPU 趟现判的 {sorted(selftest.gpu_judged_sequences())}，应都是 [2]）："
                       f"{flipped_gpu.stdout[-400:]}{flipped_gpu.stderr[-300:]}",
                       "看 GpuJudging.state：同一把键（这一趟自己的键下）全部定论里最后一个是没红的，不看更早的抓到还认不认；gpu_decided_keys：GPU 趟的真实状态是没红的，CPU 一趟的没红行也不沿用")
        # 第二段没跑（BROAD_MUTATION_STAGE_ONE_ONLY）：GPU 趟接在第二段后面，不起
        ran = gpu_run("gpu-stage-one", [], *two, BROAD_MUTATION_STAGE_ONE_ONLY="1")
        selftest.check(ran.returncode == 0 and not selftest.gpu_calls() and "第二段没跑（只供自证的 BROAD_MUTATION_STAGE_ONE_ONLY），不跑" in ran.stdout and not selftest.gpu_rows("gpu-stage-one"),
                       f"第二段没跑时还是起了 GPU 那一趟（{selftest.gpu_calls()[:2]}）", "看 run_gpu_phase 开头的 stage_two_skipped")
    finally:
        selftest.write_plan(SELFTEST_PLAN)
        selftest.clear_calls()


GPU_JUDGING_CHILD = r'''
import importlib.util, sys
specification = importlib.util.spec_from_file_location("broad_mutation_gpu_judging_child", sys.argv[1])
driver = importlib.util.module_from_spec(specification)
specification.loader.exec_module(driver)
print(driver.gpu_judging_key(sys.argv[2]))
if len(sys.argv) > 3:
    print(driver.judging_units_key(sys.argv[2]))
'''


def selftest_gpu_judging_cells(selftest):
    """GPU 那一趟的判法单元指纹（文件头「GPU 那一趟」）：在一份拷贝上算，只改注释与文档串不变；改了 GPU 趟判结局的函数（gpu_classify_test 的体）就变；
    改它不动 CPU 那一趟的判法单元指纹（库里 CPU 那一趟的判定一条都不作废）。由自证起的子进程算（弄坏开关在那里认）。"""
    copy = os.path.join(selftest.work, "gpu-judging-copy")
    for path in sorted({path for path, _name in (*JUDGING_ENTRIES, *GPU_JUDGING_ENTRIES)}) + ["scripts/gate/lib_libtest_result_lines.py", "scripts/gate/lib_function_units.py",
                                                                                              "scripts/gate/lib_reuse_checks.py", "scripts/hosts/heavy_threads.py",
                                                                                              ".claude/scripts/project_preflight.py"]:
        if os.path.isfile(os.path.join(REPOSITORY, path)):
            os.makedirs(os.path.dirname(os.path.join(copy, path)), exist_ok=True)
            shutil.copyfile(os.path.join(REPOSITORY, path), os.path.join(copy, path))
    driver_copy = os.path.join(copy, "scripts/mutation/broad-mutation.py")

    def keys_now(with_cpu):
        done = subprocess.run([sys.executable, "-c", GPU_JUDGING_CHILD, SELF_PATH, copy, *(["cpu"] if with_cpu else [])], capture_output=True, text=True, stdin=subprocess.DEVNULL,
                              env={name: value for name, value in selftest.environment.items() if name not in ("BROAD_MUTATION_JUDGING_KEY", "BROAD_MUTATION_GPU_JUDGING_KEY")})
        lines = done.stdout.split()
        return lines if done.returncode == 0 and len(lines) == (2 if with_cpu else 1) else [f"退 {done.returncode}：{done.stderr.strip()[-300:]}"] * (2 if with_cpu else 1)
    with open(driver_copy, encoding="utf-8") as handle:
        original = handle.read()
    marker = "    text = gpu_log_text(log_path)\n    passed = sum("
    changed_body = 'return GpuOutcome("caught", gpu_failing_test(log_path), ' + 'GPU_TEST_STEP)'
    if original.count(marker) != 1 or original.count(changed_body) != 1:
        selftest.check(False, "GPU 趟判法单元指纹那一格找不到要改的两处（gpu_classify_test 的开头与抓到那一行）", "看 selftest_gpu_judging_cells 里的 marker 与 changed_body：gpu_classify_test 改了写法就跟着改")
        return
    before_gpu, before_cpu = keys_now(True)
    with open(driver_copy, "w", encoding="utf-8") as handle:
        handle.write(original.replace(marker, "    # 自证：只加一行注释\n" + marker, 1).replace("\nimport argparse\n", "\n# 自证：文件里再加一行注释\nimport argparse\n", 1))
    commented_gpu = keys_now(False)[0]
    with open(driver_copy, "w", encoding="utf-8") as handle:
        handle.write(original.replace(changed_body, 'return GpuOutcome("caught", gpu_failing_test(log_path), ' + 'GPU_TEST_STEP + "（自证改了判结局的函数）")', 1))
    body_gpu, body_cpu = keys_now(True)
    # GPU 趟沿用不沿用没红的那一句（GpuJudging.state 里）：改它 GPU 趟的判法单元指纹变，CPU 那一趟的不变
    carry_rule = '            if decisive and decisive[-1] == "missed":\n                return ItemState(NOT_RUN, False), "missed-rejudged", None\n'
    if original.count(carry_rule) == 1:
        with open(driver_copy, "w", encoding="utf-8") as handle:
            handle.write(original.replace(carry_rule, carry_rule.replace('decisive[-1] == "missed"', 'decisive[-1] != "caught"'), 1))
        rule_gpu, rule_cpu = keys_now(True)
    else:
        rule_gpu = rule_cpu = "找不到要改的那一句（GpuJudging.state 里撤绿没红的那一句改了写法就跟着改这一格）"
    selftest.check(re.fullmatch(r"[0-9a-f]{64}", before_gpu) and commented_gpu == before_gpu,
                   f"只改驱动注释 GPU 趟的判法单元指纹变了（改前 {before_gpu[:16]}、只改注释之后 {commented_gpu[:16]}）",
                   "看 gpu_judging_key：取 GPU_JUDGING_ENTRIES 那几个函数的 python 单元闭包键（注释与文档串不进指纹），不取整份原文")
    selftest.check(re.fullmatch(r"[0-9a-f]{64}", body_gpu) and body_gpu != before_gpu,
                   f"改了 GPU 趟判结局的函数（gpu_classify_test 的体）GPU 趟的判法单元指纹没变（{before_gpu[:16]} → {body_gpu[:16]}）",
                   "看 GPU_JUDGING_ENTRIES：命令怎么拼、步结局怎么分、沿用怎么判、两趟怎么合那几个函数都要在里面")
    selftest.check(re.fullmatch(r"[0-9a-f]{64}", rule_gpu) and rule_gpu != before_gpu and rule_cpu == before_cpu,
                   f"改了 GPU 趟沿用不沿用没红的那一句（GpuJudging.state 里）GPU 趟的判法单元指纹没变、或 CPU 那一趟的跟着变了（GPU {before_gpu[:16]} → {rule_gpu[:16]}；CPU {before_cpu[:16]} → {rule_cpu[:16]}）",
                   "看 GPU_JUDGING_ENTRIES：GpuJudging.state 与 gpu_decided_keys 要在里面，撤绿没红的规矩写在它们里面；GpuJudging.state 不许被 JUDGING_ENTRIES 的闭包点到")
    selftest.check(re.fullmatch(r"[0-9a-f]{64}", before_cpu) and body_cpu == before_cpu,
                   f"改了 GPU 趟的函数，CPU 那一趟的判法单元指纹跟着变了（{before_cpu[:16]} → {body_cpu[:16]}）：库里 CPU 那一趟的判定会一条都不沿用",
                   "GPU 那一节的函数不许被 JUDGING_ENTRIES 的闭包点到（process_item 等不改、不调 GPU 那一节）；CPU 那一趟的判法单元指纹只管 CPU 那一趟")


SELFTEST_CELL_NAMES = (
    "selftest_fresh_build_cells",
    "selftest_run_directory_cells",
    "selftest_prune_cells",
    "selftest_reuse_cells",
    "selftest_global_cells",
    "selftest_judging_cells",
    "selftest_gpu_cells",
    "selftest_gpu_judging_cells",
    "selftest_refused_cells",
    "selftest_basic_cells",
    "selftest_state_cells",
    "selftest_table_cells",
    "selftest_environment_cells",
    "selftest_sync_cells",
    "selftest_baseline_threads_cells",
    "selftest_baseline_key_cells",
    "selftest_baseline_entry_cells",
    "selftest_service_cells",
    "selftest_shared_library_cells",
    "selftest_registered_cells",
    "selftest_test_variable_cells",
    "selftest_progress_cells",
    "selftest_stop_cells",
    "selftest_rocksdb_entry_cells",
    "selftest_multi_host_configuration_cells",
)


def run_selftest_cells(selftest, only=None):
    """自证的各组格，照 SELFTEST_CELL_NAMES 的次序；only 给了只跑点名的几组（环境变量 BROAD_MUTATION_SELFTEST_CELLS，开发时分段跑用：有的组接着前面组留下的库目录，单跑要连前面那一组一起点）。"""
    for name in SELFTEST_CELL_NAMES:
        if only is None or name in only:
            globals()[name](selftest)


def selftest_rocksdb_entry_cells(selftest):
    """真跑起 cargo 之前判 RocksDB 预编库的取值：被判的树里 验证包依赖 rocksdb、进程环境里四个取值是空的 ⇒ 拒（退 2）、假的一步一次都没起、报出 entry 的红句。
    别的格的树（SELFTEST_SOURCES）没有依赖 rocksdb 的清单，entry 报不判。"""
    tree = selftest_source_copy(selftest, "rocksdb-source", {"crates/singlefs-verification/Cargo.toml": selftest_package("singlefs-verification")
                                                            + '\n[dependencies]\nrocksdb = { version = "=0.25.0", optional = true }\n'})
    calls_before = len(selftest.calls())
    refused = selftest.run("rocksdb", source=tree, ROCKSDB_LIB_DIR="", ROCKSDB_STATIC="", SNAPPY_LIB_DIR="", SNAPPY_STATIC="")
    selftest.check(refused.returncode == EXIT_USAGE and len(selftest.calls()) == calls_before and "rocksdb-prebuilt entry：" in refused.stdout + refused.stderr,
                   f"被判的树里 验证包依赖 rocksdb 而进程环境里没有预编库的取值，没在起 cargo 之前拒（退 {refused.returncode}，"
                   f"假的一步起了 {len(selftest.calls()) - calls_before} 次）：{(refused.stdout + refused.stderr)[-600:]}",
                   "看 main 里 check_real_run_is_allowed 之后的 check_rocksdb_entry（scripts/hosts/rocksdb-prebuilt.py entry）")


def selftest_multi_host_configuration_cells(selftest):
    """多机配置交给测试（文件头同名一段）：前面几组起过的每一步里，种类是 test 的（名字里不带 build 的）都收到自证那一份多机配置的绝对路径、编译步没收到；
    多机配置不在 ⇒ 真跑起 cargo 之前拒（退 2）、说是多机配置、假的一步一次都没起。弄坏开关 test-steps-configuration-unpassed 下两格判错。"""
    path = os.path.join(selftest.work, "multi-host-configuration.log")
    records = []
    if os.path.exists(path):
        with open(path) as handle:
            records = [tuple(line.rstrip("\n").split("\t")) for line in handle if line.strip()]
    test_records = [record for record in records if len(record) == 3 and "build" not in record[1]]
    build_records = [record for record in records if len(record) == 3 and "build" in record[1]]
    wrong = sorted({f"{record[1]}={record[2]}" for record in test_records if record[2] != selftest.multi_host_configuration})
    selftest.check(test_records and not wrong and all(record[2] == "-" for record in build_records),
                   f"真跑测试的步没收到多机配置（测试步 {len(test_records)} 次，收到的不是 {selftest.multi_host_configuration} 的：{wrong[:5]}；"
                   f"编译步收到的：{sorted({record[2] for record in build_records})[:3]}）",
                   "看 Runner.run_step 里设 SINGLEFS_MULTI_HOST_CONFIG 那一段与 main 里的 check_multi_host_configuration")
    calls_before = len(selftest.calls())
    absent = os.path.join(selftest.work, "multi-host-absent.env")
    refused = selftest.run("multi-host-absent", SINGLEFS_MULTI_HOST_CONFIG=absent)
    selftest.check(refused.returncode == EXIT_USAGE and len(selftest.calls()) == calls_before and f"多机配置 {absent} 不在" in refused.stdout + refused.stderr,
                   f"多机配置不在时没在起 cargo 之前拒（退 {refused.returncode}，假的一步起了 {len(selftest.calls()) - calls_before} 次）：{(refused.stdout + refused.stderr)[-600:]}",
                   "看 main 里 check_rocksdb_entry 之后的 check_multi_host_configuration")


def run_selftest(only=None):
    unknown = [name for name in (only or []) if name not in SELFTEST_CELL_NAMES]
    if unknown:
        refuse(f"BROAD_MUTATION_SELFTEST_CELLS 点了不认得的组：{'、'.join(unknown)}", f"只认这几组：{'、'.join(SELFTEST_CELL_NAMES)}")
    service_problem = prune_server_client().selftest_service_problem()   # 判定库只在临时 prune-server 上写（客户端文件头「命令行」--with-temporary-service）
    if service_problem:
        refuse(f"--selftest 不在临时 prune-server 底下：{service_problem}", prune_server_client().SELFTEST_SERVICE_REMEDY)
    work = tempfile.mkdtemp(prefix="broad-mutation-selftest.")
    selftest = Selftest(work)
    try:
        run_selftest_cells(selftest, only)
    finally:
        open(os.path.join(work, "release"), "w").close()   # 挂住的那一步（有的话）见到它就退
        time.sleep(0.2)
        shutil.rmtree(work, ignore_errors=True)
    if selftest.failures:
        print(f"  ✗ --selftest：{selftest.cells} 格里 {selftest.failures} 格判错", file=sys.stderr)
        print(f"     → 怎么办：照上面每一格给的出路改，再跑 python3 {SELF_PATH} --selftest", file=sys.stderr)
        return 1
    say("  ✓ --selftest：临时库目录拒、生成清单与列项、项写错拒、点名与两段、杀掉再起只补没做完的、各种结局与汇总六栏、第二段只补第一段没跑到的、--from、"
        "上一趟被 KILL 留下的运行目录开跑先删、点名跳过沿用得了的条与点名现判（自证变量）、定论不被降级、认不出的选项退 2、清单按单元存的表改坏判红、认不出的退出码、"
        "重起验编译与连着无效或秒红、恢复原件半途被杀之后下一趟照常判、工具链进每一把键、"
        "不带参数就是剪枝（沿用数 > 0）、改了一个函数只现判它里面的条（同文件在某提交之后改过别处照样沿用）、点名现判（自证变量）判成没红、"
        "多登记一条 / 只改 Cargo.toml 注释照样沿用、改 RUSTFLAGS 全部现判、只改驱动注释判法单元指纹不变、拒算的条每趟现判、"
        "mutants.toml 写 exclude_re 之后清单少那一条且整版重记而别的条沿用、基线按线程数分开记、"
        "基线键按测试的单元闭包取（改不在测试闭包里的函数测试步基线照沿用、改测试就重跑、编译步拒算照跑）、"
        "基线键拒算的测试步按条记基线（收得全的测试各自沿用、只照跑拒算与改过的那几条，验证包拒算的那几条另起 --exact）、"
        "判定库只在 prune-server 里（按写入次序读回、库目录里不建判定库、连不上退 75）、每一趟自己的编译目录（不照用上一趟的产物）、跑完运行目录里不剩编译目录与树副本（正常、出错、被 TERM 停三种结局都删，有进程开着就不删、报出来、退 9）、两趟写进同一份判定库、"
        "登记的崩溃枚举用例与放量分项（测试步命令接 --skip、对上的静态测试进排除单、标了 #[ignore] 的不算真跑得到的测试）、只供自证的变量不出自证、一步跑过进度间隔要报进度、被 TERM 连进程组一起停、"
        "GPU 那一趟（只有 CPU 红 / 只有 GPU 红 / 两趟都没红 / 没卡 / 中途没卡 / 基线红 / 认不出的退出码与无效与一个测试都没跑到、命令的写法、复用与点名现判（自证变量）、"
        "第二段没跑时不起 GPU 趟、GPU 趟判法单元指纹与 CPU 的分开、复用三问与复用哨兵带上 GPU 趟），"
        f"{selftest.cells} 格都对")
    return 0


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
