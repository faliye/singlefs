# m3-close-b19-code-r1 正推腿报告（Sonnet，正推）

日期：2026-10-29。轮名 m3-close-b19-code-r1。分到这条腿的格：T1–T6。本腿没有写模型，没有编译 Rust、没有跑任何测试（派发写明「重型测试：不跑」，定义也不让本腿编译）：
下面所有「代码做什么」都是读码推的，每一处引文与行号都在被引文件里现查过；谈行为的判定都带「什么现象会推翻它」。

## 读法与复核

- 开工快照：背景材料里的 `m3-close-b19-code-r1-snapshot/` 只有两份 sha256 清单（crates 10 份、kb 若干份）。本腿对着此刻主工作区复核，命令与结果如下，
  `sha256sum -c research/prompts/m3-close-b19-code-r1-snapshot/crates-sha256.txt` 十行全是 OK（`admission.rs`、`allocator.rs`、`instance_table_sweep.rs`、`mounted_session.rs`、`mount.rs`、`root_record.rs`、`walk.rs`、`crash_judge_checker_scan.wgsl`、`02-根环.md`、`05-空间记账.md`）；
  `sha256sum -c research/prompts/m3-close-b19-code-r1-snapshot/kb-sha256.txt` 过滤掉 OK 之后没有剩行。所以下面引的行号 = 快照里的行号。
- 快照清单之外的文件（`transaction.rs`、`recovery.rs`、`root_ring.rs`、`make_filesystem.rs`、`instance_table.rs`、`crash_judge_checker.wgsl`、`crates/mutations.tsv`）本腿在主工作区现读，没有快照可比，引到时标「快照外」。
- 引 kb 条款：写 kb 文件名加那份文件自己的行号。kb 的行很长（一行几千字），引文限 400 字以内，只抄压着这一格的那几句，行号指向整行；没有摘改一个字。
- 「编号（简称）」：C773（读坏与全零根槽的回收门槛钉只在内存里）、C724（准入抬 F 上限里同名拒因不在单子上）、C928（冻结着时整理照发 discard）、C874（补写那一支进不了崩溃测试、运行时看不出）、C929（补写盘回卷之后不再过补写闸）、C690（清扫触发阈值的默认值没定）。

## T1　C773 写者带（钉值, 槽）到写根

### 压着的条款

- 「采乙：第 0 代根所在槽（槽号 0）的钉值取 0，p 照存公式槽 `3·槽 + 区域`；写者把（钉值, 槽）一起往下带到写根那一刻、平局按（钉值, 槽）破」（`research/prompts/m3-close-b19-core-r1-main-verification.md:16`）
- 「钉也落盘：根记录带这次发布用的回收门槛 t 与钉槽 p，写者取 t = max(这条根自己带的 F, 分配器记这条根时的回收门槛)；钉着的槽不得住着本实例写的根」（`.claude/kb/decisions/16-发布语义.md:39`）
- 欠账行里登记的写者疑点：「第一次可写挂载崩在那次根槽写之前、第 0 代根留在区域 1 的槽 0 而那一槽读坏时，钉在 1 而不是 0，后果没量」（`.claude/kb/checks-owed.md:481`）
- D16（发布语义） 已定项 1 里「槽号 0 的钉值是第 0 代根的 0」那一句，判决写「随实现那一批写回」；本腿在 `.claude/kb/decisions/16-发布语义.md` 与 `.claude/kb/decisions/23-journal的角色与格式.md` 里 `grep -nF '槽号 0'` 零命中：条款还没写回。

### 代码做了什么（读码）

1. 钉值：`txgs_ring_slots_bad_at_mount_can_hold` 对每个读坏槽的公式最旧 txg 再过一道 `oldest_txg_a_ring_slot_bad_at_mount_can_have_held`（`crates/singlefs-core/src/mount.rs:2976`）；
   那一道对「mkfs 种第 0 代根的槽」取 `GENESIS_CHECKPOINT_TXG`（`crates/singlefs-core/src/mount.rs:2997`、`crates/singlefs-core/src/mount.rs:3000`）。
   「mkfs 种第 0 代根的槽」是三个区域的槽 0：「(0..ROOT_RING_REGIONS).map(|region| RootRingSlot { region, slot: 0 })」（`crates/singlefs-core/src/make_filesystem.rs:826`，快照外）。
2. 三形都过这一道：读故障（`reader.read` 返回 `None`）归 `BadAndNotAllZero`（`crates/singlefs-core/src/mount.rs:2798`），读得出而自证不过也归它；读回全零归 `AllZeroNeverWritten`（`crates/singlefs-core/src/mount.rs:2801`）；
   重建分配器时两类各调一次 `txgs_ring_slots_bad_at_mount_can_hold`（`crates/singlefs-core/src/mount.rs:3379`、`crates/singlefs-core/src/mount.rs:3386`），所以读坏、读回全零、读故障的槽 0 钉值都是 0；
   公式落不到的槽（年轻的环）不交回、不钉，由函数注释写明理由（第 0 代根此时还在区域 0 槽 0）。
3. 带（钉值, 槽）：记根时把每个钉着的槽配上钉值，取 `min_by_key`：「.min_by_key(|pin| (pin.pinned_txg, pin.ring_slot));」（`crates/singlefs-core/src/allocator.rs:2793`）。
   `RootRingSlot` 的派生次序是区域在前、槽号在后：「#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]」（`crates/singlefs-core/src/root_ring.rs:12`，快照外），所以平局取区域号最小的槽。
4. p 的编码：「ROOT_RING_REGIONS * self.ring_slot.slot + self.ring_slot.region」（`crates/singlefs-core/src/allocator.rs:1813`），落点公式是「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」（`crates/singlefs-core/src/root_ring.rs:172`，快照外）与「slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),」（`crates/singlefs-core/src/root_ring.rs:173`，快照外）：
   第一圈里 txg = 3·槽 + 区域，所以非槽 0 的槽 p 等于它的公式最旧 txg。
5. 所有写根路径：带单元的发布两条路都读同一对：「pinned_ring_slot: reclaim_floor_when_the_root_was_recorded.pinned_ring_slot(),」（`crates/singlefs-core/src/transaction.rs:2262`、`crates/singlefs-core/src/transaction.rs:16282`，快照外）；
   零单元发布照抄上一版：「pinned_ring_slot: previous_root.pinned_ring_slot,」（`crates/singlefs-core/src/transaction.rs:1725`，快照外），它的 journal 新根段取根上的（`crates/singlefs-core/src/transaction.rs:1746`）；
   施加记录拼根取记录新根段的（`crates/singlefs-core/src/recovery.rs:5281`、`crates/singlefs-core/src/recovery.rs:5307`，快照外）。mkfs 写 `NO_SLOT_PINNED`。`grep -rn 'RootRecord {' crates/singlefs-core/src` 里写根的构造只有上面这几处（transaction.rs 三处、make_filesystem.rs 一处、recovery.rs 施加记录拼根），别的命中在测试与 `mounted_read.rs` 的测试夹具里（该文件 `#[cfg(test)]` 起于 1244 行，夹具在 1428 行）。
6. 进程内重读（E2）与记根取同一个最小：重读循环「.min_by_key(|(_, pinned_txg)| **pinned_txg)」（`crates/singlefs-core/src/allocator.rs:1760`）对 BTreeMap（按槽升序）取第一个最小，
   平局同样落到区域号最小的槽，与记根的 `(pinned_txg, ring_slot)` 最小是同一个槽。

### 写者与池级 checker、GPU 内核对 p 的还原

- 池级 checker：「let slot = encoded_ring_slot / geometry.regions;」（`crates/singlefs-checker/src/walk.rs:9777`）、「region: encoded_ring_slot % geometry.regions,」（`crates/singlefs-checker/src/walk.rs:9780`），
  钉值「if slot == RING_SLOT_THE_GENESIS_ROOT_IS_SEEDED_IN_EVERY_REGION {」（`crates/singlefs-checker/src/walk.rs:9757`）为真取 0，否则取 p 本身。
- GPU 走读内核：「let pinned_txg = u64_of(select(pinned, 0u, pinned / g.regions == 0u));」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:5282`，快照外）。
- 三处是同一个函数：p → (区域 p mod 3, 槽 p div 3)；钉值 = 0 当槽 = 0，否则 = p。写者一侧的钉值：槽 0 取 0，别的槽取公式最旧 txg = p。两边互为逆。

### 判定

- **相符**：写者三形、全部写根路径、与池级 checker、GPU 内核的还原关系，对得上判决 R1（采乙）。
- **条款没写**：D16（发布语义） 已定项 1 的「槽号 0 的钉值是第 0 代根的 0」一句、平局顺序「区域在前、槽号在后」只在代码注释里（`crates/singlefs-core/src/allocator.rs` 的 `LowestPinnedRingSlot` 文档），kb 里没有；平局取哪一槽只影响 I-7.19 判哪一槽，不影响 t。
- 变异覆盖（`grep -n 'C773' crates/mutations.tsv`，快照外）：6142、6151 行是「槽 0 退回落点公式的钉值」，6143 行是「p 写钉值不写槽的编码」，6144 行是「平局倒着破」，6152 行是 checker 槽 0 不取 0。四个坏法都有行；本腿没跑，不证红。

### 什么现象会推翻它

- 造一个读坏的区域 1 槽 0（first_txg 14，区域 0 槽 0 已被 txg 12 盖掉），写行那次的根里 (t, p) 不是 (0, 1)：推翻写者一侧。已有用例就是这一形（`crates/singlefs-harness/tests/common_images_for_the_checker_tier/root_record_carries_the_reclaim_threshold_and_the_pinned_ring_slot.rs`，本腿没跑）。
- 区域 0、1 的槽 0 同时读坏，根里 p 不是 0：推翻平局规则。
- 池级 checker 或 GPU 对同一块盘面，I-7.18 / I-7.19 给出不同的结局：推翻「两边同一个函数」。

## T2　C724 两处 NoValidRoot

### 压着的条款

- 「`:3944` 在本进程准入路径上走不到的两条前提写进代码注释与 D23（journal 的角色与格式） 已定项 14 射程；共享函数保留这个错（有外部调用方）；不加进拒因单子」（`research/prompts/m3-close-b19-core-r1-main-verification.md:17`）
- 同一行：「正推腿没找到条款原文，是推的，实现时补一条断言钉住」（`research/prompts/m3-close-b19-core-r1-main-verification.md:17`）
- 「仍欠：那两处 `.ok_or(RecoveryFailure::NoValidRoot)` 要么也加进单子、要么写明它们在合法盘面上走不到（推的，没造出走到的盘面）」（`.claude/kb/checks-owed.md:439`）
- D23（journal 的角色与格式） 已定项 14 射程里的「两条前提」：`grep -c 'C724' .claude/kb/decisions/23-journal的角色与格式.md` 输出 0，条款还没写回（判决写「随实现那一批」）。

### 第二处 `expect` 的理由句与论证

理由句：「valid 不空（每块盘最新的有效根取得出最小值，空的 valid 在那一判已交回 NoValidRoot），最旧有效根就有：上限一定算得出」，在 `crates/singlefs-core/src/mount.rs:4024`。论证逐步读码：

1. `valid` 只在一处造出：「.filter(|root| root.checkpoint_txg >= current_floor)」（`crates/singlefs-core/src/mount.rs:3952`）接着滤抛弃，之后只被借用，再没被改。
2. `newest_per_device` 只从 `valid` 里填；空 `valid` ⇒ 空表 ⇒ `.min()` 为 `None` ⇒ 「.ok_or(RecoveryFailure::NoValidRoot)?;」（`crates/singlefs-core/src/mount.rs:3978`）当场交回，走不到后面。
3. `ceiling_from_newest_and_non_empty_roots` 交 `None` 只有一条路：第 4 新的不同状态没有、且最旧有效根没有：「.or(oldest_valid_root)?;」（`crates/singlefs-core/src/mount.rs:4082`）。
   最旧有效根是调用处的第三个参数「valid.iter().map(|root| root.checkpoint_txg).min(),」（`crates/singlefs-core/src/mount.rs:4021`），`valid` 不空它就是 `Some`。
4. 3978 行到 4021 行之间的 `pointers_of(..)?` 只会带别的错早退，不会改 `valid`。

结论：论证成立，与盘面、故障无关；这个 `expect` 是函数内部的不变量，对任何调用方（包括手拼分配器的实验装置）都不可能失败。

### 第一处的两条前提有没有钉住

- 前提 b（根带的 F 不高于它自己的 txg）：有断言：「current_root.rollback_floor <= current_root.checkpoint_txg,」（`crates/singlefs-core/src/mount.rs:4412`），在包装函数 `rollback_floor_ceiling_of_this_process`（`crates/singlefs-core/src/mount.rs:4404`）进门处。它与 `valid` 的滤法同一个量：`current_floor` 就是传进去的 `current_root.rollback_floor`。
  这一条断言在合法历史上不会触发的依据（读码，推的）：写者给新根带的 F 取盘上已有的根与系统配置里的；新实例第一次发布的 txg 取环里全部根与记录的最大 txg 加一（`crates/singlefs-core/src/mount.rs:1402` 的 `first_txg_of_new_instance`）；
  抬 F 的新 F 不超过上限，上限不超过某条有效根的 txg（`ceiling_from_newest_and_non_empty_roots` 取 `newest_on_every_device.min(..)`）；卸载抬到现行那一版的 txg，带它的空发布 txg 更高。
- 前提 a（本进程每条现行根都经记根进根环表，它的槽是「知道住着根的槽」）：不是断言前提本身，是在后果处断言：「Err(MountError::Recovery(RecoveryFailure::NoValidRoot))」出现在 `!matches!(..)` 里（`crates/singlefs-core/src/mount.rs:4430`）。
  记根的调用点读码都在：带单元的发布两处（`crates/singlefs-core/src/transaction.rs:2107`、`crates/singlefs-core/src/transaction.rs:14635`，快照外）先记再装根；零单元发布两处在根落盘之后记（`crates/singlefs-core/src/mount.rs:5433`、`crates/singlefs-core/src/mount.rs:7856`）。
  读坏即拒的那一支：知道住着根的槽重读仍坏 ⇒ `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`（`crates/singlefs-core/src/mount.rs:3847`），先于 3978 行。
- 前提之外还有一条隐含前提（C）：现行根不被它自己那张实例表判成被抛弃。判法是「.any(|row| row.instance == instance && checkpoint_txg > row.selected_root_txg)」（`crates/singlefs-core/src/recovery.rs:2114`，快照外）。
  代码注释的依据是「这个实例在现行那一张实例表里没有行」（`crates/singlefs-core/src/mount.rs:4387` 一带的注释）：行写的是被选中的旧根所属的实例，新实例的行要到下一次挂载才有。这一条没有断言单独钉，也不在 R2 的两条里；破了的后果同样落到 4430 行的断言上（现行根被滤掉，`valid` 可能为空）。

### 共享函数保留这个错之后，外部调用方拿到什么

- `rollback_floor_ceiling`（`crates/singlefs-core/src/mount.rs:3891`）与 `rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`（`crates/singlefs-core/src/mount.rs:3928`）签名与返回都没动；`# Errors` 里「一条有效根一条都没有 ⇒ RecoveryFailure::NoValidRoot」留着。手拼分配器（根环表为空、读坏的槽一律当没有根）的调用方仍拿到 `Err(NoValidRoot)`。
- 变的是包装函数的调用方：`raise_rollback_floor_to_the_admission_ceiling`（`pub`，`crates/singlefs-core/src/mount.rs:4486`）与 `raise_rollback_floor` 的准入入口（`crates/singlefs-core/src/mount.rs:4892`）经包装函数走，`Err(NoValidRoot)` 现在变成 panic。
  这两个入口的分配器参数是调用方给的。`record_root_written_by_this_process` 要求分配器装了根环表（`crates/singlefs-core/src/allocator.rs:1876` 一带的注释），所以写过根的分配器都有表；
  能让前提 a 破的只有「没写过根的手拼分配器 + 现行根所在槽读坏 + 环里没有别的 txg 不低于 F 的根」。

### 判定

- 第二处 `expect` 与理由句：**相符**（论证成立）。
- 前提 b 的断言：**相符**（有断言，行号在上）。
- 前提 a：**相符**（后果处断言），但钉的是后果不是前提；**条款没写**：D23 已定项 14 射程两条前提未写回、前提 C 两边都没写。
- 共享函数语义：**相符**；公开准入入口的 `Err` → panic 的变化，R2 只说「本进程准入路径」，没点名这两个 `pub` 入口：**条款没写**。

### 什么现象会推翻它

- 造一个 `valid` 为空而 3978 行没拦住的盘面（这个 `expect` 触发）：推翻第二处论证。读码上做不到，因为 3978 行与 4021 行读的是同一个 `valid`。
- 经 `mount_writable` 起会话、写过几次之后，让现行根的槽 FUA 落盘后读坏（读故障层），再调准入抬 F，得到 `NoValidRoot` 的 panic 而不是 `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`：推翻前提 a。已有用例造的就是这一形，期望是后者（`crates/singlefs-harness/tests/admission_floor_ceiling_with_every_root_ring_slot_unreadable_on_its_own_read.rs`，本腿没跑）。
- 条件式的会红用例（本腿没造）：`PoolAllocator::new` 手拼的分配器（不装根环表）+ 一个正常卸载抬过 F 的池（F 等于现行根 txg，环里只剩现行根不低于 F）+ 对现行根所在槽的读恒报错，调 `raise_rollback_floor_to_the_admission_ceiling`：
  期望（按公开 API 返回错误的读法）是 `Err`，今天读码得到 `crates/singlefs-core/src/mount.rs:4430` 的 panic。是不是缺陷取决于条款要不要让这两个 `pub` 入口也「不 panic」，所以只记成条款没写。

## T3　C928 冻结着时整理不发 discard

### 压着的条款

- 「整理入口 `run_compaction` 一族与 `discard_the_segments_reusable_as_a_whole` 都不判 `frozen_publish()`，照发 discard，可能 discard 掉冻结那次已落盘的根引用的单元，重发之前断电则恢复择到的根单元已没了」（`.claude/kb/checks-owed.md:591`）
- 「改法候选 discard 开头判冻结、或整理入口第一道判冻结（零轮），改完这条转绿、去掉那一判的变异必红」（`.claude/kb/checks-owed.md:591`）
- 「发布不接受失败。一次发布失败之后把它冻结，下一次发布之前先逐字节原样重发它」（`.claude/kb/decisions/23-journal的角色与格式.md:409`）

### 代码做了什么（读码）

1. 开头判冻结：「if let Some(frozen) = allocator.frozen_publish() {」（`crates/singlefs-core/src/mounted_session.rs:2931`），里面只记冻结那次的 txg 就「return discards;」（`crates/singlefs-core/src/mounted_session.rs:2934`），
   在「let segments_to_discard = allocator.take_the_segments_to_discard_on_every_device();」（`crates/singlefs-core/src/mounted_session.rs:2936`）之前。待 discard 表（`segments_reclaimed_not_yet_discarded`，`crates/singlefs-core/src/allocator.rs:358`）只在 `take_the_segments_to_discard` 里摘（`crates/singlefs-core/src/allocator.rs:878`），所以冻结着时表原样留着。
2. 能发 discard 的路径只有一条：`crates/singlefs-core/src` 里 `grep -rn 'discard_at'` 除 `block_device.rs`（设备实现与它自己的测试）外只剩 `crates/singlefs-core/src/mounted_session.rs:2988` 一处，在这个函数里。
   这个函数的调用点三处：用户改动发成之后（`crates/singlefs-core/src/mounted_session.rs:2136`）、整理显式入口（`crates/singlefs-core/src/mounted_session.rs:2307`）、后台入口（`crates/singlefs-core/src/mounted_session.rs:2498`）。
3. 入口与批间钩子：整理那一趟里的每次发布在发布路径第一道被冻结拒（「refuse_while_a_publish_is_frozen(allocator)?;」，`crates/singlefs-core/src/transaction.rs:9477`、`crates/singlefs-core/src/transaction.rs:12130`，快照外），拒了以 `CompactionRunOutcome::Failed(..Batch(refused))` 收（`crates/singlefs-core/src/mounted_session.rs:3409`）；
   批间钩子「in_the_middle_of_each_batch.in_the_middle_of_the_batch(background_compute);」（`crates/singlefs-core/src/mounted_session.rs:3657`）只拿后台计算状态，没有分配器与设备，发不了 discard。
4. `DiscardsAfterThePublishes` 的累积：每次调用从「let mut discards = DiscardsAfterThePublishes::default();」起（`crates/singlefs-core/src/mounted_session.rs:2930`），不跨调用累积；跨调用留着的是分配器上的待 discard 表，不是这个结构。
5. 冻结解除之后：冻结时「std::mem::replace(allocator, allocator_before_this_publish);」（`crates/singlefs-core/src/transaction.rs:1574`）把调用方手里的分配器换回发布之前那一份；重发成功「*allocator = allocator_after_the_publish;」（`crates/singlefs-core/src/transaction.rs:1612`）换成冻结那次成立之后的那一份。
   后一份是发布开头那份克隆（`crates/singlefs-core/src/transaction.rs:1837`、`crates/singlefs-core/src/transaction.rs:12132`）接着走完发布的，里头的待 discard 表 = 冻结前积下的 + 这次发布自己回收的。冻结期间没摘过表，所以没丢。
   重发那个函数自己不调 discard（没有第四个调用点）；积下的 discard 由解除之后下一次进上面三个调用点之一去发。冻结那次的落点所在的段在这一份里 `used_per_segment > 0`，`take_the_segments_to_discard` 对它只摘表不发（「} else if self.used_per_segment[segment_index] > 0 {」，`crates/singlefs-core/src/allocator.rs:890`），不会发到那一段。

### 判定

- 冻结着时没有别的路径把冻结那次落点所在的段交给 discard：**相符**（C928 的会红用例今天读码为绿；变异行在 `crates/mutations.tsv` 的 6145 行，快照外，本腿没跑）。
- 冻结解除之后积下的 discard 什么时候发、会不会丢：读码是「下一次进三个调用点之一就发、不丢」；条款没写这一步，也没有用例钉：**条款没写**。
  `grep -n 'resend' crates/singlefs-harness/tests/discard_while_a_publish_is_frozen_after_a_system_configuration_write_failure.rs` 零命中，用例断言到冻结着的那一趟为止。
- 旁注：`take_the_segments_to_discard_on_every_device`（`crates/singlefs-core/src/allocator.rs:2005`）与 `DeviceFreeMap::take_the_segments_to_discard` 是 `pub`、本身不判冻结；今天核心层只有上面那个函数调它们，但保护靠调用方守「先判冻结」，不靠这两个函数。

### 什么现象会推翻它

- 冻结着时（`allocator.frozen_publish().is_some()`）任何 `discard_at` 被调到：推翻「没有别的路径」。已有用例造的是系统配置槽轮换写失败的冻结，再调 `run_compaction`；
  本腿建议再补两形（本腿没造）：冻结发生在整理那一趟自己的某一批里（批发布的系统配置写注入 `WriteFails`），那一趟末尾的 discard；冻结发生在后台入口那一趟。
- 重发成功之后（`resend_the_frozen_publish` 返回 `Ok(Some(..))`）再调 `run_compaction`，`discards` 里没有冻结之前积下的、又不装冻结那次落点的段：推翻「不丢」。会红的用例形态：在已有用例末尾接一步重发再调一次整理，断言 `applied`/`unsupported` 里有被判的段之外的、冻结前回收空的段，且仍没有冻结那次落点所在的段。

## T4　C874 补写闸的原因与补写过的盘（连同 C929）

### 压着的条款

- 「补写闸没过的原因进挂载错误成员的载荷（新的公开封闭枚举：闸前读错加五件闸，只报第一件、第一块盘，条文写明），成功一支在挂载输出加补写过的盘」（`research/prompts/m3-close-b19-core-r1-main-verification.md:19`）
- 补写闸五件的条文：「这块盘带所选那一版的全部单元（清单非空、每个落点读一次、字节与验得过的那一份相同）；这块盘的根环槽与 journal 环槽没有一槽设备报读错；这些槽里没有比所选那一版新的根或记录（按择根的序）；这块盘单元区里没有写序实例大于所选那一版实例的单元；所选那一版的根槽那一半不缺」（`.claude/kb/decisions/22-单元原子性怎么合成.md:185`）
- 「补写闸哪一件没过在运行时有可观测的输出，用例断言它」（`.claude/kb/checks-owed.md:540`）
- C929：「这一态该判什么条款没写」（`.claude/kb/checks-owed.md:592`）；登记的待定题是「先定条款：回卷之后这块盘算补写过还是算落后」（`.claude/kb/checks-owed.md:592`）。

### 枚举成员是不是覆盖每一处拒绝

`devices_without_the_selected_version`（`crates/singlefs-core/src/mount.rs:6068`）里 `return Err` 恰两处：闸前那一判「if slots_reporting_a_read_error > 0 {」（`crates/singlefs-core/src/mount.rs:6094`，返回处 6095），
与补写闸有一件不过「Some(gate_not_met) => {」（`crates/singlefs-core/src/mount.rs:6111`，返回处 6112）。别的拒绝（盘有自证槽但缺单元或缺根槽）不返回 `Err`，推进 `devices_without`，走另一个错误（`WritableMountRefusedByDevicesWithoutTheSelectedVersion`），不是「没补写」。
补写闸函数一共五处 `return Some(..)`，各对一个成员：
闸一「return Some(RewriteGateNotMet::UnitsOfTheSelectedVersionNotAllOnTheDevice);」（`crates/singlefs-core/src/mount.rs:6187`）、
闸五「return Some(RewriteGateNotMet::RingSlotOnTheDeviceReportsAReadError);」（`crates/singlefs-core/src/mount.rs:6196`）、
闸三「return Some(RewriteGateNotMet::RootOrRecordNewerThanTheSelectedVersionInItsRings);」（`crates/singlefs-core/src/mount.rs:6199`）、
闸四「return Some(RewriteGateNotMet::UnitWrittenByALaterInstanceInItsUnitArea);」（`crates/singlefs-core/src/mount.rs:6214`）、
闸二「return Some(RewriteGateNotMet::RootOfTheSelectedVersionNotInItsRingSlot);」（`crates/singlefs-core/src/mount.rs:6224`）。
两个公开枚举：`DeviceNotRewrittenByTheAcquisition`（2 个成员，`crates/singlefs-core/src/mount.rs:6030`）与 `RewriteGateNotMet`（5 个成员，`crates/singlefs-core/src/mount.rs:6041`），合起来 6 个原因 = 6 处拒绝，一一对应。
载荷进错误：「SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {」（`crates/singlefs-core/src/mount.rs:7381`）带 `device` 与 `not_rewritten_because`。判决写的是「新的公开封闭枚举」一个，实现拆成两个（一个装闸前、一个装五件），语义等价。

### 「只报第一件、第一块盘」是不是确定的

- 盘的次序：「for device in reader.device_identities() {」（`crates/singlefs-core/src/mount.rs:6077`），文件后端的池按盘表次序给（`crates/singlefs-core/src/recovery.rs:306` 一带 `self.iter().map(..)`）；遇到第一块不过的就整个返回。
- 一块盘里的次序：先闸前读错，再闸一、闸五与闸三（同一次读环，读错先于更新，`rings_of_the_device_against_the_selected_version` 的枚举次序）、闸四、闸二；与代码注释写的「闸一、闸五、闸三、闸四、闸二」逐项相同（注释在 `crates/singlefs-core/src/mount.rs:375`）。
- 多盘多件同时不过：报盘表次序第一块两槽全废又没被补写的盘的第一件，与别的盘无关。同一份盘面、读结果不变时是确定的。
  两个次序副作用要知道：前面一块「有自证槽但缺单元」的盘只进 `devices_without`，被后面一块盘的 `Err` 盖掉（报的是系统配置读不出，不是缺单元）；闸的次序按读盘代价排（闸二最贵放最后），不按严重程度排。
- 「条文写明」：D22（单元原子性怎么合成） 的 185 行没写「只报第一件、第一块盘」（`grep -n '只报第一件' .claude/kb/decisions/22-单元原子性怎么合成.md` 零命中），只写在代码注释里：**条款没写**。

### 成功一支报的补写过的盘与真写了的盘是不是同一集合

- 进表的条件：两槽没有一份自证过、没有槽报读错、五件闸都过：「devices_rewritten_by_the_acquisition.push(device);」（`crates/singlefs-core/src/mount.rs:6108`），按盘表次序。
- 取号那一步自己再读一次，只放行这张表里的盘：「if devices_rewritten_by_the_acquisition.contains(&slots_of_this_device.device) {」（`crates/singlefs-core/src/transaction.rs:957`，快照外）；表外一份自证过的都没有的盘，取号报错。
- 取号写两轮对**每块盘**都写一次系统配置槽：「for index in 0..pool.devices.len() {」（`crates/singlefs-core/src/transaction.rs:1273`，快照外）。所以「真写了」的盘 = 全部盘（有自证槽的盘是常规轮换写），「补写」= 从无到有的那几块 = 表里的盘。
  报出来的集合（`MountOutput::devices_rewritten_by_the_acquisition`，`crates/singlefs-core/src/mount.rs:1080`；取值在 7132 行、装进输出在 7224 行）正是后一个集合；取号失败（含回卷）整个挂载返回错误，没有输出，所以成功一支不会报没写成的盘。
- 注意：字段名读起来像「所有被重写的盘」，实际只含「从无到有」的盘；表在取号之前读盘算出，取号时重读一次，两次读之间一块盘自己好了的话它仍会被报。属于改名或加注释的问题，不是集合不等。

### C929：回卷之后那块盘该不该再过闸，今天的代码做哪一种

1. 回卷怎么写：「for &index in written {」（`crates/singlefs-core/src/transaction.rs:737`，快照外）对已写出的每块盘再写一次，世代号 = 盘上自证过的最大世代号加一，`journal_instance` = 取号之前最高的实例，tail = 见证值（`crates/singlefs-core/src/transaction.rs:741`）。
   补写盘第一轮落了、第二轮后别处报错回卷之后，这块盘的两槽都自证得过（C929 登记的就是这一态）。
2. 下一次挂载：这块盘有自证过的槽，进不了「两槽全废」那一支（`newest_self_verified` 有值，补写闸函数不调），走 `what_a_device_behind_the_version_lacks`（`crates/singlefs-core/src/mount.rs:6452`）。
   缺单元那一半第一句「if newest_system_configuration_journal_tail >= version_journal_position {」（`crates/singlefs-core/src/mount.rs:6381`）为真就交回 `None`；缺根槽那一半「let is_behind_the_version = newest_system_configuration_journal_tail < version.journal_position;」（`crates/singlefs-core/src/mount.rs:6477`）为假也 `None`。
   回卷写的是（取号前最高实例，见证 tail），一般不落后于所选那一版 ⇒ 两处都 `None` ⇒ 这块盘什么都不再核就放行（推的，没造盘面跑）。就算算作落后，也只核单元和根槽（对应闸一、闸二），闸三、闸四、闸五不再核。
3. 所以今天做的是：**不再过补写闸**；回卷后的盘与一块普通的、有自证系统配置的盘无从区分，没有记号可让下一次挂载认出「这块盘是补写失败回卷的」。要改成「再过闸」得加记号或对所有有自证槽的盘都核闸三、四、五，这是格式或判法的改动，不是补一行。
   登记的 C929 要先定条款，条款没定：**条款没写**。
4. 会红的用例形态（等条款定了才有对错）：回卷用例（`crates/singlefs-harness/tests/acquisition_rewriting_a_device_without_a_self_verified_system_configuration_rolls_back_on_a_write_failure.rs`）之后，在回卷盘的单元区里放一个写序实例大于所选那一版的单元（闸四会拒的那一形），再挂一次：
   今天读码是挂得上；条款若定「回卷后算补写过、再过闸」则应拒，若定「算落后」则应走落后支（核单元与根槽）。两种定法下，「改走另一支的变异必红」都需要这一条用例。

### 判定

- 枚举覆盖每一处拒绝：**相符**。
- 取法确定：**相符**（代码）；「条文写明」：**条款没写**。
- 补写过的盘与真写了的盘：**相符**（集合：从无到有的盘；字段名有歧义，提示改名或补注释）。
- C929：**条款没写**；今天 = 不再过闸（读码，推的）。

### 什么现象会推翻它

- 造两块盘都两槽全废且都过不了闸、盘 1 先过不了闸一，报的却是盘 0 或别的件：推翻「确定」。
- 成功挂载的输出里 `devices_rewritten_by_the_acquisition` 点名的盘，取号写之前它已经有自证过的槽（不是从无到有）：推翻集合同一。已有用例的断言就是点名（`crates/singlefs-harness/tests/common_images_for_the_checker_tier/acquisition_rewrites_a_device_that_lost_both_system_configuration_slots_only_through_the_gates.rs`，本腿没跑）。
- 回卷用例之后再挂，盘面按闸四本该拒而今天挂得上：不推翻本格的读码，坐实 C929 的缺口。

## T5　C690 清扫阈值 365

### 压着的条款

- 「默认值定下之后：连挂到行数越过阈值的录制流里断言越过的那次可写挂载发出清扫发布、没越过的不发，且任一合法历史上行数不越过切换预留式子里行数项的上界」（`.claude/kb/checks-owed.md:360`）
- 「阈值是配置项，默认值另定」（`.claude/kb/decisions/18-块里携带什么信息.md:318`）：kb 条款侧还没写 365。
- 用户定案在调度记录：「用户弹窗定（原话即选项标签）」（`records/2026-10-18-里程碑三收尾与近期欠账.md:1848`），同一行里 C690 的选项是 365。
- 切换预留里的行数项：「**链重写**：副本数 × 32768 × max(1, ⌈(rows0 + N_switch) / 每片行数⌉) 字节」（`.claude/kb/decisions/28-挂载期承诺量.md:94`）。

### 代码做了什么（读码）

1. 常量：「pub const DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE: Self = Self(365);」（`crates/singlefs-core/src/instance_table_sweep.rs:72`），产品配置读它。
2. 触发：行数取挂载末尾现行那一版的表（`crates/singlefs-core/src/instance_table_sweep.rs:797`，含本次挂载写的那一行），「if rows <= configuration.row_count_threshold.0 {」（`crates/singlefs-core/src/instance_table_sweep.rs:807`）不扫，所以 366 行起扫。
3. 每片行数没动：「rows.div_ceil(rows_per_page).max(1)」（`crates/singlefs-core/src/instance_table.rs:124`）、「rows.chunks(instance_rows_per_page()).collect()」（`crates/singlefs-core/src/instance_table.rs:140`）都是 369；阈值只在 807 行读。
4. 预留：「.div_ceil(INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE)」（`crates/singlefs-core/src/admission.rs:566`）算 ⌈(rows0 + 3) / 369⌉，rows0 ≤ 366 是一片，367 起两片。E183 的第三段读数与它相符：「θ = 64、365 不越过一片的上界，θ = 369、738 越过」（`.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:22`）。
5. 第 366–369 行：
   - 行写得进：写行用的 `instance_table_rows_of_each_page` 按 369 切片，第 370 行开第二片，没有按行数拒绝的地方；这条路本来就是设计内的（树表 0 条的一版上写行「要实例表长到 64 片」，`crates/singlefs-core/src/transaction.rs:215`，快照外）。所以**没有「行写不进」这一形**。
   - 越过 365 却不清扫，有三形：（i）现行那一版树表 0 条（没写过文件的池）：「NoRowDeletedThisRound::SweepOnAVersionWithoutFileNotSupportedInTheFirstVersion,」（`crates/singlefs-core/src/instance_table_sweep.rs:548`）直接不清扫，空池上每次可写挂载一行，行数一路过 369；
     （ii）判一轮清扫交回任一 `NoRowDeleted`（实例表读不出、盘数不齐、某落点读失败、某根的账读不出、抹前重读不符）：行数不降；
     （iii）开关开着（交给挂着时的后台清扫，挂载末尾不扫）：产品配置 `PRODUCT` 不是这一档，只有调用方自己给清扫配置的挂载入口走得到。
     这三形下的后果不是写不进，是下一次挂载的切换预留按两片留（`crates/singlefs-core/src/admission.rs:566` 的式子）：保守，不危险。
   - 能清扫时清得下来：候选行 = 实例在根环里没有根的行：「.filter(|row| !instances_with_a_root_in_the_ring.contains(&row.instance))」（`crates/singlefs-core/src/instance_table_sweep.rs:359`），根环至多 3S ≤ 48 条根，所以行数超 365 时候选行不会为空。

### 旁发现（不在这一格的代码里，一并并排写出）

- kb 行没跟上：C690 行写「清扫器已写（`crates/singlefs-core/src/instance_table_sweep.rs`，阈值是占位值 369）」（`.claude/kb/checks-owed.md:360`）；代码是「pub const DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE: Self = Self(365);」（`crates/singlefs-core/src/instance_table_sweep.rs:72`）。
- 变异表里有一行引着改名前的常量：「InstanceTableRowCountThreshold::PLACEHOLDER_UNTIL_C690,」（`crates/mutations.tsv:2266`，快照外）；`grep -rn PLACEHOLDER_UNTIL_C690 crates --include=*.rs` 零命中，这一行变异后文本编不过，判不了它杀没杀。
  （这一行不在本批补丁里：`git diff HEAD -- crates/mutations.tsv | grep -c PLACEHOLDER_UNTIL_C690` 为 0。）

### 判定

- 阈值只改触发、不改每片 369，两者关系（366 行起扫、369 行写满一片）：**相符**。
- 第 366–369 行之间行写不进：**没有这一形**（链长第二片）。越过 365 而不清扫：**有三形，条款没写**（C690 的验证条文只说越过的那次发、没越过的不发）。
- 旁发现：kb 行与代码**不符**（上两行并排），变异表一行**不符**（编不过）。

### 什么现象会推翻它

- 一个 366 行的表在挂载末尾没有清扫发布、又不属于上面三形：推翻「366 起扫」。
- 写第 370 行时挂载被拒：推翻「行写得进」。
- rows0 = 366 的挂载，切换预留不是一片：推翻 admission 那一侧的 off-by-one 对账。

## T6　其余 `mount.rs` 改动

逐个函数先看 diff 里它到底动没动，再判压着哪条条款（C773 或 C874）。`research/prompts/_m3-close-b19-code-r1-diff.md` 里 `mount.rs` 的 hunk 按归属分：

| 函数 / 位置 | diff 里的改动 | 压着的条款 | 判定 |
|---|---|---|---|
| `txgs_ring_slots_bad_at_mount_can_hold`（`crates/singlefs-core/src/mount.rs:2936`） | 闭包里公式最旧 txg 再过 `oldest_txg_a_ring_slot_bad_at_mount_can_have_held`（2976 行），加文档 | C773，判决 R1 采乙（见 T1） | 相符；最大 txg 一路没动，`oldest <= highest` 的断言（0 不大于任何）仍成立 |
| `ring_slots_self_verified_on_only_one_of_the_two_root_ring_passes` | 函数本身没动；它只是 hunk 头（`@@ -2908`）里的上下文名，被改的是紧随其后的 `txgs_ring_slots_bad_at_mount_can_hold` 文档 | 无（文档属 C773 那一格） | 无行为变化 |
| `ring_slots_recorded_as_holding_an_abandoned_root_by` | 函数本身没动；hunk 头（`@@ -4339`）上下文名，被改的是 `rollback_floor_ceiling_of_this_process` 的文档与函数体 | C724（见 T2） | 无行为变化（函数本身） |
| `establish_instance`（`crates/singlefs-core/src/mount.rs:7132`、`crates/singlefs-core/src/mount.rs:7224`） | 多两处：取号前把补写盘表克隆一份、放进 `MountOutput` | C874，判决 R4「成功一支在挂载输出加补写过的盘」 | 相符（见 T4） |
| `devices_without_the_selected_version`（`crates/singlefs-core/src/mount.rs:6068`） | `Err` 载荷由 `DeviceIdentity` 变成 `(DeviceIdentity, DeviceNotRewrittenByTheAcquisition)`，两处 `return` 带原因 | C874，判决 R4 | 相符；拒绝的集合与判的次序不变，只多载荷 |

`mount.rs` 其余 hunk 的归属：

- 归 C874：`SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition` 加字段、`MountOutput` 加字段、两个新 `pub enum`、`devices_rewritten_by_the_acquisition_after_checking_each_device` 的 `map_err`。
- 归 C724：`rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount` 的 `expect`、`rollback_floor_ceiling_of_this_process` 签名与两处断言、两个调用点改传 `&current.root`。
- 归 C773：`oldest_txg_a_ring_slot_bad_at_mount_can_have_held` 与测试模块里的用例改参照、加用例。
- 归 C690：`mount_writable_with_compaction_switches` 一带文档里「阈值默认 365 行」一句，纯文字。
- 别的会话的改动，题面排除，本腿不判：`background_compute` 的导入、`mount_writable_with_background_compute_settings` 与新的 `..._and_the_gc_cpu_compensation_threshold` 一族（GC 停着时在 CPU 上补跑的阈值）。

压不着条款的行为变化：这五处里**没有**。`rollback_floor_ceiling_of_this_process` 的 `Err(NoValidRoot)` → panic 与前提 b 的新断言是行为变化，但压在 C724 / R2 上（T2 已判，其中公开入口那一点记成条款没写）。

顺带核的：`root_record.rs` 的大块改动是把整份挪进 `verus!`（规格函数、`assume_specification`），`ROOT_RECORD_FLAG_UNMOUNT_MARKER` 由 `1 << 0` 写成 `1`、`ROOT_MAGIC` 由字节串写成 `[0x53, 0x46, 0x53, 0x52]`（'S'、'F'、'S'、'R'），读写顺序与偏移没变（校验和之后 296 字节：三条指针、t 在 428、p 在 436、算法类型、nonce 与 MAC，合 466）。
钉槽字节只经 `of_the_lowest_pin(Option<u64>)`（`crates/singlefs-core/src/root_record.rs:153`）收槽的编码，`requires` 与 panic 的两种反面一致。Verus 能不能验过，本腿不跑。

### 什么现象会推翻它

- 把上表任一个「函数本身没动」的函数 `git diff HEAD -- crates/singlefs-core/src/mount.rs` 里找到它函数体内的改动行：推翻「无行为变化」。本腿是对着背景材料的 diff 全文（`research/prompts/_m3-close-b19-code-r1-diff.md`）逐 hunk 读的，没有另跑 `git diff`。
- `devices_without_the_selected_version` 在某个盘面上拒的盘或次序与改前不同：推翻「只多载荷」。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1　C773 写者带（钉值, 槽） | 相符；D16 槽号 0 一句条款没写 | 三形钉值都经同一个第 0 代根取 0 的函数；全部写根路径带同一对；写者、池级 checker、GPU 内核对 p 同一个函数关系；平局取区域号最小的槽，只在代码注释里 |
| T2　C724 两处 NoValidRoot | 相符；公开准入入口 Err 变 panic 条款没写；D23 两条前提未写回 | 第二处 `expect` 论证成立（同一个 `valid`）；前提 b 有断言；前提 a 钉在后果处；多一条隐含前提（现行根不被自己的表判抛弃）没人钉 |
| T3　C928 冻结着时不发 discard | 相符；解除后条款没写、无用例 | 全仓唯一的 `discard_at` 在被守住的函数里；待 discard 表冻结期间不摘，重发换回的分配器里还在，下一次进三个调用点之一发 |
| T4　C874 原因与补写过的盘 | 相符；「只报第一件」条文没写；C929 条款没写 | 6 个原因对 6 处拒绝；盘表次序第一块、闸次序固定；报的盘 = 从无到有的盘；回卷后那块盘今天不再过闸（读码，推的） |
| T5　C690 阈值 365 | 相符；越过不扫三形条款没写；旁发现两处不符 | 366 行起扫，369 不动；没有行写不进的形（链长第二片）；kb 的 C690 行还写占位 369；`crates/mutations.tsv:2266` 引已删的常量 |
| T6　其余 mount.rs 改动 | 相符 | 五处各压 C773 或 C874（两处函数本身没动）；这五处没有压不着条款的行为变化 |

## 没做什么

- 没有判 P1–P4（攻方腿的推论格），没有替主 agent 采纳或出判决。
- 没有编译 Rust、没有跑任何测试、没有碰 GPU：以上全部是读码加 grep 的结论；上面说的「已有用例」与「变异行」，本腿只读了文本，没有证红证绿。重型测试（checker 档）一概没跑。
- 没有写模型，没有 `tools/rerun.sh`，没有要主 agent 复跑的命令；可核的只有行号与引文，`python3 scripts/agents/cite-check.py` 的末行见回复。
- 没有读禁读清单里的任何一份（另一条腿的报告与模型、主 agent 核对稿）。
- 没有读派发提示之外的 `git diff HEAD` 全文，`mount.rs` 的 hunk 归属按背景材料里的 diff 全文逐个读；快照清单之外的文件（`transaction.rs`、`recovery.rs`、`root_ring.rs`、`make_filesystem.rs`、`instance_table.rs`、`crash_judge_checker.wgsl`、`crates/mutations.tsv`）在主工作区现读，已标「快照外」，没有快照可对 sha256。

## 复核不了

- 各条「读码，推的」的行为判定（T2 的 panic 触发条件、T3 的解除后不丢、T4 的 C929 今天走哪一支）需要建盘面运行才能坐实；本腿按派发不跑也不编译，写明会红的用例形态交主 agent 或实现员。
- GPU 内核与池级 checker 的逐盘面一致只读了两处还原的表达式，没跑 GPU（题面也禁碰）。
