# m3-close-b19-r3fix-code-r1 正推腿报告（Sonnet，2026-11-05）

立场：正推。从 D16（发布语义） 已定项 1 与 C953 的改法推出「这三处应该是什么」，再与 `crates/singlefs-core/src/mount.rs` 逐格比。
没读禁读清单里的任何东西（攻方报告与草稿）。没往统一任务服务交任务，没跑任何编译、测试、脚本；标「读码坐实」的是读代码与 `grep -nF` 看到的，标「推的」的是由读码推出、没有跑出来的。

## 零、读过什么、实现今天的样子（读码坐实）

读过：`crates/singlefs-core/src/mount.rs`（sha256 `cb00d9209f06c9d89652e16e71070e52aa601f2cafca47348c4acff3bee78e4a`，与派发给的相同）的抬 F 读环到卸载那一格（4930–5340）、准入算上限（3880–4110）、重建分配器读环（3190–3300）、C 那一圈各函数（1960–2400）、`recovery.rs` 读槽（1852–1880）、`root_ring.rs` 落点公式（167–195）、`allocator.rs` 的 E2 重读（1772）与根环表（2790 一带）、`walk.rs` 里 checker 的认盘与门槛（8825–8900、9860–10060）、实现员报告 `~/.local/share/singlefs/work/m3-close-b19-r3-impl-core/result.md`。

今天三处的落点（行号是此刻主工作区的，和派发提示里的差几行）：
- `crates/singlefs-core/src/mount.rs:4942`：「fn readable_roots_for_raising_the_floor<Reader: PoolReader + ?Sized>(」组装抬 F 那一遍读出的根；第 4994 行「roots_with_ring_slots.push((*ring_slot, root))」与第 4997 行「.push((*ring_slot, *root_record_self_verified_on_one_rebuild_pass)),」是补读坏的那几槽的两条分支，带的都是循环变量 `*ring_slot`。
- `crates/singlefs-core/src/mount.rs:5013`：「struct RootsReadForRaisingTheFloor {」；调用方在 5216 起解构。
- 最旧有效根：`crates/singlefs-core/src/mount.rs:5041`「let bad_at_mount = ring_slots_pinned_since_the_mount_without_a_root_read_on_this_pass(」→ 5069 的新函数，过滤在 5080「.filter(|(ring_slot, _)| !ring_slots_read_from_the_ring.contains(ring_slot))」，读出根的槽取自 5076「.map(|(ring_slot_the_root_was_read_from, _)| *ring_slot_the_root_was_read_from)」。
- 卸载那一格：`crates/singlefs-core/src/mount.rs:5310`「ReuseOfTheSlotsReclaimedByTheSequence::ImmediatelyBelowTheNewestValidRootOnEveryDevice => {」，5311 调 5088 的新函数，取最小在 5320–5324（`floor_of_the_reclaim.min(newest_valid_root_on_every_device)`），`None` 在 5328「None => Vec::new(),」。
- 准入算上限那一处（上一轮已按槽，没动）：`crates/singlefs-core/src/mount.rs:4024`「let device = system_configuration.immutable.region_devices」。

两条贯穿全报告的底层事实（读码坐实）：
1. 抬 F 那一遍读出的每一条根，不论走 4942 的第一次读、重读，还是补读，都经 `read_root_ring_slot`，它在读出根之后核「根住不住公式给的那一槽」：`crates/singlefs-core/src/recovery.rs:1867`「if !root_sits_in_the_ring_slot_the_formula_gives(」。公式在 `crates/singlefs-core/src/root_ring.rs:188`「if checkpoint_txg == crate::make_filesystem::GENESIS_CHECKPOINT_TXG {」处只给 txg 0（第 0 代根）开了例外，别的 txg 只认 `target_for_publish` 落到的那一槽（`crates/singlefs-core/src/root_ring.rs:172`「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」）。所以**读出的非第 0 代根，它所在的槽恒等于公式槽**；按槽与按 txg 只在 txg 0 的根上可能不同。补读时顶上的那条根来自根环表，表里的 (槽, 根) 来自重建分配器读环那一遍，那一遍同样核过（`crates/singlefs-core/src/mount.rs:2808`「if !root_sits_in_the_ring_slot_the_formula_gives(」）。
2. 第 0 代根的实例代号是 0：`crates/singlefs-core/src/make_filesystem.rs:52`「pub const MKFS_INSTANCE_GENERATION: InstanceGeneration = InstanceGeneration(0);」；实例表的行只给实例 ≥ 1 写：`crates/singlefs-core/src/mount.rs:6245`「中间实例 (i, 0, 0)；实例 0（mkfs）不写行」。抛弃的判法是「表里有它那个实例的行 (i, Ti, Wi) 且它的 txg > Ti」（`crates/singlefs-core/src/recovery.rs:2207`），所以**第 0 代根在今天的写者写出的实例表上恒不被抛弃**（读码坐实；表里被人手写进实例 0 的行不在合法历史里）。

C953 的原文只写了「三处一并改成按槽认盘，与准入那一处同一个口径，随下一批实现」（`.claude/kb/checks-owed.md:953`），以及「只在第 0 代根上与按槽不同、算出来的数相同」（同一行）。派发提示把它转述成「写者读环时按每一份根所在那一槽的区域认盘，不按 txg 推」，这句话 checks-owed 里没有原样出现，我按原文的两句理解，不按转述。

## 一、Q1 正推：三处改完之后，逐句对 D16 已定项 1

先把条款推出的「应该是什么」写成六句，再各配代码行。条款行号是 `.claude/kb/decisions/16-发布语义.md` 自己的行号（grep -nF 现查）。

| 编号 | 条款（D16 已定项 1） | 代码走到的行 | 判定 |
|---|---|---|---|
| S1 | 「盘上全部根槽里自证合法、按实例表判仍然有效的根的最小 checkpoint_txg」（`.claude/kb/decisions/16-发布语义.md:36`） | `crates/singlefs-core/src/mount.rs:5036` 取这一遍读出的根里 `!abandoned_by_table` 的 txg；读环覆盖 `every_root_ring_slot` 全部槽（`crates/singlefs-core/src/recovery.rs:2016` 一族） | 一致（读码坐实）。三处改动不碰这一句 |
| S2 | 「抬 F 那一遍里根环表挂载时读不出的那几项同样按这一句并进最旧有效根」（`.claude/kb/decisions/16-发布语义.md:36`）；钉值「槽号 0 的钉值是第 0 代根的 0（mkfs 在三个区域的槽 0 都种了第 0 代根）」（`.claude/kb/decisions/16-发布语义.md:39`） | 钉着的槽 = 根环表里读坏与读回全零两类（`crates/singlefs-core/src/allocator.rs:1772` 的 E2 读它们）；这一遍读出了根的槽不再并（5069 新函数）；钉值在 `crates/singlefs-core/src/mount.rs:2994` 的 `oldest_txg_a_ring_slot_bad_at_mount_can_have_held` 里对三个区域槽 0 取 0 | 一致（读码坐实）。「这一遍读出了根」改按槽认之后，认的就是「读出它的那一槽」，比按 txg 推更贴字面 |
| S3 | 「读得出、自证过而按实例表被抛弃的，当没有根」（`.claude/kb/decisions/16-发布语义.md:36`） | 抛弃根不进 `valid_root_txgs`（5036 起），它所在的钉着的槽因为「这一遍读出了根」而摘出钉里（5080）；E2 里 `AbandonedByTheInstanceTable` 同样拿掉 | 一致（读码坐实） |
| S4 | 「有一块盘一条不被抛弃的自证有效根都没读出（按读出的那一槽所在区域归盘）」（`.claude/kb/decisions/16-发布语义.md:39`，这一句管重建那一格，D16 另用同一个口径管卸载那一格） | 卸载那一格 5311 调 5088：`crates/singlefs-core/src/mount.rs:5096`「region_devices[usize::try_from(ring_slot_the_root_was_read_from.region)」；与重建那一格 3237 的 `roots_on_the_devices_they_were_read_from`、准入 4024 同一个口径 | 一致（读码坐实） |
| S5 | 「只有释放代不高于「每块盘上最新有效根的 txg」最小者的那一截当场可用（预演与真发都算上）」（`.claude/kb/decisions/16-发布语义.md:43`） | 5320–5324 `floor_of_the_reclaim.min(newest_valid_root_on_every_device)` 当场放开，5328 之后另一条 `HeldUntilFloorTakesEffect` 回收其余；`newest_valid_root_txg_on_every_device`（4265）按 `all_devices` 逐盘取最大再取最小，抛弃根不算 | 一致（读码坐实） |
| S6 | 「有一块盘一条有效根都读不出时这一截为空、整截扣到生效，这一遍读不出、自证不过的槽当没有根」（`.claude/kb/decisions/16-发布语义.md:43`） | 某块盘 `None` ⇒ 整个 `?` 交回 `None`（4265 起 `collect::<Option<Vec<_>>>()?`）⇒ 5328 `Vec::new()`，其余走扣住那一支；读不出又不在「知道住着根的槽」里的槽没进 `roots_with_ring_slots` | 一致，但见下面「条款内部分工」一条 |

### Q1 的结论

**三处改完之后，上面六句都对得上，没有找到「代码走到的行」与条款字面不一致的合法历史。**（读码坐实：逐句对过；「没找到」这半句是否定结论，只算线索，查过的情形在 Q2 第 1、3 小问里列。）

按槽与按 txg 的差别，精确地说是这样（读码坐实 + 推的）：
- 差别只出在 txg 为 0 的根上（零节底层事实 1）。第 0 代根在三个区域的槽 0 各一份，`crates/singlefs-core/src/make_filesystem.rs:821`「pub fn genesis_root_ring_slots() -> impl Iterator<Item = RootRingSlot> {」；公式把三份都推到区域 0 的槽 0。
- **最旧有效根那一处，两种认法算出的数恒相同**（推的，下面给证）：设这一遍读出了第 0 代根，它不被抛弃（底层事实 2），`valid_root_txgs` 里就有 0，最小值是 0，钉着的槽无论摘不摘，最小都不会低于 0；设这一遍一份第 0 代根都没读出，则「读出根的槽」两种认法对非第 0 代根给出同一个集合，对第 0 代根集合是空，也相同。E2 的重读次数可能不同（按槽认时区域 0 槽 0 仍钉着，会多重读它一次，`crates/singlefs-core/src/allocator.rs:1772` 里 `oldest < pinned_txg` 不成立就重读），但重读的每一种结局（读出有效根 / 被抛弃 / 全零 / 仍读不出）最小值都是 0，数不变。
- **卸载那一格，两种认法的数可以不同，但放开的那一截相同**（推的）：每块盘取最大再取最小，非第 0 代根两种认法归同一块盘；第 0 代根 txg 最小，只在「某块盘除第 0 代根外没有别的有效根」时改变那块盘的最大值。这时按 txg 推：区域 1 的盘一条根都没有，交回 `None`；按槽认：交回 `Some(0)`。`None` ⇒ 当场放开的一截为空；`Some(0)` ⇒ 放开「释放代 ≤ 0」的一截，而已释放槽的释放代是释放它的那次发布的 txg（`crates/singlefs-core/src/allocator.rs:2283`「record.generation = release_generation;」），写者的发布 txg 从 1 起，所以这一截也是空的。数不同、效果同。
  - 这一步用到「没有一个已释放槽的释放代是 0」。读码坐实的只是：释放代取调用者传进来的 `release_generation`，今天的写者在发布路径上传发布的 txg（≥ 1）；测试里手拼分配器可以传 0，那是装置不是写者，不算合法历史。

### 条款内部分工（不是补丁引入的差异，Q1 要求逐句对，所以列出）

1. D16 第 43 行说「这一遍读不出、自证不过的槽当没有根」（`.claude/kb/decisions/16-发布语义.md:43`），第 39 行说重建分配器两遍里只有一遍自证过的槽「仍坏不拒，照挂载时读出、记在根环表里的那条根算」（`.claude/kb/decisions/16-发布语义.md:39`）。代码走第 39 行的顶上：`crates/singlefs-core/src/mount.rs:4997`「.push((*ring_slot, *root_record_self_verified_on_one_rebuild_pass)),」，顶上的根随后进 5311 的卸载那一格。两句字面上对「读不出的槽」的处置不同。
   - 能不能算错？推的：顶上的根是挂载之前的根（那一槽「之后没写过」），它的 txg 小于这个进程暖机写的每一条根；暖机覆盖每块盘（`crates/singlefs-core/src/mount.rs:6214`「fn warm_up_publish_txgs」按区域落点直到盘盖全；mkfs 同进程那条流是 `transaction.rs` 的 `warm_up` 写 txg 1、2，分别落区域 1、2，即盘 1 与盘 0）。每块盘的「最新有效根」因此是这个进程写的根，顶上的旧根在取最大时输给它，**不影响卸载那一截**；它只进最旧有效根，而那里第 39 行明文要它进。所以这一处是条款措辞的分工，不是代码错。前提是暖机在卸载之前已落完；没经暖机的手拼流（见 Q2 第 3 问）这一条不保证。
2. D16 第 39 行写「这个进程写掉那一槽、或进程内记根时重读读出有效根或被抛弃的根才解开」（`.claude/kb/decisions/16-发布语义.md:39`），而抬 F 那一遍在本地计算里看到「这一遍读出了根」就把钉着的槽摘出（5080）。这是这三处改动之前就有的行为（RR4，早先几轮已判的那一格），不是新引入；摘的是本次计算的局部拷贝，分配器里的钉不动（5029 起的函数只读盘、只借分配器的表）。读出了根就说明那一槽此刻住的是这条根，钉值（可能住过的更旧 txg）已不适用；读出抛弃根时当没有根，同第 36 行。判为一致。

### 最小的、按槽与按 txg 结局不同的合法历史（只在函数一级造得出）

实现员的三条单测造的就是它们：R = 3、第一版几何区域归属 0 / 1 / 0；
- 卸载那一格：三个区域槽 0 的第 0 代根 + 区域 0 槽 1 的 txg 3：按槽盘 1 最新 0，交回 `Some(0)`；按 txg 盘 1 无根，交回 `None`（`crates/singlefs-core/src/mount.rs:11206` 那一条）。
- 最旧有效根：区域 1 槽 0 钉着（钉 0）、这一遍在区域 0、区域 1 的槽 0 读出第 0 代根：按槽不再钉；按 txg 照旧钉（`crates/singlefs-core/src/mount.rs:11247`）。
- 区域 0 槽 0 钉着、这一遍只在区域 1、2 的槽 0 读出第 0 代根：按槽照旧钉；按 txg 摘掉（`crates/singlefs-core/src/mount.rs:11276`）。
这三形放到真写者上，数或放开的一截不变（上面两条证明）。**要让按 txg 的结局错到放开被引用的单元，需要一块盘上最新有效根是 txg ≥ 1 而被误算成另一块盘的**；读出的非第 0 代根槽恒等于公式槽，这一误算造不出来。

## 二、Q2 四小问

### 2.1 按槽认盘之后，写者与 checker 在准入或卸载上结局会不会不同

先把两边认盘的口径并排（读码坐实）：

| 格 | 写者 | checker |
|---|---|---|
| 准入算上限的「每块盘上最新的持久有效根」 | `crates/singlefs-core/src/mount.rs:4024`「let device = system_configuration.immutable.region_devices」，下标是根所在槽的区域 | `crates/singlefs-checker/src/walk.rs:8888`「let device = geometry.region_devices[usize::try_from(*region).expect(」，`roots` 是 (区域, 槽, 根) 三元组，区域取自读出它的槽 |
| 最旧有效根（回收门槛、I-3.1 丁-defer、I-7.18） | 5036 起取这一遍读出的有效根的 txg 最小，并上钉着而没读出的槽 | `crates/singlefs-checker/src/walk.rs:9880`「fn deferred_slots_reclaim_threshold(」只看 txg：全部可读根里有效的取最小，没有按盘、按槽分；I-7.18 另并上根自己带的钉槽 p 的钉值 |
| 卸载那一格当场放开到哪儿 | 5310–5328 | **没有一条 checker 不变量直接按「每块盘上最新有效根的最小者」判**（`crates/singlefs-checker/src/walk.rs:8766` 的同名量只在 I-7.9 的抬 F 上限里用）；放开的那一截只通过「别用了被根引用的单元」那几条间接被看见 |

checker 里没有任何一处用 `txg % 区域数` 推盘（`grep -nE 'txg % |% geometry.regions' crates/singlefs-checker/src/walk.rs` 只剩 9970 的槽编码还原与几条测试里的行）。所以 C953 说的那条分歧（写者按 txg、checker 按槽）改完之后在这三处上两边的口径是一个：按读出它的那一槽的区域。

查过的、两边结局可能不同的情形（读码，没跑；否定结论只算线索）：
1. **只剩第 0 代根的那块盘**（盘 1）：写者卸载那一格按槽得 `Some(0)`、放开的一截为空；checker 没有直接判它的不变量，间接那几条看不到「空」与「空」的区别。两边同。
2. **盘上一条根都没有的盘**：准入：写者 4020–4035 用 `BTreeMap` 只在有根的盘上取最小，与 checker 8759–8770 同；卸载：写者 `None`（整截扣住，条款第 43 行），checker 无对应格。两边在准入同、在卸载 checker 不判。
3. **钉着的槽摘不摘**：写者在 5080 摘掉这一遍读出了根的槽，checker 的 I-7.18 用根里落盘的钉槽 p 的钉值（`walk.rs:9954` 起），两边的来源不同（写者读内存里的根环表，checker 读最新根里写的 p），但这是三处改动之前就有的结构，而且按槽与按 txg 的差别只在钉值为 0 的那三个槽上，最小值都是 0，I-7.18 的现算值不变（推的）。
4. **region_devices 的两份**：写者读环用 `system_configuration.immutable.region_devices`，卸载那一格认盘用 `parameters_of_the_pool.region_devices`（5311–5313）。两份相等：`crates/singlefs-core/src/mount.rs:7311`「caller_region_devices == disk_region_devices,」不等就在读环之前拒，而 `parameters_of_the_pool` 的区域归属来自盘上选出的系统配置（`crates/singlefs-core/src/mount.rs:7217`「region_devices: system_configuration.immutable.region_devices,」）。

**结论（推的，否定结论）：没有找到让这三处改动造成的写者与 checker 结局不同的合法历史。** 它能推翻的现象：一条 checker 不变量按「每块盘上最新有效根」判放开的那一截，且写者在某块盘只有第 0 代根时与它说法不同——我没找到这样的不变量；若主 agent 另有，请指给我看。

### 2.2 补读坏的那几槽带的槽，与它真正住的槽可能不同吗

补读发生在 `crates/singlefs-core/src/mount.rs:4942` 函数里，对象是根环表里「重建分配器两遍里只有一遍自证过」的槽，迭代的键就是表里的槽，三种结局：

| 结局 | 代码行 | 带的槽 | 真正住的槽 | 会不会不同 |
|---|---|---|---|---|
| 重读读出自证过的根 | 4994 | `*ring_slot` | `read_root_ring_slot(.., *ring_slot)` 读的就是这一槽，读出之后核过公式槽（`crates/singlefs-core/src/recovery.rs:1867`） | 非第 0 代根：恒相同。第 0 代根：三份字节相同，读出它的就是被问的那一槽，仍相同 |
| 重读读坏、不是全零 | 4997 | `*ring_slot`，根是表里记着的 | 这条根是在重建那两遍的某一遍里从这一槽读出的（`crates/singlefs-core/src/mount.rs:2808` 核过公式槽，两遍按槽配对见 2866 起） | 相同。这一槽此刻读不出它，但它最后一次被看见的地方就是这一槽 |
| 重读读回全零 | 4997 | 同上 | 同上；`read_root_ring_slot` 把全零当 `Bad(NotSelfVerified)`，与上一行同一分支 | 相同。此刻这一槽里没有这条根（若盘真被清零），带的是「它曾住的槽」，不是「它现在住的槽」；这是条款要的顶上（`.claude/kb/decisions/16-发布语义.md:39`「仍坏不拒，照挂载时读出、记在根环表里的那条根算」） |

槽不会被别的根换掉：这个进程写掉那一槽时根环表里这一项换成 `ValidRoot`（`crates/singlefs-core/src/allocator.rs:2797`「.insert(ring_slot, RootRingOccupant::ValidRoot { checkpoint_txg });」），不再是「只有一遍自证过」那一类，就不在补读之列。

假如带错了（反事实，用来回答「哪一处会算错」；今天走不到）：
- 5041 的最旧有效根：多摘一个钉着的槽 ⇒ 门槛偏高 ⇒ **少扣**，可能放开那一槽里可能住过的更旧的根引用的单元；少摘 ⇒ 多扣，安全。
- 5311 的卸载放开：把根算到别的盘上 ⇒ 那块盘多出一条它没有的根，最大值偏高，最小值偏高 ⇒ **少扣**，可能放开被「只剩这块盘」时恢复会择中的根引用的单元；算到有根的盘上而漏了本盘 ⇒ `None` ⇒ 多扣。
- `roots`（5217 起去掉槽的那份）不受影响：影子账与候选集不看槽。
今天带的槽 = 观测到它的槽，所以这三种都不会发生；而且第 0 代根不引用任何用户单元，即使带错，第 0 代根造成的偏差也只落在 `Some(0)` / `None` 上，放开的一截不变（见 Q1 的证明）。

结论：不可能不同（读码坐实：带的槽来自读出它或观测到它的那一槽，非第 0 代根的槽被公式核过）。

### 2.3 「真写者上两种认法算出的数相同」：成立要靠的前提，逐个查

实现员的原句要靠一个前提：「暖机在准入与卸载之前已在每块盘落了本实例的根」。我把它拆开，并且找出**真正必要的前提**（比它弱，所以那句话成立得比实现员说的更宽）。

必要前提（读码坐实 + 推的），逐个查今天的写者：

| 前提 | 为什么要 | 今天的写者满不满足 |
|---|---|---|
| P1 读出的非第 0 代根，所在槽恒等于公式槽 | 否则按槽与按 txg 会在第 0 代根以外也不同 | 满足。读槽的唯一入口 `read_root_ring_slot` 核过：`crates/singlefs-core/src/recovery.rs:1867`「if !root_sits_in_the_ring_slot_the_formula_gives(」；补读顶上的根来自重建那一遍，同样核过（`crates/singlefs-core/src/mount.rs:2808`）。读码坐实 |
| P2 第 0 代根不被实例表抛弃 | 最旧有效根里它才进 `valid_root_txgs`；卸载那一格里它才算那块盘的有效根 | 满足。第 0 代根实例代号 0（`crates/singlefs-core/src/make_filesystem.rs:52`），实例表不给实例 0 写行（`crates/singlefs-core/src/mount.rs:6245`）。读码坐实 |
| P3 第 0 代根的 txg（0）是全环最小 | 最旧有效根里它出现就把最小值压到 0；卸载那一格里它不会改变一块盘上「别的根」的最大值 | 满足（0 是 u64 的最小值） |
| P4 没有已释放槽的释放代是 0 | 卸载那一格里 `Some(0)` 与 `None` 放开的一截才相同（都空） | 写者满足：释放代取发布的 txg（`crates/singlefs-core/src/allocator.rs:2283`「record.generation = release_generation;」），新实例第一次发布的 txg ≥ 1。读码只坐实了这个赋值，没有逐个核全部调用者传的是不是发布的 txg，推的 |

**暖机那一句不是必要前提。** 证明（推的）：把「读出的根」分成第 0 代根（txg 0）与其余。两种认法对其余的根归盘一致（P1）。每块盘的最大值只在「这块盘除第 0 代根以外一条有效根都没有」时才可能不同；这时按槽的最大值 ∈ {0, 无根}、按 txg 的最大值 ∈ {0, 无根}；于是整体最小值 ∈ {无根/`None`, `Some(0)`}（P3），放开的一截都是「释放代 ≤ 0」的空集（P4）。所以**暖机落没落满每块盘，不改变放开的一截**；最旧有效根那一处同理（P2、P3：只要读出第 0 代根，最小值就是 0，与钉不钉无关）。

实现员那句里的前提逐项查：
1. 可写挂载：`warm_up_publish_txgs`（`crates/singlefs-core/src/mount.rs:6214`）从写行那次发布起逐次加一、按落点公式看落哪块盘、直到盘盖全，至多 R = 3 次；第一版几何 0 / 1 / 0 下满足。满足。
2. mkfs 同进程那条流：`warm_up` 固定推 `WARM_UP_EMPTY_PUBLISHES` 次（`crates/singlefs-format/src/lib.rs:335`「pub const WARM_UP_EMPTY_PUBLISHES: u64 = 2;」），txg 1、2 落区域 1、2，即盘 1、盘 0。第一版几何下满足；区域归属换成三块不同的盘时（`region_devices` 是 `[DeviceIdentity; 3]`，格式允许）只盖两块，盘 0 只有第 0 代根。**这时前提 1 不满足**。
3. 暖机的根在抬 F 那一遍读得出：它是「知道住着根的槽」，读坏重读仍坏就拒这次抬（`crates/singlefs-core/src/recovery.rs:2016` 一族），不会带着「读不出」走到 5311。满足。
4. 暖机的根真的落在盘上：设备「吞写」（FUA 返回成功、数据没落）时前提 1 在盘上不成立而写者以为成立。

给出前提不满足的反例历史，逐个算两种认法：
- 反例 A（2 的变体，区域归属 [0, 1, 2]，三块盘，mkfs 同进程，txg 1、2 暖机之后卸载）：盘 0 只有第 0 代根（区域 0 槽 0），盘 1 有 txg 1，盘 2 有 txg 2。按 txg：第 0 代根三份都归盘 0，盘 0 = 0，盘 1 = 1，盘 2 = 2，最小 0；按槽：盘 0 = 0（只有 (0,0) 那份），同上，最小 0。**相同。**
- 反例 B（4：吞掉盘 1 上暖机根的写，区域归属 0 / 1 / 0）：区域 1 槽 0 还是第 0 代根，写者记它为「知道住着根的槽」，抬 F 那一遍读到的是第 0 代根（有效、txg 0）；盘 1 上没有别的根。按 txg：第 0 代根归盘 0，盘 1 无根 ⇒ `None`；按槽：盘 1 = 0 ⇒ `Some(0)`。**数不同，放开的一截都是空。** 这就是上面「`None` 对 `Some(0)`」那一格的真实造法；两种认法的整池结局相同。
- 其余：手拼的流（装置、测试里 mkfs 之后不暖机直接 `unmount`）同样落在 `{None, Some(0)}` 里，P4 对手拼分配器不保证（传 0 的释放代）：那不是合法历史。

结论：实现员那句话**成立，但它给的理由不是必要条件**；真正的依据是 P1–P4。没有找到让「放开的一截」不同的合法历史（否定结论，线索）。一个会推翻它的现象：一个已释放槽的释放代为 0，或第 0 代根按实例表被抛弃，或读出的非第 0 代根所在槽不等于公式槽——这三样我在今天的写者上都没找到。

### 2.4 三条函数级单测与三行变异测到了要测的东西吗

先对三行变异与三条单测，推每条改法让哪几条单测红（读码推，没跑；任务 6526 是主 agent 说副本上三行都抓到，我没复跑）：

| 变异（`crates/mutations.tsv` 行） | 改的行 | 单测 1（`mount.rs:11206`，卸载取最小） | 单测 2（`mount.rs:11247`，区域 1 槽 0 读出） | 单测 3（`mount.rs:11276`，区域 0 槽 0 仍钉） |
|---|---|---|---|---|
| 「抬 F 那一遍按 txg 认盘」（6742） | 5096 换成 `root.checkpoint_txg.0 % 3` | **红**：盘 1 无根，`None` ≠ `Some(0)` | 绿（不经过这个函数） | 绿 |
| 「抬 F 那一遍按 txg 认读出根的槽」（6743） | 5076 换成 `target_for_publish(root.checkpoint_txg, AT_MAKE_FILESYSTEM)` | 绿 | **红**：区域 1 槽 0 仍钉着，交回 `{(1,0):0}` | **红**：区域 0 槽 0 被别区域的第 0 代根摘掉，交回空 |
| 「抬 F 那一遍读出根的槽不从钉里滤掉」（6744） | 5080 换成恒真 | 绿 | **红**：交回 `{(1,0):0}` | 绿（本来就不该被摘） |

每行的锚点我用 python 字面计数查了：`crates/mutations.tsv` 第 2226、2239、6742–6744 行的「原文」在 `mount.rs` 里各恰好命中 1 次（读码坐实；换锚点的两行 2226、2239 也是 1 次）。命名上，6742 行点名的单测 1、6743 行点名的单测 3、6744 行点名的单测 2 都在上表里是红的；6743 行另外让单测 2 也红，是多罩一层，不是问题。

**这三行变异在整池一级是等价变异**：2.3 的证明说明按槽与按 txg 的差别在整池上不改最旧有效根的数，也不改卸载放开的一截，所以没有一条整池测试能杀它们；它们只能在函数一级（把返回的 `Option` / 钉着的槽表）里被杀。这不是缺陷，但这意味着：**三行变异罩住的是两个小函数的返回值，罩不住它们被谁调用、调用时传了什么。**

没有变异罩着的改法（读码）：
1. 补读时给根带槽，`crates/singlefs-core/src/mount.rs:4994` 与 4997 的两处 `(*ring_slot, ..)`：改成固定带 `RootRingSlot { region: 0, slot: 0 }` 没有变异、没有单测（三条单测直接调 5069 与 5088 两个函数，不经 4942 的组装）；整池一级同样是等价变异，要杀它得造 `readable_roots_for_raising_the_floor` 的函数级用例（要一个 `PoolReader`、一份系统配置和一张带「只有一遍自证过」槽的根环表）。**这是三处里唯一真正新写的、取值来源在组装里的一处，没有罩。**
2. 5041 的实参：把 `roots_with_ring_slots_read_from_the_ring` 换成 `&[]`，效果同 6744（读出的槽一个都不摘），但 6744 改的是函数内部，调用点换实参没有行。同样是函数一级可杀、整池一级等价。
3. 5311–5313 的实参 `&parameters_of_the_pool.region_devices`：换成别的数组没有行。对非第 0 代根这会改变整池结局，由已有的整池卸载用例间接罩（推的，没逐条核是哪几条）。
4. 5013 结构体字段本身（`roots_with_ring_slots` 的槽被丢弃，5217 起 `roots` 只取根）：没有测。

另一点：单测 1 断言的是 `Some(GENESIS_CHECKPOINT_TXG)`（`crates/singlefs-core/src/mount.rs:11237` 的断言消息），而按 2.3，这个值与 `None` 在整池上效果相同——断言钉的是函数的返回值，不是「放开了多少」。若以后有人把「卸载取最小」改成别的写法（例如直接返回放开的上界），这条单测会误红。判为：测到了它要测的函数，没测到「放开的一截」这个效果。

## 三、Q3 一句话与逐处清单

**一句话：这三处之外，`mount.rs` 里还有按 txg 推区域、推槽或推盘的地方，一共十一处；其中没有一处该改成按槽，两处（第 2251 行一带与第 5257/5263 行的两次表查）可以改但改了数不变，一处（第 6982 行）今天碰不到第 0 代根。**（读码坐实：每处的行号都是 `grep -nF` 命中的；「该不该按槽」的判断是推的。）

检索办法：`grep -nF 'target_for_publish' crates/singlefs-core/src/mount.rs`、`grep -n '% ROOT_RING_REGIONS\|% 3' crates/singlefs-core/src/mount.rs`、`grep -n '\.region\b' crates/singlefs-core/src/mount.rs`。`% ROOT_RING_REGIONS` 在 `mount.rs` 里只有 2360、2943、5473、6231 四处，全是环长或次数上界，不推区域；`% 3` 只剩测试里的 11062；`.region` 的取值点 2794、3246、4025、5096、6729 是按槽的，不在此列；4795、5192、6221、6987 是按 txg 推出的槽再取 `.region`。

| # | 行 | 做什么 | 该不该按槽 | 理由 |
|---|---|---|---|---|
| 1 | 2033（`.entry(target_for_publish(record.checkpoint_txg, slots_per_region))`） | 没施加的 journal 记录点名的落点，按「那次发布的 txg 将写（写过）的槽」归组，C 那一圈在「这个进程写掉那一槽」时解开 | 不该 | 记录没有槽；键必须是写者将写的槽，与解开的时机（写者写那一槽）对口。记录的 txg ≥ 1，不碰第 0 代根 |
| 2 | 2111（`.entry(target_for_publish(checkpoint_txg, slots_per_region))`） | 同上，账里的落点 | 不该 | 同 1 |
| 3 | 2251（`.entry(target_for_publish(root.checkpoint_txg, slots_per_region))`） | 比所选那一版新的可读根的账，按根的 txg 归槽 | 可以按槽，数不变 | 这条根是从环里读出的，读出它的槽等于公式槽（P1）；它比所选那一版新所以 txg ≥ 1。要和 1、2 的键对齐，现在按 txg 反而是同一个口径，改了无收益 |
| 4 | 2363、2369（`target_for_publish(*newer_root_txg, ..)` 与 `target_for_publish(CheckpointTxg(txg_in_between), ..)`） | 丙并中间：对一段 txg 区间逐个算它们将写的槽 | 不能按槽 | 中间那些发布没有根可读，只有 txg 可算 |
| 5 | 2953、2961（`let ring_slot = target_for_publish(CheckpointTxg(txg), slots_per_region);`，函数 `txgs_ring_slots_bad_at_mount_can_hold`，2937） | 由 txg 区间反推一个读坏的槽能住哪些 txg | 不能按槽 | 方向是槽→txg 集合，只能走公式；第 0 代根住的区域 1、2 槽 0 由 2994 的 `oldest_txg_a_ring_slot_bad_at_mount_can_have_held` 单独特判成钉 0。公式在 [0, first_txg) 里落不到的槽（年轻的环）不钉，注释写明那时第 0 代根还在区域 0 槽 0（2990–2993 一带），是已判过的取舍，不是这一轮要攻的 |
| 6 | 4790–4795（`push_one_floor_raise_within_the_admission_budget` 的 `device_of_txg`） | 预算：抬 F 那一串各次空发布将落哪块盘，数次数 | 不能按槽 | 将来的发布，没有根可读，写者的落点就是公式 |
| 7 | 5187–5192（`raise_the_floor_through` 的 `device_of_txg`） | 同上，真排 | 不能按槽 | 同 6 |
| 8 | 6219–6221（`warm_up_publish_txgs`） | 暖机各次落哪块盘 | 不能按槽 | 同 6 |
| 9 | 6982–6987（`root_of_the_version_not_in_its_ring_slot_on_the_device`） | 现行那一版的根该在哪一槽、在不在这块盘，只对「这块盘落后」的盘判 | 理论上第 0 代版该按槽，今天碰不到 | 这条函数只在 `is_behind_the_version` 成立时才被调（`crates/singlefs-core/src/mount.rs:6946`「let is_behind_the_version = newest_system_configuration_journal_tail < version.journal_position;」），第 0 代版没有 journal 记录，位置计数器是 0（6886 一带 `counter < FIRST_JOURNAL_COUNTER` 返回），盘上的系统配置 tail 不可能小于它。读码，推的 |
| 10 | 5257、5263（`placements_recorded_for_the_abandoned_root` 与 `placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot` 两次表查，内部 `allocator.rs:1386`、`1431` 按 txg 求槽） | 抬 F 那一遍里被抛弃的根，用它的 txg 在根环表里找它的槽 | 可以按槽，数不变 | 对象是被抛弃的根，非第 0 代根（P2），读出它的槽等于公式槽。这两次是「调用点按 txg 求槽的调用」，不含 `% ROOT_RING_REGIONS` 字样，所以用 `grep target_for_publish` 在 `mount.rs` 里看不到它们，得往 `allocator.rs` 追 |
| 11 | 1500 一带（`record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount` 传 txg 进分配器，`allocator.rs:2794`、`2891` 内部按 txg 求槽） | 进程内记一条刚写的根 | 不该 | 记的是写者自己写的槽，写者的落点就是公式 |

另外 `mount.rs` 里已经按槽的：3246（重建那一格归盘）、4025（准入算上限）、5096（本次新写）、6729（`rings_of_the_device_against_the_selected_version` 按槽逐槽读），它们与本次三处同一个口径。

`mount.rs` 之外的 `mounted_session.rs:1819`（`target_for_publish(CheckpointTxg(txg), slots_per_region) == ring_slot`）与 `transaction.rs:570` 也按 txg 求槽，都是写者将写的槽，不在这一轮范围，没读细。

## 四、要主 agent 跑的命令

一条都没交。下面是我写好、没跑的东西与可以复核我的结论的命令，由主 agent 按需交：

1. 按槽与按 txg 的等价性枚举（我写的独立小模型，不引 `crates/` 的代码，只用标准库；`sha256 097f7a92f8e3f898cab1cda82d1310e208a1a20705d5d714cb56352fbbec15a7`，在 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-r3fix-code-r1-sonnet/by_slot_vs_by_txg.py`）。我只用 `ast.parse` 看了语法，没跑。它枚举 R = 3、每区 S 槽、区域归属 0 / 1 / 0 的槽内容（全零、第 0 代根、公式槽的正 txg 根含抛弃与否）、这一遍读出与否、钉与否，比 2.3 的两条：最旧有效根的数必须相同；卸载那一格的数可以 `None` 对 `Some(0)`，放开的那一截必须相同。
   - `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b19-r3fix-code-r1-sonnet -- python3 -I by_slot_vs_by_txg.py 1 6`（小）
   - `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b19-r3fix-code-r1-sonnet -- python3 -I by_slot_vs_by_txg.py 2 6`（S = 2，估计一分钟量级，不标重型）
   - 自证会红（弄坏开关 `--selftest` 把按槽那一边永远认盘 0）：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b19-r3fix-code-r1-sonnet -- python3 -I by_slot_vs_by_txg.py --selftest 1 6`，退 0 才算红了（找到了反例）。
   - 看退出码：前两条 `exit=0` 且打印「核了 N 个状态」；有反例就打印首个反例、退 1。模型是我自己的推理的一份机械化，**它与 2.3 的证明同出一人，不算独立第二证据**；它能抓的是我证明里的漏洞，抓不到我对 `crates/` 读错的地方。
2. 复核 2.4 的「各红在哪」表（我是读码推的，主 agent 说任务 6526 副本上三行都抓到，只回答了「红了」，没回答「红在哪几条」）：对每一行变异单跑、看红的测试名。命令形态照 `crates/mutations.tsv` 第 6742–6744 行第四列：`task-server/task submit -- bash scripts/mutation/prove-red.sh singlefs-core '抬 F 那一遍按 txg 认读出根的槽'`（另两行把名字换成「抬 F 那一遍按 txg 认盘」「抬 F 那一遍读出根的槽不从钉里滤掉」），再看日志里 `ring_slots_of_the_roots_read_for_raising_the_floor_tests` 下哪几条 FAILED。预期：6742 只红 `the_genesis_root_read_...`；6743 红 `a_ring_slot_pinned_...` 与 `a_ring_slot_of_region_zero_...`；6744 只红 `a_ring_slot_pinned_...`。（`prove-red.sh` 的用法我没读，上面的参数形态照实现员报告第 5 条抄；以脚本自己的 `--help` 为准。）
3. 2.4 里没变异罩着的第 1 条（补读时带槽）若要补，建议造一条函数级用例：喂 `readable_roots_for_raising_the_floor`（`crates/singlefs-core/src/mount.rs:4942`）一个内存盘，让区域 1 槽 0 只留第 0 代根并在根环表里记成「两遍里只有一遍自证过」、此遍读坏，断言交回里带的槽是 `(1,0)` 不是 `(0,0)`。这是改动建议，不是我做的。

## 五、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Q1 S1 环里最旧有效根的集合 | 一致 | 读出的有效根 txg 取最小，三处改动不碰（读码坐实） |
| Q1 S2 挂载时读不出的槽并进最旧有效根 | 一致 | 「这一遍读出了根」改按读出它的那一槽认，比按 txg 推更贴字面；钉值 0 取自 2994（读码坐实） |
| Q1 S3 读出被抛弃根当没有根 | 一致 | 抛弃根不进 txg 表，所在钉着的槽摘出（读码坐实） |
| Q1 S4 每块盘归盘按读出它的那一槽的区域 | 一致 | 5096 与重建那一格 3237、准入 4024 同口径（读码坐实） |
| Q1 S5 卸载当场放开的一截 | 一致 | 逐盘最大取最小、`None` 整截扣住（读码坐实） |
| Q1 S6 读不出、自证不过的槽当没有根 | 一致，条款内部分工 | 第 43 行与第 39 行对「两遍只有一遍自证过」的槽说法不同，代码走第 39 行；顶上的旧根在暖机后的卸载取最大时输给本实例的根（推的，前提暖机落满） |
| Q2.1 写者与 checker 结局不同 | 没找到 | 两边都按读出它的那一槽的区域归盘；checker 里没有按 txg 推盘的地方；卸载放开的一截没有直接对应的 checker 不变量（否定结论，线索） |
| Q2.2 补读槽与真正住的槽 | 不同不了 | 槽来自观测它的那一槽，非第 0 代根被公式核过；带错时 5041 / 5311 两处会少扣（反事实，今天走不到） |
| Q2.3 「两种认法算出的数相同」 | 成立，理由要换 | 最旧有效根的数恒同；卸载的数可以 `None` 对 `Some(0)` 但放开的一截同。必要前提是 P1–P4，暖机落没落满不是必要的 |
| Q2.4 测到了要测的东西 | 部分 | 三条单测在三行变异下红在不同的条（表见 2.4）；三行变异在整池一级是等价变异，只能在函数一级杀；补读时带槽那一段（4994、4997）没有变异也没有单测罩着，5041、5311 的实参同 |
| Q3 其余按 txg 推区域或槽的地方 | 有，十一处 | 九处不该或不能按槽；两处（2251、5257/5263 一组）可按槽但数不变；6982 今天碰不到第 0 代根 |

每条结论「什么现象会推翻它」：
- Q1 一致：找到一条第 0 代根以外、读出的槽 ≠ 公式槽的根（`recovery.rs:1867` 那一核被绕过），或第 0 代根被实例表抛弃。
- Q2.1：找到按「每块盘上最新有效根」判放开的那一截的 checker 不变量，且一段合法历史上两边说法不同。
- Q2.2：找到一条路径，补读时顶上的根带的槽不是它被观测到的槽。
- Q2.3：找到一个已释放槽的释放代为 0 的写者路径，或 `by_slot_vs_by_txg.py` 跑出反例。
- Q2.4：prove-red 复跑里某一行变异红在我表里没列的测试，或我列为「没罩着」的某一处其实被现成用例罩着。
- Q3：找到第 6982 行的函数在第 0 代版上被调用并产生不同的结局，或 `allocator.rs` 里有我没追到的按 txg 求槽的调用。

## 六、没做什么

- 不判别的腿的格，不替主 agent 采纳或出判决；没往统一任务服务交任务，没跑任何测试、变异、证红与脚本（`by_slot_vs_by_txg.py` 只做了语法检查）；判决要引的数没复跑，上面所有「推的」都等主 agent 复跑，命令在第四节。
- 没有跑过整份模型目录的 `rerun.sh`（这一轮没建模型目录，只在草稿目录放了一个小脚本，不是入库模型）。
- 没读攻方腿的报告与草稿；没读实现员报告以外的批 19 其它腿的东西。
- 没有核 `mounted_session.rs` 与 `transaction.rs` 里按 txg 求槽的调用（第三节末尾说了）。
- 在 2.4 里我对锚点命中次数用了一次只读的内联 python 字面计数（读 `crates/mutations.tsv` 与 `mount.rs`，没写任何文件），读到的是每行原文各命中 1 次。这一步若按「脚本一律交主 agent」算越线，请主 agent 重核（`python3 scripts/mutation/crates-mutation-rows.py static-check <副本>` 本来就判这一条）。
