#!/usr/bin/env python3
# admission: always 判的是此刻被判的仓、它的 crates/mutations.tsv 与 git common-dir 里的判定库记录，上一次的结论不替这一次作保
# run-condition: none 算判法版本与行键要的 git、cargo、rustc 由 admission.py 的 git_common_directory 与本模块的 toolchain_text 起、起不来报错退 2；llvm-profdata、llvm-cov 由 run_stage 现找、找不到判红；--selftest 用自己造的假 cargo、假 llvm 工具与假内存包装，不编译
"""门禁 checker-tier-crates-mutation-replay（crates 变异表复跑）的判法与判定库记录，连同门禁 code-source-discipline 静态判的那几样：一份判法只在这里，两道阶段都调它。

变异表一行成形 = 六段制表符分隔（变异名、文件、原文、替换文、cargo test 参数、必须红的测试名）；替换文可以空（删掉原文：改坏时原文换成空串），
其余五段一段都不许空（malformed_reason，read_table 照它读；checker-tier-crates-mutation-replay、static-check、import 同一份）。

checker-tier-crates-mutation-replay 经 import 调 run_stage（阶段文件里那几行内嵌 python）；这里不给整张表开命令行入口：重型测试闸（.claude/hooks/heavy-test-guard.sh）
按阶段文件名认 checker-tier-crates-mutation-replay，另开一个入口它认不出。

子命令（<根> 是被判仓的根）：
  static-check <根> [--table <变异表>]  checker-tier-crates-mutation-replay 派活之前判的每一样（row_problems，checker-tier-crates-mutation-replay 的 prechecks 调的是同一个函数）连同拷贝范围 ②，
                                        每处一行（制表符分隔），退 0 一处都没有、退 1 有、退 2 读不了登记表或变异表：
                                          MALFORMED <行号> <变异名> <为什么>             这一行不成形（见上）
                                          ANCHOR   <行号> <变异名> <为什么>              原文在「文件」那一段指的源码里不是恰好命中一次，或文件不在、读不了
                                          NAME     <行号> <变异名> <测试名>              必须红的测试名不是一个用例名（带了正则符号或空白）
                                          ROW      <行号> <变异名> <文件>                ① 变异表这一行改的文件不在 checker-tier-crates-mutation-replay 每片拷的范围里
                                          STRAY    <行号> <变异名> <段名> <转义>         原文或替换文里有 \\n 以外的反斜杠转义
                                          LITERAL  <文件> <行号> <字面量> <解析到的路径> ② crates/ 下 .rs 里以 "../ 起头、按所在 crate 的根解析之后
                                                                                         逃出 crates/、又不在拷贝范围里的字符串字面量（只 code-source-discipline 判，checker-tier-crates-mutation-replay 靠基线）
                                          COPIED   <拷贝范围，空格分隔>
                                          CHECKED  <成形的行数> <.rs 份数> <逃出 crates/ 的字面量处数（在范围里的与不在的合计）>
                                        门禁 code-source-discipline 调它；--table 不给就读 <根>/crates/mutations.tsv（不在当空表）；
                                        参数不对（缺 <根>、--table 缺值、<根> 不是目录）与 --table 指的文件不在都打 ERROR 行（原因）、
                                        下一行「→ 怎么办」，退 2
  judge-version <根>              这棵树此刻的判法版本（见「判定库」）：打「<sha256> <项数>」；算不出退 2
  records-directory <根>                这棵树此刻那一格判定库记录目录的路径（双机分片的驱动拷记录用）；算不出退 2
  row-plan <根>                         双机分片的驱动（scripts/mutation/mutation-shard-run.sh）分行用：每一条成形的变异打一行
                                          <行号> <行键> <这一趟会不会复用，1 或 0> <上一次现跑的秒数，没记过写 ->
                                        （制表符分隔；「会不会复用」与 run_stage 同一份判法 reuse_decision，SINGLEFS_GATE_FULL=1 或 GATE_MUTATION_START_OVER=1 时一律 0）；
                                        退 0；读不了表、上限写错、算不出判法版本退 2
  plan <根>                             只读：不编、不跑、不写，按现有记录逐行打「<行号>\\t<变异名>\\t复用|现跑\\t<栏>\\t<细节>」（栏见「复用判据」），
                                        末行「合计 <N> 行：按覆盖复用 <M> 行、现跑 <K> 行（<各栏的数>）」。退 0；读不了表、上限写错、算不出判法版本退 2
  import <根> <别处的记录目录>          把另一台写的判定库记录导进本机：逐条核文件名与内容对得上、判法版本等于本机此刻算的、行键是本机变异表里某一行
                                        （连同本机这一趟的每条内存上限、限时与工具链）算得出的，再照本机此刻的源码过一遍复用判据（reuse_decision，
                                        覆盖记录、快照与源码副本取别处那一份）：抓到的要判成复用、基线 ok 的要快照与本机此刻相同；符的连同它引的
                                        覆盖记录、快照与源码副本写进本机那一格记录目录（同目录临时文件写完改名），不符的拒、逐条列出栏与细节。
                                        跨机不比主机名，记录里也不写。退 0 全收；退 1 有拒的；退 2 用法错或算不出判法版本
  --list-items <根>                     逐行打可点名的变异：「<变异名>\\t第 <行号> 行，改 <文件>」，不编、不跑（与门禁 --list-items 同一份）
  --item <行号或变异名> [--item …] <根>  只判点名的几行：交给门禁 checker-tier-crates-mutation-replay.sh --item（同一份选行模式，点名单跑与整张跑时那一行判定相同，
                                        判定库记录照写、照复用）；点名的不在表里退 2。是重型（.claude/hooks/lib_heavy_tests.py 按名字认）
  llvm-tools <根>                       打找到的 llvm-profdata、llvm-cov 与它们的 LLVM 版本；找不到或与 rustc 的 LLVM 版本对不上退 1、给出路
  --selftest                            自证（假 cargo、假 llvm 工具、假内存包装，在临时仓里跑 run_stage；格数由成功行现算）

拷贝范围 = .claude/gate.d/stage-inputs.tsv 里 checker-tier-crates-mutation-replay 那一行登记的路径（读本模块所在那一份仓的登记表：判别力样本的目录里没有登记表），
  底下每一段都不叫 target（各 crate 自己的编译目录）。checker-tier-crates-mutation-replay 照它拷每片的源码副本，code-source-discipline 照它静态判；判法只有 is_copied_into_each_shard 一处。
  ② 管不到的：路径拼在变量里、环境变量给的、format! 拼出来的、不以 "../ 起头而运行期再 join("..") 的；#[path = …] 不算（编译期带进来的源文件，
  副本里编得过就在）；指到仓根本身（"../../" 这一类）按不在范围里算（之后拼上的是哪一份判不出）。

判定库（checker-tier-crates-mutation-replay 每条判完立刻写，下一趟按「复用判据」逐行判复用还是现跑）：
  Python 一侧判定库的共用部分在 scripts/crash/lib_verdict_library.py（原子写 write_atomically、按键读 read_text、现判与复用分开报 Tally），这里调它；
    这一份每条另挂覆盖记录、快照与源码副本，存放布局与复用判据在这里，不在共用部分。
  键分两列（判定库文件头「键分两列」）：判法版本定这一格目录，行键（这一行的输入指纹里不随源码变的那一段）定一条记录；
    这一行改坏的源码与它执行过的函数不进行键，由快照与覆盖记录按「复用判据」比。
  放在被判那棵树的 git common-dir 下 singlefs-mutation-rows/<判法版本>/：一条一个文件 mutation.<行键>（变异的判定）、baseline.<行键>（基线），
    子目录 coverage/<覆盖号>.json.gz（覆盖记录）、snapshots/<快照号>.json.gz（快照）、blobs/<sha256>（源码副本，zlib 压过）。
  判法版本 = 登记行 + 阶段脚本 + 本模块 + scripts/mutation/mutation-prune.py（切词与切 fn 项）+ scripts/gate/lib_libtest_result_lines.py（libtest 结果行）
    + scripts/crash/lib_verdict_library.py（判定库共用部分）+ 这一趟用的内存包装，连同构建环境（admission.py 的 build_environment_lines：照理是输入，
    对每一行都一样，变了整张重判，所以放进这一列），不含被测源码；阶段脚本、本模块、mutation-prune.py、lib_libtest_result_lines.py、lib_verdict_library.py 与内存包装按内容进（名字里不带绝对路径，主工作区与临时 worktree、两台机器上算出同一个数）。这几样变了换一格目录，旧记录一条都不用。
  行键 = sha256（这一行六段原样用制表符接起来 + 这一趟的每条内存上限 + 每条限时 + 工具链：cargo -V 与 rustc -V 原样，toolchain_text）。
  快照 = 拷贝范围里 scripts/ 之外每个文件（跳过 target 与 crates/mutations.tsv，照 prepare_shard 拷的那样走）的「路径 → sha256」；快照号是它的 sha256。
    每个工作进程按自己那份源码副本算一份，记录里写的是那一份（开跑之后主工作区又改了，记录照实写跑的是哪一份）。
  覆盖记录（cargo 参数 + 点名用例一份，被同一组合的各行共用；覆盖号 = sha256（组合键 + 快照号），组合键 = sha256（`--` 之前的参数、`--` 之后的开关、
    点名用例的全名、工具链））：有一行判成抓到、这一组合在那份快照上还没有覆盖记录时，在没改坏的副本上记一份（judge_coverage）：
    ① cargo test --no-run --message-format=json-render-diagnostics 带 -C instrument-coverage（环境里有 RUSTFLAGS / CARGO_ENCODED_RUSTFLAGS 就接在后面，
       没有就经 --config build.rustflags，与配置文件里的合并），编译目录是这个工作进程的编译目录加 -coverage；
       LLVM_PROFILE_FILE=<这一格的 TMPDIR>/build-profiles/build-%p-%m.profraw：cargo 没带 --target 时 RUSTFLAGS 也罩构建脚本与过程宏，它们在各自依赖包的源码目录
       （~/.cargo/registry/src/ 下）里起，不设这个变量就在那里写 default_*.profraw；所以走「设变量」而不是「插桩只罩工作区成员」（后者要 --target <宿主>，
       换了产物目录布局，dep-info 与可执行文件路径都跟着变）；
    ② cargo test -- <开关> --exact <用例全名>，LLVM_PROFILE_FILE=<这一格的 TMPDIR>/cov-%p-%m.profraw；点名用例要恰好一行 ok；
    ③ llvm-profdata merge -sparse，llvm-cov export -format=text -skip-expansions，对象是 ① 编出来的本仓每个可执行文件（测试与 bin）；
    ④ 记：编进去的文件（① 里本仓每个编译单元的 cargo dep-info，proc-macro 与构建脚本的单元另记成编译期）、涉及的包目录、
       执行过的函数（计数 > 0 的函数按它第一段代码区域落进哪个最外层 fn 项，照 scripts/mutation/mutation-prune.py 的 covered_containers：
       「文件 + 限定名」，落在 fn 项之外记 <函数外>、文件切不开记 <整个文件>），编进去的每个 .rs 存一份源码副本。
    ①–③ 经内存包装、上限与限时同变异；有一步不成这一组合不记（下一趟照「没有覆盖记录」现跑），成功句另报「覆盖没记成 N 组」，不判红。
    记成之后把这一趟判成抓到、所判快照与它相同的同组各行改指到它（复用来的保留原来的 finished_utc）。
  llvm 工具：GATE_MUTATION_LLVM_TOOLS=<目录> 设了用它（双机分片的驱动设了就带到第二台），不设取 rustc --print sysroot 下 lib/rustlib/<host>/bin
    （rustup 组件 llvm-tools）；两样都要在、--version 的 LLVM 版本要以 rustc -vV 的「LLVM version」起头，不然这一道判红、一条都不跑，不退回不记覆盖。
  只有「抓到」与基线 ok 的记录复用，别的档每趟重跑；记录不按时限过期（SINGLEFS_REUSE_HOURS 这一道不认）；
  SINGLEFS_GATE_FULL=1 或 GATE_MUTATION_START_OVER=1 一律不复用（照旧写新记录、记覆盖）。
  基线 ok 的记录只在它的快照与此刻的快照相同时复用。
  每趟收尾删掉没有行引的覆盖记录、没有记录引的快照、没有覆盖记录引的源码副本（collect_garbage）。
  singlefs-mutation-rows/ 底下只留最新的 3 个判法版本目录（按修改时刻，这一趟的那个算最新），更早的开跑时删；别处一概不删。记录里不写主机名。

复用判据（reuse_decision，纯函数；按次序判，头一条不成立的就是这一行现跑的栏）：
  一律现跑   SINGLEFS_GATE_FULL=1 或 GATE_MUTATION_START_OVER=1
  行新加或改了   这一格目录里没有这个行键的 mutation 记录（六段、上限、限时、工具链任一样变了行键就变）；目录里一条新格式记录都没有时记「没有覆盖记录」
  没有覆盖记录   上次不是抓到（只有抓到记覆盖），或抓到而没有覆盖记录、覆盖记录或它的快照读不到
  （上次判抓到、所判快照与此刻相同：一字未动，复用，不看覆盖）
  被变异的函数变了   被变异那份文件里原文不是恰好一处；或那份文件是编进测试二进制的 .rs，包住原文的最外层 fn 项（mutation-prune.py 的 FileIndex 切的，
                    含属性、文档注释、签名与函数体，比词法单元序列，注释与空白不算）此刻没了、变了，或原文此刻不落在它里面；原文不在任何 fn 项里的，
                    那份文件函数外的部分要不变；不是编进去的 .rs（litmus/ 这类）按整份 sha256 比
  执行过的函数变了   那条用例执行过的每个「文件 + 限定名」，此刻的词法单元序列要与覆盖记录那份快照里的相同（<函数外> 比函数外部分，<整个文件> 比整份）
  函数外有改动   覆盖记录那份快照与此刻之间每个变了的文件（新加、删掉、改了）按 path_effect 分：
                    编进去的 .rs、两边都在：函数外的词法单元要不变、切得开，不许新加函数（#[test] 函数与 fn 里嵌套的 fn 除外），
                      变了的函数不许改属性、不许是 const fn、函数体里不许有 impl 块（outside_problem）；
                    编进去的其余文件（include_str! 读的、proc-macro 与构建脚本单元的、删掉了的）一律算函数外；
                    涉及的包目录里没编进去的：.rs 只有落在 cargo 自动发现的目标位置（build.rs、src/main.rs、src/lib.rs、src/bin/、tests/、examples/、benches/）
                      而新加或删掉的算，别的 .rs 不算；非 .rs（Cargo.toml、测试读的数据文件）一律算；
                    没涉及的包目录里的不算；不在任何包目录里的（根 Cargo.toml、Cargo.lock、.cargo/、litmus/）一律算
  都成立就「按覆盖复用」。前提：用例对同一份源码执行的函数集合是确定的（随机源、线程、时序只改函数里走哪条分支）；不守这条的用例照常复用：
    判定本来就是单跑一趟的结论，同组有一行现跑时记的新覆盖顶掉旧的，SINGLEFS_GATE_FULL=1 整张重判。
  GATE_MUTATION_RECORDS_HOME=<目录> 设了，判定库记录与用时表放在它底下、不放 git common-dir（双机分片的驱动给第二台设：那棵树每趟新拷、跑完删，
    记录与用时表要跨趟留着）；run_stage、records-directory、import、row-plan、plan 都认它。
  用时表：同一个 git common-dir 下 singlefs-mutation-row-timings/<这一行六段原样的 sha256>，跨判法版本沿用，记上一次现跑的用时与超没超时；
    import 不导用时（另一台的快慢不代表本机）。
  用时从包装起跑到返回分两段记：排队（调包装到命令在 scope 里起来）与跑（起来到返回）。命令起来的那一刻由包在命令外面的一层 bash 记进
    这一趟源码副本总目录底下的一个文件（不在那一条的 TMPDIR 里，不进「删前留有几个文件」）；包装的峰值表仍按原命令记键（RUN_WITH_MEMORY_CAP_KEY）。
  包装排队时每 60 秒往 stderr 报一行（scripts/hosts/run-with-memory-cap.sh 的 WAITING_REPORT_EVERY_SECONDS）；这里收包装的 stderr 判结局，
    那几行（WRAPPER_QUEUE_LINE）边跑边另转打到本进程的 stderr，行首带是哪一行变异或哪一批基线（run_forwarding_queue_lines）；收进来判结局的输出与 stdout 都不变。

并行理由（包装文件头「日常一条」：包装 2026-10-06 撤了「同一个会话同时只许一条」，并行理由它已经不看；下面照旧设着，无害）：
  这里起的每次包装（基线的批与每条变异）都带 RUN_WITH_MEMORY_CAP_PARALLEL_REASON=「同一次 checker-tier-crates-mutation-replay（本模块进程 <pid>）的第 <i> 个工作进程，
  各用各的源码副本与编译目录（这一轮共 <n> 个）」，<i> 是那个工作进程的片号（编译目录 -w<i> 那个号）；不设的话会话里平常跑时第 2 个工作进程起就排不上、记成「排不上没跑」。
  调用方已经设了（非空）的照原样传、不覆盖。只设在那一次包装的环境里（run_under_wrapper），不进本进程与 --check 那一次的环境；包装把它清掉之后才起 cargo。
  做法照 scripts/mutation/mutate.sh 文件头「并行」那一段。提交时的门禁带 SINGLEFS_HEAVY_TESTS=commit，不算日常，不受这条影响。

基线（派变异之前，按批在没改坏的副本上跑一遍点名的测试）：
  批的键 = 这一行 cargo 参数里 `--` 之前那一段 + `--` 之后以 `--` 起头的开关；过滤词取这一批要跑基线的各行 `--` 之后非开关词的并集，
  有一行没有过滤词就整批不带过滤词。要单独带值的开关（`--skip 名字`、`--test-threads 4`）写成 `--test-threads=4`：分开写时那个值会被当成过滤词。
  同样经内存包装、同样的上限与限时。每一行的点名测试在基线输出里要恰好对上一个用例、结果都是 ok，否则这一行记「基线不绿」：
  不算抓到、不算没红、它的变异不跑，整道判红。已有作数的抓到记录或基线 ok 记录的行不再跑基线。

本次未跑（2026-10-01 加）：点名的测试在本机没跑到被改坏的那一处时，打一行 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>` 后返回成功（GPU 对拍用例在没卡的机器上）。
  变异那一趟的 libtest 参数里补 --show-output（with_shown_output，插在第一个 `--` 之后）：成功用例的输出 libtest 默认不交出来；
  不用 --nocapture：单线程跑时它把用例的输出插进「test <名字> ... 」与结果之间，结果行就认不出。基线那一批不补。
  点名的测试结果是 ok、输出里有一行 test= 对上点名的测试（只比 :: 分开的末一段）的，这一条记「本次未跑」（not_run_reason）：不算抓到、不算没红，逐条打出来；
  整道不因它判红，成功句写明抓到几条、本次未跑几条，全绿标记不写；一条抓到的都没有、只有本次未跑的退 77（选行模式照旧退 3 / 1）。
  GATE_MUTATION_NOT_RUN_RECORD=<文件> 设了，每条本次未跑往里追加一行（阶段把它转给 gate.sh 的 GATE_NOT_RUN_FILE）；写不进去整道判红。
  只有「抓到」复用，本次未跑的下一趟照跑。

弄坏开关（GATE_MUTATION_BREAK，只给证红用，逗号分隔可给几个；CRATES_MUTATION_ROWS_BREAK 是同义名，直接起本模块自证时用，两个变量都认）：
  nobaseline           跳过基线：点名的测试在没改坏的副本上本来就红的行照样记抓到（checker-tier-crates-mutation-replay 红样本、本模块自证都必须判错）
  timingsignored       派活不看用时表（自证「上一次跑得久的先派」那一格必须判错）
  importunchecked      import 不核文件名、判法版本、行键与复用判据，照单全收（自证「导入一条行键对不上的记录：拒」「导入改了执行过的函数的记录：拒」两格必须判错）
  rowkeywithouttoolchain  行键不带工具链（复用判据「行新加或改了」那一半：工具链换了照样复用）
                       （自证「工具链变了：每一行都按行新加或改了现跑」那一格必须判错）
  mutatedfunctionignored  复用判据不看被变异的函数（自证「改了被变异、没被执行的函数：只现跑那一行」那一格必须判错；
                       「改了被执行、没被变异的函数」「导入改了执行过的函数的记录」两格同样判错：那里两行 adds 也只靠这一条现跑）
  executedfunctionsignored  复用判据不看执行过的函数（自证「改了被执行、没被变异的函数：只现跑执行过它的那几行」「导入改了执行过的函数的记录」两格必须判错）
  outsidechangesignored  复用判据不看函数外的改动（自证「改了函数外的类型定义：编进它的用例点名的行全部现跑」「新加一个用到的源文件：全部现跑」两格必须判错）
  coveragenotrecorded  抓到之后不记覆盖（自证「头一趟每组记一份覆盖、同组的行共用一份」那一格必须判错；靠覆盖复用的格跟着判错，共 13 格）
  llvmversionunchecked 不核 llvm 工具与 rustc 的 LLVM 版本（自证「llvm 工具的 LLVM 版本与 rustc 对不上：判红、一条都不跑」那一格必须判错）
  buildprofilesunset   插桩编译那一步不设 LLVM_PROFILE_FILE（改前的样子：构建脚本与过程宏的剖析文件落进它们起的目录）
                       （本模块自证「插桩编译那一步写出的剖析文件只落在这一格的临时目录」那一格必须判错）
  keepmutationtmp      每条变异的 TMPDIR 不删（checker-tier-crates-mutation-replay 两份样本都必须判错）
  notrunasmissed       点名的测试打了 NOT_RUN_NO_GPU_ADAPTER 也当没红记（checker-tier-crates-mutation-replay 的 not-run 样本、本模块自证「点名的测试打了 NOT_RUN」那一格都必须判错）
  emptyreplacementrefused  替换文空着也判不成形（checker-tier-crates-mutation-replay 绿样本里的删行变异、code-source-discipline 绿样本里的删行变异、本模块自证「删行变异照常判」那一格都必须判错）
  queuelineswallowed   内存包装排队报的那几行不转打到本进程的 stderr、只收进变量（本模块自证「包装排队报的那一行边跑边转打」那一格必须判错）
  gpumemoryasoutput    内存包装退 245（显存不够）不单列、照输出判（本模块自证「包装退 245（显存不够）记显存不够、不算抓到」那一格必须判错）
  cargoconfignotcopied 拷每片源码副本时跳过拷贝范围里的 .cargo/（登记漏了它时的样子）：副本里的 cargo 读不到 .cargo/config.toml 的 [env]
                       （本模块自证「拷贝范围含 .cargo/：点名的测试要编 gpu 特性的那一行基线绿、变异抓到」那一格必须判错）
  staticusagebare      static-check 参数不对时只打改前那一行用法，不写缺了什么、不给「→ 怎么办」（本模块自证「static-check 缺 <根>」那一格必须判错）
  tablemissingok       static-check --table 指的文件不在时照旧当空表判、报「CHECKED 0 …」退 0（本模块自证「static-check --table 指的文件不在」那一格必须判错）
  shardrootkeptonsignal  run_pool 不接 TERM / HUP（改前的样子：收到就停，$TMPDIR 下的 singlefs-mutation-replay-* 留着）
                       （本模块自证「被 TERM 停：$TMPDIR 下的 singlefs-mutation-replay-* … 停完删掉，退 143」那一格必须判错）
  listitemspipeunhandled  --list-items 不照 SIGPIPE 惯例退（接 | head 时打 BrokenPipeError 的回溯、退 1）
                       （本模块自证「--list-items 接 | head」那一格必须判错）
  extratargetskept     开跑时不回收片号大于这一趟工作进程数的 -w<片号> 编译目录（改前的样子：分片各一份、只增不减）
                       （本模块自证「片号大于这一趟工作进程数的 -w<片号> 连同 -coverage 开跑时回收、不大于的留着、打一行删了几份多大」那一格必须判错）
  inusetargetremoved   回收之前不核有没有进程在用（工作目录、命令行、CARGO_TARGET_DIR、在用标记），多出来的照删
                       （本模块自证「多出来的 -w<片号> 有进程在用的不删」那一格必须判错）
  targetdefaulttmp     GATE_MUTATION_TARGET_DIR 与 GATE_CROSS_RUN_TMPDIR 都没设时编译目录照改前退回 ${TMPDIR:-/tmp}
                       （本模块自证「两个变量都没设时编译目录不落 /tmp、落在跨轮缓存家目录下」那一格必须判错）
  local-shard-numbering  工作进程的片号不领锁、照改前从本调用的共享计数器现领本地序号（改前的样子：两次并发调用的第 2 片都指向同一个 -w2）
                       （本模块自证「同一个共用编译目录上并发的两次调用各起 2 个工作进程：领到的片号集合不相交、四把片号锁都被占」那一格必须判错）
  sharedcrashlibrary   起 cargo 时不换崩溃判定库的家目录、不清调用方点名的单份库（改前的样子：变异版的结论写进真跑的长期库）
                       （本模块自证「变异复跑起的 cargo 写自己的崩溃判定库」「设了 GATE_MUTATION_CRASH_LIBRARY_HOME」两格必须判错）
编译目录（default_shared_target_directory，阶段脚本不另算）：GATE_MUTATION_TARGET_DIR 设了用它；不然 GATE_CROSS_RUN_TMPDIR 设了（gate.sh 设成它开跑前的 TMPDIR）
  放它底下的 singlefs-crates-mutation-target；都没设放 ${SINGLEFS_CROSS_RUN_CACHE_HOME:-~/.local/share/singlefs/cross-run-cache}/singlefs-crates-mutation-target，
  不退回 $TMPDIR 或 /tmp（command-safety.md「有意跨轮复用的缓存」：放 GATE_CROSS_RUN_TMPDIR 下或项目定的位置）。第 1 片用它，其余片加 -w<片号>，记覆盖的再加 -coverage。
  回收（reclaim_extra_target_directories）：派活之前，同一层里片号大于这一趟工作进程数的 -w<片号> 与 -w<片号>-coverage 删掉，stderr 打一行删了几份、共多大；
  删之前核没人在用——有进程的工作目录在它底下、命令行里有它底下的路径、环境里 CARGO_TARGET_DIR 指到它底下，或那一片的在用标记
  （工作进程活着时在自己的编译目录里拿着 .singlefs-mutation-worker.lock 的共享锁，回收一侧试拿排他锁拿不到）被别的进程拿着，就留着、逐份写明谁在用。
  代价：这一趟工作进程少（--item 只点一两行）之后，下一趟整张跑时多出来的那几片要从头冷编一次。
  片号（claim_worker_slot）跨调用经锁领：共用编译目录那一层里每个片号一把锁文件（第 1 片 <共用目录>.slot，其余片 <共用目录>-w<片号>.slot），
  工作进程启动时从 1 往上试拿排他 flock（不阻塞），拿到第一个空闲的就是它的片号，锁拿到进程退出（stderr 每个工作进程报一行「领到片号 <n>」）。
  同一台机器上并发的两次调用因此拿到不相交的片号集合，不再两边的第 2 片都指向同一个 -w2 而被 cargo 的「Blocking waiting for file lock on build directory」互相排队；
  编译目录仍按片号跨轮复用（不按仓路径或会话号分目录：--staged 的临时 worktree 每趟都会冷编）。片号锁文件建不了、或前 4096 片都被占：报一行，退回本调用内的本地序号。
  回收（reclaim_extra_target_directories）同样核片号锁：别的调用的工作进程刚领到的那一片，被占着的不删。
工作进程数（worker_count_for）：三项取最小、至少 1：核数的一半（至多 16，GATE_MUTATION_WORKERS 顶掉它）、这一趟要现跑的条数、外层内存上限 ÷ 每条变异的上限（向下取整）。
  外层内存上限读 RUN_WITH_MEMORY_CAP_OUTER_CAP（scripts/hosts/run-with-memory-cap.sh 套在外层时给命令设的，写法 正整数加 K / M / G / T，1024 进制）：
  套在外层包装里的里层包装不排队（那份文件头「套在包装里」），几条里层同时跑、合起来超过外层上限时只有 slice 的总上限兜着，所以按外层收；
  没设或写法不认就是没套在外层里：没有这一项（每条自己经包装排队、放不下的在它那里等），stderr 头一行写明「外层内存上限没读到」及原因，不悄悄少一项。
  外层上限不足一条的上限时按 1 个开（头一行写「那一项不足 1」）：里层的上限比外层大，包装会按用法错退 251，那是调用方该改上限的事，不在这里改每条的上限。
  每条变异的上限本身不动（量到峰值之后再定；它进行键，一变全表判定记录作废）。
测试线程数：不压。起 cargo test 时 RUST_TEST_THREADS 与仓里各测试读的线程数变量（SINGLEFS_THREAD_CAP 等）调用方设了什么就原样透传；只有 CARGO_BUILD_JOBS（编译并行度）按核数 ÷ 工作进程数收。
变异复跑的判定库（mutation_crash_library_environment）：走判定库的 checker 档用例在变异版上判出的结论不写进真跑的长期库。起 cargo 的环境里
  SINGLEFS_CRASH_AMPLIFICATION_HOME 换成 GATE_MUTATION_CRASH_LIBRARY_HOME（设了、非空）；没设取真跑的家目录（调用方的 SINGLEFS_CRASH_AMPLIFICATION_HOME，
  没设是 ~/.local/share/singlefs/crash-amplification）底下的 mutation-replay/，跨轮留着、照用例自己的键复用；调用方点名的单份库
  SINGLEFS_CRASH_AMPLIFICATION_LIBRARY 清掉（它压过家目录）。stderr 报一行这一趟用的家目录。双机分片的驱动给第二台设成那棵树的跨趟目录底下一份。
libtest 的结果行（`test <全名>[ - should panic] ... <结局>`）与汇总行只在 scripts/gate/lib_libtest_result_lines.py 解析一份（results_by_test 调它），
  它的弄坏开关 LIBTEST_RESULT_LINES_BREAK 写在它文件头；本模块自证里认得它的那一格：
  LIBTEST_RESULT_LINES_BREAK=should-panic-in-name  本模块自证「should panic 的用例红了判抓到、点名省掉模块路径只对上那一个」那一格必须判错
下面三个开关一行一个，下一行起写打开后必须判错的自证项：
  outermemoryignored   工作进程数不看外层内存上限（当 RUN_WITH_MEMORY_CAP_OUTER_CAP 读不到，改前的样子）
    本模块自证「工作进程数不超过外层内存上限 ÷ 每条变异的上限」「外层内存上限宽裕时仍卡在别的项」「外层上限不够一条的上限」「外层内存上限交给 run_stage：开的工作进程数跟着收」四格必须判错
  noparallelreason     自己起的包装不设并行理由（改前的跑法）
    本模块自证「会话里平常跑（假包装照日常一条演）、4 个工作进程：各条照样判对，每次包装带自己那个工作进程的并行理由」
  overridecallerreason 调用方设了的并行理由也换成自己的
    本模块自证「调用方设了的并行理由原样传给每次包装、不覆盖」
只供测试的开关：GATE_MUTATION_STOP_AFTER_ROWS=<k> 现跑的变异判完 k 条就不再派活、等在跑的收完，整道判红退出（造「跑到一半被杀」）。
"""
import calendar
import collections
import concurrent.futures
import contextlib
import fnmatch
import gzip
import hashlib
import heapq
import importlib.util
import io
import json
import multiprocessing
import os
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time
import types
import zlib

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
STAGE_REPOSITORY = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(STAGE_REPOSITORY, ".claude", "scripts"))
from project_preflight import preflight  # noqa: E402

LIBTEST_RESULT_LINES_PATH = os.path.join(SCRIPT_DIRECTORY, "..", "gate", "lib_libtest_result_lines.py")   # libtest 结果行的解析（全仓一份），进判法版本
_LIBTEST_RESULT_LINES_SPECIFICATION = importlib.util.spec_from_file_location("lib_libtest_result_lines_for_crates_mutation_rows", LIBTEST_RESULT_LINES_PATH)
libtest_result_lines = importlib.util.module_from_spec(_LIBTEST_RESULT_LINES_SPECIFICATION)
_LIBTEST_RESULT_LINES_SPECIFICATION.loader.exec_module(libtest_result_lines)
VERDICT_LIBRARY_PATH = os.path.join(SCRIPT_DIRECTORY, "..", "crash", "lib_verdict_library.py")   # 判定库共用部分（原子写、按键读、现判与复用分开报），进判法版本
_VERDICT_LIBRARY_SPECIFICATION = importlib.util.spec_from_file_location("lib_verdict_library_for_crates_mutation_rows", VERDICT_LIBRARY_PATH)
verdict_library = importlib.util.module_from_spec(_VERDICT_LIBRARY_SPECIFICATION)
_VERDICT_LIBRARY_SPECIFICATION.loader.exec_module(verdict_library)
write_atomically = verdict_library.write_atomically   # 同目录排他建临时文件、写完 fsync、改名换上；data 是 str 或 bytes

STAGE = "checker-tier-crates-mutation-replay.sh"   # 登记表的键、阶段脚本进判法版本，都按这个文件名
STAGE_SCRIPT = os.path.join(STAGE_REPOSITORY, ".claude", "gate.d", STAGE)
MODULE_PATH = os.path.realpath(__file__)
DEFAULT_MEMORY_CAP_RUNNER = os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "run-with-memory-cap.sh")
TABLE = "crates/mutations.tsv"
RECORDS_DIRECTORY_NAME = "singlefs-mutation-rows"
TIMINGS_DIRECTORY_NAME = "singlefs-mutation-row-timings"
COVERAGE_DIRECTORY_NAME = "coverage"
SNAPSHOT_DIRECTORY_NAME = "snapshots"
BLOB_DIRECTORY_NAME = "blobs"
SNAPSHOT_LEFT_OUT_PREFIX = "scripts/"   # 拷贝范围里 scripts/ 下的（内存包装、本模块与它读的几份库）按内容进判法版本，不进快照
LLVM_TOOLS = ("llvm-profdata", "llvm-cov")
LLVM_TOOLS_VARIABLE = "GATE_MUTATION_LLVM_TOOLS"
COVERAGE_TARGET_SUFFIX = "-coverage"
# 编译目录（文件头「编译目录」）：GATE_MUTATION_TARGET_DIR 设了用它；不然 GATE_CROSS_RUN_TMPDIR 设了放它底下；都没设放跨轮缓存家目录底下（不落 /tmp）
TARGET_DIRECTORY_NAME = "singlefs-crates-mutation-target"
CROSS_RUN_CACHE_HOME_VARIABLE = "SINGLEFS_CROSS_RUN_CACHE_HOME"
DEFAULT_CROSS_RUN_CACHE_HOME = "~/.local/share/singlefs/cross-run-cache"
# 变异复跑的崩溃判定库（文件头「变异复跑的判定库」）：起 cargo 时家目录换成它，不写进真跑的长期库
MUTATION_CRASH_LIBRARY_HOME_VARIABLE = "GATE_MUTATION_CRASH_LIBRARY_HOME"
CRASH_LIBRARY_HOME_VARIABLE = "SINGLEFS_CRASH_AMPLIFICATION_HOME"
CRASH_LIBRARY_DIRECTORY_VARIABLE = "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY"
DEFAULT_CRASH_LIBRARY_HOME = "~/.local/share/singlefs/crash-amplification"
MUTATION_CRASH_LIBRARY_SUBDIRECTORY = "mutation-replay"
# 工作进程活着的时候在自己的编译目录里拿着这把共享锁（flock）：别的一趟回收多出来的 -w<片号> 之前试拿排他锁，拿不到就是有工作进程在用
WORKER_TARGET_LOCK_NAME = ".singlefs-mutation-worker.lock"
# 片号跨调用经锁领：共用编译目录那一层里每个片号一把锁文件（第 1 片 <共用目录>.slot，其余片 <共用目录>-w<片号>.slot）；工作进程启动时从 1 往上试拿排他锁（不阻塞），
# 拿到第一个空闲的就是它的片号，锁拿到进程退出。两次并发调用因此拿到不相交的片号集合，不会两边的第 2 片都指向同一个 -w2 而被 cargo 的目录锁互相排队
WORKER_SLOT_LOCK_SUFFIX = ".slot"
WORKER_SLOT_SEARCH_LIMIT = 4096
COVERAGE_FLAG = "-Cinstrument-coverage"
BUILD_PROFILES_DIRECTORY_NAME = "build-profiles"   # 插桩编译那一步（构建脚本、过程宏）写出的剖析文件的去处，在那一格的 TMPDIR 下，随 TMPDIR 一起删
# cargo 自动发现的目标位置（相对包目录）：这里新加或删掉一个 .rs 就可能多编、少编一个目标
AUTO_TARGET_FORMS = [re.compile(form) for form in (r"build\.rs", r"src/(main|lib)\.rs", r"src/bin/[^/]+\.rs", r"src/bin/[^/]+/main\.rs",
                                                    r"tests/[^/]+\.rs", r"tests/[^/]+/main\.rs", r"examples/.*", r"benches/.*")]
# 复用判据的栏（reuse_decision 交的栏名，次序就是报数的次序）
REUSE_COLUMNS = (("start-over", "一律现跑"), ("row-changed", "行新加或改了"), ("no-coverage", "没有覆盖记录"),
                 ("mutated-function", "被变异的函数变了"), ("executed-function", "执行过的函数变了"), ("outside", "函数外有改动"))
COLUMN_TITLES = dict(REUSE_COLUMNS)
BASE_DIRECTORIES_KEPT = 3
FINGERPRINT_FORM = re.compile(r"[0-9a-f]{64}")
RECORD_KINDS = ("mutation", "baseline")
REUSABLE_VERDICT = {"mutation": "caught", "baseline": "ok"}
RECORD_HEADER = ("# 门禁 checker-tier-crates-mutation-replay 的判定库记录（scripts/mutation/crates-mutation-rows.py 写，一条一个文件）：只有「抓到」与基线 ok 的、"
                 "照复用判据判过的复用（scripts/mutation/crates-mutation-rows.py 文件头「复用判据」）。不进工作树，别手改。\n")
# 拷目录时跳过的名字（各 crate 自己的编译目录）。拷贝范围由它与登记的路径一起定：prepare_shard 照它们拷，is_copied_into_each_shard 照它们判。
IGNORED_WHEN_COPYING_INTO_EACH_SHARD = ["target"]
# 拷贝范围里的 cargo 配置目录：.cargo/config.toml 的 [env] BINDGEN_EXTRA_CLANG_ARGS 给 librocksdb-sys 的 bindgen 指头文件；副本在 /tmp 下、往上找不到仓里那一份，
# 点名的测试要编 gpu 或 verdict-store 特性的行基线必然不绿（弄坏开关 cargoconfignotcopied 拷的时候跳过它）
CARGO_CONFIGURATION_DIRECTORY = ".cargo"
MEMORY_CAP_HIT_EXIT = 250            # run-with-memory-cap.sh：撞了这一条自己的上限（scope 的 Result 是 oom-kill、自己的 oom 计数不为 0）
MEMORY_CAP_UNAVAILABLE_EXIT = 251    # run-with-memory-cap.sh：带上限的 scope 或 slice 的总上限起不来
MEMORY_ADMISSION_REFUSED_EXIT = 252  # run-with-memory-cap.sh：排队等满还放不下，命令没跑
TIME_LIMIT_HIT_EXIT = 253            # run-with-memory-cap.sh：超过 RUN_WITH_MEMORY_CAP_TIME_LIMIT（scope 的 Result 是 timeout）
SLICE_TOTAL_HIT_EXIT = 254           # run-with-memory-cap.sh：被 slice 的总上限挤掉（整个 slice 满了，内核挑了这一条杀）
GPU_MEMORY_HIT_EXIT = 245            # run-with-memory-cap.sh：显存不够（命令退了非 0、起止之间内核报了 NV_ERR_NO_MEMORY），结果不算数
DEFAULT_MEMORY_MAX_PER_MUTATION = "4G"         # 依据见 checker-tier-crates-mutation-replay 文件头
MEMORY_MAX_DIRECTIVE = re.compile(r"^#\s*每条变异的内存上限[：:]\s*(\S+)\s*$")
OUTER_MEMORY_CAP_VARIABLE = "RUN_WITH_MEMORY_CAP_OUTER_CAP"   # run-with-memory-cap.sh 套在外层时给命令设的外层上限（那份文件头「套在包装里」一段）
SIZE_TEXT = re.compile(r"^([1-9][0-9]*)([KMGT])$")             # systemd 的写法，1024 进制（与包装的 size_in_bytes 一致）
SIZE_UNIT_BYTES = {"K": 1024, "M": 1024 ** 2, "G": 1024 ** 3, "T": 1024 ** 4}
DEFAULT_TIMEOUT_SECONDS_PER_MUTATION = "1800"  # 依据见 checker-tier-crates-mutation-replay 文件头
TIMEOUT_DIRECTIVE = re.compile(r"^#\s*每条变异的超时秒数[：:]\s*(\S+)\s*$")
DEFAULT_LONG_ROW_SECONDS = "300"               # 上一次用时不短于它（或超时）的行先派：GATE_MUTATION_LONG_ROW_SECONDS
TIMEOUT_KILL_GRACE_SECONDS = 30                # 到点发 TERM 之后再等这么久，还不退就发 KILL（交给包装的 RUN_WITH_MEMORY_CAP_KILL_GRACE）
PER_MUTATION_TEMPORARY_DIRECTORY_NAME = "per-mutation-tmp"
START_STAMP_DIRECTORY_NAME = "started-at"
# 包在命令外面的那一层：命令在 scope 里起来的那一刻写进 $1，再 exec 原命令（用时分排队与跑两段，见文件头）
START_STAMP_SCRIPT = 'printf "%s" "$EPOCHREALTIME" > "$1"; shift; exec "$@"'
VERDICTS_KILLED_BY_WRAPPER = ("timeout", "memory", "squeezed")
VERDICTS_THAT_DID_NOT_RUN = ("refused", "unavailable")
TEST_NAME_FORM = re.compile(r"[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)*")
# 点名的测试在本机没跑到被改坏的那一处时打的一行（见文件头「本次未跑」）；变异那一趟补 --show-output，libtest 才把成功用例的这一行交出来
NOT_RUN_LINE = re.compile(r"^\s*NOT_RUN_NO_GPU_ADAPTER test=(\S+)(?: reason=(.*))?$", re.M)
LIBTEST_SHOW_OUTPUT = "--show-output"
BYTES_PER_STAT_BLOCK = 512   # os.stat 的 st_blocks 按 512 字节一块数（POSIX）
EXIT_SELECTION_ONLY = 3      # 只判了选中的行、都抓到：不是整道结论，门禁汇总记失败，不会被当成通过
# run_pool 里接住这两个信号：先停自己起的工作进程、删 shard_root，再退 128 + 信号号（双机分片的驱动停这一份、门禁被打断时都走这一条）
STOP_SIGNALS = (signal.SIGTERM, signal.SIGHUP)
# 源码里一处字符串字面量或注释的起头（找字面量时跳过注释用）
LITERAL_OR_COMMENT_START = re.compile(r'//|/\*|b?r#*"|b?"|\'')
PATH_ATTRIBUTE_BEFORE_LITERAL = re.compile(r"#\s*\[\s*path\s*=\s*$")
PATH_ATTRIBUTE_AFTER_LITERAL = re.compile(r"\s*\]")
# run_stage 读的环境变量（自证起每一格之前从调用方的环境里清掉，弄坏开关除外）
STAGE_VARIABLES = ("GATE_MUTATION_WORKERS", "GATE_MUTATION_CARGO_JOBS", "GATE_MUTATION_MEMORY_MAX", "GATE_MUTATION_TIMEOUT",
                   "GATE_MUTATION_ROW_SELECTION", "GATE_MUTATION_START_OVER", "GATE_MUTATION_LONG_ROW_SECONDS",
                   "GATE_MUTATION_STOP_AFTER_ROWS", "GATE_MUTATION_TARGET_DIR", "GATE_MUTATION_MEMORY_CAP_RUNNER",
                   "GATE_MUTATION_RECORDS_HOME", "GATE_MUTATION_NOT_RUN_RECORD", "GATE_MUTATION_LLVM_TOOLS", "GATE_MUTATION_CRASH_LIBRARY_HOME", "SINGLEFS_GATE_FULL",
                   "SINGLEFS_REUSE_HOURS", OUTER_MEMORY_CAP_VARIABLE)

Row = collections.namedtuple("Row", "line_number name path old new arguments expected fields")
FIELD_NAMES = ("变异名", "文件", "原文", "替换文", "cargo test 参数", "必须红的测试名")
REPLACEMENT_FIELD = FIELD_NAMES.index("替换文")   # 六段里只有它可以空：替换文空着就是把原文删掉
# 派活之前（checker-tier-crates-mutation-replay）与静态（code-source-discipline 经 static-check）判的那几样，打印与输出都照这个次序
ROW_PROBLEM_KINDS = ("MALFORMED", "ANCHOR", "NAME", "ROW", "STRAY")
_ADMISSION_MODULE = []


def admission_module():
    """scripts/gate/admission.py（判法版本里的构建环境与清单算法、git common-dir、登记表都用它的），第一次用到时才读进来。"""
    if not _ADMISSION_MODULE:
        specification = importlib.util.spec_from_file_location("admission_for_crates_mutation_rows", os.path.join(SCRIPT_DIRECTORY, "..", "gate", "admission.py"))
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _ADMISSION_MODULE.append(module)
    return _ADMISSION_MODULE[0]


# 弄坏开关从这两个变量里认（逗号分隔可给几个）：GATE_MUTATION_BREAK 是阶段脚本一路传下来的那个，CRATES_MUTATION_ROWS_BREAK 是直接起本模块自证时用的同义名
BREAK_VARIABLES = ("GATE_MUTATION_BREAK", "CRATES_MUTATION_ROWS_BREAK")


def break_is(environment, switch_name):
    return any(switch_name in environment.get(variable, "").split(",") for variable in BREAK_VARIABLES)


def say(text):
    print(text, flush=True)


def note(text):
    print(text, file=sys.stderr, flush=True)


def expanded_home_path(path, environment):
    """「~/」起头的按 environment 里的 HOME 展开（没有就按本进程的），再取绝对路径。"""
    if path == "~" or path.startswith("~/"):
        path = os.path.join(environment.get("HOME") or os.path.expanduser("~"), path[2:])
    return os.path.abspath(path)


def mutation_crash_library_environment(environment):
    """变异复跑起的 cargo 用的崩溃判定库家目录（文件头「变异复跑的判定库」）：返回 (家目录, 要清掉的变量)。
    GATE_MUTATION_CRASH_LIBRARY_HOME 设了（非空）用它；不然取真跑的家目录（SINGLEFS_CRASH_AMPLIFICATION_HOME，没设是 ~/.local/share/singlefs/crash-amplification）
    底下的 mutation-replay/。调用方点名的单份库 SINGLEFS_CRASH_AMPLIFICATION_LIBRARY 一律清掉：它压过家目录，留着就照写进那一份。
    弄坏开关 sharedcrashlibrary：返回 (None, ())，照改前不换、不清。"""
    if break_is(environment, "sharedcrashlibrary"):
        return None, ()
    configured = environment.get(MUTATION_CRASH_LIBRARY_HOME_VARIABLE, "").strip()
    if configured:
        return expanded_home_path(configured, environment), (CRASH_LIBRARY_DIRECTORY_VARIABLE,)
    real_home = environment.get(CRASH_LIBRARY_HOME_VARIABLE, "").strip() or DEFAULT_CRASH_LIBRARY_HOME
    return os.path.join(expanded_home_path(real_home, environment), MUTATION_CRASH_LIBRARY_SUBDIRECTORY), (CRASH_LIBRARY_DIRECTORY_VARIABLE,)


# ── 变异表 ────────────────────────────────────────────────────────────────────

def unescape(text):
    return text.replace("\\n", "\n")


def malformed_reason(fields, environment):
    """一行拆出来的几段不成形的原因，成形回 None。成形 = 六段；替换文可以空（删掉原文：改坏时原文换成空串），其余五段一段都不许空。
    弄坏开关 emptyreplacementrefused（只给证红用）让替换文空着也算不成形。"""
    if len(fields) != len(FIELD_NAMES):
        return f"不是六段（{len(fields)} 段）"
    empty = [FIELD_NAMES[index] for index, field in enumerate(fields)
             if field == "" and (index != REPLACEMENT_FIELD or break_is(environment, "emptyreplacementrefused"))]
    return "、".join(f"「{name}」" for name in empty) + "空着" if empty else None


def read_table(path, environment=None):
    """(成形的行, [(行号, 变异名, 为什么不成形)], 表头「每条变异的内存上限」, 表头「每条变异的超时秒数」)。成形照 malformed_reason：
    checker-tier-crates-mutation-replay、static-check（门禁 code-source-discipline）与 import 都照这一份读，没有第二份判法。environment 不给就读本进程的环境（只看弄坏开关）。"""
    environment = os.environ if environment is None else environment
    rows, malformed = [], []
    memory_max_from_table = timeout_from_table = None
    with open(path, encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, 1):
            line = line.rstrip("\n")
            directive = MEMORY_MAX_DIRECTIVE.match(line)
            if directive:
                memory_max_from_table = directive.group(1)
            directive = TIMEOUT_DIRECTIVE.match(line)
            if directive:
                timeout_from_table = directive.group(1)
            if not line or line.startswith("#"):
                continue
            fields = line.split("\t")
            reason = malformed_reason(fields, environment)
            if reason:
                malformed.append((line_number, fields[0] or "（变异名空着）", reason))
                continue
            rows.append(Row(line_number, fields[0], fields[1], unescape(fields[2]), unescape(fields[3]), fields[4], fields[5], tuple(fields)))
    return rows, malformed, memory_max_from_table, timeout_from_table


def limits_of(memory_max_from_table, timeout_from_table, environment):
    """(每条内存上限, 每条限时秒数, [(卡在哪, 下一步)])。三处取，先到先用：表头那一行、环境变量、默认。"""
    problems = []
    memory_max = memory_max_from_table or environment.get("GATE_MUTATION_MEMORY_MAX", "").strip() or DEFAULT_MEMORY_MAX_PER_MUTATION
    if not re.fullmatch(r"[1-9][0-9]*[KMGT]", memory_max):
        problems.append((f"每条变异的内存上限写成了「{memory_max}」，不是 systemd 的写法",
                         "写成正整数加 K / M / G / T，单位必写（不带单位的数 systemd 当字节数），例 4G、512M（表头那一行「# 每条变异的内存上限：…」或 GATE_MUTATION_MEMORY_MAX）。"))
    timeout_text = timeout_from_table or environment.get("GATE_MUTATION_TIMEOUT", "").strip() or DEFAULT_TIMEOUT_SECONDS_PER_MUTATION
    timeout_seconds = int(timeout_text) if re.fullmatch(r"[1-9][0-9]*", timeout_text) else None
    if timeout_seconds is None:
        problems.append((f"每条变异的限时写成了「{timeout_text}」，不是正整数秒",
                         "写成正整数秒，例 1800（表头那一行「# 每条变异的超时秒数：…」或 GATE_MUTATION_TIMEOUT）。"))
    return memory_max, timeout_seconds, problems


# ── 拷贝范围（checker-tier-crates-mutation-replay 拷进每片的、code-source-discipline 静态判的，同一份判法）──────────────────────

def copied_paths(stage_repository=STAGE_REPOSITORY):
    """登记表 checker-tier-crates-mutation-replay 那一行登记的路径（目录去掉末尾的斜杠）。登记表读不了、没有那一行抛 admission 的 RegistrationError。"""
    admission = admission_module()
    input_paths, _rows = admission.resolve_input_paths(admission.read_registration_rows(stage_repository), STAGE)
    return [input_path.strip().rstrip("/") for input_path in input_paths if input_path.strip()]


def is_copied_into_each_shard(path, copied):
    """一个仓根起的路径在不在每片的源码副本里：落在 copied 的某一项之内，而且那一项底下的每一段路径都不叫
    IGNORED_WHEN_COPYING_INTO_EACH_SHARD 里的名字。绝对路径、用 `..` 跳出去的、仓根本身都不在。"""
    normalized = os.path.normpath(path)
    if os.path.isabs(normalized):
        return False
    for item in copied:
        if normalized == item:
            return True
        if normalized.startswith(item + os.sep):
            below_item = normalized[len(item) + 1:].split(os.sep)
            return not any(fnmatch.fnmatch(part, pattern) for part in below_item for pattern in IGNORED_WHEN_COPYING_INTO_EACH_SHARD)
    return False


def rows_outside_copy(rows, copied):
    return [row for row in rows if not is_copied_into_each_shard(row.path, copied)]


def stray_escapes(segment):
    """一段里 \\n 以外的反斜杠转义：本模块与 mutate.sh 只把 \\n 还原成换行，别的反斜杠按字面写进源码，那条变异编不过、等于没跑（C427）。"""
    return [match.group(0) for match in re.finditer(r"\\(.)", segment) if match.group(1) != "n"]


def row_problems(root, rows, malformed, copied):
    """派活之前一个工作进程都不起就能判的那几样，checker-tier-crates-mutation-replay（prechecks）与门禁 code-source-discipline（static-check）都调这一份。回 {类别: [明细]}，类别照 ROW_PROBLEM_KINDS：
      MALFORMED (行号, 变异名, 为什么)   不成形（read_table 判的：六段，只有替换文可以空）
      ANCHOR    (行号, 变异名, 为什么)   原文在被判仓里「文件」那一段指的源码里不是恰好命中一次，或文件不在、读不了
      NAME      (行号, 变异名, 测试名)   必须红的测试名不是一个用例名（带了正则符号或空白）
      ROW       (行号, 变异名, 文件)     文件不在每片拷的范围里
      STRAY     (行号, 变异名, 段名, 转义) 原文或替换文里有 \\n 以外的反斜杠转义"""
    problems = {kind: [] for kind in ROW_PROBLEM_KINDS}
    problems["MALFORMED"] = list(malformed)
    for row in rows:
        try:
            with open(os.path.join(root, row.path), encoding="utf-8") as handle:
                count = handle.read().count(row.old)
        except FileNotFoundError:
            problems["ANCHOR"].append((row.line_number, row.name, f"文件 {row.path} 不存在"))
        except (OSError, UnicodeDecodeError) as error:
            problems["ANCHOR"].append((row.line_number, row.name, f"文件 {row.path} 读不了（{type(error).__name__}）"))
        else:
            if count != 1:
                problems["ANCHOR"].append((row.line_number, row.name, f"原文在 {row.path} 里命中 {count} 次，要恰好 1 次"))
        if not TEST_NAME_FORM.fullmatch(row.expected):
            problems["NAME"].append((row.line_number, row.name, row.expected))
        for segment_name, segment in (("原文", row.fields[2]), ("替换文", row.fields[3])):
            for escape in stray_escapes(segment):
                problems["STRAY"].append((row.line_number, row.name, segment_name, escape))
    problems["ROW"] = [(row.line_number, row.name, row.path) for row in rows_outside_copy(rows, copied)]
    return problems


def string_literals(text):
    """源码里每一处字符串字面量（"…"、r#"…"#、b"…"）的 (起, 止)，注释里的不算；字面量的止点照 admission.py 的 end_of_literal。"""
    end_of_literal = admission_module().end_of_literal
    position = 0
    while True:
        found = LITERAL_OR_COMMENT_START.search(text, position)
        if found is None:
            return
        start, token = found.start(), found.group(0)
        if token == "//":
            line_end = text.find("\n", start)
            if line_end < 0:
                return
            position = line_end
            continue
        if token == "/*":
            depth, cursor = 0, start
            while cursor < len(text):
                if text.startswith("/*", cursor):
                    depth, cursor = depth + 1, cursor + 2
                elif text.startswith("*/", cursor):
                    depth, cursor = depth - 1, cursor + 2
                    if depth == 0:
                        break
                else:
                    cursor += 1
            position = cursor
            continue
        if token[0] in "br" and start > 0 and (text[start - 1].isalnum() or text[start - 1] == "_"):
            position = start + 1   # 标识符的末一个字母，不是字面量的前缀
            continue
        end = end_of_literal(text, start)
        if end is None:
            position = start + 1
            continue
        if token != "'":
            yield start, end
        position = end


def literal_value(literal):
    """字面量里引号之间的原文：r#"…"#、b"…" 都去掉前缀与引号；转义不还原（路径里用不到）。"""
    raw = re.match(r'^b?r(#*)"(.*)"\1$', literal, re.S)
    if raw:
        return raw.group(2)
    plain = re.match(r'^b?"(.*)"$', literal, re.S)
    return plain.group(1) if plain else literal


def escaping_literals(root, copied):
    """crates/<crate>/ 下每份 .rs（跳过 target）里以 "../ 起头的字符串字面量，按 crates/<crate>/ 解析之后逃出 crates/ 的：
    返回 ([(文件, 行号, 字面量, 解析到的路径, 在不在拷贝范围里)], 扫了几份 .rs)。#[path = …] 里的不算。"""
    found, scanned = [], 0
    crates_directory = os.path.join(root, "crates")
    if not os.path.isdir(crates_directory):
        return found, scanned
    for crate_name in sorted(os.listdir(crates_directory)):
        crate_root = os.path.join(crates_directory, crate_name)
        if not os.path.isdir(crate_root):
            continue
        for current_directory, directory_names, file_names in os.walk(crate_root):
            directory_names[:] = sorted(name for name in directory_names
                                        if not any(fnmatch.fnmatch(name, pattern) for pattern in IGNORED_WHEN_COPYING_INTO_EACH_SHARD))
            for file_name in sorted(file_names):
                if not file_name.endswith(".rs"):
                    continue
                path = os.path.join(current_directory, file_name)
                with open(path, encoding="utf-8", errors="replace") as handle:
                    text = handle.read()
                scanned += 1
                for start, end in string_literals(text):
                    literal = text[start:end]
                    value = literal_value(literal)
                    if not value.startswith("../"):
                        continue
                    if PATH_ATTRIBUTE_BEFORE_LITERAL.search(text[max(0, start - 200):start]) and PATH_ATTRIBUTE_AFTER_LITERAL.match(text, end):
                        continue
                    resolved = os.path.normpath(os.path.join("crates", crate_name, value))
                    if resolved == "crates" or resolved.startswith("crates" + os.sep):
                        continue
                    found.append((os.path.relpath(path, root), text.count("\n", 0, start) + 1, literal, resolved,
                                  is_copied_into_each_shard(resolved, copied)))
    return found, scanned


# ── 判法版本、行键、判定库记录、快照与用时表 ────────────────────────────────────

def read_bytes(path):
    with open(path, "rb") as handle:
        return handle.read()


def judge_version(root, environment, memory_cap_runner):
    """(判法版本, 项数)：见文件头「判定库」。登记表没有 checker-tier-crates-mutation-replay 那一行抛 RegistrationError，读不了、构建环境算不出抛 InputManifestError。"""
    admission = admission_module()
    stage_rows = admission.rows_of_key(admission.read_registration_rows(STAGE_REPOSITORY), STAGE)
    if not stage_rows:
        raise admission.RegistrationError(f"{admission.REGISTRATION_TABLE} 里没有 {STAGE} 这一行")
    registration_text = "".join(row.fingerprint_text() for row in stage_rows).encode("utf-8", "surrogateescape")
    try:
        named_lines = [(f"<登记行：{STAGE}>", registration_text),
                       (f"<阶段脚本：.claude/gate.d/{STAGE}>", read_bytes(STAGE_SCRIPT)),
                       ("<共用模块：scripts/mutation/crates-mutation-rows.py>", read_bytes(MODULE_PATH)),
                       ("<切词与切 fn 项：scripts/mutation/mutation-prune.py>", read_bytes(os.path.join(SCRIPT_DIRECTORY, "mutation-prune.py"))),
                       ("<libtest 结果行：scripts/gate/lib_libtest_result_lines.py>", read_bytes(LIBTEST_RESULT_LINES_PATH)),
                       ("<判定库共用部分：scripts/crash/lib_verdict_library.py>", read_bytes(VERDICT_LIBRARY_PATH)),
                       ("<内存包装：这一趟每条经它跑的那一份>", read_bytes(memory_cap_runner))] + admission.build_environment_lines(root, environment)
    except OSError as error:
        raise admission.InputManifestError(f"读不了判法版本的一项：{error}") from error
    _manifest, fingerprint, _file_count = admission.manifest_of_files(root, [], named_lines)
    return fingerprint, len(named_lines)


def toolchain_text(root, environment):
    """行键里的工具链：cargo -V 与 rustc -V 原样接起来（与 admission.py 的 toolchain_line 同一个口径，只是起在 environment 里）；起不来抛 InputManifestError。
    弄坏开关 rowkeywithouttoolchain 交空串。"""
    if break_is(environment, "rowkeywithouttoolchain"):
        return ""
    admission = admission_module()
    outputs = []
    for command in (["cargo", "-V"], ["rustc", "-V"]):
        try:
            completed = subprocess.run(command, cwd=root, capture_output=True, env=environment, stdin=subprocess.DEVNULL)
        except OSError as error:
            raise admission.InputManifestError(f"{' '.join(command)} 起不来：{error}") from error
        if completed.returncode != 0:
            raise admission.InputManifestError(f"{' '.join(command)} 退 {completed.returncode}")
        outputs.append(completed.stdout)
    return b"".join(outputs).rstrip(b"\n").decode("utf-8", "surrogateescape")


def row_key(row, memory_max, timeout_seconds, toolchain):
    text = "\t".join(row.fields) + f"\nmemory_max={memory_max}\ntimeout_seconds={timeout_seconds}\ntoolchain={toolchain}\n"
    return hashlib.sha256(text.encode("utf-8", "surrogateescape")).hexdigest()


def row_content_hash(row):
    return hashlib.sha256("\t".join(row.fields).encode("utf-8", "surrogateescape")).hexdigest()


def combo_key(row, test_full_name, toolchain):
    """覆盖记录按这个分：`--` 之前的参数、`--` 之后的开关、点名用例的全名、工具链。"""
    before, switches = batch_key(row.arguments)
    text = "\t".join([" ".join(before), " ".join(switches), test_full_name, toolchain])
    return hashlib.sha256(text.encode("utf-8", "surrogateescape")).hexdigest()


def coverage_id(combo, snapshot):
    return hashlib.sha256(f"{combo}\n{snapshot}\n".encode()).hexdigest()


def encode_value(value):
    return str(value).replace("\\", "\\\\").replace("\n", "\\n")


def decode_value(value):
    return re.sub(r"\\(.)", lambda escape: "\n" if escape.group(1) == "n" else escape.group(1), value)


def record_text(fields):
    return RECORD_HEADER + "".join(f"{key}={encode_value(value)}\n" for key, value in fields.items())


def read_record(path):
    """判定库记录或用时表的一格：{键: 值}；不在、读不了交 None（按键读经判定库共用部分的 read_text）。"""
    text = verdict_library.read_text(path)
    if text is None:
        return None
    lines = text.split("\n")
    fields = {}
    for line in lines:
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        fields[key] = decode_value(value)
    return fields


def utc_now_text():
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())


def reuse_refusal(environment):
    """一律不复用的理由（照写新记录）；复用时交 None。"""
    for variable in ("SINGLEFS_GATE_FULL", "GATE_MUTATION_START_OVER"):
        if environment.get(variable, "") == "1":
            return f"{variable}=1"
    return None


def records_home(root, environment):
    """判定库记录与用时表放在哪个目录底下：GATE_MUTATION_RECORDS_HOME 设了就是它，不设是被判那棵树的 git common-dir；取不到抛 InputManifestError。"""
    configured = environment.get("GATE_MUTATION_RECORDS_HOME", "").strip()
    if configured:
        return configured
    admission = admission_module()
    common_directory = admission.git_common_directory(root)
    if common_directory is None:
        raise admission.InputManifestError(f"{root} 不是 git 工作树（取不到 git common-dir）")
    return common_directory


def prune_base_directories(records_root, current):
    """只留最新的 BASE_DIRECTORIES_KEPT 个判法版本目录（这一趟的先 touch 成最新），更早的删掉；只删 records_root 底下名字是 64 位十六进制的目录。
    返回删了的目录名。"""
    current_directory = os.path.join(records_root, current)
    os.makedirs(current_directory, exist_ok=True)
    os.utime(current_directory)
    others = [entry for entry in os.scandir(records_root)
              if entry.is_dir(follow_symlinks=False) and FINGERPRINT_FORM.fullmatch(entry.name) and entry.name != current]
    others.sort(key=lambda entry: (-entry.stat(follow_symlinks=False).st_mtime, entry.name))
    removed = []
    for entry in others[BASE_DIRECTORIES_KEPT - 1:]:
        shutil.rmtree(os.path.join(records_root, entry.name), ignore_errors=True)
        removed.append(entry.name)
    return removed


# ── 快照、覆盖记录与源码副本的存放 ────────────────────────────────────────────

def snapshot_of(base, copied):
    """base 底下拷贝范围里（scripts/ 之外，跳过 target 与 crates/mutations.tsv）每个文件的 {路径: sha256}；照 prepare_shard 拷的那样走（跟符号链接）。"""
    files = {}
    for item in copied:
        if item.startswith(SNAPSHOT_LEFT_OUT_PREFIX):
            continue
        path = os.path.join(base, item)
        if os.path.isfile(path):
            if item != TABLE:
                files[item] = hashlib.sha256(read_bytes(path)).hexdigest()
            continue
        for directory, names, file_names in os.walk(path, followlinks=True):
            names[:] = sorted(name for name in names if not any(fnmatch.fnmatch(name, pattern) for pattern in IGNORED_WHEN_COPYING_INTO_EACH_SHARD))
            for name in sorted(file_names):
                full = os.path.join(directory, name)
                relative = os.path.relpath(full, base)
                if relative == TABLE or not os.path.isfile(full):
                    continue
                files[relative] = hashlib.sha256(read_bytes(full)).hexdigest()
    return files


def snapshot_identity(files):
    return hashlib.sha256("".join(f"{files[path]}  {path}\n" for path in sorted(files)).encode("utf-8", "surrogateescape")).hexdigest()


class Store:
    """一格判法版本目录：判定库记录、覆盖记录、快照与源码副本。读过的覆盖记录与快照留在内存里。"""

    def __init__(self, directory):
        self.directory = directory
        self.coverage_directory = os.path.join(directory, COVERAGE_DIRECTORY_NAME)
        self.snapshot_directory = os.path.join(directory, SNAPSHOT_DIRECTORY_NAME)
        self.blob_directory = os.path.join(directory, BLOB_DIRECTORY_NAME)
        self._coverage, self._snapshots = {}, {}

    def ensure(self):
        for directory in (self.directory, self.coverage_directory, self.snapshot_directory, self.blob_directory):
            os.makedirs(directory, exist_ok=True)

    def has_new_format_records(self):
        """目录里有没有一条带快照的 mutation 记录（没有：头一趟、判法版本刚换、只有旧格式记录）。"""
        try:
            names = os.listdir(self.directory)
        except OSError:
            return False
        return any(name.startswith("mutation.") and (read_record(os.path.join(self.directory, name)) or {}).get("snapshot") for name in names)

    @staticmethod
    def read_json(path):
        try:
            with gzip.open(path, "rt", encoding="utf-8", errors="surrogateescape") as handle:
                return json.load(handle)
        except (OSError, ValueError, EOFError):
            return None

    @staticmethod
    def write_json(path, value):
        write_atomically(path, gzip.compress(json.dumps(value, sort_keys=True, ensure_ascii=False).encode("utf-8", "surrogateescape"), mtime=0))

    def snapshot(self, identity):
        if identity not in self._snapshots:
            value = self.read_json(os.path.join(self.snapshot_directory, f"{identity}.json.gz"))
            self._snapshots[identity] = value.get("files") if isinstance(value, dict) else None
        return self._snapshots[identity]

    def put_snapshot(self, identity, files):
        path = os.path.join(self.snapshot_directory, f"{identity}.json.gz")
        if not os.path.exists(path):
            self.write_json(path, {"files": files})
        self._snapshots[identity] = files

    def coverage(self, identity):
        if identity not in self._coverage:
            value = self.read_json(os.path.join(self.coverage_directory, f"{identity}.json.gz")) if identity else None
            self._coverage[identity] = value if isinstance(value, dict) else None
        return self._coverage[identity]

    def put_coverage(self, identity, value):
        self.write_json(os.path.join(self.coverage_directory, f"{identity}.json.gz"), value)
        self._coverage[identity] = value

    def blob(self, digest):
        try:
            return zlib.decompress(read_bytes(os.path.join(self.blob_directory, digest)))
        except (OSError, zlib.error):
            return None

    def put_blob(self, digest, data):
        path = os.path.join(self.blob_directory, digest)
        if not os.path.exists(path):
            write_atomically(path, zlib.compress(data, 6))


def collect_garbage(store):
    """删掉没有行引的覆盖记录、没有记录引的快照、没有覆盖记录引的源码副本；回 (删的覆盖记录数, 快照数, 源码副本数)。"""
    records = []
    for name in os.listdir(store.directory):
        if name.split(".", 1)[0] in RECORD_KINDS:
            record = read_record(os.path.join(store.directory, name))
            if record is not None:
                records.append(record)
    wanted_coverage = {record.get("coverage") for record in records if record.get("coverage")}
    removed = [0, 0, 0]
    for name in os.listdir(store.coverage_directory):
        if name.endswith(".json.gz") and name[:-len(".json.gz")] not in wanted_coverage:
            with contextlib.suppress(OSError):
                os.unlink(os.path.join(store.coverage_directory, name))
                removed[0] += 1
    wanted_snapshots = {record.get("snapshot") for record in records if record.get("snapshot")}
    wanted_blobs = set()
    for identity in wanted_coverage:
        coverage = store.coverage(identity)
        if coverage is None:
            continue
        wanted_snapshots.add(coverage.get("snapshot"))
        files = store.snapshot(coverage.get("snapshot")) or {}
        wanted_blobs |= {files[path] for path in coverage.get("compiled", {}) if path in files and path.endswith(".rs")}
    for name in os.listdir(store.snapshot_directory):
        if name.endswith(".json.gz") and name[:-len(".json.gz")] not in wanted_snapshots:
            with contextlib.suppress(OSError):
                os.unlink(os.path.join(store.snapshot_directory, name))
                removed[1] += 1
    for name in os.listdir(store.blob_directory):
        if FINGERPRINT_FORM.fullmatch(name) and name not in wanted_blobs:
            with contextlib.suppress(OSError):
                os.unlink(os.path.join(store.blob_directory, name))
                removed[2] += 1
    return tuple(removed)


# ── 复用判据（reuse_decision 与它用的几个纯函数） ──────────────────────────────

_PRUNE_LIBRARY = []


def prune_library():
    """scripts/mutation/mutation-prune.py（切词、切 fn 项、读覆盖），第一次用到时才读进来；它的弄坏开关不在这里认。"""
    if not _PRUNE_LIBRARY:
        specification = importlib.util.spec_from_file_location("mutation_prune_for_crates_mutation_rows", os.path.join(SCRIPT_DIRECTORY, "mutation-prune.py"))
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _PRUNE_LIBRARY.append(module)
    return _PRUNE_LIBRARY[0]


Decision = collections.namedtuple("Decision", "reuse column detail how")   # how：same-snapshot / coverage / None


class Comparison:
    """一份旧快照与此刻之间的比较，按快照号缓存：哪些文件变了、各文件旧新两份的切法、函数外的判定。"""

    def __init__(self, store, old_files, current_files, read_current):
        self.store, self.old, self.current, self.read_current = store, old_files, current_files, read_current
        self.changed = sorted(path for path in set(old_files) | set(current_files) if old_files.get(path) != current_files.get(path))
        self._indexes, self._outside = {}, {}

    def index(self, path, side):
        """旧（"old"，从源码副本读）或新（"new"，从此刻的树读）那一份的 FileIndex；读不到交 None。"""
        digest = (self.old if side == "old" else self.current).get(path)
        if digest is None:
            return None
        if digest not in self._indexes:
            data = self.store.blob(digest) if side == "old" else self.read_current(path)
            self._indexes[digest] = None if data is None else prune_library().FileIndex(path, data.decode("utf-8", "replace"))
        return self._indexes[digest]

    def text(self, path, side):
        index = self.index(path, side)
        return None if index is None else index.text

    def outside_problem(self, path):
        if path not in self._outside:
            self._outside[path] = outside_problem(self.index(path, "old"), self.index(path, "new"))
        return self._outside[path]


def attribute_texts(index, function):
    """fn 项前面那几个 #[…] 属性的词法单元（文档注释不算）。"""
    texts, position = [], function.first
    while position < function.last:
        token = index.tokens[position]
        if token[0] == "doc":
            position += 1
        elif token[1] == "#" and index.text_of(position + 1) == "[" and position + 1 in index.pairs:
            close = index.pairs[position + 1]
            texts.append(" ".join(item[1] for item in index.tokens[position:close + 1]))
            position = close + 1
        else:
            break
    return texts


def is_test_function(index, function):
    """属性里有一个最末一段是 test 的（#[test]、#[tokio::test]）。"""
    for text in attribute_texts(index, function):
        words = text[:-1].replace("#", " ").replace("[", " ").split()
        if words and words[-1] == "test":
            return True
    return False


def has_item_impl(index, function):
    """函数体里有项位置上的 impl（前一个词法单元是 { } ; ] 之一）：函数体里的 impl 块对整个 crate 生效，覆盖看不见。"""
    if function.body_first is None:
        return False
    for position in range(function.body_first + 1, function.last):
        token = index.tokens[position]
        if token[0] == "ident" and token[1] == "impl" and index.tokens[position - 1][1] in ("{", "}", ";", "]"):
            return True
    return False


def outside_problem(old_index, new_index):
    """编进测试二进制的一份 .rs 两份之间：改动有落在函数之外的交那一句，都在函数之内交 None（复用判据「函数外有改动」那一条）。"""
    if old_index is None or new_index is None:
        return "读不到旧的源码副本或此刻那一份"
    if not old_index.ok or not new_index.ok:
        return "括号配不上、切不开函数"
    if old_index.outside_fingerprint != new_index.outside_fingerprint:
        return "函数外的词法单元变了（类型、常量、static、宏、impl 头、use、mod 这类）"
    old_functions = {function.qualified: function for function in old_index.functions}
    new_qualified = {function.qualified for function in new_index.functions}
    for function in new_index.functions:
        if function.qualified in old_functions or is_test_function(new_index, function):
            continue
        parent = function.qualified.rsplit("::", 1)[0] if "::" in function.qualified else None
        if parent is not None and parent in new_qualified:
            continue   # fn 里嵌套的 fn：只有外层调得到
        return f"新加了函数 {function.qualified}"
    for function in new_index.functions:
        old = old_functions.get(function.qualified)
        if old is None or old.fingerprint == function.fingerprint:
            continue
        if attribute_texts(old_index, old) != attribute_texts(new_index, function):
            return f"函数 {function.qualified} 的属性变了"
        if old.is_const or function.is_const:
            return f"const fn {function.qualified} 变了（常量上下文里在编译期求值，覆盖看不见）"
        if has_item_impl(old_index, old) or has_item_impl(new_index, function):
            return f"函数 {function.qualified} 的函数体里有 impl 块（对整个 crate 生效）"
    return None


def path_effect(path, coverage, old_files, current_files, package_directories):
    """一个变了的文件对这一组合算什么：「compiled-rs」（编进去的 .rs、两边都在，要逐函数比）、「outside」（算函数外有改动）、None（不相干）。"""
    compiled = coverage.get("compiled", {})
    if path in compiled:
        if compiled[path] == "runtime" and path.endswith(".rs") and path in old_files and path in current_files:
            return "compiled-rs"
        return "outside"
    involved = coverage.get("packages", [])
    owner = max((directory for directory in package_directories if path.startswith(directory + "/")), key=len, default=None)
    if owner is None:
        return "outside"
    if owner not in involved:
        return None
    if not path.endswith(".rs"):
        return "outside"
    if (path in old_files) != (path in current_files):
        below = path[len(owner) + 1:]
        if any(form.fullmatch(below) for form in AUTO_TARGET_FORMS):
            return "outside"
    return None


def package_directories_of(files):
    """快照里每个不在仓根的 Cargo.toml 所在的目录。"""
    return sorted(os.path.dirname(path) for path in files if path.endswith("/Cargo.toml"))


def mutated_function_problem(row, comparison, coverage, environment):
    """复用判据「被变异的函数变了」：成立（没变）交 None，不成立交那一句。"""
    data = comparison.read_current(row.path) if row.path in comparison.current else None
    if data is None:
        return f"被变异的文件 {row.path} 此刻不在"
    current_text = data.decode("utf-8", "replace")
    if current_text.count(row.old) != 1:
        return f"原文在 {row.path} 里命中 {current_text.count(row.old)} 次，要恰好 1 次"
    if break_is(environment, "mutatedfunctionignored"):
        return None
    if comparison.old.get(row.path) == comparison.current.get(row.path):
        return None
    if coverage.get("compiled", {}).get(row.path) != "runtime" or not row.path.endswith(".rs") or row.path not in comparison.old:
        return f"被变异的文件 {row.path} 变了（不是编进测试二进制的 .rs，按整份 sha256 比）"
    old_index, new_index = comparison.index(row.path, "old"), comparison.index(row.path, "new")
    if old_index is None or new_index is None or not old_index.ok or not new_index.ok:
        return f"{row.path} 切不开函数，按整份比：变了"
    old_start = old_index.text.find(row.old)
    if old_start < 0:
        return f"原文不在覆盖记录那一份 {row.path} 里"
    old_container = old_index.container_at(old_start, old_start + len(row.old))
    new_start = new_index.text.find(row.old)
    new_container = new_index.container_at(new_start, new_start + len(row.old))
    if old_container is None:
        if new_container is not None or old_index.outside_fingerprint != new_index.outside_fingerprint:
            return f"原文在 {row.path} 的函数外，那一部分变了"
        return None
    if new_container is None or new_container.qualified != old_container.qualified:
        return f"原文此刻不在 {old_container.qualified} 里"
    if new_container.fingerprint != old_container.fingerprint:
        return f"{row.path} 的 {old_container.qualified} 变了"
    return None


def executed_function_problem(coverage, comparison, environment):
    """复用判据「执行过的函数变了」：成立交 None，不成立交那一句。"""
    if break_is(environment, "executedfunctionsignored"):
        return None
    library = prune_library()
    changed = set(comparison.changed)
    for path, container in coverage.get("executed", []):
        if path not in changed:
            continue
        old_index, new_index = comparison.index(path, "old"), comparison.index(path, "new")
        if old_index is None or new_index is None:
            return f"执行过的 {path} 没了或读不到"
        if container == library.WHOLE_FILE or not old_index.ok or not new_index.ok:
            return f"执行过的 {path} 切不开函数，按整份比：变了"
        if container == library.OUTSIDE:
            if old_index.outside_fingerprint != new_index.outside_fingerprint:
                return f"执行过的 {path} 函数外那部分变了"
            continue
        old = next((function for function in old_index.functions if function.qualified == container), None)
        new = next((function for function in new_index.functions if function.qualified == container), None)
        if old is None or new is None or old.fingerprint != new.fingerprint:
            return f"执行过的 {path} 的 {container} {'没了' if new is None else '变了'}"
    return None


def outside_change_problem(coverage, comparison, package_directories, environment):
    """复用判据「函数外有改动」：成立交 None，不成立交那一句。"""
    if break_is(environment, "outsidechangesignored"):
        return None
    for path in comparison.changed:
        effect = path_effect(path, coverage, comparison.old, comparison.current, package_directories)
        if effect == "outside":
            state = "新加" if path not in comparison.old else "删掉" if path not in comparison.current else "改了"
            return f"{state} {path}（{'编进测试二进制的' if path in coverage.get('compiled', {}) else '不是函数里能装下的'}改动）"
        if effect == "compiled-rs":
            problem = comparison.outside_problem(path)
            if problem is not None:
                return f"{path}：{problem}"
    return None


def reuse_decision(row, key, record, store, current, comparisons, environment, refusal, store_has_records):
    """这一行这一趟复用还是现跑（文件头「复用判据」）。current 是此刻的 {"id", "files", "packages", "read"}；comparisons 按快照号缓存 Comparison。"""
    if refusal:
        return Decision(False, "start-over", refusal, None)
    if record is None or record.get("kind") != "mutation" or record.get("row_key") != key:
        if not store_has_records:
            return Decision(False, "no-coverage", "这一格记录目录里还没有一条新格式记录（头一趟、判法版本刚换，或只有旧格式记录）", None)
        return Decision(False, "row-changed", "没有这个行键的记录（六段、内存上限、限时或工具链变了，或是新加的行）", None)
    if record.get("verdict") != REUSABLE_VERDICT["mutation"]:
        return Decision(False, "no-coverage", f"上次判的是 {record.get('verdict') or '（没写）'}，只有抓到的记覆盖", None)
    if record.get("snapshot") and record.get("snapshot") == current["id"]:
        return Decision(True, None, "源码与上次一字未动", "same-snapshot")
    coverage = store.coverage(record.get("coverage", ""))
    old_files = store.snapshot(coverage.get("snapshot", "")) if coverage else None
    if coverage is None or old_files is None or record.get("snapshot") != coverage.get("snapshot"):
        return Decision(False, "no-coverage", "抓到了，但没有它那份快照上的覆盖记录（旧格式记录、覆盖没记成或被删了）", None)
    comparison = comparisons.get(coverage["snapshot"])
    if comparison is None:
        comparison = comparisons[coverage["snapshot"]] = Comparison(store, old_files, current["files"], current["read"])
    problem = mutated_function_problem(row, comparison, coverage, environment)
    if problem is not None:
        return Decision(False, "mutated-function", problem, None)
    problem = executed_function_problem(coverage, comparison, environment)
    if problem is not None:
        return Decision(False, "executed-function", problem, None)
    problem = outside_change_problem(coverage, comparison, current["packages"], environment)
    if problem is not None:
        return Decision(False, "outside", problem, None)
    return Decision(True, None, f"覆盖记录那份快照之后改了 {len(comparison.changed)} 个文件，复用判据都成立", "coverage")


def current_state(root, copied):
    """此刻这棵树的快照：{"id", "files", "packages", "read"}（read(路径) 交此刻的字节，读不到交 None）。"""
    files = snapshot_of(root, copied)

    def read(path):
        try:
            return read_bytes(os.path.join(root, path))
        except OSError:
            return None
    return {"id": snapshot_identity(files), "files": files, "packages": package_directories_of(files), "read": read}


def read_timing(timings_directory, content_hash):
    """(上一次跑的秒数, 上一次超没超时)；没有记过交 None。"""
    record = read_record(os.path.join(timings_directory, content_hash))
    if record is None:
        return None
    try:
        return float(record.get("seconds_running", "")), record.get("timed_out") == "1"
    except ValueError:
        return None


# ── 基线的批、派活的次序、libtest 的结果行 ────────────────────────────────────

def split_at_double_dash(arguments):
    words = arguments.split()
    if "--" in words:
        index = words.index("--")
        return words[:index], words[index + 1:]
    return words, []


def batch_key(arguments):
    before, after = split_at_double_dash(arguments)
    return tuple(before), tuple(word for word in after if word.startswith("--"))


def baseline_batches(rows):
    """{批的键: [这一批的行，按表序]}，批按第一行在表里的次序排。"""
    batches = collections.OrderedDict()
    for row in rows:
        batches.setdefault(batch_key(row.arguments), []).append(row)
    return batches


def baseline_arguments(key, rows):
    """这一批基线的 cargo test 参数：键里的两段，加这一批各行过滤词的并集（有一行没有过滤词就一个都不带）。"""
    before, switches = key
    filters = []
    for row in rows:
        row_filters = [word for word in split_at_double_dash(row.arguments)[1] if not word.startswith("-")]
        if not row_filters:
            filters = []
            break
        filters += [word for word in row_filters if word not in filters]
    tail = list(switches) + filters
    return list(before) + (["--"] + tail if tail else [])


def crate_of(path):
    """`crates/<crate 名>/…` 里的 crate 名：同一个工作进程连着改同一个 crate，增量编译才稳。"""
    parts = path.split("/")
    return parts[1] if len(parts) > 2 and parts[0] == "crates" else path


def rows_grouped_by_crate(rows):
    """按 crate 归堆、堆内保持表序，再把几堆首尾相接（行多的堆在前）。"""
    groups = {}
    for row in rows:
        groups.setdefault(crate_of(row.path), []).append(row)
    ordered = []
    for crate_name in sorted(groups, key=lambda name: (-len(groups[name]), name)):
        ordered.extend(groups[crate_name])
    return ordered


def dispatch_order(rows, timings_directory, long_row_seconds, environment):
    """(派变异的次序, 前移了几条)：上一次用时不短于 long_row_seconds 或上一次超时的行先派、按用时从长到短，其余照按 crate 归堆的次序。"""
    if break_is(environment, "timingsignored"):
        return rows_grouped_by_crate(rows), 0
    long_rows = []
    for row in rows:
        timing = read_timing(timings_directory, row_content_hash(row))
        if timing is not None and (timing[1] or timing[0] >= long_row_seconds):
            long_rows.append((timing[0], row))
    long_rows.sort(key=lambda item: (-item[0], item[1].line_number))
    moved_lines = {row.line_number for _seconds, row in long_rows}
    return [row for _seconds, row in long_rows] + rows_grouped_by_crate([row for row in rows if row.line_number not in moved_lines]), len(long_rows)


def results_by_test(expected, output):
    """libtest 的结果行 `test <模块路径>::<名字> ... ok|FAILED|ignored` 里对上点名的那几条：{完整名: [结果…]}。
    #[should_panic] 的用例在名字与 ` ... ` 之间多一段 ` - should panic`；点名可以省掉模块路径。解析在 lib_libtest_result_lines.py（全仓一份）。"""
    return libtest_result_lines.results_of_named_test(expected, output)


def output_tail(output, lines=8):
    return "\n".join(output.splitlines()[-lines:])


def failure_excerpt(output, full_name):
    """libtest 给这一条打的 `---- <名字> stdout ----` 那一段的头几行；没有就交 None。"""
    marker = f"---- {full_name} stdout ----"
    index = output.find(marker)
    if index < 0:
        return None
    return "\n".join(output[index:].splitlines()[:6])


def wrapper_outcome(returncode, memory_max, timeout_seconds):
    """包装自己的几种结局：(档, 那一句)；是命令自己的退出码交 None。"""
    outcomes = {
        TIME_LIMIT_HIT_EXIT: ("timeout", f"{timeout_seconds} 秒没跑完（超时）"),
        MEMORY_CAP_HIT_EXIT: ("memory", f"撞了内存上限 {memory_max}（内存撞顶）"),
        SLICE_TOTAL_HIT_EXIT: ("squeezed", "被 slice 的总上限挤掉（整个 slice 满了，内核挑了这一条杀）"),
        MEMORY_ADMISSION_REFUSED_EXIT: ("refused", "内存不够排不上（包装等满了等待上限），没跑"),
        MEMORY_CAP_UNAVAILABLE_EXIT: ("unavailable", "带内存上限的 scope 起不来，没跑"),
    }
    if not break_is(os.environ, "gpumemoryasoutput"):
        outcomes[GPU_MEMORY_HIT_EXIT] = ("gpumemory", "显存不够（命令起止之间内核报了 NVRM … NV_ERR_NO_MEMORY，测试是被它弄红的）")
    return outcomes.get(returncode)


def baseline_problem(row, returncode, output, memory_max, timeout_seconds):
    """这一行的点名测试在基线输出里不是「恰好一个用例、结果都是 ok」时交那一句（接在「在没改坏的副本上」后面），是的交 None。"""
    outcome = wrapper_outcome(returncode, memory_max, timeout_seconds)
    if outcome is not None:
        return f"没判成：那一批{outcome[1]}"
    results = results_by_test(row.expected, output)
    if len(results) == 1 and all(result == "ok" for result in next(iter(results.values()))):
        return None
    if len(results) > 1:
        return f"对上了 {len(results)} 个不同的用例（{'、'.join(sorted(results))}），判不出是哪一个；把点名写全到模块路径"
    if results:
        full_name, outcomes = next(iter(results.items()))
        return f"的结果是 {'、'.join(outcomes)}（{full_name}）"
    if re.search(r"^error(\[E\d+\])?: ", output, re.M) or "could not compile" in output:
        return f"一行都没出现：那一批编不过（退出码 {returncode}）"
    return f"一行都没出现（那一批退出码 {returncode}；参数筛不到它，或名字认不出）"


def with_shown_output(arguments):
    """变异那一趟的 cargo test 参数：libtest 参数里补 --show-output（插在第一个 `--` 之后；没有 `--` 就补 `-- --show-output`；已经有了不再补）。"""
    words = list(arguments)
    if "--" not in words:
        return words + ["--", LIBTEST_SHOW_OUTPUT]
    index = words.index("--")
    if LIBTEST_SHOW_OUTPUT in words[index + 1:]:
        return words
    return words[:index + 1] + [LIBTEST_SHOW_OUTPUT] + words[index + 1:]


def not_run_reason(expected, output):
    """输出里 NOT_RUN_NO_GPU_ADAPTER 那一行的 test= 对上点名的测试（只比 :: 分开的末一段）时交它 reason= 之后那一句（没写交「没写为什么」），对不上交 None。"""
    wanted = expected.split("::")[-1]
    for found in NOT_RUN_LINE.finditer(output):
        if found.group(1).split("::")[-1] == wanted:
            return (found.group(2) or "").strip() or "没写为什么"
    return None


def verdict_of_run(row, returncode, output, memory_max, timeout_seconds, environment=None):
    """一条变异跑完之后判哪一档：(档, 那一条要打印的话)。environment 只看弄坏开关（notrunasmissed）。"""
    tail = output_tail(output)
    outcome = wrapper_outcome(returncode, memory_max, timeout_seconds)
    # 包装报的几种结局先判：被停、被杀的测试二进制可能已经打出了半截输出，不许拿它判抓到或没红
    if outcome is not None:
        verdict, sentence = outcome
        extra = {"timeout": f"，点名的测试 {row.expected} 没来得及红", "memory": f"，点名的测试 {row.expected} 没来得及红",
                 "squeezed": "，这一条的结果不算数", "gpumemory": "，这一条的结果不算数"}.get(verdict, "")
        return verdict, f"{TABLE}:{row.line_number} {row.name}：{sentence}{extra}\n{tail}"
    # 点名省掉模块路径时在输出里只许对上一个用例：对上两个就分不出红的是不是点名的那一个，按没判成记，不按「有一个红了」算抓到。
    results = results_by_test(row.expected, output)
    if len(results) > 1:
        return "failure", (f"{TABLE}:{row.line_number} {row.name}：点名的测试 {row.expected} 在输出里对上了 {len(results)} 个不同的用例"
                           f"（{'、'.join(sorted(results))}），判不出红的是哪一个；把点名写全到模块路径\n{tail}")
    if returncode != 0 and any("FAILED" in outcomes for outcomes in results.values()):
        return "caught", caught_text(row)
    # 点名的测试是 ok、又打了 NOT_RUN_NO_GPU_ADAPTER：没跑到被改坏的那一处，记本次未跑（文件头「本次未跑」），不记没红
    reason = not_run_reason(row.expected, output) if results and not break_is(environment or {}, "notrunasmissed") else None
    if reason is not None and all(outcome == "ok" for outcomes in results.values() for outcome in outcomes):
        return "notrun", (f"{TABLE}:{row.line_number} {row.name}：本次未跑——点名的测试 {row.expected} 打了 NOT_RUN_NO_GPU_ADAPTER（{reason}）后返回成功，"
                          "没跑到被改坏的那一处；不算抓到、也不算没红")
    if results:
        return "failure", f"{TABLE}:{row.line_number} {row.name}：{row.expected} 没红（退出码 {returncode}）\n{tail}"
    if re.search(r"^error(\[E\d+\])?: ", output, re.M) or "could not compile" in output:
        # 替换文写进源码之后编不过：既不算被抓也不算没红，是一条无效变异（.claude/rules/mutation-sampling.md 第八类）。
        return "invalid", (f"{TABLE}:{row.line_number} {row.name}：点名的测试 {row.expected} 一行都没出现，输出里有 error 行"
                           f"（替换文编不过，或测试进程在跑到它之前被杀，或名字认不出；退出码 {returncode}）\n{tail}")
    return "failure", f"{TABLE}:{row.line_number} {row.name}：点名的测试 {row.expected} 没跑到（退出码 {returncode}）\n{tail}"


def caught_text(row, how=""):
    """抓到那一行；how 是「（现判：<栏>）」或「（按覆盖复用）」，判定本身在它前面（比对两趟的判定时把它去掉，selftest_verdict_view）。"""
    return f"  ✓ {row.name}：{row.expected} 红了{how}"


def single_full_name(expected, output):
    """输出里点名的测试恰好对上一个用例时交它的全名（带模块路径），不然交 None。"""
    results = results_by_test(expected, output)
    return next(iter(results)) if len(results) == 1 else None


# ── 工作进程 ──────────────────────────────────────────────────────────────────

WORKER_STATE = {}


class StopRequested(SystemExit):
    """run_pool 收到 STOP_SIGNALS 里的信号：照 SystemExit 往外抛（退出码 128 + 信号号），一路上的 finally 都跑。"""


def raise_stop_requested(signal_number, _frame):
    raise StopRequested(128 + signal_number)


@contextlib.contextmanager
def own_workers_stopped_on_stop(pool, shard_root):
    """收到停的信号时，在进程池等工作进程收工之前先停自己起的那几个工作进程（只对它们的进程号发 TERM），再删 shard_root，再往外抛。"""
    try:
        yield
    except StopRequested:
        for process in list((getattr(pool, "_processes", None) or {}).values()):
            with contextlib.suppress(OSError, ValueError):
                process.terminate()
        shutil.rmtree(shard_root, ignore_errors=True)
        raise


def init_worker(sequence_counter, shard_root, context):
    """工作进程启动时跑一次：领一个稳定的片号、建自己的源码副本与编译目录。片号从共用编译目录那一层的锁文件领（claim_worker_slot，跨调用不重），
    活是动态领的（checker-tier-crates-mutation-replay 文件头）。sequence_counter 只在拿不到锁文件、或弄坏开关 local-shard-numbering 下当本地序号用。"""
    # 父进程在 run_pool 里把 STOP_SIGNALS 接成「收尾再退」；工作进程照默认（收到就停），不把父进程的接法带进来
    for signal_number in STOP_SIGNALS:
        signal.signal(signal_number, signal.SIG_DFL)
    worker_sequence, slot_lock = claim_worker_slot(context["shared_target_directory"], sequence_counter, context["environment"])
    WORKER_STATE["slot_lock"] = slot_lock   # 拿到进程退出：内核放，别的调用的工作进程才试得到这一片
    work, target_directory = prepare_shard(worker_sequence, shard_root, context)
    note(f"  … 工作进程 {os.getpid()} 领到片号 {worker_sequence}（编译目录 {target_directory}；"
         + (f"片号锁 {slot_lock.name}" if slot_lock is not None else "没有片号锁，按本地序号") + "）")
    environment = dict(context["environment"])
    environment["CARGO_TARGET_DIR"] = target_directory
    # 在用的标记（WORKER_TARGET_LOCK_NAME）：进程退出内核就放，别的一趟回收时拿不到排他锁就不删这份
    WORKER_STATE["target_lock"] = hold_worker_target_lock(target_directory)
    # 这份副本自己的快照：记录里写的是这一份（开跑之后主工作区又改了，记录照实写跑的是哪一份）
    snapshot_files = snapshot_of(work, context["copied"])
    snapshot = snapshot_identity(snapshot_files)
    store = Store(context["records_directory"])
    store.put_snapshot(snapshot, snapshot_files)
    WORKER_STATE.update(work=work, environment=environment, context=context, shard_root=shard_root, worker_sequence=worker_sequence,
                        per_mutation_temporary_root=os.path.join(shard_root, PER_MUTATION_TEMPORARY_DIRECTORY_NAME),
                        snapshot=snapshot, snapshot_files=snapshot_files, store=store, file_indexes={})
    # 源码副本建在父进程的 shard_root 底下、由父进程收尾时整个删；编译目录留着跨轮复用。
    # 不在这里挂 atexit：进程池的工作进程走 os._exit 退出，atexit 在这里一次都不跑。


def slot_lock_path(shared, sequence):
    """片号 sequence 的锁文件：第 1 片 <共用目录>.slot，其余片 <共用目录>-w<片号>.slot，都在共用编译目录那一层里。"""
    base = os.path.normpath(shared)
    return base + WORKER_SLOT_LOCK_SUFFIX if sequence == 1 else f"{base}-w{sequence}{WORKER_SLOT_LOCK_SUFFIX}"


def claim_worker_slot(shared, local_counter, environment):
    """领片号：从 1 往上，对每一片的锁文件试拿排他 flock（不阻塞），拿到第一个空闲的就是这个工作进程的片号。回 (片号, 锁文件句柄或 None)。
    锁文件拿在句柄上，工作进程退出内核放；同一台机器上并发的几次调用因此拿到不相交的片号集合，缓存目录（-w<片号>）照旧跨轮复用。
    锁文件建不了、或 WORKER_SLOT_SEARCH_LIMIT 片都被占：报一行，退回本地序号（本调用内不重，跨调用可能重）。
    弄坏开关 local-shard-numbering：不看锁文件，照改前从本调用的共享计数器现领序号。"""
    import fcntl
    if not break_is(environment, "local-shard-numbering"):
        reason = None
        try:
            os.makedirs(os.path.dirname(os.path.normpath(shared)) or ".", exist_ok=True)
            for sequence in range(1, WORKER_SLOT_SEARCH_LIMIT + 1):
                handle = open(slot_lock_path(shared, sequence), "a")
                try:
                    fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
                except OSError:
                    handle.close()
                    continue
                return sequence, handle
            reason = f"前 {WORKER_SLOT_SEARCH_LIMIT} 片的锁都被占着"
        except OSError as error:
            reason = f"锁文件拿不了（{error}）"
        note(f"  ! {reason}：片号退回本调用内的本地序号，同一台机器上并发的别的调用可能撞同一个 -w<片号>、被 cargo 的目录锁互相排队")
    with local_counter.get_lock():
        local_counter.value += 1
        return local_counter.value, None


def prepare_shard(worker_sequence, shard_root, context):
    """一个工作进程自己的一份源码副本加一个编译目录。第 1 片用共用的编译目录（跨轮热着），其余片 -w<片号> 自己冷编译一次，不从共用目录拷种子。
    ⚠️ 拷的时候不留原来的修改时刻（shutil.copy，不用 copy2）：编译目录跨轮复用，里面最后一次编的可能是上一轮改坏的源码、
    带着上一轮副本的 CARGO_MANIFEST_DIR；副本留着原来较早的修改时刻，cargo 就判它不用重编，没改坏的基线跑的是上一轮改坏的二进制
    （2026-09-28 样本里实测：基线报 attempt to subtract with overflow、读 litmus 读到上一轮已经删掉的副本路径）。
    修改时刻取拷的那一刻，每片头一次编译把工作区的几个 crate 重编一遍（外部依赖不在副本里，照旧热着）。"""
    work = tempfile.mkdtemp(prefix=f"shard-{worker_sequence}-", dir=shard_root)
    for item in context["copied"]:
        source = os.path.join(context["root"], item)
        if not os.path.exists(source):
            continue
        if item == CARGO_CONFIGURATION_DIRECTORY and break_is(context["environment"], "cargoconfignotcopied"):
            continue   # 弄坏开关（文件头）：照改前的拷贝范围，副本里没有 .cargo/
        target = os.path.join(work, item)
        if os.path.isdir(source):
            shutil.copytree(source, target, ignore=shutil.ignore_patterns(*IGNORED_WHEN_COPYING_INTO_EACH_SHARD), copy_function=shutil.copy)
        else:
            os.makedirs(os.path.dirname(target), exist_ok=True)
            shutil.copy(source, target)
    shared = context["shared_target_directory"]
    return work, (shared if worker_sequence == 1 else f"{shared}-w{worker_sequence}")


def expanded_home_path(path, environment):
    """~ 开头的按 environment 里的 HOME 展开（自证给的是自己的 HOME），没有 HOME 再按本进程的。"""
    if path == "~" or path.startswith("~/"):
        return (environment.get("HOME") or os.path.expanduser("~")) + path[1:]
    return path


def default_shared_target_directory(environment):
    """第 1 片的编译目录（其余片加 -w<片号>，记覆盖的再加 -coverage）：GATE_MUTATION_TARGET_DIR 设了用它；GATE_CROSS_RUN_TMPDIR 设了放它底下；
    都没设放 ${SINGLEFS_CROSS_RUN_CACHE_HOME:-~/.local/share/singlefs/cross-run-cache} 底下，不退回 $TMPDIR 或 /tmp。
    弄坏开关 targetdefaulttmp：都没设时照改前退回 ${TMPDIR:-/tmp}。"""
    explicit = environment.get("GATE_MUTATION_TARGET_DIR", "").strip()
    if explicit:
        return explicit
    cross_run = environment.get("GATE_CROSS_RUN_TMPDIR", "").strip()
    if cross_run:
        return os.path.join(cross_run, TARGET_DIRECTORY_NAME)
    if break_is(environment, "targetdefaulttmp"):
        return os.path.join(environment.get("TMPDIR") or "/tmp", TARGET_DIRECTORY_NAME)
    home = environment.get(CROSS_RUN_CACHE_HOME_VARIABLE, "").strip() or DEFAULT_CROSS_RUN_CACHE_HOME
    return os.path.join(expanded_home_path(home, environment), TARGET_DIRECTORY_NAME)


def hold_worker_target_lock(target_directory):
    """工作进程在自己的编译目录里拿共享锁、拿着到进程退出；建不了只报不停（回收那一侧还有按进程认的那几样）。"""
    import fcntl
    try:
        os.makedirs(target_directory, exist_ok=True)
        handle = open(os.path.join(target_directory, WORKER_TARGET_LOCK_NAME), "a")
        fcntl.flock(handle, fcntl.LOCK_SH)
        return handle
    except OSError as error:
        note(f"  ! 编译目录 {target_directory} 里拿不到在用的标记 {WORKER_TARGET_LOCK_NAME}（{error}）：别的一趟回收时只按进程认它在不在用")
        return None


def path_mentioned_in(text, path):
    """text 里出现 path 本身或它底下的路径（后面接 / 或结尾，或不是路径名里的字符）：-w3 不算 -w30、也不算 -w3-coverage。"""
    return re.search(re.escape(path) + r"(?![A-Za-z0-9_.-])", text) is not None


def processes_using_directories(directories):
    """{目录: [「进程 <pid>（<命令>）的<哪一样>」…]}：工作目录在它底下、命令行里有它或它底下的路径、环境里 CARGO_TARGET_DIR 指到它底下的进程。
    读不了的进程（刚退出、别的用户的）跳过；本进程不算。"""
    forms = {directory: {os.path.normpath(directory), os.path.realpath(directory)} for directory in directories}
    users = {directory: [] for directory in directories}
    for entry in os.listdir("/proc"):
        if not entry.isdigit() or int(entry) == os.getpid():
            continue
        base = os.path.join("/proc", entry)
        try:
            with open(os.path.join(base, "cmdline"), "rb") as handle:
                arguments = [word.decode(errors="replace") for word in handle.read().split(b"\0") if word]
        except OSError:
            continue
        try:
            working_directory = os.readlink(os.path.join(base, "cwd"))
        except OSError:
            working_directory = None
        target_variable = None
        try:
            with open(os.path.join(base, "environ"), "rb") as handle:
                for item in handle.read().split(b"\0"):
                    if item.startswith(b"CARGO_TARGET_DIR="):
                        target_variable = item[len(b"CARGO_TARGET_DIR="):].decode(errors="replace")
        except OSError:
            pass
        if target_variable and not os.path.isabs(target_variable) and working_directory:
            target_variable = os.path.join(working_directory, target_variable)
        shown = " ".join(arguments)[:120]
        for directory, paths in forms.items():
            reasons = []
            if working_directory and any(working_directory == path or working_directory.startswith(path + os.sep) for path in paths):
                reasons.append("工作目录")
            if any(path_mentioned_in(argument, path) for argument in arguments for path in paths):
                reasons.append("命令行")
            if target_variable and any(os.path.normpath(target_variable) == path or os.path.normpath(target_variable).startswith(path + os.sep)
                                       for path in paths):
                reasons.append("CARGO_TARGET_DIR")
            if reasons:
                users[directory].append(f"进程 {entry}（{shown}）的{'、'.join(reasons)}")
    return users


def worker_lock_is_held(target_directory):
    """那份编译目录里的在用标记被别的进程拿着：(True, None)；没被拿着：(False, 拿着排他锁的文件或 None)，调用方删完再关。"""
    return file_lock_is_held(os.path.join(target_directory, WORKER_TARGET_LOCK_NAME))


def file_lock_is_held(path):
    """path 这把锁被别的进程拿着：(True, None)；没被拿着（或文件不在、打不开）：(False, 拿着排他锁的文件或 None)，调用方删完再关。不建文件。"""
    import fcntl
    if not os.path.exists(path):
        return False, None
    try:
        handle = open(path, "a")
    except OSError:
        return False, None
    try:
        fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except OSError:
        handle.close()
        return True, None
    return False, handle


def allocated_bytes_of_tree(directory):
    total = 0
    for current_directory, subdirectory_names, file_names in os.walk(directory):
        for name in [*file_names, *subdirectory_names]:
            with contextlib.suppress(OSError):
                total += os.lstat(os.path.join(current_directory, name)).st_blocks * 512
    return total


def format_size(byte_count):
    for unit, size in (("GiB", 1 << 30), ("MiB", 1 << 20), ("KiB", 1 << 10)):
        if byte_count >= size:
            return f"{byte_count / size:.1f} {unit}"
    return f"{byte_count} B"


def reclaim_extra_target_directories(shared, worker_count, environment):
    """开跑时回收片号比这一趟的工作进程数大的 -w<片号> 编译目录（连同它的 -coverage）：编译目录数按这一趟的工作进程数复用，多出来的删掉。
    删之前核没人在用：有进程的工作目录、命令行或 CARGO_TARGET_DIR 指到它，或它（记覆盖的看它那一片）的在用标记被别的进程拿着，就留着、写明谁在用。
    回 (删掉的 [(路径, 占盘字节)], 留着的 [(路径, 谁在用)], 删不成的 [(路径, 原因)])。
    弄坏开关 extratargetskept：一份都不回收；inusetargetremoved：不核在不在用，多出来的照删。"""
    removed, kept, failed = [], [], []
    if break_is(environment, "extratargetskept"):
        return removed, kept, failed
    parent, base = os.path.split(os.path.normpath(shared))
    form = re.compile(re.escape(base) + r"-w([1-9][0-9]*)(" + re.escape(COVERAGE_TARGET_SUFFIX) + r")?$")
    try:
        names = os.listdir(parent or ".")
    except OSError:
        return removed, kept, failed
    extra = sorted(((int(match.group(1)), name) for name in names
                    for match in [form.match(name)] if match and int(match.group(1)) > worker_count and os.path.isdir(os.path.join(parent, name))))
    if not extra:
        return removed, kept, failed
    paths = [os.path.join(parent, name) for _sequence, name in extra]
    check_users = not break_is(environment, "inusetargetremoved")
    users = processes_using_directories(paths) if check_users else {path: [] for path in paths}
    for (sequence, _name), path in zip(extra, paths):
        reasons = list(users[path])
        lock_handles = []
        if check_users:
            held, lock_handle = worker_lock_is_held(os.path.join(parent, f"{base}-w{sequence}"))
            lock_handles.append(lock_handle)
            if held:
                reasons.append(f"-w{sequence} 的在用标记 {WORKER_TARGET_LOCK_NAME} 被别的进程拿着（有一趟的工作进程在用）")
            # 片号锁（claim_worker_slot）：别的调用的工作进程刚领到这一片、还没来得及拿在用标记时也留着；拿到的话删的时候一直拿着，删完再放
            held, lock_handle = file_lock_is_held(slot_lock_path(shared, sequence))
            lock_handles.append(lock_handle)
            if held:
                reasons.append(f"-w{sequence} 的片号锁 {os.path.basename(slot_lock_path(shared, sequence))} 被别的进程拿着（有一趟的工作进程领了这一片）")
        try:
            if reasons:
                kept.append((path, "；".join(reasons)))
                continue
            size = allocated_bytes_of_tree(path)
            shutil.rmtree(path)
            removed.append((path, size))
        except OSError as error:
            failed.append((path, str(error)))
        finally:
            for lock_handle in lock_handles:
                if lock_handle is not None:
                    lock_handle.close()
    return removed, kept, failed


def target_reclaim_lines(shared, worker_count, outcome):
    removed, kept, failed = outcome
    lines = [f"  … 编译目录：{shared}（第 1 片；其余片加 -w<片号>，记覆盖的再加 {COVERAGE_TARGET_SUFFIX}）；"
             + (f"回收了片号大于这一趟 {worker_count} 个工作进程的 {len(removed)} 份，共 {format_size(sum(size for _path, size in removed))}"
                f"（{'、'.join(os.path.basename(path) for path, _size in removed)}）" if removed
                else f"片号大于这一趟 {worker_count} 个工作进程、要回收的 0 份")
             + (f"；有进程在用、没删 {len(kept)} 份" if kept else "") + (f"；删不成 {len(failed)} 份" if failed else "")]
    lines += [f"    没删 {path}：{reasons}" for path, reasons in kept]
    lines += [f"    删不成 {path}：{reason}（下一趟开跑时再试；急着要空间就核没人在用之后手删）" for path, reason in failed]
    return lines


def measure_and_remove_temporary_directory(directory, context):
    """数下一条的 TMPDIR 里有几个文件、占盘与表观各多少字节，再整个删掉。回 (文件数, 占盘字节, 表观字节, 删完之后它还在不在)。"""
    file_count = allocated_bytes = apparent_bytes = 0
    for current_directory, _subdirectory_names, file_names in os.walk(directory):
        for file_name in file_names:
            try:
                file_status = os.lstat(os.path.join(current_directory, file_name))
            except OSError:
                continue   # 数的时候已经没了（还没退干净的进程在收尾），不算
            file_count += 1
            allocated_bytes += file_status.st_blocks * BYTES_PER_STAT_BLOCK
            apparent_bytes += file_status.st_size
    if not break_is(context["environment"], "keepmutationtmp"):
        shutil.rmtree(directory, ignore_errors=True)
    return file_count, allocated_bytes, apparent_bytes, os.path.lexists(directory)


# 内存包装排队时报的那几行（run-with-memory-cap.sh：「排队（已等 …」每 60 秒一行，「排了 … 秒，起跑」放行时一行）
WRAPPER_QUEUE_LINE = re.compile(r"^run-with-memory-cap: 排(队（|了 )")


def run_forwarding_queue_lines(command, label, sink=None, **popen_keywords):
    """跑一条经内存包装的命令、收它的 stdout 与 stderr，交回 subprocess.CompletedProcess（与 capture_output=True, text=True, errors="replace" 收的逐字相同）；
    stderr 里包装排队报的那几行（WRAPPER_QUEUE_LINE）边跑边转打到 sink（默认本进程的 stderr），行首带 label，不进 stdout（文件头「用时」一段之后那一段）。"""
    sink = sink if sink is not None else sys.stderr
    forward = not break_is(popen_keywords.get("env") or os.environ, "queuelineswallowed")
    process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, errors="replace", **popen_keywords)
    stderr_lines = []

    def pump():
        for line in process.stderr:
            stderr_lines.append(line)
            if forward and WRAPPER_QUEUE_LINE.match(line):
                print(f"  … {label}：{line.rstrip()}", file=sink, flush=True)

    reader = threading.Thread(target=pump, name="wrapper-stderr", daemon=True)
    reader.start()
    stdout = process.stdout.read()
    process.wait()
    reader.join()
    return subprocess.CompletedProcess(command, process.returncode, stdout, "".join(stderr_lines))


def run_under_wrapper(command, temporary_directory, stamp_name, extra_environment=None):
    """经内存包装跑一条命令（cargo test --offline … 这一类）：(subprocess 的结果, 用时)。限时交给包装（从起跑算、不算排队），TMPDIR 是这一次自己的；
    extra_environment 只加在这一次的环境里（覆盖那一格的编译目录、插桩开关与 LLVM_PROFILE_FILE）。"""
    state, context = WORKER_STATE, WORKER_STATE["context"]
    stamp_path = os.path.join(state["shard_root"], START_STAMP_DIRECTORY_NAME, stamp_name)
    environment = dict(state["environment"], TMPDIR=temporary_directory,
                       RUN_WITH_MEMORY_CAP_TIME_LIMIT=str(context["timeout_seconds"]),
                       RUN_WITH_MEMORY_CAP_KILL_GRACE=str(TIMEOUT_KILL_GRACE_SECONDS),
                       RUN_WITH_MEMORY_CAP_KEY=" ".join(command))
    environment.update(extra_environment or {})
    # 会话里的「日常一条」：每次包装带这个工作进程自己的并行理由，调用方设了的原样传、不覆盖；只设在这一次包装的环境里（文件头「并行理由」一段）
    if not state["environment"].get("RUN_WITH_MEMORY_CAP_PARALLEL_REASON") or break_is(context["environment"], "overridecallerreason"):
        if not break_is(context["environment"], "noparallelreason"):
            environment["RUN_WITH_MEMORY_CAP_PARALLEL_REASON"] = (
                f"同一次 checker-tier-crates-mutation-replay（本模块进程 {context['stage_process']}）的第 {state['worker_sequence']} 个工作进程，"
                f"各用各的源码副本与编译目录（这一轮共 {context['worker_count']} 个）")
    called = time.time()
    kind, _dash, line_number = stamp_name.partition("-")
    label = {"baseline": f"基线（批首行是变异表第 {line_number} 行）", "coverage": f"覆盖（组首行是变异表第 {line_number.split('-')[0]} 行）"}.get(
        kind, f"变异表第 {line_number} 行")
    run = run_forwarding_queue_lines(["bash", context["memory_cap_runner"], context["memory_max"], "bash", "-c", START_STAMP_SCRIPT, "started-at", stamp_path]
                                     + command, label, cwd=state["work"], env=environment)
    returned = time.time()
    started = None
    with contextlib.suppress(OSError, ValueError):
        with open(stamp_path, encoding="utf-8") as handle:
            started = float(handle.read().strip().replace(",", "."))
    with contextlib.suppress(OSError):
        os.unlink(stamp_path)
    queued = (started - called) if started is not None else (returned - called)
    running = (returned - started) if started is not None else 0.0
    return run, {"seconds_total": f"{returned - called:.3f}", "seconds_queued": f"{max(0.0, queued):.3f}",
                 "seconds_running": f"{max(0.0, running):.3f}"}


def write_record_quietly(kind, key, fields):
    """写一条判定库记录；写不进去交那一句（交回父进程报在 stderr，下一趟这一条照跑），写成交 None。"""
    path = os.path.join(WORKER_STATE["context"]["records_directory"], f"{kind}.{key}")
    try:
        write_atomically(path, record_text(fields))
    except OSError as error:
        return f"{path}：{error}"
    return None


def judge_batch(batch):
    """进程池里的一格活：一批基线（没改坏的副本上跑一遍这一批各行点名的测试）。回 (每行的 (行号, 问题或 None), 那一批的尾巴, 写不进去的记录)。"""
    key, batch_rows = batch
    context = WORKER_STATE["context"]
    arguments = baseline_arguments(key, batch_rows)
    temporary_directory = tempfile.mkdtemp(prefix=f"baseline-{batch_rows[0].line_number}-", dir=WORKER_STATE["per_mutation_temporary_root"])
    try:
        run, seconds = run_under_wrapper(["cargo", "test", "--offline"] + arguments, temporary_directory, f"baseline-{batch_rows[0].line_number}")
    finally:
        measure_and_remove_temporary_directory(temporary_directory, context)
    output = run.stdout + run.stderr
    judged, record_problems = [], []
    for row in batch_rows:
        problem = baseline_problem(row, run.returncode, output, context["memory_max"], context["timeout_seconds"])
        excerpt = None
        if problem is not None:
            results = results_by_test(row.expected, output)
            excerpt = failure_excerpt(output, next(iter(results))) if len(results) == 1 else None
        judged.append((row.line_number, problem, excerpt or output_tail(output)))
        failed_to_write = write_record_quietly("baseline", context["row_keys"][row.line_number], dict(
            kind="baseline", judge_version=context["judging"], row_key=context["row_keys"][row.line_number],
            row_content_hash=row_content_hash(row), table_line=row.line_number, name=row.name, file=row.path,
            memory_max=context["memory_max"], timeout_seconds=context["timeout_seconds"], snapshot=WORKER_STATE["snapshot"],
            test_full_name=single_full_name(row.expected, output) or "",
            verdict="ok" if problem is None else "not-ok", problem=problem or "", batch_arguments=" ".join(arguments),
            batch_exit_code=run.returncode, finished_utc=utc_now_text(), **seconds))
        if failed_to_write:
            record_problems.append(failed_to_write)
    return judged, record_problems


def judge_row(row):
    """进程池里的一格活：一条变异。建它自己的 TMPDIR、改坏那一处、跑点名的测试、还原、删掉那个 TMPDIR，判完立刻写判定库记录与用时表。
    回 (((行号, 档, 那一条要打印的话, 删 TMPDIR 的结果), {snapshot, test_full_name}), 写不进去的记录)。"""
    context = WORKER_STATE["context"]
    full = os.path.join(WORKER_STATE["work"], row.path)
    with open(full, encoding="utf-8") as handle:
        pristine = handle.read()
    assert pristine.count(row.old) == 1, (row.path, row.name)
    temporary_directory = tempfile.mkdtemp(prefix=f"line-{row.line_number}-", dir=WORKER_STATE["per_mutation_temporary_root"])
    with open(full, "w", encoding="utf-8") as handle:
        handle.write(pristine.replace(row.old, row.new))
    try:
        run, seconds = run_under_wrapper(["cargo", "test", "--offline"] + with_shown_output(row.arguments.split()), temporary_directory, f"line-{row.line_number}")
    finally:
        with open(full, "w", encoding="utf-8") as handle:
            handle.write(pristine)
        # 不论退出码是几都删（含限时、撞顶被杀）：被杀的测试 Drop 守卫不跑，它建的镜像只有这里删得掉
        removal = measure_and_remove_temporary_directory(temporary_directory, context)
    output = run.stdout + run.stderr
    verdict, text = verdict_of_run(row, run.returncode, output, context["memory_max"], context["timeout_seconds"], context["environment"])
    if verdict in VERDICTS_KILLED_BY_WRAPPER:
        file_count, allocated_bytes, _apparent_bytes, is_still_there = removal
        first_line, line_break, rest = text.partition("\n")
        text = (first_line + f"；它的临时目录{'没删掉' if is_still_there else '已整个删掉'}（删前留有 {file_count} 个文件、占盘 {allocated_bytes} 字节）"
                + line_break + rest)
    key = context["row_keys"][row.line_number]
    full_name = single_full_name(row.expected, output) or ""
    record_problems = []
    failed_to_write = write_record_quietly("mutation", key, dict(
        kind="mutation", judge_version=context["judging"], row_key=key, row_content_hash=row_content_hash(row),
        table_line=row.line_number, name=row.name, file=row.path, memory_max=context["memory_max"], timeout_seconds=context["timeout_seconds"],
        verdict=verdict, text=text, removal_files=removal[0], removal_allocated_bytes=removal[1], removal_apparent_bytes=removal[2],
        removal_left=int(removal[3]), exit_code=run.returncode, finished_utc=utc_now_text(), snapshot=WORKER_STATE["snapshot"],
        test_full_name=full_name, combo_key=combo_key(row, full_name, context["toolchain"]) if full_name else "", coverage="", **seconds))
    if failed_to_write:
        record_problems.append(failed_to_write)
    if verdict not in VERDICTS_THAT_DID_NOT_RUN and verdict not in ("squeezed", "gpumemory"):
        timing_path = os.path.join(context["timings_directory"], row_content_hash(row))
        try:
            write_atomically(timing_path, record_text(dict(row_content_hash=row_content_hash(row), seconds_running=seconds["seconds_running"],
                                                           seconds_queued=seconds["seconds_queued"], timed_out=int(verdict == "timeout"),
                                                           finished_utc=utc_now_text())))
        except OSError as error:
            record_problems.append(f"{timing_path}：{error}")
    return ((row.line_number, verdict, text, removal), dict(snapshot=WORKER_STATE["snapshot"], test_full_name=full_name)), record_problems


# ── 覆盖（judge_coverage 与它用的几个函数） ────────────────────────────────────

def coverage_build_settings(environment):
    """插桩编译的开关：(只加在那几次的环境, 接在 cargo 后面的参数)。环境里有 CARGO_ENCODED_RUSTFLAGS / RUSTFLAGS 就接在后面
    （它们顶掉配置文件里的 build.rustflags，接在后面才不丢），没有就经 --config build.rustflags（与配置文件里的合并）。"""
    encoded = environment.get("CARGO_ENCODED_RUSTFLAGS", "")
    if encoded:
        return {"CARGO_ENCODED_RUSTFLAGS": encoded + "\x1f" + COVERAGE_FLAG}, []
    flags = environment.get("RUSTFLAGS", "").strip()
    if flags:
        return {"RUSTFLAGS": flags + " " + COVERAGE_FLAG}, []
    return {}, ["--config", f'build.rustflags=["{COVERAGE_FLAG}"]']


def dep_info_paths(filenames):
    """一个编译单元的产物（compiler-artifact 的 filenames）旁边 rustc 写的 dep-info：libx-<哈希>.rlib → x-<哈希>.d，可执行文件 x-<哈希> → x-<哈希>.d。"""
    found = []
    for name in filenames:
        directory, base = os.path.split(name)
        stem, extension = os.path.splitext(base)
        if extension in (".rlib", ".rmeta", ".so", ".a", ".dylib", ".dll"):
            stem = stem[3:] if stem.startswith("lib") else stem
        elif extension not in ("", ".exe"):
            continue
        candidate = os.path.join(directory, stem + ".d")
        if os.path.isfile(candidate) and candidate not in found:
            found.append(candidate)
    return found


def parse_dep_info(path):
    """rustc 的 dep-info（Makefile 写法）第一条规则列的文件；`\\ ` 是路径里的空格。"""
    with open(path, encoding="utf-8", errors="surrogateescape") as handle:
        for line in handle:
            if not line.strip() or line.startswith("#"):
                continue
            _target, separator, rest = line.partition(": ")
            if not separator:
                continue
            return [part.replace("\\ ", " ") for part in re.split(r"(?<!\\) ", rest.strip()) if part]
    return []


def relative_inside(path, work):
    """dep-info 与 cargo 消息里的路径 → 相对源码副本根的路径；落在副本之外（注册表、OUT_DIR）交 None。"""
    full = os.path.normpath(path if os.path.isabs(path) else os.path.join(work, path))
    root = os.path.normpath(work)
    return os.path.relpath(full, root) if full.startswith(root + os.sep) else None


class CompiledFileIndexes:
    """给 mutation-prune.py 的 covered_containers 用的 files：只认编进去的运行期 .rs，按需切、在这个工作进程里缓存（副本不变）。"""

    def __init__(self, work, compiled, cache):
        self.work, self.compiled, self.cache = work, compiled, cache

    def __contains__(self, path):
        return self.compiled.get(path) == "runtime" and path.endswith(".rs")

    def __getitem__(self, path):
        if path not in self.cache:
            with open(os.path.join(self.work, path), encoding="utf-8", errors="replace") as handle:
                self.cache[path] = prune_library().FileIndex(path, handle.read())
        return self.cache[path]


def coverage_from_build(messages, work, snapshot_files):
    """--no-run 那一次的 compiler-artifact 消息 → (编进去的文件 {路径: runtime / compile-time}, 涉及的包目录, 本仓的可执行文件)。"""
    compiled, packages, executables = {}, set(), []
    for message in messages:
        if message.get("reason") != "compiler-artifact":
            continue
        package = relative_inside(os.path.dirname(message.get("manifest_path", "")), work)
        if package is None:
            continue
        packages.add(package)
        kinds = message.get("target", {}).get("kind", [])
        compile_time = any(kind in ("proc-macro", "custom-build") for kind in kinds)
        for dep_info in dep_info_paths(message.get("filenames", [])):
            for path in parse_dep_info(dep_info):
                relative = relative_inside(path, work)
                if relative is None or relative not in snapshot_files:
                    continue
                if compile_time:
                    compiled.setdefault(relative, "compile-time")
                else:
                    compiled[relative] = "runtime"
        executable = message.get("executable")
        if executable and "custom-build" not in kinds and executable not in executables:
            executables.append(executable)
    return compiled, sorted(packages), executables


def judge_coverage(job):
    """进程池里的一格活：一组「cargo 参数 + 点名用例」在没改坏的副本上记一份覆盖（文件头「判定库」覆盖记录 ①–④）。
    回 ({combo, coverage（覆盖号或 None）, snapshot, problem（没记成的那一句或 None）, line}, 写不进去的记录)。"""
    state, context = WORKER_STATE, WORKER_STATE["context"]
    result = dict(combo=job["combo"], coverage=None, snapshot=state["snapshot"], problem=None, line=job["line"])
    if break_is(context["environment"], "coveragenotrecorded"):
        return result, []
    identity = coverage_id(job["combo"], state["snapshot"])
    if state["store"].coverage(identity) is not None:
        result.update(coverage=identity, existing=True)
        return result, []
    overrides, configuration = coverage_build_settings(state["environment"])
    overrides["CARGO_TARGET_DIR"] = state["environment"]["CARGO_TARGET_DIR"] + COVERAGE_TARGET_SUFFIX
    temporary_directory = tempfile.mkdtemp(prefix=f"coverage-{job['line']}-", dir=state["per_mutation_temporary_root"])
    profiles = os.path.join(temporary_directory, "profiles")
    os.mkdir(profiles)
    try:
        result["problem"] = record_coverage(job, identity, overrides, configuration, temporary_directory, profiles)
        if result["problem"] is None:
            result["coverage"] = identity
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        result["problem"] = f"记覆盖时出错（{type(error).__name__}：{error}）"
    finally:
        measure_and_remove_temporary_directory(temporary_directory, context)
    return result, []


def record_coverage(job, identity, overrides, configuration, temporary_directory, profiles):
    """judge_coverage 的 ①–④：记成交 None，哪一步不成交那一句。"""
    state, context = WORKER_STATE, WORKER_STATE["context"]
    stamp = f"coverage-{job['line']}"
    # 带 -Cinstrument-coverage 的编译：cargo 没带 --target 时 RUSTFLAGS 也罩构建脚本与过程宏，它们在各自的源码目录（~/.cargo/registry/src/ 下依赖包的目录）里
    # 起，没设 LLVM_PROFILE_FILE 就在那里写 default_<哈希>_<进程号>.profraw；编译这一步同样把剖析文件指进这一格的 TMPDIR（不并进 profiles：那里只放测试那一趟的）
    build_profiles = os.path.join(temporary_directory, BUILD_PROFILES_DIRECTORY_NAME)
    os.mkdir(build_profiles)
    build_overrides = overrides if break_is(context["environment"], "buildprofilesunset") else dict(
        overrides, LLVM_PROFILE_FILE=os.path.join(build_profiles, "build-%p-%m.profraw"))
    build, _seconds = run_under_wrapper(["cargo", *configuration, "test", "--offline", *job["before"], "--no-run", "--message-format=json-render-diagnostics"],
                                        temporary_directory, stamp + "-build", build_overrides)
    outcome = wrapper_outcome(build.returncode, context["memory_max"], context["timeout_seconds"])
    if outcome is not None or build.returncode != 0:
        return f"插桩编译没成（{outcome[1] if outcome else f'退出码 {build.returncode}'}）：{output_tail(build.stderr, 3)}"
    messages = []
    for line in build.stdout.splitlines():
        if line.startswith("{"):
            with contextlib.suppress(ValueError):
                messages.append(json.loads(line))
    compiled, packages, executables = coverage_from_build(messages, state["work"], state["snapshot_files"])
    if not executables:
        return "插桩编译没交出一个本仓的可执行文件"
    run, _seconds = run_under_wrapper(["cargo", *configuration, "test", "--offline", *job["before"], "--", *job["switches"], "--exact", job["test"]],
                                      temporary_directory, stamp + "-run", dict(overrides, LLVM_PROFILE_FILE=os.path.join(profiles, "cov-%p-%m.profraw")))
    output = run.stdout + run.stderr
    outcome = wrapper_outcome(run.returncode, context["memory_max"], context["timeout_seconds"])
    results = results_by_test(job["test"], output)
    if outcome is not None or run.returncode != 0 or len(results) != 1 or any(result != "ok" for result in next(iter(results.values()))):
        return (f"插桩跑 {job['test']} 没恰好一个 ok（{outcome[1] if outcome else f'退出码 {run.returncode}'}；"
                f"对上 {len(results)} 个用例）：{output_tail(output, 3)}")
    raw_profiles = sorted(os.path.join(profiles, name) for name in os.listdir(profiles) if name.endswith(".profraw"))
    if not raw_profiles:
        return "插桩跑完一份 .profraw 都没留下（LLVM_PROFILE_FILE 没传到测试进程）"
    merged = os.path.join(temporary_directory, "merged.profdata")
    tools = context["llvm_tools"]
    merging = subprocess.run([os.path.join(tools, "llvm-profdata"), "merge", "-sparse", *raw_profiles, "-o", merged], capture_output=True, text=True, errors="replace")
    if merging.returncode != 0:
        return f"llvm-profdata merge 退 {merging.returncode}：{output_tail(merging.stderr, 3)}"
    objects = [executables[0]] + [word for executable in executables[1:] for word in ("--object", executable)]
    exporting = subprocess.run([os.path.join(tools, "llvm-cov"), "export", "-format=text", "-skip-expansions", "-instr-profile", merged, *objects],
                               capture_output=True, text=True, errors="replace")
    if exporting.returncode != 0:
        return f"llvm-cov export 退 {exporting.returncode}：{output_tail(exporting.stderr, 3)}"
    export = json.loads(exporting.stdout)
    tree = types.SimpleNamespace(root=state["work"], files=CompiledFileIndexes(state["work"], compiled, state["file_indexes"]))
    executed = sorted([path, container] for path, container in prune_library().covered_containers(export, tree))
    for path, kind in compiled.items():
        if kind == "runtime" and path.endswith(".rs"):
            data = read_bytes(os.path.join(state["work"], path))
            digest = hashlib.sha256(data).hexdigest()
            if digest != state["snapshot_files"].get(path):
                return f"副本里的 {path} 与这一份快照对不上（记覆盖时被改了）"
            state["store"].put_blob(digest, data)
    state["store"].put_coverage(identity, dict(
        combo=job["combo"], snapshot=state["snapshot"], arguments=" ".join(job["before"]), switches=" ".join(job["switches"]), test=job["test"],
        compiled=compiled, packages=packages, executed=executed, executables=len(executables), created_utc=utc_now_text()))
    return None


# ── 整道 ──────────────────────────────────────────────────────────────────────

# gate-overlap:copy-kept scripts/mutation/mutate.sh 那边的 worker_count_and_head_line 照这里的「各项取最小、至少 1、核数一半至多 16」写了一份 bash：两边的项不同（这里 GATE_MUTATION_WORKERS 顶掉核数那一项、第二项是要现跑的条数、第三项是外层内存上限 ÷ 每条上限（读不到就没有这一项）；那边没有第三项，MUTATE_JOBS 只能压小、第二项是表里的条数、另算每个 cargo 的编译并行度），派活一边是本进程的进程池、一边是 bash 子 shell 按目录认领，共用的只剩一个取最小值，抽成共用要让 mutate.sh 每趟多起一个 python 只为算一个数、两边头一行的字也不同
def bytes_of_size(text):
    """systemd 写法的大小（正整数加 K / M / G / T，1024 进制）→ 字节；别的写法交 None。"""
    matched = SIZE_TEXT.match(text.strip())
    return int(matched.group(1)) * SIZE_UNIT_BYTES[matched.group(2)] if matched else None


def worker_count_for(row_count, environment, memory_max):
    """开几个工作进程，连同头一行要报的话。三项取最小、至少 1：核数的一半（至多 16，GATE_MUTATION_WORKERS 顶掉它）、这一趟要现跑的条数、
    外层内存上限 ÷ 每条变异的上限（向下取整；外层上限读不到时没有这一项）。
    一条变异的挂钟 = 重编译那几个 crate + 跑点名的那个测试，测试那一段是单线程的，所以开得比核数 ÷ 4 多，再用 GATE_MUTATION_CARGO_JOBS
    把每个 cargo 的编译并行度收住。
    外层内存上限读环境变量 RUN_WITH_MEMORY_CAP_OUTER_CAP（run-with-memory-cap.sh 套在外层时给命令设的，例 16G）：套在外层里的里层包装不排队
    （那份文件头「套在包装里」），几条里层同时跑、合起来超过外层上限时只有 slice 的总上限兜着，所以工作进程数要自己按外层收。
    没设、写法不认（不是正整数加 K / M / G / T）就是没套在外层包装里：没有这一项，每条自己经包装排队、放不下的在它那里等，头一行写明没读到。
    弄坏开关 outermemoryignored：当读不到外层上限。"""
    bounds = []
    requested = environment.get("GATE_MUTATION_WORKERS", "").strip()
    if requested:
        bounds.append(("GATE_MUTATION_WORKERS", int(requested), f"GATE_MUTATION_WORKERS 给 {int(requested)} 个"))
    else:
        processor_count = os.cpu_count() or 4
        by_processor = max(1, min(16, processor_count // 2))
        bounds.append(("核数", by_processor, f"核数 {processor_count} 的一半、至多 16，给 {by_processor} 个"))
    bounds.append(("这一趟要现跑的条数", row_count, f"这一趟要现跑 {row_count} 条（没有作数的抓到记录的）"))
    outer_text = environment.get(OUTER_MEMORY_CAP_VARIABLE, "").strip()
    outer_bytes = None if break_is(environment, "outermemoryignored") else bytes_of_size(outer_text)
    per_mutation_bytes = bytes_of_size(memory_max)
    by_memory = outer_bytes // per_mutation_bytes if outer_bytes is not None and per_mutation_bytes is not None else None
    if by_memory is not None:
        bounds.append(("外层内存上限", by_memory, f"外层内存上限 {outer_text} ÷ 每条变异 {memory_max}，向下取整给 {by_memory} 个"))
    smallest = min(count for _name, count, _text in bounds)
    binding = "、".join(f"「{name}」" for name, count, _text in bounds if count == smallest)
    floor_note = "（那一项不足 1，按 1 个开）" if smallest < 1 else ""
    worker_count = max(1, smallest)
    head_line = (f"  … 这一轮开 {worker_count} 个工作进程（各项取最小、至少 1）：{'；'.join(text for _name, _count, text in bounds)}。"
                 f"卡在{binding}这一项{floor_note}；")
    if by_memory is not None:
        head_line += "套在外层包装里的里层不排队，工作进程数按外层内存上限收（slice 的总上限兜底）"
    else:
        head_line += (f"内存不按进程数收：每条经 run-with-memory-cap.sh 排队，放不下的等（slice 的总上限兜底）；"
                      f"外层内存上限没读到（{OUTER_MEMORY_CAP_VARIABLE} {'没设' if not outer_text else f'写成「{outer_text}」，不是正整数加 K / M / G / T'}，按没套在外层包装里算）")
    return worker_count, head_line


def run_pool(context, worker_count, batches, ordered_rows, baseline_ok_lines, stop_after, coverage_planner):
    """一个进程池里先派基线、再派变异、最后派覆盖：一批基线判完，它 ok 的行进变异的队；已有基线 ok 的行一开始就在队里；
    一条变异判成抓到、它那一组还要记覆盖的（coverage_planner.job_after），那一组的覆盖进队；开跑时就要记的（coverage_planner.initial_jobs）一开始就在队里。
    同时在跑的至多 worker_count 格，空出一格补一格（谁先跑完谁再领），基线优先、变异按 ordered_rows 的次序、覆盖最后。
    回 dict：mutation（变异的判定）、mutation_info（{行号: {snapshot, test_full_name}}）、baseline（{行号: (问题或 None, 尾巴)}）、coverage（覆盖那几格的结果）、
    submitted、received、stopped、left_behind、record_problems。"""
    sequence_counter = multiprocessing.Value("i", 0)
    # 被停的信号也要删掉 shard_root（$TMPDIR 下的 singlefs-mutation-replay-*）：先接住再建，建完之后收到的都走 finally；
    # 只在主线程接得了。弄坏开关 shardrootkeptonsignal 下不接（改前的样子：收到就停，shard_root 留在 $TMPDIR）
    previous_handlers = {}
    if threading.current_thread() is threading.main_thread() and not break_is(context["environment"], "shardrootkeptonsignal"):
        for signal_number in STOP_SIGNALS:
            previous_handlers[signal_number] = signal.signal(signal_number, raise_stop_requested)
    shard_root = None
    try:
        shard_root = tempfile.mkdtemp(prefix="singlefs-mutation-replay-")
        per_mutation_temporary_root = os.path.join(shard_root, PER_MUTATION_TEMPORARY_DIRECTORY_NAME)
        os.mkdir(per_mutation_temporary_root)
        os.mkdir(os.path.join(shard_root, START_STAMP_DIRECTORY_NAME))
        order_index = {row.line_number: index for index, row in enumerate(ordered_rows)}
        rows_by_line = {row.line_number: row for row in ordered_rows}
        ready, serial = [], 0
        for index, batch in enumerate(batches):
            heapq.heappush(ready, (0, index, serial, "baseline", batch))
            serial += 1
        for row in ordered_rows:
            if row.line_number in baseline_ok_lines:
                heapq.heappush(ready, (1, order_index[row.line_number], serial, "mutation", row))
                serial += 1
        for job in coverage_planner.initial_jobs():
            heapq.heappush(ready, (2, serial, serial, "coverage", job))
            serial += 1
        outcome = dict(mutation=[], mutation_info={}, baseline={}, coverage=[], submitted=0, received=0, stopped=False, left_behind=[], record_problems=[])
        in_flight = {}
        jobs = {"baseline": judge_batch, "mutation": judge_row, "coverage": judge_coverage}
        with concurrent.futures.ProcessPoolExecutor(max_workers=worker_count, initializer=init_worker,
                                                    initargs=(sequence_counter, shard_root, context)) as pool, own_workers_stopped_on_stop(pool, shard_root):
            while True:
                while ready and len(in_flight) < worker_count and not outcome["stopped"]:
                    _phase, _index, _serial, kind, item = heapq.heappop(ready)
                    in_flight[pool.submit(jobs[kind], item)] = kind
                    outcome["submitted"] += 1
                if not in_flight:
                    break
                done, _pending = concurrent.futures.wait(list(in_flight), return_when=concurrent.futures.FIRST_COMPLETED)
                for future in done:
                    kind = in_flight.pop(future)
                    result, record_problems = future.result()   # 工作进程里抛出来的异常在这里原样炸出去：并行不许把失败吃掉
                    outcome["received"] += 1
                    outcome["record_problems"] += record_problems
                    if kind == "baseline":
                        for line_number, problem, tail in result:
                            outcome["baseline"][line_number] = (problem, tail)
                            if problem is None and line_number in order_index:
                                heapq.heappush(ready, (1, order_index[line_number], serial, "mutation", rows_by_line[line_number]))
                                serial += 1
                    elif kind == "mutation":
                        judgement, info = result
                        outcome["mutation"].append(judgement)
                        outcome["mutation_info"][judgement[0]] = info
                        job = coverage_planner.job_after(rows_by_line[judgement[0]], judgement, info)
                        if job is not None:
                            heapq.heappush(ready, (2, serial, serial, "coverage", job))
                            serial += 1
                        if stop_after is not None and len(outcome["mutation"]) >= stop_after:
                            outcome["stopped"] = True
                    else:
                        outcome["coverage"].append(result)
                    note(f"  … 已判 {outcome['received']} 格、派出 {outcome['submitted']} 格、还排着 {len(ready)} 格（基线的批、变异与覆盖合计；基线不绿的行不派变异）")
        outcome["left_behind"] = sorted(os.listdir(per_mutation_temporary_root))
        outcome["left_behind_contents"] = {name: (sorted(os.listdir(os.path.join(per_mutation_temporary_root, name)))
                                                  if os.path.isdir(os.path.join(per_mutation_temporary_root, name)) else ["（不是目录）"])
                                           for name in outcome["left_behind"]}
        outcome["per_mutation_temporary_root"] = per_mutation_temporary_root
        return outcome
    finally:
        if shard_root is not None:
            shutil.rmtree(shard_root, ignore_errors=True)
        for signal_number, handler in previous_handlers.items():
            signal.signal(signal_number, handler)


def read_selection(path, rows):
    """GATE_MUTATION_ROW_SELECTION 那份文件：一行一个变异表行号（# 起头的行与空行不看）。回 (选中的行号集合, [问题])。"""
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except (OSError, UnicodeDecodeError) as error:
        return set(), [f"读不了 {path}：{error}"]
    selected, problems = set(), []
    row_lines = {row.line_number for row in rows}
    for index, line in enumerate(lines, 1):
        text = line.strip()
        if not text or text.startswith("#"):
            continue
        if not text.isdigit():
            problems.append(f"{path} 第 {index} 行「{text}」不是一个行号")
        elif int(text) not in row_lines:
            problems.append(f"{path} 第 {index} 行：{TABLE} 第 {text} 行不是一条成形的变异")
        else:
            selected.add(int(text))
    if not selected and not problems:
        problems.append(f"{path} 里一个行号都没有")
    return selected, problems


def print_rejection(headline, details, howto_lines):
    say(f"  ✗ {headline}")
    for detail in details:
        say(f"      {detail}")
    for index, howto_line in enumerate(howto_lines):
        say(("     → 怎么办：" if index == 0 else "       ") + howto_line)


def prechecks(root, rows, malformed, environment):
    """派活之前先判、一个工作进程都不起的那几样（row_problems，与门禁 code-source-discipline 经 static-check 判的是同一份）。回 (拷贝范围, 判红了没有)。"""
    admission = admission_module()
    try:
        copied = copied_paths()
    except admission.RegistrationError as error:
        copied, reason = [], str(error)
    else:
        reason = "那一行一条路径都没登记"
    if not copied:
        print_rejection(f"读不到 .claude/gate.d/stage-inputs.tsv 里 {STAGE} 那一行登记的路径（{reason}）：判不出每片的源码副本该拷哪些", [],
                        [f"单独执行 python3 scripts/gate/admission.py paths \"{STAGE_REPOSITORY}\" {STAGE} 看它报什么；那一行不在就照别的行给本阶段登记它读的路径。"])
        return None, True
    # 锚点只在真仓上核：文件落在拷贝范围之外时，工作进程在自己那份副本里打不开它，所以拷贝范围也在派活之前判。有一样红就一个工作进程都不起。
    problems = row_problems(root, rows, malformed, copied)
    not_started = "这一轮一个工作进程都没起；门禁 code-source-discipline 静态判同样几样（不跑变异），改完先跑它。"
    if problems["MALFORMED"]:
        print_rejection(f"{TABLE} 有几行不成形（六段制表符分隔：{'、'.join(FIELD_NAMES)}；替换文可以空，那是删掉原文，其余五段不许空）：",
                        [f"{TABLE}:{line_number} {name}：{why}" for line_number, name, why in problems["MALFORMED"]],
                        ["补上空着的那一段，制表符删补到恰好六段；原文 / 替换文里的换行写成 \\n。要删掉原文就让替换文空着（两个制表符挨着）。", not_started])
    if problems["ANCHOR"]:
        print_rejection("变异表的锚点腐化：", [f"{TABLE}:{line_number} {name}：{why}" for line_number, name, why in problems["ANCHOR"]],
                        ["改代码时把变异表里的原文一起改到今天的写法（锚点腐化的那条变异等于没跑过，mutation-sampling.md 第七类）。", not_started])
    if problems["NAME"]:
        print_rejection("变异表有几行的「必须红的测试名」不是一个用例名（带了正则符号或空白），按字面在测试输出里永远找不到：",
                        [f"{TABLE}:{line_number} {name}：「{expected}」" for line_number, name, expected in problems["NAME"]],
                        ["那一列写成用例名本身（例 new_pool_file_creation_self_release_is_one_slot，或带模块路径 tests::new_pool_file_creation_self_release_is_one_slot），",
                         "去掉 $、^ 这类符号；只写名字时本阶段已经按「在输出里只许对上一个用例」判，不用再靠 $ 锚定。", not_started])
    if problems["ROW"]:
        print_rejection("变异表有几行的文件不在每片拷的范围里（锚点在真仓上核得过，工作进程在自己的源码副本里却打不开它）：",
                        [f"{TABLE}:{line_number} {name}：{path}" for line_number, name, path in problems["ROW"]],
                        [f"把那个路径（或它所在的目录）登记进 .claude/gate.d/stage-inputs.tsv 本阶段那一行（现在是 {'、'.join(copied)}），"
                         "或者把这条变异改到拷贝范围之内的文件上。", not_started])
    if problems["STRAY"]:
        print_rejection("变异表有几行的原文或替换文里有 \\n 以外的反斜杠转义（这里只把 \\n 还原成换行，别的反斜杠按字面写进源码，那条变异编不过、等于没跑）：",
                        [f"{TABLE}:{line_number} {name}：{segment}里的 {escape}" for line_number, name, segment, escape in problems["STRAY"]],
                        ["去掉那个反斜杠（例 \\&report.outcome 写成 &report.outcome），改完单跑那一行证明点名的测试红。", not_started])
    if any(problems.values()):
        return None, True
    return copied, False


def run_stage(root, environment=None):
    """门禁 checker-tier-crates-mutation-replay 的整道：判定打 stdout（按表的行号排序，与进程数、复用了多少都无关），进度与复用几条打 stderr。回退出码。"""
    environment = dict(os.environ if environment is None else environment)
    admission = admission_module()
    table_path = os.path.join(root, TABLE)
    if not os.path.isfile(table_path):
        print_rejection(f"没有 {TABLE}", [], ["建一张六段制表符分隔的表（变异名、文件、原文、替换文、cargo test 参数、必须红的测试名），每一处三方打中之后的改法留一条。"])
        return 1
    try:
        rows, malformed, memory_max_from_table, timeout_from_table = read_table(table_path, environment)
    except (OSError, UnicodeDecodeError) as error:
        print_rejection(f"读不了 {TABLE}（{type(error).__name__}：{error}）", [], ["转成 UTF-8 文本再跑；读不了的表一条都判不了。"])
        return 1
    # 不成形的行（read_table 判的）在 prechecks 里与别的几样一起报：整张表的不成形行不因选行而放过
    if not rows and not malformed:
        print_rejection(f"{TABLE} 里一条成形的变异都没有", [], ["至少一条：三方打中之后的每一处改法，留一条「改回去它就红」的变异。"])
        return 1
    memory_max, timeout_seconds, limit_problems = limits_of(memory_max_from_table, timeout_from_table, environment)
    for headline, howto_line in limit_problems:
        print_rejection(headline, [], [howto_line])
    if limit_problems:
        return 1
    numeric_settings = {}
    for variable, default in (("GATE_MUTATION_LONG_ROW_SECONDS", DEFAULT_LONG_ROW_SECONDS), ("GATE_MUTATION_STOP_AFTER_ROWS", "")):
        text = environment.get(variable, "").strip() or default
        if text and not re.fullmatch(r"[1-9][0-9]*", text):
            print_rejection(f"{variable} 写成了「{text}」，不是正整数", [], [f"写成正整数（{variable} 的意思见 scripts/mutation/crates-mutation-rows.py 文件头），或者不设它。"])
            return 1
        numeric_settings[variable] = int(text) if text else None
    selection_path = environment.get("GATE_MUTATION_ROW_SELECTION", "").strip()
    if selection_path and rows:
        selected, selection_problems = read_selection(selection_path, rows)
        if selection_problems:
            print_rejection(f"GATE_MUTATION_ROW_SELECTION 指的那份选行文件不对，一行都没判：", selection_problems,
                            [f"一行写一个 {TABLE} 里成形的变异所在的行号（# 起头的行与空行不看）；行号照被判的那一份表现数。"])
            return 1
        rows = [row for row in rows if row.line_number in selected]
    copied, refused = prechecks(root, rows, malformed, environment)
    if refused:
        return 1
    memory_cap_runner = environment.get("GATE_MUTATION_MEMORY_CAP_RUNNER", "").strip() or DEFAULT_MEMORY_CAP_RUNNER
    try:
        judging, _judging_items = judge_version(root, environment, memory_cap_runner)
        toolchain = toolchain_text(root, environment)
        common_directory = records_home(root, environment)
    except (admission.RegistrationError, admission.InputManifestError, OSError) as error:
        print_rejection(f"算不出判法版本或工具链（{error}），判定库记录没处对：一条都没跑", [],
                        ["被判的要是 git 工作树、cargo -V 与 rustc -V 跑得通、.claude/gate.d/stage-inputs.tsv 里有本阶段那一行；",
                         f"单跑 python3 scripts/mutation/crates-mutation-rows.py judge-version {root} 看它报什么。"])
        return 1
    records_root = os.path.join(common_directory, RECORDS_DIRECTORY_NAME)
    records_directory = os.path.join(records_root, judging)
    timings_directory = os.path.join(common_directory, TIMINGS_DIRECTORY_NAME)
    os.makedirs(timings_directory, exist_ok=True)
    pruned = prune_base_directories(records_root, judging)
    store = Store(records_directory)
    store.ensure()
    current = current_state(root, copied)
    store.put_snapshot(current["id"], current["files"])
    refusal = reuse_refusal(environment)
    skip_baseline = break_is(environment, "nobaseline")
    row_keys = {row.line_number: row_key(row, memory_max, timeout_seconds, toolchain) for row in rows}
    store_has_records = store.has_new_format_records()
    records, decisions, comparisons = {}, {}, {}
    reused, baseline_ok_lines, reused_baseline_count = {}, set(), 0
    for row in rows:
        key = row_keys[row.line_number]
        record = read_record(os.path.join(records_directory, f"mutation.{key}"))
        records[row.line_number] = record
        decision = reuse_decision(row, key, record, store, current, comparisons, environment, refusal, store_has_records)
        if decision.reuse:
            try:
                removal = (int(record.get("removal_files", "")), int(record.get("removal_allocated_bytes", "")),
                           int(record.get("removal_apparent_bytes", "")), record.get("removal_left") == "1")
            except ValueError:
                decision = Decision(False, "no-coverage", "记录里删临时目录的那几个数读不出", None)
        decisions[row.line_number] = decision
        if decision.reuse:
            reused[row.line_number] = (row.line_number, "caught", caught_text(row, "（按覆盖复用）"), removal)
            baseline_ok_lines.add(row.line_number)
        elif not refusal:
            baseline = read_record(os.path.join(records_directory, f"baseline.{key}"))
            if (baseline is not None and baseline.get("kind") == "baseline" and baseline.get("row_key") == key
                    and baseline.get("verdict") == REUSABLE_VERDICT["baseline"] and baseline.get("snapshot") == current["id"]):
                baseline_ok_lines.add(row.line_number)
                reused_baseline_count += 1
    candidates = [row for row in rows if row.line_number not in reused]
    if skip_baseline:
        baseline_ok_lines |= {row.line_number for row in candidates}
    needs_baseline = [row for row in candidates if row.line_number not in baseline_ok_lines]
    batches = list(baseline_batches(needs_baseline).items())
    ordered_candidates, moved_count = dispatch_order(candidates, timings_directory, numeric_settings["GATE_MUTATION_LONG_ROW_SECONDS"], environment)
    planner = CoveragePlanner(rows, row_keys, records, decisions, toolchain, current["id"], store)
    initial_coverage_jobs = planner.initial_jobs()
    # stderr 头一行：复用几条、现跑几条、前移几条（逐栏的数在 stdout 的「复用：」那一行）
    note(f"  … 判定库记录：复用抓到 {len(reused)} 条、复用基线 ok {reused_baseline_count} 行；现跑基线 {len(batches)} 批 {len(needs_baseline)} 行、"
         f"现跑变异至多 {len(candidates)} 条、补记覆盖 {len(initial_coverage_jobs)} 组；前移 {moved_count} 条（上一次用时 ≥ {numeric_settings['GATE_MUTATION_LONG_ROW_SECONDS']} 秒"
         f"或上一次超时的先派，按用时从长到短）；判法版本 {judging[:16]}…，记录在 {records_directory}"
         + (f"；不复用：{refusal}" if refusal else "") + (f"；删了更早的判法版本目录 {len(pruned)} 个" if pruned else "")
         + ("；基线没跑：GATE_MUTATION_BREAK=nobaseline（只给证红用）" if skip_baseline else ""))
    outcome = dict(mutation=[], mutation_info={}, baseline={}, coverage=[], submitted=0, received=0, stopped=False, left_behind=[], record_problems=[])
    worker_count, cargo_jobs = 0, "-"
    if candidates or initial_coverage_jobs:
        memory_cap_check = subprocess.run(["bash", memory_cap_runner, "--check", memory_max], capture_output=True, text=True, errors="replace",
                                          env=environment)
        if memory_cap_check.returncode != 0:
            print_rejection(f"带内存上限（每条 {memory_max}）的 systemd scope 起不来（{memory_cap_runner} --check 退 {memory_cap_check.returncode}），一条变异都没跑：",
                            (memory_cap_check.stdout + memory_cap_check.stderr).splitlines(),
                            ["照上面那几行修用户级 systemd / D-Bus（systemctl --user status）再跑这一道；不许拿掉上限去跑，无界分配的变异会把整机拖进 OOM。"])
            return 1
        tools, tools_text, tools_problem = llvm_tools(root, environment)
        if tools_problem is not None:
            print_rejection(f"记覆盖要的 llvm 工具不对（{tools_problem[0]}），一条变异都没跑：", [],
                            [tools_problem[1], "不退回不记覆盖去跑：没有覆盖记录，下一趟这几行全都要现跑。"])
            return 1
        note(f"  … 记覆盖用的 llvm 工具：{tools_text}")
        worker_count, worker_count_head_line = worker_count_for(len(candidates) + len(initial_coverage_jobs), environment, memory_max)
        note(worker_count_head_line)
        # 每个 cargo 的编译并行度：进程数 × 它 ≈ 核数，免得 N 个 cargo 各自按核数开 rustc、互相抢
        cargo_jobs = environment.get("GATE_MUTATION_CARGO_JOBS", "").strip() or str(max(1, (os.cpu_count() or 4) // worker_count))
        environment["CARGO_BUILD_JOBS"] = cargo_jobs
        # 变异复跑起的 cargo 写自己的崩溃判定库（文件头「变异复跑的判定库」）：不写进真跑的长期库
        crash_library_home, crash_library_cleared = mutation_crash_library_environment(environment)
        if crash_library_home is not None:
            environment[CRASH_LIBRARY_HOME_VARIABLE] = crash_library_home
            for name in crash_library_cleared:
                environment.pop(name, None)
        # 设了 GATE_MUTATION_CRASH_LIBRARY_HOME 的不打路径（双机分片的驱动给第二台设的是那棵树的目录，输出里不带第二台的目录）
        crash_library_home_text = (f"{MUTATION_CRASH_LIBRARY_HOME_VARIABLE} 设的那一份" if environment.get(MUTATION_CRASH_LIBRARY_HOME_VARIABLE, "").strip()
                                   else str(environment.get(CRASH_LIBRARY_HOME_VARIABLE) or "没设（用例照 " + DEFAULT_CRASH_LIBRARY_HOME + "）"))
        note(f"  … 起 cargo 时的崩溃判定库家目录：{crash_library_home_text}"
             + (f"，清掉调用方点名的 {'、'.join(crash_library_cleared)}" if crash_library_home is not None else "（弄坏开关 sharedcrashlibrary：照调用方的环境）"))
        shared_target = default_shared_target_directory(environment)
        # 编译目录数按这一趟的工作进程数复用：片号更大的旧 -w<片号>（连同 -coverage）开跑时回收，有人在用的留着（reclaim_extra_target_directories）
        for line in target_reclaim_lines(shared_target, worker_count, reclaim_extra_target_directories(shared_target, worker_count, environment)):
            note(line)
        context = dict(root=root, copied=copied, environment=environment, memory_max=memory_max, timeout_seconds=timeout_seconds,
                       memory_cap_runner=memory_cap_runner, judging=judging, toolchain=toolchain, llvm_tools=tools,
                       records_directory=records_directory, timings_directory=timings_directory,
                       row_keys=row_keys, shared_target_directory=shared_target,
                       worker_count=worker_count, stage_process=os.getpid())
        try:
            outcome = run_pool(context, worker_count, batches, ordered_candidates, baseline_ok_lines, numeric_settings["GATE_MUTATION_STOP_AFTER_ROWS"], planner)
        except concurrent.futures.process.BrokenProcessPool:
            # 一个工作进程被硬杀（OOM、段错误）时当场抛这个，而不是安静地少产出几项
            print_rejection("有工作进程中途死了，这一轮的变异没跑全", [],
                            ["多半是内存不够被 OOM 杀的——GATE_MUTATION_WORKERS 调小再跑一遍（判完的那几条有判定库记录，下一趟只跑剩下的）；",
                             "单跑 GATE_MUTATION_WORKERS=1 能跑完就是并发度的问题，还死就去看那一条变异本身。"])
            return 1
    for problem in outcome["record_problems"]:
        note(f"  ! 判定库记录或用时表没写成（下一趟这一条照跑，不会复用）：{problem}")
    # 派出去多少格就要收回来多少格：对不上整道红，不许少跑一条还报绿（command-safety.md）
    if outcome["submitted"] != outcome["received"]:
        print_rejection(f"派出去 {outcome['submitted']} 格（基线的批、变异与覆盖），只收回 {outcome['received']} 格", [],
                        ["这是分片并发自己的完整性闸红了，不是变异的问题；把 GATE_MUTATION_WORKERS=1 再跑一遍看串行下是不是全的，",
                         "是就去查领活与收束那一段（scripts/mutation/crates-mutation-rows.py 的 run_pool）。"])
        return 1
    if outcome["stopped"]:
        print_rejection(f"只供测试的开关 GATE_MUTATION_STOP_AFTER_ROWS={numeric_settings['GATE_MUTATION_STOP_AFTER_ROWS']}："
                        f"现跑的变异判完 {len(outcome['mutation'])} 条就没再派活，这一轮没跑全", [],
                        ["这个开关只给自证造「跑到一半被杀」用，门禁里别设它；判完的那几条已经写了判定库记录，去掉它再跑一趟只跑剩下的。"])
        return 1
    attached = planner.attach(outcome, records_directory)
    garbage = collect_garbage(store)
    rows_by_line = {row.line_number: row for row in rows}
    fresh = []
    for line_number, verdict, text, removal in outcome["mutation"]:
        if verdict == "caught":
            text = caught_text(rows_by_line[line_number], f"（现判：{COLUMN_TITLES[decisions[line_number].column]}）")
        fresh.append((line_number, verdict, text, removal))
    judgements = list(reused.values()) + fresh
    baseline_problems = {}
    for row in candidates:
        problem, tail = outcome["baseline"].get(row.line_number, (None, ""))
        if problem is not None:
            baseline_problems[row.line_number] = problem
            judgements.append((row.line_number, "baseline",
                               f"{TABLE}:{row.line_number} {row.name}：基线不绿——点名的测试 {row.expected} 在没改坏的副本上{problem}\n{tail}", None))
    judged_lines = {judgement[0] for judgement in judgements}
    missing = [row for row in rows if row.line_number not in judged_lines]
    if missing:
        print_rejection(f"有 {len(missing)} 行一个判定都没有（没有作数的记录、基线也没判出它来）", [f"{TABLE}:{row.line_number} {row.name}" for row in missing],
                        ["这是判定库记录与派活自己的完整性闸红了，不是变异的问题；GATE_MUTATION_START_OVER=1 整道重跑一遍，还缺就去查 run_stage 与 run_pool 那一段。"])
        return 1
    tally = verdict_library.Tally()
    for row in rows:
        tally.count(decisions[row.line_number].reuse, decisions[row.line_number].column)
    coverage_failures = [result for result in outcome["coverage"] if result["problem"] is not None]
    for result in coverage_failures:
        note(f"  ! 覆盖没记成（组首行 {TABLE}:{result['line']}；下一趟这一组的行照「没有覆盖记录」现跑）：{result['problem']}")
    coverage_recorded = sum(1 for result in outcome["coverage"] if result["coverage"] and not result.get("existing"))
    coverage_existing = sum(1 for result in outcome["coverage"] if result.get("existing"))
    note(f"  … 覆盖：这一趟记了 {coverage_recorded} 组、那份快照上已有沿用 {coverage_existing} 组、没记成 {len(coverage_failures)} 组，"
         f"{attached} 行改指到这一趟的覆盖记录；收尾删了没人引的覆盖记录 {garbage[0]} 份、快照 {garbage[1]} 份、源码副本 {garbage[2]} 份")
    return report(rows, judgements, outcome, memory_max, timeout_seconds, selection_path, skip_baseline, dict(
        root=root, worker_count=worker_count, cargo_jobs=cargo_jobs, reused=tally.reused, reused_baseline=reused_baseline_count,
        reused_same_snapshot=sum(1 for line_number in reused if decisions[line_number].how == "same-snapshot"),
        fresh_columns=tally.reasons, fresh_rows=tally.judged, coverage_recorded=coverage_recorded, coverage_existing=coverage_existing,
        coverage_failed=len(coverage_failures),
        fresh_mutations=len(outcome["mutation"]), fresh_batches=len(batches), records_directory=records_directory,
        not_run_record=environment.get("GATE_MUTATION_NOT_RUN_RECORD", "").strip()))


class CoveragePlanner:
    """这一趟要记哪几组覆盖（文件头「判定库」覆盖记录）：判成抓到的一行，它那一组这一趟还没派过就派一格；
    一字未动复用来的、还没有它那份快照上的覆盖的，开跑时就派。跑完把这一趟判成抓到、所判快照与覆盖那一份相同的同组各行改指到它（attach）。"""

    def __init__(self, rows, row_keys, records, decisions, toolchain, current_snapshot, store):
        self.rows, self.row_keys, self.records, self.decisions = rows, row_keys, records, decisions
        self.toolchain, self.current_snapshot, self.store = toolchain, current_snapshot, store
        self.scheduled = set()
        self.initial = []
        for row in rows:
            decision, record = decisions[row.line_number], records[row.line_number]
            if not (decision.reuse and decision.how == "same-snapshot"):
                continue
            coverage = store.coverage(record.get("coverage", ""))
            if coverage is not None and coverage.get("snapshot") == current_snapshot:
                continue
            job = self.job(row, record.get("test_full_name", ""))
            if job is not None:
                self.initial.append(job)

    def job(self, row, test_full_name):
        if not test_full_name:
            return None
        combo = combo_key(row, test_full_name, self.toolchain)
        if combo in self.scheduled:
            return None
        self.scheduled.add(combo)
        before, switches = batch_key(row.arguments)
        return dict(combo=combo, before=list(before), switches=list(switches), test=test_full_name, line=row.line_number)

    def initial_jobs(self):
        return list(self.initial)

    def job_after(self, row, judgement, info):
        return self.job(row, info.get("test_full_name", "")) if judgement[1] == "caught" else None

    def attach(self, outcome, records_directory):
        """回改指了几行。"""
        results = {result["combo"]: result for result in outcome["coverage"] if result["coverage"]}
        fresh_info = outcome.get("mutation_info", {})
        fresh_verdicts = {judgement[0]: judgement[1] for judgement in outcome["mutation"]}
        attached = 0
        for row in self.rows:
            line_number = row.line_number
            if line_number in fresh_verdicts:
                if fresh_verdicts[line_number] != "caught":
                    continue
                full_name, validated = fresh_info[line_number]["test_full_name"], fresh_info[line_number]["snapshot"]
            elif self.decisions[line_number].reuse:
                full_name, validated = self.records[line_number].get("test_full_name", ""), self.current_snapshot
            else:
                continue
            if not full_name:
                continue
            combo = combo_key(row, full_name, self.toolchain)
            result = results.get(combo)
            if result is None or result["snapshot"] != validated:
                continue
            path = os.path.join(records_directory, f"mutation.{self.row_keys[line_number]}")
            record = read_record(path)
            if record is None or record.get("verdict") != REUSABLE_VERDICT["mutation"]:
                continue
            if record.get("coverage") == result["coverage"] and record.get("snapshot") == validated:
                continue
            record.update(coverage=result["coverage"], snapshot=validated, combo_key=combo)
            if line_number not in fresh_verdicts:
                record["rebased_utc"] = utc_now_text()
            with contextlib.suppress(OSError):
                write_atomically(path, record_text(record))
                attached += 1
        return attached


def llvm_tools(root, environment):
    """记覆盖用的 llvm-profdata、llvm-cov：(目录, 「目录与版本」那一句, None)，不对交 (None, None, (卡在哪, 下一步))（文件头「判定库」llvm 工具）。"""
    def run(command):
        try:
            return subprocess.run(command, capture_output=True, text=True, errors="replace", cwd=root, env=environment, stdin=subprocess.DEVNULL)
        except OSError:
            return None
    rustc = run(["rustc", "-vV"])
    if rustc is None or rustc.returncode != 0:
        return None, None, ("rustc -vV 跑不起来", "装好 Rust 工具链（bash .claude/scripts/env.sh 会报缺什么）")
    fields = dict(line.split(": ", 1) for line in rustc.stdout.splitlines() if ": " in line)
    wanted, host = fields.get("LLVM version", ""), fields.get("host", "")
    directory = environment.get(LLVM_TOOLS_VARIABLE, "").strip()
    if not directory:
        sysroot = run(["rustc", "--print", "sysroot"])
        if sysroot is None or sysroot.returncode != 0 or not host:
            return None, None, ("rustc --print sysroot 或 rustc -vV 的 host 取不到", "装好 Rust 工具链再跑")
        directory = os.path.join(sysroot.stdout.strip(), "lib", "rustlib", host, "bin")
    versions = []
    for tool in LLVM_TOOLS:
        path = os.path.join(directory, tool)
        if not os.access(path, os.X_OK):
            return None, None, (f"{path} 不在或不能执行", f"装 rustup 组件：rustup component add llvm-tools（与 rustc 同一份 LLVM）；装在别处就设 {LLVM_TOOLS_VARIABLE}=<目录>")
        answered = run([path, "--version"])
        found = re.search(r"LLVM version (\S+)", answered.stdout if answered is not None else "")
        version = found.group(1) if found else ""
        if not break_is(environment, "llvmversionunchecked") and (not wanted or not (version == wanted or version.startswith(wanted + "-"))):
            return None, None, (f"{tool} 的 LLVM 版本是「{version or '读不出'}」，rustc 的是「{wanted or '读不出'}」",
                                "装与 rustc 同一份的 rustup 组件：rustup component add llvm-tools（rustup update 之后要重装）；"
                                f"{LLVM_TOOLS_VARIABLE} 指的目录换成那一份")
        versions.append(f"{tool} {version}")
    return directory, f"{directory}（{'、'.join(versions)}；rustc 的 LLVM {wanted}）", None


def report(rows, judgements, outcome, memory_max, timeout_seconds, selection_path, skip_baseline, counts):
    """按变异表的行号排序打印判定、计数、基线与临时目录，判红的逐栏列出；回退出码。"""
    judgements.sort(key=lambda judgement: judgement[0])
    by_verdict = collections.defaultdict(list)
    for _line_number, verdict, text, _removal in judgements:
        by_verdict[verdict].append(text)
    caught, failures, invalid = by_verdict["caught"], by_verdict["failure"], by_verdict["invalid"]
    memory_hits, timeouts, squeezed = by_verdict["memory"], by_verdict["timeout"], by_verdict["squeezed"]
    refused, unavailable, baseline_bad = by_verdict["refused"], by_verdict["unavailable"], by_verdict["baseline"]
    not_run = by_verdict["notrun"]
    gpu_memory = by_verdict["gpumemory"]
    for text in caught:
        say(text)
    # 各栏各数各的；「计数：」这一行的格式别的定义与规则照栏读（mutation-triage），基线另起一行报
    say(f"  计数：抓到 {len(caught)} 条、没红 {len(failures)} 条、无效 {len(invalid)} 条、内存撞顶 {len(memory_hits)} 条、超时 {len(timeouts)} 条、"
        f"被总上限挤掉 {len(squeezed)} 条、排不上没跑 {len(refused)} 条、scope 起不来没跑 {len(unavailable)} 条（共 {len(rows)} 条）")
    if gpu_memory:   # 「计数：」那一行的格式不动（别的定义照栏读），显存不够另起一行报
        say(f"  显存不够：{len(gpu_memory)} 条（内存包装退 {GPU_MEMORY_HIT_EXIT}：不算抓到、不算没红，结果不算数）")
    if skip_baseline:
        say("  基线：没跑（GATE_MUTATION_BREAK=nobaseline，只给证红用）")
    else:
        say(f"  基线：没改坏的副本上点名的测试恰好对上一个用例、结果是 ok 的 {len(rows) - len(baseline_bad)} 行，基线不绿 {len(baseline_bad)} 行"
            f"（按 cargo 参数分 {len(baseline_batches(rows))} 批；共 {len(rows)} 行）")
    # 复用与覆盖各另起一行（「计数：」那一行的格式不动）；判定本身与进程数无关，这两行随记录变（比对两趟的判定时去掉，selftest_verdict_view）
    reuse_text = reuse_summary_text(counts)
    say(f"  复用：{reuse_text}")
    say(f"  覆盖：这一趟记了 {counts['coverage_recorded']} 组（cargo 参数 + 点名用例），那份快照上已有、沿用 {counts['coverage_existing']} 组，没记成 {counts['coverage_failed']} 组"
        + ("（逐组的原因在 stderr；那几组的行下一趟照「没有覆盖记录」现跑）" if counts["coverage_failed"] else ""))
    # 本次未跑的逐条列出来（文件头「本次未跑」）：不算抓到、不算没红；「计数：」那一行的格式不动，另起一行报
    not_run_record_problem = None
    if not_run:
        say(f"  本次未跑：{len(not_run)} 条（点名的测试在本机打了 NOT_RUN_NO_GPU_ADAPTER 后返回成功，没跑到被改坏的那一处：不算抓到、不算没红，这一趟不是整张表的结论）")
        for text in not_run:
            say(f"    · {text}")
        not_run_record_problem = write_not_run_record(counts.get("not_run_record", ""), not_run)
    removals = [removal for _line, _verdict, _text, removal in judgements if removal is not None]
    killed_removals = [removal for _line, verdict, _text, removal in judgements if removal is not None and verdict in VERDICTS_KILLED_BY_WRAPPER]
    left_behind = outcome["left_behind"]
    removal_state = "每条结束时不论退出码都整个删掉了" if not left_behind else f"跑完有 {len(left_behind)} 个没删掉"
    say(f"  临时目录：{len(removals)} 条变异各一个自己的 TMPDIR，{removal_state}；删前合计 {sum(removal[0] for removal in removals)} 个文件、"
        f"占盘 {sum(removal[1] for removal in removals)} 字节（表观 {sum(removal[2] for removal in removals)} 字节），"
        f"其中被包装停掉或杀掉的 {len(killed_removals)} 条（超时、内存撞顶、被总上限挤掉）留下 {sum(removal[0] for removal in killed_removals)} 个文件、"
        f"占盘 {sum(removal[1] for removal in killed_removals)} 字节")
    sections = [
        (baseline_bad, f"有几行基线不绿（基线不绿 {len(baseline_bad)} 行；点名的测试在没改坏的副本上就不是恰好一个 ok，改坏之后红了也证明不了什么，这几行的变异没跑）：",
         ["先看点名的测试读的文件在不在拷贝范围里（.claude/gate.d/stage-inputs.tsv 本阶段那一行；bash .claude/gate.d/code-source-discipline.sh 静态判这一样），",
          "不在就登记进那一行；再在真仓上单跑它（照那一行的 cargo test 参数），真仓上也红就是测试本来就坏，先修测试；对上多个用例的把点名写全到模块路径。"]),
        (invalid, f"有变异无效（无效 {len(invalid)} 条；替换文写进源码之后编不过，那条行为今天零变异覆盖）：",
         ["先看尾巴分来源，不是去补用例。有 process didn't exit successfully 与信号的，是测试进程被杀，先查是谁杀的；点名的名字里带 $ 这类字符的，改那一行的测试名；其余是替换文编不过——",
          "先看反斜杠：表里只有 \\n 会被还原成换行，\\& \\\" 这类原样写进源码就编不过；",
          "别的编译错就在副本里把替换后的那一行 cargo check 一遍，改成编得过、而且真会改行为的写法（.claude/rules/mutation-sampling.md 第八类）。"]),
        (failures, f"有变异没红（没红 {len(failures)} 条）：",
         ["没红 = 那处改法没有任何测试守着：先造一条会红的用例再改代码（show-me-test.md）；「没跑到」多半是 cargo test 参数选错了范围。"]),
        (memory_hits, f"有变异撞了内存上限（内存撞顶 {len(memory_hits)} 条，每条上限 {memory_max}）：",
         ["撞顶 = 这条破坏让被测代码无界分配，点名的测试没来得及红——不算抓到，也不是没红。换一处改法让它红在点名的测试上，",
          "或者给被测代码加一道先红的界（循环步数上限、分配量的断言）；正常的测试确实要这么多内存，才调大上限（表头「# 每条变异的内存上限：…」或 GATE_MUTATION_MEMORY_MAX）。"]),
        (timeouts, f"有变异超时（超时 {len(timeouts)} 条，每条限 {timeout_seconds} 秒）：",
         ["超时 = 这条破坏让被测代码不终止了，或者点名的测试在并发下就要跑这么久——不算抓到，也不是没红。先在副本里只改坏这一处、",
          "不限时单跑它那一行的 cargo test 参数，看它结束不结束：不结束就换一处改法让它红在点名的测试上，或者给被测代码加一道先红的界（循环步数上限）；",
          "结束得了就调大限时（表头「# 每条变异的超时秒数：…」或 GATE_MUTATION_TIMEOUT），别把它记成抓到。"]),
        (squeezed, f"有变异被 slice 的总上限挤掉（被总上限挤掉 {len(squeezed)} 条；这几条没撞各自的上限 {memory_max}，是 slice 里合起来满了）：",
         ["这几条的结果不算数，重跑（抓到的有判定库记录，只重跑没作数的）；常这样说明包装的峰值表记少了（同时跑的几条实际占的比表里记的多），",
          "跑的时候 bash scripts/hosts/run-with-memory-cap.sh --status 看 slice 里是谁在占，别拿掉包装去跑。"]),
        (gpu_memory, f"有变异显存不够（显存不够 {len(gpu_memory)} 条；内存包装认出它们起止之间内核报了 NV_ERR_NO_MEMORY）：",
         ["这几条的结果不算数，重跑（抓到的有判定库记录，只重跑没作数的）；先看同一段时间谁在用卡（nvidia-smi），少给这几条或同时在跑的几条一些显存，",
          "上卡的跑法经 scripts/hosts/host-dispatch.py --gpu 按各台显存余量派；别把它记成抓到。"]),
        (refused, f"有变异排不上没跑（排不上没跑 {len(refused)} 条；包装等满了等待上限，slice 还是放不下）：",
         ["bash scripts/hosts/run-with-memory-cap.sh --status 看 slice 被谁占着，等那几件重活跑完再重跑；", "别拿掉包装去跑——排不上说明此刻整机放不下。"]),
        (unavailable, f"跑到一半带内存上限的 scope 起不来了（{len(unavailable)} 条没跑）：",
         ["修好用户级 systemd / D-Bus（systemctl --user status）再重跑；不许拿掉上限去跑。"]),
    ]
    for items, headline, howto_lines in sections:
        if items:
            print_rejection(headline, items, howto_lines)
    if left_behind:
        print_rejection(f"有 {len(left_behind)} 个临时目录跑完没删掉（{outcome['per_mutation_temporary_root']} 底下还剩这几个；line-<行号> 是变异、baseline-<行号> 是那一批基线，行号是 {TABLE} 的）：",
                        [f"{name}：里面 {len(outcome['left_behind_contents'][name])} 项，前几项 {'、'.join(outcome['left_behind_contents'][name][:5]) or '（空）'}"
                         for name in left_behind],
                        ["看 judge_row / judge_batch 末尾 measure_and_remove_temporary_directory 那一步跑到没有、删不删得掉（只读的目录、没退干净的进程又把它建回来）；",
                         "这一道收尾时连同源码副本一起删掉，现场只剩上面列的名字。别拿掉这一步：被杀的测试 Drop 守卫不跑，它建的镜像全堆在这一轮的 TMPDIR 里，一轮能堆几百 G。"])
    if not_run_record_problem:
        print_rejection(f"本次未跑的 {len(not_run)} 条没写进 GATE_MUTATION_NOT_RUN_RECORD 那份记录（{not_run_record_problem}），门禁汇总就看不见它们", [],
                        ["看那份文件所在的目录满没满、有没有写权限；记不下本次未跑的这一趟按红记，免得被当成全判过。"])
    red = bool(invalid or failures or memory_hits or timeouts or squeezed or gpu_memory or refused or unavailable or baseline_bad or left_behind
               or not_run_record_problem)
    note(f"  … 这一轮 {counts['worker_count']} 个工作进程，每个 cargo 编译并行度 {counts['cargo_jobs']}，每条变异内存上限 {memory_max}、限时 {timeout_seconds} 秒；"
         f"复用抓到 {counts['reused']} 条、复用基线 ok {counts['reused_baseline']} 行，现跑变异 {counts['fresh_mutations']} 条、现跑基线 {counts['fresh_batches']} 批")
    if selection_path:
        # 只判了一部分：不写全绿标记；都抓到也退 3（门禁汇总记失败，不会被当成通过），有红退 1
        say(f"  … 只判了选中的 {len(rows)} 行，不是整道结论（GATE_MUTATION_ROW_SELECTION={selection_path}；"
            f"{'有红，见上面' if red else (f'选中的里抓到 {len(caught)} 条、本次未跑 {len(not_run)} 条' if not_run else '选中的都红在点名的测试上')}；"
            f"判定库记录写进了 {counts['records_directory']}）；这一趟{reuse_text}")
        return 1 if red else EXIT_SELECTION_ONLY
    if red:
        return 1
    if not_run:
        # 有本次未跑的：不说全抓到、不写全绿标记（没判全的一趟不算上次成功）；一条抓到的都没有就退 77（无对象判到）
        if not caught:
            say(f"  · crates 变异表复跑：{len(rows)} 条全是本次未跑（见上），一条都没判到被改坏的那一处：这一道记本次未跑，不是通过；全绿标记没写")
            return 77
        say(f"  ✓ crates 变异表复跑：{len(rows)} 条里抓到 {len(caught)} 条、本次未跑 {len(not_run)} 条（本次未跑的逐条见上，不算抓到；这一趟不是整张表都红在点名的测试上，"
            f"全绿标记没写）；没红、无效、内存撞顶、超时、被总上限挤掉、排不上 0 条，每条一个自己的 TMPDIR、结束时都删掉了；这一趟{reuse_text}")
        return 0
    # ⚠️ 进程数不写进 stdout：写进去就让「GATE_MUTATION_WORKERS=1 与 =16 的 stdout 逐字相同」当场为假；复用几条接在成功句末尾「；这一趟现判」之后
    say(f"  ✓ crates 变异表复跑：{len(rows)} 条变异各自红在点名的测试上（原文都恰好命中一次；点名的测试在没改坏的副本上都恰好一个 ok；"
        f"每条在内存上限 {memory_max} 里跑、内存撞顶 0 条，每条经 run-with-memory-cap.sh 排队、被总上限挤掉与排不上 0 条，每条限时 {timeout_seconds} 秒、超时 0 条，"
        f"每条一个自己的 TMPDIR、结束时都删掉了；工作进程动态领活——谁先跑完谁再领下一条，不按条数预先切片；输出按表的行号排序、判定与进程数无关）"
        f"；这一趟{reuse_text}")
    write_stage_marker_with_counts(counts["root"], dict(reused_caught_rows=counts["reused"], reused_caught_rows_same_snapshot=counts["reused_same_snapshot"],
                                                        reused_baseline_ok_rows=counts["reused_baseline"],
                                                        freshly_run_mutation_rows=counts["fresh_mutations"], freshly_run_baseline_batches=counts["fresh_batches"],
                                                        row_records=counts["records_directory"]))
    return 0


def reuse_summary_text(counts):
    """「现判 N 条（各栏的数）、按覆盖复用 M 条（其中一字未动 K 条）」：成功句末尾与「复用：」那一行用同一句。"""
    columns = "、".join(f"{title} {counts['fresh_columns'].get(name, 0)}" for name, title in REUSE_COLUMNS)
    return (f"现判 {counts['fresh_rows']} 条（{columns}）、按覆盖复用 {counts['reused']} 条"
            f"（其中源码与上次一字未动 {counts['reused_same_snapshot']} 条）")


def write_not_run_record(path, texts):
    """每条本次未跑往 GATE_MUTATION_NOT_RUN_RECORD 那份文件追加一行（阶段转给 gate.sh 的 GATE_NOT_RUN_FILE）；没设不写。写不进去交那一句，写成交 None。"""
    if not path:
        return None
    try:
        with open(path, "a", encoding="utf-8") as handle:
            handle.write("".join(f"{STAGE} 本次未跑：{text}\n" for text in texts))
    except OSError as error:
        return f"{path}：{error}"
    return None


def write_stage_marker_with_counts(root, counts):
    """判绿之后写这一道这批输入的全绿标记（admission.py 的 write_stage_marker），再把复用几条、现跑几条接在末尾（同目录临时文件写完改名）。写不成只报不红。"""
    admission = admission_module()
    try:
        fingerprint, file_count, _paths = admission.stage_fingerprint(root, STAGE)
        marker_path = admission.write_stage_marker(root, STAGE, fingerprint, file_count)
        with open(marker_path, encoding="utf-8", errors="surrogateescape") as handle:
            text = handle.read()
        write_atomically(marker_path, text + "".join(f"{key}={value}\n" for key, value in counts.items()))
    except (admission.RegistrationError, admission.InputManifestError, OSError) as error:
        say(f"  ! 没写成这一道的全绿标记（{error}）：下一趟这一道照跑，不会复用")


# ── 命令行 ────────────────────────────────────────────────────────────────────

def static_check_usage_problem(arguments):
    """static-check 的参数哪里不对：→ None（对）或 (原因, 下一步)。"""
    if not arguments:
        return "缺 <根>（被判那棵树的仓根）", 'python3 scripts/mutation/crates-mutation-rows.py static-check "$PWD"（在仓根跑）'
    if arguments[0].startswith("-"):
        return f"第一个参数要是 <根>，给的是选项 {arguments[0]}", 'python3 scripts/mutation/crates-mutation-rows.py static-check "$PWD" [--table <变异表>]'
    if len(arguments) == 2 and arguments[1] == "--table":
        return "--table 后面没给变异表的路径", f"python3 scripts/mutation/crates-mutation-rows.py static-check {arguments[0]} --table <变异表>"
    if len(arguments) not in (1, 3) or (len(arguments) == 3 and arguments[1] != "--table"):
        return (f"参数个数或写法不对（给了 {len(arguments)} 个：{' '.join(arguments)}；只认 <根> 与可选的 --table <变异表>）",
                f"python3 scripts/mutation/crates-mutation-rows.py static-check {arguments[0]} [--table <变异表>]")
    if not os.path.isdir(arguments[0]):
        return f"<根> {arguments[0]} 不是一个目录", "<根> 写被判那棵树的仓根（git 工作树），例如 \"$PWD\""
    return None


def command_static_check(arguments):
    problem = None if break_is(os.environ, "staticusagebare") else static_check_usage_problem(arguments)
    if problem is not None:
        print(f"ERROR\t{problem[0]}；用法：crates-mutation-rows.py static-check <根> [--table <变异表>]")
        print(f"     → 怎么办：{problem[1]}")
        return 2
    if not arguments or len(arguments) not in (1, 3) or (len(arguments) == 3 and arguments[1] != "--table"):
        print("ERROR\t用法：crates-mutation-rows.py static-check <根> [--table <变异表>]")
        return 2
    root = arguments[0]
    table_path = arguments[2] if len(arguments) == 3 else os.path.join(root, TABLE)
    if len(arguments) == 3 and not os.path.isfile(table_path) and not break_is(os.environ, "tablemissingok"):
        print(f"ERROR\t--table 指的变异表不在：{table_path}（不判，也不报「查了 0 行」）")
        print("     → 怎么办：核对路径（相对路径相对当前工作目录）；判仓里那一张就去掉 --table（默认读 <根>/crates/mutations.tsv）")
        return 2
    admission = admission_module()
    try:
        copied = copied_paths()
    except admission.RegistrationError as error:
        print(f"ERROR\t读不到 .claude/gate.d/stage-inputs.tsv 里 {STAGE} 那一行：{error}")
        return 2
    if not copied:
        print(f"ERROR\t.claude/gate.d/stage-inputs.tsv 里 {STAGE} 那一行一条路径都没登记")
        return 2
    rows, malformed = [], []
    if os.path.isfile(table_path):
        try:
            rows, malformed, _memory, _timeout = read_table(table_path)
        except (OSError, UnicodeDecodeError) as error:
            print(f"ERROR\t读不了 {table_path}：{error}")
            return 2
    problems = row_problems(root, rows, malformed, copied)
    literals, scanned = escaping_literals(root, copied)
    outside_literals = [literal for literal in literals if not literal[4]]
    print("COPIED\t" + " ".join(copied))
    for kind in ROW_PROBLEM_KINDS:
        for detail in problems[kind]:
            print("\t".join([kind] + [str(item) for item in detail]))
    for relative, line_number, literal, resolved, _inside in outside_literals:
        print(f"LITERAL\t{relative}\t{line_number}\t{literal}\t{resolved}")
    print(f"CHECKED\t{len(rows)}\t{scanned}\t{len(literals)}")
    return 1 if any(problems.values()) or outside_literals else 0


def command_judge_version(arguments, want_directory=False):
    if len(arguments) != 1:
        print(f"  ✗ 用法：crates-mutation-rows.py {'records-directory' if want_directory else 'judge-version'} <根>")
        print("     → 怎么办：<根> 是被判那棵树的仓根（git 工作树）")
        return 2
    admission = admission_module()
    environment = dict(os.environ)
    try:
        fingerprint, item_count = judge_version(arguments[0], environment, environment.get("GATE_MUTATION_MEMORY_CAP_RUNNER", "").strip()
                                                      or DEFAULT_MEMORY_CAP_RUNNER)
        common_directory = records_home(arguments[0], environment)
    except (admission.RegistrationError, admission.InputManifestError, OSError) as error:
        print(f"  ✗ 算不出判法版本：{error}")
        print("     → 怎么办：被判的要是 git 工作树、.claude/gate.d/stage-inputs.tsv 里有 checker-tier-crates-mutation-replay 那一行、构建环境读得到")
        return 2
    print(os.path.join(common_directory, RECORDS_DIRECTORY_NAME, fingerprint) if want_directory else f"{fingerprint} {item_count}")
    return 0


def plan_context(root, environment):
    """plan、row-plan、import 共用：读表、算上限、拷贝范围、判法版本、工具链、记录目录与此刻的快照；一个字节都不写。
    交 dict；不成就打两行（✗ 与 →）交 None。"""
    admission = admission_module()
    try:
        rows, _malformed, memory_max_from_table, timeout_from_table = read_table(os.path.join(root, TABLE), environment)
    except (OSError, UnicodeDecodeError) as error:
        print(f"  ✗ 读不了 {os.path.join(root, TABLE)}：{error}")
        print("     → 怎么办：先让这张表在、是 UTF-8；读不了就一行都判不了")
        return None
    memory_max, timeout_seconds, limit_problems = limits_of(memory_max_from_table, timeout_from_table, environment)
    if limit_problems:
        print(f"  ✗ {limit_problems[0][0]}")
        print(f"     → 怎么办：{limit_problems[0][1]}")
        return None
    try:
        copied = copied_paths()
        judging, _items = judge_version(root, environment, environment.get("GATE_MUTATION_MEMORY_CAP_RUNNER", "").strip() or DEFAULT_MEMORY_CAP_RUNNER)
        toolchain = toolchain_text(root, environment)
        home = records_home(root, environment)
    except (admission.RegistrationError, admission.InputManifestError, OSError) as error:
        print(f"  ✗ 算不出判法版本或工具链：{error}")
        print("     → 怎么办：被判的要是 git 工作树、cargo -V 与 rustc -V 跑得通、.claude/gate.d/stage-inputs.tsv 里有 checker-tier-crates-mutation-replay 那一行")
        return None
    records_directory = os.path.join(home, RECORDS_DIRECTORY_NAME, judging)
    store = Store(records_directory)
    return dict(rows=rows, row_keys={row.line_number: row_key(row, memory_max, timeout_seconds, toolchain) for row in rows},
                memory_max=memory_max, timeout_seconds=timeout_seconds, judging=judging, toolchain=toolchain, home=home,
                records_directory=records_directory, store=store, current=current_state(root, copied),
                store_has_records=store.has_new_format_records(), timings_directory=os.path.join(home, TIMINGS_DIRECTORY_NAME))


def planned_decisions(context, environment):
    """每一行 (row, Decision)：与 run_stage 同一份 reuse_decision，只读。"""
    refusal, comparisons, decided = reuse_refusal(environment), {}, []
    for row in context["rows"]:
        key = context["row_keys"][row.line_number]
        record = read_record(os.path.join(context["records_directory"], f"mutation.{key}"))
        decided.append((row, reuse_decision(row, key, record, context["store"], context["current"], comparisons, environment, refusal,
                                            context["store_has_records"])))
    return decided


def command_row_plan(arguments):
    """每一条成形的变异一行：行号、行键、这一趟会不会复用、上一次现跑的秒数（见文件头 row-plan）。"""
    if len(arguments) != 1:
        print("  ✗ 用法：crates-mutation-rows.py row-plan <根>")
        print("     → 怎么办：<根> 是被判那棵树的仓根（git 工作树）")
        return 2
    environment = dict(os.environ)
    context = plan_context(arguments[0], environment)
    if context is None:
        return 2
    for row, decision in planned_decisions(context, environment):
        timing = read_timing(context["timings_directory"], row_content_hash(row))
        print(f"{row.line_number}\t{context['row_keys'][row.line_number]}\t{1 if decision.reuse else 0}\t{'-' if timing is None else f'{timing[0]:.3f}'}")
    return 0


def command_plan(arguments):
    """只读：每一行会复用还是现跑、栏与细节，末行总数（文件头 plan）。"""
    if len(arguments) != 1:
        print("  ✗ 用法：crates-mutation-rows.py plan <根>")
        print("     → 怎么办：<根> 是被判那棵树的仓根（git 工作树）；只读，不编、不跑、不写")
        return 2
    environment = dict(os.environ)
    context = plan_context(arguments[0], environment)
    if context is None:
        return 2
    columns, reused, same_snapshot = collections.Counter(), 0, 0
    for row, decision in planned_decisions(context, environment):
        if decision.reuse:
            reused += 1
            same_snapshot += decision.how == "same-snapshot"
            print(f"{row.line_number}\t{row.name}\t复用\t按覆盖复用\t{decision.detail}")
        else:
            columns[decision.column] += 1
            print(f"{row.line_number}\t{row.name}\t现跑\t{COLUMN_TITLES[decision.column]}\t{decision.detail}")
    fresh = sum(columns.values())
    print(f"合计 {len(context['rows'])} 行：按覆盖复用 {reused} 行（其中源码与上次一字未动 {same_snapshot} 行）、现跑 {fresh} 行（"
          + "、".join(f"{title} {columns.get(name, 0)}" for name, title in REUSE_COLUMNS) + "）")
    return 0


def copy_record_dependencies(record, source, target):
    """导入的一条记录引的覆盖记录、快照与源码副本，从 source 拷进 target（已有的不动）。"""
    snapshots = {record.get("snapshot")} - {None, ""}
    coverage = source.coverage(record.get("coverage", "")) if record.get("coverage") else None
    if coverage is not None:
        target.put_coverage(record["coverage"], coverage)
        snapshots.add(coverage.get("snapshot"))
    for identity in snapshots:
        files = source.snapshot(identity)
        if files is not None:
            target.put_snapshot(identity, files)
    if coverage is not None:
        files = source.snapshot(coverage.get("snapshot")) or {}
        for path, kind in coverage.get("compiled", {}).items():
            if kind == "runtime" and path.endswith(".rs") and path in files:
                data = source.blob(files[path])
                if data is not None:
                    target.put_blob(files[path], data)


def command_import(arguments):
    if len(arguments) != 2:
        print("  ✗ 用法：crates-mutation-rows.py import <根> <别处的记录目录>")
        print("     → 怎么办：<别处的记录目录> 是另一台 singlefs-mutation-rows/<判法版本>/ 拷过来的那一份（连同 coverage/、snapshots/、blobs/）")
        return 2
    root, source_directory = arguments
    environment = dict(os.environ)
    context = plan_context(root, environment)
    if context is None:
        return 2
    if not os.path.isdir(source_directory):
        print(f"  ✗ {source_directory} 不是目录")
        print("     → 怎么办：给另一台 singlefs-mutation-rows/<判法版本>/ 拷过来的那一份目录")
        return 2
    rows_by_key = {context["row_keys"][row.line_number]: row for row in context["rows"]}
    source, target = Store(source_directory), context["store"]
    target.ensure()
    unchecked = break_is(environment, "importunchecked")
    accepted, rejected, comparisons = [], [], {}
    for name in sorted(os.listdir(source_directory)):
        path = os.path.join(source_directory, name)
        if os.path.isdir(path) and name in (COVERAGE_DIRECTORY_NAME, SNAPSHOT_DIRECTORY_NAME, BLOB_DIRECTORY_NAME):
            continue
        record = read_record(path) if os.path.isfile(path) else None
        if record is None:
            rejected.append((name, "不是一份读得出的记录文件"))
            continue
        problems = []
        kind, key = record.get("kind", ""), record.get("row_key", "")
        if kind not in RECORD_KINDS:
            problems.append(f"类别「{kind}」认不出（只认 {'、'.join(RECORD_KINDS)}）")
        if name != f"{kind}.{key}":
            problems.append("文件名与记录里的类别、行键对不上")
        if record.get("judge_version") != context["judging"]:
            problems.append(f"判法版本 {record.get('judge_version', '')[:16]}… 不是本机此刻的 {context['judging'][:16]}…")
        if key not in rows_by_key:
            problems.append(f"行键不是本机 {TABLE} 里任何一行（连同本机这一趟的每条上限 {context['memory_max']}、限时 {context['timeout_seconds']} 秒与工具链）算得出的")
        if not problems:
            if kind == "mutation" and record.get("verdict") == REUSABLE_VERDICT["mutation"]:
                decision = reuse_decision(rows_by_key[key], key, record, source, context["current"], comparisons, environment, None, True)
                if not decision.reuse:
                    problems.append(f"照本机此刻的源码判成现跑（{COLUMN_TITLES[decision.column]}：{decision.detail}）")
            elif kind == "baseline" and record.get("verdict") == REUSABLE_VERDICT["baseline"] and record.get("snapshot") != context["current"]["id"]:
                problems.append("基线 ok 那一份快照不是本机此刻的")
        if unchecked:
            problems = []
        if problems:
            rejected.append((name, "；".join(problems)))
            continue
        copy_record_dependencies(record, source, target)
        with open(path, encoding="utf-8", errors="surrogateescape") as handle:
            write_atomically(os.path.join(context["records_directory"], name), handle.read())
        accepted.append(name)
    print(f"  … 导入：收了 {len(accepted)} 条、拒了 {len(rejected)} 条（写进 {context['records_directory']}）")
    if rejected:
        print(f"  ✗ 拒了 {len(rejected)} 条另一台的判定库记录（文件名、判法版本、行键或复用判据与本机此刻对不上，照收会让本机复用别的输入下的结论）：")
        for name, reason in rejected:
            print(f"      {name}：{reason}")
        print("     → 怎么办：两台的判法版本（阶段脚本、本模块、内存包装、构建环境）、变异表、上限、限时与工具链要同一套；源码不同的照复用判据判，")
        print("       判成现跑的那几行本机 checker-tier-crates-mutation-replay 照跑，不会少判。")
        return 1
    return 0


def command_list_items(arguments):
    """可点名的变异，一行一条（门禁 --list-items 调的就是这一份）。"""
    if len(arguments) != 1:
        print("  ✗ 用法：crates-mutation-rows.py --list-items <根>")
        print("     → 怎么办：<根> 是被判那棵树的仓根")
        return 2
    if not break_is(os.environ, "listitemspipeunhandled"):
        # 接 | head 时读的一头先关了：照 SIGPIPE 的惯例安静退出（被 SIGPIPE 停，shell 里退 141），不打 BrokenPipeError 的回溯
        signal.signal(signal.SIGPIPE, signal.SIG_DFL)
    path = os.path.join(arguments[0], TABLE)
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except (OSError, UnicodeDecodeError) as error:
        print(f"  ✗ 读不了 {path}（{error}）：列不出可点名的变异")
        print("     → 怎么办：在项目根跑，或把项目根作为参数传进来。")
        return 1
    for number, line in enumerate(lines, 1):
        if line.startswith("#") or not line.strip():
            continue
        fields = line.split("\t")
        print(f"{fields[0]}\t第 {number} 行，改 {fields[1] if len(fields) > 1 else ''}")
    return 0


def command_item(arguments):
    """--item <行号或变异名> [--item …] <根>：交给门禁 checker-tier-crates-mutation-replay.sh --item（同一份选行模式）。"""
    items, rest, index = [], [], 0
    while index < len(arguments):
        if arguments[index] == "--item":
            if index + 1 >= len(arguments) or not arguments[index + 1]:
                print("  ✗ --item 后面缺行号或变异名")
                print("     → 怎么办：写成 --item <行号或变异名>（可给几次）<根>；可点的用 crates-mutation-rows.py --list-items <根> 看")
                return 2
            items.append(arguments[index + 1])
            index += 2
        else:
            rest.append(arguments[index])
            index += 1
    if not items or len(rest) != 1:
        print("  ✗ 用法：crates-mutation-rows.py --item <行号或变异名> [--item …] <根>")
        print("     → 怎么办：至少点一行、最后给被判那棵树的仓根；可点的用 crates-mutation-rows.py --list-items <根> 看")
        return 2
    command = ["bash", STAGE_SCRIPT] + [word for item in items for word in ("--item", item)] + rest
    sys.stdout.flush()
    os.execvp("bash", command)
    return 2   # execvp 不返回


def command_llvm_tools(arguments):
    if len(arguments) != 1:
        print("  ✗ 用法：crates-mutation-rows.py llvm-tools <根>")
        print("     → 怎么办：<根> 是被判那棵树的仓根（rustc 在那里起，认得它的 rust-toolchain 文件）")
        return 2
    _directory, text, problem = llvm_tools(arguments[0], dict(os.environ))
    if problem is not None:
        print(f"  ✗ {problem[0]}")
        print(f"     → 怎么办：{problem[1]}")
        return 1
    print(f"  ✓ {text}")
    return 0


# ── 自证 ──────────────────────────────────────────────────────────────────────

FAKE_CARGO = r'''#!/usr/bin/env python3
# 自证用的假 cargo：-V 打一行版本（FAKE_CARGO_VERSION 设了打它）；test 读 crates/tiny/src/lib.rs 里「// TEST <名字> NEEDS <原文>」「// TEST <名字> READS <相对 crate 根的路径>」，
# 原文在、文件在就 ok，不在就 FAILED；NEEDS 那一句末尾可以跟「 RUNS <函数名>[,<函数名>…]」，说这条用例执行哪几个函数（不跟就是定义行里有那段原文的函数）；
# 「// TEST <名字> NOTRUN <一句>」照 GPU 对拍用例在没卡的机器上：一律 ok，带 --show-output 时照 libtest 的样子
# 在末尾交出它打的 NOT_RUN_NO_GPU_ADAPTER 那一行；每起一次往 $FAKE_CARGO_LOG 追加一行「<参数>\t<lib.rs 的 sha256 前 12 位>」；
# $FAKE_CARGO_ENVIRONMENT_LOG 设了，每起一次 test 再往它追加一行 JSON：收到的 *_THREADS、SINGLEFS_THREAD_CAP、CARGO_BUILD_JOBS 与 CARGO_TARGET_DIR（现在没有自证格读它，留给排查）。
# 开头的 --config <值> 跳过。test --no-run（插桩编译）：每个 -p 点名的包（不点名是 tiny）在 $CARGO_TARGET_DIR/debug/deps/ 下写一个假的测试可执行文件
# （内容是 JSON：源码副本的根与这个包 src/ 下的 .rs）与它的 dep-info，打一行 compiler-artifact 消息；
# 设了 LLVM_PROFILE_FILE 的 test 把这一趟跑到的用例执行的函数写进它指的那份 .profraw（%p 换成进程号、%m 换成 0，内容是 JSON）。
# test --no-run 还照真插桩编译里构建脚本的样子写一份剖析文件：有 LLVM_PROFILE_FILE 写它指的，没有写当前目录的 default_%m_%p.profraw；
# 写了的路径连同当前目录追加进 $FAKE_CARGO_PROFILE_LOG（设了才记）。
# `--` 之前的 --features / -F 里有 gpu 或 verdict-store 的，照真 cargo 编 librocksdb-sys 那样要 BINDGEN_EXTRA_CLANG_ARGS：环境里没有、
# 从当前目录往上每一层的 .cargo/config.toml（或 .cargo/config）的 [env] 里也没有，就照它的构建脚本报 stdbool.h 找不到、退 101，一个用例都不跑
import hashlib, json, os, re, sys, tomllib
arguments = sys.argv[1:]
while arguments[:1] == ["--config"]:
    arguments = arguments[2:]
if arguments[:1] in (["-V"], ["--version"]):
    print(os.environ.get("FAKE_CARGO_VERSION") or "cargo 0.0.0 (crates-mutation-rows selftest)")
    sys.exit(0)
if arguments[:1] != ["test"]:
    sys.exit(2)
source = open("crates/tiny/src/lib.rs", encoding="utf-8").read()
with open(os.environ["FAKE_CARGO_LOG"], "a", encoding="utf-8") as log:
    log.write(" ".join(arguments[1:]) + "\t" + hashlib.sha256(source.encode()).hexdigest()[:12] + "\n")
if os.environ.get("FAKE_CARGO_ENVIRONMENT_LOG"):
    # 设了才记：每起一次 test 往里追加一行 JSON，收到的线程数变量、编译并行度与编译目录
    names = ("RUST_TEST_THREADS", "CARGO_BUILD_JOBS", "CARGO_TARGET_DIR", "SINGLEFS_CRASH_AMPLIFICATION_HOME", "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY")
    with open(os.environ["FAKE_CARGO_ENVIRONMENT_LOG"], "a", encoding="utf-8") as environment_log:
        environment_log.write(json.dumps({name: value for name, value in os.environ.items() if name in names or name.endswith("_THREADS") or name == "SINGLEFS_THREAD_CAP"}, sort_keys=True) + "\n")
if os.environ.get("FAKE_CARGO_HANG_FILE"):
    # 自证「被 TERM 停」那一格：起来记一笔，等到放行、父进程（工作进程）没了或 30 秒，记一笔「退了」再退 101
    import time
    parent = os.getppid()
    with open(os.environ["FAKE_CARGO_HANG_FILE"], "w", encoding="utf-8") as handle:
        handle.write(str(parent))
    deadline = time.monotonic() + 30
    while time.monotonic() < deadline and os.getppid() == parent and not os.path.exists(os.environ["FAKE_CARGO_RELEASE_FILE"]):
        time.sleep(0.05)
    open(os.environ["FAKE_CARGO_DONE_FILE"], "w").close()
    sys.exit(101)
before = arguments[1:arguments.index("--")] if "--" in arguments else arguments[1:]
features = set()
for index, word in enumerate(before):
    if word in ("--features", "-F") and index + 1 < len(before):
        word = "--features=" + before[index + 1]
    if word.startswith("--features="):
        features |= {feature.split("/")[-1] for feature in word[len("--features="):].replace(",", " ").split()}


def bindgen_include_given():
    if os.environ.get("BINDGEN_EXTRA_CLANG_ARGS"):
        return True
    directory = os.getcwd()
    while True:
        for name in ("config.toml", "config"):
            path = os.path.join(directory, ".cargo", name)
            if os.path.isfile(path):
                with open(path, "rb") as handle:
                    if "BINDGEN_EXTRA_CLANG_ARGS" in tomllib.load(handle).get("env", {}):
                        return True
        parent = os.path.dirname(directory)
        if parent == directory:
            return False
        directory = parent


if features & {"gpu", "verdict-store"} and not bindgen_include_given():
    print("error: failed to run custom build command for `librocksdb-sys v0.0.0 (crates-mutation-rows selftest)`", file=sys.stderr)
    print("  rocksdb/include/rocksdb/c.h:65:10: fatal error: 'stdbool.h' file not found", file=sys.stderr)
    sys.exit(101)
if "--no-run" in before:
    packages = [before[index + 1] for index, word in enumerate(before) if word == "-p" and index + 1 < len(before)] or ["tiny"]
    deps = os.path.join(os.environ["CARGO_TARGET_DIR"], "debug", "deps")
    os.makedirs(deps, exist_ok=True)
    for package in packages:
        crate = os.path.join(os.getcwd(), "crates", package)
        sources = sorted(os.path.relpath(os.path.join(directory, name), os.getcwd()) for directory, _names, names in os.walk(os.path.join(crate, "src"))
                         for name in names if name.endswith(".rs"))
        executable = os.path.join(deps, f"{package}-0000000000000000")
        with open(executable, "w", encoding="utf-8") as handle:
            json.dump({"root": os.getcwd(), "sources": sources}, handle)
        with open(executable + ".d", "w", encoding="utf-8") as handle:
            handle.write(f"{executable}: {' '.join(sources)}\n\n")
        print(json.dumps({"reason": "compiler-artifact", "manifest_path": os.path.join(crate, "Cargo.toml"), "target": {"kind": ["lib"], "name": package},
                          "profile": {"test": True}, "filenames": [executable], "executable": executable, "fresh": False}))
    stray = os.path.abspath((os.environ.get("LLVM_PROFILE_FILE") or "default_%m_%p.profraw").replace("%p", str(os.getpid())).replace("%m", "0"))
    os.makedirs(os.path.dirname(stray), exist_ok=True)
    with open(stray, "w", encoding="utf-8") as handle:
        handle.write("{}")
    if os.environ.get("FAKE_CARGO_PROFILE_LOG"):
        with open(os.environ["FAKE_CARGO_PROFILE_LOG"], "a", encoding="utf-8") as log:
            log.write(os.getcwd() + "\t" + stray + "\n")
    sys.exit(0)
after = arguments[arguments.index("--") + 1:] if "--" in arguments else []
filters = [word for word in after if not word.startswith("-")]
failed = False
shown, executed = [], set()
definitions = {match.group(1): match.group(0) for match in re.finditer(r"(?m)^\s*(?:pub\s+)?(?:const\s+)?fn\s+(\w+).*$", source)}
for line in source.splitlines():
    words = line.split(" ", 4)
    if len(words) < 5 or words[:2] != ["//", "TEST"]:
        continue
    name, how, what = words[2], words[3], words[4]
    if filters and not any(word in "tests::" + name for word in filters):
        continue
    if how == "NOTRUN":
        print(f"test tests::{name} ... ok")
        if "--show-output" in after:
            shown.append(f"---- tests::{name} stdout ----\nNOT_RUN_NO_GPU_ADAPTER test={name} reason={what}\n")
        continue
    what, _runs, runs = what.partition(" RUNS ")
    executed |= set(runs.split(",")) if runs else {function for function, definition in definitions.items() if what in definition}
    ok = (what in source.replace(line, "")) if how == "NEEDS" else os.path.isfile(os.path.join("crates", "tiny", what))
    failed = failed or not ok
    print(f"test tests::{name} ... {'ok' if ok else 'FAILED'}")
if shown:
    print("\nsuccesses:\n\n" + "\n".join(shown))
if os.environ.get("LLVM_PROFILE_FILE"):
    with open(os.environ["LLVM_PROFILE_FILE"].replace("%p", str(os.getpid())).replace("%m", "0"), "w", encoding="utf-8") as handle:
        json.dump({"executed": sorted(executed)}, handle)
sys.exit(101 if failed else 0)
'''
# 自证用的假 llvm 工具（同一份写成 llvm-profdata 与 llvm-cov 两个名字，按名字分工）：--version 打 rustc -vV 的 LLVM 版本（FAKE_LLVM_VERSION 设了打它）；
# llvm-profdata merge 把几份 .profraw 的函数名并起来；llvm-cov export 读合并的那一份与每个假可执行文件，照源码里每个 fn 的定义行交函数（区域起在 fn 那一列），
# 计数是执行过的 1、没执行的 0
FAKE_LLVM_TOOL = r'''#!/usr/bin/env python3
import json, os, re, subprocess, sys
arguments = sys.argv[1:]
if arguments[:1] == ["--version"]:
    version = os.environ.get("FAKE_LLVM_VERSION") or re.search(r"LLVM version: (\S+)", subprocess.run(["rustc", "-vV"], capture_output=True, text=True).stdout).group(1)
    print(f"LLVM (http://llvm.org/):\n  LLVM version {version}-rust-selftest\n  Optimized build.")
    sys.exit(0)
if os.path.basename(sys.argv[0]) == "llvm-profdata":
    if arguments[:1] != ["merge"]:
        sys.exit(2)
    executed = set()
    for path in arguments[1:]:
        if path.endswith(".profraw"):
            with open(path, encoding="utf-8") as handle:
                executed |= set(json.load(handle)["executed"])
    with open(arguments[arguments.index("-o") + 1], "w", encoding="utf-8") as handle:
        json.dump({"executed": sorted(executed)}, handle)
    sys.exit(0)
if arguments[:1] != ["export"]:
    sys.exit(2)
profile, objects, index = None, [], 1
while index < len(arguments):
    word = arguments[index]
    if word == "-instr-profile":
        profile, index = arguments[index + 1], index + 2
    elif word == "--object":
        objects.append(arguments[index + 1])
        index += 2
    elif word.startswith("-"):
        index += 1
    else:
        objects.append(word)
        index += 1
with open(profile, encoding="utf-8") as handle:
    executed = set(json.load(handle)["executed"])
functions, seen = [], set()
for path in objects:
    with open(path, encoding="utf-8") as handle:
        built = json.load(handle)
    for source in built["sources"]:
        full = os.path.join(built["root"], source)
        if full in seen:
            continue
        seen.add(full)
        with open(full, encoding="utf-8") as handle:
            for number, line in enumerate(handle.read().splitlines(), 1):
                match = re.match(r"^(\s*)(?:pub\s+)?(?:const\s+)?fn\s+(\w+)", line)
                if match:
                    count = 1 if match.group(2) in executed else 0
                    functions.append({"name": match.group(2), "count": count, "filenames": [full],
                                      "regions": [[number, len(match.group(1)) + 1, number, len(line) + 1, count, 0, 0, 0]]})
print(json.dumps({"data": [{"functions": functions}], "type": "llvm.coverage.json.export", "version": "2.0.1"}))
'''
# 自证用的假内存包装（排队那一格）：先像真包装排队时那样往 stderr 报一行，再等调用方把这一行转打出来（$QUEUE_LINE_SINK 里出现它，至多 10 秒）：
# 等到了往 stdout 打一行、stderr 再打一行命令自己的、退 0；等不到退 9（转打要在命令跑着的时候，不是跑完之后）
SELFTEST_QUEUEING_WRAPPER = '''#!/usr/bin/env bash
echo "run-with-memory-cap: 排队（已等 60 秒，至多 3600 秒）：自证造的一行" >&2
for _ in $(seq 1 200); do
  if grep -q '自证造的一行' "$QUEUE_LINE_SINK" 2>/dev/null; then echo "test result: ok"; echo "命令自己的 stderr" >&2; exit 0; fi
  sleep 0.05
done
exit 9
'''
FAKE_MEMORY_CAP_RUNNER = '''#!/usr/bin/env bash
# 自证用的假内存包装：--check 退 0；其余去掉上限那一个参数、原样执行命令
if [[ "${1:-}" == "--check" ]]; then exit 0; fi
shift
exec "$@"
'''
# 自证用的假内存包装（日常一条那两格）：照 run-with-memory-cap.sh 的「日常一条」演——环境里有 CLAUDE_CODE_SESSION_ID、没有 SINGLEFS_HEAVY_TESTS 的算日常；
# 账（$DAILY_LEDGER 目录，在跑的一条一个文件）上已有一条日常的在跑、这一条的 RUN_WITH_MEMORY_CAP_PARALLEL_REASON 不足 8 个字，就退 252、命令不跑。
# 每起一次往 $DAILY_WRAPPER_LOG 记一行「<并行理由>\t<CARGO_TARGET_DIR>」；命令跑完再在账上留 0.3 秒，几个工作进程才在账上重叠。--check 退 0。
SELFTEST_DAILY_WRAPPER = r'''#!/usr/bin/env bash
if [[ "${1:-}" == "--check" ]]; then exit 0; fi
shift
reason="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON-}"
printf '%s\t%s\n' "$reason" "${CARGO_TARGET_DIR-}" >> "$DAILY_WRAPPER_LOG"
entry=""
if [[ -n "${CLAUDE_CODE_SESSION_ID:-}" && -z "${SINGLEFS_HEAVY_TESTS:-}" ]]; then
  shopt -s nullglob
  exec 9>"$DAILY_LEDGER/.lock"
  flock 9
  others=("$DAILY_LEDGER"/entry.*)
  if (( ${#others[@]} > 0 && ${#reason} < 8 )); then
    exec 9>&-
    echo "  ✗ 这个会话（$CLAUDE_CODE_SESSION_ID）已经有一条日常的在跑，同一个会话同时只许一条；命令没跑（自证的假包装）" >&2
    echo "  → 怎么办：等那一条跑完再起（自证的假包装）" >&2
    exit 252
  fi
  entry="$DAILY_LEDGER/entry.$$"
  : > "$entry"
  exec 9>&-
fi
"$@"
status=$?
sleep 0.3
if [[ -n "$entry" ]]; then rm -f -- "$entry"; fi
exit "$status"
'''
SELFTEST_SOURCE = '''// 本模块自证的样本 crate（假 cargo 按下面几行判）
// TEST adds NEEDS left + right
// TEST doubles NEEDS value * 2
// TEST negates NEEDS 0 - value
// TEST halves NEEDS value / 2
pub fn add(left: u32, right: u32) -> u32 { left + right }
pub fn double(value: u32) -> u32 { value * 2 }
pub fn negate(value: i32) -> i32 { 0 - value }
pub fn halve(value: u32) -> u32 { value / 2 }
'''
SELFTEST_TABLE_ROWS = [
    "add-becomes-sub\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left * right }\t-p tiny --lib -- adds\tadds",
    "double-becomes-triple\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ value * 3 }\t-p tiny --lib -- doubles\tdoubles",
    "negate-becomes-identity\tcrates/tiny/src/lib.rs\t{ 0 - value }\t{ value }\t-p tiny --lib -- negates\tnegates",
    "halve-becomes-third\tcrates/tiny/src/lib.rs\t{ value / 2 }\t{ value / 3 }\t-p tiny --lib -- halves\thalves",
]
SELFTEST_READS_OUTSIDE_TEST = "// TEST reads_answer READS ../../fixture-data/answer.txt\n"
SELFTEST_READS_OUTSIDE_ROW = "reads-outside\tcrates/tiny/src/lib.rs\t{ 0 - value }\t{ 1 - value }\t-p tiny --lib -- reads_answer\treads_answer"
# 点名的测试照 GPU 对拍用例在没卡的机器上那样打一行 NOT_RUN_NO_GPU_ADAPTER 就返回成功：改坏的那一处它碰不到（文件头「本次未跑」）
SELFTEST_NOT_RUN_TEST = "// TEST gpu_parity NOTRUN 样本：本机没有 GPU 适配器\n"
SELFTEST_NOT_RUN_ROW = "card-path-changed\tcrates/tiny/src/lib.rs\t{ value / 2 }\t{ value / 4 }\t-p tiny --lib -- gpu_parity\tgpu_parity"
# 点名的测试要编 gpu 特性的一行（真表里 GPU 判器那几批）：假 cargo 要读得到 .cargo/config.toml 的 [env] BINDGEN_EXTRA_CLANG_ARGS（FAKE_CARGO 头上那一段）
SELFTEST_GPU_FEATURE_ROW = "gpu-double-becomes-triple\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ value * 3 }\t-p tiny --features gpu --lib -- doubles\tdoubles"
SELFTEST_CARGO_CONFIGURATION = '[env]\nBINDGEN_EXTRA_CLANG_ARGS = "-I/crates-mutation-rows-selftest/include"\n'
# 替换文空着的删行变异（真表里删掉一句调用、删掉 litmus 里一道屏障的那一类）
SELFTEST_DELETION_ROW = "halve-deleted\tcrates/tiny/src/lib.rs\t{ value / 2 }\t\t-p tiny --lib -- halves\thalves"
# 派活之前那几样各踩一样的行（接在 4 行正常变异与删行变异之后：变异表第 7–11 行），ROW 那一行改的 docs/outside.rs 另写进样本仓
SELFTEST_STATIC_PROBLEM_ROWS = [
    ("MALFORMED", "empty-expected\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left - right }\t-p tiny --lib -- adds\t"),
    ("ANCHOR", "anchor-missing\tcrates/tiny/src/lib.rs\t{ left / right }\t{ left }\t-p tiny --lib -- adds\tadds"),
    ("NAME", "name-anchored\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left + 1 }\t-p tiny --lib -- adds\tadds$"),
    ("ROW", "outside-copy\tdocs/outside.rs\tpub fn outside() {}\tpub fn outside() { }\t-p tiny --lib -- adds\tadds"),
    ("STRAY", "stray-escape\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ \\&value * 2 }\t-p tiny --lib -- doubles\tdoubles"),
]
# 覆盖那几格的样本 crate：helps 只执行 add（RUNS），点名它的那条变异改的是没执行的 helper；unused 谁都不执行；Pair 是函数外的类型定义
SELFTEST_COVERAGE_SOURCE = '''// 覆盖那几格的样本 crate（假 cargo 按下面几行判）
// TEST adds NEEDS left + right
// TEST doubles NEEDS value * 2
// TEST helps NEEDS value + 7 RUNS add
pub struct Pair { pub left: u32 }
pub fn add(left: u32, right: u32) -> u32 { left + right }
pub fn double(value: u32) -> u32 { value * 2 }
pub fn helper(value: u32) -> u32 { value + 7 }
pub fn unused(value: u32) -> u32 { value + 1 }
'''
# 两行同点 adds（同一组「cargo 参数 + 点名用例」，共用一份覆盖）
SELFTEST_COVERAGE_ROWS = [
    "add-becomes-mul\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left * right }\t-p tiny --lib -- adds\tadds",
    "add-becomes-sub\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left - right }\t-p tiny --lib -- adds\tadds",
    "double-becomes-triple\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ value * 3 }\t-p tiny --lib -- doubles\tdoubles",
    "helper-becomes-eight\tcrates/tiny/src/lib.rs\t{ value + 7 }\t{ value + 8 }\t-p tiny --lib -- helps\thelps",
]


def selftest_write(path, text, executable=False):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    if executable:
        os.chmod(path, 0o755)


def build_selftest_repository(directory, table_rows, extra_source="", source=SELFTEST_SOURCE):
    """一棵临时 git 仓：Cargo.toml、crates/tiny、变异表、仓根下拷贝范围之外的 fixture-data/answer.txt。"""
    selftest_write(os.path.join(directory, "Cargo.toml"), '[workspace]\nmembers = ["crates/tiny"]\n')
    selftest_write(os.path.join(directory, "crates", "tiny", "Cargo.toml"), '[package]\nname = "tiny"\nversion = "0.1.0"\n')
    selftest_write(os.path.join(directory, "crates", "tiny", "src", "lib.rs"), source + extra_source)
    selftest_write(os.path.join(directory, TABLE), "# 变异名\t文件\t原文\t替换文\tcargo test 参数\t必须红的测试名\n" + "".join(row + "\n" for row in table_rows))
    selftest_write(os.path.join(directory, "fixture-data", "answer.txt"), "42\n")
    subprocess.run(["git", "init", "-q", directory], check=True)


class Selftest:
    def __init__(self, work):
        self.work = work
        self.cells = 0
        self.failures = []
        self.bin_directory = os.path.join(work, "bin")
        selftest_write(os.path.join(self.bin_directory, "cargo"), FAKE_CARGO, executable=True)
        self.llvm_directory = os.path.join(work, "llvm")
        for tool in LLVM_TOOLS:
            selftest_write(os.path.join(self.llvm_directory, tool), FAKE_LLVM_TOOL, executable=True)
        self.runner = os.path.join(work, "fake-memory-cap.sh")
        selftest_write(self.runner, FAKE_MEMORY_CAP_RUNNER, executable=True)
        self.log = os.path.join(work, "cargo.log")
        # BINDGEN_EXTRA_CLANG_ARGS 也不带进来：调用方 export 了它，副本里没有 .cargo/ 照样编得过，「拷贝范围含 .cargo/」那一格就红不出来；
        # RUSTFLAGS 一类也不带：假 cargo 不看它，带进来只会让判法版本随调用方变
        self.base_environment = {key: value for key, value in os.environ.items()
                                 if key not in STAGE_VARIABLES
                                 and key not in ("BINDGEN_EXTRA_CLANG_ARGS", "FAKE_CARGO_VERSION", "FAKE_LLVM_VERSION")}
        self.base_environment.update(PATH=self.bin_directory + os.pathsep + os.environ.get("PATH", ""), FAKE_CARGO_LOG=self.log,
                                     GATE_MUTATION_MEMORY_CAP_RUNNER=self.runner, GATE_MUTATION_TARGET_DIR=os.path.join(work, "target"),
                                     GATE_MUTATION_WORKERS="1", GATE_MUTATION_LLVM_TOOLS=self.llvm_directory)

    def run(self, root, **changes):
        """在 root 上跑一趟 run_stage：(退出码, stdout, stderr, 这一趟假 cargo 起的每一行)。"""
        with contextlib.suppress(FileNotFoundError):
            os.unlink(self.log)
        environment = dict(self.base_environment, **changes)
        stdout, stderr = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            exit_code = run_stage(root, environment)
        invocations = []
        if os.path.exists(self.log):
            with open(self.log, encoding="utf-8") as handle:
                invocations = handle.read().splitlines()
        return exit_code, stdout.getvalue(), stderr.getvalue(), invocations

    def check(self, label, condition, detail=""):
        self.cells += 1
        if not condition:
            self.failures.append(f"{label}{('：' + detail) if detail else ''}")


def plain_invocations(invocations):
    """假 cargo 的记录里基线与变异那几次（记覆盖的 --no-run 与 --exact 两次不算）。"""
    return [line for line in invocations if "--no-run" not in line and "--exact" not in line]


def coverage_invocations(invocations):
    return [line for line in invocations if "--no-run" in line or "--exact" in line]


def mutation_invocations(invocations, pristine_hash):
    """假 cargo 的记录里改坏了源码的那几次（lib.rs 的哈希不是没改坏的那一份）。"""
    return [line for line in invocations if not line.endswith("\t" + pristine_hash)]


def selftest_verdict_view(stdout):
    """stdout 去掉随记录变的那几处（「复用：」「覆盖：」两行、抓到那一行末尾的「（现判：…）」「（按覆盖复用）」、成功句末尾「；这一趟现判 …」），剩下的是判定本身。"""
    lines = []
    for line in stdout.splitlines():
        if line.startswith("  复用：") or line.startswith("  覆盖："):
            continue
        line = re.sub(r"（现判：[^）]*）$|（按覆盖复用）$", "", line)
        lines.append(re.sub(r"；这一趟现判 .*$", "", line))
    return "\n".join(lines)


def source_hash(root):
    with open(os.path.join(root, "crates", "tiny", "src", "lib.rs"), encoding="utf-8") as handle:
        return hashlib.sha256(handle.read().encode()).hexdigest()[:12]


def replace_in_file(path, old, new):
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    assert text.count(old) == 1, (path, old)
    selftest_write(path, text.replace(old, new))


def per_row_how(stdout):
    """stdout 里抓到那几行：{变异名: 「现判：<栏>」或「按覆盖复用」}。"""
    found = {}
    for match in re.finditer(r"^  ✓ (\S+)：\S+ 红了（(现判：[^）]*|按覆盖复用)）$", stdout, re.M):
        found[match.group(1)] = match.group(2)
    return found


def selftest_queue_line_cells(selftest, work):
    """包装排队报的那一行边跑边转打到 stderr（带是哪一行）；收进来的 stdout、stderr 与 capture_output 收的逐字相同、stdout 不多一行。"""
    wrapper = os.path.join(work, "queueing-wrapper.sh")
    selftest_write(wrapper, SELFTEST_QUEUEING_WRAPPER, executable=True)
    sink_path = os.path.join(work, "queue-line-sink.txt")
    with open(sink_path, "w", encoding="utf-8", buffering=1) as sink:
        run = run_forwarding_queue_lines(["bash", wrapper], "变异表第 7 行", sink=sink, env=dict(os.environ, QUEUE_LINE_SINK=sink_path))
    with open(sink_path, encoding="utf-8") as handle:
        forwarded = handle.read()
    selftest.check("包装排队报的那一行边跑边转打到 stderr（带是哪一行），收进来判结局的输出不变",
                   run.returncode == 0 and run.stdout == "test result: ok\n"
                   and run.stderr == "run-with-memory-cap: 排队（已等 60 秒，至多 3600 秒）：自证造的一行\n命令自己的 stderr\n"
                   and forwarded == "  … 变异表第 7 行：run-with-memory-cap: 排队（已等 60 秒，至多 3600 秒）：自证造的一行\n",
                   f"退 {run.returncode}（9 是假包装等了 10 秒没见到转打）；stdout {run.stdout!r}；stderr {run.stderr!r}；转打出来的 {forwarded!r}")


def selftest_worker_count_cells(selftest, work):
    """工作进程数（文件头「工作进程数」）：核数、要现跑的条数、外层内存上限 ÷ 每条上限三项取最小；外层上限读不到时没有第三项、头一行写明。"""
    sixteen = dict(selftest.base_environment, GATE_MUTATION_WORKERS="16")   # 弄坏开关随 base_environment 带进来
    wide, head = worker_count_for(20, dict(sixteen, **{OUTER_MEMORY_CAP_VARIABLE: "38G"}), "4G")
    narrow, narrow_head = worker_count_for(20, dict(sixteen, **{OUTER_MEMORY_CAP_VARIABLE: "40G"}), "6G")
    selftest.check("工作进程数不超过外层内存上限 ÷ 每条变异的上限（38G ÷ 4G 开 9 个、40G ÷ 6G 开 6 个，都不是 GATE_MUTATION_WORKERS 给的 16）",
                   wide == 9 and "卡在「外层内存上限」这一项" in head and narrow == 6 and "卡在「外层内存上限」这一项" in narrow_head, f"{wide}：{head}；{narrow}：{narrow_head}")
    roomy, roomy_head = worker_count_for(5, dict(sixteen, **{OUTER_MEMORY_CAP_VARIABLE: "1T"}), "4G")
    selftest.check("外层内存上限宽裕时仍卡在别的项（1T ÷ 4G 远多于要现跑的 5 条：开 5 个、卡在条数）",
                   roomy == 5 and "卡在「这一趟要现跑的条数」这一项" in roomy_head and "外层内存上限没读到" not in roomy_head, f"{roomy}：{roomy_head}")
    squeezed, squeezed_head = worker_count_for(20, dict(sixteen, **{OUTER_MEMORY_CAP_VARIABLE: "3G"}), "4G")
    selftest.check("外层上限不够一条的上限（3G ÷ 4G）时按 1 个开、头一行写那一项不足 1", squeezed == 1 and "那一项不足 1，按 1 个开" in squeezed_head, f"{squeezed}：{squeezed_head}")
    unset, unset_head = worker_count_for(20, dict(sixteen), "4G")
    garbled, garbled_head = worker_count_for(20, dict(sixteen, **{OUTER_MEMORY_CAP_VARIABLE: "sixteen"}), "4G")
    selftest.check("外层内存上限没设、写法不认时没有第三项（照 16 个开），头一行写明没读到与原因",
                   unset == 16 and garbled == 16 and "外层内存上限没读到（RUN_WITH_MEMORY_CAP_OUTER_CAP 没设" in unset_head
                   and "写成「sixteen」，不是正整数加 K / M / G / T" in garbled_head, f"{unset}：{unset_head}；{garbled}：{garbled_head}")
    repository = os.path.join(work, "worker-count")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    outer = selftest.run(repository, GATE_MUTATION_WORKERS="16", GATE_MUTATION_MEMORY_MAX="4G", GATE_MUTATION_START_OVER="1", **{OUTER_MEMORY_CAP_VARIABLE: "8G"})
    selftest.check("外层内存上限交给 run_stage：开的工作进程数跟着收（8G ÷ 4G 开 2 个，不是给的 16 个也不是条数 4）",
                   outer[0] == 0 and "这一轮开 2 个工作进程" in outer[2] and "卡在「外层内存上限」这一项" in outer[2], f"退 {outer[0]}；{outer[2][:600]}")


def selftest_gpu_memory_cell(selftest, work):
    """内存包装退 245（显存不够）：判 gpumemory、不算抓到——输出里点名的测试 FAILED 是被显存不够弄红的。"""
    row = Row(7, "selftest-gpu-memory", "crates/selftest/src/lib.rs", "a", "b", "-p selftest", "tests::named", [])
    output = "running 1 test\ntest tests::named ... FAILED\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored\n"
    verdict, text = verdict_of_run(row, GPU_MEMORY_HIT_EXIT, output, "4G", "1800", environment=os.environ)
    selftest.check("包装退 245（显存不够）记显存不够、不算抓到", verdict == "gpumemory" and "显存不够" in text, f"判成 {verdict}：{text[:200]}")


def selftest_libtest_result_cell(selftest, work):
    """libtest 结果行经 lib_libtest_result_lines.py：should panic 的用例红了判抓到；点名省掉模块路径时只对上全名以「::<点名>」收尾的那一个。"""
    row = Row(8, "selftest-should-panic", "crates/selftest/src/lib.rs", "a", "b", "-p selftest --lib", "answer_panics", [])
    output = ("running 2 tests\ntest tests::answer_is_42 ... ok\ntest tests::answer_panics - should panic ... FAILED\n\n"
              "test result: FAILED. 1 passed; 1 failed; 0 ignored\n\nerror: test failed, to rerun pass `-p selftest --lib`\n")
    verdict, text = verdict_of_run(row, 101, output, "4G", "1800", environment=os.environ)
    results = results_by_test("answer_panics", output + "test other::not_answer_panics ... ok\n")
    selftest.check("should panic 的用例红了判抓到、点名省掉模块路径只对上那一个",
                   verdict == "caught" and results == {"tests::answer_panics": ["FAILED"]}, f"判成 {verdict}：{text[:200]}；对上的 {results}")


# 自证「被 TERM 停」那一格在子进程里跑 run_stage（按路径载入本模块）
SELFTEST_STAGE_CHILD = ("import importlib.util, os, sys\n"
                        "specification = importlib.util.spec_from_file_location('crates_mutation_rows_selftest_child', sys.argv[1])\n"
                        "module = importlib.util.module_from_spec(specification)\n"
                        "sys.modules[specification.name] = module\n"
                        "specification.loader.exec_module(module)\n"
                        "sys.exit(module.run_stage(sys.argv[2], dict(os.environ)))\n")


def selftest_signal_and_pipe_cells(selftest, work):
    """被 TERM 停时（双机分片的驱动停这一份）$TMPDIR 下的 singlefs-mutation-replay-* 删掉、退 143；--list-items 接 | head 时照 SIGPIPE 惯例安静退。
    子进程只对自证自己起的那一个发信号（Popen 的 send_signal）。"""
    repository = os.path.join(work, "signalled")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    temporary = os.path.join(work, "signalled-tmp")
    os.makedirs(temporary)
    hang, released, done = (os.path.join(work, name) for name in ("cargo-hanging", "cargo-released", "cargo-done"))
    environment = dict(os.environ, TMPDIR=temporary, GATE_MUTATION_START_OVER="1", FAKE_CARGO_HANG_FILE=hang, FAKE_CARGO_RELEASE_FILE=released,
                       FAKE_CARGO_DONE_FILE=done)
    stderr_path = os.path.join(work, "signalled-stderr.txt")
    # stderr 落文件、不接管道：没接住信号时工作进程成了孤儿、还开着它，接管道就等不到头
    with open(stderr_path, "w", encoding="utf-8") as stderr_handle:
        child = subprocess.Popen([sys.executable, "-c", SELFTEST_STAGE_CHILD, MODULE_PATH, repository], env=environment,
                                 stdout=subprocess.DEVNULL, stderr=stderr_handle)
    deadline = time.monotonic() + 60
    while not os.path.exists(hang) and child.poll() is None and time.monotonic() < deadline:
        time.sleep(0.05)
    while_running = sorted(name for name in os.listdir(temporary) if name.startswith("singlefs-mutation-replay-"))
    if child.poll() is None:
        child.send_signal(signal.SIGTERM)
    try:
        child.wait(timeout=60)
    except subprocess.TimeoutExpired:
        child.kill()
        child.wait()
    open(released, "w").close()
    deadline = time.monotonic() + 35
    while os.path.exists(hang) and not os.path.exists(done) and time.monotonic() < deadline:
        time.sleep(0.05)
    # 挂住的假 cargo 记下的父进程就是那个工作进程：照理已经被停了；还活着（没接住信号的样子）就只对这一个进程号发 TERM，不留孤儿
    worker = int(open(hang, encoding="utf-8").read() or 0) if os.path.exists(hang) else 0
    worker_alive = bool(worker) and os.path.exists(f"/proc/{worker}")
    if worker_alive:
        with contextlib.suppress(OSError):
            os.kill(worker, signal.SIGTERM)
    left = sorted(name for name in os.listdir(temporary) if name.startswith("singlefs-mutation-replay-"))
    stderr = open(stderr_path, encoding="utf-8", errors="replace").read()
    selftest.check("被 TERM 停（双机分片的驱动停这一份的样子）：$TMPDIR 下的 singlefs-mutation-replay-* 跑的时候在、停完删掉，退 143，工作进程停了，挂住的假 cargo 也退了",
                   len(while_running) == 1 and left == [] and child.returncode == 128 + signal.SIGTERM and not worker_alive and os.path.exists(done),
                   f"跑的时候 {while_running}，停完 {left}，退 {child.returncode}，工作进程 {worker} 还活着 {worker_alive}，假 cargo 退了 {os.path.exists(done)}；"
                   f"stderr 尾：{stderr.strip()[-300:]}")
    read_end, write_end = os.pipe()
    os.close(read_end)
    closed_pipe = subprocess.run([sys.executable, MODULE_PATH, "--list-items", repository], stdout=write_end, stderr=subprocess.PIPE, text=True,
                                 errors="replace", env=dict(os.environ))
    os.close(write_end)
    selftest.check("--list-items 接 | head（读的一头先关了）：被 SIGPIPE 安静停掉，stderr 一个字都没有",
                   closed_pipe.returncode == -signal.SIGPIPE and closed_pipe.stderr == "", f"退 {closed_pipe.returncode}：{closed_pipe.stderr.strip()[-300:]!r}")


def selftest_cargo_configuration_cell(selftest, work):
    """拷贝范围含 .cargo/（拷贝范围取真登记表里本阶段那一行）：点名的测试要编 gpu 特性的那一行，副本里的假 cargo 照真 cargo 的找法读得到
    .cargo/config.toml 的 [env] BINDGEN_EXTRA_CLANG_ARGS，基线绿、变异抓到；读不到就照 librocksdb-sys 的构建脚本退 101，记「基线不绿」。"""
    repository = os.path.join(work, "cargo-configuration")
    build_selftest_repository(repository, [SELFTEST_TABLE_ROWS[0], SELFTEST_GPU_FEATURE_ROW])
    selftest_write(os.path.join(repository, CARGO_CONFIGURATION_DIRECTORY, "config.toml"), SELFTEST_CARGO_CONFIGURATION)
    exit_code, stdout, stderr, invocations = selftest.run(repository, GATE_MUTATION_START_OVER="1")
    feature_runs = [line for line in plain_invocations(invocations) if "--features gpu" in line]
    selftest.check("拷贝范围含 .cargo/：点名的测试要编 gpu 特性的那一行基线绿、变异抓到",
                   exit_code == 0 and "基线不绿 0 行" in stdout and "计数：抓到 2 条、没红 0 条" in stdout and len(feature_runs) == 2,
                   f"退 {exit_code}；假 cargo 带 --features gpu 起了 {len(feature_runs)} 次（要基线 1 次、变异 1 次）；拷贝范围 {copied_paths()}；{stdout}{stderr[-400:]}")


PARALLEL_REASON_FORM = re.compile(r"^同一次 checker-tier-crates-mutation-replay（本模块进程 \d+）的第 (\d+) 个工作进程，各用各的源码副本与编译目录（这一轮共 4 个）$")


def selftest_daily_cells(selftest, work):
    """会话里不带 SINGLEFS_HEAVY_TESTS 平常跑（假包装照「日常一条」演）：4 个工作进程照样各条判对，每次包装带自己那个工作进程的并行理由
    （理由里的片号与那一次的编译目录对得上，记覆盖那几次的编译目录多一段 -coverage）；调用方设了的并行理由原样传、不覆盖。"""
    wrapper = os.path.join(work, "daily-memory-cap.sh")
    selftest_write(wrapper, SELFTEST_DAILY_WRAPPER, executable=True)
    ledger, wrapper_log = os.path.join(work, "daily-ledger"), os.path.join(work, "daily-wrapper.log")
    os.makedirs(ledger)
    repository = os.path.join(work, "daily")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    shared_target = selftest.base_environment["GATE_MUTATION_TARGET_DIR"]
    daily = dict(GATE_MUTATION_MEMORY_CAP_RUNNER=wrapper, GATE_MUTATION_WORKERS="4", GATE_MUTATION_START_OVER="1",
                 CLAUDE_CODE_SESSION_ID="crates-mutation-rows-selftest-session", SINGLEFS_HEAVY_TESTS="", RUN_WITH_MEMORY_CAP_PARALLEL_REASON="",
                 DAILY_LEDGER=ledger, DAILY_WRAPPER_LOG=wrapper_log)

    def wrapper_lines():
        with open(wrapper_log, encoding="utf-8") as handle:
            lines = [line.split("\t") for line in handle.read().splitlines()]
        os.unlink(wrapper_log)
        return lines

    plain = selftest.run(repository, **daily)
    seen = wrapper_lines() if os.path.exists(wrapper_log) else []
    mismatched = []
    for reason, target_directory in seen:
        form = PARALLEL_REASON_FORM.match(reason)
        directory = target_directory[:-len(COVERAGE_TARGET_SUFFIX)] if target_directory.endswith(COVERAGE_TARGET_SUFFIX) else target_directory
        if form is None or directory != (shared_target if form.group(1) == "1" else f"{shared_target}-w{form.group(1)}"):
            mismatched.append(f"理由「{reason}」、编译目录 {target_directory}")
    judged = [line for line in seen if not line[1].endswith(COVERAGE_TARGET_SUFFIX)]
    selftest.check("会话里平常跑（假包装照日常一条演）、4 个工作进程：各条照样判对，每次包装带自己那个工作进程的并行理由",
                   plain[0] == 0 and "计数：抓到 4 条、没红 0 条" in plain[1] and "排不上没跑 0 条" in plain[1] and len(judged) == 5
                   and len(seen) == 5 + 2 * 4 and not mismatched,
                   f"退 {plain[0]}；包装起了 {len(seen)} 次（要 1 批基线 + 4 条变异 + 4 组覆盖各 2 次）；理由不对的 {mismatched}；{plain[1]}")
    caller_reason = "调用方给的并行理由（自证）"
    passed_through = selftest.run(repository, **dict(daily, RUN_WITH_MEMORY_CAP_PARALLEL_REASON=caller_reason))
    seen = wrapper_lines() if os.path.exists(wrapper_log) else []
    selftest.check("调用方设了的并行理由原样传给每次包装、不覆盖",
                   passed_through[0] == 0 and len(seen) == 5 and all(reason == caller_reason for reason, _target in seen),
                   f"退 {passed_through[0]}；包装看到的理由 {sorted({reason for reason, _target in seen})}")


def selftest_target_directory_cells(selftest, work):
    """编译目录（文件头「编译目录」）：片号大于这一趟工作进程数的 -w<片号> 连同 -coverage 开跑时回收、有进程在用的不删、两个变量都没设时不落 /tmp。"""
    repository = os.path.join(work, "target-reclaim")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)

    def seed(directory, names):
        for name in names:
            selftest_write(os.path.join(directory, name, "debug", "deps", "libold.rlib"), "x" * 8192)

    # 一：2 个工作进程。-w3、-w3-coverage、-w5 回收；第 1 片、它的 -coverage、-w2、-w2-coverage 留着；只是名字像的不碰
    targets = os.path.join(work, "reclaim-targets")
    shared = os.path.join(targets, TARGET_DIRECTORY_NAME)
    kept_names = [TARGET_DIRECTORY_NAME, TARGET_DIRECTORY_NAME + COVERAGE_TARGET_SUFFIX, TARGET_DIRECTORY_NAME + "-w2",
                  TARGET_DIRECTORY_NAME + "-w2" + COVERAGE_TARGET_SUFFIX, TARGET_DIRECTORY_NAME + "-w2x", "another-target-w9"]
    seed(targets, kept_names + [TARGET_DIRECTORY_NAME + "-w3", TARGET_DIRECTORY_NAME + "-w3" + COVERAGE_TARGET_SUFFIX, TARGET_DIRECTORY_NAME + "-w5"])
    exit_code, _stdout, stderr, _ = selftest.run(repository, GATE_MUTATION_TARGET_DIR=shared, GATE_MUTATION_WORKERS="2", GATE_MUTATION_START_OVER="1")
    present = sorted(name for name in os.listdir(targets) if not name.endswith(WORKER_SLOT_LOCK_SUFFIX))   # 片号锁文件（.slot）是领片号留下的，不是编译目录
    reclaim_lines = [line for line in stderr.splitlines() if "编译目录：" in line]
    selftest.check("片号大于这一趟工作进程数的 -w<片号> 连同 -coverage 开跑时回收、不大于的留着、打一行删了几份多大",
                   exit_code == 0 and present == sorted(kept_names) and any("回收了片号大于这一趟 2 个工作进程的 3 份，共 " in line for line in reclaim_lines),
                   f"退 {exit_code}；剩下 {present}（要 {sorted(kept_names)}）；{reclaim_lines}")
    # 二：多出来的四份各有人在用（工作目录、命令行、CARGO_TARGET_DIR、在用标记的共享锁），一份没人用：只删没人用的那份
    targets = os.path.join(work, "reclaim-in-use")
    shared = os.path.join(targets, TARGET_DIRECTORY_NAME)
    used = {kind: f"{TARGET_DIRECTORY_NAME}-w{sequence}" for kind, sequence in (("cwd", 4), ("cmdline", 6), ("variable", 7), ("lock", 8))}
    seed(targets, [TARGET_DIRECTORY_NAME, *used.values(), TARGET_DIRECTORY_NAME + "-w9"])
    lock_ready = os.path.join(work, "reclaim-lock-ready")
    sleeper = "import time; time.sleep(120)"
    lock_holder = ("import fcntl, os, time\nhandle = open(os.environ['SELFTEST_LOCK_PATH'], 'a')\nfcntl.flock(handle, fcntl.LOCK_SH)\n"
                   "open(os.environ['SELFTEST_LOCK_READY'], 'w').close()\ntime.sleep(120)\n")
    plain_environment = {key: value for key, value in os.environ.items() if key != "CARGO_TARGET_DIR"}
    holders = []
    try:
        holders.append(subprocess.Popen([sys.executable, "-c", sleeper], cwd=os.path.join(targets, used["cwd"]), env=plain_environment))
        holders.append(subprocess.Popen([sys.executable, "-c", sleeper, os.path.join(targets, used["cmdline"], "debug")], cwd=work, env=plain_environment))
        holders.append(subprocess.Popen([sys.executable, "-c", sleeper], cwd=work,
                                        env=dict(plain_environment, CARGO_TARGET_DIR=os.path.join(targets, used["variable"]))))
        holders.append(subprocess.Popen([sys.executable, "-c", lock_holder], cwd=work,
                                        env=dict(plain_environment, SELFTEST_LOCK_PATH=os.path.join(targets, used["lock"], WORKER_TARGET_LOCK_NAME),
                                                 SELFTEST_LOCK_READY=lock_ready)))
        deadline = time.time() + 30
        while not os.path.exists(lock_ready) and time.time() < deadline:
            time.sleep(0.05)
        exit_code, _stdout, stderr, _ = selftest.run(repository, GATE_MUTATION_TARGET_DIR=shared, GATE_MUTATION_WORKERS="2", GATE_MUTATION_START_OVER="1")
    finally:
        for holder in holders:
            holder.kill()
            holder.wait()
    present = set(os.listdir(targets))
    reclaim_lines = [line for line in stderr.splitlines() if "编译目录：" in line or line.startswith("    没删 ")]
    selftest.check("多出来的 -w<片号> 有进程在用的不删（工作目录、命令行、CARGO_TARGET_DIR、在用标记各一份），没人用的照删",
                   exit_code == 0 and set(used.values()) <= present and TARGET_DIRECTORY_NAME + "-w9" not in present
                   and any("有进程在用、没删 4 份" in line for line in reclaim_lines),
                   f"退 {exit_code}；剩下 {sorted(present)}；在用的四份 {sorted(used.values())}；{reclaim_lines}")
    # 三：GATE_MUTATION_TARGET_DIR 与 GATE_CROSS_RUN_TMPDIR 都没设：落在跨轮缓存家目录下，不落 /tmp；设了 GATE_CROSS_RUN_TMPDIR 放它底下
    # HOME 与 TMPDIR 都换成临时目录里的一份（弄坏开关退回 $TMPDIR 时也不往真的 /tmp 里建）；自证的临时目录本身可能在 /tmp 下，
    # 所以判的是「不在 $TMPDIR 底下、也不是 /tmp 底下那一份」；rustc -V 要的 rustup 家目录照原样指回去
    home, temporary = os.path.join(work, "home"), os.path.join(work, "plain-tmp")
    os.makedirs(temporary, exist_ok=True)
    unset = dict(GATE_MUTATION_TARGET_DIR="", GATE_CROSS_RUN_TMPDIR="", HOME=home, TMPDIR=temporary, GATE_MUTATION_WORKERS="1", GATE_MUTATION_START_OVER="1",
                 RUSTUP_HOME=os.environ.get("RUSTUP_HOME") or os.path.expanduser("~/.rustup"),
                 CARGO_HOME=os.environ.get("CARGO_HOME") or os.path.expanduser("~/.cargo"))
    unset[CROSS_RUN_CACHE_HOME_VARIABLE] = ""
    wanted = os.path.join(home, ".local", "share", "singlefs", "cross-run-cache", TARGET_DIRECTORY_NAME)
    chosen = default_shared_target_directory(dict(selftest.base_environment, **unset))
    under_cross_run = default_shared_target_directory(dict(selftest.base_environment, **dict(unset, GATE_CROSS_RUN_TMPDIR=os.path.join(work, "cross-run"))))
    exit_code, _stdout, stderr, _ = selftest.run(repository, **unset)
    selftest.check("两个变量都没设时编译目录不落 /tmp、落在跨轮缓存家目录下（设了 GATE_CROSS_RUN_TMPDIR 放它底下）",
                   chosen == wanted and not chosen.startswith(temporary + os.sep) and chosen != os.path.join("/tmp", TARGET_DIRECTORY_NAME)
                   and under_cross_run == os.path.join(work, "cross-run", TARGET_DIRECTORY_NAME)
                   and exit_code == 0 and f"编译目录：{wanted}（" in stderr and os.path.isdir(wanted),
                   f"选的是 {chosen}（要 {wanted}）；设了 GATE_CROSS_RUN_TMPDIR 时 {under_cross_run}；退 {exit_code}；"
                   f"{[line for line in stderr.splitlines() if '编译目录：' in line]}")


SELFTEST_OTHER_BATCH_ROW = "double-becomes-triple\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ value * 3 }\t-p tiny --lib --quiet -- doubles\tdoubles"


def selftest_worker_slot_cells(selftest, work):
    """片号跨调用经锁领（文件头「编译目录」）：同一个共用编译目录上并发的两次调用各起 2 个工作进程（假 cargo 挂着不放，两次的工作进程都活着），
    两次领到的片号集合不相交，四把片号锁都被占，放掉之后锁都空了；片号锁被占着的多出来的 -w<片号> 回收时留着。
    两次调用各是一个子进程（stderr 落文件，片号从工作进程报的「领到片号 <n>」那一行读），基线的批分成两批、两个工作进程才都起得来。"""
    shared = os.path.join(work, "slot-targets", TARGET_DIRECTORY_NAME)
    released = os.path.join(work, "slot-cargo-released")
    invocations = {}
    for name in ("earlier", "later"):
        repository = os.path.join(work, f"slots-{name}")
        build_selftest_repository(repository, [SELFTEST_TABLE_ROWS[0], SELFTEST_OTHER_BATCH_ROW])
        temporary = os.path.join(work, f"slots-{name}-tmp")
        os.makedirs(temporary)
        environment = dict(selftest.base_environment, GATE_MUTATION_TARGET_DIR=shared, GATE_MUTATION_WORKERS="2", GATE_MUTATION_START_OVER="1", TMPDIR=temporary,
                           FAKE_CARGO_LOG=os.path.join(work, f"slots-{name}-cargo.log"), FAKE_CARGO_HANG_FILE=os.path.join(work, f"slots-{name}-hanging"),
                           FAKE_CARGO_RELEASE_FILE=released, FAKE_CARGO_DONE_FILE=os.path.join(work, f"slots-{name}-done"))
        invocations[name] = dict(repository=repository, environment=environment, stderr_path=os.path.join(work, f"slots-{name}-stderr.txt"))

    def slots_reported(name):
        with open(invocations[name]["stderr_path"], encoding="utf-8", errors="replace") as handle:
            return sorted(int(number) for number in re.findall(r"领到片号 (\d+)（", handle.read()))

    def start(name):
        with open(invocations[name]["stderr_path"], "w", encoding="utf-8") as stderr_handle:
            invocations[name]["process"] = subprocess.Popen([sys.executable, "-c", SELFTEST_STAGE_CHILD, MODULE_PATH, invocations[name]["repository"]],
                                                            env=invocations[name]["environment"], stdout=subprocess.DEVNULL, stderr=stderr_handle)
        deadline = time.monotonic() + 90
        while len(slots_reported(name)) < 2 and invocations[name]["process"].poll() is None and time.monotonic() < deadline:
            time.sleep(0.05)
        return slots_reported(name)

    earlier_slots = start("earlier")
    later_slots = start("later")
    locks_while_running = {slot: file_lock_is_held(slot_lock_path(shared, slot)) for slot in sorted({*earlier_slots, *later_slots})}
    held_while_running = sorted(slot for slot, (held, handle) in locks_while_running.items() if held)
    for _held, handle in locks_while_running.values():
        if handle is not None:
            handle.close()
    open(released, "w").close()

    def wait_for_own_selftest_child(process):
        """等自证自己起的那一个子进程退出；90 秒还没退就停掉它（只停这一个，按进程对象，不按模式找）。"""
        try:
            process.wait(timeout=90)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()

    for name in invocations:
        wait_for_own_selftest_child(invocations[name]["process"])
    free_afterwards = []
    for slot in sorted({*earlier_slots, *later_slots}):
        held, handle = file_lock_is_held(slot_lock_path(shared, slot))
        free_afterwards.append(not held)
        if handle is not None:
            handle.close()
    selftest.check("同一个共用编译目录上并发的两次调用各起 2 个工作进程：领到的片号集合不相交、四把片号锁都被占、两次退出之后锁都空了",
                   len(earlier_slots) == 2 and len(later_slots) == 2 and not set(earlier_slots) & set(later_slots)
                   and held_while_running == sorted({*earlier_slots, *later_slots}) and len(held_while_running) == 4 and all(free_afterwards),
                   f"先起的那次领到 {earlier_slots}、后起的那次领到 {later_slots}；跑的时候被占的锁 {held_while_running}；退出之后锁空了 {free_afterwards}")
    # 回收：多出来的 -w<片号> 的片号锁被别的调用的工作进程拿着（它刚领到这一片、还没来得及拿在用标记）时留着，没人拿的照删
    targets = os.path.join(work, "slot-reclaim")
    reclaim_shared = os.path.join(targets, TARGET_DIRECTORY_NAME)
    for name in (TARGET_DIRECTORY_NAME, TARGET_DIRECTORY_NAME + "-w3", TARGET_DIRECTORY_NAME + "-w4"):
        selftest_write(os.path.join(targets, name, "debug", "deps", "libold.rlib"), "x" * 4096)
    import fcntl
    holder = open(slot_lock_path(reclaim_shared, 3), "a")
    try:
        fcntl.flock(holder, fcntl.LOCK_EX | fcntl.LOCK_NB)
        removed, kept, failed = reclaim_extra_target_directories(reclaim_shared, 2, selftest.base_environment)
    finally:
        holder.close()
    selftest.check("多出来的 -w<片号> 的片号锁被别的进程拿着的不删（没有进程的工作目录、命令行、环境指到它），没人拿的照删",
                   [os.path.basename(path) for path, _size in removed] == [TARGET_DIRECTORY_NAME + "-w4"]
                   and [os.path.basename(path) for path, _reason in kept] == [TARGET_DIRECTORY_NAME + "-w3"] and "片号锁" in kept[0][1] and not failed
                   and os.path.isdir(os.path.join(targets, TARGET_DIRECTORY_NAME + "-w3")),
                   f"删了 {removed}；留着 {kept}；删不成 {failed}")


def selftest_crash_library_home_cells(selftest, work):
    """变异复跑起的 cargo 写自己的崩溃判定库（文件头「变异复跑的判定库」）：调用方的家目录是 <甲>、又点名了单份库时，每次起的 cargo 收到的
    SINGLEFS_CRASH_AMPLIFICATION_HOME 都是 <甲>/mutation-replay（不等于真跑的 <甲>）、收不到 SINGLEFS_CRASH_AMPLIFICATION_LIBRARY；设了 GATE_MUTATION_CRASH_LIBRARY_HOME 收到的是它。"""
    repository = os.path.join(work, "crash-library-home")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    environment_log = os.path.join(work, "crash-library-home-environment.jsonl")
    real_home = os.path.join(work, "real-crash-library-home")

    def received(**changes):
        with contextlib.suppress(FileNotFoundError):
            os.unlink(environment_log)
        exit_code, _stdout, stderr, _ = selftest.run(repository, GATE_MUTATION_START_OVER="1", FAKE_CARGO_ENVIRONMENT_LOG=environment_log,
                                                     SINGLEFS_CRASH_AMPLIFICATION_HOME=real_home,
                                                     SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=os.path.join(work, "named-crash-library"), **changes)
        records = []
        if os.path.exists(environment_log):
            with open(environment_log, encoding="utf-8") as handle:
                records = [json.loads(line) for line in handle.read().splitlines() if line.strip()]
        homes = sorted({str(record.get(CRASH_LIBRARY_HOME_VARIABLE)) for record in records})
        libraries = sorted({str(record.get(CRASH_LIBRARY_DIRECTORY_VARIABLE)) for record in records})
        return exit_code, records, homes, libraries, stderr

    wanted = os.path.join(real_home, MUTATION_CRASH_LIBRARY_SUBDIRECTORY)
    exit_code, records, homes, libraries, stderr = received()
    selftest.check("变异复跑起的 cargo 写自己的崩溃判定库：家目录是真跑那一份底下的 mutation-replay/、不等于真跑的，调用方点名的单份库清掉",
                   exit_code == 0 and len(records) >= 2 and homes == [wanted] and wanted != real_home and libraries == ["None"],
                   f"退 {exit_code}；假 cargo 起了 {len(records)} 次，收到的家目录 {homes}（要 [{wanted}]，真跑的是 {real_home}）、单份库 {libraries}（要都没有）；{stderr[-400:]}")
    configured = os.path.join(work, "configured-mutation-crash-library-home")
    exit_code, records, homes, libraries, stderr = received(GATE_MUTATION_CRASH_LIBRARY_HOME=configured)
    selftest.check("设了 GATE_MUTATION_CRASH_LIBRARY_HOME：变异复跑起的 cargo 收到的家目录就是它、单份库照样清掉",
                   exit_code == 0 and len(records) >= 2 and homes == [configured] and libraries == ["None"],
                   f"退 {exit_code}；假 cargo 起了 {len(records)} 次，收到的家目录 {homes}（要 [{configured}]）、单份库 {libraries}；{stderr[-400:]}")


def run_selftest():
    work = tempfile.mkdtemp(prefix="crates-mutation-rows-selftest-")
    saved_environment = dict(os.environ)
    try:
        selftest = Selftest(work)
        # import、plan 与 records-directory 读本进程的环境：与 run_stage 那一份对齐
        os.environ.clear()
        os.environ.update(selftest.base_environment)
        run_selftest_cells(selftest, work)
        selftest_coverage_cells(selftest, work)
        selftest_build_profile_cell(selftest, work)
        selftest_queue_line_cells(selftest, work)
        selftest_daily_cells(selftest, work)
        selftest_gpu_memory_cell(selftest, work)
        selftest_worker_count_cells(selftest, work)
        selftest_libtest_result_cell(selftest, work)
        selftest_cargo_configuration_cell(selftest, work)
        selftest_target_directory_cells(selftest, work)
        selftest_worker_slot_cells(selftest, work)
        selftest_signal_and_pipe_cells(selftest, work)
        selftest_crash_library_home_cells(selftest, work)
    finally:
        os.environ.clear()
        os.environ.update(saved_environment)
        shutil.rmtree(work, ignore_errors=True)
    if selftest.failures:
        print(f"  ✗ crates-mutation-rows.py 自证有 {len(selftest.failures)} 格判错（共 {selftest.cells} 格）：")
        for failure in selftest.failures:
            print(f"      {failure}")
        print("     → 怎么办：设了 GATE_MUTATION_BREAK 的，这就是要的结果（那个开关把对应的判法弄坏了）；没设的，照上面那几格去看 run_stage、reuse_decision、")
        print("       judge_coverage、dispatch_order、command_import、command_plan、escaping_literals、read_table 与 row_problems 里对应的那一段；并行理由那两格看 run_under_wrapper。")
        return 1
    print(f"  ✓ crates-mutation-rows.py 自证 {selftest.cells} 格都对（复用、改一行只重跑那一行、跑到一半续跑、基线不绿、"
          "久的先派、选行只判一部分退 3、导入核行键、判法版本与复用判据、只留 3 个判法版本目录、记录里没有主机名、拷贝范围两样、"
          "替换文空着的删行变异照常判、派活之前那几样 checker-tier-crates-mutation-replay 与 static-check 同一份判法、"
          "点名的测试打了 NOT_RUN 记本次未跑且变异那一趟补了 --show-output、只有本次未跑的退 77、should panic 的用例红了判抓到、"
          "会话里平常跑时每次包装带自己那个工作进程的并行理由且调用方设了的不覆盖、"
          "工作进程数不超过外层内存上限 ÷ 每条变异的上限而外层上限读不到时头一行写明、"
          "拷贝范围含 .cargo/（点名的测试要编 gpu 特性的那一行在副本里读得到 [env]）、被 TERM 停时删掉 $TMPDIR 下的 singlefs-mutation-replay-*、"
          "片号大于这一趟工作进程数的 -w<片号> 开跑时回收而有进程在用的不删、两个变量都没设时编译目录不落 /tmp、"
          "同一个共用编译目录上并发的两次调用领到的片号集合不相交（片号锁）而片号锁被占着的 -w<片号> 回收时留着、"
          "--list-items 接 | head 时安静退、变异复跑起的 cargo 写自己的崩溃判定库（真跑家目录底下的 mutation-replay/，或 GATE_MUTATION_CRASH_LIBRARY_HOME）而点名的单份库清掉、"
          "按覆盖复用：每组记一份覆盖、改没执行的函数全复用、改被变异的函数与执行过的函数只现跑那几行、函数外改动与新加目标全现跑、"
          "别的包改了不相干、工具链变了全现跑、不按时限过期、llvm 工具不对判红、plan 只读且逐栏报数）")
    return 0


def run_selftest_cells(selftest, work):
    repository = os.path.join(work, "repository")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    pristine = source_hash(repository)
    # 一：头一趟全跑；第二趟输入不变，假 cargo 一次都不起、判定逐字相同
    first = selftest.run(repository)
    selftest.check("头一趟判绿", first[0] == 0, first[1] + first[2])
    selftest.check("头一趟起 1 批基线 + 4 条变异，另记 4 组覆盖（各一次插桩编译、一次插桩跑）",
                   len(plain_invocations(first[3])) == 5 and len(coverage_invocations(first[3])) == 8, "\n".join(first[3]))
    second = selftest.run(repository)
    selftest.check("第二趟输入不变：cargo 一次都不起", second[0] == 0 and second[3] == [], "\n".join(second[3]))
    selftest.check("第二趟判定与头一趟逐字相同（只差复用的报数）", selftest_verdict_view(second[1]) == selftest_verdict_view(first[1])
                   and "按覆盖复用 4 条（其中源码与上次一字未动 4 条）" in second[1], second[1])
    selftest.check("第二趟 stderr 头一行报复用 4 条", "复用抓到 4 条" in (second[2].splitlines() or [""])[0], second[2])
    # 二：进程数不同、一律不复用，判定仍逐字相同
    parallel = selftest.run(repository, GATE_MUTATION_WORKERS="4", GATE_MUTATION_START_OVER="1")
    selftest.check("GATE_MUTATION_WORKERS=4 与 =1 的判定逐字相同", selftest_verdict_view(parallel[1]) == selftest_verdict_view(first[1]), parallel[1])
    selftest.check("GATE_MUTATION_START_OVER=1 一律不复用（报「一律现跑」那一栏）", len(plain_invocations(parallel[3])) == 5
                   and "不复用：GATE_MUTATION_START_OVER=1" in parallel[2] and "一律现跑 4" in parallel[1], parallel[1] + parallel[2])
    full = selftest.run(repository, SINGLEFS_GATE_FULL="1")
    selftest.check("SINGLEFS_GATE_FULL=1 一律不复用", len(plain_invocations(full[3])) == 5, "\n".join(full[3]))
    unlimited = selftest.run(repository, SINGLEFS_REUSE_HOURS="0")
    selftest.check("SINGLEFS_REUSE_HOURS=0 照样复用（记录不按时限过期）", unlimited[0] == 0 and unlimited[3] == [], "\n".join(unlimited[3]))
    # 三：只改一行的替换文，只那一行重跑（它自己的基线 + 它的变异），栏是「行新加或改了」
    table_path = os.path.join(repository, TABLE)
    replace_in_file(table_path, "{ value * 3 }", "{ value * 4 }")
    one_row = selftest.run(repository)
    changed = mutation_invocations(plain_invocations(one_row[3]), pristine)
    selftest.check("只改一行的替换文：只那一行重跑，栏是行新加或改了", one_row[0] == 0 and len(plain_invocations(one_row[3])) == 2 and len(changed) == 1
                   and "doubles" in changed[0] and per_row_how(one_row[1]).get("double-becomes-triple") == "现判：行新加或改了",
                   one_row[1] + "\n".join(one_row[3]))
    # 四：改一行注释（锚点照旧命中）：词法单元不变，全部按覆盖复用
    replace_in_file(os.path.join(repository, "crates", "tiny", "src", "lib.rs"), "pub fn add(", "// 改了一行注释\npub fn add(")
    pristine = source_hash(repository)
    comment_changed = selftest.run(repository)
    selftest.check("改一行注释：词法单元不变，全部按覆盖复用、cargo 一次都不起", comment_changed[0] == 0 and comment_changed[3] == []
                   and "按覆盖复用 4 条（其中源码与上次一字未动 0 条）" in comment_changed[1], comment_changed[1] + "\n".join(comment_changed[3]))
    # 五：跑到一半停掉（只供测试的开关），下一趟只跑剩下的；基线已判过 ok 的不再跑。先清掉这一格的记录，从零起
    shutil.rmtree(run_command_capturing(command_judge_version, [repository], want_directory=True)[1].strip(), ignore_errors=True)
    halfway = selftest.run(repository, GATE_MUTATION_START_OVER="1", GATE_MUTATION_STOP_AFTER_ROWS="2")
    selftest.check("跑完 2 条就退：判红、说明是只供测试的开关", halfway[0] == 1 and "GATE_MUTATION_STOP_AFTER_ROWS=2" in halfway[1]
                   and len(mutation_invocations(halfway[3], pristine)) == 2, halfway[1] + "\n".join(halfway[3]))
    resumed = selftest.run(repository)
    selftest.check("下一趟只跑剩下的 2 条、基线不再跑，判完的 2 条一字未动复用、补记它们的覆盖",
                   resumed[0] == 0 and len(plain_invocations(resumed[3])) == 2 and len(mutation_invocations(resumed[3], pristine)) == 2
                   and len(coverage_invocations(resumed[3])) == 8, "\n".join(resumed[3]))
    # 六：上一次跑得久（或超时）的先派，按用时从长到短；stderr 头一行报前移几条
    timings_directory = os.path.join(repository, ".git", TIMINGS_DIRECTORY_NAME)
    table_rows, _malformed, _memory, _timeout = read_table(table_path)
    long_rows = {"negates": ("400", "0"), "halves": ("1800", "1")}
    for row in table_rows:
        if row.expected in long_rows:
            seconds, timed_out = long_rows[row.expected]
            write_atomically(os.path.join(timings_directory, row_content_hash(row)),
                             record_text(dict(seconds_running=seconds, timed_out=timed_out, finished_utc=utc_now_text())))
    ordered = selftest.run(repository, GATE_MUTATION_START_OVER="1")
    order = [line.split("\t")[0].split()[-1] for line in mutation_invocations(ordered[3], pristine)]
    ordered_head_line = (ordered[2].splitlines() or [""])[0]
    selftest.check("上一次超时的、跑得久的先派（按用时从长到短）", order[:2] == ["halves", "negates"] and "前移 2 条" in ordered_head_line,
                   f"派活次序 {order}；{ordered_head_line}")
    # 七：选行只判一部分：都抓到退 3、末行写明不是整道结论；不写全绿标记
    selection_file = os.path.join(work, "selection.txt")
    selftest_write(selection_file, "".join(f"{row.line_number}\n" for row in table_rows[:2]))
    selected = selftest.run(repository, GATE_MUTATION_ROW_SELECTION=selection_file, GATE_MUTATION_START_OVER="1")
    selftest.check("选 2 行：都抓到退 3，末行写明只判了选中的 2 行", selected[0] == EXIT_SELECTION_ONLY
                   and "只判了选中的 2 行，不是整道结论" in selected[1].splitlines()[-1] and len(mutation_invocations(selected[3], pristine)) == 2,
                   selected[1])
    reused_selected = selftest.run(repository, GATE_MUTATION_ROW_SELECTION=selection_file)
    selftest.check("选 2 行、不设 START_OVER：照判定库记录复用，与整张跑时那两行的判定相同", reused_selected[0] == EXIT_SELECTION_ONLY
                   and reused_selected[3] == [] and "按覆盖复用 2 条" in reused_selected[1], reused_selected[1])
    selftest_write(os.path.join(work, "bad-selection.txt"), "99999\n")
    bad_selection = selftest.run(repository, GATE_MUTATION_ROW_SELECTION=os.path.join(work, "bad-selection.txt"))
    selftest.check("选行文件里的行号不是一条变异：判红、一格都没跑", bad_selection[0] == 1 and bad_selection[3] == [], bad_selection[1])
    # 八：导入另一台的记录：同一份内容的另一棵仓先全收，之后一格都不跑、判定与本机现跑逐字相同；行键或判法版本对不上的拒
    reference = selftest.run(repository)
    other = os.path.join(work, "other")
    build_selftest_repository(other, [line for line in open(table_path, encoding="utf-8").read().splitlines()[1:]])
    shutil.copyfile(os.path.join(repository, "crates", "tiny", "src", "lib.rs"), os.path.join(other, "crates", "tiny", "src", "lib.rs"))
    source_records = run_command_capturing(command_judge_version, [repository], want_directory=True)[1].strip()
    imported = run_command_capturing(command_import, [other, source_records])
    after_import = selftest.run(other)
    selftest.check("导入同一判法版本、同一份源码的记录：全收", imported[0] == 0, imported[1])
    selftest.check("导入之后另一棵仓一格都不跑、判定与本机逐字相同", after_import[3] == [] and selftest_verdict_view(after_import[1]) == selftest_verdict_view(reference[1]),
                   after_import[1] + after_import[2])
    forged_directory = os.path.join(work, "forged")
    os.makedirs(forged_directory)
    genuine_name = sorted(name for name in os.listdir(source_records) if name.startswith("mutation."))[0]
    genuine = read_record(os.path.join(source_records, genuine_name))
    forged_key = hashlib.sha256(b"no such row").hexdigest()
    selftest_write(os.path.join(forged_directory, f"mutation.{forged_key}"), record_text(dict(genuine, row_key=forged_key)))
    foreign_judging = dict(genuine, judge_version=hashlib.sha256(b"another judging").hexdigest())
    selftest_write(os.path.join(forged_directory, genuine_name), record_text(foreign_judging))
    forged = run_command_capturing(command_import, [other, forged_directory])
    selftest.check("导入一条行键对不上的记录：拒、列出来", forged[0] == 1 and f"mutation.{forged_key}" in forged[1] and "行键不是本机" in forged[1], forged[1])
    selftest.check("导入一条判法版本对不上的记录：拒", "判法版本" in forged[1] and "拒了 2 条" in forged[1], forged[1])
    # 九：只留最新的 3 个判法版本目录，别的名字不动
    records_root = os.path.dirname(source_records)
    for index in range(4):
        stale = os.path.join(records_root, hashlib.sha256(f"stale {index}".encode()).hexdigest())
        os.makedirs(stale)
        os.utime(stale, (1000 + index, 1000 + index))
    keep_me = os.path.join(records_root, "not-a-fingerprint")
    os.makedirs(keep_me)
    selftest.run(repository)
    remaining = sorted(name for name in os.listdir(records_root) if FINGERPRINT_FORM.fullmatch(name))
    selftest.check("只留最新的 3 个判法版本目录（这一趟的在里面），别的名字不动", len(remaining) == 3 and os.path.basename(source_records) in remaining
                   and os.path.isdir(keep_me), f"{remaining}")
    # 十：记录里不写主机名（判定库记录与解开的覆盖记录、快照）
    host_name = os.uname().nodename
    record_texts = [open(os.path.join(source_records, name), encoding="utf-8").read() for name in os.listdir(source_records)
                    if os.path.isfile(os.path.join(source_records, name))]
    for directory_name in (COVERAGE_DIRECTORY_NAME, SNAPSHOT_DIRECTORY_NAME):
        directory = os.path.join(source_records, directory_name)
        record_texts += [json.dumps(Store.read_json(os.path.join(directory, name)), ensure_ascii=False) for name in os.listdir(directory)]
    selftest.check("记录里不写主机名", len(host_name) >= 3 and record_texts and not any(host_name in text for text in record_texts), host_name)
    # 十一：基线不绿（点名的测试读一个副本里没有的文件）：不算抓到、整道判红；弄坏开关 nobaseline 下误判成抓到
    outside = os.path.join(work, "outside")
    build_selftest_repository(outside, SELFTEST_TABLE_ROWS + [SELFTEST_READS_OUTSIDE_ROW], SELFTEST_READS_OUTSIDE_TEST)
    baseline_red = selftest.run(outside)
    selftest.check("点名的测试读副本里没有的文件：记基线不绿、不算抓到、整道判红", baseline_red[0] == 1 and "基线不绿 1 行" in baseline_red[1]
                   and "reads-outside：基线不绿——点名的测试 reads_answer 在没改坏的副本上的结果是 FAILED" in baseline_red[1]
                   and "计数：抓到 4 条" in baseline_red[1], baseline_red[1])
    # 十二：每片的副本不留原来的修改时刻（留着的话，跨轮复用的编译目录里上一轮改坏的二进制会被 cargo 当成新的，基线跑的是改坏的代码）
    stale_source = os.path.join(repository, "crates", "tiny", "src", "lib.rs")
    os.utime(stale_source, (1000, 1000))
    shard_root = os.path.join(work, "shard-root")
    os.makedirs(shard_root)
    shard_work, _target = prepare_shard(1, shard_root, dict(copied=["crates", "Cargo.toml"], root=repository, shared_target_directory=os.path.join(work, "t")))
    copied_mtime = os.stat(os.path.join(shard_work, "crates", "tiny", "src", "lib.rs")).st_mtime
    selftest.check("每片的副本不留原来的修改时刻（取拷的那一刻）", copied_mtime > 1000 + 3600, f"副本的修改时刻 {copied_mtime}")
    # 十三：拷贝范围两样（code-source-discipline 调的那一份判法）
    copied = ["crates", "Cargo.toml", "Cargo.lock", "litmus"]
    selftest.check("拷贝范围：litmus/ 下的文件在、docs/ 下的不在、target 底下的不在",
                   is_copied_into_each_shard("litmus/a.litmus", copied) and not is_copied_into_each_shard("docs/a.rs", copied)
                   and not is_copied_into_each_shard("crates/x/target/debug/a", copied) and not is_copied_into_each_shard("../crates/a", copied))
    scope_root = os.path.join(work, "scope")
    selftest_write(os.path.join(scope_root, "crates", "demo", "tests", "paths.rs"),
                   '#[path = "../../outside/mod.rs"]\nmod outside;\n// "../../in-a-comment/x"\n/* "../../in-a-block/x" */\n'
                   'const A: &str = "../../litmus";\nconst B: &str = r#"../../fixture-data/x"#;\nconst C: &str = "../other-crate/src";\n'
                   'const D: &str = "../../";\nlet e = b"../../bytes/x";\n')
    literals, scanned = escaping_literals(scope_root, copied)
    outside_literals = sorted((line, resolved) for _file, line, _literal, resolved, inside in literals if not inside)
    inside_literals = [resolved for _file, _line, _literal, resolved, inside in literals if inside]
    selftest.check("逃出 crates/ 的字面量：#[path]、注释、crates/ 之内的不算，litmus 在范围里，原始字符串、字节串、指到仓根的判不在",
                   scanned == 1 and inside_literals == ["litmus"] and outside_literals == [(6, "fixture-data/x"), (8, "."), (9, "bytes/x")],
                   f"在范围里 {inside_literals}；不在 {outside_literals}")
    # 十四：替换文空着的删行变异：成形、照常改坏源码（原文换成空串）、判抓到；弄坏开关 emptyreplacementrefused 下判不成形、这一格判错
    deletion = os.path.join(work, "deletion")
    build_selftest_repository(deletion, SELFTEST_TABLE_ROWS + [SELFTEST_DELETION_ROW])
    deleted = selftest.run(deletion)
    deleted_hash = hashlib.sha256(SELFTEST_SOURCE.replace("{ value / 2 }", "", 1).encode()).hexdigest()[:12]
    selftest.check("替换文空着的删行变异：成形、原文换成空串去跑、判抓到", deleted[0] == 0 and "✓ halve-deleted：halves 红了" in deleted[1]
                   and any(line.endswith("\t" + deleted_hash) for line in deleted[3]), deleted[1] + "\n".join(deleted[3]))
    # 十五：派活之前那几样只有一份判法：static-check（门禁 code-source-discipline 调它）与 run_stage 的 prechecks 对同一张表报同样几行，删行变异（第 6 行）两边都不报
    static_root = os.path.join(work, "static")
    build_selftest_repository(static_root, SELFTEST_TABLE_ROWS + [SELFTEST_DELETION_ROW] + [row for _kind, row in SELFTEST_STATIC_PROBLEM_ROWS])
    selftest_write(os.path.join(static_root, "docs", "outside.rs"), "pub fn outside() {}\n")
    expected_problems = [(kind, 7 + index) for index, (kind, _row) in enumerate(SELFTEST_STATIC_PROBLEM_ROWS)]
    static = run_command_capturing(command_static_check, [static_root])
    reported = [(line.split("\t")[0], int(line.split("\t")[1])) for line in static[1].splitlines() if line.split("\t")[0] in ROW_PROBLEM_KINDS]
    selftest.check("static-check：不成形、锚点、测试名、拷贝范围、转义五样各报一行，删行变异不报", static[0] == 1 and reported == expected_problems,
                   static[1])
    # static-check 的失败那一路：缺 <根> 时 ERROR 行写原因、下一行给照抄的「→ 怎么办」；--table 指的文件不在时 ERROR 退 2，不报「CHECKED 0 …」退 0
    bare = run_command_capturing(command_static_check, [])
    selftest.check("static-check 缺 <根>：ERROR 行写原因、下一行给 → 怎么办、退 2",
                   bare[0] == 2 and "ERROR\t缺 <根>" in bare[1] and "→ 怎么办：python3 scripts/mutation/crates-mutation-rows.py static-check \"$PWD\"" in bare[1],
                   bare[1])
    missing_table = run_command_capturing(command_static_check, [static_root, "--table", os.path.join(work, "no-such-table.tsv")])
    selftest.check("static-check --table 指的文件不在：ERROR 退 2，不报 CHECKED", missing_table[0] == 2
                   and "ERROR\t--table 指的变异表不在" in missing_table[1] and "CHECKED" not in missing_table[1], missing_table[1])
    staged = selftest.run(static_root)
    selftest.check("checker-tier-crates-mutation-replay 派活之前判的是同一份：同样几行判红、删行变异不报、假 cargo 一次都没起",
                   staged[0] == 1 and staged[3] == [] and all(f"{TABLE}:{line_number} " in staged[1] for _kind, line_number in expected_problems)
                   and f"{TABLE}:6 " not in staged[1], staged[1])
    # 十六：点名的测试打了 NOT_RUN_NO_GPU_ADAPTER 后返回成功（第 6 行）：记本次未跑、逐条打出来，不算抓到也不算没红，整道不判红、成功句不说全抓到，
    #       往 GATE_MUTATION_NOT_RUN_RECORD 报一行；变异那一趟补了 --show-output（假 cargo 只在带它时交出那一行）。弄坏开关 notrunasmissed 下当没红记，这一格判错
    not_run_root = os.path.join(work, "not-run")
    build_selftest_repository(not_run_root, SELFTEST_TABLE_ROWS + [SELFTEST_NOT_RUN_ROW], SELFTEST_NOT_RUN_TEST)
    not_run_pristine = source_hash(not_run_root)
    not_run_record = os.path.join(work, "not-run-record.txt")
    not_run = selftest.run(not_run_root, GATE_MUTATION_NOT_RUN_RECORD=not_run_record)
    recorded_lines = open(not_run_record, encoding="utf-8").read().splitlines() if os.path.exists(not_run_record) else []
    success_lines = [line for line in not_run[1].splitlines() if line.startswith("  ✓ crates 变异表复跑")]
    selftest.check("点名的测试打了 NOT_RUN：记本次未跑、逐条列出、计数里抓到 4 条没红 0 条、整道不判红、成功句不说全抓到、记录里一行",
                   not_run[0] == 0 and "本次未跑：1 条" in not_run[1]
                   and f"{TABLE}:6 card-path-changed：本次未跑——点名的测试 gpu_parity 打了 NOT_RUN_NO_GPU_ADAPTER（样本：本机没有 GPU 适配器）" in not_run[1]
                   and "计数：抓到 4 条、没红 0 条" in not_run[1] and len(success_lines) == 1 and "抓到 4 条、本次未跑 1 条" in success_lines[0]
                   and "各自红在点名的测试上" not in not_run[1] and len(recorded_lines) == 1 and "card-path-changed" in recorded_lines[0],
                   not_run[1] + "\n".join(recorded_lines))
    shown_runs = [line for line in mutation_invocations(not_run[3], not_run_pristine) if "-- --show-output gpu_parity\t" in line]
    selftest.check("变异那一趟的 libtest 参数里补了 --show-output（插在 -- 之后），基线那一批不补",
                   len(shown_runs) == 1 and not any("--show-output" in line for line in not_run[3] if line.endswith("\t" + not_run_pristine)),
                   "\n".join(not_run[3]))
    # 十七：一条抓到的都没有、只有本次未跑的：退 77（不是通过，也不是红）
    only_not_run_root = os.path.join(work, "not-run-only")
    build_selftest_repository(only_not_run_root, [SELFTEST_NOT_RUN_ROW], SELFTEST_NOT_RUN_TEST)
    only_not_run = selftest.run(only_not_run_root)
    selftest.check("只有本次未跑、一条都没抓到：退 77", only_not_run[0] == 77 and "本次未跑：1 条" in only_not_run[1]
                   and "这一道记本次未跑，不是通过" in only_not_run[1], only_not_run[1])


def build_coverage_repository(directory):
    """覆盖那几格的样本仓：crates/tiny 用 SELFTEST_COVERAGE_SOURCE，另有一个 -p tiny 编不进去的包 crates/other。"""
    build_selftest_repository(directory, SELFTEST_COVERAGE_ROWS, source=SELFTEST_COVERAGE_SOURCE)
    selftest_write(os.path.join(directory, "crates", "other", "Cargo.toml"), '[package]\nname = "other"\nversion = "0.1.0"\n')
    selftest_write(os.path.join(directory, "crates", "other", "src", "lib.rs"), "pub fn other() -> u32 { 1 }\n")


def selftest_build_profile_cell(selftest, work):
    """插桩编译那一步（假 cargo 照构建脚本的样子写一份剖析文件，没设 LLVM_PROFILE_FILE 就写当前目录）的剖析文件只落在这一格的临时目录。"""
    repository = os.path.join(work, "build-profiles")
    build_coverage_repository(repository)
    profile_log = os.path.join(work, "cargo-profiles.log")
    with contextlib.suppress(FileNotFoundError):
        os.unlink(profile_log)
    run = selftest.run(repository, FAKE_CARGO_PROFILE_LOG=profile_log)
    written = []
    if os.path.exists(profile_log):
        with open(profile_log, encoding="utf-8") as handle:
            written = [line.split("\t") for line in handle.read().splitlines()]
    builds = [line for line in coverage_invocations(run[3]) if "--no-run" in line]
    # 这一格的 TMPDIR 是 coverage-<行号>-<随机> 下的 build-profiles/：落点的目录要叫这个名字、父目录要叫 coverage-…，且不在编译所在的工作目录里
    outside = [path for work_directory, path in written
               if os.path.dirname(path).endswith(os.sep + BUILD_PROFILES_DIRECTORY_NAME)
               and os.path.basename(os.path.dirname(os.path.dirname(path))).startswith("coverage-")
               and not (path == work_directory or path.startswith(work_directory + os.sep))]
    selftest.check("插桩编译那一步写出的剖析文件只落在这一格的临时目录、工作目录里一个都没有（假 cargo 没设 LLVM_PROFILE_FILE 就写当前目录）",
                   run[0] == 0 and len(builds) == 3 and len(written) == len(builds) and len(outside) == len(written),
                   f"退 {run[0]}；插桩编译 {len(builds)} 次；假 cargo 写了 {len(written)} 份：{written}；落对的 {len(outside)} 份")


def selftest_coverage_cells(selftest, work):
    """按覆盖复用的几格（文件头「复用判据」）：每一条判据一个改法，看哪几行现跑、栏对不对；弄坏开关各让一格判错。"""
    repository = os.path.join(work, "coverage")
    build_coverage_repository(repository)
    source = os.path.join(repository, "crates", "tiny", "src", "lib.rs")
    first = selftest.run(repository)
    records_directory = run_command_capturing(command_judge_version, [repository], want_directory=True)[1].strip()
    coverage_files = sorted(os.listdir(os.path.join(records_directory, COVERAGE_DIRECTORY_NAME))) if os.path.isdir(records_directory) else []
    pointed = {read_record(os.path.join(records_directory, name)).get("coverage") for name in os.listdir(records_directory) if name.startswith("mutation.")}
    selftest.check("头一趟每组记一份覆盖、同组的行共用一份（4 行 3 组：两行同点 adds）",
                   first[0] == 0 and len(coverage_files) == 3 and len(coverage_invocations(first[3])) == 6 and len(pointed - {""}) == 3 and "" not in pointed
                   and "没有覆盖记录 4" in first[1] and "覆盖：这一趟记了 3 组" in first[1],
                   f"覆盖记录 {coverage_files}；记录指到 {pointed}；{first[1]}")
    plan_before = sorted(os.listdir(records_directory))
    replace_in_file(source, "{ value + 1 }", "{ value + 2 }")
    plan = run_command_capturing(command_plan, [repository])
    selftest.check("plan：只读（记录目录一个文件都没多）、逐行打复用还是现跑、末行报总数",
                   plan[0] == 0 and sorted(os.listdir(records_directory)) == plan_before and plan[1].count("\t复用\t按覆盖复用\t") == 4
                   and plan[1].splitlines()[-1].startswith("合计 4 行：按覆盖复用 4 行（其中源码与上次一字未动 0 行）、现跑 0 行"), plan[1])
    unexecuted = selftest.run(repository)
    selftest.check("改了没被任何用例执行的函数：全部按覆盖复用、cargo 一次都不起", unexecuted[0] == 0 and unexecuted[3] == []
                   and set(per_row_how(unexecuted[1]).values()) == {"按覆盖复用"}, unexecuted[1] + "\n".join(unexecuted[3]))
    replace_in_file(source, "pub fn helper(value: u32) -> u32 { value + 7 }", "pub fn helper(value: u32) -> u32 { let _seven = 7; { value + 7 } }")
    mutated = selftest.run(repository)
    how = per_row_how(mutated[1])
    selftest.check("改了被变异、没被执行的函数：只现跑那一行（被变异的函数变了）",
                   mutated[0] == 0 and how.get("helper-becomes-eight") == "现判：被变异的函数变了"
                   and all(how.get(name) == "按覆盖复用" for name in ("add-becomes-mul", "add-becomes-sub", "double-becomes-triple"))
                   and len(plain_invocations(mutated[3])) == 2, f"{how}；{mutated[1]}" + "\n".join(mutated[3]))
    replace_in_file(source, "pub fn add(left: u32, right: u32) -> u32 { left + right }", "pub fn add(left: u32, right: u32) -> u32 { let _sum = 0; { left + right } }")
    executed = selftest.run(repository)
    how = per_row_how(executed[1])
    selftest.check("改了被执行、没被变异的函数：只现跑执行过它的那几行（helps 执行过的函数变了，两行 adds 被变异的函数变了）",
                   executed[0] == 0 and how.get("helper-becomes-eight") == "现判：执行过的函数变了"
                   and how.get("add-becomes-mul") == how.get("add-becomes-sub") == "现判：被变异的函数变了" and how.get("double-becomes-triple") == "按覆盖复用",
                   f"{how}；{executed[1]}")
    replace_in_file(os.path.join(repository, "crates", "other", "src", "lib.rs"), "{ 1 }", "{ 2 }")
    unrelated = selftest.run(repository)
    selftest.check("改了 -p tiny 编不进去的另一个包：全部按覆盖复用", unrelated[0] == 0 and unrelated[3] == []
                   and set(per_row_how(unrelated[1]).values()) == {"按覆盖复用"}, unrelated[1])
    replace_in_file(source, "pub struct Pair { pub left: u32 }", "pub struct Pair { pub left: u64 }")
    outside = selftest.run(repository)
    selftest.check("改了函数外的类型定义：编进它的用例点名的行全部现跑（函数外有改动）", outside[0] == 0
                   and set(per_row_how(outside[1]).values()) == {"现判：函数外有改动"} and "函数外有改动 4" in outside[1], outside[1])
    selftest_write(os.path.join(repository, "crates", "tiny", "tests", "extra.rs"), "#[test]\nfn extra() {}\n")
    new_target = selftest.run(repository)
    selftest.check("新加一个用到的源文件（tests/ 下 cargo 自动发现的测试目标）：全部现跑（函数外有改动）", new_target[0] == 0
                   and set(per_row_how(new_target[1]).values()) == {"现判：函数外有改动"}, new_target[1])
    toolchain = selftest.run(repository, FAKE_CARGO_VERSION="cargo 0.0.1 (crates-mutation-rows selftest：换了工具链)")
    selftest.check("工具链变了：每一行都按行新加或改了现跑", toolchain[0] == 0
                   and set(per_row_how(toolchain[1]).values()) == {"现判：行新加或改了"}, toolchain[1])
    mismatched = selftest.run(repository, FAKE_LLVM_VERSION="0.0.0", GATE_MUTATION_START_OVER="1")
    selftest.check("llvm 工具的 LLVM 版本与 rustc 对不上：判红、一条都不跑", mismatched[0] == 1 and mismatched[3] == []
                   and "记覆盖要的 llvm 工具不对" in mismatched[1], mismatched[1] + mismatched[2])
    empty_tools = os.path.join(work, "no-llvm-tools")
    os.makedirs(empty_tools)
    missing = selftest.run(repository, GATE_MUTATION_LLVM_TOOLS=empty_tools, GATE_MUTATION_START_OVER="1")
    selftest.check("llvm 工具不在：判红、一条都不跑、出路写 rustup component add llvm-tools", missing[0] == 1 and missing[3] == []
                   and "rustup component add llvm-tools" in missing[1], missing[1])
    # 导入照复用判据核：另一棵仓与这一棵只差被执行的 add，helps 与两行 adds 判成现跑、拒；double 那一行收
    source_records = records_directory
    other = os.path.join(work, "coverage-other")
    shutil.copytree(repository, other, ignore=shutil.ignore_patterns(".git"))
    subprocess.run(["git", "init", "-q", other], check=True)
    replace_in_file(os.path.join(other, "crates", "tiny", "src", "lib.rs"), "{ let _sum = 0; { left + right } }", "{ let _sum = 1; { left + right } }")
    imported = run_command_capturing(command_import, [other, source_records])
    selftest.check("导入改了执行过的函数的记录：拒（照本机此刻的源码判成现跑），没改到的那一行收",
                   imported[0] == 1 and imported[1].count("照本机此刻的源码判成现跑") == 3 and "收了" in imported[1]
                   and "执行过的函数变了" in imported[1] and "被变异的函数变了" in imported[1], imported[1])
def run_command_capturing(command, arguments, **keywords):
    stdout = io.StringIO()
    with contextlib.redirect_stdout(stdout):
        exit_code = command(arguments, **keywords)
    return exit_code, stdout.getvalue()


def main(arguments):
    if arguments[:1] == ["--selftest"]:
        return run_selftest()
    if arguments[:1] == ["--item"]:
        return command_item(arguments)
    commands = {"static-check": command_static_check, "judge-version": command_judge_version,
                "records-directory": lambda rest: command_judge_version(rest, want_directory=True), "import": command_import,
                "row-plan": command_row_plan, "plan": command_plan, "llvm-tools": command_llvm_tools, "--list-items": command_list_items}
    if not arguments or arguments[0] not in commands:
        print("  ✗ 用法：crates-mutation-rows.py {static-check|judge-version|records-directory|import|row-plan|plan|llvm-tools|--list-items|--item} … 或 --selftest",
              file=sys.stderr)
        print("     → 怎么办：各子命令的参数见文件头；整张表的复跑只经门禁 checker-tier-crates-mutation-replay.sh（bash .claude/gate.d/checker-tier-crates-mutation-replay.sh）", file=sys.stderr)
        return 2
    return commands[arguments[0]](arguments[1:])


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
