# m3-close-b19-e165-r1：C650 剩下那一族造不造得出（2026-11-05）

## 一、题面

C650（择根与重放没有合判）还欠一句：「合法历史上会让三种改坏择根序红的这一族状态」。三种改坏择根序是 `crates/mutations.tsv` 里的三条择根核变体（E165（恢复重放有界穷举与变异杀伤界） 第 3 次重跑起的登记名）：

| 登记名 | 改的是 |
|---|---|
| v1823 择根核比较不看实例代号 | txg 平局时停在读根环次序里前面那条 |
| v1825 择根核 txg 平局时实例代号低者赢 | 平局取低 |
| v1826 择根核以实例代号为主、txg 只破平局 | 比较的主次序反过来 |

E165 已量到的（量的，实验页 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md`）：

- 第 3 次重跑第三段（合法历史 K、K9、KW 与对抗根环 KA）：三条在 36 个合法历史格上与基线逐格同，只在 KA 上择得不同。
- 第 4 次重跑第二段（带读故障的择根合判族，C331（择根倒挂压过已确认的写） 那一形）：三条在四个合法域 410 个子格与 KG 80 个子格上 KJ1、KJ1T、KJ2 都是 0，v1823、v1825 只在 KA 上不同，v1826 在 KG 与 KA 上不同（产物 `git show 59ce5a86550d:research/results/e165-replay-bounded-enumeration-2026-10-03-r4-segment2-root-choice-faults-kill-bounds.out`）。
- 第 5 次重跑第五段 5-乙（系统配置读错取 R2 之后的带故障一族）：与第 4 次逐个相同；实验页那一段写「R2 那一族四个合法域里能让它们露出来的两种根环（同 txg 不同实例同在、实例代号更高而 txg 更低同在）每格 0」。
- 三条现由 Verus 必红世界（`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` 择根那一节）与 KA（第 7 次重跑起换成照落点公式摆根的 KL）罩着。

这些都是有界域上的读数。没有人论证过界外造不造得出，也没有人论证过造不出时这三条该怎么记（`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」：取样点不敏感要补取样点；在所有合法输入上同值才是等价变异）。这一轮判：两种根环在合法写者下造不造得出；造得出给最小历史；造不出时 C650 剩下那一句怎么关。

## 二、实现今天的样子（主 agent 的观测，2026-11-05）

方案按 `crates/` 今天的实现来谈。

- 择根：`crates/singlefs-core/src/recovery.rs:1888` `choose_root` 读三个区域全部槽，自证过、未被抛弃的摊成择新键交 `crates/singlefs-replay-prefix/src/lib.rs:3989` `choose_root_slot`；核逐槽往后，`checkpoint_txg` 严格大，或 txg 相同而实例代号严格大才换（读码）。
- 新实例第一次发布的 txg：`crates/singlefs-core/src/mount.rs:1402` `first_txg_of_new_instance` = max(根环里自证过的根的 txg, 环里自证通过的记录的 txg) + 1；根那一半是 `recovery.rs:1930` `highest_root_txg`，经 `visit_valid_roots` 只数此刻读得出、自证过的根，读坏的槽（`RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => continue` 一类，`recovery.rs:1809` 一带）不进 max（读码）。
- 取号：`crates/singlefs-core/src/transaction.rs:1019` `next_instance_generation_to_acquire` = max(每块盘两槽里自证过的系统配置的实例代号, 根环里自证过的根的实例代号) + 1（`transaction.rs:901` `highest_system_configuration_instance`、`recovery.rs:4935` `highest_root_instance`）。
- 判不出就拒：R2（`recovery.rs` 的 `system_configuration_witness_is_undecidable`：有系统配置槽设备报读错，且没有一块盘两槽都自证过）在取号写之前拒；C331（择根倒挂压过已确认的写） 由它挡（`.claude/kb/checks-owed.md` C331 那一行，已还清）。R2 看的是系统配置槽，不看根环槽（读码）。
- D16（发布语义） 已定项 1 对挂载那一刻读坏的根环槽：算抬 F 的上限时当作可能是一条非空有效根，判回退候选与算 F_生效 时当没有根（`.claude/kb/decisions/16-发布语义.md`「抬 F 的上限」「根槽这一次读坏」两行）；进程内重读一次见 `mount.rs` 里「进程内重读一个挂载那一刻读坏的根环槽」那段注释（读码）。

## 三、要回答的问题

两种根环（下称形甲、形乙），都指某一次恢复时同时自证过、未被抛弃、因而都进择根的两条根：

- **形甲**：两条根 `checkpoint_txg` 相同、实例代号不同。
- **形乙**：一条根实例代号更高而 `checkpoint_txg` 更低（另一条实例代号低而 txg 高）。

**Q1（正推）**：按 `crates/` 今天的写者（任意次挂载、崩溃、可写挂载末尾清扫、回退、多盘、环绕回、设备瞬时读错与读坏，都在 D16（发布语义）、D23（journal 的角色与格式）、R2 已定的口径内），形甲、形乙各造不造得出：

1. 造不出的，逐条写出挡住它的写者条款与代码（文件:行），并说清那条挡法在「挂载那一刻读坏的根环槽」下还成不成立——`highest_root_txg` 不数读坏的槽，而 R2 只看系统配置槽。
2. 造得出的，给最小历史（挂载、写、崩溃、读故障逐步），标明到哪一步盘上出现那一形、下一次恢复择根时三条变体各择哪一条、基线择哪一条、已确认的完整发布丢不丢。
3. 两形都造不出时，三条变体在全部合法状态上与基线同值，按 `.claude/rules/mutation-sampling.md` 记成等价变异要「把等价性写成一条测试留档」：给出这条测试该是什么（Verus 引理、池级断言、还是写者侧的不变量），并说 C650 剩下那一句该关成什么。

**Q2（攻）**：

1. 专攻读坏的根环槽：可写挂载那一刻 txg 最高的那条根所在的槽读坏（设备报读错、读得出不是全零而自证不过、读回全零三种各算一次），它那次发布的记录也读不到，R2 不触发——新实例第一次发布的 txg 会不会不大于那条根的 txg？之后那一槽读回来，盘上是不是形甲（txg 恰好相等）或形乙？给最小历史，或指出哪一条已有的挡法让它走不到（文件:行，或 D16、D23 已定项原文）。
2. 回退、清扫、被抛弃判（`recovery.rs:2230` 一带 `checkpoint_txg > row.selected_root_txg`）会不会把形甲、形乙里的一条在择根之前筛掉，使那一形在盘上出现但择根看不见？
3. 正推若判「造不出」，打它：三条变体在合法状态上真的同值，还是只在 E165 的界内同值？给界外的历史或说明界外为什么也同值。

**Q3（判据）**：C650 那一格写「改坏择根序这一族要红」。若 Q1、Q2 判两形造不出，这一判据要的输入在合法写者下不存在，属 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错」哪一种？判据该改成什么、C650 剩下那一句的去向写什么（关成无对象 / 改由 Verus 必红世界与 KL 罩 / 另立一笔）。

前提观测都在 E165 实验页与产物里；没量过的写「推的」。
