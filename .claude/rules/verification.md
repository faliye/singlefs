# 验证纪律：只有一个门禁，不指定范围与指定范围两种跑法

**这是 singlefs 的项目本地规则**，不在共享 SOP 里——它说的是这个仓的验证代码住在哪、什么时候跑。
共享规则在 `.claude/singlefs-ai-sop/rules/`。

## 定义与名字

四个包，四个名字，全仓只用这四个名字称呼它们：

| 名字 | 包 | 装什么 | 什么时候跑 |
|---|---|---|---|
| **验证包** | `crates/singlefs-verification` | 单元测试与集成测试，加上它们用的脚手架库：录制器、内存池（`memory_pool`）、理想模型、随机历史、故障注入设备、场景；崩溃态枚举引擎与记录核对器（`crash`）、崩溃枚举用例经判定库的两段（流先入库、再从库里读回判，判在 GPU 或 CPU）与它的计数（`crash_state_enumeration`）、崩溃点重放发现日志与工具链身份（`crash_point_replay_progress`）、一份长期判定库与两段协议（`crash_library`；块键只由内容定，测试的类只当索引）、崩溃注入与坏盘输入两场战役（随机历史两项的给定状态表、它的判器 trait 与 CPU 就地判在 `crash_given_states`）、可写挂载每个写落在哪一段的划分与清扫段的写（`writable_mount_phases`）、真设备一侧的设备日志比对与运行模式、崩溃放量（崩溃点身份与复用键 `crash_identity`、先录后核的流水线 `crash_amplification`、判定存储 `verdict_store`、单元校验和上 GPU `gpu_unit_checks`、三段流第 ① 段的事实表与 CPU 参照核对 `crash_facts`、第 ② 段的 GPU 核对内核 `crash_verify_gpu`、GPU 判器的输入表 `crash_judge_tables`、GPU 判器的判法 `crash_judge_gpu` 与它的派活 `crash_judge_dispatch`；`gpu_unit_checks`、`crash_verify_gpu`、`crash_judge_gpu` 与 `crash_judge_dispatch` 挂特性 `gpu`）、实验与真设备装置二进制（`src/bin/`），以及它们的全部用例（照「一个门禁：两种跑法」跑）；连同 QEMU（门禁 qemu-device-streams）、herd7（lkmm）、crates 变异整表（crates-mutation-replay）、全部实验复跑（research-build-and-replay 的 experiment-replay 格）、形式化验证（formal-verification：Verus 验登记表 `.claude/gate.d/verus-pins.tsv` 点名的各 crate，`crates/singlefs-replay-prefix` 与 `crates/singlefs-core` 的被验区域，E165 红例钉表两格）、广谱变异（驱动 `scripts/mutation/broad-mutation.py`；提交门禁 broad-mutation 判此刻源码上每一条变异都有单元键相同的结论、没有没红） | 不指定范围的在提交时由主 agent 跑，提交之外要跑先弹窗问用户；指定范围的主 agent 随时经任务服务交 |
| **池级 checker** | `crates/singlefs-checker` | 判一个镜像的判决器库（O2，`check_pool_image`，D13（验证路线） 已定项 7）；只依赖 `singlefs-format` | 它是库，被验证包调用；它自己的库单测随时跑 |
| **RefFS** | `crates/singlefs-reference-filesystem` | 语义级的参照文件系统：只在内存里按 POSIX 操作走，判一次崩溃之后恢复读出的内容落不落在允许集合里；依赖只有 `std`，不引实现、池级 checker、验证包，实现与池级 checker 也不依赖它（门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格 ⑤⑥⑦ 判） | 它是库，被验证包调用；它自己的单测随时跑 |
| **函数单元图** | `crates/singlefs-unit-graph` | Rust 与 WGSL 源码按条目切成单元、取去注释指纹、从入口收闭包、算复用键的库，只用 `std`（D13（验证路线） 已定项 17 的键算法住在这里）；归测试基础设施、验证包依赖的一侧，不是产品代码 | 它是库，被验证包的构建脚本与门禁脚本调用；它自己的单测随时跑 |

测试文件按它测什么起名：领域在前、场景在后（`rollback_by_a_forward_publish`、`crash_enumeration_fixed_script_stream`），崩溃枚举、崩溃点、崩溃注入、坏盘输入与记录核对器的测试文件以模块起头（`crash_enumeration_`、`crash_points_`、`crash_injection_`、`bad_disk_input_`、`record_checker_`）；名字不带来历照 `.claude/rules/test-script-discipline.md`「用例原子化：每一条都能单独跑」，本仓判的形态是以里程碑序数起头的、`*_step_four_*`、`*_supplement_two_*`、`*_parallel_line_one_*`、`*_c519_*`。`scripts/crash/crash-case-check.py` 的 file-names 那一样判，门禁 code-source-discipline 的 test-file-names 格在真仓上跑它。
不许用的叫法：「checker 包」（分不清是池级 checker 还是验证包）、「放量用例」「验证档」（说验证包）。

## 测试不分档，红了只重跑红的

- 验证包的测试不分档，不带参数就是全部：`cargo test -p singlefs-verification` 跑到包里全部测试；测试不按快慢、大小标 `#[ignore]`。主 agent 经 `task-server/task submit -- <命令>` 交，子 agent 写进报告「要主 agent 跑的命令」一节（`.claude/agent-common.md`「子 agent 不交任务」），内存上限、线程与哪台机器由服务定，调用方不写。
- 红了只重跑红的那几条（`.claude/rules/test-script-discipline.md`「用例原子化：每一条都能单独跑」）：经 `task-server/task submit -- python3 scripts/gate/rerun-failed-tests.py <上一趟的日志> -p <包>` 交、`task-server/task log <号> --all` 取它按测试目标打印的只跑那几条的命令（开头已是 `SINGLEFS_RUN_SCOPE='…' task-server/task submit --`），逐条原样交，不再插 submit；它判红、一条命令都不打时才整份重跑，完整的一遍归提交时的整轮门禁。
- 调崩溃枚举函数（`enumerate_crash_states` 一族）或自己逐个造崩溃状态（名字带 `every_crash`；循环里对录制操作取到循环变量为止的前缀去 `apply`、或造 `CrashImage`），直接这样做或经同一文件里的函数这样做的测试是崩溃枚举用例，写进 `crates/singlefs-verification/tests/`、登记 `crash-case:`、不标 `#[ignore]`；`scripts/crash/crash-case-check.py` 的 placement 那一样判这一条，写在别的包里、标了 `#[ignore]` 或没登记的判红，门禁 `crash-point-replay` 开跑前在真仓上跑它。
- 一条用例一个场景照 `.claude/rules/test-script-discipline.md`「用例原子化：每一条都能单独跑」：本仓按「每一轮新建一个池」认参数循环（`build_pool`、`build_through_*`、`format_pool`、`MemoryPool::with_devices`），确是一个场景的在用例上面写一行 `// harness-test-granularity:one-scenario <理由>`。`scripts/crash/crash-case-check.py` 的 one-scenario 那一样判，门禁 model-differential-and-scenarios 的 one-scenario 格在真仓上跑它。

## 一个门禁：两种跑法

门禁只有一个，跑法两种：不指定范围（不带选条参数，门禁照单元键自己选要判的，判全部项）与指定范围（`--item`、`--check`、点名测试目标）。没有分档，也没有不读判定库、全部现判的跑法。

| 跑法 | 怎么起 | 谁跑、什么时候 |
|---|---|---|
| 不指定范围 | 门禁阶段不带 `--check` 与 `--item`；`scripts/crash/crash-point-replay.sh`、`scripts/crash/crash-amplification.sh` 不带 `--item`；`scripts/experiment-apparatus/replay.sh` 不带实验号；`cargo test` 不点名测试目标；`gate.sh` 整轮。范围声明种类 `whole` | 在提交时由主 agent 经任务服务交（`.claude/skills/crash-test/commit-stages.md`、`.claude/skills/gate/staged-gate.md`）；提交之外要跑先弹窗问用户 |
| 指定范围 | `--item <项>`（可给几次，`--list-items` 列可点的项）、`--check <格>`、`cargo test` 点名测试目标（`--test`、`--lib`、`--bin`）。范围声明种类 `items` 或 `target` | 主 agent 随时经任务服务交，点这一次改到的项、格与目标；子 agent 一律不交任务，写进报告「要主 agent 跑的命令」一节 |

- 判哪些、复用哪些由门禁照单元键自己选：只剪单元键相同、已判过的与等价的；单元键只由判定依赖的函数单元各自去注释后的指纹、闭包真引到的 cfg 原子与它们的真值、工具链、解析出的闭包真引到的依赖版本与判定自己的内容输入组成，路径、名字、行号、资源上限、限时、提交与树不进键；两种排除：文件级是 `scripts/crash/lib_verdict_library.py` 的 `NOT_JUDGING_FILES` 那两份，管怎么存、怎么算键，不管判对错，它们的单元不进任何一份判定的键，crates 变异判法除外：crates 变异判法不在文件级排除的射程里，那里两份照进判法单元指纹、覆盖号带判法单元指纹，边级是只打提示、不改判定结局的子进程边，按实测排，不沿它收，登记在 `scripts/gate/lib_function_units.py` 的 `EXCLUDED_PROGRAM_EDGES`，边级排除的三格自证在 `.claude/rules/implementation-workflow.md`「复用门禁的判定：按每条判定的单元键判，不按「改了哪个目录」或「哪棵树绿过」判」；python 模块有单元进闭包时，它在 import 时执行的模块级语句一起进闭包：import、赋值、类头本身（不连方法）、带装饰器或缺省值的 def 头；D13（验证路线） 已定项 17。
- 缩域：崩溃域只往小缩，崩溃点重放入口、崩溃放量入口与 `scripts/gate/merge-and-verify.sh` 认 `--crash-expand-up-to <n>`，环境变量是 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO`、`SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`；不设就是全量域。缩过域的那一趟成功句写「缩过域、不算门禁通过」并写出实际值，不当门禁通过。
- 门禁 `crash-point-replay` 不指定范围那一趟：crash-case-replay 格起 `cargo test --release -p singlefs-verification --lib --tests`（库、集成测试与装置二进制 `src/bin/` 的内联单测：`--tests` 带上全部 `test = true` 的目标，装置二进制都是），包里全部测试函数按单元键复用，登记的崩溃枚举用例与放量分项经 `--skip` 排掉、交按块那一路逐条跑；feature-gated-tests 格再一趟 `cargo test --release -p singlefs-verification --features <tests/ 里 cfg 点到的特性> --lib --test <挂特性的测试文件>…`（`BINDGEN_EXTRA_CLANG_ARGS` 取 `scripts/crash/crash-amplification-lib.sh` 的 `CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS`；特性里有 `gpu` 的这一趟直接起（清场由服务在最外层做，门禁里不另套），照旧一条 `cargo test`；不带特性那一趟与特性里没有 `gpu` 的那一趟经 `python3 scripts/gate/test-targets-parallel.py --jobs <每台默认重活线程数> -- <cargo test 的参数>` 按测试目标并行起，`--jobs` 取登记的每台默认重活线程数（`.claude/heavy-thread-default`），第二个 `--` 之后的 `--nocapture`、`--exact --skip` 原样交给每个目标）；挂特性的测试文件跑到加复用 0 条的、测试找不到显卡时 GPU 对拍打的 `NOT_RUN_NO_GPU_ADAPTER` 逐条报「本次未跑」；另有 product-gpu-real-card-tests 一格（产品 GPU 包 `singlefs-background-gpu` 带 `tests-on-a-real-gpu` 特性才编的真卡对拍测试，`--test-threads 1`，每条测试函数按单元键复用，对拍判红整格红，测试找不到显卡打的 `NOT_RUN_NO_GPU_ADAPTER` 逐条报「本次未跑」，一个对拍目标跑到加复用 0 条的也报「本次未跑」）；要上卡的三处（登记的崩溃枚举用例里带 `gpu` 特性的命令、带 `gpu` 特性那一趟、product-gpu-real-card-tests 的对拍那一趟）开跑前都核环境里有服务交的卡（`GPU_CARDS_BY_PRIORITY`），服务没交卡的那一格判红、那一趟不起；登记的崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的行）逐条按崩溃状态块跑：块键相同、库里有判器结论与观察者结论都不红的复用，缺的当场判，有红块判红，跑完打 `CRASH_CASE_BLOCKS` 一行；登记行第三列写了 `red-blocks=judged-by-test` 的用例（故障注入之后的红由用例自己认下）按 `cargo test` 的退出码判、块那一行只核 `blocks = reused + fresh + refused`，库里的红块不判；按条复用那几趟里每条测试函数各是一条判定，键是它的单元键（测试函数的单元闭包、闭包真引到的 cfg 原子各带这一次编译的真值、release 档、工具链、解析出的闭包真引到的依赖版本；闭包没引到的 cfg 原子不进键），键相同的经 `--exact --skip` 跳过、只跑键对不上的，拒算的照跑、不复用，判红的照同一把键记红、撤掉这把键原有的绿、下一趟现判，成功句首「按条复用」：现判 N、复用 M、拒算 R。崩溃放量的判器整套上了 GPU，罩全部池级 checker 不变量（D13（验证路线） 已定项 15，`crates/singlefs-verification/src/crash_judge_gpu.rs`）。
- 崩溃放量入口 `bash scripts/crash/crash-amplification.sh`：不带参数跑登记表里全部分项（`--list-items` 列、`--item <分项>` 点名），GPU 清场由服务做、跑完不复原（照「凡用 GPU 先清场，跑完不复原」一条）；每项交 `crash-amplification-item.sh`，机器与卡由服务分，配置 `ENABLE_GPU=1` 时由 GPU 判，用不上时照 `.claude/rules/test-script-discipline.md`「用上现有的硬件加速」退 78（配置对不上、点名的卡用不了、第二台连不上、清场没过），算法上判不了的（有状态 GPU 判不了、内核展开超预算、建不出表）退 248，退出码表在入口与分项驱动的文件头；展开深度取引擎常量，缩域照「缩域」一条；崩溃放量、崩溃枚举用例、给定状态表与两种对拍的判定都在 prune-server 那一份库里（不随趟、不随判器换，测试的类只当索引；「判定库只有一份」）；复用与续跑照同一篇「判定可复用：按输入存，按前缀剪」：每块的键是块键（这一步的写表与起步镜像的摘要、段展开、判器另读的输入与段内序号）加判这一块的判器组的单元键（判器入口的函数单元闭包，WGSL 按 fn 切、去注释；闭包真引到的 cfg 原子各带这一次编译的真值；工具链与 cargo 解析后的构建档；`Cargo.lock` 里闭包真引到的外部包），闭包没引到的 cfg 原子不进键，带不带 `gpu` 特性编的两趟里闭包不提 `gpu` 的判器组键相同、判定共用这一份库；键的算法在 `crates/singlefs-verification/src/crash_identity.rs`（判器组的单元键由 `crates/singlefs-verification/build.rs` 经 `crates/singlefs-unit-graph` 按 `crates/singlefs-verification/src/judging_units_entries.rs` 算好编进二进制），存放布局在 `crates/singlefs-verification/src/verdict_store.rs`。

qemu-device-streams（每档）、lkmm（每份 litmus）、crates-mutation-replay（每行）、formal-verification（每条）、broad-mutation（每条变异）与 research-build-and-replay 的 experiment-replay 格（每行复跑）、mutation-tables 格（每条变异）各按判定入库：每条判定的键是它的单元键，键对得上的复用、对不上的现判；整道、整格没有复用判定，各道的键与它在同一份判定库里的键前缀写在它的文件头（库只有一份，「判定库只有一份」）。
除判定库服务 prune-server 与统一任务服务 task-server 自己的自证（门禁 code-tooling 的 prune-server-selftest 格这一类）、只读文本的静态检查格之外，凡起编译、跑自证、现算键的判定都这样按判定入库、按单元键复用剪枝、记红撤绿，没有每趟整格照跑的格；带判定库的门禁与入口登记在 `.claude/gate.d/verdict-library-scripts.tsv`。
`crash-point-replay` 按条复用那几趟、model-differential-and-scenarios 各模型对拍格、research-build-and-replay 的 research-unit-tests 格、experiment-replay 格（对不上的）与 mutation-tables 格（没被抓到或编不过的）、code-tooling 按条复用的各格（research-script-selftests、reuse-selftests、function-units-two-implementations-agree、mutation-list-items、entry-scripts 与两格开关复跑）、qemu-device-streams、lkmm、crates-mutation-replay、formal-verification 判红的照同一把键记红、撤掉这把键原有的绿，键记着红的下一趟现判、不当绿复用；broad-mutation 的驱动只算翻面（同一把键先抓到、后判没红）：不沿用、现判一次。这些判定都在 prune-server 那一份库里（「判定库只有一份」）：各道的库名是库根那棵仓的 git common-dir 加它自己的固定目录名，只当名字与命名空间，本地不建、不读、不写（广谱变异的种类固定是 `broad-mutation-units`，不取库根）；库根不进键，落在临时目录底下的另占一份临时命名空间（`scripts/prune-server/prune_server_client.py` 文件头「键空间」）；库根照 `scripts/crash/lib_verdict_library.py` 的 `library_root` 取：`SINGLEFS_VERDICT_LIBRARY_ROOT` 设了取它指的那棵仓（仓副本里判的设它指到主仓），没设取被判那棵树。

崩溃判定库的键不读仓根；断言就是关于真仓、要读活树的测试照旧拒算（照跑、不复用、不入库）。

- 都经 `task-server/task submit -- <命令>` 交；内存上限、哪台机器、哪几张卡、GPU 清场与排队都由服务定。
- 门禁红了必须修好或查清原因，红着不提交。要知道一趟有多少条要现判：门禁 `crash-point-replay` 用 `--list-items --with-keys`（逐条点名拒算的测试函数与理由），crates-mutation-replay 用 `--list-items --with-keys`，formal-verification 用 `--list-items --with-keys`；崩溃点重放入口 `scripts/crash/crash-point-replay.sh --list-items --with-keys` 按项打 `ITEM_PLAN` 行。
- 提交时只走一条路：先照 `.claude/skills/crash-test/commit-stages.md` 跑提交时的六道门禁（`crash-point-replay`、qemu-device-streams、lkmm、crates-mutation-replay、formal-verification、broad-mutation；broad-mutation 不指定范围那一趟自己先起广谱变异驱动），它们各按自己的键前缀把判定写进同一份判定库；再照 `.claude/skills/gate/staged-gate.md` 跑 `scripts/gate/gate-staged.sh` 整轮，整轮里这六道照键复用先跑那一趟写下的判定、不重判。
- 谁跑得了由 `.claude/hooks/project-code-guard.sh` 判（判定在 `lib_project_code_runs.py`）：只判经没经任务服务与认不出的写法——一切跑项目代码的命令只认经 `task-server/task submit` 起，直接起的由它的 ① 拒，要经服务的外部程序（`qemu-system-*`、`herd7`、`e152-file-system-benchmark`）同拒；子 agent 起 `task-server/task` 的 submit、wait、cancel、clean 由它的 ② 拒（`.claude/agent-common.md`「子 agent 不交任务」）。要核弄坏开关（`CRASH_POINT_REPLAY_BREAK`、`FORMAL_VERIFICATION_BREAK`、`GATE_MUTATION_BREAK` 这类 `*_BREAK`）下的判红，命令写进报告「要主 agent 跑的命令」一节交主 agent。派发提示里叫子 agent 跑项目代码的句子由 `.claude/hooks/runner-dispatch-guard.sh` 拒。
- 起任何验证（崩溃点重放与崩溃放量入口、门禁阶段的一道或几格、`cargo test` 点名的包与目标、合入后验证、实验复跑）的命令开头带范围声明 `SINGLEFS_RUN_SCOPE='种类=<whole|items|target>; 跑什么=…; 范围=…; 为什么=…'`（写在 `task-server/task submit` 前面），四段的写法与种类的判法以 `.claude/hooks/run-scope-guard.sh` 文件头为准：它在执行前拒没带声明的、缺段的、种类或范围数与命令形状对不上的、为什么太短的；子 agent 经 submit 交的一律拒，停下交回。改完只跑上次红的与这一次改到的包、目标、项或格（拒绝说明附上次红的清单，取自 `~/.local/share/singlefs/last-runs/`，每道门禁与入口跑完写），整轮留提交前；范围合不合适由主 agent 事后对着检出记录看。
- 带判定库的门禁与入口都登记在 `.claude/gate.d/verdict-library-scripts.tsv`，自证里各有一格 `reuse-three-questions`（判法在 `scripts/gate/lib_reuse_checks.py`：同一份输入第二趟现判 0、复用的恰好是头一趟现判的那几项；闭包外改动之后现判 0；闭包内改一个函数单元之后现判的恰好是闭包含它的那几项；少复用一条红、多现判一条红），门禁 code-tooling 的 reuse-selftests 格按登记表逐份核那一格的规范输出；每一趟真跑开跑前从键相同、库里有判定的项里随机抽几条当哨兵、再抽一条强制现判（种子打进日志），哨兵有一条被现判、强制现判的判定与库里那一条不同、或强制现判那一条仍复用，整道红。

**凡用 GPU 先清场，跑完不复原**，清场由服务做：崩溃放量入口与分项驱动、带 `gpu` 特性的 cargo（GPU 对拍、门禁带 `gpu` 特性那一趟）、手跑的 GPU 测试都算，调用方经 `task-server/task submit -- <命令>` 交，不在外面套清场。手跑的 GPU 测试编与跑一条交，不先另交 `--no-run` 只编的那一条；门禁与入口脚本照各自的起法。跑完、跑红或被打断都不复原。哪几张卡由服务分。清场回读没过不跑、退 78：卡上有不归清场白名单的计算进程时就是这样，等那个进程跑完再交，不停它。清场之前查卡或清场回读判出卡坏（清场那一行写 `result=card-fault`，服务把那一件的结局记成 `card-fault`，`task wait` 末行 `type=card-fault`）时，主 agent 当场弹窗请用户重置那张卡，不等、不重试。要复原，手动跑多机配置 `QUIESCE_START_COMMAND` 那一条私有复原脚本，再跑 `QUIESCE_STARTED_CHECK_COMMAND` 回读到它退 0。崩溃放量入口 `scripts/crash/crash-amplification.sh` 与门禁 `crash-point-replay` 的 feature-gated-tests 格自己都不清场、不挑卡，经服务交（服务在最外层清场、分卡），不在外面套。

- 指定范围的那几趟（点名测试目标、`--item`）崩溃状态数压在 20 万量级；不指定范围那一趟不管。几十万态以上的大域对拍，跑之前先报预计要多久。
- 单个 GPU 测试二进制只用分到的卡（哪几张卡由服务分），带 `--test-threads 1` 跑：门禁 `crash-point-replay` 带 gpu 特性那一趟与 `scripts/crash/crash-point-replay.sh` 开了 GPU 时带；崩溃放量的分片驱动 `scripts/crash/crash-amplification-two-hosts.sh`（`scripts/crash/crash-point-replay-shard-run.sh` 交给它）经服务分片时的各片与合并、不分片时本机整份那一趟，在共用库拼好的 `cargo test` 参数后开了 GPU 时另加；`scripts/gate/admission.py` 拼的 crash-case 命令里特性带 gpu 的也带；变异门禁 `scripts/mutation/crates-mutation-rows.py` 起的 `cargo test`（基线的批、变异、记覆盖的插桩跑）特性里有 gpu 的也带，`--` 之后行里自己已写 `--test-threads` 的不重复加。崩溃放量单机那几趟（`cargo test` 的参数由 `scripts/crash/crash-amplification-lib.sh` 拼）还没带，改成带时同时改这一条。CPU 一侧的并行写在用例里，用例自己起多线程。测试目标 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state` 除外：它的测试按显存额度排队（排队写在 `crates/singlefs-verification` 的用例里），门禁 `crash-point-replay` 带 gpu 特性那一趟（这个目标另起一趟）、`scripts/crash/crash-point-replay.sh` 按条复用那一段与 `scripts/gate/admission.py` 拼的 crash-case 命令都不给它带 `--test-threads 1`，测试线程取登记的每台默认重活线程数（`.claude/heavy-thread-default`）；除外名单在这三处各一份 `GPU_TEST_THREADS_RELEASED_TARGETS`，三处一起改；变异门禁 `scripts/mutation/crates-mutation-rows.py` 不另抄，直接取 `scripts/gate/admission.py` 的那一份，点名的测试目标全在名单里的才带登记值，别的 GPU 测试目标照旧带 `--test-threads 1`。
- GPU 对拍与内存对拍不压并发：用例内部的 CPU 线程数由用例自己读的变量定，GPU 对拍测试的 CPU 一侧读 `SINGLEFS_CRASH_JUDGE_CPU_THREADS`，没设读 `SINGLEFS_THREAD_CAP`，再没设取登记的每台默认重活线程数（`.claude/heavy-thread-default`）；崩溃放量与对拍驱动的机器、卡与内存显存额度由服务分。
- 哪几张卡由服务分；派活（哪件上哪张卡）与判法拆开。

## 判定库只有一份：全仓统一写进 prune-server 的那份 RocksDB

- 判定库的内容：剪枝按单元键复用时要读的键，就这几类——Rust 一侧 `crates/singlefs-verification/src/verdict_store.rs` 文件头登记的 28 张表（判定块、判定向量两张、违例、录入块、路径索引、事实表、核对表、判法单元组成清单、布局行、结论块与结论违例、恢复字、计数向量、块属性、录下的流程、一趟的清单、判表、给定状态表、片进度、观察者四张、被抛弃根、配对结论、读集见证两张）；Python 一侧是 `scripts/prune-server/prune_server_client.py` 文件头「键空间」点名的五族（verdict-library、unit-verdicts、mutation-units、broad-mutation、apparatus-items）各自种类下的条目。不进库的：运行记录（峰值表、占盘时序、上次红的清单）放多机配置 `NFS_SHARED_ROOTS` 点名的共享根（两台同一绝对路径，不经 `~` 与 `$HOME` 取：第二台的家目录是另一个） 下、两台读同一份；进仓的登记表随 git 走；产物、日志、临时现场归各自的目录。
- 全仓只有一份判定库：prune-server 自己的那份 RocksDB（服务状态目录下的 `library`，布局只一种：全部表在默认列族、键原样、开库选项照 `verdict_store.rs`）。各族各自的键前缀（Python 一侧 `scripts/prune-server/prune_server_client.py` 文件头「键空间」，Rust 一侧 `verdict_store.rs` 的表标签）。
- 一族「接完」= 三件按次序做完：① 它的入口与门禁缺省走服务，本地文件库那条代码路径删掉、不留开关；② 旧库不导入：那一族还要的判定由它的入口经服务现判、照键写回；③ 只删那一族旧库所在的目录，别的族的目录不碰。接完的族登记在 `.claude/gate.d/verdict-library-scripts.tsv` 第三列（写「prune-server」；一行脚本跨几族的，按族各写），哪族接完了以这一列现读为准：写了判定库的行第三列都是 prune-server（`scripts/experiment-apparatus/replay.sh` 与 `.claude/gate.d/research-build-and-replay.sh` 两行是 unit-verdicts 族的 replay-row 种类，读写只经 `scripts/gate/admission.py` 的 `library-files` 走服务，replay.sh 文件头「判定库」）；第三列不是 prune-server 的只剩「不算：」行（只提到别人的复用）与写「无判定库：待立」的 `.claude/gate.d/fixtures`（门禁样本档）一行；登记表射程外带判定库的脚本（E171（三oracle合谋测试）的 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py`、E203（准入之前扫类的步代价）的 `scripts/experiment-apparatus/e203-admission-scan-cost-run.sh` 这类）照样逐个登记一行，第三列写 prune-server。task-server 按这一列收任务：命令点到第三列不是 prune-server 的带判定库那一行的，提交时当场拒（`task-server/task_server.py` 文件头 ②「提交时拒」③），不按开关。各族都只走服务：再写旧库、或悄悄另建一份本地库，都算违例。
- 旧库不原地打开、不并存、不导入：那一族接完之后旧库由主 agent 删掉，删法照「接完」第 ③ 件。
- 不许任何测试、门禁、入口脚本、实验装置、agent 新建自己的判定库：另开一个目录存判定文件、另写一张 TSV 或 sqlite、另开一份 RocksDB 都算。中转库不算另开：`scripts/gate/admission.py library-files` 给 replay.sh 与 E182（被抛弃根读不出的三条岔路）两份长跑脚本搬判定用的中转目录这类只在并回服务那一份之前存在、并回即删的库；并不进去的，由开它的那一趟报红、不留着。自证与压测一律对自己起的临时服务，格完即删，不写真服务，也不拿真任务当测试。要存判定就经 prune-server 的客户端写进那一份（Python `scripts/prune-server/prune_server_client.py`，Rust `prune_server::reliable_client`）。
- 不设开关：读写判定库只有经服务这一条路；交任务不带任何选库的变量。服务连不上退 75，不退回本地文件。
- 指定范围与不指定范围是同一套脚本、同一份库，只差范围：判出来的判定都写进同一份库、按同一把单元键复用。
- 这一条门禁还没判（哪个脚本往别处写了判定），欠账 C948 在 `.claude/kb/checks-owed.md`；立前靠 review：审改动时见到新的判定存放处就打回。

## 所有任务都要多机可用，不许绑定某一台

- 没有一个任务是必须绑在一台机器上的：测试、门禁、入口脚本、实验装置、服务的控制命令，派到两台里任何一台都要跑得起、判得对，判出来的结果两台都读得到（判定进那一份库，运行记录与产物放 `NFS_SHARED_ROOTS` 点名的共享根）。派到某一台就红的，是缺陷，当场修，不登例外。
- `task-server/task_server.py` 没有只派一台的名单（文件头 ② 排队的「机器」）：任何命令都派得到两台，哪台由余量定；派到某一台红的由脚本那一侧修，不往服务里加只派一台的名单，也不钉台。
- 不读会改变判定结局的本机独有状态；许读的本机私有状态只有 `~/.config/singlefs/` 下的私有名单与密钥，多机配置 `multi-host.env` 两台用同一份（仓根 `multi-host.env.example` 文件头）；不写死主机名与地址；环境相关的一切只由多机配置定（没写就是没有，不补缺省）。一条测试自己的配置文件（QEMU 的 `qemu.env`、预编库的 `rocksdb-prebuilt.env` 这类）是那条测试的事，与多机无关：进仓或放共享根，两台都读得到；只在一台有的，是那条测试的缺陷。判本台环境的门禁格（机器干净不干净、本机私有名单）两台结局本来不同，不算。
- 这一条门禁还没判，欠账 C949 在 `.claude/kb/checks-owed.md`；立前靠 review 与服务派到第二台的那几趟。

## 函数名与类型名不许只由空泛词拼成

`run_cell`、`CellRun`、`check_cell`、`fn run`、`fn get` 这一类名字判违规：每一个词都在 `.claude/naming-vague-words` 里，名字就什么都没说。改成说出跑的是什么、判的是哪条、取的是哪个量（`evaluate_stream_domain`、`count_named`、`perform_commit_step`）。`impl <trait> for <类型>` 块里 trait 规定的方法名不判。门禁 code-source-discipline 的 vague-names 格判；还没改完名的文件登记在 `.claude/naming-vague-exclude`，只缩不涨。

## 崩溃一致性只能靠崩溃点重放验证

把块层所有写请求记下来，在**每一个**可能的崩溃点截断、重放、跑池级 checker 与记录核对器。**没跑过这个的写路径就不算验过**：单测全绿说明不了崩溃一致性。

## 功能正确性靠模型对拍

在内存里维护一个只管语义、不管性能也不管崩溃的理想文件系统（`crates/singlefs-verification/src/model.rs`），把同一串随机操作分别施加到模型和实现上，比结果。这是这个项目唯一的功能对照物。

## checker 即规范

不变量清单（`crates/singlefs-checker/invariants/`）每加一条，池级 checker 就加一个检查。**「这个格式到底是什么」，答案以 checker 的源码为准，不是文档。**

## 文件系统特有的反推缺口

- **「测试全绿」不等于「实现正确」。** 要说「崩溃一致性成立」，先说清这一趟本来撞不撞得上：枚举了多少个崩溃点，是不是全部。
- **「checker 没报错」不等于「镜像是好的」。** 说这句话之前，先看 `crates/singlefs-checker/invariants/` 里对应那条的实现状态。

## 门禁的结构

每道门禁都能全跑，也能逐格点名跑。

- 每格一个函数，判一件事。门禁 source 共用库 `.claude/gate.d/lib/stage-cells.sh`（住在 `lib/` 下，`gate.sh` 不把它当阶段跑），用 `stage_cell <格名> <函数> <判什么的一句> <判红时的出路>` 逐格登记；参数解析、样本标记、逐格汇总都经它，不各写一份。写法在它的文件头，样板是 `.claude/gate.d/doc-decisions.sh`。门禁 `crash-point-replay` 不 source 它，参数照同一张表自己解析。
- 文件头写格名表：每格一行 `# gate-cell: <格名> <判什么的一句>`，与登记的逐格相同、次序相同。

| 参数 | 做什么 |
|---|---|
| 不带 | 全部格按登记次序跑 |
| `--list` | 逐行打「格名、制表符、判什么」，不跑任何格，也不起任何重活 |
| `--check <格名>[,<格名>…]` | 只跑点名的格 |
| 格名写错、`--check` 后面缺格名 | 列出可用的格名，退 2 |

其余参数（项目根、`--force`、门禁自己的选项如 `--write`）原样交给门禁。

- 汇总：每格在自己的子 shell 里跑，汇总逐格列「格名：绿 / 红 / 本次未跑」。有一格红整道退 1，红格后面接它自己登记的出路与只重跑红格的 `--check` 命令；跑到的格全退 77 整道退 77；否则退 0。退 77 的格逐格报进 `$GATE_NOT_RUN_FILE`。
- 样本要证某一格时，在样本根放 `.gate-cells`（一行一个格名），门禁只跑点名的格；被判的根是门禁所在的仓、或是 `gate.sh --staged` 的临时树时，这个文件不认。
- `--list` / `--check` 只管格，与文件头 `# gate-cell:` 表逐格相同。按项判的门禁逐项点名另用两个参数：`--list-items` 逐行打可点名的项，不编、不跑；`--item <项>` 只跑点名的项，给几次取并集，项写错退 2 并列出可点的项。可点的项：`crash-point-replay` 按崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）、qemu-device-streams 按模式、research-build-and-replay 的 experiment-replay 格按实验、crates-mutation-replay 按变异行、formal-verification 按条（`.claude/gate.d/verus-pins.tsv` 登记的各 crate 逐个判；证明每个有证明义务的单元按全路径 `<crate>::<模块>::<名>` 一项，impl 与 trait 里的方法写 `<类型>::<方法>`、`<trait>::<方法>`：proof fn、exec fn、带 `decreases` 的 spec fn 与 `derive(Clone)` 生成的 `<类型>::clone`，不带 `decreases` 的 spec fn、trait 里没有函数体的声明与 `external_body` 的 fn 不成项、随用到它的项进键，整格跑时每个 crate 各项验过的义务之和等于登记表里它那一行登记的数；证明整格、工具链、钉全各一项；必红世界每行、空真每处 `requires`、红例钉表每行各一项；点名单跑一项与它在整格里跑的判定相同；每条的单元键——证明项是被验函数单元连同它 `requires`、`ensures`、`decreases` 与函数体用到的 spec fn、proof fn、类型、常量的闭包，钉表每行是那一行档与参数重放真走到的装置函数单元闭包，连同状态输入与期望逐态行——与判定库写在它的文件头「复用」一段）。
- `scripts/gate/gate-structure-check.py` 判 `.claude/gate.d/` 顶层每道门禁（共用库在 `lib/` 下，不判）：文件头有格名表；`--list` 退 0、输出与格名表逐格相同、跑的时候没起 cargo、QEMU、herd7 这类重活；`--check` 不存在的格退 2。
- 阶段归属表 `.claude/gate.d/stage-owners.tsv` 一行一个门禁与格名：门禁、格名、判红时派给谁修、为什么。

## 门禁管哪一半

| 判什么 | 谁判 |
|---|---|
| 崩溃点重放跑了、包里全部测试函数按单元键复用或现判都绿、每条登记用例的崩溃状态块在崩溃判定库里有单元键相同、不红的判定；挂特性的验证包测试文件在不指定范围那一趟至少编进来一次、跑到或复用的用例不为 0 | 门禁 `crash-point-replay`（后一半是 feature-gated-tests 格）；覆盖声明 `# gate-covers: 崩溃点重放`，键登记在项目根 `.claude/gate-not-implemented.tsv` |
| 形式化验证：恢复重放取前缀、择根、扫环三截的核与择根取前缀的合判推论经 Verus 验过，E165 的红例都钉进钉表、钉表自己成形、在被判那棵树的实现上逐行重放与原实现相同 | 门禁 formal-verification：verus-toolchain-pinned、verus-proof、verus-vacuity、verus-must-red-worlds 四格用 Verus 逐个 crate 验登记表 `.claude/gate.d/verus-pins.tsv` 的 crate 行点名的被验区域；verus-toolchain-pinned 另核 Verus 与产品的 rustc 版本与登记表的 `rustc` 行相同；verus-proof 起 Verus 之前照信任清单 `.claude/gate.d/verus-trust-list.tsv` 判被验区域里的可信声明与 cfg 原子（逐条登记；axiom、作弊标记与 `verus!` 外面被验类型的 trait impl 判红），整格跑时另比产品构建与 `verus --compile` 构建的 MIR 逐条目相同；红例钉表 `.claude/gate.d/e165-pinned-red-examples.tsv` 两格：pinned-red-examples-table 只判钉表自己（每行 22 段、指纹、抄来的段、档的写法、点名的变异还在，不读红例账、产物与 `research/`），pinned-red-examples-replay 逐行重放；钉全由 E165 第二段驱动 `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` 写完一对红例账与产物时当场判（追加、再 `--check` 同一对，没钉全那一趟判红；它文件头「钉红例」一段） |
| 广谱变异：此刻源码上每一条变异都有单元键相同的结论、没有一条没红 | 门禁 broad-mutation：gate-verdict 格判驱动 `--summary` 的 `BROAD_MUTATION_GATE` 行与汇总行，驱动错、只过了第一段算没结论，驱动 `--summary` 退 1（判定库或清单表对不上、结局认不出）照判红；驱动不给任何一步设内存上限、不限时，撞上限与卡住由统一任务服务管，被停的那一条不落行、不算抓到（`scripts/mutation/broad-mutation.py` 文件头「内存与限时」）；判定在 prune-server 里（broad-mutation 族的 `verdicts/` 等七张表，驱动文件头「判定库」），判定行不带校验列；不指定范围那一趟 gate-verdict 格自己先起驱动剪枝跑，`--check gate-verdict` 指定范围时不起、只读库 |
| 模型对拍跑了、每段都判过 | 门禁 model-differential-and-scenarios 以 `model-` 起头的那几格；覆盖声明 `# gate-covers: 模型对拍`，键登记在项目根 `.claude/gate-not-implemented.tsv` |
| 崩溃枚举用例住在验证包的 `tests/`、不标 `#[ignore]`、登记进崩溃点重放合表 `.claude/gate.d/crash-point-replay-items.tsv`（点名 `crash-case:` 行或放量分项）；验证包测试文件照 `scripts/crash/crash-case-check.py` 的 modules 那一样声明模块 | 门禁 `crash-point-replay` 开跑前跑 `scripts/crash/crash-case-check.py --only placement,modules`，红了不起 cargo；门禁 code-tooling 的 research-script-selftests 格跑那份脚本的自证 |
| 函数名与类型名不只由空泛词拼成；测试文件不按里程碑起名 | 门禁 code-source-discipline：vague-names 格读词表 `.claude/naming-vague-words`、还没改完的文件 `.claude/naming-vague-exclude`；test-file-names 格跑 `scripts/crash/crash-case-check.py --only file-names` |
| 池级 checker 库不依赖实现；RefFS 的依赖闭包只有自己与 `singlefs-format`、源码零处引实现与池级 checker 与验证包（真名与改名别名都算）、实现与池级 checker 的依赖闭包里没有它 | 门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格（⑤⑥⑦ 判 RefFS 那三样；RefFS 包不在时 ⑤⑥ 报本次未跑） |
| 判器调用点（池级 checker、记录核对器、GPU 判器）落在判定段里，或包住它的函数带 `// gpu-exempt: <截名>` 或 `// cpu-reference-for-gpu-comparison` 标记；两样都不是的调用点判红 | 门禁 checker-independence-and-sync 的 cpu-judging-sites 格：判法 `scripts/crash/cpu-judging-sites-check.py`，判定段与判器本体登记在 `.claude/gate.d/cpu-judging-sites.tsv`。标记写在包住调用点的那个函数前面：属性、文档注释与签名之前紧挨着的 `//` 注释行，或属性与签名之间的注释行；外层函数带了标记，里面嵌套的函数一并算；写在函数体里的不算 |
| 验证包一条用例一个场景 | 门禁 model-differential-and-scenarios 的 one-scenario 格（跑 `scripts/crash/crash-case-check.py --only one-scenario`，读 `crates/singlefs-verification/tests/` 的用例源码，不跑测试） |
| 变异行点名的测试跑得到：`--` 之后的筛选词筛得到点名的测试 | 门禁 code-source-discipline 的 mutation-tables 格 |
| 每道阶段认第一个参数当项目根 | 门禁 code-tooling 的 stage-owners 格 |
| 跑项目代码的命令经没经任务服务、写法认不认得出 | `.claude/hooks/project-code-guard.sh`，判定在 `lib_project_code_runs.py`：不经 `task-server/task submit` 直接起的会编的 cargo、编出来的代码、本仓的代码（门禁阶段与格、入口、实验装置与复跑、变异、各脚本的 `--selftest` 与样本档与弄坏开关）与要经服务的外部程序，执行前拒；子 agent 起 `task-server/task` 的 submit、wait、cancel、clean 执行前拒 |
| 登记的自证与样本档不起真工具 | `scripts/gate/selftest-process-guard.py` 盯 `.claude/hooks/selftest-registry.tsv` 登记的自证与样本档跑的整棵进程树，起了真 cargo、rustc、`qemu-system-*`、herd7、verus 就判红 |
