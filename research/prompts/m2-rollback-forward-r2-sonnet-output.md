# m2-rollback-forward-r2 云端辩方腿（Sonnet）报告

轮名 m2-rollback-forward-r2（设计轮第二轮）。任务：G6 + 复核第一轮判决 F3（`research/prompts/m2-rollback-forward-r1-main-verification.md`）。替第一轮被判「可以删」但被打穿的一方（影子账）与被判「还要」的一方（`abandoned_by_table`、`OnAbandonedTimeline`、I-7.4）各自复核；查见证删掉之后 H12 会不会变坏；查判决里哪一句够不着它引的证据。

**没有写模型、没有跑代码、没有建原型**：本腿全部结论来自对冻结副本 `/tmp/claude-1000/m2-rollback-forward-r2/tree/crates/` 的现读（grep + 逐行读）与对第一轮三条腿报告、判决原文的核对，不新增测试或崩溃点扫描，因此不建 `research/prompts/m2-rollback-forward-r2-sonnet-model/`（该目录只在写了模型时才建，见共用约束）。

## 一、G6 问① H6 那段历史是不是真的只有影子账挡得住？有没有更窄形态（只在崩溃恢复落到旧根的那次挂载里隔离）？

**结论：不是更窄能挡住，是「按挂载数隔离」这条路本身就挡不住——需要跨越「与故障恢复无关的下一次挂载」才会中招，任何绑定挂载次数的窄化都会被同一手法用更长的故障序列绕开。**

H6 的历史（`research/prompts/m2-rollback-forward-r1-opus-output.md:175-182`，t7c）分三段：
1. 实例 1：A(3) B(4) C(5)。
2. 重开：C 的根槽与数据单元**暂时**读不出 ⇒ 落回 B，实例 2 写行 (1,4,W)（这是「崩溃恢复落到旧根」的那一次挂载——若窄化成「只在这次挂载里隔离」，隔离窗口应当在这里开）。
3. 撤故障（C 又读得出）；**再重开（实例 3）**——这是下一次挂载，本身是一次干净挂载（全部根可读，不需要任何回退判定），如果隔离窗口只挂在第 2 步那次挂载上，在第 3 步这个窗口已经关闭。第 3 步里覆盖写 D、E：关影子账时 D 落在 C 的数据槽 50184；开影子账时落在 50186（`m2-rollback-forward-r1-opus-output.md:181`）。
4. 之后把比 C 新的 7 条根改坏，恢复落回 C，读 C 引用的 50184——已被 D 复用，读失败（`m2-rollback-forward-r1-opus-output.md:184`）。

真正需要防护的读取（第 4 步）发生在**第 3 步那次挂载写入之后**，而第 3 步本身不是「崩溃恢复落到旧根」的那次挂载——它是故障已经撤销、一切正常的下一次挂载。「只在崩溃恢复落到旧根的那次挂载里隔离」这一窄化，窗口会在第 2 步结束时关闭，第 3 步的分配器重建（`mount.rs:742` `rebuilt_allocator`，被 `mount_writable_with_space_admission`——`mount.rs:2367`——在每次可写挂载时调用）不会再算 C 为「本次要保护」的对象，于是直接退化成关影子账那一臂，`t7c` already 证明这一臂会把 50184 发给 D。**把隔离窗口从「每次挂载都重算」缩到「只在那一次挂载」，在这段历史上就是把已经打穿的那一臂又跑一遍**，不会挡住 H6。

冻结副本今天的实现证实隔离范围本来就不是按「一次挂载」记账，而是每次可写挂载重新从环上现读现算：`mount.rs:604`「候选根读不出就当它什么都不豁免（隔离只会多不会少）」、`mount.rs:606`「清只在被抛弃的根离开根环的那一次发布里清（`PoolAllocator::record_root_written_by_this_process`，按交回的每条根引用的落点）」——即隔离持续到被抛弃的根**离开根环**（被同一区域下一次发布盖掉）或 F 抬过它为止，不是持续到"下一次挂载"或"下一次干净挂载"为止。这条边界已经是能收的最窄边界：把它继续收紧到"只一次挂载"，H6 的第 3、4 步就是现成的反例；再往下收（比如"隔离 N 次挂载"，N 取任意有限值）同样会被"故障撤销之后先垫 N 次以上无害挂载、再写重用块"这类更长的历史绕开——因为病根是"被抛弃根什么时候会被选中"这件事本身不由经过了几次挂载决定，只由它是否还在根环里、是否还 ≥ F 决定。

**推翻条件**：如果能证明"被抛弃根离开根环之前必然经过的挂载数"存在一个有限、可枚举的上界（比如根环只有 R×S 个槽，槽满之前不可能再被同区域覆盖），那么"隔离 N 次挂载"这类窄化在 N 取够大时可能等价于今天的实现——但那时它已经不比现在的实现窄，只是绕着同一个边界重新表达了一遍；本腿没有去验这个等价关系是否成立，留给愿意做这个化简的人。

## 二、G6 问② `abandoned_by_table`、`OnAbandonedTimeline`、I-7.4 在回退改成向前发布之后是不是真的还要

三者不能一概而论，拆开看：

### `abandoned_by_table`（`mount.rs:576`）：还要，证据比第一轮判决引的更直接

`abandoned_by_table` 在冻结副本里有四处调用，只有一处与「管理员回退」路径相关：
- `mount.rs:790`（`rebuilt_allocator` 内 `is_abandoned` 闭包的一支，`mount.rs:785-791`）——**每一次可写挂载都会跑**（`mount_writable_with_space_admission` 在 `mount.rs:2367`，`mount_rollback_with_space_admission` 在 `mount.rs:2491`，两者都调 `rebuilt_allocator`），与是否发生过管理员回退无关；
- `mount.rs:991`（`rollback_floor_ceiling` 算抬 F 上限用的「有效」）；
- `mount.rs:1155`、`mount.rs:1171`（同一函数体内，向 `isolate_slots_referenced_only_by_abandoned_roots` 传 `is_abandoned` 闭包的另一处调用点，同样在可写挂载路径上）；
- 只有候选集判定那一支（`mount.rs:2537-2540`，在 `mount_rollback_with_space_admission` 内）是专属管理员回退路径的，这一支随 `mount_rollback_with_space_admission` 整个函数一起被这一轮删掉（正文第一节「删掉：……挂载时的回退路径（`mount_rollback`、`mount_rollback_with_space_admission`）」）。

H6（`t7c`）用的正是 `mount.rs:790` 那一支：两臂（开/关影子账）都不改 `abandoned_by_table` 本身，只改 `ShadowLedger::On`/`Off`（`mount.rs:801-811`）——`abandoned_by_table` 若被删掉，`is_abandoned` 闭包对这一支恒假，效果等同于关掉影子账，H6 已经证明这一臂会把 C 的数据槽发给 D。**`abandoned_by_table` 的必要性由 H6 证实，不需要另外造史**——它是影子账赖以工作的输入，删了 `abandoned_by_table` 等于删了影子账的候选源。

### `OnAbandonedTimeline`（`mount.rs:2546`）：判决写「还要」，但它的唯一落点正是这一轮要删掉的函数——这是判决与本轮方案之间的一处缺口，不是判决本身错

`OnAbandonedTimeline`（`RollbackCandidateExclusion` 的一个变体，定义于 `mount.rs:257`）在整个冻结副本里只出现一次调用点：`mount.rs:2546`，位于 `mount_rollback_with_space_admission`（`mount.rs:2491-2648`）内的候选集校验块（`mount.rs:2537-2548`）。这一整块——连同它所在的整个函数——正是正文第一节点名要删掉的（「挂载时的回退路径（`mount_rollback`、`mount_rollback_with_space_admission`）」）。也就是说：**判「`OnAbandonedTimeline` 还要」的时候，它今天唯一的家已经在同一份正文里被判了拆除**。这不是说这条判断错了（候选集要不要排除"被实例表判仍抛弃"的根，这个安全属性大概率在新函数里仍然需要），而是正文第一节第 17 行「留着……`OnAbandonedTimeline`」与正文第一节 G1 那条列出的新函数要留的检查（复活、释放、水位、"同槽同分配代"核）之间有一处没接上：**新函数的规格清单里没有列"按实例表判仍然有效"这一条候选排除**，只列了"同槽同分配代"这一道核。

这处缺口是否要紧，H7 给出的证据是含糊的，不是清楚的"还要"：`t8b`（H6 那段历史第 3 步，C 被实例表抛弃又读得出，向前回退到 C）里，删掉 `OnAbandonedTimeline` 这一条候选排除之后，原型自带的"同槽同分配代"核（`TargetUnitNotProtected`）顶上去拒绝了，没有观察到读错（`未打穿到读错`，见下一节引文）；**但"两道都删"的那一臂 Opus 自己没造**，只推："新根引用一个账里空闲的槽，下一次分配就复用——零故障"（`m2-rollback-forward-r1-opus-output.md:190`）——这句推的话面上甚至像是在说"两道都删也安全"，与判决"还要"的方向相反，但它明确标了"没造"，不是量出来的。**如果这一轮的新函数（G1 那个"挂着的时候"回退）真的只留"同槽同分配代"一道核、不留"按实例表判仍然有效"这一条，等于恰好复刻了 H7 里那个从未被跑过的"两道都删"的臂**——这件事既没有被第一轮的三条腿证实安全，也没有被证伪，是一个真空。

### I-7.4：还要，依据不靠 H6/H7，靠不变量自己的定义

I-7.4（`invariants.md:54`，冻结副本 kb 快照同一行）判的对象是"回退候选集里每一个根（按实例表判仍然有效 ∧ txg ≥ F_生效）所引用的块……均未被重新分配"——这条判据本身就把"按实例表判仍然有效"焊进了它判定的对象集合，checker 端（`walk::check_pool_image`）落地在 `crates/singlefs-checker`，与 `mount_rollback` 是否存在无关：**只要 `abandoned_by_table` 这个"仍然有效"的判法在 checker 里还在用（H6 已证实它在挂载端还要），I-7.4 的依据就自动跟着还在**，不需要额外援引 H6 或 H7。第一轮判决把 I-7.4 判「还要，依据收窄到崩溃恢复的候选集」（`m2-rollback-forward-r1-main-verification.md:45`）——这句话没有被本轮攻到，站得住。

## 三、G6 问③ 见证删掉之后，H12 那一形会不会变坏

**结论：不会变坏——H12 今天已经中、见证今天已经在场却完全没起作用，删掉见证是删掉一件对这一形从没出过力的东西。**

先查见证在 `choose_root` 里到底管什么：`recovery.rs:669` 起的 `choose_root` 遍历根环，唯一的排除条件是 `rollback_witness.abandons(candidate.instance, candidate.checkpoint_txg)`（`recovery.rs:681`：`if rollback_witness.abandons(candidate.instance, candidate.checkpoint_txg) {`），此外只挑 `(checkpoint_txg, instance)` 最大且自证过的根——**`choose_root` 本身完全不查 `abandoned_by_table`**。也就是说，`choose_root` 会不会落到某条根上，只由"读不读得出/自证不自证"与"见证表有没有点名"两件事决定，与实例表判它是否"被抛弃"无关。

H12（`t7`，`m2-rollback-forward-r1-opus-output.md:192-194`）里，实例 2 的四条根重开时全部暂时读不出，恢复落到更早的根；之后实例 3 复用了它们的槽；撤故障、把实例 3 的根全改坏，`choose_root` 落回实例 2 那几条根——读失败（`UnitUnreadable { slot: SlotNumber(50304) }`）。**这整段历史里没有任何一次管理员发起的回退（`mount_rollback`）**，见证表因此自始至终是空的（`mount.rs:2620-2625` 是唯一置 `PreviousInstanceRow { is_rollback: true, .. }` 的地方，`t7` 没有走过它）。见证表本身在**每一次可写挂载**（`establish_instance`，`mount.rs:2055`，`mount_writable_with_space_admission` 与 `mount_rollback_with_space_admission` 都调它）都会走一遍维护逻辑（`rollback_witness_tables_of_this_mount`，`mount.rs:1896`，调用点在 `mount.rs:2163`）——搬运已有条目、按删除规则清；但**新增条目**只在 `previous_row.is_rollback` 为真时发生（`this_rollback = previous_row.is_rollback.then_some(...)`，`mount.rs:1925`），而 `is_rollback: true` 只在 `mount_rollback_with_space_admission` 里被置（`mount.rs:2624`）。`t7` 全程没有一次管理员回退，见证表因此从未获得过一条新条目，恒空——`choose_root` 的排除条件恒假，开着见证与删掉见证在这一形上行为逐字相同，Opus 的原话是"影子账开 / 关两臂逐字相同，今天的冻结副本同样中"（`m2-rollback-forward-r1-opus-output.md:194`，`m2-rollback-forward-r1-main-verification.md:54` 同一句判决重申）。

**唯一需要留意的旁支**：`mount.rs:787` 那个 `rebuilt_allocator` 内部的 `is_abandoned` 闭包同时 OR 了 `rollback_witness.abandons(...)`（`mount.rs:787`）与 `abandoned_by_table(...)`（`mount.rs:790`）——也就是说见证不止在 `choose_root` 起作用，也在**普通可写挂载**的影子账 `is_abandoned` 判定里起作用。但这条 OR 支路要起效，前提是见证表里确实写了条目，而写见证条目的唯一入口是 `mount_rollback_with_space_admission`（`mount.rs:2620` 起）——这一形（H12/t7）没有走过那条路径，这个 OR 支路在这段历史上从头到尾恒假，不影响上面的结论。这一轮把 `mount_rollback_with_space_admission` 整个删掉之后，这条 OR 支路本身也没有了输入来源，见证的这一半作用随管理员回退一起自然消亡，不是单独需要再判一次的缺口。

**推翻条件**：如果 G1 那个"挂着的时候"回退新函数最终仍然需要写某种"这条根被这次回退绕过了"的持久标记（哪怕不叫见证表），且这个标记被喂进 `is_abandoned` 或 `choose_root` 的候选判定，那么"见证删掉不影响 H12"这句话仍然成立（H12 与管理员回退无关），但"见证整体可以删"就要重新核——那时候新写的标记本质上是见证的另一种实现，不是真的删掉了这件事。这属于 G1 的范围，不归这一格判。

## 四、判决里哪一句够不着它引的证据

**`m2-rollback-forward-r1-main-verification.md:43`**（F3 表格「候选集按实例表判『被抛弃』（`abandoned_by_table`）与 `OnAbandonedTimeline`」一行，判「**还要**」，依据写「崩溃恢复落到旧根、实例表抛弃较新的根照样发生，与管理员回退无关（H7）」）够不着它引的 H7：

- H7（`m2-rollback-forward-r1-opus-output.md:27`）自己的结论是「未打穿到读错：删掉之后原型自带的『同槽同分配代』核拒了；两道都删的那一格没造」——这是「攻击没有找到反例」，不是「找到了证明还要的证据」；三方规则里「没打中」的门槛是至少两次独立观测才能采信（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」），这里连一次完整的「两道都删」都没跑,只跑了单删一道。
- H7 测的历史（`t8b`）本身是**一次管理员发起的、向前回退到 C 的动作**（`m2-rollback-forward-r1-opus-output.md:190`：「`t8b`：H6 那段历史走到第 3 步……向前回退到 C」）——这恰恰是「与管理员回退**有关**」的场景，与判决第 43 行写的理由「与管理员回退无关」字面相反。
- 「与管理员回退无关，崩溃恢复落到旧根、实例表抛弃较新的根照样发生」这句话本身是真的，但它描述的是 **H6** 的场景（`m2-rollback-forward-r1-main-verification.md:44`：「H6：一段没有任何管理员回退的历史（崩溃恢复因暂时读错落到旧根）……」），不是 H7 的。第 43 行把 H6 的理由文字安在了 H7 的引用括号里。

**这处引用错位不改变最终判定本身**：`abandoned_by_table` 的「还要」由 H6 独立、干净地证实（见本报告第二节——H6 用的 `mount.rs:790` 那条调用路径，`abandoned_by_table` 是影子账的输入源，H6 已经跑出「关掉这条路径等价的效果」会读错），不需要借 H7。但 `OnAbandonedTimeline` 这半句「还要」目前没有任何一条腿的实测正面支持它——H7 唯一测过的、削弱它的那次尝试是「未打穿到读错」（说明去掉它可能是安全的，因为有别的核顶上），「两道都删」这个更贴近本轮真实方案（`OnAbandonedTimeline` 唯一的代码位置随 `mount_rollback_with_space_admission` 一起被删，见本报告第二节）的组合从未被跑过。**这是本轮 G6 新查到的、判决原文没写清楚的一处证据缺口，不是主 agent 已经权衡过、只是没有明写的省略。**

**推翻条件**：把 H7 的「两道都删」那一臂实际跑一遍（`t8b` 历史 + 同时不做候选集实例表校验、不做 `OnAbandonedTimeline` 排除、也不做「同槽同分配代」核），如果读对，说明 `OnAbandonedTimeline` 真的可以随 `mount_rollback` 一起删、只留"同槽同分配代"一道核就够；如果读错，说明新函数的候选集规格里必须显式补回等价的"按实例表判仍然有效"排除，正文第一节 G1 那条清单要加这一句。这件事没有归任何一条腿的分工表（Opus 这一轮攻 G1-G5，本地攻方只算算术），建议交下一轮或单独记欠账。

## 五、复核表：判决条目 / 辩方判定 / 证据

| 判决条目（`m2-rollback-forward-r1-main-verification.md`） | 辩方判定 | 证据 |
|---|---|---|
| F3：影子账「**还要**（正推腿判『可以删』，被打穿）」（第 44 行，H6） | **站得住**，且窄化到「只在崩溃恢复落到旧根那一次挂载里隔离」同样打不住 | 本报告第一节；`mount.rs:604`「隔离只会多不会少」、`mount.rs:606`「清只在被抛弃的根离开根环的那一次发布里清」——今天的实现已经是能收的最窄边界，H6 的第 3、4 步落在"下一次挂载"而非"落回旧根那一次挂载"，任何按挂载次数记账的窄化都会被同一手法绕开 |
| F3：`abandoned_by_table` 「**还要**」（第 43 行半句） | **站得住**，但判决引的证据（H7）不是最直接的一份——H6 才是 | 本报告第二节；`abandoned_by_table` 在 `mount.rs:790` 是影子账 `is_abandoned` 的输入源，H6 关影子账那一臂等价于让这条谓词失效，H6 已实测读错 |
| F3：`OnAbandonedTimeline`「**还要**」（第 43 行另半句） | **够不着**——引的 H7 结论是「未打穿到读错」（不是证实还要），H7 测的是与本轮判决理由字面相反的「管理员回退到 C」场景，且它在冻结副本里唯一的代码位置（`mount.rs:2546`）随本轮要删的 `mount_rollback_with_space_admission`（`mount.rs:2491-2648`）一起被删，正文 G1 给新函数列的检查清单没有列出等价替代 | 本报告第二、四节；`m2-rollback-forward-r1-opus-output.md:27`、`:190` |
| F3：I-7.4「还要，依据收窄到崩溃恢复的候选集」（第 45 行） | **站得住** | 本报告第二节；`invariants.md:54` 的判据文字本身把「按实例表判仍然有效」焊进判定对象集合，不依赖 H6/H7 单独证实 |
| 越格 H12（第 52-54 行，「立欠账，不挡这一轮」） | **站得住，且见证删掉之后不会变坏** | 本报告第三节；`t7` 全程没有一次管理员回退，见证表恒空，`choose_root`（`recovery.rs:669`、排除条件在 `recovery.rs:681`）与影子账 `is_abandoned`（`mount.rs:787` 的 OR 支路）里"见证"那一半在这形上从未被触发过 |
| F3：回退见证整套「可以删」（第 41 行） | **不在本轮辩护范围内，未复核**（判决本身没有被判「出局」，是被判定「可以删」且没打穿，不是本轮 G6 指定要辩护的一方） | — |
| F3：实例表回退行与截断「可以删（推的）」（第 42 行） | **不在本轮辩护范围内，未复核**——正文第五节写明「第一轮已攻过的挂载时那一形不再攻」，回退行删除本身归 G5，这一轮分工给 Opus | — |

**同一形状在别的臂上是不是同样成立**：H6 攻的是「影子账可以删」这一条判断本身，场景（`t7c`）从头到尾没有触发任何管理员回退，与第一轮 F1 判的候选 A（挂载时做）还是候选 B（挂着时做，这一轮取用）无关——不管回退实现成哪一种形态，`rebuilt_allocator` 在每次可写挂载都跑、影子账都要保护崩溃恢复产生的被抛弃根，这一点在两种候选下逐字相同。**H6 因此不是「打中了某一个特定候选、放过另一个」，是打中了两个候选共有的底层假设**，这本轮 G1 无论怎么设计"挂着的时候回退"都要认下。

## 没做什么

- 没写模型、没建原型、没跑任何测试或崩溃点扫描：本腿全部结论来自对冻结副本源码的现读（grep + 逐行读）与对第一轮三腿报告、判决原文的比对，没有新造历史，因此没有 `research/prompts/m2-rollback-forward-r2-sonnet-model/` 目录。
- 不判 G1-G5（`m2-rollback-forward-r2-opus-*`、本地攻方分工范围），也没有读它们的产出（禁读清单）。
- 不判正文 G1 那份"挂着的时候回退"规格本身对不对、代价多少——第四节末尾指出的"候选集清单缺一条排除"是写给主 agent 的一处待核提醒，不是这一格自己的判定。
- 不判 F2（抬 F、B1/B2、C419 修法）、F4（代价），这两项不在辩方分工（G6）范围内。
- 不撤销或维持第一轮判决——只交复核结果，判决是否改写由主 agent 定。
- 读文件：`m2-rollback-forward-r1-{sonnet,opus,verifier,local-attack-output-s1,local-attack-output-s2}` 全份、`m2-rollback-forward-r1-main-verification.md` 全份、`_m2-rollback-forward-r2-background.md`（第 1-250 行，材料员补注与小节清单前段，因单文件读取上限被截断，后续小节清单未读完——但本腿结论没有依赖背景材料第 250 行之后的内容，全部依据都直接查了冻结副本源码或 r1 三腿报告原文）、`_m2-rollback-forward-r2-body.md` 全份、冻结副本 `mount.rs`、`recovery.rs` 相关函数与行号。
- 报告落盘路径：`research/prompts/m2-rollback-forward-r2-sonnet-output.md`（本文件，分段写完，共 93 行）。
