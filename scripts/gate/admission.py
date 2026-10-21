#!/usr/bin/env python3
# admission: always 判的是此刻的登记表、源码与判定库，门禁阶段与实验每一次开跑之前都要现判
# run-condition: none 它要的 git 与工具链版本由它自己读进单元键，读不到的按它的约定判（算不出的那一条按拒算现判、前提没齐退 3），不交给调用方预判
"""准入：门禁与实验开跑之前判「这一条判定该不该现判」。门禁与实验共用这一个模块、一张登记表。

复用的单位是一条判定（一条测试函数、一份自证、复跑表的一行、一条崩溃枚举用例日志的一项），不是一道门禁、也不是一格：
  单元键相同 ⇒ 不跑，沿用那一条判定。门禁各道在道内按判定入库，这个模块只给它们算单元键、查库、记库；
  实验：装置入口单元闭包的单元键与这一行的参数，与 research/results/ 里某份产物头记的相同就拒重跑。
  登记表 .claude/gate.d/stage-inputs.tsv 只答「从哪些入口起算」，整张表与登记行都不进任何键（D13（验证路线） 已定项 17）。
  不比树、不比提交、不比 HEAD：refs/sop/staged-green 只是 scripts/gate/gate-staged.sh 记的「上次整轮全绿的树」，这里一处都不读。

单元键（compute_unit_key）：下面这些行各以换行收尾、排好序连起来取 sha256：
  entry <入口闭包的键>   同目录 lib_function_units.py 的 closure_key：入口目标的生效 cfg 集、闭包里每个单元（种类、去注释指纹）的多重集、数据单元摘要、
                          工具链行、依赖版本行、闭包里点名的外部程序（cargo、herd7、busybox、qemu-system-*：内容 sha256 与 --version 第一行）
    工具链行（Rust 入口）  rustc -vV 每一行、cargo -V、构建环境的生效值（build_environment_setting_lines：环境变量 RUSTFLAGS、CARGO_ENCODED_RUSTFLAGS、CARGO_PROFILE_*、
                          CARGO_BUILD_*（CARGO_BUILD_JOBS 除外）、CARGO_TARGET_*_RUSTFLAGS / _RUNNER、RUSTC_WRAPPER、RUSTC_WORKSPACE_WRAPPER 的值；
                          闭包根往上各层与 CARGO_HOME 的 .cargo/config(.toml) 照 cargo 的合法合成之后逐键摊开的值（注释、空白、写的次序都不进）；
                          rust-toolchain(.toml) 摊开的值；runner / wrapper 指的程序与参数里的脚本的内容 sha256；RUSTC 的 -V），
                          请求带 profile:<名> 的另加「profile <名>」与清单 [profile.<名>] 摊开的值；release 这一类档的 cfg 不带 debug_assertions
    工具链行（Python、shell 入口）  python3 --version 或 bash --version 的第一行
    依赖版本行  闭包里有单元的工作区包，Cargo.toml 写的外部依赖（测试编译的入口另加它那个包的 dev-dependencies；本机目标不要的 target 段不收），
                顺着 Cargo.lock 收传递依赖（名字、版本、来源、checksum）；外部包的 Cargo.toml 在 cargo 缓存里找得到时，只在别的平台才要的依赖不收
  judge <判法入口闭包的键>   请求带 judge:<入口> 的：判这一条结局的那几个函数（例 scripts/gate/lib_libtest_result_lines.py::result_line）的闭包键，连同解释器版本
  tool <内容 sha256> <版本>   请求带 tool:<程序> 的
  line <文本>               请求带 line:<文本> 的：这一条判定自己的内容输入（这一行的参数、GPU 开关与特性集这一类），调用方不许把名字、路径、上限、限时写进来
  file <内容 sha256>        请求带 file:<仓根起的路径> 的：数据（期望产物这一类）只进内容，不进路径；不在也是一种取值
  environment <第一行>       请求带 environment-of:<登记键> 的：那一行 environment= 探针打的工具版本（QEMU、herd7、Verus 这类不在 cargo 工具链里的）
  不带路径、名字、行号、登记行、手写版本号、资源上限、限时、提交、树。组成清单（单元行：种类、限定名、文件:起止行、指纹；其余输入行）随键落盘，只当索引。
  拒算（lib_function_units.py 文件头「拒算」：闭包收不全的形态；另有 key-uncomputable：入口找不到、在这一组 cfg 下不编、工具链起不来）的那一条没有键，
  这一趟照跑、不复用、不入库；红句与成功句报拒算条数与头几处形态。

判定请求（verdict-plan 的请求文件一行一条，制表符分隔；unit-key 的参数同一套）：<名字><制表符><入口>[<制表符><项>…]
  名字是这一条判定在库里的索引（门禁写测试函数的入口原样、格名、自证的命令），不进键。入口照 lib_function_units.py 的写法，相对 root: 那个目录。项：
    root:<仓根起的目录>   闭包从哪棵树算（research 工作区写 root:scripts/experiments；不写是仓根）
    target:<目标号>        lib_function_units 的目标号 <包目录名>:<lib|bin|test|…>:<名>（不写照它的默认挑）
    feature:<包>/<特性>    no-default-features    cfg:<名或 名="值">    profile:<名>（cargo 的编译档，例 release）
    judge:<入口>   judge-root:<目录>（判法在哪棵树里算闭包：门禁写它自己所在的仓，样本档被判的是样本；不写是被判的根；只当索引，不进键）
    tool:<程序>   line:<文本>   file:<路径>   environment-of:<登记键>

判定库（VerdictLibrary）：<git common-dir>/singlefs-unit-verdicts/<种类>/，各 worktree 共用一份、跨轮留着，不随判法换目录、没有版本目录、不按个数删。
  种类由调用方定（门禁写登记键：阶段文件名或 <阶段>:<格>）。keys/<键>.json 是一条判定（只记绿：verdict、key、item、request、how（跑法）、summary、
  judged（判定日期）、composition（组成清单）），items/<名字的 sha256>.json 是这个名字最近一次判绿的那一份，只用来在键对不上时比组成清单、报出变了的单元。
  键相同就复用，不看记下来的是哪个名字（两条判定内容相同，判定就相同）。全量（--full、SINGLEFS_GATE_FULL=1）只写不读：现判的照键写回同一份库。
  同一种类目录下另有 flow-lines/<组成键>.lines：一个测试二进制整份现判时打的覆盖行与计数行，由 scripts/crash/crash-flow-coverage.py collect-quick 照它
  文件头「子命令」写、读（组成键是那个测试二进制里编得进的测试函数的（名字，键）排好序取 sha256），这一份不读不写它，删了只是下一趟整份复用的那几份报行不全。
  取不到 git common-dir（不在 git 工作树里）就不读不写，全部现判。旧的整道全绿标记（singlefs-stage-green.*）、崩溃枚举用例全绿标记（singlefs-crash-case-green.*）、
  环境记录（singlefs-gate-environment.*）整个弃用，这一份一样都不读，留在 common-dir 里的可以删。

复用哨兵（verdict-plan 与 test-plan 每一趟都插；规则照 scripts/gate/lib_reuse_checks.py 文件头「二」，判法是那里的 ReuseSentinel，这里不另写一份）：
  候选 = 此刻键算得出、库里 keys/<键>.json 记着判绿、记的键与此刻相同的请求（sentinel_candidates 直接读键文件，与 VerdictLibrary.lookup 分开查）。
  种子取 SINGLEFS_REUSE_SENTINEL_SEED（不设现取、打进输出），抽 SINGLEFS_REUSE_SENTINEL_COUNT 条（默认 3）当哨兵，再从剩下的抽 1 条强制现判：
    强制现判那一条不查库、决定写 judge、原因以「强制现判（复用哨兵抽中…」起头；哨兵查完库就记下它的结局，有一条不是 reuse ⇒「复用失灵」，verdict-plan 退 1；
    强制现判那一条的判定在 verdict-record 里比（判绿了与库里同为绿；没判绿的不在这里判）。状态存在 <计划文件>.reuse-sentinel.json（与 .observed），计划文件是 - 时不存。
  不抽（打一行「不抽哨兵」）：全量（--full、SINGLEFS_GATE_FULL=1）、没有判定库、被判的根下有 .unit-verdict-sample/（门禁样本档：样本钉死了现判与复用的条数）、
    同目录没有 lib_reuse_checks.py（拷进不带它的小树里跑的这一份）。键相同的不足 K 条报「哨兵不足 K 条」，不红。
  自证：reuse-three-questions（复用三问四趟，规范输出 REUSE_SELFTEST script=scripts/gate/admission.py …）、reuse-sentinel（哨兵被现判判红），在 run_reuse_three_question_cells。

按条复用失灵（不能复用就判红；调 verdict-plan / test-plan 的四处共用这一份判法：门禁 checker-tier-crash-point-replay 快档两格、code-tooling 的 research-script-selftests 格、
  harness-model-differential-and-scenarios 各格、checker-tier-research-build-and-replay 的 research-unit-tests 格）：
  调用方在 verdict-plan / test-plan 退非 0、退 0 而没打 PLAN 那一行、verdict-record 退非 0 时调 plan-failure，把那一趟的输出交给它；
  失灵 = 输出里有复用哨兵的红句（「复用失灵：」「判定不稳：」「…强制现判没起作用」那几行 ✗），或计划那两条退非 0、没打 PLAN。
  plan-failure 退 1 并打 ✗ 与出路（点名哪一条哨兵、或退了几，出路指到本文件头的哪一段）；调用方照旧全部现判跑完那一趟，那一道（那一格）结论判红。
  verdict-record 退非 0 而输出里没有哨兵红句的（记不进库）不算失灵：plan-failure 退 0，调用方照旧只报一句、下一趟照判。
  样本档造失灵：被判的根下有 .unit-verdict-sample/plan-failure（一行 exit-1 或 sentinel-reuse-failed）时，verdict-plan / test-plan 照它装失灵——
    exit-1 不算键、不查库、打一行 ✗ 退 1；sentinel-reuse-failed 照常算、照常打计划，再添一条「复用失灵：<头一条请求> 键相同却现判了（样本档造的假哨兵…）」退 1。
    只有门禁样本档放这份文件；写的不是这两样退 2。
  自证：plan-failure 那几格（run_plan_failure_cells：哨兵三种红句、退 1 / 2、没打 PLAN、记库退非 0 而没有红句、两种样本标记）。

登记表是 .claude/gate.d/stage-inputs.tsv（唯一登记位，门禁阶段与实验各占几行；写法在它文件头）。一行一条，制表符分隔：
    <键> <制表符> <入口，空格分隔；没有就写 -> [<制表符> <准入条件与跑法字段，空格分隔>] <制表符> #<为什么是这几个入口>
  键：门禁阶段写文件名（checker-tier-crates-mutation-replay.sh）；阶段里的一格写 <阶段文件名>:<格名>；实验写实验号（E142），同一个实验另有一种调用方式的写
      实验号加斜杠加方式名（E142/p8g-crates-dump）；崩溃枚举用例写 crash-case:<名>。
  入口（第二列）：<文件>::<限定名>（测试函数全路径、bin 的 main、判法函数）、<文件>（Python / shell 脚本的程序入口、litmus 文件）、
      tests-of:<包>（这个包 lib 与各测试目标里全部不标 ignore 的测试函数，test-functions 现列）、vm:<档名>（虚机档，门禁 checker-tier-qemu-device-streams 认）、
      @<别的键>（那个键登记的全部入口）；路径从仓根起。实验的入口在复跑表那一行之外另加进行键（准入键登记的装置入口）。崩溃枚举用例写 -，入口就是第三列的 test=。
  准入条件（第三列）：实验行认三种：
      question-row=<文件>#<行号>:<正则>          那份文件里第一列是 <行号> 的表格行，最后一格（状态）要匹配正则
      product-field=<实验键>:<行名>:<字段>=<值>   scripts/experiments/replay.sh 登记给那个实验键的产物里（准入键是它的那一行），
                                                  最后一行 `E7RESULT name=<行名>` 的 <字段> 要等于 <值>
      same-run-field=<行名>:<字段>=<值>           同一趟里先跑的那一段这一趟的输出（路径由调用方经环境变量 SINGLEFS_ADMISSION_SAME_RUN_PRODUCT 交来）里，
                                                  最后一行 `E7RESULT name=<行名>` 的 <字段> 要等于 <值>；不读 research/results/ 下的留存产物（用户 2026-10-18 定
                                                  「改成同一趟现判」：research/ 可以整个删掉）。变量没设、指的文件不在、没有那一行或字段、值不等，都算前提没齐（退 3）
    question-row 与 product-field 点名的文件（那份问题单、replay.sh 给那个实验键登记的留存产物）不在仓里，登记表自查（lint_registration_rows）判红：
      前提指的是可以被归档删掉的东西，登记就靠不住；上一段的结论改用 same-run-field 在同一趟里现判。
    门禁行认五种：
      command=<可执行名>                  PATH 里找得到它
      readwrite=<路径>                    那个路径在、可读也可写（相对路径从仓库根起）
      probe=<仓内脚本>[:<参数>…]          bash <根>/<脚本> <参数…> 退 0（工作目录是仓库根）
      environment=<仓内脚本>[:<参数>…]    不是前提：它打出来的工具版本经请求项 environment-of:<登记键> 进这一道每条判定的键
      copy-range=<路径>                   不是前提也不进键：门禁 checker-tier-crates-mutation-replay 拷进每片源码副本的范围（crates-mutation-rows.py 的 copied_paths
                                          经 resolve_input_paths 读它；目录写成带斜杠的前缀）
      正则、值、路径与参数里不许有空白（条件之间按空白切）。实验行写了门禁的种类、门禁行写了实验的种类，自查都判错。
  崩溃枚举用例行（键是 crash-case:<名>，门禁 checker-tier-crash-point-replay 逐条跑）第三列认八种：
      test=<包>:<测试目标>:<用例函数>     恰好一条：跑的是 cargo test --release -p <包> --test <测试目标> -- --include-ignored --exact <用例函数>；
                                         用例函数的每一处定义都要标 #[ignore]（不带 --ignored 的 cargo test 不跑它，重型测试闸按这一条认它）：
                                         同名的每一处 fn <用例函数>( 都算（cfg 二选一的、子模块里同名的也算），有一处没标就算没标；
                                         测试目标顺着 include!("<字面路径>")、#[path = "<字面路径>"] mod、mod <名>; 带进来的源文件一起判
                                         （include! 的参数不是字符串字面量、带进来的找不到、用 use 改名 include 族宏的，判不出，按没标算）；
                                         # 与 [ 之间许空白；找不到字面的定义（宏生成的用例）判错
      count-line=<前缀>                  日志里以「<前缀> 」开头的行恰好一行
      exhaustive=<前缀>                  那一行带 exhaustive=true（前缀要先登记成 count-line=）
      threads=<前缀>                     按那一行的 states= 找 CRASH_POINT_REPLAY_PARALLEL_FINISHED 行判工作线程（前缀要先登记成 count-line=）；
                                         那一行自己带 worker_threads= 与 slices= 的（CRASH_INJECTION_FINISHED 这一类）拿它自己判；
                                         那一行带 shards= 的（双机分片 merge 那一趟）逐片判，见 judge_threads_of_each_shard
      threads-variable=<环境变量名>      至多一条：用例读线程数的环境变量（没登记是 SINGLEFS_CRASH_POINT_REPLAY_THREADS）；crash-case-command 把它设成配的线程数
      features=<特性,特性…>              至多一条：用例要编进的 cargo 特性；crash-case-command 在 -p <包> 之后带 --features。多机配置（${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}，
                                         按 git common-dir 找）写了 ENABLE_GPU=1 的一律另加 gpu；最后的特性里有 gpu 且 ENABLE_GPU=1 的，整条经 scripts/crash/gpu-quiesce-run.sh 起，
                                         环境里交清场之后那一份显卡先后（GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED，没写取 GPU_CARDS_BY_PRIORITY，都没写清掉）、清掉 GPU_COMPUTATION_CLASSES 与退役的 SINGLEFS_GPU_CARDS_CONFIG；
                                         带 features= 的另设 BINDGEN_EXTRA_CLANG_ARGS（取 scripts/crash/crash-amplification-lib.sh 的 CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS）。
                                         不与 shard=across-machines 并存：分片的驱动脚本自己拼 cargo、不带特性
      shard=across-machines              至多一条：这条用例的全量在双机配置开着时两台分片（门禁 checker-tier-crash-point-replay --full 交给 scripts/crash/crash-point-replay-shard-run.sh）
      red-blocks=judged-by-test          至多一条：这条用例故障注入之后的红块是用例自己认下的（用例自己分清单、断言清单外的失败为空），门禁快档与 --full 都不按库里的红块判，
                                         照全量起用例按 cargo test 的退出码判（块那一行照 crash-case-blocks --ran 核）；值只认 judged-by-test。
                                         crash-case-judged-by-test 子命令答登记了没有

崩溃枚举用例的复用不在这里：用例一级没有复用标记，复用全部落在 crates 侧的崩溃状态块判定库（块键与库在 crates/singlefs-checker-tier/src/crash_identity.rs 文件头）。
  用例跑完每条流打一行 CRASH_CASE_BLOCKS blocks=<块数> reused=<键相同、库里有判定的块> fresh=<这一趟现判的块> red=<判红的块> refused=<键算不出、照判不入库的块>
  （每一行 blocks = reused + fresh + refused；多流的用例打几行，逐行收、按用例合计；
  接口照 crash-case-blocks 子命令判；crates 侧在用例源码里认得 SINGLEFS_CRASH_CASE_BLOCKS_QUERY 才算接上）：
    查询（SINGLEFS_CRASH_CASE_BLOCKS_QUERY=1）：用例只算块键、查库，一块都不判，fresh= 是库里没有键相同判定的块数，照常退 0；
      门禁快档按这一行判「这条用例的全部块都有键相同的判定且无红」：fresh、red、refused 都是 0 才算罩住，red 不是 0 判红，别的报本次未跑。
      登记了 red-blocks=judged-by-test 的用例快档不查询、--full 也不按红块判：照平常跑起用例、按 cargo test 的退出码判，块那一行只核在不在、数对不对得上（crash-case-blocks --ran）。
    只写不读（SINGLEFS_CRASH_LIBRARY_WRITE_ONLY=1，门禁 --start-over）：库不读，全部块现判，照键写回同一份库。
    平常跑（门禁 --full）：键相同的块复用、对不上的现判；red 不是 0 判红。

复跑表按行的准入键（scripts/experiments/replay.sh 里 ADMISSION_ROW_KEYS 那一段 heredoc；replay-row-keys、experiment 与 product-field= 都读它）：
  一行一条，制表符分隔：<准入键> <制表符> <复跑表那一行的前三列，原样抄：实验号|二进制|参数> <制表符> <产物头谁写>
    产物头谁写：device  那一行跑的装置自己问准入、把 E7INPUT 行打在产物最前面（装置入口传的键要与这里的键逐字相同）；
               replay  那一行跑的东西不接准入（crates 的导出 bin 这一类），replay.sh 把准入放行时给的 E7INPUT 行接在新产物最前面。
  没写在这里的行，准入键是实验号、产物头由装置写。键的实验号要与那一行的实验号相同；每一条恰好指到复跑表一行，同一行只许指一次。
  登记了准入的键被复跑表两行以上共用判错：准入只按一个键比产物，分不清哪一行对哪一份。

复跑表每一行的行键（replay-row-inputs；scripts/experiments/replay.sh 按行复用的键，算法 replay_row_request）：一行就是一条判定请求，入口是：
  装置    二进制写名字的行：scripts/experiments/Cargo.toml 里 [[bin]] name 是它的那一份 path 的 main（root:scripts/experiments、target 那个 bin）；
          二进制写 @<驱动函数> 的行：replay.sh 里那个函数（shell 单元闭包），连同驱动闭包里字面点名的 research 二进制的 main 与
          `-p <包> … --bin <名>` 点名的仓根工作区 bin 的 main（包是 scripts/experiments/Cargo.toml 那个研究包的，--bin 的名字解到研究工作区 [[bin]] 的源码，
          root:scripts/experiments）；`-p <包>` 点名了包却点不出 bin 的那一行拒算（replay-package-without-binary）
  登记    这一行的准入键在登记表里登记的入口（入口另加进来，不是登记行原文）
  判法    judge:scripts/experiments/replay.sh::replay_judge_fresh_product 与比对前删字段的几个函数（REPLAY_JUDGING_FUNCTIONS）
  内容    line:arguments=<这一行的参数>、line:compare=<exact|timing>、驱动闭包里点到 CHECKER_TIER_FEATURES 的行另加 line:gpu=<0|1>（replay.sh 开跑前读多机配置给）
          与 checker 档的 feature:singlefs-checker-tier/verdict-store、gpu；file:research/results/<留存产物>（期望产物的内容）
  实验号、产物文件名不进键。实验的准入（experiment）用的是同一把行键去掉期望产物那一项：产物头记着键，产物进了键就对不上自己。

子命令（<根> 是被判仓的根）：
  entries <根> <键>              列出登记给这个键的入口（@ 引用已展开），一行一条；paths 是它的旧名，只打入口所在的文件（去重）
  test-functions <根> <包> [--root <目录>] [--feature <包>/<特性>]… [--profile <名>]
                                 这个包 lib、各 bin 与各测试目标里的测试函数，一条一行：<入口><制表符><目标号><制表符><目标根文件（包目录起）>
                                 <制表符><libtest 认的全名><制表符><compiled|not-compiled|ignored>（not-compiled：这一组特性下 cfg 关着）
  test-plan <根> <种类> <包> <计划文件> [--root <目录>] [--feature <包>/<特性>]… [--profile <名>] [--token <项>]… [--library-root <根>] [--full]
                                 test-functions 与 verdict-plan 在一个进程里做（闭包只收一次）：每条编得进的测试函数一条判定（请求带 target:<目标号> 与 --token 给的项），
                                 一条一行「<名字><制表符><reuse|judge|refused|not-compiled><制表符><键或 -><制表符><目标号><制表符><目标根文件><制表符><libtest 全名><制表符><原因>」，
                                 末行「PLAN judge=… reuse=… refused=… not-compiled=… ignored=…」；名字是入口，同一个函数编进几个目标的另接「@<目标号>」。
                                 门禁 checker-tier-crash-point-replay（快档两格）与 checker-tier-research-build-and-replay（research-unit-tests）照它按条跑 cargo test
  unit-key <根> <入口> [<项>…]   打「key <键>」与组成清单；拒算打拒算行、退 4
  verdict-plan <根> <种类> <请求文件> <计划文件> [--library-root <根>] [--full]
                                 逐条算键、查库：stdout 一条一行「<名字><制表符><reuse|judge|refused><制表符><键或 -><制表符><原因>」，末行
                                 「PLAN judge=<N> reuse=<M> refused=<R>」；计划文件记下请求与键，交给 verdict-record。--library-root 给了就查那棵树的库（样本档拿上一版的树算、记进这一版的库）。
                                 复用哨兵（下面「复用哨兵」）那几行打在逐条的行之后、PLAN 之前；哨兵被现判时 PLAN 之后另打 ✗ 与出路、退 1
  verdict-record <根> <种类> <计划文件> <判绿的名字文件> --how <跑法> [--library-root <根>] [--summary <一句>]
                                 名字文件里每个名字（一行一个）照计划文件里的请求重算一次键：与计划里的相同才记（跑的过程中输入变了的不记、点名报出来）；
                                 末行「RECORD recorded=<N> changed=<K> refused=<R>」
                                 计划文件旁边有 verdict-plan 存的复用哨兵状态（<计划文件>.reuse-sentinel.json）就在 RECORD 之前打哨兵的判定：
                                 强制现判那一条判绿了报与库里同为绿；没判绿的（红了或没跑到）只说一句，它的红由门禁这一趟自己报
  plan-failure <调用方> <verdict-plan|test-plan|verdict-record> <退出码> [<输出文件>…]
                                 那一趟按条复用失灵没有（上面「按条复用失灵」）：失灵退 1、打 ✗ 与出路（<调用方> 写进红句，例「code-tooling 的 research-script-selftests 格」）；
                                 没失灵退 0、一个字不打；用法错退 2
  gate-reuse <根> <键> [<请求文件>]
                                 门禁的复用答法：给了请求文件按它逐条算键，没给按库里那个名字最近一次判绿时的请求重算；登记给这个键的入口
                                 （tests-of: 展开成测试函数）每一条都有键相同的判定 ⇒ 退 10 可跳过；否则退 0 要跑，stdout 报要现判的入口清单与「键对不上的条数」。
                                 模块自己出错一律退 0（按要跑处理）。不读 refs/sop/staged-green、不比树
  gate-preconditions <阶段所在的仓根> <阶段文件名>
                                 门禁开跑前判第三列的前提（command=、readwrite=、probe=）：退 0 齐了（stdout 一个字不打）；
                                 退 1 没齐、退 2 登记有错，两种都在 stdout 打 ✗、明细与出路，阶段照判红
  experiment <根> <实验键>       实验的准入：退 0 放行，stdout 是要写在产物最前面的几行（E7INPUT 开头，sha256= 记的是这一行的实验键）；
                                 退 77 research/results/ 里有一份产物头上记着同一个键；退 3 前提没齐；退 2 登记或调用有错。说明一律打到 stderr
  keys <根>                      列出登记表里的实验键，一行一个
  replay-row-keys <根>           复跑表每一行打一行「<前三列><制表符><准入键><制表符><产物头谁写><制表符><登记了准入：yes|no>」，按表里的次序
  replay-row-inputs <根> [--gpu 0|1] [--library <判定库目录>] [--compositions <目录>] [实验号…]
                                 复跑表每一行（给了实验号只算那几个实验的行）打一行「<前三列><制表符><行键或 refused><制表符><组成清单的单元数><制表符><说明>」，
                                 按表里的次序；说明是拒算的头几处，或给了 --library 时与那一行上一次判过的组成清单比出来的变了的单元。
                                 --compositions 给了就把每把键的组成清单写成 <目录>/<键>.tsv（replay.sh 记库时拷进那一条判定）
  crash-cases <根>               核登记的崩溃枚举用例（第三列的写法、包与测试目标在不在、用例函数在不在、它标没标 #[ignore]），每条打一行
                                 「<键><制表符><包><制表符><测试目标><制表符><用例函数>」；登记有错退 2，原因打在 stdout（不带 ✗，阶段自己打 ✗ 与出路）
  crash-case-command <根> <键> [--start-over] [--blocks-query]
                                 跑这一条的命令与环境：stdout 以 NUL 分隔，本机核数、线程数、explicit|default，其后是整条命令
                                 （env 设 SINGLEFS_CRASH_POINT_REPLAY_THREADS，登记了 threads-variable= 的另把那个变量设成同一个数；
                                 清掉调用方环境里的 SINGLEFS_CRASH_AMPLIFICATION_LIBRARY（用例照长期库）与分片的两个变量；--start-over 另设 SINGLEFS_CRASH_LIBRARY_WRITE_ONLY=1；
                                 --blocks-query 另设 SINGLEFS_CRASH_CASE_BLOCKS_QUERY=1；再起 cargo test --release …，features= 与 GPU 照上面「崩溃枚举用例行」）。
                                 线程数：调用方设了 SINGLEFS_CRASH_POINT_REPLAY_THREADS 用它（explicit），没设取每台默认重活线程数（default：SINGLEFS_THREAD_CAP 设了照它，否则
                                 .claude/heavy-thread-default 的登记值）；本机核数另取 os.cpu_count() 与 CPU 亲和的小者（判「本机多于 1 核」），不认 OMP_NUM_THREADS。交不出退 2
  crash-case-judge <根> <键> <日志> <记录行文件>
                                 判 --full 跑那一条的日志：退 0 判绿（日志里认出的行写进记录行文件，stdout 是线程那一句）；退 1 判红（stdout 逐条原因）。
                                 本机核数、线程数与是不是显式设的由它现取；日志里线程来源是用例自己取的默认（worker_threads_source=available_parallelism）
                                 而配的线程数不等于每台默认重活线程数的，判红
  crash-case-blocks <根> <日志> [--query | --ran]
                                 判日志里的 CRASH_CASE_BLOCKS 行（每条流一行，逐行收、按用例合计）：退 0 全部块都有键相同的判定且无红（--query）或无红（平常跑）；
                                 退 1 有红的块；退 3 --query 下有块没有判定或拒算（本次未跑）；退 4 一行都没有、有一行缺字段或数对不上（每一行 blocks = reused + fresh + refused）。
                                 --ran（门禁快档实跑与 --full 跑登记了 red-blocks=judged-by-test 的用例那一趟）：只核那一行在、每一行数对得上，红块与现判块不判：
                                 退 0 对得上，退 4 一行都没有、缺字段或数对不上；--query 与 --ran 同给退 2。
                                 crates 侧还没接上（crash_case_blocks_supported：用例源码里没有 SINGLEFS_CRASH_CASE_BLOCKS_QUERY）退 5，门禁按本次未跑报
  crash-case-shardable <根> <键>  这条用例登记了 shard=across-machines 没有：退 0 登记了、退 1 没登记（stdout 各一句）、退 2 登记有错
  crash-case-judged-by-test <根> <键>
                                 这条用例登记了 red-blocks=judged-by-test 没有：退 0 登记了、退 1 没登记（stdout 各一句）、退 2 登记有错
  --selftest                     自证（格数由成功行现算）

强制重跑：设 SINGLEFS_EXPERIMENT_RERUN_REASON=<理由>。只越过「输入没变」，不越过前提；理由原样写进产物头。
弄坏开关（ADMISSION_BREAK=<项>[,<项>…]，只给自证与样本档证红用；自证逐个打开，核点名的那几格转红）：
  experiment-key-with-kb-documents  .claude/kb/decisions/ 下的文件整份进键（改前实验准入收 kb 决策文档的样子）：自证 T19 那一格（改 kb 决策文档的一句，实验键不变）红
  tree-compare-reuse     gate-reuse 退回按树比：refs/sop/staged-green 那棵树（没有它取 HEAD 的树）与工作区在登记入口的文件上逐字相同就答可跳过，不查判定库：
                         自证 H2 那一格（改动带着 HEAD 一起前移、staged-green 跟着挪，键对不上的那一条必须现判）红
  skip-preconditions     实验不判前提          skip-gate-preconditions  门禁不判前提
  same-run-field-unchecked  same-run-field= 不读同一趟的输出、一律当齐了：自证「same-run-field：同一趟的输出里值不等」那一格红
  condition-files-unchecked 登记表自查不核 question-row=、product-field= 点名的文件在不在：自证「登记表自查点得出前提点名的文件不在」那一格红
  skip-build-environment 构建环境的生效值不进工具链行：自证 E02（RUSTFLAGS 加一个 -C target-cpu=native，键要变）红
  ignore-row-keys        复跑表按行的准入键那一段不认：自证「复跑表按行的准入键」那几格红
  row-key-with-product-name  行键另加留存产物的文件名：门禁 checker-tier-research-build-and-replay 的样本 replay-row-key-samples 里 R03（只改产物文件名，行键不变）红
  row-key-without-gpu    行键不加 GPU 开关：自证 E22（切 ENABLE_GPU，点到 CHECKER_TIER_FEATURES 的那一行要现判）红
  research-package-unresolved  驱动点名 -p <研究包> --bin <名> 时照仓根工作区去找 bin（改前的样子）：自证「驱动点名 -p e7-index-bench --bin e900」那一格红（E900 拒算 replay-package-without-binary）
  test-key-with-whole-file  test-plan 给每条测试函数的键另加它所在的整份文件（门禁 checker-tier-crash-point-replay 的 GATE_CRASH_POINT_REPLAY_BREAK=setting-file-whole-closure 转成它）：
                         那一道的样本 unit-key-samples-green 里 N19（只改设点文件里 test_u 的一个字，同文件的 test_v 不作废）红；这里的自证不点它
  dependencies-by-crate  外部依赖照包收（闭包碰到一个工作区包就收它 Cargo.toml 写的全部外部依赖，不看单元点没点它的名字）：自证 S14 形态那一格（itoa 抬版本只 test_v 变）红
  blocks-count-unchecked crash-case-blocks 不核 blocks = reused + fresh + refused：自证「数对不上」那一格红
  blocks-count-without-refused crash-case-blocks 照改前核 blocks = reused + fresh（拒算的块不算进去）：自证「平常跑：拒算 1 块计在 refused」那一格红
  blocks-single-line-only crash-case-blocks 照改前要日志里恰好一行：自证「多流用例每条流一行」那一格红
  ran-blocks-line-optional crash-case-blocks --ran 日志里一行 CRASH_CASE_BLOCKS 都没有也退 0：自证「--ran：日志里没有那一行」那一格红；
                         门禁 checker-tier-crash-point-replay 的样本 quick-judged-by-test-missing-blocks-line 判错（期望退出 1，实测 3）
  verdict-key-with-entry-file  verdict-plan 每条请求的键另加入口所在的整份文件（闭包外的改动也作废）：自证「弄坏开关 verdict-key-with-entry-file 下 reuse-three-questions 的 ② 转红」那一格
                         在自己那一串里打开它、核 ② 红；外面设了它，reuse-three-questions 那一格的 ② 同样红（③ 跟着红：闭包外那一趟改过的入口文件让三条都对不上记录）
  verdict-lookup-skipped VerdictLibrary.lookup 不读库、一律现判（复用失灵的样子）：自证 reuse-sentinel 那一格在自己那一趟里打开它、核「复用失灵」退 1；
                         外面设了它，reuse-three-questions 那一格 ①②③ 全红
  共用库的 REUSE_CHECKS_BREAK=rejudged-sentinel-passes（哨兵被现判不红）：自证 reuse-sentinel 那一格判错
  plan-failure-not-red   plan-failure 认出失灵也退 0、只打一行 !（改前四处调用方退回全部现判、不判红的样子）：自证 plan-failure 里判红的那几格转错；
                         四处调用方的样本里造失灵的那几份（*-plan-exit-red、*-plan-sentinel-red）照样跑完、不判红
  以及崩溃枚举用例日志与命令那几格的老开关（threads-by-worker-count、first-definition-only、ignore-attribute-without-space、single-worker-threads-field、
  threads-ignore-shards、keep-caller-library、target-own-files-only、judge-takes-forwarded-threads、cores-from-nproc、thread-variable-ignored、features-not-passed、
  features-passed-to-every-case、gpu-case-not-quiesced、gpu-test-threads-unset、threads-skip-self-contained-finish-line、threads-default-unchecked、threads-default-from-cores、
  runner-program-only、runner-arguments-from-configuration-directory-only），各弄坏哪一处写在用到它的那个函数里，自证里各有一格。

管不到的：跑的是不是按今天的源码编出来的二进制（键按源码算，跑的是 target/ 里的旧二进制时两边对不上，cargo test 与 replay.sh 开跑前都会重编）；
  闭包收全没收全以 lib_function_units.py 为准（它文件头「管不到的」）：收不全而没认出来的形态，改了那一处不让判定作废。
  外部依赖按包收：闭包碰到一个工作区包，那个包 Cargo.toml 写的外部依赖全收，不看闭包里的单元真用到哪一个；外部包的 Cargo.toml 不在 cargo 缓存里时，
  只在别的平台才要的依赖照收（多作废，接受）。gate-reuse 不给请求文件时按库里记的请求重算，门禁这一趟的请求项变了（GPU 开关、特性）它看不见：
  调 gate-reuse 的门禁要给请求文件。
"""
import ast
import collections
import copy
import glob
import gzip
import hashlib
import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

REGISTRATION_TABLE = ".claude/gate.d/stage-inputs.tsv"
REPLAY_SCRIPT = "scripts/experiments/replay.sh"
RESULTS_DIRECTORY = "research/results"
GATE_GREEN_TREE_RECORD = "refs/sop/staged-green"   # 只有弄坏开关 tree-compare-reuse 读它（改前的复用路）；平常一处都不读
FORCE_RERUN_VARIABLE = "SINGLEFS_EXPERIMENT_RERUN_REASON"
BREAK_VARIABLE = "ADMISSION_BREAK"
HEADER_PREFIX = "E7INPUT "
HEADER_LINES_READ_AT_MOST = 20
FUNCTION_UNITS_LIBRARY = "lib_function_units.py"
VERDICT_LIBRARY_DIRECTORY_NAME = "singlefs-unit-verdicts"
CHANGED_UNITS_SHOWN_AT_MOST = 8

EXIT_ADMITTED = 0
EXIT_REGISTRATION_ERROR = 2
EXIT_PRECONDITION_MISSING = 3
EXIT_INPUTS_UNCHANGED = 77
EXIT_GATE_MUST_RUN = 0
EXIT_GATE_MAY_SKIP = 10
EXIT_GATE_PRECONDITION_MISSING = 1
EXIT_UNIT_KEY_REFUSED = 4
PROBE_TIMEOUT_SECONDS = 300
GATE_PRECONDITION_KINDS = ("command", "readwrite", "probe")

EXPERIMENT_KEY_FORM = re.compile(r"^E[0-9]+[A-Z0-9]*(/[a-z0-9][a-z0-9-]*)?$")
QUESTION_ROW_FORM = re.compile(r"^question-row=(?P<path>[^#]+)#(?P<row>[^:]+):(?P<pattern>.+)$")
PRODUCT_FIELD_FORM = re.compile(r"^product-field=(?P<key>[^:]+):(?P<line_name>[^:]+):(?P<field>[^=]+)=(?P<value>\S+)$")
REPLAY_ROW_FORM = re.compile(r"^(E[0-9]+[A-Z0-9]*)\|([^|]*)\|([^|]*)\|([^|]*)\|(exact|timing)$")
REPLAY_ROW_KEYS_START = "ADMISSION_ROW_KEYS=$(cat <<'TSV'\n"
REPLAY_ROW_KEYS_END_LINE = "TSV"
REPLAY_ROW_HEADER_WRITERS = ("device", "replay")
GATE_CONDITION_FORM = re.compile(r"^(?P<kind>command|readwrite|probe|environment|copy-range)=(?P<value>\S+)$")
STAGE_CELL_KEY_FORM = re.compile(r"^[a-z0-9][a-z0-9-]*\.sh:[a-z0-9][a-z0-9-]*$")
ENTRY_FORM = re.compile(r"^(?:tests-of:[A-Za-z0-9_-]+|vm:[a-z0-9][a-z0-9-]*|@[^\s]+|[A-Za-z0-9_./+-]+(?:::[^\s]+)?)$")

# 崩溃枚举用例（门禁 checker-tier-crash-point-replay 逐条跑）
CRASH_CASE_KEY_PREFIX = "crash-case:"
CRASH_CASE_KEY_FORM = re.compile(r"^crash-case:[a-z0-9][a-z0-9-]*$")
CRASH_CASE_CONDITION_FORM = re.compile(r"^(?P<kind>test|count-line|exhaustive|threads|threads-variable|shard|features|red-blocks)=(?P<value>\S+)$")
CRASH_CASE_FEATURES_FORM = re.compile(r"^[a-z0-9][a-z0-9_-]*(,[a-z0-9][a-z0-9_-]*)*$")
VERDICT_STORE_FEATURE = "verdict-store"
GPU_FEATURE = "gpu"
MULTI_HOST_CONFIGURATION_VARIABLE = "SINGLEFS_MULTI_HOST_CONFIG"
MULTI_HOST_CONFIGURATION_NAME = "multi-host.env"
GPU_QUIESCE_SCRIPT = "scripts/crash/gpu-quiesce-run.sh"
BINDGEN_LIBRARY_SCRIPT = "scripts/crash/crash-amplification-lib.sh"
BINDGEN_VARIABLE = "BINDGEN_EXTRA_CLANG_ARGS"
BINDGEN_ASSIGNMENT_FORM = re.compile(r'^CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS="([^"]*)"', re.MULTILINE)
THREADS_VARIABLE_FORM = re.compile(r"^[A-Z_][A-Z0-9_]*$")
CRASH_CASE_SHARD_ACROSS_MACHINES = "across-machines"
# 登记行的 red-blocks=：故障注入之后的红块由用例自己认下，门禁快档与 --full 起用例按退出码判（文件头「崩溃枚举用例行」）
CRASH_CASE_RED_BLOCKS_JUDGED_BY_TEST = "judged-by-test"
CRASH_LIBRARY_DIRECTORY_VARIABLE = "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY"
CRASH_SHARE_VARIABLE = "SINGLEFS_CRASH_AMPLIFICATION_SHARE"
CRASH_IMPORT_VARIABLE = "SINGLEFS_CRASH_AMPLIFICATION_IMPORT"
CRASH_LIBRARY_WRITE_ONLY_VARIABLE = "SINGLEFS_CRASH_LIBRARY_WRITE_ONLY"
CRASH_CASE_BLOCKS_QUERY_VARIABLE = "SINGLEFS_CRASH_CASE_BLOCKS_QUERY"
CRASH_CASE_BLOCKS_PREFIX = "CRASH_CASE_BLOCKS "
CRASH_CASE_BLOCKS_FIELDS = ("blocks", "reused", "fresh", "red", "refused")
CRASH_CASE_BLOCKS_SOURCE_DIRECTORY = "crates/singlefs-checker-tier/src"
EXIT_BLOCKS_RED = 1
EXIT_BLOCKS_NOT_COVERED = 3
EXIT_BLOCKS_LINE_BAD = 4
EXIT_BLOCKS_UNSUPPORTED = 5
CRASH_CASE_TEST_FORM = re.compile(r"^(?P<package>[A-Za-z0-9_-]+):(?P<target>[A-Za-z0-9_]+):(?P<function>[A-Za-z_][A-Za-z0-9_]*)$")
COUNT_LINE_PREFIX_FORM = re.compile(r"^[A-Z][A-Z0-9_]*$")
CRASH_POINT_REPLAY_PARALLEL_FINISHED_PREFIX = "CRASH_POINT_REPLAY_PARALLEL_FINISHED "
PASSED_ONE_TEST_FORM = re.compile(r"^test result: ok\. 1 passed; 0 failed; ")
POSITIVE_INTEGER_FORM = re.compile(r"[1-9][0-9]*")
CRASH_POINT_REPLAY_THREADS_VARIABLE = "SINGLEFS_CRASH_POINT_REPLAY_THREADS"
HEAVY_THREAD_DEFAULT_FILE = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))), ".claude", "heavy-thread-default")
THREAD_CAP_VARIABLE = "SINGLEFS_THREAD_CAP"
HEAVY_THREAD_COMMENT_OR_BLANK_FORM = re.compile(r"[ \t]*(#|$)")
HEAVY_THREAD_REGISTRATION_FORM = re.compile(r"[ \t]*([1-9][0-9]*)[ \t]+#[ \t]*[^ \t]")
DEFAULT_WORKER_THREAD_SOURCES = ("available_parallelism",)
IGNORE_ATTRIBUTE_FORM = re.compile(r"^#\s*\[\s*ignore\s*(?:\]|=)")
IGNORE_ATTRIBUTE_FORM_WITHOUT_SPACE = re.compile(r"^#\[\s*ignore\s*(?:\]|=)")  # 弄坏开关 ignore-attribute-without-space 换回的旧写法
ATTRIBUTE_START = re.compile(r"#!?\s*\[")
FUNCTION_QUALIFIER_AT_END = re.compile(r'(?:\bpub(?:\s*\([^()]*\))?|\basync|\bunsafe|\bconst|\bextern(?:\s*"[^"]*")?)$')
TEST_ATTRIBUTE_FORM = re.compile(r"#\s*\[\s*test\s*\]")
TEST_IGNORE_FORM = re.compile(r"#\s*\[\s*ignore\b")

# 构建环境的生效值（build_environment_lines）
CARGO_CONFIGURATION_FILE_NAMES = ("config", "config.toml")
TOOLCHAIN_FILE_NAMES = ("rust-toolchain", "rust-toolchain.toml")
BUILD_ENVIRONMENT_EXACT_VARIABLES = ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER")
BUILD_ENVIRONMENT_VARIABLE_FORMS = (re.compile(r"^CARGO_PROFILE_"), re.compile(r"^CARGO_BUILD_"),
                                    re.compile(r"^CARGO_TARGET_.+_RUSTFLAGS$"), re.compile(r"^CARGO_TARGET_.+_RUNNER$"))
BUILD_ENVIRONMENT_VARIABLES_LEFT_OUT = {"CARGO_BUILD_JOBS": "只定编译并行度；scripts/hosts/capped.sh 与 mutate.sh 会设它"}
BUILD_ENVIRONMENT_PROGRAM_VARIABLES = ("RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_RUSTC_WRAPPER", "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER")
BUILD_ENVIRONMENT_PROGRAM_VARIABLE_FORMS = (re.compile(r"^CARGO_TARGET_.+_RUNNER$"),)
BUILD_ENVIRONMENT_COMPILER_VARIABLES = ("RUSTC", "CARGO_BUILD_RUSTC")
CONFIGURATION_PROGRAM_BUILD_KEYS = ("rustc-wrapper", "rustc-workspace-wrapper")

# 复跑表的行键（replay_row_request）
REPLAY_RESEARCH_DIRECTORY = "scripts/experiments"
REPLAY_JUDGING_FUNCTIONS = ("replay_judge_fresh_product", "strip_admission_header", "strip_timing", "strip_load_fields", "timing_load_reading_raw", "timing_load_decide")
REPLAY_GPU_FEATURE_VARIABLE = "CHECKER_TIER_FEATURES"
REPLAY_SHELL_WORD = re.compile(r"[A-Za-z_][A-Za-z0-9_-]*")
REPLAY_PACKAGE_MENTION = re.compile(r"(?:^|\s)-p\s+([A-Za-z0-9_-]+)")
REPLAY_BIN_MENTION = re.compile(r"--bin\s+([A-Za-z0-9_-]+)")
CHECKER_TIER_PACKAGE = "singlefs-checker-tier"


class RegistrationError(Exception):
    """登记表或调用写错了：退 2，出路是改登记表那一行或改调用。"""


class InputManifestError(Exception):
    """算不出一样输入：探针退非 0、读文件出错、工具链起不来。"""


class RegistrationRow:
    def __init__(self, key, entries, conditions, line_number, comment):
        self.key = key
        self.entries = entries
        self.conditions = conditions
        self.line_number = line_number
        self.comment = comment


# ── 登记表 ────────────────────────────────────────────────────────────────────

def read_registration_rows(root):
    """整张表的行；表不在返回空表。第一列是键，第二列是入口（- 是没有），第三列起到 # 之前是条件。"""
    table_path = os.path.join(root, REGISTRATION_TABLE)
    if not os.path.isfile(table_path):
        return []
    rows = []
    with open(table_path, encoding="utf-8", errors="surrogateescape") as handle:
        for line_number, line in enumerate(handle.read().split("\n"), 1):
            if not line.strip() or line.startswith("#"):
                continue
            columns = line.split("\t")
            conditions = []
            comment = ""
            for column in columns[2:]:
                if column.lstrip().startswith("#"):
                    comment = column.strip().lstrip("#").strip()
                    break
                conditions.extend(column.split())
            entries = [entry for entry in (columns[1].split() if len(columns) > 1 else []) if entry != "-"]
            rows.append(RegistrationRow(columns[0], entries, conditions, line_number, comment))
    return rows


def rows_of_key(rows, key):
    return [row for row in rows if row.key == key]


def resolve_entries(rows, key, referencing_keys=()):
    """这个键登记的全部入口；@<别的键> 展开成那个键的入口。"""
    own_rows = rows_of_key(rows, key)
    if not own_rows:
        raise RegistrationError(f"{REGISTRATION_TABLE} 里没有 {key} 这一行")
    resolved = []
    for row in own_rows:
        for entry in row.entries:
            if not entry.startswith("@"):
                resolved.append(entry)
                continue
            referenced_key = entry[1:]
            if referenced_key == key or referenced_key in referencing_keys:
                raise RegistrationError(f"{REGISTRATION_TABLE} 第 {row.line_number} 行：{key} 的 @{referenced_key} 引用绕成了圈")
            resolved.extend(resolve_entries(rows, referenced_key, referencing_keys + (key,)))
    return list(dict.fromkeys(resolved))


def resolve_input_paths(rows, key):
    """门禁 checker-tier-crates-mutation-replay 的拷贝范围（第三列 copy-range=）：返回 (路径, 贡献了它的行)。不是任何判定的键：只答拷哪些文件进每片的源码副本。"""
    own_rows = rows_of_key(rows, key)
    if not own_rows:
        raise RegistrationError(f"{REGISTRATION_TABLE} 里没有 {key} 这一行")
    paths = [token[len("copy-range="):] for row in own_rows for token in row.conditions if token.startswith("copy-range=")]
    if not paths:
        raise RegistrationError(f"{REGISTRATION_TABLE} 里 {key} 那一行第三列没有 copy-range=（拷贝范围）")
    return paths, own_rows


def entry_file(entry):
    """入口所在的文件（仓根起）；tests-of: / vm: 这一类没有文件交 None。"""
    if entry.startswith(("tests-of:", "vm:", "@")):
        return None
    return entry.split("::", 1)[0]


def experiment_condition_file_problems(root, kind, parts):
    """实验行一条前提点名的文件不在仓里的说明（文件头「准入条件」）：question-row 那份问题单，product-field 是 replay.sh 给那个实验键登记的留存产物；
    same-run-field 读同一趟的输出、不点名仓里的文件。复跑表按行的准入键那一段写错抛 RegistrationError。"""
    if kind == "question-row":
        if not os.path.isfile(os.path.join(root, parts["path"])):
            return [f"点名的问题单 {parts['path']} 不在仓里（前提指的东西可以被归档删掉，登记就靠不住；上一段的结论改用 same-run-field= 在同一趟里现判）"]
        return []
    if kind == "product-field":
        products = replay_products_of(root, parts["key"])
        if not products:
            return [f"点名的实验键 {parts['key']} 在 {REPLAY_SCRIPT} 的复跑表里一行留存产物都没有"]
        return [f"点名的留存产物 {product} 不在仓里（research/ 可以整个删掉；上一段的结论改用 same-run-field= 在同一趟里现判）"
                for product in products if not os.path.isfile(os.path.join(root, product))]
    return []


def lint_registration_rows(rows, root=None, all_rows=None):
    """登记表自己写对没有：键的形状、入口的写法（给了 root 还核入口的文件在不在）、准入条件认不认得（实验行只认实验的种类、门禁行只认门禁的种类）、
    @ 引用解不解得开、崩溃枚举用例第二列写 -；给了 root 还核门禁行 probe= / environment= 指的脚本在不在、实验行 question-row= / product-field= 点名的文件在不在
    （弄坏开关 condition-files-unchecked 下不核这一样）。返回问题清单。"""
    problems = []
    for row in rows:
        is_experiment = bool(EXPERIMENT_KEY_FORM.match(row.key))
        is_crash_case = bool(CRASH_CASE_KEY_FORM.match(row.key))
        if not is_experiment and not is_crash_case and not row.key.endswith(".sh") and not STAGE_CELL_KEY_FORM.match(row.key):
            problems.append(f"第 {row.line_number} 行：键 {row.key} 既不是门禁阶段文件名（*.sh）、阶段里的一格（<阶段>.sh:<格名>）、实验键（E<号>[/<方式>]），"
                            "也不是崩溃枚举用例（crash-case:<小写字母、数字、->）")
        for entry in row.entries:
            if not ENTRY_FORM.match(entry):
                problems.append(f"第 {row.line_number} 行：{row.key} 的入口 {entry} 认不出（写 <文件>::<限定名>、<文件>、tests-of:<包>、vm:<档名> 或 @<别的键>）")
                continue
            path = entry_file(entry)
            if root is not None and path is not None and not os.path.exists(os.path.join(root, path)):
                problems.append(f"第 {row.line_number} 行：{row.key} 的入口 {entry} 指的文件 {path} 不在")
        if is_crash_case:
            if row.entries:
                problems.append(f"第 {row.line_number} 行：崩溃枚举用例 {row.key} 的第二列写 -（入口就是第三列的 test=），实际写了 {' '.join(row.entries)}")
            try:
                crash_case = parse_crash_case(row)
                if root is not None:
                    crash_case_test_files(root, crash_case)
            except RegistrationError as error:
                problems.append(f"第 {row.line_number} 行：{error}")
            continue
        for token in row.conditions:
            try:
                if is_experiment:
                    kind, parts = parse_condition(token)
                    if root is not None and not break_is_set("condition-files-unchecked"):
                        problems.extend(f"第 {row.line_number} 行：{row.key} 的 {token} {problem}"
                                        for problem in experiment_condition_file_problems(root, kind, parts))
                    continue
                kind, parts = parse_gate_condition(token)
                if root is not None and kind in ("probe", "environment") and not os.path.isfile(os.path.join(root, parts["script"])):
                    problems.append(f"第 {row.line_number} 行：门禁行 {row.key} 的 {token} 指的脚本 {parts['script']} 不在")
            except RegistrationError as error:
                problems.append(f"第 {row.line_number} 行：{error}")
        try:
            resolve_entries(all_rows or rows, row.key)
        except RegistrationError as error:
            problems.append(f"第 {row.line_number} 行：{error}")
    crash_case_lines = {}
    for row in rows:
        if CRASH_CASE_KEY_FORM.match(row.key):
            crash_case_lines.setdefault(row.key, []).append(row.line_number)
    for key, line_numbers in crash_case_lines.items():
        if len(line_numbers) > 1:
            problems.append(f"第 {'、'.join(str(number) for number in line_numbers)} 行：崩溃枚举用例 {key} 登记了 {len(line_numbers)} 行，一条用例只许一行")
    return problems


def parse_gate_condition(token):
    """门禁行第三列的一条，切成 (种类, 各段)：command / readwrite 是 {"value"}，probe / environment 是 {"script", "arguments"}。
    认不出、脚本写成绝对路径或带 .. 都是登记错。"""
    match = GATE_CONDITION_FORM.match(token)
    if not match:
        raise RegistrationError(f"认不出的门禁准入条件：{token}（门禁行只认 command=<可执行名>、readwrite=<路径>、"
                                "probe=<仓内脚本>[:<参数>…]、environment=<仓内脚本>[:<参数>…]、copy-range=<路径>）")
    kind, value = match.group("kind"), match.group("value")
    if kind in ("probe", "environment"):
        script, *arguments = value.split(":")
        if not script or script.startswith("/") or ".." in script.split("/"):
            raise RegistrationError(f"门禁准入条件 {token} 的脚本要写成仓库根起的相对路径（不许绝对路径、不许带 ..）")
        return kind, {"script": script, "arguments": arguments}
    return kind, {"value": value}



# ── 函数单元键（lib_function_units.py 算闭包；这里拼工具链、依赖版本、判法单元与这条判定自己的内容输入）──────────

FUNCTION_UNITS_MODULE = {}


def function_units():
    """同目录的 lib_function_units.py，只载一次；找不到、载不进抛 UnitKeyError（调用方按「拒算」处置：这一条现判、不复用）。"""
    if "module" not in FUNCTION_UNITS_MODULE:
        path = os.path.join(os.path.dirname(os.path.realpath(__file__)), FUNCTION_UNITS_LIBRARY)
        if not os.path.isfile(path):
            raise UnitKeyError(f"找不到函数单元库 {path}（它与这一份同目录，随仓走）")
        specification = importlib.util.spec_from_file_location("singlefs_function_units", path)
        module = importlib.util.module_from_spec(specification)
        try:
            specification.loader.exec_module(module)
        except Exception as error:  # 载不进就算不出键：交给调用方按拒算处置，不当「没变」
            raise UnitKeyError(f"载不进函数单元库 {path}：{type(error).__name__}：{error}") from error
        FUNCTION_UNITS_MODULE["module"] = module
    return FUNCTION_UNITS_MODULE["module"]


class UnitKeyError(Exception):
    """算不出一把单元键（库载不进、入口找不到、工具链起不来）：这一条判定按拒算处置，红句与 --list-items 报出来。"""


class Request:
    """一条判定的请求：入口与它的键要拼进来的东西（请求的写法见文件头「判定请求」）。tokens 原样存进库，下一趟 gate-reuse 拿它重算。"""

    def __init__(self, name, entry, tokens):
        self.name = name
        self.entry = entry
        self.tokens = list(tokens)
        self.root = "."
        self.target = None
        self.features = []
        self.no_default_features = False
        self.cfg = []
        self.profile = None
        self.judges = []
        self.judge_root = None
        self.tools = []
        self.lines = []
        self.files = []
        self.environment_keys = []
        for token in self.tokens:
            kind, _separator, value = token.partition(":")
            if token == "no-default-features":
                self.no_default_features = True
            elif kind == "root" and value:
                self.root = value
            elif kind == "target" and value:
                self.target = value
            elif kind == "feature" and "/" in value:
                self.features.append(value)
            elif kind == "cfg" and value:
                self.cfg.append(value)
            elif kind == "profile" and value:
                self.profile = value
            elif kind == "judge" and value:
                self.judges.append(value)
            elif kind == "judge-root" and value:
                self.judge_root = value
            elif kind == "tool" and value:
                self.tools.append(value)
            elif kind == "line" and value:
                self.lines.append(value)
            elif kind == "file" and value:
                self.files.append(value)
            elif kind == "environment-of" and value:
                self.environment_keys.append(value)
            else:
                raise RegistrationError(f"判定请求 {name} 里认不出的一项「{token}」（写法见 admission.py 文件头「判定请求」）")
        if self.root.startswith("/") or ".." in self.root.split("/"):
            raise RegistrationError(f"判定请求 {name} 的 root:{self.root} 要写成仓根起的相对目录")

    def text(self):
        return "\t".join([self.name, self.entry, *self.tokens])


def parse_request_line(line):
    columns = line.rstrip("\n").split("\t")
    if len(columns) < 2 or not columns[0] or not columns[1]:
        raise RegistrationError(f"判定请求那一行要「<名字><制表符><入口>[<制表符><项>…]」：{line.strip()}")
    return Request(columns[0], columns[1], [column for column in columns[2:] if column])


class UnitKey:
    """一把单元键：key 是 64 位十六进制，refused 是拒算行（不空时 key 是 None）；composition 是组成清单（单元行与输入行，只当索引，作废时比它报出变了什么）。"""

    def __init__(self, key, composition, refusals):
        self.key = key
        self.composition = composition
        self.refusals = refusals


def entry_language(entry):
    path = entry.split("::", 1)[0]
    for suffix, language in ((".rs", "rust"), (".wgsl", "wgsl"), (".py", "python"), (".sh", "shell"), (".bash", "shell")):
        if path.endswith(suffix):
            return language
    return "rust" if "::" in entry and "/" not in path else None


def run_text(command, cwd, environment=None):
    """起一条只问版本的命令：交 (退出码, stdout 去掉两头空白)；起不来交 (127, "")。"""
    try:
        completed = subprocess.run(command, cwd=cwd, capture_output=True, text=True, errors="replace", env=environment, timeout=PROBE_TIMEOUT_SECONDS)
    except (OSError, subprocess.TimeoutExpired):
        return 127, ""
    return completed.returncode, completed.stdout.strip()


def flattened_setting_lines(prefix, value, out):
    """TOML 解析出来的值摊成「<前缀>.<键>=<值>」：只取生效的值（注释、空白、写的次序都不进）。"""
    if isinstance(value, dict):
        for name in sorted(value):
            flattened_setting_lines(f"{prefix}.{name}" if prefix else name, value[name], out)
    else:
        out.append(f"{prefix}={json.dumps(value, ensure_ascii=False, sort_keys=True)}")


def merged_configuration(layers):
    """cargo 的配置层（从近到远）合成一份生效的：标量取最近的那一层，数组照 cargo 的做法远的在前、近的接在后面。"""
    merged = {}

    def merge(into, value):
        for name, item in value.items():
            if isinstance(item, dict) and isinstance(into.get(name), dict):
                merge(into[name], item)
            elif isinstance(item, list) and isinstance(into.get(name), list):
                into[name] = into[name] + item
            elif isinstance(item, dict):
                into[name] = copy.deepcopy(item)
            else:
                into[name] = item
    for layer in reversed(layers):
        merge(merged, layer)
    return merged


class KeyContext:
    """一趟里算单元键的共用状态：每个闭包根一份 Rust 工作区、工具链行与 Cargo.lock；外部程序的版本与内容摘要只取一次。"""

    def __init__(self, repository_root, environment=None):
        self.root = os.path.abspath(repository_root)
        self.environment = dict(os.environ if environment is None else environment)
        self.workspaces = {}
        self.toolchains = {}
        self.locks = {}
        self.externals = {}
        self.interpreters = {}
        self.registration_rows = None
        self.host = None
        self.script_indices = {}
        self.judge_keys = {}
        self.rust_closures = {}

    def rust_closure(self, closure_root, workspace, unit, target_id, cfg, features, no_default_features):
        """同一个入口、同一组 cfg 与特性的 Rust 闭包一趟只收一次（test-plan 判编不编时收过的，算键时接着用）。"""
        cache_key = (closure_root, id(unit), target_id, tuple(cfg), tuple(features), no_default_features)
        if cache_key not in self.rust_closures:
            self.rust_closures[cache_key] = function_units().rust_closure_of(workspace, unit, target_id, list(cfg), True, list(features), no_default_features)
        return self.rust_closures[cache_key]

    # Python 与 shell 那一侧：同一棵树的脚本索引（解析过的模块、bash 规范化过的脚本）一趟只建一份，每个入口各收各的闭包
    def script_closure(self, closure_root, entry):
        library = function_units()
        path, _separator, qualified = entry.partition("::")
        under = [place for place in library.DEFAULT_SCRIPT_DIRECTORIES if os.path.isdir(os.path.join(closure_root, place))]
        under = list(dict.fromkeys(under + [os.path.dirname(path) or "."]))
        index_key = (closure_root, tuple(under))
        try:
            base = self.script_indices.get(index_key)
            if base is None:
                base = self.script_indices[index_key] = library.ScriptClosure(closure_root, under)
            script = library.ScriptClosure(closure_root, under)
            script.python = base.python
            script.shell = base.shell
            language = entry_language(entry)
            if language == "python":
                module = script.python.module(path)
                if module is None:
                    raise UnitKeyError(f"入口文件不在：{path}")
                if qualified:
                    entries = [unit for unit in module.units if unit.name == qualified and not unit.data]
                    if len(entries) != 1:
                        raise UnitKeyError(f"{path} 里限定名 {qualified} 命中 {len(entries)} 个单元（要恰好一个）")
                else:
                    entries = module.program_entries()[:1]
                script.closure.refusals.update(module.refusals)
                closure = script.run(entries)
                if base.shell is None:
                    base.shell = script.shell
                return closure
            index = base.shell_index([path])
            script.shell = index
            shell_file = index.files.get(path)
            if shell_file is None:
                raise UnitKeyError(f"入口文件不在：{path}")
            script.closure.refusals.update(shell_file.refusals)
            if shell_file.script_body is None:
                return script.closure
            if qualified:
                entries = [unit for unit in shell_file.units if unit.name == qualified and not unit.data]
                if len(entries) != 1:
                    raise UnitKeyError(f"{path} 里限定名 {qualified} 命中 {len(entries)} 个单元（要恰好一个）")
            else:
                entries = [shell_file.script_body]
            return script.run(entries)
        except AttributeError:
            # 函数单元库的内部变了（ScriptClosure 的字段对不上）：退回它的公开入口，每个入口各建一份索引（慢，结果一样）
            try:
                return library.closure_of(closure_root, entry, library.Options())
            except (library.FunctionUnitsError, library.UsageError) as error:
                raise UnitKeyError(f"{entry}：{error}") from error
        except (library.FunctionUnitsError, library.UsageError) as error:
            raise UnitKeyError(f"{entry}：{error}") from error

    def judge_key(self, judge, judge_root=None):
        """判法入口的闭包键（连同解释器版本与它点名的外部程序）：(键或 None, 组成清单, 拒算行)；同一个判法一趟只算一次。
        judge_root 是判法所在的那棵树（门禁写它自己所在的仓：样本档被判的是样本，判法在门禁那一份仓里）；不给是被判的根。"""
        root = os.path.normpath(os.path.join(self.root, judge_root)) if judge_root else self.root
        cache_key = (root, judge)
        if cache_key not in self.judge_keys:
            library = function_units()
            try:
                closure = self.script_closure(root, judge) if entry_language(judge) in ("python", "shell") else \
                    closure_of_entry(self, root, Request(judge, judge, []), judge)[0]
                toolchain = [f"interpreter {name}: {self.interpreter(name)}" for name in interpreter_lines(entry_language(judge))]
                key, refusals = library.closure_key(closure, toolchain, (), closure_programs_externals(self, closure))
                self.judge_keys[cache_key] = (key, closure_unit_lines(closure, prefix="judge "), [refusal.text() for refusal in refusals])
            except UnitKeyError as error:
                self.judge_keys[cache_key] = (None, [], [f"refused\tkey-uncomputable\t{judge}\t{error}"])
        return self.judge_keys[cache_key]

    # Rust 那一侧
    def workspace(self, closure_root):
        if closure_root not in self.workspaces:
            library = function_units()
            try:
                self.workspaces[closure_root] = library.RustWorkspace(closure_root)
            except library.FunctionUnitsError as error:
                raise UnitKeyError(f"Rust 工作区读不出（{error}）") from error
        return self.workspaces[closure_root]

    def host_triple(self, closure_root):
        if self.host is None:
            for line in self.rust_toolchain_lines(closure_root):
                if line.startswith("rustc -vV: host: "):
                    self.host = line[len("rustc -vV: host: "):]
            self.host = self.host or "x86_64-unknown-linux-gnu"
        return self.host

    def rust_toolchain_lines(self, closure_root):
        """工具链行：rustc -vV、cargo -V、构建环境的生效值（环境变量、各层 .cargo 配置合成之后的键、rust-toolchain、runner 与 wrapper 指的程序的内容摘要）。"""
        if closure_root in self.toolchains:
            return self.toolchains[closure_root]
        lines = []
        code, output = run_text(["rustc", "-vV"], closure_root, self.environment)
        if code != 0 or not output:
            raise UnitKeyError(f"rustc -vV 退 {code}（工具链起不来，单元键里没有工具链那一行就算不出）")
        lines += [f"rustc -vV: {line}" for line in output.split("\n") if line.strip()]
        code, output = run_text(["cargo", "-V"], closure_root, self.environment)
        if code != 0 or not output:
            raise UnitKeyError(f"cargo -V 退 {code}")
        lines.append(f"cargo -V: {output}")
        if not break_is_set("skip-build-environment"):
            try:
                lines += build_environment_setting_lines(closure_root, self.environment)
            except InputManifestError as error:
                raise UnitKeyError(f"构建环境读不出：{error}") from error
        self.toolchains[closure_root] = sorted(lines)
        return self.toolchains[closure_root]

    def lock(self, closure_root):
        if closure_root not in self.locks:
            path = os.path.join(closure_root, "Cargo.lock")
            try:
                with open(path, "rb") as handle:
                    self.locks[closure_root] = tomllib.load(handle).get("package") or []
            except (OSError, ValueError):
                self.locks[closure_root] = None
        return self.locks[closure_root]

    def dependency_lines(self, closure_root, workspace, closure, entry_target):
        """解析出的依赖版本行：闭包里有单元的工作区包，各自 Cargo.toml 写的外部依赖（入口是测试编译的另加入口那个包的 dev-dependencies），
        照 Cargo.lock 顺着传递依赖收（名字、版本、来源、checksum）；外部包自己的 Cargo.toml 在 cargo 缓存里找得到时，只在别的平台才要的依赖不收。
        Cargo.lock 不在交一行「Cargo.lock 不在」（不在也是一种取值）。"""
        packages = self.lock(closure_root)
        if packages is None:
            return ["Cargo.lock 不在或读不出"]
        crates, referenced = {}, set()
        for unit in closure.order:
            if getattr(unit, "language", None) != "rust" or getattr(unit, "data", False):
                continue
            crate = workspace.crate_of_path(unit.file)
            if crate is not None:
                crates[crate.directory] = crate
            referenced |= set(getattr(unit, "plain", None) or ())
            referenced |= {pair[0] for pair in (getattr(unit, "qualified", None) or ()) if isinstance(pair, tuple) and pair}
        entry_crate_directory = workspace.targets[entry_target].crate.directory if entry_target in workspace.targets else None
        entry_is_test = (entry_target is not None and entry_target.split(":")[1:2] in (["test"], ["bench"])) or any(line.startswith("test@") for line in closure.cfg_lines)
        by_name = collections.defaultdict(list)
        for package in packages:
            by_name[package.get("name")].append(package)
        wanted, lines, seen = [], set(), set()
        for directory, crate in sorted(crates.items()):
            declared = manifest_dependency_names(closure_root, directory, include_dev=entry_is_test and directory == entry_crate_directory,
                                                 host=self.host_triple(closure_root))
            # 闭包里有单元点了它的名字（use、路径、extern crate）的才收：同一个包里别的二进制用的外部包，改了不让这一条作废（弄坏开关 dependencies-by-crate 下照包收）
            allowed = {package for code_name, package in declared.items() if code_name in referenced or break_is_set("dependencies-by-crate")}
            own = [package for package in by_name.get(crate.package, ()) if not package.get("source")]
            if not own:
                continue
            for dependency in own[0].get("dependencies") or []:
                if dependency.split()[0] in allowed:
                    wanted.append(dependency)
        while wanted:
            dependency = wanted.pop()
            found = resolve_lock_dependency(by_name, dependency)
            if found is None:
                lines.add(f"Cargo.lock 里找不到 {dependency}")
                continue
            identity = (found.get("name"), found.get("version"), found.get("source"))
            if identity in seen:
                continue
            seen.add(identity)
            if not found.get("source"):
                continue   # 工作区里的包：它的单元在闭包里就由单元管，不在就没被用到
            lines.add(f"{found.get('name')} {found.get('version')} {found.get('source')} {found.get('checksum', '-')}")
            kept = registry_dependency_filter(self.environment, found, self.host_triple(closure_root))
            for next_dependency in found.get("dependencies") or []:
                if kept is None or next_dependency.split()[0] in kept:
                    wanted.append(next_dependency)
        return sorted(lines)

    # 外部程序与解释器
    def external(self, program):
        """闭包里点名的外部程序（cargo、herd7、busybox、qemu-system-*）：(内容 sha256, --version 第一行)；不在 PATH 上也是一种取值。"""
        if program not in self.externals:
            path = shutil.which(program, path=self.environment.get("PATH"))
            if path is None:
                self.externals[program] = ("找不到", f"{program} 不在 PATH 上")
            else:
                try:
                    digest = sha256_of_file(os.path.realpath(path))
                except OSError:
                    digest = "读不了"
                _code, output = run_text([path, "--version"], self.root, self.environment)
                self.externals[program] = (digest, (output.split("\n")[0] if output else "没打版本").replace("\t", " "))
        return self.externals[program]

    def interpreter(self, name):
        if name not in self.interpreters:
            code, output = run_text([name, "--version"], self.root, self.environment)
            self.interpreters[name] = (output.split("\n")[0] if output else f"{name} --version 退 {code}")
        return self.interpreters[name]

    def rows(self):
        if self.registration_rows is None:
            self.registration_rows = read_registration_rows(self.root)
        return self.registration_rows


def resolve_lock_dependency(by_name, dependency):
    """Cargo.lock 里一条依赖的写法（「名字」「名字 版本」「名字 版本 (来源)」）对到那一个包；对不上交 None。"""
    parts = dependency.split()
    candidates = by_name.get(parts[0], [])
    if len(parts) >= 2:
        candidates = [package for package in candidates if package.get("version") == parts[1]]
    if len(parts) >= 3:
        source = dependency.split("(", 1)[1].rstrip(")") if "(" in dependency else None
        candidates = [package for package in candidates if package.get("source") == source] or candidates
    return candidates[0] if len(candidates) == 1 else (candidates[0] if candidates and len(parts) >= 2 else None)


def host_cfg_value(expression, host):
    """target.'cfg(…)' 的谓词在本机目标上真不真：True / False / None（认不出：当真，多收一项依赖）。"""
    facts = {"target_os": "linux", "target_family": "unix", "target_arch": host.split("-")[0], "target_env": "gnu", "target_vendor": "unknown",
             "target_pointer_width": "64", "target_endian": "little"}
    names = {"unix": True, "windows": False, "test": False, "debug_assertions": None, "miri": False}
    tokens = re.findall(r'[A-Za-z_][A-Za-z0-9_]*|"[^"]*"|[(),=]', expression)
    position = [0]

    def peek():
        return tokens[position[0]] if position[0] < len(tokens) else None

    def take():
        position[0] += 1
        return tokens[position[0] - 1]

    def parse():
        word = take()
        if word in ("all", "any", "not") and peek() == "(":
            take()
            values = []
            while peek() not in (")", None):
                values.append(parse())
                if peek() == ",":
                    take()
            take()
            if word == "not":
                return None if not values or values[0] is None else not values[0]
            if word == "all":
                return False if False in values else (None if None in values else True)
            return True if True in values else (None if None in values else False)
        if peek() == "=":
            take()
            value = take().strip('"')
            if word not in facts:
                return None
            return facts[word] == value
        return names.get(word)
    try:
        return parse()
    except IndexError:
        return None


def target_section_applies(specification, host):
    if specification.startswith("cfg(") and specification.endswith(")"):
        return host_cfg_value(specification[4:-1], host) is not False
    return specification == host


def manifest_dependency_names(root, directory, include_dev, host):
    """工作区包 Cargo.toml 写的外部依赖：{代码里用的名字（键，- 换成 _）: 包名（package = 改名的取真名）}；本机目标不要的 target 段不收。"""
    sections = manifest_sections(os.path.join(root, directory or ".", "Cargo.toml")) or {}
    shared = ((manifest_sections(os.path.join(root, "Cargo.toml")) or {}).get("workspace") or {}).get("dependencies") or {}
    tables = [sections.get("dependencies") or {}, sections.get("build-dependencies") or {}]
    if include_dev:
        tables.append(sections.get("dev-dependencies") or {})
    for specification, table in (sections.get("target") or {}).items():
        if isinstance(table, dict) and target_section_applies(specification, host):
            tables += [table.get("dependencies") or {}, table.get("build-dependencies") or {}]
            if include_dev:
                tables.append(table.get("dev-dependencies") or {})
    names = {}
    for table in tables:
        for name, value in table.items():
            if isinstance(value, dict) and value.get("workspace") is True:
                value = dict(shared.get(name, {}) if isinstance(shared.get(name), dict) else {}, **{key: item for key, item in value.items() if key != "workspace"})
            if isinstance(value, dict) and "path" in value:
                continue
            names[name.replace("-", "_")] = value.get("package", name) if isinstance(value, dict) else name
    return names


def registry_dependency_filter(environment, package, host):
    """外部包自己的 Cargo.toml（cargo 缓存里 registry/src/*/<名>-<版本>/）：本机目标要的依赖名集合；找不到交 None（照 Cargo.lock 全收）。"""
    cargo_home = environment.get("CARGO_HOME") or os.path.join(environment.get("HOME") or os.path.expanduser("~"), ".cargo")
    matches = glob.glob(os.path.join(cargo_home, "registry", "src", "*", f"{package.get('name')}-{package.get('version')}", "Cargo.toml"))
    if not matches:
        return None
    sections = manifest_sections(matches[0])
    if sections is None:
        return None
    names = set()
    tables = [sections.get("dependencies") or {}, sections.get("build-dependencies") or {}]
    for specification, table in (sections.get("target") or {}).items():
        if isinstance(table, dict) and target_section_applies(specification, host):
            tables += [table.get("dependencies") or {}, table.get("build-dependencies") or {}]
    for table in tables:
        for name, value in table.items():
            names.add(value.get("package", name) if isinstance(value, dict) else name)
    return names


def profile_lines(closure_root, profile):
    """--release 这一类编译档：「profile <名>」与清单里 [profile.<名>] 摊开的生效值。"""
    sections = manifest_sections(os.path.join(closure_root, "Cargo.toml")) or {}
    lines = [f"profile {profile}"]
    out = []
    flattened_setting_lines(f"profile.{profile}", ((sections.get("profile") or {}).get(profile) or {}), out)
    return lines + [f"setting {line}" for line in out]


def rust_cfg_for_profile(closure_root, profile):
    """入口目标的生效 cfg：库的默认那一组，release 这一类不开 debug_assertions 的档去掉它（[profile.<名>] 写了 debug-assertions = true 的照留）。"""
    default = list(function_units().DEFAULT_RUST_CFG)
    if profile in (None, "dev", "test"):
        return default
    sections = manifest_sections(os.path.join(closure_root, "Cargo.toml")) or {}
    if ((sections.get("profile") or {}).get(profile) or {}).get("debug-assertions") is True:
        return default
    return [item for item in default if item != "debug_assertions"]


def closure_of_entry(context, closure_root, request, entry):
    """一个入口的闭包：交 (闭包, 目标号或 None, 工作区或 None)。入口找不到、cfg 下不编抛 UnitKeyError。"""
    library = function_units()
    language = entry_language(entry)
    if language is None:
        raise UnitKeyError(f"认不出入口 {entry} 是哪种语言（写 <文件>::<限定名>、<文件>，或 Rust 的 <crate>::<模块…>::<名>）")
    try:
        if language == "rust":
            workspace = context.workspace(closure_root)
            path, _separator, qualified = entry.partition("::")
            path, qualified = (None, entry) if "/" not in path and not path.endswith(".rs") else (path, qualified)
            unit, target_id = workspace.find_entry(path, qualified, request.target)
            cfg = rust_cfg_for_profile(closure_root, request.profile) + list(request.cfg)
            closure = context.rust_closure(closure_root, workspace, unit, target_id, cfg, request.features, request.no_default_features)
            return closure, target_id, workspace
        if language in ("python", "shell"):
            return context.script_closure(closure_root, entry), None, None
        closure = library.closure_of(closure_root, entry, library.Options())
        return closure, None, None
    except (library.FunctionUnitsError, library.UsageError) as error:
        raise UnitKeyError(f"{entry}：{error}") from error


def closure_unit_lines(closure, prefix=""):
    return [prefix + unit.text() for unit in sorted(closure.order, key=lambda unit: unit.sort_key())]


def closure_programs_externals(context, closure):
    return {program: context.external(program) for program in sorted(getattr(closure, "programs", {}) or {})}


def interpreter_lines(language):
    return {"python": ["python3"], "shell": ["bash"]}.get(language, [])


def compute_unit_key(context, request):
    """一条判定请求的单元键（键的组成见文件头「单元键」）：交 UnitKey。算不出（库载不进、入口找不到、工具链起不来）的交拒算（key 是 None），不抛。"""
    kb_documents_in_key = break_is_set("experiment-key-with-kb-documents")
    closure_root = os.path.normpath(os.path.join(context.root, request.root))
    lines, composition, refusals = [], [], []
    try:
        library = function_units()
        closure, target_id, workspace = closure_of_entry(context, closure_root, request, request.entry)
        language = entry_language(request.entry)
        toolchain = list(context.rust_toolchain_lines(closure_root)) if language == "rust" else []
        toolchain += [f"interpreter {name}: {context.interpreter(name)}" for name in interpreter_lines(language)]
        if request.profile and language == "rust":
            toolchain += profile_lines(closure_root, request.profile)
        dependencies = context.dependency_lines(closure_root, workspace, closure, target_id) if workspace is not None else []
        externals = closure_programs_externals(context, closure)
        entry_key, entry_refusals = library.closure_key(closure, toolchain, dependencies, externals)
        refusals += [refusal.text() for refusal in entry_refusals]
        lines.append(f"entry\t{entry_key}")
        composition += closure_unit_lines(closure)
        composition += [f"cfg\t{item}" for item in sorted(closure.cfg_lines)] + [f"toolchain\t{item}" for item in toolchain]
        composition += [f"dependency\t{item}" for item in dependencies]
        composition += [f"external\t{name}\t{sha}\t{version}" for name, (sha, version) in externals.items()]
        for judge in request.judges:
            judge_key, judge_composition, judge_refusals = context.judge_key(judge, request.judge_root)
            refusals += judge_refusals
            lines.append(f"judge\t{judge_key}")
            present = set(composition)
            composition += [line for line in judge_composition if line not in present]
    except UnitKeyError as error:
        return UnitKey(None, composition, refusals + [f"refused\tkey-uncomputable\t{request.entry}\t{error}"])
    for tool in request.tools:
        sha, version = context.external(tool)
        lines.append(f"tool\t{sha}\t{version}")
        composition.append(f"tool\t{tool}\t{sha}\t{version}")
    for text in request.lines:
        lines.append(f"line\t{text}")
        composition.append(f"line\t{text}")
    for relative in request.files:
        path = os.path.join(context.root, relative)
        digest = sha256_of_file(path) if os.path.isfile(path) else "不在"
        lines.append(f"file\t{digest}")
        composition.append(f"file\t{relative}\t{digest}")
    for registered_key in request.environment_keys:
        try:
            environment_lines, _digest = current_gate_environment(context.root, rows_of_key(context.rows(), registered_key))
        except (InputManifestError, RegistrationError) as error:
            refusals.append(f"refused\tenvironment-unreadable\t{registered_key}\t{error}")
            continue
        for token, first_line in environment_lines:
            lines.append(f"environment\t{first_line}")
            composition.append(f"environment\t{token}\t{first_line}")
    if kb_documents_in_key:
        # 弄坏开关 experiment-key-with-kb-documents：.claude/kb/decisions/ 下的文件整份进键（改前实验准入的样子：kb 决策文档进指纹）
        for path in sorted(glob.glob(os.path.join(context.root, ".claude", "kb", "decisions", "*"))):
            if os.path.isfile(path):
                lines.append(f"file\t{sha256_of_file(path)}")
    if refusals:
        return UnitKey(None, composition, sorted(set(refusals)))
    return UnitKey(hashlib.sha256("".join(line + "\n" for line in sorted(lines)).encode("utf-8", "surrogateescape")).hexdigest(), composition, [])


def changed_composition(earlier, later, shown_at_most=CHANGED_UNITS_SHOWN_AT_MOST):
    """两份组成清单比出变了什么：单元按（种类、限定名）比指纹，别的行按原文比。交 (说明句的清单, 共几处)。"""
    def split(lines):
        units, others = {}, set()
        for line in lines:
            parts = line.split("\t")
            if len(parts) == 4 and parts[0] not in ("external", "tool", "file") and ":" in parts[2]:
                units[(parts[0], parts[1], parts[2].split(":", 1)[0])] = parts[3]
            else:
                others.add(line)
        return units, others
    earlier_units, earlier_others = split(earlier)
    later_units, later_others = split(later)
    notes = []
    for identity in sorted(set(earlier_units) | set(later_units)):
        kind, name, file = identity
        if identity not in earlier_units:
            notes.append(f"多了 {kind} {file}::{name}")
        elif identity not in later_units:
            notes.append(f"少了 {kind} {file}::{name}")
        elif earlier_units[identity] != later_units[identity]:
            notes.append(f"变了 {kind} {file}::{name}")
    for line in sorted(later_others - earlier_others):
        notes.append(f"新的输入行 {line.replace(chr(9), ' ')[:160]}")
    for line in sorted(earlier_others - later_others):
        notes.append(f"没了的输入行 {line.replace(chr(9), ' ')[:160]}")
    return notes[:shown_at_most], len(notes)


# ── 判定库（每类判定一份，固定在 git common-dir 下；文件头「判定库」）────────────────────────────────────────

class VerdictLibrary:
    """<git common-dir>/singlefs-unit-verdicts/<种类>/：keys/<键>.json 是一条判定（只记绿），items/<名字的 sha256>.json 是这个名字最近一次判绿的那一份，
    compositions/<键>.tsv.gz 是那把键的组成清单（按键存一份，算过键就写，只当索引：键对不上时拿它与此刻的比，报出变了的单元）。
    取不到 git common-dir 时 directory 是 None：不读不写，全部现判。"""

    def __init__(self, library_root, kind):
        self.kind = kind
        common_directory = git_common_directory(library_root)
        self.directory = os.path.join(common_directory, VERDICT_LIBRARY_DIRECTORY_NAME, kind_directory_name(kind)) if common_directory else None

    def path_of_key(self, key):
        return os.path.join(self.directory, "keys", key + ".json")

    def path_of_item(self, name):
        return os.path.join(self.directory, "items", hashlib.sha256(name.encode("utf-8", "surrogateescape")).hexdigest() + ".json")

    def path_of_composition(self, key):
        return os.path.join(self.directory, "compositions", key + ".tsv.gz")

    def read(self, path):
        try:
            with open(path, encoding="utf-8") as handle:
                record = json.load(handle)
        except (OSError, ValueError):
            return None
        return record if isinstance(record, dict) and record.get("verdict") == "green" else None

    def store_composition(self, unit_key):
        """组成清单按键落盘一份（已经在就不写）；写不成不碍判定，只是下一趟报不出变了的单元。"""
        if self.directory is None or unit_key.key is None:
            return
        path = self.path_of_composition(unit_key.key)
        if os.path.isfile(path):
            return
        write_atomically(path, gzip.compress("".join(line + "\n" for line in unit_key.composition).encode("utf-8", "surrogateescape"), mtime=0))

    def composition_of(self, key):
        try:
            with open(self.path_of_composition(key), "rb") as handle:
                return gzip.decompress(handle.read()).decode("utf-8", "surrogateescape").rstrip("\n").split("\n")
        except (OSError, EOFError, gzip.BadGzipFile):
            return None

    def lookup(self, request, unit_key, full=False):
        """交 (决定, 原因)：决定是 reuse、judge、refused 之一。"""
        if unit_key.key is None:
            return "refused", "拒算：" + "；".join(line.split("\t", 1)[1].replace("\t", " ") for line in unit_key.refusals[:3])
        if self.directory is None:
            return "judge", "没有判定库（取不到 git common-dir）"
        if full:
            return "judge", "全量：只写不读"
        if break_is_set("verdict-lookup-skipped"):   # 弄坏开关：不读库（复用失灵的样子），自证 reuse-three-questions 与 reuse-sentinel 那两格转红
            return "judge", "弄坏开关 verdict-lookup-skipped：不读库"
        record = self.read(self.path_of_key(unit_key.key))
        if record is not None:
            return "reuse", f"键相同的判定（{record.get('judged', '没记日期')}，{record.get('how', '没记跑法')}）"
        earlier = self.read(self.path_of_item(request.name))
        if earlier is None:
            return "judge", "库里没有这一条"
        earlier_composition = self.composition_of(earlier.get("key", ""))
        if earlier_composition is None:
            return "judge", f"键对不上（上一次 {earlier.get('judged', '没记日期')} 判绿，那一份的组成清单不在库里）"
        notes, total = changed_composition(earlier_composition, unit_key.composition)
        return "judge", f"键对不上（上一次 {earlier.get('judged', '没记日期')} 判绿），变了 {total} 处：{'；'.join(notes) or '组成清单相同、键的写法变了'}"

    def record(self, request_text, name, key, how, summary=""):
        """判绿之后记一条：键、判定日期、跑法、请求原文（组成清单在 compositions/ 里按键另存）；同目录建临时文件写完再改名换上。写不成交原因，写成交空串。"""
        if self.directory is None or not key:
            return "没有判定库或这一条拒算，不记"
        record = {"verdict": "green", "key": key, "item": name, "request": request_text, "how": how, "summary": summary, "judged": judged_date()}
        text = json.dumps(record, ensure_ascii=False, indent=1) + "\n"
        for path in (self.path_of_key(key), self.path_of_item(name)):
            problem = write_atomically(path, text)
            if problem:
                return problem
        return ""

    def items(self):
        """库里每个名字最近一次判绿的那一份：[(名字, 记录)]。"""
        found = []
        for path in sorted(glob.glob(os.path.join(self.directory or "/nonexistent", "items", "*.json"))):
            record = self.read(path)
            if record is not None:
                found.append((record.get("item"), record))
        return found


def kind_directory_name(kind):
    return re.sub(r"[^A-Za-z0-9._:-]", "-", kind)


JUDGED_DATE = []


def judged_date():
    if not JUDGED_DATE:
        environment = dict(os.environ, TZ="Asia/Tokyo")
        code, output = run_text(["date", "+%F"], "/", environment)
        JUDGED_DATE.append(output if code == 0 and output else time.strftime("%Y-%m-%d"))
    return JUDGED_DATE[0]


def write_atomically(path, content):
    """content 是 str 或 bytes；同目录建临时文件、写完 fsync、改名换上。写不成交原因，写成交空串。"""
    directory = os.path.dirname(path)
    try:
        os.makedirs(directory, exist_ok=True)
        handle_number, temporary = tempfile.mkstemp(dir=directory, prefix="." + os.path.basename(path) + ".partial.")
        try:
            with os.fdopen(handle_number, "wb") as handle:
                handle.write(content if isinstance(content, bytes) else content.encode("utf-8", "surrogateescape"))
                handle.flush()
                os.fsync(handle.fileno())
            os.replace(temporary, path)
        except OSError:
            os.unlink(temporary)
            raise
    except OSError as error:
        return f"写不了 {path}：{error}"
    return ""


def full_requested(options_full):
    return options_full or os.environ.get("SINGLEFS_GATE_FULL", "") == "1"


def composition_files(root, unit_key):
    """组成清单里点到的文件（单元行的文件、数据行的文件）各自此刻的 sha256：verdict-record 拿它判「跑的过程中这一条的输入动没动」，动了才重算键。
    只当这一趟的捷径，不进键。"""
    files = set()
    for line in unit_key.composition:
        parts = line.split("\t")
        if len(parts) == 4 and ":" in parts[2]:
            files.add(parts[2].split(":", 1)[0])
        elif len(parts) == 3 and parts[0] == "file":
            files.add(parts[1])
    digests = {}
    for relative in sorted(files):
        path = os.path.join(root, relative)
        digests[relative] = sha256_of_file(path) if os.path.isfile(path) else "不在"
    for manifest in ("Cargo.toml", "Cargo.lock"):
        path = os.path.join(root, manifest)
        digests[manifest] = sha256_of_file(path) if os.path.isfile(path) else "不在"
    return digests


REUSE_CHECKS_LIBRARY = "lib_reuse_checks.py"
SENTINEL_STATE_SUFFIX = ".reuse-sentinel.json"
UNIT_VERDICT_SAMPLE_DIRECTORY = ".unit-verdict-sample"
_REUSE_CHECKS = []


def reuse_checks_module():
    """同目录的 lib_reuse_checks.py（复用哨兵的判法，文件头「复用哨兵」）；拷进不带它的小树里跑的这一份交 None。"""
    if not _REUSE_CHECKS:
        path = os.path.join(os.path.dirname(os.path.realpath(__file__)), REUSE_CHECKS_LIBRARY)
        module = None
        if os.path.isfile(path):
            specification = importlib.util.spec_from_file_location("lib_reuse_checks_for_admission", path)
            module = importlib.util.module_from_spec(specification)
            specification.loader.exec_module(module)
        _REUSE_CHECKS.append(module)
    return _REUSE_CHECKS[0]


def sentinel_candidates(library, keyed):
    """复用哨兵的候选：此刻键算得出、库里 keys/<键>.json 记着判绿、记的键与此刻的相同的请求 → {名字: "绿"}。
    与复用判法（VerdictLibrary.lookup）分开查：直接读键文件，不经 lookup 与 read。"""
    candidates = {}
    if library.directory is None:
        return candidates
    for request, unit_key in keyed:
        if not unit_key.key:
            continue
        try:
            with open(os.path.join(library.directory, "keys", unit_key.key + ".json"), encoding="utf-8") as handle:
                record = json.load(handle)
        except (OSError, ValueError):
            continue
        if isinstance(record, dict) and record.get("verdict") == "green" and record.get("key") == unit_key.key:
            candidates[request.name] = "绿"
    return candidates


def plan_requests(root, kind, requests, library_root=None, full=False, context=None):
    """逐条算键、查库：交 ([(请求, UnitKey, 决定, 原因)], 判定库)，照请求的次序；算过的键各把组成清单落盘一份。
    复用哨兵（文件头「复用哨兵」）在查库之前抽：抽中强制现判的那一条不读库、照现判；抽到的哨兵查完库就记下它的结局。
    哨兵本身、要打的几行与红句挂在交出去的判定库上（sentinel、sentinel_lines、sentinel_problems）。"""
    context = context or KeyContext(root)
    library = VerdictLibrary(library_root or root, kind)
    keyed = [(request, compute_unit_key(context, request)) for request in requests]
    library.sentinel, library.sentinel_lines, library.sentinel_problems = None, [], []
    checks = reuse_checks_module()
    if checks is None:
        library.sentinel_lines.append(f"  · 复用哨兵（{kind}）：找不到 {REUSE_CHECKS_LIBRARY}（这一份 admission.py 跑在不带它的树里），不抽哨兵")
    else:
        sentinel = checks.ReuseSentinel(f"scripts/gate/{os.path.basename(__file__)}", kind)
        off_reason = ("全量（--full 或 SINGLEFS_GATE_FULL=1，只写不读）" if full else "没有判定库（取不到 git common-dir）" if library.directory is None
                      else f"样本档（根下有 {UNIT_VERDICT_SAMPLE_DIRECTORY}/，样本钉死了现判与复用的条数）"
                      if os.path.isdir(os.path.join(root, UNIT_VERDICT_SAMPLE_DIRECTORY)) else "")
        sentinel.plan(sentinel_candidates(library, keyed) if not off_reason else {}, reuse=not off_reason, off_reason=off_reason,
                      printer=library.sentinel_lines.append)
        library.sentinel = sentinel
    planned = []
    for request, unit_key in keyed:
        if library.sentinel is not None and library.sentinel.forced(request.name):
            decision, reason = "judge", (f"强制现判（复用哨兵抽中，种子 {library.sentinel.seed}；库里键相同的判定是绿，这一趟不读库）")
        else:
            decision, reason = library.lookup(request, unit_key, full)
        if library.sentinel is not None and request.name in library.sentinel.sentinels:
            library.sentinel.observe(request.name, "复用" if decision == "reuse" else "现判", "绿" if decision == "reuse" else "-")
        library.store_composition(unit_key)
        planned.append((request, unit_key, decision, reason))
    if library.sentinel is not None and library.sentinel.state == "planned":
        # 开跑前就能判的那一半：哨兵有一条被现判 ⇒ 复用失灵；强制现判那一条的判定在 verdict-record 里比
        forced_item, library.sentinel.forced_item = library.sentinel.forced_item, None
        library.sentinel_problems = library.sentinel.finish(printer=lambda line: None)
        library.sentinel.forced_item = forced_item
    return planned, library


def plan_line(root, request, unit_key, decision):
    return json.dumps({"request": request.text(), "key": unit_key.key, "decision": decision,
                       "files": composition_files(root, unit_key) if unit_key.key else {}}, ensure_ascii=False)


def break_is_set(switch_name):
    return switch_name in os.environ.get(BREAK_VARIABLE, "").split(",")


def say(message):
    print(message, file=sys.stderr)



def parse_condition(token):
    """把一条准入条件切成 (种类, 各段)；认不出、正则编不过都是登记错。"""
    question_row = QUESTION_ROW_FORM.match(token)
    if question_row:
        try:
            re.compile(question_row.group("pattern"))
        except re.error as error:
            raise RegistrationError(f"准入条件 {token} 的正则编不过：{error}") from error
        return ("question-row", question_row.groupdict())
    product_field = PRODUCT_FIELD_FORM.match(token)
    if product_field:
        return ("product-field", product_field.groupdict())
    same_run_field = SAME_RUN_FIELD_FORM.match(token)
    if same_run_field:
        return ("same-run-field", same_run_field.groupdict())
    raise RegistrationError(f"认不出的准入条件：{token}（只认 question-row=<文件>#<行号>:<正则>、product-field=<实验键>:<行名>:<字段>=<值> 与 same-run-field=<行名>:<字段>=<值>）")

def sha256_of_file(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def read_configuration_file(path):
    try:
        with open(path, "rb") as handle:
            return handle.read()
    except OSError as error:
        raise InputManifestError(f"读不了构建环境里的 {path}：{error}") from error

def resolved_program(program, base_directory, environment):
    """cargo 配置或环境变量里写的程序：带斜杠的按 base_directory 解开相对路径，不带斜杠的在 environment 的 PATH 里找；找不到交 None。"""
    if not program:
        return None
    if "/" in program:
        return program if os.path.isabs(program) else os.path.join(base_directory, program)
    return shutil.which(program, path=environment.get("PATH"))


def runner_argument_directories(root):
    """runner / wrapper 参数里的相对路径另从哪几个目录解：登记的每条崩溃枚举用例的包目录（cargo 起 runner 时当前目录是包目录）。
    交 [(仓根起的包目录, 绝对路径)]，按包目录排好；登记表读不了、写错的交空表（登记行本身另有自查）。弄坏开关
    runner-arguments-from-configuration-directory-only 下交空表（只从 .cargo 那一层 / 仓根解，改前的判法）。"""
    if break_is_set("runner-arguments-from-configuration-directory-only"):
        return []
    try:
        cases = crash_cases_of(read_registration_rows(root))
    except (RegistrationError, OSError):
        return []
    packages = workspace_package_directories(root)
    relatives = sorted({packages[case.package] for case in cases if case.package in packages})
    return [(relative, os.path.join(root, relative)) for relative in relatives]


def program_contents(words, base_directory, environment, argument_directories=()):
    """首词是程序、其后是参数的一个值：交回那个程序的内容（进指纹），找不到的交「找不到」一句（值本身另有一行进指纹）；
    首词之后指到现存文件的参数（绝对路径，或从 base_directory 起的相对路径）接着按内容进：`bash tools/r.sh` 这一类，真正定行为的脚本在参数里；
    相对路径另从 argument_directories（runner_argument_directories：登记用例的包目录，cargo 在那里起 runner）逐个解，解得到的都进，
    名字里写从哪个包目录解的（仓根起，不写绝对路径）。那个脚本再 source、再读的文件不进（文件头「管不到的」）。"""
    program = words[0] if words else ""
    path = resolved_program(program, base_directory, environment)
    contents = read_configuration_file(path) if path and os.path.isfile(path) else f"找不到：{program}".encode("utf-8", "surrogateescape")
    if break_is_set("runner-program-only"):
        return contents
    for argument in words[1:]:
        candidate = argument if os.path.isabs(argument) else os.path.join(base_directory, argument)
        if os.path.isfile(candidate):
            contents += b"\n<" + argument.encode("utf-8", "surrogateescape") + b">\n" + read_configuration_file(candidate)
        if os.path.isabs(argument):
            continue
        for relative_directory, directory in argument_directories:
            candidate = os.path.join(directory, argument)
            if os.path.isfile(candidate):
                contents += (b"\n<" + argument.encode("utf-8", "surrogateescape") + "，从包目录 ".encode("utf-8")
                             + relative_directory.encode("utf-8", "surrogateescape") + " 解>\n".encode("utf-8") + read_configuration_file(candidate))
    return contents


def compiler_version(program, root, environment, described_as):
    """编译器的 `-V`（在仓根跑）；起不来、退非 0、一个字不打都抛 InputManifestError。"""
    try:
        completed = subprocess.run([program, "-V"], cwd=root, capture_output=True, env=dict(environment))
    except OSError as error:
        raise InputManifestError(f"{described_as} 起不来：{error}") from error
    if completed.returncode != 0 or not completed.stdout.strip():
        raise InputManifestError(f"{described_as} -V 退 {completed.returncode}、打了「{completed.stdout.decode('utf-8', 'replace').strip()}」")
    return completed.stdout


def configured_program_lines(label, content, base_directory, root, environment, argument_directories=()):
    """一份 cargo 配置文件里指程序的键（build.rustc 取 -V；build.rustc-wrapper、build.rustc-workspace-wrapper、target.<…>.runner 按内容）：
    [(名字, 内容)]。base_directory 是 .cargo 所在的那一层（相对路径从它起）。读不成 TOML 的不另进（原文已经进了，cargo 也读不了它）。"""
    try:
        settings = tomllib.loads(content.decode("utf-8"))
    except (UnicodeDecodeError, ValueError):
        return []
    lines = []
    build = settings.get("build") if isinstance(settings.get("build"), dict) else {}
    compiler = build.get("rustc")
    if isinstance(compiler, str):
        lines.append((f"<构建环境：{label} 里 build.rustc 的 -V>",
                      compiler_version(resolved_program(compiler, base_directory, environment) or compiler, root, environment,
                                       f"{label} 里的 build.rustc = {compiler}")))
    for key in CONFIGURATION_PROGRAM_BUILD_KEYS:
        if isinstance(build.get(key), str):
            lines.append((f"<构建环境：{label} 里 build.{key} 指的程序>", program_contents([build[key]], base_directory, environment, argument_directories)))
    targets = settings.get("target") if isinstance(settings.get("target"), dict) else {}
    for target_name in sorted(targets):
        runner = targets[target_name].get("runner") if isinstance(targets[target_name], dict) else None
        words = runner.split() if isinstance(runner, str) else [str(word) for word in runner] if isinstance(runner, list) else []
        if words:
            lines.append((f"<构建环境：{label} 里 target.{target_name}.runner 指的程序>", program_contents(words, base_directory, environment,
                                                                                                      argument_directories)))
    return lines

def header_fields(header_line):
    """E7INPUT 行的字段；reason= 取到行尾（理由里可以有空格）。"""
    body = header_line[len(HEADER_PREFIX):]
    reason_start = body.find(" reason=")
    reason = None
    if reason_start >= 0:
        reason = body[reason_start + len(" reason="):]
        body = body[:reason_start]
    fields = dict(token.split("=", 1) for token in body.split() if "=" in token)
    if reason is not None:
        fields["reason"] = reason
    return fields


def header_lines_of(product_path):
    header_lines = []
    try:
        with open(product_path, "rb") as handle:
            for _line_index in range(HEADER_LINES_READ_AT_MOST):
                raw_line = handle.readline()
                if not raw_line:
                    break
                text = raw_line.decode("utf-8", "replace").rstrip("\n")
                if not text.startswith(HEADER_PREFIX):
                    break
                header_lines.append(text)
    except OSError:
        return []
    return header_lines


class StoredFingerprint:
    def __init__(self, product_path, fingerprint, modified_at):
        self.product_path = product_path
        self.fingerprint = fingerprint
        self.modified_at = modified_at


def stored_fingerprints(root, key):
    """research/results/ 下（含子目录）头上记着这个键的输入指纹的每一份产物。"""
    found = []
    results_directory = os.path.join(root, RESULTS_DIRECTORY)
    for directory, _subdirectories, file_names in os.walk(results_directory):
        for file_name in sorted(file_names):
            product_path = os.path.join(directory, file_name)
            for header_line in header_lines_of(product_path):
                fields = header_fields(header_line)
                if fields.get("name") == "input_fingerprint" and fields.get("key") == key and fields.get("sha256"):
                    found.append(StoredFingerprint(os.path.relpath(product_path, root), fields["sha256"], os.path.getmtime(product_path)))
                    break
    return found


class ReplayRow:
    """复跑表的一行连同它的准入键：identity 是前三列原样（实验号|二进制|参数），product 是第四列，explicit 是按行的准入键那一段指明过它。"""

    def __init__(self, experiment, identity, product, key, header_writer, explicit):
        self.experiment = experiment
        self.identity = identity
        self.product = product
        self.key = key
        self.header_writer = header_writer
        self.explicit = explicit


def replay_row_key_entries(text):
    """replay.sh 里按行的准入键那一段：返回 ([(行号, 键, 前三列, 产物头谁写)], 写法错的说明)。没有这一段返回两个空表。"""
    entries, problems = [], []
    if REPLAY_ROW_KEYS_START not in text:
        return entries, problems
    if text.count(REPLAY_ROW_KEYS_START) != 1:
        return entries, [f"{REPLAY_SCRIPT} 里按行的准入键那一段（{REPLAY_ROW_KEYS_START.strip()}）有 {text.count(REPLAY_ROW_KEYS_START)} 处，只许一处"]
    start = text.index(REPLAY_ROW_KEYS_START) + len(REPLAY_ROW_KEYS_START)
    first_line_number = text[:start].count("\n") + 1
    for offset, line in enumerate(text[start:].split("\n")):
        if line == REPLAY_ROW_KEYS_END_LINE:
            break
        if not line.strip():
            continue
        where = f"{REPLAY_SCRIPT} 第 {first_line_number + offset} 行（按行的准入键）"
        columns = line.split("\t")
        if len(columns) != 3:
            problems.append(f"{where}要三列、制表符分隔（键、复跑表那一行的前三列、产物头谁写），实际 {len(columns)} 列")
            continue
        key, identity, header_writer = columns
        identity_parts = identity.split("|")
        if not EXPERIMENT_KEY_FORM.match(key):
            problems.append(f"{where}的键 {key} 不是实验键的写法（E<号> 或 E<号>/<方式>，方式只用小写字母、数字与连字符）")
        elif len(identity_parts) != 3:
            problems.append(f"{where}第二列 {identity} 要写成复跑表那一行的前三列「实验号|二进制|参数」，实际 {len(identity_parts)} 段")
        elif key.split("/")[0] != identity_parts[0]:
            problems.append(f"{where}的键 {key} 与它指的那一行的实验号 {identity_parts[0]} 不同")
        if header_writer not in REPLAY_ROW_HEADER_WRITERS:
            problems.append(f"{where}第三列 {header_writer} 认不出，产物头谁写只认 {'、'.join(REPLAY_ROW_HEADER_WRITERS)}")
        entries.append((first_line_number + offset, key, identity, header_writer))
    else:
        problems.append(f"{REPLAY_SCRIPT} 里按行的准入键那一段没有收尾的 {REPLAY_ROW_KEYS_END_LINE} 行")
    return entries, problems


def replay_rows_with_keys(root):
    """scripts/experiments/replay.sh 复跑表的每一行连同它的准入键：返回 ([ReplayRow], 按行的准入键那一段写错的说明)。
    没被那一段指明的行，键是实验号、产物头由装置写。弄坏开关 ignore-row-keys 下那一段整个不认（改前的样子）。"""
    replay_path = os.path.join(root, REPLAY_SCRIPT)
    if not os.path.isfile(replay_path):
        return [], []
    with open(replay_path, encoding="utf-8", errors="replace") as handle:
        text = handle.read()
    table = []
    for line in text.split("\n"):
        match = REPLAY_ROW_FORM.match(line)
        if match:
            table.append((match.group(1), "|".join(match.group(1, 2, 3)), match.group(4)))
    entries, problems = ([], []) if break_is_set("ignore-row-keys") else replay_row_key_entries(text)
    chosen = {}
    for line_number, key, identity, header_writer in entries:
        hits = sum(1 for _experiment, row_identity, _product in table if row_identity == identity)
        if hits != 1:
            problems.append(f"{REPLAY_SCRIPT} 第 {line_number} 行给 {key} 指的「{identity}」在复跑表里有 {hits} 行，要恰好一行")
        elif identity in chosen:
            problems.append(f"{REPLAY_SCRIPT} 第 {line_number} 行又给「{identity}」指明了一次键（{key}），同一行只许指明一次")
        else:
            chosen[identity] = (key, header_writer)
    rows = []
    for experiment, identity, product in table:
        key, header_writer = chosen.get(identity, (experiment, "device"))
        rows.append(ReplayRow(experiment, identity, product, key, header_writer, identity in chosen))
    return rows, problems


def replay_products_of(root, experiment_key):
    """scripts/experiments/replay.sh 登记表里准入键是这个键的那几行的留存产物（仓库根起的路径）；按行的准入键那一段写错抛 RegistrationError。"""
    rows, problems = replay_rows_with_keys(root)
    if problems:
        raise RegistrationError("；".join(problems))
    return [os.path.join(RESULTS_DIRECTORY, row.product) for row in rows if row.key == experiment_key]


def markdown_row_status(root, path, row_label):
    """那份文件里第一格是 row_label 的第一行表格行的最后一格；文件或行不在返回 None 与原因。"""
    full_path = os.path.join(root, path)
    if not os.path.isfile(full_path):
        return None, f"{path} 不在"
    with open(full_path, encoding="utf-8", errors="replace") as handle:
        for line in handle.read().split("\n"):
            stripped = line.strip()
            if not stripped.startswith("|"):
                continue
            cells = [cell.strip() for cell in re.split(r"(?<!\\)\|", stripped)[1:-1]]
            if cells and cells[0] == row_label:
                return cells[-1], None
    return None, f"{path} 里没有第一格是 {row_label} 的表格行"


def product_field_value(root, experiment_key, line_name, field):
    """replay.sh 登记给这个实验键的产物里，最后一行 `E7RESULT name=<行名>` 的字段值；取不到返回 None 与原因。"""
    products = replay_products_of(root, experiment_key)
    if len(products) != 1:
        raise RegistrationError(f"product-field 要 replay.sh 登记表里 {experiment_key} 恰好一行，实际 {len(products)} 行")
    product = products[0]
    full_path = os.path.join(root, product)
    if not os.path.isfile(full_path):
        return None, product, f"{product} 不在树里"
    last_line = None
    with open(full_path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            if line.startswith(f"E7RESULT name={line_name} ") or line.rstrip("\n") == f"E7RESULT name={line_name}":
                last_line = line.rstrip("\n")
    if last_line is None:
        return None, product, f"{product} 里没有 name={line_name} 这一行"
    fields = dict(token.split("=", 1) for token in last_line.split()[1:] if "=" in token)
    if field not in fields:
        return None, product, f"{product} 的 name={line_name} 行没有 {field} 字段"
    return fields[field], product, None


# same-run-field=<行名>:<字段>=<值>（文件头「准入条件」）：同一趟里先跑的那一段这一趟的输出，路径由调用方经这个环境变量交来；不读留存产物
SAME_RUN_FIELD_FORM = re.compile(r"^same-run-field=(?P<line_name>[^:]+):(?P<field>[^=]+)=(?P<value>\S+)$")
SAME_RUN_PRODUCT_VARIABLE = "SINGLEFS_ADMISSION_SAME_RUN_PRODUCT"


def last_result_line_field(path, line_name, field):
    """那份输出里最后一行 `E7RESULT name=<行名>` 的字段值；取不到交 (None, 原因)。"""
    last_line = None
    with open(path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            if line.startswith(f"E7RESULT name={line_name} ") or line.rstrip("\n") == f"E7RESULT name={line_name}":
                last_line = line.rstrip("\n")
    if last_line is None:
        return None, f"{path} 里没有 name={line_name} 这一行"
    fields = dict(token.split("=", 1) for token in last_line.split()[1:] if "=" in token)
    if field not in fields:
        return None, f"{path} 的 name={line_name} 行没有 {field} 字段"
    return fields[field], None


def same_run_field_value(line_name, field):
    """same-run-field 的取值：SAME_RUN_PRODUCT_VARIABLE 指的那份同一趟的输出里最后一行 name=<行名> 的字段；取不到交 (None, 那份输出, 原因)。"""
    product = os.environ.get(SAME_RUN_PRODUCT_VARIABLE, "")
    if not product:
        return None, "", (f"{SAME_RUN_PRODUCT_VARIABLE} 没设：这一段要在同一趟里接在前一段之后跑，"
                          f"先跑前一段、把它这一趟的输出路径经 {SAME_RUN_PRODUCT_VARIABLE} 交给这一段（不读 research/results/ 下的留存产物）")
    if not os.path.isfile(product):
        return None, product, f"{SAME_RUN_PRODUCT_VARIABLE} 指的同一趟输出 {product} 不在"
    value, problem = last_result_line_field(product, line_name, field)
    return value, product, problem


def judge_preconditions(root, key, conditions):
    """② 前提：返回 (没齐的清单, 齐了的清单)，每项是一句说明。登记写错抛 RegistrationError。
    same-run-field 读调用方交来的同一趟输出（弄坏开关 same-run-field-unchecked 下不读、一律当齐了）。"""
    missing = []
    satisfied = []
    for token in conditions:
        kind, parts = parse_condition(token)
        if kind == "same-run-field":
            if break_is_set("same-run-field-unchecked"):
                satisfied.append(f"（弄坏开关 same-run-field-unchecked：没读同一趟的输出）{token}")
                continue
            value, product, problem = same_run_field_value(parts["line_name"], parts["field"])
            if problem is not None:
                missing.append(f"{problem}（要 name={parts['line_name']} 的 {parts['field']}={parts['value']}）")
            elif value == parts["value"]:
                satisfied.append(f"同一趟的输出 {product} 的 name={parts['line_name']} {parts['field']}={value}")
            else:
                missing.append(f"同一趟的输出 {product} 的 name={parts['line_name']} {parts['field']}={value}，要 {parts['value']}")
            continue
        if kind == "question-row":
            status, problem = markdown_row_status(root, parts["path"], parts["row"])
            if problem is not None:
                raise RegistrationError(f"{key} 的前提 {token} 读不到：{problem}")
            if re.search(parts["pattern"], status):
                satisfied.append(f"{parts['path']} 第 {parts['row']} 行的状态「{status}」匹配「{parts['pattern']}」")
            else:
                missing.append(f"{parts['path']} 第 {parts['row']} 行的状态是「{status}」，要匹配「{parts['pattern']}」")
        elif kind == "product-field":
            value, product, problem = product_field_value(root, parts["key"], parts["line_name"], parts["field"])
            if problem is not None:
                missing.append(f"{problem}（要 name={parts['line_name']} 的 {parts['field']}={parts['value']}）")
            elif value == parts["value"]:
                satisfied.append(f"{product} 的 name={parts['line_name']} {parts['field']}={value}")
            else:
                missing.append(f"{product} 的 name={parts['line_name']} {parts['field']}={value}，要 {parts['value']}")
    return missing, satisfied



def run_probe(root, parts):
    """bash <根>/<脚本> <参数…>，工作目录是根；返回 (退出码, stdout 字节, stderr 字节)。起不来、超时都算退 127。"""
    command = ["bash", os.path.join(root, parts["script"]), *parts["arguments"]]
    try:
        completed = subprocess.run(command, cwd=root, capture_output=True, timeout=PROBE_TIMEOUT_SECONDS)
    except (OSError, subprocess.TimeoutExpired) as error:
        return 127, b"", str(error).encode("utf-8", "replace")
    return completed.returncode, completed.stdout, completed.stderr


def probe_output_tail(output, errors):
    """探针输出的最后三行（stdout 与 stderr 接起来），给没齐的那一条当明细。"""
    lines = [line.strip() for line in (output + b"\n" + errors).decode("utf-8", "replace").split("\n") if line.strip()]
    return "｜".join(lines[-3:]) if lines else "一个字都没打"


def gate_conditions_of(rows, wanted_kinds):
    """这几行登记的门禁条件里种类在 wanted_kinds 之中的，[(条件原文, 种类, 各段)]；写错了抛 RegistrationError。"""
    found = []
    for row in rows:
        for token in row.conditions:
            kind, parts = parse_gate_condition(token)
            if kind in wanted_kinds:
                found.append((token, kind, parts))
    return found


def judge_gate_preconditions(root, rows):
    """门禁的前提逐条判：返回没齐的清单，每项一句说明。"""
    missing = []
    for token, kind, parts in gate_conditions_of(rows, GATE_PRECONDITION_KINDS):
        if kind == "command":
            if shutil.which(parts["value"]) is None:
                missing.append(f"{parts['value']} 不在 PATH 里（{token}）")
        elif kind == "readwrite":
            path = parts["value"] if os.path.isabs(parts["value"]) else os.path.join(root, parts["value"])
            if not os.access(path, os.R_OK | os.W_OK):
                missing.append(f"{parts['value']} 不在，或当前用户不可读写（{token}）")
        else:
            exit_code, output, errors = run_probe(root, parts)
            if exit_code != 0:
                missing.append(f"bash {' '.join([parts['script'], *parts['arguments']])} 退 {exit_code}（{token}）：{probe_output_tail(output, errors)}")
    return missing


def current_gate_environment(root, rows):
    """这一道登记的环境（environment=）逐条跑一遍：返回 ([(条件原文, 输出的第一行)], 整组的 sha256)；没登记返回 ([], None)。
    有一条退非 0 或一个字都没打，抛 InputManifestError。"""
    environment_lines = []
    digest = hashlib.sha256()
    for token, _kind, parts in gate_conditions_of(rows, ("environment",)):
        exit_code, output, errors = run_probe(root, parts)
        if exit_code != 0 or not output.strip():
            raise InputManifestError(f"{token} 退 {exit_code}：{probe_output_tail(output, errors)}")
        digest.update(token.encode("utf-8", "surrogateescape") + b"\n" + output + b"\0")
        environment_lines.append((token, output.decode("utf-8", "replace").strip().split("\n")[0]))
    if not environment_lines:
        return [], None
    return environment_lines, digest.hexdigest()


def environment_text(environment_lines):
    return "；".join(f"{token} → {first_line}" for token, first_line in environment_lines)


def git_output(root, *git_arguments):
    completed = subprocess.run(["git", "-C", root, *git_arguments], capture_output=True, text=True, errors="replace")
    return completed.returncode, completed.stdout


class CrashCase:
    def __init__(self, row, package, target, function, count_lines, exhaustive_lines, thread_lines, is_shardable_across_machines,
                 thread_variable=None, features=(), red_blocks_judged_by_test=False):
        self.row = row
        # 登记行的 red-blocks=judged-by-test：故障注入之后的红块由用例自己认下，门禁快档与 --full 起用例按退出码判、不按库里的红块判
        self.red_blocks_judged_by_test = red_blocks_judged_by_test
        # 登记行的 features=：用例要编进的 cargo 特性（没登记是空）；crash-case-command 在 -p <包> 之后带 --features
        self.features = tuple(features)
        # 用例读线程数的环境变量（登记行的 threads-variable=，没登记是 SINGLEFS_CRASH_POINT_REPLAY_THREADS）：crash-case-command 设它、判日志的记录行里记它
        self.thread_variable = thread_variable or CRASH_POINT_REPLAY_THREADS_VARIABLE
        self.is_shardable_across_machines = is_shardable_across_machines
        self.key = row.key
        self.name = row.key[len(CRASH_CASE_KEY_PREFIX):]
        self.package = package
        self.target = target
        self.function = function
        self.count_lines = count_lines
        self.exhaustive_lines = exhaustive_lines
        self.thread_lines = thread_lines

    def test_text(self):
        return f"{self.package}:{self.target}:{self.function}"


def parse_crash_case(row):
    """崩溃枚举用例行的第三列：写法认不出、test= 不是恰好一条、exhaustive= / threads= 点名的前缀没登记成 count-line=、
    shard= 不是 across-machines 或多于一条、threads-variable= 不是环境变量名或多于一条、features= 写法不对或多于一条或与 shard= 并存、
    red-blocks= 不是 judged-by-test 或多于一条，都抛 RegistrationError。"""
    tests, count_lines, exhaustive_lines, thread_lines, shard_values, thread_variables, feature_values = [], [], [], [], [], [], []
    red_blocks_values = []
    for token in row.conditions:
        match = CRASH_CASE_CONDITION_FORM.match(token)
        if not match:
            raise RegistrationError(f"崩溃枚举用例 {row.key} 的第三列认不出 {token}（只认 test=<包>:<测试目标>:<用例函数>、"
                                    f"count-line=<前缀>、exhaustive=<前缀>、threads=<前缀>、threads-variable=<环境变量名>、features=<特性,特性…>、"
                                    f"shard={CRASH_CASE_SHARD_ACROSS_MACHINES}、red-blocks={CRASH_CASE_RED_BLOCKS_JUDGED_BY_TEST}）")
        kind, value = match.group("kind"), match.group("value")
        if kind == "red-blocks":
            if value != CRASH_CASE_RED_BLOCKS_JUDGED_BY_TEST:
                raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token}：red-blocks= 只认 {CRASH_CASE_RED_BLOCKS_JUDGED_BY_TEST}"
                                        "（故障注入之后的红块由用例自己认下，门禁快档与 --full 起用例按退出码判）")
            red_blocks_values.append(value)
            continue
        if kind == "test":
            test = CRASH_CASE_TEST_FORM.match(value)
            if not test:
                raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token} 要写成 test=<包>:<测试目标>:<用例函数>（字母、数字、_，包名另许 -）")
            tests.append(test)
            continue
        if kind == "shard":
            if value != CRASH_CASE_SHARD_ACROSS_MACHINES:
                raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token}：shard= 只认 {CRASH_CASE_SHARD_ACROSS_MACHINES}")
            shard_values.append(value)
            continue
        if kind == "features":
            if not CRASH_CASE_FEATURES_FORM.match(value) or len(set(value.split(","))) != len(value.split(",")):
                raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token}：features= 要写逗号隔开的 cargo 特性名（小写字母、数字、_、-，不重复）")
            feature_values.append(value)
            continue
        if kind == "threads-variable":
            if not THREADS_VARIABLE_FORM.match(value):
                raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token}：threads-variable= 要写用例读线程数的环境变量名（大写字母、数字、_）")
            thread_variables.append(value)
            continue
        if not COUNT_LINE_PREFIX_FORM.match(value):
            raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token}：前缀只许大写字母、数字、_，以字母开头")
        {"count-line": count_lines, "exhaustive": exhaustive_lines, "threads": thread_lines}[kind].append(value)
    if len(tests) != 1:
        raise RegistrationError(f"崩溃枚举用例 {row.key} 要恰好一条 test=，实际 {len(tests)} 条")
    if len(shard_values) > 1:
        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 shard= 至多一条，实际 {len(shard_values)} 条")
    if len(thread_variables) > 1:
        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 threads-variable= 至多一条，实际 {len(thread_variables)} 条")
    if len(feature_values) > 1:
        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 features= 至多一条，实际 {len(feature_values)} 条")
    if len(red_blocks_values) > 1:
        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 red-blocks= 至多一条，实际 {len(red_blocks_values)} 条")
    if feature_values and shard_values:
        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 features= 与 shard={CRASH_CASE_SHARD_ACROSS_MACHINES} 不并存：分片的驱动脚本自己拼 cargo、不带特性，分片那一趟会漏掉特性")
    if len(set(count_lines)) != len(count_lines):
        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 count-line= 有重复的前缀")
    unregistered = [prefix for prefix in exhaustive_lines + thread_lines if prefix not in count_lines]
    if unregistered:
        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 exhaustive= / threads= 点名的 {' '.join(unregistered)} 没有登记成 count-line=")
    test = tests[0]
    thread_variable = None if break_is_set("thread-variable-ignored") else (thread_variables[0] if thread_variables else None)
    return CrashCase(row, test.group("package"), test.group("target"), test.group("function"),
                     count_lines, exhaustive_lines, thread_lines, bool(shard_values), thread_variable,
                     feature_values[0].split(",") if feature_values else (), bool(red_blocks_values))


def crash_cases_of(rows):
    """登记表里全部崩溃枚举用例，照登记表的次序；一行写错抛 RegistrationError。"""
    return [parse_crash_case(row) for row in rows if CRASH_CASE_KEY_FORM.match(row.key)]


def crash_case_of_key(rows, key):
    own_rows = rows_of_key(rows, key)
    if not CRASH_CASE_KEY_FORM.match(key) or len(own_rows) != 1:
        raise RegistrationError(f"{REGISTRATION_TABLE} 里崩溃枚举用例 {key} 要恰好一行，实际 {len(own_rows)} 行")
    return parse_crash_case(own_rows[0])


def manifest_sections(path):
    try:
        with open(path, "rb") as handle:
            return tomllib.load(handle)
    except (OSError, ValueError):  # TOMLDecodeError 与编码错都是 ValueError
        return None


def workspace_package_directories(root):
    """仓根 Cargo.toml 的工作区成员（members 按 glob 展开；仓根自己是包也算）：包名 → 仓根起的目录。"""
    root_sections = manifest_sections(os.path.join(root, "Cargo.toml")) or {}
    directories = []
    if "package" in root_sections:
        directories.append(".")
    for pattern in (root_sections.get("workspace") or {}).get("members") or []:
        for directory in sorted(glob.glob(os.path.join(root, pattern))):
            directories.append(os.path.relpath(directory, root))
    packages = {}
    for directory in directories:
        name = ((manifest_sections(os.path.join(root, directory, "Cargo.toml")) or {}).get("package") or {}).get("name")
        if name:
            packages[name] = os.path.normpath(directory)
    return packages


def test_target_source_files(root, package_directory, target):
    """包目录（相对 root）下测试目标 target 的源文件（相对 root）：tests/<目标>.rs，或 tests/<目标>/main.rs 那种目录目标里的全部 .rs；
    目标不在交空表。"""
    single_file = os.path.join(package_directory, "tests", target + ".rs")
    directory_target = os.path.join(package_directory, "tests", target)
    if os.path.isfile(os.path.join(root, single_file)):
        return [single_file]
    if os.path.isfile(os.path.join(root, directory_target, "main.rs")):
        return sorted(os.path.relpath(path, root)
                      for path in glob.glob(os.path.join(root, directory_target, "**", "*.rs"), recursive=True))
    return []


# 测试目标顺着带进来的源文件（files_brought_in_by_target）：include!（宏名前带不带 ::core:: / std:: 都算）、mod <名>; 与 #[path = "…"] mod <名>;
INCLUDE_CODE_MACRO = re.compile(r"\binclude\s*!\s*[(\[{]\s*")
MODULE_DECLARATION = re.compile(r"\bmod\s+(?:r#)?([A-Za-z_][A-Za-z0-9_]*)\s*;")
PATH_ATTRIBUTE = re.compile(r'^#\s*\[\s*path\s*=\s*(b?r?#*".*")\s*\]$', re.S)
# 用 use 把 include 族宏（include!、include_str!、include_bytes!）引进来（改不改名都算）：之后怎么写调用认不出
USE_OF_INCLUDE_MACRO = re.compile(r"\buse\b[^;]*\binclude(?:_str|_bytes)?\b[^;]*;")
# 带进来的源文件最多顺几份（防成环的 include!、mod 互相带）
TARGET_FILES_FOLLOWED_AT_MOST = 500


def string_literal_value(literal):
    """Rust 字符串字面量（"…" 或 r#"…"#）的值；带转义的（反斜杠）交 None（路径里用不到，判不出就按判不出算）。"""
    raw = re.match(r'^b?r(#*)"(.*)"\1$', literal, re.S)
    if raw:
        return raw.group(2)
    plain = re.match(r'^b?"(.*)"$', literal, re.S)
    if plain and "\\" not in plain.group(1):
        return plain.group(1)
    return None


def files_brought_in_by_target(root, relative_files):
    """测试目标的源文件（relative_files，相对 root）顺着 include!("<字面路径>")、#[path = "<字面路径>"] mod <名>; 与 mod <名>;（<名>.rs 或 <名>/mod.rs）
    带进来的全部源文件（相对 root，按找到的次序，头几份是 relative_files 本身）：返回 (文件清单, 判不出的原因或 None)。
    判不出：include! 的参数不是字符串字面量（套 concat!、env! 这一类）、带进来的文件找不到或读不了、有代码用 use 把 include 族宏引进来、带进来的超过上限。
    路径的起点照 rustc：include! 与 #[path] 从写它的那份文件所在的目录起；mod <名>; 在测试目标的根文件与 mod.rs 里从所在目录起，别的文件从「所在目录/文件名」起。
    弄坏开关 target-own-files-only 下只交 relative_files 本身（改前的判法）。"""
    if break_is_set("target-own-files-only"):
        return list(relative_files), None
    roots = set(relative_files)
    found, queue = [], list(relative_files)
    while queue:
        relative = os.path.normpath(queue.pop(0))
        if relative in found:
            continue
        if len(found) >= TARGET_FILES_FOLLOWED_AT_MOST:
            return found, f"带进来的源文件超过 {TARGET_FILES_FOLLOWED_AT_MOST} 份"
        try:
            with open(os.path.join(root, relative), encoding="utf-8", errors="replace") as handle:
                code = rust_code_without_comments(handle.read())
        except OSError as error:
            return found, f"读不了 {relative}：{error}"
        found.append(relative)
        directory = os.path.dirname(relative)
        if USE_OF_INCLUDE_MACRO.search(code):
            return found, f"{relative} 里用 use 把 include 族宏引进来了（之后的调用认不出读的是哪一份）"
        for call in INCLUDE_CODE_MACRO.finditer(code):
            literal_end = end_of_literal(code, call.end())
            value = string_literal_value(code[call.end():literal_end]) if literal_end is not None and code[call.end()] in 'br"' else None
            if value is None or not re.match(r"\s*[)\]}]", code[literal_end:]):
                return found, f"{relative} 里 include! 的参数不是一个字符串字面量（{code[call.start():call.end() + 40].strip()}…）：读的是哪一份判不出"
            queue.append(os.path.relpath(value, root) if os.path.isabs(value) else os.path.join(directory, value))
        attribute_end_to_start = {end: start for start, end in attribute_spans(code)}
        module_base = directory if (relative in roots or os.path.basename(relative) in ("mod.rs", "main.rs", "lib.rs")) \
            else os.path.join(directory, os.path.splitext(os.path.basename(relative))[0])
        for declaration in MODULE_DECLARATION.finditer(code):
            head = code[:declaration.start()].rstrip()
            qualifier = re.search(r"\bpub(?:\s*\([^()]*\))?$", head)
            if qualifier:
                head = head[:qualifier.start()].rstrip()
            path_value, cursor = None, len(head)
            while cursor in attribute_end_to_start:
                attribute = head[attribute_end_to_start[cursor]:cursor]
                path_attribute = PATH_ATTRIBUTE.match(attribute)
                if path_attribute:
                    path_value = string_literal_value(path_attribute.group(1))
                    if path_value is None:
                        return found, f"{relative} 里 {attribute} 的路径判不出"
                cursor = len(head[:attribute_end_to_start[cursor]].rstrip())
            if path_value is not None:
                queue.append(os.path.join(directory, path_value))
                continue
            name = declaration.group(1)
            candidates = [os.path.join(module_base, name + ".rs"), os.path.join(module_base, name, "mod.rs")]
            existing = [candidate for candidate in candidates if os.path.isfile(os.path.join(root, candidate))]
            if not existing:
                return found, f"{relative} 里 mod {name}; 找不到 {' 或 '.join(candidates)}"
            queue += existing
    return found, None


def crash_case_test_files(root, case):
    """这条用例的测试目标的源文件（仓根起）：tests/<目标>.rs，或 tests/<目标>/main.rs 那种目录目标里的全部 .rs，连同它们顺着
    include! / #[path] / mod 带进来的（files_brought_in_by_target）。
    包不是工作区成员、目标不在、带进来的判不出、用例函数在目标里找不到、用例函数没标 #[ignore]，都抛 RegistrationError。"""
    package_directory = workspace_package_directories(root).get(case.package)
    if package_directory is None:
        raise RegistrationError(f"崩溃枚举用例 {case.key} 的包 {case.package} 不是仓根 Cargo.toml 的工作区成员")
    files = test_target_source_files(root, package_directory, case.target)
    if not files:
        single_file = os.path.join(package_directory, "tests", case.target + ".rs")
        directory_target = os.path.join(package_directory, "tests", case.target)
        raise RegistrationError(f"崩溃枚举用例 {case.key} 的测试目标 {case.target} 不在（没有 {single_file}，也没有 {directory_target}/main.rs）")
    files, undecided = files_brought_in_by_target(root, files)
    if undecided:
        raise RegistrationError(f"崩溃枚举用例 {case.key} 的测试目标 {case.target} 带进来的源文件判不出（{undecided}）：用例函数标没标 #[ignore] "
                                "判不出，按没标算；include! / #[path] 写成字符串字面量、不用 use 改名 include 族宏")
    marks = definitions_marked_ignored(root, files, case.function)
    if not marks:
        raise RegistrationError(f"崩溃枚举用例 {case.key} 的用例函数 {case.function} 在 {'、'.join(files)} 里找不到字面的 fn {case.function}(："
                                "改了名？宏生成的用例判不出标没标 #[ignore]，要写成字面的函数")
    unmarked = [relative for relative, marked in marks if not marked]
    if unmarked:
        raise RegistrationError(f"崩溃枚举用例 {case.key} 的用例函数 {case.function} 有 {len(unmarked)} 处定义没标 #[ignore]（{'、'.join(unmarked)}；"
                                f"共 {len(marks)} 处，cfg 二选一的、子模块里同名的都算）：不带 --ignored 的 cargo test 也会跑到全量；"
                                "给每一处补上 #[ignore = \"<为什么平时不跑>\"]")
    return files


def rust_code_without_comments(text):
    """去掉 Rust 源码里的注释（行注释、可嵌套的块注释），字符串、原始字符串、字符字面量原样留着：判「别处有没有代码点名一个测试目标」时，
    注释里提到它不算，字符串里写它的路径（include_str!、#[path]、运行期按路径读）算。"""
    pieces = []
    position = 0
    length = len(text)
    while position < length:
        if text.startswith("//", position):
            line_end = text.find("\n", position)
            position = length if line_end < 0 else line_end
            continue
        if text.startswith("/*", position):
            depth = 0
            while position < length:
                if text.startswith("/*", position):
                    depth += 1
                    position += 2
                elif text.startswith("*/", position):
                    depth -= 1
                    position += 2
                    if depth == 0:
                        break
                else:
                    position += 1
            pieces.append(" ")
            continue
        end = end_of_literal(text, position)
        if end is not None:
            pieces.append(text[position:end])
            position = end
            continue
        pieces.append(text[position])
        position += 1
    return "".join(pieces)


RAW_STRING_START = re.compile(r'b?r(?P<hashes>#*)"')


def end_of_literal(code, position):
    """code[position] 起是字符串、原始字符串或字符字面量时交回它之后那个下标；不是时交 None（生命周期不算字面量）。"""
    character = code[position]
    if character in "br":
        raw_string = RAW_STRING_START.match(code, position)
        previous_is_identifier = position > 0 and (code[position - 1].isalnum() or code[position - 1] == "_")
        if raw_string and not previous_is_identifier:
            closing = '"' + raw_string.group("hashes")
            end = code.find(closing, raw_string.end())
            return len(code) if end < 0 else end + len(closing)
    if character == '"':
        end = position + 1
        while end < len(code) and code[end] != '"':
            end += 2 if code[end] == "\\" else 1
        return min(end + 1, len(code))
    if character == "'":
        if code.startswith("\\", position + 1):
            closing = code.find("'", position + 3)
            return len(code) if closing < 0 else closing + 1
        if position + 2 < len(code) and code[position + 2] == "'":
            return position + 3
    return None


def attribute_spans(code):
    """去掉注释之后的代码里每一条属性（`#[…]`、`#![…]`）的 (起, 止)：方括号按层数配对，字符串、原始字符串与字符字面量里的括号不算。"""
    spans = []
    position = 0
    while position < len(code):
        end = end_of_literal(code, position)
        if end is not None:
            position = end
            continue
        attribute = ATTRIBUTE_START.match(code, position)
        if not attribute:
            position += 1
            continue
        depth, cursor = 0, attribute.end() - 1
        while cursor < len(code):
            end = end_of_literal(code, cursor)
            if end is not None:
                cursor = end
                continue
            if code[cursor] == "[":
                depth += 1
            elif code[cursor] == "]":
                depth -= 1
                if depth == 0:
                    break
            cursor += 1
        spans.append((position, min(cursor + 1, len(code))))
        position = cursor + 1
    return spans


def attributes_before_each_definition(code, function):
    """去掉注释之后的代码里每一处 `fn <function>(`（cfg 二选一的、子模块里同名的都算，按出现的次序）前面紧挨着的那串属性：
    [[属性原文，一条一个，按出现的次序], …]；一处都没有交空表。属性与 fn 之间可以夹 pub、pub(…)、async、unsafe、const、extern "…"。
    弄坏开关 first-definition-only 下只交第一处。"""
    attribute_lists = []
    for found in re.finditer(r"\bfn\s+" + re.escape(function) + r"\s*\(", code):
        head = code[:found.start()].rstrip()
        while True:
            qualifier = FUNCTION_QUALIFIER_AT_END.search(head)
            if not qualifier:
                break
            head = head[:qualifier.start()].rstrip()
        start_of_attribute_ending_at = {end: start for start, end in attribute_spans(head)}
        attributes = []
        cursor = len(head)
        while cursor in start_of_attribute_ending_at:
            start = start_of_attribute_ending_at[cursor]
            attributes.append(head[start:cursor])
            cursor = len(head[:start].rstrip())
        attribute_lists.append(list(reversed(attributes)))
        if break_is_set("first-definition-only"):
            break
    return attribute_lists


def attributes_mark_ignored(attributes):
    """那串属性里有没有 #[ignore] 或 #[ignore = "…"]（# 与 [ 之间许空白）。"""
    form = IGNORE_ATTRIBUTE_FORM_WITHOUT_SPACE if break_is_set("ignore-attribute-without-space") else IGNORE_ATTRIBUTE_FORM
    return any(form.match(attribute) for attribute in attributes)


def definitions_marked_ignored(root, relative_files, function):
    """测试目标的源文件（relative_files，相对 root）里每一处 `fn <function>(` 标没标 #[ignore]：[(文件, 标了?)]，按文件与出现的次序；
    一处都没有交空表，读不了抛 OSError。弄坏开关 first-definition-only 下只看头一份有定义的文件里的第一处。"""
    marks = []
    for relative in relative_files:
        with open(os.path.join(root, relative), encoding="utf-8", errors="replace") as handle:
            attribute_lists = attributes_before_each_definition(rust_code_without_comments(handle.read()), function)
        marks += [(relative, attributes_mark_ignored(attributes)) for attributes in attribute_lists]
        if marks and break_is_set("first-definition-only"):
            break
    return marks


def test_function_is_marked_ignored(root, package_directory, target, function):
    """root 底下包目录 package_directory（相对 root）里测试目标 target 的用例函数 function 标没标 #[ignore]：同名的每一处定义都标了 True，
    有一处没标 False，目标不在、读不了、一处定义都找不到（宏生成的用例这一类）、顺着 include! / #[path] / mod 带进来的判不出交 None。
    .claude/hooks/lib_heavy_tests.py（重型测试闸）按文件路径导入本模块调它，与 crash-cases 自查同一套判法；闸把 None 按没标算。"""
    files, undecided = files_brought_in_by_target(root, test_target_source_files(root, package_directory, target))
    if undecided:
        return None
    try:
        marks = definitions_marked_ignored(root, files, function)
    except OSError:
        return None
    if not marks:
        return None
    return all(marked for _relative, marked in marks)


def git_common_directory(root):
    return_code, output = git_output(root, "rev-parse", "--path-format=absolute", "--git-common-dir")
    common_directory = output.strip() if return_code == 0 else ""
    return common_directory or None

def fields_of_line(line):
    return dict(token.split("=", 1) for token in line.split() if "=" in token)

def judge_worker_threads(prefix, count_line, log_lines, machine_cores, threads_explicitly_one, default_threads):
    """按计数行的 states= 找那一行 CRASH_POINT_REPLAY_PARALLEL_FINISHED，判工作线程：返回 (原因或 None, 一句说明)。
    计数行自己带 worker_threads= 与 slices= 的（跑完那一行就是计数行：crash-injection 快档的 CRASH_INJECTION_FINISHED 这一类）拿它自己判，
    它不续跑、不带 resumed_slices= / freshly_run_slices= 的按读回 0 片、这一趟跑了全部片算（弄坏开关 threads-skip-self-contained-finish-line 下
    这一类不判线程）。
    判红只在这一趟真跑了至少两片、却只起了 1 个工作线程、本机多于 1 核、线程数没显式设成 1 时：全部片从进度文件读回时起 0 个线程，
    只剩 1 片要跑时最多起 1 个，这两种都不是「线程数没传进去」。
    另判红：那一行的 worker_threads_source= 是用例自己取的默认（DEFAULT_WORKER_THREAD_SOURCES）而 configured_worker_threads= 不等于
    default_threads（每台默认重活线程数，registered_heavy_threads）：线程变量没传进去，或用例的默认还按核数算（弄坏开关 threads-default-unchecked 下不判）。"""
    count_fields = fields_of_line(count_line)
    if "worker_threads" in count_fields and "slices" in count_fields:
        if break_is_set("threads-skip-self-contained-finish-line"):
            return None, f"{prefix}：跑完那一行自己带线程数，没判（弄坏开关 threads-skip-self-contained-finish-line）"
        finished_line, fields = count_line, dict(count_fields)
        if "resumed_slices" not in fields and "freshly_run_slices" not in fields:
            fields.update(resumed_slices="0", freshly_run_slices=fields["slices"])
    else:
        states = count_fields.get("states")
        finished = [line for line in log_lines if line.startswith(f"{CRASH_POINT_REPLAY_PARALLEL_FINISHED_PREFIX}states={states} ")] if states else []
        if not finished:
            return (f"{prefix} 那一行（states={states or '读不到'}）找不到状态数对得上的 CRASH_POINT_REPLAY_PARALLEL_FINISHED 行，判不出起了几个工作线程"
                    "（全量要经 crates/singlefs-checker-tier/src/crash.rs 的 enumerate_crash_states_in_state_slices 跑）"), ""
        finished_line, fields = finished[0], fields_of_line(finished[0])
    try:
        worker_threads = int(fields["worker_threads"])
        slices = int(fields["slices"])
        resumed_slices = int(fields["resumed_slices"])
        freshly_run_slices = int(fields["freshly_run_slices"])
    except (KeyError, ValueError):
        return (f"跑完那一行缺 worker_threads= / slices= / resumed_slices= / freshly_run_slices=，或不是整数："
                f"{finished_line}"), ""
    if resumed_slices + freshly_run_slices != slices:
        return (f"跑完那一行读回的片 {resumed_slices} + 这一趟跑的片 {freshly_run_slices} ≠ 总片数 {slices}："
                f"{finished_line}"), ""
    if freshly_run_slices > 0 and worker_threads == 0:
        return f"这一趟跑了 {freshly_run_slices} 片，却报起了 0 个工作线程：{finished_line}", ""
    if "shards" in fields and not break_is_set("threads-ignore-shards"):
        return judge_threads_of_each_shard(prefix, finished_line, fields, default_threads)
    source = fields.get("worker_threads_source")
    if source in DEFAULT_WORKER_THREAD_SOURCES and fields.get("configured_worker_threads") != str(default_threads) and not break_is_set("threads-default-unchecked"):
        return (f"{prefix}：线程数没显式设（worker_threads_source={source}，用例自己取的默认），配的 {fields.get('configured_worker_threads') or '读不到'} 个"
                f"不等于每台默认重活线程数 {default_threads}（.claude/heavy-thread-default；多半是线程变量没传进去，或用例的默认还按核数算）：{finished_line}"), ""
    if break_is_set("threads-by-worker-count"):
        judged_on_one_thread = worker_threads == 1
    else:
        judged_on_one_thread = freshly_run_slices >= 2 and worker_threads == 1
    if judged_on_one_thread and machine_cores > 1 and not threads_explicitly_one:
        return (f"本机 {machine_cores} 核、线程数没显式设成 1，这一趟跑了 {freshly_run_slices} 片却只起了 1 个工作线程"
                f"（多半是线程数没传进去）：{finished_line}"), ""
    if freshly_run_slices == 0:
        note = f"{prefix}：全部 {slices} 片从进度文件读回，这一趟没起工作线程"
    else:
        note = f"{prefix}：{worker_threads} 个工作线程跑了 {freshly_run_slices} 片、从进度文件读回 {resumed_slices} 片（共 {slices} 片）"
    return None, note


# merge 那一行逐片报的数（crates/singlefs-checker-tier/src/crash.rs 的 merge_the_shard_ledgers，逗号分隔、按第几片排）
SHARD_FIELD_NAMES = ("worker_threads", "configured_worker_threads", "worker_threads_sources", "available_parallelism",
                     "resumed_slices", "freshly_run_slices")


def judge_threads_of_each_shard(prefix, finished_line, fields, default_threads):
    """双机分片 merge 那一趟的 CRASH_POINT_REPLAY_PARALLEL_FINISHED（带 shards= 与逐片的 shard_…=）：每一片照单机的判法判。返回 (原因或 None, 一句说明)。
    判红：逐片字段缺、片数不一、不是整数；某一片跑了片却报 0 个线程；某一片这一趟跑了至少两片、只起了 1 个工作线程、那台机器多于 1 核、
    那一片的线程数不是显式设成 1（来源是环境变量、配的是 1）；某一片的来源是用例自己取的默认而配的数不等于 default_threads（每台默认重活线程数，
    两台各取一份；弄坏开关 threads-default-unchecked 下不判）。机器核数与「显式设成 1」都取那一片账本里记的，不取 merge 这台的。"""
    try:
        shard_count = int(fields["shards"])
        columns = {name: fields[f"shard_{name}"].split(",") for name in SHARD_FIELD_NAMES}
        if shard_count < 1 or any(len(values) != shard_count for values in columns.values()):
            raise ValueError("片数不一")
        numbers = {name: [int(value) for value in values] for name, values in columns.items() if name != "worker_threads_sources"}
    except (KeyError, ValueError):
        return (f"{prefix} 对得上的 CRASH_POINT_REPLAY_PARALLEL_FINISHED 带 shards=，逐片的 shard_…= 字段缺、片数与 shards= 不一或不是整数：{finished_line}"), ""
    notes = []
    for shard_index in range(shard_count):
        spawned = numbers["worker_threads"][shard_index]
        configured = numbers["configured_worker_threads"][shard_index]
        source = columns["worker_threads_sources"][shard_index]
        cores = numbers["available_parallelism"][shard_index]
        resumed = numbers["resumed_slices"][shard_index]
        freshly_run = numbers["freshly_run_slices"][shard_index]
        shard_text = f"第 {shard_index}/{shard_count} 片"
        if freshly_run > 0 and spawned == 0:
            return f"{shard_text}跑了 {freshly_run} 片，却报起了 0 个工作线程：{finished_line}", ""
        if source in DEFAULT_WORKER_THREAD_SOURCES and configured != default_threads and not break_is_set("threads-default-unchecked"):
            return (f"{shard_text}：线程数没显式设（来源 {source}，用例自己取的默认），配的 {configured} 个不等于每台默认重活线程数 {default_threads}"
                    f"（.claude/heavy-thread-default；多半是线程变量没传进去，或用例的默认还按核数算）：{finished_line}"), ""
        explicitly_one = source == "environment_variable" and configured == 1
        if freshly_run >= 2 and spawned == 1 and cores > 1 and not explicitly_one:
            return (f"{shard_text}：那台机器 {cores} 核、线程数没显式设成 1，跑了 {freshly_run} 片却只起了 1 个工作线程"
                    f"（多半是线程数没传进去）：{finished_line}"), ""
        notes.append(f"{shard_text} {spawned} 个线程跑了 {freshly_run} 片、读回 {resumed} 片（那台 {cores} 核）")
    return None, f"{prefix}：双机分片 {shard_count} 片再 merge，{'；'.join(notes)}"


def judge_crash_case_log(case, log_text, machine_cores, threads_explicitly_one, default_threads):
    """判 --full 跑一条用例的日志：返回 (原因的清单, 记录行（门禁原样打出来）, 线程那几句)。原因的清单为空才算判绿。
    default_threads 是每台默认重活线程数（registered_heavy_threads），判线程来源是默认的那几行用。"""
    log_lines = log_text.split("\n")
    problems, recorded_lines, thread_notes = [], [], []
    results = [line for line in log_lines if line.startswith("test result: ")]
    if len(results) != 1 or not PASSED_ONE_TEST_FORM.match(results[0]):
        problems.append(f"读不到恰好一行「test result: ok. 1 passed; 0 failed; …」：过滤到 {case.function} 应当恰好跑一条用例，"
                        f"读到 {len(results)} 行 test result（{'｜'.join(results[:3]) or '一行都没有'}）")
    else:
        recorded_lines.append("test_result=" + results[0])
    count_line_of_prefix = {}
    for prefix in case.count_lines:
        found = [line for line in log_lines if line.startswith(prefix + " ")]
        if len(found) != 1:
            problems.append(f"以「{prefix} 」开头的计数行要恰好一行，读到 {len(found)} 行")
            continue
        count_line_of_prefix[prefix] = found[0]
        recorded_lines.append(found[0])
    for prefix in case.exhaustive_lines:
        line = count_line_of_prefix.get(prefix)
        if line is not None and fields_of_line(line).get("exhaustive") != "true":
            problems.append(f"{prefix} 那一行不带 exhaustive=true，不是全量（枚举到的状态数要等于闭式 1 + Σ(2^|段| − 1)）：{line}")
    for prefix in case.thread_lines:
        line = count_line_of_prefix.get(prefix)
        if line is None:
            continue
        problem, note = judge_worker_threads(prefix, line, log_lines, machine_cores, threads_explicitly_one, default_threads)
        if problem:
            problems.append(problem)
        else:
            thread_notes.append(note)
    recorded_lines += ["parallel_finished=" + line for line in log_lines if line.startswith(CRASH_POINT_REPLAY_PARALLEL_FINISHED_PREFIX)]
    if not break_is_set("single-worker-threads-field"):
        recorded_lines.append("started_worker_threads=" + started_worker_threads_text(case, thread_notes))
    return problems, recorded_lines, thread_notes


def started_worker_threads_text(case, thread_notes):
    """记录行里「实际起的工作线程」那一格：登记了 threads= 的取判线程那几句（CRASH_POINT_REPLAY_PARALLEL_FINISHED 里记的 worker_threads），
    没登记的写「读不到」——配的是几个另记在 configured_worker_threads=，那不是起了几个。"""
    if thread_notes:
        return "；".join(thread_notes)
    if case.thread_lines:
        return "读不到：日志没判绿，线程那一格没判"
    return ("读不到：这条用例没登记 threads=，门禁不读它起了几个工作线程（配的线程数在 configured_worker_threads=，不是起了几个；"
            "日志里有 CRASH_POINT_REPLAY_PARALLEL_FINISHED 的另原样记在 parallel_finished=）")


def configured_worker_threads_text(case, machine_cores, threads, threads_origin):
    """记录行里「配的线程数」那一格：传给用例的线程变量（这条用例读的那一个，登记行的 threads-variable=，没登记是 SINGLEFS_CRASH_POINT_REPLAY_THREADS）
    与它的值、配的数是显式设的（调用方设了 SINGLEFS_CRASH_POINT_REPLAY_THREADS）还是取的每台默认重活线程数、本机几核。"""
    origin = "显式设的" if threads_origin == "explicit" else "没设，取每台默认重活线程数"
    return f"{case.thread_variable}={threads}（{origin}），本机 {machine_cores} 核"


def machine_core_count(environment):
    """本机核数：os.cpu_count() 与这个进程的 CPU 亲和（os.sched_getaffinity）取小，不认 OMP_NUM_THREADS 这一类环境变量。
    读不到抛 InputManifestError。弄坏开关 cores-from-nproc 下照旧取 nproc（在 environment 的 PATH 里找，它认 OMP_NUM_THREADS）。"""
    if break_is_set("cores-from-nproc"):
        nproc = shutil.which("nproc", path=environment.get("PATH")) or "nproc"
        try:
            completed = subprocess.run([nproc], capture_output=True, text=True, errors="replace", env=dict(environment))
        except OSError as error:
            raise InputManifestError(f"nproc 起不来：{error}") from error
        cores_text = completed.stdout.strip()
        if completed.returncode != 0 or not POSITIVE_INTEGER_FORM.fullmatch(cores_text):
            raise InputManifestError(f"nproc 退 {completed.returncode}、打了「{cores_text}」，不是正整数")
        return int(cores_text)
    counts = [count for count in (os.cpu_count(), len(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else None) if count]
    if not counts:
        raise InputManifestError("os.cpu_count() 与 CPU 亲和都读不到本机核数")
    return min(counts)


# 与 scripts/hosts/heavy_threads.py 的 default_heavy_threads 同一个判法、在这里另写一份（这一份不 import scripts/hosts 下的库）。
# 两份由 --selftest 的「每台默认重活线程数：与 heavy_threads.py 对拍」逐格比
def registered_heavy_threads(environment, registration_file=None):
    """每台默认重活线程数：environment 里 SINGLEFS_THREAD_CAP 设了（非空）照它（要是正整数），否则读 registration_file（不给是 HEAVY_THREAD_DEFAULT_FILE）
    第一条非注释行「<正整数><空白>#<理由>」的那个数。取不了抛 InputManifestError（原因带出路），不退回按核数算。"""
    cap = environment.get(THREAD_CAP_VARIABLE, "")
    if cap:
        if not POSITIVE_INTEGER_FORM.fullmatch(cap):
            raise InputManifestError(f"{THREAD_CAP_VARIABLE}={cap} 不是正整数，取不了每台默认重活线程数（设成正整数，或 unset 它取 .claude/heavy-thread-default 的登记值）")
        return int(cap)
    path = registration_file or HEAVY_THREAD_DEFAULT_FILE
    try:
        with open(path, encoding="utf-8", errors="surrogateescape") as handle:
            lines = handle.read().split("\n")
    except OSError as error:
        raise InputManifestError(f"每台默认重活线程数的登记文件 {path} 读不了：{error}（在仓根建 .claude/heavy-thread-default，"
                                 "第一条非注释行写「31  # <理由>」）") from error
    if lines and lines[-1] == "":
        lines.pop()
    for number, line in enumerate(lines, 1):
        if HEAVY_THREAD_COMMENT_OR_BLANK_FORM.match(line):
            continue
        found = HEAVY_THREAD_REGISTRATION_FORM.match(line)
        if found:
            return int(found.group(1))
        raise InputManifestError(f"{path} 第 {number} 行「{line}」不是「<正整数>  # <理由>」（数是正整数，数与 # 之间至少一个空格，# 后面写理由）")
    raise InputManifestError(f"{path} 里没有非注释行，读不出每台默认重活线程数（第一条非注释行写「31  # <理由>」）")


def crash_case_worker_threads(case, environment):
    """--full 跑崩溃枚举用例的线程数：(本机核数, 线程数, "explicit" 或 "default")。调用方设了 SINGLEFS_CRASH_POINT_REPLAY_THREADS（非空）就用它，
    没设取每台默认重活线程数（registered_heavy_threads）；用例读的线程变量（case.thread_variable）调用方设着什么都不认，由 crash-case-command 盖成这个数。
    本机核数（machine_core_count）只用来判「本机多于 1 核」，不定线程数（弄坏开关 threads-default-from-cores 下照改前拿它当线程数）。
    crash-case-command、crash-case-judge 各自现取（checker-tier-crash-point-replay 不转线程参数）。
    核数读不到、SINGLEFS_CRASH_POINT_REPLAY_THREADS 不是正整数、每台默认重活线程数取不了，都抛 InputManifestError。"""
    cores = machine_core_count(environment)
    configured = environment.get(CRASH_POINT_REPLAY_THREADS_VARIABLE, "")
    if not configured:
        if break_is_set("threads-default-from-cores"):
            return cores, cores, "default"
        return cores, registered_heavy_threads(environment), "default"
    if not POSITIVE_INTEGER_FORM.fullmatch(configured):
        raise InputManifestError(f"{CRASH_POINT_REPLAY_THREADS_VARIABLE}={configured} 不是正整数")
    return cores, int(configured), "explicit"


def crash_case_launch(root, case, start_over, environment, blocks_query=False):
    """跑一条崩溃枚举用例的命令与环境（门禁 checker-tier-crash-point-replay 原样执行）：返回 (本机核数, 线程数, "explicit" 或 "default", 整条命令的词)。
    库：env 里清掉调用方的 SINGLEFS_CRASH_AMPLIFICATION_LIBRARY（用例照家目录下那一份长期库；弄坏开关 keep-caller-library 下不清）；start_over 时另设
    SINGLEFS_CRASH_LIBRARY_WRITE_ONLY=1（只写不读：全部块现判、照键写回同一份库），blocks_query 时另设 SINGLEFS_CRASH_CASE_BLOCKS_QUERY=1（只算块键、查库，一块都不判；
    接口在文件头「崩溃枚举用例的复用不在这里」）。分片的两个变量 SINGLEFS_CRASH_AMPLIFICATION_SHARE、SINGLEFS_CRASH_AMPLIFICATION_IMPORT 一律清掉
    （这一趟单机跑整条流；两台分片只由 scripts/crash/crash-point-replay-shard-run.sh 自己设）。线程数设进 SINGLEFS_CRASH_POINT_REPLAY_THREADS，这条用例读的线程变量
    （登记行的 threads-variable=）不是它时另设成同一个数，盖掉调用方环境里的（弄坏开关 thread-variable-ignored 下只设 SINGLEFS_CRASH_POINT_REPLAY_THREADS）。
    cargo test 带的特性：登记行的 features=，多机配置写了 ENABLE_GPU=1 的一律加 gpu（崩溃枚举用例的判段也上卡）；特性非空时 env 里另设 BINDGEN_EXTRA_CLANG_ARGS；
    特性里有 gpu 的整条前面接 bash scripts/crash/gpu-quiesce-run.sh --（清场、不复原），环境里交清场之后的显卡先后与多机配置的路径（crash_case_cargo_features），
    特性里有 gpu 的，测试参数收尾另带 --test-threads 1（弄坏开关 gpu-test-threads-unset 下不带）；GPU 关的命令不带。
    线程数取不到、多机配置在而读不了抛 InputManifestError。"""
    machine_cores, threads, threads_origin = crash_case_worker_threads(case, environment)
    thread_settings = [f"{CRASH_POINT_REPLAY_THREADS_VARIABLE}={threads}"]
    if case.thread_variable != CRASH_POINT_REPLAY_THREADS_VARIABLE:
        thread_settings.append(f"{case.thread_variable}={threads}")
    # env 的 -u 要写在第一个 NAME=VALUE 之前（之后的 -u 被当成要起的命令）
    share_unsets = ["-u", CRASH_SHARE_VARIABLE, "-u", CRASH_IMPORT_VARIABLE]
    library_unsets = [] if break_is_set("keep-caller-library") else ["-u", CRASH_LIBRARY_DIRECTORY_VARIABLE]
    library_unsets += ["-u", CRASH_LIBRARY_WRITE_ONLY_VARIABLE, "-u", CRASH_CASE_BLOCKS_QUERY_VARIABLE]
    library_setting = ([f"{CRASH_LIBRARY_WRITE_ONLY_VARIABLE}=1"] if start_over else []) + ([f"{CRASH_CASE_BLOCKS_QUERY_VARIABLE}=1"] if blocks_query else [])
    features, quiesced, configuration_file = crash_case_cargo_features(root, case, environment)
    feature_words = ["--features", ",".join(features)] if features else []
    bindgen_setting = bindgen_include_setting(root) if features else []
    quiesce_unsets, quiesce_settings = [], []
    if quiesced:
        cards = (configuration_value(configuration_file, "GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED")
                 or configuration_value(configuration_file, "GPU_CARDS_BY_PRIORITY"))
        quiesce_unsets = ["-u", "GPU_COMPUTATION_CLASSES", "-u", "SINGLEFS_GPU_CARDS_CONFIG"] + ([] if cards else ["-u", "GPU_CARDS_BY_PRIORITY"])
        quiesce_settings = ([f"GPU_CARDS_BY_PRIORITY={cards}"] if cards else [])
        if os.path.isfile(configuration_file):
            quiesce_settings.append(f"{MULTI_HOST_CONFIGURATION_VARIABLE}={os.path.abspath(configuration_file)}")
    # GPU 对拍的测试各占卡的显存，不并行起：特性里有 gpu 的（开了 GPU 的）带 --test-threads 1（.claude/rules/verification.md「两档：轻型与重型」）；GPU 关的不加。弄坏开关 gpu-test-threads-unset 下不加
    test_threads_words = ["--test-threads", "1"] if GPU_FEATURE in features and not break_is_set("gpu-test-threads-unset") else []
    command = ["env", *quiesce_unsets, *share_unsets, *library_unsets, *library_setting, *thread_settings, *bindgen_setting, *quiesce_settings,
               "cargo", "test", "--release", "-p", case.package, *feature_words, "--test", case.target,
               "--", "--include-ignored", "--exact", case.function, "--nocapture", *test_threads_words]
    if quiesced:
        command = ["bash", os.path.join(root, GPU_QUIESCE_SCRIPT), "--", *command]
    return machine_cores, threads, threads_origin, command


def multi_host_configuration_file(root, environment):
    """多机配置在哪（同 scripts/crash/crash-amplification-lib.sh 的 crash_amplification_multi_host_configuration_file）：
    调用方环境里 SINGLEFS_MULTI_HOST_CONFIG 设了照它；没设取 <root> 的 git common-dir 的上一层（主工作树的根）下的 multi-host.env，取不到 common-dir 退回 <root>/multi-host.env。
    文件在不在不判。"""
    configured = environment.get(MULTI_HOST_CONFIGURATION_VARIABLE)
    if configured:
        return configured
    common_directory = git_common_directory(root)
    if common_directory is not None:
        return os.path.join(os.path.dirname(common_directory), MULTI_HOST_CONFIGURATION_NAME)
    return os.path.join(root, MULTI_HOST_CONFIGURATION_NAME)


def configuration_value(path, key):
    """配置文件里 <键>= 开头的第一行的值（去掉两头成对的一层引号，同 crash_amplification_configuration_value）；文件不在、键没写是空串，在而读不了抛 InputManifestError。"""
    if not os.path.isfile(path):
        return ""
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except (OSError, UnicodeDecodeError) as error:
        raise InputManifestError(f"多机配置 {path} 读不了（{error}），判不出 {key}")
    for line in lines:
        line = line.strip()
        if not line.startswith(key + "="):
            continue
        value = line[len(key) + 1:].strip()
        if len(value) >= 2 and value[0] == value[-1] and value[0] in "'\"":
            value = value[1:-1]
        return value
    return ""


def crash_case_cargo_features(root, case, environment):
    """这条用例的 cargo 特性：返回 (特性列表, 是否整条经清场包装起, 多机配置文件的路径)。
    没登记 features= 又没开 GPU 的是空列表、不经包装；登记了的照登记；多机配置里 ENABLE_GPU 等于 1 的一律另加 gpu（崩溃枚举用例的判段也上卡），特性里有 gpu 的（ENABLE_GPU 等于 1）经包装起。
    弄坏开关 features-not-passed：登记的 features= 不认；features-passed-to-every-case：没登记的也当 verdict-store；gpu-case-not-quiesced：不经包装。"""
    features = list(case.features)
    if break_is_set("features-not-passed"):
        features = []
    elif break_is_set("features-passed-to-every-case") and not features:
        features = [VERDICT_STORE_FEATURE]
    configuration_file = multi_host_configuration_file(root, environment)
    gpu_enabled = configuration_value(configuration_file, "ENABLE_GPU") == "1"
    if gpu_enabled and GPU_FEATURE not in features:
        features.append(GPU_FEATURE)
    quiesced = gpu_enabled and GPU_FEATURE in features and not break_is_set("gpu-case-not-quiesced")
    return features, quiesced, configuration_file


def bindgen_include_setting(root):
    """librocksdb-sys 的 bindgen 找头文件的目录：崩溃放量那份库（BINDGEN_LIBRARY_SCRIPT）里 CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS 的值，打成 env 的一个词；库不在、读不到是空（照调用方环境）。"""
    try:
        with open(os.path.join(root, BINDGEN_LIBRARY_SCRIPT), encoding="utf-8") as handle:
            found = BINDGEN_ASSIGNMENT_FORM.search(handle.read())
    except (OSError, UnicodeDecodeError):
        return []
    return [f"{BINDGEN_VARIABLE}={found.group(1)}"] if found else []



# ── 构建环境的生效值（Rust 入口的工具链行）────────────────────────────────────

def configuration_layers(closure_root, environment):
    """cargo 读的配置层，从近到远：闭包根往上每一层的 .cargo/config(.toml)，最后 CARGO_HOME 的（同时是某一层的只算一次）。读不成 TOML 的交一行原样的「读不出」。"""
    cargo_home = environment.get("CARGO_HOME") or os.path.join(environment.get("HOME") or os.path.expanduser("~"), ".cargo")
    cargo_home_files = [os.path.join(cargo_home, name) for name in CARGO_CONFIGURATION_FILE_NAMES]
    cargo_home_real = {os.path.realpath(path) for path in cargo_home_files}
    found = []
    directory = os.path.realpath(closure_root)
    while True:
        for name in CARGO_CONFIGURATION_FILE_NAMES:
            path = os.path.join(directory, ".cargo", name)
            if os.path.isfile(path) and os.path.realpath(path) not in cargo_home_real:
                found.append((path, directory))
        parent = os.path.dirname(directory)
        if parent == directory:
            break
        directory = parent
    for path in cargo_home_files:
        if os.path.isfile(path):
            found.append((path, os.path.dirname(cargo_home)))
    return found


def build_environment_lines(root, environment=None):
    """构建环境的生效值，旧的那种形状 [(名字, 内容字节)]（scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py 的 build_environment_lines 用它）：每一行生效值各一项。算不出抛 InputManifestError。"""
    try:
        return [(f"<构建环境：{line}>", line.encode("utf-8", "surrogateescape")) for line in build_environment_setting_lines(root, environment)]
    except UnitKeyError as error:
        raise InputManifestError(str(error)) from error


def build_environment_setting_lines(closure_root, environment=None):
    """构建环境的生效值（文件头「单元键」那一段的工具链行）：[行]。不写绝对路径：同一批内容在主工作区与临时 worktree 里算出同一份。"""
    environment = os.environ if environment is None else environment
    lines, layers, unreadable = [], [], []
    argument_directories = runner_argument_directories(closure_root)
    for path, base_directory in configuration_layers(closure_root, environment):
        try:
            with open(path, "rb") as handle:
                content = handle.read()
            settings = tomllib.loads(content.decode("utf-8"))
        except (OSError, UnicodeDecodeError, ValueError):
            unreadable.append(hashlib.sha256(read_configuration_file(path) if os.path.isfile(path) else b"").hexdigest())
            continue
        layers.append(settings)
        for label, program_bytes in configured_program_lines("cargo 配置", content, base_directory, closure_root, environment, argument_directories):
            lines.append(f"build-environment program {label}: {hashlib.sha256(program_bytes).hexdigest()}")
    flattened = []
    flattened_setting_lines("", merged_configuration(layers), flattened)
    lines += [f"build-environment config {line}" for line in flattened]
    lines += [f"build-environment config 读不出的一层 {digest}" for digest in unreadable]
    directory = os.path.realpath(closure_root)
    while True:
        for name in TOOLCHAIN_FILE_NAMES:
            path = os.path.join(directory, name)
            if os.path.isfile(path):
                try:
                    with open(path, "rb") as handle:
                        content = handle.read()
                    parsed = tomllib.loads(content.decode("utf-8")) if name.endswith(".toml") else {"channel": content.decode("utf-8").strip()}
                except (OSError, UnicodeDecodeError, ValueError):
                    parsed = {"读不出": hashlib.sha256(read_configuration_file(path)).hexdigest()}
                toolchain_settings = []
                flattened_setting_lines(name, parsed, toolchain_settings)
                lines += [f"build-environment toolchain-file {line}" for line in toolchain_settings]
        parent = os.path.dirname(directory)
        if parent == directory:
            break
        directory = parent
    for variable in sorted(environment):
        if variable in BUILD_ENVIRONMENT_VARIABLES_LEFT_OUT:
            continue
        if variable in BUILD_ENVIRONMENT_EXACT_VARIABLES or any(form.match(variable) for form in BUILD_ENVIRONMENT_VARIABLE_FORMS):
            lines.append(f"build-environment env {variable}={environment[variable]}")
    for variable in sorted(environment):
        if variable in BUILD_ENVIRONMENT_PROGRAM_VARIABLES or any(form.match(variable) for form in BUILD_ENVIRONMENT_PROGRAM_VARIABLE_FORMS):
            contents = program_contents(environment[variable].split(), closure_root, environment, argument_directories)
            lines.append(f"build-environment program env {variable}: {hashlib.sha256(contents).hexdigest()}")
    for variable in BUILD_ENVIRONMENT_COMPILER_VARIABLES:
        if variable in environment:
            try:
                version = compiler_version(environment[variable], closure_root, environment, f"{variable}={environment[variable]}")
            except InputManifestError as error:
                raise UnitKeyError(str(error)) from error
            lines.append(f"build-environment compiler {variable}: {version.decode('utf-8', 'replace').strip()}")
    return lines


# ── 测试函数（tests-of:<包> 的展开；门禁按条入库的那几格要的名单）──────────────

class TestFunction:
    def __init__(self, entry, target_id, target_root, libtest_name, state):
        self.entry = entry
        self.target_id = target_id
        self.target_root = target_root
        self.libtest_name = libtest_name
        self.state = state
        self.name = entry   # 判定的名字：同一个函数编进几个目标（lib 与 bin 都 mod 了同一份文件）时另接「@<目标号>」，各是一条判定

    def line(self):
        return "\t".join([self.entry, self.target_id, self.target_root, self.libtest_name, self.state])


def unit_text_before_body(root, unit):
    """单元从它的第一个外属性起到第一个 { 之前的原文（看属性用）。"""
    try:
        with open(os.path.join(root, unit.file), encoding="utf-8", errors="replace") as handle:
            lines = handle.read().split("\n")[unit.first_line - 1:unit.last_line]
    except OSError:
        return ""
    text = "\n".join(lines)
    return text.split("{", 1)[0]


def test_functions_of_package(context, package, closure_root=None, features=(), profile=None):
    """这个包 lib、各 bin 与各测试目标里带 #[test] 的函数（cargo test --tests 跑的就是这几种目标）：[TestFunction]，按（目标号、libtest 全名）排。
    state：compiled、not-compiled（这一组特性下 cfg 关着）、ignored。libtest 认的全名 = 那份文件在目标里的模块路径 + 单元的限定名（内联 mod 已在限定名里）。
    同一个函数编进几个目标的，名字另接「@<目标号>」（入口不变）。"""
    library = function_units()
    closure_root = closure_root or context.root
    workspace = context.workspace(closure_root)
    try:
        crate = workspace.crate_by_package(package)
    except library.UsageError as error:
        raise RegistrationError(f"包 {package} 不在工作区里：{error}") from error
    cfg = rust_cfg_for_profile(closure_root, profile)
    found = []
    for target in sorted(workspace.targets.values(), key=lambda target: target.id):
        if target.crate is not crate or target.kind not in ("lib", "bin", "test"):
            continue
        for path, (module_path, _chain) in sorted(target.files.items()):
            for unit in workspace.files[path].units:
                if unit.kind != "fn":
                    continue
                head = unit_text_before_body(closure_root, unit)
                if not TEST_ATTRIBUTE_FORM.search(head):
                    continue
                libtest_name = "::".join(list(module_path) + [unit.name])
                entry = f"{path}::{unit.name}"
                if TEST_IGNORE_FORM.search(head):
                    state = "ignored"
                else:
                    try:
                        context.rust_closure(closure_root, workspace, unit, target.id, cfg, list(features), False)
                        state = "compiled"
                    except library.UsageError:
                        state = "not-compiled"
                target_root = posixpath_relative(target.root_file, crate.directory)
                found.append(TestFunction(entry, target.id, target_root, libtest_name, state))
    entry_counts = collections.Counter(function.entry for function in found)
    for function in found:
        if entry_counts[function.entry] > 1:
            function.name = f"{function.entry}@{function.target_id}"
    return sorted(found, key=lambda item: (item.target_id, item.libtest_name))


def posixpath_relative(path, directory):
    if not directory:
        return path
    return path[len(directory) + 1:] if path.startswith(directory + "/") else path


def expand_entries(context, entries):
    """登记的入口展开成一条条判定的入口：tests-of:<包> 展开成那个包里不标 ignore、这一组 cfg 下编得进的测试函数；别的原样。"""
    expanded = []
    for entry in entries:
        if entry.startswith("tests-of:"):
            expanded += [item.entry for item in test_functions_of_package(context, entry[len("tests-of:"):]) if item.state == "compiled"]
        else:
            expanded.append(entry)
    return list(dict.fromkeys(expanded))


# ── 门禁的复用答法（gate-reuse）────────────────────────────────────────────────

def gate_reuse(root, key, request_file=None):
    """返回 (退出码, 一句依据, 要现判的名字)。退 EXIT_GATE_MUST_RUN 要跑，EXIT_GATE_MAY_SKIP 可跳过。"""
    if os.environ.get("SINGLEFS_GATE_FULL", "") == "1":
        return EXIT_GATE_MUST_RUN, "强制全跑（SINGLEFS_GATE_FULL=1，只写不读）", []
    if break_is_set("tree-compare-reuse"):
        return tree_compare_reuse(root, key)
    library = VerdictLibrary(root, key)
    if library.directory is None:
        return EXIT_GATE_MUST_RUN, f"判不出来：{root} 不是 git 工作树（取不到 git common-dir），判定库没处读，按要跑处理", []
    context = KeyContext(root)
    if request_file:
        with open(request_file, encoding="utf-8", errors="surrogateescape") as handle:
            requests = [parse_request_line(line) for line in handle if line.strip() and not line.startswith("#")]
    else:
        recorded = dict(library.items())
        requests = []
        for entry in expand_entries(context, resolve_entries(context.rows(), key)):
            record = recorded.get(entry)
            requests.append(parse_request_line(record["request"]) if record and record.get("request") else Request(entry, entry, []))
    if not requests:
        return EXIT_GATE_MUST_RUN, f"{REGISTRATION_TABLE} 里 {key} 一个入口都没登记，按要跑处理", []
    to_judge, notes = [], []
    for request in requests:
        unit_key = compute_unit_key(context, request)
        decision, reason = library.lookup(request, unit_key)
        if decision != "reuse":
            to_judge.append(request.name)
            notes.append(f"{request.name}：{reason}")
    if not to_judge:
        return EXIT_GATE_MAY_SKIP, f"{key} 登记的 {len(requests)} 条判定在判定库 {library.directory} 里都有键相同的判定（复用 {len(requests)}、现判 0）", []
    shown = "；".join(notes[:CHANGED_UNITS_SHOWN_AT_MOST])
    return EXIT_GATE_MUST_RUN, (f"{key} 登记的 {len(requests)} 条判定里键对不上的 {len(to_judge)} 条要现判"
                                f"（复用 {len(requests) - len(to_judge)}）：{shown}{' …' if len(notes) > CHANGED_UNITS_SHOWN_AT_MOST else ''}"), to_judge


def tree_compare_reuse(root, key):
    """弄坏开关 tree-compare-reuse：改前的复用路——refs/sop/staged-green 那棵树（没有取 HEAD 的树）与工作区在登记入口的文件上逐字相同就答可跳过。"""
    entries = resolve_entries(read_registration_rows(root), key)
    files = sorted({path for path in (entry_file(entry) for entry in entries) if path})
    for reference in (f"{GATE_GREEN_TREE_RECORD}^{{tree}}", "HEAD^{tree}"):
        code, _output = git_output(root, "rev-parse", "-q", "--verify", reference)
        if code == 0:
            break
    else:
        return EXIT_GATE_MUST_RUN, "（弄坏开关 tree-compare-reuse）没有可比的树", files
    code, output = git_output(root, "diff", "--name-only", reference, "--", *files)
    if code == 0 and not output.strip():
        return EXIT_GATE_MAY_SKIP, f"（弄坏开关 tree-compare-reuse）登记入口的文件与 {reference} 逐字相同", []
    return EXIT_GATE_MUST_RUN, f"（弄坏开关 tree-compare-reuse）与 {reference} 不同：{output.strip()}", files


# ── 复跑表每一行的行键（replay_row_request；文件头「复跑表每一行的行键」）──────────

def research_binary_sources(root):
    """scripts/experiments/Cargo.toml 的 [[bin]]：{名字: scripts/experiments 起的源文件}；清单不在或读不出给空表。"""
    sections = manifest_sections(os.path.join(root, REPLAY_RESEARCH_DIRECTORY, "Cargo.toml")) or {}
    return {entry["name"]: os.path.normpath(entry["path"])
            for entry in sections.get("bin") or [] if isinstance(entry, dict) and isinstance(entry.get("name"), str) and isinstance(entry.get("path"), str)}


def research_package_name(root):
    """scripts/experiments/Cargo.toml 的 [package] name（研究工作区的包）；清单不在或读不出交 None。"""
    package = (manifest_sections(os.path.join(root, REPLAY_RESEARCH_DIRECTORY, "Cargo.toml")) or {}).get("package")
    return package.get("name") if isinstance(package, dict) and isinstance(package.get("name"), str) else None


def workspace_binary_entry(root, package, binary):
    """仓根工作区里包 package 的 bin binary 的 main 入口（仓根起）：[[bin]] 写了 path 的取它，没写的取 src/bin/<名>.rs 或 src/bin/<名>/main.rs；找不到交 None。"""
    packages = workspace_package_directories(root)
    if package not in packages:
        return None
    directory = packages[package]
    sections = manifest_sections(os.path.join(root, directory, "Cargo.toml")) or {}
    for entry in sections.get("bin") or []:
        if isinstance(entry, dict) and entry.get("name") == binary and isinstance(entry.get("path"), str):
            return os.path.normpath(os.path.join(directory, entry["path"])) + "::main"
    for candidate in (os.path.join(directory, "src", "bin", binary + ".rs"), os.path.join(directory, "src", "bin", binary, "main.rs")):
        if os.path.isfile(os.path.join(root, candidate)):
            return os.path.normpath(candidate) + "::main"
    return None


def shell_function_text(context, function):
    """replay.sh 里一个顶格函数的闭包（lib_function_units 的 shell 闭包）里 replay.sh 那几个单元的原文拼起来：找驱动点名的 bin、包与 CHECKER_TIER_FEATURES 用。算不出交 None。"""
    try:
        closure = context.script_closure(context.root, f"{REPLAY_SCRIPT}::{function}")
    except UnitKeyError:
        return None
    return "\n".join(unit_source_text(context.root, unit) for unit in closure.order
                     if getattr(unit, "language", None) == "shell" and unit.file == REPLAY_SCRIPT and not getattr(unit, "data", False))


def unit_source_text(root, unit):
    try:
        with open(os.path.join(root, unit.file), encoding="utf-8", errors="replace") as handle:
            return "\n".join(handle.read().split("\n")[unit.first_line - 1:unit.last_line])
    except OSError:
        return ""


def replay_row_request(context, row_text, gpu_switch, registered_entries, without_product=False):
    """复跑表一行的判定请求：交 (Request 列表, 拒算说明的清单)。一行可能有几个入口（驱动 + 它点名的装置）：每个入口一条请求，行键是它们各自的键再合起来。"""
    root = context.root
    match = REPLAY_ROW_FORM.match(row_text)
    experiment, bin_text, arguments_text, product, compare = match.group(1, 2, 3, 4, 5)
    common = [f"line:arguments={arguments_text}", f"line:compare={compare}"]
    common += [f"judge:{REPLAY_SCRIPT}::{name}" for name in REPLAY_JUDGING_FUNCTIONS]
    if not without_product:
        common.append(f"file:{RESULTS_DIRECTORY}/{product}")
        if break_is_set("row-key-with-product-name"):
            common.append(f"line:product-name={product}")
    binaries = research_binary_sources(root)
    entries, refusals = [], []
    uses_gpu = False
    if bin_text.startswith("@"):
        driver = bin_text[1:]
        entries.append((f"{REPLAY_SCRIPT}::{driver}", []))
        text = shell_function_text(context, driver)
        if text is None:
            refusals.append(f"refused\treplay-driver-unreadable\t{REPLAY_SCRIPT}\t{driver}")
            text = ""
        uses_gpu = REPLAY_GPU_FEATURE_VARIABLE in text
        for name in sorted(binaries):
            if re.search(r"(?<![A-Za-z0-9_-])" + re.escape(name) + r"(?![A-Za-z0-9_-])", text):
                entries.append((f"{binaries[name]}::main", [f"root:{REPLAY_RESEARCH_DIRECTORY}", "profile:release"]))
        named_bins = REPLAY_BIN_MENTION.findall(text)
        research_package = research_package_name(root)
        for package in sorted(set(REPLAY_PACKAGE_MENTION.findall(text))):
            if package == research_package and not break_is_set("research-package-unresolved"):
                # 研究工作区的包（驱动经 --manifest-path scripts/experiments/Cargo.toml 编它）：--bin 的名字解到那里 [[bin]] 的源码，与字面点名研究二进制同一种入口
                found = [f"{binaries[name]}::main" for name in named_bins if name in binaries]
                if not found:
                    refusals.append(f"refused\treplay-package-without-binary\t{REPLAY_SCRIPT}\t{driver} 点名了 -p {package}，点不出 bin")
                for entry in found:
                    entries.append((entry, [f"root:{REPLAY_RESEARCH_DIRECTORY}", "profile:release"]))
                continue
            found = [workspace_binary_entry(root, package, name) for name in named_bins]
            found = [entry for entry in found if entry]
            if not found:
                refusals.append(f"refused\treplay-package-without-binary\t{REPLAY_SCRIPT}\t{driver} 点名了 -p {package}，点不出 bin")
            for entry in found:
                features = []
                if package == CHECKER_TIER_PACKAGE and gpu_switch == "1" and uses_gpu:
                    features = [f"feature:{CHECKER_TIER_PACKAGE}/{VERDICT_STORE_FEATURE}", f"feature:{CHECKER_TIER_PACKAGE}/{GPU_FEATURE}"]
                entries.append((entry, ["profile:release", *features]))
    elif bin_text in binaries:
        entries.append((f"{binaries[bin_text]}::main", [f"root:{REPLAY_RESEARCH_DIRECTORY}", "profile:release"]))
    else:
        refusals.append(f"refused\treplay-binary-unknown\t{REPLAY_SCRIPT}\t{bin_text} 不是 scripts/experiments/Cargo.toml 的 [[bin]]、也不是 @<驱动函数>")
    if uses_gpu and not break_is_set("row-key-without-gpu"):
        common.append(f"line:gpu={gpu_switch}")
    for entry in registered_entries:
        if entry.startswith(REPLAY_RESEARCH_DIRECTORY + "/") and entry.split("::", 1)[0].endswith(".rs"):
            entries.append((entry[len(REPLAY_RESEARCH_DIRECTORY) + 1:], [f"root:{REPLAY_RESEARCH_DIRECTORY}", "profile:release"]))
        else:
            entries.append((entry, []))
    identity = "|".join((experiment, bin_text, arguments_text))
    requests = [Request(identity, entry, tokens + common) for entry, tokens in dict((entry, tokens) for entry, tokens in entries).items()]
    return requests, refusals


def row_unit_key(context, requests, refusals):
    """一行的行键：各入口的键（入口自己、它的判法与内容输入都在里面）排好合起来；有一个拒算整行拒算。交 UnitKey。"""
    keys, composition, seen, all_refusals = [], [], set(), list(refusals)
    for request in requests:
        unit_key = compute_unit_key(context, request)
        for line in unit_key.composition:
            if line not in seen:
                seen.add(line)
                composition.append(line)
        if unit_key.key is None:
            all_refusals += unit_key.refusals
        else:
            keys.append(unit_key.key)
    if all_refusals or not requests:
        return UnitKey(None, composition, sorted(set(all_refusals)) or ["refused\treplay-row-without-entry\t-\t这一行一个入口都没有"])
    return UnitKey(hashlib.sha256("".join(key + "\n" for key in sorted(keys)).encode()).hexdigest(), composition, [])


def replay_table_rows(root, wanted_experiments=()):
    replay_path = os.path.join(root, REPLAY_SCRIPT)
    with open(replay_path, encoding="utf-8", errors="replace") as handle:
        text = handle.read()
    rows = []
    for line in text.split("\n"):
        match = REPLAY_ROW_FORM.match(line)
        if match and (not wanted_experiments or match.group(1) in wanted_experiments):
            rows.append(line)
    return rows


def gpu_switch_of(root):
    """多机配置的 ENABLE_GPU（找法同 crash_case_cargo_features）：「1」或「0」；读不了按 0（replay.sh 开跑前自己另判配置读不读得出）。"""
    try:
        return "1" if configuration_value(multi_host_configuration_file(root, os.environ), "ENABLE_GPU") == "1" else "0"
    except InputManifestError:
        return "0"


def replay_row_keys(root, wanted_experiments=(), gpu_switch=None, without_product=False):
    """复跑表每一行的行键：[(前三列, UnitKey)]，照表里的次序。"""
    context = KeyContext(root)
    gpu_switch = gpu_switch if gpu_switch is not None else gpu_switch_of(root)
    replay_rows, _problems = replay_rows_with_keys(root)
    key_of_identity = {row.identity: row.key for row in replay_rows}
    registered_rows = context.rows()
    out = []
    for row_text in replay_table_rows(root, wanted_experiments):
        identity = "|".join(REPLAY_ROW_FORM.match(row_text).group(1, 2, 3))
        admission_key = key_of_identity.get(identity, identity.split("|")[0])
        try:
            registered = resolve_entries(registered_rows, admission_key) if rows_of_key(registered_rows, admission_key) else []
        except RegistrationError:
            registered = []
        requests, refusals = replay_row_request(context, row_text, gpu_switch, registered, without_product)
        out.append((identity, row_unit_key(context, requests, refusals)))
    return out


# ── 实验的准入（行键去掉期望产物那一项）──────────────────────────────────────────

def experiment_key(root, key):
    """实验键：复跑表里准入键是 key 的那一行的行键，不带期望产物（产物头记着它）。那一行不在复跑表里的，按登记的入口算（同一套请求，不带行的参数）。
    交 (UnitKey, 说明)。"""
    rows, problems = replay_rows_with_keys(root)
    if problems:
        raise RegistrationError("；".join(problems))
    identities = [row.identity for row in rows if row.key == key]
    context = KeyContext(root)
    registered = resolve_entries(context.rows(), key)
    if len(identities) == 1:
        experiment = identities[0].split("|")[0]
        for identity, unit_key in replay_row_keys(root, (experiment,), without_product=True):
            if identity == identities[0]:
                return unit_key, f"复跑表「{identity}」那一行的行键（不带期望产物）"
    if not registered:
        raise RegistrationError(f"{key} 在复跑表里没有恰好一行（{len(identities)} 行），登记表那一行也没登记入口：算不出实验键")
    requests = []
    for entry in registered:
        if entry.startswith(REPLAY_RESEARCH_DIRECTORY + "/") and entry.split("::", 1)[0].endswith(".rs"):
            requests.append(Request(key, entry[len(REPLAY_RESEARCH_DIRECTORY) + 1:], [f"root:{REPLAY_RESEARCH_DIRECTORY}", "profile:release"]))
        else:
            requests.append(Request(key, entry, []))
    return row_unit_key(context, requests, []), f"登记的 {len(registered)} 个入口"



def judge_inputs_unchanged(root, key, fingerprint):
    """输入没变：返回 (没变?, 说明)。research/results/ 里有一份产物头上记着同一个键、同一把实验键就算没变。"""
    stored = stored_fingerprints(root, key)
    replay_products = replay_products_of(root, key)
    matching = [entry for entry in stored if entry.fingerprint == fingerprint]
    if matching:
        preferred = next((entry for entry in matching if entry.product_path in replay_products), matching[0])
        return True, f"{preferred.product_path} 头上的实验键与今天的相同（{fingerprint[:12]}）"
    if stored:
        newest = max(stored, key=lambda entry: entry.modified_at)
        return False, f"实验键自 {newest.product_path} 以来变了（那一份 {newest.fingerprint[:12]}，今天 {fingerprint[:12]}）"
    return False, f"上一份产物头上没有 {key} 的实验键，判不了（research/results/ 里没有一份头上带它的产物）"


def admit_experiment(root, key):
    """返回 (退出码, 产物头的行)。说明打到 stderr。"""
    rows = read_registration_rows(root)
    own_rows = rows_of_key(rows, key)
    if not EXPERIMENT_KEY_FORM.match(key) or not own_rows:
        say(f"  ✗ 准入登记里没有实验键 {key}（{REGISTRATION_TABLE}）")
        say(f"     → 怎么办：在 {REGISTRATION_TABLE} 加一行「{key}<制表符><装置入口，空格分隔>[<制表符><准入条件>]<制表符>#<为什么>」，"
            "照 E142 那一行写；装置入口传的键与登记的键逐字相同")
        return EXIT_REGISTRATION_ERROR, []
    conditions = [token for row in own_rows for token in row.conditions]
    try:
        problems = lint_registration_rows(own_rows, root, rows)
        if problems:
            raise RegistrationError("；".join(problems))
        for token in conditions:
            parse_condition(token)
        replay_products_of(root, key)   # 复跑表按行的准入键那一段写错，在这里按登记有错报
        unit_key, basis = experiment_key(root, key)
        missing, satisfied = ([], []) if break_is_set("skip-preconditions") else judge_preconditions(root, key, conditions)
    except RegistrationError as error:
        say(f"  ✗ {key} 的准入登记有错：{error}")
        say(f"     → 怎么办：改 {REGISTRATION_TABLE} 里 {key} 那一行（入口写错、条件写错），或 {REPLAY_SCRIPT} 的 ADMISSION_ROW_KEYS；"
            "改到 python3 scripts/gate/admission.py entries <根> <键> 列得出、这里不再报错为止")
        return EXIT_REGISTRATION_ERROR, []
    for description in satisfied:
        say(f"  ✓ 前提齐了：{description}")
    if missing:
        say(f"  ✗ 前提没齐，不许开跑 {key}：共 {len(missing)} 条")  # gate-lint:summary
        for description in missing:
            say(f"       {description}")  # gate-lint:detail
        say(f"     → 怎么办：这几条前提登记在 {REGISTRATION_TABLE} 的 {key} 那一行（跑前登记写的「只在……之后跑」）；"
            "先把它们办齐（问题单那一行判完写成够判、上一段产物的判决字段是 true），再跑。"
            "条件本身写错了就改登记表那一行，不许绕过；强制开关只越过「输入没变」，越不过这一条")
        return EXIT_PRECONDITION_MISSING, []
    unit_count = sum(1 for line in unit_key.composition if line.count("\t") == 3 and ":" in line.split("\t")[2])
    if unit_key.key is None:
        say(f"  ! {key} 的实验键拒算（{basis}）：{'；'.join(line.split(chr(9), 1)[1].replace(chr(9), ' ') for line in unit_key.refusals[:3])}；照放行，下一趟也判不了「没变」")
        return EXIT_ADMITTED, [f"{HEADER_PREFIX}name=input_fingerprint key={key} sha256=refused units={unit_count}"]
    if break_is_set("skip-unchanged"):
        unchanged, description = False, "（弄坏开关 skip-unchanged：没比实验键）"
    else:
        unchanged, description = judge_inputs_unchanged(root, key, unit_key.key)
    header = [f"{HEADER_PREFIX}name=input_fingerprint key={key} sha256={unit_key.key} units={unit_count}"]
    force_reason = os.environ.get(FORCE_RERUN_VARIABLE)
    if force_reason is not None:
        cleaned_reason = " ".join(force_reason.split())
        if not cleaned_reason:
            say(f"  ✗ {FORCE_RERUN_VARIABLE} 设了却是空的：强制重跑要写理由")
            say(f"     → 怎么办：写成 {FORCE_RERUN_VARIABLE}=<为什么输入没变还要重跑>，理由会原样写进产物头")
            return EXIT_REGISTRATION_ERROR, []
        header.append(f"{HEADER_PREFIX}name=forced_rerun key={key} overrode_unchanged={'true' if unchanged else 'false'} reason={cleaned_reason}")
        say(f"  ✓ 放行（{FORCE_RERUN_VARIABLE} 给了理由，理由写进产物头）：{description}")
        return EXIT_ADMITTED, header
    if unchanged:
        say(f"  ✗ 拒绝开跑 {key}：实验键（{basis}）自上次产物以来没变，不必重跑——{description}")
        say(f"     → 怎么办：沿用那一份的结论；确要重跑，设 {FORCE_RERUN_VARIABLE}=<理由> 再跑，理由会写进产物。"
            "实验键只看装置入口单元闭包与这一行的参数，不看 kb 文档与登记行原文")
        return EXIT_INPUTS_UNCHANGED, []
    say(f"  ✓ 放行 {key}：{description}")
    return EXIT_ADMITTED, header


def stored_product_row_key_status(root, key):
    """给门禁 doc-experiments 的 evidence-in-repo 格：这个实验键的产物跟不跟得上今天的实验键（行键去掉期望产物）。
    返回 (状态, 说明)：unregistered（没登记准入）、unfingerprinted（没有一份带实验键的产物，或今天的实验键拒算）、
    matched（有一份记的与今天的相同）、stale（带实验键的都与今天的不同）。登记写错抛 RegistrationError。"""
    rows = read_registration_rows(root)
    if not rows_of_key(rows, key):
        return "unregistered", ""
    stored = stored_fingerprints(root, key)
    if not stored:
        return "unfingerprinted", ""
    unit_key, _basis = experiment_key(root, key)
    if unit_key.key is None:
        return "unfingerprinted", "今天的实验键拒算"
    matching = [entry for entry in stored if entry.fingerprint == unit_key.key]
    if matching:
        return "matched", f"{matching[0].product_path}（实验键 {unit_key.key[:12]}）"
    newest = max(stored, key=lambda entry: entry.modified_at)
    return "stale", f"最新带实验键的产物 {newest.product_path} 记的是 {newest.fingerprint[:12]}，今天的是 {unit_key.key[:12]}"


# ── 崩溃枚举用例的块判定（CRASH_CASE_BLOCKS 那一行；文件头「崩溃枚举用例的复用不在这里」）──────────────

def crash_case_blocks_supported(root):
    """crates 侧接上了块查询没有：用例源码目录里认不认得 SINGLEFS_CRASH_CASE_BLOCKS_QUERY（只看有没有这个名字，不看怎么用）。"""
    directory = os.path.join(root, CRASH_CASE_BLOCKS_SOURCE_DIRECTORY)
    for path in glob.glob(os.path.join(directory, "**", "*.rs"), recursive=True):
        try:
            with open(path, encoding="utf-8", errors="replace") as handle:
                if CRASH_CASE_BLOCKS_QUERY_VARIABLE in handle.read():
                    return True
        except OSError:
            continue
    return False


def judge_crash_case_blocks(log_text, query, ran=False):
    """日志里的 CRASH_CASE_BLOCKS 行：交 (退出码, 一句)。退出码见文件头 crash-case-blocks。
    多流的用例每条流各打一行：逐行认（每一行 blocks = reused + fresh + refused），按用例合计再判。
    ran（--ran，快档实跑与 --full 跑登记了 red-blocks=judged-by-test 的用例）：只核行在、数对得上，红块与现判块不判（用例的退出码由门禁判）。
    弄坏开关 blocks-single-line-only 下照改前要恰好一行，blocks-count-without-refused 下照改前核 blocks = reused + fresh；
    ran-blocks-line-optional 下 --ran 一行都没有也退 0。"""
    lines = [line.strip() for line in log_text.split("\n") if line.strip().startswith(CRASH_CASE_BLOCKS_PREFIX)]
    if ran and not lines and break_is_set("ran-blocks-line-optional"):
        return 0, "块那一行没有（弄坏开关 ran-blocks-line-optional：没有也当对得上）"
    if not lines or (len(lines) != 1 and break_is_set("blocks-single-line-only")):
        return EXIT_BLOCKS_LINE_BAD, f"日志里以「{CRASH_CASE_BLOCKS_PREFIX.strip()} 」开头的行读到 {len(lines)} 行（每条流一行，至少一行）"
    counts = dict.fromkeys(CRASH_CASE_BLOCKS_FIELDS, 0)
    for line in lines:
        fields = fields_of_line(line)
        try:
            line_counts = {name: int(fields[name]) for name in CRASH_CASE_BLOCKS_FIELDS}
        except (KeyError, ValueError):
            return EXIT_BLOCKS_LINE_BAD, f"{line} 缺 {'、'.join(CRASH_CASE_BLOCKS_FIELDS)} 之一，或不是整数"
        accounted = line_counts["reused"] + line_counts["fresh"] + (0 if break_is_set("blocks-count-without-refused") else line_counts["refused"])
        if line_counts["blocks"] != accounted and not break_is_set("blocks-count-unchecked"):
            return EXIT_BLOCKS_LINE_BAD, f"{line}：blocks 不等于 reused + fresh + refused"
        for name in CRASH_CASE_BLOCKS_FIELDS:
            counts[name] += line_counts[name]
    summary = (f"块 {counts['blocks']}{f'（{len(lines)} 条流合计）' if len(lines) > 1 else ''}：复用 {counts['reused']}、"
               f"{'没有判定' if query else '现判'} {counts['fresh']}、红 {counts['red']}、拒算 {counts['refused']}")
    if ran:
        return 0, summary
    if counts["red"]:
        return EXIT_BLOCKS_RED, summary
    if query and (counts["fresh"] or counts["refused"]):
        return EXIT_BLOCKS_NOT_COVERED, summary
    return 0, summary


# ── 子命令 ────────────────────────────────────────────────────────────────────

def command_entries(arguments, files_only=False):
    if len(arguments) != 2:
        say(f"  ✗ 用法：admission.py {'paths' if files_only else 'entries'} <项目根> <键>")
        say("     → 怎么办：键照 .claude/gate.d/stage-inputs.tsv 的第一列写")
        return EXIT_REGISTRATION_ERROR
    try:
        entries = resolve_entries(read_registration_rows(arguments[0]), arguments[1])
    except RegistrationError as error:
        say(f"  ✗ {error}")
        say(f"     → 怎么办：在 {REGISTRATION_TABLE} 给这个键登记它的入口（制表符分隔，照别的行写）")
        return EXIT_REGISTRATION_ERROR
    shown = list(dict.fromkeys(path for path in (entry_file(entry) for entry in entries) if path)) if files_only else entries
    for item in shown:
        print(item)
    return 0


def command_test_functions(arguments):
    values = option_pairs(arguments[2:], ("--root", "--feature", "--profile")) if len(arguments) >= 2 else None
    if values is None:
        say("  ✗ 用法：admission.py test-functions <项目根> <包> [--root <目录>] [--feature <包>/<特性>]… [--profile <名>]")
        say("     → 怎么办：包写 Cargo.toml 的包名；research 工作区写 --root scripts/experiments")
        return EXIT_REGISTRATION_ERROR
    root = os.path.abspath(arguments[0])
    context = KeyContext(root)
    closure_root = os.path.normpath(os.path.join(root, values.get("--root", ["."])[-1]))
    try:
        functions = test_functions_of_package(context, arguments[1], closure_root, values.get("--feature", []), values.get("--profile", [None])[-1])
    except (RegistrationError, UnitKeyError) as error:
        print(f"  ✗ 列不出 {arguments[1]} 的测试函数：{error}")
        print("     → 怎么办：包名照 Cargo.toml 写；工作区读不出时先单跑 python3 scripts/gate/lib_function_units.py dump <根> 看它报什么")
        return EXIT_REGISTRATION_ERROR
    for function in functions:
        print(function.line())
    return 0


def option_pairs(options, names):
    """--名字 值 成对的参数，同一个名字可以给几次：返回 {名字: [值…]}；有不认得的、缺值的返回 None。单个的旗标（--full）另由调用方先摘掉。"""
    values = {}
    index = 0
    while index < len(options):
        if options[index] not in names or index + 1 >= len(options):
            return None
        values.setdefault(options[index], []).append(options[index + 1])
        index += 2
    return values


def command_unit_key(arguments):
    if len(arguments) < 2:
        say("  ✗ 用法：admission.py unit-key <项目根> <入口> [<项>…]")
        say("     → 怎么办：入口照 lib_function_units.py 的写法；项见文件头「判定请求」")
        return EXIT_REGISTRATION_ERROR
    try:
        request = Request(arguments[1], arguments[1], arguments[2:])
    except RegistrationError as error:
        say(f"  ✗ {error}")
        say("     → 怎么办：项照文件头「判定请求」写")
        return EXIT_REGISTRATION_ERROR
    unit_key = compute_unit_key(KeyContext(os.path.abspath(arguments[0])), request)
    if unit_key.key is None:
        for line in unit_key.refusals:
            print(line)
        return EXIT_UNIT_KEY_REFUSED
    print(f"key\t{unit_key.key}")
    for line in unit_key.composition:
        print(line)
    return 0


def read_requests(path):
    with open(path, encoding="utf-8", errors="surrogateescape") as handle:
        return [parse_request_line(line) for line in handle if line.strip() and not line.startswith("#")]


def print_plan(root, planned, library, plan_file, extra_columns=None, extra_counts=None):
    """verdict-plan 与 test-plan 共用的收尾：逐条打、写计划文件、打末行 PLAN。"""
    counts = collections.Counter(decision for _request, _key, decision, _reason in planned)
    plan_lines = []
    for request, unit_key, decision, reason in planned:
        middle = extra_columns(request) if extra_columns else []
        print("\t".join([request.name, decision, unit_key.key or "-", *middle, reason]))
        plan_lines.append(plan_line(root, request, unit_key, decision))
    problem = write_atomically(plan_file, "".join(line + "\n" for line in plan_lines)) if plan_file != "-" else ""
    if problem:
        print(f"  ✗ 计划文件没写成：{problem}")
        print("     → 怎么办：看那个目录可不可写；计划文件写不成，这一趟判绿的也记不进库")
        return EXIT_REGISTRATION_ERROR
    # 复用哨兵（文件头「复用哨兵」）：抽中了谁、种子打在逐条那几行之后、PLAN 之前（调用方取头一行或按列取的都不碰到它）；
    # 状态存在计划文件旁边，verdict-record 拿它比强制现判那一条
    sentinel = getattr(library, "sentinel", None)
    for line in getattr(library, "sentinel_lines", []):
        print(line)
    if sentinel is not None and plan_file != "-":
        observed = "".join(f"{item}\t{state}\t{verdict}\n" for item, (state, verdict) in sentinel.observed.items())
        problem = (write_atomically(plan_file + SENTINEL_STATE_SUFFIX, json.dumps(sentinel.to_json(), ensure_ascii=False))
                   or write_atomically(plan_file + SENTINEL_STATE_SUFFIX + ".observed", observed))
        if problem:
            print(f"  ! 复用哨兵的状态没存下（{problem}）：verdict-record 比不了强制现判那一条")
    elif plan_file != "-":
        for stale in (plan_file + SENTINEL_STATE_SUFFIX, plan_file + SENTINEL_STATE_SUFFIX + ".observed"):
            if os.path.exists(stale):
                os.unlink(stale)
    extra = "".join(f" {name}={value}" for name, value in (extra_counts or {}).items())
    print(f"PLAN judge={counts['judge']} reuse={counts['reuse']} refused={counts['refused']}{extra} library={library.directory or '没有'}")
    sentinel_problems = getattr(library, "sentinel_problems", [])
    for message, howto in sentinel_problems:
        print(f"  ✗ {message}")
        print(f"     → 怎么办：{howto}")
    return 1 if sentinel_problems else 0


def command_verdict_plan(arguments):
    full = "--full" in arguments
    arguments = [argument for argument in arguments if argument != "--full"]
    values = option_pairs(arguments[4:], ("--library-root",)) if len(arguments) >= 4 else None
    if values is None:
        print("  ✗ 用法：admission.py verdict-plan <项目根> <种类> <请求文件> <计划文件> [--library-root <根>] [--full]")
        print("     → 怎么办：请求文件一行一条「<名字><制表符><入口>[<制表符><项>…]」（文件头「判定请求」）；种类写门禁的登记键")
        return EXIT_REGISTRATION_ERROR
    root, kind, request_file, plan_file = os.path.abspath(arguments[0]), arguments[1], arguments[2], arguments[3]
    try:
        requests = read_requests(request_file)
    except (OSError, RegistrationError) as error:
        print(f"  ✗ 读不了请求文件 {request_file}：{error}")
        print("     → 怎么办：照文件头「判定请求」写请求文件")
        return EXIT_REGISTRATION_ERROR
    sample_failure = plan_failure_sample(root)
    if sample_failure != "sentinel-reuse-failed" and sample_failure is not None:
        return sample_plan_failure_exit("verdict-plan", root, sample_failure)
    library_root = os.path.abspath(values.get("--library-root", [root])[-1])
    if break_is_set("verdict-key-with-entry-file"):
        # 弄坏开关：每条请求的键另加它入口所在的整份文件（闭包外的改动——同文件里没人调的函数、注释——也让它作废）：自证 reuse-three-questions 的 ② 红
        requests = [Request(request.name, request.entry,
                            [*request.tokens, f"file:{os.path.normpath(os.path.join(request.root, request.entry.split('::', 1)[0]))}"])
                    for request in requests]
    planned, library = plan_requests(root, kind, requests, library_root, full_requested(full))
    if sample_failure == "sentinel-reuse-failed":
        add_sample_sentinel_failure(root, planned, library)
    return print_plan(root, planned, library, plan_file)


def command_test_plan(arguments):
    """test-plan：test-functions 与 verdict-plan 在一个进程里做（闭包只收一次）。每条测试函数一行
    「<名字><制表符><reuse|judge|refused|not-compiled><制表符><键或 -><制表符><目标号><制表符><目标根文件><制表符><libtest 全名><制表符><原因>」。"""
    full = "--full" in arguments
    arguments = [argument for argument in arguments if argument != "--full"]
    values = option_pairs(arguments[4:], ("--root", "--feature", "--profile", "--token", "--library-root")) if len(arguments) >= 4 else None
    if values is None:
        print("  ✗ 用法：admission.py test-plan <项目根> <种类> <包> <计划文件> [--root <目录>] [--feature <包>/<特性>]… [--profile <名>] [--token <项>]… "
              "[--library-root <根>] [--full]")
        print("     → 怎么办：包写 Cargo.toml 的包名；--token 是每条请求另加的项（文件头「判定请求」：judge:、line: 这一类）")
        return EXIT_REGISTRATION_ERROR
    root, kind, package, plan_file = os.path.abspath(arguments[0]), arguments[1], arguments[2], arguments[3]
    closure_relative = values.get("--root", ["."])[-1]
    closure_root = os.path.normpath(os.path.join(root, closure_relative))
    features, profile = values.get("--feature", []), values.get("--profile", [None])[-1]
    sample_failure = plan_failure_sample(root)
    if sample_failure != "sentinel-reuse-failed" and sample_failure is not None:
        return sample_plan_failure_exit("test-plan", root, sample_failure)
    context = KeyContext(root)
    try:
        functions = test_functions_of_package(context, package, closure_root, features, profile)
    except (RegistrationError, UnitKeyError) as error:
        print(f"  ✗ 列不出 {package} 的测试函数：{error}")
        print("     → 怎么办：包名照 Cargo.toml 写；工作区读不出时先单跑 python3 scripts/gate/lib_function_units.py dump <根> 看它报什么")
        return EXIT_REGISTRATION_ERROR
    by_name = {function.name: function for function in functions}
    tokens_common = ([f"root:{closure_relative}"] if closure_relative != "." else []) + ([f"profile:{profile}"] if profile else [])
    tokens_common += [f"feature:{feature}" for feature in features] + values.get("--token", [])
    def whole_file(function):
        # 弄坏开关 test-key-with-whole-file：每条测试函数的键另加它所在的整份文件（改前那种按文件收的样子：改同文件里别的测试也作废）
        if not break_is_set("test-key-with-whole-file"):
            return []
        return [f"file:{os.path.normpath(os.path.join(closure_relative, function.entry.split('::', 1)[0]))}"]
    requests = [Request(function.name, function.entry, [f"target:{function.target_id}", *tokens_common, *whole_file(function)])
                for function in functions if function.state == "compiled"]
    library_root = os.path.abspath(values.get("--library-root", [root])[-1])
    planned, library = plan_requests(root, kind, requests, library_root, full_requested(full), context)
    if sample_failure == "sentinel-reuse-failed":
        add_sample_sentinel_failure(root, planned, library)
    not_compiled = [function for function in functions if function.state == "not-compiled"]
    for function in not_compiled:
        print("\t".join([function.name, "not-compiled", "-", function.target_id, function.target_root, function.libtest_name, "这一组特性下 cfg 关着，不编"]))
    return print_plan(root, planned, library, plan_file,
                      lambda request: [by_name[request.name].target_id, by_name[request.name].target_root, by_name[request.name].libtest_name],
                      {"not-compiled": len(not_compiled), "ignored": sum(1 for function in functions if function.state == "ignored")})


def command_verdict_record(arguments):
    values = option_pairs(arguments[4:], ("--how", "--library-root", "--summary")) if len(arguments) >= 4 else None
    if values is None or "--how" not in values:
        print("  ✗ 用法：admission.py verdict-record <项目根> <种类> <计划文件> <判绿的名字文件> --how <跑法> [--library-root <根>] [--summary <一句>]")
        print("     → 怎么办：计划文件取 verdict-plan / test-plan 那一趟写的；名字文件一行一个判绿的名字；--how 写这一趟是怎么跑的（命令的样子，不写路径与时刻）")
        return EXIT_REGISTRATION_ERROR
    root, kind, plan_file, names_file = os.path.abspath(arguments[0]), arguments[1], arguments[2], arguments[3]
    library_root = os.path.abspath(values.get("--library-root", [root])[-1])
    try:
        with open(plan_file, encoding="utf-8") as handle:
            planned = [json.loads(line) for line in handle if line.strip()]
        with open(names_file, encoding="utf-8", errors="surrogateescape") as handle:
            green = {line.rstrip("\n") for line in handle if line.strip()}
    except (OSError, ValueError) as error:
        print(f"  ✗ 读不了计划文件或名字文件：{error}")
        print("     → 怎么办：计划文件取 verdict-plan 那一趟写的；名字文件一行一个名字")
        return EXIT_REGISTRATION_ERROR
    context = KeyContext(root)
    library = VerdictLibrary(library_root, kind)
    recorded = changed = refused = 0
    for item in planned:
        request = parse_request_line(item["request"])
        if request.name not in green:
            continue
        if not item.get("key"):
            refused += 1
            continue
        files = item.get("files") or {}
        unchanged = bool(files) and all((sha256_of_file(os.path.join(root, relative)) if os.path.isfile(os.path.join(root, relative)) else "不在") == digest
                                        for relative, digest in files.items())
        if not unchanged:
            unit_key = compute_unit_key(context, request)
            if unit_key.key != item["key"]:
                changed += 1
                print(f"  ! {request.name}：跑的过程中键变了（开跑 {item['key'][:12]}，跑完 {(unit_key.key or 'refused')[:12]}），不记")
                continue
        problem = library.record(item["request"], request.name, item["key"], values["--how"][-1], values.get("--summary", [""])[-1])
        if problem:
            print(f"  ! {request.name} 没记进判定库：{problem}（下一趟这一条照判）")
            continue
        recorded += 1
    sentinel_red = report_forced_rejudge(plan_file, green)
    print(f"RECORD recorded={recorded} changed={changed} refused={refused} library={library.directory or '没有'}")
    return 1 if sentinel_red else 0


def report_forced_rejudge(plan_file, green):
    """复用哨兵收尾（文件头「复用哨兵」）：verdict-plan 存在计划文件旁边的状态在，就比强制现判那一条——判绿了与库里同为绿，记下；
    没判绿的（红了或没跑到）不在这里判，这一条的红由门禁这一趟自己报。打一行判定（在 RECORD 那一行之前），交红没红。"""
    checks = reuse_checks_module()
    state_path = plan_file + SENTINEL_STATE_SUFFIX
    if checks is None or not os.path.isfile(state_path):
        return False
    try:
        sentinel = checks.load_sentinel(state_path)
    except (OSError, ValueError, KeyError) as error:
        print(f"  ! 复用哨兵的状态读不了（{error}）：这一趟不比强制现判那一条")
        return False
    sentinel.observed = checks.observed_lines(state_path)
    if sentinel.forced_item is not None:
        if sentinel.forced_item in green:
            sentinel.observe(sentinel.forced_item, "强制现判", "绿")
        else:
            print(f"  · {sentinel.title()}：强制现判的 {sentinel.forced_item} 这一趟没判绿（红了或没跑到）：不在这里比，它的红由门禁这一趟自己报")
    problems = sentinel.finish(printer=print)
    for message, howto in problems:
        print(f"  ✗ {message}")
        print(f"     → 怎么办：{howto}")
    return bool(problems)


# ── 按条复用失灵（文件头「按条复用失灵」）：四处调用方共用这一份判法 ──
PLAN_FAILURE_SAMPLE_MARKER = os.path.join(UNIT_VERDICT_SAMPLE_DIRECTORY, "plan-failure")
PLAN_FAILURE_SAMPLE_FORMS = ("exit-1", "sentinel-reuse-failed")
PLAN_COMMANDS = ("verdict-plan", "test-plan")
# 复用哨兵的三种红句（scripts/gate/lib_reuse_checks.py 的 ReuseSentinel.finish；verdict-plan / verdict-record 打成「  ✗ <红句>」一行）
SENTINEL_FAILURE_FORMS = (
    (re.compile(r"^\s*✗ (复用失灵：.*)$"), "复用失灵",
     "这一项此刻的键与库里那一条判绿的记录相同、这一趟却现判了：看 scripts/gate/admission.py 文件头「复用哨兵」一段，查 plan_requests 与 VerdictLibrary.lookup 那一段为什么没读到它"),
    (re.compile(r"^\s*✗ (判定不稳：.*)$"), "判定不稳",
     "同一把键两次判出不同的结论：看 scripts/gate/admission.py 文件头「单元键」一段，判法依赖了键之外的输入就把它加进键或去掉依赖，再删掉库里那一条重判"),
    (re.compile(r"^\s*✗ (复用哨兵.*强制现判没起作用.*)$"), "强制现判没起作用",
     "抽中强制现判的那一条这一趟还是复用了：看 scripts/gate/admission.py 文件头「复用哨兵」一段与 plan_requests 里先问 forced 的那一句"),
)


def plan_failure_sample(root):
    """门禁样本档造的失灵（文件头「按条复用失灵」）：<根>/.unit-verdict-sample/plan-failure 不在交 None；在就交它写的那一样（写的不是认得的两样也照交，调用方退 2）。"""
    try:
        with open(os.path.join(root, PLAN_FAILURE_SAMPLE_MARKER), encoding="utf-8") as handle:
            return handle.read().strip()
    except OSError:
        return None


def sample_plan_failure_exit(command, root, form):
    """样本档的 exit-1：不算键、不查库、退 1；标记写坏了退 2。"""
    marker = os.path.join(root, PLAN_FAILURE_SAMPLE_MARKER)
    if form not in PLAN_FAILURE_SAMPLE_FORMS:
        print(f"  ✗ 样本档标记 {marker} 写的是「{form}」，认不出")
        print(f"     → 怎么办：只写一行 {' 或 '.join(PLAN_FAILURE_SAMPLE_FORMS)}（文件头「按条复用失灵」）；真仓里不该有这份文件")
        return EXIT_REGISTRATION_ERROR
    print(f"  ✗ 样本档造的失灵：{marker} 写着 exit-1，{command} 不算键、不查库、退 1")
    print("     → 怎么办：这是门禁样本档的假失灵（文件头「按条复用失灵」），调用方要把这一趟判红；真仓里不该有这份文件，有就删掉")
    return 1


def add_sample_sentinel_failure(root, planned, library):
    """样本档的 sentinel-reuse-failed：照常算完，再添一条复用哨兵的「复用失灵」红句（print_plan 照它退 1）。"""
    item = planned[0][0].name if planned else "（这一趟没有请求）"
    library.sentinel_problems = list(getattr(library, "sentinel_problems", [])) + [(
        f"复用失灵：{item} 键相同却现判了（样本档造的假哨兵：{os.path.join(root, PLAN_FAILURE_SAMPLE_MARKER)} 写着 sentinel-reuse-failed）",
        "这是门禁样本档的假失灵（文件头「按条复用失灵」），调用方要把这一趟判红；真仓里不该有这份文件，有就删掉")]


def plan_failure_findings(command, exit_code, text):
    """那一趟按条复用失灵在哪：[(红句, 出路)]；没失灵交 []。"""
    findings = []
    for line in text.split("\n"):
        for pattern, label, howto in SENTINEL_FAILURE_FORMS:
            match = pattern.match(line)
            if match is not None:
                findings.append((f"复用哨兵报红（{label}）：{match.group(1).strip()}", howto))
                break
    if findings or command not in PLAN_COMMANDS:
        return findings
    failure_lines = [line.strip() for line in text.split("\n") if line.strip().startswith("✗ ")]
    shown = failure_lines[-1] if failure_lines else " ".join(line.strip() for line in text.strip().split("\n")[-2:]) or "没有输出"
    if exit_code != 0:
        findings.append((f"scripts/gate/admission.py {command} 退 {exit_code}（{shown[:300]}）",
                         f"看 scripts/gate/admission.py 文件头子命令「{command}」那一条：退 2 是登记或用法错（请求文件照文件头「判定请求」写），"
                         "退 1 是复用哨兵或样本档造的失灵（文件头「复用哨兵」「按条复用失灵」），别的退出码是它自己崩了，单跑同一条命令看它报什么"))
    elif not re.search(r"^PLAN ", text, re.M):
        findings.append((f"scripts/gate/admission.py {command} 退 0 却没打 PLAN 那一行",
                         f"看 scripts/gate/admission.py 文件头子命令「{command}」那一条（末行 PLAN judge=… reuse=… refused=…）与 print_plan，单跑同一条命令看它打了什么"))
    return findings


def command_plan_failure(arguments):
    """plan-failure <调用方> <子命令> <退出码> [<输出文件>…]（文件头「按条复用失灵」）。"""
    if len(arguments) < 3 or arguments[1] not in (*PLAN_COMMANDS, "verdict-record") or not re.fullmatch(r"[0-9]+", arguments[2]):
        print("  ✗ 用法：admission.py plan-failure <调用方> <verdict-plan|test-plan|verdict-record> <退出码> [<输出文件>…]")
        print("     → 怎么办：输出文件给那一趟 admission.py 的标准输出与标准错误落的文件（文件头「按条复用失灵」）")
        return EXIT_REGISTRATION_ERROR
    caller, command, exit_code = arguments[0], arguments[1], int(arguments[2])
    texts = []
    for path in arguments[3:]:
        try:
            with open(path, encoding="utf-8", errors="replace") as handle:
                texts.append(handle.read())
        except OSError as error:
            texts.append(f"✗ 读不了那一趟的输出 {path}：{error}")
    findings = plan_failure_findings(command, exit_code, "\n".join(texts))
    if not findings:
        return 0
    if break_is_set("plan-failure-not-red"):
        # 弄坏开关：认出失灵也不判红（改前四处调用方退回全部现判、不判红的样子）
        for message, _howto in findings:
            print(f"  ! {caller}按条复用失灵（弄坏开关 plan-failure-not-red：退回全部现判、不判红）：{message}")
        return 0
    for message, howto in findings:
        print(f"  ✗ {caller}按条复用失灵：{message}；这一趟照旧全部现判跑完，结论判红（不能复用就判红）")
        print(f"     → 怎么办：{howto}")
    return 1


def command_gate_reuse(arguments):
    if len(arguments) not in (2, 3) or not arguments[0] or not arguments[1]:
        print("  ✗ 用法：admission.py gate-reuse <项目根> <登记键> [<请求文件>]")
        print("     → 怎么办：登记键照 .claude/gate.d/stage-inputs.tsv 的第一列写，例 checker-tier-crates-mutation-replay.sh。")
        return EXIT_GATE_MUST_RUN
    try:
        exit_code, reason, to_judge = gate_reuse(os.path.abspath(arguments[0]), arguments[1], arguments[2] if len(arguments) == 3 else None)
    except Exception as error:  # 模块自己出错一律按要跑处理：出错就跳过等于把这道门禁关掉
        print(f"判不出来：准入模块出错（{type(error).__name__}：{error}），按要跑处理")
        return EXIT_GATE_MUST_RUN
    print(reason)
    for name in to_judge:
        print(f"  judge\t{name}")
    return exit_code


def command_experiment(arguments):
    if len(arguments) != 2:
        say("  ✗ 用法：admission.py experiment <项目根> <实验键>")
        say("     → 怎么办：实验键照 .claude/gate.d/stage-inputs.tsv 的第一列写，例 E142、E142/p8g-crates-dump")
        return EXIT_REGISTRATION_ERROR
    exit_code, header = admit_experiment(os.path.abspath(arguments[0]), arguments[1])
    for header_line in header:
        print(header_line)
    return exit_code


def command_replay_row_inputs(arguments):
    """replay.sh 按行复用调：复跑表每一行打「<前三列><制表符><行键或 refused><制表符><单元数><制表符><说明>」；算不出整份退 2，✗ 与出路打在 stdout。"""
    if not arguments:
        print("  ✗ 用法：admission.py replay-row-inputs <项目根> [--gpu 0|1] [--library <判定库目录>] [--compositions <目录>] [实验号…]")
        print("     → 怎么办：第一个参数给项目根，后面可以跟几个实验号（只算那几个实验的行）")
        return EXIT_REGISTRATION_ERROR
    root, rest = os.path.abspath(arguments[0]), list(arguments[1:])
    gpu_switch, library_directory, compositions = None, None, None
    wanted = []
    while rest:
        option = rest.pop(0)
        if option in ("--gpu", "--library", "--compositions") and rest:
            value = rest.pop(0)
            if option == "--gpu":
                gpu_switch = value
            elif option == "--library":
                library_directory = value
            else:
                compositions = value
        else:
            wanted.append(option)
    try:
        keys = replay_row_keys(root, tuple(wanted), gpu_switch)
    except (OSError, RegistrationError) as error:
        print(f"  ✗ 复跑表每一行的行键算不出：{error}")
        print("     → 怎么办：单跑 python3 scripts/gate/admission.py replay-row-inputs . 看它报什么；工具链起不来的先修好 rustc -vV 与 cargo -V")
        return EXIT_REGISTRATION_ERROR
    for identity, unit_key in keys:
        unit_count = sum(1 for line in unit_key.composition if line.count("\t") == 3 and ":" in line.split("\t")[2])
        if unit_key.key is None:
            note = "拒算：" + "；".join(line.split("\t", 1)[1].replace("\t", " ") for line in unit_key.refusals[:3])
            print(f"{identity}\trefused\t{unit_count}\t{note}")
            continue
        note = ""
        if library_directory:
            note = replay_row_change_note(library_directory, identity, unit_key)
        if compositions:
            problem = write_atomically(os.path.join(compositions, unit_key.key + ".tsv"), "".join(line + "\n" for line in unit_key.composition))
            note = note or (f"组成清单没写成：{problem}" if problem else "")
        print(f"{identity}\t{unit_key.key}\t{unit_count}\t{note}")
    return 0


def replay_row_change_note(library_directory, identity, unit_key):
    """这一行上一次判过的那一条（<库>/items/<前三列的 sha256> 记着键）与今天的组成清单比：键相同交空串，不同交变了的单元。"""
    index = os.path.join(library_directory, "items", hashlib.sha256(identity.encode("utf-8", "surrogateescape")).hexdigest())
    try:
        with open(index, encoding="utf-8") as handle:
            earlier_key = handle.read().strip()
    except OSError:
        return "库里没有这一行判过的记录"
    if earlier_key == unit_key.key:
        return ""
    try:
        with open(os.path.join(library_directory, "keys", earlier_key, "composition"), encoding="utf-8") as handle:
            earlier = [line.rstrip("\n") for line in handle if line.strip()]
    except OSError:
        return f"键对不上（上一次 {earlier_key[:12]}，那一份没留组成清单）"
    notes, total = changed_composition(earlier, unit_key.composition)
    return f"键对不上（上一次 {earlier_key[:12]}），变了 {total} 处：{'；'.join(notes)}"


def command_crash_case_blocks(arguments):
    query = "--query" in arguments
    ran = "--ran" in arguments
    arguments = [argument for argument in arguments if argument not in ("--query", "--ran")]
    if len(arguments) != 2 or (query and ran):
        print("  ✗ 用法：admission.py crash-case-blocks <项目根> <日志> [--query | --ran]")
        print("     → 怎么办：日志取那一趟用例的全量输出；块查询那一趟带 --query，快档实跑与 --full 跑登记了 red-blocks=judged-by-test 的那一趟带 --ran，两样不同给")
        return EXIT_REGISTRATION_ERROR
    root, log_file = arguments
    if not crash_case_blocks_supported(root):
        print(f"crates 侧还没接上块查询：{CRASH_CASE_BLOCKS_SOURCE_DIRECTORY} 里找不到 {CRASH_CASE_BLOCKS_QUERY_VARIABLE}（接口在 admission.py 文件头「崩溃枚举用例的复用不在这里」）")
        return EXIT_BLOCKS_UNSUPPORTED
    try:
        with open(log_file, encoding="utf-8", errors="surrogateescape") as handle:
            log_text = handle.read()
    except OSError as error:
        print(f"读不了日志 {log_file}：{error}")
        return EXIT_BLOCKS_LINE_BAD
    exit_code, summary = judge_crash_case_blocks(log_text, query, ran)
    print(summary)
    return exit_code


def command_gate_preconditions(arguments):
    if len(arguments) != 2 or not arguments[0] or not arguments[1]:
        print("  ✗ 用法：admission.py gate-preconditions <阶段所在的仓根> <阶段文件名>")
        print("     → 怎么办：阶段里写成 python3 <仓根>/scripts/gate/admission.py gate-preconditions <仓根> \"$(basename \"$0\")\"")
        return EXIT_REGISTRATION_ERROR
    root, stage = arguments
    try:
        stage_rows = rows_of_key(read_registration_rows(root), stage)
        if not stage_rows:
            raise RegistrationError(f"{REGISTRATION_TABLE} 里没有 {stage} 这一行，判不出这一道开跑前要什么")
        missing = [] if break_is_set("skip-gate-preconditions") else judge_gate_preconditions(root, stage_rows)
    except RegistrationError as error:
        print(f"  ✗ {stage} 的准入登记有错：{error}")
        print(f"     → 怎么办：在 {REGISTRATION_TABLE} 给 {stage} 登记一行，或改那一行（门禁行第三列只认 command=、readwrite=、probe=、environment=，"
              "写法见 scripts/gate/admission.py 文件头）；python3 scripts/gate/admission.py --selftest 的最后一格点得出登记表哪一行写错")
        return EXIT_REGISTRATION_ERROR
    except Exception as error:  # 判不出前提就不开跑：出错就放行等于把前提这一道关掉
        print(f"  ✗ 判不出 {stage} 的前提：准入模块出错（{type(error).__name__}：{error}）")
        print("     → 怎么办：单跑 python3 scripts/gate/admission.py gate-preconditions <仓根> <阶段文件名> 看报错，修准入模块或登记表那一行")
        return EXIT_GATE_PRECONDITION_MISSING
    if missing:
        why = "；".join(row.comment for row in stage_rows if row.comment) or "那一行没写注释"
        print(f"  ✗ 前提没齐，{stage} 不开跑：共 {len(missing)} 条（登记在 {REGISTRATION_TABLE} 的 {stage} 那一行第三列）")  # gate-lint:summary
        for description in missing:
            print(f"       {description}")  # gate-lint:detail
        print(f"     → 怎么办：把这几条办齐再跑这一道（这一道为什么要它们：{why}）；"
              "前提写错了就改登记表那一行，不许绕过。这一道判红、不记本次未跑：前提没齐时它一格都没判")
        return EXIT_GATE_PRECONDITION_MISSING
    return 0

def command_keys(arguments):
    if len(arguments) != 1:
        say("  ✗ 用法：admission.py keys <项目根>")
        say("     → 怎么办：只给项目根一个参数")
        return EXIT_REGISTRATION_ERROR
    seen = []
    for row in read_registration_rows(arguments[0]):
        if EXPERIMENT_KEY_FORM.match(row.key) and row.key not in seen:
            seen.append(row.key)
    for key in seen:
        print(key)
    return 0


def replay_row_key_problems(root):
    """复跑表每一行的准入键，连同 replay-row-keys 判红的三类：返回 ([ReplayRow], 登记了准入的键集合, 写法与登记的说明, 被几行共用的 [(键, 行数)])。"""
    rows, problems = replay_rows_with_keys(root)
    registered = {row.key for row in read_registration_rows(root) if EXPERIMENT_KEY_FORM.match(row.key)}
    for row in rows:
        if row.explicit and row.key not in registered:
            problems.append(f"复跑表「{row.identity}」那一行指明的准入键 {row.key} 没在 {REGISTRATION_TABLE} 登记")
    counts = {}
    for row in rows:
        if row.key in registered:
            counts[row.key] = counts.get(row.key, 0) + 1
    shared = [(key, count) for key, count in counts.items() if count > 1]
    return rows, registered, problems, shared


def command_replay_row_keys(arguments):
    """replay.sh 开跑前调：复跑表每一行打「<前三列><制表符><准入键><制表符><产物头谁写><制表符><登记了准入：yes|no>」；判错退 2，✗ 与出路打在 stdout。"""
    if len(arguments) != 1:
        print("  ✗ 用法：admission.py replay-row-keys <项目根>")
        print("     → 怎么办：只给项目根一个参数")
        return EXIT_REGISTRATION_ERROR
    rows, registered, problems, shared = replay_row_key_problems(arguments[0])
    if problems:
        print(f"  ✗ 复跑表按行的准入键有 {len(problems)} 处不对：")  # gate-lint:summary
        for problem in problems:
            print(f"       {problem}")  # gate-lint:detail
        print(f"     → 怎么办：照上面每一条改 {REPLAY_SCRIPT} 里 ADMISSION_ROW_KEYS 那一段（键、复跑表那一行的前三列原样抄、产物头谁写 device 或 replay），"
              f"指明的键在 {REGISTRATION_TABLE} 各登记一行它那一行读的输入；写法见 {os.path.join('scripts', 'gate', 'admission.py')} 文件头「复跑表按行的准入键」")
    for key, count in shared:
        print(f"  ✗ {key} 登记了准入，而这张复跑表里它有 {count} 行：准入只按实验号比产物，分不清哪一行对哪一份")
        print("     → 怎么办：给每一行起一个带方式的准入键（照 .claude/gate.d/stage-inputs.tsv 的 E142/p8g-crates-dump 写），在准入模块里按行认键之后再登记："
              f"写进 {REPLAY_SCRIPT} 的 ADMISSION_ROW_KEYS（一行一条：键、那一行的前三列、产物头谁写），每个键在 {REGISTRATION_TABLE} 各登记一行")
    if problems or shared:
        return EXIT_REGISTRATION_ERROR
    for row in rows:
        print(f"{row.identity}\t{row.key}\t{row.header_writer}\t{'yes' if row.key in registered else 'no'}")
    return 0


def command_crash_cases(arguments):
    """核登记的崩溃枚举用例，逐条打「键、包、测试目标、用例函数」；登记有错退 2，原因不带 ✗（阶段自己打 ✗ 与出路）。"""
    if len(arguments) != 1:
        print("  ✗ 用法：admission.py crash-cases <项目根>")
        print("     → 怎么办：只给项目根一个参数")
        return EXIT_REGISTRATION_ERROR
    root = arguments[0]
    rows = read_registration_rows(root)
    crash_case_rows = [row for row in rows if CRASH_CASE_KEY_FORM.match(row.key)]
    problems = [problem for problem in lint_registration_rows(crash_case_rows, root)]
    if problems:
        for problem in problems:
            print(f"{REGISTRATION_TABLE} {problem}")
        return EXIT_REGISTRATION_ERROR
    for case in crash_cases_of(crash_case_rows):
        print(f"{case.key}\t{case.package}\t{case.target}\t{case.function}")
    return 0

def option_values(options, names):
    """--名字 值 成对的参数：返回 {名字: 值}；有不认得的、缺值的返回 None。"""
    values = {}
    index = 0
    while index < len(options):
        if options[index] not in names or index + 1 >= len(options):
            return None
        values[options[index]] = options[index + 1]
        index += 2
    return values


def forwarded_worker_threads(arguments):
    """弄坏开关 judge-takes-forwarded-threads 下 crash-case-judge 照旧收的 --machine-cores / --threads / --threads-origin
    （checker-tier-crash-point-replay 转过来的，改前的写法）：交 (核数, 线程数, 来源) 或 None。开关没开时交 None、调用方不认这三个参数。"""
    if not break_is_set("judge-takes-forwarded-threads"):
        return None
    values = option_values(arguments, ("--machine-cores", "--threads", "--threads-origin"))
    if not values or set(values) != {"--machine-cores", "--threads", "--threads-origin"}:
        return None
    return int(values["--machine-cores"]), int(values["--threads"]), values["--threads-origin"]


def command_crash_case_judge(arguments):
    """判 --full 跑那一条的日志：核数、线程数与它是不是显式设的由这里现取（crash_case_worker_threads，与 crash-case-command 交给用例的同一个算法、
    同一份环境），不收调用方转过来的。"""
    forwarded = forwarded_worker_threads(arguments[4:]) if len(arguments) > 4 else None
    if len(arguments) != 4 and forwarded is None:
        print("  ✗ 用法：admission.py crash-case-judge <项目根> <键> <日志> <记录行文件>")
        print("     → 怎么办：核数与线程数由它自己现取（与 crash-case-command 同一份环境），不再带 --machine-cores / --threads / --threads-origin")
        return EXIT_REGISTRATION_ERROR
    root, key, log_file, recorded_file = arguments[:4]
    try:
        case = crash_case_of_key(read_registration_rows(root), key)
        with open(log_file, encoding="utf-8", errors="surrogateescape") as handle:
            log_text = handle.read()
        machine_cores, threads, threads_origin = forwarded or crash_case_worker_threads(case, os.environ)
        default_threads = registered_heavy_threads(os.environ)
    except (RegistrationError, InputManifestError, OSError) as error:
        print(f"判不了 {key} 的日志：{error}")
        return EXIT_REGISTRATION_ERROR
    threads_explicitly_one = threads_origin == "explicit" and threads == 1
    problems, recorded_lines, thread_notes = judge_crash_case_log(case, log_text, machine_cores, threads_explicitly_one, default_threads)
    if problems:
        for problem in problems:
            print(problem)
        return 1
    recorded_lines.append("configured_worker_threads=" + configured_worker_threads_text(case, machine_cores, threads, threads_origin))
    with open(recorded_file, "w", encoding="utf-8", errors="surrogateescape") as handle:
        handle.write("".join(line + "\n" for line in recorded_lines))
    for note in thread_notes:
        print(note)
    return 0


def command_crash_case_command(arguments):
    """跑一条崩溃枚举用例的命令与环境（crash_case_launch），stdout 以 NUL 分隔：本机核数、线程数、explicit 或 default，其后是整条命令的词。
    有错退 2，原因打在 stdout（不带 ✗，checker-tier-crash-point-replay 自己打 ✗ 与出路）。"""
    flags = [argument for argument in arguments[2:]]
    if len(arguments) < 2 or any(flag not in ("--start-over", "--blocks-query") for flag in flags) or len(set(flags)) != len(flags):
        print("  ✗ 用法：admission.py crash-case-command <项目根> <键> [--start-over] [--blocks-query]")
        print("     → 怎么办：要只写不读（全部块现判、写回同一份库）带 --start-over；快档问块判定库带 --blocks-query")
        return EXIT_REGISTRATION_ERROR
    root, key = arguments[:2]
    try:
        case = crash_case_of_key(read_registration_rows(root), key)
        machine_cores, threads, threads_origin, command = crash_case_launch(root, case, "--start-over" in flags, os.environ, "--blocks-query" in flags)
    except (RegistrationError, InputManifestError) as error:
        print(f"交不出 {key} 的命令：{error}")
        return EXIT_REGISTRATION_ERROR
    words = [str(machine_cores), str(threads), threads_origin, *command]
    sys.stdout.write("".join(word + "\0" for word in words))
    return 0


def command_crash_case_shardable(arguments):
    if len(arguments) != 2:
        print("  ✗ 用法：admission.py crash-case-shardable <项目根> <键>")
        print("     → 怎么办：键照 .claude/gate.d/stage-inputs.tsv 里 crash-case: 开头的那一行写")
        return EXIT_REGISTRATION_ERROR
    root, key = arguments
    try:
        case = crash_case_of_key(read_registration_rows(root), key)
    except RegistrationError as error:
        print(f"找不到 {key}：{error}")
        return EXIT_REGISTRATION_ERROR
    if case.is_shardable_across_machines:
        print(f"{key} 登记了 shard={CRASH_CASE_SHARD_ACROSS_MACHINES}")
        return 0
    print(f"{key} 没登记 shard={CRASH_CASE_SHARD_ACROSS_MACHINES}：单机跑")
    return 1


def command_crash_case_judged_by_test(arguments):
    if len(arguments) != 2:
        print("  ✗ 用法：admission.py crash-case-judged-by-test <项目根> <键>")
        print("     → 怎么办：键照 .claude/gate.d/stage-inputs.tsv 里 crash-case: 开头的那一行写")
        return EXIT_REGISTRATION_ERROR
    root, key = arguments
    try:
        case = crash_case_of_key(read_registration_rows(root), key)
    except RegistrationError as error:
        print(f"找不到 {key}：{error}")
        return EXIT_REGISTRATION_ERROR
    if case.red_blocks_judged_by_test:
        print(f"{key} 登记了 red-blocks={CRASH_CASE_RED_BLOCKS_JUDGED_BY_TEST}：快档与 --full 起用例按退出码判")
        return 0
    print(f"{key} 没登记 red-blocks={CRASH_CASE_RED_BLOCKS_JUDGED_BY_TEST}：快档查库、--full 有红块判红")
    return 1



# ── 自证：单元键、判定库、门禁的复用答法、实验键与复跑表的行键（临时小仓，PATH 前面放假 cargo、假 rustc）──────────

SELFTEST_HERE = os.path.dirname(os.path.abspath(__file__))
PASSED_ONE_LINE = "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.01s"
# 临时仓里跑真仓脚本的副本时要从真仓链进来的两样：规范副本与项目的 preflight 垫片（.claude/scripts/preflight.sh）所在的目录
LINKED_INTO_SELFTEST_REPOSITORIES = (".claude/singlefs-ai-sop", ".claude/scripts")


def link_sop_copy(work):
    """临时仓里跑真仓脚本的副本（replay.sh 这一类）时，副本开头按相对路径 source .claude/scripts/preflight.sh，垫片再找规范副本里的 preflight.sh：
    LINKED_INTO_SELFTEST_REPOSITORIES 那两样在临时仓里建成指向真仓那一份的符号链接，并写进临时仓的 .gitignore（不算进改动范围）。"""
    os.makedirs(os.path.join(work, ".claude"), exist_ok=True)
    real_root = os.path.dirname(os.path.dirname(SELFTEST_HERE))
    with open(os.path.join(work, ".gitignore"), "a", encoding="utf-8") as handle:
        for relative_path in LINKED_INTO_SELFTEST_REPOSITORIES:
            os.symlink(os.path.join(real_root, relative_path), os.path.join(work, relative_path))
            handle.write(relative_path + "\n")

FAKE_RUSTC_VERSION_FILE = ".fake-rustc-version"   # 临时仓里假 rustc 读的版本串（E01：换一版工具链）


def fake_toolchain_scripts(version_file):
    """假 cargo、假 rustc：-V / -vV 照 version_file 里的一行打（文件不在打 0.0.0-selftest），别的参数一律退 0、什么都不做。"""
    rustc = ('#!/usr/bin/env bash\nversion="$(cat ' + f"'{version_file}'" + ' 2>/dev/null || echo 0.0.0-selftest)"\n'
             'case "${1:-}" in\n  -V) echo "rustc $version";;\n'
             '  -vV) printf "rustc %s\\nbinary: rustc\\nhost: x86_64-unknown-linux-gnu\\nrelease: %s\\n" "$version" "$version";;\nesac\nexit 0\n')
    cargo = '#!/usr/bin/env bash\n[[ "${1:-}" == -V || "${1:-}" == --version ]] && echo "cargo 0.0.0-selftest"\nexit 0\n'
    return {"rustc": rustc, "cargo": cargo}


UNIT_SAMPLE_FILES = {
    "Cargo.toml": '[workspace]\nmembers = ["crates/demo", "crates/other"]\nresolver = "2"\n\n[profile.release]\noverflow-checks = true\n',
    "Cargo.lock": ('# 样本的锁\nversion = 4\n\n[[package]]\nname = "demo"\nversion = "0.1.0"\ndependencies = [\n "itoa",\n]\n\n'
                   '[[package]]\nname = "itoa"\nversion = "1.0.0"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = "aaaa"\n\n'
                   '[[package]]\nname = "other"\nversion = "0.1.0"\ndependencies = [\n "ryu",\n]\n\n'
                   '[[package]]\nname = "ryu"\nversion = "1.0.0"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = "bbbb"\n'),
    "crates/demo/Cargo.toml": '[package]\nname = "demo"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\nitoa = "1"\n',
    "crates/demo/src/lib.rs": ("pub const WIDTH: u32 = 88;\n\npub fn helper() -> u32 {\n    WIDTH + 1\n}\n\npub fn unrelated() -> u32 {\n    7\n}\n\n"
                               "pub fn formats(value: u32) -> String {\n    itoa::Buffer::new().format(value).to_string()\n}\n"),
    "crates/demo/tests/setting.rs": ("use demo::helper;\n\nfn observer() -> u32 {\n    helper() * 2\n}\n\n#[test]\nfn test_t() {\n    assert_eq!(observer(), 178);\n}\n\n"
                                     "#[test]\nfn test_u() {\n    assert_eq!(2, 2);\n}\n\n#[test]\nfn test_v() {\n    assert_eq!(demo::formats(3), \"3\");\n}\n\n"
                                     "#[test]\n#[ignore]\nfn full_case() {}\n"),
    "crates/other/Cargo.toml": '[package]\nname = "other"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\nryu = "1"\n',
    "crates/other/src/lib.rs": "pub fn printed(value: f64) -> String {\n    ryu::Buffer::new().format(value).to_string()\n}\n",
    "scripts/judge.py": "def judge(line):\n    return line.endswith(' ok')\n\n\nif __name__ == '__main__':\n    print(judge('test a ... ok'))\n",
    "scripts/tool.py": ("import sys\n\n\ndef anchor_hints(text):\n    return text.split()\n\n\ndef selftest():\n    return 0 if anchor_hints('a b') == ['a', 'b'] else 1\n\n\n"
                        "if __name__ == '__main__':\n    sys.exit(selftest())\n"),
    "scripts/other.py": "def unrelated():\n    return 2\n\n\nif __name__ == '__main__':\n    print(unrelated())\n",
    ".claude/kb/decisions/08-sample.md": "# 样本决策\n\n一句。\n",
    REGISTRATION_TABLE: "# 样本登记表\ndemo-gate.sh\ttests-of:demo\t# 样本门禁\nother-gate.sh\tscripts/other.py\t# 别的一道\n",
}
UNIT_SAMPLE_TEST_TOKENS = ["profile:release", "judge:scripts/judge.py::judge"]


def unit_sample_requests():
    return [Request(f"crates/demo/tests/setting.rs::{name}", f"crates/demo/tests/setting.rs::{name}", UNIT_SAMPLE_TEST_TOKENS) for name in ("test_t", "test_u", "test_v")]


def edit_once(path, old, new):
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    if text.count(old) != 1:
        raise AssertionError(f"自证样本 {path} 里「{old}」命中 {text.count(old)} 次（要恰好一次）")
    write_text(path, text.replace(old, new))


def build_unit_sample_repository(work):
    """单元键那几格的临时小仓：两个工作区包（demo 用 itoa、other 用 ryu）、一份三条测试的设点文件、一个判法、两份 python 脚本、kb 决策、登记表；提交成 base。"""
    run_quietly(["git", "init", "-q", "-b", "master", work], GIT_IDENTITY)
    for path, text in UNIT_SAMPLE_FILES.items():
        write_text(os.path.join(work, path), text)
    tools = os.path.join(work, ".tools")
    for name, script in fake_toolchain_scripts(os.path.join(work, FAKE_RUSTC_VERSION_FILE)).items():
        write_executable(os.path.join(tools, name), script)
    write_text(os.path.join(work, ".gitignore"), ".tools/\n.fake-rustc-version\n*.plan\n*.requests\n*.names\n")
    run_quietly(["git", "-C", work, "add", "-A"], GIT_IDENTITY)
    run_quietly(["git", "-C", work, "commit", "-qm", "base"], GIT_IDENTITY)
    return environment_without_build_settings({"PATH": tools + os.pathsep + os.environ.get("PATH", ""), "CARGO_HOME": os.path.join(work, ".tools", "cargo-home")})


class EnvironmentPatch:
    """在本进程里临时换几样环境变量（给进程内算键那几格），退出时照原样还回去。"""
    def __init__(self, changes):
        self.changes = changes
        self.saved = {}

    def __enter__(self):
        for name, value in self.changes.items():
            self.saved[name] = os.environ.get(name)
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value
        return self

    def __exit__(self, *_exception):
        for name, value in self.saved.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value
        return False


def sample_keys(work, environment, requests=None):
    """进程内算一遍样本的键：{名字: 键或 None}。每次都新建 KeyContext（不吃上一格的缓存）。"""
    with EnvironmentPatch({name: environment.get(name) for name in ("PATH", "CARGO_HOME", "RUSTFLAGS", BREAK_VARIABLE)}):
        context = KeyContext(work, environment)
        return {request.name: compute_unit_key(context, request).key for request in (requests or unit_sample_requests())}


def short_name(name):
    return name.rsplit("::", 1)[-1]


def run_unit_key_cells(selftest, module):
    """单元键：只改注释、加一个没人调的函数、改别的测试、登记表加一行注释、无关包抬版本、构建配置只改注释 ⇒ 键不变；
    改被调的函数、被引用的常量、用到的外部包、工具链、RUSTFLAGS、release 档的设置、判法 ⇒ 用到它的那几条键变、别的不变。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-units-")
    try:
        environment = build_unit_sample_repository(work)
        baseline = sample_keys(work, environment)
        selftest.expect("单元键：样本三条测试都算得出键（不拒算）", all(baseline.values()), f"算出 {baseline}")
        lib = os.path.join(work, "crates/demo/src/lib.rs")
        setting = os.path.join(work, "crates/demo/tests/setting.rs")
        lock = os.path.join(work, "Cargo.lock")

        def cell(label, path, old, new, changed_names, environment_changes=None, revert=True):
            if path is not None:
                edit_once(path, old, new)
            try:
                after = sample_keys(work, dict(environment, **(environment_changes or {})))
            finally:
                if path is not None and revert:
                    edit_once(path, new, old)
            changed = sorted(short_name(name) for name in baseline if after.get(name) != baseline[name])
            selftest.expect(f"单元键：{label} ⇒ 变的是 {changed_names or '无'}", changed == sorted(changed_names), f"实际变的是 {changed or '无'}")
            return changed

        cell("被调函数 helper 体里只加一行注释（N01）", lib, "    WIDTH + 1\n", "    // 只是注释\n    WIDTH + 1\n", [])
        cell("同文件加一个没人调的函数（N04）", lib, "pub fn unrelated", "pub fn nobody_calls() -> u32 {\n    3\n}\n\npub fn unrelated", [])
        cell("被调函数 helper 改一个字（S02）", lib, "WIDTH + 1", "WIDTH + 2", ["test_t"])
        cell("被引用的常量 WIDTH 88 → 89（V11 形态）", lib, "pub const WIDTH: u32 = 88;", "pub const WIDTH: u32 = 89;", ["test_t"])
        cell("同设点文件里别的测试 test_u 改一个字（N19）", setting, "assert_eq!(2, 2);", "assert_eq!(2, 2 + 0);", ["test_u"])
        cell("观察者函数 observer 改一个字（S17）", setting, "helper() * 2", "helper() * 3", ["test_t"])
        cell("测试模块加一条空单测（N06）", setting, "#[test]\n#[ignore]", "#[test]\nfn added() {}\n\n#[test]\n#[ignore]", [])
        cell("仓根 Cargo.toml 加一行注释（N07）", os.path.join(work, "Cargo.toml"), "resolver = \"2\"\n", "resolver = \"2\"\n# 只是注释\n", [])
        cell("Cargo.lock 里没人用的 ryu 抬版本（N09）", lock, 'name = "ryu"\nversion = "1.0.0"', 'name = "ryu"\nversion = "1.0.1"', [])
        cell("Cargo.lock 里 formats 用到的 itoa 抬版本（S14 形态）", lock, 'name = "itoa"\nversion = "1.0.0"', 'name = "itoa"\nversion = "1.0.1"', ["test_v"])
        by_crate = dict(environment, **{BREAK_VARIABLE: "dependencies-by-crate"})
        before_bump = sample_keys(work, by_crate)
        edit_once(lock, 'name = "itoa"\nversion = "1.0.0"', 'name = "itoa"\nversion = "1.0.1"')
        after_bump = sample_keys(work, by_crate)
        edit_once(lock, 'name = "itoa"\nversion = "1.0.1"', 'name = "itoa"\nversion = "1.0.0"')
        selftest.expect("弄坏开关 dependencies-by-crate 下 S14 那一格转红（外部包按包收：itoa 抬版本三条测试全变）",
                        all(before_bump[name] != after_bump[name] for name in before_bump), f"变了的 {[short_name(name) for name in before_bump if before_bump[name] != after_bump[name]]}")
        cell("release 档的 overflow-checks 改成 false（S16）", os.path.join(work, "Cargo.toml"), "overflow-checks = true", "overflow-checks = false",
             ["test_t", "test_u", "test_v"])
        cell("登记表加一行注释、别的一道多登记一个入口（N22、N23）", os.path.join(work, REGISTRATION_TABLE), "other-gate.sh\tscripts/other.py",
             "# 加一行注释\nother-gate.sh\tscripts/other.py scripts/tool.py", [])
        cell("判法 scripts/judge.py::judge 只加一行注释", os.path.join(work, "scripts/judge.py"), "    return line.endswith(' ok')",
             "    # 只是注释\n    return line.endswith(' ok')", [])
        cell("判法 scripts/judge.py::judge 改一个字", os.path.join(work, "scripts/judge.py"), "line.endswith(' ok')", "line.endswith(' ok ')",
             ["test_t", "test_u", "test_v"])
        write_text(os.path.join(work, FAKE_RUSTC_VERSION_FILE), "1.99.0-selftest\n")
        cell("工具链换一版（E01：假 rustc 的 -vV 换了）", None, None, None, ["test_t", "test_u", "test_v"])
        os.remove(os.path.join(work, FAKE_RUSTC_VERSION_FILE))
        cell("RUSTFLAGS 加 -C target-cpu=native（E02）", None, None, None, ["test_t", "test_u", "test_v"], {"RUSTFLAGS": "-C target-cpu=native"})
        cell("弄坏开关 skip-build-environment 下 RUSTFLAGS 那一格转红（键不变）", None, None, None, [],
             {"RUSTFLAGS": "-C target-cpu=native", BREAK_VARIABLE: "skip-build-environment"})
        write_text(os.path.join(work, ".cargo/config.toml"), "# 构建配置\n[build]\nrustflags = [\"-Cdebuginfo=1\"]\n")
        with_configuration = sample_keys(work, environment)
        edit_once(os.path.join(work, ".cargo/config.toml"), "# 构建配置\n", "# 构建配置，只改注释\n")
        comment_only = sample_keys(work, environment)
        edit_once(os.path.join(work, ".cargo/config.toml"), "-Cdebuginfo=1", "-Cdebuginfo=2")
        value_changed = sample_keys(work, environment)
        os.remove(os.path.join(work, ".cargo/config.toml"))
        selftest.expect("单元键：.cargo/config.toml 只改注释键不变（N42）、改 rustflags 的值键变",
                        comment_only == with_configuration and all(value_changed[name] != with_configuration[name] for name in with_configuration),
                        f"只改注释 {comment_only == with_configuration}，改值 {[value_changed[name] != with_configuration[name] for name in with_configuration]}")
        python_requests = [Request("tool", "scripts/tool.py", []), Request("other", "scripts/other.py", [])]
        python_before = sample_keys(work, environment, python_requests)
        edit_once(os.path.join(work, "scripts/tool.py"), "return text.split()", "return text.split(None)")
        python_after = sample_keys(work, environment, python_requests)
        edit_once(os.path.join(work, "scripts/tool.py"), "return text.split(None)", "return text.split()")
        selftest.expect("单元键：研究脚本自证走到的 anchor_hints 改一个字（T21）⇒ 只 scripts/tool.py 那一条变",
                        python_before["tool"] and python_before["tool"] != python_after["tool"] and python_before["other"] == python_after["other"],
                        f"改前 {python_before}，改后 {python_after}")
        exit_code, output, _messages = run_in_environment([sys.executable, module, "unit-key", work, "crates/demo/tests/setting.rs::no_such_test"], environment)
        selftest.expect("unit-key：入口找不到 ⇒ 拒算退 4、打拒算行", exit_code == EXIT_UNIT_KEY_REFUSED and output.startswith("refused\tkey-uncomputable"),
                        f"退 {exit_code}，stdout「{output.strip()[:300]}」")
        exit_code, output, _messages = run_in_environment([sys.executable, module, "test-functions", work, "demo"], environment)
        listed = [line.split("\t") for line in output.strip().split("\n") if line]
        states = {row[3]: row[4] for row in listed if len(row) == 5}
        selftest.expect("test-functions：列出 demo 的四条测试（三条 compiled、标了 ignore 的 full_case 是 ignored）",
                        exit_code == 0 and states == {"test_t": "compiled", "test_u": "compiled", "test_v": "compiled", "full_case": "ignored"},
                        f"退 {exit_code}，列出 {states}")
    except AssertionError as error:
        selftest.expect("单元键的样本改得动", False, str(error))
    finally:
        shutil.rmtree(work, ignore_errors=True)


def run_verdict_library_cells(selftest, module):
    """判定库：头一趟全现判、记绿之后第二趟全复用；改一个被调函数只那一条现判、原因点名变了的单元；全量只写不读；跑的过程中键变了的不记；
    门禁的复用答法：全部有判定退 10、有一条对不上退 0 并点名；H2（改动带着 HEAD 一起前移、staged-green 跟着挪）必须现判，H3（空提交）照复用；
    弄坏开关 tree-compare-reuse 下 H2 那一格转红。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-library-")
    try:
        environment = build_unit_sample_repository(work)
        # 复用哨兵钉成默认的 3 条：这几格的库里至多 3 条，都当哨兵、不抽强制现判，现判与复用的条数照下面写的判（哨兵那一格在 run_reuse_three_question_cells）
        environment.update({"SINGLEFS_REUSE_SENTINEL_COUNT": "3", "SINGLEFS_LAST_RUNS_DIRECTORY": os.path.join(work, ".last-runs")})
        requests_file = os.path.join(work, "demo.requests")
        plan_file = os.path.join(work, "demo.plan")
        names_file = os.path.join(work, "demo.names")
        write_text(requests_file, "".join(request.text() + "\n" for request in unit_sample_requests()))

        def admission(*arguments, changes=None):
            return run_in_environment([sys.executable, module, *arguments], dict(environment, **(changes or {})))

        def plan(changes=None):
            exit_code, output, messages = admission("verdict-plan", work, "demo-gate.sh", requests_file, plan_file, changes=changes)
            decisions = {short_name(line.split("\t")[0]): line.split("\t")[1] for line in output.split("\n") if line.count("\t") >= 3}
            return exit_code, decisions, output + messages

        def record(names, changes=None):
            write_text(names_file, "".join(f"crates/demo/tests/setting.rs::{name}\n" for name in names))
            return admission("verdict-record", work, "demo-gate.sh", plan_file, names_file, "--how", "cargo test --release -p demo（自证）", changes=changes)

        exit_code, decisions, output = plan()
        selftest.expect("verdict-plan：库是空的 ⇒ 三条都 judge，末行 PLAN judge=3 reuse=0 refused=0",
                        exit_code == 0 and decisions == {"test_t": "judge", "test_u": "judge", "test_v": "judge"} and "PLAN judge=3 reuse=0 refused=0" in output,
                        f"退 {exit_code}，{decisions}，{output.strip()[-300:]}")
        exit_code, output, _messages = record(["test_t", "test_u", "test_v"])
        selftest.expect("verdict-record：三条判绿都记进库（RECORD recorded=3 changed=0）", exit_code == 0 and "RECORD recorded=3 changed=0" in output,
                        f"退 {exit_code}，{output.strip()}")
        exit_code, decisions, output = plan()
        selftest.expect("verdict-plan：记过之后第二趟三条都 reuse", decisions == {"test_t": "reuse", "test_u": "reuse", "test_v": "reuse"}, f"{decisions}，{output.strip()[-300:]}")
        common_directory = git_common_directory(work)
        record_path = os.path.join(common_directory, VERDICT_LIBRARY_DIRECTORY_NAME, "demo-gate.sh", "keys")
        records = sorted(os.listdir(record_path)) if os.path.isdir(record_path) else []
        sample_record = {}
        if records:
            with open(os.path.join(record_path, records[0]), encoding="utf-8") as handle:
                sample_record = json.load(handle)
        composition = VerdictLibrary(work, "demo-gate.sh").composition_of(sample_record.get("key", "")) or []
        selftest.expect("判定库在 git common-dir 的 singlefs-unit-verdicts/<种类>/ 下，每条记着键、判定日期与跑法，组成清单按键另存（compositions/<键>.tsv.gz）",
                        len(records) == 3 and re.fullmatch(r"\d{4}-\d{2}-\d{2}", sample_record.get("judged", "")) is not None
                        and sample_record.get("how", "").startswith("cargo test") and any(line.startswith("fn\t") for line in composition),
                        f"{record_path} 里 {records}，样本记录的键 {sorted(sample_record)}，组成清单 {len(composition)} 行")
        edit_once(os.path.join(work, "crates/demo/src/lib.rs"), "WIDTH + 1", "WIDTH + 2")
        exit_code, decisions, output = plan()
        reason = next((line.split("\t", 3)[3] for line in output.split("\n") if "::test_t\t" in line), "")
        selftest.expect("verdict-plan：改了被调函数 helper ⇒ 只 test_t 现判，原因点名「变了 fn crates/demo/src/lib.rs::helper」",
                        decisions == {"test_t": "judge", "test_u": "reuse", "test_v": "reuse"} and "变了 fn crates/demo/src/lib.rs::helper" in reason,
                        f"{decisions}，原因「{reason}」")
        exit_code, decisions, output = plan({"SINGLEFS_GATE_FULL": "1"})
        selftest.expect("verdict-plan：SINGLEFS_GATE_FULL=1 ⇒ 只写不读，三条都 judge", decisions == {"test_t": "judge", "test_u": "judge", "test_v": "judge"},
                        f"{decisions}")
        plan()
        edit_once(os.path.join(work, "crates/demo/src/lib.rs"), "WIDTH + 2", "WIDTH + 3")
        exit_code, output, _messages = record(["test_t"])
        selftest.expect("verdict-record：开跑之后被判的函数又改了 ⇒ 不记、点名（changed=1）", "RECORD recorded=0 changed=1" in output, output.strip())
        edit_once(os.path.join(work, "crates/demo/src/lib.rs"), "WIDTH + 3", "WIDTH + 1")

        # 门禁的复用答法：H2 / H3 / staged-green 不读（controls L15）
        def reuse(changes=None, with_requests=True):
            arguments = ["gate-reuse", work, "demo-gate.sh"] + ([requests_file] if with_requests else [])
            return admission(*arguments, changes=changes)

        exit_code, output, _messages = reuse()
        selftest.expect("gate-reuse：登记的三条在库里都有键相同的判定 ⇒ 退 10 可跳过", exit_code == EXIT_GATE_MAY_SKIP, f"退 {exit_code}：{output.strip()}")
        exit_code, output, _messages = reuse(with_requests=False)
        selftest.expect("gate-reuse：不给请求文件（tests-of:demo 展开、按库里记的请求重算）⇒ 同样退 10", exit_code == EXIT_GATE_MAY_SKIP, f"退 {exit_code}：{output.strip()}")
        edit_once(os.path.join(work, "crates/demo/src/lib.rs"), "WIDTH + 1", "WIDTH + 2")
        run_quietly(["git", "-C", work, "commit", "-qam", "改动带着 HEAD 一起前移"], GIT_IDENTITY)
        mark_tree_green(work, staged_tree_of(work))
        exit_code, output, _messages = reuse()
        selftest.expect("gate-reuse：H2（改了 helper 并提交、refs/sop/staged-green 也挪到这一棵）⇒ 退 0 要跑，点名 test_t、报键对不上的 1 条",
                        exit_code == EXIT_GATE_MUST_RUN and "键对不上的 1 条" in output and "::test_t" in output, f"退 {exit_code}：{output.strip()}")
        exit_code, output, _messages = reuse({BREAK_VARIABLE: "tree-compare-reuse"})
        selftest.expect("弄坏开关 tree-compare-reuse 下 H2 那一格转红（按树比，答可跳过）", exit_code == EXIT_GATE_MAY_SKIP, f"退 {exit_code}：{output.strip()}")
        edit_once(os.path.join(work, "crates/demo/src/lib.rs"), "WIDTH + 2", "WIDTH + 1")
        run_quietly(["git", "-C", work, "commit", "-qam", "改回去"], GIT_IDENTITY)
        run_quietly(["git", "-C", work, "commit", "-q", "--allow-empty", "-m", "空提交"], GIT_IDENTITY)
        mark_tree_green(work, staged_tree_of(work))
        exit_code, output, _messages = reuse()
        selftest.expect("gate-reuse：H3（空提交，HEAD 前移、内容回到判过的样子）⇒ 退 10 照复用", exit_code == EXIT_GATE_MAY_SKIP, f"退 {exit_code}：{output.strip()}")
        write_text(os.path.join(work, FAKE_RUSTC_VERSION_FILE), "1.99.0-selftest\n")
        exit_code, output, _messages = reuse()
        os.remove(os.path.join(work, FAKE_RUSTC_VERSION_FILE))
        selftest.expect("gate-reuse：E01（只换工具链）⇒ 退 0 要跑，三条都键对不上", exit_code == EXIT_GATE_MUST_RUN and "键对不上的 3 条" in output,
                        f"退 {exit_code}：{output.strip()}")
        exit_code, output, _messages = reuse({"SINGLEFS_GATE_FULL": "1"})
        selftest.expect("gate-reuse：SINGLEFS_GATE_FULL=1 ⇒ 退 0 要跑（只写不读）", exit_code == EXIT_GATE_MUST_RUN and "SINGLEFS_GATE_FULL=1" in output,
                        f"退 {exit_code}：{output.strip()}")
    except AssertionError as error:
        selftest.expect("判定库的样本改得动", False, str(error))
    finally:
        shutil.rmtree(work, ignore_errors=True)


def run_reuse_three_question_cells(selftest, module):
    """verdict-plan / verdict-record 的复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」，格名 reuse-three-questions）与复用哨兵（「二」，格名 reuse-sentinel）。
    每一串各用一份自己的临时小仓（单元键那几格的样本）、自己的判定库（种类 reuse-questions-gate.sh，在那份临时仓的 git common-dir 里），哨兵的条数与种子在这里钉死、
    上次红的清单（SINGLEFS_LAST_RUNS_DIRECTORY）指到临时仓里，不读写共用的位置：
      四趟：库是空的（三条都现判，记绿）、同一份输入（①）、设点文件里加一个没人调的函数（闭包外，②）、改 helper（只在 test_t 的闭包里，③ affected test_t）；
      这四趟哨兵数设 1000（键相同的全当哨兵、不抽强制现判），规范输出照 report_three_questions 打。
      哨兵：哨兵 1 条、种子 7：一条当哨兵复用、一条强制现判（verdict-record 判绿之后报与库里同为绿），整趟退 0；
      开着弄坏开关 verdict-lookup-skipped（不读库）再跑同一个种子，退 1、红「复用失灵」（共用库的 REUSE_CHECKS_BREAK=rejudged-sentinel-passes 下这一格判错）。
      另一份临时小仓开着弄坏开关 verdict-key-with-entry-file（键另加入口所在的整份文件）再走那四趟：② 必须红（这一串的规范输出不打）。"""
    checks = reuse_checks_module()
    if checks is None:
        selftest.expect(f"reuse-three-questions：同目录有 {REUSE_CHECKS_LIBRARY}", False, "找不到复用三问的判法，这一格判不了")
        return
    kind = "reuse-questions-gate.sh"
    names = [request.name for request in unit_sample_requests()]

    class Questions:
        """一份临时小仓上的 verdict-plan / verdict-record。"""

        def __init__(self, work, changes):
            self.work = work
            self.environment = build_unit_sample_repository(work)
            self.environment.pop(checks.SENTINEL_SEED_VARIABLE, None)
            self.environment.update({checks.SENTINEL_COUNT_VARIABLE: "1000", checks.LAST_RUNS_VARIABLE: os.path.join(work, ".last-runs")}, **changes)
            self.requests_file, self.plan_file, self.names_file = (os.path.join(work, f"questions.{suffix}") for suffix in ("requests", "plan", "names"))
            write_text(self.requests_file, "".join(request.text() + "\n" for request in unit_sample_requests()))

        def admission(self, *arguments, changes=None):
            return run_in_environment([sys.executable, module, *arguments], dict(self.environment, **(changes or {})))

        def plan(self, changes=None):
            """一趟 verdict-plan：交 (退出码, {项: 复用 | 现判 | 强制现判 | 拒算}, 原样输出)。"""
            exit_code, output, messages = self.admission("verdict-plan", self.work, kind, self.requests_file, self.plan_file, changes=changes)
            states = {}
            for line in output.split("\n"):
                columns = line.split("\t")
                if len(columns) >= 4 and columns[0] in names:
                    decision, reason = columns[1], columns[3]
                    states[short_name(columns[0])] = ("复用" if decision == "reuse" else "拒算" if decision == "refused"
                                                      else "强制现判" if reason.startswith("强制现判") else "现判")
            return exit_code, states, output + messages

        def record(self, green, changes=None):
            write_text(self.names_file, "".join(f"crates/demo/tests/setting.rs::{name}\n" for name in green))
            return self.admission("verdict-record", self.work, kind, self.plan_file, self.names_file, "--how", "cargo test --release -p demo（自证）", changes=changes)

        def four_runs(self):
            """头一趟（记绿）、同一份输入、闭包外改动、闭包内改 helper：交 (四趟的结局, 末趟输出)。"""
            _exit, first, _output = self.plan()
            self.record(sorted(first))
            _exit, second, _output = self.plan()
            edit_once(os.path.join(self.work, "crates/demo/tests/setting.rs"), "#[test]\nfn test_u()",
                      "fn spare_unused() -> u32 {\n    9\n}\n\n#[test]\nfn test_u()")
            _exit, unrelated, _output = self.plan()
            edit_once(os.path.join(self.work, "crates/demo/src/lib.rs"), "WIDTH + 1", "WIDTH + 2")
            _exit, unit_change, unit_output = self.plan()
            return (first, second, unrelated, unit_change), unit_output

    works = [tempfile.mkdtemp(prefix="admission-selftest-reuse-questions-") for _ in range(2)]
    try:
        questions = Questions(works[0], {})
        runs, unit_output = questions.four_runs()
        lines = []
        passed, problems = checks.report_three_questions("scripts/gate/admission.py", *runs, ["test_t"], printer=lines.append)
        for line in lines:
            print(line)
        selftest.expect("reuse-three-questions：verdict-plan 头一趟三条现判、同一份输入第二趟全复用、闭包外加一个没人调的函数全复用、改 helper 只现判 test_t",
                        passed, f"{'；'.join(problems)}；四趟：{' / '.join(map(str, runs))}；末趟输出：{unit_output.strip()[-300:]}")

        # 复用哨兵：库里三条都记绿（键是此刻的），哨兵 1 条、种子 7
        questions.record(["test_t"])
        sentinel_settings = {checks.SENTINEL_COUNT_VARIABLE: "1", checks.SENTINEL_SEED_VARIABLE: "7"}
        green_exit, states, green_output = questions.plan(sentinel_settings)
        forced = [name for name, state in states.items() if state == "强制现判"]
        record_exit, record_output, _messages = questions.record(sorted(states), sentinel_settings)
        lookup_exit, _states, lookup_output = questions.plan(dict(sentinel_settings, **{BREAK_VARIABLE: "verdict-lookup-skipped"}))
        selftest.expect("reuse-sentinel：哨兵 1 条、种子 7 那一趟退 0、种子打进输出、一条强制现判（verdict-record 报与库里同为绿）；不读库那一趟退 1、红「复用失灵」",
                        green_exit == 0 and "种子 7（SINGLEFS_REUSE_SENTINEL_SEED=7 复现）" in green_output and len(forced) == 1
                        and record_exit == 0 and f"强制现判 crates/demo/tests/setting.rs::{forced[0] if forced else '?'} 与库里同为「绿」" in record_output
                        and lookup_exit == 1 and "复用失灵：" in lookup_output,
                        f"照常那一趟退 {green_exit}、结局 {states}；verdict-record 退 {record_exit}：{record_output.strip()[-300:]}；"
                        f"不读库那一趟退 {lookup_exit}：{lookup_output.strip()[-400:]}；照常那一趟输出：{green_output.strip()[-400:]}")

        # 弄坏开关 verdict-key-with-entry-file：另一份临时小仓，键另加入口所在的整份文件，闭包外加函数那一趟必须现判（② 红）
        broken_runs, _output = Questions(works[1], {BREAK_VARIABLE: "verdict-key-with-entry-file"}).four_runs()
        broken_passed, _line, broken_problems = checks.three_questions("scripts/gate/admission.py", *broken_runs, ["test_t"])
        selftest.expect("弄坏开关 verdict-key-with-entry-file 下 reuse-three-questions 的 ② 转红（闭包外的改动也让整份入口文件的三条作废）",
                        not broken_passed and any(problem.startswith("② ") for problem in broken_problems),
                        f"三问{'过了' if broken_passed else '红在：' + '；'.join(broken_problems)}；四趟：{' / '.join(map(str, broken_runs))}")
    except AssertionError as error:
        selftest.expect("reuse-three-questions：样本改得动", False, str(error))
    finally:
        for work in works:
            shutil.rmtree(work, ignore_errors=True)


RESEARCH_SAMPLE_FILES = {
    "scripts/experiments/Cargo.toml": ('[package]\nname = "e7-index-bench"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\naes-gcm = "0.11.1"\n\n'
                                       '[lib]\npath = "lib.rs"\n\n[[bin]]\nname = "e23_journal_geom"\npath = "e23_journal_geom.rs"\n\n'
                                       '[[bin]]\nname = "e140-header-alignment"\npath = "e140_header_alignment.rs"\n\n[[bin]]\nname = "e900"\npath = "e900.rs"\n'),
    "scripts/experiments/Cargo.lock": ('version = 4\n\n[[package]]\nname = "aes-gcm"\nversion = "0.11.1"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\n'
                                       'checksum = "cccc"\n\n[[package]]\nname = "e7-index-bench"\nversion = "0.1.0"\ndependencies = [\n "aes-gcm",\n]\n'),
    "scripts/experiments/lib.rs": ("pub struct Emitter;\n\nimpl Emitter {\n    pub fn emit(&self) -> u32 {\n        1\n    }\n}\n\n"
                                   "pub fn lru_new() -> u32 {\n    assert!(true, \"容量\");\n    5\n}\n"),
    "scripts/experiments/e23_journal_geom.rs": ("use e7_index_bench::Emitter;\n\nfn simulate_journal_run() -> u64 {\n    let start = 0;\n    start\n}\n\n"
                                                "fn main() {\n    let _ = Emitter.emit();\n    let _ = simulate_journal_run();\n}\n"),
    "scripts/experiments/e140_header_alignment.rs": "use aes_gcm::Aes256Gcm;\n\nfn main() {\n    let _ = std::mem::size_of::<Aes256Gcm>();\n}\n",
    "scripts/experiments/e900.rs": "fn main() {\n    let _ = 1;\n}\n",
    RESULTS_DIRECTORY + "/e23-2026-09-01.out": "E7RESULT name=e23 rows=1\nE7RESULT name=done emitted=2\n",
    RESULTS_DIRECTORY + "/e140-2026-09-01.out": "E7RESULT name=e140 rows=1\nE7RESULT name=done emitted=2\n",
    RESULTS_DIRECTORY + "/e900-2026-09-01.out": "E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n",
    RESULTS_DIRECTORY + "/e901-2026-09-01.out": "E7RESULT name=e901 rows=1\nE7RESULT name=done emitted=2\n",
    ".claude/kb/decisions/08-sample.md": "# 样本决策\n\n一句。\n",
    REGISTRATION_TABLE: ("# 样本登记表\nE900\tscripts/experiments/e900.rs::main\t# 样本实验\n"
                         "E900/second\t@E900\tquestion-row=research/questions.md#6:够判[：:][^（(|]*对照(本身)?是好的 product-field=E900:verdict:control_ok=true\t# 样本第二段\n"
                         "E900/third\t@E900\tsame-run-field=verdict:control_ok=true\t# 样本第二段的同一趟现判写法：前一段这一趟的输出经环境变量交来\n"),
}
RESEARCH_SAMPLE_REPLAY = """#!/usr/bin/env bash
# 自证的复跑表（admission.py --selftest 的样本：只有判一行的几个函数与两个驱动，不跑东西）
TABLE=$(cat <<'TSV'
E23|e23_journal_geom||e23-2026-09-01.out|exact
E140|e140-header-alignment||e140-2026-09-01.out|exact
E900|@driver_e900||e900-2026-09-01.out|exact
E901|@driver_gpu||e901-2026-09-01.out|exact
TSV
)
ADMISSION_ROW_KEYS=$(cat <<'TSV'
TSV
)
replay_judge_fresh_product() {
  diff -q "$1" "$2" >/dev/null
}
strip_admission_header() { sed '/^E7INPUT /d'; }
strip_timing() { sed 's/ elapsed=[0-9.]*//'; }
strip_load_fields() { sed 's/ loadavg1=[0-9.]*//'; }
timing_load_reading_raw() { echo 0; }
timing_load_decide() { echo judged; }
driver_e900() {
  printf 'E7RESULT name=verdict control_ok=true\\nE7RESULT name=done emitted=2\\n'
}
driver_gpu() {
  echo "${CHECKER_TIER_FEATURES[@]}"
  ./scripts/experiments/target/release/e900
}
"""


def build_research_sample_repository(work):
    run_quietly(["git", "init", "-q", "-b", "master", work], GIT_IDENTITY)
    for path, text in RESEARCH_SAMPLE_FILES.items():
        write_text(os.path.join(work, path), text)
    write_text(os.path.join(work, REPLAY_SCRIPT), RESEARCH_SAMPLE_REPLAY)
    write_questions(work, "开着")
    tools = os.path.join(work, ".tools")
    for name, script in fake_toolchain_scripts(os.path.join(work, FAKE_RUSTC_VERSION_FILE)).items():
        write_executable(os.path.join(tools, name), script)
    os.makedirs(os.path.join(work, "scripts", "gate"), exist_ok=True)
    shutil.copy(os.path.abspath(__file__), os.path.join(work, "scripts/gate/admission.py"))
    shutil.copy(os.path.join(SELFTEST_HERE, FUNCTION_UNITS_LIBRARY), os.path.join(work, "scripts/gate", FUNCTION_UNITS_LIBRARY))
    link_sop_copy(work)
    write_text(os.path.join(work, ".gitignore"), ".tools/\n.fake-rustc-version\n.claude/singlefs-ai-sop\n.claude/scripts\n")
    run_quietly(["git", "-C", work, "add", "-A"], GIT_IDENTITY)
    run_quietly(["git", "-C", work, "commit", "-qm", "base"], GIT_IDENTITY)
    return environment_without_build_settings({"PATH": tools + os.pathsep + os.environ.get("PATH", ""), "CARGO_HOME": os.path.join(work, ".tools", "cargo-home")})


def run_experiment_cells(selftest, module):
    """实验的准入（行键去掉期望产物）：老产物没有实验键放行；存一份带它的产物之后判「输入没变」退 77；改装置 main 放行、改回去又 77；
    装置只加注释、kb 决策文档改一句（T19）、登记表别的行与注释（N22）都照 77；强制开关；前提；登记错退 2；给 doc-experiments 的四种状态。
    弄坏开关 experiment-key-with-kb-documents 下 T19 那一格转红。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-experiment-")
    try:
        environment = build_research_sample_repository(work)
        copied = os.path.join(work, "scripts/gate/admission.py")

        def experiment(key, changes=None):
            return run_in_environment([sys.executable, copied, "experiment", work, key], dict(environment, **(changes or {})))

        exit_code, output, messages = experiment("E900")
        first_header = output.strip().split("\n")[0] if output.strip() else ""
        selftest.expect("实验准入：老产物头上没有实验键时放行并报判不了", exit_code == 0 and "判不了" in messages, f"退 {exit_code}，stderr：{messages.strip()[-400:]}")
        selftest.expect("实验准入：放行时产物头第一行是带键的实验键行（sha256= 64 位十六进制、units= 单元数）",
                        re.fullmatch(r"E7INPUT name=input_fingerprint key=E900 sha256=[0-9a-f]{64} units=[1-9][0-9]*", first_header) is not None,
                        f"头一行是「{first_header}」")
        write_text(os.path.join(work, RESULTS_DIRECTORY, "e900-2026-09-02.out"), first_header + "\nE7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n")
        edit_once(os.path.join(work, REPLAY_SCRIPT), "E900|@driver_e900||e900-2026-09-01.out|exact", "E900|@driver_e900||e900-2026-09-02.out|exact")
        exit_code, output, messages = experiment("E900")
        selftest.expect("实验准入：存了带这把实验键的产物、复跑表指到它 ⇒ 输入没变，退 77、stdout 为空、出路写强制开关（改复跑表的产物文件名不动实验键）",
                        exit_code == EXIT_INPUTS_UNCHANGED and output == "" and FORCE_RERUN_VARIABLE in messages, f"退 {exit_code}，stderr：{messages.strip()[-400:]}")
        exit_code, _output, _messages = experiment("E900", {BREAK_VARIABLE: "skip-unchanged"})
        selftest.expect("弄坏开关 skip-unchanged 下「输入没变」那一格红（放行了）", exit_code == 0, f"弄坏之后仍退 {exit_code}")
        device = os.path.join(work, "scripts/experiments/e900.rs")
        edit_once(device, "let _ = 1;", "let _ = 2;")
        exit_code, _output, messages = experiment("E900")
        selftest.expect("实验准入：装置 main 改一个字 ⇒ 放行并报变了", exit_code == 0 and "变了" in messages, f"退 {exit_code}，stderr：{messages.strip()[-400:]}")
        edit_once(device, "let _ = 2;", "let _ = 1;")
        edit_once(device, "    let _ = 1;\n", "    // 只是注释\n    let _ = 1;\n")
        exit_code, _output, _messages = experiment("E900")
        selftest.expect("实验准入：装置 main 只加一行注释 ⇒ 照判没变（退 77）", exit_code == EXIT_INPUTS_UNCHANGED, f"退 {exit_code}")
        decision = os.path.join(work, ".claude/kb/decisions/08-sample.md")
        edit_once(decision, "一句。", "改了一句。")
        exit_code, _output, _messages = experiment("E900")
        selftest.expect("实验准入：T19（kb 决策文档改一句）⇒ 实验键不变，照判没变（退 77）", exit_code == EXIT_INPUTS_UNCHANGED, f"退 {exit_code}")
        exit_code, _output, _messages = experiment("E900", {BREAK_VARIABLE: "experiment-key-with-kb-documents"})
        selftest.expect("弄坏开关 experiment-key-with-kb-documents 下 T19 那一格转红（放行了）", exit_code == 0, f"弄坏之后仍退 {exit_code}")
        table_path = os.path.join(work, REGISTRATION_TABLE)
        with open(table_path, encoding="utf-8") as handle:
            good_table = handle.read()
        write_text(table_path, good_table + "# 加一行注释\nE901\tscripts/experiments/e23_journal_geom.rs::main scripts/experiments/e900.rs::main\t# 别的一行\n")
        exit_code, _output, _messages = experiment("E900")
        selftest.expect("实验准入：N22（登记表加注释、别的实验行多登记入口）⇒ 照判没变（退 77）", exit_code == EXIT_INPUTS_UNCHANGED, f"退 {exit_code}")
        write_text(table_path, good_table)
        exit_code, output, _messages = experiment("E900", {FORCE_RERUN_VARIABLE: "换了机器  复核一次"})
        selftest.expect("实验准入：强制开关带理由时放行、理由原样进产物头",
                        exit_code == 0 and "E7INPUT name=forced_rerun key=E900 overrode_unchanged=true reason=换了机器 复核一次" in output, f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, _output, _messages = experiment("E900", {FORCE_RERUN_VARIABLE: "   "})
        selftest.expect("实验准入：强制开关理由为空时退 2", exit_code == EXIT_REGISTRATION_ERROR, f"退 {exit_code}")
        exit_code, _output, messages = experiment("E900/second")
        selftest.expect("实验准入：前提没齐时拒绝（退 3）并点名那一行", exit_code == EXIT_PRECONDITION_MISSING and "第 6 行的状态是「开着」" in messages,
                        f"退 {exit_code}，stderr：{messages.strip()[-400:]}")
        exit_code, _output, _messages = experiment("E900/second", {BREAK_VARIABLE: "skip-preconditions"})
        selftest.expect("弄坏开关 skip-preconditions 下「前提没齐」那一格红（放行了）", exit_code == 0, f"弄坏之后仍退 {exit_code}")
        write_questions(work, "**够判：装置的期望值过时、对照本身是好的**（样本）")
        exit_code, _output, messages = experiment("E900/second")
        selftest.expect("实验准入：前提齐了放行（E900/second 不在复跑表里，按登记的入口算键）", exit_code == 0, f"退 {exit_code}，stderr：{messages.strip()[-400:]}")
        # same-run-field（文件头「准入条件」）：前一段这一趟的输出经 SAME_RUN_PRODUCT_VARIABLE 交来；复跑表登记给 E900 的留存产物里写着 control_ok=true，不算数
        same_run_product = os.path.join(work, ".same-run", "first-segment.out")
        environment.pop(SAME_RUN_PRODUCT_VARIABLE, None)   # 调用这一份自证的环境里设着它也不算
        exit_code, _output, messages = experiment("E900/third")
        selftest.expect("same-run-field：没交同一趟的输出时前提没齐（退 3）并点名那个环境变量", exit_code == EXIT_PRECONDITION_MISSING and SAME_RUN_PRODUCT_VARIABLE in messages,
                        f"退 {exit_code}，stderr：{messages.strip()[-400:]}")
        write_text(same_run_product, "E7RESULT name=verdict control_ok=false\nE7RESULT name=done emitted=2\n")
        exit_code, _output, messages = experiment("E900/third", {SAME_RUN_PRODUCT_VARIABLE: same_run_product})
        selftest.expect("same-run-field：同一趟的输出里值不等时前提没齐（退 3），留存产物里的 control_ok=true 不算数",
                        exit_code == EXIT_PRECONDITION_MISSING and "control_ok=false" in messages, f"退 {exit_code}，stderr：{messages.strip()[-400:]}")
        exit_code, _output, _messages = experiment("E900/third", {SAME_RUN_PRODUCT_VARIABLE: same_run_product, BREAK_VARIABLE: "same-run-field-unchecked"})
        selftest.expect("弄坏开关 same-run-field-unchecked 下「same-run-field：同一趟的输出里值不等」那一格红（放行了）", exit_code == 0, f"弄坏之后仍退 {exit_code}")
        write_text(same_run_product, "E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n")
        exit_code, _output, messages = experiment("E900/third", {SAME_RUN_PRODUCT_VARIABLE: same_run_product})
        selftest.expect("same-run-field：同一趟的输出里值相等时放行，说明点名那份输出",
                        exit_code == 0 and f"同一趟的输出 {same_run_product} 的 name=verdict control_ok=true" in messages, f"退 {exit_code}，stderr：{messages.strip()[-400:]}")
        exit_code, _output, _messages = experiment("E901")
        selftest.expect("实验准入：没登记的实验键退 2", exit_code == EXIT_REGISTRATION_ERROR, f"退 {exit_code}")
        write_text(table_path, good_table + "E902\tscripts/experiments/e902_nowhere.rs::main\t# 入口文件不在\nE903\tscripts/experiments/e900.rs::main\tbogus=1\t# 认不出的条件\n")
        exit_code, _output, _messages = experiment("E902")
        selftest.expect("实验准入：登记的入口文件不在时退 2", exit_code == EXIT_REGISTRATION_ERROR, f"退 {exit_code}")
        exit_code, _output, _messages = experiment("E903")
        selftest.expect("实验准入：认不出的准入条件退 2", exit_code == EXIT_REGISTRATION_ERROR, f"退 {exit_code}")
        problems = lint_registration_rows(read_registration_rows(work), work)
        selftest.expect("登记表自查点得出入口文件不在、认不出的条件", any("e902_nowhere.rs" in problem for problem in problems) and any("bogus=1" in problem for problem in problems),
                        f"自查结果：{problems}")
        # 前提点名的文件不在仓里（文件头「准入条件」）：question-row 的问题单、product-field 指的留存产物（E901 那一行的产物挪走）
        write_text(table_path, good_table + "E904\t@E900\tquestion-row=research/no-such-questions.md#6:够判\t# 问题单不在\n"
                   "E905\t@E900\tproduct-field=E901:e901:rows=1\t# 指的留存产物挪走了\n")
        stored_e901 = os.path.join(work, RESULTS_DIRECTORY, "e901-2026-09-01.out")
        os.rename(stored_e901, stored_e901 + ".moved")
        problems = lint_registration_rows(read_registration_rows(work), work)
        selftest.expect("登记表自查点得出前提点名的文件不在：question-row 的问题单、product-field 指的留存产物",
                        any("research/no-such-questions.md 不在仓里" in problem for problem in problems)
                        and any("e901-2026-09-01.out 不在仓里" in problem for problem in problems), f"自查结果：{problems}")
        with EnvironmentPatch({BREAK_VARIABLE: "condition-files-unchecked"}):
            problems = lint_registration_rows(read_registration_rows(work), work)
        selftest.expect("弄坏开关 condition-files-unchecked 下「登记表自查点得出前提点名的文件不在」那一格红（一处都不报）",
                        not any("不在仓里" in problem for problem in problems), f"弄坏之后自查结果：{problems}")
        os.rename(stored_e901 + ".moved", stored_e901)
        write_text(table_path, good_table)
        with EnvironmentPatch({"PATH": environment["PATH"], "CARGO_HOME": environment["CARGO_HOME"]}):
            status, _detail = stored_product_row_key_status(work, "E900")
            selftest.expect("给 doc-experiments：产物头的实验键对得上时状态是 matched", status == "matched", f"实际 {status}")
            edit_once(device, "let _ = 1;", "let _ = 3;")
            status, _detail = stored_product_row_key_status(work, "E900")
            selftest.expect("给 doc-experiments：装置变了时状态是 stale", status == "stale", f"实际 {status}")
            status, _detail = stored_product_row_key_status(work, "E900/second")
            selftest.expect("给 doc-experiments：没有一份带这个键的产物时状态是 unfingerprinted", status == "unfingerprinted", f"实际 {status}")
            status, _detail = stored_product_row_key_status(work, "E904")
            selftest.expect("给 doc-experiments：没登记的键状态是 unregistered", status == "unregistered", f"实际 {status}")
    except AssertionError as error:
        selftest.expect("实验准入的样本改得动", False, str(error))
    finally:
        shutil.rmtree(work, ignore_errors=True)


def run_row_key_cells(selftest, module):
    """复跑表的行键（controls L25）：R01 装置函数体变只那一行变；R02 只加注释不变；R03 只改产物文件名不变（弄坏开关 row-key-with-product-name 下转红）；
    R04 共用库里两行都不用的函数变两行都不变；R05 aes-gcm 抬版本只 E140 变；T18 共用库里 E23 用的 Emitter::emit 变只 E23 变；
    E22 切 GPU 开关只点到 CHECKER_TIER_FEATURES 的那一行变（弄坏开关 row-key-without-gpu 下转红）；期望产物内容变那一行变；判法变每一行都变。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-rows-")
    try:
        environment = build_research_sample_repository(work)
        copied = os.path.join(work, "scripts/gate/admission.py")

        def row_keys(arguments=(), changes=None):
            exit_code, output, messages = run_in_environment([sys.executable, copied, "replay-row-inputs", work, *arguments], dict(environment, **(changes or {})))
            keys = {line.split("\t")[0].split("|")[0]: line.split("\t")[1] for line in output.split("\n") if line.count("\t") >= 3}
            return exit_code, keys, output + messages

        exit_code, baseline, output = row_keys(["--gpu", "0"])
        selftest.expect("replay-row-inputs：四行都算得出行键（不拒算）", exit_code == 0 and len(baseline) == 4 and all(re.fullmatch(r"[0-9a-f]{64}", key) for key in baseline.values()),
                        f"退 {exit_code}，{output.strip()[-600:]}")

        def cell(label, path, old, new, changed_rows, arguments=("--gpu", "0"), changes=None):
            if path is not None:
                edit_once(os.path.join(work, path), old, new)
            try:
                _exit_code, after, output = row_keys(arguments, changes)
            finally:
                if path is not None:
                    edit_once(os.path.join(work, path), new, old)
            changed = sorted(row for row in baseline if after.get(row) != baseline[row])
            selftest.expect(f"行键：{label} ⇒ 变的是 {changed_rows or '无'}", changed == sorted(changed_rows), f"实际变的是 {changed or '无'}：{output.strip()[-400:]}")

        cell("R01 E23 装置函数体改一个字", "scripts/experiments/e23_journal_geom.rs", "let start = 0;", "let start = 0u64;", ["E23"])
        cell("R02 E23 装置函数里只加一行注释", "scripts/experiments/e23_journal_geom.rs", "    let start = 0;\n", "    // 只是注释\n    let start = 0;\n", [])
        write_text(os.path.join(work, RESULTS_DIRECTORY, "e23-renamed-2026-09-01.out"), RESEARCH_SAMPLE_FILES[RESULTS_DIRECTORY + "/e23-2026-09-01.out"])
        cell("R03 E23 产物改名（内容相同）", REPLAY_SCRIPT, "|e23-2026-09-01.out|", "|e23-renamed-2026-09-01.out|", [])
        broken = {BREAK_VARIABLE: "row-key-with-product-name"}
        _exit_code, broken_before, _output = row_keys(["--gpu", "0"], broken)
        edit_once(os.path.join(work, REPLAY_SCRIPT), "|e23-2026-09-01.out|", "|e23-renamed-2026-09-01.out|")
        _exit_code, broken_after, _output = row_keys(["--gpu", "0"], broken)
        edit_once(os.path.join(work, REPLAY_SCRIPT), "|e23-renamed-2026-09-01.out|", "|e23-2026-09-01.out|")
        selftest.expect("弄坏开关 row-key-with-product-name 下 R03 那一格转红（只改产物文件名，E23 的行键变了）",
                        broken_before.get("E23") and broken_before.get("E23") != broken_after.get("E23"), f"改名前 {broken_before.get('E23')}，改名后 {broken_after.get('E23')}")
        cell("R04 共用库里两行都不用的 lru_new 改断言消息", "scripts/experiments/lib.rs", "\"容量\"", "\"容量上限\"", [])
        cell("R05 Cargo.lock 里 aes-gcm 抬版本", "scripts/experiments/Cargo.lock", 'version = "0.11.1"', 'version = "0.11.2"', ["E140"])
        cell("T18 共用库里 E23 用到的 Emitter::emit 改一个字", "scripts/experiments/lib.rs", "        1\n", "        2\n", ["E23"])
        cell("E22 切 GPU 开关（--gpu 1）", None, None, None, ["E901"], arguments=("--gpu", "1"))
        broken = {BREAK_VARIABLE: "row-key-without-gpu"}
        _exit_code, gpu_off, _output = row_keys(["--gpu", "0"], broken)
        _exit_code, gpu_on, _output = row_keys(["--gpu", "1"], broken)
        selftest.expect("弄坏开关 row-key-without-gpu 下 E22 那一格转红（切 GPU 开关，点到 CHECKER_TIER_FEATURES 的 E901 行键不变）",
                        gpu_off.get("E901") and gpu_off.get("E901") == gpu_on.get("E901"), f"关 {gpu_off.get('E901')}，开 {gpu_on.get('E901')}")
        cell("期望产物的内容加一行", RESULTS_DIRECTORY + "/e140-2026-09-01.out", "E7RESULT name=done", "E7RESULT name=extra\nE7RESULT name=done", ["E140"])
        cell("驱动 driver_e900 改一个字", REPLAY_SCRIPT, "control_ok=true\\n", "control_ok=true \\n", ["E900"])
        cell("判法 replay_judge_fresh_product 改一个字", REPLAY_SCRIPT, 'diff -q "$1" "$2"', 'diff -q -- "$1" "$2"', ["E140", "E23", "E900", "E901"])
        # 驱动在研究工作区里点名 -p <研究包> --bin <名>（E174 那四个驱动的写法）：行键从 --bin 的名字解到 scripts/experiments 里那份源码，不拒算
        package_line = "  cargo build -q --release --manifest-path scripts/experiments/Cargo.toml -p e7-index-bench --bin e900 >&2 || return 1\n"
        edit_once(os.path.join(work, REPLAY_SCRIPT), "driver_e900() {\n", "driver_e900() {\n" + package_line)
        try:
            exit_code, package_keys, output = row_keys(["--gpu", "0"])
            selftest.expect("行键：驱动点名 -p e7-index-bench --bin e900（研究工作区的包）⇒ E900 算得出行键、不拒算",
                            exit_code == 0 and re.fullmatch(r"[0-9a-f]{64}", package_keys.get("E900", "")) is not None, f"退 {exit_code}，{output.strip()[-400:]}")
            _exit_code, broken_keys, output = row_keys(["--gpu", "0"], {BREAK_VARIABLE: "research-package-unresolved"})
            selftest.expect("弄坏开关 research-package-unresolved 下那一格转红（E900 拒算 replay-package-without-binary）",
                            broken_keys.get("E900") == "refused" and "replay-package-without-binary" in output, f"E900 {broken_keys.get('E900')}：{output.strip()[-400:]}")
        finally:
            edit_once(os.path.join(work, REPLAY_SCRIPT), "driver_e900() {\n" + package_line, "driver_e900() {\n")
    except AssertionError as error:
        selftest.expect("行键的样本改得动", False, str(error))
    finally:
        shutil.rmtree(work, ignore_errors=True)


def crash_blocks_log(blocks, reused, fresh, red, refused, lines=1):
    return "running 1 test\n" + f"CRASH_CASE_BLOCKS blocks={blocks} reused={reused} fresh={fresh} red={red} refused={refused}\n" * lines + PASSED_ONE_LINE + "\n"


def crash_blocks_streams_log(*streams):
    """多流用例的日志：每条流一行 CRASH_CASE_BLOCKS，streams 每项是 (blocks, reused, fresh, red, refused)。"""
    return ("running 1 test\n" + "".join(f"CRASH_CASE_BLOCKS blocks={blocks} reused={reused} fresh={fresh} red={red} refused={refused}\n"
                                         for blocks, reused, fresh, red, refused in streams) + PASSED_ONE_LINE + "\n")


def run_crash_case_blocks_cells(selftest, module):
    """CRASH_CASE_BLOCKS 行（文件头「崩溃枚举用例的复用不在这里」）：查询下全部块有判定且无红退 0、有块没判定退 3、有红退 1；平常跑无红退 0；
    多流用例每条流一行、逐行收按用例合计；一行都没有、有一行数对不上（blocks = reused + fresh + refused）退 4
    （弄坏开关 blocks-count-unchecked、blocks-count-without-refused、blocks-single-line-only 下各自那一格转红）；crates 侧源码里没有查询那个变量名退 5。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-blocks-")
    try:
        write_text(os.path.join(work, CRASH_CASE_BLOCKS_SOURCE_DIRECTORY, "lib.rs"), "// 样本：还没接块查询\n")
        log_path = os.path.join(work, "case.log")

        def blocks(log_text, query, changes=None):
            write_text(log_path, log_text)
            arguments = [sys.executable, module, "crash-case-blocks", work, log_path] + (["--query"] if query else [])
            return run_quietly(arguments, changes)[:2]
        exit_code, output = blocks(crash_blocks_log(4, 4, 0, 0, 0), True)
        selftest.expect("crash-case-blocks：crates 侧源码里没有 SINGLEFS_CRASH_CASE_BLOCKS_QUERY ⇒ 退 5（还没接上）", exit_code == EXIT_BLOCKS_UNSUPPORTED, f"退 {exit_code}：{output.strip()}")
        write_text(os.path.join(work, CRASH_CASE_BLOCKS_SOURCE_DIRECTORY, "lib.rs"), f'pub const QUERY: &str = "{CRASH_CASE_BLOCKS_QUERY_VARIABLE}";\n')
        cells = [("查询：4 块都有判定、无红", crash_blocks_log(4, 4, 0, 0, 0), True, 0),
                 ("查询：1 块没有判定", crash_blocks_log(4, 3, 1, 0, 0), True, EXIT_BLOCKS_NOT_COVERED),
                 ("查询：1 块拒算", crash_blocks_log(4, 3, 0, 0, 1), True, EXIT_BLOCKS_NOT_COVERED),
                 ("查询：1 块判红", crash_blocks_log(4, 4, 0, 1, 0), True, EXIT_BLOCKS_RED),
                 ("平常跑：复用 3 块、现判 1 块、无红", crash_blocks_log(4, 3, 1, 0, 0), False, 0),
                 ("平常跑：现判的块里 1 块红", crash_blocks_log(4, 3, 1, 1, 0), False, EXIT_BLOCKS_RED),
                 ("日志里没有那一行", "running 1 test\n" + PASSED_ONE_LINE + "\n", False, EXIT_BLOCKS_LINE_BAD),
                 ("平常跑：拒算 1 块计在 refused、数对得上", crash_blocks_log(4, 3, 0, 0, 1), False, 0),
                 ("多流用例每条流一行：逐行收、按用例合计", crash_blocks_streams_log((3, 3, 0, 0, 0), (2, 1, 1, 0, 0)), False, 0),
                 ("多流用例查询：两行合计全有判定", crash_blocks_streams_log((3, 3, 0, 0, 0), (2, 2, 0, 0, 0)), True, 0),
                 ("多流用例里一条流有块没有判定", crash_blocks_streams_log((3, 3, 0, 0, 0), (2, 1, 1, 0, 0)), True, EXIT_BLOCKS_NOT_COVERED),
                 ("多流用例里一条流有红", crash_blocks_streams_log((3, 3, 0, 0, 0), (2, 1, 1, 1, 0)), False, EXIT_BLOCKS_RED),
                 ("多流用例里一行数对不上", crash_blocks_streams_log((3, 3, 0, 0, 0), (3, 1, 1, 0, 0)), False, EXIT_BLOCKS_LINE_BAD),
                 ("blocks 不等于 reused + fresh + refused", crash_blocks_log(5, 3, 1, 0, 0), False, EXIT_BLOCKS_LINE_BAD)]
        for label, log_text, query, expected in cells:
            exit_code, output = blocks(log_text, query)
            selftest.expect(f"crash-case-blocks：{label} ⇒ 退 {expected}", exit_code == expected, f"退 {exit_code}：{output.strip()}")
        exit_code, output = blocks(crash_blocks_streams_log((3, 3, 0, 0, 0), (2, 1, 1, 0, 0)), False)
        selftest.expect("crash-case-blocks：多流用例的一句报两条流合计的块数（块 5、复用 4、现判 1）",
                        "块 5（2 条流合计）：复用 4、现判 1" in output, output.strip())
        exit_code, output = blocks(crash_blocks_log(5, 3, 1, 0, 0), False, {BREAK_VARIABLE: "blocks-count-unchecked"})
        selftest.expect("弄坏开关 blocks-count-unchecked 下「blocks 不等于 reused + fresh + refused」那一格转红（退 0）", exit_code == 0, f"退 {exit_code}：{output.strip()}")
        exit_code, output = blocks(crash_blocks_log(4, 3, 0, 0, 1), False, {BREAK_VARIABLE: "blocks-count-without-refused"})
        selftest.expect("弄坏开关 blocks-count-without-refused 下「平常跑：拒算 1 块计在 refused」那一格转红（退 4）", exit_code == EXIT_BLOCKS_LINE_BAD,
                        f"退 {exit_code}：{output.strip()}")
        exit_code, output = blocks(crash_blocks_streams_log((3, 3, 0, 0, 0), (2, 1, 1, 0, 0)), False, {BREAK_VARIABLE: "blocks-single-line-only"})
        selftest.expect("弄坏开关 blocks-single-line-only 下「多流用例每条流一行」那一格转红（退 4）", exit_code == EXIT_BLOCKS_LINE_BAD,
                        f"退 {exit_code}：{output.strip()}")

        # --ran（快档实跑与 --full 跑登记了 red-blocks=judged-by-test 的用例）：只核行在、数对得上，红块与现判块不判
        def ran_blocks(log_text, extra=(), changes=None):
            write_text(log_path, log_text)
            return run_quietly([sys.executable, module, "crash-case-blocks", work, log_path, "--ran", *extra], changes)[:2]
        ran_cells = [("--ran：4 块里现判 3、红 2（用例认下的红）", crash_blocks_log(4, 1, 3, 2, 0), 0),
                     ("--ran：日志里没有那一行", "running 1 test\n" + PASSED_ONE_LINE + "\n", EXIT_BLOCKS_LINE_BAD),
                     ("--ran：blocks 不等于 reused + fresh + refused", crash_blocks_log(5, 1, 3, 2, 0), EXIT_BLOCKS_LINE_BAD),
                     ("--ran：多流用例两行合计", crash_blocks_streams_log((3, 3, 0, 1, 0), (2, 1, 1, 1, 0)), 0)]
        for label, log_text, expected in ran_cells:
            exit_code, output = ran_blocks(log_text)
            selftest.expect(f"crash-case-blocks：{label} ⇒ 退 {expected}", exit_code == expected, f"退 {exit_code}：{output.strip()}")
        exit_code, output = ran_blocks(crash_blocks_log(4, 1, 3, 2, 0))
        selftest.expect("crash-case-blocks --ran：一句照平常跑的写法报块数（块 4：复用 1、现判 3、红 2、拒算 0）",
                        output.strip() == "块 4：复用 1、现判 3、红 2、拒算 0", output.strip())
        exit_code, output = ran_blocks(crash_blocks_log(4, 4, 0, 0, 0), ("--query",))
        selftest.expect("crash-case-blocks：--query 与 --ran 同给 ⇒ 退 2", exit_code == EXIT_REGISTRATION_ERROR, f"退 {exit_code}：{output.strip()}")
        exit_code, output = ran_blocks("running 1 test\n" + PASSED_ONE_LINE + "\n", changes={BREAK_VARIABLE: "ran-blocks-line-optional"})
        selftest.expect("弄坏开关 ran-blocks-line-optional 下「--ran：日志里没有那一行」那一格转红（退 0）", exit_code == 0, f"退 {exit_code}：{output.strip()}")
    finally:
        shutil.rmtree(work, ignore_errors=True)


class Selftest:
    def __init__(self):
        self.failures = 0
        self.checked = 0
        self.names = []

    def expect(self, name, condition, detail):
        self.checked += 1
        self.names.append(name)
        if condition:
            print(f"  ✓ {name}")
        else:
            print(f"  ✗ {name}：{detail}")  # gate-lint:detail
            self.failures += 1


def run_quietly(command, environment_changes=None, working_directory=None):
    """外面设的 ADMISSION_BREAK 照样传给子进程：`ADMISSION_BREAK=skip-unchanged admission.py --selftest` 要看到「输入没变」那几格红。"""
    environment = dict(os.environ)
    for variable in (FORCE_RERUN_VARIABLE, "SINGLEFS_GATE_FULL", "SINGLEFS_STAGED_TREE"):
        environment.pop(variable, None)
    environment.update(environment_changes or {})
    completed = subprocess.run(command, capture_output=True, text=True, errors="replace", env=environment, cwd=working_directory)
    return completed.returncode, completed.stdout, completed.stderr


def write_text(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


def write_questions(work, row_six_status):
    write_text(os.path.join(work, "research/questions.md"),
               "| # | 问题 | 状态 |\n|---|---|---|\n"
               "| 5 | 别的 | **够判：能** |\n"
               f"| 6 | 阳性对照是好的还是坏的 | {row_six_status} |\n")


def point_replay_row_at(work, product_name):
    write_text(os.path.join(work, REPLAY_SCRIPT), f"TABLE=$(cat <<'TSV'\nE900|@driver_e900||{product_name}|exact\nTSV\n)\n")


GIT_IDENTITY = {"GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@t", "GIT_COMMITTER_NAME": "t", "GIT_COMMITTER_EMAIL": "t@t"}


def write_executable(path, text):
    write_text(path, text)
    os.chmod(path, 0o755)


def staged_tree_of(work):
    run_quietly(["git", "-C", work, "add", "-A"], GIT_IDENTITY)
    _exit_code, output, _messages = run_quietly(["git", "-C", work, "write-tree"])
    return output.strip()


def mark_tree_green(work, tree):
    """照 scripts/gate/gate-staged.sh 的做法：refs/sop/staged-green 指一个包着那棵暂存树的提交。"""
    _exit_code, commit, _messages = run_quietly(["git", "-C", work, "commit-tree", tree, "-m", "green"], GIT_IDENTITY)
    run_quietly(["git", "-C", work, "update-ref", GATE_GREEN_TREE_RECORD, commit.strip()])


def run_gate_precondition_cells(selftest, module):
    """门禁行的前提：齐了一个字不打；command= / readwrite= / probe= 各缺一样判红并点名；没登记退 2；种类写串了自查点得出。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-gate-")
    tools = tempfile.mkdtemp(prefix="admission-selftest-tools-")
    try:
        run_quietly(["git", "init", "-q", "-b", "master", work], GIT_IDENTITY)
        present_tool = os.path.join(tools, "selftest-present-tool")
        write_executable(present_tool, "#!/usr/bin/env bash\nexit 0\n")
        device = os.path.join(work, "device.bin")
        kernel = os.path.join(work, "kernel.img")
        write_text(device, "")
        write_text(kernel, "")
        write_text(os.path.join(work, "tools/check.sh"),
                   'if [[ "${1:-}" == --check && -f kernel.img ]]; then exit 0; fi\necho "  ✗ 找不到内核镜像（样本）"\nexit 1\n')
        write_text(os.path.join(work, REGISTRATION_TABLE),
                   "55-demo.sh\tcrates/\tcommand=selftest-present-tool readwrite=device.bin probe=tools/check.sh:--check\t# 样本：三个前置\n")

        def preconditions(stage="55-demo.sh", extra_environment=None):
            changes = {"PATH": tools + os.pathsep + os.environ.get("PATH", "")}
            changes.update(extra_environment or {})
            return run_quietly([sys.executable, module, "gate-preconditions", work, stage], changes)

        exit_code, output, messages = preconditions()
        selftest.expect("门禁前提都齐时退 0、stdout 一个字不打", exit_code == 0 and output == "",
                        f"退 {exit_code}，stdout「{output.strip()}」，stderr：{messages.strip()}")
        os.remove(present_tool)
        exit_code, output, _messages = preconditions()
        selftest.expect("门禁前提 command= 缺了判红（退 1）：点名那个可执行名，带出路与那一行的注释",
                        exit_code == EXIT_GATE_PRECONDITION_MISSING and "✗ 前提没齐" in output and "selftest-present-tool 不在 PATH 里" in output
                        and "→ 怎么办" in output and "样本：三个前置" in output, f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, _output, _messages = preconditions(extra_environment={BREAK_VARIABLE: "skip-gate-preconditions"})
        selftest.expect("弄坏开关 skip-gate-preconditions 下「前提没齐」那一格红（放行了）", exit_code == 0,
                        f"弄坏之后仍退 {exit_code}：这一格判不出前提被跳过")
        write_executable(present_tool, "#!/usr/bin/env bash\nexit 0\n")
        os.chmod(device, 0o444)
        exit_code, output, _messages = preconditions()
        selftest.expect("门禁前提 readwrite= 不可写判红（退 1）并点名那个路径（以 root 跑这一格判不出）",
                        exit_code == EXIT_GATE_PRECONDITION_MISSING and "device.bin 不在，或当前用户不可读写" in output,
                        f"退 {exit_code}，stdout「{output.strip()}」")
        os.chmod(device, 0o644)
        os.remove(kernel)
        exit_code, output, _messages = preconditions()
        selftest.expect("门禁前提 probe= 退非 0 判红（退 1），明细带出探针自己打的那几行",
                        exit_code == EXIT_GATE_PRECONDITION_MISSING and "tools/check.sh --check 退 1" in output and "找不到内核镜像（样本）" in output,
                        f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, output, _messages = preconditions(stage="99-unregistered.sh")
        selftest.expect("登记表里没有那一道时退 2，照样打 ✗ 与出路", exit_code == EXIT_REGISTRATION_ERROR and "✗" in output and "→ 怎么办" in output,
                        f"退 {exit_code}，stdout「{output.strip()}」")
        mixed_rows = [RegistrationRow("55-demo.sh", ["crates/"], ["question-row=a.md#1:x"], 1, ""),
                      RegistrationRow("E950", ["crates/"], ["command=cargo"], 2, ""),
                      RegistrationRow("57-demo.sh", ["crates/"], ["probe=tools/nowhere.sh"], 3, "")]
        problems = lint_registration_rows(mixed_rows, work)
        selftest.expect("自查点得出门禁行写实验的种类、实验行写门禁的种类、probe= 指的脚本不在",
                        any(problem.startswith("第 1 行") and "认不出的门禁准入条件" in problem for problem in problems)
                        and any(problem.startswith("第 2 行") and "认不出的准入条件" in problem for problem in problems)
                        and any(problem.startswith("第 3 行") and "tools/nowhere.sh 不在" in problem for problem in problems),
                        f"自查结果：{problems}")
    finally:
        shutil.rmtree(work, ignore_errors=True)
        shutil.rmtree(tools, ignore_errors=True)


FAKE_HERD7_SCRIPT = '#!/usr/bin/env bash\nif [[ "${1:-}" == -version ]]; then echo "herd7 version 7.57-selftest-%s"; exit 0; fi\nexit 2\n'



def environment_without_build_settings(changes=None):
    """子进程的环境：去掉会进构建环境那一格的变量（CARGO_HOME、RUSTC 与名字对得上的那几类）与握手用的几个，再套上 changes。"""
    environment = {name: value for name, value in os.environ.items()
                   if not (name in BUILD_ENVIRONMENT_EXACT_VARIABLES or name in ("RUSTC", "CARGO_HOME")
                           or any(form.match(name) for form in BUILD_ENVIRONMENT_VARIABLE_FORMS))}
    for variable in (FORCE_RERUN_VARIABLE, "SINGLEFS_GATE_FULL", "SINGLEFS_STAGED_TREE"):
        environment.pop(variable, None)
    environment.update(changes or {})
    return environment


def run_in_environment(command, environment, working_directory=None):
    completed = subprocess.run(command, capture_output=True, text=True, errors="replace", env=environment, cwd=working_directory)
    return completed.returncode, completed.stdout, completed.stderr


class BreakSwitch:
    """在本进程里临时设 ADMISSION_BREAK（给纯函数那几格证明会红），退出时照原样还回去。"""
    def __init__(self, switch_name):
        self.switch_name = switch_name
        self.saved = None

    def __enter__(self):
        self.saved = os.environ.get(BREAK_VARIABLE)
        os.environ[BREAK_VARIABLE] = self.switch_name
        return self

    def __exit__(self, *_exception):
        if self.saved is None:
            os.environ.pop(BREAK_VARIABLE, None)
        else:
            os.environ[BREAK_VARIABLE] = self.saved
        return False


FAKE_TOOLCHAIN_SCRIPTS = fake_toolchain_scripts("/nonexistent/fake-rustc-version")



def build_crash_case_repository(work, registration_rows):
    """一个临时小仓：包 pkg 里有用例 own_case（mod common、mod helper_target）、被注释提到的 other_target、被字符串点名的 string_named。"""
    run_quietly(["git", "init", "-q", "-b", "master", work], GIT_IDENTITY)
    write_text(os.path.join(work, "Cargo.toml"), '[workspace]\nmembers = ["crates/pkg"]\n')
    write_text(os.path.join(work, "crates/pkg/Cargo.toml"), '[package]\nname = "pkg"\nversion = "0.0.0"\n')
    write_text(os.path.join(work, "crates/pkg/src/lib.rs"),
               '//! 注释里提到 other_target 不算\npub const PATH: &str = "tests/string_named.rs";\npub fn one() -> u32 { 1 }\n')
    write_text(os.path.join(work, "crates/pkg/tests/own_case.rs"),
               'mod common;\nmod helper_target;\n#[test]\n#[ignore = "崩溃枚举（样本）"]\nfn the_case() {}\n')
    write_text(os.path.join(work, "crates/pkg/tests/common/mod.rs"), "pub fn shared() {}\n")
    write_text(os.path.join(work, "crates/pkg/tests/helper_target.rs"), "pub fn helper() {}\n")
    write_text(os.path.join(work, "crates/pkg/tests/other_target.rs"), "#[test]\nfn other() {}\n")
    write_text(os.path.join(work, "crates/pkg/tests/string_named.rs"), "#[test]\nfn named() {}\n")
    write_text(os.path.join(work, REGISTRATION_TABLE), "".join(row + "\n" for row in registration_rows))


def run_crash_cases_listing_cells(selftest, module):
    """crash-cases：逐条打键、包、测试目标、用例函数；登记行自查（写法、第二列写 -、用例函数找不到、目标不在、包不在工作区、test= 两条、同键两行）；
    用例函数的每一处定义都要标 #[ignore]（cfg 二选一、子模块里同名、# [ignore]、顺着 include! / #[path] / mod 带进来的源文件）。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-crash-case-")
    try:
        own_row = "crash-case:own\t-\ttest=pkg:own_case:the_case count-line=CRASH_POINT_REPLAY exhaustive=CRASH_POINT_REPLAY\t# 样本"
        build_crash_case_repository(work, [own_row])
        table_path = os.path.join(work, REGISTRATION_TABLE)

        def admission(*arguments, changes=None):
            return run_quietly([sys.executable, module, *arguments], changes)

        exit_code, output, _messages = admission("crash-cases", work)
        selftest.expect("crash-cases 逐条打「键、包、测试目标、用例函数」", exit_code == 0 and output == "crash-case:own\tpkg\town_case\tthe_case\n",
                        f"退 {exit_code}，stdout「{output.strip()}」")
        lint_rows = [
            ("用例函数在测试目标里找不到", "crash-case:own\t-\ttest=pkg:own_case:no_such_function\t# 样本", "找不到"),
            ("测试目标不在", "crash-case:own\t-\ttest=pkg:no_such_target:the_case\t# 样本", "不在"),
            ("包不是工作区成员", "crash-case:own\t-\ttest=nowhere:own_case:the_case\t# 样本", "不是仓根 Cargo.toml 的工作区成员"),
            ("test= 写了两条", "crash-case:own\t-\ttest=pkg:own_case:the_case test=pkg:own_case:the_case\t# 样本", "恰好一条 test="),
            ("exhaustive= 点名的前缀没登记成 count-line=", "crash-case:own\t-\ttest=pkg:own_case:the_case exhaustive=CRASH_POINT_REPLAY\t# 样本", "没有登记成 count-line="),
            ("认不出的条件", "crash-case:own\t-\ttest=pkg:own_case:the_case command=cargo\t# 样本", "认不出"),
            ("同一个键两行", own_row + "\n" + own_row, "一条用例只许一行"),
            ("第二列写了路径（用例一级不按路径登记输入）", "crash-case:own\tcrates/ Cargo.toml\ttest=pkg:own_case:the_case\t# 样本", "第二列写 -"),
            ("标记名写错（red-block= 少一个 s）", "crash-case:own\t-\ttest=pkg:own_case:the_case red-block=judged-by-test\t# 样本", "认不出"),
            ("red-blocks= 写了 judged-by-test 之外的值", "crash-case:own\t-\ttest=pkg:own_case:the_case red-blocks=judged-by-library\t# 样本",
             "red-blocks= 只认 judged-by-test"),
            ("red-blocks= 写了两条", "crash-case:own\t-\ttest=pkg:own_case:the_case red-blocks=judged-by-test red-blocks=judged-by-test\t# 样本",
             "red-blocks= 至多一条"),
        ]
        for label, rows_text, expected_phrase in lint_rows:
            write_text(table_path, rows_text + "\n")
            exit_code, output, _messages = admission("crash-cases", work)
            selftest.expect(f"crash-cases 自查：{label}，退 2 并说出来", exit_code == EXIT_REGISTRATION_ERROR and expected_phrase in output,
                            f"退 {exit_code}，stdout「{output.strip()}」")
        # red-blocks=judged-by-test（文件头「崩溃枚举用例行」）：带标记的行 crash-cases 照常列出，crash-case-judged-by-test 对它退 0、对没带的退 1
        marked_row = "crash-case:marked\t-\ttest=pkg:own_case:the_case count-line=CRASH_POINT_REPLAY red-blocks=judged-by-test\t# 样本"
        write_text(table_path, own_row + "\n" + marked_row + "\n")
        exit_code, output, _messages = admission("crash-cases", work)
        selftest.expect("crash-cases：带 red-blocks=judged-by-test 的行解析得出、照常逐条列出",
                        exit_code == 0 and output == "crash-case:own\tpkg\town_case\tthe_case\ncrash-case:marked\tpkg\town_case\tthe_case\n",
                        f"退 {exit_code}，stdout「{output.strip()}」")
        marked_case = crash_case_of_key(read_registration_rows(work), "crash-case:marked")
        plain_case = crash_case_of_key(read_registration_rows(work), "crash-case:own")
        selftest.expect("parse_crash_case：带标记的行 red_blocks_judged_by_test 为真、没带的为假",
                        marked_case.red_blocks_judged_by_test and not plain_case.red_blocks_judged_by_test,
                        f"带标记的 {marked_case.red_blocks_judged_by_test}，没带的 {plain_case.red_blocks_judged_by_test}")
        for key, expected in (("crash-case:marked", 0), ("crash-case:own", 1), ("crash-case:missing", EXIT_REGISTRATION_ERROR)):
            exit_code, output, _messages = admission("crash-case-judged-by-test", work, key)
            selftest.expect(f"crash-case-judged-by-test {key} ⇒ 退 {expected}", exit_code == expected, f"退 {exit_code}，stdout「{output.strip()}」")
        write_text(table_path, own_row + "\n")

        # 登记的用例函数要标 #[ignore]：没标的、只有 cfg_attr 里条件 ignore 的、ignore 挂在前一个函数上的、只写在注释里的，crash-cases 退 2
        own_case_path = os.path.join(work, "crates/pkg/tests/own_case.rs")
        with open(own_case_path, encoding="utf-8") as handle:
            own_case_text = handle.read()
        modules = "mod common;\nmod helper_target;\n"
        cfg_split_case = "#[cfg(debug_assertions)]\n#[test]\n#[ignore]\nfn the_case() {}\n#[cfg(not(debug_assertions))]\n#[test]\nfn the_case() {}\n"
        submodule_first_case = "mod slow {\n    #[test]\n    #[ignore]\n    fn the_case() {}\n}\n#[test]\nfn the_case() {}\n"
        spaced_ignore_case = "#[test]\n# [ignore]\nfn the_case() {}\n"
        ignore_cells = [
            ("用例函数没标 #[ignore]", modules + "#[test]\nfn the_case() {}\n", False),
            ("只有 cfg_attr 里的条件 ignore", modules + "#[test]\n#[cfg_attr(miri, ignore)]\nfn the_case() {}\n", False),
            ("#[ignore] 挂在前一个函数上", modules + "#[test]\n#[ignore]\nfn other() {}\n#[test]\nfn the_case() {}\n", False),
            ("#[ignore] 只写在注释里", modules + "#[test]\n// #[ignore]\nfn the_case() {}\n", False),
            ("#[ignore] 在 #[test] 前面、fn 前面有 pub", modules + "#[ignore]\n#[test]\npub fn the_case() {}\n", True),
            ("ignore 的理由里带方括号、引号与 //", modules + '#[test]\n#[ignore = "样本 [x] \\"y\\" // z"]\nfn the_case() {}\n', True),
            ("同名函数 cfg 二选一：debug 那一份标 ignore、release 那一份不标", modules + cfg_split_case, False),
            ("子模块里同名标 ignore 的写在前面、顶层那一份（--exact the_case 跑的）不标", modules + submodule_first_case, False),
            ("# [ignore]（# 与 [ 之间有空格，rustc 认）", modules + spaced_ignore_case, True),
            ("同名的两处都标了 ignore（cfg 二选一）", modules + cfg_split_case.replace("#[test]\nfn the_case", "#[test]\n#[ignore]\nfn the_case"), True),
        ]
        for label, source_text, marked in ignore_cells:
            write_text(own_case_path, source_text)
            exit_code, output, _messages = admission("crash-cases", work)
            green = exit_code == 0 if marked else (exit_code == EXIT_REGISTRATION_ERROR and "没标 #[ignore]" in output)
            selftest.expect(f"crash-cases 自查：{label} ⇒ {'退 0' if marked else '退 2 并说没标 #[ignore]'}", green,
                            f"退 {exit_code}，stdout「{output.strip()}」")
        for switch, label, source_text, broken_exit_code in (
                ("first-definition-only", "同名函数 cfg 二选一、一份不标", modules + cfg_split_case, 0),
                ("first-definition-only", "子模块里同名标 ignore 的写在前面、顶层那一份不标", modules + submodule_first_case, 0),
                ("ignore-attribute-without-space", "# [ignore]", modules + spaced_ignore_case, EXIT_REGISTRATION_ERROR)):
            write_text(own_case_path, source_text)
            exit_code, output, _messages = admission("crash-cases", work, changes={BREAK_VARIABLE: switch})
            selftest.expect(f"弄坏开关 {switch} 下「{label}」那一格红（crash-cases 退 {broken_exit_code}）", exit_code == broken_exit_code,
                            f"弄坏之后退 {exit_code}：这一格分不出差别；stdout「{output.strip()}」")
        # 顺着 include!("<字面路径>") / #[path = "…"] mod / mod <名>; 带进来的源文件一起判；判不出的按没标算（弄坏开关 target-own-files-only 下头一格红）
        marked_here = "#[test]\n#[ignore]\nfn the_case() {}\n"
        release_unmarked = {"crates/pkg/tests/common/release_case.rs": "#[test]\nfn the_case() {}\n"}
        cfg_include = ("#[cfg(debug_assertions)]\n" + marked_here + "#[cfg(not(debug_assertions))]\ninclude!(\"common/release_case.rs\");\n")
        included_cells = [
            ("debug 下标 ignore、release 下 include!(\"common/release_case.rs\") 进来一份不标的同名用例", modules + cfg_include, release_unmarked,
             "没标 #[ignore]"),
            ("include! 的参数是 concat!(…)（读的是哪一份判不出）", modules + marked_here + 'include!(concat!("common/", "release_case.rs"));\n',
             release_unmarked, "判不出"),
            ("#[path = \"common/p.rs\"] mod p; 里有一份不标的同名用例", modules + marked_here + '#[path = "common/p.rs"]\nmod p;\n',
             {"crates/pkg/tests/common/p.rs": "#[test]\nfn the_case() {}\n"}, "没标 #[ignore]"),
            ("mod 指的文件找不到", modules + marked_here + "mod missing_module;\n", {}, "判不出"),
            ("use core::include as pull; 把 include! 改名引进来", modules + "use core::include as pull;\n" + marked_here, {}, "判不出"),
            ("对照：include! 进来的同名用例也标了 ignore", modules + cfg_include.replace("release_case.rs", "marked_case.rs"),
             {"crates/pkg/tests/common/marked_case.rs": marked_here}, None),
        ]
        broken_exit_code, broken_output = None, ""
        for label, source_text, extra_files, phrase in included_cells:
            write_text(own_case_path, source_text)
            for relative, content in extra_files.items():
                write_text(os.path.join(work, relative), content)
            exit_code, output, _messages = admission("crash-cases", work)
            if label == included_cells[0][0]:
                broken_exit_code, broken_output, _messages = admission("crash-cases", work, changes={BREAK_VARIABLE: "target-own-files-only"})
            for relative in extra_files:
                os.remove(os.path.join(work, relative))
            green = exit_code == 0 if phrase is None else (exit_code == EXIT_REGISTRATION_ERROR and phrase in output)
            selftest.expect(f"crash-cases 自查（带进来的源文件）：{label} ⇒ {'退 0' if phrase is None else '退 2 并说' + phrase}", green,
                            f"退 {exit_code}，stdout「{output.strip()}」")
        selftest.expect("弄坏开关 target-own-files-only 下「release 下 include! 进来一份不标的同名用例」那一格红（crash-cases 退 0）", broken_exit_code == 0,
                        f"弄坏之后退 {broken_exit_code}：这一格分不出差别；stdout「{broken_output.strip()}」")
        write_text(own_case_path, own_case_text)
        exit_code, output, _messages = admission("crash-case-command", work, "crash-case:own", "--start-over", "--blocks-query")
        words = output.split("\0")
        selftest.expect("crash-case-command：--start-over 设只写不读、--blocks-query 设块查询，两样都在 cargo 之前设进环境、库目录变量照样清掉",
                        exit_code == 0 and f"{CRASH_LIBRARY_WRITE_ONLY_VARIABLE}=1" in words and f"{CRASH_CASE_BLOCKS_QUERY_VARIABLE}=1" in words
                        and words.index(f"{CRASH_LIBRARY_WRITE_ONLY_VARIABLE}=1") < words.index("cargo") and CRASH_LIBRARY_DIRECTORY_VARIABLE in words,
                        f"退 {exit_code}，交的词 {words}")
        exit_code, output, _messages = admission("crash-case-command", work, "crash-case:own")
        words = output.split("\0")
        selftest.expect("crash-case-command：不带 --start-over 与 --blocks-query ⇒ 两个变量都从调用方环境里清掉、不设",
                        exit_code == 0 and f"{CRASH_LIBRARY_WRITE_ONLY_VARIABLE}=1" not in words and f"{CRASH_CASE_BLOCKS_QUERY_VARIABLE}=1" not in words
                        and CRASH_LIBRARY_WRITE_ONLY_VARIABLE in words, f"退 {exit_code}，交的词 {words}")
        run_crash_case_command_library_cells(selftest, work)
    finally:
        shutil.rmtree(work, ignore_errors=True)

SAMPLE_MACHINE_CORES = 32


def selftest_default_threads():
    """自证里当「每台默认重活线程数」用的数：本仓登记的那一个（不认调用方的 SINGLEFS_THREAD_CAP；判日志的纯函数那几格显式传它）；读不出交 None。"""
    try:
        return registered_heavy_threads({})
    except InputManifestError:
        return None


def synthetic_crash_point_replay_log(states=100, slices=64, worker_threads=None, resumed_slices=0, freshly_run_slices=64, exhaustive="true",
                         result="test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 1.00s",
                         crash_point_replay_lines=1, with_checker=True, with_finished=True, finished_fields=None, configured_worker_threads=None,
                         worker_threads_source="environment_variable"):
    """合成的一趟 --full 日志：CRASH_POINT_REPLAY_PARALLEL_FINISHED、CRASH_POINT_REPLAY 计数行、CHECKER 行、test result 各几行由参数定。
    起的与配的线程数不给都取每台默认重活线程数（selftest_default_threads）。"""
    default_threads = selftest_default_threads() or 1
    worker_threads = default_threads if worker_threads is None else worker_threads
    configured_worker_threads = default_threads if configured_worker_threads is None else configured_worker_threads
    lines = [f"CRASH_POINT_REPLAY_PARALLEL_START states={states} slices={slices}"]
    if with_finished:
        fields = finished_fields or (f"worker_threads={worker_threads} configured_worker_threads={configured_worker_threads} "
                                     f"worker_threads_source={worker_threads_source} "
                                     f"resumed_slices={resumed_slices} freshly_run_slices={freshly_run_slices}")
        lines.append(f"CRASH_POINT_REPLAY_PARALLEL_FINISHED states={states} slices={slices} {fields} progress_file_after_completion=deleted elapsed_seconds=1.0")
    lines += [f"CRASH_POINT_REPLAY states={states} closed_form={states} violations=0 exhaustive={exhaustive}"] * crash_point_replay_lines
    if with_checker:
        lines.append("CHECKER record_root_without_record=0 record_claimed_state_missing_unit=0")
    if result is not None:
        lines.append(result)
    return "\n".join(lines) + "\n"


def run_crash_case_log_cells(selftest):
    """--full 跑一条用例的日志怎么判（纯函数，合成日志）：test result 恰好 1 passed、计数行恰好一行、exhaustive=true、工作线程按这一趟跑了几片判。"""
    case = parse_crash_case(RegistrationRow("crash-case:demo", [], ["test=pkg:target:function", "count-line=CRASH_POINT_REPLAY",
                                                                             "count-line=CHECKER", "exhaustive=CRASH_POINT_REPLAY", "threads=CRASH_POINT_REPLAY"], 1, ""))
    default = selftest_default_threads()
    selftest.expect("每台默认重活线程数：本仓的登记读得出（判日志那几格的线程数从它来）", default is not None, f"{HEAVY_THREAD_DEFAULT_FILE} 读不出")
    default = default or 1
    cores = SAMPLE_MACHINE_CORES
    cells = [
        (f"全量、{default} 个线程跑了 64 片", synthetic_crash_point_replay_log(), cores, False, True),
        ("全部 64 片从进度文件读回、起 0 个线程", synthetic_crash_point_replay_log(worker_threads=0, resumed_slices=64, freshly_run_slices=0), cores, False, True),
        ("读回 63 片、只剩 1 片要跑、起 1 个线程", synthetic_crash_point_replay_log(worker_threads=1, resumed_slices=63, freshly_run_slices=1), cores, False, True),
        (f"跑了 64 片却只起 1 个线程（本机 {cores} 核、线程数没显式设）", synthetic_crash_point_replay_log(worker_threads=1), cores, False, False),
        ("跑了 64 片只起 1 个线程，线程数显式设成 1", synthetic_crash_point_replay_log(worker_threads=1), cores, True, True),
        ("跑了 64 片只起 1 个线程，本机只有 1 核", synthetic_crash_point_replay_log(worker_threads=1), 1, False, True),
        ("跑了 5 片却报起了 0 个线程", synthetic_crash_point_replay_log(worker_threads=0, resumed_slices=59, freshly_run_slices=5), cores, False, False),
        ("读回的片 + 这一趟跑的片 ≠ 总片数", synthetic_crash_point_replay_log(resumed_slices=10, freshly_run_slices=10), cores, False, False),
        ("CRASH_POINT_REPLAY_PARALLEL_FINISHED 缺续跑片数（旧格式）", synthetic_crash_point_replay_log(finished_fields=f"worker_threads={default}"), cores, False, False),
        ("没有状态数对得上的 CRASH_POINT_REPLAY_PARALLEL_FINISHED", synthetic_crash_point_replay_log(with_finished=False), cores, False, False),
        ("过滤之后一条用例都没跑（0 passed）",
         synthetic_crash_point_replay_log(result="test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s"), cores, False, False),
        ("没有 test result 行", synthetic_crash_point_replay_log(result=None), cores, False, False),
        ("不是全量（exhaustive=false）", synthetic_crash_point_replay_log(exhaustive="false"), cores, False, False),
        ("CRASH_POINT_REPLAY 计数行抄了两遍", synthetic_crash_point_replay_log(crash_point_replay_lines=2), cores, False, False),
        ("没有 CHECKER 行", synthetic_crash_point_replay_log(with_checker=False), cores, False, False),
        (f"线程数没显式设（来源 available_parallelism）、用例取的默认等于每台默认重活线程数 {default}",
         synthetic_crash_point_replay_log(worker_threads_source="available_parallelism"), cores, False, True),
        (f"线程数没显式设（来源 available_parallelism）、用例按核数取了 {default + 1}，不等于每台默认重活线程数 {default}",
         synthetic_crash_point_replay_log(worker_threads=default + 1, configured_worker_threads=default + 1, worker_threads_source="available_parallelism"), cores, False, False),
        (f"线程数显式设成 {default + 1}（来源 environment_variable），不按每台默认判", synthetic_crash_point_replay_log(worker_threads=default + 1,
         configured_worker_threads=default + 1), cores, False, True),
    ]
    for label, log_text, machine_cores, explicitly_one, expected_green in cells:
        problems, recorded_lines, _notes = judge_crash_case_log(case, log_text, machine_cores, explicitly_one, default)
        green = not problems
        recorded_ok = not green or (recorded_lines[0].startswith("test_result=test result: ok. 1 passed")
                                    and any(line.startswith("CRASH_POINT_REPLAY ") for line in recorded_lines)
                                    and any(line.startswith("parallel_finished=CRASH_POINT_REPLAY_PARALLEL_FINISHED ") for line in recorded_lines))
        selftest.expect(f"判 --full 日志：{label} ⇒ {'绿' if expected_green else '红'}", green == expected_green and recorded_ok,
                        f"判成{'绿' if green else '红'}：{problems}；记下的行 {recorded_lines}")
    with BreakSwitch("threads-by-worker-count"):
        problems, _recorded, _notes = judge_crash_case_log(
            case, synthetic_crash_point_replay_log(worker_threads=1, resumed_slices=63, freshly_run_slices=1), cores, False, default)
    selftest.expect("弄坏开关 threads-by-worker-count 下「只剩 1 片要跑、起 1 个线程」那一格红（照旧只看 worker_threads=1）", bool(problems),
                    "弄坏之后仍判绿：这一格分不出「这一趟跑了几片」")
    with BreakSwitch("threads-default-unchecked"):
        problems, _recorded, _notes = judge_crash_case_log(case, cells[-2][1], cores, False, default)
    selftest.expect(f"弄坏开关 threads-default-unchecked 下「来源 available_parallelism、按核数取了 {default + 1}」那一格判绿（不比每台默认重活线程数）",
                    not problems, f"弄坏之后仍判红：{problems}")
    # 标记里「实际起的工作线程」那一格（started_worker_threads=）：登记了 threads= 的取 CRASH_POINT_REPLAY_PARALLEL_FINISHED 里起了几个，没登记的写读不到
    problems, recorded_lines, _notes = judge_crash_case_log(case, synthetic_crash_point_replay_log(worker_threads=24), cores, False, default)
    started = [line for line in recorded_lines if line.startswith("started_worker_threads=")]
    selftest.expect(f"判绿时记 started_worker_threads=：登记了 threads= 的记 CRASH_POINT_REPLAY_PARALLEL_FINISHED 里起了几个（24 个，不是配的 {default}）",
                    not problems and started == ["started_worker_threads=CRASH_POINT_REPLAY：24 个工作线程跑了 64 片、从进度文件读回 0 片（共 64 片）"],
                    f"判出 {problems}，记下 {started}")
    unthreaded = parse_crash_case(RegistrationRow("crash-case:plain", [], ["test=pkg:target:function", "count-line=PLAIN"], 1, ""))
    problems, recorded_lines, _notes = judge_crash_case_log(unthreaded, "PLAIN states=4\n" + PASSED_ONE_LINE + "\n", cores, False, default)
    started = [line for line in recorded_lines if line.startswith("started_worker_threads=")]
    selftest.expect("判绿时记 started_worker_threads=：没登记 threads= 的记「读不到」（门禁不读它起了几个）",
                    not problems and len(started) == 1 and started[0].startswith("started_worker_threads=读不到：这条用例没登记 threads="),
                    f"判出 {problems}，记下 {started}")
    run_shard_merge_log_cells(selftest, case)


def synthetic_merge_log(shard_worker_threads=None, shard_configured=None, shard_sources=("environment_variable", "environment_variable"),
                        shard_cores=(32, 32), shard_resumed=(0, 0), shard_freshly_run=(32, 32), shards=None):
    """合成的双机分片 merge 那一趟日志：CRASH_POINT_REPLAY_PARALLEL_FINISHED 的合计照 crates/singlefs-checker-tier/src/crash.rs 的 merge_the_shard_ledgers
    （线程与片数是各片之和、来源写 shard_ledgers），另带 shards= 与逐片的 shard_…=。逐片起的与配的线程数不给都取每台默认重活线程数（两台各一份）。"""
    default_threads = selftest_default_threads() or 1
    shard_worker_threads = shard_worker_threads or (default_threads, default_threads)
    shard_configured = shard_configured or (default_threads, default_threads)
    joined = lambda values: ",".join(str(value) for value in values)
    finished_fields = (f"worker_threads={sum(shard_worker_threads)} configured_worker_threads={sum(shard_configured)} worker_threads_source=shard_ledgers "
                       f"resumed_slices={sum(shard_resumed)} freshly_run_slices={sum(shard_freshly_run)} "
                       f"shards={shards if shards is not None else len(shard_worker_threads)} shard_worker_threads={joined(shard_worker_threads)} "
                       f"shard_configured_worker_threads={joined(shard_configured)} shard_worker_threads_sources={joined(shard_sources)} "
                       f"shard_available_parallelism={joined(shard_cores)} shard_resumed_slices={joined(shard_resumed)} "
                       f"shard_freshly_run_slices={joined(shard_freshly_run)} shard_elapsed_milliseconds=1000,1000")
    return synthetic_crash_point_replay_log(finished_fields=finished_fields)


def run_shard_merge_log_cells(selftest, case):
    """双机分片 merge 那一趟的日志：带 shards= 的 CRASH_POINT_REPLAY_PARALLEL_FINISHED 逐片判工作线程（机器核数与「显式设成 1」取那一片自己的）；
    登记行的 shard= 只认 across-machines、至多一条。"""
    default = selftest_default_threads() or 1
    cells = [
        (f"merge：两片各 {default} 个线程各跑 32 片", synthetic_merge_log(), True),
        ("merge：第 1 片那台 32 核、线程数没显式设成 1（来源 crash_amplification_threads_variable、配的 32），跑了 32 片却只起 1 个线程",
         synthetic_merge_log(shard_worker_threads=(default, 1), shard_configured=(default, 32),
                             shard_sources=("environment_variable", "crash_amplification_threads_variable")), False),
        ("merge：第 1 片只起 1 个线程，那一片的线程数显式设成 1",
         synthetic_merge_log(shard_worker_threads=(default, 1), shard_configured=(default, 1)), True),
        ("merge：第 1 片只起 1 个线程，那台只有 1 核", synthetic_merge_log(shard_worker_threads=(default, 1), shard_configured=(default, 1),
                                                          shard_sources=("environment_variable", "crash_amplification_threads_variable"),
                                                          shard_cores=(32, 1)), True),
        ("merge：第 1 片跑了 32 片却报起了 0 个线程", synthetic_merge_log(shard_worker_threads=(default, 0)), False),
        ("merge：shards=3 而逐片字段只有两片", synthetic_merge_log(shards=3), False),
        (f"merge：两片线程数都没显式设（来源 available_parallelism）、各取每台默认重活线程数 {default}",
         synthetic_merge_log(shard_sources=("available_parallelism", "available_parallelism")), True),
        (f"merge：第 1 片线程数没显式设（来源 available_parallelism）、按那台核数取了 {default + 1}，不等于每台默认重活线程数 {default}",
         synthetic_merge_log(shard_worker_threads=(default, default + 1), shard_configured=(default, default + 1),
                             shard_sources=("environment_variable", "available_parallelism")), False),
    ]
    for label, log_text, expected_green in cells:
        problems, _recorded, notes = judge_crash_case_log(case, log_text, SAMPLE_MACHINE_CORES, False, default)
        green = not problems
        selftest.expect(f"判 --full 日志：{label} ⇒ {'绿' if expected_green else '红'}",
                        green == expected_green and (not green or any("双机分片 2 片再 merge" in note for note in notes)),
                        f"判成{'绿' if green else '红'}：{problems}；线程那一句 {notes}")
    with BreakSwitch("threads-ignore-shards"):
        problems, _recorded, _notes = judge_crash_case_log(case, cells[1][1], SAMPLE_MACHINE_CORES, False, default)
    selftest.expect(f"弄坏开关 threads-ignore-shards 下「第 1 片只起 1 个线程跑了 32 片」那一格判绿（只看各片之和 {default + 1} 个线程，分不出是哪一片只起了 1 个）",
                    not problems, f"弄坏之后仍判红：{problems}")
    with BreakSwitch("threads-default-unchecked"):
        problems, _recorded, _notes = judge_crash_case_log(case, cells[-1][1], SAMPLE_MACHINE_CORES, False, default)
    selftest.expect(f"弄坏开关 threads-default-unchecked 下「merge：第 1 片来源 available_parallelism、取了 {default + 1}」那一格判绿（不比每台默认重活线程数）",
                    not problems, f"弄坏之后仍判红：{problems}")
    for label, conditions, expected in (
            ("登记了 shard=across-machines", ["test=pkg:target:function", "shard=across-machines"], True),
            ("没登记 shard=", ["test=pkg:target:function"], False),
            ("shard= 写成别的值", ["test=pkg:target:function", "shard=yes"], "error"),
            ("shard= 写了两条", ["test=pkg:target:function", "shard=across-machines", "shard=across-machines"], "error")):
        try:
            outcome = parse_crash_case(RegistrationRow("crash-case:demo", [], conditions, 1, "")).is_shardable_across_machines
        except RegistrationError:
            outcome = "error"
        selftest.expect(f"崩溃枚举用例登记行：{label} ⇒ {expected}", outcome == expected, f"读成 {outcome}")



def run_crash_case_features_cells(selftest):
    """登记行 features= 带进 crash-case-command：带特性的行拼出 -p <包> 之后带 --features、env 里设 bindgen 头文件目录的命令（多机配置 ENABLE_GPU=1 的另加 gpu、整条经 gpu-quiesce-run.sh 起），
    不带的行在 GPU 关着时命令照旧（不带 --features、不设 bindgen、不经包装），GPU 开着时一律加 gpu、经包装；登记行写错（小写以外、写两条、与 shard=across-machines 并存）拒。
    弄坏开关：features-not-passed（带特性那一格红）、features-passed-to-every-case（不带特性那一格红）、gpu-case-not-quiesced（开 GPU 那一格红）、gpu-test-threads-unset（开 GPU 的命令带 --test-threads 1 那一格红）。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-features-")
    try:
        plain_row = "crash-case:plain\t-\ttest=pkg:own_case:the_case count-line=CRASH_POINT_REPLAY\t# 样本"
        engine_row = "crash-case:engine\t-\ttest=pkg:own_case:the_case count-line=CRASH_POINT_REPLAY features=verdict-store\t# 样本"
        build_crash_case_repository(work, [plain_row, engine_row])
        write_text(os.path.join(work, BINDGEN_LIBRARY_SCRIPT), 'CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS="-I/sample/gcc/13/include -I/sample/gcc/14/include"\n')
        gpu_off = os.path.join(work, "multi-host-gpu-off.env")
        gpu_on = os.path.join(work, "multi-host-gpu-on.env")
        write_text(gpu_off, "ENABLE_GPU=0\n")
        write_text(gpu_on, 'ENABLE_GPU=1\nGPU_CARDS_BY_PRIORITY="local:0:8G"\nGPU_CARDS_BY_PRIORITY_WHEN_QUIESCED="local:0:24G"\n')
        rows = read_registration_rows(work)
        quiesce_script = os.path.join(work, GPU_QUIESCE_SCRIPT)
        cargo_head = ["cargo", "test", "--release", "-p", "pkg"]
        cargo_tail = ["--test", "own_case", "--", "--include-ignored", "--exact", "the_case", "--nocapture"]
        gpu_tail = cargo_tail + ["--test-threads", "1"]   # 开了 GPU 的命令收尾多一个 --test-threads 1（GPU 对拍的测试各占卡的显存，不并行起）

        def launched(key, configuration):
            caller = dict(os.environ, **{MULTI_HOST_CONFIGURATION_VARIABLE: configuration})
            caller.pop(CRASH_POINT_REPLAY_THREADS_VARIABLE, None)
            return crash_case_launch(work, crash_case_of_key(rows, key), False, caller)[3]

        def has_bindgen(command):
            return f"{BINDGEN_VARIABLE}=-I/sample/gcc/13/include -I/sample/gcc/14/include" in command

        plain = launched("crash-case:plain", gpu_off)
        plain_is_old = (plain[0] == "env" and plain[plain.index("cargo"):] == cargo_head + cargo_tail
                        and not has_bindgen(plain) and not any(word.startswith("GPU_") for word in plain))
        selftest.expect("crash-case-command：没登记 features= 的行、多机配置没开 GPU：命令照旧，不带 --features、不设 bindgen、不经 gpu-quiesce-run.sh",
                        plain_is_old, f"命令是 {plain}")
        plain_on_gpu = launched("crash-case:plain", gpu_on)
        plain_on_gpu_ok = (plain_on_gpu[:3] == ["bash", quiesce_script, "--"] and plain_on_gpu[plain_on_gpu.index("cargo"):] == cargo_head + ["--features", "gpu"] + gpu_tail
                           and has_bindgen(plain_on_gpu) and "GPU_CARDS_BY_PRIORITY=local:0:24G" in plain_on_gpu)
        selftest.expect("crash-case-command：没登记 features= 的行、多机配置开着 GPU：一律带 --features gpu（崩溃枚举用例的判段上卡）、设 bindgen、经 gpu-quiesce-run.sh 起、交清场之后那一份显卡先后",
                        plain_on_gpu_ok, f"命令是 {plain_on_gpu}")
        with BreakSwitch("features-passed-to-every-case"):
            broken_plain = launched("crash-case:plain", gpu_off)
        selftest.expect("弄坏开关 features-passed-to-every-case 下「不带特性的行命令照旧」那一格红（带上了 --features）", "--features" in broken_plain,
                        f"弄坏之后命令是 {broken_plain}")

        engine = launched("crash-case:engine", gpu_off)
        engine_ok = (engine[0] == "env" and engine[engine.index("cargo"):] == cargo_head + ["--features", "verdict-store"] + cargo_tail and has_bindgen(engine))
        selftest.expect("crash-case-command：登记了 features=verdict-store 的行（GPU 关）：-p pkg 之后带 --features verdict-store、env 里设 bindgen 头文件目录、不经包装",
                        engine_ok, f"命令是 {engine}")
        with BreakSwitch("features-not-passed"):
            broken_engine = launched("crash-case:engine", gpu_off)
        selftest.expect("弄坏开关 features-not-passed 下「带特性的行带 --features」那一格红（没带）", "--features" not in broken_engine, f"弄坏之后命令是 {broken_engine}")

        gpu = launched("crash-case:engine", gpu_on)
        gpu_ok = (gpu[:3] == ["bash", quiesce_script, "--"] and gpu[3] == "env" and "--features" in gpu
                  and gpu[gpu.index("--features") + 1] == "verdict-store,gpu" and "GPU_CARDS_BY_PRIORITY=local:0:24G" in gpu
                  and f"{MULTI_HOST_CONFIGURATION_VARIABLE}={gpu_on}" in gpu and gpu[gpu.index("-u") + 1] == "GPU_COMPUTATION_CLASSES")
        selftest.expect("crash-case-command：登记了 features=verdict-store 的行、多机配置 ENABLE_GPU=1：特性加 gpu，整条经 gpu-quiesce-run.sh 起，交清场之后那一份显卡先后",
                        gpu_ok, f"命令是 {gpu}")
        with BreakSwitch("gpu-case-not-quiesced"):
            broken_gpu = launched("crash-case:engine", gpu_on)
        selftest.expect("弄坏开关 gpu-case-not-quiesced 下「开 GPU 的带特性行经清场包装」那一格红（直接起 env）", broken_gpu[0] == "env" and "gpu" in broken_gpu[broken_gpu.index("--features") + 1],
                        f"弄坏之后命令是 {broken_gpu}")
        engine_on_gpu_tail = gpu[gpu.index("cargo"):][-len(gpu_tail):]
        selftest.expect("crash-case-command：开着 GPU（带特性行与不带特性行）的 cargo 参数收尾带 --test-threads 1，GPU 关的两种行都不带",
                        engine_on_gpu_tail == gpu_tail and plain_on_gpu[-2:] == ["--test-threads", "1"]
                        and "--test-threads" not in plain and "--test-threads" not in engine, f"开 GPU 带特性行的收尾是 {engine_on_gpu_tail}，不带特性行是 {plain_on_gpu[-2:]}，GPU 关的是 {plain[-2:]}、{engine[-2:]}")
        with BreakSwitch("gpu-test-threads-unset"):
            broken_threads_gpu = launched("crash-case:engine", gpu_on)
        selftest.expect("弄坏开关 gpu-test-threads-unset 下「开 GPU 的 cargo 参数带 --test-threads 1」那一格红（没带）", "--test-threads" not in broken_threads_gpu,
                        f"弄坏之后命令是 {broken_threads_gpu}")
        # 多机配置在而读不了：判不出开没开 GPU，抛错，不当成关
        unreadable = os.path.join(work, "multi-host-unreadable.env")
        with open(unreadable, "wb") as handle:
            handle.write(b"ENABLE_GPU=\xff\xfe\n")
        try:
            launched("crash-case:engine", unreadable)
            outcome = "没抛错"
        except InputManifestError as error:
            outcome = f"拒：{error}"
        selftest.expect("crash-case-command：多机配置在而读不了 ⇒ 抛 InputManifestError（不当成 GPU 关）", outcome.startswith("拒"), outcome)

        written = [("features= 写成大写", ["features=Verdict_Store"]), ("features= 写了两条", ["features=verdict-store", "features=gpu"]),
                   ("features= 里有空项", ["features=verdict-store,,gpu"]), ("features= 里有重复项", ["features=gpu,gpu"]),
                   ("features= 与 shard=across-machines 并存", ["features=verdict-store", "shard=across-machines"])]
        for label, conditions in written:
            try:
                parse_crash_case(RegistrationRow("crash-case:demo", [], ["test=pkg:target:function", *conditions], 1, ""))
                outcome = "读得出"
            except RegistrationError as error:
                outcome = f"拒：{error}"
            selftest.expect(f"崩溃枚举用例登记行：{label} ⇒ 拒", outcome.startswith("拒") and "features=" in outcome, outcome)
    finally:
        shutil.rmtree(work, ignore_errors=True)


def run_real_crash_case_features_cells(selftest, rows):
    """真仓登记表里经引擎的三条用例（crash-injection-fast-tier、read-faulted-states-never-hold-the-witness-criterion 两条随机历史用例，
    与随机历史每步镜像加故障注入经库判的 random-histories-and-fault-injection-given-states）登记着 features=verdict-store，
    其余的崩溃枚举用例不登记（它们的用例函数不经引擎，带特性只多编 RocksDB）；登记了 features= 的没有一条同时登记 shard=across-machines。"""
    engine_keys = (
        "crash-case:crash-injection-fast-tier",
        "crash-case:read-faulted-states-never-hold-the-witness-criterion",
        "crash-case:random-histories-and-fault-injection-given-states",
    )
    cases = crash_cases_of(rows)
    for key in engine_keys:
        found = [case for case in cases if case.key == key]
        selftest.expect(f"真仓登记的 {key} 登记着 features=verdict-store", len(found) == 1 and found[0].features == (VERDICT_STORE_FEATURE,),
                        f"登记的特性是 {[case.features for case in found]}")
    others = [case.key for case in cases if case.features and case.key not in engine_keys]
    selftest.expect("真仓登记表里登记了 features= 的只有经引擎的那三条用例", not others, f"另外还有 {others}")
    both = [case.key for case in cases if case.features and case.is_shardable_across_machines]
    selftest.expect("真仓登记表里 features= 与 shard=across-machines 没有并存的", not both, f"并存的有 {both}")


def heavy_threads_library():
    """scripts/hosts/heavy_threads.py（与 HEAVY_THREAD_DEFAULT_FILE 同一棵树的那一份）按路径载入；载不进交 (None, 原因)。只给自证对拍用。"""
    import importlib.util
    path = os.path.join(os.path.dirname(os.path.dirname(HEAVY_THREAD_DEFAULT_FILE)), "scripts", "hosts", "heavy_threads.py")
    try:
        specification = importlib.util.spec_from_file_location("heavy_threads_for_admission_selftest", path)
        library = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(library)
    except (OSError, ImportError, SyntaxError, AttributeError) as error:
        return None, f"{path} 载不进：{error}"
    return library, ""


def run_heavy_thread_default_cells(selftest, case, environment, cores, work):
    """没设 SINGLEFS_CRASH_POINT_REPLAY_THREADS 时线程数取每台默认重活线程数：SINGLEFS_THREAD_CAP=3 ⇒ 3、不是正整数拒，都没设 ⇒ 登记值（heavy_threads.py 读同一份交的数）；
    弄坏开关 threads-default-from-cores 下「都没设」那一格红（交本机核数）。registered_heavy_threads 与 heavy_threads.py 的 default_heavy_threads 拿同几份登记逐格对拍。"""
    without_cap = {name: value for name, value in environment.items() if name not in (THREAD_CAP_VARIABLE, "HEAVY_THREADS_BREAK")}
    library, why = heavy_threads_library()
    selftest.expect("每台默认重活线程数：scripts/hosts/heavy_threads.py 载得进（对拍要它）", library is not None, why)
    got = crash_case_worker_threads(case, dict(without_cap, **{THREAD_CAP_VARIABLE: "3"}))
    selftest.expect("没设 SINGLEFS_CRASH_POINT_REPLAY_THREADS、SINGLEFS_THREAD_CAP=3 ⇒ 线程数 3、default", got == (cores, 3, "default"), f"交的是 {got}")
    try:
        got = crash_case_worker_threads(case, dict(without_cap, **{THREAD_CAP_VARIABLE: "abc"}))
    except InputManifestError as error:
        got = f"拒：{error}"
    selftest.expect("没设 SINGLEFS_CRASH_POINT_REPLAY_THREADS、SINGLEFS_THREAD_CAP=abc ⇒ 拒（抛 InputManifestError）", str(got).startswith("拒") and THREAD_CAP_VARIABLE in str(got),
                    f"交的是 {got}")
    registered = library.default_heavy_threads(without_cap) if library is not None else None
    got = crash_case_worker_threads(case, without_cap)
    selftest.expect(f"没设 SINGLEFS_CRASH_POINT_REPLAY_THREADS 与 SINGLEFS_THREAD_CAP ⇒ 线程数是登记值（heavy_threads.py 读同一份交 {registered}）、default",
                    got == (cores, registered, "default"), f"交的是 {got}")
    with BreakSwitch("threads-default-from-cores"):
        broken_got = crash_case_worker_threads(case, without_cap)
    selftest.expect("弄坏开关 threads-default-from-cores 下「都没设 ⇒ 登记值」那一格红（交本机核数）", broken_got[1] != registered or cores == registered,
                    f"弄坏之后交的是 {broken_got}，登记值 {registered}")
    if library is None:
        return
    variants = [("登记 7、带理由", "7  # 样本理由\n", None), ("登记前有注释行与空行", "# 注释\n\n7  # 样本理由\n8  # 第二条不认\n", None),
                ("登记文件不在", None, None), ("写坏：不是数", "abc  # 样本理由\n", None), ("写坏：0", "0  # 样本理由\n", None),
                ("写坏：没写理由", "7\n", None), ("写坏：数与 # 之间没有空白", "7# 样本理由\n", None), ("写坏：只有注释", "# 只有注释\n", None),
                ("SINGLEFS_THREAD_CAP=5 盖过登记", "7  # 样本理由\n", "5"), ("SINGLEFS_THREAD_CAP=0", "7  # 样本理由\n", "0"),
                ("SINGLEFS_THREAD_CAP 设成空串当没设", "7  # 样本理由\n", "")]
    for index, (label, registration, cap) in enumerate(variants):
        path = os.path.join(work, f"heavy-thread-default-{index}")
        if registration is not None:
            write_text(path, registration)
        variant_environment = dict(without_cap) if cap is None else dict(without_cap, **{THREAD_CAP_VARIABLE: cap})
        try:
            mine = registered_heavy_threads(variant_environment, registration_file=path)
        except InputManifestError:
            mine = "refused"
        try:
            theirs = library.default_heavy_threads(variant_environment, registration_file=path)
        except library.HeavyThreadDefaultError:
            theirs = "refused"
        selftest.expect(f"每台默认重活线程数：与 heavy_threads.py 对拍，{label} ⇒ 两边都交「{theirs}」", mine == theirs, f"准入模块交「{mine}」，heavy_threads.py 交「{theirs}」")


def run_thread_source_cells(selftest, module):
    """门禁批第三轮 FM 与 FD3：线程从哪来、判法自己现取。
    FM：登记行的 threads-variable= 由 crash-case-command 设成配的线程数（调用方设着的不认，弄坏开关 thread-variable-ignored 下漏进来）、
    标记里 configured_worker_threads= 记这个变量；本机核数不认 OMP_NUM_THREADS 与 PATH 里的 nproc（弄坏开关 cores-from-nproc 下认）。
    FD3：crash-case-judge 自己现取线程数：显式设 SINGLEFS_CRASH_POINT_REPLAY_THREADS=1 的 64 片 1 线程判绿、没设的判红；转过来的 --threads 1 --threads-origin explicit
    是用法错（退 2），翻不了判定（弄坏开关 judge-takes-forwarded-threads 下照旧收、判绿）。
    每台默认重活线程数：没设 SINGLEFS_CRASH_POINT_REPLAY_THREADS 时线程数取它（SINGLEFS_THREAD_CAP 设了照它、不是正整数拒；弄坏开关 threads-default-from-cores 下照改前取本机核数），
    registered_heavy_threads 与 scripts/hosts/heavy_threads.py 的 default_heavy_threads 拿同几份登记逐格对拍。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-threads-")
    try:
        row = ("crash-case:own\t-\ttest=pkg:own_case:the_case count-line=CRASH_POINT_REPLAY count-line=CHECKER exhaustive=CRASH_POINT_REPLAY threads=CRASH_POINT_REPLAY "
               "threads-variable=SINGLEFS_SAMPLE_CASE_THREADS\t# 样本")
        build_crash_case_repository(work, [row])
        case = crash_case_of_key(read_registration_rows(work), "crash-case:own")
        cores = machine_core_count(os.environ)
        caller = dict(os.environ, SINGLEFS_SAMPLE_CASE_THREADS="1")
        caller.pop(CRASH_POINT_REPLAY_THREADS_VARIABLE, None)
        try:
            default = registered_heavy_threads(caller)
        except InputManifestError as error:
            selftest.expect("每台默认重活线程数：调用方这份环境里取得出", False, str(error))
            default = 1

        def seen_by_the_case(case_now):
            command = crash_case_launch(work, case_now, False, caller)[3]
            launched = command[:command.index("cargo")] + ["sh", "-c", 'printf "%s %s" "${SINGLEFS_SAMPLE_CASE_THREADS-unset}" "${SINGLEFS_CRASH_POINT_REPLAY_THREADS-unset}"']
            return subprocess.run(launched, env=caller, capture_output=True, text=True, check=False).stdout
        seen = seen_by_the_case(case)
        selftest.expect(f"crash-case-command：登记了 threads-variable=SINGLEFS_SAMPLE_CASE_THREADS、调用方设着它等于 1 ⇒ 起的用例看到配的 {default}（每台默认重活线程数，盖掉调用方的）",
                        seen == f"{default} {default}", f"用例看到「{seen}」")
        with BreakSwitch("thread-variable-ignored"):
            broken_case = crash_case_of_key(read_registration_rows(work), "crash-case:own")
            broken_seen = seen_by_the_case(broken_case)
            broken_text = configured_worker_threads_text(broken_case, cores, default, "default")
        selftest.expect("弄坏开关 thread-variable-ignored 下「调用方设着用例读的线程变量」那一格红（用例看到 1）", broken_seen.startswith("1 "),
                        f"弄坏之后用例看到「{broken_seen}」：这一格分不出命令设没设它")
        text = configured_worker_threads_text(case, cores, default, "default")
        selftest.expect("标记里 configured_worker_threads= 记的是这条用例读的那个变量（SINGLEFS_SAMPLE_CASE_THREADS=…），不是 SINGLEFS_CRASH_POINT_REPLAY_THREADS",
                        text.startswith(f"SINGLEFS_SAMPLE_CASE_THREADS={default}（没设，取每台默认重活线程数）"), f"记成「{text}」")
        selftest.expect("弄坏开关 thread-variable-ignored 下「标记记的变量」那一格红（记成 SINGLEFS_CRASH_POINT_REPLAY_THREADS）", broken_text.startswith("SINGLEFS_CRASH_POINT_REPLAY_THREADS="),
                        f"弄坏之后记成「{broken_text}」")
        registration_cells = [("threads-variable= 写成小写", "threads-variable=sample_threads"), ("threads-variable= 写了两条",
                              "threads-variable=A_THREADS threads-variable=B_THREADS")]
        for label, conditions in registration_cells:
            try:
                parse_crash_case(RegistrationRow("crash-case:demo", [], ["test=pkg:target:function", *conditions.split()], 1, ""))
                outcome = "读得出"
            except RegistrationError as error:
                outcome = f"拒：{error}"
            selftest.expect(f"崩溃枚举用例登记行：{label} ⇒ 拒", outcome.startswith("拒") and "threads-variable=" in outcome, outcome)
        # 本机核数：PATH 里的 nproc 打 1、OMP_NUM_THREADS=1 都不认
        tools = os.path.join(work, ".tools")
        write_executable(os.path.join(tools, "nproc"), "#!/usr/bin/env bash\necho 1\n")
        leaky = dict(os.environ, PATH=tools + os.pathsep + os.environ.get("PATH", ""), OMP_NUM_THREADS="1")
        leaky.pop(CRASH_POINT_REPLAY_THREADS_VARIABLE, None)
        got = crash_case_worker_threads(case, leaky)
        selftest.expect(f"本机核数：PATH 里的 nproc 打 1、OMP_NUM_THREADS=1 ⇒ 仍是 os.cpu_count() 与 CPU 亲和的小者（{cores}），线程数取每台默认重活线程数（{default}）、default",
                        got == (cores, default, "default"), f"交的是 {got}")
        run_heavy_thread_default_cells(selftest, case, leaky, cores, work)
        with BreakSwitch("cores-from-nproc"):
            broken_got = crash_case_worker_threads(case, leaky)
        selftest.expect("弄坏开关 cores-from-nproc 下「nproc 打 1」那一格红（交的核数是 1）", broken_got[0] == 1, f"弄坏之后交的是 {broken_got}")
        # FD3：crash-case-judge 自己现取线程数
        one_thread_log = os.path.join(work, "one-thread.log")
        write_text(one_thread_log, synthetic_crash_point_replay_log(worker_threads=1))
        judged = os.path.join(work, "judged.out")
        plain_environment = {name: value for name, value in os.environ.items() if name != CRASH_POINT_REPLAY_THREADS_VARIABLE}

        def judge(extra_arguments=(), changes=None):
            return run_in_environment([sys.executable, module, "crash-case-judge", work, "crash-case:own", one_thread_log, judged, *extra_arguments],
                                      dict(plain_environment, **(changes or {})))[0]
        forwarded = ("--machine-cores", str(cores), "--threads", "1", "--threads-origin", "explicit")
        cells = [("没设 SINGLEFS_CRASH_POINT_REPLAY_THREADS：64 片 1 个线程", (), None, 1 if cores > 1 else 0),
                 ("调用方显式设 SINGLEFS_CRASH_POINT_REPLAY_THREADS=1：64 片 1 个线程", (), {CRASH_POINT_REPLAY_THREADS_VARIABLE: "1"}, 0),
                 ("没设 SINGLEFS_CRASH_POINT_REPLAY_THREADS、转过来 --threads 1 --threads-origin explicit（checker-tier-crash-point-replay 改前的转法）", forwarded, None, EXIT_REGISTRATION_ERROR)]
        for label, extra_arguments, changes, expected in cells:
            got_exit = judge(extra_arguments, changes)
            selftest.expect(f"crash-case-judge 自己现取线程数：{label} ⇒ 退 {expected}", got_exit == expected, f"退 {got_exit}")
        broken_exit = judge(forwarded, {BREAK_VARIABLE: "judge-takes-forwarded-threads"})
        selftest.expect("弄坏开关 judge-takes-forwarded-threads 下「转过来 --threads 1 explicit」那一格红（照旧收、判绿退 0）", broken_exit == 0,
                        f"弄坏之后退 {broken_exit}")
    finally:
        shutil.rmtree(work, ignore_errors=True)


# command_crash_case_shardable 里「没登记」那一支的末两行（自证把 return 1 改成 return 0 造一个判法改动）

C561_SIGMA_STATES = 262144


def c561_sigma_log(exhaustive="true", worker_threads=None, configured_worker_threads=None):
    """合成的 crash-case:c561-sigma-full 一趟 --full 日志：用例打的计数行与同形的线程行（照 crates/singlefs-checker-tier/tests/
    record_checker_judges_absence_by_the_persisted_set.rs 的 count_line 与线程行；来源 available_parallelism，即线程数没显式设）。
    起的与配的线程数不给都取每台默认重活线程数。"""
    default_threads = selftest_default_threads() or 1
    worker_threads = default_threads if worker_threads is None else worker_threads
    configured_worker_threads = default_threads if configured_worker_threads is None else configured_worker_threads
    return (f"CRASH_POINT_REPLAY_PARALLEL_FINISHED states={C561_SIGMA_STATES} slices=64 worker_threads={worker_threads} configured_worker_threads={configured_worker_threads} "
            f"worker_threads_source=available_parallelism resumed_slices=0 freshly_run_slices=64 progress_file_after_completion=none elapsed_seconds=1.0\n"
            f"C561_SIGMA_FULL states={C561_SIGMA_STATES} closed_form_states={C561_SIGMA_STATES} exhaustive={exhaustive} "
            f"record_claimed_state_missing_unit=0\n{PASSED_ONE_LINE}\n")


def run_real_crash_case_row_cells(selftest, rows):
    """真仓登记表里两条崩溃枚举用例的第三列拿合成日志判：c561-sigma-full 的计数行不带 exhaustive=true、1 个线程跑了 64 片都判红，两样都对判绿
    （只登记 count-line= 时这两种都判绿）；crash-injection-fast-tier 登记着，计数行 CRASH_INJECTION_FINISHED 恰好一行判绿、两行判红，
    跑完那一行报 1 个线程跑了 24 片判红（它登记了 threads=CRASH_INJECTION_FINISHED；弄坏开关 threads-skip-self-contained-finish-line 下这一格转绿）；
    它登记的 threads-variable= 是 crates/singlefs-checker-tier/src/crash_injection.rs 里的 CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE。"""
    fast_tier_line = "CRASH_INJECTION_FINISHED seeds=[1,25) slices=24 worker_threads=4 elapsed_seconds=1.0\n"
    cells = [
        ("crash-case:c561-sigma-full", "两行都对", c561_sigma_log(), True),
        ("crash-case:c561-sigma-full", "计数行 exhaustive=false（状态数不等于闭式）", c561_sigma_log(exhaustive="false"), False),
        ("crash-case:c561-sigma-full", "本机 32 核、线程数没显式设，1 个线程跑了 64 片", c561_sigma_log(worker_threads=1), False),
        ("crash-case:c561-sigma-full", "线程数没显式设，用例按本机核数取了 32 个（不是每台默认重活线程数）",
         c561_sigma_log(worker_threads=32, configured_worker_threads=32), (selftest_default_threads() or 1) == 32),
        ("crash-case:crash-injection-fast-tier", "计数行恰好一行", fast_tier_line + PASSED_ONE_LINE + "\n", True),
        ("crash-case:crash-injection-fast-tier", "计数行打了两行", fast_tier_line * 2 + PASSED_ONE_LINE + "\n", False),
        ("crash-case:crash-injection-fast-tier", "本机 32 核、线程数没显式设，跑完那一行报 1 个线程跑了 24 片",
         fast_tier_line.replace("worker_threads=4", "worker_threads=1") + PASSED_ONE_LINE + "\n", False),
    ]
    for key, label, log_text, expected_green in cells:
        try:
            problems = judge_crash_case_log(crash_case_of_key(rows, key), log_text, SAMPLE_MACHINE_CORES, False, selftest_default_threads() or 1)[0]
        except RegistrationError as error:
            problems = [f"真仓登记表里读不出这一条：{error}"]
        green = not problems
        selftest.expect(f"真仓登记的 {key}：{label} ⇒ {'绿' if expected_green else '红'}", green == expected_green,
                        f"判成{'绿' if green else '红'}：{problems}")
    one_thread_fast_tier = cells[-1][2]
    with BreakSwitch("threads-skip-self-contained-finish-line"):
        problems = judge_crash_case_log(crash_case_of_key(rows, "crash-case:crash-injection-fast-tier"), one_thread_fast_tier, SAMPLE_MACHINE_CORES, False,
                                        selftest_default_threads() or 1)[0]
    selftest.expect("弄坏开关 threads-skip-self-contained-finish-line 下「crash-injection 快档 1 个线程跑了 24 片」那一格红（判绿）", not problems,
                    f"弄坏之后仍判红：{problems}")
    source_path = os.path.join(os.path.dirname(os.path.dirname(SELFTEST_HERE)), "crates/singlefs-checker-tier/src/crash_injection.rs")
    try:
        with open(source_path, encoding="utf-8") as handle:
            read_variables = re.findall(r'CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE: &str =\s*"([A-Z0-9_]+)"', handle.read())
        registered = crash_case_of_key(rows, "crash-case:crash-injection-fast-tier").thread_variable
    except (OSError, RegistrationError) as error:
        read_variables, registered = [], f"读不出：{error}"
    selftest.expect("真仓登记的 crash-case:crash-injection-fast-tier 的 threads-variable= 就是用例读的那个变量", read_variables == [registered],
                    f"登记的 {registered}，{source_path} 里读的 {read_variables}")


def library_setting_seen_by_the_case(work, caller_library):
    """照 crash_case_launch 交的命令起一趟（cargo 起往后换成打 SINGLEFS_CRASH_AMPLIFICATION_LIBRARY 的 sh），调用方环境里设着 caller_library：交回那一趟看到的值。"""
    case = crash_case_of_key(read_registration_rows(work), "crash-case:own")
    environment = dict(os.environ, SINGLEFS_CRASH_POINT_REPLAY_THREADS="2", SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=caller_library)
    command = crash_case_launch(work, case, False, environment)[3]
    launched = command[:command.index("cargo")] + ["sh", "-c", 'printf "%s" "${SINGLEFS_CRASH_AMPLIFICATION_LIBRARY-unset}"']
    return subprocess.run(launched, env=environment, capture_output=True, text=True, check=False).stdout


def run_crash_case_command_library_cells(selftest, work):
    """crash-case-command 交的命令起用例时清掉调用方环境里的 SINGLEFS_CRASH_AMPLIFICATION_LIBRARY（单机跑的那一趟照家目录下崩溃枚举用例那一类的长期库，
    不落进调用方留下的别的库）；弄坏开关 keep-caller-library 下那一格转红。"""
    seen = library_setting_seen_by_the_case(work, "/caller/library")
    selftest.expect("crash-case-command：调用方环境里设着 SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=/caller/library ⇒ 起的用例看不到它", seen == "unset", f"用例看到的是「{seen}」")
    with BreakSwitch("keep-caller-library"):
        seen = library_setting_seen_by_the_case(work, "/caller/library")
    selftest.expect("弄坏开关 keep-caller-library 下「调用方设着库目录」那一格红（用例看到 /caller/library）", seen == "/caller/library",
                    f"弄坏之后用例看到的是「{seen}」：这一格分不出命令清没清它")


REPLAY_SELFTEST_DRIVERS = r'''
# ── admission.py --selftest 拼进来的桩驱动（真仓的 replay.sh 没有这一段）──
driver_e900() {
  : >"$OUT_DIR/driver-ran-E900"
  printf 'E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n'
}
driver_e901() {
  : >"$OUT_DIR/driver-ran-E901"
  printf 'E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n'
}
driver_e902() {
  : >"$OUT_DIR/driver-ran-E902"
  printf 'E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n'
}
driver_e903() {
  : >"$OUT_DIR/driver-ran-E903"
  python3 scripts/gate/admission.py experiment . E903 || return $?
  printf 'E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n'
}
driver_e903_second() {
  : >"$OUT_DIR/driver-ran-E903-second"
  printf 'E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n'
}
'''
REPLAY_TABLE_START = "TABLE=$(cat <<'TSV'\n"
REPLAY_TABLE_END = "\nTSV\n)\n"
# 临时仓里跑 replay.sh 的副本要的计时行负载字段滞后表（replay.sh 开跑前读它，不在整份退 2）：自证的复跑表一行 timing 都没有，表只放一行表头
REPLAY_TIMING_LAG_TABLE = ".claude/gate.d/timing-load-field-lag.tsv"
REPLAY_TIMING_LAG_TABLE_HEAD = "# 自证：计时行产物的负载字段滞后表，一行都不登记\n"
REPLAY_STRIP_HEADER_COMMAND = "sed '/^E7INPUT /d'"


def replay_copy_text(table_rows, break_header_strip, row_keys=()):
    """真仓 replay.sh 的副本：只换掉复跑表与按行的准入键那一段（row_keys，一行一条、制表符分隔；不给就是空的，真仓 E142 那几条不带进来）、
    在表后面拼上桩驱动；break_header_strip 时把比对前删产物头那一步换成 cat。
    replay.sh 的形状变了（表的起止、删头那一句、按行的准入键那一段不是恰好一处）抛 RegistrationError，那一格判错。"""
    with open(os.path.join(SELFTEST_HERE, "..", "experiments", "replay.sh"), encoding="utf-8") as handle:
        text = handle.read()
    if (text.count(REPLAY_TABLE_START) != 1 or text.count(REPLAY_STRIP_HEADER_COMMAND) != 1
            or text.count(REPLAY_ROW_KEYS_START) != 1):
        raise RegistrationError("replay.sh 里复跑表的开头、比对前删产物头那一句或按行的准入键那一段不是恰好一处，自证拼不出副本")
    start = text.index(REPLAY_TABLE_START) + len(REPLAY_TABLE_START)
    end = text.index(REPLAY_TABLE_END, start)
    copy = (text[:start] + "\n".join(table_rows) + REPLAY_TABLE_END + REPLAY_SELFTEST_DRIVERS
            + text[end + len(REPLAY_TABLE_END):])
    keys_start = copy.index(REPLAY_ROW_KEYS_START) + len(REPLAY_ROW_KEYS_START)
    keys_end = copy.index("\n" + REPLAY_ROW_KEYS_END_LINE + "\n", keys_start - 1) + 1
    copy = copy[:keys_start] + "".join(line + "\n" for line in row_keys) + copy[keys_end:]
    if break_header_strip:
        copy = copy.replace(REPLAY_STRIP_HEADER_COMMAND, "cat")
    return copy



def run_replay_cells(selftest, module):
    """replay.sh 接准入的那几处，在临时仓里拿真仓 replay.sh 的副本（只换复跑表、加桩驱动）跑：cargo 与 rustc 用桩。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-replay-")
    tools = tempfile.mkdtemp(prefix="admission-selftest-tools-")
    try:
        run_quietly(["git", "init", "-q", "-b", "master", work], GIT_IDENTITY)
        write_executable(os.path.join(tools, "cargo"), '#!/usr/bin/env bash\n[[ "${1:-}" == -V ]] && echo "cargo 0.0.0-selftest"\nexit 0\n')
        write_executable(os.path.join(tools, "rustc"), '#!/usr/bin/env bash\n[[ "${1:-}" == -V ]] && echo "rustc 0.0.0-selftest"\nexit 0\n')
        os.makedirs(os.path.join(work, "scripts", "gate"), exist_ok=True)
        shutil.copy(module, os.path.join(work, "scripts/gate/admission.py"))
        shutil.copy(os.path.join(SELFTEST_HERE, FUNCTION_UNITS_LIBRARY), os.path.join(work, "scripts/gate", FUNCTION_UNITS_LIBRARY))
        write_text(os.path.join(work, REGISTRATION_TABLE),
                   "E900\tresearch/src/e900.sh\t# 样本\n"
                   "E901\tresearch/src/e901.sh\tquestion-row=research/questions.md#6:够判\t# 样本：前提没齐\n"
                   "E902\tresearch/src/e902.sh\t# 样本：装置没打指纹行\n")
        stored_body = "E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n"
        for number in ("900", "901", "902"):
            write_text(os.path.join(work, f"research/src/e{number}.sh"), f"#!/usr/bin/env bash\necho 输入 {number}\n")
            write_text(os.path.join(work, RESULTS_DIRECTORY, f"e{number}-2026-09-01.out"), stored_body)
        write_questions(work, "开着")
        write_text(os.path.join(work, REPLAY_TIMING_LAG_TABLE), REPLAY_TIMING_LAG_TABLE_HEAD)
        link_sop_copy(work)
        run_quietly(["git", "-C", work, "add", "-A"], GIT_IDENTITY)
        run_quietly(["git", "-C", work, "commit", "-qm", "base"], GIT_IDENTITY)
        table_rows = [f"E{number}|@driver_e{number}||e{number}-2026-09-01.out|exact" for number in ("900", "901", "902")]
        # E900、E901 的产物头由 replay.sh 接（驱动不问准入：驱动闭包里点了 admission.py，它的闭包收不全，实验键就拒算了）；E902 照装置自己打、却没打
        writer_rows = ["E900\tE900|@driver_e900|\treplay", "E901\tE901|@driver_e901|\treplay"]

        def replay(experiment, rows, break_header_strip=False):
            write_text(os.path.join(work, REPLAY_SCRIPT), replay_copy_text(rows, break_header_strip, writer_rows))
            output_directory = tempfile.mkdtemp(prefix="out-", dir=work)
            exit_code, output, messages = run_quietly(
                ["bash", os.path.join(work, REPLAY_SCRIPT), experiment],
                {"PATH": tools + os.pathsep + os.environ.get("PATH", ""), "REPLAY_OUT": output_directory, "REPLAY_JOBS": "1"})
            return exit_code, output + messages, output_directory

        exit_code, output, first_run = replay("E900", table_rows)
        selftest.expect("replay.sh：老产物没有产物头、新跑的有，比对前删掉产物头之后判字节一致",
                        exit_code == 0 and re.search(r"^E900 .*字节一致", output, re.M) is not None
                        and os.path.exists(os.path.join(first_run, "driver-ran-E900")), f"退 {exit_code}，输出：{output.strip()[-600:]}")
        exit_code, output, _run = replay("E900", table_rows, break_header_strip=True)
        selftest.expect("弄坏 replay.sh 比对前删产物头那一步（换成 cat）判对不上：上一格分得出差别",
                        exit_code == 1 and re.search(r"^E900 .*对不上", output, re.M) is not None, f"退 {exit_code}，输出：{output.strip()[-600:]}")
        shutil.copy(os.path.join(first_run, "E900.1.out"), os.path.join(work, RESULTS_DIRECTORY, "e900-2026-09-02.out"))
        rows_with_fingerprinted_product = [row.replace("e900-2026-09-01.out", "e900-2026-09-02.out") for row in table_rows]
        exit_code, output, unchanged_run = replay("E900", rows_with_fingerprinted_product)
        selftest.expect("replay.sh：留存产物头上的实验键与今天的相同时判「输入没变」、装置一次都没跑、汇总报没跑 1 行（只改复跑表里的产物文件名不动实验键）",
                        exit_code == 0 and re.search(r"^E900 .*输入没变", output, re.M) is not None and "输入没变没跑 1" in output
                        and not os.path.exists(os.path.join(unchanged_run, "driver-ran-E900")), f"退 {exit_code}，输出：{output.strip()[-600:]}")
        exit_code, output, refused_run = replay("E901", table_rows)
        selftest.expect("replay.sh：前提没齐时判跑不了（准入退 3）、装置一次都没跑",
                        exit_code == 1 and re.search(r"^E901 .*跑不了 .*准入没放行（退 3）", output, re.M) is not None
                        and not os.path.exists(os.path.join(refused_run, "driver-ran-E901")), f"退 {exit_code}，输出：{output.strip()[-600:]}")
        exit_code, output, _run = replay("E902", table_rows)
        selftest.expect("replay.sh：登记了准入、装置却没打实验键行时判跑不了",
                        exit_code == 1 and "产物头却没有 key=E902 的实验键行" in output, f"退 {exit_code}，输出：{output.strip()[-600:]}")
    except RegistrationError as error:
        selftest.expect("replay.sh 的副本拼得出来", False, str(error))
    finally:
        shutil.rmtree(work, ignore_errors=True)
        shutil.rmtree(tools, ignore_errors=True)



def run_replay_row_key_cells(selftest, module):
    """复跑表按行的准入键（records/2026-09-16-subagent拆分提案.md 第六十七节）：临时仓里一个实验 E903 在复跑表里两行，拿真仓 replay.sh 的副本跑。
    外面设 ADMISSION_BREAK=ignore-row-keys 或 REPLAY_BREAK=row-key-by-experiment 时，「两行各给键」那几格转红。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-row-keys-")
    tools = tempfile.mkdtemp(prefix="admission-selftest-tools-")
    try:
        run_quietly(["git", "init", "-q", "-b", "master", work], GIT_IDENTITY)
        for name, script in FAKE_TOOLCHAIN_SCRIPTS.items():
            write_executable(os.path.join(tools, name), script)
        os.makedirs(os.path.join(work, "scripts", "gate"), exist_ok=True)
        shutil.copy(module, os.path.join(work, "scripts/gate/admission.py"))
        shutil.copy(os.path.join(SELFTEST_HERE, FUNCTION_UNITS_LIBRARY), os.path.join(work, "scripts/gate", FUNCTION_UNITS_LIBRARY))
        write_text(os.path.join(work, REGISTRATION_TABLE),
                   "E903\tresearch/src/e903.sh\t# 样本：E903 两行里默认那一种（装置自己打产物头）\n"
                   "E903/second\tresearch/src/e903-second.sh\t# 样本：E903 第二行（跑的东西不接准入，产物头由 replay.sh 写）\n"
                   "E903/third\t@E903\tproduct-field=E903:verdict:control_ok=true\t# 样本：按实验号点名产物，复跑表里键是 E903 的只一行\n")
        stored_body = "E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n"
        for name in ("e903", "e903-second"):
            write_text(os.path.join(work, f"research/src/{name}.sh"), f"#!/usr/bin/env bash\necho 输入 {name}\n")
            write_text(os.path.join(work, RESULTS_DIRECTORY, f"{name}-2026-09-01.out"), stored_body)
        write_text(os.path.join(work, REPLAY_TIMING_LAG_TABLE), REPLAY_TIMING_LAG_TABLE_HEAD)
        link_sop_copy(work)
        run_quietly(["git", "-C", work, "add", "-A"], GIT_IDENTITY)
        run_quietly(["git", "-C", work, "commit", "-qm", "base"], GIT_IDENTITY)
        table_rows = ["E903|@driver_e903||e903-2026-09-01.out|exact", "E903|@driver_e903_second||e903-second-2026-09-01.out|exact"]
        row_keys = ["E903\tE903|@driver_e903|\tdevice", "E903/second\tE903|@driver_e903_second|\treplay"]

        def replay(arguments, keys):
            write_text(os.path.join(work, REPLAY_SCRIPT), replay_copy_text(table_rows, False, keys))
            output_directory = tempfile.mkdtemp(prefix="out-", dir=work)
            exit_code, output, messages = run_quietly(
                ["bash", os.path.join(work, REPLAY_SCRIPT), *arguments],
                {"PATH": tools + os.pathsep + os.environ.get("PATH", ""), "REPLAY_OUT": output_directory, "REPLAY_JOBS": "1"})
            ran = sorted(name for name in os.listdir(output_directory) if name.startswith("driver-ran-"))
            return exit_code, output + messages, output_directory, ran

        def head_of(path):
            if not os.path.isfile(path):
                return ""
            with open(path, encoding="utf-8", errors="replace") as handle:
                return handle.read()[:400]

        exit_code, output, _run, ran = replay(["E903"], ())
        selftest.expect("复跑表按行的准入键：同一实验两行、只按实验号登记时 replay.sh 整份退 2，红在「准入只按实验号比产物」那一句，一行都没跑",
                        exit_code == 2 and "E903 登记了准入，而这张复跑表里它有 2 行：准入只按实验号比产物，分不清哪一行对哪一份" in output and not ran,
                        f"退 {exit_code}，跑了 {ran}，输出：{output.strip()[-600:]}")
        exit_code, output, keyed_run, ran = replay(["E903"], row_keys)
        first_head, second_head = head_of(os.path.join(keyed_run, "E903.1.out")), head_of(os.path.join(keyed_run, "E903.2.out"))
        selftest.expect("复跑表按行的准入键：两行各给键、各自登记时两行都跑、都判字节一致；第一行产物头是装置打的 key=E903，第二行是 replay.sh 接上的 key=E903/second",
                        exit_code == 0 and len(re.findall(r"^E903 .*字节一致", output, re.M)) == 2
                        and "E7INPUT name=input_fingerprint key=E903 " in first_head
                        and first_head.startswith("E7INPUT name=input_fingerprint key=E903 ")
                        and second_head.startswith("E7INPUT name=input_fingerprint key=E903/second "),
                        f"退 {exit_code}，跑了 {ran}，第一行产物头「{first_head[:120]}」，第二行「{second_head[:120]}」，输出：{output.strip()[-600:]}")
        exit_code, output, _run, ran = replay(["--admission-only", "E903"], row_keys)
        selftest.expect("复跑表按行的准入键：--admission-only 逐行列出各自的键与判定，一行都没跑",
                        exit_code == 0 and re.search(r"^E903 +@driver_e903 +放行 +准入键 E903，", output, re.M) is not None
                        and re.search(r"^E903 +@driver_e903_second +放行 +准入键 E903/second，", output, re.M) is not None and not ran,
                        f"退 {exit_code}，跑了 {ran}，输出：{output.strip()[-600:]}")
        unregistered_keys = [row_keys[0], "E903/fourth\tE903|@driver_e903_second|\treplay"]
        exit_code, output, _run, ran = replay(["E903"], unregistered_keys)
        selftest.expect("复跑表按行的准入键：两行各给键而其中一个没在登记表登记时整份退 2，点名那个键，一行都没跑",
                        exit_code == 2 and f"指明的准入键 E903/fourth 没在 {REGISTRATION_TABLE} 登记" in output and not ran,
                        f"退 {exit_code}，跑了 {ran}，输出：{output.strip()[-600:]}")
        dangling_keys = [row_keys[0], "E903/second\tE903|@driver_e903_missing|\treplay"]
        exit_code, output, _run, ran = replay(["E903"], dangling_keys)
        selftest.expect("复跑表按行的准入键：指到复跑表里没有的行时整份退 2，点名那一条",
                        exit_code == 2 and "「E903|@driver_e903_missing|」在复跑表里有 0 行，要恰好一行" in output and not ran,
                        f"退 {exit_code}，跑了 {ran}，输出：{output.strip()[-600:]}")
        write_text(os.path.join(work, REPLAY_SCRIPT), replay_copy_text(table_rows, False, row_keys))
        exit_code, _output, messages = run_quietly(["python3", os.path.join(work, "scripts/gate/admission.py"), "experiment", work, "E903/third"])
        selftest.expect("复跑表按行的准入键：product-field=E903 只认键是 E903 的那一行产物（复跑表里 E903 两行），前提判得出、放行",
                        exit_code == 0 and "e903-2026-09-01.out 的 name=verdict control_ok=true" in messages, f"退 {exit_code}，stderr：{messages.strip()[-600:]}")
    except RegistrationError as error:
        selftest.expect("复跑表按行的准入键：replay.sh 的副本拼得出来", False, str(error))
    finally:
        shutil.rmtree(work, ignore_errors=True)
        shutil.rmtree(tools, ignore_errors=True)




def run_plan_failure_cells(selftest, module):
    """plan-failure（文件头「按条复用失灵」）：拿造出来的几趟输出起它，判红的退 1、点名哪一条哨兵或退了几；外面设了 ADMISSION_BREAK=plan-failure-not-red，
    判红的那几格转错（子进程照传外面的环境）。两种样本标记在单元键那几格的临时小仓上起真的 verdict-plan。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-plan-failure-")
    try:
        def failure(command, exit_code, text, changes=None):
            path = os.path.join(work, f"output.{len(os.listdir(work))}")
            write_text(path, text)
            return run_in_environment([sys.executable, module, "plan-failure", "样本调用方的", command, str(exit_code), path], dict(os.environ, **(changes or {})))
        cases = (
            ("哨兵报复用失灵：verdict-plan 退 1、红句点名那一项与「复用失灵」", "verdict-plan", 1,
             "a\treuse\tk\t键相同\n  · 复用哨兵：…\nPLAN judge=0 reuse=1 refused=0 library=x\n  ✗ 复用失灵：a 键相同却现判了（复用哨兵，种子 7：这一趟报的是「现判」）\n     → 怎么办：…\n",
             1, ("样本调用方的按条复用失灵：复用哨兵报红（复用失灵）：复用失灵：a 键相同却现判了", "文件头「复用哨兵」")),
            ("哨兵报判定不稳：verdict-record 退 1 也判红", "verdict-record", 1, "  ✗ 判定不稳：b（库里那一条是「绿」，这一趟强制现判是「红」）\nRECORD recorded=0 changed=0 refused=0 library=x\n",
             1, ("复用哨兵报红（判定不稳）：判定不稳：b", "文件头「单元键」")),
            ("哨兵报强制现判没起作用：test-plan 退 1 判红", "test-plan", 1, "  ✗ 复用哨兵（x）：抽中强制现判的 c 这一趟还是复用了，强制现判没起作用（种子 3）\n",
             1, ("复用哨兵报红（强制现判没起作用）：复用哨兵（x）：抽中强制现判的 c", "先问 forced")),
            ("verdict-plan 退 2（读不了请求文件）：红句写退 2 与那一行 ✗、出路指到子命令那一条", "verdict-plan", 2, "  ✗ 读不了请求文件 r：不在\n     → 怎么办：…\n",
             1, ("scripts/gate/admission.py verdict-plan 退 2（✗ 读不了请求文件 r：不在）", "文件头子命令「verdict-plan」")),
            ("test-plan 退 0 却没打 PLAN 那一行：判红", "test-plan", 0, "x\tjudge\tk\n", 1, ("test-plan 退 0 却没打 PLAN 那一行",)),
            ("verdict-plan 退 0、打了 PLAN、没有哨兵红句：不判、一个字不打", "verdict-plan", 0, "a\tjudge\tk\t空库\nPLAN judge=1 reuse=0 refused=0 library=x\n", 0, ()),
            ("verdict-record 退 2 而没有哨兵红句（记不进库）：不算失灵、退 0", "verdict-record", 2, "  ✗ 读不了计划文件或名字文件：不在\n", 0, ()),
        )
        for label, command, exit_code, text, wanted_exit, needles in cases:
            got_exit, output, messages = failure(command, exit_code, text)
            selftest.expect(f"plan-failure：{label}", got_exit == wanted_exit and all(needle in output for needle in needles) and (wanted_exit or not output.strip()),
                            f"退 {got_exit}（要 {wanted_exit}），输出：{(output + messages).strip()[-500:]}")
        got_exit, output, _messages = failure("verdict-plan", 1, "  ✗ 复用失灵：a 键相同却现判了\n", {BREAK_VARIABLE: "plan-failure-not-red"})
        selftest.expect("plan-failure：弄坏开关 plan-failure-not-red 在自己那一趟里打开，复用失灵也退 0、只打一行 !（改前不判红的样子）",
                        got_exit == 0 and "弄坏开关 plan-failure-not-red：退回全部现判、不判红" in output, f"退 {got_exit}，输出：{output.strip()[-300:]}")
        # 两种样本标记：单元键那几格的临时小仓上起真的 verdict-plan
        repository = os.path.join(work, "repository")
        environment = build_unit_sample_repository(repository)
        requests_file = os.path.join(work, "marker.requests")
        write_text(requests_file, "".join(request.text() + "\n" for request in unit_sample_requests()))
        for form, needles in (("exit-1", ("样本档造的失灵", "写着 exit-1")),
                              ("sentinel-reuse-failed", ("PLAN judge=", "✗ 复用失灵：crates/demo/tests/setting.rs::", "样本档造的假哨兵"))):
            write_text(os.path.join(repository, PLAN_FAILURE_SAMPLE_MARKER), form + "\n")
            plan_exit, plan_output, plan_messages = run_in_environment(
                [sys.executable, module, "verdict-plan", repository, "plan-failure-sample.sh", requests_file, os.path.join(work, f"marker-{form}.plan")], environment)
            output_path = os.path.join(work, f"marker-{form}.out")
            write_text(output_path, plan_output + plan_messages)
            judged_exit, judged_output, _messages = run_in_environment([sys.executable, module, "plan-failure", "样本调用方的", "verdict-plan", str(plan_exit), output_path],
                                                                       dict(os.environ))
            selftest.expect(f"plan-failure：样本标记 {form} 下 verdict-plan 退 1、plan-failure 判红",
                            plan_exit == 1 and all(needle in plan_output for needle in needles) and judged_exit == 1 and "样本调用方的按条复用失灵：" in judged_output,
                            f"verdict-plan 退 {plan_exit}：{(plan_output + plan_messages).strip()[-400:]}；plan-failure 退 {judged_exit}：{judged_output.strip()[-300:]}")
        write_text(os.path.join(repository, PLAN_FAILURE_SAMPLE_MARKER), "exit-2\n")
        plan_exit, plan_output, _messages = run_in_environment(
            [sys.executable, module, "verdict-plan", repository, "plan-failure-sample.sh", requests_file, os.path.join(work, "marker-bad.plan")], environment)
        selftest.expect("plan-failure：样本标记写的不是认得的两样，verdict-plan 退 2、点名那份标记", plan_exit == EXIT_REGISTRATION_ERROR and "认不出" in plan_output,
                        f"退 {plan_exit}：{plan_output.strip()[-300:]}")
    finally:
        shutil.rmtree(work, ignore_errors=True)


def run_selftest():
    selftest = Selftest()
    module = os.path.abspath(__file__)
    run_unit_key_cells(selftest, module)
    run_verdict_library_cells(selftest, module)
    run_reuse_three_question_cells(selftest, module)
    run_plan_failure_cells(selftest, module)
    run_experiment_cells(selftest, module)
    run_row_key_cells(selftest, module)
    run_replay_cells(selftest, module)
    run_replay_row_key_cells(selftest, module)
    run_gate_precondition_cells(selftest, module)
    run_crash_cases_listing_cells(selftest, module)
    run_crash_case_log_cells(selftest)
    run_thread_source_cells(selftest, module)
    run_crash_case_features_cells(selftest)
    run_crash_case_blocks_cells(selftest, module)
    # 真仓的登记表自己写对没有（键、入口的写法与文件在不在、条件的种类分门禁与实验、@ 引用、probe= / environment= 指的脚本在不在）
    repository_root = os.path.dirname(os.path.dirname(SELFTEST_HERE))
    real_rows = read_registration_rows(repository_root)
    real_problems = lint_registration_rows(real_rows, repository_root)
    selftest.expect(f"真仓的登记表 {REGISTRATION_TABLE} 自查没有问题", not real_problems, "；".join(real_problems))
    run_real_crash_case_row_cells(selftest, real_rows)
    run_real_crash_case_features_cells(selftest, real_rows)
    _real_rows, _registered, real_row_key_problems, real_shared_keys = replay_row_key_problems(repository_root)
    selftest.expect("真仓 replay.sh 复跑表每一行定得下准入键：按行的准入键那一段写得对、指明的键都登记了、没有登记了准入的键被两行共用",
                    not real_row_key_problems and not real_shared_keys,
                    "；".join(real_row_key_problems + [f"{key} 被 {count} 行共用" for key, count in real_shared_keys]))
    if selftest.failures:
        print(f"  ✗ admission.py 自证没过：{selftest.failures} 格判错（共 {selftest.checked} 格）")
        print("     → 怎么办：照上面每一格的说明改被测的那一段；判据与为什么这么定见文件头。弄坏开关那几格红不了，说明那一格分不出差别")
        return 1
    print(f"  ✓ admission.py 自证通过：{selftest.checked} 格都对（单元键的阳性与阴性对照、判定库的现判 / 复用 / 只写不读、门禁复用答法的 H2 / H3 / E01、"
          "verdict-plan 的复用三问与复用哨兵、实验键的 T19 / N22、复跑表行键的 R01–R05 / T18 / E22、崩溃枚举用例的登记 / 日志 / 线程 / 特性 / 块判定那一行 / red-blocks= 标记与 --ran；"
          "含弄坏开关 verdict-key-with-entry-file、verdict-lookup-skipped、tree-compare-reuse、"
          "experiment-key-with-kb-documents、skip-build-environment、row-key-with-product-name、row-key-without-gpu、blocks-count-unchecked、ran-blocks-line-optional、skip-unchanged、"
          "skip-preconditions、skip-gate-preconditions 与崩溃枚举用例那几格的老开关下各自那一格转红）")
    return 0


COMMANDS = {
    "entries": command_entries,
    "paths": lambda arguments: command_entries(arguments, files_only=True),
    "test-functions": command_test_functions,
    "test-plan": command_test_plan,
    "unit-key": command_unit_key,
    "verdict-plan": command_verdict_plan,
    "verdict-record": command_verdict_record,
    "plan-failure": command_plan_failure,
    "gate-reuse": command_gate_reuse,
    "gate-preconditions": command_gate_preconditions,
    "experiment": command_experiment,
    "keys": command_keys,
    "replay-row-keys": command_replay_row_keys,
    "replay-row-inputs": command_replay_row_inputs,
    "crash-cases": command_crash_cases,
    "crash-case-command": command_crash_case_command,
    "crash-case-judge": command_crash_case_judge,
    "crash-case-blocks": command_crash_case_blocks,
    "crash-case-shardable": command_crash_case_shardable,
    "crash-case-judged-by-test": command_crash_case_judged_by_test,
}


def main(arguments):
    if arguments[:1] == ["--selftest"]:
        return run_selftest()
    if not arguments or arguments[0] not in COMMANDS:
        say(f"  ✗ 用法：admission.py {{{'|'.join(COMMANDS)}}} … 或 --selftest")
        say("     → 怎么办：各子命令的参数见文件头；整道全绿标记、用例全绿标记、输入清单与环境记录那几个子命令删了，复用改由各道按判定入库（verdict-plan / verdict-record）")
        return EXIT_REGISTRATION_ERROR
    return COMMANDS[arguments[0]](arguments[1:])


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
