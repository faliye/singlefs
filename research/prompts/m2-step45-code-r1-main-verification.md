# 里程碑「第二个事务」步 4 / 步 5 那批代码三方对抗第一轮：主 agent 核实（2026-09-17）

<!-- doc-lint:not-numbers S1 S2 S3 S4 S5 S6 S7 S8 S9 S10 S11 S12 X1 X2 X3 X4 X5 X6 X7 X8 X9 X10 P1 P2 F1 F24 -->

材料：正文 `research/prompts/_m2-step45-code-r1-body.md`（S1–S12 选择表、X1–X10 判据、四条腿的分工）、背景材料 `_m2-step45-code-r1-background.md`（正文 + 15 份 kb 文件的小节清单 + 附录一 40 段整抄）、附录二 `_m2-step45-code-r1-diff.md`（从提交 5f9e449 起 `crates/` 的全部 diff + 五个新文件全文 + `crates/mutations.tsv`）。
四条腿：云端攻方（Opus，`m2-step45-code-r1-opus-output.md` 378 行，模型 `m2-step45-code-r1-opus-model/opus_attack_m2_step45.rs`，六条用例）、云端正推（Sonnet，`m2-step45-code-r1-sonnet-output.md` 602 行）、本地攻方（`m2-step45-code-r1-local-attack-output-s1.md`、`-s2.md`，七次里五次判损坏）、本地辩方（`m2-step45-code-r1-local-defense-output-s1.md`、`-s2.md`，五次里三次判损坏）。核查员 `m2-step45-code-r1-verifier-output.md`（第七节）。另一轮 alloc-basis-r1（另一个会话，`research/prompts/alloc-basis-r1-main-verification.md`）的四笔账与这一轮重叠，第五节按账号标。

被判的文件（门禁 56 号按路径认）：`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/instance_table.rs`、`crates/singlefs-core/src/lib.rs`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-harness/src/crash.rs`；用例 `crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs`、`second_transaction_step_five_reuse.rs`、`second_transaction_step_zero_layer0.rs`、`second_transaction_step_three_second_instance.rs`；这一轮的改法落在 `mount.rs`、`allocator.rs`、`crash.rs`（第五节）。

## 一、本地攻方（X5、X6、X7、X9；两份样本）

| 问 | s1 | s2 | 主 agent 核 |
|---|---|---|---|
| 1（X5 回收与复用） | 1a / 1b HIT：回收门槛只用 F_生效、省了「环里最旧有效根」，24 次发布之后最旧的根不是 txg 0；1c / 1d HIT：复用时 defer 计数不减 | 1a HIT（同）；1b NO HIT（少回收是安全方向）；1c / 1d NO HIT | 1a / 1b 两份一致，**打中**：`reclaim_released_up_to(floor)` 的 floor 此前只取 F_生效，D16（发布语义） 已定项 1 的谓词是 max(F_生效, 环里最旧有效根)；固定脚本到 txg 17 走不到 txg 24，代码却省了那一项（alloc-basis 那一轮账 1 同一条）。1c / 1d 不中：回收那一刻 `mark_reclaimed` 已把 defer 减掉，复用只走 `reclaimed` 集合里的落点，s1 把「回收」与「复用」记成一步 |
| 2（X6 上限与生效） | 2a HIT：撕掉的记录让非空根被算成空、上限偏低；2b HIT：事务号非 0 但没改用户可见状态；2c HIT：walk 用最新根自己的 F、allocator 用各盘最小 | 2a NO HIT（撕掉的记录不进表，上限偏低是安全方向）；2b NO HIT（第一版事务号 0 只给空发布）；2c HIT（同） | 2a：两份相反，上限偏低是保守方向，记决策点「非空按记录事务号认」；2b：第一版事务号非 0 就是文件版本，不中；2c 两份一致：两个 F 各有用途——walk 的候选集与那条根自己的记账行说同一件事（S8），回退候选集与回收用 F_生效——**不是代码错，是口径没写**，alloc-basis 那一轮账 5 |
| 3（X7 checker 候选集） | 3a HIT（同 2c）；3b / 3c NO HIT | 3a HIT（同）；3b / 3c NO HIT | 同 2c |
| 4（X9 第一个事务改字节） | 没打中 | 没打中 | 与正推腿 S10 一致 |

## 二、本地辩方（两份样本，八问）

| 问 | s1 | s2 | 主 agent 核 |
|---|---|---|---|
| ① P1 | 站得住 | 站得住（环是定长的、盖槽是设计） | 两份都没看到云端攻方的那段历史；P1 出局（第四节 X2） |
| ② 回退行 W 恒 0 | 站得住 | 站得住 | 与条款一致 |
| ③ 影子账窄读法 | 站不住：不查 R_old 记录的分配状态、只住内存 | 站不住（同） | 两份一致，与云端攻方 X3 同一格；改法见第五节 |
| ④ 非空按记录事务号 | 站不住（谓词没测过） | 站得住（第一版事务号 0 只给空发布） | 记决策点 |
| ⑤ 先回收再推带新 F 的根 | 站不住：回收与落盘之间可以插一次真写 | 站不住（同） | 不中：`raise_rollback_floor` 里回收与两次空发布是同一个调用里连着做的，中间没有别的发布；重开之后按 F_生效 重算 |
| ⑥ 链首无锚点认 txg + 1 | 站不住：不查实例 | 站不住（同） | 不中：候选表在链首判定之前已按实例过滤（`replay_journal` 的 `above`），两份都没有那条事实 |
| ⑦ 第一个文件版本释放 mkfs 树表 | 站不住：mkfs 与第一个文件版本不在同一个进程时，重建出来的分配器不认得那片单元、永远不释放 | 站不住（同） | 两份一致，**打中一个接缝**：今天 mkfs 之后没发过文件的池开不了可写挂载（上一轮 X2 ①），所以走不到；改法见第五节 |
| ⑧ 树表空拒开 | 站不住（错误信息混淆） | 站得住 | 记决策点（上一轮已记） |

## 三、云端正推（S1–S12、X10）

| 格 | 判 | 主 agent 核 |
|---|---|---|
| S1 | **不一致（窄化）**：候选集的 F 检查用最新根自己带的 `rollback_floor`，条款要的是 F_生效 | 核实（`mount.rs` 那一行），改法见第五节 |
| S2–S4、S6–S12 | 一致（S6 / S7 / S8 的简化 kb 已标预想） | 逐格对过附录原文与代码行号，判对 |
| S5 | 一致；「事务号 0 的记录也不施加」在 W > 0 时观测不到 | 核实 |
| X10 | 里程碑步 0 / 步 4 / 步 5 现状、登记表两行与第二条流那句、I-3.1 / I-2.1 读法与代码对得上；闭式复核不了 | 闭式由门禁 54 号实测（第五节） |

核查员：正推腿引 kb 条款几乎处处写的是小节标题行的行号（12 处），代码行号准确（30 处里 2 处差 1 行）。

## 四、云端攻方（X1、X2、X3、X4、X8）

| 格 | 报告 | 主 agent 核 | 处置 |
|---|---|---|---|
| X1 | **打中两处**：① 回退实例的根都读不出时最新根是被抛弃的根，候选集按那张旧表判、退到被抛弃的 (2, 6) 被接受；② 最新根的实例表单元两份都坏时目标 A 被 `InstanceTableMalformed` 拒、普通挂载也开不了 | 核实：① 是 C332（回退实例两个根都读不出时回退被撤销） 那一族，判别子当时观测不到、改代码修不了；② 同一个坏单元把两条路一起关掉，是射程 | 两条都记决策点（第六节），代码不动 |
| X2 | **打中**：B 的记录已提交、根没落盘时发起回退，P1 的记录（jsn = R_old 那条 + 1）盖掉 B 那条；崩在回退生效之前，恢复从施加 1 条变 0 条，「与没发起回退时相同」不成立（副本两个崩溃点，零故障） | 核实：核查员在自己的副本上复跑六条用例全过；条款 D23（journal 的角色与格式） 已定项 14 回退段逐字如报告所引 | **改法：P2**——新实例的第一条 jsn 接在环里最大的 jsn 之后，被抛弃的记录一条不盖（第五节 1）；P1 与 P2 是岔路（P2 把 X8 那一格放大：被抛弃的记录留在环里、靠候选集与实例表挡），代价数没有，标预想交用户 |
| X3 | **打中**：影子账只住内存、只在 `mount_rollback` 里算；回退之后普通重开一次隔离归零，68 个只被 C 引用的槽里 40 个可分配、接下来三次发布落在其中 28 个上 | 核实：`rebuilt_allocator` 此前不算隔离；核查员复跑那条用例得 56 项而报告贴 52 项（方向不变，来历没追） | **改法**：影子账在每次挂载的重建里算（按最新根实例表判被抛弃的根），且按 D23 主句的保守读法隔离被抛弃根账里还分配着的每一个槽（第五节 2） |
| X4 | **打中**：`rollback_high_water_of_root` 要「所选根的实例在它自己的表里有回退行」，回退行写的是 r_old 那一行、带这张表的根属于新实例——互斥，任何可达历史上取不到真；变异表那条必红的用例是直接喂 W 的 | 核实：副本上回退之后 11 条可读根全部 `None` | 第五条今天是死条款；活过来要回退行另有一处不依赖新实例根的落点，是格式级决定（第六节）；代码不动，C124（回退行与重放下界没有会红的检查） 按今天的实现还不清 |
| X8 | **打中**：P1 让根 (1, 4) 与 (2, 5) 一起失锚，对 (2, 5) 重放时无锚点规则认 txg + 1 那条、把被抛弃的实例 2 时间线三条整段重放回来 | 核实：随 P2 消失（记录槽不再被盖、锚点都在）；无锚点规则本身没变 | 随 X2 的改法处置；第六条口径落地时链首要收严，记决策点 |

## 五、处置（都已落地，`check.sh` 绿、命名纪律绿、门禁 59 号 44 条变异各红、52 号绿）

1. **P2**（`crates/singlefs-core/src/mount.rs` `mount_rollback`）：`next_counter = 环里最大 jsn + 1`；`RollbackRecordUnreadable` 删掉（R_old 自己那条读不出时与可写挂载同一条路顶上）。会红的用例 `rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation`（D 的 jsn 9，(1, 4)、(2, 5)、(2, 8) 三条记录原样在环里）；变异「回退之后 jsn 接 R_old 那条之后（P1）」它红。原来为 P1 加的记录核对器「被后来的记录合法覆盖不算洞」撤回（P2 下没有合法覆盖，那条变异成了等价变异）。
2. **影子账每次挂载都算、保守读法**（`mount.rs` `rebuilt_allocator`、`allocator.rs` `isolate`）：可写挂载与回退共用重建路，被抛弃的根 = 按最新根实例表有行且 txg > T 的根（回退时再加 (txg, 实例) > R_old 的），它们账里还分配着（未释放）的每个槽都隔离，隔离位独立于分配位。用例同上（普通重开后 `isolated_slots_per_device` 36、下一次发布不落在被抛弃根的槽上），步 4 原用例的隔离数 34 → 36；变异「普通重开不隔离」「影子账把被抛弃根账里已释放的落点也隔离」各红。**与用户 2026-09-16 定的窄读法措辞「仍被有效根引用的槽不在其内」字面不一致（预想、偏离用户定案的措辞），alloc-basis 第二轮当一条臂判。** E 的数据单元随之落 50178（此前 50176）：mkfs 实例表那片被 B 的根引用、隔离着；mkfs 树表那 1 槽回收了、50179 从没分配过。
3. **候选集的 F 用 F_生效**（`mount_rollback`）：用例 `roots_below_a_floor_carried_by_only_one_device_remain_rollback_candidates`（改坏盘 1 的载体、F_生效 回到 0，txg 9 的根仍退得到）；变异「候选集的 F 用最新根自己的」它红。
4. **回收门槛 = max(F_生效, 环里最旧有效根)**（`mount::reclaim_floor`，重建与抬 F 两处）：单测 `reclaim_floor_takes_the_larger_of_the_effective_floor_and_the_oldest_valid_ring_root`；变异「回收门槛不看环里最旧有效根」它红。固定脚本走不到 txg 24，第一版根环 24 槽的那一格只有单测。
5. **重建分配器认得出还没被换下的第 0 版树表单元**（`allocator.rs` `rebuild_from_records`：分配代 0、跨度 1、未释放）：单测 `rebuild_from_records_remembers_the_genesis_tree_table_until_it_is_released`；变异各红。今天走不到（没发过文件的池开不了可写挂载），接缝先补上。
6. **写回**：`02-second-txn.md` 步 4 / 步 5 现状、`second-txn-layout.md` 按上面五条改；决策点见第六节。
7. **再攻一轮**（`.claude/rules/three-way-inference.md`「一次打穿不算数」）：第二轮攻击面 = 这五处改法（都被攻过零轮）+ 第六节里代码没动的格；alloc-basis 第二轮在它之后开（另一个会话与用户定）。

## 六、交用户的决策点（都标预想）

| 项 | 今天的取法 | 谁打的 |
|---|---|---|
| C340 回退之后记录链从哪条之后接 | P2（环里最大 + 1）；P1 在「B 的记录已提交、根没落盘」窗口里盖掉已提交的记录 | 云端攻方 X2 |
| 影子账的读法 | 保守读法（被抛弃根账里还分配着的每个槽都隔离，含 R_old 也引用的）；与用户定的窄读法措辞不一致 | alloc-basis 账 6、本地辩方③、云端攻方 X3 |
| 前缀第五条（回退行的 W）怎么才能活 | 回退行只挂在新实例的根下面，所选根是旧实例的根时看不见它；要么另给回退行一处不依赖新实例根的落点，要么条款改成读「环里最新可读根的表」 | 云端攻方 X4、C124 |
| 回退实例的根都读不出时 | 最新根是被抛弃的根、候选集按旧表判，退到被抛弃根被接受；判别子观测不到 | 云端攻方 X1 ①、C332 |
| 最新根的实例表单元两份都坏 | 回退与普通挂载都开不了 | 云端攻方 X1 ② |
| 「非空持久有效根」怎么认 | 环里有它自己那条记录且事务号非 0；撕掉的记录让上限偏低（保守） | 本地攻方 2a、本地辩方④ |
| 两个 F | walk 与那条根的记账行用根自己带的 F，回退候选集与回收用 F_生效；口径没写进条款 | 本地攻方 2c / 3a，alloc-basis 账 5 |
| 树表空拒开的错误信息 | `NoPublishedVersion` 同时罩「只 mkfs 的池」与「退到没有文件版本的根」 | 本地辩方⑧ |

## 七、核查员（`m2-step45-code-r1-verifier-output.md`，268 行）

- 判别力自证：把一条正确引用的行号 +1 判 ✗。
- Opus 六条用例在核查员副本上全过；X3 那条用例的清单报告贴 52 项、复跑 56 项（多出的四项都是 C 的数据单元两盘各两槽），方向不变，来历没追——第五节的改法之后那份清单不再成立。
- 核了约 95 处：✓ 66、✗ 16、核不动 10。Sonnet 报告 kb 引用几乎处处写的是小节标题行号（12 处），代码行号准确；本地攻方提示的事实段逐条精确；本地辩方提示第 7 题一句「注释明写」不成立（行为属实）。
