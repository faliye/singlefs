# 背景材料：里程碑「第二个事务」步 0（固定脚本到 C）与步 3（第二个可写实例）那批代码的三方对抗第一轮（2026-09-17）

<!-- doc-lint:not-numbers S1 S2 S3 S4 S5 S6 S7 S8 S9 S10 S11 X1 X2 X3 X4 X5 X6 X7 X8 X9 X10 X11 M67 -->

## 一、被判的对象

被判的文件（门禁 56 号按路径认）：`crates/singlefs-core/src/mount.rs`（新）、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/unit.rs`、`crates/singlefs-core/src/lib.rs`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker/src/image.rs`、`crates/singlefs-harness/src/crash.rs`；用例 `crates/singlefs-harness/tests/second_transaction_step_three_second_instance.rs`（新）、`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`、`crates/singlefs-harness/tests/common/mod.rs`；变异表 `crates/mutations.tsv`（25 行）。diff 与新文件全文在附录二 `research/prompts/_m2-step3-code-r1-diff.md`（基准：提交 5f9e449）。
落的是 `.claude/kb/milestone/02-second-txn.md` 步 0（固定脚本从 B 扩到 C）与步 3（关闭、重开、取号、写行、暖机、发布 C）；步 3 的「现状」段是主 agent 按代码写的，也在被判之列（X10）。
今天的状态（主 agent 现查，2026-09-17）：`check.sh` 绿（fmt / clippy / 构建 / 单测）；命名纪律绿；步 3 五条验收用例绿；层 0 快的那条 57 个状态绿，全量 789555 个状态的用例交门禁 54 号在 release 下跑；门禁 59 号 25 条变异各红在点名的测试上；门禁 52 号第二条流的段序列与用例里钉的数组、闭式、写数相符；E142（第一个事务的干跑） 装置同步了两条恢复规则（不跨实例、链首接所选根自己那条），产物 `research/results/e142-first-txn-dry-run-2026-09-16-instance-boundary.out` 只有 `verification_ran` 9 → 6 变了。
按 `.claude/rules/implementation-workflow.md`，这批代码提交前要过这一轮；第二轮判决 `research/prompts/m2-code-r2-main-verification.md` 里攻方腿打中的七处改法被攻过零轮（`.claude/rules/three-way-inference.md`「攻方腿自己提的收严，只在它自己的模型上量过，算『没被攻过』」），并入这一轮的 X11。

**实现今天的样子**（主 agent 读过的 `crates/` 路径与看到的事实，都是观测；方案按 `crates/` 今天的实现来谈）：

- `crates/singlefs-core/src/mount.rs` `mount_writable`：`recover` → `rebuild_version`（从盘读实例表、树表、五棵树的根节点、数据单元、inode 叶，都经 `read_unit_via_locations` 读、与恢复走读同一个读法；树表空报 `NoPublishedVersion`）→ `PoolAllocator::rebuild_from_records`（每条记录标已分配、`is_released` 的再进 defer 队列、`open_segment` 留空）→ `acquire_instance`（在 `crates/singlefs-core/src/transaction.rs`：max(超级块的 journal_instance, 根环里最高实例) + 1，逐盘写超级块槽 0，再 `pool.perform(CommitStep::Barrier)`）→ 写行那次 `publish_version`（`InstanceTablePlan::Rewrite`；行 (i, T, W)：上一个实例 T = 生效根的 txg、W = `journal.maximum_applied_transaction`，中间实例 (i, 0, 0)，回退位 0；文件四个角色照抄指针，重写实例表 + 分配记录 + 记账 + 映射 + 树表；txg = max(根环最高 txg, 环里记录最高 txg) + 1；jsn = 环里最大 jsn + 1；事务号 0；反向链 0）→ 空发布循环（条件：区域盘里还有本实例的根没落到的盘，且次数 < `ROOT_RING_REGIONS` = 3；落哪块盘按 `parameters.region_devices[target.region]` 从写出的 txg 算，不读回根环）。
- `crates/singlefs-core/src/recovery.rs` `replay_journal`：候选 = 同实例且 (实例, txg) > 水位；链首 = 所选根自己那条记录（同实例、同 txg）的 jsn + 1，那条读不出时 `expected_next = None`、从候选里最小的一条接；断号即止；`maximum_applied_transaction` 取施加过的记录里最大的事务号。`allocation_records_are_one_per_device` 逐盘核（槽号唯一、同一批 (槽, 跨度, 代, 已释放)、每盘不少于 10 个落点）。`highest_root_txg`、`rebuild_version` 是新函数。
- `crates/singlefs-core/src/transaction.rs`：`TransactionUnit::InstanceTable` 是第九个角色，`IN_BUMP_ORDER` 仍是八个，实例表单元在八个之后写；`PublishPlan { file: Option<FileVersionPlan>, instance_table: InstanceTablePlan::{Carry, Rewrite}, … }`；`publish_version` 先做释放判定（只查不改）与准入（内容 ≤ 32634、分配记录 ≤ 812 条）、再克隆分配器、`publish_admitted` 失败就换回；`placements_to_release_via_mapping(previous, allocator, roles)` 只释放这次重写的角色的落点，跨度取分配记录里的。
- `crates/singlefs-core/src/allocator.rs`：`free_slots` 独立字段（分配时减）；`record_for`；`rebuild_from_records`。
- `crates/singlefs-checker/src/walk.rs` `judge_instance_table_rows`：只对最新根（(txg, 实例) 最大）判，行按实例代号唯一、每行实例 < 最新根的实例、链指针记录（kind 1）在末尾；`crates/singlefs-checker/src/image.rs` `IMPLEMENTED_INVARIANTS` 24 条。
- `crates/singlefs-harness/src/crash.rs`：oracle 按最新持久根的 (txg, 实例) 找版本，`newest_persisted_root` 从根槽写的字节里读实例（偏移 24）与 txg（偏移 28）；`ignored_violations` 另计；`PublishedVersion { instance, checkpoint_txg, content }`。
- `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs`：`Script::{ThroughSecondPublish, ThroughThirdPublish}`；到 C 的流 26 段 `[2,2,1,2,2,1,18,2,1,18,2,1,4,10,2,1,10,2,1,10,2,1,18,2,1,2]`、118 次写、闭式 789555；版本表 (1,3) 第一次内容、(1,4) / (2,5) / (2,6) / (2,7) 第二次内容、(2,8) 第三次内容；五个对照仍在到 B 的脚本上。
- `crates/` 里没有的（主 agent 现查 grep 零命中）：fsync 接口（`grep -rn "fn fsync" crates/`）、干净关闭标记、回退行的写者（`is_rollback` 只有解析与 checker 读它）、可再分配谓词与回收（defer 队列只增不减）、分配记录树的分裂。

## 二、代码做了哪些选择（每条都是可以被攻的推论）

| 编号 | 选择 | 压着的条款（原文在附录一） |
|---|---|---|
| S1 | 前缀规则：只收所选根自己那个实例的记录；链首接在所选根自己那条记录之后（同实例同 txg 的那条，jsn + 1）；那条读不出时从水位之上同实例最小的一条接；断号即止；`maximum_applied_transaction` 报施加过的最大事务号；所选根是 mkfs 的第 0 代根时一条都不施加 | D23（journal 的角色与格式） 已定项 14；I-8.3（重放前缀严格连续） |
| S2 | 重开后的上一版从盘上重建（`rebuild_version`）：读实例表、树表、extent / inode / 分配记录 / 记账 / 映射五棵树的根节点、数据单元、inode 叶；`rewritten` 空、`mapped_units` 从映射节点解；树表空（还没发过文件版本）报 `NoPublishedVersion`，第一版只支持至少发过一版文件的池 | D16（发布语义） 已定项 8；D19（块指针的结构与宽度预算） 已定项 5 |
| S3 | 分配器从分配记录重建：每条记录标已分配，`is_released` 的再进 defer 队列（槽仍占着、不发出）；空闲按位图现数；开放段不续，重开后第一次分配从空闲位图重找最低空槽对；记账行不重算、照上一版记账树里的 | D3（空间分配） 已定项 7 / 已定项 11；D5（快照 / 空间记账机制） 已定项 4 |
| S4 | 取号：max(超级块的 journal_instance, 根环里最高实例) + 1，逐盘写超级块槽 0（世代号 + 1），然后 `acquire_instance` 自己发一道屏障——第二次以后的挂载写行那次发布有单元写，等不到空发布开头那道；首次挂载上它与暖机第一次空发布开头那道背靠背 | D23（journal 的角色与格式） 已定项 16；C322（取号那一步的屏障怎么放没有条款） |
| S5 | 写行：给 [max(所选根的实例, 1), 新实例) 里每个实例各一行，上一个实例 (i, T = 这次恢复生效的根的 txg, W = 这次施加的最大事务号，没施加就 0)，中间实例 (i, 0, 0)，回退位 0；链指针记录照旧在末尾；写行那次发布重写实例表 + 四个固定点单元，文件四个角色照抄上一版的指针；txg = max(根环最高 txg, 记录最高 txg) + 1；jsn 全局接着数；事务号 0；反向链 0 | D18（块里携带什么信息） 已定项 11；D28（挂载期承诺量） 已定项 3；C329（写行那次发布之前推抬 F 的空发布没有检查）；C330（中间实例那一行的 T_pub 取所选根的 txg）；D23（journal 的角色与格式） 已定项 7 / 已定项 19 |
| S6 | 暖机：写行之后连推空发布，直到本实例的根落到每块区域盘上（写行那次的根也算一块），上限 `ROOT_RING_REGIONS` = 3 次；每次空发布重写四个固定点单元（分配记录、记账、映射、树表）、释放上一版的四个；这条脚本上 c_max = 4；`mount_writable` 暖机做完才返回，代码里没有 fsync 接口 | D16（发布语义） 已定项 8 / 已定项 9；D28（挂载期承诺量） 已定项 4；C334（切换的所选根没有会红的检查） |
| S7 | 发布 C：`publish_overwrite`，实例 2、事务号 = 上一版 + 1 = 1、反向链 = 上一条记录头的 CRC32C、txg 8；释放 B 的八个落点（经上一版——txg 7 的暖机输出——的映射节点查落点、按分配记录核跨度）；数据单元落 50184 | D23（journal 的角色与格式） 已定项 7 / 已定项 19；D3（空间分配） 已定项 7 |
| S8 | checker I-3.8：只对最新根（(txg, 实例) 最大）判，行按实例代号唯一、每行实例 < 最新根的实例、链指针记录在末尾；回收条件（删行）没有输入、不判；坏镜像两份（写者多写一行实例 2；改字节多写一行实例 1） | I-3.8（实例表行唯一且低于挂载根） |
| S9 | 层 0 到 C：整条录制流按屏障 / FUA 切 26 段（进程退出与重开之间没有屏障，B 的超级块槽写与重开取号的超级块槽写合成 4 写一段）；oracle 按最新持久根的 (txg, 实例) 找版本；(2, 5)、(2, 6)、(2, 7) 三个根都读第二次内容；不看 journal 那一遍恢复也过同一个 oracle，违例另计 | D13（验证路线） 已定项 4；E77（发布的持久顺序） 判据 1 |
| S10 | E142（第一个事务的干跑） 装置同步：候选只收同实例、链首接所选根自己那条；层 0 `verification_ran` 9 → 6，别的计数不变；单测钉 6、变异表加 M67 | D23（journal 的角色与格式） 已定项 14 第 1 条；门禁 54 号第一条流与产物逐字比 |
| S11 | 第二轮攻方腿打中的七处改法（被攻过零轮）：释放判定路径三种错在动分配器之前报（`ReleaseTargetNotAllocated` / `ReleaseTargetAlreadyReleased` / `ReleaseSpanMismatch`）、跨度取分配记录里的、失败的发布把分配器换回进来时的样子、内容超长报 `ContentExceedsDataUnit`、oracle 补「更新的根下没版本却报没有文件」那一臂、Ignore 那一遍恢复也过 oracle、同盘槽号唯一 | D19（块指针的结构与宽度预算） 已定项 5；D13（验证路线） 已定项 4；第二轮判决第三节 |

## 三、判据（跑前写死；每条写「什么观测会让它触发」）

| 判据 | 问 | 触发的观测 |
|---|---|---|
| X1 前缀五条 | S1 与 D23（journal 的角色与格式） 已定项 14 五条口径逐条对：跨实例、链首、断号、commit 与点名单元、在飞上限；「所选根自己那条读不出时从最小的一条接」有没有条款、会不会把不该接的接上；回退行那第五条今天没有输入（步 4 才有回退行）代码是怎么表达「没有输入」的 | 一段历史让恢复施加了不该施加的记录，或漏施加该施加的 |
| X2 重开重建 | S2 读出的上一版与写它那个进程内存里的 `TransactionOutput` 哪些字段不同（`rewritten` 空、`record_bytes`、`allocation_records` 顺序）、这些不同会不会漏进下一次发布；映射条目、分配记录、树表指针三者互相不一致的镜像重开时会怎样——报错、panic、还是带着不一致往下发布；`NoPublishedVersion` 拒开的池与 D16（发布语义） 已定项 8 冲不冲突（第一次可写挂载之后没发过文件就关掉，再也开不了） | 一份走读能过的镜像重开时 panic 或写出错的东西 |
| X3 分配器重建 | S3 重建出来的已分配 / 空闲 / defer 与记账行（I-3.1（已分配统计对得上） / I-5.2（空闲统计对得上））在写行、暖机、C 之后一致吗；开放段不续的代价；已释放记录的释放代在重建里丢了没有（谓词步 5 要用） | 一个状态让 I-3.1（已分配统计对得上） 或 I-5.2（空闲统计对得上） 红，或重建后的分配器发出一个已释放的槽 |
| X4 写行的 T 与 W | S5 的 T 取「生效根的 txg」（施加记录之后）与 D18（块里携带什么信息） 已定项 11、C330（中间实例那一行的 T_pub 取所选根的 txg） 的字面哪个对；W 取 `maximum_applied_transaction` 与「属于该实例的最大已施加事务号」对不对（没施加写 0 还是写上一版的事务号）；中间实例 (i, 0, 0) 与「实例 0 不写」；写行那次发布的事务号 0 与 D23（journal 的角色与格式） 已定项 7 | 一段历史让下一次恢复按这一行做错（施加过头或停早了） |
| X5 暖机 | S6 的次数按覆盖现算与 D16（发布语义） 已定项 8 的常量 2；上限 3 次不够覆盖时代码做什么（今天：返回、不报错）；「本实例的根覆盖两块盘」用写出的 txg 算落盘、不读回根环，崩溃后重开再算对不对；c_max = 4 与 D28（挂载期承诺量） 已定项 4 现算的值 | 一段历史让 `mount_writable` 返回之后一块盘上没有本实例的根 |
| X6 跨实例的计数 | jsn 全局接着、事务号重置、反向链首条 0、在飞上限按实例还是全局；记录核对器与 checker 认不认这条链 | 一条记录被核对器判错，或两个实例的记录互相冒充 |
| X7 I-3.8 射程 | S8 的「挂载根 = 最新根」在层 0 的中间状态（新实例的根已持久、实例表单元没持久；或反过来）判什么；行只增不删与回收条件；I-3.8（实例表行唯一且低于挂载根） 与 I-7.7（超级块实例代号不低于根环） 的分工 | 一个合法的中间状态被判红，或一个坏镜像判绿 |
| X8 层 0 到 C | S9 的 oracle 假阴性：三个根同内容时靠 (txg, 实例) 分辨——`newest_persisted_root` 读根槽字节的偏移对不对；重开接缝合成 4 写一段对不对（进程退出算不算屏障）；对照仍在到 B 的脚本上，到 C 的脚本没有对照 | 一个状态该判违例而代码判绿，或一个对照搬到到 C 的脚本上不红 |
| X9 装置同步 | S10：E142（第一个事务的干跑） 与 `crates/` 的两条恢复规则是不是同一条（第一条流上 54 号只比计数，计数相同不等于规则相同） | 一个镜像两边恢复结果不同 |
| X10 写回 | `02-second-txn.md` 步 0 / 步 3 的「现状」、`layout/01-first-txn.md` 八的三行「装置钉住」与那句「第二条流」、`invariants.md` I-3.8（实例表行唯一且低于挂载根） 的状态列，与代码对不对得上 | 一句现状与代码不符 |
| X11 第二轮的七处改法 | S11 每一处：三种释放错的判序、跨度取记录时记录与映射不一致、分配器换回时 `records` 与位图一起换回了吗、内容超长的界 32634、oracle 新臂的假阳性、Ignore 那一遍的 `ignored_violations` 有没有漏计、同盘槽号唯一对合法镜像的误判 | 一段历史或一个镜像让改法判错 |

**反向接受条款**：X1–X8、X11 打中 ⇒ 改代码、补一条会红的用例与一条变异行、再攻一轮；打中的格若落在没有条款的地方（X4 的 T 取法、X5 的次数与上限、S3 的开放段）⇒ 记进 `02-second-txn.md` 步 3 的决策点、标预想交用户，代码按最保守的读法改；X9 打中 ⇒ 改 E142（第一个事务的干跑） 装置并重跑产物；X10 打中 ⇒ 改现状。
**失败条款**：一条腿引的条款与附录原文不符 ⇒ 那一格作废；「没问题」一次不算（本地腿抽两次）；腿在副本上量出的数不进 kb，要在入库装置上重做。

## 四、腿的分工（四条腿，攻击面不重叠）

| 腿 | 立场 | 分到的格 | 不许碰的格 |
|---|---|---|---|
| Opus 攻方（`three-way-attack`） | 假设主 agent 的倾向是错的，造可达的历史打穿 | X1、X2、X4、X6、X8 | X3、X5、X7、X11（本地攻方的） |
| 本地攻方（`three-way-local-attack`，英文，抽两次） | 同上，另一组攻击面 | X3、X5、X7、X11 | X1、X2、X4、X6、X8 |
| Sonnet 正推（`three-way-forward`） | 核「代码做的是不是条款说的」：S1–S11 逐条对附录原文；X9、X10 | — | 不判攻方的格 |
| 本地辩方（`three-way-local-defense`，英文，抽两次） | 替六处实做时定下的取法辩护（实例表当第九个角色写在最后；暖机次数按覆盖现算；开放段不续；没有干净关闭标记一律走恢复；链首读不出时从最小的一条接；树表空拒开）：每处先写最强的反对，再答它站不站得住；另复核第二轮判决第三节「空闲独立计数」「释放经映射」两格在重开重建这条路上还成不成立 | — | 不替攻方找反例 |

## 五、禁读清单（各条腿）

这一轮别的腿的输出（`research/prompts/m2-step3-code-r1-*-output*.md`）与主 agent 的核实（`research/prompts/m2-step3-code-r1-main-verification.md`，这一轮结束才有）。前几轮判决可读：`research/prompts/m2-code-r1-main-verification.md`、`research/prompts/m2-code-r2-main-verification.md`。
