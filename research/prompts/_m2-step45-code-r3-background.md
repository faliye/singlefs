# 背景材料：m2-step45-code-r3（2026-09-17）

拼接顺序：正文 → 第二轮判决全文 → diff → 附录。

---

## 一、正文

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

---

## 二、第二轮判决全文（`research/prompts/m2-step45-code-r2-main-verification.md`）

# 里程碑「第二个事务」步 4 / 步 5 那批代码三方对抗第二轮：主 agent 核实（2026-09-17）

<!-- doc-lint:not-numbers S1 S2 S3 S4 S5 S6 X1 X2 X3 X4 X5 X6 X7 P1 P2 -->

材料：正文 `research/prompts/_m2-step45-code-r2-body.md`（S1–S6 五处改法与一处撤回、X1–X7 判据、四条腿的分工）、背景材料 `_m2-step45-code-r2-background.md`（正文 + 附录 + 第一轮判决 + diff）。
四条腿：云端攻方（Opus，`m2-step45-code-r2-opus-output.md`，模型 `m2-step45-code-r2-opus-model/`，X1 / X2 / X3）、云端辩方（Sonnet，`m2-step45-code-r2-sonnet-output.md`，复核第一轮判决第四节到第六节与写回 X7）、本地攻方（`m2-step45-code-r2-local-attack-output-s1/s2/s3.md` 三份干净样本，X4 / X5 / X6，七次调用里两次是提示自身的问题、两次真损坏作废）、本地辩方（`m2-step45-code-r2-local-defense-output-s3.md` 一份干净样本，五次调用四次带损坏——按「一条腿只抽一次样不算一次观测」，它的「站得住」一条都不采信，只当线索读）。

被判的文件（门禁 56 号按路径认）：`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs`、`crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs`、`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs`、`crates/mutations.tsv`。第一轮点过名、这一轮没再动的：`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/instance_table.rs`、`crates/singlefs-core/src/lib.rs`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker/src/image.rs`、`crates/singlefs-harness/src/crash.rs`。

## 一、本地攻方（X4、X5、X6；三份干净样本）

| 问 | s1 / s2 / s3 | 主 agent 核 |
|---|---|---|
| 1a（抬 F 里 `current` 的表与盘上最新根的表会不会不同） | 三份都没打中（s2 说「发布写盘写到一半时不同」，那不是两次调用之间可达的点） | 不中：`raise_rollback_floor` 与 `rebuilt_allocator` 之间没有别的发布，两张表恒同一份 |
| 1b（低于 F_生效 的最旧根会不会改 `reclaim_floor`） | 三份都没打中 | 不中，取 max 就是为了这一格 |
| 1c（`effective_rollback_floor` 不过滤被抛弃的根，F_生效 会不会被被抛弃时间线抬高） | 三份都打中：被抛弃时间线抬过的 F 留在它的根上，回退之后 F_生效 仍算它 | **不中，但记一条预想**：候选集与回收门槛用的是同一个 F_生效，退得到的根 txg ≥ F_生效，被回收的释放代 ≤ F_生效，被抛弃时间线抬的 F 只把它自己那次抬 F 已经排除掉的根继续排除，没有一个候选根的单元会被回收。「被抛弃根算不算 F 的载体」条款没写（D16（发布语义） 已定项 1 只说「各幸存盘所带 F」），今天的代码算，标预想交用户 |
| 1d（`readable_roots` 全是被抛弃根） | 三份都没打中 | 不中：最新根按它自己指着的表判不到被抛弃 |
| 2a（分配代 0、跨度 1 的记录第一版只有树表一条） | 三份都没打中 | 不中 |
| 2b（两盘树表记录一盘释放一盘没释放的撕裂态） | 三份按「会取盘 1 那份」作答 | 不中：一个单元两盘的记录写在同一版分配记录树的同一个节点里，随同一次发布原子出现，构造不出一盘释放一盘没释放 |
| 2c（`publish_version` 走 `previous = None` 且树表已释放） | 三份都没打中 | 不中，那条路只有 mkfs 之后第一个文件版本走 |
| 3a（所选根的表里有它自己实例的行） | 三份都没打中 | 不中，与第一轮 X4 的结构性证明一致 |
| 3b（候选集多加一条「最新根的实例号只能比目标大一」） | 三份都说会误拒 | 一致，不加 |
| 3c（「非空」除事务号之外还有没有别的字段可核） | 三份都没找到 | 一致，决策点原样 |

## 二、本地辩方（一份干净样本，五项；只当线索）

| 项 | s3 | 主 agent 核 |
|---|---|---|
| 1 P2 | 站得住，理由是「重放遇到别的实例就停」 | 理由错：重放先按同实例筛，被抛弃记录与 R_old 同实例时照样接上（云端攻方 X1-A 实测重放 3 条）；它给的 `journal.rs` 路径不存在。P2 的代价见第四节 |
| 2 影子账保守读法 | 站得住，「空闲计数与隔离位不同步是有意为之」 | 不采信：与用户 2026-09-16 定的窄读法字面相反（云端辩方第六节 2），改回窄读法（第五节 1） |
| 3 F_生效 做候选集 | 站得住，「退到单元被盖的根时 `rebuild_version` 读错就是正确的失败」 | 只答了走读失败那一半；云端攻方 X3-A 打中的是新根把 F 写回 0、checker 三条红，它没碰到 |
| 4 两处 `oldest_valid_root` 的表 | 站得住 | 与第一节 1a 同一格，不中 |
| 5 认第 0 版树表 | 站得住 | 一致；它的反证（mkfs 多写一条分配代 0 跨度 1 的记录）今天构造不出 |

## 三、云端辩方（Sonnet，复核第一轮判决）

| 格 | 判 | 主 agent 核 |
|---|---|---|
| 第一轮 X1 / X2 / X4 / X8 | 判对（X2 在副本上把变异改回 P1 复现红） | 一致 |
| 第一轮 X3 | 打中够得着；处置把「每次挂载都算」与「换成保守读法」捆在一起，后一半不是前一半的必然 | **采纳**：窄读法在「每次挂载重算」的框架下做得到（被抛弃根引用的槽减去候选根引用的并集），第五节 1 |
| 第一轮 S4 回收门槛 | 判对，但今天全部用例上两种写法逐字节相同、只有单测分得出 | 一致，第一轮判决第五节 4 已写 |
| 第六节 2 影子账读法 | **够不着**：不是决策点，是代码没照 D23（journal 的角色与格式） 已定项 14 已定的窄读法做 | 采纳一半：保守读法撤掉。实现取的豁免集是候选集（多了 txg ≥ F、抬 F 之后重算），是对用户措辞的收严——alloc-basis 那一轮的 G5 臂（singlefs-8e 会话核过：按原措辞，抬 F 到 11 之后 mkfs 实例表那 2 槽会发出去而被抛弃根 B 还引用着，违反主句），所以「影子账的读法」留在决策点表里（第六节 5） |
| 第六节 7 两个 F | 部分够不着：I-3.1（已分配统计对得上） 那一行「= 回退候选集里的根」与「txg ≥ 最新根带的 F」自相矛盾 | 采纳，`invariants.md` 那一行改成明说「checker 用最新根自己的 F、回退候选集用 F_生效、两者何时不同」 |
| 第六节其余六条 | 够得着，真无条款 | 一致 |
| X7 写回 | `02-second-txn.md` 步 4 现状仍写「记录核对器分辨合法覆盖」（逻辑已随 P2 撤回）；`mount.rs` `mount_rollback` 函数级文档仍写「取 P1」；步 4 用例文件的模块级文档仍写 jsn 4；`checks-owed.md` C340（回退之后记录链从哪条之后接没有定义） 第三列仍写「P1 为准、P2 必红」 | 四处都核实，都改（第五节 3） |

## 四、云端攻方（Opus，X1、X2、X3）

| 格 | 报告 | 主 agent 核 | 处置 |
|---|---|---|---|
| X1-A | **打中**：P2 把被抛弃记录留在环里；同一个实例连着覆盖写三次（jsn 4–6）、回退到 A、翻掉 txg 4–8 五个根槽 ⇒ 恢复落回 A、同实例连号的 jsn 4–6 三条整段重放、读回被抛弃的第四版；同一段历史 P1 施加 0 条 | 核实机理：`replay_journal` 按同实例筛、锚在 R_old 自己那条之后连号；回退行在实例 2 的表里、前缀第五条读不到（第一轮 X4 的死条款）。是 C332（回退实例两个根都读不出时回退被撤销） 那一族，新的是「P1 / P2 在这一格判得不一样」这一维与那个数 | 与第一轮 X2 互斥：P1 在这一格不中而在第一轮 X2 那一格中。代码不动，C340（回退之后记录链从哪条之后接没有定义） 第三列改成两格各中、交用户（第六节 1） |
| X1-B | **打中**：D23（journal 的角色与格式） 已定项 14 第 3 条「从前缀末 + 1 接着写」字面是 P1；C340 第三列写着 P1 为准 | 核实，第三列是第一轮改法落地时漏改的写回 | 已改（第五节 3）；条款字面交用户 |
| X2-A | **打中**：被抛弃根的树表单元两份都坏 ⇒ `rebuilt_allocator` 的 `?` 让每一次可写挂载都开不了，只读恢复照常 | 核实：副本上把 C 的树表两盘各翻一个字节，`mount_writable` 报 `UnitUnreadable` | **改法**：读不出账的被抛弃根跳过并计数（`MountOutput::abandoned_roots_unreadable`），不拒绝挂载；那条根引用的槽罩不到——「读不出账的被抛弃根怎么办」没有条款，标预想（第六节 4） |
| X2-B | **打中**：保守读法把 R_old 也引用的槽（mkfs 实例表 50176–50177）隔离，它们被释放、回收之后记账算空闲、位图发不出，而 D28（挂载期承诺量） 已定项 1 第九项按定义扣不到它们 | 核实机理。随窄读法消失：F = 0 时 A 是候选、那两槽豁免、不隔离（隔离 34 = 独占 34）；抬到 11 之后 A 不再是候选、那两槽只剩被抛弃的 B 引用，回收之前按新 F 重算把它们补隔离——这时它们按第九项的定义就在「被抛弃根独占量」里，记账与隔离位说同一件事 | **改法**：G5（豁免只给候选集里的根）、每次挂载与每次抬 F 现算（第五节 1） |
| X3-A | **打中**：抬 F 到 11、回收、E 合法复用之后翻掉盘 1 唯一带 F 的根 ⇒ F_生效 回落 0 ⇒ 下一次可写挂载的新根把 F 写回 0 ⇒ checker 候选集重新含进第 0 代根与 A，它们的单元已被合法盖掉 ⇒ I-3.1（已分配统计对得上）、I-2.1（校验和与内容匹配）、I-5.1（物理范围不重叠） 一起红 | 核实机理：`InstanceStart::effective_floor` 写进新根；「F 只许涨」这条今天没有任何地方守。三个改法各修哪一格攻方腿已列（新根写 max(上一条根的 F, F_生效) 修 A / B 不修 C / D；候选集也用 max 修 C / D 但第一轮正推腿判的「窄化」回来；只改 checker 修一半） | 代码不动：三个改法没有一个把四格都修掉，而且每个都改一条用户定过的条款（D16（发布语义） 已定项 1 生效与回退候选集两行）。交用户，带四格各中不中（第六节 2） |
| X3-B | **打中**：盘 1 上 6 条根全坏时 F_生效 仍是 11（那块盘不算幸存盘），一个故障比六个更坏 | 核实，与 X3-A 同一条款 | 同上 |
| X3-C / X3-D | 打中的下游：F 回落之后退到那些根，走读 `UnitUnreadable` | 核实，判据字面的「走读失败」那一半 | 同上 |
| 没打中的形状 | X1 所选根是 (1, 4)、被抛弃实例 2 的记录、环绕圈；X2 `newest_table = None`、全是已释放的槽；X3 根落在别处 | 逐条读过，判法与我一致 | 无 |

## 五、处置（都已落地：`check.sh` 绿、命名纪律绿、门禁 59 号全表复跑各红、doc-lint 绿）

1. **影子账改成 G5（豁免只给候选集里的根，对用户窄读法措辞的收严）、每次挂载与每次抬 F 现算**（`crates/singlefs-core/src/mount.rs` `isolate_slots_referenced_only_by_abandoned_roots`：被抛弃根账里还分配着的槽，减去候选集（可读 ∧ 按实例表有效 ∧ txg ≥ F）里每条根账里还分配着的、再减去当前这一版账里还分配着的；`rebuilt_allocator` 每次挂载调它，`raise_rollback_floor` 在回收之前按新 F 再调一次，新增入参 `ShadowLedger`）。用例：步 4 三条用例的隔离数从 36 改回 34（`rolling_back_to_the_first_root_…`、`…a_plain_remount_keeps_the_isolation`、`without_the_shadow_ledger_…`）；步 5 `raising_the_floor_to_the_first_release_generation_…` E 仍落 50178（抬 F 之后补隔离 50176–50177、回收的 50178 只被第 0 代根引用）；复用窗口置 0 那条用例 E 改落 50176（F = 0 时 A 是候选、50176 不隔离、回收之后被拿走，I-2.1（校验和与内容匹配） 仍红）。变异（`crates/mutations.tsv`）：「影子账不豁免候选根引用的槽（保守读法）」「抬 F 之后不按新候选集重算影子账」「影子账的豁免不看候选根的 txg 是否 ≥ F」各红；「影子账把被抛弃根账里已释放的落点也隔离」重新锚定。
2. **读不出账的被抛弃根跳过并计数**（`rebuilt_allocator` 不再 `?`，`MountOutput::abandoned_roots_unreadable`）。用例 `torn_tree_table_of_an_abandoned_root_is_counted_and_does_not_fail_the_mount`（C 的树表两盘各翻一个字节：挂载成功、计 1、隔离 24 = 34 − 只被 C 引用的 10）；变异「被抛弃根的树表读不出时不计数」它红。
3. **写回四处**：`mount_rollback` 函数级文档改成 P2；步 4 用例文件模块级文档 jsn 9；`02-second-txn.md` 步 4 现状删掉「记录核对器分辨合法覆盖」、影子账一句改成窄读法与读不出计数、隔离 34、新用例、变异清单；`checks-owed.md` C340（回退之后记录链从哪条之后接没有定义） 第三列改成两格各中；`invariants.md` I-3.1（已分配统计对得上） 那一行改成明说两个 F。
4. **`allocation_records_under_root` 只读根节点**（`crates/singlefs-core/src/recovery.rs`）：第一版分配记录树只有一层，读到层 > 0 的根报 `UnitMalformed`，不把内部节点的指针当分配记录解（云端辩方腿的复核顺带指出的单节点假设）。断言性质，没有用例。

## 六、交用户的决策点（都标预想；与第一轮判决第六节合并）

| # | 决策点 | 今天的代码（预想） | 每条路各中哪一格 |
|---|---|---|---|
| 1 | C340（回退之后记录链从哪条之后接没有定义）：P1 还是 P2 | P2 | P1：崩在回退生效之前，B 已提交的记录被盖（第一轮 X2，零故障、两个崩溃点）；P2：(txg, 实例) 大于 R_old 的每条可读根都坏掉时恢复落回 R_old、同实例连号的被抛弃记录整段重放（固定脚本上 5 个故障重放 3 条）。两格都不中的路是回退行另有一处不依赖新实例根的落点（格式级） |
| 2 | D16（发布语义） 已定项 1：抬 F 生效之后 F_生效 会不会回落 | 各幸存盘所带 F 最大值的最小值，新实例的根写它 | 一个字节翻掉一块盘唯一的载体根 ⇒ F 回落到 0、三条不变量一起红。改法 (a) 新根写 max(上一条根的 F, F_生效)：修 X3-A / B，不修 C / D；(b) 候选集也用 max：修 C / D，「窄化」回来；(c) 只改 checker：修一半。要「F 只许涨」就得在盘上或条款里定一个不随载体根丢失的下界 |
| 3 | 被抛弃根算不算 F 的载体（本地攻方 1c） | 算 | 与 2 同一族：被抛弃时间线抬的 F 留在它的根上，回退之后仍参与 F_生效 |
| 4 | 读不出账的被抛弃根怎么办 | 跳过、计数、不拒挂；它引用的槽罩不到 | 拒挂 ⇒ 一条被抛弃根的两个字节让池永远开不了；跳过 ⇒ 那条根引用的槽可能被复用，退到它时走读失败 |
| 5 | 影子账的读法 | G5：豁免只给候选集（可读 ∧ 按实例表有效 ∧ txg ≥ F）里的根、抬 F 之后按新 F 重算 | 用户 2026-09-16 的措辞「仍被有效根引用的槽不在其内」有效只按实例表判：按它抬 F 到 11 之后 mkfs 实例表那 2 槽被回收、发出去而 B 还引用着，违反 D23（journal 的角色与格式） 已定项 14 主句；保守读法（第一轮）隔离 R_old 也引用的槽，记账扣不到它们（X2-B）；G5 两格都不中，但它是对定案措辞的收严，alloc-basis 那一轮把三条臂一起交用户 |
| 6–11 | 第一轮判决第六节的 1、3、4、5、6、8（回退行怎么活、回退实例的根都读不出、最新根的实例表两份都坏、非空持久有效根怎么认、`NoPublishedVersion` 一码两义、树表空拒开） | 不变 | 不变 |

## 七、核查员（`m2-step45-code-r2-verifier-output.md`，279 行）

判别力自证过（把 D23（journal 的角色与格式） 那条引文的行号故意加 1，方法判 ✗）。云端攻方：核了 27 处引用，✓ 26、✗ 1（「一块盘上一条根都没有就不算它」那句注在 `recovery.rs` 348–349 行的函数头，报告标成 357–370 的函数体——位置错、内容对）；三组复跑命令（八条用例、P1 对照补丁、还原后步 4 / 步 5 各五条）都与报告逐字相符，P1 对照下那条用例按 P2 写的断言失败是预期。云端辩方：核了 28 处，✓ 26、✗ 2（`InstanceTableMalformed` 命中数报 2 处实为 5 处、行号 670 实为 671；`NoPublishedVersion` 报的第 944 行不存在）——两处都是计数与行号的错，不改变它判的任何一格。本地攻方的六条转述条目全 ✓；本地辩方十二条里九条 ✓、三条「部分」（把 D16（发布语义） 已定项 1 的节标题行当具体行号引，内容在 375–376 行）。合计核了 90 处：✓ 84、✗ 3、部分 3、核不动 0。第四节到第六节的判决按核查表逐条回看：没有一格建在 ✗ 的那三处上。

## 八、这一轮改法被攻过零轮

第五节 1、2 两处改法（窄读法现算、读不出计数）与第一轮的 S1、S3、S4、S5 一样是新形态；按 `.claude/rules/three-way-inference.md`「攻方腿自己提的收严，只在它自己的模型上量过，算没被攻过」，要第三轮再攻一次，攻击面只放这两处与第六节 2 那个 F 回落的改法方向。

---

## 三、Diff

# Diff：m2-step45-code-r3（2026-09-17）

第二轮判决（`research/prompts/m2-step45-code-r2-main-verification.md`）与 alloc-basis 第二轮云端攻方腿（`research/prompts/alloc-basis-r2-opus-output.md` 2.1 节、第七节第 1 条）打中之后改的代码，整段抄自工作区（未提交，与第二轮之间没有提交点，所以不是 `git diff`）。每段前写文件名与行区间，用 `awk 'NR==A,NR==B'` 现取，行号现查（2026-09-17）。

### `crates/singlefs-core/src/mount.rs` 第 29-52 行——MountError 枚举

```rust
pub enum MountError {
    Recovery(RecoveryFailure),
    /// 所选根下面还没有发布过文件版本（树表为空）：第一版的可写挂载只接在有文件的池后面，刚 mkfs 的池走第一次可写挂载那条路。
    NoPublishedVersion,
    /// 所选根指着的实例表单元解不出行与链指针。
    InstanceTableMalformed,
    /// 抬 F 要读当前版本的实例表判候选集，而这个进程还没做过可写挂载（实例表单元没重写过）：只 mkfs 加第一个事务的进程不能抬 F
    /// （alloc-basis 第二轮云端攻方腿第七节第 1 条：`TransactionOutput::unit` 在那里会 panic）。
    RaiseNeedsWritableMountInThisProcess,
    Acquisition(AcquisitionFailed),
    Publish(PublishError),
    /// 回退的目标根不在根环里（没有那个 (实例, txg) 的可读根槽）。
    RollbackTargetNotInRing(RollbackTarget),
    /// 回退的目标根在根环里，却不在回退候选集里：被抛弃时间线的根，或 txg 低于回退下界 F。
    RollbackTargetNotACandidate {
        target: RollbackTarget,
        reason: &'static str,
    },
    /// 要抬的 F 超过上限 min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)（D16（发布语义） 已定项 1）。
    RollbackFloorAboveCeiling {
        requested: CheckpointTxg,
        ceiling: CheckpointTxg,
    },
}
```

### `crates/singlefs-core/src/mount.rs` 第 81-102 行——MountOutput 结构体

```rust
/// 一次可写挂载写出的东西。
#[derive(Debug)]
pub struct MountOutput {
    pub instance: InstanceGeneration,
    /// 恢复择到的根（施加前缀之前）。
    pub chosen_root: RootRecord,
    /// 施加前缀之后的根：写行与照抄都以它为准。
    pub effective_root: RootRecord,
    pub journal: JournalScanReport,
    /// 这次挂载写进实例表的行（上一个实例那一行，中间实例各一行）。
    pub rows_written: Vec<InstanceRow>,
    /// 写行那次发布（本实例的第一次发布）。
    pub row_publish: TransactionOutput,
    /// 之后的暖机空发布，直到本实例的根覆盖每块盘。
    pub warm_up_publishes: Vec<TransactionOutput>,
    /// 影子账隔离的槽数，逐盘（D28（挂载期承诺量） 已定项 1 第九项「被抛弃根独占量」，只住内存）：只被被抛弃根引用的槽，
    /// 仍被候选集里的根引用的不在其内（对用户 2026-09-16 定的窄读法措辞的收严，见 `isolate_slots_referenced_only_by_abandoned_roots`；预想）。
    pub isolated_slots_per_device: Vec<(DeviceIdentity, u64)>,
    /// 被抛弃根里树表或分配记录树读不出、解不开的条数：这样的根影子账罩不到，只计数、不拒绝挂载
    /// （步 4 / 步 5 代码三方第二轮云端攻方腿打中：一条被抛弃根的树表撕裂不能让每次挂载都失败）。
    pub abandoned_roots_unreadable: u64,
}
```

### `crates/singlefs-core/src/mount.rs` 第 195-199 行——RebuiltAllocator 结构体

```rust
struct RebuiltAllocator {
    allocator: PoolAllocator,
    effective_floor: CheckpointTxg,
    abandoned_roots_unreadable: u64,
}
```

### `crates/singlefs-core/src/mount.rs` 第 201-258 行——isolate_slots_referenced_only_by_abandoned_roots 整个函数（含文档注释与 allow）

```rust
/// 影子账（D28（挂载期承诺量） 已定项 1 第九项）：只被被抛弃根引用的槽 = 被抛弃根引用的槽 − 候选集里的根引用的槽 − 当前这一版账里
/// 还分配着的槽；候选 = 可读 ∧ 按实例表有效 ∧ txg ≥ F。用户 2026-09-16 定的窄读法措辞是「仍被有效根引用的槽不在其内」、有效只按实例表判，
/// 豁免只给候选集里的根（多了 txg ≥ F、抬 F 之后按新 F 重算）是对那句措辞的收严（alloc-basis 那一轮的 G5 臂）：按原措辞，抬 F 到 11 之后
/// A 仍豁免 mkfs 实例表那 2 槽，它们被回收、发出去，而被抛弃根 B 还引用着，违反 D23（journal 的角色与格式） 已定项 14 的主句。
/// 预想、偏离用户定案的措辞，交用户。
/// 一条根引用的 = 它那一版账里还分配着的落点；账里已释放的是它换下的上一版的，不算它引用。
/// 树表或分配记录树读不出、解不开的被抛弃根只计数，不拒绝挂载；候选根读不出就当它什么都不豁免（隔离只会多不会少）。
/// 隔离位独立于分配位（`DeviceFreeMap::isolate`），所以候选集缩小（抬 F）之后重算只会多隔离几个槽，不用撤销。
#[allow(
    clippy::ptr_arg,
    reason = "allocation_records_under_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    roots: &[RootRecord],
    is_abandoned: &dyn Fn(&RootRecord) -> bool,
    floor: CheckpointTxg,
    current_records: &[AllocationRecord],
) -> u64 {
    let mut referenced_by_candidates: BTreeSet<(u32, u64)> = current_records
        .iter()
        .filter(|record| !record.is_released)
        .map(|record| (record.device.0, record.slot.0))
        .collect();
    for root in roots
        .iter()
        .filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)
    {
        if let Ok(records) = allocation_records_under_root(devices, root) {
            referenced_by_candidates.extend(
                records
                    .iter()
                    .filter(|record| !record.is_released)
                    .map(|record| (record.device.0, record.slot.0)),
            );
        }
    }
    let mut isolated: BTreeSet<(u32, u64)> = BTreeSet::new();
    let mut unreadable = 0;
    for root in roots.iter().filter(|root| is_abandoned(root)) {
        let Ok(records) = allocation_records_under_root(devices, root) else {
            unreadable += 1;
            continue;
        };
        for record in records {
            let key = (record.device.0, record.slot.0);
            if record.is_released
                || referenced_by_candidates.contains(&key)
                || !isolated.insert(key)
            {
                continue;
            }
            allocator.isolate_abandoned(record.device, record.slot, u64::from(record.span_slots));
        }
    }
    unreadable
}
```

### `crates/singlefs-core/src/mount.rs` 第 260-325 行——rebuilt_allocator 整个函数（含文档注释与 allow）

```rust
/// 重建分配器：从上一版的分配记录重建，按可再分配谓词的门槛回收，再把只被被抛弃根引用的槽隔离（影子账；
/// `extra_abandoned` 是这次挂载新抛弃的根——回退时是 (txg, 实例) 大于 R_old 的那些，普通挂载没有）。
/// 影子账只住内存，所以每次挂载都要重算，不只回退那一次（步 4 / 步 5 代码三方第一轮云端攻方腿打中：回退之后普通重开一次隔离就归零）。
#[allow(
    clippy::ptr_arg,
    reason = "choose_root 等走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn rebuilt_allocator<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    superblock: &crate::superblock::Superblock,
    previous: &TransactionOutput,
    extra_abandoned: &dyn Fn(&RootRecord) -> bool,
    shadow_ledger: ShadowLedger,
) -> RebuiltAllocator {
    let device_maps: Vec<DeviceFreeMap> = devices
        .iter()
        .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
        .collect();
    let mut allocator =
        PoolAllocator::rebuild_from_records(device_maps, previous.allocation_records.clone());
    let effective_floor = effective_rollback_floor(
        devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    let roots = readable_roots(
        devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    let newest_table = choose_root(devices, superblock)
        .and_then(|newest| instance_table_of_root(devices, &newest));
    let is_abandoned = |root: &RootRecord| {
        extra_abandoned(root)
            || newest_table
                .as_ref()
                .is_some_and(|table| abandoned_by_table(root, table))
    };
    let oldest_valid_root = roots
        .iter()
        .filter(|root| !is_abandoned(root))
        .map(|root| root.checkpoint_txg)
        .min();
    allocator.reclaim_released_up_to(
        reclaim_floor(effective_floor, oldest_valid_root),
        ReclaimedReuse::Immediately,
    );
    let abandoned_roots_unreadable = match shadow_ledger {
        ShadowLedger::On => isolate_slots_referenced_only_by_abandoned_roots(
            devices,
            &mut allocator,
            &roots,
            &is_abandoned,
            effective_floor,
            &previous.allocation_records,
        ),
        ShadowLedger::Off => 0,
    };
    RebuiltAllocator {
        allocator,
        effective_floor,
        abandoned_roots_unreadable,
    }
}
```

### `crates/singlefs-core/src/mount.rs` 第 392-507 行——raise_rollback_floor 整个函数（含文档注释）

```rust
/// 抬回退下界 F 到 `new_floor`（D16（发布语义） 已定项 1）：正常的触发是准入不够，这里是只供测试的强制入口（`.claude/rules/fs-design.md` 五条硬要求第 2 条）。
/// 推空发布直到每块盘上都有带新 F 的根（生效），然后把释放代 ≤ 新 F 的已释放落点回收。
///
/// # Errors
/// 这个进程没做过可写挂载；`new_floor` 超过上限；发布失败。
pub fn raise_rollback_floor<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    new_floor: CheckpointTxg,
    shadow_ledger: ShadowLedger,
) -> Result<RaisedFloor, MountError> {
    let superblock = choose_superblock(&*devices)?;
    let records = scan_journal(&*devices, &superblock);
    let instance_table_unit = current
        .units
        .iter()
        .find(|unit| unit.identity == TransactionUnit::InstanceTable)
        .ok_or(MountError::RaiseNeedsWritableMountInThisProcess)?;
    let table = InstanceTableRecords::parse(&instance_table_unit.bytes)
        .ok_or(MountError::InstanceTableMalformed)?;
    let ceiling = rollback_floor_ceiling(
        devices,
        &superblock,
        &records,
        current.root.rollback_floor,
        &table,
    )
    .ok_or(RecoveryFailure::NoValidRoot)?;
    if new_floor > ceiling {
        return Err(MountError::RollbackFloorAboveCeiling {
            requested: new_floor,
            ceiling,
        });
    }
    let all_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
    let device_of_txg = |txg: CheckpointTxg| {
        let target = target_for_publish(txg);
        parameters.region_devices[usize::try_from(target.region).expect("区域号")]
    };
    // 回收在写第一条带新 F 的根之前：一条根带的 F 与它的记账行要说同一件事——checker 的 I-3.1（已分配统计对得上） 按那条根自己的 F
    // 取候选集的并集，释放代 ≤ F 的落点不在并集里，记账的「已分配」也不能再算它们。但回收的槽在生效（每块盘都有带新 F 的根）之前
    // 不许发出去：带新 F 的空发布自己就在分配固定点，开放段满了会开到刚回收空的那一段（alloc-basis 第二轮云端攻方腿打中），所以先扣住、
    // 落满每块盘之后再放开。记账按写那条根的那一刻的 F 算还是按它持久之后的 F_生效 算，口径交 alloc-basis 那一轮（预想）。
    let oldest_valid_root = readable_roots(
        devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    )
    .into_iter()
    .filter(|root| !abandoned_by_table(root, &table))
    .map(|root| root.checkpoint_txg)
    .min();
    // 候选集按新 F 缩小，只被被抛弃根引用的槽会变多（一个槽此前靠一条低于新 F 的根豁免），回收之前先按新 F 重算影子账，
    // 不然那个槽回收之后就发得出去（步 4 / 步 5 代码三方第二轮辩方腿：窄读法要按每次挂载与每次抬 F 的候选集现算）。
    if shadow_ledger == ShadowLedger::On {
        let roots = readable_roots(
            devices,
            &superblock.region_devices,
            &superblock.geometry,
            &superblock.filesystem_identifier,
        );
        isolate_slots_referenced_only_by_abandoned_roots(
            devices,
            allocator,
            &roots,
            &|root| abandoned_by_table(root, &table),
            new_floor,
            &current.allocation_records,
        );
    }
    let reclaimed = allocator.reclaim_released_up_to(
        reclaim_floor(new_floor, oldest_valid_root),
        ReclaimedReuse::HeldUntilFloorTakesEffect,
    );
    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
    let mut covered: Vec<DeviceIdentity> = Vec::new();
    let mut publishes = Vec::new();
    while all_devices
        .iter()
        .any(|identity| !covered.contains(identity))
        && u64::try_from(publishes.len()).expect("次数") < ROOT_RING_REGIONS
    {
        let next = publish_version(
            &mut pool,
            allocator,
            PublishPlan {
                txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
                counter: current.record.counter + 1,
                transaction: 0,
                instance: current.root.instance,
                back_chain: back_chain_of(&current.record_bytes),
                file: None,
                instance_table: InstanceTablePlan::Carry(current.root.instance_table),
                tree_birth_txg: current.tree_birth_txg(),
                tree_identifier_watermark: current.root.tree_identifier_watermark,
                rollback_floor: new_floor,
            },
            Some(&*current),
        )?;
        let device = device_of_txg(next.root.checkpoint_txg);
        if !covered.contains(&device) {
            covered.push(device);
        }
        publishes.push(next.clone());
        *current = next;
    }
    allocator.release_reclaim_holds();
    Ok(RaisedFloor {
        ceiling,
        publishes,
        reclaimed,
    })
}
```

### `crates/singlefs-core/src/mount.rs` 第 790-811 行——mount_rollback 里从「影子账：」注释到 establish_instance( 调用之前

```rust
    // 影子账：这次回退新抛弃的根 = 根环里 (txg, 实例) 大于 R_old 的每一条可读根；只被它们引用的槽隔离，
    // 连同按实例表早已被抛弃的根一起在重建里算。
    let newly_abandoned = |root: &RootRecord| {
        (root.checkpoint_txg, root.instance) > (target.checkpoint_txg, target.instance)
    };
    let RebuiltAllocator {
        allocator,
        effective_floor: _,
        abandoned_roots_unreadable,
    } = rebuilt_allocator(
        devices,
        &superblock,
        &previous,
        &newly_abandoned,
        shadow_ledger,
    );
    let previous_row = PreviousInstanceRow {
        instance: target.instance,
        selected_root_txg: target.checkpoint_txg,
        applied_transaction_high_water: 0,
        is_rollback: true,
    };
```

### `crates/singlefs-core/src/allocator.rs` 第 100-127 行——DeviceFreeMap 结构体定义

```rust
pub struct DeviceFreeMap {
    pub device: DeviceIdentity,
    unit_area_slots: u64,
    /// 每个 16 KiB 槽一位：true = 已分配。
    allocated: Vec<bool>,
    /// 每个 64 槽聚簇段里已分配的槽数（增量维护全空段数用）。
    used_per_segment: Vec<u64>,
    /// 空闲槽的连续段数（增量维护）。
    free_runs: u64,
    /// 占着的槽数：仍分配的加上已释放、还在 defer 窗口里的——它们仍被根环里的有效根引用、仍占着空间
    /// （I-3.1（已分配统计对得上） 的读法 2026-09-14 用户定甲：按根环里全部有效根的引用取并集）。
    allocated_slots: u64,
    /// 空闲槽数，独立维护：分配时减、回收放回时加，不由「容量 − 已分配」现算（D5（快照 / 空间记账机制） 已定项 4 的 ⚠️：
    /// 那样 I-5.2（空闲统计对得上） 是恒真式；三方代码第一轮攻方腿打中）。已释放而还在 defer 窗口里的不算空闲。
    free_slots: u64,
    /// 影子账（D23（journal 的角色与格式） 已定项 14 回退段；D28（挂载期承诺量） 已定项 1 第九项「被抛弃根独占量」）：
    /// 只被被抛弃根引用的槽，当前账里既不是已分配也不是 defer，分配器却不许发出去，直到被抛弃的根离开根环。只住内存、不进记账行。
    isolated: Vec<bool>,
    isolated_per_segment: Vec<u64>,
    isolated_slots: u64,
    /// 其中已释放、还在 defer 窗口里的槽数（D5（快照 / 空间记账机制） 已定项 4 第 5 项）。
    deferred_slots: u64,
    /// 抬 F 回收、但 F 还没在每块盘上生效的槽：记账已经算它空闲，分配器却不许发出去，直到带新 F 的根落满每块盘
    /// （D16（发布语义） 已定项 1「每块幸存盘上都有带新 F 的持久根才生效」；alloc-basis 第二轮云端攻方腿打中：抬 F 自己的空发布开新段时
    /// 正好开到刚回收空的那一段，崩在两条带新 F 的根之间时 F 之下的根仍是候选、它们的单元已被盖）。只住内存。
    held_until_floor_takes_effect: Vec<bool>,
    held_per_segment: Vec<u64>,
}
```

### `crates/singlefs-core/src/allocator.rs` 第 130-154 行——DeviceFreeMap::new

```rust
    /// 单元区从 50176 起到设备末尾；mkfs 占的槽由调用方标上。
    #[must_use]
    pub fn new(device: DeviceIdentity, device_bytes: u64) -> Self {
        let unit_area_slots = device_bytes / SLOT_BYTES - UNIT_AREA_START_SLOT;
        let segments = unit_area_slots.div_ceil(CLUSTER_SEGMENT_SLOTS);
        Self {
            device,
            unit_area_slots,
            allocated: vec![false; usize::try_from(unit_area_slots).expect("单元区槽数")],
            used_per_segment: vec![0; usize::try_from(segments).expect("段数")],
            free_runs: u64::from(unit_area_slots > 0),
            allocated_slots: 0,
            free_slots: unit_area_slots,
            deferred_slots: 0,
            isolated: vec![false; usize::try_from(unit_area_slots).expect("单元区槽数")],
            isolated_per_segment: vec![0; usize::try_from(segments).expect("段数")],
            isolated_slots: 0,
            held_until_floor_takes_effect: vec![
                false;
                usize::try_from(unit_area_slots)
                    .expect("单元区槽数")
            ],
            held_per_segment: vec![0; usize::try_from(segments).expect("段数")],
        }
    }
```

### `crates/singlefs-core/src/allocator.rs` 第 160-167 行——DeviceFreeMap::is_free

```rust
    #[must_use]
    pub fn is_free(&self, slot: SlotNumber) -> bool {
        slot.0 >= UNIT_AREA_START_SLOT
            && slot.0 < UNIT_AREA_START_SLOT + self.unit_area_slots
            && !self.allocated[Self::index(slot)]
            && !self.isolated[Self::index(slot)]
            && !self.held_until_floor_takes_effect[Self::index(slot)]
    }
```

### `crates/singlefs-core/src/allocator.rs` 第 197-212 行——DeviceFreeMap::isolate

```rust
    /// 隔离一个只被被抛弃根引用的落点：不进已分配、不进 defer、不动空闲计数，只让分配器绕开它（用户数据落点与开放段都不落在它上面）。
    /// 隔离位独立于分配位：一个槽在当前账里已分配、后来被释放、再被回收，隔离位照样让它发不出去——抬 F 之后候选集缩小，
    /// 一个此前靠候选根豁免的槽会在回收之前被补隔离（`mount::raise_rollback_floor`）。同一个槽隔离两次不重复计数。
    pub fn isolate(&mut self, slot: SlotNumber, span: u64) {
        let start = Self::index(slot);
        let end = start + usize::try_from(span).expect("跨度");
        assert!(end <= self.allocated.len(), "跨度越过单元区末尾");
        for index in start..end {
            if !self.isolated[index] {
                self.isolated[index] = true;
                let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
                self.isolated_per_segment[segment] += 1;
                self.isolated_slots += 1;
            }
        }
    }
```

### `crates/singlefs-core/src/allocator.rs` 第 220-232 行——DeviceFreeMap::hold_until_floor_takes_effect

```rust
    /// 抬 F 回收的落点先扣住：空闲计数照加，位图照清，但 `is_free` 与开段都绕开它，直到 `release_holds`。
    pub fn hold_until_floor_takes_effect(&mut self, slot: SlotNumber, span: u64) {
        let start = Self::index(slot);
        let end = start + usize::try_from(span).expect("跨度");
        assert!(end <= self.allocated.len(), "跨度越过单元区末尾");
        for index in start..end {
            if !self.held_until_floor_takes_effect[index] {
                self.held_until_floor_takes_effect[index] = true;
                let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
                self.held_per_segment[segment] += 1;
            }
        }
    }
```

### `crates/singlefs-core/src/allocator.rs` 第 234-238 行——DeviceFreeMap::release_holds

```rust
    /// F 在每块盘上生效之后把扣住的槽放开。
    pub fn release_holds(&mut self) {
        self.held_until_floor_takes_effect.fill(false);
        self.held_per_segment.fill(0);
    }
```

### `crates/singlefs-core/src/allocator.rs` 第 240-261 行——DeviceFreeMap::mark_reclaimed

```rust
    /// 回收：一个已释放、释放代 ≤ max(F_生效, 环里最旧有效根) 的落点回到空闲（D16（发布语义） 已定项 1 的可再分配谓词）：
    /// 位图清掉、占着的槽数与 defer 队列各减、空闲加——记账的空闲字节到这一刻才动（里程碑「第二个事务」步 5）。
    pub fn mark_reclaimed(&mut self, slot: SlotNumber, span: u64) {
        let start = Self::index(slot);
        let end = start + usize::try_from(span).expect("跨度");
        assert!(
            self.allocated[start..end].iter().all(|taken| *taken),
            "回收的跨度里有没分配的槽"
        );
        let left_free = start > 0 && !self.allocated[start - 1];
        let right_free = end < self.allocated.len() && !self.allocated[end];
        // runs 的增量：两边都空是把两段并成一段（−1），两边都占是新开一段（+1），一边空是接上去（不变）。
        self.free_runs = self.free_runs + 1 - u64::from(left_free) - u64::from(right_free);
        for index in start..end {
            self.allocated[index] = false;
            let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
            self.used_per_segment[segment] -= 1;
        }
        self.allocated_slots -= span;
        self.deferred_slots -= span;
        self.free_slots += span;
    }
```

### `crates/singlefs-core/src/allocator.rs` 第 327-344 行——DeviceFreeMap::lowest_empty_segment

```rust
    /// 单元区内最低的 64 槽对齐全空段的起点。
    #[must_use]
    pub fn lowest_empty_segment(&self) -> Option<SlotNumber> {
        let per_segment = usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
        let full_segments = self.allocated.len() / per_segment;
        (0..full_segments)
            .find(|segment| {
                self.used_per_segment[*segment] == 0
                    && self.isolated_per_segment[*segment] == 0
                    && self.held_per_segment[*segment] == 0
            })
            .map(|segment| {
                SlotNumber(
                    UNIT_AREA_START_SLOT
                        + u64::try_from(segment).expect("段号") * CLUSTER_SEGMENT_SLOTS,
                )
            })
    }
```

### `crates/singlefs-core/src/allocator.rs` 第 347-352 行——ReclaimedReuse 枚举

```rust
/// 回收的槽什么时候能再发出去：重建分配器时 F 已经生效，立刻；抬 F 时要等带新 F 的根落满每块盘。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReclaimedReuse {
    Immediately,
    HeldUntilFloorTakesEffect,
}
```

### `crates/singlefs-core/src/allocator.rs` 第 533-561 行——PoolAllocator::allocate_commit_generated

```rust
    /// 提交内生块：从开放段 bump；容器按 32768 对齐档，节点取游标处最低空槽。开放段满了就开下一个全空段。
    pub fn allocate_commit_generated(
        &mut self,
        footprint: UnitFootprint,
        generation: CheckpointTxg,
    ) -> Option<Placement> {
        if self.open_segment.is_none() {
            let start = self.devices[0].lowest_empty_segment()?;
            self.open_segment = Some(start);
            self.bump_cursor = start.0;
        }
        let open = self.open_segment.expect("上面刚开");
        let segment_end = open.0 + CLUSTER_SEGMENT_SLOTS;
        let mut start = self.bump_cursor;
        if footprint == UnitFootprint::TwoSlotsAligned && !start.is_multiple_of(2) {
            start += 1;
        }
        if start + footprint.slots() > segment_end {
            self.open_segment = None;
            return self.allocate_commit_generated(footprint, generation);
        }
        let placement = Placement {
            slot: SlotNumber(start),
            span: footprint.slots(),
        };
        self.bump_cursor = start + footprint.slots();
        self.record(placement, generation);
        Some(placement)
    }
```

### `crates/singlefs-core/src/allocator.rs` 第 563-604 行——PoolAllocator::reclaim_released_up_to

```rust
    /// 回收：释放代 ≤ `floor` 的已释放落点回到空闲，等着被再分配（D16（发布语义） 已定项 1：可再分配 ⟺ 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)；
    /// 第一版根环 24 槽、种子 txg 0，环里最旧有效根恒 0，`floor` 就是 F_生效）。回收过的不重复回收；返回这次回收的落点（两盘同槽，按盘 0 报）。
    /// `reuse` 说回收的槽什么时候能发：重建时 F 已经生效，立刻；抬 F 时要等带新 F 的根落满每块盘（`release_reclaim_holds`）。
    pub fn reclaim_released_up_to(
        &mut self,
        floor: CheckpointTxg,
        reuse: ReclaimedReuse,
    ) -> Vec<Placement> {
        let mut reclaimed_now = Vec::new();
        let candidates: Vec<AllocationRecord> = self
            .records
            .iter()
            .filter(|record| record.is_released && record.generation <= floor)
            .copied()
            .collect();
        for record in candidates {
            let key = (record.device, record.slot);
            if !self.reclaimed.insert(key) {
                continue;
            }
            let device_map = self
                .devices
                .iter_mut()
                .find(|device_map| device_map.device == record.device)
                .expect("分配记录的盘在池里");
            device_map.mark_reclaimed(record.slot, u64::from(record.span_slots));
            match reuse {
                ReclaimedReuse::Immediately => {}
                ReclaimedReuse::HeldUntilFloorTakesEffect => {
                    device_map
                        .hold_until_floor_takes_effect(record.slot, u64::from(record.span_slots));
                }
            }
            if record.device == self.devices[0].device {
                reclaimed_now.push(Placement {
                    slot: record.slot,
                    span: u64::from(record.span_slots),
                });
            }
        }
        reclaimed_now
    }
```

### `crates/singlefs-core/src/allocator.rs` 第 606-611 行——PoolAllocator::release_reclaim_holds

```rust
    /// 抬 F 生效（带新 F 的根落满每块盘）之后，把扣住的回收槽放开。
    pub fn release_reclaim_holds(&mut self) {
        for device_map in &mut self.devices {
            device_map.release_holds();
        }
    }
```

### `crates/singlefs-core/src/allocator.rs` 第 613-621 行——PoolAllocator::isolate_abandoned

```rust
    /// 回退的影子账：把一个只被被抛弃根引用的落点在它那块盘上隔离。
    pub fn isolate_abandoned(&mut self, device: DeviceIdentity, slot: SlotNumber, span: u64) {
        let device_map = self
            .devices
            .iter_mut()
            .find(|device_map| device_map.device == device)
            .expect("被抛弃根的分配记录的盘在池里：走读逐盘核过");
        device_map.isolate(slot, span);
    }
```

### `crates/singlefs-core/src/recovery.rs` 第 377-417 行——allocation_records_under_root

```rust
pub fn allocation_records_under_root(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> Result<Vec<AllocationRecord>, RecoveryFailure> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
    let tree_table =
        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "树表单元",
        })?;
    let mut allocation_pointer = None;
    for bytes in &tree_table.entries {
        let entry = TreeTableEntry::parse(bytes).ok_or(RecoveryFailure::UnitMalformed {
            what: "树表条目",
        })?;
        if entry.kind == TREE_KIND_ALLOCATION {
            allocation_pointer = Some(entry.root);
        }
    }
    let Some(allocation_pointer) = allocation_pointer else {
        return Ok(Vec::new());
    };
    let allocation_bytes =
        read_unit_via_locations(reader, &allocation_pointer.locations, node_bytes)?;
    let allocation_node =
        parse_index_node(&allocation_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "分配记录树根",
        })?;
    // 第一版的分配记录树只有一个节点（层 0），多层的树这条路还不会走（里程碑「第二个事务」步 6 的欠账）；
    // 读到层 > 0 的根就报格式错，不把内部节点的指针当分配记录解。
    if allocation_node.level != 0 {
        return Err(RecoveryFailure::UnitMalformed {
            what: "分配记录树根不止一层",
        });
    }
    Ok(allocation_node
        .entries
        .iter()
        .map(|bytes| AllocationRecord::parse(bytes))
        .collect())
}
```

### `crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs` 第 1-476 行——整个文件

```rust
//! 里程碑「第二个事务」步 4 的验收：发布 C 之后进程退出、重开走管理员回退到 A 的根 (1, 3)——不施加 A 之后的任何记录、
//! 取实例代号 3、在 A 那一版实例表上写回退行 (1, 3, 0) 与中间实例行 (2, 0, 0)、发布 D（txg 9，jsn 接在 A 那条记录之后 = 4）、
//! 暖机一次（txg 10 落盘 1）——冷启动择实例 3 的根读回第一次的内容；只被被抛弃根引用的槽由影子账隔离；池级 checker 全绿。

mod common;

use common::{build_pool, file_content, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{
    mount_rollback, mount_writable, InstanceRow, MountError, Mounted, RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{
    choose_root, choose_superblock, recover, replay_journal, scan_journal, JournalPolicy,
    RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolWriter, TransactionOutput, TransactionUnit,
};
use std::collections::BTreeSet;

const SECOND_FILE_BYTES: usize = 4100;
const THIRD_FILE_BYTES: usize = 2500;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn second_content() -> Vec<u8> {
    content_of(SECOND_FILE_BYTES, 3)
}

fn third_content() -> Vec<u8> {
    content_of(THIRD_FILE_BYTES, 11)
}

fn overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// 固定脚本到 C：A、B（实例 1）、重开取号 2、写行、暖机两次、C（实例 2）。
fn build_through_third_publish(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted.current;
    overwrite_in_process(&mut pool, &third_content(), InstanceGeneration(2));
    pool
}

fn first_root() -> RollbackTarget {
    RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
    }
}

/// 进程退出、重开走回退到 A 的根；回来的可写态装回 pool。
fn rollback_to_first_root(pool: &mut BuiltPool, shadow_ledger: ShadowLedger) -> Mounted {
    let mut devices = pool.reopen_recorded();
    let rolled_back =
        mount_rollback(&parameters(), &mut devices, first_root(), shadow_ledger).expect("回退");
    pool.devices = Some(devices);
    pool.allocator = rolled_back.allocator.clone();
    pool.output = rolled_back.current.clone();
    rolled_back
}

fn region_device(txg: u64) -> DeviceIdentity {
    let target = target_for_publish(CheckpointTxg(txg));
    parameters().region_devices[usize::try_from(target.region).expect("区域号")]
}

/// 一版账里这块盘上占着的每个槽（记录按跨度展开）。
fn slots_of(output: &TransactionOutput, device: DeviceIdentity) -> BTreeSet<u64> {
    output
        .allocation_records
        .iter()
        .filter(|record| record.device == device)
        .flat_map(|record| record.slot.0..record.slot.0 + u64::from(record.span_slots))
        .collect()
}

/// 验收第一条：回退行 (1, 3, 0)、中间实例行 (2, 0, 0)；D 的根 (3, 9)、jsn 9（接在环里最大的 jsn 8 之后，C340 取 P2）、事务号 0、反向链 0，
/// 重写实例表 + 四个固定点单元；暖机一次落到另一块盘；一条记录都不施加；冷启动读回第一次的内容；被抛弃根独占的槽逐盘 34 个、
/// D 与暖机一个都不落在上面；checker 全绿（I-3.1 的并集按实例表把被抛弃的根排除，I-3.8 看见回退行）。
#[test]
fn rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content(
) {
    let mut pool = build_through_third_publish("step-four-rollback");
    let third = pool.output.clone();
    let rolled_back = rollback_to_first_root(&mut pool, ShadowLedger::On);
    let output = &rolled_back.output;
    assert_eq!(output.instance, InstanceGeneration(3));
    assert_eq!(
        output.rows_written,
        vec![
            InstanceRow {
                instance: InstanceGeneration(1),
                selected_root_txg: CheckpointTxg(3),
                applied_transaction_high_water: 0,
                is_rollback: true,
            },
            InstanceRow {
                instance: InstanceGeneration(2),
                selected_root_txg: CheckpointTxg(0),
                applied_transaction_high_water: 0,
                is_rollback: false,
            },
        ],
        "回退行与中间实例行"
    );
    assert_eq!(output.journal.prefix_applied, 0, "A 之后的记录一条都不施加");
    assert_eq!(
        (
            output.effective_root.instance,
            output.effective_root.checkpoint_txg
        ),
        (InstanceGeneration(1), CheckpointTxg(3))
    );
    let rollback_publish = &output.row_publish;
    assert_eq!(
        (
            rollback_publish.root.instance,
            rollback_publish.root.checkpoint_txg,
            rollback_publish.record.counter,
            rollback_publish.record.transaction,
            rollback_publish.record.back_chain
        ),
        (InstanceGeneration(3), CheckpointTxg(9), 9, 0, 0),
        "D：txg = max(根环 8, 记录 8) + 1；jsn 接在环里最大的 8 之后（C340 取 P2，被抛弃的记录一条不盖）；本实例第一条反向链 0"
    );
    assert_eq!(
        rollback_publish.rewritten,
        vec![
            TransactionUnit::InstanceTable,
            TransactionUnit::AllocationTree,
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
        ]
    );
    assert_eq!(
        output.warm_up_publishes.len(),
        1,
        "D 落盘 0、txg 10 落盘 1，一次就够"
    );
    let warm_up = &output.warm_up_publishes[0];
    assert_eq!(warm_up.root.checkpoint_txg, CheckpointTxg(10));
    assert_ne!(region_device(9), region_device(10));
    assert_eq!(warm_up.record.counter, 10);

    // 影子账：被抛弃的根 B、(2, 5)、(2, 6)、(2, 7)、C 引用而 A 不引用的槽——B 10、写行 6、暖机 4 + 4、C 10 = 34 个槽，逐盘。
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let abandoned: BTreeSet<u64> = slots_of(&third, device)
            .difference(&slots_of(rollback_publish, device))
            .copied()
            .collect();
        assert_eq!(abandoned.len(), 34, "盘 {device:?} 上只被被抛弃根引用的槽");
        // 影子账按窄读法只隔离这 34 个：mkfs 实例表那 2 个槽 A（候选）与 B 都引用，不在其内。
        assert!(
            output.isolated_slots_per_device.contains(&(device, 34)),
            "隔离的槽数 = 只被被抛弃根引用的 34 个 {:?}",
            output.isolated_slots_per_device
        );
        for publish in std::iter::once(rollback_publish).chain(output.warm_up_publishes.iter()) {
            for placement in publish.placements() {
                for slot in placement.slot.0..placement.slot.0 + placement.span {
                    assert!(
                        !abandoned.contains(&slot),
                        "txg {} 的落点 {slot} 落在被抛弃根引用的槽上",
                        publish.root.checkpoint_txg.0
                    );
                }
            }
        }
    }

    let image = pool.memory_pool();
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(3), CheckpointTxg(10)),
            content: file_content()
        },
        "{:?}",
        report.journal
    );
    assert_eq!(
        report.journal.valid_records, 10,
        "jsn 1–8 原样在（P2 一条不盖）、9 是 D、10 是暖机"
    );
    assert_eq!(
        report.journal.prefix_applied, 0,
        "(3, 10) 之后 jsn 11 一条都没有"
    );
    let verdicts = check_pool_image(&image);
    for (invariant, verdict) in &verdicts {
        assert_eq!(
            *verdict,
            InvariantVerdict::Holds,
            "{invariant} 在回退之后的镜像上要成立"
        );
    }
    for must_hold in ["I-3.1", "I-3.8", "I-5.2", "I-7.2"] {
        assert!(
            verdicts.iter().any(|(name, _)| *name == must_hold),
            "{must_hold} 真被判过"
        );
    }
}

/// 回退候选集（D23 已定项 14）：实例表里有行 (i, Ti, Wi) 的实例，只有 T ≤ Ti 的根可选——B 的根 (1, 4) 与 C 的根 (2, 8) 都是被抛弃时间线的；
/// 根环里没有的 (1, 42) 另报。
#[test]
fn rolling_back_onto_an_abandoned_timeline_or_a_missing_root_is_refused() {
    let mut pool = build_through_third_publish("step-four-refused");
    rollback_to_first_root(&mut pool, ShadowLedger::On);
    for (target, expected) in [
        (
            RollbackTarget {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
            },
            "实例表里那个实例的行 T 更小：这是被抛弃时间线的根",
        ),
        (
            RollbackTarget {
                instance: InstanceGeneration(2),
                checkpoint_txg: CheckpointTxg(8),
            },
            "实例表里那个实例的行 T 更小：这是被抛弃时间线的根",
        ),
    ] {
        let mut devices = pool.reopen_recorded();
        let refused = mount_rollback(&parameters(), &mut devices, target, ShadowLedger::On);
        pool.devices = Some(devices);
        match refused {
            Err(MountError::RollbackTargetNotACandidate {
                target: reported,
                reason,
            }) => assert_eq!((reported, reason), (target, expected)),
            other => panic!(
                "{target:?} 该被拒：{:?}",
                other.map(|mounted| mounted.output.instance)
            ),
        }
    }
    let mut devices = pool.reopen_recorded();
    let missing = RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(42),
    };
    let refused = mount_rollback(&parameters(), &mut devices, missing, ShadowLedger::On);
    pool.devices = Some(devices);
    assert!(
        matches!(refused, Err(MountError::RollbackTargetNotInRing(reported)) if reported == missing),
        "根环里没有 (1, 42)"
    );
}

/// C314（回退可以复用被抛弃的根引用的单元） 那一格的必红，影子账开关强制进入：关掉影子账，回退之后再发两版文件，
/// 数据单元落回 B 与 C 的数据槽（50182、50184）；把实例 3 的四个根槽都改坏，恢复挂上 C 的根 (2, 8)，它的数据单元已被盖掉，读不出第三次的内容。
/// 影子账开着：两版数据落 50186、50188，同样改坏四个根槽之后恢复挂上 C 的根、第三次的内容原样读回。
#[test]
fn without_the_shadow_ledger_publishes_after_the_rollback_reuse_the_abandoned_data_slots_and_a_recovery_onto_the_abandoned_root_reads_a_torn_unit(
) {
    for (shadow_ledger, expected_slots, expect_third_content_readable) in [
        (ShadowLedger::Off, [50182, 50184], false),
        (ShadowLedger::On, [50186, 50188], true),
    ] {
        let mut pool = build_through_third_publish("step-four-shadow");
        let rolled_back = rollback_to_first_root(&mut pool, shadow_ledger);
        let expected_isolated = if shadow_ledger == ShadowLedger::On {
            34
        } else {
            0
        };
        assert!(
            rolled_back
                .output
                .isolated_slots_per_device
                .iter()
                .all(|(_, isolated)| *isolated == expected_isolated),
            "{shadow_ledger:?}：{:?}",
            rolled_back.output.isolated_slots_per_device
        );
        let fourth = overwrite_in_process(&mut pool, &content_of(3000, 5), InstanceGeneration(3));
        let fifth = overwrite_in_process(&mut pool, &content_of(3100, 9), InstanceGeneration(3));
        assert_eq!(
            [
                fourth.data_pointer.locations[0].slot.0,
                fifth.data_pointer.locations[0].slot.0
            ],
            expected_slots,
            "{shadow_ledger:?} 下回退之后两版的数据落点"
        );
        let mut image = pool.memory_pool();
        for txg in [9u64, 10, 11, 12] {
            let target = target_for_publish(CheckpointTxg(txg));
            image.flip_byte(region_device(txg), slot_offset(target, 4096), 100);
        }
        let report = recover(&image, JournalPolicy::Consult);
        if expect_third_content_readable {
            assert_eq!(
                report.outcome,
                RecoveryOutcome::FileRead {
                    root: (InstanceGeneration(2), CheckpointTxg(8)),
                    content: third_content()
                },
                "影子账开着：C 引用的单元一个没被盖，回到 C 读第三次的内容"
            );
        } else {
            assert_ne!(
                report.outcome,
                RecoveryOutcome::FileRead {
                    root: (InstanceGeneration(2), CheckpointTxg(8)),
                    content: third_content()
                },
                "影子账关着：C 的数据单元已被第五版盖掉，第三次的内容读不回来"
            );
        }
    }
}

/// 前缀第五条（D23 已定项 14）：所选根的实例有回退行时，该实例的记录只施加到回退行的 W 为止——直接喂 `replay_journal`：
/// 到 B 为止的镜像上选 A 的根 (1, 3)，不带回退行施加 B 那条（事务号 2）；W = 0 一条都不施加；W = 2 施加到 B；W = 1 停在 B 之前。
#[test]
fn the_rollback_row_caps_the_prefix_of_the_chosen_roots_instance_at_its_high_water() {
    let mut pool = build_pool("step-four-cap");
    overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    let image = pool.memory_pool();
    let superblock = choose_superblock(&image).expect("超级块");
    let newest = choose_root(&image, &superblock).expect("B 的根");
    assert_eq!(newest.checkpoint_txg, CheckpointTxg(4));
    let records = scan_journal(&image, &superblock);
    let roots = singlefs_core::recovery::readable_roots(
        &image,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    let first = roots
        .iter()
        .find(|root| root.checkpoint_txg == CheckpointTxg(3))
        .copied()
        .expect("A 的根在环里");
    for (high_water, expected_applied, expected_txg) in [
        (None, 1, 4u64),
        (Some(0), 0, 3),
        (Some(1), 0, 3),
        (Some(2), 1, 4),
    ] {
        let (report, effective) = replay_journal(
            &image,
            &first,
            superblock.geometry.journal_ring_bytes,
            &records,
            true,
            high_water,
        );
        assert_eq!(
            (report.prefix_applied, effective.checkpoint_txg.0),
            (expected_applied, expected_txg),
            "回退行 W = {high_water:?}"
        );
    }
}

/// C340 取 P2：回退之后新实例的第一条 jsn 接在环里最大的 jsn 之后，被抛弃发布的记录一条不盖——B 的记录已提交、根还没落盘时发起回退，
/// 崩在回退生效之前那次恢复才仍「与没发起回退时相同」（步 4 / 步 5 代码三方第一轮云端攻方腿打中 P1 盖掉 B 那条）。
/// 影子账只住内存，所以每次挂载都重算：回退之后普通重开一次，被抛弃根引用的槽照样隔离（同一轮攻方腿打中重开后隔离归零）。
#[test]
fn rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation() {
    let mut pool = build_through_third_publish("step-four-p2-remount");
    let rolled_back = rollback_to_first_root(&mut pool, ShadowLedger::On);
    assert_eq!(
        rolled_back.output.row_publish.record.counter, 9,
        "D 的 jsn 接在 C 的 8 之后"
    );
    let image = pool.memory_pool();
    let superblock = choose_superblock(&image).expect("超级块");
    let records = scan_journal(&image, &superblock);
    for (instance, counter) in [(1u32, 4u64), (2, 5), (2, 8), (3, 9), (3, 10)] {
        assert!(
            records.contains_key(&(InstanceGeneration(instance), counter)),
            "记录 ({instance}, jsn {counter}) 该原样在环里：{:?}",
            records.keys().collect::<Vec<_>>()
        );
    }
    let mut devices = pool.reopen_recorded();
    let remounted = mount_writable(&parameters(), &mut devices).expect("回退之后普通重开");
    pool.devices = Some(devices);
    pool.allocator = remounted.allocator.clone();
    pool.output = remounted.current.clone();
    assert_eq!(remounted.output.instance, InstanceGeneration(4));
    assert_eq!(
        remounted.output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 34), (DeviceIdentity(1), 34)],
        "按 D 那一版实例表判被抛弃的根（B、实例 2 的四条）引用的槽，普通重开照样隔离"
    );
    assert_eq!(remounted.output.abandoned_roots_unreadable, 0);
    // 被抛弃根引用的槽：B 的数据 50182–50183、C 的数据 50184–50185 与它们的节点（隔离），以及 mkfs 实例表 50176–50177
    // （A 也引用、不隔离，但 D 释放它之后还在 defer 里）——重开后的发布一个都不该落上去。
    let abandoned: BTreeSet<u64> = (50176..50178).chain(50182..50186).collect();
    let next = overwrite_in_process(&mut pool, &content_of(2100, 41), InstanceGeneration(4));
    for placement in next.placements() {
        for slot in placement.slot.0..placement.slot.0 + placement.span {
            assert!(
                !abandoned.contains(&slot),
                "重开后的发布落到了被抛弃根引用的槽 {slot}"
            );
        }
    }
}

/// 被抛弃根 C 的树表单元在两块盘上都改坏：影子账罩不到 C（读不出就没法知道它引用谁），挂载照样成功、只计数一条读不出的被抛弃根，
/// 别的被抛弃根引用的槽照旧隔离（步 4 / 步 5 代码三方第二轮云端攻方腿打中：一条被抛弃根的树表撕裂不能让每次挂载都失败）。
#[test]
fn torn_tree_table_of_an_abandoned_root_is_counted_and_does_not_fail_the_mount() {
    let mut pool = build_through_third_publish("step-four-torn-abandoned");
    let third = pool.output.clone();
    rollback_to_first_root(&mut pool, ShadowLedger::On);
    let mut devices = pool.reopen_recorded();
    for location in &third.root.tree_table.locations {
        let (_, recorded) = devices
            .iter_mut()
            .find(|(identity, _)| *identity == location.device)
            .expect("C 的树表所在的盘");
        let offset = location.slot.to_device_offset();
        let mut bytes = vec![0u8; usize::try_from(singlefs_format::NODE_BYTES).expect("16384")];
        recorded.read_at(offset, &mut bytes).expect("读 C 的树表");
        bytes[200] ^= 0xff;
        recorded
            .write_at(offset, &bytes, WriteDurability::Plain)
            .expect("改坏 C 的树表");
    }
    let remounted =
        mount_writable(&parameters(), &mut devices).expect("被抛弃根的树表撕裂不拒绝挂载");
    assert_eq!(remounted.output.abandoned_roots_unreadable, 1, "C 读不出");
    // C 独占的槽（它的数据 2 + 它的节点）罩不到；B、写行与暖机那些照旧隔离。
    assert_eq!(
        remounted.output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 24), (DeviceIdentity(1), 24)],
        "少了只被 C 引用的 10 个槽"
    );
}
```

### `crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs` 第 1-442 行——整个文件

```rust
//! 里程碑「第二个事务」步 5 的验收：回退之后再覆盖写四次（txg 11–14；第一次把 A 的八个单元释放、释放代 11），抬回退下界 F 到 11
//! （上限 = min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根) = min(13, 11)；两次空发布 txg 15、16 让两块盘各有一条带 F = 11 的根），
//! 释放代 ≤ 11 的落点回收、之后的仍在 defer 队列里；发布 E（txg 17）把数据单元落回 50178（mkfs 树表那 1 槽回收了、50179 从没分配过；mkfs 实例表那片 50176 也回收了但 B 的根还引用它、影子账隔离着；A 的数据单元 50180 排在后面）；冷启动读回 E；checker 全绿。
//! 必红：不抬 F 就回收（复用窗口置 0），第 0 代根还在候选集里（F = 0）、它们引用的 mkfs 树表单元被 E 盖掉，checker 在它们上判 I-2.1 红。

mod common;

use common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::{Placement, ReclaimedReuse};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{
    mount_rollback, mount_writable, raise_rollback_floor, MountError, RaisedFloor, RollbackTarget,
    ShadowLedger,
};
use singlefs_core::recovery::{
    choose_superblock, readable_roots, recover, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// 固定脚本到 D：A、B、重开取号 2、写行、暖机两次、C、重开回退到 (1, 3)、D、暖机一次。
fn build_through_rollback(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite_in_process(&mut pool, &content_of(4100, 3), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted.current;
    overwrite_in_process(&mut pool, &content_of(2500, 11), InstanceGeneration(2));
    let mut reopened = pool.reopen_recorded();
    let rolled_back = mount_rollback(
        &parameters(),
        &mut reopened,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
        ShadowLedger::On,
    )
    .expect("回退");
    pool.devices = Some(reopened);
    pool.allocator = rolled_back.allocator;
    pool.output = rolled_back.current;
    pool
}

/// 回退之后再覆盖写四次（txg 11–14）：第一次释放 A 的八个单元（释放代 11）。
fn four_overwrites_after_the_rollback(pool: &mut BuiltPool) -> Vec<TransactionOutput> {
    [17usize, 19, 23, 29]
        .iter()
        .map(|seed| {
            overwrite_in_process(pool, &content_of(3000 + seed, *seed), InstanceGeneration(3))
        })
        .collect()
}

fn raise_floor(pool: &mut BuiltPool, new_floor: CheckpointTxg) -> Result<RaisedFloor, MountError> {
    let mut current = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let raised = raise_rollback_floor(
        &parameters(),
        devices,
        &mut pool.allocator,
        &mut current,
        new_floor,
        ShadowLedger::On,
    );
    pool.output = current;
    raised
}

fn newest_root_floor(pool: &BuiltPool) -> CheckpointTxg {
    let image = pool.memory_pool();
    let superblock = choose_superblock(&image).expect("超级块");
    readable_roots(
        &image,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    )
    .into_iter()
    .max_by_key(|root| (root.checkpoint_txg, root.instance))
    .expect("根")
    .rollback_floor
}

/// 验收第一、二条：上限 11；两次空发布带 F = 11 落到两块盘；A 的八个落点（10 槽）回收、defer 队列从 40 槽减到 30；E 的数据单元落 50180、
/// 它的分配记录改写成代 17、未释放；后释放的（代 12–14）仍占着；冷启动读回 E；根记录 F = 11；checker 全绿（A 的根在 F 之下、不在候选集）。
#[test]
fn raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it(
) {
    let mut pool = build_through_rollback("step-five-reuse");
    let overwrites = four_overwrites_after_the_rollback(&mut pool);
    assert_eq!(overwrites[0].root.checkpoint_txg, CheckpointTxg(11));
    assert_eq!(overwrites[3].root.checkpoint_txg, CheckpointTxg(14));
    for device in &pool.allocator.devices {
        assert_eq!(
            device.deferred_slots(),
            51,
            "A 的账里 mkfs 树表 1 槽已释放；D 释放 A 的四个固定点单元与 mkfs 实例表（6 槽）、暖机释放 D 的四个（4 槽）、四次覆盖写各释放上一版的 10 个槽"
        );
    }
    let raised = raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F 到 11");
    assert_eq!(
        raised.ceiling,
        CheckpointTxg(11),
        "min(每块盘最新的有效根 14 / 13, 第 4 新的非空 11)"
    );
    assert_eq!(
        raised
            .publishes
            .iter()
            .map(|publish| (
                publish.root.checkpoint_txg.0,
                publish.root.rollback_floor.0,
                publish.record.transaction
            ))
            .collect::<Vec<_>>(),
        vec![(15, 11, 0), (16, 11, 0)],
        "两次带新 F 的空发布"
    );
    assert!(raised.reclaimed.contains(&Placement {
        slot: SlotNumber(50180),
        span: 2
    }));
    assert_eq!(
        raised.reclaimed.len(),
        18,
        "释放代 ≤ 11 的落点：A 放掉的 mkfs 树表（代 3）、D 放掉的 5 个（代 9）、暖机放掉的 4 个（代 10）、第一次覆盖写放掉 A 的 8 个（代 11）"
    );
    for device in &pool.allocator.devices {
        assert_eq!(
            device.deferred_slots(),
            38,
            "回收了 1 + 6 + 4 + 10 = 21 个槽，抬 F 的两次空发布又各放掉上一版的 4 个"
        );
        assert!(device.is_free(SlotNumber(50180)) && device.is_free(SlotNumber(50181)));
        for later in &overwrites[..3] {
            let slot = later.data_pointer.locations[0].slot;
            assert!(
                !device.is_free(slot),
                "释放代 > F 的数据单元 {slot:?} 仍占着"
            );
        }
    }
    let reuse = overwrite_in_process(&mut pool, &content_of(2000, 31), InstanceGeneration(3));
    assert_eq!(reuse.root.checkpoint_txg, CheckpointTxg(17));
    assert_eq!(
        reuse.data_pointer.locations[0].slot,
        SlotNumber(50178),
        "E 的数据单元落回最低的可再分配偶数槽对 50178–50179：mkfs 树表那 1 槽（A 换下、释放代 3）回收了、50179 从没分配过；mkfs 实例表那片 50176 虽被 D 放掉、也回收了，但 B 的根还引用它、被影子账隔离；A 的数据单元 50180 排在后面"
    );
    let reused_record = pool
        .allocator
        .record_for(DeviceIdentity(0), SlotNumber(50178))
        .expect("50178 的记录");
    assert_eq!(
        (
            reused_record.generation,
            reused_record.is_released,
            reused_record.span_slots
        ),
        (CheckpointTxg(17), false, 2),
        "复用时那条记录改写"
    );
    assert_eq!(
        pool.allocator
            .records()
            .iter()
            .filter(|record| record.device == DeviceIdentity(0) && record.slot == SlotNumber(50178))
            .count(),
        1,
        "同盘同槽只有一条记录"
    );
    for device in &pool.allocator.devices {
        assert_eq!(device.deferred_slots(), 48, "E 又释放了第四版的 10 个槽");
    }
    assert_eq!(newest_root_floor(&pool), CheckpointTxg(11));
    let image = pool.memory_pool();
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(3), CheckpointTxg(17)),
            content: content_of(2000, 31)
        },
        "{:?}",
        report.journal
    );
    let verdicts = check_pool_image(&image);
    for (invariant, verdict) in &verdicts {
        assert_eq!(
            *verdict,
            InvariantVerdict::Holds,
            "{invariant} 在 E 之后的镜像上要成立"
        );
    }
    for must_hold in ["I-2.1", "I-3.1", "I-5.2", "I-7.2"] {
        assert!(
            verdicts.iter().any(|(name, _)| *name == must_hold),
            "{must_hold} 真被判过"
        );
    }
}

/// 生效（D16 已定项 1）：每块幸存盘上都有带新 F 的持久根才生效，恢复后生效值 = 各盘所带 F 最大值的最小值——把 txg 16 的根槽（盘 1 上唯一带 F = 11 的根）改坏，
/// 重开之后 F_生效 回到 0：所选根是 txg 15、链上 txg 16 的记录照样施加（同实例），盘上写着已释放的 59 个槽一个都不回收（新实例写行再放 6、暖机两次各放 4 ⇒ 73）；根槽都好时回收 21 个槽（59 − 21 + 14 = 52）。
#[test]
fn one_device_carrying_the_floor_alone_does_not_take_effect_on_remount() {
    for (damage_second_carrier, expected_deferred, expected_chosen_txg) in
        [(false, 52, 16), (true, 73, 15)]
    {
        let mut pool = build_through_rollback("step-five-effective");
        four_overwrites_after_the_rollback(&mut pool);
        let raised = raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F");
        let second_carrier = raised.publishes[1].root.checkpoint_txg;
        assert_eq!(second_carrier, CheckpointTxg(16));
        let mut devices = pool.reopen_recorded();
        if damage_second_carrier {
            let target = target_for_publish(second_carrier);
            let device =
                parameters().region_devices[usize::try_from(target.region).expect("区域号")];
            let offset = slot_offset(target, 4096);
            let (_, recorded) = devices
                .iter_mut()
                .find(|(identity, _)| *identity == device)
                .expect("那块盘");
            let mut bytes = vec![0u8; 4096];
            recorded.read_at(offset, &mut bytes).expect("读根槽");
            bytes[100] ^= 0xff;
            recorded
                .write_at(offset, &bytes, WriteDurability::Plain)
                .expect("改坏根槽");
        }
        let mounted = mount_writable(&parameters(), &mut devices).expect("重开");
        pool.devices = Some(devices);
        assert_eq!(
            mounted.output.chosen_root.checkpoint_txg,
            CheckpointTxg(expected_chosen_txg)
        );
        for device in &mounted.allocator.devices {
            assert_eq!(
                device.deferred_slots(),
                expected_deferred,
                "改坏第二块盘的载体 = {damage_second_carrier}：F_生效 = 各盘 F 最大值的最小值；数里含写行放掉的 6 与暖机两次放掉的 8"
            );
        }
    }
}

/// 上限：第 4 新的非空持久有效根是 11（非空的有 14、13、12、11；D 与暖机是空发布不算；A 的根 3 是第 5 新）⇒ 抬到 12 被拒；
/// 抬到 11 之后上限仍是 11，再抬 12 仍被拒。
#[test]
fn raising_the_floor_above_the_fourth_newest_non_empty_root_is_refused() {
    let mut pool = build_through_rollback("step-five-ceiling");
    four_overwrites_after_the_rollback(&mut pool);
    for attempt in [0, 1] {
        let refused = raise_floor(&mut pool, CheckpointTxg(12));
        assert!(
            matches!(
                refused,
                Err(MountError::RollbackFloorAboveCeiling {
                    requested: CheckpointTxg(12),
                    ceiling: CheckpointTxg(11)
                })
            ),
            "第 {attempt} 次：上限 11"
        );
        if attempt == 0 {
            raise_floor(&mut pool, CheckpointTxg(11)).expect("抬到上限");
        }
    }
    assert_eq!(newest_root_floor(&pool), CheckpointTxg(11));
}

/// 必红（C22（刚释放的块立即重分配）、复用窗口置 0）：不抬 F、直接把释放代 ≤ 11 的落点回收，E 落回 A 的数据落点 50176——
/// F = 0 时 A（txg 3）还是候选，影子账按窄读法豁免它引用的槽、没隔离 50176；checker 走 A 时那片数据的校验和对不上 ⇒ I-2.1 红。
#[test]
fn reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red(
) {
    let mut pool = build_through_rollback("step-five-window-zero");
    four_overwrites_after_the_rollback(&mut pool);
    let reclaimed = pool
        .allocator
        .reclaim_released_up_to(CheckpointTxg(11), ReclaimedReuse::Immediately);
    assert_eq!(reclaimed.len(), 18);
    let reuse = overwrite_in_process(&mut pool, &content_of(2000, 31), InstanceGeneration(3));
    assert_eq!(
        reuse.data_pointer.locations[0].slot,
        SlotNumber(50176),
        "A 的数据落点被拿走：A（F = 0 时仍是候选）还引用它，窄读法没隔离它"
    );
    assert_eq!(newest_root_floor(&pool), CheckpointTxg(0), "F 没抬");
    let verdicts = check_pool_image(&pool.memory_pool());
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    assert!(
        violated.contains(&"I-2.1"),
        "A 的根还是候选，它的数据单元被盖了：{verdicts:?}"
    );
}

/// 回退候选集的 F 用 F_生效（各幸存盘所带 F 最大值的最小值），不是最新根自己带的 F：抬到 11 之后把盘 1 的载体（txg 16）改坏，
/// F_生效 回到 0，txg 9 的根 D 仍是候选、退得到；按最新根（txg 15，F = 11）自己的 F 判会把它拒掉。
#[test]
fn roots_below_a_floor_carried_by_only_one_device_remain_rollback_candidates() {
    let mut pool = build_through_rollback("step-five-candidate-floor");
    four_overwrites_after_the_rollback(&mut pool);
    let raised = raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F");
    let second_carrier = raised.publishes[1].root.checkpoint_txg;
    let mut devices = pool.reopen_recorded();
    let target = target_for_publish(second_carrier);
    let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    let offset = slot_offset(target, 4096);
    let (_, recorded) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("那块盘");
    let mut bytes = vec![0u8; 4096];
    recorded.read_at(offset, &mut bytes).expect("读根槽");
    bytes[100] ^= 0xff;
    recorded
        .write_at(offset, &bytes, WriteDurability::Plain)
        .expect("改坏根槽");
    let rolled_back = mount_rollback(
        &parameters(),
        &mut devices,
        RollbackTarget {
            instance: InstanceGeneration(3),
            checkpoint_txg: CheckpointTxg(9),
        },
        ShadowLedger::On,
    )
    .expect("F_生效 是 0，txg 9 的根仍在候选集里");
    pool.devices = Some(devices);
    assert_eq!(rolled_back.output.instance, InstanceGeneration(4));
    assert_eq!(
        rolled_back.output.row_publish.root.rollback_floor,
        CheckpointTxg(0),
        "新实例的根写 F_生效"
    );
}

/// 抬 F 生效之前回收的槽不许发出去（D16（发布语义） 已定项 1「每块幸存盘上都有带新 F 的持久根才生效」；alloc-basis 第二轮云端攻方腿打中：
/// 抬 F 自己的空发布在开放段满了之后按最低全空段开新段，刚回收空的那一段正好中选，崩在两条带新 F 的根之间时 F 之下的根仍是候选、
/// 它们的单元已被盖）。历史照那条腿的：A、B、重开取号 2、六次覆盖写（txg 8–13，开放段用满）、抬 F 到 8——两次空发布的落点一个都不在
/// 这次回收的槽上。
#[test]
fn slots_reclaimed_by_raising_the_floor_are_not_handed_out_before_the_floor_takes_effect() {
    let mut pool = build_pool("step-five-hold-until-effective");
    overwrite_in_process(&mut pool, &content_of(4100, 3), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted.current;
    for seed in [31usize, 37, 41, 43, 47, 53] {
        overwrite_in_process(
            &mut pool,
            &content_of(2000 + seed, seed),
            InstanceGeneration(2),
        );
    }
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(13));
    let raised = raise_floor(&mut pool, CheckpointTxg(8)).expect("抬到 8");
    assert!(
        raised.publishes.len() >= 2,
        "至少两次空发布才能让两块盘各有一条带新 F 的根：{}",
        raised.publishes.len()
    );
    assert!(!raised.reclaimed.is_empty(), "回收了 A 与前几版释放的落点");
    let reclaimed_slots: std::collections::BTreeSet<u64> = raised
        .reclaimed
        .iter()
        .flat_map(|placement| placement.slot.0..placement.slot.0 + placement.span)
        .collect();
    for publish in &raised.publishes {
        for placement in publish.placements() {
            for slot in placement.slot.0..placement.slot.0 + placement.span {
                assert!(
                    !reclaimed_slots.contains(&slot),
                    "txg {} 的固定点落在这次回收的槽 {slot} 上：F 还没在两块盘上生效",
                    publish.root.checkpoint_txg.0
                );
            }
        }
    }
    assert_eq!(newest_root_floor(&pool), CheckpointTxg(8));
}

/// 只 mkfs 加第一个事务、没做过可写挂载的进程里抬 F：当前版本没有重写过的实例表单元，报错而不是 panic
/// （alloc-basis 第二轮云端攻方腿第七节第 1 条）。
#[test]
fn raising_the_floor_in_a_process_that_never_mounted_writable_is_refused_instead_of_panicking() {
    let mut pool = build_pool("step-five-raise-without-mount");
    let refused = raise_floor(&mut pool, CheckpointTxg(1));
    assert!(
        matches!(
            refused,
            Err(MountError::RaiseNeedsWritableMountInThisProcess)
        ),
        "第一个事务之后直接抬 F 该报没做过可写挂载：{:?}",
        refused.as_ref().err()
    );
}
```

### `crates/mutations.tsv` 以「步 4」「步 5」开头的行（第 31-43、45-56 行，跳过第 44 行——那行属于步 3）

```tsv
# crates 的变异表：每行六段，制表符分隔——变异名 <TAB> 文件 <TAB> 原文 <TAB> 替换文 <TAB> cargo test 的参数 <TAB> 必须红的测试名。
步 4：回退行不带回退位	crates/singlefs-core/src/mount.rs	        applied_transaction_high_water: 0,\n        is_rollback: true,	        applied_transaction_high_water: 0,\n        is_rollback: false,	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
步 4：只写被退回的实例那一行、不写中间实例行	crates/singlefs-core/src/mount.rs	    for row_instance in first_row_instance..instance.0 {	    for row_instance in first_row_instance..first_row_instance + 1 {	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
步 4：回退之后 jsn 接 R_old 那条之后（C340 的 P1，盖掉被抛弃发布的记录槽）	crates/singlefs-core/src/mount.rs	    let next_counter = highest_counter + 1;	    let next_counter = own_record.counter + 1;	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_keeps_the_abandoned_records	rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation
步 4：影子账一个槽都不隔离	crates/singlefs-core/src/mount.rs	            allocator.isolate_abandoned(record.device, record.slot, u64::from(record.span_slots));	            let _ = (record.device, record.slot, record.span_slots);	-p singlefs-harness --test second_transaction_step_four_rollback -- without_the_shadow_ledger	without_the_shadow_ledger_publishes_after_the_rollback_reuse_the_abandoned_data_slots_and_a_recovery_onto_the_abandoned_root_reads_a_torn_unit
步 4：回退候选集不按实例表判（被抛弃时间线的根也能退到）	crates/singlefs-core/src/mount.rs	        .any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg)	        .any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg && false)	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_onto_an_abandoned	rolling_back_onto_an_abandoned_timeline_or_a_missing_root_is_refused
步 4：前缀第五条不判（回退行的 W 不封顶）	crates/singlefs-core/src/recovery.rs	            if high_water == 0 || record.transaction > high_water {	            if false && (high_water == 0 || record.transaction > high_water) {	-p singlefs-harness --test second_transaction_step_four_rollback -- the_rollback_row_caps	the_rollback_row_caps_the_prefix_of_the_chosen_roots_instance_at_its_high_water
步 4：checker 的 I-3.1 并集不按实例表排除被抛弃的根	crates/singlefs-checker/src/walk.rs	        if index != newest_index && !abandoned && !below_floor {	        if index != newest_index && !below_floor {	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
步 5：回收不看释放代（复用窗口置 0）	crates/singlefs-core/src/allocator.rs	            .filter(|record| record.is_released && record.generation <= floor)	            .filter(|record| record.is_released)	-p singlefs-harness --test second_transaction_step_three_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
步 5：抬 F 的上限不看第 4 新的非空根	crates/singlefs-core/src/mount.rs	    Some(newest_on_every_device.min(fourth_newest))	    Some(newest_on_every_device.max(fourth_newest))	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_above	raising_the_floor_above_the_fourth_newest_non_empty_root_is_refused
步 5：抬 F 的空发布只推一次（F 只落在一块盘上）	crates/singlefs-core/src/mount.rs	        && u64::try_from(publishes.len()).expect("次数") < ROOT_RING_REGIONS	        && u64::try_from(publishes.len()).expect("次数") < 1	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
步 5：复用时追加记录而不改写（同盘同槽两条）	crates/singlefs-core/src/allocator.rs	            if self.reclaimed.remove(&key) {	            if self.reclaimed.remove(&key) && false {	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
步 5：checker 的候选集不按 F 收（F 之下的根照走）	crates/singlefs-checker/src/walk.rs	        if index != newest_index && !abandoned && !below_floor {	        if index != newest_index && !abandoned {	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
步 5：F_生效 取各盘 F 最大值的最大值而不是最小值	crates/singlefs-core/src/recovery.rs	    highest_per_device\n        .values()\n        .copied()\n        .min()\n        .unwrap_or(CheckpointTxg(0))	    highest_per_device\n        .values()\n        .copied()\n        .max()\n        .unwrap_or(CheckpointTxg(0))	-p singlefs-harness --test second_transaction_step_five_reuse -- one_device_carrying_the_floor_alone	one_device_carrying_the_floor_alone_does_not_take_effect_on_remount
步 4：普通重开不隔离被抛弃根引用的槽（影子账只在回退那一次算）	crates/singlefs-core/src/mount.rs	        &|_| false,\n        ShadowLedger::On,\n    );	        &|_| false,\n        ShadowLedger::Off,\n    );	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_keeps_the_abandoned_records	rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation
步 4：回退候选集的 F 用最新根自己带的 F 而不是 F_生效	crates/singlefs-core/src/mount.rs	    if target.checkpoint_txg < effective_floor {	    if target.checkpoint_txg < newest_root.rollback_floor {	-p singlefs-harness --test second_transaction_step_five_reuse -- roots_below_a_floor_carried	roots_below_a_floor_carried_by_only_one_device_remain_rollback_candidates
步 5：回收门槛不看环里最旧有效根	crates/singlefs-core/src/mount.rs	    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))	    oldest_valid_root.map_or(effective_floor, |_oldest| effective_floor)	-p singlefs-core reclaim_floor_takes	reclaim_floor_takes_the_larger_of_the_effective_floor_and_the_oldest_valid_ring_root
步 4：影子账把被抛弃根账里已释放的落点也隔离	crates/singlefs-core/src/mount.rs	            if record.is_released\n                || referenced_by_candidates.contains(&key)\n                || !isolated.insert(key)	            if referenced_by_candidates.contains(&key) || !isolated.insert(key)	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
步 5：重建分配器时不认第 0 版树表单元（另一个进程里的第一个文件版本漏释放它）	crates/singlefs-core/src/allocator.rs	                record.generation == CheckpointTxg(0)\n                    && record.span_slots == 1\n                    && !record.is_released	                record.generation == CheckpointTxg(0)\n                    && record.span_slots == 1\n                    && !record.is_released\n                    && false	-p singlefs-core rebuild_from_records_remembers	rebuild_from_records_remembers_the_genesis_tree_table_until_it_is_released
步 4：影子账不豁免候选根引用的槽（保守读法）	crates/singlefs-core/src/mount.rs	            if record.is_released\n                || referenced_by_candidates.contains(&key)\n                || !isolated.insert(key)	            if record.is_released || !isolated.insert(key)	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
步 4：被抛弃根的树表读不出时不计数	crates/singlefs-core/src/mount.rs	        let Ok(records) = allocation_records_under_root(devices, root) else {\n            unreadable += 1;\n            continue;\n        };	        let Ok(records) = allocation_records_under_root(devices, root) else {\n            continue;\n        };	-p singlefs-harness --test second_transaction_step_four_rollback -- torn_tree_table_of_an_abandoned_root	torn_tree_table_of_an_abandoned_root_is_counted_and_does_not_fail_the_mount
步 5：抬 F 之后不按新候选集重算影子账	crates/singlefs-core/src/mount.rs	    if shadow_ledger == ShadowLedger::On {\n        let roots = readable_roots(	    if shadow_ledger == ShadowLedger::Off {\n        let roots = readable_roots(	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
步 5：影子账的豁免不看候选根的 txg 是否 ≥ F	crates/singlefs-core/src/mount.rs	        .filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)	        .filter(|root| !is_abandoned(root))	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
步 5：抬 F 回收的槽不扣住、生效之前就能发出去	crates/singlefs-core/src/mount.rs	        ReclaimedReuse::HeldUntilFloorTakesEffect,	        ReclaimedReuse::Immediately,	-p singlefs-harness --test second_transaction_step_five_reuse -- slots_reclaimed_by_raising	slots_reclaimed_by_raising_the_floor_are_not_handed_out_before_the_floor_takes_effect
步 5：抬 F 生效之后不放开扣住的槽	crates/singlefs-core/src/mount.rs	    allocator.release_reclaim_holds();\n    Ok(RaisedFloor {	    Ok(RaisedFloor {	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
步 5：没做过可写挂载的进程抬 F 时 panic 而不是报错	crates/singlefs-core/src/mount.rs	        .ok_or(MountError::RaiseNeedsWritableMountInThisProcess)?;	        .expect("实例表单元");	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_in_a_process_that_never_mounted	raising_the_floor_in_a_process_that_never_mounted_writable_is_refused_instead_of_panicking
```


---

## 四、附录

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:1206-1244`（整段抄，未转述）**

```markdown
### 已定项 14（2026-09-02，用户定案）：重放的下界由所选根给出
<!-- doc-lint:not-numbers CJ2 -->

**显式例外：管理员回退（2026-09-05，随 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3）。** 回退 = 一次恢复：管理员带外从回退候选集里选一个旧根 R_old——候选集 = 根环里按实例表判仍然有效、且 txg ≥ F_生效（2026-09-13，D16（发布语义） 已定项 1）的根，(i, T) 可选 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti（被抛弃时间线的根一条都选不中）；不施加 R_old 之后的任何记录；取新实例代号；在 R_old 指着的那一版实例表上写回退行 (r_old, T_old, 0) 与中间实例的 (i, 0, 0)（r_old 诞生代号 ≤ T_old 的单元全部已发布、之后的一个都不算，不需要事务号上限；表的底版：2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方）；第一个新根的 checkpoint_txg = max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1（2026-09-16 随 C143（inode 号水位在回退后会退回去重发） 定案取 CJ2：回退本来就全环扫描、逐条验证，额外读 0；被抛弃时间线里根槽写失败留下的「只有记录、没有根」的 txg 因此也被跳过；新实例的第一次发布同样从这个最大值 + 1 起，见注 3）；defer 队列、分配器游标、全部记账统计量的现行值从 R_old 那棵账重新载入。**被抛弃时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头——分配器多查环里每一个可读根的账、只隔离其中**只被被抛弃根引用**的槽——查账的集合 2026-09-16 用户定案取窄读法：按主语「被抛弃时间线的根」读，仍被有效根引用的槽不在其内；宽读法（凡被环里任一可读根引用过的槽都隔离）会让抬 F 在第一版 24 槽里买不到任何东西，里程碑「第二个事务」三方第一轮攻方腿指出两种读法（`research/prompts/m2-r1-main-verification.md` 第三节）（影子账；2026-09-13 用户定案，C314（回退可以复用被抛弃的根引用的单元） 取 F-A，E150（回退复用被抛弃的根引用的单元） 两格零违例、多隔离 2–3 个单元到被抛弃的最新根被轮转覆写为止；隔离量怎么进准入不等式见 C318（影子账隔离的单元没进准入不等式））。** 回退与它的第一个新根同一次发布：回退行、那次发布的单元与回退根在那次发布之前都不生效，崩了就重做——崩在它之前，下一次挂载选的根、施加的记录、已发布谓词的判定都与没发起回退时相同；留在盘上的是取号写进超级块的新号（取号先于那次发布持久，D23（journal 的角色与格式） 已定项 16）以及那次发布已落盘而不生效的单元与记录，新号让下一次取号跳过一个号，被跳过的号按 D18（块里携带什么信息） 已定项 11 的写行规则得一行（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）；回退深度由候选集定（txg ≥ F_生效，2026-09-13 D16（发布语义） 已定项 1）：平时是整个根环，盘紧时可缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内。E104（扫描重建的现行版本判定）：不写回退行时全规则臂复活 3，新根取 T_old + 1 时压不过被抛弃的根。

**恢复只施加 `(实例代号, checkpoint_txg)` 严格大于所选根的记录；
陈旧 tail 只是「从哪开始扫环」的优化，不再决定重放集合。**

**依据（E78（重放的起点），四条恢复算法各过一遍判据）**：陈旧 tail 叠上块重用之后，
「全环扫描 + 断号即止 + 幂等」三件套不闭合——从 tail 逐条验证重放的形态在**健康镜像上自我中止**
（把陈旧失配当损坏），失配跳过的形态撕裂判别力归零。两条可行出路里取**水位臂**；
**尾删臂（盲放全前缀 + 收尾走读 + 尾删重试）落选**，理由：D16（发布语义）已定项 6 定案之后
水位就是根记录已有的 `(实例代号, checkpoint_txg)`（D22（单元原子性怎么合成）已定项 7），
**零新增格式字段**，而尾删臂要在恢复路径里加一个重试环——同价买贵的没有理由。

**前缀判定的完整口径是六条，缺一不可**（引用 I-8.3（重放前缀严格连续）时连这句一起引；第六条 2026-09-13 随 D16（发布语义） 已定项 4 用户定案加）：
jsn 严格连续（断号即止）、**`(实例代号, checkpoint_txg)` 大于根的水位**、
提交标记齐全的事务才施加（D23（journal 的角色与格式）已定项 7）、
施加前逐项验证点名单元（D16（发布语义）已定项 7，E77（发布的持久顺序）证明承重）、
**所选根的实例在实例表里有回退行时，该实例的记录只施加到回退行的 W 为止**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2，2026-09-05：
否则一次落在 R_old 上的普通恢复会把被回退抛弃的那段时间线整段重放回来，管理员的回退被静默撤销）。
⚠️ **第六条（2026-09-13，随 D16（发布语义） 已定项 4 用户定案加）：施加的单位是一次发布——合法前缀停在一次发布的两个事务之间时，那次发布整体不施加，前五条判出来的前缀再按发布边界截短一次；引用这五条时连这一条一起引。**

⚠️ **它对 D23（journal 的角色与格式）已定项 3 是收窄不是推翻**：「先全环扫描、逐条验证、
不许先信 tail」原样成立——变的是验证出的合法前缀里**哪一段被施加**。
⚠️ **checker 侧欠账**：水位判定的会红检查（陈旧 tail + 已复用镜像上恢复必须完成且终态与真值逐格相等）
落在 [checks-owed.md](../checks-owed.md) C77（重放起点未定义），等崩溃点重放 harness。

**跨实例边界、链首、比较序、计数器与切换时的 W（2026-09-13，C199（实例代号递增与 jsn 断号即止互相矛盾） 两轮三方论证，主 agent 按三条腿的观测定，2026-09-14 用户复核确认）**：

1. **前缀规则不跨实例边界**：链从所选根覆盖的最后一条记录之后接，下一条的实例代号与所选根不同即停（六条口径第一条原样）。所选根是 mkfs 的第 0 代根时它一条记录都不覆盖，之后的记录全属于更新的实例 ⇒ 一条都不施加。代价写在 D16（发布语义） 已定项 7 的注，要不要让新实例先暖机是 D16（发布语义） 已定项 8。
2. **记录水位 `(实例代号, checkpoint_txg)` 按实例代号为主比**，与 jsn 同序；择根照旧 txg 为主、实例代号破平局（D22（单元原子性怎么合成） 已定项 7），两处比的东西不同，各守各的序。按 txg 为主时，回退新根取「根环最大 + 1」，而被抛弃实例开放 checkpoint 里已提交的记录 txg 可以更大，会被当成水位之后施加到回退根上（第二轮反推腿 R-TXG，零故障）。
3. **计数器全池接着走**：新实例从前缀末 + 1 接着写、不归零。定长环的槽位由 jsn 决定（槽位与序号解耦那条出路已被 E36（槽位映射那一维） 判掉），归零会让新实例的记录落到旧前缀还要用的槽上；已定项 9 的「48 位撑 3202 年」本来就按全卷寿命算。⚠️ **checkpoint_txg 也一样（2026-09-16，C143（inode 号水位在回退后会退回去重发） 定案 CJ2）**：新实例（普通挂载、切换、回退）的第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1；普通挂载与回退本来就全环扫描，切换不重扫（内存里的 txg 已不小于这次挂载见过的一切）。它与 C331（择根倒挂压过已确认的写） 的修法候选「记录扫描水位」是同一个量，C331（择根倒挂压过已确认的写） 还清时一起写。代价：跳号比今天大（最多到被抛弃时间线的长度），D5（快照 / 空间记账机制） 已定项 2 的点删随之改成「删掉 ≤ 当前代 − K 的全部代」。
4. **切换时的 W 取被重发的那个 checkpoint 里的最大事务号**：开放 checkpoint 里已完成的事务全部 > W、按新写序重做。按第 2 条，它们的记录不在所选根之后；W 若取字面的「最后施加的事务号」，它们的单元会被谓词判成已发布而谁都不施加它们（C287（切换收养开放 checkpoint 的事务后再崩））。被重发的 checkpoint 里没有事务号非 0 的记录（重发的是空发布）时 W = 0（已定项 19 ①：事务号 0 不进 W 的 max）；W 写在每个被照旧事务的写序实例那一行，切换的所选根按已定项 14 那一句取（2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方）。

依据：`research/prompts/c199-r1-main-verification.md`、`research/prompts/c199-r2-main-verification.md`。

⚠️ **回退例外里「那次发布之前都不生效」这一句要靠影子账才成立**：回退那次发布先写它自己的 COW 单元（D16（发布语义） 已定项 7 的顺序），而分配器从 R_old 的账重新载入、I-7.4（近 K 代块未被复用） 不护被抛弃的根，那些单元可以落在最新那个根引用的单元上；崩在记录之前，下一次恢复挂上那个根、读到被复用的单元（第二轮反推腿 Y-R，零故障）。回退确认之后同样可以复用，再让回退实例的根全读不出就挂上被抛弃的根。落 [checks-owed.md](../checks-owed.md) C314（回退可以复用被抛弃的根引用的单元）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:358-416`（整段抄，未转述）**

```markdown
### 已定项 1（2026-09-13，用户定案 + E139（按盘回退下界的收严形态） + 四轮三方论证）：defer 窗口取按盘回退下界的收严形态，按状态数读

**用户定案逐字（2026-09-13）**：「选 1 最近四个状态，这个本来是承诺所有 但是后来放弃了」。回退承诺原本是整个根环里的根都可退；
2026-09-11 用户定案盘紧时候选集可以一直缩、最少保留 4 个、每块盘上最新的持久有效根都在内；2026-09-12 用户澄清「4」数的是可退到的不同状态
（只数改过用户可见状态的根，推空与抬 F 产生的空发布根不算）；2026-09-13 在三种「哪 4 个」里选了最近 4 个。

**窗口的单位是发布，不是秒**：2026-08-31 的占位 8 秒作废——defer 窗口保护的是「环里的根还能不能用」，
E92（最坏块重用延迟需求） 普查的消费者没有一个按秒计价（2026-09-11 三方论证两条云端腿一致）。

**形态**（E139（按盘回退下界的收严形态） 的臂 G2S，与装置里跑的一字不差）：

| 项 | 定案 |
|---|---|
| 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根) |
| 环里最旧有效根 | 盘上全部根槽里自证合法、按实例表判仍然有效的根的最小 checkpoint_txg；写失败的槽按旧内容算，在飞、没持久的发布不算（C282（环里最旧根没有定义） 要的定义） |
| 回退下界 F | 根记录带 8 字节（D22（单元原子性怎么合成） 已定项 7，242 → 250）。平时不动，窗口就是整个根环；只在准入不够时抬 |
| 抬 F 的上限 | min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根。一次处置的目标 = min(这次释放的释放代, 第 4 新的非空根) |
| 生效 | 每块幸存盘上都有带新 F 的持久根才生效；恢复后生效值 = 各幸存盘所带 F 最大值的最小值 |
| 回退候选集 | 按实例表判仍然有效 ∧ txg ≥ F_生效（D23（journal 的角色与格式） 已定项 14 的候选集随之加这一条） |
| 准入 | 可分配 = min(可再分配 + 活元数据 − 保留池, `df`)；准入不够时先推空发布抬 F（写行那次发布之前不推：它是新实例的第一次发布，元数据走切换预留，同一次发布里的用户数据重做照走准入、不够返回 ENOSPC；2026-09-14 用户定案，C329（写行那次发布之前推抬 F 的空发布没有检查） 三轮三方），一次准入最多 B = 4 + 2 k_tol 次发布（k_tol = 2 ⇒ 8），做满仍不够才报 ENOSPC |
| `df` | 没用过的 + 已释放的 + 活元数据 − 保留池 − 推空最坏残留 − 滞后量；保留池 = 2 × 5 + (B − 1) × c_max = 10 + 7 c_max，残留 = 5 + (B − 2) × c_max = 5 + 6 c_max；滞后量 = 释放代 > max(第 4 新的非空有效根, 环里最旧有效根) 且不是空发布放掉的已释放块 |
| 环深下限 | 每区槽数 S ≥ k_tol + 2 = 4（D22（单元原子性怎么合成） 已定项 2 的 S 下界随之抬） |

**E139（按盘回退下界的收严形态） 的判决**：全部故障世界（活化区间内崩溃不丢盘、先跳号再抬 F、推空期间 1..3 次根槽写失败、坏槽、开销升到 c_max）零假候选、零不可恢复；
近满盘、推空期间 ≤ 2 次写失败、坏槽、开销升到 c_max 各格按这条臂自己的 `df` 假性 ENOSPC 为 0；一次准入的发布数实测最大 7 ≤ 8；跑前保留池下不卡死。

**用户知情接受的代价**（2026-09-13 的选项里逐条列过）：

- 删掉的空间要再发生 3 次改变用户可见状态的发布才回可分配集合；在那之前 `df` 不把它算空闲，按 `df` 写它是真 ENOSPC ⇒ D3（空间分配） 已定项 9 第 1 条仍成立，
  第 2 条的「有界步数」按这种发布计、界是 3（D3（空间分配） 已定项 9 随之写明）。E139（按盘回退下界的收严形态）：填满之后删了再写失败在主格第 1..3 窗口、保留池扫描里到第 6 窗口，之后到第 403 窗口一次没有。
- `df` 少报 = 保留池 + 残留 = 15 + 13 c_max（c_max = 5 / 10 时 80 / 145 块）再加滞后量。滞后量 = 4 × 最近四个窗口的释放量，**没有上界**：
  一窗删一个 8 块对象时平均约 34 块、最大 44 / 52；第四轮反推腿探针一窗删 32 个时 1044 块。删掉 s 字节，`df` 一个字节都不涨。
- c_max 是声明的最坏空发布开销，口径 2026-09-13 由 D28（挂载期承诺量） 已定项 4 定为按当时结构现算（E148（提交固定点按两棵记录树重算） 池规模 9 块 ⇒ 保留池 10 + 7 × 9 = 73 块）；保留池那 10 + 7 c_max 算不算挂载期承诺量归 D28（挂载期承诺量） 已定项 4。

**按选 1 推出的三件事**（主 agent 按用户的选项推，未走三方论证，用户可推翻）：

1. D3（空间分配） 已定项 9 第 2 条的界按改变用户可见状态的发布数计，界 3；文件系统自己推的空发布不算。
2. 最少保留 4 个状态在一次处置里根槽写失败 ≤ k_tol 次时兑现。E139（按盘回退下界的收严形态） 判据 7 没写容忍例外：S = 4、c₀ = 5、3 次失败那一格违反 24 次，
   按字面记一次输，按 k_tol 的射程在外；第四轮反推腿探针全枚举下 S = 4、c₀ = 1 也违反 160 次（同为超过容忍的格）。
3. 环深下限 S ≥ k_tol + 2：第四轮反推腿探针量出 k = 2 时 S = 2 / 3 违反最少保留（184 / 224 次）、S ≥ 4 不违反，k = 3 要 S ≥ 5
   （(c₀, c_max) = (5, 10)、8 个种子，未入库）；S = 1 时连 4 个状态都给不出。

**真正的定理**（第四轮反推腿，在 E139（按盘回退下界的收严形态） 的装置副本上核过）：一个被保留的状态钉住的是在它那一刻活着、后来才死的全部块。
选最近 4 个就是每次删除都撞上——滞后量与「删了要等 3 次状态」都从这里来，是选项本身的代价，不是这一族的缺陷。

**射程与它答不了的**（照 E139（按盘回退下界的收严形态）「它答不了的」）：计数模型，按发布计不按墙钟；只建根一级的引用、没建 journal 记录重放；
实例切换的挂载时预留由 D28（挂载期承诺量） 已定项 3 扣；容量 4000 块、两块盘；k_tol 固定 2；滞后量口径是写登记的人替状态数读法构造的；
保留池扫描的最小值落在扫描下沿、真正要多少没量到；环深下限是探针的数。回退后第一次发布会复用被抛弃时间线还引用的块
（C281（回退后第一次发布会复用被抛弃时间线还引用的块）），各臂都有，病根在 D23（journal 的角色与格式） 已定项 14。

**连带改的**：D22（单元原子性怎么合成） 已定项 7 的根记录加 F（242 → 250）、已定项 2 的 S 下界；D23（journal 的角色与格式） 已定项 14 的回退候选集加「txg ≥ F_生效」；
D3（空间分配） 已定项 9 第 2 条写明界；I-7.4（近 K 代块未被复用） 的 K 改由 F 表达，C215（回退深度的承诺与 K 的下限不能同时成立） 与 C222（I-7.4 的下限 2 与第一版两盘几何要的 3 打架） 按这一形收口；
C282（环里最旧根没有定义） 的定义与 C283（准入失败时不先推发布就报 ENOSPC） 的条款都写在这里，两笔的会红检查仍欠。

**改第一个事务的字节：是**：根记录加 F 8 字节；第一个事务里 F 恒 0（mkfs 与第一次发布时非空有效根不足 4 个，上限取最旧有效根 txg 0）。

**依据**：E139（按盘回退下界的收严形态）（跑前登记 `research/prompts/e139-preregistration.md`，产物 `research/results/e139-tightened-floor-2026-09-12.out`）；
四轮三方论证（第四轮 `research/prompts/d16-item1-r4-main-verification.md`）；E135（动态回退下界）、E138（按盘回退下界与推空的空间要求） 两轮前身；用户 2026-09-11、09-12、09-13 三次定案。

```

**出处 `.claude/kb/milestone/02-second-txn.md:158-186`（整段抄，未转述）**

```markdown
## 步 4　管理员回退到 A 的根，再发布 D

**现状（2026-09-17 落地）**：`crates/singlefs-core/src/mount.rs` 的 `mount_rollback`（入参：目标根 `RollbackTarget { instance, checkpoint_txg }`、只供测试的开关 `ShadowLedger::{On, Off}`）：读根环里全部可读根（`recovery::readable_roots`），目标根不在环里报 `RollbackTargetNotInRing`；候选集按最新根指着的实例表判（(i, T) 可选 ⟺ 无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti）且 txg ≥ F_生效（各幸存盘所带 F 最大值的最小值，不是最新根自己带的 F——三方第一轮正推腿判「窄化」后改），不在候选集报 `RollbackTargetNotACandidate`；从盘上重建 R_old 那一版与分配器（与可写挂载同一条路）、R_old 之后的记录一条都不施加；影子账在每次挂载的重建里算、不只回退那一次（三方第一轮云端攻方腿打中「回退之后普通重开一次隔离归零」）：按最新根指着的实例表判被抛弃的根（有它那个实例的行且 txg > 那一行的 T），回退时再加上 (txg, 实例) 大于 R_old 的可读根，只隔离只被它们引用的槽：被抛弃根账里还分配着的槽，减去候选集里的根（可读 ∧ 按实例表有效 ∧ txg ≥ F）账里还分配着的、再减去当前这一版账里还分配着的（对用户 2026-09-16 定的窄读法措辞「仍被有效根引用的槽不在其内」的收严：原措辞的有效只按实例表判、不含 F，按它抬 F 到 11 之后 mkfs 实例表那 2 槽会被回收、发出去，而被抛弃根 B 还引用着，违反 D23（journal 的角色与格式） 已定项 14 的主句；alloc-basis 那一轮的 G5 臂，预想、偏离用户定案的措辞，交用户；代码三方第二轮辩方腿判保守读法与定案措辞相抵、云端攻方腿打中保守读法的记账缺口后改成 G5；`mount::isolate_slots_referenced_only_by_abandoned_roots`，`PoolAllocator::isolate_abandoned`：隔离位独立于分配位，不进已分配、不进 defer、不动空闲，只让用户数据落点与开放段绕开；只住内存，`MountOutput::isolated_slots_per_device` 报数）；抬 F 之后候选集缩小，回收之前按新 F 重算一遍、只会多隔离不会撤销；被抛弃根的树表或分配记录树读不出、解不开的只计数（`MountOutput::abandoned_roots_unreadable`）、不拒绝挂载（代码三方第二轮云端攻方腿打中「一条被抛弃根的树表撕裂让每次挂载都失败」）；取号 3；写行那次发布 D（txg = max(根环, 记录) + 1 = 9、jsn = 环里最大 + 1 = 9（C340（回退之后记录链从哪条之后接没有定义） 取 P2：接 R_old 那条之后会把「B 的记录已提交、根还没落盘」窗口里 B 那条盖掉，崩在回退生效之前那次恢复不再「与没发起回退时相同」，三方第一轮云端攻方腿打中）、事务号 0、反向链 0）在 R_old 那一版实例表上写回退行 (1, 3, 0, 回退位) 与中间实例行 (2, 0, 0)，重写实例表 + 四个固定点单元；暖机一次（txg 10 落盘 1）。恢复的前缀第五条接上：所选根的实例在它自己指着的实例表里有回退行时只施加到 W 为止（`recovery::rollback_high_water_of_root`；W = 0 一条都不施加）。checker：I-3.1（已分配统计对得上） 的并集只取按最新根实例表仍有效的根（被抛弃时间线的根不进）。验收（`crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs`）：回退之后冷启动读回第一次的内容、行 (1, 3, 0) 与 (2, 0, 0)、I-3.8（实例表行唯一且低于挂载根） 绿、被抛弃根独占的槽逐盘 34、隔离也是 34（A 与 B 都引用的 mkfs 实例表 2 槽不在其内）、D 与暖机一个不落在上面（`rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content`）；被抛弃的记录原样留在环里、普通重开一次隔离照旧、之后的发布不落在被抛弃根引用的槽上（`rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation`）；退到被抛弃时间线的根（B 的 (1, 4)、C 的 (2, 8)）与根环里没有的根被拒（`rolling_back_onto_an_abandoned_timeline_or_a_missing_root_is_refused`）；影子账关掉 ⇒ 回退之后两版的数据落回 B 与 C 的数据槽 50182、50184，改坏实例 3 的四个根槽之后恢复挂上 C 的根、读不回第三次的内容；开着 ⇒ 落 50186、50188、C 原样读回（`without_the_shadow_ledger_publishes_after_the_rollback_reuse_the_abandoned_data_slots_and_a_recovery_onto_the_abandoned_root_reads_a_torn_unit`，C314（回退可以复用被抛弃的根引用的单元） 两格里在层 0 之外能造出的那一格）；前缀第五条在函数级钉住（`the_rollback_row_caps_the_prefix_of_the_chosen_roots_instance_at_its_high_water`：W = 0 或 1 一条不施加、W = 2 施加到 B）；被抛弃根 C 的树表两盘都改坏 ⇒ 挂载成功、读不出的被抛弃根计 1、隔离 24（少了只被 C 引用的 10 槽；`torn_tree_table_of_an_abandoned_root_is_counted_and_does_not_fail_the_mount`）。层 0：固定脚本到 D 33 段、791624 个状态零违例，`the_fixed_script_through_the_rollback_publish_keeps_its_registered_segment_sequence` 钉段序列。变异在 `crates/mutations.tsv` 以「步 4」开头的行（回退行不带回退位、不写中间实例行、jsn 接 R_old 那条之后、影子账不隔离、普通重开不隔离、影子账把已释放的也隔离、影子账不豁免候选根引用的槽、被抛弃根读不出不计数、候选集不核、候选集的 F 用最新根自己的、前缀第五条不判、checker 并集不排除被抛弃根）。验收里造不出来的一格：「预置一条实例 1 的记录…回退行在场 ⇒ 一条都不施加；抹掉回退行 ⇒ 它被施加」——回退行写在新实例的表里、带这张表的根属于新实例，而前缀第五条问的是「所选根的实例」，两件事互斥：所选根是实例 1 的旧根时它自己的表里没有回退行，所选根是新实例的根时它不是实例 1，第五条在任何可达历史上都取不到真（三方第一轮云端攻方腿打中，C124（回退行与重放下界没有会红的检查） 那笔账按今天的实现还不清；回退行要不要另有一处不依赖新实例根的落点是格式级决定，交用户）。实做时定下的（标预想、交用户）：回退之后新实例的第一条 jsn = 环里最大 + 1（C340（回退之后记录链从哪条之后接没有定义） 取 P2；接 R_old 那条之后那种写法由变异钉红）；回退行的 W 恒 0；影子账的豁免只给候选集里的根、抬 F 之后按新 F 重算（G5，对用户窄读法措辞的收严）；被抛弃根读不出只计数不拒挂；前缀第五条只读所选根自己指着的实例表。

**设想实现**：按 D23（journal 的角色与格式） 已定项 14 的显式例外走一次回退：从回退候选集选 A 的根，不施加它之后的任何记录，取实例代号 3，
在 A 指着的那一版实例表（第一次可写挂载写行区间为空，A 那一版是零行）上写两行：回退行 (1, 3, 0)（flags bit0 = 1）与中间实例行 (2, 0, 0)，第一个新根的 checkpoint_txg = 根环里全部根 txg 的最大值 + 1（2026-09-16 D23（journal 的角色与格式） 已定项 14 随 C143（inode 号水位在回退后会退回去重发） 定案改成 max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1，这条脚本上两者相等），回退行、中间实例行、这次发布的单元与回退根同一次发布（发布 D）；
之后暖机，次数按落点公式现算：D 在 txg 9 落盘 0、txg 10 落盘 1，一次就够。B 与 C 的时间线被抛弃，只被它们引用的两个数据单元由影子账挡住，不许当空闲发出去。

**预想的细节**：
- 回退候选集 = 按实例表判仍然有效 ∧ txg ≥ F_生效（此时 F = 0 ⇒ 整个根环）；A 的根 txg 3、实例 1，在里面。
- 回退行 = `kind` 0 行、flags bit0 = 1，中间实例行 (2, 0, 0)（D18（块里携带什么信息） 已定项 11）；缺中间实例行，实例 2 就是「无行 ⇒ 可选、已发布」，C 的根回到回退候选集、影子账没有输入（三方第一轮攻方腿打中的第一条，此前这里只写回退行）。D 的根 txg = 9（C 是 8）。
- 崩在 D 之前：下一次挂载选的根、施加的记录、已发布谓词的判定都与没发起回退时相同；盘上留下取号写进超级块的 3、以及 D 已落盘而不生效的单元与记录（D23（journal 的角色与格式） 已定项 14 逐字）。
- 影子账（D28（挂载期承诺量） 已定项 1 第九项「被抛弃根独占量」）：上界 = 被抛弃根的已分配 − A 的已分配，从两条根各自可达的记账树根读；分配器把只被被抛弃根引用的槽当作不可用。⚠️ 「查谁的账」条款有两种读法：D23（journal 的角色与格式） 已定项 14 那句主语是「被抛弃时间线的根」、做法那半句写「分配器多查环里每一个可读根的账」；按主语读只隔离被抛弃根独占的槽（这一步与步 5 都按这个读法写），按做法读则 A 与实例 3 早先的根引用过的 50180 在它们被轮转覆写之前永远隔离、抬 F 在第一版 24 槽里买不到任何东西。2026-09-16 用户定案取窄读法，D23（journal 的角色与格式） 已定项 14 那半句同日改写；三方第一轮攻方腿打中的第十二条指出两种读法。
- 回退之后的暖机与步 3 同型（甲′：不向管理员确认回退，直到本实例的根覆盖两块盘）。
- 「被抛弃根独占量」在被抛弃的根被轮转覆写时清零（D28（挂载期承诺量） 已定项 1 第九项逐字），影子账隔离到被抛弃的最新根被轮转覆写为止（D23（journal 的角色与格式） 已定项 14 逐字）——第一版 24 个根槽，这个里程碑的脚本走不到那一刻，被抛弃时间线独占的两个单元一直隔离着。
- 回退那次恢复把 defer 队列、分配器游标、全部记账统计量的现行值从 A 那棵账重新载入（D23（journal 的角色与格式） 已定项 14 逐字）⇒ 步 2 在 B 里记的释放在回退之后一笔都不在账上。

**验收标准**：
- 回退之后冷启动读回的内容与第一次写入逐字节相同；实例表读出回退行 (1, 3, 0) 与中间实例行 (2, 0, 0)；I-3.8（实例表行唯一且低于挂载根） 判绿。
- 预置一条实例 1 的、txg 4 之后的记录（校验和过、点名单元在）：回退行在场 ⇒ 只施加到 W = 0 为止、一条都不施加（前缀第五条逐字「有回退行时」）；把回退行抹掉 ⇒ 它被施加，判红。
- 崩在 D 的每个崩溃点：根槽没持久 ⇒ 恢复到 C 的根、文件是第三次的内容；根槽已持久 ⇒ 恢复到 D、文件是第一次的内容；两边 checker 都绿。
- 准入读数：可用比回退前少了正好被隔离的那几块（C318（影子账隔离的单元没进准入不等式） 的判别力自证：把第九项置 0 必须多报正好那几块）。
- 必红（C314（回退可以复用被抛弃的根引用的单元） 两格，E150（回退复用被抛弃的根引用的单元） 的形态）：关掉影子账 ⇒ ① 回退那次发布崩在单元写完之后、记录之前，恢复挂上最新那个根、它引用的单元已被复用 ⇒ I-7.4（近 K 代块未被复用） 红；② 回退确认之后复用、再让回退实例的根全读不出 ⇒ 恢复挂上被抛弃的根，同一判决。影子账打开 ⇒ 两格 0 违例。
- 变异：回退行不写 ⇒ 下一次恢复把 B、C 的记录施加回来，读回内容是第三次的，验收断言红（C333（删行那次发布被重放） 同族，没有不变量号）；只写回退行、不写中间实例行 ⇒ C 的根回到回退候选集，判红（C124（回退行与重放下界没有会红的检查） 那一族今天缺的形态）。

**写出的字节**：[layout/01-first-txn.md](../layout/01-first-txn.md)「一、mkfs 已经种下的（第一个事务之前就在盘上）」（实例表单元多两行：回退行与中间实例行）「七、发布（根记录与根槽）」「八、根槽写路径的段序列登记表（C316（提交步骤的登记位有四处且互不相同） 的登记位，2026-09-13 立）」（实例切换 / 管理员回退那一行从预想写成字节）。

**会碰到的决策点**：D23（journal 的角色与格式） 已定项 14；D28（挂载期承诺量） 已定项 1；D16（发布语义） 已定项 8；C314（回退可以复用被抛弃的根引用的单元）；C281（回退后第一次发布会复用被抛弃时间线还引用的块）；C331（择根倒挂压过已确认的写）；C332（回退实例两个根都读不出时回退被撤销）；C333（删行那次发布被重放）；C143（inode 号水位在回退后会退回去重发）；C124（回退行与重放下界没有会红的检查）；C340（回退之后记录链从哪条之后接没有定义）（2026-09-16 新立：回退之后新实例的第一条记录接在哪个 jsn 之后）；影子账查谁的账（2026-09-16 用户定案取窄读法）；回退之后的暖机次数（同步 3 那一格）；层 0 oracle 的输入：它今天按 (txg, 实例) 字典序认最新持久根（D22（单元原子性怎么合成） 已定项 7 的择新序；三方代码第一轮攻方腿打中此前只按 txg），回退之后被抛弃实例的根与回退实例的根同时在环里、恢复正确地不选被抛弃的那条会被它判成「根槽已持久而恢复到旧态」——这一步开工前给 oracle 实例表当输入。仓里两个择新序并存：根环择新 txg 为主（D22（单元原子性怎么合成） 已定项 7）、journal 水线实例为主（D23（journal 的角色与格式） 已定项 14），各对各的条款，oracle 只跟了前者；一条实例低、txg 高的记录在根环序里在所选根之上、在水线之下，单实例流走不到、回退一开就走得到（第二轮攻方腿记一笔）。

```

**出处 `.claude/kb/milestone/02-second-txn.md:187-213`（整段抄，未转述）**

```markdown
## 步 5　抬回退下界 F，延迟之后重用第一个数据单元的落点（发布 E）

**现状（2026-09-17 落地）**：可再分配谓词接上：`PoolAllocator::reclaim_released_up_to(floor)` 把释放代 ≤ floor 的已释放落点回到空闲（位图清、占着与 defer 各减、空闲加；D16（发布语义） 已定项 1，第一版环里最旧有效根恒 0、floor 就是 F_生效）；回收过的落点被再分配时那条记录改写（代 = 这次 txg、已释放位清），槽号每盘仍唯一。`crates/singlefs-core/src/mount.rs`：`rollback_floor_ceiling` = min(每块盘上最新的有效根, 第 4 新的非空有效根)（有效 = 自证 ∧ 按最新根实例表有效 ∧ txg ≥ 今天的 F；非空 = 环里有它自己那条记录且事务号非 0，预想）；`raise_rollback_floor(new_floor)` 是只供测试的强制入口（正常触发是准入不够）：超过上限报 `RollbackFloorAboveCeiling`，先回收、再连推带新 F 的空发布直到每块盘上都有一条（生效），返回 `RaisedFloor { ceiling, publishes, reclaimed }`；重开时 `recovery::effective_rollback_floor`（各幸存盘所带 F 最大值的最小值）既给分配器回收、也写进新实例的根——一条根带的 F 与它的记账行说同一件事，checker 的 I-3.1（已分配统计对得上） 并集按最新根自己的 F 收候选集（F 之下的根不走，I-2.1（校验和与内容匹配） 也不在它们上判）。第一个文件版本重写树表时把 mkfs 那片第 0 版树表单元释放（不释放它，抬 F 之后那一槽永远占着、I-3.1（已分配统计对得上） 红；第一个事务的字节表五随之改，E142（第一个事务的干跑） 第十次跑）。验收（`crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs`）：回退之后四次覆盖写（txg 11–14）、上限 11、抬到 11 两次空发布（15 落盘 0、16 落盘 1）、回收 18 个落点（含 mkfs 树表、D 与暖机放掉的）、defer 从 51 到 38、E（txg 17）的数据单元落回最低的可再分配偶数槽对 50178（mkfs 树表那 1 槽回收了、50179 从没分配过；mkfs 实例表那片 50176 也回收了但 B 的根还引用它、影子账隔离着；A 的数据单元 50180 排在后面——此前预想 50180）、那条记录改写成代 17、冷启动读回 E、根记录 F = 11、checker 全绿（`raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it`）；抬到 12 被拒（`raising_the_floor_above_the_fourth_newest_non_empty_root_is_refused`）；不抬 F 直接回收（复用窗口置 0）⇒ E 落回 50178、第 0 代根还在候选集里（F = 0）、它们引用的 mkfs 树表单元被盖 ⇒ I-2.1（校验和与内容匹配） 红（`reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red`，C22（刚释放的块立即重分配） 的必红）；只有一块盘带新 F ⇒ 重开后 F_生效 回到 0、一个都不回收（`one_device_carrying_the_floor_alone_does_not_take_effect_on_remount`），那时 txg 在两个 F 之间的根仍是回退候选（`roots_below_a_floor_carried_by_only_one_device_remain_rollback_candidates`）。回收门槛 = max(F_生效, 环里最旧有效根)（`mount::reclaim_floor`；第 0 代根被盖之前恒 F_生效，三方第一轮本地攻方腿与 alloc-basis 那一轮都指出此前省了后一项）；重建分配器时认得出还没被换下的第 0 版树表单元（本地辩方腿两份样本都指出 mkfs 与第一个文件版本不在同一个进程里那一格）。层 0：固定脚本到 E 54 段、2104413 个状态（快的那条 108 个），全量交门禁 54 号。变异在 `crates/mutations.tsv` 以「步 5」开头的行（其中两条钉代码三方第二轮的改法：抬 F 之后不按新候选集重算影子账、影子账的豁免不看候选根的 txg 是否 ≥ F；三条钉 alloc-basis 第二轮云端攻方腿打中之后的改法：抬 F 回收的槽不扣住、生效之后不放开、没做过可写挂载的进程抬 F 时 panic）。抬 F 回收的槽扣住到生效（`ReclaimedReuse::HeldUntilFloorTakesEffect`，`DeviceFreeMap` 的扣住位让 `is_free` 与开段都绕开它，带新 F 的根落满每块盘之后 `release_reclaim_holds` 放开；alloc-basis 第二轮云端攻方腿打中：抬 F 自己的空发布在开放段满了之后开到刚回收空的那一段，崩在两条带新 F 的根之间时 F 之下的根仍是候选、它们的单元已被盖；用例 `slots_reclaimed_by_raising_the_floor_are_not_handed_out_before_the_floor_takes_effect`：A、B、重开、六次覆盖写、抬 F 到 8，扣住之前 txg 14 的记账树落在回收的 50240 上）；记账仍在第一条带新 F 的根之前动，按写那条根那一刻的 F 还是它持久之后的 F_生效 算的口径归 alloc-basis 那一轮（预想）。没做过可写挂载的进程抬 F 报 `RaiseNeedsWritableMountInThisProcess`（`raising_the_floor_in_a_process_that_never_mounted_writable_is_refused_instead_of_panicking`）。同一次挂载里转环不回收（回收只在重建与抬 F 两处；固定脚本走不到 txg 24，alloc-basis 第二轮云端攻方腿 1.1 / 1.2 节，口径归那一轮）。没做的：抬 F 的正常触发（准入）、I-7.4（近 K 代块未被复用） / I-4.8（近 K 代根校验和自洽） 的 checker。决策点：checker 怎么认「非空持久有效根」（今天按环里记录的事务号）；F 只在一块盘上时新实例的根写 F_生效（把 F 写低）是不是条款的意思。

**设想实现**：回退之后再覆盖写一次（释放第一个数据单元，释放代 = 这次的 txg），接着推几次非空发布把「第 4 新的非空持久有效根」攒到那个 txg 之上，
再用只供测试的开关触发一次抬 F 的空发布，把根记录的 F 写成那个释放代；然后发布 E 写入时分配器把 50180 发出去——正好是第一个数据单元的落点。
释放代大于 F 的那些单元不许发出去，分得清。

**预想的细节**：
- 可再分配 ⟺ 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)（D16（发布语义） 已定项 1）。根环 24 槽、mkfs 的种子 txg 0 ⇒ 环里最旧有效根在前 24 次发布里恒为 0，第一版的重用只能靠抬 F。
- 抬 F 的上限 = min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；一次处置的目标 = min(这次释放的释放代, 第 4 新的非空根)。「4」数的是可退到的不同状态——只数改过用户可见状态的根，推空与抬 F 产生的空发布根不算（D16（发布语义） 已定项 1 逐字）⇒ 攒根的那几次必须是覆盖写，空发布不算数。**预想**：释放那次发布之后再覆盖写 3 次（每次释放前一个单元），上限刚好够到第一次的释放代。
- 抬 F 的正常触发是准入不够；测试用开关强制触发（步 0）。生效 = 每块幸存盘上都有带新 F 的持久根 ⇒ 抬 F 的空发布之后还要让两块盘各有一个带新 F 的根，**预想**再推一次。
- 分配器：政策函数「已选设备内槽号最小」会先把 50180 发出去；后释放的那几个单元释放代大于 F，仍在 defer 队列里。
- 记账：defer 待释放减掉发出去的那一块、已分配加回来；空闲字节到这时才动。

**验收标准**：
- E 之后冷启动：新内容读回正确，它的数据单元落在 50180–50181，映射条目的位置就是它；后释放的那几个单元仍在 defer 队列里、落点没被发出去。
- 根记录读回 F = 释放代；checker：回退候选集只剩 txg ≥ F 的根，I-7.4（近 K 代块未被复用） 对候选集里每个根判绿；I-4.8（近 K 代根校验和自洽） 从候选集里任一根出发遍历判绿。
- 抬 F 那次空发布的段序列与登记表「抬 F 的空发布」那一行逐字相同。
- 必红（C22（刚释放的块立即重分配））：复用窗口置 0 让 E 在 F 还是 0 时就复用 ⇒ I-4.8（近 K 代根校验和自洽） 从 A 的根出发遍历，第一个数据单元的校验和与父指针对不上，红。
- 变异：F 抬过上限（候选集少于 4 个可退到的状态）⇒ 要红的是一条新检查，checker 从盘上算「第 4 新的非空持久有效根」得先认得出哪些根改过用户可见状态，今天没有条款说 checker 怎么认（三方第一轮正推腿判据八那格；攻方腿没打中的第一条 按三种「非空」读法在这条脚本上同答 11，换一条脚本可能不同），列成决策点；F 只在一块盘的根上 ⇒ 恢复后生效值仍是旧的，E 发不出 50180，验收断言红。
- 判别力自证（影子账的读法）：把影子账的查账集合从「被抛弃的根」放宽到「环里全部可读根」⇒ E 必须发不出 50180；不加这一条，「落在 50180」对两种读法之一恒真（攻方腿打中的第十二条）。

**写出的字节**：[layout/01-first-txn.md](../layout/01-first-txn.md)「七、发布（根记录与根槽）」（F 第一次不为 0）「五、空间记账与分配」（已释放记录被重用之后再改写一次）「八、根槽写路径的段序列登记表（C316（提交步骤的登记位有四处且互不相同） 的登记位，2026-09-13 立）」（抬 F 的空发布那一行从预想写成字节）。

**会碰到的决策点**：D16（发布语义） 已定项 1；D3（空间分配） 已定项 7 / 已定项 8；D28（挂载期承诺量） 已定项 1；C22（刚释放的块立即重分配）；C282（环里最旧根没有定义）；C215（回退深度的承诺与 K 的下限不能同时成立）；C222（I-7.4 的下限 2 与第一版两盘几何要的 3 打架）；C283（准入失败时不先推发布就报 ENOSPC）；checker 怎么从盘上认「非空持久有效根」；影子账查谁的账（同步 4）；A 的根被根环轮转覆写之后 checker I-3.1（已分配统计对得上） 读法甲的并集不再含 A 独占的那 10 槽、而分配器的「占着」不缩 ⇒ 回收要在覆写之前把已释放落点放回空闲，或者「已分配」按仍被有效根引用维护（三方代码第一轮本地腿第二次抽样提出、主 agent 核实机理，`research/prompts/m2-code-r1-main-verification.md`）。

```

**出处 `.claude/kb/decisions/28-挂载期承诺量.md:30-30`（整段抄，未转述）**

```markdown
**第九项「被抛弃根独占量」是 2026-09-13 用户定案加的**（C318（影子账隔离的单元没进准入不等式）；C314（回退可以复用被抛弃的根引用的单元） 取影子账之后，只被被抛弃的根引用的单元在当前账里是空闲的、分配器却不许拿它们，直到被抛弃的根离开根环）：上界 = 被抛弃根的已分配统计量 − R_old 的已分配统计量，两个数各从那条根可达的记账树根读、不遍历树、不加盘上字段；回退选中 R_old 那一刻算出、只住内存、按设备算（已分配统计量带设备维），被抛弃的根被轮转覆写时清零。由回退路径维护。E150（回退复用被抛弃的根引用的单元） 的几何下隔离量 2–3 个单元、到第 11 次发布归零；判别力自证（把隔离量置 0，只差那几块的池要从只读翻成可写）仍记在 C318（影子账隔离的单元没进准入不等式）。
```

**出处 `.claude/kb/checks-owed.md:296-296`（整段抄，未转述）**

```markdown
| C318 | 影子账隔离的单元没进准入不等式 | **C314（回退可以复用被抛弃的根引用的单元） 取影子账（2026-09-13 用户定案）之后，只被被抛弃的根引用的单元在 R_old 的账里是空闲的、分配器却不许拿它们（被抛弃的根离开根环之前）**，而 D28（挂载期承诺量） 已定项 1 的准入不等式八项里没有一项装它——「已分配」按当前账算不含它们、「defer 待释放」按代按队列算也不含 ⇒ 可用被高估，E139（按盘回退下界的收严形态） 那类没扣进 df 的量会翻成假性 ENOSPC（第四轮探针 > 60 块就出 24 次）。E150（回退复用被抛弃的根引用的单元） 的几何下隔离量 2–3 个单元、到第 11 次发布归零，池规模下随被抛弃时间线的分配量走 | 准入不等式加一项或并进 defer 待释放：候选口径「被抛弃根独占量」的上界 = 被抛弃时间线自 R_old 起的分配量（按根记录携带的已分配统计量之差算，不遍历树），被抛弃的根被轮转覆写时减回；对照表登记它由谁维护；判别力自证：把隔离量置 0，只差那几块的池要从只读翻成可写 | 前置已还：2026-09-13 用户定案加第九项「被抛弃根独占量」（D28（挂载期承诺量） 已定项 1），口径取「被抛弃根的已分配统计量 − R_old 的」、从两条根各自可达的记账树根读，不加根记录字段；判别力自证的模型形态同日由 E150（回退复用被抛弃的根引用的单元） 第二次跑做出（带第九项的可用 = 分配器发得出的槽数，不带就多报 2 块），实现侧的检查登记在 [verification-build.md](verification-build.md) 事务层第一版「运行时计数与准入自证」表 | 2026-09-13 重新推理确认时发现；同日用户定案加项 |
```

**出处 `.claude/kb/checks-owed.md:303-303`（整段抄，未转述）**

```markdown
| C332 | 回退实例两个根都读不出时回退被撤销 | 管理员回退之后，回退实例的根在两块盘上都读不出（2 个故障）时，恢复选到被抛弃时间线上的根（择根不看实例表，回退行救不了），或落回 R_old 并施加 R_old 之后被抛弃的记录，回退被静默撤销、之后已确认的写丢掉；与暖机只防单故障同一类。D18（块里携带什么信息） 已定项 11「回退行不许被后来的恢复覆盖（落在 R_old 上的那次恢复会给 [所选根的实例, 新实例) 每个实例写行，正好盖到它）」与 D23（journal 的角色与格式） 已定项 14「重放下界遇回退行截断」按表随根分版本读都没有输入：落在 R_old 上的恢复读的是 R_old 那一版表，回退行不在里面 | 多次挂载的崩溃点重放里，回退之后注入「回退实例的根全部读不出」，逐状态判：挂载根不在被回退抛弃的时间线上；判别力自证：今天的择根在这个注入下必须红 | 择根要不要看实例表、回退要不要多一份持久见证，没有条款；多次挂载的录制流 | 2026-09-14 C329（写行那次发布之前推抬 F 的空发布没有检查）、C330（中间实例那一行的 T_pub 取所选根的 txg） 第二轮攻方腿（`research/prompts/c329-c330-r2-main-verification.md` 第 46 行）、第三轮攻方腿第三节（`c329-c330-r3-opus-output.md`） |
```

**出处 `.claude/kb/checks-owed.md:311-311`（整段抄，未转述）**

```markdown
| C340 | 回退之后记录链从哪条之后接没有定义 | D23（journal 的角色与格式） 已定项 14 回退段（23:1206）只写「不施加 R_old 之后的任何记录」，切换段（23:1233）写「链从所选根覆盖的最后一条记录之后接」；回退之后新实例的第一条记录接在哪（R_old 覆盖的最后一条 + 1，还是可读链末尾 + 1）要从切换那一段借「所选根」推出来，回退自己那一段没有一句——照 `.claude/rules/fs-design.md`「借用式条款要把被借规则的每个参数都绑死」 | 两种取法各中一格（代码三方两轮实测，`research/prompts/m2-step45-code-r1-main-verification.md`、`m2-step45-code-r2-main-verification.md`）：P1（R_old 覆盖的最后一条 + 1）盖掉「B 的记录已提交、根还没落盘」窗口里 B 那条，崩在回退生效之前的恢复不再与没发起回退时相同；P2（可读链末尾 + 1，今天 `crates/singlefs-core/src/mount.rs` `mount_rollback` 的取法，预想）把被抛弃的记录留在环里，(txg, 实例) 大于 R_old 的每条可读根都坏掉时恢复落回 R_old、把同实例连号的那段被抛弃记录整段重放（固定脚本上 5 个故障重放 3 条）。取哪个交用户；定下之后变异表钉另一个必红（今天钉的是 P1 必红：`crates/mutations.tsv`「回退之后 jsn 接 R_old 那条之后」） | D23（journal 的角色与格式） 已定项 14 回退段补一句；回退行要不要另有一处不依赖新实例根的落点（前缀第五条活过来、两种取法的那一格都不中） | C143（inode 号水位在回退后会退回去重发） 第二轮攻方腿（H10）、第三轮辩方腿复核 |
```

**出处 `.claude/kb/invariants.md:50-50`（整段抄，未转述）**

```markdown
| I-7.4 | 近 K 代块未被复用 | 回退候选集里每一个根（按实例表判仍然有效 ∧ txg ≥ F_生效，D23（journal 的角色与格式） 已定项 14）所引用的块，其物理范围均未被重新分配给其他对象、也未被清扫抹头。**「K 代」2026-09-13 起就指这个候选集**：平时回退下界 F 不动时是整个根环，盘紧时抬 F 缩到最近 4 个可退到的状态、且每块盘上最新的持久有效根都在内（D16（发布语义） 已定项 1）；原「K 是运行时策略、下限 ≥ 2」作废。[decisions.md](decisions.md) D22（单元原子性怎么合成）：**根环深度是块重用延迟的上界**（D22（单元原子性怎么合成）同时明令块复用的界不能是格式常量），不满足则环里的旧根是假的回退候选。「有效」按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 的回退候选集判：(i, T) 有效 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti——被抛弃时间线的根不在候选集里，这只管回退目标的选择；它们引用的块在离开根环之前同样不许重新分配、不许抹头（2026-09-13 用户定案，C314（回退可以复用被抛弃的根引用的单元） 取影子账，D23（journal 的角色与格式） 已定项 14）。崩溃后 defer 队列的内存态丢失，接手的是清扫准入「释放代 ≤ max(F_生效, 环里最旧有效根)」那条闸（谓词由 D16（发布语义） 已定项 1 给，2026-09-13；有效性按实例表判），保护宽度随之变 | 未实现 |
```

**出处 `.claude/kb/invariants.md:107-107`（整段抄，未转述）**

```markdown
| I-2.1 | 校验和与内容匹配 | 任一被引用的块，其校验和与内容匹配 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；2026-09-17 起「被引用」按 I-3.1（已分配统计对得上） 那个候选集里的根算：F 之下的根引用的单元可以已被回收复用，不在它们上判——不抬 F 就回收复用时 A 的根还在候选集里、这一条红，是 C22（刚释放的块立即重分配） 的必红） |
```

**出处 `.claude/kb/invariants.md:120-120`（整段抄，未转述）**

```markdown
| I-3.1 | 已分配统计对得上 | 已分配空间统计 == 实际遍历所有引用得到的和 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） ⚠️ **checker 读法（2026-09-14 用户收尾弹窗定甲）**：「实际遍历所有引用」按根环里全部有效根的引用取并集——第一个事务之后第 0 版树表单元只被更早的根引用、仍占着空间（已释放、在 defer 队列里），只按最新根算会少 1 槽。**2026-09-17 起「有效根」= 回退候选集里的根**：按最新根指着的实例表有效（无那个实例的行，或有行 (i, Ti, Wi) 且 T ≤ Ti）且 txg ≥ 最新根自己带的回退下界 F（checker 只看一个镜像，读最新根那一份 F；回退候选集用的 F_生效 要跨盘算（各幸存盘所带 F 最大值的最小值，D16（发布语义） 已定项 1），一块盘的载体根坏掉之后两者可以不同：那段时间 checker 按最新根的 F 判，下一次可写挂载的新根把 F_生效 写回盘上——代码三方第二轮云端攻方腿打中 F 回落让三条不变量一起红，口径交用户，见里程碑「第二个事务」步 5 的决策点）——被抛弃时间线的根引用的单元由影子账隔离、F 之下的根引用的已释放单元已可再分配，都不在当前账里（里程碑「第二个事务」步 4 / 步 5，`crates/singlefs-checker/src/walk.rs`） |
```

**出处 `research/prompts/alloc-basis-r2-opus-output.md:164-213`（整段抄，未转述）**

````markdown
### 2.1 F-今 在「第一条带新 F 的根持久、第二条没持久」那一格把回收的槽发了出去（打中）

`mount.rs:354-356` 注释整行：

```
    // 回收在写第一条带新 F 的根之前：一条根带的 F 与它的记账行要说同一件事——checker 的 I-3.1（已分配统计对得上） 按那条根自己的 F
    // 取候选集的并集，释放代 ≤ F 的落点不在并集里，记账的「已分配」也不能再算它们。生效（两块盘都有带新 F 的根）之前这个进程不会再发布，
    // 回收的槽在生效之前发不出去。
```

最后一句不成立：`mount.rs:371-399` 那个循环自己就在发布，每次空发布经 `publish_version` 重写四个固定点，提交内生块从开放聚簇段 bump（`allocator.rs:495-522`），开放段满了就开「单元区内最低的、used 与 isolated 都为 0 的 64 槽段」（`allocator.rs:297-312`）。刚回收完的一整段就是这样的段。

历史（全部是合法操作；探针 `legal-hit 1 6 8 1 1 3`，抬 F 那一步照 `mount.rs:331-399` 一行一行做、停在第一次发布之后，因为 `raise_rollback_floor` 本身不停在中间；实例表改从盘上最新根读，理由见第七节 1）：

| txg | 操作 | 带的 F | 落哪块盘 |
|---|---|---|---|
| 3 | A（第一个事务，实例 1） | 0 | 盘 0 |
| 4 | B 覆盖写 | 0 | 盘 1 |
| 5 / 6 / 7 | 重开（实例 2）：写行 / 暖机 / 暖机 | 0 | 盘 0 / 盘 0 / 盘 1 |
| 8–13 | 六次覆盖写 | 0 | — |
| — | 抬 F 到 8（上限 10 = 第 4 新的非空根；回收 30 个落点） | — | — |
| 14 | 抬 F 的第一次空发布 | 8 | 盘 0（区域 2） |
| — | **崩**（txg 15 的根没写） | — | — |

`outputs/s4l-legal-hit-k1-1-k2-6-f8.txt` 第 1–6 行整行：

```
S4L before-raise txg=13 inst=2 F_root=0 F_eff=0 acct_alloc=97 acct_free=211871 acct_defer=85 mem_alloc=97 mem_defer=85 mem_isolated=0 | GREEN
S4L ceiling=10 reclaimed_count=30
S4L publish txg=14 F=8 wrote=[(AllocationTree, 50367, 1), (AccountingTree, 50240, 1), (MappingTree, 50241, 1), (TreeTable, 50242, 1)]
S4L crash-state txg=14 inst=2 F_root=8 F_eff=0 acct_alloc=66 acct_free=211902 acct_defer=54 mem_alloc=66 mem_defer=54 mem_isolated=0 | GREEN
S4L crash->rollback(victim) failed: Recovery(UnitUnreadable { slot: SlotNumber(50240) })
S4L crash->remount txg=16 inst=3 F_root=0 F_eff=0 acct_alloc=108 acct_free=211860 acct_defer=96 mem_alloc=108 mem_defer=96 mem_isolated=0 | I-2.1[树 11（种类 1）的根 在盘 0 槽 50240 的那一份与位置条目里的校验和对不上] ; I-5.1[盘 0：树表单元（槽 50242 跨 1）与 inode 树的孩子（槽 50242）重叠]
```

读法：
- 崩溃态盘上只有盘 0 带着 F = 8 的根，`recovery.rs:351` 的 `effective_rollback_floor` 算出 0，A（txg 3，实例 1，实例表里那一行 T = 4，3 ≤ 4）按 `16-发布语义.md:376`（`| 回退候选集 | 按实例表判仍然有效 ∧ txg ≥ F_生效（D23（journal 的角色与格式） 已定项 14 的候选集随之加这一条） |`）是回退候选。
- txg 14 把记账树写在 50240（A 的 extent 根）、树表写在 50242（A 的 inode 叶）。今天的 checker 用最新根自己带的 F（`walk.rs:786-790`）把 A 排出候选集，崩溃态全绿。
- 管理员照 `mount_rollback` 回退到 (1, 3)：候选检查放行，读 A 的单元时报 `UnitUnreadable { slot: 50240 }`。
- 不回退、普通重开：新实例的根写 F_生效 = 0（`mount.rs:460`），A 回到 checker 的候选集，I-2.1 与 I-5.1 红——一段只由合法操作组成的历史，结尾是一个 checker 判红的镜像。
- 同一个崩溃态换成「checker 按恢复的定义现算 F_生效」（`SFPROBE_CHECKER_FLOOR=effective`），`outputs/p-s4l-legal-hit-k1-1-k2-6-f8-checkerF-effective.txt` 第 4 行整行：`S4L crash-state txg=14 inst=2 F_root=8 F_eff=0 acct_alloc=66 acct_free=211902 acct_defer=54 mem_alloc=66 mem_defer=54 mem_isolated=0 | I-2.1[树 11（种类 1）的根 在盘 0 槽 50240 的那一份与位置条目里的校验和对不上] ; I-3.1[盘 0：记账的已分配 Some(1081344)，遍历全部有效根得到 1605632] ; I-5.1[盘 0：树表单元（槽 50242 跨 1）与 inode 树的孩子（槽 50242）重叠]`。

它有多常见：`legal-search 20 25`（k1 = 1..20 次覆盖写 → 重开 → k2 = 0..25 次覆盖写 → 抬 F 到 (旧 F, 上限] 的每个值）12989 格，`outputs/s2l-legal-search.txt` 末两行整行 `S2L SUMMARY cells=12989 clean_hits=258` 与 `PROBE-COMPLETE`；打中的 (k1, k2, f) 格共 138 个；其中 91 格有干净崩溃态（写到受害槽的那次发布不是让 F 生效的最后一次、受害根没被这次发布自己的根盖掉），干净的那 258 行里受害根是 (1, 3) 240 行、(1, 10) 6 行、(1, 17)–(1, 22) 各 2 行；另 47 格只在最后一次发布上打中（要崩在它的单元写完、根没写之前），没逐格核。（数法：`grep "S2L HIT" outputs/s2l-legal-search.txt` 取第 3–5 列去重，带 `last=false` 的再去重。）不重开、只在同一进程里覆盖写的那一族（`s2-window-search.txt`）也打中，但要在抬 F 之前多一次空发布才对得上开放段的边界，那次空发布今天没有合法来源，只当旁证。

四句：
1. 分不分辨臂：分辨。同一搜索的子集（k1 ≤ 8、k2 ≤ 16）换成 F-生（回收推迟到两条根都持久之后，副本上 `SFPROBE_T1=1 SFPROBE_RAISE=fsheng`），`outputs/p-s2l-legal-search-T1-fsheng-8-16.txt` 整份两行：`S2L SUMMARY cells=1942 clean_hits=0`、`PROBE-COMPLETE`（最后一次发布上的也是 0 行）；F-今 在同一子集（k1 ≤ 8、k2 ≤ 16）上 88 格打中、其中 60 格有干净崩溃态。
2. 看得到吗：看得到。写者知道第二条带新 F 的根还没写；checker 从盘上各根带的 F 就能算出 F_生效。
3. 字面：W2（`_alloc-basis-r2-body.md:87`），候选集按 `16-发布语义.md:376`；崩溃态 checker 全绿，同时是 W4。它也违反 `16-发布语义.md:371` 与 `:375`（`| 生效 | 每块幸存盘上都有带新 F 的持久根才生效；恢复后生效值 = 各幸存盘所带 F 最大值的最小值 |`）：回收用的是还没生效的 F。
4. 改法：F-生（回收推迟到生效之后）在这 91 格上不中（子集 0）。「先回收、但开段时绕开回收的槽直到生效」也修得掉，没量。

````

**出处 `research/prompts/alloc-basis-r2-opus-output.md:420-420`（整段抄，未转述）**

```markdown
1. `raise_rollback_floor` 在一个从没做过可写挂载的进程里会 panic：`mount.rs:333` 整行 `    let table = InstanceTableRecords::parse(&current.unit(TransactionUnit::InstanceTable).bytes)`，而 `transaction.rs:535` 整行 `            .expect("八个文件 / 固定点角色每种一个；实例表单元要先重写过一次才在")`——实例表单元要等可写挂载写行时重写过才在 `units` 里。副本探针第一版照这一行写，单实例的搜索当场 panic（那次输出整行 `thread 'main' (1556105) panicked at crates/singlefs-core/src/transaction.rs:535:14:`，下一行是上面那句 expect 的消息；那份输出已被重跑覆盖，没留档），之后改成从盘上最新根读实例表。今天步 5 的验收都在回退之后调它，走不到。归步 4 / 步 5 那一轮。
```
