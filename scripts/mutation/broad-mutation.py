#!/usr/bin/env python3
# admission: always 每次调都现算此刻源码上每条变异的单元键再查判定库；哪些条要跑由键与沿用判据定，上一次的结论不替这一次作保
# run-condition: command git python3
# broad-mutation-driver-signature: singlefs 广谱变异逐条驱动（重型测试闸 .claude/hooks/lib_heavy_tests.py 按这一行认改名或拷走的一份，别删）
# 长活声明（判法 .claude/hooks/lib_long_run.py：scripts/hosts/detached-run.sh start --wall-clock over-2h 与 long-run-check、scripts/hosts/host-dispatch.py 文件头 ⓪ 都读这几行）：
# long-run-shards: 按清单序号切：--shard <k>/<n> 只跑「序号 mod n == k」的条（n 由调用方给，两台各起一趟 --shard k/2），跑完在一台上 --import-results <另一台的库目录> 把判定并进本机那一份库
# long-run-resume: 重起同一条命令就是续跑：判定库里键相同、沿用得了的条跳过，超时、内存撞顶、驱动错与被打断的那一条照样再选中重跑（被打断的先照在途记录拷回原件）
# long-run-unshardable: 多机：--shard 的片数、在哪几台起、每台的 --threads 与 --memory-cap 都由调用方手给（--threads 默认 SINGLEFS_THREAD_CAP 或每台默认重活线程数（.claude/heavy-thread-default 的登记值），--memory-cap 默认 24G），驱动不读 crash-point-replay-shard-configuration-check.sh --emit-hosts 的清单、也不按 host-resources.py --json 现算
"""广谱变异的逐条驱动（.claude/kb/checks-owed.md 的 C489；用户 2026-09-30 定：一直跑、中断要续接不从头来、一条一条可点名；
2026-10-15 定：剪枝的单元是函数，广谱变异一条一条按内容算，不按 HEAD、不按提交算）。

  broad-mutation.py [选哪些条] [怎么跑]             不带选条参数就是剪枝：把 --source 给的工作树（默认本仓的工作区）换进库目录的专用树，由 cargo-mutants
                                                   生成此刻的清单，每条变异现算单元键过沿用判据（scripts/mutation/mutation-prune.py），沿用得了沿用、沿用不了现判；
                                                   成功那句报「现判 N、沿用 M、拒算 R」。重跑同一条命令就是续跑
  broad-mutation.py --full [怎么跑]                全量：不读判定库、选中的条全部现判（只写不读），现判的照键写回同一份库；沿用得了而现判翻了面的逐条打 FLIPPED。
                                                   SINGLEFS_GATE_FULL=1 同义
  broad-mutation.py --refresh-list                 只换树、生成清单、把清单按单元记进库（不编不跑），之后 --list-items / --summary 在同一份源码上列得出全部条
  broad-mutation.py --list-items                   逐行打「序号 <TAB> 包 <TAB> 变异名 <TAB> 状态 <TAB> stale=<同一处变异键对不上的判定行数> <TAB> key=<此刻的键>」，不编不跑、不起 cargo；
                                                   状态是两趟合起来的（CPU 那一趟没红又要 GPU 那一趟的条取合起来的状态，见下「GPU 那一趟」）
  broad-mutation.py --summary                      八栏条数与提交门禁那一行（见下「输出」，八栏按两趟合起来的状态数，另报 GPU 那一趟要几条、复用几条、本次未跑几条）；不编不跑、不起 cargo，只读库目录
  broad-mutation.py --check-ignored                只判 --source 那棵树里 harness 档的 #[ignore] 用例登记得对不对（见下「忽略用例登记」），打第二段慢档用例那一步的命令；不编不跑、不要前缀
  broad-mutation.py --unit-environment             只打一行：此刻环境里构建环境与工具链认的变量（拿掉它构建环境的生效值就变、或在树里起的 rustc -Vv 就变的那几个，
                                                   例 RUSTUP_TOOLCHAIN；scripts/hosts/detached-run.sh 本来就带进单元的 UNIT_CARRIED_VARIABLES 除外），
                                                   写成 `env 名=值 …`（按 shell 引好）；一个都没有打空行。不编（只起 rustc -Vv）、不跑、不改库目录。
                                                   经 detached-run.sh 起单元之前跑它，把这一行原样接在 `--` 后面（.claude/skills/crash-test/commit-stages.md 第 2b 步）
  broad-mutation.py --import-results <目录>        把另一台机器库目录里的判定行并进本机这一份库（两台各跑一片之后合）：逐行核段数、结局、键的写法，
                                                   键相同而两边定论不同的不并、逐条列出；每行的校验列先用另一台的密钥核（两台交换密钥，对端那一份的取法见下
                                                   「判定行的校验列」），一行核不过整表拒、退 1，对端密钥取不到拒导入、退 2；核过去掉、用本机的密钥重签。
                                                   同目录的 gpu-verdicts.tsv（GPU 那一趟的判定）也并进本机的 gpu-verdicts.tsv：逐行核键——行的键要是本机此刻源码（--source，
                                                   与两台跑的同一份）清单里某一条的单元键、行的 gpu_key 要等于本机此刻 GPU 趟自己的键——对不上的行不并、逐条列出（IMPORT_GPU_REJECTED），
                                                   之后本机补跑 GPU 趟时照现判；核键要本机已记清单（--refresh-list，两台各跑过一片的本机都记过了），清单不全拒
  broad-mutation.py --selftest                     自证（假清单、假测试命令、假 GPU，不起 cargo）；环境变量 BROAD_MUTATION_SELFTEST_CELLS=<组名,…> 只跑点名的几组格（开发时分段跑，
                                                   组名见 SELFTEST_CELL_NAMES，有的组接着前面组留下的库目录；不设就是全部，门禁与提交前跑的都不设）
已删的（传了退 2，出路写新跑法）：--sync-tree、--commit、--changed-since、--milestone-full（「一版」不再是容器，不按提交选条；里程碑末全量用 --full）。

选哪些条（不给 --item 就是此刻清单里沿用不了的全部条，按序号）：
  --item <序号或变异名>   可给多次，只跑点名的里面沿用不了的；沿用得了的跳过并逐条说一句。不与 --from / --package / --shard 同用
  --rerun                 只与 --item 同用：点名的条沿用得了也重跑，追加新行，旧行留着
  --from <序号>           只要序号不小于它的
  --package <包>          只要这个包的
  --shard <k>/<n>         只要「序号 mod n == k」的（k 取 0..n-1），两台机器各跑一半用（两台的 --source 要是同一份源码，序号才对得上）
  序号是此刻这份清单里的次序（先 core 后 checker，同包里按文件、起止、替换文排），只当索引，不进任何键。
怎么跑：
  --source <目录>         被判的工作树（主工作区或一份副本），默认本仓根；驱动只读它，变异改的是库目录里的专用树
  --skip-stage-two        只跑第一段；第一段没红的记「只过了第一段」（stage-one-only），不算「没红」定论
  --stage-two-only        只给过了第一段的条补第二段；点名的条没过过第一段（判定库里没有它键相同的 stage-one-only 行）拒，退 2
  --stage-two-eatmydata   第二段的文档测试与 checker 档快档也经 eatmydata 跑（默认不经；harness 慢档用例那一步与第一段同一种起法，一直经）
  --skip-gpu              不跑 GPU 那一趟（见下「GPU 那一趟」）：要这一趟的条逐条打 BROAD_MUTATION_GPU_NOT_RUN（本次未跑，不落行、不算没红、也不算通过）；机器上没卡时用
  --threads <N>           线程数，默认取 SINGLEFS_THREAD_CAP，没设取每台默认重活线程数（.claude/heavy-thread-default 的登记值）；编译与测试都照它（经 scripts/hosts/capped.sh）
  --memory-cap <上限>     每条命令的内存上限（scripts/hosts/run-with-memory-cap.sh 的写法），默认 BROAD_MUTATION_MEMORY_CAP，没设 24G
  --timeout-floor <秒>    单步超时的地板，默认 900；--timeout-multiplier <倍> 默认 5。单步超时 = max(地板, 倍数 × 基线里这一步的秒数)：
                          基线是开跑时在这台机器当时的负载下量的（.claude/kb/tooling.md「超时地板按空闲机器标定，就会把「机器上还有别的活」记成「变异抓不到」」）
  --suspect-streak <K>    连续 K 条「无效」、或连续 K 条「测试那一步不到 --suspect-test-seconds 秒就红的抓到」，就在没改过的树上把那一步重跑一遍：
                          红了退 6 报「疑似环境坏了」并列出这 K 条，绿了接着跑。默认 8；--suspect-test-seconds 默认 0.2
  --stop-grace <秒>       被 TERM / INT 停时等在途那一步退出多久，默认 10，过了还在就 KILL
  --lock-wait <秒>        库目录的锁被上一趟留下的进程占着时等多久，默认 120
  --home <库目录>         默认 ${SINGLEFS_BROAD_MUTATION_HOME:-~/.local/share/singlefs/broad-mutation-units}；要写库的用法落在开机会清空的目录里拒（只读的几种不写库、不拒）；
                          有 tree.source 或 versions/ 的是「一版一个目录」那时的旧库，整个弃用：拒，另起一个新的（不迁移、不补签）
  只读的几种（--list-items、--summary）另认：--result-key <文件>（校验列的密钥换成这一份）、--toolchain-from <文件>（rustc -Vv 的输出换成这份文件）、
  --tools-from <文件>（工具版本换成这份文件，一行一个）：提交门禁的样本用，真跑与写行的用法不认。

库目录（一份，固定一处，不随趟、不随判法换目录、没有版本目录）：
  runs/run-<进程号>-<随机>/   这一趟的运行目录（用户 2026-10-23 定案「自己用完自己删」：编译生成物每趟新建、跑完删，判定库照留、照旧按单元键复用）。
                   拿到库目录的锁之后建（Library.begin_run），正常结束、拒绝退出、被 TERM / INT 停都在 main 的收尾删掉（Library.end_run；
                   删之前经 scripts/hosts/run_build_directory.py 核本机没有进程开着里面的文件，有就不删、逐个报进程，这一趟别的都成了也退 9）。
                   被 KILL 的那一趟留下的，下一趟拿到锁之后先删（同样先核开着的文件；里面有在途记录的报一句被打断在哪一条）。里面四样：
    tree/          专用树：开跑把 --source 那棵树的导出范围逐文件拷进来（写的文件 mtime 取此刻），变异就地改这里的一个文件、判完拷回原件：
                   编译目录在这一趟里跨条复用，每条只付增量编译。导出范围取门禁 checker-tier-crates-mutation-replay 登记的
                   拷贝范围（scripts/mutation/crates-mutation-rows.py 的 copied_paths，去掉 research/ 下的）加 .cargo/
    target/、target-coverage/   CARGO_TARGET_DIR；scratch/ 是每一步的 TMPDIR（每一步开跑前清空），scratch-coverage/ 是覆盖那一步的产物
    inflight.json、inflight.original   在途记录：改文件之前先存原件、记下改的是哪一条，判完照它拷回（临时件 inflight.restore 在运行目录里，不在树里）
  tree.synced      上一次换树拷了几个文件、什么时候（只当记录，不进任何键）
  verdicts.tsv     判定库（只追加，每行带校验列）：一行一条判定，键是那一条变异的单元键（见下「键」），另记结局、哪一段判的、抓到它的测试与它的测试键、
                   覆盖者键、编译与测试秒数、判定日期、跑法、序号与变异名与位置（只当索引）、拒算形态、备注
  compositions/<键前两位>/<键>.txt   每把键（变异键、测试键）的组成清单：单元行（种类、限定名、文件:起止、指纹）、依赖行、工具链那几行；
                   作废时照它报出变了的是哪几个单元（--list-items、--summary 的 NO_CONCLUSION 那一行）
  lists.tsv        清单按单元存：一行「清单的单元键 <TAB> 这个单元里的变异（JSON）」（mutation-prune.py 的 list_unit_key、list_entry）；
                   被变异包里每一个单元都记（没有变异的记空表），--list-items 与 --summary 照它在此刻的源码上复原清单，不起 cargo
  baselines.tsv    基线（只追加）：一行七段：基线键、步名、结局（ok / exit-<码>）、秒数、时刻、线程数、这一步的内存峰值字节；
                   基线键按这一步真跑到的判定的单元闭包算（见下「基线」）；步名只当索引，不进键
  baselines/<基线键前 16 位>/   test-times.<线程数>.<步名>.tsv（runner 量的逐条 CPU 秒）、peaks.<线程数>.tsv（包装记的峰值）；
                   基线键拒算的步放 baselines/refused/（一份，每趟头一回用到时清空）
  baseline-entries.tsv   按条记基线（只追加，见下「基线」）：一行八段：记录键、步名、线程数、kind（test / rest / binary）、二进制（只当索引）、
                   runner 记的那几行（JSON）、量它那一趟的内存峰值字节、时刻
  coverage.tsv     覆盖记录（只追加）：一行「记录键 <TAB> 哪一段 <TAB> 二进制 <TAB> 用例 <TAB> 执行到的单元（种类:指纹，JSON）<TAB> profraw 份数 <TAB> 日期」；
                   记录键是那条测试的测试键（checker 档逐个二进制的取那个二进制里各条测试的测试键），测试的闭包不变就照用，变了那几条重跑插桩
  escalated-verdicts.tsv、escalation/   升档判的抓到的登记表与两趟日志（见下「升档」）
  imports.tsv      导入记录；logs/ 没过的那一步的输出（每条每步一份，后一趟同名的盖掉前一趟）
  gpu-verdicts.tsv GPU 那一趟的判定库（只追加，每行带同一把密钥算的校验列，十四段：键、序号、包、变异名、结局、抓到它的测试、测试键、覆盖者键、GPU 趟自己的键、测试秒、判定日期、跑法、备注、校验列）：
                   键是那条变异的单元键（与 verdicts.tsv 里 CPU 那一趟的同一把），GPU 那一趟的结局只记在这里，不进 verdicts.tsv；沿用判据同 CPU 那一趟（见下「GPU 那一趟」）

键（规则与弄坏开关在 scripts/mutation/mutation-prune.py 文件头；单元与闭包由 scripts/gate/lib_function_units.py 算）：
  变异键 = 容器单元（包住这条变异的最小单元）的单元闭包键（闭包里每个单元的种类与去注释指纹、数据单元摘要、入口目标的生效 cfg、
    闭包真引到的外部包照 Cargo.lock 解析出的版本行、全局那几行）+ 变异在容器里的相对位置（去注释的词法单元下标）+ 原文 + 替换文。
  全局那几行（每把键里都有）只留：rustc -vV、cargo nextest / cargo-mutants / llvm-profdata 的版本、构建环境的生效值（编译变量与它们指的程序、
    cargo 配置里 build / profile / target / env / unstable 解析后的值、rust-toolchain 解析后的值、工作区 Cargo.toml 解析后的 [profile]）、判法单元指纹。
    Cargo.toml、.cargo/ 的原文不进（只改注释照样沿用）；排除表不进（加一条排除只让被排除的那几条测试不再算数：抓到它的那条测试进了排除表的现判）；
    限时参数、--timeout-floor、--memory-cap、--threads 不进（加大了重跑照样沿用别的条）。
  判法单元指纹 = lib_function_units.py 在本驱动与 mutation-prune.py 上从 JUDGING_ENTRIES 那几个函数（选条、沿用、结论分类、升档）收的 python 单元闭包键：
    只改注释与文档串不变，改了判结局的函数（连同它们点到的常量，例如升档的四个倍数常量）就变，库里的判定一条都不沿用。没有手写的版本号。
  测试键 = 抓到它那条测试（nextest 的「二进制 id 用例名」，libtest 的照日志里的 Running / Doc-tests 行认出二进制）的单元闭包键；
  覆盖者键 = 静态闭包里有容器单元的每一条测试（第一段的、慢档用例、checker 档、被变异包的文档测试；排除表里的不算）的测试键多重集。
沿用判据（mutation-prune.py 的 carry_decision）：判定库里键相同的那几行，抓到的另要测试键与此刻的相同、那条测试不在排除表里（升档判的抓到带档位，
  另要覆盖者键相同：超时、撞顶是在碰得到它的那几条测试跑着时出的，它们变了、或有新测试碰到它就现判）；没红的要覆盖者键相同；无效的键相同就沿用；超时、内存撞顶、驱动错不是结论，每趟照选中现判。
  拒算（容器单元找不到、闭包里有 lib_function_units 认不出的形态、抓到它的测试认不出）的条不沿用、每趟现判，成功那句与 --summary 报条数；
  抓到它的测试认不出时那一行另记覆盖者键，--summary 照它核（提交门禁认它：判定的内容与此刻相同、能碰到它的测试一条没变）。

一条一条跑：每条变异一次独立的判定，判完立刻往 verdicts.tsv 追加一行并 fsync（驱动自己不经 eatmydata，只有测试命令经它）。一行十七段，制表符分隔：
  键、序号、包、变异名、结局、哪一段判的（one / two）、抓到它的第一条测试、测试键、覆盖者键、编译秒数、测试秒数、判定日期、跑法、位置、拒算形态、备注、校验列
  跑法：fresh（剪枝里现判）、full（--full 现判）、import:<另一台的跑法>（--import-results 并进来的）。
判定行的校验列（用户 2026-10-03 定：库目录同一个用户手写得了，行带密钥算的校验列、门禁逐行核）：
  校验列 = HMAC-SHA256(密钥, 前十六段照写进表里的样子用制表符接起来、UTF-8)，小写十六进制（result_mac）。驱动写的每一行都带（现判的、导入的）。
  密钥放库目录之外、仓之外：${XDG_CONFIG_HOME:-~/.config}/singlefs/broad-mutation-result.key（DEFAULT_RESULT_KEY），一行 64 位小写十六进制（32 字节随机数），
    属主是自己、权限 600，所在目录 700。没有时，往判定库里写行的用法（真跑、--import-results）开跑前排他新建一份并说一句；只读的（--summary、--list-items）不建。
    密钥落在库目录里、权限不是 600、内容写坏，写行的用法拒（退 2）；只读的照「校验不了」办。
  另一台的密钥（用户 2026-10-26 定「两台交换密钥」）：--import-results 并进来的行带的是另一台那把密钥算的校验列，导入时用另一台的密钥核。
    对端那一份放在本机密钥同一个目录、名字在 .key 前加 .peer：${XDG_CONFIG_HOME:-~/.config}/singlefs/broad-mutation-result.peer.key（peer_result_key：
    由本机密钥的路径推出，自证里是 BROAD_MUTATION_RESULT_KEY 那一份旁边的 .peer.key），内容是另一台那份 broad-mutation-result.key 的原样，
    同样要属主是自己、权限 600、落在库目录之外；两台各自把对方的那一份经私有通道拷到自己这里，不打印、不进仓、不进库目录。
    取不到（不在、权限不对、内容写坏、落在库目录里）拒导入（退 2，一行都不写）；verdicts.tsv 或 gpu-verdicts.tsv 有一行用它核不过（取回来的表被改过，
    或这份对端密钥不是那一台此刻的）整表拒（退 1，一行都不写）；全部核过才去掉对端的校验列、照本机的密钥重签落行。
  读判定库时段数不对的行、校验列对不上的行、没有密钥时的每一行都不算判定（那几条照沿用不了选、续跑时重跑），--summary 把它们报成对不上、退 1；
    提交门禁 gate-verdict 格另用同一把密钥逐行核 verdicts.tsv，一行不过就红。
  管不到的：升档登记表（escalated-verdicts.tsv）的行不带校验列，证据靠日志的 sha256 与驱动、包装那两句，门禁另核。
  结局：caught 抓到、missed 没红（两段都绿）、unviable 无效（编译那一步编不过）、timeout 超时、memory-cap 内存撞顶、stage-one-only 只过了第一段、
        driver-error 驱动错（某一步退了认不出的码，不是这条变异的结局）。
  一条的状态不取最后一行：键相同、沿用判据认的那几行里有 caught / missed / unviable 的取其中最后一行；没有而有 timeout / memory-cap 的取其中最后一行
  （环境型的结局不盖掉前面的定论）；都没有而有 stage-one-only 行的是 stage-one-only；只有 driver-error 行的是 driver-error，续跑时重试；一行都没有是 none。
  末行没写完（没有换行结尾）不算数，下一次追加之前截掉并说一句。
两段式：第一段每条只跑 harness 档四个包的测试（eatmydata + cargo nextest，遇第一个失败即停）；第一段没红的先落一行 stage-one-only，再进第二段。
  第二段只补第一段没跑到的（用户 2026-09-30 定）：被变异的两个包的文档测试（cargo test --doc）、harness 档四个包里标了 #[ignore] 的慢档用例
  （与第一段同一种起法，eatmydata + cargo nextest 加 --run-ignored only，再用 -E 排掉排除表里的，步名 two-test-slow-harness）、checker 档快档（cargo test --release -p singlefs-checker-tier --lib --tests）；
  都没红才记 missed。checker 档标了 #[ignore] 的全量崩溃枚举不在这里（另列第三段，驱动还没做）。第二段被打断的条，续跑时只补第二段。
GPU 那一趟（C489；用户 2026-10-22 定「第二段补一趟带 gpu 特性的快档」，出处 .claude/kb/checks-owed.md 的 C489、records/2026-09-30-里程碑三开工第一轮.md 的 GPU 普查）：
  CPU 那一趟（上面的两段）全跑完之后，对「CPU 那一趟最后落的结论是没红、又有带 gpu 特性的测试够得着」的条另补一趟 checker 档快档（verification.md「checker 档自己分快档与全量」带特性那一趟）：
    cargo test --release -p singlefs-checker-tier --features <tests/ 里 cfg 点到的特性> --lib / --test <挂特性的测试文件>… -- --nocapture --test-threads 1
    特性与挂特性的文件每趟从 checker 档 tests/ 顶层每个 .rs 的 #[cfg(…)] / #![cfg(…)] 现读（gated_test_files，同门禁 checker-tier-crash-point-replay 的 feature-gated-tests 格），
    一个文件都不挂特性、或特性里没有 gpu：没有这一趟。每条先编（cargo test --no-run，编不过记无效）再跑测试步；测试步整条经 scripts/crash/gpu-quiesce-run.sh 照多机配置清场并回读，
    跑完不复原（要复原手动跑多机配置里的私有复原脚本），GPU 测试各占卡的显存所以 --test-threads 1；BINDGEN_EXTRA_CLANG_ARGS 取 scripts/crash/crash-amplification-lib.sh
    的 CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS（与崩溃放量同一个值，编译产物才共用）；每一步仍经内存包装（--memory-cap、--threads 同 CPU 那一趟），整步限时 =
    max(--timeout-floor, --timeout-multiplier × 这一趟开跑时在没改过的树上量的秒数)。
  要不要这一趟：一条两趟里任一趟红算抓到，所以只给 CPU 那一趟没红（missed）的条跑；还要有一条这一趟跑得到的测试（checker 档的库与挂特性的测试文件里不标 #[ignore] 的）带 gpu 特性的静态闭包
    含这条变异的容器单元、又真引到 gpu 原子（feature = "gpu" 带真值 true，闭包键里 cfg 原子的那一项）——改坏对象落在带 gpu 特性才编进来的代码、或判器单元闭包含它，都是这一条的特例；
    够不着它的、够得着却不引 gpu 的，这一趟帮不上，照旧只有 CPU 那一趟。要跑的目标取这些测试所在的目标（lib、挂特性的文件）。
  结局：编不过 unviable，测试步退 101（有测试没过）caught（抓到它的测试取第一条没过的；--nocapture 加 --test-threads 1 下结果行常被测试自己的输出打散，认不出就读收尾的 failures: 名单），
    全过且跑到了测试 missed；日志里有 NOT_RUN_NO_GPU_ADAPTER（GPU 对拍没找到卡，打一行就返回成功）、或清场包装退 78（清场回读没过，测试没起）是本次未跑：不落行、不算没红，
    BROAD_MUTATION_GPU_NOT_RUN 逐条报；退 0 而一个测试都没跑到记 driver-error（不算没红）；超时（253）、内存撞顶（250）记 timeout / memory-cap（没结论，这一趟不做升档：
    加大 --timeout-floor / --memory-cap 再 --item <序号> --rerun）；内存包装自己的结局（245、251、252、254）整趟停，退 3，不落行。
  基线：这一趟要跑的目标先在没改过的树上跑一遍（编译步、测试步各一次），不绿拒（退 6，基线不绿后面每一条抓到都不作数）；基线里就见到 NOT_RUN_NO_GPU_ADAPTER 或清场没过，
    整趟本次未跑；这一趟的基线不记库，每趟现跑。
  两趟的结局分开记：CPU 那一趟的行照旧只在 verdicts.tsv（这一节的函数不在 JUDGING_ENTRIES 的闭包里，CPU 那一趟的判法单元指纹不因它们变，库里 CPU 那一趟的判定一条都不作废），
    GPU 那一趟的行在 gpu-verdicts.tsv；一条的合起来的状态（gpu_combined_state）：CPU 没红、要 GPU 趟的条，GPU 抓到 / 没红 / 无效就是定论，超时 / 内存撞顶 / 驱动错是没结论，
    库里还没有它的 GPU 结论是 none（理由 gpu-not-run，不算没红）；别的条就是 CPU 那一趟的状态。--summary 的八栏、MISSED / NO_CONCLUSION 明细、提交门禁那一行和 --list-items 的状态都按合起来的状态数。
  沿用：GPU 判定行的键是变异的单元键，另带 gpu_key（这一趟的写法、编进来的特性、GPU 趟的判法单元指纹 gpu_judging_key，从 GPU_JUDGING_ENTRIES 收，与 CPU 那一趟的指纹分开）；
    抓到的要抓到它的测试此刻带特性的测试键（它的单元闭包键）与行里的相同，没红的要覆盖者键（这一趟够得着它的那些测试各自测试键的多重集）相同，无效的键相同就沿用，
    超时 / 内存撞顶 / 驱动错不是结论；CPU 那一趟沿用不了重判过的条，GPU 那一趟照自己的键判；--full、--rerun 下 GPU 那一趟也不读库、现判；复用哨兵 GPU 趟另有一份（项名 gpu:<序号>）。
  导入：两台各跑一片时，GPU 趟在拿卡的那一台（或两台）上跑、判定在那一台库目录的 gpu-verdicts.tsv；--import-results 把那一份并进本机的 gpu-verdicts.tsv（跑法加 import: 前缀、校验列先用另一台的密钥核过再照本机密钥重签，
    本机已有的相同行不重复导），并之前逐行核键（行的键在本机此刻清单的单元键里、gpu_key 等于本机此刻的 GpuJudging.pass_key，两把键见上「沿用」），核不过的不并、列出来；
    并进来之后本机不带 --shard 的一趟里，键相同、gpu_key 相同的行照常沿用，GPU 趟只现判没有结论的条（沿用判据的其余几问——测试键、覆盖者键——仍在用到时现核，核不过的那一条照现判）。
  没做：升档（超时、撞顶两趟取证）。
  CPU 那一趟的前提（不归这一节）：mutation-prune.py 的 coverers_key 跳过默认 cfg 下不编的测试（tests/ 里整个文件 #![cfg(feature = "gpu")] 的，entry-cfg-inactive；
    它们不可能是 CPU 那一趟的覆盖者，不算拒、不进条数），CPU 那一趟没红的行的覆盖者键因此算得出，GPU 那一趟才选得到它们；别的拒因仍整条拒（状态 none，理由 coverer-refused）。
    弄坏开关 coverers-cfg-inactive-refused（BROAD_MUTATION_BREAK）：不跳过、照旧整条拒，红在 mutation-prune.py --selftest 的覆盖者键那一格；本驱动的 --selftest 不另设这一格。
忽略用例登记：harness 档四个包（crates/<包>/src 与 tests 下）里每一条 #[ignore] 用例只有两种归宿，要跑第二段的这一趟开跑前（基线之前）按专用树里的源码逐条判：
  理由以「harness 慢档用例：」开头的，进第二段慢档用例那一步；按环境变量起的大档与工具（没给环境变量就红、或规模由环境变量定）写在排除表
  scripts/mutation/broad-mutation-ignored-exclusions.tsv，一行一条：<二进制 id><空格><用例名><TAB><理由>，二进制 id 与用例名照 cargo nextest list 的写法，
  第二段那一步用 -E 'not ((binary_id(=…) & test(=…)) | …)' 把它们排掉。下面每一种判红退 1、一步不跑：
  新加的忽略用例两样都不是（没登记）、排除表指着树里没有的用例、排除表里的用例理由又写着「harness 慢档用例：」（两边都在）、排除表有写坏（不是两段、名字里有不认的字符、理由是空）或重复的行、
  认不出写法的 #[ignore]（多行的、带条件的）。只跑第一段的趟（--skip-stage-two）不判。--check-ignored 只做这一判、打第二段那一步的完整命令。
  第二段那一步的步名是 two-test-slow-harness（旧步名 two-test-ignored 是没排除大档的那一版命令，基线表里旧名的行不认）。
中断续接：任何时刻被 TERM / KILL，重跑同一条命令：沿用得了的条一条都不重跑（带 --item 的一样）；被打断的那一条没有新行、只重跑它（或只补它的第二段）；
  被 KILL 的那一趟的运行目录（改过的树、在途记录、编译目录）下一趟拿到锁之后先删掉，这一趟在新建的运行目录里照源码换树（库目录「runs/」）。
  被 TERM / INT 停：在途那一步连同内存包装给它开的 scope 一起停（systemctl --user stop 自己那一个 scope），不留着它跑完。
  被 KILL：留下的那一步还活着时占着库目录的锁（锁的描述符在驱动与内存包装最外层那个 bash 手里）：等 --lock-wait 秒，还占着就退 4、不跑。
基线：每一步、每个线程数（--threads）按基线键先在没改过的树上跑一遍，全绿才开始跑变异（基线红了一律退 6）；基线的秒数定这个线程数下的超时。
  取基线按（基线键, 步名, 线程数）取最后一行：是绿的、逐条时间表与峰值都在就沿用，不然在这个线程数下重跑那一步、追加一行；别的线程数的行不动、不盖。
  基线键（Judgement.baseline_key，D13 已定项 17）= 这一步真跑到的每一条判定各自单元闭包键的多重集、这一步的命令（去掉线程数）、全局那几行（判法单元指纹除外）：
    判定是测试函数（mutation-prune.py 的 test_entries 静态列：第一段 harness 档四个包不标 #[ignore] 的，慢档用例那一步标了的、去掉排除表里的，
    checker 档快档 singlefs-checker-tier 不标的）、这几个目标里的条目级宏调用（宏展开出来的测试静态列不出，把宏调用连同它的闭包算进去）、
    被变异两个包的文档测试；checker 档快档带 --release，闭包照 cfg 去掉 debug_assertions 收。闭包键里有闭包单元的种类与去注释指纹、闭包真引到的
    cfg 原子、依赖版本行、工具链那几行；在这一步的 cfg 与特性下不编的判定不算这一步跑到的。不带步名、手写的版本号、包级与文件级指纹：
    改一个不在这一步任何判定闭包里的单元，基线照样沿用。
  拒算（照跑、不沿用、不退回包级）：编译步（编得过与否、用时随它编的那几个包里每一个单元变，按单元闭包切不开）、有一条判定的闭包拒算
    （读运行期拼出的路径、起自己这个可执行文件这一类，lib_function_units.py 认不全）、认不出跑哪些判定的步。拒算的步这一趟用 run_token 算一把
    这一趟独有的键：基线行与升档登记照常记、照常对得上，以后哪一趟都对不上。stdout 那一句写「基线键拒算（为什么）」。
  按条记基线（经 runner 的三个测试步里，基线键只因为有判定的闭包拒算而算不出的；Judgement.baseline_entry_plan、Runner.ensure_entry_baseline）：
    不整步照跑。nextest 那两步一条测试一份记录（键 = 去掉线程数的命令、全局那几行、它自己的单元闭包键），有条目级宏调用的二进制另一份
    「宏调用展开的那几条」（键 = 那个二进制里宏调用的闭包键多重集）；checker 档快档 runner 按二进制量 CPU 秒，一个二进制一份记录（键 = 它里面闭包
    算得出的判定与宏调用的闭包键多重集；里面有拒算的，量的是 --exact --skip 掉它们的那一趟）。每份记录照（记录键、kind、线程数）在
    baseline-entries.tsv 找最后一行，找得到的沿用它记的 CPU 秒；找不到的与拒算的那几条在没改过的树上照跑（nextest 用 -E 只点它们，
    第二段慢档用例那一步与排除表的 -E 取交；checker 档拒算的那几条另起一趟 --exact 只跑它们，那个二进制的 CPU 秒 = 沿用或现量的那一份加这一趟），
    跑绿的照键追加一行；宏调用的闭包拒算的二进制整个照跑。名字与二进制只当索引、不进键（rest 另按二进制找：改名只会重跑）。
    这一步的逐条时间表由沿用的与现跑的拼成；baselines.tsv 照常追加这一趟那一行（这一趟独有的键，峰值取现跑那几趟与沿用的记录里最大的）。
    stdout 写「按条记基线：N 份记录沿用 M、照跑 K；拒算的 R 条每趟照跑（头几条）」。文档测试那一步不经 runner、量不了逐条，照旧整步。
    今天的树上第一段测试步 3 条、checker 档快档 15 条拒算（读 env!("CARGO_MANIFEST_DIR") 拼出的目录或运行期给的文件名、起自己这个可执行文件），
    lib_function_units.py 与 crates/singlefs-unit-graph 逐条同义，那几种形态两边都拒算，不在这里另认。
  每一趟的编译目录是新建的（库目录「runs/」）：头一回编（验环境与编译步的基线）从头编，之后各条在这一趟里只付增量编译；不照用上一趟编的产物。
  每次重起（这一趟没现跑基线的那一段）先在没改过的树上把编译那几步各跑一遍，红了退 6：基线绿过之后环境坏了要在落行之前发现。
  编译那一步退 101 而日志里没有带源码位置的编译错误（磁盘满、锁文件不符这一类），不记无效，记 driver-error 并当场在没改过的树上验一次编译。
内存：每一步命令各自经 scripts/hosts/run-with-memory-cap.sh 跑（与 mutate.sh、门禁 checker-tier-crates-mutation-replay 同一种套法），
  驱动自己不在 scope 里：整条包一层的话一条无界分配的变异会连驱动一起杀掉，续跑又撞同一条、永远过不去。
  撞这一步自己的上限（退 250）：systemd 只停这一步的 scope，记 memory-cap（再照「升档」重跑一次）。
  撞 slice 的总上限（几条合起来超了）：内核在整个 slice 里挑一条杀，挑中的可能是 slice 里别人那一条（别的会话的重活）。所以：
    改坏了源码的那几步带 RUN_WITH_MEMORY_CAP_OOM_FIRST（MUTATION_STEP_OOM_REASON），包装把它的 oom_score_adj 写成 1000，撞总上限时内核先杀它；
    升档那趟带 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP（ESCALATED_QUEUE_REASON），包装按它自己的上限排队（不按峰值表里的小峰值），放不下就等、等满或按上限算
    比 slice 总上限还大就退 252，不放进去再撞总上限；撞过顶的键包装也按上限排队（scripts/hosts/run-with-memory-cap.sh 文件头「排队」）。
    基线与验环境那几步两样都不带。变异步被总上限挤掉（退 254）是包装自己的结局，照「退出码」3 办；重跑时那个键已记成撞过顶、按上限排队。
  超时用包装的 RUN_WITH_MEMORY_CAP_TIME_LIMIT（退 253）。
  基线那几步另把 RUN_WITH_MEMORY_CAP_PEAKS 指到 baselines/<基线键>/peaks.<线程数>.tsv、键 broad-mutation baseline <步名>（BASELINE_PEAK_KEY_PREFIX）：
  每一步开跑前删掉它，跑完读出包装记的峰值、写进 baselines.tsv 那一行的峰值一段，升档的基线峰值取那一行。基线那一步记过绿而这个线程数下那一行没有峰值的，真跑重跑那一步的基线。
  基线那几步的告警照旧叫醒主 agent：峰值表指到默认之外时包装把告警写到峰值表旁边（scripts/hosts/run-with-memory-cap.sh 文件头「告警跟着峰值表走」），所以环境里还没有
  SINGLEFS_CORE_USAGE_ALERTS 的，驱动把它设成 scripts/hosts/core_usage.py 的 alerts_path() 给的真记录（按文件路径导入那一份取，real_alerts_record）；调用方已经设了的照调用方的。非基线的步不设。
升档（用户 2026-10-03 定：超时与内存撞顶不交用户判，由驱动凭证据自动判）：
  一条变异某一步退 253（超时）或 250（撞了内存上限），驱动不换参数、当场在同一棵改过的树上把那一步升档重跑一次（escalate_step）：
    超时：限时 = max(ESCALATION_TIME_FACTOR 倍原限时, ESCALATION_TIME_BASELINE_MULTIPLE 倍基线秒数) 向上取整，内存上限不变；
          没有整步限时的步（经 runner 按单条限 CPU 的测试步，time_limit_of 交 None）不升档，照旧记超时。
    撞顶：内存上限 = max(ESCALATION_MEMORY_FACTOR 倍原上限, ESCALATION_MEMORY_BASELINE_MULTIPLE 倍基线峰值) 按 MiB 向上取整（写成 <MiB>M），限时不变。
  升档那趟跑完（退 0）接着跑这一段后面的步，结局照升档那趟往后判（测试没过记抓到、都过了记没红，与不升档的判法同一份）；
  升档那趟另退一种结局（测试没过、编不过、认不出的码）照那一种判；升档那趟是内存包装自己的结局（251、252、254）不停整趟，照原结局记、备注写没跑成。
  两趟都超时（都退 253）或都撞顶（都退 250），而证据齐，判抓到：判定行结局 caught、「抓到它的测试」一段写标签 caught-by-timeout / caught-by-memory，
    登记表 escalated-verdicts.tsv 追加一行；证据不齐的不写登记表，判定行照旧记 timeout / memory-cap（没结论，备注写缺哪一样）。证据（escalation_evidence_problems 判）：
    ① 同一把基线键、同一步、同一个线程数在没改过的树上的基线：baselines.tsv 里那一步最后一行绿记录，超时取它的秒数，撞顶取它的峰值字节；
       登记行记那一行的行号（baseline_row）与基线键、取到的值，之后追加的行不动它；
    ② 两趟都退同一种（都 253 或都 250），两趟的日志都在、各带包装打的那一句（「超过限时 <限时> 秒」「撞了内存上限 <上限>」）；
    ③ 升档那趟的限时或上限比原来的大，且不低于基线的 K 倍：超时 K = ESCALATION_TIME_BASELINE_MULTIPLE（20），撞顶 K = ESCALATION_MEMORY_BASELINE_MULTIPLE（2）；
    ④ 两趟日志挪进库目录的 escalation/<序号>.<步名>.<一次升档的号>.{first,escalated}.log，末尾由驱动追加一行
       BROAD_MUTATION_EVIDENCE key=<那条变异的键> sequence=<序号> step=<步名> attempt=<first / escalated> limit=<那一趟的限时或上限> exit=<退出码>，
       登记行记两份的路径（相对库目录）与 sha256；⑤ 登记行的 key 是那条变异的键。
  K 为什么够：超时那一边，默认参数下 K 这一道恒满足、不咬：原限时 = max(--timeout-floor 900 秒, 5 倍基线)，升档那趟 = max(4 倍原限时, 20 倍基线) 天然不低于 20 倍基线。
    挡住「机器慢」被判成抓到的是两个绝对数：两趟里头一趟已经过了 900 秒、升档那趟过了不少于 3600 秒；真库的基线是几秒到几十秒。
    K 只在调小 --timeout-floor 或 --timeout-multiplier 时起作用。测试步那一边「超 3 倍基线 CPU 加 30 秒算抓到」（用户定「卡住算抓到」）比它松得多。
    撞顶那一边，包装记的峰值含页缓存，基线峰值偏大不偏小；两趟都在不低于它 2 倍的上限里被 OOM 杀，是这条变异自己要的回收不掉的内存比整步没改过时的总占用还多一倍。
  登记表 <库目录>/escalated-verdicts.tsv：只追加，一行一条升档判的抓到，列见 ESCALATION_COLUMNS。只由驱动写，手写的行门禁判红。
    升档判的抓到照别的判定一样进判定库、按键沿用（带档位：抓到它的测试一段是标签；那一行另记覆盖者键，碰得到它的测试变了就现判）；--summary 对状态是升档判的抓到的条逐条打 ESCALATED <TAB> 序号 <TAB> 标签 <TAB> 变异名 <TAB> 键，
    判定行带 escalated=<条数>；提交门禁 gate-verdict 格逐行复核登记表里那把键的行（倍数门槛按此刻驱动里的常量、日志在且 sha256 对、带驱动与包装打的那两句、键对；
    基线按登记行的 baseline_row 找 baselines.tsv 的那一行：基线键、步名、绿、线程数都与登记的相同，秒数或峰值与登记的相同），每一条 ESCALATED 都要有一行复核过的登记行。
前缀：起 cargo 的与改库目录的（跑、--full、--refresh-list、--import-results）都要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，不带退 2；
  --list-items、--summary、--check-ignored、--help（-h）、--selftest 不要。重型测试闸同样这样分（.claude/hooks/lib_heavy_tests.py 文件头「广谱变异的逐条驱动」）。

按覆盖挑测试（C2；规则在 scripts/mutation/mutation-prune.py）：每趟在没改过的树上插桩列一遍测试（--coverage-step，驱动自己起、经内存包装；第一段与慢档用例逐条测试、
  checker 档逐个二进制，E6），覆盖记录键对得上的那几条照用 coverage.tsv 里的、对不上的才插桩跑；每条变异先编，再只跑执行到它的容器单元的测试
  （-E 按块切，避开单参数 131072 字节上限），遇第一个失败即停；零覆盖记没红（备注写零覆盖）。
  例外一律跑全部（E1）：容器不是 fn 单元的变异、常量上下文里用到的 const fn、起子进程的测试二进制、文档测试（第二段每条都跑）。
  覆盖要 llvm-profdata、llvm-cov：取 rustup 组件 llvm-tools（rustup component add llvm-tools；与编译器同一份 LLVM），装在别处给 --llvm-bin <目录>；它的版本进全局那几行。
限时（C4）：测试步经 runner（库目录 test-runner.py，CARGO_TARGET_<三元组>_RUNNER）逐条测试（checker 档逐个二进制）按 CPU 时间限：
  --cpu-multiplier（默认 3，不许小于 2：盖住 SMT 兄弟被占的 2.0 倍）× 基线 CPU 秒 + --cpu-floor（默认 30），墙钟兜底 --wall-multiplier（10）× 基线墙钟 + --wall-floor（300）；
  超了算那条测试没过 = 抓到（用户定「卡住算抓到」），备注写哪一种停的；测试步不再设整步总时限（编译步与文档测试步照旧按 --timeout-floor / --timeout-multiplier）。
  第二段基线红报成退 6 的拒绝，列红的测试与出路（修好之后重跑；在那之前只要第一段的结论就加 --skip-stage-two）。

输出：开跑一行 BROAD_MUTATION_PLAN list=<清单条数> judge=<现判> carried=<沿用> refused=<拒算> selected=<选中> threads=… memory_cap=…，
      要现判的每条一行 BROAD_MUTATION_JUDGE sequence=… reason=…（沿用不了的理由；作废时另带 changed=<变了的单元>）；
      每条判完一行 BROAD_MUTATION_ITEM sequence=… outcome=… stage=… build_seconds=… test_seconds=… wall_seconds=… judged=1 red=<没红记 1，别的记 0> reused=0 name=…
      （入口脚本那一行的判了、红了、复用了多少：这一行只给现判的条打，一条一判，复用的条不打这一行、数在 BROAD_MUTATION_PLAN 的 carried=；C641）；
      末尾一行 BROAD_MUTATION_TOTAL selected=… ran=… judged=<现判> carried=<沿用> refused=<拒算> 各结局条数，成功那句报「现判 N、沿用 M、拒算 R」，后面接「GPU 趟（要 K 条）：现判 a、复用 b、本次未跑 c」；
      GPU 那一趟：BROAD_MUTATION_GPU_PLAN needed=<要这一趟的条数> judge=<现判> carried=<沿用> refused=<拒算> features=<特性>，现判的每条一行 BROAD_MUTATION_GPU_JUDGE sequence=… reason=… labels=<目标>，
      判完一行 BROAD_MUTATION_GPU_ITEM sequence=… cpu=<CPU 那一趟的结局> gpu=<GPU 那一趟的结局> combined=<合起来的状态> build_seconds=… test_seconds=… wall_seconds=… judged=1 red=<GPU 没红记 1> reused=0 name=…，
      本次未跑的每条一行 BROAD_MUTATION_GPU_NOT_RUN sequence=… reason=<no-adapter / quiesce-refused / skip-gpu> name=…，末尾一行 BROAD_MUTATION_GPU_TOTAL needed=… judged=… carried=… not_run=… 各结局条数；
      --full 另打 BROAD_MUTATION_FULL flipped=<条数> 与 FLIPPED 明细；
      --summary 打 BROAD_MUTATION_SUMMARY 一行（八栏与 refused=、unlisted_units=、gpu_needed= gpu_reused= gpu_not_run= gpu_without_conclusion=）与 MISSED、DRIVER_ERROR、REFUSED、UNLISTED 明细，
      BROAD_MUTATION_GATE 一行（total、concluded、missed、without_conclusion、refused、unlisted_units、escalated、gpu_needed、gpu_reused、gpu_not_run、gpu_without_conclusion、verdict）与 NO_CONCLUSION、ESCALATED 明细；
      gpu_needed 是要 GPU 那一趟的条数（CPU 那一趟没红、又有带 gpu 特性的测试够得着），它 = gpu_reused（库里键相同、沿用判据认的 GPU 结论）+ gpu_not_run（还没有 GPU 结论：没卡、没来得及跑）+
      gpu_without_conclusion（GPU 那一趟超时 / 内存撞顶 / 驱动错）；后两种的条在 without_conclusion 里、NO_CONCLUSION 明细的理由写 gpu-…，不在 missed 里；
      NO_CONCLUSION 明细的理由后面接「上一趟限时 <N> 秒」（last_time_limit_text：定下状态的那一行备注里记的整步限时，超时之后升档重跑过的取升档那趟的；
      备注里没有整步限时的不接：直接撞顶的、驱动错、GPU 趟的、经 runner 按单条限 CPU 的测试步），之后才是作废时变了的单元；
      升档那一步各打一行 BROAD_MUTATION_ESCALATION sequence=… step=… first=… escalated=… exit=…；--import-results 打 BROAD_MUTATION_IMPORT 一行（CPU 那一趟的判定行：rows_read、imported、already_present、conflicting_keys…；
      GPU 那一趟的：gpu_rows_read、gpu_imported、gpu_already_present、gpu_conflicting_keys、gpu_withheld_rows、gpu_rejected、gpu_partial_tail_ignored）与 IMPORT_CONFLICT、
      IMPORT_GPU_CONFLICT（<TAB>键前十六位<TAB>本机结局<TAB>另一台结局<TAB>变异名）、IMPORT_GPU_REJECTED（<TAB>序号<TAB>键前十六位<TAB>理由 key-not-in-list / gpu-pass-changed<TAB>变异名<TAB>说明）明细。
      一步跑着时每过 PROGRESS_INTERVAL_SECONDS 秒（300）往 stderr 打一行
      BROAD_MUTATION_PROGRESS sequence=<序号，基线是 baseline，验环境是 environment> step=<步名> elapsed_seconds=… compiled_crates=… tests_passed=… tests_failed=…
退出码：0 选中的条都判完；1 --summary 栏数对不上或判定库有对不上的行（段数、校验列、没有密钥）、--import-results 有冲突的条或键对不上被拒的 GPU 判定行（别的行已导入）、忽略用例登记对不上（含排除表写坏）、
  清单按单元存复原不回去、清单里有落不进任何单元的变异、--selftest 有格判错；
  2 用法错（项写错、库目录不合或是旧库、没带前缀、缺工具、真跑起 cargo 之前 RocksDB 预编库的取值判红（scripts/hosts/rocksdb-prebuilt.py entry：这台没装、
    此刻进程环境的取值缺了或对不上；出路是开新会话，或命令前 eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"）、只供自证的变量出现在自证之外、要写行而密钥用不了、传了已删的参数）；
  3 内存包装自己的结局（251 起不来、252 排不上、254 被总上限挤掉、245 显存不够）：那一条不落行，修好重跑同一条命令；
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
      红：「✗ 自证：不带参数、库里有可沿用的判定时没有沿用」、「✗ 自证：同一条变异内容没变、判定库里有可沿用的行、所在文件在某提交之后改过别处，没照样沿」、「✗ 自证：抓到它的那条测试改了之后那几条没现判」、「✗ 自证：改一条排除之后别的条没照样沿用」、「✗ 自证：只改 Cargo.toml 注释之后别的条没照样沿用」、
      「✗ 自证：加大限时参数与上限之后别的条没照样沿用」、「✗ 自证：拒算的条没有每趟现判、或别的条没沿用」、「✗ 自证：剪枝跑没有重跑超时、内存撞顶的条」、「✗ 自证：--stage-two-only 不对」、「✗ 自证：重起之后沿用得了的条或基线被重跑了」、「✗ 自证：--shard 0/2 跑的不是沿用不了的偶数序号」、
      「✗ 自证：点名沿用得了的条没跳过」、「✗ 自证：点名加 --rerun 没有重跑并追加新行」、「✗ 自证：第二段的定论被后来的 stage-one-only 行降级了」、「✗ 自证：抓到的条被后来的超时行盖掉了」、「✗ 自证：--stage-two-only 点到没过过第一段的条没拒」、「✗ 自证：认不出的退出码没处理对」、
      「✗ 自证：工具链没进每一把键」、「✗ 自证：--summary 没列升档判的抓到、或下一趟没按键沿用」、「✗ 自证：--unit-environment 打的不是构建环境认的那几个变量」、「✗ 自证：--unit-environment 没把换了工具链的 RUSTUP_TOOLC」、「✗ 自证：两台各一片的结果没合对」
      另红：「✗ 自证：reuse-three-questions：」（① 第二趟全现判）、「✗ 自证：reuse-sentinel：」（这一格要的就是那一趟复用哨兵判红；跟着判错的是照常那一趟）
      另红：「✗ --stage-two-only 点到的第 2、7 条在判定库里没有键相同的 stage-one-only 行」、「✗ 复用失灵：…键相同却现判了」、「✗ 自证：同一条命令第二趟 GPU 那一趟没复用」、「✗ 自证：有卡之后重跑同一条命令没把 GPU 那一趟补上」、「✗ 自证：键对不上的 GPU 判定行没被拒、或拒了之后本机的 GPU 趟没把它照现判」、「✗ 自证：第二台的 GPU 判定行没随导入并进本机库、或并进来之后本机的 GPU 趟还是把同一批判了一遍」
  REUSE_CHECKS_BREAK=rejudged-sentinel-passes（共用库 scripts/gate/lib_reuse_checks.py 的开关，哨兵被现判不红）
      红：「✗ 自证：reuse-sentinel：」
  BROAD_MUTATION_BREAK=interrupted-as-done  被打断的那一条重起时照「已完成」落一行、不重跑
      红：「✗ 自证：被打断的第 3 条没有重跑、或被当成已完成」、「✗ 自证：两段式的结局不对」、「✗ 自证：汇总不对」、「✗ 自证：末行没写完的判定库没处理对」
      另红：「✗ 提交门禁判红：…条沿用不了、也没有现判的结论」
  BROAD_MUTATION_BREAK=no-row-flush  判定行攒在内存里到收尾才写
      红：「✗ 自证：杀驱动之前，前两条的判定行没有都落盘」、「✗ 自证：重起之后沿用得了的条或基线被重跑了」
  BROAD_MUTATION_BREAK=stage-two-missed-as-caught  第二段没红记成抓到
      红：「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑、测试步经 runner 限 CPU 不对」、「✗ 自证：不带参数、库里有可沿用的判定时没有沿用」、「✗ 自证：同一条变异内容没变、判定库里有可沿用的行、所在文件在某提交之后改过别处，没照样沿用」、
      「✗ 自证：抓到它的那条测试改了之后那几条没现判」、「✗ 自证：--full 没有只写不读、或没列出翻面的条」、「✗ 自证：reuse-three-questions」、「✗ 自证：reuse-sentinel」、
      「✗ 自证：只有 CPU 红：CPU 那一趟抓到的条也起了 GPU 那一趟、或两趟的结局没分开记」、「✗ 自证：GPU 那一趟的结局没有单独存进 gpu-verdicts.tsv」、
      「✗ 自证：只有 GPU 红：GPU 那一趟红了没算抓到、抓到它的测试没认出来、或 --list-items 的状态不是两趟合起来的」、「✗ 自证：两趟都没红：没记成没红、或 GPU 那一趟跑了闭包够不着的条」、「✗ 自证：GPU 那一趟的命令不对」、
      「✗ 自证：没卡：没报本次未跑、记成了没红、或落了 GPU 行」、「✗ 自证：有卡之后重跑同一条命令没把 GPU 那一趟补上」、「✗ 自证：中途没卡：这一条之后的没停下、或落了行」、「✗ 自证：--skip-gpu 还是起了 GPU 的步、或没逐条报本次未跑」、「✗ 自证：清场没过」、
      「✗ 自证：GPU 趟基线红没拒成「基线不绿」」、「✗ 自证：GPU 趟超时 / 一个测试都没跑到不是没结论」、「✗ 自证：GPU 趟编不过没记成无效」、「✗ 自证：同一条命令第二趟 GPU 那一趟没复用」、「✗ 自证：点名加 --rerun 没有重跑 GPU 那一趟并追加新行」、
      「✗ 自证：手追加的没有校验列、校验列对不上的 GPU 判定行没判红」、「✗ 自证：BROAD_MUTATION_ITEM 每条一行没带判了、红了、复用了多少，或 red= 与结局对不上」、「✗ 自证：两段式的结局不对」、「✗ 自证：汇总不对」、「✗ 自证：旧步名的基线行被认了」
      另红：「✗ 提交门禁判红：…条沿用不了、也没有现判的结论」、「✗ 自证：两台各跑一片的 GPU 导入格前提不成立」
  BROAD_MUTATION_BREAK=summary-mismatch-silent  汇总对不上（认不出的结局、校验列、清单表复原不回去）不报
      红：「✗ 自证：判定库里有认不出的结局时汇总没判红」、「✗ 自证：清单按单元存的表被改坏了没判红」、「✗ 自证：手追加的没有校验列、校验列对不上的 caught 行被当成了判定」、「✗ 自证：密钥不在时 --summary 没判红、或替它建了一份」
      另红：「✗ 自证：手追加的没有校验列、校验列对不上的 GPU 判定行没判红」
  BROAD_MUTATION_BREAK=interrupted-run-directory-kept  开跑时不删上一趟被 KILL 留下的运行目录、不认它里面的在途记录
      红：「✗ 自证：被打断的第 3 条没有重跑、或被当成已完成」、「✗ 自证：重起跑完之后库目录里还剩上一趟被 KILL 留下的、或这一趟的运行目录」、「✗ 自证：上一趟被 KILL 留下的运行目录」、
      「✗ 自证：恢复原件半途被杀之后树里多出了文件、临时件不在运行目录里、或重跑没照常判完」
  BROAD_MUTATION_BREAK=accept-unknown-item  不认得的项悄悄略过
      红：「✗ 自证：不认得的项没拒」
  BROAD_MUTATION_BREAK=exclude-re-ignored  换树时不把 .cargo/mutants.toml 拷进专用树（cargo-mutants 读不到 exclude_re，等于忽略它）
      红：「✗ 自证：exclude_re 没排掉那一条、清单没整版重判、或别的条没沿用」
  BROAD_MUTATION_BREAK=no-conclusion-limit-dropped  --summary 的 NO_CONCLUSION 理由不接上一趟的限时（改前的样子）
      红：「✗ 自证：--summary 的 NO_CONCLUSION 超时那一行理由里没带上一趟的限时秒数」
  BROAD_MUTATION_BREAK=item-counts-dropped  每条一行 BROAD_MUTATION_ITEM 不带 judged=、red=、reused=（改前的样子）
      红：「✗ 自证：BROAD_MUTATION_ITEM 每条一行没带判了、红了、复用了多少」
  BROAD_MUTATION_BREAK=test-variables-anywhere  只供自证的变量隔一层进程、库目录在自证工作目录之外也认
      红：「✗ 自证：只供自证的变量指着自证工作目录之外的库时没拒」、「✗ 自证：父进程不是 --selftest 的进程带着假一步的变量时没拒」、「✗ 自证：父进程不是 --selftest 的进程只带着弄坏开关时没拒」
      另红：「✗ 自证：平常跑设了弄坏开关、带认不得的选项时没在参数解析之前拒并点名」
  BROAD_MUTATION_BREAK=break-guard-after-parsing  只供自证的变量（含这个弄坏开关）平常跑设了的拒排回参数解析之后（改前的样子：认不得的选项先被 argparse 当用法错拒，不点名变量）
      红：「✗ 自证：平常跑设了弄坏开关、带认不得的选项时没在参数解析之前拒并点名」
  BROAD_MUTATION_BREAK=state-is-last-row  一条的状态照旧取最后一行
      红：「✗ 自证：第二段的定论被后来的 stage-one-only 行降级了」、「✗ 自证：抓到的条被后来的超时行盖掉了」
  BROAD_MUTATION_BREAK=named-items-always-rerun  点名的条沿用得了也重跑
      红：「✗ 自证：点名沿用得了的条没跳过」、「✗ 自证：点名加 --rerun 没有重跑并追加新行」、「✗ 自证：抓到的条被后来的超时行盖掉了」、「✗ 自证：--summary 没列升档判的抓到、或下一趟没按键沿用」、「✗ 自证：--unit-environment 打的不是构建环境认的那几个变量」、
      「✗ 自证：--unit-environment 没把换了工具链的 RUSTUP_TOOLC」
      另红：「✗ 自证：有卡之后重跑同一条命令没把 GPU 那一趟补上」
  BROAD_MUTATION_BREAK=partial-row-kept  末行不完整时照旧补一个换行留着它
      红：「✗ 自证：末行没写完的判定库没处理对」
      另红：「✗ 汇总对不上：判定库里没有校验列或校验列对不上的…行不算」
  BROAD_MUTATION_BREAK=no-environment-check  重起不验编译、连着无效或秒红也不查
      红：「✗ 自证：认不出的退出码没处理对」、「✗ 自证：基线绿过之后编译坏了，重起没发现」、「✗ 自证：连续无效而原件上编译照过时没有接着跑」、「✗ 自证：连续 3 条秒红的抓到、原件上那一步也红，没有停下报疑似环境坏了」
  BROAD_MUTATION_BREAK=unviable-by-exit-code  编译步退 101 一律记无效
      红：「✗ 自证：认不出的退出码没处理对」
  BROAD_MUTATION_BREAK=unknown-exit-stops  一步退认不出的码照旧整趟停
      红：「✗ 自证：认不出的退出码没处理对」
      另红：「✗ 第 3 条的 one-test 退 104（认不出的退出码）：不落行」
  BROAD_MUTATION_BREAK=gpu-memory-as-driver-error  一步退 245（内存包装认出的显存不够）照认不出的码落一行 driver-error、不停
      红：「✗ 自证：一步退 245」
  BROAD_MUTATION_BREAK=stage-two-without-stage-one  点名加 --stage-two-only 不看过没过第一段
      红：「✗ 自证：--stage-two-only 点到没过过第一段的条没拒」、「✗ 自证：父进程不是 --selftest 的进程带着假一步的变量时没拒」
  BROAD_MUTATION_BREAK=stage-two-full-cargo-test  第二段照旧是整遍 cargo test 加快档
      红：「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑、测试步经 runner 限 CPU」、「✗ 自证：--stage-two-only 不对」、「✗ 自证：两段式的结局不对」、「✗ 自证：汇总不对」、「✗ 自证：第二段的步不对」、
      「✗ 自证：第二段的定论被后来的 stage-one-only 行降级了」、
      「✗ 自证：登记对的树上 --check-ignored 没判绿、或第二段那一步的命令不对」、「✗ 自证：旧步名的基线行被认了」
      另红：「✗ 提交门禁判红：…条沿用不了、也没有现判的结论」、「✗ 自证：重起之后沿用得了的条或基线被重跑了」
  BROAD_MUTATION_BREAK=term-leaves-step-running  被 TERM 时只给包装发一次 TERM
      红：「✗ 自证：包装自己不停 scope 时被 TERM 停，在途那一步没有一起停」（selftest_stop_cells 里经包装的弄坏开关 RUN_WITH_MEMORY_CAP_BREAK=term-leaves-scope 起的那一趟：
      包装被 TERM 时照改前不停自己开的 scope，只给包装发 TERM 停不下在途那一步；照常经真包装的那一趟包装自己停 scope，不跟着红。
      那一格转述的驱动输出里驱动那一句被 TERM 停的话，句首的叉号换成了〔转述〕，不另成一行失败行）
  BROAD_MUTATION_BREAK=import-unchecked  导入不核段数、结局与键的写法（verdicts.tsv 与 gpu-verdicts.tsv 两张表都不核）
      红：「✗ 自证：该拒的导入没拒、或别的源码的行顶替了本机的」、「✗ 自证：不成形的 GPU 判定行没整表拒、或拒了还写了别的行」
  BROAD_MUTATION_BREAK=peer-mac-unchecked  导入不用另一台的密钥核它的校验列（verdicts.tsv 与 gpu-verdicts.tsv 都不核，对端密钥取不到也照导：改前的样子）
      红：「✗ 自证：另一台的行被改了一个字（校验列没跟着改）时导入没整表拒」、「✗ 自证：取不到另一台的密钥时导入没拒」
  BROAD_MUTATION_BREAK=import-verdicts-only  导入只并 verdicts.tsv、不看 gpu-verdicts.tsv（改前的样子：第二台的 GPU 判定不随导入回来，本机的 GPU 趟把同一批再判一遍）
      红：「✗ 自证：第二台的 GPU 判定行没随导入并进本机库、或并进来之后本机的 GPU 趟还是把同一批判了一遍」、「✗ 自证：键对不上的 GPU 判定行没被拒、或拒了之后本机的 GPU 趟没把它照现判」、
      「✗ 自证：不成形的 GPU 判定行没整表拒、或拒了还写了别的行」、「✗ 自证：两台定论不同的 GPU 判定行没挡下」
  BROAD_MUTATION_BREAK=import-gpu-keys-unchecked  导入 GPU 判定行不核两把键（行的键在不在本机此刻清单里、gpu_key 等不等于本机此刻 GPU 趟自己的键）
      红：「✗ 自证：键对不上的 GPU 判定行没被拒、或拒了之后本机的 GPU 趟没把它照现判」
  BROAD_MUTATION_BREAK=import-gpu-conflict-ignored  导入 GPU 判定行不看两边定论不同（同一把键本机与另一台各有定论而结局不同的也并进来）
      红：「✗ 自证：两台定论不同的 GPU 判定行没挡下」
  BROAD_MUTATION_BREAK=unregistered-ignored-passes  没登记的忽略用例判绿
      红：「✗ 自证：新加的没登记的忽略用例没判红」
  BROAD_MUTATION_BREAK=stale-exclusion-passes  排除表指着不存在的用例或两边都在判绿
      红：「✗ 自证：排除表指着不存在的用例、或两边都在，没判红」
  BROAD_MUTATION_BREAK=malformed-exclusion-row-skipped  排除表写坏或重复的行悄悄略过
      红：「✗ 自证：排除表写坏的行或重复的行没判红」
  BROAD_MUTATION_BREAK=exclusion-filter-dropped  第二段慢档用例那一步的命令不带排除的 -E
      红：「✗ 自证：登记对的树上 --check-ignored 没判绿、或第二段那一步的命令不对」
  BROAD_MUTATION_BREAK=ignored-scan-skips-nested-modules  扫源码时模块里的 #[ignore] 略过
      红：「✗ 自证：登记对的树上 --check-ignored 没判绿、或第二段那一步的命令不对」（直接判模块里的 #[ignore] 扫没扫到：mod 里的两条没扫到，排除表里那两行成了指着不存在的用例）；
      跟着红的（排除表对不上，要跑第二段的趟都在基线之前被拒，后面的格前提不成立）：「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑、测试步经 runner 限 CPU 不对」、「✗ 自证：不带参数、库里有可沿用的判定时没有沿用」、「✗ 自证：同一条变异内容没变、判定库里有可沿用的行、所在文件在某提交之后改过别处，没照样沿用」、
      「✗ 自证：抓到它的那条测试改了之后那几条没现判」、「✗ 自证：--full 没有只写不读、或没列出翻面的条」、「✗ 自证：第二段基线红没报成带出路的拒绝」、「✗ 自证：reuse-three-questions」、「✗ 自证：reuse-sentinel」、
      「✗ 自证：只有 CPU 红：CPU 那一趟抓到的条也起了 GPU 那一趟、或两趟的结局没分开记」、「✗ 自证：GPU 那一趟的结局没有单独存进 gpu-verdicts.tsv」、
      「✗ 自证：只有 GPU 红：GPU 那一趟红了没算抓到、抓到它的测试没认出来、或 --list-items 的状态不是两趟合起来的」、「✗ 自证：两趟都没红：没记成没红、或 GPU 那一趟跑了闭包够不着的条」、「✗ 自证：GPU 那一趟的命令不对」、
      「✗ 自证：没卡：没报本次未跑、记成了没红、或落了 GPU 行」、「✗ 自证：有卡之后重跑同一条命令没把 GPU 那一趟补上」、「✗ 自证：中途没卡：这一条之后的没停下、或落了行」、「✗ 自证：--skip-gpu 还是起了 GPU 的步、或没逐条报本次未跑」、「✗ 自证：清场没过」、
      「✗ 自证：GPU 趟基线红没拒成「基线不绿」」、「✗ 自证：GPU 趟超时 / 一个测试都没跑到不是没结论」、「✗ 自证：GPU 趟编不过没记成无效」、「✗ 自证：同一条命令第二趟 GPU 那一趟没复用」、「✗ 自证：点名加 --rerun 没有重跑 GPU 那一趟并追加新行」、
      「✗ 自证：手追加的没有校验列、校验列对不上的 GPU 判定行没判红」、「✗ 自证：--stage-two-only 不对」、「✗ 自证：杀驱动之前，前两条的判定行没有都落盘」、「✗ 自证：驱动被杀之后它那一趟的运行目录里没有留着改过的树与在途记录」、「✗ 自证：上一趟留下的进程还占着锁时没拒」、
      「✗ 自证：重起之后沿用得了的条或基线被重跑了」、「✗ 自证：BROAD_MUTATION_ITEM 每条一行没带判了、红了、复用了多少，或 red= 与结局对不上」、「✗ 自证：被打断的第 3 条没有重跑、或被当成已完成」、「✗ 自证：两段式的结局不对」、「✗ 自证：汇总不对」、
      「✗ 自证：第二段的步不对」、「✗ 自证：判定库里有认不出的结局时汇总没判红」、「✗ 自证：第二段的定论被后来的 stage-one-only 行降级了」、「✗ 自证：旧步名的基线行被认了」、「✗ 自证：排除表指着不存在的用例、或两边都在，没判红」、
      「✗ 自证：基线没按条记：一步里收得全的测试没各自沿用、或拒算的那几条没照跑」
      另红：「✗ 提交门禁判红：…条沿用不了、也没有现判的结论」、「✗ 树里的忽略用例登记对不上：排除表里…行指着树里没有的用例」、「✗ 自证：两台各跑一片的 GPU 导入格前提不成立」
  BROAD_MUTATION_BREAK=slow-step-keeps-old-name  第二段慢档用例那一步还叫旧步名 two-test-ignored
      红：「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑、测试步经 runner 限 CPU」、「✗ 自证：--stage-two-only 不对」、「✗ 自证：两段式的结局不对」、「✗ 自证：汇总不对」、「✗ 自证：第二段的步不对」、
      「✗ 自证：第二段的定论被后来的 stage-one-only 行降级了」、
      「✗ 自证：登记对的树上 --check-ignored 没判绿、或第二段那一步的命令不对」、「✗ 自证：旧步名的基线行被认了」
      另红：「✗ 提交门禁判红：…条沿用不了、也没有现判的结论」
  BROAD_MUTATION_BREAK=progress-never  一步跑着时进度间隔的判断永不触发、一行 BROAD_MUTATION_PROGRESS 都不打
      红：「✗ 自证：一步跑过进度间隔没有报进度」
  BROAD_MUTATION_BREAK=coverage-selection-ignored  剪枝：不按覆盖挑测试
      红：「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑、测试步经 runner 限 CPU」
  BROAD_MUTATION_BREAK=stage-two-not-by-binary  剪枝：第二段不按二进制挑
      红：「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑、测试步经 runner 限 CPU」
  BROAD_MUTATION_BREAK=doc-tests-skipped  剪枝：文档测试不跑
      红：「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑、测试步经 runner 限 CPU」、「✗ 自证：两段式的结局不对」、「✗ 自证：汇总不对」、「✗ 自证：第二段的步不对」
      另红：「✗ 提交门禁判红：…条沿用不了、也没有现判的结论」
  BROAD_MUTATION_BREAK=whole-step-time-limit  剪枝：测试步照旧设整步总时限
      红：「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑、测试步经 runner 限 CPU」、「✗ 自证：剪枝跑没有重跑超时、内存撞顶的条」、「✗ 自证：被打断的第 3 条没有重跑、或被当成已完成」、「✗ 自证：两段式的结局不对」、「✗ 自证：汇总不对」、「✗ 自证：抓到的条被后来的超时行盖掉了」、
      「✗ 自证：两趟都撞顶、证据齐的没判成 caught-by-memory」、「✗ 自证：两台各一片的结果没合对」
      另红：「✗ 提交门禁判红：…条沿用不了、也没有现判的结论」
  BROAD_MUTATION_BREAK=cpu-limit-ignored  剪枝：测试步不经 runner 限 CPU
      红：「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑、测试步经 runner 限 CPU」
      另红：「✗ 自证：基线没按条记：一步里收得全的测试没各自沿用、或拒算的那几条没照跑」
  BROAD_MUTATION_BREAK=carry-without-store-check  判定库里键相同就沿用（抓到它的测试键、覆盖者键都不比）
      红：「✗ 自证：抓到它的那条测试改了之后那几条没现判」
      另红：「✗ 自证：reuse-three-questions：」
  BROAD_MUTATION_BREAK=summary-gate-silent  --summary 有没红或没结论也退 0
      红：「✗ 自证：--full 没有只写不读、或没列出翻面的条」、「✗ 自证：剪枝跑没有重跑超时、内存撞顶的条」、「✗ 自证：汇总不对」、「✗ 自证：末行没写完的判定库没处理对」、「✗ 自证：认不出的退出码没处理对」、「✗ 自证：两台各一片的结果没合对」
      另红：「✗ 自证：两趟都没红：」、「✗ 自证：只有 CPU 红：」、「✗ 自证：没卡：」
  BROAD_MUTATION_BREAK=stage-two-baseline-red-generic  第二段基线红只报一句不给出路
      红：「✗ 自证：第二段基线红没报成带出路的拒绝」
      另红：「✗ 基线 two-test-checker-tier 在没改过的树上退 101：基线不绿」
  BROAD_MUTATION_BREAK=environmental-not-reselected  不带参数的剪枝跑不重跑超时、内存撞顶的条
      红：「✗ 自证：剪枝跑没有重跑超时、内存撞顶的条」、「✗ 自证：两台各一片的结果没合对」
  BROAD_MUTATION_BREAK=escalation-skipped  超时、撞顶不升档，照旧记没结论
      红：「✗ 自证：加大 --timeout-floor、--memory-cap 再 --item」、「✗ 自证：两趟都超时、证据齐的没判成 caught-by-timeout」、「✗ 自证：两趟都撞顶、证据齐的没判成 caught-by-memory」、「✗ 自证：升档那趟跑完没照它往后的结局判」、
      「✗ 自证：证据不齐的升档判成了抓到」、「✗ 自证：--summary 没列升档判的抓到、或下一趟没按键沿用」、「✗ 自证：升档那趟没按自己的上限排队、或改坏了源码的那几步没设撞总上限时先挨杀」、「✗ 自证：换了 --threads 之后基线没在新线程数下另记、或登记行指的基线行对不上」
  BROAD_MUTATION_BREAK=escalation-same-limit  升档那趟照原来的限时与上限跑
      红：「✗ 自证：两趟都超时、证据齐的没判成 caught-by-timeout」、「✗ 自证：两趟都撞顶、证据齐的没判成 caught-by-memory」、「✗ 自证：--summary 没列升档判的抓到、或下一趟没按键沿用」、「✗ 自证：换了 --threads 之后基线没在新线程数下另记、或登记行指的基线行对不上」
  BROAD_MUTATION_BREAK=escalation-evidence-unchecked  两趟都撞了什么就判抓到，不核证据（两趟结局不同、基线缺也判）
      红：「✗ 自证：剪枝跑没有重跑超时、内存撞顶的条」、「✗ 自证：两段式的结局不对」、「✗ 自证：汇总不对」、「✗ 自证：加大 --timeout-floor、--memory-cap 再 --item」、「✗ 自证：证据不齐的升档判成了抓到」、「✗ 自证：两台各一片的结果没合对」
      另红：「✗ 提交门禁判红：…条沿用不了、也没有现判的结论」
  BROAD_MUTATION_BREAK=escalated-not-summarised  --summary 不列 ESCALATED、判定行不带 escalated=
      红：「✗ 自证：--summary 没列升档判的抓到、或下一趟没按键沿用」
  BROAD_MUTATION_BREAK=unit-environment-fixed-list  --unit-environment 只认 RUSTFLAGS、RUSTDOCFLAGS、CARGO_PROFILE_* 这几个
      红：「✗ 自证：--unit-environment 打的不是构建环境认的那几个变量」、「✗ 自证：--unit-environment 没把换了工具链的 RUSTUP_TOOLC」
  BROAD_MUTATION_BREAK=unit-environment-toolchain-ignored  --unit-environment 不比工具链（只看构建环境）
      红：「✗ 自证：--unit-environment 没把换了工具链的 RUSTUP_TOOLC」
  BROAD_MUTATION_BREAK=result-mac-unchecked  读判定库不核校验列
      红：「✗ 自证：手追加的没有校验列、校验列对不上的 caught 行被当成了判定」、「✗ 自证：密钥不在时 --summary 没判红、或替它建了一份」
  BROAD_MUTATION_BREAK=baseline-any-threads  取基线不看线程数
      红：「✗ 自证：换了 --threads 之后基线没在新线程数下另记、或登记行指的基线行对不上」
  BROAD_MUTATION_BREAK=escalation-queues-by-peak  升档那趟不带 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP
      红：「✗ 自证：升档那趟没按自己的上限排队、或改坏了源码的那几步没设撞总上限时先挨杀」
  BROAD_MUTATION_BREAK=mutation-steps-not-oom-first  改坏了源码的那几步不带 RUN_WITH_MEMORY_CAP_OOM_FIRST
      红：「✗ 自证：升档那趟没按自己的上限排队、或改坏了源码的那几步没设撞总上限时先挨杀」
  BROAD_MUTATION_BREAK=rocksdb-entry-unchecked  真跑起 cargo 之前不调 scripts/hosts/rocksdb-prebuilt.py entry（改前的样子）
      红：「✗ 自证：被判的树里 checker 档包依赖 rocksdb 而进程环境里没有预编库的取值，没在起 cargo 之前拒」
      （entry 的跳过办法：被判的树里没有依赖 rocksdb 的清单（自证的 SELFTEST_SOURCES 就是这样），或 PATH 上的 cargo 是带
      『# rocksdb-prebuilt-entry: selftest-fake-cargo』那一行的假 cargo；见 scripts/hosts/rocksdb-prebuilt.py 文件头『entry』）
  BROAD_MUTATION_BREAK=no-carry-without-arguments  不带选条参数时不读判定库（全部现判）
      红：「✗ 自证：不带参数、库里有可沿用的判定时没有沿用」、「✗ 自证：同一条变异内容没变、判定库里有可沿用的行、所在文件在某提交之后改过别处，没照样沿」、「✗ 自证：抓到它的那条测试改了之后那几条没现判」、「✗ 自证：改一条排除之后别的条没照样沿用」、「✗ 自证：只改 Cargo.toml 注释之后别的条没照样沿用」、
      「✗ 自证：加大限时参数与上限之后别的条没照样沿用」、「✗ 自证：拒算的条没有每趟现判、或别的条没沿用」、「✗ 自证：剪枝跑没有重跑超时、内存撞顶的条」、「✗ 自证：--stage-two-only 不对」、「✗ 自证：重起之后沿用得了的条或基线被重跑了」、「✗ 自证：--shard 0/2 跑的不是沿用不了的偶数序号」、
      「✗ 自证：第二段的定论被后来的 stage-one-only 行降级了」、「✗ 自证：认不出的退出码没处理对」、「✗ 自证：工具链没进每一把键」、「✗ 自证：两台各一片的结果没合对」
      另红：「✗ 复用失灵：…键相同却现判了」、「✗ 自证：reuse-sentinel：」、「✗ 自证：reuse-three-questions：」、「✗ 自证：同一条命令第二趟 GPU 那一趟没复用」、「✗ 自证：键对不上的 GPU 判定行没被拒、或拒了之后本机的 GPU 趟没把它照现判」、「✗ 自证：第二台的 GPU 判定行没随导入并进本机库、或并进来之后本机的 GPU 趟还是把同一批判了一遍」
  BROAD_MUTATION_BREAK=select-by-commit  恢复按提交选条：被判那棵树里相对 HEAD 改过的文件里的条一律现判
      红：「✗ 自证：同一条变异内容没变、判定库里有可沿用的行、所在文件在某提交之后改过别处，没照样沿」、「✗ 自证：抓到它的那条测试改了之后那几条没现判」
  BROAD_MUTATION_BREAK=judge-units-raw-text  判法单元指纹取两份脚本整份原文的 sha256（只改注释也变）
      红：「✗ 自证：只改驱动注释判法单元指纹变了」
      另红：「✗ 自证：改了 GPU 趟的函数，CPU 那一趟的判法单元指纹跟着变了」
  BROAD_MUTATION_BREAK=limits-in-key  --timeout-floor、--memory-cap、--cpu-multiplier 进每一把键
      红：「✗ 自证：加大限时参数与上限之后别的条没照样沿用」、「✗ 自证：加大 --timeout-floor、--memory-cap 再 --item」、「✗ 自证：两趟都超时、证据齐的没判成 caught-by-timeout」、「✗ 自证：两趟都撞顶、证据齐的没判成 caught-by-memory」、
      「✗ 自证：升档那趟跑完没照它往后的结局判」、「✗ 自证：证据不齐的升档判成了抓到」、「✗ 自证：--summary 没列升档判的抓到、或下一趟没按键沿用」
      另红：「✗ 自证：reuse-sentinel：」
  BROAD_MUTATION_BREAK=refused-carried  拒算的条与抓到它的测试认不出的条照沿用
      红：「✗ 自证：拒算的条没有每趟现判、或别的条没沿用」
  BROAD_MUTATION_BREAK=removed-arguments-accepted  已删的参数不另拒（落进 argparse 的用法错）
      红：「✗ 自证：已删的参数或旧库目录没拒」
  BROAD_MUTATION_BREAK=old-layout-accepted  旧库目录（有 tree.source / versions/）照用
      红：「✗ 自证：已删的参数或旧库目录没拒」
  BROAD_MUTATION_BREAK=read-only-volatile-refused  只读的用法（--summary 这一类）也拒落在开机会清空的目录里的库（改前的写法：样本拷进 /tmp 再判时判不了）
      红：「✗ 自证：库目录落在 /dev/shm 里时只读的 --summary 也拒了」
  BROAD_MUTATION_BREAK=list-cache-unchecked  清单按单元记进 lists.tsv 之后不复原核对
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
  BROAD_MUTATION_BREAK=baseline-alerts-beside-peaks  基线那几步不设 SINGLEFS_CORE_USAGE_ALERTS（改前的写法：告警跟着基线峰值表写到它旁边，叫不醒主 agent）
      红：「✗ 自证：基线那几步交给一步的 SINGLEFS_CORE_USAGE_ALERTS 不是 core_usage.py 的 alerts_path() 给的真记录」
  BROAD_MUTATION_BREAK=gpu-red-not-caught  GPU 那一趟红了不算抓到（合起来仍记没红）
      红：「✗ 自证：只有 GPU 红：」
  BROAD_MUTATION_BREAK=gpu-all-items  不看闭包：每一条 CPU 那一趟没红的都跑 GPU 那一趟、目标取全部（闭包不引 gpu 的、没有测试够得着的也跑）
      红：「✗ 自证：只有 CPU 红：」、「✗ 自证：两趟都没红：」、「✗ 自证：GPU 那一趟的命令不对」、「✗ 自证：同一条命令第二趟 GPU 那一趟没复用」、「✗ 自证：点名加 --rerun 没有重跑 GPU 那一趟并追加新行」
      另红：「✗ 自证：reuse-sentinel：」、「✗ 自证：reuse-three-questions：」、「✗ 自证：两台各跑一片的 GPU 导入格前提不成立」
  BROAD_MUTATION_BREAK=gpu-not-run-as-missed  没找到卡（NOT_RUN_NO_GPU_ADAPTER）记成没红
      红：「✗ 自证：没卡：」、「✗ 自证：有卡之后重跑同一条命令没把 GPU 那一趟补上」、「✗ 自证：中途没卡：」
  BROAD_MUTATION_BREAK=gpu-carry-ignored  GPU 那一趟沿用得了的行照样重跑
      红：「✗ 自证：同一条命令第二趟 GPU 那一趟没复用」、「✗ 自证：点名加 --rerun 没有重跑 GPU 那一趟并追加新行」、复用哨兵那一格（reuse-sentinel：GPU 趟沿用得了的被现判，复用失灵）
      另红：「✗ 自证：reuse-sentinel：」、「✗ 自证：reuse-three-questions：」、「✗ 自证：键对不上的 GPU 判定行没被拒、或拒了之后本机的 GPU 趟没把它照现判」、「✗ 自证：第二台的 GPU 判定行没随导入并进本机库、或并进来之后本机的 GPU 趟还是把同一批判了一遍」
  BROAD_MUTATION_BREAK=gpu-quiesce-dropped  GPU 那一趟的测试步不经 scripts/crash/gpu-quiesce-run.sh
      红：「✗ 自证：GPU 那一趟的命令不对」
  BROAD_MUTATION_BREAK=gpu-test-threads-dropped  GPU 那一趟的测试步不加 --test-threads 1
      红：「✗ 自证：GPU 那一趟的命令不对」
  BROAD_MUTATION_BREAK=gpu-mac-unchecked  读 GPU 判定库不核校验列
      红：「✗ 自证：手追加的没有校验列、校验列对不上的 GPU 判定行没判红」
  BROAD_MUTATION_BREAK=gpu-also-when-cpu-caught  CPU 那一趟抓到的条也跑 GPU 那一趟
      红：「✗ 自证：只有 CPU 红：」
      另红：「✗ 自证：reuse-three-questions：」
mutation-prune.py 文件头列的那几项（驱动在自证里把它们转给规则库）打开时这份 --selftest 不一定红，由 mutation-prune.py --selftest 判。
只供测试（--selftest 自己设；这个进程的父进程不是带 --selftest 的这份脚本、或库目录不在那一趟自证的工作目录里，设了任何一个都拒，退 2）：
  BROAD_MUTATION_STEP_DRIVER=1 每一步的真命令换成自证工作目录（BROAD_MUTATION_SELFTEST_WORK）里的 fake-step-driver
    （调法 fake-step-driver <步名> <序号> <树>，不经内存包装、不要前缀；文件名写死，起它的命令是 lib_function_units.py 认得的根外程序形态）、
  BROAD_MUTATION_STEP_VIA_WRAPPER=1 假的一步也经内存包装起（证「被 TERM 时连 scope 一起停」那一格用）、
  BROAD_MUTATION_LIST_DRIVER=<脚本> 换掉 cargo mutants --list（调法 <脚本> <包>，打 JSON）、BROAD_MUTATION_EXPORT_EVERYTHING=1 导出范围换成 --source 整棵树、
  BROAD_MUTATION_TOOLCHAIN=<文字> 换掉 rustc -Vv 的输出、BROAD_MUTATION_KILL_AT=<点> 走到那一点给自己发 KILL、
  BROAD_MUTATION_ALLOW_VOLATILE_HOME=1 许库目录落在 /tmp 下、BROAD_MUTATION_EXCLUSIONS=<文件> 换掉排除表、
  BROAD_MUTATION_PROGRESS_INTERVAL=<秒> 换掉进度间隔、BROAD_MUTATION_TOOL_VERSIONS=<文字> 换掉工具版本、
  BROAD_MUTATION_JUDGING_KEY=<文字> 换掉本仓这一份的判法单元指纹（自证每一趟不现算它，判法单元指纹那几格在拷贝上现算）、
  BROAD_MUTATION_GPU_JUDGING_KEY=<文字> 换掉本仓这一份的 GPU 趟判法单元指纹（同上，GPU 趟指纹那一格在拷贝上现算）、
  BROAD_MUTATION_RESULT_KEY=<文件> 判定行的密钥换成这一份（自证起的每个子进程都带：自证不读写真的密钥）、
  BROAD_MUTATION_RUSTC=1 工具链换成起自证工作目录里 fake-rustc-vv 的 -Vv（--unit-environment 比工具链那一格用；文件名同样写死）。

gate-similar: checker-tier-crates-mutation-replay.sh 它按 crates/mutations.tsv 逐条改坏、点名的测试必须红，行是人写的、判的是「这条测试抓不抓得到」；这里的清单由 cargo-mutants 生成、判每条变异有没有任何测试抓到，导出范围取它登记的那一份
gate-similar: mutate.sh 它跑一张手写变异表对一个实验二进制，每条 cargo test 经内存包装；这里同样逐步经内存包装，但清单是生成的、判定按单元键落在库目录里沿用与续接
gate-similar: crash-amplification.sh 它是崩溃放量的分项驱动，--list-items / --item 与「重跑同一条命令就是续跑」的形态照它；判的对象不同（崩溃点状态对变异）
gate-similar: gpu-quiesce-run.sh 它只管清场、回读、跑（不复原）；这里 GPU 那一趟的测试步整条经它起，清场的写法、多机配置与回读都不在这里另抄
gate-similar: checker-tier-crash-point-replay.sh 它的 feature-gated-tests 格对崩溃枚举用例的状态块跑同一条带特性的快档命令、判的是「状态块有没有红」；这里同一条命令判的是「CPU 那一趟没红的变异有没有被带特性的测试抓到」，特性与文件的现读写法照它的
gate-similar: mutation-shard-run.sh 它把 crates/mutations.tsv 的行分给两台、第二台的判定库记录经 crates-mutation-rows.py import 再导入；这里两台各起一趟 --shard k/2、判定库是一张只追加的 TSV，导入（--import-results）逐行核段数、结局与键，两边的记录形态不同，导入不共用
"""
import argparse
import datetime
import difflib
import fcntl
import hashlib
import hmac
import importlib.util
import json
import math
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
MEMORY_CAP_RUNNER = os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "run-with-memory-cap.sh")
THREAD_CAP_RUNNER = os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "capped.sh")
CORE_USAGE_LIBRARY = os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "core_usage.py")   # 基线那几步的告警记录取它的 alerts_path()（文件头「内存」）
DEFAULT_HOME = "~/.local/share/singlefs/broad-mutation-units"
OLD_LAYOUT_MARKERS = ("tree.source", "versions")      # 「一版一个目录」那时的库目录里才有的：见到就拒（旧库整个弃用，不迁移、不补签）
DEFAULT_MEMORY_CAP = "24G"        # 2026-09-30 派发给的上限；各步的实测峰值由内存包装记在 ~/.local/share/singlefs/memory-peaks.tsv（键 broad-mutation <步名>）
DEFAULT_TIMEOUT_FLOOR_SECONDS = 900
DEFAULT_TIMEOUT_MULTIPLIER = 5
DEFAULT_LOCK_WAIT_SECONDS = 120
DEFAULT_STOP_GRACE_SECONDS = 10
DEFAULT_SUSPECT_STREAK = 8           # 依据见文件头 --suspect-streak
DEFAULT_SUSPECT_TEST_SECONDS = 0.2
PROGRESS_INTERVAL_SECONDS = 300      # 一步跑着时隔多久报一行进度（文件头「输出」）
LOCK_POLL_SECONDS = 2
MUTATED_PACKAGES = ("singlefs-core", "singlefs-checker")    # 清单的次序：先 core 后 checker，序号从 1 起
HARNESS_TIER_PACKAGES = ("singlefs-harness", "singlefs-checker", "singlefs-core", "singlefs-format")
CHECKER_TIER_PACKAGE = "singlefs-checker-tier"
SLOW_STEP_NAME = "two-test-slow-harness"       # 第二段 harness 慢档用例那一步；旧名 OLD_SLOW_STEP_NAME 是没排除大档的那一版命令，基线表里的旧行因此不认
OLD_SLOW_STEP_NAME = "two-test-ignored"
EXCLUSIONS_FILE = os.path.join(SCRIPT_DIRECTORY, "broad-mutation-ignored-exclusions.tsv")
TIMED_CASE_REASON_PREFIX = "harness 慢档用例："
HEAVY_PREFIX_VARIABLE = "SINGLEFS_HEAVY_TESTS"
HEAVY_PREFIX_VALUES = ("commit", "user-request")
FULL_VARIABLE = "SINGLEFS_GATE_FULL"
VOLATILE_PREFIXES = ("/tmp", "/run", "/dev/shm", "/var/run")
VOLATILE_FILESYSTEMS = ("tmpfs", "ramfs")
IGNORED_DIRECTORY_NAMES = ("target", "mutants.out", "mutants.out.old")
# gate-overlap:copy-kept lib_verdict_library.py 广谱变异的判定库是只追加的 TSV（一行一条判定、按单元键取、每行带密钥校验列、两台导入按行核），判定库共用部分按项名与两列键存 JSON、一项一个文件；形态不同，存放、按键读、原子写都不抽
VERDICT_COLUMNS = ("key", "sequence", "package", "name", "outcome", "stage", "caught_by", "test_key", "coverage_key", "build_seconds", "test_seconds",
                   "judged_on", "method", "locator", "refused", "note", "mac")
RESULT_SIGNED_COLUMNS = len(VERDICT_COLUMNS) - 1    # mac 一段签的是它前面那十六段（文件头「判定行的校验列」）
DEFAULT_RESULT_KEY = "singlefs/broad-mutation-result.key"     # 相对 ${XDG_CONFIG_HOME:-~/.config}
RESULT_KEY_HEX_LENGTH = 64
BASELINE_COLUMNS = ("baseline_key", "step", "outcome", "seconds", "finished_at", "threads", "peak_bytes")
# 按条记基线（文件头「基线」）：一行一份在没改过的树上跑绿的记录；kind 是 test（一条测试）、rest（一个二进制里宏调用展开的那几条）、binary（checker 档一个二进制，
# 有拒算的是去掉那几条的那一份），lines 是 runner 记的那几行（JSON：[[文件名, 用例, CPU 秒, 墙钟秒]…]），binary 只当索引（rest 另按它找）
BASELINE_ENTRY_COLUMNS = ("record_key", "step", "threads", "kind", "binary", "lines", "peak_bytes", "finished_at")
COVERAGE_COLUMNS = ("record_key", "stage", "binary", "test", "covered", "profiles", "judged_on")
LIST_COLUMNS = ("unit_key", "mutations")
# 判法单元指纹（文件头「键」）从这几个函数收 python 单元闭包：选条、沿用、结论分类、升档（连同它们点到的常量与被调的函数）
# 方法写 <类>.<方法>（lib_function_units.py 的 python 限定名）：Judgement、UnitTree 是调用方传进来的对象，只从函数起收不到它们的方法，另点名
JUDGING_ENTRIES = (("scripts/mutation/broad-mutation.py", "select_items"), ("scripts/mutation/broad-mutation.py", "verdict_state"),
                   ("scripts/mutation/broad-mutation.py", "Judgement.state"), ("scripts/mutation/broad-mutation.py", "Judgement.row_holds"),
                   ("scripts/mutation/broad-mutation.py", "Judgement.selection"), ("scripts/mutation/broad-mutation.py", "process_item"),
                   ("scripts/mutation/broad-mutation.py", "Runner.escalate_step"), ("scripts/mutation/broad-mutation.py", "escalation_evidence_problems"),
                   ("scripts/mutation/mutation-prune.py", "carry_decision"), ("scripts/mutation/mutation-prune.py", "mutation_key"),
                   ("scripts/mutation/mutation-prune.py", "UnitTree.closure_key"), ("scripts/mutation/mutation-prune.py", "catching_test_key"),
                   ("scripts/mutation/mutation-prune.py", "coverers_key"), ("scripts/mutation/mutation-prune.py", "global_lines"),
                   ("scripts/mutation/mutation-prune.py", "build_environment_lines"))
# 收判法单元闭包时扫哪几份（驱动、规则库、libtest 结果行的解析）；它们按字面路径载入的 lib_function_units.py 照样收进来
JUDGING_SCAN_FILES = ("scripts/mutation/broad-mutation.py", "scripts/mutation/mutation-prune.py", "scripts/gate/lib_libtest_result_lines.py")
RUNNER_TEST_STEPS = ("one-test", "two-test-slow-harness", "two-test-checker-tier")   # 经 runner 按单条（checker 档按二进制）限 CPU 时间的测试步
REFUSED_BASELINE_DIRECTORY = "refused"   # baselines/ 下拒算的基线放逐条时间表与峰值的那一份（文件头「基线」）
EXIT_GATE_RED = 8
# 升档（文件头「升档」）：四个常量是判法的一部分（escalate_step 点到它们），改了判法单元指纹就变，库里的判定一条都不沿用；提交门禁复核登记行时按此刻的常量判倍数
ESCALATION_TIME_FACTOR = 4                    # 超时那一步升档：限时乘几倍
ESCALATION_TIME_BASELINE_MULTIPLE = 20        # 升档那趟的限时不低于基线秒数的几倍（K，依据见文件头「升档」）
ESCALATION_MEMORY_FACTOR = 1.5                # 撞顶那一步升档：内存上限乘几倍（本机 slice 总上限约 40 GiB，默认上限 24G 乘 1.5 放得下）
ESCALATION_MEMORY_BASELINE_MULTIPLE = 2       # 升档那趟的上限不低于基线峰值的几倍（K，依据见文件头「升档」）
ESCALATED_TIMEOUT_LABEL = "caught-by-timeout"
ESCALATED_MEMORY_LABEL = "caught-by-memory"
ESCALATED_LABELS = (ESCALATED_TIMEOUT_LABEL, ESCALATED_MEMORY_LABEL)
ESCALATION_REGISTRY = "escalated-verdicts.tsv"      # 库目录里的登记表，只追加
ESCALATION_DIRECTORY = "escalation"                 # 库目录里放两趟日志的子目录
ESCALATION_COLUMNS = ("key", "sequence", "name", "label", "step", "baseline_key", "baseline", "baseline_row", "threads", "first_limit", "escalated_limit",
                      "required_multiple", "first_log", "first_log_sha256", "escalated_log", "escalated_log_sha256", "judged_on")
ESCALATED_QUEUE_REASON = "广谱变异升档重跑的那一趟：按它自己的上限排队，放不下就等或退 252"   # 设进 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP
MUTATION_STEP_OOM_REASON = "广谱变异改坏了源码的那一步：撞 slice 总上限时先杀它"           # 设进 RUN_WITH_MEMORY_CAP_OOM_FIRST
ESCALATION_EVIDENCE_PREFIX = "BROAD_MUTATION_EVIDENCE"   # 驱动追加在两趟日志末尾的那一行
BASELINE_PEAK_KEY_PREFIX = "broad-mutation baseline "    # 基线那几步在峰值表里的键：前缀加步名
MEMORY_CAP_FORM = re.compile(r"^([1-9][0-9]*)([KMGT])$")  # run-with-memory-cap.sh 认的上限写法（1024 进制，单位必写）
MEMORY_CAP_UNITS = {"K": 1024, "M": 1024 ** 2, "G": 1024 ** 3, "T": 1024 ** 4}
# scripts/hosts/detached-run.sh 本来就带进单元的变量：--unit-environment 不再打它们
UNIT_CARRIED_VARIABLES = ("SINGLEFS_HEAVY_TESTS", "PATH", "HOME", "CARGO_HOME", "RUSTUP_HOME")
# 弄坏开关 unit-environment-fixed-list 下认的那几个（构建环境改取生效值之前驱动自己写的那一份）
FIXED_LIST_VARIABLES = ("RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_BUILD_RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS")
IMPORT_COLUMNS = ("imported_at", "source", "source_verdicts_sha256", "rows_read", "rows_imported", "rows_already_present", "conflicting_keys")
DECISIVE_OUTCOMES = ("caught", "missed", "unviable")          # 这条变异自己的结局
ENVIRONMENTAL_OUTCOMES = ("timeout", "memory-cap")            # 不是定论：每趟照选中现判；不盖掉前面的 caught / missed / unviable
FINAL_OUTCOMES = (*DECISIVE_OUTCOMES, *ENVIRONMENTAL_OUTCOMES)
STAGE_ONE_ONLY = "stage-one-only"
DRIVER_ERROR = "driver-error"
NOT_RUN = "none"
ROW_OUTCOMES = (*FINAL_OUTCOMES, STAGE_ONE_ONLY, DRIVER_ERROR)
SUMMARY_COLUMNS = (*FINAL_OUTCOMES, STAGE_ONE_ONLY, DRIVER_ERROR, NOT_RUN)
SUMMARY_KEYS = {"caught": "caught", "missed": "missed", "unviable": "unviable", "timeout": "timeout", "memory-cap": "memory_cap",
                STAGE_ONE_ONLY: "stage_one_only", DRIVER_ERROR: "driver_error", NOT_RUN: "not_run"}
OUTCOME_TEXT = {"caught": "抓到", "missed": "没红", "unviable": "无效", "timeout": "超时", "memory-cap": "内存撞顶",
                STAGE_ONE_ONLY: "只过了第一段", DRIVER_ERROR: "驱动错", NOT_RUN: "沿用不了、还没现判"}
MEMORY_CAP_HIT_EXIT = 250            # run-with-memory-cap.sh 的退出码，含义见它的文件头「退出码」一段
MEMORY_CAP_UNAVAILABLE_EXIT = 251
MEMORY_ADMISSION_REFUSED_EXIT = 252
TIME_LIMIT_HIT_EXIT = 253
SLICE_TOTAL_HIT_EXIT = 254
GPU_MEMORY_HIT_EXIT = 245            # 显存不够：命令退了非 0、起止之间内核报了 NV_ERR_NO_MEMORY，结果不算数
WRAPPER_INFRASTRUCTURE_EXITS = {MEMORY_CAP_UNAVAILABLE_EXIT: "带上限的 scope 起不来", MEMORY_ADMISSION_REFUSED_EXIT: "内存排不上",
                                SLICE_TOTAL_HIT_EXIT: "被 slice 的总上限挤掉",
                                GPU_MEMORY_HIT_EXIT: "显存不够：起止之间内核报了 NV_ERR_NO_MEMORY，输出里的失败是被它弄红的"}
NEXTEST_TEST_FAILED_EXIT = 100       # cargo nextest run：有测试没过
CARGO_FAILED_EXIT = 101              # cargo：编不过、或 cargo test 有测试没过
EXIT_INCONSISTENT, EXIT_USAGE, EXIT_INFRASTRUCTURE, EXIT_LOCKED, EXIT_DIRTY_TREE, EXIT_BASELINE_RED, EXIT_STOPPED = 1, 2, 3, 4, 5, 6, 143
EXIT_RUN_DIRECTORY_KEPT = 9          # 这一趟别的都成了，运行目录没删掉（有进程开着里面的文件，或回读还在）
NEXTEST_FAILURE_LINE = re.compile(r"^\s*(?:FAIL|TIMEOUT|ABORT|LEAK-FAIL|SIG[A-Z0-9]+)\s+\[[^\]]*\]\s+(?:\([^)]*\)\s+)?(\S.*?)\s*$")
# libtest 那几步（文档测试、checker 档快档）的日志里认出二进制的两种行：cargo test 起每个测试二进制前打的 Running、文档测试打的 Doc-tests
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
# 内存包装给一步开的 scope：名字里第一段数字是包装自己的进程号（scripts/hosts/run-with-memory-cap.sh 的 unit="singlefs-memory-cap-$$-…"）
OWN_SCOPE_UNIT = re.compile(r"/(singlefs-memory-cap-(\d+)-\d+\.scope)\s*$")
SELFTEST_WORK_VARIABLE = "BROAD_MUTATION_SELFTEST_WORK"
SELFTEST_CELLS_VARIABLE = "BROAD_MUTATION_SELFTEST_CELLS"     # 开发时分段跑自证：逗号隔开的几组格名（SELFTEST_CELL_NAMES）；不设就是全部，门禁与提交前跑的都不设
SELFTEST_STEP_DRIVER_NAME = "fake-step-driver"   # 自证写在自证工作目录里的假一步（文件头「只供测试」BROAD_MUTATION_STEP_DRIVER）
SELFTEST_RUSTC_NAME = "fake-rustc-vv"            # 自证写在自证工作目录里的假 rustc（BROAD_MUTATION_RUSTC）
SELFTEST_TRANSCRIBED_FAILURE_MARK = "〔转述〕"   # 失败行转述的驱动输出里，✗ 换成它（文件头开关表前那一段「转述」）
SELFTEST_DRIVER_FAILURE_LINES_SHOWN = 8          # 失败行末尾整行接上的、这一格起的驱动报的 ✗ 行：去掉数字之后相同的并成一种，至多这么多种，多的只报种数
TEST_ONLY_VARIABLES = ("BROAD_MUTATION_STEP_DRIVER", "BROAD_MUTATION_STEP_VIA_WRAPPER", "BROAD_MUTATION_LIST_DRIVER", "BROAD_MUTATION_EXPORT_EVERYTHING",
                       "BROAD_MUTATION_TOOLCHAIN", "BROAD_MUTATION_KILL_AT", "BROAD_MUTATION_ALLOW_VOLATILE_HOME", "BROAD_MUTATION_EXCLUSIONS", "BROAD_MUTATION_BREAK",
                       "BROAD_MUTATION_PROGRESS_INTERVAL", "BROAD_MUTATION_TOOL_VERSIONS", "BROAD_MUTATION_JUDGING_KEY", "BROAD_MUTATION_GPU_JUDGING_KEY",
                       "BROAD_MUTATION_RESULT_KEY", "BROAD_MUTATION_RUSTC")
SELFTEST_ANCESTOR_DEPTH = 6
REMOVED_ARGUMENTS = {"--sync-tree": "「一版」不再是容器：每一趟开跑自己把 --source 那棵树换进专用树",
                     "--commit": "不按提交导出：被判的是 --source 那棵工作树此刻的内容",
                     "--changed-since": "不按提交选条：每条变异按单元键过沿用判据，沿用不了的才现判",
                     "--milestone-full": "里程碑末全量改叫 --full（只写不读，翻面的逐条打 FLIPPED）"}


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


def break_is(name):
    return test_mode() and raw_break(name)


def path_is_under(path, directory):
    return path == directory or path.startswith(directory.rstrip("/") + "/")


def heavy_prefix_text():
    """出路里起重活那条命令的前缀：照这一趟自己带的（提交流程里是 =commit、用户要求时 =user-request）；这一趟没带（--summary、--list-items 不要前缀）就两种都列。"""
    given = os.environ.get(HEAVY_PREFIX_VARIABLE)
    if given in HEAVY_PREFIX_VALUES:
        return f"{HEAVY_PREFIX_VARIABLE}={given}"
    return f"{HEAVY_PREFIX_VARIABLE}=commit（用户要求时 ={HEAVY_PREFIX_VALUES[1]}）"


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


# ── 只追加的表：判定库、基线表、覆盖记录、清单表、导入记录 ────────────────────────────────────────────────

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
        if break_is("partial-row-kept"):
            lines.append("")
        else:
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


def write_fully(descriptor, data):
    """os.write 可以只写进去一部分（盘快满时）：写到写完为止，写不进去的让它抛出来。"""
    view = memoryview(data)
    while view:
        written = os.write(descriptor, view)
        view = view[written:]


def drop_partial_tail(path):
    """表的末行没写完（上一次写到一半断了）：截到最后一个换行，截掉的文字另存一份并说一句。交回表是不是空了（连表头都没写完）。"""
    with open(path, "rb") as handle:
        data = handle.read()
    if not data or data.endswith(b"\n"):
        return not data
    keep = data.rfind(b"\n") + 1
    dropped = data[keep:]
    with open(path + ".dropped-partial-rows", "ab") as handle:
        handle.write(dropped + b"\n")
    os.truncate(path, keep)
    note(f"  · {path} 的末行没写完（{len(dropped)} 字节，上一次写到一半断了）：已截掉、不算数，那一条会重跑；截掉的文字留在 {path}.dropped-partial-rows")
    return keep == 0


def append_rows(path, header_columns, rows):
    """往只追加的表里加几行并 fsync 一次；表还不在就先写表头（# 起头）。末行没写完的先截掉（drop_partial_tail）。"""
    data = b"".join(("\t".join(clean_field(field) for field in fields) + "\n").encode() for fields in rows)
    is_new = not os.path.exists(path)
    if not is_new:
        if break_is("partial-row-kept"):
            with open(path, "rb") as handle:
                existing = handle.read()
            if existing and not existing.endswith(b"\n"):
                data = b"\n" + data
        else:
            is_new = drop_partial_tail(path)
    if is_new:
        data = ("# " + "\t".join(header_columns) + "\n").encode() + data
    descriptor = os.open(path, os.O_WRONLY | os.O_APPEND | os.O_CREAT, 0o644)
    try:
        if test_variable("BROAD_MUTATION_KILL_AT") == "row-half-written" and os.path.basename(path) == "verdicts.tsv":
            write_fully(descriptor, data[:-5])     # 行差最后几个字节没写完（表头是完整的）就被杀
            kill_point("row-half-written")
        write_fully(descriptor, data)
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    if is_new:
        fsync_directory(os.path.dirname(path))


def append_row(path, header_columns, fields):
    append_rows(path, header_columns, [fields])


def read_baseline_rows(path):
    """baselines.tsv 里成形的行：[(文件里的行号, {列名: 值})]。行号从 1 起、表头那一行也算（升档登记行记的 baseline_row 就是它，门禁按它找那一行）；
    末行没写完（没有换行结尾）的不算。"""
    found = []
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            lines = handle.read().split("\n")
    except FileNotFoundError:
        return found
    for number, line in enumerate(lines[:-1], start=1):
        fields = line.split("\t")
        if line and not line.startswith("#") and len(fields) == len(BASELINE_COLUMNS):
            found.append((number, dict(zip(BASELINE_COLUMNS, fields))))
    return found


# ── 判定行的校验列（文件头「判定行的校验列」） ───────────────────────────────────────────────────────────────

def result_key_path():
    """判定行的密钥放哪：${XDG_CONFIG_HOME:-~/.config}/singlefs/broad-mutation-result.key（库目录之外、仓之外）；自证里是 BROAD_MUTATION_RESULT_KEY 指的那一份。"""
    fake = test_variable("BROAD_MUTATION_RESULT_KEY")
    if fake:
        return os.path.realpath(fake)
    if test_mode():
        refuse("自证起的子进程没设 BROAD_MUTATION_RESULT_KEY：自证不许读写真的密钥", "看 Selftest 的环境：每一趟都带 BROAD_MUTATION_RESULT_KEY=<自证工作目录里的一份>")
    base = os.environ.get("XDG_CONFIG_HOME") or os.path.join(os.path.expanduser("~"), ".config")
    return os.path.realpath(os.path.join(base, DEFAULT_RESULT_KEY))


def read_result_key(path):
    """读密钥：交 (字节, "")；用不了交 (None, 为什么)。要只有自己能读写（属主是自己、属组与别人没有任何位）、内容是 64 位小写十六进制。"""
    try:
        status = os.stat(path)
    except FileNotFoundError:
        return None, f"密钥 {path} 不在"
    except OSError as error:
        return None, f"读不了密钥 {path}：{error}"
    if status.st_uid != os.getuid() or status.st_mode & 0o077:
        return None, f"密钥 {path} 的权限是 {status.st_mode & 0o777:o}、属主 {status.st_uid}：要属主是自己、权限 600"
    with open(path, encoding="ascii", errors="replace") as handle:
        text = handle.read().strip()
    if len(text) != RESULT_KEY_HEX_LENGTH or not re.fullmatch(r"[0-9a-f]+", text):
        return None, f"密钥 {path} 的内容不是 {RESULT_KEY_HEX_LENGTH} 位小写十六进制"
    return bytes.fromhex(text), ""


def create_result_key(path):
    """没有密钥时建一份：目录 700、文件 600（排他新建）、32 字节随机数写成十六进制。别的一趟刚建了就照用它的，交 False。"""
    directory = os.path.dirname(path)
    os.makedirs(directory, mode=0o700, exist_ok=True)
    try:
        descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    except FileExistsError:
        return False
    try:
        write_fully(descriptor, (secrets.token_hex(RESULT_KEY_HEX_LENGTH // 2) + "\n").encode())
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    fsync_directory(directory)
    return True


def result_mac(key, fields):
    """一行判定的校验列：HMAC-SHA256(密钥, 前十六段照写进表里的样子用制表符接起来、UTF-8)，小写十六进制。"""
    text = "\t".join(clean_field(field) for field in fields[:RESULT_SIGNED_COLUMNS])
    return hmac.new(key, text.encode("utf-8", "surrogateescape"), hashlib.sha256).hexdigest()


def signed_result_row(key, fields):
    """十六段的判定行加上校验列：交十七段（前十六段已按写表的样子洗过）。"""
    cleaned = [clean_field(field) for field in fields[:RESULT_SIGNED_COLUMNS]]
    return cleaned + [result_mac(key, cleaned)]


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
                refuse(f"库目录 {home} 的锁被占着（等了 {wait_seconds} 秒）：另一趟驱动在跑，或上一趟被 KILL 之后它起的那一步（内存包装与它开的 scope）还没退",
                       "`ps -o pid,ppid,etimes,args -u \"$(id -u)\"` 找出还在跑的那一趟（run-with-memory-cap.sh、cargo、cargo-nextest、测试二进制），"
                       "等它退出再重跑同一条命令；确是上一趟留下的，逐个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>` 停掉再跑；"
                       "要多等就加 --lock-wait <秒>", EXIT_LOCKED)
            if not announced:
                note(f"  · 库目录的锁被占着，最多等 {wait_seconds} 秒（上一趟留下的进程退出就放开）")
                announced = True
            time.sleep(LOCK_POLL_SECONDS)


def export_paths():
    """导出哪些路径：门禁 checker-tier-crates-mutation-replay 登记的拷贝范围（去掉 research/ 下的：那是那道门禁自己的脚本）加 .cargo。
    拷贝范围已登记了 .cargo 就不再加。"""
    specification = importlib.util.spec_from_file_location("crates_mutation_rows_for_broad_mutation",
                                                           os.path.join(SCRIPT_DIRECTORY, "crates-mutation-rows.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    try:
        copied = module.copied_paths(REPOSITORY)
    except Exception as error:  # 登记表读不了、没有那一行
        refuse(f"读不出导出范围（scripts/mutation/crates-mutation-rows.py 的 copied_paths）：{error}",
               "看 .claude/gate.d/stage-inputs.tsv 里 checker-tier-crates-mutation-replay.sh 那一行还在不在、成不成形")
    paths = [path for path in copied if not path.startswith("research/")]
    if ".cargo" not in paths:
        paths.append(".cargo")
    if "crates" not in paths:
        refuse(f"导出范围里没有 crates（读到的是 {copied}）", "看 .claude/gate.d/stage-inputs.tsv 里 checker-tier-crates-mutation-replay.sh 那一行登记的路径")
    return paths


def toolchain_text(directory, environment=None):
    """这一刻的工具链：在 directory 里跑 rustc -Vv 的输出（rustup 按目录与 RUSTUP_TOOLCHAIN 这类变量挑工具链）；environment 给了就在它下面跑。
    自证里 BROAD_MUTATION_RUSTC=1 就跑自证工作目录里的假 rustc（--unit-environment 比工具链那一格用）。"""
    fake = test_variable("BROAD_MUTATION_TOOLCHAIN")
    if fake is not None:
        return fake.strip() + "\n"
    fake_rustc = test_variable("BROAD_MUTATION_RUSTC") == "1"
    if test_mode() and not fake_rustc:
        return "selftest-toolchain\n"
    child_environment = None if environment is None else dict(environment)
    try:
        if fake_rustc:
            found = subprocess.run([os.path.join(test_variable(SELFTEST_WORK_VARIABLE) or "", SELFTEST_RUSTC_NAME), "-Vv"], capture_output=True, text=True,
                                   cwd=directory, stdin=subprocess.DEVNULL, env=child_environment)
        else:
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
    """一条各行的结局（按文件里的次序）→ ItemState。不取最后一行：
    caught / missed / unviable 取其中最后一行；没有它们才取 timeout / memory-cap 的最后一行；都没有而有 stage-one-only 的是它；只有 driver-error 的是它。"""
    passed_stage_one = STAGE_ONE_ONLY in outcomes
    if break_is("state-is-last-row"):
        return ItemState(outcomes[-1] if outcomes else NOT_RUN, passed_stage_one)
    decisive = [outcome for outcome in outcomes if outcome in DECISIVE_OUTCOMES]
    environmental = [outcome for outcome in outcomes if outcome in ENVIRONMENTAL_OUTCOMES]
    if decisive:
        return ItemState(decisive[-1], passed_stage_one)
    if environmental:
        return ItemState(environmental[-1], passed_stage_one)
    if passed_stage_one:
        return ItemState(STAGE_ONE_ONLY, True)
    if DRIVER_ERROR in outcomes:
        return ItemState(DRIVER_ERROR, False)
    return ItemState(NOT_RUN, False)


def today_text():
    """判定日期：只写日期（东京时区），不写时刻。"""
    return datetime.datetime.now(zoneinfo.ZoneInfo("Asia/Tokyo")).strftime("%Y-%m-%d")


# ── 带校验列的只追加表（判定库） ─────────────────────────────────────────────────────────────────────────────

class SignedTableRead(NamedTuple):
    rows: list              # 校验对得上的行（{列名: 值}，不含校验列），按文件里的次序
    forged: list            # [(文件里的行号, 键的前 16 位, 为什么不算)]：段数不对、校验列对不上、没有密钥校验不了
    malformed_lines: list   # 写完了而段数既不是全列也不是少一段的行的行号
    partial_tail: str       # 末行没写完时它的文字

    def forged_text(self):
        number, key, why = self.forged[0]
        return f"{len(self.forged)} 行不算（例：文件第 {number} 行、键 {key}，{why}）"


def read_signed_table(path, columns, key, key_problem=""):
    """读判定库：全列且校验列对得上的行才算数；少一段的（没有校验列）、校验列对不上的、没有密钥时的每一行都不算、列进 forged。
    弄坏开关 result-mac-unchecked 下两种都算，不核校验列。"""
    rows, forged, malformed_lines, partial_tail = [], [], [], ""
    if not os.path.exists(path):
        return SignedTableRead(rows, forged, malformed_lines, partial_tail)
    with open(path, "rb") as handle:
        lines = handle.read().decode("utf-8", errors="surrogateescape").split("\n")
    if lines[-1] != "":
        if break_is("partial-row-kept"):
            lines.append("")
        else:
            partial_tail = lines[-1]
    unchecked = break_is("result-mac-unchecked")
    signed = len(columns) - 1
    for number, line in enumerate(lines[:-1], start=1):
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) == signed:
            if unchecked:
                rows.append(dict(zip(columns, fields)))
            else:
                forged.append((number, fields[0][:16], "没有校验列（少一段）"))
            continue
        if len(fields) != len(columns):
            malformed_lines.append(number)
            continue
        if not unchecked:
            if key is None:
                forged.append((number, fields[0][:16], key_problem or "没有密钥，校验不了"))
                continue
            if not hmac.compare_digest(result_mac(key, fields), fields[-1]):
                forged.append((number, fields[0][:16], "校验列对不上"))
                continue
        rows.append(dict(zip(columns, fields[:signed])))
    return SignedTableRead(rows, forged, malformed_lines, partial_tail)


# ── 库目录 ─────────────────────────────────────────────────────────────────────────────────────────────────

class Library:
    """库目录里的各个位置（文件头「库目录」）：一份，固定一处，没有版本目录。"""

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
        self.verdicts_path = os.path.join(home, "verdicts.tsv")
        self.lists_path = os.path.join(home, "lists.tsv")
        self.baselines_path = os.path.join(home, "baselines.tsv")
        self.baseline_entries_path = os.path.join(home, "baseline-entries.tsv")
        self.coverage_path = os.path.join(home, "coverage.tsv")
        self.registry_path = os.path.join(home, ESCALATION_REGISTRY)
        self.imports_path = os.path.join(home, "imports.tsv")
        self.logs = os.path.join(home, "logs")
        self.manifest = None
        self.lock_descriptor = None
        self.deferred_rows = []
        self.result_key_cache = None
        self.result_key_override = None

    def result_key(self, create=False):
        """判定行的密钥（文件头「判定行的校验列」）：交 (字节, "")；没有或用不了交 (None, 为什么)。create（要往判定库里写行的那几种用法）时没有就建，
        建了还用不了拒（退 2）。密钥落在库目录里拒：库目录里的东西手写得了，密钥就防不住手写的行。只读的用法给了 --result-key 就读那一份。"""
        if self.result_key_cache is not None:
            return self.result_key_cache, ""
        path = os.path.realpath(self.result_key_override) if self.result_key_override else result_key_path()
        if path_is_under(path, os.path.realpath(self.home)):
            refuse(f"判定行的密钥 {path} 落在库目录 {self.home} 里：手写得了判定行的人也读得到它，校验列就防不住手写的行",
                   f"密钥放库目录之外（默认 ${{XDG_CONFIG_HOME:-~/.config}}/{DEFAULT_RESULT_KEY}）")
        key, problem = read_result_key(path)
        if key is None and create and not os.path.exists(path):
            if create_result_key(path):
                say(f"  · 判定行的密钥还没有：建了 {path}（权限 600）；之后每一行判定都带它算的校验列，门禁逐行核")
            key, problem = read_result_key(path)
        if key is None and create:
            refuse(f"判定行的密钥用不了：{problem}",
                   f"照那一句修（chmod 600 {path}、内容是 64 位小写十六进制）；密钥丢了或换了，旧的判定行就都校验不过：另起一个空库目录（--home），照提交流程第 2b 步重判")
        self.result_key_cache = key
        return key, problem

    def read_verdicts(self):
        key, key_problem = self.result_key()
        table = read_signed_table(self.verdicts_path, VERDICT_COLUMNS, key, key_problem)
        deferred = [dict(zip(VERDICT_COLUMNS, [clean_field(field) for field in row])) for row in self.deferred_rows]
        return SignedTableRead(table.rows + deferred, table.forged, table.malformed_lines, table.partial_tail)

    def record_verdict(self, fields):
        """落一行判定：十六段加上校验列（signed_result_row）。"""
        if break_is("no-row-flush"):
            self.deferred_rows.append(fields)
            return
        append_row(self.verdicts_path, VERDICT_COLUMNS, signed_result_row(self.result_key(create=True)[0], fields))

    def record_verdicts(self, rows):
        """一次落几行（导入的行），各带校验列，fsync 一次。"""
        key = self.result_key(create=True)[0]
        append_rows(self.verdicts_path, VERDICT_COLUMNS, [signed_result_row(key, fields) for fields in rows])

    def flush_deferred_rows(self):
        if self.deferred_rows:
            self.record_verdicts(self.deferred_rows)
        self.deferred_rows = []

    def composition_path(self, key):
        return os.path.join(self.home, "compositions", key[:2], f"{key}.txt")

    def write_composition(self, key, lines):
        """一把键的组成清单（文件头「库目录」compositions/）：同一把键只写一次。"""
        if not key:
            return
        path = self.composition_path(key)
        if os.path.exists(path):
            return
        os.makedirs(os.path.dirname(path), exist_ok=True)
        write_atomically(path, "".join(f"{line}\n" for line in lines).encode("utf-8", "surrogateescape"))

    def read_composition(self, key):
        try:
            with open(self.composition_path(key), encoding="utf-8", errors="surrogateescape") as handle:
                return handle.read().splitlines()
        except (OSError, TypeError):
            return None

    def read_list_cache(self):
        """清单按单元存的那张表：{清单的单元键: [这个单元里的变异]}；同一个键取最后一行。"""
        cache = {}
        for unit_key, mutations in read_table(self.lists_path, len(LIST_COLUMNS)).rows:
            try:
                cache[unit_key] = json.loads(mutations)
            except ValueError:
                continue
        return cache

    def baseline_directory(self, baseline_key):
        return os.path.join(self.home, "baselines", baseline_key[:16])

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
        有就不删、逐个报进程（scripts/hosts/run_build_directory.py）。交删成了没有；没建过（build-directory-in-home、--import-results）交 True。
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


def resolve_home(given, read_only=False):
    """库目录：先核只供自证的变量与旧库；要往库里写的用法另拒落在开机会清空的目录里的库。只读的用法（--list-items、--summary、--check-ignored、
    --unit-environment）不写库，落在哪都照读：共享门禁的 stage-selftest.sh 把样本拷进 mktemp -d（/tmp）再判，样本的库就在那里。"""
    home = os.path.realpath(os.path.expanduser(given or os.environ.get("SINGLEFS_BROAD_MUTATION_HOME") or DEFAULT_HOME))
    # 只供自证的变量先判：它们带着假的一步与弄坏开关，出现在自证之外比库目录落在哪更根本；放在后面，临时目录那一拒会先把它盖掉
    check_test_only_variables(home)
    old = [name for name in OLD_LAYOUT_MARKERS if os.path.lexists(os.path.join(home, name))]
    if old and not break_is("old-layout-accepted"):
        refuse(f"库目录 {home} 是「一版一个目录」那时的旧库（有 {'、'.join(old)}）：旧库目录与旧版本目录整个弃用，判定不迁移、不补签",
               f"另起一个新的库目录：默认 {DEFAULT_HOME}（或 --home / SINGLEFS_BROAD_MUTATION_HOME 指到一个空目录），照提交流程第 2b 步不带选条参数重判；"
               "旧的那一个确认没用了就自己删掉（连同里面的编译目录：每一趟的编译目录建在这一趟的运行目录里，跑完删，不跨趟复用）")
    if test_variable("BROAD_MUTATION_ALLOW_VOLATILE_HOME") == "1":
        return home
    if read_only and not break_is("read-only-volatile-refused"):
        return home
    existing = home
    while not os.path.exists(existing):
        existing = os.path.dirname(existing)
    under_volatile = any(home == prefix or home.startswith(prefix + "/") for prefix in VOLATILE_PREFIXES)
    if under_volatile or filesystem_type_of(existing) in VOLATILE_FILESYSTEMS:
        refuse(f"库目录 {home} 落在开机会清空的目录里（/tmp、/run、/dev/shm 或 tmpfs）：跑了几天的判定一重启就没了",
               f"换一个落在盘上的目录：--home <目录> 或 SINGLEFS_BROAD_MUTATION_HOME=<目录>；默认是 {DEFAULT_HOME}")
    return home


def resolve_source(given, home, read_only=False):
    """被判的那棵工作树（--source，默认本仓根）。要换树的用法里它与库目录不许一个套着另一个；只读的用法（不换树）不拦。"""
    source = os.path.realpath(os.path.expanduser(given or REPOSITORY))
    if not os.path.isdir(source):
        refuse(f"--source 给的 {source} 不是目录", "给被判的那棵工作树（主工作区或一份副本），不给就是本仓根")
    if not read_only and (path_is_under(source, home) or path_is_under(home, source)):
        refuse(f"--source {source} 与库目录 {home} 一个套着另一个：换树会把库目录自己（专用树、编译目录）当源码拷进去",
               "库目录与被判的工作树各在各处：库目录默认在 ~/.local/share/singlefs/ 下")
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
    if break_is("exclude-re-ignored"):
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
    """清单按单元记进 lists.tsv（文件头「库目录」）：被变异包里每一个单元一行（没有变异的记空表）；之后照这张表在此刻的源码上复原一遍，
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
        append_rows(library.lists_path, LIST_COLUMNS, rows)
    restored, unlisted, unrestorable = restore_list(judgement, cache)
    if [item_identity(item) for item in restored] != [item_identity(item) for item in items] and not break_is("list-cache-unchecked"):
        missing = sorted(set(map(item_identity, items)) - set(map(item_identity, restored)))
        extra = sorted(set(map(item_identity, restored)) - set(map(item_identity, items)))
        refuse(f"清单按单元存之后复原不回去：cargo-mutants 给了 {len(items)} 条，照 lists.tsv 复原出 {len(restored)} 条（少 {len(missing)} 条，例 {missing[:1]}；"
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
                "relative-position-counts-comments", "dependency-lines-dropped", "dependency-whole-lock", "profile-ignored", "manifest-text-in-global-key",
                "exclusions-in-global-key", "catching-test-exclusion-ignored", "unviable-rejudged", "missed-coverers-ignored", "missed-always-rejudged",
                "escalated-not-carried", "escalated-coverers-ignored", "list-cache-context-dropped", "outside-mutation-selected", "const-context-runs-selected", "spawning-binary-selected",
                "coverage-paths-unnormalized", "nextest-filter-single-argument", "limit-below-smt", "coverers-cfg-inactive-refused")
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
    """一条在判定库里键相同、沿用判据认的那几行的结局（按文件里的次序）→ ItemState（文件头「判定行」末段）。"""
    return state_of_outcomes(outcomes)


class Judgement:
    """一棵树上的判定：单元（UnitTree）、全局那几行、每条的键、判定库里键相同的行、沿用与状态。"""

    def __init__(self, library, root, arguments, for_run, exclusions, toolchain=None, tools=None):
        self.rules, self.library, self.root, self.arguments, self.for_run = prune_rules(), library, root, arguments, for_run
        try:
            self.tree = self.rules.UnitTree(root)
        except Exception as error:  # lib_function_units 起不来：根不在、清单读不出
            refuse(f"在 {root} 上切不出单元（scripts/gate/lib_function_units.py）：{error}", "看报错：Cargo.toml 要读得了；lib_function_units.py 坏了就先修它", EXIT_INCONSISTENT)
        for path in self.tree.files:
            self.tree.text(path)    # 先把原文都读进来：变异改树的时候算测试键、覆盖者键读的还是没改过的
        self.exclusions = list(exclusions)
        build_root = library.tree if (not for_run and os.path.isdir(library.tree)) else root    # 构建环境与 rustc -Vv 在驱动编译的那一处算
        self.toolchain = toolchain if toolchain is not None else toolchain_text(build_root)
        self.tools = tools if tools is not None else tool_versions(arguments)
        exclusions_digest = hashlib.sha256("".join(f"{binary} {test}\n" for binary, test, _reason in self.exclusions).encode()).hexdigest()
        self.judging_key = judging_units_key()
        self.globals = self.rules.global_lines(self.toolchain, self.tools, build_environment_of(build_root, os.environ), self.judging_key, exclusions_digest)
        if break_is("limits-in-key"):
            self.globals.append(f"limits floor={arguments.timeout_floor} memory={arguments.memory_cap} cpu={arguments.cpu_multiplier}")
        self.mutants_version = next((line for line in self.tools if "mutants" in line), " | ".join(self.tools))
        self.mutants_configuration = self.rules.mutants_configuration_digest(root)
        self.keys, self._tests = {}, None
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

    def tests(self):
        """静态列出的测试（覆盖者键用）：第一段与慢档用例（harness 档四个包）、checker 档、被变异包的文档测试；排除表里的不算。"""
        if self._tests is None:
            self._tests = (self.rules.test_entries(self.tree, (*HARNESS_TIER_PACKAGES, CHECKER_TIER_PACKAGE), self.exclusions)
                           + self.rules.doctest_entries(self.tree, MUTATED_PACKAGES))
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
        if outcome == "caught" and row["caught_by"] not in ESCALATED_LABELS:
            test_now, test_why = self.test_key_for(row["caught_by"])
        if outcome == "missed" or (outcome == "caught" and row["caught_by"] in ESCALATED_LABELS):
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
        """覆盖那一步的各条（静态列出的）：{(二进制, 用例): 记录键或 None}。第一段与慢档用例逐条，记录键是那条测试的测试键；
        checker 档逐个二进制（用例写 *），记录键取那个二进制里各条测试的测试键（有一条算不出就是 None，每趟现跑）。"""
        jobs = {}
        if stage == "one":
            for entry in self.rules.test_entries(self.tree, HARNESS_TIER_PACKAGES, self.exclusions):
                if not entry["ignored"]:
                    key, _forms = self.rules.test_key(self.tree, entry["unit"], entry["target"], self.globals)
                    jobs[(entry["binary"], entry["test"])] = hashlib.sha256(f"coverage-v1\none\n{key}\n".encode()).hexdigest() if key else None
            return jobs
        for entry in self.rules.test_entries(self.tree, HARNESS_TIER_PACKAGES, self.exclusions):
            if entry["ignored"]:
                key, _forms = self.rules.test_key(self.tree, entry["unit"], entry["target"], self.globals)
                jobs[(entry["binary"], entry["test"])] = hashlib.sha256(f"coverage-v1\nslow\n{key}\n".encode()).hexdigest() if key else None
        by_binary = {}
        for entry in self.rules.test_entries(self.tree, (CHECKER_TIER_PACKAGE,), self.exclusions):
            key, _forms = self.rules.test_key(self.tree, entry["unit"], entry["target"], self.globals)
            target = entry["target"].split(":", 2)
            label = "lib" if target[1] == "lib" else target[2]
            by_binary.setdefault(f"checker:{label}", []).append(key)
        for binary, keys in by_binary.items():
            jobs[(binary, "*")] = (hashlib.sha256(("coverage-v1\nchecker\n" + "".join(f"{key}\n" for key in sorted(keys))).encode()).hexdigest()
                                   if all(keys) else None)
        return jobs

    def selection(self, item, runner):
        """C2 / E1 / E6：这条变异每一步跑哪些测试。{"one": None 全部 / [(二进制, 用例)], "slow": None / [二进制], "checker": None / [目标], "reason": …}。"""
        current = self.key_of(item)
        everything = self.rules.run_everything_reason(self.tree, current)
        if everything or break_is("coverage-selection-ignored") or "one" not in runner.coverage_available:
            return {"one": None, "slow": None, "checker": None, "reason": everything or "no-coverage"}
        unit = current["unit"]
        fingerprint = f"{unit.kind}:{unit.fingerprint}"
        spawning = self.rules.spawning_binaries(self.tree, HARNESS_TIER_PACKAGES, runner.profile_counts)
        chosen = {"one": self.rules.select_tests(runner.coverage["one"], fingerprint, spawning), "reason": "coverage"}
        checker_spawning = {binary.split("::", 1)[1] if "::" in binary else "lib" for binary in self.rules.spawning_binaries(self.tree, (CHECKER_TIER_PACKAGE,))}
        if "two" not in runner.coverage_available or break_is("stage-two-not-by-binary"):
            chosen["slow"] = chosen["checker"] = None
            return chosen
        chosen["slow"] = sorted({job[0] for job, covered in runner.coverage["slow"].items() if fingerprint in covered} | (spawning & {job[0] for job in runner.coverage["slow"]}))
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
        → None（不按条：编译步、认不出的步、不经 runner 的步，弄坏开关 baseline-refused-step-whole）或
        {"style": nextest / libtest, "records": [记录], "refused": [(二进制, 用例)], "whole": [整个照跑的二进制], "refused_names": {二进制: [用例]}}。
        nextest 那两步：一条测试一份记录（kind test，键 = 去掉线程数的命令、全局那几行、它自己的单元闭包键）；有条目级宏调用的二进制另一份
        「宏调用展开的那几条」（kind rest，键 = 那个二进制里宏调用的闭包键多重集），宏调用的闭包拒算的二进制整个照跑；拒算的测试每趟照跑。
        checker 档快档（cargo test，runner 按二进制量 CPU 秒）：一个二进制一份记录（kind binary，键 = 它里面闭包算得出的判定与宏调用的闭包键多重集；
        有拒算的，这一份量的是 --exact --skip 掉它们的那一趟），拒算的那几条另起 --exact 照跑；宏调用拒算的二进制整个照跑。
        记录里的二进制、名字只当索引，不进键（rest 另按二进制找，改名只会重跑、不会错沿用）。"""
        name, _stage, kind, command, _failed = step
        if kind != "test" or name not in RUNNER_TEST_STEPS or break_is("baseline-refused-step-whole"):
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
            group = groups.setdefault(entry["binary"], {"target": target, "tests": [], "refused": [], "macros": [], "macro_refused": False, "names": set()})
            key, refused = self.release_entry_key(entry, global_lines) if release else self.rules.test_key(self.tree, entry["unit"], entry["target"], global_lines)
            is_macro = entry["unit"].kind == "item_macro"
            if not is_macro:
                group["names"].add(entry["test"])
            if key is None and refused == ["entry-cfg-inactive"]:
                continue
            if is_macro:
                group["macros"].append(key)
                group["macro_refused"] = group["macro_refused"] or key is None
            elif key is None:
                group["refused"].append(entry["test"])
            else:
                group["tests"].append((entry["test"], key))
        plan = {"style": style, "records": [], "refused": [], "whole": [], "refused_names": {}}
        for binary, group in sorted(groups.items()):
            target = group["target"]
            stem = (target.name if target is not None else binary.rsplit("/", 1)[-1].rsplit("::", 1)[-1]).replace("-", "_")
            selector = (["--lib"] if target is None or target.kind == "lib" else [f"--{target.kind}", target.name])
            plan["refused"] += [(binary, test) for test in group["refused"]]
            if group["refused"]:
                plan["refused_names"][binary] = sorted(group["refused"])
            if style == "nextest":
                plan["records"] += [{"kind": "test", "binary": binary, "test": test, "stem": stem, "key": digest(head + [f"entry\t{key}"])}
                                    for test, key in group["tests"]]
                if group["macro_refused"]:
                    plan["whole"].append({"binary": binary, "stem": stem, "selector": selector})
                elif group["macros"]:
                    plan["records"].append({"kind": "rest", "binary": binary, "test": "", "stem": stem, "names": sorted(group["names"]),
                                            "key": digest(head + ["rest"] + [f"macro\t{key}" for key in sorted(group["macros"])])})
                continue
            if group["macro_refused"]:
                plan["whole"].append({"binary": binary, "stem": stem, "selector": selector})
                continue
            plan["records"].append({"kind": "binary", "binary": binary, "test": "*", "stem": stem, "selector": selector, "refused": sorted(group["refused"]),
                                    "key": digest(head + [f"entry\t{key}" for key in sorted(key for _test, key in group["tests"])]
                                                  + [f"macro\t{key}" for key in sorted(group["macros"])])})
        return plan

    def baseline_entries(self, name):
        """一步真跑到的判定：→ (判定列表, 按不按 --release 的 cfg 收闭包)；认不出的步交 (None, False)。测试函数照 mutation-prune.py 的 test_entries 静态列，
        另加这几个目标里的条目级宏调用（宏展开出来的测试静态列不出：把宏调用连同它的闭包算进去）；文档测试照 doctest_entries。"""
        if name == "one-test":
            return [entry for entry in self.rules.test_entries(self.tree, HARNESS_TIER_PACKAGES) if not entry["ignored"]] + self.macro_entries(HARNESS_TIER_PACKAGES), False
        if name in (SLOW_STEP_NAME, OLD_SLOW_STEP_NAME):
            return ([entry for entry in self.rules.test_entries(self.tree, HARNESS_TIER_PACKAGES, self.exclusions) if entry["ignored"]]
                    + self.macro_entries(HARNESS_TIER_PACKAGES)), False
        if name == "two-test-doc":
            return self.rules.doctest_entries(self.tree, MUTATED_PACKAGES), False
        if name == "two-test-checker-tier":
            return ([entry for entry in self.rules.test_entries(self.tree, (CHECKER_TIER_PACKAGE,)) if not entry["ignored"]]
                    + self.macro_entries((CHECKER_TIER_PACKAGE,))), True
        return None, False

    def macro_entries(self, packages):
        """这几个包的 lib、bin、tests 目标里的条目级宏调用（item_macro 单元），照判定的写法交（二进制、名字只当报错的索引）。"""
        found = []
        for target in self.tree.workspace.targets.values():
            if target.crate.package not in packages or target.kind not in ("lib", "test", "bin"):
                continue
            for path in sorted(target.files):
                found += [{"binary": self.rules.target_binary(target), "test": unit.name, "unit": unit, "target": target.id, "ignored": False}
                          for unit in self.tree.code_units(path) if unit.kind == "item_macro"]
        return found

    def release_entry_key(self, entry, global_lines):
        """带 --release 的那一步（checker 档快档）的一条判定：cfg 去掉 debug_assertions 收闭包（试法照 mutation-prune.py 的 UnitTree.closure：
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
        packages = set(MUTATED_PACKAGES if name == "two-test-doc" else (CHECKER_TIER_PACKAGE,) if "checker-tier" in name else HARNESS_TIER_PACKAGES)
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


# ── 忽略用例登记：第二段慢档用例那一步排掉哪些、新加的忽略用例有没有登记 ───────────────────────────────────────

IGNORE_ATTRIBUTE = re.compile(r'^(?P<indent>\s*)#\[ignore(?:\s*=\s*"(?P<reason>(?:[^"\\]|\\.)*)")?\s*\]\s*$')
IGNORE_LIKE_ATTRIBUTE = re.compile(r"^\s*#\[(?:ignore\b|cfg_attr\(.*\bignore\b)")
FUNCTION_LINE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)")
MODULE_OPEN_LINE = re.compile(r"^(?P<indent>\s*)(?:pub(?:\([^)]*\))?\s+)?mod\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*\{\s*$")
EXCLUSION_WORD = re.compile(r"^[A-Za-z0-9_:/.\-]+$")
FUNCTION_LOOKAHEAD_LINES = 40


def load_exclusions(path=None):
    """排除表：[(二进制 id, 用例名, 理由)]。每一行「<二进制 id><空格><用例名><TAB><理由>」；写坏的、重复的行、表不在都拒（退 1）。"""
    path = path or test_variable("BROAD_MUTATION_EXCLUSIONS") or EXCLUSIONS_FILE
    howto = (f"排除表是 {path}：一行一条，<二进制 id><空格><用例名><TAB><理由>（# 起头的行是注释），二进制 id 与用例名照 cargo nextest list 的写法，"
             "例：singlefs-harness::random_histories shrink_one_failing_seed_from_the_environment")
    if not os.path.isfile(path):
        refuse(f"排除表 {path} 不在", howto, EXIT_INCONSISTENT)
    table = read_table(path, 2)
    problems = []
    if table.malformed_lines:
        problems.append(f"第 {'、'.join(str(number) for number in table.malformed_lines[:5])} 行不是「<二进制 id> <用例名><TAB><理由>」两段")
    if table.partial_tail:
        problems.append("末行没有换行结尾")
    rows, seen = [], set()
    for head, reason in table.rows:
        words = head.split(" ")
        if len(words) != 2 or not all(EXCLUSION_WORD.match(word) for word in words) or not reason.strip():
            problems.append(f"「{head}」这一行的头不是「<二进制 id> <用例名>」两个词（只认字母数字 _ : / . -）、或理由是空的")
            continue
        if tuple(words) in seen:
            problems.append(f"「{head}」重复了")
            continue
        seen.add(tuple(words))
        rows.append((words[0], words[1], reason))
    if problems and not break_is("malformed-exclusion-row-skipped"):
        refuse(f"排除表写坏了：{'；'.join(problems)}", howto, EXIT_INCONSISTENT)
    return rows


def exclusion_filter(exclusions):
    """第二段慢档用例那一步的 -E 表达式：排掉排除表里的每一条；表是空的交 None。"""
    if not exclusions or break_is("exclusion-filter-dropped"):
        return None
    terms = " | ".join(f"(binary_id(={binary_id}) & test(={test}))" for binary_id, test, _reason in exclusions)
    return f"not ({terms})"


def source_files_with_identity(root):
    """harness 档四个包里 src/ 与 tests/ 下的每个 .rs：(路径, 相对路径, 二进制 id, 文件自己在那个二进制里的模块路径)。
    二进制 id 与 cargo nextest 的写法一致：tests/<名>.rs 是 <包>::<名>，lib 的 src 是 <包>，src/bin/<名>.rs 是 <包>::bin/<名>。"""
    found = []
    for package in HARNESS_TIER_PACKAGES:
        for kind in ("src", "tests"):
            base = os.path.join(root, "crates", package, kind)
            for directory, directory_names, file_names in os.walk(base):
                directory_names[:] = sorted(directory_names)
                for file_name in sorted(file_names):
                    if not file_name.endswith(".rs"):
                        continue
                    path = os.path.join(directory, file_name)
                    relative = os.path.relpath(path, base).replace(os.sep, "/")
                    parts = relative[:-3].split("/")
                    if kind == "tests":
                        binary_id, module = f"{package}::{parts[0]}", parts[1:]
                    elif parts == ["main"]:
                        binary_id, module = f"{package}::bin/{package}", []
                    elif parts[0] == "bin" and len(parts) > 1:
                        binary_id, module = f"{package}::bin/{parts[1]}", parts[2:]
                    else:
                        binary_id, module = package, ([] if parts == ["lib"] else parts)
                    if module and module[-1] in ("main", "mod"):
                        module = module[:-1]
                    found.append((path, os.path.relpath(path, root), binary_id, module))
    return found


def ignored_cases_in_file(path, display, binary_id, file_module):
    """一个源文件里每条 #[ignore] 用例：({binary_id, test, reason, where}…)、认不出写法的位置。
    模块路径按缩进配对 `mod x {` 与同缩进的 `}` 取（仓里的源码都过 rustfmt）；用例名是 #[ignore] 之后第一个 fn。"""
    with open(path, encoding="utf-8", errors="replace") as handle:
        lines = handle.read().split("\n")
    cases, unparsed, stack = [], [], []
    for index, line in enumerate(lines):
        stripped = line.strip()
        indent = line[:len(line) - len(line.lstrip())]
        if stack and stripped.startswith("}") and stack[-1][0] == indent:
            stack.pop()
            continue
        opened = MODULE_OPEN_LINE.match(line)
        if opened:
            stack.append((opened.group("indent"), opened.group("name")))
            continue
        if stripped.startswith("//"):
            continue
        if stack and break_is("ignored-scan-skips-nested-modules"):
            continue
        attribute = IGNORE_ATTRIBUTE.match(line)
        if attribute is None:
            if IGNORE_LIKE_ATTRIBUTE.match(line):
                unparsed.append(f"{display}:{index + 1}")
            continue
        name = next((found.group("name") for found in (FUNCTION_LINE.match(later) for later in lines[index + 1:index + 1 + FUNCTION_LOOKAHEAD_LINES]) if found), None)
        if name is None:
            unparsed.append(f"{display}:{index + 1}")
            continue
        cases.append({"binary_id": binary_id, "test": "::".join([*file_module, *(module_name for _indent, module_name in stack), name]),
                      "reason": attribute.group("reason") or "", "where": f"{display}:{index + 1}"})
    return cases, unparsed


class IgnoredClassification(NamedTuple):
    timed: list          # 理由写着「harness 慢档用例：」、不在排除表里：进第二段
    excluded: list       # 在排除表里、理由不是慢档用例：第二段排掉
    unregistered: list   # 两样都不是
    both: list           # 理由写着慢档用例、又在排除表里
    stale: list          # 排除表里指着树里没有的用例：[(二进制 id, 用例名, 理由)]
    unparsed: list       # 认不出写法的 #[ignore]

    def problems(self):
        def first(cases):
            case = cases[0]
            return f"例：{case['binary_id']} {case['test']}（{case['where']}，理由「{case['reason'][:30]}」）"
        problems = []
        if self.unregistered:
            problems.append(f"{len(self.unregistered)} 条 #[ignore] 用例既不是「{TIMED_CASE_REASON_PREFIX}」开头的理由、也不在排除表里，{first(self.unregistered)}")
        if self.both:
            problems.append(f"{len(self.both)} 条用例理由写着「{TIMED_CASE_REASON_PREFIX}」又在排除表里，{first(self.both)}")
        if self.stale:
            problems.append(f"排除表里 {len(self.stale)} 行指着树里没有的用例，例：{self.stale[0][0]} {self.stale[0][1]}")
        if self.unparsed:
            problems.append(f"{len(self.unparsed)} 处 #[ignore] 写法认不出（要单独一行的 #[ignore] 或 #[ignore = \"理由\"]，后面 {FUNCTION_LOOKAHEAD_LINES} 行内要有 fn），例：{self.unparsed[0]}")
        return problems

    def line(self):
        return (f"BROAD_MUTATION_IGNORED timed={len(self.timed)} excluded={len(self.excluded)} unregistered={len(self.unregistered)} "
                f"stale={len(self.stale)} both={len(self.both)} unparsed={len(self.unparsed)}")


def classify_ignored_cases(root, exclusions):
    cases, unparsed = [], []
    for path, display, binary_id, module in source_files_with_identity(root):
        found, bad = ignored_cases_in_file(path, display, binary_id, module)
        cases += found
        unparsed += bad
    names = {(case["binary_id"], case["test"]) for case in cases}
    excluded_names = {(binary_id, test) for binary_id, test, _reason in exclusions}
    timed, excluded, unregistered, both = [], [], [], []
    for case in cases:
        is_timed = case["reason"].startswith(TIMED_CASE_REASON_PREFIX)
        is_excluded = (case["binary_id"], case["test"]) in excluded_names
        (both if is_timed and is_excluded else timed if is_timed else excluded if is_excluded else unregistered).append(case)
    stale = [row for row in exclusions if (row[0], row[1]) not in names]
    if break_is("unregistered-ignored-passes"):
        unregistered = []
    if break_is("stale-exclusion-passes"):
        stale, both = [], []
    return IgnoredClassification(timed, excluded, unregistered, both, stale, unparsed)


def check_ignored_registration(root, exclusions):
    """root 那棵树里 harness 档的每条 #[ignore] 用例都登记了：慢档用例进第二段、大档与工具在排除表里；排除表不指着不存在的。不对就拒（退 1）。"""
    classified = classify_ignored_cases(root, exclusions)
    say(classified.line())
    problems = classified.problems()
    if problems:
        refuse(f"树里的忽略用例登记对不上：{'；'.join(problems)}",
               f"新加的忽略用例：是 harness 慢档用例就把 #[ignore] 的理由写成「{TIMED_CASE_REASON_PREFIX}…」开头（它进第二段）；是按环境变量起的大档或工具，"
               f"就在 {EXCLUSIONS_FILE} 加一行（<二进制 id> <用例名><TAB><理由>，名字照 cargo nextest list）。排除表指着不存在的用例：删掉那一行或改对名字"
               "（排除表与被判的那棵工作树要对得上：用例是这棵树里还没有的，就等它进了树再加那一行）",
               EXIT_INCONSISTENT)
    return classified


def check_ignored_command(root, arguments):
    """--check-ignored：只判登记，再打第二段慢档用例那一步的完整命令（看 -E 排掉了哪几条）。"""
    exclusions = load_exclusions()
    check_ignored_registration(root, exclusions)
    step = next(step for step in step_table(arguments, exclusions) if step[0] in (SLOW_STEP_NAME, OLD_SLOW_STEP_NAME))
    say(f"STEP_COMMAND\t{step[0]}\t{shlex.join(step[3])}")
    return 0


# ── 每一步命令 ─────────────────────────────────────────────────────────────────────────────────────────────

def package_arguments(packages):
    return [word for package in packages for word in ("-p", package)]


def step_table(arguments, exclusions):
    """每一步：(步名, 哪一段, build / test, 命令, 测试没过的退出码)。
    第二段只补第一段没跑到的：文档测试（被变异的两个包）、harness 档四个包里标了 #[ignore] 的慢档用例（与第一段同一种起法，加 --run-ignored only 只跑它们，
    再用 -E 排掉排除表里按环境变量起的大档与工具，exclusions 是排除表的行）、checker 档快档。第一段跑过的不标 #[ignore] 的用例第二段不再跑。"""
    harness = package_arguments(HARNESS_TIER_PACKAGES)
    threads = str(arguments.threads)
    sync_free = ["eatmydata"]
    stage_two_prefix = sync_free if arguments.stage_two_eatmydata else []
    checker_tier = ["--release", "-p", CHECKER_TIER_PACKAGE, "--lib", "--tests"]
    filterset = exclusion_filter(exclusions)
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
        ("two-build-harness", "two", "build", harness_build, None),
        ("two-test-doc", "two", "test", stage_two_prefix + ["cargo", "test", "--locked", "--doc", *package_arguments(MUTATED_PACKAGES)], CARGO_FAILED_EXIT),
        (OLD_SLOW_STEP_NAME if break_is("slow-step-keeps-old-name") else SLOW_STEP_NAME, "two", "test",
         harness_test + ["--run-ignored", "only"] + (["-E", filterset] if filterset else []), NEXTEST_TEST_FAILED_EXIT),
        ("two-build-checker-tier", "two", "build", ["cargo", "test", "--locked", *checker_tier, "--no-run"], None),
        ("two-test-checker-tier", "two", "test", stage_two_prefix + ["cargo", "test", "--locked", *checker_tier], CARGO_FAILED_EXIT),
    ]


def descendants_of(process_id):
    """一个进程的全部后代进程号（读 /proc/<pid>/task/<tid>/children，逐层往下）；读不到的那一支略过。"""
    found, pending = [], [process_id]
    while pending:
        current = pending.pop()
        try:
            task_directory = f"/proc/{current}/task"
            for task in os.listdir(task_directory):
                with open(f"{task_directory}/{task}/children", encoding="utf-8") as handle:
                    children = [int(word) for word in handle.read().split()]
                found += children
                pending += children
        except (OSError, ValueError):
            continue
    return found


def own_scope_unit(wrapper_process_id):
    """内存包装给这一步开的 scope 的 unit 名：在包装的后代里找 cgroup 落在 singlefs-memory-cap-<包装的进程号>-….scope 里的；找不到交 None。
    名字里的进程号要正好是自己起的那个包装：别人开的 scope（驱动自己被包在外层的那一个也算别人的）不认。"""
    for process_id in descendants_of(wrapper_process_id):
        try:
            with open(f"/proc/{process_id}/cgroup", encoding="utf-8") as handle:
                lines = handle.read().splitlines()
        except OSError:
            continue
        for line in lines:
            found = OWN_SCOPE_UNIT.search(line)
            if found and found.group(2) == str(wrapper_process_id):
                return found.group(1)
    return None


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


class Escalation(NamedTuple):
    exit_code: int          # 交回调用方接着判的退出码：升档那趟的；没升档、或升档那趟没跑成时是原来那一趟的
    seconds: float          # 升档那趟的秒数；0 是没升档（这一步没有整步限时、或上限写法认不出）
    label: str              # 证据齐的抓到：caught-by-timeout / caught-by-memory；否则空
    note: str               # 记进结果行备注的一句
    environmental: str      # 照环境型结局记、没结论：timeout / memory-cap；否则空


def memory_cap_bytes(text):
    """run-with-memory-cap.sh 的上限写法（正整数加 K / M / G / T，1024 进制）→ 字节；认不出交 None。"""
    found = MEMORY_CAP_FORM.match(text or "")
    return int(found.group(1)) * MEMORY_CAP_UNITS[found.group(2)] if found else None


def wrapper_hit_text(exit_code, limit_text):
    """包装撞上限时往 stderr 打的那一句里认的一段（scripts/hosts/run-with-memory-cap.sh 文件头「退出码」的 253、250）。"""
    return f"超过限时 {limit_text} 秒" if exit_code == TIME_LIMIT_HIT_EXIT else f"撞了内存上限 {limit_text}"


def escalation_evidence_problems(first_exit, escalated_exit, baseline, first_amount, escalated_amount, first_text, escalated_text, first_log_text, escalated_log_text):
    """文件头「升档」①–③：证据缺哪几样（空表是证据齐）。baseline 是基线秒数或峰值字节（没记是 None），amount 是两趟的限时秒数或上限字节。"""
    problems = []
    if escalated_exit != first_exit:
        problems.append(f"两趟结局不同（原来退 {first_exit}、升档那趟退 {escalated_exit}）")
    multiple = ESCALATION_TIME_BASELINE_MULTIPLE if first_exit == TIME_LIMIT_HIT_EXIT else ESCALATION_MEMORY_BASELINE_MULTIPLE
    if baseline is None:
        problems.append("这把基线键、这个线程数下没有这一步在没改过的树上的基线（超时要 baseline.tsv 的绿记录，撞顶要那一行记的峰值）")
    elif escalated_amount is None or escalated_amount < multiple * baseline:
        problems.append(f"升档那趟的 {escalated_text} 不到基线 {baseline:g} 的 {multiple} 倍")
    if escalated_amount is None or first_amount is None or escalated_amount <= first_amount:
        problems.append(f"升档那趟的 {escalated_text} 没比原来的 {first_text} 大")
    if wrapper_hit_text(first_exit, first_text) not in first_log_text:
        problems.append(f"原来那趟的日志里没有包装打的「{wrapper_hit_text(first_exit, first_text)}」")
    if wrapper_hit_text(escalated_exit, escalated_text) not in escalated_log_text:
        problems.append(f"升档那趟的日志里没有包装打的「{wrapper_hit_text(escalated_exit, escalated_text)}」")
    return problems


def real_alerts_record():
    """基线那几步交给内存包装的告警记录（文件头「内存」）：(变量名 SINGLEFS_CORE_USAGE_ALERTS, scripts/hosts/core_usage.py 的 alerts_path() 给的路径)。
    按文件路径导入那一份取，不另写一份路径；工作根那份库导入失败时路径是 None。"""
    specification = importlib.util.spec_from_file_location("core_usage_for_broad_mutation", CORE_USAGE_LIBRARY)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module.ALERTS_VARIABLE, module.alerts_path()


class Runner:
    def __init__(self, library, arguments):
        self.library, self.arguments = library, arguments
        try:
            self.exclusions = load_exclusions()
        except Refusal:
            if not arguments.skip_stage_two:
                raise
            self.exclusions = []     # 只跑第一段的趟用不着排除表跑第二段，表坏了也不挡它（键里不收排除表）
        self.steps = step_table(arguments, self.exclusions)
        self.fake_driver = test_variable("BROAD_MUTATION_STEP_DRIVER") == "1"
        self.selftest_work = test_variable(SELFTEST_WORK_VARIABLE) or ""   # 假一步在它下面（文件头「只供测试」）
        self.fake_driver_via_wrapper = test_variable("BROAD_MUTATION_STEP_VIA_WRAPPER") == "1"
        self.baseline_seconds = {}
        self.stages_with_fresh_baseline = set()
        self.judgement = None
        self.full = False
        self.runner_path = None
        self.coverage = {"one": {}, "slow": {}, "checker": {}}
        self.coverage_available = set()
        self.profile_counts = {}
        self.baseline_keys = {}
        self.baseline_refusals = {}    # 步名 → 基线键拒算的那一句（Judgement.baseline_key）
        self.run_token = secrets.token_hex(16)   # 拒算的基线这一趟独有的那一把键从它算：记下的基线行以后哪一趟都对不上、不沿用
        self.refused_directory_cleared = False
        self.current_key = ""

    def times_path(self, step_name):
        return os.path.join(self.baseline_directory_of(step_name), f"test-times.{self.arguments.threads}.{step_name}.tsv")

    def baseline_key_of(self, step_name):
        """这一步此刻的基线键（Judgement.baseline_key：这一步真跑到的判定的单元闭包）。拒算的（文件头「基线」）交这一趟独有的一把（run_token 算的）：
        这一趟照跑、记的基线行与升档登记照常对得上，以后哪一趟都对不上、不沿用。"""
        if step_name not in self.baseline_keys:
            step = next((step for step in self.steps if step[0] == step_name), (step_name, "", "test", [], None))   # 覆盖那一步不在步表里
            key, refused = self.judgement.baseline_key(step, self.arguments.threads)
            if key is None:
                self.baseline_refusals[step_name] = refused
                key = hashlib.sha256(f"refused-baseline\0{step_name}\0{self.run_token}".encode()).hexdigest()
            self.baseline_keys[step_name] = key
        return self.baseline_keys[step_name]

    def baseline_directory_of(self, step_name):
        """这一步的逐条时间表与峰值放哪：键算得出的放 baselines/<键前 16 位>/；拒算的放 baselines/refused/（一份，这一趟头一回用到时清空，不按趟攒）。"""
        key = self.baseline_key_of(step_name)
        if step_name not in self.baseline_refusals:
            return self.library.baseline_directory(key)
        directory = os.path.join(self.library.home, "baselines", REFUSED_BASELINE_DIRECTORY)
        if not self.refused_directory_cleared:
            remove_tree(directory)
            self.refused_directory_cleared = True
        return directory

    def limits_path(self):
        return os.path.join(self.library.home, f"limits.{self.arguments.threads}.tsv")

    def prepare_limits(self, stages=None):
        """C4：基线逐条（checker 档逐个二进制）的 CPU 秒 → 上限表（倍数 × CPU + 下限，墙钟兜底另算），runner 在 limit 模式读它。
        stages 给了只读这几段的测试步（不跑的那一段不为它现算基线键）。"""
        rules = prune_rules()
        times = {}
        stage_of = {step[0]: step[1] for step in self.steps}
        for step_name in RUNNER_TEST_STEPS:
            if stages is not None and stage_of.get(step_name) not in stages:
                continue
            times.update(rules.read_times(self.times_path(step_name)))
        rules.write_limits(self.limits_path(), rules.limits_table(times, self.arguments.cpu_multiplier, self.arguments.cpu_floor,
                                                                  self.arguments.wall_multiplier, self.arguments.wall_floor))

    def command_for(self, step, selection):
        """这一条这一步的命令与「跑了哪些」的一句：按覆盖挑出来的测试（C2）、第二段按二进制（E6）；交 (None, …) 是这一步没有要跑的。"""
        name, command = step[0], list(step[3])
        selection = selection or {}
        if name == "one-test" and selection.get("one") is not None:
            if not selection["one"]:
                return None, "none"
            filters = prune_rules().nextest_filters(selection["one"])
            return command + [word for text in filters for word in ("-E", text)], f"{len(selection['one'])} tests"
        if name == SLOW_STEP_NAME and selection.get("slow") is not None:
            if not selection["slow"]:
                return None, "none"
            binaries = " | ".join(f"binary_id(={binary})" for binary in selection["slow"])
            if "-E" in command:
                command[command.index("-E") + 1] = f"({binaries}) & {command[command.index('-E') + 1]}"
            else:
                command += ["-E", f"({binaries})"]
            return command, ",".join(selection["slow"])
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
        """C2：没改过的树上的覆盖（第一段与慢档用例逐条测试、checker 档逐个二进制），按记录键（那条测试的测试键）在 coverage.tsv 里照用，
        对不上的那几条插桩跑（--coverage-step）；跑红拒（退 6）。之后 self.coverage[段] = {(二进制, 用例): {种类:指纹}}。"""
        judgement, rules = self.judgement, self.judgement.rules
        records = {}
        for row in read_table(self.library.coverage_path, len(COVERAGE_COLUMNS)).rows:
            entry = dict(zip(COVERAGE_COLUMNS, row))
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
                                                        "known": [list(job) for job in sorted(known)]}).encode())
            say(f"  · 覆盖 {name}：在没改过的树上插桩列一遍测试，记录键对得上的 {len(known)} 条照用、其余现跑")
            step = (name, stage, "test", ["python3", SELF_PATH, "--coverage-step", specification], None)
            exit_code, seconds = self.run_step(step, BASELINE_SEQUENCE, None, extra={"BROAD_MUTATION_STEP_SPEC": specification})
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
                append_rows(self.library.coverage_path, COVERAGE_COLUMNS, new_rows)
            say(f"  · 覆盖 {name} 好了，{seconds:.1f} 秒：照用 {len(known)} 条、现跑 {len(result.get('jobs', []))} 条（记进库的 {len(new_rows)} 条）")

    @staticmethod
    def coverage_part(stage, binary):
        if stage == "one":
            return "one"
        return "checker" if binary.startswith("checker:") else "slow"

    def step_named(self, step_name):
        return next(step for step in self.steps if step[0] == step_name)

    def log_path(self, sequence, step_name):
        prefix = "baseline" if sequence == BASELINE_SEQUENCE else str(sequence)
        return os.path.join(self.library.logs, f"{prefix}.{step_name}.log")

    def time_limit_of(self, step_name):
        """整步的墙钟兜底：编译步与文档测试步照旧；经 runner 的测试步按单条测试的 CPU 时间限（C4），不再设整步总时限。"""
        if step_name in RUNNER_TEST_STEPS and not break_is("whole-step-time-limit"):
            return None
        return int(max(self.arguments.timeout_floor, self.arguments.timeout_multiplier * self.baseline_seconds.get(step_name, 0)))

    def stop_step(self, child):
        """驱动被 TERM / INT 停：在途这一步连同内存包装给它开的 scope 一起停，不留着它跑完。
        包装的 bash 在等前台子进程时把自己的 TERM 处理推到子进程退出之后，只给它发 TERM 那一步照样跑；所以另停它开的那一个 scope
        （systemctl --user stop，systemd 给 scope 里每个进程发 TERM），到点还在就 KILL 那个 scope 与包装。发信号只点自己起的：child 与名字里带 child 进程号的 scope。"""
        grace = self.arguments.stop_grace
        unit = None if break_is("term-leaves-step-running") else own_scope_unit(child.pid)
        child.terminate()   # 只点自己起的这一个进程
        if break_is("term-leaves-step-running"):
            try:
                child.wait(timeout=grace)
            except subprocess.TimeoutExpired:
                pass
            return
        if unit is not None:
            try:
                subprocess.run(["systemctl", "--user", "stop", unit], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                               timeout=grace)
            except (OSError, subprocess.TimeoutExpired):
                subprocess.run(["systemctl", "--user", "kill", "--signal=KILL", unit], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                               stderr=subprocess.DEVNULL, check=False)
        try:
            child.wait(timeout=grace)
        except subprocess.TimeoutExpired:
            child.kill()    # 只点自己起的这一个进程
            child.wait()

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

    def baseline_peaks_path(self, step_name):
        """基线那一步跑的时候包装记峰值的那一份（RUN_WITH_MEMORY_CAP_PEAKS，包装的峰值表写法，键 BASELINE_PEAK_KEY_PREFIX + 步名）：
        每一步开跑之前删掉，跑完读出来写进 baseline.tsv 那一行的 peak_bytes（文件头「基线」），之后不再读它。"""
        return os.path.join(self.baseline_directory_of(step_name), f"peaks.{self.arguments.threads}.tsv")

    def wrapper_peak(self, step_name):
        """包装刚给这一步记下的峰值（字节）；没记交 None。"""
        try:
            with open(self.baseline_peaks_path(step_name), encoding="utf-8") as handle:
                for line in handle:
                    fields = line.rstrip("\n").split("\t")
                    if line.startswith("#") or len(fields) < 4 or fields[3] != BASELINE_PEAK_KEY_PREFIX + step_name:
                        continue
                    return int(fields[0]) if fields[0].isdigit() else None
        except OSError:
            return None
        return None

    def baseline_rows_of(self, step_name):
        """baselines.tsv 里这把基线键、这个线程数下这一步的行：[(行号, {列名: 值})]。弄坏开关 baseline-any-threads 下不看线程数（改前的取法）。"""
        threads = str(self.arguments.threads)
        baseline_key = self.baseline_key_of(step_name)
        return [(number, row) for number, row in read_baseline_rows(self.library.baselines_path)
                if row["baseline_key"] == baseline_key and row["step"] == step_name and (row["threads"] == threads or break_is("baseline-any-threads"))]

    def baseline_row(self, step_name):
        """升档取的基线（文件头「升档」①）：这把基线键、这个线程数下这一步最后一行绿记录，交 (行号, {列名: 值})；没有交 None。"""
        rows = [(number, row) for number, row in self.baseline_rows_of(step_name) if row["outcome"] == "ok"]
        return rows[-1] if rows else None

    def baseline_peak(self, step_name):
        """这把基线键、这个线程数下这一步在没改过的树上的内存峰值（字节，baseline_row 那一行的 peak_bytes）；没记过交 None。"""
        found = self.baseline_row(step_name)
        return int(found[1]["peak_bytes"]) if found and found[1]["peak_bytes"].isdigit() else None

    def recorded_baseline_seconds(self, step_name):
        """这把基线键、这个线程数下这一步最后一行绿记录的秒数（原样的文字）；没有交 None。"""
        found = self.baseline_row(step_name)
        return found[1]["seconds"] if found else None

    def run_step(self, step, sequence, time_limit, command=None, selection_text="all", runner_mode=None, extra=None, memory_cap=None, attempt="first"):
        """跑一步：交 (退出码, 秒数)。输出落进这一条这一步的日志。command 不给就用步表里的；runner_mode 给了就经 runner 起测试二进制（measure / limit）。
        memory_cap 不给取 --memory-cap（升档那趟给升过的）；attempt 是 first 或 escalated（只交给假的一步）。基线那几步另让包装把峰值记进这一版的 baseline-peaks。"""
        step_name, _stage, _kind, table_command, _test_failed_exit = step
        command = table_command if command is None else command
        memory_cap = memory_cap or self.arguments.memory_cap
        remove_tree(self.library.scratch)
        os.makedirs(self.library.scratch)
        os.makedirs(self.library.target, exist_ok=True)
        os.makedirs(self.library.logs, exist_ok=True)
        environment = dict(os.environ, CARGO_TARGET_DIR=self.library.target, TMPDIR=self.library.scratch, CARGO_TERM_COLOR="never", **(extra or {}))
        if runner_mode and self.runner_path and not break_is("cpu-limit-ignored"):
            environment[f"CARGO_TARGET_{host_triple(self.library).upper().replace('-', '_')}_RUNNER"] = self.runner_path
            environment["BROAD_MUTATION_RUNNER_MODE"] = runner_mode
            environment["BROAD_MUTATION_RUNNER_LIMITS"] = self.limits_path()
            environment["BROAD_MUTATION_RUNNER_TIMES"] = self.times_path(step_name)
        if self.fake_driver:
            environment["BROAD_MUTATION_STEP_SELECTION"] = selection_text
            environment["BROAD_MUTATION_STEP_TIME_LIMIT"] = str(time_limit)
            environment["BROAD_MUTATION_STEP_MEMORY_CAP"] = memory_cap
            environment["BROAD_MUTATION_STEP_ATTEMPT"] = attempt
        is_baseline = sequence == BASELINE_SEQUENCE
        if is_baseline:
            os.makedirs(os.path.dirname(self.baseline_peaks_path(step_name)), exist_ok=True)
            environment["RUN_WITH_MEMORY_CAP_PEAKS"] = self.baseline_peaks_path(step_name)
            environment["RUN_WITH_MEMORY_CAP_KEY"] = BASELINE_PEAK_KEY_PREFIX + step_name
            # 文件头「内存」：峰值表指到基线那一份，包装的告警会跟着写到它旁边；基线的告警照旧叫醒主 agent，调用方没设 SINGLEFS_CORE_USAGE_ALERTS 的设成真记录
            if not break_is("baseline-alerts-beside-peaks"):
                alerts_variable, alerts_record = real_alerts_record()
                if not environment.get(alerts_variable):
                    if alerts_record:
                        environment[alerts_variable] = alerts_record
                    else:
                        note(f"  ! 基线那一步的告警记录找不到工作根（{CORE_USAGE_LIBRARY} 的 alerts_path() 交了 None）：包装会把告警写到基线峰值表旁边，看门狗读不到；"
                             f"设 {alerts_variable} 指到工作根下的 core-usage-alerts.jsonl 再跑")
        # 文件头「内存」：升档那趟按它自己的上限排队；改坏了源码的那几步撞 slice 总上限时先挨杀（包装的 oom_score_adj）；基线与验环境那几步两样都不设
        environment.pop("RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP", None)
        environment.pop("RUN_WITH_MEMORY_CAP_OOM_FIRST", None)
        if attempt == "escalated":
            if not break_is("escalation-queues-by-peak"):
                environment["RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP"] = ESCALATED_QUEUE_REASON
        elif sequence not in (BASELINE_SEQUENCE, ENVIRONMENT_SEQUENCE) and not break_is("mutation-steps-not-oom-first"):
            environment["RUN_WITH_MEMORY_CAP_OOM_FIRST"] = MUTATION_STEP_OOM_REASON
        if self.fake_driver and not self.fake_driver_via_wrapper:
            argv = [os.path.join(self.selftest_work, SELFTEST_STEP_DRIVER_NAME), step_name, str(sequence), self.library.tree]
        else:
            if self.fake_driver:
                argv = ["bash", MEMORY_CAP_RUNNER, "64M", os.path.join(self.selftest_work, SELFTEST_STEP_DRIVER_NAME), step_name, str(sequence), self.library.tree]
            else:
                argv = ["bash", MEMORY_CAP_RUNNER, memory_cap, "bash", THREAD_CAP_RUNNER, str(self.arguments.threads), *command]
            environment["RUN_WITH_MEMORY_CAP_KEY"] = BASELINE_PEAK_KEY_PREFIX + step_name if is_baseline else f"broad-mutation {step_name}"
            environment.pop("RUN_WITH_MEMORY_CAP_TIME_LIMIT", None)
            if time_limit is not None:
                environment["RUN_WITH_MEMORY_CAP_TIME_LIMIT"] = str(time_limit)
        started = time.monotonic()
        with open(self.log_path(sequence, step_name), "wb") as log:
            child = subprocess.Popen(argv, cwd=self.library.tree, env=environment, stdin=subprocess.DEVNULL, stdout=log,
                                     stderr=subprocess.STDOUT, pass_fds=(self.library.lock_descriptor,))
            try:
                exit_code = self.wait_reporting_progress(child, step_name, sequence, started)
            except Stopped:
                self.stop_step(child)
                raise
        return exit_code, time.monotonic() - started

    def append_evidence_line(self, relative_log, sequence, step_name, attempt, limit_text, exit_code):
        """升档那两趟的日志末尾追加驱动的那一行（文件头「升档」④）：那条变异的键、序号、步名、哪一趟、那一趟的限时或上限、退出码。"""
        path = os.path.join(self.library.home, relative_log)
        with open(path, "rb") as handle:
            existing = handle.read()
        with open(path, "ab") as handle:
            handle.write((b"" if not existing or existing.endswith(b"\n") else b"\n")
                         + f"{ESCALATION_EVIDENCE_PREFIX} key={self.current_key} sequence={sequence} step={step_name} attempt={attempt} "
                           f"limit={limit_text} exit={exit_code}\n".encode())
            handle.flush()
            os.fsync(handle.fileno())

    def escalate_step(self, item, step, command, selection_text, runner_mode, first_exit):
        """文件头「升档」：这一条这一步退 253（超时）或 250（撞顶），当场在同一棵改过的树上升档重跑那一步一次，交 Escalation。
        两趟都超时 / 都撞顶而证据齐的往库目录登记表追加一行、交标签；证据不齐的交环境型结局；升档那趟另有结局的交它的退出码给调用方照常判。"""
        step_name, sequence = step[0], item["sequence"]
        self.current_key = item.get("verdict_key", "")
        first_time_limit = self.time_limit_of(step_name)
        first_cap = self.arguments.memory_cap
        same_limit = break_is("escalation-same-limit")
        baseline_entry = self.baseline_row(step_name)     # 登记行记它的行号（baseline_row），门禁按行号找那一行复核
        baseline_row_field = str(baseline_entry[0]) if baseline_entry else ""
        if first_exit == TIME_LIMIT_HIT_EXIT:
            if first_time_limit is None:
                return Escalation(first_exit, 0.0, "", f"{step_name} 没有整步限时（经 runner 按单条限 CPU），不升档", "")
            baseline_text = self.recorded_baseline_seconds(step_name)
            baseline = float(baseline_text) if baseline_text else None
            escalated = first_time_limit if same_limit else int(math.ceil(max(ESCALATION_TIME_FACTOR * first_time_limit,
                                                                               ESCALATION_TIME_BASELINE_MULTIPLE * (baseline or 0.0))))
            time_limit, memory_cap = escalated, first_cap
            first_text, escalated_text, first_amount, escalated_amount = str(first_time_limit), str(escalated), first_time_limit, escalated
            baseline_field, label, kind_text, multiple = baseline_text or "", ESCALATED_TIMEOUT_LABEL, "超时", ESCALATION_TIME_BASELINE_MULTIPLE
        else:
            first_bytes = memory_cap_bytes(first_cap)
            if first_bytes is None:
                return Escalation(first_exit, 0.0, "", f"--memory-cap {first_cap} 的写法认不出，不升档", "")
            baseline = self.baseline_peak(step_name)
            if same_limit:
                escalated_text = first_cap
            else:
                wanted = max(ESCALATION_MEMORY_FACTOR * first_bytes, ESCALATION_MEMORY_BASELINE_MULTIPLE * (baseline or 0))
                escalated_text = f"{int(math.ceil(wanted / MEMORY_CAP_UNITS['M']))}M"
            time_limit, memory_cap = first_time_limit, escalated_text
            first_text, first_amount, escalated_amount = first_cap, first_bytes, memory_cap_bytes(escalated_text)
            baseline_field, label, kind_text, multiple = ("" if baseline is None else str(baseline)), ESCALATED_MEMORY_LABEL, "撞顶", ESCALATION_MEMORY_BASELINE_MULTIPLE
        os.makedirs(os.path.join(self.library.home, ESCALATION_DIRECTORY), exist_ok=True)
        stem = f"{sequence}.{step_name}.{time.time_ns()}"
        first_log = os.path.join(ESCALATION_DIRECTORY, f"{stem}.first.log")
        os.replace(self.log_path(sequence, step_name), os.path.join(self.library.home, first_log))
        self.append_evidence_line(first_log, sequence, step_name, "first", first_text, first_exit)
        exit_code, seconds = self.run_step(step, sequence, time_limit, command=command, selection_text=selection_text, runner_mode=runner_mode,
                                           memory_cap=memory_cap, attempt="escalated")
        say(f"BROAD_MUTATION_ESCALATION sequence={sequence} step={step_name} first={first_text} escalated={escalated_text} exit={exit_code}")
        upgraded = f"{step_name} {kind_text}之后升档重跑（{first_text} → {escalated_text}）"
        first_environmental = "timeout" if first_exit == TIME_LIMIT_HIT_EXIT else "memory-cap"
        if exit_code not in (TIME_LIMIT_HIT_EXIT, MEMORY_CAP_HIT_EXIT):
            if exit_code in WRAPPER_INFRASTRUCTURE_EXITS:
                return Escalation(first_exit, seconds, "", f"{upgraded}：升档那趟没跑成（包装退 {exit_code}，{WRAPPER_INFRASTRUCTURE_EXITS[exit_code]}），照原结局记、没结论",
                                  first_environmental)
            if exit_code == 0:
                os.remove(os.path.join(self.library.home, first_log))
            return Escalation(exit_code, seconds, "", upgraded, "")
        escalated_log = os.path.join(ESCALATION_DIRECTORY, f"{stem}.escalated.log")
        os.replace(self.log_path(sequence, step_name), os.path.join(self.library.home, escalated_log))
        self.append_evidence_line(escalated_log, sequence, step_name, "escalated", escalated_text, exit_code)
        texts = []
        for relative in (first_log, escalated_log):
            with open(os.path.join(self.library.home, relative), encoding="utf-8", errors="replace") as handle:
                texts.append(handle.read())
        problems = escalation_evidence_problems(first_exit, exit_code, None if not baseline_field else float(baseline_field), first_amount, escalated_amount,
                                                first_text, escalated_text, texts[0], texts[1])
        if problems and not break_is("escalation-evidence-unchecked"):
            return Escalation(exit_code, seconds, "", f"{upgraded}，仍{'超时' if exit_code == TIME_LIMIT_HIT_EXIT else '撞顶'}；证据不齐、没结论：{'；'.join(problems)}",
                              "timeout" if exit_code == TIME_LIMIT_HIT_EXIT else "memory-cap")
        append_row(os.path.join(self.library.home, ESCALATION_REGISTRY), ESCALATION_COLUMNS,
                   [self.current_key, sequence, item["name"], label, step_name, self.baseline_key_of(step_name), baseline_field, baseline_row_field,
                    self.arguments.threads, first_text, escalated_text, multiple,
                    first_log, file_digest(os.path.join(self.library.home, first_log)), escalated_log, file_digest(os.path.join(self.library.home, escalated_log)), today_text()])
        return Escalation(exit_code, seconds, label, f"{upgraded}，两趟都{kind_text}（基线 {baseline_field or '没记'}，要不低于它的 {multiple} 倍）："
                                                     f"证据登记在 {ESCALATION_REGISTRY}", "")

    def stop_for_infrastructure(self, step_name, sequence, exit_code):
        who = "nvidia-smi 看谁在用卡" if exit_code == GPU_MEMORY_HIT_EXIT else "bash scripts/hosts/run-with-memory-cap.sh --status 看谁占着"
        refuse(f"第 {sequence} 条的 {step_name} 退 {exit_code}（{WRAPPER_INFRASTRUCTURE_EXITS[exit_code]}）：这是内存包装自己的结局，不是这条变异的结局，不落行",
               f"看日志 {self.log_path(sequence, step_name)}（{who}），"
               "修好或等别的重活退了，重跑同一条命令接着跑", EXIT_INFRASTRUCTURE)

    def ensure_baseline(self, stage):
        """这一段的每一步在这把基线键、这个线程数下、没改过的树上都跑绿过；没有就现跑（不限时：基线就是用来标定超时的）。
        baselines.tsv 只追加，一行记基线键、步名、结局、秒数、时刻、线程数与这一步的内存峰值（文件头「基线」）。"""
        ran_every_step = True
        for step in self.steps:
            step_name, step_stage = step[0], step[1]
            if step_stage != stage:
                continue
            rows = self.baseline_rows_of(step_name)
            row = rows[-1][1] if rows else None
            measured = step_name not in RUNNER_TEST_STEPS or self.fake_driver or os.path.isfile(self.times_path(step_name))
            # 升档的证据要这一步在没改过的树上的内存峰值（文件头「升档」①）：真跑时这个线程数下没记过的重跑这一步的基线；假的一步不经包装、不记峰值
            peak_known = self.fake_driver or (row is not None and row["peak_bytes"].isdigit())
            if row is not None and row["outcome"] == "ok" and measured and peak_known:
                self.baseline_seconds[step_name] = float(row["seconds"])
                ran_every_step = False
                continue
            plan = self.judgement.baseline_entry_plan(step, self.arguments.threads) if step_name in self.baseline_refusals else None
            if plan is not None:
                self.ensure_entry_baseline(step, stage, plan)
                continue
            if step_name in self.baseline_refusals:
                say(f"  · 基线 {step_name}：基线键拒算（{self.baseline_refusals[step_name]}），在没改过的树上照跑、不沿用")
            else:
                say(f"  · 基线 {step_name}：在没改过的树上跑（这把基线键下还没有它的绿记录，或这个线程数下还没量过逐条的 CPU 时间、还没记过内存峰值）")
            runner_mode = "measure" if step_name in RUNNER_TEST_STEPS else None
            if runner_mode and os.path.exists(self.times_path(step_name)):
                os.remove(self.times_path(step_name))
            command = list(step[3]) + (["--no-fail-fast"] if step_name == "two-test-checker-tier" else [])
            if os.path.exists(self.baseline_peaks_path(step_name)):
                os.remove(self.baseline_peaks_path(step_name))     # 包装只记这一步这一趟的峰值，读完就不再用（不读上一趟留下的）
            exit_code, seconds = self.run_step(step, BASELINE_SEQUENCE, None, command=command, runner_mode=runner_mode)
            outcome = "ok" if exit_code == 0 else f"exit-{exit_code}"
            peak = self.wrapper_peak(step_name)
            append_row(self.library.baselines_path, BASELINE_COLUMNS,
                       [self.baseline_key_of(step_name), step_name, outcome, f"{seconds:.1f}", now_text(), self.arguments.threads, "" if peak is None else peak])
            if exit_code != 0:
                self.refuse_red_baseline(step_name, stage, exit_code)
            self.baseline_seconds[step_name] = seconds
            limit = self.time_limit_of(step_name)
            say(f"  · 基线 {step_name} 绿，{seconds:.1f} 秒；" + (f"这一步的超时定为 {limit} 秒" if limit is not None else
                                                              f"逐条按 CPU 时间限（{self.arguments.cpu_multiplier} 倍加 {self.arguments.cpu_floor} 秒，墙钟兜底 {self.arguments.wall_multiplier} 倍加 {self.arguments.wall_floor} 秒）"))
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
                   "已知的一条：crates/singlefs-checker-tier/src/crash_identity.rs 里读 .claude/gate.d/stage-inputs.tsv（导出树之外）的那条 lib 测试，"
                   "要实现员改成不读导出树之外的文件（或改门禁 checker-tier-crates-mutation-replay 登记的拷贝范围把登记表并进导出树），修好之后重跑同一条命令；"
                   f"在那之前只要第一段的结论就加 --skip-stage-two（第一段没红的记「只过了第一段」，不算定论）；全文看日志 {log_path}",
                   EXIT_BASELINE_RED)
        refuse(f"基线 {step_name}{part_text} 在没改过的树上退 {exit_code}：基线不绿，后面每一条「抓到」都不作数，一条变异都不跑",
               f"看日志 {log_path}；代码本来就红就先修代码再重跑，是环境（缺工具、内存包装退 250–254）就修环境，重跑同一条命令",
               EXIT_BASELINE_RED)

    def entry_baseline_rows(self, step_name):
        """baseline-entries.tsv 里这一步、这个线程数下的记录：{(记录键, kind, rest 的二进制或空): (runner 记的那几行, 那一行)}，同一把取最后一行；
        lines 那一段读不出的行不算（那一份照跑）。"""
        threads = str(self.arguments.threads)
        found = {}
        for fields in read_table(self.library.baseline_entries_path, len(BASELINE_ENTRY_COLUMNS)).rows:
            row = dict(zip(BASELINE_ENTRY_COLUMNS, fields))
            if row["step"] != step_name or (row["threads"] != threads and not break_is("baseline-any-threads")):
                continue
            try:
                lines = [[str(stem), str(test), float(cpu), float(wall)] for stem, test, cpu, wall in json.loads(row["lines"])]
            except (ValueError, TypeError):
                continue
            found[(row["record_key"], row["kind"], row["binary"] if row["kind"] == "rest" else "")] = (lines, row)
        return found

    def ensure_entry_baseline(self, step, stage, plan):
        """按条记基线（文件头「基线」）：基线键拒算的测试步，Judgement.baseline_entry_plan 拆出来的每份记录照（记录键、kind、线程数）在 baseline-entries.tsv
        里找最后一行，找得到的沿用它记的 CPU 秒，找不到的与拒算的那几条在没改过的树上照跑（nextest 按 -E 只点它们；checker 档按二进制起，
        有拒算的二进制另起 --exact --skip 掉它们的一趟与只跑它们的一趟），跑绿的照键追加一行。这一步的逐条时间表由沿用的与现跑的拼成；
        baselines.tsv 照常追加这一趟那一行（键是这一趟独有的那一把，峰值取现跑那几趟与沿用的记录里最大的），升档照它取。"""
        step_name = step[0]
        recorded = self.entry_baseline_rows(step_name)
        rules = prune_rules()

        def normalized(stem):
            return stem.replace("-", "_")

        def hit_of(record):
            return recorded.get((record["key"], record["kind"], record["binary"] if record["kind"] == "rest" else ""))

        reused, reused_peaks, runs, fresh_records = {}, [], [], []
        if plan["style"] == "nextest":
            tests, binaries = list(plan["refused"]), [item["binary"] for item in plan["whole"]]
            for record in plan["records"]:
                hit = hit_of(record)
                if hit is None:
                    fresh_records.append(record)
                    if record["kind"] == "test":
                        tests.append((record["binary"], record["test"]))
                    else:
                        binaries.append(record["binary"])
                    continue
                lines, row = hit
                reused_peaks.append(row["peak_bytes"])
                if record["kind"] == "test":
                    stem = lines[0][0] if lines and row["binary"] == record["binary"] else record["stem"]
                    reused[(normalized(stem), record["test"])] = [stem, record["test"], *(lines[0][2:4] if lines else [0.0, 0.0])]
                else:
                    reused.update({(normalized(line[0]), line[1]): line for line in lines})
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
                measured = fresh_records + [record for record in plan["records"] if record["binary"] in binaries and record not in fresh_records]
                runs.append({"text": f"{len(tests)} 条测试、{len(binaries)} 个整个照跑的二进制", "command": command,
                             "selection": {"tests": [list(test) for test in tests], "binaries": binaries}, "records": measured})
        else:
            base = [word for word in step[3] if word not in ("--lib", "--tests")] + ["--no-fail-fast"]
            whole = list(plan["whole"])
            for record in plan["records"]:
                hit = hit_of(record)
                exact = {"text": f"{record['binary']} 里拒算的 {len(record['refused'])} 条（--exact）", "binary": record["binary"], "stem": record["stem"],
                         "command": base + record["selector"] + ["--", "--exact", *record["refused"]],
                         "selection": {"binaries": [record["binary"]], "part": "exact"}, "records": []}
                if hit is not None:
                    lines, row = hit
                    reused_peaks.append(row["peak_bytes"])
                    stem = lines[0][0] if lines and row["binary"] == record["binary"] else record["stem"]
                    reused[(normalized(stem), "*")] = [stem, "*", *(lines[0][2:4] if lines else [0.0, 0.0])]
                    if record["refused"]:
                        runs.append(exact)
                    continue
                fresh_records.append(record)
                if record["refused"]:
                    runs.append({"text": f"{record['binary']} 去掉拒算的 {len(record['refused'])} 条（--exact --skip）", "binary": record["binary"], "stem": record["stem"],
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
        total_seconds, peaks, fresh = 0.0, [], {}
        sums = {}     # checker 档：按文件名把这一趟现跑的 CPU 秒、墙钟秒加起来（去掉拒算那一趟与只跑拒算那一趟各算一份，加起来是整个二进制的）
        entry_rows = []
        for run in runs:
            for path in (self.times_path(step_name), self.baseline_peaks_path(step_name)):
                if os.path.exists(path):
                    os.remove(path)
            os.makedirs(os.path.dirname(self.times_path(step_name)), exist_ok=True)
            selection_text = "all" if run["selection"].get("all") else "per-entry " + json.dumps(run["selection"], ensure_ascii=False, sort_keys=True)
            extra = {"BROAD_MUTATION_STEP_TESTS": json.dumps(run["selection"], ensure_ascii=False)} if self.fake_driver else None
            exit_code, seconds = self.run_step(step, BASELINE_SEQUENCE, None, command=run["command"], selection_text=selection_text, runner_mode="measure", extra=extra)
            total_seconds += seconds
            peak = self.wrapper_peak(step_name)
            if peak is not None:
                peaks.append(peak)
            if exit_code != 0:
                append_row(self.library.baselines_path, BASELINE_COLUMNS,
                           [self.baseline_key_of(step_name), step_name, f"exit-{exit_code}", f"{total_seconds:.1f}", now_text(), self.arguments.threads,
                            "" if not peaks else max(peaks)])
                self.refuse_red_baseline(step_name, stage, exit_code, f"（按条记基线照跑的那一部分：{run['text']}）")
            lines = runner_time_lines(self.times_path(step_name))
            if plan["style"] == "nextest":
                fresh.update({(normalized(line[0]), line[1]): line[:4] for line in lines})
                for record in run["records"]:
                    if record["kind"] == "test":
                        line = fresh.get((normalized(record["stem"]), record["test"]))
                        if line is not None:
                            entry_rows.append((record, [line], peak))
                    else:
                        names = set(record["names"])
                        entry_rows.append((record, [line[:4] for line in lines if normalized(line[0]) == normalized(record["stem"]) and line[1] not in names], peak))
                continue
            by_stem = {}
            for line in lines:
                total = by_stem.setdefault(normalized(line[0]), [line[0], "*", 0.0, 0.0])
                total[2] += line[2]
                total[3] += line[3]
            for key, total in by_stem.items():
                running = sums.setdefault((key, "*"), [total[0], "*", 0.0, 0.0])
                running[2] += total[2]
                running[3] += total[3]
            for record in run["records"]:
                total = by_stem.get(normalized(record["stem"]))
                if total is not None:
                    entry_rows.append((record, [total], peak))
        if plan["style"] != "nextest":
            for key, total in sums.items():
                if key in reused:
                    total = [total[0], "*", total[2] + reused[key][2], total[3] + reused[key][3]]
                fresh[key] = total
        assembled = dict(reused)
        assembled.update(fresh)
        with open(self.times_path(step_name), "w", encoding="utf-8") as handle:
            handle.write("".join(f"{stem}\t{test}\t{cpu:.3f}\t{wall:.3f}\t0\n" for stem, test, cpu, wall in assembled.values()))
        if entry_rows:
            append_rows(self.library.baseline_entries_path, BASELINE_ENTRY_COLUMNS,
                        [[record["key"], step_name, self.arguments.threads, record["kind"], record["binary"], json.dumps(lines, ensure_ascii=False),
                          "" if peak is None else peak, now_text()] for record, lines, peak in entry_rows])
        unmeasured = len(fresh_records) - len({id(record) for record, _lines, _peak in entry_rows} & {id(record) for record in fresh_records})
        peak_values = peaks + [int(value) for value in reused_peaks if str(value).isdigit()]
        append_row(self.library.baselines_path, BASELINE_COLUMNS,
                   [self.baseline_key_of(step_name), step_name, "ok", f"{total_seconds:.1f}", now_text(), self.arguments.threads, max(peak_values) if peak_values else ""])
        self.baseline_seconds[step_name] = total_seconds
        say(f"  · 基线 {step_name} 绿（按条记基线：照跑 {len(runs)} 趟共 {total_seconds:.1f} 秒，记下 {len(entry_rows)} 份"
            + (f"，照跑而 runner 没量到的 {unmeasured} 份下一趟照跑" if unmeasured else "") + f"）；逐条按 CPU 时间限（{self.arguments.cpu_multiplier} 倍加 "
            f"{self.arguments.cpu_floor} 秒，墙钟兜底 {self.arguments.wall_multiplier} 倍加 {self.arguments.wall_floor} 秒）")

    def verify_on_pristine_tree(self, step_names, why, suspects=()):
        """在没改过的树上把这几步各跑一遍（调用时树必须是原样）：退非 0 就是环境坏了，退 6。suspects 是因此不可信的那几条的序号。"""
        for step_name in step_names:
            step = self.step_named(step_name)
            exit_code, seconds = self.run_step(step, ENVIRONMENT_SEQUENCE, self.time_limit_of(step_name))
            if exit_code == 0:
                os.remove(self.log_path(ENVIRONMENT_SEQUENCE, step_name))
                continue
            rerun = ""
            if suspects:
                rerun = (f"；这 {len(suspects)} 条（第 {'、'.join(str(sequence) for sequence in suspects)} 条）的行不可信，环境修好之后点名重跑："
                         f"{heavy_prefix_text()} python3 {SELF_PATH} " + " ".join(f"--item {sequence}" for sequence in suspects) + " --rerun")
            refuse(f"疑似环境坏了（{why}）：{step_name} 在没改过的树上退 {exit_code}（{seconds:.1f} 秒），基线绿过的这一步现在过不了",
                   f"看日志 {self.log_path(ENVIRONMENT_SEQUENCE, step_name)}：盘满、编译目录被清、工具链换了、scratch 写不进这一类；修好之后重跑同一条命令{rerun}",
                   EXIT_BASELINE_RED)

    def verify_environment_at_start(self, stages):
        """每次重起：这一趟没现跑基线的那几段，先在没改过的树上把编译那几步各跑一遍。"""
        if break_is("no-environment-check"):
            return
        step_names = [step[0] for step in self.steps if step[2] == "build" and step[1] in stages and step[1] not in self.stages_with_fresh_baseline]
        if step_names:
            self.verify_on_pristine_tree(step_names, "重起先验编译")
            say(f"  · 重起先验编译：{'、'.join(step_names)} 在没改过的树上照过")


def runner_time_lines(path):
    """runner 在 measure 模式写的表（mutation-prune.py 的 RUNNER_SCRIPT：文件名、用例、CPU 秒、墙钟秒、退出状态）逐行交 [文件名, 用例, CPU 秒, 墙钟秒, 退出状态]，
    按写的次序、不合并（按条记基线照条记，Runner.ensure_entry_baseline）；表不在交空表，读不出的行不算。"""
    found = []
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            for line in handle:
                fields = line.rstrip("\n").split("\t")
                try:
                    found.append([fields[0], fields[1], float(fields[2]), float(fields[3]), fields[4] if len(fields) > 4 else ""])
                except (IndexError, ValueError):
                    continue
    except OSError:
        return found
    return found


def libtest_outcome(line):
    """libtest 结果行的结局（ok / FAILED / ignored），不是结果行交 None（解析在 lib_libtest_result_lines.py）。"""
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


def parse_shard(text):
    found = re.fullmatch(r"(\d+)/(\d+)", text or "")
    if not found or int(found.group(2)) < 1 or int(found.group(1)) >= int(found.group(2)):
        refuse(f"--shard 写成了「{text}」", "写成 <k>/<n>，k 取 0..n-1，例：两台机器一台 --shard 0/2、一台 --shard 1/2")
    return int(found.group(1)), int(found.group(2))


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


def host_triple(library):
    if test_mode():
        return "x86_64-unknown-linux-gnu"
    return next((line.split(":", 1)[1].strip() for line in toolchain_text(library.tree).splitlines() if line.startswith("host:")), "x86_64-unknown-linux-gnu")


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
    escalation_notes = []    # 升档那趟跑完、接着往后判的：它那一句跟进这一条最后落的那一行的备注
    method = "full" if runner.full else "fresh"

    def record(outcome, stage, test_name="", note_text="", step_name="", step_seconds=0.0, environment_doubt=""):
        if outcome == "missed" and not ran_any_test:
            note_text = (note_text + "；" if note_text else "") + "零覆盖：没有一条测试执行到它（C2 记没红）"
        note_text = "；".join(text for text in (note_text, *escalation_notes) if text)
        refused = list(current["refused"])
        test_key = coverage_key = ""
        if outcome == "caught" and test_name not in ESCALATED_LABELS:
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
        if outcome == "missed" or (outcome == "caught" and test_name in ESCALATED_LABELS):
            coverage_key = judgement.coverers_for(current["unit"])[0] or ""
        fields = [current["key"] or "", sequence, item["package"], item["name"], outcome, stage, test_name, test_key, coverage_key,
                  f"{totals['build']:.1f}", f"{totals['test']:.1f}", today_text(), method, current["locator"], ",".join(refused), note_text]
        library.write_composition(current["key"], current["composition"])
        library.record_verdict(fields)
        return ItemRow(fields, step_name, step_seconds, environment_doubt)

    def limit_note(log_path):
        try:
            with open(log_path, encoding="utf-8", errors="replace") as handle:
                text = handle.read()
        except OSError:
            return ""
        return "；CPU 限时停的（卡住算抓到）" if "BROAD_MUTATION_LIMIT kind=cpu" in text else "；墙钟兜底停的（卡住算抓到）" if "BROAD_MUTATION_LIMIT kind=wall" in text else ""

    def run_stage(stage):
        """跑这一段的每一步：交 None（全绿）或已经落下的那一行。"""
        for step in runner.steps:
            step_name, step_stage, kind, _command, test_failed_exit = step
            if step_stage != stage:
                continue
            command, selection_text = runner.command_for(step, selection)
            if command is None:
                continue
            if step_name in RUNNER_TEST_STEPS:
                ran_any_test.append(step_name)
            runner_mode = "limit" if step_name in RUNNER_TEST_STEPS else None
            exit_code, seconds = runner.run_step(step, sequence, runner.time_limit_of(step_name), command=command, selection_text=selection_text,
                                                 runner_mode=runner_mode)
            totals[kind] += seconds
            if exit_code in (TIME_LIMIT_HIT_EXIT, MEMORY_CAP_HIT_EXIT) and not break_is("escalation-skipped"):
                # 文件头「升档」：当场升档重跑那一步一次；两趟都超时 / 撞顶而证据齐的判抓到，证据不齐的照环境型结局记，升档那趟另有结局的往下照常判
                escalation = runner.escalate_step(item, step, command, selection_text, runner_mode, exit_code)
                totals[kind] += escalation.seconds
                if escalation.label:
                    return record("caught", stage, escalation.label, note_text=escalation.note, step_name=step_name, step_seconds=escalation.seconds)
                if escalation.environmental:
                    return record(escalation.environmental, stage, note_text=escalation.note, step_name=step_name, step_seconds=escalation.seconds)
                escalation_notes.append(escalation.note)
                if escalation.seconds:     # 0 是没升档（这一步没有整步限时、或上限写法认不出）：照原来那一趟往下判
                    exit_code, seconds = escalation.exit_code, escalation.seconds
                    if exit_code == 0:
                        say(f"  · 第 {sequence} 条：{escalation.note}，升档那趟过了，接着往后判")
            log_path = runner.log_path(sequence, step_name)
            if exit_code == 0:
                os.remove(log_path)
                continue
            if exit_code == TIME_LIMIT_HIT_EXIT:
                return record("timeout", stage, note_text=f"{step_name} 超过 {runner.time_limit_of(step_name)} 秒", step_name=step_name, step_seconds=seconds)
            if exit_code == MEMORY_CAP_HIT_EXIT:
                return record("memory-cap", stage, note_text=f"{step_name} 撞了内存上限 {runner.arguments.memory_cap}", step_name=step_name, step_seconds=seconds)
            if kind == "build" and exit_code == CARGO_FAILED_EXIT:
                if log_shows_compile_error(log_path) or break_is("unviable-by-exit-code"):
                    return record("unviable", stage, note_text=f"{step_name} 编不过", step_name=step_name, step_seconds=seconds)
                return record(DRIVER_ERROR, stage, note_text=f"{step_name} 退 {exit_code} 而日志里没有带源码位置的编译错误：不算无效", step_name=step_name,
                              step_seconds=seconds, environment_doubt=f"第 {sequence} 条的 {step_name} 退 {exit_code} 而没有编译错误")
            if kind == "test" and exit_code == test_failed_exit:
                package_of_step = CHECKER_TIER_PACKAGE if "checker-tier" in step_name else ""
                return record("caught", stage, first_failing_test(log_path, test_failed_exit, package_of_step), note_text=step_name + limit_note(log_path),
                              step_name=step_name, step_seconds=seconds)
            if exit_code == GPU_MEMORY_HIT_EXIT and break_is("gpu-memory-as-driver-error"):
                return record(DRIVER_ERROR, stage, note_text=f"{step_name} 退 {exit_code}（认不出的退出码，不是这条变异的结局；日志 {os.path.basename(log_path)}）",
                              step_name=step_name, step_seconds=seconds)
            if exit_code in WRAPPER_INFRASTRUCTURE_EXITS or break_is("unknown-exit-stops"):
                if exit_code not in WRAPPER_INFRASTRUCTURE_EXITS:
                    refuse(f"第 {sequence} 条的 {step_name} 退 {exit_code}（认不出的退出码）：不落行", f"看日志 {log_path}", EXIT_INFRASTRUCTURE)
                runner.stop_for_infrastructure(step_name, sequence, exit_code)
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


# ── GPU 那一趟（C489：第二段补一趟带 gpu 特性的 checker 档快档；文件头「GPU 那一趟」） ───────────────────────────────────────
# 这一节的函数都不在 JUDGING_ENTRIES 的闭包里：CPU 那一趟的判法单元指纹不因它们变，库里 CPU 那一趟的判定照沿用；
# GPU 那一趟自己的判法单元指纹从 GPU_JUDGING_ENTRIES 收（gpu_judging_key），进 GPU 判定行的 gpu_key。

GPU_VERDICTS = "gpu-verdicts.tsv"
GPU_VERDICT_COLUMNS = ("key", "sequence", "package", "name", "outcome", "caught_by", "test_key", "coverage_key", "gpu_key", "test_seconds",
                       "judged_on", "method", "note", "mac")
GPU_SIGNED_COLUMNS = len(GPU_VERDICT_COLUMNS) - 1       # mac 一段签的是它前面那十三段
GPU_ROW_OUTCOMES = (*FINAL_OUTCOMES, DRIVER_ERROR)
GPU_FEATURE = "gpu"
GPU_TESTS_DIRECTORY = "crates/singlefs-checker-tier/tests"
GPU_BUILD_STEP, GPU_TEST_STEP = "gpu-build-checker-tier", "gpu-test-checker-tier"
GPU_BASELINE_SEQUENCE = "gpu-baseline"
GPU_NO_ADAPTER_MARK = "NOT_RUN_NO_GPU_ADAPTER"      # GPU 对拍找不到 GPU 适配器时测试打的那一行（门禁 checker-tier-crash-point-replay 的 feature-gated-tests 格读同一个词）
GPU_QUIESCE_REFUSED_EXIT = 78                        # scripts/crash/gpu-quiesce-run.sh 清场回读没过、命令没起时退的码
GPU_NOT_RUN = "not-run"                              # 这一趟没跑成：不落行、不算没红
GPU_INFRASTRUCTURE = "infrastructure"                # 内存包装自己的结局：不落行，整趟停（退 3）
QUIESCE_RUNNER = os.path.join(SCRIPT_DIRECTORY, "..", "crash", "gpu-quiesce-run.sh")
BINDGEN_LIBRARY = os.path.join(SCRIPT_DIRECTORY, "..", "crash", "crash-amplification-lib.sh")
GPU_ATTRIBUTE_START = re.compile(r"#(!?)\[\s*cfg\s*\(")
GPU_FEATURE_NAME = re.compile(r"\bfeature\s*=\s*\"([^\"]+)\"")
GPU_PASSED_COUNT = re.compile(r"^test result: \w+\. (\d+) passed", re.M)
# GPU 那一趟自己的判法单元指纹（文件头「GPU 那一趟」）：从这几个函数收 python 单元闭包（命令怎么拼、步结局怎么分、沿用怎么判、两趟怎么合）
GPU_JUDGING_ENTRIES = (("scripts/mutation/broad-mutation.py", "gpu_pass_command"), ("scripts/mutation/broad-mutation.py", "gpu_classify_build"),
                       ("scripts/mutation/broad-mutation.py", "gpu_classify_test"), ("scripts/mutation/broad-mutation.py", "gpu_combined_state"),
                       ("scripts/mutation/broad-mutation.py", "GpuJudging.need"), ("scripts/mutation/broad-mutation.py", "GpuJudging.row_holds"),
                       ("scripts/mutation/broad-mutation.py", "GpuJudging.test_key_of"))
GPU_JUDGING_KEYS = {}


class GpuOutcome(NamedTuple):
    outcome: str       # missed / caught / timeout / memory-cap / driver-error / not-run / infrastructure
    caught_by: str     # 抓到它的测试（「<二进制 id> <用例名>」），别的结局是空
    note: str


class GpuTotals(NamedTuple):
    needed: int        # 要这一趟的条数（CPU 那一趟没红、又有带特性的测试够得着它）
    judged: int        # 这一趟现判了几条
    carried: int       # 键相同、沿用得了的几条
    not_run: int       # 本次未跑几条（没卡、清场没过、--skip-gpu）
    failed: bool       # 复用哨兵判红
    outcomes: dict     # 现判的各结局条数


# gate-overlap:copy-kept checker-tier-crash-point-replay.sh 的 list_feature_gated_test_files 是嵌在 bash 里的 python，读不进来；这里同一套判法照搬一份，两边都认 tests/ 里的 cfg 点到的特性
def gated_test_files(tests_directory):
    """checker 档 tests/ 顶层每个 .rs 里 #[cfg(…)] / #![cfg(…)] 点到的特性：[(文件名, whole-file 或 items, [特性])]，照文件名排序；挂特性的一个都没有交空表。
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
    """GPU 那一趟一步的命令（verification.md「checker 档自己分快档与全量」带特性那一趟）：编译步 cargo test --release -p singlefs-checker-tier --features <特性> <目标> --no-run；
    测试步在它后面加 `-- --nocapture --test-threads 1`（GPU 对拍的测试各占卡的显存，不并行起），并整条经 scripts/crash/gpu-quiesce-run.sh 清场再起（跑完不复原）。
    labels 是要跑的目标：lib 是 --lib，别的是 --test <文件名>。弄坏开关 gpu-quiesce-dropped 不经清场、gpu-test-threads-dropped 不加 --test-threads 1。"""
    targets = []
    for label in sorted(labels, key=lambda text: (text != "lib", text)):
        targets += ["--lib"] if label == "lib" else ["--test", label]
    head = ["cargo", "test", "--locked", "--release", "-p", CHECKER_TIER_PACKAGE, "--features", ",".join(features), *targets]
    if kind == "build":
        return head + ["--no-run"]
    quiesce = [] if break_is("gpu-quiesce-dropped") else ["bash", QUIESCE_RUNNER, "--"]
    threads = [] if break_is("gpu-test-threads-dropped") else ["--test-threads", "1"]
    return quiesce + head + ["--", "--nocapture", *threads]


def gpu_log_text(log_path):
    try:
        with open(log_path, encoding="utf-8", errors="replace") as handle:
            return handle.read()
    except OSError:
        return ""


def gpu_failing_test(log_path):
    """GPU 那一趟第一条没过的测试：先照 libtest 结果行认（first_failing_test）；--nocapture 加 --test-threads 1 下结果行常被测试自己的输出打散，认不出就读收尾的
    「failures:」名单（缩进四格的测试名），二进制照前面最近的 Running 行补。认不出交 UNRECOGNISED_TEST。"""
    name = first_failing_test(log_path, CARGO_FAILED_EXIT, CHECKER_TIER_PACKAGE)
    if name != UNRECOGNISED_TEST:
        return name
    binary, listing = "", False
    for line in gpu_log_text(log_path).splitlines():
        binary = libtest_binary(line, binary, CHECKER_TIER_PACKAGE)
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
    """GPU 那一趟编译步的结局：→ (None 编得过 | missed 不会有 | unviable / timeout / memory-cap / driver-error / infrastructure, 备注)。
    编译那一步退 101 而日志里没有带源码位置的编译错误（磁盘满、锁文件不符这一类）不记无效，记驱动错。"""
    if exit_code == 0:
        return None, ""
    if exit_code == TIME_LIMIT_HIT_EXIT:
        return "timeout", f"{GPU_BUILD_STEP} 超时"
    if exit_code == MEMORY_CAP_HIT_EXIT:
        return "memory-cap", f"{GPU_BUILD_STEP} 撞了内存上限"
    if exit_code in WRAPPER_INFRASTRUCTURE_EXITS:
        return GPU_INFRASTRUCTURE, f"{GPU_BUILD_STEP} 退 {exit_code}（{WRAPPER_INFRASTRUCTURE_EXITS[exit_code]}）"
    if exit_code == CARGO_FAILED_EXIT:
        if log_shows_compile_error(log_path):
            return "unviable", f"{GPU_BUILD_STEP} 编不过"
        return DRIVER_ERROR, f"{GPU_BUILD_STEP} 退 {exit_code} 而日志里没有带源码位置的编译错误：不算无效"
    return DRIVER_ERROR, f"{GPU_BUILD_STEP} 退 {exit_code}（认不出的退出码，不是这条变异的结局）"


def gpu_classify_test(exit_code, log_path):
    """GPU 那一趟测试步的结局（GpuOutcome）：退 0 且跑到了测试、日志里没有 NOT_RUN_NO_GPU_ADAPTER 才是没红（missed）；
    日志里有 NOT_RUN_NO_GPU_ADAPTER（GPU 测试没找到卡，打一行就返回成功）或清场包装退 78 是本次未跑（not-run，不落行、不算没红）；
    退 0 而一个测试都没跑到不算没红，记驱动错；退 101 是有测试没过 = 抓到，带第一条没过的测试。"""
    text = gpu_log_text(log_path)
    passed = sum(int(count) for count in GPU_PASSED_COUNT.findall(text))
    if exit_code == 0:
        if GPU_NO_ADAPTER_MARK in text and not break_is("gpu-not-run-as-missed"):
            return GpuOutcome(GPU_NOT_RUN, "", f"no-adapter：日志里有 {GPU_NO_ADAPTER_MARK}，GPU 测试没找到卡、没跑")
        if passed == 0:
            return GpuOutcome(DRIVER_ERROR, "", "GPU 趟一个测试都没跑到（日志里没有 passed 的计数）：不算没红")
        return GpuOutcome("missed", "", f"{passed} 条测试都过了")
    if exit_code == GPU_QUIESCE_REFUSED_EXIT:
        return GpuOutcome(GPU_NOT_RUN, "", "quiesce-refused：清场回读没过（卡上有不归清场白名单的进程，或多机配置里没写全清场命令），测试没起")
    if exit_code == TIME_LIMIT_HIT_EXIT:
        return GpuOutcome("timeout", "", f"{GPU_TEST_STEP} 超时")
    if exit_code == MEMORY_CAP_HIT_EXIT:
        return GpuOutcome("memory-cap", "", f"{GPU_TEST_STEP} 撞了内存上限")
    if exit_code in WRAPPER_INFRASTRUCTURE_EXITS:
        return GpuOutcome(GPU_INFRASTRUCTURE, "", f"{GPU_TEST_STEP} 退 {exit_code}（{WRAPPER_INFRASTRUCTURE_EXITS[exit_code]}）")
    if exit_code == CARGO_FAILED_EXIT:
        return GpuOutcome("caught", gpu_failing_test(log_path), GPU_TEST_STEP)
    return GpuOutcome(DRIVER_ERROR, "", f"{GPU_TEST_STEP} 退 {exit_code}（认不出的退出码，不是这条变异的结局）")


def gpu_combined_state(cpu, needed, gpu):
    """两趟合起来的状态（ItemState）与没结论时的理由：CPU 那一趟没红、又要 GPU 那一趟的条，看 GPU 那一趟的状态——抓到 / 没红 / 无效就是定论，
    超时、内存撞顶、驱动错照样是没结论，库里还没有它的结论是 none（理由 gpu-not-run，不算没红）；别的条就是 CPU 那一趟的状态。
    一条两趟里任一趟红算抓到：CPU 那一趟抓到的，GPU 那一趟不用跑。弄坏开关 gpu-red-not-caught：GPU 那一趟红不算抓到；gpu-not-run-as-missed：本次未跑记成没红。"""
    if cpu.state != "missed" or not needed:
        return cpu, ""
    if break_is("gpu-red-not-caught") and gpu.state == "caught":
        return ItemState("missed", cpu.passed_stage_one), ""
    if gpu.state in DECISIVE_OUTCOMES:
        return ItemState(gpu.state, cpu.passed_stage_one), ""
    if gpu.state in (*ENVIRONMENTAL_OUTCOMES, DRIVER_ERROR):
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


# ── GPU 判定库：gpu-verdicts.tsv（只追加，每行带同一把密钥算的校验列） ─────────────────────────────────────────

def gpu_verdicts_path(library):
    return os.path.join(library.home, GPU_VERDICTS)


def gpu_result_mac(key, fields):
    """GPU 判定行的校验列：HMAC-SHA256(密钥, 前十三段照写进表里的样子用制表符接起来、UTF-8)，小写十六进制（与 result_mac 同一把密钥、同一种算法，只是段数不同）。"""
    text = "\t".join(clean_field(field) for field in fields[:GPU_SIGNED_COLUMNS])
    return hmac.new(key, text.encode("utf-8", "surrogateescape"), hashlib.sha256).hexdigest()


def read_gpu_verdicts(library):
    """GPU 判定库：全列且校验列对得上的行才算数；少一段的、校验列对不上的、没有密钥时的每一行都不算、列进 forged（弄坏开关 gpu-mac-unchecked 下都算）。
    → SignedTableRead（同 read_signed_table 的形状）。"""
    path = gpu_verdicts_path(library)
    key, key_problem = library.result_key()
    rows, forged, malformed_lines, partial_tail = [], [], [], ""
    if not os.path.exists(path):
        return SignedTableRead(rows, forged, malformed_lines, partial_tail)
    with open(path, "rb") as handle:
        lines = handle.read().decode("utf-8", errors="surrogateescape").split("\n")
    if lines[-1] != "":
        partial_tail = lines[-1]
    unchecked = break_is("gpu-mac-unchecked")
    for number, line in enumerate(lines[:-1], start=1):
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) == GPU_SIGNED_COLUMNS and not unchecked:
            forged.append((number, fields[0][:16], "没有校验列（少一段）"))
            continue
        if len(fields) not in (GPU_SIGNED_COLUMNS, len(GPU_VERDICT_COLUMNS)):
            malformed_lines.append(number)
            continue
        if not unchecked:
            if key is None:
                forged.append((number, fields[0][:16], key_problem or "没有密钥，校验不了"))
                continue
            if not hmac.compare_digest(gpu_result_mac(key, fields), fields[-1]):
                forged.append((number, fields[0][:16], "校验列对不上"))
                continue
        rows.append(dict(zip(GPU_VERDICT_COLUMNS, fields[:GPU_SIGNED_COLUMNS])))
    return SignedTableRead(rows, forged, malformed_lines, partial_tail)


def record_gpu_verdict(library, fields):
    """落一行 GPU 判定：十三段加上校验列，fsync。"""
    key = library.result_key(create=True)[0]
    cleaned = [clean_field(field) for field in fields[:GPU_SIGNED_COLUMNS]]
    append_row(gpu_verdicts_path(library), GPU_VERDICT_COLUMNS, cleaned + [gpu_result_mac(key, cleaned)])


def record_gpu_verdicts(library, rows):
    """一次落几行 GPU 判定（导入的行），各带本机密钥算的校验列，fsync 一次。"""
    key = library.result_key(create=True)[0]
    cleaned = [[clean_field(field) for field in fields[:GPU_SIGNED_COLUMNS]] for fields in rows]
    append_rows(gpu_verdicts_path(library), GPU_VERDICT_COLUMNS, [fields + [gpu_result_mac(key, fields)] for fields in cleaned])


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
        self.feature_arguments = [f"{CHECKER_TIER_PACKAGE}/{feature}" for feature in self.features]
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
        """这一趟跑得到的测试（静态列出，不含 #[ignore] 的）：checker 档包的 lib 与挂特性的测试文件里的测试，带上要跑的目标（lib / 文件名）。"""
        if self.entry_list is None:
            found = []
            for entry in self.rules.test_entries(self.judgement.tree, (CHECKER_TIER_PACKAGE,), self.judgement.exclusions):
                if entry["ignored"]:
                    continue
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
        超时、内存撞顶、驱动错不是结论，认下来只为定状态；抓到它的测试认不出的，真跑每趟现判、只读时看覆盖者键。"""
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
    """GPU 那一趟的步（文件头「GPU 那一趟」）：编译步与测试步各自经 Runner.run_step 起（内存包装在最外层，清场包装在测试步命令里面），不设 runner 模式。
    基线 = 在没改过的树上把这一趟要跑的目标跑绿一遍（编译步、测试步各一次），秒数定每一步的限时（--timeout-floor、--timeout-multiplier）；这一趟的基线不记库，每趟现跑。"""

    def __init__(self, runner, judging):
        self.runner, self.judging = runner, judging
        self.arguments, self.library = runner.arguments, runner.library
        self.bindgen = gpu_bindgen_arguments()
        self.baseline_seconds = {}
        self.build_step = (GPU_BUILD_STEP, "gpu", "build", [], None)
        self.test_step = (GPU_TEST_STEP, "gpu", "test", [], None)

    def run(self, step, sequence, labels, limit):
        command = gpu_pass_command("build" if step is self.build_step else "test", self.judging.features, labels)
        extra = {"BINDGEN_EXTRA_CLANG_ARGS": self.bindgen} if self.bindgen else {}
        if self.runner.fake_driver:
            extra["BROAD_MUTATION_STEP_COMMAND"] = json.dumps(command)
        return self.runner.run_step(step, sequence, limit, command=command, extra=extra)

    def limit(self, step_name):
        return int(max(self.arguments.timeout_floor, self.arguments.timeout_multiplier * self.baseline_seconds.get(step_name, 0)))

    def ensure_baseline(self, labels):
        """→ (跑得了, 为什么跑不了)。没改过的树上编译红 / 测试红 / 一个测试都没跑到：拒（退 6，基线不绿后面每一条抓到都不作数）；
        内存包装自己的结局：拒（退 3）；清场没过或没找到卡：这一趟本次未跑，不拒。"""
        sequence = GPU_BASELINE_SEQUENCE
        exit_code, seconds = self.run(self.build_step, sequence, labels, None)
        build_log = self.runner.log_path(sequence, GPU_BUILD_STEP)
        outcome, note_text = gpu_classify_build(exit_code, build_log)
        if outcome == GPU_INFRASTRUCTURE:
            self.runner.stop_for_infrastructure(GPU_BUILD_STEP, sequence, exit_code)
        if outcome is not None:
            refuse(f"GPU 趟基线 {GPU_BUILD_STEP} 在没改过的树上退 {exit_code}（{note_text}）：基线不绿，后面每一条 GPU 趟的抓到都不作数，一条都不跑",
                   f"看日志 {build_log}；代码本来就编不过就先修代码（带 --features {','.join(self.judging.features)} 的 checker 档），是环境（盘满、缺 libclang、BINDGEN_EXTRA_CLANG_ARGS）就修环境，重跑同一条命令；"
                   "这一趟先不要 GPU 趟就加 --skip-gpu", EXIT_BASELINE_RED)
        self.baseline_seconds[GPU_BUILD_STEP] = seconds
        os.remove(build_log)
        exit_code, seconds = self.run(self.test_step, sequence, labels, None)
        test_log = self.runner.log_path(sequence, GPU_TEST_STEP)
        result = gpu_classify_test(exit_code, test_log)
        if result.outcome == GPU_NOT_RUN:
            return False, result.note
        if result.outcome == GPU_INFRASTRUCTURE:
            self.runner.stop_for_infrastructure(GPU_TEST_STEP, sequence, exit_code)
        if result.outcome != "missed":
            refuse(f"GPU 趟基线 {GPU_TEST_STEP} 在没改过的树上不绿（{'有测试没过：' + result.caught_by if result.outcome == 'caught' else result.note}）：基线不绿，后面每一条 GPU 趟的抓到都不作数，一条都不跑",
                   f"看日志 {test_log}；代码本来就红就先修代码，是卡的毛病（显存、别的会话占着）就等卡空了重跑同一条命令；这一趟先不要 GPU 趟就加 --skip-gpu",
                   EXIT_BASELINE_RED)
        self.baseline_seconds[GPU_TEST_STEP] = seconds
        os.remove(test_log)
        say(f"  · GPU 趟基线绿：{GPU_BUILD_STEP} {self.baseline_seconds[GPU_BUILD_STEP]:.1f} 秒、{GPU_TEST_STEP} {seconds:.1f} 秒（目标 {','.join(labels)}）；"
            f"每一步的限时定为 {self.limit(GPU_BUILD_STEP)} / {self.limit(GPU_TEST_STEP)} 秒")
        return True, ""


def plan_gpu_sentinel(candidates, arguments):
    """GPU 那一趟的复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「二」）：候选是这一趟罩得住、沿用判据此刻认定论的 GPU 判定，项名「gpu:<序号>」。
    --full、--rerun 不读库不抽。弄坏开关 gpu-carry-ignored（沿用得了的照样重跑）下这一趟判红「复用失灵」。"""
    sentinel = reuse_checks.ReuseSentinel(SCRIPT_KEY, "广谱变异 GPU 趟")
    off = "--full（不读判定库）" if arguments.full else "--rerun（点名的照样重跑）" if arguments.rerun else ""
    sentinel.plan({f"gpu:{sequence}": outcome for sequence, outcome in candidates.items()}, reuse=not off, off_reason=off, printer=say)
    return sentinel


def judge_gpu_item(library, gpu_pass, item, need):
    """判一条的 GPU 那一趟：先编（编不过记无效）再跑测试步，落一行 GPU 判定（本次未跑的不落行）。→ (GpuOutcome, 编译秒, 测试秒)。
    树里的原件无论怎么退都拷回；被 TERM 停（Stopped）不落行。"""
    runner, judging = gpu_pass.runner, gpu_pass.judging
    sequence, current = item["sequence"], judging.judgement.key_of(item)
    labels, method = need["labels"], "full" if gpu_pass.arguments.full else "fresh"
    seconds = {"build": 0.0, "test": 0.0}

    def record(outcome, caught_by="", test_key="", coverage_key="", note_text=""):
        record_gpu_verdict(library, [current["key"], sequence, item["package"], item["name"], outcome, caught_by, test_key, coverage_key, judging.pass_key(),
                                     f"{seconds['test']:.1f}", today_text(), method, note_text])

    try:
        apply_mutation(library, item)
        exit_code, took = gpu_pass.run(gpu_pass.build_step, sequence, labels, gpu_pass.limit(GPU_BUILD_STEP))
        seconds["build"] += took
        build_log = runner.log_path(sequence, GPU_BUILD_STEP)
        outcome, note_text = gpu_classify_build(exit_code, build_log)
        if outcome == GPU_INFRASTRUCTURE:
            runner.stop_for_infrastructure(GPU_BUILD_STEP, sequence, exit_code)
        if outcome is not None:
            record(outcome, note_text=note_text)
            return GpuOutcome(outcome, "", note_text), seconds["build"], seconds["test"]
        os.remove(build_log)
        exit_code, took = gpu_pass.run(gpu_pass.test_step, sequence, labels, gpu_pass.limit(GPU_TEST_STEP))
        seconds["test"] += took
        test_log = runner.log_path(sequence, GPU_TEST_STEP)
        result = gpu_classify_test(exit_code, test_log)
        if result.outcome == GPU_INFRASTRUCTURE:
            runner.stop_for_infrastructure(GPU_TEST_STEP, sequence, exit_code)
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
    要这一趟的条 = 这一趟罩得住、CPU 那一趟最后落的结论是没红、又有带特性的测试够得着的；键相同、沿用得了的复用；沿用不了的现判（--full、--rerun 不读库）。
    开跑先在没改过的树上跑一遍基线；没找到卡、清场没过、带了 --skip-gpu：要现判的条逐条打 BROAD_MUTATION_GPU_NOT_RUN（本次未跑，不落行、不算没红）。"""
    outcomes = dict.fromkeys(GPU_ROW_OUTCOMES, 0)

    def totals(needed=0, judged=0, carried=0, not_run=0, failed=False, report=True):
        if report:
            say(f"BROAD_MUTATION_GPU_TOTAL needed={needed} judged={judged} carried={carried} not_run={not_run} "
                + " ".join(f"{SUMMARY_KEYS[name]}={count}" for name, count in outcomes.items()))
        return GpuTotals(needed, judged, carried, not_run, failed, outcomes)

    if arguments.skip_stage_two:
        say("  · GPU 趟接在第二段后面，这一趟带了 --skip-stage-two，不跑")
        return totals(report=False)
    judgement = runner.judgement
    refresh_verdict_rows(judgement)      # CPU 那一趟刚落的行要读进来（单元树、每条的键、测试的闭包照旧用这一趟已经算好的：树里的源码这一趟没变）
    judging = GpuJudging(library, judgement, arguments)
    if not judging.enabled:
        say(f"  · GPU 趟：{GPU_TESTS_DIRECTORY} 下没有文件的 cfg 点到 gpu 特性，没有带 gpu 特性的快档可跑")
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
            candidates[item["sequence"]] = gpu.state     # 沿用判据此刻认定论的：复用哨兵的候选（弄坏开关 gpu-carry-ignored 下它们被现判，哨兵就判红）
        if gpu.state in DECISIVE_OUTCOMES and not arguments.full and not arguments.rerun and not break_is("gpu-carry-ignored"):
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
    not_run_reason = "skip-gpu：这一趟带了 --skip-gpu" if arguments.skip_gpu else ""
    gpu_pass = GpuPass(runner, judging)
    if not not_run_reason:
        say("  · GPU 趟：每一步测试经 scripts/crash/gpu-quiesce-run.sh 照多机配置清场并回读，跑完不复原（要复原手动跑多机配置里的私有复原脚本）")
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
    """交 [(条, 要不要跑第一段, 要不要跑第二段)]，按序号（点名的按点名的次序）。states 是 {序号: (ItemState, 理由, 定下状态的那一行)}。
    --full（或 SINGLEFS_GATE_FULL=1）不读判定库：选中的全部现判。"""
    nothing = (ItemState(NOT_RUN, False), "", None)
    if break_is("rerun-done") or arguments.full or (break_is("no-carry-without-arguments") and not arguments.item):
        states = {}
    forced = set()
    if break_is("select-by-commit"):
        changed = changed_since_head(arguments.source_root)
        forced = {item["sequence"] for item in items if item["file"] in changed}
    want_stage_two = not arguments.skip_stage_two
    how_to_look = f"python3 {SELF_PATH} --list-items 逐行列「序号、包、变异名、状态」；序号写数字，变异名整条照抄"
    if arguments.rerun and not arguments.item:
        refuse("--rerun 只与 --item 同用", "要重跑沿用得了的条就点名：--item <序号> --rerun；不点名的只跑沿用不了的")
    if arguments.item:
        if arguments.start_from is not None or arguments.package or arguments.shard:
            refuse("--item 与 --from / --package / --shard 同时给了", "点名就只给 --item（可给多次）；按范围跑就不给 --item")
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
        if arguments.stage_two_only and not break_is("stage-two-without-stage-one"):
            not_passed = [item["sequence"] for item in named if not states.get(item["sequence"], nothing)[0].passed_stage_one]
            if not_passed:
                refuse(f"--stage-two-only 点到的第 {'、'.join(str(sequence) for sequence in not_passed)} 条在判定库里没有键相同的 stage-one-only 行（没过过第一段）："
                       "不经第一段直接记第二段的结局不作数",
                       "先让它们过第一段：去掉 --stage-two-only 重跑这几条（第一段没红的会接着跑第二段）")
        selected = []
        for item in named:
            state = states.get(item["sequence"], nothing)[0]
            if arguments.rerun or break_is("named-items-always-rerun") or item["sequence"] in forced:
                selected.append((item, not arguments.stage_two_only, want_stage_two))
            elif state.state in DECISIVE_OUTCOMES:
                say(f"  · 第 {item['sequence']} 条沿用得了（{OUTCOME_TEXT[state.state]}），跳过；要重跑加 --rerun")
            elif state.state == STAGE_ONE_ONLY or (state.state in ENVIRONMENTAL_OUTCOMES and state.passed_stage_one):
                if want_stage_two:
                    selected.append((item, False, True))
                else:
                    say(f"  · 第 {item['sequence']} 条已过第一段，这一趟带着 --skip-stage-two，跳过；要重跑第一段加 --rerun")
            else:
                selected.append((item, True, want_stage_two))
        return selected
    if arguments.package and arguments.package not in MUTATED_PACKAGES:
        refuse(f"--package 给的「{arguments.package}」不在清单的包里", f"只有这几个：{'、'.join(MUTATED_PACKAGES)}；{how_to_look}")
    if arguments.start_from is not None and not 1 <= arguments.start_from <= len(items):
        refuse(f"--from {arguments.start_from} 不在 1..{len(items)} 里", how_to_look)
    shard = parse_shard(arguments.shard) if arguments.shard else None
    selected = []
    for item in items:
        if arguments.start_from is not None and item["sequence"] < arguments.start_from:
            continue
        if arguments.package and item["package"] != arguments.package:
            continue
        if shard and item["sequence"] % shard[1] != shard[0]:
            continue
        state = states.get(item["sequence"], nothing)[0]
        if item["sequence"] in forced:
            selected.append((item, not arguments.stage_two_only, want_stage_two))
        elif state.state in (NOT_RUN, DRIVER_ERROR) and not arguments.stage_two_only:
            selected.append((item, True, want_stage_two))
        elif state.state == STAGE_ONE_ONLY and want_stage_two:
            selected.append((item, False, True))
        elif state.state in ENVIRONMENTAL_OUTCOMES and not arguments.stage_two_only and not break_is("environmental-not-reselected"):
            # 超时、内存撞顶不是定论：照选中重跑（过过第一段的只补第二段），那一步照「升档」自动升档重跑一次
            if state.passed_stage_one and want_stage_two:
                selected.append((item, False, True))
            else:
                selected.append((item, True, want_stage_two))
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
    judgement = Judgement(library, source, arguments, for_run=False, exclusions=load_exclusions(), toolchain=toolchain, tools=tools)
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
             f"{heavy_prefix_text()} python3 {SELF_PATH} --refresh-list 记上")


def last_time_limit_text(row):
    """--summary 的 NO_CONCLUSION 理由里那一段（文件头「输出」）：定下状态的那一行备注里记的上一趟限时——「<步> 超过 <N> 秒」取 N，
    超时之后升档重跑过的「<步> 超时之后升档重跑（<原> → <升档>）…」取升档那趟的那个数，写成「上一趟限时 <N> 秒」；
    备注里没有整步限时（直接撞顶的、驱动错、GPU 趟的、经 runner 按单条限 CPU 的测试步记成「超过 None 秒」的、没有定下状态的那一行）交空串。弄坏开关 no-conclusion-limit-dropped：一律交空串（改前的样子）。"""
    if row is None or break_is("no-conclusion-limit-dropped"):
        return ""
    note = row.get("note", "")
    found = re.search(r"超时之后升档重跑（\d+ → (\d+)）", note) or re.search(r"超过 (\d+) 秒", note)
    return f"上一趟限时 {found.group(1)} 秒" if found else ""


def summarise(library, arguments, source):
    """八栏条数与提交门禁那一行（文件头「输出」）；判定库对不上（段数、校验列、结局认不出）判红退 1，提交门禁判红退 8。"""
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
    refused, escalated, without, missed_items = [], [], [], []
    for item in items:
        state, reason, deciding = states[item["sequence"]]
        counts[state.state] = counts.get(state.state, 0) + 1
        forms = judgement.refused_text(item, deciding)
        if forms:
            refused.append((item, forms))
        if state.state == "caught" and deciding is not None and deciding["caught_by"] in ESCALATED_LABELS:
            escalated.append((item, deciding["caught_by"]))
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
    if table.partial_tail:
        say(f"  · 判定库末行没写完（{len(table.partial_tail.encode())} 字节），不算数；下一次追加之前会截掉，那一条会重跑")
    unknown_outcomes = [row for row in table.rows if row["outcome"] not in ROW_OUTCOMES]
    if unknown_outcomes:
        problems.append(f"{len(unknown_outcomes)} 行是认不出的结局（例：键 {unknown_outcomes[0]['key'][:16]}「{unknown_outcomes[0]['outcome']}」）")
    if table.malformed_lines:
        problems.append(f"判定库里有 {len(table.malformed_lines)} 行段数不是 {len(VERDICT_COLUMNS)}（文件第 {'、'.join(str(number) for number in table.malformed_lines[:5])} 行）")
    if table.forged:
        problems.append(f"判定库里没有校验列或校验列对不上的 {table.forged_text()}：不是驱动用这台机器的密钥写的")
    unknown_gpu_outcomes = [row for row in gpu_table.rows if row["outcome"] not in GPU_ROW_OUTCOMES]
    if unknown_gpu_outcomes:
        problems.append(f"GPU 判定库里有 {len(unknown_gpu_outcomes)} 行是认不出的结局（例：键 {unknown_gpu_outcomes[0]['key'][:16]}「{unknown_gpu_outcomes[0]['outcome']}」）")
    if gpu_table.malformed_lines:
        problems.append(f"GPU 判定库 {GPU_VERDICTS} 里有 {len(gpu_table.malformed_lines)} 行段数不是 {len(GPU_VERDICT_COLUMNS)}"
                        f"（文件第 {'、'.join(str(number) for number in gpu_table.malformed_lines[:5])} 行）")
    if gpu_table.forged:
        problems.append(f"GPU 判定库 {GPU_VERDICTS} 里没有校验列或校验列对不上的 {gpu_table.forged_text()}：不是驱动用这台机器的密钥写的")
    if unrestorable:
        unit, entry = unrestorable[0]
        problems.append(f"清单按单元存的表里有 {len(unrestorable)} 条在此刻的源码上复原不回去（例：{unit.file} {unit.name} 的「{entry.get('description', '')[:60]}」）：lists.tsv 写坏或手改过")
    if sum(counts.values()) != total:
        problems.append(f"八栏加起来 {sum(counts.values())}，清单 {total} 条")
    gate_red = bool(without or counts["missed"] or unlisted or total == 0)
    escalated_field = "" if break_is("escalated-not-summarised") else f" escalated={len(escalated)}"
    say(f"BROAD_MUTATION_GATE total={total} concluded={total - len(without)} missed={counts['missed']} without_conclusion={len(without)} refused={len(refused)} "
        f"unlisted_units={len(unlisted)}{escalated_field}{gpu_fields} verdict={'red' if gate_red else 'green'}")
    for item, state, reason in without:
        changed = judgement.changed_units(item)
        reason_text = "；".join(part for part in (reason, last_time_limit_text(states[item["sequence"]][2])) if part)
        say(f"NO_CONCLUSION\t{item['sequence']}\t{state}\t{item['name']}\t{reason_text}" + (f"；变了：{changed}" if changed else ""))
    if not break_is("escalated-not-summarised"):
        for item, label in escalated:
            say(f"ESCALATED\t{item['sequence']}\t{label}\t{item['name']}\t{judgement.key_of(item)['key']}")
    if problems and not break_is("summary-mismatch-silent"):
        print(f"  ✗ 汇总对不上：{'；'.join(problems)}", file=sys.stderr)
        print(f"  → 怎么办：看 {library.verdicts_path}（GPU 趟的判定在同目录的 {GPU_VERDICTS}）里点到的那几行。段数不对、结局认不出、没有校验列或校验列对不上的行是写坏或手写的：停掉驱动，把判定库拷一份留底，"
              "删掉点到的那几行（别的行不动），那几条再用 --item <序号> --rerun 重判；每一行都校验不过（密钥丢了、换了）就另起一个空库目录重判；"
              f"清单复原不回去的：{heavy_prefix_text()} python3 {SELF_PATH} --refresh-list 重新记", file=sys.stderr)
        return EXIT_INCONSISTENT
    if gate_red and not break_is("summary-gate-silent"):
        print(f"  ✗ 提交门禁判红：{len(without)} 条沿用不了、也没有现判的结论（其中要 GPU 那一趟而没有结论的 {gpu_not_run + gpu_without} 条），{counts['missed']} 条没红，"
              f"{len(unlisted)} 个单元没记清单", file=sys.stderr)
        print(f"  → 怎么办：主 agent 照 .claude/skills/crash-test/commit-stages.md 第 2b 步不带选条参数起驱动剪枝跑（{heavy_prefix_text()} python3 {SELF_PATH} "
              "--threads … --memory-cap …：每条按单元键沿用、沿用不了的现判，超时、内存撞顶的那一步自动升档重跑一次；CPU 那一趟没红又要 GPU 那一趟的条"
              "（NO_CONCLUSION 里理由是 gpu-…）接在最后经 scripts/crash/gpu-quiesce-run.sh 清场再跑，要在有卡的机器上跑，没卡的机器上它们是本次未跑、不算没红也不算通过），再跑这一道；"
              "证据不齐、仍是超时或内存撞顶的（NO_CONCLUSION 那几行），加大 --timeout-floor 或 --memory-cap 再 --item <序号> --rerun；"
              "没红的加一条能杀它的测试再判；确是等价变异的可写进 .cargo/mutants.toml 的 exclude_re（只那几条不再判），理由写在那一行旁边", file=sys.stderr)
        return EXIT_GATE_RED
    return 0


# ── 导入另一台机器的判定行 ───────────────────────────────────────────────────────────────────────────────────

# 这一节不新加模块级的赋值、类、import，也不给新函数的参数写缺省值：判法单元指纹（judging_units_key、gpu_judging_key）的闭包里带着模块级的赋值、类、import 与带缺省值的 def 头，
# 多一条或改一条，库里的判定就一条都不沿用。要常量的写进函数里，要成组的返回值用 argparse.Namespace。

def peer_result_key_path():
    """另一台的密钥放哪（文件头「判定行的校验列」的「另一台的密钥」）：本机密钥（result_key_path）同一个目录、名字在 .key 前加 .peer。"""
    stem, suffix = os.path.splitext(result_key_path())
    return f"{stem}.peer{suffix}"


def peer_mac_howto():
    """对端密钥取不到、或拿它核不过时的出路（两台交换密钥）。"""
    return (f"两台交换密钥：让另一台把它的 ${{XDG_CONFIG_HOME:-~/.config}}/{DEFAULT_RESULT_KEY} 经私有通道原样拷到本机 {peer_result_key_path()}"
            "（chmod 600，所在目录 700；不打印、不进仓、不进库目录），再重跑同一条导入；对端密钥对、表核不过的，从另一台重新取一遍库目录")


def peer_result_key(library):
    """导入时核另一台校验列用的密钥：→ 字节。取不到（不在、权限不对、内容写坏、落在库目录里）拒导入（退 2）；读法同本机那一把（read_result_key），内容不打印。"""
    path = os.path.realpath(peer_result_key_path())
    if path_is_under(path, os.path.realpath(library.home)):
        refuse(f"另一台的密钥 {path} 落在库目录 {library.home} 里：手写得了判定行的人也读得到它，核另一台的校验列就防不住手写的行；一行都没导入", peer_mac_howto())
    key, problem = read_result_key(path)
    if key is None:
        refuse(f"另一台的密钥取不到（{problem}）：它的判定行带的是那一台的密钥算的校验列，核不了就不导；一行都没导入", peer_mac_howto())
    return key


def peer_mac_mismatches(rows, peer_key, mac):
    """另一台表里（read_table 读出的全列行）校验列用 peer_key 核不过的行：→ [「第 <序号> 条（键 <前十六位>）」]；peer_key 是 None（弄坏开关 peer-mac-unchecked）不核、交空表。"""
    if peer_key is None:
        return []
    return [f"第 {fields[1]} 条（键 {fields[0][:16]}）" for fields in rows if not hmac.compare_digest(mac(peer_key, fields), fields[-1])]


def gpu_import_reject_text(reason):
    """被拒的 GPU 判定行的理由，写成 IMPORT_GPU_REJECTED 那一行末尾的人话。"""
    return {"key-not-in-list": "键不是本机此刻清单里任何一条的单元键（两台的源码不同，或这一条已不在清单里）",
            "gpu-pass-changed": "GPU 趟自己的键（这一趟的写法、编进来的特性、GPU 趟判法单元指纹）与本机此刻的不同"}[reason]


def gpu_import_plan(path, rows_read, fresh, already_present, conflicts, withheld, rejected, names, partial_tail):
    """一次 GPU 判定行导入核完之后的结果（plan_gpu_import 的返回）：path 另一台库目录里的 gpu-verdicts.tsv；rows_read 读到的成形的行数；
    fresh 核过、要并进本机的行（前十三段，跑法已加 import: 前缀）；already_present 本机已有相同行的条数；conflicts {键: (本机结局, 另一台结局)}（同一把键两边都有定论而不同，
    这几把键的行不导）；withheld 因此没导的行数；rejected [(行, 理由)]（键对不上、GPU 趟自己的键对不上的行，不导）；names {键: 变异名}（列明细用）；
    partial_tail 另一台那张表的末行没写完（不算数）。"""
    return argparse.Namespace(path=path, rows_read=rows_read, fresh=fresh, already_present=already_present, conflicts=conflicts, withheld=withheld,
                              rejected=rejected, names=names, partial_tail=partial_tail)


def gpu_row_bare(row):
    """一行 GPU 判定拿来比「本机有没有相同行」的样子：前十三段去掉跑法（导入的行跑法带 import: 前缀）。"""
    return tuple(str(row[column]) for column in GPU_VERDICT_COLUMNS[:GPU_SIGNED_COLUMNS] if column != "method")


def plan_gpu_import(library, arguments, directory, peer_key):
    """另一台库目录里 gpu-verdicts.tsv 的行并进本机之前先核完（一行都不写）。段数不是十四、结局认不出、键不是 64 位十六进制的整表拒（同 verdicts.tsv 那一侧）。
    成形的行逐条核两把键：行的键要是本机此刻清单里某一条的单元键（按本机此刻的源码现算）、行的 gpu_key 要等于本机此刻 GPU 趟自己的键（GpuJudging.pass_key，
    含 GPU 趟判法单元指纹），对不上的不导、列出来；对得上的，本机已有相同行的不重复导，同一把键两边都有定论而结局不同的不导。
    每行的校验列先用另一台的密钥（peer_key，import_results 取的；弄坏开关 peer-mac-unchecked 下是 None、不核）核，一行核不过整表拒；核过去掉，落行时照本机的密钥重签。
    弄坏开关 import-verdicts-only：不看 gpu-verdicts.tsv（改前的样子）；
    import-gpu-keys-unchecked：不核两把键；import-gpu-conflict-ignored：两边定论不同的也导；import-unchecked：也不核段数、结局与键的写法。"""
    peer_path = os.path.join(directory, GPU_VERDICTS)
    nothing = gpu_import_plan(peer_path, 0, [], 0, {}, 0, [], {}, False)
    if break_is("import-verdicts-only") or not os.path.isfile(peer_path):
        return nothing
    unchecked = break_is("import-unchecked")
    fetch_howto = "把另一台库目录里的 gpu-verdicts.tsv 与 verdicts.tsv（连同 compositions/）取回到一个目录里再给它"
    table = read_table(peer_path, len(GPU_VERDICT_COLUMNS))
    if table.malformed_lines and not unchecked:
        refuse(f"{peer_path} 里有 {len(table.malformed_lines)} 行段数不是 {len(GPU_VERDICT_COLUMNS)}（第 {'、'.join(str(number) for number in table.malformed_lines[:5])} 行）：取回来的表不完整或被改过",
               "一行都没导入（verdicts.tsv 的行也没导）。重新取一遍；另一台上先跑 --summary 看那张表自己对不对得上", EXIT_INCONSISTENT)
    forged = peer_mac_mismatches(table.rows, peer_key, gpu_result_mac)
    if forged:
        refuse(f"{peer_path} 里有 {len(forged)} 行的校验列用另一台的密钥核不过（{forged[0]}" + ("；……" if len(forged) > 1 else "") + "）：取回来的表被改过，或本机存的对端密钥不是那一台此刻的",
               "一行都没导入（verdicts.tsv 的行也没导）。" + peer_mac_howto(), EXIT_INCONSISTENT)
    rows = [dict(zip(GPU_VERDICT_COLUMNS, row[:GPU_SIGNED_COLUMNS])) for row in table.rows]
    wrong = []
    for row in rows:
        if unchecked:
            break
        if not re.fullmatch(r"[0-9a-f]{64}", row["key"]):
            wrong.append(f"第 {row['sequence']} 条的键「{row['key'][:20]}」不是 64 位十六进制")
        elif row["outcome"] not in GPU_ROW_OUTCOMES:
            wrong.append(f"第 {row['sequence']} 条的结局「{row['outcome']}」认不出")
    if wrong:
        refuse(f"{peer_path} 里有 {len(wrong)} 行不成形：{wrong[0]}" + ("；……" if len(wrong) > 1 else ""), "一行都没导入（verdicts.tsv 的行也没导）。" + fetch_howto, EXIT_INCONSISTENT)
    if not rows:
        return gpu_import_plan(peer_path, 0, [], 0, {}, 0, [], {}, bool(table.partial_tail))
    judgement, items, unlisted, unrestorable = read_only_judgement(library, arguments, arguments.source_root)
    if not items or unlisted or unrestorable:
        refuse(f"本机此刻的清单不全（{len(items)} 条，没记清单的单元 {len(unlisted)} 个，复原不回去 {len(unrestorable)} 处）：另一台的 {len(rows)} 行 GPU 判定没有一张完整的单元键表可核",
               f"一行都没导入（verdicts.tsv 的行也没导）。先记清单：{heavy_prefix_text()} python3 {SELF_PATH} --refresh-list（--source 与两台跑的是同一份源码），再重跑同一条导入")
    judging = GpuJudging(library, judgement, arguments)
    current_keys = {judgement.key_of(item)["key"] for item in items} - {None}
    pass_key = judging.pass_key()
    local = judging.table
    if (local.forged or local.malformed_lines) and not unchecked:
        refuse(f"本机的 GPU 判定库自己就对不上（{local.forged_text() if local.forged else f'{len(local.malformed_lines)} 行段数不对'}）",
               f"先修本机的：python3 {SELF_PATH} --summary 看出路", EXIT_INCONSISTENT)
    kept, rejected = [], []
    for row in rows:
        reason = ""
        if not break_is("import-gpu-keys-unchecked"):
            if row["key"] not in current_keys:
                reason = "key-not-in-list"
            elif row["gpu_key"] != pass_key:
                reason = "gpu-pass-changed"
        if reason:
            rejected.append((row, reason))
        else:
            kept.append(row)
    local_rows = {gpu_row_bare(row) for row in local.rows}
    local_decisive = {row["key"]: row["outcome"] for row in local.rows if row["gpu_key"] == pass_key and row["outcome"] in DECISIVE_OUTCOMES}
    peer_decisive = {row["key"]: row["outcome"] for row in kept if row["outcome"] in DECISIVE_OUTCOMES}
    conflicts = {} if break_is("import-gpu-conflict-ignored") else {key: (local_decisive[key], outcome) for key, outcome in peer_decisive.items()
                                                                    if key in local_decisive and local_decisive[key] != outcome}
    fresh, already_present, withheld = [], 0, 0
    method_index = GPU_VERDICT_COLUMNS.index("method")
    for row in kept:
        if gpu_row_bare(row) in local_rows:
            already_present += 1
        elif row["key"] in conflicts:
            withheld += 1
        else:
            fields = [row[column] for column in GPU_VERDICT_COLUMNS[:GPU_SIGNED_COLUMNS]]
            if not fields[method_index].startswith("import:"):
                fields[method_index] = f"import:{fields[method_index]}"
            fresh.append(fields)
    return gpu_import_plan(peer_path, len(rows), fresh, already_present, conflicts, withheld, rejected, {row["key"]: row["name"] for row in rows}, bool(table.partial_tail))


def import_results(library, source, arguments):
    """两台各跑一片（--shard k/2）之后，把另一台库目录里的判定行并进本机这一份库（文件头「--import-results」）。source 是取回来的那份库目录
    （或其中的 verdicts.tsv）；compositions/ 在就一起拷。全部核完才写：每行段数、结局认得、键是 64 位十六进制；本机已有的相同行不重复导；
    同一把键两边都有定论而结局不同的不导、逐条列出、退 1。另一台的行带的是它那台的密钥算的校验列：先用另一台的密钥（peer_result_key）逐行核，
    对端密钥取不到拒（退 2）、一行核不过整表拒（退 1），全部核过才去掉、照本机的密钥重签。
    同目录的 gpu-verdicts.tsv（GPU 那一趟的判定）也并进来，核法见 plan_gpu_import：键对不上本机此刻源码与 GPU 趟指纹的行不导、列出来、退 1，之后本机的 GPU 趟照现判。"""
    library.result_key(create=True)
    directory = source if os.path.isdir(source) else os.path.dirname(os.path.abspath(source))
    peer_path = os.path.join(directory, "verdicts.tsv")
    fetch_howto = "把另一台库目录里的 verdicts.tsv（连同 gpu-verdicts.tsv、compositions/）取回到一个目录里再给它"
    if not os.path.isfile(peer_path):
        refuse(f"{directory} 里没有 verdicts.tsv：没有判定行可导", fetch_howto)
    table = read_table(peer_path, len(VERDICT_COLUMNS))
    unchecked = break_is("import-unchecked")
    if table.malformed_lines and not unchecked:
        refuse(f"{peer_path} 里有 {len(table.malformed_lines)} 行段数不是 {len(VERDICT_COLUMNS)}（第 {'、'.join(str(number) for number in table.malformed_lines[:5])} 行）：取回来的表不完整或被改过",
               "重新取一遍；另一台上先跑 --summary 看那张表自己对不对得上", EXIT_INCONSISTENT)
    peer_key = None if break_is("peer-mac-unchecked") else peer_result_key(library)
    forged = peer_mac_mismatches(table.rows, peer_key, result_mac)
    if forged:
        refuse(f"{peer_path} 里有 {len(forged)} 行的校验列用另一台的密钥核不过（{forged[0]}" + ("；……" if len(forged) > 1 else "") + "）：取回来的表被改过，或本机存的对端密钥不是那一台此刻的",
               "一行都没导入。" + peer_mac_howto(), EXIT_INCONSISTENT)
    peer_rows = [dict(zip(VERDICT_COLUMNS, row[:RESULT_SIGNED_COLUMNS])) for row in table.rows]
    wrong = []
    for row in peer_rows:
        if unchecked:
            break
        if not re.fullmatch(r"[0-9a-f]{64}", row["key"]):
            wrong.append(f"第 {row['sequence']} 条的键「{row['key'][:20]}」不是 64 位十六进制")
        elif row["outcome"] not in ROW_OUTCOMES:
            wrong.append(f"第 {row['sequence']} 条的结局「{row['outcome']}」认不出")
    if wrong:
        refuse(f"{peer_path} 里有 {len(wrong)} 行不成形：{wrong[0]}" + ("；……" if len(wrong) > 1 else ""), "一行都没导入。" + fetch_howto, EXIT_INCONSISTENT)
    local = library.read_verdicts()
    if (local.forged or local.malformed_lines) and not unchecked:
        refuse(f"本机的判定库自己就对不上（{local.forged_text() if local.forged else f'{len(local.malformed_lines)} 行段数不对'}）",
               f"先修本机的：python3 {SELF_PATH} --summary 看出路", EXIT_INCONSISTENT)
    gpu_plan = plan_gpu_import(library, arguments, directory, peer_key)     # GPU 判定行的核也在写任何一行之前做完（拒了就一行都不导）
    signed = RESULT_SIGNED_COLUMNS

    def bare(row):
        return tuple(str(row[column]) for column in VERDICT_COLUMNS[:signed] if column != "method")

    local_rows = {bare(row) for row in local.rows}

    def decisive(rows):
        found = {}
        for row in rows:
            if row["outcome"] in DECISIVE_OUTCOMES:
                found[row["key"]] = row["outcome"]
        return found
    local_decisive, peer_decisive = decisive(local.rows), decisive(peer_rows)
    conflicts = {key: (local_decisive[key], outcome) for key, outcome in peer_decisive.items() if key in local_decisive and local_decisive[key] != outcome}
    to_import, already_present, withheld = [], 0, 0
    for row in peer_rows:
        if bare(row) in local_rows:
            already_present += 1
        elif row["key"] in conflicts:
            withheld += 1
        else:
            fields = [row[column] for column in VERDICT_COLUMNS[:signed]]
            fields[VERDICT_COLUMNS.index("method")] = row["method"] if row["method"].startswith("import:") else f"import:{row['method']}"
            to_import.append(fields)
    if to_import:
        library.record_verdicts(to_import)
    if gpu_plan.fresh:
        record_gpu_verdicts(library, gpu_plan.fresh)
    copied = 0
    for fields in to_import:
        for key in (fields[0], fields[VERDICT_COLUMNS.index("test_key")]):
            origin = os.path.join(directory, "compositions", key[:2], f"{key}.txt") if key else ""
            if origin and os.path.isfile(origin) and not os.path.exists(library.composition_path(key)):
                os.makedirs(os.path.dirname(library.composition_path(key)), exist_ok=True)
                shutil.copyfile(origin, library.composition_path(key))
                copied += 1
    after = {bare(row) for row in library.read_verdicts().rows}
    lost = [fields for fields in to_import if bare(dict(zip(VERDICT_COLUMNS, fields))) not in after]
    gpu_after = {gpu_row_bare(row) for row in read_gpu_verdicts(library).rows}
    gpu_lost = [fields for fields in gpu_plan.fresh if gpu_row_bare(dict(zip(GPU_VERDICT_COLUMNS, fields))) not in gpu_after]
    append_row(library.imports_path, IMPORT_COLUMNS, [now_text(), os.path.abspath(peer_path), file_digest(peer_path), len(peer_rows), len(to_import), already_present, len(conflicts)])
    if gpu_plan.rows_read:
        append_row(library.imports_path, IMPORT_COLUMNS, [now_text(), os.path.abspath(gpu_plan.path), file_digest(gpu_plan.path), gpu_plan.rows_read, len(gpu_plan.fresh),
                                                          gpu_plan.already_present, len(gpu_plan.conflicts)])
    say(f"BROAD_MUTATION_IMPORT rows_read={len(peer_rows)} imported={len(to_import)} already_present={already_present} conflicting_keys={len(conflicts)} "
        f"withheld_rows={withheld} compositions_copied={copied} partial_tail_ignored={1 if table.partial_tail else 0} "
        f"gpu_rows_read={gpu_plan.rows_read} gpu_imported={len(gpu_plan.fresh)} gpu_already_present={gpu_plan.already_present} gpu_conflicting_keys={len(gpu_plan.conflicts)} "
        f"gpu_withheld_rows={gpu_plan.withheld} gpu_rejected={len(gpu_plan.rejected)} gpu_partial_tail_ignored={1 if gpu_plan.partial_tail else 0}")
    for key in sorted(conflicts):
        name = next((row["name"] for row in peer_rows if row["key"] == key), "")
        say(f"IMPORT_CONFLICT\t{key[:16]}\t本机 {conflicts[key][0]}\t另一台 {conflicts[key][1]}\t{name}")
    for key in sorted(gpu_plan.conflicts):
        say(f"IMPORT_GPU_CONFLICT\t{key[:16]}\t本机 {gpu_plan.conflicts[key][0]}\t另一台 {gpu_plan.conflicts[key][1]}\t{gpu_plan.names.get(key, '')}")
    for row, reason in gpu_plan.rejected:
        say(f"IMPORT_GPU_REJECTED\t{row['sequence']}\t{row['key'][:16]}\t{reason}\t{row['name']}\t{gpu_import_reject_text(reason)}")
    if lost:
        refuse(f"导入之后回读，{len(lost)} 行不在判定库里（例：第 {lost[0][1]} 条）", f"看 {library.verdicts_path} 所在的盘是不是满了；修好重跑同一条导入（已在的行不重复导）",
               EXIT_INCONSISTENT)
    if gpu_lost:
        refuse(f"导入之后回读，{len(gpu_lost)} 行 GPU 判定不在 {GPU_VERDICTS} 里（例：第 {gpu_lost[0][1]} 条）",
               f"看 {gpu_verdicts_path(library)} 所在的盘是不是满了；修好重跑同一条导入（已在的行不重复导）", EXIT_INCONSISTENT)
    if conflicts:
        print(f"  ✗ {len(conflicts)} 把键在两台上都有定论而结局不同，这几把键的行没导入（别的 {len(to_import)} 行已导入）", file=sys.stderr)
        print(f"  → 怎么办：同一把键两边都判过说明分片给错了或有人点名跑过；看上面 IMPORT_CONFLICT 各行，"
              f"在本机点名重跑定一次：{heavy_prefix_text()} python3 {SELF_PATH} --item <序号> --rerun", file=sys.stderr)
    if gpu_plan.rejected or gpu_plan.conflicts:
        print(f"  ✗ GPU 判定：{len(gpu_plan.rejected)} 行的键对不上本机此刻的源码或 GPU 趟指纹、{len(gpu_plan.conflicts)} 把键在两台上都有定论而结局不同，这些行没导入"
              f"（别的 {len(gpu_plan.fresh)} 行已导入）", file=sys.stderr)
        print(f"  → 怎么办：看上面 IMPORT_GPU_REJECTED / IMPORT_GPU_CONFLICT 各行。键对不上多半是两台的 --source 不是同一份源码、本机这条命令的构建环境与工具链认的变量与跑片时不同（--unit-environment 打的那一行要接在命令前），或另一台跑完之后本机的源码、驱动的 GPU 趟写法又改了：对齐之后"
              f"被拒的旧行由本机 GPU 趟现判，不靠重导（在有卡的机器上起 {heavy_prefix_text()} python3 {SELF_PATH}，不带 --shard）；"
              f"定论不同的在本机点名重跑定一次：{heavy_prefix_text()} python3 {SELF_PATH} --item <序号> --rerun", file=sys.stderr)
    if conflicts or gpu_plan.rejected or gpu_plan.conflicts:
        return EXIT_INCONSISTENT
    say(f"  ✓ 导入 {len(to_import)} 行（读到 {len(peer_rows)} 行，本机已有 {already_present} 行）；GPU 判定导入 {len(gpu_plan.fresh)} 行（读到 {gpu_plan.rows_read} 行，本机已有 {gpu_plan.already_present} 行）；"
        f"python3 {SELF_PATH} --summary 看合起来的八栏")
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


def check_prefix(what):
    """起 cargo 的与改库目录的都要带前缀；自证自己起的子进程（假的一步、假的树）不要。"""
    if test_mode():
        return
    if os.environ.get(HEAVY_PREFIX_VARIABLE) not in HEAVY_PREFIX_VALUES:
        refuse(f"没带 {HEAVY_PREFIX_VARIABLE}=commit 或 =user-request：{what}，只在提交时或用户要求时做",
               f"用户要求时这样起：{HEAVY_PREFIX_VARIABLE}=user-request nice -n 19 python3 {SELF_PATH} [参数…]；"
               "只列项、看汇总不用前缀（--list-items、--summary、--check-ignored）")


def check_real_run_is_allowed():
    """真跑（起 cargo）之前：带没带前缀、工具在不在。"""
    check_prefix("广谱变异是重型测试")
    if test_mode():
        return
    missing = [tool for tool in ("cargo", "rustc", "cargo-nextest", "eatmydata", "systemd-run", "systemctl") if shutil.which(tool) is None]
    if missing:
        refuse(f"PATH 上缺：{'、'.join(missing)}", "cargo-nextest 用 cargo install cargo-nextest --locked 装，eatmydata 用系统包管理器装（apt install eatmydata）")


ROCKSDB_PREBUILT_PATH = os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "rocksdb-prebuilt.py")


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


def write_runner(library):
    path = os.path.join(library.home, "test-runner.py")
    write_atomically(path, prune_rules().RUNNER_SCRIPT.encode())
    os.chmod(path, 0o755)
    return path


def coverage_step(specification_path):
    """--coverage-step：驱动在没改过的树上起的一步（经内存包装）。插桩编一遍、列出全部测试，第一段与慢档用例逐条单独一个进程跑、checker 档逐个二进制跑
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

    def listed(executable, cwd, ignored):
        done = subprocess.run([executable, "--list", "--format", "terse", *(["--ignored"] if ignored else [])], cwd=cwd, env=environment,
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

    tests, jobs, harness = [], [], package_arguments(HARNESS_TIER_PACKAGES)
    if spec["kind"] == "one":
        for binary, _kind, _name, executable, cwd in build(harness):
            for test in listed(executable, cwd, False):
                tests.append([binary, test])
                if (binary, test) not in known:
                    jobs.append((len(jobs), binary, test, executable, cwd, [executable, "--exact", test, "--test-threads=1"], True))
    else:
        excluded = {(binary, test) for binary, test, _reason in load_exclusions()}
        for binary, _kind, _name, executable, cwd in build(harness):
            for test in listed(executable, cwd, True):
                if (binary, test) not in excluded:
                    tests.append([binary, test])
                    if (binary, test) not in known:
                        jobs.append((len(jobs), binary, test, executable, cwd, [executable, "--ignored", "--exact", test, "--test-threads=1"], True))
        for _binary, kind, name, executable, cwd in build(["--release", "-p", CHECKER_TIER_PACKAGE, "--lib", "--tests"]):
            job = ("checker:" + ("lib" if kind == "lib" else name), "*")
            tests.append(list(job))
            if job not in known:
                jobs.append((len(jobs), job[0], "*", executable, cwd, [executable], False))
    with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, spec["workers"] if spec["kind"] == "one" else 1)) as pool:
        records = list(pool.map(cover, jobs))
    failed = [record for record in records if record["exit"] != 0]
    write_atomically(spec["out"], (json.dumps({"tests": tests, "jobs": records, "failed": failed}, ensure_ascii=False) + "\n").encode())
    say(f"BROAD_MUTATION_COVERAGE kind={spec['kind']} tests={len(tests)} jobs={len(jobs)} failed={len(failed)}")
    for record in failed:
        print(f"  ✗ 覆盖那一遍在没改过的树上跑红：{record['binary']} {record['test']}（退 {record['exit']}）", file=sys.stderr)  # gate-lint:detail
    if failed:
        print("  → 怎么办：没改过的树上本来就红的测试先修好（或登记进排除表），再重跑驱动；覆盖那一遍不替红的测试记覆盖", file=sys.stderr)
    return 1 if failed else 0


# ── 主流程 ─────────────────────────────────────────────────────────────────────────────────────────────────

def parse_arguments(argv):
    parser = ArgumentParser(description="广谱变异的逐条驱动；用法与库目录的布局见文件头", add_help=True)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--list-items", action="store_true")
    mode.add_argument("--summary", action="store_true")
    mode.add_argument("--check-ignored", action="store_true")
    mode.add_argument("--unit-environment", action="store_true")
    mode.add_argument("--refresh-list", action="store_true")
    mode.add_argument("--import-results", metavar="目录")
    mode.add_argument("--selftest", action="store_true")
    mode.add_argument("--coverage-step", metavar="spec")
    parser.add_argument("--full", action="store_true")
    parser.add_argument("--source")
    parser.add_argument("--result-key")
    parser.add_argument("--toolchain-from")
    parser.add_argument("--tools-from")
    parser.add_argument("--llvm-bin")
    parser.add_argument("--cpu-multiplier", type=float, default=None)
    parser.add_argument("--cpu-floor", type=float, default=None)
    parser.add_argument("--wall-multiplier", type=float, default=None)
    parser.add_argument("--wall-floor", type=float, default=None)
    parser.add_argument("--item", action="append", default=[])
    parser.add_argument("--rerun", action="store_true")
    parser.add_argument("--from", dest="start_from", type=int)
    parser.add_argument("--package")
    parser.add_argument("--shard")
    stage = parser.add_mutually_exclusive_group()
    stage.add_argument("--skip-stage-two", action="store_true")
    stage.add_argument("--stage-two-only", action="store_true")
    parser.add_argument("--stage-two-eatmydata", action="store_true")
    parser.add_argument("--skip-gpu", action="store_true")
    parser.add_argument("--threads", type=int, default=default_threads())
    parser.add_argument("--memory-cap", default=os.environ.get("BROAD_MUTATION_MEMORY_CAP") or DEFAULT_MEMORY_CAP)
    parser.add_argument("--timeout-floor", type=int, default=DEFAULT_TIMEOUT_FLOOR_SECONDS)
    parser.add_argument("--timeout-multiplier", type=float, default=DEFAULT_TIMEOUT_MULTIPLIER)
    parser.add_argument("--suspect-streak", type=int, default=DEFAULT_SUSPECT_STREAK)
    parser.add_argument("--suspect-test-seconds", type=float, default=DEFAULT_SUSPECT_TEST_SECONDS)
    parser.add_argument("--stop-grace", type=int, default=DEFAULT_STOP_GRACE_SECONDS)
    parser.add_argument("--lock-wait", type=int, default=DEFAULT_LOCK_WAIT_SECONDS)
    parser.add_argument("--home")
    arguments = parser.parse_args(argv)
    if arguments.threads < 1:
        parser.error("--threads 要是正整数")
    if arguments.suspect_streak < 1:
        parser.error("--suspect-streak 要是正整数")
    if arguments.coverage_step:
        return arguments
    rules = prune_rules()
    for name, default in (("cpu_multiplier", rules.DEFAULT_CPU_MULTIPLIER), ("cpu_floor", rules.DEFAULT_CPU_FLOOR_SECONDS),
                          ("wall_multiplier", rules.DEFAULT_WALL_MULTIPLIER), ("wall_floor", rules.DEFAULT_WALL_FLOOR_SECONDS)):
        if getattr(arguments, name) is None:
            setattr(arguments, name, default)
    refused_multiplier = rules.check_cpu_multiplier(arguments.cpu_multiplier)
    if refused_multiplier:
        parser.error(refused_multiplier)
    arguments.full = arguments.full or os.environ.get(FULL_VARIABLE) == "1"
    if arguments.full and (arguments.item or arguments.rerun or arguments.stage_two_only):
        parser.error("--full（或 SINGLEFS_GATE_FULL=1）不与 --item、--rerun、--stage-two-only 同用：它不读判定库，选中的全部现判")
    read_only = arguments.list_items or arguments.summary
    if (arguments.result_key or arguments.toolchain_from or arguments.tools_from) and not read_only:
        parser.error("--result-key、--toolchain-from、--tools-from 只给只读的 --list-items / --summary 用（提交门禁的样本）；真跑与写行的用法不认")
    return arguments


def check_removed_arguments(argv):
    """已删的参数（文件头）：传了退 2，出路写新跑法。"""
    removed = [word.split("=", 1)[0] for word in argv if word.split("=", 1)[0] in REMOVED_ARGUMENTS]
    if removed and not break_is("removed-arguments-accepted"):
        refuse("参数已删：" + "；".join(f"{word}（{REMOVED_ARGUMENTS[word]}）" for word in dict.fromkeys(removed)),
               f"不带选条参数就是剪枝：{heavy_prefix_text()} nice -n 19 python3 {SELF_PATH} --threads <线程数> --memory-cap <上限>"
               "（每条变异按单元键过沿用判据，沿用得了沿用、沿用不了现判）；里程碑末全量加 --full；点名重判 --item <序号> --rerun；"
               "提交时照 .claude/skills/crash-test/commit-stages.md 第 2b 步起")


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
    judgement = runner.judgement = Judgement(library, library.tree, arguments, for_run=True, exclusions=runner.exclusions)
    runner.full = arguments.full
    items = generate_list(library)
    written, units = record_list_cache(library, judgement, items)
    say(f"  · 清单：{len(items)} 条，按单元记进 lists.tsv（{units} 个单元，这一趟新写 {written} 行）")
    interrupted = getattr(arguments, "interrupted", None)
    if interrupted is not None and break_is("interrupted-as-done"):
        # 弄坏开关：被打断的那一条照「已完成」落一行抓到，不重跑
        item = next(item for item in items if item["sequence"] == interrupted["sequence"])
        current = judgement.key_of(item)
        library.record_verdict([current["key"] or "", item["sequence"], item["package"], item["name"], "unviable", "one", "", "", "", "0.0", "0.0", today_text(),
                                "fresh", current["locator"], "", "break"])
        judgement = runner.judgement = Judgement(library, library.tree, arguments, for_run=True, exclusions=runner.exclusions)
    states = item_states(judgement, items)
    selected = select_items(judgement, items, arguments, states)
    sentinel = plan_reuse_sentinel(items, states, arguments)
    forced = next((item for item in items if sentinel.forced(str(item["sequence"]))), None)
    if forced is not None and all(item["sequence"] != forced["sequence"] for item, _one, _two in selected):
        selected = sorted(selected + [(forced, True, not arguments.skip_stage_two)], key=lambda entry: entry[0]["sequence"])
    judged, carried, refused = plan_line(items, states, selected)
    say(f"BROAD_MUTATION_PLAN list={len(items)} judge={judged} carried={carried} refused={refused} selected={len(selected)} threads={arguments.threads} "
        f"memory_cap={arguments.memory_cap}{' full=1' if arguments.full else ''}")
    for item, _one, _two in selected:
        _state, reason, _row = states.get(item["sequence"], (None, "full", None))
        if forced is not None and item["sequence"] == forced["sequence"]:
            reason = "sentinel-forced"
        changed = judgement.changed_units(item)
        say(f"BROAD_MUTATION_JUDGE sequence={item['sequence']} reason={reason or ('full' if arguments.full else 'selected')}" + (f" changed={changed}" if changed else ""))
    if not selected:
        if finish_reuse_sentinel(sentinel, {}):
            return 1
        gpu_totals = run_gpu_phase(library, runner, items, arguments)     # CPU 那一趟没有要现判的，GPU 那一趟（上一趟没卡、没来得及跑的）照样补
        if gpu_totals.failed:
            return 1
        say(f"  ✓ 选中 0 条要跑：清单 {len(items)} 条，现判 0、沿用 {carried}、拒算 0（--summary 看八栏与提交门禁那一行）；{gpu_totals_text(gpu_totals)}")
        return 0
    # --full：开跑时沿用得了的结论记下来，现判之后逐条比，翻了面的打 FLIPPED（里程碑末全量靠它看「沿用的那些还对不对」）
    snapshot = {item["sequence"]: states[item["sequence"]][0].state for item, _one, _two in selected} if arguments.full else {}
    stages = [stage for stage, wanted in (("one", any(one for _item, one, _two in selected)), ("two", any(two for _item, _one, two in selected))) if wanted]
    if "two" in stages:
        check_ignored_registration(library.tree, runner.exclusions)    # 基线之前：一步 cargo 都还没起
    runner.runner_path = write_runner(library)
    for stage in stages:
        runner.ensure_baseline(stage)
    runner.prepare_limits(stages)
    runner.ensure_coverage(stages)
    runner.verify_environment_at_start(stages)
    ran = dict.fromkeys(ROW_OUTCOMES, 0)
    outcomes = {}
    flips = []
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
            before = snapshot.get(item["sequence"])
            if before in DECISIVE_OUTCOMES and outcome in FINAL_OUTCOMES and outcome != before:
                flips.append((item["sequence"], before, outcome, item["name"]))
            streaks.saw(item["sequence"], row)
    finally:
        library.flush_deferred_rows()
    say(f"BROAD_MUTATION_TOTAL selected={len(selected)} ran={sum(ran.values())} judged={judged} carried={carried} refused={refused} "
        + " ".join(f"{SUMMARY_KEYS[name]}={count}" for name, count in ran.items()) + f" seconds={time.monotonic() - started:.0f}")
    if arguments.full:
        for sequence, carried_outcome, fresh_outcome, name in flips:
            say(f"FLIPPED\t{sequence}\t{carried_outcome}\t{fresh_outcome}\t{name}")
        say(f"BROAD_MUTATION_FULL flipped={len(flips)}")
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
    """这一趟选条参数罩得住的条（--item、--from、--package、--shard；不带就是全部）：复用哨兵只从这里面抽。"""
    if arguments.item:
        return str(item["sequence"]) in arguments.item or item["name"] in arguments.item
    if arguments.start_from is not None and item["sequence"] < arguments.start_from:
        return False
    if arguments.package and item["package"] != arguments.package:
        return False
    if arguments.shard:
        shard = parse_shard(arguments.shard)
        return item["sequence"] % shard[1] == shard[0]
    return True


def plan_reuse_sentinel(items, states, arguments):
    """复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「二」）：候选是这一趟罩得住、沿用判据此刻认定论（抓到、没红、无效）的条，库里的判定是那一条的结局；
    --skip-stage-two 时只取第一段就定了论的（强制现判那一条只跑第一段，比得了）。--full、--rerun、--stage-two-only 不抽。"""
    sentinel = reuse_checks.ReuseSentinel(SCRIPT_KEY, "广谱变异")
    off = "--full（不读判定库）" if arguments.full else "--rerun（点名的照样重跑）" if arguments.rerun else "--stage-two-only（只补第二段）" if arguments.stage_two_only else ""
    candidates = {}
    for item in items if not off else []:
        state, _reason, row = states.get(item["sequence"], (ItemState(NOT_RUN, False), "", None))
        if state.state in DECISIVE_OUTCOMES and in_run_scope(item, arguments) and (not arguments.skip_stage_two or (row or {}).get("stage") == "one"):
            candidates[str(item["sequence"])] = state.state
    sentinel.plan(candidates, reuse=not off, off_reason=off, printer=say)
    return sentinel


def finish_reuse_sentinel(sentinel, outcomes):
    """哨兵判：outcomes 是这一趟现判的 {序号: 结局}；没在里面的候选就是沿用了。超时、撞顶、驱动错不是结论，强制现判的那一条撞上它们不比。红了交真。"""
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


def unit_environment_names(environment, root):
    """构建环境与工具链认的那几个会话变量：拿掉它、在 root 上算的构建环境的生效值（build_environment_of）就变的，或在 root 里起的 rustc -Vv（toolchain_text）就变的
    （RUSTUP_TOOLCHAIN 这一类：换了工具链，每把键都变）；detached-run.sh 本来带进单元的（UNIT_CARRIED_VARIABLES）与只供自证的变量不算。
    拿掉它就算不出构建环境、起不了 rustc 的也算。弄坏开关 unit-environment-fixed-list 下只认构建环境改取生效值之前那几个，
    unit-environment-toolchain-ignored 下不比工具链。"""
    if break_is("unit-environment-fixed-list"):
        return [name for name in sorted(environment) if name in FIXED_LIST_VARIABLES or name.startswith("CARGO_PROFILE_")]
    base = build_environment_of(root, environment)
    base_toolchain = None if break_is("unit-environment-toolchain-ignored") else toolchain_text(root, environment)
    names = []
    for name in sorted(environment):
        if name in UNIT_CARRIED_VARIABLES or name in TEST_ONLY_VARIABLES or name == SELFTEST_WORK_VARIABLE:
            continue
        without = {other: value for other, value in environment.items() if other != name}
        try:
            changed = build_environment_of(root, without) != base
            if not changed and base_toolchain is not None:
                changed = toolchain_text(root, without) != base_toolchain
        except Refusal:
            changed = True
        if changed:
            names.append(name)
    return names


def print_unit_environment(root):
    """--unit-environment：打一行 `env 名=值 …`（按 shell 引好；一个都没有打空行），提交流程把它原样接在 detached-run.sh start 的 `--` 后面。"""
    names = unit_environment_names(os.environ, root)
    say(("env " + " ".join(shlex.quote(f"{name}={os.environ[name]}") for name in names)) if names else "")
    note(f"  · 构建环境与工具链认的会话变量（{'、'.join(names) if names else '没有'}）：经 detached-run.sh 起单元时把上一行原样接在 `--` 后面；"
         f"{'、'.join(UNIT_CARRIED_VARIABLES)} 单元本来就带")
    return 0


def main(argv):
    check_test_only_variables_before_parsing(argv)
    check_removed_arguments(argv)
    arguments = parse_arguments(argv)
    if arguments.selftest:
        return run_selftest([name for name in os.environ.get(SELFTEST_CELLS_VARIABLE, "").split(",") if name] or None)
    if arguments.coverage_step:
        check_prefix("--coverage-step 在没改过的树上插桩跑全部测试")
        return coverage_step(arguments.coverage_step)
    read_only = arguments.list_items or arguments.summary or arguments.check_ignored or arguments.unit_environment
    library = Library(resolve_home(arguments.home, read_only))   # 里面先核只供自证的变量（check_test_only_variables），再核旧库与（要写库的用法）库目录落在哪
    source = arguments.source_root = resolve_source(arguments.source, library.home, read_only)
    library.result_key_override = arguments.result_key
    if arguments.unit_environment:
        return print_unit_environment(source)
    if arguments.list_items:
        if not break_is("list-items-pipe-unhandled"):
            # 接 | head 时读的一头先关了：照 SIGPIPE 的惯例安静退出（被 SIGPIPE 停，shell 里退 141），不打 BrokenPipeError 的回溯
            signal.signal(signal.SIGPIPE, signal.SIG_DFL)
        list_items(library, arguments, source)
        return 0
    if arguments.summary:
        return summarise(library, arguments, source)
    if arguments.check_ignored:
        return check_ignored_command(source, arguments)
    if arguments.import_results:
        check_prefix("--import-results 往本机的判定库里加行")
    else:
        check_real_run_is_allowed()
        check_rocksdb_entry(source)
    signal.signal(signal.SIGTERM, stop_on_signal)
    signal.signal(signal.SIGINT, stop_on_signal)
    library.lock_descriptor = acquire_lock(library.home, arguments.lock_wait)
    if arguments.import_results:
        return import_results(library, arguments.import_results, arguments)
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
    library.result_key(create=True)     # 要往判定库里写行：密钥没有就先建（文件头「判定行的校验列」），建不了当场拒
    if arguments.refresh_list:
        judgement = Judgement(library, library.tree, arguments, for_run=True, exclusions=[])
        items = generate_list(library)
        written, units = record_list_cache(library, judgement, items)
        say(f"  ✓ 清单 {len(items)} 条，按单元记进 lists.tsv（{units} 个单元，这一趟新写 {written} 行）；不编不跑")
        return 0
    table = library.read_verdicts()
    if table.forged:
        note(f"  · 判定库里 {table.forged_text()}：没有校验列或校验列对不上的行不当判定，那几条照沿用不了选；--summary 照样判红，出路在它那里")
    return run_items(library, arguments, source)


# ── 自证 ───────────────────────────────────────────────────────────────────────────────────────────────────

SELFTEST_STEP_DRIVER = r'''#!/usr/bin/env python3
# 假的一步：照 plan.tsv（序号 <TAB> 步名 <TAB> 动作）办，记一笔调用（序号、步名、树里那份源码此刻是不是被改过）。
import os, sys, time
step, sequence, tree = sys.argv[1], sys.argv[2], sys.argv[3]
work = os.environ["BROAD_MUTATION_SELFTEST_WORK"]
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
    log.write(f"{sequence}\t{step}\t{os.environ.get('BROAD_MUTATION_STEP_SELECTION', '')}\t{os.environ.get('BROAD_MUTATION_RUNNER_MODE', '')}\t"
              f"{os.environ.get('BROAD_MUTATION_STEP_TIME_LIMIT', '')}\n")
prepared_times = os.path.join(work, "runner-lines.tsv")
if os.environ.get("BROAD_MUTATION_RUNNER_MODE") == "measure" and os.environ.get("BROAD_MUTATION_RUNNER_TIMES") and os.path.exists(prepared_times):
    # 基线那几步替 runner 记逐条的秒数：照 <工作目录>/runner-lines.tsv（步名、二进制、文件名、用例、CPU 秒、墙钟秒、哪一部分，- 是哪一部分都算），
    # 只记驱动经 BROAD_MUTATION_STEP_TESTS 点了的那几条、那几个二进制、那一部分（没点是全部）
    import json
    chosen = json.loads(os.environ.get("BROAD_MUTATION_STEP_TESTS") or '{"all": true}')
    with open(os.environ["BROAD_MUTATION_RUNNER_TIMES"], "a") as out:
        for prepared_line in open(prepared_times):
            row_step, binary, stem, test, cpu, wall, part = prepared_line.rstrip("\n").split("\t")
            picked = chosen.get("all") or [binary, test] in chosen.get("tests", []) or binary in chosen.get("binaries", [])
            if row_step == step and picked and part in ("-", chosen.get("part", "-")):
                out.write(f"{stem}\t{test}\t{cpu}\t{wall}\t0\n")
with open(os.path.join(work, "limits.log"), "a") as log:
    # 升档那几格读：哪一趟（first / escalated）、整步时限、内存上限
    log.write(f"{sequence}\t{step}\t{os.environ.get('BROAD_MUTATION_STEP_ATTEMPT', '')}\t{os.environ.get('BROAD_MUTATION_STEP_TIME_LIMIT', '')}\t"
              f"{os.environ.get('BROAD_MUTATION_STEP_MEMORY_CAP', '')}\n")
with open(os.path.join(work, "wrapper-env.log"), "a") as log:
    # 驱动交给内存包装的两样：按自己的上限排队（升档那趟）、撞总上限时先挨杀（改坏了源码的那几步）
    log.write(f"{sequence}\t{step}\t{os.environ.get('BROAD_MUTATION_STEP_ATTEMPT', '')}\t"
              f"{'queue-by-cap' if os.environ.get('RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP') else '-'}\t{'oom-first' if os.environ.get('RUN_WITH_MEMORY_CAP_OOM_FIRST') else '-'}\n")
with open(os.path.join(work, "alerts-env.log"), "a") as log:
    # 驱动交给一步的告警记录（SINGLEFS_CORE_USAGE_ALERTS，没设写 -）：基线那几步照旧写真记录（文件头「内存」）
    log.write(f"{sequence}\t{step}\t{os.environ.get('SINGLEFS_CORE_USAGE_ALERTS') or '-'}\n")
mutated = any("MUTATED_" in open(os.path.join(tree, path)).read()
              for path in ("crates/singlefs-core/src/lib.rs", "crates/singlefs-checker/src/lib.rs"))
with open(os.path.join(work, "calls.log"), "a") as log:
    log.write(f"{sequence}\t{step}\t{'mutated' if mutated else 'pristine'}\n")
action = "ok"
for line in open(os.path.join(work, "plan.tsv")):
    fields = line.rstrip("\n").split("\t")
    if fields[:2] == [sequence, step]:
        action = fields[2]
if "|" in action:
    # 「原来那趟|升档那趟」：驱动升档重跑时 BROAD_MUTATION_STEP_ATTEMPT=escalated
    action = action.split("|", 1)[1 if os.environ.get("BROAD_MUTATION_STEP_ATTEMPT") == "escalated" else 0]
if step.startswith("gpu-"):
    # GPU 那一趟的假步（假 GPU）：记下收到的环境与命令（自证核命令的写法、BINDGEN 的取值、限时），测试步的输出照 libtest 的写法打 Running 与 test result
    with open(os.path.join(work, "gpu-env.log"), "a") as log:
        log.write(f"{sequence}\t{step}\t{os.environ.get('BINDGEN_EXTRA_CLANG_ARGS', '-')}\t{os.environ.get('BROAD_MUTATION_STEP_COMMAND', '-')}\t"
                  f"{os.environ.get('BROAD_MUTATION_STEP_TIME_LIMIT', '')}\n")
    running = "     Running tests/gpu_pair.rs (target/release/deps/gpu_pair-0123456789abcdef)"
    if step == "gpu-test-checker-tier":
        if action in ("ok", "gpu-pass"):
            print(running)
            print("test gpu_case ... ok")
            print("test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out")
            sys.exit(0)
        if action == "gpu-no-adapter":
            print(running)
            print("NOT_RUN_NO_GPU_ADAPTER test=gpu_case reason=没有找到 GPU 适配器（自证的假卡）")
            print("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out")
            sys.exit(0)
        if action == "gpu-fail":
            print(running)
            print("test gpu_case ... FAILED")
            print("failures:")
            print("    gpu_case")
            print("test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out")
            sys.exit(101)
        if action == "gpu-fail-scrambled":
            # --nocapture 加 --test-threads 1：结果行被测试自己的输出打散，认名字靠收尾的 failures: 名单
            print(running)
            print("test gpu_case ... GPU 对拍：第 3 个状态两边不一致")
            print("FAILED")
            print("failures:")
            print("    gpu_case")
            print("test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out")
            sys.exit(101)
        if action == "gpu-zero":
            print(running)
            print("test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out")
            sys.exit(0)
if action.startswith("peak:"):
    # 基线那一步：照包装的峰值表写一行（峰值字节、量的时刻、上限、键、线程数；时刻一段假的一步写 selftest），驱动拿它当这一步的基线峰值
    peaks = os.environ.get("RUN_WITH_MEMORY_CAP_PEAKS")
    if peaks:
        with open(peaks, "a") as handle:
            handle.write(f"{action.split(':', 1)[1]}\tselftest\t{os.environ.get('BROAD_MUTATION_STEP_MEMORY_CAP', '')}\t"
                         f"{os.environ.get('RUN_WITH_MEMORY_CAP_KEY', '')}\t1\n")
    sys.exit(0)
if action == "exit:253":
    print(f"run-with-memory-cap: 超过限时 {os.environ.get('BROAD_MUTATION_STEP_TIME_LIMIT')} 秒（自证的假一步）", flush=True)
    sys.exit(253)
if action == "exit:250":
    print(f"run-with-memory-cap: 撞了内存上限 {os.environ.get('BROAD_MUTATION_STEP_MEMORY_CAP')}（自证的假一步）", flush=True)
    sys.exit(250)
if action == "exit:245":
    print("        FAIL [   0.010s] (3/9) singlefs-harness::catching the_case_that_catches_it")
    print("run-with-memory-cap: 显存不够（自证的假一步）", flush=True)
    sys.exit(245)
if action == "fail-nextest":
    print("        FAIL [   0.010s] (3/9) singlefs-harness::catching the_case_that_catches_it")
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
    print("        PASS [   0.010s] (1/9) singlefs-harness::catching the_case_that_passes", flush=True)
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
        print("        FAIL [   0.010s] (1/1) singlefs-harness::catching built_from_stale_" + "_and_".join(stale))
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
    "Cargo.toml": '[workspace]\nmembers = ["crates/singlefs-core", "crates/singlefs-checker", "crates/singlefs-format", "crates/singlefs-harness"]\nresolver = "2"\n',
    "crates/singlefs-core/Cargo.toml": selftest_package("singlefs-core"),
    "crates/singlefs-core/src/lib.rs": SELFTEST_CORE,
    "crates/singlefs-checker/Cargo.toml": selftest_package("singlefs-checker"),
    "crates/singlefs-checker/src/lib.rs": "pub fn seven() -> u32 {\n    7\n}\npub fn eight() -> u32 {\n    8\n}\n",
    # 忽略用例登记那几格用：lib 里模块中的两条（一条慢档用例、一条工具）、tests 里顶层的慢档用例与大档、嵌套模块里的工具；包是 singlefs-format，不碰清单的两个包
    "crates/singlefs-format/Cargo.toml": selftest_package("singlefs-format"),
    "crates/singlefs-format/src/lib.rs": '#[cfg(test)]\nmod tests {\n    #[test]\n    #[ignore = "harness 慢档用例：debug 下单条跑过 60 秒"]\n    fn in_lib_slow() {}\n\n'
                                         '    #[test]\n    #[ignore = "收缩一个种子：SINGLEFS_SELFTEST_SEED 必给"]\n    fn in_lib_tool() {}\n}\n',
    "crates/singlefs-format/tests/ignored_cases.rs": '#[test]\n#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒"]\nfn slow_one() {}\n\n'
                                                     '#[test]\n#[ignore = "大档：规模从环境变量取"]\nfn env_large_tier() {}\n\n'
                                                     'mod inner {\n    #[test]\n    #[ignore = "收缩一个种子：SINGLEFS_SELFTEST_SEED 必给"]\n    fn env_tool() {}\n}\n',
    # 抓到它的那条测试：nextest 名字「singlefs-harness::catching the_case_that_catches_it」，静态认得出、测试键算得出
    "crates/singlefs-harness/Cargo.toml": selftest_package("singlefs-harness"),
    "crates/singlefs-harness/src/lib.rs": "",
    "crates/singlefs-harness/tests/catching.rs": "#[test]\nfn the_case_that_catches_it() {\n    assert_eq!(1 + 1, 2);\n}\n",
}
SELFTEST_EXCLUSIONS = ("# 自证用的排除表\n"
                       "singlefs-format tests::in_lib_tool\t自证：库里模块中的工具用例\n"
                       "singlefs-format::ignored_cases env_large_tier\t自证：按环境变量起的大档\n"
                       "singlefs-format::ignored_cases inner::env_tool\t自证：嵌套模块里的工具用例\n")
# 序号 → 假的一步怎么答（没写的步答 0）：各种结局，两段各有；第二段的三种测试步各答一次
SELFTEST_PLAN = [("1", "one-test", "fail-nextest"), ("2", "one-build", "fail-compile"), ("3", "one-test", "exit:253"),
                 ("5", "two-test-doc", "fail-cargo"), ("6", "one-test", "exit:250"), ("7", SLOW_STEP_NAME, "fail-nextest"),
                 ("8", "two-test-checker-tier", "exit:253")]
SELFTEST_EXPECTED = {"1": ("caught", "one"), "2": ("unviable", "one"), "3": ("timeout", "one"), "4": ("missed", "two"),
                     "5": ("caught", "two"), "6": ("memory-cap", "one"), "7": ("caught", "two"), "8": ("timeout", "two")}
SELFTEST_EXPECTED_SUMMARY = "total=8 caught=3 missed=1 unviable=1 timeout=2 memory_cap=1 stage_one_only=0 driver_error=0 not_run=0 refused=1 unlisted_units=0"
SELFTEST_STAGE_TWO_STEPS = ["two-build-harness", "two-test-doc", SLOW_STEP_NAME, "two-build-checker-tier", "two-test-checker-tier"]
SELFTEST_BUILD_STEPS = ("one-build", "two-build-harness", "two-build-checker-tier", "two-build-debug")   # 编译步：基线键拒算、每趟照跑
SELFTEST_NEXTEST_CASE = "singlefs-harness::catching the_case_that_catches_it"
SELFTEST_SLOW_CASE = "singlefs-format::ignored_cases slow_one"
SELFTEST_JUDGING_KEY = "selftest-judging-units"
SELFTEST_GPU_JUDGING_KEY = "selftest-gpu-judging-units"
# 按条记基线那一格（selftest_baseline_entry_cells）另加的源码：第一段测试步一个测试二进制三条、一条起自己这个可执行文件（current_exe，单元闭包拒算）；
# checker 档包的库里两条（一条拒算）、另一个测试目标一条
SELFTEST_ENTRY_BASELINE_FILE = "crates/singlefs-harness/tests/per_entry.rs"
SELFTEST_ENTRY_BASELINE_SOURCES = {
    "Cargo.toml": SELFTEST_SOURCES["Cargo.toml"].replace('"crates/singlefs-harness"]', '"crates/singlefs-harness", "crates/singlefs-checker-tier"]'),
    SELFTEST_ENTRY_BASELINE_FILE: "#[test]\nfn kept_first() {\n    assert_eq!(2 + 2, 4);\n}\n\n#[test]\nfn kept_second() {\n    assert_eq!(3 + 3, 6);\n}\n\n"
                                  "#[test]\nfn refused_third() {\n    let _path = std::env::current_exe();\n}\n",
    "crates/singlefs-checker-tier/Cargo.toml": selftest_package("singlefs-checker-tier"),
    "crates/singlefs-checker-tier/src/lib.rs": "pub fn nine() -> u32 {\n    9\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn kept_in_the_library() {\n"
                                               "        assert_eq!(super::nine(), 9);\n    }\n\n    #[test]\n    fn refused_in_the_library() {\n"
                                               "        let _path = std::env::current_exe();\n    }\n}\n",
    "crates/singlefs-checker-tier/tests/quick.rs": "#[test]\nfn kept_in_a_test_target() {\n    assert_eq!(1 + 1, 2);\n}\n",
}
# 假的一步替 runner 记的逐条秒数（步名、二进制、文件名、用例、CPU 秒、墙钟秒、哪一部分）：checker 档的库那一份去掉拒算那一条量 2.0 秒、只跑它量 0.5 秒
SELFTEST_ENTRY_RUNNER_LINES = ("one-test\tsinglefs-harness::catching\tcatching\tthe_case_that_catches_it\t0.5\t0.6\t-\n"
                               "one-test\tsinglefs-harness::per_entry\tper_entry\tkept_first\t1.25\t1.5\t-\n"
                               "one-test\tsinglefs-harness::per_entry\tper_entry\tkept_second\t1.5\t1.75\t-\n"
                               "one-test\tsinglefs-harness::per_entry\tper_entry\trefused_third\t0.75\t1.0\t-\n"
                               "two-test-checker-tier\tsinglefs-checker-tier\tsinglefs_checker_tier\t*\t2.0\t2.5\tskip\n"
                               "two-test-checker-tier\tsinglefs-checker-tier\tsinglefs_checker_tier\t*\t0.5\t0.75\texact\n"
                               "two-test-checker-tier\tsinglefs-checker-tier::quick\tquick\t*\t1.0\t1.25\twhole\n")


# GPU 那一趟的格用的工作树（SELFTEST_SOURCES 加一个 checker 档包）：库里没有测试；tests/gpu_pair.rs 的测试体里有 #[cfg(feature = "gpu")] 的语句（挂 gpu 特性、不带特性也编），
# 里面一条测试经 judge() 够得着 singlefs-core 的 one、two、four、five（序号 1、2、4、5 的变异）；tests/plain_case.rs 不挂特性，只够得着 three（序号 3，CPU 那一趟的测试碰得到、
# 带 gpu 的这一趟帮不上）；six 与 singlefs-checker 的 seven、eight（序号 6、7、8）没有任何测试够得着。
# 不让 gpu_pair.rs 整个文件挂 #![cfg(feature = "gpu")]：这一格测的是测试体里有挂 gpu 的语句的写法；整个文件挂的那种（不带特性时不编，entry-cfg-inactive）
# CPU 那一趟的覆盖者键（mutation-prune.py 的 coverers_key）现在跳过，由 mutation-prune.py 自己的自证管，这一格不去测。
SELFTEST_GPU_FILES = {
    "Cargo.toml": SELFTEST_SOURCES["Cargo.toml"].replace('"crates/singlefs-harness"]', '"crates/singlefs-harness", "crates/singlefs-checker-tier"]'),
    "crates/singlefs-checker-tier/Cargo.toml": selftest_package("singlefs-checker-tier") + '\n[features]\ngpu = []\n\n[dependencies]\nsinglefs-core = { path = "../singlefs-core" }\n',
    "crates/singlefs-checker-tier/src/lib.rs": "pub fn judge() -> u32 {\n    singlefs_core::one() + singlefs_core::two() + singlefs_core::four() + singlefs_core::five()\n}\n",
    "crates/singlefs-checker-tier/tests/gpu_pair.rs": '#[test]\nfn gpu_case() {\n    #[cfg(feature = "gpu")]\n    let extra = 1;\n    #[cfg(not(feature = "gpu"))]\n    let extra = 0;\n'
                                                      '    assert!(singlefs_checker_tier::judge() + extra > 0);\n}\n',
    "crates/singlefs-checker-tier/tests/plain_case.rs": "#[test]\nfn plain_case() {\n    assert!(singlefs_core::three() > 0);\n}\n",
}
SELFTEST_GPU_CASE = "singlefs-checker-tier::gpu_pair gpu_case"      # 抓到它的测试，nextest 的写法（二进制 id 加用例名）


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
        self.exclusions_path = os.path.join(work, "exclusions.tsv")
        with open(self.exclusions_path, "w") as handle:
            handle.write(SELFTEST_EXCLUSIONS)
        self.environment = dict(os.environ, BROAD_MUTATION_SELFTEST_WORK=work, BROAD_MUTATION_STEP_DRIVER="1",
                                BROAD_MUTATION_LIST_DRIVER=os.path.join(work, "list-driver.py"), BROAD_MUTATION_EXPORT_EVERYTHING="1",
                                BROAD_MUTATION_ALLOW_VOLATILE_HOME="1", BROAD_MUTATION_EXCLUSIONS=self.exclusions_path,
                                BROAD_MUTATION_RESULT_KEY=self.result_key_path(), BROAD_MUTATION_JUDGING_KEY=SELFTEST_JUDGING_KEY,
                                BROAD_MUTATION_GPU_JUDGING_KEY=SELFTEST_GPU_JUDGING_KEY)
        # 复用哨兵：别的格比的是现判与沿用的条数，沿用得了的全当哨兵、不抽强制现判；reuse-three-questions 与 reuse-sentinel 两格自己设
        self.environment[reuse_checks.SENTINEL_COUNT_VARIABLE] = "1000"
        self.environment.pop(reuse_checks.SENTINEL_SEED_VARIABLE, None)
        # 告警记录：调用方没设时基线那几步会被设成工作根那份真记录（文件头「内存」）；自证起的每一趟都显式指到工作目录里这一份，经真内存包装的那一格也不写真记录
        self.alerts_variable, _real_record = real_alerts_record()
        self.environment[self.alerts_variable] = os.path.join(work, "core-usage-alerts.jsonl")
        for name in ("SINGLEFS_BROAD_MUTATION_HOME", HEAVY_PREFIX_VARIABLE, FULL_VARIABLE, "BROAD_MUTATION_KILL_AT", "BROAD_MUTATION_STEP_VIA_WRAPPER",
                     "BROAD_MUTATION_TOOLCHAIN", "BROAD_MUTATION_PROGRESS_INTERVAL", "BROAD_MUTATION_RUSTC", "BROAD_MUTATION_TOOL_VERSIONS", "RUSTUP_TOOLCHAIN"):
            self.environment.pop(name, None)
        self.slice = f"singlefs_broad_mutation_selftest_{os.getpid()}.slice"
        self.slice_used = False

    def write_plan(self, plan):
        with open(os.path.join(self.work, "plan.tsv"), "w") as handle:
            handle.write("".join("\t".join(row) + "\n" for row in plan))

    def write_mutants(self, mutants):
        with open(os.path.join(self.work, "mutants.json"), "w") as handle:
            json.dump(mutants, handle)

    def result_key_path(self):
        """自证里的判定行密钥（库目录之外：各个库都在工作目录下各自的子目录里，密钥在 result-key/ 下），驱动头一回写行时建它。"""
        return os.path.join(self.work, "result-key", "broad-mutation-result.key")

    def peer_side_key_path(self):
        """自证里「另一台」自己的密钥（另一台那几份库各自跑时用它签），与本机这一份不同；导入前经 exchange_peer_key 拷到本机的对端密钥位置。"""
        return os.path.join(self.work, "peer-result-key", "broad-mutation-result.key")

    def peer_key_path(self):
        """本机存另一台密钥的位置：照驱动的取法（peer_result_key_path）由本机这一份推出。"""
        stem, suffix = os.path.splitext(self.result_key_path())
        return f"{stem}.peer{suffix}"

    def exchange_peer_key(self):
        """两台交换密钥：把「另一台」的密钥原样拷到本机的对端密钥位置（600）。交 (另一台的密钥字节, 为什么不成)。"""
        key, problem = read_result_key(self.peer_side_key_path())
        if key is None:
            return None, problem
        os.makedirs(os.path.dirname(self.peer_key_path()), mode=0o700, exist_ok=True)
        shutil.copyfile(self.peer_side_key_path(), self.peer_key_path())
        os.chmod(self.peer_key_path(), 0o600)
        return key, ""

    def peer_signed_line(self, fields):
        """自证改过的一行判定照另一台的写法重签（用「另一台」的密钥）：交写进表里的那一行。"""
        key, _problem = read_result_key(self.peer_side_key_path())
        return "\t".join(signed_result_row(key, fields[:RESULT_SIGNED_COLUMNS]))

    def peer_signed_gpu_fields(self, fields):
        """自证改过的一行 GPU 判定照另一台的写法重签：交十四段。"""
        key, _problem = read_result_key(self.peer_side_key_path())
        cleaned = [clean_field(field) for field in fields[:GPU_SIGNED_COLUMNS]]
        return cleaned + [gpu_result_mac(key, cleaned)]

    def signed(self, fields):
        """自证手写的行照驱动的写法签上校验列（用自证这一份密钥）；密钥还没建交原样。"""
        key, _problem = read_result_key(self.result_key_path())
        return signed_result_row(key, fields) if key is not None else fields

    def command(self, home, *arguments, source=None):
        return [sys.executable, SELF_PATH, "--home", os.path.join(self.work, home), "--source", source or self.source, "--timeout-floor", "5", "--lock-wait", "30",
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
        """GPU 那一趟的假步收到的 [(序号, 步名, BINDGEN_EXTRA_CLANG_ARGS, 命令, 整步限时)]。"""
        path = os.path.join(self.work, "gpu-env.log")
        if not os.path.exists(path):
            return []
        with open(path) as handle:
            return [tuple(line.rstrip("\n").split("\t")) for line in handle if line.strip()]

    def gpu_rows(self, home):
        """这份库的 GPU 判定库里成形的行（{列名: 值}）。"""
        return [dict(zip(GPU_VERDICT_COLUMNS, row)) for row in read_table(gpu_verdicts_path(Library(os.path.join(self.work, home))), len(GPU_VERDICT_COLUMNS)).rows]

    def clear_calls(self):
        for name in ("calls.log", "selections.log", "coverage-known.log", "gpu-env.log"):
            path = os.path.join(self.work, name)
            if os.path.exists(path):
                os.remove(path)

    def verdicts_path(self, home):
        return os.path.join(self.work, home, "verdicts.tsv")

    def rows(self, home):
        """这份库的判定库里成形的行（{列名: 值}）。"""
        return [dict(zip(VERDICT_COLUMNS, row)) for row in read_table(self.verdicts_path(home), len(VERDICT_COLUMNS)).rows]

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
    """那几格基本的：临时库目录拒、生成清单与列项、项写错、点名与两段、杀掉再起、各种结局、汇总、--from 与 --shard、上一趟被 KILL 留下的运行目录开跑先删。"""
    work = selftest.work
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
            shutil.copytree(os.path.join(work, "first"), os.path.join(shm_work, "first"), symlinks=True, ignore=shutil.ignore_patterns("target", "target-coverage"))
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
    unknown = selftest.run("first", "--item", "no-such-mutant", "--skip-stage-two")
    selftest.check(unknown.returncode == EXIT_USAGE and not selftest.calls(), f"不认得的项没拒（退 {unknown.returncode}，调了 {len(selftest.calls())} 步）",
                   "看 select_items：清单里找不到的序号或变异名要退 2 并写怎么查")
    # 点名（一条按序号、一条按变异名）只跑第一段：只有基线与这两条被调，第二段的步一次都没调；基线在原样的树上、每条在改过的树上跑（判完拷回原件，下一条才对得上）
    name_of_seven = listed_lines[6][2] if len(listed_lines) >= 7 and len(listed_lines[6]) == 6 else "?"
    named = selftest.run("first", "--item", "1", "--item", name_of_seven, "--skip-stage-two")
    calls = selftest.calls()
    rows = {row["sequence"]: row for row in selftest.rows("first")}
    selftest.check(named.returncode == 0 and selftest.called_sequences() == {"0", "1", "7"} and not any(call[1].startswith("two-") for call in calls)
                   and all(call[2] == ("pristine" if call[0] == "0" else "mutated") for call in calls)
                   and set(rows) == {"1", "7"} and (rows["1"]["outcome"], rows["1"]["stage"], rows["1"]["caught_by"]) == ("caught", "one", SELFTEST_NEXTEST_CASE)
                   and re.fullmatch(r"[0-9a-f]{64}", rows["1"]["test_key"]) and (rows["7"]["outcome"], rows["7"]["stage"]) == (STAGE_ONE_ONLY, "one"),
                   f"点名两条只跑第一段不对（退 {named.returncode}；调用 {calls}；行 {sorted((row['sequence'], row['outcome'], row['stage']) for row in rows.values())}）：{named.stderr}",
                   "看 select_items 的点名那一支、process_item 的第一段与 --skip-stage-two：基线在没改过的树上跑、每条在改过的树上跑、判完拷回原件；抓到的那一行带测试键")
    # --stage-two-only：只给「只过了第一段」的那一条补第二段，第一段的步不再调；第二段里 harness 慢档用例那一步抓到它，带测试名
    selftest.clear_calls()
    second = selftest.run("first", "--stage-two-only")
    calls = selftest.calls()
    seven = [(row["outcome"], row["stage"], row["caught_by"]) for row in selftest.rows_of("first", 7)]
    selftest.check(second.returncode == 0 and selftest.called_sequences() == {"0", "7"} and all(call[1].startswith("two-") for call in calls)
                   and seven == [(STAGE_ONE_ONLY, "one", ""), ("caught", "two", SELFTEST_NEXTEST_CASE)],
                   f"--stage-two-only 不对（退 {second.returncode}；调用 {calls}；第 7 条的行 {seven}）：{second.stderr}",
                   "看 select_items：只选状态是 stage-one-only 的、不跑第一段；process_item 第二段哪一步红了就记抓到并带测试名")
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
    selftest.check("3" in called and [row["outcome"] for row in rows if row["sequence"] == "3"] == ["timeout"] and "上一趟被打断在第 3 条" in resumed.stdout,
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
    # 两段式的结局：抓到（第一段、文档测试、慢档用例各一）、没红、无效、超时（两段各一）、内存撞顶，逐条对
    final = {row["sequence"]: (row["outcome"], row["stage"]) for row in rows if row["outcome"] != STAGE_ONE_ONLY}
    five = [row for row in rows if row["sequence"] == "5" and row["outcome"] == "caught"]
    selftest.check(final == SELFTEST_EXPECTED and [row["caught_by"] for row in five] == ["tests::the_stage_two_case"]
                   and five and five[0]["refused"].startswith("catching-test-unresolved") and re.fullmatch(r"[0-9a-f]{64}", five[0]["coverage_key"]),
                   f"两段式的结局不对（第 5 条 {[(row['caught_by'], row['refused'], row['coverage_key'][:12]) for row in five]}）：{sorted(final.items())}",
                   "看 process_item 的 run_stage：第二段全绿才记 missed，抓到要带第一条失败的测试名（libtest 结果行经 lib_libtest_result_lines.py，should panic 那一段不进名字）；"
                   "抓到它的测试认不出（没有 Running / Doc-tests 行）的记 catching-test-unresolved 并另记覆盖者键")
    # 汇总八栏加起来等于清单条数，没红的逐条列名；第 5 条（抓到它的测试认不出）算拒算、照覆盖者键认
    summary = selftest.run("second", "--summary")
    selftest.check(summary.returncode == EXIT_GATE_RED and SELFTEST_EXPECTED_SUMMARY in summary.stdout and "verdict=red" in summary.stdout
                   and len([line for line in summary.stdout.splitlines() if line.startswith("MISSED\t")]) == 1
                   and len([line for line in summary.stdout.splitlines() if line.startswith("REFUSED\t5\t")]) == 1,
                   f"汇总不对（退 {summary.returncode}）：{summary.stdout}{summary.stderr}", "看 summarise 的八栏、refused=、MISSED 与 REFUSED 明细")
    # 第二段只补第一段没跑到的：没红的第 4 条第二段依次调的就是这五步（文档测试、慢档用例、快档），命令里没有重跑第一段的那一遍
    stage_two_of_four = [call[1] for call in selftest.calls() if call[0] == "4" and call[1].startswith("two-")]
    selftest_exclusions = load_exclusions(selftest.exclusions_path)
    slow_filter = exclusion_filter(selftest_exclusions)
    table = {step[0]: step[3] for step in step_table(argparse.Namespace(threads=3, stage_two_eatmydata=False), selftest_exclusions)}
    table_without_exclusions = {step[0]: step[3] for step in step_table(argparse.Namespace(threads=3, stage_two_eatmydata=False), [])}
    commands_are_right = (
        table.get("two-test-doc") == ["cargo", "test", "--locked", "--doc", "-p", "singlefs-core", "-p", "singlefs-checker"]
        and table.get(SLOW_STEP_NAME, [])[:3] == ["eatmydata", "cargo", "nextest"] and OLD_SLOW_STEP_NAME not in table
        and table_without_exclusions[SLOW_STEP_NAME][-2:] == ["--run-ignored", "only"]
        and table.get("two-test-checker-tier") == ["cargo", "test", "--locked", "--release", "-p", CHECKER_TIER_PACKAGE, "--lib", "--tests"]
        and not any("--include-ignored" in command or "--ignored" in command for command in table.values())
        and slow_filter is not None and slow_filter.startswith("not (") and table["one-test"] + ["--run-ignored", "only", "-E", slow_filter] == table[SLOW_STEP_NAME]
        and not any(name.startswith("two-test") and command[-len(table["one-test"]):] == table["one-test"] for name, command in table.items()))
    selftest.check(stage_two_of_four == SELFTEST_STAGE_TWO_STEPS and commands_are_right,
                   f"第二段的步不对：第 4 条调的是 {stage_two_of_four}；各步的命令 {table}",
                   "看 step_table：第二段 = 文档测试（被变异的两个包）+ harness 慢档用例（第一段的起法加 --run-ignored only 与排除表的 -E）+ checker 档快档，不重跑第一段那一遍")
    # 汇总对不上必须报：给第 4 条的键追加一行认不出的结局
    four_rows = selftest.rows_of("second", 4)
    if not four_rows:
        # 前面几格红了、第 4 条一行判定都没有：这一格判不了，记判错，不在这里抛出去（后面的格照判）
        selftest.check(False, "判定库里有认不出的结局时汇总没判红（前提不成立：second 里第 4 条一行判定都没有，照上面几格先修）",
                       "看 run_items：second 那几趟要给第 4 条落判定行，这一格拿它最后一行造认不出的结局")
    else:
        four = four_rows[-1]
        append_row(selftest.verdicts_path("second"), VERDICT_COLUMNS, selftest.signed([four[column] if column != "outcome" else "no-such-outcome"
                                                                                       for column in VERDICT_COLUMNS[:RESULT_SIGNED_COLUMNS]]))
        mismatched = selftest.run("second", "--summary")
        selftest.check(mismatched.returncode == EXIT_INCONSISTENT, f"判定库里有认不出的结局时汇总没判红（退 {mismatched.returncode}）",
                       "看 summarise 末尾的 problems：对不上要退 1")
    # --from 与 --shard（只跑第一段）
    selftest.clear_calls()
    from_seven = selftest.run("third", "--from", "7", "--skip-stage-two")
    selftest.check(from_seven.returncode == 0 and selftest.called_sequences() == {"0", "7", "8"},
                   f"--from 7 跑的不是第 7、8 条（退 {from_seven.returncode}；{sorted(selftest.called_sequences())}）", "看 select_items 的 start_from")
    selftest.clear_calls()
    shard = selftest.run("third", "--shard", "0/2", "--skip-stage-two")
    selftest.check(shard.returncode == 0 and selftest.called_sequences() == {"2", "4", "6"},
                   f"--shard 0/2 跑的不是沿用不了的偶数序号（退 {shard.returncode}；{sorted(selftest.called_sequences())}）", "看 select_items 的 shard：序号 mod n == k")
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
    dirty = selftest.run("third", "--item", "1", "--skip-stage-two")
    third_calls = [call for call in selftest.calls() if call[0] == "1"]
    selftest.check(dirty.returncode == 0 and "1" in selftest.called_sequences() and third_calls and all(call[2] == "mutated" for call in third_calls)
                   and f"删了这一趟的编译目录 {stale_run}" in dirty.stderr and not selftest.build_leftovers("third"),
                   f"上一趟被 KILL 留下的运行目录（手改过的树、多出的测试文件、旧产物）没在开跑时删掉、或这一趟没照常判（退 {dirty.returncode}；"
                   f"调用 {sorted(selftest.called_sequences())}；库目录里还剩 {selftest.build_leftovers('third')}）：{dirty.stdout[-400:]}{dirty.stderr[-300:]}",
                   "看 Library.begin_run：拿到锁之后删 runs/ 下上一趟留下的每一个运行目录（删之前核没有进程开着），再新建这一趟的（弄坏开关 interrupted-run-directory-kept 下不删）")
    return listed_lines


def selftest_state_cells(selftest):
    """一条的状态不取最后一行、点名默认跳过沿用得了的条、--stage-two-only 要过过第一段（库「first」接着用：第 1 条第一段抓到，第 7 条第二段抓到）。"""
    selftest.clear_calls()
    skipped = selftest.run("first", "--item", "1", "--skip-stage-two")
    rows_of_one = selftest.rows_of("first", 1)
    selftest.check(skipped.returncode == 0 and "1" not in selftest.called_sequences() and len(rows_of_one) == 1 and "跳过" in skipped.stdout,
                   f"点名沿用得了的条没跳过（退 {skipped.returncode}；被调的序号 {sorted(selftest.called_sequences())}；第 1 条 {len(rows_of_one)} 行）：{skipped.stdout}",
                   "看 select_items 的点名那一支：沿用得了的条不带 --rerun 不选，逐条说一句")
    again = selftest.run("first", "--item", "1", "--skip-stage-two", "--rerun")
    selftest.check(again.returncode == 0 and len(selftest.rows_of("first", 1)) == 2,
                   f"点名加 --rerun 没有重跑并追加新行（退 {again.returncode}）：{again.stderr}", "看 select_items：带 --rerun 的点名不看状态，判定库只追加")
    # 第二段的定论不被后来的 stage-one-only 行降级：第 7 条（第二段抓到）点名重跑只跑第一段，状态还是 caught，之后不带 --rerun 的续跑不再碰它
    downgrade = selftest.run("first", "--item", "7", "--skip-stage-two", "--rerun")
    last_of_seven = [row["outcome"] for row in selftest.rows_of("first", 7)][-1:]
    selftest.clear_calls()
    resumed = selftest.run("first", "--from", "7", "--package", "singlefs-checker")
    selftest.check(downgrade.returncode == 0 and last_of_seven == [STAGE_ONE_ONLY] and selftest.states("first").get("7") == "caught"
                   and resumed.returncode == 0 and "7" not in selftest.called_sequences(),
                   f"第二段的定论被后来的 stage-one-only 行降级了（第 7 条最后一行 {last_of_seven}，状态 {selftest.states('first').get('7')}；续跑里被调的 {sorted(selftest.called_sequences())}）",
                   "看 verdict_state：caught / missed / unviable 取其中最后一行，stage-one-only 行不盖它们")
    # 抓到的条点名重跑时机器忙、记了超时：状态还是 caught
    selftest.write_plan([("1", "one-test", "exit:253")])
    busy = selftest.run("first", "--item", "1", "--skip-stage-two", "--rerun")
    selftest.write_plan(SELFTEST_PLAN)
    outcomes_of_one = [row["outcome"] for row in selftest.rows_of("first", 1)]
    selftest.check(busy.returncode == 0 and outcomes_of_one == ["caught", "caught", "timeout"] and selftest.states("first").get("1") == "caught",
                   f"抓到的条被后来的超时行盖掉了（第 1 条各行 {outcomes_of_one}，状态 {selftest.states('first').get('1')}）",
                   "看 verdict_state：timeout / memory-cap 是环境型的结局，不盖掉前面的 caught / missed / unviable")
    # --stage-two-only 点到没过过第一段的条：拒，退 2，一步都不调
    selftest.clear_calls()
    rows_before = len(selftest.rows("first"))
    not_passed = selftest.run("first", "--item", "2", "--item", "7", "--stage-two-only")
    selftest.check(not_passed.returncode == EXIT_USAGE and not selftest.calls() and len(selftest.rows("first")) == rows_before and "第 2 条" in not_passed.stderr,
                   f"--stage-two-only 点到没过过第一段的条没拒（退 {not_passed.returncode}，调了 {len(selftest.calls())} 步）：{not_passed.stderr}",
                   "看 select_items：点名加 --stage-two-only 的条要有键相同的 stage-one-only 行，没有就拒")


def selftest_table_cells(selftest):
    """判定库与清单表：清单表被改坏、末行没写完、认不出的退出码。"""
    # 清单按单元存的表被改坏（第 1 条那个单元的那一行换成原文对不上的一条）：--summary 判红退 1、--list-items 少那一条
    selftest.run("names", "--refresh-list")
    lists = os.path.join(selftest.work, "names", "lists.tsv")
    rows = read_table(lists, len(LIST_COLUMNS)).rows
    one = next((row for row in rows if "MUTATED_1()" in row[1]), None)
    if one is not None:
        entries = json.loads(one[1])
        entries[0]["original"] = "不是这一段原文"
        append_rows(lists, LIST_COLUMNS, [[one[0], json.dumps(entries, ensure_ascii=False)]])
    summary = selftest.run("names", "--summary")
    listed = selftest.listed("names")
    selftest.check(one is not None and summary.returncode == EXIT_INCONSISTENT and "复原不回去" in summary.stderr and len(listed) == len(selftest.mutants) - 1,
                   f"清单按单元存的表被改坏了没判红（汇总退 {summary.returncode}；列项 {len(listed)} 条）：{summary.stderr[-300:]}",
                   "看 restore_list 与 summarise 的 unrestorable：表里的一条在此刻的源码上复原不回去要报出来、退 1")
    # cargo-mutants 列了被变异的两个包之外的一条（singlefs-format 里的）：那个单元不记清单，复原不回去，--refresh-list 拒（退 1）
    format_path = "crates/singlefs-format/src/lib.rs"
    format_text = SELFTEST_SOURCES[format_path]
    marker = "fn in_lib_slow() {}"
    line_index = format_text.split("\n").index("    " + marker)
    start = (line_index + 1, 5 + len(marker) - 2)
    mutated = replace_span(format_text, start, (line_index + 1, 5 + len(marker)), "{ panic!() }" + MUTATION_MARKER)
    stray = {"package": "singlefs-format", "file": format_path, "name": f"{format_path}:{start[0]}:{start[1]}: replace in_lib_slow with panic",
             "span": {"start": {"line": start[0], "column": start[1]}, "end": {"line": start[0], "column": 5 + len(marker)}}, "replacement": "{ panic!() }",
             "diff": "".join(difflib.unified_diff(format_text.splitlines(keepends=True), mutated.splitlines(keepends=True), f"a/{format_path}", f"b/{format_path}"))}
    selftest.write_mutants([dict(stray, package="singlefs-core")] + selftest.mutants)
    try:
        outside = selftest.run("outside-list", "--refresh-list")
    finally:
        selftest.write_mutants(selftest.mutants)
    selftest.check(outside.returncode == EXIT_INCONSISTENT and "复原不回去" in outside.stderr,
                   f"清单里有被变异包之外的一条时 --refresh-list 没拒（退 {outside.returncode}）：{outside.stderr[-300:]}",
                   "看 record_list_cache：清单按单元记进 lists.tsv 之后照它复原一遍，与 cargo-mutants 给的逐条比，复原不回去就拒（弄坏开关 list-cache-unchecked 下不比）")
    # 判定行写到一半被杀：末行不算数、汇总不因此红；下一次追加之前截掉，那一条重跑、只留一行完整的
    half = selftest.run("partial", "--item", "1", "--skip-stage-two", BROAD_MUTATION_KILL_AT="row-half-written")
    results_path = selftest.verdicts_path("partial")
    with open(results_path, "rb") as handle:
        ends_mid_row = not handle.read().endswith(b"\n")
    summary = selftest.run("partial", "--summary")
    rerun = selftest.run("partial", "--item", "1", "--skip-stage-two")
    with open(results_path, "rb") as handle:
        text_after = handle.read()
    rows_of_one = [row["outcome"] for row in selftest.rows_of("partial", 1)]
    after = selftest.run("partial", "--summary")
    selftest.check(half.returncode == -signal.SIGKILL and ends_mid_row and summary.returncode == EXIT_GATE_RED and summary_field(summary.stdout, "not_run") == 8
                   and rerun.returncode == 0 and text_after.endswith(b"\n") and rows_of_one == ["caught"] and after.returncode == EXIT_GATE_RED
                   and summary_field(after.stdout, "caught") == 1 and os.path.exists(results_path + ".dropped-partial-rows"),
                   f"末行没写完的判定库没处理对（写一半那趟退 {half.returncode}、末行不完整 {ends_mid_row}；汇总退 {summary.returncode}；重跑退 {rerun.returncode}、"
                   f"第 1 条各行 {rows_of_one}；再汇总退 {after.returncode}）：{summary.stderr}{after.stderr}",
                   "看 read_signed_table 的 partial_tail（末行没有换行结尾的不算数）与 append_rows 里的 drop_partial_tail（追加之前截掉）")
    # 一步退了认不出的码：落一行 driver-error、往下走，汇总单列；编译步退 101 而没有编译错误的不算无效；续跑时重试这两条
    selftest.write_plan([("2", "one-build", "exit:101"), ("3", "one-test", "exit:104")])
    selftest.clear_calls()
    first = selftest.run("unknown", "--skip-stage-two")
    outcomes = {row["sequence"]: row["outcome"] for row in selftest.rows("unknown")}
    summary = selftest.run("unknown", "--summary")
    doubt_checked = selftest.environment_steps()
    selftest.clear_calls()
    retried = selftest.run("unknown", "--skip-stage-two")
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
    selftest.run("environment", "--item", "1", "--skip-stage-two")
    selftest.write_plan(SELFTEST_PLAN + [(ENVIRONMENT_SEQUENCE, "one-build", "exit:101")])
    selftest.clear_calls()
    broken = selftest.run("environment", "--item", "3", "--skip-stage-two")
    selftest.check(broken.returncode == EXIT_BASELINE_RED and selftest.environment_steps() == ["one-build"] and not selftest.called_sequences()
                   and not selftest.rows_of("environment", 3) and "疑似环境坏了" in broken.stderr,
                   f"基线绿过之后编译坏了，重起没发现（退 {broken.returncode}；验的步 {selftest.environment_steps()}；被调的序号 {sorted(selftest.called_sequences())}）：{broken.stderr}",
                   "看 Runner.verify_environment_at_start：这一趟没现跑基线的那一段，开跑前在没改过的树上把编译步各跑一遍")
    # 连续 K 条无效而原件上编译照过：不是环境，接着跑
    selftest.write_plan([])
    selftest.run("streak", "--item", "8", "--skip-stage-two")
    selftest.write_plan([(str(sequence), "one-build", "fail-compile") for sequence in (1, 2, 3)])
    selftest.clear_calls()
    continued = selftest.run("streak", "--package", "singlefs-core", "--skip-stage-two", "--suspect-streak", "3")
    outcomes = {row["sequence"]: row["outcome"] for row in selftest.rows("streak")}
    selftest.check(continued.returncode == 0 and selftest.environment_steps() == ["one-build", "one-build"]
                   and [outcomes.get(str(sequence)) for sequence in (1, 2, 3, 4, 5, 6)] == ["unviable"] * 3 + [STAGE_ONE_ONLY] * 3 and "不是环境" in continued.stdout,
                   f"连续无效而原件上编译照过时没有接着跑（退 {continued.returncode}；验的步 {selftest.environment_steps()}；各条 {sorted(outcomes.items())}）：{continued.stderr}",
                   "看 SuspectStreaks.saw：到了 K 条就在没改过的树上重跑那一步，绿了清掉这一串接着跑")
    # 连续 K 条秒红的抓到而原件上那一步也红：退 6 报疑似环境坏了、列出这 K 条的点名重跑命令，后面的条不跑
    selftest.write_plan([])
    selftest.run("fast", "--item", "8", "--skip-stage-two")
    selftest.write_plan([(str(sequence), "one-test", "fail-nextest") for sequence in (1, 2, 3, 4)] + [(ENVIRONMENT_SEQUENCE, "one-test", "exit:100")])
    selftest.clear_calls()
    stopped = selftest.run("fast", "--skip-stage-two", "--suspect-streak", "3", "--suspect-test-seconds", "30")
    selftest.write_plan(SELFTEST_PLAN)
    selftest.check(stopped.returncode == EXIT_BASELINE_RED and selftest.called_sequences() == {"1", "2", "3"} and selftest.environment_steps() == ["one-build", "one-test"]
                   and "--item 1 --item 2 --item 3 --rerun" in stopped.stderr,
                   f"连续 3 条秒红的抓到、原件上那一步也红，没有停下报疑似环境坏了（退 {stopped.returncode}；被调的序号 {sorted(selftest.called_sequences())}；"
                   f"验的步 {selftest.environment_steps()}）：{stopped.stderr}",
                   "看 SuspectStreaks.saw 与 Runner.verify_on_pristine_tree：原件上那一步红了退 6，出路里列出这几条的点名重跑命令")


def selftest_sync_cells(selftest):
    """恢复原件的临时件不进树、恢复半途被杀之后下一趟照常判；换了工具链每把键都变。"""
    work = selftest.work
    killed = selftest.run("extras", "--item", "3", "--skip-stage-two", BROAD_MUTATION_KILL_AT="restore-after-copy")
    killed_runs = selftest.leftover_run_directories("extras")
    stray = sorted(files_in_tree(os.path.join(killed_runs[0], "tree")) - set(SELFTEST_SOURCES)) if len(killed_runs) == 1 else ["（runs/ 下不是恰好一个运行目录）"]
    restore_beside_tree = len(killed_runs) == 1 and os.path.exists(os.path.join(killed_runs[0], "inflight.restore"))
    rerun = selftest.run("extras", "--item", "3", "--skip-stage-two")
    selftest.check(killed.returncode == -signal.SIGKILL and not stray and restore_beside_tree and rerun.returncode == 0 and not selftest.build_leftovers("extras"),
                   f"恢复原件半途被杀之后树里多出了文件、临时件不在运行目录里、或重跑没照常判完（被杀那趟退 {killed.returncode}；多出 {stray}；"
                   f"重跑退 {rerun.returncode}；库目录里还剩 {selftest.build_leftovers('extras')}）：{rerun.stderr}",
                   "看 restore_original：临时件建在这一趟的运行目录里（inflight.restore），不建在树里；Library.begin_run 开跑先删上一趟留下的运行目录")
    # 工具链进每一把键：换了工具链，判过的一条都不沿用（沿用 0）；换回来照沿用
    first_toolchain, second_toolchain = "rustc 1.0.0 (selftest a)", "rustc 2.0.0 (selftest b)"
    selftest.write_plan([(str(sequence), "one-test", "fail-nextest") for sequence in range(1, 9)])
    judged = selftest.run("toolchain", "--skip-stage-two", BROAD_MUTATION_TOOLCHAIN=first_toolchain)
    selftest.clear_calls()
    changed = selftest.run("toolchain", "--skip-stage-two", BROAD_MUTATION_TOOLCHAIN=second_toolchain)
    rejudged = selftest.judged_sequences()
    selftest.clear_calls()
    back = selftest.run("toolchain", "--skip-stage-two", BROAD_MUTATION_TOOLCHAIN=first_toolchain)
    selftest.write_plan(SELFTEST_PLAN)
    selftest.check(judged.returncode == 0 and changed.returncode == 0 and rejudged == {str(sequence) for sequence in range(1, 9)} and plan_field(changed.stdout, "carried") == 0
                   and back.returncode == 0 and not selftest.judged_sequences() and plan_field(back.stdout, "carried") == 8,
                   f"工具链没进每一把键（换了工具链现判的 {sorted(rejudged)}、沿用 {plan_field(changed.stdout, 'carried')}；换回来现判的 {sorted(selftest.judged_sequences())}、"
                   f"沿用 {plan_field(back.stdout, 'carried')}）：{changed.stderr[-300:]}",
                   "看 Judgement 的 globals（mutation-prune.py 的 global_lines：rustc -vV 各行进每一把键）")
    # exclude_re（等价变异的出路）走一遍：被判那棵树的 .cargo/mutants.toml 写一条 exclude_re 排掉第 3 条，再跑：清单少那一条（--list-items、--summary 都是 7 条）、
    # 清单整版重判（mutants.toml 解析后的值进清单按单元存的键：lists.tsv 每个单元重写一行），别的条照沿用（判定的键里没有 mutants.toml，现判 0）。
    # 假清单驱动照 cargo-mutants 的读法在专用树里读 .cargo/mutants.toml、按变异名 re.search 去掉匹配上的。弄坏开关 exclude-re-ignored（换树不拷 .cargo/mutants.toml）下清单照旧 8 条，这一格判错
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
    before_exclusion = selftest.run("exclude-re", "--skip-stage-two", source=excluded_source, BROAD_MUTATION_LIST_DRIVER=excluding_driver)
    before_names = sorted(entry["name"] for entry in selftest.listed("exclude-re", excluded_source).values())
    write_sources(excluded_source, {".cargo/mutants.toml": "# 自证：第 3 条当等价变异排掉\nexclude_re = ['MUTATED_3\\(\\)']\n"})
    selftest.clear_calls()
    after_exclusion = selftest.run("exclude-re", "--skip-stage-two", source=excluded_source, BROAD_MUTATION_LIST_DRIVER=excluding_driver)
    rejudged_after_exclusion = selftest.judged_sequences()
    after_names = sorted(entry["name"] for entry in selftest.listed("exclude-re", excluded_source).values())
    excluded_summary = selftest.run("exclude-re", "--summary", source=excluded_source)
    selftest.write_plan(SELFTEST_PLAN)
    selftest.clear_calls()
    list_line = re.search(r"清单：(\d+) 条，按单元记进 lists\.tsv（(\d+) 个单元，这一趟新写 (\d+) 行）", after_exclusion.stdout)
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
    """假一步记的 [(序号, 步名, 跑了哪些, runner 模式, 整步时限)]。"""
    path = os.path.join(selftest.work, "selections.log")
    if not os.path.exists(path):
        return []
    with open(path) as handle:
        return [tuple(line.rstrip("\n").split("\t")) for line in handle if line.strip()]


def selftest_limits(selftest):
    """假一步记的 [(序号, 步名, 哪一趟, 整步时限, 内存上限)]。"""
    path = os.path.join(selftest.work, "limits.log")
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
    """剪枝：按覆盖挑测试（C2）、第二段按二进制（E6）、文档测试每次跑（E1）、测试步不设整步时限而经 runner 限 CPU（C4）；
    不带参数就是剪枝（沿用数 > 0）、改了一个函数只现判那个函数里的条、所在文件在某提交之后改过别处照样沿用、--full 只写不读与翻面、提交门禁判绿、第二段基线红的出路。"""
    work, core = selftest.work, "crates/singlefs-core/src/lib.rs"
    coverage_one = {"tests": [["fake-package::fake_target", "covers_one"], ["fake-package::fake_target", "covers_two"]],
                    "jobs": [{"binary": "fake-package::fake_target", "test": "covers_one", "covered": [[core, "one"]], "profiles": 1, "exit": 0},
                             {"binary": "fake-package::fake_target", "test": "covers_two", "covered": [[core, "two"]], "profiles": 1, "exit": 0}], "failed": []}
    coverage_two = {"tests": [["fake-package::slow", "slow_case"], ["checker:tier_one", "*"], ["checker:lib", "*"]],
                    "jobs": [{"binary": "fake-package::slow", "test": "slow_case", "covered": [[core, "one"]], "profiles": 1, "exit": 0},
                             {"binary": "checker:tier_one", "test": "*", "covered": [[core, "one"]], "profiles": 1, "exit": 0},
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
    expected = {("1", "one-test"): ["1 tests", "limit", "None"], ("1", SLOW_STEP_NAME): ["fake-package::slow", "limit", "None"],
                ("1", "two-test-checker-tier"): ["tier_one", "limit", "None"], ("1", "two-test-doc"): ["all", "", picked.get(("1", "two-test-doc"), ["", "", ""])[2]],
                ("3", "two-test-doc"): ["all", "", picked.get(("3", "two-test-doc"), ["", "", ""])[2]], ("0", "one-test"): ["all", "measure", "None"]}
    selftest.check(ran.returncode == 0 and all(picked.get(key) == value for key, value in expected.items())
                   and not any(key in picked for key in (("3", "one-test"), ("3", SLOW_STEP_NAME), ("3", "two-test-checker-tier")))
                   and ("3", "one-build") in picked and rows.get("3", {}).get("outcome") == "missed" and "零覆盖" in rows.get("3", {}).get("note", ""),
                   f"按覆盖挑测试、第二段按二进制、文档测试每次跑、测试步经 runner 限 CPU 不对（退 {ran.returncode}；各步 {sorted(picked.items())}；第 3 条 {rows.get('3')}）：{ran.stderr[-400:]}",
                   "看 Judgement.selection（执行到容器单元的测试、第二段执行到它的二进制）、Runner.ensure_coverage、Runner.command_for（挑空的步不跑，文档测试一直跑）"
                   "与 Runner.run_step / time_limit_of（测试步 runner 模式 limit、基线 measure，不设整步时限）")
    # 不带参数就是剪枝：头一趟全部现判（抓到 6 条、无效 1 条、没红 1 条），第二趟不带任何选条参数，8 条全部沿用、一步不调
    catching = f"fail-nextest:{SELFTEST_SLOW_CASE}"
    plan = [(str(sequence), "one-test", catching) for sequence in (1, 3, 5, 6, 7, 8)] + [("2", "one-build", "fail-compile")]
    selftest.write_plan(plan)
    pruned_source = selftest_source_copy(selftest, "pruned-source", git=True)
    first = selftest.run("carry", source=pruned_source)
    selftest.clear_calls()
    second = selftest.run("carry", source=pruned_source)
    selftest.check(first.returncode == 0 and second.returncode == 0 and not selftest.judged_sequences() and plan_field(second.stdout, "carried") == 8
                   and "现判 0、沿用 8、拒算 0" in second.stdout,
                   f"不带参数、库里有可沿用的判定时没有沿用（头一趟退 {first.returncode}；第二趟退 {second.returncode}、现判的 {sorted(selftest.judged_sequences())}、"
                   f"沿用 {plan_field(second.stdout, 'carried')}）：{second.stdout[-400:]}{second.stderr[-300:]}",
                   "看 run_items、item_states 与 select_items：不带选条参数时每条过沿用判据（键相同、抓到的测试键相同、没红的覆盖者键相同、无效的键相同），沿用得了的不选")
    # 改了一个函数（five 的体）：只有它里面那一条现判；同一份文件在上一次提交之后改过别处，别的函数里的条照样沿用（不按提交选条）
    with open(os.path.join(pruned_source, core), "w") as handle:
        handle.write(SELFTEST_CORE.replace("pub fn five() -> u32 {\n", "pub fn five() -> u32 {\n    let _unused = 0;\n"))
    selftest.write_mutants(selftest_mutants(dict(SELFTEST_SOURCES, **{core: SELFTEST_CORE.replace("pub fn five() -> u32 {\n", "pub fn five() -> u32 {\n    let _unused = 0;\n")})))
    try:
        selftest.clear_calls()
        changed = selftest.run("carry", source=pruned_source)
        changed_lines = [line for line in changed.stdout.splitlines() if line.startswith("BROAD_MUTATION_JUDGE ")]
        selftest.check(changed.returncode == 0 and selftest.judged_sequences() == {"5"} and plan_field(changed.stdout, "carried") == 7
                       and "现判 1、沿用 7、拒算 0" in changed.stdout and any("改了 five" in line for line in changed_lines),
                       f"同一条变异内容没变、判定库里有可沿用的行、所在文件在某提交之后改过别处，没照样沿用（退 {changed.returncode}；现判的 {sorted(selftest.judged_sequences())}；"
                       f"沿用 {plan_field(changed.stdout, 'carried')}；{changed_lines}）：{changed.stderr[-300:]}",
                       "看 select_items：只按单元键选条（键对不上的现判，作废时 BROAD_MUTATION_JUDGE 报变了的单元），不按 git diff 选（弄坏开关 select-by-commit 恢复按提交选条）")
        # 抓到它的那条测试（slow_one）改了函数体：抓到的那六条的测试键都变了，现判；无效、没红的两条照沿用
        cases = "crates/singlefs-format/tests/ignored_cases.rs"
        with open(os.path.join(pruned_source, cases), "w") as handle:
            handle.write(SELFTEST_SOURCES[cases].replace("fn slow_one() {}", "fn slow_one() {\n    assert_eq!(2 + 2, 4);\n}"))
        selftest.clear_calls()
        retested = selftest.run("carry", source=pruned_source)
        selftest.check(retested.returncode == 0 and selftest.judged_sequences() == {"1", "3", "5", "6", "7", "8"} and plan_field(retested.stdout, "carried") == 2
                       and "catching-test-changed" in retested.stdout,
                       f"抓到它的那条测试改了之后那几条没现判（现判的 {sorted(selftest.judged_sequences())}；沿用 {plan_field(retested.stdout, 'carried')}）：{retested.stdout[-400:]}",
                       "看 Judgement.row_holds 与 mutation-prune.py 的 catching_test_key：抓到的判定要那条测试的测试键（它的单元闭包键）与此刻的相同才沿用")
        # --full：不读判定库，选中的全部现判、照键写回；第 6 条这一回没红（翻了面），逐条打 FLIPPED
        selftest.write_plan([row for row in plan if row[0] != "6"])
        selftest.clear_calls()
        full = selftest.run("carry", "--full", source=pruned_source)
        summary = selftest.run("carry", "--summary", source=pruned_source)
        selftest.check(full.returncode == 0 and selftest.judged_sequences() == {str(sequence) for sequence in range(1, 9)} and "FLIPPED\t6\tcaught\tmissed\t" in full.stdout
                       and "BROAD_MUTATION_FULL flipped=1" in full.stdout and summary.returncode == EXIT_GATE_RED and summary_field(summary.stdout, "missed") == 2,
                       f"--full 没有只写不读、或没列出翻面的条（退 {full.returncode}；现判的 {sorted(selftest.judged_sequences())}）：{full.stdout[-400:]}；"
                       f"汇总 {[line for line in summary.stdout.splitlines() if 'GATE' in line]}",
                       "看 select_items（--full 不读判定库）与 run_items 的 snapshot / flips：开跑时沿用得了的结论记下来，现判之后逐条比，翻面的列 FLIPPED")
    finally:
        selftest.write_mutants(selftest.mutants)
        selftest.write_plan(SELFTEST_PLAN)
    # 全部有结论、没有没红：提交门禁绿
    selftest.write_plan([(str(sequence), "one-test", "fail-nextest") for sequence in range(1, 9)])
    selftest.run("green", "--skip-stage-two")
    green = selftest.run("green", "--summary")
    selftest.check(green.returncode == 0 and "verdict=green" in green.stdout and "refused=0" in green.stdout,
                   f"每条都抓到的提交门禁没判绿（退 {green.returncode}）：{green.stdout[-300:]}{green.stderr[-300:]}",
                   "看 summarise：抓到、没红、无效都算有结论，没红为零、没有单元缺清单才绿")
    # 第二段基线红：报拒绝、点名红的测试、给出路
    selftest.write_plan([("0", "two-test-checker-tier", "fail-cargo")])
    red = selftest.run("stage-two-red", "--item", "4")
    selftest.write_plan(SELFTEST_PLAN)
    selftest.check(red.returncode == EXIT_BASELINE_RED and "tests::the_stage_two_case" in red.stderr and "--skip-stage-two" in red.stderr and "crash_identity" in red.stderr,
                   f"第二段基线红没报成带出路的拒绝（退 {red.returncode}）：{red.stderr[-400:]}",
                   "看 Runner.ensure_baseline 第二段那一支：列出红的测试，出路写已知的那条测试要谁改、修好之前 --skip-stage-two")
    selftest.clear_calls()


def selftest_reuse_cells(selftest):
    """复用三问（reuse-three-questions）与复用哨兵（reuse-sentinel），写法在文件头「复用三问与复用哨兵」。项里除了 CPU 那一趟的 1–8 条，
    还有 GPU 那一趟的（项名 gpu:<序号>：CPU 那一趟没红、又有带 gpu 特性的测试够得着的第 4、5 条），两趟的复用一起问；复用哨兵 CPU 一份、GPU 一份。"""
    core = "crates/singlefs-core/src/lib.rs"
    head = "pub fn five() -> u32 {\n"
    plan = [(str(sequence), "one-test", f"fail-nextest:{SELFTEST_SLOW_CASE}") for sequence in (1, 3, 6, 7, 8)] + [("2", "one-build", "fail-compile")]
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
        gpu_skipped = selftest.run("three-questions", source=source, BROAD_MUTATION_BREAK="gpu-carry-ignored", **sentinel_environment)
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
    """全局那几行：加一条排除、只改 Cargo.toml 的注释、加大限时参数，别的条照样沿用；改了构建环境的生效值（RUSTFLAGS），每一条都现判。"""
    selftest.write_plan([(str(sequence), "one-test", f"fail-nextest:{SELFTEST_SLOW_CASE}") for sequence in range(1, 9)])
    source = selftest_source_copy(selftest, "global-source")
    first = selftest.run("globals", "--skip-stage-two", source=source)
    cases = []
    exclusions_with_row = os.path.join(selftest.work, "exclusions-plus-one.tsv")
    with open(exclusions_with_row, "w") as handle:
        handle.write(SELFTEST_EXCLUSIONS + "singlefs-format::ignored_cases some_other_case\t自证：排除表加了一条别的测试\n")
    selftest.clear_calls()
    excluded = selftest.run("globals", "--skip-stage-two", source=source, BROAD_MUTATION_EXCLUSIONS=exclusions_with_row)
    cases.append(("改一条排除", excluded, selftest.judged_sequences(), "exclusions-in-global-key"))
    with open(os.path.join(source, "Cargo.toml"), "w") as handle:
        handle.write("# 只改注释：成员与 profile 一样没变\n" + SELFTEST_SOURCES["Cargo.toml"])
    selftest.clear_calls()
    commented = selftest.run("globals", "--skip-stage-two", source=source)
    cases.append(("只改 Cargo.toml 注释", commented, selftest.judged_sequences(), "manifest-text-in-global-key"))
    selftest.clear_calls()
    limits = selftest.run("globals", "--skip-stage-two", "--cpu-multiplier", "4", "--timeout-floor", "60", "--memory-cap", "32G", source=source)
    cases.append(("加大限时参数与上限", limits, selftest.judged_sequences(), "limits-in-key"))
    for what, ran, judged, switch in cases:
        selftest.check(first.returncode == 0 and ran.returncode == 0 and not judged and plan_field(ran.stdout, "carried") == 8,
                       f"{what}之后别的条没照样沿用（退 {ran.returncode}；现判的 {sorted(judged)}；沿用 {plan_field(ran.stdout, 'carried')}）：{ran.stderr[-300:]}",
                       f"看 mutation-prune.py 的 global_lines 与 build_environment_lines：排除表、Cargo.toml 原文、限时参数都不进键（弄坏开关 {switch} 把它放进键）")
    selftest.clear_calls()
    flags = selftest.run("globals", "--skip-stage-two", source=source, RUSTFLAGS="-C selftest-flag")
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
    selftest.check(re.fullmatch(r"[0-9a-f]{64}", before) and commented == before,
                   f"只改驱动注释判法单元指纹变了（改前 {before[:16]}、只改注释与文档串之后 {commented[:16]}）",
                   "看 judging_units_key：取 JUDGING_ENTRIES 那几个函数的 python 单元闭包键（lib_function_units.py：注释与文档串不进指纹），不取整份原文（弄坏开关 judge-units-raw-text）")
    selftest.check(re.fullmatch(r"[0-9a-f]{64}", body) and body != before,
                   f"改了判结局的函数（verdict_state 的体）判法单元指纹没变（{before[:16]} → {body[:16]}）",
                   "看 JUDGING_ENTRIES：选条、沿用、结论分类、升档那几个函数都要在里面，它们点到的函数与常量跟着进闭包")


def selftest_refused_cells(selftest):
    """拒算：容器单元的闭包里有认不出的形态（include_str! 的参数不是字面量），那一条每趟现判、成功那句报拒算条数；别的条照沿用。"""
    core = "crates/singlefs-core/src/lib.rs"
    refused_core = SELFTEST_CORE.replace("pub fn four() -> u32 {\n", "pub fn four() -> u32 {\n    let _ = runtime_text();\n") + \
        'fn runtime_text() -> usize {\n    include_str!(env!("SELFTEST_INCLUDE")).len()\n}\n'
    source = selftest_source_copy(selftest, "refused-source", {core: refused_core})
    selftest.write_mutants(selftest_mutants(dict(SELFTEST_SOURCES, **{core: refused_core})))
    selftest.write_plan([(str(sequence), "one-test", "fail-nextest") for sequence in range(1, 9)])
    try:
        first = selftest.run("refused", "--skip-stage-two", source=source)
        selftest.clear_calls()
        second = selftest.run("refused", "--skip-stage-two", source=source)
        summary = selftest.run("refused", "--summary", "--skip-stage-two", source=source)
    finally:
        selftest.write_mutants(selftest.mutants)
        selftest.write_plan(SELFTEST_PLAN)
    selftest.check(first.returncode == 0 and second.returncode == 0 and selftest.judged_sequences() == {"4"} and "现判 0、沿用 7、拒算 1" in second.stdout
                   and "REFUSED\t4\tinclude-argument" in summary.stdout,
                   f"拒算的条没有每趟现判、或别的条没沿用（第二趟现判的 {sorted(selftest.judged_sequences())}）：{second.stdout[-300:]}；汇总 {summary.stdout[-300:]}",
                   "看 Judgement.state：真跑时键里有拒算形态的条沿用不了（每趟现判，弄坏开关 refused-carried 下照沿用），--summary 逐条列 REFUSED")


def selftest_removed_cells(selftest):
    """已删的参数与旧库：--sync-tree、--commit、--changed-since、--milestone-full 退 2 写新跑法；有 tree.source / versions/ 的旧库目录拒。"""
    selftest.clear_calls()
    wrong = []
    for arguments in (["--sync-tree"], ["--sync-tree", "--commit", "HEAD"], ["--changed-since", "HEAD"], ["--milestone-full"], ["--changed-since=HEAD~1"]):
        ran = selftest.run("removed", *arguments)
        if not (ran.returncode == EXIT_USAGE and "参数已删" in ran.stderr and "不带选条参数就是剪枝" in ran.stderr):
            wrong.append(f"{' '.join(arguments)} 退 {ran.returncode}：{ran.stderr.strip()[-160:]}")
    old_home = os.path.join(selftest.work, "old-layout")
    os.makedirs(os.path.join(old_home, "versions", "0" * 64))
    old = selftest.run("old-layout", "--skip-stage-two")
    selftest.check(not wrong and not selftest.calls() and old.returncode == EXIT_USAGE and "旧库" in old.stderr,
                   f"已删的参数或旧库目录没拒（{'；'.join(wrong) or '参数那几样都拒了'}；旧库退 {old.returncode}：{old.stderr.strip()[-200:]}）",
                   "看 check_removed_arguments（REMOVED_ARGUMENTS 逐个退 2、出路写不带参数剪枝）与 resolve_home（OLD_LAYOUT_MARKERS 在就拒）")


def selftest_environmental_cells(selftest):
    """不带参数的剪枝跑也重跑超时、内存撞顶的条（它们不是定论，提交门禁算没结论，出路要走得通）：第 3 条超时、第 6 条撞顶，
    下一趟（源码没改）把这两条选中重跑、这一回抓到；别的条照沿用。"""
    selftest.write_plan([("3", "one-test", "exit:253"), ("6", "one-test", "exit:250")] + [(str(sequence), "one-test", "fail-nextest") for sequence in (1, 2, 4, 5, 7, 8)])
    first = selftest.run("environmental", "--skip-stage-two")
    before = selftest.run("environmental", "--summary")
    selftest.write_plan([(str(sequence), "one-test", "fail-nextest") for sequence in range(1, 9)])
    selftest.clear_calls()
    rerun = selftest.run("environmental", "--skip-stage-two")
    called = selftest.judged_sequences()
    states = selftest.states("environmental")
    selftest.write_plan(SELFTEST_PLAN)
    selftest.clear_calls()
    selftest.check(first.returncode == 0 and "NO_CONCLUSION\t3\ttimeout" in before.stdout and "NO_CONCLUSION\t6\tmemory-cap" in before.stdout
                   and "第 2b 步" in before.stderr and rerun.returncode == 0 and called == {"3", "6"}
                   and states.get("3") == "caught" and states.get("6") == "caught",
                   f"剪枝跑没有重跑超时、内存撞顶的条（头一趟退 {first.returncode}；第二趟退 {rerun.returncode}、现判的 {sorted(called)}；"
                   f"之后第 3 条 {states.get('3')}、第 6 条 {states.get('6')}）：{rerun.stdout[-300:]}{rerun.stderr[-300:]}",
                   "看 select_items：timeout / memory-cap 不是定论，照选中重跑（过过第一段的只补第二段）")


def selftest_gpu_memory_cells(selftest):
    """一步退 245（内存包装认出的显存不够，输出里照样有失败行）：是包装自己的结局，那一条不落行，整趟退 EXIT_INFRASTRUCTURE、报显存不够；
    显存空出来之后重跑同一条命令，那一条照常判。"""
    selftest.write_plan([("1", "one-test", "exit:245")])
    stopped = selftest.run("gpu-memory", "--item", "1", "--skip-stage-two")
    rows_after_stop = selftest.rows_of("gpu-memory", 1)
    selftest.write_plan(SELFTEST_PLAN)
    resumed = selftest.run("gpu-memory", "--item", "1", "--skip-stage-two")
    states = selftest.states("gpu-memory")
    selftest.clear_calls()
    selftest.check(stopped.returncode == EXIT_INFRASTRUCTURE and not rows_after_stop and "显存不够" in stopped.stderr
                   and resumed.returncode == 0 and states.get("1") == "caught",
                   f"一步退 245（显存不够）没当包装自己的结局停下（头一趟退 {stopped.returncode}、第 1 条落了 {len(rows_after_stop)} 行；"
                   f"重跑退 {resumed.returncode}、第 1 条 {states.get('1')}）：{stopped.stderr[-300:]}",
                   "看 WRAPPER_INFRASTRUCTURE_EXITS 与 stop_for_infrastructure：245 是内存包装认出的显存不够，输出里的失败不算这条变异的结局")


def selftest_rerun_larger_limit_cells(selftest):
    """证据不齐的超时、撞顶：第 2 条编译那一步超时、升档那趟撞顶（两趟结局不同，证据不齐）→ 照撞顶记、没结论、登记表不写；
    加大 --timeout-floor 与 --memory-cap 再 --item 2 --rerun：两样不进键，用的是加大的限时与上限，这一回判出结论。"""
    selftest.write_plan([("2", "one-build", "exit:253|exit:250"), ("2", "one-test", "fail-nextest")])
    first = selftest.run("larger", "--item", "2", "--skip-stage-two")
    first_rows = selftest.rows_of("larger", 2)
    first_state = selftest.states("larger").get("2")
    registry = os.path.join(selftest.work, "larger", ESCALATION_REGISTRY)
    registry_rows_after_first = len(read_table(registry, len(ESCALATION_COLUMNS)).rows)
    selftest.write_plan([("2", "one-test", "fail-nextest")])
    if os.path.exists(os.path.join(selftest.work, "limits.log")):
        os.remove(os.path.join(selftest.work, "limits.log"))
    rerun = selftest.run("larger", "--item", "2", "--skip-stage-two", "--rerun", "--timeout-floor", "60", "--memory-cap", "32G")
    limits = [entry for entry in selftest_limits(selftest) if entry[:2] == ("2", "one-build")]
    selftest.write_plan(SELFTEST_PLAN)
    selftest.check(first.returncode == 0 and first_state == "memory-cap" and registry_rows_after_first == 0
                   and first_rows and "证据不齐" in first_rows[-1]["note"]
                   and rerun.returncode == 0 and selftest.states("larger").get("2") == "caught" and limits and limits[-1][3:] == ("60", "32G"),
                   f"加大 --timeout-floor、--memory-cap 再 --item --rerun 没判出结论（头一趟退 {first.returncode}、状态 {first_state}、登记 {registry_rows_after_first} 行、"
                   f"备注 {first_rows[-1]['note'] if first_rows else '没有行'}；重跑退 {rerun.returncode}、状态 {selftest.states('larger').get('2')}、"
                   f"编译那一步的时限与上限 {limits[-1][3:] if limits else '没调'}）：{rerun.stderr[-400:]}",
                   "看 escalate_step（两趟结局不同是证据不齐，照环境型结局记）与 Judgement 的 globals（--timeout-floor、--memory-cap 不进键）")


def selftest_escalation_cells(selftest):
    """文件头「升档」：超时、撞顶的那一步当场升档重跑一次；两趟都超时 / 撞顶、证据齐的判抓到并登记，升档那趟另有结局的照它往后判，证据不齐的没结论，
    --summary 逐条列升档判的抓到；升档判的抓到进判定库、下一趟按键沿用。"""
    work = selftest.work
    selftest.write_plan([("0", "one-build", "peak:52428800"), ("0", "one-test", "peak:104857600"),
                         ("1", "one-build", "exit:253|exit:253"), ("2", "one-test", "exit:250|exit:250"),
                         ("3", "one-build", "exit:253|ok"), ("3", "one-test", "fail-nextest"),
                         ("4", "one-build", "exit:253|exit:250"), ("5", "one-test", "exit:250|fail-nextest")])
    for log_name in ("limits.log", "wrapper-env.log"):
        if os.path.exists(os.path.join(work, log_name)):
            os.remove(os.path.join(work, log_name))
    ran = selftest.run("escalation", *[word for sequence in "12345" for word in ("--item", sequence)], "--skip-stage-two", "--memory-cap", "1G")
    summary = selftest.run("escalation", "--summary")
    selftest.clear_calls()
    again = selftest.run("escalation", "--item", "1", "--item", "2", "--skip-stage-two", "--memory-cap", "1G")
    carried_again = not selftest.judged_sequences()
    selftest.write_plan(SELFTEST_PLAN)
    listed = selftest.listed("escalation")
    states = {sequence: entry["state"] for sequence, entry in listed.items()}
    rows = selftest.rows("escalation")
    last = {}
    for row in rows:
        if row["outcome"] in DECISIVE_OUTCOMES + ENVIRONMENTAL_OUTCOMES:
            last[row["sequence"]] = row
    registry = {row[1]: dict(zip(ESCALATION_COLUMNS, row)) for row in read_table(os.path.join(work, "escalation", ESCALATION_REGISTRY), len(ESCALATION_COLUMNS)).rows}
    limits = selftest_limits(selftest)

    def evidence_holds(entry, first_limit, escalated_limit):
        """登记行的两份日志在、sha256 对、末尾有驱动那一行，限时或上限是该升到的那个，键是那一条的键。"""
        if not entry or entry["key"] != listed.get(entry["sequence"], {}).get("key") or (entry["first_limit"], entry["escalated_limit"]) != (first_limit, escalated_limit):
            return False
        for attempt in ("first", "escalated"):
            path = os.path.join(work, "escalation", entry[f"{attempt}_log"])
            if not entry[f"{attempt}_log"].startswith(ESCALATION_DIRECTORY + "/") or not os.path.isfile(path) or file_digest(path) != entry[f"{attempt}_log_sha256"]:
                return False
            with open(path, encoding="utf-8") as handle:
                if f"{ESCALATION_EVIDENCE_PREFIX} key={entry['key']} sequence={entry['sequence']} step={entry['step']} attempt={attempt}" not in handle.read():
                    return False
        return True
    timeout_entry, memory_entry = registry.get("1"), registry.get("2")
    selftest.check(ran.returncode == 0 and states.get("1") == "caught" and last.get("1", {}).get("caught_by") == ESCALATED_TIMEOUT_LABEL
                   and evidence_holds(timeout_entry, "5", "20") and ("1", "one-build", "escalated", "20", "1G") in limits,
                   f"两趟都超时、证据齐的没判成 caught-by-timeout（退 {ran.returncode}；状态 {states.get('1')}；末行 {last.get('1')}；登记 {timeout_entry}）：{ran.stderr[-400:]}",
                   "看 escalate_step 超时那一支：限时升到 max(4 倍原限时, 20 倍基线)，两趟都退 253 而基线、日志都在，写登记表、判定行记 caught 带标签")
    selftest.check(states.get("2") == "caught" and last.get("2", {}).get("caught_by") == ESCALATED_MEMORY_LABEL
                   and evidence_holds(memory_entry, "1G", "1536M") and memory_entry.get("baseline") == "104857600" and ("2", "one-test", "escalated", "None", "1536M") in limits,
                   f"两趟都撞顶、证据齐的没判成 caught-by-memory（状态 {states.get('2')}；末行 {last.get('2')}；登记 {memory_entry}）",
                   "看 escalate_step 撞顶那一支：上限升到 max(1.5 倍原上限, 2 倍基线峰值)，基线峰值取 baselines.tsv 里这把基线键、这个线程数下那一步最后一行绿记录的 peak_bytes")
    selftest.check(states.get("3") == "caught" and last.get("3", {}).get("caught_by") == SELFTEST_NEXTEST_CASE and "升档重跑" in last.get("3", {}).get("note", "")
                   and states.get("5") == "caught" and last.get("5", {}).get("caught_by") == SELFTEST_NEXTEST_CASE and not {"3", "5"} & set(registry),
                   f"升档那趟跑完没照它往后的结局判（第 3 条 {last.get('3')}；第 5 条 {last.get('5')}；登记了 {sorted(registry)}）",
                   "看 process_item：升档那趟退 0 接着跑这一段后面的步，退测试没过照抓到判，都不写登记表")
    selftest.check(states.get("4") in ENVIRONMENTAL_OUTCOMES and "4" not in registry and "证据不齐" in last.get("4", {}).get("note", ""),
                   f"证据不齐的升档判成了抓到（第 4 条状态 {states.get('4')}、末行 {last.get('4')}；登记了 {sorted(registry)}）",
                   "看 escalation_evidence_problems：两趟结局不同、基线缺、限时或上限不到基线的 K 倍、日志里没有包装那一句，都是证据不齐，照环境型结局记")
    # --summary 的 NO_CONCLUSION 理由带上一趟的限时秒数（last_time_limit_text）：第 4 条编译步超时、升档那趟（限时 5 → 20）撞顶、证据不齐，没结论，
    # 上一趟就是升档那趟、限时 20 秒；第 2 条撞顶那一支判了抓到，不在 NO_CONCLUSION 里。弄坏开关 no-conclusion-limit-dropped 下不带，这一格判错
    no_conclusion_line = next((line for line in summary.stdout.splitlines() if line.startswith("NO_CONCLUSION\t4\t")), "")
    selftest.check(no_conclusion_line.count("\t") >= 4 and "上一趟限时 20 秒" in no_conclusion_line.split("\t", 4)[4],
                   f"--summary 的 NO_CONCLUSION 超时那一行理由里没带上一趟的限时秒数（第 4 条那一行 {no_conclusion_line!r}；末行备注 {last.get('4', {}).get('note', '')!r}）",
                   "看 last_time_limit_text：定下状态的那一行备注里「<步> 超过 <N> 秒」或「超时之后升档重跑（<原> → <升档>）」的那个数，接在 NO_CONCLUSION 的理由后面")
    labelled = {sequence: row["caught_by"] for sequence, row in last.items() if row["outcome"] == "caught" and row["caught_by"] in ESCALATED_LABELS}
    escalated_lines = {fields[1]: fields[2] for fields in (line.split("\t") for line in summary.stdout.splitlines()) if fields[0] == "ESCALATED" and len(fields) == 5}
    selftest.check(escalated_lines == labelled and summary_field(summary.stdout, "escalated") == len(labelled) and carried_again and again.returncode == 0,
                   f"--summary 没列升档判的抓到、或下一趟没按键沿用（判定库里 {labelled}，--summary 列了 {escalated_lines}、判定行 escalated={summary_field(summary.stdout, 'escalated')}；"
                   f"再点名第 1、2 条现判的 {sorted(selftest.judged_sequences())}）",
                   "看 summarise：状态是升档判的抓到的条逐条打 ESCALATED（带键），判定行带 escalated=<条数>；升档判的抓到进判定库带档位，键相同就沿用")
    # 文件头「内存」：升档那趟带 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP（按自己的上限排队），改坏了源码的头一趟带 RUN_WITH_MEMORY_CAP_OOM_FIRST，基线两样都不带
    wrapper_environment = []
    if os.path.exists(os.path.join(work, "wrapper-env.log")):
        with open(os.path.join(work, "wrapper-env.log")) as handle:
            wrapper_environment = [tuple(line.rstrip("\n").split("\t")) for line in handle if line.strip()]
    escalated_entries = [entry for entry in wrapper_environment if entry[2:3] == ("escalated",)]
    first_entries = [entry for entry in wrapper_environment if entry[0] not in ("0", ENVIRONMENT_SEQUENCE) and entry[2:3] == ("first",)]
    baseline_entries = [entry for entry in wrapper_environment if entry[0] == "0"]
    selftest.check(escalated_entries and all(entry[3:] == ("queue-by-cap", "-") for entry in escalated_entries)
                   and first_entries and all(entry[3:] == ("-", "oom-first") for entry in first_entries)
                   and baseline_entries and all(entry[3:] == ("-", "-") for entry in baseline_entries),
                   f"升档那趟没按自己的上限排队、或改坏了源码的那几步没设撞总上限时先挨杀（升档那趟 {escalated_entries}；头一趟 {first_entries}；基线 {baseline_entries}）",
                   "看 Runner.run_step：attempt 是 escalated 的设 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP，别的变异步设 RUN_WITH_MEMORY_CAP_OOM_FIRST，基线与验环境那几步两样都不设")


def selftest_escalation_evidence_cells(selftest):
    """文件头「升档」②③ 的五支（escalation_evidence_problems）各一格：只坏那一样的证据，要报那一样。对照是证据齐的一份，要报空表。"""
    good = dict(first_exit=TIME_LIMIT_HIT_EXIT, escalated_exit=TIME_LIMIT_HIT_EXIT, baseline=10.0, first_amount=900, escalated_amount=3600,
                first_text="900", escalated_text="3600", first_log_text="run-with-memory-cap: 超过限时 900 秒（自证）\n",
                escalated_log_text="run-with-memory-cap: 超过限时 3600 秒（自证）\n")
    control = escalation_evidence_problems(**good)
    selftest.check(control == [], f"证据齐的一份（两趟都 253、3600 对基线 10 秒、两份日志都带包装那一句）报了证据不齐：{control}",
                   "看 escalation_evidence_problems：五样都齐时交空表")
    cases = [("两趟结局不同", dict(good, escalated_exit=MEMORY_CAP_HIT_EXIT, escalated_log_text="run-with-memory-cap: 撞了内存上限 3600（自证）\n"), "两趟结局不同"),
             ("不到 K 倍", dict(good, baseline=200.0), "不到基线"),
             ("没比原来大", dict(good, escalated_amount=900, escalated_text="900", escalated_log_text="run-with-memory-cap: 超过限时 900 秒（自证）\n"), "没比原来的"),
             ("原来那趟缺包装那一句", dict(good, first_log_text="   Compiling singlefs-core（自证）\n"), "原来那趟的日志里没有包装打的"),
             ("升档那趟缺包装那一句", dict(good, escalated_log_text="   Compiling singlefs-core（自证）\n"), "升档那趟的日志里没有包装打的")]
    for what, given, wanted in cases:
        found = escalation_evidence_problems(**given)
        selftest.check(any(wanted in problem for problem in found),
                       f"升档的证据{what}没报出来（要有一条带「{wanted}」，实际 {found}）",
                       f"看 escalation_evidence_problems 里管「{what}」的那一支：只坏这一样也要报证据不齐")


def selftest_baseline_threads_cells(selftest):
    """基线按线程数分开记：--threads 2 下升档判的抓到登记了基线那一行的行号；换 --threads 3 之后基线在新线程数下重量、追加新行，
    旧登记行指的那一行照旧是它自己线程数、它记的那个值（门禁按行号复核就对得上），新登记行指新线程数那一行。"""
    work = selftest.work
    selftest.write_plan([("0", "one-test", "peak:104857600"), ("1", "one-build", "exit:253|exit:253"), ("2", "one-test", "exit:250|exit:250")])
    first = selftest.run("threads", "--item", "1", "--item", "2", "--skip-stage-two", "--threads", "2", "--memory-cap", "1G")
    selftest.write_plan([("0", "one-build", "sleep:0.3"), ("0", "one-test", "peak:209715200"), ("3", "one-build", "exit:253|exit:253")])
    second = selftest.run("threads", "--item", "3", "--skip-stage-two", "--threads", "3", "--memory-cap", "1G")
    selftest.write_plan(SELFTEST_PLAN)
    baseline = dict(read_baseline_rows(os.path.join(work, "threads", "baselines.tsv")))
    registry = [dict(zip(ESCALATION_COLUMNS, row)) for row in read_table(os.path.join(work, "threads", ESCALATION_REGISTRY), len(ESCALATION_COLUMNS)).rows]
    problems = []
    for entry in registry:
        row = baseline.get(int(entry["baseline_row"])) if entry["baseline_row"].isdigit() else None
        value_column = "seconds" if entry["label"] == ESCALATED_TIMEOUT_LABEL else "peak_bytes"
        if row is None or (row["baseline_key"], row["step"], row["outcome"], row["threads"], row[value_column]) != (entry["baseline_key"], entry["step"], "ok", entry["threads"], entry["baseline"]):
            problems.append(f"第 {entry['sequence']} 条登记的基线 {entry['baseline']}（线程 {entry['threads']}）与 baselines.tsv 第 {entry['baseline_row']} 行 {row} 对不上")
    threads_seen = {(row["step"], row["threads"]) for row in baseline.values() if row["outcome"] == "ok"}
    by_sequence = {entry["sequence"]: entry for entry in registry}
    selftest.check(first.returncode == 0 and second.returncode == 0 and not problems and {"1", "2", "3"} <= set(by_sequence)
                   and {("one-build", "2"), ("one-test", "2"), ("one-build", "3"), ("one-test", "3")} <= threads_seen
                   and by_sequence["3"]["threads"] == "3" and by_sequence["3"]["baseline_row"] != by_sequence["1"]["baseline_row"],
                   f"换了 --threads 之后基线没在新线程数下另记、或登记行指的基线行对不上（头一趟退 {first.returncode}、换线程那一趟退 {second.returncode}；"
                   f"基线表里的（步名, 线程数）{sorted(threads_seen)}；登记 {[(entry['sequence'], entry['threads'], entry['baseline_row'], entry['baseline']) for entry in registry]}；"
                   f"{'；'.join(problems)}）：{second.stderr[-300:]}",
                   "看 Runner.baseline_rows_of、ensure_baseline 与 escalate_step：基线按（基线键, 步名, 线程数）取最后一行，没有就在这个线程数下重量、追加一行（秒数与峰值），"
                   "登记行记它的行号（baseline_row）与基线键")


def selftest_baseline_key_cells(selftest):
    """基线键（文件头「基线」）：编译步拒算、每趟照跑；测试步的键按它跑到的测试的单元闭包取——改一个不在任何测试闭包里的函数（core 的 six），
    第一段测试那一步的基线照样沿用、不重跑；改抓到它的那条 harness 测试的函数体，那一步的基线重跑。弄坏开关 baseline-by-packages（包级）下这一格必须判错。"""
    core, catching = "crates/singlefs-core/src/lib.rs", "crates/singlefs-harness/tests/catching.rs"
    changed_core = SELFTEST_CORE.replace("pub fn six() -> u32 {\n", "pub fn six() -> u32 {\n    let _unused = 0;\n")
    source = selftest_source_copy(selftest, "baseline-key-source")
    selftest.write_plan([])
    try:
        selftest.clear_calls()
        first = selftest.run("baseline-key", "--item", "1", "--skip-stage-two", source=source)
        first_baselines = sorted(call[1] for call in selftest.calls() if call[0] == str(BASELINE_SEQUENCE))
        write_sources(source, {core: changed_core})
        selftest.write_mutants(selftest_mutants(dict(SELFTEST_SOURCES, **{core: changed_core})))
        selftest.clear_calls()
        outside = selftest.run("baseline-key", "--item", "1", "--skip-stage-two", "--rerun", source=source)
        outside_baselines = sorted(call[1] for call in selftest.calls() if call[0] == str(BASELINE_SEQUENCE))
        write_sources(source, {catching: SELFTEST_SOURCES[catching].replace("assert_eq!(1 + 1, 2);", "assert_eq!(2 + 2, 4);")})
        selftest.clear_calls()
        inside = selftest.run("baseline-key", "--item", "1", "--skip-stage-two", "--rerun", source=source)
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
    """按条记基线（文件头「基线」）：第一段测试步里 per_entry 一个测试二进制三条、refused_third 起自己这个可执行文件（闭包拒算）；checker 档快档库里两条
    （一条拒算）、tests/quick.rs 一条。头一趟都照跑、照条记下；改一个不在任何测试闭包里的函数（core 的 six）之后，第一段测试那一步只照跑 refused_third、
    别的三条沿用，checker 档快档只另起只跑拒算那一条的一趟（--exact），拼出来的逐条时间表里沿用的与现跑的都在；再改 kept_first 的断言，
    第一段测试那一步照跑它与 refused_third。弄坏开关 baseline-refused-step-whole（一条拒算整步照跑）下这一格必须判错。"""
    core = "crates/singlefs-core/src/lib.rs"
    changed_core = SELFTEST_CORE.replace("pub fn six() -> u32 {\n", "pub fn six() -> u32 {\n    let _unused = 0;\n")
    source = selftest_source_copy(selftest, "baseline-entry-source", SELFTEST_ENTRY_BASELINE_SOURCES)
    prepared = os.path.join(selftest.work, "runner-lines.tsv")
    refused_directory = os.path.join(selftest.work, "baseline-entry", "baselines", REFUSED_BASELINE_DIRECTORY)
    with open(prepared, "w") as handle:
        handle.write(SELFTEST_ENTRY_RUNNER_LINES)

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

    def assembled_times():
        """拒算的基线放逐条时间表的那一份里此刻的行：{步名: {(文件名, 用例): CPU 秒}}（下一趟开跑会清空它，取在两趟之间）。"""
        found = {}
        for name in sorted(os.listdir(refused_directory)) if os.path.isdir(refused_directory) else []:
            step_name = next((step for step in ("one-test", "two-test-checker-tier") if name.startswith("test-times.") and name.endswith(f".{step}.tsv")), None)
            if step_name:
                with open(os.path.join(refused_directory, name)) as handle:
                    found[step_name] = {tuple(line.split("\t")[:2]): line.split("\t")[2] for line in handle if line.strip()}
        return found

    selftest.write_plan([])
    try:
        selftest.clear_calls()
        first = selftest.run("baseline-entry", "--item", "4", source=source)
        first_selections = baseline_selections()
        write_sources(source, {core: changed_core})
        selftest.write_mutants(selftest_mutants(dict(SELFTEST_SOURCES, **{core: changed_core})))
        selftest.clear_calls()
        outside = selftest.run("baseline-entry", "--item", "4", "--rerun", source=source)
        outside_selections = baseline_selections()
        outside_times = assembled_times()
        write_sources(source, {SELFTEST_ENTRY_BASELINE_FILE: SELFTEST_ENTRY_BASELINE_SOURCES[SELFTEST_ENTRY_BASELINE_FILE].replace("assert_eq!(2 + 2, 4);", "assert_eq!(4 + 4, 8);")})
        selftest.clear_calls()
        inside = selftest.run("baseline-entry", "--item", "4", "--rerun", source=source)
        inside_selections = baseline_selections()
    finally:
        if os.path.exists(prepared):
            os.remove(prepared)
        selftest.write_mutants(selftest.mutants)
        selftest.write_plan(SELFTEST_PLAN)
    harness_binary = "singlefs-harness::per_entry"
    checker_parts = [selection.get("part") if isinstance(selection, dict) else selection for selection in first_selections.get("two-test-checker-tier", [])]
    inside_tests = inside_selections.get("one-test", [{}])
    selftest.check((first.returncode, outside.returncode, inside.returncode) == (0, 0, 0)
                   and first_selections.get("one-test") == ["all"] and checker_parts == ["whole", "skip", "exact"]
                   and outside_selections.get("one-test") == [{"tests": [[harness_binary, "refused_third"]], "binaries": []}]
                   and outside_selections.get("two-test-checker-tier") == [{"binaries": ["singlefs-checker-tier"], "part": "exact"}]
                   and "按条记基线" in outside.stdout
                   and outside_times.get("one-test", {}).get(("per_entry", "kept_first")) == "1.250"
                   and outside_times.get("one-test", {}).get(("per_entry", "refused_third")) == "0.750"
                   and outside_times.get("two-test-checker-tier", {}).get(("singlefs_checker_tier", "*")) == "2.500"
                   and outside_times.get("two-test-checker-tier", {}).get(("quick", "*")) == "1.000"
                   and len(inside_tests) == 1 and isinstance(inside_tests[0], dict)
                   and sorted(test for _binary, test in inside_tests[0].get("tests", [])) == ["kept_first", "refused_third"],
                   f"基线没按条记：一步里收得全的测试没各自沿用、或拒算的那几条没照跑（头一趟退 {first.returncode}、第一段测试步 {first_selections.get('one-test')}、"
                   f"checker 档快档 {checker_parts}；改了不在测试闭包里的 six 之后退 {outside.returncode}、第一段测试步 {outside_selections.get('one-test')}、"
                   f"checker 档快档 {outside_selections.get('two-test-checker-tier')}、拼出的时间表 {outside_times}；改了 kept_first 之后退 {inside.returncode}、"
                   f"第一段测试步 {inside_selections.get('one-test')}）：{outside.stdout[-400:]}{outside.stderr[-300:]}{inside.stderr[-300:]}",
                   "看 Judgement.baseline_entry_plan 与 Runner.ensure_entry_baseline：基线键拒算的测试步按条记（一条测试、checker 档一个二进制一份记录，键是各自的单元闭包键），"
                   "找得到记录的沿用它的 CPU 秒，只照跑找不到的与拒算的那几条，不整步照跑")


def selftest_alerts_path_in(environment):
    """在给定的环境里 scripts/hosts/core_usage.py 的 alerts_path() 交什么（另起一个 python 按文件路径导入它，不动自证进程自己的环境）。"""
    probe = ("import importlib.util, sys\n"
             "specification = importlib.util.spec_from_file_location('core_usage_probe', sys.argv[1])\n"
             "module = importlib.util.module_from_spec(specification)\n"
             "specification.loader.exec_module(module)\n"
             "print(module.alerts_path())\n")
    found = subprocess.run([sys.executable, "-c", probe, CORE_USAGE_LIBRARY], capture_output=True, text=True, stdin=subprocess.DEVNULL, env=environment)
    return found.stdout.strip() if found.returncode == 0 else f"<探针退 {found.returncode}：{found.stderr.strip()[-200:]}>"


def selftest_baseline_alerts_cells(selftest):
    """文件头「内存」：基线那几步交给一步的 SINGLEFS_CORE_USAGE_ALERTS 是 core_usage.py 的 alerts_path() 给的真记录（这一格把工作根换到工作目录里认），
    调用方先设了别的值的照调用方的，非基线的步不带它；自证每一趟都显式指到工作目录里的那一份，交给一步的没有一个是真工作根那份告警记录。"""
    work = selftest.work
    variable = selftest.alerts_variable
    real_record = selftest_alerts_path_in(os.environ)
    log_path = os.path.join(work, "alerts-env.log")

    def handed(home, environment):
        if os.path.exists(log_path):
            os.remove(log_path)
        ran = selftest.run(home, "--item", "1", "--skip-stage-two", environment=environment)
        entries = []
        if os.path.exists(log_path):
            with open(log_path) as handle:
                entries = [tuple(line.rstrip("\n").split("\t")) for line in handle if line.strip()]
        return ran, [entry for entry in entries if entry[0] == str(BASELINE_SEQUENCE)], [entry for entry in entries if entry[0] != str(BASELINE_SEQUENCE)]
    selftest.write_plan(SELFTEST_PLAN)
    work_root = os.path.join(work, "alerts-work-root")
    os.makedirs(work_root, exist_ok=True)
    unset_environment = dict(selftest.environment, SINGLEFS_WORK_ROOT=work_root)
    unset_environment.pop(variable, None)
    expected = selftest_alerts_path_in(unset_environment)
    unset_run, unset_baseline, unset_others = handed("baseline-alerts", unset_environment)
    selftest.check(unset_run.returncode == 0 and path_is_under(expected, work_root) and unset_baseline and all(entry[2] == expected for entry in unset_baseline),
                   f"基线那几步交给一步的 {variable} 不是 core_usage.py 的 alerts_path() 给的真记录（调用方没设、工作根换成 {work_root}；alerts_path() 交 {expected}；"
                   f"驱动退 {unset_run.returncode}；基线那几步拿到的 {unset_baseline}）：{unset_run.stderr[-400:]}",
                   f"看 Runner.run_step 基线那一支：环境里还没有 {variable} 的设成 real_alerts_record() 给的路径（core_usage.alerts_path()），"
                   "不然包装照峰值表把基线的告警写到基线峰值表旁边、叫不醒主 agent；BROAD_MUTATION_BREAK 设着 baseline-alerts-beside-peaks 的话本来就该红")
    selftest.check(unset_others and all(entry[2] == "-" for entry in unset_others),
                   f"非基线的步带了 {variable}（调用方没设；那几步拿到的 {unset_others}）",
                   f"看 Runner.run_step：只在基线那一支设 {variable}，变异步与验环境那几步照调用方的环境")
    caller_record = os.path.join(work, "caller-alerts.jsonl")
    caller_run, caller_baseline, caller_others = handed("baseline-alerts-caller", dict(selftest.environment, **{variable: caller_record}))
    selftest.check(caller_run.returncode == 0 and caller_baseline and all(entry[2] == caller_record for entry in caller_baseline),
                   f"调用方先设了 {variable}={caller_record}，基线那几步没照调用方的（驱动退 {caller_run.returncode}；基线那几步拿到的 {caller_baseline}）：{caller_run.stderr[-400:]}",
                   f"看 Runner.run_step 基线那一支：环境里已经有 {variable} 的不动它，自证与调用方靠它自己隔离")
    seen = {entry[2] for entry in unset_baseline + unset_others + caller_baseline + caller_others}
    isolated = selftest.environment.get(variable, "")
    selftest.check(path_is_under(isolated, work) and real_record not in seen and real_record != isolated,
                   f"自证交给一步的告警记录指到了真工作根那一份 {real_record}（自证的环境里 {variable}={isolated}；这一格交给各步的 {sorted(seen)}）",
                   f"看 Selftest.__init__：自证起的每一趟都显式把 {variable} 指到自证工作目录里的那一份，经真内存包装的那一格撞顶、挨挤也不写真记录")


def selftest_result_mac_cells(selftest):
    """判定行的校验列：驱动写的每一行都带对得上的校验列、密钥在库目录之外且权限 600；手追加的没有校验列的 caught 行、
    校验列对不上的 caught 行都不算，--summary 判红（退 1）、点到那几行；密钥不在时 --summary 判红、不替它建。"""
    work = selftest.work
    ran = selftest.run("mac", "--item", "1", "--item", "2", "--skip-stage-two")
    key_path = selftest.result_key_path()
    key, key_problem = read_result_key(key_path)
    lines = read_table(selftest.verdicts_path("mac"), len(VERDICT_COLUMNS)).rows
    bad = [row[1] for row in lines if key is None or row[-1] != hmac.new(key, "\t".join(row[:RESULT_SIGNED_COLUMNS]).encode(), hashlib.sha256).hexdigest()]
    mode = os.stat(key_path).st_mode & 0o777 if os.path.exists(key_path) else None
    selftest.check(ran.returncode == 0 and lines and not bad and mode == 0o600 and not path_is_under(key_path, os.path.join(work, "mac")),
                   f"驱动写的判定行没带对得上的校验列、或密钥的位置与权限不对（退 {ran.returncode}；{len(lines)} 行里校验对不上 {bad}；密钥 {key_path} 权限 "
                   f"{'不在' if mode is None else oct(mode)}，{key_problem}）：{ran.stderr[-300:]}",
                   "看 Library.record_verdict 与 result_key：每一行照 signed_result_row 带 HMAC-SHA256 校验列，密钥没有就在库目录之外建一份、权限 600")
    listed = selftest.listed("mac")

    def forged_row(sequence):
        return [listed.get(sequence, {}).get("key", "0" * 64), sequence, "singlefs-core", listed.get(sequence, {}).get("name", "?"), "caught", "one", "某条测试",
                "", "", "0.0", "0.0", today_text(), "fresh", "", "", ""]
    with open(selftest.verdicts_path("mac"), "a") as handle:
        handle.write("\t".join(forged_row("4")) + "\n" + "\t".join(forged_row("5") + ["0" * 64]) + "\n")
    summary = selftest.run("mac", "--summary")
    states = selftest.states("mac")
    selftest.check(summary.returncode == EXIT_INCONSISTENT and "校验列" in summary.stderr and states.get("4") != "caught" and states.get("5") != "caught",
                   f"手追加的没有校验列、校验列对不上的 caught 行被当成了判定（--summary 退 {summary.returncode}；第 4 条 {states.get('4')}、第 5 条 {states.get('5')}）："
                   f"{summary.stderr[-300:]}",
                   "看 read_signed_table：少一段的行与校验列对不上的行都不算、列进 forged，summarise 把它们报成对不上、退 1")
    missing_key = selftest.run("mac", "--summary", BROAD_MUTATION_RESULT_KEY=os.path.join(work, "no-such-key", "result.key"))
    selftest.check(missing_key.returncode == EXIT_INCONSISTENT and "不在" in missing_key.stderr and not os.path.exists(os.path.join(work, "no-such-key")),
                   f"密钥不在时 --summary 没判红、或替它建了一份（退 {missing_key.returncode}）：{missing_key.stderr[-300:]}",
                   "看 Library.result_key：只读的用法不建密钥，没有密钥时每一行都校验不了、照不算报")


def selftest_unit_environment_cells(selftest):
    """--unit-environment：会话里设了构建环境认的变量判过的条，经单元起（只带 detached-run.sh 那五个变量）就沿用不了；
    照 --unit-environment 打的那一行带进去就照沿用；不认的变量不打。"""
    session = dict(selftest.environment, RUSTFLAGS="-C session flag", CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C overflow-checks=off",
                   RUSTDOCFLAGS="--cfg session", SELFTEST_UNRELATED_VARIABLE="1")
    selftest.write_plan([("1", "one-test", "fail-nextest")])
    selftest.run("unitenv", "--item", "1", "--skip-stage-two", environment=session)
    printed = selftest.run("unitenv", "--unit-environment", environment=session)
    words = shlex.split(printed.stdout.strip()) if printed.returncode == 0 else []
    carried = dict(word.split("=", 1) for word in words[1:] if "=" in word) if words[:1] == ["env"] else {}
    selftest.clear_calls()
    unit = selftest.run("unitenv", "--item", "1", "--skip-stage-two", environment=dict(selftest.environment, **carried))
    carried_with = not selftest.judged_sequences()
    selftest.clear_calls()
    without = selftest.run("unitenv", "--item", "1", "--skip-stage-two")
    judged_without = selftest.judged_sequences()
    selftest.check(printed.returncode == 0 and {"RUSTFLAGS", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS", "RUSTDOCFLAGS"} <= set(carried)
                   and "SELFTEST_UNRELATED_VARIABLE" not in carried and not set(UNIT_CARRIED_VARIABLES) & set(carried)
                   and unit.returncode == 0 and carried_with and without.returncode == 0 and judged_without == {"1"},
                   f"--unit-environment 打的不是构建环境认的那几个变量（打了「{printed.stdout.strip()}」；带着它点名第 1 条沿用 {carried_with}、不带它现判的 {sorted(judged_without)}）："
                   f"{printed.stderr[-300:]}{unit.stderr[-300:]}",
                   "看 unit_environment_names：拿掉一个变量构建环境的生效值（build_environment_of）就变的才打，detached-run.sh 本来带的五个不打")
    # 工具链也比：会话里设了 RUSTUP_TOOLCHAIN，rustc -Vv 就换了一个；--unit-environment 要把它打出来，带着它照沿用
    fake_rustc = os.path.join(selftest.work, SELFTEST_RUSTC_NAME)
    with open(fake_rustc, "w") as handle:
        handle.write('#!/usr/bin/env bash\necho "rustc 1.0.0-selftest (${RUSTUP_TOOLCHAIN:-default})"\necho "host: x86_64-unknown-linux-gnu"\n')
    os.chmod(fake_rustc, 0o755)
    with_rustc = dict(selftest.environment, BROAD_MUTATION_RUSTC="1")
    toolchain_session = dict(with_rustc, RUSTUP_TOOLCHAIN="nightly-selftest")
    selftest.run("unittool", "--item", "1", "--skip-stage-two", environment=toolchain_session)
    printed = selftest.run("unittool", "--unit-environment", environment=toolchain_session)
    words = shlex.split(printed.stdout.strip()) if printed.returncode == 0 else []
    carried = dict(word.split("=", 1) for word in words[1:] if "=" in word) if words[:1] == ["env"] else {}
    selftest.clear_calls()
    unit = selftest.run("unittool", "--item", "1", "--skip-stage-two", environment=dict(with_rustc, **carried))
    carried_with = not selftest.judged_sequences()
    selftest.write_plan(SELFTEST_PLAN)
    selftest.check(printed.returncode == 0 and carried.get("RUSTUP_TOOLCHAIN") == "nightly-selftest" and not set(TEST_ONLY_VARIABLES) & set(carried)
                   and unit.returncode == 0 and carried_with,
                   f"--unit-environment 没把换了工具链的 RUSTUP_TOOLCHAIN 打出来（打了「{printed.stdout.strip()}」；带着它沿用 {carried_with}）：{printed.stderr[-300:]}{unit.stderr[-300:]}",
                   "看 unit_environment_names：拿掉一个变量 rustc -Vv（toolchain_text）就变的也打")


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
    first = selftest.run("fresh", "--item", "7", "--skip-stage-two", source=source)
    built_before = compiled()
    left_after_first = selftest.build_leftovers("fresh")
    if os.path.exists(os.path.join(work, "cargo.log")):
        os.remove(os.path.join(work, "cargo.log"))
    second = selftest.run("fresh", "--item", "7", "--skip-stage-two", "--rerun", source=source)    # 第 7 条的键没变，点名重跑才起基线
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
    failed = selftest.run("own-error", "--item", "1", "--skip-stage-two")
    left = selftest.build_leftovers("own-error")
    selftest.check(failed.returncode == EXIT_BASELINE_RED and not left,
                   f"跑完运行目录里不剩它建的编译目录与树副本（出错退出）：基线红那一趟退 {failed.returncode}（应当退 6），库目录里还剩 {left}：{failed.stderr[-400:]}",
                   "看 main 的 finally 里的 Library.end_run：拒绝（Refusal）退出也删这一趟的运行目录")
    holder_file = os.path.join(work, "holder.pid")
    if os.path.exists(holder_file):
        os.remove(holder_file)
    selftest.write_plan([("1", "one-test", "leave-holder")])
    held = selftest.run("own-holder", "--item", "1", "--skip-stage-two")
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


def selftest_import_cells(selftest):
    """两台各跑一片之后合并导入：核每行的段数、结局与键的写法；本机已有的不重复导；同一把键两边定论不同的不导；别的源码判的行并进来也不顶替本机这一份的。
    两台各用各的密钥（另一台那几份库用 peer_side_key_path 签），导入前交换密钥（exchange_peer_key）：校验列用另一台的密钥核过才导、照本机的密钥重签；
    另一台的行被改了一个字、取不到另一台的密钥，都拒、一行不写。"""
    work = selftest.work
    selftest.run("peer", "--shard", "1/2", "--skip-stage-two", BROAD_MUTATION_RESULT_KEY=selftest.peer_side_key_path())
    selftest.run("local", "--shard", "0/2", "--skip-stage-two")
    peer_home = os.path.join(work, "peer")
    peer_key, peer_key_problem = selftest.exchange_peer_key()
    imported = selftest.run("local", "--import-results", peer_home)
    summary = selftest.run("local", "--summary")
    again = selftest.run("local", "--import-results", peer_home)
    selftest.clear_calls()
    nothing_left = selftest.run("local", "--skip-stage-two")
    selftest.check(imported.returncode == 0 and "imported=4" in imported.stdout and summary.returncode == EXIT_GATE_RED
                   and "caught=1 missed=0 unviable=1 timeout=1 memory_cap=1 stage_one_only=4 driver_error=0 not_run=0" in summary.stdout
                   and again.returncode == 0 and "imported=0 already_present=4" in again.stdout
                   and nothing_left.returncode == 0 and selftest.judged_sequences() == {"3", "6"},
                   f"两台各一片的结果没合对（导入退 {imported.returncode}：{imported.stdout.splitlines()[:1]}；汇总 {summary.stdout.splitlines()[:1]}；"
                   f"再导一遍 {again.stdout.splitlines()[:1]}；之后剪枝跑现判的序号 {sorted(selftest.judged_sequences())}（只该是超时、撞顶的 3、6））：{imported.stderr}",
                   "看 import_results：核完才写、相同的行不重复导；导入之后状态照合起来的行算")
    local_key, _local_problem = read_result_key(selftest.result_key_path())
    brought = [[row[column] for column in VERDICT_COLUMNS] for row in selftest.rows("local") if row["method"].startswith("import:")]
    selftest.check(peer_key is not None and local_key is not None and peer_key != local_key and len(brought) == 4
                   and all(hmac.compare_digest(result_mac(local_key, fields), fields[-1]) for fields in brought)
                   and not any(hmac.compare_digest(result_mac(peer_key, fields), fields[-1]) for fields in brought),
                   f"另一台的密钥核过的行导进来之后没照本机的密钥重签（另一台的密钥{'在' if peer_key is not None else f'取不到：{peer_key_problem}'}、"
                   f"本机的密钥{'在' if local_key is not None else '不在'}、两把{'不同' if peer_key != local_key else '相同'}；导进来的 {len(brought)} 行）",
                   "看 import_results：校验列先用 peer_result_key 取的另一台密钥逐行核，核过去掉、record_verdicts 照本机的密钥重签")
    rows_before = len(selftest.rows("local"))

    def peer_copy(name, edit):
        directory = os.path.join(work, name)
        os.makedirs(directory)
        with open(os.path.join(peer_home, "verdicts.tsv")) as handle:
            lines = handle.read().split("\n")
        with open(os.path.join(directory, "verdicts.tsv"), "w") as handle:
            handle.write("\n".join(edit(lines)))
        return directory

    def break_first_key(lines):
        fields = lines[1].split("\t")
        fields[0] = "不是键"
        return [lines[0], selftest.peer_signed_line(fields)] + lines[2:]     # 照另一台的密钥重签：这一格核的是键的写法，不是校验列
    wrong_key = selftest.run("local", "--import-results", peer_copy("peer-wrong-key", break_first_key))
    rows_after_wrong = len(selftest.rows("local"))
    local_two = selftest.rows_of("local", 2)[0]

    def add_conflicting_row(lines):
        fields = [local_two[column] for column in VERDICT_COLUMNS[:RESULT_SIGNED_COLUMNS]]
        fields[VERDICT_COLUMNS.index("outcome")] = "caught"
        return lines[:-1] + [selftest.peer_signed_line(fields)] + lines[-1:]
    conflict = selftest.run("local", "--import-results", peer_copy("peer-conflict", add_conflicting_row))
    # 另一台的一行被改了一个字（变异名末尾换一个字），校验列没跟着改：用另一台的密钥核不过，整表拒、一行不写
    def tamper_one_character(lines):
        fields = lines[1].split("\t")
        name = VERDICT_COLUMNS.index("name")
        fields[name] = fields[name][:-1] + ("甲" if fields[name][-1:] != "甲" else "乙")
        return [lines[0], "\t".join(fields)] + lines[2:]
    rows_before_tamper = len(selftest.rows("local"))
    tampered = selftest.run("local", "--import-results", peer_copy("peer-tampered", tamper_one_character))
    selftest.check(tampered.returncode == EXIT_INCONSISTENT and "用另一台的密钥核不过" in tampered.stderr and len(selftest.rows("local")) == rows_before_tamper,
                   f"另一台的行被改了一个字（校验列没跟着改）时导入没整表拒（退 {tampered.returncode}，应当 {EXIT_INCONSISTENT}；判定行 {rows_before_tamper} → {len(selftest.rows('local'))}）："
                   f"{tampered.stdout[-300:]}{tampered.stderr[-300:]}",
                   "看 import_results 与 peer_mac_mismatches：每行的校验列用另一台的密钥核，一行核不过整表拒（退 1）、一行不写（弄坏开关 peer-mac-unchecked 下不核）")
    # 取不到另一台的密钥：拒导入（退 2）、一行不写，说明里点名本机存对端密钥的位置与交换的做法
    away = selftest.peer_key_path() + ".away"
    os.rename(selftest.peer_key_path(), away)
    try:
        keyless = selftest.run("local", "--import-results", peer_copy("peer-keyless", lambda lines: lines))
    finally:
        os.rename(away, selftest.peer_key_path())
    selftest.check(keyless.returncode == EXIT_USAGE and "另一台的密钥取不到" in keyless.stderr and selftest.peer_key_path() in keyless.stderr
                   and "两台交换密钥" in keyless.stderr and len(selftest.rows("local")) == rows_before_tamper,
                   f"取不到另一台的密钥时导入没拒（退 {keyless.returncode}，应当 {EXIT_USAGE}；判定行 {rows_before_tamper} → {len(selftest.rows('local'))}）：{keyless.stdout[-300:]}{keyless.stderr[-400:]}",
                   "看 peer_result_key：对端密钥在本机密钥同目录的 .peer.key，取不到就拒导入（退 2）、说明里点名那个位置与交换的做法（弄坏开关 peer-mac-unchecked 下不取）")
    # 别的源码（第 1 条那个函数的体改了）判的行并进来：键不同，本机第 1 条的状态不变，只多一条 stale
    foreign = selftest_source_copy(selftest, "foreign-source", {"crates/singlefs-core/src/lib.rs": SELFTEST_CORE.replace("pub fn one() -> u32 {\n", "pub fn one() -> u32 {\n    let _other = 0;\n")})
    selftest.write_mutants(selftest_mutants(dict(SELFTEST_SOURCES, **{"crates/singlefs-core/src/lib.rs": SELFTEST_CORE.replace("pub fn one() -> u32 {\n", "pub fn one() -> u32 {\n    let _other = 0;\n")})))
    try:
        selftest.run("foreign", "--item", "1", "--skip-stage-two", source=foreign, BROAD_MUTATION_RESULT_KEY=selftest.peer_side_key_path())
    finally:
        selftest.write_mutants(selftest.mutants)
    state_before = selftest.listed("local").get("1", {})
    other_source = selftest.run("local", "--import-results", os.path.join(work, "foreign"))
    state_after = selftest.listed("local").get("1", {})
    selftest.check(wrong_key.returncode == EXIT_INCONSISTENT and rows_after_wrong == rows_before
                   and conflict.returncode == EXIT_INCONSISTENT and "IMPORT_CONFLICT\t" in conflict.stdout and selftest.states("local").get("2") == "unviable"
                   and other_source.returncode == 0 and state_after.get("state") == state_before.get("state") and state_after.get("stale") == "stale=1",
                   f"该拒的导入没拒、或别的源码的行顶替了本机的（键写坏的退 {wrong_key.returncode}、之后 {rows_after_wrong} 行对 {rows_before} 行；"
                   f"定论不同的退 {conflict.returncode}；别的源码那份退 {other_source.returncode}、第 1 条 {state_before} → {state_after}）：{wrong_key.stderr}{conflict.stdout}",
                   "看 import_results：每行的段数、结局、键的写法全部核完才写；同一把键两边都有定论而不同的不导、列出来、退 1；别的源码的键与本机的不同，并进来只是多一行")
    selftest_gpu_import_cells(selftest)     # 导入的 GPU 判定行那几格接在这一组后面（不另列进 SELFTEST_CELL_NAMES：模块级的赋值在判法单元指纹的闭包里，改它库里的判定就一条都不沿用）


def selftest_ignored_cells(selftest):
    """忽略用例登记：树里每条 #[ignore] 用例不是慢档用例（进第二段）就在排除表里；新加的没登记、排除表指着不存在的或写坏的，判红；第二段那一步的命令带排除的 -E、步名换了、旧基线行不认。"""
    work = selftest.work
    cases_file = "crates/singlefs-format/tests/ignored_cases.rs"
    green = selftest.run("ignored", "--check-ignored")
    command_line = next((line for line in green.stdout.splitlines() if line.startswith("STEP_COMMAND\t")), "")
    selftest.check(green.returncode == 0 and "timed=2 excluded=3 unregistered=0 stale=0 both=0 unparsed=0" in green.stdout
                   and f"\t{SLOW_STEP_NAME}\t" in command_line and "--run-ignored only -E " in command_line
                   and all(term in command_line for term in ("binary_id(=singlefs-format) & test(=tests::in_lib_tool)",
                                                              "binary_id(=singlefs-format::ignored_cases) & test(=env_large_tier)",
                                                              "binary_id(=singlefs-format::ignored_cases) & test(=inner::env_tool)"))
                   and "slow_one" not in command_line and "in_lib_slow" not in command_line,
                   f"登记对的树上 --check-ignored 没判绿、或第二段那一步的命令不对（退 {green.returncode}）：{green.stdout}{green.stderr}",
                   "看 classify_ignored_cases（按缩进配模块、算二进制 id）、exclusion_filter 与 step_table：慢档用例进第二段，排除表的每一条进 -E")
    # 旧步名的基线行不认：库里先塞一行「two-test-ignored 绿」，两段都跑的一条要在没改过的树上把新步名的基线重量
    os.makedirs(os.path.join(work, "ignored"), exist_ok=True)
    append_row(os.path.join(work, "ignored", "baselines.tsv"), BASELINE_COLUMNS, ["0" * 64, OLD_SLOW_STEP_NAME, "ok", "1.0", now_text(), default_threads(), ""])
    selftest.clear_calls()
    both_stages = selftest.run("ignored", "--item", "4")
    calls = selftest.calls()
    selftest.check(both_stages.returncode == 0 and ("0", SLOW_STEP_NAME, "pristine") in calls and not any(call[1] == OLD_SLOW_STEP_NAME for call in calls)
                   and [row["outcome"] for row in selftest.rows_of("ignored", 4)] == [STAGE_ONE_ONLY, "missed"],
                   f"旧步名的基线行被认了（退 {both_stages.returncode}；调用 {calls}）：{both_stages.stderr}",
                   "看 SLOW_STEP_NAME 与 Runner.ensure_baseline：命令改了的一步换了步名，基线表里旧名的行就不认、照新步名重量")
    # 红：新加两条没登记的（一条理由是大档、一条光秃秃的 #[ignore]）。两段都跑的趟在基线之前拒、一步不调；只跑第一段的趟照跑
    source = selftest_source_copy(selftest, "ignored-new-source", {cases_file: SELFTEST_SOURCES[cases_file]
                                  + '\n#[test]\n#[ignore = "大档：新加的、没登记"]\nfn new_env_tier() {}\n\n#[test]\n#[ignore]\nfn bare_ignore() {}\n'})
    unregistered = selftest.run("ignored-new", "--check-ignored", source=source)
    selftest.clear_calls()
    refused = selftest.run("ignored-new", "--item", "4", source=source)
    calls_when_refused = selftest.calls()
    stage_one = selftest.run("ignored-new", "--item", "4", "--skip-stage-two", source=source)
    selftest.check(unregistered.returncode == EXIT_INCONSISTENT and "unregistered=2" in unregistered.stdout
                   and "既不是" in unregistered.stderr and refused.returncode == EXIT_INCONSISTENT and "既不是" in refused.stderr and not calls_when_refused
                   and stage_one.returncode == 0 and [row["outcome"] for row in selftest.rows_of("ignored-new", 4)] == [STAGE_ONE_ONLY],
                   f"新加的没登记的忽略用例没判红（--check-ignored 退 {unregistered.returncode}；两段都跑退 {refused.returncode}、调了 {len(calls_when_refused)} 步；只跑第一段退 {stage_one.returncode}）："
                   f"{unregistered.stdout}{unregistered.stderr}{refused.stderr}",
                   "看 check_ignored_registration 与 run_items：要跑第二段的趟在基线之前核一遍，两样都不是的判红退 1；只跑第一段的趟不判")

    def with_rows(name, extra):
        path = os.path.join(work, name)
        with open(selftest.exclusions_path) as handle:
            text = handle.read()
        with open(path, "w") as handle:
            handle.write(text + extra)
        return selftest.run("ignored", "--check-ignored", BROAD_MUTATION_EXCLUSIONS=path)

    stale = with_rows("exclusions-stale.tsv", "singlefs-format::ignored_cases no_such_test\t自证：树里没有的用例\n")
    both = with_rows("exclusions-both.tsv", "singlefs-format::ignored_cases slow_one\t自证：理由写着慢档用例的又被排除\n")
    selftest.check(stale.returncode == EXIT_INCONSISTENT and "no_such_test" in stale.stderr and "stale=1" in stale.stdout
                   and both.returncode == EXIT_INCONSISTENT and "slow_one" in both.stderr and "both=1" in both.stdout,
                   f"排除表指着不存在的用例、或两边都在，没判红（不存在的退 {stale.returncode}：{stale.stderr}；两边都在退 {both.returncode}：{both.stderr}）",
                   "看 classify_ignored_cases 的 stale 与 both：排除表的每一行都要在树里找得到对应的 #[ignore] 用例，理由写着慢档用例的不许同时在排除表里")
    garbled = with_rows("exclusions-garbled.tsv", "这一行没有制表符\n")
    duplicate = with_rows("exclusions-duplicate.tsv", "singlefs-format::ignored_cases env_large_tier\t自证：重复的一行\n")
    selftest.check(garbled.returncode == EXIT_INCONSISTENT and "两段" in garbled.stderr
                   and duplicate.returncode == EXIT_INCONSISTENT and "重复" in duplicate.stderr and "env_large_tier" in duplicate.stderr,
                   f"排除表写坏的行或重复的行没判红（写坏的退 {garbled.returncode}：{garbled.stderr}；重复的退 {duplicate.returncode}：{duplicate.stderr}）",
                   "看 load_exclusions：不是两段、头不是两个词、理由是空、重复的行都拒（退 1），不悄悄略过")


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
    relayed = subprocess.run([sys.executable, "-c", relay, *selftest.command("first", "--item", "2", "--skip-stage-two")], capture_output=True, text=True,
                             stdin=subprocess.DEVNULL, env=selftest.environment)
    selftest.check(relayed.returncode == EXIT_USAGE and not selftest.calls() and "不是 --selftest 自己起的子进程" in relayed.stderr
                   and selftest.states("first").get("2") == NOT_RUN,
                   f"父进程不是 --selftest 的进程带着假一步的变量时没拒（退 {relayed.returncode}，调了 {len(selftest.calls())} 步，第 2 条状态 {selftest.states('first').get('2')}）：{relayed.stderr}",
                   "看 test_mode 与 resolve_home：只供自证的变量与弄坏开关只在父进程是带 --selftest 的这份脚本时认，这一判要排在「库目录不在临时目录」之前")
    selftest.clear_calls()
    rows_before = len(selftest.rows("first"))
    break_only = {name: value for name, value in selftest.environment.items() if name not in TEST_ONLY_VARIABLES}
    break_only["BROAD_MUTATION_BREAK"] = ",".join(part for part in (os.environ.get("BROAD_MUTATION_BREAK", ""), "rerun-done") if part)
    relayed_break = subprocess.run([sys.executable, "-c", relay, *selftest.command("first", "--item", "2", "--skip-stage-two")], capture_output=True, text=True,
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
                   "看 main 开头的 check_test_only_variables_before_parsing：它要排在 check_removed_arguments 与 parse_arguments 之前")


def selftest_progress_cells(selftest):
    """一步跑过进度间隔要往 stderr 报进度：间隔换成 0.3 秒、第 1 条的测试那一步跑 1.5 秒，要有点名这一条这一步的行，其中有一行数到了日志里已过的那一条用例。"""
    selftest.write_plan([("1", "one-test", "sleep:1.5")])
    ran = selftest.run("progress", "--item", "1", "--skip-stage-two", BROAD_MUTATION_PROGRESS_INTERVAL="0.3")
    selftest.write_plan(SELFTEST_PLAN)
    lines = [line for line in ran.stderr.splitlines() if line.startswith("BROAD_MUTATION_PROGRESS sequence=1 step=one-test ")]
    selftest.check(ran.returncode == 0 and lines and any(" tests_passed=1 tests_failed=0" in line for line in lines),
                   f"一步跑过进度间隔没有报进度（退 {ran.returncode}；点名第 1 条 one-test 的进度行 {lines[:3]}）：{ran.stderr[-600:]}",
                   "看 Runner.wait_reporting_progress 与 progress_line：一步跑着时每过 PROGRESS_INTERVAL_SECONDS 秒往 stderr 打一行，数日志里已过的用例")


def selftest_stop_cells(selftest):
    """被 TERM 停：在途那一步连同内存包装给它开的 scope 一起停（这一格的假一步经真的内存包装起，要用户级 systemd）。
    照常的包装被 TERM 时自己停 scope，另一趟给包装设它的弄坏开关 term-leaves-scope（改前的包装），证驱动自己停 scope、不靠包装。"""
    work = selftest.work
    selftest.slice_used = True
    wrapper_environment = dict(selftest.environment, BROAD_MUTATION_STEP_VIA_WRAPPER="1", RUN_WITH_MEMORY_CAP_SLICE=selftest.slice,
                               RUN_WITH_MEMORY_CAP_SLICE_TOTAL="256M", RUN_WITH_MEMORY_CAP_STATE_DIR=os.path.join(work, "admission"),
                               RUN_WITH_MEMORY_CAP_PEAKS=os.path.join(work, "memory-peaks.tsv"),
                               RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="broad-mutation 自证里的假一步，上限是写死的 64M")
    wrapper_environment.pop("CLAUDE_CODE_SESSION_ID", None)
    selftest.write_plan([("1", "one-test", "hang")])
    pid_file = os.path.join(work, "hang.pids")
    if os.path.exists(pid_file):
        os.remove(pid_file)
    driver = subprocess.Popen(selftest.command("stop", "--item", "1", "--skip-stage-two", "--timeout-floor", "120"), stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                              text=True, stdin=subprocess.DEVNULL, env=wrapper_environment)
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
                   "看 main 的 finally：被 TERM / INT 停（Stopped）也走 Library.end_run，在途那一步连同 scope 停下之后删这一趟的运行目录")
    selftest.write_plan(SELFTEST_PLAN)
    selftest.clear_calls()
    rerun = subprocess.run(selftest.command("stop", "--item", "1", "--skip-stage-two", "--lock-wait", "0"), capture_output=True, text=True,
                           stdin=subprocess.DEVNULL, env=wrapper_environment)
    open(os.path.join(work, "release"), "w").close()
    rows_of_one = [row["outcome"] for row in selftest.rows_of("stop", 1)]
    selftest.check(hanging and os.path.exists(os.path.join(work, "hanging")) and driver.returncode == EXIT_STOPPED and len(step_processes) >= 2 and all_stopped
                   and rerun.returncode == 0 and rows_of_one == ["caught"],
                   f"被 TERM 停时在途那一步没有一起停（那一步{'挂住了' if hanging else '没起来'}；驱动退 {driver.returncode}；那一步的进程 {step_processes}、"
                   f"{'都停了' if all_stopped else '还有活着的'}；紧接着重跑退 {rerun.returncode}、第 1 条各行 {rows_of_one}）：{errors}{rerun.stderr}",
                   "看 Runner.stop_step：给包装发 TERM 之外，还要停它开的那一个 scope（own_scope_unit 找、systemctl --user stop）。"
                   "驱动退 6、日志里是 251：这台机器起不了带上限的 scope（没有用户级 systemd），真跑同样起不来，先修那一头")

    # 包装自己不停 scope 的那一种（改前的包装：起了 scope 之后被 TERM 只删标记与账就退，命令在 scope 里照跑）：同一个假一步经真的内存包装起，给包装设它自己的弄坏开关
    # RUN_WITH_MEMORY_CAP_BREAK=term-leaves-scope。包装只在它自证的标记在时认这个开关（run-with-memory-cap.sh 文件头「弄坏开关只在自证里认」）：照那一段建一份标记文件、
    # 经 RUN_WITH_MEMORY_CAP_SELFTEST_MARK 带过去。只给包装发 TERM 时在途那一步停不下，驱动另停包装开的那一个 scope 才停得下：弄坏开关 term-leaves-step-running 在这一格红。
    memory_cap_mark = os.path.join(work, "memory-cap-selftest-mark")
    with open(memory_cap_mark, "w") as handle:
        handle.write("run-with-memory-cap.sh --selftest 起的包装\n")
    scope_kept_environment = dict(wrapper_environment, RUN_WITH_MEMORY_CAP_BREAK="term-leaves-scope", RUN_WITH_MEMORY_CAP_SELFTEST_MARK=memory_cap_mark)
    for name in ("release", "hanging", "hang.pids"):
        if os.path.exists(os.path.join(work, name)):
            os.remove(os.path.join(work, name))
    selftest.write_plan([("1", "one-test", "hang")])
    driver = subprocess.Popen(selftest.command("stop-scope-kept", "--item", "1", "--skip-stage-two", "--timeout-floor", "120"), stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE, text=True, stdin=subprocess.DEVNULL, env=scope_kept_environment)
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
    open(os.path.join(work, "release"), "w").close()   # 停不下的那一步（弄坏开关下）见到它就退，不留到自证收尾
    wait_until(lambda: not any(process_is_running(process_id) for process_id in step_processes), 10)
    selftest.write_plan(SELFTEST_PLAN)
    selftest.check(hanging and driver.returncode == EXIT_STOPPED and len(step_processes) >= 2 and all_stopped,
                   f"包装自己不停 scope 时被 TERM 停，在途那一步没有一起停（包装带 RUN_WITH_MEMORY_CAP_BREAK=term-leaves-scope，改前的包装；"
                   f"那一步{'挂住了' if hanging else '没起来'}；驱动退 {driver.returncode}；那一步的进程 {step_processes}、{'都停了' if all_stopped else '还有活着的'}）：{errors[-600:]}",
                   "看 Runner.stop_step：给包装发 TERM 之外，驱动自己停包装开的那一个 scope（own_scope_unit 找、systemctl --user stop），不靠包装自己停；"
                   "驱动退 2、日志里点名 RUN_WITH_MEMORY_CAP_BREAK：标记文件的那一行与 run-with-memory-cap.sh 的 SELFTEST_MARK_CONTENT 对不上了，照它改这里")


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
    命令的写法（特性、目标、清场、--test-threads 1、BINDGEN）；超时 / 无效 / 一个测试都没跑到；--skip-gpu、清场没过、中途没卡、基线红；复用与 --rerun；
    手追加的 GPU 判定行不算；--skip-stage-two 不跑；--list-items 的状态是两趟合起来的。假的一步当假 GPU（SELFTEST_STEP_DRIVER 的 gpu- 那几步）。"""
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
                       "看 run_gpu_phase：只给 CPU 那一趟落的结论是没红的条起 GPU 那一趟（一条两趟里任一趟红算抓到，CPU 红了就不用跑）；GPU 的结局落 gpu-verdicts.tsv，"
                       "不进 verdicts.tsv，BROAD_MUTATION_GPU_ITEM 一行分开写 cpu= 与 gpu=")
        selftest.check(set(os.listdir(os.path.join(selftest.work, "gpu-cpu-red"))) >= {GPU_VERDICTS, "verdicts.tsv"}
                       and {(row["sequence"], row["outcome"]) for row in selftest.rows("gpu-cpu-red")} == {("1", "caught"), ("2", STAGE_ONE_ONLY), ("2", "missed")},
                       "GPU 那一趟的结局没有单独存进 gpu-verdicts.tsv（verdicts.tsv 里只该有 CPU 那一趟的三行）", "看 record_gpu_verdict：GPU 趟的判定行落 gpu-verdicts.tsv，库里 CPU 那一趟的 verdicts.tsv 一行不改")

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
        # 命令的写法：特性是 tests/ 里 cfg 点到的、目标是够得着的那几个、测试步经清场包装并加 --nocapture --test-threads 1、编译步不经、BINDGEN_EXTRA_CLANG_ARGS 取崩溃放量那一份、每步有整步限时
        bindgen = gpu_bindgen_arguments()
        build_commands = [json.loads(entry[3]) for entry in environment if entry[1] == GPU_BUILD_STEP]
        test_commands = [json.loads(entry[3]) for entry in environment if entry[1] == GPU_TEST_STEP]
        right_test = ["bash", QUIESCE_RUNNER, "--", "cargo", "test", "--locked", "--release", "-p", CHECKER_TIER_PACKAGE, "--features", "gpu", "--test", "gpu_pair",
                      "--", "--nocapture", "--test-threads", "1"]
        right_build = ["cargo", "test", "--locked", "--release", "-p", CHECKER_TIER_PACKAGE, "--features", "gpu", "--test", "gpu_pair", "--no-run"]
        limits = {(entry[0], entry[1]): entry[4] for entry in environment}
        selftest.check(bindgen and len(environment) == 2 * (1 + len(covered)) and build_commands == [right_build] * (1 + len(covered)) and test_commands == [right_test] * (1 + len(covered))
                       and all(entry[2] == bindgen for entry in environment) and limits[(GPU_BASELINE_SEQUENCE, GPU_BUILD_STEP)] == "None"
                       and all(limits[(sequence, step)].isdigit() and int(limits[(sequence, step)]) >= 5 for sequence in covered for step in (GPU_BUILD_STEP, GPU_TEST_STEP)),
                       f"GPU 那一趟的命令不对（BINDGEN {bindgen[:40]!r}；收到 {len(environment)} 步；编译步 {build_commands[:1]}；测试步 {test_commands[:1]}；限时 {sorted(limits.items())[:4]}）",
                       "看 gpu_pass_command：编译步 cargo test --locked --release -p singlefs-checker-tier --features <特性> <目标> --no-run；测试步整条经 scripts/crash/gpu-quiesce-run.sh -- 起，"
                       "后面加 -- --nocapture --test-threads 1；BINDGEN_EXTRA_CLANG_ARGS 取 gpu_bindgen_arguments（crash-amplification-lib.sh 的那一份）；基线不设限时、逐条设整步限时")

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

        # --skip-gpu：一步 GPU 都不起，要这一趟的条逐条本次未跑
        ran = gpu_run("gpu-skipped", [], *two, "--skip-gpu")
        selftest.check(ran.returncode == 0 and not selftest.gpu_calls() and not selftest.gpu_rows("gpu-skipped")
                       and sorted(entry["sequence"] for entry in gpu_lines_of(ran.stdout, "BROAD_MUTATION_GPU_NOT_RUN")) == ["1", "2"],
                       f"--skip-gpu 还是起了 GPU 的步、或没逐条报本次未跑（GPU 步 {selftest.gpu_calls()}）：{ran.stdout[-300:]}", "看 run_gpu_phase 里的 skip_gpu")

        # 清场没过（清场包装退 78、测试没起）：同样本次未跑，不拒、不记没红
        ran = gpu_run("gpu-quiesce-refused", [("gpu-baseline", GPU_TEST_STEP, "exit:78")], *two)
        not_run = gpu_lines_of(ran.stdout, "BROAD_MUTATION_GPU_NOT_RUN")
        selftest.check(ran.returncode == 0 and not selftest.gpu_rows("gpu-quiesce-refused") and sorted(entry["sequence"] for entry in not_run) == ["1", "2"]
                       and all(entry["reason"] == "quiesce-refused" for entry in not_run),
                       f"清场没过（退 78）没报成本次未跑（退 {ran.returncode}；{not_run}）：{ran.stdout[-300:]}{ran.stderr[-300:]}",
                       "看 gpu_classify_test 的退 78：清场回读没过、命令没起，是 not-run，出路是等卡上那个不归清场白名单的进程跑完再起，不停它")

        # 基线红：没改过的树上 GPU 那一趟就不绿 ⇒ 拒（退 6）、一条 GPU 行都不落、树回到原样；CPU 那一趟已落的行在
        ran = gpu_run("gpu-baseline-red", [("gpu-baseline", GPU_TEST_STEP, "gpu-fail")], *two)
        selftest.check(ran.returncode == EXIT_BASELINE_RED and "GPU 趟基线" in ran.stderr and not selftest.gpu_rows("gpu-baseline-red") and {(row["sequence"], row["outcome"]) for row in selftest.rows("gpu-baseline-red")} == {("1", STAGE_ONE_ONLY), ("1", "missed"), ("2", STAGE_ONE_ONLY), ("2", "missed")}
                       and selftest.gpu_judged_sequences() == set() and "--skip-gpu" in ran.stderr,
                       f"GPU 趟基线红没拒成「基线不绿」（退 {ran.returncode}）：{ran.stderr[-400:]}", "看 GpuPass.ensure_baseline：没改过的树上编译红、测试红、一个测试都没跑到，退 6 并写出路")

        # 超时、一个测试都没跑到、编不过：没结论的（超时、驱动错）不当定论，编不过是无效
        ran = gpu_run("gpu-timeout", [("1", GPU_TEST_STEP, "exit:253"), ("2", GPU_TEST_STEP, "gpu-zero")], *two)
        gpu_rows = {row["sequence"]: row["outcome"] for row in selftest.gpu_rows("gpu-timeout")}
        summary = summary_of("gpu-timeout")
        reasons = {line.split("\t")[1]: line.split("\t")[-1] for line in summary.stdout.splitlines() if line.startswith("NO_CONCLUSION\t")}
        selftest.check(ran.returncode == 0 and gpu_rows == {"1": "timeout", "2": DRIVER_ERROR} and reasons.get("1") == "gpu-timeout" and reasons.get("2") == f"gpu-{DRIVER_ERROR}"
                       and gate_fields(summary.stdout).get("gpu_without_conclusion") == "2" and summary_field(summary.stdout, "missed") == 0,
                       f"GPU 趟超时 / 一个测试都没跑到不是没结论（GPU 行 {gpu_rows}；理由 {reasons}）：{ran.stdout[-300:]}{ran.stderr[-300:]}",
                       "看 gpu_classify_test：退 253 是 timeout、退 0 而一个测试都没跑到是驱动错，都不是没红；汇总里它们是没结论（gpu_without_conclusion=）")
        ran = gpu_run("gpu-unviable", [("1", GPU_BUILD_STEP, "fail-compile")], *two)
        gpu_rows = {row["sequence"]: row["outcome"] for row in selftest.gpu_rows("gpu-unviable")}
        selftest.check(ran.returncode == 0 and gpu_rows == {"1": "unviable", "2": "missed"} and selftest.states("gpu-unviable", source).get("1") == "unviable",
                       f"GPU 趟编不过没记成无效（GPU 行 {gpu_rows}）：{ran.stdout[-300:]}{ran.stderr[-300:]}", "看 gpu_classify_build 与 judge_gpu_item：编译步退 101 又有带源码位置的编译错误才是无效")

        # 复用：同一条命令再跑，GPU 那一趟一步都不起（沿用得了）；带 --rerun 点名的那一条重跑并追加新行
        selftest.write_plan([])
        selftest.clear_calls()
        reused = selftest.run("gpu-neither", source=source)
        selftest.check(reused.returncode == 0 and not selftest.gpu_calls() and not selftest.judged_sequences() and "BROAD_MUTATION_GPU_PLAN needed=4 judge=0 carried=4 " in reused.stdout
                       and "GPU 趟（要 4 条）：现判 0、复用 4、本次未跑 0" in reused.stdout and len(selftest.gpu_rows("gpu-neither")) == 4,
                       f"同一条命令第二趟 GPU 那一趟没复用（GPU 步 {selftest.gpu_calls()[:3]}）：{reused.stdout[-400:]}{reused.stderr[-300:]}",
                       "看 GpuJudging.state 与 row_holds：键相同、gpu_key 相同、没红的覆盖者键相同才沿用；run_gpu_phase 里沿用得了的不选")
        rerun = selftest.run("gpu-neither", "--item", "2", "--rerun", source=source)
        selftest.check(rerun.returncode == 0 and selftest.gpu_judged_sequences() == {"2"} and len([row for row in selftest.gpu_rows("gpu-neither") if row["sequence"] == "2"]) == 2,
                       f"点名加 --rerun 没有重跑 GPU 那一趟并追加新行（GPU 步 {selftest.gpu_calls()}）：{rerun.stdout[-300:]}{rerun.stderr[-300:]}",
                       "看 run_gpu_phase：--rerun 下沿用得了的也现判，GPU 判定库只追加")
        # --skip-stage-two：GPU 趟接在第二段后面，不起
        ran = gpu_run("gpu-stage-one", [], *two, "--skip-stage-two")
        selftest.check(ran.returncode == 0 and not selftest.gpu_calls() and "--skip-stage-two，不跑" in ran.stdout and not selftest.gpu_rows("gpu-stage-one"),
                       f"--skip-stage-two 还是起了 GPU 那一趟（{selftest.gpu_calls()[:2]}）", "看 run_gpu_phase 开头的 skip_stage_two")
        # 手追加的 GPU 判定行不算：少一段（没有校验列）与校验列对不上的两行，--summary 判红退 1、点名 GPU 判定库
        forged_path = gpu_verdicts_path(Library(os.path.join(selftest.work, "gpu-neither")))
        genuine_rows = selftest.gpu_rows("gpu-neither")
        if not genuine_rows:
            # 前面几格红了、gpu-neither 里一行 GPU 判定都没有：这一格判不了，记判错，不在这里抛出去（后面几组的格照判）
            selftest.check(False, "手追加的没有校验列、校验列对不上的 GPU 判定行没判红（前提不成立：gpu-neither 里一行 GPU 判定都没有，照上面几格先修）",
                           "看 run_gpu_phase：gpu-neither 那几趟要落 GPU 判定行，这一格拿其中一行造手追加的行")
        else:
            genuine = genuine_rows[0]
            fields = [genuine[column] if column != "outcome" else "caught" for column in GPU_VERDICT_COLUMNS[:GPU_SIGNED_COLUMNS]]
            append_row(forged_path, GPU_VERDICT_COLUMNS, fields)
            append_row(forged_path, GPU_VERDICT_COLUMNS, fields + ["0" * 64])
            forged = summary_of("gpu-neither")
            selftest.check(forged.returncode == EXIT_INCONSISTENT and "GPU 判定库" in forged.stderr and "校验列" in forged.stderr,
                           f"手追加的没有校验列、校验列对不上的 GPU 判定行没判红（退 {forged.returncode}）：{forged.stderr[-300:]}",
                           "看 read_gpu_verdicts 与 summarise 的 problems：GPU 判定行同判定行一样带密钥算的校验列，对不上的不算判定、--summary 退 1")
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
    changed_body = 'return GpuOutcome("timeout", "", f"{GPU_TEST_STEP} ' + '超时")'
    if original.count(marker) != 1 or original.count(changed_body) != 1:
        selftest.check(False, "GPU 趟判法单元指纹那一格找不到要改的两处（gpu_classify_test 的开头与超时那一行）", "看 selftest_gpu_judging_cells 里的 marker 与 changed_body：gpu_classify_test 改了写法就跟着改")
        return
    before_gpu, before_cpu = keys_now(True)
    with open(driver_copy, "w", encoding="utf-8") as handle:
        handle.write(original.replace(marker, "    # 自证：只加一行注释\n" + marker, 1).replace("\nimport argparse\n", "\n# 自证：文件里再加一行注释\nimport argparse\n", 1))
    commented_gpu = keys_now(False)[0]
    with open(driver_copy, "w", encoding="utf-8") as handle:
        handle.write(original.replace(changed_body, 'return GpuOutcome("timeout", "", f"{GPU_TEST_STEP} ' + '超时（自证改了判结局的函数）")', 1))
    body_gpu, body_cpu = keys_now(True)
    selftest.check(re.fullmatch(r"[0-9a-f]{64}", before_gpu) and commented_gpu == before_gpu,
                   f"只改驱动注释 GPU 趟的判法单元指纹变了（改前 {before_gpu[:16]}、只改注释之后 {commented_gpu[:16]}）",
                   "看 gpu_judging_key：取 GPU_JUDGING_ENTRIES 那几个函数的 python 单元闭包键（注释与文档串不进指纹），不取整份原文")
    selftest.check(re.fullmatch(r"[0-9a-f]{64}", body_gpu) and body_gpu != before_gpu,
                   f"改了 GPU 趟判结局的函数（gpu_classify_test 的体）GPU 趟的判法单元指纹没变（{before_gpu[:16]} → {body_gpu[:16]}）",
                   "看 GPU_JUDGING_ENTRIES：命令怎么拼、步结局怎么分、沿用怎么判、两趟怎么合那几个函数都要在里面")
    selftest.check(re.fullmatch(r"[0-9a-f]{64}", before_cpu) and body_cpu == before_cpu,
                   f"改了 GPU 趟的函数，CPU 那一趟的判法单元指纹跟着变了（{before_cpu[:16]} → {body_cpu[:16]}）：库里 CPU 那一趟的判定会一条都不沿用",
                   "GPU 那一节的函数不许被 JUDGING_ENTRIES 的闭包点到（process_item、Runner.escalate_step 等不改、不调 GPU 那一节）；CPU 那一趟的判法单元指纹只管 CPU 那一趟")


def selftest_gpu_import_cells(selftest):
    """两台各跑一片时，第二台的 GPU 判定行随 --import-results 并进本机库（文件头「--import-results」）：第二台跑奇数片（序号 1、5 要 GPU 那一趟）、本机跑偶数片（2、4），
    ① 导入之后本机 GPU 趟现判 0、复用 4（等于本机自己判的 2 条加导进来的 2 条），再导一遍一行不加；② 一行键对不上本机此刻的源码、一行 GPU 趟自己的键对不上：被拒、列出来、
    退 1，对得上的那一行照导，之后本机的 GPU 趟只把被拒的那一条照现判；③ 一行结局认不出：整表拒，verdicts.tsv 的行也不导；④ 两边定论不同的一把键不导、列出来、退 1。"""
    work = selftest.work
    source = selftest_source_copy(selftest, "gpu-source", SELFTEST_GPU_FILES)
    column = {name: index for index, name in enumerate(GPU_VERDICT_COLUMNS)}
    peer_home = os.path.join(work, "gpu-import-peer")
    local_home = os.path.join(work, "gpu-import-local")

    def gpu_run(home, *arguments):
        selftest.write_plan([])
        selftest.clear_calls()
        return selftest.run(home, *arguments, source=source)

    def import_fields(output):
        found = gpu_lines_of(output, "BROAD_MUTATION_IMPORT")
        return found[0] if found else {}

    def row_of(rows, sequence):
        return next(fields for fields in rows if fields[column["sequence"]] == str(sequence))

    def peer_copy(name, edit):
        """另一台库目录的一份拷贝：verdicts.tsv 照原样，gpu-verdicts.tsv 的行换成 edit 给的（每行是字段表；edit 拿到的是原来的各行）。"""
        directory = os.path.join(work, name)
        os.makedirs(directory)
        shutil.copyfile(os.path.join(peer_home, "verdicts.tsv"), os.path.join(directory, "verdicts.tsv"))
        with open(os.path.join(peer_home, GPU_VERDICTS), encoding="utf-8") as handle:
            lines = handle.read().split("\n")
        original = [line.split("\t") for line in lines[1:] if line]
        with open(os.path.join(directory, GPU_VERDICTS), "w", encoding="utf-8") as handle:
            # 改过的行照另一台的密钥重签：这几格核的是键、结局与定论，不是校验列（校验列那两格在 selftest_import_cells）
            handle.write("\n".join([lines[0]] + ["\t".join(selftest.peer_signed_gpu_fields(fields)) for fields in edit(original)]) + "\n")
        return directory

    try:
        selftest.write_plan([])
        selftest.clear_calls()
        peer_run = selftest.run("gpu-import-peer", "--shard", "1/2", source=source, BROAD_MUTATION_RESULT_KEY=selftest.peer_side_key_path())
        selftest.exchange_peer_key()
        local_run = gpu_run("gpu-import-local", "--shard", "0/2")
        peer_sequences = sorted(row["sequence"] for row in selftest.gpu_rows("gpu-import-peer"))
        local_sequences = sorted(row["sequence"] for row in selftest.gpu_rows("gpu-import-local"))
        prepared = peer_run.returncode == 0 and local_run.returncode == 0 and peer_sequences == ["1", "5"] and local_sequences == ["2", "4"]
        selftest.check(prepared,
                       f"两台各跑一片的 GPU 导入格前提不成立（第二台那一片退 {peer_run.returncode}、GPU 行 {peer_sequences}，应当 [1, 5]；本机那一片退 {local_run.returncode}、GPU 行 {local_sequences}，应当 [2, 4]）："
                       f"{peer_run.stderr[-300:]}{local_run.stderr[-300:]}",
                       "看 run_gpu_phase 与 in_run_scope：--shard k/2 只给序号 mod 2 == k 的条跑两趟，CPU 没红又有带 gpu 特性的测试够得着的条（序号 1、2、4、5）各落一行 GPU 判定")
        if not prepared:
            return
        for name in ("gpu-import-local-bad", "gpu-import-local-malformed"):
            shutil.copytree(local_home, os.path.join(work, name))

        # ① 第二台的 GPU 判定行随导入并进本机库：之后本机的 GPU 趟一条都不现判，复用数 = 本机自己判的 + 导进来的
        imported = selftest.run("gpu-import-local", "--import-results", peer_home, source=source)
        fields = import_fields(imported.stdout)
        brought = sorted((row["sequence"], row["method"]) for row in selftest.gpu_rows("gpu-import-local") if row["method"].startswith("import:"))
        merged = sorted(row["sequence"] for row in selftest.gpu_rows("gpu-import-local"))
        again = selftest.run("gpu-import-local", "--import-results", peer_home, source=source)
        again_fields = import_fields(again.stdout)
        plain = gpu_run("gpu-import-local")
        selftest.check(imported.returncode == 0 and (fields.get("gpu_rows_read"), fields.get("gpu_imported"), fields.get("gpu_rejected")) == ("2", "2", "0")
                       and brought == [("1", "import:fresh"), ("5", "import:fresh")] and merged == ["1", "2", "4", "5"]
                       and again.returncode == 0 and (again_fields.get("gpu_imported"), again_fields.get("gpu_already_present")) == ("0", "2")
                       and plain.returncode == 0 and not selftest.gpu_calls() and not selftest.judged_sequences()
                       and "BROAD_MUTATION_GPU_PLAN needed=4 judge=0 carried=4 " in plain.stdout and "GPU 趟（要 4 条）：现判 0、复用 4、本次未跑 0" in plain.stdout
                       and len(selftest.gpu_rows("gpu-import-local")) == 4,
                       f"第二台的 GPU 判定行没随导入并进本机库、或并进来之后本机的 GPU 趟还是把同一批判了一遍（导入退 {imported.returncode}：{imported.stdout[-400:]}；并进来的 {brought}、合起来 {merged}；"
                       f"再导一遍 {again_fields}；之后不带 --shard 的一趟退 {plain.returncode}、GPU 步 {selftest.gpu_calls()[:2]}）：{imported.stderr[-300:]}{plain.stdout[-300:]}",
                       "看 import_results 与 plan_gpu_import：另一台库目录里的 gpu-verdicts.tsv 的行核过键（行的键在本机此刻清单的键里、gpu_key 等于 GpuJudging.pass_key）就并进本机的 gpu-verdicts.tsv，"
                       "跑法加 import: 前缀、照本机密钥重签，本机已有的相同行不重复导；本机的 GPU 趟读的就是 gpu-verdicts.tsv，键相同、gpu_key 相同的行沿用")

        # ② 一行键对不上、一行 GPU 趟自己的键对不上：被拒、列出来、退 1；对得上的那一行照导；之后本机的 GPU 趟只把被拒的那一条照现判
        def mixed(rows):
            stale = list(row_of(rows, 1))
            stale[column["key"]] = "a" * 64
            other_pass = list(row_of(rows, 5))
            other_pass[column["gpu_key"]] = "b" * 64
            return [stale, row_of(rows, 5), other_pass]
        mixed_peer = peer_copy("gpu-import-peer-mixed", mixed)
        bad = selftest.run("gpu-import-local-bad", "--import-results", mixed_peer, source=source)
        bad_fields = import_fields(bad.stdout)
        reasons = sorted(line.split("\t")[3] for line in bad.stdout.splitlines() if line.startswith("IMPORT_GPU_REJECTED\t"))
        kept = sorted(row["sequence"] for row in selftest.gpu_rows("gpu-import-local-bad"))
        rejudged = gpu_run("gpu-import-local-bad")
        selftest.check(bad.returncode == EXIT_INCONSISTENT and (bad_fields.get("gpu_rows_read"), bad_fields.get("gpu_imported"), bad_fields.get("gpu_rejected")) == ("3", "1", "2")
                       and reasons == ["gpu-pass-changed", "key-not-in-list"] and kept == ["2", "4", "5"]
                       and rejudged.returncode == 0 and selftest.gpu_judged_sequences() == {"1"}
                       and "BROAD_MUTATION_GPU_PLAN needed=4 judge=1 carried=3 " in rejudged.stdout,
                       f"键对不上的 GPU 判定行没被拒、或拒了之后本机的 GPU 趟没把它照现判（导入退 {bad.returncode}，应当 {EXIT_INCONSISTENT}；{bad_fields}；被拒的理由 {reasons}；导入之后本机的 GPU 行 {kept}，应当 [2, 4, 5]；"
                       f"之后一趟现判过 {sorted(selftest.gpu_judged_sequences())}，应当 [1]）：{bad.stderr[-400:]}{rejudged.stdout[-300:]}",
                       "看 plan_gpu_import：行的键不在本机此刻清单的单元键里记 key-not-in-list，gpu_key 不等于 GpuJudging.pass_key() 记 gpu-pass-changed，这两种不导、IMPORT_GPU_REJECTED 逐条列出、整条导入退 1；"
                       "对得上的行照导，被拒的那一条在本机没有 GPU 判定，之后补跑 GPU 趟时照现判")

        # ③ 一行结局认不出：整表拒、退 1，verdicts.tsv 的行也一行不导
        def unknown_outcome(rows):
            broken = list(row_of(rows, 1))
            broken[column["outcome"]] = "不是结局"
            return [broken, row_of(rows, 5)]
        malformed_peer = peer_copy("gpu-import-peer-malformed", unknown_outcome)
        verdict_rows_before = len(selftest.rows("gpu-import-local-malformed"))
        gpu_rows_before = len(selftest.gpu_rows("gpu-import-local-malformed"))
        malformed = selftest.run("gpu-import-local-malformed", "--import-results", malformed_peer, source=source)
        selftest.check(malformed.returncode == EXIT_INCONSISTENT and "认不出" in malformed.stderr
                       and len(selftest.rows("gpu-import-local-malformed")) == verdict_rows_before and len(selftest.gpu_rows("gpu-import-local-malformed")) == gpu_rows_before,
                       f"不成形的 GPU 判定行没整表拒、或拒了还写了别的行（退 {malformed.returncode}；判定行 {verdict_rows_before} → {len(selftest.rows('gpu-import-local-malformed'))}；"
                       f"GPU 行 {gpu_rows_before} → {len(selftest.gpu_rows('gpu-import-local-malformed'))}）：{malformed.stderr[-300:]}",
                       "看 plan_gpu_import：段数不是十四、结局认不出、键不是 64 位十六进制的整表拒，且排在 import_results 写任何一行（含 verdicts.tsv 的行）之前")

        # ④ 同一把键两边都有定论而不同：这一把键的行不导、列出来、退 1
        def opposite(rows):
            opposed = list(row_of(rows, 5))
            opposed[column["outcome"]] = "caught"
            opposed[column["caught_by"]] = SELFTEST_GPU_CASE
            opposed[column["test_key"]] = "c" * 64
            opposed[column["coverage_key"]] = ""
            return [opposed]
        conflict_peer = peer_copy("gpu-import-peer-conflict", opposite)
        rows_before = len(selftest.gpu_rows("gpu-import-local"))
        conflict = selftest.run("gpu-import-local", "--import-results", conflict_peer, source=source)
        conflict_fields = import_fields(conflict.stdout)
        selftest.check(conflict.returncode == EXIT_INCONSISTENT and "IMPORT_GPU_CONFLICT\t" in conflict.stdout
                       and (conflict_fields.get("gpu_conflicting_keys"), conflict_fields.get("gpu_imported")) == ("1", "0") and len(selftest.gpu_rows("gpu-import-local")) == rows_before,
                       f"两台定论不同的 GPU 判定行没挡下（退 {conflict.returncode}；{conflict_fields}；GPU 行 {rows_before} → {len(selftest.gpu_rows('gpu-import-local'))}）：{conflict.stdout[-300:]}{conflict.stderr[-300:]}",
                       "看 plan_gpu_import：同一把键本机与另一台各有一条定论（caught / missed / unviable）、结局不同，这一把键的行不导，IMPORT_GPU_CONFLICT 列出来，整条导入退 1")
    finally:
        selftest.write_plan(SELFTEST_PLAN)
        selftest.clear_calls()


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
    "selftest_removed_cells",
    "selftest_environmental_cells",
    "selftest_gpu_memory_cells",
    "selftest_basic_cells",
    "selftest_state_cells",
    "selftest_table_cells",
    "selftest_environment_cells",
    "selftest_sync_cells",
    "selftest_rerun_larger_limit_cells",
    "selftest_escalation_cells",
    "selftest_escalation_evidence_cells",
    "selftest_baseline_threads_cells",
    "selftest_baseline_key_cells",
    "selftest_baseline_entry_cells",
    "selftest_baseline_alerts_cells",
    "selftest_result_mac_cells",
    "selftest_unit_environment_cells",
    "selftest_import_cells",
    "selftest_ignored_cells",
    "selftest_test_variable_cells",
    "selftest_progress_cells",
    "selftest_stop_cells",
    "selftest_rocksdb_entry_cells",
)


def run_selftest_cells(selftest, only=None):
    """自证的各组格，照 SELFTEST_CELL_NAMES 的次序；only 给了只跑点名的几组（环境变量 BROAD_MUTATION_SELFTEST_CELLS，开发时分段跑用：有的组接着前面组留下的库目录，单跑要连前面那一组一起点）。"""
    for name in SELFTEST_CELL_NAMES:
        if only is None or name in only:
            globals()[name](selftest)


def selftest_rocksdb_entry_cells(selftest):
    """真跑起 cargo 之前判 RocksDB 预编库的取值：被判的树里 checker 档包依赖 rocksdb、进程环境里四个取值是空的 ⇒ 拒（退 2）、假的一步一次都没起、报出 entry 的红句。
    别的格的树（SELFTEST_SOURCES）没有依赖 rocksdb 的清单，entry 报不判。"""
    tree = selftest_source_copy(selftest, "rocksdb-source", {"crates/singlefs-checker-tier/Cargo.toml": selftest_package("singlefs-checker-tier")
                                                            + '\n[dependencies]\nrocksdb = { version = "=0.25.0", optional = true }\n'})
    calls_before = len(selftest.calls())
    refused = selftest.run("rocksdb", source=tree, ROCKSDB_LIB_DIR="", ROCKSDB_STATIC="", SNAPPY_LIB_DIR="", SNAPPY_STATIC="")
    selftest.check(refused.returncode == EXIT_USAGE and len(selftest.calls()) == calls_before and "rocksdb-prebuilt entry：" in refused.stdout + refused.stderr,
                   f"被判的树里 checker 档包依赖 rocksdb 而进程环境里没有预编库的取值，没在起 cargo 之前拒（退 {refused.returncode}，"
                   f"假的一步起了 {len(selftest.calls()) - calls_before} 次）：{(refused.stdout + refused.stderr)[-600:]}",
                   "看 main 里 check_real_run_is_allowed 之后的 check_rocksdb_entry（scripts/hosts/rocksdb-prebuilt.py entry）")


def run_selftest(only=None):
    unknown = [name for name in (only or []) if name not in SELFTEST_CELL_NAMES]
    if unknown:
        refuse(f"BROAD_MUTATION_SELFTEST_CELLS 点了不认得的组：{'、'.join(unknown)}", f"只认这几组：{'、'.join(SELFTEST_CELL_NAMES)}")
    work = tempfile.mkdtemp(prefix="broad-mutation-selftest.")
    selftest = Selftest(work)
    try:
        run_selftest_cells(selftest, only)
    finally:
        open(os.path.join(work, "release"), "w").close()   # 挂住的那一步（有的话）见到它就退
        time.sleep(0.2)
        if selftest.slice_used:
            # 自证自己开的那一个 slice（名字带这个进程的进程号）：停掉，不留着
            subprocess.run(["systemctl", "--user", "stop", selftest.slice], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False)
        shutil.rmtree(work, ignore_errors=True)
    if selftest.failures:
        print(f"  ✗ --selftest：{selftest.cells} 格里 {selftest.failures} 格判错", file=sys.stderr)
        print(f"     → 怎么办：照上面每一格给的出路改，再跑 python3 {SELF_PATH} --selftest", file=sys.stderr)
        return 1
    say("  ✓ --selftest：临时库目录拒、生成清单与列项、项写错拒、点名与两段、杀掉再起只补没做完的、各种结局与汇总八栏、第二段只补第一段没跑到的、--from 与 --shard、"
        "上一趟被 KILL 留下的运行目录开跑先删、点名跳过沿用得了的条与 --rerun、定论不被降级、--stage-two-only 要过过第一段、清单按单元存的表改坏判红、末行没写完、认不出的退出码、"
        "一步退 245（显存不够）停下不落行、重起验编译与连着无效或秒红、恢复原件半途被杀之后下一趟照常判、工具链进每一把键、"
        "不带参数就是剪枝（沿用数 > 0）、改了一个函数只现判它里面的条（同文件在某提交之后改过别处照样沿用）、--full 只写不读与翻面、"
        "加一条排除 / 只改 Cargo.toml 注释 / 加大限时照样沿用、改 RUSTFLAGS 全部现判、只改驱动注释判法单元指纹不变、拒算的条每趟现判、已删的参数与旧库拒、"
        "剪枝跑重跑超时与内存撞顶的条、--summary 的 NO_CONCLUSION 理由带上一趟限时（升档那趟的）、mutants.toml 写 exclude_re 之后清单少那一条且整版重记而别的条沿用、加大限时与上限再点名重跑判出结论、超时与撞顶自动升档（证据齐的判抓到并登记、升档那趟跑完照它往后判、证据不齐的没结论、"
        "--summary 逐条列、升档判的抓到按键沿用、升档那趟按自己的上限排队、变异步撞总上限时先挨杀）、升档证据五支各一格、基线按线程数分开记且登记行指的那一行对得上、"
        "基线键按测试的单元闭包取（改不在测试闭包里的函数测试步基线照沿用、改测试就重跑、编译步拒算照跑）、"
        "基线键拒算的测试步按条记基线（收得全的测试各自沿用、只照跑拒算与改过的那几条，checker 档拒算的那几条另起 --exact）、"
        "基线那几步的告警照旧写真记录（调用方设了的照调用方的、别的步不带）、"
        "判定行带密钥校验列（手追加的不算）、--unit-environment 打构建环境与工具链认的会话变量、每一趟自己的编译目录（不照用上一趟的产物）、跑完运行目录里不剩编译目录与树副本（正常、出错、被 TERM 停三种结局都删，有进程开着就不删、报出来、退 9）、合并导入、"
        "第二台的 GPU 判定行随导入并进本机库（导入之后本机 GPU 趟现判 0、键对不上的行被拒并照现判、不成形的整表拒、两边定论不同的不导）、"
        "忽略用例登记与排除表（没登记、指着不存在、两边都在、写坏、命令带 -E、旧基线行不认）、只供自证的变量不出自证、一步跑过进度间隔要报进度、被 TERM 连 scope 一起停、"
        "GPU 那一趟（只有 CPU 红 / 只有 GPU 红 / 两趟都没红 / 没卡 / 清场没过 / 中途没卡 / --skip-gpu / 基线红 / 超时与无效与一个测试都没跑到、命令的写法、复用与 --rerun、"
        "手追加的 GPU 判定行不算、--skip-stage-two 不跑、GPU 趟判法单元指纹与 CPU 的分开、复用三问与复用哨兵带上 GPU 趟），"
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
