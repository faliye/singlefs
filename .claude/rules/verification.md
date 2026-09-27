# 验证纪律：harness 档随时跑，checker 档默认只在提交时跑

**这是 singlefs 的项目本地规则**，不在共享 SOP 里——它说的是这个仓的验证代码住在哪、什么时候跑。
共享规则在 `.claude/singlefs-ai-sop/rules/`。

## 定义与名字

三个包，三个名字，全仓只用这三个名字称呼它们：

| 名字 | 包 | 装什么 | 什么时候跑 |
|---|---|---|---|
| **harness 档** | `crates/singlefs-harness` | 单元测试与集成测试，加上它们用的脚手架库：录制器、内存池（`memory_pool`）、理想模型、随机历史、故障注入设备、场景 | 改了代码随时跑 |
| **checker 档** | `crates/singlefs-checker-tier` | 崩溃态枚举引擎与记录核对器（`crash`）、断点续跑与双机分片（`layer0_progress`）、崩溃注入与坏盘输入两场战役、真设备一侧的设备日志比对与运行模式、实验与真设备装置二进制（`src/bin/`），以及它们的全部用例；连同 QEMU（55 号）、herd7（57 号）、crates 变异整表（59 号）、全部实验复跑（87 号） | 默认只在提交时跑；想单独跑带 `SINGLEFS_HEAVY_TESTS=user-request` |
| **池级 checker** | `crates/singlefs-checker` | 判一个镜像的判决器库（O2，`check_pool_image`，D13（验证路线） 已定项 7）；只依赖 `singlefs-format` | 它是库，被两档调用；它自己的库单测归 harness 档，随时跑 |

归类判据：一条测试或一段代码要枚举崩溃状态、要真设备或外部工具、或跑一次以十分钟计，归 checker 档；否则归 harness 档。拿不准的放 checker 档，再由代码三方判要不要挪回。
依赖只许一个方向：checker 档依赖 harness 档与池级 checker，harness 档不依赖 checker 档（它的依赖闭包里没有 `singlefs-checker-tier`，dev-dependencies 也算；源码里零处引它；门禁 94 号判）。两档都要用的一小段代码，宁可各留一份（例：`crates/singlefs-harness/tests/common_corrupted_allocation_record/mod.rs` 抄自 checker 档的坏盘输入），也不让 harness 依赖 checker 档。
不许用的叫法：「checker 包」（分不清是池级 checker 还是 checker 档）、「放量用例」「验证档」（说 checker 档）。

## harness 档里再分轻重

- 轻：每次改完跑，`cargo test -p singlefs-harness` 不带 `--ignored` 跑到的全部。
- 复测只跑上一趟红的用例：`python3 research/scripts/rerun-failed-tests.py <上一趟的日志> -p <包>` 按测试目标打印只跑它们的命令，经 Bash 起（加内存包装）；上一趟之后代码又改过、或上一趟日志不全（脚本判红）时才整份重跑。
- 重：随机历史长档、release 下要跑几分钟的用例，标 `#[ignore]`，`#[ignore = "…"]` 的消息写清多久、给谁跑；要跑随时跑，一律经 `research/scripts/run-with-memory-cap.sh`，线程数按派发提示的上限。
- 调全量崩溃枚举函数（`enumerate_layer0` 一族，快档 `quick_tier` 那几个除外，直接调或经同一文件里的函数调）的测试不是 harness 档的重，它是 checker 档，写进 `crates/singlefs-checker-tier/tests/`；`research/scripts/crash-case-check.py` 判这一条，写在别的包里判红。

## checker 档自己分快档与全量

| 档 | 命令 | 什么时候 |
|---|---|---|
| 快档 | `cargo test --release -p singlefs-checker-tier --lib --tests` 不带 `--ignored`（库与集成测试；装置二进制 `src/bin/` 的内联单测不在快档里，归它们的变异表与实验复跑）；门禁 54 号跑它，再逐条核 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的用例各自那一格全绿标记，标记不作数的报「本次未跑」、不判红 | 整轮门禁与每次提交 |
| 全量 | 54 号 `--full`：在 HEAD + 暂存区的 worktree 里逐条跑登记的崩溃枚举用例（`--include-ignored --exact`），那一格全绿标记在就复用；分层照 D13（验证路线） 已定项 9；GPU 只接校验和那一截，要不要接归 D24（后台重活能不能卸给 GPU） | 用户要求或夜间 |

55、57、59、87 号照各自的复用判定跑（`research/scripts/stage-must-run.sh` 文件头）。

## 崩溃枚举用例住哪、怎么登记

- 写在 `crates/singlefs-checker-tier/tests/<流的名字>.rs`，全量那条标 `#[ignore]`，同文件的快档用例不标。
- 共用的搭建模块经 `#[path = "../../singlefs-harness/tests/common/mod.rs"] mod common;` 这类声明指回 harness 档的 `tests/common*/mod.rs`，不抄第二份。
- 全量那条登记进 `.claude/gate.d/stage-inputs.tsv` 一行 `crash-case:<名>`，第三列 `test=singlefs-checker-tier:<测试目标>:<用例函数>`，计数行、`exhaustive=`、`threads=` 按 `research/scripts/admission.py` 文件头的写法；用例打不出的那一项不登记、在注释里写明。
- 池级 checker 与实现的依赖闭包（dev-dependencies 也算）除 `singlefs-format` 之外不相交，池级 checker 的源码零处引 `singlefs_core`（D13（验证路线） 已定项 5，门禁 94 号判）；它今天只依赖 `singlefs-format`、没有 `[dev-dependencies]`，这一句 94 号不单判。

## 函数名与类型名不许只由空泛词拼成

`run_cell`、`CellRun`、`check_cell`、`fn run`、`fn get` 这一类名字判违规：每一个词都在 `.claude/naming-vague-words` 里，名字就什么都没说。改成说出跑的是什么、判的是哪条、取的是哪个量（`evaluate_stream_domain`、`count_named`、`perform_commit_step`）。`impl <trait> for <类型>` 块里 trait 规定的方法名不判。门禁 13 号判；还没改完名的文件登记在 `.claude/naming-vague-exclude`，只缩不涨。

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

## 门禁管哪一半

| 判什么 | 谁判 |
|---|---|
| 崩溃点重放跑了、快档绿、每条登记用例的全绿标记作不作数 | 54 号；覆盖声明 `# gate-covers: 崩溃点重放`，键登记在项目根 `.claude/gate-not-implemented.tsv` |
| 模型对拍跑了、每段都判过 | 74 号；覆盖声明 `# gate-covers: 模型对拍`，键登记在项目根 `.claude/gate-not-implemented.tsv` |
| 崩溃枚举用例住在 checker 档、标了 `#[ignore]` 的登记了 `crash-case:` | `research/scripts/crash-case-check.py`（47 号跑它的自证） |
| 函数名与类型名不只由空泛词拼成 | 13 号，词表 `.claude/naming-vague-words`、还没改完的文件 `.claude/naming-vague-exclude` |
| 池级 checker 库不依赖实现；harness 档不依赖 checker 档 | 94 号 |
| 谁在什么时候跑得了 checker 档 | `.claude/hooks/heavy-test-guard.sh`，判定在 `lib_heavy_tests.py`：跑到 checker 档包 `singlefs-checker-tier` 的测试（库单测、集成测试、装置二进制的内联测试）、55 / 57 / 59 / 87 号、QEMU、herd7、`crates/mutations.tsv` 整表、全量 `cargo test`、整轮门禁、E152 装置算重型 |

**它们管不到的**：harness 里一条重用例该不该标 `#[ignore]`、快档抽的取样点够不够、全量该多久跑一次——这几样靠人与代码三方。
