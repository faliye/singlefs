#!/usr/bin/env python3
# task-server-cargo-family: -p singlefs-unit-graph --release
# admission: always 判的是此刻被判的仓、它的 crates/mutations.tsv 与 git common-dir 里那一份判定库，上一次的结论不替这一次作保
# run-condition: none 算判法单元、行键要的 git、cargo、rustc 由 admission.py 的 git_common_directory 与 lib_mutation_units.py 的 toolchain_lines 起、起不来报错退 2；llvm-profdata、llvm-cov 由 run_stage 现找、找不到判红；--selftest 用自己造的假 cargo 与假 llvm 工具，不编译
"""门禁 crates-mutation-replay（crates 变异表复跑）的判法与判定库，连同门禁 code-source-discipline 静态判的那几样：一份判法只在这里，两道阶段都调它。

变异表一行成形 = 六段制表符分隔（变异名、文件、原文、替换文、cargo test 参数、必须红的测试名）；替换文可以空（删掉原文：改坏时原文换成空串），
其余五段一段都不许空（malformed_reason，read_table 照它读；crates-mutation-replay、static-check 同一份）。

crates-mutation-replay 经 import 调 run_stage（阶段文件里那几行内嵌 python）；这里不给整张表开命令行入口：项目代码闸（.claude/hooks/project-code-guard.sh）
按阶段文件名认 crates-mutation-replay，另开一个入口它认不出。

子命令（<根> 是被判仓的根）：
  static-check <根> [--table <变异表>]
                                        crates-mutation-replay 派活之前判的每一样（row_problems，crates-mutation-replay 的 prechecks 调的是同一个函数）连同拷贝范围 ②，
                                        每处一行（制表符分隔），退 0 一处都没有、退 1 有、退 2 读不了登记表或变异表：
                                          MALFORMED <行号> <变异名> <为什么>             这一行不成形（见上）
                                          ANCHOR   <行号> <变异名> <为什么>              原文在「文件」那一段指的源码里不是恰好命中一次，或文件不在、读不了
                                          NAME     <行号> <变异名> <测试名>              必须红的测试名不是一个用例名（带了正则符号或空白）
                                          ROW      <行号> <变异名> <文件>                ① 变异表这一行改的文件不在 crates-mutation-replay 每片拷的范围里
                                          STRAY    <行号> <变异名> <段名> <转义>         原文或替换文里有 \\n 以外的反斜杠转义
                                          LITERAL  <文件> <行号> <字面量> <解析到的路径> ② crates/ 下 .rs 里以 "../ 起头、按所在 crate 的根解析之后
                                                                                         逃出 crates/、又不在拷贝范围里的字符串字面量（只 code-source-discipline 判，crates-mutation-replay 靠基线）
                                          UNDEFINED <行号> <变异名> <文件> <标识符，逗号分隔>  替换文施加之后新出现、被改文件与它所在的包的 .rs 里都一处没有的标识符
                                                                                         （replacement_undefined_identifiers；替换文自己绑定的、方法与路径里的段、宏、属性里的、_ 起头的、
                                                                                         标准库预导入与基本类型不算；只 static-check 判，crates-mutation-replay 派活之前不判、套用后编不过记无效）
                                          IDENTIFIERS_CHECKED <判了标识符的行数> <没判的行数（文件不是 .rs 或锚点失效）>
                                          RELEASE  <行号> <变异名> <依据，；分隔>                命中重测试目标而第五段没写 release（见「重测试目标只许 release」；--profile release 也算写了）：
                                                                                         有这一行退 1，这几行之后一行「     → 怎么办：…」写出路（第五段补上 --release）；
                                                                                         判的是表里每一行（行数在 CHECKED 那一行），只 static-check 判，crates-mutation-replay 派活之前不判、照补 --release 起；
                                                                                         登记表读不了打 ERROR 行与「→ 怎么办」、退 2
                                        另判「点名测试够不着被改坏的函数」（判法见文件头那一节，一律判），打这几种行：
                                          UNREACHABLE <行号> <变异名> <closure 或 gpu-route> <被改坏的单元，、分隔> <点名测试> <门禁给这一行用的 cargo 参数>
                                                                                         点名测试在门禁实际的特性与多机配置下调不到被改坏的函数；有这一行退 1，
                                                                                         这几行之后一行「     → 怎么办：…」写出路
                                          REACHABILITY_EXEMPT <行号> <变异名> <cpu-configuration 或 gpu-tables-refused> <凭据单元>  照豁免认了、不判的行
                                          REACHABILITY_UNJUDGED <行号> <变异名> <为什么>   判不了的行（锚点失效、被改坏的单元或点名测试拒算、闭包算不成、闭包拒算而已收部分里没有被改坏的单元）
                                          REACHABILITY_CHECKED <判了的行数> <没判的行数> <豁免的行数> <截过闭包的行数> <多机配置> <ENABLE_GPU=1 或 GPU 没开>
                                          REACHABILITY_NOT_JUDGED <为什么>               选路表、截断表、豁免表点名的函数在 <根> 的源码里一个都没有（门禁样本那种小根）：这一趟没判可达性，
                                                                                         照实报，替换上面 REACHABILITY_CHECKED 那一行（门禁成功句写「可达性本次未判」，不算判过）
                                        选路表、截断表、豁免表里有的函数在 <根> 的源码里找不到（不是一个都没有）、多机配置读不了，打 ERROR 行与「→ 怎么办」、退 2
                                          COPIED   <拷贝范围，空格分隔>
                                          CHECKED  <成形的行数> <.rs 份数> <逃出 crates/ 的字面量处数（在范围里的与不在的合计）>
                                        门禁 code-source-discipline 调它；--table 不给就读 <根>/crates/mutations.tsv（不在当空表）；
                                        参数不对（缺 <根>、--table 缺值、<根> 不是目录）与 --table 指的文件不在都打 ERROR 行（原因）、
                                        下一行「→ 怎么办」，退 2；变异行的 --manifest-path 写坏了（见「--manifest-path」）同样打 ERROR、退 2
  judging-units <根> [--units]          判法单元指纹（见「判法单元」）：打「<sha256> <单元数>」，带 --units 再逐个打单元（种类、文件:起-止、限定名、指纹）；算不出（拒算）退 2
  toolchain <根>                        行键里的工具链行（rustc -vV、cargo -V、构建环境解析后的值，照起 cargo 时的环境算、含关调试信息那四个变量）；双机分片的驱动拿两台的比。算不出退 2
  library-directory <根>                这棵树此刻那一份判定库的库目录名（prune-server 里那份库的名字与命名空间，本地没有文件）；算不出退 2
  row-plan <根> [--rows <选行文件>]     双机分片的驱动（scripts/mutation/mutation-shard-run.sh）分行用：每一条成形的变异打一行
                                          <行号> <行键，拒算写 -> <这一趟会不会复用，1 或 0> <上一次现跑的秒数，没记过写 -> <anchor-broken 或 ->
                                        （制表符分隔；「会不会复用」与 run_stage 同一份判法 row_decision；
                                        锚点失效的行（见「锚点失效」）不算键，打「<行号> - 0 - anchor-broken」）；
                                        带 --rows 时只对选行文件（一行一个行号，写法同 GATE_MUTATION_ROW_SELECTION）点名的行算键、判复用（与 run_stage 的选行模式同一份 plan_rows、decide_rows），
                                        别的行只给用时（键写 -、复用写 0）：点名一行不付整张表算键的代价，驱动分行取没记过用时的中位数仍取全表；
                                        stderr 打一行「  · row-plan：算了 <N> 行的键（…）」，N 是这一趟真算键的行数（点名两行就是 2）；
                                        退 0；读不了表、上限写错、选行文件不对、算不出判法单元或工具链退 2
  plan <根>                             只读：不编、不跑、不写，逐行打「<行号>\\t<变异名>\\t复用|现判|拒算|锚点失效\\t<栏>\\t<细节>」（栏见「复用判据」，细节里列变了哪几个单元），
                                        末行「合计 <N> 行：复用 <M> 行、现判 <K> 行（<各栏的数>）、拒算 <R> 行、锚点失效 <A> 行」。退 0；读不了表、上限写错、算不出判法单元退 2
  --list-items <根>                     逐行打可点名的变异：「<变异名>\\t第 <行号> 行，改 <文件>」，锚点失效的那一行末尾接「；锚点失效：<为什么>」，不编、不跑（与门禁 --list-items 同一份）；
                                        末行合计这一趟键对不上（要现判）、对得上（复用）、拒算、锚点失效的各几条，锚点失效的逐行点名（行号与变异名）；
                                        再一行报牵连面（见「牵连面」）
  --list-items --with-keys [--rows <选行文件>] <根>
                                        另在逐行之后、末行合计之前，每项打一行 ITEM_PLAN（见「按项机读行」）；键没算出来退 1、不退回只列项名；
                                        --rows（门禁把 --item 点名的几行换成选行文件交来）只列、只算点名那几行的键与闭包，成本随点名的行数走，不随整表走
  --item <行号或变异名> [--item …] <根>  只判点名的几行：交给门禁 crates-mutation-replay.sh --item（同一份选行模式，点名单跑与整张跑时那一行判定相同，
                                        判定库照写、照复用）；点名的不在表里退 2。起 cargo、写判定库，经统一任务服务交
  sample-tools <目录>                   把自证用的假 cargo 与假 llvm 工具（FAKE_CARGO、FAKE_LLVM_TOOL）写成可执行文件放进 <目录>：cargo、llvm-profdata、llvm-cov。
                                        门禁 crates-mutation-replay 的样本档（样本根有 .crates-mutation-replay-sample-tools/ 时）把它放到 PATH 最前面、
                                        GATE_MUTATION_LLVM_TOOLS 指到它，样本 crate 里 `// TEST …` 与 `// COMPILE-ERROR-IF …` 几行就是假 cargo 判红绿的依据（写法见 FAKE_CARGO 头上的说明）；
                                        不起 cargo、不编译。退 0；<目录> 写不了退 1，用法不对退 2
  llvm-tools <根>                       打找到的 llvm-profdata、llvm-cov 与它们的 LLVM 版本；找不到或与 rustc 的 LLVM 版本对不上退 1、给出路
  --selftest                            自证（假 cargo、假 llvm 工具，在临时仓里跑 run_stage；格数由成功行现算）；
                                        有格的前提没造出来（并发领片号那一格两次调用没重叠）记本次未跑、逐格列出，别的格都对时退 77，不判绿

拷贝范围 = 被判那棵树的共用拷贝范围（copied_with_path_dependencies：scripts/hosts/lib_repository_copy.py 的 copy_closure，带测试代码扫——点名的测试在副本里跑：
  共用清单加 path 依赖指到的目录（例：验证包常在依赖的仓根 prune-server/）、编译期与运行期读的仓根文件连同它们 source、起的；那份文件头「闭包」），
  再加 .claude/gate.d/stage-inputs.tsv 里 crates-mutation-replay 那一行登记的路径（读本模块所在那一份仓的登记表：判别力样本的目录里没有登记表；
  经 declared 声明给 copy_closure，为什么是 REGISTERED_COPY_RANGE_REASON）；闭包算不成（path 依赖指到树之外、编译期读的文件认不出或不在）拒绝，
  run_stage 一条都不跑、static-check 打 ERROR 退 2，出路另写 <根> 是仓副本、缺的是闭包里的仓根文件那一种（照 minimal-copy.py 另建副本，COPY_REBUILD_HOWTO）。
  多机配置怎么交给副本里真跑的测试见「真跑测试的格把多机配置交给测试」一段，只编不跑的不带，
  再加变异行 --manifest-path 指的包目录里还不在范围里的（copied_with_manifest_directories，见「--manifest-path」；连同它的 path 依赖），
  底下每一段都不叫 target（各 crate 自己的编译目录）。crates-mutation-replay 照它拷每片的源码副本，code-source-discipline 照它静态判；判法只有 is_copied_into_each_shard 一处。
  ② 管不到的：路径拼在变量里、环境变量给的、format! 拼出来的、不以 "../ 起头而运行期再 join("..") 的；#[path = …] 不算（编译期带进来的源文件，
  副本里编得过就在）；指到仓根本身（"../../" 这一类）按不在范围里算（之后拼上的是哪一份判不出）。

锚点失效（anchor_problem，row_problems 的 ANCHOR 与 run_stage、plan、row-plan、--list-items 同一份判法）：原文在被判仓里「文件」那一段指的源码里不是恰好命中一次，
  或文件不在、读不了。按行判，不让整张表退出：那一行记它自己的判定「锚点失效」（不起 cargo、不算键、不写判定库、不复用，整道判红），别的行照判、照入库；
  「计数：」那一行有「锚点失效 N 行」一栏，判红的那一段逐行点名（行号、变异名、为什么）；选行（--item、双机分片的驱动）同样按行。
  派活之前另几样（不成形、测试名、拷贝范围、转义）照旧有一行就整道判红、一个工作进程都不起，那时锚点失效的行连同它们一起列出。
  row-plan 给锚点失效的行写「-」键与 anchor-broken，双机分片的驱动不把它们分给第二台。

--manifest-path（变异行点仓根工作区之外的独立工程，例仓根 Cargo.toml 的 exclude 列着的 prune-server/）：cargo 参数在 `--` 之前写
  --manifest-path <路径> 或 --manifest-path=<路径> 的行，包目录取那份 Cargo.toml 所在的目录（manifest_directory_of，相对 <根>；没写的是 <根> 那一份工作区）：
  · cargo 照原样带 --manifest-path 起（基线的批、变异、记覆盖都在副本根起，相对路径落在副本里那个目录）；基线按 `--` 之前整段分批，带不带、指哪一份各成一批；
  · 拷贝范围：那个目录的 Cargo.toml 已在范围里（仓根 prune-server/ 经 验证包的 path 依赖已拷）不动，不在就把那个目录补进来，连同它的 path 依赖（copied_with_manifest_directories）；
  · 单元键照那个包的源码算（unit_keys_under_manifest）：lib_mutation_units.py 的 UnitKeys 以包目录为根，被改坏的单元、点名的测试（在那个包的编译目标里找）、
    闭包与依赖版本行（那个包自己的 Cargo.lock）都在它上面算，单元的文件照仓根起的路径写；被改的文件不在那个包底下的这一行拒算；
    工具链行取 <根> 那一份、manifest 那几行换成那个包的 Cargo.toml 里的（KeyContext.toolchain_of）；记覆盖的工作进程与覆盖复用判据照同一份单元键核；
  · 写坏了拒：写了不止一次、后面缺路径、绝对路径、文件名不是 Cargo.toml、落在 <根> 之外、文件不在——run_stage 一条都不跑（prechecks）、
    static-check 打 ERROR 退 2，plan、row-plan、--list-items 把那一行记拒算。
  弄坏开关 manifest-path-ignored：不认 --manifest-path。

牵连面（impact_summary，只报数、不判红）：每一行的键依赖的单元 = 点名测试的单元闭包（KeyContext 为这一行算好的那一份，不另算）连同被改坏的单元；
  一个单元进了几行的这一份，改它一次就作废几行的判定。成功句之后与 --list-items 末了各打一行
  「牵连最大的函数单元：<文件>::<限定名> 进了 N 行的闭包（前 5：<单元> N、…）；一次改动最坏牵连 N 行」，单元按（种类、文件、限定名）认，
  N 相同按单元名排；拒算与锚点失效的行没有闭包，不算进，那一行末尾写明算了几行、没算几行。


复用：每行一条判定，行键是被改坏的函数单元指纹、原文、替换文、点名测试的单元闭包键、判「抓没抓到」的判法单元指纹、工具链、解析出的依赖版本，
  不带变异名、路径、测试名；没有管整库的版本号、没有版本目录；一份库，两台判出的都经 prune-server 写进同一份（分片驱动 scripts/mutation/mutation-shard-run.sh
  合并时按此刻的行键核两片判的行在服务里读得到，它文件头「合并」）。
  没有整道全绿标记，不问 stage-must-run.sh。单元的切法、指纹与闭包在 scripts/gate/lib_function_units.py，这里经 scripts/mutation/lib_mutation_units.py 调
  （mutate.sh 的按条库也用那一份，文件头写了单元、工具链行、依赖版本行与库的存放）。

判法单元（judging_units，JUDGING_ENTRIES）：判「抓没抓到」的那几个函数——结局分类 verdict_of_run（连同它调到的 libtest 结果行解析，
  scripts/gate/lib_libtest_result_lines.py 一份）、基线分类 baseline_problem、覆盖复用判据 baseline_reuse_problem、切 fn 规则
  （lib_mutation_units.py 的 UnitKeys.mutated_units、resolve_test、executed_units，连同它们调到的 lib_function_units.py），
  连同真参与判结局的那几段（JUDGING_ENTRIES 的后一半：judge_row 施加与还原变异、judge_batch 与 baseline_arguments、row_baseline_output 拼基线与按行取结果、
  with_shown_output 补 --show-output、run_cargo_command 起 cargo、prepare_shard 拷每片的源码副本）——从它们收 python 闭包合起来，
  起进程那一下（run_capturing_output 的 Popen）、找多机配置文件（test_runs_multi_host_configuration）与起这一次自己的临时 prune-server（mutation_run_prune_server）
  经 init_worker 交进 WORKER_STATE 的引用调，不进闭包（起的程序各有身份；多机配置是交给测试的一份文件；临时服务是这一次自己的一份空库，起它的
  scripts/prune-server/prune_server_client_selftest.py 是客户端的自证，载入它会经按名字连的边把自证起的 admission.py 与 lib_function_units.py 自证里
  折不出命令的 subprocess 一起拉进闭包，整张表拒算）；记覆盖的 judge_coverage 不进（它的闭包够到 lib_function_units.py 的自证夹具，折不出路径的 subprocess 拒算），
  覆盖落单元的判法另由 executed_units 管。弄坏开关 judging-entries-verdict-only：只收结局分类那几个；
  judging-entries-with-temporary-service：另从 mutation_run_prune_server 收闭包，run_cargo_command 直接调它。
  指纹 = 闭包里单元（种类，去注释指纹）的多重集；scripts/crash/lib_verdict_library.py 的 NOT_JUDGING_FILES 那两份（D13 已定项 17 的文件级排除）不在这里排：
  lib_function_units.py 经 executed_units 管覆盖落进哪几个单元，覆盖决定基线复用，是判对错的一环，它的单元照进指纹、里面的拒算照算。
  只改注释、文档串、别的函数（阶段脚本的调度、判定库的存放、派活与报数）指纹不变；
  改了闭包里任何一个函数（含 lib_function_units.py 的切单元），整张表的行键、基线键与覆盖号都变。闭包里有拒算（getattr 变量名、折不出的 import 路径这一类）就算不出，这一道判红、一条都不跑。
  磁盘缓存（judging_units_cached）：现算一趟要把闭包碰到的几百份脚本都解析一遍（一分钟上下），点名几行的 row-plan 又每趟都是新进程，指纹与单元清单按「算这一趟读过的每一处」存成一份 json，
  放 ${GATE_CROSS_RUN_TMPDIR}/singlefs-judging-units-cache/，没设放本仓的 git common-dir 底下（git worktree 里是主仓的 .git；不在 git 仓里放 TMPDIR 底下）（GATE_JUDGING_CACHE_DIRECTORY 设了放那里，自证用；只留最近 16 份）。读过的是：打开读的每份文件的内容 sha256、
  isfile / isdir / exists / listdir 问过的答案、仓里脚本文件（.py、.sh、.bash）名单的摘要，再加现算用的代码本身（本文件、lib_mutation_units.py、lib_function_units.py）的内容摘要；
  下一趟每一处现读都与存的相同才取，不看修改时刻、不设时限；任何一处不同整个重算、重存；算的当中读过的文件被改了、闭包有拒算不存。

行键（KeyContext.components，文件头「复用」那一句的逐样）：
  被改坏的单元   原文所在那份 .rs 里与原文重叠的最里层单元（fn 项；函数外的项按项单元：const、static、struct、impl 头、宏……）的（种类，指纹）：
                 所在文件与限定名不进键（D13（验证路线） 已定项 17：路径、名字、行号不进键）；
                 原文不落在任何单元上拒算；.wgsl 按 fn 切（原文所在的 wgsl fn，不在 fn 里的取它自己的常量、struct、绑定声明单元，取不到拒算，
                 不退回整文件）；别的不是 .rs 的（litmus/ 这类测试读的数据文件）取整份字节的 sha256
  原文与替换文   原样（\\n \\t \\\\ 转义）
  点名测试       cargo 参数选中的目标里那条 #[test] fn 单元的闭包键（入口 = 那条测试函数，lib_function_units.py 的 rust_closure_of，
                 带上 cargo 参数里的特性）；认不出是哪一个单元、或闭包拒算，这一行拒算
  开关与特性     cargo 参数里不是名字的开关（--release、--include-ignored 这一类）与特性
  工具链         lib_mutation_units.py 的 toolchain_lines（rustc -vV、cargo -V、构建环境解析后的值），照起 cargo 时的环境算：含关调试信息那四个变量（见「调试信息」）
  依赖版本       点名测试的闭包与被改坏的单元真点名的外部 crate 在 Cargo.lock 里落到的包与传递依赖（lib_mutation_units.py 的 dependency_lines）
  拒算的行每趟照跑、不复用、不写记录，成功句另报「拒算 R 条」，stderr 逐行写为什么。
  撞键就拒算（refuse_colliding_row_keys）：表里两行以上算出同一把行键（两份内容相同的函数各改一处、点名同一条测试），
    这几行拒算：照跑、不复用、不写记录；stderr 逐行写「拒算（行键撞键：与第 N 行同键）」，成功句末尾另接「；其中撞键拒算 K 条：第 <行> 行拒算（行键撞键：与第 N 行同键）、…」。
    基线键与覆盖号不带被改坏的单元，不跟着拒算（两行各自的基线照常复用）。选行（--item、双机分片的驱动、row-plan --rows）只对选中的行算键时，
    整张表里原文与替换文都与选中的某一行相同的行（键撞得上的只有这几行）一起算键，撞键照整张表判。弄坏开关 row-key-collision-unrefused：不判撞键。

判定库（crates-mutation-replay 每条判完立刻写，下一趟按「复用判据」逐行判复用还是现判；存放在 lib_mutation_units.py 的 Library，读写都经 prune-server）：
  库的名字是库根的 singlefs-crates-mutation-library（只当 prune-server 里的名字与命名空间，本地不建、不读、不写；键前缀照
  scripts/prune-server/prune_server_client.py 文件头「键空间」：mutation-units[@命名空间]/singlefs-crates-mutation-library/）。库根是被判那棵树的 git common-dir
  （records_home）；取不到（不是 git 工作树）一条都不跑（run_stage 退 1、library-directory 退 2），不退回别处（弄坏开关 library-outside-git-tolerated 下拿被判的根当库根往下跑）。
  键里只用库名，哪棵仓不进键：主仓与仓副本（git init 过的）写进同一份；命名空间只在临时服务底下给（客户端 namespace_of，自证一趟一个库）。
  成功句末尾点名库在哪。服务连不上客户端退 75，这一道照 75 退出，不当「库里没有」现判：
    records/<行键>        一条变异的判定（判定、要打的那一行、判定日期、跑法、点名测试的闭包号、被改坏的单元号、工具链与依赖的号）
    baselines/<基线键>    一条点名测试在没改坏的副本上的基线（基线键 = 判法单元、测试 fn 单元指纹、点名测试所在的包与测试目标、开关与特性、工具链、依赖）
    coverage/<覆盖号>     覆盖记录（覆盖号 = 判法单元指纹、测试 fn 单元指纹、点名测试所在的包与测试目标、开关与特性、工具链）：那条测试执行过的单元及各自的指纹，不带用例全名与快照号；
                          两个测试目标里字面相同的测试，基线与覆盖不互相顶替（弄坏开关 baseline-key-without-target：不带包与目标）；
                          判法单元变了（含 lib_function_units.py 切单元的改动）覆盖号跟着变，旧记录不再被引用，下一趟重记
    compositions/、units/  组成清单（判法单元、每份点名测试的闭包、工具链行、依赖版本行）与单元表：作废时照它报变了哪几个单元
    index/<变异名与文件>  这一行上一次的键与几样组成的号：只用来说清现判的原因，不当复用的依据
  服务没有删的操作：库不按天数、个数、版本回收。记录里不写主机名。
  只给自证换的两个变量只在临时 prune-server 底下认（连哪一处与临时服务的标记都指同一份临时服务，判法同客户端的 selftest_service_problem；refuse_overrides_outside_service）：
    GATE_MUTATION_RUN_RECORDS_DIRECTORY=<目录>（运行记录的共享根取它）、GATE_MUTATION_MULTI_HOST_CONFIG=<文件>（真跑测试的格交给测试的多机配置）。
    不在临时服务底下设了任一个，点名设了的几个、一条都不跑（run_stage 退 1、library-directory 退 2）。
    弄坏开关 run-overrides-honored-outside-service 下这两个不在临时服务底下也照旧认。
  运行记录不进库（run_records_directory）：用时表 timings/<这一行六段原样的 sha256>（上一次现跑的用时，派活次序用；另一台的快慢不代表本机）
  与占盘记录 target-occupancy.tsv（文件头「磁盘余量」）放在共享根下的 crates-mutation-run-records<库的命名空间>/：共享根是多机配置
  （SINGLEFS_MULTI_HOST_CONFIG 指的那一份，没设取本模块所在仓的 multi-host.env，同 scripts/gate/admission.py 的 multi_host_configuration_file）里
  NFS_SHARED_ROOTS 点名的、不含本仓也不在本仓之内的几段里排在最前的一段（两台同一绝对路径，不经 ~ 取）；配置里没有这一项就整道退 2，不补缺省、不落到 git common-dir。
  GATE_MUTATION_RUN_RECORDS_DIRECTORY=<目录> 设了（只在临时服务底下认，见上），共享根取它。

复用判据（row_decision，按次序）：
  拒算            这一行算不出键，或与表里别的行撞键（见「行键」）：照跑，不复用
  复用            库里有这把行键的记录，判的是抓到
  上次没判成抓到  有这把键的记录，判的是别的档（没红、无效、基线不绿、本次未跑……只有抓到的复用）
  记红、撤绿：判出的每一档都照行键写 records/（judge_row），没红、无效这些不是抓到的盖掉同一把行键原有的抓到（弄坏开关 red-not-recorded：只记抓到）；
  不算数的结局（本次未跑：VERDICTS_UNCOUNTED）不盖同一把行键原有的算数结论
  （抓到、没红、无效：VERDICTS_COUNTED），那一条不写、原有的留着；库里没有这把键的算数结论时照写（uncounted_verdict_keeps_counted_record；
  弄坏开关 uncounted-overwrites-counted：不算数的照写、盖掉原有的算数结论）。
  基线不绿的行另照它的行键记一条 baseline-not-ok、它的基线键记 not-ok（judge_batch；弄坏开关 baseline-red-keeps-caught：行键那一条不记），下一趟这几行现判、先跑基线。
  键对不上的那一行照索引里它上一次的那一份逐样比，栏报头一样变了的：判法单元变了、被改坏的单元变了、点名测试的闭包变了、原文或替换文变了、
  工具链或依赖变了（索引里没有这一行的报「行新加」），细节里列变了的单元（改了 / 新进 / 出了 <文件>::<限定名>）。
  前提：用例对同一份源码执行的函数集合是确定的（随机源、线程、时序只改函数里走哪条分支）。
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
    ①–③ 同变异一样直接起，不设上限与限时；有一步不成这一条测试不记（下一趟它的基线只按闭包复用），成功句另报「覆盖没记成 N 份」，不判红。
  llvm 工具：GATE_MUTATION_LLVM_TOOLS=<目录> 设了用它（双机分片的驱动设了就带到第二台），不设取 rustc --print sysroot 下 lib/rustlib/<host>/bin
    （rustup 组件 llvm-tools）；两样都要在、--version 的 LLVM 版本要以 rustc -vV 的「LLVM version」起头，不然这一道判红、一条都不跑，不退回不记覆盖。
  用时表：运行记录目录（文件头「判定库」末段，run_records_directory）里 timings/<这一行六段原样的 sha256>，记上一次现跑的用时；不进判定库。
  用时：从起 cargo 到它返回的秒数（seconds_running）。

上限、限时与卡住（records/2026-10-30-编译绕过内存总上限.md：工具里不限时，整条照旧交服务）：每条 cargo 直接起，不套内存包装、不设上限、不限时；
  一整条门禁交给统一任务服务（task-server/），内存上限由服务按记录给、撞了放大重排，死循环的变异由服务停整条任务（用时远超记录而日志又满 10 分钟不涨就停掉、打回）；
  不判「内存撞顶」「超时」「显存不够」：显存不够而红的照抓到记。

基线（派变异之前，按批在没改坏的副本上跑一遍点名的测试）：
  批的键 = 这一行 cargo 参数里 `--` 之前那一段 + `--` 之后以 `--` 起头的开关；过滤词取这一批要跑基线的各行 `--` 之后非开关词的并集，
  有一行没有过滤词就整批不带过滤词。要单独带值的开关（`--skip 名字`、`--test-threads 4`）写成 `--test-threads=4`：分开写时那个值会被当成过滤词。
  与变异一样直接起 cargo，不设上限与限时。
  一行的基线结局不随这一趟拼进了哪几行而变（--item 只选几行、整张表一批几十行，同一行判同一个结论）：每行只看它自己的过滤词（连同 --exact、--skip=）选得中的结果行
  （row_baseline_output，选法照 libtest：带 --exact 名字逐字相同、不带是子串）。弄坏开关 baseline-batch-composition：照整批的输出判。
  每一行的点名测试在基线输出里要恰好对上一个用例、结果都是 ok，否则这一行记「基线不绿」：
  不算抓到、不算没红、它的变异不跑，整道判红。已有作数的抓到记录或基线 ok 记录的行不再跑基线。

本次未跑：点名的测试在本机没跑到被改坏的那一处时，打一行 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>` 后返回成功（GPU 对拍用例在没卡的机器上）。
  变异那一趟的 libtest 参数里补 --show-output（with_shown_output，插在第一个 `--` 之后）：成功用例的输出 libtest 默认不交出来；
  不用 --nocapture：单线程跑时它把用例的输出插进「test <名字> ... 」与结果之间（隔开的结果行由 joined_result_lines 接回，但整段输出会长得多）。基线那一批不补。
  点名的测试结果是 ok、输出里有这一族本次未跑行的，这一条记「本次未跑」（not_run_reason）：不算抓到、不算没红，逐条打出来；
  test= 对上点名的测试（只比 :: 分开的末一段）的那一行先取它的 reason=；都对不上的照样记本次未跑（崩溃枚举用例判段不跑时打的是
  test=crash_state_enumeration::<项名>，项名不是测试名），那一句写明 test= 是什么、对不上测试名：宁可记本次未跑，不记没红
  （弄坏开关 not-run-test-name-only：只认 test= 对得上的）；
  整道不因它判红，成功句写明抓到几条、本次未跑几条；一条抓到的都没有、只有本次未跑的退 77（选行模式照旧退 3 / 1）。
  GATE_MUTATION_NOT_RUN_RECORD=<文件> 设了，每条本次未跑往里追加一行（阶段把它转给 gate.sh 的 GATE_NOT_RUN_FILE）；写不进去整道判红。
  只有「抓到」复用，本次未跑的下一趟照跑。
  带 gpu 特性的格另有一种记本次未跑（给定状态全交 CPU 判），见下面「真跑测试的格把多机配置交给测试、带 gpu 特性的格跑完核真用了卡」。

带 gpu 特性的格不另套清场（.claude/rules/verification.md「凡用 GPU 先清场，跑完不复原」）：
  清场与分卡归统一任务服务：服务认这一道为要卡的入口（task-server/task_server.py 的 mutation_gate_uses_gpu：变异表里有上卡的行就算），在最外层给整条门禁套一次清场、分卡，
  GPU_CARDS_BY_PRIORITY 经环境到测试。这一道起的每一次 cargo 都直接起，不调清场包装（任务里面再调清场包装由包装自己拒）；双机分片在第二台起的那一份同样。
  上卡的特性（GPU_TEST_FEATURES：验证包的 gpu，与产品 GPU 包 singlefs-background-gpu 真显卡对拍的 tests-on-a-real-gpu；
  --features / -F 里写这些名字或 <包>/<名字>，或 --all-features）照旧决定真跑测试的那几次收尾补 --test-threads、跑完核用没用上卡（gpu_test_run）。

真跑测试的格把多机配置交给测试、带 gpu 特性的格跑完核真用了卡（test_runs_multi_host_configuration、gpu_test_run、gpu_card_unused_reason）：
  工作进程在源码副本里起 cargo，副本不是 git 工作树，测试照 git common-dir 找不到多机配置，crates 那侧找不到就直接报错（不再当两个开关都关）。
  所以真跑测试的每一次（基线的批、变异、插桩跑，带不带 gpu 特性都算；只编不跑的 --no-run 不算）的环境里设 SINGLEFS_MULTI_HOST_CONFIG=<多机配置的绝对路径>。
  多机配置在哪：调用方设了 SINGLEFS_MULTI_HOST_CONFIG 照它（相对路径按本进程的当前目录取成绝对路径）；没设取本模块所在仓的
  ${GATE_MUTATION_MULTI_HOST_CONFIG:-<git common-dir 的上一层>/multi-host.env}（取不到 common-dir 退回本模块所在仓的根，同 scripts/gate/admission.py 的
  multi_host_configuration_file；GATE_MUTATION_MULTI_HOST_CONFIG 是自证换它用的，只在临时服务底下认）。那份文件不在的，run_stage 起任何 cargo 之前就拒、退 2、一条都不跑（配置错，不当本次未跑）。
  变异那一条判出「没红」（verdict_of_run 交 failure）、点名的测试恰好对上一个用例而且都是 ok 的，带 gpu 特性的再看这一条用没用上卡，用不上记「本次未跑」（同文件头「本次未跑」：
  不算抓到、不算没红，逐条打出来，整道不因它判红）：
    点名的测试那一段输出（libtest 的「---- <全名> stdout ----」；输出里一段都没有时看整份）里有 CRASH_POINT_REPLAY_GIVEN_STATES 行，
       而每一行的 states_judged_by_the_gpu_here 都是 0：给定状态全交 CPU 判，GPU 一个状态都没判。
  点名的测试打了 FAILED 的照记抓到（.claude/rules/mutation-sampling.md「改了一个格式常量之后」一节的次序：FAILED 先于本次未跑），不再看用没用卡。
  不打 CRASH_POINT_REPLAY_GIVEN_STATES 行的点名测试（GPU 对拍、取卡、卡池这些不经给定状态判的），不另判用没用卡、照常判：
    要卡的用例都经 crate singlefs-verification 的 gpu_unit_checks::gpu_test_start 取卡（用例 every_gpu_test_takes_its_cards_through_gpu_test_start 判这一条），
    它不读多机配置的 GPU 开关：卡起来了就在卡上跑，本机没有 NVIDIA 卡打 NOT_RUN_NO_GPU_ADAPTER（上面「本次未跑」那一路已经认），
    有卡而一张都起不来当场 panic（测试红），没有悄悄改交 CPU 判的一支；不要卡的（源码扫描、假设备的账本单测）判的东西跟卡无关。
  弄坏开关 test-runs-configuration-unpassed、gpu-unused-as-missed、given-states-count-ignored（见下）。

开 GPU 时崩溃判段那几行补 gpu 特性（gpu_route_rows，plan_rows 里；配 GPU 必用 GPU，不降级）：
  多机配置（上一段的找法，test_runs_multi_host_configuration）写了 ENABLE_GPU=1 时，点名测试的单元闭包够得着 GPU_ROUTE_UNITS 里一处选路函数
  （crate singlefs-verification 的 crash_state_enumeration::crash_point_replay_judge_route 与 crash_amplification::judge_not_run_without_the_gpu_feature：
  配了 GPU 而二进制没编进 gpu 就判段不跑、只录就返回 ok）、cargo 参数里没开 gpu（也没 --all-features）的行，这一趟的 cargo 参数补 --features gpu
  （-p 点名了那个包写 gpu，没点名写 singlefs-verification/gpu；插在第一个 `--` 之前）。之后这一行起的每一次 cargo 都照补了之后的参数：
  真跑测试的那几次照上面一段交多机配置；键照补了之后的特性算（闭包按这一组特性收、特性进键），配置换了开关键跟着换。
  变异表里那一行原样（Row.fields 不动，用时表与索引照它取名字）；stderr 一行点名补了哪几行。闭包算不出（拒算）的行判不了够不够得着，照原样跑；
  补了之后点名的测试不编的（#[cfg(not(feature = "gpu"))] 的用例，判的就是没编 gpu 的二进制）照原样跑，stderr 逐行点名。
  弄坏开关 gpu-route-feature-unadded：不补。

点名测试够不着被改坏的函数（static-check 可达性，reachability_of_rows；空转代码要全部优化，跑不到的代码没有存在的意义，
  records/2026-11-02-一百七十六节欠账.md）：静态判，不编、不跑、不插桩（覆盖判不了：GPU 路上排计划时对基镜像恢复一次，被改坏的那几行执行过、结局不看）。
  每一行照门禁实际给它用的那一组参数与多机配置判：
  · 参数：多机配置（test_runs_multi_host_configuration 的找法）写了 ENABLE_GPU=1、点名测试的单元闭包够得着 GPU_ROUTE_UNITS 的，照 gpu_route_rows 补 gpu 特性
    （同一份 gpu_route_feature_word_of），闭包照补了之后的参数算；补了就不编的照原样参数算（门禁照原样跑，判段不跑，CPU 判器同样不跑）。
  · closure：被改坏的单元（UnitKeys.mutated_units，按种类、文件、限定名认）一个都不在这一组参数下点名测试的单元闭包里：开不开 GPU 都调不到。
    点名测试读被改坏那份文件的源码文本的（闭包里有那份文件的 data-file 单元，或罩住它的 data-directory 单元：include_str! 与运行期读文件、读目录），
    算够得着、不判这一行（gpu-route 也不判：读到的文本看得见改动，与走哪条判段无关；source_text_reader）。按整份文件算，偏宽：测试只扫文件的一段时，
    改在别的段也算够得着。
  · 闭包拒算（算不出键：runtime-path 读真仓的测试、current-exe 起自己的子进程这一类，ClosureInfo.key 是 None）：行键那一路照旧拒算、不打键行；可达性照已收的成员
    同一套判：被改坏的单元（或读到那份源码文本的数据单元）落在已收部分里算够得着，不报；没落进去的报 REACHABILITY_UNJUDGED，原因句写「闭包拒算、已收部分里没有被改坏的单元」
    （没收进来的那一段可能调得到它，所以不报够不着）。开 GPU 时截过的闭包同理：已收部分里有就照截过的闭包判，截过的已收部分里没有的同样报没判，不报 gpu-route。豁免照旧。
  · gpu-route：多机配置开了 GPU、闭包够得着 GPU_ROUTE_UNITS 的行，从同一个测试单元、同一组 cfg 与特性另收一份截过的闭包（lib_function_units.py 的
    rust_closure_of 的 stop_at，它文件头 Rust 闭包「到这些单元就停」），截在 REACHABILITY_GPU_ROUTE_STOPS：崩溃枚举与崩溃放量判段 CPU 那一臂连到 CPU 判块的
    那一条边（按边停：测试或别的函数直接调 check_blocks_where_until 一族照连），给定状态引擎的 judged_on_the_cpu_here 与排计划时那一处宽松的基镜像恢复（按单元停）。
    被改坏的单元在全闭包里、一个都不在截过的闭包里的判。两路共用的代码（观察者、搭建、建 GPU 判表、取样那一支直接调的 CPU 判器）在截过的闭包里照留。
  豁免（截过的闭包里认得出就不判 gpu-route，打 REACHABILITY_EXEMPT 行）：
  · cpu-configuration：测试自己写一份关着 GPU 的多机配置交给引擎（cpu_configuration_writer）。认法：截过的闭包里有测试代码（集成测试目标里的单元、带 #[test]
    或 cfg(test) 的）的串字面量含 ENABLE_GPU=0，或同一个单元里既有空串字面量、又有 multi-host 起头 .env 收尾的串（写一份空配置，就是单机 CPU）。
  · gpu-tables-refused：GPU 许建不出判表、回到 CPU 判器的镜像：截过的闭包里有 REACHABILITY_CPU_FALLBACK_ROOTS 里的函数（cpu_fallback_root_in）。
  起跑先核 GPU_ROUTE_UNITS、REACHABILITY_GPU_ROUTE_STOPS、REACHABILITY_CPU_FALLBACK_ROOTS 里每个函数在 <根> 的源码里都在（reachability_names_missing），有的不在报配置错、退 2；
  一个都不在的（门禁样本那种小根，没有崩溃放量那一族函数）不是配置错：打 REACHABILITY_NOT_JUDGED 一行写原因，一行都不判，退出码不受它影响（reachability_of_rows 抛 ReachabilityNotJudged）。
  static-check 一律判可达性（没有开关参数）；门禁 code-source-discipline 的 mutation-tables 格与 scripts/gate/apply-writer-patch.py 的变异锚点静态核都经它。
  判不了的行逐行打 REACHABILITY_UNJUDGED：锚点失效、--manifest-path 写坏、被改坏的单元或点名测试拒算、闭包算不成、闭包拒算而已收部分里没有被改坏的单元
  （已收部分里有的照判，不在此列）、闭包拒算而截过的闭包已收部分里没有被改坏的单元、截过的闭包算不成。
  已知会漏：闭包同名就连、偏宽，被改坏的函数经同名的别处连进截过的闭包就不判；GPU 路上将来再出一处调 CPU 判器而结局不看的，要补进截断表。
  点名测试的闭包照 KeyContext 那一份算法并行算（key_workers），截过的闭包另开同样多的 fork 子进程并行算（reachability_cut_answers）；截过的闭包不算键、不进判定库。
  弄坏开关 reachability-stops-ignored、reachability-edges-as-units、reachability-closure-unchecked、reachability-cpu-configuration-ignored、
  reachability-fallback-roots-ignored、reachability-stops-unverified、reachability-source-text-ignored、reachability-refused-unjudged（弄坏开关那一节）。

带 gpu 特性的格测试参数收尾带 --test-threads（with_gpu_test_threads，run_cargo_command 起命令前调；.claude/rules/verification.md 里 GPU 那一条：
  单个 GPU 测试二进制只用本机点名的卡，带 --test-threads 1 跑；GPU_TEST_THREADS_RELEASED_TARGETS 里的测试目标放开）：
  这一次起的 cargo test 是 gpu_test_run 认的那一种（`--` 之前开了上卡的特性、真跑测试）时，libtest 参数收尾带 --test-threads：
    · 基线的批、变异、记覆盖的插桩跑同样照办（都经 run_cargo_command）；只编不跑的几次（--no-run）与不带 gpu 特性的不带；
    · cargo 参数点名的测试目标（--test <名>，可以几个）全在 scripts/gate/admission.py 的 GPU_TEST_THREADS_RELEASED_TARGETS 里（直接取它，本模块不抄一份）的，
      带每台默认重活线程数（scripts/hosts/heavy_threads.py 的 default_heavy_threads：SINGLEFS_THREAD_CAP 设了照它，否则 .claude/heavy-thread-default）；
      别的（--lib、没点名目标、有一个目标不在名单里）带 1；登记取不出的打一行说明、照旧带 1；
    · 行里 `--` 之后自己已写 --test-threads（--test-threads=N 或分开写）的原样不动、不重复补，行写几就是几（行的作者定了，不按名单改写）；
      与 --exact 并存时各只出现一次（--test-threads 补在末尾，在点名用例之后）；没有 `--` 的参数先补 `--`。
  线程参数不进行键（文件头「行键」那一份）：行里自己写的 --test-threads=N 是 libtest 开关、照 lib_mutation_units.py 的 parse_cargo_arguments 进键（libtest:--test-threads=N）；
  这里补的不改行的六段，也不进键——它由键里已有的两样（带不带 gpu 特性、测试目标）加一个每台的常数（登记的线程数）决定，没有新的输入；线程数只管用例并行多少、
  放开的目标按显存额度排队，与 -j / --jobs 同属不改判定的调度参数；进键则登记值一改（每台默认重活线程数是登记的值），所有带 gpu 的行已入库的判定无故作废、全部现判。
  弄坏开关 gpu-test-threads-unadded：不补，带 gpu 特性的格多个 GPU 对拍用例并行抢显存。

重测试目标只许 release（release_forced_row、release_unwritten_rows；.claude/rules/test-script-discipline.md「编译与算法都要快」第一句：穷举、长时间随机、真设备这类用例用优化构建跑）：
  判法只在 scripts/mutation/lib_release_only_tests.py（它文件头写全了怎么算命中、怎么算已带 release），这里不另写：命中 = cargo 参数选中了登记的崩溃枚举用例目标
  （.claude/gate.d/stage-inputs.tsv 的 crash-case: 行 test= 点到的包与测试目标），或特性里带 gpu（gpu、<包>/gpu、tests-on-a-real-gpu、--all-features）。
  登记表读本模块所在那一份仓的（RELEASE_REGISTRY_ROOT，同拷贝范围：判别力样本的目录里没有登记表；自证换成自己造的一份）。
  · 门禁（plan_rows：run_stage、plan、row-plan、--list-items 同一份）：命中而没带 release 的行，这一趟的 cargo 参数补 --release（插在第一个 `--` 之前；
    行里已带 --release 或 --profile release 的不重复加）；之后这一行起的每一次 cargo（基线的批、变异的先编与跑、插桩编译与插桩跑、先编一份依赖）都照补了之后的参数。
    开 GPU 时补了 gpu 特性的行（gpu_route_rows）补完特性再照判一次。变异表那一行原样（Row.fields 不动，用时表与索引照它取名字）；
    stderr 一行报补了几行，点名前 REASONS_SHOWN 行与依据。
  · 档进键：--release 是 cargo 参数里的开关，照 lib_mutation_units.py 的 parse_cargo_arguments 进行键的「开关与特性」，闭包照 release 档的 cfg 收（不带 debug_assertions）；
    release 档的调试信息照「调试信息」那一段关、取值进工具链行。补了之后的键与行里自己写 --release 的键相同，与不带 release 的键不同：头一趟这几行现判，不迁移旧键。
  · static-check：命中而第五段没写 release 的行逐行打 RELEASE（文件头 static-check 那一段），退 1；crates-mutation-replay 派活之前不判（那边照上一条补 --release 起）。
  登记表不在、读不了、crash-case: 行写法认不出：run_stage 一条都不跑（退 1），plan、row-plan、--list-items 照算不出键拒，static-check 打 ERROR 退 2；不当不命中。
  弄坏开关 release-unforced、release-unchecked（弄坏开关那一节末尾「重测试目标只许 release」那两行）。


弄坏开关（GATE_MUTATION_BREAK，只给证红用，逗号分隔可给几个；CRATES_MUTATION_ROWS_BREAK 是同义名，直接起本模块自证时用，两个变量都认）：
  nobaseline           跳过基线：点名的测试在没改坏的副本上本来就红的行照样记抓到（crates-mutation-replay 红样本、本模块自证都必须判错）
  timingsignored       派活不看用时表（自证「上一次跑得久的先派」那一格必须判错）
  workers-by-processor-half  没给 GATE_MUTATION_WORKERS 时工作进程数取核数一半、至多 16，不取每台默认重活线程数、取不到也不拒
                       （本模块自证「没给 GATE_MUTATION_WORKERS：工作进程数取每台默认重活线程数…」「…取不到每台默认重活线程数（SINGLEFS_THREAD_CAP=abc）…」两格必须判错）
  workers-beyond-rows  工作进程数不收「这一趟要现跑的条数」那一项（文件头「工作进程数」），行少而线程数、上限大时开得比要判的行多
                       （本模块自证「工作进程数取 GATE_MUTATION_WORKERS 与要现跑的条数的小者，没给内存项时头一行不提内存（20 条开 16 个、5 条开 5 个）」必须判错）
  memory-cap-ignored   开几个工作进程不看内存上限（TASK_SERVER_MEMORY_CAP_BYTES、自己 scope 的 memory.max），照线程数开满
                       （本模块自证 selftest_memory_cells 的十六格全部必须判错，头一格是「服务交的内存上限 20 GiB、每个工作进程的峰值 4 GiB（编译并行度 4 下的实测）…」；
                       selftest_worker_peak_cells 里经 memory_bound_for 的七格也判错：「固定编译并行度：…」「编译并行度跟着进程数走：…」「从没量过：…」
                       「只有更低编译并行度的实测：…」「峰值记录里有一份内容不是正整数：…」「峰值记录的 worker-peaks 是一份文件、不是目录：…」「交给 run_stage：…开 2 个工作进程…」）
  memory-cap-short-run-anyway  内存上限装不下一个工作进程的峰值时开 1 个硬跑，不按撞内存上限退 250
                       （本模块自证 selftest_memory_cells 里「内存上限装不下一个工作进程的峰值：run_stage 按撞内存上限退 250…」那一格必须判错，别的格不红）
  memory-max-not-hard-cap  内存上限取到服务交的变量或多机配置的值就不看自己 scope 的 memory.max：scope 真实上限只有 20 GiB，多机配置写 50G 就按 50G 开
                       （本模块自证 selftest_memory_cells 里「多机配置 50G、scope 的 memory.max 20 GiB」「服务交的变量 50 GiB、scope 的 memory.max 20 GiB」两格与
                       「run_stage 整趟：服务交的变量 50 GiB、scope 的 memory.max 8 GiB」一格必须判错，别的格不红）
  records-written-at-end  判定库记录（基线、变异）先攒在工作进程内存里，进程池正常收尾时才一并写（被怀疑过的样子：整组被杀、判过的格不留下），init_worker 换上 DeferredWriteLibrary
                       （本模块自证 selftest_killed_run_resume_cell「一趟跑到一半整组硬杀，重跑同一条命令」那一格必须判错，别的格不红）
  killed-run-rerun-keyed-by-other-code  只在自证 selftest_killed_run_resume_cell 里认：重跑那一趟的子进程另加 judge-units-whole-file，判法单元指纹换一种算法
                       （两趟算键的判法代码不同，判完的那几条一条都复用不上；那一格必须判错，别的格不红）
  worker-peaks-unkeyed-by-jobs  读每个工作进程的峰值记录时不分编译并行度，各编译并行度都取全部记录里最大的（量与除不认编译并行度的样子）
                       （本模块自证「记下的峰值按编译并行度分开、读得回来：…」「固定编译并行度：…」「编译并行度跟着进程数走：…」三格必须判错）
  worker-peaks-window-unbounded  读峰值记录时每个编译并行度取全部记录的最大、不只看最新的 WORKER_PEAKS_RECENT 份，也不删更旧的
                       （本模块自证「每个编译并行度只看最新的 256 份：…」那一格必须判错）
  jobs-feedback-ignored  试每个进程数时编译并行度都照天花板那个进程数算，不照这个进程数算：开的进程少了、编译并行度跟着变大，峰值没跟着算
                       （本模块自证「编译并行度跟着进程数走：…」那一格必须判错）
  unmeasured-worker-peak-guessed  某个编译并行度与更高的都没有实测时，每个工作进程的峰值照写死的默认 3G，不取保守值
                       （本模块自证「从没量过：…」「只有更低编译并行度的实测：…」两格必须判错）
  unreadable-worker-peaks-guessed  峰值记录读不了时每个工作进程的峰值照写死的默认 3G，不取保守值
                       （本模块自证「峰值记录里有一份内容不是正整数：…」「峰值记录的 worker-peaks 是一份文件、不是目录：…」两格必须判错）
  worker-tree-self-only  量一个工作进程的用量时只算它自己，不算它起的整棵进程树
                       （本模块自证「量一棵进程树：…」那一格必须判错）
  worker-peaks-unrecorded  工作进程起来时不起采样线程，一份峰值记录都不写
                       （本模块自证「交给 run_stage 的工作进程各写一份峰值记录：…」那一格必须判错）
  sample-root-peaks-recorded  被判的不是本仓（门禁样本根）、也不在临时 prune-server 底下时，工作进程照样写峰值记录（样本 crate 的小读数掺进本仓那一份的样子）
                       （本模块自证「样本根上的工作进程不写峰值记录：…」那一格必须判错）
  multiple-packages-last-only  cargo 参数里有多个 -p 只认最后一个（lib_mutation_units.py 的 parse_cargo_arguments）
                       （本模块自证「多个 -p：…」那一格（selftest_multiple_packages_cell 调 lib_mutation_units.py 的 multiple_packages_cell）必须判错）
  rowplanselectionignored  row-plan 不看 --rows，照整张表算键（mutation-shard-run.sh --selftest 里 row-plan 点名两行那一格设它起 row-plan，必须判错）
  rowkeywithouttoolchain  行键不带工具链（自证「工具链变了：每一行都现判」那一格必须判错）
  mutated-unit-ignored  行键不带被改坏的单元指纹（自证「改了被改坏、点名测试够不着的函数：只现判那一行」那一格必须判错）
  row-key-collision-unrefused  表里两行算出同一把行键时不判撞键，两行共用一把键、互相复用
                       （本模块自证「撞键拒算：left.rs 与 right.rs 里内容相同的函数各改一处…」那一格必须判错）
  not-run-test-name-only  本次未跑只认 test= 对得上点名的测试名（只比 :: 末段）的那一行：test=crash_state_enumeration::<项名> 记成没红
                       （本模块自证「本次未跑认项名：…」那一格必须判错）
  gpu-route-feature-unadded  多机配置开了 GPU 时，点名测试走崩溃判段、不带 gpu 特性的行不补 gpu 特性：判段不跑、只录就返回 ok
                       （本模块自证「开 GPU 时点名测试走崩溃判段的那一行补 gpu 特性：…」那一格必须判错）
  gpu-test-threads-unadded  带 gpu 特性、真跑测试的 cargo test 的 libtest 参数收尾不带 --test-threads
  reachability-stops-ignored  static-check 可达性截闭包时不截（stop_at 给空）
                       （本模块自证「static-check 可达性的 gpu-route：…」那一格必须判错）
  reachability-edges-as-units  static-check 可达性截断表里按边停的也按单元停（测试直接调的 CPU 判器入口也截掉）
                       （本模块自证「static-check 可达性放行够得着的：…」那一格必须判错）
  reachability-closure-unchecked  static-check 可达性不判 closure 那一路（被改坏的单元根本不在点名测试的闭包里也不报）
                       （本模块自证「static-check 可达性的 closure：…」那一格必须判错）
  reachability-cpu-configuration-ignored  static-check 可达性不认测试自己写的 CPU 配置
                       （本模块自证「static-check 可达性的 cpu-configuration 豁免：…」那一格必须判错）
  reachability-fallback-roots-ignored  static-check 可达性不认 GPU 许建不出判表、回到 CPU 判器的那个函数
                       （本模块自证「static-check 可达性的 gpu-tables-refused 豁免：…」那一格必须判错）
  reachability-stops-unverified  static-check 可达性起跑不核选路表、截断表、豁免表里的函数在不在源码里
                       （本模块自证「static-check 可达性起跑先核截断表：…」那一格必须判错）
  reachability-source-text-ignored  static-check 可达性不认读源码文本的点名测试：只拿函数单元比，闭包里有那份文件的数据单元照报 closure
                       （本模块自证「static-check 可达性放行读源码文本的：…」那一格必须判错）
  reachability-refused-unjudged  static-check 可达性遇到闭包拒算的行一律报没判，不看已收的成员
                       （本模块自证「static-check 可达性的闭包拒算：点名测试的闭包拒算…」那一格必须判错；同一开关下后面三格也红）
                       （本模块自证「带 gpu 的行收尾带 --test-threads 1、放开名单里的目标带登记值、不带 gpu 的行不带…」那一格必须判错）
  baseline-key-without-target  基线键与覆盖号不带点名测试所在的包与测试目标
                       （本模块自证「基线键与覆盖号带包与测试目标：…两行的基线键、覆盖号都不同」那一格必须判错）
  baseline-batch-composition  一批基线照整批的输出判每一行：同一行单跑与整张表判出不同的基线结局
                       （本模块自证「一行的基线结局不随这一批拼进了哪几行而变：…」那一格必须判错）
  judging-entries-verdict-only  判法单元只从结局分类那几个入口收闭包，judge_row、judge_batch、run_cargo_command 这几段不进
                       （本模块自证「判法单元收全：改判法拷贝里 judge_row 的一处，每一行都现判（栏是判法单元变了）」那一格必须判错）
  judging-entries-with-temporary-service  判法单元另从 mutation_run_prune_server 收闭包：run_cargo_command 直接调它，闭包经它载入的
                       scripts/prune-server/prune_server_client_selftest.py 连到自证起的 admission.py 与 lib_function_units.py 自证里折不出命令的 subprocess
                       （本模块自证「判法单元算得出（闭包里没有拒算）」那一格必须判错）
  red-not-recorded     judge_row 只记「抓到」，没红、无效这些不写，同一把行键原有的「抓到」留着照复用
                       （本模块自证「记红撤绿：先抓到入库，同一把键改判没红之后，下一趟这几行现判…」那一格必须判错）
  uncounted-overwrites-counted  不算数的结局（VERDICTS_UNCOUNTED）照行键写、盖掉同一把行键原有的抓到、没红、无效
                       （本模块自证「不算数的结局不盖算数的：库里一行抓到，同一把键经哨兵强制现判出本次未跑之后…」那一格必须判错）
  baseline-red-keeps-caught  基线不绿的行不照行键记「基线不绿」，同一把行键原有的「抓到」留着照复用
                       （本模块自证「记红撤绿：先抓到入库，同一把键判一趟基线不绿之后，下一趟这几行现判…」那一格必须判错）
  library-outside-git-tolerated  被判的树取不到 git common-dir 时拿被判的根当库根往下跑（没处放也照跑）
                       （本模块自证「库根：被判的树不是 git 工作树：退 1、点名它、一条都不跑」那一格必须判错）
  run-overrides-honored-outside-service  不在临时 prune-server 底下设了 GATE_MUTATION_RUN_RECORDS_DIRECTORY、GATE_MUTATION_MULTI_HOST_CONFIG 也照旧认
                       （本模块自证「不在临时 prune-server 底下设了只给自证换的运行记录、多机配置两个变量：…」那一格必须判错）
  mutation-run-on-real-service  起 cargo test 真跑测试的那一次不起自己的临时 prune-server，照调用方的环境连那一份服务：变异版判出的崩溃结论写进真库
                       （本模块自证「变异复跑各对自己的临时 prune-server 跑：…」那一格必须判错）
  test-closure-ignored  行键不带点名测试的闭包键（自证「改了点名测试调到、不是被改坏的函数：点名它的那一行现判」
                       「改了被改坏函数用的常量（V5 那一类）：原文在那个函数里的两行现判」「改了点名测试够得着、没执行到的函数：那一行现判」三格必须判错）
  baseline-by-tree-snapshot  基线 ok 只按整树快照复用（自证「加一个没人调的函数、同时改一行的替换文：只那一行现判、它的基线复用」
                       「改了点名测试够得着、没执行到的函数：那一行现判，基线按覆盖复用」两格必须判错）
  judge-units-whole-file  判法单元指纹取闭包碰到的那几份文件的整份原文（自证「只改判法脚本的注释：判法单元指纹不变」那一格必须判错）
  judging-function-units-excluded  判法单元指纹不收 lib_function_units.py 的单元（D13 已定项 17 文件级排除用在 crates 变异判法上的样子）
                       （本模块自证「改 lib_function_units.py 的 RustFile.row 一处函数体：判法单元指纹变，闭包里有 lib_function_units.py 的单元」那一格必须判错）
  coverage-id-without-judging  覆盖号不带判法单元指纹
                       （本模块自证「覆盖号随判法单元指纹变：改 lib_function_units.py 的 RustFile.row 一处函数体，同一条点名测试的覆盖号跟着变」那一格必须判错）
  coveragenotrecorded  抓到之后不记覆盖（自证「头一趟每条点名测试记一份覆盖、同一条测试的几行共用一份」
                       「改了点名测试够得着、没执行到的函数：那一行现判，基线按覆盖复用」两格必须判错；另有记覆盖次数的四格跟着判错：
                       「头一趟起 1 批基线 + 4 条变异，另记 4 份覆盖」「下一趟只跑剩下的 2 条」「插桩编译那一步写出的剖析文件」）
  llvmversionunchecked 不核 llvm 工具与 rustc 的 LLVM 版本（自证「llvm 工具的 LLVM 版本与 rustc 对不上：判红、一条都不跑」那一格必须判错）
  buildprofilesunset   插桩编译那一步不设 LLVM_PROFILE_FILE：构建脚本与过程宏的剖析文件落进它们起的目录
                       （本模块自证「插桩编译那一步写出的剖析文件只落在这一格的临时目录」那一格必须判错）
  keepmutationtmp      每条变异的 TMPDIR 不删（crates-mutation-replay 两份样本都必须判错）
  notrunasmissed       点名的测试打了 NOT_RUN_NO_GPU_ADAPTER 也当没红记（crates-mutation-replay 的 not-run 样本、本模块自证「点名的测试打了 NOT_RUN」那一格都必须判错）
  emptyreplacementrefused  替换文空着也判不成形（crates-mutation-replay 绿样本里的删行变异、code-source-discipline 绿样本里的删行变异、本模块自证「删行变异照常判」那一格都必须判错）
  cargoconfignotcopied 拷每片源码副本时跳过拷贝范围里的 .cargo/（登记漏了它时的样子）：副本里的 cargo 读不到 .cargo/config.toml 的 [env]
                       （本模块自证「拷贝范围含 .cargo/：点名的测试要编 gpu 特性的那一行基线绿、变异抓到」那一格必须判错）
  path-dependencies-not-copied  拷贝范围不调 lib_repository_copy.py 的 copy_closure（不跟 path 依赖、不认编译期与运行期读的仓根文件），只拷登记的那几处
                       （副本里缺 prune-server/，cargo 解析工作区就失败）
                       （本模块自证「拷贝范围跟 path 依赖：…」「拷贝范围跟 path 依赖指到树之外：…」两格必须判错）
  manifest-path-ignored  不认变异行 cargo 参数里的 --manifest-path：拷贝范围不补那个包目录，单元键照 <根> 那一份工作区算，写坏了也不拒
                       （本模块自证「--manifest-path 静态核：…」「--manifest-path 基线与判红：…」「--manifest-path 单元键照那个包的源码算：…」
                       「--manifest-path 不在仓内：…」四格必须判错）
  product-gpu-feature-unquiesced  产品 GPU 包的 tests-on-a-real-gpu 不当上卡的特性认，只认 gpu：这一包的真显卡对拍行不补 --test-threads、不核用没用上卡
                       （本模块自证「产品 GPU 包（--features tests-on-a-real-gpu）的行：真跑测试的那几次各收尾带 --test-threads 1…」
                       「产品 GPU 包（--features tests-on-a-real-gpu）的行：点名的测试没卡打 NOT_RUN_NO_GPU_ADAPTER…」两格必须判错）
  test-runs-configuration-unpassed  真跑测试的格不往环境里设 SINGLEFS_MULTI_HOST_CONFIG、也不判多机配置在不在
                       （本模块自证「真跑测试的每一次在副本里起的 cargo 环境里有 SINGLEFS_MULTI_HOST_CONFIG…」
                       「多机配置不在：run_stage 退 2…」两格必须判错）
  gpu-unused-as-missed  带 gpu 特性的格判出没红时不核用没用上卡：给定状态全交 CPU 判的也记没红
                       （本模块自证「带 gpu 特性的行：给定状态全交 CPU 判…」那一格必须判错）
  given-states-count-ignored  有 CRASH_POINT_REPLAY_GIVEN_STATES 行就当没用上卡，不读 states_judged_by_the_gpu_here 的数
                       （本模块自证「带 gpu 特性的行：给定状态有 GPU 判过的…照常判」那一格必须判错）
  undefinedidentifiersunchecked  static-check 不判替换文里新引的标识符
                       （本模块自证「static-check 的 UNDEFINED：替换文引了已删的标识符 halved_before 的那一行报一行…」那一格必须判错）
  overlapunchecked     并发领片号那一格不核两次调用重叠没有、照判对错
                       （本模块自证「先起的那次在后起的那次开始之前就退了（样本）：并发领片号那一格认出两次调用没重叠、记本次未跑…」那一格必须判错）
  staticusagebare      static-check 参数不对时只打那一行用法，不写缺了什么、不给「→ 怎么办」（本模块自证「static-check 缺 <根>」那一格必须判错）
  tablemissingok       static-check --table 指的文件不在时照旧当空表判、报「CHECKED 0 …」退 0（本模块自证「static-check --table 指的文件不在」那一格必须判错）
  shardrootkeptonsignal  run_pool 不接 TERM / HUP：收到就停，$TMPDIR 下的 singlefs-mutation-replay-* 留着
                       （本模块自证「被 TERM 停：$TMPDIR 下的 singlefs-mutation-replay-* … 停完删掉，退 143」那一格必须判错）
  broken-pool-looked-up-late  「有工作进程中途死了」那一句判 except 的时候才按 concurrent.futures.process 属性找 BrokenProcessPool：进程池还没起就被停时
                       那个子模块没载入，判 except 抛 AttributeError、盖掉 StopRequested（本模块自证「进程池还没起就被 TERM 停」那一格必须判错）
  term-sent-unwaited   自证「被 TERM 停」那一格不等「临时目录在、假 cargo 在工作进程里挂住」就发 TERM（碰上忙机器：run_pool 还没装好信号处理，默认处理退 -15）
                       （本模块自证「被 TERM 停：$TMPDIR 下的 singlefs-mutation-replay-* 跑的时候在、停完删掉，退 143 …」那一格必须判错）
  shardrootnamedlate   run_pool 建 shard_root 用 tempfile.mkdtemp（目录在 mkdir 里建出来、名字要等它返回才到调用方手里：TERM 落在这两步之间，名字丢了、目录漏在 $TMPDIR）
                       （本模块自证「TERM 落在建 shard_root 的 mkdir 之后、名字交到调用方手里之前：收尾照样删掉」那一格必须判错）
  listitemspipeunhandled  --list-items 不照 SIGPIPE 惯例退（接 | head 时打 BrokenPipeError 的回溯、退 1）
                       （本模块自证「--list-items 接 | head」那一格必须判错）
  extratargetskept     开跑时不回收片号大于这一趟工作进程数的 -w<片号> 编译目录：分片各一份、只增不减
                       （本模块自证「片号大于这一趟工作进程数的 -w<片号> 连同 -coverage 开跑时回收、不大于的留着、打一行删了几份多大」那一格必须判错）
  inusetargetremoved   回收之前不核有没有进程在用（工作目录、命令行、CARGO_TARGET_DIR、在用标记），多出来的照删
                       （本模块自证「多出来的 -w<片号> 有进程在用的不删」那一格必须判错）
  targetdefaulttmp     没设 GATE_MUTATION_TARGET_DIR 时编译目录放 ${TMPDIR:-/tmp}/singlefs-crates-mutation-target（跨轮共用、跑完不删）
                       （本模块自证「跑完运行目录里不剩它建的编译目录（没设 GATE_MUTATION_TARGET_DIR：…）」与「跑完运行目录里不剩它建的编译目录（被 TERM 停）」两格必须判错）
  runtargetkept        跑完不删这一趟自己建的编译目录（连同它所在的 $TMPDIR 下的 singlefs-mutation-replay-*；编译目录跨轮留着）
                       （本模块自证「跑完运行目录里不剩它建的编译目录（没设 GATE_MUTATION_TARGET_DIR：…）」与「跑完运行目录里不剩它建的编译目录（被 TERM 停）」两格必须判错；
                       连带「被 TERM 停（双机分片的驱动停这一份的样子）：…」那一格，$TMPDIR 下的 singlefs-mutation-replay-* 留着）
  runtargetopenfilesunchecked  删这一趟的编译目录之前不核有没有进程开着里面的文件
                       （本模块自证「这一趟的编译目录里还有进程开着文件：不删、报出那个进程」那一格必须判错）
  local-shard-numbering  工作进程的片号不领锁、从本调用的共享计数器现领本地序号：两次并发调用的第 2 片都指向同一个 -w2
                       （本模块自证「同一个共用编译目录上并发的两次调用各起 2 个工作进程：领到的片号集合不相交、四把片号锁都被占」那一格必须判错）
  anchor-stops-table   锚点失效的一行让整张表退出、一个工作进程都不起
  anchor-mid-run-asserts  派活之后工作副本里原文不再恰好一次时 judge_row 抛 AssertionError、整个进程池跟着炸
                       （本模块自证「派活之后锚点失效：那一行记锚点失效，别的行照判照入库」那一格必须判错）
                       （本模块自证「锚点失效按行判：那一行记锚点失效、别的行照跑照入库」那一格必须判错）
  anchor-rows-planned  plan、row-plan、--list-items 不先剔掉锚点失效的行，照常算键：文件不是 .rs 的那一行算得出键、会被分给第二台
                       （本模块自证「row-plan、plan、--list-items 把锚点失效的行单列」那一格必须判错）
  impact-row-dropped   牵连面少算一行（算进的头一行不数）
                       （本模块自证「牵连面：样本 crate 上牵连最大的单元与行数」那一格必须判错）
  reuse-lookup-skipped 复用判据一律不复用（键相同、判过抓到照样现判）
                       （本模块自证 reuse-three-questions 那一格必须判错；reuse-sentinel 那一格要的就是这一趟复用哨兵判红「复用失灵」）
  共用库 scripts/gate/lib_reuse_checks.py 的开关 REUSE_CHECKS_BREAK=rejudged-sentinel-passes（哨兵被现判不红）：本模块自证 reuse-sentinel 那一格必须判错
  uncounted-forced-observed  强制现判那一条落在不算数的结局（VERDICTS_UNCOUNTED：本次未跑）上也当现判结论交哨兵，拿它比库里的「抓到」判成「判定不稳」
                       （本模块自证「复用哨兵：强制现判那一条本次未跑（notrun）：不当现判结论交哨兵，打一行不判、不判成不稳」那一格必须判错）
  exactduplicated      记覆盖那一趟把行里已有的 --exact 与自己补的叠成两个（libtest 报 Option 'exact' given more than once、覆盖记不成）
                       （本模块自证「行里已带 --exact 的变异…」那一格必须判错）
  item-plan-reusable-zero  --list-items --with-keys 的 ITEM_PLAN 里 reusable 一律 0（能复用的行不数）
                       （本模块自证「--list-items --with-keys：每项一行 ITEM_PLAN…」那一格必须判错）
  item-plan-selection-ignored  --list-items --with-keys 带 --rows 也列整表、算整表的键
                       （本模块自证「--list-items --with-keys --rows：只打点名那一行的 ITEM_PLAN…」那一格必须判错）
  item-done-reused-silent  只复用的项不打 ITEM_DONE
                       （本模块自证「真跑：全复用的第二趟每项一行 ITEM_DONE…」那一格必须判错）
  item-done-seconds-whole-run  ITEM_DONE 的 seconds 记自开跑起的整趟，不记这一项自己的
                       （本模块自证「真跑：每项各记各的挂钟…」那一格必须判错）
  split-result-line-unseen  结果词被测试中途打的行隔开的结果行不接回一行：认不出、基线记成「一行都没出现」
                       （本模块自证「结果词被插进来的 GPU_LIFECYCLE 行隔开时照认基线绿」那一格必须判错）
  closure-error-masked  scripts/mutation/lib_mutation_units.py 里的开关（同样从 GATE_MUTATION_BREAK 或 CRATES_MUTATION_ROWS_BREAK 认）：按路径载入的 lib_function_units.py
                       不登记进 sys.modules、算闭包的工作进程把异常原样抛（--list-items 末尾是 PicklingError，真错被盖住）
                       （本模块自证「闭包算不成（并行算…）」与「--list-items 遇到闭包算不成」两格必须判错）
  wgsl-whole-file      scripts/mutation/lib_mutation_units.py 里的开关（同样从 GATE_MUTATION_BREAK 或 CRATES_MUTATION_ROWS_BREAK 认）：.wgsl 当数据单元、
                       取整份字节的 sha256（本模块自证「WGSL 按 fn 取键」那一格必须判错）
  judging-cache-unverified  判法单元指纹的磁盘缓存不核读过的内容，有存的就一直沿用（不看内容的缓存）
                       （本模块自证「判法单元指纹的磁盘缓存」那一格必须判错）
  specification-lines-product-view  scripts/mutation/lib_mutation_units.py 里的开关（同样从 GATE_MUTATION_BREAK 或 CRATES_MUTATION_ROWS_BREAK 认）：规格行（原文落在 verus! 里产品行为视图剥掉的字上的变异行）也取产品行为视图（本模块 row_view 同样认它）
                       （本模块自证「verus! 里的规格行：原文落在产品行为视图剥掉的字上」那一格（selftest_specification_line_cell 调 lib_mutation_units.py 的 specification_line_cell）必须判错）
  rocksdb-entry-unchecked  起任何 cargo 之前不调 scripts/hosts/rocksdb-prebuilt.py entry
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
    栏写「复用哨兵强制现判」、照现判跑。哨兵被现判、强制现判那一行不是抓到，整道判红；强制现判那一行落在不算数的结局（本次未跑）上的
    不当现判结论交哨兵，哨兵打一行「没报结局，不判」（observe_fresh_outcomes）。种子与条数取环境里的 SINGLEFS_REUSE_SENTINEL_SEED、
    SINGLEFS_REUSE_SENTINEL_COUNT。自证别的格设 SINGLEFS_REUSE_SENTINEL_COUNT=1000（键相同的全当哨兵、不抽强制现判）。
按项机读行（统一任务服务按项先算后排读它；一项 = 变异名，同名的几行合成一项，--item 点名同名就是全判）：
  · --list-items --with-keys：每项一行（stdout，次序是变异表的次序，与逐行列出的项一一对应；条数之和等于末行合计的数）
      ITEM_PLAN item=<项名> to_judge=<要现判的条数：键对不上与拒算> reusable=<能复用的条数：键对得上>
    锚点失效的行两样都不算（不跑）。item= 到 to_judge= 之间原样是项名（名字里可以有空格，按行尾三段 to_judge= reusable= 从后往前切）。不起 cargo、不起服务。
  · 真跑（run_stage，ItemProgress）：每项的行全有了结局就打一行（stderr，进度类：带挂钟，不进 stdout，stdout 与进程数无关那一条不动）
      ITEM_DONE item=<项名> seconds=<这一项自己的挂钟秒数> judged=<现判条数> reused=<复用条数> exit=<这一项的退出码>
    seconds 是这一项各行在工作进程里判一行从头到尾的秒数相加（几项并行时各记各的，不记整趟；基线的批几行共用，不记进去；复用的 0）；
    exit：有一行没红、无效、基线不绿、锚点失效的 1，没有而有本次未跑的 77，其余 0（复用的行是抓到）；
    开跑时复用的与锚点失效的项立刻打，现判的项在它最后一行判完时打，被停（只供测试的开关、编译目录停派）没判到的项在收尾补一行（judged=0 reused=0 exit=1）：
    跑到的每一项恰好一行。弄坏开关 item-plan-reusable-zero（ITEM_PLAN 的 reusable 一律 0）、item-done-reused-silent（只复用的项不打 ITEM_DONE）、
    item-done-seconds-whole-run（seconds 记自开跑起的整趟）、item-plan-selection-ignored（--rows 点名也算整表）各让自证 selftest_item_lines_cells 的一格判错。
编译目录（default_shared_target_directory，阶段脚本不另算；自己用完自己删，不跨轮复用）：GATE_MUTATION_TARGET_DIR 设了用它（调用方给的，跑完不删）；
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
工作进程数（worker_count_for）：四项取最小、至少 1：每台默认重活线程数（scripts/hosts/heavy_threads.py 的 default_heavy_threads：SINGLEFS_THREAD_CAP 设了照它，
  否则 .claude/heavy-thread-default；GATE_MUTATION_WORKERS 给了顶掉它）、这一趟要现跑的条数、磁盘余量（再下一段）、内存上限 ÷ 每个工作进程的峰值（下一段「内存上限」）。
  这一趟要现跑的条数就是这一趟要判的单位数（records/2026-10-31-task-server巡检.md）：
  点到的行里没有作数的抓到记录的那几行（run_stage 的 candidates，拒算的照算），不照拿到的上限开满；基线的批（baseline_batches：要现跑基线的几行合成一批，
  批里的行都在这几行里）与覆盖（一行抓到之后补记的那一份、开跑时就要记的那几份，都是这几行的）跟着这几行走，进同一个进程池、不另算工作进程，
  所以工作进程数不超过这几行的行数（统一任务服务对登记了的这一道照「底座 + 每件峰值 × min(--item 个数, 线程数)」给上限，与这一项同一个口径：.claude/memory-bounded-parallelism.tsv）。
  弄坏开关 workers-beyond-rows 下不收这一项（工作进程数只照线程数、磁盘与内存那几项开，行少而上限大时开得比行多）。
  次序：线程数与条数的小者当天花板交给磁盘那一项，这三项的小者再当天花板交给内存那一项（内存那一项要按真开的进程数算编译并行度，天花板要含磁盘那一项）。取不到每台默认重活线程数（登记文件不在、写坏、
  SINGLEFS_THREAD_CAP 不是正整数）而又没给 GATE_MUTATION_WORKERS 的，整道判红退 2、一条都不跑，不退回按核数算（弄坏开关 workers-by-processor-half：取核数一半、至多 16）。
  不在这里设内存上限：一整条门禁的内存上限仍由统一任务服务按记录给（上面「上限、限时与卡住」），开几个工作进程照下一段按拿到的上限收。
编译并行度（cargo_jobs_for）：每个 cargo 的 CARGO_BUILD_JOBS 取 GATE_MUTATION_CARGO_JOBS（正整数，写法不对整道拒）给了照它，不然 max(1, 核数 ÷ 工作进程数)；
  内存那一项算每个进程数要的编译并行度、run_stage 设 CARGO_BUILD_JOBS 用的是同一个函数。选的是「编译并行度跟着进程数走、算到自洽」，不是固定编译并行度：
  进程数被内存卡小时，固定的编译并行度会让核空着；跟着走则每试一个进程数 n 就照 n 算出编译并行度、再取那个编译并行度下的每进程峰值，满足 n × 峰值 ≤ 上限的最大 n 才开。
内存上限（memory_bound_for）：开几个工作进程按这一条拿到的内存上限 ÷ 每个工作进程的峰值算，与另外三项取最小、至少 1（worker_count_for 里 1 个都放不下按 1 个算、头一行写「那一项不足 1」；run_stage 见到内存那一项是 0 就按撞内存上限退 250，下面「撞内存上限」）。
  内存上限的取法（task_memory_cap_bytes，次序）：① 统一任务服务交的环境变量 TASK_SERVER_MEMORY_CAP_BYTES（这一次尝试的上限，字节十进制整数；写法不对的头一行点名、改走②）；
  ② 仓根多机配置 multi-host.env 的 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项（变异的内存上限直接按仓根目录设置的 50G 走；值是 <地址>=<systemd 写法> 的若干项，
  读法用 scripts/gate/admission.py 的 configuration_value，这一台照多机配置判法 --emit-addresses 认（scripts/hosts/multi_host_addresses.py 的 resolve，整份配置判过才给地址），
  找文件同 test_runs_multi_host_configuration；文件不在、键没写、认不出这一台、读不了都往下走，头一行点名）；
  ③ 自己 scope 的 memory.max（scope_memory_max：/proc/self/cgroup 里 `0::<路径>` 那一行接 /sys/fs/cgroup 再接 /memory.max；只读自己这一层，不往上找父 cgroup）；
  ④ 三样都没有（memory.max 写 max、读不到、写法不认）这一项不收，工作进程数照线程数，头一行写明原因。
  memory.max 是有限值就是硬顶（超了整个 scope 被杀）：①② 取到的值比它大，取 memory.max（两者取小）；①② 都没有，取 memory.max 本身；memory.max 是 max 或读不到，不收窄，照①②。
  头一行写各档读到的值、取了哪一个、为什么（「各档读到：…」）。依据：scope 的真实上限（如 20736M）可以比多机配置写的 50G 小，只按 50G 开会开太多工作进程。
  弄坏开关 memory-max-not-hard-cap：①② 取到了就不看 memory.max。自证用 GATE_MUTATION_CGROUP_LISTING、GATE_MUTATION_CGROUP_ROOT 换掉 /proc/self/cgroup 与 /sys/fs/cgroup（生产不设）。
  每个工作进程的峰值（门禁自己量，不查内存包装的峰值表：各格的 cargo 是工作进程直接起的、不经包装，那张表里永远没有它们）：
  · 量什么：一个工作进程连同它起的整棵进程树（cargo、rustc、测试二进制、各格的临时 prune-server）此刻的用量，口径是每个进程 /proc/<pid>/status 的
    RssAnon + RssShmem + VmPTE 加总（process_tree_usage；子进程顺 /proc/<pid>/task/<tid>/children 往下找，读不到的刚退出、不算）。
    与 scripts/hosts/run-with-memory-cap.sh 记的「回收不掉的用量」（cgroup 的 memory.current 减 active_file、inactive_file，即 anon + shmem + 内核 + sock）比：
    anon、shmem、页表三样同；差在 ① 内核里不归哪个进程的那几样（slab、内核栈、percpu、sock）这里不算；② 没被任何进程映射着的 tmpfs 文件页这里不算；
    ③ 几个进程共享的匿名页（fork 之后没写过的）这里按进程各算一次，偏大。门禁所在 scope 里另有起这一道的父进程自己的用量，也不在任何一个工作进程的读数里。
  · 怎么量（sample_worker_peak）：每个工作进程起来时（init_worker）在自己里面起一条采样线程，每 WORKER_PEAK_SAMPLE_SECONDS 秒量一次整棵树，记这一趟到此刻的最大；
    最大涨了 WORKER_PEAK_WRITE_STEP_BYTES 以上就把它写进一份记录（同目录临时文件、改名换上），整组被硬杀时已写的留着（记下的比真峰值至多少一个步长；
    比采样间隔还短的尖峰采不到，推的）。量不了（这台内核没有 /proc/<pid>/task/<tid>/children）就不记，stderr 报一行，下一趟照「没有实测」办。
  · 记在哪（record_worker_peak、worker_peaks_directory）：运行记录目录（文件头「判定库」末段，run_records_directory：多机配置 NFS_SHARED_ROOTS 点名的共享根下，
    两台读写同一份，不经 ~ 与 $HOME 取）下 worker-peaks/<编译并行度>/<起采样的时刻纳秒 20 位>-<进程号>-<8 位随机字符>，内容是一个正整数字节数加换行；
    一个工作进程一趟一份，按编译并行度分目录。是运行记录，不进判定库。只有被判的是本仓、--staged 的临时树（GATE_STAGED_FROM），或在临时 prune-server 底下时才写
    （worker_peaks_recorded_for）：门禁样本根不写，样本 crate 的峰值不代表本仓，不在临时服务底下时它的运行记录目录就是本仓那一份。
  · 怎么读（measured_worker_peaks）：每个编译并行度只看名字排在最后（最新）的 WORKER_PEAKS_RECENT 份，取里面最大的；更旧的读的时候删掉。
    目录还没有是「从没量过」；有一样不是这一版的写法（编译并行度目录名不是正整数、记录名不对、内容不是一个正整数字节数）或读不了，整份记录读不了，
    头一行点名哪一份、为什么，出路是删掉那一份（或整个 worker-peaks/）再跑。
  · 怎么除（worker_peak_for_jobs、memory_bound_for）：天花板 c 是另外三项的小者；n 从 1 试到 c，每个 n 照 cargo_jobs_for 取编译并行度 j，每进程峰值 P(j) 取
    ① 编译并行度 j 有实测：它最新 WORKER_PEAKS_RECENT 份里的最大；② 没有，而更高的编译并行度有实测：借比 j 大的里最小的那一个的（推的：编译并行度越高，每个工作进程的峰值不会越低）；
    ③ 都没有（从没量过，或只有更低编译并行度的实测），或整份记录读不了：取保守值，就是内存上限本身——按一个工作进程占满整份上限算，只开 1 个；
      这一口径下只开 1 个，不会因为同时开了几个而撞顶（单开一个就超上限的，开几个都超）。不用写死的默认值。
    满足 n × P(j) ≤ 上限的最大 n 就是这一项（一个都不满足交 0）；n 到了天花板 c 还满足，这一项报 ⌊上限 ÷ P(j)⌋（编译并行度照 c 算），不与另外三项并列成「卡在」。
    n 从 1 到 c 每个都试：编译并行度随 n 变，借来的峰值也跟着变，满足与否不随 n 单调。
    GATE_MUTATION_CARGO_JOBS 给了时 j 固定，这一项就是 ⌊上限 ÷ P(j)⌋。
  · 头一行与收尾一行（stderr）写「内存上限 X（来源）÷ 每个工作进程的峰值 P（编译并行度 j 下：取法）= 同时开 N 个工作进程（n × P = 合计，不超过上限）」，
    卡在这一项时再写多开 1 个要的编译并行度、峰值与合计；末了写峰值记录的位置与各编译并行度的读数。stdout 不写（stdout 与进程数无关）。
  依据：查内存包装的峰值表永远查不到（工作进程的 cargo 不经包装），按写死的默认每格峰值（2G）开会低估：只开 4 个（编译并行度约 8）就到 48G，平摊约 12G。
  弄坏开关 memory-cap-ignored：不看内存上限、照线程数开满；worker-peaks-unkeyed-by-jobs、worker-peaks-window-unbounded、jobs-feedback-ignored、
  unmeasured-worker-peak-guessed、unreadable-worker-peaks-guessed、worker-tree-self-only、worker-peaks-unrecorded、sample-root-peaks-recorded（弄坏开关那一节）。
  自证在 --selftest 的 selftest_memory_cells（上限的取法）与 selftest_worker_peak_cells（量、记、读、除）。
  撞内存上限（run_stage）：内存那一项是 0（量到的每个工作进程的峰值比这一条拿到的上限还大，一个都放不下），派活之前整道退 250，
    红句写上限、峰值与「按撞内存上限退出」：250 是统一任务服务错误类型表里 memory-cap-hit 那一行（task-server/task_server_lib.py 文件头「错误类型」），服务据此照「重试」加额度重排；
    不起 1 个工作进程硬跑（硬跑在中途被杀，判过的格白判），也不改成别的拒绝。没有实测时取保守值（上限本身），1 个放得下，不走这一条。
    弄坏开关 memory-cap-short-run-anyway：开 1 个硬跑（自证 selftest_memory_cells 里「内存上限装不下一个工作进程的峰值」那一格必须判错）。
  底座读数（行名 MUTATION_BASE_READING，stderr 一行）：派活之前量起这一道的父进程（门禁这一层 python）自己此刻的用量，与「先编一份依赖」那一步（工作进程起来之前）
    父进程整棵进程树的峰值，口径同每个工作进程的读数（RssAnon + RssShmem + VmPTE 加总）：
    MUTATION_BASE_READING parent_bytes=<字节或 none> seed_step_tree_peak_bytes=<字节或 none> seed_step=built|not-built
    先编那一步没起（只开 1 个工作进程、够不着外部依赖、弄坏开关 compile-per-worker）时 seed_step=not-built，那一段只量到父进程自己；只报数、不判红。
    登记表 .claude/memory-bounded-parallelism.tsv 这一道那一行的底座照它取：两个数里大的那一个（推的口径同上「量什么」，几个进程共享的匿名页重复算）。
磁盘余量（disk_bound_for）：每个工作进程最多占两份编译目录（普通一份、插桩一份，prepare_shard 与 judge_coverage 起的），一份涨到多少按实测峰值算：
  每个工作进程每判完一格（trim_worker_target，清之前）量一次自己那份编译目录的占盘，追加进运行记录目录（文件头「判定库」末段，run_records_directory）下的 target-occupancy.tsv（一行「<时刻秒><TAB>普通|插桩<TAB><字节>」，
  record_target_occupancy），派活之前两种各取最近 20 次里最大的当这一种的涨幅（measured_target_peaks；记录多过 1000 行时只留每种最近 20 行）；
  哪一种还没有读数（头一趟、换了库）的那一种按「编译目录上限」（下下段）算，上限只当清理线。头一行写明每种的涨幅与来源。下面「还能涨的量」「上限」照这个涨幅读；
  开 2 个以上工作进程时先编一份依赖的 -seed、-seed-coverage 两份也每一趟新编，各按一份上限另算（n ≥ 2 才加）。
  派活之前按目标盘（编译目录所在那块）的余量核：片号 1…n 各两份「还能涨的量」（上限减去此刻已占，目录还没建的是整个上限；每一趟自己建的都还没建）加起来，
  n ≥ 2 再加先编那两份，再加备用 GATE_MUTATION_DISK_RESERVE（默认 30G，推的），
  不超过余量的最大 n 就是这一项；n 从 1 数到前两项最小的那个数再多 1（放得下再多 1 个才不与前两项并列成「卡在」）。一个都放不下按 1 个开（头一行写「那一项不足 1」）。
  头一行写：磁盘余量多少、来源、备用多少、每份涨到多少（含先编那两份），n 个共还要多少，再多 1 个要多少、放不下，所以给 n 个；「卡在「磁盘余量」这一项」。
  余量的来源：外层给了 GATE_MUTATION_DISK_FREE（systemd 写法，例 400G）就用它（双机分片的驱动、自证用）；没给读编译目录往上第一层存在的目录所在的盘此刻可用的字节（statvfs）；
  两样都读不到（写法不认的整道拒；statvfs 失败的）没有这一项，头一行写「磁盘余量没读到」及原因，工作进程数不按磁盘收。已有的编译目录按此刻占盘量，
  回收（reclaim_extra_target_directories）发生在这一步之后，所以只回收得掉的多出来的目录不算作余量（偏保守）。
  依据：第二台 9 个工作进程各一份 -w<片号> 加各一份插桩，会把 1.8T 的盘写满（tempfile.mkdtemp 抛 OSError: [Errno 28]）；压到 4 个仍涨到普通 39–71G、插桩 30–68G。
派活那一段（run_pool）：基线的批、变异与覆盖进同一个进程池，
  互相独立的几格按工作进程并行（同时在跑至多 worker_count 格）；一个工作进程只有一份编译目录（-w<片号>），cargo 锁着它，同一个工作进程里的几格只能逐格跑。
编译目录上限（trim_target_directory、trim_worker_target，GATE_MUTATION_TARGET_DIR_CAP，默认 64G 是推的）：每个工作进程每判完一格（基线的批、一条变异、一组覆盖），
  量一次它自己那份编译目录（判基线与变异的普通那份，判覆盖的插桩那份）此刻的占盘；超过上限就清掉 <目录>/<档>/incremental 与 <目录>/<目标三元组>/<档>/incremental
  （rustc 每编一次改坏的副本往里加一个会话目录，只增不减；两格之间没有 cargo 在这份目录上跑），清完回读再量，stderr 报清了几处、共多大、清后多大。
  清完增量仍超上限的，再清 <目录>/<档>/deps 与 <目录>/<目标三元组>/<档>/deps 里同名产物的旧版本（stale_dependency_artifacts）：文件名「<名字>-<16 位单元哈希>[.<后缀>]」
  按（目录、名字、后缀）归堆，每堆按修改时刻只留最新那一版；同一个包换了特性或档编出的另一版也算同名，之后的格要用它时 cargo 照「产物不在」重编（慢，判定不变）。
  清完回读仍超上限就停派：工作进程往这一趟 shard_root 里的停派目录（target_over_cap_directory）写一句，run_pool 每收一格看一眼，有就不再派活、等在跑的收完，
  整道判红，拒绝说明逐份点名清后仍超线的编译目录、出路是调大上限或换盘（判完的格已入库，再跑一趟只跑剩下的）。清不成（权限、文件被占）只报、不丢这一格的判定、不停派。
  本机量到的一份普通编译目录 77G，其中 debug/incremental 13G、debug/deps 30G、release 30G，所以清增量产物不保证清到线下；磁盘余量那一项按实测峰值算，不依赖清得掉。
调试信息（cargo_debug_overrides、key_toolchain_lines）：这一道起的每条 cargo（基线、变异的先编与跑、插桩编译与插桩跑、先编一份依赖）都带
  CARGO_PROFILE_DEV_DEBUG、CARGO_PROFILE_TEST_DEBUG、CARGO_PROFILE_RELEASE_DEBUG、CARGO_PROFILE_BENCH_DEBUG = CARGO_DEBUG_SETTING（false）：cargo test 用 test 档（承 dev），
  cargo test --release 用 release 档（bench 承 release），四档都设，顶掉调用方设的同名变量与仓根 Cargo.toml 的 [profile.release] debug = true。只设在起 cargo 的那一份环境里。
  命中重测试目标、这一趟补了 --release 的行（「重测试目标只许 release」）走的就是 release 档，同样关调试信息；档本身照 --release 进行键的「开关与特性」。
  取值进行键：算工具链行时照起 cargo 的环境算（lib_mutation_units.py 的 toolchain_lines 收 CARGO_PROFILE_*），run_stage、plan、row-plan、--list-items 与 toolchain 子命令同一份；
  取值改了（连同这一段头一次进来）整张表现判一次，栏是「工具链或依赖变了」，不迁移旧键。
  不改判定：调试信息只进 DWARF 段，opt-level、overflow-checks、debug-assertions 不动；插桩的覆盖映射不靠 DWARF（量过：一个探针包关掉调试信息、带 -Cinstrument-coverage
  编出来的测试二进制没有 .debug_info 段，llvm-cov export 照样报执行过的函数计数 1、没执行的 0）。
  量到的（本机，singlefs-core 的测试目标冷编）：开着时写盘 916 MB、编译目录 826 MB、测试二进制 79 MB；关掉后 510 MB、421 MB、17 MB。
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
  并行度 = 核数（这时还没有工作进程在跑），GATE_MUTATION_CARGO_JOBS 设了用它（seed_jobs）。
  拷（copy_seed_dependencies、sync_dependency_artifacts）：每个工作进程起来时各拷各的（几个工作进程同时拷），一个工作进程里普通、插桩两份要拷的文件
    再分到 CARGO_BUILD_JOBS（核数 ÷ 工作进程数）条线程上拷。
  先编没编成（退出码不是 0）的那一份只报、不判红，工作进程照旧自己编依赖（判定不受影响，只是慢）；拷的时候出错同样只报。编完照「编译目录上限」清一次那两份的增量产物；
  「磁盘余量」给这两份各留一份上限（每一趟新编）。只开 1 个工作进程、要现跑的行都够不着外部依赖（树里没有 Cargo.lock 的自证小仓就是这样）时不起，stderr 写明没起的原因。
  量到的（本机，singlefs-core 的测试目标、关掉调试信息）：冷编一份写盘 510 MB、CPU 31.6 秒；拷依赖产物 175 个文件 98 MB；拷完之后在另一份新副本里编，
  只有 singlefs-core、singlefs-format、singlefs-replay-prefix 三个工作区包重编，依赖 16 个全 Fresh，写盘 343 MB、CPU 21.3 秒（与拿着整份旧编译目录的工作进程 343 MB、20.6 秒相同）。
测试线程数：不压。起 cargo test 时 RUST_TEST_THREADS 与仓里各测试读的线程数变量（SINGLEFS_THREAD_CAP 等）调用方设了什么就原样透传；只有 CARGO_BUILD_JOBS（编译并行度）按核数 ÷ 工作进程数收。
变异复跑的崩溃判定：起 cargo test 真跑测试的每一次（基线的批、变异、记覆盖的插桩跑；只编不跑的 --no-run 不起）各起一份自己的临时 prune-server
  （mutation_run_prune_server：照 scripts/prune-server/prune_server_client_selftest.py 的 TestServer 起，socket、状态目录与库都在它自己的临时目录里），
  那一次环境里连哪一处（SINGLEFS_PRUNE_SERVER_SOCKET、SINGLEFS_PRUNE_SERVER_TCP_ADDRESS）与临时服务的标记（PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET）都指它，
  跑完停服务、删目录：走判定库的 验证包用例在变异版上判出的崩溃结论只落进那一份临时库，不读真库、不写真库（变异复跑本就该现判）。
  服务二进制取 PRUNE_SERVER_BINARY（绝对路径、可执行；与客户端自证的 resolve_binary 认同一个变量），没设由 run_pool 开派之前照本模块所在仓的 prune-server/
  编一份进这一趟的 <shard_root>/prune-server-build（cargo build --release --offline，收尾随 shard_root 删）；取不到整道判红、一条都不跑。
  起服务经 <shard_root> 里写的一份包装：先设父进程死亡信号再换成真二进制，工作进程被停或被硬杀时（finally 不跑）服务跟着停，不留孤儿。
  弄坏开关 mutation-run-on-real-service：不起临时服务，照调用方的环境连那一份服务，变异版判出的崩溃结论写进真库同一把键下。
libtest 的结果行（`test <全名>[ - should panic] ... <结局>`）与汇总行只在 scripts/gate/lib_libtest_result_lines.py 解析一份（results_by_test 调它）；
  结果词被测试中途打的行隔开的写法（`test <名> ... ` 之后隔着别的输出、结局另起一行；测试里的线程直接写标准输出时出现）由 joined_result_lines 先接回一行再交它：
  结局单独一行，起头与结局之间的行不算，起头之后先来了别的结果起头的不接，单独的结局行多出来没处接的整段不接（同 lib_libtest_result_lines.py 的 account_binary）。
  弄坏开关 split-result-line-unseen 见上面的开关表。
  它的弄坏开关 LIBTEST_RESULT_LINES_BREAK 写在它文件头；本模块自证里认得它的那一格：
  LIBTEST_RESULT_LINES_BREAK=should-panic-in-name  本模块自证「should panic 的用例红了判抓到、点名省掉模块路径只对上那一个」那一格必须判错
下面五个开关（磁盘余量、编译目录上限）一行一个，下一行起写打开后必须判错的自证项：
  disknotchecked       工作进程数不看磁盘余量：第二台 9 个工作进程把盘写满
    本模块自证「磁盘余量放不下 16 份…」「已有的编译目录只算还能涨的…」「磁盘余量交给 run_stage…」「外层没给磁盘余量时读编译目录所在的盘…」四格必须判错
  targetcapignored     编译目录超过上限也不清增量产物：产物只增不减
    本模块自证「编译目录超过上限：清掉两处增量产物…」「清完仍超线：只清增量产物…」「编译目录上限交给 run_stage…」「编译目录清完增量仍超上限：deps 里同名产物只留最新一版…」
    「编译目录清完仍超上限（1K）：头一格判完就停派…」五格必须判错
  depsnotpruned        清完增量仍超上限也不清 deps 里同名产物的旧版本
    本模块自证「编译目录清完增量仍超上限：deps 里同名产物只留最新一版…」那一格必须判错
  overcapdispatched    清完仍超上限只报、照派
    本模块自证「编译目录清完仍超上限（1K）：头一格判完就停派…」那一格必须判错
  diskbudgetbycap      磁盘预算不看实测峰值、每份编译目录一律按上限算
    本模块自证「磁盘预算按实测峰值算：…」那一格必须判错
下面九个开关（调试信息、先编一份依赖）一行一个，下一行起写打开后必须判错的自证项：
  debuginfo-kept       起 cargo 时不设 CARGO_PROFILE_<档>_DEBUG：调试信息开着
    本模块自证「门禁起的每条 cargo 都关了调试信息」「改了构建档的取值之后行键变了」两格必须判错
  debuginfo-unkeyed    起 cargo 时照设，算工具链行时不算进去（构建档的取值不进行键）
    本模块自证「改了构建档的取值之后行键变了」那一格必须判错
  compile-per-worker   不起先编那一份，每个工作进程自己编外部依赖
    本模块自证「4 个工作进程：外部依赖只在先编那一份里各编一次」那一格必须判错；「先编那一份的源码按内容同步」「先编那一份的修改时刻被留住」两格跟着判错（先编那一趟没起）
  workspaceunitsshared 从先编那一份拷产物时连工作区的包一起拷
    本模块自证「先编那一份只拷外部依赖的产物」那一格必须判错
  seedmtimekept        同步先编那一份的源码时，内容变了的文件写进去之后修改时刻改回原来的、一个包都不摸（修改时刻规矩坏了的样子）
    本模块自证「先编那一份的源码按内容同步：改过的包重编、整道照常」那一格必须判错（先编那一趟判红）
  seedfreshunchecked   先编那一趟不核摸过的包有没有重编
    本模块自证「先编那一份的修改时刻被留住：先编那一趟判红、一条都不跑」那一格必须判错
  seedfreshacrossgroups 先编那一趟每一组各判各的：前面的组已经编过的产物，后面的组里 fresh 也记成坏的
    本模块自证「先编那一份有两组：后一组复用前一组编出来的产物、cargo 报 fresh 不算坏，整道照常」那一格必须判错（先编那一趟判红）
  seed-copy-serial     工作进程从先编那一份拷依赖产物时在一条线程上逐个拷
    本模块自证「工作进程拷依赖产物分到几条线程上同时拷」那一格必须判错
下面这个开关（基线的批按工作进程并行），下一行起写打开后必须判错的自证项：
  baseline-one-at-a-time 基线的批一次只派一批（在跑的那一批收完才派下一批）
    本模块自证「4 个工作进程、8 批基线：同时在跑的基线不少于工作进程数…」那一格必须判错
重测试目标只许 release 那两个开关（文件头同名一段；写成 CRATES_MUTATION_ROWS_BREAK=<项>，GATE_MUTATION_BREAK 同样认），一行一个，下一行起写打开后必须判错的自证项：
  CRATES_MUTATION_ROWS_BREAK=release-unforced  命中重测试目标而 cargo 参数没带 release 的行不补 --release：崩溃枚举用例目标与 gpu 行在 debug 档跑
    本模块自证「重测试目标只许 release：命中的两行（crash-case 目标、gpu 特性）起的每一次 cargo 都带 --release、只带一个，不命中的那一行一次都不带，stderr 点名补了哪两行」那一格必须判错
  CRATES_MUTATION_ROWS_BREAK=release-unchecked  static-check 不判命中重测试目标而第五段没写 --release 的行
    本模块自证「static-check 的 RELEASE：命中而第五段没写 --release 的两行（gpu 特性、crash-case 目标）各报一行、带依据、跟一行写出路，退 1；命中写了的与不命中的不报，两行都写上之后退 0」那一格必须判错
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
import traceback
from concurrent.futures.process import BrokenProcessPool

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
STAGE_REPOSITORY = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(STAGE_REPOSITORY, ".claude", "scripts"))
from project_preflight import preflight  # noqa: E402
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "hosts"))
from heavy_threads import HeavyThreadDefaultError, default_heavy_threads  # noqa: E402
import multi_host_addresses  # noqa: E402

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

STAGE = "crates-mutation-replay.sh"   # 登记表的键（拷贝范围）、阶段脚本的文件名
STAGE_SCRIPT = os.path.join(STAGE_REPOSITORY, ".claude", "gate.d", STAGE)
MODULE_PATH = os.path.realpath(__file__)
ROCKSDB_PREBUILT_PATH = os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "rocksdb-prebuilt.py")   # 起 cargo 之前判 RocksDB 预编库的取值（run_stage）
TABLE = "crates/mutations.tsv"
# 判定库（文件头「判定库」）：被判那棵树的 git common-dir 底下这一个目录，固定一处、不按版本号分目录
LIBRARY_DIRECTORY_NAME = "singlefs-crates-mutation-library"
ROW_KEY_FORM = "singlefs-crates-mutation-row-v1"
BASELINE_KEY_FORM = "singlefs-crates-mutation-baseline-v1"
COVERAGE_KEY_FORM = "singlefs-crates-mutation-coverage-v1"
# 判法单元（文件头「判法单元」）：判「抓没抓到」的那几个函数，从它们收 python 闭包；派活的调度、判定库的存放不在里面
JUDGING_ENTRIES_VERDICT_ONLY = (
    "scripts/mutation/crates-mutation-rows.py::verdict_of_run",            # 结局分类，连同 libtest 结果行的解析（results_by_test → lib_libtest_result_lines.py）
    "scripts/mutation/crates-mutation-rows.py::baseline_problem",          # 基线那一批的结局分类
    "scripts/mutation/crates-mutation-rows.py::baseline_reuse_problem",    # 覆盖复用判据：基线 ok 那一条复用不复用
    "scripts/mutation/lib_mutation_units.py::UnitKeys.mutated_units",      # 切 fn 规则：一条变异落在哪几个单元上
    "scripts/mutation/lib_mutation_units.py::UnitKeys.resolve_test",       # 点名的测试是哪一个单元
    "scripts/mutation/lib_mutation_units.py::UnitKeys.executed_units",     # 覆盖落进哪几个单元
)
# 施加变异、还原源码、拼基线与变异的命令、起 cargo、拷每片的源码副本：这几段改了，同一把行键下判出的结局就会变，一并进判法单元
# （弄坏开关 judging-entries-verdict-only 下只收上面那几个）。起进程那一下、找多机配置文件与起临时 prune-server 经 init_worker 交的引用，不进闭包
# （judge_row 那几个的闭包里没有折不出命令的 Popen，也不载入客户端自证那一份；弄坏开关 judging-entries-with-temporary-service 下另收 mutation_run_prune_server）；记覆盖的 judge_coverage 不进：它的闭包够到 scripts/gate/lib_function_units.py 的自证夹具（折不出路径的 subprocess），覆盖落单元的判法另由
# UnitKeys.executed_units 管
JUDGING_ENTRIES = JUDGING_ENTRIES_VERDICT_ONLY + (
    "scripts/mutation/crates-mutation-rows.py::judge_row",                 # 改坏那一处、跑点名的测试、还原、判结局、写记录
    "scripts/mutation/crates-mutation-rows.py::judge_batch",               # 一批基线：拼参数、跑、逐行判基线结局、写记录
    "scripts/mutation/crates-mutation-rows.py::baseline_arguments",        # 一批基线的 cargo test 参数
    "scripts/mutation/crates-mutation-rows.py::row_baseline_output",       # 一行的基线结局只看它自己的过滤词选中的那几条用例
    "scripts/mutation/crates-mutation-rows.py::with_shown_output",         # 变异那一趟补 --show-output
    "scripts/mutation/crates-mutation-rows.py::run_cargo_command",         # 起 cargo（多机配置与 --test-threads 照这一行的特性）
    "scripts/mutation/crates-mutation-rows.py::prepare_shard",             # 每片的源码副本拷什么
)


JUDGING_ENTRY_TEMPORARY_SERVICE = "scripts/mutation/crates-mutation-rows.py::mutation_run_prune_server"   # 只给弄坏开关 judging-entries-with-temporary-service


def judging_entries(environment):
    """这一趟判法单元从哪几个入口收闭包（文件头「判法单元」）。弄坏开关 judging-entries-verdict-only：只收结局分类那几个；
    judging-entries-with-temporary-service：另收起临时 prune-server 那一段，run_cargo_command 直接调 mutation_run_prune_server。"""
    if break_is(environment, "judging-entries-verdict-only"):
        return JUDGING_ENTRIES_VERDICT_ONLY
    if break_is(environment, "judging-entries-with-temporary-service"):
        return JUDGING_ENTRIES + (JUDGING_ENTRY_TEMPORARY_SERVICE,)
    return JUDGING_ENTRIES
LLVM_TOOLS = ("llvm-profdata", "llvm-cov")
LLVM_TOOLS_VARIABLE = "GATE_MUTATION_LLVM_TOOLS"
COVERAGE_TARGET_SUFFIX = "-coverage"
# 编译目录（文件头「编译目录」）：GATE_MUTATION_TARGET_DIR 设了用它（跑完不删）；没设建在这一趟的 <shard_root>/build 里、跑完删
TARGET_DIRECTORY_NAME = "singlefs-crates-mutation-target"
RUN_BUILD_DIRECTORY_NAME = "build"
RUN_BUILD_REMOVAL_FAILURES = []   # run_pool 收尾没删成的这一趟编译目录（有进程开着里面的文件、或删不成）：run_stage 照它整道判红
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
REUSE_COLUMNS = (("row-new", "行新加"), ("judging", "判法单元变了"), ("mutated", "被改坏的单元变了"),
                 ("test", "点名测试的闭包变了"), ("text", "原文或替换文变了"), ("environment", "工具链或依赖变了"),
                 ("not-caught", "上次没判成抓到"), ("unexplained", "键对不上"))
COLUMN_TITLES = dict(REUSE_COLUMNS, refused="拒算", sentinel="复用哨兵强制现判")
REASONS_SHOWN = 20   # stderr 逐行列现判与拒算原因的行数上限（只管列多少，全部的看 plan）
REUSABLE_VERDICT = {"mutation": "caught", "baseline": "ok"}
RECORD_HEADER = ("# 门禁 crates-mutation-replay 的判定库记录（scripts/mutation/crates-mutation-rows.py 写，一条一个文件，按键寻址）：只有「抓到」与基线 ok 的、"
                 "照复用判据判过的复用（scripts/mutation/crates-mutation-rows.py 文件头「复用判据」）。不进工作树，别手改。\n")
# 拷目录时跳过的名字（各 crate 自己的编译目录）。拷贝范围由它与登记的路径一起定：prepare_shard 照它们拷，is_copied_into_each_shard 照它们判。
IGNORED_WHEN_COPYING_INTO_EACH_SHARD = ["target"]
# 拷贝范围里的 cargo 配置目录：.cargo/config.toml 的 [env] BINDGEN_EXTRA_CLANG_ARGS 给 librocksdb-sys 的 bindgen 指头文件；副本在 /tmp 下、往上找不到仓里那一份，
# 验证包常编 librocksdb-sys（prune-server 是它常在的依赖），点名它的测试的行基线必然不绿（弄坏开关 cargoconfignotcopied 拷的时候跳过它）
CARGO_CONFIGURATION_DIRECTORY = ".cargo"
DEFAULT_LONG_ROW_SECONDS = "300"               # 上一次用时不短于它的行先派：GATE_MUTATION_LONG_ROW_SECONDS
STOP_GRACE_SECONDS = 30                        # 被停时对自己起的那一条命令的进程组发 TERM，过这么久还不退再发 KILL
PER_MUTATION_TEMPORARY_DIRECTORY_NAME = "per-mutation-tmp"
SIZE_TEXT = re.compile(r"^([1-9][0-9]*)([KMGT])$")             # 磁盘余量与编译目录上限的写法：systemd 的写法，1024 进制
SIZE_UNIT_BYTES = {"K": 1024, "M": 1024 ** 2, "G": 1024 ** 3, "T": 1024 ** 4}
# 编译目录占盘（文件头「磁盘余量」「编译目录上限」）：每个工作进程最多占两份编译目录（普通一份、插桩一份），一份涨满按 TARGET_DIRECTORY_CAP 算。
# 默认 64G 是推的：第二台量到的 -w<片号> 普通 39–71G、插桩 30–68G（records/2026-09-16-subagent拆分提案.md 第七十八节追记），取中偏上；
# 本机一份普通的此刻 77G，其中 debug/incremental 13G、debug/deps 30G、release 30G（量的），所以清增量产物不一定清到线下，清完仍超线的只报、不动别的。
# 备用 30G 是推的：留给别的会话与系统，不让工作进程把盘写满（第二台 1.8T 的盘会被写满）。
TARGET_DIRECTORY_CAP_VARIABLE = "GATE_MUTATION_TARGET_DIR_CAP"
DEFAULT_TARGET_DIRECTORY_CAP = "64G"
DISK_FREE_VARIABLE = "GATE_MUTATION_DISK_FREE"          # 外层给的目标盘余量（systemd 写法）；不设就读编译目录所在那块盘
DISK_RESERVE_VARIABLE = "GATE_MUTATION_DISK_RESERVE"
DEFAULT_DISK_RESERVE = "30G"
# 内存上限与每个工作进程的峰值（文件头「内存上限」）：开几个工作进程取满足「n × 那个编译并行度下每个工作进程的峰值 ≤ 内存上限」的最大 n。
# 上限先读统一任务服务交的 TASK_SERVER_MEMORY_CAP_BYTES（这一次尝试的上限，字节十进制整数），
# 没有读仓根多机配置 multi-host.env 的 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项（用 scripts/gate/admission.py 的 configuration_value，这一台照 multi_host_addresses.resolve，
# 找文件同 test_runs_multi_host_configuration），
# 再没有读自己 scope 的 memory.max（/proc/self/cgroup 的 cgroup v2 那一行接 /sys/fs/cgroup 再接 /memory.max），值是 max 或都读不到就没有这一项（照线程数）。
# memory.max 是有限值时它是硬顶：前两级取到的值比它大就取它（两者取小），前两级都没有取它本身。
# 每个工作进程的峰值由工作进程自己量整棵进程树、按编译并行度记进运行记录目录下的 worker-peaks/（文件头「内存上限」的「每个工作进程的峰值」）；
# 没有实测取保守值（内存上限本身，只开 1 个），不用写死的默认值。
TASK_MEMORY_CAP_VARIABLE = "TASK_SERVER_MEMORY_CAP_BYTES"
WORKER_PEAKS_RECENT = 256                # 每个编译并行度只看最新的这么多份记录（一个工作进程一趟一份）：取得宽，几趟只跑几行的点名跑挤不掉整张表那几趟的重读数（推的）
WORKER_PEAK_SAMPLE_SECONDS = 0.2         # 工作进程里的采样线程隔多久量一次整棵进程树
WORKER_PEAK_WRITE_STEP_BYTES = 16 << 20  # 这一趟的最大涨了这么多才重写一次记录：记下的比真峰值至多少这么多
WORKER_PEAK_RECORD_NAME = re.compile(r"[0-9]{20}-[0-9]+-[0-9a-z]{8}")   # worker-peaks/<编译并行度>/ 下一份记录的名字：起采样的时刻纳秒、进程号、随机字符
SLICE_MEMORY_CAP_KEY = "SLICE_MEMORY_CAP_BY_HOST"   # 仓根多机配置 multi-host.env 里各台 slice 的总上限（<地址>=<systemd 写法> 的若干项，取这一台那一项；变异的内存上限直接按根目录设置的 50G 走）
CGROUP_LISTING = "/proc/self/cgroup"
CGROUP_ROOT = "/sys/fs/cgroup"
TARGET_DIRECTORY_COPIES_PER_WORKER = 2                  # 普通一份、插桩一份
INCREMENTAL_DIRECTORY_NAME = "incremental"
# 算数的结论与不算数的结局（文件头「复用判据」记红撤绿那一段）：不算数的照行键写，但不盖同一把行键原有的算数结论
VERDICTS_COUNTED = ("caught", "failure", "invalid")
VERDICTS_UNCOUNTED = ("notrun",)
TEST_NAME_FORM = re.compile(r"[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)*")
# 点名的测试在本机没跑到被改坏的那一处时打的一行（见文件头「本次未跑」）；变异那一趟补 --show-output，libtest 才把成功用例的这一行交出来
NOT_RUN_LINE = re.compile(r"^\s*NOT_RUN_NO_GPU_ADAPTER test=(\S+)(?: reason=(.*))?$", re.M)
LIBTEST_SHOW_OUTPUT = "--show-output"
# 开了就要上卡的 cargo 特性：验证包的 gpu，与产品 GPU 包（singlefs-background-gpu）真显卡对拍的 tests-on-a-real-gpu（Cargo.toml 的 [features] 与 [[test]] required-features 里的那个名字）
GPU_TEST_FEATURES = frozenset({"gpu", "tests-on-a-real-gpu"})
# 配了 GPU 而二进制没编进 gpu 特性就判段不跑的那两处选路函数（包、所在文件、函数名）：点名测试的单元闭包够得着其中一个，多机配置开了 GPU 时这一行补 gpu 特性起
#   （文件头「开 GPU 时崩溃判段那几行补 gpu 特性」）。崩溃枚举用例走 crash_state_enumeration 的选路，给定状态引擎走 crash_amplification 的那一处，前者也经它取本次未跑那一句。
#   一样的列表，自证那一格换成自己的小仓里的那一处（GPU_ROUTE_UNITS[0] 整个换掉再换回）
GPU_ROUTE_UNITS = [(("singlefs-verification", "crates/singlefs-verification/src/crash_state_enumeration.rs", "crash_point_replay_judge_route"),
                    ("singlefs-verification", "crates/singlefs-verification/src/crash_amplification.rs", "judge_not_run_without_the_gpu_feature"))]
GPU_ROUTE_FEATURE = "gpu"
# 开 GPU 时 GPU 路上不走的 CPU 判器入口（文件头「点名测试够不着被改坏的函数」）：static-check 可达性照它截点名测试的单元闭包。
#   一项 = (来处, 到)，两样都写 (所在文件, 函数名)；来处是 None 的按单元停（那个函数不进截过的闭包），写了来处的只停从来处那个函数连到它的那一条边
#   （同一个函数被测试或别的函数直接调，照连）。一样的列表，自证那一格换成自己的小仓里的那几处（REACHABILITY_GPU_ROUTE_STOPS[0] 整个换掉再换回）
REACHABILITY_GPU_ROUTE_STOPS = [(
    # 崩溃枚举判段的 CPU 那一臂（judge_the_recorded_item 里 CrashPointReplayJudgeRoute::Cpu）
    (("crates/singlefs-verification/src/crash_state_enumeration.rs", "judge_the_recorded_item"),
     ("crates/singlefs-verification/src/crash_amplification.rs", "check_blocks_where_until_with_threads")),
    # 崩溃放量判段的 CPU 那一臂（run_from_environment_with_observers 里 ComputedOn::Cpu）
    (("crates/singlefs-verification/src/crash_amplification.rs", "run_from_environment_with_observers"),
     ("crates/singlefs-verification/src/crash_amplification.rs", "check_selected_blocks")),
    # 给定状态引擎的 CPU 判器：配了 GPU 只在 GPU 建不出判表（豁免见 REACHABILITY_CPU_FALLBACK_ROOTS）或有 GPU 判不了的状态时才进
    (None, ("crates/singlefs-verification/src/crash_amplification.rs", "judged_on_the_cpu_here")),
    # 排计划时对基镜像恢复一次，结局只核「不晚于最新根」：改坏恢复照样过，执行了也判不出
    (None, ("crates/singlefs-verification/src/crash_amplification.rs", "assert_journal_recovery_of_the_base_image_lands_no_later_than_its_newest_root")),
)]
# GPU 许建不出判表、回到 CPU 判器的镜像：截过的闭包里有这几个函数之一的行豁免（文件头「点名测试够不着被改坏的函数」豁免那一段）；一样的列表，自证那一格整个换掉再换回
REACHABILITY_CPU_FALLBACK_ROOTS = [(("crates/singlefs-verification/tests/common_images_for_the_given_states_engine/mod.rs", "hand_out_allowing_the_gpu_to_refuse_its_tables"),)]
# 测试自己写的那份多机配置关着 GPU：串字面量里有它（文件头「点名测试够不着被改坏的函数」豁免那一段）
REACHABILITY_CPU_CONFIGURATION_TEXT = "ENABLE_GPU=0"
REACHABILITY_HOWTO = ("换成调得到被改坏函数的用例，或删掉这一行（closure：点名测试的单元闭包里根本没有被改坏的单元，开不开 GPU 都调不到；"
                      "gpu-route：开 GPU 时点名测试走 GPU 判段，被改坏的单元只在 GPU 路上不走的 CPU 判器入口后面。换的用例照 UNREACHABLE 行末那一段 cargo 参数（门禁真给这一行用的）核："
                      "直接调 CPU 判器，或自己写一份 ENABLE_GPU=0 的多机配置经 SINGLEFS_MULTI_HOST_CONFIG 交给引擎；换完单跑 python3 scripts/mutation/crates-mutation-rows.py static-check <根> 看那一行不再报，"
                      "再单跑那一行证明点名的测试红（scripts/mutation/prove-red.sh））")
COLLISION_REFUSAL_PREFIX = "行键撞键："   # 撞键拒算那一句的起头（refuse_colliding_row_keys 写、run_stage 与成功句照它认）
BYTES_PER_STAT_BLOCK = 512   # os.stat 的 st_blocks 按 512 字节一块数（POSIX）
EXIT_SELECTION_ONLY = 3      # 只判了选中的行、都抓到：不是整道结论，门禁汇总记失败，不会被当成通过
# run_pool 里接住这两个信号：先停自己起的工作进程、删 shard_root，再退 128 + 信号号（双机分片的驱动停这一份、门禁被打断时都走这一条）
STOP_SIGNALS = (signal.SIGTERM, signal.SIGHUP)
# run_pool 在 $TMPDIR 下建的 shard_root 的名字起头（mutation-shard-run.sh 的自证、本模块的自证都按这个起头找它）
SHARD_ROOT_PREFIX = "singlefs-mutation-replay-"
# 变异复跑的崩溃判定（文件头「变异复跑的崩溃判定」）：每次真跑测试各起一份临时 prune-server，起法在客户端自证那一份（TestServer、selftest_root）
PRUNE_SERVER_TEMPORARY_SERVICE_SCRIPT = os.path.join(SCRIPT_DIRECTORY, "..", "prune-server", "prune_server_client_selftest.py")
PRUNE_SERVER_BINARY_VARIABLE = "PRUNE_SERVER_BINARY"
PRUNE_SERVER_BUILD_DIRECTORY_NAME = "prune-server-build"   # 没给 PRUNE_SERVER_BINARY 时这一趟编服务的编译目录，放 <shard_root> 底下
PRUNE_SERVER_WRAPPER_NAME = "prune-server-dies-with-its-worker"   # mutation_run_prune_server_binary 写进 <shard_root> 的包装
# 那份包装：先设父进程死亡信号（起它的工作进程一死，内核给它发 SIGKILL），设之前起它的进程已经没了就退，再换成真二进制（REAL_BINARY_PATH 换成它的路径）
PRUNE_SERVER_PARENT_DEATH_WRAPPER = '''#!/usr/bin/env python3
# scripts/mutation/crates-mutation-rows.py 写的包装（它文件头「变异复跑的崩溃判定」）：设父进程死亡信号，再换成真的 prune-server
import ctypes, os, signal, sys
PARENT_DEATH_SIGNAL_OPTION = 1   # prctl 的 PR_SET_PDEATHSIG
parent = os.getppid()
if ctypes.CDLL(None, use_errno=True).prctl(PARENT_DEATH_SIGNAL_OPTION, int(signal.SIGKILL)) != 0:
    sys.exit(f"prctl(PR_SET_PDEATHSIG) 没成：errno {ctypes.get_errno()}")
if os.getppid() != parent:
    sys.exit("起这个服务的工作进程在设父进程死亡信号之前就没了")
os.execv(REAL_BINARY_PATH, [REAL_BINARY_PATH] + sys.argv[1:])
'''
# 源码里一处字符串字面量或注释的起头（找字面量时跳过注释用）
LITERAL_OR_COMMENT_START = re.compile(r'//|/\*|b?r#*"|b?"|\'')
PATH_ATTRIBUTE_BEFORE_LITERAL = re.compile(r"#\s*\[\s*path\s*=\s*$")
PATH_ATTRIBUTE_AFTER_LITERAL = re.compile(r"\s*\]")
# run_stage 读的环境变量（自证起每一格之前从调用方的环境里清掉，弄坏开关除外）
STAGE_VARIABLES = ("GATE_MUTATION_WORKERS", "GATE_MUTATION_CARGO_JOBS",
                   "GATE_MUTATION_ROW_SELECTION", "GATE_MUTATION_LONG_ROW_SECONDS",
                   "GATE_MUTATION_STOP_AFTER_ROWS", "GATE_MUTATION_TARGET_DIR",
                   "GATE_MUTATION_NOT_RUN_RECORD", "GATE_MUTATION_LLVM_TOOLS",
                   "SINGLEFS_REUSE_HOURS", "GATE_MUTATION_KEY_WORKERS",
                   TARGET_DIRECTORY_CAP_VARIABLE, DISK_FREE_VARIABLE, DISK_RESERVE_VARIABLE, TASK_MEMORY_CAP_VARIABLE)

Row = collections.namedtuple("Row", "line_number name path old new arguments expected fields")

FIELD_NAMES = ("变异名", "文件", "原文", "替换文", "cargo test 参数", "必须红的测试名")
REPLACEMENT_FIELD = FIELD_NAMES.index("替换文")   # 六段里只有它可以空：替换文空着就是把原文删掉
# 派活之前（crates-mutation-replay）与静态（code-source-discipline 经 static-check）判的那几样，打印与输出都照这个次序
ROW_PROBLEM_KINDS = ("MALFORMED", "ANCHOR", "NAME", "ROW", "STRAY")
_ADMISSION_MODULE = []
# 拷贝范围（copied_with_path_dependencies）用 scripts/hosts/lib_repository_copy.py 的 copy_closure，不另抄一份；多机配置只交给真跑的测试（test_runs_multi_host_configuration）
REPOSITORY_COPY_PATH = os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "lib_repository_copy.py")
# 拷贝范围跟不全时 <根> 是仓副本的那一种出路（编译期、运行期读的仓根文件在副本里没有：副本拷不全）；建法与 lib_repository_copy.py 的 copy_rebuild_howto 同一句
COPY_REBUILD_HOWTO = ("<根> 是仓副本、报的是读的仓根文件「在仓里没有」的，是副本拷不全：仓副本只用 python3 scripts/hosts/minimal-copy.py <仓根> <新副本目录> --with-test-inputs 建"
                      "（拷哪些由 scripts/hosts/lib_repository_copy.py 的闭包现算），副本里自己的改动搬进新副本再判；不手拷、不手补缺的条目")
REGISTERED_COPY_RANGE_REASON = "登记在 .claude/gate.d/stage-inputs.tsv 本阶段那一行的 copy-range"
_REPOSITORY_COPY_MODULE = []
_PRUNE_SERVER_TEMPORARY_SERVICE_MODULE = []
# 重测试目标只许 release（文件头同名一段）：判法只在 lib_release_only_tests.py，这里按路径载入调它
RELEASE_ONLY_TESTS_PATH = os.path.join(SCRIPT_DIRECTORY, "lib_release_only_tests.py")
_RELEASE_ONLY_TESTS_MODULE = []
# 登记表从哪一份仓根读：本模块所在那一份（判别力样本的目录里没有登记表，同拷贝范围）；自证那两格换成自己造的一份再换回
RELEASE_REGISTRY_ROOT = [STAGE_REPOSITORY]
RELEASE_HOWTO = ("第五段补上 --release（插在第一个 -- 之前）：穷举、长时间随机、真设备这类用例用优化构建跑（.claude/rules/test-script-discipline.md「编译与算法都要快」）；"
                 "命中哪一样写在每行的依据里，判法在 scripts/mutation/lib_release_only_tests.py；补完单跑 python3 scripts/mutation/crates-mutation-rows.py static-check . 看那一行不再报 RELEASE")
RELEASE_REGISTRY_HOWTO = ("照报错改 .claude/gate.d/stage-inputs.tsv 的 crash-case: 行（写法在那张表的文件头与 scripts/gate/admission.py 的 parse_crash_case）；"
                          "登记表读不了就判不了哪几行是重测试目标，不当不命中")


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


def cargo_debug_overrides(environment):
    """这一道起的 cargo 的构建档覆盖（文件头「调试信息」）：{CARGO_PROFILE_<档>_DEBUG: 取值}，起 cargo 的环境照它顶掉调用方的同名变量。
    弄坏开关 debuginfo-kept：交空（调试信息照 Cargo.toml 与调用方的环境开着）。"""
    if break_is(environment, "debuginfo-kept"):
        return {}
    return {f"CARGO_PROFILE_{profile}_DEBUG": CARGO_DEBUG_SETTING[0] for profile in CARGO_DEBUG_PROFILES}


def key_toolchain_lines(root, environment):
    """行键里的工具链行（文件头「行键」「调试信息」）：照起 cargo 时的环境算（调用方的环境加 cargo_debug_overrides），构建档的取值就进了键。
    KeyContext 与 toolchain 子命令都调它。rustc / cargo 起不来抛 MutationUnitsError。弄坏开关 debuginfo-unkeyed：照调用方的环境算（构建档的取值不进键）。"""
    if break_is(environment, "debuginfo-unkeyed"):
        return mutation_units.toolchain_lines(root, environment)
    return mutation_units.toolchain_lines(root, dict(environment, **cargo_debug_overrides(environment)))


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
    """(成形的行, [(行号, 变异名, 为什么不成形)])。成形照 malformed_reason：
    crates-mutation-replay 与 static-check（门禁 code-source-discipline）都照这一份读，没有第二份判法。environment 不给就读本进程的环境（只看弄坏开关）。"""
    environment = os.environ if environment is None else environment
    rows, malformed = [], []
    with open(path, encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, 1):
            line = line.rstrip("\n")
            if not line or line.startswith("#"):
                continue
            fields = line.split("\t")
            reason = malformed_reason(fields, environment)
            if reason:
                malformed.append((line_number, fields[0] or "（变异名空着）", reason))
                continue
            rows.append(Row(line_number, fields[0], fields[1], unescape(fields[2]), unescape(fields[3]), fields[4], fields[5], tuple(fields)))
    return rows, malformed


# ── 拷贝范围（crates-mutation-replay 拷进每片的、code-source-discipline 静态判的，同一份判法）──────────────────────

def copied_paths(stage_repository=STAGE_REPOSITORY):
    """登记表 crates-mutation-replay 那一行登记的路径（目录去掉末尾的斜杠）。登记表读不了、没有那一行抛 admission 的 RegistrationError。"""
    admission = admission_module()
    input_paths, _rows = admission.resolve_input_paths(admission.read_registration_rows(stage_repository), STAGE)
    return [input_path.strip().rstrip("/") for input_path in input_paths if input_path.strip()]


def repository_copy_module():
    """scripts/hosts/lib_repository_copy.py（拷贝范围的 copy_closure、path_dependency_entries 与副本里的环境），第一次用到时才读进来。"""
    if not _REPOSITORY_COPY_MODULE:
        specification = importlib.util.spec_from_file_location("lib_repository_copy_for_crates_mutation_rows", REPOSITORY_COPY_PATH)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _REPOSITORY_COPY_MODULE.append(module)
    return _REPOSITORY_COPY_MODULE[0]


def release_only_tests_module():
    """scripts/mutation/lib_release_only_tests.py（重测试目标只许 release 的判法），第一次用到时才读进来。"""
    if not _RELEASE_ONLY_TESTS_MODULE:
        specification = importlib.util.spec_from_file_location("lib_release_only_tests_for_crates_mutation_rows", RELEASE_ONLY_TESTS_PATH)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _RELEASE_ONLY_TESTS_MODULE.append(module)
    return _RELEASE_ONLY_TESTS_MODULE[0]


def release_registered_targets():
    """登记表里 crash-case: 行点到的测试目标（lib_release_only_tests.py 的 crash_case_targets，读 RELEASE_REGISTRY_ROOT 那一份）；读不了抛它的 RegistryUnreadable。"""
    return release_only_tests_module().crash_case_targets(RELEASE_REGISTRY_ROOT[0])


def release_forced_row(row, registered, environment):
    """(行, 依据或 None)：命中重测试目标而 cargo 参数没带 release 的行换成补了 --release 的（文件头「重测试目标只许 release」），依据是补的理由；
    别的原样、依据交 None。Row.fields 不动（用时表与索引照它取名字）。弄坏开关 release-unforced：一律原样。"""
    if break_is(environment, "release-unforced"):
        return row, None
    arguments, requirement = release_only_tests_module().release_forced(row.arguments, registered=registered)
    if arguments == row.arguments:
        return row, None
    return row._replace(arguments=arguments), requirement.basis


def release_unwritten_rows(rows, environment):
    """static-check 的 RELEASE（文件头 static-check 那一段）：命中重测试目标而第五段没写 release 的行 [(行号, 变异名, 依据)]，按表的次序；
    登记表读不了抛 lib_release_only_tests.py 的 RegistryUnreadable。弄坏开关 release-unchecked：一行都不报。"""
    if break_is(environment, "release-unchecked"):
        return []
    module = release_only_tests_module()
    registered = release_registered_targets()
    unwritten = []
    for row in rows:
        requirement = module.release_requirement(row.arguments, registered=registered)
        if requirement.release_missing:
            unwritten.append((row.line_number, row.name, requirement.basis))
    return unwritten


def copied_with_path_dependencies(root, copied, environment):
    """登记的拷贝范围 copied 再加被判那棵树 root 的共用拷贝范围（文件头「拷贝范围」）：lib_repository_copy.py 的 copy_closure（带测试代码扫，
    登记的那几处经 declared 声明，它们的 path 依赖照跟）。登记的在前、照原次序，闭包算出的接在后面（已落在登记那几处里的不重复交）。
    闭包算不成抛 ValueError（ClosureRefused 是它的子类）或 OSError。
    弄坏开关 path-dependencies-not-copied：不调 copy_closure，只交登记的那几处，副本里缺 prune-server/，cargo 解析工作区就失败。"""
    if break_is(environment, "path-dependencies-not-copied"):
        return list(copied)
    library = repository_copy_module()
    closure = library.copy_closure(os.path.realpath(root), [(item, REGISTERED_COPY_RANGE_REASON) for item in copied], test_inputs=True)
    return list(copied) + [item for item in closure.entries if not library.is_covered(item, copied)]


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


def manifest_directory_of(root, arguments, environment):
    """一行 cargo 参数（`--` 之前）里 --manifest-path 指的那份 Cargo.toml 所在的目录，相对 <根>（文件头「--manifest-path」）：(目录, None)；
    没写 --manifest-path 交 ("", None)，是 <根> 那一份工作区。写坏了交 (None, 为什么)：写了不止一次、后面缺路径、是绝对路径
    （每片在自己的源码副本根起 cargo，绝对路径指不进副本）、文件名不是 Cargo.toml、落在 <根> 之外（经符号链接出去的同样算）、那份文件不在。
    相对路径照 cargo 在副本根起时的解法，相对 <根> 算。
    弄坏开关 manifest-path-ignored：一律交 ("", None)，拷贝范围不补那个包目录，单元键照 <根> 那一份工作区算。"""
    if break_is(environment, "manifest-path-ignored"):
        return "", None
    before = split_at_double_dash(arguments)[0]
    named = []
    for index, word in enumerate(before):
        if word == "--manifest-path":
            named.append(before[index + 1] if index + 1 < len(before) else "")
        elif word.startswith("--manifest-path="):
            named.append(word.partition("=")[2])
    if not named:
        return "", None
    if len(named) > 1:
        return None, f"--manifest-path 写了 {len(named)} 次，cargo 只认一份"
    path = named[0]
    if not path:
        return None, "--manifest-path 后面缺路径"
    if os.path.isabs(path):
        return None, f"--manifest-path {path} 是绝对路径（每片在自己的源码副本根起 cargo，要写相对仓根的路径）"
    normalized = os.path.normpath(path)
    if os.path.basename(normalized) != "Cargo.toml":
        return None, f"--manifest-path {path} 指的不是一份 Cargo.toml"
    tree = os.path.realpath(root)
    if os.path.commonpath((tree, os.path.realpath(os.path.join(root, normalized)))) != tree or normalized.startswith(os.pardir + os.sep):
        return None, f"--manifest-path {path} 不在仓内（落在被判那棵树 {root} 之外）"
    if not os.path.isfile(os.path.join(root, normalized)):
        return None, f"--manifest-path {path} 指的文件不在"
    directory = os.path.dirname(normalized)
    return ("" if directory in ("", os.curdir) else directory), None


def copied_with_manifest_directories(root, copied, rows, environment):
    """拷贝范围（已跟过 path 依赖的 copied）再补变异行 --manifest-path 指的包目录（文件头「拷贝范围」「--manifest-path」）：
    那个包目录的 Cargo.toml 已在范围里的不动（仓根 prune-server/ 这一类，验证包经 path 依赖已拷进来），不在的把那个目录补进来，
    连同它底下各份 Cargo.toml 经 path 依赖再指到的（跟法同 copied_with_path_dependencies）。
    --manifest-path 写坏了（manifest_directory_of 交的为什么：不在仓内、不是 Cargo.toml、文件不在……）抛 ValueError，带变异表的行号与变异名；
    补进来的包的 path 依赖指到树之外的同样抛 ValueError。弄坏开关 manifest-path-ignored 下 manifest_directory_of 一律交 <根>，一个都不补。"""
    result, added = list(copied), False
    for row in rows:
        directory, why = manifest_directory_of(root, row.arguments, environment)
        if why is not None:
            raise ValueError(f"{TABLE}:{row.line_number} {row.name}：{why}")
        if directory and not is_copied_into_each_shard(os.path.join(directory, "Cargo.toml"), result):
            result.append(directory)
            added = True
    if added and not break_is(environment, "path-dependencies-not-copied"):
        extra = repository_copy_module().path_dependency_entries(os.path.realpath(root), result)
        result += [item for item in sorted(extra) if item not in result]
    return result


def unit_keys_under_manifest(tree_root, directory):
    """--manifest-path 指的包（directory，相对 tree_root、不空）那一份单元键（文件头「--manifest-path」）：lib_mutation_units.py 的 UnitKeys 以包目录为根算，
    那个包自己是工作区根，被改坏的单元、点名的测试、闭包与依赖版本行都照它的源码与它自己的 Cargo.lock 算；进出的路径换成相对 tree_root：
    问进来的路径去掉 <directory>/，交出去的单元（被改坏的、闭包成员、执行过的）的文件接上 <directory>/，库里的组成清单、牵连面与作废原因因此都写仓根起的路径。
    被改的文件不在那个包底下的交拒算那一句（单元键照那个包的源码算，包外的文件不在它的单元里）。KeyContext 与记覆盖的工作进程（record_coverage）各用一份。
    类写在函数里：本文件有单元进判法单元的闭包，模块级的类头在 import 时执行、会进闭包（文件头「判法单元」），写在这里不动判法单元指纹。"""
    unit_keys = mutation_units.UnitKeys(os.path.join(tree_root, directory))
    prefix = directory + os.sep

    def inside(path):
        return path[len(prefix):] if path.startswith(prefix) else None

    def outward(reference):
        # 数据文件与数据目录的单元拿路径当限定名（lib_mutation_units.py 的 mutated_units、lib_function_units.py 的数据单元）：限定名跟着文件一起换
        name = prefix + reference.name if reference.name == reference.file else reference.name
        return reference._replace(file=prefix + reference.file, name=name)

    def inward(reference):
        relative = inside(reference.file)
        return reference._replace(file=relative, name=relative if reference.name == reference.file else reference.name)

    class ManifestUnitKeys:
        root = tree_root   # row_view 照 <根> 起的路径打开被改的文件

        @property
        def workspace(self):
            return unit_keys.workspace

        def touches_stripped_text(self, path, text, start, end):
            relative = inside(path)
            return relative is not None and unit_keys.touches_stripped_text(relative, text, start, end)

        def mutated_units(self, path, original):
            relative = inside(path)
            if relative is None:
                return None, f"被改坏的文件 {path} 不在 --manifest-path 指的包 {directory}{os.sep} 底下（这一行的单元键照那个包的源码算）"
            references, refusal = unit_keys.mutated_units(relative, original)
            return (None, refusal) if refusal is not None else ([outward(reference) for reference in references], None)

        def resolve_test(self, arguments, expected, view=None):
            return unit_keys.resolve_test(arguments, expected, view=view)

        def features_of(self, arguments, target_id):
            return unit_keys.features_of(arguments, target_id)

        def precompute_closures(self, requests, workers, labels=None):
            return unit_keys.precompute_closures(requests, workers, labels)

        def closure(self, unit, target_id, arguments=None):
            info = unit_keys.closure(unit, target_id, arguments)
            return info._replace(members=[outward(member) for member in info.members])

        def reference_dependencies(self, references):
            return unit_keys.reference_dependencies([inward(reference) for reference in references if inside(reference.file) is not None])

        def current_fingerprints(self, references):
            current, within = {}, []
            for reference in references:
                identity = (reference.kind, reference.file, reference.name)
                current[identity] = set()   # 不在那个包底下的文件照「文件不在」交空集合（lib_mutation_units.py 的 current_fingerprints）
                if inside(reference.file) is not None:
                    within.append((identity, inward(reference)))
            found = unit_keys.current_fingerprints([inner for _identity, inner in within])
            for identity, inner in within:
                current[identity] = found.get((inner.kind, inner.file, inner.name), set())
            return current

        def executed_units(self, export, work_root):
            executed, unplaced = unit_keys.executed_units(export, os.path.join(work_root, directory))
            return [outward(reference) for reference in executed], unplaced

    return ManifestUnitKeys()


def unit_keys_for_directory(tree_root, directory):
    """包目录（manifest_directory_of 交的，相对 tree_root）那一份单元键：空串是 tree_root 那一份工作区（lib_mutation_units.py 的 UnitKeys），
    别的是 unit_keys_under_manifest。"""
    return mutation_units.UnitKeys(tree_root) if directory == "" else unit_keys_under_manifest(tree_root, directory)


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
    """派活之前一个工作进程都不起就能判的那几样，crates-mutation-replay（prechecks）与门禁 code-source-discipline（static-check）都调这一份。回 {类别: [明细]}，类别照 ROW_PROBLEM_KINDS：
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


# ── 判法单元指纹的磁盘缓存（文件头「判法单元」末段）──────────────────────────────────────────────

JUDGING_CACHE_FORM = "singlefs-judging-units-cache-v1"
JUDGING_CACHE_DIRECTORY_NAME = "singlefs-judging-units-cache"
JUDGING_CACHE_DIRECTORY_VARIABLE = "GATE_JUDGING_CACHE_DIRECTORY"   # 设了就放这里（自证用）；不设照 judging_cache_directory
JUDGING_CACHE_KEPT = 16   # 目录里只留最近写的这么多份（一份 = 一种弄坏开关与入口组合）
JUDGING_CACHE_STATS = {"computed": 0, "reused": 0}   # 本进程里判法单元现算了几趟、取了几趟磁盘缓存（自证数它）
SCRIPT_NAME_SUFFIXES = (".py", ".sh", ".bash")
WRITE_MODE_CHARACTERS = frozenset("wax+")


def judging_cache_directory(environment):
    configured = (environment.get(JUDGING_CACHE_DIRECTORY_VARIABLE) or os.environ.get(JUDGING_CACHE_DIRECTORY_VARIABLE) or "").strip()
    if configured:
        return configured
    if environment.get("GATE_CROSS_RUN_TMPDIR"):
        return os.path.join(environment["GATE_CROSS_RUN_TMPDIR"], JUDGING_CACHE_DIRECTORY_NAME)
    # 没设跨轮目录放本仓的 git common-dir 底下（与判定库一处；git worktree 里是主仓那一份，与主仓共用；lib_function_units.py 的 git_common_directory_of），
    # 不随这一趟的 TMPDIR 走、跑完不留在那一趟的临时目录里；不在 git 仓里才放 TMPDIR 底下
    common = mutation_units.function_units().git_common_directory_of(STAGE_REPOSITORY)
    return os.path.join(common if common is not None else (environment.get("TMPDIR") or tempfile.gettempdir()), JUDGING_CACHE_DIRECTORY_NAME)


def judging_cache_file(environment, whole_file):
    """这一种算法（入口表、整份原文开关、两个变量家族里的弄坏开关）的缓存文件；同一种算法一份，内容对不上就覆盖它。"""
    breaks = [f"{name}={environment.get(name, '')}|{os.environ.get(name, '')}" for name in ("FUNCTION_UNITS_BREAK",) + BREAK_VARIABLES]
    identity = "\n".join([JUDGING_CACHE_FORM, *judging_entries(environment), f"whole_file={whole_file}", *breaks])
    return os.path.join(judging_cache_directory(environment), hashlib.sha256(identity.encode("utf-8")).hexdigest() + ".json")


def read_value(kind, path):
    """一种读在此刻的值：open 是文件内容的 sha256（打不开是 None），isfile、isdir、exists 是布尔，listdir 是名字表的 sha256（列不了是 None）。
    用的是没被记读的包装换掉的真函数（io.open、genericpath、posix.listdir），所以在记读的当中调它不会记到自己。"""
    import genericpath
    import io
    import posix
    if kind == "open":
        try:
            with io.open(path, "rb") as handle:
                return hashlib.sha256(handle.read()).hexdigest()
        except OSError:
            return None
    if kind == "listdir":
        try:
            names = posix.listdir(path)
        except OSError:
            return None
        return hashlib.sha256("\n".join(sorted(names)).encode("utf-8", "surrogateescape")).hexdigest()
    return {"isfile": genericpath.isfile, "isdir": genericpath.isdir, "exists": genericpath.exists}[kind](path)


def code_digests():
    """现算判法单元指纹的代码本身（本文件、lib_mutation_units.py、lib_function_units.py）此刻的内容摘要，路径相对这份文件所在的仓：[[相对路径, sha256], …]。
    判法在 repository 的拷贝上算时（自证），执行的代码仍是这一份，不在读过的清单里（清单只记 repository 里的路径）。"""
    paths = {os.path.realpath(__file__), os.path.realpath(MUTATION_UNITS_PATH), os.path.realpath(mutation_units.FUNCTION_UNITS_PATH)}
    return [[os.path.relpath(path, STAGE_REPOSITORY), read_value("open", path)] for path in sorted(paths)]


def script_names_digest(repository):
    """仓里脚本文件（.py、.sh、.bash，跳过 .git、target 这类）的相对路径名单的 sha256：闭包按名字找脚本、走目录，新增或删掉一份脚本会改变找到的东西。"""
    skipped = mutation_units.function_units().SKIPPED_DIRECTORIES
    names = []
    for directory, subdirectories, files in os.walk(repository):
        subdirectories[:] = sorted(name for name in subdirectories if name not in skipped)
        names += [os.path.relpath(os.path.join(directory, name), repository) for name in files if name.endswith(SCRIPT_NAME_SUFFIXES)]
    return hashlib.sha256("\n".join(sorted(names)).encode("utf-8", "surrogateescape")).hexdigest()


class ReadRecorder:
    """算判法单元的这一趟里，把对仓内路径的读都记下来：打开读的每份文件（builtins.open，不带 w、a、x、+）、posixpath.isfile / isdir / exists、os.listdir。
    reads 是 {(种类, 相对仓根的路径): 读的那一刻的值}（值的写法见 read_value）；同一处读两次读到不同的值（算的当中文件被改了）记进 unstable。
    出了 with 就还原；没有别的线程同时在读：判法单元在主线程里一趟算完。"""

    def __init__(self, repository):
        self.root = os.path.abspath(repository)
        self.reads, self.unstable = {}, set()
        self._saved = None

    def relative(self, path):
        if not isinstance(path, (str, os.PathLike)):
            return None
        absolute = os.path.abspath(os.fspath(path))
        if not isinstance(absolute, str) or not (absolute == self.root or absolute.startswith(self.root + os.sep)):
            return None
        return os.path.relpath(absolute, self.root).replace(os.sep, "/")

    def note(self, kind, path, relative=None):
        relative = relative or self.relative(path)
        if relative is None:
            return
        value = read_value(kind, os.path.join(self.root, relative))
        if self.reads.setdefault((kind, relative), value) != value:
            self.unstable.add((kind, relative))

    def __enter__(self):
        import builtins
        import genericpath
        import io
        import posix
        import posixpath
        recorder = self

        def recording_open(file, mode="r", *arguments, **options):
            if isinstance(mode, str) and not (WRITE_MODE_CHARACTERS & set(mode)):
                recorder.note("open", file)
            return io.open(file, mode, *arguments, **options)

        def recording(kind, real):
            def wrapper(path=".", *arguments):
                recorder.note(kind, path)
                return real(path, *arguments)
            return wrapper
        self._saved = (builtins.open, posixpath.isfile, posixpath.isdir, posixpath.exists, os.listdir)
        builtins.open = recording_open
        posixpath.isfile, posixpath.isdir, posixpath.exists = (recording("isfile", genericpath.isfile), recording("isdir", genericpath.isdir),
                                                               recording("exists", genericpath.exists))
        os.listdir = recording("listdir", posix.listdir)
        return self

    def __exit__(self, *exception):
        import builtins
        import posixpath
        builtins.open, posixpath.isfile, posixpath.isdir, posixpath.exists, os.listdir = self._saved
        return False


def read_judging_cache(path):
    """缓存文件读出来的 dict（形态对得上才交），读不了、不是这一版的写法交 None。"""
    try:
        with open(path, encoding="utf-8") as handle:
            entry = json.load(handle)
        if (entry["form"] != JUDGING_CACHE_FORM or not isinstance(entry["key"], str) or not isinstance(entry["names"], str)
                or not all(isinstance(item, list) and len(item) == 2 for item in entry["code"])
                or not all(isinstance(item, list) and len(item) == 5 and all(isinstance(field, str) for field in item) for item in entry["references"])
                or not all(isinstance(item, list) and len(item) == 3 and isinstance(item[0], str) and isinstance(item[1], str) for item in entry["reads"])):
            return None
    except (OSError, ValueError, KeyError, TypeError):
        return None
    return entry


def judging_cache_valid(entry, repository):
    """存的那一趟读过的每一处，此刻在 repository 里现读都是同一个值（含脚本名单的摘要）。弄坏开关 judging-cache-unverified 不走这里。"""
    if entry["names"] != script_names_digest(repository) or entry["code"] != code_digests():
        return False
    return all(read_value(kind, os.path.join(repository, relative)) == value for kind, relative, value in entry["reads"])


def write_judging_cache(path, entry):
    """原子写、只留最近 JUDGING_CACHE_KEPT 份；写不成（目录不可写、盘满）报到 stderr、不影响这一趟的结果。"""
    try:
        mutation_units.write_atomically(path, json.dumps(entry, ensure_ascii=False, sort_keys=True))
        directory = os.path.dirname(path)
        written = sorted((name for name in os.listdir(directory) if name.endswith(".json")), key=lambda name: os.path.getmtime(os.path.join(directory, name)), reverse=True)
        for name in written[JUDGING_CACHE_KEPT:]:
            with contextlib.suppress(OSError):
                os.unlink(os.path.join(directory, name))
    except OSError as error:
        print(f"  · 判法单元指纹没存进磁盘缓存（{path}：{error}），这一趟照现算的用", file=sys.stderr)


def judging_units_cached(environment, repository, whole_file):
    """mutation_units.judging_units 的磁盘缓存版：→ (键或 None, UnitReference 列表, 拒算列表)，与不缓存时逐项相同。
    有存的、而且存的那一趟读过的每一处现读都没变才取（只看内容，不看修改时刻、不设时限）；否则现算，算的当中读过什么都记下来，
    键算得出（没有拒算）、算的当中读过的文件没被改过才存。弄坏开关 judging-cache-unverified：有存的就一直沿用、不核读过的内容。"""
    path = judging_cache_file(environment, whole_file)
    entry = read_judging_cache(path)
    if entry is not None and (break_is(environment, "judging-cache-unverified") or judging_cache_valid(entry, repository)):
        JUDGING_CACHE_STATS["reused"] += 1
        return entry["key"], [mutation_units.UnitReference(*item) for item in entry["references"]], []
    names_before, code_before = script_names_digest(repository), code_digests()
    with ReadRecorder(repository) as recorder:
        key, references, refusals = mutation_units.judging_units(repository, judging_entries(environment), whole_file=whole_file)
    JUDGING_CACHE_STATS["computed"] += 1
    unchanged = not recorder.unstable and names_before == script_names_digest(repository) and code_before == code_digests() and all(
        read_value(kind, os.path.join(recorder.root, relative)) == value for (kind, relative), value in recorder.reads.items())
    if key is not None and not refusals and unchanged:
        write_judging_cache(path, dict(form=JUDGING_CACHE_FORM, key=key, names=names_before, code=code_before, references=[list(reference) for reference in references],
                                       reads=[[kind, relative, value] for (kind, relative), value in sorted(recorder.reads.items())]))
    return key, references, refusals


def judging_units(environment, repository=None):
    """(判法单元指纹, 单元清单)：文件头「判法单元」那几个入口函数的闭包合起来，键只随闭包里单元的（种类，去注释指纹）变。
    repository 是判法所在那一份仓（不给取 JUDGING_ROOT；自证拿临时拷贝改了判法再算）。弄坏开关 judge-units-whole-file：指纹换成闭包碰到的那几份文件的整份原文。
    同一个进程里闭包碰到的那几份文件都没动过（修改时刻与大小）就沿用上一次算的（自证一趟起几十次 run_stage）；跨进程另有磁盘缓存（judging_units_cached，文件头「判法单元」末段）。"""
    repository = repository or JUDGING_ROOT[0]
    whole_file = break_is(environment, "judge-units-whole-file")
    function_units_excluded = break_is(environment, "judging-function-units-excluded")
    entries = judging_entries(environment)   # 进程内缓存按入口分开：弄坏开关 judging-entries-* 换了入口，不沿用没开关那一趟算的
    cached = _JUDGING_CACHE.get((repository, whole_file, function_units_excluded, entries))
    if cached is not None:
        key, references, stats = cached
        if all(_file_stat(os.path.join(repository, path)) == stat for path, stat in stats.items()):
            return key, references
    key, references, refusals = judging_units_cached(environment, repository, whole_file)
    if key is not None and function_units_excluded:
        # 弄坏开关（只给证红用）：lib_function_units.py 的单元不进指纹（D13 已定项 17 文件级排除用在这里的样子）
        references = [reference for reference in references if reference.file != "scripts/gate/lib_function_units.py"]
        key = mutation_units.key_of_lines([("form", ["judging-function-units-excluded"]), ("unit", [f"{reference.kind}\t{reference.fingerprint}" for reference in references])])
    if key is None:
        raise JudgingUnitsRefused("；".join(refusals[:3]) + (f" 等 {len(refusals)} 处" if len(refusals) > 3 else ""))
    _JUDGING_CACHE[(repository, whole_file, function_units_excluded, entries)] = (key, references, {path: _file_stat(os.path.join(repository, path)) for path in {reference.file for reference in references}})
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


def refuse_overrides_outside_service(environment):
    """只给自证换的变量（文件头「判定库」）不在临时 prune-server 底下设了：抛 InputManifestError、点名设了的几个，一条都不跑。
    弄坏开关 run-overrides-honored-outside-service 下放过这两个。"""
    admission = admission_module()
    if under_temporary_prune_server(environment):
        return
    names = ([] if break_is(environment, "run-overrides-honored-outside-service")
             else ["GATE_MUTATION_RUN_RECORDS_DIRECTORY", "GATE_MUTATION_MULTI_HOST_CONFIG"])
    found = [f"{name}（「{environment[name].strip()}」）" for name in names if environment.get(name, "").strip()]
    if found:
        raise admission.InputManifestError(f"不在临时 prune-server 底下设了 {'、'.join(found)}：这几个只给自证换（运行记录、多机配置），"
                                           "平常跑 unset 它们（库根是被判那棵树的 git common-dir）")


def records_home(root, environment):
    """判定库放在哪个目录底下（文件头「判定库」）：被判那棵树的 git common-dir；
    取不到抛 InputManifestError（弄坏开关 library-outside-git-tolerated 下拿被判的根当库根）。先过 refuse_overrides_outside_service。"""
    refuse_overrides_outside_service(environment)
    admission = admission_module()
    common_directory = admission.git_common_directory(root)
    if not common_directory:
        if break_is(environment, "library-outside-git-tolerated"):
            return os.path.abspath(root)
        raise admission.InputManifestError(f"{root} 不是 git 工作树（取不到 git common-dir），判定库没有名字")
    return common_directory


def under_temporary_prune_server(environment):
    """environment 连的是 --with-temporary-service 那样起的临时 prune-server：判法同客户端的 selftest_service_problem（临时服务的标记设了、与 socket 相同、
    TCP 地址设了、socket 在），判的是给的这一份环境（客户端那一份只看本进程的 os.environ）。"""
    client = mutation_units.Library.client()
    marked = environment.get(client.TEMPORARY_SERVICE_VARIABLE, "")
    return (bool(marked) and marked == environment.get(client.SOCKET_VARIABLE, "") and bool(environment.get(client.TCP_ADDRESS_VARIABLE, ""))
            and os.path.exists(marked))


def library_directory(root, environment):
    return os.path.join(records_home(root, environment), LIBRARY_DIRECTORY_NAME)


def open_library(root, environment):
    return mutation_units.Library(library_directory(root, environment), RECORD_HEADER)


def run_records_directory(root, environment):
    """用时表与占盘记录放哪（文件头「判定库」末段）：GATE_MUTATION_RUN_RECORDS_DIRECTORY 设了取它，没设取多机配置 NFS_SHARED_ROOTS 里
    不含本仓、也不在本仓之内的几段里排在最前的一段；底下再接 crates-mutation-run-records<库的命名空间>（库根在临时目录底下的自证一趟一份，别处两台同一份）。
    配置里没有这一项抛 InputManifestError（不补缺省、不落到 git common-dir）。目录名写在函数体里、不做模块级常量：模块级的赋值在判法单元的闭包里。"""
    admission = admission_module()
    shared_root = environment.get("GATE_MUTATION_RUN_RECORDS_DIRECTORY", "").strip()
    if not shared_root:
        configuration = admission.multi_host_configuration_file(STAGE_REPOSITORY, environment)
        repositories = [os.path.realpath(path) for path in (root, STAGE_REPOSITORY)]

        def overlaps(candidate):
            real = os.path.realpath(candidate)
            return any(real == repository or real.startswith(repository.rstrip(os.sep) + os.sep) or repository.startswith(real.rstrip(os.sep) + os.sep)
                       for repository in repositories)

        candidates = [path for path in admission.configuration_value(configuration, "NFS_SHARED_ROOTS").split(":") if os.path.isabs(path) and not overlaps(path)]
        if not candidates:
            raise admission.InputManifestError(f"多机配置 {configuration} 里没有不含本仓的 NFS_SHARED_ROOTS（两台同一绝对路径的共享根）：用时表与占盘记录没处放")
        shared_root = candidates[0]
    namespace = mutation_units.Library.client().namespace_of(library_directory(root, environment), environment)
    return os.path.join(shared_root, "crates-mutation-run-records" + namespace)


def snapshot_of(base, copied):
    """base 底下拷贝范围里（跳过 target 与 crates/mutations.tsv）每个文件的 {路径: sha256}：只给弄坏开关 baseline-by-tree-snapshot 用（基线按整树快照比），
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
    """算闭包开几个子进程：GATE_MUTATION_KEY_WORKERS 给了（正整数）照它；不然取每台默认重活线程数（scripts/hosts/heavy_threads.py）。
    登记文件读不了抛 HeavyThreadDefaultError。"""
    requested = environment.get("GATE_MUTATION_KEY_WORKERS", "").strip()
    if re.fullmatch(r"[1-9][0-9]*", requested):
        return int(requested)
    return int(default_heavy_threads(environment))


def row_view(keys, row):
    """这一行的被改坏的单元与点名测试的闭包取哪个视图（lib_mutation_units.py 文件头「视图」）：lib_function_units.py 的 FORMAL_VERIFICATION 或 PRODUCT_BEHAVIOR。
    原文里有产品行为视图剥掉的字（UnitKeys.touches_stripped_text）的取形式化视图，别的（原文不是 .rs、命中不是恰好 1 次、读不了、没有被剥掉的字）取产品行为视图；
    弄坏开关 specification-lines-product-view 下一律取产品行为视图。自己按原文判，不靠 mutated_units 与 resolve_test 谁先谁后
    （resolve_test 的 view 不给，是照紧挨着的上一次 mutated_units：调用次序一调换，视图就悄悄变）。"""
    lfu = mutation_units.function_units()
    if not row.path.endswith(".rs") or mutation_units.break_is("specification-lines-product-view"):
        return lfu.PRODUCT_BEHAVIOR
    try:
        with open(os.path.join(keys.root, row.path), "rb") as handle:
            text = handle.read().decode("utf-8", "surrogateescape")
    except OSError:
        return lfu.PRODUCT_BEHAVIOR
    if text.count(row.old) != 1:
        return lfu.PRODUCT_BEHAVIOR
    start = text.find(row.old)
    return lfu.FORMAL_VERIFICATION if keys.touches_stripped_text(row.path, text, start, start + len(row.old)) else lfu.PRODUCT_BEHAVIOR


def row_unit_keys(root, row, environment, keys_by_directory):
    """(这一行的单元键, None) 或 (None, 拒算那一句)：没写 --manifest-path 的是 <根> 那一份，写了的是那个包自己的一份（unit_keys_under_manifest）；
    --manifest-path 写坏了（manifest_directory_of 交的为什么）这一行拒算。keys_by_directory 是调用方的缓存（包目录 → 单元键，<根> 那一份的包目录是 ""），没有就现建。"""
    directory, why = manifest_directory_of(root, row.arguments, environment)
    if why is not None:
        return None, f"cargo 参数的 --manifest-path 认不出：{why}"
    if directory not in keys_by_directory:
        keys_by_directory[directory] = mutation_units.UnitKeys(root) if directory == "" else unit_keys_under_manifest(root, directory)
    return keys_by_directory[directory], None


class KeyContext:
    """一棵树这一趟算键要的几样（文件头「行键」）：判法单元指纹、工具链行、单元键（UnitKeys）、依赖版本行按用到的包缓存。"""

    def __init__(self, root, environment, judging_repository=None):
        self.root, self.environment = root, environment
        self.judging, self.judging_references = judging_units(environment, judging_repository)
        self.toolchain_lines = [] if break_is(environment, "rowkeywithouttoolchain") else key_toolchain_lines(root, environment)
        self.toolchain = lines_key("toolchain", self.toolchain_lines)
        self.keys = mutation_units.UnitKeys(root)
        self._manifest_keys = {"": self.keys}   # 包目录 → 单元键（row_unit_keys：<根> 那一份是 ""，别的是 unit_keys_under_manifest）
        self._manifest_toolchains = {}   # 包目录 → (工具链键, 工具链行)（toolchain_of）
        self._dependencies = {}
        self.closures = {}
        self.collisions = []   # 撞键的几组行号（refuse_colliding_row_keys）
        self.gpu_route_lines, self.gpu_route_configuration = [], None   # 补了 gpu 特性的行（行号, 补的特性）与那份多机配置（gpu_route_rows）
        self.gpu_route_unbuilt = []   # 补了 gpu 特性点名的测试就不编、照原样留着的行（行号, 为什么）
        # 命中重测试目标、补了 --release 的行（行号, 依据）与那一份登记的测试目标（plan_rows、gpu_route_rows；文件头「重测试目标只许 release」）
        self.release_lines, self.release_registered = [], None

    def keys_for(self, row):
        """(这一行的单元键, None) 或 (None, 拒算那一句)：单元键与工具链行按行的包目录取（文件头「--manifest-path」，工具链行见 toolchain_of）。
        没写 --manifest-path 的是 <根> 那一份（self.keys），写了的是那个包自己的一份（unit_keys_under_manifest，按包目录缓存）；
        --manifest-path 写坏了（manifest_directory_of 交的为什么）这一行拒算。判法在 row_unit_keys（static-check 可达性用同一份）。"""
        return row_unit_keys(self.root, row, self.environment, self._manifest_keys)

    def toolchain_of(self, row):
        """(工具链键, 工具链行)：这一行起 cargo 时作数的那一份（文件头「--manifest-path」）。没写 --manifest-path 的照 <根> 那一份；
        写了的，那个包是它自己的工作区根，cargo 读它的 Cargo.toml 里 [profile] 这几段、不读 <根> 那一份，配置文件照起 cargo 的当前目录（副本根）往上找：
        取 <根> 那一份的行，manifest 那几行换成那个包的 Cargo.toml 里的（key_toolchain_lines 在包目录上算，只取 manifest 行），按包目录缓存。
        --manifest-path 写坏了的（那一行拒算）与弄坏开关 rowkeywithouttoolchain 下交 <根> 那一份。"""
        directory, why = manifest_directory_of(self.root, row.arguments, self.environment)
        if why is not None or directory == "" or break_is(self.environment, "rowkeywithouttoolchain"):
            return self.toolchain, self.toolchain_lines
        if directory not in self._manifest_toolchains:
            package_lines = [line for line in key_toolchain_lines(os.path.join(self.root, directory), self.environment) if line.startswith("manifest\t")]
            lines = [line for line in self.toolchain_lines if not line.startswith("manifest\t")] + package_lines
            self._manifest_toolchains[directory] = (lines_key("toolchain", lines), lines)
        return self._manifest_toolchains[directory]

    def prepare(self, rows):
        """先把这一趟每一行点名测试的闭包算好（并行，进程数见 key_workers）；components 之后照缓存取。回算了几个闭包。
        按行的单元键分组（keys_for：<根> 那一份与各个 --manifest-path 指的包各一组）各算各的；几组里闭包算不成的并成一个 ClosureRefused，别的组照算完再抛。"""
        groups = collections.OrderedDict()   # id(单元键) → (单元键, requests, labels)
        for row in rows:
            keys, keys_refusal = self.keys_for(row)
            if keys_refusal is not None:
                continue
            _mutated, refusal = keys.mutated_units(row.path, row.old)
            test, test_refusal = ((None, refusal) if refusal is not None
                                  else keys.resolve_test(row.arguments, row.expected, view=row_view(keys, row)))
            if test_refusal is None:
                _keys, requests, labels = groups.setdefault(id(keys), (keys, [], []))
                requests.append((test.unit, test.target, mutation_units.parse_cargo_arguments(row.arguments)))
                labels.append(f"{TABLE}:{row.line_number} {row.name}")
        computed, failures = 0, []
        for keys, requests, labels in groups.values():
            try:
                computed += keys.precompute_closures(requests, key_workers(self.environment), labels)
            except mutation_units.ClosureRefused as error:
                failures += error.failures
        if failures:
            raise mutation_units.ClosureRefused(failures)
        return computed

    def dependency(self, lines):
        """(依赖版本行的键, 那几行)：行是点名测试的闭包与被改坏的单元点名的外部 crate 收到的（lib_mutation_units.py 文件头「依赖版本行」），按行缓存。"""
        lines = tuple(sorted(lines))
        if lines not in self._dependencies:
            self._dependencies[lines] = (lines_key("dependency", list(lines)), list(lines))
        return self._dependencies[lines]

    def components(self, row):
        """这一行的 Components：键算得出交齐全的一份，算不出（拒算）row_key 是 None、refusal 写为什么。"""
        toolchain, _toolchain_lines = self.toolchain_of(row)
        keys, refusal = self.keys_for(row)
        if refusal is not None:
            return Components(None, refusal, None, None, None, None, None, toolchain, None, None, None, self.judging)
        mutated, refusal = keys.mutated_units(row.path, row.old)
        if refusal is not None:
            return Components(None, f"被改坏的单元：{refusal}", None, None, None, None, None, toolchain, None, None, None, self.judging)
        test, refusal = keys.resolve_test(row.arguments, row.expected, view=row_view(keys, row))
        if refusal is not None:
            return Components(None, refusal, mutated, None, None, None, None, toolchain, None, None, None, self.judging)
        arguments = mutation_units.parse_cargo_arguments(row.arguments)
        closure = keys.closure(test.unit, test.target, arguments)
        if closure.key is None:
            return Components(None, f"点名测试 {test.full_name} 的闭包拒算（{closure.refusals[0]}）", mutated, test.unit, test.full_name,
                              None, None, toolchain, None, None, None, self.judging)
        self.closures[closure.key] = closure
        mutated_dependencies, dependency_refusals = keys.reference_dependencies(mutated)
        if dependency_refusals:
            return Components(None, f"被改坏的单元的依赖版本行拒算（{dependency_refusals[0]}）", mutated, test.unit, test.full_name,
                              None, None, toolchain, None, None, None, self.judging)
        dependency, _lines = self.dependency(set(closure.dependencies) | set(mutated_dependencies))
        # 被改坏的单元只带种类与指纹：路径、名字、行号不进键（D13（验证路线） 已定项 17）；两份内容相同的函数各改一处、点名同一条测试的两行
        # 算出同一把键，由 refuse_colliding_row_keys 判成拒算（文件头「行键」）
        mutated_lines = [] if break_is(self.environment, "mutated-unit-ignored") else [f"{reference.kind}\t{reference.fingerprint}" for reference in mutated]
        # 基线键与覆盖号带点名测试所在的包与测试目标：两个目标里各有一条字面相同的测试，基线与覆盖不互相顶替（文件头「判定库」）。
        # 弄坏开关 baseline-key-without-target：不带
        test_target = [] if break_is(self.environment, "baseline-key-without-target") else [
            ("test-target", [f"{keys.workspace.targets[test.target].crate.package}\t{test.target}"])]
        test_key = "" if break_is(self.environment, "test-closure-ignored") else closure.key
        groups = [("form", [ROW_KEY_FORM]), ("judging", [self.judging]), ("mutated", mutated_lines),
                  ("original", [mutation_units.escape_field(row.old)]), ("replacement", [mutation_units.escape_field(row.new)]),
                  ("test", [test_key]), ("switches", arguments.switches), ("features", sorted(keys.features_of(arguments, test.target))),
                  ("toolchain", [toolchain]), ("dependency", [dependency])]
        row_key = mutation_units.key_of_lines(groups)
        baseline_key = mutation_units.key_of_lines([("form", [BASELINE_KEY_FORM]), ("judging", [self.judging]), ("test-unit", [test.unit.fingerprint])] + test_target
                                                   + [("switches", arguments.switches), ("features", sorted(keys.features_of(arguments, test.target))),
                                                      ("toolchain", [toolchain]), ("dependency", [dependency])])
        # 覆盖记录里的 executed 由 executed_units 落单元（lib_function_units.py 给行号区间）：判法单元指纹变了（含切单元的改动）旧记录不能再认，覆盖号带判法单元指纹
        coverage_judging = [] if break_is(self.environment, "coverage-id-without-judging") else [("judging", [self.judging])]
        coverage_id = mutation_units.key_of_lines([("form", [COVERAGE_KEY_FORM])] + coverage_judging + [("test-unit", [test.unit.fingerprint])] + test_target
                                                  + [("switches", arguments.switches), ("features", sorted(keys.features_of(arguments, test.target))),
                                                     ("toolchain", [toolchain])])
        return Components(row_key, None, mutated, test.unit, test.full_name, closure.key, arguments.switches, toolchain, dependency,
                          baseline_key, coverage_id, self.judging)

    def persist(self, library, components_by_line):
        """组成清单进库（判法单元、每一份点名测试的闭包、工具链行、依赖版本行），新见到的单元并进 units/。只在 run_stage 里调（plan 只读）。"""
        library.put_composition(self.judging, self.judging_references)
        library.put_composition(self.toolchain, lines=self.toolchain_lines)
        for key, lines in self._manifest_toolchains.values():
            library.put_composition(key, lines=lines)
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
        return "environment", f"工具链行：{mutation_units.summarize_changes(mutation_units.changed_lines(library, previous.get('toolchain'), context.toolchain_of(row)[1]))}"
    if previous.get("dependency") != components.dependency:
        return "environment", "依赖版本行变了"
    return "unexplained", "键对不上而几样组成的号都与上一次相同（cargo 参数里的开关或特性改了）"


def row_decision(library, components, row, context, previous=None, record=None):
    """这一行这一趟复用还是现判（文件头「复用判据」）：(Decision, 记录)。拒算的 column 是 refused。"""
    if components.row_key is None:
        return Decision(False, "refused", components.refusal), None
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
    不按整树快照比：同一个文件里加一个没人调的函数、改别的测试，基线照样复用。弄坏开关 baseline-by-tree-snapshot：只按整树快照比。"""
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
    """上一次跑的秒数；没有记过、读不了交 None。用时表是运行记录（文件头「判定库」末段），timings_directory 是运行记录目录底下的 timings/。"""
    try:
        with open(os.path.join(timings_directory, content_hash), encoding="utf-8", errors="surrogateescape") as handle:
            record = mutation_units.parse_record(handle.read())
    except OSError:
        return None
    try:
        return float(record.get("seconds_running", ""))
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


def row_libtest_selection(arguments):
    """一行 cargo 参数交给 libtest 的选择：(过滤词, 带不带 --exact, --skip=<名> 的那几个名)，写法同 baseline_arguments（`--` 之后不以 - 起头的是过滤词）。"""
    after = split_at_double_dash(arguments)[1]
    return ([word for word in after if not word.startswith("-")], "--exact" in after,
            [word.split("=", 1)[1] for word in after if word.startswith("--skip=")])


def libtest_selects(full_name, filters, exact, skips):
    """libtest 按这几样选不选得中这条用例：带 --exact 时名字要逐字相同，不带时是子串；没有过滤词就是全部，--skip 的照同一种比法剔掉。"""
    def matches(pattern):
        return full_name == pattern if exact else pattern in full_name
    if filters and not any(matches(word) for word in filters):
        return False
    return not any(matches(word) for word in skips)


def row_baseline_output(row, output, environment):
    """一批基线的输出里这一行看得见的那一份（文件头「基线」）：只留这一行自己的过滤词选得中的用例的结果行，别的结果行去掉。
    一批的过滤词是这一趟拼进来的各行的并集，拼进了哪几行随这一趟选了哪几行变；这样一行的基线结局与它单跑时相同，点名单跑与整张表对同一行判同一个结论。
    弄坏开关 baseline-batch-composition：照整批的输出判。结果词被隔开的结果行先接回一行再按名字选（joined_result_lines）。"""
    if break_is(environment, "baseline-batch-composition"):
        return output
    filters, exact, skips = row_libtest_selection(row.arguments)
    kept = []
    for line in joined_result_lines(output, environment).split("\n"):
        parsed = libtest_result_lines.result_line(line)
        if parsed is not None and not libtest_selects(parsed.full_name, filters, exact, skips):
            continue
        kept.append(line)
    return "\n".join(kept)


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
    """(派变异的次序, 前移了几条)：上一次用时不短于 long_row_seconds 的行先派、按用时从长到短，其余照按 crate 归堆的次序。"""
    if break_is(environment, "timingsignored"):
        return rows_grouped_by_crate(rows), 0
    long_rows = []
    for row in rows:
        timing = read_timing(timings_directory, row_content_hash(row))
        if timing is not None and timing >= long_row_seconds:
            long_rows.append((timing, row))
    long_rows.sort(key=lambda item: (-item[0], item[1].line_number))
    moved_lines = {row.line_number for _seconds, row in long_rows}
    return [row for _seconds, row in long_rows] + rows_grouped_by_crate([row for row in rows if row.line_number not in moved_lines]), len(long_rows)


def joined_result_lines(output, environment=None):
    """结果词被测试中途打的行隔开的结果行接回一行：`test <名> ... <测试自己打的字>`，后面隔着别的输出，结局另起一行单独写（`ok`、`FAILED`、`ignored`，可带「, 理由」或用时）的，
    把结局接到起头那一行上、单独那一行去掉，别的行原样留着（libtest 打 `test <名> ... ` 与结局是两次写，别的线程的输出插在中间）。
    起头之后先来了别的 `test <名> ... ` 或整行结果行的，前一条认不出结局、不接；这一段另有没有起头可接的单独一行结局，接出来的都不认
    （同 lib_libtest_result_lines.py 的 account_binary），原文照旧交回。
    弄坏开关 split-result-line-unseen（environment 没给取进程环境）：不接，只认同一行写全的结果行。"""
    environment = os.environ if environment is None else environment
    if break_is(environment, "split-result-line-unseen"):
        return output
    lines = output.split("\n")
    pending = None
    joined_at, orphans = {}, 0
    for index, line in enumerate(lines):
        stripped = line.rstrip("\r")
        if libtest_result_lines.result_line(stripped) is not None:
            pending = None
            continue
        started = libtest_result_lines.SCATTERED_START_FORM.match(stripped)
        if started is not None and not stripped.startswith("test result: "):
            pending = (index, started.start("rest"))
            continue
        if libtest_result_lines.STANDALONE_OUTCOME_FORM.match(stripped):
            if pending is None:
                orphans += 1
            else:
                joined_at[pending[0]] = (index, pending[1])
                pending = None
    if orphans or not joined_at:
        return output
    outcome_lines = {outcome_index for outcome_index, _rest_start in joined_at.values()}
    kept = []
    for index, line in enumerate(lines):
        if index in outcome_lines:
            continue
        if index in joined_at:
            outcome_index, rest_start = joined_at[index]
            line = line.rstrip("\r")[:rest_start] + lines[outcome_index].strip()
        kept.append(line)
    return "\n".join(kept)


def results_by_test(expected, output, environment=None):
    """libtest 的结果行 `test <模块路径>::<名字> ... ok|FAILED|ignored` 里对上点名的那几条：{完整名: [结果…]}。
    #[should_panic] 的用例在名字与 ` ... ` 之间多一段 ` - should panic`；点名可以省掉模块路径；结果词被测试中途打的行隔开的先接回一行（joined_result_lines）。
    解析在 lib_libtest_result_lines.py（全仓一份）。"""
    return libtest_result_lines.results_of_named_test(expected, joined_result_lines(output, environment))


def output_tail(output, lines=8):
    return "\n".join(output.splitlines()[-lines:])


def failure_excerpt(output, full_name):
    """libtest 给这一条打的 `---- <名字> stdout ----` 那一段的头几行；没有就交 None。"""
    marker = f"---- {full_name} stdout ----"
    index = output.find(marker)
    if index < 0:
        return None
    return "\n".join(output[index:].splitlines()[:6])


def baseline_problem(row, returncode, output):
    """这一行的点名测试在基线输出里不是「恰好一个用例、结果都是 ok」时交那一句（接在「在没改坏的副本上」后面），是的交 None。"""
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


def not_run_reason(expected, output, environment=None):
    """输出里有 NOT_RUN_NO_GPU_ADAPTER 那一族本次未跑行时交接在「打了 NOT_RUN_NO_GPU_ADAPTER」后面括号里的那一句（文件头「本次未跑」），一行都没有交 None：
    test= 对上点名的测试（只比 :: 分开的末一段）的那一行先取，交它 reason= 之后那一句（没写交「没写为什么」）；
    都对不上的（崩溃枚举用例判段不跑时打的是 test=crash_state_enumeration::<项名>，项名不是测试名）照样交，写明 test= 是什么、对不上测试名：
    宁可记本次未跑，不记没红。弄坏开关 not-run-test-name-only（environment 给了才认）：只认 test= 对得上的。"""
    wanted = expected.split("::")[-1]
    found_lines = list(NOT_RUN_LINE.finditer(output))
    for found in found_lines:
        if found.group(1).split("::")[-1] == wanted:
            return (found.group(2) or "").strip() or "没写为什么"
    if not found_lines or break_is(environment or {}, "not-run-test-name-only"):
        return None
    first = found_lines[0]
    return f"那一行是 test={first.group(1)}，对不上点名的测试名，照本次未跑记：{(first.group(2) or '').strip() or '没写为什么'}"


def verdict_of_run(row, returncode, output, environment=None):
    """一条变异跑完之后判哪一档：(档, 那一条要打印的话)。environment 只看弄坏开关（notrunasmissed、not-run-test-name-only）。"""
    tail = output_tail(output)
    # 点名省掉模块路径时在输出里只许对上一个用例：对上两个就分不出红的是不是点名的那一个，按没判成记，不按「有一个红了」算抓到。
    results = results_by_test(row.expected, output, environment)
    if len(results) > 1:
        return "failure", (f"{TABLE}:{row.line_number} {row.name}：点名的测试 {row.expected} 在输出里对上了 {len(results)} 个不同的用例"
                           f"（{'、'.join(sorted(results))}），判不出红的是哪一个；把点名写全到模块路径\n{tail}")
    if returncode != 0 and any("FAILED" in outcomes for outcomes in results.values()):
        return "caught", caught_text(row)
    # 点名的测试是 ok、又打了 NOT_RUN_NO_GPU_ADAPTER：没跑到被改坏的那一处，记本次未跑（文件头「本次未跑」），不记没红
    reason = not_run_reason(row.expected, output, environment) if results and not break_is(environment or {}, "notrunasmissed") else None
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



def deferred_write_library(library):
    """弄坏开关 records-written-at-end 用：包一层真库，write、write_json 先攒在这个工作进程的内存里，进程池正常收尾（工作进程正常退出时 multiprocessing 的退出终结器）才一并写进真库；
    读与别的方法转给真库。工作进程被硬杀时攒着的全丢（被怀疑过的样子：整组被杀、判过的格不留下）。
    类写在函数体里、不做模块级的类：模块级的类头在判法单元的闭包里（文件头「判法单元」），多一个整库的行键就全变。"""
    import multiprocessing.util as exit_hooks

    class Deferred:
        def __init__(self):
            self.pending = []
            exit_hooks.Finalize(None, self.flush, exitpriority=10)

        def write(self, kind, name, fields):
            self.pending.append(("write", kind, name, fields))

        def write_json(self, kind, name, value):
            self.pending.append(("write_json", kind, name, value))

        def flush(self):
            for method, kind, name, value in self.pending:
                getattr(library, method)(kind, name, value)
            self.pending.clear()

        def __getattr__(self, name):
            return getattr(library, name)
    return Deferred()


def init_worker(sequence_counter, shard_root, context):
    """工作进程启动时跑一次：领一个稳定的片号、建自己的源码副本与编译目录。片号从共用编译目录那一层的锁文件领（claim_worker_slot，跨调用不重），
    活是动态领的（crates-mutation-replay 文件头）。sequence_counter 只在拿不到锁文件、或弄坏开关 local-shard-numbering 下当本地序号用。"""
    # 父进程在 run_pool 里把 STOP_SIGNALS 接成「收尾再退」；工作进程照默认（收到就停），不把父进程的接法带进来
    #   （它起着的临时 prune-server 经 mutation_run_prune_server_binary 写的包装带父进程死亡信号，工作进程一死就跟着停）
    for signal_number in STOP_SIGNALS:
        signal.signal(signal_number, signal.SIG_DFL)
    # 每个工作进程的峰值（文件头「内存上限」的「每个工作进程的峰值」）：起采样线程量本进程整棵树（连拷副本、先编依赖的拷贝在内），按编译并行度记进运行记录目录。
    #   被判的不是本仓（门禁样本根）不起（worker_peaks_recorded_for）；弄坏开关 worker-peaks-unrecorded：都不起
    if context.get("worker_peaks_recorded") and not break_is(context["environment"], "worker-peaks-unrecorded"):
        threading.Thread(target=sample_worker_peak, name="worker-peak-sampler", daemon=True,
                         args=(context["run_records_directory"], context["environment"]["CARGO_BUILD_JOBS"], context["environment"])).start()
    worker_sequence, slot_lock = claim_worker_slot(context["shared_target_directory"], sequence_counter, context["environment"])
    WORKER_STATE["slot_lock"] = slot_lock   # 拿到进程退出：内核放，别的调用的工作进程才试得到这一片
    work, target_directory = prepare_shard(worker_sequence, shard_root, context)
    note(f"  … 工作进程 {os.getpid()} 领到片号 {worker_sequence}（编译目录 {target_directory}；"
         + (f"片号锁 {slot_lock.name}" if slot_lock is not None else "没有片号锁，按本地序号") + "）")
    environment = dict(context["environment"])
    environment["CARGO_TARGET_DIR"] = target_directory
    # 在用的标记（WORKER_TARGET_LOCK_NAME）：进程退出内核就放，别的一趟回收时拿不到排他锁就不删这份
    WORKER_STATE["target_lock"] = hold_worker_target_lock(target_directory)
    # 先编一份依赖（文件头「先编一份依赖」）：先编成的那一份里外部依赖的产物拷进自己的两份编译目录（普通、插桩），工作区的包照旧自己编
    copy_seed_dependencies(context, target_directory)
    # 这份副本的整树快照：只当基线记录的一样属性记下（弄坏开关 baseline-by-tree-snapshot 按它比），不当复用的依据
    snapshot = snapshot_identity(snapshot_of(work, context["copied"]))
    # 判完一格就原子写进判定库（Library.write：同目录临时文件、fsync、改名），整组被硬杀时判过的格留着，重跑同一条命令照键复用。弄坏开关 records-written-at-end：攒到进程池收尾才写
    library = mutation_units.Library(context["library_directory"], RECORD_HEADER)
    if break_is(context["environment"], "records-written-at-end"):
        library = deferred_write_library(library)
    WORKER_STATE.update(work=work, environment=environment, context=context, shard_root=shard_root, worker_sequence=worker_sequence,
                        per_mutation_temporary_root=os.path.join(shard_root, PER_MUTATION_TEMPORARY_DIRECTORY_NAME),
                        snapshot=snapshot, library=library, unit_keys={},   # 包目录 → 记覆盖用的单元键（record_coverage）
                        # 起进程那一下、找多机配置文件、取放开 --test-threads 1 的测试目标名单与起临时 prune-server 经这几个引用（文件头「判法单元」）：判法单元从 judge_row、
                        # run_cargo_command 这几个入口收闭包，起的程序（cargo、prune-server）各有自己的身份，多机配置是交给测试的一份文件，名单在 admission.py，
                        # 临时服务的起法在客户端自证那一份；这几样不进判法单元，闭包也就不碰折不出命令的 Popen、不载入客户端自证
                        launch=run_capturing_output, multi_host_configuration=test_runs_multi_host_configuration,
                        gpu_released_targets=gpu_released_test_targets, temporary_service=mutation_run_prune_server)
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
    弄坏开关 local-shard-numbering：不看锁文件，从本调用的共享计数器现领序号。"""
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
    （基线会报 attempt to subtract with overflow、读 litmus 读到上一轮已经删掉的副本路径）。
    修改时刻取拷的那一刻，每片头一次编译把工作区的几个 crate 重编一遍（外部依赖不在副本里，照旧热着）。"""
    work = tempfile.mkdtemp(prefix=f"shard-{worker_sequence}-", dir=shard_root)
    for item in context["copied"]:
        source = os.path.join(context["root"], item)
        if not os.path.exists(source):
            continue
        if item == CARGO_CONFIGURATION_DIRECTORY and break_is(context["environment"], "cargoconfignotcopied"):
            continue   # 弄坏开关（文件头）：不拷 .cargo/，副本里没有它
        target = os.path.join(work, item)
        if os.path.isdir(source):
            shutil.copytree(source, target, ignore=shutil.ignore_patterns(*IGNORED_WHEN_COPYING_INTO_EACH_SHARD), copy_function=shutil.copy)
        else:
            os.makedirs(os.path.dirname(target), exist_ok=True)
            shutil.copy(source, target)
    shared = context["shared_target_directory"]
    return work, (shared if worker_sequence == 1 else f"{shared}-w{worker_sequence}")


def default_shared_target_directory(environment):
    """调用方给的第 1 片编译目录（其余片加 -w<片号>，记覆盖的再加 -coverage）：GATE_MUTATION_TARGET_DIR 设了交它（跑完不删）；没设交 None，
    run_pool 在这一趟的 <shard_root>/build 里建（run_build_target_directory）、收尾删。弄坏开关 targetdefaulttmp：没设时交
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
    每份的涨幅：peaks（measured_target_peaks 交的 {"普通": 字节或 None, "插桩": 字节或 None}，运行记录目录里近几次每格判完量到的占盘峰值）里有那一种的读数就用它，
    没有读数、peaks 是 None 的按 TARGET_DIRECTORY_CAP 算（上限只当清理线，没实测时才拿它顶）。
    余量没读到、写法不认交 None 与原因，头一行写明。弄坏开关 disknotchecked：不看磁盘；diskbudgetbycap：不看实测峰值、一律按上限算。"""
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


def scope_memory_max(cgroup_listing, cgroup_root):
    """自己 scope 的 memory.max（cgroup v2：cgroup_listing 即 /proc/self/cgroup 里 `0::<路径>` 那一行接 cgroup_root 再接 /memory.max；只读自己这一层，不往上找父 cgroup）：
    (字节或 None, 说明)。有限值交 (字节, 「自己 scope 的 <路径>」)；值是 max（不限）、文件读不到、没有 v2 的那一行、写法不认交 (None, 原因)。"""
    try:
        with open(cgroup_listing, encoding="utf-8") as handle:
            listing = handle.read().splitlines()
    except OSError as error:
        return None, f"读不到 {cgroup_listing}（{type(error).__name__}：{error}）"
    scope = next((line[3:] for line in listing if line.startswith("0::")), None)
    if scope is None:
        return None, f"{cgroup_listing} 里没有 cgroup v2 的那一行（0::<路径>）"
    memory_max = os.path.join(cgroup_root, scope.lstrip("/"), "memory.max")
    try:
        with open(memory_max, encoding="utf-8") as handle:
            text = handle.read().strip()
    except OSError as error:
        return None, f"读不到 {memory_max}（{type(error).__name__}：{error}）"
    if text == "max":
        return None, f"{memory_max} 是 max（不限）"
    if not re.fullmatch(r"[1-9][0-9]*", text):
        return None, f"{memory_max} 写着「{text}」，不是 max 也不是正整数"
    return int(text), f"自己 scope 的 {memory_max}"


def this_host_slice_cap_text(configuration, by_host_text):
    """多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项（文件头「内存上限」②）：(值或 None, 这一台的地址或 None, 取不到的原因)。
    这一台照多机配置判法 --emit-addresses 认（multi_host_addresses.resolve：整份配置判过才给地址）；每项从第一个 = 切。"""
    try:
        address = multi_host_addresses.resolve(configuration).this
    except multi_host_addresses.AddressesUnavailable as error:
        return None, None, f"认不出这一台（{error}）"
    for item in by_host_text.split():
        item_address, _separator, value = item.partition("=")
        if item_address == address:
            return value, address, ""
    return None, address, f"没有这一台 {address} 那一项"


def task_memory_cap_bytes(environment, cgroup_listing=CGROUP_LISTING, cgroup_root=CGROUP_ROOT, configuration_path=None):
    """这一条命令拿到的内存上限（文件头「内存上限」）：(字节或 None, 说明或没有这一项的原因)。
    次序：① 统一任务服务交的 TASK_SERVER_MEMORY_CAP_BYTES（正整数字节）；② 没有（或写法不对，说明里点名）读多机配置的 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项
    （this_host_slice_cap_text；configuration_path 给了读它，不给按 test_runs_multi_host_configuration 找；文件不在、键没写、认不出这一台、写法不认、读不了都往下走，
    说明里点名）；③ 自己 scope 的 memory.max（scope_memory_max）。
    memory.max 是有限值就是硬顶（超了整组被杀）：① ② 取到的值比它大，取 memory.max（两者取小）；① ② 都没有，取 memory.max 本身；
    memory.max 是 max（不限）或读不到，不收窄，照 ① ② 的值，① ② 也没有就交 None 与原因，调用方照线程数开。
    cgroup_listing、cgroup_root 是默认值（/proc/self/cgroup、/sys/fs/cgroup）时，环境里的 GATE_MUTATION_CGROUP_LISTING、GATE_MUTATION_CGROUP_ROOT 设了就换成它们（只给自证用，生产不设）。
    这两个变量名与默认值写在函数体里、不做模块级常量：模块级的赋值与 def 头缺省值都在判法单元的闭包里（文件头「判法单元」），多一个整库的行键就全变。
    说明写各档读到的值、取了哪一个、为什么（开跑行原样打出）。弄坏开关 memory-max-not-hard-cap：①② 取到了就不拿 memory.max 当硬顶；memory-cap-ignored 在 memory_bound_for。"""
    if cgroup_listing == CGROUP_LISTING:
        cgroup_listing = environment.get("GATE_MUTATION_CGROUP_LISTING", "").strip() or cgroup_listing
    if cgroup_root == CGROUP_ROOT:
        cgroup_root = environment.get("GATE_MUTATION_CGROUP_ROOT", "").strip() or cgroup_root
    notes, readings, chosen = [], [], None   # chosen：①②里取到的 (字节, 原话)
    given = environment.get(TASK_MEMORY_CAP_VARIABLE, "").strip()
    if given:
        if re.fullmatch(r"[1-9][0-9]*", given):
            chosen = (int(given), f"服务交的 {TASK_MEMORY_CAP_VARIABLE}={given}")
            readings.append(f"{TASK_MEMORY_CAP_VARIABLE}={given}（{format_size(int(given))}）")
        else:
            notes.append(f"{TASK_MEMORY_CAP_VARIABLE} 写成「{given}」，不是正整数字节数，往下读多机配置的 {SLICE_MEMORY_CAP_KEY}")
            readings.append(f"{TASK_MEMORY_CAP_VARIABLE} 写法不认")
    else:
        readings.append(f"{TASK_MEMORY_CAP_VARIABLE} 没给")
    configuration = configuration_path if configuration_path is not None else test_runs_multi_host_configuration(environment)[0]
    try:
        by_host_text = admission_module().configuration_value(configuration, SLICE_MEMORY_CAP_KEY)
    except Exception as error:   # InputManifestError：文件在而读不了；不吞，写进说明、往下走
        if chosen is None:
            notes.append(str(error))
        readings.append(f"多机配置 {SLICE_MEMORY_CAP_KEY} 读不了")
    else:
        slice_text, this_address, unavailable = this_host_slice_cap_text(configuration, by_host_text) if by_host_text else (None, None, "")
        label = f"{SLICE_MEMORY_CAP_KEY} 里这一台 {this_address} 那一项"
        slice_cap = bytes_of_size(slice_text) if slice_text else None
        if slice_cap is not None:
            readings.append(f"多机配置 {label}={slice_text}（{format_size(slice_cap)}）")
            if chosen is None:
                chosen = (slice_cap, f"多机配置 {configuration} 的 {label}={slice_text}")
        elif slice_text:
            if chosen is None:
                notes.append(f"多机配置 {configuration} 的 {label}写成「{slice_text}」，不是正整数加 K / M / G / T")
            readings.append(f"多机配置 {label}写法不认")
        elif by_host_text:
            if chosen is None:
                notes.append(f"多机配置 {configuration} 的 {SLICE_MEMORY_CAP_KEY} 取不到这一台那一项：{unavailable}")
            readings.append(f"多机配置 {SLICE_MEMORY_CAP_KEY} 取不到这一台那一项")
        else:
            readings.append(f"多机配置没写 {SLICE_MEMORY_CAP_KEY}")
    scope_cap, scope_text = scope_memory_max(cgroup_listing, cgroup_root)
    readings.append(f"scope 的 memory.max={scope_cap}（{format_size(scope_cap)}）" if scope_cap is not None else f"scope 的 memory.max 没有硬顶（{scope_text}）")
    if chosen is None:
        # ① ② 都没有：memory.max 是有限值就取它（它自己就是这一条拿到的上限），不然这一项不收
        return scope_cap, "；".join(notes + [scope_text])
    readings_text = "各档读到：" + "、".join(readings)
    hard_cap = None if break_is(environment, "memory-max-not-hard-cap") else scope_cap   # 弄坏开关：①② 取到了就不拿 memory.max 当硬顶
    if hard_cap is not None and hard_cap < chosen[0]:
        return scope_cap, "；".join(notes + [f"{scope_text}（硬顶：超了整组被杀）；比它大的 {chosen[1]}（{format_size(chosen[0])}）不取，取两者里小的", readings_text])
    why = ("scope 的 memory.max 没有硬顶可比，取服务交的变量，没有就取多机配置的" if scope_cap is None
           else "scope 的 memory.max 不比它小、不收窄，取服务交的变量，没有就取多机配置的")
    return chosen[0], "；".join(notes + [chosen[1], readings_text, why])


def cargo_jobs_for(environment, worker_count, processor_count):
    """每个 cargo 的编译并行度（文件头「编译并行度」）：GATE_MUTATION_CARGO_JOBS 给了照它（run_stage 先核过是正整数），不然 max(1, 核数 ÷ 工作进程数)。
    memory_bound_for 试每个进程数、run_stage 设 CARGO_BUILD_JOBS 用的都是这一份，两边算的编译并行度对得上。"""
    given = environment.get("GATE_MUTATION_CARGO_JOBS", "").strip()
    return int(given) if given else max(1, processor_count // max(1, worker_count))


def worker_peaks_directory(records_directory):
    """每个工作进程的峰值记录（文件头「内存上限」的「每个工作进程的峰值」）：运行记录目录下 worker-peaks/<编译并行度>/<记录名>。"""
    return os.path.join(records_directory, "worker-peaks")


def new_worker_peak_record_name():
    """一个工作进程这一趟那份记录的名字：起采样的时刻纳秒（20 位，名字排序就是先后）、进程号、8 位随机字符（两台的进程号会撞）。"""
    alphabet = "abcdefghijklmnopqrstuvwxyz0123456789"
    chooser = random.SystemRandom()
    return f"{time.time_ns():020d}-{os.getpid()}-{''.join(chooser.choice(alphabet) for _ in range(8))}"


def record_worker_peak(records_directory, jobs, name, peak_bytes):
    """把一个工作进程这一趟到此刻的最大写进 worker-peaks/<jobs>/<name>（同目录临时文件、改名换上；写不进抛 OSError）。"""
    write_atomically(os.path.join(worker_peaks_directory(records_directory), str(jobs), name), f"{peak_bytes}\n")


def worker_peaks_recorded_for(root, environment):
    """这一趟的工作进程写不写峰值记录（文件头「内存上限」的「记在哪」）：被判的是本模块所在的仓、或 --staged 的临时树（GATE_STAGED_FROM 设着）、
    或在临时 prune-server 底下（自证：运行记录目录按库所在的目录分开）才写；别处（门禁样本根这一类）不写：样本 crate 的峰值不代表本仓，
    不在临时服务底下时它的运行记录目录就是本仓那一份，写进去会把小读数掺进去。弄坏开关 sample-root-peaks-recorded：都写。"""
    return (break_is(environment, "sample-root-peaks-recorded") or os.path.realpath(root) == os.path.realpath(STAGE_REPOSITORY)
            or bool(environment.get("GATE_STAGED_FROM", "").strip()) or under_temporary_prune_server(environment))


def process_usage_bytes(pid):
    """一个进程此刻的用量：/proc/<pid>/status 的 RssAnon + RssShmem + VmPTE（kB 换成字节；内核线程与僵尸进程没有这几行，算 0）。读不到（进程刚退出）交 None。"""
    try:
        with open(f"/proc/{pid}/status", encoding="utf-8", errors="replace") as handle:
            text = handle.read()
    except OSError:
        return None
    total = 0
    for line in text.splitlines():
        name, _separator, value = line.partition(":")
        words = value.split()
        if name in ("RssAnon", "RssShmem", "VmPTE") and words and words[0].isdigit():
            total += int(words[0]) * 1024
    return total


def base_reading_line(parent_bytes, seed_tree_peak_bytes, seed_built):
    """底座读数那一行（文件头「内存上限」的「底座读数」）：机读的 MUTATION_BASE_READING 三个字段，后面接一句人读的。"""
    parent = "none" if parent_bytes is None else str(parent_bytes)
    seed = "none" if seed_tree_peak_bytes is None else str(seed_tree_peak_bytes)
    parent_text = "读不到" if parent_bytes is None else format_size(parent_bytes)
    seed_text = "读不到" if seed_tree_peak_bytes is None else format_size(seed_tree_peak_bytes)
    return (f"MUTATION_BASE_READING parent_bytes={parent} seed_step_tree_peak_bytes={seed} seed_step={'built' if seed_built else 'not-built'}"
            f"  （底座读数：派活之前起这一道的父进程自己 {parent_text}；先编一份依赖那一步（工作进程起来之前）父进程整棵进程树的峰值 {seed_text}"
            f"{'' if seed_built else '，这一趟没先编，只量到父进程自己'}；口径同每个工作进程的读数：RssAnon + RssShmem + VmPTE 加总）")


def process_tree_usage(root_pid, environment):
    """root_pid 连同它整棵进程树此刻的用量（文件头「内存上限」的「量什么」）：字节。子进程顺 /proc/<pid>/task/<tid>/children 往下找，读不到的（刚退出）不算。
    root_pid 自己读不到、或它主线程的 children 文件不在（这台内核没有这一项）交 None。弄坏开关 worker-tree-self-only：只算 root_pid 自己。"""
    total, pending, seen = 0, [root_pid], set()
    while pending:
        pid = pending.pop()
        if pid in seen:
            continue
        seen.add(pid)
        usage = process_usage_bytes(pid)
        if usage is None:
            if pid == root_pid:
                return None
            continue
        total += usage
        if break_is(environment, "worker-tree-self-only"):
            break
        try:
            threads = os.listdir(f"/proc/{pid}/task")
        except OSError:
            continue
        for thread in threads:
            try:
                with open(f"/proc/{pid}/task/{thread}/children", encoding="ascii") as handle:
                    pending += [int(word) for word in handle.read().split()]
            except FileNotFoundError:
                if pid == root_pid and thread == str(root_pid):
                    return None
            except OSError:
                continue
    return total


def sample_worker_peak(records_directory, jobs, environment):
    """工作进程里的采样线程（init_worker 起的守护线程，随工作进程一起没）：每 WORKER_PEAK_SAMPLE_SECONDS 秒量一次本进程整棵树（process_tree_usage），
    头一次量到就写一份记录，之后这一趟的最大涨了 WORKER_PEAK_WRITE_STEP_BYTES 以上再重写（record_worker_peak）。
    量不了、写不进各报一行就收手：这一格的判定不受影响，下一趟少这一份读数。"""
    name, peak, written = new_worker_peak_record_name(), 0, None
    while True:
        usage = process_tree_usage(os.getpid(), environment)
        if usage is None:
            note(f"  ! 工作进程 {os.getpid()} 量不了自己的进程树（/proc/{os.getpid()}/task/{os.getpid()}/children 不在：这台内核没有这一项），这一趟不记峰值；"
                 "下一趟这个编译并行度照没有实测取保守值")
            return
        peak = max(peak, usage)
        if written is None or peak >= written + WORKER_PEAK_WRITE_STEP_BYTES:
            try:
                record_worker_peak(records_directory, jobs, name, peak)
            except OSError as error:
                note(f"  ! 工作进程 {os.getpid()} 的峰值记录没写进 {worker_peaks_directory(records_directory)}（{type(error).__name__}：{error}）；下一趟少这一份读数，不碍这一格的判定")
                return
            written = peak
        time.sleep(WORKER_PEAK_SAMPLE_SECONDS)


def measured_worker_peaks(records_directory, environment):
    """读回每个工作进程的峰值记录（文件头「内存上限」的「怎么读」）：({编译并行度: (最新 WORKER_PEAKS_RECENT 份里最大的字节, 份数)} 或 None, 说明)。
    worker-peaks/ 还没有交 {}（从没量过）；有一样不是这一版的写法或读不了交 None（整份读不了），说明里点名哪一份、为什么与出路。
    读的时候删掉每个编译并行度里比最新那几份更旧的。弄坏开关 worker-peaks-unkeyed-by-jobs：各编译并行度都取全部记录里最大的；
    worker-peaks-window-unbounded：取全部记录的最大、不删更旧的。"""
    base = worker_peaks_directory(records_directory)
    howto = f"删掉那一份（或整个 {base}）再跑：下一趟照没有实测取保守值、重新量"
    try:
        jobs_names = sorted(os.listdir(base))
    except FileNotFoundError:
        return {}, f"峰值记录 {base} 还没有（从没量过）"
    except OSError as error:
        return None, f"峰值记录 {base} 读不了（{type(error).__name__}：{error}）；{howto}"
    unbounded = break_is(environment, "worker-peaks-window-unbounded")
    readings = {}
    for jobs_name in jobs_names:
        directory = os.path.join(base, jobs_name)
        if not re.fullmatch(r"[1-9][0-9]*", jobs_name):
            return None, f"峰值记录 {base} 读不了：{directory} 的名字不是编译并行度（正整数）；{howto}"
        try:
            names = sorted(name for name in os.listdir(directory) if not name.startswith(".partial-"))
        except OSError as error:
            return None, f"峰值记录 {base} 读不了：{directory}（{type(error).__name__}：{error}）；{howto}"
        records = []
        for name in names:
            path = os.path.join(directory, name)
            if not WORKER_PEAK_RECORD_NAME.fullmatch(name):
                return None, f"峰值记录 {base} 读不了：{path} 的名字不是「<时刻纳秒 20 位>-<进程号>-<8 位随机字符>」；{howto}"
            try:
                with open(path, encoding="utf-8", errors="replace") as handle:
                    text = handle.read()
            except FileNotFoundError:
                continue   # 别的一趟读的时候刚删掉的更旧那几份
            except OSError as error:
                return None, f"峰值记录 {base} 读不了：{path}（{type(error).__name__}：{error}）；{howto}"
            if not re.fullmatch(r"[1-9][0-9]*\n", text):
                return None, f"峰值记录 {base} 读不了：{path} 写着「{text.strip()[:40]}」，不是一个正整数字节数；{howto}"
            records.append((name, int(text)))
        window = records if unbounded else records[-WORKER_PEAKS_RECENT:]
        if not unbounded:
            for name, _value in records[:-WORKER_PEAKS_RECENT]:
                with contextlib.suppress(OSError):
                    os.unlink(os.path.join(directory, name))
        if window:
            readings[int(jobs_name)] = (max(value for _name, value in window), len(window))
    if break_is(environment, "worker-peaks-unkeyed-by-jobs") and readings:
        merged = (max(value for value, _count in readings.values()), sum(count for _value, count in readings.values()))
        readings = {jobs: merged for jobs in readings}
    listed = "、".join(f"编译并行度 {jobs} 最新 {count} 份里最大 {format_size(value)}" for jobs, (value, count) in sorted(readings.items()))
    return readings, f"峰值记录 {base}：" + (listed or "一份都没有（从没量过）")


def worker_peak_for_jobs(peaks, jobs, cap, environment):
    """编译并行度 jobs 下每个工作进程的峰值（文件头「内存上限」的「怎么除」）：(字节, 取法)。peaks 是 measured_worker_peaks 交的（None 是整份读不了）。
    ① jobs 有实测取它；② 没有而更高的编译并行度有，借比它大的里最小那一个的；③ 都没有、或整份读不了，取保守值 = 内存上限 cap 本身（只开 1 个）。
    弄坏开关 unmeasured-worker-peak-guessed（③ 里没有实测那一种）、unreadable-worker-peaks-guessed（读不了那一种）：写死的默认 3G。"""
    if peaks is None:
        if break_is(environment, "unreadable-worker-peaks-guessed"):
            return bytes_of_size("3G"), "峰值记录读不了，照写死的默认 3G（弄坏开关 unreadable-worker-peaks-guessed）"
        return cap, "峰值记录读不了，取保守值：按一个工作进程占满整份内存上限算"
    if jobs in peaks:
        value, count = peaks[jobs]
        return value, f"最新 {count} 份实测里最大的"
    higher = [measured for measured in peaks if measured > jobs]
    if higher:
        value, count = peaks[min(higher)]
        return value, f"没有实测，借编译并行度 {min(higher)} 最新 {count} 份实测里最大的（推的：编译并行度越高，每个工作进程的峰值不会越低）"
    if break_is(environment, "unmeasured-worker-peak-guessed"):
        return bytes_of_size("3G"), "没有实测，照写死的默认 3G（弄坏开关 unmeasured-worker-peak-guessed）"
    return cap, ("从没量过" if not peaks else "只有更低编译并行度的实测、不借") + "，取保守值：按一个工作进程占满整份内存上限算"


def memory_bound_for(environment, ceiling, processor_count, records_directory, cgroup_listing=CGROUP_LISTING, cgroup_root=CGROUP_ROOT, configuration_path=None):
    """按内存上限能开几个工作进程（文件头「内存上限」）：(个数或 None, 头一行要写的话)。ceiling 是另外三项的小者，processor_count 是核数（cargo_jobs_for 用），
    records_directory 是运行记录目录（峰值记录在它的 worker-peaks/ 下，measured_worker_peaks 读）。
    n 从 1 试到 ceiling：编译并行度 j = cargo_jobs_for(n)，每个工作进程的峰值 P = worker_peak_for_jobs(j)，满足 n × P ≤ 上限的最大 n 是这一项；
    到了 ceiling 还满足报 ⌊上限 ÷ P⌋（编译并行度照 ceiling 算，不与另外三项并列成「卡在」）；一个都不满足交 0（调用方按 1 个开）。
    没有内存上限（memory.max 是 max、读不到）交 None 与原因，这一项不收、照线程数开。
    弄坏开关 memory-cap-ignored：不看内存上限；jobs-feedback-ignored：每个 n 的编译并行度都照 ceiling 算。"""
    if break_is(environment, "memory-cap-ignored"):
        return None, "弄坏开关 memory-cap-ignored：不看内存上限"
    cap, cap_source = task_memory_cap_bytes(environment, cgroup_listing, cgroup_root, configuration_path)
    if cap is None:
        return None, cap_source
    peaks, peaks_source = measured_worker_peaks(records_directory, environment)
    ceiling = max(1, ceiling)

    def planned(count):
        jobs = cargo_jobs_for(environment, ceiling if break_is(environment, "jobs-feedback-ignored") else count, processor_count)
        return (jobs,) + worker_peak_for_jobs(peaks, jobs, cap, environment)

    best = max((count for count in range(1, ceiling + 1) if count * planned(count)[1] <= cap), default=0)
    jobs, peak, basis = planned(max(1, best))
    count = best if best < ceiling else cap // peak
    text = (f"内存上限 {format_size(cap)}（{cap_source}）÷ 每个工作进程的峰值 {format_size(peak)}（编译并行度 {jobs} 下：{basis}）= 同时开 {count} 个工作进程"
            + (f"（{best} × {format_size(peak)} = {format_size(best * peak)}，不超过上限）" if best else f"（1 个就要 {format_size(peak)}，超了上限）"))
    if 1 <= best < ceiling:
        next_jobs, next_peak, next_basis = planned(best + 1)
        text += (f"；再多 1 个：编译并行度 {next_jobs} 下每个 {format_size(next_peak)}（{next_basis}），"
                 f"{best + 1} × {format_size(next_peak)} = {format_size((best + 1) * next_peak)}，超了上限")
    return count, f"{text}；{peaks_source}"


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
    每个产物的最新一版不动，清完仍超线的由调用方报并停派（trim_worker_target）。弄坏开关 targetcapignored：不清；depsnotpruned：只清增量、不清 deps 里的旧版本。"""
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
    if occupied and context.get("run_records_directory"):
        problem = record_target_occupancy(context["run_records_directory"], coverage, occupied)
        if problem is not None:
            note(f"  ! 编译目录占盘没记进 {target_occupancy_file(context['run_records_directory'])}（{problem}）；磁盘预算少一个读数，不碍这一格的判定")
    if before > (cap or before):
        incremental_removed = [size for path, size in removed if os.path.basename(path) == INCREMENTAL_DIRECTORY_NAME]
        deps_removed = [size for path, size in removed if os.path.basename(path) != INCREMENTAL_DIRECTORY_NAME]
        still_over = after > cap
        note(f"  … 编译目录 {directory} 占盘 {format_size(before)} 超过上限 {format_size(cap)}：清掉增量产物 {len(incremental_removed)} 处共 {format_size(sum(incremental_removed))}"
             + (f"、deps 里同名产物的旧版本 {len(deps_removed)} 份共 {format_size(sum(deps_removed))}" if deps_removed else "")
             + f"，清后 {format_size(after)}"
             + ("；清后仍超上限（剩下的是每个依赖与测试二进制的最新一版）：停派（文件头「编译目录上限」）" if still_over else ""))
        if still_over and not break_is(context["environment"], "overcapdispatched"):
            # 停派（文件头「编译目录上限」）：往这一趟 shard_root 里的停派目录写一份，父进程每收一格看一眼，有就不再派活。弄坏开关 overcapdispatched：只报不停
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


def target_occupancy_file(records_directory):
    """每格判完量到的编译目录占盘记在运行记录目录（run_records_directory，文件头「判定库」末段）下这一份（文件头「磁盘余量」）：
    一行「<时刻秒><TAB>普通|插桩<TAB><字节>」，只追加。是运行记录，不进判定库。"""
    return os.path.join(records_directory, "target-occupancy.tsv")


def record_target_occupancy(records_directory, coverage, size_bytes):
    """追加一行（O_APPEND，一行远小于 PIPE_BUF，几个工作进程同时追加不交错）；写进回 None，写不进回原因。"""
    line = f"{int(time.time())}\t{'插桩' if coverage else '普通'}\t{size_bytes}\n".encode()
    try:
        descriptor = os.open(target_occupancy_file(records_directory), os.O_WRONLY | os.O_APPEND | os.O_CREAT, 0o644)
        try:
            os.write(descriptor, line)
        finally:
            os.close(descriptor)
    except OSError as error:
        return f"{type(error).__name__}：{error}"
    return None


def measured_target_peaks(records_directory, recent):
    """{"普通": 字节或 None, "插桩": 字节或 None}：两种编译目录各取最近 recent 次记录里最大的（文件头「磁盘余量」）；文件不在、读不了的都是 None，写坏的行不算。
    记录多过 recent 的 50 倍时只留每种最近 recent 次，同目录临时文件改名换上（几趟同时用一份运行记录时可能丢几个读数，只是预算少几个样本）。"""
    kept = {"普通": [], "插桩": []}
    path = target_occupancy_file(records_directory)
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


def seed_jobs(context):
    """先编那一份的 CARGO_BUILD_JOBS：GATE_MUTATION_CARGO_JOBS 设了用它，不然取核数（这时还没有工作进程在跑，整台的核都给它）。"""
    given = context["environment"].get("GATE_MUTATION_CARGO_JOBS", "").strip()
    return given or str(max(1, os.cpu_count() or 4))


def build_seed_group(context, command, seed_source, extra_environment, temporary_root, instrumented):
    """先编一组：subprocess 的结果。TMPDIR 是这一组自己的，插桩那一份的构建脚本与过程宏的剖析文件也落在里面，编完整个删。"""
    temporary = tempfile.mkdtemp(prefix="seed-", dir=temporary_root)
    try:
        environment = dict(context["environment"], TMPDIR=temporary, **extra_environment)
        if instrumented:
            environment["LLVM_PROFILE_FILE"] = os.path.join(temporary, "build-%p-%m.profraw")
        return run_capturing_output(command, cwd=seed_source, env=environment)
    finally:
        shutil.rmtree(temporary, ignore_errors=True)


def prepare_seed(context, batches, ordered_rows, coverage_planner, temporary_root):
    """派活之前先编一份依赖：交 (要报的话, 判红的那几句)。编成的记进 context["seed_targets"]（{"plain" 或 "coverage": 编译目录}），
    工作进程起来时照它拷（copy_seed_dependencies）；没起、没编成的不在里面。弄坏开关 compile-per-worker：不起。"""
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
    jobs = seed_jobs(context)
    lines, problems = [], []
    try:
        with seed_lock(shared, exclusive=True):
            touched, changed_count = sync_seed_source(context["root"], context["copied"], seed_source, environment)
            lines.append(f"  … 先编一份依赖：源码副本 {seed_source} 同步了 {changed_count} 个文件（摸了整包的 {len(touched)} 个"
                         + (f"：{'、'.join(sorted(touched))}" if touched else "") + f"）；并行度 {jobs}")
            overrides, configuration = coverage_build_settings(environment)
            for kind, groups, target, extra, words in (("plain", plain, shared + SEED_TARGET_SUFFIX, {}, []),
                                                       ("coverage", coverage, shared + SEED_TARGET_SUFFIX + COVERAGE_TARGET_SUFFIX, overrides, configuration)):
                if not groups:
                    continue
                failed = []
                compiled_units = set()
                for arguments in groups.values():
                    command = ["cargo", *words, "test", "--offline", *arguments, "--no-run", "--message-format=json-render-diagnostics"]
                    run = build_seed_group(context, command, seed_source, dict(extra, CARGO_TARGET_DIR=target, CARGO_BUILD_JOBS=jobs), temporary_root,
                                           kind == "coverage")
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
    return lines, problems


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
    要拷的文件分到 threads 条线程上同时拷（弄坏开关 seed-copy-serial：在一条线程上逐个拷）；哪一个拷不成，OSError 照抛给调用方。回 (拷了几个文件, 几字节)。"""
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


def run_capturing_output(command, **popen_keywords):
    """跑一条命令、收它的 stdout 与 stderr，交回 subprocess.CompletedProcess（与 capture_output=True, text=True, errors="replace" 收的逐字相同）。
    命令起在它自己的进程组里；这一进程里被停（run_pool 接住的 TERM、HUP，先编一份依赖那几趟在父进程里跑）时对整组发 TERM，过 STOP_GRACE_SECONDS 还在再发 KILL，
    等它退了再往外抛。不限时：卡住的由统一任务服务按日志与 CPU 都不涨停整条任务（task-server/）。"""
    process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, errors="replace", start_new_session=True, **popen_keywords)
    try:
        stdout, stderr = process.communicate()
    except BaseException:
        with contextlib.suppress(ProcessLookupError):
            os.killpg(process.pid, signal.SIGTERM)
        try:
            process.wait(timeout=STOP_GRACE_SECONDS)
        except subprocess.TimeoutExpired:
            with contextlib.suppress(ProcessLookupError):
                os.killpg(process.pid, signal.SIGKILL)
            process.wait()
        raise
    return subprocess.CompletedProcess(command, process.returncode, stdout, stderr)


def gpu_test_run(command, environment=None):
    """这一次起的是不是带 gpu 特性、真跑测试的 cargo test：`--` 之前开了上卡的特性（--features / -F 里有 GPU_TEST_FEATURES 里的名字或 <包>/<名字>，或 --all-features），
    而且不带 --no-run。收尾补 --test-threads（with_gpu_test_threads）与核用没用上卡（文件头「真跑测试的格把多机配置交给测试、带 gpu 特性的格跑完核真用了卡」）都照它认。
    弄坏开关 product-gpu-feature-unquiesced（environment 给了才认）：只认 gpu，产品 GPU 包的 tests-on-a-real-gpu 不算。"""
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


def with_gpu_test_threads(command, environment, released_targets):
    """带 gpu 特性、真跑测试的 cargo test（gpu_test_run 认的那一种）的 libtest 参数收尾补 --test-threads（文件头「带 gpu 特性的格测试参数收尾带 --test-threads」）：
    `--` 之后已经有 --test-threads（--test-threads=N 或分开写）的原样不动、不重复补；没有的补 `--test-threads 1`，没有 `--` 的先补 `--`；
    cargo 参数点名的测试目标（--test <名>）全在 scripts/gate/admission.py 的 GPU_TEST_THREADS_RELEASED_TARGETS 里的补每台默认重活线程数
    （scripts/hosts/heavy_threads.py 的 default_heavy_threads；取不到打一行说明、照旧带 1）。只编不跑的、不带 gpu 特性的原样交回。
    released_targets 是调用方交的取名单的函数（init_worker 交的 gpu_released_test_targets：名单在 admission.py，不进判法单元的闭包）。
    补的是起命令时的参数，不进行键（文件头「行键」）。弄坏开关 gpu-test-threads-unadded：不补。"""
    if break_is(environment, "gpu-test-threads-unadded") or not gpu_test_run(command, environment):
        return list(command)
    words = list(command)
    test_index = words.index("test", words.index("cargo") + 1)
    separator = words.index("--") if "--" in words else None
    after = words[separator + 1:] if separator is not None else []
    if any(word == "--test-threads" or word.startswith("--test-threads=") for word in after):
        return words
    targets = mutation_units.parse_cargo_arguments(words[test_index + 1:separator] if separator is not None else words[test_index + 1:]).targets
    released = set(released_targets())
    threads = "1"
    if targets and all(kind == "test" and value in released for kind, value in targets):
        try:
            threads = str(default_heavy_threads(environment))
        except HeavyThreadDefaultError as error:
            note(f"  ! 取不出每台默认重活线程数（{error}），放开 --test-threads 1 的测试目标 {'、'.join(sorted(released))} 照旧带 --test-threads 1")
    return words + ([] if separator is not None else ["--"]) + ["--test-threads", threads]


def gpu_released_test_targets():
    """放开 --test-threads 1 的测试目标：scripts/gate/admission.py 的 GPU_TEST_THREADS_RELEASED_TARGETS，不另抄一份（with_gpu_test_threads 靠 init_worker 交的这个函数取）。"""
    return admission_module().GPU_TEST_THREADS_RELEASED_TARGETS


def test_runs_multi_host_configuration(environment):
    """真跑测试的格交给测试的多机配置（文件头「真跑测试的格把多机配置交给测试、带 gpu 特性的格跑完核真用了卡」）：(绝对路径, 文件在不在)。
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
    """text 里每一行 CRASH_POINT_REPLAY_GIVEN_STATES（给定状态引擎每一趟收尾打的那一行，crate singlefs-verification 的
    crash_amplification::GIVEN_STATES_LINE_PREFIX；_KEY_REFUSED 这类同前缀的别的行不算）的 states_judged_by_the_gpu_here，按出现的次序。"""
    line = re.compile(r"^\s*CRASH_POINT_REPLAY_GIVEN_STATES\s(?:.*\s)?states_judged_by_the_gpu_here=(\d+)(?=\s|$)", re.M)
    return [int(found.group(1)) for found in line.finditer(text)]


def gpu_card_unused_reason(row, output, environment):
    """带 gpu 特性、判出没红的那一条用没用上卡（文件头「真跑测试的格把多机配置交给测试、带 gpu 特性的格跑完核真用了卡」）：用不上交接在「点名的测试 <名字>」后面的那一句，
    用上了或判不到这一层（点名的测试不是恰好对上一个用例、结果不都是 ok）交 None。
    不打 CRASH_POINT_REPLAY_GIVEN_STATES 行的照常判（gpu_test_start 取卡：没卡打 NOT_RUN_NO_GPU_ADAPTER、有卡起不来就 panic，不悄悄改交 CPU）。
    弄坏开关 gpu-unused-as-missed：一律交 None；given-states-count-ignored：有那一行就当没用上卡，不读数。"""
    if break_is(environment, "gpu-unused-as-missed"):
        return None
    results = results_by_test(row.expected, output)
    if len(results) != 1 or not all(outcome == "ok" for outcomes in results.values() for outcome in outcomes):
        return None
    counts = given_states_gpu_counts(named_test_stdout(next(iter(results)), output))
    if counts and (break_is(environment, "given-states-count-ignored") or not any(counts)):
        return (f"带 gpu 特性，它打的 {len(counts)} 行 CRASH_POINT_REPLAY_GIVEN_STATES 里 states_judged_by_the_gpu_here 全是 0："
                "给定状态全交 CPU 判、GPU 一个状态都没判，改坏的 GPU 那一处没跑到")
    return None


def prune_server_temporary_service_module():
    """scripts/prune-server/prune_server_client_selftest.py（临时服务的起法：TestServer 与放它的临时目录 selftest_root），第一次起临时服务时才读进来。"""
    if not _PRUNE_SERVER_TEMPORARY_SERVICE_MODULE:
        specification = importlib.util.spec_from_file_location("prune_server_temporary_service_for_crates_mutation_rows", PRUNE_SERVER_TEMPORARY_SERVICE_SCRIPT)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _PRUNE_SERVER_TEMPORARY_SERVICE_MODULE.append(module)
    return _PRUNE_SERVER_TEMPORARY_SERVICE_MODULE[0]


@contextlib.contextmanager
def mutation_run_prune_server(binary, command, temporary_directory, environment):
    """起 cargo test 那一次自己的临时 prune-server（文件头「变异复跑的崩溃判定」）：交要加进那一次环境的三个变量（连哪一处与临时服务的标记，都指它），
    收尾停服务、删它的目录。服务的目录放这一次的 temporary_directory 里（那一格收尾的删与留下核也管它）；socket 全路径放不下 107 字节时照客户端自证的
    selftest_root 另取（TMPDIR 太长退到 /tmp）。只编不跑的（--no-run）不起、交空。弄坏开关 mutation-run-on-real-service：不起、交空（照调用方的环境连那一份服务）。
    服务起不来抛 RuntimeError（工作进程里抛的照原样炸到 run_pool，不吃掉）。"""
    if "--no-run" in command or break_is(environment, "mutation-run-on-real-service"):
        yield {}
        return
    module = prune_server_temporary_service_module()
    directory = tempfile.mkdtemp(prefix="prune-server-", dir=temporary_directory)
    for fallback in (module.selftest_root, lambda: tempfile.mkdtemp(prefix="prune-server-", dir="/tmp")):
        if len(os.path.join(directory, "service", "server.sock").encode()) <= module.MAXIMUM_SOCKET_PATH_BYTES:
            break
        os.rmdir(directory)
        directory = fallback()
    server = None
    try:
        server = module.TestServer(binary, os.path.join(directory, "service"))
        yield {module.SOCKET_VARIABLE: server.socket, module.TCP_ADDRESS_VARIABLE: server.tcp_address, module.TEMPORARY_SERVICE_VARIABLE: server.socket}
    finally:
        if server is not None:
            server.stop()
        shutil.rmtree(directory, ignore_errors=True)


def mutation_run_prune_server_binary(environment, shard_root):
    """变异复跑起临时 prune-server 用的可执行文件（文件头「变异复跑的崩溃判定」）：(路径, "") 或 (None, 为什么)。真二进制取 PRUNE_SERVER_BINARY（绝对路径、可执行）；
    没设照本模块所在仓的 prune-server/ 编一份进 <shard_root>/prune-server-build（这一趟的编译目录，收尾随 shard_root 删）。
    交出去的是 <shard_root> 里写的一份包装（PRUNE_SERVER_PARENT_DEATH_WRAPPER）：服务各在自己的会话里，起它的工作进程被停或被硬杀时 finally 不跑，
    包装先设父进程死亡信号再换成真二进制，工作进程一死服务就跟着停。"""
    configured = environment.get(PRUNE_SERVER_BINARY_VARIABLE, "").strip()
    if configured:
        if not (os.path.isabs(configured) and os.path.isfile(configured) and os.access(configured, os.X_OK)):
            return None, f"{PRUNE_SERVER_BINARY_VARIABLE}={configured} 不是可执行文件的绝对路径"
        binary = configured
    else:
        target = os.path.join(shard_root, PRUNE_SERVER_BUILD_DIRECTORY_NAME)
        manifest = os.path.join(STAGE_REPOSITORY, "prune-server", "Cargo.toml")
        completed = subprocess.run(["cargo", "build", "--release", "--offline", "--manifest-path", manifest, "--bin", "prune-server"],
                                   env=dict(environment, CARGO_TARGET_DIR=target), capture_output=True, text=True, check=False)
        binary = os.path.join(target, "release", "prune-server")
        if completed.returncode != 0 or not os.access(binary, os.X_OK):
            return None, f"cargo build --release --offline --manifest-path {manifest} --bin prune-server 退 {completed.returncode}：{completed.stderr.strip()[-400:]}"
    wrapper = os.path.join(shard_root, PRUNE_SERVER_WRAPPER_NAME)
    with open(wrapper, "x", encoding="utf-8") as handle:
        handle.write(PRUNE_SERVER_PARENT_DEATH_WRAPPER.replace("REAL_BINARY_PATH", repr(binary)))
    os.chmod(wrapper, 0o755)
    return wrapper, ""


def run_cargo_command(command, temporary_directory, label, extra_environment=None):
    """起一条 cargo 命令（cargo test --offline … 这一类）：(subprocess 的结果, 用时)。不套内存包装与清场包装、不设上限与限时：一整条门禁交给统一任务服务（task-server/），
    内存上限由服务按记录给、撞了放大重排，卡住的由服务按日志与 CPU 都不涨停整条任务，清场与分卡由服务在最外层套（文件头「带 gpu 特性的格不另套清场」）。TMPDIR 是这一次自己的；extra_environment 只加在这一次的环境里
    （覆盖那一格的编译目录、插桩开关与 LLVM_PROFILE_FILE）。label 只用在 stderr 的那几句里（基线的批、变异表第几行、覆盖）。
    真跑测试的那一次对它自己的临时 prune-server 跑（mutation_run_prune_server，经 init_worker 交的引用调，不进判法单元的闭包；文件头「变异复跑的崩溃判定」「判法单元」），
    跑完停、删。"""
    state, context = WORKER_STATE, WORKER_STATE["context"]
    command = with_gpu_test_threads(command, context["environment"], state["gpu_released_targets"])
    environment = dict(state["environment"], TMPDIR=temporary_directory)
    environment.update(extra_environment or {})
    # 真跑测试的每一次把多机配置交给测试（文件头「真跑测试的格把多机配置交给测试」）：副本里它自己找不到，crates 那侧找不到就报错；
    # 文件在不在 run_stage 起 cargo 之前判过。弄坏开关 test-runs-configuration-unpassed：不设
    if "--no-run" not in command and not break_is(context["environment"], "test-runs-configuration-unpassed"):
        environment["SINGLEFS_MULTI_HOST_CONFIG"] = state["multi_host_configuration"](context["environment"])[0]   # init_worker 交的 test_runs_multi_host_configuration
    with state["temporary_service"](context["prune_server_binary"], command, temporary_directory, context["environment"]) as service_settings:   # init_worker 交的 mutation_run_prune_server
        environment.update(service_settings)
        started = time.time()
        run = state["launch"](command, cwd=state["work"], env=environment)   # init_worker 交的 run_capturing_output
        seconds_running = time.time() - started
    return run, {"seconds_running": f"{seconds_running:.3f}"}


def uncounted_verdict_keeps_counted_record(existing, verdict, row_key, environment):
    """这一条要写进 records/<row_key> 的结局 verdict 不算数（VERDICTS_UNCOUNTED），而库里同一把行键原有的记录 existing（读不到交 None）是算数的结论
    （VERDICTS_COUNTED）时交 True：这一条不写，原有的留着（文件头「复用判据」记红撤绿那一段）；judge_row 与 judge_batch 共用这一份。
    弄坏开关 uncounted-overwrites-counted：一律交 False，不算数的照写、盖掉原有的算数结论。"""
    if break_is(environment, "uncounted-overwrites-counted"):
        return False
    return (verdict in VERDICTS_UNCOUNTED and existing is not None and existing.get("row_key") == row_key
            and existing.get("verdict") in VERDICTS_COUNTED)


def write_record_quietly(kind, key, fields):
    """写一条判定库记录；写不进去交那一句（交回父进程报在 stderr，下一趟这一条照跑），写成交 None。"""
    try:
        WORKER_STATE["library"].write(kind, key, fields)
    except OSError as error:
        return f"{WORKER_STATE['library'].path(kind, key)}：{error}"
    return None


def how_text(context, kind):
    """记录里的跑法：哪一道、什么起法、第几片（只是一样属性，不进键）。"""
    return f"{STAGE} {kind}（直接起 cargo，不设上限与限时；第 {WORKER_STATE['worker_sequence']} 片，共 {context['worker_count']} 片）"


def run_baseline_batch(key, batch_rows, context):
    """一批基线在没改坏的副本上跑一趟：(cargo test 的参数, 跑的结果, 用时)。"""
    arguments = baseline_arguments(key, batch_rows)
    temporary_directory = tempfile.mkdtemp(prefix=f"baseline-{batch_rows[0].line_number}-", dir=WORKER_STATE["per_mutation_temporary_root"])
    try:
        run, seconds = run_cargo_command(["cargo", "test", "--offline"] + arguments, temporary_directory, f"基线（批首行是变异表第 {batch_rows[0].line_number} 行）")
    finally:
        measure_and_remove_temporary_directory(temporary_directory, context)
    return arguments, run, seconds


def judge_batch(batch):
    """进程池里的一格活：一批基线（没改坏的副本上跑一遍这一批各行点名的测试）。回 (每行的 (行号, 问题或 None), 那一批的尾巴, 写不进去的记录)。
    一行的基线结局不随这一趟拼进了哪几行而变（文件头「基线」）：每行只看它自己的过滤词选得中的结果行（row_baseline_output）。
    弄坏开关 baseline-batch-composition：照整批那一趟的输出判。"""
    key, batch_rows = batch
    context = WORKER_STATE["context"]
    arguments, run, seconds = run_baseline_batch(key, batch_rows, context)
    judged, record_problems = [], []
    for row in batch_rows:
        output = row_baseline_output(row, run.stdout + run.stderr, context["environment"])
        problem = baseline_problem(row, run.returncode, output)
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
                batch_exit_code=run.returncode, judged_date=mutation_units.today_text(), how=how_text(context, "基线"), **seconds))
            if failed_to_write:
                record_problems.append(failed_to_write)
        # 记红、撤绿（文件头「复用判据」）：基线不绿的这一行，按它的行键记一条「基线不绿」，盖掉同一把行键原有的「抓到」，下一趟它照现判（先跑基线）。
        # 弄坏开关 baseline-red-keeps-caught：不记，同一把行键上次抓到的那一份照复用、基线也不再跑
        if problem is not None and components.get("row_key") and not break_is(context["environment"], "baseline-red-keeps-caught"):
            fields = collections.OrderedDict(
                kind="mutation", row_key=components["row_key"], judging=context["judging"], verdict="baseline-not-ok",
                text=f"{TABLE}:{row.line_number} {row.name}：基线不绿——点名的测试 {row.expected} 在没改坏的副本上{problem}",
                exit_code=run.returncode, judged_date=mutation_units.today_text(), how=how_text(context, "基线"),
                test_closure=components.get("test_closure", ""), mutated=components.get("mutated", ""), toolchain=components.get("toolchain", ""),
                dependency=components.get("dependency", ""))
            failed_to_write = write_record_quietly("records", components["row_key"], fields)
            if failed_to_write:
                record_problems.append(failed_to_write)
    return judged, record_problems


def judge_row(row):
    """进程池里的一格活：一条变异。建它自己的 TMPDIR、改坏那一处、跑点名的测试、还原、删掉那个 TMPDIR，判完立刻写判定库记录与用时表。
    回 (((行号, 档, 那一条要打印的话, 删 TMPDIR 的结果), {test_full_name}), 写不进去的记录)。"""
    judge_started = time.monotonic()
    context = WORKER_STATE["context"]
    full = os.path.join(WORKER_STATE["work"], row.path)
    with open(full, encoding="utf-8") as handle:
        pristine = handle.read()
    if pristine.count(row.old) != 1:
        # 派活之后源码被改、这一份工作副本里原文不再恰好一次（文件头「锚点失效」）：照锚点失效记这一行，不改坏、不跑、不入库，别的行照判
        # 弄坏开关 anchor-mid-run-asserts：在这里抛 AssertionError，整个进程池跟着炸
        assert not break_is(context["environment"], "anchor-mid-run-asserts"), (row.path, row.name)
        why = f"派活之后源码被改了：工作副本里的原文命中 {pristine.count(row.old)} 次，要恰好 1 次"
        return ((row.line_number, "anchor", anchor_broken_text(row, why), None), {}), []
    temporary_directory = tempfile.mkdtemp(prefix=f"line-{row.line_number}-", dir=WORKER_STATE["per_mutation_temporary_root"])
    with open(full, "w", encoding="utf-8") as handle:
        handle.write(pristine.replace(row.old, row.new))
    try:
        run, seconds = run_cargo_command(["cargo", "test", "--offline"] + with_shown_output(row.arguments.split()), temporary_directory,
                                         f"变异表第 {row.line_number} 行")
    finally:
        with open(full, "w", encoding="utf-8") as handle:
            handle.write(pristine)
        # 不论退出码是几都删（含被信号杀掉的测试进程）：被杀的测试 Drop 守卫不跑，它建的镜像只有这里删得掉
        removal = measure_and_remove_temporary_directory(temporary_directory, context)
    output = run.stdout + run.stderr
    verdict, text = verdict_of_run(row, run.returncode, output, context["environment"])
    # 带 gpu 特性、判出没红的那一条核用没用上卡（文件头「真跑测试的格把多机配置交给测试、带 gpu 特性的格跑完核真用了卡」）：用不上记本次未跑，不进没红
    unused = gpu_card_unused_reason(row, output, context["environment"]) if verdict == "failure" and gpu_test_run(
        ["cargo", "test"] + row.arguments.split(), context["environment"]) else None
    if unused is not None:
        verdict, text = "notrun", f"{TABLE}:{row.line_number} {row.name}：本次未跑——点名的测试 {row.expected} {unused}；不算抓到、也不算没红"
    components = context["row_records"].get(row.line_number) or {}
    full_name = single_full_name(row.expected, output) or ""
    record_problems = []
    # 判出的每一档都照行键写（文件头「复用判据」）：没红、无效这些不是「抓到」的盖掉同一把行键原有的「抓到」，下一趟照现判。
    # 弄坏开关 red-not-recorded：只记「抓到」（不是抓到的不写，原有的「抓到」留着照复用）
    # 不算数的结局（VERDICTS_UNCOUNTED）不盖同一把行键原有的算数结论：键相同仍现判（复用哨兵强制现判）时才碰得到；库里没有算数结论的照写
    keeps_counted = bool(components.get("row_key")) and verdict in VERDICTS_UNCOUNTED and uncounted_verdict_keeps_counted_record(
        WORKER_STATE["library"].read("records", components["row_key"]), verdict, components["row_key"], context["environment"])
    if components.get("row_key") and not keeps_counted and (verdict == REUSABLE_VERDICT["mutation"] or not break_is(context["environment"], "red-not-recorded")):
        fields = collections.OrderedDict(
            kind="mutation", row_key=components["row_key"], judging=context["judging"], verdict=verdict, text=text,
            removal_files=removal[0], removal_allocated_bytes=removal[1], removal_apparent_bytes=removal[2], removal_left=int(removal[3]),
            exit_code=run.returncode, judged_date=mutation_units.today_text(), how=how_text(context, "现判"), test_full_name=full_name,
            test_closure=components.get("test_closure", ""), mutated=components.get("mutated", ""), toolchain=components.get("toolchain", ""),
            dependency=components.get("dependency", ""))
        fields.update(seconds)
        failed_to_write = write_record_quietly("records", components["row_key"], fields)
        if failed_to_write:
            record_problems.append(failed_to_write)
    timing_path = os.path.join(context["timings_directory"], row_content_hash(row))
    try:
        write_atomically(timing_path, mutation_units.record_text(RECORD_HEADER, dict(
            row_content_hash=row_content_hash(row), seconds_running=seconds["seconds_running"], judged_date=mutation_units.today_text())))
    except OSError as error:
        record_problems.append(f"{timing_path}：{error}")
    return ((row.line_number, verdict, text, removal), dict(test_full_name=full_name, wall_seconds=time.monotonic() - judge_started)), record_problems


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
    # 带 -Cinstrument-coverage 的编译：cargo 没带 --target 时 RUSTFLAGS 也罩构建脚本与过程宏，它们在各自的源码目录（~/.cargo/registry/src/ 下依赖包的目录）里
    # 起，没设 LLVM_PROFILE_FILE 就在那里写 default_<哈希>_<进程号>.profraw；编译这一步同样把剖析文件指进这一格的 TMPDIR（不并进 profiles：那里只放测试那一趟的）
    build_profiles = os.path.join(temporary_directory, BUILD_PROFILES_DIRECTORY_NAME)
    os.mkdir(build_profiles)
    build_overrides = overrides if break_is(context["environment"], "buildprofilesunset") else dict(
        overrides, LLVM_PROFILE_FILE=os.path.join(build_profiles, "build-%p-%m.profraw"))
    label = f"覆盖（组首行是变异表第 {job['line']} 行）"
    build, _seconds = run_cargo_command(["cargo", *configuration, "test", "--offline", *job["before"], "--no-run", "--message-format=json-render-diagnostics"],
                                        temporary_directory, label, build_overrides)
    if build.returncode != 0:
        return f"插桩编译没成（退出码 {build.returncode}）：{output_tail(build.stderr, 3)}"
    messages = []
    for line in build.stdout.splitlines():
        if line.startswith("{"):
            with contextlib.suppress(ValueError):
                messages.append(json.loads(line))
    executables = executables_of_build(messages, state["work"])
    if not executables:
        return "插桩编译没交出一个本仓的可执行文件"
    # 点名用例前自己补 --exact：行的 libtest 参数里已经带了的不再重复补（libtest 报 Option 'exact' given more than once、退 101，覆盖就记不成）；
    # 弄坏开关 exactduplicated：原样接在后面
    switches = list(job["switches"]) if break_is(context["environment"], "exactduplicated") else [word for word in job["switches"] if word != "--exact"]
    run, _seconds = run_cargo_command(["cargo", *configuration, "test", "--offline", *job["before"], "--", *switches, "--exact", job["test"]],
                                      temporary_directory, label, dict(overrides, LLVM_PROFILE_FILE=os.path.join(profiles, "cov-%p-%m.profraw")))
    output = run.stdout + run.stderr
    results = results_by_test(job["test"], output)
    if run.returncode != 0 or len(results) != 1 or any(result != "ok" for result in next(iter(results.values()))):
        return (f"插桩跑 {job['test']} 没恰好一个 ok（退出码 {run.returncode}；"
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
    # ④ 执行过的单元：计数 > 0 的函数落进的 fn 单元，连同点名测试闭包里不是 fn 的单元（常量、类型、静态项、宏、数据文件），都在这份副本上切；
    #   单元键按这一行的包目录取（--manifest-path 指的包用它自己那一份，文件头「--manifest-path」），同一个包目录在这个工作进程里只建一份
    directory, why = manifest_directory_of(state["work"], job["arguments"], context["environment"])
    if why is not None:
        return f"副本里认不出 cargo 参数的 --manifest-path（{why}）"
    if directory not in state["unit_keys"]:
        state["unit_keys"][directory] = unit_keys_for_directory(state["work"], directory)
    keys = state["unit_keys"][directory]
    # 覆盖记的是副本上执行过的单元，没有被改坏的那一行：一律取产品行为视图（这里的 UnitKeys 没问过 mutated_units）
    test, refusal = keys.resolve_test(job["arguments"], job["expected"], view=mutation_units.function_units().PRODUCT_BEHAVIOR)
    if refusal is not None:
        return f"副本里认不出点名的测试（{refusal}）"
    closure = keys.closure(test.unit, test.target, job["arguments"])
    executed, unplaced = keys.executed_units(export, state["work"])
    members = {(reference.kind, reference.file, reference.name, reference.fingerprint): reference
               for reference in executed + [member for member in closure.members if member.kind != "fn"]}
    state["library"].write_json("coverage", job["coverage_id"], dict(
        test=list(mutation_units.reference_of(test.unit)), executed=sorted(list(reference) for reference in members.values()),
        executed_functions=len(executed), unplaced_functions=unplaced, complete=unplaced == 0 and bool(executed),
        executables=len(executables), judged_date=mutation_units.today_text(), how=how_text(context, "覆盖")))
    return None


# ── 整道 ──────────────────────────────────────────────────────────────────────

# gate-overlap:copy-kept scripts/mutation/mutate.sh 那边的 worker_count_and_head_line 照这里的「各项取最小、至少 1、头一项是每台默认重活线程数」写了一份 bash：两边的项不同（这里 GATE_MUTATION_WORKERS 顶掉登记值那一项、第二项是要现跑的条数、第三项是磁盘余量；那边没有第三项，MUTATE_JOBS 只能压小、第二项是表里的条数、另算每个 cargo 的编译并行度），派活一边是本进程的进程池、一边是 bash 子 shell 按目录认领，共用的只剩一个取最小值，抽成共用要让 mutate.sh 每趟多起一个 python 只为算一个数、两边头一行的字也不同
def bytes_of_size(text):
    """systemd 写法的大小（正整数加 K / M / G / T，1024 进制）→ 字节；别的写法交 None。"""
    matched = SIZE_TEXT.match(text.strip())
    return int(matched.group(1)) * SIZE_UNIT_BYTES[matched.group(2)] if matched else None


def worker_count_for(row_count, environment, disk=None, memory=None):
    """(开几个工作进程, 头一行要报的话)。四项取最小、至少 1：每台默认重活线程数
    （GATE_MUTATION_WORKERS 顶掉它；取不到抛 HeavyThreadDefaultError）、这一趟要现跑的条数、磁盘余量放得下几个工作进程的编译目录（disk 给了才有这一项）、
    内存上限 ÷ 每个工作进程的峰值（memory 给了才有这一项）。
    disk 与 memory 都是函数：前面几项取最小的那个数当天花板（ceiling）传给它，它交回 (放得下几个或 None, 头一行要写的话)，None 是没有这一项，头一行写明原因
    （disk_bound_for，文件头「磁盘余量」；memory_bound_for，文件头「内存上限」）。内存那一项排在最后：它按最后开的进程数算编译并行度，天花板要含磁盘那一项。
    一条变异的挂钟 = 重编译那几个 crate + 跑点名的那个测试，测试那一段是单线程的，所以开得比核数 ÷ 4 多，再用 GATE_MUTATION_CARGO_JOBS
    把每个 cargo 的编译并行度收住。"""
    bounds = []
    requested = environment.get("GATE_MUTATION_WORKERS", "").strip()
    if requested:
        bounds.append(("GATE_MUTATION_WORKERS", int(requested), f"GATE_MUTATION_WORKERS 给 {int(requested)} 个"))
    elif break_is(environment, "workers-by-processor-half"):
        processor_count = os.cpu_count() or 4
        by_processor = max(1, min(16, processor_count // 2))
        bounds.append(("核数", by_processor, f"核数 {processor_count} 的一半、至多 16，给 {by_processor} 个（弄坏开关 workers-by-processor-half）"))
    else:
        registered = int(default_heavy_threads(environment))   # 取不到抛 HeavyThreadDefaultError，run_stage 判红退 2
        bounds.append(("每台默认重活线程数", registered, f"每台默认重活线程数 {registered}（scripts/hosts/heavy_threads.py：SINGLEFS_THREAD_CAP 或 .claude/heavy-thread-default）"))
    # 要判的单位（文件头「工作进程数」）：工作进程数不超过它，不照拿到的上限开满；弄坏开关 workers-beyond-rows 下不收这一项
    if not break_is(environment, "workers-beyond-rows"):
        bounds.append(("这一趟要现跑的条数", row_count, f"这一趟要现跑 {row_count} 条（没有作数的抓到记录的；基线的批与覆盖跟着这几行走，不另算工作进程）"))
    disk_unread = None
    if disk is not None:
        disk_count, disk_text = disk(max(1, min(count for _name, count, _text in bounds)))
        if disk_count is None:
            disk_unread = disk_text
        else:
            bounds.append(("磁盘余量", disk_count, disk_text))
    memory_unlimited = None
    if memory is not None:
        memory_count, memory_text = memory(max(1, min(count for _name, count, _text in bounds)))
        if memory_count is None:
            memory_unlimited = memory_text
        else:
            bounds.append(("内存上限", memory_count, memory_text))
    smallest = min(count for _name, count, _text in bounds)
    binding = "、".join(f"「{name}」" for name, count, _text in bounds if count == smallest)
    floor_note = "（那一项不足 1，按 1 个开）" if smallest < 1 else ""
    worker_count = max(1, smallest)
    head_line = (f"  … 这一轮开 {worker_count} 个工作进程（各项取最小、至少 1）：{'；'.join(text for _name, _count, text in bounds)}。"
                 f"卡在{binding}这一项{floor_note}")
    if memory_unlimited is not None:
        head_line += f"；没有内存上限这一项（{memory_unlimited}），工作进程数不按内存收"
    if disk_unread is not None:
        head_line += f"；磁盘余量没读到（{disk_unread}），工作进程数不按磁盘收"
    return worker_count, head_line


def new_shard_root_path():
    """$TMPDIR 下 SHARD_ROOT_PREFIX 起头加八位随机字符的路径，只起名、不建。tempfile.mkdtemp 是在 mkdir 里把目录建出来、返回之后名字才到调用方手里：
    TERM 落在这两步之间，收尾的 rmtree 不知道名字，空目录就漏在 $TMPDIR。这里名字先交到调用方、记进 shard_root，再由调用方 os.mkdir，
    收尾删的就是这个名字（名字在、目录不在也无妨）。"""
    alphabet = "abcdefghijklmnopqrstuvwxyz0123456789_"
    chooser = random.SystemRandom()
    return os.path.join(tempfile.gettempdir(), SHARD_ROOT_PREFIX + "".join(chooser.choice(alphabet) for _ in range(8)))


def run_pool(context, worker_count, batches, ordered_rows, baseline_ok_lines, stop_after, coverage_planner, item_progress=None):
    """一个进程池里先派基线、再派变异、最后派覆盖：一批基线判完，它 ok 的行进变异的队；已有基线 ok 的行一开始就在队里；
    一条变异判成抓到、它那一组还要记覆盖的（coverage_planner.job_after），那一组的覆盖进队；开跑时就要记的（coverage_planner.initial_jobs）一开始就在队里。
    同时在跑的至多 worker_count 格，空出一格补一格（谁先跑完谁再领），基线优先、变异按 ordered_rows 的次序、覆盖最后。
    回 dict：mutation（变异的判定）、mutation_info（{行号: {snapshot, test_full_name}}）、baseline（{行号: (问题或 None, 尾巴)}）、coverage（覆盖那几格的结果）、
    submitted、received、stopped、left_behind、record_problems。item_progress 是 ItemProgress（按项机读行，文件头同名一段）：基线不绿与变异判完的行在这里报结局。"""
    sequence_counter = multiprocessing.Value("i", 0)
    # 被停的信号也要删掉 shard_root（$TMPDIR 下的 singlefs-mutation-replay-*）：先接住再建，建完之后收到的都走 finally；
    # 接信号、起名、建目录都在 try 里：哪一步之后收到都走 finally（信号处理还原、按已记下的名字删目录），名字先记进 shard_root 再 os.mkdir（new_shard_root_path）。
    # 只在主线程接得了。弄坏开关 shardrootkeptonsignal 下不接：收到就停，shard_root 留在 $TMPDIR；
    # 弄坏开关 shardrootnamedlate 下用 tempfile.mkdtemp 建（名字要等它返回才记得下）
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
        os.mkdir(target_over_cap_directory(shard_root))   # 停派目录（文件头「编译目录上限」）：工作进程清完仍超上限往里写
        # 这一趟自己的编译目录（文件头「编译目录」）：调用方没给 GATE_MUTATION_TARGET_DIR 就建在 <shard_root>/build 里，收尾（finally）删
        if context["shared_target_directory"] is None:
            build_root = os.path.join(shard_root, RUN_BUILD_DIRECTORY_NAME)
            os.mkdir(build_root)
            context["shared_target_directory"] = run_build_target_directory(shard_root)
            for line in target_reclaim_lines(context["shared_target_directory"], worker_count, ([], [], []), owned=True):
                note(line)
        # 先编一份依赖（文件头「先编一份依赖」）：工作进程起来之前编好，编成的由 init_worker 拷进各自的编译目录；摸过的包没重编整道判红、一个工作进程都不起
        # 底座读数（文件头「内存上限」的「底座读数」）：父进程自己此刻的用量，与先编一份依赖那一步（工作进程还没起）父进程整棵树的峰值，口径同每个工作进程的读数
        base_parent_bytes = process_usage_bytes(os.getpid())
        seed_tree_peak, seed_sampling_done = [None], threading.Event()

        def sample_seed_step_tree():
            while True:
                usage = process_tree_usage(os.getpid(), context["environment"])
                if usage is not None:
                    seed_tree_peak[0] = max(seed_tree_peak[0] or 0, usage)
                if seed_sampling_done.wait(WORKER_PEAK_SAMPLE_SECONDS):
                    return
        seed_sampler = threading.Thread(target=sample_seed_step_tree, daemon=True)
        seed_sampler.start()
        try:
            seed_lines, seed_problems = prepare_seed(context, batches, ordered_rows, coverage_planner, shard_root)
        finally:
            seed_sampling_done.set()
            seed_sampler.join(timeout=5)
        for line in seed_lines:
            note(line)
        note(base_reading_line(base_parent_bytes, seed_tree_peak[0], bool(context["seed_targets"])))
        if seed_problems:
            return dict(mutation=[], mutation_info={}, baseline={}, coverage=[], submitted=0, received=0, stopped=False, left_behind=[], record_problems=[],
                        left_behind_contents={}, per_mutation_temporary_root=per_mutation_temporary_root, seed_problems=seed_problems)
        # 变异复跑各起的临时 prune-server 用哪一份二进制（文件头「变异复跑的崩溃判定」）：工作进程起来之前定下，交给它们；取不到一个工作进程都不起
        context["prune_server_binary"], prune_server_problem = mutation_run_prune_server_binary(context["environment"], shard_root)
        if prune_server_problem:
            return dict(mutation=[], mutation_info={}, baseline={}, coverage=[], submitted=0, received=0, stopped=False, left_behind=[], record_problems=[],
                        left_behind_contents={}, per_mutation_temporary_root=per_mutation_temporary_root, prune_server_problem=prune_server_problem)
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
        jobs = {"baseline": baseline_job, "mutation": mutation_job, "coverage": coverage_job}   # 各判一格，判完清一次自己那份超线的编译目录
        baseline_one_at_a_time = break_is(context["environment"], "baseline-one-at-a-time")
        with concurrent.futures.ProcessPoolExecutor(max_workers=worker_count, initializer=init_worker,
                                                    initargs=(sequence_counter, shard_root, context)) as pool, own_workers_stopped_on_stop(pool, shard_root, context["environment"]):
            while True:
                while ready and len(in_flight) < worker_count and not outcome["stopped"]:
                    entry = heapq.heappop(ready)
                    if baseline_one_at_a_time and entry[3] == "baseline" and "baseline" in in_flight.values():
                        # 弄坏开关（只给证红用）：基线的批一次只派一批，在跑的那一批收完才派下一批
                        heapq.heappush(ready, entry)
                        break
                    future = pool.submit(jobs[entry[3]], entry[4])
                    in_flight[future] = entry[3]
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
                            if problem is not None and item_progress is not None:
                                item_progress.settle(line_number, "baseline")
                            if problem is None and line_number in order_index:
                                heapq.heappush(ready, (1, order_index[line_number], serial, "mutation", rows_by_line[line_number]))
                                serial += 1
                    elif kind == "mutation":
                        judgement, info = result
                        outcome["mutation"].append(judgement)
                        outcome["mutation_info"][judgement[0]] = info
                        if item_progress is not None:
                            item_progress.settle(judgement[0], judgement[1], seconds=info.get("wall_seconds", 0.0))
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


def broken_pool_error_type(environment):
    """「有工作进程中途死了」认的异常类：模块载入时绑定的 BrokenProcessPool。concurrent.futures 不自己载入 process 子模块，进程池还没起（先编一份依赖那一段）
    就被停时按 concurrent.futures.process 属性找它，判 except 的那一刻抛 AttributeError、盖掉原来的 StopRequested（被停退 143 成了回溯退 1）。
    弄坏开关 broken-pool-looked-up-late：在判 except 的时候按属性找。"""
    if break_is(environment, "broken-pool-looked-up-late"):
        return concurrent.futures.process.BrokenProcessPool
    return BrokenProcessPool


def run_pool_reporting_dead_worker(context, worker_count, batches, ordered_rows, baseline_ok_lines, stop_after, coverage_planner, item_progress=None):
    """run_pool 的结果；有工作进程被硬杀（段错误、被内核杀）时打「有工作进程中途死了」、交 None。别的异常（被停的 StopRequested 这类）原样往外抛。"""
    try:
        return run_pool(context, worker_count, batches, ordered_rows, baseline_ok_lines, stop_after, coverage_planner, item_progress)
    except broken_pool_error_type(context["environment"]):
        # 一个工作进程被硬杀时进程池当场抛这个，而不是安静地少产出几项
        print_rejection("有工作进程中途死了，这一轮的变异没跑全", [],
                        ["整条门禁撞了统一任务服务给的内存上限时由服务放大重排，不在这里调；别的死法（段错误）单跑 GATE_MUTATION_WORKERS=1 看是不是并发度的问题，",
                         "还死就去看那一条变异本身（判完的那几条有判定库记录，下一趟只跑剩下的）。"])
        return None


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
    "上面每处点名了测试和变异表的行：测试带 #[cfg(feature = \"…\")] 的，给那几行的 cargo 参数补上编它要的特性（例 --features gpu，写法照同一张表里别的带 gpu 的行），",
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
    弄坏开关 anchor-stops-table：锚点失效也让整道判红。"""
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
    try:
        copied = copied_with_path_dependencies(root, copied, environment)
    except (ValueError, OSError) as error:
        print_rejection(f"拷贝范围跟不全 path 依赖（{error}）：判不出每片的源码副本该拷哪些，一个工作进程都没起", [],
                        ["path 依赖指到被判那棵树之外的，改成树里的目录（判定库服务 prune-server/ 这一类放仓根下）；Cargo.toml 读不懂的照报错修，再跑。",
                         COPY_REBUILD_HOWTO + "。"])
        return None, True, {}
    try:
        copied = copied_with_manifest_directories(root, copied, rows, environment)
    except (ValueError, OSError) as error:
        print_rejection(f"拷贝范围补不上变异行 --manifest-path 指的包目录（{error}）：判不出每片的源码副本该拷哪些，一个工作进程都没起", [],
                        ["那一行的 --manifest-path 写成仓内一份 Cargo.toml 相对仓根的路径（例 --manifest-path prune-server/Cargo.toml），只写一次；",
                         "那个包的 path 依赖指到树之外的改成树里的目录；改完先跑 python3 scripts/mutation/crates-mutation-rows.py static-check <根>。"])
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


def baseline_check(library, row, components, key_context, snapshot, environment):
    """(基线 ok 复用得上没有, 不复用的那一句)。拒算的行（没有基线键）照跑。
    覆盖记录里执行过的单元照这一行的单元键核（KeyContext.keys_for：--manifest-path 指的包用它自己那一份，文件头「--manifest-path」）。"""
    if not components.baseline_key:
        return False, components.refusal or "没有基线键"
    keys, _refusal = key_context.keys_for(row)
    record = library.read("baselines", components.baseline_key)
    coverage = library.read_json("coverage", components.coverage_id) if components.coverage_id else None
    problem = baseline_reuse_problem(record, components, keys, coverage, snapshot, environment)
    return problem is None, problem


def plan_rows(root, rows, environment, judging_repository=None, table_rows=()):
    """(KeyContext, 这一趟的行, {行号: Components}, 判定库)：run_stage、plan、row-plan、--list-items、import 共用这一份算法；只读（不写库）。
    交回的行：命中重测试目标而没带 release 的换成补了 --release 的（release_forced_row，文件头「重测试目标只许 release」），
    多机配置开了 GPU 时点名测试走崩溃判段的那几行换成补了 gpu 特性的（gpu_route_rows），别的原样；键都照换了之后的参数算。
    table_rows 是整张表成形的行（选行时给）：没选中、而原文与替换文都与选中的某一行相同的行（行键撞得上的只有这几行）一起算键，撞键照整张表判（refuse_colliding_row_keys）。
    重测试目标的登记表读不了抛 lib_release_only_tests.py 的 RegistryUnreadable（调用方拒、一条都不跑）。"""
    key_context = KeyContext(root, environment, judging_repository)
    planned_lines = {row.line_number for row in rows}
    texts = {(row.old, row.new) for row in rows}
    peers = [row for row in table_rows if row.line_number not in planned_lines and (row.old, row.new) in texts]
    key_context.release_registered = release_registered_targets()
    forced_rows = [release_forced_row(row, key_context.release_registered, environment) for row in rows]
    forced_peers = [release_forced_row(row, key_context.release_registered, environment) for row in peers]
    # 报出来的只是这一趟判的行：一起算键的别的行（peers）补没补 --release 不报
    key_context.release_lines = [(row.line_number, basis) for row, basis in forced_rows if basis is not None]
    rows, peers = [row for row, _basis in forced_rows], [row for row, _basis in forced_peers]
    key_context.prepare(list(rows) + peers)
    components = {row.line_number: key_context.components(row) for row in list(rows) + peers}
    every_row, components = gpu_route_rows(key_context, list(rows) + peers, components, environment)
    key_context.collisions = refuse_colliding_row_keys(components, environment)
    # 报出来的只是这一趟判的行：一起算键的别的行（peers）补没补特性不报
    key_context.gpu_route_lines = [item for item in key_context.gpu_route_lines if item[0] in planned_lines]
    key_context.gpu_route_unbuilt = [item for item in key_context.gpu_route_unbuilt if item[0] in planned_lines]
    key_context.release_lines = sorted(item for item in key_context.release_lines if item[0] in planned_lines)
    rows = [row for row in every_row if row.line_number in planned_lines]
    return key_context, rows, {row.line_number: components[row.line_number] for row in rows}, open_library(root, environment)


def refuse_colliding_row_keys(components, environment):
    """表里两行以上算出同一把行键时这几行拒算（文件头「行键」）：行键不带路径与名字（D13（验证路线） 已定项 17），
    两份内容相同的函数各改一处、点名同一条测试的两行算得出同一把键，谁的判定都不能替另一行作保；拒算的照跑、不复用、不写记录。
    就地换掉 components 里撞键的那几份（row_key 置空、refusal 写「行键撞键：与第 N 行同键」，基线键与覆盖号不动：它们不带被改坏的单元，两行各自的基线照常复用），
    交撞键的几组行号（每组从小到大，组按头一个行号排）。弄坏开关 row-key-collision-unrefused：不判撞键（撞键的两行共用一把键、互相复用）。"""
    if break_is(environment, "row-key-collision-unrefused"):
        return []
    lines_by_key = collections.defaultdict(list)
    for line_number, comp in components.items():
        if comp.row_key is not None:
            lines_by_key[comp.row_key].append(line_number)
    groups = sorted(sorted(lines) for lines in lines_by_key.values() if len(lines) > 1)
    for lines in groups:
        for line_number in lines:
            others = "、".join(str(other) for other in lines if other != line_number)
            components[line_number] = components[line_number]._replace(row_key=None, refusal=f"{COLLISION_REFUSAL_PREFIX}与第 {others} 行同键")
    return groups


def gpu_route_enabled(environment):
    """多机配置开没开 GPU（文件头「开 GPU 时崩溃判段那几行补 gpu 特性」）：配置照 test_runs_multi_host_configuration 找（真跑测试的格交给测试的就是这一份），
    写了 ENABLE_GPU=1 交 (True, 路径)，别的（不在、没写、写的不是 1）交 (False, 路径)。在而读不了抛 admission.py 的 InputManifestError。"""
    path, present = test_runs_multi_host_configuration(environment)
    return present and admission_module().configuration_value(path, "ENABLE_GPU") == "1", path


def gpu_route_feature_word(row, components, key_context):
    """这一行要补的特性写法（点名测试的单元闭包够得着 GPU_ROUTE_UNITS 里的一处、cargo 参数里还没开 gpu 时）：cargo 参数的 -p 点名了那一处的包写 gpu，
    没点名写 <那一处的包>/gpu；不用补（闭包够不着、已开了 gpu 或 --all-features、闭包算不出）交 None。"""
    closure = key_context.closures.get(components.test_closure) if components.test_closure else None
    return None if closure is None else gpu_route_feature_word_of(row.arguments, closure)


def gpu_route_package(closure):
    """点名测试的单元闭包（ClosureInfo）够得着 GPU_ROUTE_UNITS 里的哪一处：交那一处的包，够不着交 None。"""
    wanted = {(file, name): package for package, file, name in GPU_ROUTE_UNITS[0]}
    for member in closure.members:
        package = wanted.get((member.file, member.name.split("::")[-1]))
        if package is not None:
            return package
    return None


def gpu_feature_on(arguments):
    """解析过的 cargo 参数里已开了 gpu（--features 里有 gpu 或 <包>/gpu，或 --all-features）。"""
    return arguments.all_features or any(feature.split("/")[-1] == GPU_ROUTE_FEATURE for feature in arguments.features)


def gpu_route_feature_word_of(arguments_text, closure):
    """同 gpu_route_feature_word，闭包由调用方给（ClosureInfo）：cargo 参数里没开 gpu、闭包够得着选路函数时交要补的特性写法，否则交 None。"""
    arguments = mutation_units.parse_cargo_arguments(arguments_text)
    if gpu_feature_on(arguments):
        return None
    package = gpu_route_package(closure)
    if package is None:
        return None
    return GPU_ROUTE_FEATURE if package in arguments.packages else f"{package}/{GPU_ROUTE_FEATURE}"


def with_feature(arguments, feature):
    """cargo 参数里补一个 --features <特性>：插在第一个 `--` 之前（libtest 的参数不动），没有 `--` 接在末尾。"""
    words = arguments.split()
    index = words.index("--") if "--" in words else len(words)
    return " ".join(words[:index] + ["--features", feature] + words[index:])


def gpu_route_rows(key_context, rows, components, environment):
    """(这一趟的行, {行号: Components})：多机配置开了 GPU（gpu_route_enabled）时，点名测试走崩溃判段、cargo 参数没开 gpu 的那几行换成补了 gpu 特性的
    （配 GPU 必用 GPU，不降级：不补的话二进制没编进 gpu，测试判段不跑、只录就返回 ok，改坏的那一处没判过），按补了之后的参数重算键
    （闭包照这一组特性收，特性进键）；之后起的 cargo 都照这份参数，真跑测试的那几次照旧交多机配置、带 gpu 特性的收尾补 --test-threads（with_gpu_test_threads、gpu_test_run）。
    原样的六段（Row.fields）不动：用时表与索引照它取名字。换了哪几行记在 key_context.gpu_route_lines（行号, 补的特性）。
    补了之后点名的测试不编的（#[cfg(not(feature = "gpu"))] 的用例，判的就是没编 gpu 的二进制）照原样留着，记在 key_context.gpu_route_unbuilt（行号, 为什么）。
    弄坏开关 gpu-route-feature-unadded：不补。"""
    if break_is(environment, "gpu-route-feature-unadded"):
        return rows, components
    enabled, configuration = gpu_route_enabled(environment)
    if not enabled:
        return rows, components
    key_context.gpu_route_configuration = configuration
    replaced, release_bases = {}, {}
    for row in rows:
        feature = gpu_route_feature_word(row, components[row.line_number], key_context)
        if feature is not None:
            # 补了 gpu 特性就命中重测试目标（文件头「重测试目标只许 release」）：补完特性再照判一次，没带 release 的补 --release
            featured, basis = release_forced_row(row._replace(arguments=with_feature(row.arguments, feature)), key_context.release_registered, environment)
            if basis is not None:
                release_bases[row.line_number] = basis
            replaced[row.line_number] = (featured, feature)
    try:
        key_context.prepare([row for row, _feature in replaced.values()])
    except mutation_units.ClosureRefused as error:
        labels = {f"{TABLE}:{row.line_number} {row.name}": line_number for line_number, (row, _feature) in replaced.items()}
        for _class_name, message, named in error.failures:
            for label in named:
                if labels.get(label) in replaced:
                    del replaced[labels[label]]
                    key_context.gpu_route_unbuilt.append((labels[label], message))
        key_context.prepare([row for row, _feature in replaced.values()])   # 剩下的照旧算；再有算不成的照常抛（不是补特性带出来的）
    if not replaced:
        return rows, components
    already_released = {line_number for line_number, _basis in key_context.release_lines}
    for line_number, (row, feature) in replaced.items():
        components[line_number] = key_context.components(row)
        key_context.gpu_route_lines.append((line_number, feature))
        if line_number in release_bases and line_number not in already_released:
            key_context.release_lines.append((line_number, release_bases[line_number]))
    return [replaced[row.line_number][0] if row.line_number in replaced else row for row in rows], components


class ReachabilityConfigurationError(Exception):
    """static-check 的截断表、豁免表或选路表里有函数在 <根> 的源码里找不到（文件头「点名测试够不着被改坏的函数」）。"""


class ReachabilityNotJudged(Exception):
    """static-check 的选路表、截断表、豁免表里的函数在 <根> 的源码里一个都没有（门禁样本那种小根）：这一趟没有判可达性，原因是异常的文字（文件头「点名测试够不着被改坏的函数」）。"""


def reachability_units_named(workspace, path, name):
    """工作区里 path 那份 .rs 里短名是 name 的 fn 单元；文件不在工作区里交空。"""
    rust_file = workspace.files.get(path)
    return [] if rust_file is None else [unit for unit in rust_file.units if unit.kind == "fn" and unit.short == name]


def reachability_table_names():
    """GPU_ROUTE_UNITS、REACHABILITY_GPU_ROUTE_STOPS、REACHABILITY_CPU_FALLBACK_ROOTS 点名的全部函数：[(文件, 函数名)]。"""
    names = [(file, name) for _package, file, name in GPU_ROUTE_UNITS[0]] + list(REACHABILITY_CPU_FALLBACK_ROOTS[0])
    for source, target in REACHABILITY_GPU_ROUTE_STOPS[0]:
        names += ([source] if source is not None else []) + [target]
    return names


def reachability_names_missing(workspace, environment):
    """reachability_table_names 里在这一份工作区里找不到的函数（「文件::函数名」）。
    弄坏开关 reachability-stops-unverified：一个都不报（不核的样子：函数改了名，截断与豁免悄悄不起作用）。"""
    if break_is(environment, "reachability-stops-unverified"):
        return []
    return [f"{file}::{name}" for file, name in reachability_table_names() if not reachability_units_named(workspace, file, name)]


def reachability_stop_pairs(workspace, environment):
    """REACHABILITY_GPU_ROUTE_STOPS 在这一份工作区里落到的 (来处单元或 None, 单元)，交给 lib_function_units.py 的 rust_closure_of 的 stop_at。
    弄坏开关 reachability-stops-ignored：交空（不截）；reachability-edges-as-units：按边停的也按单元停（测试直接调的 CPU 判器入口也截掉）。"""
    if break_is(environment, "reachability-stops-ignored"):
        return ()
    edges_as_units = break_is(environment, "reachability-edges-as-units")
    pairs = []
    for source, target in REACHABILITY_GPU_ROUTE_STOPS[0]:
        targets = reachability_units_named(workspace, *target)
        if source is None or edges_as_units:
            pairs += [(None, unit) for unit in targets]
        else:
            pairs += [(origin, unit) for origin in reachability_units_named(workspace, *source) for unit in targets]
    return tuple(pairs)


def cpu_configuration_writer(workspace, closure, environment):
    """截过的闭包里测试自己写 CPU 配置的那一个单元（文件头「点名测试够不着被改坏的函数」豁免那一段）：交「文件::限定名」，没有交 None。
    弄坏开关 reachability-cpu-configuration-ignored：一律交 None。"""
    if break_is(environment, "reachability-cpu-configuration-ignored"):
        return None
    lfu = mutation_units.function_units()
    for unit in closure.order:
        target_id = closure.unit_targets.get(id(unit))
        if (unit.language != "rust" or unit.first_token is None or unit.kind == "doctest" or unit.kind in lfu.RUST_STRUCTURE_ONLY_KINDS
                or target_id is None):
            continue
        if not lfu.rust_unit_is_test(workspace, unit, target_id):
            continue
        rust_file = workspace.files[unit.file]
        values = [lfu.rust_string_value(rust_file.texts[index]) for index in range(unit.first_token, unit.last_token) if rust_file.kinds[index] == lfu.LITERAL]
        values = [value for value in values if value is not None]
        empty_configuration = "" in values and any(value.startswith("multi-host") and value.endswith(".env") for value in values)
        if empty_configuration or any(REACHABILITY_CPU_CONFIGURATION_TEXT in value for value in values):
            return f"{unit.file}::{unit.name}"
    return None


def cpu_fallback_root_in(workspace, closure, environment):
    """截过的闭包里有 REACHABILITY_CPU_FALLBACK_ROOTS 的哪一个：交「文件::函数名」，没有交 None。弄坏开关 reachability-fallback-roots-ignored：一律交 None。"""
    if break_is(environment, "reachability-fallback-roots-ignored"):
        return None
    for path, name in REACHABILITY_CPU_FALLBACK_ROOTS[0]:
        if any(id(unit) in closure.members for unit in reachability_units_named(workspace, path, name)):
            return f"{path}::{name}"
    return None


def reachability_cut_answer(request, environment):
    """一份截过的闭包（文件头「点名测试够不着被改坏的函数」）：request = (单元键, 工作区, 测试单元, 目标, 解析过的 cargo 参数, 要问的单元, stop_at)。
    交 (截过的闭包里有的那几个要问的单元, 豁免的 (哪一种, 凭据) 或 None)。cfg 与特性照 lib_mutation_units.py 的 UnitKeys.closure 那一份取（门禁给这一行算键用的同一组）。"""
    keys, workspace, unit, target_id, parsed, probes, stop_at = request
    lfu = mutation_units.function_units()
    cfg = [item for item in lfu.DEFAULT_RUST_CFG if not (parsed.release and item == "debug_assertions")]
    closure = lfu.rust_closure_of(workspace, unit, target_id, cfg, True, keys.features_of(parsed, target_id), parsed.no_default_features, stop_at=stop_at)
    members = {(member.kind, member.file, member.name) for member in closure.order}
    writer = cpu_configuration_writer(workspace, closure, environment)
    fallback = None if writer is not None else cpu_fallback_root_in(workspace, closure, environment)
    exempt = ("cpu-configuration", writer) if writer is not None else (("gpu-tables-refused", fallback) if fallback is not None else None)
    return frozenset(probe for probe in probes if probe in members), exempt


_REACHABILITY_PARALLEL = {}


def _reachability_cut_task(index):
    """算截过的闭包的一格（并行时在 fork 出来的子进程里跑，单元与工作区是父进程那一份的拷贝）：交 (下标, 有的那几个要问的单元, 豁免, 算不成的那一句或 None, 回溯或 None)。
    lib_function_units.py 的 UsageError（入口在这一组 cfg 下不编）交那一句，那一行记没判；别的异常（算法自己的错）交回溯，父进程照抛。异常对象不交回父进程（不一定 pickle 得动）。"""
    try:
        present, exempt = reachability_cut_answer(_REACHABILITY_PARALLEL["requests"][index], _REACHABILITY_PARALLEL["environment"])
        return index, present, exempt, None, None
    except Exception as error:
        known = isinstance(error, mutation_units.function_units().UsageError)
        return index, frozenset(), None, f"{type(error).__qualname__}：{error}", None if known else traceback.format_exc()


def reachability_cut_answers(requests, environment):
    """每份 request 的 (有的那几个要问的单元, 豁免, 算不成的那一句或 None)，次序同 requests。不到 lib_mutation_units.py 的 PARALLEL_MINIMUM 份
    或 key_workers 只给 1 个时就地算，别的开 fork 出来的子进程并行算（同 lib_mutation_units.py 的 precompute_closures）。收回的份数对不上抛 RuntimeError。"""
    workers = key_workers(environment)
    _REACHABILITY_PARALLEL.clear()
    _REACHABILITY_PARALLEL.update(requests=requests, environment=environment)
    try:
        if workers <= 1 or len(requests) < mutation_units.PARALLEL_MINIMUM:
            results = [_reachability_cut_task(index) for index in range(len(requests))]
        else:
            with multiprocessing.get_context("fork").Pool(processes=min(workers, len(requests))) as pool:
                results = list(pool.imap_unordered(_reachability_cut_task, range(len(requests)), chunksize=2))
    finally:
        _REACHABILITY_PARALLEL.clear()
    unexpected = [detail for _index, _present, _exempt, _failure, detail in results if detail]
    if unexpected:
        raise RuntimeError(f"算截过的闭包时出了算法自己的错（{len(unexpected)} 处）：\n" + "\n".join(unexpected))
    answers = {index: (present, exempt, failure) for index, present, exempt, failure, _detail in results}
    if sorted(answers) != list(range(len(requests))):
        raise RuntimeError(f"截过的闭包派出 {len(requests)} 份、收回 {len(answers)} 份")
    return [answers[index] for index in range(len(requests))]


def reachability_precompute(plans, arguments_field, environment):
    """plans 里每一份照 plan[arguments_field] 的 cargo 参数先把点名测试的闭包算好（按单元键分组，同 KeyContext.prepare，进程数见 key_workers）。
    算不成的（ClosureRefused）这里不报：别的照算完，调用方逐行再问 reachability_closure 时照原话记没判。"""
    groups = collections.OrderedDict()
    for plan in plans:
        _keys, requests = groups.setdefault(id(plan["keys"]), (plan["keys"], []))
        requests.append((plan["test"].unit, plan["test"].target, mutation_units.parse_cargo_arguments(plan[arguments_field])))
    for keys, requests in groups.values():
        with contextlib.suppress(mutation_units.ClosureRefused):
            keys.precompute_closures(requests, key_workers(environment))


def reachability_closure(plan, arguments_text):
    """(点名测试照这组 cargo 参数的 ClosureInfo, None) 或 (None, 算不成那一句)。"""
    try:
        return plan["keys"].closure(plan["test"].unit, plan["test"].target, mutation_units.parse_cargo_arguments(arguments_text)), None
    except mutation_units.function_units().UsageError as error:
        return None, f"点名测试 {plan['test'].full_name} 的闭包算不成：{error}"


def source_text_reader(data_units, path, environment):
    """点名测试的闭包里罩住被改坏的文件 path 的那个数据单元（文件头「点名测试够不着被改坏的函数」closure 那一条）：data-file 的路径就是 path，
    或 data-directory 的目录在 path 之上；交「data-file <路径>」这一类，没有交 None。data_units 是闭包里的 (种类, 路径) 集合。
    弄坏开关 reachability-source-text-ignored：一律交 None，只拿函数单元比，读源码文本的测试报够不着。"""
    if break_is(environment, "reachability-source-text-ignored"):
        return None
    if ("data-file", path) in data_units:
        return f"data-file {path}"
    for kind, directory in sorted(data_units):
        if kind == "data-directory" and path.startswith(directory.rstrip("/") + "/"):
            return f"data-directory {directory}"
    return None


def reachability_refused_reason(plan, where, probes):
    """闭包拒算的行判不了时的原因句：where 是「已收部分」（全闭包）或「截过的闭包已收部分」，probes 是被改坏的单元（种类、文件、限定名）。
    拒算的闭包不全，没收进来的那一段可能调得到被改坏的单元，所以没落在已收部分里的不报够不着，只报判不了。"""
    mutated = "、".join(sorted(f"{file}::{name}" for _kind, file, name in probes))
    return (f"点名测试 {plan['test'].full_name} 闭包拒算、{where}里没有被改坏的单元（{mutated}）：拒算原因 {plan['refusal']}；"
            f"拒算的闭包不全，够不够得着判不了")


def reachability_of_rows(root, rows, environment):
    """static-check 的可达性判法（文件头「点名测试够不着被改坏的函数」）：→ (够不着的行, 豁免的行, 没判的行, 截过闭包的行数, 多机配置, 开没开 GPU)。
      够不着的行 [(行号, 变异名, 哪一路 gpu-route 或 closure, 被改坏的单元, 点名测试, 门禁给这一行用的 cargo 参数)]；豁免的行 [(行号, 变异名, 哪一种, 凭据)]；
      没判的行 [(行号, 变异名, 为什么)]；三样都按行号排。截断表、豁免表或选路表里有函数在 <根> 的源码里找不到抛 ReachabilityConfigurationError；
      这些函数在 <根> 的源码里一个都没有（门禁样本那种小根）抛 ReachabilityNotJudged，一行都不判；
      多机配置在而读不了抛 admission.py 的 InputManifestError。弄坏开关 reachability-closure-unchecked：不判「闭包里根本没有」那一路。
      闭包拒算（ClosureInfo.key 是 None）的行照已收的成员同一套判：被改坏的单元或读到的源码文本落在已收部分里算够得着；不在的报没判（reachability_refused_reason），
      不报够不着（全闭包与截过的闭包两处一样）。弄坏开关 reachability-refused-unjudged：拒算的一律报没判。"""
    lfu = mutation_units.function_units()
    keys_by_directory = {"": mutation_units.UnitKeys(root)}
    root_keys = keys_by_directory[""]
    table_names = reachability_table_names()
    if not any(reachability_units_named(root_keys.workspace, file, name) for file, name in table_names):
        raise ReachabilityNotJudged(f"选路表、截断表、豁免表点名的 {len(table_names)} 处函数在 {root} 的源码里一个都没有（{table_names[0][0]}::{table_names[0][1]} 等），没有可判的路")
    missing = reachability_names_missing(root_keys.workspace, environment)
    if missing:
        raise ReachabilityConfigurationError("、".join(missing))
    enabled, configuration = gpu_route_enabled(environment)
    unreachable, exempt, unjudged, plans = [], [], [], []
    for row in rows:
        why = anchor_problem(root, row)
        keys, refusal = (None, f"锚点失效：{why}") if why is not None else row_unit_keys(root, row, environment, keys_by_directory)
        mutated = test = view = None
        if refusal is None:
            mutated, refusal = keys.mutated_units(row.path, row.old)
        if refusal is None:
            view = row_view(keys, row)
            test, refusal = keys.resolve_test(row.arguments, row.expected, view=view)
        if refusal is not None:
            unjudged.append((row.line_number, row.name, refusal))
            continue
        plans.append(dict(row=row, keys=keys, mutated=mutated, test=test, view=view, arguments=row.arguments))
    reachability_precompute(plans, "arguments", environment)
    judged = []
    for plan in plans:
        closure, why = reachability_closure(plan, plan["arguments"])
        if closure is None:
            unjudged.append((plan["row"].line_number, plan["row"].name, why))
            continue
        plan["closure"] = closure
        word = gpu_route_feature_word_of(plan["arguments"], closure) if enabled else None
        if word is not None:
            plan["gpu_arguments"] = with_feature(plan["arguments"], word)
        judged.append(plan)
    # 开 GPU 时门禁给这几行补 gpu 特性（gpu_route_rows），闭包照补了之后的参数算；补了就不编的（cfg(not(feature = "gpu")) 的用例）门禁照原样跑，
    # 判段不跑、CPU 判器同样不跑，照原样参数的闭包截
    reachability_precompute([plan for plan in judged if "gpu_arguments" in plan], "gpu_arguments", environment)
    stop_pairs, cut_requests, cut_plans, member_sets, data_sets = {}, collections.OrderedDict(), [], {}, {}
    for plan in judged:
        row = plan["row"]
        if "gpu_arguments" in plan:
            closure, _why = reachability_closure(plan, plan["gpu_arguments"])
            if closure is not None:
                plan["closure"], plan["arguments"] = closure, plan["gpu_arguments"]
        closure = plan["closure"]
        if closure.key is None:   # 闭包拒算：键算不出，但已收的成员照用（文件头「点名测试够不着被改坏的函数」拒算那一条）
            plan["refusal"] = closure.refusals[0] if closure.refusals else "没写原因"
            if break_is(environment, "reachability-refused-unjudged"):
                unjudged.append((row.line_number, row.name, f"点名测试 {plan['test'].full_name} 的闭包拒算（{plan['refusal']}），够不够得着判不了"))
                continue
        if id(closure) not in member_sets:   # 几行点名同一条测试共用一份闭包：成员集合只摊开一次
            member_sets[id(closure)] = frozenset((member.kind, member.file, member.name) for member in closure.members)
            data_sets[id(closure)] = frozenset((kind, name) for kind, _file, name in member_sets[id(closure)] if kind in ("data-file", "data-directory"))
        probes = frozenset((reference.kind, reference.file, reference.name) for reference in plan["mutated"])
        present = probes & member_sets[id(closure)]
        if source_text_reader(data_sets[id(closure)], row.path, environment) is not None:
            continue   # 点名测试读被改坏那份文件的源码文本（include_str! 或运行期读文件、读目录）：读到的文本就看得见这一处改动，够得着，也不看 GPU 那一路
        if not present and "refusal" in plan:   # 拒算的闭包不全：已收部分里没有，说不准够不够得着，不报够不着
            unjudged.append((row.line_number, row.name, reachability_refused_reason(plan, "已收部分", probes)))
            continue
        if not present and not break_is(environment, "reachability-closure-unchecked"):
            unreachable.append((row.line_number, row.name, "closure", "、".join(sorted(f"{file}::{name}" for _kind, file, name in probes)),
                                plan["test"].full_name, plan["arguments"]))
            continue
        if not enabled or plan["keys"] is not root_keys or gpu_route_package(closure) is None:
            continue   # 没开 GPU、或判段不经选路：闭包里有它就算够得着
        workspace = root_keys.formal_workspace if plan["view"] == lfu.FORMAL_VERIFICATION else root_keys.workspace
        if id(workspace) not in stop_pairs:
            stop_pairs[id(workspace)] = reachability_stop_pairs(workspace, environment)
        parsed = mutation_units.parse_cargo_arguments(plan["arguments"])
        request_key = (id(workspace),) + root_keys.cache_key(plan["test"].unit, plan["test"].target, parsed)
        if request_key not in cut_requests:
            cut_requests[request_key] = [workspace, plan["test"].unit, plan["test"].target, parsed, set()]
        cut_requests[request_key][4] |= present or probes
        cut_plans.append((plan, present or probes, request_key))
    request_keys = list(cut_requests)
    answers = dict(zip(request_keys, reachability_cut_answers(
        [(root_keys, workspace, unit, target_id, parsed, frozenset(probes), stop_pairs[id(workspace)])
         for workspace, unit, target_id, parsed, probes in (cut_requests[key] for key in request_keys)], environment)))
    for plan, probes, request_key in cut_plans:
        row = plan["row"]
        present, exemption, failure = answers[request_key]
        if failure is not None:
            unjudged.append((row.line_number, row.name, f"点名测试 {plan['test'].full_name} 截过的闭包算不成：{failure}"))
        elif exemption is not None:
            # 认得出豁免的行不判 gpu-route（够不够得着都打 REACHABILITY_EXEMPT：这一行的判段不走或不只走 GPU 路，截过的闭包不代表它）
            exempt.append((row.line_number, row.name) + exemption)
        elif not present & probes and "refusal" in plan:   # 全闭包拒算：截过的闭包已收部分里没有，也说不准（拒算的那一段可能经 CPU 判器连到）
            unjudged.append((row.line_number, row.name, reachability_refused_reason(plan, "截过的闭包已收部分", probes)))
        elif not present & probes:
            unreachable.append((row.line_number, row.name, "gpu-route", "、".join(sorted(f"{file}::{name}" for _kind, file, name in probes)),
                                plan["test"].full_name, plan["arguments"]))
    return sorted(unreachable), sorted(exempt), sorted(unjudged), len(cut_plans), configuration, enabled


def decide_rows(library, rows, components, key_context, environment):
    """每一行 (Decision, 记录)：与 run_stage 同一份 row_decision。"""
    decided = {}
    for row in rows:
        comp = components[row.line_number]
        previous = library.read("index", row_identity(row)) if comp.row_key else None
        decided[row.line_number] = row_decision(library, comp, row, key_context, previous)
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


ITEM_PLAN_PREFIX = "ITEM_PLAN"
ITEM_DONE_PREFIX = "ITEM_DONE"
ITEM_EXIT_BY_VERDICT = {"caught": 0, "notrun": 77}   # 别的档（没红 failure、无效 invalid、基线不绿 baseline、锚点失效 anchor、没判到 unrun）都是 1


def item_exit_code(verdicts):
    """一项（同名的几行合成一项）的退出码（文件头「按项机读行」）：有一行不是抓到也不是本次未跑的 1，没有而有本次未跑的 77，其余 0。"""
    codes = [ITEM_EXIT_BY_VERDICT.get(verdict, 1) for verdict in verdicts]
    return 1 if 1 in codes else 77 if 77 in codes else 0


class ItemProgress:
    """真跑时每项跑完打一行 ITEM_DONE（stderr，文件头「按项机读行」）：同名的几行合成一项，一项的行全有了结局（复用的、现判的、基线不绿的、锚点失效的）就打一次。
    seconds 是这一项各行在工作进程里判一行从头到尾的秒数相加（几项并行时各记各的），不是整趟。弄坏开关 item-done-reused-silent、item-done-seconds-whole-run。"""

    def __init__(self, rows, environment):
        self.environment = environment
        self.started = time.monotonic()
        self.pending = collections.OrderedDict()   # 项名 -> 还没有结局的行号
        self.name_of = {}
        for row in rows:
            self.pending.setdefault(row.name, set()).add(row.line_number)
            self.name_of[row.line_number] = row.name
        self.verdicts = collections.defaultdict(list)
        self.judged = collections.Counter()
        self.reused = collections.Counter()
        self.seconds = collections.defaultdict(float)
        self.printed = set()

    def settle(self, line_number, verdict, reused=False, seconds=0.0):
        """一行有了结局：verdict 是 caught / failure / invalid / baseline / notrun / anchor；reused 是复用的那一行。最后一行到了就打这一项。"""
        name = self.name_of.get(line_number)
        if name is None or line_number not in self.pending[name]:
            return
        self.pending[name].discard(line_number)
        self.verdicts[name].append(verdict)
        if reused:
            self.reused[name] += 1
        elif verdict != "anchor":
            self.judged[name] += 1
        self.seconds[name] += seconds
        if not self.pending[name]:
            self.print_done(name)

    def print_done(self, name):
        if name in self.printed:
            return
        self.printed.add(name)
        if break_is(self.environment, "item-done-reused-silent") and self.reused[name] and not self.judged[name]:
            return
        seconds = time.monotonic() - self.started if break_is(self.environment, "item-done-seconds-whole-run") else self.seconds[name]
        note(f"{ITEM_DONE_PREFIX} item={name} seconds={seconds:.3f} judged={self.judged[name]} reused={self.reused[name]} exit={item_exit_code(self.verdicts[name])}")

    def finish(self):
        """收尾：还有行没有结局的项（被停没判到）补一行，这些行按没判到记（exit=1）。跑到的每一项恰好一行。"""
        for name, lines in self.pending.items():
            if name not in self.printed:
                self.verdicts[name] += ["unrun"] * len(lines)
                lines.clear()
                self.print_done(name)


def item_plan_lines(names, context, environment):
    """--list-items --with-keys 的 ITEM_PLAN 行（文件头「按项机读行」）：names 是逐行列出的变异名（表的次序），同名的几行合成一项；
    要现判 = 键对不上与拒算，能复用 = 键对得上，锚点失效的行（不在 context 的行里）两样都不算。弄坏开关 item-plan-reusable-zero：reusable 一律 0。"""
    counts = collections.OrderedDict((name, [0, 0]) for name in names)
    for row in context["rows"]:
        decision, _record = context["decided"][row.line_number]
        entry = counts.setdefault(row.name, [0, 0])
        if decision.reuse:
            entry[1] += 0 if break_is(environment, "item-plan-reusable-zero") else 1
        else:
            entry[0] += 1
    return [f"{ITEM_PLAN_PREFIX} item={name} to_judge={to_judge} reusable={reusable}" for name, (to_judge, reusable) in counts.items()]


def run_stage(root, environment=None, judging_repository=None):
    """门禁 crates-mutation-replay 的整道：判定打 stdout（按表的行号排序，与进程数、复用了多少都无关），进度与复用几条打 stderr。回退出码。
    judging_repository 只给自证用：拿判法的临时拷贝改了判法，看每一行都现判。"""
    environment = dict(os.environ if environment is None else environment)
    admission = admission_module()
    table_path = os.path.join(root, TABLE)
    if not os.path.isfile(table_path):
        print_rejection(f"没有 {TABLE}", [], ["建一张六段制表符分隔的表（变异名、文件、原文、替换文、cargo test 参数、必须红的测试名），每一处三方打中之后的改法留一条。"])
        return 1
    try:
        rows, malformed = read_table(table_path, environment)
    except (OSError, UnicodeDecodeError) as error:
        print_rejection(f"读不了 {TABLE}（{type(error).__name__}：{error}）", [], ["转成 UTF-8 文本再跑；读不了的表一条都判不了。"])
        return 1
    # 不成形的行（read_table 判的）在 prechecks 里与别的几样一起报：整张表的不成形行不因选行而放过
    if not rows and not malformed:
        print_rejection(f"{TABLE} 里一条成形的变异都没有", [], ["至少一条：三方打中之后的每一处改法，留一条「改回去它就红」的变异。"])
        return 1
    size_problems = size_setting_problems(environment)
    for headline, howto_line in size_problems:
        print_rejection(headline, [], [howto_line])
    if size_problems:
        return 1
    numeric_settings = {}
    for variable, default in (("GATE_MUTATION_LONG_ROW_SECONDS", DEFAULT_LONG_ROW_SECONDS), ("GATE_MUTATION_STOP_AFTER_ROWS", ""), ("GATE_MUTATION_CARGO_JOBS", "")):
        text = environment.get(variable, "").strip() or default
        if text and not re.fullmatch(r"[1-9][0-9]*", text):
            print_rejection(f"{variable} 写成了「{text}」，不是正整数", [], [f"写成正整数（{variable} 的意思见 scripts/mutation/crates-mutation-rows.py 文件头），或者不设它。"])
            return 1
        numeric_settings[variable] = int(text) if text else None
    selection_path = environment.get("GATE_MUTATION_ROW_SELECTION", "").strip()
    table_rows = rows   # 整张表成形的行：选行时撞键照整张表判（plan_rows 的 table_rows）
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
    selected_rows = list(rows)   # 锚点失效的行也是一项（按项机读行）
    rows = [row for row in rows if row.line_number not in anchor_broken]
    try:
        key_context, rows, components, library = plan_rows(root, rows, environment, judging_repository, table_rows)
    except mutation_units.ClosureRefused as error:
        print_rejection(f"点名的测试在 cargo 参数选的这一组特性下认不出或不编（{len(error.failures)} 处），键算不出来：一条都没跑", error.detail_lines(), CLOSURE_REFUSED_HOWTO)
        return 1
    except release_only_tests_module().RegistryUnreadable as error:
        print_rejection(f"判不了哪几行是重测试目标（{error}），补不了 --release：一条都没跑", [], [RELEASE_REGISTRY_HOWTO])
        return 1
    except (JudgingUnitsRefused, mutation_units.MutationUnitsError, admission.InputManifestError, HeavyThreadDefaultError, OSError) as error:
        print_rejection(f"算不出判法单元指纹、工具链、依赖版本或判定库的位置（{error}），判定库没处对：一条都没跑", [],
                        ["被判的要是 git 工作树（rsync 拷的副本先在副本根 git init；只给自证换的几个变量只在临时 prune-server 底下认）、rustc -vV 与 cargo -V 跑得通、Cargo.lock 读得懂；",
                         f"单跑 python3 scripts/mutation/crates-mutation-rows.py judging-units {root} 与 plan {root} 看它报什么。"])
        return 1
    # 用时表与占盘记录是运行记录（文件头「判定库」末段）：放共享根下，不进判定库、不落到 git common-dir
    try:
        run_records = run_records_directory(root, environment)
        timings_directory = os.path.join(run_records, "timings")
        os.makedirs(timings_directory, exist_ok=True)
    except (admission.InputManifestError, OSError) as error:
        print_rejection(f"用时表与占盘记录没处放（{error}），一条都没跑", [],
                        ["多机配置（仓根 multi-host.env，或 SINGLEFS_MULTI_HOST_CONFIG 指的那一份）写上 NFS_SHARED_ROOTS（两台同一绝对路径的共享根）；不补缺省。"])
        return 2
    # 真跑测试的每一次都要把多机配置交给测试（文件头「真跑测试的格把多机配置交给测试」）：那份文件不在，起任何 cargo 之前就拒（配置错，不当本次未跑）。
    #   弄坏开关 test-runs-configuration-unpassed 下不判
    configuration_path, configuration_present = test_runs_multi_host_configuration(environment)
    if not configuration_present and not break_is(environment, "test-runs-configuration-unpassed"):
        print_rejection(f"多机配置 {configuration_path} 不在：真跑测试的每一次都要把它交给测试（副本不是 git 工作树，测试自己找不到，crates 那侧找不到就报错），一条都没跑", [],
                        ["照仓根 multi-host.env.example 在主工作树根写一份 multi-host.env，或设 SINGLEFS_MULTI_HOST_CONFIG 指到它；不补缺省。"])
        return 2
    skip_baseline = break_is(environment, "nobaseline")
    snapshot = snapshot_identity(snapshot_of(root, copied)) if break_is(environment, "baseline-by-tree-snapshot") else None
    decided = decide_rows(library, rows, components, key_context, environment)
    decisions = {line_number: decision for line_number, (decision, _record) in decided.items()}
    sentinel = plan_reuse_sentinel(library, rows, components, environment)
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
        ok, _problem = baseline_check(library, row, components[row.line_number], key_context, snapshot, environment)
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
    planner = CoveragePlanner(components, library, key_context)
    # 组成清单先进库（判法单元、点名测试的闭包、工具链行、依赖版本行）：这一趟中途停了，下一趟照样报得出变了哪几个单元
    key_context.persist(library, components)
    # stderr 头一行：复用几条、现跑几条、前移几条（逐栏的数在 stdout 的「复用：」那一行）
    note(f"  … 判定库记录：复用抓到 {len(reused)} 条、复用基线 ok {reused_baseline_count} 行；现跑基线 {len(batches)} 批 {len(needs_baseline)} 行、"
         f"现跑变异至多 {len(candidates)} 条（其中拒算 {len(refused_lines)} 条）、锚点失效不跑 {len(anchor_judgements)} 行、补记覆盖 0 份；前移 {moved_count} 条（上一次用时 ≥ {numeric_settings['GATE_MUTATION_LONG_ROW_SECONDS']} 秒"
         f"的先派，按用时从长到短）；判法单元 {key_context.judging[:16]}…（{len(key_context.judging_references)} 个单元），库在 {library.directory}"
         + ("；基线没跑：GATE_MUTATION_BREAK=nobaseline（只给证红用）" if skip_baseline else ""))
    for line in sentinel.plan_lines:
        note(line)
    # 按项机读行（文件头同名一段）：复用的与锚点失效的项开跑时就有了结局，现判的项在 run_pool 里一行行判完时报
    item_progress = ItemProgress(selected_rows, environment)
    for judgement in anchor_judgements:
        item_progress.settle(judgement[0], "anchor")
    for line_number in reused:
        item_progress.settle(line_number, "caught", reused=True)
    # 多机配置开了 GPU、点名测试走崩溃判段的行补了 gpu 特性（文件头「开 GPU 时崩溃判段那几行补 gpu 特性」）：逐行点名
    if key_context.gpu_route_lines:
        note(f"  … 多机配置 {key_context.gpu_route_configuration} 开了 GPU（ENABLE_GPU=1）：{len(key_context.gpu_route_lines)} 行点名的测试走崩溃判段、cargo 参数没开 gpu，"
             "这一趟补上 gpu 特性起（键照补了之后的特性算；真跑测试的那几次交多机配置）："
             + "、".join(f"{TABLE}:{line_number}（--features {feature}）" for line_number, feature in key_context.gpu_route_lines))
    for line_number, why in key_context.gpu_route_unbuilt:   # 补了 gpu 特性点名的测试就不编（判的是没编 gpu 的二进制）：照原样跑
        note(f"  … 多机配置开了 GPU，{TABLE}:{line_number} 点名的测试走崩溃判段，补 gpu 特性之后不编（{why}），照原样跑")
    # 命中重测试目标而没带 release 的行补了 --release（文件头「重测试目标只许 release」）：报行数，点名前 REASONS_SHOWN 行与依据
    if key_context.release_lines:
        shown = key_context.release_lines[:REASONS_SHOWN]
        note(f"  … 重测试目标只许 release：{len(key_context.release_lines)} 行命中而 cargo 参数没带 release，这一趟补上 --release 起（键照补了之后的参数算）："
             + "、".join(f"{TABLE}:{line_number}（{'；'.join(basis)}）" for line_number, basis in shown)
             + (f"，另 {len(key_context.release_lines) - len(shown)} 行没列" if len(key_context.release_lines) > len(shown) else "")
             + "；变异表那几行的第五段补上 --release 之后不再补（static-check 报 RELEASE 的就是它们）")
    # 现判与拒算的原因逐行进 stderr（作废时报出变了哪几个单元，文件头「复用判据」）；多了只列前 REASONS_SHOWN 行，全部的看 plan
    fresh_reasons = [row for row in rows if row.line_number not in reused and row.line_number not in refused_lines]
    for row in fresh_reasons[:REASONS_SHOWN]:
        decision = decisions[row.line_number]
        note(f"  · 现判：{TABLE}:{row.line_number} {row.name}：{COLUMN_TITLES[decision.column]}——{decision.detail}")
    refused_rows = [row for row in rows if row.line_number in refused_lines]
    for row in refused_rows[:REASONS_SHOWN]:
        refused_reason = components[row.line_number].refusal or ""
        if refused_reason.startswith(COLLISION_REFUSAL_PREFIX):   # 撞键（refuse_colliding_row_keys）：「拒算（行键撞键：与第 N 行同键）」
            note(f"  · 拒算（{refused_reason}）：{TABLE}:{row.line_number} {row.name}；照跑、不复用")
        else:
            note(f"  · 拒算（照跑、不复用）：{TABLE}:{row.line_number} {row.name}：{refused_reason}")
    if len(fresh_reasons) > REASONS_SHOWN or len(refused_rows) > REASONS_SHOWN:
        note(f"  · 现判 {len(fresh_reasons)} 行、拒算 {len(refused_rows)} 行，各只列了前 {REASONS_SHOWN} 行；逐行的栏与变了的单元：python3 scripts/mutation/crates-mutation-rows.py plan {root}")
    outcome = dict(mutation=[], mutation_info={}, baseline={}, coverage=[], submitted=0, received=0, stopped=False, left_behind=[], record_problems=[])
    worker_count, cargo_jobs, memory_text = 0, "-", "没有要现跑的行，没开工作进程"
    if candidates:
        tools, tools_text, tools_problem = llvm_tools(root, environment)
        if tools_problem is not None:
            print_rejection(f"记覆盖要的 llvm 工具不对（{tools_problem[0]}），一条变异都没跑：", [],
                            [tools_problem[1], "不退回不记覆盖去跑：没有覆盖记录，基线就只能按点名测试的闭包复用。"])
            return 1
        note(f"  … 记覆盖用的 llvm 工具：{tools_text}")
        # 编译目录所在的盘放得下几个工作进程的两份编译目录（文件头「磁盘余量」）：算在核数、条数两项之后，取最小。
        # 调用方没给 GATE_MUTATION_TARGET_DIR 时编译目录由 run_pool 在这一趟的 shard_root 里建，这里按它将来所在的盘（$TMPDIR）算
        shared_target = default_shared_target_directory(environment)
        disk_probe = shared_target if shared_target is not None else planned_run_build_target_directory()
        # 每份编译目录的涨幅按运行记录目录里近 20 次每格判完量到的占盘峰值算（文件头「磁盘余量」），没有读数的那一种按上限
        target_peaks = measured_target_peaks(run_records, 20)
        # 开几个工作进程按这一条拿到的内存上限 ÷ 那个编译并行度下每个工作进程的实测峰值收（文件头「内存上限」）：撞内存上限由服务放大重排、从头跑，开满线程数只会一次次撞顶。
        #   排在另外三项之后（天花板含磁盘那一项）；编译并行度与下面设 CARGO_BUILD_JOBS 同用 cargo_jobs_for
        processor_count = os.cpu_count() or 4
        memory_results = []

        def memory_bound(ceiling):
            memory_results.append(memory_bound_for(environment, ceiling, processor_count, run_records))
            return memory_results[-1]
        try:
            worker_count, worker_count_head_line = worker_count_for(len(candidates), environment,
                                                                    disk=lambda ceiling: disk_bound_for(disk_probe, environment, ceiling, target_peaks),
                                                                    memory=memory_bound)
        except HeavyThreadDefaultError as error:
            print_rejection(f"取不到每台默认重活线程数（{error}），定不了开几个工作进程，一条变异都没跑：", [],
                            [error.howto, "或给 GATE_MUTATION_WORKERS=<正整数> 指定工作进程数；不退回按核数算。"])
            return 2
        note(worker_count_head_line)
        # 撞内存上限（文件头「内存上限」的「撞内存上限」）：内存那一项是 0、一个工作进程都放不下，按统一任务服务错误类型表里 memory-cap-hit 的退出码 250 退，
        #   服务据此加额度重排；不起 1 个硬跑。250 写在这里、不做模块级常量：模块级的赋值进判法单元的闭包，多一个整库的行键就全变
        if memory_results and memory_results[-1][0] == 0 and not break_is(environment, "memory-cap-short-run-anyway"):
            print_rejection(f"内存上限装不下一个工作进程的峰值（{memory_results[-1][1]}），按撞内存上限退出（退 250，统一任务服务错误类型表里 memory-cap-hit 那一行），"
                            "一条变异都没跑：", [],
                            ["经 task-server/task submit 交的，服务照「重试」加额度重排，不用改命令；手跑的给更大的内存上限再跑。",
                             "不开 1 个工作进程硬跑：硬跑会在中途被杀，判过的格白判（任务 7498）。"])
            return 250
        memory_text = (memory_results[-1][1] if memory_results[-1][0] is not None
                       else f"没有内存上限这一项（{memory_results[-1][1]}），照线程数")
        # 每个 cargo 的编译并行度（文件头「编译并行度」）：进程数 × 它 ≈ 核数，免得 N 个 cargo 各自按核数开 rustc、互相抢；与内存那一项试进程数时同一个函数
        cargo_jobs = str(cargo_jobs_for(environment, worker_count, processor_count))
        environment["CARGO_BUILD_JOBS"] = cargo_jobs
        # 起 cargo 时关调试信息（文件头「调试信息」）：只设在起 cargo 的这一份环境里；取值另照 key_toolchain_lines 进了行键
        debug_overrides = cargo_debug_overrides(environment)
        environment.update(debug_overrides)
        note("  … 起 cargo 时的构建档：" + ("、".join(f"{name}={value}" for name, value in sorted(debug_overrides.items())) + "（关调试信息，取值进行键）"
                                       if debug_overrides else "照调用方的环境与 Cargo.toml（弄坏开关 debuginfo-kept：调试信息开着）"))
        note("  … 起 cargo 时的崩溃判定：变异版判出的结论由引擎经 prune-server 写进那一份判定库（文件头「变异复跑的崩溃判定」），不另设库的变量")
        # 这一趟自己建的由 run_pool 建好之后打那一行
        if shared_target is not None:
            for line in target_reclaim_lines(shared_target, worker_count, reclaim_extra_target_directories(shared_target, worker_count, environment), owned=False):
                note(line)
        context = dict(root=root, copied=copied, environment=environment, judging=key_context.judging, llvm_tools=tools,
                       library_directory=library.directory, run_records_directory=run_records, timings_directory=timings_directory,
                       row_records={row.line_number: light_components(components[row.line_number]) for row in rows},
                       shared_target_directory=shared_target, worker_count=worker_count, stage_process=os.getpid(),
                       worker_peaks_recorded=worker_peaks_recorded_for(root, environment),
                       target_directory_cap_bytes=target_directory_cap_bytes(environment))
        RUN_BUILD_REMOVAL_FAILURES.clear()   # 这一趟 run_pool 收尾没删成的编译目录（同一进程里跑几趟时不带上一趟的）
        outcome = run_pool_reporting_dead_worker(context, worker_count, batches, ordered_candidates, baseline_ok_lines,
                                                 numeric_settings["GATE_MUTATION_STOP_AFTER_ROWS"], planner, item_progress)
        if outcome is None:
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
        if outcome.get("prune_server_problem"):
            print_rejection(f"变异复跑要各起一份临时 prune-server，服务的二进制取不到（{outcome['prune_server_problem']}），一条都没跑", [],
                            [f"设 {PRUNE_SERVER_BINARY_VARIABLE}=<编好的 prune-server 的绝对路径> 再跑；或修好 prune-server/ 的编译"
                             "（cargo build --release --offline --manifest-path prune-server/Cargo.toml --bin prune-server，经 task-server/task submit 交）。"])
            return 1
    item_progress.finish()
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
    judged_lines = {judgement[0] for judgement in outcome["mutation"] if judgement[1] != "anchor"} | set(reused)
    for row in rows:
        if row.line_number in judged_lines and components[row.line_number].row_key:
            library.write("index", row_identity(row), index_entry(components[row.line_number], row))
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
                        ["这是判定库记录与派活自己的完整性闸红了，不是变异的问题；再跑一遍，还缺就去查 run_stage 与 run_pool 那一段。"])
        return 1
    fresh_columns = collections.Counter(decisions[row.line_number].column for row in rows if row.line_number not in reused and row.line_number not in refused_lines)
    coverage_failures = [result for result in outcome["coverage"] if result["problem"] is not None]
    for result in coverage_failures:
        note(f"  ! 覆盖没记成（组首行 {TABLE}:{result['line']}；下一趟这一条测试的基线只按它的闭包复用）：{result['problem']}")
    coverage_recorded = sum(1 for result in outcome["coverage"] if result["coverage"] and not result.get("existing"))
    coverage_existing = sum(1 for result in outcome["coverage"] if result.get("existing"))
    note(f"  … 覆盖：这一趟记了 {coverage_recorded} 份、已有沿用 {coverage_existing} 份、没记成 {len(coverage_failures)} 份")
    for line_number in reused:
        sentinel.observe(sentinel_item(rows_by_line[line_number]), reuse_checks.REUSED, "抓到")
    observe_fresh_outcomes(sentinel, outcome["mutation"], rows_by_line, environment)
    sentinel_problems = sentinel.finish(printer=note)
    exit_code = report(rows, judgements, outcome, selection_path, skip_baseline, dict(
        worker_count=worker_count, cargo_jobs=cargo_jobs, memory_text=memory_text, reused=len(reused), reused_baseline=reused_baseline_count,
        fresh_columns=fresh_columns, fresh_rows=len(candidates) - len(refused_lines), refused_rows=len(refused_lines),
        collision_rows=[(row.line_number, components[row.line_number].refusal) for row in rows
                        if (components[row.line_number].refusal or "").startswith(COLLISION_REFUSAL_PREFIX)],
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


def observe_fresh_outcomes(sentinel, mutation_outcomes, rows_by_line, environment):
    """现判那几行的结局交复用哨兵。派活之后才锚点失效的那一行没跑（judge_row），不交；强制现判那一条落在不算数的结局上
    （VERDICTS_UNCOUNTED：本次未跑）也不交，哨兵照「这一趟没报结局」打一行不判（lib_reuse_checks.ReuseSentinel.finish）——
    没跑成的那一次不是现判结论，拿它去比库里的「抓到」会判成不稳。别的行不算数的照交：哨兵被派去现判就已是复用失灵，哨兵看的是状态、不看结论。
    弄坏开关 uncounted-forced-observed 下强制现判那一条不算数的也照交。"""
    for line_number, verdict, _text, _removal in mutation_outcomes:
        if verdict == "anchor":
            continue
        item = sentinel_item(rows_by_line[line_number])
        forced = sentinel.forced(item)
        if forced and verdict in VERDICTS_UNCOUNTED and not break_is(environment, "uncounted-forced-observed"):
            continue
        sentinel.observe(item, reuse_checks.FORCED if forced else reuse_checks.JUDGED, "抓到" if verdict == "caught" else verdict)


def plan_reuse_sentinel(library, rows, components, environment):
    """复用哨兵（文件头「复用三问与复用哨兵」）：候选是此刻行键在 records/ 里有「抓到」记录的行，直接读记录（不经 row_decision）。"""
    sentinel = reuse_checks.ReuseSentinel(SCRIPT_KEY, "crates 变异表", environment=environment)
    candidates = {}
    for row in rows:
        key = components[row.line_number].row_key
        record = library.read("records", key) if key else None
        if record is not None and record.get("row_key") == key and record.get("verdict") == REUSABLE_VERDICT["mutation"]:
            candidates[sentinel_item(row)] = "抓到"
    # 抽中的几条先攒着，等 stderr 头一行（「… 判定库记录：」）打完再打（别的定义与自证照头一行读复用几条）
    sentinel.plan_lines = []
    sentinel.plan(candidates, printer=sentinel.plan_lines.append)
    return sentinel


class CoveragePlanner:
    """这一趟要记哪几份覆盖（文件头「判定库」覆盖记录）：一行现判成抓到、它点名的测试此刻还没有一份执行过的单元都没变的覆盖记录，就派一格（同一份覆盖号只派一次）。"""

    def __init__(self, components, library, key_context):
        """执行过的单元照每一行自己的单元键核（key_context.keys_for，文件头「--manifest-path」）。"""
        self.components, self.library, self.key_context = components, library, key_context
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
            if not executed_set_is_current(self.key_context.keys_for(row)[0], self.library.read_json("coverage", components.coverage_id)):
                picked.append(row)
        return picked

    def job_after(self, row, judgement, info):
        if judgement[1] != "caught":
            return None
        components = self.components.get(row.line_number)
        if components is None or not components.coverage_id or components.coverage_id in self.scheduled or not components.test_full_name:
            return None
        coverage = self.library.read_json("coverage", components.coverage_id)
        if executed_set_is_current(self.key_context.keys_for(row)[0], coverage):
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



def report(rows, judgements, outcome, selection_path, skip_baseline, counts):
    """按变异表的行号排序打印判定、计数、基线与临时目录，判红的逐栏列出；回退出码。"""
    judgements.sort(key=lambda judgement: judgement[0])
    by_verdict = collections.defaultdict(list)
    for _line_number, verdict, text, _removal in judgements:
        by_verdict[verdict].append(text)
    caught, failures, invalid, baseline_bad = by_verdict["caught"], by_verdict["failure"], by_verdict["invalid"], by_verdict["baseline"]
    not_run = by_verdict["notrun"]
    anchor_broken = by_verdict["anchor"]   # 锚点失效（文件头「锚点失效」）：rows 里没有这几行，「共」那个数把它们加回来
    for text in caught:
        say(text)
    # 各栏各数各的；「计数：」这一行的格式别的定义与规则照栏读（mutation-triage），基线另起一行报
    say(f"  计数：抓到 {len(caught)} 条、没红 {len(failures)} 条、无效 {len(invalid)} 条、锚点失效 {len(anchor_broken)} 行"
        f"（共 {len({row.line_number for row in rows} | {judgement[0] for judgement in judgements if judgement[1] == 'anchor'})} 条）")
    if skip_baseline:
        say("  基线：没跑（GATE_MUTATION_BREAK=nobaseline，只给证红用）")
    else:
        say(f"  基线：没改坏的副本上点名的测试恰好对上一个用例、结果是 ok 的 {len(rows) - len(baseline_bad)} 行，基线不绿 {len(baseline_bad)} 行"
            f"（按 cargo 参数分 {len(baseline_batches(rows))} 批；共 {len(rows)} 行）")
    # 复用与覆盖各另起一行（「计数：」那一行的格式不动）；判定本身与进程数无关，这两行随记录变（比对两趟的判定时去掉，selftest_verdict_view）
    counts["not_run_rows"] = len(not_run)
    reuse_text = reuse_summary_text(counts)
    # 成功句点名判定库在哪（文件头「判定库」）：接在「；这一趟现判 …」之后，比对两趟的判定时连它一起去掉（selftest_verdict_view）
    library_text = f"；判定库在 {counts['records_directory']}"
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
    left_behind = outcome["left_behind"]
    removal_state = "每条结束时不论退出码都整个删掉了" if not left_behind else f"跑完有 {len(left_behind)} 个没删掉"
    say(f"  临时目录：{len(removals)} 条变异各一个自己的 TMPDIR，{removal_state}；删前合计 {sum(removal[0] for removal in removals)} 个文件、"
        f"占盘 {sum(removal[1] for removal in removals)} 字节（表观 {sum(removal[2] for removal in removals)} 字节）")
    sections = [
        (anchor_broken, f"有几行锚点失效（锚点失效 {len(anchor_broken)} 行；原文在「文件」那一段指的源码里不是恰好命中一次，或文件不在、读不了：这几行没跑、不入库、不复用，别的行照判）：",
         ["把那一行的原文改到今天源码里逐字存在、只出现一次的那一段（原文里的换行写成 \\n），文件挪了就改「文件」那一段；",
          "改完单跑那几行：bash .claude/gate.d/crates-mutation-replay.sh --item <行号或变异名>（.claude/rules/mutation-sampling.md 第七类）。"]),
        (baseline_bad, f"有几行基线不绿（基线不绿 {len(baseline_bad)} 行；点名的测试在没改坏的副本上就不是恰好一个 ok，改坏之后红了也证明不了什么，这几行的变异没跑）：",
         ["先看点名的测试读的文件在不在拷贝范围里（.claude/gate.d/stage-inputs.tsv 本阶段那一行；bash .claude/gate.d/code-source-discipline.sh 静态判这一样），",
          "不在就登记进那一行；再在真仓上单跑它（照那一行的 cargo test 参数），真仓上也红就是测试本来就坏，先修测试；对上多个用例的把点名写全到模块路径。"]),
        (invalid, f"有变异无效（无效 {len(invalid)} 条；替换文写进源码之后编不过，那条行为今天零变异覆盖）：",
         ["先看尾巴分来源，不是去补用例。有 process didn't exit successfully 与信号的，是测试进程被杀，先查是谁杀的；点名的名字里带 $ 这类字符的，改那一行的测试名；其余是替换文编不过——",
          "先看反斜杠：表里只有 \\n 会被还原成换行，\\& \\\" 这类原样写进源码就编不过；",
          "别的编译错就在副本里把替换后的那一行 cargo check 一遍，改成编得过、而且真会改行为的写法（.claude/rules/mutation-sampling.md 第八类）。"]),
        (failures, f"有变异没红（没红 {len(failures)} 条）：",
         ["没红 = 那处改法没有任何测试守着：先造一条会红的用例再改代码（show-me-test.md）；「没跑到」多半是 cargo test 参数选错了范围。"]),
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
    red = bool(invalid or failures or baseline_bad or left_behind or not_run_record_problem or anchor_broken)
    note(f"  … 这一轮 {counts['worker_count']} 个工作进程，每个 cargo 编译并行度 {counts['cargo_jobs']}，不设上限与限时（内存与卡住归统一任务服务）；"
         f"按内存：{counts['memory_text']}；"
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
            say(f"  · crates 变异表复跑：{len(rows)} 条全是本次未跑（见上），一条都没判到被改坏的那一处：这一道记本次未跑，不是通过；这一趟{reuse_text}{library_text}")
            return 77
        say(f"  ✓ crates 变异表复跑：{len(rows)} 条里抓到 {len(caught)} 条、本次未跑 {len(not_run)} 条（本次未跑的逐条见上，不算抓到；这一趟不是整张表都红在点名的测试上）"
            f"；没红、无效 0 条，锚点失效 0 行，每条一个自己的 TMPDIR、结束时都删掉了；这一趟{reuse_text}{library_text}")
        say(f"  {counts['impact']}")
        return 0
    # ⚠️ 进程数不写进 stdout：写进去就让「GATE_MUTATION_WORKERS=1 与 =16 的 stdout 逐字相同」当场为假；复用几条接在成功句末尾「；这一趟现判」之后
    say(f"  ✓ crates 变异表复跑：{len(rows)} 条变异各自红在点名的测试上（原文都恰好命中一次、锚点失效 0 行；点名的测试在没改坏的副本上都恰好一个 ok；"
        f"每条一个自己的 TMPDIR、结束时都删掉了；工作进程动态领活——谁先跑完谁再领下一条，不按条数预先切片；输出按表的行号排序、判定与进程数无关）"
        f"；这一趟{reuse_text}{library_text}")
    # 牵连面（文件头「牵连面」）：成功句之后另起一行，只报数
    say(f"  {counts['impact']}")
    return 0


def reuse_summary_text(counts):
    """「现判 N 条、复用 M 条、拒算 R 条、本次未跑 K 条（现判的按栏：…）」：成功句末尾与「复用：」那一行用同一句。
    拒算的行照跑、不复用，不算进现判；本次未跑是判出来的一档（点名的测试打了 NOT_RUN_NO_GPU_ADAPTER），算在现判或拒算里。
    拒算里有撞键的（refuse_colliding_row_keys）末尾另接「；其中撞键拒算 N 条：第 <行> 行拒算（行键撞键：与第 <行> 行同键）、…」，没有不接。"""
    columns = "、".join(f"{title} {counts['fresh_columns'].get(name, 0)}" for name, title in REUSE_COLUMNS)
    collisions = counts.get("collision_rows", [])
    collision_text = (f"；其中撞键拒算 {len(collisions)} 条：" + "、".join(f"第 {line_number} 行拒算（{reason}）" for line_number, reason in collisions)
                      if collisions else "")
    return (f"现判 {counts['fresh_rows']} 条、复用 {counts['reused']} 条、拒算 {counts['refused_rows']} 条、本次未跑 {counts.get('not_run_rows', 0)} 条"
            f"（现判的按栏：{columns}）{collision_text}")


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
    try:
        copied = copied_with_path_dependencies(root, copied, os.environ)
    except (ValueError, OSError) as error:
        print(f"ERROR\t拷贝范围跟不全 path 依赖：{error}")
        print(f"     → 怎么办：path 依赖指到被判那棵树之外的，改成树里的目录；Cargo.toml 读不懂的照报错修，再跑；{COPY_REBUILD_HOWTO}")
        return 2
    rows, malformed = [], []
    if os.path.isfile(table_path):
        try:
            rows, malformed = read_table(table_path)
        except (OSError, UnicodeDecodeError) as error:
            print(f"ERROR\t读不了 {table_path}：{error}")
            return 2
    try:
        copied = copied_with_manifest_directories(root, copied, rows, os.environ)
    except (ValueError, OSError) as error:
        print(f"ERROR\t拷贝范围补不上变异行 --manifest-path 指的包目录：{error}")
        print("     → 怎么办：那一行的 --manifest-path 写成仓内一份 Cargo.toml 相对仓根的路径（例 --manifest-path prune-server/Cargo.toml），只写一次；"
              "那个包的 path 依赖指到树之外的改成树里的目录，再跑")
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
    # 替换文里新引的标识符找不到定义（replacement_undefined_identifiers）：只 static-check 判，crates-mutation-replay 派活之前不判（那边套用后编不过记无效）
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
    # 命中重测试目标而第五段没写 release（文件头「重测试目标只许 release」）：只 static-check 判，crates-mutation-replay 派活之前不判（那边照补 --release 起）
    try:
        release_unwritten = release_unwritten_rows(rows, os.environ)
    except release_only_tests_module().RegistryUnreadable as error:
        print(f"ERROR\t判不了哪几行是重测试目标：{error}")
        print(f"     → 怎么办：{RELEASE_REGISTRY_HOWTO}")
        return 2
    for line_number, name, basis in release_unwritten:
        print(f"RELEASE\t{line_number}\t{name}\t{'；'.join(basis)}")
    if release_unwritten:
        print(f"     → 怎么办：{RELEASE_HOWTO}")
    unreachable = []
    try:
        unreachable, exempted, unjudged, cut, configuration, enabled = reachability_of_rows(root, rows, os.environ)
    except ReachabilityNotJudged as error:
        print(f"REACHABILITY_NOT_JUDGED\t{error}")
    except ReachabilityConfigurationError as error:
        print(f"ERROR\t点名测试够不着被改坏的函数那一道的选路表、截断表或豁免表里有函数在 {root} 的源码里找不到：{error}")
        print("     → 怎么办：函数改了名或搬了文件，照今天的源码改 scripts/mutation/crates-mutation-rows.py 的 GPU_ROUTE_UNITS、REACHABILITY_GPU_ROUTE_STOPS、"
              "REACHABILITY_CPU_FALLBACK_ROOTS（判法见它文件头「点名测试够不着被改坏的函数」），再跑")
        return 2
    except (admission.InputManifestError, mutation_units.MutationUnitsError, mutation_units.function_units().FunctionUnitsError,
            HeavyThreadDefaultError, OSError, RuntimeError) as error:
        # RuntimeError 是算截过的闭包时算法自己的错（reachability_cut_answers）：照 ERROR 退 2，不让未捕获的异常退 1 冒充「有问题行」
        print(f"ERROR\t点名测试够不着被改坏的函数那一道判不了：{error}")
        print("     → 怎么办：多机配置读不了的修那份文件（SINGLEFS_MULTI_HOST_CONFIG 指的，没设是本仓的 multi-host.env）；"
              ".claude/heavy-thread-default 读不了的照它的报错修；工作区切不开的照 scripts/gate/lib_function_units.py 的报错修，再跑")
        return 2
    else:
        for line_number, name, route, mutated, test, cargo_arguments in unreachable:
            print(f"UNREACHABLE\t{line_number}\t{name}\t{route}\t{mutated}\t{test}\t{cargo_arguments}")
        if unreachable:
            print(f"     → 怎么办：{REACHABILITY_HOWTO}")
        for line_number, name, kind, evidence in exempted:
            print(f"REACHABILITY_EXEMPT\t{line_number}\t{name}\t{kind}\t{evidence}")
        for line_number, name, why in unjudged:
            print(f"REACHABILITY_UNJUDGED\t{line_number}\t{name}\t{why}")
        print(f"REACHABILITY_CHECKED\t{len(rows) - len(unjudged)}\t{len(unjudged)}\t{len(exempted)}\t{cut}\t{configuration}\t"
              f"{'ENABLE_GPU=1' if enabled else 'GPU 没开'}")
    print(f"CHECKED\t{len(rows)}\t{scanned}\t{len(literals)}")
    return 1 if any(problems.values()) or outside_literals or undefined_rows or release_unwritten or unreachable else 0



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
        print("     → 怎么办：被判的要是 git 工作树（rsync 拷的副本先在副本根 git init）；只给自证换的几个变量只在临时 prune-server 底下认")
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


def plan_context(root, environment, selection_path=""):
    """plan、row-plan、--list-items 共用：读表、算每一行的键与判定（与 run_stage 同一份 plan_rows、decide_rows）；不写库。
    selection_path 给了（row-plan --rows）就只对选中的行算键、判定，别的行放进 unselected_rows、不算键；弄坏开关 rowplanselectionignored 下照整张表算。
    交 dict；不成就打两行（✗ 与 →）交 None。"""
    admission = admission_module()
    try:
        rows, _malformed = read_table(os.path.join(root, TABLE), environment)
    except (OSError, UnicodeDecodeError) as error:
        print(f"  ✗ 读不了 {os.path.join(root, TABLE)}：{error}")
        print("     → 怎么办：先让这张表在、是 UTF-8；读不了就一行都判不了")
        return None
    selected_lines = None
    if selection_path and not break_is(environment, "rowplanselectionignored"):
        selected_lines, selection_problems = read_selection(selection_path, rows)
        if selection_problems:
            print(f"  ✗ row-plan 的选行文件不对：{'；'.join(selection_problems)}")
            print(f"     → 怎么办：一行写一个 {TABLE} 里成形的变异所在的行号（# 起头的行与空行不看）；行号照被判那一份表现数")
            return None
    # 锚点失效的行（文件头「锚点失效」）先剔出去、单列：不算键（run_stage 不跑它们，row-plan 不让双机分片的驱动把它们分给第二台）
    anchor_broken = {} if break_is(environment, "anchor-rows-planned") else anchor_broken_rows(root, rows)
    anchor_rows = [(row, anchor_broken[row.line_number]) for row in rows if row.line_number in anchor_broken]
    rows = [row for row in rows if row.line_number not in anchor_broken]
    unselected_rows = []
    if selected_lines is not None:   # 选行：只对选中的行算键；别的行只给用时（row-plan 的第四列读用时表，不要键）
        unselected_rows = [row for row in rows if row.line_number not in selected_lines]
        rows = [row for row in rows if row.line_number in selected_lines]
    try:
        key_context, rows, components, library = plan_rows(root, rows, environment, table_rows=rows + unselected_rows)
    except mutation_units.ClosureRefused as error:
        print_rejection(f"点名的测试在 cargo 参数选的这一组特性下认不出或不编（{len(error.failures)} 处），键算不出来", error.detail_lines(), CLOSURE_REFUSED_HOWTO)
        return None
    except release_only_tests_module().RegistryUnreadable as error:
        print_rejection(f"判不了哪几行是重测试目标（{error}），键算不出来", [], [RELEASE_REGISTRY_HOWTO])
        return None
    except (JudgingUnitsRefused, mutation_units.MutationUnitsError, admission.InputManifestError, HeavyThreadDefaultError, OSError) as error:
        print(f"  ✗ 算不出判法单元指纹、工具链、依赖版本或判定库的位置：{error}")
        print("     → 怎么办：被判的要是 git 工作树（rsync 拷的副本先在副本根 git init）、rustc -vV 与 cargo -V 跑得通、Cargo.lock 读得懂")
        return None
    return dict(rows=rows, components=components, key_context=key_context, library=library,
                decided=decide_rows(library, rows, components, key_context, environment),
                anchor_rows=anchor_rows, unselected_rows=unselected_rows)


def command_row_plan(arguments):
    """每一条成形的变异一行：行号、行键（拒算写 -）、这一趟会不会复用、上一次现跑的秒数（见文件头 row-plan）；带 --rows 只对选中的行算键。"""
    if len(arguments) not in (1, 3) or (len(arguments) == 3 and (arguments[1] != "--rows" or not arguments[2])):
        print("  ✗ 用法：crates-mutation-rows.py row-plan <根> [--rows <选行文件>]")
        print("     → 怎么办：<根> 是被判那棵树的仓根（git 工作树）；--rows 后面是一份选行文件（一行一个变异表行号）")
        return 2
    environment = dict(os.environ)
    context = plan_context(arguments[0], environment, arguments[2] if len(arguments) == 3 else "")
    if context is None:
        return 2
    try:
        timings_directory = os.path.join(run_records_directory(arguments[0], environment), "timings")
    except admission_module().InputManifestError as error:
        print(f"  ✗ 用时表在哪取不到：{error}")
        print("     → 怎么办：多机配置（仓根 multi-host.env，或 SINGLEFS_MULTI_HOST_CONFIG 指的那一份）写上 NFS_SHARED_ROOTS（文件头「判定库」末段）")
        return 2
    lines = []
    for row in context["rows"]:
        decision, _record = context["decided"][row.line_number]
        timing = read_timing(timings_directory, row_content_hash(row))
        key = context["components"][row.line_number].row_key or "-"
        lines.append((row.line_number, f"{row.line_number}\t{key}\t{1 if decision.reuse else 0}\t{'-' if timing is None else f'{timing:.3f}'}\t-"))
    # 选行之外的行：只给用时（驱动分行取没记过用时的中位数仍取全表），键写 -、不复用，不算键
    for row in context["unselected_rows"]:
        timing = read_timing(timings_directory, row_content_hash(row))
        lines.append((row.line_number, f"{row.line_number}\t-\t0\t{'-' if timing is None else f'{timing:.3f}'}\t-"))
    # 锚点失效的行：没有键、不复用，第五列 anchor-broken（双机分片的驱动不把它们分给第二台）
    lines += [(row.line_number, f"{row.line_number}\t-\t0\t-\tanchor-broken") for row, _why in context["anchor_rows"]]
    for _line_number, text in sorted(lines):
        print(text)
    table_rows = len(context["rows"]) + len(context["unselected_rows"]) + len(context["anchor_rows"])
    selection_note = f"，选行文件点名了 {len(context['rows'])} 行、别的行只给用时" if len(arguments) == 3 and not break_is(environment, "rowplanselectionignored") else ""
    note(f"  · row-plan：算了 {len(context['components'])} 行的键（表里成形的 {table_rows} 行{selection_note}）")
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


def command_list_items(arguments):
    """可点名的变异，一行一条（门禁 --list-items 调的就是这一份），每行末尾一段写这一行的键对不对得上；末行合计键对不上的条数。"""
    with_keys = "--with-keys" in arguments
    arguments = [argument for argument in arguments if argument != "--with-keys"]
    selection_path = ""
    if "--rows" in arguments:
        index = arguments.index("--rows")
        selection_path = arguments[index + 1] if index + 1 < len(arguments) else ""
        arguments = arguments[:index] + arguments[index + 2:]
        if not selection_path or not with_keys:
            print("  ✗ 用法：crates-mutation-rows.py --list-items --with-keys --rows <选行文件> <根>")
            print("     → 怎么办：--rows 只和 --with-keys 一起用，后面是一份选行文件（一行一个变异表行号，门禁把 --item 点名的几行换成它）")
            return 2
    if len(arguments) != 1:
        print("  ✗ 用法：crates-mutation-rows.py --list-items [--with-keys [--rows <选行文件>]] <根>")
        print("     → 怎么办：<根> 是被判那棵树的仓根；--with-keys 另给每项一行 ITEM_PLAN（文件头「按项机读行」），再加 --rows 只列、只算点名那几行")
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
    selected = None
    if selection_path and not break_is(environment, "item-plan-selection-ignored"):
        selected, selection_problems = read_selection(selection_path, table_rows)
        if selection_problems:
            print(f"  ✗ --rows 的选行文件不对：{'；'.join(selection_problems)}")
            print(f"     → 怎么办：一行写一个 {TABLE} 里成形的变异所在的行号（# 起头的行与空行不看）")
            return 2
        items = [(number, fields) for number, fields in items if number in selected]
    else:
        selection_path = ""   # 弄坏开关 item-plan-selection-ignored：点名也算整表
    for number, fields in items:
        anchor_note = f"；锚点失效：{anchor_broken[number]}" if number in anchor_broken else ""
        print(f"{fields[0]}\t第 {number} 行，改 {fields[1] if len(fields) > 1 else ''}{anchor_note}", flush=True)
    status_output = io.StringIO()
    with contextlib.redirect_stdout(status_output):
        context = plan_context(arguments[0], environment, selection_path)
    if context is None:
        print("  · 键对不对得上没算出来：" + " ".join(line.strip() for line in status_output.getvalue().splitlines()))
        if with_keys:
            print("  ✗ --with-keys 要每项的键，上面这一行写着键为什么没算出来")
            print("     → 怎么办：照上一行修了再跑；不退回只列项名（没有键的 ITEM_PLAN 排不了活）。")
            return 1
        return 0
    if with_keys:
        for line in item_plan_lines([fields[0] for _number, fields in items], context, environment):
            print(line, flush=True)
    counter = collections.Counter()
    for row in context["rows"]:
        decision, _record = context["decided"][row.line_number]
        state = "复用" if decision.reuse else "拒算" if decision.column == "refused" else "键对不上"
        counter[state] += 1
    anchor_rows = [(row, why) for row, why in context["anchor_rows"] if selected is None or row.line_number in selected]
    anchor_named = "、".join(f"第 {row.line_number} 行 {row.name}" for row, _why in anchor_rows)
    print(f"  · 键对不上 {counter['键对不上']} 条（这一趟要现判）、键对得上 {counter['复用']} 条（复用）、拒算 {counter['拒算']} 条（照跑）、"
          f"锚点失效 {len(anchor_rows)} 行{'（不跑、整道判红：' + anchor_named + '）' if anchor_named else ''}；"
          f"逐行的栏与变了哪几个单元：python3 scripts/mutation/crates-mutation-rows.py plan {arguments[0]}")
    # 牵连面（文件头「牵连面」）：末了另起一行，只报数
    print(f"  · {impact_summary(context['rows'], context['components'], context['key_context'], environment, len(anchor_rows))}")
    return 0


def command_item(arguments):
    """--item <行号或变异名> [--item …] <根>：交给门禁 crates-mutation-replay.sh --item（同一份选行模式）。"""
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


def command_sample_tools(arguments):
    """sample-tools <目录>：写假 cargo 与两个假 llvm 工具（文件头同名子命令）。"""
    if len(arguments) != 1:
        print("  ✗ 用法：crates-mutation-rows.py sample-tools <目录>")
        print("     → 怎么办：<目录> 是放假工具的目录（门禁样本档在自己的临时目录里建）")
        return 2
    try:
        selftest_write(os.path.join(arguments[0], "cargo"), FAKE_CARGO, executable=True)
        for tool in LLVM_TOOLS:
            selftest_write(os.path.join(arguments[0], tool), FAKE_LLVM_TOOL, executable=True)
    except OSError as error:
        print(f"  ✗ 写不了 {arguments[0]}（{error}）：样本档的假工具放不下")
        print("     → 怎么办：看 ${TMPDIR:-/tmp} 满没满、有没有写权限，修好再跑。")
        return 1
    print(f"  · 假 cargo 与假 llvm 工具（{'、'.join(LLVM_TOOLS)}）写进了 {arguments[0]}")
    return 0


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
# 「// TEST <名字> FILES <相对 crate 根的路径>[,<路径>…] <原文> RUNS …」：原文在列出的每一份文件里都在才 ok（撞键拒算那一格改的是 lib.rs 以外的两份文件）；
# 过滤词照 libtest：`--` 之后带 --exact 时要与全名 tests::<名字> 逐字相同，不带是全名的子串；名字里带 :: 的就是全名（样本里两个模块各有一条同名用例）；
# 「// COMPILE-ERROR-IF <原文>」：源码里（这一行自己除外）有这段原文，test 就照 rustc 编不过的样子打 error[E0425] 与 could not compile、退 101（样本里替换文编不过的变异）；
# 「// TEST <名字> NOTRUN <一句>」照 GPU 对拍用例在没卡的机器上：一律 ok，带 --show-output 时照 libtest 的样子
# 在末尾交出它打的 NOT_RUN_NO_GPU_ADAPTER 那一行；$FAKE_CARGO_EDIT_SOURCE（「<原文>\t<改成>」）设了，test 跑完用例之后把当前目录那份 lib.rs 里恰好一次的原文改掉；
# 每起一次往 $FAKE_CARGO_LOG 追加一行「<参数>\t<lib.rs 的 sha256 前 12 位>」；
# $FAKE_CARGO_ENVIRONMENT_LOG 设了，每起一次 test 再往它追加一行 JSON：收到的 *_THREADS、SINGLEFS_THREAD_CAP、CARGO_BUILD_JOBS、CARGO_TARGET_DIR、
# 构建档覆盖（CARGO_PROFILE_*），外加键 ARGUMENTS（这一次的参数，认得出是不是插桩编译那一趟）；调试信息那几格读它。
# $FAKE_CARGO_SERVICE_LOG 设了，不带 --no-run 的 test 照收到的连哪一处往 prune-server 写一条（键 $FAKE_CARGO_CRASH_VERDICT_PREFIX 接 TMPDIR 的名字，
# 客户端从 $FAKE_CARGO_CLIENT_DIRECTORY 载入），再往它追加一行「<socket><TAB><临时服务的标记><TAB><TMPDIR 的名字>」：变异复跑的崩溃判定那一格读它。
# $FAKE_CARGO_BARRIER_DIR 设了，不带 --no-run 且源码不是 $FAKE_CARGO_BARRIER_PRISTINE（lib.rs 的 sha256 前 12 位）的那一次（改坏了的变异那一趟）在里面按
# CARGO_TARGET_DIR 记一个文件，等到里面有两个（两个工作进程都到了）或 20 秒：造「两个工作进程同时在跑」。
# $FAKE_CARGO_MUTATION_SLEEP 设了，不带 --no-run 且源码不是 $FAKE_CARGO_BARRIER_PRISTINE 的那一次（改坏了的变异那一趟，基线与覆盖不睡）睡这么多秒：
# 造「每一项自己的挂钟」（按项机读行 ITEM_DONE 的 seconds 那一格读它）。
# $FAKE_CARGO_SPAN_LOG 设了，test 每起一次往里记「start<TAB><TMPDIR 的名字><TAB><时刻>」，睡 $FAKE_CARGO_SPAN_SLEEP 秒（没设睡 0.6），退出时记一行 end（O_APPEND，一次写一行）：
# 派活那一段那几格按 TMPDIR 的名字（baseline-<行号>-…、line-<行号>-…）认是哪一格、几格同时在跑。
# $FAKE_CARGO_FORCED_OUTCOME 是 notrun 时每条用例都 ok，带 --show-output 时照没卡的机器交出一行 NOT_RUN_NO_GPU_ADAPTER（不算数的结局那几格造本次未跑）。
# 测试那一趟的 `--` 之后 --exact 出现两次，照 libtest 报 error: Option 'exact' given more than once、退 101。
# --test-threads（=N 或分开写）出现两次同样报 Option 'test-threads' given more than once、退 101；分开写时后面的 N 是它的值，不当过滤词。
# 开头的 --config <值> 跳过。test --no-run（插桩编译、先编一份依赖）：每个 -p 点名的包（不点名是 tiny）在 $CARGO_TARGET_DIR/debug/deps/ 下写一个假的测试可执行文件
# （内容是 JSON：源码副本的根与这个包 src/ 下的 .rs）与它的 dep-info，打一行 compiler-artifact 消息；
# 不带 --no-run 的 test（基线与变异直接跑测试，真 cargo 也先编进编译目录）：$CARGO_TARGET_DIR 设了，同样往里写 debug/deps/ 下每个 -p 点名的包一份 lib<包>-<16 个 0>.rlib，
# $FAKE_CARGO_INCREMENTAL_BYTES 设了再写一份增量产物（编译目录上限、停派、占盘记录那几格量的是这份）；
# 设了 LLVM_PROFILE_FILE 的 test 把这一趟跑到的用例执行的函数写进它指的那份 .profraw（%p 换成进程号、%m 换成 0，内容是 JSON）。
# test --no-run 还照真插桩编译里构建脚本的样子写一份剖析文件：有 LLVM_PROFILE_FILE 写它指的，没有写当前目录的 default_%m_%p.profraw；
# 写了的路径连同当前目录追加进 $FAKE_CARGO_PROFILE_LOG（设了才记）。
# `--` 之前的 --features / -F 里有 gpu 的（自证拿它当编 librocksdb-sys 的那几行；真仓里 验证包常编它），照真 cargo 编 librocksdb-sys 那样要 BINDGEN_EXTRA_CLANG_ARGS：环境里没有、
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
if os.environ.get("FAKE_CARGO_SPAN_LOG"):
    import atexit, time
    span_name = os.path.basename(os.environ.get("TMPDIR", ""))

    def write_span(kind):
        with open(os.environ["FAKE_CARGO_SPAN_LOG"], "a", encoding="utf-8") as span_log:
            span_log.write(f"{kind}\t{span_name}\t{time.time():.6f}\n")
    write_span("start")
    atexit.register(write_span, "end")
    time.sleep(float(os.environ.get("FAKE_CARGO_SPAN_SLEEP") or "0.6"))
source = open("crates/tiny/src/lib.rs", encoding="utf-8").read()
with open(os.environ["FAKE_CARGO_LOG"], "a", encoding="utf-8") as log:
    log.write(" ".join(arguments[1:]) + "\t" + hashlib.sha256(source.encode()).hexdigest()[:12] + "\n")
if os.environ.get("FAKE_CARGO_ENVIRONMENT_LOG"):
    # 设了才记：每起一次 test 往里追加一行 JSON，收到的线程数变量、编译并行度与编译目录
    names = ("RUST_TEST_THREADS", "CARGO_BUILD_JOBS", "CARGO_TARGET_DIR")
    with open(os.environ["FAKE_CARGO_ENVIRONMENT_LOG"], "a", encoding="utf-8") as environment_log:
        recorded = {name: value for name, value in os.environ.items()
                    if name in names or name.endswith("_THREADS") or name == "SINGLEFS_THREAD_CAP" or name.startswith("CARGO_PROFILE_")}
        recorded["ARGUMENTS"] = " ".join(arguments[1:])
        environment_log.write(json.dumps(recorded, sort_keys=True) + "\n")
if os.environ.get("FAKE_CARGO_SERVICE_LOG") and "--no-run" not in arguments:
    # 设了才做：真跑测试的这一次照收到的连哪一处往服务里写一条（演用例里的引擎写崩溃判定：键 $FAKE_CARGO_CRASH_VERDICT_PREFIX 接 TMPDIR 的名字），
    # 写成之后往 $FAKE_CARGO_SERVICE_LOG 追加一行「<SINGLEFS_PRUNE_SERVER_SOCKET><TAB><PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET><TAB><TMPDIR 的名字>」；
    # 客户端从 $FAKE_CARGO_CLIENT_DIRECTORY 载入，连不上照客户端退 75
    sys.path.insert(0, os.environ["FAKE_CARGO_CLIENT_DIRECTORY"])
    import prune_server_client
    run_name = os.path.basename(os.environ.get("TMPDIR", ""))
    prune_server_client.process_client().write_many([((os.environ["FAKE_CARGO_CRASH_VERDICT_PREFIX"] + run_name).encode(), b"red")])
    with open(os.environ["FAKE_CARGO_SERVICE_LOG"], "a", encoding="utf-8") as service_log:
        service_log.write(f"{os.environ.get('SINGLEFS_PRUNE_SERVER_SOCKET', '')}\t{os.environ.get('PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET', '')}\t{run_name}\n")
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
if os.environ.get("FAKE_CARGO_BARRIER_DIR") and "--no-run" not in arguments and hashlib.sha256(source.encode()).hexdigest()[:12] != os.environ.get("FAKE_CARGO_BARRIER_PRISTINE"):
    import time
    barrier = os.environ["FAKE_CARGO_BARRIER_DIR"]
    os.makedirs(barrier, exist_ok=True)
    open(os.path.join(barrier, hashlib.sha256(os.environ["CARGO_TARGET_DIR"].encode()).hexdigest()[:12]), "w").close()
    barrier_deadline = time.monotonic() + 20
    while len(os.listdir(barrier)) < 2 and time.monotonic() < barrier_deadline:
        time.sleep(0.05)
if os.environ.get("FAKE_CARGO_MUTATION_SLEEP") and "--no-run" not in arguments and hashlib.sha256(source.encode()).hexdigest()[:12] != os.environ.get("FAKE_CARGO_BARRIER_PRISTINE"):
    import time
    time.sleep(float(os.environ["FAKE_CARGO_MUTATION_SLEEP"]))
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


if "gpu" in features and not bindgen_include_given():
    print("error: failed to run custom build command for `librocksdb-sys v0.0.0 (crates-mutation-rows selftest)`", file=sys.stderr)
    print("  rocksdb/include/rocksdb/c.h:65:10: fatal error: 'stdbool.h' file not found", file=sys.stderr)
    sys.exit(101)
if "--no-run" not in before and os.environ.get("CARGO_TARGET_DIR"):
    # 真 cargo test 不带 --no-run 也先把要跑的包编进 $CARGO_TARGET_DIR 再跑（基线与变异那一趟不另起 --no-run 先编）：照样往这份编译目录写增量产物与 deps 里的一份产物，
    # 编译目录上限、停派、占盘记录那几格才有东西可量；CARGO_TARGET_DIR 没设（真 cargo 会写当前目录的 target）的调用不写，免得往源码副本里落东西
    run_target = os.environ["CARGO_TARGET_DIR"]
    os.makedirs(os.path.join(run_target, "debug", "deps"), exist_ok=True)
    if os.environ.get("FAKE_CARGO_INCREMENTAL_BYTES"):
        run_session = os.path.join(run_target, "debug", "incremental", "session-" + str(os.getpid()))
        os.makedirs(run_session, exist_ok=True)
        with open(os.path.join(run_session, "dep-graph.bin"), "wb") as handle:
            handle.write(os.urandom(int(os.environ["FAKE_CARGO_INCREMENTAL_BYTES"])))
    for run_package in [before[index + 1] for index, word in enumerate(before) if word == "-p" and index + 1 < len(before)] or ["tiny"]:
        with open(os.path.join(run_target, "debug", "deps", f"lib{run_package}-{'0' * 16}.rlib"), "w", encoding="utf-8") as handle:
            handle.write(run_package)
if "--no-run" in before:
    packages = [before[index + 1] for index, word in enumerate(before) if word == "-p" and index + 1 < len(before)] or ["tiny"]
    deps = os.path.join(os.environ["CARGO_TARGET_DIR"], "debug", "deps")
    os.makedirs(deps, exist_ok=True)
    # 增量产物（rustc 每编一次改坏的副本往 incremental 里加的会话目录）：FAKE_CARGO_INCREMENTAL_BYTES 设了，每次编往里写这么多字节（编译目录上限那一格读；不带 --no-run 的 test 在上面同样写一份）
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
    # 剖析文件只在插桩编译时写（参数或 RUSTFLAGS / CARGO_ENCODED_RUSTFLAGS 里带 -Cinstrument-coverage）：先编一份依赖那一趟不插桩，不写
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
for compile_line in source.splitlines():
    if compile_line.startswith("// COMPILE-ERROR-IF ") and compile_line[len("// COMPILE-ERROR-IF "):] in source.replace(compile_line, ""):
        print("error[E0425]: cannot find value in this scope", file=sys.stderr)
        print("error: could not compile `tiny` (lib test) due to 1 previous error", file=sys.stderr)
        sys.exit(101)
after = arguments[arguments.index("--") + 1:] if "--" in arguments else []
if after.count("--exact") > 1:
    print("error: Option 'exact' given more than once", file=sys.stderr)
    sys.exit(101)
# --test-threads 照 libtest：只能出现一次（--test-threads=N 或分开写 --test-threads N），分开写时 N 是它的值、不是过滤词
if sum(1 for word in after if word == "--test-threads" or word.startswith("--test-threads=")) > 1:
    print("error: Option 'test-threads' given more than once", file=sys.stderr)
    sys.exit(101)
if "--test-threads" in after:
    index = after.index("--test-threads")
    after = after[:index] + after[index + 2:]
filters = [word for word in after if not word.startswith("-")]
exact = "--exact" in after
failed = False
shown, executed = [], set()
definitions = {match.group(1): match.group(0) for match in re.finditer(r"(?m)^\s*(?:pub\s+)?(?:const\s+)?fn\s+(\w+).*$", source)}
for line in source.splitlines():
    words = line.split(" ", 4)
    if len(words) < 5 or words[:2] != ["//", "TEST"]:
        continue
    name, how, what = words[2], words[3], words[4]
    full_name = name if "::" in name else "tests::" + name
    if filters and not any((word == full_name) if exact else (word in full_name) for word in filters):
        continue
    if how == "NOTRUN" or os.environ.get("FAKE_CARGO_FORCED_OUTCOME") == "notrun":
        print(f"test {full_name} ... ok")
        if "--show-output" in after:
            shown.append(f"---- {full_name} stdout ----\nNOT_RUN_NO_GPU_ADAPTER test={name} reason={what if how == 'NOTRUN' else '自证：强令本次未跑'}\n")
        continue
    what, _runs, runs = what.partition(" RUNS ")
    executed |= set(runs.split(",")) if runs else {function for function, definition in definitions.items() if what in definition}
    if how == "FILES":
        paths, _space, text = what.partition(" ")
        ok = all(text in open(os.path.join("crates", "tiny", path), encoding="utf-8").read() for path in paths.split(","))
    else:
        ok = (what in source.replace(line, "")) if how == "NEEDS" else os.path.isfile(os.path.join("crates", "tiny", what))
    if os.environ.get("FAKE_CARGO_FORCED_OUTCOME") in ("ok", "FAILED"):
        ok = os.environ["FAKE_CARGO_FORCED_OUTCOME"] == "ok"   # 记红撤绿那几格：同一把键、不改源码，让这一趟每条用例都 ok 或都 FAILED
    failed = failed or not ok
    print(f"test {full_name} ... {'ok' if ok else 'FAILED'}")
if shown:
    print("\nsuccesses:\n\n" + "\n".join(shown))
if os.environ.get("LLVM_PROFILE_FILE"):
    with open(os.environ["LLVM_PROFILE_FILE"].replace("%p", str(os.getpid())).replace("%m", "0"), "w", encoding="utf-8") as handle:
        json.dump({"executed": sorted(executed)}, handle)
if os.environ.get("FAKE_CARGO_EDIT_SOURCE") and source.count(os.environ["FAKE_CARGO_EDIT_SOURCE"].split("\t")[0]) == 1:
    # 派活之后锚点失效那一格：跑完用例之后把这一份副本里的那段原文改掉，造「开跑之后别的会话改了源码」
    with open("crates/tiny/src/lib.rs", "w", encoding="utf-8") as handle:
        handle.write(source.replace(*os.environ["FAKE_CARGO_EDIT_SOURCE"].split("\t")))
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
# 产品 GPU 包的真显卡对拍开的是 tests-on-a-real-gpu（不是 gpu）：同样当上卡的特性认（GPU_TEST_FEATURES）
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
# 单元键那几格（controls 路第 5 节 crates 变异那一行的 S34、S30、N33、N34、N45，加上被改坏的单元、点名测试的闭包、常量、覆盖复用基线几格）的样本 crate：
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


def selftest_new_library_copy(work, repository, name):
    """repository 整份（连同它自己的 .git）拷到 work/<name>：git common-dir 不同，判定库在临时 prune-server 里的命名空间是新的一份（文件头「判定库」），
    那一份里一条记录都没有，跑一趟就是全部现判（基线、覆盖也现跑）。要「同一份输入再现判一趟」的格用它换一棵树，库没有「不读」这一路。"""
    copy = os.path.join(work, name)
    shutil.copytree(repository, copy, symlinks=True)
    return copy


def selftest_one_forced(row_count):
    """复用哨兵取 row_count - 1 条、剩下那一条强制现判（SINGLEFS_REUSE_SENTINEL_COUNT、SINGLEFS_REUSE_SENTINEL_SEED，lib_reuse_checks.py 文件头「二」）：
    库里 row_count 行都是抓到、键相同的一趟里，只让一行经哨兵的强制现判路径不读库、其余照复用。同键现判、记红盖绿的几格用它。"""
    return {reuse_checks.SENTINEL_COUNT_VARIABLE: str(row_count - 1), reuse_checks.SENTINEL_SEED_VARIABLE: "5"}


def selftest_forced_row_name(run):
    """一趟 selftest.run 的结果里，复用哨兵抽中强制现判的那一行的变异名（stderr 里「强制现判 1 条：<行号> <变异名>」）；没抽出交 None。"""
    match = re.search(r"强制现判 1 条：\d+ (.+)$", run[2], re.M)
    return match.group(1) if match else None


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
        self.log = os.path.join(work, "cargo.log")
        # BINDGEN_EXTRA_CLANG_ARGS 也不带进来：调用方 export 了它，副本里没有 .cargo/ 照样编得过，「拷贝范围含 .cargo/」那一格就红不出来；
        # RUSTFLAGS 一类也不带：假 cargo 不看它，带进来只会让工具链行随调用方变
        self.base_environment = {key: value for key, value in os.environ.items()
                                 if key not in STAGE_VARIABLES and key not in mutation_units.BUILD_VARIABLES and not key.startswith("CARGO_PROFILE_")
                                 and key not in ("BINDGEN_EXTRA_CLANG_ARGS", "FAKE_CARGO_VERSION", "FAKE_LLVM_VERSION",
                                                 # 本台的多机配置：交多机配置那几格自己设
                                                 "SINGLEFS_MULTI_HOST_CONFIG")}
        self.base_environment.update(PATH=self.bin_directory + os.pathsep + os.environ.get("PATH", ""), FAKE_CARGO_LOG=self.log,
                                     GATE_MUTATION_TARGET_DIR=os.path.join(work, "target"),
                                     GATE_MUTATION_WORKERS="1", GATE_MUTATION_LLVM_TOOLS=self.llvm_directory)
        # 目标盘余量：别的格不比它，给一个用不完的假值；磁盘余量那几格自己设
        self.base_environment[DISK_FREE_VARIABLE] = "100T"
        # 内存上限：别的格不比它，这一项没有——服务交的变量不设（调用方设的已随 STAGE_VARIABLES 清掉），下面假 memory.max 写 max，交给本模块的多机配置不写 SLICE_MEMORY_CAP_BY_HOST；
        #   有了上限就要读每个工作进程的峰值记录，各格的运行记录目录随各自的小仓变（run_records_directory 的命名空间），没有实测就只开 1 个。内存上限那几格自己设
        # 自己 scope 的 memory.max：它的有限值是硬顶（task_memory_cap_bytes），自证跑在任务服务给的 scope 里，真 scope 的上限会把别的格要的工作进程数收窄；
        # 换成临时目录里写着 max（不限）的一份（GATE_MUTATION_CGROUP_LISTING、GATE_MUTATION_CGROUP_ROOT），内存上限那几格自己改
        cgroup_listing = os.path.join(work, "selftest-cgroup-listing")
        cgroup_root = os.path.join(work, "selftest-cgroup")
        selftest_write(cgroup_listing, "0::/selftest.slice\n")
        selftest_write(os.path.join(cgroup_root, "selftest.slice", "memory.max"), "max\n")
        self.base_environment["GATE_MUTATION_CGROUP_LISTING"] = cgroup_listing
        self.base_environment["GATE_MUTATION_CGROUP_ROOT"] = cgroup_root
        # 复用哨兵：别的格比的是现判与复用的条数，键相同的全当哨兵、不抽强制现判；reuse-three-questions 与 reuse-sentinel 两格自己设
        self.base_environment[reuse_checks.SENTINEL_COUNT_VARIABLE] = "1000"
        self.base_environment.pop(reuse_checks.SENTINEL_SEED_VARIABLE, None)
        # 交给真跑测试那几格的多机配置（文件头「真跑测试的格把多机配置交给测试、带 gpu 特性的格跑完核真用了卡」）：调用方设的不带进来，
        # 本模块所在仓那一份也不读（有没有、写了什么随机器变），换成临时目录里的一份；交多机配置那几格自己设
        self.base_environment.pop("SINGLEFS_MULTI_HOST_CONFIG", None)
        self.multi_host_configuration = os.path.join(work, "selftest-multi-host", "multi-host.env")
        selftest_write(self.multi_host_configuration, "# 自证用的多机配置：假 cargo 不读它，交多机配置那几格只核它的路径到没到测试的环境里\nENABLE_GPU=1\n")
        self.base_environment["GATE_MUTATION_MULTI_HOST_CONFIG"] = self.multi_host_configuration
        # 用时表与占盘记录（运行记录，文件头「判定库」末段）放这一趟自证自己的目录，不读真仓的多机配置、不碰真共享根
        self.base_environment["GATE_MUTATION_RUN_RECORDS_DIRECTORY"] = os.path.join(work, "selftest-run-records")
        # 判法单元指纹的磁盘缓存（文件头「判法单元」末段）放进这一趟自证自己的临时目录，不碰真仓那一份
        self.base_environment[JUDGING_CACHE_DIRECTORY_VARIABLE] = os.path.join(work, "judging-units-cache")
        # lib_function_units.py 的 Rust 词法单元缓存（它文件头「Rust 词法单元的缓存」）同样放进这一趟自证自己的目录
        self.base_environment["FUNCTION_UNITS_LEX_CACHE_DIRECTORY"] = os.path.join(work, "rust-lex-cache")

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


def plain_invocations(invocations):
    """假 cargo 的记录里基线与变异那几次（记覆盖的 --no-run 与 --exact 两次不算）。"""
    return [line for line in invocations if "--no-run" not in line and "--exact" not in line]


def coverage_invocations(invocations):
    return [line for line in invocations if "--no-run" in line or "--exact" in line]


def mutation_invocations(invocations, pristine_hash):
    """假 cargo 的记录里改坏了源码的那几次（lib.rs 的哈希不是没改坏的那一份）。"""
    return [line for line in invocations if not line.endswith("\t" + pristine_hash)]


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


def loaded_repository_code(extra_paths=()):
    """本进程载入的、文件落在本仓里的模块（sys.modules 里 __file__ 在 STAGE_REPOSITORY 底下的）连同 extra_paths（相对仓根）此刻的内容：
    {相对仓根的路径: sha256（读不了是 None）}。自证「一趟跑到一半整组 SIGKILL，重跑同一条命令」在被杀那一趟起之前、重跑那一趟退出之后各取一次：
    两趟子进程各自现载入本仓的代码、照本仓现在那一份算判法单元，中间哪一份变了就对不上。"""
    root = os.path.realpath(STAGE_REPOSITORY)
    paths = set(extra_paths)
    for module in list(sys.modules.values()):
        path = getattr(module, "__file__", None)
        if not isinstance(path, str):
            continue
        real = os.path.realpath(path)
        if real.startswith(root + os.sep):
            paths.add(os.path.relpath(real, root))
    return {relative: read_value("open", os.path.join(root, relative)) for relative in sorted(paths)}


def library_of(repository):
    """自证里那棵仓的判定库目录：只当 prune-server（临时服务）里的名字与命名空间，本地没有文件（文件头「判定库」）。"""
    return os.path.join(repository, ".git", LIBRARY_DIRECTORY_NAME)


def selftest_library(directory):
    """自证读写库目录名是 directory 的那一份库（经 prune-server；库目录在临时目录底下，命名空间一趟一份）。"""
    return mutation_units.Library(directory, RECORD_HEADER)


def selftest_worker_count_cells(selftest, work):
    """工作进程数（文件头「工作进程数」）：GATE_MUTATION_WORKERS（顶掉每台默认重活线程数那一项）与要现跑的条数取最小，这几格直接调 worker_count_for 不给内存项（内存那几格在 selftest_memory_cells），头一行一个字都不提内存；
    没给 GATE_MUTATION_WORKERS 时取每台默认重活线程数（SINGLEFS_THREAD_CAP=7 取 7），取不到（SINGLEFS_THREAD_CAP=abc）抛 HeavyThreadDefaultError、整道退 2。
    弄坏开关 workers-by-processor-half 让后两格判错（取核数一半至多 16、不认登记值，也不拒）。"""
    sixteen = dict(selftest.base_environment, GATE_MUTATION_WORKERS="16")
    many, many_head = worker_count_for(20, sixteen)
    few, few_head = worker_count_for(5, sixteen)
    selftest.check("工作进程数取 GATE_MUTATION_WORKERS 与要现跑的条数的小者，没给内存项时头一行不提内存（20 条开 16 个、5 条开 5 个）",
                   many == 16 and "卡在「GATE_MUTATION_WORKERS」这一项" in many_head and few == 5 and "卡在「这一趟要现跑的条数」这一项" in few_head
                   and "内存" not in many_head + few_head, f"{many}：{many_head}；{few}：{few_head}")
    registered_environment = {key: value for key, value in selftest.base_environment.items() if key != "GATE_MUTATION_WORKERS"}
    registered_environment["SINGLEFS_THREAD_CAP"] = "7"
    by_registry, registry_head = worker_count_for(20, registered_environment)
    selftest.check("没给 GATE_MUTATION_WORKERS：工作进程数取每台默认重活线程数（SINGLEFS_THREAD_CAP=7、20 条开 7 个，卡在「每台默认重活线程数」这一项）",
                   by_registry == 7 and "卡在「每台默认重活线程数」这一项" in registry_head, f"{by_registry}：{registry_head}")
    repository = os.path.join(work, "workers-unregistered")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS[:1])
    refused = selftest.run(repository, GATE_MUTATION_WORKERS="", GATE_MUTATION_KEY_WORKERS="2", SINGLEFS_THREAD_CAP="abc")
    selftest.check("没给 GATE_MUTATION_WORKERS、取不到每台默认重活线程数（SINGLEFS_THREAD_CAP=abc）：整道退 2、报「取不到每台默认重活线程数」，一次 cargo test 都不起",
                   refused[0] == 2 and "取不到每台默认重活线程数" in refused[1] + refused[2] and not plain_invocations(refused[3]),
                   f"退 {refused[0]}；cargo {len(refused[3])} 次；{(refused[1] + refused[2])[-600:]}")


# 自证「进程池还没起就被停」那一格在子进程里跑（按路径载入本模块）：拿掉 concurrent.futures 的 process 子模块（进程池还没起时它没载入），
# 把 run_pool 换成一进来就抛 StopRequested 的，再调 run_pool_reporting_dead_worker
SELFTEST_POOL_ERROR_CHILD = ("import concurrent.futures, importlib.util, os, sys\n"
                             "specification = importlib.util.spec_from_file_location('crates_mutation_rows_pool_error_child', sys.argv[1])\n"
                             "module = importlib.util.module_from_spec(specification)\n"
                             "sys.modules[specification.name] = module\n"
                             "specification.loader.exec_module(module)\n"
                             "sys.modules.pop('concurrent.futures.process', None)\n"
                             "concurrent.futures.__dict__.pop('process', None)\n"
                             "def stopped_before_the_pool(*arguments, **keywords):\n"
                             "    raise module.StopRequested(143)\n"
                             "module.run_pool = stopped_before_the_pool\n"
                             "module.run_pool_reporting_dead_worker(dict(environment=dict(os.environ)), 1, [], [], set(), None, None)\n")


def selftest_stop_before_pool_cell(selftest, work):
    """进程池还没起（先编一份依赖那一段）就被 TERM 停：concurrent.futures 的 process 子模块此刻没载入，「有工作进程中途死了」那一句的 except 照样认出
    这是 StopRequested、原样往外抛，退 143，不在判 except 的时候抛 AttributeError（灰度那一趟的样子：被停成了回溯退 1）。
    弄坏开关 broken-pool-looked-up-late（判 except 的时候按 concurrent.futures.process 属性找）下这一格必须判错。"""
    child = subprocess.run([sys.executable, "-c", SELFTEST_POOL_ERROR_CHILD, MODULE_PATH], capture_output=True, text=True, errors="replace",
                           env=dict(selftest.base_environment), stdin=subprocess.DEVNULL)
    selftest.check("进程池还没起就被 TERM 停（process 子模块没载入）：StopRequested 原样往外抛、退 143，不抛 AttributeError",
                   child.returncode == 128 + signal.SIGTERM and "AttributeError" not in child.stderr,
                   f"退 {child.returncode}；stderr 尾：{child.stderr.strip()[-300:]}")


def selftest_libtest_result_cell(selftest, work):
    """libtest 结果行经 lib_libtest_result_lines.py：should panic 的用例红了判抓到；点名省掉模块路径时只对上全名以「::<点名>」收尾的那一个。"""
    row = Row(8, "selftest-should-panic", "crates/selftest/src/lib.rs", "a", "b", "-p selftest --lib", "answer_panics", [])
    output = ("running 2 tests\ntest tests::answer_is_42 ... ok\ntest tests::answer_panics - should panic ... FAILED\n\n"
              "test result: FAILED. 1 passed; 1 failed; 0 ignored\n\nerror: test failed, to rerun pass `-p selftest --lib`\n")
    verdict, text = verdict_of_run(row, 101, output, environment=os.environ)
    results = results_by_test("answer_panics", output + "test other::not_answer_panics ... ok\n")
    selftest.check("should panic 的用例红了判抓到、点名省掉模块路径只对上那一个",
                   verdict == "caught" and results == {"tests::answer_panics": ["FAILED"]}, f"判成 {verdict}：{text[:200]}；对上的 {results}")


SELFTEST_SPLIT_LIFECYCLE_LINE = ('GPU_LIFECYCLE unix_us=1793843585212043 elapsed_us=12792699 pid=67478 thread=ThreadId(7) test=Some("{name}") '
                                 'phase=queue_submit_end card=0 pci="01:00.0" states=0..1')


def selftest_split_result_line_cell(selftest, work):
    """结果词被测试中途打的行隔开：`test <名> ... ` 之后隔着 GPU_LIFECYCLE 行、结局另起一行单独写。
    两条行共一批基线：各自照认基线绿（只看自己过滤词选中的那条）；隔开的 FAILED 照认抓到；起头之后先来了别的起头的那一条认不出结局，不接；
    没起头可接的单独结局行多出来时整段不接。弄坏开关 split-result-line-unseen：不接，头一问判成「一行都没出现」。"""
    environment = os.environ
    first = Row(8, "selftest-split-first", "crates/selftest/src/lib.rs", "a", "b", "-p selftest --lib -- the_gpu_judge_reddens_first", "the_gpu_judge_reddens_first", [])
    second = Row(9, "selftest-split-second", "crates/selftest/src/lib.rs", "a", "b", "-p selftest --lib -- the_gpu_judge_reddens_second", "the_gpu_judge_reddens_second", [])
    lifecycle = SELFTEST_SPLIT_LIFECYCLE_LINE
    split_output = ("running 2 tests\n"
                    "test tests::the_gpu_judge_reddens_first ... " + lifecycle.format(name="first") + "\n" + lifecycle.format(name="first") + "\nok\n"
                    "test tests::the_gpu_judge_reddens_second ... " + lifecycle.format(name="second") + "\nFAILED\n\n"
                    "test result: FAILED. 1 passed; 1 failed; 0 ignored\n")
    first_view = row_baseline_output(first, split_output, environment)
    second_view = row_baseline_output(second, split_output, environment)
    first_problem = baseline_problem(first, 101, first_view)
    first_results = results_by_test(first.expected, first_view, environment)
    second_results = results_by_test(second.expected, split_output, environment)
    verdict, text = verdict_of_run(second, 101, split_output, environment)
    selftest.check("结果词被插进来的 GPU_LIFECYCLE 行隔开时照认基线绿：一批两条各看自己的、隔开的 FAILED 判抓到",
                   first_problem is None and first_results == {"tests::the_gpu_judge_reddens_first": ["ok"]}
                   and first_view.count("test tests::") == 1 and second_view.count("test tests::") == 1
                   and second_results == {"tests::the_gpu_judge_reddens_second": ["FAILED"]} and verdict == "caught",
                   f"第一条的基线问题 {first_problem}，对上的 {first_results}；第二条对上的 {second_results}，判成 {verdict}：{text[:160]}")
    interrupted = ("running 2 tests\ntest tests::lost_one ... " + lifecycle.format(name="lost_one") + "\n"
                   "test tests::kept_one ... " + lifecycle.format(name="kept_one") + "\nok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored\n")
    lost_results = results_by_test("lost_one", interrupted, environment)
    orphaned = split_output + "ok\n"
    orphaned_results = results_by_test("the_gpu_judge_reddens_first", orphaned, environment)
    selftest.check("不该接的不接：起头之后先来了别的起头的那一条认不出结局、没起头可接的单独结局行多出来时整段不接",
                   lost_results == {} and orphaned_results == {},
                   f"被打断的那一条对上 {lost_results}，多出一行单独结局时第一条对上 {orphaned_results}")


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
    environment = dict(os.environ, TMPDIR=temporary, FAKE_CARGO_HANG_FILE=hang, FAKE_CARGO_RELEASE_FILE=released,
                       FAKE_CARGO_DONE_FILE=done, GATE_MUTATION_TARGET_DIR="")   # 编译目录建在这一趟的 shard_root 里（文件头「编译目录」），停完连同它删
    stderr_path = os.path.join(work, "signalled-stderr.txt")
    # stderr 落文件、不接管道：没接住信号时工作进程成了孤儿、还开着它，接管道就等不到头
    with open(stderr_path, "w", encoding="utf-8") as stderr_handle:
        child = subprocess.Popen([sys.executable, "-c", SELFTEST_STAGE_CHILD, MODULE_PATH, repository], env=environment,
                                 stdout=subprocess.DEVNULL, stderr=stderr_handle)
    # TERM 发在「$TMPDIR 下的临时目录建出来了、假 cargo 在工作进程里挂住了（标记文件里有工作进程的进程号）」之后：run_pool 装好信号处理、建了临时目录、
    # 起了工作进程，那时收到 TERM 才走「收尾再退」；在那之前（解释器还在载入、run_stage 还在算键）收到的就是默认处理，退 -15、什么也没建。
    # 机器忙时头一步可以慢到分钟，所以按条件等（轮询，带超时），不靠定长；等不到就不发（子进程自证自己停掉），这一格判错并写明没等到什么。
    # 弄坏开关 term-sent-unwaited：不等条件，起了子进程立刻发 TERM（碰上忙机器的样子），这一格必须判错
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


def selftest_killed_run_resume_cell(selftest, work):
    """判过的格被整组硬杀也留下（文件头「判定库」：每条判完立刻写、写入原子）：一趟 run_stage 在子进程里起（自己一个会话），一个工作进程，每次 cargo 睡 2 秒（FAKE_CARGO_SPAN_SLEEP）；
    等到第三条变异的 cargo 起来（前两条判完、记录写完，第三条在睡）就对整组发 SIGKILL（进程池的工作进程连同父进程一起死，与内存包装杀整组一样不给任何收尾的机会），
    等第三条的假 cargo 自己退出，再重跑同一条命令：判完的 2 条照键复用，只现判剩下的 2 条（基线记录也留着，基线不再跑）。
    重跑同样起子进程、现载入本仓的代码，不在自证进程里调 run_stage：自证进程里的是开跑时载入的那一份，一趟自证跑二十多分钟，别的会话中途改了判法代码
    （如 lib_function_units.py）时两趟算出的键就不同。两趟的判法单元都照本仓现在那一份算（不照开跑时冻好的拷贝：新代码在旧拷贝上算，闭包多出的文件拷贝里没有）。
    两趟之间本进程载入的本仓代码与冻好的那份拷贝里列着的判法文件（loaded_repository_code）有哪一份变了，这一格记本次未跑。
    前提没造出来（第三条起来时前两条没恰好判完：机器忙到 2 秒内没轮询到）也记本次未跑。
    弄坏开关 records-written-at-end（记录攒在工作进程内存里、进程池正常收尾才写）下被杀之后库里一条都没有，这一格必须判错；
    killed-run-rerun-keyed-by-other-code（重跑那一趟另加 judge-units-whole-file）下两趟的键不同，这一格同样必须判错。"""
    child_script = ("import importlib.util, os, sys\n"
                    "specification = importlib.util.spec_from_file_location('crates_mutation_rows_selftest_child', sys.argv[1])\n"
                    "module = importlib.util.module_from_spec(specification)\n"
                    "sys.modules[specification.name] = module\n"
                    "specification.loader.exec_module(module)\n"
                    "sys.exit(module.run_stage(sys.argv[2], dict(os.environ), sys.argv[3]))\n")
    repository = os.path.join(work, "killed-run")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    pristine = source_hash(repository)
    temporary = os.path.join(work, "killed-run-tmp")
    os.makedirs(temporary)
    target = os.path.join(work, "killed-run-target")
    span_log = os.path.join(work, "killed-run-spans.log")
    environment = dict(selftest.base_environment, TMPDIR=temporary, GATE_MUTATION_WORKERS="1", GATE_MUTATION_TARGET_DIR=target,
                       FAKE_CARGO_SPAN_LOG=span_log, FAKE_CARGO_SPAN_SLEEP="2")

    def span_events():
        if not os.path.exists(span_log):
            return []
        with open(span_log, encoding="utf-8") as handle:
            return [line.split("\t") for line in handle.read().splitlines() if line.count("\t") == 2]

    def mutation_events(kind):
        return [event for event in span_events() if event[0] == kind and event[1].startswith("line-")]

    def start_child(child_environment, stdout_handle, stderr_handle):
        # 两趟同一条命令：子进程现载入本仓的这一份模块，判法单元照本仓现在那一份算（与载入的代码同一时刻）
        return subprocess.Popen([sys.executable, "-c", child_script, MODULE_PATH, repository, STAGE_REPOSITORY], env=child_environment, start_new_session=True,
                                stdout=stdout_handle, stderr=stderr_handle)

    # 冻好的那份拷贝里列着的就是开跑时判法单元闭包碰到的文件（judging_copy）：连同载入的模块一起核两趟之间变没变
    frozen = os.path.realpath(JUDGING_ROOT[0])
    judging_files = ([] if frozen == os.path.realpath(STAGE_REPOSITORY) else
                     [os.path.relpath(os.path.join(directory, name), frozen) for directory, _subdirectories, names in os.walk(frozen) for name in names])
    code_before = loaded_repository_code(judging_files)
    stderr_path = os.path.join(work, "killed-run-stderr.txt")
    with open(stderr_path, "w", encoding="utf-8") as stderr_handle:
        child = start_child(environment, subprocess.DEVNULL, stderr_handle)
    deadline = time.monotonic() + 300
    while len(mutation_events("start")) < 3 and child.poll() is None and time.monotonic() < deadline:
        time.sleep(0.02)
    started, ended = len(mutation_events("start")), len(mutation_events("end"))
    exited_early = child.poll() is not None
    if not exited_early:
        os.killpg(child.pid, signal.SIGKILL)   # 整组：子进程自己是会话首进程，进程池的工作进程在同一个进程组里；只对自证自己起的这一组发
    child.wait(timeout=60)
    # 被杀的那一格的假 cargo 在自己的会话里，睡完自己退：等它的 end 记下来，免得它晚到把一行写进重跑那一趟的 cargo 记录
    settle_deadline = time.monotonic() + 60
    while time.monotonic() < settle_deadline and len([event for event in span_events() if event[0] == "start"]) != len([event for event in span_events() if event[0] == "end"]):
        time.sleep(0.05)
    if exited_early:
        selftest.check("一趟跑到一半整组 SIGKILL 之后重跑同一条命令：第一趟子进程在被杀之前没有自己退出", False,
                       f"子进程退 {child.returncode}，起 {started} 条变异、判完 {ended} 条；stderr 尾：{open(stderr_path, encoding='utf-8', errors='replace').read()[-400:]}")
        return
    if (started, ended) != (3, 2):
        selftest.skip("被整组硬杀的一趟判过的格留下，重跑同一条命令照键复用", f"第三条变异的 cargo 起来时前两条没恰好判完（起 {started} 条、判完 {ended} 条）")
        return
    library = selftest_library(library_of(repository))
    recorded = 0
    for name in library.stored_names("records"):
        text = library.read_text("records", name) or ""
        recorded += 1 if "kind=mutation" in text and "verdict=caught" in text else 0
    rerun_environment = dict(selftest.base_environment, GATE_MUTATION_TARGET_DIR=target)
    if break_is(selftest.base_environment, "killed-run-rerun-keyed-by-other-code"):
        rerun_environment["GATE_MUTATION_BREAK"] = ",".join(filter(None, [rerun_environment.get("GATE_MUTATION_BREAK", ""), "judge-units-whole-file"]))
    with contextlib.suppress(FileNotFoundError):
        os.unlink(selftest.log)
    rerun_stdout_path, rerun_stderr_path = os.path.join(work, "killed-run-rerun-stdout.txt"), os.path.join(work, "killed-run-rerun-stderr.txt")
    with open(rerun_stdout_path, "w", encoding="utf-8") as stdout_handle, open(rerun_stderr_path, "w", encoding="utf-8") as stderr_handle:
        rerun_child = start_child(rerun_environment, stdout_handle, stderr_handle)
    try:
        rerun_exit = rerun_child.wait(timeout=600)
    except subprocess.TimeoutExpired:
        os.killpg(rerun_child.pid, signal.SIGKILL)   # 只对自证自己起的这一组发
        rerun_child.wait(timeout=60)
        rerun_exit = f"600 秒没跑完、整组杀掉（退 {rerun_child.returncode}）"
    with open(rerun_stderr_path, encoding="utf-8", errors="replace") as handle:
        rerun_stderr = handle.read()
    rerun_invocations = []
    if os.path.exists(selftest.log):
        with open(selftest.log, encoding="utf-8") as handle:
            rerun_invocations = handle.read().splitlines()
    code_after = loaded_repository_code(judging_files)
    changed = sorted(path for path in set(code_before) | set(code_after) if code_before.get(path) != code_after.get(path))
    if changed:
        selftest.skip("被整组硬杀的一趟判过的格留下，重跑同一条命令照键复用",
                      f"两趟之间本仓里载入的代码变了（别的会话在改）：{'、'.join(changed[:5])}{' 等 ' + str(len(changed)) + ' 份' if len(changed) > 5 else ''}；"
                      "两趟算键的代码不同，复用不上说明不了什么，代码不动时再跑")
        return
    rerun_head = (rerun_stderr.split("判定库记录：", 1) + [""])[1].splitlines()[0]
    rerun_reasons = [line.strip() for line in rerun_stderr.splitlines() if line.startswith("  · 现判：")]
    resumed = len(mutation_invocations(rerun_invocations, pristine))
    selftest.check("一趟跑到一半整组 SIGKILL（前两条变异判完、第三条在跑），重跑同一条命令：判完的 2 条每条都已原子写进判定库、照键复用，只现判剩下的 2 条（基线记录也留着，基线不再跑）",
                   child.returncode == -signal.SIGKILL and recorded == ended and rerun_exit == 0 and f"复用抓到 {ended} 条、" in rerun_head and "现跑基线 0 批 0 行" in rerun_head
                   and resumed == len(SELFTEST_TABLE_ROWS) - ended and len(plain_invocations(rerun_invocations)) == resumed,
                   f"子进程退 {child.returncode}；被杀时判完 {ended} 条、库里抓到的记录 {recorded} 条；重跑退 {rerun_exit}，头一行「{rerun_head[:300]}」，"
                   f"现判的原因「{'；'.join(rerun_reasons[:2])[:400]}」，"
                   f"重跑起的变异 {resumed} 次、基线与变异合计 {len(plain_invocations(rerun_invocations))} 次；第一趟 stderr 尾：{open(stderr_path, encoding='utf-8', errors='replace').read()[-300:]}")


def selftest_shard_root_window_cell(selftest, work):
    """TERM 落在 run_pool 建 shard_root 的 mkdir 之后、名字交到调用方手里之前（tempfile.mkdtemp 的窗口）：在进程内把 os.mkdir 换成「真建出
    SHARD_ROOT_PREFIX 起头的目录之后抛 StopRequested」（等同 TERM 恰好落在那一瞬），跑 run_pool，退出码是 143、$TMPDIR 下一个目录都不剩。
    弄坏开关 shardrootnamedlate（用 tempfile.mkdtemp，名字丢了）下这一格必须判错。"""
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
    exit_code, stdout, stderr, invocations = selftest.run(repository)
    feature_runs = [line for line in plain_invocations(invocations) if "--features gpu" in line]
    selftest.check("拷贝范围含 .cargo/：点名的测试要编 gpu 特性的那一行基线绿、变异抓到",
                   exit_code == 0 and "基线不绿 0 行" in stdout and "计数：抓到 2 条、没红 0 条" in stdout and len(feature_runs) == 2,
                   f"退 {exit_code}；假 cargo 带 --features gpu 起了 {len(feature_runs)} 次（要基线 1 次、变异 1 次）；拷贝范围 {copied_paths()}；{stdout}{stderr[-400:]}")


def selftest_product_gpu_feature_cells(selftest, work):
    """产品 GPU 包（singlefs-background-gpu）真显卡对拍开的 tests-on-a-real-gpu 与 gpu 同样当上卡的特性认（GPU_TEST_FEATURES、gpu_test_run）。
    这一道不另套清场（文件头「带 gpu 特性的格不另套清场」）：两格的假 cargo 记录里起的就是 cargo 本身。两格：
    ① 带 tests-on-a-real-gpu 的那一行真跑测试的几次（基线、变异、插桩跑）各收尾带 --test-threads 1，只编不跑的与不带的那一行不带，照抓到；
    ② 点名的测试在没卡的机器上打 NOT_RUN_NO_GPU_ADAPTER 后返回成功：记本次未跑，不算抓到也不算没红，整道不判红，真跑的那几次同样收尾带 --test-threads 1。
    弄坏开关 product-gpu-feature-unquiesced 下这一特性不当上卡的特性认（两格判错）。"""
    def product_world(name, product_row, extra_source=""):
        """一棵带一行不带上卡特性、一行带 tests-on-a-real-gpu（product_row）的小仓。"""
        root = os.path.join(work, name)
        build_selftest_repository(root, [SELFTEST_TABLE_ROWS[0], product_row], extra_source=extra_source)
        selftest_write(os.path.join(root, CARGO_CONFIGURATION_DIRECTORY, "config.toml"), SELFTEST_CARGO_CONFIGURATION)
        return root

    def split_runs(invocations):
        """(带 tests-on-a-real-gpu 真跑的每一次的参数, 不带它真跑的每一次的参数, 只编不跑的每一次的参数)。"""
        runs = [line.split("\t")[0].split() for line in invocations]
        real = [words for words in runs if "--no-run" not in words]
        return ([words for words in real if "tests-on-a-real-gpu" in words], [words for words in real if "tests-on-a-real-gpu" not in words],
                [words for words in runs if "--no-run" in words])

    root = product_world("product-gpu-feature", SELFTEST_PRODUCT_GPU_FEATURE_ROW)
    exit_code, stdout, stderr, invocations = selftest.run(root)
    product_runs, plain_runs, compile_runs = split_runs(invocations)
    selftest.check("产品 GPU 包（--features tests-on-a-real-gpu）的行：真跑测试的那几次各收尾带 --test-threads 1，只编不跑的与不带的不带，照抓到",
                   exit_code == 0 and "计数：抓到 2 条、没红 0 条" in stdout and len(product_runs) == 3
                   and all(words[-2:] == ["--test-threads", "1"] for words in product_runs)
                   and plain_runs and all("--test-threads" not in words for words in plain_runs)
                   and compile_runs and all("--test-threads" not in words for words in compile_runs),
                   f"退 {exit_code}；带 tests-on-a-real-gpu 真跑 {product_runs}（要基线、变异、插桩跑各 1 次，收尾都是 --test-threads 1）；不带的 {plain_runs}；"
                   f"只编不跑 {compile_runs}；{stdout[-600:]}{stderr[-400:]}")
    root = product_world("product-gpu-card-absent", SELFTEST_PRODUCT_GPU_NOT_RUN_ROW, SELFTEST_NOT_RUN_TEST)
    not_run_record = os.path.join(work, "product-gpu-not-run-record.txt")
    exit_code, stdout, stderr, invocations = selftest.run(root, GATE_MUTATION_NOT_RUN_RECORD=not_run_record)
    recorded_lines = open(not_run_record, encoding="utf-8").read().splitlines() if os.path.exists(not_run_record) else []
    product_runs, _plain_runs, _compile_runs = split_runs(invocations)
    selftest.check("产品 GPU 包（--features tests-on-a-real-gpu）的行：点名的测试没卡打 NOT_RUN_NO_GPU_ADAPTER 后返回成功，记本次未跑（不算抓到、不算没红），"
                   "整道不判红，记录里一行，真跑的那几次收尾带 --test-threads 1",
                   exit_code == 0 and "本次未跑：1 条" in stdout and "计数：抓到 1 条、没红 0 条" in stdout
                   and f"{TABLE}:3 product-card-path-changed：本次未跑——点名的测试 gpu_parity 打了 NOT_RUN_NO_GPU_ADAPTER（样本：本机没有 GPU 适配器）" in stdout
                   and len(recorded_lines) == 1 and "product-card-path-changed" in recorded_lines[0]
                   and len(product_runs) >= 2 and all(words[-2:] == ["--test-threads", "1"] for words in product_runs),
                   f"退 {exit_code}；带 tests-on-a-real-gpu 真跑 {product_runs}（基线、变异至少 2 次，收尾都是 --test-threads 1）；记录 {recorded_lines}；"
                   f"{stdout[-900:]}{stderr[-400:]}")


def selftest_gpu_configuration_cells(selftest, work):
    """真跑测试的格把多机配置交给测试、带 gpu 特性的格跑完核真用了卡（文件头同名一段）。PATH 上换一份包着假 cargo 的假 cargo（这一格自己写，FAKE_CARGO 不动）：
    每起一次往记录里写一行 JSON（参数、起在哪个目录、收到的 SINGLEFS_MULTI_HOST_CONFIG），点名 gpu_parity、带 --show-output、它是 ok 的那一次
    照 libtest 的样子在末尾交出 $GIVEN_STATES_TALLIES（<GPU 判的>/<CPU 判的>，逗号分开几组）那几行 CRASH_POINT_REPLAY_GIVEN_STATES。五格：
    ① 交多机配置：一行不带 gpu、一行带 gpu（都抓到），两行真跑测试的每一次（基线、变异、插桩跑）在副本里起的 cargo 都收到
       SINGLEFS_MULTI_HOST_CONFIG=那份配置的绝对路径，只编不跑的收不到；
    ② 多机配置不在：run_stage 起 cargo 之前就拒、退 2、点名那份配置，假 cargo 一次都没起；
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
            root, GATE_MUTATION_MULTI_HOST_CONFIG=configuration, GPU_CONFIGURATION_LOG=log, GIVEN_STATES_TALLIES=tallies,
            INNER_FAKE_CARGO=os.path.join(selftest.bin_directory, "cargo"), PATH=wrapper_directory + os.pathsep + selftest.base_environment["PATH"])
        return root, exit_code, stdout + stderr[-600:], environment_records(log)

    def test_runs(records, with_gpu):
        return [record for record in records if ("--features gpu" in record["ARGUMENTS"]) == with_gpu and "--no-run" not in record["ARGUMENTS"].split()]

    configuration = os.path.join(work, "gpu-configuration-present", "multi-host.env")
    selftest_write(configuration, "# 自证用的多机配置：假 cargo 不读它，这一格只核它的路径到没到测试的环境里\nENABLE_GPU=1\n")
    root, exit_code, output, records = configuration_world("gpu-configuration-passed", "3/0", configuration, "value / 2 }")
    with_gpu, without_gpu = test_runs(records, True), test_runs(records, False)
    compiles = [record for record in records if "--no-run" in record["ARGUMENTS"].split()]
    received = [record.get("SINGLEFS_MULTI_HOST_CONFIG") for record in with_gpu + without_gpu]
    selftest.check("真跑测试的每一次在副本里起的 cargo 环境里有 SINGLEFS_MULTI_HOST_CONFIG 指向本模块所在仓的多机配置（绝对路径，带不带 gpu 都有），只编不跑的没有",
                   exit_code == 0 and "计数：抓到 2 条、没红 0 条" in output and len(with_gpu) == 3 and without_gpu and received == [configuration] * len(received)
                   and all(record["WORKING_DIRECTORY"] != root for record in with_gpu + without_gpu) and compiles
                   and all(record.get("SINGLEFS_MULTI_HOST_CONFIG") is None for record in compiles),
                   f"退 {exit_code}；带 gpu 真跑 {len(with_gpu)} 次（要基线、变异、插桩跑各 1 次）、不带 gpu 真跑 {len(without_gpu)} 次，收到 {received}（要都是 {configuration}），"
                   f"起在 {[record['WORKING_DIRECTORY'] for record in with_gpu + without_gpu]}（不许是 {root}）；只编不跑的收到 "
                   f"{[record.get('SINGLEFS_MULTI_HOST_CONFIG') for record in compiles]}（要都没有）；{output[-700:]}")
    absent = os.path.join(work, "gpu-configuration-absent", "multi-host.env")
    _root, exit_code, output, records = configuration_world("gpu-configuration-missing", "3/0", absent, "{ value * 2 }")
    selftest.check("多机配置不在：run_stage 退 2、点名那份配置，起 cargo 之前就拒（假 cargo 一次都没起）",
                   exit_code == 2 and f"多机配置 {absent} 不在" in output and not records,
                   f"退 {exit_code}；假 cargo 起了 {len(records)} 次（要 0 次）；{output[-900:]}")
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


PARALLEL_REASON_FORM = re.compile(r"^同一次 crates-mutation-replay（本模块进程 \d+）的第 (\d+) 个工作进程，各用各的源码副本与编译目录（这一轮共 4 个）$")


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
    exit_code, _stdout, stderr, _ = selftest.run(repository, GATE_MUTATION_TARGET_DIR=shared, GATE_MUTATION_WORKERS="2")
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
    in_use_repository = selftest_new_library_copy(work, repository, "target-reclaim-in-use")   # 换一棵树：库是新的一份，这一趟要现跑才开工作进程、才回收
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
        exit_code, _stdout, stderr, _ = selftest.run(in_use_repository, GATE_MUTATION_TARGET_DIR=shared, GATE_MUTATION_WORKERS="2")
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
    unset = dict(GATE_MUTATION_TARGET_DIR="", HOME=home, TMPDIR=temporary, GATE_MUTATION_WORKERS="1",
                 RUSTUP_HOME=os.environ.get("RUSTUP_HOME") or os.path.expanduser("~/.rustup"),
                 CARGO_HOME=os.environ.get("CARGO_HOME") or os.path.expanduser("~/.cargo"))
    problems = []
    saved_tempdir = tempfile.tempdir   # run_stage 在本进程里跑：shard_root 照 tempfile.gettempdir() 建，这里指进 temporary，跑完还原
    for name, extra, wanted_exit in (("正常", {}, 0), ("没跑全", {"GATE_MUTATION_STOP_AFTER_ROWS": "1"}, 1)):
        unset_repository = selftest_new_library_copy(work, repository, f"target-reclaim-unset-{wanted_exit}")   # 每一趟换一棵树：库是新的一份，各自要现跑
        tempfile.tempdir = temporary
        try:
            exit_code, _stdout, stderr, _ = selftest.run(unset_repository, **dict(unset, **extra))
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
    弄坏开关 overlapunchecked：不核，一律回 None，两次没重叠也照判对错。"""
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
        environment = dict(selftest.base_environment, GATE_MUTATION_TARGET_DIR=shared, GATE_MUTATION_WORKERS="2", TMPDIR=temporary,
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



def environment_records(path):
    """假 cargo 的 FAKE_CARGO_ENVIRONMENT_LOG：每起一次 test 一行 JSON（假 cargo 头上那一段）；文件不在交空表。"""
    if not os.path.exists(path):
        return []
    with open(path, encoding="utf-8") as handle:
        return [json.loads(line) for line in handle.read().splitlines() if line.strip()]


def selftest_write_random(path, size):
    """size 字节的随机内容（压不下去、占盘就是 size 往上），自证里造占盘的编译目录用。"""
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb") as handle:
        handle.write(os.urandom(size))


def selftest_memory_cells(selftest, work):
    """开几个工作进程按内存上限收（文件头「内存上限」）：内存上限的取法（服务交的变量、多机配置、自己 scope 的 memory.max 与它当硬顶）与除出来的个数，
    同线程数（GATE_MUTATION_WORKERS 顶掉登记值）、要现跑的条数取最小、至少 1。每个工作进程的峰值在这几格固定：编译并行度照 GATE_MUTATION_CARGO_JOBS=4 固定，
    临时运行记录目录里编译并行度 4 一份 4 GiB 的实测，个数就是 ⌊上限 ÷ 4 GiB⌋；量、记、读、除的那几格在 selftest_worker_peak_cells。
    直接调 worker_count_for 与 memory_bound_for，/proc/self/cgroup 与 /sys/fs/cgroup 换成临时目录里造的假文件：
    ① 服务交的 TASK_SERVER_MEMORY_CAP_BYTES=20 GiB（假多机配置的 SLICE_MEMORY_CAP_BY_HOST 这一台那一项写 50G、假 memory.max 写 64 GiB 摆在旁边）、线程数 16、20 条：开 5 个，
      头一行写明上限、每个工作进程的峰值、个数与合计、卡在「内存上限」（假 memory.max 写 64 GiB 不比它小，不收窄：服务交的变量先于多机配置）；
    多机配置两格：无服务变量、假多机配置的 SLICE_MEMORY_CAP_BY_HOST 这一台那一项写 24G、另一台那一项写 48G（整值带引号）、假 memory.max 64 GiB 在旁边：取这一台那一项，开 6 个；
    这一台那一项写 lots：判法整份判不过、认不出这一台，头一行点名、改读 memory.max（8 GiB）开 2 个；
    多机配置照判法写全键（判法认这一台要整份判得过）：主机数组写 127.0.0.1（每一台的网卡上都有，是这一台）与文档用的地址段 192.0.2.70；
    ② 上限 100 GiB 放得下 25 个，线程数 16 更小：开 16 个，卡在 GATE_MUTATION_WORKERS；③ 上限 1 GiB 一个都放不下：按 1 个开，头一行写「那一项不足 1」；
    ⑤ 没有服务交的变量、假 memory.max 写 8589934592：开 2 个，头一行写 memory.max 的路径；写 max：不收，照线程数开 16 个，头一行写「是 max」；假 /proc/self/cgroup 读不到：同样不收；
    ⑥ TASK_SERVER_MEMORY_CAP_BYTES 写成 abc：改读 memory.max（8 GiB 开 2 个），头一行点名写法不对。再起一趟 run_stage：线程数 16，上限 8 GiB、每个 4 GiB，开 2 个、卡在「内存上限」，
    收尾那一行也写个数。⑧ 再起一趟 run_stage：上限 2 GiB、每个 4 GiB，一个都放不下：退 250（撞内存上限），红句写上限与峰值，假 cargo 一次基线与变异都没起。
    弄坏开关 memory-cap-ignored（不看内存上限，照线程数开满）下这十六格全部判错；memory-cap-short-run-anyway（开 1 个硬跑）下 ⑧ 那一格判错。
    memory.max 是硬顶（scope 真上限 20736M，多机配置写 50G）：⑦ 多机配置 50G（无服务变量）、memory.max 20 GiB：按 20 GiB 开 5 个，头一行写各档读到的值与取 memory.max 的原因；
    服务交的变量 50 GiB、memory.max 20 GiB：同样按 20 GiB 开 5 个；服务交的变量 50 GiB、memory.max 写 max：照 50 GiB 开 12 个（没有硬顶可比）；服务交的变量 50 GiB、/proc/self/cgroup 读不到：照 50 GiB 开 12 个；
    再起一趟 run_stage：服务交的变量 50 GiB、经 GATE_MUTATION_CGROUP_LISTING / GATE_MUTATION_CGROUP_ROOT 指到 memory.max 8 GiB：开 2 个。
    弄坏开关 memory-max-not-hard-cap（取到变量或多机配置的值就不看 memory.max）下 ⑦ 的三格（多机配置、服务交的变量、run_stage）判错，写 max 与读不到那两格不红。"""
    gibibyte = 1 << 30
    scope_directory = os.path.join(work, "memory-cgroup", "fake.slice", "job.scope")
    os.makedirs(scope_directory, exist_ok=True)
    listing = os.path.join(work, "memory-cgroup-listing")
    with open(listing, "w", encoding="utf-8") as handle:
        handle.write("12:memory:/legacy\n0::/fake.slice/job.scope\n")
    root = os.path.join(work, "memory-cgroup")
    memory_max = os.path.join(scope_directory, "memory.max")

    def write_memory_max(text):
        with open(memory_max, "w", encoding="utf-8") as handle:
            handle.write(text)

    configuration = os.path.join(work, "memory-config", "multi-host.env")
    os.makedirs(os.path.dirname(configuration), exist_ok=True)

    def write_configuration(text):
        with open(configuration, "w", encoding="utf-8") as handle:
            handle.write(text)

    def write_slice_configuration(slice_items):
        # 判法（scripts/crash/crash-point-replay-shard-configuration-check.sh）整份判得过的多机配置，SLICE_MEMORY_CAP_BY_HOST 照给的写
        write_configuration("HOSTS=127.0.0.1 192.0.2.70\nCARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=/selftest/this-cargo 192.0.2.70=/selftest/other-cargo\n"
                            "BUILD_ROOT_BY_HOST=127.0.0.1=/selftest/this-builds 192.0.2.70=/selftest/other-builds\n"
                            f"SLICE_MEMORY_CAP_BY_HOST={slice_items}\nQUIESCE_STOP_COMMAND=\nQUIESCE_STOPPED_CHECK_COMMAND=\nQUIESCE_START_COMMAND=\n"
                            f"QUIESCE_STARTED_CHECK_COMMAND=\nENABLE_ACROSS_MACHINES=0\nENABLE_GPU=0\nNFS_SHARED_ROOTS={os.path.dirname(configuration)}\n")

    # 每个工作进程的峰值固定成编译并行度 4 下一份 4 GiB 的实测（直接调 memory_bound_for 的那几格读这一份）
    records = os.path.join(work, "memory-run-records")
    record_worker_peak(records, 4, f"{1:020d}-1-selftest", 4 * gibibyte)

    def counted(row_count, environment):
        return worker_count_for(row_count, environment, memory=lambda ceiling: memory_bound_for(environment, ceiling, 16, records, listing, root, configuration))

    sixteen = {key: value for key, value in selftest.base_environment.items() if key != TASK_MEMORY_CAP_VARIABLE}
    sixteen.update(GATE_MUTATION_WORKERS="16", GATE_MUTATION_CARGO_JOBS="4")
    peak_text = "每个工作进程的峰值 4.0 GiB（编译并行度 4 下：最新 1 份实测里最大的）"
    write_memory_max(str(64 * gibibyte) + "\n")
    write_slice_configuration("127.0.0.1=50G 192.0.2.70=8G")
    served = dict(sixteen, **{TASK_MEMORY_CAP_VARIABLE: str(20 * gibibyte)})
    count, head = counted(20, served)
    selftest.check("服务交的内存上限 20 GiB、每个工作进程的峰值 4 GiB（编译并行度 4 下的实测）、线程数 16、20 条：同时开 5 个工作进程，头一行写上限、每个工作进程的峰值、个数与合计，"
                   "卡在「内存上限」（服务交的变量先于多机配置的 50G；假 memory.max 的 64 GiB 不比它小、不收窄，头一行写各档读到的值）",
                   count == 5 and "卡在「内存上限」这一项" in head and "内存上限 20.0 GiB（服务交的 TASK_SERVER_MEMORY_CAP_BYTES=" in head
                   and peak_text in head and "= 同时开 5 个工作进程（5 × 4.0 GiB = 20.0 GiB，不超过上限）" in head
                   and f"scope 的 memory.max={64 * gibibyte}（64.0 GiB）" in head and "多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台 127.0.0.1 那一项=50G（50.0 GiB）" in head
                   and "不收窄" in head, f"{count}：{head}")
    sliced = dict(sixteen)
    write_slice_configuration("'127.0.0.1=24G 192.0.2.70=48G'")
    count, head = counted(20, sliced)
    selftest.check("没有服务交的变量、多机配置 SLICE_MEMORY_CAP_BY_HOST 这一台那一项 24G、另一台那一项 48G（整值带引号）、假 memory.max 64 GiB 摆在旁边（不比它小、不收窄）："
                   "取这一台那一项，24 GiB ÷ 4 GiB 开 6 个，头一行写多机配置的路径与这一台那一项",
                   count == 6 and f"多机配置 {configuration} 的 SLICE_MEMORY_CAP_BY_HOST 里这一台 127.0.0.1 那一项=24G" in head and "卡在「内存上限」这一项" in head, f"{count}：{head}")
    write_slice_configuration("127.0.0.1=lots 192.0.2.70=8G")
    write_memory_max(str(8 * gibibyte) + "\n")
    count, head = counted(20, sliced)
    selftest.check("多机配置 SLICE_MEMORY_CAP_BY_HOST 这一台那一项写成 lots：判法整份判不过，头一行点名取不到这一台那一项与「lots」、改读 memory.max（8 GiB 开 2 个）",
                   count == 2 and "SLICE_MEMORY_CAP_BY_HOST 取不到这一台那一项：认不出这一台" in head and "「lots」" in head, f"{count}：{head}")
    write_configuration("ENABLE_GPU=1\n")
    write_memory_max(str(200 * gibibyte) + "\n")   # 比下面三格的服务交的变量都大：memory.max 不收窄它们
    roomy = dict(sixteen, **{TASK_MEMORY_CAP_VARIABLE: str(100 * gibibyte)})
    count, head = counted(20, roomy)
    selftest.check("内存上限 100 GiB 放得下 25 个、线程数 16 更小：开 16 个，卡在 GATE_MUTATION_WORKERS，头一行仍写内存那一项",
                   count == 16 and "卡在「GATE_MUTATION_WORKERS」这一项" in head and "= 同时开 25 个工作进程" in head, f"{count}：{head}")
    tiny = dict(sixteen, **{TASK_MEMORY_CAP_VARIABLE: str(gibibyte)})
    count, head = counted(20, tiny)
    selftest.check("内存上限 1 GiB 一个都放不下（每个工作进程的峰值 4 GiB）：按 1 个开，头一行写「那一项不足 1」",
                   count == 1 and "那一项不足 1，按 1 个开" in head and "= 同时开 0 个工作进程（1 个就要 4.0 GiB，超了上限）" in head, f"{count}：{head}")
    scope_only = dict(sixteen)
    write_memory_max(str(8 * gibibyte) + "\n")
    count, head = counted(20, scope_only)
    selftest.check("没有服务交的变量、自己 scope 的 memory.max 是 8 GiB：开 2 个，头一行写 memory.max 的路径（假 /proc/self/cgroup 里 cgroup v2 的那一行接假 /sys/fs/cgroup）",
                   count == 2 and "自己 scope 的 " + memory_max in head and "卡在「内存上限」这一项" in head, f"{count}：{head}")
    write_memory_max("max\n")
    count, head = counted(20, scope_only)
    selftest.check("自己 scope 的 memory.max 是 max（不限）：这一项不收，照线程数开 16 个，头一行写「是 max」",
                   count == 16 and "没有内存上限这一项" in head and "是 max（不限）" in head and "卡在「GATE_MUTATION_WORKERS」这一项" in head, f"{count}：{head}")
    unreadable = memory_bound_for(scope_only, 16, 16, records, os.path.join(work, "memory-cgroup-listing-absent"), root)
    selftest.check("读不到 /proc/self/cgroup：没有内存上限这一项（交 None 与原因），不当成 0 个也不当成放开之后再算",
                   unreadable[0] is None and "读不到" in unreadable[1], f"{unreadable}")
    write_memory_max(str(8 * gibibyte) + "\n")
    garbled = dict(sixteen, **{TASK_MEMORY_CAP_VARIABLE: "abc"})
    count, head = counted(20, garbled)
    selftest.check("TASK_SERVER_MEMORY_CAP_BYTES 写成 abc：头一行点名写法不对、改读 memory.max（8 GiB 开 2 个）",
                   count == 2 and "TASK_SERVER_MEMORY_CAP_BYTES 写成「abc」" in head, f"{count}：{head}")
    # 自己 scope 的 memory.max 是硬顶（scope 真上限只有 20736M，按多机配置的 50G 会开 12 个工作进程，顶满被杀）
    write_memory_max(str(20 * gibibyte) + "\n")
    write_slice_configuration("127.0.0.1=50G 192.0.2.70=8G")
    count, head = counted(20, sliced)
    selftest.check("多机配置 SLICE_MEMORY_CAP_BY_HOST 这一台那一项 50G（没有服务交的变量）、自己 scope 的 memory.max 20 GiB：memory.max 是硬顶，取两者小的 20 GiB，开 5 个（不是 12 个），"
                   "头一行写各档读到的值、取 memory.max 与多机配置的 50G 不取",
                   count == 5 and "内存上限 20.0 GiB（" in head and f"自己 scope 的 {memory_max}（硬顶：超了整组被杀）" in head
                   and f"比它大的 多机配置 {configuration} 的 SLICE_MEMORY_CAP_BY_HOST 里这一台 127.0.0.1 那一项=50G（50.0 GiB）不取" in head
                   and (f"各档读到：{TASK_MEMORY_CAP_VARIABLE} 没给、多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台 127.0.0.1 那一项=50G（50.0 GiB）、"
                        f"scope 的 memory.max={20 * gibibyte}（20.0 GiB）") in head
                   and "= 同时开 5 个工作进程" in head, f"{count}：{head}")
    served_over = dict(sixteen, **{TASK_MEMORY_CAP_VARIABLE: str(50 * gibibyte)})
    count, head = counted(20, served_over)
    selftest.check("服务交的变量 50 GiB、自己 scope 的 memory.max 20 GiB：取两者小的 20 GiB，开 5 个，头一行写服务交的 50 GiB 不取",
                   count == 5 and "内存上限 20.0 GiB（" in head and f"比它大的 服务交的 {TASK_MEMORY_CAP_VARIABLE}={50 * gibibyte}（50.0 GiB）不取" in head
                   and "= 同时开 5 个工作进程" in head, f"{count}：{head}")
    write_memory_max("max\n")
    count, head = counted(20, served_over)
    selftest.check("服务交的变量 50 GiB、自己 scope 的 memory.max 是 max（不限）：没有硬顶可比，照服务交的 50 GiB 开 12 个，头一行写 memory.max 没有硬顶与原因",
                   count == 12 and "内存上限 50.0 GiB（服务交的 TASK_SERVER_MEMORY_CAP_BYTES=" in head and "scope 的 memory.max 没有硬顶（" in head
                   and f"{memory_max} 是 max（不限）" in head and "= 同时开 12 个工作进程" in head, f"{count}：{head}")
    unreadable_served = memory_bound_for(served_over, 16, 16, records, os.path.join(work, "memory-cgroup-listing-absent"), root)
    selftest.check("服务交的变量 50 GiB、/proc/self/cgroup 读不到：没有硬顶可比，照服务交的 50 GiB 放得下 12 个，头一行写读不到",
                   unreadable_served[0] == 12 and "内存上限 50.0 GiB（" in unreadable_served[1] and "scope 的 memory.max 没有硬顶（读不到" in unreadable_served[1], f"{unreadable_served}")
    write_memory_max(str(8 * gibibyte) + "\n")
    repository = os.path.join(work, "memory-workers")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    target = os.path.join(work, "memory-workers-target")
    # run_stage 读的运行记录目录是共享根底下按这个小仓的命名空间分的那一层（run_records_directory），峰值记录照它放
    stage_changes = dict(GATE_MUTATION_RUN_RECORDS_DIRECTORY=os.path.join(work, "memory-workers-run-records"), GATE_MUTATION_CARGO_JOBS="4")
    record_worker_peak(run_records_directory(repository, dict(selftest.base_environment, **stage_changes)), 4, f"{1:020d}-1-selftest", 4 * gibibyte)
    exit_code, _stdout, stderr, _ = selftest.run(
        repository, GATE_MUTATION_WORKERS="16", GATE_MUTATION_TARGET_DIR=target, **stage_changes,
        **{TASK_MEMORY_CAP_VARIABLE: str(8 * gibibyte)})
    selftest.check("内存上限交给 run_stage：开的工作进程数按内存收（8 GiB ÷ 4 GiB 开 2 个，不是条数 4），头一行与收尾一行都写上限、每个工作进程的峰值、个数，卡在「内存上限」",
                   exit_code == 0 and "这一轮开 2 个工作进程" in stderr and "卡在「内存上限」这一项" in stderr and stderr.count("= 同时开 2 个工作进程") >= 2
                   and "内存上限 8.0 GiB" in stderr and peak_text in stderr, f"退 {exit_code}；{stderr[:1200]}")
    # 第二趟换一棵树（库是新的一份，要现跑才开工作进程）：运行记录目录按库的命名空间分，峰值记录也放一份给它
    second_repository = selftest_new_library_copy(work, repository, "memory-workers-second")
    record_worker_peak(run_records_directory(second_repository, dict(selftest.base_environment, **stage_changes)), 4, f"{1:020d}-1-selftest", 4 * gibibyte)
    exit_code, _stdout, stderr, _ = selftest.run(
        second_repository, GATE_MUTATION_WORKERS="16", GATE_MUTATION_TARGET_DIR=target, **stage_changes,
        GATE_MUTATION_CGROUP_LISTING=listing, GATE_MUTATION_CGROUP_ROOT=root, **{TASK_MEMORY_CAP_VARIABLE: str(50 * gibibyte)})
    selftest.check("run_stage 整趟：服务交的变量 50 GiB、自己 scope 的 memory.max（经 GATE_MUTATION_CGROUP_LISTING / GATE_MUTATION_CGROUP_ROOT 指到假的）8 GiB："
                   "开的工作进程数按 8 GiB 收（2 个，不是按 50 GiB 的 12 与条数 4），开跑行写 memory.max 是硬顶",
                   exit_code == 0 and "这一轮开 2 个工作进程" in stderr and "内存上限 8.0 GiB（" in stderr and "（硬顶：超了整组被杀）" in stderr
                   and "卡在「内存上限」这一项" in stderr, f"退 {exit_code}；{stderr[:1500]}")
    # ⑧ 撞内存上限：上限 2 GiB 装不下一个 4 GiB 的工作进程，派活之前退 250（统一任务服务错误类型表里 memory-cap-hit 那一行），不开 1 个硬跑
    short_repository = selftest_new_library_copy(work, repository, "memory-workers-short")
    record_worker_peak(run_records_directory(short_repository, dict(selftest.base_environment, **stage_changes)), 4, f"{1:020d}-1-selftest", 4 * gibibyte)
    exit_code, stdout, stderr, invocations = selftest.run(
        short_repository, GATE_MUTATION_WORKERS="16", GATE_MUTATION_TARGET_DIR=target, **stage_changes, **{TASK_MEMORY_CAP_VARIABLE: str(2 * gibibyte)})
    selftest.check("内存上限装不下一个工作进程的峰值：run_stage 按撞内存上限退 250（上限 2 GiB、每个工作进程的峰值 4 GiB），红句写上限、峰值与 memory-cap-hit，"
                   "假 cargo 基线与变异一次都没起",
                   exit_code == 250 and "内存上限装不下一个工作进程的峰值" in stdout and "按撞内存上限退出（退 250" in stdout and "memory-cap-hit" in stdout
                   and "内存上限 2.0 GiB" in stdout and peak_text in stdout and not plain_invocations(invocations),
                   f"退 {exit_code}；假 cargo 起了 {len(plain_invocations(invocations))} 次基线与变异；stdout「{stdout[-800:]}」；stderr「{stderr[-600:]}」")


def selftest_worker_peak_cells(selftest, work):
    """每个工作进程的峰值：量、记、读、除（文件头「内存上限」的「每个工作进程的峰值」）。
    ① 量一棵进程树：本进程起一个 sh、sh 再起一个占着 256 MiB 的 python（孙进程），process_tree_usage(本进程) 比起之前至少多 240 MiB（弄坏开关 worker-tree-self-only 下判错）；
    ② 记下的峰值按编译并行度分开、读得回来：编译并行度 4 记 5 GiB、6 GiB，8 记 12 GiB，读回 {4: (6 GiB, 2 份), 8: (12 GiB, 1 份)}（worker-peaks-unkeyed-by-jobs 下判错）；
    ③ 每个编译并行度只看最新的 WORKER_PEAKS_RECENT 份：编译并行度 3 记多两份，最旧的两份 20 GiB、其余 1 GiB，读回 1 GiB，读完只剩那么多份（worker-peaks-window-unbounded 下判错）；
    ④ 固定编译并行度（GATE_MUTATION_CARGO_JOBS=4）：上限 20 GiB ÷ 编译并行度 4 下的 6 GiB = 3 个，3 × 6 GiB 不超过上限（worker-peaks-unkeyed-by-jobs 下判错）；
    ⑤ 编译并行度跟着进程数走：核数 16、天花板 8，编译并行度 2、4、8、16 各实测 4、7、9、14 GiB，上限 20 GiB：开 2 个（编译并行度 8、每个 9 GiB、合计 18 GiB），
      3 个时编译并行度 5 借 8 的 9 GiB、合计 27 GiB 超了（jobs-feedback-ignored、worker-peaks-unkeyed-by-jobs 下判错）；
    ⑥ 从没量过：上限 20 GiB，保守值就是上限本身，开 1 个，头一行写「从没量过」「保守值」与峰值记录在哪（unmeasured-worker-peak-guessed 下判错）；
    ⑦ 只有更低编译并行度的实测：GATE_MUTATION_CARGO_JOBS=4、只有编译并行度 2 的 1 GiB，不往低处借，取保守值开 1 个（同上）；
    ⑧ 峰值记录里有一份内容不是正整数、⑨ worker-peaks 是一份文件不是目录：整份读不了，不退回写死的默认值，取保守值开 1 个，头一行写「读不了」与那一份（unreadable-worker-peaks-guessed 下判错）；
    ⑩ 交给 run_stage：GATE_MUTATION_CARGO_JOBS=3、运行记录里编译并行度 3 一份 3 GiB，上限 6 GiB、4 条：开 2 个工作进程，头一行与收尾一行都写个数；
    ⑪ 交给 run_stage 的工作进程各写一份峰值记录：那一趟之后编译并行度 3 下多出至少 2 份（一个工作进程一份），每份是不小于 1 MiB 的正整数（worker-peaks-unrecorded 下判错）；
    ⑫ 样本根上的工作进程不写峰值记录：不在临时服务底下时别处的根不写、本模块所在的仓与设了 GATE_STAGED_FROM 的写，在临时服务底下写（sample-root-peaks-recorded 下判错）。
    弄坏开关 memory-cap-ignored 下经 memory_bound_for 的 ④–⑩ 七格判错。"""
    gibibyte = 1 << 30
    environment = {key: value for key, value in selftest.base_environment.items() if key not in (TASK_MEMORY_CAP_VARIABLE, "GATE_MUTATION_CARGO_JOBS")}

    def record(records, jobs, index, value):
        record_worker_peak(records, jobs, f"{index:020d}-1-selftest", value)

    holder_code = "import sys\nblock = b'\\x01' * (256 << 20)\nsys.stdout.write('ready\\n')\nsys.stdout.flush()\nsys.stdin.read()\n"
    before = process_tree_usage(os.getpid(), environment)
    # sh 后面还有一条命令就不 exec，python 是本进程的孙进程；读到标准输入结束就退
    holder = subprocess.Popen(["sh", "-c", '"$0" -c "$1"; exit 0', sys.executable, holder_code], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
    try:
        ready = holder.stdout.readline().strip()
        during = process_tree_usage(os.getpid(), environment)
    finally:
        holder.stdin.close()
        try:
            holder.wait(timeout=60)
        except subprocess.TimeoutExpired:
            holder.kill()   # 只对自己起的这一个发信号；孙进程读到标准输入结束自己退
            holder.wait()
        holder.stdout.close()
    grown = (during or 0) - (before or 0)
    selftest.check("量一棵进程树：本进程起的 sh 再起一个占着 256 MiB 的 python（孙进程），process_tree_usage(本进程) 比起之前多出至少 240 MiB",
                   ready == "ready" and before is not None and during is not None and grown >= 240 << 20, f"ready={ready!r}，之前 {before}、之后 {during}")

    keyed = os.path.join(work, "worker-peaks-keyed")
    record(keyed, 4, 1, 5 * gibibyte)
    record(keyed, 4, 2, 6 * gibibyte)
    record(keyed, 8, 3, 12 * gibibyte)
    peaks, source = measured_worker_peaks(keyed, environment)
    selftest.check("记下的峰值按编译并行度分开、读得回来：编译并行度 4 记了 5 GiB 与 6 GiB、8 记了 12 GiB，读回编译并行度 4 最新 2 份里最大 6 GiB、8 最新 1 份里最大 12 GiB",
                   peaks == {4: (6 * gibibyte, 2), 8: (12 * gibibyte, 1)} and "编译并行度 4 最新 2 份里最大 6.0 GiB" in source, f"{peaks}：{source}")
    windowed = os.path.join(work, "worker-peaks-window")
    for index in range(1, WORKER_PEAKS_RECENT + 3):
        record(windowed, 3, index, 20 * gibibyte if index <= 2 else gibibyte)
    peaks, source = measured_worker_peaks(windowed, environment)
    left = len(os.listdir(os.path.join(worker_peaks_directory(windowed), "3")))
    selftest.check(f"每个编译并行度只看最新的 {WORKER_PEAKS_RECENT} 份：编译并行度 3 记了 {WORKER_PEAKS_RECENT + 2} 份，最旧的两份 20 GiB、其余 1 GiB，"
                   f"读回 1 GiB（{WORKER_PEAKS_RECENT} 份），读完只剩 {WORKER_PEAKS_RECENT} 份",
                   peaks == {3: (gibibyte, WORKER_PEAKS_RECENT)} and left == WORKER_PEAKS_RECENT, f"{peaks}；剩 {left} 份")

    roof = dict(environment, **{TASK_MEMORY_CAP_VARIABLE: str(20 * gibibyte)})
    fixed = dict(roof, GATE_MUTATION_CARGO_JOBS="4")
    count, text = memory_bound_for(fixed, 16, 16, keyed)
    selftest.check("固定编译并行度：GATE_MUTATION_CARGO_JOBS=4、编译并行度 4 最大 6 GiB（8 那一份 12 GiB 不算），上限 20 GiB，天花板 16：开 ⌊20 ÷ 6⌋ = 3 个，3 × 6 GiB = 18 GiB 不超过上限",
                   count == 3 and count * 6 * gibibyte <= 20 * gibibyte and "每个工作进程的峰值 6.0 GiB（编译并行度 4 下：最新 2 份实测里最大的）" in text
                   and "= 同时开 3 个工作进程（3 × 6.0 GiB = 18.0 GiB，不超过上限）" in text, f"{count}：{text}")
    following = os.path.join(work, "worker-peaks-following")
    for index, (jobs, size) in enumerate(((2, 4), (4, 7), (8, 9), (16, 14)), 1):
        record(following, jobs, index, size * gibibyte)
    count, text = memory_bound_for(roof, 8, 16, following)
    selftest.check("编译并行度跟着进程数走：核数 16、天花板 8，编译并行度 2、4、8、16 各实测 4、7、9、14 GiB，上限 20 GiB：开 2 个（编译并行度 8、每个 9 GiB，2 × 9 GiB = 18 GiB 不超过上限），"
                   "3 个时编译并行度 5 借 8 的 9 GiB、合计 27 GiB 超了；不照天花板 8 个时的编译并行度 2 去除（那样开 5 个，编译并行度 3 借 4 的 7 GiB、合计 35 GiB）",
                   count == 2 and count * 9 * gibibyte <= 20 * gibibyte and "每个工作进程的峰值 9.0 GiB（编译并行度 8 下：最新 1 份实测里最大的）" in text
                   and "= 同时开 2 个工作进程（2 × 9.0 GiB = 18.0 GiB，不超过上限）" in text
                   and "再多 1 个：编译并行度 5 下每个 9.0 GiB（没有实测，借编译并行度 8 最新 1 份实测里最大的" in text and "3 × 9.0 GiB = 27.0 GiB，超了上限" in text,
                   f"{count}：{text}")
    never = os.path.join(work, "worker-peaks-never")
    count, text = memory_bound_for(roof, 16, 16, never)
    selftest.check("从没量过：峰值记录还没有，上限 20 GiB：每个工作进程的峰值取保守值（上限本身），开 1 个，头一行写来源（从没量过、保守值、峰值记录在哪）",
                   count == 1 and "= 同时开 1 个工作进程（1 × 20.0 GiB = 20.0 GiB，不超过上限）" in text and "从没量过，取保守值" in text
                   and f"峰值记录 {worker_peaks_directory(never)} 还没有（从没量过）" in text, f"{count}：{text}")
    lower = os.path.join(work, "worker-peaks-lower")
    record(lower, 2, 1, gibibyte)
    count, text = memory_bound_for(fixed, 16, 16, lower)
    selftest.check("只有更低编译并行度的实测：GATE_MUTATION_CARGO_JOBS=4、只有编译并行度 2 的 1 GiB，不往低处借，取保守值开 1 个",
                   count == 1 and "只有更低编译并行度的实测、不借，取保守值" in text, f"{count}：{text}")
    garbled = os.path.join(work, "worker-peaks-garbled")
    selftest_write(os.path.join(worker_peaks_directory(garbled), "4", f"{1:020d}-1-selftest"), "lots\n")
    count, text = memory_bound_for(fixed, 16, 16, garbled)
    selftest.check("峰值记录里有一份内容不是正整数：整份读不了，不退回写死的默认值，取保守值开 1 个，头一行写「读不了」、那一份的路径与「lots」",
                   count == 1 and "峰值记录读不了，取保守值" in text and "写着「lots」，不是一个正整数字节数" in text and f"{1:020d}-1-selftest" in text, f"{count}：{text}")
    flat = os.path.join(work, "worker-peaks-flat")
    selftest_write(worker_peaks_directory(flat), "不是目录\n")
    count, text = memory_bound_for(fixed, 16, 16, flat)
    selftest.check("峰值记录的 worker-peaks 是一份文件、不是目录：整份读不了，取保守值开 1 个，头一行写「读不了」与 NotADirectoryError",
                   count == 1 and "峰值记录读不了，取保守值" in text and "NotADirectoryError" in text, f"{count}：{text}")

    repository = os.path.join(work, "worker-peaks-run-stage")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    # run_stage 读写的运行记录目录是共享根底下按这个小仓的命名空间分的那一层（run_records_directory），预先放的那一份照它放
    stage_changes = dict(GATE_MUTATION_RUN_RECORDS_DIRECTORY=os.path.join(work, "worker-peaks-run-stage-records"), GATE_MUTATION_CARGO_JOBS="3")
    stage_records = run_records_directory(repository, dict(selftest.base_environment, **stage_changes))
    record(stage_records, 3, 1, 3 * gibibyte)
    exit_code, _stdout, stderr, _ = selftest.run(
        repository, GATE_MUTATION_WORKERS="16", GATE_MUTATION_TARGET_DIR=os.path.join(work, "worker-peaks-run-stage-target"),
        **stage_changes, **{TASK_MEMORY_CAP_VARIABLE: str(6 * gibibyte)})
    selftest.check("交给 run_stage：GATE_MUTATION_CARGO_JOBS=3、运行记录里编译并行度 3 一份 3 GiB，上限 6 GiB、4 条：开 2 个工作进程，头一行与收尾一行都写个数，卡在「内存上限」",
                   exit_code == 0 and "这一轮开 2 个工作进程" in stderr and stderr.count("= 同时开 2 个工作进程") >= 2 and "卡在「内存上限」这一项" in stderr
                   and "每个工作进程的峰值 3.0 GiB（编译并行度 3 下：最新 1 份实测里最大的）" in stderr, f"退 {exit_code}；{stderr[:1500]}")
    recorded = os.path.join(worker_peaks_directory(stage_records), "3")
    written = sorted(name for name in os.listdir(recorded) if name != f"{1:020d}-1-selftest" and not name.startswith(".partial-")) if os.path.isdir(recorded) else []
    values = []
    for name in written:
        with open(os.path.join(recorded, name), encoding="utf-8") as handle:
            text = handle.read()
        values.append(int(text) if re.fullmatch(r"[1-9][0-9]*\n", text) else text)
    selftest.check("交给 run_stage 的工作进程各写一份峰值记录：那一趟之后编译并行度 3 下多出至少 2 份（一个工作进程一份），每份是不小于 1 MiB 的正整数字节数",
                   len(written) >= 2 and all(isinstance(value, int) and value >= 1 << 20 for value in values), f"多出 {len(written)} 份：{values}")
    sample_root = os.path.join(work, "worker-peaks-sample-root")
    os.makedirs(sample_root, exist_ok=True)
    outside = {key: value for key, value in environment.items() if key not in (mutation_units.Library.client().TEMPORARY_SERVICE_VARIABLE, "GATE_STAGED_FROM")}
    decided = (worker_peaks_recorded_for(sample_root, outside), worker_peaks_recorded_for(STAGE_REPOSITORY, outside),
               worker_peaks_recorded_for(sample_root, dict(outside, GATE_STAGED_FROM=STAGE_REPOSITORY)), worker_peaks_recorded_for(sample_root, environment))
    selftest.check("样本根上的工作进程不写峰值记录：不在临时 prune-server 底下时，被判的是别处的根不写，是本模块所在的仓、或设了 GATE_STAGED_FROM 写；在临时服务底下（自证）写",
                   decided == (False, True, True, True), f"（别处的根、本仓、GATE_STAGED_FROM、临时服务底下）= {decided}")


def selftest_disk_cells(selftest, work):
    """磁盘余量（文件头「磁盘余量」）：每个工作进程最多占两份编译目录（普通、插桩）、各涨到上限，派活前按目标盘余量定工作进程数。
    先直接调 worker_count_for 与 disk_bound_for：GATE_MUTATION_WORKERS 本来给 16 个，假的余量 100G、上限 10G、备用 5G，每个工作进程要 20G、开 2 个以上另加先编一份依赖的 20G，
    放得下 3 个（共 80G），头一行写明卡在磁盘余量；已有的编译目录只算还能涨的（片号 1 的普通编译目录已占 8M，上限 10M、余量 100M、备用 1M：放得下 4 个，没有已有目录是 3 个）。
    再起一趟 run_stage：假的余量 70G、备用 10G，开的工作进程数按余量收成 2 个（条数 4 放得下更多），头一行写明；余量写法不认的整道拒；
    余量设成空（外层没给）时读编译目录所在的盘，头一行写明来源。弄坏开关 disknotchecked（不看磁盘）下 run_stage 那一格与直接调的两格判错。"""
    environment = dict(selftest.base_environment)
    sixteen = dict(environment, GATE_MUTATION_WORKERS="16")
    sized = dict(sixteen, **{DISK_FREE_VARIABLE: "100G", TARGET_DIRECTORY_CAP_VARIABLE: "10G", DISK_RESERVE_VARIABLE: "5G"})
    shared = os.path.join(work, "disk-target")
    count, head = worker_count_for(20, sized, disk=lambda ceiling: disk_bound_for(shared, sized, ceiling, None))
    selftest.check("磁盘余量放不下 16 份：GATE_MUTATION_WORKERS 本来给 16 个，余量 100G（上限 10G、备用 5G、每个工作进程两份共 20G、开 2 个以上另加先编一份依赖的两份 20G）按余量收成 3 个，头一行写明卡在磁盘余量与放得下几个",
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
        repository, GATE_MUTATION_WORKERS="16", GATE_MUTATION_TARGET_DIR=target,
        **{DISK_FREE_VARIABLE: "70G", TARGET_DIRECTORY_CAP_VARIABLE: "10G", DISK_RESERVE_VARIABLE: "10G"})
    selftest.check("磁盘余量交给 run_stage：开的工作进程数按余量收（余量 70G、备用 10G、每个工作进程 20G、先编一份依赖的两份 20G 开 2 个，不是条数 4），头一行写明卡在磁盘余量",
                   exit_code == 0 and "这一轮开 2 个工作进程" in stderr and "卡在「磁盘余量」这一项" in stderr, f"退 {exit_code}；{stderr[:900]}")
    exit_code, stdout, stderr, _ = selftest.run(selftest_new_library_copy(work, repository, "disk-workers-unwritten"), GATE_MUTATION_TARGET_DIR=target,
                                                **{DISK_FREE_VARIABLE: "lots"})
    selftest.check("磁盘余量的写法不认：整道拒、带出路，一条变异都没跑", exit_code == 1 and f"{DISK_FREE_VARIABLE} 写成了「lots」" in stdout + stderr,
                   f"退 {exit_code}；{(stdout + stderr)[:500]}")
    exit_code, _stdout, stderr, _ = selftest.run(selftest_new_library_copy(work, repository, "disk-workers-unset"), GATE_MUTATION_TARGET_DIR=target,
                                                 **{DISK_FREE_VARIABLE: ""})
    selftest.check("外层没给磁盘余量时读编译目录所在的盘：头一行写来源，照常跑完", exit_code == 0 and "编译目录所在的盘此刻可用" in stderr, f"退 {exit_code}；{stderr[:700]}")


def selftest_target_cap_cells(selftest, work):
    """编译目录上限（文件头「编译目录上限」）：一份编译目录占盘超过上限，在两格之间清掉它底下的增量产物（<目录>/<档>/incremental、<目录>/<三元组>/<档>/incremental），
    依赖与测试二进制不动；没超线不动；清完仍超线的照实报。先直接调 trim_target_directory：上限 2 MiB，增量 3+1 MiB、依赖 1 MiB，清掉两处增量、依赖还在、清后回读不到 2 MiB；
    没超线不动；清完仍超线（依赖 3 MiB）只清增量、清后仍超。再起两趟 run_stage：假 cargo 每次编（基线与变异直接跑测试的那一趟、覆盖的插桩编译）往增量产物里写 64 KiB，上限 1T 时跑完两份编译目录（普通、插桩）的增量产物都留着（对照），
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
    roomy_exit, _stdout, _stderr, _ = selftest.run(repository, GATE_MUTATION_TARGET_DIR=target, FAKE_CARGO_INCREMENTAL_BYTES="65536",
                                                   **{TARGET_DIRECTORY_CAP_VARIABLE: "1T"})
    roomy_left = [path for path in incremental_paths if os.path.exists(path)]
    shutil.rmtree(target, ignore_errors=True)
    shutil.rmtree(target + COVERAGE_TARGET_SUFFIX, ignore_errors=True)
    # 清的那一趟另起一棵仓（判定库在仓的 git 目录里，沿用上一趟的覆盖记录就不会再编插桩那一份）
    trimmed_repository = os.path.join(work, "cap-run-trimmed")
    build_selftest_repository(trimmed_repository, SELFTEST_TABLE_ROWS)
    # 上限 40K：假 cargo 每次编写进增量产物的 64K 一来就超线、清掉增量之后剩下的（目录与 deps 里一两份小文件，十几 K）在线下，所以照常跑完、不停派
    exit_code, _stdout, stderr, _ = selftest.run(trimmed_repository, GATE_MUTATION_TARGET_DIR=target, FAKE_CARGO_INCREMENTAL_BYTES="65536",
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
    exit_code, stdout, stderr, _ = selftest.run(repository, GATE_MUTATION_TARGET_DIR=target, **{TARGET_DIRECTORY_CAP_VARIABLE: "1K"})
    selftest.check("编译目录清完仍超上限（1K）：头一格判完就停派、整道判红，现判的变异 0 条，拒绝说明点名停派的那份编译目录",
                   exit_code == 1 and "仍超上限 GATE_MUTATION_TARGET_DIR_CAP，停派" in stdout and "现判的变异判完 0 条" in stdout and f"编译目录 {target}：" in stdout,
                   f"退 {exit_code}；{(stdout + stderr)[-900:]}")


def selftest_disk_peak_cells(selftest, work):
    """磁盘预算按实测峰值算（文件头「磁盘余量」）：每格判完记一次占盘，下一趟每份编译目录的涨幅取近几次的峰值，上限只当清理线。
    先直接调：记下普通 9G、1G、2G、3G 与插桩 4G，取近 3 次是普通 3G（9G 那次出了窗口）、插桩 4G；余量 40G、上限 10G、备用 5G，
    峰值普通 2G、插桩 3G 时每个工作进程 5G、放得下 6 个，插桩没有读数的那一种按上限 10G、只放得下 1 个（弄坏开关 diskbudgetbycap 下一律按上限、头一格也是 1 个，那一格判错）。
    再起一趟 run_stage：跑完运行记录目录（run_records_directory，不是判定库）下记了普通编译目录的占盘。"""
    records_directory = os.path.join(work, "peak-run-records")
    os.makedirs(records_directory)
    for coverage, size in ((False, 9 << 30), (False, 1 << 30), (True, 4 << 30), (False, 2 << 30), (False, 3 << 30)):
        record_target_occupancy(records_directory, coverage, size)
    peaks = measured_target_peaks(records_directory, 3)
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
    exit_code, _stdout, stderr, _ = selftest.run(repository, GATE_MUTATION_TARGET_DIR=os.path.join(work, "peak-run-target"))
    run_records = run_records_directory(repository, dict(selftest.base_environment))
    recorded = measured_target_peaks(run_records, 20)
    library_files = [name for _walked, _directories, names in os.walk(library_of(repository)) for name in names]
    selftest.check("run_stage 每格判完记一次编译目录占盘：跑完运行记录目录下有普通编译目录的读数，判定库目录本地一个文件都没有",
                   exit_code == 0 and recorded["普通"] is not None and not library_files,
                   f"退 {exit_code}；{run_records} 的读数 {recorded}；库目录本地的文件 {library_files[:3]}；{stderr[-400:]}")


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


SELFTEST_REACHABILITY_SOURCE = '''// static-check 可达性那几格的样本 crate：选路函数 judge_not_run_without_the_gpu_feature；CPU 判器入口 cpu_judge_entry 只在 run_enumeration 的 CPU 那一臂里调，
// 宽松的恢复 lax_base_recovery 在 GPU 路上也调（按单元停）；never_called 谁都不调
pub fn judge_not_run_without_the_gpu_feature(configured: bool, compiled: bool) -> bool { configured && !compiled }

pub fn cpu_only_check(value: u32) -> bool { value + 1 > 0 }

pub fn cpu_judge_entry(value: u32) -> bool { cpu_only_check(value) }

pub fn gpu_judge(value: u32) -> bool { value < 1000 }

pub fn recover_value(value: u32) -> u32 { value + 3 }

pub fn lax_base_recovery(value: u32) -> bool { recover_value(value) > 0 }

pub fn run_enumeration(value: u32) -> bool {
    let _ = lax_base_recovery(value);
    if judge_not_run_without_the_gpu_feature(true, true) { gpu_judge(value) } else { cpu_judge_entry(value) }
}

pub fn plain() -> u32 { 2 + 2 }

pub fn never_called(value: u32) -> u32 { value * 7 }

#[cfg(test)]
mod tests {
    fn configuration_on_the_cpu() -> &'static str { "ENABLE_GPU=0\\n" }

    fn hand_out_letting_the_gpu_refuse() -> bool { true }

    #[test]
    fn enumeration_judges() { assert!(super::run_enumeration(1)); }
    #[test]
    fn enumeration_with_an_observer() { assert!(super::run_enumeration(1)); assert!(super::cpu_judge_entry(1)); }
    #[test]
    fn enumeration_on_the_cpu() { assert!(!configuration_on_the_cpu().is_empty()); assert!(super::run_enumeration(1)); }
    #[test]
    fn enumeration_on_refused_tables() { assert!(hand_out_letting_the_gpu_refuse()); assert!(super::run_enumeration(1)); }
    #[test]
    fn plain_sum() { assert_eq!(super::plain(), 4); }
    #[test]
    fn source_names_every_function() { assert!(include_str!("lib.rs").contains("fn never_called")); }
}
'''
SELFTEST_REACHABILITY_ROWS = [
    "cpu-check-blind\tcrates/tiny/src/lib.rs\t{ value + 1 > 0 }\t{ value + 1 > 5 }\t-p tiny --lib -- enumeration_judges\tenumeration_judges",
    "cpu-check-observed\tcrates/tiny/src/lib.rs\t{ value + 1 > 0 }\t{ value + 1 > 5 }\t-p tiny --lib -- enumeration_with_an_observer\tenumeration_with_an_observer",
    "cpu-check-on-the-cpu-configuration\tcrates/tiny/src/lib.rs\t{ value + 1 > 0 }\t{ value + 1 > 5 }\t-p tiny --lib -- enumeration_on_the_cpu\tenumeration_on_the_cpu",
    "cpu-check-refused-tables\tcrates/tiny/src/lib.rs\t{ value + 1 > 0 }\t{ value + 1 > 5 }\t-p tiny --lib -- enumeration_on_refused_tables\tenumeration_on_refused_tables",
    "never-called-seven\tcrates/tiny/src/lib.rs\t{ value * 7 }\t{ value * 8 }\t-p tiny --lib -- plain_sum\tplain_sum",
    "recovery-three\tcrates/tiny/src/lib.rs\t{ value + 3 }\t{ value + 4 }\t-p tiny --lib -- enumeration_judges\tenumeration_judges",
    "plain-four\tcrates/tiny/src/lib.rs\t{ 2 + 2 }\t{ 2 + 3 }\t-p tiny --lib -- plain_sum\tplain_sum",
    "never-called-read-as-text\tcrates/tiny/src/lib.rs\t{ value * 7 }\t{ value * 9 }\t-p tiny --lib -- source_names_every_function\tsource_names_every_function",
]
SELFTEST_REACHABILITY_FILE = "crates/tiny/src/lib.rs"
# 样本 crate 的截断表：run_enumeration 连到 cpu_judge_entry 的那一条边（按边停）与 lax_base_recovery（按单元停）
SELFTEST_REACHABILITY_STOPS = (((SELFTEST_REACHABILITY_FILE, "run_enumeration"), (SELFTEST_REACHABILITY_FILE, "cpu_judge_entry")),
                               (None, (SELFTEST_REACHABILITY_FILE, "lax_base_recovery")))


def selftest_reachability_static_check(root, stops, environment_changes=None):
    """在样本仓上跑一趟 static-check 可达性：选路表、截断表、豁免表换成样本 crate 里的那几处（REACHABILITY_GPU_ROUTE_STOPS 换成 stops），跑完换回。
    environment_changes 临时并进 os.environ（多机配置换成 GPU 关那一份这类）。→ (退出码, stdout, {变异名: 哪一路}, {变异名: 豁免的种类})。"""
    saved = (GPU_ROUTE_UNITS[0], REACHABILITY_GPU_ROUTE_STOPS[0], REACHABILITY_CPU_FALLBACK_ROOTS[0])
    saved_environment = {name: os.environ.get(name) for name in (environment_changes or {})}
    try:
        GPU_ROUTE_UNITS[0] = (("tiny", SELFTEST_REACHABILITY_FILE, "judge_not_run_without_the_gpu_feature"),)
        REACHABILITY_GPU_ROUTE_STOPS[0] = stops
        REACHABILITY_CPU_FALLBACK_ROOTS[0] = ((SELFTEST_REACHABILITY_FILE, "hand_out_letting_the_gpu_refuse"),)
        os.environ.update(environment_changes or {})
        exit_code, output = run_command_capturing(command_static_check, [root])
    finally:
        GPU_ROUTE_UNITS[0], REACHABILITY_GPU_ROUTE_STOPS[0], REACHABILITY_CPU_FALLBACK_ROOTS[0] = saved
        for name, value in saved_environment.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value
    routes = {fields[2]: fields[3] for fields in (line.split("\t") for line in output.splitlines()) if fields[0] == "UNREACHABLE" and len(fields) > 3}
    exempted = {fields[2]: fields[3] for fields in (line.split("\t") for line in output.splitlines()) if fields[0] == "REACHABILITY_EXEMPT" and len(fields) > 3}
    return exit_code, output, routes, exempted


def selftest_reachability_cells(selftest, work):
    """static-check 可达性（文件头「点名测试够不着被改坏的函数」）在样本 crate 上：多机配置 ENABLE_GPU=1（自证的那一份），截断表 =
    run_enumeration 连到 cpu_judge_entry 的那一条边（按边停）与 lax_base_recovery（按单元停），豁免表 = hand_out_letting_the_gpu_refuse。
    每格配一个弄坏开关（文件头弄坏开关那一节 reachability-*）。"""
    root = os.path.join(work, "reachability")
    build_selftest_repository(root, SELFTEST_REACHABILITY_ROWS, source=SELFTEST_REACHABILITY_SOURCE)
    stops = SELFTEST_REACHABILITY_STOPS
    exit_code, output, routes, exempted = selftest_reachability_static_check(root, stops)
    gpu_off = os.path.join(work, "reachability-gpu-off", "multi-host.env")
    selftest_write(gpu_off, "# 自证用的多机配置：GPU 关\nENABLE_GPU=0\n")
    off_code, off_output, off_routes, _off_exempted = selftest_reachability_static_check(root, stops, {"GATE_MUTATION_MULTI_HOST_CONFIG": gpu_off})
    selftest.check("static-check 可达性的 gpu-route：开 GPU 时点名测试走选路、被改坏的函数只在 CPU 判器入口（按边停）或宽松的恢复（按单元停）后面的两行报 gpu-route、退 1；"
                   "多机配置换成 GPU 关时这两行不报",
                   exit_code == 1 and routes.get("cpu-check-blind") == "gpu-route" and routes.get("recovery-three") == "gpu-route"
                   and "cpu-check-blind" not in off_routes and "recovery-three" not in off_routes and off_code in (0, 1),
                   f"开：退 {exit_code}、{routes}；关：退 {off_code}、{off_routes}；{output[-800:]}{off_output[-400:]}")
    selftest.check("static-check 可达性的 closure：被改坏的函数 never_called 不在点名测试 plain_sum 的闭包里（开不开 GPU 都调不到）的那一行报 closure",
                   routes.get("never-called-seven") == "closure", f"{routes}；{output[-600:]}")
    selftest.check("static-check 可达性的 cpu-configuration 豁免：测试自己写 ENABLE_GPU=0 配置的那一行不报 UNREACHABLE、打 REACHABILITY_EXEMPT cpu-configuration",
                   "cpu-check-on-the-cpu-configuration" not in routes and exempted.get("cpu-check-on-the-cpu-configuration") == "cpu-configuration",
                   f"{routes}；豁免 {exempted}")
    selftest.check("static-check 可达性放行够得着的：测试自己直接调 CPU 判器入口（不经选路那一条边）的那一行、不经选路的 plain-four 都不报",
                   "cpu-check-observed" not in routes and "plain-four" not in routes and "cpu-check-observed" not in exempted, f"{routes}；豁免 {exempted}")
    selftest.check("static-check 可达性的 gpu-tables-refused 豁免：截过的闭包里有 GPU 许建不出判表、回到 CPU 判器的那个函数的那一行不报、打 REACHABILITY_EXEMPT gpu-tables-refused",
                   "cpu-check-refused-tables" not in routes and exempted.get("cpu-check-refused-tables") == "gpu-tables-refused", f"{routes}；豁免 {exempted}")
    selftest.check("static-check 可达性放行读源码文本的：点名测试 include_str! 读被改坏的那份文件（闭包里有它的 data-file 单元）、不调被改坏的函数 never_called 的那一行不报",
                   "never-called-read-as-text" not in routes and "never-called-read-as-text" not in exempted, f"{routes}；豁免 {exempted}；{output[-600:]}")
    checked = next((line.split("\t") for line in output.splitlines() if line.startswith("REACHABILITY_CHECKED\t")), [])
    selftest.check("static-check 可达性报数与出路：REACHABILITY_CHECKED 判了 8 行、没判 0 行、截过闭包 5 行；有 UNREACHABLE 就跟一行「→ 怎么办」写换用例或删行",
                   checked[1:3] == ["8", "0"] and checked[4:5] == ["5"] and f"→ 怎么办：{REACHABILITY_HOWTO}" in output,
                   f"{checked}；{output[-600:]}")
    missing_stops = stops + ((None, (SELFTEST_REACHABILITY_FILE, "judge_that_is_not_in_the_source")),)
    missing_code, missing_output, _routes, _exempted = selftest_reachability_static_check(root, missing_stops)
    selftest.check("static-check 可达性起跑先核截断表：表里的函数在源码里找不到打 ERROR 点名它、退 2",
                   missing_code == 2 and any(line.startswith("ERROR\t") and "judge_that_is_not_in_the_source" in line for line in missing_output.splitlines()),
                   f"退 {missing_code}；{missing_output[-400:]}")
    small_root = os.path.join(work, "reachability-small")
    build_selftest_repository(small_root, SELFTEST_TABLE_ROWS)
    small_code, small_output = run_command_capturing(command_static_check, [small_root])
    selftest.check("static-check 可达性：选路表、截断表、豁免表点名的函数在这棵树里一个都没有（门禁样本那种小根）时打 REACHABILITY_NOT_JUDGED 点名原因、不打 REACHABILITY_CHECKED、不报 ERROR，退出码不受它影响",
                   small_code == 0 and any(line.startswith("REACHABILITY_NOT_JUDGED\t") for line in small_output.splitlines())
                   and "REACHABILITY_CHECKED" not in small_output and "ERROR\t" not in small_output,
                   f"退 {small_code}；{small_output[-500:]}")
    unknown_code, unknown_output = run_command_capturing(command_static_check, [small_root, "--option-never-used-before"])
    selftest.check("static-check 认不出的选项：打 ERROR 与「→ 怎么办」、退 2，一行都不判",
                   unknown_code == 2 and "ERROR\t" in unknown_output and "→ 怎么办" in unknown_output and "CHECKED" not in unknown_output,
                   f"退 {unknown_code}；{unknown_output[-400:]}")


# 闭包拒算的样本测试（读 CARGO_MANIFEST_DIR 的函数拒算 runtime-path，调它的测试闭包算不出键、已收的成员照有）插进 SELFTEST_REACHABILITY_SOURCE 的 mod tests 开头
SELFTEST_REACHABILITY_REFUSED_TESTS = '''    fn reads_the_manifest_directory() -> bool { std::env::var("CARGO_MANIFEST_DIR").is_ok() }

    #[test]
    fn refused_calls_never_called() { assert!(reads_the_manifest_directory()); assert_eq!(super::never_called(1), 7); }
    #[test]
    fn refused_calls_plain_only() { assert!(reads_the_manifest_directory()); assert_eq!(super::plain(), 4); }
    #[test]
    fn refused_enumeration_on_the_gpu_route() { assert!(reads_the_manifest_directory()); assert!(super::run_enumeration(1)); }

'''
SELFTEST_REACHABILITY_REFUSED_ROWS = [
    "refused-reaches-never-called\tcrates/tiny/src/lib.rs\t{ value * 7 }\t{ value * 8 }\t-p tiny --lib -- refused_calls_never_called\trefused_calls_never_called",
    "refused-misses-never-called\tcrates/tiny/src/lib.rs\t{ value * 7 }\t{ value * 8 }\t-p tiny --lib -- refused_calls_plain_only\trefused_calls_plain_only",
    "refused-behind-the-cpu-judge\tcrates/tiny/src/lib.rs\t{ value + 1 > 0 }\t{ value + 1 > 5 }\t-p tiny --lib -- refused_enumeration_on_the_gpu_route\trefused_enumeration_on_the_gpu_route",
    "whole-misses-never-called\tcrates/tiny/src/lib.rs\t{ value * 7 }\t{ value * 8 }\t-p tiny --lib -- plain_sum\tplain_sum",
    "whole-behind-the-cpu-judge\tcrates/tiny/src/lib.rs\t{ value + 1 > 0 }\t{ value + 1 > 5 }\t-p tiny --lib -- enumeration_judges\tenumeration_judges",
]


def selftest_reachability_refused_cells(selftest, work):
    """static-check 可达性遇到点名测试的闭包拒算（文件头「点名测试够不着被改坏的函数」闭包拒算那一条）：样本里 refused_* 三条测试调了读 CARGO_MANIFEST_DIR 的函数
    （闭包算不出键），whole_* 两行点名的测试闭包不拒算作对照。截断表、多机配置与 selftest_reachability_cells 同一份。
    弄坏开关 reachability-refused-unjudged（拒算的一律报没判）下头一格必须判错。"""
    root = os.path.join(work, "reachability-refused")
    source = SELFTEST_REACHABILITY_SOURCE.replace("mod tests {\n", "mod tests {\n" + SELFTEST_REACHABILITY_REFUSED_TESTS, 1)
    build_selftest_repository(root, SELFTEST_REACHABILITY_REFUSED_ROWS, source=source)
    exit_code, output, routes, exempted = selftest_reachability_static_check(root, SELFTEST_REACHABILITY_STOPS)
    unjudged = {fields[2]: fields[3] for fields in (line.split("\t") for line in output.splitlines()) if fields[0] == "REACHABILITY_UNJUDGED" and len(fields) > 3}
    selftest.check("static-check 可达性的闭包拒算：点名测试的闭包拒算（调了读 CARGO_MANIFEST_DIR 的函数）、被改坏的函数 never_called 落在已收部分里的那一行不报够不着、不报没判",
                   "refused-reaches-never-called" not in routes and "refused-reaches-never-called" not in unjudged and "refused-reaches-never-called" not in exempted,
                   f"{routes}；没判 {unjudged}；{output[-800:]}")
    selftest.check("static-check 可达性的闭包拒算：被改坏的函数不在已收部分里的那一行不报 UNREACHABLE closure、报 REACHABILITY_UNJUDGED，原因句写「闭包拒算、已收部分里没有被改坏的单元」与拒算原因 runtime-path",
                   "refused-misses-never-called" not in routes and "闭包拒算、已收部分里没有被改坏的单元" in unjudged.get("refused-misses-never-called", "")
                   and "runtime-path" in unjudged.get("refused-misses-never-called", ""), f"{routes}；没判 {unjudged}；{output[-800:]}")
    selftest.check("static-check 可达性的闭包拒算：开 GPU、被改坏的 CPU 判器只在截过的闭包之外的那一行不报 gpu-route、报 REACHABILITY_UNJUDGED，原因句写「闭包拒算、截过的闭包已收部分里没有被改坏的单元」",
                   "refused-behind-the-cpu-judge" not in routes
                   and "闭包拒算、截过的闭包已收部分里没有被改坏的单元" in unjudged.get("refused-behind-the-cpu-judge", ""), f"{routes}；没判 {unjudged}；{output[-800:]}")
    checked = next((line.split("\t") for line in output.splitlines() if line.startswith("REACHABILITY_CHECKED\t")), [])
    selftest.check("static-check 可达性的闭包拒算，不拒算的行判法不变：点名测试闭包不拒算的两行照报（never_called 不在 plain_sum 的闭包里报 closure、CPU 判器只在 CPU 那一臂后面报 gpu-route），"
                   "退 1，REACHABILITY_CHECKED 判了 3 行、没判 2 行",
                   exit_code == 1 and routes.get("whole-misses-never-called") == "closure" and routes.get("whole-behind-the-cpu-judge") == "gpu-route"
                   and checked[1:3] == ["3", "2"], f"退 {exit_code}；{routes}；{checked}；{output[-800:]}")


def selftest_closure_refused_cell(selftest, work):
    """点名的测试在这一组特性下不编（#[cfg(feature = "gpu")] 的测试、cargo 参数没开 gpu）：闭包算不成时真错照原话报出、点名它的变异表的行、带出路，
    不被进程池换成 PicklingError。10 个闭包要算，GATE_MUTATION_KEY_WORKERS=2 走并行（子进程里抛的），=1 走就地；run_stage 与 --list-items 各核。
    弄坏开关 closure-error-masked（模块不登记进 sys.modules、子进程原样抛）下并行那一趟必须判错。"""
    tests = "".join(f"    #[test]\n    fn adds{index}() {{}}\n" for index in range(9))
    source = ("pub fn add(left: u32, right: u32) -> u32 { left + right }\n\n#[cfg(test)]\nmod tests {\n" + tests
              + '    #[cfg(feature = "gpu")]\n    #[test]\n    fn gpu_only() {}\n}\n')
    rows = [f"plain-{index}\tcrates/tiny/src/lib.rs\t{{ left + right }}\t{{ left * right }}\t-p tiny --lib -- adds{index}\tadds{index}" for index in range(9)]
    rows.append("gated-without-feature\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left - right }\t-p tiny --lib -- gpu_only\tgpu_only")
    repository = os.path.join(work, "closure-refused")
    build_selftest_repository(repository, rows, source=source)
    for key_workers, how in (("2", "并行算（子进程里抛）"), ("1", "就地算")):
        try:
            exit_code, stdout, _stderr, _ = selftest.run(repository, GATE_MUTATION_KEY_WORKERS=key_workers)
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


def selftest_coverage_exact_cell(selftest, work):
    """记覆盖时点名用例前自己补的 --exact，不与行里已有的 --exact 叠成两个（libtest 报 Option 'exact' given more than once、覆盖记不成）：
    一行 libtest 参数已带 --exact 的变异，假 cargo 见两个 --exact 退 101；这一行抓到、覆盖记成 1 份，每一趟带 --exact 的 cargo 命令里 --exact 只出现一次。
    行里 --exact 后面写全名 tests::adds：libtest 带 --exact 时过滤词要与全名逐字相同（基线按行取结果的 row_baseline_output 与假 cargo 都照这一条选），
    写短名 adds 真 libtest 一条都不跑，基线判「一行都没出现」。弄坏开关 exactduplicated 下必须判错。"""
    repository = os.path.join(work, "coverage-exact")
    build_selftest_repository(repository, ["add-becomes-mul-exact\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left * right }\t-p tiny --lib -- --exact tests::adds\tadds"])
    exit_code, stdout, stderr, invocations = selftest.run(repository)
    exact_lines = [line.split("\t")[0] for line in invocations if "--exact" in line.split("\t")[0].split()]
    selftest.check("行里已带 --exact 的变异：覆盖记成 1 份、没记成 0 份，带 --exact 的每一条 cargo 命令里 --exact 只出现一次（记覆盖那一趟也是）",
                   exit_code == 0 and "覆盖：这一趟记了 1 份" in stdout and "没记成 0 份" in stdout and exact_lines
                   and all(line.split().count("--exact") == 1 for line in exact_lines) and any("--message-format" not in line and "--no-run" not in line for line in exact_lines),
                   f"退 {exit_code}；带 --exact 的命令 {exact_lines}；{stdout[-300:]}；{stderr[-300:]}")


SELFTEST_NOT_RUN_ITEM_LINE = ("NOT_RUN_NO_GPU_ADAPTER test=crash_state_enumeration::fixed_script::stale_tail "
                              "reason=配置开了 GPU（ENABLE_GPU=1）而这份二进制没编进 gpu 特性：判段不跑\n")
SELFTEST_NOT_RUN_ITEM_OUTPUT = ("running 1 test\ntest tests::stale_tail_recovers_every_crash_state ... ok\n\nsuccesses:\n\n"
                                "---- tests::stale_tail_recovers_every_crash_state stdout ----\n" + SELFTEST_NOT_RUN_ITEM_LINE + "\n"
                                "successes:\n    tests::stale_tail_recovers_every_crash_state\n\ntest result: ok. 1 passed; 0 failed\n")


def selftest_not_run_item_name_cell(selftest, work):
    """本次未跑认项名形态（not_run_reason，文件头「本次未跑」）：点名的测试 ok、它那一段输出里是崩溃枚举用例判段不跑打的
    NOT_RUN_NO_GPU_ADAPTER test=crash_state_enumeration::<项名>（项名 fixed_script::stale_tail 的末一段不是测试名），记本次未跑、不记没红；
    同一份输出去掉那一行照记没红。弄坏开关 not-run-test-name-only（只认 test= 对得上测试名的）下这一格判错。"""
    environment = dict(selftest.base_environment)
    row = Row(7, "judge-step-skipped", "crates/tiny/src/lib.rs", "{ 1 }", "{ 2 }", "-p tiny --lib -- stale_tail_recovers_every_crash_state",
              "stale_tail_recovers_every_crash_state", ())
    verdict, text = verdict_of_run(row, 0, SELFTEST_NOT_RUN_ITEM_OUTPUT, environment)
    plain, plain_text = verdict_of_run(row, 0, SELFTEST_NOT_RUN_ITEM_OUTPUT.replace(SELFTEST_NOT_RUN_ITEM_LINE, ""), environment)
    selftest.check("本次未跑认项名：点名的测试 ok、输出里是 test=crash_state_enumeration::<项名> 的本次未跑行（项名不是测试名），记本次未跑、不记没红；没有那一行照记没红",
                   verdict == "notrun" and "test=crash_state_enumeration::fixed_script::stale_tail" in text and "对不上点名的测试名" in text and plain == "failure",
                   f"有那一行判 {verdict}（{text}）；没有那一行判 {plain}（{plain_text.splitlines()[0] if plain_text else ''}）")


SELFTEST_GPU_ROUTE_SOURCE = '''// 开 GPU 补 gpu 特性那一格的样本 crate：enumeration_judges 经 run_enumeration 够到选路函数 judge_not_run_without_the_gpu_feature，plain_sum 够不着
// TEST enumeration_judges NEEDS 1 + 1 RUNS enumeration_judges,run_enumeration,judge_not_run_without_the_gpu_feature
// TEST plain_sum NEEDS 2 + 2
// TEST without_the_gpu_feature_routes_to_not_run NEEDS configured && !compiled
pub fn judge_not_run_without_the_gpu_feature(configured: bool, compiled: bool) -> bool { configured && !compiled }

pub fn run_enumeration() -> u32 { if judge_not_run_without_the_gpu_feature(false, true) { 0 } else { 1 + 1 } }

pub fn plain() -> u32 { 2 + 2 }

#[cfg(test)]
mod tests {
    #[test]
    fn enumeration_judges() { assert_eq!(super::run_enumeration(), 2); }
    #[test]
    fn plain_sum() { assert_eq!(super::plain(), 4); }
    #[cfg(not(feature = "gpu"))]
    #[test]
    fn without_the_gpu_feature_routes_to_not_run() { assert!(super::judge_not_run_without_the_gpu_feature(true, false)); }
}
'''
SELFTEST_GPU_ROUTE_ROWS = [
    "enumeration-two-to-three\tcrates/tiny/src/lib.rs\t{ 1 + 1 }\t{ 1 + 2 }\t-p tiny --lib -- enumeration_judges\tenumeration_judges",
    "plain-four-to-five\tcrates/tiny/src/lib.rs\t{ 2 + 2 }\t{ 2 + 3 }\t-p tiny --lib -- plain_sum\tplain_sum",
    "route-and-becomes-or\tcrates/tiny/src/lib.rs\t{ configured && !compiled }\t{ configured || !compiled }\t"
    "-p tiny --lib -- without_the_gpu_feature_routes_to_not_run\twithout_the_gpu_feature_routes_to_not_run",
]


def selftest_gpu_route_feature_cell(selftest, work):
    """开 GPU 时崩溃判段那几行补 gpu 特性（gpu_route_rows，文件头「开 GPU 时崩溃判段那几行补 gpu 特性」）：选路函数换成样本 crate 里的
    judge_not_run_without_the_gpu_feature（GPU_ROUTE_UNITS[0] 临时换掉），多机配置 ENABLE_GPU=1、两行都不带特性：点名测试够得着选路函数的那一行
    起的每一次 cargo 都带 --features gpu（基线、变异的先编与跑），够不着的那一行一次都不带；
    点名 #[cfg(not(feature = "gpu"))] 用例的那一行（也够得着选路函数）补了就不编，照原样跑、一次都不带；stderr 点名补了哪一行、哪一行补不上；
    行键照补了之后的特性算：配置换成 ENABLE_GPU=0 时那一行的键不同、参数不带特性，另一行的键相同。弄坏开关 gpu-route-feature-unadded（不补）下这一格判错。"""
    repository = os.path.join(work, "gpu-route-feature")
    build_selftest_repository(repository, SELFTEST_GPU_ROUTE_ROWS, source=SELFTEST_GPU_ROUTE_SOURCE)
    selftest_write(os.path.join(repository, CARGO_CONFIGURATION_DIRECTORY, "config.toml"), SELFTEST_CARGO_CONFIGURATION)
    gpu_off = os.path.join(work, "gpu-route-off", "multi-host.env")
    selftest_write(gpu_off, "# 自证用的多机配置：GPU 关\nENABLE_GPU=0\n")
    saved = GPU_ROUTE_UNITS[0]
    try:
        GPU_ROUTE_UNITS[0] = (("tiny", "crates/tiny/src/lib.rs", "judge_not_run_without_the_gpu_feature"),)
        exit_code, stdout, stderr, invocations = selftest.run(repository)
        table_rows = read_table(os.path.join(repository, TABLE), selftest.base_environment)[0]
        _context, on_rows, on_components, _library = plan_rows(repository, table_rows, dict(selftest.base_environment))
        _context, off_rows, off_components, _library = plan_rows(repository, table_rows, dict(selftest.base_environment, GATE_MUTATION_MULTI_HOST_CONFIG=gpu_off))
    finally:
        GPU_ROUTE_UNITS[0] = saved
    enumeration, plain, unbuilt = (row.line_number for row in table_rows)
    calls = [line.split("\t")[0] for line in invocations]
    enumeration_calls = [line for line in calls if "enumeration_judges" in line]
    gpu_calls = [line for line in calls if "--features gpu" in line]
    route_note = next((line for line in stderr.splitlines() if "补上 gpu 特性起" in line), "")
    selftest.check("开 GPU 时点名测试走崩溃判段的那一行补 gpu 特性：配置 ENABLE_GPU=1、行不带特性，起的 cargo 带 --features gpu，够不着选路函数的那一行不带；"
                   "带 gpu 特性就不编的用例（cfg(not(feature = \"gpu\"))）那一行照原样跑；stderr 点名补了哪一行、哪一行补不上；"
                   "行键照补了之后的特性算（GPU 关时那一行的键不同、另一行相同）",
                   exit_code == 0 and "计数：抓到 3 条、没红 0 条" in stdout
                   and any("--show-output" in line and "enumeration_judges" in line for line in gpu_calls)
                   and enumeration_calls and all("--features gpu" in line for line in enumeration_calls)
                   and not any("plain_sum" in line or "without_the_gpu_feature_routes_to_not_run" in line for line in gpu_calls)
                   and f"{TABLE}:{enumeration}（--features gpu）" in route_note and f"{TABLE}:{plain}（" not in route_note
                   and f"{TABLE}:{unbuilt} 点名的测试走崩溃判段，补 gpu 特性之后不编" in stderr
                   and "--features gpu" in {row.line_number: row for row in on_rows}[enumeration].arguments
                   and "--features" not in {row.line_number: row for row in off_rows}[enumeration].arguments
                   and on_components[enumeration].row_key and on_components[enumeration].row_key != off_components[enumeration].row_key
                   and on_components[plain].row_key == off_components[plain].row_key,
                   f"退 {exit_code}；带 --features gpu 的 {gpu_calls}；点名 enumeration_judges 的 {enumeration_calls}；"
                   f"键 开 {on_components[enumeration].row_key} / 关 {off_components[enumeration].row_key}；{stdout[-300:]}{stderr[-500:]}")


SELFTEST_GPU_THREADS_SOURCE = '''
// TEST released_agrees NEEDS 1 + 1 RUNS released_agrees,shared
pub fn shared() -> u32 { 1 + 1 }
'''
SELFTEST_GPU_THREADS_ROWS = [
    # 放开名单里的测试目标（目标名取 admission.py 的 GPU_TEST_THREADS_RELEASED_TARGETS，格里 format 进去）
    "released-one-plus-one\tcrates/tiny/src/lib.rs\t{ 1 + 1 }\t{ 1 + 2 }\t-p tiny --features gpu --test {target} -- released_agrees\treleased_agrees",
    SELFTEST_GPU_FEATURE_ROW,
    SELFTEST_TABLE_ROWS[0],
    "gpu-halve-exact\tcrates/tiny/src/lib.rs\t{ value / 2 }\t{ value / 3 }\t-p tiny --features gpu --lib -- --exact tests::halves\thalves",
    "gpu-negate-own-threads\tcrates/tiny/src/lib.rs\t{ 0 - value }\t{ value }\t-p tiny --features gpu --lib -- negates --test-threads=3\tnegates",
]


def selftest_gpu_test_threads_cell(selftest, work):
    """带 gpu 特性的格测试参数收尾带 --test-threads（with_gpu_test_threads，文件头「带 gpu 特性的格测试参数收尾带 --test-threads」）：
    五行同跑一趟（SINGLEFS_THREAD_CAP=7 当登记值）：点名放开名单里的测试目标（GPU_TEST_THREADS_RELEASED_TARGETS 取自 admission.py）的 gpu 行，
    基线、变异、插桩跑真跑测试的每一次都收尾带 `--test-threads 7`；--lib 的 gpu 行带 `--test-threads 1`；不带 gpu 的行一次都不带；
    gpu 行自己带 --exact 的，--exact 与 --test-threads 各只出现一次（点名用例在前、--test-threads 收尾）；gpu 行自己写了 --test-threads=3 的原样（不再补）；
    只编不跑的（--no-run）一次都不带。线程参数不进行键：登记值 7 换成 9，五行的键都不变。弄坏开关 gpu-test-threads-unadded（不补）下这一格判错。"""
    released = admission_module().GPU_TEST_THREADS_RELEASED_TARGETS[0]
    root = os.path.join(work, "gpu-test-threads")
    build_selftest_repository(root, [row.replace("{target}", released) for row in SELFTEST_GPU_THREADS_ROWS], extra_source=SELFTEST_GPU_THREADS_SOURCE)
    selftest_write(os.path.join(root, "crates", "tiny", "tests", f"{released}.rs"), "mod tests {\n    #[test]\n    fn released_agrees() { assert_eq!(tiny::shared(), 2); }\n}\n")
    selftest_write(os.path.join(root, CARGO_CONFIGURATION_DIRECTORY, "config.toml"), SELFTEST_CARGO_CONFIGURATION)
    exit_code, stdout, stderr, invocations = selftest.run(root, SINGLEFS_THREAD_CAP="7")
    table_rows = read_table(os.path.join(root, TABLE), selftest.base_environment)[0]
    _context, _rows, seven_components, _library = plan_rows(root, table_rows, dict(selftest.base_environment, SINGLEFS_THREAD_CAP="7"))
    _context, _rows, nine_components, _library = plan_rows(root, table_rows, dict(selftest.base_environment, SINGLEFS_THREAD_CAP="9"))

    def libtest_tail(line):
        words = line.split("\t")[0].split()
        return words[words.index("--") + 1:] if "--" in words else []

    def real_runs(test):
        """起了点名这条测试、真跑测试的每一次（--no-run 的先编不算）的 libtest 参数。"""
        return [libtest_tail(line) for line in invocations if "--no-run" not in line.split("\t")[0].split()
                and any(word in (test, "tests::" + test) for word in libtest_tail(line))]

    def thread_words(tail):
        """libtest 参数里的 --test-threads 开关词（--test-threads 或 --test-threads=N，不含分开写时它的值）。"""
        return [word for word in tail if word == "--test-threads" or word.startswith("--test-threads=")]
    released_runs, lib_runs, plain_runs = real_runs("released_agrees"), real_runs("doubles"), real_runs("adds")
    exact_runs, own_runs = [tail for tail in real_runs("tests::halves") if "--exact" in tail], real_runs("negates")
    compile_runs = [line for line in invocations if "--no-run" in line.split("\t")[0].split()]
    selftest.check("带 gpu 的行收尾带 --test-threads 1、放开名单里的目标带登记值、不带 gpu 的行不带：基线、变异、插桩跑每一次都照办，自己带 --exact 或 --test-threads=N 的不重复、各只一个，"
                   "只编不跑的不带，登记值换了行键不变",
                   exit_code == 0 and len(released_runs) >= 3 and all(tail[-2:] == ["--test-threads", "7"] for tail in released_runs)
                   and len(lib_runs) >= 3 and all(tail[-2:] == ["--test-threads", "1"] for tail in lib_runs)
                   and plain_runs and all(not thread_words(tail) for tail in plain_runs)
                   and len(exact_runs) >= 1 and all(tail.count("--exact") == 1 and thread_words(tail) == ["--test-threads"] and tail[-2:] == ["--test-threads", "1"]
                                                    and tail.index("--exact") < tail.index("--test-threads") for tail in exact_runs)
                   and len(own_runs) >= 3 and all(thread_words(tail) == ["--test-threads=3"] for tail in own_runs)
                   and compile_runs and all("--test-threads" not in line for line in compile_runs)
                   and all(seven_components[number].row_key and seven_components[number].row_key == nine_components[number].row_key for number in seven_components),
                   f"退 {exit_code}；放开目标 {released_runs}；--lib {lib_runs}；不带 gpu {plain_runs}；带 --exact {exact_runs}；自带 --test-threads=3 {own_runs}；"
                   f"先编 {compile_runs}；{stdout[-300:]}{stderr[-300:]}")


# 重测试目标只许 release 那两格（文件头同名一段）：tiny 另有一个集成测试目标 tiny_crash_stream，自证自己的登记表只登记它一条崩溃枚举用例
SELFTEST_RELEASE_SOURCE = '''
// TEST streams_agree NEEDS 3 + 4 RUNS streams_agree,stream_total
pub fn stream_total() -> u32 { 3 + 4 }
'''
SELFTEST_RELEASE_REGISTRY = ("# 自证用的登记表：一条崩溃枚举用例点 tiny 的集成测试目标 tiny_crash_stream\n"
                             "crash-case:tiny-stream\t-\ttest=tiny:tiny_crash_stream:streams_agree count-line=TINY\t# 自证样本\n")
SELFTEST_RELEASE_CRASH_ROW = "stream-seven-becomes-eight\tcrates/tiny/src/lib.rs\t{ 3 + 4 }\t{ 3 + 5 }\t-p tiny --test tiny_crash_stream -- streams_agree\tstreams_agree"
SELFTEST_RELEASE_WRITTEN_ROW = ("halve-released-becomes-third\tcrates/tiny/src/lib.rs\t{ value / 2 }\t{ value / 3 }\t"
                                "-p tiny --release --features gpu --lib -- halves\thalves")


def selftest_release_repository(work, name, rows):
    """→ (小仓, 登记表的根)：小仓带 tiny 的集成测试目标 tiny_crash_stream（.cargo/ 里有 gpu 那几行要的 [env]）；登记表另放一个根，那两格把 RELEASE_REGISTRY_ROOT 换成它。"""
    repository = os.path.join(work, name)
    build_selftest_repository(repository, rows, extra_source=SELFTEST_RELEASE_SOURCE)
    selftest_write(os.path.join(repository, "crates", "tiny", "tests", "tiny_crash_stream.rs"),
                   "mod tests {\n    #[test]\n    fn streams_agree() { assert_eq!(tiny::stream_total(), 7); }\n}\n")
    selftest_write(os.path.join(repository, CARGO_CONFIGURATION_DIRECTORY, "config.toml"), SELFTEST_CARGO_CONFIGURATION)
    registry_root = os.path.join(work, name + "-registry")
    selftest_write(os.path.join(registry_root, ".claude", "gate.d", "stage-inputs.tsv"), SELFTEST_RELEASE_REGISTRY)
    return repository, registry_root


def selftest_release_forced_cell(selftest, work):
    """门禁补 --release（release_forced_row，文件头「重测试目标只许 release」）：三行同跑一趟——点名登记的崩溃枚举用例目标的一行、带 gpu 特性的一行、
    两样都不中的一行；前两行起的每一次 cargo（基线、变异、插桩编译与插桩跑）`--` 之前都带 --release、只带一个，第三行一次都不带；stderr 那一行点名前两行、不点第三行。
    弄坏开关 release-unforced（不补）下这一格判错。"""
    repository, registry_root = selftest_release_repository(work, "release-forced", [SELFTEST_RELEASE_CRASH_ROW, SELFTEST_GPU_FEATURE_ROW, SELFTEST_TABLE_ROWS[0]])
    saved = RELEASE_REGISTRY_ROOT[0]
    try:
        RELEASE_REGISTRY_ROOT[0] = registry_root
        exit_code, stdout, stderr, invocations = selftest.run(repository)
    finally:
        RELEASE_REGISTRY_ROOT[0] = saved

    def cargo_words(line):
        words = line.split("\t")[0].split()
        return words[:words.index("--")] if "--" in words else words
    calls = [cargo_words(line) for line in invocations]
    hit_calls = [words for words in calls if "tiny_crash_stream" in words or "gpu" in words]
    plain_calls = [words for words in calls if "tiny_crash_stream" not in words and "gpu" not in words and "-p" in words]
    release_note = next((line for line in stderr.splitlines() if "重测试目标只许 release" in line), "")
    selftest.check("重测试目标只许 release：命中的两行（crash-case 目标、gpu 特性）起的每一次 cargo 都带 --release、只带一个，不命中的那一行一次都不带，stderr 点名补了哪两行",
                   exit_code == 0 and len(hit_calls) >= 4 and all(words.count("--release") == 1 for words in hit_calls)
                   and plain_calls and not any("--release" in words for words in plain_calls)
                   and f"{TABLE}:2（crash-case 目标 tiny:tiny_crash_stream" in release_note and f"{TABLE}:3（gpu 特性 gpu）" in release_note
                   and f"{TABLE}:4（" not in release_note,
                   f"退 {exit_code}；命中的 {hit_calls}；不命中的 {plain_calls}；stderr 那一行 {release_note!r}；{stdout[-300:]}{stderr[-300:]}")


def selftest_release_static_check_cell(selftest, work):
    """static-check 的 RELEASE（文件头「重测试目标只许 release」）：一张表四行——带 gpu 特性没写 --release、点名登记的目标没写、命中而写了 --release、两样都不中：
    前两行各报一行 RELEASE（带依据），跟一行「→ 怎么办」写「第五段补上 --release」，退 1；另一张表把前两行写上 --release，一行都不报、退 0。
    弄坏开关 release-unchecked（不判）下这一格判错。"""
    rows = [SELFTEST_GPU_FEATURE_ROW, SELFTEST_RELEASE_CRASH_ROW, SELFTEST_RELEASE_WRITTEN_ROW, SELFTEST_TABLE_ROWS[0]]
    repository, registry_root = selftest_release_repository(work, "release-static", rows)
    written = [row.replace("--lib -- doubles", "--lib --release -- doubles").replace("--test tiny_crash_stream --", "--test tiny_crash_stream --release --") for row in rows]
    written_repository, _written_registry = selftest_release_repository(work, "release-static-written", written)
    saved = RELEASE_REGISTRY_ROOT[0]
    try:
        RELEASE_REGISTRY_ROOT[0] = registry_root
        exit_code, output = run_command_capturing(command_static_check, [repository])
        written_code, written_output = run_command_capturing(command_static_check, [written_repository])
    finally:
        RELEASE_REGISTRY_ROOT[0] = saved
    release_lines = [line for line in output.splitlines() if line.startswith("RELEASE\t")]
    selftest.check("static-check 的 RELEASE：命中而第五段没写 --release 的两行（gpu 特性、crash-case 目标）各报一行、带依据、跟一行写出路，退 1；命中写了的与不命中的不报，两行都写上之后退 0",
                   exit_code == 1 and len(release_lines) == 2
                   and release_lines[0] == "RELEASE\t2\tgpu-double-becomes-triple\tgpu 特性 gpu"
                   and release_lines[1].startswith("RELEASE\t3\tstream-seven-becomes-eight\tcrash-case 目标 tiny:tiny_crash_stream（--test 点名；登记 crash-case:tiny-stream）")
                   and "     → 怎么办：第五段补上 --release" in output
                   and written_code == 0 and "RELEASE\t" not in written_output,
                   f"退 {exit_code}：{output}；都写上之后退 {written_code}：{written_output}")


def selftest_debuginfo_cells(selftest, work):
    """起 cargo 时关调试信息（文件头「调试信息」）：一趟跑下来假 cargo 收到的每一次 test（基线、变异、插桩编译与插桩跑）环境里四档的
    CARGO_PROFILE_<档>_DEBUG 都是 false，调用方设的 CARGO_PROFILE_RELEASE_DEBUG=true 也被顶掉；取值换成 line-tables-only 的那一趟每一行都现判、栏是工具链或依赖变了，
    换回 false 又都复用。弄坏开关 debuginfo-kept（不设）下两格判错，debuginfo-unkeyed（不进键）下后一格判错。"""
    repository = os.path.join(work, "debuginfo")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    environment_log = os.path.join(work, "debuginfo-environment.jsonl")
    exit_code, _stdout, stderr, _ = selftest.run(repository, FAKE_CARGO_ENVIRONMENT_LOG=environment_log, CARGO_PROFILE_RELEASE_DEBUG="true")
    records = environment_records(environment_log)
    wanted = {f"CARGO_PROFILE_{profile}_DEBUG": "false" for profile in CARGO_DEBUG_PROFILES}
    off = [record for record in records if all(record.get(name) == value for name, value in wanted.items())]
    forms = {("--no-run" in record["ARGUMENTS"].split(), any(word.startswith("--message-format") for word in record["ARGUMENTS"].split())) for record in records}
    selftest.check(f"门禁起的每条 cargo 都关了调试信息：{len(records)} 次（基线、变异、插桩编译与插桩跑）每一次四档的 CARGO_PROFILE_<档>_DEBUG 都是 false，调用方设的 true 也顶掉",
                   exit_code == 0 and records and len(off) == len(records) and {(False, False), (True, True)} <= forms,
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
    seeded = selftest.run(repository, GATE_MUTATION_WORKERS="4", GATE_MUTATION_TARGET_DIR=target, FAKE_CARGO_DEPENDENCY_LOG=dependency_log,
                          FAKE_CARGO_ENVIRONMENT_LOG=environment_log, FAKE_CARGO_BARRIER_DIR=os.path.join(work, "seed-barrier"), FAKE_CARGO_BARRIER_PRISTINE=source_hash(repository))
    compiled = open(dependency_log, encoding="utf-8").read().splitlines() if os.path.exists(dependency_log) else []
    seeds = sorted([target + SEED_TARGET_SUFFIX, target + SEED_TARGET_SUFFIX + COVERAGE_TARGET_SUFFIX])
    worker_targets = {record.get("CARGO_TARGET_DIR") for record in environment_records(environment_log)} - set(seeds)
    plain_worker_targets = {path for path in worker_targets if not path.endswith(COVERAGE_TARGET_SUFFIX)}
    copied_into = [path for path in sorted(worker_targets) if os.path.isfile(os.path.join(path, "debug", "deps", "libfakedep-"
                   + hashlib.sha256(b"fakedep1.2.3").hexdigest()[:16] + ".rlib"))]
    selftest.check("4 个工作进程：外部依赖只在先编那一份里各编一次（普通、插桩各 1 次），起过 cargo 的工作进程编译目录里一次都没编、依赖产物都是拷来的",
                   seeded[0] == 0 and sorted(compiled) == seeds and len(plain_worker_targets) >= 2 and copied_into == sorted(worker_targets),
                   f"退 {seeded[0]}；编过依赖的编译目录 {compiled}（要 {seeds}）；起过 cargo 的工作进程编译目录 {sorted(worker_targets)}，有拷来的 {copied_into}；{seeded[2][-600:]}")
    reference = selftest.run(selftest_new_library_copy(work, repository, "seed-reference"), GATE_MUTATION_WORKERS="1",
                             GATE_MUTATION_TARGET_DIR=os.path.join(work, "seed-reference-target"))
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
    first = selftest.run(fresh_repository, GATE_MUTATION_WORKERS="2", GATE_MUTATION_TARGET_DIR=fresh_target)
    replace_in_file(os.path.join(fresh_repository, "crates", "tiny", "src", "lib.rs"), "// 本模块自证的样本 crate", "// 本模块自证的样本 crate（改过一次）")
    # 只改了注释，单元键不变：同一棵树再跑一趟全复用、不开工作进程，所以改过的这一份换一棵树（库是新的一份）再跑
    second = selftest.run(selftest_new_library_copy(work, fresh_repository, "seed-fresh-edited-once"), GATE_MUTATION_WORKERS="2", GATE_MUTATION_TARGET_DIR=fresh_target)
    selftest.check("先编那一份的源码按内容同步：改了 tiny 的 lib.rs 之后只同步那 1 个文件、摸 crates/tiny，先编那一趟 tiny 重编、整道照常",
                   first[0] == 0 and second[0] == 0 and "同步了 1 个文件（摸了整包的 1 个：crates/tiny）" in second[2]
                   and "先编一份依赖（普通）：" in second[2], f"头一趟退 {first[0]}；改过之后退 {second[0]}：{(second[1] + second[2])[-900:]}")
    replace_in_file(os.path.join(fresh_repository, "crates", "tiny", "src", "lib.rs"), "（改过一次）", "（改过两次）")
    kept = selftest.run(selftest_new_library_copy(work, fresh_repository, "seed-fresh-edited-twice"), GATE_MUTATION_WORKERS="2", GATE_MUTATION_TARGET_DIR=fresh_target,
                        GATE_MUTATION_BREAK=with_break(selftest, "seedmtimekept"))
    selftest.check("先编那一份的修改时刻被留住（seedmtimekept）：先编那一趟判红、点名 crates/tiny 是 fresh，一条都不跑",
                   kept[0] == 1 and "crates/tiny 的 tiny 是 fresh" in kept[1] + kept[2] and plain_invocations(kept[3]) == [],
                   f"退 {kept[0]}；{(kept[1] + kept[2])[-900:]}；假 cargo {kept[3]}")
    selftest_seed_group_cells(selftest, work)


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
    common = dict(GATE_MUTATION_WORKERS="2", GATE_MUTATION_TARGET_DIR=target)
    twice = selftest.run(repository, **common)
    selftest.check("先编那一份有两组（-p tiny 与 -p tiny --release）：后一组复用前一组编出来的 tiny、假 cargo 报 fresh，不算坏、整道照常",
                   twice[0] == 0 and "先编一份依赖（普通）：" in twice[2] and "编了 2 组" in twice[2] and "是 fresh" not in twice[1] + twice[2],
                   f"退 {twice[0]}：{(twice[1] + twice[2])[-900:]}")
    replace_in_file(os.path.join(repository, "crates", "tiny", "src", "lib.rs"), "// 本模块自证的样本 crate", "// 本模块自证的样本 crate（改过一次）")
    edited = selftest.run(selftest_new_library_copy(work, repository, "seed-groups-edited"), **common)   # 注释改动单元键不变：换一棵树才有行要现判
    selftest.check("先编那一份有两组、改了 tiny 的 lib.rs 之后：摸 crates/tiny、前一组重编、后一组复用，整道照常",
                   edited[0] == 0 and "摸了整包的 1 个：crates/tiny" in edited[2] and "编了 2 组" in edited[2] and "是 fresh" not in edited[1] + edited[2],
                   f"退 {edited[0]}：{(edited[1] + edited[2])[-900:]}")
    replace_in_file(os.path.join(repository, "crates", "tiny", "src", "lib.rs"), "（改过一次）", "（改过两次）")
    per_group = selftest.run(selftest_new_library_copy(work, repository, "seed-groups-per-group"), **common, GATE_MUTATION_BREAK=with_break(selftest, "seedfreshacrossgroups"))
    selftest.check("先编那一份有两组、每组各判各的（seedfreshacrossgroups）：后一组里 tiny 是 fresh 记成坏的，先编那一趟判红、一条都不跑",
                   per_group[0] == 1 and "crates/tiny 的 tiny 是 fresh" in per_group[1] + per_group[2] and "--release" in per_group[1] + per_group[2]
                   and plain_invocations(per_group[3]) == [], f"退 {per_group[0]}：{(per_group[1] + per_group[2])[-900:]}；假 cargo {per_group[3]}")
    replace_in_file(os.path.join(repository, "crates", "tiny", "src", "lib.rs"), "（改过两次）", "（改过三次）")
    kept = selftest.run(selftest_new_library_copy(work, repository, "seed-groups-kept"), **common, GATE_MUTATION_BREAK=with_break(selftest, "seedmtimekept"))
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


def selftest_pool_parallel_cells(selftest, work):
    """派活那一段（文件头「派活那一段」）：变异表 8 行、cargo 参数 `--` 之前各差一个 -j，基线分成 8 批；假 cargo 记每次起止的时刻（FAKE_CARGO_SPAN_LOG）。
    4 个工作进程那一趟：同时在跑的基线峰值不少于 4。
    磁盘余量压到 1 个那一趟（GATE_MUTATION_DISK_FREE=200G：一个工作进程两份编译目录各按上限 64G 算、加备用 30G，第二个连同先编那两份放不下）：
    头一行写开 1 个、卡在「磁盘余量」这一项，同时在跑的基线至多 1 批。
    弄坏开关 baseline-one-at-a-time 让头一格判错（同时在跑的基线只有 1 批）。"""
    def table_rows(count):
        return [f"pool-row-{index}\tcrates/tiny/src/lib.rs\t{{ left + right }}\t{{ left * (right + {index}) }}\t-p tiny --lib -j {index + 1} -- adds\tadds"
                for index in range(count)]

    def spans_of(path):
        """[(start 或 end, TMPDIR 的名字, 时刻)]，照文件里的次序（假 cargo 每次 O_APPEND 一次写一行，行序就是真实的先后）。"""
        events = []
        with contextlib.suppress(OSError):
            with open(path, encoding="utf-8") as handle:
                for line in handle.read().splitlines():
                    fields = line.split("\t")
                    with contextlib.suppress(ValueError, IndexError):
                        events.append((fields[0], fields[1], float(fields[2].replace(",", "."))))
        return events

    def baseline_peak(events):
        opened, peak = set(), 0
        for kind, stamp, _moment in events:
            if not stamp.startswith("baseline-"):
                continue
            if kind == "start":
                opened.add(stamp)
                peak = max(peak, len(opened))
            else:
                opened.discard(stamp)
        return peak

    for name in ("pool-parallel-spans.log", "pool-serial-spans.log"):
        with contextlib.suppress(FileNotFoundError):
            os.unlink(os.path.join(work, name))
    parallel_repository = os.path.join(work, "pool-parallel")
    build_selftest_repository(parallel_repository, table_rows(8))
    parallel_spans = os.path.join(work, "pool-parallel-spans.log")
    parallel = selftest.run(parallel_repository, GATE_MUTATION_WORKERS="4",
                            FAKE_CARGO_SPAN_LOG=parallel_spans, SINGLEFS_THREAD_CAP="4")
    parallel_events = spans_of(parallel_spans)
    selftest.check("4 个工作进程、8 批基线：同时在跑的基线不少于工作进程数（4），8 行都判成抓到",
                   parallel[0] == 0 and "这一轮开 4 个工作进程" in parallel[2] and "现跑基线 8 批 8 行" in parallel[2]
                   and baseline_peak(parallel_events) >= 4 and len(per_row_how(parallel[1])) == 8,
                   f"退 {parallel[0]}；同时在跑的基线峰值 {baseline_peak(parallel_events)}（cargo 起止 {len(parallel_events)} 行）；抓到 {len(per_row_how(parallel[1]))} 行；"
                   f"{parallel[2][:700]}")
    serial_repository = os.path.join(work, "pool-serial")
    build_selftest_repository(serial_repository, table_rows(3))
    serial_spans = os.path.join(work, "pool-serial-spans.log")
    disk_squeezed = {DISK_FREE_VARIABLE: "200G", TARGET_DIRECTORY_CAP_VARIABLE: "64G", DISK_RESERVE_VARIABLE: "30G"}
    serial = selftest.run(serial_repository, GATE_MUTATION_WORKERS="16",
                          FAKE_CARGO_SPAN_LOG=serial_spans, FAKE_CARGO_SPAN_SLEEP="0.1", SINGLEFS_THREAD_CAP="31", **disk_squeezed)
    serial_events = spans_of(serial_spans)
    selftest.check("工作进程数被磁盘余量压到 1：头一行写开 1 个、卡在「磁盘余量」这一项，同时在跑的基线至多 1 批",
                   serial[0] == 0 and "这一轮开 1 个工作进程" in serial[2] and "卡在「磁盘余量」这一项" in serial[2]
                   and baseline_peak(serial_events) == 1 and len(per_row_how(serial[1])) == 3,
                   f"退 {serial[0]}；cargo 起止 {len(serial_events)} 行，同时在跑的基线峰值 {baseline_peak(serial_events)}；{serial[2][:700]}")


def selftest_mutation_run_service_cells(selftest, work):
    """变异复跑的崩溃判定（文件头「变异复跑的崩溃判定」）：假 cargo 每次真跑测试照收到的连哪一处往服务里写一条（演引擎写崩溃判定），
    一趟跑下来：每一次连的都是它自己的一份临时服务（socket 与临时服务的标记相同、不是这一趟自证所在的那份临时服务、各次互不相同、跑完 socket 不在了），
    次数等于假 cargo 真跑测试的次数；这一趟自证所在的那份服务（演真服务）按那个前缀一条都没多。弄坏开关 mutation-run-on-real-service 下这一格判错。"""
    client = mutation_units.Library.client()
    repository = os.path.join(work, "mutation-run-service")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    service_log = os.path.join(work, "mutation-run-service.log")
    prefix = f"crates-mutation-rows-selftest/mutation-run-crash-verdicts/{os.getpid()}-{os.urandom(4).hex()}/"
    outer_socket = selftest.base_environment.get(client.SOCKET_VARIABLE, "")
    exit_code, _stdout, stderr, invocations = selftest.run(repository, FAKE_CARGO_SERVICE_LOG=service_log,
                                                           FAKE_CARGO_CRASH_VERDICT_PREFIX=prefix,
                                                           FAKE_CARGO_CLIENT_DIRECTORY=os.path.dirname(os.path.realpath(PRUNE_SERVER_TEMPORARY_SERVICE_SCRIPT)))
    seen = []
    if os.path.exists(service_log):
        with open(service_log, encoding="utf-8") as handle:
            seen = [line.split("\t") for line in handle.read().splitlines()]
    sockets = [fields[0] for fields in seen]
    test_runs = [line for line in invocations if "--no-run" not in line.split("\t")[0].split()]
    written_to_outer = client.process_client().scan(prefix.encode())
    selftest.check("变异复跑各对自己的临时 prune-server 跑：每次真跑测试连的都是它自己那一份（socket 与临时服务的标记相同、不是自证所在的那一份、各次互不相同、跑完不在了），"
                   "次数等于真跑测试的次数，自证所在的那一份（演真服务）按这个前缀一条都没多",
                   exit_code == 0 and seen and len(seen) == len(test_runs) and all(len(fields) == 3 and fields[0] and fields[0] == fields[1] for fields in seen)
                   and outer_socket not in sockets and len(set(sockets)) == len(sockets) and not any(os.path.exists(path) for path in sockets)
                   and not written_to_outer,
                   f"退 {exit_code}；真跑测试 {len(test_runs)} 次、写了 {len(seen)} 次；连的 {sockets[:4]}（自证所在的那一份 {outer_socket}）；"
                   f"自证所在的那一份按前缀多了 {len(written_to_outer)} 条；{stderr[-300:]}")


def selftest_path_dependency_copy_cells(selftest, work):
    """拷贝范围跟 path 依赖（文件头「拷贝范围」）：tiny 经 path 依赖仓根下拷贝范围之外的 sidecar/（sidecar 再经 path 依赖 sidecar-inner/），
    static-check 的 COPIED 里有这两处（run_stage 的 prechecks 调同一个 copied_with_path_dependencies）；path 指到树之外的，static-check 打 ERROR、退 2。
    弄坏开关 path-dependencies-not-copied 下两格判错。"""
    root = os.path.join(work, "path-dependency-copy")
    build_selftest_repository(root, SELFTEST_TABLE_ROWS)
    with open(os.path.join(root, "crates", "tiny", "Cargo.toml"), "a", encoding="utf-8") as handle:
        handle.write('\n[dependencies]\nsidecar = { path = "../../sidecar" }\n')
    selftest_write(os.path.join(root, "sidecar", "Cargo.toml"), '[package]\nname = "sidecar"\nversion = "0.1.0"\n\n[dependencies]\nsidecar-inner = { path = "../sidecar-inner" }\n')
    selftest_write(os.path.join(root, "sidecar", "src", "lib.rs"), "")
    selftest_write(os.path.join(root, "sidecar-inner", "Cargo.toml"), '[package]\nname = "sidecar-inner"\nversion = "0.1.0"\n')
    selftest_write(os.path.join(root, "sidecar-inner", "src", "lib.rs"), "")
    exit_code, output = run_command_capturing(command_static_check, [root])
    copied_line = next((line.split("\t", 1)[1].split(" ") for line in output.splitlines() if line.startswith("COPIED\t")), [])
    selftest.check("拷贝范围跟 path 依赖：tiny 依赖的 sidecar/ 与 sidecar 再依赖的 sidecar-inner/ 都进 static-check 的 COPIED",
                   exit_code in (0, 1) and "sidecar" in copied_line and "sidecar-inner" in copied_line, f"退 {exit_code}，COPIED {copied_line}；{output[-400:]}")
    outside = os.path.join(work, "path-dependency-outside", "tree")
    build_selftest_repository(outside, SELFTEST_TABLE_ROWS)
    with open(os.path.join(outside, "crates", "tiny", "Cargo.toml"), "a", encoding="utf-8") as handle:
        handle.write('\n[dependencies]\nstray = { path = "../../../stray" }\n')
    selftest_write(os.path.join(work, "path-dependency-outside", "stray", "Cargo.toml"), '[package]\nname = "stray"\nversion = "0.1.0"\n')
    exit_code, output = run_command_capturing(command_static_check, [outside])
    selftest.check("拷贝范围跟 path 依赖指到树之外：static-check 打 ERROR（拷贝范围跟不全 path 依赖）、退 2，出路连副本拷不全那一种指到 minimal-copy.py",
                   exit_code == 2 and "ERROR\t拷贝范围跟不全 path 依赖" in output and "minimal-copy.py" in output, f"退 {exit_code}；{output[-400:]}")


def selftest_manifest_path_cells(selftest, work):
    """变异行带 --manifest-path（文件头「--manifest-path」）：样本仓根下放一份工作区之外的小工程 standalone-project/（自己是工作区根，仓根工作区不列它、
    也没有谁经 path 依赖它、不在登记的拷贝范围里），变异表是样本表头一行（-p tiny）加两行各点它的一条测试（--manifest-path <路径> 与 --manifest-path=<路径> 两种写法）。
    假 cargo 只读 crates/tiny/src/lib.rs 里的「// TEST」行：那两条照 FILES 读副本里 standalone-project/src/lib.rs 的原文，改坏了就红。
      静态核：static-check 退 0、COPIED 里有 standalone-project、没有 ROW 行；
      基线与判红：头一趟退 0、三行都现判抓到、拒算 0 条，基线与变异的 cargo 照原样带两种写法的 --manifest-path 起；
      单元键照那个包的源码算：第二趟输入不变、三行都复用、cargo 一次都不起；改 standalone-project 里 scale 的函数体另一处，下一趟只现判点 scales 的那一行；
      不在仓内：另一棵样本仓里一行 --manifest-path 指到仓根之外的一份 Cargo.toml，static-check 打 ERROR、退 2，run_stage 退 1、一条都不跑。
    弄坏开关 manifest-path-ignored 下四格判错（拷贝范围不补，static-check 报 ROW、run_stage 判红；指到仓外的那一行不拒）。"""
    standalone_source = ("// 工作区之外的小工程（自证「--manifest-path」那几格）\n"
                         "pub fn scale(value: u32) -> u32 {\n    let margin = 0;\n    value * 5 + margin\n}\n"
                         "pub fn shift(value: u32) -> u32 { value + 11 }\n\n"
                         "#[cfg(test)]\nmod tests {\n    #[test]\n    fn scales() { assert_eq!(super::scale(2), 10); }\n"
                         "    #[test]\n    fn shifts() { assert_eq!(super::shift(1), 12); }\n}\n")
    tiny_tests = ("// TEST scales FILES ../../standalone-project/src/lib.rs value * 5 + margin\n"
                  "// TEST shifts FILES ../../standalone-project/src/lib.rs { value + 11 }\n")
    manifest_rows = [
        "standalone-scale-becomes-six\tstandalone-project/src/lib.rs\tvalue * 5 + margin\tvalue * 6 + margin\t"
        "--manifest-path standalone-project/Cargo.toml --lib -- scales\tscales",
        "standalone-shift-becomes-twelve\tstandalone-project/src/lib.rs\t{ value + 11 }\t{ value + 12 }\t"
        "--manifest-path=standalone-project/Cargo.toml --lib -- shifts\tshifts",
    ]
    repository = os.path.join(work, "manifest-path")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS[:1] + manifest_rows, extra_source=tiny_tests)
    selftest_write(os.path.join(repository, "standalone-project", "Cargo.toml"), '[package]\nname = "standalone"\nversion = "0.1.0"\n\n[workspace]\n')
    selftest_write(os.path.join(repository, "standalone-project", "src", "lib.rs"), standalone_source)
    exit_code, output = run_command_capturing(command_static_check, [repository])
    copied_line = next((line.split("\t", 1)[1].split(" ") for line in output.splitlines() if line.startswith("COPIED\t")), [])
    row_lines = [line for line in output.splitlines() if line.startswith("ROW\t")]
    selftest.check("--manifest-path 静态核：static-check 退 0，拷贝范围照 --manifest-path 补进工作区之外的 standalone-project、没有 ROW 行",
                   exit_code == 0 and "standalone-project" in copied_line and not row_lines, f"退 {exit_code}，COPIED {copied_line}，ROW {row_lines}；{output[-400:]}")
    first = selftest.run(repository)
    first_how = per_row_how(first[1])
    plain = plain_invocations(first[3])
    selftest.check("--manifest-path 基线与判红：头一趟退 0，-p tiny 那一行与两行 --manifest-path 都现判抓到、拒算 0 条，基线与变异的 cargo 照原样带两种写法的 --manifest-path 起",
                   first[0] == 0 and "现判 3 条、复用 0 条、拒算 0 条" in first[1]
                   and all(first_how.get(name, "").startswith("现判") for name in ("add-becomes-sub", "standalone-scale-becomes-six", "standalone-shift-becomes-twelve"))
                   and any("--manifest-path standalone-project/Cargo.toml" in line for line in plain)
                   and any("--manifest-path=standalone-project/Cargo.toml" in line for line in plain),
                   f"退 {first[0]}；{first_how}；cargo {plain}；{first[1][-500:]}{first[2][-300:]}")
    second = selftest.run(repository)
    replace_in_file(os.path.join(repository, "standalone-project", "src", "lib.rs"), "let margin = 0;", "let margin = 0 * value;")
    third = selftest.run(repository)
    third_how = per_row_how(third[1])
    selftest.check("--manifest-path 单元键照那个包的源码算：第二趟三行都复用、cargo 一次都不起；改 scale 的函数体另一处之后只现判点 scales 的那一行，别的两行复用",
                   second[0] == 0 and second[3] == [] and "现判 0 条、复用 3 条、拒算 0 条" in second[1]
                   and third[0] == 0 and "现判 1 条、复用 2 条、拒算 0 条" in third[1] and third_how.get("standalone-scale-becomes-six", "").startswith("现判")
                   and third_how.get("standalone-shift-becomes-twelve") == "复用" and third_how.get("add-becomes-sub") == "复用",
                   f"第二趟退 {second[0]}、cargo {second[3]}；第三趟退 {third[0]}、{third_how}；{second[1][-300:]}{third[1][-300:]}")
    outside = os.path.join(work, "manifest-path-outside", "tree")
    stray_row = "stray-manifest\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ value * 4 }\t--manifest-path ../stray-project/Cargo.toml --lib -- doubles\tdoubles"
    build_selftest_repository(outside, SELFTEST_TABLE_ROWS[:1] + [stray_row])
    selftest_write(os.path.join(work, "manifest-path-outside", "stray-project", "Cargo.toml"), '[package]\nname = "stray"\nversion = "0.1.0"\n\n[workspace]\n')
    exit_code, output = run_command_capturing(command_static_check, [outside])
    refused = selftest.run(outside)
    selftest.check("--manifest-path 不在仓内：指到仓根之外的一份 Cargo.toml，static-check 打 ERROR、点名那一行、退 2，run_stage 退 1、一条都不跑",
                   exit_code == 2 and "ERROR\t拷贝范围补不上变异行 --manifest-path 指的包目录" in output and f"{TABLE}:3 stray-manifest" in output
                   and "不在仓内" in output and refused[0] == 1 and refused[3] == [] and "拷贝范围补不上变异行 --manifest-path 指的包目录" in refused[1],
                   f"static-check 退 {exit_code}；{output[-300:]}；run_stage 退 {refused[0]}、cargo {refused[3]}；{refused[1][-300:]}")


def selftest_run_overrides_outside_service_cell(selftest, work):
    """只给自证换的运行记录、多机配置两个变量只在临时 prune-server 底下认（文件头「判定库」）：自证的基础环境里两个都设着，
    只把临时服务的标记设成空串（演平常跑） ⇒ run_stage 退 1、两个都点名、假 cargo 一次 test 都没起。弄坏开关 run-overrides-honored-outside-service 下这一格判错。"""
    client = mutation_units.Library.client()
    repository = os.path.join(work, "run-overrides-outside-service")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    exit_code, stdout, stderr, invocations = selftest.run(repository, **{client.TEMPORARY_SERVICE_VARIABLE: ""})
    printed = stdout + stderr   # 拒绝句经 print_rejection → say 打在标准输出上
    named = all(f"{name}（「" in printed for name in ("GATE_MUTATION_RUN_RECORDS_DIRECTORY", "GATE_MUTATION_MULTI_HOST_CONFIG"))
    selftest.check("不在临时 prune-server 底下设了只给自证换的运行记录、多机配置两个变量：退 1、两个都点名、一条都不跑",
                   exit_code == 1 and "不在临时 prune-server 底下设了" in printed and named and not invocations,
                   f"退 {exit_code}、假 cargo 起了 {len(invocations)} 次；{printed[-500:]}")


def run_selftest():
    work = tempfile.mkdtemp(prefix="crates-mutation-rows-selftest-")
    saved_environment = dict(os.environ)
    try:
        selftest = Selftest(work)
        # 下面第一趟 judging_copy 就算判法单元（环境给的是空的）：两份缓存目录先指到这一趟自证自己的目录，不碰真仓那两份
        for variable in (JUDGING_CACHE_DIRECTORY_VARIABLE, "FUNCTION_UNITS_LEX_CACHE_DIRECTORY"):
            os.environ[variable] = selftest.base_environment[variable]
        # 进程内直接调的几条命令（row-plan、import 这类，拿 os.environ）同样不读真仓的多机配置、不碰真共享根（文件头「判定库」末段）
        os.environ["GATE_MUTATION_RUN_RECORDS_DIRECTORY"] = selftest.base_environment["GATE_MUTATION_RUN_RECORDS_DIRECTORY"]
        # 判法冻成一份临时拷贝：别的会话这时候改了判法闭包里的哪一份（lib_function_units.py 这一类），自证里几十趟的判法单元照样一致
        frozen, frozen_made = judging_copy(work, "judging-frozen")
        selftest.check("判法单元的闭包算得出、冻成一份临时拷贝", frozen_made, "judging-units 算不出；单跑 python3 scripts/mutation/crates-mutation-rows.py judging-units . 看拒算")
        if frozen_made:
            JUDGING_ROOT[0] = frozen
        # plan 与 library-directory 读本进程的环境：与 run_stage 那一份对齐
        os.environ.clear()
        os.environ.update(selftest.base_environment)
        # 照这一趟的环境（连同弄坏开关）先算一次判法单元：拒算的（弄坏开关 judging-entries-with-temporary-service 这一类）红在「判法单元算得出」那一格，
        #   别的格每一趟都先算它（进程内直接调 plan_rows 的会抛 JudgingUnitsRefused），一格都不往下跑
        try:
            judging_units(selftest.base_environment)
            units_refusal = None
        except (JudgingUnitsRefused, OSError) as error:
            units_refusal = str(error)
        if units_refusal is not None:
            selftest.check("判法单元算得出（闭包里没有拒算）", False,
                           f"{units_refusal}；别的格每一趟都先算判法单元，这一趟一格都不往下跑（单跑 python3 scripts/mutation/crates-mutation-rows.py judging-units . 看拒算）")
        else:
            run_selftest_cells(selftest, work)
            selftest_unit_key_cells(selftest, work)
            selftest_reuse_cells(selftest, work)
            selftest_sentinel_uncounted_cell(selftest)
            selftest_parallel_key_cell(selftest, work)
            selftest_wgsl_unit_key_cell(selftest, work)
            selftest_multiple_packages_cell(selftest, work)
            selftest_specification_line_cell(selftest, work)
            selftest_judging_unit_cells(selftest, work)
            selftest_judging_cache_cell(selftest, work)
            selftest_reuse_fix_cells(selftest, work)
            selftest_item_lines_cells(selftest, work)
            selftest_build_profile_cell(selftest, work)
            selftest_worker_count_cells(selftest, work)
            selftest_memory_cells(selftest, work)
            selftest_worker_peak_cells(selftest, work)
            selftest_disk_cells(selftest, work)
            selftest_target_cap_cells(selftest, work)
            selftest_target_prune_cells(selftest, work)
            selftest_disk_peak_cells(selftest, work)
            selftest_libtest_result_cell(selftest, work)
            selftest_split_result_line_cell(selftest, work)
            selftest_cargo_configuration_cell(selftest, work)
            selftest_product_gpu_feature_cells(selftest, work)
            selftest_gpu_configuration_cells(selftest, work)
            selftest_target_directory_cells(selftest, work)
            selftest_worker_slot_cells(selftest, work)
            selftest_signal_and_pipe_cells(selftest, work)
            selftest_killed_run_resume_cell(selftest, work)
            selftest_shard_root_window_cell(selftest, work)
            selftest_stop_before_pool_cell(selftest, work)
            selftest_closure_refused_cell(selftest, work)
            selftest_coverage_exact_cell(selftest, work)
            selftest_not_run_item_name_cell(selftest, work)
            selftest_gpu_route_feature_cell(selftest, work)
            selftest_gpu_test_threads_cell(selftest, work)
            selftest_release_forced_cell(selftest, work)
            selftest_release_static_check_cell(selftest, work)
            selftest_anchor_and_impact_cells(selftest, work)
            selftest_undefined_identifier_cell(selftest, work)
            selftest_reachability_cells(selftest, work)
            selftest_reachability_refused_cells(selftest, work)
            selftest_debuginfo_cells(selftest, work)
            selftest_seed_cells(selftest, work)
            selftest_pool_parallel_cells(selftest, work)
            selftest_mutation_run_service_cells(selftest, work)
            selftest_run_overrides_outside_service_cell(selftest, work)
            selftest_path_dependency_copy_cells(selftest, work)
            selftest_manifest_path_cells(selftest, work)
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
        print("       KeyContext.components、baseline_reuse_problem、judge_coverage、dispatch_order、command_plan、escaping_literals、read_table 与 row_problems；")
        print("       单元切分与闭包看 scripts/mutation/lib_mutation_units.py 与 scripts/gate/lib_function_units.py；起 cargo 那一段看 run_cargo_command。")
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
          "久的先派、选行只判一部分退 3、多久没被引用才删、记录里没有主机名、拷贝范围两样、"
          "替换文空着的删行变异照常判、派活之前那几样 crates-mutation-replay 与 static-check 同一份判法、"
          "点名的测试打了 NOT_RUN 记本次未跑且变异那一趟补了 --show-output、只有本次未跑的退 77、should panic 的用例红了判抓到、"
          "单元键：被改坏函数的别一行变只现判原文在它里面的行（S34）、判法单元变了整张现判（S30）、改名一行都不现判（N34）、"
          "同文件加没人调的函数一行都不现判（N45）、WGSL 按 fn 取键（改另一个 fn 键不变、改被改坏的 fn 键变、原文只在注释里的拒算）、改点名测试调到的函数与被改坏函数用的常量现判点名它的行、基线按执行过的单元复用而不按整树快照、"
          "只改判法注释判法单元指纹不变而改 libtest 解析一处就变、拒算的照跑、"
          "撞键拒算（内容相同的两个函数各改一处、点名同一条测试的两行同键，两趟都拒算照跑、选行时照整张表判）、基线键与覆盖号带包与测试目标、一行的基线结局不随这一批拼进了哪几行而变、"
          "判法单元收全（改 judge_row 一处每一行都现判）、记红撤绿（同一把键改判没红或基线不绿之后下一趟现判）、不算数的结局不盖同一把键的算数结论、被判的树不是 git 工作树一条都不跑、"
          "按项机读行（--list-items --with-keys 每项一行 ITEM_PLAN 且条数对得上合计、不起 cargo；真跑每项恰好一行 ITEM_DONE、全复用的一趟也是、复用的 0 秒、现判的各项各记各的挂钟）、"
          "工作进程数取每台默认重活线程数（或 GATE_MUTATION_WORKERS）、要现跑的条数与内存上限 ÷ 那个编译并行度下每个工作进程的实测峰值的小者、取不到登记值就拒、没有内存上限这一项时照线程数、"
          "工作进程量自己整棵进程树的峰值、按编译并行度记下读得回来、只看最新的几份，编译并行度跟着进程数走时算到自洽，从没量过与记录读不了取保守值只开 1 个、"
          "拷贝范围含 .cargo/（点名的测试要编 gpu 特性的那一行在副本里读得到 [env]）、"
          "产品 GPU 包的 tests-on-a-real-gpu 与 gpu 同样当上卡的特性认（真跑测试的那几次收尾带 --test-threads 1、没卡打 NOT_RUN 记本次未跑）、不另套清场、"
          "真跑测试的每一次在副本里收到本模块所在仓的多机配置（只编不跑的收不到）、多机配置不在起 cargo 之前就拒，带 gpu 特性而给定状态全交 CPU 判的没红记本次未跑而 GPU 判过的照常判、"
          "被 TERM 停时删掉 $TMPDIR 下的 singlefs-mutation-replay-*（TERM 落在建目录那一瞬也不漏；进程池还没起就被停照样退 143、不抛 AttributeError）、"
          "片号大于这一趟工作进程数的 -w<片号> 开跑时回收而有进程在用的不删、两个变量都没设时编译目录不落 /tmp、"
          "同一个共用编译目录上并发的两次调用领到的片号集合不相交（片号锁）而片号锁被占着的 -w<片号> 回收时留着、"
          "--list-items 接 | head 时安静退、"
          "点名的测试在这一组特性下不编时 run_stage 与 --list-items 打真错并点名变异表的行（并行算与就地算一样）、行里已带 --exact 的变异记覆盖时 --exact 只出现一次、"
          "本次未跑认项名（test=crash_state_enumeration::<项名> 对不上测试名也记本次未跑）、开 GPU 时点名测试走崩溃判段的行补 gpu 特性起且键照补了之后的特性算、"
          "命中重测试目标的行（crash-case 目标、gpu 特性）补 --release 起而不命中的不补、static-check 报命中而第五段没写 --release 的行、"
          "工具链变了全现判、不按时限过期、llvm 工具不对判红、plan 只读且逐栏报数、"
          "锚点失效按行判（那一行记锚点失效、不入库、整道判红，别的行照跑照入库）而 row-plan、plan、--list-items 把锚点失效的行单列（数据文件那一行也不算键）、"
          "牵连面在样本 crate 上报对牵连最大的单元与行数、"
          "static-check 可达性判点名测试够不着被改坏的函数（开 GPU 时只在 CPU 判器入口后面的报 gpu-route、GPU 关时不报，闭包里根本没有的报 closure，"
          "测试自己写 CPU 配置的与 GPU 许建不出判表的豁免，测试直接调 CPU 判器入口的放行，截断表里的函数找不到报配置错）、"
          "磁盘余量放不下时工作进程数按余量收（已有的编译目录只算还能涨的、余量写法不认整道拒、没给余量读编译目录所在的盘）、"
          "编译目录超过上限时判完一格清掉增量产物、"
          "起的每条 cargo 都关了调试信息而构建档的取值进行键、几个工作进程时外部依赖只在先编那一份里编一次且只拷外部依赖的产物、先编那一份的源码按内容同步而摸过的包没重编就判红、"
          "拷依赖产物分到几条线程上同时拷、"
          "几个工作进程时基线的批同时在跑的不少于工作进程数、磁盘余量压到 1 个时同时在跑的基线至多 1 批、"
          "真跑测试的每一次各连自己的一份临时 prune-server 而真服务一条没多、不在临时服务底下设了只给自证换的变量拒跑、"
          "变异行带 --manifest-path 点工作区之外的小工程时拷贝范围补进那个目录、基线与变异照原样带 --manifest-path 起、单元键照那个包的源码算而指到仓外的拒）")
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
    # 起 cargo 之前判 RocksDB 预编库的取值（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」）：同一份小仓另放一份依赖 rocksdb 的 验证包清单、
    #   进程环境里四个取值是空的，PATH 上的假 cargo 不带 entry 认的标记 ⇒ 退 1、假 cargo 一次都不起、报出 entry 的红句
    rocksdb_repository = os.path.join(work, "rocksdb-repository")
    shutil.copytree(repository, rocksdb_repository, symlinks=True)
    selftest_write(os.path.join(rocksdb_repository, "crates", "singlefs-verification", "Cargo.toml"),
                   '[package]\nname = "singlefs-verification"\nversion = "0.0.0"\n[dependencies]\nrocksdb = { version = "=0.25.0", optional = true }\n')
    refused_by_rocksdb = selftest.run(rocksdb_repository, ROCKSDB_LIB_DIR="", ROCKSDB_STATIC="", SNAPPY_LIB_DIR="", SNAPPY_STATIC="")
    selftest.check("RocksDB 预编库的取值缺了：起 cargo 之前判红、退 1、假 cargo 一次都不起",
                   refused_by_rocksdb[0] == 1 and refused_by_rocksdb[3] == [] and "rocksdb-prebuilt entry：" in refused_by_rocksdb[1] + refused_by_rocksdb[2],
                   refused_by_rocksdb[1] + refused_by_rocksdb[2])
    shutil.rmtree(rocksdb_repository)
    # 二：进程数不同，换一棵树（库是新的一份，全部现判），判定仍逐字相同
    parallel = selftest.run(selftest_new_library_copy(work, repository, "parallel-repository"), GATE_MUTATION_WORKERS="4")
    selftest.check("GATE_MUTATION_WORKERS=4 与 =1 的判定逐字相同", selftest_verdict_view(parallel[1]) == selftest_verdict_view(first[1]), parallel[1])
    selftest.check("换一棵树（库是新的一份）：1 批基线 + 4 条变异全部现判，报「行新加」那一栏", len(plain_invocations(parallel[3])) == 5
                   and "行新加 4" in parallel[1], parallel[1] + parallel[2])
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
    # 五：跑到一半停掉（只供测试的开关），下一趟只跑剩下的；基线已判过 ok 的不再跑。从一份空库起：prune-server 没有删的操作，
    # 换一棵树：仓整份拷一份（连同它自己的 .git，git common-dir 不同，库的命名空间是新的一份）
    halfway_repository = os.path.join(work, "halfway-repository")
    shutil.copytree(repository, halfway_repository, symlinks=True)
    halfway = selftest.run(halfway_repository, GATE_MUTATION_STOP_AFTER_ROWS="2")
    selftest.check("跑完 2 条就退：判红、说明是只供测试的开关", halfway[0] == 1 and "GATE_MUTATION_STOP_AFTER_ROWS=2" in halfway[1]
                   and len(mutation_invocations(halfway[3], pristine)) == 2, halfway[1] + "\n".join(halfway[3]))
    resumed = selftest.run(halfway_repository)
    selftest.check("下一趟只跑剩下的 2 条、基线不再跑，判完的 2 条复用；给这一趟现判的 2 条测试补记覆盖",
                   resumed[0] == 0 and len(plain_invocations(resumed[3])) == 2 and len(mutation_invocations(resumed[3], pristine)) == 2
                   and len(coverage_invocations(resumed[3])) == 4, "\n".join(resumed[3]))
    # 六：上一次跑得久的先派，按用时从长到短；stderr 头一行报前移几条（用时表在运行记录目录里，文件头「判定库」末段）
    ordered_repository = selftest_new_library_copy(work, repository, "ordered-repository")   # 换一棵树：库是新的一份，6 条变异都要现判才有派活次序
    timings_directory = os.path.join(run_records_directory(ordered_repository, dict(selftest.base_environment)), "timings")
    table_rows, _malformed = read_table(table_path)
    long_rows = {"negates": "400", "halves": "1800"}
    for row in table_rows:
        if row.expected in long_rows:
            write_atomically(os.path.join(timings_directory, row_content_hash(row)),
                             mutation_units.record_text(RECORD_HEADER, dict(seconds_running=long_rows[row.expected])))
    ordered = selftest.run(ordered_repository)
    order = [line.split("\t")[0].split()[-1] for line in mutation_invocations(ordered[3], pristine)]
    ordered_head_line = (ordered[2].splitlines() or [""])[0]
    selftest.check("上一次跑得久的先派（按用时从长到短）", order[:2] == ["halves", "negates"] and "前移 2 条" in ordered_head_line,
                   f"派活次序 {order}；{ordered_head_line}")
    # 七：选行只判一部分：都抓到退 3、末行写明不是整道结论
    selection_file = os.path.join(work, "selection.txt")
    selftest_write(selection_file, "".join(f"{row.line_number}\n" for row in table_rows[:2]))
    selected_repository = selftest_new_library_copy(work, repository, "selected-repository")   # 换一棵树：库是新的一份，选中的 2 行现判
    selected = selftest.run(selected_repository, GATE_MUTATION_ROW_SELECTION=selection_file)
    selftest.check("选 2 行：都抓到退 3，末行写明只判了选中的 2 行", selected[0] == EXIT_SELECTION_ONLY
                   and "只判了选中的 2 行，不是整道结论" in selected[1].splitlines()[-1] and len(mutation_invocations(selected[3], pristine)) == 2,
                   selected[1])
    reused_selected = selftest.run(selected_repository, GATE_MUTATION_ROW_SELECTION=selection_file)
    selftest.check("选 2 行再跑一趟：照判定库复用，与头一趟现判出来的那两行的判定相同", reused_selected[0] == EXIT_SELECTION_ONLY
                   and reused_selected[3] == [] and "复用 2 条" in reused_selected[1], reused_selected[1])
    selftest_write(os.path.join(work, "bad-selection.txt"), "99999\n")
    bad_selection = selftest.run(repository, GATE_MUTATION_ROW_SELECTION=os.path.join(work, "bad-selection.txt"))
    selftest.check("选行文件里的行号不是一条变异：判红、一格都没跑", bad_selection[0] == 1 and bad_selection[3] == [], bad_selection[1])
    # 判定库那一份（九 扫它）
    source_library = library_of(repository)
    # 九：记录里不写主机名（判定库记录与解开的覆盖记录、组成清单；整份库经临时服务按前缀扫出来），判定库目录本地一个文件都没有
    host_name = os.uname().nodename
    record_texts = [gzip.decompress(data).decode("utf-8", "replace") if relative.endswith(".json.gz") else data.decode("utf-8", "replace")
                    for relative, data in selftest_library(source_library).service.scan("")]
    local_files = [name for _walked, _directories, names in os.walk(source_library) for name in names]
    selftest.check("记录里不写主机名", len(host_name) >= 3 and record_texts and not any(host_name in text for text in record_texts) and not local_files,
                   f"{host_name}；库里 {len(record_texts)} 条；库目录本地的文件 {local_files[:3]}")
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
    selftest.check("crates-mutation-replay 派活之前判的是同一份：同样几行判红、删行变异不报、假 cargo 一次都没起",
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
    """单元键几格（controls 路第 5 节 crates 变异那一行：S34、S30、N33、N34、N45；另加被改坏的单元、点名测试的闭包、V5 那一类常量、
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
    coverage_files = selftest_library(library_of(repository)).stored_names("coverage")
    selftest.check("头一趟每条点名测试记一份覆盖、同一条测试的几行共用一份（5 行 4 条测试：两行点名 checks_root）",
                   first[0] == 0 and len(coverage_files) == 4 and len(coverage_invocations(first[3])) == 8 and "行新加 5" in first[1]
                   and "覆盖：这一趟记了 4 份" in first[1], f"覆盖记录 {coverage_files}；{first[1]}")
    second = selftest.run(repository)
    selftest.check("输入不变的第二趟：5 行都复用、cargo 一次都不起", second[0] == 0 and second[3] == [] and fresh_and_reused(second)[1] == sorted(names), second[1])
    # plan：只读、逐行打、末行报总数
    library_before = selftest_library(library_of(repository)).service.scan("")
    plan = run_command_capturing(command_plan, [repository])
    library_after = selftest_library(library_of(repository)).service.scan("")
    selftest.check("plan：只读（库里一条都没多、没改）、逐行打复用还是现判、末行报总数",
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
    mismatched = selftest.run(selftest_new_library_copy(work, repository, "units-llvm-mismatched"), FAKE_LLVM_VERSION="0.0.0")   # 换一棵树：要现跑的行才查 llvm 工具
    selftest.check("llvm 工具的 LLVM 版本与 rustc 对不上：判红、一条都不跑", mismatched[0] == 1 and mismatched[3] == []
                   and "记覆盖要的 llvm 工具不对" in mismatched[1], mismatched[1] + mismatched[2])
    empty_tools = os.path.join(work, "no-llvm-tools")
    os.makedirs(empty_tools)
    missing = selftest.run(selftest_new_library_copy(work, repository, "units-llvm-missing"), GATE_MUTATION_LLVM_TOOLS=empty_tools)
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


SelftestSentinelRow = collections.namedtuple("SelftestSentinelRow", "line_number name")


def selftest_sentinel_uncounted_cell(selftest):
    """强制现判那一条本次未跑（notrun）：observe_fresh_outcomes 不把它当现判结论交哨兵，哨兵照「这一趟没报结局」打一行不判、不判成不稳。
    两条候选、哨兵 1 条，另一条就是强制现判；只喂结局，不起 run_stage。弄坏开关 uncounted-forced-observed 下照交，哨兵判「判定不稳」，这一格必须判错。"""
    rows = [SelftestSentinelRow(2, "root-457"), SelftestSentinelRow(3, "root-double")]
    rows_by_line = {row.line_number: row for row in rows}
    sentinel = reuse_checks.ReuseSentinel(SCRIPT_KEY, "crates 变异表", count=1, seed=5)
    planned = []
    sentinel.plan({sentinel_item(row): "抓到" for row in rows}, printer=planned.append)
    forced = [row for row in rows if sentinel.forced(sentinel_item(row))]
    for row in rows:
        if row not in forced:
            sentinel.observe(sentinel_item(row), reuse_checks.REUSED, "抓到")
    observe_fresh_outcomes(sentinel, [(row.line_number, "notrun", "", None) for row in forced], rows_by_line, selftest.base_environment)
    printed = []
    problems = sentinel.finish(printer=printed.append)
    expected = f"强制现判的 {sentinel_item(forced[0])} 这一趟没报结局（没跑到），不判" if forced else "（没抽出强制现判）"
    selftest.check("复用哨兵：强制现判那一条本次未跑（notrun）：不当现判结论交哨兵，打一行不判、不判成不稳",
                   len(forced) == 1 and not problems and any(expected in line for line in printed),
                   f"强制现判 {[sentinel_item(row) for row in forced]}；哨兵判 {problems}；打的 {printed}；抽的 {planned}")


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
    键从 row-plan 读（只读，不起 cargo）。弄坏开关 wgsl-whole-file（.wgsl 取整份字节的 sha256）下这一格必须判错。"""
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


def selftest_multiple_packages_cell(selftest, work):
    """cargo 参数里有多个 -p（lib_mutation_units.py 文件头「点名的测试」）：自证函数在 lib_mutation_units.py 的 multiple_packages_cell，这里登记、跑它。
    弄坏开关 multiple-packages-last-only（只认最后一个 -p）下这一格必须判错。"""
    passed, note = mutation_units.multiple_packages_cell(work)
    selftest.check("多个 -p：键按并集特性算、与 -p 的先后无关、改别的包的无关函数键不变、改共有包里点名测试够得着的函数键变", passed, note)


def selftest_specification_line_cell(selftest, work):
    """verus! 里的规格行（lib_mutation_units.py 文件头「视图」）：自证函数在 lib_mutation_units.py 的 specification_line_cell，这里登记、跑它。
    弄坏开关 specification-lines-product-view（规格行也取产品行为视图）下这一格必须判错。"""
    passed, note = mutation_units.specification_line_cell(work)
    selftest.check(mutation_units.SPECIFICATION_LINE_CELL, passed, note)


SELFTEST_TWIN_UNIT_SOURCE = '''// 撞键拒算那一格的样本 crate：两份内容相同的函数各在一个文件里，点名的测试两个都走；另一条测试只走 three
// TEST values_are_one FILES src/left.rs,src/right.rs { 1 } RUNS values_are_one,value
// TEST three_is_three NEEDS 1 + 2
pub mod left;
pub mod right;

pub fn three() -> u32 { 1 + 2 }

#[cfg(test)]
mod tests {
    #[test]
    fn values_are_one() { assert_eq!(super::left::value() + super::right::value(), 2); }
    #[test]
    fn three_is_three() { assert_eq!(super::three(), 3); }
}
'''
SELFTEST_TWIN_UNIT_ROWS = [
    "left-one-to-two\tcrates/tiny/src/left.rs\t{ 1 }\t{ 2 }\t-p tiny --lib -- values_are_one\tvalues_are_one",
    "right-one-to-two\tcrates/tiny/src/right.rs\t{ 1 }\t{ 2 }\t-p tiny --lib -- values_are_one\tvalues_are_one",
    "three-becomes-four\tcrates/tiny/src/lib.rs\t{ 1 + 2 }\t{ 1 + 3 }\t-p tiny --lib -- three_is_three\tthree_is_three",
]
SELFTEST_TARGET_TEST = "fn run() {{ assert_eq!(tiny::shared(), {0}); }}\n\n#[test]\nfn smoke() {{ run(); }}\n"
SELFTEST_TARGET_ROWS = [
    "alpha-one-to-three\tcrates/tiny/tests/alpha.rs\t(), 1)\t(), 3)\t-p tiny --test alpha -- smoke\tsmoke",
    "beta-two-to-three\tcrates/tiny/tests/beta.rs\t(), 2)\t(), 3)\t-p tiny --test beta -- smoke\tsmoke",
]


def selftest_row_key_collision_cell(selftest, work, environment):
    """撞键拒算（refuse_colliding_row_keys，文件头「行键」）：left.rs 与 right.rs 里内容相同的 value 各改一处、点名同一条测试的两行算出同一把行键，
    头一趟两行都拒算、照跑（各起一次变异）、成功句与 stderr 逐行报「拒算（行键撞键：与第 N 行同键）」，不撞键的 three-becomes-four 现判；
    第二趟两行照旧拒算、照跑，three-becomes-four 复用；只选 left 那一行（选行，别的行不算键）时照整张表判、仍拒算。
    弄坏开关 row-key-collision-unrefused（不判撞键）下这一格判错。"""
    twin = os.path.join(work, "reuse-fix-twin-units")
    build_selftest_repository(twin, SELFTEST_TWIN_UNIT_ROWS, source=SELFTEST_TWIN_UNIT_SOURCE)
    for name in ("left", "right"):
        selftest_write(os.path.join(twin, "crates", "tiny", "src", f"{name}.rs"), "pub fn value() -> u32 { 1 }\n")
    twin_rows = read_table(os.path.join(twin, TABLE), environment)[0]
    left, right = (row.line_number for row in twin_rows[:2])
    _context, _rows, alone, _library = plan_rows(twin, twin_rows[:1], environment, table_rows=twin_rows)
    first = selftest.run(twin)
    second = selftest.run(twin)

    def mutation_runs(invocations):
        """这一趟起的变异那几次（带 --show-output 的只有变异那一趟），按点名的测试数。"""
        return collections.Counter(word for line in invocations if "--show-output" in line.split("\t")[0].split()
                                   for word in ("values_are_one", "three_is_three") if word in line)
    wanted_reasons = [f"拒算（行键撞键：与第 {right} 行同键）：{TABLE}:{left} left-one-to-two", f"拒算（行键撞键：与第 {left} 行同键）：{TABLE}:{right} right-one-to-two"]
    first_how, second_how = per_row_how(first[1]), per_row_how(second[1])
    selftest.check("撞键拒算：left.rs 与 right.rs 里内容相同的函数各改一处、点名同一条测试的两行同键，两趟都拒算、照跑，成功句与 stderr 逐行报"
                   "「拒算（行键撞键：与第 N 行同键）」；不撞键的一行头一趟现判、第二趟复用；只选 left 那一行也照整张表判撞键",
                   first[0] == 0 and second[0] == 0
                   and first_how.get("left-one-to-two") == "拒算" and first_how.get("right-one-to-two") == "拒算" and first_how.get("three-becomes-four", "").startswith("现判")
                   and second_how.get("left-one-to-two") == "拒算" and second_how.get("right-one-to-two") == "拒算" and second_how.get("three-becomes-four") == "复用"
                   and mutation_runs(first[3]) == {"values_are_one": 2, "three_is_three": 1} and mutation_runs(second[3]) == {"values_are_one": 2}
                   and all(reason in first[2] and reason in second[2] for reason in wanted_reasons)
                   and f"其中撞键拒算 2 条：第 {left} 行拒算（行键撞键：与第 {right} 行同键）" in second[1] and "现判 0 条、复用 1 条、拒算 2 条" in second[1]
                   and alone[left].row_key is None and alone[left].refusal == f"{COLLISION_REFUSAL_PREFIX}与第 {right} 行同键",
                   f"头一趟退 {first[0]}、{first_how}、变异 {dict(mutation_runs(first[3]))}；第二趟退 {second[0]}、{second_how}、变异 {dict(mutation_runs(second[3]))}；"
                   f"只选 left 那一行：键 {alone[left].row_key}、{alone[left].refusal}；第二趟 {second[1][-400:]}{second[2][-600:]}")


ITEM_PLAN_LINE = re.compile(r"^ITEM_PLAN item=(?P<item>.*) to_judge=(?P<to_judge>[0-9]+) reusable=(?P<reusable>[0-9]+)$")
ITEM_DONE_LINE = re.compile(r"^ITEM_DONE item=(?P<item>.*) seconds=(?P<seconds>[0-9]+\.[0-9]+) judged=(?P<judged>[0-9]+) reused=(?P<reused>[0-9]+) exit=(?P<exit>[0-9]+)$")


def selftest_item_lines_cells(selftest, work):
    """按项机读行（文件头「按项机读行」）：
    ① 真跑：头一趟 4 项全现判，stderr 里每项恰好一行 ITEM_DONE（项名与表里的一一对上、judged=1 reused=0 exit=0），stdout 里没有这一行（stdout 与进程数无关）；
    ② 真跑：输入不变的第二趟全复用，仍是每项一行（judged=0 reused=1 exit=0 seconds=0.000）——弄坏开关 item-done-reused-silent 下一行都没有，这一格判错；
    ③ --list-items --with-keys：改了一行的替换文，4 项各一行 ITEM_PLAN，那一项 to_judge=1 reusable=0、别的 to_judge=0 reusable=1，条数之和等于末行合计的「键对不上」与「键对得上」，
       行数等于逐行列出的项数，不起 cargo——弄坏开关 item-plan-reusable-zero 下 reusable 一律 0，这一格判错；
    ④ 真跑：每项各记各的挂钟：假 cargo 每条改坏了的变异睡 1 秒（基线与覆盖不睡），1 个工作进程串行判 4 条，每项 seconds 在 1 到 1.9 之间——
       弄坏开关 item-done-seconds-whole-run 下记整趟，后几项超过 1.9，这一格判错。"""
    names = [row.split("\t")[0] for row in SELFTEST_TABLE_ROWS]

    def done_lines(run):
        return [match.groupdict() for match in map(ITEM_DONE_LINE.match, run[2].splitlines()) if match]

    repository = os.path.join(work, "item-lines")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    first = selftest.run(repository)
    first_done = done_lines(first)
    selftest.check("真跑：头一趟 4 项全现判，stderr 里每项恰好一行 ITEM_DONE（judged=1 reused=0 exit=0），stdout 里没有",
                   first[0] == 0 and sorted(line["item"] for line in first_done) == sorted(names)
                   and all(line["judged"] == "1" and line["reused"] == "0" and line["exit"] == "0" for line in first_done)
                   and "ITEM_DONE" not in first[1], f"退 {first[0]}；ITEM_DONE {first_done}；{first[2][-600:]}")
    second = selftest.run(repository)
    second_done = done_lines(second)
    selftest.check("真跑：全复用的第二趟每项一行 ITEM_DONE（judged=0 reused=1 exit=0 seconds=0.000）",
                   second[0] == 0 and sorted(line["item"] for line in second_done) == sorted(names)
                   and all(line["judged"] == "0" and line["reused"] == "1" and line["exit"] == "0" and line["seconds"] == "0.000" for line in second_done),
                   f"退 {second[0]}；ITEM_DONE {second_done}；{second[2][-600:]}")
    replace_in_file(os.path.join(repository, TABLE), "{ value * 3 }", "{ value * 4 }")
    with contextlib.suppress(FileNotFoundError):
        os.unlink(selftest.log)
    listed = run_command_capturing(command_list_items, ["--with-keys", repository])
    plan_text = [line for line in listed[1].splitlines() if line.startswith("ITEM_PLAN ")]
    plan = {match["item"]: (int(match["to_judge"]), int(match["reusable"])) for match in map(ITEM_PLAN_LINE.match, plan_text) if match}
    listed_items = [line for line in listed[1].splitlines() if "\t第 " in line]
    totals = re.search(r"键对不上 ([0-9]+) 条（这一趟要现判）、键对得上 ([0-9]+) 条（复用）、拒算 ([0-9]+) 条", listed[1])
    cargo_runs = open(selftest.log, encoding="utf-8").read().splitlines() if os.path.exists(selftest.log) else []
    selftest.check("--list-items --with-keys：每项一行 ITEM_PLAN（行数等于逐行列出的项数），改了替换文的那一项 to_judge=1、别的 reusable=1，条数之和等于末行合计，不起 cargo",
                   listed[0] == 0 and len(plan_text) == len(listed_items) == len(names) and len(plan) == len(names)
                   and plan == {name: ((1, 0) if name == "double-becomes-triple" else (0, 1)) for name in names}
                   and totals is not None and (sum(value[0] for value in plan.values()), sum(value[1] for value in plan.values())) == (int(totals[1]) + int(totals[3]), int(totals[2]))
                   and not cargo_runs, f"退 {listed[0]}；ITEM_PLAN {plan_text}；逐行 {len(listed_items)} 项；合计 {totals and totals.group(0)}；假 cargo 起过 {cargo_runs}；{listed[1][-500:]}")
    table_rows_now = read_table(os.path.join(repository, TABLE))[0]
    named_line = next(row.line_number for row in table_rows_now if row.name == "double-becomes-triple")
    selection_file = os.path.join(work, "item-lines-selection.txt")
    selftest_write(selection_file, f"{named_line}\n")
    only_one = run_command_capturing(command_list_items, ["--with-keys", "--rows", selection_file, repository])
    only_one_plan = [line for line in only_one[1].splitlines() if line.startswith("ITEM_PLAN ")]
    only_one_context = plan_context(repository, dict(os.environ), selection_file)
    selftest.check("--list-items --with-keys --rows：只打点名那一行的 ITEM_PLAN、只算那一行的键（别的行的单元闭包不读），合计只数这一行",
                   only_one[0] == 0 and only_one_plan == ["ITEM_PLAN item=double-becomes-triple to_judge=1 reusable=0"]
                   and len([line for line in only_one[1].splitlines() if "\t第 " in line]) == 1
                   and only_one_context is not None and len(only_one_context["components"]) == 1
                   and "键对不上 1 条（这一趟要现判）、键对得上 0 条（复用）" in only_one[1],
                   f"退 {only_one[0]}；ITEM_PLAN {only_one_plan}；算了键的行 {len(only_one_context['components']) if only_one_context else None}；{only_one[1][-400:]}")
    sleeping_repository = selftest_new_library_copy(work, repository, "item-lines-sleeping")   # 换一棵树：库是新的一份，4 条变异都要现判
    sleeping = selftest.run(sleeping_repository, FAKE_CARGO_MUTATION_SLEEP="1.0", FAKE_CARGO_BARRIER_PRISTINE=source_hash(sleeping_repository))
    sleeping_seconds = [float(line["seconds"]) for line in done_lines(sleeping)]
    selftest.check("真跑：每项各记各的挂钟（每条改坏了的变异睡 1 秒、串行判 4 条，每项 seconds 在 1 到 1.9 之间，不是整趟）",
                   sleeping[0] == 0 and len(sleeping_seconds) == len(names) and all(0.99 <= seconds <= 1.9 for seconds in sleeping_seconds),
                   f"退 {sleeping[0]}；每项的 seconds {sleeping_seconds}；{sleeping[2][-400:]}")


def selftest_reuse_fix_cells(selftest, work):
    """复用的几处修补（文件头「行键」「判定库」「基线」「复用判据」）：
    撞键拒算（selftest_row_key_collision_cell；弄坏开关 row-key-collision-unrefused 下判错）；
    基线键与覆盖号带包与测试目标（两个目标里字面相同的测试，基线键与覆盖号不同；baseline-key-without-target 下相同）；
    一行的基线结局不随这一批拼进了哪几行而变（核查报告那个复现：只选一行、两行一批，同一行同一个结论；baseline-batch-composition 下不同）；
    判法单元收全（改判法拷贝里 judge_row 的一处，判法单元指纹变、每一行都现判；judging-entries-verdict-only 下不变）；
    记红撤绿（先抓到入库，同一把键改判没红、基线不绿各一趟，下一趟现判；red-not-recorded、baseline-red-keeps-caught 下照复用）；
    不算数的结局不盖算数的（selftest_uncounted_keeps_counted_cells；uncounted-overwrites-counted 下判错）；
    库根（被判的树不是 git 工作树：退 1、点名它、一条都不跑；library-outside-git-tolerated 下照跑）。"""
    environment = dict(selftest.base_environment)
    selftest_row_key_collision_cell(selftest, work, environment)
    # 基线键与覆盖号带包与测试目标
    targets = os.path.join(work, "reuse-fix-test-targets")
    build_selftest_repository(targets, SELFTEST_TARGET_ROWS, source="pub fn shared() -> u32 { 1 }\n")
    for name, value in (("alpha", 1), ("beta", 2)):
        selftest_write(os.path.join(targets, "crates", "tiny", "tests", f"{name}.rs"), SELFTEST_TARGET_TEST.format(value))
    target_rows = read_table(os.path.join(targets, TABLE), environment)[0]
    _context, _rows, target_components, _library = plan_rows(targets, target_rows, environment)
    pairs = [(target_components[row.line_number].baseline_key, target_components[row.line_number].coverage_id) for row in target_rows]
    selftest.check("基线键与覆盖号带包与测试目标：tests/alpha.rs 与 tests/beta.rs 里字面相同的 smoke，两行的基线键、覆盖号都不同",
                   len(pairs) == 2 and all(all(pair) for pair in pairs) and pairs[0][0] != pairs[1][0] and pairs[0][1] != pairs[1][1], f"（基线键, 覆盖号）{pairs}")
    # 一行的基线结局不随这一批拼进了哪几行而变（核查报告的复现：两个模块各有一条 foo，一行点 a::foo、短名 foo，一行点 b::foo）
    one_to_three = Row(1, "one_to_three", "crates/demo/src/lib.rs", "1", "3", "-p demo --lib -- --exact a::foo", "foo", ())
    two_to_three = Row(2, "two_to_three", "crates/demo/src/lib.rs", "2", "3", "-p demo --lib -- --exact b::foo", "b::foo", ())
    single_row_output = "running 1 test\ntest a::foo ... ok\n\ntest result: ok. 1 passed; 0 failed\n"
    two_row_output = "running 2 tests\ntest a::foo ... ok\ntest b::foo ... ok\n\ntest result: ok. 2 passed; 0 failed\n"
    single_row = baseline_problem(one_to_three, 0, row_baseline_output(one_to_three, single_row_output, environment))
    two_rows = baseline_problem(one_to_three, 0, row_baseline_output(one_to_three, two_row_output, environment))
    other = baseline_problem(two_to_three, 0, row_baseline_output(two_to_three, two_row_output, environment))
    selftest.check("一行的基线结局不随这一批拼进了哪几行而变：one_to_three 一行一批与两行一批都判基线 ok，two_to_three 也 ok",
                   single_row is None and two_rows is None and other is None, f"一行一批 {single_row!r}、两行一批 {two_rows!r}、two_to_three {other!r}")
    # 判法单元收全：判法拷贝里改 judge_row 的一处，判法单元指纹变、每一行都现判（栏是判法单元变了）
    collected = os.path.join(work, "reuse-fix-judging")
    build_selftest_repository(collected, SELFTEST_TABLE_ROWS)
    collected_copy, collected_copied = judging_copy(work, "reuse-fix-judging-copy")
    selftest.run(collected, judging_repository=collected_copy)
    replace_in_file(os.path.join(collected_copy, "scripts", "mutation", "crates-mutation-rows.py"),
                    "    full_name = single_full_name(row.expected, output) or \"\"\n",
                    "    full_name = single_full_name(row.expected, output) or \"\" + \"\"\n")
    edited = selftest.run(collected, judging_repository=collected_copy)
    selftest.check("判法单元收全：改判法拷贝里 judge_row 的一处，每一行都现判（栏是判法单元变了）",
                   collected_copied and edited[0] == 0 and "复用 0 条" in edited[1] and f"判法单元变了 {len(SELFTEST_TABLE_ROWS)}" in edited[1], edited[1] + edited[2])
    # 记红撤绿：先抓到入库；同一把键（不改源码、不改表）经复用哨兵强制现判一行（哨兵取行数少一条，剩下那一条强制现判，其余照复用），判一趟没红，下一趟那一行现判；
    #   另一棵树先抓到入库（不跑基线，库里没有基线记录），同一把键强制现判一行、判一趟基线不绿，下一趟那一行现判（先跑基线）
    row_count = len(SELFTEST_TABLE_ROWS)
    one_forced = selftest_one_forced(row_count)
    withdrawn = os.path.join(work, "reuse-fix-red-withdraws")
    build_selftest_repository(withdrawn, SELFTEST_TABLE_ROWS)
    first = selftest.run(withdrawn)
    survived = selftest.run(withdrawn, FAKE_CARGO_FORCED_OUTCOME="ok", **one_forced)
    survived_name = selftest_forced_row_name(survived)
    after_survived = selftest.run(withdrawn)
    selftest.check("记红撤绿：先抓到入库，同一把键经哨兵强制现判一行、改判没红之后，下一趟那一行现判（栏是上次没判成抓到，不复用上次的抓到），别的行照复用",
                   first[0] == 0 and survived[0] == 1 and survived_name is not None and after_survived[0] == 0
                   and per_row_how(after_survived[1]).get(survived_name) == "现判：上次没判成抓到"
                   and f"现判 1 条、复用 {row_count - 1} 条" in after_survived[1], survived[2][-400:] + after_survived[1] + after_survived[2])
    baseline_withdrawn = os.path.join(work, "reuse-fix-baseline-red-withdraws")
    build_selftest_repository(baseline_withdrawn, SELFTEST_TABLE_ROWS)
    without_baseline = selftest.run(baseline_withdrawn, GATE_MUTATION_BREAK=with_break(selftest, "nobaseline"))
    baseline_red = selftest.run(baseline_withdrawn, FAKE_CARGO_FORCED_OUTCOME="FAILED", **one_forced)
    baseline_red_name = selftest_forced_row_name(baseline_red)
    after_baseline_red = selftest.run(baseline_withdrawn)
    selftest.check("记红撤绿：先抓到入库，同一把键经哨兵强制现判一行、判一趟基线不绿之后，下一趟那一行现判（先跑基线，不复用上次的抓到），别的行照复用",
                   without_baseline[0] == 0 and baseline_red[0] == 1 and "基线不绿 1 行" in baseline_red[1] and baseline_red_name is not None
                   and after_baseline_red[0] == 0 and per_row_how(after_baseline_red[1]).get(baseline_red_name) == "现判：上次没判成抓到"
                   and f"现判 1 条、复用 {row_count - 1} 条" in after_baseline_red[1] and len(plain_invocations(after_baseline_red[3])) == 2,
                   baseline_red[1] + after_baseline_red[1] + "\n".join(after_baseline_red[3]))
    selftest_uncounted_keeps_counted_cells(selftest, work)
    # 库根：被判的树没有 .git（rsync 拷的副本那样）：一条都不跑、点名它，不退回别处
    copy = os.path.join(work, "reuse-fix-library-copy")
    build_selftest_repository(copy, SELFTEST_TABLE_ROWS)
    shutil.rmtree(os.path.join(copy, ".git"))
    rooted = selftest.run(copy)
    selftest.check("库根：被判的树不是 git 工作树：退 1、点名它、一条都不跑",
                   rooted[0] == 1 and "不是 git 工作树" in rooted[1] + rooted[2] and copy in rooted[1] + rooted[2] and not plain_invocations(rooted[3]),
                   f"退 {rooted[0]}，假 cargo 真跑 {len(plain_invocations(rooted[3]))} 次：{rooted[1][-300:]}{rooted[2][-400:]}")


def selftest_uncounted_keeps_counted_cells(selftest, work):
    """不算数的结局不盖算数的（文件头「复用判据」记红撤绿那一段）：先抓到入库；同一把键经复用哨兵强制现判一行、记本次未跑（假 cargo 强令），
    库里每一行仍是抓到、下一趟照复用；另一棵没有记录的小仓同样本次未跑，照写本次未跑。
    弄坏开关 uncounted-overwrites-counted 下「库里一行抓到…现判出本次未跑之后」那一格判错。"""
    row_count = len(SELFTEST_TABLE_ROWS)

    def verdicts_in(repository):
        library = selftest_library(library_of(repository))
        return [library.read("records", name).get("verdict") for name in library.stored_names("records")]

    kept = os.path.join(work, "reuse-fix-uncounted-keeps-counted")
    build_selftest_repository(kept, SELFTEST_TABLE_ROWS)
    caught_first = selftest.run(kept)
    not_run_over = selftest.run(kept, FAKE_CARGO_FORCED_OUTCOME="notrun", **selftest_one_forced(row_count))
    verdicts_after_not_run = verdicts_in(kept)
    after_not_run = selftest.run(kept)
    selftest.check("不算数的结局不盖算数的：库里一行抓到，同一把键经哨兵强制现判出本次未跑之后，库里仍是抓到、下一趟照复用（现判 0 条）",
                   caught_first[0] == 0 and not_run_over[0] == 0 and "本次未跑：1 条" in not_run_over[1]
                   and verdicts_after_not_run == ["caught"] * row_count and after_not_run[0] == 0
                   and f"现判 0 条、复用 {row_count} 条" in after_not_run[1],
                   f"头一趟退 {caught_first[0]}；本次未跑那一趟退 {not_run_over[0]}；之后库里的判定 {verdicts_after_not_run}；"
                   f"下一趟退 {after_not_run[0]}：{after_not_run[1][-300:]}{not_run_over[1][-300:]}")
    fresh = os.path.join(work, "reuse-fix-uncounted-fresh")
    build_selftest_repository(fresh, SELFTEST_TABLE_ROWS)
    fresh_not_run = selftest.run(fresh, FAKE_CARGO_FORCED_OUTCOME="notrun")
    selftest.check(f"不算数的结局不盖算数的：库里没有这把键的记录时，本次未跑照行键写（{row_count} 条本次未跑的记录）",
                   fresh_not_run[0] == 77 and verdicts_in(fresh) == ["notrun"] * row_count,
                   f"退 {fresh_not_run[0]}；库里的判定 {verdicts_in(fresh)}")


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
        _context, _rows, found, _library = plan_rows(coverage_repository, coverage_rows[:1], environment, judging_root)
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


def selftest_judging_cache_cell(selftest, work):
    """判法单元指纹的磁盘缓存（文件头「判法单元」末段）：每一趟先清掉进程内那一份（像新进程）、数现算与取缓存各几趟：
    头一趟现算、第二趟不改任何东西取缓存、指纹相同；改判法单元里一个函数（libtest 结果行解析的一处）之后现算（缓存不被沿用）、指纹变；再一趟又取缓存、指纹同前；
    只改闭包外的函数体（派活的 run_pool）：读过的文件内容变了照样现算（按文件内容存），指纹不变。弄坏开关 judging-cache-unverified（有存的就沿用、不核内容）下这一格必须判错。"""
    environment = dict(selftest.base_environment, **{JUDGING_CACHE_DIRECTORY_VARIABLE: os.path.join(work, "judging-cache-cell")})
    copy, copied = judging_copy(work, "judging-cache-cell-copy")

    def trip():
        """像新进程那样算一趟：(指纹, 现算几趟, 取缓存几趟)。"""
        _JUDGING_CACHE.clear()
        before = dict(JUDGING_CACHE_STATS)
        key, _references = judging_units(environment, copy)
        return key, JUDGING_CACHE_STATS["computed"] - before["computed"], JUDGING_CACHE_STATS["reused"] - before["reused"]

    try:
        first, second = trip(), trip()
        replace_in_file(os.path.join(copy, "scripts", "gate", "lib_libtest_result_lines.py"),
                        "        if parsed.full_name == expected or parsed.full_name.endswith(\"::\" + expected):\n",
                        "        if expected == parsed.full_name or parsed.full_name.endswith(\"::\" + expected):\n")
        edited, edited_again = trip(), trip()
        replace_in_file(os.path.join(copy, "scripts", "mutation", "crates-mutation-rows.py"),
                        "    sequence_counter = multiprocessing.Value(\"i\", 0)\n", "    sequence_counter = multiprocessing.Value(\"i\", 1)\n")
        outside, outside_again = trip(), trip()
    except (JudgingUnitsRefused, OSError) as error:
        selftest.check("判法单元指纹的磁盘缓存", False, f"判法单元算不出：{error}")
        return
    selftest.check("判法单元指纹的磁盘缓存：第二趟不改东西取缓存、指纹同；改判法单元里一个函数缓存不被沿用、指纹变；改闭包外的函数体读过的文件变了照样现算、指纹不变",
                   copied and first[1:] == (1, 0) and second == (first[0], 0, 1) and edited[0] != first[0] and edited[1:] == (1, 0) and edited_again == (edited[0], 0, 1)
                   and outside[0] == edited[0] and outside[1:] == (1, 0) and outside_again == (outside[0], 0, 1),
                   f"头一趟 {first}；第二趟 {second}；改函数后 {edited}、再一趟 {edited_again}；改闭包外函数体后 {outside}、再一趟 {outside_again}"
                   "（每项是 (指纹, 现算几趟, 取缓存几趟)，要 头一趟现算 1 取 0、第二趟现算 0 取 1 且指纹同、改函数后现算 1 取 0 且指纹变、再一趟现算 0 取 1、改闭包外后现算 1 取 0 且指纹同前）")


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
    records_after_first = selftest_library(library_of(anchor_root)).stored_names("records")
    anchored_again = selftest.run(anchor_root)
    anchor_line = f"{TABLE}:6 anchor-missing：锚点失效——原文在 crates/tiny/src/lib.rs 里命中 0 次，要恰好 1 次（这一行没跑、不入库、不复用）"
    selftest.check("锚点失效按行判：那一行记锚点失效、别的行照跑照入库（计数报锚点失效 1 行、整道判红；第二趟那 4 行复用、锚点失效那一行照旧红）",
                   anchored[0] == 1 and len(mutation_invocations(plain_invocations(anchored[3]), anchor_pristine)) == 4
                   and "计数：抓到 4 条、没红 0 条" in anchored[1] and "无效 0 条、锚点失效 1 行（共 5 条）" in anchored[1]
                   and anchor_line in anchored[1] and "有几行锚点失效（锚点失效 1 行" in anchored[1] and len(records_after_first) == 4
                   and anchored_again[0] == 1 and anchored_again[3] == [] and "复用 4 条" in anchored_again[1] and anchor_line in anchored_again[1],
                   anchored[1] + anchored[2][-800:] + "\n".join(anchored[3]) + f"\n记录 {len(records_after_first)} 条\n" + anchored_again[1])
    # 派活之后锚点失效：基线那一趟跑完用例之后，假 cargo 把工作副本里 halves 那一行的原文改掉（造「开跑之后别的会话改了源码」）；
    # 那一行记锚点失效、不入库，别的 3 行照判照入库，整道判红，进程池不炸
    mid_root = os.path.join(work, "anchor-mid-run")
    build_selftest_repository(mid_root, SELFTEST_TABLE_ROWS)
    try:
        mid = selftest.run(mid_root, GATE_MUTATION_WORKERS="1", FAKE_CARGO_EDIT_SOURCE="{ value / 2 }\t{ value >> 1 }")
    except AssertionError as error:
        mid = (None, f"抛了 AssertionError：{error}", "", [])
    mid_record_count = len(selftest_library(library_of(mid_root)).stored_names("records"))
    selftest.check("派活之后锚点失效：那一行记锚点失效，别的行照判照入库",
                   mid[0] == 1 and "计数：抓到 3 条、没红 0 条" in mid[1] and "锚点失效 1 行（共 4 条）" in mid[1]
                   and "halve-becomes-third：锚点失效——派活之后源码被改了" in mid[1] and mid_record_count == 3,
                   mid[1] + mid[2][-800:] + f"\n记录 {mid_record_count} 条")
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
        # 判定库那几格写进 prune-server：只在临时服务上跑（scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service），
        # 不是就一格都不跑，不写真服务
        client = mutation_units.Library.client()
        service_problem = client.selftest_service_problem()
        if service_problem:
            print(f"  ✗ crates-mutation-rows.py 自证没跑：{service_problem}")
            print(f"     → 怎么办：{client.SELFTEST_SERVICE_REMEDY}")
            return 2
        return run_selftest()
    if arguments[:1] == ["--item"]:
        return command_item(arguments)
    commands = {"static-check": command_static_check, "judging-units": command_judging_units, "toolchain": command_toolchain,
                "library-directory": command_library_directory, "row-plan": command_row_plan, "plan": command_plan,
                "llvm-tools": command_llvm_tools, "sample-tools": command_sample_tools, "--list-items": command_list_items}
    if not arguments or arguments[0] not in commands:
        print("  ✗ 用法：crates-mutation-rows.py {static-check|judging-units|toolchain|library-directory|row-plan|plan|llvm-tools|sample-tools|--list-items|--item} … 或 --selftest",
              file=sys.stderr)
        print("     → 怎么办：各子命令的参数见文件头；整张表的复跑只经门禁 crates-mutation-replay.sh（bash .claude/gate.d/crates-mutation-replay.sh）", file=sys.stderr)
        return 2
    return commands[arguments[0]](arguments[1:])


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
