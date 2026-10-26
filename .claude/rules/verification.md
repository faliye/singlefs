# 验证纪律：harness 档随时跑，checker 档默认只在提交时跑

**这是 singlefs 的项目本地规则**，不在共享 SOP 里——它说的是这个仓的验证代码住在哪、什么时候跑。
共享规则在 `.claude/singlefs-ai-sop/rules/`。

## 定义与名字

五个包，五个名字，全仓只用这五个名字称呼它们：

| 名字 | 包 | 装什么 | 什么时候跑 |
|---|---|---|---|
| **harness 档** | `crates/singlefs-harness` | 单元测试与集成测试，加上它们用的脚手架库：录制器、内存池（`memory_pool`）、理想模型、随机历史、故障注入设备、场景 | 改了代码随时跑 |
| **checker 档** | `crates/singlefs-checker-tier` | 崩溃态枚举引擎与记录核对器（`crash`）、崩溃枚举用例经判定库的两段（流先入库、再从库里读回判，判在 GPU 或 CPU）与它的计数（`crash_state_enumeration`）、崩溃点重放发现日志与工具链身份（`crash_point_replay_progress`）、崩溃全量积压登记表的读写、核法与取件（`crash_full_run_backlog`；表在 crate 根 `crash-full-run-backlog.tsv`，崩溃点重放入口 `--backlog` 与门禁 code-tooling 的 crash-full-run-backlog 格经小程序 `crates/singlefs-checker-tier/src/bin/crash_full_run_backlog.rs` 读写）、一份长期判定库与两段协议（`crash_library`；块键只由内容定，测试的类只当索引）、崩溃注入与坏盘输入两场战役（随机历史两项的给定状态表、它的判器 trait 与 CPU 就地判在 `crash_given_states`）、可写挂载每个写落在哪一段的划分与清扫段的写（`writable_mount_phases`）、真设备一侧的设备日志比对与运行模式、崩溃放量（崩溃点身份与复用键 `crash_identity`、先录后核的流水线 `crash_amplification`、判定存储 `verdict_store`、单元校验和上 GPU `gpu_unit_checks`、三段流第 ① 段的事实表与 CPU 参照核对 `crash_facts`、第 ② 段的 GPU 核对内核 `crash_verify_gpu`、GPU 判器的输入表 `crash_judge_tables`、GPU 判器的判法 `crash_judge_gpu` 与它的派活 `crash_judge_dispatch`；`crash_amplification` 与 `verdict_store` 挂特性 `verdict-store`，`gpu_unit_checks`、`crash_verify_gpu`、`crash_judge_gpu` 与 `crash_judge_dispatch` 挂特性 `gpu`）、实验与真设备装置二进制（`src/bin/`），以及它们的全部用例（崩溃点重放、形式化验证、crates 变异表三样另分轻型与重型，轻型照「两档：轻型与重型」跑）；连同 QEMU（门禁 checker-tier-qemu-device-streams）、herd7（checker-tier-lkmm）、crates 变异整表（checker-tier-crates-mutation-replay）、全部实验复跑（checker-tier-research-build-and-replay 的 experiment-replay 格）、形式化验证（checker-tier-formal-verification：Verus 验登记表 `.claude/gate.d/verus-pins.tsv` 点名的各 crate，`crates/singlefs-replay-prefix` 与 `crates/singlefs-core` 的被验区域，E165 红例钉表两格）、广谱变异（驱动 `scripts/mutation/broad-mutation.py`；提交门禁 checker-tier-broad-mutation 判此刻源码上每一条变异都有单元键相同的结论、没有没红） | 默认只在提交时跑；想单独跑带 `SINGLEFS_HEAVY_TESTS=user-request` |
| **池级 checker** | `crates/singlefs-checker` | 判一个镜像的判决器库（O2，`check_pool_image`，D13（验证路线） 已定项 7）；只依赖 `singlefs-format` | 它是库，被两档调用；它自己的库单测归 harness 档，随时跑 |
| **RefFS** | `crates/singlefs-reference-filesystem` | 语义级的参照文件系统：只在内存里按 POSIX 操作走，判一次崩溃之后恢复读出的内容落不落在允许集合里；依赖只有 `std`，不引实现、池级 checker、harness 档、checker 档，实现与池级 checker 也不依赖它（门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格 ⑤⑥⑦ 判） | 它是库，被 harness 档与 checker 档调用；它自己的单测归 harness 档，随时跑 |
| **函数单元图** | `crates/singlefs-unit-graph` | Rust 与 WGSL 源码按条目切成单元、取去注释指纹、从入口收闭包、算复用键的库，只用 `std`（D13（验证路线） 已定项 17 的键算法住在这里）；归测试基础设施、checker 档依赖的一侧，不是产品代码 | 它是库，被 checker 档的构建脚本与门禁脚本调用；它自己的单测随时跑 |

归类判据：一条测试或一段代码要枚举崩溃状态、要真设备或外部工具、或跑一次以十分钟计，归 checker 档；否则归 harness 档。拿不准的放 checker 档；要不要挪回是设计判断，照 `.claude/rules/three-way-inference.md` 开推论轮判。
依赖只许一个方向：checker 档依赖 harness 档与池级 checker，harness 档不依赖 checker 档（它的依赖闭包里没有 `singlefs-checker-tier`，dev-dependencies 也算；源码里零处引它；门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格判）。两档都要用的一小段代码，宁可各留一份（例：`crates/singlefs-harness/tests/common_corrupted_allocation_record/mod.rs` 抄自 checker 档的坏盘输入），也不让 harness 依赖 checker 档。
测试文件按它测什么起名：领域在前、场景在后（`rollback_by_a_forward_publish`、`crash_enumeration_fixed_script_stream`），checker 档的以模块起头（`crash_enumeration_`、`crash_points_`、`crash_injection_`、`bad_disk_input_`、`record_checker_`）；名字不带来历照 `.claude/rules/test-script-discipline.md`「用例原子化：每一条都能单独跑」，本仓判的形态是以里程碑序数起头的、`*_step_four_*`、`*_supplement_two_*`、`*_parallel_line_one_*`、`*_c519_*`。`scripts/crash/crash-case-check.py` 的 file-names 那一样判，门禁 code-source-discipline 的 test-file-names 格在真仓上跑它。
不许用的叫法：「checker 包」（分不清是池级 checker 还是 checker 档）、「放量用例」「验证档」（说 checker 档）。

## harness 档里再分快档用例与慢档用例

- 轻：每次改完跑，`cargo test -p singlefs-harness` 不带 `--ignored` 跑到的全部。
- 红了只重跑红的那几条（`.claude/rules/test-script-discipline.md`「用例原子化：每一条都能单独跑」），快档用例与慢档用例一样：`python3 scripts/gate/rerun-failed-tests.py <上一趟的日志> -p <包>` 按测试目标打印只跑那几条的命令（一律带 `--include-ignored`），经 Bash 起、加内存包装；它判红、一条命令都不打时才整份重跑，完整的一遍归提交时的整轮门禁。
- 慢档用例：debug 下单条跑到 60 秒及以上的用例，标 `#[ignore = "harness 慢档用例：…"]`；要跑随时跑，一律经 `scripts/hosts/run-with-memory-cap.sh`，线程数：子 agent 照派发提示的「线程上限：N」，`scripts/hosts/capped.sh default`（登记的每台默认重活线程数）只给主 agent 与脚本内部用。单条用时看平常跑的时候量到的，不另外单独量。
- 调全量崩溃枚举函数（`enumerate_crash_states` 一族，快档 `quick_tier` 那几个除外）或自己逐个造崩溃状态（名字带 `every_crash`；循环里对录制操作取到循环变量为止的前缀去 `apply`、或造 `CrashImage`），直接这样做或经同一文件里的函数这样做的测试不是 harness 档的重，它是 checker 档，写进 `crates/singlefs-checker-tier/tests/`；`scripts/crash/crash-case-check.py` 的 placement 那一样判这一条，写在别的包里判红，门禁 `checker-tier-crash-point-replay` 开跑前在真仓上跑它。
- 一条用例一个场景照 `.claude/rules/test-script-discipline.md`「用例原子化：每一条都能单独跑」：本仓按「每一轮新建一个池」认参数循环（`build_pool`、`build_through_*`、`format_pool`、`MemoryPool::with_devices`），确是一个场景的在用例上面写一行 `// harness-test-granularity:one-scenario <理由>`。`scripts/crash/crash-case-check.py` 的 one-scenario 那一样判，门禁 harness-model-differential-and-scenarios 的 one-scenario 格在真仓上跑它。

## checker 档自己分快档与全量

| 档 | 命令 | 什么时候 |
|---|---|---|
| 快档 | `cargo test --release -p singlefs-checker-tier --lib --tests` 不带 `--ignored`（库、集成测试与装置二进制 `src/bin/` 的内联单测：`--tests` 带上全部 `test = true` 的目标，装置二进制都是），再一趟 `cargo test --release -p singlefs-checker-tier --features <tests/ 里 cfg 点到的特性> --lib --test <挂特性的测试文件>…`（`BINDGEN_EXTRA_CLANG_ARGS` 取 `scripts/crash/crash-amplification-lib.sh` 的 `CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS`；特性里有 `gpu` 的这一趟经 `scripts/crash/gpu-quiesce-run.sh` 起）；门禁 `checker-tier-crash-point-replay` 跑这两趟（feature-gated-tests 格：挂特性的测试文件跑到加复用 0 条的、没卡时 GPU 对拍打的 `NOT_RUN_NO_GPU_ADAPTER` 逐条报「本次未跑」），再逐条核 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的用例：用例带 `SINGLEFS_CRASH_CASE_BLOCKS_QUERY=1` 起、只查库不判，每个崩溃状态块在崩溃判定库里都有单元键相同、不红的判定才算罩住，有红块判红，别的报「本次未跑」；登记行第三列写了 `red-blocks=judged-by-test` 的用例（故障注入之后的红由用例自己认下）不查库，照全量起 `cargo test` 按退出码判、块那一行只核 `blocks = reused + fresh + refused`（`--full` 同一套判法，库里的红块不判）；两趟里每条测试函数各是一条判定，键是它的单元键（测试函数的单元闭包、闭包真引到的 cfg 原子各带这一次编译的真值、release 档、工具链、解析出的闭包真引到的依赖版本；闭包没引到的 cfg 原子不进键），键相同的经 `--exact --skip` 跳过、只跑键对不上的，拒算的照跑、不复用，成功句报「现判 N、复用 M、拒算 R」 | 整轮门禁与每次提交 |
| 全量 | `checker-tier-crash-point-replay` `--full`：在 HEAD + 暂存区的 worktree 里逐条跑登记的崩溃枚举用例（`--include-ignored --exact`），每条用例的崩溃状态块按单元键在崩溃判定库里复用、键对不上的块现判（再带 `--start-over` 不读库、全部块现判，现判的照键写回同一份库）；分层照 D13（验证路线） 已定项 9；崩溃放量的判器整套上了 GPU，罩全部池级 checker 不变量（D13（验证路线） 已定项 15，`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`） | 用户要求 |
| 崩溃放量全量 | `bash scripts/crash/crash-amplification.sh`：不带参数跑登记表里全部分项（`--list-items` 列、`--item <分项>` 点名），开跑经 `scripts/crash/gpu-quiesce-run.sh` 照多机配置清场、跑完不复原（照「凡用 GPU 先清场，跑完不复原」一条）；每项交 `crash-amplification-item.sh`：配置开了双机就两台跑、没开就一台跑，配置 `ENABLE_GPU=1` 时由 GPU 判，显卡照多机配置的 `GPU_CARDS_BY_PRIORITY` / `GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED` 的先后与额度用，用不上时照 `.claude/rules/test-script-discipline.md`「用上现有的硬件加速」退 78（配置对不上、点名的卡用不了、第二台连不上、清场没过），算法上判不了的（有状态 GPU 判不了、内核展开超预算、建不出表）退 248，退出码表在入口与分项驱动的文件头；不带参数按入口登记的全量展开深度跑（`FULL_EXPAND_UP_TO`），给了 `--expand-up-to` 的是缩过规模的一趟；崩溃放量、崩溃枚举用例、给定状态表与两种对拍共用一份崩溃判定库 `${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}/crash-state-verdicts`（不随趟、不随判器换，测试的类只当索引），落在开机会清空的目录里被拒；复用与续跑照同一篇「判定可复用：按输入存，按前缀剪」：每块的键是块键（这一步的写表与起步镜像的摘要、段展开、判器另读的输入与段内序号）加判这一块的判器组的单元键（判器入口的函数单元闭包，WGSL 按 fn 切、去注释；闭包真引到的 cfg 原子各带这一次编译的真值；工具链与 cargo 解析后的构建档；`Cargo.lock` 里闭包真引到的外部包），闭包没引到的 cfg 原子不进键，带不带 `gpu` 特性编的两趟里闭包不提 `gpu` 的判器组键相同、判定共用这一份库；键的算法在 `crates/singlefs-checker-tier/src/crash_identity.rs`（判器组的单元键由 `crates/singlefs-checker-tier/build.rs` 经 `crates/singlefs-unit-graph` 按 `crates/singlefs-checker-tier/src/judging_units_entries.rs` 算好编进二进制），存放布局在 `crates/singlefs-checker-tier/src/verdict_store.rs` | 用户要求 |

checker-tier-qemu-device-streams（每档）、checker-tier-lkmm（每份 litmus）、checker-tier-crates-mutation-replay（每行）、checker-tier-formal-verification（每条）、checker-tier-broad-mutation（每条变异）与 checker-tier-research-build-and-replay 的 experiment-replay 格（每行复跑）、mutation-tables-compile 格（每条变异）各按判定入库：每条判定的键是它的单元键，键对得上的复用、对不上的现判；整道、整格没有复用判定，各道的键与判定库写在它的文件头。

崩溃判定库的键与库目录都不读仓根：库目录由家目录定，入口脚本与双机驱动另交的库目录（环境变量 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY`）只收绝对路径，给相对路径的拒；断言就是关于真仓、要读活树的测试照旧拒算（照跑、不复用、不入库）。

## 两档：轻型与重型

崩溃点重放、形式化验证、crates 变异表三样各分轻型与重型；harness 档不在这两档里，照「定义与名字」随时跑。

| 档 | 守什么 | 跑法 | 谁跑、什么时候 |
|---|---|---|---|
| 轻型 | 两个目标：守住系统核心代码不红（核心部分，每次都跑）；测这一次改到的代码（可变部分，跑的人按自己改到的文件点名） | 三条命令带 `--quick`：`bash scripts/crash/crash-point-replay.sh --quick`（只用本机、不动第二台时加 `--local-only`）、`bash .claude/gate.d/checker-tier-formal-verification.sh --quick <根>`、`bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --quick <根>`。只带 `--quick` 跑核心部分；再带 `--item <项>`（可给几次）跑核心部分加点名的项，可点名的项用各条的 `--list-items` 列；点名的项在轻型的域上跑（崩溃取「甲二」与展开 6 两个域的并集，形式化与变异就是那几项本身）。跑全部的项与域；放量那一段经崩溃判定库复用块键与判器单元键都相同、已判过的块（与重型同一份库），不只跑一部分 | 形式化那一条：实现员、调查员交回前跑；崩溃与变异两条：子 agent 不起，命令写进报告「要主 agent 跑的命令」一节，主 agent 在实现员的副本里跑；合入后验证三条的全部由主 agent 跑；三条带不带 `SINGLEFS_HEAVY_TESTS` 前缀都放行 |
| 重型 | 总门闸：验全体 | 同三条命令不带 `--quick`，三种跑法：不带参数是剪枝（只剪单元键相同、已判过的与等价的；单元键只由判定依赖的函数单元各自去注释后的指纹、闭包真引到的 cfg 原子与它们的真值、工具链、解析出的闭包真引到的依赖版本与判定自己的内容输入组成，路径、名字、行号、资源上限、限时、提交与树不进键；两种排除：文件级是 `scripts/crash/lib_verdict_library.py` 的 `NOT_JUDGING_FILES` 那两份，管怎么存、怎么算键，不管判对错，它们的单元不进任何一份判定的键，crates 变异判法除外：crates 变异判法不在文件级排除的射程里，那里两份照进判法单元指纹、覆盖号带判法单元指纹，边级是只打提示、不改判定结局的子进程边，按实测排，不沿它收，登记在 `scripts/gate/lib_function_units.py` 的 `EXCLUDED_PROGRAM_EDGES`，边级排除的三格自证在 `.claude/rules/implementation-workflow.md`「复用重型门禁的判定：按每条判定的单元键判」一节；python 模块有单元进闭包时，它在 import 时执行的模块级语句一起进闭包：import、赋值、类头本身（不连方法）、带装饰器或缺省值的 def 头；D13（验证路线） 已定项 17）、`--item <项>` 是逐条、全量是不读判定库、现判出的判定照键写回（崩溃点重放入口带 `--full`；形式化与变异两道门禁不认 `--full`，设 `SINGLEFS_GATE_FULL=1`，变异那一道也认 `GATE_MUTATION_START_OVER=1`） | 只由主 agent 带前缀跑：提交时，或用户要求时 |

- 核心部分取什么：形式化是工具链、证明、钉表三项（钉表那一项只判红例钉表 `.claude/gate.d/e165-pinned-red-examples.tsv` 自己，不读 `research/`）；crates 变异表是改坏对象在 `crates/singlefs-core/src/` 的那些行里每个测试目标固定一行（登记表 `.claude/gate.d/crates-mutation-light-rows.tsv`）；崩溃是每项在「甲二」与展开 6 两个域的并集上判，连同小输入上的 GPU 与 CPU 对拍、剪枝自证、钉回来的红例（两个域的写法以 `scripts/crash/crash-point-replay.sh` 文件头为准）。重型是全部：形式化的全部项、变异整表、崩溃整域。
- 轻型是重型的子集：轻型里有的每一项，重型里都有。写测试先写重型，再把其中关键的步骤拿出来放进轻型；轻型登记的每一行写明取自重型的哪一项，指得到重型里现存的那一项。
- 三条都经 `scripts/hosts/run-with-memory-cap.sh` 起；带 GPU 的照「凡用 GPU 先清场，跑完不复原」一条经 `scripts/crash/gpu-quiesce-run.sh` 起，要卡时照清场规则排队等卡，不抢在跑的重型。
- 重型红了必须修好或查清原因，红着不提交。提交时用哪种跑法由用户在弹窗里选；选逐条的，主 agent 列出有判定的单元键对不上的项给用户点（键对不上的条数：门禁 `checker-tier-crash-point-replay` 用 `--list-items --with-keys`，checker-tier-crates-mutation-replay 与 checker-tier-formal-verification 的 `--list-items` 末了报；拒算的测试函数照跑、不复用，`checker-tier-crash-point-replay` 的 `--list-items --with-keys` 逐条点名拒算的测试函数与理由），没点到的报「本次未跑」。新入口接进提交那一趟之前，提交时照「checker 档自己分快档与全量」一节跑。
- 提交时只走一条路：先照 `.claude/skills/crash-test/commit-stages.md` 跑六道重型门禁（checker-tier-broad-mutation 之前照它第 2b 步起广谱变异驱动），它们把判定写进各自的判定库；再照 `.claude/skills/gate/staged-gate.md` 跑 `scripts/gate/gate-staged.sh` 整轮，整轮里这六道照键复用先跑那一趟写下的判定、不重判。
- 轻型保留到事务功能做全；之后由用户判稳定，再并进重型。
- 谁跑得了哪一档由 `.claude/hooks/heavy-test-guard.sh` 判（认轻型的判法在 `.claude/hooks/lib_heavy_tests.py` 的 `light_tier_command`）：三条带 `--quick`（可再带 `--item <项>`；崩溃点重放入口另可带 `--local-only`）的谁都能跑，子 agent 不经内存包装起的拒；夹了别的参数的、不带 `--quick` 的照重型拒；环境里设了该入口或该门禁自己文件头写的弄坏开关变量（`CRASH_POINT_REPLAY_BREAK`、`FORMAL_VERIFICATION_BREAK`、`GATE_MUTATION_BREAK` 这类 `*_BREAK`，命令前、`env` 里与前面 `export` 的都算，值非空才算设了）、或该文件头 `LIGHT_SCALE_VARIABLE=<变量名>` 表登记的改轻型规模的变量（例 `SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES`；名单只在文件头那张表里，判法现读它，不另抄）的 `--quick` 不算轻型，照重型拒（设了这两类变量时，`--quick` 的结果不作轻型证据）；别的门禁的开关与变量、值为空的不算。新增改轻型规模的变量，在入口或门禁文件头那张表里加一行。要核弄坏开关下的判红，命令写进报告「要主 agent 跑的命令」一节交主 agent。派发提示许可子 agent 跑轻型的放行，许可跑重型的由 `.claude/hooks/runner-dispatch-guard.sh` 拒。
- 起任何验证（轻型三条、重型、门禁阶段的一道或几格、`cargo test` 点名的包与目标、合入后验证、实验复跑）的命令开头带范围声明 `SINGLEFS_RUN_SCOPE='种类=<whole|items|quick|target>; 跑什么=…; 范围=…; 为什么=…'`，四段的写法与种类的判法以 `.claude/hooks/run-scope-guard.sh` 文件头为准：它在执行前拒没带声明的、缺段的、种类或范围数与命令形状对不上的、为什么太短的；子 agent 起 whole 一律拒，停下交回。改完只跑上次红的与这一次改到的包、目标、项或格（拒绝说明附上次红的清单，取自 `~/.local/share/singlefs/last-runs/`，每道门禁与入口跑完写），整轮留提交前；范围合不合适由主 agent 事后对着检出记录看。
- 带判定库的门禁与入口都登记在 `.claude/gate.d/verdict-library-scripts.tsv`，自证里各有一格 `reuse-three-questions`（判法在 `scripts/gate/lib_reuse_checks.py`：同一份输入第二趟现判 0、复用的恰好是头一趟现判的那几项；闭包外改动之后现判 0；闭包内改一个函数单元之后现判的恰好是闭包含它的那几项；少复用一条红、多现判一条红），门禁 code-tooling 的 reuse-selftests 格按登记表逐份核那一格的规范输出；每一趟真跑开跑前从键相同、库里有判定的项里随机抽几条当哨兵、再抽一条强制现判（种子打进日志），哨兵有一条被现判、强制现判的判定与库里那一条不同、或强制现判那一条仍复用，整道红。

**凡用 GPU 先清场，跑完不复原**，经 `bash scripts/crash/gpu-quiesce-run.sh -- <命令…>`：崩溃放量全量与分项驱动、带 `gpu` 特性的 cargo（GPU 对拍、快档带特性那一趟）、手跑的 GPU 测试都算。手跑的 GPU 测试先不清场 `--no-run` 编，编完再经它清场、只跑测试；门禁与入口脚本照各自的起法。它照多机配置清场并回读、跑；跑完、跑红或被打断都不复原。显卡额度用清场之后那一份。清场回读没过不跑、退 78：卡上有不归清场白名单的计算进程时就是这样，等那个进程跑完再起，不停它。要复原，手动跑多机配置 `QUIESCE_START_COMMAND` 那一条私有复原脚本，再跑 `QUIESCE_STARTED_CHECK_COMMAND` 回读到它退 0。崩溃放量入口、分项驱动与门禁 `checker-tier-crash-point-replay` 的 feature-gated-tests 格自己经它起，不在外面再套一层。

- 日常验证（harness 档与轻型三条）的崩溃状态数压在 20 万量级；这一条不管提交时跑的重型与用户要求的全量。几十万态以上的大域对拍，跑之前先报预计要多久。
- 单个 GPU 测试二进制只用本机点名的卡，带 `--test-threads 1` 跑：门禁 `checker-tier-crash-point-replay` 带 gpu 特性那一趟与 `scripts/crash/crash-point-replay.sh` 开了 GPU 的单机分支带；该脚本的双机分支只起 `scripts/crash/crash-amplification-two-hosts.sh`，它在共用库拼好的 `cargo test` 参数后开了 GPU 时另加；`scripts/gate/admission.py` 拼的 crash-case 命令里特性带 gpu 的也带。崩溃放量单机那几趟（`cargo test` 的参数由 `scripts/crash/crash-amplification-lib.sh` 拼）还没带，改成带时同时改这一条。CPU 一侧的并行写在用例里，用例自己起多线程。
- GPU 对拍与内存对拍不压并发：用例内部的 CPU 线程数由用例自己读的变量定，GPU 对拍测试的 CPU 一侧读 `SINGLEFS_CRASH_JUDGE_CPU_THREADS`，没设读 `SINGLEFS_THREAD_CAP`，再没设取登记的每台默认重活线程数（`.claude/heavy-thread-default`）；崩溃放量与对拍驱动照多机配置用两台与全部卡；只限一样：不让哪台的内存或显存撞顶。
- 显卡照多机配置的优先级先用满一张、再分下一张；派活（哪件上哪张卡）与判法拆开。

## 崩溃枚举用例住哪、怎么登记

- checker 档每个测试文件第一行写它测哪几个模块：`//! checker 档模块：<模块，按 crash、crash_state_enumeration、crash_point_replay_progress、crash_full_run_backlog、crash_injection、writable_mount_phases、crash_given_states、bad_disk_input、device_log、on_device_modes、crash_identity、crash_library、crash_amplification、verdict_store、gpu_unit_checks、crash_facts、crash_verify_gpu、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch 的次序用、隔开>`，与它从 `singlefs_checker_tier::` 导入的模块逐个相同；一个都不导入的写 `无（为什么）`。按模块找用例：`grep -l '^//! checker 档模块：.*crash_injection' crates/singlefs-checker-tier/tests/*.rs`。`scripts/crash/crash-case-check.py` 的 modules 那一样判，门禁 `checker-tier-crash-point-replay` 开跑前在真仓上跑它。

- 写在 `crates/singlefs-checker-tier/tests/<流的名字>.rs`，全量那条标 `#[ignore]`，同文件的快档用例不标。
- 共用的搭建模块经 `#[path = "../../singlefs-harness/tests/common/mod.rs"] mod common;` 这类声明指回 harness 档的 `tests/common*/mod.rs`，不抄第二份。
- 全量那条登记进 `.claude/gate.d/stage-inputs.tsv` 一行 `crash-case:<名>`，再让崩溃点重放合表 `.claude/gate.d/crash-point-replay-items.tsv` 的一项在第二列点名它（没被点名的，入口 `scripts/crash/crash-point-replay.sh` 不跑，`scripts/crash/crash-case-check.py` 的 placement 判红）；stage-inputs 那一行的第三列 `test=singlefs-checker-tier:<测试目标>:<用例函数>`，计数行、`exhaustive=`、`threads=` 按 `scripts/gate/admission.py` 文件头的写法；用例打不出的那一项不登记、在注释里写明。
- 池级 checker 与实现的依赖闭包（dev-dependencies 也算）除 `singlefs-format` 之外不相交，池级 checker 的源码零处引 `singlefs_core`（D13（验证路线） 已定项 5，门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格判）；它只依赖 `singlefs-format`、没有 `[dev-dependencies]`，这一句那一格不单判。

## 函数名与类型名不许只由空泛词拼成

`run_cell`、`CellRun`、`check_cell`、`fn run`、`fn get` 这一类名字判违规：每一个词都在 `.claude/naming-vague-words` 里，名字就什么都没说。改成说出跑的是什么、判的是哪条、取的是哪个量（`evaluate_stream_domain`、`count_named`、`perform_commit_step`）。`impl <trait> for <类型>` 块里 trait 规定的方法名不判。门禁 code-source-discipline 的 vague-names 格判；还没改完名的文件登记在 `.claude/naming-vague-exclude`，只缩不涨。

## 崩溃一致性只能靠崩溃点重放验证

把块层所有写请求记下来，在**每一个**可能的崩溃点截断、重放、跑池级 checker 与记录核对器。**没跑过这个的写路径就不算验过**：单测全绿说明不了崩溃一致性。

## 功能正确性靠模型对拍

在内存里维护一个只管语义、不管性能也不管崩溃的理想文件系统（`crates/singlefs-harness/src/model.rs`），把同一串随机操作分别施加到模型和实现上，比结果。这是这个项目唯一的功能对照物。

## checker 即规范

不变量清单（`crates/singlefs-checker/invariants/`）每加一条，池级 checker 就加一个检查。**「这个格式到底是什么」，答案以 checker 的源码为准，不是文档。**

## 文件系统特有的反推缺口

- **「测试全绿」不等于「实现正确」。** 要说「崩溃一致性成立」，先说清这一趟本来撞不撞得上：枚举了多少个崩溃点，是不是全部。
- **「checker 没报错」不等于「镜像是好的」。** 说这句话之前，先看 `crates/singlefs-checker/invariants/` 里对应那条的实现状态。

## 门禁的结构

每道门禁都能全跑，也能逐格点名跑。

- 每格一个函数，判一件事。门禁 source 共用库 `.claude/gate.d/lib/stage-cells.sh`（住在 `lib/` 下，`gate.sh` 不把它当阶段跑），用 `stage_cell <格名> <函数> <判什么的一句> <判红时的出路>` 逐格登记；参数解析、样本标记、逐格汇总都经它，不各写一份。写法在它的文件头，样板是 `.claude/gate.d/doc-decisions.sh`。`checker-tier-crash-point-replay` 不 source 它，参数照同一张表自己解析。
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
- `--list` / `--check` 只管格，与文件头 `# gate-cell:` 表逐格相同。重型门禁的逐项点名另用两个参数：`--list-items` 逐行打可点名的项，不编、不跑；`--item <项>` 只跑点名的项，给几次取并集，项写错退 2 并列出可点的项。可点的项：`checker-tier-crash-point-replay` 按崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）、checker-tier-qemu-device-streams 按模式、checker-tier-research-build-and-replay 的 experiment-replay 格按实验、checker-tier-crates-mutation-replay 按变异行、checker-tier-formal-verification 按条（`.claude/gate.d/verus-pins.tsv` 登记的各 crate 逐个判；证明每个有证明义务的单元按全路径 `<crate>::<模块>::<名>` 一项，impl 与 trait 里的方法写 `<类型>::<方法>`、`<trait>::<方法>`：proof fn、exec fn、带 `decreases` 的 spec fn 与 `derive(Clone)` 生成的 `<类型>::clone`，不带 `decreases` 的 spec fn、trait 里没有函数体的声明与 `external_body` 的 fn 不成项、随用到它的项进键，整格跑时每个 crate 各项验过的义务之和等于登记表里它那一行登记的数；证明整格、工具链、钉全各一项；必红世界每行、空真每处 `requires`、红例钉表每行各一项；点名单跑一项与它在整格里跑的判定相同；每条的单元键——证明项是被验函数单元连同它 `requires`、`ensures`、`decreases` 与函数体用到的 spec fn、proof fn、类型、常量的闭包，钉表每行是那一行档与参数重放真走到的装置函数单元闭包，连同状态输入与期望逐态行——与判定库写在它的文件头「复用」一段）。
- `scripts/gate/gate-structure-check.py` 判 `.claude/gate.d/` 顶层每道门禁（共用库在 `lib/` 下，不判）：文件头有格名表；`--list` 退 0、输出与格名表逐格相同、跑的时候没起 cargo、QEMU、herd7 这类重活；`--check` 不存在的格退 2。
- 阶段归属表 `.claude/gate.d/stage-owners.tsv` 一行一个门禁与格名：门禁、格名、判红时派给谁修、为什么。

## 门禁管哪一半

| 判什么 | 谁判 |
|---|---|
| 崩溃点重放跑了、快档绿、每条登记用例的崩溃状态块在崩溃判定库里有没有单元键相同、不红的判定；挂特性的 checker 档测试文件快档至少一趟编进来、跑到的用例不为 0 | 门禁 `checker-tier-crash-point-replay`（后一半是 feature-gated-tests 格）；覆盖声明 `# gate-covers: 崩溃点重放`，键登记在项目根 `.claude/gate-not-implemented.tsv` |
| 形式化验证：恢复重放取前缀、择根、扫环三截的核与择根取前缀的合判推论经 Verus 验过，E165 全量的红例都钉进快档、钉表自己成形、在被判那棵树的实现上逐行重放与原实现相同 | 门禁 checker-tier-formal-verification：verus-toolchain-pinned、verus-proof、verus-vacuity、verus-must-red-worlds 四格用 Verus 验 `crates/singlefs-replay-prefix`（登记表 `.claude/gate.d/verus-pins.tsv`）；红例钉表 `.claude/gate.d/e165-pinned-red-examples.tsv` 两格：pinned-red-examples-table 只判钉表自己（每行 22 段、指纹、抄来的段、档的写法、点名的变异还在，不读红例账、产物与 `research/`），pinned-red-examples-replay 逐行重放；钉全由 E165 第二段驱动 `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` 写完一对红例账与产物时当场判（追加、再 `--check` 同一对，没钉全那一趟判红；它文件头「钉红例」一段） |
| 广谱变异：此刻源码上每一条变异都有单元键相同的结论、没有一条没红 | 门禁 checker-tier-broad-mutation：gate-verdict 格判驱动 `--summary` 的 `BROAD_MUTATION_GATE` 行与汇总行，超时、内存撞顶经驱动升档判成的抓到逐行核库目录登记表 `escalated-verdicts.tsv` 的证据（`scripts/mutation/broad-mutation.py` 文件头「升档」；基线按登记行记的行号找库目录 `baselines.tsv` 的那一行复核，基线键按执行过的单元算），证据不齐的超时、撞顶照旧算没结论、加大限时或上限再点名重跑；判定库 `verdicts.tsv` 每一行带用库目录与仓之外的密钥算的校验列，这一格另用同一把密钥逐行核（驱动文件头「判定行的校验列」）；驱动由主 agent 提交时先起（`.claude/skills/crash-test/commit-stages.md` 第 2b 步） |
| 模型对拍跑了、每段都判过 | 门禁 harness-model-differential-and-scenarios 以 `model-` 起头的那几格；覆盖声明 `# gate-covers: 模型对拍`，键登记在项目根 `.claude/gate-not-implemented.tsv` |
| 崩溃枚举用例住在 checker 档、标了 `#[ignore]` 的登记进崩溃点重放合表 `.claude/gate.d/crash-point-replay-items.tsv`（点名 `crash-case:` 行或放量分项）；checker 档测试文件声明模块 | 门禁 `checker-tier-crash-point-replay` 开跑前跑 `scripts/crash/crash-case-check.py --only placement,modules`，红了不起 cargo；门禁 code-tooling 的 research-script-selftests 格跑那份脚本的自证 |
| 函数名与类型名不只由空泛词拼成；测试文件不按里程碑起名 | 门禁 code-source-discipline：vague-names 格读词表 `.claude/naming-vague-words`、还没改完的文件 `.claude/naming-vague-exclude`；test-file-names 格跑 `scripts/crash/crash-case-check.py --only file-names` |
| 池级 checker 库不依赖实现；harness 档不依赖 checker 档；RefFS 的依赖闭包只有自己与 `singlefs-format`、源码零处引实现与池级 checker 与两档（真名与改名别名都算）、实现与池级 checker 的依赖闭包里没有它 | 门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格（⑤⑥⑦ 判 RefFS 那三样；RefFS 包不在时 ⑤⑥ 报本次未跑） |
| 判器调用点（池级 checker、记录核对器、GPU 判器）落在判定段里，或包住它的函数带 `// gpu-exempt: <截名>` 或 `// cpu-reference-for-gpu-comparison` 标记；两样都不是的调用点判红 | 门禁 checker-independence-and-sync 的 cpu-judging-sites 格：判法 `scripts/crash/cpu-judging-sites-check.py`，判定段与判器本体登记在 `.claude/gate.d/cpu-judging-sites.tsv`。标记写在包住调用点的那个函数前面：属性、文档注释与签名之前紧挨着的 `//` 注释行，或属性与签名之间的注释行；外层函数带了标记，里面嵌套的函数一并算；写在函数体里的不算 |
| harness 档与 checker 档一条用例一个场景 | 门禁 harness-model-differential-and-scenarios 的 one-scenario 格（跑 `scripts/crash/crash-case-check.py --only one-scenario`，读 `crates/singlefs-harness/tests/` 与 `crates/singlefs-checker-tier/tests/` 的用例源码，不跑 checker 档） |
| 变异行点名的测试跑得到：标了 `#[ignore]` 的带 `--include-ignored`、`--` 之后的筛选词筛得到点名的测试 | 门禁 code-source-discipline 的 mutation-tables 格 |
| 每道阶段认第一个参数当项目根 | 门禁 code-tooling 的 stage-owners 格 |
| 谁在什么时候跑得了 checker 档 | `.claude/hooks/heavy-test-guard.sh`，判定在 `lib_heavy_tests.py`：轻型三条（「两档：轻型与重型」）不算重型；崩溃点重放入口 `scripts/crash/crash-point-replay.sh` 的重型跑法（不带参数、`--item`、`--full`）、跑到 checker 档包 `singlefs-checker-tier` 的测试（库单测、集成测试、装置二进制的内联测试）、门禁 checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification（连同直接起它的判法脚本 `scripts/gate/verus-proof-check.py` 证明、空真、必红三种用法与 `scripts/experiment-apparatus/e165-pin-red-examples.py --replay`）与 checker-tier-research-build-and-replay 的 experiment-replay 格、实验复跑入口 `scripts/experiment-apparatus/replay.sh`（不带参数、点名实验号、`--admission-only`；只带一个 `--list-items`、或登记为 light 的单独 `--selftest` 不算）、QEMU、herd7、`crates/mutations.tsv` 整表、全量 `cargo test`、整轮门禁、E152 装置、广谱变异（`scripts/mutation/broad-mutation.py` 起 cargo 或改库目录的每一种用法；`--list-items`、`--summary`、`--check-ignored`、`--help`（`-h`）与登记为 light 的单独 `--selftest` 不算）与门禁 checker-tier-broad-mutation（只点名 gate-verdict 一格的 `--check gate-verdict` 不算）、真编 QEMU（`scripts/devices/build-patched-qemu.sh` 除 `--selftest` 之外的用法）算重型 |

**它们管不到的**：harness 慢档用例标得对不对、快档抽的取样点够不够、全量该多久跑一次——这几样是设计判断，靠人与推论轮三方（`.claude/rules/three-way-inference.md`）。
