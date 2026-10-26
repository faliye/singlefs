#!/usr/bin/env python3
# admission: always 判的是此刻被判的仓、它的 crates/mutations.tsv 与 git common-dir 里那一份判定库，上一次的结论不替这一次作保
# run-condition: none 算判法单元、行键要的 git、cargo、rustc 由 admission.py 的 git_common_directory 与 lib_mutation_units.py 的 toolchain_lines 起、起不来报错退 2；llvm-profdata、llvm-cov 由 run_stage 现找、找不到判红；--selftest 用自己造的假 cargo、假 llvm 工具与假内存包装，不编译
"""门禁 checker-tier-crates-mutation-replay（crates 变异表复跑）的判法与判定库，连同门禁 code-source-discipline 静态判的那几样：一份判法只在这里，两道阶段都调它。

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
                                          UNDEFINED <行号> <变异名> <文件> <标识符，逗号分隔>  替换文施加之后新出现、被改文件与它所在的包的 .rs 里都一处没有的标识符
                                                                                         （replacement_undefined_identifiers；替换文自己绑定的、方法与路径里的段、宏、属性里的、_ 起头的、
                                                                                         标准库预导入与基本类型不算；只 static-check 判，checker-tier-crates-mutation-replay 派活之前不判、套用后编不过记无效）
                                          IDENTIFIERS_CHECKED <判了标识符的行数> <没判的行数（文件不是 .rs 或锚点失效）>
                                          COPIED   <拷贝范围，空格分隔>
                                          CHECKED  <成形的行数> <.rs 份数> <逃出 crates/ 的字面量处数（在范围里的与不在的合计）>
                                        门禁 code-source-discipline 调它；--table 不给就读 <根>/crates/mutations.tsv（不在当空表）；
                                        参数不对（缺 <根>、--table 缺值、<根> 不是目录）与 --table 指的文件不在都打 ERROR 行（原因）、
                                        下一行「→ 怎么办」，退 2
  judging-units <根> [--units]          判法单元指纹（见「判法单元」）：打「<sha256> <单元数>」，带 --units 再逐个打单元（种类、文件:起-止、限定名、指纹）；算不出（拒算）退 2
  toolchain <根>                        行键里的工具链行（rustc -vV、cargo -V、构建环境解析后的值，照起 cargo 时的环境算、含关调试信息那四个变量）；双机分片的驱动拿两台的比。算不出退 2
  library-directory <根>                这棵树此刻那一份判定库的路径（双机分片的驱动从第二台拷回来比对用）；算不出退 2
  row-plan <根>                         双机分片的驱动（scripts/mutation/mutation-shard-run.sh）分行用：每一条成形的变异打一行
                                          <行号> <行键，拒算写 -> <这一趟会不会复用，1 或 0> <上一次现跑的秒数，没记过写 -> <anchor-broken 或 ->
                                        （制表符分隔；「会不会复用」与 run_stage 同一份判法 row_decision，SINGLEFS_GATE_FULL=1 或 GATE_MUTATION_START_OVER=1 时一律 0；
                                        锚点失效的行（见「锚点失效」）不算键，打「<行号> - 0 - anchor-broken」）；
                                        退 0；读不了表、上限写错、算不出判法单元或工具链退 2
  plan <根>                             只读：不编、不跑、不写，逐行打「<行号>\\t<变异名>\\t复用|现判|拒算|锚点失效\\t<栏>\\t<细节>」（栏见「复用判据」，细节里列变了哪几个单元），
                                        末行「合计 <N> 行：复用 <M> 行、现判 <K> 行（<各栏的数>）、拒算 <R> 行、锚点失效 <A> 行」。退 0；读不了表、上限写错、算不出判法单元退 2
  import <根> <另一台的库目录>          把另一台这一趟判出来的记录并进本机那一份库：records/ 与 baselines/ 逐条核文件名等于记录里的键、判法单元指纹等于本机此刻的、
                                        键是本机变异表里某一行照本机此刻的源码、判法单元、工具链与依赖算得出的（行键里有被改坏的单元、点名测试的闭包，
                                        源码不同的那几行自然对不上）；coverage/ 只收本机某条点名测试此刻的覆盖号；compositions/ 与 units/ 照收（只当索引）。
                                        符的写进本机那一份库，不符的拒、逐条列出。跨机不比主机名，记录里也不写。退 0 全收；退 1 有拒的；退 2 用法错或算不出判法单元
  --list-items <根>                     逐行打可点名的变异：「<变异名>\\t第 <行号> 行，改 <文件>」，锚点失效的那一行末尾接「；锚点失效：<为什么>」，不编、不跑（与门禁 --list-items 同一份）；
                                        末行合计这一趟键对不上（要现判）、对得上（复用）、拒算、锚点失效的各几条，锚点失效的逐行点名（行号与变异名）；
                                        再一行报牵连面（见「牵连面」）
  --item <行号或变异名> [--item …] <根>  只判点名的几行：交给门禁 checker-tier-crates-mutation-replay.sh --item（同一份选行模式，点名单跑与整张跑时那一行判定相同，
                                        判定库照写、照复用）；点名的不在表里退 2。是重型（.claude/hooks/lib_heavy_tests.py 按名字认）
  llvm-tools <根>                       打找到的 llvm-profdata、llvm-cov 与它们的 LLVM 版本；找不到或与 rustc 的 LLVM 版本对不上退 1、给出路
  --selftest                            自证（假 cargo、假 llvm 工具、假内存包装，在临时仓里跑 run_stage；格数由成功行现算）；
                                        有格的前提没造出来（并发领片号那一格两次调用没重叠）记本次未跑、逐格列出，别的格都对时退 77，不判绿

拷贝范围 = .claude/gate.d/stage-inputs.tsv 里 checker-tier-crates-mutation-replay 那一行登记的路径（读本模块所在那一份仓的登记表：判别力样本的目录里没有登记表），
  底下每一段都不叫 target（各 crate 自己的编译目录）。checker-tier-crates-mutation-replay 照它拷每片的源码副本，code-source-discipline 照它静态判；判法只有 is_copied_into_each_shard 一处。
  ② 管不到的：路径拼在变量里、环境变量给的、format! 拼出来的、不以 "../ 起头而运行期再 join("..") 的；#[path = …] 不算（编译期带进来的源文件，
  副本里编得过就在）；指到仓根本身（"../../" 这一类）按不在范围里算（之后拼上的是哪一份判不出）。

锚点失效（anchor_problem，row_problems 的 ANCHOR 与 run_stage、plan、row-plan、--list-items 同一份判法）：原文在被判仓里「文件」那一段指的源码里不是恰好命中一次，
  或文件不在、读不了。按行判，不让整张表退出：那一行记它自己的判定「锚点失效」（不起 cargo、不算键、不写判定库、不复用，整道判红），别的行照判、照入库；
  「计数：」那一行有「锚点失效 N 行」一栏，判红的那一段逐行点名（行号、变异名、为什么）；选行（--item、--quick、双机分片的驱动）同样按行。
  派活之前另几样（不成形、测试名、拷贝范围、转义）照旧有一行就整道判红、一个工作进程都不起，那时锚点失效的行连同它们一起列出。
  row-plan 给锚点失效的行写「-」键与 anchor-broken，双机分片的驱动不把它们分给第二台。

牵连面（impact_summary，只报数、不判红）：每一行的键依赖的单元 = 点名测试的单元闭包（KeyContext 为这一行算好的那一份，不另算）连同被改坏的单元；
  一个单元进了几行的这一份，改它一次就作废几行的判定。成功句之后与 --list-items 末了各打一行
  「牵连最大的函数单元：<文件>::<限定名> 进了 N 行的闭包（前 5：<单元> N、…）；一次改动最坏牵连 N 行」，单元按（种类、文件、限定名）认，
  N 相同按单元名排；拒算与锚点失效的行没有闭包，不算进，那一行末尾写明算了几行、没算几行。


复用：每行一条判定，行键是被改坏的函数单元指纹、原文、替换文、点名测试的单元闭包键、判「抓没抓到」的判法单元指纹、工具链、解析出的依赖版本，
  不带变异名、路径、测试名、内存上限、限时；没有管整库的版本号、没有版本目录；一份库，第二台判出的经 import 逐行核键并进本机那一份。
  没有整道全绿标记，不问 stage-must-run.sh。单元的切法、指纹与闭包在 scripts/gate/lib_function_units.py，这里经 scripts/mutation/lib_mutation_units.py 调
  （mutate.sh 的按条库也用那一份，文件头写了单元、工具链行、依赖版本行与库的存放）。

判法单元（judging_units，JUDGING_ENTRIES）：判「抓没抓到」的那几个函数——结局分类 verdict_of_run（连同它调到的 libtest 结果行解析，
  scripts/gate/lib_libtest_result_lines.py 一份）、基线分类 baseline_problem、覆盖复用判据 baseline_reuse_problem、切 fn 规则
  （lib_mutation_units.py 的 UnitKeys.mutated_units、resolve_test、executed_units，连同它们调到的 lib_function_units.py）——从它们收 python 闭包合起来，
  指纹 = 闭包里单元（种类，去注释指纹）的多重集；scripts/crash/lib_verdict_library.py 的 NOT_JUDGING_FILES 那两份（D13 已定项 17 的文件级排除）不在这里排：
  lib_function_units.py 经 executed_units 管覆盖落进哪几个单元，覆盖决定基线复用，是判对错的一环，它的单元照进指纹、里面的拒算照算。
  只改注释、文档串、别的函数（阶段脚本的调度、内存包装、判定库的存放、派活与报数）指纹不变；
  改了闭包里任何一个函数（含 lib_function_units.py 的切单元），整张表的行键、基线键与覆盖号都变。闭包里有拒算（getattr 变量名、折不出的 import 路径这一类）就算不出，这一道判红、一条都不跑。

行键（KeyContext.components，文件头「复用」那一句的逐样）：
  被改坏的单元   原文所在那份 .rs 里与原文重叠的最里层单元（fn 项；函数外的项按项单元：const、static、struct、impl 头、宏……）的（种类，指纹）；
                 原文不落在任何单元上拒算；.wgsl 按 fn 切（原文所在的 wgsl fn，不在 fn 里的取它自己的常量、struct、绑定声明单元，取不到拒算，
                 不退回整文件）；别的不是 .rs 的（litmus/ 这类测试读的数据文件）取整份字节的 sha256
  原文与替换文   原样（\\n \\t \\\\ 转义）
  点名测试       cargo 参数选中的目标里那条 #[test] fn 单元的闭包键（入口 = 那条测试函数，lib_function_units.py 的 rust_closure_of，
                 带上 cargo 参数里的特性）；认不出是哪一个单元、或闭包拒算，这一行拒算
  开关与特性     cargo 参数里不是名字的开关（--release、--include-ignored 这一类）与特性
  工具链         lib_mutation_units.py 的 toolchain_lines（rustc -vV、cargo -V、构建环境解析后的值），照起 cargo 时的环境算：含关调试信息那四个变量（见「调试信息」）
  依赖版本       点名测试的闭包与被改坏的单元真点名的外部 crate 在 Cargo.lock 里落到的包与传递依赖（lib_mutation_units.py 的 dependency_lines）
  拒算的行每趟照跑、不复用、不写记录，成功句另报「拒算 R 条」，stderr 逐行写为什么。

判定库（checker-tier-crates-mutation-replay 每条判完立刻写，下一趟按「复用判据」逐行判复用还是现判；存放在 lib_mutation_units.py 的 Library）：
  放在被判那棵树的 git common-dir（GATE_MUTATION_RECORDS_HOME 设了放它底下）的 singlefs-crates-mutation-library/，固定一处：
    records/<行键>        一条变异的判定（判定、要打的那一行、判定日期、跑法、点名测试的闭包号、被改坏的单元号、工具链与依赖的号）；
                          只有超时、内存撞顶那两档另记这一趟的上限与限时（这两档本来不复用）
    baselines/<基线键>    一条点名测试在没改坏的副本上的基线（基线键 = 判法单元、测试 fn 单元指纹、开关与特性、工具链、依赖）
    coverage/<覆盖号>     覆盖记录（覆盖号 = 判法单元指纹、测试 fn 单元指纹、开关与特性、工具链）：那条测试执行过的单元及各自的指纹，不带用例全名与快照号；
                          判法单元变了（含 lib_function_units.py 切单元的改动）覆盖号跟着变，旧记录不再被引用，下一趟重记
    compositions/、units/  组成清单（判法单元、每份点名测试的闭包、工具链行、依赖版本行）与单元表：作废时照它报变了哪几个单元
    index/<变异名与文件>  这一行上一次的键与几样组成的号：只用来说清现判的原因，不当复用的依据
    timings/<六段>        上一次现跑的用时与超没超时（派活次序用；import 不导：另一台的快慢不代表本机）
  每趟读到、写到的都摸成此刻；收尾删掉 30 天里没被任何一趟引用的（lib_mutation_units.py 的 UNREFERENCED_DAYS_KEPT），不按个数、不按版本删。
  旧的 singlefs-mutation-rows/<整库版本号>/ 与 singlefs-mutation-row-timings/ 不再读也不再写（不迁移、不补签）。记录里不写主机名。
  GATE_MUTATION_RECORDS_HOME=<目录> 设了，判定库放在它底下、不放 git common-dir；run_stage、library-directory、import、row-plan、plan、--list-items 都认它。

复用判据（row_decision，按次序）：
  一律现判        SINGLEFS_GATE_FULL=1 或 GATE_MUTATION_START_OVER=1：只写不读（不读库、现判的照键写回同一份库）
  拒算            这一行算不出键（见「行键」）：照跑，不复用
  复用            库里有这把行键的记录，判的是抓到
  上次没判成抓到  有这把键的记录，判的是别的档（没红、无效、超时、内存撞顶……只有抓到的复用）
  键对不上的那一行照索引里它上一次的那一份逐样比，栏报头一样变了的：判法单元变了、被改坏的单元变了、点名测试的闭包变了、原文或替换文变了、
  工具链或依赖变了（索引里没有这一行的报「行新加」），细节里列变了的单元（改了 / 新进 / 出了 <文件>::<限定名>）。
  前提：用例对同一份源码执行的函数集合是确定的（随机源、线程、时序只改函数里走哪条分支）；SINGLEFS_GATE_FULL=1 整张重判。
  基线 ok 那一条（要现判的行先看它的基线能不能复用，baseline_reuse_problem）：库里有这把基线键的 ok 记录，而且覆盖记录里那条测试执行过的单元
  （执行过的 fn 单元，连同它闭包里不是 fn 的单元：常量、类型、静态项、宏、数据文件）此刻逐个是那个指纹，或者点名测试的闭包键与记录里的相同；
  不按整树快照比（同一个文件里加一个没人调的函数、改别的测试，基线照样复用）。

覆盖记录（一条现判成抓到的行，它点名的测试此刻还没有一份执行过的单元都没变的覆盖记录时，在没改坏的副本上记一份，judge_coverage）：
    ① cargo test --no-run --message-format=json-render-diagnostics 带 -C instrument-coverage（环境里有 RUSTFLAGS / CARGO_ENCODED_RUSTFLAGS 就接在后面，
       没有就经 --config build.rustflags，与配置文件里的合并），编译目录是这个工作进程的编译目录加 -coverage；
       LLVM_PROFILE_FILE=<这一格的 TMPDIR>/build-profiles/build-%p-%m.profraw：cargo 没带 --target 时 RUSTFLAGS 也罩构建脚本与过程宏，它们在各自依赖包的源码目录
       （~/.cargo/registry/src/ 下）里起，不设这个变量就在那里写 default_*.profraw；
    ② cargo test -- <开关> --exact <用例全名>，LLVM_PROFILE_FILE=<这一格的 TMPDIR>/cov-%p-%m.profraw；点名用例要恰好一行 ok；
       行里 libtest 参数已带 --exact 的，开关里去掉它再补自己的，--exact 只出现一次（libtest 见两个报 Option 'exact' given more than once、退 101）；
    ③ llvm-profdata merge -sparse，llvm-cov export -format=text -skip-expansions，对象是 ① 编出来的本仓每个可执行文件（测试与 bin）；
    ④ 记：计数 > 0 的函数按它第一段代码区域的行落进的最里层单元（lib_mutation_units.py 的 executed_units），连同点名测试闭包里不是 fn 的单元，
       各自的（种类、文件、限定名、指纹）；有函数落不进任何单元的，这一份记成不全、不拿来复用。
    ①–③ 经内存包装、上限与限时同变异；有一步不成这一条测试不记（下一趟它的基线只按闭包复用），成功句另报「覆盖没记成 N 份」，不判红。
  llvm 工具：GATE_MUTATION_LLVM_TOOLS=<目录> 设了用它（双机分片的驱动设了就带到第二台），不设取 rustc --print sysroot 下 lib/rustlib/<host>/bin
    （rustup 组件 llvm-tools）；两样都要在、--version 的 LLVM 版本要以 rustc -vV 的「LLVM version」起头，不然这一道判红、一条都不跑，不退回不记覆盖。
  用时表：库里 timings/<这一行六段原样的 sha256>，记上一次现跑的用时与超没超时。
  用时从包装起跑到返回分两段记：排队（调包装到命令在 scope 里起来）与跑（起来到返回）。命令起来的那一刻由包在命令外面的一层 bash 记进
    这一趟源码副本总目录底下的一个文件（不在那一条的 TMPDIR 里，不进「删前留有几个文件」）；包装的峰值表仍按原命令记键（RUN_WITH_MEMORY_CAP_KEY）。
  包装排队时每 60 秒往 stderr 报一行（scripts/hosts/run-with-memory-cap.sh 的 WAITING_REPORT_EVERY_SECONDS）；这里收包装的 stderr 判结局，
    那几行（WRAPPER_QUEUE_LINE）边跑边另转打到本进程的 stderr，行首带是哪一行变异或哪一批基线（run_forwarding_queue_lines）；收进来判结局的输出与 stdout 都不变。
  包装排队时现查别的机器（包装文件头「排队时去找别的机器」「整体兜底」）打的四种行首（HOST_LOOKUP_LINE：「run-with-memory-cap: 没查到别的机器：」
    「run-with-memory-cap: 现查别的机器：」「  → <机器>此刻放得下（」「  ! 告警记录写不进（」）只出提示、不改结局：判结局（verdict_of_run）读输出之前去掉它们，
    那一句带的末几行也就不带它们；升档证据两句（「撞了内存上限」「超过限时」）不在其中。包装 HostLookup 新加一处 say 而 HOST_LOOKUP_LINE 不认，自证那一格红。

并行理由（包装文件头「日常一条」：包装 2026-10-06 撤了「同一个会话同时只许一条」，并行理由它已经不看；下面照旧设着，无害）：
  这里起的每次包装（基线的批与每条变异）都带 RUN_WITH_MEMORY_CAP_PARALLEL_REASON=「同一次 checker-tier-crates-mutation-replay（本模块进程 <pid>）的第 <i> 个工作进程，
  各用各的源码副本与编译目录（这一轮共 <n> 个）」，<i> 是那个工作进程的片号（编译目录 -w<i> 那个号）；不设的话会话里平常跑时第 2 个工作进程起就排不上、记成「排不上没跑」。
  调用方已经设了（非空）的照原样传、不覆盖。只设在那一次包装的环境里（run_under_wrapper），不进本进程与 --check 那一次的环境；包装把它清掉之后才起 cargo。
  做法照 scripts/mutation/mutate.sh 文件头「并行」那一段。提交时的门禁带 SINGLEFS_HEAVY_TESTS=commit，不算日常，不受这条影响。

基线（派变异之前，按批在没改坏的副本上跑一遍点名的测试）：
  批的键 = 这一行 cargo 参数里 `--` 之前那一段 + `--` 之后以 `--` 起头的开关；过滤词取这一批要跑基线的各行 `--` 之后非开关词的并集，
  有一行没有过滤词就整批不带过滤词。要单独带值的开关（`--skip 名字`、`--test-threads 4`）写成 `--test-threads=4`：分开写时那个值会被当成过滤词。
  同样经内存包装、同样的上限与限时（一批取这一批里各行逐行上限的大者）；限时只罩跑测试那一趟：先在同一个上限里不限时跑一趟
  cargo test <-- 之前那一段> --no-run（编译），编不成就把那一趟当这一批的结果，编成了再限时跑（run_under_wrapper 的 compile_first；变异那一条同样）。
  每一行的点名测试在基线输出里要恰好对上一个用例、结果都是 ok，否则这一行记「基线不绿」：
  不算抓到、不算没红、它的变异不跑，整道判红。已有作数的抓到记录或基线 ok 记录的行不再跑基线。

本次未跑（2026-10-01 加）：点名的测试在本机没跑到被改坏的那一处时，打一行 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>` 后返回成功（GPU 对拍用例在没卡的机器上）。
  变异那一趟的 libtest 参数里补 --show-output（with_shown_output，插在第一个 `--` 之后）：成功用例的输出 libtest 默认不交出来；
  不用 --nocapture：单线程跑时它把用例的输出插进「test <名字> ... 」与结果之间，结果行就认不出。基线那一批不补。
  点名的测试结果是 ok、输出里有一行 test= 对上点名的测试（只比 :: 分开的末一段）的，这一条记「本次未跑」（not_run_reason）：不算抓到、不算没红，逐条打出来；
  整道不因它判红，成功句写明抓到几条、本次未跑几条；一条抓到的都没有、只有本次未跑的退 77（选行模式照旧退 3 / 1）。
  GATE_MUTATION_NOT_RUN_RECORD=<文件> 设了，每条本次未跑往里追加一行（阶段把它转给 gate.sh 的 GATE_NOT_RUN_FILE）；写不进去整道判红。
  只有「抓到」复用，本次未跑的下一趟照跑。
  带 gpu 特性的格另有两种记本次未跑（多机配置不在、给定状态全交 CPU 判），见下面「带 gpu 特性的格把多机配置交给测试、跑完核真用了卡」。

带 gpu 特性的格经清场起（gpu_quiesce_prefix、separate_gpu_quiesce_lines；.claude/rules/verification.md「凡用 GPU 先清场，跑完不复原」）：
  这一道起的 cargo test 里 `--` 之前开了上卡的特性（GPU_TEST_FEATURES：checker 档包的 gpu，与产品 GPU 包 singlefs-background-gpu 真显卡对拍的 tests-on-a-real-gpu；
  --features / -F 里写这些名字或 <包>/<名字>，或 --all-features）、真跑测试的那几次（基线的批、变异、记覆盖的插桩跑），
  命令前面另套一层 bash scripts/crash/gpu-quiesce-run.sh --（套在内存包装外面：清场等锁、清场命令不算进测试的限时，也不跑在带内存上限的 scope 里）；
  清场、回读、几趟同时用卡、跑完不复原都照那一份，用哪张卡由用例照清场之后的显卡配置取。只编不跑的几次（--no-run：先编、插桩编译、先编一份依赖）与不带 gpu 特性的不经它。
  按格套、不整趟套：只有这几格上卡；整趟套要改阶段脚本与双机分片的驱动，整张表跑几个钟头里一直挂着清场登记。
  清场包装自己打的几种行（GPU_QUIESCE step=…、回读看的卡、清了场、没复原）挪出那一次的输出，判结局与尾巴只看命令自己的。
  包装退 78、打了 GPU_QUIESCE step=command result=not-started（清场回读没过、多机配置没写全清场键、锁等不到：命令没起）：
    变异那一条记「清场没过没跑」（不交判法：不算抓到、不算没红，不写用时表），另起一行「清场没过没跑：N 条」（「计数：」那一行的格式不动），整道判红、出路是等卡上那个进程跑完再点名重跑；
    基线那一批记基线不绿，那一句写明清场回读没过、命令没起；记覆盖的插桩跑记「覆盖没记成」（不判红）。
  清场包装取 GATE_MUTATION_GPU_QUIESCE_RUNNER（设了用它，自证用），没设取本模块所在仓的 scripts/crash/gpu-quiesce-run.sh（它照那一份仓的多机配置清场）。
  双机分片的驱动在第二台起的那一份照第二台那棵树的清场包装与多机配置清场。

带 gpu 特性的格把多机配置交给测试、跑完核真用了卡（gpu_test_run、gpu_rows_multi_host_configuration、gpu_card_unused_reason）：
  工作进程在源码副本里起 cargo，副本不是 git 工作树，测试照 git common-dir 找不到多机配置，GPU 开关就是关的、全交 CPU 判。
  所以上面那几次（--features 开了 gpu、真跑测试：基线的批、变异、插桩跑；与清场同一份判法 gpu_test_run）的环境里设 SINGLEFS_MULTI_HOST_CONFIG=<多机配置的绝对路径>；
  只编不跑的几次与不带 gpu 特性的不设（调用方自己设了的照原样往下传，不动）。
  多机配置在哪：调用方设了 SINGLEFS_MULTI_HOST_CONFIG 照它（相对路径按本进程的当前目录取成绝对路径）；没设取本模块所在仓的
  ${GATE_MUTATION_MULTI_HOST_CONFIG:-<git common-dir 的上一层>/multi-host.env}（取不到 common-dir 退回本模块所在仓的根，同 scripts/gate/admission.py 的
  multi_host_configuration_file；GATE_MUTATION_MULTI_HOST_CONFIG 是自证换它用的）。那份文件不在的，不设，那一行照下面的「GPU 用不上」记。
  变异那一条判出「没红」（verdict_of_run 交 failure）、点名的测试恰好对上一个用例而且都是 ok 的，再看这一条用没用上卡，用不上记「本次未跑」（同文件头「本次未跑」：
  不算抓到、不算没红，逐条打出来，整道不因它判红）：
    ① 多机配置不在（上一段）：GPU 用不上；
    ② 点名的测试那一段输出（libtest 的「---- <全名> stdout ----」；输出里一段都没有时看整份）里有 CRASH_POINT_REPLAY_GIVEN_STATES 行，
       而每一行的 states_judged_by_the_gpu_here 都是 0：给定状态全交 CPU 判，GPU 一个状态都没判。
  点名的测试打了 FAILED 的照记抓到（.claude/rules/mutation-sampling.md「改了一个格式常量之后」一节的次序：FAILED 先于本次未跑），不再看用没用卡。
  不打 CRASH_POINT_REPLAY_GIVEN_STATES 行的点名测试（GPU 对拍、取卡、卡池这些不经给定状态判的），不另判用没用卡、照常判：
    要卡的用例都经 crate singlefs-checker-tier 的 gpu_unit_checks::gpu_test_start 取卡（用例 every_gpu_test_takes_its_cards_through_gpu_test_start 判这一条），
    它不读多机配置的 GPU 开关：卡起来了就在卡上跑，本机没有 NVIDIA 卡打 NOT_RUN_NO_GPU_ADAPTER（上面「本次未跑」那一路已经认），
    有卡而一张都起不来当场 panic（测试红），没有悄悄改交 CPU 判的一支；不要卡的（源码扫描、假设备的账本单测）判的东西跟卡无关。
  弄坏开关 gpu-rows-configuration-unpassed、gpu-unused-as-missed、given-states-count-ignored（见下）。


弄坏开关（GATE_MUTATION_BREAK，只给证红用，逗号分隔可给几个；CRATES_MUTATION_ROWS_BREAK 是同义名，直接起本模块自证时用，两个变量都认）：
  nobaseline           跳过基线：点名的测试在没改坏的副本上本来就红的行照样记抓到（checker-tier-crates-mutation-replay 红样本、本模块自证都必须判错）
  timingsignored       派活不看用时表（自证「上一次超时的、跑得久的先派」那一格必须判错）
  importunchecked      import 不核文件名、判法单元指纹与键，照单全收（自证「导入一条键对不上的记录：拒、列出来」「导入一条判法单元指纹对不上的记录：拒」两格必须判错）
  rowkeywithouttoolchain  行键不带工具链（自证「工具链变了：每一行都现判」那一格必须判错）
  row-key-with-limits  行键带上每条的内存上限与限时（改前的行键）（自证「每条限时 1800 → 3600（N33）：一行都不现判」那一格必须判错）
  mutated-unit-ignored  行键不带被改坏的单元指纹（自证「改了被改坏、点名测试够不着的函数：只现判那一行」那一格必须判错）
  test-closure-ignored  行键不带点名测试的闭包键（自证「改了点名测试调到、不是被改坏的函数：点名它的那一行现判」
                       「改了被改坏函数用的常量（V5 那一类）：原文在那个函数里的两行现判」「改了点名测试够得着、没执行到的函数：那一行现判」三格必须判错）
  baseline-by-tree-snapshot  基线 ok 只按整树快照复用（改前的判法）（自证「加一个没人调的函数、同时改一行的替换文：只那一行现判、它的基线复用」
                       「改了点名测试够得着、没执行到的函数：那一行现判，基线按覆盖复用」两格必须判错）
  judge-units-whole-file  判法单元指纹取闭包碰到的那几份文件的整份原文（改前的取法）（自证「只改判法脚本的注释：判法单元指纹不变」那一格必须判错）
  judging-function-units-excluded  判法单元指纹不收 lib_function_units.py 的单元（D13 已定项 17 文件级排除用在 crates 变异判法上的样子）
                       （本模块自证「改 lib_function_units.py 的 RustFile.row 一处函数体：判法单元指纹变，闭包里有 lib_function_units.py 的单元」那一格必须判错）
  coverage-id-without-judging  覆盖号不带判法单元指纹（改前的样子）
                       （本模块自证「覆盖号随判法单元指纹变：改 lib_function_units.py 的 RustFile.row 一处函数体，同一条点名测试的覆盖号跟着变」那一格必须判错）
  coveragenotrecorded  抓到之后不记覆盖（自证「头一趟每条点名测试记一份覆盖、同一条测试的几行共用一份」
                       「改了点名测试够得着、没执行到的函数：那一行现判，基线按覆盖复用」两格必须判错；另有记覆盖次数的四格跟着判错：
                       「头一趟起 1 批基线 + 4 条变异，另记 4 份覆盖」「下一趟只跑剩下的 2 条」「插桩编译那一步写出的剖析文件」「会话里平常跑……4 个工作进程」）
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
  gpu-rows-unquiesced  带 gpu 特性的格也不经清场包装、只经内存包装起（改前的样子）
                       （本模块自证「带 gpu 特性的行：真跑测试的那几次…各经 gpu-quiesce-run.sh 起…」「带 gpu 特性的行：变异那一趟清场回读没过…」
                       「带 gpu 特性的行：基线那一批清场回读没过…」三格必须判错；产品 GPU 包那三格「产品 GPU 包（--features tests-on-a-real-gpu）的行：真跑测试的那几次各经…」
                       「产品 GPU 包的行：变异那一趟清场回读没过…」「产品 GPU 包的行：清场过了、点名的测试没卡打 NOT_RUN_NO_GPU_ADAPTER…」同样判错，共六格）
  product-gpu-feature-unquiesced  产品 GPU 包的 tests-on-a-real-gpu 不当上卡的特性认，只认 gpu（改前的样子：这一包的真显卡对拍行不经清场、也不交多机配置）
                       （本模块自证「产品 GPU 包（--features tests-on-a-real-gpu）的行：真跑测试的那几次各经…」「产品 GPU 包的行：变异那一趟清场回读没过…」
                       「产品 GPU 包的行：清场过了、点名的测试没卡打 NOT_RUN_NO_GPU_ADAPTER…」三格必须判错）
  quiesce-refusal-unread  清场包装退 78、命令没起也不认，照命令的输出判（变异记成点名的测试没跑到、基线记成一行都没出现）
                       （本模块自证「带 gpu 特性的行：变异那一趟清场回读没过…」「带 gpu 特性的行：基线那一批清场回读没过…」「产品 GPU 包的行：变异那一趟清场回读没过…」三格必须判错）
  gpu-rows-configuration-unpassed  带 gpu 特性的格不往环境里设 SINGLEFS_MULTI_HOST_CONFIG、也不认多机配置不在（改前的样子）
                       （本模块自证「带 gpu 特性的行：真跑测试的那几次在副本里起的 cargo 环境里有 SINGLEFS_MULTI_HOST_CONFIG…」
                       「带 gpu 特性的行：多机配置不在…」两格必须判错）
  gpu-unused-as-missed  带 gpu 特性的格判出没红时不核用没用上卡（改前的样子：多机配置不在、给定状态全交 CPU 判的也记没红）
                       （本模块自证「带 gpu 特性的行：多机配置不在…」「带 gpu 特性的行：给定状态全交 CPU 判…」两格必须判错）
  given-states-count-ignored  有 CRASH_POINT_REPLAY_GIVEN_STATES 行就当没用上卡，不读 states_judged_by_the_gpu_here 的数
                       （本模块自证「带 gpu 特性的行：给定状态有 GPU 判过的…照常判」那一格必须判错）
  undefinedidentifiersunchecked  static-check 不判替换文里新引的标识符（改前的样子）
                       （本模块自证「static-check 的 UNDEFINED：替换文引了已删的标识符 halved_before 的那一行报一行…」那一格必须判错）
  overlapunchecked     并发领片号那一格不核两次调用重叠没有、照判对错（改前的样子）
                       （本模块自证「先起的那次在后起的那次开始之前就退了（样本）：并发领片号那一格认出两次调用没重叠、记本次未跑…」那一格必须判错）
  staticusagebare      static-check 参数不对时只打改前那一行用法，不写缺了什么、不给「→ 怎么办」（本模块自证「static-check 缺 <根>」那一格必须判错）
  tablemissingok       static-check --table 指的文件不在时照旧当空表判、报「CHECKED 0 …」退 0（本模块自证「static-check --table 指的文件不在」那一格必须判错）
  shardrootkeptonsignal  run_pool 不接 TERM / HUP（改前的样子：收到就停，$TMPDIR 下的 singlefs-mutation-replay-* 留着）
                       （本模块自证「被 TERM 停：$TMPDIR 下的 singlefs-mutation-replay-* … 停完删掉，退 143」那一格必须判错）
  term-sent-unwaited   自证「被 TERM 停」那一格不等「临时目录在、假 cargo 在工作进程里挂住」就发 TERM（改前碰上忙机器的样子：run_pool 还没装好信号处理，默认处理退 -15）
                       （本模块自证「被 TERM 停：$TMPDIR 下的 singlefs-mutation-replay-* 跑的时候在、停完删掉，退 143 …」那一格必须判错）
  shardrootnamedlate   run_pool 建 shard_root 照改前用 tempfile.mkdtemp（目录在 mkdir 里建出来、名字要等它返回才到调用方手里：TERM 落在这两步之间，名字丢了、目录漏在 $TMPDIR）
                       （本模块自证「TERM 落在建 shard_root 的 mkdir 之后、名字交到调用方手里之前：收尾照样删掉」那一格必须判错）
  listitemspipeunhandled  --list-items 不照 SIGPIPE 惯例退（接 | head 时打 BrokenPipeError 的回溯、退 1）
                       （本模块自证「--list-items 接 | head」那一格必须判错）
  extratargetskept     开跑时不回收片号大于这一趟工作进程数的 -w<片号> 编译目录（改前的样子：分片各一份、只增不减）
                       （本模块自证「片号大于这一趟工作进程数的 -w<片号> 连同 -coverage 开跑时回收、不大于的留着、打一行删了几份多大」那一格必须判错）
  inusetargetremoved   回收之前不核有没有进程在用（工作目录、命令行、CARGO_TARGET_DIR、在用标记），多出来的照删
                       （本模块自证「多出来的 -w<片号> 有进程在用的不删」那一格必须判错）
  targetdefaulttmp     没设 GATE_MUTATION_TARGET_DIR 时编译目录照更早的样子放 ${TMPDIR:-/tmp}/singlefs-crates-mutation-target（跨轮共用、跑完不删）
                       （本模块自证「跑完运行目录里不剩它建的编译目录（没设 GATE_MUTATION_TARGET_DIR：…）」与「跑完运行目录里不剩它建的编译目录（被 TERM 停）」两格必须判错）
  runtargetkept        跑完不删这一趟自己建的编译目录（连同它所在的 $TMPDIR 下的 singlefs-mutation-replay-*；改前的样子：编译目录跨轮留着）
                       （本模块自证「跑完运行目录里不剩它建的编译目录（没设 GATE_MUTATION_TARGET_DIR：…）」与「跑完运行目录里不剩它建的编译目录（被 TERM 停）」两格必须判错；
                       连带「被 TERM 停（双机分片的驱动停这一份的样子）：…」那一格，$TMPDIR 下的 singlefs-mutation-replay-* 留着）
  runtargetopenfilesunchecked  删这一趟的编译目录之前不核有没有进程开着里面的文件
                       （本模块自证「这一趟的编译目录里还有进程开着文件：不删、报出那个进程」那一格必须判错）
  local-shard-numbering  工作进程的片号不领锁、照改前从本调用的共享计数器现领本地序号（改前的样子：两次并发调用的第 2 片都指向同一个 -w2）
                       （本模块自证「同一个共用编译目录上并发的两次调用各起 2 个工作进程：领到的片号集合不相交、四把片号锁都被占」那一格必须判错）
  sharedcrashlibrary   起 cargo 时不换崩溃判定库的家目录、不清调用方点名的单份库（改前的样子：变异版的结论写进真跑的长期库）
                       （本模块自证「变异复跑起的 cargo 写自己的崩溃判定库」「设了 GATE_MUTATION_CRASH_LIBRARY_HOME」两格必须判错）
  anchor-stops-table   锚点失效的一行让整张表退出、一个工作进程都不起（改前的样子）
                       （本模块自证「锚点失效按行判：那一行记锚点失效、别的行照跑照入库」那一格必须判错）
  anchor-rows-planned  plan、row-plan、--list-items 不先剔掉锚点失效的行，照常算键（改前的样子：文件不是 .rs 的那一行算得出键、会被分给第二台）
                       （本模块自证「row-plan、plan、--list-items 把锚点失效的行单列」那一格必须判错）
  impact-row-dropped   牵连面少算一行（算进的头一行不数）
                       （本模块自证「牵连面：样本 crate 上牵连最大的单元与行数」那一格必须判错）
  reuse-lookup-skipped 复用判据一律不复用（键相同、判过抓到照样现判）
                       （本模块自证 reuse-three-questions 那一格必须判错；reuse-sentinel 那一格要的就是这一趟复用哨兵判红「复用失灵」）
  共用库 scripts/gate/lib_reuse_checks.py 的开关 REUSE_CHECKS_BREAK=rejudged-sentinel-passes（哨兵被现判不红）：本模块自证 reuse-sentinel 那一格必须判错
  rowlimitsignored     不读轻型变异登记表里的「# 逐行上限：」（改前的样子：每行照整张表的那一个上限与限时）
                       （本模块自证「逐行上限：一行另给的上限与限时只给它…」那一格必须判错）
  timelimitincludescompile  基线与变异不先编、限时含重编译，插桩编译也限时（改前的样子）
                       （本模块自证「逐行上限：…先编那一趟不限时…」与日常一条那两格（包装起的次数少了先编那几次）必须判错）
  compilejobsbyworkers  编译那一趟（基线与变异的先编、插桩编译）的并行度照改前取核数 ÷ 工作进程数，不按这一行的内存上限收
                       （本模块自证「编译并行度按内存上限收…」那一格必须判错；一个工作进程的两趟 4G / 9G 那一格在核数大于 2 的机器上也跟着判错）
  memorycompilemerged  包装退 250 不分编译那一趟与测试那一趟，都记内存撞顶、出路都写「无界分配」（改前的样子）
                       （本模块自证「先编那一趟撞了内存上限：记编译撞顶…」那一格必须判错）
  hostlookuplineskept  判结局（verdict_of_run）照旧读内存包装现查别的机器打的那几行（不去 HOST_LOOKUP_LINE，包装文件头「整体兜底」）
                       （本模块自证「包装现查别的机器打的那几行内容不同，判出的档与那一句逐字相同…」那一格必须判错）
  crashlibraryacrossshards  变异复跑的崩溃判定库几片共用一份、不按片号分子目录（改前的样子）
                       （本模块自证「崩溃判定库每片各一份…」「两个工作进程同时在跑…」「变异复跑起的 cargo 写自己的崩溃判定库」「设了 GATE_MUTATION_CRASH_LIBRARY_HOME」四格必须判错）
  exactduplicated      记覆盖那一趟把行里已有的 --exact 与自己补的叠成两个（改前的样子，libtest 报 Option 'exact' given more than once、覆盖记不成）
                       （本模块自证「行里已带 --exact 的变异…」那一格必须判错）
  closure-error-masked  scripts/mutation/lib_mutation_units.py 里的开关（同样从 GATE_MUTATION_BREAK 或 CRATES_MUTATION_ROWS_BREAK 认）：按路径载入的 lib_function_units.py
                       不登记进 sys.modules、算闭包的工作进程把异常原样抛（改前的样子，--list-items 末尾是 PicklingError，真错被盖住）
                       （本模块自证「闭包算不成（并行算…）」与「--list-items 遇到闭包算不成」两格必须判错）
  wgsl-whole-file      scripts/mutation/lib_mutation_units.py 里的开关（同样从 GATE_MUTATION_BREAK 或 CRATES_MUTATION_ROWS_BREAK 认）：.wgsl 照改前当数据单元、
                       取整份字节的 sha256（本模块自证「WGSL 按 fn 取键」那一格必须判错）
  rocksdb-entry-unchecked  起任何 cargo 之前不调 scripts/hosts/rocksdb-prebuilt.py entry（改前的样子）
                       （本模块自证「RocksDB 预编库的取值缺了：起 cargo 之前判红、退 1、假 cargo 一次都不起」那一格必须判错）。
                       entry 的跳过办法：被判的树里没有依赖 rocksdb 的清单（自证的小仓就是这样），或 PATH 上的 cargo 是带
                       「# rocksdb-prebuilt-entry: selftest-fake-cargo」那一行的假 cargo（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」）
复用三问与复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「一」「二」）：
  自证 reuse-three-questions 一格：一份小仓（SELFTEST_UNIT_ROWS 五行）头一趟、同一份输入第二趟、同一个文件里加一个没人调的函数（闭包外）、
    改被改坏的函数 check_root 的别一行（闭包内：原文在它里面的 root-457、root-double 两行现判）各跑一趟，打规范输出
    「REUSE_SELFTEST script=scripts/mutation/crates-mutation-rows.py …」。已有同义的格在这一格里点名、不另留：「输入不变的第二趟：5 行都复用」是 ①，
    「同一个文件里加一个没人调的函数（N45）」与弄坏开关 baseline-by-tree-snapshot 守的「加一个没人调的函数、同时改一行的替换文」是 ②，
    「被改坏函数的别一行变了（S34）」是 ③。
  每一趟 run_stage 开跑前从库里抽哨兵：候选是此刻行键在 records/ 里有「抓到」记录的行（直接读记录，与 row_decision 分开）；抽中强制现判的那一行
    栏写「复用哨兵强制现判」、照现判跑。哨兵被现判、强制现判那一行不是抓到，整道判红。种子与条数取环境里的 SINGLEFS_REUSE_SENTINEL_SEED、
    SINGLEFS_REUSE_SENTINEL_COUNT；SINGLEFS_GATE_FULL=1、GATE_MUTATION_START_OVER=1 不抽。自证别的格设 SINGLEFS_REUSE_SENTINEL_COUNT=1000（键相同的全当哨兵、不抽强制现判）。
编译目录（default_shared_target_directory，阶段脚本不另算；用户 2026-10-23 定案「自己用完自己删」，不跨轮复用）：GATE_MUTATION_TARGET_DIR 设了用它（调用方给的，跑完不删）；
  没设时 run_pool 在这一趟自己的运行目录（$TMPDIR 下的 singlefs-mutation-replay-*，下称 shard_root）里建 <shard_root>/build/singlefs-crates-mutation-target，
  这一趟的全部编译目录（第 1 片用它，其余片加 -w<片号>，记覆盖的再加 -coverage，先编一份依赖的 -seed、-seed-coverage、-seed-source，片号锁 .slot）都在 <shard_root>/build 里；
  正常结束、判红、被 TERM / HUP 停都在 run_pool 的收尾删掉（scripts/hosts/run_build_directory.py：删之前核本机没有进程开着里面的文件，有就是这一趟的子进程没收干净，
  不删、连同 shard_root 留着、逐个报进程，整道判红）。GATE_CROSS_RUN_TMPDIR 与跨轮缓存家目录不再读。
  回收（reclaim_extra_target_directories）只对调用方给的 GATE_MUTATION_TARGET_DIR 起作用：派活之前，同一层里片号大于这一趟工作进程数的 -w<片号> 与 -w<片号>-coverage 删掉，
  stderr 打一行删了几份、共多大；删之前核没人在用——有进程的工作目录在它底下、命令行里有它底下的路径、环境里 CARGO_TARGET_DIR 指到它底下，或那一片的在用标记
  （工作进程活着时在自己的编译目录里拿着 .singlefs-mutation-worker.lock 的共享锁，回收一侧试拿排他锁拿不到）被别的进程拿着，就留着、逐份写明谁在用。
  片号（claim_worker_slot）经锁领：编译目录那一层里每个片号一把锁文件（第 1 片 <编译目录>.slot，其余片 <编译目录>-w<片号>.slot），
  工作进程启动时从 1 往上试拿排他 flock（不阻塞），拿到第一个空闲的就是它的片号，锁拿到进程退出（stderr 每个工作进程报一行「领到片号 <n>」）。
  调用方给的同一个 GATE_MUTATION_TARGET_DIR 上并发的两次调用因此拿到不相交的片号集合，不再两边的第 2 片都指向同一个 -w2 而被 cargo 的目录锁互相排队。
  片号锁文件建不了、或前 4096 片都被占：报一行，退回本调用内的本地序号。回收同样核片号锁：别的调用的工作进程刚领到的那一片，被占着的不删。
逐行上限（read_row_limits、limits_for_lines）：核心部分的行各自要的内存与限时写在 .claude/gate.d/crates-mutation-light-rows.tsv 里，单起一行
  「# 逐行上限：<变异名><TAB><内存上限，systemd 写法，或 -><TAB><限时秒，或 ->[<TAB># 依据]」（# 开头：那张表的核法 crates-mutation-light-rows.py 与门禁取名字的 awk 都不读它）。
  点名的变异名照 crates/mutations.tsv 第一列认，同名的几行都拿它；没写的行、写 - 的那一样照整张表的那一个数（表头「# 每条变异的内存上限：」、GATE_MUTATION_MEMORY_MAX、默认 4G；
  限时同形）。写法不对、点名的不是成形的一行，整道判红、一行都不判。一批基线（几行共用一份 cargo 参数）取这一批里最大的上限与最长的限时，一组覆盖取它那一行的；
  工作进程数的外层内存那一项按每条的默认上限算（下一段），个别行的大上限由派活按额度排队。取法定成逐行写（不按 host-resources.py 的峰值表现取）：峰值表里撞过顶的键记的是那一次的上限，
  照它取只是把撞顶的数原样交回；逐行写的数要写依据（「推的」或量过的峰值与日期）。上限与限时都不进行键（row-key-with-limits 那一格的口径），改了不作废记录
  （自证「逐行上限不进行键：改那一行的上限与限时之后一行都不现判」）。
工作进程数（worker_count_for）：四项取最小、至少 1：核数的一半（至多 16，GATE_MUTATION_WORKERS 顶掉它）、这一趟要现跑的条数、外层内存上限 ÷ 每条变异的默认上限（向下取整；
  默认上限是没有逐行上限的那些行用的那个值，不是这一趟逐行上限里最大的：有一行写了 24G 不再把整趟压成 1 个）。
  外层内存上限读 RUN_WITH_MEMORY_CAP_OUTER_CAP（scripts/hosts/run-with-memory-cap.sh 套在外层时给命令设的，写法 正整数加 K / M / G / T，1024 进制）：
  套在外层包装里的里层包装不排队（那份文件头「套在包装里」），几条里层同时跑、合起来超过外层上限时只有 slice 的总上限兜着，所以按外层收；
  没设或写法不认就是没套在外层里：没有这一项（每条自己经包装排队、放不下的在它那里等），stderr 头一行写明「外层内存上限没读到」及原因，不悄悄少一项。
  外层上限不足一条的上限时按 1 个开（头一行写「那一项不足 1」）：里层的上限比外层大，包装会按用法错退 251，那是调用方该改上限的事，不在这里改每条的上限。
  按额度排队（job_memory_bytes、pick_ready_job，run_pool 里；外层上限读不到时整段不起作用，派活与改前一样）：派的每一格（基线的批、一条变异、一组覆盖）按它自己跑起来经包装给的上限
  （limits_for_lines 取的那个数）从外层额度里扣，在跑的各格上限之和不超过外层上限；扣不下的那一格留在队里，次序在它后面、扣得下的小格可以先派；
  不许饿死大格：一格被越过的次数到了 worker_count 就停下，它后面的都不派，等在跑的收完释放额度，释放到扣得下它就派它；什么都没在跑时队首一定派。
  一格的上限比整个外层还大：等在跑的全收完、单独起（选这条退路而不是开跑前就报错：整张表的别的行照跑，那一格由里层包装按用法错退 251、照旧记成「scope 起不来，没跑」，
  与改前的单个工作进程一样报出来；不在这里改它的上限，那是登记它的人该改的）。头一行写「逐行上限最大 X、按额度排队」，最大那一格比外层大时另写这一句。
  每条变异的上限本身不动（量到峰值之后再定）；上限与限时都不进行键，改了不作废记录（只记在超时、内存撞顶那两档的记录里）。
  第四项是磁盘余量，下一段。
磁盘余量（disk_bound_for，第四项）：每个工作进程最多占两份编译目录（普通一份、插桩一份，prepare_shard 与 judge_coverage 起的），一份涨到多少按实测峰值算：
  每个工作进程每判完一格（trim_worker_target，清之前）量一次自己那份编译目录的占盘，追加进判定库目录下的 target-occupancy.tsv（一行「<时刻秒><TAB>普通|插桩<TAB><字节>」，
  record_target_occupancy），派活之前两种各取最近 20 次里最大的当这一种的涨幅（measured_target_peaks；记录多过 1000 行时只留每种最近 20 行）；
  哪一种还没有读数（头一趟、换了库）的那一种按「编译目录上限」（下下段）算，上限只当清理线。头一行写明每种的涨幅与来源。下面「还能涨的量」「上限」照这个涨幅读；
  开 2 个以上工作进程时先编一份依赖的 -seed、-seed-coverage 两份也每一趟新编，各按一份上限另算（n ≥ 2 才加）。
  派活之前按目标盘（编译目录所在那块）的余量核：片号 1…n 各两份「还能涨的量」（上限减去此刻已占，目录还没建的是整个上限；每一趟自己建的都还没建）加起来，
  n ≥ 2 再加先编那两份，再加备用 GATE_MUTATION_DISK_RESERVE（默认 30G，推的），
  不超过余量的最大 n 就是这一项；n 从 1 数到前三项最小的那个数再多 1（放得下再多 1 个才不与前三项并列成「卡在」）。一个都放不下按 1 个开（头一行写「那一项不足 1」）。
  头一行写：磁盘余量多少、来源、备用多少、每份涨到多少（含先编那两份），n 个共还要多少，再多 1 个要多少、放不下，所以给 n 个；「卡在「磁盘余量」这一项」。
  余量的来源：外层给了 GATE_MUTATION_DISK_FREE（systemd 写法，例 400G）就用它（双机分片的驱动、自证用）；没给读编译目录往上第一层存在的目录所在的盘此刻可用的字节（statvfs）；
  两样都读不到（写法不认的整道拒；statvfs 失败的）没有这一项，头一行写「磁盘余量没读到」及原因，工作进程数不按磁盘收。已有的编译目录按此刻占盘量，
  回收（reclaim_extra_target_directories）发生在这一步之后，所以只回收得掉的多出来的目录不算作余量（偏保守）。
  依据：2026-10-20 第二台 9 个工作进程各一份 -w<片号> 加各一份插桩，1.8T 的盘写满（tempfile.mkdtemp 抛 OSError: [Errno 28]）；2026-10-21 压到 4 个仍涨到普通 39–71G、插桩 30–68G。
编译目录上限（trim_target_directory、trim_worker_target，GATE_MUTATION_TARGET_DIR_CAP，默认 64G 是推的）：每个工作进程每判完一格（基线的批、一条变异、一组覆盖），
  量一次它自己那份编译目录（判基线与变异的普通那份，判覆盖的插桩那份）此刻的占盘；超过上限就清掉 <目录>/<档>/incremental 与 <目录>/<目标三元组>/<档>/incremental
  （rustc 每编一次改坏的副本往里加一个会话目录，只增不减；两格之间没有 cargo 在这份目录上跑），清完回读再量，stderr 报清了几处、共多大、清后多大。
  清完增量仍超上限的，再清 <目录>/<档>/deps 与 <目录>/<目标三元组>/<档>/deps 里同名产物的旧版本（stale_dependency_artifacts）：文件名「<名字>-<16 位单元哈希>[.<后缀>]」
  按（目录、名字、后缀）归堆，每堆按修改时刻只留最新那一版；同一个包换了特性或档编出的另一版也算同名，之后的格要用它时 cargo 照「产物不在」重编（慢，判定不变）。
  清完回读仍超上限就停派：工作进程往这一趟 shard_root 里的停派目录（target_over_cap_directory）写一句，run_pool 每收一格看一眼，有就不再派活、等在跑的收完，
  整道判红，拒绝说明逐份点名清后仍超线的编译目录、出路是调大上限或换盘（判完的格已入库，再跑一趟只跑剩下的）。清不成（权限、文件被占）只报、不丢这一格的判定、不停派。
  本机量到的一份普通编译目录 77G，其中 debug/incremental 13G、debug/deps 30G、release 30G（2026-10-21），所以清增量产物不保证清到线下；磁盘余量那一项按实测峰值算，不依赖清得掉。
调试信息（cargo_debug_overrides、key_toolchain_lines）：这一道起的每条 cargo（基线、变异的先编与跑、插桩编译与插桩跑、先编一份依赖）都带
  CARGO_PROFILE_DEV_DEBUG、CARGO_PROFILE_TEST_DEBUG、CARGO_PROFILE_RELEASE_DEBUG、CARGO_PROFILE_BENCH_DEBUG = CARGO_DEBUG_SETTING（false）：cargo test 用 test 档（承 dev），
  cargo test --release 用 release 档（bench 承 release），四档都设，顶掉调用方设的同名变量与仓根 Cargo.toml 的 [profile.release] debug = true。只设在起 cargo 的那一份环境里。
  取值进行键：算工具链行时照起 cargo 的环境算（lib_mutation_units.py 的 toolchain_lines 收 CARGO_PROFILE_*），run_stage、plan、row-plan、import、--list-items 与 toolchain 子命令同一份；
  取值改了（连同这一段头一次进来）整张表现判一次，栏是「工具链或依赖变了」，不迁移旧键。
  不改判定：调试信息只进 DWARF 段，opt-level、overflow-checks、debug-assertions 不动；插桩的覆盖映射不靠 DWARF（2026-10-22 量过：一个探针包关掉调试信息、带 -Cinstrument-coverage
  编出来的测试二进制没有 .debug_info 段，llvm-cov export 照样报执行过的函数计数 1、没执行的 0）。
  量到的（本机，2026-10-22，singlefs-core 的测试目标冷编）：开着时写盘 916 MB、编译目录 826 MB、测试二进制 79 MB；关掉后 510 MB、421 MB、17 MB。
先编一份依赖（prepare_seed、sync_dependency_artifacts）：每个工作进程各有一份源码副本与一份编译目录（-w<片号>，插桩的再加 -coverage），冷的那几份各自把外部依赖
  （Cargo.lock 里带 source 的包：libc、vstd、wgpu、rocksdb 这一类）编一遍。这一趟开 2 个以上工作进程、要现跑的行（基线与变异）选的包顺 Cargo.lock 够得着外部依赖时，
  派活之前先在 <编译目录>-seed（插桩的 <编译目录>-seed-coverage）里按组各编一次（组 = 一行 `--` 之前的参数只留选包、特性、档与目标三元组，seed_group_of；
  先编那一组用它头一行自己选的目标，cargo 才连开发依赖一起编），每个工作进程起来时把这两份里外部依赖的产物拷进自己的两份编译目录（大小与修改时刻都对得上的不拷，拷的保留修改时刻）。
  插桩那一份只编这一趟可能要记覆盖的行（现判的行里覆盖记录不是此刻的那几行，CoveragePlanner.predicted_rows）所在的组。
  工作区里的包（Cargo.lock 里的 path 包）照旧各编各的、不拷：编出来的二进制把 CARGO_MANIFEST_DIR 编死在里面（env!），换了路径的副本拿着拷来的产物 cargo 照样判 Fresh
  （cargo 1.98 量过），跑的时候读的就是先编那一份副本的文件；unit-graph 这类在跑的时候读别的包的源码，改坏的包不是它的依赖时它不重编，判定就变了。
  先编那一份的源码副本 <编译目录>-seed-source 每一趟新建（调用方给了 GATE_MUTATION_TARGET_DIR 时跟着它留着）、按内容同步（.claude/rules/test-script-discipline.md「编译与算法都要快」那一节的修改时刻规矩）：内容没变的文件不重写、修改时刻不动；
  变了、新加、删掉的文件所在的包（crates/<目录>，有 Cargo.toml 的）整包把修改时刻摸成此刻，不在哪个包底下的改动摸全部包。先编每一组都带 --message-format=json-render-diagnostics，
  摸过的包编出来的每一个产物，在普通（或插桩）那一份里头一次出现时都要不是 fresh（那个包有 Compiling 行）；有一个头一次出现就是 fresh，整道判红、一条都不跑（stale_seed_artifacts）。
  同一份里前面的组已经以非 fresh 编出过的同一个产物，后面的组复用它、cargo 报 fresh 是对的，不算坏：产物认 cargo JSON 里的单元身份（seed_unit_identity：package_id、manifest_path、
  目标的 kind、crate_types、name，档、特性、filenames、executable；filenames 里 deps/ 下的文件名带 cargo 按包、特性、档、目标三元组、rustc 参数与依赖算的单元哈希，同一份编译目录里两个单元不会同名），
  只按包名不放行：同包另一套特性或档编出来的单元是另一个身份，头一次出现就是 fresh 照判坏。普通与插桩两份各记各的。
  建的时候（同步源码、编）拿 <编译目录>-seed.lock 的排他锁，工作进程拷的时候拿共享锁（调用方给的同一个 GATE_MUTATION_TARGET_DIR 上几次调用共用这一份时也不互相顶掉；产物按单元哈希起名）。
  内存上限：外层读得到用整个外层（这时还没有工作进程在跑），读不到用每条的默认上限 × 工作进程数；并行度 = min(核数, 上限 ÷ COMPILE_GIB_PER_JOB)，GATE_MUTATION_CARGO_JOBS 设了用它（seed_limits）。
  串行段声明（scripts/hosts/core_usage.py 文件头「声明」）：先编那一段几组共用一份编译目录、cargo 锁着它逐组编，每组的并行度又已按内存上限收过，
    所以开编之前往 SINGLEFS_SERIAL_DECLARATIONS 写一行「此刻串行：<理由>」（理由写组数、并行度与内存上限），编完（连同编坏、读写出错）写「此刻并行」。
    调用方没设这个变量时设成这一趟 shard_root 里的 serial-declarations，并放进起 cargo 的环境：agent-watch.py 到重活那棵树的进程环境里找它，
    本进程改自己的环境 /proc 里看不到，起的 cargo 与包装看得到。写不进只报一行，不碍编。
  拷（copy_seed_dependencies、sync_dependency_artifacts）：每个工作进程起来时各拷各的（几个工作进程同时拷），一个工作进程里普通、插桩两份要拷的文件
    再分到 CARGO_BUILD_JOBS（核数 ÷ 工作进程数）条线程上拷；拷是读写盘，不声明串行。
  先编没编成（退出码不是 0）的那一份只报、不判红，工作进程照旧自己编依赖（判定不受影响，只是慢）；拷的时候出错同样只报。编完照「编译目录上限」清一次那两份的增量产物；
  「磁盘余量」给这两份各留一份上限（每一趟新编）。只开 1 个工作进程、要现跑的行都够不着外部依赖（树里没有 Cargo.lock 的自证小仓就是这样）时不起，stderr 写明没起的原因。
  量到的（本机，2026-10-22，singlefs-core 的测试目标、关掉调试信息）：冷编一份写盘 510 MB、CPU 31.6 秒；拷依赖产物 175 个文件 98 MB；拷完之后在另一份新副本里编，
  只有 singlefs-core、singlefs-format、singlefs-replay-prefix 三个工作区包重编，依赖 16 个全 Fresh，写盘 343 MB、CPU 21.3 秒（与拿着整份旧编译目录的工作进程 343 MB、20.6 秒相同）。
测试线程数：不压。起 cargo test 时 RUST_TEST_THREADS 与仓里各测试读的线程数变量（SINGLEFS_THREAD_CAP 等）调用方设了什么就原样透传；只有 CARGO_BUILD_JOBS（编译并行度）按核数 ÷ 工作进程数收。
编译并行度（compile_jobs_for）：上一句算出来的是一个工作进程的上限。先编那一趟（cargo test … --no-run：基线、变异的先编与插桩编译）另按这一行的内存上限收，
  取 上限 ÷ COMPILE_GIB_PER_JOB（1.75 GiB，向下取整、至少 1）与它的小者：每条 4G 时至多 2、8G 至多 4。1.75 是推的，只有一个读数（单编 librocksdb-sys，上限 4G，
  并行度 2 峰值 3470786560 字节编成，并行度 10 起 15 秒就撞），量到别的再改。GATE_MUTATION_CARGO_JOBS 设了原样用；跑测试那一趟不收（编译已在先编那一趟做完）。
  stderr 末尾「这一轮 …」那句写明收到几。编译撞顶的救法是这一行的上限与这个系数，不是给更多行加逐行上限。
  套在外层包装里时（外层额度读得到，额度账 OuterBudget 在，compile_plan）先编那一趟不只按这一格自己的上限收，改从外层额度里此刻空着的那一部分算：
    想要的并行度（核数 ÷ 工作进程数）× COMPILE_GIB_PER_JOB 减去这一格自己的上限，向额度账借；空着的 = 外层 − 在跑各格的上限之和 − 别的先编已借走的 −
    给每个空着的工作进程接下一格留的默认上限，几个工作进程同时要时各分一份（空着的 ÷ 还没借着额度的工作进程数）；借到多少，这一趟先编的包装内存上限就是自己的上限加借到的，
    并行度是（总上限 ÷ COMPILE_GIB_PER_JOB，向下取整）与想要的并行度的小者，不低于老算法的；借到了而并行度没比老算法高的当场还回去。编完（包装一返回）就还。
    账的口径同「按额度排队」那一套：run_pool 派一格记一笔（自己的上限）、收一格销一笔，挑与记账、先编借额度都在同一把锁里做完，所以在跑各格的上限加借走的之和不超过外层上限
    （什么都没在跑时单独起的比外层还大的那一格除外，那时没有借的）。末了 stderr 报一行「额度账：…峰值」。例：外层 36G、2 个工作进程在跑各 4G、核数 32：
    每个先编借 14G，包装上限 18432M，并行度 10（老算法 2）；9 个工作进程各 4G 时额度已满，借不到，照老算法。
    先编撞了内存上限（退 250）而借过额度的，判定里写的上限仍是这一格自己的，另往 stderr 报借了多少、包装实际收到多少。
  外层额度读不到（没套在外层包装里）没有额度账：先编的并行度只按这一行自己的上限收（老算法），行为与改前一样。
撞顶分两档（verdict_of_run、baseline_problem）：包装退 250 时看交回的是先编那一趟还是跑测试那一趟（run_under_wrapper 给先编的那一趟的结果打 compile_pass 标记）。
  先编那一趟撞的记 compilememory：单列一栏「有变异在编译那一趟撞了内存上限」，出路写编译并行度与逐行上限，不写「无界分配」（点名的测试一次都没起）；
  测试那一趟撞的记 memory，栏与出路照旧。「计数：」那一行不分（两档都算内存撞顶，格式不动）；基线不绿的那一句写明「那一批编译那一趟撞了内存上限」。
变异复跑的判定库（mutation_crash_library_environment）：走判定库的 checker 档用例在变异版上判出的结论不写进真跑的长期库。起 cargo 的环境里
  SINGLEFS_CRASH_AMPLIFICATION_HOME 换成 GATE_MUTATION_CRASH_LIBRARY_HOME（设了、非空）；没设取真跑的家目录（调用方的 SINGLEFS_CRASH_AMPLIFICATION_HOME，
  没设是 ~/.local/share/singlefs/crash-amplification）底下的 mutation-replay/，跨轮留着、照用例自己的键复用；调用方点名的单份库
  SINGLEFS_CRASH_AMPLIFICATION_LIBRARY 清掉（它压过家目录）。stderr 报一行这一趟用的家目录。双机分片的驱动给第二台设成那棵树的跨趟目录底下一份。
  每个工作进程再用这个家目录底下的 shard-<片号>（crash_library_home_for_shard，片号就是 claim_worker_slot 领到的）：几个工作进程不共用一份库，
  开库等锁（crash_library.rs 的 DEFAULT_LOCK_WAIT_SECONDS，1800 秒）与测试限时（默认 1800 秒）不会相叠、把等锁记成超时；并发的几次调用片号不相交，各片的库也不被别的调用占着。
  各片的库各自照用例自己的键复用（同一条用例这一趟落在哪一片不定，换片就重判一次）；从前共用的那一份 mutation-replay/crash-state-verdicts 不再被读。
libtest 的结果行（`test <全名>[ - should panic] ... <结局>`）与汇总行只在 scripts/gate/lib_libtest_result_lines.py 解析一份（results_by_test 调它），
  它的弄坏开关 LIBTEST_RESULT_LINES_BREAK 写在它文件头；本模块自证里认得它的那一格：
  LIBTEST_RESULT_LINES_BREAK=should-panic-in-name  本模块自证「should panic 的用例红了判抓到、点名省掉模块路径只对上那一个」那一格必须判错
下面六个开关一行一个，下一行起写打开后必须判错的自证项：
  largestrowmemory     改前的样子：每条的上限换成这一趟逐行上限里最大的那个来算工作进程数，派活也不按额度排队
    本模块自证「外层 36G、一行 24G 加二十行 4G：工作进程多于 1 个，任一时刻在跑的各格上限之和不超过 36G，24G 那一行跑完了」必须判错（头一行只开 1 个）
  rowmemoryunbudgeted  工作进程数照默认上限算，但派活不按额度扣（有一行 24G 的外层 36G 里同时塞满 9 个）
    同一格必须判错（在跑的各格上限之和超过 36G）
  bypassunbounded      按额度排队但不设「一格最多被越过 worker_count 次」的限
    本模块自证「按额度排队（离散模拟）：…被后面的小格越过的次数在 1 到 worker_count 之间」（越过 15 格）与「外层 36G、一行 24G 加二十行 4G：24G 那一行…越过它的（按父进程派活的次序数）不超过 worker_count（9）行」（越过 15 行）两格必须判错；
    后一格头一批同时派出去的行由假包装按行号错开收完，所以每次都判错，不是看「同一轮收完几个」的运气
  outermemoryignored   工作进程数不看外层内存上限（当 RUN_WITH_MEMORY_CAP_OUTER_CAP 读不到，改前的样子）
    本模块自证「工作进程数不超过外层内存上限 ÷ 每条变异的上限」「外层内存上限宽裕时仍卡在别的项」「外层上限不够一条的上限」「外层内存上限交给 run_stage：开的工作进程数跟着收」四格必须判错
  noparallelreason     自己起的包装不设并行理由（改前的跑法）
    本模块自证「会话里平常跑（假包装照日常一条演）、4 个工作进程：各条照样判对，每次包装带自己那个工作进程的并行理由」
  overridecallerreason 调用方设了的并行理由也换成自己的
    本模块自证「调用方设了的并行理由原样传给每次包装、不覆盖」
下面六个开关（磁盘余量、编译目录上限、先编借额度）一行一个，下一行起写打开后必须判错的自证项：
  disknotchecked       工作进程数不看磁盘余量（改前的样子：2026-10-20 第二台 9 个工作进程把盘写满）
    本模块自证「磁盘余量放不下 9 份…」「已有的编译目录只算还能涨的…」「磁盘余量交给 run_stage…」「外层没给磁盘余量时读编译目录所在的盘…」四格必须判错
  targetcapignored     编译目录超过上限也不清增量产物（改前的样子：产物只增不减）
    本模块自证「编译目录超过上限：清掉两处增量产物…」「清完仍超线：只清增量产物…」「编译目录上限交给 run_stage…」「编译目录清完增量仍超上限：deps 里同名产物只留最新一版…」
    「编译目录清完仍超上限（1K）：头一格判完就停派…」五格必须判错
  depsnotpruned        清完增量仍超上限也不清 deps 里同名产物的旧版本（改前的样子）
    本模块自证「编译目录清完增量仍超上限：deps 里同名产物只留最新一版…」那一格必须判错
  overcapdispatched    清完仍超上限只报、照派（改前的样子）
    本模块自证「编译目录清完仍超上限（1K）：头一格判完就停派…」那一格必须判错
  diskbudgetbycap      磁盘预算不看实测峰值、每份编译目录一律按上限算（改前的样子）
    本模块自证「磁盘预算按实测峰值算：…」那一格必须判错
  compilejobsownlimit  先编那一趟只按这一格自己的上限收，不向外层额度借（改前的样子：每条 4G 时至多 2）
    本模块自证「先编向外层额度借…」「借走的加在跑的各格上限不超过外层…」「外层 36G、2 个工作进程、每条 4G、核数 32：先编…收到的并行度全是 10…」
    「外层 36G、2 个工作进程、每条 4G：任一时刻在跑的各次包装的上限之和不超过 36G，先编借过额度…」四格必须判错
下面九个开关（调试信息、先编一份依赖）一行一个，下一行起写打开后必须判错的自证项：
  debuginfo-kept       起 cargo 时不设 CARGO_PROFILE_<档>_DEBUG（改前的样子：调试信息开着）
    本模块自证「门禁起的每条 cargo 都关了调试信息」「改了构建档的取值之后行键变了」两格必须判错
  debuginfo-unkeyed    起 cargo 时照设，算工具链行时不算进去（构建档的取值不进行键）
    本模块自证「改了构建档的取值之后行键变了」那一格必须判错
  compile-per-worker   不起先编那一份，每个工作进程自己编外部依赖（改前的样子）
    本模块自证「4 个工作进程：外部依赖只在先编那一份里各编一次」那一格必须判错；「先编那一份的源码按内容同步」「先编那一份的修改时刻被留住」两格跟着判错（先编那一趟没起）；
    「先编那一段声明串行段：调用方给的声明文件里…」「先编那一段声明串行段、调用方没设 SINGLEFS_SERIAL_DECLARATIONS…」两格也跟着判错（先编那一趟没起，没有声明）
  workspaceunitsshared 从先编那一份拷产物时连工作区的包一起拷
    本模块自证「先编那一份只拷外部依赖的产物」那一格必须判错
  seedmtimekept        同步先编那一份的源码时，内容变了的文件写进去之后修改时刻改回原来的、一个包都不摸（修改时刻规矩坏了的样子）
    本模块自证「先编那一份的源码按内容同步：改过的包重编、整道照常」那一格必须判错（先编那一趟判红）
  seedfreshunchecked   先编那一趟不核摸过的包有没有重编
    本模块自证「先编那一份的修改时刻被留住：先编那一趟判红、一条都不跑」那一格必须判错
  seedfreshacrossgroups 先编那一趟每一组各判各的（改前的样子）：前面的组已经编过的产物，后面的组里 fresh 也记成坏的
    本模块自证「先编那一份有两组：后一组复用前一组编出来的产物、cargo 报 fresh 不算坏，整道照常」那一格必须判错（先编那一趟判红）
  seed-serial-undeclared 先编那一段不往 SINGLEFS_SERIAL_DECLARATIONS 写串行段声明（改前的样子：看门狗按核数过少报、看不到理由）
    本模块自证「先编那一段声明串行段：调用方给的声明文件里…」「先编那一段声明串行段、调用方没设 SINGLEFS_SERIAL_DECLARATIONS…」两格必须判错
  seed-copy-serial     工作进程从先编那一份拷依赖产物时照改前在一条线程上逐个拷
    本模块自证「工作进程拷依赖产物分到几条线程上同时拷」那一格必须判错
只供测试的开关：GATE_MUTATION_STOP_AFTER_ROWS=<k> 现跑的变异判完 k 条就不再派活、等在跑的收完，整道判红退出（造「跑到一半被杀」）。
"""
import ast
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
import random
import re
import shutil
import signal
import stat
import subprocess
import sys
import tempfile
import threading
import time
import tomllib

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
STAGE_REPOSITORY = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(STAGE_REPOSITORY, ".claude", "scripts"))
from project_preflight import preflight  # noqa: E402
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "hosts"))
from heavy_threads import HeavyThreadDefaultError, default_heavy_threads  # noqa: E402

LIBTEST_RESULT_LINES_PATH = os.path.join(SCRIPT_DIRECTORY, "..", "gate", "lib_libtest_result_lines.py")   # libtest 结果行的解析（全仓一份），进判法单元
_LIBTEST_RESULT_LINES_SPECIFICATION = importlib.util.spec_from_file_location("lib_libtest_result_lines_for_crates_mutation_rows", LIBTEST_RESULT_LINES_PATH)
libtest_result_lines = importlib.util.module_from_spec(_LIBTEST_RESULT_LINES_SPECIFICATION)
_LIBTEST_RESULT_LINES_SPECIFICATION.loader.exec_module(libtest_result_lines)
MUTATION_UNITS_PATH = os.path.join(SCRIPT_DIRECTORY, "lib_mutation_units.py")   # 函数单元键与按条判定库（与 mutate.sh 共用），切 fn 规则进判法单元
_MUTATION_UNITS_SPECIFICATION = importlib.util.spec_from_file_location("lib_mutation_units_for_crates_mutation_rows", MUTATION_UNITS_PATH)
mutation_units = importlib.util.module_from_spec(_MUTATION_UNITS_SPECIFICATION)
sys.modules[_MUTATION_UNITS_SPECIFICATION.name] = mutation_units   # 按名字登记：算闭包的子进程靠这个名字找回要跑的函数
_MUTATION_UNITS_SPECIFICATION.loader.exec_module(mutation_units)
write_atomically = mutation_units.write_atomically   # 同目录排他建临时文件、写完 fsync、改名换上；data 是 str 或 bytes
REUSE_CHECKS_PATH = os.path.join(SCRIPT_DIRECTORY, "..", "gate", "lib_reuse_checks.py")   # 复用三问与复用哨兵（全仓一份），不进判法单元
_REUSE_CHECKS_SPECIFICATION = importlib.util.spec_from_file_location("lib_reuse_checks_for_crates_mutation_rows", REUSE_CHECKS_PATH)
reuse_checks = importlib.util.module_from_spec(_REUSE_CHECKS_SPECIFICATION)
_REUSE_CHECKS_SPECIFICATION.loader.exec_module(reuse_checks)
SCRIPT_KEY = "scripts/mutation/crates-mutation-rows.py"   # 规范输出与复用哨兵认的脚本名

STAGE = "checker-tier-crates-mutation-replay.sh"   # 登记表的键（拷贝范围）、阶段脚本的文件名
STAGE_SCRIPT = os.path.join(STAGE_REPOSITORY, ".claude", "gate.d", STAGE)
MODULE_PATH = os.path.realpath(__file__)
ROCKSDB_PREBUILT_PATH = os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "rocksdb-prebuilt.py")   # 起 cargo 之前判 RocksDB 预编库的取值（run_stage）
DEFAULT_MEMORY_CAP_RUNNER = os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "run-with-memory-cap.sh")
TABLE = "crates/mutations.tsv"
# 判定库（文件头「判定库」）：被判那棵树的 git common-dir（或 GATE_MUTATION_RECORDS_HOME）底下这一个目录，固定一处、不按版本号分目录
LIBRARY_DIRECTORY_NAME = "singlefs-crates-mutation-library"
ROW_KEY_FORM = "singlefs-crates-mutation-row-v1"
BASELINE_KEY_FORM = "singlefs-crates-mutation-baseline-v1"
COVERAGE_KEY_FORM = "singlefs-crates-mutation-coverage-v1"
# 判法单元（文件头「判法单元」）：判「抓没抓到」的那几个函数，从它们收 python 闭包；阶段脚本的调度、内存包装、判定库的存放不在里面
JUDGING_ENTRIES = (
    "scripts/mutation/crates-mutation-rows.py::verdict_of_run",            # 结局分类，连同 libtest 结果行的解析（results_by_test → lib_libtest_result_lines.py）
    "scripts/mutation/crates-mutation-rows.py::baseline_problem",          # 基线那一批的结局分类
    "scripts/mutation/crates-mutation-rows.py::baseline_reuse_problem",    # 覆盖复用判据：基线 ok 那一条复用不复用
    "scripts/mutation/lib_mutation_units.py::UnitKeys.mutated_units",      # 切 fn 规则：一条变异落在哪几个单元上
    "scripts/mutation/lib_mutation_units.py::UnitKeys.resolve_test",       # 点名的测试是哪一个单元
    "scripts/mutation/lib_mutation_units.py::UnitKeys.executed_units",     # 覆盖落进哪几个单元
)
LLVM_TOOLS = ("llvm-profdata", "llvm-cov")
LLVM_TOOLS_VARIABLE = "GATE_MUTATION_LLVM_TOOLS"
COVERAGE_TARGET_SUFFIX = "-coverage"
# 编译目录（文件头「编译目录」）：GATE_MUTATION_TARGET_DIR 设了用它（跑完不删）；没设建在这一趟的 <shard_root>/build 里、跑完删
TARGET_DIRECTORY_NAME = "singlefs-crates-mutation-target"
RUN_BUILD_DIRECTORY_NAME = "build"
RUN_BUILD_REMOVAL_FAILURES = []   # run_pool 收尾没删成的这一趟编译目录（有进程开着里面的文件、或删不成）：run_stage 照它整道判红
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
# 这一道起的 cargo 关调试信息（文件头「调试信息」）：cargo test 用 test 档（承 dev），cargo test --release 用 release 档（bench 承 release），四档都设。
# 取值放在一格的列表里：自证「改了构建档的取值之后行键变了」那一格就地换它
CARGO_DEBUG_PROFILES = ("DEV", "TEST", "RELEASE", "BENCH")
CARGO_DEBUG_SETTING = ["false"]
# 先编一份依赖（文件头「先编一份依赖」）：<共用目录>-seed（插桩的再加 -coverage）、源码副本 <共用目录>-seed-source、锁 <共用目录>-seed.lock
SEED_TARGET_SUFFIX = "-seed"
SEED_SOURCE_SUFFIX = "-seed-source"
SEED_LOCK_SUFFIX = "-seed.lock"
# 组的键（seed_group_of）：选目标的开关与只管输出的开关不进键，选包、特性、档、目标三元组与别的开关进键
SEED_SELECTOR_SWITCHES = frozenset(("--lib", "--bins", "--tests", "--benches", "--examples", "--doc", "--all-targets"))
SEED_VALUE_SELECTOR_SWITCHES = frozenset(("--test", "--bin", "--example", "--bench"))
SEED_DROPPED_SWITCHES = frozenset(("-q", "--quiet", "--offline", "-v", "--verbose", "--no-run"))
SEED_VALUE_DROPPED_SWITCHES = frozenset(("-j", "--jobs", "--color", "--message-format"))
SEED_VALUE_KEPT_SWITCHES = frozenset(("-p", "--package", "--features", "-F", "--profile", "--target", "--manifest-path", "--config", "-Z"))
# 编译目录里一个单元的产物与目录：deps/ 下「[lib]<名字>-<16 位哈希>[.<后缀>]」，.fingerprint/ 与 build/ 下「<包名>-<16 位哈希>」
UNIT_ARTIFACT_NAME = re.compile(r"^.+-(?P<hash>[0-9a-f]{16})(?:\..+)?$")
UNIT_DIRECTORY_NAME = re.compile(r"^(?P<name>.+)-[0-9a-f]{16}$")
SEED_COPY_TEMPORARY_SUFFIX = ".singlefs-seed-copy"
# 现判的栏（row_decision 交的栏名，次序就是报数的次序；拒算另报，不在栏里）
REUSE_COLUMNS = (("start-over", "一律现判"), ("row-new", "行新加"), ("judging", "判法单元变了"), ("mutated", "被改坏的单元变了"),
                 ("test", "点名测试的闭包变了"), ("text", "原文或替换文变了"), ("environment", "工具链或依赖变了"),
                 ("not-caught", "上次没判成抓到"), ("unexplained", "键对不上"))
COLUMN_TITLES = dict(REUSE_COLUMNS, refused="拒算", sentinel="复用哨兵强制现判")
REASONS_SHOWN = 20   # stderr 逐行列现判与拒算原因的行数上限（只管列多少，全部的看 plan）
KEY_WORKER_MEMORY = "512M"   # 算闭包的一个子进程按这么多内存算（套在外层内存包装里时按它收子进程数，key_workers；本仓整张表 24 个子进程合计峰值 4.67 GB）
REUSABLE_VERDICT = {"mutation": "caught", "baseline": "ok"}
RECORD_HEADER = ("# 门禁 checker-tier-crates-mutation-replay 的判定库记录（scripts/mutation/crates-mutation-rows.py 写，一条一个文件，按键寻址）：只有「抓到」与基线 ok 的、"
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
# 逐行的上限与限时（文件头「逐行上限」）：轻型变异登记表里单起一行「# 逐行上限：<变异名><TAB><内存上限或 -><TAB><限时秒或 ->[<TAB># 依据]」
LIGHT_ROWS_TABLE = os.path.join(".claude", "gate.d", "crates-mutation-light-rows.tsv")
ROW_LIMIT_DIRECTIVE = re.compile(r"^#\s*逐行上限[：:]\s*(?P<name>[^\t]+?)\t(?P<memory>[^\t]+)\t(?P<timeout>[^\t]+?)\s*(?:\t#.*)?$")
DEFAULT_LONG_ROW_SECONDS = "300"               # 上一次用时不短于它（或超时）的行先派：GATE_MUTATION_LONG_ROW_SECONDS
TIMEOUT_KILL_GRACE_SECONDS = 30                # 到点发 TERM 之后再等这么久，还不退就发 KILL（交给包装的 RUN_WITH_MEMORY_CAP_KILL_GRACE）
PER_MUTATION_TEMPORARY_DIRECTORY_NAME = "per-mutation-tmp"
START_STAMP_DIRECTORY_NAME = "started-at"
# 包在命令外面的那一层：命令在 scope 里起来的那一刻写进 $1，再 exec 原命令（用时分排队与跑两段，见文件头）
START_STAMP_SCRIPT = 'printf "%s" "$EPOCHREALTIME" > "$1"; shift; exec "$@"'
VERDICTS_KILLED_BY_WRAPPER = ("timeout", "memory", "compilememory", "squeezed")
# 编译那一趟的并行度按这一行的内存上限收（文件头「编译并行度」）：每个并行的编译任务按这么多 GiB 算。推的：单编 librocksdb-sys，上限 4G、并行度 2 峰值 3470786560 字节
# （含页缓存）= 每个任务约 1.6 GiB，取 1.75 留余地；并行度 10 同一上限 15 秒就撞（records/2026-10-18-里程碑三收尾与近期欠账.md 批 4）。只有这一个读数，量到别的再改。
COMPILE_GIB_PER_JOB = 1.75
# 编译目录占盘（文件头「磁盘余量」「编译目录上限」）：每个工作进程最多占两份编译目录（普通一份、插桩一份），一份涨满按 TARGET_DIRECTORY_CAP 算。
# 默认 64G 是推的：第二台量到的 -w<片号> 普通 39–71G、插桩 30–68G（records/2026-09-16-subagent拆分提案.md 第七十八节追记，2026-10-21），取中偏上；
# 本机一份普通的此刻 77G，其中 debug/incremental 13G、debug/deps 30G、release 30G（量的，2026-10-21），所以清增量产物不一定清到线下，清完仍超线的只报、不动别的。
# 备用 30G 是推的：留给别的会话与系统，不让工作进程把盘写满（2026-10-20 第二台 1.8T 写满过一次、2026-10-21 又只剩 4.2G）。
TARGET_DIRECTORY_CAP_VARIABLE = "GATE_MUTATION_TARGET_DIR_CAP"
DEFAULT_TARGET_DIRECTORY_CAP = "64G"
DISK_FREE_VARIABLE = "GATE_MUTATION_DISK_FREE"          # 外层给的目标盘余量（systemd 写法）；不设就读编译目录所在那块盘
DISK_RESERVE_VARIABLE = "GATE_MUTATION_DISK_RESERVE"
DEFAULT_DISK_RESERVE = "30G"
TARGET_DIRECTORY_COPIES_PER_WORKER = 2                  # 普通一份、插桩一份
INCREMENTAL_DIRECTORY_NAME = "incremental"
CRASH_LIBRARY_SHARD_DIRECTORY_PREFIX = "shard-"   # 每片的崩溃判定库家目录：<变异复跑的家目录>/shard-<片号>
VERDICTS_THAT_DID_NOT_RUN = ("refused", "unavailable")
TEST_NAME_FORM = re.compile(r"[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)*")
# 点名的测试在本机没跑到被改坏的那一处时打的一行（见文件头「本次未跑」）；变异那一趟补 --show-output，libtest 才把成功用例的这一行交出来
NOT_RUN_LINE = re.compile(r"^\s*NOT_RUN_NO_GPU_ADAPTER test=(\S+)(?: reason=(.*))?$", re.M)
LIBTEST_SHOW_OUTPUT = "--show-output"
# 开了就要上卡的 cargo 特性：checker 档包的 gpu，与产品 GPU 包（singlefs-background-gpu）真显卡对拍的 tests-on-a-real-gpu（Cargo.toml 的 [features] 与 [[test]] required-features 里的那个名字）
GPU_TEST_FEATURES = frozenset({"gpu", "tests-on-a-real-gpu"})
BYTES_PER_STAT_BLOCK = 512   # os.stat 的 st_blocks 按 512 字节一块数（POSIX）
EXIT_SELECTION_ONLY = 3      # 只判了选中的行、都抓到：不是整道结论，门禁汇总记失败，不会被当成通过
# run_pool 里接住这两个信号：先停自己起的工作进程、删 shard_root，再退 128 + 信号号（双机分片的驱动停这一份、门禁被打断时都走这一条）
STOP_SIGNALS = (signal.SIGTERM, signal.SIGHUP)
# run_pool 在 $TMPDIR 下建的 shard_root 的名字起头（mutation-shard-run.sh 的自证、本模块的自证都按这个起头找它）
SHARD_ROOT_PREFIX = "singlefs-mutation-replay-"
# 源码里一处字符串字面量或注释的起头（找字面量时跳过注释用）
LITERAL_OR_COMMENT_START = re.compile(r'//|/\*|b?r#*"|b?"|\'')
PATH_ATTRIBUTE_BEFORE_LITERAL = re.compile(r"#\s*\[\s*path\s*=\s*$")
PATH_ATTRIBUTE_AFTER_LITERAL = re.compile(r"\s*\]")
# run_stage 读的环境变量（自证起每一格之前从调用方的环境里清掉，弄坏开关除外）
STAGE_VARIABLES = ("GATE_MUTATION_WORKERS", "GATE_MUTATION_CARGO_JOBS", "GATE_MUTATION_MEMORY_MAX", "GATE_MUTATION_TIMEOUT",
                   "GATE_MUTATION_ROW_SELECTION", "GATE_MUTATION_START_OVER", "GATE_MUTATION_LONG_ROW_SECONDS",
                   "GATE_MUTATION_STOP_AFTER_ROWS", "GATE_MUTATION_TARGET_DIR", "GATE_MUTATION_MEMORY_CAP_RUNNER",
                   "GATE_MUTATION_RECORDS_HOME", "GATE_MUTATION_NOT_RUN_RECORD", "GATE_MUTATION_LLVM_TOOLS", "GATE_MUTATION_CRASH_LIBRARY_HOME", "SINGLEFS_GATE_FULL",
                   "SINGLEFS_REUSE_HOURS", "GATE_MUTATION_KEY_WORKERS", OUTER_MEMORY_CAP_VARIABLE,
                   TARGET_DIRECTORY_CAP_VARIABLE, DISK_FREE_VARIABLE, DISK_RESERVE_VARIABLE)

Row = collections.namedtuple("Row", "line_number name path old new arguments expected fields")

FIELD_NAMES = ("变异名", "文件", "原文", "替换文", "cargo test 参数", "必须红的测试名")
REPLACEMENT_FIELD = FIELD_NAMES.index("替换文")   # 六段里只有它可以空：替换文空着就是把原文删掉
# 派活之前（checker-tier-crates-mutation-replay）与静态（code-source-discipline 经 static-check）判的那几样，打印与输出都照这个次序
ROW_PROBLEM_KINDS = ("MALFORMED", "ANCHOR", "NAME", "ROW", "STRAY")
_ADMISSION_MODULE = []


def admission_module():
    """scripts/gate/admission.py（git common-dir、登记表里本阶段那一行的拷贝范围、前提都用它的），第一次用到时才读进来。"""
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


def cargo_debug_overrides(environment):
    """这一道起的 cargo 的构建档覆盖（文件头「调试信息」）：{CARGO_PROFILE_<档>_DEBUG: 取值}，起 cargo 的环境照它顶掉调用方的同名变量。
    弄坏开关 debuginfo-kept：交空（改前的样子，调试信息照 Cargo.toml 与调用方的环境开着）。"""
    if break_is(environment, "debuginfo-kept"):
        return {}
    return {f"CARGO_PROFILE_{profile}_DEBUG": CARGO_DEBUG_SETTING[0] for profile in CARGO_DEBUG_PROFILES}


def key_toolchain_lines(root, environment):
    """行键里的工具链行（文件头「行键」「调试信息」）：照起 cargo 时的环境算（调用方的环境加 cargo_debug_overrides），构建档的取值就进了键。
    KeyContext 与 toolchain 子命令都调它。rustc / cargo 起不来抛 MutationUnitsError。弄坏开关 debuginfo-unkeyed：照调用方的环境算（构建档的取值不进键）。"""
    if break_is(environment, "debuginfo-unkeyed"):
        return mutation_units.toolchain_lines(root, environment)
    return mutation_units.toolchain_lines(root, dict(environment, **cargo_debug_overrides(environment)))


def compile_jobs_for(memory_max, requested_jobs, environment):
    """编译那一趟（cargo test … --no-run，基线、变异与插桩编译的先编）的并行度，字符串：requested_jobs（核数 ÷ 工作进程数）与这一行的内存上限
    memory_max 装得下的任务数（上限 ÷ COMPILE_GIB_PER_JOB，向下取整、至少 1）取小者。GATE_MUTATION_CARGO_JOBS 设了（调用方自己定的并行度）、
    上限写法不认、requested_jobs 不是正整数的原样交回。测试那一趟不经这里（编译已在先编那一趟做完，cargo test 不再起 rustc）。
    弄坏开关 compilejobsbyworkers：照改前一律交 requested_jobs。"""
    if break_is(environment, "compilejobsbyworkers") or environment.get("GATE_MUTATION_CARGO_JOBS", "").strip():
        return str(requested_jobs)
    limit = bytes_of_size(str(memory_max))
    if limit is None or not re.fullmatch(r"[1-9][0-9]*", str(requested_jobs).strip()):
        return str(requested_jobs)
    fitting = max(1, int(limit // (COMPILE_GIB_PER_JOB * 1024 ** 3)))
    return str(min(int(str(requested_jobs).strip()), fitting))


def size_text_of_bytes(byte_count):
    """字节数 → systemd 写法（向下取整到 MiB，至少 1M）；run-with-memory-cap.sh 认正整数加 K / M / G / T。"""
    return f"{max(1, int(byte_count) // (1 << 20))}M"


class OuterBudget:
    """套在外层内存包装里时（RUN_WITH_MEMORY_CAP_OUTER_CAP 读得到）的额度账，父进程与各工作进程共用一份（multiprocessing.Value，同一把锁）：
    claimed 是在跑的各格各自的上限之和（run_pool 派一格记一笔、收一格销一笔，pick_ready_job 与记账在同一把锁里做完），
    leased 是先编那一趟（compile_plan）从外层额度里此刻空着的那一部分借走的、还没还的字节，holders 是此刻借着额度的工作进程数，
    in_flight 是此刻在跑的格数，peak 是 claimed + leased 到过的最大值。不变量：claimed + leased 不超过外层上限，除非什么都没在跑时单独起了一格比外层还大的
    （pick_ready_job 的退路），那时 leased 为 0。"""

    def __init__(self, outer_bytes, worker_count, default_job_bytes):
        self.outer_bytes = outer_bytes
        self.worker_count = worker_count
        self.default_job_bytes = default_job_bytes   # 一个空着的工作进程接下一格至少要留的额度：没有逐行上限的那些行的上限
        self.claimed = multiprocessing.Value("q", 0)
        lock = self.claimed.get_lock()
        self.leased = multiprocessing.Value("q", 0, lock=lock)
        self.holders = multiprocessing.Value("i", 0, lock=lock)
        self.in_flight = multiprocessing.Value("i", 0, lock=lock)
        self.peak = multiprocessing.Value("q", 0, lock=lock)
        self.lock = lock

    def _note_peak(self):
        self.peak.value = max(self.peak.value, self.claimed.value + self.leased.value)

    def used(self):
        with self.lock:
            return self.claimed.value + self.leased.value

    def claim(self, need):
        with self.lock:
            self.claimed.value += need
            self.in_flight.value += 1
            self._note_peak()

    def release(self, need):
        with self.lock:
            self.claimed.value -= need
            self.in_flight.value -= 1

    def lease(self, wanted_extra):
        """先编那一趟想在自己这一格的上限之外再要 wanted_extra 字节：从外层额度里此刻空着的那一部分借，交回借到的字节（可以是 0）。
        空着的 = 外层 − 在跑各格的上限 − 别的先编已借走的 − 给每个空着的工作进程接下一格留的默认上限；
        几个工作进程同时要时各分一份（空着的 ÷ 还没借着额度的工作进程数），先到的不独吞。"""
        with self.lock:
            reserved = max(0, self.worker_count - self.in_flight.value) * self.default_job_bytes
            free = self.outer_bytes - self.claimed.value - self.leased.value - reserved
            divisor = max(1, self.worker_count - self.holders.value)
            taken = max(0, min(int(wanted_extra), free // divisor))
            if taken > 0:
                self.leased.value += taken
                self.holders.value += 1
                self._note_peak()
            return taken

    def give_back(self, taken):
        if taken > 0:
            with self.lock:
                self.leased.value -= taken
                self.holders.value -= 1


def compile_plan(memory_max, requested_jobs, environment, budget):
    """先编那一趟（基线、变异的先编与插桩编译）的 (内存上限, 并行度, 借的字节)。
    老算法（compile_jobs_for）只按这一格自己的上限收：每条 4G 至多 2。套在外层包装里（budget 不是 None）时改从外层额度里此刻空着的那一部分算：
    想要的并行度（requested_jobs，核数 ÷ 工作进程数）× COMPILE_GIB_PER_JOB 减去这一格自己的上限，向额度账借（OuterBudget.lease），借到多少，
    这一趟先编的内存上限就是自己的上限加借到的，并行度是（总上限 ÷ COMPILE_GIB_PER_JOB，向下取整）与 requested_jobs 的小者，不低于老算法的；
    借到了而并行度没比老算法高的，当场还回去、照老算法。GATE_MUTATION_CARGO_JOBS 设了、外层额度读不到（budget 是 None）、上限或并行度写法不认的照老算法。
    弄坏开关 compilejobsownlimit：照老算法，只按这一格自己的上限收（改前的样子）。"""
    own_jobs = compile_jobs_for(memory_max, requested_jobs, environment)
    if budget is None or break_is(environment, "compilejobsownlimit") or environment.get("GATE_MUTATION_CARGO_JOBS", "").strip():
        return memory_max, own_jobs, 0
    own_bytes = bytes_of_size(str(memory_max))
    if own_bytes is None or not re.fullmatch(r"[1-9][0-9]*", str(requested_jobs).strip()):
        return memory_max, own_jobs, 0
    requested = int(str(requested_jobs).strip())
    per_job_bytes = int(COMPILE_GIB_PER_JOB * 1024 ** 3)
    taken = budget.lease(max(0, requested * per_job_bytes - own_bytes))
    if taken <= 0:
        return memory_max, own_jobs, 0
    jobs = max(int(own_jobs), min(requested, (own_bytes + taken) // per_job_bytes))
    if jobs <= int(own_jobs):
        budget.give_back(taken)
        return memory_max, own_jobs, 0
    return size_text_of_bytes(own_bytes + taken), str(jobs), taken


def crash_library_home_for_shard(environment, worker_sequence):
    """片号 worker_sequence 那个工作进程起 cargo 用的崩溃判定库家目录：变异复跑的家目录（mutation_crash_library_environment 算的，已在 environment 里）底下的
    shard-<片号>，每片各一份。几个工作进程共用一份库时，开库等锁（crash_library.rs 的 DEFAULT_LOCK_WAIT_SECONDS 1800 秒）的重试会与测试限时（默认 1800 秒）相叠，
    等锁被记成「超时」。片号经锁领（claim_worker_slot），并发的几次调用拿到的片号不相交，所以各片的库也不会被别的调用占着。
    环境里没有换过的家目录（弄坏开关 sharedcrashlibrary）、或弄坏开关 crashlibraryacrossshards：交 None，不分。"""
    home = environment.get(CRASH_LIBRARY_HOME_VARIABLE, "").strip()
    if not home or break_is(environment, "sharedcrashlibrary") or break_is(environment, "crashlibraryacrossshards"):
        return None
    return os.path.join(home, f"{CRASH_LIBRARY_SHARD_DIRECTORY_PREFIX}{worker_sequence}")


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


def read_row_limits(root, rows, environment):
    """轻型变异登记表里的「# 逐行上限：」那几行（文件头「逐行上限」）→ ({变异表行号: (内存上限或 None, 限时秒或 None)}, [(卡在哪, 下一步)])。
    点名同一个变异名的几行都拿它；表不在交空的。弄坏开关 rowlimitsignored：不读（每行照整张表的那一个数）。"""
    limits, problems = {}, []
    if break_is(environment, "rowlimitsignored"):
        return limits, problems
    try:
        with open(os.path.join(root, LIGHT_ROWS_TABLE), encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except FileNotFoundError:
        return limits, problems
    except (OSError, UnicodeDecodeError) as error:
        return limits, [(f"读不了 {LIGHT_ROWS_TABLE}（{type(error).__name__}：{error}），逐行上限判不了", "让那张表是 UTF-8 文本、读得了再跑")]
    lines_of_name = collections.defaultdict(list)
    for row in rows:
        lines_of_name[row.name].append(row.line_number)
    for number, line in enumerate(lines, 1):
        if not re.match(r"^#\s*逐行上限[：:]", line):
            continue
        found = ROW_LIMIT_DIRECTIVE.match(line)
        where = f"{LIGHT_ROWS_TABLE}:{number}"
        if not found:
            problems.append((f"{where} 的「# 逐行上限：」不是「<变异名><TAB><内存上限或 -><TAB><限时秒或 ->」三段", "照 crates-mutation-rows.py 文件头「逐行上限」的写法改这一行"))
            continue
        name, memory, timeout = found.group("name"), found.group("memory").strip(), found.group("timeout").strip()
        if memory != "-" and not SIZE_TEXT.match(memory):
            problems.append((f"{where} 的内存上限写成了「{memory}」，不是 systemd 的写法", "写成正整数加 K / M / G / T（例 10G），不另给就写 -"))
            continue
        if timeout != "-" and not re.fullmatch(r"[1-9][0-9]*", timeout):
            problems.append((f"{where} 的限时写成了「{timeout}」，不是正整数秒", "写成正整数秒（例 3600），不另给就写 -"))
            continue
        if name not in lines_of_name:
            problems.append((f"{where} 点名的「{name}」不是 {TABLE} 里成形的一行的变异名", "名字照变异表第一列原样抄；那一行删了或改了名，就删掉或改这一行"))
            continue
        for line_number in lines_of_name[name]:
            limits[line_number] = (memory if memory != "-" else None, int(timeout) if timeout != "-" else None)
    return limits, problems


def limits_for_lines(context, line_numbers):
    """一批行（一行变异、一批基线、一组覆盖；给变异表行号）的 (内存上限, 限时秒)：逐行上限写了的取它，没写的取整张表的那一个数；几行取大的。"""
    memory_max, timeout_seconds = context["memory_max"], context["timeout_seconds"]
    for line_number in line_numbers:
        row_memory, row_timeout = context.get("row_limits", {}).get(line_number, (None, None))
        if row_memory and (bytes_of_size(row_memory) or 0) > (bytes_of_size(memory_max) or 0):
            memory_max = row_memory
        if row_timeout and row_timeout > timeout_seconds:
            timeout_seconds = row_timeout
    return memory_max, timeout_seconds


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


def anchor_problem(root, row):
    """这一行锚点失效的原因（文件头「锚点失效」）：原文在被判仓里「文件」那一段指的源码里不是恰好命中一次，或文件不在、读不了；对得上回 None。"""
    try:
        with open(os.path.join(root, row.path), encoding="utf-8") as handle:
            count = handle.read().count(row.old)
    except FileNotFoundError:
        return f"文件 {row.path} 不存在"
    except (OSError, UnicodeDecodeError) as error:
        return f"文件 {row.path} 读不了（{type(error).__name__}）"
    return None if count == 1 else f"原文在 {row.path} 里命中 {count} 次，要恰好 1 次"


def anchor_broken_rows(root, rows):
    """{行号: 为什么}：锚点失效的那几行（run_stage、plan_context 先把它们剔出去、单记一档）。"""
    broken = {}
    for row in rows:
        why = anchor_problem(root, row)
        if why is not None:
            broken[row.line_number] = why
    return broken


def anchor_broken_text(row, why):
    """锚点失效那一行的判定（run_stage 的「锚点失效」那一档与 --list-items 末行的逐行点名同一句）。"""
    return f"{TABLE}:{row.line_number} {row.name}：锚点失效——{why}（这一行没跑、不入库、不复用）"


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
        why = anchor_problem(root, row)
        if why is not None:
            problems["ANCHOR"].append((row.line_number, row.name, why))
        if not TEST_NAME_FORM.fullmatch(row.expected):
            problems["NAME"].append((row.line_number, row.name, row.expected))
        for segment_name, segment in (("原文", row.fields[2]), ("替换文", row.fields[3])):
            for escape in stray_escapes(segment):
                problems["STRAY"].append((row.line_number, row.name, segment_name, escape))
    problems["ROW"] = [(row.line_number, row.name, row.path) for row in rows_outside_copy(rows, copied)]
    return problems


def package_directory_of(root, relative_path):
    """被改文件所在的包：从它所在的目录往上找第一份 Cargo.toml，不越过 <根>、也不取 <根> 本身（仓根那一份是工作区）；找不到回 None。"""
    root_directory = os.path.abspath(root)
    directory = os.path.dirname(os.path.abspath(os.path.join(root, relative_path)))
    while directory != root_directory and directory.startswith(root_directory + os.sep):
        if os.path.isfile(os.path.join(directory, "Cargo.toml")):
            return directory
        directory = os.path.dirname(directory)
    return None


def rust_identifier_set(text):
    """一段 Rust 源码里作为标识符出现过的词（lib_function_units.py 的 rust_lex：注释、字符串里的不算；r#name 记成 name）。"""
    units = mutation_units.function_units()
    kinds, texts, _lines, _documents = units.rust_lex(text)
    return {word[2:] if word.startswith("r#") else word for kind, word in zip(kinds, texts) if kind == units.IDENT}


def package_identifier_set(package_directory, cache):
    """包目录底下每份 .rs（跳过 target）里出现过的标识符的并集；cache 是 {包目录: 集合}，同一趟 static-check 里每个包只读一遍。"""
    if package_directory not in cache:
        words = set()
        for current_directory, directory_names, file_names in os.walk(package_directory):
            directory_names[:] = sorted(name for name in directory_names if name != "target")
            for file_name in sorted(file_names):
                if file_name.endswith(".rs"):
                    with open(os.path.join(current_directory, file_name), encoding="utf-8", errors="replace") as handle:
                        words |= rust_identifier_set(handle.read())
        cache[package_directory] = words
    return cache[package_directory]


def rust_unbound_references(kinds, words, watched):
    """一串 rust_lex 词里，watched 中的名字哪几个有「引用」而没有一处「绑定」：按头一次引用的次序回出来（replacement_undefined_identifiers 判 UNDEFINED 用）。
    words 是去掉 r# 的词。认作绑定的：let 之后到 = ; 单个 : 之前、for 之后到 in 之前的模式，闭包两条 | 之间的参数，fn struct enum const static type mod trait union mut ref
    后面紧跟的名字，fn / impl / struct / enum / trait / type 之后尖括号里的泛型参数，后面接单个 : 的（参数名、字段名、类型标注前的绑定），后面接 @ 的，
    match 臂 => 之前模式里小写起头的名字（往后只隔着模式里会有的词就到 =>；越过深度 0 的逗号时到 => 要在外层，越过的右花括号之后只许接 => , ) ] } | if）。
    不算引用的：_ 起头的、#[…] / #![…] 属性里的、跟在 . 后面或 :: 前后的、后面接 ! 的。"""
    units = mutation_units.function_units()
    identifier, punctuation = units.IDENT, units.PUNCT
    count = len(words)
    padded = ["", ""] + words + ["", ""]   # rust_lex 的标点一个字符一个词：「::」是两个「:」，「=>」是「=」「>」；padded[index + 2] 是 words[index]
    single_binders = ("fn", "struct", "enum", "const", "static", "type", "mod", "trait", "union", "mut", "ref")
    pattern_words = ("ref", "mut", "if")
    pattern_punctuation = (",", "(", ")", "[", "]", "{", "}", "|", "@", "&", ".", "-")

    def is_identifier(index):
        return 0 <= index < count and kinds[index] == identifier

    def colon_alone(index):   # words[index] 是单个「:」，不是「::」里的
        return words[index] == ":" and padded[index + 1] != ":" and padded[index + 3] != ":"

    def arrow_at(index):
        return words[index] == "=" and padded[index + 3] == ">"

    def match_arm_binding(index):
        depth, crossed_comma, step = 0, False, index + 1
        while step < count and step < index + 400:
            word = words[step]
            if arrow_at(step):
                return depth < 0 or not crossed_comma
            if word == "if" and kinds[step] == identifier:
                return True
            if kinds[step] == identifier and word not in pattern_words and word in units.RUST_KEYWORDS:
                return False
            if kinds[step] == punctuation and word not in pattern_punctuation:
                return False
            if word in ("(", "[", "{"):
                depth += 1
            elif word in (")", "]", "}"):
                depth -= 1
                if word == "}" and depth < 0 and not (padded[step + 3] == "=" and padded[step + 4] == ">") \
                        and padded[step + 3] not in (",", ")", "]", "}", "|", "if"):
                    return False   # 出了外层的块（match 臂的块收尾），后面接的是下一臂
            elif word == ",":
                if depth < 0:
                    return False
                if depth == 0:
                    crossed_comma = True
            step += 1
        return False

    bound, references, pattern_until, attribute_depth, generic_depth = set(), [], None, 0, 0
    for index in range(count):
        word, kind = words[index], kinds[index]
        if attribute_depth:   # #[…] / #![…] 里：数方括号到收尾
            attribute_depth += {"[": 1, "]": -1}.get(word, 0) if kind == punctuation else 0
            continue
        if kind == punctuation and word == "[" and (padded[index + 1] == "#" or padded[index + 1] == "!" and padded[index] == "#"):
            attribute_depth = 1
            continue
        if generic_depth:   # fn / impl / struct … 之后尖括号里：< 或 , 后面紧跟的名字是泛型参数
            generic_depth += {"<": 1, ">": -1}.get(word, 0) if kind == punctuation else 0
            if kind == identifier and padded[index + 1] in ("<", ",") and word in watched:
                bound.add(word)
            continue
        if kind == punctuation and word == "<" and (is_identifier(index - 1) and padded[index] in ("fn", "struct", "enum", "trait", "type", "union")
                                                    or padded[index + 1] == "impl"):
            generic_depth = 1
            continue
        if pattern_until is not None:
            if word in pattern_until and (word != ":" or colon_alone(index)):
                pattern_until = None
            elif kind == identifier:
                bound.add(word)
            continue
        if kind == identifier and word == "let":
            pattern_until = ("=", ";", ":")
            continue
        if kind == identifier and word == "for" and padded[index + 3] != "<":
            pattern_until = ("in",)
            continue
        if kind == punctuation and word == "|" and padded[index + 1] in ("", "(", ",", "=", "move", "{", ";", ">", "return"):
            pattern_until = ("|",)
            continue
        if kind != identifier or word not in watched:
            continue
        before, after = padded[index + 1], padded[index + 3]
        if padded[index + 1] in single_binders or after == "@" or after == ":" and colon_alone(index + 1) \
                or word[:1].islower() and after not in ("(", "{", "!", ":", ".") and match_arm_binding(index):
            bound.add(word)
            continue
        if word.startswith("_") or before == "." or before == ":" and padded[index] == ":" or after in ("!", ":"):
            continue
        if word not in references:
            references.append(word)
    return [word for word in references if word not in bound]


def replacement_undefined_identifiers(root, row, cache):
    """static-check 的 UNDEFINED（文件头 static-check 那一段）：替换文施加之后新出现的标识符（施加之前整份被改文件里一处都没有的），
    在被改文件所在的包的 .rs 里也一处不出现、又不是替换文自己绑定的，按头一次引用的次序回出来；对得上回 []。
    先粗筛：替换文单独切词，去掉关键字、标准库预导入与基本类型的名字、_ 起头的、被改文件与它所在的包里出现过的；一个都不剩就回 []。
    剩下的再细判：把替换文施加进被改文件、整份切词（这样替换文起止落在字符串或注释中间也切得对），只看施加前没有的那几个名字，
    绑定与不算引用的写法照 rust_unbound_references。cache 是 {包目录: 标识符集合, ("file", 路径): 被改文件的标识符集合}。
    不判的：文件不是 .rs、锚点失效（ANCHOR 已报）。弄坏开关 undefinedidentifiersunchecked：一个都不报。"""
    if break_is(os.environ, "undefinedidentifiersunchecked") or not row.path.endswith(".rs") or anchor_problem(root, row) is not None:
        return []
    units = mutation_units.function_units()
    predeclared = frozenset((
        "_", "self", "Self", "super", "crate", "std", "core", "alloc",
        "bool", "char", "str", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64",
        "Some", "None", "Ok", "Err", "Option", "Result", "Vec", "String", "Box", "ToString", "ToOwned", "Clone", "Copy", "Default", "Drop",
        "Eq", "PartialEq", "Ord", "PartialOrd", "Iterator", "IntoIterator", "Extend", "FromIterator", "DoubleEndedIterator", "ExactSizeIterator",
        "Fn", "FnMut", "FnOnce", "From", "Into", "TryFrom", "TryInto", "AsRef", "AsMut", "Send", "Sync", "Sized", "Unpin", "drop"))
    file_key = ("file", row.path)
    with open(os.path.join(root, row.path), encoding="utf-8", errors="replace") as handle:
        original_text = handle.read()
    if file_key not in cache:
        cache[file_key] = rust_identifier_set(original_text)
    candidates = {word for word in rust_identifier_set(row.new)
                  if word not in units.RUST_KEYWORDS and word not in predeclared and not word.startswith("_") and word not in cache[file_key]}
    package_directory = package_directory_of(root, row.path) if candidates else None
    if package_directory is not None:
        candidates -= package_identifier_set(package_directory, cache)
    if not candidates:
        return []
    kinds, texts, _lines, _documents = units.rust_lex(original_text.replace(row.old, row.new, 1))
    words = [word[2:] if word.startswith("r#") else word for word in texts]
    return rust_unbound_references(kinds, words, candidates)


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


# ── 判法单元、行键、判定库 ──────────────────────────────────────────────────────

def read_bytes(path):
    with open(path, "rb") as handle:
        return handle.read()


class JudgingUnitsRefused(Exception):
    """判法单元的闭包里有拒算：算不出判法单元指纹，一条记录都对不上（文件头「判法单元」）。"""


_JUDGING_CACHE = {}
# 判法所在那一份仓：平常是本模块所在的仓；自证开头换成判法的一份冻住的临时拷贝（别的会话这时候改了 lib_function_units.py，自证里几十趟 run_stage 的判法单元照样一致）
JUDGING_ROOT = [STAGE_REPOSITORY]


def judging_units(environment, repository=None):
    """(判法单元指纹, 单元清单)：文件头「判法单元」那几个入口函数的闭包合起来，键只随闭包里单元的（种类，去注释指纹）变。
    repository 是判法所在那一份仓（不给取 JUDGING_ROOT；自证拿临时拷贝改了判法再算）。弄坏开关 judge-units-whole-file：指纹换成闭包碰到的那几份文件的整份原文。
    同一个进程里闭包碰到的那几份文件都没动过（修改时刻与大小）就沿用上一次算的（自证一趟起几十次 run_stage）。"""
    repository = repository or JUDGING_ROOT[0]
    whole_file = break_is(environment, "judge-units-whole-file")
    function_units_excluded = break_is(environment, "judging-function-units-excluded")
    cached = _JUDGING_CACHE.get((repository, whole_file, function_units_excluded))
    if cached is not None:
        key, references, stats = cached
        if all(_file_stat(os.path.join(repository, path)) == stat for path, stat in stats.items()):
            return key, references
    key, references, refusals = mutation_units.judging_units(repository, JUDGING_ENTRIES, whole_file=whole_file)
    if key is not None and function_units_excluded:
        # 弄坏开关（只给证红用）：lib_function_units.py 的单元不进指纹（D13 已定项 17 文件级排除用在这里的样子）
        references = [reference for reference in references if reference.file != "scripts/gate/lib_function_units.py"]
        key = mutation_units.key_of_lines([("form", ["judging-function-units-excluded"]), ("unit", [f"{reference.kind}\t{reference.fingerprint}" for reference in references])])
    if key is None:
        raise JudgingUnitsRefused("；".join(refusals[:3]) + (f" 等 {len(refusals)} 处" if len(refusals) > 3 else ""))
    _JUDGING_CACHE[(repository, whole_file, function_units_excluded)] = (key, references, {path: _file_stat(os.path.join(repository, path)) for path in {reference.file for reference in references}})
    return key, references


def _file_stat(path):
    try:
        status = os.stat(path)
    except OSError:
        return None
    return status.st_mtime_ns, status.st_size


def row_content_hash(row):
    """六段原样的 sha256：只给用时表与索引当名字，不进任何键。"""
    return hashlib.sha256("\t".join(row.fields).encode("utf-8", "surrogateescape")).hexdigest()


def row_identity(row):
    """索引（index/）里这一行的名字：变异名与文件。只用来说清现判的原因，不进键。"""
    return hashlib.sha256(f"{row.name}\t{row.path}".encode("utf-8", "surrogateescape")).hexdigest()


def utc_now_text():
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())


def reuse_refusal(environment):
    """一律不读库的理由（全量开关：只写不读，照键写回同一份库）；读库时交 None。"""
    for variable in ("SINGLEFS_GATE_FULL", "GATE_MUTATION_START_OVER"):
        if environment.get(variable, "") == "1":
            return f"{variable}=1"
    return None


def records_home(root, environment):
    """判定库放在哪个目录底下：GATE_MUTATION_RECORDS_HOME 设了就是它，不设是被判那棵树的 git common-dir；取不到抛 InputManifestError。"""
    configured = environment.get("GATE_MUTATION_RECORDS_HOME", "").strip()
    if configured:
        return configured
    admission = admission_module()
    common_directory = admission.git_common_directory(root)
    if common_directory is None:
        raise admission.InputManifestError(f"{root} 不是 git 工作树（取不到 git common-dir）")
    return common_directory


def library_directory(root, environment):
    return os.path.join(records_home(root, environment), LIBRARY_DIRECTORY_NAME)


def open_library(root, environment):
    return mutation_units.Library(library_directory(root, environment), RECORD_HEADER)


def snapshot_of(base, copied):
    """base 底下拷贝范围里（跳过 target 与 crates/mutations.tsv）每个文件的 {路径: sha256}：只给弄坏开关 baseline-by-tree-snapshot 用（基线按整树快照比，改前的判法），
    基线记录里照记一份当属性，不当复用的依据。"""
    files = {}
    for item in copied:
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


Components = collections.namedtuple("Components", "row_key refusal mutated test_unit test_full_name test_closure switches toolchain dependency "
                                                  "baseline_key coverage_id judging")


def lines_key(label, lines):
    return mutation_units.key_of_lines([(label, lines)])


def key_workers(environment):
    """算闭包开几个子进程：GATE_MUTATION_KEY_WORKERS 给了（正整数）照它；不然取每台默认重活线程数（scripts/hosts/heavy_threads.py），
    套在外层内存包装里时（RUN_WITH_MEMORY_CAP_OUTER_CAP）再按外层上限 ÷ KEY_WORKER_MEMORY 收、至少 1。登记文件读不了抛 HeavyThreadDefaultError。"""
    requested = environment.get("GATE_MUTATION_KEY_WORKERS", "").strip()
    if re.fullmatch(r"[1-9][0-9]*", requested):
        return int(requested)
    count = int(default_heavy_threads(environment))
    outer = bytes_of_size(environment.get(OUTER_MEMORY_CAP_VARIABLE, "").strip() or "-")
    if outer is not None:
        count = min(count, max(1, outer // bytes_of_size(KEY_WORKER_MEMORY)))
    return count


class KeyContext:
    """一棵树这一趟算键要的几样（文件头「行键」）：判法单元指纹、工具链行、单元键（UnitKeys）、依赖版本行按用到的包缓存。"""

    def __init__(self, root, environment, memory_max, timeout_seconds, judging_repository=None):
        self.root, self.environment = root, environment
        self.memory_max, self.timeout_seconds = memory_max, timeout_seconds
        self.judging, self.judging_references = judging_units(environment, judging_repository)
        self.toolchain_lines = [] if break_is(environment, "rowkeywithouttoolchain") else key_toolchain_lines(root, environment)
        self.toolchain = lines_key("toolchain", self.toolchain_lines)
        self.keys = mutation_units.UnitKeys(root)
        self._dependencies = {}
        self.closures = {}

    def prepare(self, rows):
        """先把这一趟每一行点名测试的闭包算好（并行，进程数见 key_workers）；components 之后照缓存取。回算了几个闭包。"""
        requests, labels = [], []
        for row in rows:
            _mutated, refusal = self.keys.mutated_units(row.path, row.old)
            test, test_refusal = (None, refusal) if refusal is not None else self.keys.resolve_test(row.arguments, row.expected)
            if test_refusal is None:
                requests.append((test.unit, test.target, mutation_units.parse_cargo_arguments(row.arguments)))
                labels.append(f"{TABLE}:{row.line_number} {row.name}")
        return self.keys.precompute_closures(requests, key_workers(self.environment), labels)

    def dependency(self, lines):
        """(依赖版本行的键, 那几行)：行是点名测试的闭包与被改坏的单元点名的外部 crate 收到的（lib_mutation_units.py 文件头「依赖版本行」），按行缓存。"""
        lines = tuple(sorted(lines))
        if lines not in self._dependencies:
            self._dependencies[lines] = (lines_key("dependency", list(lines)), list(lines))
        return self._dependencies[lines]

    def components(self, row):
        """这一行的 Components：键算得出交齐全的一份，算不出（拒算）row_key 是 None、refusal 写为什么。"""
        mutated, refusal = self.keys.mutated_units(row.path, row.old)
        if refusal is not None:
            return Components(None, f"被改坏的单元：{refusal}", None, None, None, None, None, self.toolchain, None, None, None, self.judging)
        test, refusal = self.keys.resolve_test(row.arguments, row.expected)
        if refusal is not None:
            return Components(None, refusal, mutated, None, None, None, None, self.toolchain, None, None, None, self.judging)
        arguments = mutation_units.parse_cargo_arguments(row.arguments)
        closure = self.keys.closure(test.unit, test.target, arguments)
        if closure.key is None:
            return Components(None, f"点名测试 {test.full_name} 的闭包拒算（{closure.refusals[0]}）", mutated, test.unit, test.full_name,
                              None, None, self.toolchain, None, None, None, self.judging)
        self.closures[closure.key] = closure
        mutated_dependencies, dependency_refusals = self.keys.reference_dependencies(mutated)
        if dependency_refusals:
            return Components(None, f"被改坏的单元的依赖版本行拒算（{dependency_refusals[0]}）", mutated, test.unit, test.full_name,
                              None, None, self.toolchain, None, None, None, self.judging)
        dependency, _lines = self.dependency(set(closure.dependencies) | set(mutated_dependencies))
        mutated_lines = [] if break_is(self.environment, "mutated-unit-ignored") else [f"{reference.kind}\t{reference.fingerprint}" for reference in mutated]
        test_key = "" if break_is(self.environment, "test-closure-ignored") else closure.key
        groups = [("form", [ROW_KEY_FORM]), ("judging", [self.judging]), ("mutated", mutated_lines),
                  ("original", [mutation_units.escape_field(row.old)]), ("replacement", [mutation_units.escape_field(row.new)]),
                  ("test", [test_key]), ("switches", arguments.switches), ("features", sorted(self.keys.features_of(arguments, test.target))),
                  ("toolchain", [self.toolchain]), ("dependency", [dependency])]
        if break_is(self.environment, "row-key-with-limits"):
            groups.append(("limits", [f"{self.memory_max}\t{self.timeout_seconds}"]))
        row_key = mutation_units.key_of_lines(groups)
        baseline_key = mutation_units.key_of_lines([("form", [BASELINE_KEY_FORM]), ("judging", [self.judging]), ("test-unit", [test.unit.fingerprint]),
                                                    ("switches", arguments.switches), ("features", sorted(self.keys.features_of(arguments, test.target))),
                                                    ("toolchain", [self.toolchain]), ("dependency", [dependency])])
        # 覆盖记录里的 executed 由 executed_units 落单元（lib_function_units.py 给行号区间）：判法单元指纹变了（含切单元的改动）旧记录不能再认，覆盖号带判法单元指纹
        coverage_judging = [] if break_is(self.environment, "coverage-id-without-judging") else [("judging", [self.judging])]
        coverage_id = mutation_units.key_of_lines([("form", [COVERAGE_KEY_FORM])] + coverage_judging + [("test-unit", [test.unit.fingerprint]),
                                                   ("switches", arguments.switches), ("features", sorted(self.keys.features_of(arguments, test.target))),
                                                   ("toolchain", [self.toolchain])])
        return Components(row_key, None, mutated, test.unit, test.full_name, closure.key, arguments.switches, self.toolchain, dependency,
                          baseline_key, coverage_id, self.judging)

    def persist(self, library, components_by_line):
        """组成清单进库（判法单元、每一份点名测试的闭包、工具链行、依赖版本行），新见到的单元并进 units/。只在 run_stage 里调（plan 只读）。"""
        library.put_composition(self.judging, self.judging_references)
        library.put_composition(self.toolchain, lines=self.toolchain_lines)
        for key, lines in self._dependencies.values():
            library.put_composition(key, lines=lines)
        for components in components_by_line.values():
            if components.test_closure in self.closures:
                library.put_composition(components.test_closure, self.closures[components.test_closure].members)
        library.flush_units()


def index_entry(components, row):
    """索引里这一行的那一份（只用来说清现判的原因）：键与几样组成的号。"""
    return collections.OrderedDict(
        row_key=components.row_key or "", judging=components.judging or "", toolchain=components.toolchain or "", dependency=components.dependency or "",
        test_closure=components.test_closure or "", text=hashlib.sha256(f"{row.old}\0{row.new}".encode("utf-8", "surrogateescape")).hexdigest(),
        mutated=",".join(mutation_units.unit_id(reference) for reference in components.mutated or []), recorded_date=mutation_units.today_text())


Decision = collections.namedtuple("Decision", "reuse column detail")


def explain_fresh(library, components, row, previous, context):
    """键对不上的一行：(栏, 细节)。照索引里这一行上一次的那一份逐样比（判法单元 → 被改坏的单元 → 点名测试的闭包 → 原文与替换文 → 工具链 → 依赖），
    变了的那几个单元列进细节（文件头「判定库」作废时报变了哪几个单元）。"""
    if previous is None:
        return "row-new", "判定库里没有这一行的记录（新加的行，或改了变异名、文件）"
    if previous.get("row_key") == components.row_key:
        return "not-caught", "上一趟算过这把键而没留下判定（被打断、记录没写成，或判出的不是抓到）"
    if previous.get("judging") != components.judging:
        changes = mutation_units.changed_units(library, previous.get("judging"), context.judging_references)
        return "judging", f"判法单元指纹 {previous.get('judging', '')[:12]}… → {components.judging[:12]}…：{mutation_units.summarize_changes(changes)}"
    current_ids = ",".join(mutation_units.unit_id(reference) for reference in components.mutated)
    if previous.get("mutated") != current_ids:
        names = "、".join(f"{reference.file}::{reference.name}" for reference in components.mutated)
        return "mutated", f"被改坏的单元 {names} 变了"
    if previous.get("test_closure") != components.test_closure:
        closure = context.closures.get(components.test_closure)
        changes = mutation_units.changed_units(library, previous.get("test_closure"), closure.members if closure else [])
        return "test", f"点名测试 {components.test_full_name} 的闭包：{mutation_units.summarize_changes(changes)}"
    if previous.get("text") != hashlib.sha256(f"{row.old}\0{row.new}".encode("utf-8", "surrogateescape")).hexdigest():
        return "text", "这一行的原文或替换文改了"
    if previous.get("toolchain") != components.toolchain:
        return "environment", f"工具链行：{mutation_units.summarize_changes(mutation_units.changed_lines(library, previous.get('toolchain'), context.toolchain_lines))}"
    if previous.get("dependency") != components.dependency:
        return "environment", "依赖版本行变了"
    return "unexplained", "键对不上而几样组成的号都与上一次相同（cargo 参数里的开关或特性改了）"


def row_decision(library, components, row, refusal, context, previous=None, record=None):
    """这一行这一趟复用还是现判（文件头「复用判据」）：(Decision, 记录)。拒算的 column 是 refused。"""
    if components.row_key is None:
        return Decision(False, "refused", components.refusal), None
    if refusal:
        return Decision(False, "start-over", refusal), None
    if record is None:
        record = library.read("records", components.row_key)
    if record is not None and record.get("row_key") == components.row_key:
        if record.get("verdict") == REUSABLE_VERDICT["mutation"]:
            return Decision(True, None, f"键相同（{record.get('judged_date', '日期没记')} {record.get('how', '')} 判的抓到）"), record
        return Decision(False, "not-caught", f"上次判的是 {record.get('verdict') or '（没写）'}，只有抓到的复用"), record
    column, detail = explain_fresh(library, components, row, previous, context)
    return Decision(False, column, detail), None


def executed_set_is_current(keys, coverage):
    """覆盖记录里那一份执行过的单元（执行过的 fn 单元，连同点名测试闭包里不是 fn 的单元：常量、类型、静态项、宏、数据文件）此刻逐个还是那个指纹。"""
    if not isinstance(coverage, dict) or not coverage.get("complete"):
        return False
    references = [mutation_units.UnitReference(*item) for item in coverage.get("executed", [])]
    if not references:
        return False
    current = keys.current_fingerprints(references)
    return all(reference.fingerprint in current.get((reference.kind, reference.file, reference.name), set()) for reference in references)


def baseline_reuse_problem(record, components, keys, coverage, snapshot, environment):
    """覆盖复用判据（基线 ok 那一条复用不复用）：成立交 None，不成立交那一句。判据（按次序，头一条成立就复用）：
      覆盖记录里点名测试执行过的单元此刻逐个没变（executed_set_is_current）；或者点名测试的闭包键与记录里的相同。
    不按整树快照比：同一个文件里加一个没人调的函数、改别的测试，基线照样复用。弄坏开关 baseline-by-tree-snapshot：只按整树快照比（改前的判法）。"""
    if record is None or record.get("verdict") != REUSABLE_VERDICT["baseline"]:
        return "没有基线 ok 的记录"
    if break_is(environment, "baseline-by-tree-snapshot"):
        return None if record.get("tree_snapshot") == snapshot else "源码树快照与上次基线那一份不同（弄坏开关 baseline-by-tree-snapshot）"
    if executed_set_is_current(keys, coverage):
        return None
    if record.get("test_closure") and record.get("test_closure") == components.test_closure:
        return None
    return "点名测试执行过的单元与它的闭包都与上次基线那一份不同"


def read_timing(timings_directory, content_hash):
    """(上一次跑的秒数, 上一次超没超时)；没有记过交 None。"""
    path = os.path.join(timings_directory, content_hash)
    try:
        with open(path, encoding="utf-8", errors="surrogateescape") as handle:
            record = mutation_units.parse_record(handle.read())
    except OSError:
        return None
    mutation_units.Library.touch(path)
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


def baseline_problem(row, returncode, output, memory_max, timeout_seconds, compile_pass=False):
    """这一行的点名测试在基线输出里不是「恰好一个用例、结果都是 ok」时交那一句（接在「在没改坏的副本上」后面），是的交 None。
    compile_pass：交回的是先编那一趟，句子写明是编译那一趟（撞顶说明编译没装进上限，不是测试本身要这么多）。"""
    outcome = wrapper_outcome(returncode, memory_max, timeout_seconds)
    if outcome is not None:
        return f"没判成：那一批{'编译那一趟' if compile_pass else ''}{outcome[1]}"
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


def verdict_of_run(row, returncode, output, memory_max, timeout_seconds, environment=None, compile_pass=False):
    """一条变异跑完之后判哪一档：(档, 那一条要打印的话)。environment 只看弄坏开关（notrunasmissed、memorycompilemerged）。
    compile_pass：这一趟交回的是先编那一趟（run_under_wrapper 的 compile_pass 标记）；包装退 250 时分两档：编译那一趟撞的记 compilememory（改坏之后的代码没编成，
    点名的测试没起，说不上被测代码无界分配），测试那一趟撞的记 memory。弄坏开关 memorycompilemerged：不分，照改前都记 memory。"""
    output = strip_host_lookup_lines(output, environment)   # 包装现查别的机器打的那几行只出提示、不改结局：判法不读（HOST_LOOKUP_LINE）
    tail = output_tail(output)
    outcome = wrapper_outcome(returncode, memory_max, timeout_seconds)
    # 包装报的几种结局先判：被停、被杀的测试二进制可能已经打出了半截输出，不许拿它判抓到或没红
    if outcome is not None:
        verdict, sentence = outcome
        if verdict == "memory" and compile_pass and not break_is(environment or {}, "memorycompilemerged"):
            verdict, sentence = "compilememory", f"编译那一趟撞了内存上限 {memory_max}（内存撞顶）"
        extra = {"timeout": f"，点名的测试 {row.expected} 没来得及红", "memory": f"，点名的测试 {row.expected} 没来得及红",
                 "compilememory": f"，改坏之后的代码没编成、点名的测试 {row.expected} 一次都没起",
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
    """抓到那一行；how 是「（现判：<栏>）」「（复用）」或「（拒算）」，判定本身在它前面（比对两趟的判定时把它去掉，selftest_verdict_view）。"""
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
def own_workers_stopped_on_stop(pool, shard_root, environment):
    """收到停的信号时，在进程池等工作进程收工之前先停自己起的那几个工作进程（只对它们的进程号发 TERM）、等它们退出（每个至多 60 秒），
    再删这一趟自己建的编译目录（remove_owned_build_root：核过没有进程开着里面的文件）与 shard_root，再往外抛。编译目录没删成就连同 shard_root 留着。"""
    try:
        yield
    except StopRequested:
        processes = list((getattr(pool, "_processes", None) or {}).values())
        for process in processes:
            with contextlib.suppress(OSError, ValueError):
                process.terminate()
        for process in processes:
            with contextlib.suppress(OSError, ValueError, AssertionError):
                process.join(60)
        if remove_owned_build_root(shard_root, environment):
            shutil.rmtree(shard_root, ignore_errors=True)
        else:
            note(f"  · 这一趟的临时目录 {shard_root} 留着（里面的编译目录没删）")
        raise


def remove_owned_build_root(shard_root, environment):
    """删这一趟自己建的 <shard_root>/build（文件头「编译目录」；调用方给了 GATE_MUTATION_TARGET_DIR 时没有它）：交回删了或本来就没有。
    没删成（有进程开着里面的文件、删不成）记进 RUN_BUILD_REMOVAL_FAILURES（run_stage 判红）。弄坏开关 runtargetkept：不删、交回 False。"""
    build_root = os.path.join(shard_root, RUN_BUILD_DIRECTORY_NAME)
    if not os.path.lexists(build_root):
        return True
    if break_is(environment, "runtargetkept"):
        note(f"  ! 弄坏开关 GATE_MUTATION_BREAK=runtargetkept：这一趟的编译目录 {build_root} 不删")
        return False
    removed, removal_lines = remove_run_build_root(build_root, environment)
    for line in removal_lines:
        note(line)
    if not removed:
        RUN_BUILD_REMOVAL_FAILURES.append(build_root)
    return removed



def init_worker(sequence_counter, shard_root, context, budget=None):
    """工作进程启动时跑一次：领一个稳定的片号、建自己的源码副本与编译目录。片号从共用编译目录那一层的锁文件领（claim_worker_slot，跨调用不重），
    活是动态领的（checker-tier-crates-mutation-replay 文件头）。sequence_counter 只在拿不到锁文件、或弄坏开关 local-shard-numbering 下当本地序号用。
    budget 是外层额度账（OuterBudget，套在外层内存包装里时有，先编那一趟向它借额度）；外层读不到时是 None。"""
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
    shard_crash_library_home = crash_library_home_for_shard(environment, worker_sequence)
    if shard_crash_library_home is not None:
        environment[CRASH_LIBRARY_HOME_VARIABLE] = shard_crash_library_home
        note(f"  … 工作进程 {os.getpid()} 起 cargo 用的崩溃判定库家目录：{shard_crash_library_home}（每片各一份，不与别的工作进程共用）")
    # 在用的标记（WORKER_TARGET_LOCK_NAME）：进程退出内核就放，别的一趟回收时拿不到排他锁就不删这份
    WORKER_STATE["target_lock"] = hold_worker_target_lock(target_directory)
    # 先编一份依赖（文件头「先编一份依赖」）：先编成的那一份里外部依赖的产物拷进自己的两份编译目录（普通、插桩），工作区的包照旧自己编
    copy_seed_dependencies(context, target_directory)
    # 这份副本的整树快照：只当基线记录的一样属性记下（弄坏开关 baseline-by-tree-snapshot 按它比），不当复用的依据
    snapshot = snapshot_identity(snapshot_of(work, context["copied"]))
    WORKER_STATE.update(work=work, environment=environment, context=context, shard_root=shard_root, worker_sequence=worker_sequence,
                        per_mutation_temporary_root=os.path.join(shard_root, PER_MUTATION_TEMPORARY_DIRECTORY_NAME),
                        snapshot=snapshot, library=mutation_units.Library(context["library_directory"], RECORD_HEADER), unit_keys=None, budget=budget)
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
    """调用方给的第 1 片编译目录（其余片加 -w<片号>，记覆盖的再加 -coverage）：GATE_MUTATION_TARGET_DIR 设了交它（跑完不删）；没设交 None，
    run_pool 在这一趟的 <shard_root>/build 里建（run_build_target_directory）、收尾删。弄坏开关 targetdefaulttmp：没设时照更早的样子交
    ${TMPDIR:-/tmp}/singlefs-crates-mutation-target（跨轮共用、不删）。"""
    explicit = environment.get("GATE_MUTATION_TARGET_DIR", "").strip()
    if explicit:
        return explicit
    if break_is(environment, "targetdefaulttmp"):
        return os.path.join(environment.get("TMPDIR") or "/tmp", TARGET_DIRECTORY_NAME)
    return None


def run_build_target_directory(shard_root):
    """这一趟自己建的第 1 片编译目录：<shard_root>/build/singlefs-crates-mutation-target（别的片与先编那一份都是它在 build 里的兄弟）。"""
    return os.path.join(shard_root, RUN_BUILD_DIRECTORY_NAME, TARGET_DIRECTORY_NAME)


def planned_run_build_target_directory():
    """run_pool 还没建 shard_root 时，磁盘余量按它将来所在的盘算（$TMPDIR 底下；路径此刻不在，disk_free_bytes 往上找到 $TMPDIR）。"""
    return os.path.join(tempfile.gettempdir(), SHARD_ROOT_PREFIX + "planned", RUN_BUILD_DIRECTORY_NAME, TARGET_DIRECTORY_NAME)


def remove_run_build_root(build_root, environment):
    """删这一趟自己建的 <shard_root>/build（文件头「编译目录」）：删之前核本机没有进程开着里面的文件。回 (删成了没有, [要打的行])。
    弄坏开关 runtargetopenfilesunchecked：不核开着的文件就删。"""
    import run_build_directory   # scripts/hosts/run_build_directory.py（模块开头已把 scripts/hosts 放进 sys.path）
    return run_build_directory.remove_run_build_directory(build_root, check_open=not break_is(environment, "runtargetopenfilesunchecked"))


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


def target_reclaim_lines(shared, worker_count, outcome, owned):
    removed, kept, failed = outcome
    lines = [f"  … 编译目录：{shared}（第 1 片；其余片加 -w<片号>，记覆盖的再加 {COVERAGE_TARGET_SUFFIX}；"
             + ("这一趟自己建的，跑完连同 build 整个删" if owned else "调用方给的 GATE_MUTATION_TARGET_DIR，跑完不删") + "）；"
             + (f"回收了片号大于这一趟 {worker_count} 个工作进程的 {len(removed)} 份，共 {format_size(sum(size for _path, size in removed))}"
                f"（{'、'.join(os.path.basename(path) for path, _size in removed)}）" if removed
                else f"片号大于这一趟 {worker_count} 个工作进程、要回收的 0 份")
             + (f"；有进程在用、没删 {len(kept)} 份" if kept else "") + (f"；删不成 {len(failed)} 份" if failed else "")]
    lines += [f"    没删 {path}：{reasons}" for path, reasons in kept]
    lines += [f"    删不成 {path}：{reason}（下一趟开跑时再试；急着要空间就核没人在用之后手删）" for path, reason in failed]
    return lines


def size_setting_problems(environment):
    """GATE_MUTATION_TARGET_DIR_CAP、GATE_MUTATION_DISK_FREE、GATE_MUTATION_DISK_RESERVE 里写法不认的：[(卡在哪, 下一步)]。"""
    problems = []
    for variable in (TARGET_DIRECTORY_CAP_VARIABLE, DISK_FREE_VARIABLE, DISK_RESERVE_VARIABLE):
        text = environment.get(variable, "").strip()
        if text and bytes_of_size(text) is None:
            problems.append((f"{variable} 写成了「{text}」，不是 systemd 的写法",
                             f"写成正整数加 K / M / G / T，单位必写，例 64G（{variable} 的意思见 scripts/mutation/crates-mutation-rows.py 文件头「磁盘余量」「编译目录上限」），或者不设它。"))
    return problems


def target_directory_cap_bytes(environment):
    """每份编译目录的上限字节（GATE_MUTATION_TARGET_DIR_CAP，默认 DEFAULT_TARGET_DIRECTORY_CAP）。"""
    return bytes_of_size(environment.get(TARGET_DIRECTORY_CAP_VARIABLE, "").strip() or DEFAULT_TARGET_DIRECTORY_CAP)


def worker_target_directories(shared, sequence):
    """片号 sequence 的 (普通编译目录, 插桩编译目录)：第 1 片用共用的，其余片加 -w<片号>，插桩的再加 -coverage（与 prepare_shard、judge_coverage 同一套）。"""
    plain = shared if sequence == 1 else f"{shared}-w{sequence}"
    return plain, plain + COVERAGE_TARGET_SUFFIX


def disk_free_bytes(shared, environment):
    """目标盘（编译目录所在那块）此刻的余量：(字节或 None, 说明或没读到的原因)。外层给了 GATE_MUTATION_DISK_FREE 用它，不然读编译目录往上第一层存在的目录所在的盘。"""
    given = environment.get(DISK_FREE_VARIABLE, "").strip()
    if given:
        value = bytes_of_size(given)
        if value is None:
            return None, f"{DISK_FREE_VARIABLE} 写成「{given}」，不是正整数加 K / M / G / T"
        return value, f"外层给的 {DISK_FREE_VARIABLE}={given}"
    probe = os.path.abspath(shared)
    while not os.path.exists(probe) and os.path.dirname(probe) != probe:
        probe = os.path.dirname(probe)
    try:
        status = os.statvfs(probe)
    except OSError as error:
        return None, f"外层没给 {DISK_FREE_VARIABLE}，读编译目录所在的盘也不成（{type(error).__name__}：{error}）"
    return status.f_bavail * status.f_frsize, "编译目录所在的盘此刻可用"


def disk_bound_for(shared, environment, ceiling, peaks):
    """磁盘余量放得下几个工作进程（文件头「磁盘余量」）：(个数或 None, 头一行要写的话)。
    每个工作进程最多占两份编译目录（普通、插桩），每份涨到「每份的涨幅」为止：片号 1…n 各份还能涨的量（涨幅减去此刻已占的，目录还没建的是整个涨幅）
    加起来；n ≥ 2 时先编一份依赖的 -seed、-seed-coverage 两份也按同样的算法加上（每一趟新编）；再加备用 DISK_RESERVE，不超过目标盘余量的最大 n；
    n 从 1 数到 ceiling（前几项取最小的那个数，再多开也用不上）。一个都放不下交 0（调用方按 1 个开）。
    每份的涨幅：peaks（measured_target_peaks 交的 {"普通": 字节或 None, "插桩": 字节或 None}，判定库里近几次每格判完量到的占盘峰值）里有那一种的读数就用它，
    没有读数、peaks 是 None 的按 TARGET_DIRECTORY_CAP 算（上限只当清理线，没实测时才拿它顶）。
    余量没读到、写法不认交 None 与原因，头一行写明。弄坏开关 disknotchecked：不看磁盘（改前的样子）；diskbudgetbycap：不看实测峰值、一律按上限算（改前的样子）。"""
    if break_is(environment, "disknotchecked"):
        return None, "弄坏开关 disknotchecked：不看磁盘余量"
    free, source = disk_free_bytes(shared, environment)
    if free is None:
        return None, source
    cap = target_directory_cap_bytes(environment)
    reserve = bytes_of_size(environment.get(DISK_RESERVE_VARIABLE, "").strip() or DEFAULT_DISK_RESERVE)
    if cap is None or reserve is None:
        return None, f"{TARGET_DIRECTORY_CAP_VARIABLE} 或 {DISK_RESERVE_VARIABLE} 的写法不认"
    cap_text = environment.get(TARGET_DIRECTORY_CAP_VARIABLE, "").strip() or DEFAULT_TARGET_DIRECTORY_CAP
    by_cap = peaks is None or break_is(environment, "diskbudgetbycap")
    growth = {kind: cap if by_cap or peaks.get(kind) is None else peaks[kind] for kind in ("普通", "插桩")}
    growth_text = "、".join(f"{kind}那份涨到 {cap_text}（没有实测读数，按上限）" if by_cap or peaks.get(kind) is None
                            else f"{kind}那份涨到 {format_size(growth[kind])}（近几次每格判完量到的占盘峰值）" for kind in ("普通", "插桩"))

    def room_of(directories):
        return sum(max(0, growth[kind] - allocated_bytes_of_tree(directory)) if os.path.isdir(directory) else growth[kind]
                   for kind, directory in zip(("普通", "插桩"), directories))
    seed_base = os.path.normpath(shared) + SEED_TARGET_SUFFIX
    seed_room = room_of([seed_base, seed_base + COVERAGE_TARGET_SUFFIX])
    count, total, next_total = 0, 0, None
    for sequence in range(1, max(1, ceiling) + 2):   # 多数一个：放得下天花板之外的一个，才不与前几项里最小的那一项并列成「卡在」
        room = room_of(worker_target_directories(shared, sequence)) + (seed_room if sequence == 2 else 0)
        if reserve + total + room > free:
            next_total = total + room
            break
        total += room
        count = sequence
    text = (f"磁盘余量 {format_size(free)}（{source}），留备用 {format_size(reserve)}，每个工作进程最多占两份编译目录（{growth_text}），"
            f"开 2 个以上时先编一份依赖的两份也照这两种涨幅算（已有的目录只算还能涨的）："
            f"{count} 个共还要 {format_size(total)}"
            + (f"，再多 1 个要 {format_size(next_total)}、放不下，所以给 {count} 个" if next_total is not None else f"，前几项的天花板 {ceiling} 个（连同再多 1 个）都放得下，这一项不卡"))
    return count, text


def incremental_directories_of(target_directory):
    """编译目录底下的增量产物目录：<目录>/<档>/incremental 与 <目录>/<目标三元组>/<档>/incremental。"""
    found = []
    try:
        levels = [entry.path for entry in os.scandir(target_directory) if entry.is_dir(follow_symlinks=False)]
    except OSError:
        return found
    for level in sorted(levels):
        candidate = os.path.join(level, INCREMENTAL_DIRECTORY_NAME)
        if os.path.isdir(candidate):
            found.append(candidate)
        try:
            deeper = sorted(entry.path for entry in os.scandir(level) if entry.is_dir(follow_symlinks=False))
        except OSError:
            continue
        found += [os.path.join(path, INCREMENTAL_DIRECTORY_NAME) for path in deeper if os.path.isdir(os.path.join(path, INCREMENTAL_DIRECTORY_NAME))]
    return found


def trim_target_directory(directory, cap_bytes, environment):
    """一份编译目录占盘超过 cap_bytes 时清掉它底下的增量产物（incremental，rustc 每编一次改坏的副本都往里加、只增不减），回 (清前占盘, 清后占盘, [(清掉的目录或文件, 字节)])。
    清完增量仍超线的，再清 deps 里同名产物的旧版本（stale_dependency_artifacts：每个产物只留最新那一版）。
    没超线、目录不在、cap_bytes 是 None 都不动，清前清后交同一个数。清完回读再量一遍，不拿清前减清掉的估。只在两格之间调（run_pool 里每格判完），那时没有 cargo 在这份目录上跑。
    每个产物的最新一版不动，清完仍超线的由调用方报并停派（trim_worker_target）。弄坏开关 targetcapignored：不清（改前的样子）；depsnotpruned：只清增量、不清 deps 里的旧版本。"""
    if break_is(environment, "targetcapignored") or cap_bytes is None or not os.path.isdir(directory):
        return 0, 0, []
    before = allocated_bytes_of_tree(directory)
    if before <= cap_bytes:
        return before, before, []
    removed = []
    for path in incremental_directories_of(directory):
        size = allocated_bytes_of_tree(path)
        shutil.rmtree(path)
        removed.append((path, size))
    after = allocated_bytes_of_tree(directory)
    if after > cap_bytes and not break_is(environment, "depsnotpruned"):
        # 清完增量仍超线：deps 里同名产物只留最新那一版（stale_dependency_artifacts），清完再量（文件头「编译目录上限」）
        for path in stale_dependency_artifacts(directory):
            size = allocated_bytes_of_tree(path)
            os.unlink(path)
            removed.append((path, size))
        after = allocated_bytes_of_tree(directory)
    return before, after, removed


def stale_dependency_artifacts(target_directory):
    """<目录>/<档>/deps 与 <目录>/<目标三元组>/<档>/deps 里同名产物的旧版本：文件名照 cargo 写成「<名字>-<16 位单元哈希>[.<后缀>]」，
    按（目录、名字、后缀）归堆，每堆按修改时刻只留最新的那一份，其余的交出来（路径的列表，按路径排）。名字里不带单元哈希的（cargo 拷到档那一层的工作区产物这一类）不动。
    同一个包换了特性、档或参数编出来的另一版也算同名：留新的、删旧的，之后的格要用旧那一版时 cargo 照「产物不在」重编（慢，判定不变）。"""
    stale, deps_directories = [], []
    try:
        levels = [entry.path for entry in os.scandir(target_directory) if entry.is_dir(follow_symlinks=False)]
    except OSError:
        levels = []
    for level in sorted(levels):
        deps_directories.append(os.path.join(level, "deps"))
        with contextlib.suppress(OSError):
            deps_directories += [os.path.join(entry.path, "deps") for entry in os.scandir(level) if entry.is_dir(follow_symlinks=False)]
    for deps in sorted(set(path for path in deps_directories if os.path.isdir(path))):
        piles = {}
        with contextlib.suppress(OSError):
            for entry in os.scandir(deps):
                found = re.fullmatch(r"(.+)-[0-9a-f]{16}((?:\.[^.]+)*)", entry.name)
                if found is None or not entry.is_file(follow_symlinks=False):
                    continue
                piles.setdefault((found.group(1), found.group(2)), []).append((entry.stat(follow_symlinks=False).st_mtime_ns, entry.path))
        for pile in piles.values():
            stale += [path for _mtime, path in sorted(pile)[:-1]]
    return sorted(stale)


def trim_worker_target(coverage):
    """这个工作进程刚判完一格：它自己的那份编译目录（coverage 为真是插桩那份）占盘超过上限就清增量产物，stderr 报清了什么、清完还剩多少。
    清不成只报，不让这一格的判定因此丢（并行不许把失败吃掉：这里报出来，判定照交）。"""
    context = WORKER_STATE["context"]
    directory = WORKER_STATE["environment"]["CARGO_TARGET_DIR"] + (COVERAGE_TARGET_SUFFIX if coverage else "")
    cap = context.get("target_directory_cap_bytes")
    try:
        before, after, removed = trim_target_directory(directory, cap, context["environment"])
    except OSError as error:
        note(f"  ! 编译目录 {directory} 超过上限时清增量产物没清成（{type(error).__name__}：{error}）；下一格判完再试")
        return
    # 每格判完记一次占盘（清之前量的那个数，文件头「磁盘余量」：下一趟的磁盘预算按近几次的峰值算）；trim 没量的（目录不在、弄坏开关 targetcapignored）这里量
    occupied = before or (allocated_bytes_of_tree(directory) if os.path.isdir(directory) else 0)
    if occupied and context.get("library_directory"):
        problem = record_target_occupancy(context["library_directory"], coverage, occupied)
        if problem is not None:
            note(f"  ! 编译目录占盘没记进 {target_occupancy_file(context['library_directory'])}（{problem}）；磁盘预算少一个读数，不碍这一格的判定")
    if before > (cap or before):
        incremental_removed = [size for path, size in removed if os.path.basename(path) == INCREMENTAL_DIRECTORY_NAME]
        deps_removed = [size for path, size in removed if os.path.basename(path) != INCREMENTAL_DIRECTORY_NAME]
        still_over = after > cap
        note(f"  … 编译目录 {directory} 占盘 {format_size(before)} 超过上限 {format_size(cap)}：清掉增量产物 {len(incremental_removed)} 处共 {format_size(sum(incremental_removed))}"
             + (f"、deps 里同名产物的旧版本 {len(deps_removed)} 份共 {format_size(sum(deps_removed))}" if deps_removed else "")
             + f"，清后 {format_size(after)}"
             + ("；清后仍超上限（剩下的是每个依赖与测试二进制的最新一版）：停派（文件头「编译目录上限」）" if still_over else ""))
        if still_over and not break_is(context["environment"], "overcapdispatched"):
            # 停派（文件头「编译目录上限」）：往这一趟 shard_root 里的停派目录写一份，父进程每收一格看一眼，有就不再派活。弄坏开关 overcapdispatched：只报不停（改前的样子）
            report = (f"第 {WORKER_STATE.get('worker_sequence', '?')} 片的{'插桩' if coverage else '普通'}编译目录 {directory}："
                      f"清前 {format_size(before)}、清后 {format_size(after)}，上限 {format_size(cap)}")
            try:
                write_atomically(os.path.join(target_over_cap_directory(WORKER_STATE["shard_root"]), f"{os.getpid()}-{'coverage' if coverage else 'plain'}"), report + "\n")
            except OSError as error:
                note(f"  ! 停派的标记没写成（{type(error).__name__}：{error}）：父进程照旧派活")


def target_over_cap_directory(shard_root):
    """这一趟的停派目录（文件头「编译目录上限」）：工作进程清完仍超上限往里写一份，run_pool 每收一格看一眼。"""
    return os.path.join(shard_root, "target-over-cap")


def target_over_cap_reports(shard_root):
    """停派目录里每一份写的那一句（按文件名排）；目录不在或读不了的交 []。"""
    reports = []
    with contextlib.suppress(OSError):
        for name in sorted(os.listdir(target_over_cap_directory(shard_root))):
            with contextlib.suppress(OSError):
                with open(os.path.join(target_over_cap_directory(shard_root), name), encoding="utf-8", errors="replace") as handle:
                    reports.append(handle.read().strip())
    return reports


def target_occupancy_file(library_directory):
    """每格判完量到的编译目录占盘记在判定库目录下这一份（文件头「磁盘余量」）：一行「<时刻秒><TAB>普通|插桩<TAB><字节>」，只追加。"""
    return os.path.join(library_directory, "target-occupancy.tsv")


def record_target_occupancy(library_directory, coverage, size_bytes):
    """追加一行（O_APPEND，一行远小于 PIPE_BUF，几个工作进程同时追加不交错）；写进回 None，写不进回原因。"""
    line = f"{int(time.time())}\t{'插桩' if coverage else '普通'}\t{size_bytes}\n".encode()
    try:
        descriptor = os.open(target_occupancy_file(library_directory), os.O_WRONLY | os.O_APPEND | os.O_CREAT, 0o644)
        try:
            os.write(descriptor, line)
        finally:
            os.close(descriptor)
    except OSError as error:
        return f"{type(error).__name__}：{error}"
    return None


def measured_target_peaks(library_directory, recent):
    """{"普通": 字节或 None, "插桩": 字节或 None}：两种编译目录各取最近 recent 次记录里最大的（文件头「磁盘余量」）；文件不在、读不了的都是 None，写坏的行不算。
    记录多过 recent 的 50 倍时只留每种最近 recent 次，同目录临时文件改名换上（几趟同时用一份库时可能丢几个读数，只是预算少几个样本）。"""
    kept = {"普通": [], "插桩": []}
    path = target_occupancy_file(library_directory)
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            lines = handle.read().splitlines()
    except OSError:
        lines = []
    for line in lines:
        fields = line.split("\t")
        if len(fields) == 3 and fields[1] in kept and fields[2].isdigit():
            kept[fields[1]].append((line, int(fields[2])))
    if len(lines) > 50 * recent:
        survivors = sorted((entry for values in kept.values() for entry in values[-recent:]), key=lambda entry: entry[0].split("\t")[0])
        with contextlib.suppress(OSError):
            write_atomically(path, "".join(entry[0] + "\n" for entry in survivors))
    return {kind: max(size for _line, size in values[-recent:]) if values else None for kind, values in kept.items()}


# ── 先编一份依赖（文件头「先编一份依赖」） ───────────────────────────────────────

def seed_group_of(before):
    """一行 cargo test 参数里 `--` 之前那一段 → (组的键, 先编这一组用的参数)。键只留选包、特性、档、目标三元组与别的开关（外部依赖编成什么只随它们变），
    选目标的开关（--test <名>、--lib……）、TESTNAME 过滤词与只管输出的开关（-q、-j……）不进键；先编用的参数是键里那几样加这一行自己选的目标。"""
    key, selectors, index = [], [], 0
    while index < len(before):
        word = before[index]
        name, equals, _value = word.partition("=")
        width = 1 if equals else 2
        if name in SEED_VALUE_SELECTOR_SWITCHES:
            selectors += before[index:index + width]
            index += width
        elif name in SEED_SELECTOR_SWITCHES:
            selectors.append(word)
            index += 1
        elif name in SEED_VALUE_DROPPED_SWITCHES:
            index += width
        elif name in SEED_DROPPED_SWITCHES or not word.startswith("-"):
            index += 1
        elif name in SEED_VALUE_KEPT_SWITCHES:
            key += before[index:index + width]
            index += width
        else:
            key.append(word)
            index += 1
    return tuple(key), key + selectors


def read_lock_file_packages(root):
    """根下 Cargo.lock 的 [[package]]（读不了、读不懂、没有交空表）。"""
    try:
        with open(os.path.join(root, "Cargo.lock"), "rb") as handle:
            packages = tomllib.load(handle).get("package", [])
    except (OSError, tomllib.TOMLDecodeError):
        return []
    return [package for package in packages if isinstance(package, dict) and isinstance(package.get("name"), str)]


def path_packages_reaching_external(root):
    """根下 Cargo.lock 里每个 path 包（没有 source 的）顺 dependencies 够不够得着带 source 的包：{包名: 真或假}。可选依赖照算（Cargo.lock 不分特性）。"""
    packages = read_lock_file_packages(root)
    by_name = collections.defaultdict(list)
    for package in packages:
        by_name[package["name"]].append(package)

    def resolved(reference):
        name, _space, rest = str(reference).partition(" ")
        version = rest.split(" ")[0] if rest else None
        return [package for package in by_name.get(name, []) if version is None or package.get("version") == version]
    reaching = {}
    for package in packages:
        if "source" in package:
            continue
        seen, stack, found = set(), [package], False
        while stack and not found:
            for dependency in (item for reference in stack.pop().get("dependencies", []) for item in resolved(reference)):
                identity = (dependency["name"], dependency.get("version"), dependency.get("source"))
                if identity in seen:
                    continue
                seen.add(identity)
                if "source" in dependency:
                    found = True
                    break
                stack.append(dependency)
        reaching[package["name"]] = reaching.get(package["name"], False) or found
    return reaching


def seed_groups(rows, reaching):
    """这一趟要先编的组：{组的键: 先编这一组用的参数}，按行的次序；选的包（-p，没写是整个工作区）都够不着外部依赖的组不编。"""
    groups = collections.OrderedDict()
    for row in rows:
        key, arguments = seed_group_of(split_at_double_dash(row.arguments)[0])
        if key in groups:
            continue
        packages = [key[index + 1] for index, word in enumerate(key[:-1]) if word in ("-p", "--package")]
        packages += [word.partition("=")[2] for word in key if word.startswith(("-p=", "--package="))]
        if any(reaching.get(package, False) for package in (packages or list(reaching))):
            groups[key] = arguments
    return groups


@contextlib.contextmanager
def seed_lock(shared, exclusive):
    """先编那一份的锁：建（同步源码、编）的时候排他，工作进程拷的时候共享；拿不到就等。"""
    import fcntl
    path = os.path.normpath(shared) + SEED_LOCK_SUFFIX
    os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
    with open(path, "a") as handle:
        fcntl.flock(handle, fcntl.LOCK_EX if exclusive else fcntl.LOCK_SH)
        try:
            yield
        finally:
            fcntl.flock(handle, fcntl.LOCK_UN)


def files_under(base, item):
    """base 底下 item（一个文件或一个目录）里的文件，相对 base；跳过 IGNORED_WHEN_COPYING_INTO_EACH_SHARD 的目录，不在交空表。"""
    top = os.path.join(base, item)
    if not os.path.isdir(top):
        return [item] if os.path.lexists(top) else []
    found = []
    for directory, subdirectories, file_names in os.walk(top):
        subdirectories[:] = sorted(name for name in subdirectories
                                   if not any(fnmatch.fnmatch(name, pattern) for pattern in IGNORED_WHEN_COPYING_INTO_EACH_SHARD))
        found += [os.path.relpath(os.path.join(directory, name), base) for name in sorted(file_names)]
    return found


def package_directories(source):
    """源码副本里的包目录（相对副本根）：crates/ 底下一层、有 Cargo.toml 的。"""
    crates = os.path.join(source, "crates")
    try:
        names = sorted(os.listdir(crates))
    except OSError:
        return set()
    return {f"crates/{name}" for name in names if os.path.isfile(os.path.join(crates, name, "Cargo.toml"))}


def sync_seed_source(root, copied, seed_source, environment):
    """先编那一份的源码副本按内容同步：内容没变的文件不动；变了、新加的写进去（修改时刻是此刻），副本里有、拷贝范围里没有的删掉；
    改动所在的包整包把修改时刻摸成此刻，不在哪个包底下的改动摸全部包。回 (摸过的包目录的集合, 改动的文件数)。
    弄坏开关 seedmtimekept：内容变了的文件写进去之后修改时刻改回原来的、一个包都不摸（摸过的包照样交回，给 stale_seed_artifacts 核）。"""
    items = [item for item in copied if not (item == CARGO_CONFIGURATION_DIRECTORY and break_is(environment, "cargoconfignotcopied"))]
    wanted = sorted({relative for item in items for relative in files_under(root, item)})
    os.makedirs(seed_source, exist_ok=True)
    changed = []
    for relative in wanted:
        source, target = os.path.join(root, relative), os.path.join(seed_source, relative)
        with open(source, "rb") as handle:
            data = handle.read()
        previous = None
        if os.path.isdir(target) and not os.path.islink(target):
            shutil.rmtree(target)
        elif os.path.lexists(target):
            with open(target, "rb") as handle:
                if handle.read() == data:
                    continue
            previous = os.stat(target)
        os.makedirs(os.path.dirname(target), exist_ok=True)
        temporary = target + SEED_COPY_TEMPORARY_SUFFIX
        with open(temporary, "wb") as handle:
            handle.write(data)
        shutil.copymode(source, temporary)
        os.replace(temporary, target)
        if previous is not None and break_is(environment, "seedmtimekept"):
            os.utime(target, ns=(previous.st_atime_ns, previous.st_mtime_ns))
        changed.append(relative)
    tops = {item.split("/")[0] for item in items}
    for name in sorted(os.listdir(seed_source)):
        if name not in tops:
            path = os.path.join(seed_source, name)
            changed += files_under(seed_source, name) or [name]
            shutil.rmtree(path) if os.path.isdir(path) and not os.path.islink(path) else os.unlink(path)
    for relative in sorted(set(relative for item in items for relative in files_under(seed_source, item)) - set(wanted)):
        os.unlink(os.path.join(seed_source, relative))
        changed.append(relative)
    packages = package_directories(seed_source)
    touched = set()
    for relative in changed:
        parts = relative.split("/")
        package = "/".join(parts[:2]) if len(parts) > 2 and parts[0] == "crates" else None
        touched |= {package} if package in packages else packages
    if not break_is(environment, "seedmtimekept"):
        now = time.time_ns()
        for package in sorted(touched):
            for relative in files_under(seed_source, package):
                os.utime(os.path.join(seed_source, relative), ns=(now, now), follow_symlinks=False)
    return touched, len(changed)


def seed_unit_identity(message):
    """一条 compiler-artifact 消息 → 编译单元的身份（JSON 文本）：package_id、manifest_path、目标的 kind、crate_types、name，档、特性、filenames、executable。
    filenames 里 deps/ 下的文件名带 cargo 按包、特性、档、目标三元组、rustc 参数与依赖算的单元哈希（-C metadata），同一份编译目录里两个单元不会同名；
    同包另一套特性或档编出来的单元哈希不同、身份也不同。其余几样在 filenames 为空（只检查）、或消息缺字段时垫底。"""
    target = message.get("target") or {}
    return json.dumps([message.get("package_id"), message.get("manifest_path"), target.get("kind"), target.get("crate_types"), target.get("name"),
                       message.get("profile"), sorted(message.get("features") or []), message.get("filenames"), message.get("executable")], sort_keys=True)


def stale_seed_artifacts(output, seed_source, touched, compiled=None):
    """先编一组的 cargo JSON 输出里，摸过的包（touched）编出来而 fresh 为真的产物：[「<包目录> 的 <目标名>」…]。
    compiled 是同一份编译目录（普通或插桩）里前面的组已经以非 fresh 编出过的单元身份（seed_unit_identity）的集合，这一组里非 fresh 的往里加；
    fresh 的产物身份在里面的是复用前面的组编出来的，不算坏；头一次出现就是 fresh 的才算。不给（None）是只看这一组自己（弄坏开关 seedfreshacrossgroups 的样子）。"""
    root = os.path.realpath(seed_source)
    compiled = set() if compiled is None else compiled
    stale = []
    for line in output.splitlines():
        if not line.startswith("{"):
            continue
        try:
            message = json.loads(line)
        except ValueError:
            continue
        if not isinstance(message, dict) or message.get("reason") != "compiler-artifact":
            continue
        identity = seed_unit_identity(message)
        if not message.get("fresh"):
            compiled.add(identity)
            continue
        if identity in compiled:
            continue
        package = os.path.relpath(os.path.dirname(os.path.realpath(message.get("manifest_path") or "/")), root)
        if package in touched:
            stale.append(f"{package} 的 {(message.get('target') or {}).get('name', '?')}")
    return stale


def seed_limits(context):
    """先编那一份的 (内存上限的 systemd 写法, CARGO_BUILD_JOBS)：外层读得到用整个外层，读不到用每条的默认上限 × 工作进程数；
    并行度 min(核数, 上限 ÷ COMPILE_GIB_PER_JOB)，GATE_MUTATION_CARGO_JOBS 设了用它。"""
    outer = context.get("outer_memory_bytes")
    cap = outer if outer else (bytes_of_size(context["memory_max"]) or 0) * context["worker_count"]
    given = context["environment"].get("GATE_MUTATION_CARGO_JOBS", "").strip()
    return size_text_of_bytes(cap), given or str(max(1, min(os.cpu_count() or 4, int(cap // (COMPILE_GIB_PER_JOB * 1024 ** 3)))))


def build_seed_group(context, command, seed_source, extra_environment, cap, label, temporary_root, instrumented):
    """先编一组（经内存包装、不限时）：subprocess 的结果。TMPDIR 是这一组自己的，插桩那一份的构建脚本与过程宏的剖析文件也落在里面，编完整个删。"""
    temporary = tempfile.mkdtemp(prefix="seed-", dir=temporary_root)
    try:
        environment = dict(context["environment"], TMPDIR=temporary, RUN_WITH_MEMORY_CAP_KEY=" ".join(command), **extra_environment)
        environment.pop("RUN_WITH_MEMORY_CAP_TIME_LIMIT", None)
        if instrumented:
            environment["LLVM_PROFILE_FILE"] = os.path.join(temporary, "build-%p-%m.profraw")
        return run_forwarding_queue_lines(["bash", context["memory_cap_runner"], cap] + command, label, cwd=seed_source, env=environment)
    finally:
        shutil.rmtree(temporary, ignore_errors=True)


def prepare_seed(context, batches, ordered_rows, coverage_planner, temporary_root):
    """派活之前先编一份依赖：交 (要报的话, 判红的那几句)。编成的记进 context["seed_targets"]（{"plain" 或 "coverage": 编译目录}），
    工作进程起来时照它拷（copy_seed_dependencies）；没起、没编成的不在里面。弄坏开关 compile-per-worker：不起（改前的样子）。"""
    context["seed_targets"] = {}
    environment = context["environment"]
    if break_is(environment, "compile-per-worker"):
        return ["  … 先编一份依赖：没起（弄坏开关 compile-per-worker：每个工作进程自己编外部依赖）"], []
    if context["worker_count"] < 2:
        return [f"  … 先编一份依赖：没起（这一趟只开 {context['worker_count']} 个工作进程，没有别的工作进程共用）"], []
    reaching = path_packages_reaching_external(context["root"])
    plain = seed_groups([row for _key, batch_rows in batches for row in batch_rows] + list(ordered_rows), reaching)
    coverage = seed_groups(coverage_planner.predicted_rows(ordered_rows), reaching)
    if not plain and not coverage:
        return ["  … 先编一份依赖：没起（要现跑的行选的包顺 Cargo.lock 都够不着外部依赖" + ("" if reaching else "；树里没有读得懂的 Cargo.lock") + "）"], []
    shared = os.path.normpath(context["shared_target_directory"])
    seed_source = shared + SEED_SOURCE_SUFFIX
    cap, jobs = seed_limits(context)
    lines, problems = [], []
    declared = declare_seed_serial(environment, temporary_root, len(plain) + len(coverage), jobs, cap, lines)
    try:
        with seed_lock(shared, exclusive=True):
            touched, changed_count = sync_seed_source(context["root"], context["copied"], seed_source, environment)
            lines.append(f"  … 先编一份依赖：源码副本 {seed_source} 同步了 {changed_count} 个文件（摸了整包的 {len(touched)} 个"
                         + (f"：{'、'.join(sorted(touched))}" if touched else "") + f"）；内存上限 {cap}、并行度 {jobs}")
            overrides, configuration = coverage_build_settings(environment)
            for kind, groups, target, extra, words in (("plain", plain, shared + SEED_TARGET_SUFFIX, {}, []),
                                                       ("coverage", coverage, shared + SEED_TARGET_SUFFIX + COVERAGE_TARGET_SUFFIX, overrides, configuration)):
                if not groups:
                    continue
                failed = []
                compiled_units = set()
                for arguments in groups.values():
                    command = ["cargo", *words, "test", "--offline", *arguments, "--no-run", "--message-format=json-render-diagnostics"]
                    run = build_seed_group(context, command, seed_source, dict(extra, CARGO_TARGET_DIR=target, CARGO_BUILD_JOBS=jobs), cap,
                                           f"先编一份依赖（{'插桩' if kind == 'coverage' else '普通'}：{' '.join(arguments)}）", temporary_root, kind == "coverage")
                    stale = [] if break_is(environment, "seedfreshunchecked") else stale_seed_artifacts(
                        run.stdout, seed_source, touched, None if break_is(environment, "seedfreshacrossgroups") else compiled_units)
                    problems += [f"{' '.join(command)}：{item} 是 fresh" for item in stale]
                    if run.returncode != 0:
                        failed.append(f"{' '.join(arguments)} 退 {run.returncode}：{output_tail(run.stderr, 3)}")
                with contextlib.suppress(OSError):
                    trim_target_directory(target, context.get("target_directory_cap_bytes"), environment)
                if failed:
                    lines.append(f"  ! 先编一份依赖（{'插桩' if kind == 'coverage' else '普通'}，{target}）{len(groups)} 组里 {len(failed)} 组没编成，"
                                 f"这一趟各工作进程自己编依赖：{'；'.join(failed[:3])}")
                else:
                    context["seed_targets"][kind] = target
                    lines.append(f"  … 先编一份依赖（{'插桩' if kind == 'coverage' else '普通'}）：{target} 编了 {len(groups)} 组，工作进程起来时拷其中外部依赖的产物")
    except OSError as error:
        # 同步源码、编、清增量产物时读写出错：这一趟不用先编那一份（工作进程照旧自己编依赖），判定不受影响
        context["seed_targets"] = {}
        lines.append(f"  ! 先编一份依赖没做成（{type(error).__name__}：{error}）：这一趟各工作进程自己编依赖")
    finally:
        if declared is not None:
            append_serial_declaration(declared, "此刻并行", lines)
    return lines, problems


def append_serial_declaration(path, body, lines):
    """往串行段声明文件追加一行「<秒>\\t<body>」（写法同 scripts/hosts/core_usage.py 文件头「声明」）；写不进往 lines 里报一行、交 False。"""
    try:
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(f"{time.time():.3f}\t{body}\n")
        return True
    except OSError as error:
        lines.append(f"  ! 串行段声明写不进 {path}（{type(error).__name__}：{error}）：看门狗看不到先编那一段的理由，照报核数过少")
        return False


def declare_seed_serial(environment, temporary_root, group_count, jobs, cap, lines):
    """先编那一段开编之前声明串行段（文件头「先编一份依赖」的「串行段声明」）：交声明文件的路径（编完照它写「此刻并行」），没写成交 None。
    调用方没设 SINGLEFS_SERIAL_DECLARATIONS 时设成 temporary_root（这一趟的 shard_root）里的 serial-declarations，并放进 environment（起 cargo 的那一份）。
    弄坏开关 seed-serial-undeclared：不声明。"""
    if break_is(environment, "seed-serial-undeclared"):
        lines.append("  … 先编一份依赖：没声明串行段（弄坏开关 seed-serial-undeclared）")
        return None
    # 变量名、文件名与两种标记都写成字面量、不另立模块级常量：模块级赋值进判法单元的闭包（scripts/gate/lib_function_units.py 文件头「import 时执行的模块级语句」）
    path = environment.get("SINGLEFS_SERIAL_DECLARATIONS", "").strip()
    if not path:
        path = os.path.join(temporary_root, "serial-declarations")
        environment["SINGLEFS_SERIAL_DECLARATIONS"] = path
    reason = (f"先编一份依赖：{group_count} 组共用一份编译目录、cargo 锁着它逐组编，组与组不能同时编；"
              f"每组的并行度 {jobs} 已按内存上限 {cap} ÷ {COMPILE_GIB_PER_JOB} GiB 与核数取小")
    if not append_serial_declaration(path, "此刻串行：" + reason, lines):
        return None
    lines.append(f"  … 先编一份依赖：串行段声明写进 {path}（{reason}）；编完写「此刻并行」")
    return path


def dependency_artifact_entries(seed_target, seed_source, environment):
    """先编那一份编译目录里外部依赖的产物：相对 seed_target 的路径（文件或目录）。每一档（底下有 .fingerprint 的那一层：<目录>/<档>、<目录>/<三元组>/<档>）里
    .fingerprint/、build/ 下包名不是 Cargo.lock 里 path 包的目录；deps/ 下按单元哈希归堆，那一堆恰好一份 .d、里面列的源文件都是绝对路径、都不在源码副本底下的那一堆。
    incremental/ 与档那一层的文件（cargo 拷上来的工作区产物）不拷。弄坏开关 workspaceunitsshared：.fingerprint/、build/、deps/ 底下的全拷。"""
    path_names = {package["name"] for package in read_lock_file_packages(seed_source) if "source" not in package}
    root = os.path.realpath(seed_source)
    everything = break_is(environment, "workspaceunitsshared")
    levels = []
    for name in sorted(os.listdir(seed_target)) if os.path.isdir(seed_target) else []:
        level = os.path.join(seed_target, name)
        if os.path.isdir(os.path.join(level, ".fingerprint")):
            levels.append(name)
        elif os.path.isdir(level):
            levels += [os.path.join(name, deeper) for deeper in sorted(os.listdir(level)) if os.path.isdir(os.path.join(level, deeper, ".fingerprint"))]
    entries = []
    for level in levels:
        for kind in (".fingerprint", "build"):
            base = os.path.join(seed_target, level, kind)
            for name in sorted(os.listdir(base)) if os.path.isdir(base) else []:
                matched = UNIT_DIRECTORY_NAME.match(name)
                if matched and (everything or matched.group("name") not in path_names):
                    entries.append(os.path.join(level, kind, name))
        deps = os.path.join(seed_target, level, "deps")
        units = collections.defaultdict(list)
        for name in sorted(os.listdir(deps)) if os.path.isdir(deps) else []:
            matched = UNIT_ARTIFACT_NAME.match(name)
            if matched:
                units[matched.group("hash")].append(name)
        for names in units.values():
            dep_infos = [name for name in names if name.endswith(".d")]
            sources = dependency_info_sources(os.path.join(deps, dep_infos[0])) if len(dep_infos) == 1 else []
            if everything or (sources and all(os.path.isabs(source) and not os.path.realpath(source).startswith(root + os.sep) for source in sources)):
                entries += [os.path.join(level, "deps", name) for name in names]
    return entries


def dependency_info_sources(path):
    """rustc 写的 .d 头一条规则冒号后面的源文件（反斜杠转义的空格还原）；读不了交空表。"""
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            text = handle.read().replace("\\\n", " ")
    except OSError:
        return []
    _target, _colon, rest = text.split("\n", 1)[0].partition(": ")
    return [word.replace("\\ ", " ") for word in re.split(r"(?<!\\) ", rest) if word]


def sync_dependency_artifacts(seed_target, destination, seed_source, environment, threads):
    """把先编那一份里外部依赖的产物（dependency_artifact_entries）拷进 destination：大小与修改时刻都对得上的不拷，拷的保留修改时刻、同目录临时文件写完改名。
    要拷的文件分到 threads 条线程上同时拷（弄坏开关 seed-copy-serial：照改前在一条线程上逐个拷）；哪一个拷不成，OSError 照抛给调用方。回 (拷了几个文件, 几字节)。"""
    pending = dependency_copy_jobs(seed_target, destination, seed_source, environment)
    copy_dependency_files(pending, threads, environment)
    return len(pending), sum(size for _path, _target, size in pending)


def copy_one_dependency_file(job):
    path, target, _size = job
    os.makedirs(os.path.dirname(target), exist_ok=True)
    temporary = target + SEED_COPY_TEMPORARY_SUFFIX
    shutil.copy2(path, temporary)
    os.replace(temporary, target)


def copy_dependency_files(pending, threads, environment):
    """pending 里的 (源, 目标, 字节) 逐个拷：threads 大于 1 时分到那么多条线程上同时拷，出错的头一个原样抛（并行不许把失败吃掉）。"""
    count = 1 if break_is(environment, "seed-copy-serial") else max(1, min(int(threads or 1), len(pending)))
    if count == 1:
        for job in pending:
            copy_one_dependency_file(job)
        return
    with concurrent.futures.ThreadPoolExecutor(max_workers=count) as pool:
        for future in [pool.submit(copy_one_dependency_file, job) for job in pending]:
            future.result()


def seed_copy_threads(context):
    """一个工作进程拷依赖产物用几条线程：CARGO_BUILD_JOBS（run_stage 按核数 ÷ 工作进程数设的），读不出按核数 ÷ 工作进程数。"""
    given = str(context["environment"].get("CARGO_BUILD_JOBS", "")).strip()
    if given.isdigit() and int(given) > 0:
        return int(given)
    return max(1, (os.cpu_count() or 4) // max(1, int(context.get("worker_count") or 1)))


def dependency_copy_jobs(seed_target, destination, seed_source, environment):
    """先编那一份里外部依赖的产物里要拷进 destination 的文件：[(源, 目标, 字节)]；大小与修改时刻都对得上的不在里面。"""
    pending = []
    for relative in dependency_artifact_entries(seed_target, seed_source, environment):
        source = os.path.join(seed_target, relative)
        paths = ([os.path.join(directory, name) for directory, _subdirectories, names in os.walk(source) for name in names]
                 if os.path.isdir(source) else [source])
        for path in paths:
            status = os.lstat(path)
            if not stat.S_ISREG(status.st_mode):
                continue
            target = os.path.join(destination, os.path.relpath(path, seed_target))
            with contextlib.suppress(FileNotFoundError):
                existing = os.lstat(target)
                if existing.st_size == status.st_size and existing.st_mtime_ns == status.st_mtime_ns:
                    continue
            pending.append((path, target, status.st_size))
    return pending


def copy_seed_dependencies(context, target_directory):
    """工作进程起来时（init_worker）：先编成的那一份（普通、插桩）里外部依赖的产物拷进自己的两份编译目录，拿着先编那一份的共享锁拷。拷不成只报，自己编依赖。"""
    seeds = context.get("seed_targets") or {}
    if not seeds:
        return
    shared = os.path.normpath(context["shared_target_directory"])
    environment = context["environment"]
    threads = 1 if break_is(environment, "seed-copy-serial") else seed_copy_threads(context)
    reports, pending = [], []
    try:
        with seed_lock(shared, exclusive=False):
            # 普通、插桩两份要拷的文件先列全，再一起分到 threads 条线程上拷（文件头「先编一份依赖」的「拷」）
            for kind, destination in (("plain", target_directory), ("coverage", target_directory + COVERAGE_TARGET_SUFFIX)):
                if kind in seeds:
                    jobs = dependency_copy_jobs(seeds[kind], destination, shared + SEED_SOURCE_SUFFIX, environment)
                    pending += jobs
                    reports.append(f"{'插桩' if kind == 'coverage' else '普通'} {len(jobs)} 个文件 {format_size(sum(size for _p, _t, size in jobs))}")
            copy_dependency_files(pending, threads, environment)
    except OSError as error:
        note(f"  ! 工作进程 {os.getpid()} 从先编那一份拷依赖产物没拷成（{type(error).__name__}：{error}）：这一片自己编依赖")
        return
    note(f"  … 工作进程 {os.getpid()} 从先编那一份拷来外部依赖的产物：{'；'.join(reports)}（{threads} 条线程同时拷；大小与修改时刻对得上的不拷）")


def baseline_job(batch):
    """进程池里的一格基线：判完清一次普通编译目录（trim_worker_target）。"""
    result = judge_batch(batch)
    trim_worker_target(False)
    return result


def mutation_job(row):
    """进程池里的一格变异：判完清一次普通编译目录。"""
    result = judge_row(row)
    trim_worker_target(False)
    return result


def coverage_job(job):
    """进程池里的一格覆盖：判完清一次插桩编译目录。"""
    result = judge_coverage(job)
    trim_worker_target(True)
    return result


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
# 内存包装排队时现查别的机器（HostLookup，包装文件头「排队时去找别的机器」「整体兜底」）打的四种行首：只出提示、不改结局，判法读包装输出之前去掉
# （strip_host_lookup_lines）。包装里 HostLookup 新加一处 say 而这里不认，自证 host-lookup-say-sites-covered 那一格红。
# 「  ! 告警记录写不进」与包装 append_outcome_record 打的那一句同首，一起去掉（HOST_LOOKUP_ALSO_MATCHED）。
# 这一段在 scripts/gate/verus-proof-check.py、scripts/experiment-apparatus/e165-pin-red-examples.py、scripts/mutation/crates-mutation-rows.py 各一份、照同一份写，各自的自证核自己那一份。
HOST_LOOKUP_LINE = re.compile(r"^(?:run-with-memory-cap: (?:没查到别的机器|现查别的机器)：|  → .+?此刻放得下（|  ! 告警记录写不进（)")
HOST_LOOKUP_ALSO_MATCHED = {"append_outcome_record": "包装收尾写告警记录写不进时打的同一句，只出提示、不改结局"}


def strip_host_lookup_lines(text, environment=None):
    """去掉内存包装 HostLookup 打的那几行（HOST_LOOKUP_LINE）；environment 只看弄坏开关：hostlookuplineskept 下原样交回。"""
    if break_is(environment or {}, "hostlookuplineskept"):
        return text
    return "".join(line for line in text.splitlines(keepends=True) if not HOST_LOOKUP_LINE.match(line))


def host_lookup_say_problems(wrapper_path, pattern=HOST_LOOKUP_LINE):
    """包装内嵌 python（3<<'PY' 那一段）里 class HostLookup 的每一处 say(…)：拼出的那一行（f 串的插值换成 X；say 一个名字的，取本函数里给它赋的值）要被 pattern 认；
    HostLookup 之外的 say(…) 与 print(…) 被它认的，只许是 HOST_LOOKUP_ALSO_MATCHED 点名的函数。交问题清单（空 = 都对）。"""
    with open(wrapper_path, encoding="utf-8") as handle:
        text = handle.read()
    body = next((block for block in re.findall(r"3<<'PY'\n(.*?)\nPY\n", text, re.S) if "class HostLookup" in block), None)
    if body is None:
        return ["包装里找不到含 class HostLookup 的内嵌 python（3<<'PY' … PY）：包装改了写法，这一格的找法要跟着改"]
    module = ast.parse(body)
    problems, found = [], []

    def sample(node, function):
        if isinstance(node, ast.Name) and function is not None:
            values = [statement.value for statement in ast.walk(function) if isinstance(statement, ast.Assign)
                      and any(isinstance(target, ast.Name) and target.id == node.id for target in statement.targets)]
            node = values[-1] if values else node
        if isinstance(node, ast.Constant) and isinstance(node.value, str):
            return node.value
        if isinstance(node, ast.JoinedStr):
            return "".join(part.value if isinstance(part, ast.Constant) else "X" for part in node.values)
        return None

    def visit(node, owner, function):
        for child in ast.iter_child_nodes(node):
            child_owner = child.name if isinstance(child, ast.ClassDef) and owner is None else owner
            child_function = child if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef)) else function
            if isinstance(child, ast.Call) and isinstance(child.func, ast.Name) and child.func.id in ("say", "print") and child.args:
                line = sample(child.args[0], function)
                where = f"{owner + '.' if owner else ''}{function.name if function is not None else '<模块>'}（内嵌 python 第 {child.lineno} 行）"
                if owner == "HostLookup" and child.func.id == "say":
                    found.append(where)
                    if line is None:
                        problems.append(f"{where} 的 say 拼不出那一行（不是串、f 串或本函数里赋过值的名字）")
                    elif not pattern.match(line):
                        problems.append(f"{where} 的 say 打「{line[:80]}」，HOST_LOOKUP_LINE 不认")
                elif owner != "HostLookup" and line is not None and pattern.match(line) and (function is None or function.name not in HOST_LOOKUP_ALSO_MATCHED):
                    problems.append(f"{where} 打「{line[:80]}」不归 HostLookup，HOST_LOOKUP_LINE 却认它（去多了）")
            visit(child, child_owner, child_function)

    visit(module, None, None)
    if not found:
        problems.append("HostLookup 里一处 say 都没找到：包装改了写法，这一格的找法要跟着改")
    return problems


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
    try:
        stdout = process.stdout.read()
        process.wait()
    except BaseException:
        # 父进程里被停（run_pool 接住的 TERM、HUP，先编一份依赖那几趟在父进程里跑）：只停自己起的这一个包装，等它退了再往外抛
        process.terminate()
        try:
            process.wait(timeout=TIMEOUT_KILL_GRACE_SECONDS)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
        raise
    reader.join()
    return subprocess.CompletedProcess(command, process.returncode, stdout, "".join(stderr_lines))


def gpu_quiesce_prefix(command, environment):
    """这一次起的 cargo 要不要经清场包装（文件头「带 gpu 特性的格经清场起」）：cargo test 的 `--` 之前开了 gpu 特性（--features / -F 里有 gpu 或 <包>/gpu，
    或 --all-features），而且真跑测试（不带 --no-run：只编不跑的不上卡），交 ["bash", <清场包装>, "--"]；别的交 []。
    清场包装取 GATE_MUTATION_GPU_QUIESCE_RUNNER（自证换成它自己的），没设取本模块所在仓的 scripts/crash/gpu-quiesce-run.sh。
    弄坏开关 gpu-rows-unquiesced：一律交 []（改前的样子：带 gpu 特性的也只经内存包装起）。"""
    if break_is(environment, "gpu-rows-unquiesced") or not gpu_test_run(command, environment):
        return []
    return ["bash", environment.get("GATE_MUTATION_GPU_QUIESCE_RUNNER", "").strip() or os.path.join(SCRIPT_DIRECTORY, "..", "crash", "gpu-quiesce-run.sh"), "--"]


def gpu_test_run(command, environment=None):
    """这一次起的是不是带 gpu 特性、真跑测试的 cargo test：`--` 之前开了上卡的特性（--features / -F 里有 GPU_TEST_FEATURES 里的名字或 <包>/<名字>，或 --all-features），
    而且不带 --no-run。清场（gpu_quiesce_prefix）与交多机配置（文件头「带 gpu 特性的格把多机配置交给测试、跑完核真用了卡」）都照它认。
    弄坏开关 product-gpu-feature-unquiesced（environment 给了才认）：只认 gpu，产品 GPU 包的 tests-on-a-real-gpu 不算（改前的样子）。"""
    if "cargo" not in command:
        return False
    words = command[command.index("cargo") + 1:]
    if "test" not in words:
        return False
    words = words[words.index("test") + 1:]
    before = words[:words.index("--")] if "--" in words else words
    if "--no-run" in before:
        return False
    features = set()
    for index, word in enumerate(before):
        if word in ("--features", "-F") and index + 1 < len(before):
            word = "--features=" + before[index + 1]
        elif word.startswith("-F") and len(word) > 2:
            word = "--features=" + word[2:]
        if word.startswith("--features="):
            features |= {feature.split("/")[-1] for feature in word[len("--features="):].replace(",", " ").split()}
    wanted = {"gpu"} if environment is not None and break_is(environment, "product-gpu-feature-unquiesced") else GPU_TEST_FEATURES
    return bool(features & wanted) or "--all-features" in before


def gpu_rows_multi_host_configuration(environment):
    """带 gpu 特性的格交给测试的多机配置（文件头「带 gpu 特性的格把多机配置交给测试、跑完核真用了卡」）：(绝对路径, 文件在不在)。
    调用方设了 SINGLEFS_MULTI_HOST_CONFIG 照它；没设取 GATE_MUTATION_MULTI_HOST_CONFIG（自证用），再没设照 admission.py 的 multi_host_configuration_file
    取本模块所在仓的那一份（git common-dir 的上一层下的 multi-host.env）。相对路径按本进程的当前目录取成绝对路径：测试在源码副本里起，相对路径到那里就指错了。"""
    path = (environment.get("SINGLEFS_MULTI_HOST_CONFIG", "").strip() or environment.get("GATE_MUTATION_MULTI_HOST_CONFIG", "").strip()
            or admission_module().multi_host_configuration_file(STAGE_REPOSITORY, {}))
    path = os.path.abspath(path)
    return path, os.path.isfile(path)


def named_test_stdout(full_name, output):
    """libtest 给点名的测试交出来的那一段输出（「---- <全名> stdout ----」之后、到下一段的头、「successes:」「failures:」或「test result:」为止）；
    有几段（成功与失败各交一次）就接起来。输出里一段都没有（--nocapture 那种起法）交整份；有别的用例的段而没有它的，交空串（它没打字）。"""
    header = f"---- {full_name} stdout ----"
    collected, inside, found, any_section = [], False, False, False
    for line in output.splitlines():
        if line.startswith("---- ") and line.endswith(" stdout ----"):
            any_section = True
            inside = line == header
            found = found or inside
            continue
        if inside and (line in ("successes:", "failures:") or line.startswith("test result:")):
            inside = False
            continue
        if inside:
            collected.append(line)
    if found:
        return "\n".join(collected)
    return "" if any_section else output


def given_states_gpu_counts(text):
    """text 里每一行 CRASH_POINT_REPLAY_GIVEN_STATES（给定状态引擎每一趟收尾打的那一行，crate singlefs-checker-tier 的
    crash_amplification::GIVEN_STATES_LINE_PREFIX；_KEY_REFUSED 这类同前缀的别的行不算）的 states_judged_by_the_gpu_here，按出现的次序。"""
    line = re.compile(r"^\s*CRASH_POINT_REPLAY_GIVEN_STATES\s(?:.*\s)?states_judged_by_the_gpu_here=(\d+)(?=\s|$)", re.M)
    return [int(found.group(1)) for found in line.finditer(text)]


def gpu_card_unused_reason(row, output, configuration, environment):
    """带 gpu 特性、判出没红的那一条用没用上卡（文件头「带 gpu 特性的格把多机配置交给测试、跑完核真用了卡」）：用不上交接在「点名的测试 <名字>」后面的那一句，
    用上了或判不到这一层（点名的测试不是恰好对上一个用例、结果不都是 ok）交 None。configuration 是那一次交给测试的多机配置 (路径, 在不在)，没核过的是 None。
    不打 CRASH_POINT_REPLAY_GIVEN_STATES 行的照常判（gpu_test_start 取卡：没卡打 NOT_RUN_NO_GPU_ADAPTER、有卡起不来就 panic，不悄悄改交 CPU）。
    弄坏开关 gpu-unused-as-missed：一律交 None；given-states-count-ignored：有那一行就当没用上卡，不读数。"""
    if break_is(environment, "gpu-unused-as-missed"):
        return None
    results = results_by_test(row.expected, output)
    if len(results) != 1 or not all(outcome == "ok" for outcomes in results.values() for outcome in outcomes):
        return None
    if configuration is not None and not configuration[1]:
        return (f"带 gpu 特性，本模块所在仓的多机配置 {configuration[0]} 不在，没交给测试：GPU 用不上（测试在副本里找不到多机配置，GPU 开关是关的），"
                "改坏的 GPU 那一处没跑到")
    counts = given_states_gpu_counts(named_test_stdout(next(iter(results)), output))
    if counts and (break_is(environment, "given-states-count-ignored") or not any(counts)):
        return (f"带 gpu 特性，它打的 {len(counts)} 行 CRASH_POINT_REPLAY_GIVEN_STATES 里 states_judged_by_the_gpu_here 全是 0："
                "给定状态全交 CPU 判、GPU 一个状态都没判，改坏的 GPU 那一处没跑到")
    return None


def separate_gpu_quiesce_lines(run, environment):
    """经清场包装起的那一次（文件头「带 gpu 特性的格经清场起」）：包装自己打的几种行（GPU_QUIESCE step=…、回读看的卡、清了场、没复原……）挪出 run.stderr、
    放进 run.quiesce_lines，判结局与尾巴只看命令自己的输出；包装退 78 且打了 step=command result=not-started 的（清场回读没过、多机配置缺清场键、锁等不到：命令没起）
    把 run.quiesce_refused 置成真。弄坏开关 quiesce-refusal-unread：不认没起（改前的样子：照命令的输出判，记成点名的测试没跑到）。"""
    own_line = re.compile(r"^(?:GPU_QUIESCE step=|  · 回读看的卡：|  · 照多机配置清了场|  · 已有 \d+ 趟清过场在用卡|  · 清了场、没复原)")
    kept, own = [], []
    for line in run.stderr.splitlines(keepends=True):
        (own if own_line.match(line) else kept).append(line)
    run.stderr = "".join(kept)
    run.quiesce_lines = [line.rstrip("\n") for line in own]
    run.quiesce_refused = (run.returncode == 78 and any(line.startswith("GPU_QUIESCE step=command result=not-started") for line in run.quiesce_lines)
                           and not break_is(environment, "quiesce-refusal-unread"))


def quiesce_refusal_text(run):
    """经清场包装起、命令没起的那一次（run.quiesce_refused）交接在「点名的测试」「那一批」后面的那一句；别的交 None。"""
    if not getattr(run, "quiesce_refused", False):
        return None
    said = next((line for line in run.quiesce_lines if line.startswith("GPU_QUIESCE step=quiesce ")), "GPU_QUIESCE 没打清场那一行")
    return (f"带 gpu 特性、经 scripts/crash/gpu-quiesce-run.sh 起，清场回读没过、命令没起（{said}；卡上有不归清场白名单的计算进程时就是这样，"
            "等那个进程跑完再只重跑这几行，不停它）")


def run_under_wrapper(command, temporary_directory, stamp_name, extra_environment=None, limits=None, compile_first=False):
    """经内存包装跑一条命令（cargo test --offline … 这一类）：(subprocess 的结果, 用时)。限时交给包装（从起跑算、不算排队），TMPDIR 是这一次自己的；
    extra_environment 只加在这一次的环境里（覆盖那一格的编译目录、插桩开关与 LLVM_PROFILE_FILE）。limits 是这一次的 (内存上限, 限时秒；None 是不限时)，
    不给取整张表的那一个数（文件头「逐行上限」）。compile_first（文件头「基线」一段的限时）：先在同一个上限里不限时跑一趟 cargo test <-- 之前那一段> --no-run，
    编不成（编不过、撞了上限……）就把那一趟当这一次的结果交回；编成了再限时跑原命令。用时两趟相加。弄坏开关 timelimitincludescompile：不先编（改前的样子，限时含重编译）。"""
    context = WORKER_STATE["context"]
    memory_max, timeout_seconds = limits or (context["memory_max"], context["timeout_seconds"])
    if compile_first and not break_is(context["environment"], "timelimitincludescompile") and "test" in command:
        before = command[:command.index("--")] if "--" in command else list(command)
        build, build_seconds = run_one_under_wrapper(before + ["--no-run"], temporary_directory, stamp_name + "-build", extra_environment, memory_max, None)
        if build.returncode != 0:
            build.compile_pass = True   # 交回的是先编那一趟：verdict_of_run、baseline_problem 靠它分编译撞顶与测试撞顶
            return build, build_seconds
        run, run_seconds = run_one_under_wrapper(command, temporary_directory, stamp_name, extra_environment, memory_max, timeout_seconds)
        return run, {name: f"{float(build_seconds[name]) + float(run_seconds[name]):.3f}" for name in run_seconds}
    return run_one_under_wrapper(command, temporary_directory, stamp_name, extra_environment, memory_max, timeout_seconds)


def run_one_under_wrapper(command, temporary_directory, stamp_name, extra_environment, memory_max, timeout_seconds):
    """run_under_wrapper 的一趟：timeout_seconds 是 None 时不设 RUN_WITH_MEMORY_CAP_TIME_LIMIT（不限时，只有内存上限）。"""
    state, context = WORKER_STATE, WORKER_STATE["context"]
    stamp_path = os.path.join(state["shard_root"], START_STAMP_DIRECTORY_NAME, stamp_name)
    environment = dict(state["environment"], TMPDIR=temporary_directory,
                       RUN_WITH_MEMORY_CAP_KILL_GRACE=str(TIMEOUT_KILL_GRACE_SECONDS),
                       RUN_WITH_MEMORY_CAP_KEY=" ".join(command))
    if timeout_seconds is not None:
        environment["RUN_WITH_MEMORY_CAP_TIME_LIMIT"] = str(timeout_seconds)
    else:
        environment.pop("RUN_WITH_MEMORY_CAP_TIME_LIMIT", None)
    environment.update(extra_environment or {})
    # 带 gpu 特性、真跑测试的那一次把多机配置交给测试（文件头「带 gpu 特性的格把多机配置交给测试、跑完核真用了卡」）：副本里它自己找不到；
    # 文件不在就不设，judge_row 照「GPU 用不上」记。弄坏开关 gpu-rows-configuration-unpassed：不设、不记（改前的样子）
    configuration = None
    if gpu_test_run(command, context["environment"]) and not break_is(context["environment"], "gpu-rows-configuration-unpassed"):
        configuration = gpu_rows_multi_host_configuration(context["environment"])
        if configuration[1]:
            environment["SINGLEFS_MULTI_HOST_CONFIG"] = configuration[0]
    cap, leased_bytes = memory_max, 0
    if stamp_name.endswith("-build") and environment.get("CARGO_BUILD_JOBS"):
        # 编译那一趟（基线与变异的先编、插桩编译）：并行度不按核数 ÷ 工作进程数；按这一行的内存上限收，套在外层包装里时再向外层额度里此刻空着的那一部分借
        # （compile_plan，文件头「编译并行度」）：借到的算进这一趟包装的内存上限，编完就还
        cap, environment["CARGO_BUILD_JOBS"], leased_bytes = compile_plan(memory_max, environment["CARGO_BUILD_JOBS"], context["environment"], state.get("budget"))
    # 会话里的「日常一条」：每次包装带这个工作进程自己的并行理由，调用方设了的原样传、不覆盖；只设在这一次包装的环境里（文件头「并行理由」一段）
    if not state["environment"].get("RUN_WITH_MEMORY_CAP_PARALLEL_REASON") or break_is(context["environment"], "overridecallerreason"):
        if not break_is(context["environment"], "noparallelreason"):
            environment["RUN_WITH_MEMORY_CAP_PARALLEL_REASON"] = (
                f"同一次 checker-tier-crates-mutation-replay（本模块进程 {context['stage_process']}）的第 {state['worker_sequence']} 个工作进程，"
                f"各用各的源码副本与编译目录（这一轮共 {context['worker_count']} 个）")
    called = time.time()
    building = stamp_name.endswith("-build") and not stamp_name.startswith("coverage-")
    kind, _dash, line_number = stamp_name[:-len("-build")].partition("-") if building else stamp_name.partition("-")
    label = {"baseline": f"基线（批首行是变异表第 {line_number} 行）", "coverage": f"覆盖（组首行是变异表第 {line_number.split('-')[0]} 行）"}.get(
        kind, f"变异表第 {line_number} 行") + ("（先编，不限时）" if building else "")
    # 带 gpu 特性、真跑测试的那一次经清场包装起，清场包装在内存包装外面（文件头「带 gpu 特性的格经清场起」）
    quiesce_prefix = gpu_quiesce_prefix(command, context["environment"])
    try:
        run = run_forwarding_queue_lines(quiesce_prefix + ["bash", context["memory_cap_runner"], cap, "bash", "-c", START_STAMP_SCRIPT, "started-at", stamp_path]
                                         + command, label + ("（带 gpu 特性，经清场包装起）" if quiesce_prefix else ""), cwd=state["work"], env=environment)
    finally:
        if leased_bytes:
            state["budget"].give_back(leased_bytes)
    run.gpu_configuration = configuration
    if quiesce_prefix:
        separate_gpu_quiesce_lines(run, context["environment"])
        if run.quiesce_refused:
            note(f"  ! {label}：带 gpu 特性，清场回读没过、命令没起（清场包装退 78）：{'；'.join(run.quiesce_lines)}")
    if leased_bytes and run.returncode == MEMORY_CAP_HIT_EXIT:
        note(f"  ! {label}：先编那一趟撞了内存上限 {cap}（这一格自己的 {memory_max} 加向外层额度借的 {format_size(leased_bytes)}，并行度 {environment['CARGO_BUILD_JOBS']}）；"
             f"判定里写的上限是这一格自己的 {memory_max}")
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
    try:
        WORKER_STATE["library"].write(kind, key, fields)
    except OSError as error:
        return f"{WORKER_STATE['library'].path(kind, key)}：{error}"
    return None


def how_text(context, kind, limits=None):
    """记录里的跑法：哪一道、什么起法、这一条的上限与限时（只是一样属性，不进键；limits 不给取整张表的那一个数）。"""
    memory_max, timeout_seconds = limits or (context["memory_max"], context["timeout_seconds"])
    return (f"{STAGE} {kind}（经 run-with-memory-cap.sh 每条 {memory_max}、限时 {timeout_seconds} 秒、先编不限时；"
            f"第 {WORKER_STATE['worker_sequence']} 片，共 {context['worker_count']} 片）")


def judge_batch(batch):
    """进程池里的一格活：一批基线（没改坏的副本上跑一遍这一批各行点名的测试）。回 (每行的 (行号, 问题或 None), 那一批的尾巴, 写不进去的记录)。"""
    key, batch_rows = batch
    context = WORKER_STATE["context"]
    arguments = baseline_arguments(key, batch_rows)
    temporary_directory = tempfile.mkdtemp(prefix=f"baseline-{batch_rows[0].line_number}-", dir=WORKER_STATE["per_mutation_temporary_root"])
    try:
        limits = limits_for_lines(context, [row.line_number for row in batch_rows])
        run, seconds = run_under_wrapper(["cargo", "test", "--offline"] + arguments, temporary_directory, f"baseline-{batch_rows[0].line_number}",
                                         limits=limits, compile_first=True)
    finally:
        measure_and_remove_temporary_directory(temporary_directory, context)
    output = run.stdout + run.stderr
    judged, record_problems = [], []
    refused = quiesce_refusal_text(run)   # 带 gpu 特性那一批清场回读没过、命令没起（文件头「带 gpu 特性的格经清场起」）：不交判法，照基线不绿记、写明为什么
    for row in batch_rows:
        problem = f"没判成：那一批{refused}" if refused else baseline_problem(row, run.returncode, output, *limits, compile_pass=getattr(run, "compile_pass", False))
        excerpt = None
        if problem is not None:
            results = results_by_test(row.expected, output)
            excerpt = failure_excerpt(output, next(iter(results))) if len(results) == 1 else None
        judged.append((row.line_number, problem, excerpt or output_tail(output)))
        components = context["row_records"].get(row.line_number) or {}
        if components.get("baseline_key"):
            failed_to_write = write_record_quietly("baselines", components["baseline_key"], dict(
                kind="baseline", baseline_key=components["baseline_key"], judging=context["judging"], test_closure=components.get("test_closure", ""),
                coverage_id=components.get("coverage_id", ""), tree_snapshot=WORKER_STATE["snapshot"], test_full_name=single_full_name(row.expected, output) or "",
                verdict="ok" if problem is None else "not-ok", problem=problem or "", batch_arguments=" ".join(arguments),
                batch_exit_code=run.returncode, judged_date=mutation_units.today_text(), how=how_text(context, "基线", limits), **seconds))
            if failed_to_write:
                record_problems.append(failed_to_write)
    return judged, record_problems


def judge_row(row):
    """进程池里的一格活：一条变异。建它自己的 TMPDIR、改坏那一处、跑点名的测试、还原、删掉那个 TMPDIR，判完立刻写判定库记录与用时表。
    回 (((行号, 档, 那一条要打印的话, 删 TMPDIR 的结果), {test_full_name}), 写不进去的记录)。"""
    context = WORKER_STATE["context"]
    full = os.path.join(WORKER_STATE["work"], row.path)
    with open(full, encoding="utf-8") as handle:
        pristine = handle.read()
    assert pristine.count(row.old) == 1, (row.path, row.name)
    temporary_directory = tempfile.mkdtemp(prefix=f"line-{row.line_number}-", dir=WORKER_STATE["per_mutation_temporary_root"])
    limits = limits_for_lines(context, [row.line_number])
    with open(full, "w", encoding="utf-8") as handle:
        handle.write(pristine.replace(row.old, row.new))
    try:
        run, seconds = run_under_wrapper(["cargo", "test", "--offline"] + with_shown_output(row.arguments.split()), temporary_directory, f"line-{row.line_number}",
                                         limits=limits, compile_first=True)
    finally:
        with open(full, "w", encoding="utf-8") as handle:
            handle.write(pristine)
        # 不论退出码是几都删（含限时、撞顶被杀）：被杀的测试 Drop 守卫不跑，它建的镜像只有这里删得掉
        removal = measure_and_remove_temporary_directory(temporary_directory, context)
    output = run.stdout + run.stderr
    refused = quiesce_refusal_text(run)
    if refused:
        # 带 gpu 特性那一条清场回读没过、命令没起（文件头「带 gpu 特性的格经清场起」）：记清场没过没跑，不交判法、不算抓到也不算没红
        verdict, text = "quiescerefused", f"{TABLE}:{row.line_number} {row.name}：点名的测试 {row.expected} {refused}；不算抓到、也不算没红"
    else:
        verdict, text = verdict_of_run(row, run.returncode, output, *limits, context["environment"], compile_pass=getattr(run, "compile_pass", False))
        # 带 gpu 特性、判出没红的那一条核用没用上卡（文件头「带 gpu 特性的格把多机配置交给测试、跑完核真用了卡」）：用不上记本次未跑，不进没红
        unused = gpu_card_unused_reason(row, output, getattr(run, "gpu_configuration", None), context["environment"]) if verdict == "failure" and gpu_test_run(
            ["cargo", "test"] + row.arguments.split(), context["environment"]) else None
        if unused is not None:
            verdict, text = "notrun", f"{TABLE}:{row.line_number} {row.name}：本次未跑——点名的测试 {row.expected} {unused}；不算抓到、也不算没红"
    if verdict in VERDICTS_KILLED_BY_WRAPPER:
        file_count, allocated_bytes, _apparent_bytes, is_still_there = removal
        first_line, line_break, rest = text.partition("\n")
        text = (first_line + f"；它的临时目录{'没删掉' if is_still_there else '已整个删掉'}（删前留有 {file_count} 个文件、占盘 {allocated_bytes} 字节）"
                + line_break + rest)
    components = context["row_records"].get(row.line_number) or {}
    full_name = single_full_name(row.expected, output) or ""
    record_problems = []
    if components.get("row_key"):
        fields = collections.OrderedDict(
            kind="mutation", row_key=components["row_key"], judging=context["judging"], verdict=verdict, text=text,
            removal_files=removal[0], removal_allocated_bytes=removal[1], removal_apparent_bytes=removal[2], removal_left=int(removal[3]),
            exit_code=run.returncode, judged_date=mutation_units.today_text(), how=how_text(context, "现判", limits), test_full_name=full_name,
            test_closure=components.get("test_closure", ""), mutated=components.get("mutated", ""), toolchain=components.get("toolchain", ""),
            dependency=components.get("dependency", ""))
        if verdict in ("timeout", "memory", "compilememory"):
            # 上限与限时只记在这几档的记录里（这两档本来不复用）；别的档的键与记录里都没有它们
            fields.update(memory_max=limits[0], timeout_seconds=limits[1])
        fields.update(seconds)
        failed_to_write = write_record_quietly("records", components["row_key"], fields)
        if failed_to_write:
            record_problems.append(failed_to_write)
    if verdict not in VERDICTS_THAT_DID_NOT_RUN and verdict not in ("squeezed", "gpumemory", "quiescerefused"):
        timing_path = os.path.join(context["timings_directory"], row_content_hash(row))
        try:
            write_atomically(timing_path, mutation_units.record_text(RECORD_HEADER, dict(
                row_content_hash=row_content_hash(row), seconds_running=seconds["seconds_running"], seconds_queued=seconds["seconds_queued"],
                timed_out=int(verdict == "timeout"), judged_date=mutation_units.today_text())))
        except OSError as error:
            record_problems.append(f"{timing_path}：{error}")
    return ((row.line_number, verdict, text, removal), dict(test_full_name=full_name)), record_problems


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


def relative_inside(path, work):
    """cargo 消息里的路径 → 相对源码副本根的路径；落在副本之外（注册表、OUT_DIR）交 None。"""
    full = os.path.normpath(path if os.path.isabs(path) else os.path.join(work, path))
    root = os.path.normpath(work)
    return os.path.relpath(full, root) if full.startswith(root + os.sep) else None


def executables_of_build(messages, work):
    """--no-run 那一次的 compiler-artifact 消息 → 本仓编出来的可执行文件（测试与 bin，构建脚本不算）。"""
    executables = []
    for message in messages:
        if message.get("reason") != "compiler-artifact" or relative_inside(os.path.dirname(message.get("manifest_path", "")), work) is None:
            continue
        executable = message.get("executable")
        if executable and "custom-build" not in message.get("target", {}).get("kind", []) and executable not in executables:
            executables.append(executable)
    return executables


def judge_coverage(job):
    """进程池里的一格活：一条点名测试在没改坏的副本上记一份覆盖（文件头「判定库」覆盖记录 ①–④）。
    回 ({coverage（覆盖号或 None）, problem（没记成的那一句或 None）, line}, 写不进去的记录)。"""
    state, context = WORKER_STATE, WORKER_STATE["context"]
    result = dict(coverage=None, problem=None, line=job["line"], identity=job["coverage_id"])
    if break_is(context["environment"], "coveragenotrecorded"):
        return result, []
    if state["library"].read_json("coverage", job["coverage_id"]) is not None and not job.get("stale"):
        result.update(coverage=job["coverage_id"], existing=True)
        return result, []
    overrides, configuration = coverage_build_settings(state["environment"])
    overrides["CARGO_TARGET_DIR"] = state["environment"]["CARGO_TARGET_DIR"] + COVERAGE_TARGET_SUFFIX
    temporary_directory = tempfile.mkdtemp(prefix=f"coverage-{job['line']}-", dir=state["per_mutation_temporary_root"])
    profiles = os.path.join(temporary_directory, "profiles")
    os.mkdir(profiles)
    try:
        result["problem"] = record_coverage(job, overrides, configuration, temporary_directory, profiles)
        if result["problem"] is None:
            result["coverage"] = job["coverage_id"]
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        result["problem"] = f"记覆盖时出错（{type(error).__name__}：{error}）"
    finally:
        measure_and_remove_temporary_directory(temporary_directory, context)
    return result, []


def record_coverage(job, overrides, configuration, temporary_directory, profiles):
    """judge_coverage 的 ①–④：记成交 None，哪一步不成交那一句。"""
    state, context = WORKER_STATE, WORKER_STATE["context"]
    stamp = f"coverage-{job['line']}"
    # 带 -Cinstrument-coverage 的编译：cargo 没带 --target 时 RUSTFLAGS 也罩构建脚本与过程宏，它们在各自的源码目录（~/.cargo/registry/src/ 下依赖包的目录）里
    # 起，没设 LLVM_PROFILE_FILE 就在那里写 default_<哈希>_<进程号>.profraw；编译这一步同样把剖析文件指进这一格的 TMPDIR（不并进 profiles：那里只放测试那一趟的）
    build_profiles = os.path.join(temporary_directory, BUILD_PROFILES_DIRECTORY_NAME)
    os.mkdir(build_profiles)
    build_overrides = overrides if break_is(context["environment"], "buildprofilesunset") else dict(
        overrides, LLVM_PROFILE_FILE=os.path.join(build_profiles, "build-%p-%m.profraw"))
    limits = limits_for_lines(context, [job["line"]])
    # 插桩编译不限时、只有内存上限（文件头「基线」一段：限时只罩跑测试那一趟、不含重编译）；弄坏开关 timelimitincludescompile 下照改前限时
    build_limits = limits if break_is(context["environment"], "timelimitincludescompile") else (limits[0], None)
    build, _seconds = run_under_wrapper(["cargo", *configuration, "test", "--offline", *job["before"], "--no-run", "--message-format=json-render-diagnostics"],
                                        temporary_directory, stamp + "-build", build_overrides, limits=build_limits)
    outcome = wrapper_outcome(build.returncode, *limits)
    if outcome is not None or build.returncode != 0:
        return f"插桩编译没成（{outcome[1] if outcome else f'退出码 {build.returncode}'}）：{output_tail(build.stderr, 3)}"
    messages = []
    for line in build.stdout.splitlines():
        if line.startswith("{"):
            with contextlib.suppress(ValueError):
                messages.append(json.loads(line))
    executables = executables_of_build(messages, state["work"])
    if not executables:
        return "插桩编译没交出一个本仓的可执行文件"
    # 点名用例前自己补 --exact：行的 libtest 参数里已经带了的不再重复补（libtest 报 Option 'exact' given more than once、退 101，覆盖就记不成）；
    # 弄坏开关 exactduplicated：照改前原样接在后面
    switches = list(job["switches"]) if break_is(context["environment"], "exactduplicated") else [word for word in job["switches"] if word != "--exact"]
    run, _seconds = run_under_wrapper(["cargo", *configuration, "test", "--offline", *job["before"], "--", *switches, "--exact", job["test"]],
                                      temporary_directory, stamp + "-run", dict(overrides, LLVM_PROFILE_FILE=os.path.join(profiles, "cov-%p-%m.profraw")), limits=limits)
    refused = quiesce_refusal_text(run)
    if refused:
        return f"插桩跑 {job['test']} 没起：{refused}"
    output = run.stdout + run.stderr
    outcome = wrapper_outcome(run.returncode, *limits)
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
    # ④ 执行过的单元：计数 > 0 的函数落进的 fn 单元，连同点名测试闭包里不是 fn 的单元（常量、类型、静态项、宏、数据文件），都在这份副本上切
    if state["unit_keys"] is None:
        state["unit_keys"] = mutation_units.UnitKeys(state["work"])
    keys = state["unit_keys"]
    test, refusal = keys.resolve_test(job["arguments"], job["expected"])
    if refusal is not None:
        return f"副本里认不出点名的测试（{refusal}）"
    closure = keys.closure(test.unit, test.target, job["arguments"])
    executed, unplaced = keys.executed_units(export, state["work"])
    members = {(reference.kind, reference.file, reference.name, reference.fingerprint): reference
               for reference in executed + [member for member in closure.members if member.kind != "fn"]}
    state["library"].write_json("coverage", job["coverage_id"], dict(
        test=list(mutation_units.reference_of(test.unit)), executed=sorted(list(reference) for reference in members.values()),
        executed_functions=len(executed), unplaced_functions=unplaced, complete=unplaced == 0 and bool(executed),
        executables=len(executables), judged_date=mutation_units.today_text(), how=how_text(context, "覆盖", limits)))
    return None


# ── 整道 ──────────────────────────────────────────────────────────────────────

# gate-overlap:copy-kept scripts/mutation/mutate.sh 那边的 worker_count_and_head_line 照这里的「各项取最小、至少 1、核数一半至多 16」写了一份 bash：两边的项不同（这里 GATE_MUTATION_WORKERS 顶掉核数那一项、第二项是要现跑的条数、第三项是外层内存上限 ÷ 每条上限（读不到就没有这一项）；那边没有第三项，MUTATE_JOBS 只能压小、第二项是表里的条数、另算每个 cargo 的编译并行度），派活一边是本进程的进程池、一边是 bash 子 shell 按目录认领，共用的只剩一个取最小值，抽成共用要让 mutate.sh 每趟多起一个 python 只为算一个数、两边头一行的字也不同
def bytes_of_size(text):
    """systemd 写法的大小（正整数加 K / M / G / T，1024 进制）→ 字节；别的写法交 None。"""
    matched = SIZE_TEXT.match(text.strip())
    return int(matched.group(1)) * SIZE_UNIT_BYTES[matched.group(2)] if matched else None


def outer_memory_bytes(environment):
    """外层内存上限（RUN_WITH_MEMORY_CAP_OUTER_CAP）的字节数；没设、写法不认交 None。弄坏开关 outermemoryignored：当读不到。"""
    if break_is(environment, "outermemoryignored"):
        return None
    return bytes_of_size(environment.get(OUTER_MEMORY_CAP_VARIABLE, "").strip())


def worker_count_for(row_count, environment, memory_max, largest_memory=None, disk=None):
    """开几个工作进程，连同头一行要报的话。四项取最小、至少 1：核数的一半（至多 16，GATE_MUTATION_WORKERS 顶掉它）、这一趟要现跑的条数、
    外层内存上限 ÷ 每条变异的上限（向下取整；外层上限读不到时没有这一项）、磁盘余量放得下几个工作进程的编译目录（disk 给了才有这一项）。
    disk 是一个函数：前三项取最小的那个数当天花板（ceiling）传给它，它交回 (放得下几个或 None, 头一行要写的话)，None 是磁盘余量没读到、没有这一项，
    头一行写明原因（disk_bound_for，文件头「磁盘余量」）。
    memory_max 是每条的默认上限（没有逐行上限的那些行用的那个值），不是这一趟逐行上限里最大的：个别行的大上限由 run_pool 按额度排队
    （文件头「工作进程数」），不压低工作进程数。largest_memory 是这一趟逐行上限里最大的那个，只用来在头一行报「逐行上限最大 X、按额度排队」。
    弄坏开关 largestrowmemory：改前的样子，每条的上限换成 largest_memory（run_pool 也不排队）。
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
    outer_bytes = outer_memory_bytes(environment)
    if largest_memory and break_is(environment, "largestrowmemory"):
        memory_max = largest_memory
    per_mutation_bytes = bytes_of_size(memory_max)
    by_memory = outer_bytes // per_mutation_bytes if outer_bytes is not None and per_mutation_bytes is not None else None
    if by_memory is not None:
        bounds.append(("外层内存上限", by_memory, f"外层内存上限 {outer_text} ÷ 每条变异 {memory_max}，向下取整给 {by_memory} 个"))
    disk_unread = None
    if disk is not None:
        disk_count, disk_text = disk(max(1, min(count for _name, count, _text in bounds)))
        if disk_count is None:
            disk_unread = disk_text
        else:
            bounds.append(("磁盘余量", disk_count, disk_text))
    smallest = min(count for _name, count, _text in bounds)
    binding = "、".join(f"「{name}」" for name, count, _text in bounds if count == smallest)
    floor_note = "（那一项不足 1，按 1 个开）" if smallest < 1 else ""
    worker_count = max(1, smallest)
    head_line = (f"  … 这一轮开 {worker_count} 个工作进程（各项取最小、至少 1）：{'；'.join(text for _name, _count, text in bounds)}。"
                 f"卡在{binding}这一项{floor_note}；")
    if disk_unread is not None:
        head_line += f"磁盘余量没读到（{disk_unread}），工作进程数不按磁盘收；"
    if by_memory is not None:
        head_line += ("套在外层包装里的里层不排队，工作进程数按外层内存上限收（slice 的总上限兜底）；"
                      "先编那一趟的并行度另从外层额度里此刻空着的那一部分借（文件头「编译并行度」）")
        if largest_memory:
            largest_bytes = bytes_of_size(largest_memory)
            head_line += (f"；逐行上限最大 {largest_memory}、按额度排队：每一件（基线的批、变异、覆盖）按自己的上限从外层 {outer_text} 里扣，"
                          f"在跑的各件上限之和不超过它，扣不下的等前面的跑完，后面装得下的小件可以先派（至多先派 {worker_count} 件后停下等大件）")
            if largest_bytes is not None and outer_bytes is not None and largest_bytes > outer_bytes:
                head_line += (f"；最大那一件比外层还大：等在跑的收完、单独起，里层包装按用法错退 {MEMORY_CAP_UNAVAILABLE_EXIT}，"
                              "照旧报成「scope 起不来，没跑」，不在这里改它的上限")
    else:
        head_line += (f"内存不按进程数收：每条经 run-with-memory-cap.sh 排队，放不下的等（slice 的总上限兜底）；"
                      f"外层内存上限没读到（{OUTER_MEMORY_CAP_VARIABLE} {'没设' if not outer_text else f'写成「{outer_text}」，不是正整数加 K / M / G / T'}，按没套在外层包装里算）")
    return worker_count, head_line


def new_shard_root_path():
    """$TMPDIR 下 SHARD_ROOT_PREFIX 起头加八位随机字符的路径，只起名、不建。tempfile.mkdtemp 是在 mkdir 里把目录建出来、返回之后名字才到调用方手里：
    TERM 落在这两步之间，收尾的 rmtree 不知道名字，空目录就漏在 $TMPDIR。这里名字先交到调用方、记进 shard_root，再由调用方 os.mkdir，
    收尾删的就是这个名字（名字在、目录不在也无妨）。"""
    alphabet = "abcdefghijklmnopqrstuvwxyz0123456789_"
    chooser = random.SystemRandom()
    return os.path.join(tempfile.gettempdir(), SHARD_ROOT_PREFIX + "".join(chooser.choice(alphabet) for _ in range(8)))


def job_memory_bytes(context, kind, item):
    """派活队里一件（基线的批、一条变异、一组覆盖）自己的内存上限，字节：与它跑起来经包装给的上限同一个数（limits_for_lines：逐行上限写了的取它，
    没写的取整张表的那一个数，一批基线取这一批里最大的）。"""
    if kind == "baseline":
        lines = [row.line_number for row in item[1]]
    elif kind == "mutation":
        lines = [item.line_number]
    else:
        lines = [item["line"]]
    return bytes_of_size(limits_for_lines(context, lines)[0]) or 0


def pick_ready_job(ready, claimed_bytes, in_flight_count, outer_bytes, need_of, bypass_counts, bypass_limit):
    """派活堆 ready（元素 (阶段, 次序, 序号, 种类, 内容)）里此刻该派的一件，已从堆里取出；此刻不派回 None。
    outer_bytes 是 None（外层内存上限读不到）时不排队，取堆顶，与改前一样。否则按额度派：在跑的各件上限之和 claimed_bytes 加这一件的上限不超过 outer_bytes
    才派；扣不下的留在堆里，次序在它后面、扣得下的小件可以先派（每先派一件，被越过的每一件的 bypass_counts[序号] 加 1）；
    一件被越过的次数到了 bypass_limit，它后面的就都不派了，等在跑的收完释放额度，不许饿死大件（bypass_limit 是 None 时不设这个限，只给弄坏开关 bypassunbounded 用）。
    什么都没在跑时堆顶一定派：上限比整个外层还大的那一件也在这时单独起，包装按用法错退 251、照旧报出来。"""
    if not ready:
        return None
    if outer_bytes is None:
        return heapq.heappop(ready)
    held, chosen = [], None
    while ready:
        entry = heapq.heappop(ready)
        if in_flight_count == 0 or claimed_bytes + need_of(entry) <= outer_bytes:
            chosen = entry
            break
        held.append(entry)
        if bypass_limit is not None and bypass_counts.get(entry[2], 0) >= bypass_limit:
            break
    for entry in held:
        heapq.heappush(ready, entry)
    if chosen is not None:
        for entry in held:
            bypass_counts[entry[2]] = bypass_counts.get(entry[2], 0) + 1
    return chosen


def run_pool(context, worker_count, batches, ordered_rows, baseline_ok_lines, stop_after, coverage_planner):
    """一个进程池里先派基线、再派变异、最后派覆盖：一批基线判完，它 ok 的行进变异的队；已有基线 ok 的行一开始就在队里；
    一条变异判成抓到、它那一组还要记覆盖的（coverage_planner.job_after），那一组的覆盖进队；开跑时就要记的（coverage_planner.initial_jobs）一开始就在队里。
    同时在跑的至多 worker_count 格，空出一格补一格（谁先跑完谁再领），基线优先、变异按 ordered_rows 的次序、覆盖最后。
    套在外层内存包装里时（context["outer_memory_bytes"] 不是 None）另按额度派：每一格按自己的上限（job_memory_bytes）从外层额度里扣，在跑的各格上限之和不超过外层
    （pick_ready_job：扣不下的等、后面装得下的小格先派、一格最多被越过 worker_count 次）。弄坏开关 largestrowmemory 与 rowmemoryunbudgeted 下不排队，bypassunbounded 下不设越过次数的限。
    回 dict：mutation（变异的判定）、mutation_info（{行号: {snapshot, test_full_name}}）、baseline（{行号: (问题或 None, 尾巴)}）、coverage（覆盖那几格的结果）、
    submitted、received、stopped、left_behind、record_problems。"""
    sequence_counter = multiprocessing.Value("i", 0)
    # 被停的信号也要删掉 shard_root（$TMPDIR 下的 singlefs-mutation-replay-*）：先接住再建，建完之后收到的都走 finally；
    # 接信号、起名、建目录都在 try 里：哪一步之后收到都走 finally（信号处理还原、按已记下的名字删目录），名字先记进 shard_root 再 os.mkdir（new_shard_root_path）。
    # 只在主线程接得了。弄坏开关 shardrootkeptonsignal 下不接（改前的样子：收到就停，shard_root 留在 $TMPDIR）；
    # 弄坏开关 shardrootnamedlate 下照改前用 tempfile.mkdtemp 建（名字要等它返回才记得下）
    previous_handlers = {}
    shard_root = None
    try:
        if threading.current_thread() is threading.main_thread() and not break_is(context["environment"], "shardrootkeptonsignal"):
            for signal_number in STOP_SIGNALS:
                previous_handlers[signal_number] = signal.signal(signal_number, raise_stop_requested)
        if break_is(context["environment"], "shardrootnamedlate"):
            shard_root = tempfile.mkdtemp(prefix=SHARD_ROOT_PREFIX)
        else:
            shard_root = new_shard_root_path()
            try:
                os.mkdir(shard_root, 0o700)
            except FileExistsError:
                shard_root = None   # 别人的同名目录：收尾不许删它
                raise
        per_mutation_temporary_root = os.path.join(shard_root, PER_MUTATION_TEMPORARY_DIRECTORY_NAME)
        os.mkdir(per_mutation_temporary_root)
        os.mkdir(os.path.join(shard_root, START_STAMP_DIRECTORY_NAME))
        os.mkdir(target_over_cap_directory(shard_root))   # 停派目录（文件头「编译目录上限」）：工作进程清完仍超上限往里写
        # 这一趟自己的编译目录（文件头「编译目录」）：调用方没给 GATE_MUTATION_TARGET_DIR 就建在 <shard_root>/build 里，收尾（finally）删
        if context["shared_target_directory"] is None:
            build_root = os.path.join(shard_root, RUN_BUILD_DIRECTORY_NAME)
            os.mkdir(build_root)
            context["shared_target_directory"] = run_build_target_directory(shard_root)
            for line in target_reclaim_lines(context["shared_target_directory"], worker_count, ([], [], []), owned=True):
                note(line)
        # 先编一份依赖（文件头「先编一份依赖」）：工作进程起来之前编好，编成的由 init_worker 拷进各自的编译目录；摸过的包没重编整道判红、一个工作进程都不起
        seed_lines, seed_problems = prepare_seed(context, batches, ordered_rows, coverage_planner, shard_root)
        for line in seed_lines:
            note(line)
        if seed_problems:
            return dict(mutation=[], mutation_info={}, baseline={}, coverage=[], submitted=0, received=0, stopped=False, left_behind=[], record_problems=[],
                        left_behind_contents={}, per_mutation_temporary_root=per_mutation_temporary_root, seed_problems=seed_problems)
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
        in_flight, in_flight_bytes = {}, {}
        jobs = {"baseline": baseline_job, "mutation": mutation_job, "coverage": coverage_job}   # 各判一格，判完清一次自己那份超线的编译目录
        outer_bytes = (None if break_is(context["environment"], "largestrowmemory") or break_is(context["environment"], "rowmemoryunbudgeted")
                       else context.get("outer_memory_bytes"))
        # 额度账（OuterBudget）：外层读得到时父进程派一格记一笔、先编那一趟向它借；读不到时没有账，派活与先编的并行度与改前一样
        budget = None if outer_bytes is None else OuterBudget(outer_bytes, worker_count, bytes_of_size(context["memory_max"]) or 0)
        bypass_limit = None if break_is(context["environment"], "bypassunbounded") else worker_count
        bypass_counts, needs = {}, {}

        def need_of(entry):
            if entry[2] not in needs:
                needs[entry[2]] = job_memory_bytes(context, entry[3], entry[4])
            return needs[entry[2]]
        with concurrent.futures.ProcessPoolExecutor(max_workers=worker_count, initializer=init_worker,
                                                    initargs=(sequence_counter, shard_root, context, budget)) as pool, own_workers_stopped_on_stop(pool, shard_root, context["environment"]):
            while True:
                while ready and len(in_flight) < worker_count and not outcome["stopped"]:
                    # 挑与记账在同一把锁里做完：先编那一趟借额度也在这把锁里，在跑各格的上限加借走的不会在两步之间超过外层
                    with (budget.lock if budget is not None else contextlib.nullcontext()):
                        entry = pick_ready_job(ready, budget.used() if budget is not None else sum(in_flight_bytes.values()), len(in_flight),
                                               outer_bytes, need_of, bypass_counts, bypass_limit)
                        if entry is not None and budget is not None:
                            budget.claim(need_of(entry))
                    if entry is None:
                        break   # 额度扣不下：等在跑的收完释放（什么都没在跑时 pick_ready_job 一定派，下面 `not in_flight` 才会退出）
                    future = pool.submit(jobs[entry[3]], entry[4])
                    in_flight[future] = entry[3]
                    in_flight_bytes[future] = need_of(entry) if outer_bytes is not None else 0
                    outcome["submitted"] += 1
                if not in_flight:
                    break
                done, _pending = concurrent.futures.wait(list(in_flight), return_when=concurrent.futures.FIRST_COMPLETED)
                for future in done:
                    kind = in_flight.pop(future)
                    released = in_flight_bytes.pop(future, None)
                    if budget is not None and released is not None:
                        budget.release(released)
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
                # 停派（文件头「编译目录上限」）：有工作进程清完仍超上限，在跑的收完就不再派活
                over_cap = target_over_cap_reports(shard_root)
                if over_cap and not outcome.get("over_cap"):
                    outcome["stopped"], outcome["over_cap"] = True, over_cap
                    note(f"  ! 编译目录清完仍超上限，停派：在跑的 {len(in_flight)} 格收完就不再派活（还排着 {len(ready)} 格；收尾判红，出路在那里）")
        outcome["budget_peak"] = budget.peak.value if budget is not None else None
        outcome["left_behind"] = sorted(os.listdir(per_mutation_temporary_root))
        outcome["left_behind_contents"] = {name: (sorted(os.listdir(os.path.join(per_mutation_temporary_root, name)))
                                                  if os.path.isdir(os.path.join(per_mutation_temporary_root, name)) else ["（不是目录）"])
                                           for name in outcome["left_behind"]}
        outcome["per_mutation_temporary_root"] = per_mutation_temporary_root
        return outcome
    finally:
        # 先删这一趟的编译目录（remove_owned_build_root：核过没有进程开着里面的文件），没删成就连同 shard_root 留着（run_stage 判红）；
        # 被停时 own_workers_stopped_on_stop 已经删过，这里看到不在就照删 shard_root。弄坏开关 runtargetkept：两样都不删
        if shard_root is not None and os.path.isdir(shard_root):
            if remove_owned_build_root(shard_root, context["environment"]):
                shutil.rmtree(shard_root, ignore_errors=True)
            else:
                note(f"  · 这一趟的临时目录 {shard_root} 留着（里面的编译目录没删）")
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


CLOSURE_REFUSED_HOWTO = [
    "上面每处点名了测试和变异表的行：测试带 #[cfg(feature = \"…\")] 的，给那几行的 cargo 参数补上编它要的特性（例 --features verdict-store,gpu，写法照同一张表里别的带 gpu 的行），",
    "或点名一条在现有特性下编得到的测试；「没有叫 … 的包」这一类就改那一行的 -p。改完单跑 python3 scripts/mutation/crates-mutation-rows.py plan <根> 看还有没有别的行（键算不出来，整张表一行都判不了）。"]


def print_rejection(headline, details, howto_lines):
    say(f"  ✗ {headline}")
    for detail in details:
        say(f"      {detail}")
    for index, howto_line in enumerate(howto_lines):
        say(("     → 怎么办：" if index == 0 else "       ") + howto_line)


def prechecks(root, rows, malformed, environment):
    """派活之前先判、一个工作进程都不起的那几样（row_problems，与门禁 code-source-discipline 经 static-check 判的是同一份）。回 (拷贝范围, 整道判红了没有, {行号: 锚点失效的原因})。
    锚点失效按行判（文件头「锚点失效」）：只有它的时候不判整道，交回那几行由 run_stage 单记一档；别的几样有一样就整道判红，锚点失效的行一起列出。
    弄坏开关 anchor-stops-table：锚点失效也让整道判红（改前的样子）。"""
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
        return None, True, {}
    # 锚点只在真仓上核：文件落在拷贝范围之外时，工作进程在自己那份副本里打不开它，所以拷贝范围也在派活之前判。
    # 锚点失效之外的几样有一样红就一个工作进程都不起；只有锚点失效的，那几行单记一档、别的行照跑。
    problems = row_problems(root, rows, malformed, copied)
    whole_table = [kind for kind in ROW_PROBLEM_KINDS if problems[kind] and (kind != "ANCHOR" or break_is(environment, "anchor-stops-table"))]
    if not whole_table:
        return copied, False, {line_number: why for line_number, _name, why in problems["ANCHOR"]}
    not_started = "这一轮一个工作进程都没起；门禁 code-source-discipline 静态判同样几样（不跑变异），改完先跑它。"
    if problems["MALFORMED"]:
        print_rejection(f"{TABLE} 有几行不成形（六段制表符分隔：{'、'.join(FIELD_NAMES)}；替换文可以空，那是删掉原文，其余五段不许空）：",
                        [f"{TABLE}:{line_number} {name}：{why}" for line_number, name, why in problems["MALFORMED"]],
                        ["补上空着的那一段，制表符删补到恰好六段；原文 / 替换文里的换行写成 \\n。要删掉原文就让替换文空着（两个制表符挨着）。", not_started])
    if problems["ANCHOR"]:
        print_rejection("变异表有几行锚点失效：", [f"{TABLE}:{line_number} {name}：{why}" for line_number, name, why in problems["ANCHOR"]],
                        ["改代码时把变异表里的原文一起改到今天的写法、让它恰好命中一次（锚点失效的那一行等于没跑过，.claude/rules/mutation-sampling.md 第七类）。", not_started])
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
    return None, True, {}



def light_components(components):
    """交给工作进程的那一份（进程池按 pickle 传）：只有键与几样组成的号，不带单元对象。"""
    return dict(row_key=components.row_key or "", baseline_key=components.baseline_key or "", coverage_id=components.coverage_id or "",
                test_closure=components.test_closure or "", toolchain=components.toolchain or "", dependency=components.dependency or "",
                mutated=",".join(mutation_units.unit_id(reference) for reference in components.mutated or []))


def baseline_check(library, components, key_context, snapshot, environment, refusal):
    """(基线 ok 复用得上没有, 不复用的那一句)。全量开关下不读库；拒算的行（没有基线键）照跑。"""
    if refusal:
        return False, refusal
    if not components.baseline_key:
        return False, components.refusal or "没有基线键"
    record = library.read("baselines", components.baseline_key)
    coverage = library.read_json("coverage", components.coverage_id) if components.coverage_id else None
    problem = baseline_reuse_problem(record, components, key_context.keys, coverage, snapshot, environment)
    return problem is None, problem


def plan_rows(root, rows, environment, memory_max, timeout_seconds, judging_repository=None):
    """(KeyContext, {行号: Components}, 判定库)：run_stage、plan、row-plan、--list-items、import 共用这一份算法；只读（不写库）。"""
    key_context = KeyContext(root, environment, memory_max, timeout_seconds, judging_repository)
    key_context.prepare(rows)
    components = {row.line_number: key_context.components(row) for row in rows}
    return key_context, components, open_library(root, environment)


def decide_rows(library, rows, components, key_context, environment):
    """每一行 (Decision, 记录)：与 run_stage 同一份 row_decision。"""
    refusal = reuse_refusal(environment)
    decided = {}
    for row in rows:
        comp = components[row.line_number]
        previous = library.read("index", row_identity(row)) if comp.row_key and not refusal else None
        decided[row.line_number] = row_decision(library, comp, row, refusal, key_context, previous)
    return decided


def impact_unit_name(reference):
    """牵连面那一行里一个单元的名字：<文件>::<限定名>；数据文件的限定名就是路径，只写路径。"""
    return reference.file if reference.name == reference.file else f"{reference.file}::{reference.name}"


def impact_summary(rows, components, key_context, environment, anchor_broken_count=0):
    """牵连面那一行（文件头「牵连面」）：每一行的键依赖的单元（KeyContext 为它算好的点名测试闭包，连同被改坏的单元）按（种类、文件、限定名）计数，
    报进的行数最多的那个单元、前 5 个与一次改动最坏牵连几行。只报数，不判红。弄坏开关 impact-row-dropped：算进的头一行不数。"""
    counts, names, counted = collections.Counter(), {}, 0
    for row in rows:
        comp = components.get(row.line_number)
        closure = key_context.closures.get(comp.test_closure) if comp is not None and comp.test_closure else None
        if closure is None:
            continue
        counted += 1
        if counted == 1 and break_is(environment, "impact-row-dropped"):
            continue
        units = {}
        for reference in list(closure.members) + list(comp.mutated or []):
            units[(reference.kind, reference.file, reference.name)] = impact_unit_name(reference)
        for identity, name in units.items():
            counts[identity] += 1
            names[identity] = name
    tail = f"（算了 {counted} 行的闭包；拒算与锚点失效的 {len(rows) - counted + anchor_broken_count} 行没有闭包、没算进）"
    if not counts:
        return f"牵连最大的函数单元：没有（一行的闭包都没算出来）；一次改动最坏牵连 0 行{tail}"
    ranked = sorted(counts.items(), key=lambda item: (-item[1], names[item[0]], item[0]))
    top_identity, top_count = ranked[0]
    top_five = "、".join(f"{names[identity]} {count}" for identity, count in ranked[:5])
    return f"牵连最大的函数单元：{names[top_identity]} 进了 {top_count} 行的闭包（前 5：{top_five}）；一次改动最坏牵连 {top_count} 行{tail}"


def run_stage(root, environment=None, judging_repository=None):
    """门禁 checker-tier-crates-mutation-replay 的整道：判定打 stdout（按表的行号排序，与进程数、复用了多少都无关），进度与复用几条打 stderr。回退出码。
    judging_repository 只给自证用：拿判法的临时拷贝改了判法，看每一行都现判。"""
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
    # 逐行上限（文件头「逐行上限」）：照整张成形的表认名字，选行之前读
    row_limits, row_limit_problems = read_row_limits(root, rows, environment)
    limit_problems += row_limit_problems + size_setting_problems(environment)
    for headline, howto_line in limit_problems:
        print_rejection(headline, [], [howto_line])
    if limit_problems:
        return 1
    if row_limits:
        note(f"  … 逐行上限（{LIGHT_ROWS_TABLE} 的「# 逐行上限：」）：{len(row_limits)} 行另给了上限或限时，别的行照每条 {memory_max}、限时 {timeout_seconds} 秒")
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
    copied, refused, anchor_broken = prechecks(root, rows, malformed, environment)
    if refused:
        return 1
    # RocksDB 预编库的取值（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」）：起任何 cargo 之前（算键、基线、变异、插桩编译都在这之后）判这台装了、
    #   此刻进程环境的取值对得上；判红一条都不跑、退 1，红句照 entry 原样转打。树里没有依赖 rocksdb 的清单、PATH 上是带标记的假 cargo，entry 报不判，
    #   这时什么都不打（stderr 头一行仍是「判定库记录」那一行）。弄坏开关 rocksdb-entry-unchecked 下不判（自证「RocksDB 预编库的取值缺了」那一格判错）
    if not break_is(environment, "rocksdb-entry-unchecked"):
        entry = subprocess.run([sys.executable, ROCKSDB_PREBUILT_PATH, "entry", "--root", root], env=environment, capture_output=True, text=True,
                               stdin=subprocess.DEVNULL)
        if entry.returncode != 0:
            say(entry.stdout.rstrip("\n") + ("\n" + entry.stderr.rstrip("\n") if entry.stderr.strip() else ""))
            print_rejection("起 cargo 之前 RocksDB 预编库的取值没判过（上面 rocksdb-prebuilt.py entry 的红句），一条都没跑", [],
                            ['照上面的出路修（开新会话，或命令前 eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"），再跑同一条命令。'])
            return 1
    # 锚点失效的行（文件头「锚点失效」）单记一档：不算键、不派活、不写库，别的行照跑
    anchor_judgements = [(row.line_number, "anchor", anchor_broken_text(row, anchor_broken[row.line_number]), None)
                         for row in rows if row.line_number in anchor_broken]
    rows = [row for row in rows if row.line_number not in anchor_broken]
    memory_cap_runner = environment.get("GATE_MUTATION_MEMORY_CAP_RUNNER", "").strip() or DEFAULT_MEMORY_CAP_RUNNER
    try:
        key_context, components, library = plan_rows(root, rows, environment, memory_max, timeout_seconds, judging_repository)
    except mutation_units.ClosureRefused as error:
        print_rejection(f"点名的测试在 cargo 参数选的这一组特性下认不出或不编（{len(error.failures)} 处），键算不出来：一条都没跑", error.detail_lines(), CLOSURE_REFUSED_HOWTO)
        return 1
    except (JudgingUnitsRefused, mutation_units.MutationUnitsError, admission.InputManifestError, HeavyThreadDefaultError, OSError) as error:
        print_rejection(f"算不出判法单元指纹、工具链、依赖版本或判定库的位置（{error}），判定库没处对：一条都没跑", [],
                        ["被判的要是 git 工作树（或设 GATE_MUTATION_RECORDS_HOME）、rustc -vV 与 cargo -V 跑得通、Cargo.lock 读得懂；",
                         f"单跑 python3 scripts/mutation/crates-mutation-rows.py judging-units {root} 与 plan {root} 看它报什么。"])
        return 1
    timings_directory = os.path.join(library.directory, "timings")
    os.makedirs(timings_directory, exist_ok=True)
    refusal = reuse_refusal(environment)
    skip_baseline = break_is(environment, "nobaseline")
    snapshot = snapshot_identity(snapshot_of(root, copied)) if break_is(environment, "baseline-by-tree-snapshot") else None
    decided = decide_rows(library, rows, components, key_context, environment)
    decisions = {line_number: decision for line_number, (decision, _record) in decided.items()}
    sentinel = plan_reuse_sentinel(library, rows, components, refusal, environment)
    reused, baseline_ok_lines, reused_baseline_count = {}, set(), 0
    for row in rows:
        decision, record = decided[row.line_number]
        if decision.reuse and break_is(environment, "reuse-lookup-skipped"):
            decision = decisions[row.line_number] = Decision(False, "unexplained", "弄坏开关 reuse-lookup-skipped：没读库")
        if decision.reuse and sentinel.forced(sentinel_item(row)):
            decision = decisions[row.line_number] = Decision(False, "sentinel", "复用哨兵抽中：键相同，这一趟不读库")
        if decision.reuse:
            try:
                removal = (int(record.get("removal_files", "")), int(record.get("removal_allocated_bytes", "")),
                           int(record.get("removal_apparent_bytes", "")), record.get("removal_left") == "1")
            except ValueError:
                decision = decisions[row.line_number] = Decision(False, "unexplained", "记录里删临时目录的那几个数读不出")
        if decision.reuse:
            reused[row.line_number] = (row.line_number, "caught", caught_text(row, "（复用）"), removal)
            baseline_ok_lines.add(row.line_number)
            continue
        ok, _problem = baseline_check(library, components[row.line_number], key_context, snapshot, environment, refusal)
        if ok:
            baseline_ok_lines.add(row.line_number)
            reused_baseline_count += 1
    refused_lines = {row.line_number for row in rows if components[row.line_number].row_key is None}
    candidates = [row for row in rows if row.line_number not in reused]
    if skip_baseline:
        baseline_ok_lines |= {row.line_number for row in candidates}
    needs_baseline = [row for row in candidates if row.line_number not in baseline_ok_lines]
    batches = list(baseline_batches(needs_baseline).items())
    ordered_candidates, moved_count = dispatch_order(candidates, timings_directory, numeric_settings["GATE_MUTATION_LONG_ROW_SECONDS"], environment)
    planner = CoveragePlanner(components, library, key_context.keys)
    # 组成清单先进库（判法单元、点名测试的闭包、工具链行、依赖版本行）：这一趟中途停了，下一趟照样报得出变了哪几个单元
    key_context.persist(library, components)
    # stderr 头一行：复用几条、现跑几条、前移几条（逐栏的数在 stdout 的「复用：」那一行）
    note(f"  … 判定库记录：复用抓到 {len(reused)} 条、复用基线 ok {reused_baseline_count} 行；现跑基线 {len(batches)} 批 {len(needs_baseline)} 行、"
         f"现跑变异至多 {len(candidates)} 条（其中拒算 {len(refused_lines)} 条）、锚点失效不跑 {len(anchor_judgements)} 行、补记覆盖 0 份；前移 {moved_count} 条（上一次用时 ≥ {numeric_settings['GATE_MUTATION_LONG_ROW_SECONDS']} 秒"
         f"或上一次超时的先派，按用时从长到短）；判法单元 {key_context.judging[:16]}…（{len(key_context.judging_references)} 个单元），库在 {library.directory}"
         + (f"；不读库：{refusal}（只写不读，现判的照键写回同一份库）" if refusal else "")
         + ("；基线没跑：GATE_MUTATION_BREAK=nobaseline（只给证红用）" if skip_baseline else ""))
    for line in sentinel.plan_lines:
        note(line)
    # 现判与拒算的原因逐行进 stderr（作废时报出变了哪几个单元，文件头「复用判据」）；多了只列前 REASONS_SHOWN 行，全部的看 plan
    fresh_reasons = [row for row in rows if row.line_number not in reused and row.line_number not in refused_lines]
    for row in fresh_reasons[:REASONS_SHOWN]:
        decision = decisions[row.line_number]
        note(f"  · 现判：{TABLE}:{row.line_number} {row.name}：{COLUMN_TITLES[decision.column]}——{decision.detail}")
    refused_rows = [row for row in rows if row.line_number in refused_lines]
    for row in refused_rows[:REASONS_SHOWN]:
        note(f"  · 拒算（照跑、不复用）：{TABLE}:{row.line_number} {row.name}：{components[row.line_number].refusal}")
    if len(fresh_reasons) > REASONS_SHOWN or len(refused_rows) > REASONS_SHOWN:
        note(f"  · 现判 {len(fresh_reasons)} 行、拒算 {len(refused_rows)} 行，各只列了前 {REASONS_SHOWN} 行；逐行的栏与变了的单元：python3 scripts/mutation/crates-mutation-rows.py plan {root}")
    outcome = dict(mutation=[], mutation_info={}, baseline={}, coverage=[], submitted=0, received=0, stopped=False, left_behind=[], record_problems=[])
    worker_count, cargo_jobs = 0, "-"
    if candidates:
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
                            [tools_problem[1], "不退回不记覆盖去跑：没有覆盖记录，基线就只能按点名测试的闭包复用。"])
            return 1
        note(f"  … 记覆盖用的 llvm 工具：{tools_text}")
        # 工作进程数的 ③（外层上限 ÷ 每条上限）按每条的默认上限算（没有逐行上限的那些行用的那个值），不按这一趟最大的那一行算：有一行 24G 就只开 1 个的事不再有；
        # 个别行的大上限由 run_pool 按额度排队（每一格按自己的上限从外层额度里扣，文件头「工作进程数」）。largest_memory 只用来在头一行报
        largest_memory = limits_for_lines(dict(memory_max=memory_max, timeout_seconds=timeout_seconds, row_limits=row_limits),
                                          [row.line_number for row in candidates])[0]
        # 编译目录所在的盘放得下几个工作进程的两份编译目录（文件头「磁盘余量」）：算在核数、条数、外层内存三项之后，取最小。
        # 调用方没给 GATE_MUTATION_TARGET_DIR 时编译目录由 run_pool 在这一趟的 shard_root 里建，这里按它将来所在的盘（$TMPDIR）算
        shared_target = default_shared_target_directory(environment)
        disk_probe = shared_target if shared_target is not None else planned_run_build_target_directory()
        # 每份编译目录的涨幅按判定库里近 20 次每格判完量到的占盘峰值算（文件头「磁盘余量」），没有读数的那一种按上限
        target_peaks = measured_target_peaks(library.directory, 20)
        worker_count, worker_count_head_line = worker_count_for(len(candidates), environment, memory_max, largest_memory,
                                                                disk=lambda ceiling: disk_bound_for(disk_probe, environment, ceiling, target_peaks))
        note(worker_count_head_line)
        # 每个 cargo 的编译并行度：进程数 × 它 ≈ 核数，免得 N 个 cargo 各自按核数开 rustc、互相抢
        cargo_jobs = environment.get("GATE_MUTATION_CARGO_JOBS", "").strip() or str(max(1, (os.cpu_count() or 4) // worker_count))
        environment["CARGO_BUILD_JOBS"] = cargo_jobs
        # 起 cargo 时关调试信息（文件头「调试信息」）：只设在起 cargo 的这一份环境里；取值另照 key_toolchain_lines 进了行键
        debug_overrides = cargo_debug_overrides(environment)
        environment.update(debug_overrides)
        note("  … 起 cargo 时的构建档：" + ("、".join(f"{name}={value}" for name, value in sorted(debug_overrides.items())) + "（关调试信息，取值进行键）"
                                       if debug_overrides else "照调用方的环境与 Cargo.toml（弄坏开关 debuginfo-kept：调试信息开着）"))
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
             + (f"，清掉调用方点名的 {'、'.join(crash_library_cleared)}" if crash_library_home is not None else "（弄坏开关 sharedcrashlibrary：照调用方的环境）")
             + (f"；每个工作进程再用底下自己片号的 {CRASH_LIBRARY_SHARD_DIRECTORY_PREFIX}<片号> 子目录，不共用一份库" if crash_library_home is not None else ""))
        # 调用方给的编译目录：片号更大的旧 -w<片号>（连同 -coverage）开跑时回收，有人在用的留着（reclaim_extra_target_directories）；
        # 这一趟自己建的由 run_pool 建好之后打那一行
        if shared_target is not None:
            for line in target_reclaim_lines(shared_target, worker_count, reclaim_extra_target_directories(shared_target, worker_count, environment), owned=False):
                note(line)
        context = dict(root=root, copied=copied, environment=environment, memory_max=memory_max, timeout_seconds=timeout_seconds, row_limits=row_limits,
                       memory_cap_runner=memory_cap_runner, judging=key_context.judging, llvm_tools=tools,
                       library_directory=library.directory, timings_directory=timings_directory,
                       row_records={row.line_number: light_components(components[row.line_number]) for row in rows},
                       shared_target_directory=shared_target, worker_count=worker_count, stage_process=os.getpid(),
                       outer_memory_bytes=outer_memory_bytes(environment), target_directory_cap_bytes=target_directory_cap_bytes(environment))
        RUN_BUILD_REMOVAL_FAILURES.clear()   # 这一趟 run_pool 收尾没删成的编译目录（同一进程里跑几趟时不带上一趟的）
        try:
            outcome = run_pool(context, worker_count, batches, ordered_candidates, baseline_ok_lines, numeric_settings["GATE_MUTATION_STOP_AFTER_ROWS"], planner)
        except concurrent.futures.process.BrokenProcessPool:
            # 一个工作进程被硬杀（OOM、段错误）时当场抛这个，而不是安静地少产出几项
            print_rejection("有工作进程中途死了，这一轮的变异没跑全", [],
                            ["多半是内存不够被 OOM 杀的——GATE_MUTATION_WORKERS 调小再跑一遍（判完的那几条有判定库记录，下一趟只跑剩下的）；",
                             "单跑 GATE_MUTATION_WORKERS=1 能跑完就是并发度的问题，还死就去看那一条变异本身。"])
            return 1
        if RUN_BUILD_REMOVAL_FAILURES:
            print_rejection(f"这一趟自己建的编译目录收尾没删成（{'、'.join(RUN_BUILD_REMOVAL_FAILURES)}），整道判红", [],
                            ["照上面那几行报的进程逐个停掉（python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号>），再删那个目录；",
                             "是这一趟的工作进程或 cargo 没收干净：去看 scripts/mutation/crates-mutation-rows.py 的 run_pool 收尾。"])
            return 1
        if outcome.get("seed_problems"):
            print_rejection(f"先编一份依赖：源码副本里摸过的包有 {len(outcome['seed_problems'])} 个产物 cargo 当成没变（fresh），修改时刻规矩坏了，一条都没跑",
                            outcome["seed_problems"],
                            ["去看 scripts/mutation/crates-mutation-rows.py 的 sync_seed_source（内容变了的文件要写成此刻的修改时刻、所在的包整包摸过）；",
                             f"修好之前删掉 {os.path.normpath(context['shared_target_directory'] or planned_run_build_target_directory()) + SEED_SOURCE_SUFFIX} 再跑（整份重拷，修改时刻都是此刻）。"])
            return 1
    if outcome.get("budget_peak") is not None:
        note(f"  … 额度账：在跑各格的上限加先编借走的额度，这一趟的峰值 {format_size(outcome['budget_peak'])}（外层上限 {format_size(context['outer_memory_bytes'])}）")
    for problem in outcome["record_problems"]:
        note(f"  ! 判定库记录或用时表没写成（下一趟这一条照跑，不会复用）：{problem}")
    # 派出去多少格就要收回来多少格：对不上整道红，不许少跑一条还报绿（command-safety.md）
    if outcome["submitted"] != outcome["received"]:
        print_rejection(f"派出去 {outcome['submitted']} 格（基线的批、变异与覆盖），只收回 {outcome['received']} 格", [],
                        ["这是分片并发自己的完整性闸红了，不是变异的问题；把 GATE_MUTATION_WORKERS=1 再跑一遍看串行下是不是全的，",
                         "是就去查领活与收束那一段（scripts/mutation/crates-mutation-rows.py 的 run_pool）。"])
        return 1
    if outcome.get("over_cap"):
        print_rejection(f"编译目录清掉增量产物与 deps 里同名产物的旧版本之后仍超上限 {TARGET_DIRECTORY_CAP_VARIABLE}，停派：在跑的那几格判完就没再派活，"
                        f"这一轮没跑全（现判的变异判完 {len(outcome['mutation'])} 条）", outcome["over_cap"],
                        [f"剩下的是每个依赖与测试二进制的最新一版：把 {TARGET_DIRECTORY_CAP_VARIABLE} 调大到放得下这一趟的一份编译目录（文件头「编译目录上限」），",
                         "或给编译目录换一块余量大的盘（GATE_MUTATION_TARGET_DIR）；判完的那几格已经写了判定库记录，再跑一趟只跑剩下的。"])
        return 1
    if outcome["stopped"]:
        print_rejection(f"只供测试的开关 GATE_MUTATION_STOP_AFTER_ROWS={numeric_settings['GATE_MUTATION_STOP_AFTER_ROWS']}："
                        f"现跑的变异判完 {len(outcome['mutation'])} 条就没再派活，这一轮没跑全", [],
                        ["这个开关只给自证造「跑到一半被杀」用，门禁里别设它；判完的那几条已经写了判定库记录，去掉它再跑一趟只跑剩下的。"])
        return 1
    # 索引：这一趟复用的与现判出结果的每一行，记下它此刻的键与几样组成的号（下一趟键对不上时照它报变了哪几个单元）
    judged_lines = {judgement[0] for judgement in outcome["mutation"]} | set(reused)
    for row in rows:
        if row.line_number in judged_lines and components[row.line_number].row_key:
            with contextlib.suppress(OSError):
                library.write("index", row_identity(row), index_entry(components[row.line_number], row))
    garbage = library.collect_garbage()
    rows_by_line = {row.line_number: row for row in rows}
    fresh = []
    for line_number, verdict, text, removal in outcome["mutation"]:
        if verdict == "caught":
            how = "（拒算）" if line_number in refused_lines else f"（现判：{COLUMN_TITLES[decisions[line_number].column]}）"
            text = caught_text(rows_by_line[line_number], how)
        fresh.append((line_number, verdict, text, removal))
    judgements = list(reused.values()) + fresh + anchor_judgements
    for row in candidates:
        problem, tail = outcome["baseline"].get(row.line_number, (None, ""))
        if problem is not None:
            judgements.append((row.line_number, "baseline",
                               f"{TABLE}:{row.line_number} {row.name}：基线不绿——点名的测试 {row.expected} 在没改坏的副本上{problem}\n{tail}", None))
    judged_lines = {judgement[0] for judgement in judgements}
    missing = [row for row in rows if row.line_number not in judged_lines]
    if missing:
        print_rejection(f"有 {len(missing)} 行一个判定都没有（没有作数的记录、基线也没判出它来）", [f"{TABLE}:{row.line_number} {row.name}" for row in missing],
                        ["这是判定库记录与派活自己的完整性闸红了，不是变异的问题；GATE_MUTATION_START_OVER=1 整道重跑一遍，还缺就去查 run_stage 与 run_pool 那一段。"])
        return 1
    fresh_columns = collections.Counter(decisions[row.line_number].column for row in rows if row.line_number not in reused and row.line_number not in refused_lines)
    coverage_failures = [result for result in outcome["coverage"] if result["problem"] is not None]
    for result in coverage_failures:
        note(f"  ! 覆盖没记成（组首行 {TABLE}:{result['line']}；下一趟这一条测试的基线只按它的闭包复用）：{result['problem']}")
    coverage_recorded = sum(1 for result in outcome["coverage"] if result["coverage"] and not result.get("existing"))
    coverage_existing = sum(1 for result in outcome["coverage"] if result.get("existing"))
    note(f"  … 覆盖：这一趟记了 {coverage_recorded} 份、已有沿用 {coverage_existing} 份、没记成 {len(coverage_failures)} 份；"
         f"收尾删了 {mutation_units.UNREFERENCED_DAYS_KEPT} 天里没被任何一趟引用的 {sum(garbage.values())} 份（{'、'.join(f'{kind} {count}' for kind, count in sorted(garbage.items())) or '一份都没有'}）")
    for line_number in reused:
        sentinel.observe(sentinel_item(rows_by_line[line_number]), reuse_checks.REUSED, "抓到")
    for line_number, verdict, _text, _removal in outcome["mutation"]:
        item = sentinel_item(rows_by_line[line_number])
        sentinel.observe(item, reuse_checks.FORCED if sentinel.forced(item) else reuse_checks.JUDGED, "抓到" if verdict == "caught" else verdict)
    sentinel_problems = sentinel.finish(printer=note)
    exit_code = report(rows, judgements, outcome, memory_max, timeout_seconds, selection_path, skip_baseline, dict(
        worker_count=worker_count, cargo_jobs=cargo_jobs, reused=len(reused), reused_baseline=reused_baseline_count,
        compile_jobs_text=("（GATE_MUTATION_CARGO_JOBS 给的，编译那一趟不再按内存上限收）" if environment.get("GATE_MUTATION_CARGO_JOBS", "").strip()
                           else (f"（编译那一趟再按每条的内存上限收：每条 {memory_max} 时 {compile_jobs_for(memory_max, cargo_jobs, environment)}"
                                 + ("；套在外层包装里时另向外层额度里此刻空着的那一部分借，借到的越多并行度越高、至多 " + cargo_jobs
                                    if outcome.get("budget_peak") is not None and not break_is(environment, "compilejobsownlimit") else "") + "）")
                           if cargo_jobs != "-" else ""),
        fresh_columns=fresh_columns, fresh_rows=len(candidates) - len(refused_lines), refused_rows=len(refused_lines),
        coverage_recorded=coverage_recorded, coverage_existing=coverage_existing, coverage_failed=len(coverage_failures),
        fresh_mutations=len(outcome["mutation"]), fresh_batches=len(batches), records_directory=library.directory,
        not_run_record=environment.get("GATE_MUTATION_NOT_RUN_RECORD", "").strip(),
        impact=impact_summary(rows, components, key_context, environment, len(anchor_judgements))))
    for message, howto in sentinel_problems:
        print_rejection(message, [], [howto])
    return 1 if sentinel_problems else exit_code


def sentinel_item(row):
    """复用哨兵里一行的名字：行号与变异名（变异名可能重名）。"""
    return f"{row.line_number} {row.name}"


def plan_reuse_sentinel(library, rows, components, refusal, environment):
    """复用哨兵（文件头「复用三问与复用哨兵」）：候选是此刻行键在 records/ 里有「抓到」记录的行，直接读记录（不经 row_decision）；不读库的一趟不抽。"""
    sentinel = reuse_checks.ReuseSentinel(SCRIPT_KEY, "crates 变异表", environment=environment)
    candidates = {}
    for row in rows if not refusal else []:
        key = components[row.line_number].row_key
        record = library.read("records", key) if key else None
        if record is not None and record.get("row_key") == key and record.get("verdict") == REUSABLE_VERDICT["mutation"]:
            candidates[sentinel_item(row)] = "抓到"
    # 抽中的几条先攒着，等 stderr 头一行（「… 判定库记录：」）打完再打（别的定义与自证照头一行读复用几条）
    sentinel.plan_lines = []
    sentinel.plan(candidates, reuse=not refusal, off_reason=f"不读库（{refusal}）" if refusal else "", printer=sentinel.plan_lines.append)
    return sentinel


class CoveragePlanner:
    """这一趟要记哪几份覆盖（文件头「判定库」覆盖记录）：一行现判成抓到、它点名的测试此刻还没有一份执行过的单元都没变的覆盖记录，就派一格（同一份覆盖号只派一次）。"""

    def __init__(self, components, library, keys):
        self.components, self.library, self.keys = components, library, keys
        self.scheduled = set()

    def initial_jobs(self):
        return []

    def predicted_rows(self, rows):
        """这一趟现跑之后可能要记覆盖的行（先编一份依赖的插桩那一份按它们的组编，文件头「先编一份依赖」）：与 job_after 同一条判据，只是不看判定、不记 scheduled，
        同一份覆盖号只交头一行。"""
        picked, seen = [], set()
        for row in rows:
            components = self.components.get(row.line_number)
            if components is None or not components.coverage_id or not components.test_full_name or components.coverage_id in seen:
                continue
            seen.add(components.coverage_id)
            if not executed_set_is_current(self.keys, self.library.read_json("coverage", components.coverage_id)):
                picked.append(row)
        return picked

    def job_after(self, row, judgement, info):
        if judgement[1] != "caught":
            return None
        components = self.components.get(row.line_number)
        if components is None or not components.coverage_id or components.coverage_id in self.scheduled or not components.test_full_name:
            return None
        coverage = self.library.read_json("coverage", components.coverage_id)
        if executed_set_is_current(self.keys, coverage):
            return None
        self.scheduled.add(components.coverage_id)
        before, switches = batch_key(row.arguments)
        return dict(coverage_id=components.coverage_id, before=list(before), switches=list(switches), test=components.test_full_name,
                    arguments=row.arguments, expected=row.expected, line=row.line_number, stale=coverage is not None)


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
    compile_memory_hits = by_verdict["compilememory"]   # 编译那一趟撞了内存上限（verdict_of_run）：「计数：」里仍算在内存撞顶里，那一行的格式不动
    refused, unavailable, baseline_bad = by_verdict["refused"], by_verdict["unavailable"], by_verdict["baseline"]
    not_run = by_verdict["notrun"]
    gpu_memory = by_verdict["gpumemory"]
    quiesce_refused = by_verdict["quiescerefused"]   # 带 gpu 特性、清场回读没过、命令没起（文件头「带 gpu 特性的格经清场起」）
    anchor_broken = by_verdict["anchor"]   # 锚点失效（文件头「锚点失效」）：rows 里没有这几行，「共」那个数把它们加回来
    for text in caught:
        say(text)
    # 各栏各数各的；「计数：」这一行的格式别的定义与规则照栏读（mutation-triage），基线另起一行报
    say(f"  计数：抓到 {len(caught)} 条、没红 {len(failures)} 条、无效 {len(invalid)} 条、内存撞顶 {len(memory_hits) + len(compile_memory_hits)} 条、超时 {len(timeouts)} 条、"
        f"被总上限挤掉 {len(squeezed)} 条、排不上没跑 {len(refused)} 条、scope 起不来没跑 {len(unavailable)} 条、锚点失效 {len(anchor_broken)} 行"
        f"（共 {len(rows) + len(anchor_broken)} 条）")
    if gpu_memory:   # 「计数：」那一行的格式不动（别的定义照栏读），显存不够另起一行报
        say(f"  显存不够：{len(gpu_memory)} 条（内存包装退 {GPU_MEMORY_HIT_EXIT}：不算抓到、不算没红，结果不算数）")
    if quiesce_refused:   # 同上，另起一行报
        say(f"  清场没过没跑：{len(quiesce_refused)} 条（点名的测试要编 gpu 特性、经 scripts/crash/gpu-quiesce-run.sh 起，清场回读没过、命令没起：不算抓到、不算没红）")
    if skip_baseline:
        say("  基线：没跑（GATE_MUTATION_BREAK=nobaseline，只给证红用）")
    else:
        say(f"  基线：没改坏的副本上点名的测试恰好对上一个用例、结果是 ok 的 {len(rows) - len(baseline_bad)} 行，基线不绿 {len(baseline_bad)} 行"
            f"（按 cargo 参数分 {len(baseline_batches(rows))} 批；共 {len(rows)} 行）")
    # 复用与覆盖各另起一行（「计数：」那一行的格式不动）；判定本身与进程数无关，这两行随记录变（比对两趟的判定时去掉，selftest_verdict_view）
    counts["not_run_rows"] = len(not_run)
    reuse_text = reuse_summary_text(counts)
    say(f"  复用：{reuse_text}")
    say(f"  覆盖：这一趟记了 {counts['coverage_recorded']} 份（点名测试执行过的单元，基线按它复用），已有、沿用 {counts['coverage_existing']} 份，没记成 {counts['coverage_failed']} 份"
        + ("（逐份的原因在 stderr；那几条测试的基线下一趟只按闭包复用）" if counts["coverage_failed"] else ""))
    # 本次未跑的逐条列出来（文件头「本次未跑」）：不算抓到、不算没红；「计数：」那一行的格式不动，另起一行报
    not_run_record_problem = None
    if not_run:
        say(f"  本次未跑：{len(not_run)} 条（点名的测试在本机打了 NOT_RUN_NO_GPU_ADAPTER 后返回成功，或带 gpu 特性而多机配置不在、给定状态全交 CPU 判，"
            "没跑到被改坏的那一处：不算抓到、不算没红，这一趟不是整张表的结论）")
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
        (anchor_broken, f"有几行锚点失效（锚点失效 {len(anchor_broken)} 行；原文在「文件」那一段指的源码里不是恰好命中一次，或文件不在、读不了：这几行没跑、不入库、不复用，别的行照判）：",
         ["把那一行的原文改到今天源码里逐字存在、只出现一次的那一段（原文里的换行写成 \\n），文件挪了就改「文件」那一段；",
          "改完单跑那几行：bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --item <行号或变异名>（.claude/rules/mutation-sampling.md 第七类）。"]),
        (baseline_bad, f"有几行基线不绿（基线不绿 {len(baseline_bad)} 行；点名的测试在没改坏的副本上就不是恰好一个 ok，改坏之后红了也证明不了什么，这几行的变异没跑）：",
         ["先看点名的测试读的文件在不在拷贝范围里（.claude/gate.d/stage-inputs.tsv 本阶段那一行；bash .claude/gate.d/code-source-discipline.sh 静态判这一样），",
          "不在就登记进那一行；再在真仓上单跑它（照那一行的 cargo test 参数），真仓上也红就是测试本来就坏，先修测试；对上多个用例的把点名写全到模块路径；",
          "写「编译那一趟撞了内存上限」的是编译没装进这一行的上限（不是测试本身要这么多）：看尾巴里在编什么、stderr 末尾的编译并行度，给这一行单起更大的上限（轻型变异登记表「# 逐行上限：」），或先把依赖编一遍再跑。"]),
        (invalid, f"有变异无效（无效 {len(invalid)} 条；替换文写进源码之后编不过，那条行为今天零变异覆盖）：",
         ["先看尾巴分来源，不是去补用例。有 process didn't exit successfully 与信号的，是测试进程被杀，先查是谁杀的；点名的名字里带 $ 这类字符的，改那一行的测试名；其余是替换文编不过——",
          "先看反斜杠：表里只有 \\n 会被还原成换行，\\& \\\" 这类原样写进源码就编不过；",
          "别的编译错就在副本里把替换后的那一行 cargo check 一遍，改成编得过、而且真会改行为的写法（.claude/rules/mutation-sampling.md 第八类）。"]),
        (failures, f"有变异没红（没红 {len(failures)} 条）：",
         ["没红 = 那处改法没有任何测试守着：先造一条会红的用例再改代码（show-me-test.md）；「没跑到」多半是 cargo test 参数选错了范围。"]),
        (compile_memory_hits, f"有变异在编译那一趟撞了内存上限（编译撞顶 {len(compile_memory_hits)} 条，每条上限 {memory_max}；「计数：」里算在内存撞顶里）：",
         ["编译撞顶 = 改坏之后的代码在这个上限里没编完，点名的测试一次都没起——不算抓到、也不是没红，更不说明这条破坏让被测代码无界分配：别去改那条变异、别加分配界。",
          "看上面每条尾巴里在编什么（librocksdb-sys 这类 C++ 依赖最占内存）和 stderr 末尾的编译并行度；编译那一趟的并行度已按这一行的上限收",
          f"（每个并行任务按 {COMPILE_GIB_PER_JOB} GiB 算），收了还撞就给这一行（连同它所在的一批基线）单起更大的上限（轻型变异登记表「# 逐行上限：」）或调大 GATE_MUTATION_MEMORY_MAX，",
          "也可以先在宽一点的上限里把依赖编一遍（编译目录跨轮留着）再跑。"]),
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
        (quiesce_refused, f"有变异带 gpu 特性、清场回读没过没跑（清场没过没跑 {len(quiesce_refused)} 条；经 scripts/crash/gpu-quiesce-run.sh 起，命令没起）：",
         ["卡上有不归清场白名单的计算进程（别的会话的 GPU 活）时清场回读本来就不过：等那个进程跑完再只重跑这几行，不停它：",
          "bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --item <行号或变异名>；回读没过的原话在清场包装留的本机日志里（GPU_QUIESCE 那一行的 log=），",
          "多机配置没写全 QUIESCE_STOP_COMMAND、QUIESCE_STOPPED_CHECK_COMMAND 的照仓根 multi-host.env.example 写全；别拿掉清场包装去跑。"]),
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
    red = bool(invalid or failures or memory_hits or compile_memory_hits or timeouts or squeezed or gpu_memory or quiesce_refused or refused or unavailable or baseline_bad
               or left_behind or not_run_record_problem or anchor_broken)
    note(f"  … 这一轮 {counts['worker_count']} 个工作进程，每个 cargo 编译并行度 {counts['cargo_jobs']}{counts['compile_jobs_text']}，每条变异内存上限 {memory_max}、限时 {timeout_seconds} 秒；"
         f"复用抓到 {counts['reused']} 条、复用基线 ok {counts['reused_baseline']} 行，现跑变异 {counts['fresh_mutations']} 条、现跑基线 {counts['fresh_batches']} 批")
    if selection_path:
        # 只判了一部分：都抓到也退 3（门禁汇总记失败，不会被当成通过），有红退 1
        say(f"  … 只判了选中的 {len(rows)} 行，不是整道结论（GATE_MUTATION_ROW_SELECTION={selection_path}；"
            f"{'有红，见上面' if red else (f'选中的里抓到 {len(caught)} 条、本次未跑 {len(not_run)} 条' if not_run else '选中的都红在点名的测试上')}；"
            f"判定库记录写进了 {counts['records_directory']}）；这一趟{reuse_text}")
        return 1 if red else EXIT_SELECTION_ONLY
    if red:
        return 1
    if not_run:
        # 有本次未跑的：不说全抓到；一条抓到的都没有就退 77（无对象判到）
        if not caught:
            say(f"  · crates 变异表复跑：{len(rows)} 条全是本次未跑（见上），一条都没判到被改坏的那一处：这一道记本次未跑，不是通过；这一趟{reuse_text}")
            return 77
        say(f"  ✓ crates 变异表复跑：{len(rows)} 条里抓到 {len(caught)} 条、本次未跑 {len(not_run)} 条（本次未跑的逐条见上，不算抓到；这一趟不是整张表都红在点名的测试上）"
            f"；没红、无效、内存撞顶、超时、被总上限挤掉、排不上 0 条，锚点失效 0 行，每条一个自己的 TMPDIR、结束时都删掉了；这一趟{reuse_text}")
        say(f"  {counts['impact']}")
        return 0
    # ⚠️ 进程数不写进 stdout：写进去就让「GATE_MUTATION_WORKERS=1 与 =16 的 stdout 逐字相同」当场为假；复用几条接在成功句末尾「；这一趟现判」之后
    say(f"  ✓ crates 变异表复跑：{len(rows)} 条变异各自红在点名的测试上（原文都恰好命中一次、锚点失效 0 行；点名的测试在没改坏的副本上都恰好一个 ok；"
        f"每条在内存上限 {memory_max} 里跑、内存撞顶 0 条，每条经 run-with-memory-cap.sh 排队、被总上限挤掉与排不上 0 条，每条限时 {timeout_seconds} 秒、超时 0 条，"
        f"每条一个自己的 TMPDIR、结束时都删掉了；工作进程动态领活——谁先跑完谁再领下一条，不按条数预先切片；输出按表的行号排序、判定与进程数无关）"
        f"；这一趟{reuse_text}")
    # 牵连面（文件头「牵连面」）：成功句之后另起一行，只报数
    say(f"  {counts['impact']}")
    return 0


def reuse_summary_text(counts):
    """「现判 N 条、复用 M 条、拒算 R 条、本次未跑 K 条（现判的按栏：…）」：成功句末尾与「复用：」那一行用同一句。
    拒算的行照跑、不复用，不算进现判；本次未跑是判出来的一档（点名的测试打了 NOT_RUN_NO_GPU_ADAPTER），算在现判或拒算里。"""
    columns = "、".join(f"{title} {counts['fresh_columns'].get(name, 0)}" for name, title in REUSE_COLUMNS)
    return (f"现判 {counts['fresh_rows']} 条、复用 {counts['reused']} 条、拒算 {counts['refused_rows']} 条、本次未跑 {counts.get('not_run_rows', 0)} 条"
            f"（现判的按栏：{columns}）")


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
    # 替换文里新引的标识符找不到定义（replacement_undefined_identifiers）：只 static-check 判，checker-tier-crates-mutation-replay 派活之前不判（那边套用后编不过记无效）
    identifier_cache, undefined_rows, identifiers_unjudged = {}, 0, 0
    for row in rows:
        if not row.path.endswith(".rs") or anchor_problem(root, row) is not None:
            identifiers_unjudged += 1
            continue
        missing = replacement_undefined_identifiers(root, row, identifier_cache)
        if missing:
            undefined_rows += 1
            print(f"UNDEFINED\t{row.line_number}\t{row.name}\t{row.path}\t{','.join(missing)}")
    print(f"IDENTIFIERS_CHECKED\t{len(rows) - identifiers_unjudged}\t{identifiers_unjudged}")
    print(f"CHECKED\t{len(rows)}\t{scanned}\t{len(literals)}")
    return 1 if any(problems.values()) or outside_literals or undefined_rows else 0



def command_judging_units(arguments):
    """judging-units <根> [--units]：打「<判法单元指纹> <单元数>」；带 --units 再逐个打单元行（种类、文件:起-止、限定名、指纹）。"""
    if not arguments or len(arguments) > 2 or (len(arguments) == 2 and arguments[1] != "--units"):
        print("  ✗ 用法：crates-mutation-rows.py judging-units <根> [--units]")
        print("     → 怎么办：<根> 是被判那棵树的仓根；判法单元按本模块所在那一份仓算（文件头「判法单元」）")
        return 2
    try:
        key, references = judging_units(dict(os.environ))
    except (JudgingUnitsRefused, mutation_units.MutationUnitsError, OSError) as error:
        print(f"  ✗ 算不出判法单元指纹：{error}")
        print("     → 怎么办：判法单元的闭包里有拒算（getattr 变量名、折不出的 import 路径这一类）就改成字面写法；拒算的形态见 scripts/gate/lib_function_units.py 文件头「拒算」")
        return 2
    print(f"{key} {len(references)}")
    if len(arguments) == 2:
        for reference in references:
            print(f"unit\t{reference.kind}\t{reference.file}:{reference.lines}\t{reference.name}\t{reference.fingerprint}")
    return 0


def command_library_directory(arguments):
    if len(arguments) != 1:
        print("  ✗ 用法：crates-mutation-rows.py library-directory <根>")
        print("     → 怎么办：<根> 是被判那棵树的仓根（git 工作树）")
        return 2
    admission = admission_module()
    try:
        print(library_directory(arguments[0], dict(os.environ)))
    except (admission.InputManifestError, OSError) as error:
        print(f"  ✗ 取不到判定库的位置：{error}")
        print("     → 怎么办：被判的要是 git 工作树，或者设 GATE_MUTATION_RECORDS_HOME")
        return 2
    return 0


def command_toolchain(arguments):
    """toolchain <根>：打行键里的工具链行（rustc -vV、cargo -V、构建环境解析后的值），双机分片的驱动拿两台的比。"""
    if len(arguments) != 1:
        print("  ✗ 用法：crates-mutation-rows.py toolchain <根>")
        print("     → 怎么办：<根> 是被判那棵树的仓根（rustc、cargo 在那里起，从它往上找 .cargo/config.toml）")
        return 2
    try:
        lines = key_toolchain_lines(arguments[0], dict(os.environ))
    except mutation_units.MutationUnitsError as error:
        print(f"  ✗ 算不出工具链行：{error}")
        print("     → 怎么办：装好 Rust 工具链（rustc -vV、cargo -V 要跑得通），根 Cargo.toml 要读得懂")
        return 2
    print("\n".join(lines))
    return 0


def plan_context(root, environment):
    """plan、row-plan、import、--list-items 共用：读表、算上限、算每一行的键与判定（与 run_stage 同一份 plan_rows、decide_rows）；不写库。
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
    # 锚点失效的行（文件头「锚点失效」）先剔出去、单列：不算键（run_stage 不跑它们，row-plan 不让双机分片的驱动把它们分给第二台）
    anchor_broken = {} if break_is(environment, "anchor-rows-planned") else anchor_broken_rows(root, rows)
    anchor_rows = [(row, anchor_broken[row.line_number]) for row in rows if row.line_number in anchor_broken]
    rows = [row for row in rows if row.line_number not in anchor_broken]
    try:
        key_context, components, library = plan_rows(root, rows, environment, memory_max, timeout_seconds)
    except mutation_units.ClosureRefused as error:
        print_rejection(f"点名的测试在 cargo 参数选的这一组特性下认不出或不编（{len(error.failures)} 处），键算不出来", error.detail_lines(), CLOSURE_REFUSED_HOWTO)
        return None
    except (JudgingUnitsRefused, mutation_units.MutationUnitsError, admission.InputManifestError, HeavyThreadDefaultError, OSError) as error:
        print(f"  ✗ 算不出判法单元指纹、工具链、依赖版本或判定库的位置：{error}")
        print("     → 怎么办：被判的要是 git 工作树（或设 GATE_MUTATION_RECORDS_HOME）、rustc -vV 与 cargo -V 跑得通、Cargo.lock 读得懂")
        return None
    return dict(rows=rows, components=components, key_context=key_context, library=library, memory_max=memory_max, timeout_seconds=timeout_seconds,
                decided=decide_rows(library, rows, components, key_context, environment), timings_directory=os.path.join(library.directory, "timings"),
                anchor_rows=anchor_rows)


def command_row_plan(arguments):
    """每一条成形的变异一行：行号、行键（拒算写 -）、这一趟会不会复用、上一次现跑的秒数（见文件头 row-plan）。"""
    if len(arguments) != 1:
        print("  ✗ 用法：crates-mutation-rows.py row-plan <根>")
        print("     → 怎么办：<根> 是被判那棵树的仓根（git 工作树）")
        return 2
    environment = dict(os.environ)
    context = plan_context(arguments[0], environment)
    if context is None:
        return 2
    lines = []
    for row in context["rows"]:
        decision, _record = context["decided"][row.line_number]
        timing = read_timing(context["timings_directory"], row_content_hash(row))
        key = context["components"][row.line_number].row_key or "-"
        lines.append((row.line_number, f"{row.line_number}\t{key}\t{1 if decision.reuse else 0}\t{'-' if timing is None else f'{timing[0]:.3f}'}\t-"))
    # 锚点失效的行：没有键、不复用，第五列 anchor-broken（双机分片的驱动不把它们分给第二台）
    lines += [(row.line_number, f"{row.line_number}\t-\t0\t-\tanchor-broken") for row, _why in context["anchor_rows"]]
    for _line_number, text in sorted(lines):
        print(text)
    return 0


def command_plan(arguments):
    """只读：每一行会复用、现判还是拒算，栏与细节（变了哪几个单元），末行总数（文件头 plan）。"""
    if len(arguments) != 1:
        print("  ✗ 用法：crates-mutation-rows.py plan <根>")
        print("     → 怎么办：<根> 是被判那棵树的仓根（git 工作树）；只读，不编、不跑、不写")
        return 2
    environment = dict(os.environ)
    context = plan_context(arguments[0], environment)
    if context is None:
        return 2
    columns, reused, refused = collections.Counter(), 0, 0
    lines = []
    for row in context["rows"]:
        decision, _record = context["decided"][row.line_number]
        if decision.reuse:
            reused += 1
            lines.append((row.line_number, f"{row.line_number}\t{row.name}\t复用\t键相同\t{decision.detail}"))
        elif decision.column == "refused":
            refused += 1
            lines.append((row.line_number, f"{row.line_number}\t{row.name}\t拒算\t拒算\t{decision.detail}"))
        else:
            columns[decision.column] += 1
            lines.append((row.line_number, f"{row.line_number}\t{row.name}\t现判\t{COLUMN_TITLES[decision.column]}\t{decision.detail}"))
    # 锚点失效的行（文件头「锚点失效」）：不算键、这一趟不跑
    lines += [(row.line_number, f"{row.line_number}\t{row.name}\t锚点失效\t锚点失效\t{why}") for row, why in context["anchor_rows"]]
    for _line_number, text in sorted(lines):
        print(text)
    print(f"合计 {len(context['rows']) + len(context['anchor_rows'])} 行：复用 {reused} 行、现判 {sum(columns.values())} 行（"
          + "、".join(f"{title} {columns.get(name, 0)}" for name, title in REUSE_COLUMNS) + f"）、拒算 {refused} 行、锚点失效 {len(context['anchor_rows'])} 行")
    return 0


def command_import(arguments):
    """import <根> <另一台的库目录>：把另一台这一趟判出来的记录并进本机那一份库（文件头 import）。"""
    if len(arguments) != 2:
        print("  ✗ 用法：crates-mutation-rows.py import <根> <另一台的库目录>")
        print(f"     → 怎么办：<另一台的库目录> 是另一台那一趟 {LIBRARY_DIRECTORY_NAME}/ 拷过来的那一份（records/、baselines/、coverage/、compositions/、units/）")
        return 2
    root, source_directory = arguments
    environment = dict(os.environ)
    if not os.path.isdir(source_directory):
        print(f"  ✗ {source_directory} 不是目录")
        print(f"     → 怎么办：给另一台那一趟 {LIBRARY_DIRECTORY_NAME}/ 拷过来的那一份目录")
        return 2
    context = plan_context(root, environment)
    if context is None:
        return 2
    judging = context["key_context"].judging
    wanted = {"records": {components.row_key for components in context["components"].values() if components.row_key},
              "baselines": {components.baseline_key for components in context["components"].values() if components.baseline_key},
              "coverage": {components.coverage_id for components in context["components"].values() if components.coverage_id}}
    source, target = mutation_units.Library(source_directory, RECORD_HEADER), context["library"]
    unchecked = break_is(environment, "importunchecked")
    accepted, rejected = [], []
    for kind, key_field in (("records", "row_key"), ("baselines", "baseline_key")):
        directory = os.path.join(source_directory, kind)
        for name in sorted(os.listdir(directory)) if os.path.isdir(directory) else []:
            record = source.read(kind, name)
            problems = []
            if record is None:
                problems.append("不是一份读得出的记录文件")
            else:
                if record.get(key_field) != name:
                    problems.append("文件名与记录里的键对不上")
                if record.get("judging") != judging:
                    problems.append(f"判法单元指纹 {record.get('judging', '')[:16]}… 不是本机此刻的 {judging[:16]}…")
                if name not in wanted[kind]:
                    problems.append(f"键不是本机 {TABLE} 里任何一行照本机此刻的源码、判法单元、工具链与依赖算得出的")
            if unchecked:
                problems = []
            if problems:
                rejected.append((f"{kind}/{name}", "；".join(problems)))
                continue
            target.write(kind, name, record)
            accepted.append(f"{kind}/{name}")
    coverage_directory = os.path.join(source_directory, "coverage")
    for name in sorted(os.listdir(coverage_directory)) if os.path.isdir(coverage_directory) else []:
        identity = name[:-len(".json.gz")] if name.endswith(".json.gz") else None
        value = source.read_json("coverage", identity) if identity else None
        if value is None or (identity not in wanted["coverage"] and not unchecked):
            rejected.append((f"coverage/{name}", "不是本机任何一条点名测试此刻的覆盖号，或读不出"))
            continue
        target.write_json("coverage", identity, value)
        accepted.append(f"coverage/{name}")
    for kind in ("compositions", "units"):
        directory = os.path.join(source_directory, kind)
        for name in sorted(os.listdir(directory)) if os.path.isdir(directory) else []:
            if not name.endswith(".json.gz"):
                continue
            value = source.read_json(kind, name[:-len(".json.gz")])
            if value is None:
                continue
            if kind == "units":
                existing = target.read_json("units", name[:-len(".json.gz")]) or {}
                existing.update(value)
                value = existing
            elif os.path.exists(target.path(kind, name)):
                continue
            target.write_json(kind, name[:-len(".json.gz")], value)
    print(f"  … 导入：收了 {len(accepted)} 条、拒了 {len(rejected)} 条（写进 {target.directory}）")
    if rejected:
        print(f"  ✗ 拒了 {len(rejected)} 条另一台的判定库记录（文件名、判法单元指纹或键与本机此刻对不上，照收会让本机复用别的输入下的结论）：")
        for name, reason in rejected:
            print(f"      {name}：{reason}")
        print("     → 怎么办：两台的判法单元、变异表、工具链（含构建环境）与依赖要同一套；源码不同的那几行键自然对不上，")
        print("       本机 checker-tier-crates-mutation-replay 照跑，不会少判。")
        return 1
    return 0


def command_list_items(arguments):
    """可点名的变异，一行一条（门禁 --list-items 调的就是这一份），每行末尾一段写这一行的键对不对得上；末行合计键对不上的条数。"""
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
    items = [(number, line.split("\t")) for number, line in enumerate(lines, 1) if not line.startswith("#") and line.strip()]
    # 锚点失效的那几行（文件头「锚点失效」）在逐行那一段末尾接一句；判法与 run_stage 同一份（anchor_problem），成形的行才判
    environment = dict(os.environ)
    try:
        table_rows = read_table(path, environment)[0]
    except (OSError, UnicodeDecodeError):
        table_rows = []
    anchor_broken = {} if break_is(environment, "anchor-rows-planned") else anchor_broken_rows(arguments[0], table_rows)
    for number, fields in items:
        anchor_note = f"；锚点失效：{anchor_broken[number]}" if number in anchor_broken else ""
        print(f"{fields[0]}\t第 {number} 行，改 {fields[1] if len(fields) > 1 else ''}{anchor_note}", flush=True)
    status_output = io.StringIO()
    with contextlib.redirect_stdout(status_output):
        context = plan_context(arguments[0], environment)
    if context is None:
        print("  · 键对不对得上没算出来：" + " ".join(line.strip() for line in status_output.getvalue().splitlines()))
        return 0
    counter = collections.Counter()
    for row in context["rows"]:
        decision, _record = context["decided"][row.line_number]
        state = "复用" if decision.reuse else "拒算" if decision.column == "refused" else "键对不上"
        counter[state] += 1
    anchor_named = "、".join(f"第 {row.line_number} 行 {row.name}" for row, _why in context["anchor_rows"])
    print(f"  · 键对不上 {counter['键对不上']} 条（这一趟要现判）、键对得上 {counter['复用']} 条（复用）、拒算 {counter['拒算']} 条（照跑）、"
          f"锚点失效 {len(context['anchor_rows'])} 行{'（不跑、整道判红：' + anchor_named + '）' if anchor_named else ''}；"
          f"逐行的栏与变了哪几个单元：python3 scripts/mutation/crates-mutation-rows.py plan {arguments[0]}")
    # 牵连面（文件头「牵连面」）：末了另起一行，只报数
    print(f"  · {impact_summary(context['rows'], context['components'], context['key_context'], environment, len(context['anchor_rows']))}")
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
# $FAKE_CARGO_ENVIRONMENT_LOG 设了，每起一次 test 再往它追加一行 JSON：收到的 *_THREADS、SINGLEFS_THREAD_CAP、CARGO_BUILD_JOBS、CARGO_TARGET_DIR、
# 崩溃判定库的两个变量，外加键 ARGUMENTS（这一次的参数，认得出是不是先编那一趟）；编译并行度、崩溃判定库每片各一份那几格读它。
# $FAKE_CARGO_BARRIER_DIR 设了，--no-run 且源码不是 $FAKE_CARGO_BARRIER_PRISTINE（lib.rs 的 sha256 前 12 位）的那一次（改坏了的变异先编那一趟）在里面按
# CARGO_TARGET_DIR 记一个文件，等到里面有两个（两个工作进程都到了）或 20 秒：造「两个工作进程同时在跑」。
# 测试那一趟的 `--` 之后 --exact 出现两次，照 libtest 报 error: Option 'exact' given more than once、退 101。
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
        recorded = {name: value for name, value in os.environ.items()
                    if name in names or name.endswith("_THREADS") or name == "SINGLEFS_THREAD_CAP" or name.startswith("CARGO_PROFILE_")}
        recorded["ARGUMENTS"] = " ".join(arguments[1:])
        environment_log.write(json.dumps(recorded, sort_keys=True) + "\n")
if os.path.isfile("Cargo.lock"):
    # 外部依赖（Cargo.lock 里带 source 的包）：这份编译目录里缺它们的产物就照真 cargo 的样子「编」一遍（<档>/deps 下的 rlib 与 .d、.fingerprint、build），
    # 往 $FAKE_CARGO_DEPENDENCY_LOG 记一行编译目录（设了才记）；先编一份依赖那几格数它
    profile_directory = os.path.join(os.environ.get("CARGO_TARGET_DIR", "target"), "debug")
    missing = []
    for package in tomllib.load(open("Cargo.lock", "rb")).get("package", []):
        if "source" in package:
            unit = hashlib.sha256((package["name"] + package["version"]).encode()).hexdigest()[:16]
            rlib = os.path.join(profile_directory, "deps", f"lib{package['name'].replace('-', '_')}-{unit}.rlib")
            if not os.path.isfile(rlib):
                missing.append((package, unit, rlib))
    for package, unit, rlib in missing:
        crate = package["name"].replace("-", "_")
        registry_source = f"/crates-mutation-rows-selftest-registry/{package['name']}-{package['version']}/src/lib.rs"
        os.makedirs(os.path.dirname(rlib), exist_ok=True)
        with open(rlib, "w", encoding="utf-8") as handle:
            handle.write(package["name"])
        with open(os.path.join(profile_directory, "deps", f"{crate}-{unit}.d"), "w", encoding="utf-8") as handle:
            handle.write(f"{rlib}: {registry_source}\n\n{registry_source}:\n")
        for kind, name, file_name in ((".fingerprint", f"{package['name']}-{unit}", f"lib-{crate}"), ("build", f"{package['name']}-{unit[::-1]}", os.path.join("out", "generated.rs"))):
            os.makedirs(os.path.dirname(os.path.join(profile_directory, kind, name, file_name)), exist_ok=True)
            with open(os.path.join(profile_directory, kind, name, file_name), "w", encoding="utf-8") as handle:
                handle.write(unit + "\n")
    if missing and os.environ.get("FAKE_CARGO_DEPENDENCY_LOG"):
        with open(os.environ["FAKE_CARGO_DEPENDENCY_LOG"], "a", encoding="utf-8") as log:
            log.write(os.environ.get("CARGO_TARGET_DIR", "target") + "\n")
if os.environ.get("FAKE_CARGO_BARRIER_DIR") and "--no-run" in arguments and hashlib.sha256(source.encode()).hexdigest()[:12] != os.environ.get("FAKE_CARGO_BARRIER_PRISTINE"):
    import time
    barrier = os.environ["FAKE_CARGO_BARRIER_DIR"]
    os.makedirs(barrier, exist_ok=True)
    open(os.path.join(barrier, hashlib.sha256(os.environ["CARGO_TARGET_DIR"].encode()).hexdigest()[:12]), "w").close()
    barrier_deadline = time.monotonic() + 20
    while len(os.listdir(barrier)) < 2 and time.monotonic() < barrier_deadline:
        time.sleep(0.05)
if os.environ.get("FAKE_CARGO_HANG_FILE"):
    # 自证「被 TERM 停」那一格：起来记一笔（先写成临时文件再改名，读的一头看到文件就是写全的进程号），等到放行、父进程（工作进程）没了或 120 秒，记一笔「退了」再退 101
    import time
    parent = os.getppid()
    with open(os.environ["FAKE_CARGO_HANG_FILE"] + ".part", "w", encoding="utf-8") as handle:
        handle.write(str(parent))
    os.rename(os.environ["FAKE_CARGO_HANG_FILE"] + ".part", os.environ["FAKE_CARGO_HANG_FILE"])
    deadline = time.monotonic() + 120
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
    # 增量产物（rustc 每编一次改坏的副本往 incremental 里加的会话目录）：FAKE_CARGO_INCREMENTAL_BYTES 设了，每次先编往里写这么多字节（编译目录上限那一格读）
    if os.environ.get("FAKE_CARGO_INCREMENTAL_BYTES"):
        session = os.path.join(os.environ["CARGO_TARGET_DIR"], "debug", "incremental", "session-" + str(os.getpid()))
        os.makedirs(session, exist_ok=True)
        with open(os.path.join(session, "dep-graph.bin"), "wb") as handle:
            handle.write(os.urandom(int(os.environ["FAKE_CARGO_INCREMENTAL_BYTES"])))
    for package in packages:
        crate = os.path.join(os.getcwd(), "crates", package)
        sources = sorted(os.path.relpath(os.path.join(directory, name), os.getcwd()) for directory, _names, names in os.walk(os.path.join(crate, "src"))
                         for name in names if name.endswith(".rs"))
        executable = os.path.join(deps, f"{package}-0000000000000000")
        # fresh 照 cargo 按修改时刻判的样子：产物在、而且不比这个包底下最新的文件旧（先编一份依赖那一格核摸过的包有没有重编读它）
        newest_source = max((os.stat(os.path.join(directory, name)).st_mtime_ns for directory, _names, names in os.walk(crate) for name in names), default=0)
        fresh = os.path.isfile(executable) and os.stat(executable).st_mtime_ns >= newest_source
        with open(executable, "w", encoding="utf-8") as handle:
            json.dump({"root": os.getcwd(), "sources": sources}, handle)
        with open(executable + ".d", "w", encoding="utf-8") as handle:
            handle.write(f"{executable}: {' '.join(sources)}\n\n")
        print(json.dumps({"reason": "compiler-artifact", "manifest_path": os.path.join(crate, "Cargo.toml"), "target": {"kind": ["lib"], "name": package},
                          "profile": {"test": True}, "filenames": [executable], "executable": executable, "fresh": fresh}))
    # 剖析文件只在插桩编译时写（参数或 RUSTFLAGS / CARGO_ENCODED_RUSTFLAGS 里带 -Cinstrument-coverage）：基线与变异先编的那一趟不插桩，不写
    instrumented = any("instrument-coverage" in word for word in sys.argv[1:]) or any(
        "instrument-coverage" in os.environ.get(name, "") for name in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"))
    if instrumented:
        stray = os.path.abspath((os.environ.get("LLVM_PROFILE_FILE") or "default_%m_%p.profraw").replace("%p", str(os.getpid())).replace("%m", "0"))
        os.makedirs(os.path.dirname(stray), exist_ok=True)
        with open(stray, "w", encoding="utf-8") as handle:
            handle.write("{}")
        if os.environ.get("FAKE_CARGO_PROFILE_LOG"):
            with open(os.environ["FAKE_CARGO_PROFILE_LOG"], "a", encoding="utf-8") as log:
                log.write(os.getcwd() + "\t" + stray + "\n")
    sys.exit(0)
after = arguments[arguments.index("--") + 1:] if "--" in arguments else []
if after.count("--exact") > 1:
    print("error: Option 'exact' given more than once", file=sys.stderr)
    sys.exit(101)
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
# 自证用的假内存包装：--check 退 0；其余去掉上限那一个参数、原样执行命令；FAKE_MEMORY_CAP_LOG 设了，先记一行「<上限><TAB><限时或「不限时」><TAB><命令>」（逐行上限那一格读）
if [[ "${1:-}" == "--check" ]]; then exit 0; fi
if [[ -n "${FAKE_MEMORY_CAP_LOG:-}" ]]; then printf '%s\t%s\t%s\n' "$1" "${RUN_WITH_MEMORY_CAP_TIME_LIMIT:-不限时}" "${*:2}" >>"$FAKE_MEMORY_CAP_LOG"; fi
# FAKE_MEMORY_CAP_OOM_ON 设了，命令行里有它这一段（例 /line-3-build cargo：变异表第 3 行先编那一趟；/line-3 cargo：测试那一趟）就照真包装撞顶的样子退 250、命令不跑
if [[ -n "${FAKE_MEMORY_CAP_OOM_ON:-}" && " ${*:2} " == *"$FAKE_MEMORY_CAP_OOM_ON"* ]]; then
  echo "run-with-memory-cap: 撞了内存上限 $1（scope singlefs-memory-cap-selftest 的 Result=oom-kill，命令退出码 143）" >&2
  exit 250
fi
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

SELFTEST_TESTS = '''
#[cfg(test)]
mod tests {
    #[test]
    fn adds() { assert_eq!(super::add(2, 3), 5); }
    #[test]
    fn doubles() { assert_eq!(super::double(3), 6); }
    #[test]
    fn negates() { assert_eq!(super::negate(3), -3); }
    #[test]
    fn halves() { assert_eq!(super::halve(6), 3); }
    #[test]
    fn reads_answer() {}
    #[test]
    fn gpu_parity() {}
}
'''
# 样本 crate：假 cargo 照「// TEST <名字> NEEDS <原文>」判红不红；下面的 #[test] 函数只给单元键认点名的测试是哪一个单元（文件头「行键」），跑不跑、红不红都照假 cargo
SELFTEST_SOURCE = '''// 本模块自证的样本 crate（假 cargo 按下面几行判）
// TEST adds NEEDS left + right
// TEST doubles NEEDS value * 2
// TEST negates NEEDS 0 - value
// TEST halves NEEDS value / 2
pub fn add(left: u32, right: u32) -> u32 { left + right }
pub fn double(value: u32) -> u32 { value * 2 }
pub fn negate(value: i32) -> i32 { 0 - value }
pub fn halve(value: u32) -> u32 { value / 2 }
''' + SELFTEST_TESTS
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
# 产品 GPU 包的真显卡对拍开的是 tests-on-a-real-gpu（不是 gpu）：同样要经清场起（文件头「带 gpu 特性的格经清场起」）
SELFTEST_PRODUCT_GPU_FEATURE_ROW = ("product-gpu-double-becomes-triple\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ value * 3 }\t"
                                    "-p tiny --features tests-on-a-real-gpu --lib -- doubles\tdoubles")
# 同一个特性、点名的测试在没卡的机器上打 NOT_RUN_NO_GPU_ADAPTER 后返回成功（SELFTEST_NOT_RUN_TEST 那一条用例）
SELFTEST_PRODUCT_GPU_NOT_RUN_ROW = ("product-card-path-changed\tcrates/tiny/src/lib.rs\t{ value / 2 }\t{ value / 4 }\t"
                                    "-p tiny --features tests-on-a-real-gpu --lib -- gpu_parity\tgpu_parity")
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
# 插桩编译那一格的样本 crate：helps 只执行 add（RUNS），两行同点 adds（同一条测试共用一份覆盖），一共 3 条测试记 3 份覆盖
SELFTEST_COVERAGE_SOURCE = '''// 覆盖那一格的样本 crate（假 cargo 按下面几行判）
// TEST adds NEEDS left + right
// TEST doubles NEEDS value * 2
// TEST helps NEEDS value + 7 RUNS helps,add
pub fn add(left: u32, right: u32) -> u32 { left + right }
pub fn double(value: u32) -> u32 { value * 2 }
pub fn helper(value: u32) -> u32 { value + 7 }

#[cfg(test)]
mod tests {
    #[test]
    fn adds() { assert_eq!(super::add(2, 3), 5); }
    #[test]
    fn doubles() { assert_eq!(super::double(3), 6); }
    #[test]
    fn helps() { assert_eq!(super::add(2, 3), 5); }
}
'''
SELFTEST_COVERAGE_ROWS = [
    "add-becomes-mul\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left * right }\t-p tiny --lib -- adds\tadds",
    "add-becomes-sub\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left - right }\t-p tiny --lib -- adds\tadds",
    "double-becomes-triple\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ value * 3 }\t-p tiny --lib -- doubles\tdoubles",
    "helper-becomes-eight\tcrates/tiny/src/lib.rs\t{ value + 7 }\t{ value + 8 }\t-p tiny --lib -- helps\thelps",
]
# 单元键那几格（第二轮复查 controls 路第 5 节 crates 变异那一行的 S34、S30、N33、N34、N45，加上被改坏的单元、点名测试的闭包、常量、覆盖复用基线几格）的样本 crate：
#   check_root 里有两条变异（root-457、root-double），点名 checks_root；sum 的变异点名 sums（它还调 pad）；helper 的变异点名 helps（它只调 add，够不着 helper）；
#   taken 的变异点名 branches（它静态上还调 untaken，执行的只有 taken）；OFFSET 是 check_root 用的常量（V5 那一类）
SELFTEST_UNIT_SOURCE = '''// 单元键那几格的样本 crate（假 cargo 按下面几行判）
// TEST checks_root NEEDS slot + 457
// TEST checks_root NEEDS base * 2
// TEST sums NEEDS left + right
// TEST helps NEEDS value + 7 RUNS helps,add
// TEST branches NEEDS value + 9 RUNS branches,taken
pub const OFFSET: u32 = 95;
pub fn check_root(slot: u32) -> u32 {
    let base = slot + 457;
    let floor = base.max(OFFSET);
    floor * 0 + base * 2
}
pub fn pad(value: u32) -> u32 { value + 0 }
pub fn sum(left: u32, right: u32) -> u32 { left + right }
pub fn add(first: u32, second: u32) -> u32 { first + second }
pub fn helper(value: u32) -> u32 { value + 7 }
pub fn taken(value: u32) -> u32 { value + 9 }
pub fn untaken(value: u32) -> u32 { value + 10 }

#[cfg(test)]
mod tests {
    #[test]
    fn checks_root() { assert_eq!(super::check_root(1), 916); }
    #[test]
    fn sums() { assert_eq!(super::sum(super::pad(2), 3), 5); }
    #[test]
    fn helps() { assert_eq!(super::add(2, 3), 5); }
    #[test]
    fn branches() { let value = super::taken(1); if value == 0 { super::untaken(value); } }
}
'''
SELFTEST_UNIT_ROWS = [
    "root-457\tcrates/tiny/src/lib.rs\t= slot + 457;\t= slot + 458;\t-p tiny --lib -- checks_root\tchecks_root",
    "root-double\tcrates/tiny/src/lib.rs\t0 + base * 2\t0 + base * 3\t-p tiny --lib -- checks_root\tchecks_root",
    "sum-sub\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left - right }\t-p tiny --lib -- sums\tsums",
    "helper-eight\tcrates/tiny/src/lib.rs\t{ value + 7 }\t{ value + 8 }\t-p tiny --lib -- helps\thelps",
    "taken-eleven\tcrates/tiny/src/lib.rs\t{ value + 9 }\t{ value + 11 }\t-p tiny --lib -- branches\tbranches",
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
        self.not_run = []   # 本次未跑的格（skip）
        self.bin_directory = os.path.join(work, "bin")
        selftest_write(os.path.join(self.bin_directory, "cargo"), FAKE_CARGO, executable=True)
        self.llvm_directory = os.path.join(work, "llvm")
        for tool in LLVM_TOOLS:
            selftest_write(os.path.join(self.llvm_directory, tool), FAKE_LLVM_TOOL, executable=True)
        self.runner = os.path.join(work, "fake-memory-cap.sh")
        selftest_write(self.runner, FAKE_MEMORY_CAP_RUNNER, executable=True)
        self.log = os.path.join(work, "cargo.log")
        # BINDGEN_EXTRA_CLANG_ARGS 也不带进来：调用方 export 了它，副本里没有 .cargo/ 照样编得过，「拷贝范围含 .cargo/」那一格就红不出来；
        # RUSTFLAGS 一类也不带：假 cargo 不看它，带进来只会让工具链行随调用方变
        self.base_environment = {key: value for key, value in os.environ.items()
                                 if key not in STAGE_VARIABLES and key not in mutation_units.BUILD_VARIABLES and not key.startswith("CARGO_PROFILE_")
                                 and key not in ("BINDGEN_EXTRA_CLANG_ARGS", "FAKE_CARGO_VERSION", "FAKE_LLVM_VERSION")}
        self.base_environment.update(PATH=self.bin_directory + os.pathsep + os.environ.get("PATH", ""), FAKE_CARGO_LOG=self.log,
                                     GATE_MUTATION_MEMORY_CAP_RUNNER=self.runner, GATE_MUTATION_TARGET_DIR=os.path.join(work, "target"),
                                     GATE_MUTATION_WORKERS="1", GATE_MUTATION_LLVM_TOOLS=self.llvm_directory)
        # 目标盘余量：别的格不比它，给一个用不完的假值；磁盘余量那几格自己设
        self.base_environment[DISK_FREE_VARIABLE] = "100T"
        # 复用哨兵：别的格比的是现判与复用的条数，键相同的全当哨兵、不抽强制现判；reuse-three-questions 与 reuse-sentinel 两格自己设
        self.base_environment[reuse_checks.SENTINEL_COUNT_VARIABLE] = "1000"
        self.base_environment.pop(reuse_checks.SENTINEL_SEED_VARIABLE, None)
        # 串行段声明：调用方设了的不带进来（自证各趟不往调用方的声明文件里写），声明那一格自己设
        self.base_environment.pop("SINGLEFS_SERIAL_DECLARATIONS", None)
        # 清场包装（文件头「带 gpu 特性的格经清场起」）：别的格里带 gpu 特性的那几次经一份只原样起命令的假包装（不读多机配置、不清场、不上卡）；
        # 清场那几格换成真的 gpu-quiesce-run.sh 与只写假清场命令的多机配置
        self.quiesce_passthrough = os.path.join(work, "fake-gpu-quiesce-run.sh")
        selftest_write(self.quiesce_passthrough, '#!/usr/bin/env bash\n# 自证用的假清场包装：去掉 -- 原样起命令\n[[ "${1:-}" == -- ]] && shift\nexec "$@"\n',
                       executable=True)
        self.base_environment["GATE_MUTATION_GPU_QUIESCE_RUNNER"] = self.quiesce_passthrough
        # 交给带 gpu 特性那几格的多机配置（文件头「带 gpu 特性的格把多机配置交给测试、跑完核真用了卡」）：调用方设的不带进来，
        # 本模块所在仓那一份也不读（有没有、写了什么随机器变），换成临时目录里的一份；交多机配置那几格自己设
        self.base_environment.pop("SINGLEFS_MULTI_HOST_CONFIG", None)
        self.multi_host_configuration = os.path.join(work, "selftest-multi-host", "multi-host.env")
        selftest_write(self.multi_host_configuration, "# 自证用的多机配置：假 cargo 不读它，交多机配置那几格只核它的路径到没到测试的环境里\nENABLE_GPU=1\n")
        self.base_environment["GATE_MUTATION_MULTI_HOST_CONFIG"] = self.multi_host_configuration

    def run(self, root, judging_repository=None, **changes):
        """在 root 上跑一趟 run_stage：(退出码, stdout, stderr, 这一趟假 cargo 起的每一行)。"""
        with contextlib.suppress(FileNotFoundError):
            os.unlink(self.log)
        environment = dict(self.base_environment, **changes)
        stdout, stderr = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            exit_code = run_stage(root, environment, judging_repository)
        invocations = []
        if os.path.exists(self.log):
            with open(self.log, encoding="utf-8") as handle:
                invocations = handle.read().splitlines()
        return exit_code, stdout.getvalue(), stderr.getvalue(), invocations

    def check(self, label, condition, detail=""):
        self.cells += 1
        if not condition:
            self.failures.append(f"{label}{('：' + detail) if detail else ''}")

    def skip(self, label, reason):
        """这一格的前提没造出来：记本次未跑（不算对、也不算错，不进格数），run_selftest 收尾逐格列出，别的格都对时整趟退 77。"""
        self.not_run.append(f"{label}：{reason}")


def compile_invocation(line):
    """基线与变异先编的那一趟（文件头「限时」：cargo test <-- 之前那一段> --no-run，不带 --message-format；记覆盖的插桩编译带）。"""
    return "--no-run" in line and "--message-format" not in line


def plain_invocations(invocations):
    """假 cargo 的记录里基线与变异那几次（记覆盖的 --no-run 与 --exact 两次、先编的那一趟不算）。"""
    return [line for line in invocations if "--no-run" not in line and "--exact" not in line]


def coverage_invocations(invocations):
    return [line for line in invocations if ("--no-run" in line or "--exact" in line) and not compile_invocation(line)]


def mutation_invocations(invocations, pristine_hash):
    """假 cargo 的记录里改坏了源码的那几次（lib.rs 的哈希不是没改坏的那一份；先编的那一趟不算）。"""
    return [line for line in invocations if not line.endswith("\t" + pristine_hash) and not compile_invocation(line)]


def selftest_verdict_view(stdout):
    """stdout 去掉随记录变的那几处（「复用：」「覆盖：」两行、抓到那一行末尾的「（现判：…）」「（复用）」「（拒算）」、成功句末尾「；这一趟现判 …」），剩下的是判定本身。"""
    lines = []
    for line in stdout.splitlines():
        if line.startswith("  复用：") or line.startswith("  覆盖："):
            continue
        line = re.sub(r"（现判：[^）]*）$|（复用）$|（拒算）$", "", line)
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
    """stdout 里抓到那几行：{变异名: 「现判：<栏>」「复用」或「拒算」}。"""
    found = {}
    for match in re.finditer(r"^  ✓ (\S+)：\S+ 红了（(现判：[^）]*|复用|拒算)）$", stdout, re.M):
        found[match.group(1)] = match.group(2)
    return found


def library_of(repository):
    return os.path.join(repository, ".git", LIBRARY_DIRECTORY_NAME)


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


# 自证「按额度排队」那一格的假内存包装：--check 退 0；其余往 $SPAN_LOG 记「起」「止」两行（每行一次写入、O_APPEND，行序就是真实的先后），
# 一行的格式「<起或止><TAB><起头名，line-3-build、line-3、baseline-2 这一类><TAB><收到的上限>」；命令前睡一会儿，几格才在时间上重叠，上限 24G 的睡得更久；
# 环境变量 STAGGER_UP_TO_LINE 设了（没设不错开）：变异表行号不超过它的那几行（头一批同时派出去的）按行号错开睡的时长（每多一行多睡 0.1 秒），它们一个一个收完、不挤在同一轮：
# 父进程每收完一个就补派一件，24G 那一行被越过的次数才由上限定、不由「同一轮收完几个」定
SELFTEST_BUDGET_WRAPPER = r'''#!/usr/bin/env bash
if [[ "${1:-}" == "--check" ]]; then exit 0; fi
cap="$1"; shift
stamp=""
for word in "$@"; do
  if [[ "$word" == */@STAMP_DIRECTORY@/* ]]; then stamp="${word##*/}"; fi
done
printf 'start\t%s\t%s\n' "$stamp" "$cap" >> "$SPAN_LOG"
line_number=""
if [[ "$stamp" =~ ^line-([0-9]+) ]]; then line_number="${BASH_REMATCH[1]}"; fi
if [[ "$cap" == "24G" ]]; then
  sleep 1.2
elif [[ -n "$line_number" && "$line_number" -le "${STAGGER_UP_TO_LINE:-0}" ]]; then
  milliseconds=$((400 + 100 * line_number))
  sleep "$((milliseconds / 1000)).$(printf '%03d' $((milliseconds % 1000)))"
else
  sleep 0.4
fi
"$@"
status=$?
printf 'end\t%s\t%s\n' "$stamp" "$cap" >> "$SPAN_LOG"
exit "$status"
'''


def simulate_budget_queue(sizes, outer, slots, bypass_limit):
    """pick_ready_job 的离散模拟（自证用，不起进程）：sizes[i] 是第 i 件的上限，件按下标的次序排队，每次最多 slots 件在跑，最早起的先收。
    回 (起的次序, 在跑的上限之和的峰值, {下标: 起的时候是不是什么都没在跑})。"""
    ready = [(0, index, index, "mutation", index) for index in range(len(sizes))]
    heapq.heapify(ready)
    running, started, peak, alone, bypass_counts = [], [], 0, {}, {}
    while ready or running:
        while ready and len(running) < slots:
            entry = pick_ready_job(ready, sum(sizes[index] for index in running), len(running), outer, lambda each: sizes[each[4]], bypass_counts, bypass_limit)
            if entry is None:
                break
            alone[entry[4]] = not running
            running.append(entry[4])
            started.append(entry[4])
            peak = max(peak, sum(sizes[index] for index in running))
        if running:
            running.pop(0)
    return started, peak, alone


def selftest_row_memory_budget_cells(selftest, work):
    """按额度排队（文件头「工作进程数」）：工作进程数按每条的默认上限算，个别行的大上限不压低它；派活时每一格按自己的上限从外层额度里扣。
    先用 pick_ready_job 的离散模拟判排队本身（不起进程）：外层 36G、一格 24G 加二十格 4G，在跑的上限之和不超过 36G，24G 那一格被越过的次数在限内、跑上了；
    比外层还大的那一格什么都没在跑时单独起；外层读不到（outer_bytes 为 None）时照队首的次序派、不扣额度。
    再起一趟 run_stage：外层 36G、变异表里一行写了 24G 另二十行 4G，假包装记每一次包装调用的起止与收到的上限，核开的工作进程多于 1 个、
    任一时刻在跑的各次包装的上限之和不超过 36G（各格先编与跑测试两次包装接着来，每一格的占用不超过它那一格的上限）、24G 那一行两次包装都跑完、没被后面的小行饿死。
    越过次数按父进程派活的次序数，不按假包装日志里 start 的先后：各工作进程写 start 的先后不是父进程派活的先后（两件紧挨着派出去的，谁的包装先起看调度，单跑也约四次里红一次）。
    做法：selftest.run 前后把本模块里的 pick_ready_job 换成一层记录（照调原函数、按次序记下派出的变异行号），finally 里换回去；假包装日志只核上限之和、同时在跑、24G 那一行跑完。
    头一批同时派出去的几行（假包装按行号错开睡的时长）一个一个收完，所以越过次数由上限定：没弄坏时恰为 worker_count，不是「同一轮收完几个」的运气。
    弄坏开关 largestrowmemory（工作进程数按最大那一行算，只开 1 个，派活也不排队）、rowmemoryunbudgeted（不扣额度）、bypassunbounded（不设越过次数的限，越过 15 次）下这一格判错；
    pick_ready_job 里越过上限的判断差一（`>=` 写成 `>`，越过 10 次）同样判错，那是改源码，不是开关。"""
    environment = selftest.base_environment
    slots = 9
    limit = None if break_is(environment, "bypassunbounded") else slots
    sizes = [4] * 5 + [24] + [4] * 15
    started, peak, _alone = simulate_budget_queue(sizes, 36, slots, limit)
    bypassers = [index for index in started[:started.index(5)] if index > 5] if 5 in started else []
    selftest.check("按额度排队（离散模拟）：外层 36G、一格 24G 加二十格 4G，在跑的上限之和不超过 36G，24G 那一格跑上了，被后面的小格越过的次数在 1 到 worker_count 之间",
                   sorted(started) == list(range(len(sizes))) and peak <= 36 and 1 <= len(bypassers) <= slots,
                   f"起的次序 {started}；峰值 {peak}；越过它的 {len(bypassers)} 格（应在 1 到 {slots} 之间）")
    started_small, peak_small, _alone = simulate_budget_queue([4] * 12, 36, slots, limit)
    selftest.check("按额度排队（离散模拟）：格格都扣得下时照队里的次序派、不乱序", started_small == list(range(12)) and peak_small <= 36, f"起的次序 {started_small}；峰值 {peak_small}")
    started_big, peak_big, alone_big = simulate_budget_queue([4, 4, 40, 4], 36, 4, limit)
    selftest.check("按额度排队（离散模拟）：一格的上限比整个外层还大（40G 对 36G）时等在跑的收完、什么都没在跑时单独起",
                   sorted(started_big) == [0, 1, 2, 3] and alone_big.get(2) is True and peak_big == 40, f"起的次序 {started_big}；单独起的 {alone_big}；峰值 {peak_big}")
    started_free, peak_free, _alone = simulate_budget_queue([24] * 5, None, 3, limit)
    selftest.check("外层上限读不到（outer_bytes 为 None）时不扣额度：照队首的次序派，在跑的 3 格各 24G 也派", started_free == [0, 1, 2, 3, 4] and peak_free == 72, f"起的次序 {started_free}；峰值 {peak_free}")
    sixteen = dict(environment, GATE_MUTATION_WORKERS="16")
    counted, counted_head = worker_count_for(21, dict(sixteen, **{OUTER_MEMORY_CAP_VARIABLE: "36G"}), "4G", "24G")
    unset_count, unset_head = worker_count_for(21, dict(sixteen), "4G", "24G")
    selftest.check("工作进程数按每条的默认上限算（36G ÷ 4G 开 9 个，不是 36G ÷ 最大那一行的 24G 开 1 个），头一行报「逐行上限最大 24G、按额度排队」；外层读不到时头一行没有这一句、照 16 个开",
                   counted == 9 and "逐行上限最大 24G、按额度排队" in counted_head and unset_count == 16 and "按额度排队" not in unset_head,
                   f"{counted}：{counted_head}；{unset_count}：{unset_head}")
    huge_count, huge_head = worker_count_for(21, dict(sixteen, **{OUTER_MEMORY_CAP_VARIABLE: "36G"}), "4G", "40G")
    selftest.check("最大那一格比外层还大（40G 对 36G）时头一行写明等在跑的收完、单独起，工作进程数照旧按默认上限算",
                   huge_count == 9 and "最大那一件比外层还大：等在跑的收完、单独起" in huge_head, f"{huge_count}：{huge_head}")
    # 起一趟 run_stage：变异表 21 行，budget-row-big 在第 6 个位置（前面 5 行是 4G，后面 15 行也是 4G），逐行上限给它 24G
    repository = os.path.join(work, "row-memory-budget")
    names = [f"budget-row-{index:02d}" for index in range(20)]
    names.insert(5, "budget-row-big")
    rows = [f"{name}\tcrates/tiny/src/lib.rs\t{{ left + right }}\t{{ left * (right + {index}) }}\t-p tiny --lib -- adds\tadds" for index, name in enumerate(names)]
    build_selftest_repository(repository, rows)
    big_line = names.index("budget-row-big") + 2   # 变异表第 1 行是表头
    selftest_write(os.path.join(repository, LIGHT_ROWS_TABLE), "# 自证用的轻型变异登记表（只放逐行上限那一行）\n"
                   "# 逐行上限：budget-row-big\t24G\t-\t# 自证：比整张表的 4G 大\n")
    wrapper = os.path.join(work, "budget-wrapper.sh")
    selftest_write(wrapper, SELFTEST_BUDGET_WRAPPER.replace("@STAMP_DIRECTORY@", START_STAMP_DIRECTORY_NAME), executable=True)
    span_log = os.path.join(work, "budget-spans.log")
    with contextlib.suppress(FileNotFoundError):
        os.unlink(span_log)
    # 派活的次序在父进程里记：run_pool 按本模块的全局名调 pick_ready_job，selftest.run 又在同一个进程里调 run_stage，所以换掉全局名就看得到每一次派出的变异行
    dispatched = []
    module_globals = globals()
    real_pick_ready_job = module_globals["pick_ready_job"]

    def recording_pick_ready_job(*arguments):
        chosen = real_pick_ready_job(*arguments)
        if chosen is not None and chosen[3] == "mutation":
            dispatched.append(chosen[4].line_number)
        return chosen
    module_globals["pick_ready_job"] = recording_pick_ready_job
    try:
        exit_code, stdout, stderr, _invocations = selftest.run(
            repository, GATE_MUTATION_WORKERS="16", GATE_MUTATION_MEMORY_MAX="4G", GATE_MUTATION_TIMEOUT="1800", GATE_MUTATION_START_OVER="1",
            GATE_MUTATION_MEMORY_CAP_RUNNER=wrapper, SPAN_LOG=span_log, STAGGER_UP_TO_LINE=str(slots + 2), **{OUTER_MEMORY_CAP_VARIABLE: "36G"})
    finally:
        module_globals["pick_ready_job"] = real_pick_ready_job
    events = []
    with contextlib.suppress(OSError):
        with open(span_log, encoding="utf-8") as handle:
            events = [line.split("\t") for line in handle.read().splitlines()]
    opened, peak_sum, peak_open, peak_open_with_big = {}, 0, 0, 0
    big_stamps = {f"line-{big_line}", f"line-{big_line}-build"}
    for kind, stamp, cap in events:
        if kind == "start":
            opened[stamp] = bytes_of_size(cap) or 0
            peak_sum = max(peak_sum, sum(opened.values()))
            peak_open = max(peak_open, len(opened))
            if big_stamps & set(opened):
                peak_open_with_big = max(peak_open_with_big, len(opened))
        else:
            opened.pop(stamp, None)
    # 越过 = 父进程在派 24G 那一行之前派出去的、行号比它大的变异行
    jumped = [line for line in dispatched[:dispatched.index(big_line)] if line > big_line] if big_line in dispatched else []
    big_finished = ["end", f"line-{big_line}", "24G"] in events and ["end", f"line-{big_line}-build", "24G"] in events
    selftest.check("外层 36G、一行 24G 加二十行 4G：开的工作进程多于 1 个（36G ÷ 4G 开 9 个），头一行报逐行上限最大 24G、按额度排队",
                   exit_code == 0 and "这一轮开 9 个工作进程" in stderr and "逐行上限最大 24G、按额度排队" in stderr,
                   f"退 {exit_code}；{stderr[:900]}")
    selftest.check("外层 36G、一行 24G 加二十行 4G：任一时刻在跑的各次包装的上限之和不超过 36G，24G 那一行与 4G 的行同时在跑过",
                   bool(events) and peak_sum <= 36 * 1024 ** 3 and peak_open_with_big >= 2,
                   f"包装起止 {len(events)} 行；上限之和的峰值 {peak_sum / 1024 ** 3:g}G；同时在跑的峰值 {peak_open}；24G 在跑时同时在跑的峰值 {peak_open_with_big}")
    selftest.check("外层 36G、一行 24G 加二十行 4G：24G 那一行先编与跑测试两次都跑完了，判成抓到，后面的小行越过它的（按父进程派活的次序数）不超过 worker_count（9）行（没饿死）",
                   big_finished and "budget-row-big" in per_row_how(stdout) and len(per_row_how(stdout)) == 21 and 1 <= len(jumped) <= 9,
                   f"24G 那一行跑完 {big_finished}；抓到 {len(per_row_how(stdout))} 行；父进程派活的次序 {dispatched}，越过它的 {len(jumped)} 行（应在 1 到 9 之间）；{stdout[-300:]}")


def selftest_row_limit_cell(selftest, work):
    """逐行上限与限时不含重编译（文件头「逐行上限」与「基线」一段的限时）：轻型变异登记表里给 double-becomes-triple（变异表第 3 行）写 9G、2400 秒，
    假内存包装把每一次收到的上限、限时（没设记「不限时」）与命令记进 FAKE_MEMORY_CAP_LOG：那一行的先编与变异两趟都是 9G，先编不限时、变异限时 2400；
    别的三行照整张表的 4G、1800；它们共用的那一批基线取大的 9G、2400（先编那一趟不限时）。弄坏开关 rowlimitsignored（那一行照 4G、1800）、
    timelimitincludescompile（不先编，限时含重编译）下这一格判错。"""
    repository = os.path.join(work, "row-limits")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    selftest_write(os.path.join(repository, LIGHT_ROWS_TABLE), "# 自证用的轻型变异登记表（只放逐行上限那一行）\n"
                   "# 逐行上限：double-becomes-triple\t9G\t2400\t# 自证：比整张表的 4G、1800 秒大\n")
    cap_log = os.path.join(work, "row-limits-caps.log")
    exit_code, stdout, stderr, _invocations = selftest.run(repository, GATE_MUTATION_START_OVER="1", GATE_MUTATION_MEMORY_MAX="4G",
                                                           GATE_MUTATION_TIMEOUT="1800", FAKE_MEMORY_CAP_LOG=cap_log)
    calls = []
    with contextlib.suppress(OSError):
        with open(cap_log, encoding="utf-8") as handle:
            calls = [line.split("\t", 2) for line in handle.read().splitlines()]

    def seen(stamp):
        """这一个起头名（line-3、baseline-2 这一类）的 先编 / 跑 两趟各收到的 (上限, 限时)。"""
        found = {}
        for cap, limit, command in calls:
            match = re.search(r"/" + START_STAMP_DIRECTORY_NAME + r"/(\S+)", command)
            if match and match.group(1) in (stamp, stamp + "-build"):
                found["build" if match.group(1).endswith("-build") else "run"] = (cap, limit)
        return found

    wanted = {"line-3": {"build": ("9G", "不限时"), "run": ("9G", "2400")}, "line-2": {"build": ("4G", "不限时"), "run": ("4G", "1800")},
              "line-5": {"build": ("4G", "不限时"), "run": ("4G", "1800")}, "baseline-2": {"build": ("9G", "不限时"), "run": ("9G", "2400")}}
    got = {stamp: seen(stamp) for stamp in wanted}
    selftest.check("逐行上限：一行另给的上限与限时只给它与它所在的那一批基线，别的行照整张表的数；先编那一趟不限时、只有内存上限，再限时跑点名的测试",
                   exit_code == 0 and got == wanted and "逐行上限（" in stderr,
                   f"退 {exit_code}；收到的 {got}，应为 {wanted}；{stderr[-300:]}")
    # 逐行上限也不进行键（row-key-with-limits 那一格的口径）：把那一行改成 10G、3000 秒再跑一趟，一行都不现判
    selftest_write(os.path.join(repository, LIGHT_ROWS_TABLE), "# 自证用的轻型变异登记表（只放逐行上限那一行）\n"
                   "# 逐行上限：double-becomes-triple\t10G\t3000\t# 自证：改了上限与限时\n")
    again = selftest.run(repository, GATE_MUTATION_MEMORY_MAX="4G", GATE_MUTATION_TIMEOUT="1800")
    selftest.check("逐行上限不进行键：改那一行的上限与限时之后一行都不现判", again[0] == 0 and "复用：现判 0 条" in again[1],
                   f"退 {again[0]}：{again[1][-400:]}")


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


def selftest_host_lookup_cells(selftest, work):
    """内存包装现查别的机器打的那几行（HOST_LOOKUP_LINE）判法不读：同一份 cargo 输出、前面那几行内容不同，verdict_of_run 交的档与那一句逐字相同、
    不带那几行（弄坏开关 hostlookuplineskept 下判错）。包装 HostLookup 的每一处 say 都要被 HOST_LOOKUP_LINE 认、HostLookup 之外的不被它认
    （除了登记的同首那一句）；再造一份在 HostLookup 里多一处 say 的包装，要报出它（包装新加一处 say 而没登记就红）。"""
    row = Row(9, "selftest-host-lookup", "crates/selftest/src/lib.rs", "a", "b", "-p selftest", "tests::named", [])
    cargo_output = "running 1 test\ntest tests::named ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored\n"
    variants = ["run-with-memory-cap: 没查到别的机器：第二台没读到（ssh 退 255）\nrun-with-memory-cap: 现查别的机器：第二台此刻也放不下（可用 1.0 GiB）\n",
                "  → 第二台此刻放得下（可用 43.0 GiB、要 8.0 GiB）：改用 bash scripts/hosts/multi-host-run.sh --peer --memory-cap 4G --label moved-from-queue-1 -- cargo test\n"
                "  ! 告警记录写不进（OSError: 自证造的）：看门狗读不到这一条，靠上面那一行叫人看；设 SINGLEFS_CORE_USAGE_ALERTS 指到可写的文件\n"]
    verdicts = [verdict_of_run(row, 0, lines + cargo_output, "4G", "1800", environment=os.environ) for lines in variants]
    selftest.check("包装现查别的机器打的那几行内容不同，判出的档与那一句逐字相同、不带那几行",
                   verdicts[0] == verdicts[1] and verdicts[0][0] == "failure" and not any(word in verdicts[0][1] for word in ("别的机器", "此刻放得下", "告警记录写不进")),
                   f"{verdicts}")
    real_problems = host_lookup_say_problems(DEFAULT_MEMORY_CAP_RUNNER)
    with open(DEFAULT_MEMORY_CAP_RUNNER, encoding="utf-8") as handle:
        wrapper_text = handle.read()
    planted_anchor = "    def wait_thread(self):\n"
    planted = os.path.join(work, "host-lookup-planted-wrapper.sh")
    selftest_write(planted, wrapper_text.replace(planted_anchor, '    def planted(self):\n        say(f"run-with-memory-cap: 新加的一句：{self.unit}")\n\n' + planted_anchor, 1))
    planted_problems = host_lookup_say_problems(planted) if planted_anchor in wrapper_text else ["包装里找不到 wait_thread，造不出多一处 say 的包装"]
    selftest.check("包装 HostLookup 的每一处 say 都被 HOST_LOOKUP_LINE 认、HostLookup 里多一处没登记的 say 报得出",
                   not real_problems and any("新加的一句" in problem for problem in planted_problems), f"真包装 {real_problems}；多一处 say 的包装 {planted_problems}")


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
                       FAKE_CARGO_DONE_FILE=done, GATE_MUTATION_TARGET_DIR="")   # 编译目录建在这一趟的 shard_root 里（文件头「编译目录」），停完连同它删
    stderr_path = os.path.join(work, "signalled-stderr.txt")
    # stderr 落文件、不接管道：没接住信号时工作进程成了孤儿、还开着它，接管道就等不到头
    with open(stderr_path, "w", encoding="utf-8") as stderr_handle:
        child = subprocess.Popen([sys.executable, "-c", SELFTEST_STAGE_CHILD, MODULE_PATH, repository], env=environment,
                                 stdout=subprocess.DEVNULL, stderr=stderr_handle)
    # TERM 发在「$TMPDIR 下的临时目录建出来了、假 cargo 在工作进程里挂住了（标记文件里有工作进程的进程号）」之后：run_pool 装好信号处理、建了临时目录、
    # 起了工作进程，那时收到 TERM 才走「收尾再退」；在那之前（解释器还在载入、run_stage 还在算键）收到的就是默认处理，退 -15、什么也没建。
    # 机器忙时头一步可以慢到分钟，所以按条件等（轮询，带超时），不靠定长；等不到就不发（子进程自证自己停掉），这一格判错并写明没等到什么。
    # 弄坏开关 term-sent-unwaited：不等条件，起了子进程立刻发 TERM（改前碰上忙机器的样子），这一格必须判错
    ready_seconds = 300
    ready_deadline = time.monotonic() + ready_seconds

    def running_state():
        shards = sorted(name for name in os.listdir(temporary) if name.startswith("singlefs-mutation-replay-"))
        marker = open(hang, encoding="utf-8").read().strip() if os.path.exists(hang) else ""
        return shards, marker

    while_running, hanging_marker = running_state()
    if not break_is(os.environ, "term-sent-unwaited"):
        while not (len(while_running) == 1 and hanging_marker.isdigit()) and child.poll() is None and time.monotonic() < ready_deadline:
            time.sleep(0.05)
            while_running, hanging_marker = running_state()
    not_ready = "" if len(while_running) == 1 and hanging_marker.isdigit() else \
        f"没等到临时目录和挂住的假 cargo 都在（{ready_seconds} 秒内；临时目录 {while_running}，标记文件里 {hanging_marker!r}，子进程已退 {child.poll()}）；"
    if child.poll() is None and (not not_ready or break_is(os.environ, "term-sent-unwaited")):
        child.send_signal(signal.SIGTERM)
    elif child.poll() is None:
        child.kill()   # 没到「该发 TERM」的时候就不发，也不留着它：只停这一个子进程
    try:
        child.wait(timeout=180)
    except subprocess.TimeoutExpired:
        child.kill()
        child.wait()
    open(released, "w").close()
    deadline = time.monotonic() + 130
    while os.path.exists(hang) and not os.path.exists(done) and time.monotonic() < deadline:
        time.sleep(0.05)

    def process_is_alive(pid):
        """/proc/<pid> 在、而且不是僵尸（已退、等着被收的不算活着）。"""
        try:
            with open(f"/proc/{pid}/stat", encoding="utf-8") as handle:
                return handle.read().rsplit(")", 1)[1].split()[0] != "Z"
        except OSError:
            return False

    # 挂住的假 cargo 记下的父进程就是那个工作进程：照理已经被停了；还活着（没接住信号的样子）就只对这一个进程号发 TERM，不留孤儿
    worker = int(open(hang, encoding="utf-8").read().strip() or 0) if os.path.exists(hang) else 0
    worker_alive = bool(worker) and process_is_alive(worker)
    if worker_alive:
        with contextlib.suppress(OSError):
            os.kill(worker, signal.SIGTERM)
    left = sorted(name for name in os.listdir(temporary) if name.startswith("singlefs-mutation-replay-"))
    stderr = open(stderr_path, encoding="utf-8", errors="replace").read()
    selftest.check("被 TERM 停（双机分片的驱动停这一份的样子）：$TMPDIR 下的 singlefs-mutation-replay-* 跑的时候在、停完删掉，退 143，工作进程停了，挂住的假 cargo 也退了",
                   not not_ready and len(while_running) == 1 and left == [] and child.returncode == 128 + signal.SIGTERM and not worker_alive and os.path.exists(done),
                   f"{not_ready}跑的时候 {while_running}，停完 {left}，退 {child.returncode}，工作进程 {worker} 还活着 {worker_alive}，假 cargo 退了 {os.path.exists(done)}；"
                   f"stderr 尾：{stderr.strip()[-300:]}")
    term_targets = selftest_run_target_directories(stderr)
    selftest.check("跑完运行目录里不剩它建的编译目录（被 TERM 停）：编译目录建在 $TMPDIR 下这一趟的 singlefs-mutation-replay-* 的 build 里，停完连同它删掉",
                   not not_ready and len(term_targets) == 1 and term_targets[0].startswith(os.path.join(temporary, SHARD_ROOT_PREFIX)) and not os.path.exists(term_targets[0])
                   and left == [] and f"删了这一趟的编译目录 {os.path.dirname(term_targets[0])}" in stderr,
                   f"{not_ready}编译目录 {term_targets}，停完 $TMPDIR 下剩 {left}；{[line for line in stderr.splitlines() if '编译目录' in line][:3]}")
    read_end, write_end = os.pipe()
    os.close(read_end)
    closed_pipe = subprocess.run([sys.executable, MODULE_PATH, "--list-items", repository], stdout=write_end, stderr=subprocess.PIPE, text=True,
                                 errors="replace", env=dict(os.environ))
    os.close(write_end)
    selftest.check("--list-items 接 | head（读的一头先关了）：被 SIGPIPE 安静停掉，stderr 一个字都没有",
                   closed_pipe.returncode == -signal.SIGPIPE and closed_pipe.stderr == "", f"退 {closed_pipe.returncode}：{closed_pipe.stderr.strip()[-300:]!r}")


def selftest_shard_root_window_cell(selftest, work):
    """TERM 落在 run_pool 建 shard_root 的 mkdir 之后、名字交到调用方手里之前（改前 tempfile.mkdtemp 的窗口）：在进程内把 os.mkdir 换成「真建出
    SHARD_ROOT_PREFIX 起头的目录之后抛 StopRequested」（等同 TERM 恰好落在那一瞬），跑 run_pool，退出码是 143、$TMPDIR 下一个目录都不剩。
    弄坏开关 shardrootnamedlate（照改前用 tempfile.mkdtemp，名字丢了）下这一格必须判错。"""
    temporary = os.path.join(work, "shard-root-window-tmp")
    os.makedirs(temporary)
    real_mkdir, saved_tempdir = os.mkdir, tempfile.tempdir
    fired = []

    def mkdir_then_stop(path, *arguments, **keywords):
        real_mkdir(path, *arguments, **keywords)
        if os.path.basename(os.fspath(path)).startswith(SHARD_ROOT_PREFIX) and not fired:
            fired.append(os.fspath(path))
            raise StopRequested(128 + signal.SIGTERM)

    raised = None
    tempfile.tempdir = temporary
    os.mkdir = mkdir_then_stop
    try:
        run_pool(dict(environment=dict(os.environ)), 1, [], [], set(), None, None)
    except StopRequested as stop:
        raised = stop.code
    finally:
        os.mkdir = real_mkdir
        tempfile.tempdir = saved_tempdir
    left = sorted(name for name in os.listdir(temporary) if name.startswith(SHARD_ROOT_PREFIX))
    selftest.check("TERM 落在建 shard_root 的 mkdir 之后、名字交到调用方手里之前：run_pool 照样退 143，$TMPDIR 下的 singlefs-mutation-replay-* 一个都不剩",
                   len(fired) == 1 and raised == 128 + signal.SIGTERM and left == [],
                   f"mkdir 之后抛停的信号 {len(fired)} 次（要 1 次），run_pool 退出码 {raised}（要 {128 + signal.SIGTERM}），停完还剩 {left}")


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


def selftest_gpu_quiesce_cells(selftest, work):
    """带 gpu 特性的格经清场起（文件头「带 gpu 特性的格经清场起」）：清场包装用真的 scripts/crash/gpu-quiesce-run.sh，多机配置是临时目录里只写假清场命令的一份
    （清场与回读各往记录里追加一行；状态目录、本机日志都在临时目录里，不清真场、不上卡）。三格：
    ① 带 gpu 的那一行真跑测试的几次（基线、变异、插桩跑）各经清场包装进一次、回读一次，只编不跑的几次与不带 gpu 的那一行不经；
    ② 变异那一趟回读没过：那一行记清场没过没跑、不算抓到也不算没红，整道判红；不带 gpu 的照抓到；
    ③ 基线那一批回读没过：记基线不绿，写明清场回读没过、命令没起。
    产品 GPU 包（singlefs-background-gpu）的 tests-on-a-real-gpu 特性另有三格，同一份判法：
    ④ 带 tests-on-a-real-gpu 的那一行真跑测试的几次各经清场包装进一次，只编不跑的与不带的不经；
    ⑤ 变异那一趟回读没过：记清场没过没跑，整道判红；
    ⑥ 清场过了、点名的测试在没卡的机器上打 NOT_RUN_NO_GPU_ADAPTER 后返回成功：记本次未跑，不算抓到也不算没红，整道不判红。
    弄坏开关 product-gpu-feature-unquiesced 下这一特性不当上卡的特性认（④⑤⑥ 三格判错）。"""
    real_runner = os.path.join(SCRIPT_DIRECTORY, "..", "crash", "gpu-quiesce-run.sh")

    def quiesce_world(name, readback_passes_up_to, gpu_row=SELFTEST_GPU_FEATURE_ROW, extra_source=""):
        """一棵带一行不带 gpu、一行带 gpu（gpu_row）的小仓，一份假清场配置：回读前 readback_passes_up_to 次过、之后不过（-1 是一直过）。交 (仓, 环境的改动, 清场记录)。"""
        root = os.path.join(work, name)
        build_selftest_repository(root, [SELFTEST_TABLE_ROWS[0], gpu_row], extra_source=extra_source)
        selftest_write(os.path.join(root, CARGO_CONFIGURATION_DIRECTORY, "config.toml"), SELFTEST_CARGO_CONFIGURATION)
        calls = os.path.join(work, name + "-quiesce-calls.log")
        passes = "true" if readback_passes_up_to < 0 else f'[ "$(grep -c check "{calls}")" -le {readback_passes_up_to} ]'
        configuration = os.path.join(work, name + "-multi-host.env")
        selftest_write(configuration, "# 自证用的多机配置：只写假清场命令，清场与回读各往记录里追加一行\n"
                       f"QUIESCE_STOP_COMMAND='echo stop >> \"{calls}\"'\n"
                       f"QUIESCE_STOPPED_CHECK_COMMAND='echo check >> \"{calls}\"; {passes}'\n")
        # 状态目录不是默认那一份、RESOURCE_BROADCAST_DIRECTORY 空着：清场包装不发资源释放广播（它文件头「资源释放广播」）
        changes = dict(GATE_MUTATION_GPU_QUIESCE_RUNNER=real_runner, SINGLEFS_MULTI_HOST_CONFIG=configuration, GPU_CARDS_BY_PRIORITY="",
                       GPU_QUIESCE_STATE_DIRECTORY=os.path.join(work, name + "-quiesce-state"), GPU_QUIESCE_LOG_DIRECTORY=os.path.join(work, name + "-quiesce-logs"),
                       RESOURCE_BROADCAST_DIRECTORY="", RESOURCE_BROADCAST_PUSH="off", GATE_MUTATION_START_OVER="1")
        return root, changes, calls

    def quiesce_calls(calls):
        if not os.path.exists(calls):
            return []
        with open(calls, encoding="utf-8") as handle:
            return handle.read().split()

    root, changes, calls = quiesce_world("gpu-quiesce-passes", -1)
    exit_code, stdout, stderr, invocations = selftest.run(root, **changes)
    gpu_runs = [line for line in invocations if "--features gpu" in line and "--no-run" not in line]
    seen = quiesce_calls(calls)
    selftest.check("带 gpu 特性的行：真跑测试的那几次（基线、变异、插桩跑）各经 gpu-quiesce-run.sh 起、清场回读一次，只编不跑的与不带 gpu 的不经",
                   exit_code == 0 and "计数：抓到 2 条、没红 0 条" in stdout and len(gpu_runs) == 3 and seen.count("check") == 3 and seen.count("stop") == 3,
                   f"退 {exit_code}；假 cargo 带 --features gpu 真跑了 {len(gpu_runs)} 次（要基线、变异、插桩跑各 1 次）；清场记录 {seen}（要 stop、check 各 3 次）；"
                   f"{stdout[-600:]}{stderr[-400:]}")
    root, changes, calls = quiesce_world("gpu-quiesce-mutation-refused", 1)
    exit_code, stdout, stderr, invocations = selftest.run(root, **changes)
    selftest.check("带 gpu 特性的行：变异那一趟清场回读没过，记清场没过没跑（不算抓到、不算没红），整道判红、点名那一行；不带 gpu 的照抓到",
                   exit_code == 1 and "计数：抓到 1 条、没红 0 条" in stdout and "清场没过没跑：1 条" in stdout
                   and "gpu-double-becomes-triple：点名的测试 doubles 带 gpu 特性、经 scripts/crash/gpu-quiesce-run.sh 起，清场回读没过、命令没起" in stdout,
                   f"退 {exit_code}；清场记录 {quiesce_calls(calls)}；{stdout[-900:]}{stderr[-400:]}")
    root, changes, calls = quiesce_world("gpu-quiesce-baseline-refused", 0)
    exit_code, stdout, stderr, invocations = selftest.run(root, **changes)
    selftest.check("带 gpu 特性的行：基线那一批清场回读没过，记基线不绿并写明清场回读没过、命令没起",
                   exit_code == 1 and "基线不绿 1 行" in stdout and "没判成：那一批带 gpu 特性、经 scripts/crash/gpu-quiesce-run.sh 起，清场回读没过、命令没起" in stdout,
                   f"退 {exit_code}；清场记录 {quiesce_calls(calls)}；{stdout[-900:]}{stderr[-400:]}")
    root, changes, calls = quiesce_world("product-gpu-quiesce-passes", -1, SELFTEST_PRODUCT_GPU_FEATURE_ROW)
    exit_code, stdout, stderr, invocations = selftest.run(root, **changes)
    product_runs = [line for line in invocations if "--features tests-on-a-real-gpu" in line and "--no-run" not in line]
    seen = quiesce_calls(calls)
    selftest.check("产品 GPU 包（--features tests-on-a-real-gpu）的行：真跑测试的那几次各经 gpu-quiesce-run.sh 起、清场回读一次，只编不跑的与不带的不经",
                   exit_code == 0 and "计数：抓到 2 条、没红 0 条" in stdout and len(product_runs) == 3 and seen.count("check") == 3 and seen.count("stop") == 3,
                   f"退 {exit_code}；假 cargo 带 --features tests-on-a-real-gpu 真跑了 {len(product_runs)} 次（要基线、变异、插桩跑各 1 次）；清场记录 {seen}（要 stop、check 各 3 次）；"
                   f"{stdout[-600:]}{stderr[-400:]}")
    root, changes, calls = quiesce_world("product-gpu-quiesce-mutation-refused", 1, SELFTEST_PRODUCT_GPU_FEATURE_ROW)
    exit_code, stdout, stderr, invocations = selftest.run(root, **changes)
    selftest.check("产品 GPU 包的行：变异那一趟清场回读没过，记清场没过没跑（不算抓到、不算没红），整道判红、点名那一行；不带 gpu 的照抓到",
                   exit_code == 1 and "计数：抓到 1 条、没红 0 条" in stdout and "清场没过没跑：1 条" in stdout
                   and "product-gpu-double-becomes-triple：点名的测试 doubles 带 gpu 特性、经 scripts/crash/gpu-quiesce-run.sh 起，清场回读没过、命令没起" in stdout,
                   f"退 {exit_code}；清场记录 {quiesce_calls(calls)}；{stdout[-900:]}{stderr[-400:]}")
    root, changes, calls = quiesce_world("product-gpu-quiesce-card-absent", -1, SELFTEST_PRODUCT_GPU_NOT_RUN_ROW, SELFTEST_NOT_RUN_TEST)
    not_run_record = os.path.join(work, "product-gpu-not-run-record.txt")
    exit_code, stdout, stderr, invocations = selftest.run(root, GATE_MUTATION_NOT_RUN_RECORD=not_run_record, **changes)
    recorded_lines = open(not_run_record, encoding="utf-8").read().splitlines() if os.path.exists(not_run_record) else []
    seen = quiesce_calls(calls)
    selftest.check("产品 GPU 包的行：清场过了、点名的测试没卡打 NOT_RUN_NO_GPU_ADAPTER 后返回成功，记本次未跑（不算抓到、不算没红），整道不判红，记录里一行",
                   exit_code == 0 and "本次未跑：1 条" in stdout and "计数：抓到 1 条、没红 0 条" in stdout
                   and f"{TABLE}:3 product-card-path-changed：本次未跑——点名的测试 gpu_parity 打了 NOT_RUN_NO_GPU_ADAPTER（样本：本机没有 GPU 适配器）" in stdout
                   and len(recorded_lines) == 1 and "product-card-path-changed" in recorded_lines[0] and seen.count("stop") >= 2,
                   f"退 {exit_code}；清场记录 {seen}（带 tests-on-a-real-gpu 真跑的几次各清一次，至少 2 次）；记录 {recorded_lines}；{stdout[-900:]}{stderr[-400:]}")


def selftest_gpu_configuration_cells(selftest, work):
    """带 gpu 特性的格把多机配置交给测试、跑完核真用了卡（文件头同名一段）。PATH 上换一份包着假 cargo 的假 cargo（这一格自己写，FAKE_CARGO 不动）：
    每起一次往记录里写一行 JSON（参数、起在哪个目录、收到的 SINGLEFS_MULTI_HOST_CONFIG），点名 gpu_parity、带 --show-output、它是 ok 的那一次
    照 libtest 的样子在末尾交出 $GIVEN_STATES_TALLIES（<GPU 判的>/<CPU 判的>，逗号分开几组）那几行 CRASH_POINT_REPLAY_GIVEN_STATES。五格：
    ① 交多机配置：一行不带 gpu、一行带 gpu（都抓到），带 gpu 那一行真跑测试的几次（基线、变异、插桩跑）在副本里起的 cargo 收到
       SINGLEFS_MULTI_HOST_CONFIG=那份配置的绝对路径，不带 gpu 的与只编不跑的收不到；
    ② 多机配置不在：带 gpu、给定状态有 GPU 判过、变异没红的那一行收不到那个变量，记本次未跑（写明配置不在），不进没红；
    ③ 给定状态有 GPU 判过的（两行里一行 states_judged_by_the_gpu_here 大于 0）：照常判，没红就记没红、整道判红；
    ④ 给定状态全交 CPU 判（两行都是 0）：记本次未跑、不进没红，整道不判红；
    ⑤ named_test_stdout 只读点名的测试那一段（别的用例那一段里 GPU 判过的、同前缀的拒算行都不算）。"""
    gpu_row = "gpu-halve-quartered\tcrates/tiny/src/lib.rs\t{ value / 2 }\t{ value / 4 }\t-p tiny --features gpu --lib -- gpu_parity\tgpu_parity"
    wrapper_directory = os.path.join(work, "gpu-configuration-bin")
    selftest_write(os.path.join(wrapper_directory, "cargo"), r'''#!/usr/bin/env python3
# 包着自证的假 cargo（$INNER_FAKE_CARGO）：记一行 JSON 进 $GPU_CONFIGURATION_LOG，原样起它；点名 gpu_parity、带 --show-output、它是 ok 的那一次
# 照 libtest 的样子在末尾交出 $GIVEN_STATES_TALLIES 那几行 CRASH_POINT_REPLAY_GIVEN_STATES
import json, os, subprocess, sys
arguments = sys.argv[1:]
while arguments[:1] == ["--config"]:   # 插桩编译与插桩跑在 test 前面带 --config build.rustflags=…（同假 cargo 的认法）
    arguments = arguments[2:]
if os.environ.get("GPU_CONFIGURATION_LOG") and arguments[:1] == ["test"]:
    with open(os.environ["GPU_CONFIGURATION_LOG"], "a", encoding="utf-8") as log:
        log.write(json.dumps({"ARGUMENTS": " ".join(arguments[1:]), "WORKING_DIRECTORY": os.getcwd(),
                              "SINGLEFS_MULTI_HOST_CONFIG": os.environ.get("SINGLEFS_MULTI_HOST_CONFIG")}) + "\n")
inner = subprocess.run([sys.executable, os.environ["INNER_FAKE_CARGO"]] + sys.argv[1:], capture_output=True, text=True)
sys.stdout.write(inner.stdout)
sys.stderr.write(inner.stderr)
after = arguments[arguments.index("--") + 1:] if "--" in arguments else []
if "--show-output" in after and "gpu_parity" in after and "test tests::gpu_parity ... ok" in inner.stdout:
    lines = "".join(f"CRASH_POINT_REPLAY_GIVEN_STATES item=selftest-{index} stage=judge mode=judge judged_by=selftest tables_judged_here=1 "
                    f"states_judged_by_the_gpu_here={gpu} states_judged_by_the_cpu_here={cpu} states_reused=0 states_the_gpu_could_not_judge=0\n"
                    for index, (gpu, cpu) in enumerate(tally.split("/") for tally in os.environ["GIVEN_STATES_TALLIES"].split(",")))
    print("\nsuccesses:\n\n---- tests::gpu_parity stdout ----\n" + lines)
sys.exit(inner.returncode)
''', executable=True)

    def configuration_world(name, tallies, configuration, needed):
        """一棵带一行不带 gpu（adds，抓到）、一行带 gpu（gpu_parity 只看 needed：给 { value * 2 } 时改 halve 的那一条没红，给 value / 2 } 时抓到；带收尾的花括号才只命中代码那一行、不命中别的用例的说明行，也不让变异的原文 { value / 2 } 在说明行里再命中一次）的小仓，跑一趟。
        交 (仓, 退出码, stdout 与 stderr 末尾, 包着的假 cargo 记的每一次)。configuration 是交给本模块的那一份多机配置的路径（GATE_MUTATION_MULTI_HOST_CONFIG）。"""
        root = os.path.join(work, name)
        build_selftest_repository(root, [SELFTEST_TABLE_ROWS[0], gpu_row], extra_source=f"// TEST gpu_parity NEEDS {needed}\n")
        selftest_write(os.path.join(root, CARGO_CONFIGURATION_DIRECTORY, "config.toml"), SELFTEST_CARGO_CONFIGURATION)
        log = os.path.join(work, name + "-cargo-environment.jsonl")
        exit_code, stdout, stderr, _ = selftest.run(
            root, GATE_MUTATION_START_OVER="1", GATE_MUTATION_MULTI_HOST_CONFIG=configuration, GPU_CONFIGURATION_LOG=log, GIVEN_STATES_TALLIES=tallies,
            INNER_FAKE_CARGO=os.path.join(selftest.bin_directory, "cargo"), PATH=wrapper_directory + os.pathsep + selftest.base_environment["PATH"])
        return root, exit_code, stdout + stderr[-600:], environment_records(log)

    def test_runs(records, with_gpu):
        return [record for record in records if ("--features gpu" in record["ARGUMENTS"]) == with_gpu and "--no-run" not in record["ARGUMENTS"].split()]

    configuration = os.path.join(work, "gpu-configuration-present", "multi-host.env")
    selftest_write(configuration, "# 自证用的多机配置：假 cargo 不读它，这一格只核它的路径到没到测试的环境里\nENABLE_GPU=1\n")
    root, exit_code, output, records = configuration_world("gpu-configuration-passed", "3/0", configuration, "value / 2 }")
    with_gpu, without_gpu = test_runs(records, True), test_runs(records, False)
    compiles = [record for record in records if "--no-run" in record["ARGUMENTS"].split()]
    received = [record.get("SINGLEFS_MULTI_HOST_CONFIG") for record in with_gpu]
    selftest.check("带 gpu 特性的行：真跑测试的那几次在副本里起的 cargo 环境里有 SINGLEFS_MULTI_HOST_CONFIG 指向本模块所在仓的多机配置（绝对路径），不带 gpu 的与只编不跑的没有",
                   exit_code == 0 and "计数：抓到 2 条、没红 0 条" in output and len(with_gpu) == 3 and received == [configuration] * 3
                   and all(record["WORKING_DIRECTORY"] != root for record in with_gpu) and without_gpu and compiles
                   and all(record.get("SINGLEFS_MULTI_HOST_CONFIG") is None for record in without_gpu + compiles),
                   f"退 {exit_code}；带 gpu 真跑 {len(with_gpu)} 次（要基线、变异、插桩跑各 1 次）收到 {received}（要都是 {configuration}），"
                   f"起在 {[record['WORKING_DIRECTORY'] for record in with_gpu]}（不许是 {root}）；不带 gpu 与只编不跑的收到 "
                   f"{[record.get('SINGLEFS_MULTI_HOST_CONFIG') for record in without_gpu + compiles]}（要都没有）；{output[-700:]}")
    absent = os.path.join(work, "gpu-configuration-absent", "multi-host.env")
    _root, exit_code, output, records = configuration_world("gpu-configuration-missing", "3/0", absent, "{ value * 2 }")
    with_gpu = test_runs(records, True)
    selftest.check("带 gpu 特性的行：多机配置不在，测试收不到 SINGLEFS_MULTI_HOST_CONFIG，没红的那一条记本次未跑（写明配置不在），不进没红",
                   exit_code == 0 and "计数：抓到 1 条、没红 0 条" in output and "本次未跑：1 条" in output
                   and f"gpu-halve-quartered：本次未跑——点名的测试 gpu_parity 带 gpu 特性，本模块所在仓的多机配置 {absent} 不在" in output
                   and with_gpu and all(record.get("SINGLEFS_MULTI_HOST_CONFIG") is None for record in with_gpu),
                   f"退 {exit_code}；带 gpu 真跑 {len(with_gpu)} 次收到 {[record.get('SINGLEFS_MULTI_HOST_CONFIG') for record in with_gpu]}（要都没有）；{output[-900:]}")
    _root, exit_code, output, _records = configuration_world("gpu-given-states-on-the-card", "0/12,7/0", configuration, "{ value * 2 }")
    selftest.check("带 gpu 特性的行：给定状态有 GPU 判过的（两行里一行 states_judged_by_the_gpu_here 大于 0）照常判：没红记没红、整道判红",
                   exit_code == 1 and "计数：抓到 1 条、没红 1 条" in output and "gpu-halve-quartered：gpu_parity 没红" in output and "本次未跑：" not in output,
                   f"退 {exit_code}；{output[-900:]}")
    _root, exit_code, output, _records = configuration_world("gpu-given-states-on-the-cpu", "0/12,0/3", configuration, "{ value * 2 }")
    selftest.check("带 gpu 特性的行：给定状态全交 CPU 判（每一行 states_judged_by_the_gpu_here 都是 0）记本次未跑、不进没红，整道不判红",
                   exit_code == 0 and "计数：抓到 1 条、没红 0 条" in output and "本次未跑：1 条" in output
                   and "gpu-halve-quartered：本次未跑——点名的测试 gpu_parity 带 gpu 特性，它打的 2 行 CRASH_POINT_REPLAY_GIVEN_STATES 里 states_judged_by_the_gpu_here 全是 0" in output,
                   f"退 {exit_code}；{output[-900:]}")
    sample = ("running 2 tests\ntest tests::gpu_parity ... ok\ntest tests::other ... ok\n\nsuccesses:\n\n"
              "---- tests::other stdout ----\nCRASH_POINT_REPLAY_GIVEN_STATES item=a states_judged_by_the_gpu_here=9 states_judged_by_the_cpu_here=0\n\n"
              "---- tests::gpu_parity stdout ----\nCRASH_POINT_REPLAY_GIVEN_STATES item=b states_judged_by_the_gpu_here=0 states_judged_by_the_cpu_here=4\n"
              "CRASH_POINT_REPLAY_GIVEN_STATES_KEY_REFUSED item=b states_judged_by_the_gpu_here=5\n\n"
              "successes:\n    tests::gpu_parity\n    tests::other\n\ntest result: ok. 2 passed; 0 failed\n")
    named = given_states_gpu_counts(named_test_stdout("tests::gpu_parity", sample))
    selftest.check("named_test_stdout 只读点名的测试那一段：别的用例那一段里 GPU 判过的、同前缀的拒算行都不算",
                   named == [0] and named_test_stdout("tests::absent", sample) == "" and named_test_stdout("tests::gpu_parity", "no sections\n") == "no sections\n",
                   f"读到 {named}（要 [0]）")


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
                   plain[0] == 0 and "计数：抓到 4 条、没红 0 条" in plain[1] and "排不上没跑 0 条" in plain[1] and len(judged) == 2 * 5
                   and len(seen) == 2 * 5 + 2 * 4 and not mismatched,
                   f"退 {plain[0]}；包装起了 {len(seen)} 次（要 1 批基线 + 4 条变异各 2 次：先编、再限时跑，4 组覆盖各 2 次）；理由不对的 {mismatched}；{plain[1]}")
    caller_reason = "调用方给的并行理由（自证）"
    passed_through = selftest.run(repository, **dict(daily, RUN_WITH_MEMORY_CAP_PARALLEL_REASON=caller_reason))
    seen = wrapper_lines() if os.path.exists(wrapper_log) else []
    selftest.check("调用方设了的并行理由原样传给每次包装、不覆盖",
                   passed_through[0] == 0 and len(seen) == 2 * 5 and all(reason == caller_reason for reason, _target in seen),
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
    # 三：没设 GATE_MUTATION_TARGET_DIR（文件头「编译目录」）：编译目录建在这一趟自己的运行目录（$TMPDIR 下的 singlefs-mutation-replay-*）的 build 里，
    #   正常退 0、没跑全退 1（GATE_MUTATION_STOP_AFTER_ROWS=1）两种结局跑完都删、$TMPDIR 下一样不剩、跨轮缓存家目录一样没建；被 TERM 停那一种在 selftest_signal_and_pipe_cells 里核。
    #   HOME 与 TMPDIR 都换成临时目录里的一份（弄坏开关退回 $TMPDIR 下那一份时也不往真的 /tmp 里建）；rustc -V 要的 rustup 家目录照原样指回去
    home, temporary = os.path.join(work, "home"), os.path.join(work, "plain-tmp")
    os.makedirs(temporary, exist_ok=True)
    unset = dict(GATE_MUTATION_TARGET_DIR="", HOME=home, TMPDIR=temporary, GATE_MUTATION_WORKERS="1", GATE_MUTATION_START_OVER="1",
                 RUSTUP_HOME=os.environ.get("RUSTUP_HOME") or os.path.expanduser("~/.rustup"),
                 CARGO_HOME=os.environ.get("CARGO_HOME") or os.path.expanduser("~/.cargo"))
    problems = []
    saved_tempdir = tempfile.tempdir   # run_stage 在本进程里跑：shard_root 照 tempfile.gettempdir() 建，这里指进 temporary，跑完还原
    for name, extra, wanted_exit in (("正常", {}, 0), ("没跑全", {"GATE_MUTATION_STOP_AFTER_ROWS": "1"}, 1)):
        tempfile.tempdir = temporary
        try:
            exit_code, _stdout, stderr, _ = selftest.run(repository, **dict(unset, **extra))
        finally:
            tempfile.tempdir = saved_tempdir
        chosen = selftest_run_target_directories(stderr)
        left = sorted(os.listdir(temporary))
        if not (exit_code == wanted_exit and len(chosen) == 1 and chosen[0].startswith(os.path.join(temporary, SHARD_ROOT_PREFIX))
                and chosen[0].endswith(os.sep + os.path.join(RUN_BUILD_DIRECTORY_NAME, TARGET_DIRECTORY_NAME)) and not os.path.exists(chosen[0]) and left == []
                and f"删了这一趟的编译目录 {os.path.dirname(chosen[0])}" in stderr and not os.path.exists(os.path.join(home, ".local", "share", "singlefs", "cross-run-cache"))):
            problems.append(f"{name}那一趟退 {exit_code}（要 {wanted_exit}），编译目录 {chosen}，$TMPDIR 下剩 {left}；"
                            f"{[line for line in stderr.splitlines() if '编译目录' in line][:3]}")
    selftest.check("跑完运行目录里不剩它建的编译目录（没设 GATE_MUTATION_TARGET_DIR：建在 $TMPDIR 下这一趟的 singlefs-mutation-replay-* 的 build 里，正常退 0、没跑全退 1 都删，$TMPDIR 下一样不剩）",
                   not problems, "；".join(problems))
    # 四：这一趟的编译目录里还有进程开着文件（自证起一个 sleep 开着 build 里的一份文件）：remove_run_build_root 不删、报出那个进程；只停自证自己起的那一个进程
    build_root = os.path.join(work, "held-run", RUN_BUILD_DIRECTORY_NAME)
    held = os.path.join(build_root, TARGET_DIRECTORY_NAME, "debug", "held")
    os.makedirs(os.path.dirname(held))
    with open(held, "w", encoding="utf-8") as held_handle:
        holder = subprocess.Popen(["sleep", "60"], stdout=held_handle, stdin=subprocess.DEVNULL)
    try:
        deadline = time.time() + 10
        while time.time() < deadline:
            with contextlib.suppress(OSError):
                if os.readlink(f"/proc/{holder.pid}/fd/1") == held:
                    break
            time.sleep(0.05)
        removed, lines = remove_run_build_root(build_root, selftest.base_environment)
    finally:
        holder.kill()
        holder.wait()
    selftest.check("这一趟的编译目录里还有进程开着文件：不删、报出那个进程",
                   not removed and os.path.isdir(build_root) and any(f"进程 {holder.pid}（sleep 60）" in line for line in lines),
                   f"删成了 {removed}，build 还在 {os.path.isdir(build_root)}；{lines}")


def selftest_run_target_directories(stderr):
    """stderr 里「… 编译目录：<路径>（第 1 片…」那几行的路径（这一趟用的第 1 片编译目录）。"""
    return [line.split("编译目录：", 1)[1].split("（", 1)[0] for line in stderr.splitlines() if line.startswith("  … 编译目录：")]


SELFTEST_OTHER_BATCH_ROW = "double-becomes-triple\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ value * 3 }\t-p tiny --lib --quiet -- doubles\tdoubles"


def selftest_slot_overlap_problem(selftest, earlier_process, earlier_slots, shared):
    """后起的那次领到片号的那一刻，两次调用重叠了没有：先起的那次的子进程还活着、它领到的两片的片号锁都被占，重叠回 None，没重叠回原因。
    弄坏开关 overlapunchecked：不核，一律回 None（改前的样子：两次没重叠也照判对错）。"""
    if break_is(selftest.base_environment, "overlapunchecked"):
        return None
    if earlier_process.poll() is not None:
        return f"后起的那次领到片号时先起的那次已经退了（退 {earlier_process.poll()}）"
    if len(earlier_slots) != 2:
        return f"先起的那次只报了片号 {earlier_slots}"
    free = []
    for slot in earlier_slots:
        held, handle = file_lock_is_held(slot_lock_path(shared, slot))
        if handle is not None:
            handle.close()
        if not held:
            free.append(slot)
    return f"后起的那次领到片号时先起的那次的片号锁 {free} 没被占" if free else None


def selftest_concurrent_slot_pair(selftest, work, tag, earlier_exits_first):
    """在一个共用编译目录上起两次调用（各 2 个工作进程），回 dict：not_ready、earlier_slots、later_slots、held_while_running、free_afterwards、overlap_problem。
    earlier_exits_first 为真是样本：先起的那次不挂假 cargo、等它退了才起后起的那次。"""
    shared = os.path.join(work, f"slot-targets-{tag}", TARGET_DIRECTORY_NAME)
    released = os.path.join(work, f"slot-cargo-released-{tag}")
    invocations = {}
    # 等的是条件（两个工作进程都报了片号；放掉假 cargo 之后两次调用都退出），不是定长：机器忙时头一步可以慢到分钟，照「被 TERM 停」那一格带超时地轮询；
    # 超时不再往下赌，记进 not_ready，这一格判错并写明没等到什么
    ready_seconds, finish_seconds = 300, 180
    not_ready = []
    for name in ("earlier", "later"):
        repository = os.path.join(work, f"slots-{tag}-{name}")
        build_selftest_repository(repository, [SELFTEST_TABLE_ROWS[0], SELFTEST_OTHER_BATCH_ROW])
        temporary = os.path.join(work, f"slots-{tag}-{name}-tmp")
        os.makedirs(temporary)
        environment = dict(selftest.base_environment, GATE_MUTATION_TARGET_DIR=shared, GATE_MUTATION_WORKERS="2", GATE_MUTATION_START_OVER="1", TMPDIR=temporary,
                           FAKE_CARGO_LOG=os.path.join(work, f"slots-{tag}-{name}-cargo.log"), FAKE_CARGO_HANG_FILE=os.path.join(work, f"slots-{tag}-{name}-hanging"),
                           FAKE_CARGO_RELEASE_FILE=released, FAKE_CARGO_DONE_FILE=os.path.join(work, f"slots-{tag}-{name}-done"))
        if earlier_exits_first and name == "earlier":
            del environment["FAKE_CARGO_HANG_FILE"]   # 样本：先起的那次不挂，跑完就退
        invocations[name] = dict(repository=repository, environment=environment, stderr_path=os.path.join(work, f"slots-{tag}-{name}-stderr.txt"))

    def slots_reported(name):
        with open(invocations[name]["stderr_path"], encoding="utf-8", errors="replace") as handle:
            return sorted(int(number) for number in re.findall(r"领到片号 (\d+)（", handle.read()))

    def start(name):
        with open(invocations[name]["stderr_path"], "w", encoding="utf-8") as stderr_handle:
            invocations[name]["process"] = subprocess.Popen([sys.executable, "-c", SELFTEST_STAGE_CHILD, MODULE_PATH, invocations[name]["repository"]],
                                                            env=invocations[name]["environment"], stdout=subprocess.DEVNULL, stderr=stderr_handle)
        deadline = time.monotonic() + ready_seconds
        while len(slots_reported(name)) < 2 and invocations[name]["process"].poll() is None and time.monotonic() < deadline:
            time.sleep(0.05)
        reported = slots_reported(name)
        if len(reported) < 2:
            not_ready.append(f"{name} 那次 {ready_seconds} 秒内只报了片号 {reported}（子进程已退 {invocations[name]['process'].poll()}）")
        return reported

    def wait_for_own_selftest_child(process):
        """等自证自己起的那一个子进程退出；放掉假 cargo 之后 finish_seconds 秒还没退就停掉它（只停这一个，按进程对象，不按模式找），并记进 not_ready。"""
        try:
            process.wait(timeout=finish_seconds)
        except subprocess.TimeoutExpired:
            not_ready.append(f"放掉假 cargo 之后 {finish_seconds} 秒还没退，停掉了它（进程 {process.pid}）")
            process.kill()
            process.wait()

    earlier_slots = start("earlier")
    if earlier_exits_first:
        wait_for_own_selftest_child(invocations["earlier"]["process"])
    later_slots = start("later")
    overlap_problem = selftest_slot_overlap_problem(selftest, invocations["earlier"]["process"], earlier_slots, shared)
    locks_while_running = {slot: file_lock_is_held(slot_lock_path(shared, slot)) for slot in sorted({*earlier_slots, *later_slots})}
    held_while_running = sorted(slot for slot, (held, handle) in locks_while_running.items() if held)
    for _held, handle in locks_while_running.values():
        if handle is not None:
            handle.close()
    open(released, "w").close()
    for name in invocations:
        wait_for_own_selftest_child(invocations[name]["process"])
    free_afterwards = []
    for slot in sorted({*earlier_slots, *later_slots}):
        held, handle = file_lock_is_held(slot_lock_path(shared, slot))
        free_afterwards.append(not held)
        if handle is not None:
            handle.close()
    return dict(not_ready=not_ready, earlier_slots=earlier_slots, later_slots=later_slots, held_while_running=held_while_running,
                free_afterwards=free_afterwards, overlap_problem=overlap_problem)


def selftest_worker_slot_cells(selftest, work):
    """片号跨调用经锁领（文件头「编译目录」）：同一个共用编译目录上并发的两次调用各起 2 个工作进程（假 cargo 挂着不放，两次的工作进程都活着），
    两次领到的片号集合不相交，四把片号锁都被占，放掉之后锁都空了；片号锁被占着的多出来的 -w<片号> 回收时留着。
    两次调用各是一个子进程（stderr 落文件，片号从工作进程报的「领到片号 <n>」那一行读），基线的批分成两批、两个工作进程才都起得来。
    并发那一格先核两次调用确实重叠（selftest_slot_overlap_problem：后起的那次领到片号时先起的那次还活着、它的两把片号锁都被占），没重叠就记这一格本次未跑、不判对错；
    样本：先起的那次不挂假 cargo、等它退了才起后起的那次，那一格要认出没重叠（弄坏开关 overlapunchecked：不核重叠，样本那一格判错）。"""
    pair = selftest_concurrent_slot_pair(selftest, work, "concurrent", False)
    label = "同一个共用编译目录上并发的两次调用各起 2 个工作进程：领到的片号集合不相交、四把片号锁都被占、两次退出之后锁都空了"
    if pair["overlap_problem"] is not None:
        selftest.skip(label, f"两次调用没重叠，这一格的前提没造出来（{pair['overlap_problem']}）；先起的那次领到 {pair['earlier_slots']}、后起的那次领到 {pair['later_slots']}")
    else:
        earlier_slots, later_slots, held_while_running = pair["earlier_slots"], pair["later_slots"], pair["held_while_running"]
        selftest.check(label,
                       not pair["not_ready"] and len(earlier_slots) == 2 and len(later_slots) == 2 and not set(earlier_slots) & set(later_slots)
                       and held_while_running == sorted({*earlier_slots, *later_slots}) and len(held_while_running) == 4 and all(pair["free_afterwards"]),
                       f"{'；'.join(pair['not_ready'])}{'；' if pair['not_ready'] else ''}先起的那次领到 {earlier_slots}、后起的那次领到 {later_slots}；"
                       f"跑的时候被占的锁 {held_while_running}；退出之后锁空了 {pair['free_afterwards']}")
    sample = selftest_concurrent_slot_pair(selftest, work, "earlier-exits", True)
    selftest.check("先起的那次在后起的那次开始之前就退了（样本）：并发领片号那一格认出两次调用没重叠、记本次未跑，不判对错",
                   sample["overlap_problem"] is not None and "先起的那次已经退了" in sample["overlap_problem"],
                   f"认出的没重叠：{sample['overlap_problem']}；先起的那次领到 {sample['earlier_slots']}、后起的那次领到 {sample['later_slots']}；{'；'.join(sample['not_ready'])}")
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
    shard_prefix = wanted + os.sep + CRASH_LIBRARY_SHARD_DIRECTORY_PREFIX
    selftest.check("变异复跑起的 cargo 写自己的崩溃判定库：家目录是真跑那一份底下的 mutation-replay/ 里这一片自己的子目录、不等于真跑的，调用方点名的单份库清掉",
                   exit_code == 0 and len(records) >= 2 and len(homes) == 1 and homes[0].startswith(shard_prefix) and wanted != real_home and libraries == ["None"],
                   f"退 {exit_code}；假 cargo 起了 {len(records)} 次，收到的家目录 {homes}（要 {shard_prefix}<片号>，真跑的是 {real_home}）、单份库 {libraries}（要都没有）；{stderr[-400:]}")
    configured = os.path.join(work, "configured-mutation-crash-library-home")
    exit_code, records, homes, libraries, stderr = received(GATE_MUTATION_CRASH_LIBRARY_HOME=configured)
    selftest.check("设了 GATE_MUTATION_CRASH_LIBRARY_HOME：变异复跑起的 cargo 收到的家目录是它底下这一片自己的子目录、单份库照样清掉",
                   exit_code == 0 and len(records) >= 2 and len(homes) == 1 and homes[0].startswith(configured + os.sep + CRASH_LIBRARY_SHARD_DIRECTORY_PREFIX)
                   and libraries == ["None"],
                   f"退 {exit_code}；假 cargo 起了 {len(records)} 次，收到的家目录 {homes}（要 {configured}/{CRASH_LIBRARY_SHARD_DIRECTORY_PREFIX}<片号>）、单份库 {libraries}；{stderr[-400:]}")



def environment_records(path):
    """假 cargo 的 FAKE_CARGO_ENVIRONMENT_LOG：每起一次 test 一行 JSON（假 cargo 头上那一段）；文件不在交空表。"""
    if not os.path.exists(path):
        return []
    with open(path, encoding="utf-8") as handle:
        return [json.loads(line) for line in handle.read().splitlines() if line.strip()]


def selftest_compile_jobs_cells(selftest, work):
    """编译那一趟的并行度按这一行的内存上限收（文件头「编译并行度」）：compile_jobs_for 在 1G / 4G / 8G / 24G 上的取值；一个工作进程（并行度 = 核数）、
    每条 4G 与 9G 各跑一趟，假 cargo 记下的先编（--no-run，含插桩编译）收到的 CARGO_BUILD_JOBS 是 min(核数, 上限装得下的任务数)，跑测试那几趟仍是核数 ÷ 工作进程数。
    弄坏开关 compilejobsbyworkers（编译那一趟照核数 ÷ 工作进程数）下「编译并行度按内存上限收」那一格必须判错。"""
    environment = dict(selftest.base_environment)
    got = {limit: compile_jobs_for(limit, "10", environment) for limit in ("1G", "4G", "8G", "24G")}
    selftest.check("编译并行度按内存上限收（每个任务按 1.75 GiB 算）：核数 ÷ 工作进程数是 10 时 1G 给 1、4G 给 2、8G 给 4、24G 装得下 13 个所以仍是 10",
                   got == {"1G": "1", "4G": "2", "8G": "4", "24G": "10"}, f"{got}")
    kept = (compile_jobs_for("4G", "1", environment), compile_jobs_for("4G", "10", dict(environment, GATE_MUTATION_CARGO_JOBS="10")))
    selftest.check("核数 ÷ 工作进程数本来就比装得下的小时不抬高；调用方给了 GATE_MUTATION_CARGO_JOBS 的原样用", kept == ("1", "10"), f"{kept}")
    processors = os.cpu_count() or 4
    repository = os.path.join(work, "compile-jobs")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    environment_log = os.path.join(work, "compile-jobs-environment.jsonl")
    for limit, fitting in (("4G", 2), ("9G", 5)):
        with contextlib.suppress(FileNotFoundError):
            os.unlink(environment_log)
        exit_code, _stdout, stderr, _ = selftest.run(repository, GATE_MUTATION_START_OVER="1", GATE_MUTATION_MEMORY_MAX=limit, FAKE_CARGO_ENVIRONMENT_LOG=environment_log)
        records = environment_records(environment_log)
        compile_jobs = {record.get("CARGO_BUILD_JOBS") for record in records if "--no-run" in record["ARGUMENTS"].split()}
        test_jobs = {record.get("CARGO_BUILD_JOBS") for record in records if "--no-run" not in record["ARGUMENTS"].split()}
        wanted = str(min(processors, fitting))
        selftest.check(f"每条 {limit}、一个工作进程：先编（含插桩编译）收到的并行度是 min(核数 {processors}, {fitting}) = {wanted}，跑测试那几趟仍是核数 {processors}，末了一句写明收了几",
                       exit_code == 0 and compile_jobs == {wanted} and test_jobs == {str(processors)} and f"每条 {limit} 时 {wanted}" in stderr,
                       f"退 {exit_code}；先编收到 {sorted(map(str, compile_jobs))}（要 {wanted}），测试收到 {sorted(map(str, test_jobs))}（要 {processors}）；{stderr[-300:]}")


def selftest_compile_memory_cells(selftest, work):
    """撞顶分编译那一趟与测试那一趟（verdict_of_run、baseline_problem）：假内存包装在变异表第 3 行先编那一趟退 250 时，这一行记编译撞顶，单列一栏、出路不写「无界分配」；
    在它测试那一趟退 250 时记测试撞顶、出路照旧；两种「计数：」里都算内存撞顶；基线的句子写明是编译那一趟。弄坏开关 memorycompilemerged 下编译撞顶那一格必须判错。"""
    repository = os.path.join(work, "compile-memory")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    claim = "撞顶 = 这条破坏让被测代码无界分配"
    exit_code, stdout, stderr, _ = selftest.run(repository, GATE_MUTATION_START_OVER="1", FAKE_MEMORY_CAP_OOM_ON="/line-3-build cargo")
    selftest.check("先编那一趟撞了内存上限：记编译撞顶，单列一栏，出路不写「无界分配」，「计数：」里仍算内存撞顶 1 条",
                   exit_code == 1 and "有变异在编译那一趟撞了内存上限（编译撞顶 1 条，每条上限 4G" in stdout
                   and "double-becomes-triple：编译那一趟撞了内存上限 4G（内存撞顶），改坏之后的代码没编成、点名的测试 doubles 一次都没起" in stdout
                   and "内存撞顶 1 条、超时 0 条" in stdout and claim not in stdout and "有变异撞了内存上限（内存撞顶" not in stdout,
                   f"退 {exit_code}；{stdout[-900:]}")
    exit_code, stdout, stderr, _ = selftest.run(repository, GATE_MUTATION_START_OVER="1", FAKE_MEMORY_CAP_OOM_ON="/line-3 cargo")
    selftest.check("测试那一趟撞了内存上限：记测试撞顶，出路照旧写「无界分配」，不写编译那一栏",
                   exit_code == 1 and "有变异撞了内存上限（内存撞顶 1 条，每条上限 4G）：" in stdout and claim in stdout
                   and "double-becomes-triple：撞了内存上限 4G（内存撞顶），点名的测试 doubles 没来得及红" in stdout
                   and "编译那一趟撞了内存上限" not in stdout and "内存撞顶 1 条、超时 0 条" in stdout,
                   f"退 {exit_code}；{stdout[-900:]}")
    row = Row(7, "selftest-compile-memory", "crates/selftest/src/lib.rs", "a", "b", "-p selftest", "tests::named", [])
    on_compile = baseline_problem(row, MEMORY_CAP_HIT_EXIT, "", "4G", "1800", compile_pass=True)
    on_test = baseline_problem(row, MEMORY_CAP_HIT_EXIT, "", "4G", "1800")
    selftest.check("基线撞顶的句子写明是编译那一趟还是测试那一趟",
                   on_compile == "没判成：那一批编译那一趟撞了内存上限 4G（内存撞顶）" and on_test == "没判成：那一批撞了内存上限 4G（内存撞顶）", f"{on_compile!r}；{on_test!r}")


def selftest_write_random(path, size):
    """size 字节的随机内容（压不下去、占盘就是 size 往上），自证里造占盘的编译目录用。"""
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb") as handle:
        handle.write(os.urandom(size))


def selftest_disk_cells(selftest, work):
    """磁盘余量（文件头「磁盘余量」）：每个工作进程最多占两份编译目录（普通、插桩）、各涨到上限，派活前按目标盘余量定工作进程数。
    先直接调 worker_count_for 与 disk_bound_for：外层内存 38G ÷ 每条 4G 本来给 9 个，假的余量 100G、上限 10G、备用 5G，每个工作进程要 20G、开 2 个以上另加先编一份依赖的 20G，
    放得下 3 个（共 80G），头一行写明卡在磁盘余量；已有的编译目录只算还能涨的（片号 1 的普通编译目录已占 8M，上限 10M、余量 100M、备用 1M：放得下 4 个，没有已有目录是 3 个）。
    再起一趟 run_stage：假的余量 70G、备用 10G，开的工作进程数按余量收成 2 个（条数 4、外层内存 9 都放得下更多），头一行写明；余量写法不认的整道拒；
    余量设成空（外层没给）时读编译目录所在的盘，头一行写明来源。弄坏开关 disknotchecked（不看磁盘）下 run_stage 那一格与直接调的两格判错。"""
    environment = dict(selftest.base_environment)
    sixteen = dict(environment, GATE_MUTATION_WORKERS="16", **{OUTER_MEMORY_CAP_VARIABLE: "38G"})
    sized = dict(sixteen, **{DISK_FREE_VARIABLE: "100G", TARGET_DIRECTORY_CAP_VARIABLE: "10G", DISK_RESERVE_VARIABLE: "5G"})
    shared = os.path.join(work, "disk-target")
    count, head = worker_count_for(20, sized, "4G", None, disk=lambda ceiling: disk_bound_for(shared, sized, ceiling, None))
    selftest.check("磁盘余量放不下 9 份：外层内存 38G ÷ 4G 本来给 9 个，余量 100G（上限 10G、备用 5G、每个工作进程两份共 20G、开 2 个以上另加先编一份依赖的两份 20G）按余量收成 3 个，头一行写明卡在磁盘余量与放得下几个",
                   count == 3 and "卡在「磁盘余量」这一项" in head and "磁盘余量 100.0 GiB" in head and "3 个共还要 80.0 GiB，再多 1 个要 100.0 GiB、放不下" in head,
                   f"{count}：{head}")
    tiny = dict(sixteen, **{DISK_FREE_VARIABLE: "100M", TARGET_DIRECTORY_CAP_VARIABLE: "10M", DISK_RESERVE_VARIABLE: "1M"})
    absent = os.path.join(work, "disk-target-absent")
    present = os.path.join(work, "disk-target-present")
    selftest_write_random(os.path.join(present, "debug", "deps", "blob.rlib"), 8 << 20)
    absent_count, _head = disk_bound_for(absent, tiny, 9, None)
    present_count, present_head = disk_bound_for(present, tiny, 9, None)
    selftest.check("已有的编译目录只算还能涨的：片号 1 的普通编译目录已占 8M（上限 10M），余量 100M、备用 1M 放得下 4 个；没有已有目录是 3 个（开 2 个以上另加先编一份依赖的两份）",
                   absent_count == 3 and present_count == 4, f"没有已有目录 {absent_count}；片号 1 已占 8M {present_count}：{present_head}")
    repository = os.path.join(work, "disk-workers")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    target = os.path.join(work, "disk-workers-target")
    exit_code, _stdout, stderr, _ = selftest.run(
        repository, GATE_MUTATION_START_OVER="1", GATE_MUTATION_WORKERS="16", GATE_MUTATION_MEMORY_MAX="4G", GATE_MUTATION_TARGET_DIR=target,
        **{OUTER_MEMORY_CAP_VARIABLE: "38G", DISK_FREE_VARIABLE: "70G", TARGET_DIRECTORY_CAP_VARIABLE: "10G", DISK_RESERVE_VARIABLE: "10G"})
    selftest.check("磁盘余量交给 run_stage：开的工作进程数按余量收（余量 70G、备用 10G、每个工作进程 20G、先编一份依赖的两份 20G 开 2 个，不是条数 4 也不是外层内存的 9），头一行写明卡在磁盘余量",
                   exit_code == 0 and "这一轮开 2 个工作进程" in stderr and "卡在「磁盘余量」这一项" in stderr, f"退 {exit_code}；{stderr[:900]}")
    exit_code, stdout, stderr, _ = selftest.run(repository, GATE_MUTATION_START_OVER="1", GATE_MUTATION_TARGET_DIR=target, **{DISK_FREE_VARIABLE: "lots"})
    selftest.check("磁盘余量的写法不认：整道拒、带出路，一条变异都没跑", exit_code == 1 and f"{DISK_FREE_VARIABLE} 写成了「lots」" in stdout + stderr,
                   f"退 {exit_code}；{(stdout + stderr)[:500]}")
    exit_code, _stdout, stderr, _ = selftest.run(repository, GATE_MUTATION_START_OVER="1", GATE_MUTATION_TARGET_DIR=target, **{DISK_FREE_VARIABLE: ""})
    selftest.check("外层没给磁盘余量时读编译目录所在的盘：头一行写来源，照常跑完", exit_code == 0 and "编译目录所在的盘此刻可用" in stderr, f"退 {exit_code}；{stderr[:700]}")


def selftest_target_cap_cells(selftest, work):
    """编译目录上限（文件头「编译目录上限」）：一份编译目录占盘超过上限，在两格之间清掉它底下的增量产物（<目录>/<档>/incremental、<目录>/<三元组>/<档>/incremental），
    依赖与测试二进制不动；没超线不动；清完仍超线的照实报。先直接调 trim_target_directory：上限 2 MiB，增量 3+1 MiB、依赖 1 MiB，清掉两处增量、依赖还在、清后回读不到 2 MiB；
    没超线不动；清完仍超线（依赖 3 MiB）只清增量、清后仍超。再起两趟 run_stage：假 cargo 每次先编往增量产物里写 64 KiB，上限 1T 时跑完两份编译目录（普通、插桩）的增量产物都留着（对照），
    上限 1K 时跑完两份底下都没有增量产物。弄坏开关 targetcapignored（不清）下三格判错。"""
    environment = dict(selftest.base_environment)
    directory = os.path.join(work, "cap-target")
    selftest_write_random(os.path.join(directory, "debug", "incremental", "session-a", "dep-graph.bin"), 3 << 20)
    selftest_write_random(os.path.join(directory, "x86_64-unknown-linux-gnu", "debug", "incremental", "session-b", "query-cache.bin"), 1 << 20)
    selftest_write_random(os.path.join(directory, "debug", "deps", "libkept.rlib"), 1 << 20)
    before, after, removed = trim_target_directory(directory, 2 << 20, environment)
    selftest.check("编译目录超过上限：清掉两处增量产物，依赖还在，清后回读不到上限",
                   before >= 5 << 20 and len(removed) == 2 and after < 2 << 20 and os.path.isfile(os.path.join(directory, "debug", "deps", "libkept.rlib"))
                   and not os.path.exists(os.path.join(directory, "debug", "incremental")) and not os.path.exists(os.path.join(directory, "x86_64-unknown-linux-gnu", "debug", "incremental")),
                   f"清前 {before}、清后 {after}、清掉 {removed}")
    again = trim_target_directory(directory, 10 << 20, environment)
    selftest.check("没超线不动", again[2] == [] and again[0] == again[1] == after, f"{again}")
    heavy = os.path.join(work, "cap-target-heavy")
    selftest_write_random(os.path.join(heavy, "debug", "incremental", "session-c", "dep-graph.bin"), 1 << 20)
    selftest_write_random(os.path.join(heavy, "debug", "deps", "libbig.rlib"), 3 << 20)
    before, after, removed = trim_target_directory(heavy, 2 << 20, environment)
    selftest.check("清完仍超线：只清增量产物、不动依赖，清后仍大于上限（由调用方报）",
                   len(removed) == 1 and after > 2 << 20 and os.path.isfile(os.path.join(heavy, "debug", "deps", "libbig.rlib")), f"清前 {before}、清后 {after}、清掉 {removed}")
    repository = os.path.join(work, "cap-run")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    target = os.path.join(work, "cap-run-target")
    incremental_paths = (os.path.join(target, "debug", INCREMENTAL_DIRECTORY_NAME), os.path.join(target + COVERAGE_TARGET_SUFFIX, "debug", INCREMENTAL_DIRECTORY_NAME))
    # 对照：上限宽裕（1T）时假 cargo 写下的增量产物两份都留着（证明假 cargo 真写了；工作进程里的 stderr 在自证里收不到，所以不靠它报的话判）
    roomy_exit, _stdout, _stderr, _ = selftest.run(repository, GATE_MUTATION_START_OVER="1", GATE_MUTATION_TARGET_DIR=target, FAKE_CARGO_INCREMENTAL_BYTES="65536",
                                                   **{TARGET_DIRECTORY_CAP_VARIABLE: "1T"})
    roomy_left = [path for path in incremental_paths if os.path.exists(path)]
    shutil.rmtree(target, ignore_errors=True)
    shutil.rmtree(target + COVERAGE_TARGET_SUFFIX, ignore_errors=True)
    # 清的那一趟另起一棵仓（判定库在仓的 git 目录里，沿用上一趟的覆盖记录就不会再编插桩那一份）
    trimmed_repository = os.path.join(work, "cap-run-trimmed")
    build_selftest_repository(trimmed_repository, SELFTEST_TABLE_ROWS)
    # 上限 40K：假 cargo 每次先编写进增量产物的 64K 一来就超线、清掉增量之后剩下的（目录与 deps 里两份小文件，十几 K）在线下，所以照常跑完、不停派
    exit_code, _stdout, stderr, _ = selftest.run(trimmed_repository, GATE_MUTATION_START_OVER="1", GATE_MUTATION_TARGET_DIR=target, FAKE_CARGO_INCREMENTAL_BYTES="65536",
                                                 **{TARGET_DIRECTORY_CAP_VARIABLE: "40K"})
    leftovers = [path for path in incremental_paths if os.path.exists(path)]
    selftest.check("编译目录上限交给 run_stage：上限宽裕（1T）时增量产物两份都留着；上限 40K 时每格判完清掉超线的那份编译目录的增量产物（普通、插桩两份），跑完两份底下都没有增量产物",
                   roomy_exit == 0 and len(roomy_left) == 2 and exit_code == 0 and not leftovers and os.path.isdir(os.path.join(target + COVERAGE_TARGET_SUFFIX, "debug", "deps")),
                   f"宽裕时退 {roomy_exit}、留着 {len(roomy_left)} 份；40K 时退 {exit_code}、还剩 {leftovers}；{stderr[-500:]}")


def selftest_target_prune_cells(selftest, work):
    """编译目录上限的后两步（文件头「编译目录上限」）：清完增量仍超线时 deps 里同名产物只留最新一版、清到线下；还超线就停派。
    先直接调 trim_target_directory：上限 5 MiB，增量 1 MiB、libfoo 新旧两版各 2 MiB、foo_check 新旧两版各 1 MiB、不带单元哈希的 libplain 256 KiB，
    清掉增量与两份旧版、新版与 libplain 还在、清后回读在线下（弄坏开关 depsnotpruned 下清后仍超线，这一格判错）。
    再起一趟 run_stage：上限 1K（清什么都清不到线下），头一格（基线的批）判完就停派、整道判红、现判的变异 0 条、点名停派的那份编译目录（弄坏开关 overcapdispatched 下照派、退 0，这一格判错）。"""
    environment = dict(selftest.base_environment)
    directory = os.path.join(work, "prune-target")
    deps = os.path.join(directory, "debug", "deps")
    old_hash, new_hash = "a" * 16, "b" * 16
    selftest_write_random(os.path.join(directory, "debug", "incremental", "session-d", "dep-graph.bin"), 1 << 20)
    for name, size in ((f"libfoo-{old_hash}.rlib", 2 << 20), (f"libfoo-{new_hash}.rlib", 2 << 20), (f"foo_check-{old_hash}", 1 << 20), (f"foo_check-{new_hash}", 1 << 20),
                       ("libplain.rlib", 256 << 10)):
        selftest_write_random(os.path.join(deps, name), size)
        stamp = 1000 if old_hash in name else 2000
        os.utime(os.path.join(deps, name), (stamp, stamp))
    before, after, removed = trim_target_directory(directory, 5 << 20, environment)
    left = sorted(os.listdir(deps))
    selftest.check("编译目录清完增量仍超上限：deps 里同名产物只留最新一版（libfoo、foo_check 的旧版删掉），不带单元哈希的不动，清后回读在线下",
                   before > 5 << 20 and after <= 5 << 20 and left == sorted([f"libfoo-{new_hash}.rlib", f"foo_check-{new_hash}", "libplain.rlib"])
                   and sorted(os.path.basename(path) for path, _size in removed) == sorted([INCREMENTAL_DIRECTORY_NAME, f"libfoo-{old_hash}.rlib", f"foo_check-{old_hash}"]),
                   f"清前 {before}、清后 {after}、deps 里剩 {left}、清掉 {[path for path, _size in removed]}")
    repository = os.path.join(work, "prune-run")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    target = os.path.join(work, "prune-run-target")
    exit_code, stdout, stderr, _ = selftest.run(repository, GATE_MUTATION_START_OVER="1", GATE_MUTATION_TARGET_DIR=target, **{TARGET_DIRECTORY_CAP_VARIABLE: "1K"})
    selftest.check("编译目录清完仍超上限（1K）：头一格判完就停派、整道判红，现判的变异 0 条，拒绝说明点名停派的那份编译目录",
                   exit_code == 1 and "仍超上限 GATE_MUTATION_TARGET_DIR_CAP，停派" in stdout and "现判的变异判完 0 条" in stdout and f"编译目录 {target}：" in stdout,
                   f"退 {exit_code}；{(stdout + stderr)[-900:]}")


def selftest_disk_peak_cells(selftest, work):
    """磁盘预算按实测峰值算（文件头「磁盘余量」）：每格判完记一次占盘，下一趟每份编译目录的涨幅取近几次的峰值，上限只当清理线。
    先直接调：记下普通 9G、1G、2G、3G 与插桩 4G，取近 3 次是普通 3G（9G 那次出了窗口）、插桩 4G；余量 40G、上限 10G、备用 5G，
    峰值普通 2G、插桩 3G 时每个工作进程 5G、放得下 6 个，插桩没有读数的那一种按上限 10G、只放得下 1 个（弄坏开关 diskbudgetbycap 下一律按上限、头一格也是 1 个，那一格判错）。
    再起一趟 run_stage：跑完判定库目录下记了普通编译目录的占盘。"""
    library = os.path.join(work, "peak-library")
    os.makedirs(library)
    for coverage, size in ((False, 9 << 30), (False, 1 << 30), (True, 4 << 30), (False, 2 << 30), (False, 3 << 30)):
        record_target_occupancy(library, coverage, size)
    peaks = measured_target_peaks(library, 3)
    selftest.check("编译目录占盘的记录：普通、插桩两种各取最近 3 次里最大的（窗口之外的 9G 不算）", peaks == {"普通": 3 << 30, "插桩": 4 << 30}, f"{peaks}")
    environment = dict(selftest.base_environment, **{DISK_FREE_VARIABLE: "40G", TARGET_DIRECTORY_CAP_VARIABLE: "10G", DISK_RESERVE_VARIABLE: "5G"})
    measured_count, measured_head = disk_bound_for(os.path.join(work, "peak-target"), environment, 9, {"普通": 2 << 30, "插桩": 3 << 30})
    mixed_count, mixed_head = disk_bound_for(os.path.join(work, "peak-target"), environment, 9, {"普通": 2 << 30, "插桩": None})
    selftest.check("磁盘预算按实测峰值算：峰值普通 2G、插桩 3G 时余量 40G（备用 5G）放得下 6 个，插桩没有读数时那一种按上限 10G、放得下 1 个，头一行写明涨幅从哪来",
                   measured_count == 6 and "普通那份涨到 2.0 GiB（近几次每格判完量到的占盘峰值）" in measured_head
                   and mixed_count == 1 and "插桩那份涨到 10G（没有实测读数，按上限）" in mixed_head,
                   f"按峰值 {measured_count}：{measured_head}；插桩没读数 {mixed_count}：{mixed_head}")
    repository = os.path.join(work, "peak-run")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    exit_code, _stdout, stderr, _ = selftest.run(repository, GATE_MUTATION_START_OVER="1", GATE_MUTATION_TARGET_DIR=os.path.join(work, "peak-run-target"))
    recorded = measured_target_peaks(library_directory(repository, dict(selftest.base_environment)), 20)
    selftest.check("run_stage 每格判完记一次编译目录占盘：跑完判定库目录下有普通编译目录的读数", exit_code == 0 and recorded["普通"] is not None,
                   f"退 {exit_code}；读数 {recorded}；{stderr[-400:]}")


def selftest_compile_budget_cells(selftest, work):
    """先编那一趟的并行度从外层额度里此刻空着的那一部分算（文件头「编译并行度」）：套在外层内存包装里（RUN_WITH_MEMORY_CAP_OUTER_CAP 读得到）时，
    先编向额度账借，借到的加在这一格自己的上限上，并行度随之升高，编完就还；各格的上限加借走的不超过外层。
    先直接调 OuterBudget 与 compile_plan：外层 36G、2 个工作进程在跑各 4G、想要 16 个并行：第一个借到空着的一半 14G（总上限 18432M、并行度 10），
    第二个借到剩下的 14G（同样 10），第三个什么也借不到（照老算法，4G 给 2），借走的之和加在跑的 8G 刚好 36G；还掉之后账回到 8G；
    GATE_MUTATION_CARGO_JOBS 设了的、没有额度账的照老算法。再起一趟 run_stage：外层 36G、2 个工作进程、每条 4G、核数按 32 算，
    假包装记每次包装的起止与上限：先编收到的并行度全是 10（大于老算法的 2），跑测试那几趟仍是 16，任一时刻在跑的各次包装的上限之和不超过 36G，借过额度（有包装的上限大于 4G）。
    弄坏开关 compilejobsownlimit（先编只按这一格自己的上限收）下三格判错。"""
    environment = dict(selftest.base_environment)
    budget = OuterBudget(36 << 30, 2, 4 << 30)
    budget.claim(4 << 30)
    budget.claim(4 << 30)
    first = compile_plan("4G", "16", environment, budget)
    second = compile_plan("4G", "16", environment, budget)
    third = compile_plan("4G", "16", environment, budget)
    used, peak = budget.used(), budget.peak.value
    for _plans in (first, second):
        budget.give_back(14 << 30)
    after_return = budget.used()
    selftest.check("先编向外层额度借：36G 外层、2 个工作进程在跑各 4G、想要 16 个并行：第一个借 14G（总上限 18432M、并行度 10），第二个借剩下的 14G，第三个借不到（4G 给 2）",
                   first == ("18432M", "10", 14 << 30) and second == ("18432M", "10", 14 << 30) and third == ("4G", "2", 0), f"{first}；{second}；{third}")
    selftest.check("借走的加在跑的各格上限不超过外层（刚好 36G，峰值同），还掉之后账回到在跑的 8G", used == 36 << 30 and peak == 36 << 30 and after_return == 8 << 30,
                   f"借完 {used}、峰值 {peak}、还完 {after_return}")
    fixed = compile_plan("4G", "16", dict(environment, GATE_MUTATION_CARGO_JOBS="12"), OuterBudget(36 << 30, 2, 4 << 30))
    unbudgeted = compile_plan("4G", "16", environment, None)
    selftest.check("GATE_MUTATION_CARGO_JOBS 设了的原样用（不借）、没有额度账（外层读不到）的照老算法", fixed == ("4G", "16", 0) and unbudgeted == ("4G", "2", 0), f"{fixed}；{unbudgeted}")
    repository = os.path.join(work, "compile-budget")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    wrapper = os.path.join(work, "compile-budget-wrapper.sh")
    selftest_write(wrapper, SELFTEST_BUDGET_WRAPPER.replace("@STAMP_DIRECTORY@", START_STAMP_DIRECTORY_NAME), executable=True)
    span_log = os.path.join(work, "compile-budget-spans.log")
    environment_log = os.path.join(work, "compile-budget-environment.jsonl")
    for path in (span_log, environment_log):
        with contextlib.suppress(FileNotFoundError):
            os.unlink(path)
    saved_cpu_count = os.cpu_count
    os.cpu_count = lambda: 32
    try:
        exit_code, _stdout, stderr, _ = selftest.run(
            repository, GATE_MUTATION_WORKERS="2", GATE_MUTATION_MEMORY_MAX="4G", GATE_MUTATION_START_OVER="1", GATE_MUTATION_MEMORY_CAP_RUNNER=wrapper,
            SPAN_LOG=span_log, FAKE_CARGO_ENVIRONMENT_LOG=environment_log, **{OUTER_MEMORY_CAP_VARIABLE: "36G"})
    finally:
        os.cpu_count = saved_cpu_count
    records = environment_records(environment_log)
    compile_jobs = {record.get("CARGO_BUILD_JOBS") for record in records if "--no-run" in record["ARGUMENTS"].split()}
    test_jobs = {record.get("CARGO_BUILD_JOBS") for record in records if "--no-run" not in record["ARGUMENTS"].split()}
    selftest.check("外层 36G、2 个工作进程、每条 4G、核数 32：先编（含插桩编译）收到的并行度全是 10（老算法给 2），跑测试那几趟仍是 16，头一行与末了一句写明借额度",
                   exit_code == 0 and compile_jobs == {"10"} and test_jobs == {"16"} and "先编那一趟的并行度另从外层额度里此刻空着的那一部分借" in stderr and "向外层额度里此刻空着的那一部分借" in stderr,
                   f"退 {exit_code}；先编收到 {sorted(map(str, compile_jobs))}（要 10），测试收到 {sorted(map(str, test_jobs))}（要 16）；{stderr[-600:]}")
    events = []
    with contextlib.suppress(OSError):
        with open(span_log, encoding="utf-8") as handle:
            events = [line.split("\t") for line in handle.read().splitlines()]
    opened, peak_sum, largest = {}, 0, 0
    for kind, stamp, cap in events:
        if kind == "start":
            opened[stamp] = bytes_of_size(cap) or 0
            peak_sum = max(peak_sum, sum(opened.values()))
            largest = max(largest, opened[stamp])
        else:
            opened.pop(stamp, None)
    selftest.check("外层 36G、2 个工作进程、每条 4G：任一时刻在跑的各次包装的上限之和不超过 36G，先编借过额度（有一次包装的上限大于每条的 4G）",
                   bool(events) and peak_sum <= 36 << 30 and largest > 4 << 30 and "额度账：" in stderr,
                   f"包装起止 {len(events)} 行；上限之和的峰值 {peak_sum / 1024 ** 3:g}G；最大一次 {largest / 1024 ** 3:g}G；{stderr[-400:]}")


def selftest_undefined_identifier_cell(selftest, work):
    """static-check 的 UNDEFINED（replacement_undefined_identifiers）：替换文引了被改文件与包里都没有的标识符（照第六十一节那样被删掉的）的那一行报一行；
    包里别的文件定义的、替换文自己绑定的（let、闭包参数、match 臂）、方法与路径里的段、宏、字符串与注释里的都不报。弄坏开关 undefinedidentifiersunchecked 下一行都不报，这一格判错。"""
    root = os.path.join(work, "undefined-identifier")
    rows = [
        "deleted-identifier\tcrates/tiny/src/lib.rs\t{ value / 2 }\t{ halved_before / 2 }\t-p tiny --lib -- halves\thalves",
        "defined-in-package\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ helper_elsewhere(value) * 2 }\t-p tiny --lib -- doubles\tdoubles",
        "bound-in-replacement\tcrates/tiny/src/lib.rs\t{ 0 - value }\t{ let flipped = 0 - value; let pick = |input: i32| input; "
        "match Some(flipped) { Some(inner) => pick(inner).max(i32::MIN), None => unreachable!() } }\t-p tiny --lib -- negates\tnegates",
        "only-in-text\tcrates/tiny/src/lib.rs\t{ left + right }\t{ let _ = \"ghost_in_string\"; /* ghost_in_comment */ left - right }\t-p tiny --lib -- adds\tadds",
    ]
    build_selftest_repository(root, rows)
    selftest_write(os.path.join(root, "crates", "tiny", "src", "other.rs"), "pub fn helper_elsewhere(value: u32) -> u32 { value }\n")
    exit_code, output = run_command_capturing(command_static_check, [root])
    undefined = [line for line in output.splitlines() if line.startswith("UNDEFINED\t")]
    selftest.check("static-check 的 UNDEFINED：替换文引了已删的标识符 halved_before 的那一行报一行、退 1；包里别的文件定义的、替换文自己绑定的、方法、路径、宏、字符串与注释里的都不报",
                   exit_code == 1 and undefined == ["UNDEFINED\t2\tdeleted-identifier\tcrates/tiny/src/lib.rs\thalved_before"]
                   and "IDENTIFIERS_CHECKED\t4\t0" in output, output)


def selftest_closure_refused_cell(selftest, work):
    """点名的测试在这一组特性下不编（#[cfg(feature = "gpu")] 的测试、cargo 参数没开 gpu）：闭包算不成时真错照原话报出、点名它的变异表的行、带出路，
    不被进程池换成 PicklingError。10 个闭包要算，GATE_MUTATION_KEY_WORKERS=2 走并行（子进程里抛的），=1 走就地；run_stage 与 --list-items 各核。
    弄坏开关 closure-error-masked（模块不登记进 sys.modules、子进程原样抛）下并行那一趟必须判错。"""
    tests = "".join(f"    #[test]\n    fn adds{index}() {{}}\n" for index in range(9))
    source = ("pub fn add(left: u32, right: u32) -> u32 { left + right }\n\n#[cfg(test)]\nmod tests {\n" + tests
              + '    #[cfg(feature = "gpu")]\n    #[test]\n    fn gpu_only() {}\n}\n')
    rows = [f"plain-{index}\tcrates/tiny/src/lib.rs\t{{ left + right }}\t{{ left * right }}\t-p tiny --lib -- adds{index}\tadds{index}" for index in range(9)]
    rows.append("gated-without-feature\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left - right }\t-p tiny --features verdict-store --lib -- gpu_only\tgpu_only")
    repository = os.path.join(work, "closure-refused")
    build_selftest_repository(repository, rows, source=source)
    for key_workers, how in (("2", "并行算（子进程里抛）"), ("1", "就地算")):
        try:
            exit_code, stdout, _stderr, _ = selftest.run(repository, GATE_MUTATION_START_OVER="1", GATE_MUTATION_KEY_WORKERS=key_workers)
        except Exception as error:   # 弄坏开关下进程池把真错换成 MaybeEncodingError（PicklingError）抛出来
            exit_code, stdout = None, f"{type(error).__name__}：{str(error)[:300]}"
        selftest.check(f"闭包算不成（{how}）：run_stage 打真错（入口在这一组 cfg 下不编）、点名它的变异表第 11 行、带出路，退 1，没有 PicklingError",
                       exit_code == 1 and "UsageError：入口 crates/tiny/src/lib.rs::tests::gpu_only 在这一组 cfg 下不编" in stdout
                       and "点名它的：crates/mutations.tsv:11 gated-without-feature" in stdout and "给那几行的 cargo 参数补上编它要的特性" in stdout
                       and "PicklingError" not in stdout, f"退 {exit_code}；{stdout[-600:]}")
    saved_handler = signal.getsignal(signal.SIGPIPE)
    saved_workers = os.environ.get("GATE_MUTATION_KEY_WORKERS")
    os.environ["GATE_MUTATION_KEY_WORKERS"] = "2"
    listing = io.StringIO()
    try:
        with contextlib.redirect_stdout(listing):
            list_exit = command_list_items([repository])
    except Exception as error:
        list_exit = None
        listing.write(f"{type(error).__name__}：{str(error)[:300]}")
    finally:
        signal.signal(signal.SIGPIPE, saved_handler)
        if saved_workers is None:
            os.environ.pop("GATE_MUTATION_KEY_WORKERS", None)
        else:
            os.environ["GATE_MUTATION_KEY_WORKERS"] = saved_workers
    text = listing.getvalue()
    selftest.check("--list-items 遇到闭包算不成：变异名照列，末尾一行写键没算出来与真错（入口在这一组 cfg 下不编、点名它的行），没有回溯",
                   list_exit == 0 and "gated-without-feature\t第 11 行" in text and "键对不对得上没算出来" in text
                   and "在这一组 cfg 下不编" in text and "点名它的：crates/mutations.tsv:11 gated-without-feature" in text and "PicklingError" not in text,
                   f"退 {list_exit}；{text[-600:]}")


def selftest_shard_crash_library_cells(selftest, work):
    """变异复跑的崩溃判定库每片各一份（crash_library_home_for_shard、init_worker）：片号 1、2、3 的家目录互不相同、都在变异复跑那一份底下；
    两个工作进程同时在跑（假 cargo 的屏障）时，各片起的 cargo 收到的 SINGLEFS_CRASH_AMPLIFICATION_HOME 互不相同、同一片的每一次都一样。
    弄坏开关 crashlibraryacrossshards（几片共用一份）下两格必须判错。"""
    base = os.path.join(work, "shard-library", MUTATION_CRASH_LIBRARY_SUBDIRECTORY)
    homes = [crash_library_home_for_shard(dict(selftest.base_environment, **{CRASH_LIBRARY_HOME_VARIABLE: base}), sequence) for sequence in (1, 2, 3)]
    unset = crash_library_home_for_shard(dict(selftest.base_environment), 1)
    selftest.check("崩溃判定库每片各一份：片号 1、2、3 的家目录互不相同、都在变异复跑那一份底下；家目录没换过时不分",
                   len(set(homes)) == 3 and all(home and home.startswith(base + os.sep + CRASH_LIBRARY_SHARD_DIRECTORY_PREFIX) for home in homes) and unset is None,
                   f"{homes}；没换过时 {unset}")
    repository = os.path.join(work, "shard-library-repository")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    environment_log = os.path.join(work, "shard-library-environment.jsonl")
    barrier = os.path.join(work, "shard-library-barrier")
    real_home = os.path.join(work, "shard-library-real-home")
    exit_code, _stdout, stderr, _ = selftest.run(
        repository, GATE_MUTATION_START_OVER="1", GATE_MUTATION_WORKERS="2", FAKE_CARGO_ENVIRONMENT_LOG=environment_log, FAKE_CARGO_BARRIER_DIR=barrier,
        FAKE_CARGO_BARRIER_PRISTINE=source_hash(repository), SINGLEFS_CRASH_AMPLIFICATION_HOME=real_home)
    by_shard = collections.defaultdict(set)
    for record in environment_records(environment_log):
        target = record.get("CARGO_TARGET_DIR", "")
        by_shard[target[:-len(COVERAGE_TARGET_SUFFIX)] if target.endswith(COVERAGE_TARGET_SUFFIX) else target].add(str(record.get(CRASH_LIBRARY_HOME_VARIABLE)))
    wanted_base = os.path.join(real_home, MUTATION_CRASH_LIBRARY_SUBDIRECTORY)
    shard_homes = [next(iter(found)) for found in by_shard.values() if len(found) == 1]
    selftest.check("两个工作进程同时在跑：各片起的 cargo 收到的崩溃判定库家目录互不相同、同一片每一次一样、都在变异复跑那一份底下",
                   exit_code == 0 and len(by_shard) == 2 and len(shard_homes) == 2 and len(set(shard_homes)) == 2
                   and all(home.startswith(wanted_base + os.sep + CRASH_LIBRARY_SHARD_DIRECTORY_PREFIX) for home in shard_homes),
                   f"退 {exit_code}；按片分的家目录 {dict(by_shard)}；{stderr[-300:]}")


def selftest_coverage_exact_cell(selftest, work):
    """记覆盖时点名用例前自己补的 --exact，不与行里已有的 --exact 叠成两个（libtest 报 Option 'exact' given more than once、覆盖记不成）：
    一行 libtest 参数已带 --exact 的变异，假 cargo 见两个 --exact 退 101；这一行抓到、覆盖记成 1 份，每一趟带 --exact 的 cargo 命令里 --exact 只出现一次。
    弄坏开关 exactduplicated 下必须判错。"""
    repository = os.path.join(work, "coverage-exact")
    build_selftest_repository(repository, ["add-becomes-mul-exact\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left * right }\t-p tiny --lib -- --exact adds\tadds"])
    exit_code, stdout, stderr, invocations = selftest.run(repository, GATE_MUTATION_START_OVER="1")
    exact_lines = [line.split("\t")[0] for line in invocations if "--exact" in line.split("\t")[0].split()]
    selftest.check("行里已带 --exact 的变异：覆盖记成 1 份、没记成 0 份，带 --exact 的每一条 cargo 命令里 --exact 只出现一次（记覆盖那一趟也是）",
                   exit_code == 0 and "覆盖：这一趟记了 1 份" in stdout and "没记成 0 份" in stdout and exact_lines
                   and all(line.split().count("--exact") == 1 for line in exact_lines) and any("--message-format" not in line and "--no-run" not in line for line in exact_lines),
                   f"退 {exit_code}；带 --exact 的命令 {exact_lines}；{stdout[-300:]}；{stderr[-300:]}")


def selftest_debuginfo_cells(selftest, work):
    """起 cargo 时关调试信息（文件头「调试信息」）：一趟跑下来假 cargo 收到的每一次 test（基线、变异的先编与跑、插桩编译与插桩跑）环境里四档的
    CARGO_PROFILE_<档>_DEBUG 都是 false，调用方设的 CARGO_PROFILE_RELEASE_DEBUG=true 也被顶掉；取值换成 line-tables-only 的那一趟每一行都现判、栏是工具链或依赖变了，
    换回 false 又都复用。弄坏开关 debuginfo-kept（不设）下两格判错，debuginfo-unkeyed（不进键）下后一格判错。"""
    repository = os.path.join(work, "debuginfo")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    environment_log = os.path.join(work, "debuginfo-environment.jsonl")
    exit_code, _stdout, stderr, _ = selftest.run(repository, GATE_MUTATION_START_OVER="1", FAKE_CARGO_ENVIRONMENT_LOG=environment_log, CARGO_PROFILE_RELEASE_DEBUG="true")
    records = environment_records(environment_log)
    wanted = {f"CARGO_PROFILE_{profile}_DEBUG": "false" for profile in CARGO_DEBUG_PROFILES}
    off = [record for record in records if all(record.get(name) == value for name, value in wanted.items())]
    forms = {("--no-run" in record["ARGUMENTS"].split(), any(word.startswith("--message-format") for word in record["ARGUMENTS"].split())) for record in records}
    selftest.check(f"门禁起的每条 cargo 都关了调试信息：{len(records)} 次（基线、变异的先编与跑、插桩编译与插桩跑）每一次四档的 CARGO_PROFILE_<档>_DEBUG 都是 false，调用方设的 true 也顶掉",
                   exit_code == 0 and records and len(off) == len(records) and {(True, False), (False, False), (True, True)} <= forms,
                   f"退 {exit_code}；{len(off)}/{len(records)} 次关了；收到的 {[{name: record.get(name) for name in wanted} for record in records[:3]]}；{stderr[-300:]}")
    reused = selftest.run(repository)
    saved = CARGO_DEBUG_SETTING[0]
    try:
        CARGO_DEBUG_SETTING[0] = "line-tables-only"
        changed = selftest.run(repository)
    finally:
        CARGO_DEBUG_SETTING[0] = saved
    back = selftest.run(repository)
    selftest.check("改了构建档的取值之后行键变了：头一趟（调用方设了 true）之后同一取值 4 行都复用，换成 line-tables-only 那一趟 4 行都现判（工具链或依赖变了），换回 false 又 4 行都复用",
                   reused[0] == 0 and "现判 0 条、复用 4 条" in reused[1] and changed[0] == 0 and len(per_row_how(changed[1])) == 4
                   and set(per_row_how(changed[1]).values()) == {"现判：工具链或依赖变了"} and back[0] == 0 and "现判 0 条、复用 4 条" in back[1],
                   f"同一取值 {per_row_how(reused[1])}；换了取值 {per_row_how(changed[1])}；换回来 {per_row_how(back[1])}")


SELFTEST_EXTERNAL_LOCK = ('version = 4\n\n[[package]]\nname = "fakedep"\nversion = "1.2.3"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\n'
                          'checksum = "0000000000000000000000000000000000000000000000000000000000000000"\n\n'
                          '[[package]]\nname = "tiny"\nversion = "0.1.0"\ndependencies = [\n "fakedep",\n]\n')


def build_seed_repository(directory):
    """先编一份依赖那几格的小仓：照 build_selftest_repository，另放一份 Cargo.lock，tiny 依赖一个带 source 的外部包 fakedep（假 cargo 照它「编」依赖）。"""
    build_selftest_repository(directory, SELFTEST_TABLE_ROWS)
    selftest_write(os.path.join(directory, "Cargo.lock"), SELFTEST_EXTERNAL_LOCK)


def with_break(selftest, switch_name):
    """在调用方给的弄坏开关之外再打开 switch_name（这一格自己造坏输入用）：交 GATE_MUTATION_BREAK 的新值。"""
    given = [item for variable in BREAK_VARIABLES for item in selftest.base_environment.get(variable, "").split(",") if item]
    return ",".join(given + [switch_name])


def selftest_seed_cells(selftest, work):
    """先编一份依赖（文件头「先编一份依赖」）：小仓的 tiny 依赖外部包 fakedep，假 cargo 在缺 fakedep 产物的编译目录里「编」一遍并记一行。
    4 个工作进程（假 cargo 的屏障逼两个工作进程同时在编）：外部依赖只在 <共用目录>-seed 与 -seed-coverage 里各编一次，工作进程的编译目录一次都没编、产物是拷来的；
    判定与 1 个工作进程（不先编）那一趟逐字相同。直接调 dependency_artifact_entries：只交外部依赖的产物，工作区的包、incremental、档那一层的文件不交；拷两遍，第二遍一个都不拷。
    源码副本按内容同步：改了 tiny 的 lib.rs 之后只同步那 1 个文件、摸 crates/tiny，先编那一趟 tiny 重编、整道照常；修改时刻被留住（seedmtimekept）时先编那一趟判红、一条都不跑。
    先编有两组（一行带 --release）：后一组复用前一组编出来的 tiny、假 cargo 报 fresh，整道照常（seedfreshacrossgroups 打开时判红）；修改时刻被留住时两组都是头一次出现就 fresh，照判红。
    直接调 stale_seed_artifacts：前一组非 fresh、后一组同一个单元 fresh 不算坏；同包另一套特性的单元（身份不同）fresh、头一次出现就 fresh 的，照算坏。
    弄坏开关 compile-per-worker、workspaceunitsshared、seedmtimekept、seedfreshunchecked、seedfreshacrossgroups 各让一格判错。"""
    repository = os.path.join(work, "seed")
    build_seed_repository(repository)
    target = os.path.join(work, "seed-target")
    dependency_log = os.path.join(work, "seed-dependencies.log")
    environment_log = os.path.join(work, "seed-environment.jsonl")
    declarations = os.path.join(work, "seed-serial-declarations")
    seeded = selftest.run(repository, GATE_MUTATION_START_OVER="1", GATE_MUTATION_WORKERS="4", GATE_MUTATION_TARGET_DIR=target, FAKE_CARGO_DEPENDENCY_LOG=dependency_log,
                          FAKE_CARGO_ENVIRONMENT_LOG=environment_log, FAKE_CARGO_BARRIER_DIR=os.path.join(work, "seed-barrier"), FAKE_CARGO_BARRIER_PRISTINE=source_hash(repository),
                          SINGLEFS_SERIAL_DECLARATIONS=declarations)
    selftest_seed_declaration_cell(selftest, seeded, declarations)
    compiled = open(dependency_log, encoding="utf-8").read().splitlines() if os.path.exists(dependency_log) else []
    seeds = sorted([target + SEED_TARGET_SUFFIX, target + SEED_TARGET_SUFFIX + COVERAGE_TARGET_SUFFIX])
    worker_targets = {record.get("CARGO_TARGET_DIR") for record in environment_records(environment_log)} - set(seeds)
    plain_worker_targets = {path for path in worker_targets if not path.endswith(COVERAGE_TARGET_SUFFIX)}
    copied_into = [path for path in sorted(worker_targets) if os.path.isfile(os.path.join(path, "debug", "deps", "libfakedep-"
                   + hashlib.sha256(b"fakedep1.2.3").hexdigest()[:16] + ".rlib"))]
    selftest.check("4 个工作进程：外部依赖只在先编那一份里各编一次（普通、插桩各 1 次），起过 cargo 的工作进程编译目录里一次都没编、依赖产物都是拷来的",
                   seeded[0] == 0 and sorted(compiled) == seeds and len(plain_worker_targets) >= 2 and copied_into == sorted(worker_targets),
                   f"退 {seeded[0]}；编过依赖的编译目录 {compiled}（要 {seeds}）；起过 cargo 的工作进程编译目录 {sorted(worker_targets)}，有拷来的 {copied_into}；{seeded[2][-600:]}")
    reference = selftest.run(repository, GATE_MUTATION_START_OVER="1", GATE_MUTATION_WORKERS="1", GATE_MUTATION_TARGET_DIR=os.path.join(work, "seed-reference-target"))
    selftest.check("先编一份依赖不改判定：4 个工作进程先编那一趟与 1 个工作进程（不先编）那一趟判定逐字相同",
                   seeded[0] == 0 and reference[0] == 0 and selftest_verdict_view(seeded[1]) == selftest_verdict_view(reference[1])
                   and "先编一份依赖：没起（这一趟只开 1 个工作进程" in reference[2], f"{seeded[1]}\n——\n{reference[1]}\n{reference[2][-300:]}")
    # 直接调：一份造出来的先编编译目录，外部依赖、工作区的包（.d 里是副本底下的绝对路径与相对路径两种）、incremental、档那一层的文件、三元组那一层各一样
    source = os.path.join(work, "classify-source")
    selftest_write(os.path.join(source, "Cargo.lock"), SELFTEST_EXTERNAL_LOCK)
    seed_target = os.path.join(work, "classify-target")
    external, workspace, relative, build, triple = "0123456789abcdef", "fedcba9876543210", "1111111111111111", "00112233445566ff", "abcdefabcdefabcd"
    layout = {f"debug/deps/libfakedep-{external}.rlib": "rlib", f"debug/deps/fakedep-{external}.d": f"x: /registry/fakedep-1.2.3/src/lib.rs\n",
              f"debug/deps/libtiny-{workspace}.rlib": "rlib", f"debug/deps/tiny-{workspace}.d": f"x: {source}/crates/tiny/src/lib.rs\n",
              f"debug/deps/smoke-{relative}": "exe", f"debug/deps/smoke-{relative}.d": "x: crates/tiny/tests/smoke.rs\n",
              f"debug/.fingerprint/fakedep-{external}/lib-fakedep": "f", f"debug/.fingerprint/tiny-{workspace}/lib-tiny": "f",
              f"debug/build/fakedep-{build}/out/generated.rs": "g", f"debug/build/tiny-{build}/out/generated.rs": "g",
              "debug/incremental/tiny-xyz/s/dep-graph.bin": "i", "debug/libtiny.rlib": "uplifted", "debug/.cargo-lock": "",
              f"x86_64-unknown-linux-gnu/release/.fingerprint/fakedep-{triple}/lib-fakedep": "f"}
    for path, text in layout.items():
        selftest_write(os.path.join(seed_target, path), text)
    entries = sorted(dependency_artifact_entries(seed_target, source, dict(selftest.base_environment)))
    wanted_entries = sorted([f"debug/.fingerprint/fakedep-{external}", f"debug/build/fakedep-{build}", f"debug/deps/fakedep-{external}.d",
                             f"debug/deps/libfakedep-{external}.rlib", f"x86_64-unknown-linux-gnu/release/.fingerprint/fakedep-{triple}"])
    destination = os.path.join(work, "classify-destination")
    first_copy = sync_dependency_artifacts(seed_target, destination, source, dict(selftest.base_environment), 1)
    second_copy = sync_dependency_artifacts(seed_target, destination, source, dict(selftest.base_environment), 1)
    kept_time = os.stat(os.path.join(destination, f"debug/deps/libfakedep-{external}.rlib")).st_mtime_ns == os.stat(os.path.join(seed_target, f"debug/deps/libfakedep-{external}.rlib")).st_mtime_ns
    selftest.check("先编那一份只拷外部依赖的产物：.fingerprint、build、deps 里 fakedep 的 5 项（含三元组那一层），工作区的包、incremental、档那一层的文件不拷；头一遍拷 5 个文件、保留修改时刻，第二遍一个都不拷",
                   entries == wanted_entries and first_copy[0] == 5 and second_copy == (0, 0) and kept_time
                   and not os.path.exists(os.path.join(destination, f"debug/deps/libtiny-{workspace}.rlib")),
                   f"交的 {entries}（要 {wanted_entries}）；头一遍 {first_copy}、第二遍 {second_copy}、修改时刻保留 {kept_time}")
    selftest_seed_copy_threads_cell(selftest, work, seed_target, source)
    # 源码副本按内容同步：头一趟整份进来；改了 tiny 的 lib.rs 之后只同步 1 个文件、摸 crates/tiny，先编那一趟 tiny 重编
    fresh_repository = os.path.join(work, "seed-fresh")
    build_seed_repository(fresh_repository)
    fresh_target = os.path.join(work, "seed-fresh-target")
    first = selftest.run(fresh_repository, GATE_MUTATION_START_OVER="1", GATE_MUTATION_WORKERS="2", GATE_MUTATION_TARGET_DIR=fresh_target)
    landed = re.search(r"串行段声明写进 (\S+)（", first[2])
    selftest.check("先编那一段声明串行段、调用方没设 SINGLEFS_SERIAL_DECLARATIONS：设成这一趟 shard_root（$TMPDIR 下的 singlefs-mutation-replay-*）里的 serial-declarations",
                   first[0] == 0 and landed is not None and os.path.basename(landed.group(1)) == "serial-declarations"
                   and os.path.basename(os.path.dirname(landed.group(1))).startswith(SHARD_ROOT_PREFIX),
                   f"退 {first[0]}；{landed.group(1) if landed else '没报声明写进哪里'}：{first[2][-600:]}")
    replace_in_file(os.path.join(fresh_repository, "crates", "tiny", "src", "lib.rs"), "// 本模块自证的样本 crate", "// 本模块自证的样本 crate（改过一次）")
    second = selftest.run(fresh_repository, GATE_MUTATION_START_OVER="1", GATE_MUTATION_WORKERS="2", GATE_MUTATION_TARGET_DIR=fresh_target)
    selftest.check("先编那一份的源码按内容同步：改了 tiny 的 lib.rs 之后只同步那 1 个文件、摸 crates/tiny，先编那一趟 tiny 重编、整道照常",
                   first[0] == 0 and second[0] == 0 and "同步了 1 个文件（摸了整包的 1 个：crates/tiny）" in second[2]
                   and "先编一份依赖（普通）：" in second[2], f"头一趟退 {first[0]}；改过之后退 {second[0]}：{(second[1] + second[2])[-900:]}")
    replace_in_file(os.path.join(fresh_repository, "crates", "tiny", "src", "lib.rs"), "（改过一次）", "（改过两次）")
    kept = selftest.run(fresh_repository, GATE_MUTATION_START_OVER="1", GATE_MUTATION_WORKERS="2", GATE_MUTATION_TARGET_DIR=fresh_target,
                        GATE_MUTATION_BREAK=with_break(selftest, "seedmtimekept"))
    selftest.check("先编那一份的修改时刻被留住（seedmtimekept）：先编那一趟判红、点名 crates/tiny 是 fresh，一条都不跑",
                   kept[0] == 1 and "crates/tiny 的 tiny 是 fresh" in kept[1] + kept[2] and plain_invocations(kept[3]) == [],
                   f"退 {kept[0]}；{(kept[1] + kept[2])[-900:]}；假 cargo {kept[3]}")
    selftest_seed_group_cells(selftest, work)


def load_core_usage():
    """scripts/hosts/core_usage.py（串行段声明的读法在那里，自证照它读回）。"""
    specification = importlib.util.spec_from_file_location("core_usage_for_crates_mutation_rows_selftest", os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "core_usage.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def selftest_seed_declaration_cell(selftest, seeded, declarations):
    """先编那一段声明串行段（文件头「先编一份依赖」的「串行段声明」）：4 个工作进程那一趟，调用方给的 SINGLEFS_SERIAL_DECLARATIONS 里恰好一段
    「此刻串行：先编一份依赖…」接一行「此刻并行」，照 core_usage.py 的读法读得回、没有写坏的行；直接调 declare_seed_serial（环境里没设变量）：
    变量设进交给它的那份环境（起 cargo 的环境照它拷）、指到给的临时目录里的 serial-declarations。弄坏开关 seed-serial-undeclared 让它判错。"""
    text = open(declarations, encoding="utf-8").read() if os.path.exists(declarations) else ""
    entries, problems = load_core_usage().parse_declarations(text)
    kinds = [kind for _moment, kind, _reason in entries]
    direct_root = os.path.join(selftest.work, "seed-declaration-direct")
    os.makedirs(direct_root, exist_ok=True)
    direct_environment = {key: value for key, value in selftest.base_environment.items() if key != "SINGLEFS_SERIAL_DECLARATIONS"}
    direct_lines = []
    direct = declare_seed_serial(direct_environment, direct_root, 2, "4", "8G", direct_lines)
    wanted = os.path.join(direct_root, "serial-declarations")
    selftest.check("先编那一段声明串行段：调用方给的声明文件里一段「此刻串行：先编一份依赖…组共用一份编译目录…」接「此刻并行」，core_usage.py 读得回；"
                   "没设变量时设进起 cargo 的那份环境",
                   seeded[0] == 0 and kinds == ["serial", "parallel"] and not problems and entries[0][0] <= entries[1][0]
                   and "先编一份依赖" in entries[0][2] and "组共用一份编译目录" in entries[0][2]
                   and f"串行段声明写进 {declarations}" in seeded[2]
                   and direct == wanted and direct_environment.get("SINGLEFS_SERIAL_DECLARATIONS") == wanted and os.path.isfile(wanted),
                   f"退 {seeded[0]}；声明文件 {text!r}；读回 {entries}，写坏的 {problems}；直接调交 {direct}、环境里 {direct_environment.get('SINGLEFS_SERIAL_DECLARATIONS')}：{direct_lines}")


def selftest_seed_copy_threads_cell(selftest, work, seed_target, source):
    """工作进程拷依赖产物分到几条线程上同时拷（文件头「先编一份依赖」的「拷」）：直接调 sync_dependency_artifacts（threads=4），
    shutil.copy2 换成先到一道两方屏障（5 秒）再拷的：头两个文件要在两条线程上同时拷到屏障才过得去，fakedep 的 rlib 拷到了（拷几个由「只拷外部依赖的产物」那一格管）。弄坏开关 seed-copy-serial 让它判错（屏障等满 5 秒被打破）。"""
    barrier = threading.Barrier(2, timeout=5)
    seen_threads, broken, arrivals = set(), [], []
    arrival_lock = threading.Lock()
    original_copy = shutil.copy2

    def copy_after_barrier(path, target, **keywords):
        with arrival_lock:
            seen_threads.add(threading.get_ident())
            arrivals.append(path)
            gated = len(arrivals) <= 2   # 只有头两个文件过屏障
        if gated:
            try:
                barrier.wait()
            except threading.BrokenBarrierError:
                broken.append(path)
        return original_copy(path, target, **keywords)
    destination = os.path.join(work, "classify-destination-threads")
    shutil.copy2 = copy_after_barrier
    try:
        copied = sync_dependency_artifacts(seed_target, destination, source, dict(selftest.base_environment), threads=4)
    finally:
        shutil.copy2 = original_copy
    selftest.check("工作进程拷依赖产物分到几条线程上同时拷：4 条线程，头两个文件同时到两方屏障（没有一次等满 5 秒），fakedep 的产物拷到了",
                   copied[0] >= 2 and len(seen_threads) >= 2 and not broken and os.path.isfile(os.path.join(destination, "debug/deps/libfakedep-0123456789abcdef.rlib")),
                   f"拷了 {copied}；拷的线程 {len(seen_threads)} 条；等满 5 秒屏障被打破的 {len(broken)} 次")


def seed_artifact_message(package, root, unit_hash, fresh, features=()):
    """cargo --message-format=json 的一条 compiler-artifact（字段照真 cargo：package_id、manifest_path、target、profile、features、filenames、executable、fresh）。"""
    return json.dumps({"reason": "compiler-artifact", "package_id": f"path+file://{root}/crates/{package}#{package}@0.1.0",
                       "manifest_path": os.path.join(root, "crates", package, "Cargo.toml"),
                       "target": {"kind": ["lib"], "crate_types": ["lib"], "name": package, "src_path": os.path.join(root, "crates", package, "src", "lib.rs")},
                       "profile": {"opt_level": "0", "debuginfo": 0, "test": True}, "features": list(features),
                       "filenames": [f"/target/debug/deps/{package}-{unit_hash}"], "executable": f"/target/debug/deps/{package}-{unit_hash}", "fresh": fresh})


def selftest_seed_group_cells(selftest, work):
    """先编有两组时，后一组复用前一组编出来的产物（cargo 报 fresh）不算坏，头一次出现就是 fresh 的才算坏。"""
    rows = SELFTEST_TABLE_ROWS + ["double-release-becomes-triple\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ value * 3 }\t-p tiny --release --lib -- doubles\tdoubles"]
    repository = os.path.join(work, "seed-groups")
    build_selftest_repository(repository, rows)
    selftest_write(os.path.join(repository, "Cargo.lock"), SELFTEST_EXTERNAL_LOCK)
    target = os.path.join(work, "seed-groups-target")
    common = dict(GATE_MUTATION_START_OVER="1", GATE_MUTATION_WORKERS="2", GATE_MUTATION_TARGET_DIR=target)
    twice = selftest.run(repository, **common)
    selftest.check("先编那一份有两组（-p tiny 与 -p tiny --release）：后一组复用前一组编出来的 tiny、假 cargo 报 fresh，不算坏、整道照常",
                   twice[0] == 0 and "先编一份依赖（普通）：" in twice[2] and "编了 2 组" in twice[2] and "是 fresh" not in twice[1] + twice[2],
                   f"退 {twice[0]}：{(twice[1] + twice[2])[-900:]}")
    replace_in_file(os.path.join(repository, "crates", "tiny", "src", "lib.rs"), "// 本模块自证的样本 crate", "// 本模块自证的样本 crate（改过一次）")
    edited = selftest.run(repository, **common)
    selftest.check("先编那一份有两组、改了 tiny 的 lib.rs 之后：摸 crates/tiny、前一组重编、后一组复用，整道照常",
                   edited[0] == 0 and "摸了整包的 1 个：crates/tiny" in edited[2] and "编了 2 组" in edited[2] and "是 fresh" not in edited[1] + edited[2],
                   f"退 {edited[0]}：{(edited[1] + edited[2])[-900:]}")
    replace_in_file(os.path.join(repository, "crates", "tiny", "src", "lib.rs"), "（改过一次）", "（改过两次）")
    per_group = selftest.run(repository, **common, GATE_MUTATION_BREAK=with_break(selftest, "seedfreshacrossgroups"))
    selftest.check("先编那一份有两组、每组各判各的（seedfreshacrossgroups）：后一组里 tiny 是 fresh 记成坏的，先编那一趟判红、一条都不跑",
                   per_group[0] == 1 and "crates/tiny 的 tiny 是 fresh" in per_group[1] + per_group[2] and "--release" in per_group[1] + per_group[2]
                   and plain_invocations(per_group[3]) == [], f"退 {per_group[0]}：{(per_group[1] + per_group[2])[-900:]}；假 cargo {per_group[3]}")
    replace_in_file(os.path.join(repository, "crates", "tiny", "src", "lib.rs"), "（改过两次）", "（改过三次）")
    kept = selftest.run(repository, **common, GATE_MUTATION_BREAK=with_break(selftest, "seedmtimekept"))
    selftest.check("先编那一份有两组、修改时刻被留住（seedmtimekept）：两组里 tiny 头一次出现就是 fresh，照判红、一条都不跑",
                   kept[0] == 1 and "crates/tiny 的 tiny 是 fresh" in kept[1] + kept[2] and plain_invocations(kept[3]) == [],
                   f"退 {kept[0]}：{(kept[1] + kept[2])[-900:]}；假 cargo {kept[3]}")
    # 直接调：两组的 cargo JSON 输出，同一份记录（compiled）串起来
    root = os.path.join(work, "identity-source")
    touched = {"crates/tiny"}
    first_group = seed_artifact_message("tiny", root, "aaaaaaaaaaaaaaaa", False)
    reused = seed_artifact_message("tiny", root, "aaaaaaaaaaaaaaaa", True)
    other_features = seed_artifact_message("tiny", root, "bbbbbbbbbbbbbbbb", True, features=("gpu",))
    compiled = set()
    carried = [stale_seed_artifacts(first_group, root, touched, compiled), stale_seed_artifacts(reused, root, touched, compiled)]
    selftest.check("先编两组的 cargo 输出：前一组 tiny 非 fresh、后一组同一个单元 fresh，不算坏",
                   carried == [[], []] and len(compiled) == 1, f"{carried}；记下的单元 {len(compiled)} 个")
    separate = [stale_seed_artifacts(first_group, root, touched), stale_seed_artifacts(reused, root, touched)]
    selftest.check("先编两组的 cargo 输出各判各的（不给记录，seedfreshacrossgroups 的样子）：后一组 tiny fresh 算坏", separate == [[], ["crates/tiny 的 tiny"]], f"{separate}")
    compiled = set()
    stale_seed_artifacts(first_group, root, touched, compiled)
    selftest.check("同包另一套特性编出来的单元（身份不同）在后一组头一次出现就是 fresh：照算坏，只按包名放行不行",
                   stale_seed_artifacts(other_features, root, touched, compiled) == ["crates/tiny 的 tiny"], f"{compiled}")
    selftest.check("头一组里头一次出现就是 fresh 的：照算坏", stale_seed_artifacts(reused, root, touched, set()) == ["crates/tiny 的 tiny"])
    selftest.check("没摸过的包 fresh 不算坏（照旧）", stale_seed_artifacts(seed_artifact_message("fakedep", root, "cccccccccccccccc", True), root, touched, set()) == [])


def run_selftest():
    work = tempfile.mkdtemp(prefix="crates-mutation-rows-selftest-")
    saved_environment = dict(os.environ)
    try:
        selftest = Selftest(work)
        # 判法冻成一份临时拷贝：别的会话这时候改了判法闭包里的哪一份（lib_function_units.py 这一类），自证里几十趟的判法单元照样一致
        frozen, frozen_made = judging_copy(work, "judging-frozen")
        selftest.check("判法单元的闭包算得出、冻成一份临时拷贝", frozen_made, "judging-units 算不出；单跑 python3 scripts/mutation/crates-mutation-rows.py judging-units . 看拒算")
        if frozen_made:
            JUDGING_ROOT[0] = frozen
        # import、plan 与 library-directory 读本进程的环境：与 run_stage 那一份对齐
        os.environ.clear()
        os.environ.update(selftest.base_environment)
        run_selftest_cells(selftest, work)
        selftest_unit_key_cells(selftest, work)
        selftest_reuse_cells(selftest, work)
        selftest_parallel_key_cell(selftest, work)
        selftest_wgsl_unit_key_cell(selftest, work)
        selftest_judging_unit_cells(selftest, work)
        selftest_build_profile_cell(selftest, work)
        selftest_queue_line_cells(selftest, work)
        selftest_daily_cells(selftest, work)
        selftest_gpu_memory_cell(selftest, work)
        selftest_row_limit_cell(selftest, work)
        selftest_worker_count_cells(selftest, work)
        selftest_row_memory_budget_cells(selftest, work)
        selftest_disk_cells(selftest, work)
        selftest_target_cap_cells(selftest, work)
        selftest_target_prune_cells(selftest, work)
        selftest_disk_peak_cells(selftest, work)
        selftest_compile_budget_cells(selftest, work)
        selftest_libtest_result_cell(selftest, work)
        selftest_host_lookup_cells(selftest, work)
        selftest_cargo_configuration_cell(selftest, work)
        selftest_gpu_quiesce_cells(selftest, work)
        selftest_gpu_configuration_cells(selftest, work)
        selftest_target_directory_cells(selftest, work)
        selftest_worker_slot_cells(selftest, work)
        selftest_signal_and_pipe_cells(selftest, work)
        selftest_shard_root_window_cell(selftest, work)
        selftest_crash_library_home_cells(selftest, work)
        selftest_shard_crash_library_cells(selftest, work)
        selftest_compile_jobs_cells(selftest, work)
        selftest_compile_memory_cells(selftest, work)
        selftest_closure_refused_cell(selftest, work)
        selftest_coverage_exact_cell(selftest, work)
        selftest_anchor_and_impact_cells(selftest, work)
        selftest_undefined_identifier_cell(selftest, work)
        selftest_debuginfo_cells(selftest, work)
        selftest_seed_cells(selftest, work)
    finally:
        JUDGING_ROOT[0] = STAGE_REPOSITORY
        os.environ.clear()
        os.environ.update(saved_environment)
        shutil.rmtree(work, ignore_errors=True)
    if selftest.failures:
        print(f"  ✗ crates-mutation-rows.py 自证有 {len(selftest.failures)} 格判错（共 {selftest.cells} 格）：")
        for failure in selftest.failures:
            print(f"      {failure}")
        print("     → 怎么办：设了 GATE_MUTATION_BREAK 的，这就是要的结果（那个开关把对应的判法弄坏了）；没设的，照上面那几格去看 run_stage、row_decision、")
        print("       KeyContext.components、baseline_reuse_problem、judge_coverage、dispatch_order、command_import、command_plan、escaping_literals、read_table 与 row_problems；")
        print("       单元切分与闭包看 scripts/mutation/lib_mutation_units.py 与 scripts/gate/lib_function_units.py；并行理由那两格看 run_under_wrapper。")
        for item in selftest.not_run:
            print(f"      本次未跑：{item}")
        return 1
    if selftest.not_run:
        # 有格的前提没造出来（skip）：不判绿，退 77（本次未跑）
        print(f"  · crates-mutation-rows.py 自证：判了的 {selftest.cells} 格都对，本次未跑 {len(selftest.not_run)} 格（前提没造出来，不算通过）：")
        for item in selftest.not_run:
            print(f"      {item}")
        print("     → 怎么办：机器闲一点（/proc/pressure/io 的 some 低下来）再跑一遍 --selftest；还是本次未跑就照那一格写的原因看造前提的那几步")
        return 77
    print(f"  ✓ crates-mutation-rows.py 自证 {selftest.cells} 格都对（复用、改一行只重跑那一行、跑到一半续跑、基线不绿、"
          "久的先派、选行只判一部分退 3、导入核键与判法单元指纹、多久没被引用才删、记录里没有主机名、拷贝范围两样、"
          "替换文空着的删行变异照常判、派活之前那几样 checker-tier-crates-mutation-replay 与 static-check 同一份判法、"
          "点名的测试打了 NOT_RUN 记本次未跑且变异那一趟补了 --show-output、只有本次未跑的退 77、should panic 的用例红了判抓到、"
          "单元键：被改坏函数的别一行变只现判原文在它里面的行（S34）、判法单元变了整张现判（S30）、限时变了与改名一行都不现判（N33、N34）、"
          "同文件加没人调的函数一行都不现判（N45）、WGSL 按 fn 取键（改另一个 fn 键不变、改被改坏的 fn 键变、原文只在注释里的拒算）、改点名测试调到的函数与被改坏函数用的常量现判点名它的行、基线按执行过的单元复用而不按整树快照、"
          "只改判法注释判法单元指纹不变而改 libtest 解析一处就变、拒算的照跑、"
          "会话里平常跑时每次包装带自己那个工作进程的并行理由且调用方设了的不覆盖、"
          "工作进程数不超过外层内存上限 ÷ 每条变异的上限而外层上限读不到时头一行写明、"
          "有一行逐行上限很大（24G）也不压低工作进程数、派活按每一格自己的上限从外层额度里扣（在跑的各格上限之和不超过外层、大格不被饿死）、"
          "拷贝范围含 .cargo/（点名的测试要编 gpu 特性的那一行在副本里读得到 [env]）、"
          "带 gpu 特性真跑测试的那几次经 gpu-quiesce-run.sh 清场再起（只编不跑的与不带 gpu 的不经；清场回读没过记清场没过没跑或基线不绿、整道判红）、"
          "带 gpu 特性真跑测试的那几次在副本里收到本模块所在仓的多机配置（不带 gpu 的与只编不跑的收不到），多机配置不在或给定状态全交 CPU 判的没红记本次未跑而 GPU 判过的照常判、"
          "被 TERM 停时删掉 $TMPDIR 下的 singlefs-mutation-replay-*（TERM 落在建目录那一瞬也不漏）、"
          "片号大于这一趟工作进程数的 -w<片号> 开跑时回收而有进程在用的不删、两个变量都没设时编译目录不落 /tmp、"
          "同一个共用编译目录上并发的两次调用领到的片号集合不相交（片号锁）而片号锁被占着的 -w<片号> 回收时留着、"
          "--list-items 接 | head 时安静退、变异复跑起的 cargo 写自己的崩溃判定库（真跑家目录底下的 mutation-replay/，或 GATE_MUTATION_CRASH_LIBRARY_HOME；每片再分一个 shard-<片号> 子目录，两个工作进程同时在跑时互不相同）而点名的单份库清掉、"
          "编译那一趟的并行度按这一行的内存上限收而跑测试那一趟不收、撞顶分编译那一趟与测试那一趟两栏各写各的出路、"
          "点名的测试在这一组特性下不编时 run_stage 与 --list-items 打真错并点名变异表的行（并行算与就地算一样）、行里已带 --exact 的变异记覆盖时 --exact 只出现一次、"
          "工具链变了全现判、不按时限过期、llvm 工具不对判红、plan 只读且逐栏报数、"
          "锚点失效按行判（那一行记锚点失效、不入库、整道判红，别的行照跑照入库）而 row-plan、plan、--list-items 把锚点失效的行单列（数据文件那一行也不算键）、"
          "牵连面在样本 crate 上报对牵连最大的单元与行数、"
          "磁盘余量放不下时工作进程数按余量收（已有的编译目录只算还能涨的、余量写法不认整道拒、没给余量读编译目录所在的盘）、"
          "编译目录超过上限时判完一格清掉增量产物、套在外层包装里时先编向外层额度借（并行度升高、在跑各格的上限加借走的不超过外层）、"
          "起的每条 cargo 都关了调试信息而构建档的取值进行键、几个工作进程时外部依赖只在先编那一份里编一次且只拷外部依赖的产物、先编那一份的源码按内容同步而摸过的包没重编就判红、"
          "先编那一段往 SINGLEFS_SERIAL_DECLARATIONS 声明串行段而拷依赖产物分到几条线程上同时拷）")
    return 0


def run_selftest_cells(selftest, work):
    repository = os.path.join(work, "repository")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    pristine = source_hash(repository)
    # 一：头一趟全跑；第二趟输入不变，假 cargo 一次都不起、判定逐字相同
    first = selftest.run(repository)
    selftest.check("头一趟判绿", first[0] == 0, first[1] + first[2])
    selftest.check("头一趟起 1 批基线 + 4 条变异，另记 4 份覆盖（各一次插桩编译、一次插桩跑）",
                   len(plain_invocations(first[3])) == 5 and len(coverage_invocations(first[3])) == 8, "\n".join(first[3]))
    second = selftest.run(repository)
    selftest.check("第二趟输入不变：cargo 一次都不起", second[0] == 0 and second[3] == [], "\n".join(second[3]))
    selftest.check("第二趟判定与头一趟逐字相同（只差复用的报数）", selftest_verdict_view(second[1]) == selftest_verdict_view(first[1])
                   and "现判 0 条、复用 4 条、拒算 0 条、本次未跑 0 条" in second[1], second[1])
    selftest.check("第二趟 stderr 头一行报复用 4 条", "复用抓到 4 条" in (second[2].splitlines() or [""])[0], second[2])
    # 起 cargo 之前判 RocksDB 预编库的取值（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」）：同一份小仓另放一份依赖 rocksdb 的 checker 档包清单、
    #   进程环境里四个取值是空的，PATH 上的假 cargo 不带 entry 认的标记 ⇒ 退 1、假 cargo 一次都不起、报出 entry 的红句
    rocksdb_repository = os.path.join(work, "rocksdb-repository")
    shutil.copytree(repository, rocksdb_repository, symlinks=True)
    selftest_write(os.path.join(rocksdb_repository, "crates", "singlefs-checker-tier", "Cargo.toml"),
                   '[package]\nname = "singlefs-checker-tier"\nversion = "0.0.0"\n[dependencies]\nrocksdb = { version = "=0.25.0", optional = true }\n')
    refused_by_rocksdb = selftest.run(rocksdb_repository, ROCKSDB_LIB_DIR="", ROCKSDB_STATIC="", SNAPPY_LIB_DIR="", SNAPPY_STATIC="")
    selftest.check("RocksDB 预编库的取值缺了：起 cargo 之前判红、退 1、假 cargo 一次都不起",
                   refused_by_rocksdb[0] == 1 and refused_by_rocksdb[3] == [] and "rocksdb-prebuilt entry：" in refused_by_rocksdb[1] + refused_by_rocksdb[2],
                   refused_by_rocksdb[1] + refused_by_rocksdb[2])
    shutil.rmtree(rocksdb_repository)
    # 二：进程数不同、一律不复用，判定仍逐字相同
    parallel = selftest.run(repository, GATE_MUTATION_WORKERS="4", GATE_MUTATION_START_OVER="1")
    selftest.check("GATE_MUTATION_WORKERS=4 与 =1 的判定逐字相同", selftest_verdict_view(parallel[1]) == selftest_verdict_view(first[1]), parallel[1])
    selftest.check("GATE_MUTATION_START_OVER=1 只写不读（报「一律现判」那一栏）", len(plain_invocations(parallel[3])) == 5
                   and "不读库：GATE_MUTATION_START_OVER=1" in parallel[2] and "一律现判 4" in parallel[1], parallel[1] + parallel[2])
    full = selftest.run(repository, SINGLEFS_GATE_FULL="1")
    selftest.check("SINGLEFS_GATE_FULL=1 只写不读", len(plain_invocations(full[3])) == 5, "\n".join(full[3]))
    unlimited = selftest.run(repository, SINGLEFS_REUSE_HOURS="0")
    selftest.check("SINGLEFS_REUSE_HOURS=0 照样复用（记录不按时限过期）", unlimited[0] == 0 and unlimited[3] == [], "\n".join(unlimited[3]))
    # 三：只改一行的替换文：只那一行现判（栏是原文或替换文变了），它点名测试的基线复用（基线键与点名测试都没变），只起那一条变异
    table_path = os.path.join(repository, TABLE)
    replace_in_file(table_path, "{ value * 3 }", "{ value * 4 }")
    one_row = selftest.run(repository)
    changed = mutation_invocations(plain_invocations(one_row[3]), pristine)
    selftest.check("只改一行的替换文：只那一行现判，栏是原文或替换文变了，它的基线复用", one_row[0] == 0 and len(plain_invocations(one_row[3])) == 1 and len(changed) == 1
                   and "doubles" in changed[0] and per_row_how(one_row[1]).get("double-becomes-triple") == "现判：原文或替换文变了",
                   one_row[1] + "\n".join(one_row[3]))
    # 四：改一行注释（锚点照旧命中）：单元指纹不变，全部复用
    replace_in_file(os.path.join(repository, "crates", "tiny", "src", "lib.rs"), "pub fn add(", "// 改了一行注释\npub fn add(")
    pristine = source_hash(repository)
    comment_changed = selftest.run(repository)
    selftest.check("改一行注释：单元指纹不变，全部复用、cargo 一次都不起", comment_changed[0] == 0 and comment_changed[3] == []
                   and "现判 0 条、复用 4 条" in comment_changed[1], comment_changed[1] + "\n".join(comment_changed[3]))
    # 五：跑到一半停掉（只供测试的开关），下一趟只跑剩下的；基线已判过 ok 的不再跑。先清掉这一棵仓的库，从零起
    shutil.rmtree(library_of(repository), ignore_errors=True)
    halfway = selftest.run(repository, GATE_MUTATION_START_OVER="1", GATE_MUTATION_STOP_AFTER_ROWS="2")
    selftest.check("跑完 2 条就退：判红、说明是只供测试的开关", halfway[0] == 1 and "GATE_MUTATION_STOP_AFTER_ROWS=2" in halfway[1]
                   and len(mutation_invocations(halfway[3], pristine)) == 2, halfway[1] + "\n".join(halfway[3]))
    resumed = selftest.run(repository)
    selftest.check("下一趟只跑剩下的 2 条、基线不再跑，判完的 2 条复用；给这一趟现判的 2 条测试补记覆盖",
                   resumed[0] == 0 and len(plain_invocations(resumed[3])) == 2 and len(mutation_invocations(resumed[3], pristine)) == 2
                   and len(coverage_invocations(resumed[3])) == 4, "\n".join(resumed[3]))
    # 六：上一次跑得久（或超时）的先派，按用时从长到短；stderr 头一行报前移几条
    timings_directory = os.path.join(library_of(repository), "timings")
    table_rows, _malformed, _memory, _timeout = read_table(table_path)
    long_rows = {"negates": ("400", "0"), "halves": ("1800", "1")}
    for row in table_rows:
        if row.expected in long_rows:
            seconds, timed_out = long_rows[row.expected]
            write_atomically(os.path.join(timings_directory, row_content_hash(row)),
                             mutation_units.record_text(RECORD_HEADER, dict(seconds_running=seconds, timed_out=timed_out)))
    ordered = selftest.run(repository, GATE_MUTATION_START_OVER="1")
    order = [line.split("\t")[0].split()[-1] for line in mutation_invocations(ordered[3], pristine)]
    ordered_head_line = (ordered[2].splitlines() or [""])[0]
    selftest.check("上一次超时的、跑得久的先派（按用时从长到短）", order[:2] == ["halves", "negates"] and "前移 2 条" in ordered_head_line,
                   f"派活次序 {order}；{ordered_head_line}")
    # 七：选行只判一部分：都抓到退 3、末行写明不是整道结论
    selection_file = os.path.join(work, "selection.txt")
    selftest_write(selection_file, "".join(f"{row.line_number}\n" for row in table_rows[:2]))
    selected = selftest.run(repository, GATE_MUTATION_ROW_SELECTION=selection_file, GATE_MUTATION_START_OVER="1")
    selftest.check("选 2 行：都抓到退 3，末行写明只判了选中的 2 行", selected[0] == EXIT_SELECTION_ONLY
                   and "只判了选中的 2 行，不是整道结论" in selected[1].splitlines()[-1] and len(mutation_invocations(selected[3], pristine)) == 2,
                   selected[1])
    reused_selected = selftest.run(repository, GATE_MUTATION_ROW_SELECTION=selection_file)
    selftest.check("选 2 行、不设 START_OVER：照判定库复用，与整张跑时那两行的判定相同", reused_selected[0] == EXIT_SELECTION_ONLY
                   and reused_selected[3] == [] and "复用 2 条" in reused_selected[1], reused_selected[1])
    selftest_write(os.path.join(work, "bad-selection.txt"), "99999\n")
    bad_selection = selftest.run(repository, GATE_MUTATION_ROW_SELECTION=os.path.join(work, "bad-selection.txt"))
    selftest.check("选行文件里的行号不是一条变异：判红、一格都没跑", bad_selection[0] == 1 and bad_selection[3] == [], bad_selection[1])
    # 八：导入另一台的库：同一份内容的另一棵仓先全收，之后一格都不跑、判定与本机现跑逐字相同；键或判法单元指纹对不上的拒
    # 另一台那一趟的库照双机分片的样子是这一趟新建的（第二台不留长期库）：在 GATE_MUTATION_RECORDS_HOME 指的一个空目录里只写不读地判一遍
    peer_home = os.path.join(work, "peer-home")
    reference = selftest.run(repository, GATE_MUTATION_RECORDS_HOME=peer_home, GATE_MUTATION_START_OVER="1")
    other = os.path.join(work, "other")
    build_selftest_repository(other, [line for line in open(table_path, encoding="utf-8").read().splitlines()[1:]])
    shutil.copyfile(os.path.join(repository, "crates", "tiny", "src", "lib.rs"), os.path.join(other, "crates", "tiny", "src", "lib.rs"))
    source_library = library_of(repository)
    imported = run_command_capturing(command_import, [other, os.path.join(peer_home, LIBRARY_DIRECTORY_NAME)])
    after_import = selftest.run(other)
    selftest.check("导入同一判法单元、同一份源码的库：全收", imported[0] == 0 and "拒了 0 条" in imported[1], imported[1])
    selftest.check("导入之后另一棵仓一格都不跑、判定与本机逐字相同", after_import[3] == [] and selftest_verdict_view(after_import[1]) == selftest_verdict_view(reference[1]),
                   after_import[1] + after_import[2])
    forged_directory = os.path.join(work, "forged")
    genuine_name = sorted(os.listdir(os.path.join(source_library, "records")))[0]
    genuine = mutation_units.parse_record(open(os.path.join(source_library, "records", genuine_name), encoding="utf-8").read())
    forged_key = hashlib.sha256(b"no such row").hexdigest()
    selftest_write(os.path.join(forged_directory, "records", forged_key), mutation_units.record_text(RECORD_HEADER, dict(genuine, row_key=forged_key)))
    foreign_judging = dict(genuine, judging=hashlib.sha256(b"another judging").hexdigest())
    selftest_write(os.path.join(forged_directory, "records", genuine_name), mutation_units.record_text(RECORD_HEADER, foreign_judging))
    forged = run_command_capturing(command_import, [other, forged_directory])
    selftest.check("导入一条键对不上的记录：拒、列出来", forged[0] == 1 and f"records/{forged_key}" in forged[1] and "键不是本机" in forged[1], forged[1])
    selftest.check("导入一条判法单元指纹对不上的记录：拒", "判法单元指纹" in forged[1] and "拒了 2 条" in forged[1], forged[1])
    # 九：多久没被任何一趟引用就删：40 天前摸过、这一趟没读到的记录删了；40 天前摸过、这一趟读到的（复用了）摸成此刻、留着；不按个数删
    records_directory = os.path.join(source_library, "records")
    stale = os.path.join(records_directory, hashlib.sha256(b"stale record").hexdigest())
    selftest_write(stale, RECORD_HEADER + "kind=mutation\n")
    long_ago = time.time() - 40 * 86400
    os.utime(stale, (long_ago, long_ago))
    current_keys = {line.split("\t")[1] for line in run_command_capturing(command_row_plan, [repository])[1].splitlines() if "\t" in line}
    kept_name = sorted(name for name in os.listdir(records_directory) if name in current_keys)[0]
    os.utime(os.path.join(records_directory, kept_name), (long_ago, long_ago))
    collected = selftest.run(repository)
    selftest.check("40 天里没被任何一趟引用的记录收尾删掉，这一趟复用的那一条摸成此刻、留着（不按个数、不按版本删）",
                   collected[0] == 0 and not os.path.exists(stale) and os.path.exists(os.path.join(records_directory, kept_name))
                   and os.stat(os.path.join(records_directory, kept_name)).st_mtime > long_ago + 86400, collected[2][-600:])
    # 十：记录里不写主机名（判定库记录与解开的覆盖记录、组成清单）
    host_name = os.uname().nodename
    record_texts = []
    for directory, _names, files in os.walk(source_library):
        for name in files:
            path = os.path.join(directory, name)
            if name.endswith(".json.gz"):
                with gzip.open(path, "rt", encoding="utf-8", errors="replace") as handle:
                    record_texts.append(handle.read())
            else:
                record_texts.append(open(path, encoding="utf-8", errors="replace").read())
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
    selftest.check("点名的测试打了 NOT_RUN：记本次未跑、逐条列出、计数里抓到 4 条没红 0 条、整道不判红、成功句不说全抓到、成功句报本次未跑 1 条、记录里一行",
                   not_run[0] == 0 and "本次未跑：1 条" in not_run[1]
                   and f"{TABLE}:6 card-path-changed：本次未跑——点名的测试 gpu_parity 打了 NOT_RUN_NO_GPU_ADAPTER（样本：本机没有 GPU 适配器）" in not_run[1]
                   and "计数：抓到 4 条、没红 0 条" in not_run[1] and len(success_lines) == 1 and "抓到 4 条、本次未跑 1 条" in success_lines[0]
                   and "本次未跑 1 条（现判的按栏" in success_lines[0]
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


def selftest_unit_key_cells(selftest, work):
    """单元键几格（第二轮复查 controls 路第 5 节 crates 变异那一行：S34、S30、N33、N34、N45；另加被改坏的单元、点名测试的闭包、V5 那一类常量、
    覆盖复用基线、拒算、plan、工具链、llvm 工具几格）：每格施加一处改动，看哪几行现判、栏对不对；弄坏开关各让点名的那一格判错。"""
    repository = os.path.join(work, "units")
    build_selftest_repository(repository, SELFTEST_UNIT_ROWS, source=SELFTEST_UNIT_SOURCE)
    source = os.path.join(repository, "crates", "tiny", "src", "lib.rs")
    table_path = os.path.join(repository, TABLE)
    names = [row.split("\t")[0] for row in SELFTEST_UNIT_ROWS]

    def fresh_and_reused(run):
        how = per_row_how(run[1])
        return sorted(name for name in how if how[name].startswith("现判")), sorted(name for name in how if how[name] == "复用"), how

    first = selftest.run(repository)
    coverage_directory = os.path.join(library_of(repository), "coverage")
    coverage_files = sorted(os.listdir(coverage_directory)) if os.path.isdir(coverage_directory) else []
    selftest.check("头一趟每条点名测试记一份覆盖、同一条测试的几行共用一份（5 行 4 条测试：两行点名 checks_root）",
                   first[0] == 0 and len(coverage_files) == 4 and len(coverage_invocations(first[3])) == 8 and "行新加 5" in first[1]
                   and "覆盖：这一趟记了 4 份" in first[1], f"覆盖记录 {coverage_files}；{first[1]}")
    second = selftest.run(repository)
    selftest.check("输入不变的第二趟：5 行都复用、cargo 一次都不起", second[0] == 0 and second[3] == [] and fresh_and_reused(second)[1] == sorted(names), second[1])
    # plan：只读、逐行打、末行报总数
    library_before = sorted(os.path.relpath(os.path.join(directory, name), library_of(repository))
                            for directory, _names, files in os.walk(library_of(repository)) for name in files)
    plan = run_command_capturing(command_plan, [repository])
    library_after = sorted(os.path.relpath(os.path.join(directory, name), library_of(repository))
                           for directory, _names, files in os.walk(library_of(repository)) for name in files)
    selftest.check("plan：只读（库里一个文件都没多）、逐行打复用还是现判、末行报总数",
                   plan[0] == 0 and library_after == library_before and plan[1].count("\t复用\t键相同\t") == 5
                   and plan[1].splitlines()[-1].startswith("合计 5 行：复用 5 行、现判 0 行"), plan[1])
    listed = run_command_capturing(command_list_items, [repository])
    selftest.check("--list-items：逐行列可点名的变异，末行报键对不上 0 条、对得上 5 条、拒算 0 条",
                   listed[0] == 0 and listed[1].count("\t第 ") == 5 and "键对不上 0 条（这一趟要现判）、键对得上 5 条（复用）、拒算 0 条" in listed[1], listed[1])
    # S34：被改坏的函数 check_root 的别一行变了：原文落在它里面的两行（root-457、root-double）现判，栏是被改坏的单元变了；别的三行复用
    replace_in_file(source, "base.max(OFFSET)", "base.max(OFFSET + 0)")
    s34 = selftest.run(repository)
    fresh, reused, how = fresh_and_reused(s34)
    selftest.check("被改坏函数的别一行变了（S34）：原文在它里面的两行现判、栏是被改坏的单元变了，别的三行复用",
                   s34[0] == 0 and fresh == ["root-457", "root-double"] and how.get("root-457") == "现判：被改坏的单元变了"
                   and reused == ["helper-eight", "sum-sub", "taken-eleven"]
                   and "root-457：被改坏的单元变了——被改坏的单元 crates/tiny/src/lib.rs::check_root 变了" in s34[2], f"{how}；{s34[1]}{s34[2][-600:]}")
    # N45：同一个文件里加一个没人调的函数：一行都不现判
    replace_in_file(source, "pub fn pad(", "pub fn unused_helper() -> u32 { 70 }\npub fn pad(")
    n45 = selftest.run(repository)
    selftest.check("同一个文件里加一个没人调的函数（N45）：一行都不现判、cargo 一次都不起", n45[0] == 0 and n45[3] == [] and fresh_and_reused(n45)[0] == [],
                   n45[1] + "\n".join(n45[3]))
    # N33：每条的限时 1800 → 3600：一行都不现判（限时不进行键）；弄坏开关 row-key-with-limits 下每一行都现判、这一格判错
    n33 = selftest.run(repository, GATE_MUTATION_TIMEOUT="3600")
    selftest.check("每条限时 1800 → 3600（N33）：一行都不现判", n33[0] == 0 and n33[3] == [] and fresh_and_reused(n33)[0] == [], n33[1] + "\n".join(n33[3]))
    # N34：给一条变异改名：一行都不现判（变异名不进行键）
    replace_in_file(table_path, "sum-sub\t", "sum-becomes-sub\t")
    n34 = selftest.run(repository)
    selftest.check("给一条变异改名（N34）：一行都不现判、改了名的那一行复用", n34[0] == 0 and n34[3] == [] and per_row_how(n34[1]).get("sum-becomes-sub") == "复用",
                   n34[1])
    # 改了点名测试调到、不是被改坏的那个函数（sums 调的 pad）：点名它的那一行现判，栏是点名测试的闭包变了；弄坏开关 test-closure-ignored 下复用、这一格判错
    replace_in_file(source, "{ value + 0 }", "{ value + 0 + 0 }")
    padded = selftest.run(repository)
    fresh, reused, how = fresh_and_reused(padded)
    selftest.check("改了点名测试调到、不是被改坏的函数：点名它的那一行现判（点名测试的闭包变了），别的复用",
                   padded[0] == 0 and fresh == ["sum-becomes-sub"] and how.get("sum-becomes-sub") == "现判：点名测试的闭包变了" and len(reused) == 4, f"{how}；{padded[1]}")
    # 改了被改坏、点名测试够不着的函数（helps 只调 add，helper 是被改坏的那个）：只现判那一行，栏是被改坏的单元变了；弄坏开关 mutated-unit-ignored 下复用、这一格判错
    replace_in_file(source, "pub fn helper(value: u32) -> u32 { value + 7 }", "pub fn helper(value: u32) -> u32 { let _seven = 7; { value + 7 } }")
    helper = selftest.run(repository)
    fresh, reused, how = fresh_and_reused(helper)
    selftest.check("改了被改坏、点名测试够不着的函数：只现判那一行（被改坏的单元变了）",
                   helper[0] == 0 and fresh == ["helper-eight"] and how.get("helper-eight") == "现判：被改坏的单元变了" and len(reused) == 4, f"{how}；{helper[1]}")
    # V5 那一类：改了被改坏函数用的常量 OFFSET（check_root 自己的指纹不变）：原文在 check_root 里的两行现判，栏是点名测试的闭包变了
    replace_in_file(source, "OFFSET: u32 = 95", "OFFSET: u32 = 91")
    constant = selftest.run(repository)
    fresh, reused, how = fresh_and_reused(constant)
    selftest.check("改了被改坏函数用的常量（V5 那一类）：原文在那个函数里的两行现判，别的复用，现判的原因点名那个常量",
                   constant[0] == 0 and fresh == ["root-457", "root-double"] and how.get("root-457") == "现判：点名测试的闭包变了" and len(reused) == 3
                   and "改了 crates/tiny/src/lib.rs::OFFSET（const）" in constant[2], f"{how}；{constant[1]}{constant[2][-600:]}")
    # 覆盖复用基线：改了 branches 静态够得着、没执行到的 untaken：那一行现判（闭包变了），它的基线按覆盖复用、不跑基线；
    # 弄坏开关 coveragenotrecorded 下没有覆盖记录、基线只按闭包比、要跑一批基线，这一格判错
    replace_in_file(source, "{ value + 10 }", "{ value + 12 }")
    untaken = selftest.run(repository)
    fresh, _reused, how = fresh_and_reused(untaken)
    selftest.check("改了点名测试够得着、没执行到的函数：那一行现判，基线按覆盖复用（只起那一条变异、不跑基线）",
                   untaken[0] == 0 and fresh == ["taken-eleven"] and len(plain_invocations(untaken[3])) == 1 and "复用基线 ok 1 行" in untaken[2],
                   f"{how}；{untaken[2][:400]}" + "\n".join(untaken[3]))
    # 基线不按整树快照：加一个没人调的函数、同时改一行的替换文：只那一行现判、它的基线复用；弄坏开关 baseline-by-tree-snapshot 下基线现跑、这一格判错
    replace_in_file(source, "pub fn pad(", "pub fn unused_too() -> u32 { 71 }\npub fn pad(")
    replace_in_file(table_path, "{ left - right }", "{ left * right }")
    snapshot = selftest.run(repository)
    fresh, _reused, how = fresh_and_reused(snapshot)
    selftest.check("加一个没人调的函数、同时改一行的替换文：只那一行现判、它的基线复用（不按整树快照）",
                   snapshot[0] == 0 and fresh == ["sum-becomes-sub"] and how.get("sum-becomes-sub") == "现判：原文或替换文变了" and len(plain_invocations(snapshot[3])) == 1,
                   f"{how}；" + "\n".join(snapshot[3]))
    # 工具链变了：每一行都现判；弄坏开关 rowkeywithouttoolchain 下复用、这一格判错
    toolchain = selftest.run(repository, FAKE_CARGO_VERSION="cargo 0.0.1 (crates-mutation-rows selftest：换了工具链)")
    selftest.check("工具链变了：每一行都现判", toolchain[0] == 0 and set(per_row_how(toolchain[1]).values()) == {"现判：工具链或依赖变了"}, toolchain[1])
    # 拒算：点名的测试是宏展开出来的、认不出单元：那一行每趟照跑、不复用，成功句报拒算 1 条，stderr 写为什么
    refused_root = os.path.join(work, "refused")
    build_selftest_repository(refused_root, SELFTEST_TABLE_ROWS[:1] + ["unknown-test\tcrates/tiny/src/lib.rs\tu32) -> u32 { value * 2 }\tu32) -> u32 { value * 5 }\t-p tiny --lib -- doubled\tdoubled"],
                              "// TEST doubled NEEDS { value * 2 }\n")
    refused_first = selftest.run(refused_root)
    refused_second = selftest.run(refused_root)
    selftest.check("点名的测试认不出单元：那一行拒算、每趟照跑，成功句报拒算 1 条，stderr 写为什么",
                   refused_first[0] == 0 and refused_second[0] == 0 and per_row_how(refused_second[1]).get("unknown-test") == "拒算"
                   and len(mutation_invocations(plain_invocations(refused_second[3]), source_hash(refused_root))) == 1
                   and "现判 0 条、复用 1 条、拒算 1 条" in refused_second[1] and "拒算（照跑、不复用）" in refused_second[2],
                   refused_second[1] + refused_second[2][-400:])
    mismatched = selftest.run(repository, FAKE_LLVM_VERSION="0.0.0", GATE_MUTATION_START_OVER="1")
    selftest.check("llvm 工具的 LLVM 版本与 rustc 对不上：判红、一条都不跑", mismatched[0] == 1 and mismatched[3] == []
                   and "记覆盖要的 llvm 工具不对" in mismatched[1], mismatched[1] + mismatched[2])
    empty_tools = os.path.join(work, "no-llvm-tools")
    os.makedirs(empty_tools)
    missing = selftest.run(repository, GATE_MUTATION_LLVM_TOOLS=empty_tools, GATE_MUTATION_START_OVER="1")
    selftest.check("llvm 工具不在：判红、一条都不跑、出路写 rustup component add llvm-tools", missing[0] == 1 and missing[3] == []
                   and "rustup component add llvm-tools" in missing[1], missing[1])
    # S30：判结局的函数变了（判法的临时拷贝里改 verdict_of_run 的一处）：每一行都现判，栏是判法单元变了、原因里点名 verdict_of_run（放在最后：这一趟写的索引是拷贝那一份判法的）
    copy, edited = judging_copy(work, "judging-s30")
    replace_in_file(os.path.join(copy, "scripts", "mutation", "crates-mutation-rows.py"), "    tail = output_tail(output)\n", "    tail = output_tail(output[:])\n")
    s30 = selftest.run(repository, judging_repository=copy)
    fresh, _reused, how = fresh_and_reused(s30)
    selftest.check("判结局的函数变了（S30）：每一行都现判，栏是判法单元变了、原因里点名 verdict_of_run",
                   s30[0] == 0 and edited and len(fresh) == 5 and set(how.values()) == {"现判：判法单元变了"}
                   and "改了 scripts/mutation/crates-mutation-rows.py::verdict_of_run" in s30[2], f"{how}；{s30[1][-800:]}{s30[2][-800:]}")


def selftest_reuse_cells(selftest, work):
    """复用三问（reuse-three-questions）与复用哨兵（reuse-sentinel）两格，写法在文件头「复用三问与复用哨兵」。"""
    repository = os.path.join(work, "three-questions")
    build_selftest_repository(repository, SELFTEST_UNIT_ROWS, source=SELFTEST_UNIT_SOURCE)
    source = os.path.join(repository, "crates", "tiny", "src", "lib.rs")
    sentinel_environment = {reuse_checks.SENTINEL_COUNT_VARIABLE: "1", reuse_checks.SENTINEL_SEED_VARIABLE: "5"}
    forced_how = "现判：" + COLUMN_TITLES["sentinel"]

    def states(run):
        found = {}
        for name, how in per_row_how(run[1]).items():
            found[name] = (reuse_checks.FORCED if how == forced_how else reuse_checks.JUDGED if how.startswith("现判")
                           else reuse_checks.REUSED if how == "复用" else how)
        return found

    runs = [selftest.run(repository, **sentinel_environment)]
    runs.append(selftest.run(repository, **sentinel_environment))
    replace_in_file(source, "pub fn pad(", "pub fn unused_in_three_questions() -> u32 { 72 }\npub fn pad(")
    runs.append(selftest.run(repository, **sentinel_environment))
    replace_in_file(source, "base.max(OFFSET)", "base.max(OFFSET + 0)")
    runs.append(selftest.run(repository, **sentinel_environment))
    passed, problems = reuse_checks.report_three_questions(SCRIPT_KEY, *[states(run) for run in runs], ["root-457", "root-double"])
    selftest.check(reuse_checks.CELL_NAME, passed and all(run[0] == 0 for run in runs),
                   f"四趟退 {[run[0] for run in runs]}；{'；'.join(problems)}；{runs[-1][2][-600:]}")
    green = selftest.run(repository, **sentinel_environment)
    skipped = selftest.run(repository, GATE_MUTATION_BREAK="reuse-lookup-skipped", **sentinel_environment)
    forced = [name for name, how in per_row_how(green[1]).items() if how == forced_how]
    selftest.check(reuse_checks.SENTINEL_CELL_NAME, green[0] == 0 and "复用哨兵（crates 变异表）：种子 5" in green[2] and len(forced) == 1
                   and skipped[0] == 1 and "复用失灵：" in skipped[1],
                   f"照常那一趟退 {green[0]}、强制现判 {forced}：{green[2][-400:]}；不读库那一趟退 {skipped[0]}：{skipped[1][-400:]}")


def selftest_parallel_key_cell(selftest, work):
    """算闭包的子进程数不同（就地算与 fork 4 个子进程），每一行的键逐字相同（并行之后重新证它与串行一致）；行数要够 PARALLEL_MINIMUM 才真起子进程。"""
    count = mutation_units.PARALLEL_MINIMUM + 2
    functions = "".join(f"pub fn step_{index}(value: u32) -> u32 {{ value + {index + 100} }}\n" for index in range(count))
    tests = "".join(f"    #[test]\n    fn steps_{index}() {{ assert_eq!(super::step_{index}(1), {index + 101}); }}\n" for index in range(count))
    source = "// 并行算闭包那一格的样本 crate\n" + functions + "\n#[cfg(test)]\nmod tests {\n" + tests + "}\n"
    rows = [f"step-{index}\tcrates/tiny/src/lib.rs\t{{ value + {index + 100} }}\t{{ value + {index + 200} }}\t-p tiny --lib -- steps_{index}\tsteps_{index}"
            for index in range(count)]
    repository = os.path.join(work, "parallel-keys")
    build_selftest_repository(repository, rows, source=source)
    saved = os.environ.get("GATE_MUTATION_KEY_WORKERS")
    outputs = {}
    try:
        for workers in ("1", "4"):
            os.environ["GATE_MUTATION_KEY_WORKERS"] = workers
            outputs[workers] = run_command_capturing(command_row_plan, [repository])
    finally:
        if saved is None:
            os.environ.pop("GATE_MUTATION_KEY_WORKERS", None)
        else:
            os.environ["GATE_MUTATION_KEY_WORKERS"] = saved
    keys = [line.split("\t")[1] for line in outputs["4"][1].splitlines() if "\t" in line]
    selftest.check(f"算闭包的子进程数不同（就地算与 4 个子进程）：{count} 行的键逐字相同、没有一行拒算",
                   outputs["1"][0] == 0 and outputs["4"] == outputs["1"] and len(keys) == count and "-" not in keys,
                   f"1 个：{outputs['1'][1][:400]}；4 个：{outputs['4'][1][:400]}")


SELFTEST_WGSL_KERNEL = '''// WGSL 按 fn 取键那一格的样本内核
const SCALE: u32 = 3u;
fn scaled(value: u32) -> u32 {
    let base = value + 1u;
    return base * SCALE;
}
fn untouched(value: u32) -> u32 {
    return value + 5u;
}
// 只在注释里的一句：wgsl 样本的注释行
'''
SELFTEST_WGSL_ROWS = [
    "wgsl-scaled-doubles\tcrates/tiny/src/kernel.wgsl\treturn base * SCALE;\treturn base * 2u;\t-p tiny --lib -- doubles\tdoubles",
    "wgsl-scale-four\tcrates/tiny/src/kernel.wgsl\tSCALE: u32 = 3u\tSCALE: u32 = 4u\t-p tiny --lib -- doubles\tdoubles",
    "wgsl-comment-only\tcrates/tiny/src/kernel.wgsl\twgsl 样本的注释行\twgsl 样本的另一行\t-p tiny --lib -- doubles\tdoubles",
]


def selftest_wgsl_unit_key_cell(selftest, work):
    """WGSL 按 fn 取键（lib_mutation_units.py 文件头「单元」）：一份 .wgsl 两个 fn（scaled、untouched）加一个模块级常量 SCALE，
    改坏 scaled 的一行与改坏常量的一行键算得出、原文只在注释里的一行拒算；改 untouched 之后两行的键都不变，改 scaled 的别一行之后只有改坏 scaled 的那一行键变。
    键从 row-plan 读（只读，不起 cargo）。弄坏开关 wgsl-whole-file（.wgsl 照改前取整份字节的 sha256）下这一格必须判错。"""
    repository = os.path.join(work, "wgsl-units")
    build_selftest_repository(repository, SELFTEST_WGSL_ROWS)
    kernel = os.path.join(repository, "crates", "tiny", "src", "kernel.wgsl")
    selftest_write(kernel, SELFTEST_WGSL_KERNEL)

    def keys():
        exit_code, output = run_command_capturing(command_row_plan, [repository])
        found = {}
        for line in output.splitlines():
            fields = line.split("\t")
            if len(fields) == 5 and fields[0].isdigit():
                found[int(fields[0])] = fields[1]
        return exit_code, [found.get(line_number) for line_number in (2, 3, 4)], output

    first_code, first, first_output = keys()
    replace_in_file(kernel, "return value + 5u;", "return value + 6u;")
    other_code, other, other_output = keys()
    replace_in_file(kernel, "let base = value + 1u;", "let base = value + 2u;")
    mutated_code, mutated, mutated_output = keys()
    keyed = all(key is not None and re.fullmatch(r"[0-9a-f]{64}", key) for key in first[:2])
    selftest.check("WGSL 按 fn 取键：改坏 fn 与改坏模块级常量的两行键算得出、原文只在注释里的一行拒算；改另一个 fn 之后两行键都不变，改被改坏的 fn 的别一行之后只有那一行键变",
                   (first_code, other_code, mutated_code) == (0, 0, 0) and keyed and first[2] == "-"
                   and other[:2] == first[:2] and mutated[0] != other[0] and mutated[1] == other[1],
                   f"头一趟 {first}；改 untouched 之后 {other}；改 scaled 之后 {mutated}：{first_output[-300:]}{other_output[-300:]}{mutated_output[-300:]}")


def judging_copy(work, name):
    """判法的一份临时拷贝（判法单元闭包碰到的那几份文件，照原路径放），给判法单元那几格改着看指纹变不变：(拷贝的根, 拷没拷成)。"""
    copy, source = os.path.join(work, name), JUDGING_ROOT[0]
    try:
        _key, references = judging_units({}, source)
    except (JudgingUnitsRefused, OSError):
        return copy, False
    # lib_function_units.py 的单元在 references 里（照进指纹）；lib_verdict_library.py（NOT_JUDGING_FILES 的另一份）闭包里没有它的单元，拷贝照旧带上
    for path in sorted({reference.file for reference in references} | {".claude/scripts/project_preflight.py", "scripts/crash/lib_verdict_library.py"}):
        if os.path.isfile(os.path.join(source, path)):
            os.makedirs(os.path.dirname(os.path.join(copy, path)), exist_ok=True)
            shutil.copyfile(os.path.join(source, path), os.path.join(copy, path))
    return copy, True


def selftest_judging_unit_cells(selftest, work):
    """判法单元指纹（文件头「判法单元」）：判法的拷贝上算出的与本仓的相同；只改判法脚本的注释不变（弄坏开关 judge-units-whole-file 下变、这一格判错）；
    改 libtest 结果行解析的一处就变；改判法单元闭包外的函数（派活的 run_pool）不变。"""
    environment = dict(selftest.base_environment)
    copy, copied = judging_copy(work, "judging-cells")
    try:
        real_key, real_references = judging_units(environment)
        copied_key, _references = judging_units(environment, copy)
    except (JudgingUnitsRefused, OSError) as error:
        selftest.check("判法单元算得出（闭包里没有拒算）", False, str(error))
        return
    selftest.check("判法单元算得出、在判法的临时拷贝上算出同一个指纹（键里不带路径）", copied and copied_key == real_key and len(real_references) > 10,
                   f"本仓 {real_key[:16]}…、拷贝 {copied_key[:16]}…，{len(real_references)} 个单元")
    module = os.path.join(copy, "scripts", "mutation", "crates-mutation-rows.py")
    replace_in_file(module, "    tail = output_tail(output)\n", "    # 判法单元自证加的一行注释\n    tail = output_tail(output)\n")
    commented_key, _references = judging_units(environment, copy)
    selftest.check("只改判法脚本的注释：判法单元指纹不变", commented_key == real_key, f"{real_key[:16]}… → {commented_key[:16]}…")
    # 改的是 run_pool 的函数体：判法入口点不到 run_pool，函数体只在调它时才跑，import 时不执行。
    # 不改它的签名：加一个带缺省值的参数，缺省值在 import 时求值，def 头按 scripts/gate/lib_function_units.py 文件头「Python 闭包：import 时执行的模块级语句」进闭包，指纹该变
    replace_in_file(module, "    sequence_counter = multiprocessing.Value(\"i\", 0)\n", "    sequence_counter = multiprocessing.Value(\"i\", 1)\n")
    scheduling_key, _references = judging_units(environment, copy)
    selftest.check("改判法单元闭包外的函数（派活的 run_pool）：判法单元指纹不变", scheduling_key == commented_key or break_is(environment, "judge-units-whole-file"),
                   f"{commented_key[:16]}… → {scheduling_key[:16]}…")
    replace_in_file(os.path.join(copy, "scripts", "gate", "lib_libtest_result_lines.py"),
                    "        if parsed.full_name == expected or parsed.full_name.endswith(\"::\" + expected):\n",
                    "        if expected == parsed.full_name or parsed.full_name.endswith(\"::\" + expected):\n")
    parsing_key, _references = judging_units(environment, copy)
    selftest.check("改 libtest 结果行解析函数的一处：判法单元指纹变", parsing_key != scheduling_key, f"{scheduling_key[:16]}… → {parsing_key[:16]}…")
    # lib_function_units.py 管覆盖落进哪几个单元（executed_units 用它给的行号区间），是判对错的一环：改它的 RustFile.row 一处函数体，判法单元指纹变、闭包里有它的单元。
    # D13 已定项 17 的文件级排除不用在这里；弄坏开关 judging-function-units-excluded 下这份的单元不进指纹、这一格判错
    row_line = '        return snippet_of(self.source_rows[line - 1]) if 0 < line <= len(self.source_rows) else ""\n'

    def edit_function_units(judging_root):
        units_file = os.path.join(judging_root, "scripts", "gate", "lib_function_units.py")
        with open(units_file, encoding="utf-8") as handle:
            units_text = handle.read()
        assert units_text.count(row_line) >= 1, units_file
        selftest_write(units_file, units_text.replace(row_line, row_line.replace("0 < line", "1 <= line"), 1))

    edit_function_units(copy)
    function_units_key, function_units_references = judging_units(environment, copy)
    function_units_count = sum(reference.file == "scripts/gate/lib_function_units.py" for reference in function_units_references)
    selftest.check("改 lib_function_units.py 的 RustFile.row 一处函数体：判法单元指纹变，闭包里有 lib_function_units.py 的单元",
                   function_units_key != parsing_key and function_units_count > 0,
                   f"{parsing_key[:16]}… → {function_units_key[:16]}…；lib_function_units.py 的单元 {function_units_count} 个")
    # 覆盖号带判法单元指纹：同一条点名测试，判法的拷贝只差 lib_function_units.py 的切单元，覆盖号要跟着变（弄坏开关 coverage-id-without-judging 下不变、这一格判错）
    coverage_repository = os.path.join(work, "judging-coverage-id")
    build_selftest_repository(coverage_repository, SELFTEST_UNIT_ROWS, source=SELFTEST_UNIT_SOURCE)
    coverage_rows = read_table(os.path.join(coverage_repository, TABLE), environment)[0]

    def coverage_id_on(judging_root):
        _context, found, _library = plan_rows(coverage_repository, coverage_rows[:1], environment, "1G", 60, judging_root)
        return found[coverage_rows[0].line_number].coverage_id, found[coverage_rows[0].line_number].judging

    coverage_copy, coverage_copied = judging_copy(work, "judging-coverage-id-cells")
    original_coverage_id, original_judging = coverage_id_on(coverage_copy)
    edit_function_units(coverage_copy)
    edited_coverage_id, edited_judging = coverage_id_on(coverage_copy)
    selftest.check("覆盖号随判法单元指纹变：改 lib_function_units.py 的 RustFile.row 一处函数体，同一条点名测试的覆盖号跟着变",
                   coverage_copied and bool(original_coverage_id) and bool(edited_coverage_id) and original_judging != edited_judging and original_coverage_id != edited_coverage_id,
                   f"判法 {original_judging[:16]}… → {edited_judging[:16]}…；覆盖号 {original_coverage_id[:16]}… → {edited_coverage_id[:16]}…")
    shown = run_command_capturing(command_judging_units, [copy])
    selftest.check("judging-units 子命令打「<指纹> <单元数>」", shown[0] == 0 and re.fullmatch(r"[0-9a-f]{64} \d+\n", shown[1]) is not None, shown[1])


def build_coverage_repository(directory):
    """插桩编译那一格的样本仓：crates/tiny 用 SELFTEST_COVERAGE_SOURCE，另有一个 -p tiny 编不进去的包 crates/other。"""
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


SELFTEST_ANCHOR_MISSING_ROW = "anchor-missing\tcrates/tiny/src/lib.rs\t{ left / right }\t{ left }\t-p tiny --lib -- adds\tadds"
# 改的是数据文件（不是 .rs）而原文命中 0 次的一行：单元键不核锚点（数据文件按整份字节算），不先剔掉就算得出键、会被双机分片的驱动分给第二台
SELFTEST_DATA_FILE_ANCHOR_ROW = "data-file-anchor-missing\tfixture-data/answer.txt\t43\t44\t-p tiny --lib -- adds\tadds"
# 牵连面那一格：单元键样本 crate 的 5 行再加一行改 OFFSET（check_root 用的常量），点名 checks_root：checks_root 的闭包进了三行
SELFTEST_IMPACT_ROW = "offset-plus-one\tcrates/tiny/src/lib.rs\tpub const OFFSET: u32 = 95;\tpub const OFFSET: u32 = 96;\t-p tiny --lib -- checks_root\tchecks_root"
SELFTEST_IMPACT_TEST = "// TEST checks_root NEEDS OFFSET: u32 = 95 RUNS checks_root,check_root\n"
# 照闭包手数：6 行的闭包都有 mod tests 那个单元；checks_root 的闭包（tests::checks_root、check_root、OFFSET）进了三行；
# 计数 1 的按名字排、add 最前（helps 的闭包里有它，helper 是那一行被改坏的单元）
SELFTEST_IMPACT_EXPECTED = ("牵连最大的函数单元：crates/tiny/src/lib.rs::tests 进了 6 行的闭包（前 5：crates/tiny/src/lib.rs::tests 6、crates/tiny/src/lib.rs::OFFSET 3、"
                            "crates/tiny/src/lib.rs::check_root 3、crates/tiny/src/lib.rs::tests::checks_root 3、crates/tiny/src/lib.rs::add 1）；一次改动最坏牵连 6 行"
                            "（算了 6 行的闭包；拒算与锚点失效的 0 行没有闭包、没算进）")


def selftest_anchor_and_impact_cells(selftest, work):
    """锚点失效按行判（文件头「锚点失效」）与牵连面（文件头「牵连面」）。"""
    # 锚点失效按行判：第 6 行原文命中 0 次，记锚点失效、整道判红；别的 4 行照跑、照入库；第二趟那 4 行复用、锚点失效那一行照旧红、不入库
    anchor_root = os.path.join(work, "anchor-per-row")
    build_selftest_repository(anchor_root, SELFTEST_TABLE_ROWS + [SELFTEST_ANCHOR_MISSING_ROW])
    anchor_pristine = source_hash(anchor_root)
    anchored = selftest.run(anchor_root)
    records_directory = os.path.join(library_of(anchor_root), "records")
    records_after_first = sorted(os.listdir(records_directory)) if os.path.isdir(records_directory) else []
    anchored_again = selftest.run(anchor_root)
    anchor_line = f"{TABLE}:6 anchor-missing：锚点失效——原文在 crates/tiny/src/lib.rs 里命中 0 次，要恰好 1 次（这一行没跑、不入库、不复用）"
    selftest.check("锚点失效按行判：那一行记锚点失效、别的行照跑照入库（计数报锚点失效 1 行、整道判红；第二趟那 4 行复用、锚点失效那一行照旧红）",
                   anchored[0] == 1 and len(mutation_invocations(plain_invocations(anchored[3]), anchor_pristine)) == 4
                   and "计数：抓到 4 条、没红 0 条" in anchored[1] and "scope 起不来没跑 0 条、锚点失效 1 行（共 5 条）" in anchored[1]
                   and anchor_line in anchored[1] and "有几行锚点失效（锚点失效 1 行" in anchored[1] and len(records_after_first) == 4
                   and anchored_again[0] == 1 and anchored_again[3] == [] and "复用 4 条" in anchored_again[1] and anchor_line in anchored_again[1],
                   anchored[1] + anchored[2][-800:] + "\n".join(anchored[3]) + f"\n记录 {len(records_after_first)} 条\n" + anchored_again[1])
    # row-plan、plan、--list-items 把锚点失效的行单列：第 6 行（.rs）与第 7 行（数据文件，单元键不核锚点）都写「-」键与 anchor-broken，不算进现判
    listed_root = os.path.join(work, "anchor-listed")
    build_selftest_repository(listed_root, SELFTEST_TABLE_ROWS + [SELFTEST_ANCHOR_MISSING_ROW, SELFTEST_DATA_FILE_ANCHOR_ROW])
    row_plan = run_command_capturing(command_row_plan, [listed_root])
    plan_lines = {line.split("\t")[0]: line.split("\t") for line in row_plan[1].splitlines() if "\t" in line}
    plan = run_command_capturing(command_plan, [listed_root])
    listed = run_command_capturing(command_list_items, [listed_root])
    selftest.check("row-plan、plan、--list-items 把锚点失效的行单列（数据文件那一行也不算键）",
                   row_plan[0] == 0 and plan_lines.get("6") == ["6", "-", "0", "-", "anchor-broken"] and plan_lines.get("7") == ["7", "-", "0", "-", "anchor-broken"]
                   and all(plan_lines.get(str(number), ["", "-"])[1] != "-" and plan_lines[str(number)][4] == "-" for number in range(2, 6))
                   and plan[0] == 0 and "6\tanchor-missing\t锚点失效\t锚点失效\t原文在 crates/tiny/src/lib.rs 里命中 0 次" in plan[1]
                   and plan[1].splitlines()[-1].startswith("合计 6 行：") and plan[1].splitlines()[-1].endswith("拒算 0 行、锚点失效 2 行")
                   and listed[0] == 0 and listed[1].count("；锚点失效：") == 2
                   and "锚点失效 2 行（不跑、整道判红：第 6 行 anchor-missing、第 7 行 data-file-anchor-missing）" in listed[1],
                   row_plan[1] + plan[1] + listed[1])
    # 牵连面：单元键样本 crate 上 6 行，checks_root 点名三行（root-457、root-double、offset-plus-one），它的闭包（tests::checks_root、check_root、OFFSET）各进 3 行；
    # 弄坏开关 impact-row-dropped 少算头一行（root-457），数就对不上
    impact_root = os.path.join(work, "impact")
    build_selftest_repository(impact_root, SELFTEST_UNIT_ROWS + [SELFTEST_IMPACT_ROW], SELFTEST_IMPACT_TEST, source=SELFTEST_UNIT_SOURCE)
    impact_listed = run_command_capturing(command_list_items, [impact_root])
    impact_lines = [line for line in impact_listed[1].splitlines() if "牵连最大的函数单元：" in line]
    impact_run = selftest.run(impact_root)
    run_impact_lines = [line for line in impact_run[1].splitlines() if "牵连最大的函数单元：" in line]
    selftest.check("牵连面：样本 crate 上牵连最大的单元与行数（--list-items 末了与成功句之后各一行，数一样）",
                   len(impact_lines) == 1 and impact_lines[0] == "  · " + SELFTEST_IMPACT_EXPECTED
                   and impact_run[0] == 0 and len(run_impact_lines) == 1 and run_impact_lines[0] == "  " + SELFTEST_IMPACT_EXPECTED
                   and impact_run[1].splitlines()[-1] == run_impact_lines[0] and impact_run[1].splitlines()[-2].startswith("  ✓ crates 变异表复跑：6 条"),
                   impact_listed[1] + impact_run[1])


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
    commands = {"static-check": command_static_check, "judging-units": command_judging_units, "toolchain": command_toolchain,
                "library-directory": command_library_directory, "import": command_import, "row-plan": command_row_plan, "plan": command_plan,
                "llvm-tools": command_llvm_tools, "--list-items": command_list_items}
    if not arguments or arguments[0] not in commands:
        print("  ✗ 用法：crates-mutation-rows.py {static-check|judging-units|toolchain|library-directory|import|row-plan|plan|llvm-tools|--list-items|--item} … 或 --selftest",
              file=sys.stderr)
        print("     → 怎么办：各子命令的参数见文件头；整张表的复跑只经门禁 checker-tier-crates-mutation-replay.sh（bash .claude/gate.d/checker-tier-crates-mutation-replay.sh）", file=sys.stderr)
        return 2
    return commands[arguments[0]](arguments[1:])


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
