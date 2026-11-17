# m4-gpugc-precheck-b-a-code-r1 正推腿（Sonnet）报告（2026-11-12）

立场：从用户定的乙、甲和 `crates/` 今天的实现推「应该是什么」，再与被判的代码逐格比。没起编译、没跑测试（子 agent 不交任务）；下面标「量的」的是我只读命令现取的输出，标「推的」的是读码推出、没量过。

## 读的是哪一棵树

冻结副本与主工作区被判文件逐字节相同（量的，`sha256sum` 取前 12 位）：

    crates/singlefs-core/src/allocator.rs main=062f04d2004a frozen=062f04d2004a
    crates/singlefs-core/src/mounted_session.rs main=d67aad713c31 frozen=d67aad713c31
    crates/singlefs-core/src/transaction.rs main=8bfdc6628feb frozen=8bfdc6628feb
    crates/singlefs-core/src/admission.rs main=746a620e907f frozen=746a620e907f
    crates/singlefs-core/src/compaction_plan.rs main=dcb3dc7688a4 frozen=dcb3dc7688a4
    crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs main=a2906eae3f76 frozen=a2906eae3f76

下面的行号两棵树一致，引用写仓根相对路径。

## T1　乙的上界对不对

### T1-0　代码是不是用户定的那一件：成立（口径比用户原话字面宽一档，同向）

- 用户课题原话在记录里：`records/2026-10-19-里程碑四GPU垃圾回收开工.md:1005`：「改法乙（分配器按段增量维护 defer 里成对的数据单元与段外单槽的计数，先算槽对那一判用它）」。
- 欠账去向一致：`.claude/kb/checks-owed.md:641`：「分配器按段增量维护 defer 里成对的数据单元与段外单槽的计数（改法乙，零轮）」。
- 代码数的是「段外两槽都不罩在未释放记录下的偶数起点槽对」：`crates/singlefs-core/src/allocator.rs:783`：「pub fn slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(」；增量维护的单槽位在 `crates/singlefs-core/src/allocator.rs:881`：「self.is_under_an_unreleased_record[index] = match transition {」。
- 这个数包含三类对：defer 里两槽成对的数据单元、defer 槽配一个空闲单槽、以及隔离或抬 F 扣住而没有未释放记录罩着的槽。前两类就是用户原话的「defer 里成对的数据单元与段外单槽」；第三类（隔离、扣住）用户原话没点名，但现行上界本来就把它们算作放得开（`free_but_blocked` 与 `isolated`，补丁删掉的就是这两项的旧读法），A/B 的乙臂也放开了隔离与扣住（A/B 事务员报告 `m4-gpugc-precheck-ab/result.md` 的「没做什么、留着什么」一节：「乙的上界放开了隔离与扣住两样」）。宁大勿小，收紧不会比用户原话更紧，所以不会多拒。
- 推翻它的现象：用户弹窗实际选项文字（我没读到，只读到记录里主 agent 的转述）若明确写「隔离与扣住不算」，则代码比用户定的松。

### T1-1　增量计数在每一条改未释放记录的路上都跟上：成立

- 未释放计数 `unreleased_per_segment` 与新加的位、两个槽对计数只在两处被改，一处是建空闲图，一处是下面这两个函数（量的，`grep -nE` 全文件，已去掉测试段）：

      847:                    if self.unreleased_per_segment[segment] == 0 && is_a_full_segment {
      850:                    self.unreleased_per_segment[segment] += 1;
      853:                    self.unreleased_per_segment[segment] = self.unreleased_per_segment[segment]
      856:                    if self.unreleased_per_segment[segment] == 0 && is_a_full_segment {
      881:        self.is_under_an_unreleased_record[index] = match transition {
      890:                self.slot_pairs_without_an_unreleased_record_per_segment[segment] -= 1;
      891:                self.slot_pairs_without_an_unreleased_record -= 1;
      894:                self.slot_pairs_without_an_unreleased_record_per_segment[segment] += 1;
      895:                self.slot_pairs_without_an_unreleased_record += 1;

  847 至 856 在 `note_slots_under_an_unreleased_record` 里，它在每个槽的循环里先调新函数，再改 `unreleased_per_segment`：`crates/singlefs-core/src/allocator.rs:842`：「self.note_one_slot_under_an_unreleased_record_in_the_slot_pair_counts(」。三个字段都是私有的，建 `DeviceFreeMap` 的 `Self {` 只有 `crates/singlefs-core/src/allocator.rs:577` 那一处（量的，`awk` 扫 525 至 1380 行），所以没有第二条改法。
- 调 `note_slots_under_an_unreleased_record` 的只有三处：释放 `crates/singlefs-core/src/allocator.rs:913`：「UnreleasedRecordTransition::SlotsLeaveTheirUnreleasedRecord,」，复活出 defer `crates/singlefs-core/src/allocator.rs:933`：「UnreleasedRecordTransition::SlotsComeUnderAnUnreleasedRecord,」，分配 `crates/singlefs-core/src/allocator.rs:1181`：「UnreleasedRecordTransition::SlotsComeUnderAnUnreleasedRecord,」。
- 产品路径上调这三个入口的（量的，`grep -n` 去掉测试）：重开重建 `crates/singlefs-core/src/allocator.rs:2514`：「device_map.mark_allocated(record.slot, span);」与 `crates/singlefs-core/src/allocator.rs:2516`：「device_map.mark_released(record.slot, span);」；记一次分配 `crates/singlefs-core/src/allocator.rs:2616`：「device.mark_allocated(placement.slot, placement.span);」；释放 `crates/singlefs-core/src/allocator.rs:2661`：「device.mark_released(placement.slot, placement.span);」；管理员回退复活 `crates/singlefs-core/src/allocator.rs:2711`：「device_map.mark_allocated(placement.slot, placement.span);」与 `crates/singlefs-core/src/allocator.rs:2713`：「device_map.mark_resurrected_out_of_the_defer_queue(placement.slot, placement.span);」。开段与 bump 提交内生块、回落都经 `record`（2616）进同一个 `mark_allocated`。
- 回收不动未释放位是对的：回收只经 `crates/singlefs-core/src/allocator.rs:3186`：「device_map.mark_reclaimed(record.slot, u64::from(record.span_slots));」，候选先过 `crates/singlefs-replay-prefix/src/gc_crash_proofs.rs:73`：「released && generation <= floor」，回收的本来就是已释放记录。
- 释放时留在已分配的隔离记录（`release_leaving_the_record_allocated_on`）：`crates/singlefs-core/src/allocator.rs:2654` 起改记号后 `continue`，不调 `mark_released`，那一槽照旧罩在未释放记录下，位不动，与记录一致。
- 隔离、清隔离、抬 F 扣住、放开扣住（`isolate*`、`clear_isolation_of_slot`、`hold_until_floor_takes_effect`、`release_holds`）只改三种挡位，不改「罩不罩在未释放记录下」，新计数按定义不随它们变，正是上界要的口径。
- 推翻它的现象：在真分配器上找到一条改 `AllocationRecord::is_released` 而不经 `mark_released`/`mark_allocated`/`mark_resurrected_out_of_the_defer_queue` 的产品路径（我 `grep -n 'is_released = ' crates/singlefs-core/src/allocator.rs` 命中 144（解析盘上记录的局部变量）、1415（改写已回收记录，随后走 `mark_allocated`）、2658（释放）、2703（复活），三处写入都在上面三个入口的同一条调用链里；`region_scan.rs:906` 与 `allocation_record_tree.rs:1051` 在测试模块里）。

### T1-2　「分配位置着而不罩在未释放记录下就是 defer 里的」：成立（读码）

- 分配位只在 `crates/singlefs-core/src/allocator.rs:1174`：「self.set_blocking_bit(index, SlotBlockingBit::Allocated, true);」置（`mark_allocated` 内，紧接着 1178 调 `note_slots_…` 置未释放位）；只在 `crates/singlefs-core/src/allocator.rs:1080`：「self.set_blocking_bit(index, SlotBlockingBit::Allocated, false);」清（`mark_reclaimed` 内，唯一产品调用点是 3186，只收已释放记录）。未释放位只在 `mark_released` 清。所以 分配位 ∧ ¬未释放位 恰是「已释放、还没回收」= defer 里的；`deferred_slots` 随 `mark_released` 加、随 `mark_reclaimed` 减，与它同步。
- 缺口（不影响现状）：`mark_reclaimed` 只断言「跨度里全是已分配」，不断言「已释放」，靠调用方的 `released && generation <= floor` 过滤守住。

### T1-3　扣住的槽按已释放记录算放得开：成立

- 扣住只加在回收出来的槽上：`crates/singlefs-core/src/allocator.rs:3191` 的 `hold_until_floor_takes_effect(record.slot, …)` 紧跟 3186 的回收，记录在 `reclaimed` 里仍写着已释放（`reclaim_released_records_up_to` 不删记录）。这些槽的未释放位是假，新计数算它放得开；`release_holds`（1048）放开后正是空闲。

### T1-4　与探针 `bound_by_real_slot_pairs` 的口径差：在可达状态上相等（推的，没量过）

- 探针（A/B 事务员模型 `m4-gpugc-precheck-ab/model/probe/core_probe_module.rs` 的 `bound_by_real_slot_pairs`）按记录扫：罩在未释放记录下的不放，罩在已释放记录下的放，没有记录罩着的看是不是隔离。代码按位算。两边只会在「没有记录罩着、只置了扣住位」的槽上不同（代码放、探针不放），实现员说这个形造不出来。我按读码核：扣住只加在 3191 紧跟回收的槽上，记录不删；记录只在 `make_room_for_record_on_device` 里被删，而它删的是与新分配的槽相交的已回收记录，新分配要求槽不被挡（含扣住），所以删记录时那几槽不是扣住的。可达状态上两边相等，代码这一边只可能更大（宁大勿小）。
- 推翻它的现象：造出（或读码找到）「扣住位置着、记录被删」的状态。

### T1-5　乙算小（误拒一个推了本来写得进的写）：读码找不到这样的形（推的，没量过；A/B 对这一条没有区分力）

结论所靠的三个事实，合起来是单调性：推串之后段外两槽都空着的槽对 ⊂ 推串之前段外两槽都不罩在未释放记录下的槽对。

- 事实一，聚簇段集合只增不减：全文件改 `cluster_segments` 的只有 `crates/singlefs-core/src/allocator.rs:2910`：「self.cluster_segments.insert(segment_start);」（量的，`grep -nE 'cluster_segments\.(insert|remove|clear|retain)|cluster_segments = '` 只命中 2910，另两行是读）。段外槽对只会因多开一段而变少，上界读的是推串之前那一刻的集合，不会比真提交时少。字段文档 `crates/singlefs-core/src/allocator.rs:2121` 写明是「这次挂载开过的聚簇段的起点」。
- 事实二，推串自己释放的记录回收不到：串里每次空发布换下的固定点单元，释放代是那次发布的 txg；回收条件是 `crates/singlefs-replay-prefix/src/gc_crash_proofs.rs:73`：「released && generation <= floor」，floor 取 max(F_生效, 环里最旧有效根)：`.claude/kb/decisions/16-发布语义.md:35`：「| 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根) |」。F 的上限 `.claude/kb/decisions/16-发布语义.md:38`：「非空有效根不足 4 个时取最旧的有效根」（同一行：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)），都是串开始之前就有的根，txg 小于串里任何一次发布的 txg；环里最旧有效根要追上串里某次发布的 txg，得再发满一圈：环长 3S（`.claude/kb/decisions/16-发布语义.md:36`：「3S 是环长」），S 不小于 4（`.claude/kb/decisions/16-发布语义.md:46`：「每区槽数 S ≥ k_tol + 2 = 4」），一次准入至多 8 次发布（`.claude/kb/decisions/16-发布语义.md:44`：「一次准入最多 B = 4 + 2 k_tol 次发布（k_tol = 2 ⇒ 8」），8 < 12，追不上。所以串里自己换下的单元不会被同一次准入里后一串回收：实现员报告「停下交主 agent 的」一节里后一串又回收到固定点单元那一形在 k_tol = 2、S 取下界时走不到（推的；S 取别的值只会更大）。
- 事实三，串里自己的分配只加未释放记录，不会让一个现在罩着的槽变空。
- 单元测试对「上界不小于真放开之后的数」有区分力：`crates/singlefs-core/src/allocator.rs:4030`：「fn slot_pairs_once_everything_is_released_never_fall_below_the_pairs_after_really_releasing_everything(」在 `crates/singlefs-core/src/allocator.rs:3932`：「for step_number in 1..=600 {」的每一步之后，拿拷贝上 `crates/singlefs-core/src/allocator.rs:3898`：「released.reclaim_released_up_to(CheckpointTxg(u64::MAX), ReclaimedReuse::Immediately);」加放开扣住、清隔离之后的 `unblocked_slot_pairs_outside_the_cluster_segments`（另一套按三种挡位位增量维护的数）当真值比。另一条 `crates/singlefs-core/src/allocator.rs:4011` 那条拿按记录扫的数比，算法与被判的位不同，满足「审计与被审计不同算法」。覆盖的步型：分配、开段或 bump、释放、回收（立即或扣住）、放开扣住、盘 0 隔离或清隔离、复活 defer 里的落点；没覆盖：释放时留在已分配的隔离记录、回收之后再复活（走 `reclaimed.remove` 那一支）、整份重建。后两者最终都落在同一个 `mark_allocated`，我读码判风险低（推的）。
- A/B 对「乙不误拒」没有区分力（量的，任务 13802 日志里 `PROBE_RESULT` 行的原样字段）：

      命令：sed -n <行号>p <日志> | tr ' ' '\n' | grep -E <要的字段> | tr '\n' ' '（日志是 task-server/task log 13802 --all 存下的 log13802.txt，在我的草稿目录里；取的是第 969、970、1153 行，只留这几个字段）
      drawn arm=as-coded oracle_published_after_a_series=0 c_published=0 c_fired=9320 c_white=19444 outlet_fewer_than_as_coded=0 published_eaten_vs_as_coded=0
      drawn arm=real-slot-pairs oracle_published_after_a_series=0 c_published=0 c_fired=22479 c_white=3151 outlet_fewer_than_as_coded=4801 published_eaten_vs_as_coded=0
      series-admits arm=real-slot-pairs oracle_published_after_a_series=5465 c_published=5740 c_fired=613 c_white=1345 outlet_fewer_than_as_coded=0 published_eaten_vs_as_coded=0

  原那一族没有任何一格在推串后写得进（`oracle_published_after_a_series=0`，四臂 `c_published=0`），`published_eaten_vs_as_coded=0` 在那里是平凡成立；新造那一族 5465 格写得进，但记录已写明（`records/2026-10-19-里程碑四GPU垃圾回收开工.md:1019`：「5465 格先拒的全是式子那一判」），乙在槽对那一判上没动。所以「乙在槽对那一判上误拒 0」没有被量过，只有上面的读码与单元测试。
- 推翻它的现象：在真分配器上造出一段历史，推串之后重发的写成功、而推串之前槽对那一判的上界小于需求。读码上唯一可能的缺口是下一节的重挂。

### T1-6　乙少报出路 4801 格丢的是不是走得通的出路：读码上丢得了一条，量的数里没丢到过，余下的没量

- 先看乙臂和现行臂在这 4801 格里差在哪：乙拒的是 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`，这个成员没有出路字段（`crates/singlefs-core/src/mounted_session.rs:691` 起的结构体）；现行臂推一串后报 `NoSpaceAfterRaisingTheFloor` 带被宽护卡住，出路有两条（`crates/singlefs-core/src/mounted_session.rs:805`：「UnmountThenMountWritableAgain {」，与推环）。
- 推环那条出路不会丢：它放的是宽护的隔离槽；乙的上界已把全部隔离槽算作放开（T1-0），上界仍小于需求，说明放完隔离也不够，推环帮不上。
- 卸载重挂那条丢得了：聚簇段集合是挂载内的（`crates/singlefs-core/src/allocator.rs:2189`：「cluster_segments: BTreeSet::new(),」；`crates/singlefs-core/src/allocator.rs:2506`：「let mut allocator = Self::new(devices);」，重建分配器从空集合起步），上界扣掉的「聚簇段里的空对」重挂之后就是用户数据的候选。这个缺口现行上界同样有（它也读同一个集合），不是乙新造的，但乙把「报了出路」变成了「不报」。
- 量的（任务 13802 日志 `FOLLOWED` 行原样）：

      drawn FOLLOWED_AS_CODED stuck_followed=285 stuck_remount_published=0 stuck_push_ring_admitted=0 lost_first_followed=177 lost_first_remount_published=0 lost_after_followed=201 lost_after_remount_published=0 white_no_stuck_followed=954 white_no_stuck_remount_published=1
      drawn FOLLOWED arm=real-slot-pairs stuck_followed=253 stuck_remount_published=0 stuck_push_ring_admitted=0 lost_first_followed=295 lost_first_remount_published=0 lost_after_followed=45 lost_after_remount_published=0 white_no_stuck_followed=605 white_no_stuck_remount_published=1
      series-admits FOLLOWED arm=real-slot-pairs stuck_followed=0 stuck_remount_published=0 stuck_push_ring_admitted=0 lost_first_followed=0 lost_first_remount_published=0 lost_after_followed=0 lost_after_remount_published=0 white_no_stuck_followed=343 white_no_stuck_remount_published=148

  乙臂被跟着重挂的 lost-first 类共 `lost_first_followed=295` 格，写进 0 次；A/B 报告说少报出路的格落在这一类，但这 295 格里有几格是 4801 格之一，日志没单列（量的：`grep -c 'outlet-fewer class'` 对 13802 日志输出 0），所以 4801 格里被跟过的数不知道，上限是 295（占 4801 的 6.1%，算的）。新造那一族没有「被宽护卡住」的格，却有 148÷343 格在不报出路的情况下重挂后写得进（`white_no_stuck_remount_published=148`），说明重挂确实能救一部分格，只是原那一族的探针几何里一次都没救到「少报出路」的格。
- 判：没有证据说这 4801 格丢的是一条走得通的出路，也没有证据说走不通；原因不在乙的算法，在探针几何没造「聚簇段里还剩空对、补上就够」的形。
- 推翻它的现象：造一个池，让开放的聚簇段里还有空对、段外的对放开隔离与 defer 之后仍不够需求，写被乙拒，卸载重挂后同一个写写进；那就是丢了一条走得通的出路，该给 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease` 一个出路，或让上界把聚簇段里的空对按「重挂可得」另记一档。

### T1-7　实现员报告「停下交主 agent 的」一节里讲后一串回收到的固定点单元那一条：不算进上界是对的（推的）

见 T1-5 事实二：同一次准入里串自己换下的单元回收不到，不用算进上界。推翻它的现象：S 小于 4 的几何（门禁拒，`.claude/kb/decisions/16-发布语义.md:46` 的下界），或一次准入多于 12 次发布。

## T2　甲的射程与次序

### T2-1　「推串之前」读成「这次调用还一串没推」对不对得上 D16 已定项 10 射程原文：条款没写甲，读法是实现员的，与射程里已有的三处范围一致，与「头一次被拒就判出」字面差一形

- 射程原文（`.claude/kb/decisions/16-发布语义.md:276`，同一行）：「推抬 F 放得出的全放出来也放不下就一串都不推，报 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`，头一次被拒就判出时盘上逐字节不变（推过一串之后、或 unlink 意图发成之后走空才判出的，盘上有这次调用写的东西）」；范围句：「先算罩 `accept`、`publish_user_change` 与走空三处，不罩改动发成之后另发的搬迁批，也不判提交内生块那一判（照推）；短缺种类混着来时可能推过一串才判出」；上界句：「式子那一判的上界是可用加 defer 加隔离，短在段外槽对那一判同样先算，上界恰好等于需求照推」。
- 这一行里没有合取 7，也没有「推串之前」：`grep -c '合取 7' .claude/kb/decisions/16-发布语义.md` 输出 0（量的）；`grep -nF '推串之前' .claude/kb/decisions/16-发布语义.md` 无输出（量的）。所以「推串之前」读成「这次调用还一串没推」不是对不对得上原文的问题，是原文根本没有这句，读法来自实现员（其报告「停下交主 agent 的」一节里讲甲射程的那一条自己写明「是我的读法」）。合取 7 的条款在 `.claude/kb/decisions/03-空间分配.md:148`：「只拦前台写入类发布」。
- 代码的读法：`crates/singlefs-core/src/mounted_session.rs:6193`：「series_pushed_in_this_change > 0」，传入的是 `crates/singlefs-core/src/mounted_session.rs:6242`：「published_in_this_change.floor_raises.len(),」。数的是推过的串，不是发成的版：走空先发成几步、没推串、然后用户那一次写被拒，也判。探针只判头一次被拒。两种读法只差这一形。
- 与射程原文对：括注「unlink 意图发成之后走空才判出的，盘上有这次调用写的东西」说的是乙在走空之后判出也算入范围，所以甲在「走空发成了几步、没推串」之后判出，盘上有走空写的东西，与括注同类，不新违例。字面「头一次被拒就判出时盘上逐字节不变」在调用开头先走空时本来就不成立（`.claude/kb/checks-owed.md:642`：「在调用开头先走空发成几步时字面不成立」），甲没有加重这一条，只是让它在甲上也可达。
- 范围三处与射程一致：三个调用点都经同一段先算（`crates/singlefs-core/src/mounted_session.rs:6304`：「let (published_in_this_change, refusal) = judge_before_pushing_a_floor_raise_for_a_refusal(」是 `push_one_floor_raise_for_a_refusal_for_space` 的第一句），传参的三处是 `crates/singlefs-core/src/mounted_session.rs:3845`：「RefusedPublishUnderTheEmptyClusterSegmentsConjunct::of_the_user_change(change),」（`publish_user_change`）、`crates/singlefs-core/src/mounted_session.rs:6424`：「RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ExemptOrNotKnownBeforeThePublish,」（走空）、`crates/singlefs-core/src/mounted_session.rs:7263`：「RefusedPublishUnderTheEmptyClusterSegmentsConjunct::of_the_user_change(」（`accept`）。搬迁批不经这个函数（它的调用者只有这三处，量的：`grep -n 'push_one_floor_raise_for_a_refusal_for_space' crates/singlefs-core/src/mounted_session.rs` 的非注释命中 3838、6292、6416、7255），合取 7 对搬迁批本来也免（`crates/singlefs-core/src/transaction.rs:13367`：「if writes_in_the_foreground && !is_a_rollback && relocations.is_empty() {」）。
- 要补条款：射程那一行要加甲、加甲的次序（先乙后甲）、加甲判的是「写之前那一刻」；实现员建议的新文与代码一致（其报告「要改的条款」一节），日期记录在第 125 批（2026-11-11）。这是主 agent 的写回，不是代码的缺陷。
- 推翻它的现象：用户的原话明确要求「只在头一次被拒时判」（那就是探针的读法，代码判宽了一形）。

### T2-2　改 size 与建快照不先判，有没有该判而漏的前台写入：有两种该判而没判，都在安全侧

合取 7 判不判（`crates/singlefs-core/src/transaction.rs:13363`：「let writes_in_the_foreground = plan.file.is_some()」起，到 13367 的 `if`）与甲的分类（`crates/singlefs-core/src/mounted_session.rs:6153` 的 `of_the_user_change`，穷举、无 `_ =>`）逐变体对：

| 会话改动 | 合取 7 判 | 甲判 | 对得上 |
|---|---|---|---|
| Overwrite、SequentialWrite、OverwriteOfAnInode、SequentialWriteOfAnInode | 判（`plan.file` 有） | 判 | 是 |
| NewInodes | 判（新 inode 记录非空） | 判 | 是 |
| WriteAtOffset | 判（部分写） | 判 | 是（推的：读 `reach_of_the_empty_cluster_segments_conjunct` 的 `PartialWrite` 臂） |
| DataChangesPublishedTogether | 判 | 判 | 是 |
| Unlink、DestroySnapshot | 免（删除类） | 免 | 是 |
| Truncate | 变长或只改记录判，截断意图与各批免 | 免（不先判） | 漏：变长的 Truncate 该判而没判 |
| CreateSnapshot | 判（有普通分配时） | 免（不先判） | 漏：建快照该判而没判 |

- 漏的两种后果：被空间拒、全空段又少于 `N₀`，甲不拦，照推一串，推完重发被合取 7 拒，报 `UserChangeRefused::Publish` 带 `EmptyClusterSegmentsBelowTheFloor`（非空间不够的拒绝原样交回，`crates/singlefs-core/src/mounted_session.rs:2116` 的 `refusal_is_short_of_space`），白写一串。这与加甲之前一样，没有更坏；取的是不会误拒的那一边。注释写明了理由（`crates/singlefs-core/src/mounted_session.rs:6165`：「UserChange::CreateSnapshot => Self::ExemptOrNotKnownBeforeThePublish」那一臂的上文）。
- 甲没有判过头的变体：判的四类都是合取 7 一定判的（有普通分配，`crates/singlefs-core/src/admission.rs:1656` 的 `publish_has_an_ordinary_allocation`：数据单元、inode 树、extent 树节点）；会话的 `UserChange` 里没有回退、没有搬迁，`!is_a_rollback && relocations.is_empty()` 恒真。
- 推翻它的现象：找到一个 `UserChange` 变体，合取 7 免而甲判（那是甲误拒），或合取 7 判而甲免且推串之后合取 7 数反而回升（那是多白写一串，不误拒）。

### T2-3　先判乙后判甲的次序会不会让甲该报的被报成乙：会，但两种都是真话，且调用方现状不读错

- 次序：`crates/singlefs-core/src/mounted_session.rs:6230`：「if !devices_short.is_empty() {」先于 `crates/singlefs-core/src/mounted_session.rs:6238`：「if let Some(below_the_floor) = empty_cluster_segments_below_the_floor_before_any_series(」。两样同时成立时报乙（`devices_short` 非空、`empty_cluster_segments_below_the_floor` 是 `None`），甲那一份不出现。二选一靠构造路守：产品代码里构造这个成员的只有 `no_space_even_with_everything_a_floor_raise_could_release`（`crates/singlefs-core/src/mounted_session.rs:6256`）的两个调用（乙：`devices_short` 非空、`None`；甲：`Vec::new()`、`Some`），类型上没挡（测试夹具造了两样都空的，`mounted_session.rs` 测试段 `refusal_after_one_series_was_pushed`）。
- 谁读：产品代码里没有读这个成员的（量的：`grep -rn 'devices_short' crates --include=*.rs` 在核心层之外只命中验证包与运行库的测试）；运行库测试 `crates/singlefs-mount-runtime/tests/mount_runtime_accept_raises_the_floor_for_space.rs:1026` 读 `!no_space.devices_short.is_empty()`，先乙后甲就是为不改它；验证包模型读 `last_refusal`（`crates/singlefs-verification/src/history.rs:5599` 起），甲判出时 `last_refusal` 是空间不够那一种，模型归类仍是单元区墙，不变。
- 一个口径裂缝：同一个事实「全空段少于 N₀」现在有两条报法。合取 7 是头一次被拒的那一判时，交回 `UserChangeRefused::Publish { cause: EmptyClusterSegmentsBelowTheFloor }`；落得下那一判先拒、甲在推串之前判出时，交回 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease` 里的 `Some`。`crates/singlefs-core/src/mounted_session.rs:2120` 的既有注释：「抬 F 把 defer 里的槽回收不改这个数，推了只多发几次空发布」，同一段的后半句承诺调用方分得出「先整理」与「真没空间」，现在要读两个成员才分得出。没有产品调用方，所以没出错，但接 ENOSPC 的调用方只读成员名会把「先整理」当「真没空间」。
- 推翻它的现象：运行库或 FUSE 层写了按成员名映射错误码的代码，且把 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease` 一律映成「真没空间」。

### T2-4　判的数是不是合取 7 在写那一刻读的同一个数：判的时候是同一个，推串之后会不会变没证明

- 同一个读法：甲读 `crates/singlefs-core/src/mounted_session.rs:6203`：「.map(|device_map| (device_map.device, device_map.empty_segments()))」与 `crates/singlefs-core/src/mounted_session.rs:6210`：「.empty_cluster_segments_floor」，合取 7 读 `crates/singlefs-core/src/transaction.rs:13475`：「.map(|device_map| (device_map.device, device_map.empty_segments()))」与 `crates/singlefs-core/src/transaction.rs:13482`：「.empty_cluster_segments_floor」，都交 `admit_the_empty_cluster_segments_floor_on_every_device`（`crates/singlefs-core/src/admission.rs:947`）。开关也同一个（`allocator.space_admission()`）。`empty_segments` 是 O(1) 的增量数（`crates/singlefs-core/src/allocator.rs:1142`），记录口径。
- 被拒那一次发布之后到甲判之前分配器没动：判的函数收 `&PoolAllocator`（`crates/singlefs-core/src/transaction.rs:13387` 起的 `judge_the_publish_before_any_read_or_write`），整次发布失败还会换回克隆（`crates/singlefs-core/src/transaction.rs:13212`：「let allocator_before_this_publish = allocator.clone();」）。
- 没证明的：甲判完、不推串，所以不存在「判时到真提交」之间的间隙；但甲的理由是「推了这次写也是被合取 7 拒」，只论证了回收不改这个数（`crates/singlefs-core/src/mounted_session.rs:6174`：「抬 F 回收 defer 里的槽不改这个数（defer 里的已释放槽本来就不算占用），推了这次写也是被合取 7 拒。」）。串里自己的空发布还会分配新的提交内生块（可能开新聚簇段，数减一）并释放换下的固定点单元（某个已满的聚簇段里最后一条未释放记录若正是被换下的那个，数加一，记录口径下 defer 槽不算占用）。数加一而恰好补到 `N₀`，则推了写得进，甲误拒（推的，没量过）。要成立得有一个已满的段里只剩会被空发布换下的固定点单元；实现员报告没说，A/B 也没造。
- A/B 对甲同样没有区分力（量的，任务 13802 日志，字段取法同 T1-5）：

      drawn arm=conjunct7-first oracle_published_after_a_series=0 c_published=0 c_fired=9562 c_white=19165 fired_by_conjunct7=315 published_eaten_vs_as_coded=0
      series-admits arm=conjunct7-first oracle_published_after_a_series=5465 c_published=5740 c_fired=604 c_white=1345 fired_by_conjunct7=0 published_eaten_vs_as_coded=0

  原那一族没有推串后写得进的格，甲判了 315 次但无从吃掉；新造那一族甲一次都没判到（`fired_by_conjunct7=0`）。所以「甲不误拒」没被量过。
- 推翻它的现象：造一个池，全空段数恰为 `N₀ - 1`，且该数在被拒的写之前的一串空发布里加一（被换下的固定点单元是某个已满段的最后一条未释放记录），推串后同一个写成功；或证明空发布的换下集合里不可能有这样的单元。

## T3　覆盖与报

### T3-1　`publish_user_change` 与走空两处传哪一种没有真池用例：推不出走空与 `publish_user_change` 的接线对，其中走空那一处接错是有害的

- 真池用例只有两条，都走 `accept`：`crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs:1184`：「fn an_accept_short_on_slot_pairs_that_releasing_everything_would_not_complete_is_refused_without_a_series(」（乙）与 `crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs:1342`：「fn an_accept_while_empty_cluster_segments_are_below_the_floor_is_refused_without_a_series()」（甲）。其余把 `ForcedForTests` 设上的测试目标（量的，`grep -rln 'ForcedForTests(' crates --include=*.rs` 命中十个文件，除这一个全是整理类用例）不碰先算。
- 核心层单测直接把枚举值喂给 `judge_before_pushing_a_floor_raise_for_a_refusal`（`crates/singlefs-core/src/mounted_session.rs:8726`：「fn a_foreground_write_refused_while_empty_cluster_segments_are_below_the_floor_is_not_pushed_for(」），变体到枚举值的映射另有一条（`crates/singlefs-core/src/mounted_session.rs:8841`：「fn data_writes_and_new_inodes_are_foreground_writes_and_the_other_changes_are_not_judged_before_pushing(」）。三个调用点把哪个值传给先算，只有 `accept` 那一处被真池用例罩住。
- 变异行：新加的 15 行里点接线的只有 `crates/mutations.tsv:7705`：「抬 F 先算：槽对那一判的上界改回每个放得开的槽各补一对（accept 真池上推一串）」与 `crates/mutations.tsv:7706`：「抬 F 先算：推串之前不判合取 7（accept 真池上推一串）」，加上映射那一行 `crates/mutations.tsv:7702`：「抬 F 先算：并进一次发布的数据写类当合取 7 免」。没有一行改 `crates/singlefs-core/src/mounted_session.rs:3845`（`publish_user_change` 传 `of_the_user_change(change)`）或 `crates/singlefs-core/src/mounted_session.rs:6424`（走空传 `ExemptOrNotKnownBeforeThePublish`）。把前者改成永远免、把后者改成永远判，按现有用例我读不出哪一条会红（推的，没跑变异）。
- 后者的后果不小：走空里被拒的是删除类的一步（合取 7 免，`crates/singlefs-core/src/transaction.rs:13367`），若误传「判」，全空段少于 `N₀` 时 unlink 的走空步会被先算拦下、不推串，而推串本来能让删除类步放得下。这是接线错的最坏形。
- 要补的两条真池用例（交主 agent 定，不在我写范围）：`publish_user_change` 写文件内容被拒、全空段少于 `N₀`、一串不推；挂载续做留下队列（unlink 意图已发成、待走空）、走空步被空间拒、全空段少于 `N₀`，仍推一串。再各加一行点名这两个接线点的变异。

### T3-2　甲那条真池用例用 `ForcedForTests` 把 `N₀` 设成每块盘全空段数最大值加一，会不会罩不到产品 `N₀` 下的形：逻辑上罩得到，产品 `N₀` 在池级的形没有用例，推不出会漏

- `N₀` 只进一个比较：甲读 `.empty_cluster_segments_floor.segments()`（`crates/singlefs-core/src/mounted_session.rs:6210`），交 `admit_the_empty_cluster_segments_floor_on_every_device`（`crates/singlefs-core/src/admission.rs:947`）判「全空段数 < N₀」。池级用例的 `N₀` 取 `crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs:1375`：「let floor_in_segments = empty_cluster_segments_on_each_device」起的 max + 1（经 `crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs:1381`：「force_the_empty_cluster_segments_floor(&mut session, floor_in_segments);」装上）。数值不同，走的是同一条路径。
- 产品 `N₀`（1）在函数级有用例：`crates/singlefs-core/src/mounted_session.rs:8708`：「fn allocator_without_an_empty_cluster_segment_and_its_refusal() -> (PoolAllocator, PublishError)」建的 `PoolAllocator::new` 取整理开关的产品档（`crates/singlefs-core/src/compaction_plan.rs:811`：「switches: CompactionSwitches::PRODUCT,」），全空段 0 个，甲判出，`floor_in_segments` 是 1。
- 池级缺的是「产品 `N₀` 下 `empty_segments` 真降到 0 又同时落得下那一判先拒、槽对上界够得上」这个形能不能在满池里出现。我读码判能（段里有一条未释放记录就不算全空，384 槽盘只有六段），但没造；`ForcedForTests` 造的是「全空段多、`N₀` 更大」，段外槽对的分布与产品形不同，所以池级只证了接线与不动盘，没证分布。
- 推翻它的现象：产品 `N₀` 的池里造不出「全空段为 0、槽对上界够得上需求、落得下那一判先拒」的形（那样甲在产品里是死代码，用例只能靠强制开关）。

### T3-3　`DeviceFreeMap` 每槽一位的新位图与准入 O(聚簇段数) 读法有没有违 `fs-design.md`「记账是事务的副产品」：未违它的主禁令；新位图把既有的「每次发布整份克隆分配器」的容量线性代价加重一档，这一档该不该算违，推不出

- 规则（`.claude/rules/fs-design.md@记账是事务的副产品，不是事后的遍历`）：「| **运行时决策路径**（分配、ENOSPC 准入、defer 窗口、生命周期判定） | **不许**，且代价不许随盘容量增长 |」。乙的读法 `crates/singlefs-core/src/allocator.rs:500`：「fn pairs_inside_the_cluster_segments(」只遍历 `cluster_segments`（这次挂载开过的段），不扫单元区；维护是每个槽 O(1)（`crates/singlefs-core/src/allocator.rs:881` 起）。不是事后遍历，是事务副产品。
- 内存：`crates/singlefs-core/src/allocator.rs:409`：「is_under_an_unreleased_record: Vec<bool>,」与既有三张同形（`crates/singlefs-core/src/allocator.rs:364`：「allocated: Vec<bool>,」、387 的 `isolated: Vec<bool>,`、395 的 `held_until_floor_takes_effect: Vec<bool>,`）。
- 加重的一档：每次发布整份克隆分配器（`crates/singlefs-core/src/transaction.rs:13211`：「整个分配器拷一份、失败就换回去：第一版每次发布约 450 KiB 的拷贝」，下一行 13212 就是 `allocator.clone()`），克隆里每块盘四张 `Vec<bool>`（算的：三张加新的一张，一槽一字节）。`SLOT_BYTES` 是 16384（`crates/singlefs-format/src/lib.rs:14`：「pub const SLOT_BYTES: u64 = 16384;」），1 TiB 的盘是 2^26 个槽（算的），每张位图 64 MiB，每次发布每块盘多克隆 64 MiB。这条代价既有三张位图时就随盘容量线性涨，是否违规规则的字面该由主 agent 判；新位图使它多约三分之一。
- 另一种写法（推的，没验）：「罩在未释放记录下」= 分配位置着且不在 defer 里；defer 里的槽数少、可用区间集合记，这样不加每槽位图，只在 `mark_released`、`mark_resurrected_out_of_the_defer_queue`、`mark_reclaimed` 里改集合，成对翻转的判法改成查集合。这条不需要为本轮采纳。

## 检查单八格

| 格 | 判定 | 证据与保留 |
|---|---|---|
| 走不到的理由二次查实 | 成立 | `crates/singlefs-core/src/allocator.rs:514`：「聚簇段起点在段边界上」。聚簇段只在 `crates/singlefs-core/src/allocator.rs:2910` 插入，插入的是各盘一致答出来的段起点：各盘自己的答案是 `crates/singlefs-core/src/allocator.rs:1251`：「self.unit_area_start.slot().0」加段号乘 64，答案不同走 `crates/singlefs-core/src/allocator.rs:2898`：「DeviceAgreement::AnswersDiffer(answer_per_device) => {」拒，所以插进去的每一个起点在每块盘上都等于自己算出的段边界；单元区起点由挂载按选中的系统配置给所有盘同一个（`crates/singlefs-core/src/mount.rs:3242` 一族，我只读了这一处）。两个调用者只传 `allocator.cluster_segments()`（`crates/singlefs-core/src/admission.rs:995` 与 `crates/singlefs-core/src/mounted_session.rs:6059`）。新减法三处：`crates/singlefs-core/src/allocator.rs:890`、`crates/singlefs-core/src/allocator.rs:891`、`crates/singlefs-core/src/allocator.rs:787` 的「总数减聚簇段里的和」，前两处只在「这一对此前计数、现在不计」时减，后一处总数不小于任意子集的和；仓根 `./Cargo.toml:24`：「overflow-checks = true」下破了不变量会当场 panic 不会回绕。推翻：造出一个聚簇段起点不在某块盘单元区段边界上的历史。 |
| 类型与名字 | 不成立（一处字面打中） | 槽数与槽对数共用裸 `u64`：`crates/singlefs-core/src/mounted_session.rs:6055`：「slot_pairs_once_everything_is_released: u64,」与同一结构里的 `deferred`、`isolated`（槽数）；`.claude/rules/implementation-workflow.md` 类型与名字格写「同一个量不同语义不共用裸整数（newtype）」。同形的既有写法：`crates/singlefs-core/src/allocator.rs:400`：「unblocked_slots: u64,」与 `crates/singlefs-core/src/allocator.rs:406`：「unblocked_slot_pairs: u64,」、`crates/singlefs-core/src/admission.rs:980`：「pub slot_pairs_for_data_units: u64,」与 `crates/singlefs-core/src/admission.rs:983`：「pub slots_for_commit_generated_units: u64,」，换算处 `crates/singlefs-core/src/mounted_session.rs:6089`：「slots.slot_pairs_once_everything_is_released * slots_of_a_data_unit,」写对了。`of_the_user_change` 无 `_ =>`（量的，补丁新加行里唯一的 `_ =>` 在测试的随机历史 `match next_random(9)` 上，对象是整数，无从穷举）。新 `expect`：产品代码里新写的是建空闲图那一处 `expect("单元区槽数")`（照既有写法，写的是量名不是依赖的不变量），段边界那三处是从既有函数原样搬来的、文字带不变量。测试里 `expect("个数")`、`expect("下标")` 过简。 |
| 不兼容旧代码与旧格式 | 成立 | 补丁三个文件里新加行没有版本号、旧布局、迁移或只为旧数据放宽的读法（量的：对改动全文 `_m4-gpugc-precheck-b-a-code-r1-diff.md` 的加号行跑 `grep -E '迁移|版本号|VERSION|version|layout|布局'`，只命中 `versions_published`、常量 `EMPTY_CLUSTER_SEGMENTS_FLOOR_OF_THE_FIRST_VERSION` 与变量名 `content_of_the_current_version`；跑 `grep -E '原先|此前|原来|以前|改成|换成|起初|最初|后来|旧|兼容|legacy|已改|本来'`，只命中「旧内容」（崩溃前盘上读得出的旧文件内容，不是旧格式）与「本来」（语义是「原本就」，不是改动经过））。没碰盘上字节：改动只涉及内存里的位图与计数。 |
| 不新建判定库 | 成立 | 加号行里没有 sqlite、RocksDB、TSV、prune-server（`grep -ciE 'sqlite\|rocksdb\|\.tsv\|prune-server'` 对加号行输出 0，量的）。变异表 `crates/mutations.tsv` 是变异行登记不是判定存放处。 |
| 所有任务多机可用 | 成立（限于我读到的加号行） | 加号行里没有 `std::env`、`/home`、`.local/share`、`/tmp`、`std::fs`、`Command::new`、网络（`grep -nE` 对加号行只命中 `DataUnitIndexInFile::` 的 `File::` 误报，量的）；新用例输入都在用例里现造（内存分配器、`MemoryPool`/稀疏盘）。池脚手架 `common_admission_pool` 的内部我没读。 |
| 测试优先多线程 | 不成立（文字上打中，实质不该并行） | 两条分配器历史是单线程 600 步（`crates/singlefs-core/src/allocator.rs:3932`：「for step_number in 1..=600 {」）。每一步的状态依赖上一步，次序本身是被测对象，不该切片；毫秒级，不需要多线程。但代码注释与测试说明里没写「为什么不并行」（量的：加号行里 `grep -E '并行|线程'` 命中 0）。真池两条是单场景。 |
| 带 --item 的命令与项 | 成立（命令与项齐，结局待） | 三项都在登记表里：`.claude/gate.d/crash-point-replay-items.tsv:18`：「floor-raise-pushed-by-the-session」、`.claude/gate.d/crash-point-replay-items.tsv:70`：「truncate-to-zero-on-a-nearly-full-pool」、`.claude/gate.d/crash-point-replay-items.tsv:76`：「last-fitting-unit-then-refusal-and-unmount-on-a-nearly-full-pool」。实现员报告「要主 agent 跑的命令」一节里讲崩溃点重放的那一条只写了 `--list-items`，例名 `crash_enumeration_floor_raise_pushed_by_admission` 是测试文件名不是登记项名（该名在登记表里 0 次，量的）；整条 `--item` 命令是主 agent 交的：任务 13913 的命令（`task-server/task status 13913` 打出）`bash scripts/crash/crash-point-replay.sh --crash-expand-up-to 4 --item floor-raise-pushed-by-the-session --item last-fitting-unit-then-refusal-and-unmount-on-a-nearly-full-pool --item truncate-to-zero-on-a-nearly-full-pool`，三项齐。13913 我读时还在跑，缩了展开深度，不算门禁通过。这一批没有新项（登记表不在补丁里）。 |
| 新崩溃流多罩了什么 | 成立（不需要新流） | 乙、甲新拒的形写之前就拒，不发任何写，没有新的崩溃点可枚举；池级用例已钉盘上逐字节不变与崩溃后挂载读得出旧内容（`crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs:1342` 那条与乙那条都经 `nothing_was_pushed_and_the_old_content_reads_back_after_a_crash`）。登记表里与准入相关的现有流（项 18、70、76）测的是推串中途或拒写之后，乙、甲只可能让其中某一次推串变成直接拒（态数会变，不会多出新崩溃点）；`floor-raise-pushed-by-the-session` 对应的测试直接调 `push_one_floor_raise_within_the_admission_budget`，不经先算。 |

## 额外读到的、不属于上面某一格的两处

- 过期的现状注释：`crates/singlefs-core/src/transaction.rs:13211`：「整个分配器拷一份、失败就换回去：第一版每次发布约 450 KiB 的拷贝」，数字按「两盘各一张位图」写，今天每块盘是四张（加了新位图之后）；`crates/singlefs-core/src/mounted_session.rs:2120` 既有注释对合取 7 的描述（回收不改这个数、调用方分得出两种）在甲上只剩前半句（T2-3）。
- C1002 的去向格：`.claude/kb/checks-owed.md:641` 的去向写「分配器按段增量维护 defer 里成对的数据单元与段外单槽的计数」，与代码实际数的「段外两槽都不罩在未释放记录下的槽对」同向但口径更宽（含隔离与扣住）。改去向时用代码的说法（T1-0）。

## 要主 agent 跑的命令

没写模型，没有新的交任务命令。可由主 agent 直接核我引的读数的只读命令（不交服务）：

    grep -nE 'unreleased_per_segment\[|is_under_an_unreleased_record\[|slot_pairs_without_an_unreleased_record(_per_segment\[segment\])? [-+]?=' crates/singlefs-core/src/allocator.rs
    grep -nE 'cluster_segments\.(insert|remove|clear|retain)|cluster_segments = ' crates/singlefs-core/src/allocator.rs
    grep -c '合取 7' .claude/kb/decisions/16-发布语义.md
    grep -n 'push_one_floor_raise_for_a_refusal_for_space' crates/singlefs-core/src/mounted_session.rs
    task-server/task log 13802 --all   # 看 PROBE_RESULT 与 FOLLOWED 行，字段取法见 T1-5
    task-server/task status 13913      # 崩溃点重放缩域那一趟的结局，我读时还在跑

读结局看：13913 的末行 `TASK_SERVER_RESULT` 的 `exit=0`；它缩了展开深度，只证三项在缩域下没红，不是门禁通过。

等结果才能往下做的、我没做的（需要新派腿或实现员，不在我的写范围）：

- 补两条真池用例与两行变异，罩 `publish_user_change` 与走空两个接线点（T3-1）。
- 量甲会不会被串里自己的空发布改数误拒（T2-4 推翻条件）；量乙的重挂出路（T1-6 推翻条件，要造「聚簇段里还剩空对、补上就够」的池）。两件都得造池，探针几何要改，A/B 的原探针造不出。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1-0 代码是不是用户定的乙 | 成立 | 数的是段外两槽都不罩在未释放记录下的槽对，比用户原话多含隔离与扣住，与 A/B 乙臂同口径 |
| T1-1 增量计数每条路跟上 | 成立 | 位与两个计数只在 `note_slots_under_an_unreleased_record` 一条链里改，三个入口覆盖分配、释放、复活、重建，回收与挡位变动本不该动它 |
| T1-2 分配位着而不罩未释放记录即 defer | 成立（读码） | 分配位只在 `mark_allocated` 置、`mark_reclaimed` 清，回收只收已释放记录 |
| T1-3 扣住的槽按已释放算放得开 | 成立 | 扣住只加在回收出来的槽上，记录不删，未释放位为假 |
| T1-4 与探针口径差 | 可达状态上相等（推的） | 只差「没有记录而置了扣住位」的槽，读码造不出 |
| T1-5 乙算小（误拒） | 读码找不到这样的形；A/B 没有区分力 | 聚簇段只增、串自己释放的回收不到、串自己的分配只加记录；原那一族无推串后写得进的格 |
| T1-6 4801 格丢的出路 | 读码上丢得了重挂一条，量的数里没丢到过，4801 格里被跟过几格不知道（上限 295） | 聚簇段集合挂载内、重挂清零；乙臂 lost-first 类跟了 295 格，写进 0；新造那一族不报出路的格重挂救 148÷343 |
| T1-7 后一串回收固定点单元 | 走不到（推的） | 一次准入至多 8 次发布，环长至少 12，追不上串里的释放代 |
| T2-1 射程与「推串之前」 | 条款没写甲；读法与三处范围一致，与「头一次被拒」差一形 | kb 16 里「合取 7」「推串之前」都是 0 次；要补条款 |
| T2-2 该判而漏的前台写入 | 有两种漏，都在安全侧 | 变长的 Truncate 与 CreateSnapshot 合取 7 判而甲不先判，后果是白推一串 |
| T2-3 先乙后甲的次序 | 乙可盖住甲，两样都是真话；现无产品调用方读 | 同一事实现有两条报法，注释承诺部分失效 |
| T2-4 判的数是同一个数 | 判的时候是；推串之后会不会变没证明（推的） | 串里自己的空发布可能让全空段数加一，甲会误拒；A/B 没造 |
| T3-1 两处接线无真池用例 | 不成立（没罩住） | 变异表没有点这两个接线点的行；走空接错的后果是拦下删除类步 |
| T3-2 `ForcedForTests` | 逻辑上罩得到，池级产品 `N₀` 的形推不出 | `N₀` 只进一次比较；函数级有产品档用例，池级没有 |
| T3-3 新位图与 fs-design | 未违主禁令；克隆代价加重一档，是否违推不出 | 读 O(聚簇段数)、维护 O(1)；每次发布克隆多一张每槽一字节的位图 |
| 检查单：走不到的理由 | 成立 | 插入的起点是各盘一致答出的段边界；新减法受不变量守、`overflow-checks` 开 |
| 检查单：类型与名字 | 不成立（字面一处） | 槽数与槽对数共用裸 `u64`，与既有同形；无 `_ =>`，`expect` 一处写量名 |
| 检查单：不兼容旧代码与旧格式 | 成立 | 加号行没有版本、迁移、旧布局，没碰盘上字节 |
| 检查单：不新建判定库 | 成立 | 加号行无 sqlite、RocksDB、TSV、prune-server |
| 检查单：所有任务多机可用 | 成立（限读到的加号行） | 无环境、路径、网络；池脚手架内部没读 |
| 检查单：测试优先多线程 | 不成立（文字上） | 次序即被测对象，不该并行，但没写理由 |
| 检查单：带 --item 的命令与项 | 成立（命令与项齐，结局待） | 三项在登记表，13913 命令整条带三项；13913 在跑、缩了域 |
| 检查单：新崩溃流多罩了什么 | 成立（不需要新流） | 新拒的形不发写，无新崩溃点；池级用例钉了盘上不变与重挂读得出 |

## 没做什么

- 没起编译、没跑测试、没往统一任务服务交任务（子 agent 不交）；全部读数是只读命令或主 agent 已跑任务的日志（13802、13913 的状态）。判决要引的数没复跑，我转述的 A/B 数都出自 13802 的日志原样字段，由主 agent 复跑才能引。
- 没写模型、没拷仓副本、没建编译目录。草稿目录 `~/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r1-sonnet/` 里只有我存下的任务日志与引文核对输出，`du -sh` 是 304K，没有仓副本与编译目录，无需清理。
- 没读禁读清单里的任何东西；没读池脚手架 `common_admission_pool` 的内部、没读 `crates/singlefs-core/src/mount.rs` 里除 `with_unit_area_start` 调用那一处之外的代码。
- 乙在槽对那一判上的误拒、甲被串里自己的空发布改数误拒、乙丢的重挂出路，都是读码推的，没造池量过；它们的推翻条件写在各节末。
- 没替主 agent 采纳或写判决；没改 kb。射程条款与 C1002 去向要补，是主 agent 的写回。
