# m3-delete-r1 正推腿报告（Sonnet，三方轮的正推立场）

日期：2026-09-30。范围：正文第三节 U1–U7 全部。方法：从已定条款与 `crates/` 今天的实现推每格「应该是什么」，再与背景材料、事实表、里程碑表逐格比；一样就写一样，冲突把两边整行并排抄。
没跑任何编译、测试、门禁。「推的」= 我读代码或条款推出来、没跑过；「现查」= 这一次 grep / 读原文得到。引 kb 写 kb 文件自己的行号（去文件里现查的），引代码写 `crates/` 下那份文件自己的行号。

## 0　实现今天的样子（现查；与事实表逐条对过，对得上的不再重复，只列下面推导要用的）

冻结副本与仓里 `crates/` 现查逐字相同：`diff -rq crates /tmp/claude-1000/m3-ring-r1/tree/crates; echo "diff exit=$?"` 输出 `diff exit=0`（一行差异都没有）。所以下面的行号在两处都对。

| # | 事实（我现查的） | 出处 |
|---|---|---|
| a | 恢复重建一版只认「一个文件 = inode 1」：extent 树读回来要恰好一条叶条目 | 「extent 树上段里不是恰好一条叶条目」（`crates/singlefs-core/src/recovery.rs:3354`）；「extent 树里第一个文件一个数据单元都没有」（`crates/singlefs-core/src/recovery.rs:3365`） |
| b | 同一个恢复函数还要 inode 树非空、且有 inode 1 那条记录 | 「inode 树根没有条目」（`crates/singlefs-core/src/recovery.rs:2452`）；「inode 树里没有第一个文件那条记录」（`crates/singlefs-core/src/recovery.rs:2505`） |
| c | 冷走读（`recover()` 用的那条）也调同一个 extent 适配，但「inode 1 记录不在」已经交回 `Ok(None)` | 「extent_tree = Some((entry.tree, extents_of_the_first_file(read)?));」（`crates/singlefs-core/src/recovery.rs:4021`）；「let Some(inode_record) = inode_record else {」（`crates/singlefs-core/src/recovery.rs:4144`） |
| d | 一版在内存里只有一条 inode 记录与一串数据指针 | 「pub inode_record: InodeRecord,」（`crates/singlefs-core/src/transaction.rs:2501`）；「pub data_pointers: Vec<DataPointer>,」（`crates/singlefs-core/src/transaction.rs:2482`） |
| e | 读侧 `open_file` 已经按 inode 号泛化（找不到报 `NoSuchInode`），extent 树读回也已是「按 inode 号升序的多文件」形状 | 「.ok_or(OpenFileFailure::NoSuchInode { inode })?;」（`crates/singlefs-core/src/mounted_read.rs:627`）；「pub files: Vec<(u64, ExtentsOfAFileReadFromDisk)>,」（`crates/singlefs-core/src/extent_tree.rs:1089`） |
| f | nlink 写死 1、读时跳过；`InodeRecord` 没有这个字段 | 「writer.put_u32(1); // nlink」（`crates/singlefs-core/src/records.rs:56`）；「reader.skip(8 + 16);」（`crates/singlefs-core/src/records.rs:85`）；`grep -rn 'nlink' crates/ --include=*.rs` 全仓只命中 `records.rs:56` 一行 |
| g | 释放一个落点不减「已分配」，只加 defer；「已分配」到回收才减；准入读的就是「已分配」 | 「self.deferred_slots += span;」（`crates/singlefs-core/src/allocator.rs:556`）；「self.allocated_slots -= span;」（`crates/singlefs-core/src/allocator.rs:673`，在 `mark_reclaimed` 里）；「allocated: BytesOnOneDevice::of_slots(device_map.allocated_slots()),」（`crates/singlefs-core/src/admission.rs:329`）；单测「assert_eq!(pool.devices[0].allocated_slots(), 5, "占着的槽数不变");」（`crates/singlefs-core/src/allocator.rs:2020`） |
| h | 记账里 defer 那一行的注释自己写着「仍算在已分配里」 | 「第 5 项：已释放、还在 defer 窗口里的（它们仍算在已分配里：占着空间、被根环里的有效根引用）。」（`crates/singlefs-core/src/transaction.rs:6303`） |
| i | 「待删占用」那一行与准入读数读的是同一个常量，恒 0；checker 一处都不读 | 「crate::admission::PENDING_DELETE_OF_THE_FIRST_VERSION.0」（`crates/singlefs-core/src/transaction.rs:6284`）；`grep -rn '待删\|PENDING_DELETE' crates/singlefs-checker/src/*.rs` 输出为空（这一次现跑） |
| j | 判不判空间准入按角色分：extent 节点、inode 节点、数据单元是「普通分配」（判），四棵固定点树与树表走保留池（不判） | 「pub const fn space_budget_of_role(role: TransactionUnit) -> SpaceBudgetOfARole {」（`crates/singlefs-core/src/admission.rs:919`）；「pub fn publish_has_an_ordinary_allocation(rewritten_roles: &[TransactionUnit]) -> bool {」（`crates/singlefs-core/src/admission.rs:943`） |
| k | 「文件变短」释放多出的数据单元，只在这次重写了数据单元时才算；没有「不重写数据、只截短」的入口 | 「(rewritten_data_units..previous_data_units)」（`crates/singlefs-core/src/transaction.rs:2858`）；那一支在「= if rewritten_data_units == 0」（`crates/singlefs-core/src/transaction.rs:2854`）之后 |
| l | 数据单元数与 inode size 绑死：写侧、读侧、恢复三处同一条除法，文件从 0 起连号无洞 | 「content_length_in_bytes.div_ceil(payload_capacity).max(1)」（`crates/singlefs-core/src/write_request_split.rs:61`）；「DataUnitsOfTheInode::NoneWritten if file_size_in_bytes == 0 => 0,」（`crates/singlefs-core/src/write_request_split.rs:83`）；「data_unit_count_implied_by_the_file_size(inode_record.size, data_units_of_the_inode)」（`crates/singlefs-core/src/mounted_read.rs:727`）；「fn data_unit_count_matches_the_inode_size(」（`crates/singlefs-core/src/recovery.rs:3739`） |
| m | inode 树写入口只有「原地换一条」与「末尾追加」，没有删记录；模块注释写明「记录只增不减」 | 「记录只增不减，合并（只许左吸收右 / 左从右借）没有对象。」（`crates/singlefs-core/src/inode_tree.rs:17`） |
| n | 全仓没有 `deleted_inodes` / `logged_ops` / unlink / 墓碑类记录 | `grep -rn 'logged_ops\|deleted_inodes' crates/ --include=*.rs` 只命中两处注释（`inode_tree.rs:16`、`walk.rs:6133`）；`grep -rn -i 'unlink' crates/ --include=*.rs` 零命中；`grep -rn 'TREE_KIND_DELETED\|TombstoneRecord\|PACKED_TYPE_TOMB' crates --include=*.rs` 零命中；打包记录类型只登记了 2 与 4：「pub const PACKED_TYPE_INODE: u16 = 2;」（`crates/singlefs-core/src/unit.rs:21`）、「pub const PACKED_TYPE_INSTANCE_TABLE: u16 = 4;」（`crates/singlefs-core/src/unit.rs:22`） |
| o | 树表读写对没登记的种类：重建只比「树表里有的种类」，多一条不认识的不拒；冷走读遇到有根的没登记种类报「树的种类没登记」 | 「只比树表里有的种类」（`crates/singlefs-core/src/recovery.rs:2233`）；「what: "树的种类没登记",」（`crates/singlefs-core/src/recovery.rs:4045`） |
| p | 管理员回退新根的树表：只有 extent、inode 两种取目标那一版的根（条件在 `crates/singlefs-core/src/mount.rs:4972`，取根在下一行） | 「target_root_of_the_tree_of_kind(entry.kind)」（`crates/singlefs-core/src/mount.rs:4973`） |
| q | 可写挂载写序列的四个挂点 | 暖机计划「let warm_up_publishes_planned =」（`crates/singlefs-core/src/mount.rs:3708`）；取号前预演「let placements_planned = match dry_run_of_the_publishes_after_acquisition(」（`crates/singlefs-core/src/mount.rs:3767`）；暖机循环「for planned_txg in &warm_up_publishes_planned {」（`crates/singlefs-core/src/mount.rs:3940`）；交回「Ok(Mounted {」（`crates/singlefs-core/src/mount.rs:4018`） |
| r | 崩溃点名字与段计数 | 「name: format!("{entry_name}.{period}.{step_in_period}"),」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2389`）；计数器按（入口下标，周期名）分：「let mut steps_of_period_in_entry: BTreeMap<(usize, String), usize> = BTreeMap::new();」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2360`） |
| s | 一次带单元的普通发布的崩溃状态数 = 2^n + 11，n = 这次单元写的次数 | 「C577 之后一次带单元的普通发布：上一次轮换段 3² − 1 = 8 + 单元段 2^n − 1 + 记录段 3 + 根槽段 1 = 2^n + 11」（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:102`）；各次发布的数「instance2_txg6=65547」「instance2_txg10=268435467」「instance1_txg3=16777227」都在 `crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:106-107`（2^16 + 11、2^28 + 11、2^24 + 11）；全流总数「const FULL_STATES_THROUGH_THE_UNMOUNT: u64 = 1_662_648_564;」（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:100`） |

由 s 反推每种发布的 n（推的，与测试里的写数吻合）：空发布 n = 16（8 个单元 × 2 份：分配记录树 5 个节点 + 记账 1 + 映射 1 + 树表 1）；第一个文件版本 / 简单覆盖写 n = 24（再加数据、extent 根、inode 叶、inode 根 4 个单元）；后面几次覆盖写 n = 28（分配记录树多脏两个节点）。测试里每次覆盖写共 33 次写（「("publish_overwrite", 33),」`crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_unmount.rs:222`）= 28 次单元写 + 2 次 journal + 1 次根槽 + 2 次系统配置轮换。下面所有「崩溃状态数」都用 2^n + 11，n 的取值区间我标出「推的」。

## 1　U1　格式：`deleted_inodes` 与意图记录住哪（C118）

**判定**：规则没说（C118 还开着）；其中「key 只存号、inode 记录留在 inode 树里」一格与 D8 一致。

### 1.1　已定的框（推导的起点，整行引）

- 删除中的 inode 进单独一棵树：「**定案**：删大目录、截断大文件会产生海量墓碑，塞进一个事务必然爆掉。」下一句就是「删除中的 inode 单独进 `deleted_inodes` 树，由后台回收」（`.claude/kb/decisions/08-核心索引结构.md:107`，整条在这一行）。
- 树里放什么：「**inode 记录在回收完成前留在 inode 树里（nlink = 0），`deleted_inodes` 只存号**。」（`.claude/kb/decisions/08-核心索引结构.md:171`，该行最后一句）。
- 意图只写目标态：「1. **意图声明目标状态，绝不声明增量。**「把文件 X 截到 N 字节」可以重复执行，「再删 1000 个 extent」不能。」（`.claude/kb/decisions/08-核心索引结构.md:121`）；I-4.4「任一未完成的意图 key，其字段只描述目标状态，不含任何增量字段」（`.claude/kb/invariants.md:168`）。
- 队列与 inode 树的关系要能判：「`deleted_inodes` 里的每个 inode 号在同一棵树的 inode 树里存在且记录的 nlink = 0」（`.claude/kb/invariants.md:279`）。
- 加一棵树是 compat：「| 新增一棵 keyspace | 若树表 day-1 就是开放列表：只是看不到新树；若树数量硬编码：读不出树表 | **compat**」（`.claude/kb/decisions/15-格式冻结政策.md:190`）；树的种类码只追加不重用：「**定案**：树表条目里「树的种类」那 2 字节的取值，唯一登记位是已定项 9 这一节的登记表；**新树只追加不重用码**，0 保留为「无」。」（`.claude/kb/decisions/08-核心索引结构.md:279`）。
- 先例（三棵零根树 day-1 进树表）：「day-1 注册进树表比第一次建快照时惰性创建少一条」（`.claude/kb/decisions/08-核心索引结构.md:295`，后面接「首次建树」路径）；稀疏旁表同一写法：「day-1 注册进树表（新池新建文件多一条 200 字节的树表条目，根指针为零）」（`.claude/kb/decisions/05-快照-空间记账机制.md:172`）。

### 1.2　从这些推出的形态（推的，标出哪一步是推）

1. **意图就是队列里的那一条 key，不需要第二个载体（推）。** unlink 一个 inode 的目标态只有一句：「inode N 没有 extent、没有 inode 记录、不在队列里」。这句话由 N 唯一确定，不含任何要数的量，所以队列条目只需要 key = inode 号 8 字节、没有 value，I-4.4 在这个形态上是恒真（一个没有字段的条目写不出「再删 N 个」）。进度不放在条目里：进度就是 extent 树里 N 还剩几个单元（第 3 节）。
2. **`deleted_inodes` 用码 2 节点、key 宽 8、条目宽 8（推）。** 头 = 86 + 2×8 + 29 = 131（同 inode 树根，`crates/singlefs-format/src/lib.rs:59`「pub const INODE_TREE_ROOT_INDEX_NODE_HEADER_BYTES: u64 = 131;」），叶容量 ⌊(16384 − 131) ÷ 8⌋ = 2031，内部条目 8 + 86 = 94、扇出 ⌊16253 ÷ 94⌋ = 172（算术：`python3 -c 'print((16384-131)//8,(16384-131)//94)'` 输出 `2031 172`）。「条目 8 字节、value 空」有先例：livelist 的节点内部布局登记成「条目 36 字节、value 空」（`.claude/kb/freeze-layer-membership.md:36`）。
3. **树 ID = 19、种类码 = 9（推）。** 发号次序是「extent、inode、分配记录、记账、中央映射、livelist、稀疏旁表、deadlist」连号 11..18（`crates/singlefs-format/src/lib.rs:182`「pub const TREE_IDENTIFIER_DEADLIST: u64 = 18;」），追加一个就是 19，水位 19 → 20（`crates/singlefs-format/src/lib.rs:185`「pub const TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH: u64 = 19;」）；种类码今天登记到 8，缺 5（中央映射不进树表）：「pub const TREE_KIND_DEADLIST: u16 = 8;」（`crates/singlefs-core/src/records.rs:267`），追加 9。I-9.16 的发号次序判定只比树表里有的种类：「只比树表里有的种类」（`crates/singlefs-core/src/recovery.rs:2233`），名单是「const TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER: [u16; 7]」（`crates/singlefs-core/src/recovery.rs:2187`），种类 9 不加进名单也不会被拒，加进去才被判次序。
4. **何时建：day-1 零根注册（照 deadlist 先例）比第一次 unlink 惰性建少两条路径（推）。** 第一个文件版本那次发布写树表时多写一条根指针为零的条目（树表 7 → 8 条、声明长度 1400 → 1600 字节，算术 8 × 200），不多写一个单元（树表本来就是一个单元）。惰性建的两条额外路径：(i) 第一次 unlink 那次发布要临时往树表加一条并发新树 ID；(ii) 管理员回退要取 R_old 那一版的 `deleted_inodes`（C118 与 D8 已定项 6 都要求，`.claude/kb/decisions/08-核心索引结构.md:171` 「`deleted_inodes` 也取 R_old 那一版的」），而今天取根的那段只认 extent 与 inode 两种（`crates/singlefs-core/src/mount.rs:4973` 「target_root_of_the_tree_of_kind(entry.kind)」，条件在 `mount.rs:4972`），且取不到时是 `expect`；R_old 若在第一次 unlink 之前，惰性建的池里 R_old 那一版没有这条，新根的树表要「删一条」。零根注册下每个带文件的版本都有这条，取根统一。
5. **态别：派生态（推，需主 agent 认）。** 队列丢了，从 inode 树里 nlink = 0 的记录能重算出来（`.claude/kb/decisions/08-核心索引结构.md:171` 保证 inode 记录在回收完成前留在树里）；重建不出时读者拿到的是「删除永远不完成（泄漏）」而不是错答案，落 D21 的判据：「重建不出这个值时，读者拿到的是保守答案还是错答案」（`.claude/kb/decisions/21-权威态与派生态的分界.md:216`）。⇒ 登记两行：key 编码落第 3 层、节点内部布局落第 4 层，态别派生态、退出冻结，依据 D8 已定项 6 与 D21 已定项 10（门禁要每棵 `TREE_IDENTIFIER_*` 在这两层各一行：「每个层与每个结构在表里都有行，漏一个判红」`.claude/kb/freeze-layer-membership.md:96`）。这一步是我的读法，D21 那句话没有对 `deleted_inodes` 点过名；主 agent 若不认，两行落「判不动」（与 livelist 三行同待遇），不影响其余各格。

### 1.3　候选与代价数（每个候选多写几个单元、几条记录、几字节）

基线「只把 inode 记录翻成 nlink = 0」的意图发布 P0 = 8 个固定点单元（分配记录树 5、记账 1、映射 1、树表 1）+ inode 叶 + inode 根 = 10 单元、n = 20（一个单元两份），崩溃状态 2^20 + 11 = 1 048 587（`python3` 算，见第 0 节 s 的式子）。下面「多」都是相对这个基线；一单元 = 16 KiB × 2 份 = 32 KiB 物理写。

| 候选 | 定义 | 建树时刻多写 | P0 多写单元 / 记录 / 字节 | P0 的 n 与崩溃状态数 | 挂载时没有待删除的代价 |
|---|---|---|---|---|---|
| K1（推荐） | 一棵 `deleted_inodes`（树 ID 19、种类 9、key 8 无 value），零根注册；不建 logged_ops | 第一个文件版本：树表 +200 字节条目，0 个单元 | +1 单元（队列根节点）；1 条 8 字节队列记录 + 1 条 55 字节映射条目；+32 KiB | n = 22，2^22 + 11 = 4 194 315（×4） | 读树表里这条的根指针 = 零，0 次额外设备读 |
| K2 | K1 + 一棵 `logged_ops`（种类 10、树 ID 20），unlink 同时写一条「种类 + 目标 inode」意图记录 | 树表 +400 字节、水位 19 → 21 | +2 单元、2 条队列 / 意图记录、2 条映射条目；+64 KiB | n = 24，2^24 + 11 = 16 777 227（×16） | 同 K1，另加一棵零根 |
| K3 | K1 但惰性建：第一次 unlink 那次才加树表条目 | 0 | 同 K1（+1 单元），另加「树表加条目」一条首次路径 | n = 22，4 194 315 | 同 K1；R_old 回退要处理「那一版没有这条」 |
| K4 | 不建树：队列 = inode 树里 nlink = 0 的记录，挂载时扫 | 0 | 0 | n = 20，1 048 587 | 多一遍对 inode 记录的扫描（今天重建本来就读全部叶：`crates/singlefs-core/src/recovery.rs:2450` 起那个 for 循环）；I-9.5 变恒真、与 D8 已定项 5 明写的「树」冲突 |
| K5 | 意图放记账行或树表条目预留位 | — | — | — | 出局，见下 |

- K5 出局的理由：记账条目是「幂等完整值」、key 是 (标签, 树, 设备, 代)，一行只能是一个统计量（`.claude/kb/decisions/08-核心索引结构.md:27` 已定项 1 标题就是「记账取幂等完整值，接受 key 数量上升」），装不下「哪个 inode 在删」；树表条目预留 76 字节里已认购 48（`.claude/kb/decisions/08-核心索引结构.md:248`「合计 48，余 28」），且预留位的任何赋义是 incompat，一条只能存一个号，装不下多个待删 inode。
- K4 出局的理由：它不违反任何已定条款的字面，但让 I-9.5 这条已立的不变量没有东西可判（队列就是被判的对象本身），并且把「续做」建在「挂载必须读完整棵 inode 树」上——今天成立，是因为重建一版把每片叶都读回来，D8 已定项 14 的「挂载怎么读」把 extent 树改成了按需，inode 树将来也可能改。
- K2 与 K1 的取舍只在一处：多一棵 `logged_ops` 现在有没有消费者。答案是没有：唯一的 unlink 意图就是队列里那一条（1.2 第 1 步）。D5 稀疏旁表与 D26 整理的意图各要什么形态第 7 节单独谈；新增一棵 keyspace 是 compat（`.claude/kb/decisions/15-格式冻结政策.md:190`），以后加不改 K1 的字节。

### 1.4　推荐与对 `crates/` 的改动（推的，按 `.claude/rules/implementation-first.md` 第 2 条写成对实现的改动）

推荐 K1。要动的地方（数字是 `grep` 现查）：
- `crates/singlefs-format/src/lib.rs`：加 `TREE_IDENTIFIER_DELETED_INODES = 19`；第 172 行树表条目数常量 7 → 8（`NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES` 全仓只有这一处定义引用）；第 185 行水位 19 → 20；节点头宽复用 131。
- `crates/singlefs-core/src/records.rs`：`TREE_KIND_DELETED_INODES: u16 = 9`（第 267 行之后）。
- `crates/singlefs-core/src/transaction.rs`：`FileVersionTreeIdentifiers` 由八个号变九个（`issued_from_watermark` 在 2390 行）；树表 `vec!` 七条变八条；`MultiLevelCodeTwoTree`（2053 行）加成员，因为码 2 树的计划、装、读、释放都已经按「字段宽 + 条目宽」泛化（`crates/singlefs-core/src/code_two_tree.rs` 的 `CodeTwoKeyFieldWidths` 只要给一个 `&[8]`）；`TransactionUnit` 加两个角色（根、根之下节点）；`grep -rn 'MultiLevelCodeTwoTree::' crates --include=*.rs` 在 `transaction.rs` 39 处、`recovery.rs` 7 处、`admission.rs` 3 处、`walk.rs` 2 处，是要逐处加臂的量。「expect("树表第 1 版起恒有七条")」（`crates/singlefs-core/src/transaction.rs:2670`）是唯一一处按条数写死的断言。
- `crates/singlefs-core/src/admission.rs:919`：`space_budget_of_role` 是没有通配臂的 `match`，新角色必须显式选一格（第 4 节的推荐要它归「普通分配」还是新的一格）。
- `crates/singlefs-core/src/recovery.rs`：2187 行名单 7 → 8；读树表条目那一段与冷走读 4045 行的种类分派加臂；`crates/singlefs-core/src/mount.rs:4972` 的回退取根加种类 9。
- 全仓引用树 ID / 种类常量的文件 `grep -rln 'TREE_IDENTIFIER_\|TREE_KIND_DEADLIST\|TREE_KIND_LIVELIST' crates/ --include=*.rs | wc -l` 输出 19，其中 `crates/singlefs-harness/tests` 下 10 个（同一条 grep 再 `| grep -c 'singlefs-harness/tests'` 输出 10）。
- kb：D8 已定项 9 登记表加第 9 行、已定项 11 的「树 ID 11..18、水位 19」改 19 / 20；`.claude/kb/freeze-layer-membership.md` 加两行（1.2 第 5 步）；`layout/01-first-txn.md` 里写着「7 条」「水位 19」的字节表行与 E142 干跑跟着改（事实表 (a) 的 a3 列了行号，我没逐行复核）；C118 还清、I-9.5 前置改成「已定」。

**什么现象会推翻 K1**：(i) 三·1 后续某一轮定下「unlink 的意图要带 unlink 之外的字段」（例如要记 ctime 或发起者），那时队列条目要有 value，K1 的「条目 8 字节」要改，是格式改动但软格式期只算同步成本（`.claude/rules/format-evolution.md:7`「在第一个外部用户出现之前，磁盘格式是软的」）；(ii) 量出 `deleted_inodes` 节点的多写使崩溃放量不可接受（P0 从 2^20 到 2^22，×4）；(iii) 主 agent 不认「派生态」，那只影响冻结登记两行，不影响 K1 本身。

## 2　U2　墓碑（D18 已定项 8 / 10，C84、C109）

**判定**：冲突。条款要求「删除在盘上留墓碑」，而墓碑要落的载体在今天的实现与 kb 里都没有落点。两边整行并排抄在下面（不加引号的块是整行原文，报告后面用 ≤400 字的片段另行核对）。

左边（条款）`.claude/kb/decisions/18-块里携带什么信息.md:202`：

> **定案**：**删除不再是「盘上什么都不留」——它在盘上留下墓碑记录。** 三句分开写：容器的单元类型标签取已定项 11 登记表的**码 3（打包记录单元）**；容器内的记录取**打包记录类型 1（墓碑区间记录）**；死者由墓碑区间记录自己的字段指认（对象 ID + 对象出生代 + 区间，已定项 10），**不用容器身份顶替记录身份**。为什么必须有它：删除若在盘上不留痕迹，扫盘重建就**没有任何输入**能知道这个 key 死过，而谱系重写序号救不了这一格（它只买到覆写那一格，且已定项 1 定案不做——盘上没有那条消息的任何痕迹时，序号无从比较）。

右边（载体欠账）`.claude/kb/checks-owed.md:98`（C103）：

> | C103 | 容器索引无落点 | D18（块里携带什么信息） 已定项 11 要求打包记录单元由一棵容器索引指向：它是打包单元物理指针（与 MAC）的唯一持有者，别的树按身份引用容器；key = (出生树 ID, 打包记录类型, 容器号, 容器出生代) 四段全进，容器号排在出生代之前，使 (树, 类型) 下最大容器号 O(log N) 可查——容器号分配器的下一个号 = 最大号 + 1，派生态、扫描可重建；计数器回退不破身份（出生代不同）。这棵树住哪、key 编码、它自己的更新代价、它独占的 MAC 重建时怎么重算，全仓无落点。第二轮正推腿：容器号分配器与 inode 号分配器是同一类从未处理过的问题。2026-09-05 起本条只覆盖打包记录类型 1（墓碑）：类型 2 的容器索引就是 inode 树的内部节点、容器号 = 触发分裂的新记录 inode 号（D8（核心索引结构） 已定项 6），这半支收口，检查列的「最大容器号一致」同样只对类型 1 | 容器索引落成分项之后：黄金镜像里造一个容器索引全丢的样本，扫描重建必须从码 3 单元头恢复出全部条目且各 (树, 类型) 的最大容器号一致；把容器号从头里删掉必须让恢复失败；伪造一条指向错误落点的条目必须被载荷校验和或 MAC 抓住 ⚠️ **两条现查过的事实（2026-09-06，主 agent 逐字核 D18（块里携带什么信息） 已定项 11 的码 3 类身份段表）**：① **容器的四个命名字段在码 3 头里全在**——出生树 ID（偏移 43）、打包记录类型（51）、容器号（53）、容器出生代（61）⇒ **扫描重建读得出容器身份**，不缺字段；② **打包记录类型是独立的 2 字节字段**，不是折在单元类型标签那 1 字节里 ⇒ 任何把「类型」塞进 1 字节标签的收口候选，先要回答那 2 字节的值域怎么办。 ⚠️ **2026-09-06 起它挡着的东西多了三样**：D2（RAID 条带策略） 已定项 12 定了条带成员表走容器索引 ⇒ C127（钉住状态在分配记录里没有落点） 的修法 ②、C128（清扫准入对 parity 格两支都判不出） 的修法、以及 **D9（加密） 已定项 9 那条让渡区 A 的签发判据第 ⑦ 条合取**（「不是钉住的格、也不属于任何含活成员的条带」）**全部要先走一次容器索引点查**。⇒ 它从一笔「日后要还」的账变成**三处的直接前置**。 ⚠️ **2026-09-06 试过一次收口，三腿判定不了案**：候选是「容器索引就是中央映射」（容器按对象进 D19（块指针的结构与宽度预算） 已定项 5 那层映射）。**栽在 value 装不下**——D18（块里携带什么信息） 已定项 11 逐字说容器索引是「打包单元物理指针（**与 MAC**）的唯一持有者」，而 MAC 128 位 + nonce 96 位住在指针头部 31 里，中央映射的 value（位置条目 14）只有密文校验和 4；D9（加密） 已定项 4 逐字「nonce 每 extent 存，不现算」⇒ 加密后解不开。**自举也会重演**，收敛在「中央映射自己的节点」那一层，而那一层零条款。逐条经过与剩下的四条候选路写在 `records/2026-09-06-C103容器索引收口轮.md`。 | 先在 D3（空间分配） 或 D8（核心索引结构） 开一条分项 | 2026-09-04 D18（块里携带什么信息） 已定项 11 三轮三方论证（第一轮立账，第二轮正推腿场景二补三句，第二轮反推腿 9.3 补 MAC 那半） |
要核对的两句（≤400 字片段）：条款「删除若在盘上不留痕迹，扫盘重建就**没有任何输入**能知道这个 key 死过」（`.claude/kb/decisions/18-块里携带什么信息.md:202`）；欠账「这棵树住哪、key 编码、它自己的更新代价、它独占的 MAC 重建时怎么重算，全仓无落点。」（`.claude/kb/checks-owed.md:98`）。实现一侧：打包记录类型只登记了 2 与 4（第 0 节 n 行），`grep -rn '清扫水位\|STATISTIC_SWEEP' crates --include=*.rs; echo "exit=$?"` 输出 `exit=1`（零命中）——D5 统计量清单第 13、14 项（墓碑回收门要读它们，`.claude/kb/decisions/05-快照-空间记账机制.md:127`「| 13 | 清扫水位」、`.claude/kb/decisions/05-快照-空间记账机制.md:128`「| 14 | 最近一次根销毁的代号」）在代码里没有登记：「pub const STATISTIC_INODE_WATERMARK: u16 = 12;」（`crates/singlefs-core/src/records.rs:23`）是登记表的最后一项。

### 2.1　一次 unlink 死掉哪些 key，哪些要墓碑（推）

| # | 死的东西 | 在哪、态别 | 怎么死 | 要写墓碑吗 | 依据 |
|---|---|---|---|---|---|
| 1 | inode N 的记录 | inode 树叶（打包记录类型 2，权威态单元） | 最后一批里从叶里删去 | 不写 | 「记录死亡不写墓碑，死亡由现行叶版本体现」（`.claude/kb/checks-owed.md:104`，C109 那一行自己写着「2026-09-05 已答」） |
| 2 | N 的 extent 条目（上段叶条目 + 下段叶记录） | extent 树，派生态 | 每批重写下段 / 上段节点，新节点里不再有 | D18 已定项 10 字面要 1 条「对象整个删除写**一条区间记录**（对象 ID + 对象出生代 + 全区间）」（`.claude/kb/decisions/18-块里携带什么信息.md:232`）；一次 unlink 不论多少 extent 都是 1 条，不是每批一条 | 第 2.2 节 |
| 3 | 被释放单元的映射条目 | 中央映射树，派生态 | 每批重写映射叶 | 不写 | 「映射可从单元自描述扫描重建」（`.claude/kb/freeze-layer-membership.md:66`） |
| 4 | 被释放落点的分配记录 | 分配记录树，派生态 | 记录改成「已释放 + 释放代」，条目不删（第 0 节 g） | 不写 | 释放代自己就是痕迹（`.claude/kb/decisions/21-权威态与派生态的分界.md:222`，「保守答案」那一行） |
| 5 | N 在 `deleted_inodes` 里的条目 | 队列，派生态（1.2 第 5 步） | 最后一批里删去 | 不写 | 队列条目本身是意图，完成即删（D8 已定项 5：「删除意图必须与最后一批工作在同一个事务里提交」`.claude/kb/decisions/08-核心索引结构.md:119`） |

只有第 2 行落在 D18 已定项 8 / 10 的字面里，别的四行 kb 已答或按态别不写。这回答 C109「哪些树的 key 死亡要写墓碑」在 unlink 上的那一格：第一版的树只有 extent、inode、分配记录、记账、映射与队列六棵，逐棵列完只有 extent 那一条线要墓碑（dirent、反向索引这两棵 C109 点名的树今天不存在）。

### 2.2　候选与代价数

| 候选 | 定义 | 多写 | 前置（都现查过） |
|---|---|---|---|
| T0 | 第一版不写墓碑；开一笔「unlink 未写墓碑」的欠账，落地条件 = 扫描重建或容器索引落地 | 0 | 无 |
| T1 | P0 里写 1 条打包记录类型 1 的整对象区间记录到共享墓碑容器（码 3，32 KiB = 2 槽，一份落每块盘） | P0 多 1 个单元（n 加 2），K1 上 P0 从 n = 22 到 n = 24，崩溃状态 4 194 315 → 16 777 227；容器开着期间每次发布都要重写它（D18 已定项 10 射程「且开放容器每次要发布时重写一次」`.claude/kb/decisions/18-块里携带什么信息.md:237`），之后每次发布再 ×4 | ① 记录字段表没定：「56 字节是量级假设」（`.claude/kb/decisions/18-块里携带什么信息.md:237`）；② 容器索引无落点，2026-09-06 收口试过一次判不了案（`.claude/kb/checks-owed.md:98` 末段）；③ 回收门要的统计量 13 / 14 代码里没有；④ 让 I-3.1 的引用集合数到墓碑容器（`crates/singlefs-checker/src/walk.rs` 的引用集合枚举只认识树里的节点，事实表 a3 列了行号，我没逐行核） |
| T1b | T1 + 把墓碑容器登记成「进映射的单元」（映射条目按容器身份），拿映射树当容器索引 | 同 T1 再加映射条目 1 条 55 字节 | 撞 C103 那次收口的失败点：映射的 value 装不下 MAC，加密后解不开（`.claude/kb/checks-owed.md:98`「栽在 value 装不下」）；第一版不开加密时不撞，但那是把 D18 已定项 11 的「容器索引是唯一持有者」改口 |
| T2 | 每批写一条区间记录 | 每批 +1 条，而且第一批之前的状态已经有部分 extent 被释放却还没有墓碑 | 违反 D18 已定项 10 的「对象整个删除写一条」，且每批墓碑都要先于该批的释放落盘，出局 |

**推荐 T0（推），并明写这是对 D18 已定项 8 字面的偏离，需要用户认。** 理由三层：(1) D18 已定项 8 自己的射程说「已定项 8 定的是「盘上留一个墓碑单元」，没有定它什么时候可以被回收——回收规则、粒度与载体归已定项 10」（`.claude/kb/decisions/18-块里携带什么信息.md:204`），载体又卡在 C103；条款要的东西今天没有地方放。(2) 墓碑的读者是扫描重建，而扫描重建不在里程碑三的出口里（「**不在出口**：崩溃放量装置本身」起的那一行，`.claude/kb/milestone/03-third-txn.md:23`，名单里有「扫描重建」），第一版没有读者。(3) 格式还是软的：「在第一个外部用户出现之前，磁盘格式是软的」（`.claude/rules/format-evolution.md:7`），以后补 T1 只算同步成本，不算永久兼容负担；按 `.claude/rules/fs-design.md:129`「⚠️ 第一个外部用户出现之前格式是软的」评估候选时同一口径。
T0 的代价要写清：v1 unlink 之后，一个仍在盘上的数据单元（释放了但槽还没被覆盖）带着完整的五元组，将来的扫描重建看得到它、认不出它已死（D18 已定项 8 引的 E59：「只扫单元时已删的 key 全数复活」的形态）。这个洞今天没有读者，补上的条件写成欠账。

**什么现象会推翻 T0**：(i) 用户把扫描重建（D5 / D18 已定项 5 的级 1）拉进里程碑三的出口——那时 T1 的四个前置要在同一轮里定；(ii) 三·3 的 checker 想判「已删对象不能被扫描复活」——它没有输入；(iii) 有人给出 C103 的收口（容器索引落点），T1 的前置 ②③ 就只剩字段表。

## 3　U3　分批与续做（C1、C2）

**判定**：与 D8 已定项 5 的四条硬要求一致；两处不一致 / 没说：(a) 三·2 验收「同一批续做两次盘上字节相同」字面不成立（3.5）；(b) 续做入口挂在哪一步、取号前预演与准入怎么把它算进去，规则没说（3.4，推）。

### 3.1　分批的粒度与方向（推）

- **方向被今天的不变量定死：只能从尾巴起删。** 数据单元从 0 起连号、没有洞：恢复报「extent 树里第一个文件的单元有洞（第一版不支持）」（`crates/singlefs-core/src/recovery.rs:3374`），读侧 `open_file` 也逐条核第 i 条记录 key 里的文件偏移等于第 i 个单元第一个字节的偏移，不等报 `ExtentRecordKeyIsNotTheFileOffsetOfItsUnit`（`crates/singlefs-core/src/mounted_read.rs:701` 附近，函数从 `crates/singlefs-core/src/mounted_read.rs:617` 起）；从头删会出洞。尾巴起删同时落在 D8 已定项 5 的布局规则上：「**真正的开关不是 K/N，是「受影响的 key 连不连续」**：连续 ⇒ 范围重建可整节点丢弃或重写」（`.claude/kb/decisions/08-核心索引结构.md:133`）。
- **粒度取「数据单元（extent）个数 K」，默认 K = 一片下段叶罩的单元数 144（推）。** 144 来自格式常量「pub const EXTENT_TREE_LOWER_LEAF_DATA_UNITS: u64 = 144;」（`crates/singlefs-format/src/lib.rs:156`），不是魔数（`.claude/rules/fs-design.md:146` 硬要求 1 要求判据「显式、可计算」）。一批的代价随 K 线性：释放前按位置项逐份读盘核校验和（`crates/singlefs-core/src/transaction.rs:3070`「pub fn copies_failing_the_release_checksum_check」），K = 144 时读 144 × 2 份 × 32 KiB = 9 216 KiB（`python3 -c 'print(144*2*32)'` 输出 `9216`）；释放的槽 144 × 2 = 288 个 < 分配记录树一片叶 812 槽，至多脏两片叶；journal 记录一条（重写角色数远小于每条 67 个点名项，`crates/singlefs-format/src/lib.rs:206`）。
- **不按「记录条数」「节点数」分**：那两个量随 extent 数变但不是每批的成本主项；按 extent 数分，测试旋钮（3.3）只有一个整数。
- **分批不是分支，unlink 只有一条路（推）：** 总是「P0 意图发布 + ⌈N ÷ K⌉ 批（N = 0 时 1 批清理）」，不设「够小就一次做完、不写意图」的快路。理由：`.claude/rules/fs-design.md:155` 「**反过来：判据写不出来时，先只做一条，而且先做慢而正确的那条。**」；快路径是以后加的优化，不改格式。代价：一次小文件 unlink 比快路多一次发布（≈ 2^20 – 2^22 个崩溃状态的一次发布）。

### 3.2　每批施加什么：size 跟着 extent 走（推，本节最重要的一个设计选择）

每批把文件尾部的 K 个单元释放，同时把 inode 记录的 size 改成剩下单元数 × 净荷容量（nlink 仍是 0）；最后一批释放剩下全部单元、并在同一发布里删掉 inode 记录与队列条目（D8 已定项 5：「不可能出现「意图没了但目标状态没达成」——**删除意图必须与最后一批工作在同一个事务里提交**。」`.claude/kb/decisions/08-核心索引结构.md:119`）。

这样每个中间态都是一个「合法的、较短的文件」，今天的判定不用改：单元数 = 上取整（size ÷ 净荷）是写侧、读侧、恢复三处同一条除法（第 0 节 l：`crates/singlefs-core/src/write_request_split.rs:61`、`crates/singlefs-core/src/mounted_read.rs:727`、`crates/singlefs-core/src/recovery.rs:3739`）；`blocks` 是按 size 现算的（`crates/singlefs-core/src/records.rs:34` 起的注释：`blocks` 不是字段），自动跟着。对照 D8 已定项 5 第 3 条：「**中间状态必须每一个都是合法的文件系统状态**——「删了一半 extent 的文件」本身就得合法。」（`.claude/kb/decisions/08-核心索引结构.md:123`）与 I-4.6「存在未完成意图的镜像上，其余全部不变量仍然成立」（`.claude/kb/invariants.md:171`）。

| 状态 | inode 记录 | extent | 队列 | 今天的恢复 / 读 / 冷走读 怎么看它（推，按第 0 节的行号） |
|---|---|---|---|---|
| S0 删前 | nlink 1，size = N × 净荷 | N 个单元 | 空 | 通过 |
| S1 P0 之后 | nlink 0，size 不变 | N 个 | {1} | 重建通过（nlink 不读）；冷走读会把它当文件读出来 ⇒ 必须加一句「nlink 为 0 就当没有这个文件」的判定，放在 `crates/singlefs-core/src/recovery.rs:4144` 的 `let Some(inode_record) = inode_record else {` 那一句旁边；`open_file` 同理（`crates/singlefs-core/src/mounted_read.rs:627` 之后） |
| S2..Sm 第 1..m−1 批之后 | nlink 0，size = 剩余 × 净荷 | 剩余单元 | {1} | 重建通过（单元数与 size 相符）；同样要 nlink 检查 |
| S 终（最后一批之后） | 没有记录 | 上段没有 inode 1 的条目 | 空 | 重建拒三处：「extent 树上段里不是恰好一条叶条目」（`crates/singlefs-core/src/recovery.rs:3354`）、「inode 树根没有条目」（`crates/singlefs-core/src/recovery.rs:2452`）、「inode 树里没有第一个文件那条记录」（`crates/singlefs-core/src/recovery.rs:2505`）⇒ 第 5 节 |

对照：**候选 S-b「size 不动、只删 extent」**。S2..Sm 里单元数与 size 对不上，恢复报「E142 走读同款」（`crates/singlefs-core/src/recovery.rs:3747`，在 `data_unit_count_matches_the_inode_size` 里）、读侧报 `ExtentRecordCountDoesNotMatchTheFileSize`（`crates/singlefs-core/src/mounted_read.rs:730`），要在三处（恢复重建、冷走读、`open_file`）加「nlink = 0 免核」的例外，checker 判 I-4.6 时也要带这张例外表——与 I-4.6 的字面「其余全部不变量仍然成立」相抵。S-b 的好处是每批少重写 inode 叶与根 2 个单元：

| | 每批重写单元数 | n | 每批崩溃状态 2^n + 11 |
|---|---|---|---|
| S-a（推荐） | 8 固定 + extent 根 1（剩 1 个单元、内联）或 extent 下段叶 + 根 2 + inode 叶 + inode 根 = 11 – 12 | 22 – 24 | 4 194 315 – 16 777 227 |
| S-b | 8 固定 + extent 1 – 2 = 9 – 10 | 18 – 20 | 262 155 – 1 048 587 |

（n 与状态数都是推的：由第 0 节 s 的式子与「空发布 n = 16」反推，没跑。）S-b 每批少 16 倍状态，但 `.claude/agent-common.md:59` 「**崩溃点测试准确率和正确率优先，其次再衡量时间成本**」，所以取 S-a。

### 3.3　分批必须能被测试强制进入（fs-design 硬要求 2）

- 只供测试的旋钮，抄 `JournalRecordNamedEntryCapacity` 的形态：`enum UnlinkBatchExtents { FromTheLowerLeafCapacity, CappedForTests { extents_per_batch: u64 } }`，放在 `crates/singlefs-core/src/transaction.rs`「pub enum JournalRecordNamedEntryCapacity」（145 行）旁边，装到 `PoolWriter` 上（同「pub fn set_journal_record_named_entry_capacity(」，251 行），带 `branch_name()`（同 171 行那种写法）满足硬要求 4「分支必须可观测」（`.claude/rules/fs-design.md:151`）。产品路径恒 `FromTheLowerLeafCapacity`。
- 固定脚本的删除段至少两批：文件要有 ≥ 2 个单元（1 个单元的文件是内联，一批就做完），K = 1 时 N = 2 得 2 批（第 1 批删单元 1、文件剩 1 个内联单元；第 2 批删单元 0、删记录、删队列条目）。今天造多单元文件的入口只有 `publish_sequential_write`（`crates/singlefs-core/src/transaction.rs:4772`）；固定脚本里现有的文件都不到一个单元：`const THIRD_FILE_BYTES: usize = 2500;`（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:131`）、`later_content` 是「(0..3000 + seed)」字节（同文件 134 行）——**里程碑「到 E 之后加 unlink 段」这句话的前提（一个单元的文件能分两批）不成立，脚本要先多一次两单元的顺序写（第 6 节算这笔账）。**
- 硬要求 5「分支变量不许在操作中途改变」（`.claude/rules/fs-design.md:153`）：K 是写入口的属性、挂载期间常量；「还剩几批」是状态、不进判据。

### 3.4　对 `crates/` 的改动：每批是一次新的发布计划，不能借「覆盖写变短」（推，本节要点）

**今天没有「只截短、不重写数据」的入口**（第 0 节 k）。借 `publish_sequential_write` 用更短的内容表面上能凑：第 j 批用 `N − jK` 个单元的内容，但它把保留的每个单元整个重写一遍（`plan_the_extent_tree_after_this_publish` 在 `crates/singlefs-core/src/transaction.rs:3945` 里「lower_nodes … RewrittenThisPublish」整段重写），N = 1000、K = 144 时 7 批共重写 2976 个数据单元（`python3` 算：856 + 712 + 568 + 424 + 280 + 136，输出 `2976`），崩溃放量的单元写数随保留单元数线性长、状态数指数长，测不了。所以要新计划：

1. `PublishPlan.file`（`crates/singlefs-core/src/transaction.rs:3570`）从 `Option<FileVersionPlan>` 变成能表达「不写内容、只截到 keep_units」的三选一；`inode_record_writes`（4232 行）按它写 nlink = 0 与新 size。
2. `plan_the_extent_tree_after_this_publish`（3945 行）加一支「保留前 keep_units 个单元」：下段落在 keep_units 上的那一片叶 + 下段根 + 上段那一串重写，被整片丢掉的下段叶进换下清单（`ExtentTreePlan.replaced_previous_roles`）。
3. `roles_replaced_via_mapping`（2845 行）里「(rewritten_data_units..previous_data_units)」只在重写了数据单元时才把多出的单元算换下（2854 行 `= if rewritten_data_units == 0`）：加一个显式的「释放 [keep_units, 原单元数)」输入，走同一条映射查落点、读盘核、`release_leaving_the_record_allocated_on` 的路（第 0 节没列这条路的行号，事实表 (b) 有）。
4. `crates/singlefs-core/src/inode_tree.rs` 加「删一条记录、叶空了就把这片叶丢掉、树空了根变零条目」：今天只有原地换与末尾追加（第 0 节 m）。**只做删，不做合并**：D8 已定项 6 写「合并时机是写路径策略」（`.claude/kb/decisions/08-核心索引结构.md:169`，同一行还有「**记录数为 0 的类型 2 容器不落盘**」），不进格式；合并（C116、C536 的对象）留在这一版之外，欠账原样开着——三·5 把它们列成「连带欠账」，我的推荐是这一版不还。
5. `crates/singlefs-core/src/records.rs`：`InodeRecord` 加 `nlink: u32`，`to_bytes` 写它（今天写死 1，56 行）、`parse` 读它（85 行 `reader.skip(8 + 16)` 要拆开）。偏移 24 起 mode / uid / gid / nlink 各 4 字节，nlink 在偏移 36，140 字节的宽度不变。

### 3.5　续做入口：挂在可写挂载的哪一步（规则没说，推）

规则给的约束：「任一含未完成意图的镜像正常挂载后没跑续做即判红」（`.claude/kb/checks-owed.md:17`，C1 的「怎么拦」，后面还有「恢复路径里不许存在跳过续做的分支」）；D8 已定项 5 第 2 条「**续做是每次恢复都跑的常规动作，不是修复手段**」（`.claude/kb/decisions/08-核心索引结构.md:122`）。⇒ 续做必须在 `mount_writable` 里面，不能只是会话上的一个可选方法；只读路径不写盘（3.6）。

| 挂点 | 位置（第 0 节 q） | 好处 | 坏处（推） |
|---|---|---|---|
| M1 暖机循环之后、准入分支之前 | `crates/singlefs-core/src/mount.rs:3940` 循环结束之后、`crates/singlefs-core/src/mount.rs:3736` 的 `let admission_before_acquisition = match start.space_admission {` 结果被用之前 | 暖机保证本实例的根覆盖每块盘（「暖机（D16（发布语义） 已定项 8 戊）：本实例的根覆盖两块盘之前连推空发布」`crates/singlefs-core/src/mount.rs:3931`），批发布是本实例的普通发布 | 池紧、取号前判过「不够」时，批发布可能先被准入拒，而能腾空间的抬 F 还在它后面（4005 行） |
| M2（推荐）暖机与抬 F 都做完之后、交回之前 | `crates/singlefs-core/src/mount.rs:4005` 的 `push_floor_raises_after_the_row_publish(` 之后、`crates/singlefs-core/src/mount.rs:4018` 的 `Ok(Mounted {` 之前 | 抬 F 已经把能回收的 defer 槽回收过，批发布的空间读数最宽 | 批发布本身释放的槽要再抬一次 F 才回收；这条靠「批发布被拒时推抬 F 再判」补（下一条） |
| M3 会话上的一个方法 | `crates/singlefs-core/src/mounted_session.rs:51` 的 `UserChange` 旁边 | 后台一批一批走、测试可在两批之间插崩溃 | 单靠它满足不了 C1（可写挂载后没人调就判红） |

**推荐 M2 + M3 并存（推）：** M2 在 `mount_writable` 内部把队列走到空（v1 单文件，批数有界），M3 给后台与测试一批一批走。两者共用一个函数，这个函数复用「被拒 → 推抬 F 的空发布 → 重判」那一条已有的路：会话那一层的写法见「发布路径报空间准入不够（`PublishError::SpaceAdmissionRefused`）、或准入放行而落点取不到（`PublishError::PlacementRefused`）时」（`crates/singlefs-core/src/mounted_session.rs:3`），抬 F 的函数是 `crates/singlefs-core/src/mount.rs:2286` 的 `pub fn push_one_floor_raise_within_the_admission_budget`。

**取号前预演与准入怎么把续做算进去（推）：**

1. 取号前那一判只管「实例切换的预留拿得到」，这一刻需求记 0（`crates/singlefs-core/src/mount.rs:3736` 起那一段注释：「写行与暖机那一串的空间就在切换预留与 checkpoint 保留池里」）；续做批不在切换预留里（切换预留的式子是 (N_switch + 1) × R × c_max，只数写行与暖机），所以**不改这一判**，批发布在自己发布时按普通发布判（第 0 节 j：extent / inode 角色属「普通分配」）。
2. 取号前预演（`crates/singlefs-core/src/mount.rs:2874` 的 `dry_run_of_the_publishes_after_acquisition`）今天只演写行 + 暖机。它存在的理由是「释放核验、树的形状、落点取不到，都在任何写之前返回」（`crates/singlefs-core/src/mount.rs:3765`）。续做批的形状（每批重写哪些角色、释放哪些落点）只由上一版与 K 决定，取号前算得出来（同 `warm_up_publish_txgs` 的做法，`mount.rs:3708`），所以**应当加进预演**：否则续做批在取号之后才发现落点取不到，挂载报错时实例行已经写了（`crates/singlefs-core/src/mount.rs:3128` 的 `publish_failed_after_acquisition` 就是这一类错的出口），取号前「一个写都没发」的保证就破了。预演要多一个参数（续做计划），演完整串批发布。
3. 池紧到批发布落点都取不到时：按 D3 的口径这是真 ENOSPC，不是破例：「它不承诺「释放一定立刻成功」——被 defer 窗口与回退下界钉住的那几代按 D16（发布语义） 已定项 1 的口径不算空闲，那是真 ENOSPC 不是破例。」（`.claude/kb/decisions/03-空间分配.md:306`）；但「释放本身不申请空间」要靠写入时预付（第 4 节候选 R3）。

### 3.6　只读挂载看到进行中的删除时读到什么

答案：**没有这个文件**，从 S1（P0 落盘）起就是。依据：D8 已定项 6 定 inode 记录「在回收完成前留在 inode 树里（nlink = 0）」（`.claude/kb/decisions/08-核心索引结构.md:171`），所以「文件在不在」不能看记录在不在，要看 nlink。今天两处读者都不读 nlink（第 0 节 f）：`open_pool_for_read` 把树里全部记录收进挂载态（`crates/singlefs-core/src/mounted_read.rs:449` 起），`open_file` 按 inode 号找记录（`crates/singlefs-core/src/mounted_read.rs:627`）；冷走读拿到记录就读内容（`crates/singlefs-core/src/recovery.rs:4144` 起）。改法：`open_file` 遇到 nlink = 0 报一个新的 `OpenFileFailure` 成员（与 `NoSuchInode` 分开，让测试分得出「从没有过」和「删除中」），冷走读遇到 nlink = 0 交回 `Ok(None)`。只读挂载看到的中间态（S2..Sm）extent 与 size 自洽，走不到读内容那一步。

### 3.7　幂等：三·2 验收「同一批续做两次盘上字节相同」字面不成立（冲突，两边整行并排）

左边 `.claude/kb/milestone/03-third-txn.md:84`（三·2 的验收格所在行）：

> | 三·2 | 实现 core：`unlink` 单文件（无目录，按 inode 号）——意图写入 → 分批释放 extent（走 D19（块指针的结构与宽度预算） 已定项 5 硬规则 1 的释放判定）→ inode 记录 nlink = 0 留树、`deleted_inodes` 加条目 → 最后一批与删意图同一事务；每次可写挂载先续做；`crates/singlefs-harness/src/model.rs` 加删除操作与删后读到「没有文件」的答案 | D8（核心索引结构） 已定项 5 的四条硬要求 | harness 用例：删后读回「没有文件」、崩在每两批之间的中间态 checker 全绿、同一批续做两次盘上字节相同（幂等）、删除意图不与最后一批同事务时用例红；变异行；代码三方一轮 | 三·1 |

右边 `.claude/kb/decisions/08-核心索引结构.md:203`（D8 已定项 6 的射程段）：

> **射程**：**唯一性的射程是一条已发布历史内**——每个容器号都是某条记录首次插入时的 inode 号；号在崩溃恢复落到较旧的根之后可以重发（管理员回退那次发布的水位取 max、不重发，D23（journal 的角色与格式） 已定项 14），所以同一棵树里两个容器可以同号，分开它们的是容器出生代（右半的出生代 = 触发分裂那次发布的 txg，而恢复之后新实例的每一个 txg 都大于环里读得出的每一条根与记录的 txg，D23（journal 的角色与格式） 已定项 14 注 3），同一次发布内「分裂 → 合并 → 再分裂」也造不出同身份两版。跨崩溃不成立：崩溃后 checkpoint 号会被重发，被抛弃那版与重试那版四段逐字相同、内容不同、都在盘上——这与数据单元的五元组、码 2 节点的 (树, 层级, 诞生代号) 是同一种暴露（根记录靠 D16（发布语义） 骑手条款 1 的实例代号分辨，别的单元类靠类身份段里的写序 (实例代号, 事务号) 分辨——容器不单独加实例代号，写序里那 4 字节就是它），常规路径由 D3（空间分配） 已定项 1 的谓词（分配记录）判，扫描重建路径按 C113（扫描重建时多版单元的现行版本判定无输入） 定案的 P2 / P3 规则判。⚠️ **判它安全的依据与树 ID 那条同源**：孤儿与活版的实例代号必不相等（D18（块里携带什么信息） 已定项 11「作废一个 checkpoint 必须换实例代号」），扫描重建的两道闸按写序择新。⚠️ **残留的疑点**：讲统计量合并规则那一句「『未发布的等于没发生』只对常规挂载路径成立，**扫描重建看的是盘**」读起来像是反方向的——它管的是「水位在一个代段里取最大值而不是后者胜」，与「未发布过的号可不可以重发」不是同一个命题，但两者挨得很近。⚠️ **重建的射程是单头镜像**：有克隆头时共享叶头里的出生树是 origin（I-1.3（块头树 ID 一致） 的 C110（跨头共享与加密后元数据类的块头一致性读法） 那一格），按出生树认领会把克隆头的树认残——多头镜像的归属划分要克隆祖先表（E90（树 ID 进 AAD 与跨头共享） 第二笔），落地前多头镜像的 inode 树重建没有输入。⚠️ 填充率没有下界：`⌈n/233⌉` 是新建树的值，1e6 顺序创建再删 90% 不合并是 9.98 倍空间放大；加宽记录能同时缩小爆炸半径（160 ⇒ 204 条），这笔取舍归 C100（inode 容器丢失是永久损失而没有任何冗余条款覆盖它）。⚠️ **挡着「新池新建文件写」的九样，① ② 已定案，其余照录**：① 树表条目已由已定项 8 定；② 记账条目的 seq 住 value、宽 4 字节（已定项 7）；③ 统计量标签值（C71（不认识的统计量标签静默忽略））；④ 不带设备维的统计量的设备段取值、新建树的记账行初始化、读现行值的读法（C117（不带设备维的统计量的设备段没有取值））；⑤ 码 3 容器带不带扩展点（C108（扩展点归属对打包记录单元无判定））；⑥ 码 2 节点头的基础字节数（已由已定项 11 收口）；⑦ `deleted_inodes` 的形态（C118（`deleted_inodes` 树的形态无落点））；⑧ ⑨ 记录格式与统计量登记表两个冻结组件的 spec 文件（C119（冻结组件没有 spec 文件））。⚠️ 跨头首次 COW 时容器身份要不要重生的另一半归 E90（树 ID 进 AAD 与跨头共享） 第二笔。⚠️ **`blocks` 不承担对外的 `st_blocks`**：stat 报的 `st_blocks` 怎么给（stat 时从 extent 树现算分到的单元）、SEEK_HOLE / SEEK_DATA，挪到做 POSIX 面的里程碑。`blocks` 按文件长度向上取整改新池新建文件的字节：那条记录偏移 48 的 8 字节从 64 变 6。

推导：续做从盘上持久态重算这一批（`ResolvedPublish` 是只读的纯计算：「只读——算它不发一个写、不动分配器，所以条款没写的那几格（`crate::inode_tree`）在这里交回时盘上逐字节不变。」`crates/singlefs-core/src/transaction.rs:3665`），同样的输入得同样的「重写哪些角色、释放哪些落点」。但每次可写挂载先取一个新实例代号（`crates/singlefs-core/src/transaction.rs:797` 的 `acquire_expected_instance`），码 2 节点头里写着实例代号（「pub instance: InstanceGeneration,」`crates/singlefs-core/src/unit.rs:29`；恢复按它核：「if node.birth_txg > root.checkpoint_txg || node.instance > root.instance {」`crates/singlefs-core/src/recovery.rs:3302`），崩溃在一批中间、重新挂载后重做这一批，写出的单元头里实例代号与写序不同——右边那句「被抛弃那版与重试那版四段逐字相同、内容不同、都在盘上」（`.claude/kb/decisions/08-核心索引结构.md:203`）说的就是这一格。落点也可能不同（旧的半截单元占着的槽被谁重用取决于分配器）。⇒ **字节相同是测不出来的，也不是幂等的字面**；D8 已定项 5 的幂等字面是「已做完的部分被幂等地重跑」和「续做一遍发现无事可做，然后删掉意图」（`.claude/kb/decisions/08-核心索引结构.md:117`）。

可测的三条替代（推，都不是字节相等）：(E1) 计划相等：同一份（上一版、队列、K）算两次，重写角色清单与释放落点集合逐项相等（纯函数，单测判）；(E2) 已完成的删除再续做一次，一个写都不发；(E3) 在一批的每一个写点截断、重新挂载续做到底，终态的逻辑内容（inode 集合、extent、队列、账）与不崩溃的终态相同，并且每个崩溃点的镜像 checker 全绿。落点与单元头的字节不比。

**什么现象会推翻 U3 的推荐**：(i) 量出 S-a 每批多重写的 inode 叶 + 根让崩溃放量超出预算而用户不认「准确率优先」——那时退到 S-b，并要在三处加 nlink = 0 的免核例外；(ii) 主 agent 认为续做可以只在会话层、不进 `mount_writable`——那要改 C1 的验收字面；(iii) 预演不加续做批而接受「取号后才发现落点取不到」——要改 mount 的「取号前一个写都不发」这条保证。

（以下为接续段：上一条正推腿撞限额停在第 3 节末尾，报告第 1–3 节原样不动；第 4 节起由接续腿追加。接续时现查：第 0–3 节 92 处引文 `cite-check.py` 全对，sha256 19791cbe067c58ce…。）

## 4　U4　空间：可用空间扣待删占用（I-3.4，C375）

**判定**：冲突（一格）+ 规则没说（一格）。冲突：I-3.4 与 D28 式子要求「扣待删占用」，而实现里「已分配」直到回收才减（第 0 节 g），任何「待删占用 = 尚未释放的 extent 字节」的读法都与「已分配」重复扣；规则没说：待删占用该记什么才不重复、又对续做有用，kb 没有一句给量。两边整行并排：

左（条款）`.claude/kb/decisions/28-挂载期承诺量.md:19`：

> 可用(d) = 容量(d) − 已分配(d) − 不可回收(d) − 挂载期承诺量(d) − 被抛弃根独占量(d) − 待删占用 ÷ 副本数 − 已承诺预留 ÷ 副本数 − checkpoint 保留池 ÷ 副本数

左（不变量）`.claude/kb/invariants.md:139`：

> | I-3.4 | 可用空间扣待删占用 | 可用空间统计已扣除「待删除但意图未完成」占用的空间 | 未实现 |

右（实现：释放只加 defer、不减已分配）`crates/singlefs-core/src/allocator.rs:556`：

> self.deferred_slots += span;

右（实现：已分配到回收才减）`crates/singlefs-core/src/allocator.rs:673`：

> self.allocated_slots -= span;

右（实现：记账行注释，defer 里的仍算在已分配）`crates/singlefs-core/src/transaction.rs:6303`：

> // 第 5 项：已释放、还在 defer 窗口里的（它们仍算在已分配里：占着空间、被根环里的有效根引用）。

右（实现：待删占用今天是个常量）`crates/singlefs-core/src/admission.rs:149`：

> /// 式子里的「待删占用」（记账第 4 项）：第一版不删对象、不销毁快照，没有「待删除但意图未完成」的结构 ⇒ 恒 0。

同型先例（kb 已处置过一次「重复扣」）`.claude/kb/decisions/28-挂载期承诺量.md:60`：

> - 删「defer 待释放」、需求改成新写的全部槽：用户 2026-09-25 定案（原话在变更史），安全设计轮第二轮判决 `research/prompts/m2-safety-r2-main-verification.md`（「defer 待释放」与「已分配」重复扣）、第三轮判决 `research/prompts/m2-safety-r3-main-verification.md`。

### 4.1　待删占用的意图是什么（条款怎么读，推）

- D5 已定项 20 把「待删除结构（意图 key、删除中队列）」定成算作活引用，射程里说它的用户可见后果是「`rm` 返回之后建的快照会**钉住**那批尚未释放的 extent，`df` 要等快照死了才回来，部分覆盖它的是 I-3.4（可用空间扣待删占用） 与 I-4.7（删除中 inode 有账） 两条不变量」（`.claude/kb/decisions/05-快照-空间记账机制.md:429`）。⇒ I-3.4 的字面意图是「还没回收的删除占着的空间不算可用」。这一句在实现里已经由「已分配」满足：extent 在 P0 之后、每批释放之前，是已分配、没进 defer；释放之后进 defer、仍在已分配里（第 0 节 g、h）。所以待删占用**不能**再记这些字节。
- D3 已定项 17 与已定项 2 给了另一个读法：删除路径「要写盘」，它的空间必须提前预付：「**定案**：COW 系统必须保证「释放空间」这个操作本身不需要申请空间。」（`.claude/kb/decisions/03-空间分配.md:304`）；「墓碑就是「解开自己」的一部分」（`.claude/kb/decisions/03-空间分配.md:52`，同一行）。unlink 的每一批（第 3 节）要新写 11–12 个单元（第 3.2 节表，推的），这笔钱要有人预付。
- 两个读法不冲突，它们在两个不同的量上：前者已经在「已分配」里；后者是「还要新写的」，不在任何一项里（已承诺预留今天恒 0：`crates/singlefs-core/src/admission.rs:156`「pub const COMMITTED_RESERVATION_OF_THE_FIRST_VERSION: BytesSummedOverAllReplicas =」）。

### 4.2　候选（每批逐格值在 4.3）

| 候选 | 待删占用记什么 | 与已分配的关系 | 对续做的保证 | 出局 / 收 |
|---|---|---|---|---|
| R0 | 恒 0（今天的常量），删除占的空间全靠「已分配」 | 不重复 | 无：P0 放行不保证后面各批放行，池紧时挂载续做取不到落点（第 3.5 节：`mount_writable` 里续做失败是挂载失败） | I-3.4 没有东西可判（值恒 0）；收作「最小改动」对照 |
| R1 | 尚未释放的 extent 字节（按全部副本之和） | **重复扣**：这些槽已经在已分配里，可用比分配器发得出的槽数少这一份 | 无 | 出局：量在 4.3 表；与 D28 已定项 1 处置「defer 待释放」是同一种错 |
| R2（推荐） | 完成这个删除还要新写的最坏字节：剩余批数 × 每批最坏需求 × 副本数，P0 起立、每批递减、最后一批清零 | 不重复：还没写的不在已分配里；每批写出来时同一份钱从待删占用转进已分配 | 有：P0 放行时已把整串批的钱扣住，后面每批必放行 | 见 4.4 的三个实现约束 |
| R3 | 写入时预付：每次写文件就按 ⌈N ÷ K⌉ × 每批最坏需求进「已承诺预留」（记账第 6 项），unlink 的 P0 把它转成待删占用 | 同 R2 | 有，且满盘上 unlink 也不会因空间被拒（合 D3 已定项 17 的字面） | 代价：每个文件写时多预留（1 GiB 文件 ⌈2^30 ÷ 32634⌉ = 32 903 个数据单元，⌈32903 ÷ 144⌉ = 229 批 × 12 槽 = 2748 槽 = 42.9 MiB 每盘，`python3` 算，输出 `32903 229 2748 42.9375 0.041666666666666664`；每 144 个数据单元（144 × 32 KiB 每盘）常驻 12 槽 = 192 KiB，≈ 4.2%，永不删的文件白占；数据单元净荷 32 634 字节见 `crates/singlefs-harness/tests/sequential_write_split_into_one_unit_transactions.rs` 文件头，`DATA_UNIT_BYTES` = 32 768 在 `crates/singlefs-format/src/lib.rs:20`） |

### 4.3　三批删除逐格（推的：由第 3.2 节表的单元数推，没跑）

设定（每盘的槽数，1 槽 = 16 384 字节，`crates/singlefs-format/src/lib.rs:14` 的 `SLOT_BYTES`）：两块盘、两份副本；每盘容量 4096 槽；式子里除「已分配」「待删占用」之外的各项（挂载期承诺量、checkpoint 保留池等）合计 100 槽，删除期间不变；删前已分配 300 槽（含文件的 3 个数据单元）；文件 N = 3 个单元、K = 1（测试旋钮），所以 P0 + 3 批；P0 新写 11 槽（其中队列根 1 槽是新建、不换下旧的），三批各新写 12 槽（最后一批同）；每次发布换下的旧节点槽 10、12、12、12，每批另释放 1 个数据单元（数据单元 32 KiB = 每盘 2 槽）；期间不抬 F（不回收），最后一行是抬 F 回收全部 defer。「可用」= 容量 − 已分配 − 100 − 待删占用÷副本数，四种口径的值同表（`python3 /tmp/claude-1000/m3-delete-r1-sonnet-2/u4.py` 的输出，逐行核过）：

| 时刻 | 已分配 | defer | 可用·R0 | 可用·R1 | 待删占用（R2，每盘份额） | 可用·R2 |
|---|---|---|---|---|---|---|
| t0 删前 | 300 | 0 | 3696 | 3696 | 0 | 3696 |
| P0 之后 | 311 | 10 | 3685 | 3679 | 36 | 3649 |
| 第 1 批之后 | 323 | 24 | 3673 | 3669 | 24 | 3649 |
| 第 2 批之后 | 335 | 38 | 3661 | 3659 | 12 | 3649 |
| 第 3 批之后 | 347 | 52 | 3649 | 3649 | 0 | 3649 |
| 抬 F 回收全部 defer 之后 | 295 | 0 | 3701 | 3701 | 0 | 3701 |

- **R1 扣两次的量**：P0 之后 6 槽、第 1 批之后 4 槽、第 2 批之后 2 槽（每盘）= 尚未释放的数据单元数 × 2 槽，恰好是那几个单元在「已分配」里已经占着的槽；即 P0 之后每盘 6 × 16 384 = 98 304 字节被扣两次（全池按副本之和记 196 608 字节，`python3` 算，输出 `98304 196608`）。判据「量出扣两次 ⇒ 待删占用的口径必须改」（正文第五节）由攻方在副本上量；我这一行是推的，不是量的。
- **R2 的形状**：P0 一次扣住 47 槽（P0 自己 11 + 后面三批 36），之后每批「已分配 +12、待删占用 −12」互相抵消，可用恒 3649，直到回收。P0 的准入需求 = 11 + 36 = 47 槽每盘（普通版 P0 只需 11）。每盘份额 36 槽 = 589 824 字节，记账行里记的是全部副本之和 1 179 648 字节（`python3` 算，输出 `589824 1179648`）。
- 最后一行的 295 = 347 − 52（defer 52 槽 = 10 + 14 + 14 + 14，每次发布换下的旧节点槽加每批释放的一个数据单元 2 槽），可用 3701；第 3 批我按 12 槽保守取，按 5.2 的形态最后一批实际 11 槽（8 固定 + extent 根 + inode 根 + 队列根，叶不落盘），差 1 槽不改表里其余各行的结论。
- 表里 R0 的可用在删除期间单调降到 3649，与 R2 终值相同；差别在中途：R0 在 P0 时以为还有 3685 槽，实际后面三批要吃 36 槽，池紧时后面某批被拒。

### 4.4　推荐 R2，与它的三个实现约束（推）

推荐 R2：I-3.4 因此有东西可判，续做「一旦意图落盘、后面每批必放行」（C1 要求续做是常规动作，挂载时续做因空间失败会让挂载失败，R0 留着这个洞）。R2 的代价数：记账行数不变（第 4 项本来每次发布都写，`crates/singlefs-core/src/transaction.rs:6284` 换掉那个常量），多写 0 个单元；P0 的准入需求从 11 涨到 11 + 36 = 47 槽（4.3 的设定）；池紧到放不下整串批时 unlink 在 P0 就被拒（真 ENOSPC，回到会话推抬 F 再判的老路，`crates/singlefs-core/src/mounted_session.rs:3`）。

**这一步偏离 D3 已定项 17 的字面，需要用户认**：那条是「COW 系统必须保证「释放空间」这个操作本身不需要申请空间」（`.claude/kb/decisions/03-空间分配.md:304`），R2 在满盘上 unlink 会被 P0 拒（删不动）；只有 R3（写入时预付进已承诺预留）守得住字面，代价是每个文件常驻约 4.2% 的额外预留（4.2 表）、每次写多一份预留的维护、以及永不删的文件白占。我的取舍：v1 单文件、盘容量远大于文件，先收 R2，把「满盘 unlink 会被拒」记成一笔欠账（落地条件 = 出现多文件 / 满盘用例，那时补 R3）；这与 U2 的 T0 是同一种取法（格式软、以后补只是同步成本，`.claude/rules/format-evolution.md:7`）。用户若要 D17 字面，改 R3，其余各格不变。

三个实现约束：
1. **批自己的需求要与待删占用抵消，否则反向重复扣。** `admission_reading_before_a_publish`（`crates/singlefs-core/src/admission.rs:1031`）读的是「发布之前」的数：第 j 批放行时读数里已经扣着待删占用（含本批自己那 12 槽），批的需求又是 12 槽，两处是同一笔钱。数：P0 恰好在可用 = 47 槽时放行（P0 自己 11 + 扣住 36，之后可用 = 0），第 1 批读数里可用 = 0 < 需求 12，被拒，虽然它已被预付。解法二选一：批的读数用「本批之后的待删占用」（24）；或把预付过的批标成不判（同「没有普通分配的发布不判」那条已有的分支：`crates/singlefs-core/src/admission.rs:943` `publish_has_an_ordinary_allocation`，条件里加一支「预付批」）。
2. **K 不进待删占用的式子。** 批数依赖 K，K 是测试旋钮（第 3.3 节）；把待删占用定成 ⌈剩余单元 ÷ 144⌉ × W × 副本数 × 16 384（144 取格式常量 `crates/singlefs-format/src/lib.rs:156`，W = 每批最坏需求 = `demand_of_the_roles_on_each_device`（`crates/singlefs-core/src/admission.rs:958`）对最坏角色清单的值，我估 12 槽），这样 I-3.4 在 checker 里能只从镜像重算（`记账第 4 项 == Σ 队列里各 inode 的式子`，不需要知道旋钮）。代价：旋钮 K < 144 的测试里预付不够真实批数，那些用例里「续做必放行」的保证不成立；测试要么池够大，要么单测「预付够不够」只在 K = 144 下判。
3. **挂载后头一次准入就要有这个数。** 今天全部读数经 `PoolWideCommitments::of_the_first_version` 构造（`grep -rn 'PoolWideCommitments::of_the_first_version' crates --include=*.rs` 现查 5 处：`admission.rs` 3 处、`crates/singlefs-harness/tests` 2 处），它把待删占用写死常量。改成由「上一版」（`admission_reading_with_the_switch_reserve_of` 已经收 `version_to_build_on: Option<&TransactionOutput>`）的队列与各文件单元数算，同时 `transaction.rs:6284` 那一行也用同一个函数，「一处定义两处引用」的既有做法不变。

checker：I-3.4 落成「记账第 4 项 == 队列里各 inode 的 ⌈剩余单元 ÷ 144⌉ × W × 副本数 × 16 384 之和」，判别力自证 = 把第 4 项改回 0，P0 之后的镜像必须红（三·3 的变异行）。C375 的单位一格（按全部副本之和记）R2 下已经自洽：4.3 表里全池记 1 179 648、每盘份额 589 824。

**什么现象会推翻 R2**：(i) 攻方在副本上量出每批最坏需求 W 大于 12（预付不够，续做仍会被拒）；(ii) 用户要 D17 字面 ⇒ R3；(iii) 用户认「续做在挂载里因空间失败可以降级为只读挂载」——那 R0 够用，I-3.4 就判「恒 0 且没有未释放的待删结构」；(iv) 攻方量出 R1 并没有扣两次（例如某处「已分配」在 P0 就减了）——那 4.3 表 R1 一栏作废，今天的读法（第 0 节 g）也要改。**推翻 R1 的出局**只需一个现象：批之间任何一刻分配器的「已分配」小于「未释放 extent 槽 + 其余现行槽」，第 0 节 g 的单测 `crates/singlefs-core/src/allocator.rs:2020` 就是在钉它的反面。

## 5　U5　恢复与模型的前置

**判定**：冲突（三处，逐行并排）+ 规则没说（「删完之后的一版长什么样」「再建文件」kb 没定）。恢复今天把「没有文件」只等同于树表 0 条；删掉 inode 1 之后的一版树表不是 0 条，被今天的重建当坏镜像拒（第 0 节 a、b）。攻方要在副本上核这一句；我这里只做静态推：三个 `UnitMalformed` 都在 `rebuild_version_in_the_unit_area_starting_at` 那条路上，没有别的出口先接住空树。

### 5.1　两边并排

左（D8 已定项 6 已给出的删空形态）`.claude/kb/decisions/08-核心索引结构.md:169`（同一行的一句）：

> **记录数为 0 的类型 2 容器不落盘**（发布前必须合并掉；唯一的叶被删空时是不落盘、根变成零条目）。

左（不变量，已实现）`.claude/kb/invariants.md:287`（整行）：

> | I-9.13 | 类型 2 容器不空 | 任一打包记录类型 2 的容器记录数 ≥ 1（空叶在重建时没有 key 范围；唯一的叶被删空时不落盘、根变成零条目） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker/src/walk.rs`……

（该行后半段较长，只抄到这里，行号与前半句由 `grep -nF` 命中；不作为整行引文。）

右（恢复：inode 树根必须有条目）`crates/singlefs-core/src/recovery.rs:2450` 与 `:2452`：

> if inode_root_node.entries.is_empty() {
> what: "inode 树根没有条目",

右（恢复：inode 1 那条记录必须在）`crates/singlefs-core/src/recovery.rs:2502`：

> .find(|inode_record| inode_record.inode == FIRST_INODE_NUMBER)

右（恢复：extent 树上段恰好一条叶条目）`crates/singlefs-core/src/recovery.rs:3354`：

> what: "extent 树上段里不是恰好一条叶条目（第一版只有第一个文件有内容）",

右（恢复：inode 1 必须有单元）`crates/singlefs-core/src/recovery.rs:3365`：

> what: "extent 树里第一个文件一个数据单元都没有",

右（extent 读：零条目的节点一律拒，包括根）`crates/singlefs-core/src/extent_tree.rs:664`：

> "extent 树节点一条条目都没有（缺席的一段不写节点）",

右（只读挂载：树表里这种树的根指针非空才认）`crates/singlefs-core/src/mounted_read.rs:414`：

> .find(|entry| entry.kind == kind && entry.root != NodePointer::empty_root())

- 反面一句：inode 树那一边 kb 已经写好了删空形态（左两行，I-9.13 的检查是「已实现」），实现这一边的 `crates/singlefs-core/src/recovery.rs:2450` 与 `mounted_read.rs` 的循环里前者拒、后者天然放行（「for entry_bytes in &inode_root.entries {」在零条目时一次都不走，`crates/singlefs-core/src/mounted_read.rs:450`；「天然放行」是我读循环推的，没跑）。extent 树这一边 kb 没写删空形态。

### 5.2　删完之后那一版应该长什么样（推）

| 结构 | 最后一批之后 | 依据 |
|---|---|---|
| 树表 | 8 条（K1 之后；不回到 0 条），extent、inode、队列各有一个根节点，其余照旧 | 回到 0 条会把记账树一起丢掉，而 inode 号水位（记账第 12 项，`crates/singlefs-core/src/records.rs:23`）住在记账树里：水位一丢，号会重发，违反 D8 已定项 6「已发布的 (inode 号, 对象出生代) 不复用」（`.claude/kb/decisions/08-核心索引结构.md:171`）；今天 `publish_first_file` 也只认树表 0 条（`crates/singlefs-core/src/transaction.rs:4640`） |
| inode 树 | 根节点 0 条目，叶不落盘 | D8 已定项 6 与 I-9.13（左两行）；inode 树写入口要加「删一条」（第 3.4 节第 4 点） |
| extent 树 | 根节点 0 条目（候选 E-c，推荐）或树表条目的根指针零（候选 E-b） | 见 5.3 |
| `deleted_inodes` | 根节点 0 条目 | 同一条规则「曾经非空的树，删空之后是零条目根」，一条规则罩三棵 |
| 记账 | inode 号水位仍是 2，已分配等照常 | 第 0 节 c2 / 事实表 (c2)：水位只加不减，删除不动它 |

中间态 S1..Sm（第 3.2 节表）不需要新形态：inode 记录 nlink = 0、extent 剩余单元、队列 {1}，重建今天就认（第 3.2 节表 S1、S2..Sm 两行）；要加的只有两句 nlink 判定（重建放行、冷走读 / `open_file` 当没有这个文件，第 3.6 节）。

### 5.3　extent 树删空的两个候选（各给代价数）

| 候选 | 形态 | 最后一批单元数 / n / 崩溃状态 | 要动的读者 |
|---|---|---|---|
| E-c（推荐） | 根节点 0 条目：放宽 `crates/singlefs-core/src/extent_tree.rs:664` 那句拒，只对根放行 | 8 固定 + extent 根 + inode 根 + 队列根 = 11 个单元；n = 22；2^22 + 11 = 4 194 315（`python3` 算） | 只有 `extent_tree.rs` 这一处（根位置 0 条目）与规划那一支（`plan_the_extent_tree_after_this_publish`，`crates/singlefs-core/src/transaction.rs:3945`）；`mounted_read.rs:414` 的「根非空」、重建、checker 走读都天然继续认它 |
| E-b | 树表条目的根指针置零（同 livelist 那三棵的写法） | 少写一个 extent 根：10 个单元；n = 20；2^20 + 11 = 1 048 587 | 三处读者要加「零根 = 空树」分支：`mounted_read.rs:414`（今天零根直接 `TreeTableHasNoTreeOfKind`，`.ok_or(OpenPoolForReadFailure::TreeTableHasNoTreeOfKind { kind })` 在 416 行）、恢复重建与冷走读读 extent 那一段、checker 的 extent 走读；还要把「零根」的语义写进 D8 已定项 9 |

E-c 每次删空多 1 个单元（16 KiB × 2 份），换掉三处读者的分支；E-b 反之。取 E-c 是因为一条规则（删空 = 零条目根）与 D8 已定项 6 现成的写法同形，且不引入「同一种树有时零根有时非零根」的第二种空态。

### 5.4　恢复的改动清单（对 `crates/` 今天的实现，推）

| 位置 | 改成什么 |
|---|---|
| `crates/singlefs-core/src/recovery.rs:2450` | inode 树根 0 条目不再拒；有 0 个记录时那一版没有文件（与 extent 树 0 条目、队列 0 条目一并核） |
| `crates/singlefs-core/src/recovery.rs:2502` | inode 1 记录不在：不再报，改成「没有文件」，并核 extent 树里也没有任何条目（有主无 inode 的 extent 是坏镜像：对应 I-4.7 第二半「其 extent 已全部释放」，`.claude/kb/invariants.md:172`） |
| `crates/singlefs-core/src/recovery.rs:3354`、`:3365` | 上段叶条目个数 0 或 1；0 条时不走单元连号的核 |
| `crates/singlefs-core/src/transaction.rs`：`TransactionOutput.inode_record`（`transaction.rs:2501`）与 `data_pointers`（`transaction.rs:2482`） | 变成「这一版有没有文件」的一个 `Option`；`grep -rn '\.inode_record\b' crates --include=*.rs` 16 处（core 8 处：`mounted_read.rs` 5、`mount.rs` 1、`transaction.rs` 2；harness 测试 8 处），逐处加臂 |
| `crates/singlefs-core/src/mounted_read.rs:627` 之后、`recovery.rs:4144` 那句 | nlink = 0 当没有这个文件（第 3.6 节） |
| checker | 按 5.2 表走：inode 根零条目 + extent 根零条目 + 队列零条目 + 记账水位 ≥ 2 是合法的「没有文件」；I-4.7 两半各一个坏镜像（有 extent 无队列条目、有队列条目无 nlink = 0 记录） |

推翻的代价说明：这一步把「没有文件」由「树表 0 条」扩成「树表 0 条」或「树表 8 条且三棵零条目根」两种，模型与观测侧都要分开（5.6）。

### 5.5　删完之后能不能再建文件

`publish_first_file` 今天的前置是那一版树表恰好 0 条：不满足报 `FirstFileVersionOnAVersionThatAlreadyHasAFile`（`crates/singlefs-core/src/transaction.rs:4640` 与 `:4642`）；删完之后树表 8 条，这条前置会拒。要「再建带数据的文件」得把「文件是 inode 1」这个前提泛化：`FIRST_INODE_NUMBER`（`crates/singlefs-core/src/transaction.rs:109`）在 core 里被 `recovery.rs` 用 4 处（2502、3357、3776、4139）、`transaction.rs` 用 11 处（`grep -rn 'FIRST_INODE_NUMBER' crates/singlefs-core/src crates/singlefs-checker/src --include=*.rs` 除去 `transaction.rs:109` 的定义与 `recovery.rs` 的一条 `use`），还有 `crates/singlefs-harness/src/model.rs` 的单文件假设。

| 候选 | 内容 | 代价 |
|---|---|---|
| X1（推荐 v1） | 不做带数据的再建；`publish_first_file` 前置不改；删完之后能做的写只有已有的 `publish_new_inodes`（`crates/singlefs-core/src/transaction.rs:4830`，size 0、无 extent，号取水位 = 2）、写行与空发布 | 0 处泛化；C479「号不重发」有观测（新发的号是 2、不是 1，`crates/singlefs-core/src/transaction.rs:6269` 的水位 `FIRST_INODE_NUMBER + 1` 只在没有上一版时用）；三·4 里程碑句「（或新流 mkfs → A → unlink → 再建第一个文件版本，推的，三·1 定）」取前半：固定脚本末尾加 unlink 段，再接一次 `publish_new_inodes` 观测发号 |
| X2 | 泛化到任意 inode：`publish_first_file` 前置改成「这一版没有文件」，新文件取水位号；恢复 / extent 规划 / 模型全跟着泛化 | 15 处 core 泛化点加模型加检查；每个泛化点是一个新的崩溃 / 读回分支；里程碑三只要求单文件 unlink，这些是多文件的工作 |

推荐 X1，把「删完之后再建带数据的文件」记欠账，落地条件 = 多文件里程碑；理由同 U2：`.claude/rules/fs-design.md:155` 「**反过来：判据写不出来时，先只做一条，而且先做慢而正确的那条。**」。

### 5.6　理想模型（`crates/singlefs-harness/src/model.rs`）的删除操作与读回答案（推）

- 今天 `ModelRoot::file: Option<ModelFileVersion>` 的注释是「树表 0 条（第 0 代根、暖机根、零单元发布）时没有文件」（`crates/singlefs-harness/src/model.rs:199`），`None` 同时承担「没有树」与「没有文件」。删完之后是「有树、没有文件」，`Option` 装不下第三态：`ModelRoot` 要多一个「树表有几条 / 有没有树」的量（三值：无树、有树有文件、有树无文件），`ModelPublishKind` 里 `RowsOnVersionWithoutFile` / `ZeroUnit` 按「树表 0 条」选角色表（第 0 节事实表 (g1)），要加「有树无文件」的一支。
- 新操作 `Unlink { inode }`：模型里一个操作对应盘上 1 + ⌈N ÷ K⌉ 个根（P0 + 各批）；模型只装得下一个数据单元的内容（`ContentExceedsDataUnitPayload`，`crates/singlefs-harness/src/model.rs:288`），N = 1，所以模型里 unlink = 2 个根（P0 + 一批）。**多批只在 core 级用例里测**，模型对拍到不了多批（要么让模型支持多单元，要么接受这个缺口，交主 agent）。
- 读回答案：`ModelReadBack::NoFile { root }` 已有（`crates/singlefs-harness/src/model.rs:1632`），语义从 P0 那个根起就是 `NoFile`（3.6：nlink = 0 当没有文件）；P0 之前的根读回旧内容。所以「在哪个根上读」决定答案，与 `ColdStartRecover`、`RollbackWhileMounted` 已有的「按所选根答」是同一套（同文件 `answer_cold_start_recover`），不新增答案种类。
- 观测侧：`ObservedFile::NoFile` 今天只由树表 0 条那一版给（`crates/singlefs-harness/src/model_comparison.rs:399`、`:502`）；要加「inode 记录 nlink = 0 或不在」这一支。
- 随机历史：`HistoryOperation`（8 个成员，事实表 (g2)）加 `Unlink`，改动点数按 `PublishOverwrite` 出现处（事实表 (g3) 列了 harness 里 13 个文件，我没逐处复核）。

**什么现象会推翻 U5 的推荐**：(i) 攻方在副本上跑出「删掉 inode 1 之后的一版」被恢复接受（第 0 节 a、b 的推断错）——那 5.4 表缩成 nlink 判定；(ii) 用户要求里程碑三就带「再建带数据的文件」——改 X2；(iii) 用户不认「树表 8 条零条目根」而要回到 0 条——那水位得另找地方放（否则违反不复用），是格式改动；(iv) checker 走读今天就拒零条目根（我没现查 `walk.rs` 有没有「根 0 条目」的拒因，`grep -n 'entries.is_empty()' crates/singlefs-checker/src/walk.rs` 输出为空，但那只说明没有这一种写法）——5.4 表最后一行要加改动。

补一条规则没说的格（推）：**零条目根的头里 key 区间写什么。** 码 2 节点头带 key 区间（D18 已定项 2），checker 与恢复对 inode 树根今天的写法是「区间贴紧首末两条条目的 key」（`crates/singlefs-checker/src/walk.rs:172` 的注释：「区间贴紧首末两条条目的 key：叶一律这样；inode 树根、树表与根兼叶的那几棵今天也这样写。」）；零条目时没有首末条目，区间无从贴。I-9.13 只说「空叶在重建时没有 key 范围」，没给根的写法。E-c 下 extent 根的区间是位置定的（叶 0 罩 inode `[0, 143)`，不缺）；inode 树根与队列根要补一句约定（例如 inode 树根写 `[水位, 水位]`、队列根写 `[0, 0]`），这是格式字节，三·6 的 `layout/03-third-txn.md` 要登记，我不替主 agent 选值。

## 6　U6　崩溃点与流

**判定**：自动命名 = 与今天一致（不需要新编码，「第几批」靠测试代码给入口起名）；流与状态数 = 规则没说，下面给算式；删除类发布（截断成 0 那一格）= 与现有登记一致（没跑），我的意见是并进同一次全量跑、不并进流。状态数全是推的：由今天测试里的闭式反推，没跑。

### 6.1　自动命名够不够分辨「第几批」

- 命名规则（第 0 节 r）：`<入口>.<周期>.<步>`，周期 = 这一屏障段里写的种类（按单元头类标签：`data_unit`、`index_node`、`packed_records`，别的写用写种类名），步 = 这个入口里这一种周期的第几段（`crates/singlefs-checker-tier/src/crash_amplification.rs:2389` 生成名字，`:2360` 的计数器按（入口下标，周期名）分）。「同一个入口调几次就是几段路径，名字相同、位置不同」是那一段的原注释，同名不同位置是允许的。
- 一次发布的写序（`crates/singlefs-core/src/transaction.rs:1052` 的 `persist_publish_writes`）是：全部单元写 → 屏障 → 记录写 → 屏障 → 根槽 FUA → 系统配置槽轮换 → 屏障，所以每次发布在名字里出四五段：`unit`（周期名由这段里的类标签拼）、`journal_record`、`root_record_fua`、`system_configuration_slot`。
- 删除的每次发布（P0 与各批）单元段里都有 inode 叶（码 3）与各根（码 2），周期名恒是 `index_node+packed_records`；最后一批叶不落盘，周期名只剩 `index_node`。若整个 unlink 用**一个入口**（名 `unlink`），批序号 = 步号（`unlink.index_node+packed_records.1` 是 P0，`.2` 是第 1 批……），最后一批变成 `unlink.index_node.1`：能分辨，但是靠「每批恰一个同周期段」这个巧合，一批多一段就错位。若每次发布各设一个入口且都叫 `unlink_batch`，则各批的名字逐字相同（步号都是 1），更糟。
- **推荐：不加新编码，让测试的步函数按序号起入口名**：`unlink_intent`、`unlink_batch_1`、`unlink_batch_2`、`unlink_batch_3`（步函数照 `crates/singlefs-checker-tier/tests/common_crash_points/mod.rs:41` 的 `overwrite_step` 写一个 `unlink_step`，入口名由调用方给，`crash_amplification_fixed_script_through_unmount.rs` 里入口清单那一个 `vec!` 就是这么登记的，`crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_unmount.rs:225` 那条断言的形态）。崩溃点名于是是 `unlink_batch_2.journal_record.1` 这种，批序号进了名字，覆盖表（里程碑三·4 的第四条出口）从录制流与登记表算，直接按名认。

### 6.2　哪条流、多少崩溃状态（算式推的）

一次发布的崩溃状态数：2^n + 2^(2r) + 7（n = 这次发布的单元写数 = 单元数 × 2 块盘，r = 这次发布的 journal 记录条数）。推导：C577 之后每次发布 = 上一次轮换段 3² − 1 = 8 + 单元段 2^n − 1 + 记录段（每条记录每盘一个写，2r 个写）2^(2r) − 1 + 根槽段 1 = 2^n + 2^(2r) + 7（r = 1 时就是测试里的 2^n + 11，`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:102`）。**我拿现成的多记录流验了这个式子**：「取号 → 暖机 × 2 → A（第一个文件，一个数据单元：12 个单元）→ B（顺序写两个数据单元：14 个单元、两条记录）」（`crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs:3`）与「→ C（顺序写三个数据单元：15 个单元、三条记录）」（`crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs:4`）全量枚举域 1 358 954 634（同文件第 21 行）：A = 2^24 + 4 + 7 = 16 777 227，B = 2^28 + 2^4 + 7 = 268 435 479，C = 2^30 + 2^6 + 7 = 1 073 741 895，加取号与暖机两次 12 + 12、最后一根之后 8、全部持久 1，`python3` 算 `16777227 268435479 1073741895 1358954634`，与文件里的数逐位相等。

三条候选流（N = 数据单元数，K = 1；每次删除发布的单元数按第 3、5 节的形态：P0 = 8 固定 + inode 叶 + inode 根 + 队列根 = 11 个单元；中间批与最后一批见下）：

| 候选 | 流 | 增量崩溃状态 | 总数 |
|---|---|---|---|
| F-1（推荐） | 多记录流后面接 unlink C 那个 3 单元文件：P0 + 3 批。P0 11 个单元（n = 22）；第 1 批释放单元 2、下段叶与上段根重写 + inode 叶 + 根 + 8 固定 = 12 个（n = 24）；第 2 批释放单元 1、剩 1 个内联单元，只重写上段根 + inode 叶 + 根 + 8 固定 = 11 个（n = 22）；第 3 批（最后）8 固定 + extent 根 + inode 根 + 队列根，各是零条目根（第 5 节 5.2）= 11 个（n = 22） | 3 × (2^22 + 11) + (2^24 + 11) = 3 × 4 194 315 + 16 777 227 = **29 360 172** | 1 358 954 634 + 29 360 172 = 1 388 314 806 |
| F-2 | 同一条流去掉 C，在 B 后 unlink 2 单元文件：P0 + 2 批（各 11 个单元，n = 22） | 3 × 4 194 315 = 12 582 945 | 33 + 16 777 227 + 268 435 479 + 12 582 945 = 297 795 684 |
| F-3 | 接在固定脚本 E 之后（里程碑三·4 表里「固定脚本到 E 之后加 unlink 段」那句）：脚本里没有多单元文件（第 3.3 节），要先多一次两单元顺序写（E 上下文 13 + 2 + 1 = 16 个单元，n = 32，2 条记录：2^32 + 2^4 + 7 = 4 294 967 319），再 unlink（E 上下文里固定点多两个分配记录树节点，每次 13 个单元、n = 26，3 次共 3 × 67 108 875 = 201 326 625） | 4 294 967 319 + 201 326 625 = 4 496 293 944（约为今天整条到卸载的流 1 662 648 564 的 2.7 倍） | 1 662 648 564 + 4 496 293 944 = 6 158 942 508 |

`python3` 算式：`python3 -c 'u=2**22+11; m=2**24+11; print(3*u+m, 1358954634+3*u+m, 3*u)'` 输出 `29360172 1388314806 12582945`；F-3 一行 `python3 -c 'print(2**32+16+7, 3*(2**26+11), 2**32+16+7+3*(2**26+11), 1662648564+2**32+16+7+3*(2**26+11))'` 这一次现跑输出 `4294967319 201326625 4496293944 6158942508`；`python3 -c 'print(16777223/1388314806)'` 输出 `0.012084595602879424`。

推荐 F-1：一是「至少两批」在它里面天然成立（N = 3、K = 1 得 3 批，还带一次「下段叶只剩一个内联单元」的形态切换，那一格 N = 2 的流碰不到）；二是它接在已有的多记录流后面，前缀共享（`crash_amplification_two_flows_share_their_prefix.rs` 文件头：「流 A 核完之后，流 B 录入时前缀全部复用、只核自己的尾巴」）让增量只算 unlink 那四次发布；三是最大段 n = 24，比 B、C 的 28、30 小得多，全量可跑（今天 GPU 上展开 28 已全枚举）。F-3 只为遵字面「固定脚本到 E 之后」，代价 2.7 倍且多一个 n = 32 的巨段，覆盖上没有多出东西。**这一步偏离里程碑表三·4 的字面「固定脚本到 E 之后加 unlink 段」**（「或新流 mkfs → A → unlink → 再建第一个文件版本」那半句字面也不同：F-1 不建再建，第 5 节 X1），三·4 的验收要的是「两条流全域 0 红；改坏『最后一批与删意图同事务』那一处，那条流的判器红」，F-1 满足，需要主 agent 认。

### 6.3　崩在回收中间之后再挂载（续做本身崩）

- 今天的崩溃注入在每个崩溃状态上真起一次可写挂载（取号、写行、暖机），挂载途中的二次崩溃取三段各摆一个：`crates/singlefs-checker-tier/src/crash_injection.rs:251` 的 `WritableMountPhase`，三个成员（`Acquisition`、`RowPublish`、`WarmUp`）。续做进 `mount_writable`（第 3.5 节 M2）之后，要加第四个成员 `Continuation`（挂载在暖机与抬 F 之后推的批发布），全仓 `WritableMountPhase::` 引用 22 处（`grep -rn 'WritableMountPhase::' crates --include=*.rs | wc -l` 输出 22），逐处加臂。
- 固定脚本流里「重开、可写挂载」是一个入口（`("mount_writable", 67),`，`crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_unmount.rs:217`）；续做批进这个入口之后它的写数变大，且入口内部分不出「哪几个写是续做」。要让二次崩溃的名字带批序号，需要在 F-1 流里加一步：「P0 之后进程重开，可写挂载（取号、写行、暖机、续做三批）」——三批在挂载里走，名字是 `mount_writable.<周期>.<步>`，批序号只靠步号推。这与 6.1 的「入口名带序号」不能同时满足，除非核心层把续做批的发布拆成一个可单独调的函数、测试只在这条流里绕开 `mount_writable` 自己的续做去调它——那是「恢复路径里存在跳过续做的分支」，正是 C1 的检查列点名要拦的东西（`.claude/kb/checks-owed.md:17` 的「怎么拦」：「恢复路径里不许存在跳过续做的分支」，整行较长，行号由 `grep -nF` 命中）。⇒ 我的取法：F-1 流里 unlink 的 P0 与各批都直接调各自的公开函数（每次发布一个入口，6.1），**这条流不含挂载续做**；续做（含二次崩溃）由 `WritableMountPhase::Continuation` 那一路在每个崩溃状态上起可写挂载来罩，不进固定脚本流。这样 C1 的字面不破，批序号进名字。
- 二次崩溃的状态量：续做那几批的写数与第一次挂载起的批同（n = 22 / 24），按 `crash_injection` 的做法在每个崩溃状态上摆有限个（三段各一个），不是全枚举，量级与今天挂载三段同（每状态多 1 次挂载）。没估具体数，因为随机摆点的个数是那个装置的旋钮。

### 6.4　安全轮第三轮「崩在删除类发布中间」（截断成 0 那一格）并不并进来

- 事实：`records/2026-09-28-里程碑三第十项清单.md:58` 那一行写着「崩在删除类发布中间（近满之后截断成 0 那一次发布，一条流 16,777,223 个状态）没跑」，归项六、`stage-inputs.tsv` 的八条 `crash-case:` 都不是这条流。16 777 223 = 2^24 − 1 + 8（一个 n = 24 的单元段加上一次轮换段，`python3 -c 'print(2**24-1+8)'` 输出 `16777223`，与文档数逐位相等；推的：这条流的其余段我没拆）。
- 它是「近满池上的一次覆盖写，释放多于写」，不是 unlink：它测的是准入在池近满时对「换下的不算可用」的判（第 4 节 R0 / R1 / R2 都不改这一条），而 unlink 批发布是被预付过的（R2）。两者的基镜像也不同（近满池 vs 正常池），不能拼进 F-1 的前缀。
- **推荐：不并进 unlink 流，另立一条流，并进同一次全量层 0 跑**（增量 16 777 223，约 F-1 总数的 1.2%，`python3 -c 'print(16777223/1388314806)'` 算得 0.01208…）。这样第十项清单那一格有主，unlink 的流不背它的基镜像。这一步取决于用户 2026-09-26 定的「随第 5 步提交时的全量层 0 一起看」（里程碑二第 5 步已收尾，那个「一起看」没落地，见上一条的清单行），需要主 agent 认。

**什么现象会推翻 U6 的推荐**：(i) 攻方在副本上跑出 F-1 里某一批的单元数与我的 11 / 12 不同（推导的 n 错）——状态数式子照常用，数变；判据是式子在多记录流上复现了 1 358 954 634，这一条已经立住，所以要推翻的只是「单元数」，不是式子；(ii) 用户不接受 F-1 偏离「固定脚本到 E 之后」——那改 F-3，代价 6.158e9 态；(iii) 核心层把续做批设计成不可单独调用（只能经 `mount_writable`）——6.1 的「入口名带序号」不成立，改回靠步号；(iv) 用户要求续做批也进固定脚本流——6.3 的取法作废，要走「拆函数 + 绕开」并另开一笔 C1 的例外。

## 7　U7　与稀疏旁表、整理共用意图机制

**判定**：规则没说（三个消费者的意图字段表 kb 里都不全，C364 开着）；一处字面冲突：D8 已定项 5 点名的载体是 `logged_ops` 树，第 1 节推荐的 K1 不建这棵树（unlink 的意图就是队列条目本身）。推荐 K1 不给别的消费者留字段、只留种类码与四条纪律；以后追加一棵树的改动面与今天加 `deleted_inodes` 同量，软格式期只算同步成本。

### 7.1　三个消费者各要什么（kb 原文，整行并排；字段宽是推的）

| 消费者 | 目标态 | 字段（kb 说了什么） | 出处 |
|---|---|---|---|
| unlink（这一轮） | inode N 无 extent、无记录、不在队列里 | 一个号（key 8 字节，无 value），第 1 节 1.2 | 本报告第 1 节 |
| 快照 / 头销毁的 deadlist 清理 | 无 [lo, hi) 里 birth > P 的条目 | 意图 (头, lo, hi, P)，四个量，按 8 字节记 32 字节（推的） | `.claude/kb/decisions/05-快照-空间记账机制.md:271` 那一行开头「\| 销毁 S \| 写意图 (头, lo = S.txg, hi, P = prev(S).txg)」 |
| 稀疏旁表删除 | 没写 | 「删除走意图机制」，意图形态一个字都没有 | `.claude/kb/decisions/05-快照-空间记账机制.md:14`（已定项 6 的一行：「提交时维护，删除走意图机制」）；规则没说 |
| 整理（D26） | R 内不存在分配代 ≤ c0 且未释放的落点 | 意图记录装 R、c0、segs0、续做点四样，宽度与编码「全仓一处都没有」 | `.claude/kb/decisions/26-后台整理与放置回收.md:48`（目标态）、`:26`（segs0 必须写成值）、`:127`（「整理进度 \| 意图记录 \| 它就是 D8（核心索引结构） 已定项 5 的续做点」）、`.claude/kb/checks-owed.md:303`（C364） |

三者的共同点只有一条：目标态、不含增量（I-4.4，`.claude/kb/invariants.md:168`）；字段个数、宽度、有没有 value 都不同（0、32、未定、≥ 4 个量）。所以「统一意图记录字段」现在拿不出输入：两个消费者的字段表还没定，我没有依据替它们选宽度。

### 7.2　字面冲突（两边整行并排）

左（D8 已定项 5 的载体）`.claude/kb/decisions/08-核心索引结构.md:107`（同一行）：

> 先把意图写进 logged_ops 树，再分批推进

右（第 1 节 K1，推荐）：不建 `logged_ops`，unlink 的意图 = `deleted_inodes` 里那一条 key（第 1 节 1.3 表的 K1 行）。

三·1 的作业点自己把这一格留成了开问：「意图记录住哪（单独一棵 logged_ops 树，还是记账或树表里一条条目）」（`.claude/kb/milestone/03-third-txn.md:83` 三·1 那一行），所以 K1 没违反里程碑，只是与 D8 已定项 5 的措辞不同；D8 新分项要把那句改成「logged_ops 在第一个需要带字段的意图的消费者到来时追加；单文件 unlink 的意图就是 `deleted_inodes` 的一条 key」。

### 7.3　候选与代价数

| 候选 | 现在多写 | 以后 | 缺点 |
|---|---|---|---|
| P1（推荐，= K1） | 0（第 1 节 1.3：+1 单元、n = 22） | 消费者到来时追加一棵树：D15 判定表「新增一棵 keyspace」是 compat（`.claude/kb/decisions/15-格式冻结政策.md:190`）；追加的改动面 = 第 1 节 1.4 那份清单（19 个文件引用树 ID / 种类常量，树表 8 → 9 条、水位 20 → 21）；种类码 10、树 ID 20 空着，登记表只追加不重用（`.claude/kb/decisions/08-核心索引结构.md:279`） | 每个消费者各写一棵树 vs 共用 `logged_ops`，到那一轮再定；现在不给它们留任何字节 |
| P2（= K2） | `logged_ops`（种类 10、树 ID 20）+1 棵零根注册：树表 +200 字节，unlink 每次多写 2 个单元；P0 n = 24，2^24 + 11 = 16 777 227（K1 的 4 倍） | 无需再加树，但字段表仍得由各消费者定，`logged_ops` 的 key / value 形态要现在猜 | 现在为不存在的消费者付 4 倍崩溃状态、多一棵要维护的树（改动面同 K1 再加一份，19 个文件再多引用一个常量），猜错了要改格式 |
| P3 | 给 `deleted_inodes` 条目留「种类 + 保留位」（每条 16 字节，容量 2031 → 1015） | 以后别的消费者往同一棵树里放 | 名字是 deleted_inodes、key 是 inode 号，别的意图没有 inode 号；「预留位的任何赋义是 incompat」（同一条推理见第 1 节 K5）；出局 |

推荐 P1，并在 D8 新分项里写四条给以后的消费者的纪律（零字节）：(1) 意图字段只写目标态（I-4.4）；(2) 意图 key 与最后一批工作同事务删除（D8 已定项 5：「不可能出现「意图没了但目标状态没达成」——**删除意图必须与最后一批工作在同一个事务里提交**」，`.claude/kb/decisions/08-核心索引结构.md:119`）；(3) 每种意图在字段表登记位有一行，门禁逐字段比对（C364 的「怎么拦」）；(4) 新种类追加树种类码、不重用（`.claude/kb/decisions/08-核心索引结构.md:279`）。

### 7.4　「不留的话以后加要不要改格式」

- 加一棵树：不改已有字节。D15 判定表 compat（`.claude/kb/decisions/15-格式冻结政策.md:190`）；但今天的读者比 compat 严：树表里出现一个没登记的种类且有根，恢复报 `树的种类没登记`（`crates/singlefs-core/src/recovery.rs:2832` 与 `:4045` 两处），旧软件读新盘会拒，不是「看不到新树」。软格式期无所谓：「在第一个外部用户出现之前，磁盘格式是软的」（`.claude/rules/format-evolution.md:7`）。
- 加一种意图带来的语义变化才是真问题：旧软件可写挂载一个带活意图的池而不续做，会违反 C1（D26 的整理更严：「`R` 不可作分配目的地」靠读活意图推出，旧软件不读就会往 R 里分配）。那要靠 feature bit（incompat，D15 已定项 4 的登记表）在第一个意图种类落盘时置上。这一格今天不需要：没有外部用户。冻结（D15 已定项 5、6）之前要把它列进「冻结前必须完成的清单」。
- 顺带一格 unlink 自己的：nlink 由恒 1 变成可为 0、`deleted_inodes` 的存在，对旧读者都是「改变已有字段含义」（`.claude/kb/decisions/15-格式冻结政策.md:188`，incompat）——旧软件读 nlink = 0 的记录会当活文件读出来（第 0 节 f：nlink 读时跳过）。同样软格式期不设位，冻结前要设。

**什么现象会推翻 U7 的推荐**：(i) 用户在这一轮就要 D26 或稀疏旁表的意图形态（那两个消费者的字段表落定）——那 P2 有了输入，可以现在建 `logged_ops`，代价数照 7.3；(ii) 攻方给出一个 unlink 意图必须带 unlink 之外字段的场景（例如每次 unlink 要带发起者、时间）——K1 的「无 value」要改，是一次格式改动；(iii) 用户认为「D8 已定项 5 的 `logged_ops` 措辞不许偏离」——则 K1 不成立，改 P2（第 1 节 K2）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| U1 格式：`deleted_inodes` 与意图住哪（C118） | 规则没说；「key 只存号、inode 记录留在 inode 树」一致 | 推荐 K1：一棵 `deleted_inodes`（树 ID 19、种类码 9、key 8、无 value），零根 day-1 注册，不建 `logged_ops`；P0 +1 单元（n = 22，4 194 315 态）；派生态登记两行要主 agent 认 |
| U2 墓碑（D18 已定项 8 / 10） | 冲突 | D18 要盘上留墓碑，载体（C103 容器索引）、字段表、回收门统计量全无落点；一次 unlink 只有 extent 一条线要墓碑；推荐 T0（v1 不写、记欠账），偏离 D18 字面要用户认 |
| U3 分批与续做（C1、C2） | 与 D8 已定项 5 四条一致；三·2「同批续做两次字节相同」字面冲突；续做挂点规则没说 | 从尾巴起删、K = 144 一批（测试旋钮 K 可压到 1）、size 跟着 extent 走；续做挂在 `mount_writable` 抬 F 之后、并加会话方法；预演要加进续做批；字节相同测不出（新实例代号），换成计划相等 / 无事可做不写 / 终态逻辑相同三条 |
| U4 空间：扣待删占用（I-3.4、C375） | 冲突（R1 读法与已分配重复扣）+ 规则没说（该记什么） | 推荐 R2：待删占用记「完成删除还要新写的最坏字节」，P0 起立、每批递减；三批逐格表见 4.3；偏离 D3 已定项 17 字面（满盘 unlink 会在 P0 被拒），R3 写入时预付合字面但常驻约 4.2% |
| U5 恢复与模型的前置 | 冲突（恢复三处拒删空的一版，D8 / I-9.13 已给删空形态）+ 规则没说（删完形态、再建文件） | 删完 = 树表 8 条、inode / extent / 队列三棵零条目根（E-c）；恢复改 5 处；再建带数据的文件推荐 X1（不做，记欠账），发号用 `publish_new_inodes`；模型要加第三态「有树无文件」与 `Unlink` |
| U6 崩溃点与流 | 自动命名一致（不需新编码）；流与状态数规则没说；删除类发布没跑（与清单一致） | 推荐 F-1：多记录流后接 unlink 3 单元文件，增量 29 360 172，总数 1 388 314 806；入口名按序号起；续做二次崩溃走 `WritableMountPhase::Continuation`、不进固定脚本流；截断成 0 另立一条并进同一次全量跑（+16 777 223） |
| U7 与稀疏旁表、整理共用意图机制 | 规则没说；D8 已定项 5 的 `logged_ops` 措辞与 K1 字面冲突 | 三个消费者字段表都不全，现在拿不出输入；推荐 P1：不留字段、只留种类码 10 与四条纪律，以后追加一棵树是 compat（今天的读者更严）；语义上要 feature bit，冻结前列清单 |

## 复核

- 这次接续做完后现跑：`python3 research/scripts/cite-check.py research/prompts/m3-delete-r1-sonnet-output.md` 末行见交回（本文件写完后最后一次跑的原样输出）。
- 整份没跑任何编译、测试、门禁；所有 n、崩溃状态数、可用空间值是从第 0 节的闭式与第 3 节的单元数推的，只有「式子在多记录流上复现 1 358 954 634」（6.2）是拿现成常量核过的一条。
- 前一条腿写的第 0–3 节我没改动；接续时核过它们的引文（`cite-check.py` 92 处全对），行号与第 3 节里的推导没有逐条重推。

## 没做什么

- 没编译、没跑任何测试与门禁；没在冻结副本上起任何原型或用例（`research/prompts/m3-delete-r1-sonnet-model/` 没建：这一轮我没写模型）。
- 不判别的腿的格：没替攻方量 U4「扣两次」（4.3 表是推的）、没在副本上核 U5「删掉 inode 1 之后恢复拒」（5.1 是读代码推的静态结论，攻方去核）。
- 没替主 agent 采纳或出判决：U2 的 T0 偏离、U4 的 D3 已定项 17 偏离、U5 的 X1、U6 的 F-1 偏离三·4 字面、U7 的 D8 措辞改写，都写成「要主 agent 认」。
- 没核的：checker 走读（`walk.rs`）对零条目根的实际行为（只核了 grep 无 `entries.is_empty()` 命中）；U6 里 unlink 各批的单元数（11 / 12）是按 3.2 的形态推的，没数过实际发布；事实表 (g3) 的 harness 改动处数没逐处复核；U4 的 W = 12 是我估的最坏值。
- 没读禁读清单里的任何文件；没碰仓内别的文件；草稿只放在 `/tmp/claude-1000/m3-delete-r1-sonnet-2/`（`u4.py`、`report.bak`，见交回）。
