# 验证纪律：harness 档随时跑，checker 档默认只在提交时跑

**这是 singlefs 的项目本地规则**，不在共享 SOP 里——它说的是这个仓的验证代码住在哪、什么时候跑。
共享规则在 `.claude/singlefs-ai-sop/rules/`。

## 定义与名字

四个包，四个名字，全仓只用这四个名字称呼它们：

| 名字 | 包 | 装什么 | 什么时候跑 |
|---|---|---|---|
| **harness 档** | `crates/singlefs-harness` | 单元测试与集成测试，加上它们用的脚手架库：录制器、内存池（`memory_pool`）、理想模型、随机历史、故障注入设备、场景 | 改了代码随时跑 |
| **checker 档** | `crates/singlefs-checker-tier` | 崩溃态枚举引擎与记录核对器（`crash`）、断点续跑与双机分片（`layer0_progress`）、崩溃注入与坏盘输入两场战役、可写挂载每个写落在哪一段的划分与清扫段的写（`writable_mount_phases`）、真设备一侧的设备日志比对与运行模式、崩溃放量（崩溃点身份与复用键 `crash_identity`、先录后核的流水线 `crash_amplification`、判定存储 `verdict_store`、单元校验和上 GPU `gpu_unit_checks`、三段流第 ① 段的事实表与 CPU 参照核对 `crash_facts`、第 ② 段的 GPU 核对内核 `crash_verify_gpu`、GPU 判器的输入表 `crash_judge_tables`、GPU 判器的判法 `crash_judge_gpu` 与它的派活 `crash_judge_dispatch`；`verdict_store` 挂特性 `verdict-store`，`gpu_unit_checks`、`crash_verify_gpu`、`crash_judge_gpu` 与 `crash_judge_dispatch` 挂特性 `gpu`）、实验与真设备装置二进制（`src/bin/`），以及它们的全部用例（崩溃点重放、形式化验证、crates 变异表三样另分轻型与重型，轻型照「两档：轻型与重型」跑）；连同 QEMU（门禁 checker-tier-qemu-device-streams）、herd7（checker-tier-lkmm）、crates 变异整表（checker-tier-crates-mutation-replay）、全部实验复跑（checker-tier-research-build-and-replay 的 experiment-replay 格）、形式化验证（checker-tier-formal-verification：Verus 验 `crates/singlefs-replay-prefix`，E165 红例钉表两格）、广谱变异（驱动 `research/scripts/broad-mutation.py`；提交门禁 checker-tier-broad-mutation 判这一次暂存树那一版每一条变异都有结论、没有没红） | 默认只在提交时跑；想单独跑带 `SINGLEFS_HEAVY_TESTS=user-request` |
| **池级 checker** | `crates/singlefs-checker` | 判一个镜像的判决器库（O2，`check_pool_image`，D13（验证路线） 已定项 7）；只依赖 `singlefs-format` | 它是库，被两档调用；它自己的库单测归 harness 档，随时跑 |
| **RefFS** | `crates/singlefs-reference-filesystem` | 语义级的参照文件系统：只在内存里按 POSIX 操作走，判一次崩溃之后恢复读出的内容落不落在允许集合里；依赖只有 `std`，不引实现、池级 checker、harness 档、checker 档，实现与池级 checker 也不依赖它（门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格 ⑤⑥⑦ 判） | 它是库，被 harness 档与 checker 档调用；它自己的单测归 harness 档，随时跑 |

归类判据：一条测试或一段代码要枚举崩溃状态、要真设备或外部工具、或跑一次以十分钟计，归 checker 档；否则归 harness 档。拿不准的放 checker 档，再由代码三方判要不要挪回。
依赖只许一个方向：checker 档依赖 harness 档与池级 checker，harness 档不依赖 checker 档（它的依赖闭包里没有 `singlefs-checker-tier`，dev-dependencies 也算；源码里零处引它；门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格判）。两档都要用的一小段代码，宁可各留一份（例：`crates/singlefs-harness/tests/common_corrupted_allocation_record/mod.rs` 抄自 checker 档的坏盘输入），也不让 harness 依赖 checker 档。
测试文件按它测什么起名：领域在前、场景在后（`rollback_by_a_forward_publish`、`crash_enumeration_fixed_script_stream`），checker 档的以模块起头（`crash_enumeration_`、`crash_points_`、`crash_injection_`、`bad_disk_input_`、`record_checker_`）；名字不带来历照 `.claude/singlefs-ai-sop/rules/test-script-discipline.md`「用例原子化：每一条都能单独跑」，本仓判的形态是以里程碑序数起头的、`*_step_four_*`、`*_supplement_two_*`、`*_parallel_line_one_*`、`*_c519_*`。`research/scripts/crash-case-check.py` 的 file-names 那一样判，门禁 code-source-discipline 的 test-file-names 格在真仓上跑它。
不许用的叫法：「checker 包」（分不清是池级 checker 还是 checker 档）、「放量用例」「验证档」（说 checker 档）。

## harness 档里再分快档用例与慢档用例

- 轻：每次改完跑，`cargo test -p singlefs-harness` 不带 `--ignored` 跑到的全部。
- 红了只重跑红的那几条（`.claude/singlefs-ai-sop/rules/test-script-discipline.md`「用例原子化：每一条都能单独跑」），快档用例与慢档用例一样：`python3 research/scripts/rerun-failed-tests.py <上一趟的日志> -p <包>` 按测试目标打印只跑那几条的命令（一律带 `--include-ignored`），经 Bash 起、加内存包装；它判红、一条命令都不打时才整份重跑，完整的一遍归提交时的整轮门禁。
- 慢档用例：debug 下单条跑到 60 秒及以上的用例，标 `#[ignore = "harness 慢档用例：…"]`；要跑随时跑，一律经 `research/scripts/run-with-memory-cap.sh`，线程数按派发提示的上限。单条用时看平常跑的时候量到的，不另外单独量。
- 调全量崩溃枚举函数（`enumerate_layer0` 一族，快档 `quick_tier` 那几个除外）或自己逐个造崩溃状态（名字带 `every_crash`；循环里对录制操作取到循环变量为止的前缀去 `apply`、或造 `CrashImage`），直接这样做或经同一文件里的函数这样做的测试不是 harness 档的重，它是 checker 档，写进 `crates/singlefs-checker-tier/tests/`；`research/scripts/crash-case-check.py` 的 placement 那一样判这一条，写在别的包里判红，门禁 `checker-tier-layer0-replay` 开跑前在真仓上跑它。
- 一条用例一个场景照 `.claude/singlefs-ai-sop/rules/test-script-discipline.md`「用例原子化：每一条都能单独跑」：本仓按「每一轮新建一个池」认参数循环（`build_pool`、`build_through_*`、`format_pool`、`MemoryPool::with_devices`），确是一个场景的在用例上面写一行 `// harness-test-granularity:one-scenario <理由>`。`research/scripts/crash-case-check.py` 的 one-scenario 那一样判，门禁 harness-model-differential-and-scenarios 的 one-scenario 格在真仓上跑它。

## checker 档自己分快档与全量

| 档 | 命令 | 什么时候 |
|---|---|---|
| 快档 | `cargo test --release -p singlefs-checker-tier --lib --tests` 不带 `--ignored`（库与集成测试；装置二进制 `src/bin/` 的内联单测不在快档里，归它们的变异表与实验复跑），再一趟 `cargo test --release -p singlefs-checker-tier --features <tests/ 里 cfg 点到的特性> --lib --test <挂特性的测试文件>…`（`BINDGEN_EXTRA_CLANG_ARGS` 取 `research/scripts/crash-amplification-lib.sh` 的 `CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS`；特性里有 `gpu` 的这一趟经 `research/scripts/gpu-quiesce-run.sh` 起）；门禁 `checker-tier-layer0-replay` 跑这两趟（feature-gated-tests 格：挂特性的测试文件跑到 0 条的、没卡时 GPU 对拍打的 `NOT_RUN_NO_GPU_ADAPTER` 逐条报「本次未跑」），再逐条核 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的用例各自那一格全绿标记，标记不作数的报「本次未跑」、不判红 | 整轮门禁与每次提交 |
| 全量 | `checker-tier-layer0-replay` `--full`：在 HEAD + 暂存区的 worktree 里逐条跑登记的崩溃枚举用例（`--include-ignored --exact`），那一格全绿标记在就复用；分层照 D13（验证路线） 已定项 9；GPU 只接校验和那一截，要不要接归 D24（后台重活能不能卸给 GPU） | 用户要求 |
| 崩溃放量全量 | `bash research/scripts/crash-amplification.sh`：不带参数跑登记表里全部分项（`--list-items` 列、`--item <分项>` 点名），开跑经 `research/scripts/gpu-quiesce-run.sh` 照多机配置清场、跑完不复原（照「凡用 GPU 先清场，跑完不复原」一条）；每项交 `crash-amplification-item.sh`：配置开了双机就两台跑、没开就一台跑，配置 `ENABLE_GPU=1` 时由 GPU 判，显卡照多机配置的 `GPU_CARDS_BY_PRIORITY` / `GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED` 的先后与额度用，用不上时照 `.claude/singlefs-ai-sop/rules/test-script-discipline.md`「用上现有的硬件加速」退 78（配置对不上、点名的卡用不了、第二台连不上、清场没过），算法上判不了的（有状态 GPU 判不了、内核展开超预算、建不出表）退 248，退出码表在入口与分项驱动的文件头；不带参数按入口登记的全量展开深度跑（`FULL_EXPAND_UP_TO`），给了 `--expand-up-to` 的是缩过规模的一趟；一份库装全部分项，落在 `${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}/library-by-conclusion-kind`，落在开机会清空的目录里被拒；复用与续跑照同一篇「判定可复用：按输入存，按前缀剪」，键与判法版本在 `crates/singlefs-checker-tier/src/crash_identity.rs`，存放布局在 `crates/singlefs-checker-tier/src/verdict_store.rs` | 用户要求 |

checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-broad-mutation 与 checker-tier-research-build-and-replay 的 experiment-replay 格照各自的复用判定跑（`research/scripts/stage-must-run.sh` 文件头）。

## 两档：轻型与重型

崩溃点重放、形式化验证、crates 变异表三样各分轻型与重型；harness 档不在这两档里，照「定义与名字」随时跑。

| 档 | 守什么 | 跑法 | 谁跑、什么时候 |
|---|---|---|---|
| 轻型 | 两个目标：守住系统核心代码不红（核心部分，每次都跑）；测这一次改到的代码（可变部分，跑的人按自己改到的文件点名） | 三条命令带 `--quick`：`bash research/scripts/crash-point-replay.sh --quick`、`bash .claude/gate.d/checker-tier-formal-verification.sh --quick <根>`、`bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --quick <根>`。只带 `--quick` 跑核心部分；再带 `--item <项>`（可给几次）跑核心部分加点名的项，可点名的项用各条的 `--list-items` 列；点名的项在轻型的域上跑（崩溃取「甲二」与展开 6 两个域的并集，形式化与变异就是那几项本身）。跑全部，不剪枝、不只跑一部分 | 形式化那一条：实现员、调查员交回前跑；崩溃与变异两条：子 agent 不起，命令写进报告「要主 agent 跑的命令」一节，主 agent 在实现员的副本里跑；合入后验证三条的全部由主 agent 跑；三条带不带 `SINGLEFS_HEAVY_TESTS` 前缀都放行 |
| 重型 | 总门闸：验全体 | 同三条命令不带 `--quick`，三种跑法：不带参数是剪枝（只剪键相同、已判过的与等价的）、`--item <项>` 是逐条、`--full` 是全量（清空复用） | 只由主 agent 带前缀跑：提交时，或用户要求时 |

- 核心部分取什么：形式化是工具链、证明、钉全三项；crates 变异表是改坏对象在 `crates/singlefs-core/src/` 的那些行里每个测试目标固定一行（登记表 `.claude/gate.d/crates-mutation-light-rows.tsv`）；崩溃是每项在「甲二」与展开 6 两个域的并集上判，连同小输入上的 GPU 与 CPU 对拍、剪枝自证、钉回来的红例（两个域的写法以 `research/scripts/crash-point-replay.sh` 文件头为准）。重型是全部：形式化的全部项、变异整表、崩溃整域。
- 轻型是重型的子集：轻型里有的每一项，重型里都有。写测试先写重型，再把其中关键的步骤拿出来放进轻型；轻型登记的每一行写明取自重型的哪一项，指得到重型里现存的那一项。
- 三条都经 `research/scripts/run-with-memory-cap.sh` 起；带 GPU 的照「凡用 GPU 先清场，跑完不复原」一条经 `research/scripts/gpu-quiesce-run.sh` 起，要卡时照清场规则排队等卡，不抢在跑的重型。
- 重型红了必须修好或查清原因，红着不提交。提交时用哪种跑法由用户在弹窗里选；选逐条的，主 agent 列出输入变了的项给用户点，没点到的报「本次未跑」。新入口接进提交那一趟之前，提交时照「checker 档自己分快档与全量」一节跑。
- 轻型保留到事务功能做全；之后由用户判稳定，再并进重型。
- 谁跑得了哪一档由 `.claude/hooks/heavy-test-guard.sh` 判（认轻型的判法在 `.claude/hooks/lib_heavy_tests.py` 的 `light_tier_command`）：三条带 `--quick`（可再带 `--item <项>`）的谁都能跑，子 agent 不经内存包装起的拒；夹了别的参数的、不带 `--quick` 的照重型拒。派发提示许可子 agent 跑轻型的放行，许可跑重型的由 `.claude/hooks/runner-dispatch-guard.sh` 拒。

**凡用 GPU 先清场，跑完不复原**，经 `bash research/scripts/gpu-quiesce-run.sh -- <命令…>`：崩溃放量全量与分项驱动、带 `gpu` 特性的 cargo（GPU 对拍、快档带特性那一趟）、手跑的 GPU 测试都算。它照多机配置清场并回读、跑；跑完、跑红或被打断都不复原。显卡额度用清场之后那一份。清场回读没过不跑、退 78：卡上有不归清场白名单的计算进程时就是这样，等那个进程跑完再起，不停它。要复原，手动跑多机配置 `QUIESCE_START_COMMAND` 那一条私有复原脚本，再跑 `QUIESCE_STARTED_CHECK_COMMAND` 回读到它退 0。崩溃放量入口、分项驱动与门禁 `checker-tier-layer0-replay` 的 feature-gated-tests 格自己经它起，不在外面再套一层。

## 崩溃枚举用例住哪、怎么登记

- checker 档每个测试文件第一行写它测哪几个模块：`//! checker 档模块：<模块，按 crash、layer0_progress、crash_injection、writable_mount_phases、bad_disk_input、device_log、on_device_modes、crash_identity、crash_amplification、verdict_store、gpu_unit_checks、crash_facts、crash_verify_gpu、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch 的次序用、隔开>`，与它从 `singlefs_checker_tier::` 导入的模块逐个相同；一个都不导入的写 `无（为什么）`。按模块找用例：`grep -l '^//! checker 档模块：.*crash_injection' crates/singlefs-checker-tier/tests/*.rs`。`research/scripts/crash-case-check.py` 的 modules 那一样判，门禁 `checker-tier-layer0-replay` 开跑前在真仓上跑它。

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

- 每格一个函数，判一件事。门禁 source 共用库 `.claude/gate.d/lib/stage-cells.sh`（住在 `lib/` 下，`gate.sh` 不把它当阶段跑），用 `stage_cell <格名> <函数> <判什么的一句> <判红时的出路>` 逐格登记；参数解析、样本标记、逐格汇总都经它，不各写一份。写法在它的文件头，样板是 `.claude/gate.d/doc-decisions.sh`。`checker-tier-layer0-replay` 不 source 它，参数照同一张表自己解析。
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
- `--list` / `--check` 只管格，与文件头 `# gate-cell:` 表逐格相同。重型门禁的逐项点名另用两个参数：`--list-items` 逐行打可点名的项，不编、不跑；`--item <项>` 只跑点名的项，给几次取并集，项写错退 2 并列出可点的项。可点的项：`checker-tier-layer0-replay` 按崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）、checker-tier-qemu-device-streams 按模式、checker-tier-research-build-and-replay 的 experiment-replay 格按实验、checker-tier-crates-mutation-replay 按变异行、checker-tier-formal-verification 按条（必红世界每行、空真每处 `requires`、红例钉表每行各一项，证明、工具链、钉全各一项；点名单跑一项与它在整格里跑的判定相同，按条复用的键与判定库写在它的文件头「按条复用」）。
- `research/scripts/gate-structure-check.py` 判 `.claude/gate.d/` 顶层每道门禁（共用库在 `lib/` 下，不判）：文件头有格名表；`--list` 退 0、输出与格名表逐格相同、跑的时候没起 cargo、QEMU、herd7 这类重活；`--check` 不存在的格退 2。
- 阶段归属表 `.claude/gate.d/stage-owners.tsv` 一行一个门禁与格名：门禁、格名、判红时派给谁修、为什么。

## 门禁管哪一半

| 判什么 | 谁判 |
|---|---|
| 崩溃点重放跑了、快档绿、每条登记用例的全绿标记作不作数；挂特性的 checker 档测试文件快档至少一趟编进来、跑到的用例不为 0 | 门禁 `checker-tier-layer0-replay`（后一半是 feature-gated-tests 格）；覆盖声明 `# gate-covers: 崩溃点重放`，键登记在项目根 `.claude/gate-not-implemented.tsv` |
| 形式化验证：恢复重放取前缀的核经 Verus 验过，E165 全量的红例都钉进快速档、在被判那棵树的实现上逐行重放与原实现相同 | 门禁 checker-tier-formal-verification：verus-toolchain-pinned、verus-proof、verus-vacuity、verus-must-red-worlds 四格用 Verus 验 `crates/singlefs-replay-prefix`（登记表 `.claude/gate.d/verus-pins.tsv`），pinned-red-examples-complete、pinned-red-examples-replay 两格判红例钉表 `.claude/gate.d/e165-pinned-red-examples.tsv` |
| 广谱变异：这一次暂存树那一版里改到的变异每一条都有这一版的结论、沿用的结论都在、没有一条没红 | 门禁 checker-tier-broad-mutation：library-version-is-staged-tree 格判驱动库目录此刻那一版与暂存树在导出范围上逐文件相同、工具链与全局键相同，gate-verdict 格判驱动 `--summary` 的 `BROAD_MUTATION_GATE` 行与汇总行，超时、内存撞顶经驱动升档判成的抓到逐行核库目录登记表 `escalated-verdicts.tsv` 的证据（`research/scripts/broad-mutation.py` 文件头「升档」；基线按登记行记的行号找那一版 `baseline.tsv` 的那一行复核），证据不齐的超时、撞顶照旧算没结论、加大限时或上限再点名重跑；结果表每一行带用库目录与仓之外的密钥算的校验列，这一格另用同一把密钥逐行核（驱动文件头「结果行的校验列」）；驱动由主 agent 提交时先起（`.claude/skills/crash-test/commit-stages.md` 第 2b 步） |
| 模型对拍跑了、每段都判过 | 门禁 harness-model-differential-and-scenarios 以 `model-` 起头的那几格；覆盖声明 `# gate-covers: 模型对拍`，键登记在项目根 `.claude/gate-not-implemented.tsv` |
| 崩溃枚举用例住在 checker 档、标了 `#[ignore]` 的登记了 `crash-case:`；checker 档测试文件声明模块 | 门禁 `checker-tier-layer0-replay` 开跑前跑 `research/scripts/crash-case-check.py --only placement,modules`，红了不起 cargo；门禁 code-tooling 的 research-script-selftests 格跑那份脚本的自证 |
| 函数名与类型名不只由空泛词拼成；测试文件不按里程碑起名 | 门禁 code-source-discipline：vague-names 格读词表 `.claude/naming-vague-words`、还没改完的文件 `.claude/naming-vague-exclude`；test-file-names 格跑 `research/scripts/crash-case-check.py --only file-names` |
| 池级 checker 库不依赖实现；harness 档不依赖 checker 档；RefFS 的依赖闭包只有自己与 `singlefs-format`、源码零处引实现与池级 checker 与两档（真名与改名别名都算）、实现与池级 checker 的依赖闭包里没有它 | 门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格（⑤⑥⑦ 判 RefFS 那三样；RefFS 包不在时 ⑤⑥ 报本次未跑） |
| harness 档一条用例一个场景 | 门禁 harness-model-differential-and-scenarios 的 one-scenario 格（跑 `research/scripts/crash-case-check.py --only one-scenario`） |
| 变异行点名的测试跑得到：标了 `#[ignore]` 的带 `--include-ignored`、`--` 之后的筛选词筛得到点名的测试 | 门禁 code-source-discipline 的 mutation-tables 格 |
| 每道阶段认第一个参数当项目根 | 门禁 code-tooling 的 stage-owners 格 |
| 谁在什么时候跑得了 checker 档 | `.claude/hooks/heavy-test-guard.sh`，判定在 `lib_heavy_tests.py`：轻型三条（「两档：轻型与重型」）不算重型；崩溃点重放入口 `research/scripts/crash-point-replay.sh` 的重型跑法（不带参数、`--item`、`--full`）、跑到 checker 档包 `singlefs-checker-tier` 的测试（库单测、集成测试、装置二进制的内联测试）、门禁 checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification（连同直接起它的判法脚本 `research/scripts/verus-proof-check.py` 证明、空真、必红三种用法与 `research/scripts/e165-pin-red-examples.py --replay`）与 checker-tier-research-build-and-replay 的 experiment-replay 格、QEMU、herd7、`crates/mutations.tsv` 整表、全量 `cargo test`、整轮门禁、E152 装置、广谱变异（`research/scripts/broad-mutation.py` 起 cargo 或改库目录的每一种用法；`--list-items`、`--summary`、`--check-ignored`、`--help`（`-h`）与登记为 light 的单独 `--selftest` 不算）与门禁 checker-tier-broad-mutation（只点名 gate-verdict 一格的 `--check gate-verdict` 不算）、真编 QEMU（`research/scripts/build-patched-qemu.sh` 除 `--selftest` 之外的用法）算重型 |

**它们管不到的**：harness 慢档用例标得对不对、快档抽的取样点够不够、全量该多久跑一次——这几样靠人与代码三方。
