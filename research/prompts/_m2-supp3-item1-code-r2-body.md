# 背景材料：增补 3 第 1 件第二轮与层 0 并行化第一轮的代码三方对抗（2026-09-18）

<!-- doc-lint:not-numbers U1 U2 U3 U4 U5 U6 Y1 Y2 Y3 Y4 Y5 Y6 -->

## 一、被判的对象

两样，一轮攻完：

- **甲**：增补 3 第 1 件（随机历史生成器）第一轮判决 `research/prompts/m2-supp3-item1-code-r1-main-verification.md` 之后的改法。实现员报告 `research/prompts/m2-supp3-item1-implementer-report.md` 第十三节，产物在 `research/prompts/m2-supp3-item1-implementer/`（`r2/`、`proof/proofs.log`、`gate59-round2*.log`）。它的第二轮。
- **乙**：层 0 崩溃点重放改多线程（`.claude/kb/milestone/02-second-txn.md` 增补 2 收口表第 41 行；`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」）。实现员报告 `research/prompts/m2-layer0-parallel-implementer-report.md`，产物与两份补丁在 `research/prompts/m2-layer0-parallel-implementer/`。它的第一轮。

基准提交 `00c9d4f`。**开工快照**：`research/prompts/m2-supp3-item1-code-r2-start-snapshot.sha256`（12 个文件）。

| 文件 | 项 |
|---|---|
| `crates/singlefs-harness/src/history.rs`（新，没进 git） | 甲：`KNOWN_RED_FORMS` 第 1 条与它的匹配函数、`raised_floor_lands_only_on_abandoned_roots` 的算法；`HarnessJudgement` 两个成员与 `harness_judgement_of_outcome`、判分配代的那一段；`AppliedEffect::Recovered` 的 `read_back`；`GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR` |
| `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs`（新） | 甲：快档里装得下的四种内容长度的计数断言、`reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms`、`an_allocated_statistic_over_count_after_raising_the_floor_without_a_rollback_is_a_new_finding` |
| `crates/singlefs-harness/src/crash.rs` | 乙：`enumerate_layer0_in_state_slices`、`state_slices`、`evaluate_state_slice`、`Layer0Tally::absorb_following_slice`、`LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE` 与线程数的取法，`enumerate_layer0_*` 一族改走它；新加的单测 |
| `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs` | 乙：改过的用例 |
| `.claude/gate.d/54-layer0-replay.sh` | 乙：线程数的传递、`LAYER0_PARALLEL_FINISHED` 的读法、多核只起 1 个线程判红的那一支 |
| `crates/mutations.tsv` | 甲：第 130 行（改点名）、第 139–141 行；乙：第 131–138 行 |

**实现今天的样子**（主 agent 读 `crates/` 看到的，都是观测；方案按 `crates/` 今天的实现来谈）：

- 甲，第 1 条已知红：匹配函数在原来的三条之外，还要 `raised_floor_lands_only_on_abandoned_roots == Some(true)`（`history.rs` 第 934 行）。这个值在抬 F 之前的镜像上算：新 F 那个 txg 上的根全属于被抛弃的实例（实现员报：读法经由 `singlefs-core` 的 `readable_roots` / `choose_root` / `instance_table_of_root`，checker 的实例表解析不公开）。第 0 条没改宽度，只加了一条：执行器自己判出的失败（`HarnessJudgement`）一律不归已知红。
- 甲，`HarnessJudgement`（第 705 行起）两个成员：`AllocationGenerationIsNotThePublishTxg`——发第一个文件、覆盖写、抬 F 的每一次发布，改写或新增的分配记录分配代要等于这次的 txg，读的是写入进程内存里的分配器记录，写行与暖机那几次发布不在其内；`ColdStartRecoveryFailedOnCheckerGreenImage`——checker 判绿的镜像上冷启动读回失败。
- 甲，第 0 条没按多算的量收窄：实现员报要分「已释放、还在延迟」与「已回收」得读写入进程内存里的分配器，盘上的记账树只给每盘一个延迟总量；主 agent 定不取（收口表第 ② 行）。实现员同时数出：今天的代码上种子 61、81 以第 0 条收尾，多算的 13 个槽里 2 个只被还在根环里的被抛弃根引用。
- 乙，`enumerate_layer0_in_state_slices`（`crash.rs` 第 1224 行起）：按状态序号切成若干片（`state_slices`），工作线程从一个原子计数器领片、算完经通道交给调用线程；调用线程每收到一片打一行 `LAYER0_PROGRESS`，再按片号从小到大合并（`absorb_following_slice`：计数相加，「第一处违例」与 checker 每条不变量的第一处留前面那一片的）并调观察者。带观察者时每片把逐状态的报告留在内存里，等前面的片都并完才交出。工作线程 panic 时立旗，其余线程不再领新片。线程数取 `SINGLEFS_LAYER0_THREADS`，没设取 `available_parallelism`，设成 0 或不是数字就停下。
- 乙，门禁 54 号：线程数没设就取本机核数传进去；成功行报线程数；没显式把线程数设成 1、而本机多于 1 核却只起了 1 个工作线程，判红（实现员报：这一支只拿合成日志核过）；进度行留在阶段输出里；另打第一条流的 `CHECKER` 行。
- 乙，实现员报的读数：两条流的 `LAYER0` / `LAYER0B` / `CHECKER` 行与单线程逐字相同；机器空闲时第一条流 1 线程 202.45 秒、32 线程 7.35 秒，第二条流 32 线程 492.23 秒；主工作区上打了补丁的门禁 54 号还没跑过。
- `crates/` 里没有的：写行与暖机那几次发布的分配代判定；盘上判已分配记录分配代的不变量（收口表第 44 行）；只调 `evaluate_state` 或 `recover` 的重放用例的并行化。

## 二、改法（每条都是可以被攻的推论）

| 编号 | 改法 | 压着的条款（原文在附录一） |
|---|---|---|
| U1 | 第 1 条已知红只在「新 F 那个 txg 上的根全属于被抛弃的实例」时接走，别的抬 F 之后的多算都是新发现 | 第一轮判决第二节第 1 行；增补 2 收口表第 43 行 |
| U2 | 分配代判定与冷启动判定都判得出它们要抓的那一类，而在合法历史上不误红 | 第一轮判决第二节第 3、4 行；增补 2 收口表第 44 行 |
| U3 | 第 0 条维持原宽度是对的：用写入进程的内存给实现的失败归类不取，而用它判实现的记账（U2 的分配代判定）取 | 第一轮判决第二节第 2 行；增补 2 收口表第 ② 行 |
| U4 | 专门取样点与内容长度计数给了第 121 行那一类与 B1 足够的判别力 | 第一轮判决第二节第 5、7 行 |
| U5 | 并行版在任何状态分布、线程数、观察者用法下，输出与单线程逐字相同，不死锁、不漏片 | 增补 2 收口表第 41 行；`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」 |
| U6 | 门禁 54 号在多核机器上只起 1 个线程时判红，在合法的并行跑上不误红 | 同 U5 |

## 三、判据（跑前写死；每条写「什么观测会让它触发」）

| 判据 | 问 | 触发的观测 |
|---|---|---|
| Y1 收窄之后的第 1 条 | U1：还遮不遮别的机理；会不会把登记的那一类（F 落在空档里）错判成新发现；判「全属于被抛弃的实例」用的是实现自己的读根与读实例表，那几个函数错了时第 1 条会不会跟着错 | 一段历史或一条对 `crates/singlefs-core`、`crates/singlefs-checker` 的变异，失败被第 1 条接走而 F 不在空档里；或 F 在空档里、机理是登记的那一类，却报新发现 |
| Y2 执行器的两条判定 | U2：合法历史上会不会误红；写行与暖机那几次发布不判，同类变异落在那里还抓不抓得到 | 一段基线历史被判 `HarnessJudgement`；或一条只改挂载路径里记录改写的变异，快档与专门取样点都不红 |
| Y3 第 0 条与两处内存 | U3：拒绝用写入进程的内存给失败归类、却用它判记账，这两个决定是不是同一个理由的两面；种子 61、81 那 2 个槽是不是别的机理 | 说得出一种情形：按同一个理由，其中一个决定该反过来 |
| Y4 判别力 | U4：专门取样点的判出率与窗口断言、四种内容长度的计数，换几何还成不成立 | 一个只改种子区间、每段步数或比重的取样点上，第 130 行或 B1 不红 |
| Y5 并行等价 | U5：切片的边界、合并次序、第一处违例、观察者的次序与内存、工作线程 panic、线程数的取法 | 一组输入（段、展开谓词、线程数、观察者）让并行与单线程的计数行或观察者看到的次序不同，或卡死，或漏片、重片而断言不红 |
| Y6 门禁 54 号与写回 | U6 与写回：判红那一支、`LAYER0_PARALLEL_FINISHED` 的读法、进度行混在输出里对 `LAYER0` 行的读法；增补 3 现状、收口表第 ②、41、43、44 行、规则那一节与代码和产物对不对得上 | 一份日志让门禁在只起 1 个线程时判绿，或在合法并行跑上判红；或一句与代码不符 |

**反向接受条款**：Y1、Y2、Y5 打中 ⇒ 改代码、补会红的用例与变异；Y3 打中 ⇒ 改决定或改写理由，交用户时写明；Y4 打中 ⇒ 加种子或补取样点；Y6 打中 ⇒ 改门禁脚本或文字，门禁脚本的改动带判别力样本。甲是第二轮：它的打中改完之后不再开第三轮（`.claude/rules/three-way-inference.md`「第三轮之后停」按同一条精神），改法写明被攻过几轮。乙是第一轮。
**失败条款**：一条腿引的条款与附录原文不符 ⇒ 那一格作废；「没打中」一次不算（本地腿抽两次）；腿在副本上量出的数不进 kb。

## 四、腿的分工（三条推论腿，攻击面不重叠）

| 腿 | 立场 | 分到的格 | 不许碰的格 |
|---|---|---|---|
| Opus 攻方（`three-way-attack`） | 在副本上施加变异、造历史与状态分布，打穿甲与乙 | Y1、Y2、Y5 | Y3、Y6 |
| 本地攻方（`three-way-local-attack`，英文，抽两次） | 按事实表逐格算 | Y5 的算术那一半（给定状态数与片数，逐格填每一片的区间、合并之后第一处违例取哪一片的）；Y6 的日志读法那一半（给定几组日志行，逐格填门禁判红还是判绿） | Y1、Y2、Y3 |
| Sonnet 辩方（`three-way-defense`） | 复核第一轮判决与它之后的两个决定，替被否掉的做法辩护 | Y3、Y4、Y6 的写回那一半 | 不替攻方找变异 |

## 五、禁读清单（各条腿）

这一轮别的腿的输出（`research/prompts/m2-supp3-item1-code-r2-*-output*.md`）与主 agent 的核实（`research/prompts/m2-supp3-item1-code-r2-main-verification.md`）。第一轮的判决与三条腿的输出可读。本地腿的提示避开 Rust 路径 `::`、不写汉字变量名。
