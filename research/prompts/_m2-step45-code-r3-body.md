# 背景材料：里程碑「第二个事务」步 4 / 步 5 那批代码的三方对抗第三轮（2026-09-17）——第二轮与 alloc-basis 第二轮打中之后的三处改法

<!-- doc-lint:not-numbers T1 T2 T3 X1 X2 X3 X4 X5 X6 X7 X8 G5 P1 P2 -->

## 一、被判的对象

第二轮（`research/prompts/m2-step45-code-r2-main-verification.md`）打中之后落地了两处改法，alloc-basis 第二轮的云端攻方腿（`research/prompts/alloc-basis-r2-opus-output.md` 2.1 节与第七节第 1 条）又打中步 5 两处、落地了第三处改法，三处都被攻过零轮；这一轮只攻这三处，以及第二轮判决第六节里代码没动、但改法方向已经列出来的那一格（F_生效 回落）。被判的文件（门禁 56 号按路径认）：`crates/singlefs-core/src/mount.rs`（`isolate_slots_referenced_only_by_abandoned_roots`、`rebuilt_allocator`、`raise_rollback_floor`、`MountOutput::abandoned_roots_unreadable`、`MountError::RaiseNeedsWritableMountInThisProcess`）、`crates/singlefs-core/src/allocator.rs`（`ReclaimedReuse`、`DeviceFreeMap::hold_until_floor_takes_effect` / `release_holds`、`PoolAllocator::release_reclaim_holds`）、`crates/singlefs-core/src/recovery.rs`（`allocation_records_under_root` 的单层守卫）、`crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs`、`crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs`、`crates/mutations.tsv`。
今天的状态（主 agent 现查，2026-09-17）：`check.sh` 绿；命名纪律绿；门禁 59 号 51 条变异在跑；层 0 快的那条 108 个状态零违例，全量（2104413）待在改完的树上重跑。

**实现今天的样子**（主 agent 读过的 `crates/` 路径与看到的事实，都是观测；方案按 `crates/` 今天的实现来谈）：

- `crates/singlefs-core/src/mount.rs` `isolate_slots_referenced_only_by_abandoned_roots(devices, allocator, roots, is_abandoned, floor, current_records)`：豁免集 = 当前这一版账里未释放的记录 ∪ 每条候选根（`!is_abandoned ∧ txg ≥ floor`）账里未释放的记录（候选根的账读不出就当它什么都不豁免）；然后对每条被抛弃根：账读不出 ⇒ `unreadable += 1; continue`；账里每条未释放、不在豁免集、没隔离过的记录 ⇒ `allocator.isolate_abandoned`。返回读不出的条数。
- `rebuilt_allocator(devices, superblock, previous, extra_abandoned, shadow_ledger) -> RebuiltAllocator { allocator, effective_floor, abandoned_roots_unreadable }`：不再返回 `Result`；`is_abandoned = extra_abandoned || 按最新根实例表判`；`floor = effective_floor`（F_生效）；`current_records = previous.allocation_records`；`ShadowLedger::Off ⇒ 0`。
- `raise_rollback_floor(parameters, devices, allocator, current, new_floor, shadow_ledger)`：`current.units` 里没有实例表单元（这个进程没做过可写挂载）⇒ `RaiseNeedsWritableMountInThisProcess` → 上限检查 → `oldest_valid_root` 按 `current` 的实例表判 → `ShadowLedger::On` 时用 `readable_roots`、`abandoned_by_table(root, &table)`、`new_floor`、`current.allocation_records` 再调一次隔离 → `reclaim_released_up_to(reclaim_floor(new_floor, oldest_valid_root), ReclaimedReuse::HeldUntilFloorTakesEffect)`：位图清掉、空闲计数加、defer 减，但每个回收的槽在 `DeviceFreeMap` 上另标「扣住」，`is_free` 与 `lowest_empty_segment` 都绕开它 → 推空发布直到两块盘都有带新 F 的根 → `release_reclaim_holds()` 放开。重建那一路（`rebuilt_allocator`）回收用 `ReclaimedReuse::Immediately`。隔离在回收之前；隔离位、扣住位都独立于分配位。
- `allocator.rs` `DeviceFreeMap::isolate`：同一个槽隔离两次不重复计数；`is_free` 与 `lowest_empty_segment` 绕开隔离位与扣住位；`mark_reclaimed` 不看隔离位、不看扣住位。`allocate_commit_generated` 开放段满了就 `lowest_empty_segment`（used、isolated、held 都为 0 的最低段）。
- `recovery.rs` `allocation_records_under_root`：读树表 → 找 `TREE_KIND_ALLOCATION` 条目 → 读它指的节点 → `level != 0 ⇒ UnitMalformed("分配记录树根不止一层")` → 逐条解成 `AllocationRecord`。
- 固定脚本（步 0 到 E）：回退那次挂载隔离逐盘 34（只被 B、写行、暖机、C 引用的）；抬 F 到 11 之后回收之前补隔离 mkfs 实例表 50176–50177（A 不再是候选、只剩被抛弃的 B 引用）；E 落 50178（mkfs 树表，A 的版本释放、只被第 0 代根引用）。复用窗口置 0（不抬 F 直接回收到 11）时 E 落 50176，checker I-2.1（校验和与内容匹配） 红。C 的树表两盘都改坏 ⇒ 挂载成功、计 1、隔离 24。
- `MountOutput::abandoned_roots_unreadable` 今天没有消费者：只有用例读它。
- 用例 `slots_reclaimed_by_raising_the_floor_are_not_handed_out_before_the_floor_takes_effect`（A、B、重开取号 2、六次覆盖写 txg 8–13、抬 F 到 8）：扣住之前 txg 14 的记账树落在回收的 50240（A 的 extent 根）上，扣住之后两次空发布一个落点都不在回收的槽上。`raising_the_floor_in_a_process_that_never_mounted_writable_is_refused_instead_of_panicking`：只 mkfs 加第一个事务就抬 F 报错不 panic。
- `crates/` 里没有的：盘上的影子账字段；「读不出账的被抛弃根」的任何条款或不变量；F 只许涨的守卫；根环绕圈（24 槽）的用例；同一次挂载里转环时的回收（回收只在重建与抬 F 两处做——alloc-basis 第二轮云端攻方腿 1.1 / 1.2 节，固定脚本走不到 txg 24，口径归 alloc-basis 那一轮）。

## 二、三处改法与一个方向（每条都是可以被攻的推论）

| 编号 | 改法 | 压着的条款（原文在附录一） |
|---|---|---|
| T1 | G5 影子账：只隔离被抛弃根账里未释放、且不被任何候选根（可读 ∧ 按实例表有效 ∧ txg ≥ F）也不被当前账未释放记录引用的槽；每次挂载算，抬 F 之后按新 F 在回收之前再算一次。对用户 2026-09-16 窄读法措辞「仍被有效根引用的槽不在其内」的收严（原措辞的有效只按实例表判），预想、偏离用户定案的措辞 | D23（journal 的角色与格式） 已定项 14 主句与窄读法那句、D28（挂载期承诺量） 已定项 1 第九项、D16（发布语义） 已定项 1 生效 / 可再分配两行 |
| T2 | 被抛弃根的树表或分配记录树读不出、解不开：跳过那条根、计数、不拒绝挂载；它引用的槽罩不到 | D23（journal 的角色与格式） 已定项 14 主句（「它们引用的单元不许重新分配」）；没有条款说读不出时怎么办 |
| T3 | 抬 F 回收的槽扣住到 F 在每块盘上生效（带新 F 的根落满每块盘）才能发出去；记账（空闲、defer）仍在写第一条带新 F 的根之前动。记账按写那条根那一刻的 F 还是它持久之后的 F_生效 算，口径归 alloc-basis 那一轮；只 mkfs 加第一个事务的进程抬 F 报 `RaiseNeedsWritableMountInThisProcess` | D16（发布语义） 已定项 1 生效与可再分配两行；I-3.1（已分配统计对得上） 的并集口径 |
| 方向 | F_生效 回落那一格（第二轮 X3-A）的三个改法：(a) 新实例的根写 max(上一条根的 F, F_生效)；(b) 回退候选集也用 max；(c) 只改 checker。代码没动，交用户 | D16（发布语义） 已定项 1 生效与回退候选集两行 |

## 三、判据（跑前写死；每条写「什么观测会让它触发」）

| 判据 | 问 | 触发的观测 |
|---|---|---|
| X1 T1 的隔离集 | 有没有可达的历史让一个只被被抛弃根引用的槽被发出去（豁免集算宽了）；或让一个候选根引用的槽被隔离（豁免集算窄了）：当前账未释放的记录、候选根的账、被抛弃根的账三者之间，释放代与 F 的关系上有没有漏格 | 一段历史让 `is_free` 对只被被抛弃根引用的槽返回真，或 `isolated_slots` 多算一个候选根引用的槽 |
| X2 T1 的时机 | 抬 F 那次重算在回收之前、在带新 F 的空发布之前：崩在两次空发布之间（F 在一块盘上生效、另一块没有）再挂载，重建里用的是 F_生效（旧 F）、隔离集按旧候选集算，而回收已经发生过——回收的槽在新挂载的账里是什么状态；`raise_rollback_floor` 里 `oldest_valid_root` 按 `current` 的表判而重建按最新根的表判，两处会不会给出不同的隔离集 | 一个崩溃点让重开之后的隔离集与崩之前不同，且差的那个槽能被发出去 |
| X8 T3 扣住 | 扣住只住内存、只在抬 F 那个进程里：崩在两条带新 F 的根之间再挂载，重建用 F_生效（旧 F）回收 ⇒ 那些槽回到「已释放、未回收」；新实例的根写 F_生效；这时 checker 的 I-3.1（已分配统计对得上） 按最新根（带新 F、只落一块盘）自己的 F 取并集、而它的记账行已经把回收算成空闲——两边说的是同一件事吗；扣住之后一次空发布分配不到固定点（开放段满、没有别的全空段）会怎样（`allocate_commit_generated` 返回 `None` ⇒ 发布失败 ⇒ 抬 F 半途而废、F 只在一块盘上）；`release_reclaim_holds` 放开的时机是循环之后，循环因为 `ROOT_RING_REGIONS` 上限退出而没落满每块盘时也放开 | 一段历史让扣住的槽在生效之前被发出去，或让抬 F 在扣住之后发不出固定点 |
| X3 T2 跳过的根 | 跳过一条被抛弃根之后它引用的槽可能被复用：那条根还在根环里、根槽自证通过、只是树表坏了——恢复会不会选中它（`choose_root` / 候选集）、退到它会怎样；计数字段没有消费者是不是一格空条款；候选根的账读不出「当它什么都不豁免」的方向对不对 | 一段历史让被跳过的根被恢复或回退选中且它引用的槽已被盖 |
| X4 T1 与 D28 第九项 | 「被抛弃根独占量」按定义是两条根已分配之差，G5 的隔离集与它在固定脚本的每一步上是不是同一个数（回退那次 34、抬 F 之后 36）；准入不等式今天没接（C318（影子账隔离的单元没进准入不等式）），差在哪一格 | 一步上隔离数与第九项的定义算出的数不同 |
| X5 单层守卫 | `allocation_records_under_root` 对层 > 0 报 `UnitMalformed`：第一版有没有任何路径写出层 > 0 的分配记录树根（分配记录条数超过一个节点装得下的时候怎么办）；报错之后挂载是拒绝还是跳过（被抛弃根走 T2 的跳过、候选根走「什么都不豁免」） | 一段历史让分配记录树长到两层 |
| X6 代码与条款 | T1 / T2 的代码做的是不是第二节写的那件事；G5 与用户措辞的差别是不是只有「txg ≥ F」这一条；D23（journal 的角色与格式） 已定项 14 主句在 G5 下有没有一格不满足 | 一句改法描述与代码不符，或主句的一格在 G5 下为假 |
| X7 写回 | `02-second-txn.md` 步 4 / 步 5 现状、`invariants.md` I-3.1（已分配统计对得上） 那一行、`checks-owed.md` C340（回退之后记录链从哪条之后接没有定义） 第三列、第二轮判决第五节，与代码对不对得上 | 一句现状与代码不符 |

**反向接受条款**：X1–X3、X8 打中 ⇒ 改代码、补一条会红的用例与一条变异行、再攻一轮；打中的格落在没有条款的地方（T2 的读不出、X2 的崩在两次空发布之间）⇒ 记进 `02-second-txn.md` 对应步的决策点、标预想交用户，代码按最保守的读法改；X4 / X5 打中 ⇒ 记决策点或改代码（各写明修的是哪一格）；X6 / X7 打中 ⇒ 改现状或改代码描述。
**失败条款**：一条腿引的条款与附录原文不符 ⇒ 那一格作废；「没问题」一次不算（本地腿抽两次）；腿在副本上量出的数不进 kb。

## 四、腿的分工（四条腿，攻击面不重叠；这一轮云端那条是正推）

| 腿 | 立场 | 分到的格 | 不许碰的格 |
|---|---|---|---|
| Opus 攻方（`three-way-attack`） | 造可达的历史打穿 T1 / T2 / T3 | X1、X2、X3、X8 | X4、X5（本地攻方的） |
| 本地攻方（`three-way-local-attack`，英文，抽两次） | 同上，另一组攻击面 | X4、X5 | X1、X2、X3 |
| Sonnet 正推（`three-way-forward`） | 核 T1 / T2 / T3 的代码做的是不是条款说的；F_生效 回落那三个改法各修哪一格，按代码逐格核第二轮攻方腿的那张表 | X6、X7、方向 | 不替攻方找反例 |
| 本地辩方（`three-way-local-defense`，英文，抽两次） | 替 T1（G5 而不是用户措辞的原文、也不是保守读法）、T2（跳过而不是拒挂）与 T3（记账先动、槽扣住，而不是把回收整个推迟到生效之后）辩护：每处先写最强的反对再答 | — | 不替攻方找反例 |

## 五、禁读清单（各条腿）

这一轮别的腿的输出（`research/prompts/m2-step45-code-r3-*-output*.md`）与主 agent 的核实（`research/prompts/m2-step45-code-r3-main-verification.md`）。前几轮判决可读：`m2-step45-code-r2-main-verification.md`、`m2-step45-code-r1-main-verification.md`、`m2-step3-code-r1-main-verification.md`；前两轮四条腿的报告可读（攻方腿不许重复它们攻过的角度：第一轮的 X1–X4、X8，第二轮的 X1-A / X1-B、X2-A / X2-B、X3-A 到 X3-D 与本地攻方的十问）。本地腿的提示避开 Rust 路径 `::`、不写汉字变量名（F_生效 写成 effective_floor）。
