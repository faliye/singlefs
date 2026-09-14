//! E142：第一个事务的干跑——把 `.claude/kb/first-txn-layout.md` 那张字节表原样写成字节，
//! mkfs + 一个小文件 + 一次发布，两块 512 槽的盘；冷启动读回；按 D13（验证路线） 已定项 4 的层 0
//! 崩溃状态集合逐个恢复。目的是让代码把「字节表里哪几格写不出来、哪几条已定条款在字节层面互相顶着」逼出来：
//! 每一处装置不得不自己补的取法都报成一行 `gap`。跑前登记 `research/prompts/e142-preregistration.md`。
//!
//! 判 oracle 用 E77（发布的持久顺序） 判据 1：根槽已持久而恢复 ≠ 新态或走读失败即违例；根槽未持久时新旧两态都合法。
//! 纯确定性：fsid、时间戳都是固定参数，跑 N 遍逐字节一样。

use e7_index_bench::Emitter;
use std::collections::BTreeMap;

// ───────────────────────── 格式常量（每条带分项号；标「预想」的是仓里没定、装置自取的） ─────────────────────────

/// D20（承重面：单元的原子性与自包含）：判定宽度是运行时探测值，本机 nvme0n1 是 512（字节表零那一节）。装置按参数给。
const PHYSICAL_BLOCK_BYTES: u64 = 512;
/// D3（空间分配） 已定项 7：落点粒度 16 KiB 槽。
const SLOT_BYTES: u64 = 16384;
/// D8（核心索引结构） 已定项 2。
const NODE_BYTES: u64 = 16384;
/// D4（校验和位置） 已定项 5 / 已定项 7。
const DATA_UNIT_BYTES: u64 = 32768;
/// D18（块里携带什么信息） 已定项 7：共同前缀 42 + 数据单元类身份段 63。
const DATA_UNIT_HEADER_BYTES: u64 = 105;
/// D18（块里携带什么信息） 已定项 11：共同前缀 42 + 码 3 类身份段 65。
const PACKED_UNIT_HEADER_BYTES: u64 = 107;
/// D18（块里携带什么信息） 已定项 7 的共同明文前缀。
const COMMON_PREFIX_BYTES: u64 = 42;
/// D18（块里携带什么信息） 已定项 14 / 已定项 16（算法类型那 1 字节 2026-09-14 用户定案加）：
/// nonce 12 + MAC 16 + 算法类型 1 = 29 字节预留位，紧接类身份段之后。
const NONCE_MAC_RESERVED_BYTES: u64 = 29;
/// D22（单元原子性怎么合成） 已定项 7 的字段表：magic 4 + fsid 16 + flags 4 + 实例代号 4 + checkpoint_txg 8
/// + 树表单元指针 86 + 树 ID 水位 8 + 回退下界 F 8 + 自证校验和 32 + 实例表单元指针 86
/// + 中央映射树根指针 86 + 算法类型 1 + nonce 12 + MAC 16（末尾三段 2026-09-14 用户定案加）。
const ROOT_RECORD_BYTES: u64 = 371;
/// D19（块指针的结构与宽度预算） 已定项 4：设备 4 + 16 KiB 槽号 6 + 密文校验和 4。
const LOC_ENTRY: u64 = 14; // naming-lint:external 名字由 kb 的 format-const 登记位定（D19 已定项 4），门禁按这个名字绑值
/// D19（块指针的结构与宽度预算） 已定项 7 / 已定项 11 的偏移表：MAC 16（偏移 0）+ nonce 12（16）+ 算法类型 1（28）
/// + 压缩算法码 1（29）+ 压后长度 2（30）+ extent 距起点偏移 2（32）+ 出生树 8（34）+ 出生 txg 8（42）。
/// 压缩那两段 2026-09-14 用户定案加，头部从 47 变 50。
const POINTER_HEAD_BYTES: u64 = 50;
/// D19（块指针的结构与宽度预算） 已定项 8：指向码 2 / 码 3 的指针 = 头部 + 位置条目 × 2 + 实例代号 4 + 出生序号 4。
const NODE_POINTER_BYTES: u64 = 86;
/// D19（块指针的结构与宽度预算） 已定项 8：指向码 1 的指针 = 头部 + 位置条目 × 2 + 写序 10。
const DATA_POINTER_BYTES: u64 = 88;
/// D8（核心索引结构） 已定项 8（2026-09-14 用户定案重排）：树 ID 8 打头 + 条目长度 2 + 树的种类 2 + flags 2
/// + 根指针 86 + previous_snapshot_txg 8 + 诞生 txg 8 + 头 ID 8 + 预留 24。
const TREE_TABLE_ENTRY_BYTES: u64 = 148;
/// D8（核心索引结构） 已定项 6。
const INODE_RECORD_BYTES: u64 = 140;
/// D8（核心索引结构） 已定项 6：分隔 key 8 + 身份引用 26 + 子指针 86。
const INODE_INTERNAL_ENTRY: u64 = 120;
/// D8（核心索引结构） 已定项 3 + D19 已定项 8：key 24 + 指向码 1 的指针 88。
const EXTENT_LEAF_RECORD_BYTES: u64 = 112;
/// D3（空间分配） 已定项 7 + 已定项 11：key 10（设备 4 + 槽号 6）+ value 10（跨度 2 + 代 8）。
const ALLOCATION_RECORD_BYTES: u64 = 20;
/// D3（空间分配） 已定项 11：跨度段进 value ⇒ 分配记录树的 key 宽 10。
const ALLOCATION_KEY_BYTES: usize = 10;
/// D5（快照 / 空间记账机制） 已定项 5 + D8 已定项 7：key 22 + value 8 + seq 4。
const ACCOUNTING_ENTRY_BYTES: u64 = 34;
/// D19（块指针的结构与宽度预算） 已定项 6 / 已定项 10（2026-09-14 用户定案回定宽）：中央映射的 key **一律 27**——
/// 码 1 天然 27（类标签 1 + 出生树 8 + 出生 txg 8 + 写序 10），码 2 / 码 3 的 25 字节末尾补零到 27。
const MAPPING_KEY_BYTES: u64 = 27;
/// 码 2 / 码 3 的映射 key 里有含义的那一段：类标签 1 + 出生树 8 + 出生 txg 8 + 实例代号 4 + 出生序号 4 = 25，其后两字节恒零。
const MAPPING_KEY_NODE_SIGNIFICANT_BYTES: u64 = 25;
/// 映射条目 = key 27 + value（位置条目 × 2）= **一律 55**（D19 已定项 6 / 已定项 10）。
const MAPPING_ENTRY_BYTES: u64 = 55;
/// D18（块里携带什么信息） 已定项 11：实例表一片记录宽 88（kind 1 + 有无下一片 1 + 位置指针 86 + 预留 0；
/// 2026-09-14 指针从 83 变 86 之后把那 3 字节预留吃掉，记录宽仍 88，打包记录定长要求 kind 0 / kind 1 同宽）。
const INSTANCE_ROW_BYTES: u64 = 88;
/// D23（journal 的角色与格式） 已定项 12。
const JOURNAL_RECORD_BYTES: u64 = 4096;
/// 记录头（D23（journal 的角色与格式） 已定项 4 的字段表）：十个字段 78（`JOURNAL_HEADER_TEN_FIELD_BYTES`）
/// + 事务号 8 + 提交标记 1 + 反向链 4 + 载荷校验和 4 + 新根段 188 + fsid 8 + MAC 16 = 307
/// （fsid 与 MAC 是 2026-09-14 用户定案加的）。4096 的记录装 (4096 − 307) / 56 = 67 个点名项。
const JOURNAL_HEADER_BYTES: u64 = 307;
/// 十个字段那一段另登记成格式常量，好让门禁分得清它与整个头（D23 已定项 4）。
const JOURNAL_HEADER_TEN_FIELD_BYTES: u64 = 78;
/// D23（journal 的角色与格式） 已定项 15：新根段 = 树表单元指针 86 + 中央映射树根指针 86 + 树 ID 水位 8 + 回退下界 F 8。
const JOURNAL_NEW_ROOT_SEGMENT_BYTES: u64 = 2 * NODE_POINTER_BYTES + 8 + 8;
/// D23（journal 的角色与格式） 已定项 4 口径的点名项宽度；构成无落点，装置按字节表六的预想构成写。
const JOURNAL_NAMED_ENTRY_BYTES: u64 = 56;
/// 超级块字段表合计（D22（单元原子性怎么合成） 已定项 9 + 已定项 15）：2026-09-14 用户定案加四个字段——
/// 自举头的写入者身份 20 与校验和算法标识 1、几何段的 mkfs 时 physical_block_size 4 与扩展点声明值 N 4，共 29。
const SUPERBLOCK_BYTES: u64 = 481;
/// 头校验和 / 自证校验和的字段宽度（D18 已定项 7、D22 已定项 7、D23 已定项 4 同口径）。
/// 算法由 D18（块里携带什么信息） 已定项 17 定（2026-09-13）：字段里放 CRC32C 4 字节 + 28 字节零。
const WIDE_CHECKSUM_BYTES: u64 = 32;
const WIDE_CHECKSUM_CRC_BYTES: usize = 4;

/// 码 2 节点头里 key 区间之外的部分（D8（核心索引结构） 已定项 11 + D18（块里携带什么信息） 已定项 18 的偏移表）：
/// 共同前缀 42 + 树 ID 8 + 层级 1 + key 宽 1 + 诞生代号 8 + fsid 8 + 写序 4 + 出生序号 4 + 载荷 CRC 4
/// + 预留 2 + 条目数 2 + 条目宽 2 = 86；含 29 字节预留位的头 = 115 + 2 × key 宽。
/// key 宽 2026-09-14 用户定案从 81 + 2k 挪到**偏移 51**——定位它不再需要先知道 k。
const INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE: u64 = 86;
/// 码 2 头里 key 宽那一格的偏移（D18 已定项 18）：共同前缀 42 + 树 ID 8 + 层级 1 = 51。
const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;

/// 字节表零：超级块槽 0 / 1 的设备内偏移。槽距 = 固定结构槽距 4096，与槽宽同值 ⇒ 两个槽首尾相接。
const SUPERBLOCK_SLOT_OFFSETS: [u64; 2] = [0, 4096];
/// D22（单元原子性怎么合成） 已定项 2 的槽宽那一格（2026-09-14 三方论证后按主 agent 推荐值写）：
/// **超级块槽宽是格式常量 4096**，不再等于挂载时探测到的 `physical_block_size`。
/// 整槽校验和罩这 4096 字节含补齐（D18（块里携带什么信息） 已定项 17），481 字节的字段表在槽里余 3615。
/// ⚠️ 它只管超级块：根槽仍按判定宽度 512 写（字节表七）。
const SUPERBLOCK_SLOT_BYTES: u64 = 4096;
/// 字节表零：根环起点 1 MiB（槽 64）、P = 3、chunk = 1 MiB、每区 8 槽、槽距 4096（预想）。
const RING_START_OFFSET: u64 = 1 << 20;
const RING_PRIME_STEP: u64 = 3;
const RING_CHUNK_BYTES: u64 = 1 << 20;
const RING_REGIONS: u64 = 3;
const RING_SLOTS_PER_REGION: u64 = 8;
const RING_SLOT_SPACING: u64 = 4096;
/// D2（RAID 条带策略） 已定项 7（2026-09-14 用户定案）：根环三个区域的设备归属第一版**写死** 0 / 1 / 0，不是 mkfs 参数。
/// devs = 2、R = 3 时 D22 已定项 14 的鸽笼下界只允许「两个区域在一块盘、一个在另一块」这一种形状，
/// 区域编号一定之后 0 / 1 / 0 是唯一的规范写法；它也把暖机恒 2 次定死（txg 1 落盘 1、txg 2 落盘 0，两次覆盖两块盘）。
const RING_REGION_DEVICES: [u32; 3] = [0, 1, 0];
/// 字节表零：journal 环 16 MiB 起，长度按 D23（journal 的角色与格式） 已定项 19 ③ 改到 768 MiB，两盘互为镜像。
const JOURNAL_START_SLOT: u64 = 1024;
const JOURNAL_RING_BYTES: u64 = 768 << 20;
/// D23（journal 的角色与格式） 已定项 18：环槽数 = 环长 ÷ 记录宽；在飞记录数上限 = 环槽数 ÷ F（安全系数 3）。
const JOURNAL_RING_SLOTS: u64 = JOURNAL_RING_BYTES / JOURNAL_RECORD_BYTES;
const JOURNAL_SAFETY_FACTOR: u64 = 3;
const JOURNAL_IN_FLIGHT_RECORD_LIMIT: u64 = JOURNAL_RING_SLOTS / JOURNAL_SAFETY_FACTOR;
/// 超级块几何段的「journal 最坏占用」：在飞记录数上限 × 一条记录的字节数（I-8.1（环几何够大））。
const JOURNAL_WORST_CASE_BYTES: u64 = JOURNAL_IN_FLIGHT_RECORD_LIMIT * JOURNAL_RECORD_BYTES;

/// D3（空间分配） 已定项 10 ④：单元区起始槽号进超级块，第一版 = journal 环末尾的下一个槽（16 MiB + 768 MiB = 784 MiB）。
const UNIT_AREA_START_SLOT: u64 = JOURNAL_START_SLOT + JOURNAL_RING_BYTES / SLOT_BYTES;
/// 镜像大小是 mkfs 参数（跟 fsid、写入时刻同一类），装置取 4 GiB 并在 `name=config` 里报出来。
/// 下界由 D23（journal 的角色与格式） 已定项 19 ③ 的「环 ≤ 设备容量 ÷ 4」逼出：默认 768 MiB 的环要 3 GiB 以上的盘。
const DEVICE_BYTES: u64 = 4 << 30;
/// D5（快照 / 空间记账机制） 已定项 7：准入不等式里的「容量」= 单元区大小（起始槽号到盘尾），固定结构不在容量里。
const UNIT_AREA_SLOTS: u64 = (DEVICE_BYTES - UNIT_AREA_START_SLOT * SLOT_BYTES) / SLOT_BYTES;
/// D3（空间分配） 已定项 10 ①：聚簇段段长 64 个 16 KiB 槽；开放段 = 单元区内最低的、64 槽对齐的全空段。
const CLUSTER_SEGMENT_SLOTS: u64 = 64;

/// 落点（D3（空间分配） 已定项 10）：m1 / m2 是 mkfs 的两个固定单元，从单元区起点起铺；
/// 用户数据 t1 取不在开放聚簇段里的、最低的 32768 对齐空槽对；t2–t8 是提交内生块，从开放聚簇段 bump。
const SLOT_INSTANCE_TABLE: u64 = 50176;
const SLOT_TREE_TABLE_GENESIS: u64 = 50178;
const SLOT_DATA_UNIT: u64 = 50180;
const OPEN_CLUSTER_SEGMENT_START_SLOT: u64 = 50240;
/// D3（空间分配） 已定项 10 ⑤（2026-09-14 用户定案）：提交内生块从开放段 bump 的次序 =
/// 按树 ID 升序、树内先叶后根、中央映射树倒数第二、树表单元最末。游标从 50240 起：
/// t2 extent 根（树 11）拿 50240 ⇒ 游标 50241；t3 inode 叶是码 3 容器、按「数据单元」那一档要起点 32768 对齐
/// ⇒ 跳过 50241 拿 50242–50243；其后 t4..t8 依次 bump。**槽 50241 因此空着**，它自己是一段空闲 run。
const SLOT_EXTENT_ROOT: u64 = 50240;
const SLOT_INODE_LEAF: u64 = 50242;
const SLOT_INODE_ROOT: u64 = 50244;
const SLOT_ALLOCATION_ROOT: u64 = 50245;
const SLOT_ACCOUNTING_ROOT: u64 = 50246;
const SLOT_MAPPING_ROOT: u64 = 50247;
const SLOT_TREE_TABLE_FIRST_PUBLISH: u64 = 50248;
/// bump 次序留下的那个空洞（上面那段注释里的 50241）：记账的 runs 那一行多一段就是它。
const SLOT_SKIPPED_BY_ALIGNMENT: u64 = 50241;

/// D8（核心索引结构） 已定项 11（2026-09-13 用户定案）：第一版树 ID 从 11 起编，与树的种类码 1..7 错开。
/// 树表单元与实例表单元的树 ID 段写 0（无归属，D5（快照 / 空间记账机制） 已定项 10 登记的保留值）。
const TREE_IDENTIFIER_NONE: u64 = 0;
const TREE_IDENTIFIER_EXTENT: u64 = 11;
const TREE_IDENTIFIER_INODE: u64 = 12;
const TREE_IDENTIFIER_ALLOCATION: u64 = 13;
const TREE_IDENTIFIER_ACCOUNTING: u64 = 14;
const TREE_IDENTIFIER_MAPPING: u64 = 15;
/// D6 已定项 2（2026-09-13 用户定案）：livelist 共享树 day-1 注册，第一个事务根指针为零。
const TREE_IDENTIFIER_LIVELIST: u64 = 16;
/// D5 已定项 6（2026-09-13 用户定案）：稀疏旁表树 day-1 注册，第一个事务根指针为零。
const TREE_IDENTIFIER_SHARE_COUNT: u64 = 17;
/// D5 已定项 11（2026-09-14 用户定案）：deadlist 树 day-1 注册进树表，第一个事务根指针为零、一条条目都不写。
const TREE_IDENTIFIER_DEADLIST: u64 = 18;
/// mkfs 那一刻一棵树都还没有，水位就是第一个要发的树 ID；发布之后越过最大的那个。
const TREE_IDENTIFIER_WATERMARK_AT_MKFS: u64 = TREE_IDENTIFIER_EXTENT;
const TREE_IDENTIFIER_WATERMARK_AFTER_PUBLISH: u64 = TREE_IDENTIFIER_DEADLIST + 1;
/// 树的种类的码（D8（核心索引结构） 已定项 9 的登记表：0 无、1 extent、2 inode、3 分配记录、4 记账、
/// 5 中央映射、6 livelist 共享树、7 稀疏旁表、8 deadlist；码 8 是 2026-09-14 用户定案加的）。
const TREE_KIND_EXTENT: u16 = 1;
const TREE_KIND_INODE: u16 = 2;
const TREE_KIND_ALLOCATION: u16 = 3;
const TREE_KIND_ACCOUNTING: u16 = 4;
const TREE_KIND_MAPPING: u16 = 5;
const TREE_KIND_LIVELIST: u16 = 6;
const TREE_KIND_SHARE_COUNT: u16 = 7;
const TREE_KIND_DEADLIST: u16 = 8;

/// D18（块里携带什么信息） 已定项 11 登记表的码。
const UNIT_CLASS_DATA: u8 = 1;
const UNIT_CLASS_INDEX_NODE: u8 = 2;
const UNIT_CLASS_PACKED: u8 = 3;
/// D18 已定项 11 第二级登记表：打包记录类型 2 = inode 记录，4 = 实例表。
const PACKED_TYPE_INODE: u16 = 2;
const PACKED_TYPE_INSTANCE_TABLE: u16 = 4;

/// 记账统计量标签 = D5（快照 / 空间记账机制） 已定项 4 那张表的行号（已定项 10 立成登记表）。
const STATISTIC_ALLOCATED_BYTES: u16 = 1;
const STATISTIC_FREE_BYTES: u16 = 2;
/// D5 已定项 8（2026-09-14 用户定案）：准入不等式要的四个统计量 day-1 各写一行值 0，标签是 D5 已定项 4 那张表的行号。
const STATISTIC_UNRECLAIMABLE_BYTES: u16 = 3;
const STATISTIC_PENDING_DELETE_BYTES: u16 = 4;
const STATISTIC_DEFER_QUEUE_BYTES: u16 = 5;
const STATISTIC_COMMITTED_RESERVATION_BYTES: u16 = 6;
const STATISTIC_FRAGMENTATION_RUNS: u16 = 10;
const STATISTIC_EMPTY_CLUSTER_SEGMENTS: u16 = 11;
const STATISTIC_INODE_WATERMARK: u16 = 12;
/// D5（快照 / 空间记账机制） 已定项 10：不带设备维的统计量，key 的设备段取保留值 0xFFFF_FFFF。
const STATISTIC_NO_DEVICE_DIMENSION: u32 = 0xFFFF_FFFF;
/// D8（核心索引结构） 已定项 10：直落记账树叶时 seq 恒 1。
const ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF: u32 = 1;

/// D19（块指针的结构与宽度预算） 已定项 7 / 已定项 11（2026-09-14 用户定案）：指针里压缩那两段的第一版取值。
/// 登记表住 D9（加密） 硬要求 2 那一节：码 0 = 不压缩；码 0 时压后长度恒 0，非 0 判该指针损坏。
const POINTER_COMPRESSION_CODE_NONE: u8 = 0;
const POINTER_COMPRESSED_LENGTH_NONE: u16 = 0;

/// D5（快照 / 空间记账机制） 已定项 9（2026-09-14 用户定案）：树表条目从 32 字节预留里切 8 字节存「头 ID」。
/// inode 树条目写自己（12）、extent 树条目写它服务的那个头（12），分配记录 / 记账 / livelist / 旁表 / deadlist 写 0。
const TREE_TABLE_HEAD_IDENTIFIER_NONE: u64 = 0;

/// D22（单元原子性怎么合成） 已定项 9（2026-09-14 用户定案）：超级块自举头的「写入者身份」=
/// 实现标识 16 字节 ASCII 零补齐 + 版本 4 字节；第一版写 `singlefs-rs` 与 1（I-1.5（超级块记管道身份）、D17（实现分层与第三方管道） 债 3）。
const WRITER_IDENTITY_NAME: &[u8] = b"singlefs-rs";
const WRITER_IDENTITY_NAME_BYTES: usize = 16;
const WRITER_IDENTITY_VERSION: u32 = 1;
/// 同一条定案的「校验和算法标识」登记表：0 无效、1 CRC-32C；第一版写 1（I-2.2（校验和算法全盘一致）、C309（32 字节校验和的算法没定））。
const CHECKSUM_ALGORITHM_CRC32_CASTAGNOLI: u8 = 1;
/// 几何段 2026-09-14 加的两个字段：mkfs 时探测到的 physical_block_size（D13（验证路线） 已定项 5 的比对要它），
/// 与扩展点声明值 N（D21（权威态与派生态的分界） 已定项 8，第一版 0 字节）。
const MKFS_PHYSICAL_BLOCK_BYTES: u32 = 512;
const EXTENSION_POINT_DECLARED_BYTES: u32 = 0;

const UNIT_MAGIC: [u8; 4] = *b"SFSU";
const ROOT_MAGIC: [u8; 4] = *b"SFSR";
const SUPERBLOCK_MAGIC: [u8; 4] = *b"SFSB";
/// D15 已定项 4：incompat 位 0 = 第一条纯 SSD 布局线，mkfs 起就置上；位图小端、位 0 是第一个字节的最低位。
const INCOMPAT_FIRST_SSD_LINE_BIT: u8 = 0x01;
/// 三张位图各 32 字节（256 位）紧跟 magic 4 + 版本 2，incompat 在前、compat_ro 居中、compat 在后（D15 已定项 1）。
const SUPERBLOCK_FEATURE_BITS_OFFSET: usize = 4 + 2;
const FEATURE_BITMAP_BYTES: usize = 32;
/// 读者认识的全部 incompat 位；多出任何一位就是「不认识不许挂」（fs-design 格式层判据）。
const SUPPORTED_INCOMPAT_BITS: u8 = INCOMPAT_FIRST_SSD_LINE_BIT;
const JOURNAL_MAGIC: [u8; 4] = *b"SFSJ";
const FORMAT_VERSION: u16 = 1;

/// 装置固定的 mkfs 参数：fsid 与写入时刻都是参数，产物才能逐字节复跑（gap G11：里程碑步 1 的「同参数两次逐字节相同」要求 fsid 是参数而不是随机）。
const FIXED_FSID: [u8; 16] = [0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00];
const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
const FIRST_INODE_NUMBER: u64 = 1;
/// D23（journal 的角色与格式） 已定项 16：mkfs 写实例代号 0（「mkfs、尚无实例」，不是有效实例），
/// 第一次可写挂载取 max(超级块, 根环) + 1 = 1，并先写进每一份超级块之后才动单元。
const MKFS_INSTANCE_GENERATION: u32 = 0;
const FIRST_INSTANCE_GENERATION: u32 = 1;
/// D22（单元原子性怎么合成） 已定项 16：超级块槽世代号从 1 起、每写一次 +1，写世代号 g 的那一次落在槽 `g mod 2`。
const SUPERBLOCK_GENERATION_AT_MKFS: u64 = 1;
const SUPERBLOCK_GENERATION_AT_INSTANCE_ACQUISITION: u64 = 2;
/// D16 已定项 8（暖机取甲′，2026-09-13 用户定案）：mkfs 之后第一次可写挂载先连推空发布，直到本实例写成的根覆盖两块盘；
/// 第一版几何区域 1 / 2 分住两块盘 ⇒ 两次（txg 1、2），第一个事务从 txg 3 起。
const WARM_UP_EMPTY_PUBLISHES: u64 = 2;
const FIRST_TRANSACTION_TXG: u64 = 3;

// ───────────────────────── 地址空间的 newtype：混用编译不过 ─────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct DeviceIdentity(u32);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct SlotNumber(u64);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct DeviceOffset(u64);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct CheckpointTxg(u64);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct InstanceGeneration(u32);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct TransactionNumber(u64);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct TreeIdentifier(u64);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct BirthSequence(u32);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct JournalCounter(u64);

impl SlotNumber {
    fn device_offset(self) -> DeviceOffset {
        DeviceOffset(self.0 * SLOT_BYTES)
    }
}

/// C113 定案 P1 的写序：实例代号 4 + 事务号低 48 位。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct WriteOrder {
    instance: InstanceGeneration,
    transaction: TransactionNumber,
}

// ───────────────────────── 字节读写 ─────────────────────────

struct ByteWriter {
    bytes: Vec<u8>,
    cursor: usize,
}

impl ByteWriter {
    fn new(total_bytes: usize) -> Self {
        Self { bytes: vec![0u8; total_bytes], cursor: 0 }
    }
    fn put(&mut self, slice: &[u8]) {
        self.bytes[self.cursor..self.cursor + slice.len()].copy_from_slice(slice);
        self.cursor += slice.len();
    }
    fn put_u8(&mut self, value: u8) {
        self.put(&[value]);
    }
    fn put_u16(&mut self, value: u16) {
        self.put(&value.to_le_bytes());
    }
    fn put_u32(&mut self, value: u32) {
        self.put(&value.to_le_bytes());
    }
    fn put_six_byte_unsigned(&mut self, value: u64) {
        assert!(value < (1u64 << 48), "48 位字段装不下 {value}");
        self.put(&value.to_le_bytes()[..6]);
    }
    fn put_u64(&mut self, value: u64) {
        self.put(&value.to_le_bytes());
    }
    fn skip(&mut self, byte_count: usize) {
        self.cursor += byte_count;
    }
    fn position(&self) -> usize {
        self.cursor
    }
    fn assert_position(&self, expected: u64, what: &str) {
        assert_eq!(self.cursor as u64, expected, "{what} 的偏移与字段表不符");
    }
}

struct ByteReader<'bytes> {
    bytes: &'bytes [u8],
    cursor: usize,
}

impl<'bytes> ByteReader<'bytes> {
    fn new(bytes: &'bytes [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }
    fn at(bytes: &'bytes [u8], offset: usize) -> Self {
        Self { bytes, cursor: offset }
    }
    fn take(&mut self, byte_count: usize) -> &'bytes [u8] {
        let slice = &self.bytes[self.cursor..self.cursor + byte_count];
        self.cursor += byte_count;
        slice
    }
    fn get_u8(&mut self) -> u8 {
        self.take(1)[0]
    }
    fn get_u16(&mut self) -> u16 {
        u16::from_le_bytes(self.take(2).try_into().expect("切了 2 字节"))
    }
    fn get_u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take(4).try_into().expect("切了 4 字节"))
    }
    fn get_six_byte_unsigned(&mut self) -> u64 {
        let mut eight = [0u8; 8];
        eight[..6].copy_from_slice(self.take(6));
        u64::from_le_bytes(eight)
    }
    fn get_u64(&mut self) -> u64 {
        u64::from_le_bytes(self.take(8).try_into().expect("切了 8 字节"))
    }
    fn skip(&mut self, byte_count: usize) {
        self.cursor += byte_count;
    }
}

// ───────────────────────── 校验和：CRC32C（D19 已定项 2 / D23 已定项 11）与 32 字节摘要（算法未定，装置取 SHA-256） ─────────────────────────

const CASTAGNOLI_POLYNOMIAL_REFLECTED: u32 = 0x82F6_3B78;

fn castagnoli_tables() -> &'static [[u32; 256]; 8] {
    static TABLES: std::sync::OnceLock<[[u32; 256]; 8]> = std::sync::OnceLock::new();
    TABLES.get_or_init(|| {
        let mut tables = [[0u32; 256]; 8];
        for byte_value in 0..256u32 {
            let mut remainder = byte_value;
            for _ in 0..8 {
                remainder = if remainder & 1 == 1 { (remainder >> 1) ^ CASTAGNOLI_POLYNOMIAL_REFLECTED } else { remainder >> 1 };
            }
            tables[0][byte_value as usize] = remainder;
        }
        for byte_value in 0..256usize {
            let mut remainder = tables[0][byte_value];
            for table_index in 1..8 {
                remainder = tables[0][(remainder & 0xff) as usize] ^ (remainder >> 8);
                tables[table_index][byte_value] = remainder;
            }
        }
        tables
    })
}

/// 完整 32 位 CRC32C（Castagnoli，反射，初值与输出异或都是全 1），slicing-by-8。
fn castagnoli_crc32(bytes: &[u8]) -> u32 {
    let tables = castagnoli_tables();
    let mut remainder = !0u32;
    let mut chunks = bytes.chunks_exact(8);
    for chunk in &mut chunks {
        let low = u32::from_le_bytes(chunk[..4].try_into().expect("切了 4 字节")) ^ remainder;
        let high = u32::from_le_bytes(chunk[4..].try_into().expect("切了 4 字节"));
        remainder = tables[7][(low & 0xff) as usize]
            ^ tables[6][((low >> 8) & 0xff) as usize]
            ^ tables[5][((low >> 16) & 0xff) as usize]
            ^ tables[4][(low >> 24) as usize]
            ^ tables[3][(high & 0xff) as usize]
            ^ tables[2][((high >> 8) & 0xff) as usize]
            ^ tables[1][((high >> 16) & 0xff) as usize]
            ^ tables[0][(high >> 24) as usize];
    }
    for &byte in chunks.remainder() {
        remainder = tables[0][((remainder ^ u32::from(byte)) & 0xff) as usize] ^ (remainder >> 8);
    }
    !remainder
}

/// 「校验和字段自身按 0 参与」（I-2.4）：把 `[field_offset, field_offset + 32)` 清零后对 `[0, cover_end)` 求校验和。
/// D18（块里携带什么信息） 已定项 17（2026-09-13 用户定案）：32 字节的校验和字段里放 CRC32C 4 字节 + 28 字节零，
/// 根记录自证校验和、超级块整槽校验和、journal `header_csum` 同口径——此前装置取 SHA-256，那是 gap G5，已收口。
fn wide_checksum_with_field_zeroed(bytes: &[u8], cover_end: usize, field_offset: usize) -> [u8; 32] {
    let mut covered = bytes[..cover_end].to_vec();
    covered[field_offset..field_offset + WIDE_CHECKSUM_BYTES as usize].fill(0);
    let mut field = [0u8; 32];
    field[..WIDE_CHECKSUM_CRC_BYTES].copy_from_slice(&castagnoli_crc32(&covered).to_le_bytes());
    field
}

// ───────────────────────── 设备、录制器、崩溃镜像 ─────────────────────────

/// 稀疏镜像：按 512 字节扇区存，没写过的扇区读出来恒为 0（mkfs「整环写 0」在这个模型里等价于不写）。
#[derive(Clone, Default)]
struct SparseDevice {
    sectors: BTreeMap<u64, [u8; PHYSICAL_BLOCK_BYTES as usize]>,
}

impl SparseDevice {
    fn read(&self, offset: DeviceOffset, length: usize) -> Vec<u8> {
        assert_eq!(offset.0 % PHYSICAL_BLOCK_BYTES, 0, "读要按扇区对齐");
        assert_eq!(length as u64 % PHYSICAL_BLOCK_BYTES, 0, "读长度要是整扇区");
        let mut out = vec![0u8; length];
        let first_sector = offset.0 / PHYSICAL_BLOCK_BYTES;
        for sector_index in 0..(length as u64 / PHYSICAL_BLOCK_BYTES) {
            if let Some(sector) = self.sectors.get(&(first_sector + sector_index)) {
                let start = (sector_index * PHYSICAL_BLOCK_BYTES) as usize;
                out[start..start + PHYSICAL_BLOCK_BYTES as usize].copy_from_slice(sector);
            }
        }
        out
    }
    fn write(&mut self, offset: DeviceOffset, bytes: &[u8]) {
        assert_eq!(offset.0 % PHYSICAL_BLOCK_BYTES, 0, "写要按扇区对齐");
        assert_eq!(bytes.len() as u64 % PHYSICAL_BLOCK_BYTES, 0, "写长度要是整扇区");
        let first_sector = offset.0 / PHYSICAL_BLOCK_BYTES;
        for (sector_index, chunk) in bytes.chunks_exact(PHYSICAL_BLOCK_BYTES as usize).enumerate() {
            self.sectors.insert(first_sector + sector_index as u64, chunk.try_into().expect("整扇区"));
        }
    }
    fn written_sectors_in(&self, offset: DeviceOffset, length: u64) -> Vec<u64> {
        let first_sector = offset.0 / PHYSICAL_BLOCK_BYTES;
        let end_sector = (offset.0 + length) / PHYSICAL_BLOCK_BYTES;
        self.sectors.range(first_sector..end_sector).map(|(sector, _)| *sector).collect()
    }
}

/// 恢复路径只通过这个 trait 读盘。两个实现：完整的池，与层 0 枚举出的崩溃镜像。
trait BlockReader {
    fn device_count(&self) -> usize;
    fn read(&self, device: DeviceIdentity, offset: DeviceOffset, length: usize) -> Vec<u8>;
    /// 扫描环时的优化：范围内有过写入的扇区号。没写过的扇区读出来恒为 0，一条全 0 的记录 magic 必不匹配，
    /// 所以跳过它们与逐扇区读零判 magic 等价。
    fn written_sectors(&self, device: DeviceIdentity, offset: DeviceOffset, length: u64) -> Vec<u64>;
}

#[derive(Clone)]
struct Pool {
    devices: Vec<SparseDevice>,
}

impl BlockReader for Pool {
    fn device_count(&self) -> usize {
        self.devices.len()
    }
    fn read(&self, device: DeviceIdentity, offset: DeviceOffset, length: usize) -> Vec<u8> {
        self.devices[device.0 as usize].read(offset, length)
    }
    fn written_sectors(&self, device: DeviceIdentity, offset: DeviceOffset, length: u64) -> Vec<u64> {
        self.devices[device.0 as usize].written_sectors_in(offset, length)
    }
}

/// 录制器里一次**写**的步骤种类。D17（实现分层与第三方管道） 已定项 2 把结构等价类定成
/// 「段边界位置 + 每段步骤种类集合」，所以步骤种类要是机器可读的封闭枚举，不能是自由文本标签（C316 ②）。
/// 没有通配臂：以后多一种写步骤，每一处 match 都编译不过。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
enum StepKind {
    /// 单元区里一次整单元写（码 1 / 码 2 / 码 3 都是同一种步骤：一个 16 KiB 或 32 KiB 落点写满）。
    UnitWrite,
    /// journal 环里一条 4 KiB 记录。
    JournalRecord,
    /// 根环槽的一次 FUA 写（D16（发布语义） 已定项 7：只有这一步等落盘才发下一条）。
    RootRecordFua,
    /// 超级块槽的一次写。
    SuperblockSlot,
}

impl StepKind {
    /// 发到输出里的名字，lowercase snake_case，一个种类一个。
    fn tag(self) -> &'static str {
        match self {
            StepKind::UnitWrite => "unit_write",
            StepKind::JournalRecord => "journal_record",
            StepKind::RootRecordFua => "root_record_fua",
            StepKind::SuperblockSlot => "superblock_slot",
        }
    }
    /// FUA 由步骤种类决定，不再是调用点各传各的布尔：超级块槽写不可能是 FUA，这样它写不出来。
    fn is_fua(self) -> bool {
        match self {
            StepKind::RootRecordFua => true,
            StepKind::UnitWrite | StepKind::JournalRecord | StepKind::SuperblockSlot => false,
        }
    }
}

/// 录制到的一步的种类：写请求带着它那四种之一，屏障自己是一种。
/// 段序列说的「步骤种类」是这个字母表——屏障是段边界，它也要有名字。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
enum RecordedStepKind {
    Write(StepKind),
    Barrier,
}

impl RecordedStepKind {
    fn of(operation: &RecordedOperation) -> Self {
        match operation {
            RecordedOperation::Write(write) => RecordedStepKind::Write(write.kind),
            RecordedOperation::Barrier => RecordedStepKind::Barrier,
        }
    }
    fn tag(self) -> &'static str {
        match self {
            RecordedStepKind::Write(kind) => kind.tag(),
            RecordedStepKind::Barrier => "barrier",
        }
    }
}

#[derive(Clone, Debug)]
struct WriteRequest {
    device: DeviceIdentity,
    offset: DeviceOffset,
    bytes: Vec<u8>,
    kind: StepKind,
}

impl WriteRequest {
    fn is_fua(&self) -> bool {
        self.kind.is_fua()
    }
}

#[derive(Clone, Debug)]
enum RecordedOperation {
    Write(WriteRequest),
    Barrier,
}

/// 录制器包在池外面：每个写请求与屏障记成一条流，给层 0 枚举用。
struct RecordingPool {
    pool: Pool,
    operations: Vec<RecordedOperation>,
}

impl RecordingPool {
    fn write(&mut self, device: DeviceIdentity, offset: DeviceOffset, bytes: &[u8], kind: StepKind) {
        self.pool.devices[device.0 as usize].write(offset, bytes);
        self.operations.push(RecordedOperation::Write(WriteRequest { device, offset, bytes: bytes.to_vec(), kind }));
    }
    fn barrier(&mut self) {
        self.operations.push(RecordedOperation::Barrier);
    }
}

/// 一个崩溃状态的镜像：崩溃前的池 + 这次事务的写请求里被判持久的那个子集。没持久的位置读到旧字节（D13 已定项 4）。
struct CrashImage<'base> {
    base: &'base Pool,
    writes: &'base [WriteRequest],
    persisted: Vec<bool>,
}

impl BlockReader for CrashImage<'_> {
    fn device_count(&self) -> usize {
        self.base.devices.len()
    }
    fn read(&self, device: DeviceIdentity, offset: DeviceOffset, length: usize) -> Vec<u8> {
        let mut out = self.base.read(device, offset, length);
        let read_start = offset.0;
        let read_end = offset.0 + length as u64;
        for (write, is_persisted) in self.writes.iter().zip(&self.persisted) {
            if !is_persisted || write.device != device {
                continue;
            }
            let write_start = write.offset.0;
            let write_end = write_start + write.bytes.len() as u64;
            let overlap_start = read_start.max(write_start);
            let overlap_end = read_end.min(write_end);
            if overlap_start >= overlap_end {
                continue;
            }
            let destination = (overlap_start - read_start) as usize;
            let source = (overlap_start - write_start) as usize;
            let overlap_length = (overlap_end - overlap_start) as usize;
            out[destination..destination + overlap_length].copy_from_slice(&write.bytes[source..source + overlap_length]);
        }
        out
    }
    fn written_sectors(&self, device: DeviceIdentity, offset: DeviceOffset, length: u64) -> Vec<u64> {
        let mut sectors = self.base.written_sectors(device, offset, length);
        for (write, is_persisted) in self.writes.iter().zip(&self.persisted) {
            if !is_persisted || write.device != device {
                continue;
            }
            let first = write.offset.0 / PHYSICAL_BLOCK_BYTES;
            let count = write.bytes.len() as u64 / PHYSICAL_BLOCK_BYTES;
            for sector in first..first + count {
                if sector * PHYSICAL_BLOCK_BYTES >= offset.0 && sector * PHYSICAL_BLOCK_BYTES < offset.0 + length {
                    sectors.push(sector);
                }
            }
        }
        sectors.sort_unstable();
        sectors.dedup();
        sectors
    }
}

// ───────────────────────── 盘上结构：位置条目、指针、单元头 ─────────────────────────

/// D19（块指针的结构与宽度预算） 已定项 4：设备 4 + 16 KiB 槽号 6 + 密文校验和 4（无加密时是整单元 CRC32C，字节表三预想）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct LocationEntry {
    device: DeviceIdentity,
    slot: SlotNumber,
    unit_checksum: u32,
}

impl LocationEntry {
    fn write_to(&self, writer: &mut ByteWriter) {
        writer.put_u32(self.device.0);
        writer.put_six_byte_unsigned(self.slot.0);
        writer.put_u32(self.unit_checksum);
    }
    fn read_from(reader: &mut ByteReader) -> Self {
        Self { device: DeviceIdentity(reader.get_u32()), slot: SlotNumber(reader.get_six_byte_unsigned()), unit_checksum: reader.get_u32() }
    }
}

/// 指针头部 50（D19 已定项 7 / 已定项 11 的偏移表）：MAC 16（0）+ nonce 12（16）+ 算法类型 1（28）
/// + 压缩算法码 1（29）+ 压后长度 2（30）+ extent 距起点偏移 2（32）+ 出生树 8（34）+ 出生 txg 8（42）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct PointerHead {
    birth_tree: TreeIdentifier,
    birth_txg: CheckpointTxg,
}

impl PointerHead {
    fn write_to(&self, writer: &mut ByteWriter) {
        let start = writer.position();
        writer.skip(16 + 12); // MAC 与 nonce：第一版留位、全 0（D9（加密） 已定项 2 / 已定项 4）
        writer.put_u8(0); // 算法类型：未加密
        writer.put_u8(POINTER_COMPRESSION_CODE_NONE);
        writer.put_u16(POINTER_COMPRESSED_LENGTH_NONE);
        writer.put_u16(0); // extent 距起点偏移：第一版恒 0，切分是预留特性
        assert_eq!(writer.position() - start, 34, "出生树要落在指针头部偏移 34");
        writer.put_u64(self.birth_tree.0);
        writer.put_u64(self.birth_txg.0);
        assert_eq!(writer.position() - start, POINTER_HEAD_BYTES as usize);
    }
    fn read_from(reader: &mut ByteReader) -> Self {
        reader.skip(16 + 12 + 1 + 1 + 2 + 2);
        Self { birth_tree: TreeIdentifier(reader.get_u64()), birth_txg: CheckpointTxg(reader.get_u64()) }
    }
}

/// 指向码 2 / 码 3 的指针 86（D19 已定项 8）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct NodePointer {
    head: PointerHead,
    locations: [LocationEntry; 2],
    instance: InstanceGeneration,
    birth_sequence: BirthSequence,
}

impl NodePointer {
    /// 空树的根：位置条目、实例代号、出生序号全零（D6 已定项 2 / D5 已定项 6 的 day-1 注册树在第一个事务里就是这样）。
    fn empty_root() -> NodePointer {
        NodePointer {
            head: PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE), birth_txg: CheckpointTxg(0) },
            locations: [
                LocationEntry { device: DeviceIdentity(0), slot: SlotNumber(0), unit_checksum: 0 },
                LocationEntry { device: DeviceIdentity(0), slot: SlotNumber(0), unit_checksum: 0 },
            ],
            instance: InstanceGeneration(0),
            birth_sequence: BirthSequence(0),
        }
    }
    fn is_empty_root(&self) -> bool {
        self.locations.iter().all(|location| location.slot.0 == 0 && location.unit_checksum == 0) && self.birth_sequence.0 == 0
    }
    fn write_to(&self, writer: &mut ByteWriter) {
        let start = writer.position();
        self.head.write_to(writer);
        for location in &self.locations {
            location.write_to(writer);
        }
        writer.put_u32(self.instance.0);
        writer.put_u32(self.birth_sequence.0);
        assert_eq!(writer.position() - start, NODE_POINTER_BYTES as usize);
    }
    fn read_from(reader: &mut ByteReader) -> Self {
        let head = PointerHead::read_from(reader);
        let locations = [LocationEntry::read_from(reader), LocationEntry::read_from(reader)];
        Self { head, locations, instance: InstanceGeneration(reader.get_u32()), birth_sequence: BirthSequence(reader.get_u32()) }
    }
}

/// 指向码 1 的指针 88（D19 已定项 8）：头部 + 位置条目 × 2 + 写序 10。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct DataPointer {
    head: PointerHead,
    locations: [LocationEntry; 2],
    write_order: WriteOrder,
}

impl DataPointer {
    fn write_to(&self, writer: &mut ByteWriter) {
        let start = writer.position();
        self.head.write_to(writer);
        for location in &self.locations {
            location.write_to(writer);
        }
        writer.put_u32(self.write_order.instance.0);
        writer.put_six_byte_unsigned(self.write_order.transaction.0);
        assert_eq!(writer.position() - start, DATA_POINTER_BYTES as usize);
    }
    fn read_from(reader: &mut ByteReader) -> Self {
        let head = PointerHead::read_from(reader);
        let locations = [LocationEntry::read_from(reader), LocationEntry::read_from(reader)];
        let instance = InstanceGeneration(reader.get_u32());
        let transaction = TransactionNumber(reader.get_six_byte_unsigned());
        Self { head, locations, write_order: WriteOrder { instance, transaction } }
    }
}

/// D18 已定项 3 的五元组（33 字节）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct DataUnitIdentity {
    tree: TreeIdentifier,
    object: u64,
    object_birth: CheckpointTxg,
    anchor_offset: u64,
}

/// 共同前缀 42：magic 4 + 版本 2 + 类标签 1 + flags 1 + 声明长度 2 + 头校验和 32。头校验和最后填。
fn write_common_prefix(writer: &mut ByteWriter, unit_class: u8, declared_length: u16) {
    writer.put(&UNIT_MAGIC);
    writer.put_u16(FORMAT_VERSION);
    writer.put_u8(unit_class);
    writer.put_u8(0);
    writer.put_u16(declared_length);
    writer.skip(WIDE_CHECKSUM_BYTES as usize);
    writer.assert_position(COMMON_PREFIX_BYTES, "共同前缀");
}

const HEADER_CHECKSUM_OFFSET: usize = 10;

/// 头校验和覆盖偏移 0 到该类明文头末尾（I-2.4），自身按 0 参与。
fn seal_header_checksum(bytes: &mut [u8], header_end: usize) {
    let digest = wide_checksum_with_field_zeroed(bytes, header_end, HEADER_CHECKSUM_OFFSET);
    bytes[HEADER_CHECKSUM_OFFSET..HEADER_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
}

fn header_checksum_holds(bytes: &[u8], header_end: usize) -> bool {
    wide_checksum_with_field_zeroed(bytes, header_end, HEADER_CHECKSUM_OFFSET) == bytes[HEADER_CHECKSUM_OFFSET..HEADER_CHECKSUM_OFFSET + 32]
}

/// fsid 在单元头里是 8 字节（D18 已定项 7）；字节表二预想取超级块 fsid 的低 8 字节。
fn unit_fsid(fsid: &[u8; 16]) -> u64 {
    u64::from_le_bytes(fsid[..8].try_into().expect("切了 8 字节"))
}

/// 码 1 数据单元 32768：头 105 + 预留 29 + 载荷从 134 起（声明长度之后补 0，补齐参与 CRC——I-2.3）。
fn build_data_unit(identity: DataUnitIdentity, birth_txg: CheckpointTxg, fsid: &[u8; 16], write_order: WriteOrder, payload: &[u8]) -> Vec<u8> {
    let payload_start = (DATA_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES) as usize;
    assert!(payload.len() <= DATA_UNIT_BYTES as usize - payload_start, "载荷装不进一个单元");
    let mut writer = ByteWriter::new(DATA_UNIT_BYTES as usize);
    write_common_prefix(&mut writer, UNIT_CLASS_DATA, u16::try_from(payload.len()).expect("声明长度 2 字节"));
    writer.put_u8(UNIT_CLASS_DATA);
    writer.put_u64(identity.tree.0);
    writer.put_u64(identity.object);
    writer.put_u64(identity.object_birth.0);
    writer.put_u64(identity.anchor_offset);
    writer.put_u64(birth_txg.0);
    writer.put_u64(unit_fsid(fsid));
    writer.put_u32(write_order.instance.0);
    writer.put_six_byte_unsigned(write_order.transaction.0);
    let payload_crc_offset = writer.position();
    writer.skip(4);
    writer.assert_position(DATA_UNIT_HEADER_BYTES, "数据单元头");
    writer.skip(NONCE_MAC_RESERVED_BYTES as usize);
    writer.put(payload);
    let mut bytes = writer.bytes;
    // 载荷 CRC 覆盖偏移 105 到单元末尾（字节表二：含预留位与补齐）。
    let payload_crc = castagnoli_crc32(&bytes[DATA_UNIT_HEADER_BYTES as usize..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    seal_header_checksum(&mut bytes, DATA_UNIT_HEADER_BYTES as usize);
    bytes
}

/// 码 3 打包记录单元的类身份四元组（D18 已定项 11）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct PackedIdentity {
    birth_tree: TreeIdentifier,
    record_type: u16,
    container: u64,
    container_birth: CheckpointTxg,
}

/// 码 3 打包记录单元 32768：头 107（偏移照 D18 已定项 11 的表）+ 预留 29 + 记录区从 136 起。
fn build_packed_unit(
    identity: PackedIdentity,
    record_width: u16,
    records: &[Vec<u8>],
    birth_txg: CheckpointTxg,
    fsid: &[u8; 16],
    write_order: WriteOrder,
    birth_sequence: BirthSequence,
) -> Vec<u8> {
    let record_area_start = (PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES) as usize;
    let declared_length = u16::try_from(records.len() * record_width as usize).expect("声明长度 2 字节");
    let mut writer = ByteWriter::new(DATA_UNIT_BYTES as usize);
    write_common_prefix(&mut writer, UNIT_CLASS_PACKED, declared_length);
    writer.put_u8(UNIT_CLASS_PACKED);
    writer.put_u64(identity.birth_tree.0);
    writer.put_u16(identity.record_type);
    writer.put_u64(identity.container);
    writer.put_u64(identity.container_birth.0);
    writer.assert_position(69, "码 3 记录数");
    writer.put_u16(u16::try_from(records.len()).expect("记录数 2 字节"));
    writer.put_u16(record_width);
    writer.assert_position(73, "码 3 诞生代号");
    writer.put_u64(birth_txg.0);
    writer.put_u64(unit_fsid(fsid));
    writer.assert_position(89, "码 3 载荷校验和");
    let payload_crc_offset = writer.position();
    writer.skip(4);
    writer.assert_position(93, "码 3 写序");
    writer.put_u32(write_order.instance.0);
    writer.put_six_byte_unsigned(write_order.transaction.0);
    writer.assert_position(103, "码 3 出生序号");
    writer.put_u32(birth_sequence.0);
    writer.assert_position(PACKED_UNIT_HEADER_BYTES, "码 3 头");
    writer.skip(NONCE_MAC_RESERVED_BYTES as usize);
    for record in records {
        assert_eq!(record.len(), record_width as usize, "记录宽与头里写的不一致");
        writer.put(record);
    }
    let _ = record_area_start;
    let mut bytes = writer.bytes;
    // 载荷校验和覆盖偏移 107 到单元末尾（D18 已定项 11 逐字：nonce / MAC 预留位 + 记录区 + 补齐）。
    let payload_crc = castagnoli_crc32(&bytes[PACKED_UNIT_HEADER_BYTES as usize..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    seal_header_checksum(&mut bytes, PACKED_UNIT_HEADER_BYTES as usize);
    bytes
}

/// 码 2 索引节点头的宽度：86 + 2 × key 宽（D8（核心索引结构） 已定项 11）；含 29 字节预留位则是 115 + 2 × key 宽。
fn index_node_header_bytes(key_width: usize) -> usize {
    INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE as usize + 2 * key_width
}

/// 码 2 索引节点 16384，偏移照 D18（块里携带什么信息） 已定项 18 的表（2026-09-14 用户定案把 key 宽挪到 51）：
/// 42 树 ID 8 / 50 层级 1 / **51 key 宽 1** / 52 key 区间 2k / 52+2k 诞生代号 8 / 60+2k fsid 8
/// / 68+2k 写序 4（只实例代号）/ 72+2k 出生序号 4 / 76+2k 载荷 CRC 4 / 80+2k 预留 2 / 82+2k 条目数 2
/// / 84+2k 条目宽 2 / 86+2k 预留位 29 / 115+2k 条目区。头校验和罩 0..86+2k，载荷 CRC 从 86+2k 到单元末尾。
/// 条目一律定宽（中央映射的条目 2026-09-14 回到一宽 55），所以条目宽是个单值、声明长度 = 条目数 × 条目宽。
#[allow(clippy::too_many_arguments, reason = "字段表就是这么多段，收成结构体只会多一层没人验的名字")]
fn build_index_node(
    tree: TreeIdentifier,
    level: u8,
    key_width: usize,
    smallest_key: &[u8],
    largest_key: &[u8],
    birth_txg: CheckpointTxg,
    fsid: &[u8; 16],
    instance: InstanceGeneration,
    birth_sequence: BirthSequence,
    entry_width: u16,
    entries: &[Vec<u8>],
) -> Vec<u8> {
    assert_eq!(smallest_key.len(), key_width);
    assert_eq!(largest_key.len(), key_width);
    for entry in entries {
        assert_eq!(entry.len(), entry_width as usize, "条目宽与节点里写的不一致");
        assert!(entry.len() >= key_width, "条目里 key 一律是前 key 宽 个字节（D8 已定项 11，2026-09-14 用户定案）");
    }
    let header_end = index_node_header_bytes(key_width);
    let entries_start = header_end + NONCE_MAC_RESERVED_BYTES as usize;
    // D8（核心索引结构） 已定项 11：声明长度 = 条目数 × 条目宽。
    let declared_length: usize = entries.len() * entry_width as usize;
    assert!(entries_start + declared_length <= NODE_BYTES as usize, "条目装不进一个节点");
    let mut writer = ByteWriter::new(NODE_BYTES as usize);
    write_common_prefix(&mut writer, UNIT_CLASS_INDEX_NODE, u16::try_from(declared_length).expect("声明长度 2 字节"));
    writer.put_u64(tree.0);
    writer.put_u8(level);
    writer.assert_position(INDEX_NODE_KEY_WIDTH_OFFSET as u64, "码 2 的 key 宽字段");
    writer.put_u8(u8::try_from(key_width).expect("key 宽 1 字节"));
    writer.put(smallest_key);
    writer.put(largest_key);
    writer.put_u64(birth_txg.0);
    writer.put_u64(unit_fsid(fsid));
    writer.put_u32(instance.0);
    writer.put_u32(birth_sequence.0);
    let payload_crc_offset = writer.position();
    writer.skip(4);
    writer.skip(2);
    writer.put_u16(u16::try_from(entries.len()).expect("条目数 2 字节"));
    writer.put_u16(entry_width);
    writer.assert_position(header_end as u64, "码 2 头");
    writer.skip(NONCE_MAC_RESERVED_BYTES as usize);
    for entry in entries {
        writer.put(entry);
    }
    let mut bytes = writer.bytes;
    // 载荷 CRC 覆盖从头末尾（含 28 字节预留位）到单元末尾（D18 已定项 7 索引节点类：口径照码 1 与码 3）。
    let payload_crc = castagnoli_crc32(&bytes[header_end..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    seal_header_checksum(&mut bytes, header_end);
    bytes
}

/// 解出来的码 2 节点头。key 宽从 2026-09-14 起住**偏移 51**（D18（块里携带什么信息） 已定项 18），
/// 定位它不再需要先知道 k ⇒ 扫描期按 `[0, 51)` 这段定长前缀读出 k、一次定出头末端（gap G2 收口）。
#[derive(Clone, Debug)]
struct IndexNodeHeader {
    tree: TreeIdentifier,
    level: u8,
    key_width: usize,
    smallest_key: Vec<u8>,
    largest_key: Vec<u8>,
    birth_txg: CheckpointTxg,
    fsid: u64,
    instance: InstanceGeneration,
    birth_sequence: BirthSequence,
    entries: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum UnitError {
    BadMagic,
    BadVersion,
    NonZeroFlags,
    WrongClass { expected: u8, found: u8 },
    HeaderChecksum,
    PayloadChecksum,
    Structure(&'static str),
}

fn check_common_prefix(bytes: &[u8], expected_class: u8) -> Result<u16, UnitError> {
    if bytes[..4] != UNIT_MAGIC {
        return Err(UnitError::BadMagic);
    }
    let mut reader = ByteReader::at(bytes, 4);
    if reader.get_u16() != FORMAT_VERSION {
        return Err(UnitError::BadVersion);
    }
    let found_class = reader.get_u8();
    if reader.get_u8() != 0 {
        return Err(UnitError::NonZeroFlags);
    }
    if found_class != expected_class {
        return Err(UnitError::WrongClass { expected: expected_class, found: found_class });
    }
    Ok(reader.get_u16())
}

/// 解码 2 节点：key 宽住**偏移 51**（D18 已定项 18，2026-09-14 用户定案），它自己的偏移不依赖 k
/// ⇒ 先读 `[0, 51)` 这段定长前缀里的 k、一次定出头末端，再核头校验和。调用方不必先知道这是哪棵树。
fn parse_index_node(bytes: &[u8]) -> Result<IndexNodeHeader, UnitError> {
    let declared_length = check_common_prefix(bytes, UNIT_CLASS_INDEX_NODE)?;
    let key_width = usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET]);
    let header_end = index_node_header_bytes(key_width);
    if header_end + NONCE_MAC_RESERVED_BYTES as usize > bytes.len() {
        return Err(UnitError::Structure("自述的 key 宽让头越出单元"));
    }
    if !header_checksum_holds(bytes, header_end) {
        return Err(UnitError::HeaderChecksum);
    }
    let mut reader = ByteReader::at(bytes, COMMON_PREFIX_BYTES as usize);
    let tree = TreeIdentifier(reader.get_u64());
    let level = reader.get_u8();
    let declared_key_width = usize::from(reader.get_u8());
    let smallest_key = reader.take(key_width).to_vec();
    let largest_key = reader.take(key_width).to_vec();
    let birth_txg = CheckpointTxg(reader.get_u64());
    let fsid = reader.get_u64();
    let instance = InstanceGeneration(reader.get_u32());
    let birth_sequence = BirthSequence(reader.get_u32());
    let payload_crc = reader.get_u32();
    reader.skip(2);
    let entry_count = reader.get_u16() as usize;
    let entry_width = reader.get_u16() as usize;
    if castagnoli_crc32(&bytes[header_end..]) != payload_crc {
        return Err(UnitError::PayloadChecksum);
    }
    // D8（核心索引结构） 已定项 11：条目定宽连续排列，声明长度 = 条目数 × 条目宽。
    if entry_width < key_width || entry_count * entry_width != declared_length as usize {
        return Err(UnitError::Structure("声明长度与条目数 × 条目宽对不上"));
    }
    let mut payload = ByteReader::at(bytes, header_end + NONCE_MAC_RESERVED_BYTES as usize);
    let mut entries: Vec<Vec<u8>> = Vec::with_capacity(entry_count);
    for _ in 0..entry_count {
        if payload.cursor + entry_width > bytes.len() {
            return Err(UnitError::Structure("条目区越界"));
        }
        entries.push(payload.take(entry_width).to_vec());
    }
    // D18（块里携带什么信息） 已定项 18：条目区之后的补齐区恒 0 且参与载荷 CRC。
    if bytes[payload.cursor..].iter().any(|&byte| byte != 0) {
        return Err(UnitError::Structure("条目区之后的补齐区非零")); // I-2.3 射程 2026-09-13 扩到码 2
    }
    Ok(IndexNodeHeader { tree, level, key_width: declared_key_width, smallest_key, largest_key, birth_txg, fsid, instance, birth_sequence, entries })
}

#[derive(Clone, Debug)]
struct PackedUnitHeader {
    identity: PackedIdentity,
    record_width: u16,
    birth_txg: CheckpointTxg,
    fsid: u64,
    write_order: WriteOrder,
    birth_sequence: BirthSequence,
    records: Vec<Vec<u8>>,
}

fn parse_packed_unit(bytes: &[u8]) -> Result<PackedUnitHeader, UnitError> {
    let declared_length = check_common_prefix(bytes, UNIT_CLASS_PACKED)?;
    if !header_checksum_holds(bytes, PACKED_UNIT_HEADER_BYTES as usize) {
        return Err(UnitError::HeaderChecksum);
    }
    let mut reader = ByteReader::at(bytes, COMMON_PREFIX_BYTES as usize);
    if reader.get_u8() != UNIT_CLASS_PACKED {
        return Err(UnitError::Structure("类身份段首字节不等于偏移 6"));
    }
    let identity = PackedIdentity {
        birth_tree: TreeIdentifier(reader.get_u64()),
        record_type: reader.get_u16(),
        container: reader.get_u64(),
        container_birth: CheckpointTxg(reader.get_u64()),
    };
    let record_count = reader.get_u16() as usize;
    let record_width = reader.get_u16();
    let birth_txg = CheckpointTxg(reader.get_u64());
    let fsid = reader.get_u64();
    let payload_crc = reader.get_u32();
    let write_order = WriteOrder { instance: InstanceGeneration(reader.get_u32()), transaction: TransactionNumber(reader.get_six_byte_unsigned()) };
    let birth_sequence = BirthSequence(reader.get_u32());
    // I-1.7 的判定顺序：边界先于载荷校验和。
    if record_count * record_width as usize > declared_length as usize || declared_length as usize > (DATA_UNIT_BYTES - PACKED_UNIT_HEADER_BYTES) as usize {
        return Err(UnitError::Structure("记录数 × 记录宽 > 声明长度"));
    }
    if castagnoli_crc32(&bytes[PACKED_UNIT_HEADER_BYTES as usize..]) != payload_crc {
        return Err(UnitError::PayloadChecksum);
    }
    let mut record_reader = ByteReader::at(bytes, (PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES) as usize);
    let records = (0..record_count).map(|_| record_reader.take(record_width as usize).to_vec()).collect();
    Ok(PackedUnitHeader { identity, record_width, birth_txg, fsid, write_order, birth_sequence, records })
}

#[derive(Clone, Debug)]
struct DataUnitHeader {
    identity: DataUnitIdentity,
    declared_length: u16,
    birth_txg: CheckpointTxg,
    fsid: u64,
    write_order: WriteOrder,
}

fn parse_data_unit(bytes: &[u8]) -> Result<DataUnitHeader, UnitError> {
    let declared_length = check_common_prefix(bytes, UNIT_CLASS_DATA)?;
    if !header_checksum_holds(bytes, DATA_UNIT_HEADER_BYTES as usize) {
        return Err(UnitError::HeaderChecksum);
    }
    let mut reader = ByteReader::at(bytes, COMMON_PREFIX_BYTES as usize);
    if reader.get_u8() != UNIT_CLASS_DATA {
        return Err(UnitError::Structure("五元组首字节不等于偏移 6"));
    }
    let identity = DataUnitIdentity {
        tree: TreeIdentifier(reader.get_u64()),
        object: reader.get_u64(),
        object_birth: CheckpointTxg(reader.get_u64()),
        anchor_offset: reader.get_u64(),
    };
    let birth_txg = CheckpointTxg(reader.get_u64());
    let fsid = reader.get_u64();
    let write_order = WriteOrder { instance: InstanceGeneration(reader.get_u32()), transaction: TransactionNumber(reader.get_six_byte_unsigned()) };
    let payload_crc = reader.get_u32();
    if castagnoli_crc32(&bytes[DATA_UNIT_HEADER_BYTES as usize..]) != payload_crc {
        return Err(UnitError::PayloadChecksum);
    }
    Ok(DataUnitHeader { identity, declared_length, birth_txg, fsid, write_order })
}

fn data_unit_payload(bytes: &[u8], declared_length: u16) -> &[u8] {
    let start = (DATA_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES) as usize;
    &bytes[start..start + declared_length as usize]
}

// ───────────────────────── 根记录、超级块、journal 记录 ─────────────────────────

/// D22（单元原子性怎么合成） 已定项 7 的字段表，371 字节，字段序照那张表（gap G4：字节表七的行序与它不同，两处都没写偏移）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct RootRecord {
    fsid: [u8; 16],
    instance: InstanceGeneration,
    checkpoint_txg: CheckpointTxg,
    tree_table: NodePointer,
    tree_identifier_watermark: u64,
    rollback_floor: CheckpointTxg,
    instance_table: NodePointer,
    /// D19（块指针的结构与宽度预算） 已定项 11（2026-09-13 用户定案）：中央映射树的根住根记录。
    mapping_root: NodePointer,
}

const ROOT_CHECKSUM_OFFSET: usize = 4 + 16 + 4 + 4 + 8 + NODE_POINTER_BYTES as usize + 8 + 8;

impl RootRecord {
    /// 写成一个判定宽度的槽：记录 371 字节，其余补 0；
    /// 自证校验和覆盖**整个 512 槽含补齐**、自身按 0 参与（D18（块里携带什么信息） 已定项 17，2026-09-13 用户定案）。
    fn to_slot(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(PHYSICAL_BLOCK_BYTES as usize);
        writer.put(&ROOT_MAGIC);
        writer.put(&self.fsid);
        writer.put_u32(0);
        writer.put_u32(self.instance.0);
        writer.put_u64(self.checkpoint_txg.0);
        self.tree_table.write_to(&mut writer);
        writer.put_u64(self.tree_identifier_watermark);
        writer.put_u64(self.rollback_floor.0);
        writer.assert_position(ROOT_CHECKSUM_OFFSET as u64, "根记录自证校验和");
        writer.skip(WIDE_CHECKSUM_BYTES as usize);
        self.instance_table.write_to(&mut writer);
        self.mapping_root.write_to(&mut writer);
        // D22 已定项 7 的表尾三段（2026-09-14 用户定案）：算法类型 1 + nonce 12 + MAC 16，第一版全 0 留位。
        writer.put_u8(0);
        writer.skip(12 + 16);
        writer.assert_position(ROOT_RECORD_BYTES, "根记录");
        let mut bytes = writer.bytes;
        let digest = wide_checksum_with_field_zeroed(&bytes, PHYSICAL_BLOCK_BYTES as usize, ROOT_CHECKSUM_OFFSET);
        bytes[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        bytes
    }

    fn parse_slot(bytes: &[u8], expected_fsid: &[u8; 16]) -> Option<Self> {
        if bytes[..4] != ROOT_MAGIC {
            return None;
        }
        if wide_checksum_with_field_zeroed(bytes, PHYSICAL_BLOCK_BYTES as usize, ROOT_CHECKSUM_OFFSET) != bytes[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32] {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 4);
        let fsid: [u8; 16] = reader.take(16).try_into().expect("切了 16 字节");
        if &fsid != expected_fsid {
            return None;
        }
        let _flags = reader.get_u32();
        let instance = InstanceGeneration(reader.get_u32());
        let checkpoint_txg = CheckpointTxg(reader.get_u64());
        let tree_table = NodePointer::read_from(&mut reader);
        let tree_identifier_watermark = reader.get_u64();
        let rollback_floor = CheckpointTxg(reader.get_u64());
        reader.skip(WIDE_CHECKSUM_BYTES as usize);
        let instance_table = NodePointer::read_from(&mut reader);
        let mapping_root = NodePointer::read_from(&mut reader);
        Some(Self { fsid, instance, checkpoint_txg, tree_table, tree_identifier_watermark, rollback_floor, instance_table, mapping_root })
    }
}

/// 超级块字段表（D22 已定项 9 + 已定项 15）：**481 字节**，512 槽内余 31。
/// 2026-09-13 用户定案去掉「间接目录单元指针 59」（第一版显式留白，不是留位），几何段加三个字段：
/// 单元区起始槽号 8（D3 已定项 10 ④）、mkfs 时的 io_min 4 与固定结构槽距 4（D2 已定项 19）；
/// 2026-09-14 用户定案再加四个：自举头的写入者身份 20 与校验和算法标识 1、
/// 几何段的 mkfs 时 physical_block_size 4 与扩展点声明值 N 4。
#[derive(Clone, PartialEq, Eq, Debug)]
struct Superblock {
    fsid: [u8; 16],
    this_device: DeviceIdentity,
    device_count: u32,
    slot_generation: u64,
    region_devices: [DeviceIdentity; 3],
    journal_tail: u64,
    journal_instance: InstanceGeneration,
}

/// magic 4 + 格式版本 2 + feature bits 96 + fsid 16 + 写入者身份 20 + 校验和算法标识 1 + 本盘设备号 4 + 设备数 4 + 槽世代号 8。
const SUPERBLOCK_CHECKSUM_OFFSET: usize = 4 + 2 + 96 + 16 + 20 + 1 + 4 + 4 + 8;
/// fsid 住自举头里 magic 4 + 格式版本 2 + feature bits 96 之后。
const SUPERBLOCK_FSID_OFFSET: usize = 4 + 2 + 96;
const SUPERBLOCK_REGION_DEVICES_OFFSET: usize = 379;
const SUPERBLOCK_TAIL_OFFSET: usize = 469;
/// D2（RAID 条带策略） 已定项 19：固定结构槽距 = max(4096, mkfs 时探测的 io_min)，两个数各占超级块一个 4 字节字段。
const FIXED_STRUCTURE_SLOT_SPACING: u32 = 4096;
const MKFS_MINIMUM_INPUT_OUTPUT_BYTES: u32 = 512;
/// D2（RAID 条带策略） 已定项 18：第一版超级块里 w_max 与 g 都写 4；g 挂载时按可写设备数夹取。
const SUPERBLOCK_MAXIMUM_WIDTH: u8 = 4;
const SUPERBLOCK_GROUP_SIZE: u8 = 4;

impl Superblock {
    fn to_slot(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(SUPERBLOCK_SLOT_BYTES as usize);
        writer.put(&SUPERBLOCK_MAGIC);
        writer.put_u16(FORMAT_VERSION);
        writer.put_u8(INCOMPAT_FIRST_SSD_LINE_BIT); // feature bits：incompat 位 0 = 第一条纯 SSD 布局线（D15 已定项 4，2026-09-13 用户定案）
        writer.skip(95); // 其余 incompat 位与 compat_ro / compat 两张位图全 0
        writer.assert_position(SUPERBLOCK_FSID_OFFSET as u64, "超级块 fsid");
        writer.put(&self.fsid);
        // 写入者身份（D22 已定项 9，2026-09-14 用户定案）：实现标识 16 字节 ASCII 零补齐 + 版本 4 字节。
        let mut writer_identity = [0u8; WRITER_IDENTITY_NAME_BYTES];
        writer_identity[..WRITER_IDENTITY_NAME.len()].copy_from_slice(WRITER_IDENTITY_NAME);
        writer.put(&writer_identity);
        writer.put_u32(WRITER_IDENTITY_VERSION);
        writer.put_u8(CHECKSUM_ALGORITHM_CRC32_CASTAGNOLI);
        writer.put_u32(self.this_device.0);
        writer.put_u32(self.device_count);
        writer.put_u64(self.slot_generation);
        writer.assert_position(SUPERBLOCK_CHECKSUM_OFFSET as u64, "超级块整槽校验和");
        writer.skip(WIDE_CHECKSUM_BYTES as usize);
        writer.skip(16 + 12 + 4); // 超级块 MAC、nonce 水位、KDF 标识（4，D22 已定项 9）
        writer.put_u8(0); // 加密类型：关
        writer.put_u8(16); // MAC 长度声明
        writer.skip(80); // 主密钥槽：内联进槽，加密关时全 0（D22 已定项 9，2026-09-13 用户定案）
        writer.put_u32(NODE_BYTES as u32);
        writer.put_u32(DATA_UNIT_BYTES as u32);
        writer.put_u32(SLOT_BYTES as u32);
        writer.put_u32(LOC_ENTRY as u32);
        // 2026-09-14 用户定案加的两个几何字段，排在位置条目宽度之后（字节表一那一节的行序）。
        writer.put_u32(MKFS_PHYSICAL_BLOCK_BYTES);
        writer.put_u32(EXTENSION_POINT_DECLARED_BYTES);
        writer.put_u64(JOURNAL_START_SLOT);
        writer.put_u64(JOURNAL_RING_BYTES);
        writer.put_u32(JOURNAL_RECORD_BYTES as u32);
        // D23（journal 的角色与格式） 已定项 18：在飞记录数上限 = 环槽数 ÷ F。
        writer.put_u32(u32::try_from(JOURNAL_IN_FLIGHT_RECORD_LIMIT).expect("在飞上限 4 字节"));
        // journal 最坏占用 = 在飞上限 × 记录宽（字节表 `first-txn-layout.md` 那一行逐字
        // 「268 435 456（65536 条 × 4096）」；此前这里写成了记录宽本身 4096）。
        writer.put_u64(JOURNAL_WORST_CASE_BYTES);
        writer.put_u32(u32::try_from(JOURNAL_SAFETY_FACTOR).expect("安全系数 4 字节"));
        writer.put_u8(RING_REGIONS as u8);
        writer.put_u8(RING_SLOTS_PER_REGION as u8);
        writer.put_u32(RING_PRIME_STEP as u32);
        writer.put_u32(RING_CHUNK_BYTES as u32);
        writer.put_u64(RING_START_OFFSET / SLOT_BYTES);
        writer.assert_position(SUPERBLOCK_REGION_DEVICES_OFFSET as u64, "根环逐区域设备身份");
        for region_device in &self.region_devices {
            writer.put_u32(region_device.0);
        }
        writer.put_u8(SUPERBLOCK_MAXIMUM_WIDTH);
        writer.put_u8(SUPERBLOCK_GROUP_SIZE);
        writer.skip(24); // 映射来源
        // D22 已定项 15 加的三个几何字段，排在「映射来源」之后。
        writer.put_u64(UNIT_AREA_START_SLOT);
        writer.put_u32(MKFS_MINIMUM_INPUT_OUTPUT_BYTES);
        writer.put_u32(FIXED_STRUCTURE_SLOT_SPACING);
        writer.put_u32(5); // T_time 秒
        writer.put_u64(2 << 30); // T_dirty
        writer.skip(24); // 整理三条水位：第一版恒 0 = 内置默认（D22 已定项 15）
        writer.assert_position(SUPERBLOCK_TAIL_OFFSET as u64, "journal tail");
        writer.put_u64(self.journal_tail);
        writer.put_u32(self.journal_instance.0);
        writer.assert_position(SUPERBLOCK_BYTES, "超级块");
        let mut bytes = writer.bytes;
        // 「整槽校验和」：覆盖整个 4096 槽含补齐、自身按 0 参与（D18 已定项 17）。
        let digest = wide_checksum_with_field_zeroed(&bytes, SUPERBLOCK_SLOT_BYTES as usize, SUPERBLOCK_CHECKSUM_OFFSET);
        bytes[SUPERBLOCK_CHECKSUM_OFFSET..SUPERBLOCK_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        bytes
    }

    fn parse_slot(bytes: &[u8]) -> Option<Self> {
        if bytes[..4] != SUPERBLOCK_MAGIC {
            return None;
        }
        if wide_checksum_with_field_zeroed(bytes, SUPERBLOCK_SLOT_BYTES as usize, SUPERBLOCK_CHECKSUM_OFFSET) != bytes[SUPERBLOCK_CHECKSUM_OFFSET..SUPERBLOCK_CHECKSUM_OFFSET + 32] {
            return None;
        }
        if !incompat_bits_are_mountable(bytes) {
            return None;
        }
        let mut reader = ByteReader::at(bytes, SUPERBLOCK_FSID_OFFSET);
        let fsid: [u8; 16] = reader.take(16).try_into().expect("切了 16 字节");
        reader.skip(WRITER_IDENTITY_NAME_BYTES + 4); // 写入者身份：读者不判它，只要在
        if reader.get_u8() != CHECKSUM_ALGORITHM_CRC32_CASTAGNOLI {
            return None; // I-2.2（校验和算法全盘一致）：算法标识不是已登记的 CRC-32C 就不认这个槽
        }
        let this_device = DeviceIdentity(reader.get_u32());
        let device_count = reader.get_u32();
        let slot_generation = reader.get_u64();
        let mut region_reader = ByteReader::at(bytes, SUPERBLOCK_REGION_DEVICES_OFFSET);
        let region_devices = [DeviceIdentity(region_reader.get_u32()), DeviceIdentity(region_reader.get_u32()), DeviceIdentity(region_reader.get_u32())];
        let mut tail_reader = ByteReader::at(bytes, SUPERBLOCK_TAIL_OFFSET);
        let journal_tail = tail_reader.get_u64();
        let journal_instance = InstanceGeneration(tail_reader.get_u32());
        Some(Self { fsid, this_device, device_count, slot_generation, region_devices, journal_tail, journal_instance })
    }
}

/// D15 已定项 4 与 fs-design「格式层的让非法状态无法表示」：incompat 位图里有读者不认识的位、
/// 或第一条 SSD 线那一位没置（没有布局身份），都拒绝挂载；compat_ro / compat 两张位图不认识随便，读者不看。
fn incompat_bits_are_mountable(slot: &[u8]) -> bool {
    let incompat = &slot[SUPERBLOCK_FEATURE_BITS_OFFSET..SUPERBLOCK_FEATURE_BITS_OFFSET + FEATURE_BITMAP_BYTES];
    let unknown_in_first_byte = incompat[0] & !SUPPORTED_INCOMPAT_BITS;
    let unknown_in_rest = incompat[1..].iter().any(|byte| *byte != 0);
    let has_layout_identity = incompat[0] & INCOMPAT_FIRST_SSD_LINE_BIT != 0;
    unknown_in_first_byte == 0 && !unknown_in_rest && has_layout_identity
}

/// 根环区域 r 的起点（设备内偏移）：`1 MiB + r × P × chunk`（字节表零，预想按设备内偏移读）。
fn ring_region_offset(region: u64) -> DeviceOffset {
    DeviceOffset(RING_START_OFFSET + region * RING_PRIME_STEP * RING_CHUNK_BYTES)
}

fn ring_slot_offset(region: u64, slot: u64) -> DeviceOffset {
    DeviceOffset(ring_region_offset(region).0 + slot * RING_SLOT_SPACING)
}

/// 第 n 次发布写区域 `n mod R` 的槽 `(n div R) mod S`（字节表一，区域内公式是预想）。
fn ring_target_for_publish(checkpoint_txg: CheckpointTxg) -> (u64, u64) {
    (checkpoint_txg.0 % RING_REGIONS, (checkpoint_txg.0 / RING_REGIONS) % RING_SLOTS_PER_REGION)
}

/// 一个单元类占多少字节：码 1 与码 3 都是 32768，码 2 是 16384。点名项不带单元大小，靠类标签定（D23 已定项 17）。
fn unit_bytes_for_class(unit_class: u8) -> Option<u64> {
    match unit_class {
        UNIT_CLASS_DATA | UNIT_CLASS_PACKED => Some(DATA_UNIT_BYTES),
        UNIT_CLASS_INDEX_NODE => Some(NODE_BYTES),
        _ => None,
    }
}

/// 点名项 56（D23（journal 的角色与格式） 已定项 17 的字段表，2026-09-13 用户定案）：
/// 位置条目 14 × 2（偏移 0）、单元类型标签 1（28）、出生树 8（29）、出生 txg 8（37）、
/// key 尾段 10（45：码 1 写序；码 2 / 码 3 实例代号 4 + 出生序号 4 + 补零 2）、flags 1（55）。
/// 不另带载荷 CRC——位置条目各带 4 字节校验和；重放从点名项直接凑出映射 key。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct NamedUnit {
    locations: [LocationEntry; 2],
    unit_class: u8,
    birth_tree: TreeIdentifier,
    birth_txg: CheckpointTxg,
    key_tail: [u8; 10],
}

impl NamedUnit {
    fn write_to(&self, writer: &mut ByteWriter) {
        let start = writer.position();
        for location in &self.locations {
            location.write_to(writer);
        }
        writer.put_u8(self.unit_class);
        writer.put_u64(self.birth_tree.0);
        writer.put_u64(self.birth_txg.0);
        writer.put(&self.key_tail);
        writer.put_u8(0); // flags：第一版恒 0、非 0 拒收（D22 已定项 17）
        assert_eq!(writer.position() - start, JOURNAL_NAMED_ENTRY_BYTES as usize);
    }
    fn read_from(reader: &mut ByteReader) -> Self {
        let locations = [LocationEntry::read_from(reader), LocationEntry::read_from(reader)];
        let unit_class = reader.get_u8();
        let birth_tree = TreeIdentifier(reader.get_u64());
        let birth_txg = CheckpointTxg(reader.get_u64());
        let key_tail: [u8; 10] = reader.take(10).try_into().expect("切了 10 字节");
        reader.skip(1);
        Self { locations, unit_class, birth_tree, birth_txg, key_tail }
    }
    /// 重放不查单元就能凑出这一项的中央映射 key（D23 已定项 17 末句）：一律 27 字节——
    /// 码 1 的尾段是 10 字节写序，码 2 / 码 3 的尾段是实例代号 4 + 出生序号 4 + 补零 2，两者加起来都正好是 key 的后 10 字节。
    fn mapping_key(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(MAPPING_KEY_BYTES as usize);
        writer.put_u8(self.unit_class);
        writer.put_u64(self.birth_tree.0);
        writer.put_u64(self.birth_txg.0);
        writer.put(&self.key_tail);
        writer.assert_position(MAPPING_KEY_BYTES, "点名项凑出来的映射 key");
        writer.bytes
    }
}

/// journal 记录 4096：头 307（字节表六的表序 + D23 已定项 15 的新根段 + 2026-09-14 加的 fsid 与 MAC）+ 点名项数组。
#[derive(Clone, PartialEq, Eq, Debug)]
struct JournalRecord {
    instance: InstanceGeneration,
    counter: JournalCounter,
    checkpoint_txg: CheckpointTxg,
    transaction: TransactionNumber,
    is_commit: bool,
    back_chain: u32,
    /// D23（journal 的角色与格式） 已定项 4（2026-09-14 用户定案）：超级块 fsid 的低 8 字节，与单元头同口径（I-1.4（块头 fsid 一致））。
    fsid: u64,
    /// D23（journal 的角色与格式） 已定项 15 的新根段：崩在记录持久之后、根槽持久之前时由它重建那次发布的根。
    new_tree_table: NodePointer,
    new_mapping_root: NodePointer,
    new_tree_identifier_watermark: u64,
    new_rollback_floor: CheckpointTxg,
    named: Vec<NamedUnit>,
}

const JOURNAL_HEADER_CHECKSUM_OFFSET: usize = 4 + 2 + 1 + 1 + 4 + 4 + 10 + 8 + 12;

/// D23（journal 的角色与格式） 已定项 19 ②（2026-09-14 用户定案改写）：`previous_hash` =
/// CRC32C(**本实例内逻辑前一条**记录的 307 字节头，其中 `header_csum` 那 32 字节按零参与)；本实例第一条恒 0、不读盘。
fn journal_back_chain(previous_record_bytes: &[u8]) -> u32 {
    let mut header = previous_record_bytes[..JOURNAL_HEADER_BYTES as usize].to_vec();
    header[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + WIDE_CHECKSUM_BYTES as usize].fill(0);
    castagnoli_crc32(&header)
}

impl JournalRecord {
    fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(JOURNAL_RECORD_BYTES as usize);
        writer.put(&JOURNAL_MAGIC);
        writer.put_u16(1); // 类型：普通记录
        writer.put_u8(0); // 算法类型
        writer.put_u8(0); // 对齐填充
        writer.put_u32(JOURNAL_RECORD_BYTES as u32);
        writer.put_u32(u32::try_from(self.named.len()).expect("点名项数 4 字节"));
        writer.put_u32(self.instance.0);
        writer.put_six_byte_unsigned(self.counter.0);
        writer.put_u64(self.checkpoint_txg.0);
        writer.skip(12); // nonce
        writer.assert_position(JOURNAL_HEADER_CHECKSUM_OFFSET as u64, "记录头校验和");
        writer.skip(WIDE_CHECKSUM_BYTES as usize);
        writer.put_u64(self.transaction.0);
        writer.put_u8(u8::from(self.is_commit));
        writer.put_u32(self.back_chain);
        let payload_checksum_offset = writer.position();
        writer.skip(4);
        let new_root_segment_start = writer.position();
        self.new_tree_table.write_to(&mut writer);
        self.new_mapping_root.write_to(&mut writer);
        writer.put_u64(self.new_tree_identifier_watermark);
        writer.put_u64(self.new_rollback_floor.0);
        assert_eq!(writer.position() - new_root_segment_start, JOURNAL_NEW_ROOT_SEGMENT_BYTES as usize, "新根段");
        // 2026-09-14 用户定案：新根段之后是 fsid 8，再之后是 MAC 16（第一版全 0 留位，D9（加密） day-1 预留第 1 项）。
        writer.put_u64(self.fsid);
        writer.skip(16);
        writer.assert_position(JOURNAL_HEADER_BYTES, "记录头");
        for named in &self.named {
            named.write_to(&mut writer);
        }
        let payload_end = writer.position();
        let mut bytes = writer.bytes;
        // 载荷校验和仍罩点名项数组（D23 已定项 13 的字段保留），它自己住头里、落在头校验和覆盖内。
        let payload_checksum = castagnoli_crc32(&bytes[JOURNAL_HEADER_BYTES as usize..payload_end]);
        bytes[payload_checksum_offset..payload_checksum_offset + 4].copy_from_slice(&payload_checksum.to_le_bytes());
        // D23 已定项 13（2026-09-14 用户定案改写）：`header_csum` 覆盖**整条记录 [0, 4096) 含补齐**、自身按零参与。
        let digest = wide_checksum_with_field_zeroed(&bytes, JOURNAL_RECORD_BYTES as usize, JOURNAL_HEADER_CHECKSUM_OFFSET);
        bytes[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        bytes
    }

    fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes[..4] != JOURNAL_MAGIC {
            return None;
        }
        if wide_checksum_with_field_zeroed(bytes, JOURNAL_RECORD_BYTES as usize, JOURNAL_HEADER_CHECKSUM_OFFSET)
            != bytes[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32]
        {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 4);
        let _record_type = reader.get_u16();
        reader.skip(2);
        let record_length = reader.get_u32();
        let named_count = reader.get_u32() as usize;
        let instance = InstanceGeneration(reader.get_u32());
        let counter = JournalCounter(reader.get_six_byte_unsigned());
        let checkpoint_txg = CheckpointTxg(reader.get_u64());
        reader.skip(12 + WIDE_CHECKSUM_BYTES as usize);
        let transaction = TransactionNumber(reader.get_u64());
        let is_commit = reader.get_u8() == 1;
        let back_chain = reader.get_u32();
        let payload_checksum = reader.get_u32();
        let new_tree_table = NodePointer::read_from(&mut reader);
        let new_mapping_root = NodePointer::read_from(&mut reader);
        let new_tree_identifier_watermark = reader.get_u64();
        let new_rollback_floor = CheckpointTxg(reader.get_u64());
        let fsid = reader.get_u64();
        reader.skip(16); // MAC：第一版留位
        let payload_end = JOURNAL_HEADER_BYTES as usize + named_count * JOURNAL_NAMED_ENTRY_BYTES as usize;
        if record_length as usize != bytes.len() || payload_end > bytes.len() {
            return None;
        }
        if castagnoli_crc32(&bytes[JOURNAL_HEADER_BYTES as usize..payload_end]) != payload_checksum {
            return None;
        }
        let named = (0..named_count).map(|_| NamedUnit::read_from(&mut reader)).collect();
        Some(Self { instance, counter, checkpoint_txg, transaction, is_commit, back_chain, fsid, new_tree_table, new_mapping_root, new_tree_identifier_watermark, new_rollback_floor, named })
    }
}

// ───────────────────────── 树表条目、inode、extent、分配、记账、映射 ─────────────────────────

/// D8（核心索引结构） 已定项 8（2026-09-14 用户定案重排）：树 ID 8 打头（= 码 2 条目的 key）+ 条目长度 2
/// + 树的种类 2 + flags 2 + 根指针 86 + previous_snapshot_txg 8 + 诞生 txg 8 + 头 ID 8 + 预留 24 = 148。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct TreeTableEntry {
    kind: u16,
    tree: TreeIdentifier,
    root: NodePointer,
    birth_txg: CheckpointTxg,
    /// D5（快照 / 空间记账机制） 已定项 9：这棵树归哪个可写头；无归属写 0。
    head_identifier: u64,
}

impl TreeTableEntry {
    fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(TREE_TABLE_ENTRY_BYTES as usize);
        writer.put_u64(self.tree.0);
        writer.put_u16(TREE_TABLE_ENTRY_BYTES as u16);
        writer.put_u16(self.kind);
        writer.put_u16(0);
        self.root.write_to(&mut writer);
        writer.put_u64(0); // previous_snapshot_txg：0 = 不适用（第一个事务没有前驱快照）
        writer.put_u64(self.birth_txg.0);
        writer.put_u64(self.head_identifier);
        writer.skip(24);
        writer.assert_position(TREE_TABLE_ENTRY_BYTES, "树表条目");
        writer.bytes
    }
    fn parse(bytes: &[u8]) -> Result<Self, UnitError> {
        let mut reader = ByteReader::new(bytes);
        let tree = TreeIdentifier(reader.get_u64());
        if reader.get_u16() as u64 != TREE_TABLE_ENTRY_BYTES {
            return Err(UnitError::Structure("树表条目长度"));
        }
        let kind = reader.get_u16();
        if reader.get_u16() != 0 {
            return Err(UnitError::Structure("树表条目 flags 未知位")); // D8 已定项 8 ㊁：未知位一律拒收
        }
        let root = NodePointer::read_from(&mut reader);
        let _previous_snapshot_txg = reader.get_u64();
        let birth_txg = CheckpointTxg(reader.get_u64());
        let head_identifier = reader.get_u64();
        if bytes[TREE_TABLE_ENTRY_BYTES as usize - 24..].iter().any(|&byte| byte != 0) {
            return Err(UnitError::Structure("树表条目预留 24 字节非零"));
        }
        Ok(Self { kind, tree, root, birth_txg, head_identifier })
    }
}

/// 每棵树的 key 宽（码 2 头里的 key 区间与自述 key 宽都随它走）。
fn key_width_for_kind(kind: u16) -> Option<usize> {
    match kind {
        TREE_KIND_EXTENT => Some(24),
        TREE_KIND_INODE => Some(8),
        TREE_KIND_ALLOCATION => Some(ALLOCATION_KEY_BYTES),
        TREE_KIND_ACCOUNTING => Some(22),
        TREE_KIND_MAPPING => Some(MAPPING_KEY_BYTES as usize),
        // livelist、稀疏旁表、deadlist 三棵 day-1 注册但第一个事务里根指针为零 ⇒ 没有节点要解，key 宽这一格还没有条款。
        _ => None,
    }
}
/// 树表单元自己按 key = 树 ID（8 字节）走（预想）。
const TREE_TABLE_KEY_WIDTH: usize = 8;

/// inode 树内部节点条目 120（D8 已定项 6）：分隔 key 8 + 身份引用 26（出生树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8）+ 子指针 86。
fn build_inode_internal_entry(separator_key: u64, child_identity: PackedIdentity, child: NodePointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(INODE_INTERNAL_ENTRY as usize);
    writer.put_u64(separator_key);
    writer.put_u64(child_identity.birth_tree.0);
    writer.put_u16(child_identity.record_type);
    writer.put_u64(child_identity.container);
    writer.put_u64(child_identity.container_birth.0);
    child.write_to(&mut writer);
    writer.assert_position(INODE_INTERNAL_ENTRY, "inode 内部条目");
    writer.bytes
}

fn parse_inode_internal_entry(bytes: &[u8]) -> (u64, PackedIdentity, NodePointer) {
    let mut reader = ByteReader::new(bytes);
    let separator_key = reader.get_u64();
    let child_identity = PackedIdentity {
        birth_tree: TreeIdentifier(reader.get_u64()),
        record_type: reader.get_u16(),
        container: reader.get_u64(),
        container_birth: CheckpointTxg(reader.get_u64()),
    };
    (separator_key, child_identity, NodePointer::read_from(&mut reader))
}

/// inode 记录 140（D8 已定项 6 的偏移表）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct InodeRecord {
    inode: u64,
    object_birth: CheckpointTxg,
    size: u64,
    change_count: u64,
}

impl InodeRecord {
    fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(INODE_RECORD_BYTES as usize);
        writer.put_u64(self.inode);
        writer.put_u64(self.object_birth.0);
        writer.put_u64(0); // locality_id：第一版无父目录取 0
        writer.put_u32(0o100644); // mode
        writer.put_u32(0); // uid
        writer.put_u32(0); // gid
        writer.put_u32(1); // nlink
        writer.assert_position(40, "size");
        writer.put_u64(self.size);
        writer.put_u64(DATA_UNIT_BYTES / 512); // blocks，按 512 字节块计
        writer.put_u64(0); // rdev
        writer.assert_position(64, "时间秒");
        for _ in 0..3 {
            writer.put_u64(FIXED_WRITE_TIME_SECONDS);
        }
        writer.assert_position(88, "改动计数");
        writer.put_u64(self.change_count);
        writer.assert_position(96, "时间纳秒");
        writer.skip(12);
        writer.assert_position(108, "填充 / flags / 预留");
        writer.skip(4 + 8 + 20);
        writer.assert_position(INODE_RECORD_BYTES, "inode 记录");
        writer.bytes
    }
    fn parse(bytes: &[u8]) -> Result<Self, UnitError> {
        let mut reader = ByteReader::new(bytes);
        let inode = reader.get_u64();
        let object_birth = CheckpointTxg(reader.get_u64());
        reader.skip(8 + 16);
        let size = reader.get_u64();
        reader.skip(16 + 24);
        let change_count = reader.get_u64();
        if bytes[108..].iter().any(|&byte| byte != 0) {
            return Err(UnitError::Structure("inode 记录三段非零")); // I-9.7
        }
        Ok(Self { inode, object_birth, size, change_count })
    }
}

/// extent 叶记录 112：key (locality_id 8, inode 8, offset 8) + 指向码 1 的指针 88。
fn build_extent_record(inode: u64, offset: u64, pointer: DataPointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(EXTENT_LEAF_RECORD_BYTES as usize);
    writer.put_u64(0);
    writer.put_u64(inode);
    writer.put_u64(offset);
    pointer.write_to(&mut writer);
    writer.assert_position(EXTENT_LEAF_RECORD_BYTES, "extent 叶记录");
    writer.bytes
}

fn parse_extent_record(bytes: &[u8]) -> ([u8; 24], DataPointer) {
    let mut reader = ByteReader::new(bytes);
    let key: [u8; 24] = reader.take(24).try_into().expect("切了 24 字节");
    (key, DataPointer::read_from(&mut reader))
}

/// 分配记录 20（D3 已定项 7 + 已定项 11）：key = 设备 4 + 槽号 6；value = 跨度 2（最高位 = 已释放）+ 分配代 8。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct AllocationRecord {
    device: DeviceIdentity,
    slot: SlotNumber,
    span_slots: u16,
    generation: CheckpointTxg,
}

impl AllocationRecord {
    fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(ALLOCATION_RECORD_BYTES as usize);
        writer.put_u32(self.device.0);
        writer.put_six_byte_unsigned(self.slot.0);
        writer.put_u16(self.span_slots);
        writer.put_u64(self.generation.0);
        writer.assert_position(ALLOCATION_RECORD_BYTES, "分配记录");
        writer.bytes
    }
    fn parse(bytes: &[u8]) -> Self {
        let mut reader = ByteReader::new(bytes);
        Self { device: DeviceIdentity(reader.get_u32()), slot: SlotNumber(reader.get_six_byte_unsigned()), span_slots: reader.get_u16(), generation: CheckpointTxg(reader.get_u64()) }
    }
    fn key_bytes(&self) -> Vec<u8> {
        self.to_bytes()[..ALLOCATION_KEY_BYTES].to_vec()
    }
    /// D8（核心索引结构） 已定项 11：key 的全序按逐字段无符号整数、字段自左向右比较——小端存储不构成 memcmp 序。
    fn sort_key(&self) -> (u32, u64) {
        (self.device.0, self.slot.0)
    }
}

/// 记账条目 34（D5 已定项 5 + D8 已定项 7）：key (标签 2, 树 ID 8, 设备 4, 代 8) + value 8 + seq 4。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct AccountingEntry {
    statistic: u16,
    tree: TreeIdentifier,
    device: DeviceIdentity,
    generation: CheckpointTxg,
    value: u64,
    sequence: u32,
}

impl AccountingEntry {
    fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(ACCOUNTING_ENTRY_BYTES as usize);
        writer.put_u16(self.statistic);
        writer.put_u64(self.tree.0);
        writer.put_u32(self.device.0);
        writer.put_u64(self.generation.0);
        writer.put_u64(self.value);
        writer.put_u32(self.sequence);
        writer.assert_position(ACCOUNTING_ENTRY_BYTES, "记账条目");
        writer.bytes
    }
    fn parse(bytes: &[u8]) -> Self {
        let mut reader = ByteReader::new(bytes);
        Self {
            statistic: reader.get_u16(),
            tree: TreeIdentifier(reader.get_u64()),
            device: DeviceIdentity(reader.get_u32()),
            generation: CheckpointTxg(reader.get_u64()),
            value: reader.get_u64(),
            sequence: reader.get_u32(),
        }
    }
    fn key_bytes(&self) -> Vec<u8> {
        self.to_bytes()[..22].to_vec()
    }
    /// key 的全序按逐字段无符号整数比较（D8（核心索引结构） 已定项 11）：标签 → 树 ID → 设备 → 代。
    fn sort_key(&self) -> (u16, u64, u32, u64) {
        (self.statistic, self.tree.0, self.device.0, self.generation.0)
    }
}

/// 中央映射 key（D19 已定项 6 / 已定项 10，2026-09-14 用户定案回定宽）：**一律 27**——
/// 码 1 = 类标签 1 + 出生树 8 + 出生 txg 8 + 写序 10 天然就是 27；
/// 码 2 / 码 3 = 类标签 1 + 出生树 8 + 出生 txg 8 + 实例代号 4 + 出生序号 4 = 25，**末尾补零到 27**。
fn mapping_key_for_data(head: PointerHead, write_order: WriteOrder) -> Vec<u8> {
    let mut writer = ByteWriter::new(MAPPING_KEY_BYTES as usize);
    writer.put_u8(UNIT_CLASS_DATA);
    writer.put_u64(head.birth_tree.0);
    writer.put_u64(head.birth_txg.0);
    writer.put_u32(write_order.instance.0);
    writer.put_six_byte_unsigned(write_order.transaction.0);
    writer.assert_position(MAPPING_KEY_BYTES, "码 1 映射 key");
    writer.bytes
}

fn mapping_key_for_node(unit_class: u8, pointer: NodePointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(MAPPING_KEY_BYTES as usize);
    writer.put_u8(unit_class);
    writer.put_u64(pointer.head.birth_tree.0);
    writer.put_u64(pointer.head.birth_txg.0);
    writer.put_u32(pointer.instance.0);
    writer.put_u32(pointer.birth_sequence.0);
    writer.assert_position(MAPPING_KEY_NODE_SIGNIFICANT_BYTES, "码 2 / 码 3 映射 key 里有含义的那一段");
    writer.skip(usize::try_from(MAPPING_KEY_BYTES - MAPPING_KEY_NODE_SIGNIFICANT_BYTES).expect("补零两字节"));
    writer.assert_position(MAPPING_KEY_BYTES, "码 2 / 码 3 映射 key");
    writer.bytes
}

/// key 的全序按逐字段无符号整数、字段自左向右比较（D8（核心索引结构） 已定项 11）：
/// 类标签 → 出生树 → 出生 txg → 尾段（码 1 的写序、码 2 / 码 3 的实例代号与出生序号）。
fn mapping_key_sort_key(key: &[u8]) -> (u8, u64, u64, u32, u64) {
    let mut reader = ByteReader::new(key);
    let unit_class = reader.get_u8();
    let birth_tree = reader.get_u64();
    let birth_txg = reader.get_u64();
    let instance = reader.get_u32();
    let tail = if unit_class == UNIT_CLASS_DATA { reader.get_six_byte_unsigned() } else { u64::from(reader.get_u32()) };
    (unit_class, birth_tree, birth_txg, instance, tail)
}

fn build_mapping_entry(key: &[u8], locations: [LocationEntry; 2]) -> Vec<u8> {
    assert_eq!(key.len() as u64, MAPPING_KEY_BYTES, "映射 key 一律 27（D19 已定项 10，2026-09-14 用户定案）");
    let mut writer = ByteWriter::new(MAPPING_ENTRY_BYTES as usize);
    writer.put(key);
    for location in &locations {
        location.write_to(&mut writer);
    }
    writer.assert_position(MAPPING_ENTRY_BYTES, "映射条目");
    writer.bytes
}

fn parse_mapping_entry(bytes: &[u8]) -> (Vec<u8>, [LocationEntry; 2]) {
    let mut reader = ByteReader::new(bytes);
    let key = reader.take(MAPPING_KEY_BYTES as usize).to_vec();
    (key, [LocationEntry::read_from(&mut reader), LocationEntry::read_from(&mut reader)])
}

/// 实例表 kind = 1 链指针记录（D18 已定项 11）：kind 1 | 有无下一片 1 | 位置指针 86（无下一片时清零）= 88。
/// 2026-09-14 指针从 83 变 86 之后那 3 字节预留被吃掉，记录宽仍 88——**这条记录没有余量了**，
/// 指针再宽一个字节 `INSTANCE_ROW_BYTES` 就要跟着涨（打包记录定长，kind 0 与 kind 1 必须同宽）。
fn build_instance_table_chain_record() -> Vec<u8> {
    let mut writer = ByteWriter::new(INSTANCE_ROW_BYTES as usize);
    writer.put_u8(1);
    writer.put_u8(0);
    writer.skip(NODE_POINTER_BYTES as usize);
    writer.assert_position(INSTANCE_ROW_BYTES, "实例表链指针记录");
    writer.bytes
}

// ───────────────────────── mkfs 与第一个事务 ─────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BarrierPolicy {
    /// D16 已定项 7：单元 → 屏障 → 记录 → 屏障 → 根槽 FUA。
    Settled,
    /// 阳性对照：一道屏障都不放。
    None,
}

#[derive(Clone, Debug)]
struct PoolParameters {
    fsid: [u8; 16],
    device_count: usize,
    region_devices: [DeviceIdentity; 3],
    barriers: BarrierPolicy,
}

impl PoolParameters {
    fn settled_two_devices() -> Self {
        Self {
            fsid: FIXED_FSID,
            device_count: 2,
            region_devices: [DeviceIdentity(RING_REGION_DEVICES[0]), DeviceIdentity(RING_REGION_DEVICES[1]), DeviceIdentity(RING_REGION_DEVICES[2])],
            barriers: BarrierPolicy::Settled,
        }
    }
    fn control_one_device_no_barriers() -> Self {
        Self { fsid: FIXED_FSID, device_count: 1, region_devices: [DeviceIdentity(0), DeviceIdentity(0), DeviceIdentity(0)], barriers: BarrierPolicy::None }
    }
    /// 一个单元的两条位置条目：两盘各一份（D2 已定项 10）；只有一块盘时两条都指盘 0（对照臂）。
    fn location_entries(&self, slot: SlotNumber, unit: &[u8]) -> [LocationEntry; 2] {
        let unit_checksum = castagnoli_crc32(unit);
        let second_device = DeviceIdentity(u32::try_from(self.device_count - 1).expect("设备数"));
        [LocationEntry { device: DeviceIdentity(0), slot, unit_checksum }, LocationEntry { device: second_device, slot, unit_checksum }]
    }
    fn devices(&self) -> Vec<DeviceIdentity> {
        (0..self.device_count).map(|index| DeviceIdentity(u32::try_from(index).expect("设备数"))).collect()
    }
}

/// 出生序号（C291：分配规则只在实验源码里）——装置预想：按 (树, txg, 实例) 从 0 计，码 2 与码 3 共用一个计数。
#[derive(Default)]
struct BirthSequenceAllocator {
    counters: BTreeMap<(TreeIdentifier, CheckpointTxg, InstanceGeneration), u32>,
}

impl BirthSequenceAllocator {
    fn next(&mut self, tree: TreeIdentifier, txg: CheckpointTxg, instance: InstanceGeneration) -> BirthSequence {
        let counter = self.counters.entry((tree, txg, instance)).or_insert(0);
        let sequence = BirthSequence(*counter);
        *counter += 1;
        sequence
    }
}

fn write_unit_to_every_device(pool: &mut RecordingPool, parameters: &PoolParameters, slot: SlotNumber, unit: &[u8]) {
    for device in parameters.devices() {
        pool.write(device, slot.device_offset(), unit, StepKind::UnitWrite);
    }
}

#[derive(Clone, Debug)]
struct MkfsOutput {
    root: RootRecord,
    instance_table_unit: Vec<u8>,
    tree_table_genesis_unit: Vec<u8>,
}

fn mkfs(parameters: &PoolParameters) -> (RecordingPool, MkfsOutput) {
    let mut pool = RecordingPool { pool: Pool { devices: vec![SparseDevice::default(); parameters.device_count] }, operations: Vec::new() };
    // D23（journal 的角色与格式） 已定项 16：mkfs 写实例代号 0，单元写序 (0, 0)，第 0 代根实例代号 0。
    let instance = InstanceGeneration(MKFS_INSTANCE_GENERATION);
    let genesis = CheckpointTxg(0);
    let genesis_write_order = WriteOrder { instance, transaction: TransactionNumber(0) };
    let mut sequences = BirthSequenceAllocator::default();

    let instance_table_identity = PackedIdentity { birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE), record_type: PACKED_TYPE_INSTANCE_TABLE, container: 0, container_birth: genesis };
    let instance_table_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), genesis, instance);
    let instance_table_unit = build_packed_unit(
        instance_table_identity,
        INSTANCE_ROW_BYTES as u16,
        &[build_instance_table_chain_record()],
        genesis,
        &parameters.fsid,
        genesis_write_order,
        instance_table_sequence,
    );
    let tree_table_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), genesis, instance);
    let tree_table_genesis_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_NONE),
        0,
        TREE_TABLE_KEY_WIDTH,
        &[0u8; 8],
        &[0u8; 8],
        genesis,
        &parameters.fsid,
        instance,
        tree_table_sequence,
        TREE_TABLE_ENTRY_BYTES as u16,
        &[],
    );
    write_unit_to_every_device(&mut pool, parameters, SlotNumber(SLOT_INSTANCE_TABLE), &instance_table_unit);
    write_unit_to_every_device(&mut pool, parameters, SlotNumber(SLOT_TREE_TABLE_GENESIS), &tree_table_genesis_unit);
    pool.barrier();

    let root = RootRecord {
        fsid: parameters.fsid,
        instance,
        checkpoint_txg: genesis,
        tree_table: NodePointer {
            head: PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE), birth_txg: genesis },
            locations: parameters.location_entries(SlotNumber(SLOT_TREE_TABLE_GENESIS), &tree_table_genesis_unit),
            instance,
            birth_sequence: tree_table_sequence,
        },
        tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        rollback_floor: CheckpointTxg(0),
        instance_table: NodePointer {
            head: PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE), birth_txg: genesis },
            locations: parameters.location_entries(SlotNumber(SLOT_INSTANCE_TABLE), &instance_table_unit),
            instance,
            birth_sequence: instance_table_sequence,
        },
        // 中央映射树的根住根记录（D19 已定项 11）；mkfs 那一刻还没有映射树。
        mapping_root: NodePointer::empty_root(),
    };
    let root_slot = root.to_slot();
    // D22 已定项 8：第 0 代根种进全部区域，各自槽 0，FUA。
    for region in 0..RING_REGIONS {
        pool.write(parameters.region_devices[region as usize], ring_slot_offset(region, 0), &root_slot, StepKind::RootRecordFua);
    }
    // D22（单元原子性怎么合成） 已定项 16：每盘恒 2 个槽，世代号从 1 起；mkfs 把两个槽都种上世代号 1。
    for device in parameters.devices() {
        let superblock = Superblock {
            fsid: parameters.fsid,
            this_device: device,
            device_count: u32::try_from(parameters.device_count).expect("设备数"),
            slot_generation: SUPERBLOCK_GENERATION_AT_MKFS,
            region_devices: parameters.region_devices,
            journal_tail: 0,
            journal_instance: instance,
        };
        for slot_offset in SUPERBLOCK_SLOT_OFFSETS {
            pool.write(device, DeviceOffset(slot_offset), &superblock.to_slot(), StepKind::SuperblockSlot);
        }
    }
    pool.barrier();
    (pool, MkfsOutput { root, instance_table_unit, tree_table_genesis_unit })
}

/// D23（journal 的角色与格式） 已定项 16：第一次可写挂载取 max(超级块, 根环) + 1 = 1，
/// **并写进每一份超级块（一次超级块槽写，世代号 +1）之后才动单元**——所以它自成一段，排在暖机之前。
/// 段的收尾靠暖机第一次空发布开头那道屏障（D16 已定项 7 的形态，不另加屏障：这是最少屏障的写法）。
fn acquire_instance(pool: &mut RecordingPool, parameters: &PoolParameters) -> InstanceGeneration {
    let instance = InstanceGeneration(FIRST_INSTANCE_GENERATION);
    let slot_index = (SUPERBLOCK_GENERATION_AT_INSTANCE_ACQUISITION % 2) as usize;
    for device in parameters.devices() {
        let superblock = Superblock {
            fsid: parameters.fsid,
            this_device: device,
            device_count: u32::try_from(parameters.device_count).expect("设备数"),
            slot_generation: SUPERBLOCK_GENERATION_AT_INSTANCE_ACQUISITION,
            region_devices: parameters.region_devices,
            journal_tail: 0,
            journal_instance: instance,
        };
        pool.write(device, DeviceOffset(SUPERBLOCK_SLOT_OFFSETS[slot_index]), &superblock.to_slot(), StepKind::SuperblockSlot);
    }
    instance
}

/// 第一个事务写出的八个单元各自的身份。以前它是自由文本标签，靠 `match` 字符串取类与 key 宽，
/// 漏一个只能在运行期 panic；做成封闭枚举之后每一处 match 都穷举，漏一种编译不过。
/// 步骤种类（`StepKind`）说的是「这一步是哪一类写」，这个说的是「写的是哪个单元」——两件事分开。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
enum TransactionUnit {
    Data,
    ExtentRoot,
    InodeLeaf,
    InodeRoot,
    AllocationTree,
    AccountingTree,
    MappingTree,
    TreeTable,
}

impl TransactionUnit {
    /// 字节表七里的步号 t1..t8，`name=write_list` 的 `units=` 用它。
    fn tag(self) -> &'static str {
        match self {
            TransactionUnit::Data => "t1",
            TransactionUnit::ExtentRoot => "t2",
            TransactionUnit::InodeLeaf => "t3",
            TransactionUnit::InodeRoot => "t4",
            TransactionUnit::AllocationTree => "t5",
            TransactionUnit::AccountingTree => "t6",
            TransactionUnit::MappingTree => "t7",
            TransactionUnit::TreeTable => "t8",
        }
    }
    /// D18（块里携带什么信息） 已定项 11 的类码，加上码 2 的 key 宽（码 1 与码 3 没有 key 区间，写 0）。
    fn class_and_key_width(self) -> (u8, usize) {
        match self {
            TransactionUnit::Data => (UNIT_CLASS_DATA, 0),
            TransactionUnit::InodeLeaf => (UNIT_CLASS_PACKED, 0),
            TransactionUnit::ExtentRoot => (UNIT_CLASS_INDEX_NODE, 24),
            TransactionUnit::InodeRoot => (UNIT_CLASS_INDEX_NODE, 8),
            TransactionUnit::AllocationTree => (UNIT_CLASS_INDEX_NODE, ALLOCATION_KEY_BYTES),
            TransactionUnit::AccountingTree => (UNIT_CLASS_INDEX_NODE, 22),
            TransactionUnit::MappingTree => (UNIT_CLASS_INDEX_NODE, MAPPING_KEY_BYTES as usize),
            TransactionUnit::TreeTable => (UNIT_CLASS_INDEX_NODE, TREE_TABLE_KEY_WIDTH),
        }
    }
    /// 点名项里的归属树；树表单元不属于任何一棵树（写 0）。
    fn tree(self) -> TreeIdentifier {
        match self {
            TransactionUnit::Data | TransactionUnit::ExtentRoot => TreeIdentifier(TREE_IDENTIFIER_EXTENT),
            TransactionUnit::InodeLeaf | TransactionUnit::InodeRoot => TreeIdentifier(TREE_IDENTIFIER_INODE),
            TransactionUnit::AllocationTree => TreeIdentifier(TREE_IDENTIFIER_ALLOCATION),
            TransactionUnit::AccountingTree => TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING),
            TransactionUnit::MappingTree => TreeIdentifier(TREE_IDENTIFIER_MAPPING),
            TransactionUnit::TreeTable => TreeIdentifier(TREE_IDENTIFIER_NONE),
        }
    }
}

/// 第一个事务写出的东西，留给探针与断言用。
#[derive(Clone, Debug)]
struct TransactionOutput {
    root: RootRecord,
    record: JournalRecord,
    record_bytes: Vec<u8>,
    units_by_slot: Vec<(SlotNumber, TransactionUnit, Vec<u8>)>,
    data_pointer: DataPointer,
    mapping_keys: Vec<Vec<u8>>,
    allocation_records: Vec<AllocationRecord>,
    accounting_entries: Vec<AccountingEntry>,
    index_node_header_widths: Vec<(&'static str, usize)>,
}

/// 单元区里的空闲槽数（D5 已定项 7：容量 = 单元区大小，固定结构既不在容量里也不算已分配）。
fn free_slot_count(occupied: &std::collections::BTreeSet<u64>) -> u64 {
    UNIT_AREA_SLOTS - occupied.len() as u64
}

/// 空闲 run 数（统计量第 10 项）：单元区里极长的连续空闲槽段有几段。
fn free_run_count(occupied: &std::collections::BTreeSet<u64>) -> u64 {
    let mut runs = 0u64;
    let mut previous_was_free = false;
    for slot in UNIT_AREA_START_SLOT..UNIT_AREA_START_SLOT + UNIT_AREA_SLOTS {
        let is_free = !occupied.contains(&slot);
        if is_free && !previous_was_free {
            runs += 1;
        }
        previous_was_free = is_free;
    }
    runs
}

/// 全空聚簇段数（统计量第 11 项，D3 已定项 10 ①）：段内 64 槽都没有未释放分配记录的段数。
fn empty_cluster_segment_count(occupied: &std::collections::BTreeSet<u64>) -> u64 {
    (0..UNIT_AREA_SLOTS / CLUSTER_SEGMENT_SLOTS)
        .filter(|segment_index| {
            let start = UNIT_AREA_START_SLOT + segment_index * CLUSTER_SEGMENT_SLOTS;
            (start..start + CLUSTER_SEGMENT_SLOTS).all(|slot| !occupied.contains(&slot))
        })
        .count() as u64
}

/// 点名项 key 尾段的两种形态（D23 已定项 17）：码 1 写 10 字节写序；码 2 / 码 3 写实例代号 4 + 出生序号 4 + 补零 2。
fn data_key_tail(write_order: WriteOrder) -> [u8; 10] {
    let mut tail = [0u8; 10];
    tail[..4].copy_from_slice(&write_order.instance.0.to_le_bytes());
    tail[4..].copy_from_slice(&write_order.transaction.0.to_le_bytes()[..6]);
    tail
}

fn node_key_tail(instance: InstanceGeneration, birth_sequence: BirthSequence) -> [u8; 10] {
    let mut tail = [0u8; 10];
    tail[..4].copy_from_slice(&instance.0.to_le_bytes());
    tail[4..8].copy_from_slice(&birth_sequence.0.to_le_bytes());
    tail
}

/// D23（journal 的角色与格式） 已定项 18：记录 n 落在环内偏移 `(计数器 − 1) mod 槽数 × 4096`。
fn journal_record_offset(counter: JournalCounter) -> DeviceOffset {
    DeviceOffset(JOURNAL_START_SLOT * SLOT_BYTES + ((counter.0 - 1) % JOURNAL_RING_SLOTS) * JOURNAL_RECORD_BYTES)
}

/// 发布之后那次超级块槽写的世代号与落点（D22 已定项 16）：取号那次是 2，之后每次发布 +1，槽 = 世代号 mod 2。
fn superblock_write_for_publish(checkpoint_txg: CheckpointTxg) -> (u64, usize) {
    let generation = checkpoint_txg.0 + SUPERBLOCK_GENERATION_AT_INSTANCE_ACQUISITION;
    (generation, (generation % 2) as usize)
}

/// 暖机（D16 已定项 8）：每次空发布照 D16 已定项 7 的顺序——屏障 → 空记录 → 屏障 → 根槽 FUA → 超级块槽轮换；
/// 空记录不点名任何单元、事务号 0（D23 已定项 19 ①：0 保留给不承载事务的记录）、提交标记 1，
/// 新根段照 mkfs 的根，根记录只改 checkpoint_txg。返回最后一条记录的字节，下一条记录的反向链要用它。
fn warm_up(pool: &mut RecordingPool, parameters: &PoolParameters, genesis: &MkfsOutput, instance: InstanceGeneration) -> (Vec<RootRecord>, Option<Vec<u8>>) {
    let mut roots = Vec::new();
    let mut previous_record_bytes: Option<Vec<u8>> = None;
    for txg_number in 1..=WARM_UP_EMPTY_PUBLISHES {
        let txg = CheckpointTxg(txg_number);
        if parameters.barriers == BarrierPolicy::Settled {
            pool.barrier();
        }
        let counter = JournalCounter(txg_number);
        let record = JournalRecord {
            instance,
            counter,
            checkpoint_txg: txg,
            transaction: TransactionNumber(0),
            is_commit: true,
            back_chain: previous_record_bytes.as_deref().map_or(0, journal_back_chain),
            fsid: unit_fsid(&parameters.fsid),
            new_tree_table: genesis.root.tree_table,
            new_mapping_root: genesis.root.mapping_root,
            new_tree_identifier_watermark: genesis.root.tree_identifier_watermark,
            new_rollback_floor: genesis.root.rollback_floor,
            named: Vec::new(),
        };
        let record_bytes = record.to_bytes();
        for device in parameters.devices() {
            pool.write(device, journal_record_offset(counter), &record_bytes, StepKind::JournalRecord);
        }
        previous_record_bytes = Some(record_bytes);
        if parameters.barriers == BarrierPolicy::Settled {
            pool.barrier();
        }
        let root = RootRecord {
            fsid: genesis.root.fsid,
            instance,
            checkpoint_txg: txg,
            tree_table: genesis.root.tree_table,
            tree_identifier_watermark: genesis.root.tree_identifier_watermark,
            rollback_floor: genesis.root.rollback_floor,
            instance_table: genesis.root.instance_table,
            mapping_root: genesis.root.mapping_root,
        };
        let (region, ring_slot) = ring_target_for_publish(txg);
        pool.write(parameters.region_devices[region as usize], ring_slot_offset(region, ring_slot), &root.to_slot(), StepKind::RootRecordFua);
        let (slot_generation, slot_index) = superblock_write_for_publish(txg);
        for device in parameters.devices() {
            let superblock = Superblock {
                fsid: parameters.fsid,
                this_device: device,
                device_count: u32::try_from(parameters.device_count).expect("设备数"),
                slot_generation,
                region_devices: parameters.region_devices,
                journal_tail: txg_number,
                journal_instance: instance,
            };
            pool.write(device, DeviceOffset(SUPERBLOCK_SLOT_OFFSETS[slot_index]), &superblock.to_slot(), StepKind::SuperblockSlot);
        }
        roots.push(root);
    }
    (roots, previous_record_bytes)
}

fn publish_first_file(
    pool: &mut RecordingPool,
    parameters: &PoolParameters,
    genesis: &MkfsOutput,
    file_bytes: &[u8],
    instance: InstanceGeneration,
    previous_record_bytes: Option<&[u8]>,
) -> TransactionOutput {
    let txg = CheckpointTxg(FIRST_TRANSACTION_TXG);
    let transaction = TransactionNumber(1);
    let write_order = WriteOrder { instance, transaction };
    let fsid = &parameters.fsid;
    let mut sequences = BirthSequenceAllocator::default();
    let mut units: Vec<(SlotNumber, TransactionUnit, Vec<u8>)> = Vec::new();
    let mut index_node_header_widths = Vec::new();

    // t1 数据单元
    let data_identity = DataUnitIdentity { tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT), object: FIRST_INODE_NUMBER, object_birth: txg, anchor_offset: 0 };
    let data_unit = build_data_unit(data_identity, txg, fsid, write_order, file_bytes);
    let data_pointer = DataPointer {
        head: PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT), birth_txg: txg },
        locations: parameters.location_entries(SlotNumber(SLOT_DATA_UNIT), &data_unit),
        write_order,
    };

    // t3 inode 树叶容器
    let inode_leaf_identity = PackedIdentity { birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE), record_type: PACKED_TYPE_INODE, container: FIRST_INODE_NUMBER, container_birth: txg };
    let inode_leaf_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_INODE), txg, instance);
    let inode_record = InodeRecord { inode: FIRST_INODE_NUMBER, object_birth: txg, size: file_bytes.len() as u64, change_count: 1 };
    let inode_leaf_unit = build_packed_unit(inode_leaf_identity, INODE_RECORD_BYTES as u16, &[inode_record.to_bytes()], txg, fsid, write_order, inode_leaf_sequence);
    let inode_leaf_pointer = NodePointer {
        head: PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE), birth_txg: txg },
        locations: parameters.location_entries(SlotNumber(SLOT_INODE_LEAF), &inode_leaf_unit),
        instance,
        birth_sequence: inode_leaf_sequence,
    };

    // t2 extent 树根兼叶
    let extent_key = build_extent_record(FIRST_INODE_NUMBER, 0, data_pointer)[..24].to_vec();
    let extent_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_EXTENT), txg, instance);
    let extent_unit = build_index_node(TreeIdentifier(TREE_IDENTIFIER_EXTENT), 0, 24, &extent_key, &extent_key, txg, fsid, instance, extent_sequence, EXTENT_LEAF_RECORD_BYTES as u16, &[build_extent_record(FIRST_INODE_NUMBER, 0, data_pointer)]);
    index_node_header_widths.push(("extent", index_node_header_bytes(24)));

    // t4 inode 树根
    let inode_root_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_INODE), txg, instance);
    let inode_key = FIRST_INODE_NUMBER.to_le_bytes();
    let inode_root_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_INODE),
        1,
        8,
        &inode_key,
        &inode_key,
        txg,
        fsid,
        instance,
        inode_root_sequence,
        INODE_INTERNAL_ENTRY as u16,
        &[build_inode_internal_entry(FIRST_INODE_NUMBER, inode_leaf_identity, inode_leaf_pointer)],
    );
    index_node_header_widths.push(("inode", index_node_header_bytes(8)));

    // t5 分配记录树：mkfs 的 m1 / m2 分配代 0，其余就是这次发布的 txg（D3 已定项 7 的「分配代」）；每盘各一条。
    // deadlist 树 day-1 注册但没有根节点 ⇒ 不占落点、不写分配记录（字节表零那一节的 t5 那一行）。
    let allocated: [(u64, u16, u64); 10] = [
        (SLOT_INSTANCE_TABLE, 2, 0),
        (SLOT_TREE_TABLE_GENESIS, 1, 0),
        (SLOT_DATA_UNIT, 2, FIRST_TRANSACTION_TXG),
        (SLOT_EXTENT_ROOT, 1, FIRST_TRANSACTION_TXG),
        (SLOT_INODE_LEAF, 2, FIRST_TRANSACTION_TXG),
        (SLOT_INODE_ROOT, 1, FIRST_TRANSACTION_TXG),
        (SLOT_ALLOCATION_ROOT, 1, FIRST_TRANSACTION_TXG),
        (SLOT_ACCOUNTING_ROOT, 1, FIRST_TRANSACTION_TXG),
        (SLOT_MAPPING_ROOT, 1, FIRST_TRANSACTION_TXG),
        (SLOT_TREE_TABLE_FIRST_PUBLISH, 1, FIRST_TRANSACTION_TXG),
    ];
    let mut allocation_records: Vec<AllocationRecord> = parameters
        .devices()
        .iter()
        .flat_map(|device| allocated.iter().map(move |(slot, span, generation)| AllocationRecord { device: *device, slot: SlotNumber(*slot), span_slots: *span, generation: CheckpointTxg(*generation) }))
        .collect();
    allocation_records.sort_by_key(AllocationRecord::sort_key);
    let allocation_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_ALLOCATION), txg, instance);
    let allocation_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_ALLOCATION),
        0,
        ALLOCATION_KEY_BYTES,
        &allocation_records[0].key_bytes(),
        &allocation_records[allocation_records.len() - 1].key_bytes(),
        txg,
        fsid,
        instance,
        allocation_sequence,
        ALLOCATION_RECORD_BYTES as u16,
        &allocation_records.iter().map(AllocationRecord::to_bytes).collect::<Vec<_>>(),
    );
    index_node_header_widths.push(("allocation", index_node_header_bytes(ALLOCATION_KEY_BYTES)));

    // t6 记账树（D5 已定项 8，2026-09-14 用户定案）：两盘时 **15 行**——
    // 带设备维的六项各两行（已分配 1、空闲 2、不可回收 3、defer 待释放 5、碎片度 runs 10、全空聚簇段数 11），
    // 池级三行（待删占用 4、已承诺预留 6、inode 号水位 12）。准入不等式要的四项 day-1 各写一行 0，
    // 「空 ≠ 0」那条防线因此保住：读不到行是坏账，读到 0 是真的 0。seq 一律 1（D8 已定项 10：直落叶）。
    let allocated_slots: u64 = allocated.iter().map(|(_, span, _)| u64::from(*span)).sum();
    let occupied: std::collections::BTreeSet<u64> = allocated.iter().flat_map(|(slot, span, _)| (0..u64::from(*span)).map(move |offset| slot + offset)).collect();
    // bump 次序 + 码 3 容器要 32768 对齐 ⇒ 开放段里 t2 与 t3 之间恰好空着一个槽，它自己是一段空闲 run。
    assert_eq!(SLOT_SKIPPED_BY_ALIGNMENT, SLOT_EXTENT_ROOT + 1, "空洞紧跟在 extent 根后面");
    assert_eq!(SLOT_INODE_LEAF, SLOT_SKIPPED_BY_ALIGNMENT + 1, "inode 叶容器跳过空洞落在下一个对齐槽对");
    assert!(!occupied.contains(&SLOT_SKIPPED_BY_ALIGNMENT), "槽 50241 空着（D3 已定项 10 ⑤ 的落点表）");
    let pool_wide = |statistic: u16, tree: u64, value: u64| AccountingEntry {
        statistic,
        tree: TreeIdentifier(tree),
        device: DeviceIdentity(STATISTIC_NO_DEVICE_DIMENSION),
        generation: txg,
        value,
        sequence: ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF,
    };
    let mut accounting_entries = vec![
        // 第 12 项带树维、不带设备维：树 ID 就是那个可写头的 inode 树（D5 已定项 9）。
        pool_wide(STATISTIC_INODE_WATERMARK, TREE_IDENTIFIER_INODE, FIRST_INODE_NUMBER + 1),
        pool_wide(STATISTIC_PENDING_DELETE_BYTES, TREE_IDENTIFIER_NONE, 0),
        pool_wide(STATISTIC_COMMITTED_RESERVATION_BYTES, TREE_IDENTIFIER_NONE, 0),
    ];
    for device in parameters.devices() {
        let per_device = |statistic: u16, value: u64| AccountingEntry {
            statistic,
            tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
            device,
            generation: txg,
            value,
            sequence: ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF,
        };
        accounting_entries.push(per_device(STATISTIC_ALLOCATED_BYTES, allocated_slots * SLOT_BYTES));
        // 第 2 项必须独立维护、不许由「容量 − 已分配」现算（D5 已定项 4 的 ⚠️）：这里从空槽数直接数出来。
        accounting_entries.push(per_device(STATISTIC_FREE_BYTES, free_slot_count(&occupied) * SLOT_BYTES));
        accounting_entries.push(per_device(STATISTIC_UNRECLAIMABLE_BYTES, 0));
        accounting_entries.push(per_device(STATISTIC_DEFER_QUEUE_BYTES, 0));
        // 第 10 项 2026-09-14 起换口径并带设备维：单元区空闲槽的连续段数，逐设备一行。
        accounting_entries.push(per_device(STATISTIC_FRAGMENTATION_RUNS, free_run_count(&occupied)));
        accounting_entries.push(per_device(STATISTIC_EMPTY_CLUSTER_SEGMENTS, empty_cluster_segment_count(&occupied)));
    }
    accounting_entries.sort_by_key(AccountingEntry::sort_key);
    let accounting_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING), txg, instance);
    let accounting_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING),
        0,
        22,
        &accounting_entries[0].key_bytes(),
        &accounting_entries[accounting_entries.len() - 1].key_bytes(),
        txg,
        fsid,
        instance,
        accounting_sequence,
        ACCOUNTING_ENTRY_BYTES as u16,
        &accounting_entries.iter().map(AccountingEntry::to_bytes).collect::<Vec<_>>(),
    );
    index_node_header_widths.push(("accounting", index_node_header_bytes(22)));

    // 五棵树的根指针（t7 映射与 t8 树表都要引它们）
    let node_pointer = |tree: TreeIdentifier, slot: u64, unit: &[u8], sequence: BirthSequence| NodePointer {
        head: PointerHead { birth_tree: tree, birth_txg: txg },
        locations: parameters.location_entries(SlotNumber(slot), unit),
        instance,
        birth_sequence: sequence,
    };
    let extent_pointer = node_pointer(TreeIdentifier(TREE_IDENTIFIER_EXTENT), SLOT_EXTENT_ROOT, &extent_unit, extent_sequence);
    let inode_root_pointer = node_pointer(TreeIdentifier(TREE_IDENTIFIER_INODE), SLOT_INODE_ROOT, &inode_root_unit, inode_root_sequence);
    let allocation_pointer = node_pointer(TreeIdentifier(TREE_IDENTIFIER_ALLOCATION), SLOT_ALLOCATION_ROOT, &allocation_unit, allocation_sequence);
    let accounting_pointer = node_pointer(TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING), SLOT_ACCOUNTING_ROOT, &accounting_unit, accounting_sequence);

    // t7 中央映射树：码 1 一条 + 码 2 / 码 3 五条（D19 已定项 8；映射树自己、树表、实例表豁免）。
    let mut mapping_entries: Vec<(Vec<u8>, [LocationEntry; 2])> = vec![
        (mapping_key_for_data(data_pointer.head, data_pointer.write_order), data_pointer.locations),
        (mapping_key_for_node(UNIT_CLASS_INDEX_NODE, extent_pointer), extent_pointer.locations),
        (mapping_key_for_node(UNIT_CLASS_PACKED, inode_leaf_pointer), inode_leaf_pointer.locations),
        (mapping_key_for_node(UNIT_CLASS_INDEX_NODE, inode_root_pointer), inode_root_pointer.locations),
        (mapping_key_for_node(UNIT_CLASS_INDEX_NODE, allocation_pointer), allocation_pointer.locations),
        (mapping_key_for_node(UNIT_CLASS_INDEX_NODE, accounting_pointer), accounting_pointer.locations),
    ];
    mapping_entries.sort_by_key(|(key, _)| mapping_key_sort_key(key));
    let mapping_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_MAPPING), txg, instance);
    let mapping_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_MAPPING),
        0,
        MAPPING_KEY_BYTES as usize,
        &mapping_entries[0].0,
        &mapping_entries[mapping_entries.len() - 1].0,
        txg,
        fsid,
        instance,
        mapping_sequence,
        MAPPING_ENTRY_BYTES as u16,
        &mapping_entries.iter().map(|(key, locations)| build_mapping_entry(key, *locations)).collect::<Vec<_>>(),
    );
    index_node_header_widths.push(("mapping", index_node_header_bytes(MAPPING_KEY_BYTES as usize)));
    let mapping_pointer = node_pointer(TreeIdentifier(TREE_IDENTIFIER_MAPPING), SLOT_MAPPING_ROOT, &mapping_unit, mapping_sequence);

    // t8 树表单元第 1 版：**七条**条目按树 ID 升序（D8 已定项 8）；中央映射树的根住根记录、不进树表（D19 已定项 11）；
    // livelist、稀疏旁表与 deadlist 三棵 day-1 注册、根指针为零。
    // 头 ID（D5 已定项 9，2026-09-14 用户定案）：inode 树写自己（12）、extent 树写它服务的那个头（12），其余写 0。
    let tree_table_entries = [
        TreeTableEntry { kind: TREE_KIND_EXTENT, tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT), root: extent_pointer, birth_txg: txg, head_identifier: TREE_IDENTIFIER_INODE },
        TreeTableEntry { kind: TREE_KIND_INODE, tree: TreeIdentifier(TREE_IDENTIFIER_INODE), root: inode_root_pointer, birth_txg: txg, head_identifier: TREE_IDENTIFIER_INODE },
        TreeTableEntry { kind: TREE_KIND_ALLOCATION, tree: TreeIdentifier(TREE_IDENTIFIER_ALLOCATION), root: allocation_pointer, birth_txg: txg, head_identifier: TREE_TABLE_HEAD_IDENTIFIER_NONE },
        TreeTableEntry { kind: TREE_KIND_ACCOUNTING, tree: TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING), root: accounting_pointer, birth_txg: txg, head_identifier: TREE_TABLE_HEAD_IDENTIFIER_NONE },
        TreeTableEntry { kind: TREE_KIND_LIVELIST, tree: TreeIdentifier(TREE_IDENTIFIER_LIVELIST), root: NodePointer::empty_root(), birth_txg: txg, head_identifier: TREE_TABLE_HEAD_IDENTIFIER_NONE },
        TreeTableEntry { kind: TREE_KIND_SHARE_COUNT, tree: TreeIdentifier(TREE_IDENTIFIER_SHARE_COUNT), root: NodePointer::empty_root(), birth_txg: txg, head_identifier: TREE_TABLE_HEAD_IDENTIFIER_NONE },
        TreeTableEntry { kind: TREE_KIND_DEADLIST, tree: TreeIdentifier(TREE_IDENTIFIER_DEADLIST), root: NodePointer::empty_root(), birth_txg: txg, head_identifier: TREE_TABLE_HEAD_IDENTIFIER_NONE },
    ];
    let tree_table_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), txg, instance);
    let tree_table_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_NONE),
        0,
        TREE_TABLE_KEY_WIDTH,
        &TREE_IDENTIFIER_EXTENT.to_le_bytes(),
        &TREE_IDENTIFIER_DEADLIST.to_le_bytes(),
        txg,
        fsid,
        instance,
        tree_table_sequence,
        TREE_TABLE_ENTRY_BYTES as u16,
        &tree_table_entries.iter().map(TreeTableEntry::to_bytes).collect::<Vec<_>>(),
    );
    index_node_header_widths.push(("tree_table", index_node_header_bytes(TREE_TABLE_KEY_WIDTH)));
    let tree_table_pointer = node_pointer(TreeIdentifier(TREE_IDENTIFIER_NONE), SLOT_TREE_TABLE_FIRST_PUBLISH, &tree_table_unit, tree_table_sequence);

    units.push((SlotNumber(SLOT_DATA_UNIT), TransactionUnit::Data, data_unit));
    units.push((SlotNumber(SLOT_EXTENT_ROOT), TransactionUnit::ExtentRoot, extent_unit));
    units.push((SlotNumber(SLOT_INODE_LEAF), TransactionUnit::InodeLeaf, inode_leaf_unit));
    units.push((SlotNumber(SLOT_INODE_ROOT), TransactionUnit::InodeRoot, inode_root_unit));
    units.push((SlotNumber(SLOT_ALLOCATION_ROOT), TransactionUnit::AllocationTree, allocation_unit));
    units.push((SlotNumber(SLOT_ACCOUNTING_ROOT), TransactionUnit::AccountingTree, accounting_unit));
    units.push((SlotNumber(SLOT_MAPPING_ROOT), TransactionUnit::MappingTree, mapping_unit));
    units.push((SlotNumber(SLOT_TREE_TABLE_FIRST_PUBLISH), TransactionUnit::TreeTable, tree_table_unit));

    // 持久顺序第一段：单元与节点。
    for (slot, _, unit) in &units {
        write_unit_to_every_device(pool, parameters, *slot, unit);
    }
    if parameters.barriers == BarrierPolicy::Settled {
        pool.barrier();
    }

    // 第二段：journal 记录，点名 t1..t8 每个两盘；反向链照 D23 已定项 19 ② 罩前一条的整个头（header_csum 按零参与）。
    // 点名项的 key 尾段与 `units` 一一对应，次序就是 t1..t8；unit_class 与 TransactionUnit 自报的那一个对账。
    let named_identities: [(u8, TreeIdentifier, [u8; 10]); 8] = [
        (UNIT_CLASS_DATA, data_pointer.head.birth_tree, data_key_tail(data_pointer.write_order)),
        (UNIT_CLASS_INDEX_NODE, extent_pointer.head.birth_tree, node_key_tail(extent_pointer.instance, extent_pointer.birth_sequence)),
        (UNIT_CLASS_PACKED, inode_leaf_pointer.head.birth_tree, node_key_tail(inode_leaf_pointer.instance, inode_leaf_pointer.birth_sequence)),
        (UNIT_CLASS_INDEX_NODE, inode_root_pointer.head.birth_tree, node_key_tail(inode_root_pointer.instance, inode_root_pointer.birth_sequence)),
        (UNIT_CLASS_INDEX_NODE, allocation_pointer.head.birth_tree, node_key_tail(allocation_pointer.instance, allocation_pointer.birth_sequence)),
        (UNIT_CLASS_INDEX_NODE, accounting_pointer.head.birth_tree, node_key_tail(accounting_pointer.instance, accounting_pointer.birth_sequence)),
        (UNIT_CLASS_INDEX_NODE, mapping_pointer.head.birth_tree, node_key_tail(mapping_pointer.instance, mapping_pointer.birth_sequence)),
        (UNIT_CLASS_INDEX_NODE, tree_table_pointer.head.birth_tree, node_key_tail(tree_table_pointer.instance, tree_table_pointer.birth_sequence)),
    ];
    let named: Vec<NamedUnit> = units
        .iter()
        .zip(named_identities)
        .map(|((slot, unit_identity, unit), (unit_class, birth_tree, key_tail))| {
            assert_eq!(unit_class, unit_identity.class_and_key_width().0, "点名项的类标签与 {} 自报的对不上", unit_identity.tag());
            assert_eq!(birth_tree, unit_identity.tree(), "点名项的出生树与 {} 自报的对不上", unit_identity.tag());
            NamedUnit { locations: parameters.location_entries(*slot, unit), unit_class, birth_tree, birth_txg: txg, key_tail }
        })
        .collect();
    let counter = JournalCounter(FIRST_TRANSACTION_TXG);
    let record = JournalRecord {
        instance,
        counter,
        checkpoint_txg: txg,
        transaction,
        is_commit: true,
        back_chain: previous_record_bytes.map_or(0, journal_back_chain),
        fsid: unit_fsid(fsid),
        new_tree_table: tree_table_pointer,
        new_mapping_root: mapping_pointer,
        new_tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AFTER_PUBLISH,
        new_rollback_floor: CheckpointTxg(0),
        named,
    };
    let record_bytes = record.to_bytes();
    for device in parameters.devices() {
        pool.write(device, journal_record_offset(counter), &record_bytes, StepKind::JournalRecord);
    }
    if parameters.barriers == BarrierPolicy::Settled {
        pool.barrier();
    }

    // 第三段：根槽 FUA，区域 3 mod 3 = 0 的槽 1（暖机之后第一个事务是 txg 3）。
    let root = RootRecord {
        fsid: parameters.fsid,
        instance,
        checkpoint_txg: txg,
        tree_table: tree_table_pointer,
        tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AFTER_PUBLISH,
        rollback_floor: CheckpointTxg(0),
        instance_table: genesis.root.instance_table,
        mapping_root: mapping_pointer,
    };
    let (region, ring_slot) = ring_target_for_publish(txg);
    pool.write(parameters.region_devices[region as usize], ring_slot_offset(region, ring_slot), &root.to_slot(), StepKind::RootRecordFua);

    // 根槽之后：超级块槽轮换，世代号 5、落槽 1（D22 已定项 16），tail 前移到 jsn 3（D16 已定项 7 的超级块注）。
    let (slot_generation, slot_index) = superblock_write_for_publish(txg);
    for device in parameters.devices() {
        let superblock = Superblock {
            fsid: parameters.fsid,
            this_device: device,
            device_count: u32::try_from(parameters.device_count).expect("设备数"),
            slot_generation,
            region_devices: parameters.region_devices,
            journal_tail: FIRST_TRANSACTION_TXG,
            journal_instance: instance,
        };
        pool.write(device, DeviceOffset(SUPERBLOCK_SLOT_OFFSETS[slot_index]), &superblock.to_slot(), StepKind::SuperblockSlot);
    }

    TransactionOutput {
        root,
        record,
        record_bytes,
        units_by_slot: units,
        data_pointer,
        mapping_keys: mapping_entries.into_iter().map(|(key, _)| key).collect(),
        allocation_records,
        accounting_entries,
        index_node_header_widths,
    }
}

// ───────────────────────── 冷启动恢复（里程碑步 6） ─────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum JournalPolicy {
    /// 全环扫描、按 D23 已定项 14 的五条口径取前缀、施加前逐项验证点名单元。
    Consult,
    /// 根本不看 journal：与 Consult 逐状态比，问 journal 在第一个事务里承不承重。
    Ignore,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum RecoveryOutcome {
    /// 择到的根下面没有文件（第 0 代）。
    NoFile { root: (InstanceGeneration, CheckpointTxg) },
    /// 沿树走到数据单元、校验和全过，读回内容。
    FileRead { root: (InstanceGeneration, CheckpointTxg), content: Vec<u8> },
    /// 走不下去：哪一步、为什么。
    Failed { root: Option<(InstanceGeneration, CheckpointTxg)>, reason: String },
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
struct JournalScanReport {
    valid_records: usize,
    above_water: usize,
    prefix_applied: usize,
    verification_passed: usize,
    verification_failed: usize,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct RecoveryReport {
    outcome: RecoveryOutcome,
    journal: JournalScanReport,
    mapping_fallbacks: usize,
}

/// 按指针里的位置条目读一个单元：逐条试，整单元 CRC32C 等于条目里的校验和才算读到（字节表三：无加密时那 4 字节就是整单元 CRC）。
fn read_unit_via_locations(reader: &dyn BlockReader, locations: &[LocationEntry; 2], unit_bytes: usize) -> Result<Vec<u8>, String> {
    for location in locations {
        if location.device.0 as usize >= reader.device_count() {
            continue;
        }
        let bytes = reader.read(location.device, location.slot.device_offset(), unit_bytes);
        if castagnoli_crc32(&bytes) == location.unit_checksum {
            return Ok(bytes);
        }
    }
    Err(format!("槽 {} 的两条位置条目都读不到校验和相符的单元", locations[0].slot.0))
}

fn choose_superblock(reader: &dyn BlockReader) -> Result<Superblock, String> {
    let mut chosen: Option<Superblock> = None;
    for device_index in 0..reader.device_count() {
        let device = DeviceIdentity(u32::try_from(device_index).expect("设备数"));
        let mut best_on_device: Option<Superblock> = None;
        for slot_offset in SUPERBLOCK_SLOT_OFFSETS {
            let bytes = reader.read(device, DeviceOffset(slot_offset), SUPERBLOCK_SLOT_BYTES as usize);
            if let Some(candidate) = Superblock::parse_slot(&bytes) {
                if best_on_device.as_ref().is_none_or(|best| candidate.slot_generation > best.slot_generation) {
                    best_on_device = Some(candidate);
                }
            }
        }
        let Some(best_on_device) = best_on_device else {
            return Err(format!("盘 {device_index} 两个超级块槽都无效"));
        };
        match &chosen {
            None => chosen = Some(best_on_device),
            Some(previous) => {
                if previous.fsid != best_on_device.fsid || previous.device_count != best_on_device.device_count {
                    return Err("两盘的超级块 fsid 或设备数对不上".to_string());
                }
            }
        }
    }
    chosen.ok_or_else(|| "池里没有设备".to_string())
}

fn choose_root(reader: &dyn BlockReader, superblock: &Superblock) -> Option<RootRecord> {
    let mut best: Option<RootRecord> = None;
    for region in 0..RING_REGIONS {
        let device = superblock.region_devices[region as usize];
        if device.0 as usize >= reader.device_count() {
            continue;
        }
        for slot in 0..RING_SLOTS_PER_REGION {
            let bytes = reader.read(device, ring_slot_offset(region, slot), PHYSICAL_BLOCK_BYTES as usize);
            if let Some(candidate) = RootRecord::parse_slot(&bytes, &superblock.fsid) {
                let candidate_key = (candidate.checkpoint_txg, candidate.instance);
                if best.is_none_or(|current| candidate_key > (current.checkpoint_txg, current.instance)) {
                    best = Some(candidate);
                }
            }
        }
    }
    best
}

/// 全环扫描（D23 已定项 3：不先信 tail），两份镜像任一份合法即算记录在（gap G9）。
/// 记录头 2026-09-14 起带 fsid（D23 已定项 4）⇒ fsid 不符的记录一律不算数（I-1.4（块头 fsid 一致））。
fn scan_journal(reader: &dyn BlockReader, expected_fsid: u64) -> BTreeMap<(InstanceGeneration, JournalCounter), JournalRecord> {
    let mut records = BTreeMap::new();
    let ring_start = JOURNAL_START_SLOT * SLOT_BYTES;
    for device_index in 0..reader.device_count() {
        let device = DeviceIdentity(u32::try_from(device_index).expect("设备数"));
        for sector in reader.written_sectors(device, DeviceOffset(ring_start), JOURNAL_RING_BYTES) {
            let offset = sector * PHYSICAL_BLOCK_BYTES;
            if (offset - ring_start) % JOURNAL_RECORD_BYTES != 0 {
                continue;
            }
            let bytes = reader.read(device, DeviceOffset(offset), JOURNAL_RECORD_BYTES as usize);
            if let Some(record) = JournalRecord::parse(&bytes) {
                if record.fsid != expected_fsid {
                    continue;
                }
                records.entry((record.instance, record.counter)).or_insert(record);
            }
        }
    }
    records
}

/// D23 已定项 14 的五条口径里第一个事务碰得到的三条：jsn 严格连续、(实例代号, checkpoint_txg) 大于根的水位、提交标记齐全；
/// 在飞记录数上限（D23 已定项 18，第一版 65536）也进前缀判定。
/// 「施加一条记录」= 把所选根的四个字段换成记录新根段里的那四个（D23 已定项 15，2026-09-13 用户定案）——
/// 树表单元指针、中央映射树根指针、树 ID 水位、回退下界 F；实例表单元指针照所选根，
/// 实例代号与 checkpoint_txg 照记录头（那次发布的身份就在头里，所以新根段里不重复）。
fn replay_journal(
    reader: &dyn BlockReader,
    root: &RootRecord,
    records: &BTreeMap<(InstanceGeneration, JournalCounter), JournalRecord>,
) -> (JournalScanReport, RootRecord) {
    let mut report = JournalScanReport { valid_records: records.len(), ..JournalScanReport::default() };
    let water = (root.instance, root.checkpoint_txg);
    let mut rebuilt = *root;
    let mut above: Vec<&JournalRecord> = records.values().filter(|record| (record.instance, record.checkpoint_txg) > water).collect();
    above.sort_by_key(|record| (record.instance, record.counter));
    report.above_water = above.len();
    let mut expected: Option<(InstanceGeneration, JournalCounter)> = None;
    for record in above.into_iter().take(usize::try_from(JOURNAL_IN_FLIGHT_RECORD_LIMIT).expect("在飞上限")) {
        if let Some(expected_key) = expected {
            if (record.instance, record.counter) != expected_key {
                break;
            }
        }
        expected = Some((record.instance, JournalCounter(record.counter.0 + 1)));
        if !record.is_commit {
            break;
        }
        let mut all_verified = true;
        for named in &record.named {
            let Some(unit_bytes) = unit_bytes_for_class(named.unit_class) else {
                all_verified = false;
                continue;
            };
            for location in &named.locations {
                if location.device.0 as usize >= reader.device_count() {
                    continue;
                }
                let bytes = reader.read(location.device, location.slot.device_offset(), usize::try_from(unit_bytes).expect("单元字节数"));
                if castagnoli_crc32(&bytes) != location.unit_checksum {
                    all_verified = false;
                }
            }
        }
        if all_verified {
            report.verification_passed += 1;
            report.prefix_applied += 1;
            rebuilt = RootRecord {
                fsid: rebuilt.fsid,
                instance: record.instance,
                checkpoint_txg: record.checkpoint_txg,
                tree_table: record.new_tree_table,
                tree_identifier_watermark: record.new_tree_identifier_watermark,
                rollback_floor: record.new_rollback_floor,
                instance_table: rebuilt.instance_table,
                mapping_root: record.new_mapping_root,
            };
        } else {
            report.verification_failed += 1;
            break;
        }
    }
    (report, rebuilt)
}

struct TreeRoots {
    extent: IndexNodeHeader,
    inode: IndexNodeHeader,
    allocation: IndexNodeHeader,
    accounting: IndexNodeHeader,
    mapping: IndexNodeHeader,
}

fn read_tree_root(reader: &dyn BlockReader, kind: u16, tree: TreeIdentifier, pointer: &NodePointer, root: &RootRecord, expected_fsid: u64) -> Result<IndexNodeHeader, String> {
    let entry = TreeTableEntry { kind, tree, root: *pointer, birth_txg: CheckpointTxg(0), head_identifier: TREE_TABLE_HEAD_IDENTIFIER_NONE };
    let key_width = key_width_for_kind(entry.kind).ok_or_else(|| format!("树的种类 {} 没登记", entry.kind))?;
    let bytes = read_unit_via_locations(reader, &entry.root.locations, NODE_BYTES as usize)?;
    // key 宽住偏移 51（D18 已定项 18，2026-09-14 用户定案）⇒ 解析器自己读得出来，不用调用方先告诉它。
    let node = parse_index_node(&bytes).map_err(|error| format!("树 {} 的根 {error:?}", entry.tree.0))?;
    if node.tree != entry.tree {
        return Err(format!("树 {} 的根头里写的树 ID 是 {}", entry.tree.0, node.tree.0)); // I-1.3
    }
    if node.key_width != key_width {
        return Err(format!("树 {} 的根自述 key 宽 {}，而这棵树的种类 {} 要 {key_width}", entry.tree.0, node.key_width, entry.kind));
    }
    if node.birth_txg > root.checkpoint_txg || node.instance > root.instance {
        return Err(format!("树 {} 的根诞生于根之后", entry.tree.0)); // I-1.2 的第一版读法
    }
    if node.fsid != expected_fsid {
        return Err(format!("树 {} 的根 fsid 不符", entry.tree.0)); // I-1.4
    }
    if node.birth_sequence != entry.root.birth_sequence {
        return Err(format!("树 {} 的根出生序号与指针不符", entry.tree.0));
    }
    // D8 已定项 11（2026-09-14 用户定案）：条目里 key 一律是条目的前 key 宽 个字节 ⇒ 区间与条目对得上是逐字节比较。
    if let (Some(first), Some(last)) = (node.entries.first(), node.entries.last()) {
        if first[..key_width] != node.smallest_key[..] || last[..key_width] != node.largest_key[..] {
            return Err(format!("树 {} 的根 key 区间与条目不符", entry.tree.0)); // I-1.1 索引节点那一半
        }
    }
    Ok(node)
}

fn walk_to_file(reader: &dyn BlockReader, root: &RootRecord, fsid: &[u8; 16], mapping_fallbacks: &mut usize) -> Result<Option<Vec<u8>>, String> {
    let expected_fsid = unit_fsid(fsid);
    // 实例表单元由根记录直接持有（D22 已定项 7）。
    let instance_table_bytes = read_unit_via_locations(reader, &root.instance_table.locations, DATA_UNIT_BYTES as usize)?;
    let instance_table = parse_packed_unit(&instance_table_bytes).map_err(|error| format!("实例表 {error:?}"))?;
    if instance_table.identity.record_type != PACKED_TYPE_INSTANCE_TABLE || instance_table.records.is_empty() {
        return Err("实例表单元不是类型 4 或没有链指针记录".to_string());
    }
    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, NODE_BYTES as usize)?;
    let tree_table = parse_index_node(&tree_table_bytes).map_err(|error| format!("树表 {error:?}"))?;
    if tree_table.key_width != TREE_TABLE_KEY_WIDTH {
        return Err(format!("树表单元自述 key 宽 {}，而它的 key 是树 ID", tree_table.key_width));
    }
    if tree_table.entries.is_empty() {
        return Ok(None);
    }
    let entries: Vec<TreeTableEntry> = tree_table.entries.iter().map(|bytes| TreeTableEntry::parse(bytes)).collect::<Result<_, _>>().map_err(|error| format!("树表条目 {error:?}"))?;
    let mut by_kind: BTreeMap<u16, IndexNodeHeader> = BTreeMap::new();
    for entry in &entries {
        if entry.tree.0 >= root.tree_identifier_watermark {
            return Err("树 ID 不低于水位".to_string()); // I-7.8
        }
        if entry.root.is_empty_root() {
            continue; // day-1 注册、还没有根的树（livelist、稀疏旁表）：没有单元可读
        }
        by_kind.insert(entry.kind, read_tree_root(reader, entry.kind, entry.tree, &entry.root, root, expected_fsid)?);
    }
    let mut take = |kind: u16, name: &str| by_kind.remove(&kind).ok_or_else(|| format!("树表里没有{name}"));
    let roots = TreeRoots {
        extent: take(TREE_KIND_EXTENT, "extent 树")?,
        inode: take(TREE_KIND_INODE, "inode 树")?,
        allocation: take(TREE_KIND_ALLOCATION, "分配记录树")?,
        accounting: take(TREE_KIND_ACCOUNTING, "记账树")?,
        // 中央映射树的根住根记录，不进树表（D19 已定项 11）。
        mapping: read_tree_root(reader, TREE_KIND_MAPPING, TreeIdentifier(TREE_IDENTIFIER_MAPPING), &root.mapping_root, root, expected_fsid)?,
    };
    if roots.allocation.entries.len() != 10 * reader.device_count() {
        return Err(format!("分配记录数 {} 不是 10 × 盘数", roots.allocation.entries.len()));
    }
    // D5 已定项 8（2026-09-14 用户定案）：池级三行（待删占用、已承诺预留、inode 号水位）+ 每盘六行
    // （已分配字节、空闲字节、不可回收、defer 待释放、碎片度 runs、全空聚簇段数）⇒ 两盘 15 行。
    if roots.accounting.entries.len() != 3 + 6 * reader.device_count() {
        return Err(format!("记账条目数 {} 不是 3 + 6 × 盘数", roots.accounting.entries.len()));
    }
    if roots.mapping.entries.len() != 6 {
        return Err(format!("映射条目数 {} 不是 6", roots.mapping.entries.len()));
    }
    for record_bytes in &roots.allocation.entries {
        let record = AllocationRecord::parse(record_bytes);
        if record.span_slots & 0x8000 != 0 || record.generation > root.checkpoint_txg || record.span_slots == 0 {
            return Err("分配记录带已释放标志、跨度为 0 或分配代晚于根".to_string()); // 第一个事务没有释放
        }
    }
    for entry_bytes in &roots.accounting.entries {
        let entry = AccountingEntry::parse(entry_bytes);
        if entry.generation > root.checkpoint_txg || entry.sequence == 0 {
            return Err("记账条目的代晚于根或 seq 为 0".to_string());
        }
    }

    // inode 树：根（码 2）→ 内部条目 → 叶容器（码 3）→ 记录。
    if roots.inode.level != 1 {
        return Err("inode 树根层级不是 1".to_string());
    }
    let mut inode_record: Option<InodeRecord> = None;
    for entry in &roots.inode.entries {
        let (separator_key, identity, child) = parse_inode_internal_entry(entry);
        if identity.record_type != PACKED_TYPE_INODE {
            return Err("inode 内部条目类型段不是 2".to_string());
        }
        let leaf_bytes = read_unit_via_locations(reader, &child.locations, DATA_UNIT_BYTES as usize)?;
        let leaf = parse_packed_unit(&leaf_bytes).map_err(|error| format!("inode 叶 {error:?}"))?;
        if leaf.identity != identity || leaf.record_width as u64 != INODE_RECORD_BYTES || leaf.fsid != expected_fsid {
            return Err("inode 叶头与条目身份引用不符".to_string()); // I-9.2
        }
        if child.head.birth_tree != identity.birth_tree || leaf.birth_sequence != child.birth_sequence {
            return Err("inode 叶的出生身份与子指针不符".to_string());
        }
        if leaf.birth_txg > root.checkpoint_txg || leaf.write_order.instance != child.instance {
            return Err("inode 叶诞生于根之后或写序实例与子指针不符".to_string()); // I-1.2 码 3 那一半
        }
        for record_bytes in &leaf.records {
            let record = InodeRecord::parse(record_bytes).map_err(|error| format!("inode 记录 {error:?}"))?;
            if record.inode < separator_key || record.inode < identity.container {
                return Err("inode 记录号小于分隔 key 或容器号".to_string()); // I-9.4 / I-9.12
            }
            if record.inode == FIRST_INODE_NUMBER {
                inode_record = Some(record);
            }
        }
    }
    let Some(inode_record) = inode_record else {
        return Ok(None);
    };

    // extent 树：按 (locality 0, inode 1, offset 0) 找指针，解引用先按位置提示、校验和不对再查映射（D19 已定项 5）。
    let mut wanted_key = [0u8; 24];
    wanted_key[8..16].copy_from_slice(&FIRST_INODE_NUMBER.to_le_bytes());
    let mut data_unit: Option<(DataPointer, Vec<u8>)> = None;
    for record_bytes in &roots.extent.entries {
        let (key, pointer) = parse_extent_record(record_bytes);
        if key != wanted_key {
            continue;
        }
        let bytes = match read_unit_via_locations(reader, &pointer.locations, DATA_UNIT_BYTES as usize) {
            Ok(bytes) => bytes,
            Err(hint_error) => {
                *mapping_fallbacks += 1;
                let mapping_key = mapping_key_for_data(pointer.head, pointer.write_order);
                let mapped = roots.mapping.entries.iter().map(|entry| parse_mapping_entry(entry)).find(|(key, _)| *key == mapping_key);
                let Some((_, locations)) = mapped else {
                    return Err(format!("{hint_error}；映射里也没有这个 key"));
                };
                read_unit_via_locations(reader, &locations, DATA_UNIT_BYTES as usize).map_err(|error| format!("经映射仍读不到：{error}"))?
            }
        };
        data_unit = Some((pointer, bytes));
    }
    let Some((pointer, bytes)) = data_unit else {
        return Err("extent 树里没有这个文件的记录".to_string());
    };
    let header = parse_data_unit(&bytes).map_err(|error| format!("数据单元 {error:?}"))?;
    if header.identity.tree != TreeIdentifier(TREE_IDENTIFIER_EXTENT) || header.identity.object != FIRST_INODE_NUMBER || header.identity.anchor_offset != 0 {
        return Err("数据单元五元组与查找路径不符".to_string()); // I-1.1
    }
    if header.identity.object_birth != inode_record.object_birth {
        return Err("对象出生代与 inode 记录不符".to_string()); // I-9.10
    }
    if header.birth_txg != pointer.head.birth_txg || header.write_order != pointer.write_order || header.fsid != expected_fsid {
        return Err("数据单元头与指针的出生身份不符".to_string());
    }
    if u64::from(header.declared_length) != inode_record.size {
        return Err("声明长度与 inode size 不符".to_string());
    }
    if bytes[(DATA_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES) as usize + header.declared_length as usize..].iter().any(|&byte| byte != 0) {
        return Err("补齐字节非零".to_string()); // I-2.3
    }
    Ok(Some(data_unit_payload(&bytes, header.declared_length).to_vec()))
}

#[allow(clippy::needless_pass_by_value, reason = "参数是 Copy 的策略枚举，按值取更贴调用点")]
fn recover(reader: &dyn BlockReader, policy: JournalPolicy) -> RecoveryReport {
    let mut mapping_fallbacks = 0;
    let superblock = match choose_superblock(reader) {
        Ok(superblock) => superblock,
        Err(reason) => return RecoveryReport { outcome: RecoveryOutcome::Failed { root: None, reason }, journal: JournalScanReport::default(), mapping_fallbacks },
    };
    let Some(root) = choose_root(reader, &superblock) else {
        return RecoveryReport { outcome: RecoveryOutcome::Failed { root: None, reason: "根环里一条合法根都没有".to_string() }, journal: JournalScanReport::default(), mapping_fallbacks };
    };
    // 报出去的 `root=` 恒是**所选**的那条根；施加记录之后走的是重建出来的根（D23 已定项 15）。
    let root_key = (root.instance, root.checkpoint_txg);
    let (journal, effective_root) = match policy {
        JournalPolicy::Consult => replay_journal(reader, &root, &scan_journal(reader, unit_fsid(&superblock.fsid))),
        JournalPolicy::Ignore => (JournalScanReport::default(), root),
    };
    let outcome = match walk_to_file(reader, &effective_root, &superblock.fsid, &mut mapping_fallbacks) {
        Ok(Some(content)) => RecoveryOutcome::FileRead { root: root_key, content },
        Ok(None) => RecoveryOutcome::NoFile { root: root_key },
        Err(reason) => RecoveryOutcome::Failed { root: Some(root_key), reason },
    };
    RecoveryReport { outcome, journal, mapping_fallbacks }
}

// ───────────────────────── 层 0 崩溃点枚举（D13 已定项 4）与 oracle（E77 判据 1） ─────────────────────────

/// 屏障把写流切成段；FUA 写完成才发下一条，装置把它也当段边界（可关，字节表两种口径都报）。
fn split_into_segments(operations: &[RecordedOperation], fua_is_boundary: bool) -> (Vec<WriteRequest>, Vec<Vec<usize>>) {
    let mut writes = Vec::new();
    let mut segments: Vec<Vec<usize>> = Vec::new();
    let mut current: Vec<usize> = Vec::new();
    for operation in operations {
        match operation {
            RecordedOperation::Write(write) => {
                let is_fua = write.is_fua();
                writes.push(write.clone());
                current.push(writes.len() - 1);
                if fua_is_boundary && is_fua {
                    segments.push(std::mem::take(&mut current));
                }
            }
            RecordedOperation::Barrier => {
                if !current.is_empty() {
                    segments.push(std::mem::take(&mut current));
                }
            }
        }
    }
    if !current.is_empty() {
        segments.push(current);
    }
    (writes, segments)
}

/// 每段的步骤种类，按与 `split_into_segments` 相同的切法分组——这是 D17（实现分层与第三方管道） 已定项 2
/// 说的「每段步骤种类集合」的机器可读输入（C316 ②）。关掉一段的那道屏障算进它关掉的那一段；
/// 段里还一个写都没有时（流首那道屏障）算进即将开始的那一段；流尾那一串只有屏障没有写的，并进上一段——
/// 这样录到的每一步都恰好出现在一个段里，段数也与 `split_into_segments` 一样。
/// 末尾的断言把两者钉在一起：逐段数出来的写数必须相等，对不上就是切法分了叉。
fn segment_step_kinds(operations: &[RecordedOperation], fua_is_boundary: bool) -> Vec<Vec<RecordedStepKind>> {
    let mut groups: Vec<Vec<RecordedStepKind>> = Vec::new();
    let mut current: Vec<RecordedStepKind> = Vec::new();
    let mut writes_in_current: usize = 0;
    for operation in operations {
        current.push(RecordedStepKind::of(operation));
        match operation {
            RecordedOperation::Write(write) => {
                writes_in_current += 1;
                if fua_is_boundary && write.is_fua() {
                    groups.push(std::mem::take(&mut current));
                    writes_in_current = 0;
                }
            }
            RecordedOperation::Barrier => {
                if writes_in_current > 0 {
                    groups.push(std::mem::take(&mut current));
                    writes_in_current = 0;
                }
            }
        }
    }
    if !current.is_empty() {
        match (writes_in_current, groups.last_mut()) {
            (0, Some(last_group)) => last_group.append(&mut current),
            (_, _) => groups.push(current),
        }
    }
    let (_, segments) = split_into_segments(operations, fua_is_boundary);
    let writes_per_group: Vec<usize> = groups
        .iter()
        .map(|group| group.iter().filter(|kind| **kind != RecordedStepKind::Barrier).count())
        .collect();
    assert_eq!(writes_per_group, segments.iter().map(Vec::len).collect::<Vec<usize>>(), "步骤种类的切法与 split_into_segments 对不上");
    groups
}

/// 把每段的步骤种类写成一行可解析的文字：段之间用 `|`，段内按枚举声明序排成规范多重集
/// `[种类×次数,…]`，只出现一次的不带次数。规范序是为了让两段只要多重集相同、文字就一模一样。
fn format_segment_kinds(groups: &[Vec<RecordedStepKind>]) -> String {
    let mut rendered: Vec<String> = Vec::new();
    for group in groups {
        let mut counts: BTreeMap<RecordedStepKind, usize> = BTreeMap::new();
        for kind in group {
            *counts.entry(*kind).or_insert(0) += 1;
        }
        let entries: Vec<String> = counts
            .into_iter()
            .map(|(kind, count)| if count == 1 { kind.tag().to_string() } else { format!("{}×{count}", kind.tag()) })
            .collect();
        rendered.push(format!("[{}]", entries.join(",")));
    }
    rendered.join("|")
}

/// E77 的闭式：1 + Σ(2^|段| − 1)。
fn closed_form_state_count(segments: &[Vec<usize>]) -> u64 {
    1 + segments.iter().map(|segment| (1u64 << segment.len()) - 1).sum::<u64>()
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct Layer0Tally {
    states: u64,
    violations: u64,
    root_persisted_states: u64,
    no_file_states: u64,
    file_read_states: u64,
    failed_states: u64,
    journal_differing_states: u64,
    verification_ran_states: u64,
    verification_failed_states: u64,
    first_violation: Option<String>,
}

fn oracle_violation(outcome: &RecoveryOutcome, root_persisted: bool, expected_content: &[u8]) -> Option<String> {
    match outcome {
        RecoveryOutcome::FileRead { content, .. } => (content != expected_content).then(|| "读回的内容不对".to_string()),
        RecoveryOutcome::NoFile { .. } => root_persisted.then(|| "根槽已持久而恢复到旧态".to_string()),
        RecoveryOutcome::Failed { reason, .. } => Some(format!("走读失败：{reason}")),
    }
}

fn evaluate_state(base: &Pool, writes: &[WriteRequest], persisted: Vec<bool>, root_index: usize, expected_content: &[u8], tally: &mut Layer0Tally) {
    let root_persisted = persisted[root_index];
    let image = CrashImage { base, writes, persisted };
    let consulted = recover(&image, JournalPolicy::Consult);
    let ignored = recover(&image, JournalPolicy::Ignore);
    tally.states += 1;
    if root_persisted {
        tally.root_persisted_states += 1;
    }
    if consulted.outcome != ignored.outcome {
        tally.journal_differing_states += 1;
    }
    if consulted.journal.verification_passed + consulted.journal.verification_failed > 0 {
        tally.verification_ran_states += 1;
    }
    if consulted.journal.verification_failed > 0 {
        tally.verification_failed_states += 1;
    }
    match &consulted.outcome {
        RecoveryOutcome::NoFile { .. } => tally.no_file_states += 1,
        RecoveryOutcome::FileRead { .. } => tally.file_read_states += 1,
        RecoveryOutcome::Failed { .. } => tally.failed_states += 1,
    }
    if let Some(reason) = oracle_violation(&consulted.outcome, root_persisted, expected_content) {
        tally.violations += 1;
        if tally.first_violation.is_none() {
            let persisted_kinds: Vec<&str> = image.persisted.iter().zip(writes).filter(|(is_persisted, _)| **is_persisted).map(|(_, write)| write.kind.tag()).collect();
            tally.first_violation = Some(format!("{reason}（持久的写：{}）", persisted_kinds.join("|")));
        }
    }
}

fn enumerate_layer0(base: &Pool, writes: &[WriteRequest], segments: &[Vec<usize>], expected_content: &[u8]) -> Layer0Tally {
    // 被判的是这条流里最后一次根槽 FUA 写：暖机的两个根在它前面，主臂与阳性对照都取这一条。
    let root_index = writes.iter().rposition(|write| write.kind == StepKind::RootRecordFua).expect("写流里有根槽那一条");
    let mut tally = Layer0Tally::default();
    let mut persisted_before = vec![false; writes.len()];
    for segment in segments {
        let full_mask = (1u64 << segment.len()) - 1;
        for mask in 0..full_mask {
            let mut persisted = persisted_before.clone();
            for (bit, write_index) in segment.iter().enumerate() {
                if mask & (1 << bit) != 0 {
                    persisted[*write_index] = true;
                }
            }
            evaluate_state(base, writes, persisted, root_index, expected_content, &mut tally);
        }
        for write_index in segment {
            persisted_before[*write_index] = true;
        }
    }
    evaluate_state(base, writes, persisted_before, root_index, expected_content, &mut tally);
    tally
}

// ───────────────────────── 坏字节探针（里程碑步 6 验收） ─────────────────────────

fn flip_byte(pool: &mut Pool, device: DeviceIdentity, offset: DeviceOffset, byte_index: u64) {
    let sector_offset = DeviceOffset(offset.0 + byte_index - byte_index % PHYSICAL_BLOCK_BYTES);
    let mut sector = pool.devices[device.0 as usize].read(sector_offset, PHYSICAL_BLOCK_BYTES as usize);
    sector[(byte_index % PHYSICAL_BLOCK_BYTES) as usize] ^= 0xff;
    pool.devices[device.0 as usize].write(sector_offset, &sector);
}

fn outcome_kind(outcome: &RecoveryOutcome) -> &'static str {
    match outcome {
        RecoveryOutcome::NoFile { .. } => "no_file",
        RecoveryOutcome::FileRead { .. } => "file_read",
        RecoveryOutcome::Failed { .. } => "failed",
    }
}

fn outcome_root(outcome: &RecoveryOutcome) -> String {
    match outcome {
        RecoveryOutcome::NoFile { root } | RecoveryOutcome::FileRead { root, .. } => format!("{}:{}", root.0 .0, root.1 .0),
        RecoveryOutcome::Failed { root: Some(root), .. } => format!("{}:{}", root.0 .0, root.1 .0),
        RecoveryOutcome::Failed { root: None, .. } => "none".to_string(),
    }
}

struct Probe {
    name: &'static str,
    flips: Vec<(DeviceIdentity, DeviceOffset, u64)>,
}

fn probes(parameters: &PoolParameters) -> Vec<Probe> {
    let (newest_root_region, newest_root_slot) = ring_target_for_publish(CheckpointTxg(FIRST_TRANSACTION_TXG));
    let newest_root_device = parameters.region_devices[newest_root_region as usize];
    let journal_start = DeviceOffset(JOURNAL_START_SLOT * SLOT_BYTES + (FIRST_TRANSACTION_TXG - 1) * JOURNAL_RECORD_BYTES);
    let data_offset = SlotNumber(SLOT_DATA_UNIT).device_offset();
    let tree_table_offset = SlotNumber(SLOT_TREE_TABLE_FIRST_PUBLISH).device_offset();
    let both = |offset: DeviceOffset, byte: u64| vec![(DeviceIdentity(0), offset, byte), (DeviceIdentity(1), offset, byte)];
    vec![
        Probe { name: "newest_root_slot_one_byte", flips: vec![(newest_root_device, ring_slot_offset(newest_root_region, newest_root_slot), 100)] },
        Probe { name: "journal_record_both_copies", flips: both(journal_start, 200) },
        Probe { name: "journal_record_one_copy", flips: vec![(DeviceIdentity(0), journal_start, 200)] },
        Probe { name: "data_payload_one_copy", flips: vec![(DeviceIdentity(0), data_offset, 200)] },
        Probe { name: "data_payload_both_copies", flips: both(data_offset, 200) },
        Probe { name: "data_header_last_byte_both_copies", flips: both(data_offset, DATA_UNIT_HEADER_BYTES - 1) },
        Probe { name: "tree_table_both_copies", flips: both(tree_table_offset, 300) },
        Probe { name: "superblock_slot_one_both_devices", flips: both(DeviceOffset(SUPERBLOCK_SLOT_OFFSETS[1]), 50) },
    ]
}

fn run_probe(full: &Pool, probe: &Probe) -> RecoveryReport {
    let mut damaged = full.clone();
    for (device, offset, byte_index) in &probe.flips {
        flip_byte(&mut damaged, *device, *offset, *byte_index);
    }
    recover(&damaged, JournalPolicy::Consult)
}

// ───────────────────────── 装置自己补的取法：空白清单 ─────────────────────────

/// 每一条是仓里没有条款、装置不得不自己取一个值才写得出字节的地方。文字里不用空格，好让结果行按空格切段。
const GAPS: &[(&str, &str)] = &[
    ("G1", "已收口（2026-09-13 D8已定项11 + D18已定项16/18）：码2头=86+2×key宽，含29字节预留位115+2×key宽（inode与树表131、分配135、记账159、extent163、映射169）；跑出空白那天三处预想数（84、68+44、72/81/90三档）互不相等"),
    ("G2", "已收口（2026-09-14用户定案，C325还清）：key宽字段从81+2k挪到**偏移51**，它自己的偏移不依赖k⇒扫描期按[0,51)这段定长前缀读出k、一次定出头末端，I-2.4对码2从此直接判得了；装置的parse_index_node不再收调用方给的key宽"),
    ("G3", "已收口（2026-09-14用户定案改写D19已定项10）：映射key**一律27**——码2/码3的25字节末尾补零到27、条目一律55；2026-09-13那一版的「按类27/25不补齐、条目55/53」作废"),
    ("G4", "根记录字段序：D22已定项7的表与字节表七的表行序不同（水位、F、校验和、实例表指针四行的先后），两处都没写偏移；2026-09-14加的算法类型1+nonce12+MAC16同样只说「放在中央映射树根指针之后」、没写偏移；装置按D22的表序"),
    ("G5", "已收口（2026-09-13 D18已定项17）：32字节校验和字段里放CRC32C4字节+28字节零，自证结构罩整个槽含补齐、字段按零参与；跑出空白那天装置取的是SHA-256"),
    ("G6", "已收口（2026-09-13 D23已定项19②，2026-09-14改写口径）：previous_hash=CRC32C(**本实例内逻辑前一条**记录的307字节头，header_csum那32字节按零参与)，本实例写出的第一条恒0、不读盘"),
    ("G7", "已收口（2026-09-13 D23已定项15）：记录头加新根段188（树表指针86+映射树根指针86+树ID水位8+回退下界F8），施加一条记录=把所选根这四个字段换成记录里的；实例代号与txg照记录头、实例表指针照所选根⇒journal从此承重（journal_effect差异态3个）"),
    ("G8", "码1映射key靠D16的事务切分纪律（一个事务最多写一个单元的用户数据）才唯一：同一事务写两个数据单元key相同；按D23已定项7一条记录一个事务，每个数据单元一条4KiB记录"),
    ("G9", "journal两份镜像何时算「记录在」没有条款（一份合法即在、还是两份都要）；装置取任一份合法即在"),
    ("G10", "里程碑步4验收「把位置提示改坏、经映射仍读到」在字节层做不到：提示住父节点、父节点被树表指针里的整单元CRC罩着，改坏提示先红父节点；要验的是搬走单元那条路"),
    ("G11", "里程碑步1验收「同参数两次mkfs逐字节相同」与字节表「fsid=mkfs随机」矛盾：fsid必须是mkfs参数"),
    ("G12", "已收口（2026-09-13 D15已定项4）：incompat位0=第一条纯SSD布局线，装置从mkfs起置上；位图其余全0"),
    ("G13", "已收口（2026-09-13 D8已定项11）：条目数2与条目宽2进头（偏移82+2k、84+2k），不再住载荷内部布局"),
    ("G14", "实例表链指针行宽2026-09-13从64改成88（C304，D18已定项11）；2026-09-14指针从83变86之后那3字节预留被吃掉，记录宽仍88⇒**这条记录没有余量了**，指针再宽一个字节INSTANCE_ROW_BYTES就要跟着涨（打包记录定长，kind0与kind1必须同宽），而条款里没有这一句"),
    ("G15", "已收口（2026-09-13 D18已定项18）：码2的声明长度=条目数×条目宽，条目区之后的补齐区恒0且参与载荷CRC，I-2.3射程扩到码2"),
    ("G16", "已收口（2026-09-13 D19已定项9）：出生序号从0起、同一棵树内码2与码3共用一个计数、换checkpoint清零、同一checkpoint里重写换新号"),
    ("G17", "已收口（2026-09-13 C313用户定案）：FUA写算段边界；装置主臂按它枚举，另一读法只报数不判"),
    ("G18", "D23已定项12按「12项事务恰占1条记录」算余量，而D16的事务切分纪律让一次带8个数据单元的fsync至少是8个事务、8条记录；两条已定条款对同一负载算出的记录数不同"),
    ("G19", "mkfs种根的13次操作（11写+2屏障，段序列4+1+1+1+4、34个崩溃状态）不在层0枚举里：装置从mkfs之后的池起枚举（取号、暖机与事务），mkfs的崩溃状态没有任何东西判；段序列另发一行钉住"),
    ("G20", "已收口（2026-09-14用户定案，C321还清）：映射条目回到一宽55⇒码2头那个u16的条目宽字段够用，声明长度=条目数×条目宽原样成立；装置删掉了「条目宽0当变长哨兵」那套自创取法，parse_index_node对条目宽0一律判结构错"),
    ("G21", "取号那一步（D23已定项16：第一次可写挂载写每一份超级块之后才动单元）不是根槽写路径，first-txn-layout八那张表罩不到它；屏障怎么放没有条款，装置按最少屏障取「不另加屏障，靠暖机第一次空发布开头那道屏障收段」⇒段序列独占一行[superblock_slot×2]"),
    ("G22", "**仍欠着**（C323，2026-09-14加注）：镜像大小（单元区的末端）全仓仍没有条款，D23已定项19③只定了「环≤设备容量÷4」⇒它给出容量的**下界**而不是值；装置按mkfs参数取4GiB（1GiB装不下默认768MiB的环）并在name=config里报出来，空闲字节3472670720、全空聚簇段数3310、runs4三个数都随它变"),
    ("G23", "已收口（2026-09-14用户定案，C324还清）：码3打包容器按「数据单元」那一档取落点（起点32768对齐、两槽都空）；连同D3已定项10⑤的bump次序（树ID升序、树内先叶后根、映射树倒数第二、树表最末）⇒t2 extent根拿50240、游标停在50241而t3要对齐⇒t3拿50242–50243，**槽50241空着**、runs因此从3变4"),
    ("G24", "树表条目的「头ID」（D5已定项9，2026-09-14用户定案）只说了inode树写自己、extent树写12、其余写0，没说**这个数从哪来**：第一版只有一个可写头，装置按「inode树的树ID就是头ID」写；多头之后头ID与树ID还是不是一回事，条款没答"),
    ("G25", "journal记录头2026-09-14加的fsid8与MAC16，条款只说fsid「与单元头同口径」（超级块fsid的低8字节）、MAC第一版全0；**读者拿fsid做什么没写**——装置按I-1.4把fsid不符的记录整条丢掉（不进重放前缀），而「丢掉」与「判损坏断链」在条款里分不出来"),
];

// ───────────────────────── main：发结果行 ─────────────────────────

fn emit(emitter: &mut Emitter, body: &str) {
    println!("{}", emitter.emit_raw(body));
}

fn main() {
    let mut emitter = Emitter::new();
    let file_bytes: Vec<u8> = (0..3000u32).map(|index| u8::try_from((index * 7 + 3) % 251).expect("小于 256")).collect();
    emit(&mut emitter, &format!(
        "name=config devices=2 physical_block_bytes={PHYSICAL_BLOCK_BYTES} file_bytes={} fsid=fixed device_bytes={DEVICE_BYTES} journal_ring_bytes={JOURNAL_RING_BYTES} journal_ring_slots={JOURNAL_RING_SLOTS} in_flight_limit={JOURNAL_IN_FLIGHT_RECORD_LIMIT} unit_area_start_slot={UNIT_AREA_START_SLOT} unit_area_slots={UNIT_AREA_SLOTS} cluster_segment_slots={CLUSTER_SEGMENT_SLOTS} open_cluster_segment_start={OPEN_CLUSTER_SEGMENT_START_SLOT}",
        file_bytes.len()
    ));

    // 判据 1：宽度对账。
    // 「预期」那一列逐格抄自 `.claude/kb/first-txn-layout.md` 零到七（2026-09-14 用户定案之后的那一版），
    // 「实测」是装置自己的常量算出来的。两列不等就记一笔 `width_mismatches`。
    let width_rows: [(&str, u64, u64); 28] = [
        ("pointer_head", 50, POINTER_HEAD_BYTES),
        ("data_unit_header", 105, DATA_UNIT_HEADER_BYTES),
        ("packed_unit_header", 107, PACKED_UNIT_HEADER_BYTES),
        ("unit_reserved", 29, NONCE_MAC_RESERVED_BYTES),
        ("data_unit_header_with_reserved", 134, DATA_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES),
        ("packed_unit_header_with_reserved", 136, PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES),
        ("root_record", 371, ROOT_RECORD_BYTES),
        ("tree_table_entry", 148, TREE_TABLE_ENTRY_BYTES),
        ("journal_header", 307, JOURNAL_HEADER_BYTES),
        ("journal_header_ten_fields", 78, JOURNAL_HEADER_TEN_FIELD_BYTES),
        ("journal_new_root_segment", 188, JOURNAL_NEW_ROOT_SEGMENT_BYTES),
        ("journal_named_entry", 56, JOURNAL_NAMED_ENTRY_BYTES),
        ("journal_named_entries_per_record", 67, (JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES) / JOURNAL_NAMED_ENTRY_BYTES),
        ("data_pointer", 88, DATA_POINTER_BYTES),
        ("node_pointer", 86, NODE_POINTER_BYTES),
        ("instance_table_row", 88, INSTANCE_ROW_BYTES),
        ("inode_record", 140, INODE_RECORD_BYTES),
        ("inode_internal_entry", 120, INODE_INTERNAL_ENTRY),
        ("extent_leaf_record", 112, EXTENT_LEAF_RECORD_BYTES),
        ("allocation_record", 20, ALLOCATION_RECORD_BYTES),
        ("allocation_key", 10, ALLOCATION_KEY_BYTES as u64),
        ("allocation_value", 10, ALLOCATION_RECORD_BYTES - ALLOCATION_KEY_BYTES as u64),
        ("accounting_entry", 34, ACCOUNTING_ENTRY_BYTES),
        ("mapping_key", 27, MAPPING_KEY_BYTES),
        ("mapping_entry", 55, MAPPING_ENTRY_BYTES),
        ("location_entry", 14, LOC_ENTRY),
        ("superblock", 481, SUPERBLOCK_BYTES),
        ("superblock_slot", 4096, SUPERBLOCK_SLOT_BYTES),
    ];
    let mut width_mismatches = 0u64;
    for (structure, expected, actual) in width_rows {
        if expected != actual {
            width_mismatches += 1;
        }
        emit(&mut emitter, &format!("name=width structure={structure} expected={expected} actual={actual}"));
    }

    let parameters = PoolParameters::settled_two_devices();
    let (mut recording, genesis) = mkfs(&parameters);
    let mkfs_operation_count = recording.operations.len();
    let instance = acquire_instance(&mut recording, &parameters);
    let acquisition_operation_count = recording.operations.len();
    let (warm_up_roots, last_warm_up_record) = warm_up(&mut recording, &parameters, &genesis, instance);
    let warm_up_operation_count = recording.operations.len();
    let output = publish_first_file(&mut recording, &parameters, &genesis, &file_bytes, instance, last_warm_up_record.as_deref());
    let warm_up_operations = &recording.operations[acquisition_operation_count..warm_up_operation_count];
    emit(&mut emitter, &format!(
        "name=warm_up publishes={} writes={} barriers={} fua={} first_transaction_txg={FIRST_TRANSACTION_TXG} last_warm_up_root_txg={}",
        warm_up_roots.len(),
        warm_up_operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(_))).count(),
        warm_up_operations.iter().filter(|operation| matches!(operation, RecordedOperation::Barrier)).count(),
        warm_up_operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(write) if write.is_fua())).count(),
        warm_up_roots.last().map_or(0, |root| root.checkpoint_txg.0)
    ));
    // 写清单只数第一个事务；层 0 枚举吃 mkfs 之后的全部操作（取号 + 暖机两次空发布 + 第一个事务）
    let transaction_operations = &recording.operations[warm_up_operation_count..];
    let post_mkfs_operations = &recording.operations[mkfs_operation_count..];
    let acquisition_operations = &recording.operations[mkfs_operation_count..acquisition_operation_count];
    // 段序列登记表（first-txn-layout 八）的输入：每条路径单独切段、再加整条流。mkfs 那一行不进层 0 枚举（G19），但段序列钉在这里。
    for (path_name, operations) in [("mkfs", &recording.operations[..mkfs_operation_count]), ("instance_acquisition", acquisition_operations), ("warm_up", warm_up_operations), ("transaction", transaction_operations), ("post_mkfs_stream", post_mkfs_operations)] {
        let (_, path_segments) = split_into_segments(operations, true);
        let sizes: Vec<String> = path_segments.iter().map(|segment| segment.len().to_string()).collect();
        emit(&mut emitter, &format!("name=segments path={path_name} operations={} segments={} closed_form={} kinds={}", operations.len(), sizes.join("+"), closed_form_state_count(&path_segments), format_segment_kinds(&segment_step_kinds(operations, true))));
    }
    let write_count = transaction_operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(_))).count();
    let barrier_count = transaction_operations.iter().filter(|operation| matches!(operation, RecordedOperation::Barrier)).count();
    let fua_count = transaction_operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(write) if write.is_fua())).count();
    let instance_table = parse_packed_unit(&genesis.instance_table_unit).expect("mkfs 写出的实例表单元自检要过");
    let tree_table_genesis = parse_index_node(&genesis.tree_table_genesis_unit).expect("mkfs 写出的树表单元自检要过");
    emit(&mut emitter, &format!("name=mkfs_units instance_table_records={} instance_table_row_bytes={} tree_table_entries={} genesis_root_watermark={}", instance_table.records.len(), instance_table.record_width, tree_table_genesis.entries.len(), genesis.root.tree_identifier_watermark));
    emit(&mut emitter, &format!("name=root_record checkpoint_txg={} instance={} tree_identifier_watermark={} rollback_floor={} record_bytes={} back_chain={}", output.root.checkpoint_txg.0, output.root.instance.0, output.root.tree_identifier_watermark, output.root.rollback_floor.0, output.record_bytes.len(), output.record.back_chain));
    let slots: Vec<String> = output.units_by_slot.iter().map(|(slot, unit_identity, unit)| format!("{}@{}x{}", unit_identity.tag(), slot.0, unit.len())).collect();
    emit(&mut emitter, &format!("name=write_list writes={write_count} barriers={barrier_count} fua={fua_count} named={} units={}", output.record.named.len(), slots.join(",")));
    for (tree, width) in &output.index_node_header_widths {
        emit(&mut emitter, &format!("name=index_node_header tree={tree} header_bytes={width} with_reserved={}", width + NONCE_MAC_RESERVED_BYTES as usize));
    }

    // 判据 3：读回。
    let full = recording.pool.clone();
    let full_report = recover(&full, JournalPolicy::Consult);
    let content_matches = matches!(&full_report.outcome, RecoveryOutcome::FileRead { content, .. } if *content == file_bytes);
    emit(&mut emitter, &format!(
        "name=recover_full outcome={} root={} content_matches={content_matches} valid_records={} above_water={} applied={} verification_passed={} mapping_fallbacks={}",
        outcome_kind(&full_report.outcome), outcome_root(&full_report.outcome), full_report.journal.valid_records, full_report.journal.above_water, full_report.journal.prefix_applied, full_report.journal.verification_passed, full_report.mapping_fallbacks
    ));

    // 判据 4：层 0 主臂。基线是 mkfs 之后的池（事务的写全没持久那一态）。
    let (base_pool, _) = mkfs(&parameters);
    let base_pool = base_pool.pool;
    let (writes, segments) = split_into_segments(post_mkfs_operations, true);
    let segment_sizes: Vec<String> = segments.iter().map(|segment| segment.len().to_string()).collect();
    let closed_form = closed_form_state_count(&segments);
    let tally = enumerate_layer0(&base_pool, &writes, &segments, &file_bytes);
    emit(&mut emitter, &format!(
        "name=layer0 arm=settled_two_devices segments={} states={} closed_form={closed_form} violations={} root_persisted_states={} no_file={} file_read={} failed={} verification_ran={} verification_failed={} first_violation={}",
        segment_sizes.join("+"), tally.states, tally.violations, tally.root_persisted_states, tally.no_file_states, tally.file_read_states, tally.failed_states, tally.verification_ran_states, tally.verification_failed_states,
        tally.first_violation.clone().unwrap_or_else(|| "none".to_string()).replace(' ', "_")
    ));
    let (_, segments_fua_free) = split_into_segments(post_mkfs_operations, false);
    emit(&mut emitter, &format!("name=layer0_fua_not_boundary segments={} closed_form={}", segments_fua_free.iter().map(|segment| segment.len().to_string()).collect::<Vec<_>>().join("+"), closed_form_state_count(&segments_fua_free)));
    emit(&mut emitter, &format!("name=journal_effect arm=settled_two_devices states={} differing_states={}", tally.states, tally.journal_differing_states));

    // 判据 5：阳性对照——一块盘、不放屏障，oracle 必须分得出「根在而单元不在」。
    let control = PoolParameters::control_one_device_no_barriers();
    let (mut control_recording, control_genesis) = mkfs(&control);
    let control_mkfs_operation_count = control_recording.operations.len();
    let _ = publish_first_file(&mut control_recording, &control, &control_genesis, &file_bytes, InstanceGeneration(FIRST_INSTANCE_GENERATION), None);
    let (control_base, _) = mkfs(&control);
    let (control_writes, control_segments) = split_into_segments(&control_recording.operations[control_mkfs_operation_count..], false);
    let control_closed_form = closed_form_state_count(&control_segments);
    let control_tally = enumerate_layer0(&control_base.pool, &control_writes, &control_segments, &file_bytes);
    let control_expected_violations = (1u64 << (control_writes.len() - 1)) - (1u64 << (control_writes.len() - 1 - 8));
    emit(&mut emitter, &format!(
        "name=layer0_control arm=one_device_no_barriers writes={} states={} closed_form={control_closed_form} violations={} expected_violations={control_expected_violations} root_persisted_states={} failed={}",
        control_writes.len(), control_tally.states, control_tally.violations, control_tally.root_persisted_states, control_tally.failed_states
    ));

    // 判据 7：坏字节探针。
    for probe in probes(&parameters) {
        let report = run_probe(&full, &probe);
        let reason = match &report.outcome {
            RecoveryOutcome::Failed { reason, .. } => reason.replace(' ', "_"),
            _ => "none".to_string(),
        };
        let content_matches = matches!(&report.outcome, RecoveryOutcome::FileRead { content, .. } if *content == file_bytes);
        emit(&mut emitter, &format!(
            "name=probe probe={} outcome={} root={} content_matches={content_matches} valid_records={} applied={} mapping_fallbacks={} reason={reason}",
            probe.name, outcome_kind(&report.outcome), outcome_root(&report.outcome), report.journal.valid_records, report.journal.prefix_applied, report.mapping_fallbacks
        ));
    }

    // 算术：码 1 映射 key 在同一事务写两个数据单元时相同；D16 的事务切分纪律下 1 MiB 写要几条记录。
    let first_key = mapping_key_for_data(output.data_pointer.head, output.data_pointer.write_order);
    let second_unit_same_transaction = mapping_key_for_data(output.data_pointer.head, output.data_pointer.write_order);
    let distinct_keys = if first_key == second_unit_same_transaction { 1 } else { 2 };
    emit(&mut emitter, &format!("name=mapping_key_collision same_transaction_data_units=2 distinct_keys={distinct_keys} mapping_entries_first_transaction={}", output.mapping_keys.len()));
    // D23 已定项 17 末句：重放从点名项直接凑出映射 key——凑出来的这几个必须覆盖映射树里那 6 条。
    let rebuilt_keys: Vec<Vec<u8>> = output.record.named.iter().map(NamedUnit::mapping_key).collect();
    let covered = output.mapping_keys.iter().filter(|key| rebuilt_keys.contains(key)).count();
    emit(&mut emitter, &format!("name=named_mapping_keys named={} rebuilt_from_named={} mapping_entries={} covered={covered}", output.record.named.len(), rebuilt_keys.len(), output.mapping_keys.len()));
    let one_mebibyte_units = (1u64 << 20) / DATA_UNIT_BYTES;
    let journal_bytes_two_devices = one_mebibyte_units * JOURNAL_RECORD_BYTES * 2;
    emit(&mut emitter, &format!(
        "name=journal_per_mebibyte data_units={one_mebibyte_units} transactions_under_split_rule={one_mebibyte_units} records={one_mebibyte_units} journal_bytes_two_devices={journal_bytes_two_devices} journal_over_data_percent={}",
        journal_bytes_two_devices * 100 / (2 << 20)
    ));
    let statistic_value = |statistic: u16| output.accounting_entries.iter().find(|entry| entry.statistic == statistic).map_or(0, |entry| entry.value);
    emit(&mut emitter, &format!(
        "name=accounting entries={} allocated_bytes_per_device={} free_bytes_per_device={} empty_cluster_segments_per_device={} fragmentation_runs_per_device={} inode_watermark={} unreclaimable_per_device={} defer_queue_per_device={} pending_delete={} committed_reservation={} sequence_all_one={} generation={}",
        output.accounting_entries.len(),
        statistic_value(STATISTIC_ALLOCATED_BYTES),
        statistic_value(STATISTIC_FREE_BYTES),
        statistic_value(STATISTIC_EMPTY_CLUSTER_SEGMENTS),
        statistic_value(STATISTIC_FRAGMENTATION_RUNS),
        statistic_value(STATISTIC_INODE_WATERMARK),
        statistic_value(STATISTIC_UNRECLAIMABLE_BYTES),
        statistic_value(STATISTIC_DEFER_QUEUE_BYTES),
        statistic_value(STATISTIC_PENDING_DELETE_BYTES),
        statistic_value(STATISTIC_COMMITTED_RESERVATION_BYTES),
        output.accounting_entries.iter().all(|entry| entry.sequence == ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF),
        output.accounting_entries[0].generation.0
    ));
    emit(&mut emitter, &format!(
        "name=allocation records={} first_slot={} last_slot={} key_bytes={} value_bytes={} mkfs_generation_records={}",
        output.allocation_records.len(),
        output.allocation_records[0].slot.0,
        output.allocation_records[output.allocation_records.len() - 1].slot.0,
        ALLOCATION_KEY_BYTES,
        ALLOCATION_RECORD_BYTES as usize - ALLOCATION_KEY_BYTES,
        output.allocation_records.iter().filter(|record| record.generation == CheckpointTxg(0)).count()
    ));
    emit(&mut emitter, &format!(
        "name=instances mkfs_instance={MKFS_INSTANCE_GENERATION} first_writable_mount_instance={FIRST_INSTANCE_GENERATION} mkfs_superblock_generation={SUPERBLOCK_GENERATION_AT_MKFS} acquisition_superblock_generation={SUPERBLOCK_GENERATION_AT_INSTANCE_ACQUISITION} transaction_superblock_generation={} transaction_superblock_slot={}",
        superblock_write_for_publish(CheckpointTxg(FIRST_TRANSACTION_TXG)).0,
        superblock_write_for_publish(CheckpointTxg(FIRST_TRANSACTION_TXG)).1
    ));
    // 反向链（D23 已定项 19 ②）：环上计数器为 1 的那条恒 0，其余罩前一条的整个头。
    let records_on_disk = scan_journal(&full, unit_fsid(&FIXED_FSID));
    let back_chain_by_counter: Vec<String> = records_on_disk.values().map(|record| format!("{}:{}", record.counter.0, record.back_chain)).collect();
    emit(&mut emitter, &format!("name=back_chain records={} chains={}", records_on_disk.len(), back_chain_by_counter.join(",")));

    // 判据 8：空白清单。
    for (gap_identifier, text) in GAPS {
        emit(&mut emitter, &format!("name=gap id={gap_identifier} text={text}"));
    }
    emit(&mut emitter, &format!("name=gaps count={}", GAPS.len()));

    // 判据 9（2026-09-13 加，D15 已定项 4）：feature bits 的实际字节与「不认识不许挂」。
    let superblock_slot = recording.pool.read(DeviceIdentity(0), DeviceOffset(SUPERBLOCK_SLOT_OFFSETS[0]), SUPERBLOCK_SLOT_BYTES as usize);
    let feature_bits = &superblock_slot[SUPERBLOCK_FEATURE_BITS_OFFSET..SUPERBLOCK_FEATURE_BITS_OFFSET + 3 * FEATURE_BITMAP_BYTES];
    let mut unknown_incompat_slot = superblock_slot.clone();
    unknown_incompat_slot[SUPERBLOCK_FEATURE_BITS_OFFSET] = 0x03;
    let mut no_layout_slot = superblock_slot.clone();
    no_layout_slot[SUPERBLOCK_FEATURE_BITS_OFFSET] = 0x00;
    emit(&mut emitter, &format!(
        "name=feature_bits incompat_byte0={:#04x} incompat_rest_zero={} compat_ro_zero={} compat_zero={} refuses_unknown_incompat={} refuses_missing_layout_bit={}",
        feature_bits[0],
        feature_bits[1..FEATURE_BITMAP_BYTES].iter().all(|byte| *byte == 0),
        feature_bits[FEATURE_BITMAP_BYTES..2 * FEATURE_BITMAP_BYTES].iter().all(|byte| *byte == 0),
        feature_bits[2 * FEATURE_BITMAP_BYTES..].iter().all(|byte| *byte == 0),
        !incompat_bits_are_mountable(&unknown_incompat_slot),
        !incompat_bits_are_mountable(&no_layout_slot),
    ));

    emit(&mut emitter, &format!(
        "name=verdict width_mismatches={width_mismatches} write_list_ok={} recover_full_ok={content_matches} layer0_states_ok={} layer0_violations={} control_states_ok={} control_violations_ok={} journal_differing_states={}",
        write_count == 21 && barrier_count == 2 && fua_count == 1,
        tally.states == closed_form && closed_form == 262165,
        tally.violations,
        control_tally.states == control_closed_form && control_closed_form == 2048,
        control_tally.violations == control_expected_violations,
        tally.journal_differing_states
    ));
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_file() -> Vec<u8> {
        (0..3000u32).map(|index| u8::try_from((index * 7 + 3) % 251).expect("小于 256")).collect()
    }

    /// mkfs → 取号 → 暖机两次空发布 → 第一个事务。返回的三个下标分别是 mkfs 之后（取号从这里起）、
    /// 取号之后（暖机从这里起）、暖机之后（事务从这里起）。
    struct BuiltPool {
        recording: RecordingPool,
        genesis: MkfsOutput,
        output: TransactionOutput,
        mkfs_operation_count: usize,
        acquisition_operation_count: usize,
        warm_up_operation_count: usize,
    }

    fn built_pool() -> BuiltPool {
        let parameters = PoolParameters::settled_two_devices();
        let (mut recording, genesis) = mkfs(&parameters);
        let mkfs_operation_count = recording.operations.len();
        let instance = acquire_instance(&mut recording, &parameters);
        let acquisition_operation_count = recording.operations.len();
        let (_, last_warm_up_record) = warm_up(&mut recording, &parameters, &genesis, instance);
        let warm_up_operation_count = recording.operations.len();
        let output = publish_first_file(&mut recording, &parameters, &genesis, &sample_file(), instance, last_warm_up_record.as_deref());
        BuiltPool { recording, genesis, output, mkfs_operation_count, acquisition_operation_count, warm_up_operation_count }
    }

    #[test]
    fn crc32c_matches_the_published_check_value() {
        assert_eq!(castagnoli_crc32(b"123456789"), 0xE306_9283);
        assert_eq!(castagnoli_crc32(&[]), 0);
    }

    /// D18（块里携带什么信息） 已定项 17：32 字节的校验和字段 = CRC32C 4 字节 + 28 字节零，字段自身按零参与。
    #[test]
    fn wide_checksum_is_crc32c_in_four_bytes_then_twenty_eight_zeroes() {
        let mut slot = vec![0u8; PHYSICAL_BLOCK_BYTES as usize];
        for (index, byte) in slot.iter_mut().enumerate() {
            *byte = u8::try_from(index % 251).expect("小于 256");
        }
        let digest = wide_checksum_with_field_zeroed(&slot, PHYSICAL_BLOCK_BYTES as usize, 10);
        assert!(digest[WIDE_CHECKSUM_CRC_BYTES..].iter().all(|byte| *byte == 0), "后 28 字节恒零");
        let mut zeroed = slot.clone();
        zeroed[10..42].fill(0);
        assert_eq!(u32::from_le_bytes(digest[..4].try_into().expect("切了 4 字节")), castagnoli_crc32(&zeroed));
        assert_eq!(WIDE_CHECKSUM_CRC_BYTES + 28, WIDE_CHECKSUM_BYTES as usize);
    }

    #[test]
    fn widths_equal_the_byte_table_and_the_root_record_is_371() {
        // 每一行左边是字节表零到七写死的绝对值，右边是装置自己算出来的——不是几个常量互相比。
        assert_eq!(ROOT_RECORD_BYTES, 371);
        assert_eq!(ROOT_CHECKSUM_OFFSET, 138, "自证校验和在根记录里的偏移");
        assert_eq!(ROOT_CHECKSUM_OFFSET as u64 + WIDE_CHECKSUM_BYTES + 2 * NODE_POINTER_BYTES + 1 + 12 + 16, ROOT_RECORD_BYTES);
        assert_eq!(POINTER_HEAD_BYTES, 50);
        assert_eq!(POINTER_HEAD_BYTES + 2 * LOC_ENTRY + 4 + 4, NODE_POINTER_BYTES);
        assert_eq!(POINTER_HEAD_BYTES + 2 * LOC_ENTRY + 10, DATA_POINTER_BYTES);
        assert_eq!(NODE_POINTER_BYTES, 86);
        assert_eq!(DATA_POINTER_BYTES, 88);
        assert_eq!(JOURNAL_HEADER_CHECKSUM_OFFSET, 46);
        assert_eq!(JOURNAL_HEADER_TEN_FIELD_BYTES, 78);
        assert_eq!(JOURNAL_HEADER_CHECKSUM_OFFSET as u64 + WIDE_CHECKSUM_BYTES, JOURNAL_HEADER_TEN_FIELD_BYTES);
        assert_eq!(JOURNAL_HEADER_TEN_FIELD_BYTES + 8 + 1 + 4 + 4 + JOURNAL_NEW_ROOT_SEGMENT_BYTES + 8 + 16, JOURNAL_HEADER_BYTES);
        assert_eq!(JOURNAL_HEADER_BYTES, 307);
        assert_eq!(JOURNAL_NEW_ROOT_SEGMENT_BYTES, 188);
        assert_eq!(JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES, 3789, "4096 − 307");
        assert_eq!((JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES) / JOURNAL_NAMED_ENTRY_BYTES, 67, "4096 的记录装 67 个点名项");
        assert_eq!(SUPERBLOCK_CHECKSUM_OFFSET, 155);
        assert_eq!(SUPERBLOCK_FSID_OFFSET, 102);
        assert_eq!(SUPERBLOCK_BYTES, 481);
        assert_eq!(SUPERBLOCK_SLOT_BYTES, 4096, "超级块槽宽是格式常量（D22 已定项 2，2026-09-14 三方论证后按主 agent 推荐值）");
        assert_eq!(SUPERBLOCK_SLOT_BYTES - SUPERBLOCK_BYTES, 3615, "481 的超级块在 4096 槽里余 3615");
        assert_eq!(SUPERBLOCK_SLOT_OFFSETS[1] - SUPERBLOCK_SLOT_OFFSETS[0], SUPERBLOCK_SLOT_BYTES, "槽距与槽宽同值 ⇒ 两个槽首尾相接、不重叠");
        assert_eq!(SUPERBLOCK_TAIL_OFFSET as u64 + 8 + 4, SUPERBLOCK_BYTES);
        assert_eq!(MAPPING_KEY_BYTES, 27);
        assert_eq!(MAPPING_KEY_BYTES + 2 * LOC_ENTRY, MAPPING_ENTRY_BYTES);
        assert_eq!(MAPPING_ENTRY_BYTES, 55, "映射条目一律 55（D19 已定项 10，2026-09-14 用户定案）");
        assert_eq!(MAPPING_KEY_NODE_SIGNIFICANT_BYTES + 2, MAPPING_KEY_BYTES, "码 2 / 码 3 的 key 末尾补两个零");
        assert_eq!(ALLOCATION_KEY_BYTES as u64 + 10, ALLOCATION_RECORD_BYTES);
        assert_eq!(24 + DATA_POINTER_BYTES, EXTENT_LEAF_RECORD_BYTES);
        assert_eq!(EXTENT_LEAF_RECORD_BYTES, 112);
        assert_eq!(8 + 26 + NODE_POINTER_BYTES, INODE_INTERNAL_ENTRY);
        assert_eq!(INODE_INTERNAL_ENTRY, 120);
        assert_eq!(TREE_TABLE_ENTRY_BYTES, 148);
        assert_eq!(8 + 2 + 2 + 2 + NODE_POINTER_BYTES + 8 + 8 + 8 + 24, TREE_TABLE_ENTRY_BYTES);
        assert_eq!(NONCE_MAC_RESERVED_BYTES, 29, "nonce 12 + MAC 16 + 算法类型 1（D18 已定项 16，2026-09-14 用户定案）");
        assert_eq!(DATA_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES, 134, "码 1 载荷从 134 起");
        assert_eq!(PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES, 136, "码 3 记录区从 136 起");
        assert_eq!((DATA_UNIT_BYTES - (PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES)) / JOURNAL_NAMED_ENTRY_BYTES, 582, "I-1.7 的 cap(136, 56)");
        assert_eq!((DATA_UNIT_BYTES - (PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES)) / INSTANCE_ROW_BYTES, 370, "实例表一片 370 条");
        // D23 已定项 18 / 19 ③：环 768 MiB ⇒ 196608 个 4 KiB 槽，在飞上限 = 槽数 ÷ 3 = 65536；单元区起于槽 50176。
        assert_eq!(JOURNAL_RING_BYTES, 805_306_368);
        assert_eq!(JOURNAL_RING_SLOTS, 196_608);
        assert_eq!(JOURNAL_IN_FLIGHT_RECORD_LIMIT, 65_536);
        // 钉绝对值：这一格此前写成记录宽 4096，而合计对账只对宽度不对取值，抓不到。
        assert_eq!(JOURNAL_WORST_CASE_BYTES, 268_435_456);
        assert_eq!(JOURNAL_WORST_CASE_BYTES, JOURNAL_IN_FLIGHT_RECORD_LIMIT * JOURNAL_RECORD_BYTES);
        assert_eq!(UNIT_AREA_START_SLOT, 50_176);
        // D23 已定项 19 ③ 的「环 ≤ 设备容量 ÷ 4」：768 MiB 的环要 3 GiB 以上的盘，装置取 4 GiB。
        assert_eq!(DEVICE_BYTES, 4_294_967_296);
        assert!(JOURNAL_RING_BYTES * 4 <= DEVICE_BYTES, "环 ≤ 设备容量 ÷ 4");
        assert_eq!(UNIT_AREA_SLOTS, 211_968, "4 GiB 镜像 262144 槽减去固定结构占的 50176 槽");
        assert_eq!(UNIT_AREA_SLOTS / CLUSTER_SEGMENT_SLOTS, 3312, "单元区整 3312 个 64 槽聚簇段");
    }

    #[test]
    /// D8（核心索引结构） 已定项 11 + D18（块里携带什么信息） 已定项 16：头 86 + 2k，含 29 字节预留位 115 + 2k。
    /// 六个数逐个抄自字节表（inode 树根 131、extent 163、分配记录 135、记账 159、映射 169、树表 131）。
    fn index_node_header_is_86_plus_twice_the_key_width() {
        let with_reserved = |key_width: usize| index_node_header_bytes(key_width) + NONCE_MAC_RESERVED_BYTES as usize;
        assert_eq!(index_node_header_bytes(8), 102);
        assert_eq!(with_reserved(8), 131, "inode 树与树表");
        assert_eq!(with_reserved(ALLOCATION_KEY_BYTES), 135, "分配记录树（key 宽 10）");
        assert_eq!(with_reserved(22), 159, "记账树");
        assert_eq!(with_reserved(24), 163, "extent 树");
        assert_eq!(with_reserved(MAPPING_KEY_BYTES as usize), 169, "中央映射树（key 宽 27）");
        assert_eq!(index_node_header_bytes(0), INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE as usize);
        // 树表一层装几条：分母是 16384 减含预留位的码 2 头，不减别的数（D8 已定项 8 写死的口径）。
        assert_eq!((NODE_BYTES as usize - with_reserved(TREE_TABLE_KEY_WIDTH)) / TREE_TABLE_ENTRY_BYTES as usize, 109);
        // inode 树内部节点一层装几条：头 131、条目 120。
        assert_eq!((NODE_BYTES as usize - with_reserved(8)) / INODE_INTERNAL_ENTRY as usize, 135);
    }

    /// key 宽住偏移 51（D18 已定项 18，2026-09-14 用户定案）：解析器不用先知道这是哪棵树就找得到头末端。
    /// 这是 G2 收口那一格的会红检查——把 key 宽写进别的偏移，解析当场失败。
    #[test]
    fn the_key_width_field_sits_at_offset_51_so_the_header_locates_itself() {
        let BuiltPool { output, .. } = built_pool();
        for (index, expected_key_width) in [(1usize, 24usize), (3, 8), (4, ALLOCATION_KEY_BYTES), (5, 22), (6, MAPPING_KEY_BYTES as usize), (7, TREE_TABLE_KEY_WIDTH)] {
            let unit = &output.units_by_slot[index].2;
            assert_eq!(usize::from(unit[INDEX_NODE_KEY_WIDTH_OFFSET]), expected_key_width, "偏移 51 那一字节就是 key 宽");
            let node = parse_index_node(unit).expect("自描述的 key 宽足以定出头末端");
            assert_eq!(node.key_width, expected_key_width);
        }
        assert_eq!(INDEX_NODE_KEY_WIDTH_OFFSET, 51);
        // 把 key 宽改成别的值：头末端跟着移，头校验和当场不过。
        let mut tampered = output.units_by_slot[6].2.clone();
        tampered[INDEX_NODE_KEY_WIDTH_OFFSET] = 25;
        assert_eq!(parse_index_node(&tampered).unwrap_err(), UnitError::HeaderChecksum);
        // 自述一个大到让头越出这段字节的 key 宽：判结构错，不许下标越界。
        let mut absurd = output.units_by_slot[6].2.clone();
        absurd[INDEX_NODE_KEY_WIDTH_OFFSET] = 255;
        absurd.truncate(200);
        assert_eq!(parse_index_node(&absurd).unwrap_err(), UnitError::Structure("自述的 key 宽让头越出单元"));
    }

    #[test]
    fn transaction_issues_21_writes_2_barriers_1_fua_in_the_settled_order() {
        let BuiltPool { recording, output, warm_up_operation_count, .. } = built_pool();
        let operations = &recording.operations[warm_up_operation_count..];
        let steps: Vec<&'static str> = operations.iter().map(|operation| RecordedStepKind::of(operation).tag()).collect();
        assert_eq!(steps.iter().filter(|tag| **tag != "barrier").count(), 21);
        assert_eq!(steps.iter().filter(|tag| **tag == "barrier").count(), 2);
        assert_eq!(steps.iter().filter(|tag| **tag == "root_record_fua").count(), 1);
        assert_eq!(operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(write) if write.is_fua())).count(), 1, "FUA 只由步骤种类决定，只有根槽那一步是");
        assert_eq!(steps[16], "barrier");
        assert_eq!(steps[19], "barrier");
        assert_eq!(steps[20], "root_record_fua");
        assert_eq!(output.record.named.len(), 8);
        assert_eq!(output.record.named.iter().map(|named| named.locations.len()).sum::<usize>(), 16);
        // D3 已定项 10 ⑤（2026-09-14 用户定案）：t1 用户数据取不在开放聚簇段里的最低 32768 对齐空槽对；
        // t2–t8 从开放段 [50240, 50304) 按「树 ID 升序、树内先叶后根、映射树倒数第二、树表最末」bump，
        // 而 t3 是码 3 容器、要起点 32768 对齐 ⇒ 跳过 50241。
        let slots: Vec<u64> = output.units_by_slot.iter().map(|(slot, _, _)| slot.0).collect();
        assert_eq!(slots, vec![50180, 50240, 50242, 50244, 50245, 50246, 50247, 50248]);
        assert_eq!(SLOT_INODE_LEAF % (DATA_UNIT_BYTES / SLOT_BYTES), 0, "码 3 容器起点也 32768 对齐");
        assert!(!slots.contains(&SLOT_SKIPPED_BY_ALIGNMENT), "槽 50241 没人占");
        assert_eq!(SLOT_DATA_UNIT % (DATA_UNIT_BYTES / SLOT_BYTES), 0, "数据单元起点 32768 对齐");
        assert!(SLOT_DATA_UNIT < OPEN_CLUSTER_SEGMENT_START_SLOT, "用户数据不进开放聚簇段");
        assert_eq!(OPEN_CLUSTER_SEGMENT_START_SLOT % CLUSTER_SEGMENT_SLOTS, 0, "开放段 64 槽对齐");
        for slot in [SLOT_EXTENT_ROOT, SLOT_INODE_LEAF, SLOT_INODE_ROOT, SLOT_ALLOCATION_ROOT, SLOT_ACCOUNTING_ROOT, SLOT_MAPPING_ROOT, SLOT_TREE_TABLE_FIRST_PUBLISH] {
            assert!((OPEN_CLUSTER_SEGMENT_START_SLOT..OPEN_CLUSTER_SEGMENT_START_SLOT + CLUSTER_SEGMENT_SLOTS).contains(&slot), "提交内生块住开放聚簇段");
        }
    }

    #[test]
    fn mkfs_seeds_three_generation_zero_roots_readable_from_all_regions() {
        let parameters = PoolParameters::settled_two_devices();
        let (recording, genesis) = mkfs(&parameters);
        let superblock = choose_superblock(&recording.pool).expect("超级块");
        let mut readable = 0;
        for region in 0..RING_REGIONS {
            let bytes = recording.pool.read(superblock.region_devices[region as usize], ring_slot_offset(region, 0), 512);
            if RootRecord::parse_slot(&bytes, &FIXED_FSID) == Some(genesis.root) {
                readable += 1;
            }
        }
        assert_eq!(readable, 3);
        assert_eq!(ring_region_offset(2).0, 7 << 20);
        // D23 已定项 16：mkfs 写实例代号 0，单元写序 (0, 0)；树 ID 水位就是第一个要发的树 ID。
        assert_eq!(genesis.root.instance, InstanceGeneration(0));
        assert_eq!(genesis.root.tree_identifier_watermark, 11);
        assert_eq!(TREE_IDENTIFIER_WATERMARK_AFTER_PUBLISH, 19, "deadlist 取树 ID 18 ⇒ 发布后水位 19");
        assert_eq!(parse_packed_unit(&genesis.instance_table_unit).expect("实例表").write_order, WriteOrder { instance: InstanceGeneration(0), transaction: TransactionNumber(0) });
        // D22 已定项 16：mkfs 把每盘两个槽都种上世代号 1。
        for device_index in 0..2u32 {
            for slot_offset in SUPERBLOCK_SLOT_OFFSETS {
                let slot = recording.pool.read(DeviceIdentity(device_index), DeviceOffset(slot_offset), SUPERBLOCK_SLOT_BYTES as usize);
                let superblock = Superblock::parse_slot(&slot).expect("mkfs 的超级块槽");
                assert_eq!(superblock.slot_generation, 1);
                assert_eq!(superblock.journal_instance, InstanceGeneration(0));
            }
        }
    }

    /// 超级块 481（D22 已定项 9 + 已定项 15，2026-09-14 用户定案加四个字段）：每个字段按**绝对偏移**读一遍，
    /// 以及「整槽校验和罩整个 4096 槽」——改 481 之后那 3615 字节补齐里的任何一个字节，解析都要拒绝。
    #[test]
    fn superblock_is_481_bytes_and_the_slot_checksum_covers_all_4096() {
        let BuiltPool { recording, .. } = built_pool();
        let slot = recording.pool.read(DeviceIdentity(0), DeviceOffset(SUPERBLOCK_SLOT_OFFSETS[1]), SUPERBLOCK_SLOT_BYTES as usize);
        assert_eq!(slot.len(), 4096, "超级块槽宽是格式常量 4096（D22 已定项 2，2026-09-14 三方论证后）");
        assert!(Superblock::parse_slot(&slot).is_some());
        let read_u64 = |offset: usize| u64::from_le_bytes(slot[offset..offset + 8].try_into().expect("切了 8 字节"));
        let read_u32 = |offset: usize| u32::from_le_bytes(slot[offset..offset + 4].try_into().expect("切了 4 字节"));
        // 自举头 2026-09-14 加的两个字段：写入者身份 20（偏移 118）与校验和算法标识 1（偏移 138）。
        assert_eq!(&slot[118..118 + WRITER_IDENTITY_NAME.len()], WRITER_IDENTITY_NAME, "实现标识 ASCII");
        assert!(slot[118 + WRITER_IDENTITY_NAME.len()..134].iter().all(|byte| *byte == 0), "实现标识零补齐到 16");
        assert_eq!(read_u32(134), 1, "写入者版本（I-1.5、D17 债 3）");
        assert_eq!(slot[138], 1, "校验和算法标识 = 1 CRC-32C（I-2.2、C309）");
        // 几何段 2026-09-14 加的两个字段：mkfs 时的 physical_block_size（偏移 321）与扩展点声明值 N（325）。
        assert_eq!(read_u32(317), 512, "mkfs 时的 physical_block_size（D13 已定项 5 的比对输入）");
        assert_eq!(read_u32(321), 0, "扩展点声明值 N（D21 已定项 8，第一版 0）");
        assert_eq!(read_u64(333), 805_306_368, "journal 环长 768 MiB（D23 已定项 19 ③）");
        assert_eq!(read_u32(345), 65_536, "在飞记录数上限（D23 已定项 18）");
        assert_eq!(read_u64(349), 268_435_456, "journal 最坏占用 = 在飞上限 × 4096（I-8.1）");
        assert_eq!(&slot[379..391], &[0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0], "根环逐区域设备身份写死 0 / 1 / 0（D2 已定项 7）");
        assert_eq!(slot[391], 4, "w_max（D2 已定项 18）");
        assert_eq!(slot[392], 4, "组大小 g（D2 已定项 18）");
        assert_eq!(read_u64(417), 50_176, "单元区起始槽号（D3 已定项 10 ④）");
        assert_eq!(read_u32(425), 512, "mkfs 时的 io_min（D2 已定项 19）");
        assert_eq!(read_u32(429), 4096, "固定结构槽距（D2 已定项 19）");
        assert!(slot[445..469].iter().all(|byte| *byte == 0), "整理三条水位 24 字节恒 0（D22 已定项 15）");
        assert!(slot[SUPERBLOCK_BYTES as usize..].iter().all(|byte| *byte == 0), "481 之后的 3615 字节补齐恒 0");
        let mut padded = slot.clone();
        padded[SUPERBLOCK_BYTES as usize] ^= 0x01;
        assert!(Superblock::parse_slot(&padded).is_none(), "整槽校验和罩到补齐区（D18 已定项 17）");
        // 算法标识写成未登记的码：这个槽不认（I-2.2）。
        let mut wrong_algorithm = slot.clone();
        wrong_algorithm[138] = 2;
        let digest = wide_checksum_with_field_zeroed(&wrong_algorithm, SUPERBLOCK_SLOT_BYTES as usize, SUPERBLOCK_CHECKSUM_OFFSET);
        wrong_algorithm[SUPERBLOCK_CHECKSUM_OFFSET..SUPERBLOCK_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        assert!(Superblock::parse_slot(&wrong_algorithm).is_none(), "校验和算法标识不是 1 就不认这个槽");
    }

    #[test]
    fn unknown_incompat_bit_or_missing_ssd_line_bit_refuses_to_mount_but_compat_bits_do_not() {
        let parameters = PoolParameters::settled_two_devices();
        let (recording, _) = mkfs(&parameters);
        let slot = recording.pool.read(DeviceIdentity(0), DeviceOffset(SUPERBLOCK_SLOT_OFFSETS[0]), SUPERBLOCK_SLOT_BYTES as usize);
        assert_eq!(slot[SUPERBLOCK_FEATURE_BITS_OFFSET], INCOMPAT_FIRST_SSD_LINE_BIT, "mkfs 起 incompat 位 0 置 1（D15 已定项 4）");
        assert!(slot[SUPERBLOCK_FEATURE_BITS_OFFSET + 1..SUPERBLOCK_FEATURE_BITS_OFFSET + 3 * FEATURE_BITMAP_BYTES].iter().all(|byte| *byte == 0), "其余 95 字节全 0");
        assert!(Superblock::parse_slot(&slot).is_some(), "原样可挂");
        let reseal = |mut bytes: Vec<u8>, offset: usize, value: u8| {
            bytes[offset] = value;
            let digest = wide_checksum_with_field_zeroed(&bytes, SUPERBLOCK_SLOT_BYTES as usize, SUPERBLOCK_CHECKSUM_OFFSET);
            bytes[SUPERBLOCK_CHECKSUM_OFFSET..SUPERBLOCK_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
            bytes
        };
        assert!(Superblock::parse_slot(&reseal(slot.clone(), SUPERBLOCK_FEATURE_BITS_OFFSET, 0x03)).is_none(), "incompat 位 1 没登记：不认识不许挂");
        assert_eq!(SUPERBLOCK_FEATURE_BITS_OFFSET, 6);
        assert!(Superblock::parse_slot(&reseal(slot.clone(), SUPERBLOCK_FEATURE_BITS_OFFSET + FEATURE_BITMAP_BYTES - 1, 0x80)).is_none(), "incompat 位 255 没登记：不认识不许挂");
        assert!(Superblock::parse_slot(&reseal(slot.clone(), SUPERBLOCK_FEATURE_BITS_OFFSET, 0x00)).is_none(), "没有布局身份：拒绝");
        assert!(Superblock::parse_slot(&reseal(slot.clone(), SUPERBLOCK_FEATURE_BITS_OFFSET + FEATURE_BITMAP_BYTES, 0x01)).is_some(), "compat_ro 位不认识只读挂，读者照样解析");
        assert!(Superblock::parse_slot(&reseal(slot, SUPERBLOCK_FEATURE_BITS_OFFSET + 2 * FEATURE_BITMAP_BYTES, 0x01)).is_some(), "compat 位不认识随便");
    }

    #[test]
    fn cold_start_reads_the_file_back_and_chooses_instance_one_txg_three() {
        let BuiltPool { recording, .. } = built_pool();
        let report = recover(&recording.pool, JournalPolicy::Consult);
        assert_eq!(report.outcome, RecoveryOutcome::FileRead { root: (InstanceGeneration(1), CheckpointTxg(3)), content: sample_file() });
        assert_eq!(report.journal.valid_records, 3, "两条暖机空记录 + 事务记录");
        // 记录头 2026-09-14 带 fsid：把三条记录的 fsid 段全改成别的池，扫描一条都不认（I-1.4）。
        let mut alien = recording.pool.clone();
        for counter in 1..=3u64 {
            let offset = journal_record_offset(JournalCounter(counter));
            for device_index in 0..2u32 {
                let device = DeviceIdentity(device_index);
                let mut record = alien.read(device, offset, JOURNAL_RECORD_BYTES as usize);
                record[283..291].copy_from_slice(&0xDEAD_BEEF_u64.to_le_bytes());
                let digest = wide_checksum_with_field_zeroed(&record, JOURNAL_RECORD_BYTES as usize, JOURNAL_HEADER_CHECKSUM_OFFSET);
                record[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
                alien.devices[device.0 as usize].write(offset, &record);
            }
        }
        assert_eq!(scan_journal(&alien, unit_fsid(&FIXED_FSID)).len(), 0, "fsid 不符的记录不算数");
        assert_eq!(scan_journal(&alien, 0xDEAD_BEEF).len(), 3, "换成它们自己的 fsid 就全认——证明上一行不是因为校验和坏了");
        assert_eq!(report.journal.above_water, 0, "记录 txg 1..3 都不高于根的水位 3，不施加");
        assert_eq!(report.mapping_fallbacks, 0);
    }

    #[test]
    /// 取号 [2 个超级块槽]（3 个状态）、暖机第一次 [2 条空记录][根 FUA][2 个超级块槽]（7 个）、第二次 [2][1]（4 个）；
    /// 第二次的超级块槽写与事务的 16 个单元写之间没有屏障、同一段 18 个（2¹⁸ − 1）；
    /// 再 [2 条记录][根 FUA][2 个超级块槽]（7 个）⇒ 1 + 3 + 7 + 4 + 262143 + 7 = 262165。
    fn layer0_state_count_is_262165_with_zero_violations() {
        let BuiltPool { recording, mkfs_operation_count, .. } = built_pool();
        let (base, _) = mkfs(&PoolParameters::settled_two_devices());
        let (writes, segments) = split_into_segments(&recording.operations[mkfs_operation_count..], true);
        assert_eq!(segments.iter().map(Vec::len).collect::<Vec<_>>(), vec![2, 2, 1, 2, 2, 1, 18, 2, 1, 2]);
        assert_eq!(closed_form_state_count(&segments), 262165);
        let tally = enumerate_layer0(&base.pool, &writes, &segments, &sample_file());
        assert_eq!(tally.states, 262165);
        assert_eq!(tally.violations, 0, "{:?}", tally.first_violation);
        assert_eq!(tally.root_persisted_states, 4, "事务根槽持久的状态照旧 4 个：根槽那一段与之后超级块段的子集");
        // 施加记录会重建那次发布的根（D23 已定项 15）⇒ 事务记录两份都持久、8 个单元都验得过的那 3 个状态也读得到文件。
        assert_eq!(tally.file_read_states, 7);
        assert_eq!(tally.no_file_states, 262158);
        assert_eq!(tally.journal_differing_states, 3, "journal 从此承重：这 3 个状态查不查 journal 结果不同");
        assert_eq!(tally.verification_ran_states, 9, "三条记录各自「持久而所属的根还没持久」的 3 个子集：暖机 jsn 1、jsn 2 与事务 jsn 3");
        assert_eq!(tally.verification_failed_states, 0);
    }

    /// FUA 不当边界（C313 已判掉的另一读法，只钉它给的数不同）：[2][2][3][2][19][2][3] ⇒ 1 + 3 + 3 + 7 + 3 + 524287 + 3 + 7 = 524314。
    #[test]
    fn fua_not_a_boundary_gives_524314_states() {
        let BuiltPool { recording, mkfs_operation_count, .. } = built_pool();
        let (_, segments) = split_into_segments(&recording.operations[mkfs_operation_count..], false);
        assert_eq!(segments.iter().map(Vec::len).collect::<Vec<_>>(), vec![2, 2, 3, 2, 19, 2, 3]);
        assert_eq!(closed_form_state_count(&segments), 524314);
    }

    /// 段序列登记表（first-txn-layout 八）的五行由这条钉住：mkfs 4+1+1+1+4（13 次操作、34 个状态）、取号 2、
    /// 暖机 2+1+2+2+1+2、事务 16+2+1+2、整条流 2+2+1+2+2+1+18+2+1+2。
    /// 改任何一条路径里屏障或 FUA 的位置都红——mkfs 那一行不进层 0 枚举，这里是它唯一的会红检查。
    ///
    /// D17（实现分层与第三方管道） 已定项 2 的结构等价类要的是「段边界位置 + 每段步骤种类集合」，
    /// 所以这里连每段的步骤种类多重集一起钉死，四条路径各钉一个**绝对值**（不是拿几条路径互相比）：
    /// 把某一步录成别的种类——例如超级块槽写录成单元写——段边界与状态数一个都不变，只有这几行会红（C316 ②）。
    #[test]
    fn registered_segment_sequences_match_every_recorded_path() {
        let BuiltPool { recording, mkfs_operation_count, acquisition_operation_count, warm_up_operation_count, .. } = built_pool();
        let sizes = |operations: &[RecordedOperation]| split_into_segments(operations, true).1.iter().map(Vec::len).collect::<Vec<_>>();
        let kinds = |operations: &[RecordedOperation]| format_segment_kinds(&segment_step_kinds(operations, true));
        let mkfs_operations = &recording.operations[..mkfs_operation_count];
        let acquisition_operations = &recording.operations[mkfs_operation_count..acquisition_operation_count];
        let warm_up_operations = &recording.operations[acquisition_operation_count..warm_up_operation_count];
        let transaction_operations = &recording.operations[warm_up_operation_count..];
        let post_mkfs_operations = &recording.operations[mkfs_operation_count..];
        assert_eq!(mkfs_operations.len(), 13, "mkfs：11 次写（m1/m2 各两盘、三个第 0 代根、两盘各两个超级块槽）+ 2 道屏障");
        assert_eq!(sizes(mkfs_operations), vec![4, 1, 1, 1, 4]);
        assert_eq!(closed_form_state_count(&split_into_segments(mkfs_operations, true).1), 34);
        assert_eq!(acquisition_operations.len(), 2, "取号：两盘各写一次超级块槽，不另加屏障");
        assert_eq!(sizes(acquisition_operations), vec![2]);
        assert_eq!(sizes(warm_up_operations), vec![2, 1, 2, 2, 1, 2]);
        assert_eq!(sizes(transaction_operations), vec![16, 2, 1, 2]);
        assert_eq!(sizes(post_mkfs_operations), vec![2, 2, 1, 2, 2, 1, 18, 2, 1, 2]);

        assert_eq!(
            kinds(mkfs_operations),
            "[unit_write×4,barrier]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[superblock_slot×4,barrier]",
            "mkfs：m1/m2 两个单元各两盘一段、三个第 0 代根各自 FUA 一段、两盘各两个超级块槽收尾"
        );
        assert_eq!(kinds(acquisition_operations), "[superblock_slot×2]", "取号那一段只有两次超级块槽写");
        assert_eq!(
            kinds(warm_up_operations),
            "[journal_record×2,barrier×2]|[root_record_fua]|[superblock_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2]",
            "暖机两次空发布：每次「空记录两盘 → 根 FUA → 超级块槽两盘」，第一段前面还有那道开场屏障"
        );
        assert_eq!(
            kinds(transaction_operations),
            "[unit_write×16,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2]",
            "第一个事务：8 个单元各两盘 → journal 记录两盘 → 根 FUA → 超级块槽两盘"
        );
        assert_eq!(
            kinds(post_mkfs_operations),
            "[superblock_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[unit_write×16,superblock_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2]",
            "整条流：取号那两次超级块槽写自成一段（收段的是暖机第一次开头那道屏障），暖机第二次的两个超级块槽与事务的 16 个单元写同一段 18 个写"
        );

        // 每一步都恰好落在一个段里：各段的步骤数加起来等于录到的操作数。
        for operations in [mkfs_operations, acquisition_operations, warm_up_operations, transaction_operations, post_mkfs_operations] {
            assert_eq!(segment_step_kinds(operations, true).iter().map(Vec::len).sum::<usize>(), operations.len());
        }
        // 种类的字母表就这五个，别处不许冒出第六个。
        let alphabet: std::collections::BTreeSet<&str> = segment_step_kinds(post_mkfs_operations, true).concat().iter().map(|kind| kind.tag()).collect();
        assert_eq!(alphabet.into_iter().collect::<Vec<_>>(), vec!["barrier", "journal_record", "root_record_fua", "superblock_slot", "unit_write"]);
    }

    /// 暖机（D16 已定项 8）：两次空发布，根落区域 1 与区域 2（分住两块盘），jsn 1、2 不点名任何单元，第一个事务从 txg 3 起。
    #[test]
    fn warm_up_writes_two_empty_publishes_covering_both_devices() {
        let BuiltPool { recording, output, acquisition_operation_count, warm_up_operation_count, .. } = built_pool();
        let operations = &recording.operations[acquisition_operation_count..warm_up_operation_count];
        let writes = operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(_))).count();
        let fua_writes: Vec<&WriteRequest> = operations.iter().filter_map(|operation| match operation { RecordedOperation::Write(write) if write.is_fua() => Some(write), RecordedOperation::Write(_) | RecordedOperation::Barrier => None }).collect();
        assert_eq!(writes, 10, "每次空发布 2 条记录 + 1 个根 + 2 个超级块槽");
        assert_eq!(operations.iter().filter(|operation| matches!(operation, RecordedOperation::Barrier)).count(), 4);
        assert_eq!(fua_writes.len(), 2);
        let parameters = PoolParameters::settled_two_devices();
        assert_eq!(RING_REGION_DEVICES, [0, 1, 0], "根环区域归属第一版写死（D2 已定项 7，2026-09-14 用户定案）");
        assert_ne!(fua_writes[0].device, fua_writes[1].device, "两个暖机根要落在两块盘上");
        assert_eq!((parameters.region_devices[1], parameters.region_devices[2]), (fua_writes[0].device, fua_writes[1].device));
        assert_eq!(output.root.checkpoint_txg, CheckpointTxg(3));
        assert_eq!(output.record.counter, JournalCounter(3));
        assert_eq!(FIRST_TRANSACTION_TXG, WARM_UP_EMPTY_PUBLISHES + 1, "第一个事务紧跟暖机之后");
        assert_eq!(output.record.instance, InstanceGeneration(1), "第一次可写挂载取的实例代号是 1（D23 已定项 16）");
        // D22 已定项 16：世代号 mkfs 1（两槽同写）、取号 2（槽 0）、w3 3（槽 1）、w6 4（槽 0）、t11 5（槽 1）。
        assert_eq!(superblock_write_for_publish(CheckpointTxg(1)), (3, 1));
        assert_eq!(superblock_write_for_publish(CheckpointTxg(2)), (4, 0));
        assert_eq!(superblock_write_for_publish(CheckpointTxg(3)), (5, 1));
        // D23 已定项 19 ②：环上计数器为 1 的那条反向链恒 0，之后每条罩前一条的整个头。
        let records = scan_journal(&recording.pool, unit_fsid(&FIXED_FSID));
        let chains: Vec<u32> = records.values().map(|record| record.back_chain).collect();
        assert_eq!(chains[0], 0, "jsn 1 的反向链恒 0");
        assert_ne!(chains[1], 0);
        assert_ne!(chains[2], 0);
        assert_eq!(chains[1], journal_back_chain(&records[&(InstanceGeneration(1), JournalCounter(1))].to_bytes()));
        assert_eq!(chains[2], journal_back_chain(&records[&(InstanceGeneration(1), JournalCounter(2))].to_bytes()));
        assert_eq!(records.values().filter(|record| record.transaction == TransactionNumber(0) && record.is_commit).count(), 2, "空发布事务号 0、提交标记 1（D23 已定项 19 ①）");
    }

    #[test]
    fn positive_control_without_barriers_has_1020_violations_out_of_2048() {
        let control = PoolParameters::control_one_device_no_barriers();
        let (mut recording, genesis) = mkfs(&control);
        let mkfs_operation_count = recording.operations.len();
        let _ = publish_first_file(&mut recording, &control, &genesis, &sample_file(), InstanceGeneration(FIRST_INSTANCE_GENERATION), None);
        let (base, _) = mkfs(&control);
        let (writes, segments) = split_into_segments(&recording.operations[mkfs_operation_count..], false);
        assert_eq!(writes.len(), 11);
        let tally = enumerate_layer0(&base.pool, &writes, &segments, &sample_file());
        assert_eq!(tally.states, 2048);
        assert_eq!(tally.violations, 1020);
        assert_eq!(tally.root_persisted_states, 1024);
    }

    #[test]
    fn flipping_the_last_header_byte_is_caught_by_the_header_checksum() {
        let BuiltPool { output, .. } = built_pool();
        let mut unit = output.units_by_slot[0].2.clone();
        unit[DATA_UNIT_HEADER_BYTES as usize - 1] ^= 0x01;
        assert_eq!(parse_data_unit(&unit).unwrap_err(), UnitError::HeaderChecksum);
        let mut payload_flipped = output.units_by_slot[0].2.clone();
        payload_flipped[DATA_UNIT_HEADER_BYTES as usize + NONCE_MAC_RESERVED_BYTES as usize + 10] ^= 0x01;
        assert_eq!(parse_data_unit(&payload_flipped).unwrap_err(), UnitError::PayloadChecksum);
        let mut reserved_flipped = output.units_by_slot[0].2.clone();
        reserved_flipped[DATA_UNIT_HEADER_BYTES as usize + 3] ^= 0x01;
        assert_eq!(parse_data_unit(&reserved_flipped).unwrap_err(), UnitError::PayloadChecksum, "预留位在载荷 CRC 覆盖内");
    }

    /// D23 已定项 13（2026-09-14 用户定案改写）：`header_csum` 覆盖**整条记录 [0, 4096) 含补齐**。
    /// 三处各翻一个字节，三处都要被抓——尤其最后那处：点名项之后的补齐区，只罩头的那个读法抓不到它。
    #[test]
    fn the_journal_header_checksum_covers_the_whole_record_including_the_padding() {
        let BuiltPool { output, .. } = built_pool();
        assert!(JournalRecord::parse(&output.record_bytes).is_some());
        let payload_checksum_offset = JOURNAL_HEADER_CHECKSUM_OFFSET + WIDE_CHECKSUM_BYTES as usize + 8 + 1 + 4;
        assert_eq!(payload_checksum_offset, 91, "载荷校验和字段的偏移");
        for (offset, what) in [
            (payload_checksum_offset, "载荷校验和字段"),
            (JOURNAL_HEADER_BYTES as usize + 3, "点名项数组"),
            (JOURNAL_RECORD_BYTES as usize - 1, "点名项之后的补齐区最后一字节"),
        ] {
            let mut record = output.record_bytes.clone();
            record[offset] ^= 0x01;
            assert!(JournalRecord::parse(&record).is_none(), "{what}落在 header_csum 覆盖内");
        }
        // 反向：只罩头 [0, 307) 的那个读法对补齐区那一字节说不出话——这一行证明上面第三条不是白抓的。
        let mut padded = output.record_bytes.clone();
        padded[JOURNAL_RECORD_BYTES as usize - 1] ^= 0x01;
        let head_only_before = wide_checksum_with_field_zeroed(&output.record_bytes, JOURNAL_HEADER_BYTES as usize, JOURNAL_HEADER_CHECKSUM_OFFSET);
        let head_only_after = wide_checksum_with_field_zeroed(&padded, JOURNAL_HEADER_BYTES as usize, JOURNAL_HEADER_CHECKSUM_OFFSET);
        assert_eq!(head_only_before, head_only_after, "只罩头的那个读法分不出这一改，罩整条的才分得出");
    }

    #[test]
    fn probes_behave_as_milestone_step_six_expects() {
        let BuiltPool { recording, .. } = built_pool();
        let parameters = PoolParameters::settled_two_devices();
        let mut by_name = BTreeMap::new();
        for probe in probes(&parameters) {
            by_name.insert(probe.name, run_probe(&recording.pool, &probe));
        }
        // 最新根槽坏了 ⇒ 择回暖机第 2 代根，再施加 jsn 3 那条记录重建那次发布的根（D23 已定项 15）⇒ 照样读得到文件。
        assert!(matches!(by_name["newest_root_slot_one_byte"].outcome, RecoveryOutcome::FileRead { root: (InstanceGeneration(1), CheckpointTxg(2)), .. }));
        assert_eq!(by_name["newest_root_slot_one_byte"].journal.prefix_applied, 1);
        assert!(matches!(by_name["journal_record_both_copies"].outcome, RecoveryOutcome::FileRead { .. }));
        assert_eq!(by_name["journal_record_both_copies"].journal.valid_records, 2, "事务记录两份都坏，暖机的两条空记录还在");
        assert_eq!(by_name["journal_record_one_copy"].journal.valid_records, 3);
        assert!(matches!(by_name["data_payload_one_copy"].outcome, RecoveryOutcome::FileRead { .. }));
        assert!(matches!(by_name["data_payload_both_copies"].outcome, RecoveryOutcome::Failed { .. }));
        assert_eq!(by_name["data_payload_both_copies"].mapping_fallbacks, 1);
        assert!(matches!(by_name["data_header_last_byte_both_copies"].outcome, RecoveryOutcome::Failed { .. }));
        assert!(matches!(by_name["tree_table_both_copies"].outcome, RecoveryOutcome::Failed { .. }));
        assert!(matches!(by_name["superblock_slot_one_both_devices"].outcome, RecoveryOutcome::FileRead { .. }));
    }

    /// 根记录 371（D22 已定项 7 的字段表，2026-09-14 用户定案在中央映射树根指针之后加算法类型 1 + nonce 12 + MAC 16）：
    /// 自证校验和罩整个 512 槽（D18 已定项 17），末尾 29 字节第一版全 0。
    #[test]
    fn root_record_is_371_bytes_and_carries_the_mapping_tree_root() {
        let BuiltPool { output, .. } = built_pool();
        let slot = output.root.to_slot();
        assert_eq!(slot.len(), PHYSICAL_BLOCK_BYTES as usize);
        assert_eq!(RootRecord::parse_slot(&slot, &FIXED_FSID), Some(output.root));
        assert!(slot[ROOT_RECORD_BYTES as usize..].iter().all(|byte| *byte == 0), "371 之后的 141 字节补齐恒 0");
        let mut padded = slot.clone();
        padded[ROOT_RECORD_BYTES as usize] ^= 0x01;
        assert!(RootRecord::parse_slot(&padded, &FIXED_FSID).is_none(), "自证校验和罩到补齐区");
        // 算法类型 / nonce / MAC 这 29 字节是第一版的留位，全 0；映射树根指针紧挨在它们之前。
        assert!(slot[342..371].iter().all(|byte| *byte == 0), "算法类型 1 + nonce 12 + MAC 16 第一版全 0");
        let mut mapping_pointer_bytes = ByteReader::at(&slot, (ROOT_RECORD_BYTES - NODE_POINTER_BYTES - 1 - 12 - 16) as usize);
        assert_eq!(NodePointer::read_from(&mut mapping_pointer_bytes), output.root.mapping_root);
        assert_eq!(output.root.mapping_root.locations[0].slot, SlotNumber(SLOT_MAPPING_ROOT));
    }

    /// journal 记录头 307（D23 已定项 15 的新根段 188 + 2026-09-14 加的 fsid 8 与 MAC 16）；
    /// 点名项 56 的 key 尾段凑得出中央映射的 6 条 key（已定项 17 末句），一律 27 字节。
    #[test]
    fn journal_record_carries_the_new_root_segment_and_the_named_entries_rebuild_the_mapping_keys() {
        let BuiltPool { output, .. } = built_pool();
        let parsed = JournalRecord::parse(&output.record_bytes).expect("记录自检要过");
        assert_eq!(parsed, output.record);
        assert_eq!(parsed.new_tree_table.locations[0].slot, SlotNumber(SLOT_TREE_TABLE_FIRST_PUBLISH));
        assert_eq!(parsed.new_mapping_root.locations[0].slot, SlotNumber(SLOT_MAPPING_ROOT));
        assert_eq!(parsed.new_tree_identifier_watermark, 19, "deadlist day-1 注册 ⇒ 水位 19");
        assert_eq!(parsed.new_rollback_floor, CheckpointTxg(0));
        assert_eq!(parsed.transaction, TransactionNumber(1));
        assert_eq!(parsed.fsid, unit_fsid(&FIXED_FSID), "记录头的 fsid 与单元头同口径（D23 已定项 4）");
        assert!(output.record_bytes[291..307].iter().all(|byte| *byte == 0), "记录头末尾 MAC 16 第一版全 0");
        let rebuilt: Vec<Vec<u8>> = parsed.named.iter().map(NamedUnit::mapping_key).collect();
        assert_eq!(rebuilt.len(), 8);
        assert_eq!(rebuilt.iter().filter(|key| key.len() == 27).count(), 8, "一律 27（D19 已定项 10，2026-09-14 用户定案）");
        for key in &output.mapping_keys {
            assert!(rebuilt.contains(key), "映射树里的 key 都凑得出来：{key:?}");
        }
    }

    /// 中央映射树：6 条条目**一宽 55**、key 一律 27（D19 已定项 10，2026-09-14 用户定案回定宽）；
    /// 码 2 / 码 3 的 25 字节 key 末尾补两个零，补零进 key 本身而不只是区间字段。
    #[test]
    fn mapping_tree_holds_one_entry_width_and_pads_short_keys_into_the_key_itself() {
        let BuiltPool { output, .. } = built_pool();
        let node = parse_index_node(&output.units_by_slot[6].2).expect("映射树根");
        assert_eq!(node.entries.len(), 6);
        assert!(node.entries.iter().all(|entry| entry.len() == 55), "条目一律 55");
        assert_eq!(node.key_width, 27, "头自述的 key 宽");
        assert_eq!(node.smallest_key.len(), 27);
        assert_eq!(node.largest_key.len(), 27);
        assert_eq!(&node.entries[0][..27], &node.smallest_key[..], "条目里 key 一律打头，区间就是首末两条的前 27 字节");
        assert_eq!(&node.entries[5][..27], &node.largest_key[..]);
        assert_eq!(node.entries[0][0], UNIT_CLASS_DATA, "最小那条是码 1（类标签 1 排在最前）");
        assert_eq!(&node.largest_key[25..], &[0, 0], "码 2 / 码 3 的 key 末尾补的那两个零");
        // 声明长度 = 条目数 × 条目宽，条目区从含预留位的头末尾（169）起。
        let declared_length = u16::from_le_bytes(output.units_by_slot[6].2[8..10].try_into().expect("切了 2 字节"));
        assert_eq!(declared_length, 330, "6 × 55");
        assert_eq!(index_node_header_bytes(27) + NONCE_MAC_RESERVED_BYTES as usize, 169);
    }

    /// 每个码 2 节点里的条目 key 严格递增（D8（核心索引结构） 已定项 11 的全序：逐字段无符号整数、自左向右）。
    /// 这是一条**跨结构**的绝对检查：任何一棵树把两条同 key 的条目写进同一个节点都会红，
    /// 不必等某一棵树自己那条断言想到这一格。
    #[test]
    fn every_index_node_keeps_its_entry_keys_strictly_ascending() {
        let BuiltPool { output, .. } = built_pool();
        let mut checked_nodes = 0;
        for (slot, unit_identity, unit) in &output.units_by_slot {
            let (unit_class, key_width) = unit_identity.class_and_key_width();
            if unit_class != UNIT_CLASS_INDEX_NODE {
                continue;
            }
            let node = parse_index_node(unit).expect("码 2 节点");
            assert_eq!(node.key_width, key_width);
            for pair in node.entries.windows(2) {
                assert!(pair[0][..key_width] < pair[1][..key_width], "{} @ 槽 {} 的条目 key 没有严格递增", unit_identity.tag(), slot.0);
            }
            checked_nodes += 1;
        }
        assert_eq!(checked_nodes, 6, "事务里六个码 2 节点：extent、inode 根、分配、记账、映射、树表");
    }

    /// 条目宽字段写坏了，解析器要当场拒绝（D8（核心索引结构） 已定项 11：声明长度 = 条目数 × 条目宽）。
    /// 第七次跑的变异 M62 逼出来的：此前那条结构检查一条测试都没碰过，整段删掉也没人红。
    #[test]
    fn the_parser_refuses_an_entry_width_field_that_contradicts_the_declared_length() {
        let BuiltPool { output, .. } = built_pool();
        let mapping_unit = &output.units_by_slot[6].2;
        let entry_width_offset = index_node_header_bytes(MAPPING_KEY_BYTES as usize) - 2;
        assert_eq!(u16::from_le_bytes(mapping_unit[entry_width_offset..entry_width_offset + 2].try_into().expect("切了 2 字节")), 55);
        // 条目宽与载荷 CRC 都住头里（偏移 76 + 2k 与 84 + 2k），改条目宽只让头校验和失配 ⇒ 重新封一次头校验和再解。
        for (bad_width, what) in [(0u16, "条目宽 0（第六次跑那个变长哨兵）"), (54, "比声明长度算出来的小一"), (27, "刚好等于 key 宽")] {
            let mut tampered = mapping_unit.clone();
            tampered[entry_width_offset..entry_width_offset + 2].copy_from_slice(&bad_width.to_le_bytes());
            seal_header_checksum(&mut tampered, index_node_header_bytes(MAPPING_KEY_BYTES as usize));
            assert_eq!(parse_index_node(&tampered).unwrap_err(), UnitError::Structure("声明长度与条目数 × 条目宽对不上"), "{what}");
        }
        // 反向：原样的 55 解得开——上面三条不是因为「改了字节就一定红」。
        assert!(parse_index_node(mapping_unit).is_ok());
    }

    #[test]
    fn two_data_units_in_one_transaction_share_a_mapping_key() {
        let BuiltPool { output, .. } = built_pool();
        let first = mapping_key_for_data(output.data_pointer.head, output.data_pointer.write_order);
        let second = mapping_key_for_data(PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT), birth_txg: CheckpointTxg(FIRST_TRANSACTION_TXG) }, WriteOrder { instance: InstanceGeneration(1), transaction: TransactionNumber(1) });
        assert_eq!(first, second);
        assert_eq!(first.len(), 27);
        let distinct: std::collections::BTreeSet<Vec<u8>> = output.mapping_keys.iter().cloned().collect();
        assert_eq!(distinct.len(), 6);
    }

    #[test]
    fn allocation_and_accounting_trees_carry_the_byte_table_numbers() {
        let BuiltPool { output, .. } = built_pool();
        assert_eq!(output.allocation_records.len(), 20, "每盘 10 条：m1 m2 与 t1–t8（deadlist 无节点、不占落点）");
        assert_eq!(output.allocation_records.iter().filter(|record| record.generation == CheckpointTxg(0)).count(), 4, "mkfs 的 m1 / m2 分配代 0");
        assert_eq!(output.allocation_records.iter().filter(|record| record.generation == CheckpointTxg(3)).count(), 16);
        assert_eq!(output.allocation_records[0].key_bytes().len(), 10);
        let slots_per_device: Vec<u64> = output.allocation_records.iter().filter(|record| record.device == DeviceIdentity(0)).map(|record| record.slot.0).collect();
        assert_eq!(slots_per_device, vec![50176, 50178, 50180, 50240, 50242, 50244, 50245, 50246, 50247, 50248]);
        // D5 已定项 8（2026-09-14 用户定案）：两盘时 **15 行**，seq 一律 1、代一律 3。
        assert_eq!(output.accounting_entries.len(), 15);
        assert!(output.accounting_entries.iter().all(|entry| entry.sequence == 1 && entry.generation == CheckpointTxg(3)));
        let value_of = |statistic: u16| output.accounting_entries.iter().find(|entry| entry.statistic == statistic).expect("统计量").value;
        let rows_of = |statistic: u16| output.accounting_entries.iter().filter(|entry| entry.statistic == statistic).count();
        assert_eq!(value_of(STATISTIC_ALLOCATED_BYTES), 212_992, "每盘 13 槽 × 16384");
        assert_eq!(value_of(STATISTIC_FREE_BYTES), 3_472_670_720, "(211968 − 13) × 16384");
        assert_eq!(value_of(STATISTIC_ALLOCATED_BYTES) + value_of(STATISTIC_FREE_BYTES), UNIT_AREA_SLOTS * SLOT_BYTES);
        assert_eq!(value_of(STATISTIC_EMPTY_CLUSTER_SEGMENTS), 3310, "3312 个 64 槽段里两个被占");
        assert_eq!(value_of(STATISTIC_FRAGMENTATION_RUNS), 4, "空闲 run：[50179]、[50182, 50239]、[50241]、[50249, 单元区末]");
        assert_eq!(value_of(STATISTIC_INODE_WATERMARK), 2);
        // 准入不等式要的四项 day-1 各写一行 0：读不到行是坏账，读到 0 是真的 0。
        for statistic in [STATISTIC_UNRECLAIMABLE_BYTES, STATISTIC_DEFER_QUEUE_BYTES, STATISTIC_PENDING_DELETE_BYTES, STATISTIC_COMMITTED_RESERVATION_BYTES] {
            assert_eq!(value_of(statistic), 0);
        }
        // 哪几项带设备维、哪几项是池级的：逐项数行数**并逐项核 key 里的设备段**——
        // 只数行数抓不到「两行都写成池级」（那也是两行），这一格是第七次跑的变异 M55 逼出来的。
        let devices_of = |statistic: u16| -> Vec<u32> {
            output.accounting_entries.iter().filter(|entry| entry.statistic == statistic).map(|entry| entry.device.0).collect()
        };
        for statistic in [STATISTIC_ALLOCATED_BYTES, STATISTIC_FREE_BYTES, STATISTIC_UNRECLAIMABLE_BYTES, STATISTIC_DEFER_QUEUE_BYTES, STATISTIC_FRAGMENTATION_RUNS, STATISTIC_EMPTY_CLUSTER_SEGMENTS] {
            assert_eq!(rows_of(statistic), 2, "带设备维的统计量每盘一行");
            assert_eq!(devices_of(statistic), vec![0, 1], "带设备维的统计量，key 的设备段写真实设备号");
        }
        for statistic in [STATISTIC_PENDING_DELETE_BYTES, STATISTIC_COMMITTED_RESERVATION_BYTES, STATISTIC_INODE_WATERMARK] {
            assert_eq!(rows_of(statistic), 1, "池级统计量一行");
            assert_eq!(devices_of(statistic), vec![STATISTIC_NO_DEVICE_DIMENSION], "池级统计量的设备段取保留值");
        }
        let inode_watermark_row = output.accounting_entries.iter().find(|entry| entry.statistic == STATISTIC_INODE_WATERMARK).expect("水位");
        assert_eq!(inode_watermark_row.tree, TreeIdentifier(TREE_IDENTIFIER_INODE), "第 12 项带树维：树 ID 就是可写头的 inode 树");
        assert_eq!(inode_watermark_row.device, DeviceIdentity(STATISTIC_NO_DEVICE_DIMENSION), "不带设备维的设备段取 0xFFFF_FFFF");
        assert_eq!(output.root.tree_identifier_watermark, 19, "树 ID 11..18，水位 19");
        assert_eq!(output.root.rollback_floor, CheckpointTxg(0));
        assert_eq!(output.root.mapping_root.head.birth_tree, TreeIdentifier(TREE_IDENTIFIER_MAPPING), "中央映射树的根住根记录");
    }

    /// 树表单元第 1 版 7 条（D8 已定项 8 / 已定项 9，2026-09-14 用户定案加 deadlist）：
    /// 条目按树 ID 升序、树 ID 打头当 key、头 ID 按 D5 已定项 9、预留 24 恒 0。
    #[test]
    fn the_tree_table_holds_seven_entries_keyed_by_tree_identifier() {
        let BuiltPool { output, .. } = built_pool();
        let node = parse_index_node(&output.units_by_slot[7].2).expect("树表单元");
        assert_eq!(node.entries.len(), 7);
        assert_eq!(node.key_width, TREE_TABLE_KEY_WIDTH);
        let parsed: Vec<TreeTableEntry> = node.entries.iter().map(|bytes| TreeTableEntry::parse(bytes).expect("树表条目")).collect();
        assert_eq!(parsed.iter().map(|entry| entry.tree.0).collect::<Vec<_>>(), vec![11, 12, 13, 14, 16, 17, 18]);
        assert_eq!(parsed.iter().map(|entry| entry.kind).collect::<Vec<_>>(), vec![1, 2, 3, 4, 6, 7, 8]);
        assert_eq!(parsed.iter().map(|entry| entry.head_identifier).collect::<Vec<_>>(), vec![12, 12, 0, 0, 0, 0, 0]);
        assert!(parsed.iter().all(|entry| entry.birth_txg == CheckpointTxg(3)), "七条诞生 txg 都是 3");
        assert_eq!(parsed.iter().filter(|entry| entry.root.is_empty_root()).count(), 3, "livelist / 稀疏旁表 / deadlist 三条根指针为零");
        // 条目里 key 一律打头（D8 已定项 11，2026-09-14 用户定案）：区间就是首末两条的前 8 字节。
        assert_eq!(&node.entries[0][..8], &TREE_IDENTIFIER_EXTENT.to_le_bytes()[..]);
        assert_eq!(&node.entries[6][..8], &TREE_IDENTIFIER_DEADLIST.to_le_bytes()[..]);
        assert_eq!(node.smallest_key, TREE_IDENTIFIER_EXTENT.to_le_bytes().to_vec());
        assert_eq!(node.largest_key, TREE_IDENTIFIER_DEADLIST.to_le_bytes().to_vec());
        let declared_length = u16::from_le_bytes(output.units_by_slot[7].2[8..10].try_into().expect("切了 2 字节"));
        assert_eq!(declared_length, 1036, "7 × 148");
        // 预留 24 里塞一个非零字节：条目解析当场拒绝。
        let mut tampered = node.entries[0].clone();
        tampered[TREE_TABLE_ENTRY_BYTES as usize - 1] = 1;
        assert_eq!(TreeTableEntry::parse(&tampered).unwrap_err(), UnitError::Structure("树表条目预留 24 字节非零"));
    }

    #[test]
    fn instance_table_row_is_88_bytes_and_the_inode_leaf_holds_one_140_byte_record() {
        let BuiltPool { genesis, output, .. } = built_pool();
        let instance_table = parse_packed_unit(&genesis.instance_table_unit).expect("实例表");
        assert_eq!(instance_table.record_width as u64, INSTANCE_ROW_BYTES);
        assert_eq!(instance_table.records.len(), 1);
        assert_eq!(instance_table.identity.record_type, PACKED_TYPE_INSTANCE_TABLE);
        let leaf = parse_packed_unit(&output.units_by_slot[2].2).expect("inode 叶");
        assert_eq!(output.units_by_slot[2].1, TransactionUnit::InodeLeaf, "t3 就是 units 里第三个");
        assert_eq!(leaf.record_width as u64, INODE_RECORD_BYTES);
        assert_eq!(leaf.records.len(), 1);
        assert_eq!(InodeRecord::parse(&leaf.records[0]).expect("记录").size, 3000);
        let tree_table = parse_index_node(&genesis.tree_table_genesis_unit).expect("树表第 0 版");
        assert_eq!(tree_table.entries.len(), 0);
    }
}
