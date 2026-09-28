# 验证纪律：harness 档随时跑，checker 档默认只在提交时跑

**这是 singlefs 的项目本地规则**，不在共享 SOP 里——它说的是这个仓的验证代码住在哪、什么时候跑。
共享规则在 `.claude/singlefs-ai-sop/rules/`。

## 定义与名字

三个包，三个名字，全仓只用这三个名字称呼它们：

| 名字 | 包 | 装什么 | 什么时候跑 |
|---|---|---|---|
| **harness 档** | `crates/singlefs-harness` | 单元测试与集成测试，加上它们用的脚手架库：录制器、内存池（`memory_pool`）、理想模型、随机历史、故障注入设备、场景 | 改了代码随时跑 |
| **checker 档** | `crates/singlefs-checker-tier` | 崩溃态枚举引擎与记录核对器（`crash`）、断点续跑与双机分片（`layer0_progress`）、崩溃注入与坏盘输入两场战役、真设备一侧的设备日志比对与运行模式、崩溃放量（崩溃点身份与复用键 `crash_identity`、先录后核的流水线 `crash_amplification`、判定存储 `verdict_store`、单元校验和上 GPU `gpu_unit_checks`、三段流第 ① 段的事实表与 CPU 参照核对 `crash_facts`、第 ② 段的 GPU 核对内核 `crash_verify_gpu`；`verdict_store` 挂特性 `verdict-store`，`gpu_unit_checks` 与 `crash_verify_gpu` 挂特性 `gpu`）、实验与真设备装置二进制（`src/bin/`），以及它们的全部用例；连同 QEMU（门禁 checker-tier-qemu-device-streams）、herd7（checker-tier-lkmm）、crates 变异整表（checker-tier-crates-mutation-replay）、全部实验复跑（checker-tier-research-build-and-replay 的 experiment-replay 格） | 默认只在提交时跑；想单独跑带 `SINGLEFS_HEAVY_TESTS=user-request` |
| **池级 checker** | `crates/singlefs-checker` | 判一个镜像的判决器库（O2，`check_pool_image`，D13（验证路线） 已定项 7）；只依赖 `singlefs-format` | 它是库，被两档调用；它自己的库单测归 harness 档，随时跑 |

归类判据：一条测试或一段代码要枚举崩溃状态、要真设备或外部工具、或跑一次以十分钟计，归 checker 档；否则归 harness 档。拿不准的放 checker 档，再由代码三方判要不要挪回。
依赖只许一个方向：checker 档依赖 harness 档与池级 checker，harness 档不依赖 checker 档（它的依赖闭包里没有 `singlefs-checker-tier`，dev-dependencies 也算；源码里零处引它；门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格判）。两档都要用的一小段代码，宁可各留一份（例：`crates/singlefs-harness/tests/common_corrupted_allocation_record/mod.rs` 抄自 checker 档的坏盘输入），也不让 harness 依赖 checker 档。
测试文件按它测什么起名：领域在前、场景在后（`rollback_by_a_forward_publish`、`crash_enumeration_fixed_script_stream`），checker 档的以模块起头（`crash_enumeration_`、`crash_points_`、`crash_injection_`、`bad_disk_input_`、`record_checker_`）；不带里程碑、步号、增补号、并行线号、欠账号（以里程碑序数起头的、`*_step_four_*`、`*_supplement_two_*`、`*_c519_*`），来历写进文件头的文档注释。`research/scripts/crash-case-check.py` 的 file-names 那一样判，门禁 code-source-discipline 的 test-file-names 格在真仓上跑它。
不许用的叫法：「checker 包」（分不清是池级 checker 还是 checker 档）、「放量用例」「验证档」（说 checker 档）。

## harness 档里再分轻用例与耗时用例

- 轻：每次改完跑，`cargo test -p singlefs-harness` 不带 `--ignored` 跑到的全部。
- 红了只重跑红的那几条，不整份重跑，轻用例与耗时用例一样：`python3 research/scripts/rerun-failed-tests.py <上一趟的日志> -p <包>` 按测试目标打印只跑那几条的命令（一律带 `--include-ignored`，红的是耗时用例也跑得到），经 Bash 起、加内存包装；修完代码也只重跑红的那几条；整份重跑只在上一趟的日志不全时（脚本判红、一条命令都不打），完整的一遍归提交时的整轮门禁。
- 耗时用例：debug 下单条跑到 60 秒及以上的用例，标 `#[ignore = "harness 耗时用例：…"]`；要跑随时跑，一律经 `research/scripts/run-with-memory-cap.sh`，线程数按派发提示的上限。单条用时看平常跑的时候量到的，不另外单独量。
- 调全量崩溃枚举函数（`enumerate_layer0` 一族，快档 `quick_tier` 那几个除外）或自己逐个造崩溃状态（名字带 `every_crash`；循环里对录制操作取到循环变量为止的前缀去 `apply`、或造 `CrashImage`），直接这样做或经同一文件里的函数这样做的测试不是 harness 档的重，它是 checker 档，写进 `crates/singlefs-checker-tier/tests/`；`research/scripts/crash-case-check.py` 的 placement 那一样判这一条，写在别的包里判红，门禁 `54-layer0-replay` 开跑前在真仓上跑它。
- 一条用例一个场景：按参数循环、每一轮新建一个池（`build_pool`、`build_through_*`、`format_pool`、`MemoryPool::with_devices`）的，拆成一个带参数的函数加每个取值一条 `#[test]`，红了只重跑那一条；确是一个场景的（同一个池上按次序做几轮）在用例上面写一行 `// harness-test-granularity:one-scenario <理由>`。`research/scripts/crash-case-check.py` 的 one-scenario 那一样判，门禁 harness-model-differential-and-scenarios 的 one-scenario 格在真仓上跑它。

## checker 档自己分快档与全量

| 档 | 命令 | 什么时候 |
|---|---|---|
| 快档 | `cargo test --release -p singlefs-checker-tier --lib --tests` 不带 `--ignored`（库与集成测试；装置二进制 `src/bin/` 的内联单测不在快档里，归它们的变异表与实验复跑）；门禁 `54-layer0-replay` 跑它，再逐条核 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的用例各自那一格全绿标记，标记不作数的报「本次未跑」、不判红 | 整轮门禁与每次提交 |
| 全量 | `54-layer0-replay` `--full`：在 HEAD + 暂存区的 worktree 里逐条跑登记的崩溃枚举用例（`--include-ignored --exact`），那一格全绿标记在就复用；分层照 D13（验证路线） 已定项 9；GPU 只接校验和那一截，要不要接归 D24（后台重活能不能卸给 GPU） | 用户要求或夜间 |

checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 与 checker-tier-research-build-and-replay 的 experiment-replay 格照各自的复用判定跑（`research/scripts/stage-must-run.sh` 文件头）。

## 崩溃枚举用例住哪、怎么登记

- checker 档每个测试文件第一行写它测哪几个模块：`//! checker 档模块：<模块，按 crash、layer0_progress、crash_injection、bad_disk_input、device_log、on_device_modes、crash_identity、crash_amplification、verdict_store、gpu_unit_checks、crash_facts、crash_verify_gpu 的次序用、隔开>`，与它从 `singlefs_checker_tier::` 导入的模块逐个相同；一个都不导入的写 `无（为什么）`。按模块找用例：`grep -l '^//! checker 档模块：.*crash_injection' crates/singlefs-checker-tier/tests/*.rs`。`research/scripts/crash-case-check.py` 的 modules 那一样判，门禁 `54-layer0-replay` 开跑前在真仓上跑它。

- 写在 `crates/singlefs-checker-tier/tests/<流的名字>.rs`，全量那条标 `#[ignore]`，同文件的快档用例不标。
- 共用的搭建模块经 `#[path = "../../singlefs-harness/tests/common/mod.rs"] mod common;` 这类声明指回 harness 档的 `tests/common*/mod.rs`，不抄第二份。
- 全量那条登记进 `.claude/gate.d/stage-inputs.tsv` 一行 `crash-case:<名>`，第三列 `test=singlefs-checker-tier:<测试目标>:<用例函数>`，计数行、`exhaustive=`、`threads=` 按 `research/scripts/admission.py` 文件头的写法；用例打不出的那一项不登记、在注释里写明。
- 池级 checker 与实现的依赖闭包（dev-dependencies 也算）除 `singlefs-format` 之外不相交，池级 checker 的源码零处引 `singlefs_core`（D13（验证路线） 已定项 5，门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格判）；它今天只依赖 `singlefs-format`、没有 `[dev-dependencies]`，这一句那一格不单判。

## 函数名与类型名不许只由空泛词拼成

`run_cell`、`CellRun`、`check_cell`、`fn run`、`fn get` 这一类名字判违规：每一个词都在 `.claude/naming-vague-words` 里，名字就什么都没说。改成说出跑的是什么、判的是哪条、取的是哪个量（`evaluate_stream_domain`、`count_named`、`perform_commit_step`）。`impl <trait> for <类型>` 块里 trait 规定的方法名不判。门禁 code-source-discipline 的 vague-names 格判；还没改完名的文件登记在 `.claude/naming-vague-exclude`，只缩不涨。

## 崩溃一致性只能靠崩溃点重放验证

把块层所有写请求记下来，在**每一个**可能的崩溃点截断、重放、跑池级 checker 与记录核对器。**没跑过这个的写路径就不算验过**：单测全绿说明不了崩溃一致性。

## 功能正确性靠模型对拍

在内存里维护一个只管语义、不管性能也不管崩溃的理想文件系统（`crates/singlefs-harness/src/model.rs`），把同一串随机操作分别施加到模型和实现上，比结果。这是这个项目唯一的功能对照物，没有现成实现可以拿输出当标准答案。

## checker 即规范

不变量清单（`.claude/kb/invariants.md`）每加一条，池级 checker 就加一个检查。**「这个格式到底是什么」，答案以 checker 的源码为准，不是文档。**

## 文件系统特有的反推缺口

这个项目测的多半是对还是不对这种二选一的东西，风险在覆盖够不够：

- **「测试全绿」不等于「实现正确」。** 崩溃窗口可能只有一次写那么宽，没撞上也许只是没遍历到那个崩溃点。要说「崩溃一致性成立」，先说清这一趟本来撞不撞得上：枚举了多少个崩溃点，是不是全部。
- **「checker 没报错」不等于「镜像是好的」。** 也可能是 checker 还没实现那条检查。说这句话之前，先看 `.claude/kb/invariants.md` 里对应那条的实现状态。

## 门禁的结构

每道门禁都能全跑，也能逐格点名跑。

- 每格一个函数，判一件事。门禁 source 共用库 `.claude/gate.d/lib/stage-cells.sh`（住在 `lib/` 下，`gate.sh` 不把它当阶段跑），用 `stage_cell <格名> <函数> <判什么的一句> <判红时的出路>` 逐格登记；参数解析、样本标记、逐格汇总都经它，不各写一份。写法在它的文件头，样板是 `.claude/gate.d/doc-decisions.sh`。`54-layer0-replay` 不 source 它，参数照同一张表自己解析。
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
- `--list` / `--check` 只管格，与文件头 `# gate-cell:` 表逐格相同。重型门禁的逐项点名另用两个参数：`--list-items` 逐行打可点名的项，不编、不跑；`--item <项>` 只跑点名的项，给几次取并集，项写错退 2 并列出可点的项。可点的项：`54-layer0-replay` 按崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）、checker-tier-qemu-device-streams 按模式、checker-tier-research-build-and-replay 的 experiment-replay 格按实验、checker-tier-crates-mutation-replay 按变异行。
- `research/scripts/gate-structure-check.py` 判 `.claude/gate.d/` 顶层每道门禁（共用库在 `lib/` 下，不判）：文件头有格名表；`--list` 退 0、输出与格名表逐格相同、跑的时候没起 cargo、QEMU、herd7 这类重活；`--check` 不存在的格退 2。
- 阶段归属表 `.claude/gate.d/stage-owners.tsv` 一行一个门禁与格名：门禁、格名、判红时派给谁修、为什么。

## 门禁管哪一半

| 判什么 | 谁判 |
|---|---|
| 崩溃点重放跑了、快档绿、每条登记用例的全绿标记作不作数 | 门禁 `54-layer0-replay`；覆盖声明 `# gate-covers: 崩溃点重放`，键登记在项目根 `.claude/gate-not-implemented.tsv` |
| 模型对拍跑了、每段都判过 | 门禁 harness-model-differential-and-scenarios 以 `model-` 起头的那几格；覆盖声明 `# gate-covers: 模型对拍`，键登记在项目根 `.claude/gate-not-implemented.tsv` |
| 崩溃枚举用例住在 checker 档、标了 `#[ignore]` 的登记了 `crash-case:`；checker 档测试文件声明模块 | 门禁 `54-layer0-replay` 开跑前跑 `research/scripts/crash-case-check.py --only placement,modules`，红了不起 cargo；门禁 code-tooling 的 research-script-selftests 格跑那份脚本的自证 |
| 函数名与类型名不只由空泛词拼成；测试文件不按里程碑起名 | 门禁 code-source-discipline：vague-names 格读词表 `.claude/naming-vague-words`、还没改完的文件 `.claude/naming-vague-exclude`；test-file-names 格跑 `research/scripts/crash-case-check.py --only file-names` |
| 池级 checker 库不依赖实现；harness 档不依赖 checker 档 | 门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格 |
| harness 档一条用例一个场景 | 门禁 harness-model-differential-and-scenarios 的 one-scenario 格（跑 `research/scripts/crash-case-check.py --only one-scenario`） |
| 变异行点名的测试跑得到：标了 `#[ignore]` 的带 `--include-ignored`、`--` 之后的筛选词筛得到点名的测试 | 门禁 code-source-discipline 的 mutation-tables 格 |
| 每道阶段认第一个参数当项目根 | 门禁 code-tooling 的 stage-owners 格 |
| 谁在什么时候跑得了 checker 档 | `.claude/hooks/heavy-test-guard.sh`，判定在 `lib_heavy_tests.py`：跑到 checker 档包 `singlefs-checker-tier` 的测试（库单测、集成测试、装置二进制的内联测试）、门禁 checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 与 checker-tier-research-build-and-replay 的 experiment-replay 格、QEMU、herd7、`crates/mutations.tsv` 整表、全量 `cargo test`、整轮门禁、E152 装置算重型 |

**它们管不到的**：harness 耗时用例标得对不对、快档抽的取样点够不够、全量该多久跑一次——这几样靠人与代码三方。
