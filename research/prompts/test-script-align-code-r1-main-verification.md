# test-script-align-code-r1 判决（2026-10-01，代码轮第一轮）

正文 `research/prompts/_test-script-align-code-r1-body.md`；腿：正推 `research/prompts/test-script-align-code-r1-sonnet-output.md`（K1、K4、K5）、攻方 `research/prompts/test-script-align-code-r1-opus-output.md`（K2、K3），模型目录各自 `-model/`。

被判的 `crates/*/src/*.rs`：`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`、`crates/singlefs-checker-tier/src/crash.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`、`crates/singlefs-harness/src/history.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs`；连同正文第一节那张表里的测试文件与 `crates/mutations.tsv` 第 1573–1610 行。

## 一、核查

- 开工快照：冻结副本 `/tmp/claude-1000/test-script-align-code-r1/tree/` 对 `research/prompts/test-script-align-code-r1-snapshot/crates-sha256.txt` `sha256sum -c` 全对。
- `python3 research/scripts/cite-check.py <两份报告> --root . --background research/prompts/_test-script-align-code-r1-background.md`：「核了 79 处引文，4 处对不上，0 处没判」。四处都在正推报告，引文是对的、行号写偏：第 51 行（量显存那条 `#[ignore]` 不在 1592 行）、第 52 行（`crash_judge_dispatch.rs` 那句在第 997、1121 行，不在 977）、第 85 行（`SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO` 在第 56 行，不在 54）、第 169 行（`CRASH_INJECTION_VARIABLE_PREFIX` 在第 2317 行，不在 1983）；判决按改正后的行号采。
- 主 agent 在冻结副本上现查：
  - R1：`crash_identity.rs` 第 88–113 行 `StepAttributes` 只有 `write_table_with_barriers` 与 `parent_output` 两样进复用键；`crash_amplification.rs` 第 303–313 行计划哈希只写段序、状态数、展开方式；`versions`、`judged_root_index`（第 122–123 行）两处都不进；第 337–339 行判法摘要「设点的测试文件、流水线自己变了，摘要不变」。攻方的机制成立。
  - C604 一半：`tests/record_checker_judges_absence_by_the_persisted_set.rs` 第 1081–1084 行 σ 对拍 `.expect("本机要有一张空闲显存够的 NVIDIA 独显…")`，没卡时 panic，不报本次未跑。
  - C606 一半：`tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 第 55–58 行 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO` 解析失败 `.unwrap_or(DEFAULT_EXPAND_UP_TO)` 悄悄退回默认。
  - K4：`0x10DE` 在 `tests/common_gpu_adapter/mod.rs:12` 与 `src/gpu_unit_checks.rs:17` 两处定义。
  - 攻方 L1、G1、S2、M3 的数都是它草稿副本上原型量的，不是入库装置的数；主 agent 没在入库装置上重做，照机制读代码采（`.claude/rules/three-way-inference.md`「副本上的数不进 kb」：这一轮只拿它定改法，不把数写进 kb）。

## 二、判定与去向

| 格 | 判定 | 去向 |
|---|---|---|
| R1 复用键不罩版本表与 `judged_root_index` | 打中，最重 | 改代码：判器读到的每一样输入都进那一块的键（或判法版本），两条流写表相同、只差版本表时不得互相复用；前缀共享在判器输入真相同时保留。要能红的对照：同一判法版本下「只差版本表」的一对，两个次序都红。改法由实现员定（攻方的 A 丢前缀共享、A2/B 零轮），依据写进报告 |
| L1 叫停那一趟驱动行写 `states_reused=0` | 打中 | 改代码：叫停、导入、只排计划三处的复用数照实报，取不出的写「取不出」，不写 0（上游「入口脚本」那一节） |
| G1 / C620 有卡而驱动坏被当成本次未跑 | 打中（两条腿一致） | 改代码：`decide_gpu_test_start` 加一个输入：PCI 上有没有 NVIDIA 显示控制器；PCI 有、Vulkan 枚举没有就判红 |
| S2 崩溃注入的重放行指到只判新发现的大档入口 | 打中 | 改代码：重放行指回出红的那条用例本身（或大档入口另带「照哪一档的断言判」），照抄那一行跑要红在同一条断言上 |
| M3 第 1587 行「复用键不罩写表」改坏后对照不红 | 打中 | 改代码：对照里加一对「父镜像相同、写表不同、有状态」的点，让这一行变异会红 |
| K1 C604 σ 对拍、量显存与计时那几条 `#[ignore]` 用例没卡时 panic 或静默通过 | 打中 | 改代码：都经 `common_gpu_adapter` 取卡，没卡报本次未跑，有卡起不来判红 |
| K1 C606 GPU 对拍的展开上限变量写错悄悄退回默认 | 打中 | 改代码：写错就 panic，带变量名与原值，与 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO` 同一判法 |
| K1 C607 `run_history_campaign` 装配 `sampling`（`history.rs:5067`）没有自证也没有变异；坏盘输入与故障注入的重放行没动 | 打中 | 改代码：装配那一步加自证与变异；坏盘输入、故障注入的重放行照随机历史那张变量表带全参数 |
| K1 C611 `by=wall_clock` 那一臂的打印没有用例核 | 打中 | 改代码：补用例（假时钟）与变异 |
| K4 `0x10DE` 两处定义；`history.rs:5141` 手写 `5 * 2 * 4 * 2`；`GenerationWeights::NAMED` 手抄五个常量 | 打中（小） | 改代码：常量一处定义；乘积从各枚举的成员数算；`NAMED` 改成穷举 `match` 生成，漏掉新常量编不过 |
| K4 名字三处（`a_line_was_printed_at`、`ExpandUpToWritesPerSegment` 的 `.0`、`ALL`）、错误口径不一 | 可议，不采 | 名字说得出它是什么；复现入口走 `Result`、展开上限走 panic 是两种调用方（入口报给人、库内不变量），不统一 |
| K5 GPU 探针用例一条做两个场景、没喂「绿」那一支 | 打中 | 改代码：拆成每个场景一条，补「有卡起好」那一支 |
| K5 pipeline 驱动行字段用例 `set_var` 三个变量不还原、读调用方没清的变量 | 打中 | 改代码：用例自己清、自己还原（守卫），不读别的用例与调用方留下的环境 |
| K2 没打中的（E165 五对列、零态、ignore 全跳过、展开上限 panic 被吞、36 段随机历史复现） | 只抽一次 | 不作「没打中」用（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」），第二轮换攻击面再攻 |
| K3 其余 33 行 | 推的会红 | 照旧，提交时 checker-tier-crates-mutation-replay 复跑；第 1575 行红在防御的 `assert_eq!` panic 上，记一笔，第二轮看要不要改成断言 `is_err` 那一判 |

打中的改法都被攻过零轮，第二轮攻改后的代码。

## 三、交给下一批的实现员（按文件聚簇）

| 簇 | 关的格 | 要动的文件 |
|---|---|---|
| 崩溃放量 | R1、L1、M3、C611、K5 pipeline 用例 | `crates/singlefs-checker-tier/src/crash_amplification.rs`、`src/crash_identity.rs`（要动时）、`tests/crash_amplification_*.rs`、`tests/common_crash_points/mod.rs` |
| GPU | G1、C604 那几条、C606 GPU 变量、`0x10DE`、K5 探针 | `tests/common_gpu_adapter/mod.rs`、`tests/gpu_adapter_probe_*.rs`、`tests/record_checker_judges_absence_by_the_persisted_set.rs`、`tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`src/crash_judge_dispatch.rs`、`src/gpu_unit_checks.rs` |
| 复现行 | S2、C607 两条、`5 * 2 * 4 * 2`、`NAMED` | `crates/singlefs-harness/src/history.rs`、`tests/random_histories.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs`、`tests/crash_injection_campaign.rs`、坏盘输入与故障注入的重放行所在文件 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 9 | 不影响 | 2026-10-01 不受影响：这一轮判的是实现，全量档的定案不动 |
| D24（后台重活能不能卸给 GPU） | 不影响 | 2026-10-01 不受影响：G1 改的是测试取卡的判定，不改哪类计算走 GPU |
