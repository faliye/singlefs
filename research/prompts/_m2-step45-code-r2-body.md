# 背景材料：里程碑「第二个事务」步 4 / 步 5 那批代码的三方对抗第二轮（2026-09-17）——第一轮打中之后的五处改法

<!-- doc-lint:not-numbers S1 S2 S3 S4 S5 S6 X1 X2 X3 X4 X5 X6 X7 P1 P2 -->

## 一、被判的对象

第一轮（`research/prompts/m2-step45-code-r1-main-verification.md`）打中五格，改法都已落地、被攻过零轮；这一轮只攻这五处改法本身与第一轮判决第六节里代码没动的格，不重复第一轮攻过的角度。被判的文件（门禁 56 号按路径认）：`crates/singlefs-core/src/mount.rs`（`mount_rollback` 的 P2 与 F_生效 候选集、`rebuilt_allocator` 的影子账、`reclaim_floor`）、`crates/singlefs-core/src/allocator.rs`（`isolate` 放开已分配位、`rebuild_from_records` 认第 0 版树表）、`crates/singlefs-harness/src/crash.rs`（撤回记录核对器那条「合法覆盖」）；用例 `crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs`（`rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation`）、`second_transaction_step_five_reuse.rs`（`roots_below_a_floor_carried_by_only_one_device_remain_rollback_candidates`）、`crates/singlefs-core/src/mount.rs` 与 `allocator.rs` 里的两条单测；变异表 `crates/mutations.tsv` 44 行。diff 在附录二 `research/prompts/_m2-step45-code-r2-diff.md`（基准：提交 5f9e449；第一轮判过的那一半不再判）。
今天的状态（主 agent 现查，2026-09-17）：`check.sh` 绿；命名纪律绿；门禁 59 号 44 条变异各红；52 号绿；层 0 快的那条 108 个状态零违例，全量（2104413）交门禁 54 号在跑。

**实现今天的样子**（主 agent 读过的 `crates/` 路径与看到的事实，都是观测；方案按 `crates/` 今天的实现来谈）：

- `crates/singlefs-core/src/mount.rs` `mount_rollback`：候选集 = 目标根在环里 ∧ 按最新根实例表有效（无那个实例的行，或 T ≤ Ti）∧ `target.checkpoint_txg ≥ effective_rollback_floor(...)`（各幸存盘所带 F 最大值的最小值）；R_old 自己那条记录读不出时拿环里最大 jsn 那条顶着（只有 jsn 会被用到）；`next_counter = 环里最大 jsn + 1`（P2）；`rebuild_version(R_old)`；`rebuilt_allocator(devices, superblock, previous, &newly_abandoned, shadow_ledger)`，`newly_abandoned(root) = (root.txg, root.instance) > (R_old.txg, R_old.instance)`。
- `mount.rs` `rebuilt_allocator`：从上一版分配记录重建 → `effective_floor` → `roots = readable_roots` → `newest_table = choose_root → instance_table_of_root`（读不出 ⇒ `None`，没有按表判的被抛弃根）→ `is_abandoned(root) = extra_abandoned(root) || 表里有它那个实例的行且 root.txg > T` → `oldest_valid_root = 不被抛弃的根里最小 txg` → `reclaim_released_up_to(reclaim_floor(effective_floor, oldest_valid_root))`（`reclaim_floor` = max）→ `ShadowLedger::On` 时对每条被抛弃根读 `allocation_records_under_root`，跳过 `is_released` 的，其余逐槽 `isolate_abandoned`。可写挂载传 `&|_| false, ShadowLedger::On`。
- `allocator.rs` `DeviceFreeMap::isolate`：不再断言未分配；隔离位独立于分配位；`is_free` 与 `lowest_empty_segment` 绕开隔离位；`isolated_slots` 计数（`MountOutput::isolated_slots_per_device` 报它）。`PoolAllocator::rebuild_from_records` 末尾：分配代 0、跨度 1、未释放的记录 ⇒ `format_time_tree_table = Some(那个落点)`。
- `mount.rs` `raise_rollback_floor`：`oldest_valid_root` 按 `current` 的实例表判（不含这次抬 F 之后的变化）；回收门槛 `reclaim_floor(new_floor, oldest_valid_root)`。
- `crates/singlefs-harness/src/crash.rs` `check_records`：`root_without_record` 回到「根在盘上而那次发布的记录一份都不在」。
- 固定脚本到 E 的写数、段序列不变（P2 只改记录落在哪个槽，不改写请求数）；E 的数据单元落 50178。
- `crates/` 里没有的：回退行不依赖新实例根的落点（前缀第五条仍是死条款）；盘上的影子账字段；根环绕圈（24 槽）的用例（`reclaim_floor` 只有单测）。

## 二、五处改法（每条都是可以被攻的推论）

| 编号 | 改法 | 压着的条款（原文在附录一） |
|---|---|---|
| S1 | P2：回退之后新实例的第一条 jsn = 环里最大 + 1，被抛弃发布的记录原样留在环里，靠候选集与实例表挡 | D23（journal 的角色与格式） 已定项 14 回退段「崩在它之前……与没发起回退时相同」、第 3 条「计数器全池接着走」；C340（回退之后记录链从哪条之后接没有定义） |
| S2 | 影子账每次挂载在重建里算（按最新根实例表判被抛弃的根，回退时再加新抛弃的），按 D23 主句的保守读法隔离被抛弃根账里还分配着的每个槽——含 R_old 也引用的；账里已释放的不算引用；只住内存 | D23（journal 的角色与格式） 已定项 14 影子账那一句；D28（挂载期承诺量） 已定项 1 第九项；C314（回退可以复用被抛弃的根引用的单元） |
| S3 | 回退候选集的 F 用 F_生效（各幸存盘所带 F 最大值的最小值） | D16（发布语义） 已定项 1 生效与回退候选集两行 |
| S4 | 回收门槛 = max(F_生效, 环里最旧有效根)，有效 = 可读 ∧ 按实例表不被抛弃 | D16（发布语义） 已定项 1 可再分配那一行 |
| S5 | 重建分配器认得出还没被换下的第 0 版树表单元（分配代 0、跨度 1、未释放） | D3（空间分配） 已定项 7；[layout/01-first-txn.md](../../.claude/kb/layout/01-first-txn.md) 一 |
| S6 | 撤回记录核对器「被后来的记录合法覆盖不算洞」 | E77（发布的持久顺序） b_ur 臂 |

## 三、判据（跑前写死；每条写「什么观测会让它触发」）

| 判据 | 问 | 触发的观测 |
|---|---|---|
| X1 P2 的代价 | 被抛弃的记录留在环里：所选根退到实例 1 的旧根（B、C、实例 3 的根都读不出）时 B 的记录被接上、回退被撤销——这是 C332（回退实例两个根都读不出时回退被撤销） 那一族还是新的？被抛弃实例 2 的记录在所选根是 (1, 4) 时（P2 下它自己那条记录在）会不会被接上；环绕圈之后被抛弃的记录槽被盖的那一刻谁在算；jsn 全池最大 + 1 与 D23 第 3 条「从前缀末 + 1 接着写」字面对不对 | 一段历史让被抛弃的记录被施加，或 P2 比 P1 多丢一条已提交的记录 |
| X2 影子账每次挂载 | 最新根的实例表读不出 ⇒ `newest_table = None` ⇒ 按表判的被抛弃根一条都不隔离（只剩回退新抛弃的）——这一格合不合 D23 主句；被抛弃根的树表或分配记录树读不出时 `rebuilt_allocator` 整个报错、挂载开不了；保守读法把 R_old 也引用的槽隔离，它被释放、回收之后一直发不出去，直到被抛弃根离开根环——第一版 24 槽走不到，账里的空闲与 `df` 报的差多少（D28 第九项只报独占量）；`is_released` 跳过 = 「引用」的定义对不对（被抛弃根账里已释放但它的上一版还引用的槽） | 一个只被被抛弃根引用的槽被发出，或一个谁都不引用的槽被永远隔离，或一个合法镜像挂载开不了 |
| X3 F_生效 的候选集 | 抬 F 生效、回收、复用之后，一块盘的载体根坏了 ⇒ F_生效 回落 ⇒ 更老的根重新成为候选，而它们引用的单元已经被合法复用——回退到那样的根会怎样（`rebuild_version` 读到被盖的单元报错？走读半路 panic？）；候选集用 F_生效 而 checker 的候选集用最新根自己的 F，两边不一致的那段时间里 I-2.1 / I-3.1 判什么 | 一个候选根退过去之后走读失败或写出错的东西 |
| X4 回收门槛 | `oldest_valid_root` 在抬 F 里按 `current` 的实例表判、在重建里按最新根的表判——两处会不会算出不同的最旧根；`reclaim_floor` 取 max 在 F_生效 > 最旧根时退化成 F_生效，第 0 代根被盖之后（txg ≥ 25）谁先到 | 一个状态让释放代 ≤ 门槛的槽没回收，或 > 门槛的被回收 |
| X5 第 0 版树表 | 「分配代 0、跨度 1、未释放」认第 0 版树表：mkfs 只写两个单元（实例表跨 2、树表跨 1），别的分配代 0 跨度 1 的记录第一版有没有；重建之后第一个文件版本走 `publish_version(previous = None)` 那条路吗（今天走不到：没发过文件的池开不了可写挂载） | 一条不是树表的记录被当成树表释放，或树表单元漏释放 |
| X6 代码没动的格 | 前缀第五条死条款（回退行只挂在新实例的根下面）今天的代码是不是最保守的可行读法；X1 ①（回退实例的根都读不出时退到被抛弃根被接受）今天有没有更保守的判法（例：候选集里先排除最新根自己被抛弃的可能）；本地攻方 2a「非空按记录事务号认」偏低是不是安全方向 | 一格能在今天的代码里做得更保守而没做 |
| X7 写回 | `02-second-txn.md` 步 4 / 步 5 现状、`second-txn-layout.md`、第一轮判决第五节，与代码对不对得上 | 一句现状与代码不符 |

**反向接受条款**：X1–X5 打中 ⇒ 改代码、补一条会红的用例与一条变异行、再攻一轮（第三轮）；打中的格落在没有条款的地方 ⇒ 记进 `02-second-txn.md` 对应步的决策点、标预想交用户，代码按最保守的读法改；X6 打中 ⇒ 按它说的更保守读法改代码或记决策点；X7 打中 ⇒ 改现状。
**失败条款**：一条腿引的条款与附录原文不符 ⇒ 那一格作废；「没问题」一次不算（本地腿抽两次）；腿在副本上量出的数不进 kb。

## 四、腿的分工（四条腿，攻击面不重叠；这一轮云端那条是辩方，复核第一轮的判决）

| 腿 | 立场 | 分到的格 | 不许碰的格 |
|---|---|---|---|
| Opus 攻方（`three-way-attack`） | 造可达的历史打穿五处改法 | X1、X2、X3 | X4、X5、X6（本地攻方的） |
| 本地攻方（`three-way-local-attack`，英文，抽两次） | 同上，另一组攻击面 | X4、X5、X6 | X1、X2、X3 |
| Sonnet 辩方（`three-way-defense`） | 复核第一轮判决 `m2-step45-code-r1-main-verification.md` 的每一格：打中的够不够得着、改法是不是同样修好所有替代方案、没打中的是不是真没打中；X7 | — | 不替攻方找反例 |
| 本地辩方（`three-way-local-defense`，英文，抽两次） | 替五处改法辩护（P2；影子账保守读法且每次挂载算；F_生效 做候选集；回收门槛带最旧有效根；重建认第 0 版树表）：每处先写最强的反对再答 | — | 不替攻方找反例 |

## 五、禁读清单（各条腿）

这一轮别的腿的输出（`research/prompts/m2-step45-code-r2-*-output*.md`）与主 agent 的核实（`research/prompts/m2-step45-code-r2-main-verification.md`）。前几轮判决可读：`m2-step45-code-r1-main-verification.md`、`m2-step3-code-r1-main-verification.md`、`m2-code-r1-main-verification.md`、`m2-code-r2-main-verification.md`；第一轮四条腿的报告可读（攻方腿不许重复它们攻过的角度）。本地腿的提示避开 Rust 路径 `::`。
