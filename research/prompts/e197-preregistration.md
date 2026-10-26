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
