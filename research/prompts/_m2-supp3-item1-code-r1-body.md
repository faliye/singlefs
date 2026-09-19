# 背景材料：增补 3 第 1 件（随机历史生成器）的代码三方对抗第一轮（2026-09-18）

<!-- doc-lint:not-numbers W1 W2 W3 W4 W5 W6 Z1 Z2 Z3 Z4 Z5 Z6 -->

## 一、被判的对象

`.claude/kb/milestone/02-second-txn.md`「增补 3　捶打 `crates/`」第 1 件，实现员报告 `research/prompts/m2-supp3-item1-implementer-report.md`（第十二节是打补丁之后的改动），产物在 `research/prompts/m2-supp3-item1-implementer/`。基准提交 `00c9d4f`；两份新文件没进 git，按「文件::项名」给。**开工快照**：`research/prompts/m2-supp3-item1-code-r1-start-snapshot.sha256`（11 个文件）。

| 文件 | 项 |
|---|---|
| `crates/singlefs-harness/src/history.rs`（新，2198 行） | `SeededRandomSource`、`generate_history`、`operation_weights`、`draw_operation`、`expected_session_after`、`HistoryPool`、`apply_operation` 与七个 `apply_*`、`execute_history_observing`、`violations_on`、`KNOWN_RED_FORMS` 与 `only_allocated_statistic_above_walked`、`ring_turn_leaves_allocated_statistic_above_walked`、`raise_after_rollback_leaves_allocated_statistic_above_walked`、`root_ring_has_turned`、`FailureSignature`、`classify_failure`、`simpler_variants`、`shrink_operations`、`shrink_failing_history`、`run_history_campaign`、`HistoryTally` |
| `crates/singlefs-harness/src/crash.rs` | `SparseDevice::read`、`SparseDevice::read_into`（新）、`impl BlockDevice for SparseBlockDevice` 的 `read_at` |
| `crates/singlefs-harness/src/lib.rs` | `pub mod history;`、`SharedStream::operation_count` |
| `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs`（新） | 全部用例，含快档 `random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`、两条已知红的复现、`#[ignore]` 的大档与收缩用例 |
| `crates/mutations.tsv` | 第 129、130 行 |

**实现今天的样子**（主 agent 读 `crates/` 看到的，都是观测；方案按 `crates/` 今天的实现来谈）：

- 随机源是手写 SplitMix64（`SeededRandomSource`），不加依赖。`operation_weights` 按会话状态分三张比重表：没开会话时可写挂载 70、回退 22、冷启动 8；开着、没有文件时发第一个文件 60、空发布 20；开着、有文件时覆盖写 52、抬 F 16、可写挂载 12、回退 10、冷启动 4、发第一个文件 4、空发布 2。
- `execute_history_observing`：每一步之后，流里的步数没变（这一步没发写）就不跑 checker，计进 `checker_runs_skipped_because_nothing_was_written`；发了写就对内存镜像跑 `check_pool_image`，有违例这段历史当场结束。整段包在 `with_panic_capture` 里，panic 同样结束这段历史。入口返回 `Err` 记成 `StepOutcome::Refused`，照常往下走。
- `apply_raise_rollback_floor` 的目标取 `今天的 F + choice % (现行 txg − 今天的 F + 3)`，也就是 [今天的 F, 现行 txg + 2]，会超过上限；不取今天的 F 之下。`publish_without_units` 只在现行那一版没有文件时调。这两条前提的出处写在 `history.rs` 那两处的注释里（实现员报：`crates/singlefs-core/src/mount.rs` 第 590 行、`crates/singlefs-core/src/transaction.rs` 第 506 行、`.claude/kb/decisions/16-发布语义.md` 第 107、192 行）。
- `KNOWN_RED_FORMS` 两条，按次序匹配，第一条命中即算：第 0 条 `ring_turn_leaves_allocated_statistic_above_walked` 只要「根环转过（checker 读法下最新根的 txg ≥ R × S = 24）∧ 没有 panic ∧ 判红的只有 I-3.1（已分配统计对得上） ∧ 违例文字里记账的已分配 > 遍历」；第 1 条 `raise_after_rollback_leaves_allocated_statistic_above_walked` 只要「这一步是抬 F ∧ 根环没转 ∧ 同样的 I-3.1 形态」——它的注释自陈不看 F 是否落在回退留下的空档里（观察里没有回退历史）。两条的 `shape` 文字都比匹配条件窄。
- 撞到已知红的那段历史到此为止，不再往下跑。快档（种子 [0, 96)、每段 30 步、至多 16 个线程）96 段里 50 段以第 0 条收尾、1 段（种子 80）以第 1 条收尾、45 段跑完（主 agent 在工作区亲跑一次的输出行：`历史 96 段：跑完 45、以已知红收尾 {0: 50}、新发现 1；根环转过一圈的 62 段；最高 txg 37`，那一次是打补丁之前）。
- 冷启动 `recovery::recover` 读回的内容不和任何东西比，那是第 2 件模型的事。
- `crash.rs`：`SparseDevice::read` 改成先开一份全 0 的缓冲再调 `read_into`；`read_into` 先把缓冲整段写 0，再对 `sectors` 按区间 `range(first_sector..first_sector + 扇区数)` 查一次，只拷写过的扇区。原来的写法是逐扇区 `get`。层 0 的崩溃点重放经 `SparseBlockDevice::read_at` 用它。
- `crates/` 里没有的：故障注入的通用设备包装（第 4 件）；与理想模型的对拍（第 2 件）；「已知红」匹配条件与回退历史之间的联系（第 1 条的注释自陈）。

## 二、改法（每条都是可以被攻的推论）

| 编号 | 改法 | 压着的条款（原文在附录一） |
|---|---|---|
| W1 | 生成的每段历史都是一个合法调用方调得出来的：只调公开入口，入口前提只取文档注释里那两条 | 里程碑「增补 3」第 1 件与「预想的细节」；D16（发布语义） 已定项 1、已定项 9 |
| W2 | 每一步发了写就跑 checker，panic 与违例算失败，入口返回错误算合法结局，没发写的那一步不跑 checker 不会漏判 | 里程碑「增补 3」第 1 件；`.claude/singlefs-ai-sop/rules/test-discipline.md`「阴性结果要能和「代码没跑到」分开」 |
| W3 | 已知红两条的匹配条件只接走它登记的那一类问题，不遮住别的 | 里程碑「增补 3」预想的细节第一条；增补 2 收口表第 ②、43 行 |
| W4 | 快档有判别力：第 41、121 行改回去都判红，第 129、130 行钉着 | 里程碑「增补 3」验收标准第一条；`.claude/singlefs-ai-sop/rules/test-discipline.md`「检查本身也可能是错的」 |
| W5 | `read_into` 与原来逐扇区读在一切对齐的输入上逐字节相同 | `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」（层 0 用它） |
| W6 | 收口表第 43 行的机理与复现（两次回退、抬 F 到被抛弃实例的 txg；F = 8、9、10 红，7、11、12 不红）与代码、用例对得上 | 增补 2 收口表第 43 行；D16（发布语义） 已定项 1 |

## 三、判据（跑前写死；每条写「什么观测会让它触发」）

| 判据 | 问 | 触发的观测 |
|---|---|---|
| Z1 可达性 | W1：生成器会不会造出合法调用方造不出的历史，让「新发现」是装置造的；两条文档注释前提在条款里有没有依据，漏没漏别的前提 | 一段生成的历史在某一步违反了入口的条款或文档注释，而它的失败正依赖那一步 |
| Z2 遮蔽 | W3：两条已知红的匹配条件会不会接走机理不同的失败 | 构造一段历史或一条对 `crates/singlefs-core`、`crates/singlefs-checker` 的变异，它的失败被第 0 或第 1 条匹配、而根因不是登记的那一类（例：不经回退的抬 F 之后 I-3.1 记账多算，被第 1 条接走） |
| Z3 漏判 | W2：没发写就不跑 checker、撞已知红就停、冷启动读回不比，三样合起来有没有一类可达的错快档与大档都判不出 | 一条落在生成器走得到的路径上的变异（`crates/singlefs-core` 或 `crates/singlefs-checker`），快档与大档都不红 |
| Z4 判别力余量 | W4：第 130 行只有种子 54、69 判得出；换几何（种子数、每段步数、比重）会不会就不红；第 41、121 行附近同类的变异抓不抓得到 | 一个只改快档规模或比重的取样点上第 130 行不红，或一条与第 121 行同类的变异快档不红 |
| Z5 读路径等价 | W5：`read_into` 与原 `read` 在区间端点、没写过的扇区、区间外的扇区上是否逐字节相同 | 一组对齐的 (偏移, 长度, 已写扇区集合) 让两者输出不同 |
| Z6 写回与第 43 行 | W6 与写回：里程碑「增补 3」现状、收口表第 ② 行补的那句、第 43 行与代码、用例、产物对不对得上 | 一句与代码或产物不符；或第 43 行那段复现在工作区上换 F 之后红与不红的格子与第 43 行写的不同 |

**反向接受条款**：Z2 或 Z3 打中 ⇒ 收窄匹配条件或补判定，补会红的用例与变异；Z1 打中 ⇒ 那一类新发现作废、改生成器；Z4 打中 ⇒ 快档加种子或调比重，或给第 121 行那一类补一个专门的取样点；Z5 打中 ⇒ 改回或修 `read_into`，重跑门禁 54 号；Z6 打中 ⇒ 改里程碑文字。第 43 行的修法（抬 F 不许落进空档、checker 候选集改读法、抬 F 时记账放掉那些槽）这一轮不判，归收口表第 ② 行的重议。
**失败条款**：一条腿引的条款与附录原文不符 ⇒ 那一格作废；「没打中」一次不算（本地腿抽两次）；腿在副本上量出的数不进 kb，要在入库装置上重做才引。

## 四、腿的分工（三条推论腿，攻击面不重叠）

| 腿 | 立场 | 分到的格 | 不许碰的格 |
|---|---|---|---|
| Opus 攻方（`three-way-attack`） | 在副本上施加变异、造历史，打穿生成器的判定 | Z2、Z3、Z4 | Z1、Z5 |
| 本地攻方（`three-way-local-attack`，英文，抽两次） | 按事实表逐格算 | Z5（给定 (偏移, 长度, 已写扇区) 的若干格，逐格填新旧两种读法的输出）；Z1 的前提表那一半（每个入口一行：文档注释写的前提、生成器守的前提、两者差在哪） | Z2、Z3、Z4 |
| Sonnet 正推（`three-way-forward`） | 核 W1 与 W6：生成器调入口的方式是不是条款与文档注释允许的，写回的文字是不是代码与产物说的 | Z1 的条款那一半、Z6 | 不替攻方找变异 |

## 五、禁读清单（各条腿）

这一轮别的腿的输出（`research/prompts/m2-supp3-item1-code-r1-*-output*.md`）与主 agent 的核实（`research/prompts/m2-supp3-item1-code-r1-main-verification.md`）。实现员报告与产物可读。本地腿的提示避开 Rust 路径 `::`、不写汉字变量名。
