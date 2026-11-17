# d3-fallback-richest-leaf-code-r2 攻方腿（Opus）报告（2026-11-13）

开工核对：`sha256sum -c research/prompts/d3-fallback-richest-leaf-code-r2-snapshot/crates-sha256.txt` 过滤掉 `: OK` 之后零行，没有对不上的文件。行号都是此刻主工作区的（与快照逐字节相同）。

没写模型目录：这一条腿的「模型」是仓副本里加的五条单测（副本 `/home/fy5090/.local/share/singlefs/work/d3-fallback-richest-leaf-code-r2-opus/copy`，只改了 `crates/singlefs-core/src/allocator.rs`，`cargo fmt` 过；与主工作区的 diff 在 `/home/fy5090/.local/share/singlefs/work/d3-fallback-richest-leaf-code-r2-opus/copy-allocator.diff`）。没调崩溃枚举函数，跑过的崩溃状态数 0。项目代码一条都没跑，下面凡是「跑出来会怎样」都是推的，命令在末节。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 攻击面 1：落点真的一个不变吗 | 产品路径上没打中；函数入参上打中（轻） | 三样对齐里，单元区起点与 `R` 靠类型不变量；聚簇段起点只靠调用约定（私有字段、唯一一处插入），`pub` 函数的入参既无类型也无断言。造得出新旧分叉的输入（聚簇段起点不在 64 槽格子上），产品可达性推的是 0 |
| 攻击面 2：对拍与增量维护的用例撑不撑得住 | 打中（方法，不改结论） | 正文把头一趟没红的那条变异说成「挡上一侧不摘段」，任务 14606 日志里没红的是「放开一侧不加回」；与它同种子同几何同步数的那条落点对拍用例因此量得出一次都没走到「一段离开集合又回来」，而那正是新取法唯一依赖集合维护的那一步 |
| 攻击面 3：集合的每一条维护路径 | 没打中 | 三张位图只经 `set_blocking_bit` 写，集合与每段槽对数在同一处按同一个转移维护；建空闲图、克隆、挂载重建、回退复活、整理搬走都落到这一处或整份拷贝；另一个会话的 `SlotPairCount` 替换没碰这两个字段 |
| 攻击面 4：新拒绝成员的分法 | 没打中 | 上层四处分流（发布层、会话层抬 F、挂载、模型对拍）对新旧两个成员一视同仁；`Unexplained` 在对拍里一律判分歧，不吞 |
| 检查单：走不到的理由 | 成立 | 新加的 `assert!` 都在 `mod tests` 里；`lowest_unblocked_slot_pair_in_the_segment` 交 `None` 那一支在产品上走不到，走到了也只是多看一段、答案不变（下文 2c） |
| 检查单：类型与名字 | 打中 | 产品代码新加的 `.expect("64")`（`allocator.rs:1345`）消息只写一个数；段号用裸 `usize`，在 `set_blocking_bit` 里与位图下标同为 `usize`、互换编得过 |
| 检查单：不兼容旧代码 | 成立 | `#[cfg(test)]` 的逐候选取法是对拍参照，不在任何产品路径上，也不认旧格式 |
| 检查单：不新建判定库 | 成立 | diff 里没有存判定的东西 |
| 检查单：多机可用 | 成立 | 新测试只在进程里造空闲图 |
| 检查单：测试优先多线程 | 打中（轻） | 增量维护那条用例是两个场景（定形三段盘、385 槽随机历史）拼在一条里，各建各的状态，没写为什么非拼不可 |
| 检查单：带 --item 的命令与项 | 成立（推的） | 落点在全部可达输入上相等（攻击面 1 的证明），崩溃枚举流的写流不变；要回归可点三项（下文） |
| 检查单：新崩溃流多罩了什么 | 成立 | 集合只住内存、挂载重建经同一处维护，不带出新的崩溃状态 |
| 第 3 条（分叉输入） | 造得出、产品走不到 | 单测全文 `/home/fy5090/.local/share/singlefs/work/d3-fallback-richest-leaf-code-r2-opus/unit-test-off-grid-cluster-segment-start.rs`，已放进副本，预期红（推的） |

## 攻击面 1：第一轮的修补本身——落点真的一个不变吗

### 1a 相等的论证（正推，读码）

新取法按段号升序走集合，跳过两种段：「let is_open_to_user_data = !cluster_segments.contains(&segment_start)」（`crates/singlefs-core/src/allocator.rs:1329`）与「&& !excluded_region.is_some_and(|region| region.contains_slot(segment_start));」（`crates/singlefs-core/src/allocator.rs:1330`）；没跳过的交「self.lowest_unblocked_slot_pair_in_the_segment(*segment_index)」（`crates/singlefs-core/src/allocator.rs:1332`）。参照取法对每个候选问「candidate >= segment.0 && candidate < segment.0 + CLUSTER_SEGMENT_SLOTS」（`crates/singlefs-core/src/allocator.rs:1376`）。

记单元区起点 U，候选 c（偶数）所在段的起点 s(c) = U + 64·⌊(c − U)/64⌋。两种取法答案相同，当且仅当下面三条成立：

1. 聚簇段集合里每个起点 g 满足 g ≡ U (mod 64)：这时 c ∈ [g, g + 64) ⟺ g = s(c)，「候选在聚簇段里」等于「段起点在集合里」。
2. `R` 恰好是 64 槽格子上的一整段：这时 c ∈ R ⟺ s(c) ∈ R。
3. 有没挡槽对的段集合 ⊇ 真有没挡槽对的段（多了不要紧：多出来的段在 `lowest_unblocked_slot_pair_in_the_segment` 里答 `None`，`find_map` 接着走下一段，答案不变，只多看一段）。

有了这三条，参照取法第一个接受的候选落在「没跳过、又有槽对的最低段」里那一段最低的一对上，新取法交的就是它。一对不跨段：段长 64 是偶数、U 在段边界上，对从偶数下标起。

### 1b 三样各由什么保证

| 前提 | 保证它的东西 | 强弱 |
|---|---|---|
| 单元区起点在段边界上 | 「pub struct UnitAreaStart(SlotNumber);」（`crates/singlefs-core/src/allocator.rs:375`）字段私有，唯一构造函数判「if !slot.0.is_multiple_of(CLUSTER_SEGMENT_SLOTS) {」（`crates/singlefs-core/src/allocator.rs:393`）；`DeviceFreeMap` 只经 `with_unit_area_start` 收它 | 类型 |
| `R` 是一整段、起点 64 的倍数 | 「/// 整理意图的区域 R：一块盘上一个 64 槽对齐的聚簇段（起始槽 64 对齐、槽数 = 64」（`crates/singlefs-core/src/logged_operations.rs:114`）；唯一构造函数判「if !start_slot.0.is_multiple_of(CLUSTER_SEGMENT_SLOTS) {」（`crates/singlefs-core/src/logged_operations.rs:211`）；从盘上读意图也走它：「let region = match CompactionRegion::of_one_cluster_segment(device, start_slot) {」（`crates/singlefs-core/src/logged_operations.rs:738`）；`contains_slot` 只罩 64 槽：「slot.0 >= self.start_slot.0 && slot.0 < self.start_slot.0 + CLUSTER_SEGMENT_SLOTS」（`crates/singlefs-core/src/logged_operations.rs:274`） | 类型（Verus 类型不变量加私有字段） |
| 聚簇段起点在格子上 | `PoolAllocator` 的私有字段起步是空集「cluster_segments: BTreeSet::new(),」（`crates/singlefs-core/src/allocator.rs:2355`），全文件唯一一处写「self.cluster_segments.insert(segment_start);」（`crates/singlefs-core/src/allocator.rs:3073`），插的是各盘一致答的「Some(CommitGeneratedDeviceAnswer::OpenEmptySegment(segment_start))」（`crates/singlefs-core/src/allocator.rs:1475`），它来自「match self.lowest_empty_segment_outside_the_region(excluded_region) {」（`crates/singlefs-core/src/allocator.rs:1473`），段起点由「+ u64::try_from(segment_index).expect("段号") * CLUSTER_SEGMENT_SLOTS,」（`crates/singlefs-core/src/allocator.rs:1207`）算 | 只靠调用约定 |

`R` 不罩几段、也不会不满一段：槽数恒 64（`slot_count` 写死），所以「`R` 跨段」「`R` 不满一段」两种形状在类型上造不出来。「段起点在 `R` 里」与逐候选判在全部可造的 `R` 上相等。

单元区不满整段的尾巴：每段起步槽对数是「(unit_area_slots - first_slot_of_the_segment).min(CLUSTER_SEGMENT_SLOTS) / 2,」（`crates/singlefs-core/src/allocator.rs:637`），尾段 r 槽记 ⌊r/2⌋ 对，r = 1 时不进集合；段内找对的上界截在「(first_index_of_the_segment + slots_per_segment).min(self.allocated.len());」（`crates/singlefs-core/src/allocator.rs:1348`），凑不成对的末槽由「first_index_of_the_pair + 1 < self.allocated.len()」（`crates/singlefs-core/src/allocator.rs:742`）答否；参照取法同样不看末槽：「while candidate + 1 < end {」（`crates/singlefs-core/src/allocator.rs:1374`）。聚簇段不会是尾段（开段只找整段：「let full_segments = self.allocated.len() / per_segment;」（`crates/singlefs-core/src/allocator.rs:1405`））；`R` 落在尾段起点时整段罩住尾段，两种取法都整段跳过。尾巴上不分叉。

挂载重建：`cluster_segments` 只住内存、重建时是空集；`R` 从盘上读回走 `of_one_cluster_segment`（上表）。整理意图续做读回的也是同一个类型。这两条路径都放不进不对齐的起点。

### 1c 打中的那一处：聚簇段起点的前提不在入参上

同一份文件里，读同一个集合的兄弟函数把这条前提写成断言：「offset_in_slots.is_multiple_of(CLUSTER_SEGMENT_SLOTS),」（`crates/singlefs-core/src/allocator.rs:592`），违反时 panic。`lowest_user_data_slot_outside_the_region` 是 `pub`、收任意 `&BTreeSet<SlotNumber>`，前提违反时不报、静默交出与参照取法不同的落点。调用点（`grep -rn` 查 `crates/` 与 `scripts/` 下的 `.rs`）：产品一处传分配器自己的集合「let cluster_segments = &self.cluster_segments;」（`crates/singlefs-core/src/allocator.rs:2927`）；其余是测试与实验装置（e181、e185、e202 各传 `allocator.cluster_segments()`，`core_review_unit_area_start_and_publish_limits.rs` 传空集），都走不到不对齐的起点。

分叉输入（第 3 条要的那一个）：一块 192 槽的盘，第 0 段前 32 槽分配掉，聚簇段集合是 {U + 32}。参照取法把 [U + 32, U + 96) 当聚簇段，落在 U + 96 = 50272；新取法只认段起点相等，第 0 段照取，落在 U + 32 = 50208（推的，单测全文在「第 3 条」一节，预期红）。

四句：不分辨产品上的新旧两臂（产品路径交不出这个输入）；被判的函数看得到它（集合就是入参，兄弟函数已经在判）；满足的是攻击面 1 末句「有没有一条路径能把不对齐的起点放进 `cluster_segments`」的反面——放不进，但函数不守；改法「入参前提写成断言（照 `pairs_inside_the_cluster_segments` 那一条）」或「聚簇段起点立一个只由段号算出的类型」在这一格上都修得到，前一种改动最小（推的，只在我的推理上量过、被攻过零轮）。

什么会推翻 1a：主 agent 跑「第 3 条」一节那四条「格子上全输入对拍」，任何一条在任何一步报两种取法不同，或报「没有一段离开集合又回来」（取样点没走到）。

## 攻击面 2：产生结论的方法——对拍与增量维护的用例撑不撑得住

### 2a 正文点错了那条没红的变异（对不上就当场说）

正文第三节第 2 条写「那条头一趟没红的变异（挡上一侧不摘段）」。任务 14606 的日志（`task-server/task log 14606` 原样存在 `/home/fy5090/.local/share/singlefs/work/d3-fallback-richest-leaf-code-r2-opus/log-14606.txt` 第 7 行）是：

```
✗ 用户数据落点按段取：槽对放开时不把那一段加回有槽对的段集合	没红	槽对放开时不把那一段加回有槽对的段集合：allocator::tests::segments_with_an_unblocked_slot_pair_are_maintained_incrementally_and_match_a_scan 没红（退出码 0）（日志 /home/fy5090/.local/share/singlefs/work/user-data-placement-scan-writer/copy/prove-red-logs/run-2341564-c83099a7/003.log）（现判：行新加）
```

没红的是放开一侧（变异行「用户数据落点按段取：槽对放开时不把那一段加回有槽对的段集合」，`crates/mutations.tsv:7738`）；挡上一侧那一行「用户数据落点按段取：一段最后一对被挡上时不把那一段从有槽对的段集合里拿掉」（`crates/mutations.tsv:7747`）是第三份补丁新加的，不在 14606 那一批里。这是我核过的（日志行与变异表行），不是印象。两条变异的性质不同，判法要分开：

| 变异 | 集合变成什么 | 落点变不变 | 谁抓得到 |
|---|---|---|---|
| 放开一侧不加回（7738） | 少一段：一段槽对数 0 → 1 之后不在集合里 | 变：那一段被整段跳过，落点跳到更高的段 | 三条落点对拍（若历史走到 0 → 1）、集合对逐对扫 |
| 挡上一侧不摘段（7747） | 多一段：槽对数到 0 之后还在集合里 | 不变：那一段在 `lowest_unblocked_slot_pair_in_the_segment` 里答 `None`，`find_map` 接着走 | 只有集合对逐对扫；落点对拍永远抓不到 |

所以：7738 判成「取样点不敏感」在种类上对（它不是等价变异）；7747 是落点上的等价变异，实现员第三份报告第二节已写明，正文那一问的前提（「多留一个已经没有槽对的段，产品路径上会交出什么」）答案是：交出的落点不变，只多看一段，代价上界「「这次挂载开过的聚簇段数 + 2」段」（`crates/singlefs-core/src/allocator.rs:1318`）失守。

### 2b 打中：有一条落点对拍量得出没走到集合翻过 0

「取样点不敏感」的依据（14606 没红）同时说明了另一件事。增量维护用例的随机那一段与第一条落点对拍用例用的是同一个生成器、同一个种子、同一对几何、同一个步数：「let seed = 0x5EED_0000_0000_0385;」（`crates/singlefs-core/src/allocator.rs:5458`）与（`crates/singlefs-core/src/allocator.rs:5609`）两处，「[385, 385],」（`crates/singlefs-core/src/allocator.rs:5462`）与（`crates/singlefs-core/src/allocator.rs:5613`）两处，步数都是 1500（第 5463、5614 行）。7738 改坏之后，历史在第一次「一段从 0 翻回正」之前与原式逐步相同，翻回的那一步集合就与逐对扫不同、断言红；14606 没红 ⇒ 这段历史在两块盘上一次都没有「一段离开集合又回来」。由此：

- `user_data_slot_found_by_segment_matches_stepping_through_every_candidate_on_two_equal_devices_ending_in_an_unpaired_slot` 这条落点对拍，在新取法唯一依赖集合维护的那一步上覆盖为零（量的：由 14606 的日志行加两处种子、几何、步数逐字相同推出；推出这一步本身是读码）。
- 另两条（385 / 1000 不等大、2000 槽 3000 步）走没走到，没有量过（推的：生成器一半步数在分配、释放与回收各占一份，活单元一直少，占满一段的 32 对不容易；实现员第三份报告第一节同样是推的）。
- 增量维护用例的定形前缀钉了一形「第 0 段占满再放开最后一对」，那一形上落点有一条硬写的期望值（「"第 0 段最后一对被挡上：那一段离开集合"」，`crates/singlefs-core/src/allocator.rs:5590` 起那一段）；集合翻过 0 时新旧落点相等的对拍证据只有这一个形状。

四句：不分辨两臂（「落点不变」在产品输入上仍成立，攻击面 1 的证明不靠这三条用例）；判别子看得到（对拍函数每步都能数集合翻过 0 的次数，只是没数）；满足的是攻击面 2「对拍与增量维护的那几条用例撑不撑得住『落点不变』」——撑不住，结论靠的是读码证明；改法：「第 3 条」一节的四条「格子上全输入对拍」每一步都对集合、对两种取法，并断言「离开集合又回来」的次数大于 0，修的正是这一格（推的，待主 agent 跑；只在我的推理上过、被攻过零轮）。

### 2c `lowest_unblocked_slot_pair_in_the_segment` 找不到对时那一支

产品上走不到：集合按槽对数维护（攻击面 3），没被跳过的段在集合里就一定有对。走到了（集合多一段，即 7747 那种坏法）交 `None`，`find_map` 接着走下一段，落点不变；没有计数、没有断言，运行时看不出走了这一支。这一支掩盖了「集合多一段」这一类坏法，只留下代价上升。它不是「走不到的理由」那一格要登记的 `unreachable!`，检查单回成立；代价那一半没有会红的检查，见 2d。

### 2d 代价那半句没有会红的检查

文档写的代价上界（第 1317–1318 行）没有用例钉。把集合换成从 0 到段数的全段遍历（落点等价）、或 7747 那种多留段，落点对拍全绿；7747 只靠集合对逐对扫抓。用户要的是「优化算法」，换算法之后 E207 那一项快了多少，正文第二节自己写的是「实现员推的，没量」。调查报告 1c 量到的另两段线性查找（`allocator_release_linear_find`、`record_intersection_filter`，随记录数涨）这次没动，换算法之后它们会不会成为大头没量过（推的）。这一条不分辨「落点不变」，只记在方法上。

### 2e 生成器走不到的形态，以及各自为什么不分叉

| 走不到的形态 | 依据 | 为什么不分叉 |
|---|---|---|
| `R` 落在末尾不满的那一段、或只在大盘有的段 | 「let segment_index = next_random(smaller_unit_area / CLUSTER_SEGMENT_SLOTS);」（`crates/singlefs-core/src/allocator.rs:5399`） | `R` 整段罩住尾段，两种取法都整段跳过（1b） |
| `R` 在别的盘上 | 生成器的 `R` 写死盘 0 | `contains_slot` 只看槽号，两种取法同样不看盘 |
| 非默认环长的单元区起点 | 生成器只用 `DeviceFreeMap::new` | U 仍是 64 的倍数（类型），1a 的三条不依赖 U 的值 |
| 三块以上的盘 | 生成器两块盘 | 新取法逐盘独立，不读别的盘 |
| 挂载重建之后接着走 | 生成器只有克隆（「let cloned_allocator = pool.clone();」，`crates/singlefs-core/src/allocator.rs:5417`），没有 `rebuild_from_records` | 重建经 `mark_allocated` / `mark_released` 落到同一处维护（攻击面 3） |
| 扣住与隔离叠在同一槽上先后放开 | 生成器只在空槽上扣住（第 8 类操作先判 `is_free`） | `set_blocking_bit` 判的是三种位合起来翻没翻，叠几种位、按什么次序放开都一样 |
| `R` 与聚簇段重合或相邻 | 生成器靠运气 | 两种取法各自独立地跳过，重合时跳一次与跳两次同 |

「第 3 条」一节的四条单测把上表前三行与「叠位」「集合翻过 0」都做成了必走的取样点：直接改一块盘的三种挡位，`R` 与聚簇段取 64 槽格子上单元区下一段、单元区每一段（含尾段）、单元区上一段的全部组合。

## 攻击面 3：从未被看过的地方——集合的每一条维护路径（没打中）

不变量：每段槽对数 = 那一段里两槽都没挡的偶数起点对数；集合 = 槽对数不是 0 的段。它是 `set_blocking_bit` 的局部性质，不依赖调用方：

- 三张位图的写在全文件只有三处，都在「fn set_blocking_bit(&mut self, index: usize, bit: SlotBlockingBit, is_set: bool) {」（`crates/singlefs-core/src/allocator.rs:748`）里（`grep -n` 位图的下标赋值与整份赋值，零处在别的地方；实现员给的 grep 我另跑了一遍同样的结论）。改一位只影响它所在的那一对，那一对翻没翻由改前改后两次 `is_unblocked_pair_at_index` 判，翻了就同步改那一段的数：翻成挡上时减一、到 0 时「self.segments_with_an_unblocked_slot_pair.remove(&segment);」（`crates/singlefs-core/src/allocator.rs:776`），翻成没挡时加一并「self.segments_with_an_unblocked_slot_pair.insert(segment);」（`crates/singlefs-core/src/allocator.rs:782`）。插入是幂等的，k → k+1 时多插一次无害。
- 建空闲图：每段的数与集合由同一个向量算出（第 633–666 行），起步全空，两者同源。
- 克隆：`DeviceFreeMap` 与 `PoolAllocator` 都是派生的 `Clone`（「#[derive(Clone, Debug)]」，`crates/singlefs-core/src/allocator.rs:429`、第 2281 行），集合随整份拷贝。固定点演练在拷贝上走、真发在活分配器上走，两份起点相同；发布被拒时整份换回，集合跟着换回。
- 挂载时从盘上重建：`rebuild_from_records` 先 `new` 再逐条 `mark_allocated`、已释放的再 `mark_released`（第 2680、2682 行），前者经 `set_blocking_bit`，后者不改挡位。
- 回退：管理员回退复活 defer 里的记录走 `mark_resurrected_out_of_the_defer_queue`，它断言分配位本来就置着、只改未释放记录那一套，不碰挡位；按根回退（推的，没逐行核：整份重建分配器），同上一条。
- 整理搬走：目的地经 `record` → `mark_allocated`，源的释放与回收经 `mark_released` / `mark_reclaimed`，后者逐槽经 `set_blocking_bit` 清分配位。
- 隔离、清隔离、扣住、放开扣住：`isolate_slot_at_index`、`clear_isolation_of_slot`、`hold_until_floor_takes_effect`、`release_holds` 四处都只经 `set_blocking_bit` 改位。
- 另一个会话的 `SlotPairCount` 替换：diff 材料第 169–171 行是上下文行，`unblocked_slot_pairs_per_segment: Vec<SlotPairCount>` 在这一批之前已是这个类型；那个会话换的是读侧返回值（`unblocked_slot_pairs_outside_the_cluster_segments`）与 `admission.rs`。`.0 -= 1`、`.0 += 1` 与 `== 0` 的判法与它无交叉。减一不会下溢：翻成挡上的前提是那一对改前没挡，那一段的数至少是 1。

试过的打法：找第二个写位图的地方（零）；找不经 `set_blocking_bit` 改 `unblocked_slot_pairs_per_segment` 的地方（零，只有构造与那一处）；找按字段换掉一部分 `DeviceFreeMap` 的地方（`self.allocated =` 这类整份赋值，零）；找从盘上读回集合的地方（没有，集合不落盘）。

什么会推翻：「第 3 条」一节四条单测里「增量维护的段集合与逐对扫的不同」那一句在任何一步报出。

## 攻击面 4：新拒绝成员的分法（没打中）

`refusal_of_commit_generated_answers_that_differ`（`crates/singlefs-core/src/allocator.rs:305`）只在 `AnswersDiffer` 时调。新成员 ⟺ 每块盘都答回落、池里取不到每块盘都空的落点：各盘自己答的回落落点都相同时，那个槽每块盘都空，池级取法的「空槽最多的叶」那一步一定取得到（逐叶走、只在上界不大于眼下最多时跳过），所以池里取不到时各盘答案必不同，落进 `AnswersDiffer`。

上层按成员分流的每一处：

| 位置 | 怎么用 | 新旧成员是否同样对待 |
|---|---|---|
| 发布层 | 「placement.map_err(\|refusal\| PublishError::PlacementRefused {」那一类，`crates/singlefs-core/src/transaction.rs:2510`、第 13828 行，整个成员原样装进去 | 是 |
| 会话层抬 F | 「PublishError::SpaceAdmissionRefused(_) \| PublishError::PlacementRefused { .. } => true,」那一行在 `crates/singlefs-core/src/mounted_session.rs:2120`，不看里面是哪个成员 | 是 |
| 挂载取号之前 | `crates/singlefs-core/src/mount.rs:8155` 起整个成员装进 `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided` | 是 |
| 实例表清扫 | `crates/singlefs-core/src/instance_table_sweep.rs:807` 按 `PublishError::PlacementRefused { .. }` 整类匹配 | 是 |
| 模型对拍 | 「PlacementRefusal::CommitGeneratedFallbackWithoutASlotFreeOnEveryDevice { .. } => {」（`crates/singlefs-verification/src/model_comparison.rs:1322`）与段对齐那一条同一臂映射成 `Unexplained`；挂载那一条经第 1413 行同一个函数 | 是 |
| 历史统计 | `crates/singlefs-verification/src/history.rs:4580` 按成员名分行 | 新成员自己一行，是观测，不分支 |

吞不吞真分歧：判器里「ObservedRefusalReason::Unexplained => None,」（`crates/singlefs-verification/src/model.rs:8703`），随后一律返回分歧（模型要求成功时是 `RefusedWhenModelRequiresSuccess`，要求拒时是 `RefusalReason`）。`Unexplained` 不放行任何一次拒绝，所以映射成它只会多报、不会把一条真分歧吞掉。换成员之前与之后，判器的去向逐处相同。

## 检查单八格

1. **走不到的理由二次查实：成立。** diff 材料里以 `+` 起头、带 `assert!` / `assert_eq!` 的行（`grep -nE` 命中 20 行，最小的在材料第 684 行）全在 `mod tests` 的块里（材料第 661 行起的块头是 `@@ -4672,8 +4817,12 @@ mod tests {`）；`todo!`、`unreachable!`、`panic!` 与 Undecided / Unsupported / OutsideTheFirstVersion 收尾的新成员零命中。`lowest_unblocked_slot_pair_in_the_segment` 找不到对的那一支见 2c：产品上走不到，走到了交 `None`、落点不变、看不出来。
2. **类型与名字：打中。**
   - diff 里新加的 `expect` 逐处分：产品代码里新加的四处带长消息（材料第 289、298、411、624 行）；另一处「let slots_per_segment = usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");」（`crates/singlefs-core/src/allocator.rs:1345`，材料第 398 行）消息只是一个数，没写靠哪条不变量——第一轮判决第二节第 8 条打中的正是这一类，第三节第 2 条的改法要求「新加的 expect 消息写清靠哪条不变量」。同一写法在这份文件里另有 10 处（第 770、951、1087 等行），是存量，不归这一批。「回落」「个数」「下标」「盘的下标」「盘号」「段号」那几处都在 `mod tests`（第 3758 行起）里。
   - 段号：「segments_with_an_unblocked_slot_pair: BTreeSet<usize>,」（`crates/singlefs-core/src/allocator.rs:485`）。规则原文「逻辑地址、物理地址、设备内偏移、代号、inode 号——一律 newtype，**混用必须编译不过**。」（`.claude/rules/fs-design.md:79`）。在 `set_blocking_bit` 里位图下标 `index` 与段号 `segment` 同为 `usize`（第 748、770 行），把第 776 行的 `&segment` 写成 `&index`、或把 `lowest_unblocked_slot_pair_in_the_segment(*segment_index)` 的参数换成一个位图下标，都编得过。叶序号这一批立了 `AllocationRecordTreeLeafIndex`，段号没立，同一份文件里「segments_reclaimed_not_yet_discarded: BTreeSet<usize>,」（`crates/singlefs-core/src/allocator.rs:501`）与 `segment_is_reusable_as_a_whole(&self, segment_index: usize)` 是存量的同一种写法。该立（推的）；立的范围连存量一起，是另一件活。
3. **不兼容旧代码与旧格式：成立。** `lowest_user_data_slot_outside_the_region_by_stepping_through_every_candidate` 带 `#[cfg(test)]`、只被 `mod tests` 调（第 5441、5533 行），不在产品路径上，不认任何旧版本的盘、库或格式；它是对拍参照，正合「运行时与 checker 必须用不同的算法」那一句的方向。
4. **不新建判定库：成立。** 新测试只在进程里断言。
5. **所有任务多机可用：成立。** 新测试不读主机状态、不读路径。
6. **测试优先多线程：打中（轻）。** 规则原文「一条用例一个场景。」（`.claude/rules/test-script-discipline.md:13`），后半句要求「确是一个场景的……在用例旁写一行注释说明为什么」。`segments_with_an_unblocked_slot_pair_are_maintained_incrementally_and_match_a_scan` 先在一块三段盘上走定形前缀（第 5574–5607 行），再在新建的两块 385 槽盘上走随机历史（第 5609–5625 行），两份状态互不相干；用例旁的注释只说「每一步读上一步留下的分配器状态」，那是随机历史内部的次序，不是两段非拼不可的理由。拆成两条（定形一条、随机一条）不丢判别力：7738、7747 两条变异都由定形那一条抓（实现员第三份报告第三节表）。三条落点对拍各是一个场景，生成器里的循环是一段历史的步，不是参数循环，不算。
7. **带 --item 的命令与项：成立（推的）。** 攻击面 1 证了新旧取法在全部可达输入上逐盘答案相同，崩溃枚举流的写流因此逐字节不变，点哪一项都判不出这次换算法的对错；实现员一条没列不算缺项。要做回归的话最贴近的三项（推的，按落点随占用变化的程度挑）：`compaction-relocation-stream`（`R` 排他与用户数据落点同在）、`floor-raise-pushed-by-the-session`（扣住与放开）、`one-file-overwritten-across-two-writable-remounts-stream`（挂载重建之后接着取落点）。
8. **新崩溃流多罩了什么：成立。** 集合只住内存，挂载时由 `rebuild_from_records` 经同一处维护建出（攻击面 3）；挂载之后第一次取落点与旧取法相同，盘上状态与崩溃状态集合都不变，没有已有流罩不到的新状态。

## 第 3 条：让新旧取法交出不同落点的输入

### 造得出的那一个（产品走不到）

全文在 `/home/fy5090/.local/share/singlefs/work/d3-fallback-richest-leaf-code-r2-opus/unit-test-off-grid-cluster-segment-start.rs`（33 行，已 `cargo fmt`），原样放进了副本 `copy/crates/singlefs-core/src/allocator.rs` 的 `mod tests`，接在 `segments_with_an_unblocked_slot_pair_are_maintained_incrementally_and_match_a_scan` 之后：

```rust

    /// 聚簇段集合里有一个起点不在 64 槽格子上（单元区起点加 32）时，按段取与逐个候选看答得不同：第 0 段前 16 对分配掉，
    /// 逐个候选看把 [U + 32, U + 96) 当聚簇段跳过、落在 U + 96；按段取只认段起点相等，第 0 段照取、落在 U + 32。
    /// 产品路径上 `cluster_segments` 只经 `OpenEmptySegment` 插段起点，走不到这一形；这条用例钉的是两种取法相等靠的那条前提
    /// （聚簇段起点在 64 槽格子上）在 `lowest_user_data_slot_outside_the_region` 的入参上没有类型、也没有断言。
    #[test]
    fn user_data_slot_found_by_segment_matches_stepping_through_every_candidate_with_a_cluster_segment_start_off_the_segment_grid(
    ) {
        let mut device_map = DeviceFreeMap::new(
            DeviceIdentity(0),
            (UNIT_AREA_START_SLOT + 3 * CLUSTER_SEGMENT_SLOTS) * SLOT_BYTES,
        );
        device_map.mark_allocated(SlotNumber(UNIT_AREA_START_SLOT), CLUSTER_SEGMENT_SLOTS / 2);
        let cluster_segments =
            BTreeSet::from([SlotNumber(UNIT_AREA_START_SLOT + CLUSTER_SEGMENT_SLOTS / 2)]);
        let by_stepping_through_every_candidate = device_map
            .lowest_user_data_slot_outside_the_region_by_stepping_through_every_candidate(
                &cluster_segments,
                None,
            );
        assert_eq!(
            by_stepping_through_every_candidate,
            Some(SlotNumber(
                UNIT_AREA_START_SLOT + CLUSTER_SEGMENT_SLOTS + CLUSTER_SEGMENT_SLOTS / 2
            )),
            "参照取法：[U + 32, U + 96) 当聚簇段跳过，落在 U + 96"
        );
        assert_eq!(
            device_map.lowest_user_data_slot_outside_the_region(&cluster_segments, None),
            by_stepping_through_every_candidate,
            "按段取与逐个候选看答得一样：聚簇段 {cluster_segments:?}"
        );
    }
```

预期（推的，待主 agent 跑）：第一个断言过（参照取法交 `Some(SlotNumber(50272))`），第二个断言红，`left` 是 `Some(SlotNumber(50208))`、`right` 是 `Some(SlotNumber(50272))`。红了说明分叉；绿了说明 1c 的推理错了（推翻条件）。它只证「前提没守」，不证「产品落点变了」：产品上唯一的调用点传的集合只经第 3073 行那一处插段起点（1b）。

### 造不出的：在 64 槽格子上的全部输入

另一份 `/home/fy5090/.local/share/singlefs/work/d3-fallback-richest-leaf-code-r2-opus/unit-test-every-aligned-cluster-set-and-region.rs`（211 行，一个带参数的函数加四条用例，已放进同一个副本）。产品能交出的聚簇段集合与 `R` 都落在 64 槽格子上，这四条把格子上的输入取全：

| 用例 | 单元区起点 | 整段 / 尾段 | 每步问几组 |
|---|---|---|---|
| `…_ending_in_one_unpaired_slot` | 默认环（50176） | 3 整段 + 1 槽 | 2⁶ 组聚簇段 × 7 种 `R` = 448 |
| `…_ending_in_sixty_three_slots` | 默认环 | 3 整段 + 63 槽 | 448 |
| `…_after_a_128_mebibyte_ring` | 128 MiB 环（9216） | 3 整段 | 2⁵ × 6 = 192 |
| `…_after_a_128_mebibyte_ring_ending_in_one_pair` | 128 MiB 环 | 3 整段 + 2 槽 | 448 |

每条 2000 步，每 500 步换一次相（填的一相七成分配、一成释放回收、一成隔离、一成扣住；放的一相五成释放回收、两成清隔离、两成放开全部扣住、一成分配），直接改一块盘的三种挡位，叠位、`R` 在尾段与单元区外、聚簇段在尾段起点与单元区外都走得到。每一步对集合与逐对扫、对两种取法；末了断言「离开集合又回来」的次数大于 0，取样点没走到集合翻过 0 时整条判红而不是空过（校验路径自己会红的那一半）。

预期（推的）：四条都绿，「返回集合」次数大于 0。估时（推的，没量）：最重的一条约 9 × 10⁵ 次两种取法对比，参照取法一次至多看约 128 个候选，调试构建下每条几秒到一分钟。

### 试过、不分叉的形态

| 形态 | 为什么不分叉 |
|---|---|
| 单元区不满整段的尾巴（1、2、63 槽） | 尾段对数 ⌊r/2⌋、末槽两边都不看（1b） |
| `R` 罩几段、`R` 不满一段、`R` 起点不对齐 | 类型上造不出（1b） |
| `R` 在尾段、在单元区外、在别的盘 | 整段跳过；`contains_slot` 不看盘 |
| `R` 与聚簇段重合或相邻 | 两边各自跳过，重合与否答案相同 |
| 聚簇段在单元区外、在尾段起点 | 那一段在这块盘的集合里不存在或整段跳过，参照取法同样没有候选在里面 |
| 三块以上的盘、盘不等大 | 新取法逐盘独立；不等大时大盘的段在小盘集合里本来就没有 |
| 扣住与隔离叠在同一槽、先后放开 | 集合按三种位合起来翻没翻维护 |
| 挂载重建、克隆、回退、整理搬走 | 都经同一处维护或整份拷贝（攻击面 3） |
| 集合多留一段（7747 那种坏法） | 那一段答 `None`，`find_map` 接着走 |
| 非默认环长 | U 仍在格子上，1a 的三条与 U 的值无关 |

唯一分叉的是聚簇段起点不在格子上，产品路径交不出。

## 自己提的改法（只在我的推理上过、被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| `lowest_user_data_slot_outside_the_region` 开头断言集合里每个起点减单元区起点是 64 的倍数（照 `pairs_inside_the_cluster_segments` 第 591–594 行那一条），或给聚簇段起点立一个只由段号算出的类型 | 1c：入参前提不守 | 推的，待主 agent 跑第一条命令看分叉是否存在 |
| 把「第 3 条」一节四条「格子上全输入对拍」并进 `mod tests`（每步对集合、对两种取法，断言集合翻过 0 至少一次） | 2b：对拍证据在集合翻过 0 那一步上覆盖为零 | 推的，待主 agent 跑第二条命令 |
| `.expect("64")`（第 1345 行）写成说不变量的消息；段号立 newtype（连存量） | 检查单「类型与名字」 | 推的 |
| 增量维护用例拆成定形一条、随机一条 | 检查单「测试优先多线程」 | 推的；两条变异行的点名测试要跟着改 |
| 代价上界加一条会红的检查（例：数一次取落点走过的段数，断言不超过开过的聚簇段数加 2） | 2d：代价那半句没有会红的检查 | 推的 |

## 要主 agent 跑的命令

两条都在副本里起：`/home/fy5090/.local/share/singlefs/work/d3-fallback-richest-leaf-code-r2-opus/copy`（`python3 scripts/hosts/minimal-copy.py /home/fy5090/code/singlefs <副本> --with-test-inputs` 建的，只改了 `crates/singlefs-core/src/allocator.rs`，在草稿目录的 `.deliverables` 里写成「交接」）。要在别的副本里跑的，把草稿目录里两份 `unit-test-*.rs` 原样接在那份 `allocator.rs` 里 `segments_with_an_unblocked_slot_pair_are_maintained_incrementally_and_match_a_scan` 的收尾 `}` 之后（此刻主工作区是第 5626 行之后），先放分叉那一份、再放全输入那一份；两份都以一个空行起头。

1. 分叉输入（预期红）：

   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-core 的测试目标 lib 里攻方腿新加的一条（聚簇段起点不在 64 槽格子上时按段取与逐个候选看对拍）; 范围=1 个测试目标; 为什么=核攻方腿报告第 3 条的分叉输入，预期红，红了坐实入参前提没守，绿了推翻 1c' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/d3-fallback-richest-leaf-code-r2-opus/copy -- cargo test -p singlefs-core --lib -- allocator::tests::user_data_slot_found_by_segment_matches_stepping_through_every_candidate_with_a_cluster_segment_start_off_the_segment_grid --exact`

   看：末行 `TASK_SERVER_RESULT` 的 `exit=` 非 0（cargo test 失败是 101）；`task-server/task log <号>` 里 `left: Some(SlotNumber(50208))`、`right: Some(SlotNumber(50272))` 与 `test result: FAILED. 0 passed; 1 failed`。编不过（`error[E…]`）不算跑成。

2. 格子上全输入对拍（预期绿）：

   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-core 的测试目标 lib 里攻方腿新加的四条格子上全输入对拍; 范围=1 个测试目标; 为什么=把攻击面 1 的读码证明放到聚簇段与 R 取遍 64 槽格子上全部组合、集合必翻过 0 的历史上跑一遍，预期四条都绿' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/d3-fallback-richest-leaf-code-r2-opus/copy -- cargo test -p singlefs-core --lib -- allocator::tests::user_data_slot_found_by_segment_matches_on_every_aligned_cluster_set_and_region`

   看：`TASK_SERVER_RESULT … exit=0`；日志里 `test result: ok. 4 passed`，四个用例名逐个 `... ok`。红在「增量维护的段集合与逐对扫的不同」推翻攻击面 3；红在「聚簇段 …、R …」那一句推翻攻击面 1；红在「取样点没走到集合翻过 0」是我的取样没造对，不是被测对象的问题，回报给接手的腿改相长或步数。

跑完：编译目录由主 agent `task-server/task clean <号>`；副本由主 agent 删（`du -sh` 现在是 54M，量的）。

## 这条腿自己的限度

- 项目代码一条没跑：分叉那一条红不红、四条全输入对拍绿不绿、耗时多少，全是推的。
- 「落点在全部可达输入上不变」靠读码证明（1a），证明里的「可达」只查了 `cluster_segments` 的写入点、`CompactionRegion` 与 `UnitAreaStart` 的构造入口；没查 unsafe 一类绕开字段私有的构造（只看了 `crates/singlefs-core/src/lib.rs` 写着 `#![deny(unsafe_code)]`，别的包没看，推的是没有）。
- 2b 的「量得出」是从 14606 的一行日志加种子、几何、步数逐字相同推出的；没有逐步打印核过那段历史里集合的变化。
- 攻击面 4 只查了 `singlefs-core` 与 `singlefs-verification/src` 里按成员分流的地方；实验装置（`src/bin/`、`scripts/experiment-apparatus/`）里按成员名比对的没逐个看。
- 2d 说的「另两段线性查找会不会成为大头」没量，是照调查报告 1c 的数推的。

## 没打中的形状

见「第 3 条」一节「试过、不分叉的形态」表与攻击面 3「试过的打法」；攻击面 4 的六处分流逐处核过，没有一处按新旧成员分支。取样范围：读码覆盖 `allocator.rs` 全部写位图、写集合、写聚簇段集合的地方，`logged_operations.rs` 的 `CompactionRegion` 全部构造入口，`crates/` 与 `scripts/` 下全部 `.rs` 里 `lowest_user_data_slot*` 的调用点（`grep -rn`）。

## 没做什么

- 没判辩方那几格（第一轮没采的 1c、固定点终止）。
- 没往统一任务服务交任务；判决要引的数没复跑（命令在上一节）。副本上的数不算入库装置上的数。
- 没写模型目录与 `rerun.sh`：攻击用的是副本里的单测，复跑就是上面两条命令。
- 没调崩溃枚举函数，崩溃状态 0 个。
- 草稿目录里留着：`copy/`（交接）、两份 `unit-test-*.rs`、`copy-allocator.diff`、`log-14606.txt`、`minimal-copy.txt`、`fmt-check.txt`、`fmt.txt`。没建编译目录。
