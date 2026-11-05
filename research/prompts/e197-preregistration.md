# E197 跑前登记：扫环胶水内存随盘数

写于 2026-10-22，装置写之前。

问法已由主 agent 删定（2026-10-22，装置写之前）：① 甲一的分母留「登记常数」，删「同次自比」（同次自比读法下「收齐之后不释放」的倍数推得比基线小，判不出 C653 登记行要的判别力自证）；② 「随记录填满程度变」读成「记录槽填满」，「甲-点名项」不再对应甲行、退回第八节几何敏感性的一格照量；③ 「降到接近 1」读成「每多一盘不再涨」（C653 管的是随盘数涨），「乙-接近一」不判、照报。乙二的两遍读写法不在岔路单里，不加臂。

## 一、问题

英文名：scan_journal_glue_memory_by_device_count

主 agent 给的问题（派发提示逐字）：

> 被测条款：C653（扫环胶水内存随盘数涨），`.claude/kb/checks-owed.md` 里 `| C653 |` 整行；去向里程碑三七·2
> 给 C653 取一个新实验号（取号前现查 `.claude/kb/experiments.md` 最大号），写跑前登记，登记里每个量对应岔路单的一行（甲 上界、乙 改法），对不上任何一行的不登记。
> 验收：臂、对照、判据、失败条款跑前写死；阳性对照是「胶水收齐之后不释放」那一形（必须红）；真实基线是今天 `crates/` 的 `scan_journal`；在默认环几何（每盘 196 608 个记录槽）上量，取样点至少 1、2、4 盘，记录槽填满与半满两档（几何敏感性一行照 `.claude/rules/mutation-sampling.md` 第六类）；峰值经 `scripts/hosts/run-with-memory-cap.sh` 量，写明量法（RSS 峰值从哪读）。
> 方案按 `crates/` 今天的实现来谈（`.claude/rules/implementation-first.md`）：装置直接调实现里的 `scan_journal`，不另写模型；登记里写「实现今天的样子」一行，读过的 `crates/` 路径加行号。
> 登记写明这实验是长活还是短活、要不要双机（默认环几何四盘的内存推一下给数，标「推的」）。

读法写死：

- 「`scan_journal` 的峰值」= 一次 `scan_journal` 调用（或第五节里替它的那一份胶水）从进入到交回之间进程内存的最高点，减去同一进程在调用之前已占的那一截。判的那一个数取内存包装量到的 cgroup 峰值（第六节「量法」），减去同一几何「只建装置、不扫环」那一格的 cgroup 峰值；进程内的分配器峰值与 VmHWM 两个数只用来互核，不判。
- 「盘数倍数」r(N) = 同一臂、同一几何（槽填满档、点名项数相同）下 N 盘的峰值 ÷ 1 盘的峰值。
- 「默认环几何」= 环长 `JOURNAL_RING_DEFAULT_BYTES` = 805 306 368 字节，每盘 805 306 368 ÷ 4096 = 196 608 个记录槽（第七节锚点 A1）；盘的大小取 `TEST_IMAGE_DEFAULT_BYTES` = 4 GiB（满足 D23 已定项 19 ③「环 ≤ 设备容量 ÷ 4」，第七节锚点 A4）。
- 「记录槽填满」= 每盘全部 196 608 槽都是自证过的记录；「半满」= 每盘前 98 304 槽是自证过的记录、其余 98 304 槽全 0（从没写过）。
- 「池里的记录怎么摆」：照今天的写路径，每条记录写到池里每一块盘的同一偏移（`crates/singlefs-core/src/transaction.rs:496-503`），所以 N 块盘内容相同：读到的记录 N × 槽数份，键 = 槽数个。
- 每条记录的点名项数 K：判的那一档取 `JOURNAL_NAMED_ENTRIES_PER_RECORD` = 64（一条 4 KiB 记录装得下的最多）；K = 1 是几何敏感性那一档（第八节）。

岔路单（`research/prompts/m3-close-c653-forks.md`，逐行抄）：

```
# C653（扫环胶水内存随盘数涨）岔路单

出处：`.claude/kb/checks-owed.md` 里 `| C653 |` 那一行；去向里程碑三七·2。被测对象：`crates/singlefs-core/` 里 `scan_journal`（调用点 `crates/singlefs-core/src/mounted_read.rs`、`crates/singlefs-core/src/system_configuration_repair.rs`）与它交给的扫环核 `crates/singlefs-replay-prefix` 的 `first_copy_of_each_record_key`。

| 岔路 | 候选（各自的定义） | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| 甲 上界 | 甲一：`scan_journal` 峰值的上界按盘数倍数登记（峰值 ÷ 单盘峰值 ≤ 登记的倍数）；甲二：按绝对字节登记（默认环几何、登记盘数上限下的峰值 ≤ 登记的字节数） | 在默认环几何（每盘 196 608 个记录槽）上，峰值随盘数的倍数是否稳定在一条直线上：稳定则甲一与甲二等价、取甲一；不稳定（随盘数超线性或随记录填满程度变）则只有甲二判得住 | 默认环几何上 1、2、4 盘（记录槽填满与半满两档）各量一次峰值，判别力自证（胶水收齐之后不释放）在同一几何上红 | 开着 |
| 乙 改法 | 乙一：核改成逐盘交、先到先得只证单步（`first_copy_of_each_record_key` 的证明义务改成逐盘增量）；乙二：核不改，胶水逐盘合并（每读完一盘就并进表、释放那一盘的 Vec）；乙三：都不改，只登记上界 | 乙二之后峰值随盘数的倍数是否降到接近 1（降到则乙二买得到内存、证明强度不变，乙一不必）；乙二若因核的接口要求一次交齐而做不到，才比乙一的证明义务改动量 | 乙二在原型上量到峰值倍数，且核的接口是否允许逐盘合并（读码定） | 开着 |
```

未定项：

- 甲二「登记盘数上限」：`crates/` 里没有池的盘数上限（`grep -rn 'MAX_DEVICE\|MAXIMUM_DEVICE\|DEVICES_PER_POOL\|device_count.*上限\|盘数上限' crates/singlefs-format/src crates/singlefs-core/src --include=*.rs | wc -l` 得 0；`device_count` 是 u32，`crates/singlefs-core/src/system_configuration.rs:278`）。这一轮只量 1–4 盘，第六节交 1–4 盘每格的峰值；登记盘数上限大于 4 时甲二的字节数是外推、推的。上限取几由主 agent 定。
- 乙一「证明义务改动量」：只在乙二判做不到时才要（岔路单乙行）。它要一份改过的 Verus 核与证明，不是内存量，这份装置不量；要量时由主 agent 另派（第五节末「分段」）。

## 二、被测条款与它引的定义

`python3 scripts/kb/quote-kb.py /home/fy5090/.local/share/singlefs/work/m3-close-b16-c653-design/quoted-clause.md '.claude/kb/checks-owed.md~^\| C653 \|' '.claude/kb/decisions/23-journal的角色与格式.md@#### 已定项 19：空发布记录的事务号与提交标记、`previous_hash`、环长'` 抽出、回读逐字节一致后追加。条款引的「默认环」定义在 D23 已定项 19 ③，整节抄；`scan_journal`、`first_copy_of_each_record_key` 是代码，见第三节。

**出处 `.claude/kb/checks-owed.md:411-411`（整段抄，未转述）**

```markdown
| C653 | 扫环胶水内存随盘数涨 | 扫环那一截进 Verus 之后，胶水 `scan_journal` 先把每块盘上每一份自证过的记录收进一个 Vec、连同键表交给扫环核 `first_copy_of_each_record_key` 建表，表建完 Vec 才释放，峰值约为盘数倍：攻方在原型上量出两盘 2.01 倍、四盘 3.93 倍（环 8192 槽，每条记录 67 个点名项）；线性外推到默认环（每盘 196 608 个记录槽），改前约 0.92 GB，改后两盘约 1.86 GB、四盘约 3.63 GB（推的，没在这个尺寸上量）。功能不错，代价随盘数涨 | 在默认环几何上经内存包装量 `scan_journal` 的峰值，随盘数的倍数超过登记的上界判红；判别力自证：胶水改成收齐之后不释放，必须红。改法二选一：核改成逐盘交、先到先得只证单步，或胶水逐盘合并，要在证明强度与内存之间取舍 | 取舍没定（代码轮第一轮判决定当时不改）；上界取多少没定；去向：里程碑三七·2（扫环核胶水内存随盘数涨：上界与改法）（五·11，2026-10-12 判）。 | 2026-10-02 `research/prompts/m3-formal-code-r1-main-verification.md` 第二节「Vec 收齐再建表的内存」一格、第三节第 6 条；攻方 `m3-formal-code-r1-opus-output.md` 量内存那一节 |
```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:532-548`（整段抄，未转述）**

```markdown
#### 已定项 19：空发布记录的事务号与提交标记、`previous_hash`、环长

**定案**：

- **① 空发布记录**：事务号 0 保留给不承载事务的记录，提交标记写 1；不进实例表 W 的 max，不进「丢掉提交标记没出现的尾巴」判定。
- **② `previous_hash`** = CRC32C(**本实例内逻辑前一条**记录的头，其中 `header_csum` 那 32 字节按零参与)；**本实例写出的第一条记录恒 0**；写路径**不读盘**。反向链是**按实例**的——I-8.3（重放前缀严格连续) 逐字「跨实例边界即停」，重放本来就不会从一个实例的记录接到另一个实例的记录上，实例边界另有实例代号挡着；取「环上前一格的记录头」那一种，写路径每写一条记录就要先把环上那一格读回来，买到的只是「跨实例也能连上」，而那条链没有任何判定在用。字节表里 w1 那一行写 0 的理由是「本实例第一条」，不是「环上无前一条」。
- **③ 环长**：环长是 **mkfs 参数**，**默认 768 MiB**；约束 **环 ≤ 设备容量 ÷ 4**，越界**拒绝 mkfs**。系统配置里仍只存「环长度（字节）」这一个字段，不另存容量或比例。**挂载时有效 `T_dirty` = min(`T_dirty` 字段, 环长 ÷ F)**——系统配置里 `T_dirty` 仍写 2 GiB，默认 768 MiB 环下有效值 = min(2 GiB, 256 MiB) = 256 MiB。单元区起始槽号随环长走，默认环下是 784 MiB（槽 50176）。`T_dirty` 的夹取是 I-8.1（环几何够大） 的直接推论——环装不下 `F ×` 最坏占用时，能脏的字节数必须跟着降，否则一次发布发不出去。

**射程**：③ 的「参数 + 上界」让大盘上 768 MiB 只占容量的百分之几、小镜像按参数缩，`环 ≤ 容量 ÷ 4` 挡住「环吃掉整个盘」；⚠️ 连带：`.claude/kb/layout/01-first-txn.md` 的预想镜像是 **4 GiB**（1 GiB 的镜像在这条约束下装不下 768 MiB 的环）。实现：`crates/singlefs-core/src/make_filesystem.rs` 收环长参数、按最小那块盘的容量 ÷ 4 拒绝越界，默认值是 `crates/singlefs-format/src/lib.rs` 的 `JOURNAL_RING_DEFAULT_BYTES`。 挂载读者一侧另判环长：在飞上限（环槽数 ÷ F，已定项 18）≥ 1，且环末端不越过编译期单元区起点（768 MiB，`crates/singlefs-format/src/lib.rs` 的 `UNIT_AREA_START_SLOT` 与 `JOURNAL_RING_START_SLOT`）；越界整池拒绝挂载（实审 A3a 落地：`crates/singlefs-core/src/recovery.rs` 的 `SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalRingBytesOutsideTheSupportedRange`）。

**依据**：

- E143（一事务一单元下的 journal 账）：按切分纪律，`T_dirty` 满窗要的环是预想 64 MiB 环的 4 倍，要么环 ≥ 768 MiB（F = 3）、要么 `T_dirty` 压小——默认环长与 `T_dirty` 夹取的来处。
- ①、②、③ 都是用户弹窗定案，没有走三方论证，可推翻；主 agent 推荐的也是这几档，原话在变更史。

**欠**：无。

```

## 三、实现今天的样子

主 agent 派发时给的被测对象之外，本节是设计员读码的观测（2026-10-22 的工作区；`sha256sum crates/singlefs-core/src/recovery.rs` = `64fdec16e110859be46811c1af66754b460891eff63abd9a1d8ff8e2037006e1`，`crates/singlefs-replay-prefix/src/lib.rs` = `c04dfaf6ea5f93bfac5525f501f164d46256c18dae53b397f905971916bf0318`，`crates/singlefs-core/src/journal.rs` = `28d557be537bafe8bd53ca6817005de9c027f40b521500b4a7056c55444b9b7c`）。

- 胶水 `scan_journal`：`crates/singlefs-core/src/recovery.rs:4431-4484`（文档注释起到函数末行；`awk 'NR>=4431 && NR<=4484' crates/singlefs-core/src/recovery.rs | sha256sum` = `0ca338aaafcfff2796656134c9246295e33d58503f439d61588a9cb394697c55`）。做法：`Vec::new()` 起一个 `scanned_records_in_read_order`，按 `reader.device_identities()` 的盘次序、每盘环内偏移次序逐槽 `reader.read` 4096 字节、`JournalRecord::parse` 过了就 push；全部盘读完之后摊出 `scanned_record_keys: Vec<(u32, u64)>`，交 `first_copy_of_each_record_key` 得 `first_copies: BTreeMap<(u32,u64), usize>`；再 `into_iter().enumerate()` 把位置等于 `first_copies` 里那一份的记录 move 进 `records: BTreeMap<(InstanceGeneration,u64), JournalRecord>`，别的丢掉；交回 `records`。所以全部盘的全部记录（连点名项的堆）在建表之前同时在内存里；记录是 move 进表的，点名项的堆不复制；Vec 的缓冲在循环结束时才放。
- 没有提示时扫整环：`journal_record_offsets_hint` 交 `None` 就逐槽列到 `min(环长, 盘字节数 − 环起点)`（同文件 4447-4460）。文件后端的池盘答 `None`（`recovery.rs:302-368` 切片那一份的 `journal_record_offsets_hint` 转给盘；注释「真盘答 `None`、全环扫描」）。
- 读口子 `PoolReader`：`recovery.rs:123-184`。每次 `read` 交一份新分配的 `Vec<u8>`（切片那一份 `recovery.rs:310-320`）。
- 扫环核 `first_copy_of_each_record_key`：`crates/singlefs-replay-prefix/src/lib.rs:4025-4058`（注释 4025-4028，函数 4029-4058；`awk 'NR>=4029 && NR<=4058' … | sha256sum` = `60e0648acc3f3a3a730b3c1731781cf0eda6e28068a56b6e0aad8d58dc1d2d41`）。入参整份 `&[(u32, u64)]`，交回每个键第一次出现的位置；`ensures first_copies_follow_the_first_come_rule(scanned_keys@, scanned_keys@.len() as int, first_copies@)` 与 `each_kept_copy_stands_for_one_record_key(first_copies@)`。spec 在 `lib.rs:1021-1049`：`first_copies_follow_the_first_come_rule(scanned_keys, end, first_copies)` 带一个 `end` 参数，循环不变式在每个前缀 `position` 上都成立（`lib.rs:4046-4048`）。
- 读码答岔路单乙行「核的接口是否允许逐盘合并」要的事实（只列事实，判不判归主 agent）：核的签名一次收整份键表，不收「已有的表 + 这一盘的键」；ensures 只对交进来的那一整份键表说话。spec 本身对任一前缀 `end` 有定义，某位置是不是首份只看它前面的键，所以「每读完一盘，拿到此为止的全部键再调一次核」不改核就能逐盘判出这一盘里哪些是首份；「把这一盘的首份并进已有的表」那一步落在胶水里，不在 Verus 里。
- 记录在内存里的样子：`JournalRecord` `crates/singlefs-core/src/journal.rs:149-174`，最后一个字段 `named: Vec<NamedUnit>`（173 行）；`parse` 用 `(0..named_count).map(…).collect()` 建点名项（`journal.rs:373-375`，容量恰好等于项数）。`NamedUnit` `journal.rs:94-100`；`LocationEntry` `crates/singlefs-core/src/pointer.rs:15-19`；`PointerHead` `pointer.rs:102-105`；`NodePointer` `pointer.rs:283-288`。
- 记录的编码与自证：`JournalRecord::to_bytes` `journal.rs:216-282`，`parse` 从 `journal.rs:291` 起；整条校验和是 CRC-32C 写进 32 字节字段（`crates/singlefs-core/src/checksum.rs:65-80`），每次核都 `to_vec` 一份 4096 字节（瞬时）。
- 格式常量：`crates/singlefs-format/src/lib.rs` 的 `SLOT_BYTES` 14 行、`JOURNAL_RECORD_BYTES` 241 行、`JOURNAL_NAMED_ENTRIES_PER_RECORD` 258 行、`JOURNAL_RING_START_SLOT` 261 行、`JOURNAL_RING_DEFAULT_BYTES` 265 行、`TEST_IMAGE_DEFAULT_BYTES` 338 行。
- 记录写到每一块盘：`crates/singlefs-core/src/transaction.rs:496-503`（`CommitStep::WriteJournalRecordToEveryDevice` 对 `self.devices` 逐块写同一偏移）。
- 调用点：只读挂载 `crates/singlefs-core/src/mounted_read.rs:655`；系统配置修复 `crates/singlefs-core/src/system_configuration_repair.rs:258`；可写挂载读阶段 `crates/singlefs-core/src/mount.rs:8050`（经 `ReadStageCache`）；`mount.rs:8197`；恢复内部 `recovery.rs:7156`、`recovery.rs:7387`。
- 观测（不进本实验的量，见第十一节末与交回报告）：可写挂载读阶段扫环经 `ReadStageCache`（`mount.rs:7836` 起的结构，`read` 在 `mount.rs:7859-7876`），读到的不全 0 的字节按 (盘, 偏移, 长度) 整份留进 `bytes_read_by_placement`；扫一遍环，环里每一份自证过不过的非 0 记录字节都留在那张表里，这一截不在 `scan_journal` 里、岔路单两行都没问到它。
- 仓里另一份进程内堆计数的口径（`mutation-sampling.md` 第五类）：`crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs:150-206` 的计数分配器按 `Layout` 的字节数记（不含 malloc 的块头），`realloc` 记新旧之差，量窗之前 `reset_peak_to_in_use`、峰值减量窗起点（同文件 208-223 行）。本实验的分配器那条互核路径用同一个口径；只读了它的写法，没读它的结果。

## 四、跑之前已经存在的数

1. **条款正文自带的数**（第二节抄的 C653 那一行）：攻方原型上两盘 2.01 倍、四盘 3.93 倍（环 8192 槽、每条记录 67 个点名项）；线性外推到默认环「改前约 0.92 GB，改后两盘约 1.86 GB、四盘约 3.63 GB（推的，没在这个尺寸上量）」。对判据的影响：这几个数在设计之前就读到了；第七节把 1.86 GB、3.63 GB 当「出自条款本身」的锚点（不符走「条款可能错」，不作废）。另：条款写「每条记录 67 个点名项」，今天 `JOURNAL_NAMED_ENTRIES_PER_RECORD` = 64（`crates/singlefs-format/src/lib.rs:258`），一条记录装不下 67 项，攻方原型与今天的格式不同。
2. **设计员判问法时自己推的数**（全部是推的，没量过）：脚本 `/home/fy5090/.local/share/singlefs/work/m3-close-b16-c653-design/predict.py`，按手算的 `size_of::<JournalRecord>()` = 376、`size_of::<NamedUnit>()` = 64（第七节 A6 的算式）、`Vec::new()` 加 push 的容量按 2 的幂长、`BTreeMap` 升序插入每片叶留 6 个键，推每格分配器峰值。命令 `python3 -I /home/fy5090/.local/share/singlefs/work/m3-close-b16-c653-design/predict.py`，原样输出：

```
slots_per_device 196608 ring_end_bytes 822083584 min_device_bytes_ring_le_quarter 3221225472
leaf_rec 4328 leaf_pos 280
K=64 full baseline     N1= 0.985GiB N2= 1.830GiB N3= 2.767GiB N4= 3.520GiB r2= 1.86 r3= 2.81 r4= 3.57 dev3= +3.3% dev4= +0.0% slope=0.857
K=64 full baseline_pc  N1= 1.735GiB N2= 2.580GiB N3= 3.517GiB N4= 4.270GiB r2= 1.49 r3= 2.03 r4= 2.46 dev3= +2.6% dev4= +0.0% slope=0.487
K=64 full split        N1= 0.985GiB N2= 1.738GiB N3= 1.741GiB N4= 1.744GiB r2= 1.76 r3= 1.77 r4= 1.77 dev3=-43.1% dev4=-86.0% slope=0.003
K=64 full split_pc     N1= 0.985GiB N2= 1.830GiB N3= 2.767GiB N4= 3.520GiB r2= 1.86 r3= 2.81 r4= 3.57 dev3= +3.3% dev4= +0.0% slope=0.857
K=64 half baseline     N1= 0.493GiB N2= 0.915GiB N3= 1.383GiB N4= 1.760GiB r2= 1.86 r3= 2.81 r4= 3.57 dev3= +3.3% dev4= +0.0% slope=0.857
K=64 half baseline_pc  N1= 0.868GiB N2= 1.290GiB N3= 1.758GiB N4= 2.135GiB r2= 1.49 r3= 2.03 r4= 2.46 dev3= +2.6% dev4= +0.0% slope=0.487
K=64 half split        N1= 0.493GiB N2= 0.869GiB N3= 0.871GiB N4= 0.872GiB r2= 1.76 r3= 1.77 r4= 1.77 dev3=-43.1% dev4=-86.0% slope=0.003
K=64 half split_pc     N1= 0.493GiB N2= 0.915GiB N3= 1.383GiB N4= 1.760GiB r2= 1.86 r3= 2.81 r4= 3.57 dev3= +3.3% dev4= +0.0% slope=0.857
K= 1 full baseline     N1= 0.247GiB N2= 0.354GiB N3= 0.552GiB N4= 0.566GiB r2= 1.43 r3= 2.23 r4= 2.29 dev3=+16.6% dev4= +0.0% slope=0.431
K= 1 full baseline_pc  N1= 0.259GiB N2= 0.365GiB N3= 0.563GiB N4= 0.578GiB r2= 1.41 r3= 2.18 r4= 2.23 dev3=+16.3% dev4= +0.0% slope=0.411
K= 1 full split        N1= 0.247GiB N2= 0.262GiB N3= 0.265GiB N4= 0.268GiB r2= 1.06 r3= 1.07 r4= 1.08 dev3= -4.4% dev4= -8.8% slope=0.012
K= 1 full split_pc     N1= 0.247GiB N2= 0.354GiB N3= 0.552GiB N4= 0.566GiB r2= 1.43 r3= 2.23 r4= 2.29 dev3=+16.6% dev4= +0.0% slope=0.431
caps {98304: 131072, 196608: 262144, 294912: 524288, 393216: 524288, 589824: 1048576, 786432: 1048576}
```

   臂名对照：baseline = 基线臂，baseline_pc = 基线的阳性对照，split = 乙二臂，split_pc = 乙二的阳性对照（第五节）。r2/r3/r4 = N 盘 ÷ 1 盘；dev3/dev4 = N 盘对过 1、2 盘两点的直线的偏离；slope = (P4 − P2) ÷ (2·P1)。
   对判据的影响，照实写：
   - 第六节的门槛是**在推出这些数之后**写的。推的数落得离门槛近的有一处：甲-直线行 K = 64 满槽 dev3 推 +3.3%，门槛 5%，只差 1.7 个百分点，这一行的判定对推算误差敏感，量出来翻面不算意外。其余推的数离门槛远：K = 1 dev3 推 +16.6%（门槛 5%）；乙-接近一行 r4 推 1.77（门槛 1.25）；乙-不涨行 slope 推 0.003（门槛 0.10）、它的阳性对照推 0.857。
   - 1、2、4 盘这三个取样点上，读到的记录数 196 608 × N 都是 3 × 2 的幂，Vec 容量向上取到 2 的幂的放大倍数恒为 4/3，所以「过 1、2 盘的直线」在 4 盘上推的偏离是 0.0%：只用 1、2、4 盘判「稳定在一条直线上」，在这条推算下必然判直线。这正是 `mutation-sampling.md` 第六类说的取样点不敏感；第八节因此加 3 盘（放大倍数变 16/9）与 K = 1（内联部分占比变大）两个方向相反的取样点。
   - 推算里甲一（同次自比）的阳性对照 r4 = 2.46，比基线的 3.57 还小：「收齐之后不释放」多占的那一截不随盘数涨，按「峰值 ÷ 单盘峰值」判时它反而更绿。第六节「甲-判别力」把两种上界写法各占一行（同次自比；绝对字节，登记常数那一读与它同一行），就是为了把这件事量出来，不是为了让它出某个结果。
   - 推算里四盘基线约 3.52 GiB、阳性对照约 4.27 GiB；这两个数用来定内存上限与并行份数（第十四节），不当判据。
3. **默认环每盘槽数** 805 306 368 ÷ 4096 = 196 608：出自条款与 D23 已定项 19 ③，另用命令核过（第七节 A1）。
4. **别处的口径**：读了 E183 装置计数分配器的写法（第三节末条），没读它的结果。没读 `.claude/kb/experiments/`、`research/results/` 下任何文件；搜条款用词只在 `.claude/kb/` 里搜，带 `--exclude-dir=experiments --exclude=experiments.md --exclude=experiments-history.md`（第十三节命令原样），没有全仓搜。

## 五、臂、阳性对照、真实基线

**装置写在哪（写死）**：入库装置，`crates/singlefs-checker-tier/src/bin/e197_scan_journal_glue_memory_by_device_count.rs`。理由：这两条岔路问的就是 `crates/` 里这份 `scan_journal` 与扫环核的性质，另写一份独立模型答不了；派发提示也要求直接调实现里的 `scan_journal`。跑格的驱动与出表脚本放 `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/`，产物进 `research/results/`。
装置要读 `crates/` 的这几个常量，执行员逐个写成本地常量、旁边 `assert_eq!` 回比 crates 的值，不直接引：`singlefs_format::JOURNAL_RING_DEFAULT_BYTES`（805 306 368）、`JOURNAL_RECORD_BYTES`（4096）、`JOURNAL_RING_START_SLOT`（1024）、`SLOT_BYTES`（16 384）、`JOURNAL_NAMED_ENTRIES_PER_RECORD`（64）、`TEST_IMAGE_DEFAULT_BYTES`（4 294 967 296）。`scan_journal`、`first_copy_of_each_record_key`、`JournalRecord::to_bytes` / `parse`、`PoolReader` 是被测与被调的代码，照原样调。

**装置的池**（各臂共用）：一个只按 (盘, 偏移) 现算字节、不存镜像的读者，实现 `PoolReader`。盘 N 块，每块 `device_size_in_bytes` = 4 GiB；`journal_record_offsets_hint` 交 `None`（与文件后端的真盘同，扫整环）；`read` 落在环里第 i 槽（0 起）且 i < 填满槽数时交 `JournalRecord::to_bytes` 现编的一条（实例 1、计数器 i + 1、K 个点名项、fsid 与系统配置相符，每块盘同一槽字节相同），别处交全 0；读者自己数 `read` 被调了几次，并在每块盘最后一槽被读到时记一次分配器此刻的在用字节（第八节轨迹）。读者不缓存任何一槽的字节：池本身占的内存由「空跑」格量出来扣掉。系统配置照 `crates/singlefs-core/src/system_configuration.rs:808-830` 那份样子建，`journal_ring_bytes` 取默认环长。

**臂**（每条臂交回的表都要与 `crates/` 的 `scan_journal` 在同一个池上交回的逐键逐值相同，第十一节停机条款）：

| 臂 | 名字（产物里的 arm=） | 怎么做 | 做完会怎样（推得出的那一句） |
|---|---|---|---|
| 真实基线（乙三「都不改」就是它） | baseline | 直接调 `singlefs_core::recovery::scan_journal` | 第三节：全部盘的全部记录连同点名项堆在建表之前同时在内存里 |
| 乙二：核不改，胶水逐盘合并 | split | 按 `device_identities()` 次序一盘一盘做：①这一盘读到的记录收进这一盘自己的 Vec；②这一盘的键接到全局键表后面（键表一直留着，每键 16 字节）；③先放掉上一盘调核交回的表，再拿**到此为止的全部键**调不改的 `first_copy_of_each_record_key`；④这一盘的 Vec `into_iter().enumerate()`，全局位置 = 前面各盘记录数之和 + 盘内序号，等于表里那一份的 move 进 `records`；⑤这一盘的 Vec 放掉，再读下一盘 | 任一时刻只有一盘的 Vec 在内存里（⑤在下一盘的①之前）；交回值与基线相同：核的 spec 对任一前缀成立，一个位置是不是首份只看它前面的键（第三节） |

对面那条臂写成支持它的人认的样子：乙二取的是「核一行不改、核的 ensures 对交进去的整份前缀键表照样成立」的强形。没登记的两种写法：每盘只拿本盘的键调核、跨盘先到先得写进胶水——核管不到跨盘那一步，证明强度比岔路单乙二的「证明强度不变」弱，支持乙二的人不认；两遍读（先只收键调核、再读一遍只解首份）——不在岔路单乙二的定义（「每读完一盘就并进表、释放那一盘的 Vec」）里，列进交回报告的待认项，不量。乙一不是这份装置的臂（第一节未定项）。甲一、甲二是上界的写法，不是代码臂，用基线臂与阳性对照的数在第六节各判一行。

**甲一的分母有两种读法**（文件头那句）：
- 读法（甲一·登记常数）：「单盘峰值」是今天基线 1 盘那一格量到之后登记下来的常数，以后判别的代码时分母不跟着变。这一读法下上界 = 倍数 × 常数 = 一个字节数，与甲二是同一个判据（第六节「甲-判别力·绝对字节」一行同时就是它）。

**阳性对照**（每条臂各一份，都在同一几何上跑）：

| 对照 | 名字 | 怎么做 | 做完会怎样 | 必须红在哪一行 |
|---|---|---|---|---|
| 基线臂的「收齐之后不释放」 | baseline_pc | 与基线同一份胶水（照 `recovery.rs:4436-4484` 抄进装置），只改最后一步：不 `into_iter` move，按引用遍历、把首份 `clone()` 进 `records`；`scanned_records_in_read_order` 留到交回之前才放 | 表建完时 Vec 连同全部点名项堆还在，另多一份首份的点名项堆：多占 槽数 × K × `size_of::<NamedUnit>()` 字节，与盘数无关 | 甲-判别力·绝对字节（第十节 F1）；甲-判别力·同次自比照实报 |
| 乙二臂的「逐盘不释放」 | split_pc | 与乙二臂同，只改④⑤：非首份不丢，push 进一个留到交回之前的 Vec | 每盘的非首份连同点名项堆一直留着，峰值随盘数涨，回到基线的形 | 乙-不涨（第十节 F2） |
| 测量路径的对照 | measurement_pc | 空跑格之外，在量窗里分配并逐页写一份 256 MiB 的 `Vec<u8>` | 分配器峰值多 268 435 456 字节；cgroup 峰值多 256 MiB 左右 | 第七节 A8（第十节 F5） |

**空跑格** empty：建池与系统配置、编一条记录（让编码那段代码进内存），不扫环；每个 (盘数, 槽填满档, 点名项数) 一格，cgroup 峰值从同几何各臂的峰值里扣掉。

**真实基线**：baseline 臂，即今天 `crates/` 的 `scan_journal`。

**分段**：只有一段，第六节每一行在这一段里都量到；顺序照「最先让岔路单某一行够判」排：①停机条款要的核对（第十一节两段 sha256、差分集 D 上各胶水与 `scan_journal` 逐键相同）与锚点 A1–A5、A8；② K = 64 满槽四个臂与空跑格 1–4 盘（甲-直线、甲-判别力、乙-接近一、乙-不涨、乙-做得到够判）；③ K = 64 半满槽（甲-槽填满够判）；④ K = 1 满槽（甲-点名项、几何敏感性够判）。交回后主 agent 对着岔路单判；乙一的证明义务改动量不在这一段里，乙二判做不到时主 agent 另派。

## 六、报哪些量与各自的判据

**量法（RSS 峰值从哪读）**：每格一个进程，经内存包装起：

```
RUN_WITH_MEMORY_CAP_PEAKS=<产物目录>/e197-memory-peaks.tsv \
RUN_WITH_MEMORY_CAP_KEY=e197/<arm>/<盘数>/<full|half>/<K>/<重复号> \
RUN_WITH_MEMORY_CAP_HOST_LOOKUP=off \
bash scripts/hosts/run-with-memory-cap.sh 8G <装置二进制> --arm <arm> --devices <N> --fill <full|half> --named <K> --repeat <r>
```

cgroup 峰值从这份峰值表里这一格的键那一行读（`scripts/hosts/memory_peaks_lib.py` 的读法）：口径是包装在 scope 里的外壳 bash 在命令退出之后读自己 cgroup 的 `memory.peak`，按 MiB 向上取整（`scripts/hosts/run-with-memory-cap.sh` 文件头「峰值表」一段，第 126–127 行），含外壳 bash 与页缓存，偏大不偏小。每格一个键，表里一格一行，不与别的格的近几次混。进程内另打一行结果行（`name=e197 arm=… devices=… fill=… named=… repeat=… reads=… records_returned=… allocator_peak_over_start_bytes=… allocator_live_at_each_device_end_bytes=… allocator_live_after_return_bytes=… vm_hwm_kib=… scan_seconds=…`）：分配器峰值按 E183 那份口径（Layout 字节，`realloc` 记新旧之差，进入扫环之前把峰值重置到此刻在用，交回之后读峰值减起点）；VmHWM 在交回之后、放掉交回值之前读 `/proc/self/status`。每格跑 5 次。
**判的那一个数** P(arm, N, fill, K) = 这一格 5 次 cgroup 峰值的中位数 − 同 (N, fill, K) 空跑格 5 次的中位数。分配器峰值与 VmHWM 只用来互核（第七节 A7、第十一节停机条款），不判。r(N) = P(N) ÷ P(1)（同臂同几何）。
**分辨率**：cgroup 峰值的粒度 1 MiB；下面每行要分的差最小的一处是乙-不涨行的门槛 0.10 × 2 × P(1)，推的 P(1) ≈ 0.985 GiB 时约 200 MiB；甲-直线行门槛 5% × P(4) ≈ 180 MiB；都比粒度大两个数量级。5 次重复之间的散布另判（第十一节作废条款 2%）。

每行各报各的判定，不合取；「判别力自证」一列是把那一行的门槛挪到被比两个数中间时必须翻面的那一对。

| 量 | 怎么算 | 门槛与判定 | 对应岔路单哪一行、取什么值让它翻面 | 判别力自证 |
|---|---|---|---|---|
| 甲-直线（4 盘） | baseline，K = 64，满槽与半满各一：dev4 = (P(4) − [P(1) + 3·(P(2) − P(1))]) ÷ P(4) | \|dev4\| ≤ 5% 判「在直线上」，否则判「不在直线上」，正负号照报（正 = 超线性） | 甲行「是否稳定在一条直线上」：\|dev4\| > 5% ⇒ 甲行翻向「只有甲二判得住」 | 门槛挪到 \|dev4\| 与 0 之间 |
| 甲-直线（3 盘） | 同上，dev3 = (P(3) − [P(1) + 2·(P(2) − P(1))]) ÷ P(3) | 同上 | 甲行同一格；这是第八节的反向取样点，\|dev3\| > 5% 而 \|dev4\| ≤ 5% 时第八节记「不稳定」，甲行不按 4 盘那一行判稳定 | 同上 |
| 甲-槽填满 | baseline，K = 64：max over N∈{2,3,4} \|r_满(N) − r_半(N)\| | ≤ 0.15 判「倍数不随槽填满程度变」，否则判「变」 | 甲行「随记录填满程度变」（读成记录槽填满）：> 0.15 ⇒ 翻向甲二 | 门槛挪到那个最大差与 0 之间 |
| 甲-点名项（不对应甲行：第八节几何敏感性一格，照量） | baseline，满槽：max over N∈{2,3,4} \|r_K64(N) − r_K1(N)\| | ≤ 0.15 判「倍数不随点名项多少变」，否则判「变」 | 甲行「随记录填满程度变」（读成每条记录点名项填满）：> 0.15 ⇒ 翻向甲二；两种读法都不认它时，它是第八节的几何敏感性一格 | 同上 |
| 甲-判别力·同次自比（不判：读法已删，照报） | baseline 与 baseline_pc 同几何：pc 红 ⟺ r_pc(N) > 1.10 × r_base(N)；N = 2、3、4，三档几何各报 | 每个 N 报红 / 不红 | 甲行够判条件「判别力自证在同一几何上红」：K = 64 满槽 4 盘不红 ⇒ 读法（甲一·同次自比）判不出「收齐之后不释放」，不满足条款自带的判别力要求，甲行翻向甲二。这一行不红是正当结果，不是测量失败（测量失败看 F1） | 1.10 挪到 r_pc(4) ÷ r_base(4) 两侧 |
| 甲-判别力·绝对字节 | 同上：pc 红 ⟺ P_pc(N) > 1.10 × P_base(N)；N = 1–4（读法（甲一·登记常数）与甲二同是这一行） | 每个 N 报红 / 不红 | 甲行同一格：K = 64 满槽任一 N 不红 ⇒ 甲二同样判不出，见 F1 | 1.10 挪到 P_pc ÷ P_base 两侧 |
| 乙-接近一（不判：读法已删，照报） | split，K = 64 满槽：r_split(4)；r_split(2)、r_split(3) 照报 | r_split(4) ≤ 1.25 判「降到接近 1」，否则判「没降到」 | 乙行「乙二之后倍数是否降到接近 1」：≤ 1.25 ⇒ 乙二买得到内存、乙一不必；> 1.25 ⇒ 这一读法下没降到 | 门槛挪到 r_split(4) 与 r_split_pc(4) 之间 |
| 乙-不涨 | split，K = 64 满槽：slope = (P_split(4) − P_split(2)) ÷ (2 × P_split(1)) | ≤ 0.10 判「每多一盘峰值不再涨」，否则判「还涨」 | 乙行同一格，「接近 1」读成「不随盘数涨」：≤ 0.10 ⇒ 乙二止住了随盘数涨；> 0.10 ⇒ 没止住 | 门槛挪到 slope_split 与 slope_split_pc 之间 |
| 乙-做得到 | split、split_pc、baseline_pc 三份胶水交回的表与 `scan_journal` 交回的逐键逐值比：全部格 + 差分集 D（第九节） | 全等判「在原型上做得到」 | 乙行「乙二若因核的接口要求一次交齐而做不到」：任一处不等先走第十一节停机条款，两边查完仍不等 ⇒ 乙二这一形做不到，乙行转去比乙一 | M7（第九节）打开时差分集 D 必须不等 |
| 每格用时 | 进程内扫环那一段的单调钟秒数 | 不判 | 附带，够判后不跑（只在跑着的格里顺手打，不为它多跑一格） | — |

乙-接近一与乙-不涨是「接近 1」的两种读法，各占一行、各判各的，交回时两行都报。

## 七、钉绝对值的断言

命令 `python3 -I /home/fy5090/.local/share/singlefs/work/m3-close-b16-c653-design/anchors.py`（脚本 sha256 `9b987d8fb87ffe355d9601a357d9f3c322483e51ecb524a2cea00aa50f875940`），原样输出：

```
A1 196608 0
A2 [196608, 393216, 589824, 786432]
A3 196608 98304
A4 3221225472 822083584 True True
A5 {196608: 262144, 393216: 524288, 589824: 1048576, 786432: 1048576, 98304: 131072, 294912: 524288}
A6 16 59 376
A8 268435456 268500992
```

**出自被测条款本身的**（不符走第十节 F3「条款可能错」，不作废）：

- C1：默认环每盘 196 608 个记录槽（C653 正文；D23 已定项 19 ③ 默认 768 MiB）。装置的本地常量 `RING_SLOTS == 196_608`，并 `assert_eq!(RING_SLOTS, JOURNAL_RING_DEFAULT_BYTES / JOURNAL_RECORD_BYTES)`。
- C2：C653 正文推的「改后两盘约 1.86 GB、四盘约 3.63 GB」。比 P_base(2)、P_base(4)（K = 64 满槽）与 1.86 × 10⁹、3.63 × 10⁹ 字节：落在 ±25% 之内记「相符」，之外记「条款的推算数不符」。条款用的是每条 67 个点名项、今天上限 64（第四节第 1 条），这一差本身约 4.5%，在容差里。

**独立算出、用命令核过的**（不符的那一格作废，第十一节）：

- A1：805 306 368 ÷ 4096 = 196 608，余 0。
- A2：每格读者被调 `read` 的次数恰好 = N × 196 608（1–4 盘：196 608、393 216、589 824、786 432），满槽与半满一样（没有提示，扫整环，全 0 的槽也读）。四个扫环的臂（baseline、baseline_pc、split、split_pc）每格都钉；empty 与 measurement_pc 不扫环，钉读数 0。
- A3：每格交回的表的键数恰好 = 每盘填满槽数（满槽 196 608，半满 98 304），与盘数无关（N 块盘内容相同）。
- A4：盘 4 GiB ≥ 4 × 环长 = 3 221 225 472（D23 已定项 19 ③「环 ≤ 设备容量 ÷ 4」）；环末端 16 MiB + 768 MiB = 822 083 584 ≤ 4 GiB，所以 `scan_journal` 里 `ring_bytes_inside_the_device` 取满环长。
- A5：`Vec::new()` 加 push 到 n 条时的容量（2 的幂，最小 4）：上面一行；A7 的式子用它。
- A8：measurement_pc 格分配器峰值减起点落在 [268 435 456, 268 500 992]（256 MiB 到 256 MiB + 64 KiB）；cgroup 峰值减同几何空跑格落在 [256 MiB, 288 MiB]。

**推的、要执行员用命令核的**（不符不作废：按量到的值代进 A7 的式子，差写进第十二节修订，时点在产物之前）：

- A6：`size_of::<LocationEntry>()` = 4 + 8 + 4 = 16；`size_of::<NamedUnit>()` = 2 × 16 + 1 + 8 + 8 + 10 = 59，按 8 对齐为 64；`size_of::<NodePointer>()` = 16 + 2 × 16 + 4 + 4 = 56；`size_of::<JournalRecord>()` = 7 × 8 + 5 × 56 + 24 + 3 × 4 + 4 = 376（第三节各结构的字段；手算布局，没编译核）。装置开头打出这四个 `size_of`。
- A7：每格分配器峰值减起点 = `predict.py` 的式子按量到的 A6 代入（基线：cap(N·s)·S_rec + N·s·K·S_named + N·s·16 + 两张 B 树按每叶 6 键的叶节点字节；baseline_pc 加 s·K·S_named；split 取第三盘起的那一刻：一张表 + 一盘的 Vec 与点名项 + 键表；split_pc 同基线），落在 ±5% 之内。B 树那一截是推的（标准库升序插入的分裂点），K = 64 时它约占 14%，偏 30% 也只动总数约 4%。出 ±5% 走第十一节停机条款，不作废。
- A9：split 与 baseline 在 1 盘上做的是同一串分配（只有一盘时乙二的①–⑤与基线逐步相同），两者的分配器峰值之差 ≤ 1%、cgroup 的 P 之差 ≤ 3%；出界走停机条款。

## 八、轨迹与几何敏感性

**轨迹**：被判的量是一次扫环调用的内存峰值，每个进程只扫一次，没有「轮」，「为正的轮数」这一格不适用。照「端点不是轨迹」改报调用之内的轨迹，每格结果行都有：
- 每块盘最后一槽被读到那一刻分配器的在用字节（N 个数，读者在那一读里记；基线与 baseline_pc 的峰值在全部盘读完之后建表那一截，split 应当在第二盘起每盘一个平台）；
- 峰值：分配器峰值减起点、VmHWM、cgroup 峰值（第六节）；
- 期末值：交回之后、放掉交回值之前分配器的在用字节减起点（交回的那张表的大小）。
第六节判的 P 只用峰值；轨迹用来核峰值落在哪一截，与第七节 A7 的式子取的那一刻对不对得上（对不上走停机条款 S3）。

**几何敏感性**（`mutation-sampling.md` 第六类）：判决那几格默认取 K = 64、满槽、1/2/4 盘；另在下面三个方向上重跑第六节每一行：

| 取样点 | 变的旋钮 | 方向 | 预期它会动哪一行（推的，第四节） |
|---|---|---|---|
| 3 盘 | 盘数取非 2 的幂 | 读到的记录数 589 824 → Vec 容量 1 048 576，放大 16/9；1、2、4 盘恒为 4/3，方向相反 | 甲-直线（3 盘）推 +3.3%（K = 64）、+16.6%（K = 1） |
| K = 1 | 每条记录点名项数 | 点名项堆从约占八成降到约占两成，内联与 B 树那一截占比升，与 K = 64 方向相反 | 甲-点名项、甲-直线（3 盘）、乙-接近一（推 1.08） |
| 半满槽 | 每盘填满槽数 | 记录数减半；容量放大倍数仍是 4/3 | 甲-槽填满（推不动） |

每一行在 K = 64 满槽与每个反向取样点上各报判定；任一取样点上翻面记「不稳定」（写明哪一行、哪个取样点），不翻记「两点相同」。K = 1 上 baseline_pc 比基线只多 196 608 × 1 × 64 字节（约 12 MiB），推的 P 比约 1.05，在 1.10 的门槛之下：K = 1 上甲-判别力两行不红是这条对照的形状所致，照报为几何敏感性，不进 F1。
**判别力自证**：出表脚本对第六节每一行把门槛挪到被比两个数中间重判一次，必须翻面；不翻的那一行记「这一几何上门槛分不出」，不判。

## 九、变异

给执行员的变异，每条写明在哪个取样点上改变输出、在哪个取样点上不变（不敏感的点不拿来判抓没抓到）：

| 变异 | 改哪 | 敏感的取样点与应红的断言 | 不敏感的取样点 |
|---|---|---|---|
| M1 | baseline_pc 改回 move（不 clone，Vec 照常放） | K = 64 满槽任一 N：甲-判别力·绝对字节由红转不红，F1 触发 | K = 1（本来就差约 5%） |
| M2 | split 的⑤不放这一盘的 Vec（变成 split_pc 的形） | K = 64 满槽 4 盘：乙-不涨 slope 由推的 0.003 变推的 0.857，判定翻面 | 1 盘（只一盘时无差） |
| M3 | 读者的 `journal_record_offsets_hint` 交 `Some(空表)` | 每格：A2 读数 0 ≠ N × 196 608，A3 键数 0 | 无 |
| M4 | 池的生成不看填满档（半满也写满） | 半满各格：A3 键数 196 608 ≠ 98 304 | 满槽各格 |
| M5 | 池的生成不看 K（总写 64 项） | K = 1 各格：A7 推的 0.247 GiB 变 0.985 GiB，出 ±5% | K = 64 各格 |
| M6 | 计数分配器的 `realloc` 只加新大小、不减旧大小 | K = 1 满槽 4 盘：Vec 历次扩容约 394 MB 被多记，A7 出界约 +65%；K = 64 满槽 1 盘约 +9.7% | measurement_pc（一次 `with_capacity`，不 realloc） |
| M7 | split 改成每盘只拿本盘的键调核、跨盘不查（后到的同键覆盖前一份） | 差分集 D 的 D2：交回值取到第二盘那一份，乙-做得到判不等 | 四档镜像格（各盘同一槽字节相同，覆盖之后值不变） |

差分集 D（小环，环长 64 × 4096 字节，盘 4 GiB，不给提示），四个胶水臂各与 `scan_journal` 逐键逐值比：
- D1：2 盘，第一盘 0–31 槽有记录，第二盘 16–63 槽有记录，重叠处字节相同；
- D2：2 盘，第 4 槽两盘都是（实例 1，计数器 5），事务号不同，`scan_journal` 留第一盘那一份；
- D3：3 盘，第一盘全 0、第二盘半满、第三盘满；
- D4：1 盘，计数器 3 的记录在第 2 槽与第 40 槽各有一份、内容不同，按偏移次序留前一份。

## 十、失败条款

- **F1 测量分不出「收齐之后不释放」，整轮作废**（阳性对照，`test-discipline.md`「两个对照组都要跑」）。
  什么观测会让它触发：K = 64 满槽、N ∈ {1, 2, 3, 4} 里任一个 P_pc(N) ≤ 1.10 × P_base(N)（推的差约 0.75 GiB，是 P_base 的 21%–76%）。
  不归它的：甲-判别力·同次自比那一行不红，是这种上界写法的性质，照第六节记正当结果，不触发 F1。
- **F2 测量分不出逐盘释放，乙的三行（乙-接近一、乙-不涨、乙-判别力自证）作废**。
  什么观测会让它触发：K = 64 满槽 split_pc 的 slope ≤ 0.10，或 r_split_pc(4) ≤ 1.25。
- **F3 条款的推算数可能错**（不作废；交回时提议改 C653 正文那几个数）。
  什么观测会让它触发：第七节 C2 里 P_base(2) 落在 1.86 × 10⁹ 字节 ±25% 之外，或 P_base(4) 落在 3.63 × 10⁹ 字节 ±25% 之外。
- **F4 条款「峰值约为盘数倍」可能错**（不作废；照报，K = 1 那一档另记几何敏感性）。
  什么观测会让它触发：K = 64 满槽 r_base(4) 落在 [3.0, 4.4] 之外，或 r_base(2) 落在 [1.6, 2.2] 之外。
- **F5 测量路径坏，整轮作废**（校验路径要先证明会红，`evidence-discipline.md`「校验路径本身也要证明它会红」）。
  什么观测会让它触发：measurement_pc 的分配器峰值减起点出 [268 435 456, 268 500 992]，或它的 cgroup 峰值减同几何空跑格出 [256 MiB, 288 MiB]。
- 结果反过来时怎么改结论（跑前写下）：甲-直线两行都在 5% 内、甲-槽填满与甲-点名项都 ≤ 0.15、甲-判别力·同次自比在 K = 64 满槽 4 盘上红，就照岔路单交「甲一与甲二等价、取甲一」；乙-接近一判「降到」，就交「乙二买得到内存、乙一不必」，即使与第四节推的方向相反。

## 十一、作废条款与停机条款

**作废**：
- V1：内存包装退 245、250–254 的那一次不算结果，同一格重跑；退 250（撞了 8G）的那一格把上限改 12G 重跑，改动写进第十二节。
- V2：一格 5 次 cgroup 峰值的（最大 − 最小）÷ 中位数 > 2%，那一格作废、重跑 5 次；再超，用到这一格的每一行报「不稳定」，不判。
- V3：空跑格 cgroup 峰值 > 128 MiB（池本该不占内存），同 (N, fill, K) 的各格作废，先查装置的池。
- V4：A2、A3、A8 不符的那一格作废；A1、A4、A5 不符（全局常量或算式错）整轮作废。
- V5：结果行读不到或缺字段，那一次作废，不当 0（`test-discipline.md`「读不到 ≠ 读到 0」）。

**停机**（`implementation-first.md` 第 4 条：两边都查，既不作废也不当结果；停下写进报告交回主 agent）：
- S1：开跑之前核实现还是登记时那一份：`awk 'NR>=4431 && NR<=4484' crates/singlefs-core/src/recovery.rs | sha256sum` 应为 `0ca338aaafcfff2796656134c9246295e33d58503f439d61588a9cb394697c55`，`awk 'NR>=4029 && NR<=4058' crates/singlefs-replay-prefix/src/lib.rs | sha256sum` 应为 `60e0648acc3f3a3a730b3c1731781cf0eda6e28068a56b6e0aad8d58dc1d2d41`。行号挪了而函数正文没变的（`grep -n` 找到函数新位置、同样长的那一段 sha256 相同）照跑并记一笔；正文变了就停：第三节与 baseline_pc、split 的抄本要按新实现重写。
- S2：baseline_pc、split、split_pc 任一份胶水在任一格或差分集 D 上交回的表与 `scan_journal` 的不逐键逐值相同。
- S3：A7 出 ±5%；A9 出界；或两条量法互不包含——分配器峰值减起点 > P + 32 MiB，或 P > 1.15 × 分配器峰值减起点 + 64 MiB；或第八节轨迹里峰值所在那一截与 A7 取的那一刻对不上。

**够判停机**：第五节「分段」的顺序跑；岔路单甲、乙两行都够判就停，没跑到的格与量逐条标「够判后未跑」。甲行要 K = 64 满槽与半满（岔路单够判条件）加第八节两个反向取样点；乙行要 K = 64 满槽四个臂与差分集 D。
**不在本实验里的观测**：第三节末条的 `ReadStageCache` 在可写挂载扫环时整份留着读到的记录字节，这一截不在 `scan_journal` 里，岔路单没有对应的行，不量；写进交回报告的待认项。

## 十二、修订

（留空；装置写之后、产物之前由执行员写，只许收严或补臂。）

### 修订一（2026-11-02，装置写之后、产物之前）：补 C881（可写挂载读阶段的读缓存随盘数涨）那一臂；跑法改成统一任务服务分片、峰值读服务记的尝试峰值

谁写、什么时点：主 agent 派设计员写（批次 `records/2026-10-18-里程碑三收尾与近期欠账.md#19`）。时点在产物之前，量的：`ls research/results/ | grep -ci e197` 得 0，`grep -c 'e197\|E197' scripts/experiment-apparatus/replay.sh` 得 0。装置单测：任务 4705（`cargo test --release -p singlefs-checker-tier --bin e197_scan_journal_glue_memory_by_device_count`），`task-server/task log 4705` 原样一行「test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.93s」。这次修订不靠哪个单测读数，依据是两件：
- C881 欠账的会红检查要「与 C653（扫环胶水内存随盘数涨） 同法」量读阶段读缓存，原登记第十一节末「不在本实验里的观测」一段把它排在外面；
- 用户定的规矩（管任务服务的会话转达）：任务里不再套内存包装，峰值读服务记下的那一次尝试峰值；C653 欠账行的会红检查现已写成这一读法（下面「第二节修订」补抄）。

原文一个字不改：第一至十一、十三、十四节原样留着，下面每一处写明它替换或补的是原文哪一句；甲、乙两行的判据、门槛、失败条款不改，只在量法上加要求（「比原来多要求什么」逐条写）。

**另立一臂还是写修订：写修订。**理由：① 同一个装置、同一个池、同一组几何与盘数，C881 正文写「与 C653 同法」，另立实验号要把池、空跑格、量法、停机条款再抄一份，两份产物的基线臂还得互相对齐；② 补的是一条臂加它的阳性对照，甲、乙的判据一个字不动，属于第十二节原文许的「补臂」；③ 产物一次没跑，照 `evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」最后一段办：原判据原样保留，新臂各判各的。代价：C881 那几格要等主 agent 认下面的丙行才跑，甲、乙不等它。

**第一节修订：问题与岔路单补一行（待主 agent 认）**

岔路单 `research/prompts/m3-close-c653-forks.md`（此刻 sha256 `a2ac0ebc78b6dc245395eba9be8a76b70edaff38c33572802d780aa7415d4a56`）只有甲、乙两行，没有 C881 的行；`test-discipline.md`「实验要写清为哪个决定而量、量到哪算够」要每个量对上一行。设计员不写岔路单，下面这一行交主 agent 认：追加进岔路单，或另起问题单 `research/prompts/m3-close-c881-questions.md`（同格式）。认之前执行员不跑丙那几格。

```
| 丙 读缓存上界（C881） | 丙一：可写挂载读阶段读缓存扫环那一截的上界按盘数倍数登记（Q ÷ 单盘 Q ≤ 登记的倍数）；丙二：按绝对字节登记（默认环几何、登记盘数上限下 Q ≤ 登记的字节数） | 在默认环几何（每盘 196 608 个记录槽）上，Q 随盘数的倍数是否稳定在一条直线上：稳定则丙一与丙二等价、取丙一；不稳定（随盘数超线性或随记录槽填满程度变）则只有丙二判得住 | 默认环几何上 1、2、4 盘（记录槽填满与半满两档）各量一次 Q，判别力自证（缓存连读回全 0 的槽也收）在同一几何上红 | 开着 |
```

读法写死（丙行）：
- Q(N, 几何) =「读阶段读缓存扫环那一截」= P(read_stage_cache, N, 几何) − P(baseline, N, 几何)：两条臂同盘数、同几何，P 照第六节与本修订「量法」算。依据：缓存到扫环读完才满，`scan_journal` 自己的峰值在全部读完之后建表那一截（第三节第 1 条），两者同时在；read_stage_cache 臂比基线臂只多缓存。
- 「单盘 Q」与甲一同读：今天 1 盘那一格量到之后登记下来的常数（文件头问法 ①），不做同次自比；同次自比照报、不判。
- 丙只问扫环那一截进缓存的字节。真挂载里择根读的根槽、重放与判 F1 读的点名单元也进同一个缓存（`crates/singlefs-core/src/mount.rs:8657-8678` 的 `read_stage` 里 `choose_root`、`scan_journal`、`replay_journal`、`publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify` 都经 `cache` 读），装置的池没有可择的根、点名单元不在盘上，这两截量不到：Q 是读阶段读缓存的下界，交回时照这句报。

岔路单乙行原文的够判条件比第一节抄的多一句「；乙一只在乙二做不到时量证明义务改动的规模」（`grep -n '乙一只在' research/prompts/m3-close-c653-forks.md` 现查得第 8 行），与第一节未定项第 2 条同义，不改判据。

未定项（修订一）：
- 丙行由主 agent 认（上面）。
- 丙各行在 V2 上限（一格 5 次散布 ≤ 2%）下的最坏噪声可能盖过门槛（下面「分辨率」一段算了）。登记的办法是「区间跨门槛就报分不出、不判」；要不要为丙另加重复次数，由主 agent 看第一轮之后定。
- 编译那一步与装置二进制放哪、两台怎么都读得到，等管任务服务的会话确认（交回报告「要管任务服务的会话确认的」第 6 条）；登记只钉「全部格用同一份二进制」（V9）。

**第二节修订：补抄 C881 整行与 C653 此刻整行**

`python3 scripts/kb/quote-kb.py /home/fy5090/.local/share/singlefs/work/m3-close-b19-e197-design/quoted-clauses.md '.claude/kb/checks-owed.md~^\| C881 \|' '.claude/kb/checks-owed.md~^\| C653 \|'` 抽出（原样回报「✓ 2 段整抄进 …，回读逐字节一致」），出口 sha256 `35ab7eb631e3570842623306cda08a0b91c31ac0a8632b2371def80e71091627`，整份追加如下。C653 那一行的会红检查已改成「经 `task-server/task submit` 交，读服务记下的那一次尝试的内存峰值」，第二节原抄的是改前那一版（「经内存包装量」），原抄留着当登记时的样子。C881 正文写的行号（7836、7870 至 7874、8118）是 2026-10-22 的，今天的行号见下面第三节修订。

**出处 `.claude/kb/checks-owed.md:541-541`（整段抄，未转述）**

```markdown
| C881 | 可写挂载读阶段的读缓存随盘数涨 | 可写挂载读阶段经 `ReadStageCache`（`crates/singlefs-core/src/mount.rs` 第 7836 行起的结构体，`read` 把读到的非 0 记录字节存进 `bytes_read_by_placement` 在第 7870 至 7874 行，第 8118 行 `read_stage_settled_at_most_on_the_one_reread` 建它；2026-10-22 现查）扫环，读到的非 0 记录字节整份留在内存里，活到读阶段判完为止，随盘数涨。这一截不在 `scan_journal` 里，E197（扫环胶水内存随盘数） 的岔路单没有对应的行，没登记。结构体文档注释写的代价是环写满一圈之后为环长乘盘数（默认环两块盘约 1.5 GiB，推的，没量） | 在默认环几何上量这块内存随盘数的倍数，与 C653（扫环胶水内存随盘数涨） 同法 | 去向：里程碑三七·2。 | E197（扫环胶水内存随盘数） 设计员报告 `~/.local/share/singlefs/work/m3-close-b16-c653-design/result.md`「登记里待主 agent 认的项」第 4 条 |
```

**出处 `.claude/kb/checks-owed.md:397-397`（整段抄，未转述）**

```markdown
| C653 | 扫环胶水内存随盘数涨 | 扫环那一截进 Verus 之后，胶水 `scan_journal` 先把每块盘上每一份自证过的记录收进一个 Vec、连同键表交给扫环核 `first_copy_of_each_record_key` 建表，表建完 Vec 才释放，峰值约为盘数倍：攻方在原型上量出两盘 2.01 倍、四盘 3.93 倍（环 8192 槽，每条记录 67 个点名项）；线性外推到默认环（每盘 196 608 个记录槽），改前约 0.92 GB，改后两盘约 1.86 GB、四盘约 3.63 GB（推的，没在这个尺寸上量）。功能不错，代价随盘数涨 | 在默认环几何上把量 `scan_journal` 的命令经 `task-server/task submit` 交，读服务记下的那一次尝试的内存峰值（服务库在 `~/.local/state/singlefs/task-server/` 下、只有服务读写，峰值是尝试记录里 scope 的 `memory.peak`，经 `task-server/task status <号>` 读），随盘数的倍数超过登记的上界判红；判别力自证：胶水改成收齐之后不释放，必须红。改法二选一：核改成逐盘交、先到先得只证单步，或胶水逐盘合并，要在证明强度与内存之间取舍 | 取舍没定（代码轮第一轮判决定当时不改）；上界取多少没定；去向：里程碑三七·2（扫环核胶水内存随盘数涨：上界与改法）（五·11，2026-10-12 判）。 | 2026-10-02 `research/prompts/m3-formal-code-r1-main-verification.md` 第二节「Vec 收齐再建表的内存」一格、第三节第 6 条；攻方 `m3-formal-code-r1-opus-output.md` 量内存那一节 |
```

**第三节修订：实现今天的样子（2026-11-02 现查）**

- `sha256sum`：`crates/singlefs-core/src/mount.rs` = `d6d80473234da43aad0c477c596329478ee714536279e8c101e8fde6b98bad4e`，`crates/singlefs-core/src/recovery.rs` = `e8fbdf8620ba8e856d352963c04b77c361fc96784b6cfd53b18e9b0cbd46ef21`，`crates/singlefs-replay-prefix/src/lib.rs` = `66f08858c51a97a98b8ebbad2209140f7dcd9da0cf1f6b8ab793de242f10a2f4`（三份都比登记时变了）。
- S1 两段今天是「挪了行号、正文没变」：`grep -n 'pub fn scan_journal(' crates/singlefs-core/src/recovery.rs` 得 4937，`awk 'NR>=4932 && NR<=4985' crates/singlefs-core/src/recovery.rs | sha256sum` = `0ca338aaafcfff2796656134c9246295e33d58503f439d61588a9cb394697c55`（与 S1 登记的相同）；`grep -n 'pub fn first_copy_of_each_record_key(' crates/singlefs-replay-prefix/src/lib.rs` 得 4034，`awk 'NR>=4034 && NR<=4063' … | sha256sum` = `60e0648acc3f3a3a730b3c1731781cf0eda6e28068a56b6e0aad8d58dc1d2d41`（相同）。照 S1 原文「照跑并记一笔」。
- 读阶段读缓存 `ReadStageCache`：`crates/singlefs-core/src/mount.rs:8446-8500`（8446 文档注释起，8450 `struct ReadStageCache<`，8457-8464 `empty_over`，8466-8500 `impl PoolReader`）；`awk 'NR>=8446 && NR<=8500' crates/singlefs-core/src/mount.rs | sha256sum` = `58f00a3e70e681cf18cf672e29707b9bee2fe2655bb2ea996a4072f72a5d3f8c`（S4 钉它）。结构是私有的（`struct`，没有 `pub`），checker 档的 bin 调不到。
- `read`（`mount.rs:8473-8490`）：先按 (盘, 偏移, 长度) 查表，有就交回一份 `clone`；没有就读盘，读回的字节里有非 0 的，`clone` 一份插进 `bytes_read_by_placement`（8484-8488），交回原份。读失败与读回全 0 的不收。`journal_record_offsets_hint` 原样转给底下的读者（8491-8499）。
- 建它与用它：`read_stage_settled_at_most_on_the_one_reread`（`mount.rs:8727` 起）在 8732 建一个空缓存，第一遍 `read_stage`（8657 起）经它择根、扫环（8664 `scan_journal(cache, …)`）、重放；判出要重读时第二遍在同一个缓存上重做（8744 一带）。缓存活到这个函数交回为止。
- 键是 `(DeviceIdentity, DeviceOffsetInBytes, usize)`：`DeviceIdentity(pub u32)`（`crates/singlefs-core/src/address.rs:21`）、`DeviceOffsetInBytes(pub u64)`（同文件 29 行）。

**第四节修订：跑之前已经存在的数（修订一新增）**

1. 设计员推的（没量）：`python3 -I /home/fy5090/.local/share/singlefs/work/m3-close-b19-e197-design/anchors_c881.py`（脚本 sha256 `3d60390d67f2535555b7e409a6553f70aa16304ae895fff020cb72c2d7768c57`），原样输出：

```
A10 ring_slots 196608 payload_one_device_full 805306368
A11 leaf_bytes 544 tree_bytes_per_entry 90.667 bytes_per_entry 4186.667
k64 full entries=[196608, 393216, 589824, 786432] payload=[805306368, 1610612736, 2415919104, 3221225472]
k64 full Q_GiB=0.767,1.533,2.300,3.066 P_cache_GiB=1.752,3.363,5.067,6.586 r_Q=1.00,2.00,3.00,4.00 dev3=+0.0000 dev4=+0.0000
k64 half entries=[98304, 196608, 294912, 393216] payload=[402653184, 805306368, 1207959552, 1610612736]
k64 half Q_GiB=0.383,0.767,1.150,1.533 P_cache_GiB=0.876,1.682,2.533,3.293 r_Q=1.00,2.00,3.00,4.00 dev3=+0.0000 dev4=+0.0000
k64 half keeps_zeros_pc entries=[196608, 393216, 589824, 786432] Q_pc_GiB=0.767,1.533,2.300,3.066 Q_pc/Q=2.000,2.000,2.000,2.000 P_pc_GiB=1.260,2.448,3.683,4.826 P_pc/P_cache=1.437,1.456,1.454,1.466
k1 full entries=[196608, 393216, 589824, 786432] payload=[805306368, 1610612736, 2415919104, 3221225472]
k1 full Q_GiB=0.767,1.533,2.300,3.066 P_cache_GiB=1.014,1.887,2.852,3.632 r_Q=1.00,2.00,3.00,4.00 dev3=+0.0000 dev4=+0.0000
C881_clause two_devices_GiB 1.5 bytes 1610612736 band_25pct 1207959552 2013265920
cap_floor_GiB 4 cells_over_floor_minus_64MiB:
  read_stage_cache k64 full N=3 P_alloc=5.067
  read_stage_cache k64 full N=4 P_alloc=6.586
  read_stage_cache_keeps_zeros_pc k64 half N=4 P_alloc=4.826
```

   算式：缓存每项 = 4096 字节的值 + B 树叶的摊分；叶 = 8 + 2 + 2 + 11 × 24 + 11 × 24 = 540，按 8 对齐 544，按装置 A7 同一个口径「升序插入每叶留 6 个键」摊成每项 90.667 字节（键 24 字节是字段重排后 8 + 8 + 4 按 8 对齐，手算，没编译核）。P_cache 用的基线数抄自第四节第 2 条 `predict.py` 的原样输出。对判据的影响，照实写：
   - 推的 Q 恰好随盘数成正比（r_Q = 1、2、3、4，dev3 = dev4 = 0）：缓存按节点分配，没有 Vec 那种按 2 的幂长的放大，第八节给甲挑的反向取样点（3 盘）对 Q 不反向。丙-直线两行在这条推算下必然判「在直线上」，是推算的性质；量出来偏离 5% 以上就是推算漏了什么，照判。
   - 半满几何上阳性对照推 Q_pc ÷ Q = 2.000（每个 N），门槛 1.10，离得远；满槽几何上两者相同（全部槽非 0），阳性对照在满槽上不敏感，所以丙的判别力只在半满几何上判。
   - C881 正文「默认环两块盘约 1.5 GiB」：推的 Q(2) 满槽 = 1.533 GiB；缓存的值那一截恰好 2 × 805 306 368 = 1 610 612 736 字节 = 1.5 GiB。第七节修订把它当「出自条款本身」的锚点。
   - 门槛（5%、0.15、1.10）照抄甲行，写在推出这些数之后。
2. 量的：任务 4705 第 1 次尝试（`task-server/task status 4705` 原样：「第 1 次：本机、上限 4096M、…退 250（memory-cap-hit）、…内存峰值 4.00 GiB」）——撞了上限的那一次，服务记的峰值贴着上限。这是 V6 的来处。
3. 量的：驱动 `drive.py` 现读的峰值不是第六节写的那一个。`drive.py:130-140` 的 `cgroup_peak_bytes` 读峰值表第六列「回收不掉的用量」（文件头 31-32 行弄坏开关 `old-peak-column` 的说明），第六节写的是 scope 的 `memory.peak`；这一改没进第十二节。本修订把量法改回 `memory.peak`（服务记的那一份），驱动这一处照「驱动要改的」删掉。
4. 没读：`.claude/kb/experiments/` 下任何文件、`research/results/` 下任何文件。读了调查员报告 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-experiment-status/result.md` 第 1-76 行里 grep 命中的行与第 234-266 行（E197 一节：装置现状、单测、没有产物，没有实验结果）。

**第五节修订：补一条臂与它的阳性对照**

装置写在哪不变：入库装置 `crates/singlefs-checker-tier/src/bin/e197_scan_journal_glue_memory_by_device_count.rs`。`ReadStageCache` 是私有结构，装置调不到，照 baseline_pc 抄 `scan_journal` 的先例把它抄进装置：照 `mount.rs:8446-8500` 逐字抄，只许三种替换——`crate::address::` 换成 `singlefs_core::address::`、结构名后加装置自己的后缀、`PoolReader` 的路径换成 `singlefs_core::recovery::PoolReader`；抄本前后各一行标记注释。装置另加一条单测：运行时读仓里 `mount.rs` 那一段（按 S4 找到的位置），做完这三种替换，与装置源文件里两行标记之间的抄本逐字相同；不同就红（S4）。`scan_journal` 照原样调 `singlefs_core::recovery::scan_journal`，不抄。

| 臂 | 名字（产物里的 arm=） | 怎么做 | 做完会怎样（推得出的那一句） |
|---|---|---|---|
| C881 臂：今天可写挂载读阶段扫环那一截（真实基线） | read_stage_cache | 在装置的池外面套一层抄来的 `ReadStageCache`（空表起，同 `mount.rs:8732`），调不改的 `scan_journal(&缓存, 系统配置)`；交回之后、读完期末值之前缓存与交回的表都留着（同 `read_stage_settled_at_most_on_the_one_reread`：缓存活到读阶段判完） | 池对每个槽的 `read` 都先经缓存；第一遍全部不命中，落到池上（A2 照旧 N × 196 608 次）；读回非 0 的槽 `clone` 一份进表（`mount.rs:8484-8488`）。N 块盘同一槽字节相同，但键带盘号，所以缓存 N × 填满槽数项、每项 4096 字节；建表那一刻缓存已满，峰值 = 基线的峰值 + 缓存 |
| C881 臂的阳性对照：缓存连读回全 0 的也收 | read_stage_cache_keeps_zeros_pc | 与 read_stage_cache 同，只改抄本 `read` 里一处：去掉「有非 0 字节才收」那个条件，读回的都收 | 半满几何下每盘未写过的 98 304 槽也进缓存，缓存项数与字节翻倍；满槽几何下全部槽本来就非 0，与臂相同（不敏感） |

对面那条臂：丙一、丙二是上界的写法，不是代码臂；read_stage_cache 本身就是今天的实现，没有「支持它的人认不认」的问题。不登记缓存的改法臂（比如扫环不经缓存、或缓存按落点不分盘）：丙行只问上界，没有改法那一列；附带一句，缓存按落点不分盘这一形会让 Q 不随盘数涨，但它改了多盘内容不同时的语义，不是今天的样子，不量。
阳性对照：read_stage_cache 有它自己的那一份（上表），只在 K = 64 半满一个几何上跑（满槽上不敏感，见上；K = 1 满槽同样不敏感）。原有四个扫环臂的阳性对照不变。
空跑格、测量路径对照（measurement_pc）照原第五节，两条新臂共用。

各段的格（原第五节「分段」那一段的格数照旧，补的是丙那几格）：
- K = 64 满槽：read_stage_cache × 盘数 1–4 × 5 次 = 20 格；
- K = 64 半满：read_stage_cache 与 read_stage_cache_keeps_zeros_pc × 盘数 1–4 × 5 次 = 40 格；
- K = 1 满槽：read_stage_cache × 盘数 1–4 × 5 次 = 20 格（第八节几何敏感性）；
- compare 段补：read_stage_cache 在三个几何上、keeps_zeros_pc 在 K = 64 半满上，交回的表与 `scan_journal` 逐键逐值比，4 × 3 + 4 = 16 格（缓存不改交回值，见上表），连原有 36 格共 52 格；差分集 D 四例两条新臂各比一次（D1–D4，共 8 行，原 12 行之外）。

**第六节修订：量法（替换原第六节「量法」一段与第十一节 V1；判据表各行原样）**

原文：每格经 `scripts/hosts/run-with-memory-cap.sh 8G` 起，cgroup 峰值从包装写的峰值表读、按 MiB 向上取整。改成：
1. 每格一个尝试：驱动 `drive.py` 的 `segment` 子命令由统一任务服务分片（文件头前 60 行写 `# task-server-shards: segment`）。计划那一步把这一组的格（至多 8 格，服务的 `MAXIMUM_SHARDS` = 8，`task-server/task_server_lib.py:901`）写进 `plan.json`，一片一格；每片起装置二进制一次（只起这一格，不起别的格），不套任何包装，不写上限、线程、挑台参数；合并那一步把各片的结果按片号次序写进这一组自己的产物文件（排他新建，已在就拒）。上限、哪台、撞上限放大都由服务定。
2. 每片的结果行另带四个字段：`task=`（环境 `TASK_SERVER_TASK`）、`attempt=`（`TASK_SERVER_ATTEMPT`）、`shard_host=`（`TASK_SERVER_SHARD_HOST`）、`binary_sha256=`（这一片起的那份二进制）。读码看到服务每次尝试都给前两个，第二台同样带（`task-server/task_server.py` 的 `attempt_command` 里第二台带的 `carried_names`，与起尝试时的 `environment.update(TASK_SERVER_TASK=…, TASK_SERVER_ATTEMPT=…)`；2026-11-02 现查在第 3401、3492 行，这份文件别的会话正在改，行号会挪）。
3. 判的峰值 = 服务记的那一次尝试的 `memory_peak`（字节，scope 的 `memory.peak` 原值；`task-server/task status <号> --json` 里那个子任务那一次尝试的那一栏）。每一件分片任务有了结局之后，主 agent 立即 `task-server/task status <父任务号> --json` 存进 `research/results/e197-<日期>-status-<父任务号>.json`（服务的记录是证据，原样存，不改）；出表脚本按结果行的 `task=`、`attempt=` 去存档里找那一次尝试，读 `memory_peak`、`memory_cap`、`host`、`outcome`。
4. 页缓存：每片先把装置二进制拷进自己的产物目录（`TASK_SERVER_SHARD_OUTPUT_DIRECTORY`）、从拷贝起，让这份二进制的页缓存每一片都记在这一片的 scope 上（不然先跑的那一片记了、后跑的不记，空跑格扣不掉）。驱动自己（python）的那一截每片相同，由空跑格扣掉。
5. P 的算法（原「判的那一个数」一句的推广，全部尝试在同一台上时与原文逐字等价）：E(h, g) = 几何 g 的空跑格在台 h 上那几次 `memory_peak` 的中位数；一格的每次尝试 i 算 d_i = memory_peak_i − E(h_i, g)；P = 这一格 5 次 d_i 的中位数。台 h 上没有 g 的空跑格时取 E(h) = 台 h 上全部几何空跑格的中位数，前提是台 h 上各几何空跑格的中位数之间相差 ≤ 8 MiB；不满足走 V8。V2 的散布改算 (max d_i − min d_i) ÷ 这 5 次 `memory_peak` 的中位数（同一台时与原文相同）。

比原来多要求的：峰值精到字节（原来按 MiB 向上取整）；每个峰值钉到服务记录里的一次尝试（任务号、第几次、哪台、结局），对不上就不算（V7）；空跑格按台扣（原来不分台）；顶着上限的那一次不算（V6）；二进制每片一份拷贝、全部格同一个 sha256（V9）。原文的峰值包含「外壳 bash 与页缓存」，现在包含驱动 python 与二进制拷贝的页缓存，两样都由同台空跑格扣掉；判据各行的门槛一个不动。

**分辨率（修订一，丙行用）**：峰值粒度 1 字节。噪声按 V2 上限算：一格 5 次散布 ≤ 2%，中位数离真值最多约 2%；Q 是两格之差，K = 64 半满 1 盘推 P_cache 0.876 GiB、P_base 0.493 GiB，最坏 ±(0.018 + 0.010) GiB ≈ Q(1) 0.383 GiB 的 ±7%，r_Q(4) 最坏 ±0.3，比丙-槽填满的门槛 0.15 大。所以丙各行一律带区间：用参与那一行的各格 d_i 的 [最小, 最大] 做区间算术，算出这一行统计量的区间；区间整个落在门槛一侧才判，跨门槛的报「分不出」、不判，连区间一起交回。原甲、乙各行不加这一条（不改原判据）；要不要给甲、乙也加，列进交回报告待认项。

**第六节修订：丙行的量与判据**（原表各行不动；下表各行等丙行认了才判，每行带上面「分辨率」的区间，跨门槛报「分不出」）

| 量 | 怎么算 | 门槛与判定 | 对应岔路单哪一行、取什么值让它翻面 | 判别力自证 |
|---|---|---|---|---|
| 丙-直线（4 盘） | K = 64，满槽与半满各一：dev4_Q = (Q(4) − [Q(1) + 3·(Q(2) − Q(1))]) ÷ Q(4) | \|dev4_Q\| ≤ 5% 判「在直线上」，否则「不在」，正负号照报 | 丙行「是否稳定在一条直线上」：> 5% ⇒ 翻向「只有丙二判得住」 | 门槛挪到 \|dev4_Q\| 与 0 之间 |
| 丙-直线（3 盘） | 同上，dev3_Q | 同上 | 丙行同一格；\|dev3_Q\| > 5% 而 \|dev4_Q\| ≤ 5% 时记「不稳定」 | 同上 |
| 丙-槽填满 | K = 64：max over N∈{2,3,4} \|r_Q满(N) − r_Q半(N)\|，r_Q(N) = Q(N) ÷ Q(1) | ≤ 0.15 判「不随槽填满程度变」，否则「变」 | 丙行「随记录槽填满程度变」：> 0.15 ⇒ 翻向丙二 | 门槛挪到那个最大差与 0 之间 |
| 丙-点名项（不对应丙行：第八节几何敏感性一格，照量） | 满槽：max over N∈{2,3,4} \|r_Q,K64(N) − r_Q,K1(N)\| | ≤ 0.15 判「不随点名项多少变」，否则「变」 | 不翻丙行；照报 | 同上 |
| 丙-判别力·绝对字节 | K = 64 半满：pc 红 ⟺ Q_pc(N) > 1.10 × Q(N)，N = 1–4（Q_pc = P(read_stage_cache_keeps_zeros_pc) − P(baseline)） | 每个 N 报红 / 不红 | 丙行够判条件「判别力自证在同一几何上红」：任一 N 不红 ⇒ 见 F6 | 1.10 挪到 Q_pc ÷ Q 两侧 |
| 丙-判别力·同次自比（不判，照报） | 同上：pc 红 ⟺ r_Q,pc(N) > 1.10 × r_Q(N)，N = 2–4 | 每个 N 报红 / 不红 | 与甲-判别力·同次自比同读：不红是这种上界写法的性质，不是测量失败 | — |
| 丙-倍数（照报） | 三个几何：r_Q(N)、N = 2–4；另报 P(read_stage_cache) 自己的 r(N) | 不判 | 丙一要登记的那个倍数从这里取 | — |

**第七节修订：钉绝对值的断言（丙那几格）**

出自被测条款本身的（不符走 F7、F8，不作废）：
- C3：C881 正文「默认环两块盘约 1.5 GiB」：K = 64 满槽 Q(2) 落在 1.5 GiB ±25% 即 [1 207 959 552, 2 013 265 920] 字节之内记「相符」。
- C4：C881 正文「随盘数涨」与结构文档注释「环长乘盘数」：K = 64 满槽 r_Q(4) ∈ [3.0, 4.4] 且 r_Q(2) ∈ [1.6, 2.2] 记「相符」（区间与 F4 同）。

独立算出、用命令核过的（`anchors_c881.py` 输出，第四节修订第 1 条；不符的那一格作废，V4 照办）：
- A10：read_stage_cache 每格进程内数缓存：项数恰好 = N × 每盘填满槽数（满槽 196 608、393 216、589 824、786 432；半满减半），值的字节和恰好 = 项数 × 4096（满槽 1 盘 805 306 368）；keeps_zeros_pc 在半满上项数恰好 = N × 196 608。装置在交回之后、放掉缓存之前数，打进结果行 `cache_entries=`、`cache_value_bytes=`。
- A2、A3 对两条新臂照钉：`read` 落到池上的次数 = N × 196 608（缓存第一遍全不命中），交回的表键数 = 每盘填满槽数。

推的、要执行员用命令核的（不符不作废，走 S5）：
- A11：read_stage_cache 的分配器峰值减起点 − 同格基线的 = 项数 × (4096 + 544 ÷ 6) 字节，±5%（每格中位数比）。装置开头另打 `size_of::<(DeviceIdentity, DeviceOffsetInBytes, usize)>()` 核 24。
- A12：read_stage_cache 的期末值（交回之后、放掉表与缓存之前的在用字节减起点）− 基线的期末值 = 项数 × (4096 + 544 ÷ 6)，±5%。

**第八节修订：轨迹与几何敏感性（丙那几格）**

轨迹三样照原第八节：每块盘最后一槽被读到时的在用字节（read_stage_cache 应每盘多一个约 0.77 GiB 的台阶，推的）、峰值、期末值（含缓存：缓存活到读阶段判完）。
几何敏感性：丙行依赖的旋钮是盘数、槽填满档、点名项数。推的 Q 与 K 无关，K 只动基线那一截：K = 1 满槽 Q 占 P_cache 的 76%（1 盘）到 84%（4 盘）、K = 64 满槽占 44%（1 盘）到 47%（4 盘）（第四节修订第 1 条算出），Q 是两格之差，这两点上噪声占比方向相反，就拿 K = 1 当丙行的反向取样点；3 盘对 Q 不反向（推的 dev3 = 0），照跑照报。每一行在 K = 64 与 K = 1 上各报判定，翻面记「不稳定」。

**第九节修订：变异（丙那几格）**

| 变异 | 改哪 | 敏感的取样点与应红的断言 | 不敏感的取样点 |
|---|---|---|---|
| M8 | 抄本 `read` 不往表里插（缓存恒空） | 每格：A10 项数 0 ≠ N × 填满槽数；A11 出界 | 无 |
| M9 | keeps_zeros_pc 的抄本照原样只收非 0 | K = 64 半满：Q_pc = Q，丙-判别力·绝对字节由红转不红，F6 触发 | 满槽各几何（本来相同） |
| M10 | 抄本的键去掉盘号（只按偏移与长度） | N ≥ 2：A10 项数 = 填满槽数 ≠ N × 填满槽数，r_Q(4) 由推的 4 变 1 | 1 盘 |
| M11 | 交回之后、读期末值之前先放掉缓存 | 每格 A12 期末值出界 | 峰值（A11 不动） |
| M12 | 抄本里改一处字：`mount.rs:8487` 抄来的那一行 `.insert(placement, bytes.clone())` 换成 `.insert(placement, bytes[..64].to_vec())` | 每格：S4 单测（抄本与仓里那一段逐字比）由绿转红；A10 值的字节和 = 项数 × 64 ≠ 项数 × 4096 | 无 |

**第十节修订：失败条款（丙那几格；原 F1–F5 不动）**

- **F6 测量分不出「缓存连全 0 也收」，丙各行作废**（阳性对照）。
  什么观测会让它触发：K = 64 半满、N ∈ {1, 2, 3, 4} 里任一个 Q_pc(N) ≤ 1.10 × Q(N)（推的 Q_pc ÷ Q = 2.000，第四节修订第 1 条）。
  不归它的：丙-判别力·同次自比不红是这种上界写法的性质，不触发 F6。
- **F7 C881 正文的推算数可能错**（不作废；交回时提议改 C881 正文那个数）。
  什么观测会让它触发：K = 64 满槽 Q(2) 落在 [1 207 959 552, 2 013 265 920] 字节之外（第七节修订 C3）。
- **F8 C881「随盘数涨」可能错**（不作废；照报）。
  什么观测会让它触发：K = 64 满槽 r_Q(4) 落在 [3.0, 4.4] 之外，或 r_Q(2) 落在 [1.6, 2.2] 之外（第七节修订 C4）。
- 结果反过来时怎么改结论（跑前写下）：丙-直线两行都在 5% 内、丙-槽填满 ≤ 0.15、丙-判别力·绝对字节在 K = 64 半满四个 N 上都红，就照丙行交「丙一与丙二等价、取丙一」；任一条不满足（不是「分不出」）交「只有丙二判得住」，即使与第四节修订推的方向相反。「分不出」的行不进这一句，交回时写明是哪几行、区间多少。

**第十一节修订：作废与停机（原 V2–V5、S1–S3 不动；V1 由下面 V6、V7 替换）**

- V1 改：服务重试代替内存包装的重试；一格的哪一次尝试算结果只看下面 V6、V7，结局不是 succeeded 的尝试一律不算（含 memory-cap-hit、squeezed-out、stalled）。撞到两台最大（memory-cap-at-maximum）的那一格报「丢了」，不当 0。比原来多要求的：原来只看包装的退出码，现在另要服务记录里那一次的结局与峰值。
- V6（顶着上限）：一次尝试 `memory_peak` ≥ `memory_cap` − 64 MiB，不算；这一格这一轮重交。依据：第四节修订第 2 条（撞上限那一次峰值记成 4.00 GiB = 上限）；没撞 250 而贴着上限的，页缓存被回收过，峰值被压住了。
- V7（钉不到服务记录）：结果行的 `task=`、`attempt=` 在存档的状态 JSON 里找不到，或那一次 `outcome` 不是 succeeded，或 `memory_peak` 为空（`test-discipline.md`「读不到 ≠ 读到 0」），那一次作废；同一片的同一次尝试被两行结果行认领，两行都作废。
- V8（同台扣不了空跑格）：一次尝试所在的台 h 上，几何 g 没有空跑格、而台 h 上各几何空跑格中位数之间相差 > 8 MiB（第六节修订第 5 条），那一次扣不了，这一格这一轮重交。另报一行「两台空跑格之差」：同一几何两台都有空跑格时，E(本机, g) − E(第二台, g)，不判。
- V9（二进制）：全部格结果行的 `binary_sha256=` 必须同一个值，且等于 checks 段记的那一个；不同的那几次作废。
- S1 补一句：今天两段都是「挪了行号、正文没变」（第三节修订），照跑并记一笔。
- S4（读缓存的抄本）：开跑之前 `ReadStageCache` 那一段与登记时相同：`awk 'NR>=8446 && NR<=8500' crates/singlefs-core/src/mount.rs | sha256sum` 应为 `58f00a3e70e681cf18cf672e29707b9bee2fe2655bb2ea996a4072f72a5d3f8c`；挪了行号的照 S1 的办法（找 `struct ReadStageCache<` 那一行，往前 4 行起取同样长的一段）比；正文变了就停，抄本按新实现重抄。装置那条逐字比的单测红了同样停。
- S5：A11、A12 出 ±5%；或两条新臂上 S3 的两条量法互不包含，停。
- 够判停机补：丙行要 K = 64 满槽与半满的 read_stage_cache、半满的 keeps_zeros_pc、同几何的 baseline 与空跑格，加 K = 1 满槽（反向取样点）；丙行认之前、或岔路单甲乙丙都够判之后，没跑的丙格逐条标「够判后未跑」或「丙行未认、未跑」。
- 原第十一节末「不在本实验里的观测」一段：丙行认了之后由本修订的丙那几格量，那一段留着当登记时的样子。

**跑法（替换原第十四节「线程」一行里经 `capped.sh` 编、经内存包装跑格的写法）**

交给服务的都是不带包装、不写上限与线程的原命令；下面是形态，具体参数由执行员在驱动里定，`drive.py list-submissions` 逐行打出整条，主 agent 照抄交：
- 编译一次（做法等确认项 6）；
- `task-server/task submit -- python3 scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py checks --binary <二进制> --out research/results/e197-<日期>-checks.out`（不分片：S1、S4、装置 `--checks`、差分集 D）；
- `task-server/task submit -- python3 …/drive.py compare --binary <二进制> --out research/results/e197-<日期>-compare.out`（不分片，不量峰值，52 格串行）；
- `task-server/task submit -- python3 …/drive.py segment <段> --round <1–5> --group <组> --binary <二进制> --out research/results/e197-<日期>-<段>-r<轮>-<组>.out`（分片，一组至多 8 格）。分组：k64-full 原有 21 格分 3 组、丙 4 格 1 组；k64-half 原有 20 格分 3 组、丙 8 格 1 组；k1-full 原有 20 格分 3 组、丙 4 格 1 组。每轮 12 件、5 轮 60 件，丙那 15 件等丙行认了才交。V2 第二轮（重复号 6–10）照同样的形态交 `--round 6`…`10`，只含 `report.py --unstable` 点名的格；
- 每件有了结局：`task-server/task status <父任务号> --json > research/results/e197-<日期>-status-<父任务号>.json`（主 agent 直接跑，只读服务）；
- 出表：`task-server/task submit -- python3 …/report.py --artifacts research/results/e197-<日期>-*.out --status research/results/e197-<日期>-status-*.json`（参数形态由执行员定，读全部组的产物与全部状态存档）。

**第十四节修订：并行与挂钟（修订一之后的跑法；原第十四节留着）**

独立单位照原文：每格一个进程、互不共享状态；现在每格是服务的一次尝试（一片）。
线程：每片单线程跑装置；服务给每次尝试套登记值 31（`.claude/heavy-thread-default`），装置只用一条。编译那一件 31。
串行：同一件的兄弟片一台同时只跑一片（`task-server/task_server_lib.py` 文件头「分片」），一件 8 片至少分 4 拨；件与件之间不串行，判的量是各自 scope 的峰值，不随并行变。
预计挂钟（推的）：量格 385 次尝试（原 305 + 丙 80）× 每次约 20 秒（服务起 scope 与拷二进制约 3–8 秒，扫环至多约 14 秒）= 7 700 秒串行；撞 4G 下限再放大的约 40 次另加约 800 秒；计划、合并 120 次 × 约 5 秒 = 600 秒；按两台合计约 20 片同时在跑（本机 slice 约 50G、每片记账至少 4G）约 8–15 分钟；compare 52 格 × 约 10 秒 ≈ 9 分钟；checks 约 2 分钟；编译 10–25 分钟。合计约 30–50 分钟。
预计在用核数：编译段 31 个；量格段约 20 个（约 20 片同时、各单线程）；出表 1 个。

**第十三节修订：修订一读过的文件与跑过的命令**

读过：`.claude/agent-common.md`（整份）；`.claude/singlefs-ai-sop/rules/test-discipline.md:1-138`、`.claude/singlefs-ai-sop/rules/evidence-discipline.md:1-182`、`.claude/rules/implementation-first.md:1-19`、`.claude/rules/mutation-sampling.md:1-93`、`.claude/rules/test-script-discipline.md:1-106`、`.claude/rules/three-way-inference.md:1-168`；本登记 `:1-341`；岔路单 `:1-8`；调查员报告 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-experiment-status/result.md:1-76`（只读了 grep 命中行）与 `:234-266`；`.claude/kb/checks-owed.md:397`、`:541`（grep 命中行，`.en.md`、`.ja.md` 同号行只看了 grep 截断的行首）；`crates/singlefs-core/src/mount.rs:8420-8506`、`:8640-8830`、`:8657-8682`，与 grep 命中的 `:398`、`:9122`；`crates/singlefs-core/src/address.rs:21`、`:29`（grep 命中）；`crates/singlefs-checker-tier/src/bin/e197_scan_journal_glue_memory_by_device_count.rs:1-45`、`:245-322`、`:520-600`、`:860-952` 与 grep 列的函数行；`scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py:1-573`（整份）；同目录 `report.py:1-40`、`:205-250` 与 grep 列的函数行；`task-server/task_server_lib.py:130-300`（资源记录、分片一段）与 grep 命中的 `:900-907`；`task-server/task:1-60` 与 grep 命中行；`task-server/task_server.py:1815-1910`、`:3390-3398`、`:3480-3488` 与 grep 命中行；`.claude/gate.d/doc-experiments.sh:2601-2700`；`.claude/heavy-thread-default`；`.claude/hooks/agent-write-scope.tsv:1-20`；`scripts/kb/replace-once.py:1-40`。没读：`.claude/kb/experiments/`、`research/results/`、别的实验的登记、`records/`。
跑过的命令（只读与草稿目录里的写；原样）：`bash scripts/hosts/work-directory-ownership.sh claim /home/fy5090/.local/share/singlefs/work/m3-close-b19-e197-design --purpose 'E197 登记修订：补 C881 读阶段读缓存那一臂，跑法去掉里层包装、峰值读服务记的'`；`python3 scripts/kb/quote-kb.py /home/fy5090/.local/share/singlefs/work/m3-close-b19-e197-design/quoted-clauses.md '.claude/kb/checks-owed.md~^\| C881 \|' '.claude/kb/checks-owed.md~^\| C653 \|'`；`python3 -I /home/fy5090/.local/share/singlefs/work/m3-close-b19-e197-design/anchors_c881.py`；`awk 'NR>=8446 && NR<=8500' crates/singlefs-core/src/mount.rs | sha256sum`；`awk 'NR>=4932 && NR<=4985' crates/singlefs-core/src/recovery.rs | sha256sum`；`awk 'NR>=4034 && NR<=4063' crates/singlefs-replay-prefix/src/lib.rs | sha256sum`；`sha256sum crates/singlefs-core/src/mount.rs crates/singlefs-core/src/recovery.rs crates/singlefs-replay-prefix/src/lib.rs`；`task-server/task status 4705`；`task-server/task log 4705`；`ls research/results/ | grep -ci e197`；`grep -c 'e197\|E197' scripts/experiment-apparatus/replay.sh`；`TZ=Asia/Tokyo date +%F`。草稿目录里：`quoted-clauses.md`（sha256 `35ab7eb631e3570842623306cda08a0b91c31ac0a8632b2371def80e71091627`）、`anchors_c881.py`（`3d60390d67f2535555b7e409a6553f70aa16304ae895fff020cb72c2d7768c57`）、`anchors_c881-output.txt`（`7638688c95a4605d832a34de13dad853a902924dba11eafc21f4743b37e79263`）。

### 修订二（2026-11-02，执行员照修订一改完驱动、出表脚本与装置之后、产物之前）：撞上限的尝试一律不算；编译与交法的执行细节

谁写、什么时点：执行员（批次 `records/2026-10-18-里程碑三收尾与近期欠账.md#19`）。时点在产物之前，量的：`ls research/results | grep -ci e197` 得 0，`grep -c 'e197\|E197' scripts/experiment-apparatus/replay.sh` 得 0。这次修订不靠单测读数（改过的装置还没编、单测还没跑）；依据是主 agent 派发提示转达的管任务服务的会话现查的七问，下面逐条写是哪一问。原文与修订一一个字不改；判据、门槛、失败条款一个不动。

1. **V10（撞上限）**，只收严。依据：派发提示第 2 条 ③「服务没有现成的「贴着上限」标记。memory.peak 含页缓存，更稳的读法是看撞没撞上限的结局（memory-cap-hit，退 250），不靠峰值离上限多远去猜。落法：登记 V6「≥ 上限 − 64 MiB 不算」照留；另加一条「结局是 memory-cap-hit 的尝试一律不算」」。写死：结局是 memory-cap-hit 的尝试一律不算，那一格这一轮由服务放大上限重试的那一次接上（同一片的下一次尝试，结果行的 `attempt=` 是那一次）。V6 照留，两条各判各的。出表脚本 `report.py` 的 `Artifact.pinned_reading` 判：结局不是 succeeded 的一律作废（作废原因写 `v7_outcome_<结局>`，memory-cap-hit 落在这里），再判 V6。
2. **编译一次放哪**（补修订一「未定项」第 3 条与「跑法」一段「编译一次（做法等确认项 6）」，不改判据）。依据：派发提示第 2 条 ⑥「编完把二进制拷进 TASK_SERVER_SHARD_PLAN_DIRECTORY（共享根，两台同一路径读得到），各片直接起那一份……二进制一定要拷出来，不能指回编译目录」。落法：只编一次，就是入库装置换进主工作区那一步 `compile-then-swap.py … --release` 编出的 `<草稿目录>/compile-then-swap/target/release/e197_scan_journal_glue_memory_by_device_count`（在共享工作根下，留到这一段产物跑完才 `--clean`）；每件分片任务的计划一步把它拷进自己的计划目录，各片再拷进自己的产物目录、从那份拷贝起（修订一第六节第 4 条）。计划一步不另编：分片的子任务服务不给编译目录（`task-server/task_server.py` 建子任务时写 `"compiles": False`，2026-11-02 现查第 2069 行），在计划里起 `cargo build` 会编进主工作区的 `target/`，60 件各编一次、两台各编也守不住 V9。V9 照修订一：全部格的 `binary_sha256=` 等于 checks 段记的那一个。
3. **交法**（不改判据）。依据：派发提示第 2 条 ⑦「约 60 件分片任务别一次全交，按轮交：一轮 5 到 10 件，前一轮大半有了结局再交下一轮」与 ⑤「任务里别起 task-server/task status」。落法：`drive.py list-submissions --binary <二进制> --date <日期> --batch <n>` 打出第 n 批：第 0 批 checks 与 compare，第 1–10 批各 6 件 segment（段依次 k64-full、k64-half、k1-full，每段重复号 1–5，每个重复号 glue-1、glue-2、glue-3、read-stage-cache 四组）。组名替修订一「跑法」一段的「<组>」：glue-1..3 是原有的格按次序每 8 格一组，read-stage-cache 是丙那几格。V2 第二轮 `drive.py list-reruns` 打出（组名 rerun-<n>、`--cells` 点名的格）。
4. **空跑格自己的 V2 散布**（修订一第六节第 5 条的读法写明，同一台时与原文相同）：空跑格的 d_i 同样是 memory_peak_i − E(台_i, 几何)，散布照修订一的式子算；V3（空跑格 > 128 MiB）看这一格 memory_peak 原值的中位数。两台底子不同时，空跑格不因两台之差判不稳定。
5. **同一格两行**（只收严）：同一个（臂, 盘数, 填满档, 点名项数, 重复号）在交给出表脚本的产物里出现两行，两行都作废（作废原因 `duplicate_cell`），不挑一行用；要重跑就换新文件名、只交一份。
6. **状态存档的字段待第一件核**（不改判据）：派发提示第 2 条 ①「这一点是推的，没逐字段打出来核：跑一件看一眼 JSON 再定」。出表脚本按 `children[].attempts[]` 的 `task_id`、`number`、`host`、`memory_peak`、`memory_cap`、`outcome` 读；第一件存档之后由承接的执行员逐字段核，对不上的停下交回，不改这几条判据。


### 修订三（2026-11-04，装置去掉计数分配器之前、判的产物之前）：去掉计数分配器；进程内那一条量法改成常驻页峰值 H

谁写、什么时点：主 agent 派设计员写（批次 `records/2026-10-18-里程碑三收尾与近期欠账.md#19`）。依据是用户 2026-11-03 弹窗定的「丙：E197 改不用、E183 登」（第三轮判决 `research/prompts/m3-close-b19-reason-r3-main-verification.md` 第三节「两份装置的计数分配器 unsafe」那一行的丙）：装置里删掉计数分配器（`unsafe impl GlobalAlloc`）。判决第二节 Q5 格：「**E197 判的数离得开（三份一致）**：判的是服务记的 cgroup 峰值，分配器只做互核与停机条款」。这次修订不靠任何单测读数，也不靠任何产物读数。
时点，量的：`ls research/results/ | grep -ci e197` 得 14（文件名带 e197 的 14 份，没读内容；照调度记录 `records/2026-10-18-里程碑三收尾与近期欠账.md` 第 2307–2322 行，是第 0 批 checks、compare 与第 1 批的状态存档，第 1 批的峰值读数主 agent 已判「不能判」、要整批重跑），`grep -c 'e197\|E197' scripts/experiment-apparatus/replay.sh` 得 0。判的量 P 没有一格进过判决。
原文、修订一、修订二一个字不改；下面每一处写明它替换的是哪一节哪一行的哪一句，被替换的原句留着、只是不再执行。甲、乙、丙各行判的量 P（cgroup 峰值减同台同几何空跑格）、门槛、失败条款 F1–F4、F6–F8 一个不动。

**写死：装置里没有计数分配器，也没有 `unsafe`。**装置源文件里不出现 `#[global_allocator]`、`GlobalAlloc`、`unsafe`；进程内的内存读数只经 `std::fs` 读 `/proc/self/status`、写 `/proc/self/clear_refs`。

**进程内那一条量法 H（替换「分配器峰值减起点」）**，口径照仓里已有的那一份（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:2281-2294`：往 `/proc/self/clear_refs` 写 `5` 把这个进程的 `VmHWM` 重置成此刻的常驻量，记下此刻的 `VmRSS` 当起点；`mutation-sampling.md` 第五类，不另起口径）：
1. 量窗开头（建池、编探针记录之后，调臂之前）往 `/proc/self/clear_refs` 写 `5`，随即读 `/proc/self/status` 的 `VmRSS` 得 rss_start（KiB）。
2. 交回之后、放掉交回值之前（经读缓存的两臂在放掉缓存之前）读 `VmHWM` 得 hwm_end（KiB）。
3. H = (hwm_end − rss_start) × 1024 字节。一格的 H 取 5 次的中位数。
4. 写 `5` 失败、或任一项读不出，结果行那几个字段写 `unreadable`，那一次作废（V11），不当 0。
5. 轨迹与期末值改读 `VmRSS`：每块盘最后一槽被读到那一刻读一次 `VmRSS` 减 rss_start；期末值 = 交回之后、放掉交回值之前（经读缓存的两臂在放掉缓存之前）的 `VmRSS` 减 rss_start。单位都换成字节。
H 与 P 量的是同一种东西：碰过、还在的页（`VmHWM` 是进程常驻页的高水位，`memory.peak` 是 scope 记账页的高水位）。分配器量的是 `Layout` 字节（含没碰过的 Vec 容量）。两种口径在大 Vec 上不同：Vec 过了 glibc 的 mmap 门槛之后按 `mremap` 长，没写过的容量页不常驻（推的，读的是机制，没量）。所以第七节 A7 的式子跟着换成常驻页口径（下面第七节修订），不沿用 `predict.py` 的 `cap(N·s)`。

**结果行字段（替换第六节「量法」一段里结果行的写法与「分配器峰值按 E183 那份口径……VmHWM 在交回之后、放掉交回值之前读 `/proc/self/status`」一句）**：`allocator_peak_over_start_bytes`、`allocator_live_at_each_device_end_bytes`、`allocator_live_after_return_bytes` 三个字段删掉，换成 `resident_peak_over_start_bytes=`（H）、`resident_at_each_device_end_bytes=`（轨迹，N 个数）、`resident_after_return_bytes=`（期末值）、`vm_rss_at_start_kib=`、`vm_hwm_kib=`（hwm_end）、`high_water_mark_reset=`（`ok` 或 `unreadable`）；别的字段照旧。

**第一节修订**：第 21 行末句「进程内的分配器峰值与 VmHWM 两个数只用来互核，不判」读成「进程内的常驻峰值 H 只用来互核，不判」。判的那一个数不变。

**第三节修订**：第 93 行（仓里另一份进程内堆计数的口径）不再适用：本实验没有分配器那条路。仓里量常驻峰值的另两处，只读了写法、没读结果：`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:1518-1524`（`VmHWM`，不重置，读不到报 0）与 `crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs:2711-2716`、`:3469-3481`（`VmHWM`，进程起到此刻，不重置）。本实验取 `gpu_unit_checks.rs` 那一份（重置、减起点），读不到作废、不报 0。

**第五节修订**：
- 第 132 行「并在每块盘最后一槽被读到时记一次分配器此刻的在用字节（第八节轨迹）」读成「并在每块盘最后一槽被读到时记一次此刻的 `VmRSS`（第八节轨迹）」。
- 第 152 行 measurement_pc 的「做完会怎样」：「分配器峰值多 268 435 456 字节」读成「H 多 256 MiB（逐页写过的 65 536 页）」；「cgroup 峰值多 256 MiB 左右」照旧。
- 单测：H 是整个进程的数，libtest 并行跑的别的用例会串进来。凡要读 H、`VmRSS` 的单测，不在与别的用例共用的进程里读：起一个只跑这一格的子进程（装置二进制本身）读它的结果行，或照 `gpu_unit_checks.rs` 的租借那样保证读数时进程里只有这一条在跑；做不到的改成不读内存的结构断言。选哪一种由执行员定，写进登记落地清单。原来点名分配器单测的变异行（第九节修订）跟着换用例。

**第四节修订：跑之前已经存在的数（修订三新增）**

1. 设计员推的（没量）：`python3 -I /home/fy5090/.local/share/singlefs/work/m3-close-b19-e197-reregister/model/resident_prediction.py`（脚本 sha256 `66a1a2e61970daa126a4f3e97274e40f4721982ba61377aacebd55178ba23bb5`），常驻页口径与分配器口径并列，记录 320 字节、点名项 64 字节（装置单测 `structure_sizes_match_the_field_by_field_layout_of_the_current_structs` 钉的值），原样输出：

```
resident  baseline         K=64 s=196608 N1=0.937 N2=1.751 N3=2.566 N4=3.380 r4=3.608 dev3=+0.0000 dev4=+0.0000 slope=0.869
allocator baseline         K=64 s=196608 N1=0.953 N2=1.784 N3=2.693 N4=3.446 r4=3.616 dev3=+0.0290 dev4=+0.0000 slope=0.872
resident  baseline_pc      K=64 s=196608 N1=1.690 N2=2.504 N3=3.319 N4=4.133 r4=2.446 dev3=+0.0000 dev4=+0.0000 slope=0.482
allocator baseline_pc      K=64 s=196608 N1=1.703 N2=2.534 N3=3.443 N4=4.196 r4=2.464 dev3=+0.0227 dev4=+0.0000 slope=0.488
resident  split            K=64 s=196608 N1=0.937 N2=1.693 N3=1.696 N4=1.699 r4=1.813 dev3=-0.4440 dev4=-0.8865 slope=0.003
allocator split            K=64 s=196608 N1=0.953 N2=1.706 N3=1.709 N4=1.712 r4=1.796 dev3=-0.4389 dev4=-0.8763 slope=0.003
resident  split_pc         K=64 s=196608 N1=0.937 N2=1.751 N3=2.566 N4=3.380 r4=3.608 dev3=+0.0000 dev4=+0.0000 slope=0.869
allocator split_pc         K=64 s=196608 N1=0.953 N2=1.784 N3=2.615 N4=3.524 r4=3.698 dev3=+0.0000 dev4=+0.0222 slope=0.913
resident  read_stage_cache K=64 s=196608 N1=1.707 N2=3.292 N3=4.876 N4=6.460 r4=3.785 dev3=+0.0000 dev4=+0.0000 slope=0.928
allocator read_stage_cache K=64 s=196608 N1=1.719 N2=3.317 N3=4.993 N4=6.512 r4=3.787 dev3=+0.0156 dev4=+0.0000 slope=0.929
resident  baseline         K=64 s= 98304 N1=0.469 N2=0.876 N3=1.283 N4=1.690 r4=3.608 dev3=+0.0000 dev4=+0.0000 slope=0.869
allocator baseline         K=64 s= 98304 N1=0.476 N2=0.892 N3=1.347 N4=1.723 r4=3.616 dev3=+0.0290 dev4=+0.0000 slope=0.872
resident  baseline_pc      K=64 s= 98304 N1=0.845 N2=1.252 N3=1.659 N4=2.067 r4=2.446 dev3=+0.0000 dev4=+0.0000 slope=0.482
allocator baseline_pc      K=64 s= 98304 N1=0.851 N2=1.267 N3=1.722 N4=2.098 r4=2.464 dev3=+0.0227 dev4=+0.0000 slope=0.488
resident  split            K=64 s= 98304 N1=0.469 N2=0.846 N3=0.848 N4=0.849 r4=1.813 dev3=-0.4440 dev4=-0.8865 slope=0.003
allocator split            K=64 s= 98304 N1=0.476 N2=0.853 N3=0.854 N4=0.856 r4=1.796 dev3=-0.4389 dev4=-0.8763 slope=0.003
resident  split_pc         K=64 s= 98304 N1=0.469 N2=0.876 N3=1.283 N4=1.690 r4=3.608 dev3=+0.0000 dev4=+0.0000 slope=0.869
allocator split_pc         K=64 s= 98304 N1=0.476 N2=0.892 N3=1.307 N4=1.762 r4=3.698 dev3=+0.0000 dev4=+0.0222 slope=0.913
resident  read_stage_cache K=64 s= 98304 N1=0.854 N2=1.646 N3=2.438 N4=3.230 r4=3.785 dev3=+0.0000 dev4=+0.0000 slope=0.928
allocator read_stage_cache K=64 s= 98304 N1=0.860 N2=1.659 N3=2.496 N4=3.256 r4=3.787 dev3=+0.0156 dev4=+0.0000 slope=0.929
resident  baseline         K= 1 s=196608 N1=0.199 N2=0.275 N3=0.351 N4=0.427 r4=2.150 dev3=+0.0000 dev4=+0.0000 slope=0.383
allocator baseline         K= 1 s=196608 N1=0.215 N2=0.307 N3=0.478 N4=0.493 r4=2.297 dev3=+0.1633 dev4=+0.0000 slope=0.432
resident  baseline_pc      K= 1 s=196608 N1=0.213 N2=0.290 N3=0.366 N4=0.442 r4=2.071 dev3=+0.0000 dev4=+0.0000 slope=0.357
allocator baseline_pc      K= 1 s=196608 N1=0.226 N2=0.319 N3=0.490 N4=0.505 r4=2.230 dev3=+0.1594 dev4=+0.0000 slope=0.410
resident  split            K= 1 s=196608 N1=0.199 N2=0.216 N3=0.219 N4=0.222 r4=1.118 dev3=-0.0668 dev4=-0.1319 slope=0.015
allocator split            K= 1 s=196608 N1=0.215 N2=0.229 N3=0.232 N4=0.235 r4=1.096 dev3=-0.0505 dev4=-0.0997 slope=0.014
resident  split_pc         K= 1 s=196608 N1=0.199 N2=0.275 N3=0.351 N4=0.427 r4=2.150 dev3=+0.0000 dev4=+0.0000 slope=0.383
allocator split_pc         K= 1 s=196608 N1=0.215 N2=0.307 N3=0.400 N4=0.571 r4=2.661 dev3=+0.0000 dev4=+0.1368 slope=0.614
resident  read_stage_cache K= 1 s=196608 N1=0.969 N2=1.815 N3=2.661 N4=3.507 r4=3.620 dev3=+0.0000 dev4=+0.0000 slope=0.873
allocator read_stage_cache K= 1 s=196608 N1=0.981 N2=1.841 N3=2.778 N4=3.559 r4=3.628 dev3=+0.0281 dev4=+0.0000 slope=0.876
gap baseline K=64 s=196608 N=1 allocator_minus_resident_MiB=16.2 relative_to_resident=+0.0169
gap baseline K=64 s=196608 N=2 allocator_minus_resident_MiB=33.2 relative_to_resident=+0.0185
gap baseline K=64 s=196608 N=3 allocator_minus_resident_MiB=130.2 relative_to_resident=+0.0496
gap baseline K=64 s=196608 N=4 allocator_minus_resident_MiB=67.2 relative_to_resident=+0.0194
gap baseline K=64 s=98304 N=1 allocator_minus_resident_MiB=8.1 relative_to_resident=+0.0169
gap baseline K=64 s=98304 N=2 allocator_minus_resident_MiB=16.6 relative_to_resident=+0.0185
gap baseline K=64 s=98304 N=3 allocator_minus_resident_MiB=65.1 relative_to_resident=+0.0496
gap baseline K=64 s=98304 N=4 allocator_minus_resident_MiB=33.6 relative_to_resident=+0.0194
gap baseline K=1 s=196608 N=1 allocator_minus_resident_MiB=16.2 relative_to_resident=+0.0799
gap baseline K=1 s=196608 N=2 allocator_minus_resident_MiB=33.2 relative_to_resident=+0.1181
gap baseline K=1 s=196608 N=3 allocator_minus_resident_MiB=130.2 relative_to_resident=+0.3623
gap baseline K=1 s=196608 N=4 allocator_minus_resident_MiB=67.2 relative_to_resident=+0.1537
A9 split_vs_baseline_N1 K=64 resident_equal=True
A9 split_vs_baseline_N1 K=1 resident_equal=True
trajectory_last_device_end K=64 s=196608 N=1 expected_MiB=832.5
trajectory_last_device_end K=64 s=196608 N=2 expected_MiB=1663.5
trajectory_last_device_end K=64 s=196608 N=3 expected_MiB=2494.5
trajectory_last_device_end K=64 s=196608 N=4 expected_MiB=3325.5
trajectory_last_device_end K=64 s=98304 N=1 expected_MiB=417.0
trajectory_last_device_end K=64 s=98304 N=2 expected_MiB=832.5
trajectory_last_device_end K=64 s=98304 N=3 expected_MiB=1248.0
trajectory_last_device_end K=64 s=98304 N=4 expected_MiB=1663.5
trajectory_last_device_end K=1 s=196608 N=1 expected_MiB=76.5
trajectory_last_device_end K=1 s=196608 N=2 expected_MiB=151.5
trajectory_last_device_end K=1 s=196608 N=3 expected_MiB=226.5
trajectory_last_device_end K=1 s=196608 N=4 expected_MiB=301.5
one_percent_of_H_base_N1_MiB {'K64_s196608': 9.6, 'K64_s98304': 4.8, 'K1_s196608': 2.04}
chunks 4112 80 3728 288 4112 560
```

   式子（推的）：glibc 块 = max(32, (请求 + 8 + 15) 按 16 向下取整)；过 mmap 门槛的大 Vec（记录 Vec、键表）只算 len × 元素大小；每条记录的点名项堆算一块 chunk(K × 64)；两张 B 树照装置 A7 同一个口径按每叶 6 个键摊（叶 = 16 + 11 × 键 + 11 × 值）。对判据的影响，照实写：
   - A7 要换式子：拿分配器式子比 H，K = 1 各盘数差 +8.0%、+11.8%、+36.2%、+15.4%，K = 64 的 3 盘差 +4.96%、贴着门槛（上面 `gap` 几行）；K = 1 四格都出 ±5%。所以 A7 改用常驻页式子（第七节修订），容差 ±5% 不动。
   - 原 S3 在常驻页口径下推得会停：P 与 H 同口径，「分配器峰值减起点 > P + 32 MiB」在 K = 64 的 2 盘（差 33.2 MiB）、3 盘、4 盘与 K = 1 的 2–4 盘上推得成立。这次换掉原 S3 的依据是用户定的去掉分配器，不是这一条；照实列。
   - 甲-直线（第六节）与第八节「3 盘」这个反向取样点：常驻页口径下基线随盘数是一条直线加截距（dev3 = dev4 = 0，三个几何都是）；第八节说的「Vec 容量放大 16/9 对 4/3」只在分配器口径里有，P 一直是 cgroup 口径。所以 3 盘对判的 P 推得不反向。这不是本修订带来的，本修订不改第八节的取样点，列进下面「未定项（修订三）」。
   - A9：常驻页口径下 split 与 baseline 在 1 盘上推得逐字节相同（`A9 … resident_equal=True`）。
   - 分辨率：H 的粒度 4 KiB（一页）。A9 的门槛 1% 换成字节，按 H_base(1) 推是 9.6 MiB（K = 64 满槽）、4.8 MiB（K = 64 半满）、2.0 MiB（K = 1 满槽）。`VmRSS` 与 `VmHWM` 读的是内核每 CPU 分开攒的计数，每个跑过的 CPU 至多攒一批页没并进总数（推的：本机 32 个 CPU 时一批约 64 页，即 256 KiB；没量），比 2.0 MiB 小。第二台的核数与内核没现查，列进「未定项（修订三）」。
2. 读调度记录时看到的数（量的，出处是记录，不是产物；没读 `research/results/` 下任何文件）：`records/2026-10-18-里程碑三收尾与近期欠账.md` 第 2307 行「E197 compare 4928 已尝试 3 次（前两次 memory-cap-hit：4096M 时峰值 3.96 GiB、6144M 时峰值 6.00 GiB），第 3 次在第二台 9216M 跑」，第 2308 行「compare 4928 第 3 次（第二台、9216M、峰值 6.54 GiB）退 0」，第 2322 行「memory_peak 9 条 null、第二台 11 条都不到装置自报 VmHWM 一半，读数不能判」，第 2319 行服务「命令在第一次采样前跑完就记 null」、改成「命令退出之后、scope 收掉之前一定读一次」。对判据的影响：第 2322 行那一句是装置当时没重置的 `VmHWM` 与服务记的 `memory_peak` 之比，与本修订 S3 的两条量法有关；S3 的两个门槛（32 MiB、1.15 × H + 64 MiB）照抄原 S3，写在读到这几句之后、没按它们改。
3. 本修订读过的、带数的别的文件：`.claude/kb/experiments/197-扫环胶水内存随盘数.md` 第 9–15 行「现状」（`size_of` 是 320、split_pc 推算差 2.4% 与 −3%、−16%，装置的推算，不是产物）、第 33–39 行「口径」、第 54–64 行「路径与结论登记」；没读那一页的「历史版本」与别的小节。

**第六节修订**：
- 第 171 行「进程内另打一行结果行（……）」的字段照上面「结果行字段」；同一行「分配器峰值按 E183 那份口径（Layout 字节，`realloc` 记新旧之差，进入扫环之前把峰值重置到此刻在用，交回之后读峰值减起点）；VmHWM 在交回之后、放掉交回值之前读 `/proc/self/status`」读成上面「进程内那一条量法 H」第 1–5 条。
- 第 172 行「分配器峰值与 VmHWM 只用来互核（第七节 A7、第十一节停机条款），不判」读成「H 只用来互核（第七节 A7–A9、修订一 A11、A12，第十一节 S3、修订一 S5），不判」。P 的算法照修订一第六节第 5 条，不动。
- 第 173 行「分辨率」一段补一句：H 粒度 4 KiB，内核计数的攒批误差推的每个跑过的 CPU 至多 256 KiB（第四节修订第 1 条）；第七节 A7–A9 各门槛换成字节都在 2 MiB 以上。

**第七节修订：钉绝对值的断言（替换 A7、A8、A9 与修订一 A11、A12 里分配器那一半；C1、C2、C3、C4、A1–A6、A10 不动）**

记号：s = 每盘填满槽数，K = 每条记录点名项数，S_rec、S_named = 这一行结果行里装置打出的 `size_journal_record`、`size_named_unit`；chunk(b) = max(32, (b + 8 + 15) 按 16 向下取整)；U = chunk(K × S_named)；T = s × (chunk(16 + 11 × 16 + 11 × S_rec) + chunk(16 + 11 × 16 + 11 × 8)) ÷ 6。

- **A7（替换原 A7，推的、要执行员用命令核的，不符走 S3 停机，不作废）**：四个胶水臂每格的 H（5 次中位数）与常驻页推算 R 之比落在 ±5% 之内。R：baseline 与 split_pc = N·s·(S_rec + U + 16) + T；baseline_pc = 上式 + s·U；split 在 1 盘同 baseline、2 盘起 = T + 2·s·U + s·S_rec + N·s·16（取最后一盘那一刻：首份的点名项堆、这一盘的点名项堆与 Vec、到此为止的键表、两张树；上一盘放掉的点名项堆被这一盘接着用）。推的数见第四节修订第 1 条 `resident` 各行（K = 64 满槽 baseline 1–4 盘 0.937、1.751、2.566、3.380 GiB）。原 A7 写的「B 树偏 30% 只动总数约 4%」照旧适用于 K = 64；K = 1 上两张树约占 R(1) 的 62%，偏 30% 动 R 约 18%，那几格出界先查树那一截。
- **A8（替换原 A8 分配器那一半；cgroup 那一半「P 落在 [256 MiB, 288 MiB]」照旧）**：measurement_pc 每次的 H 落在 [266 338 304, 272 629 760] 字节（254 MiB 到 260 MiB）。往下留 2 MiB 是内核计数的攒批（推的），往上留 4 MiB 是页粒度、读 `/proc/self/status` 那一下新碰的堆页与第二台若开了透明大页多出的一页 2 MiB（本机 `/sys/kernel/mm/transparent_hugepage/enabled` 读出 `always [madvise] never`，第二台没查）。
- **A9（替换原 A9 分配器那一半；「cgroup 的 P 之差 ≤ 3%」照旧）**：每个几何上 split 与 baseline 1 盘那两格的 H（5 次中位数）之差 ≤ baseline 那一格 H 的 1%；出界走停机。
- **修订一 A11（替换「分配器峰值减起点」）**：read_stage_cache 与 keeps_zeros_pc 每格的 H − 同格 baseline 的 H = 项数 × (chunk(4096) + chunk(16 + 11 × 24 + 11 × 24) ÷ 6) = 项数 × (4112 + 560 ÷ 6) 字节，±5%（5 次中位数相减）。原式每项 4096 + 544 ÷ 6 是 `Layout` 字节，常驻页口径加 glibc 块头。不符走 S5。
- **修订一 A12（替换「期末值（交回之后、放掉表与缓存之前的在用字节减起点）」）**：两臂的期末值（`VmRSS` 减 rss_start）− 同格 baseline 的期末值 = 项数 × (4112 + 560 ÷ 6) 字节，±5%，不符走 S5。推的依据：两臂扫完之后放掉的是同一批东西（非首份的点名项堆、Vec、键表）；记录树的叶在堆顶活着，放掉的点名项堆在它下面，glibc 两臂都留着不还，差的只有缓存（没量）。

**第八节修订：轨迹（替换第 229、231 行与修订一第八节修订的「在用字节」）**

- 第 229 行「每块盘最后一槽被读到那一刻分配器的在用字节」读成「那一刻的 `VmRSS` 减 rss_start」；第 231 行「期末值：交回之后、放掉交回值之前分配器的在用字节减起点（交回的那张表的大小）」读成「期末值：同一刻的 `VmRSS` 减 rss_start（不再是交回那张表的大小：glibc 留着放掉的堆页，推的）」。第 230 行三样峰值读成「H、cgroup 峰值」两样。修订一第八节修订的「每块盘最后一槽被读到时的在用字节」同样读成 `VmRSS`；「每盘多一个约 0.77 GiB 的台阶」照旧（196 608 × 4205.3 字节，推的）。
- 第 232 行「与第七节 A7 的式子取的那一刻对不对得上（对不上走停机条款 S3）」写死成两条（出表脚本判，不符走 S3）：baseline 与 baseline_pc 最后一盘那一个轨迹值，与「读到那一刻的常驻页」= read × (S_rec + U) + 环槽数 × 8 之比落在 ±3% 之内，read = N·s − (满槽 1、半满 0)（最后一槽读到时这一槽还没解）；H ≥ 这一格轨迹里最大的那个数。推的数：K = 64 满槽 1–4 盘 832.5、1663.5、2494.5、3325.5 MiB，K = 1 满槽 76.5–301.5 MiB（第四节修订第 1 条 `trajectory_last_device_end` 各行）。

**第九节修订：变异**

| 变异 | 改哪 | 敏感的取样点与应红的断言 | 不敏感的取样点 |
|---|---|---|---|
| M5（改法不变，抓它的断言换成 H） | 池的生成不看 K（总写 64 项） | K = 1 各格：A7 的 H 推的 0.199 GiB 变 0.937 GiB（1 盘），出 ±5% | K = 64 各格 |
| M6（删） | 原改「计数分配器的 `realloc` 只加新大小、不减旧大小」 | 被改的代码随计数分配器删掉，这一条没有可改的地方 | — |
| M14（补 M6 的位置） | H 不乘 1024（KiB 当字节记） | measurement_pc：A8 由在窗里变成约 256 KiB，F5 触发；每个扫环格 A7 出界 | 无 |
| M15（补 M6 的位置） | 交回之后读的是 `VmRSS` 而不是 `VmHWM`（峰值读成期末的常驻量） | baseline、baseline_pc 的 K = 1 满槽 1–4 盘：记录 Vec 与键表已还给系统，H 推的少 N·s·336 字节（4 盘约 252 MiB，R 的 58%），A7 出界；K = 64 满槽 4 盘少约 7.3%，出界 | measurement_pc（读数在放掉缓冲之前，两者相同）；split 的 2 盘起（还给系统的只有最后一盘的 Vec 与键表，推的少约 4.1%，在 ±5% 内） |
| 修订一 M8（改法不变） | 抄本 `read` 不往表里插 | 每格：A10 项数 0；A11 的 H 差 0，出界 | 无 |
| 修订一 M11（改法换掉） | 原改「交回之后、读期末值之前先放掉缓存」：常驻页口径下放掉的缓存块夹在点名项堆中间，glibc 不还，`VmRSS` 推得不动，A12 抓不到（`mutation-sampling.md` 第三类，取样点不敏感，不留档）。换成：数缓存项数与读期末值之前先清空缓存 | 每个经读缓存的格：A10 项数 0 ≠ 登记项数 | A11（峰值在清空之前） |

装置里点名计数分配器的单测删掉之后，`crates/mutations.tsv` 里点名那几条单测的变异行，按第五节修订最后一条换成读 H 的用例；`crates/singlefs-core/src/journal.rs` 那一行（点名项 Vec 改回空 Vec 逐项 push）原来由分配器那条单测抓，常驻页口径下 K = 1 每条记录的块从 80 字节变 272 字节（推的），换成读 H 的用例后照抓。

**第十节修订**：F5「什么观测会让它触发」读成：measurement_pc 的 H 出 [266 338 304, 272 629 760] 字节（第七节修订 A8），或它的 cgroup 峰值减同几何空跑格出 [256 MiB, 288 MiB]。F5 的结论（测量路径坏，整轮作废）不动。

**第十一节修订：作废与停机**

- V4 原文照旧，「A8」读成第七节修订的 A8。
- V11（新）：一次尝试的结果行 `high_water_mark_reset=` 不是 `ok`，或 `resident_*`、`vm_rss_at_start_kib`、`vm_hwm_kib` 任一项读不出，那一次作废（`test-discipline.md`「读不到 ≠ 读到 0」），这一格这一轮重交。
- **S3 换写法（替换第 292 行）**：A7 出 ±5%；A9 出界；或两条量法互不包含——H > P + 32 MiB，或 P > 1.15 × H + 64 MiB；或第八节修订的两条轨迹断言不符。两个门槛照抄原 S3。这两条量法是 H（装置自己读 `/proc/self/status`）与 P（服务读 scope 的 `memory.peak`，减同台空跑格），读的人不同、读的计数不同（进程的常驻页计数对 cgroup 的记账页计数），P 另含页表与内核记账的那一截。**少了什么**：原 S3 的分配器峰值不经内核记页、不受 glibc 留不留页影响，与 P 只共用「同一个进程、同一次跑」；换成 H 之后两条都只认「碰过、还在的页」，一处让页没碰到或没算进来的毛病（比如 glibc 换了长大 Vec 的办法）两条会一起错，S3 看不出来，只剩 A7 的式子能看出来，而 A7 的式子本身就是按这个口径推的。这一格因此只核「同一种页两边读得对不对得上」，不再核「逻辑字节与常驻页对不对得上」。
- **修订一 S5 读成**：修订一 A11、A12（第七节修订）出 ±5%；或两条新臂上本修订 S3 的两条量法互不包含，停。
- 够判停机、V1–V3、V5–V10 不动。

**第十四节修订**：
- 原「串行：」一行（7bdb1b2c 那一版第 552 行）「判的量是每格自己 scope 的 `memory.peak` 与进程内分配器峰值，不随别的格并行而变」读成「……与进程内的 H，不随别的格并行而变」：H 是这一个进程自己的数，别的格在别的进程里。
- 原「要不要双机」一行（第 557 行）的数是分配器口径；常驻页口径推的（第四节修订第 1 条）：四盘 baseline 约 3.38 GiB、baseline_pc 约 4.13 GiB、split 约 1.70 GiB，read_stage_cache 约 6.46 GiB。挂钟、核数、分片与修订一第十四节修订相同（每格少的只是分配器那几次计数的开销，推的，不改预计）。

**未定项（修订三）**：
- 第八节的 3 盘：常驻页口径下推得对判的 P 不反向（第四节修订第 1 条）。要不要为甲-直线另找一个反向取样点，或照实报「3 盘对 P 不反向」，由主 agent 定；本修订不改第八节。
- 第二台的核数与内核没现查：A9 在 K = 1 上的门槛换成字节约 2.0 MiB，内核计数的攒批误差推的远小于它；第二台若核数多到每 CPU 一批超过约 1 MiB，A9 在 K = 1 上分不出，那时由主 agent 定。
- 修订一 A12 的推算（两臂留下的放掉的堆页相同）是推的；第一件扫环格出来出 ±5% 时走 S5 停下，先查是推算错还是缓存没活到期末，不当场改门槛。

**第十三节修订：修订三读过的文件与跑过的命令**见登记修订稿交回报告 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-e197-reregister/result.md`「读过的文件」「跑过的命令」两节（文件加行号区间，命令原样）。


### 修订四（2026-11-05，执行员量完 A7 的 24 格与 128 行诊断读数之后、产物之前）：A7 在默认环境下 split_pc 出界；单测改在固定 mmap 门槛下判

谁写、什么时点：执行员（批次 `records/2026-10-18-里程碑三收尾与近期欠账.md#19`）。时点在产物之前，量的：`ls research/results | grep -ci e197` 得 14，`grep -c 'e197\|E197' scripts/experiment-apparatus/replay.sh` 得 0。依据是第七节 A7 的规定「不符不作废：按量到的值代进 A7 的式子，差写进第十二节修订，时点在产物之前」和任务 6356 的装置单测日志（`/home/fy5090/.local/share/singlefs/work/m3-close-b19-e197-apparatus-r3b/unit-test-6356.log`，320 行，下面「日志 N 行」都指它）。原文、修订一至三一个字不改；A7 的式子 R 与 ±5% 一个字不动，不放宽；判的量 P、门槛、失败条款一个不动。

1. **读数（量的）**。A7 的 24 格（K=64 取 8192 槽，K=1 取 32768 槽；四个胶水臂；1、2、3 盘；默认环境）：23 格的 H÷R−1 在 −1.57%（日志 126 行）到 +1.70%（125 行），一格出界：split_pc、3 盘、K=1、32768 槽，H=81104896，R=62827179，+29.09%（137 行）。同一个二进制另量 128 格（K=64 与 K=1，环槽数 8192、32768、65536，1–4 盘，四个臂），每格量两遍，差别只在子进程环境是否带 `MALLOC_MMAP_THRESHOLD_=131072`（日志 174–301 行）：默认环境 7 格出 ±5%，其中 5 格是 split_pc 的 3 盘起（204、205、220、221、236 行）、1 格是 split_pc 的 65536 槽 2 盘（235 行）、1 格是 baseline_pc 的 1 盘 K=1 8192 槽（−5.18%，194 行）；固定门槛下 2 格出 ±5%，都是 1 盘 K=1 8192 槽的 baseline（−5.51%，254 行）与 baseline_pc（−5.18%，258 行），不在 A7 的几何里；固定门槛下其余 62 格最大偏差 4.08%（262 行）。A7 的 24 格在固定门槛下最大偏差 1.03%（split_pc、1 盘、K=64，250 行）。出界的那一格固定门槛后是 −0.86%（284 行）。透明大页 madvise，内核 6.17.0-1028-oem，核数 32（日志 113 行；这一趟跑在第二台）。
2. **原因（读数判的）**：glibc 的动态 mmap 门槛。依据：唯一变量是子进程环境里的 `MALLOC_MMAP_THRESHOLD_`（设了它 glibc 不再按放掉过的大块调高 mmap 门槛）；split_pc 3 盘 K=1 32768 槽 +28.79%（220 行）→ −0.86%（284 行），65536 槽 3 盘 +19.62%（236 行）→ −0.02%（300 行），8192 槽 3 盘 +26.32%（204 行）→ −1.35%（268 行）。默认环境下 baseline 与 baseline_pc 的 32 格没有一格高于 R（最高 −0.07%，229 行）。两个子效应（中等大小的 Vec 走堆、翻倍留洞；trim 门槛随之抬高、堆顶放掉的页不还系统）这批读数分不开。
3. **R 不补项**：固定门槛下 R 对 split 与 split_pc 的 32 格都在 ±4.1% 内，没有漏的瞬时项；默认环境下多出来的洞量（H−R）随盘数、环槽数不单调：split_pc 3 盘 +4134229（8192 槽，204 行）、+18089301（32768 槽，220 行）、+24656555（65536 槽，236 行），4 盘 +3687765（205 行）、+15709525（221 行）、+363179（237 行），65536 槽 2 盘 +5786283（235 行），写不出一个不看读数的式子；不拿读数去配一个补项。
4. **单测的处置**：装置的 A7 单测（24 格）在子进程环境里固定 `MALLOC_MMAP_THRESHOLD_=131072` 再量 H，容差 ±5% 不动，默认环境下的读数不进这条断言。这只是单测核「R 的式子与装置读 H 的做法对不对」时去掉 glibc 状态这个干扰，不放宽任何门槛；产物里每一格的环境不在这一条里改（见下面未定项）。
5. 单测里放过的 K = 1 的 ±10% 回到登记的 ±5%：登记第七节 A7 与修订三第七节 A7 没有给 K = 1 单开容差，±10% 是改前单测自己带的。

**未定项（修订四）**：
- 产物里的格仍在默认环境下量 H（驱动不设该变量）。默认环境下 split 与 split_pc 的格在 A7 上可能同样出界，那时走原 S3 停机，本修订不改。单测几何里每盘的记录 Vec 是 10.5 MB，低于 glibc 抬门槛的上限 32 MiB，所以放掉它会抬门槛；产物里每盘 196 608 槽的记录 Vec 是 62.9 MB，放掉它不抬门槛，但产物里别的被放掉的块会不会抬、抬到哪，没有读数（推的，没量）。要不要在 S3 因 A7 停机时，对同一格补一次固定门槛下的复量（驱动加一个环境选项，只补信息、不改判的量 P），或产物格一律固定门槛（P 的环境与真实调用方的默认环境不同），由主 agent 定。
- 固定门槛下 1 盘 K=1 8192 槽的 baseline 与 baseline_pc 偏低 5% 出头（254、258 行）：不在 A7 的几何里，登记 A7 末句已写「K = 1 上两张树约占 R(1) 的 62%，出界先查树那一截」，这一段没查，A7 若改取那个几何时要先查。

**主 agent 定（2026-11-05）**：未定项第一条取「不动」：产物格照默认环境量 H、驱动不加环境选项、不改判的量 P 的环境；A7 在产物上出界照原 S3 停机，停了再看那一格的读数定下一步。

## 十三、读过的文件与跑过的命令

**规则与定义**（整份）：`.claude/agent-common.md`；`.claude/singlefs-ai-sop/rules/test-discipline.md:1-122`；`.claude/rules/three-way-inference.md:1-166`；`.claude/singlefs-ai-sop/rules/evidence-discipline.md:1-179`；`.claude/rules/mutation-sampling.md:1-93`；`.claude/rules/implementation-first.md:1-19`；`.claude/rules/test-script-discipline.md:1-106`。
**岔路单与条款**：`research/prompts/m3-close-c653-forks.md:1-8`；`.claude/kb/checks-owed.md:1`、`:411`、`:572`（grep 命中行）；`.claude/kb/decisions/23-journal的角色与格式.md:532-549`（已定项 19 整节）与同文件 196 行（grep 命中的标题行）；`.claude/kb/milestone/03-third-txn.md:40`、`:157`（grep 命中行，157 行只读了 C653 前后各一段）；`.claude/kb/milestone/03-third-txn.ja.md:40`、`:163`、`:164`、`:166`（grep `七·2\|三七` 命中，原样打进了输出）；`.claude/kb/experiments.md` 只读了索引行行首的实验号（`grep -o '^| E[0-9]\+'`）。
**`crates/`**：`crates/singlefs-core/src/recovery.rs:115-225`、`:295-375`、`:4300-4560`，与 grep 命中的 `:1010`、`:1182`、`:1209`、`:7156`、`:7387`、`:7512`、`:7530`、`:7730`、`:7841`；`crates/singlefs-core/src/journal.rs:30-50`、`:100-330`，与 `:330-420` 里 grep 命中的行；`crates/singlefs-replay-prefix/src/lib.rs:990-1066`（其中 1006-1066 读全）、`:3990-4080`；`crates/singlefs-format/src/lib.rs:230-270`、`:585-610`，与 grep 命中的 `:14`、`:241`、`:251-265`、`:338`、`:440`、`:459-469`、`:598-602`；`crates/singlefs-core/src/pointer.rs:15-29`、`:98`、`:102-189`、`:283-297`；`crates/singlefs-core/src/address.rs:14`、`:18`、`:26`、`:30`、`:41`；`crates/singlefs-core/src/root_record.rs:25-40`、`:74`；`crates/singlefs-core/src/transaction.rs:460-540`，与 grep 命中的 `:951`、`:3048`、`:5635`、`:11882`、`:11893`；`crates/singlefs-core/src/system_configuration.rs:800-830`，与 grep 命中的 `:113`、`:123`、`:233`、`:278`、`:544`、`:678`、`:725`、`:813`、`:819`；`crates/singlefs-core/src/mounted_read.rs:645-660`；`crates/singlefs-core/src/system_configuration_repair.rs:250-262`；`crates/singlefs-core/src/mount.rs:7836`、`:7852-7900`、`:8040-8055`、`:8190-8200`，与 grep 命中的 `:6133`、`:6532`、`:6544`、`:10243`、`:10282`、`:11339`、`:11802`；`crates/singlefs-core/src/checksum.rs:1-82`（只看了 grep 出的 `use`/`fn` 行与 60-82）；`crates/singlefs-core/src/allocator.rs:264`、`:274`（grep 命中）；`crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs:150-223`（计数分配器写法，不含结果）；`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:1428-1434`（grep 命中 VmHWM）；`grep -rn 'scan_journal(' crates` 与 `grep -rn 'impl.*PoolReader for\|impl.*BlockDevice for' crates` 列出的每一行（四十多处调用点与二十几处实现的签名行，原样在会话输出里）；`grep -rn '196.\?608\|196_608' crates` 命中的十行（`crates/singlefs-format/src/lib.rs:465`、`crates/singlefs-checker-tier/src/crash_judge_tables.rs:967`、`crates/singlefs-harness/tests/presumed_clause_checks.rs:628`、`crates/singlefs-harness/tests/mounted_read_by_offset.rs:5`、`:662`、`:698`、`:744`、`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:5203`、`:8190`、`:23156`）。
**脚本与登记**：`scripts/hosts/run-with-memory-cap.sh:1-80`、`:120-135`，与 grep 命中的 `:151`、`:401`、`:413`、`:469`、`:491`、`:569`、`:653-658`、`:769-870`、`:1204`、`:1402-1403`、`:2081-2093`、`:3162`；`scripts/kb/claim-experiment.sh:1-40`；`.claude/heavy-thread-default`；`.claude/gate.d/doc-experiments.sh` 的 `cell_preregistration_parallelism` 函数与 grep 命中的 `:26`、`:66`、`:67`、`:105`、`:129`；`scripts/kb/quote-kb.py --help` 的前 30 行；`git ls-files | grep -i 'e183\|e185'` 的文件名。
没读：`.claude/kb/experiments/` 下任何文件、`research/results/` 下任何文件、别的实验的跑前登记、`records/`、`.claude/kb/decisions-history.md`、`scripts/experiment-apparatus/perf-by-milestone.md`。

**跑过的命令**（只读与草稿目录里的写；原样）：

```
bash scripts/hosts/work-directory-ownership.sh claim /home/fy5090/.local/share/singlefs/work/m3-close-b16-c653-design --purpose 'C653 扫环胶水内存的跑前登记'
bash scripts/kb/claim-experiment.sh --next
grep -o '^| E[0-9]\+' .claude/kb/experiments.md | sort -t E -k2 -n | tail -3
bash scripts/kb/claim-experiment.sh E197 扫环胶水内存随盘数
grep -n 'C653' -r .claude/kb --exclude-dir=experiments --exclude=experiments.md --exclude=experiments-history.md
grep -rn '扫环核胶水' .claude/kb --include=*.md --exclude-dir=experiments --exclude=experiments.md --exclude=experiments-history.md
grep -n '七·2\|三七' .claude/kb/milestone/03*.md
grep -n '上界与改法\|C653' .claude/kb/milestone/*.md .claude/kb/decisions.md .claude/kb/pending-rules.md
python3 scripts/kb/quote-kb.py /home/fy5090/.local/share/singlefs/work/m3-close-b16-c653-design/quoted-clause.md '.claude/kb/checks-owed.md~^\| C653 \|' '.claude/kb/decisions/23-journal的角色与格式.md@#### 已定项 19：空发布记录的事务号与提交标记、`previous_hash`、环长'
sha256sum crates/singlefs-core/src/recovery.rs crates/singlefs-replay-prefix/src/lib.rs crates/singlefs-core/src/journal.rs
grep -rn 'MAX_DEVICE\|MAXIMUM_DEVICE\|DEVICES_PER_POOL\|device_count.*上限\|盘数上限' crates/singlefs-format/src crates/singlefs-core/src --include=*.rs | wc -l
awk 'NR>=4431 && NR<=4484' crates/singlefs-core/src/recovery.rs | sha256sum
awk 'NR>=4029 && NR<=4058' crates/singlefs-replay-prefix/src/lib.rs | sha256sum
python3 -I /home/fy5090/.local/share/singlefs/work/m3-close-b16-c653-design/predict.py
python3 -I /home/fy5090/.local/share/singlefs/work/m3-close-b16-c653-design/anchors.py
TZ=Asia/Tokyo date +%F
```

草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-b16-c653-design/`：`quoted-clause.md`（第二节出口）、`predict.py`（sha256 `29bf96ee4e0320c81c061dfe54684c16c6ceb2a0cc4040ddff947e656ddf7e6a`）、`predict-output.txt`（sha256 `0109a7dd396eee906c39ab7b8190653833147cffbcd98ecad7375a4f2896c2e3`）、`anchors.py`（sha256 `9b987d8fb87ffe355d9601a357d9f3c322483e51ecb524a2cea00aa50f875940`）、`anchors-output.txt`（sha256 `8aa3be25a04225918640bb7c3d40731cab294ad2db49d7a4c747d01c402baf47`）。

## 十四、并行与挂钟

独立单位：每个（臂, 盘数, 槽填满档, 点名项数, 重复号）格是一个独立进程，池现算、不读盘、不共享状态；差分集 D 的四例各一个进程；格与格之间没有先后依赖（扣空跑格是出表时算）。
线程：编译装置经 `bash scripts/hosts/capped.sh 31 …`（登记值 31，`.claude/heavy-thread-default`）；跑格时每个进程单线程，同时跑的格数由内存包装的总上限定，最多 5 格并行（每格上限 8G、包装对没记过峰值的键按 8G 记账，本机 slice 总上限约 40 GiB，推的）。
串行：无必须串行的步——判的量是每格自己 scope 的 `memory.peak` 与进程内分配器峰值，不随别的格并行而变；`scan_seconds` 是附带量、不判，所以计时也不要求独占机器；同一格的 5 次重复可以并行。
预计挂钟：编译段约 10–25 分钟（推的，checker-tier 包依赖全仓，按增量编译缓存在不在浮动）；跑格段 4 个扫环臂 × 4 档盘数 × 3 档几何 × 5 次 = 240 次扫环进程，每槽按现编一条 + 解一条约 15 µs 推（两遍 4 KiB 的软件 CRC-32C 加两次 4 KiB 分配，推的），4 盘一格约 12 秒、平均约 7.4 秒，加空跑与 measurement_pc 65 次与包装起 scope 每次约 1 秒，串行约 35 分钟、5 格并行约 8 分钟；差分集 D 几秒；出表几秒；合计约 20–35 分钟，2 小时内。
预计在用核数：编译段 31 个；跑格段 5 个（5 格并行、各单线程）；出表段 1 个。

长活还是短活：短活（2 小时内），用不着分片、续跑与多机三样声明。
要不要双机：不要。默认环几何四盘的内存（推的，第四节 `predict.py`，分配器口径）：基线约 3.52 GiB，baseline_pc 约 4.27 GiB，split 约 1.74 GiB；cgroup 口径另加 malloc 块头与进程底子，按每格上限 8G 留了余量；5 格并行合计不超过约 22 GiB，单台放得下。
