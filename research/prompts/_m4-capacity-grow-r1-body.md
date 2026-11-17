# 盘变大时挂载怎么把新容量记下来（m4-capacity-grow-r1）

日期：2026-11-17。

## 问题

容量比对那一题三轮判完（`research/prompts/m4-capacity-r3-main-verification.md`）：每块盘的系统配置带逐盘容量表（按设备号排、每项 8 字节整槽数），读数出口全取记录值，跨盘跨槽表不等整池拒，变小可写拒、只读放行并点名。用户另定（弹窗，`records/2026-11-17-里程碑四收口续做.md`）：盘变大时挂载就把新容量记下来，保留今天变大重挂就能用上的行为。

「记」这一步在那一题前两轮被攻方打中过四种窗口（`research/prompts/m4-capacity-r1-opus-output.md` 的 G1–G4：记得晚于分配、只写一槽被轮换退回、压在没定的 C974 上、每次写都抄探测值）。这一轮要给出一个躲得开这四种窗口的写法：

1. 哪一步写：可写挂载做成之前（分配器建图之前）、取号那几次写里、还是单独一次。只读挂载不写。
2. 写几块盘几个槽、次序；写到一半崩溃，各盘各槽的表不相等——而那一题定了「跨盘跨槽表不等整池拒」，这两句怎么并存：读者在什么条件下认「正在改表」的中间态、取哪一份。
3. 新表生效之前谁都不许按新容量分配：次序怎么保证，崩溃在哪一步之后才可能有落点越过旧容量。
4. 变大跨过分配记录树根层级边界时怎么办（今天可写挂载会被层级错拒掉，两条腿读码推的，没有用例验过；两盘等大时边界约每盘 175.89 GiB，单盘 1→2 那道边界约 1.04 GiB）：拒记、还是连树一起重建。
5. 用户把盘变大、挂载记下之后又把盘缩回去：照「变小可写拒、只读放行并点名」处置，有没有落点越过缩回后的盘尾而读者看不出的历史。
6. 与 D2（RAID 条带策略） 已定项 3「几何再求值走显式事务」的关系：挂载时记算不算那条事务的一个实例，条款要不要补一句。
7. 容量表归 D22（单元原子性怎么合成） 已定项 26 的哪一档。

## 实现今天的样子

主 agent 的观测，方案按 `crates/` 今天的实现来谈：

- 容量今天按块设备报的字节数现算：`crates/singlefs-core/src/allocator.rs:520`–`:529` 的 `absolute_slot_count_of_device`「这是这个量唯一的一处定义」，单元区槽数与分配记录树根层级都从它算。
- 写者拼系统配置时设备数按盘表现数：`crates/singlefs-core/src/transaction.rs:723` `device_count: u32::try_from(self.devices.len())`。
- 读者跨盘只比 fsid 与设备数：`crates/singlefs-core/src/recovery.rs:1899`–`:1903`，不等报 `SystemConfigurationsDisagree`。
- 系统配置每盘两槽轮换（`crates/singlefs-core/src/system_configuration.rs` 第 2 行文件头）；轮换与取号各在哪几步写系统配置、写的次序，主 agent 没现读，腿去读 `transaction.rs`、`mount.rs`。
- 崩溃点重放对每次系统配置槽写取撕裂第三态：任务 18800 两条单测（`crash::tests::the_state_plan_with_torn_in_place_overwrites_hands_out_every_landing_combination_once`、`crash::tests::the_torn_image_keeps_the_first_half_of_the_differing_bytes_new_and_the_rest_old`）绿。
- 逐盘容量表本身还没有代码（`grep -rn 'capacity_table\|容量表' crates/singlefs-core/src | wc -l` 写正文时是 0）。

## 压着这件事的条款

- `.claude/kb/decisions/12-目标介质.md`「#### 已定项 4：挂载时比对哪几条」
- `.claude/kb/decisions/22-单元原子性怎么合成.md`「#### 已定项 8：固定结构的放置」「#### 已定项 16：系统配置槽轮换规则；根环区域公式的地址空间与区域内槽位」「#### 已定项 26：系统配置的三档可改性」
- `.claude/kb/decisions/02-RAID条带策略.md`「#### 已定项 3：设备集合变化走显式「几何再求值」事务」「#### 已定项 4：几何再求值事务的三样形态」
- `.claude/kb/checks-owed.md` 的 C974、C463、C1040 三行

## 判据格

每格答「怎么做、依据（条款或实现的 文件:行）、推翻它的观测」：

1. 写的时点。
2. 写的范围、次序与中间态读者怎么认。
3. 新容量生效与分配的先后。
4. 跨根层级边界。
5. 变大再缩回。
6. 与 D2 已定项 3、4 的关系与改句。
7. D22 已定项 26 归档。

## 分工

正推腿逐格给一个完整写法；攻方腿攻第一轮的面：过度外推与越位 / 内部矛盾与循环依赖 / 哪一条最脆弱——重点是第 2 格（中间态与「不等整池拒」并存）与第 3 格（生效先后）。

## 交用户的

只交判不出、要拍板的岔路。
