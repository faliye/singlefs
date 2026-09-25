//! E142：第一个事务的干跑——把 `.claude/kb/layout/01-first-txn.md` 那张字节表原样写成字节，
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
/// + 中央映射树根指针 86 + 分配记录树根指针 86（C512（树表 0 条的一版上被换下的实例表记在哪没有条款） 2026-09-23 用户定案加）
/// + 算法类型 1 + nonce 12 + MAC 16（末尾三段 2026-09-14 用户定案加）。
const ROOT_RECORD_BYTES: u64 = 457;
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
/// + 根指针 86 + previous_snapshot_txg 8 + 诞生 txg 8 + 头 ID 8 + 预留 76（D8（核心索引结构） 已定项 8，2026-09-16 用户定案）。
const TREE_TABLE_ENTRY_BYTES: u64 = 200;
/// D8（核心索引结构） 已定项 6。
const INODE_RECORD_BYTES: u64 = 140;
/// D8（核心索引结构） 已定项 6：分隔 key 8 + 身份引用 26 + 子指针 86。
const INODE_INTERNAL_ENTRY: u64 = 120;
/// D8（核心索引结构） 已定项 3 + D19 已定项 8：key 24 + 指向码 1 的指针 88。
/// E142 第十五次跑起，第一个事务不再写这个宽度（extent 树改按位置寻址、根即上段叶，见下），
/// 常量与它的 `name=width` 行留着标「附带」（下段叶条目，第一个事务走不到）。
const EXTENT_LEAF_RECORD_BYTES: u64 = 112;
// ───────────────────────── E142 第十五次跑：D8（核心索引结构） 已定项 14「实现取值」的八个格式常量 ─────────────────────────
// 按位置寻址的两棵派生树（分配记录树、extent 树）。值抄自 `.claude/kb/decisions/08-核心索引结构.md` 已定项 14
// 「实现取值」段落里的 `format-const` 标记（第二节整段抄），门禁 27 号按名字绑值。
/// 分配记录树叶宽 W = 812（叶条目容量 (16384 − 135) ÷ 20 本身，D3 已定项 7 / 已定项 11 的记录宽 20）。
const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;
/// 分配记录树内部条目 96 = key 10（位置 key，与 `ALLOCATION_KEY_BYTES` 同构）+ 子指针 86。
const ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES: u64 = 96;
/// 分配记录树内部扇出 169 = (16384 − 135) ÷ 96。
const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;
/// extent 树上段叶条目 113 = key 24 `(0, inode, 0)` + 标签 1 + 载荷 88（数据指针）。
const EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES: u64 = 113;
/// extent 树上段叶罩 143 个 inode = (16384 − 163) ÷ 113。5.2 ε 甲（稀疏，唯一写法）只写有文件的条目，
/// 但这个数仍然是 key 区间的分片宽度（D18 已定项 2 射程：第 k 片罩 `[143k, 143k + 142]`），写路径与读路径都用它。
const EXTENT_TREE_UPPER_LEAF_INODES: u64 = 143;
/// extent 树内部条目 110 = key 24 + 子指针 86。第一个事务只有一个 inode，走不到内部节点（留给下段路径）。
#[allow(dead_code, reason = "上段内部节点，第一个事务这个装置走不到；单测钉住数值，留着给以后写多 inode 路径用")]
const EXTENT_TREE_INTERNAL_ENTRY_BYTES: u64 = 110;
/// extent 树内部扇出 147 = (16384 − 163) ÷ 110。
#[allow(dead_code, reason = "上段内部节点，第一个事务这个装置走不到；单测钉住数值，留着给以后写多 inode 路径用")]
const EXTENT_TREE_INTERNAL_FANOUT: u64 = 147;
/// extent 树下段叶罩 144 个数据单元；下段 key 的字节偏移 = 单元号 × 32634（第一个事务走不到下段）。
#[allow(dead_code, reason = "下段叶，第一个事务这个装置走不到；单测钉住数值，留着给以后写下段路径用")]
const EXTENT_TREE_LOWER_LEAF_DATA_UNITS: u64 = 144;
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
/// 记录头（D23（journal 的角色与格式） 已定项 4 的字段表）：十个字段 78（`JOURNAL_HEADER_TEN_FIELD_BYTES`，
/// 含「记录标志」那一字节，位 0 = 本次发布末条，已定项 17） + 事务号 8 + 提交标记 1 + 本次发布内序号 4
/// + 反向链 4 + 载荷校验和 4 + 新根段 188 + fsid 8 + MAC 16 = 311（用户 2026-09-24 定案，E142 第十四次跑）。
/// 4096 的记录装 (4096 − 311) / 56 = 67 个点名项。
const JOURNAL_HEADER_BYTES: u64 = 311;
/// 十个字段那一段另登记成格式常量，好让门禁分得清它与整个头（D23 已定项 4）。
const JOURNAL_HEADER_TEN_FIELD_BYTES: u64 = 78;
/// D23（journal 的角色与格式） 已定项 15：新根段 = 树表单元指针 86 + 中央映射树根指针 86 + 树 ID 水位 8 + 回退下界 F 8。
const JOURNAL_NEW_ROOT_SEGMENT_BYTES: u64 = 2 * NODE_POINTER_BYTES + 8 + 8;
/// D23（journal 的角色与格式） 已定项 4 口径的点名项宽度；构成无落点，装置按字节表六的预想构成写。
const JOURNAL_NAMED_ENTRY_BYTES: u64 = 56;
/// 系统配置字段表合计（D22（单元原子性怎么合成） 已定项 9 + 已定项 15）：2026-09-14 用户定案加四个字段——
/// 自举头的写入者身份 20 与校验和算法标识 1、几何段的 mkfs 时 physical_block_size 4 与扩展点声明值 N 4，共 29。
const SYSTEM_CONFIGURATION_BYTES: u64 = 481;
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

/// 字节表零：系统配置槽 0 / 1 的设备内偏移。槽距 = 固定结构槽距 4096，与槽宽同值 ⇒ 两个槽首尾相接。
const SYSTEM_CONFIGURATION_SLOT_OFFSETS: [u64; 2] = [0, 4096];
/// D22（单元原子性怎么合成） 已定项 2 的槽宽那一格（2026-09-14 三方论证后按主 agent 推荐值写）：
/// **系统配置槽宽是格式常量 4096**，不再等于挂载时探测到的 `physical_block_size`。
/// 整槽校验和罩这 4096 字节含补齐（D18（块里携带什么信息） 已定项 17），481 字节的字段表在槽里余 3615。
/// ⚠️ 它只管系统配置：根槽仍按判定宽度 512 写（字节表七）。
const SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = 4096;
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
/// 系统配置几何段的「journal 最坏占用」：在飞记录数上限 × 一条记录的字节数（I-8.1（环几何够大））。
const JOURNAL_WORST_CASE_BYTES: u64 = JOURNAL_IN_FLIGHT_RECORD_LIMIT * JOURNAL_RECORD_BYTES;

/// D3（空间分配） 已定项 10 ④：单元区起始槽号进系统配置，第一版 = journal 环末尾的下一个槽（16 MiB + 768 MiB = 784 MiB）。
const UNIT_AREA_START_SLOT: u64 = JOURNAL_START_SLOT + JOURNAL_RING_BYTES / SLOT_BYTES;
/// 镜像大小是 mkfs 参数（跟 fsid、写入时刻同一类），装置取 4 GiB 并在 `name=config` 里报出来。
/// 下界由 D23（journal 的角色与格式） 已定项 19 ③ 的「环 ≤ 设备容量 ÷ 4」逼出：默认 768 MiB 的环要 3 GiB 以上的盘。
const DEVICE_BYTES: u64 = 4 << 30;
/// D8（核心索引结构） 已定项 14 第 408 行「取盘上整槽数，盘尾不足一槽的零头不算」（δ 甲，唯一写法）：
/// 整数除法本身就是向下取整，写侧与读侧、G3 的形状函数都经这一个 `const fn`（M142 锚在它上面）。
const fn slots_of_device_bytes(device_bytes: u64) -> u64 {
    device_bytes / SLOT_BYTES
}
/// 分配记录树按位置寻址时的 key 空间（D8 已定项 14 δ甲，第七节 B3「device_slots」）：
/// 绝对槽号从盘首数，覆盖整个设备，不是单元区（δ乙，只在第八节 G3 的 4.5 GiB 点上与甲分得开）。
const DEVICE_SLOTS: u64 = slots_of_device_bytes(DEVICE_BYTES);
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
/// ⇒ 跳过 50241 拿 50242–50243；其后依次 bump。**槽 50241 因此空着**，它自己是一段空闲 run。
/// E142 第十五次跑起，分配记录树按位置寻址拆成 5 个节点（第七节 B2）：t5..t9 落 50245..50249，
/// 记账 / 映射 / 树表跟着顺延到 50250..50252（原来的 SLOT_ALLOCATION_ROOT 等三个名字随之作废）。
const SLOT_EXTENT_ROOT: u64 = 50240;
const SLOT_INODE_LEAF: u64 = 50242;
const SLOT_INODE_ROOT: u64 = 50244;
/// 分配记录树两片叶（层级 0）：罩盘 0 / 盘 1 的那一片（第七节 B2，`allocation_leaf_of_device_{0,1}`）。
const SLOT_ALLOCATION_LEAF_DEVICE_0: u64 = 50245;
const SLOT_ALLOCATION_LEAF_DEVICE_1: u64 = 50246;
/// 分配记录树两个层级 1 节点：罩盘 0 / 盘 1 的那一段（`allocation_internal_of_device_{0,1}`）。
const SLOT_ALLOCATION_INTERNAL_DEVICE_0: u64 = 50247;
const SLOT_ALLOCATION_INTERNAL_DEVICE_1: u64 = 50248;
/// 分配记录树根（层级 2，单一节点、按盘分流，`allocation_root`）。
const SLOT_ALLOCATION_ROOT: u64 = 50249;
const SLOT_ACCOUNTING_ROOT: u64 = 50250;
const SLOT_MAPPING_ROOT: u64 = 50251;
const SLOT_TREE_TABLE_FIRST_PUBLISH: u64 = 50252;
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
/// 登记表住 D9（加密） 已定项 14：码 0 = 不压缩；码 0 时压后长度恒 0，非 0 判该指针损坏。
const POINTER_COMPRESSION_CODE_NONE: u8 = 0;
const POINTER_COMPRESSED_LENGTH_NONE: u16 = 0;

/// D5（快照 / 空间记账机制） 已定项 9（2026-09-14 用户定案）：树表条目从 32 字节预留里切 8 字节存「头 ID」。
/// inode 树条目写自己（12）、extent 树条目写它服务的那个头（12），分配记录 / 记账 / livelist / 旁表 / deadlist 写 0。
const TREE_TABLE_HEAD_IDENTIFIER_NONE: u64 = 0;

/// D22（单元原子性怎么合成） 已定项 9（2026-09-14 用户定案）：系统配置自举头的「写入者身份」=
/// 实现标识 16 字节 ASCII 零补齐 + 版本 4 字节；第一版写 `singlefs-rs` 与 1（I-1.5（系统配置记管道身份）、D17（实现分层与第三方管道） 债 3）。
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
const SYSTEM_CONFIGURATION_MAGIC: [u8; 4] = *b"SFSB";
/// D15 已定项 4：incompat 位 0 = 第一条纯 SSD 布局线，mkfs 起就置上；位图小端、位 0 是第一个字节的最低位。
const INCOMPAT_FIRST_SSD_LINE_BIT: u8 = 0x01;
/// 三张位图各 32 字节（256 位）紧跟 magic 4 + 版本 2，incompat 在前、compat_ro 居中、compat 在后（D15 已定项 1）。
const SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET: usize = 4 + 2;
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
/// 第一次可写挂载取 max(系统配置, 根环) + 1 = 1，并先写进每一份系统配置之后才动单元。
const MKFS_INSTANCE_GENERATION: u32 = 0;
const FIRST_INSTANCE_GENERATION: u32 = 1;
/// D22（单元原子性怎么合成） 已定项 16：系统配置槽世代号从 1 起、每写一次 +1，写世代号 g 的那一次落在槽 `g mod 2`。
const SYSTEM_CONFIGURATION_GENERATION_AT_MKFS: u64 = 1;
const SYSTEM_CONFIGURATION_GENERATION_AT_INSTANCE_ACQUISITION: u64 = 2;
/// D16 已定项 8（暖机取戊，2026-09-13 用户定案）：mkfs 之后第一次可写挂载先连推空发布，直到本实例写成的根覆盖两块盘；
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
/// 根记录自证校验和、系统配置整槽校验和、journal `header_csum` 同口径——此前装置取 SHA-256，那是 gap G5，已收口。
fn wide_checksum_with_field_zeroed(bytes: &[u8], cover_end: usize, field_offset: usize) -> [u8; 32] {
    let mut covered = bytes[..cover_end].to_vec();
    covered[field_offset..field_offset + WIDE_CHECKSUM_BYTES as usize].fill(0);
    let mut field = [0u8; 32];
    field[..WIDE_CHECKSUM_CRC_BYTES].copy_from_slice(&castagnoli_crc32(&covered).to_le_bytes());
    field
}

// ───────────────────────── SHA-256（E142 第十一次跑量 5：与 crates/ 侧 `first_transaction_region_bytes` 的 `sha256=` 对齐）─────────────────────────
// 标准 FIPS 180-4 实现，不加新依赖（这个包只依赖 aes-gcm / chacha20poly1305，都不带现成的 SHA-256）。
// 64 个圆常量与 8 个初始哈希值都是规范里那两组固定质数算出来的值，单测钉了 "" 与 "abc" 两个标准测试向量。
const SHA256_INITIAL_HASH: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
const SHA256_ROUND_CONSTANTS: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786,
    0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a,
    0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut hash = SHA256_INITIAL_HASH;
    let bit_length = (bytes.len() as u64) * 8;
    let mut message = bytes.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_length.to_be_bytes());
    for chunk in message.chunks_exact(64) {
        let mut schedule = [0u32; 64];
        for (index, word) in schedule.iter_mut().take(16).enumerate() {
            *word = u32::from_be_bytes(chunk[index * 4..index * 4 + 4].try_into().expect("切了 4 字节"));
        }
        for index in 16..64 {
            let sigma0 = schedule[index - 15].rotate_right(7) ^ schedule[index - 15].rotate_right(18) ^ (schedule[index - 15] >> 3);
            let sigma1 = schedule[index - 2].rotate_right(17) ^ schedule[index - 2].rotate_right(19) ^ (schedule[index - 2] >> 10);
            schedule[index] = schedule[index - 16].wrapping_add(sigma0).wrapping_add(schedule[index - 7]).wrapping_add(sigma1);
        }
        let [mut register_first, mut register_second, mut register_third, mut register_fourth, mut register_fifth, mut register_sixth, mut register_seventh, mut register_eighth] = hash;
        for index in 0..64 {
            let big_sigma1 = register_fifth.rotate_right(6) ^ register_fifth.rotate_right(11) ^ register_fifth.rotate_right(25);
            let choose = (register_fifth & register_sixth) ^ ((!register_fifth) & register_seventh);
            let temporary_one = register_eighth.wrapping_add(big_sigma1).wrapping_add(choose).wrapping_add(SHA256_ROUND_CONSTANTS[index]).wrapping_add(schedule[index]);
            let big_sigma0 = register_first.rotate_right(2) ^ register_first.rotate_right(13) ^ register_first.rotate_right(22);
            let majority = (register_first & register_second) ^ (register_first & register_third) ^ (register_second & register_third);
            let temporary_two = big_sigma0.wrapping_add(majority);
            register_eighth = register_seventh;
            register_seventh = register_sixth;
            register_sixth = register_fifth;
            register_fifth = register_fourth.wrapping_add(temporary_one);
            register_fourth = register_third;
            register_third = register_second;
            register_second = register_first;
            register_first = temporary_one.wrapping_add(temporary_two);
        }
        for (slot, value) in hash.iter_mut().zip([register_first, register_second, register_third, register_fourth, register_fifth, register_sixth, register_seventh, register_eighth]) {
            *slot = slot.wrapping_add(value);
        }
    }
    let mut digest = [0u8; 32];
    for (index, word) in hash.iter().enumerate() {
        digest[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    digest
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex_bytes(&sha256(bytes))
}

fn hex_decode(hex: &str) -> Vec<u8> {
    (0..hex.len() / 2).map(|index| u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).unwrap_or(0)).collect()
}

/// 一行 `E7RESULT name=... key=value key=value...` 拆成 key→value；量 5 用它读 crates 侧的产出（自己格式自己拆，不新加依赖）。
fn parse_result_line(line: &str) -> std::collections::BTreeMap<String, String> {
    line.split_whitespace().filter_map(|token| token.split_once('=')).map(|(key, value)| (key.to_string(), value.to_string())).collect()
}

/// 两段等长字节比出第一个不同的下标与不同字节总数；长度不等时直接报「整段都算」。
fn byte_diff_summary(left_bytes: &[u8], right_bytes: &[u8]) -> (Option<u64>, Option<u64>) {
    if left_bytes.len() != right_bytes.len() {
        return (Some(0), Some(left_bytes.len().max(right_bytes.len()) as u64));
    }
    let mut first_diff = None;
    let mut mismatch_count = 0u64;
    for (index, (left, right)) in left_bytes.iter().zip(right_bytes.iter()).enumerate() {
        if left != right {
            if first_diff.is_none() {
                first_diff = Some(index as u64);
            }
            mismatch_count += 1;
        }
    }
    (first_diff, Some(mismatch_count))
}

// E142 第十五次跑步④：旧的「量 5 一个区域的比较结果」（`RegionComparison`/`compare_region`，按名字配对、
// 支持 head_and_tail 抽样）被 Q142.1 的新比对器取代，删除理由见文件末尾单测模块同一处的注释。

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
    /// 系统配置槽的一次写。
    SystemConfigurationSlot,
    /// mkfs 把根环三个区域与 journal 环各整段写 0（D22（单元原子性怎么合成） 已定项 8 第 3 条，
    /// 2026-09-23 用户定案改写）：两者都由 mkfs 自己经块设备的写零动作发出，每块盘上四段（根环三个区域各一段 +
    /// journal 环一段）各一次调用，区域归属只定根种在哪块盘、不定清哪块盘——两块盘各清全部四段，共 8 步；
    /// 录制流里各记一步整段清零，清零几段之间不加屏障。不是 FUA。
    ZeroFill,
}

impl StepKind {
    /// 发到输出里的名字，lowercase snake_case，一个种类一个。
    fn tag(self) -> &'static str {
        match self {
            StepKind::UnitWrite => "unit_write",
            StepKind::JournalRecord => "journal_record",
            StepKind::RootRecordFua => "root_record_fua",
            StepKind::SystemConfigurationSlot => "system_configuration_slot",
            StepKind::ZeroFill => "zero_fill",
        }
    }
    /// FUA 由步骤种类决定，不再是调用点各传各的布尔：系统配置槽写不可能是 FUA，这样它写不出来。
    fn is_fua(self) -> bool {
        match self {
            StepKind::RootRecordFua => true,
            StepKind::UnitWrite | StepKind::JournalRecord | StepKind::SystemConfigurationSlot | StepKind::ZeroFill => false,
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
    /// mkfs 的整段清零（D22（单元原子性怎么合成） 已定项 8 第 3 条）：只记一步操作，不真的把字节灌进
    /// `SparseDevice` 的扇区表。稀疏镜像没写过的扇区恒读 0（`SparseDevice` 的文档），根环区域与
    /// journal 环这两类清零写的本来就是全 0 的字节，语义上跟真写一遍没有分别；journal 环有 768 MiB，
    /// 真按扇区写一遍会让 `mkfs()` 每次调用都往 `BTreeMap` 里插上百万条目——78 条变异 × 50 个单测
    /// 每条都要跑一次 `mkfs()`，会把这个装置拖到不可用，`mutate.sh` 单条变异 120 秒的超时也会被跑穿。
    fn record_zero_fill(&mut self, device: DeviceIdentity, offset: DeviceOffset) {
        self.operations.push(RecordedOperation::Write(WriteRequest { device, offset, bytes: Vec::new(), kind: StepKind::ZeroFill }));
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

/// fsid 在单元头里是 8 字节（D18 已定项 7）；字节表二预想取系统配置 fsid 的低 8 字节。
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

// ───────────────────────── 根记录、系统配置、journal 记录 ─────────────────────────

/// D22（单元原子性怎么合成） 已定项 7 的字段表，457 字节，字段序照那张表（gap G4：字节表七的行序与它不同，两处都没写偏移）。
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
    /// C512（树表 0 条的一版上被换下的实例表记在哪没有条款） 2026-09-23 用户定案：只有「树表 0 条、写过行」
    /// 的一版指着写行那次发布写的那一片分配记录节点；这个装置从不建那一格（E142（第一个事务的干跑） 只有
    /// mkfs、暖机与带文件的一版，milestone 二「第三次可写挂载」不在射程内），恒为 `NodePointer::empty_root()`。
    allocation_record_tree_root: NodePointer,
}

const ROOT_CHECKSUM_OFFSET: usize = 4 + 16 + 4 + 4 + 8 + NODE_POINTER_BYTES as usize + 8 + 8;

impl RootRecord {
    /// 写成一个判定宽度的槽：记录 457 字节，其余补 0；
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
        // C512（树表 0 条的一版上被换下的实例表记在哪没有条款） 2026-09-23 用户定案：分配记录树根指针紧跟映射根指针之后。
        writer.assert_position(ROOT_CHECKSUM_OFFSET as u64 + WIDE_CHECKSUM_BYTES + 2 * NODE_POINTER_BYTES, "根记录：分配记录树根指针起点");
        self.allocation_record_tree_root.write_to(&mut writer);
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
        let allocation_record_tree_root = NodePointer::read_from(&mut reader);
        Some(Self { fsid, instance, checkpoint_txg, tree_table, tree_identifier_watermark, rollback_floor, instance_table, mapping_root, allocation_record_tree_root })
    }
}

/// 系统配置字段表（D22 已定项 9 + 已定项 15）：**481 字节**，512 槽内余 31。
/// 2026-09-13 用户定案去掉「间接目录单元指针 59」（第一版显式留白，不是留位），几何段加三个字段：
/// 单元区起始槽号 8（D3 已定项 10 ④）、mkfs 时的 io_min 4 与固定结构槽距 4（D2 已定项 19）；
/// 2026-09-14 用户定案再加四个：自举头的写入者身份 20 与校验和算法标识 1、
/// 几何段的 mkfs 时 physical_block_size 4 与扩展点声明值 N 4。
#[derive(Clone, PartialEq, Eq, Debug)]
struct SystemConfiguration {
    fsid: [u8; 16],
    this_device: DeviceIdentity,
    device_count: u32,
    slot_generation: u64,
    region_devices: [DeviceIdentity; 3],
    journal_tail: u64,
    journal_instance: InstanceGeneration,
}

/// magic 4 + 格式版本 2 + feature bits 96 + fsid 16 + 写入者身份 20 + 校验和算法标识 1 + 本盘设备号 4 + 设备数 4 + 槽世代号 8。
const SYSTEM_CONFIGURATION_CHECKSUM_OFFSET: usize = 4 + 2 + 96 + 16 + 20 + 1 + 4 + 4 + 8;
/// fsid 住自举头里 magic 4 + 格式版本 2 + feature bits 96 之后。
const SYSTEM_CONFIGURATION_FSID_OFFSET: usize = 4 + 2 + 96;
const SYSTEM_CONFIGURATION_REGION_DEVICES_OFFSET: usize = 379;
const SYSTEM_CONFIGURATION_TAIL_OFFSET: usize = 469;
/// D2（RAID 条带策略） 已定项 19：固定结构槽距 = max(4096, mkfs 时探测的 io_min)，两个数各占系统配置一个 4 字节字段。
const FIXED_STRUCTURE_SLOT_SPACING: u32 = 4096;
const MKFS_MINIMUM_INPUT_OUTPUT_BYTES: u32 = 512;
/// D2（RAID 条带策略） 已定项 18：第一版系统配置里 w_max 与 g 都写 4；g 挂载时按可写设备数夹取。
const SYSTEM_CONFIGURATION_MAXIMUM_WIDTH: u8 = 4;
const SYSTEM_CONFIGURATION_GROUP_SIZE: u8 = 4;

impl SystemConfiguration {
    fn to_slot(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(SYSTEM_CONFIGURATION_SLOT_BYTES as usize);
        writer.put(&SYSTEM_CONFIGURATION_MAGIC);
        writer.put_u16(FORMAT_VERSION);
        writer.put_u8(INCOMPAT_FIRST_SSD_LINE_BIT); // feature bits：incompat 位 0 = 第一条纯 SSD 布局线（D15 已定项 4，2026-09-13 用户定案）
        writer.skip(95); // 其余 incompat 位与 compat_ro / compat 两张位图全 0
        writer.assert_position(SYSTEM_CONFIGURATION_FSID_OFFSET as u64, "系统配置 fsid");
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
        writer.assert_position(SYSTEM_CONFIGURATION_CHECKSUM_OFFSET as u64, "系统配置整槽校验和");
        writer.skip(WIDE_CHECKSUM_BYTES as usize);
        writer.skip(16 + 12 + 4); // 系统配置 MAC、nonce 水位、KDF 标识（4，D22 已定项 9）
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
        // journal 最坏占用 = 在飞上限 × 记录宽（字节表 `layout/01-first-txn.md` 那一行逐字
        // 「268 435 456（65536 条 × 4096）」；此前这里写成了记录宽本身 4096）。
        writer.put_u64(JOURNAL_WORST_CASE_BYTES);
        writer.put_u32(u32::try_from(JOURNAL_SAFETY_FACTOR).expect("安全系数 4 字节"));
        writer.put_u8(RING_REGIONS as u8);
        writer.put_u8(RING_SLOTS_PER_REGION as u8);
        writer.put_u32(RING_PRIME_STEP as u32);
        writer.put_u32(RING_CHUNK_BYTES as u32);
        writer.put_u64(RING_START_OFFSET / SLOT_BYTES);
        writer.assert_position(SYSTEM_CONFIGURATION_REGION_DEVICES_OFFSET as u64, "根环逐区域设备身份");
        for region_device in &self.region_devices {
            writer.put_u32(region_device.0);
        }
        writer.put_u8(SYSTEM_CONFIGURATION_MAXIMUM_WIDTH);
        writer.put_u8(SYSTEM_CONFIGURATION_GROUP_SIZE);
        writer.skip(24); // 映射来源
        // D22 已定项 15 加的三个几何字段，排在「映射来源」之后。
        writer.put_u64(UNIT_AREA_START_SLOT);
        writer.put_u32(MKFS_MINIMUM_INPUT_OUTPUT_BYTES);
        writer.put_u32(FIXED_STRUCTURE_SLOT_SPACING);
        writer.put_u32(5); // T_time 秒
        writer.put_u64(2 << 30); // T_dirty
        writer.skip(24); // 整理三条水位：第一版恒 0 = 内置默认（D22 已定项 15）
        writer.assert_position(SYSTEM_CONFIGURATION_TAIL_OFFSET as u64, "journal tail");
        writer.put_u64(self.journal_tail);
        writer.put_u32(self.journal_instance.0);
        writer.assert_position(SYSTEM_CONFIGURATION_BYTES, "系统配置");
        let mut bytes = writer.bytes;
        // 「整槽校验和」：覆盖整个 4096 槽含补齐、自身按 0 参与（D18 已定项 17）。
        let digest = wide_checksum_with_field_zeroed(&bytes, SYSTEM_CONFIGURATION_SLOT_BYTES as usize, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
        bytes[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        bytes
    }

    fn parse_slot(bytes: &[u8]) -> Option<Self> {
        if bytes[..4] != SYSTEM_CONFIGURATION_MAGIC {
            return None;
        }
        if wide_checksum_with_field_zeroed(bytes, SYSTEM_CONFIGURATION_SLOT_BYTES as usize, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET) != bytes[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32] {
            return None;
        }
        if !incompat_bits_are_mountable(bytes) {
            return None;
        }
        let mut reader = ByteReader::at(bytes, SYSTEM_CONFIGURATION_FSID_OFFSET);
        let fsid: [u8; 16] = reader.take(16).try_into().expect("切了 16 字节");
        reader.skip(WRITER_IDENTITY_NAME_BYTES + 4); // 写入者身份：读者不判它，只要在
        if reader.get_u8() != CHECKSUM_ALGORITHM_CRC32_CASTAGNOLI {
            return None; // I-2.2（校验和算法全盘一致）：算法标识不是已登记的 CRC-32C 就不认这个槽
        }
        let this_device = DeviceIdentity(reader.get_u32());
        let device_count = reader.get_u32();
        let slot_generation = reader.get_u64();
        let mut region_reader = ByteReader::at(bytes, SYSTEM_CONFIGURATION_REGION_DEVICES_OFFSET);
        let region_devices = [DeviceIdentity(region_reader.get_u32()), DeviceIdentity(region_reader.get_u32()), DeviceIdentity(region_reader.get_u32())];
        let mut tail_reader = ByteReader::at(bytes, SYSTEM_CONFIGURATION_TAIL_OFFSET);
        let journal_tail = tail_reader.get_u64();
        let journal_instance = InstanceGeneration(tail_reader.get_u32());
        Some(Self { fsid, this_device, device_count, slot_generation, region_devices, journal_tail, journal_instance })
    }
}

/// D15 已定项 4 与 fs-design「格式层的让非法状态无法表示」：incompat 位图里有读者不认识的位、
/// 或第一条 SSD 线那一位没置（没有布局身份），都拒绝挂载；compat_ro / compat 两张位图不认识随便，读者不看。
fn incompat_bits_are_mountable(slot: &[u8]) -> bool {
    let incompat = &slot[SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET..SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET + FEATURE_BITMAP_BYTES];
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

/// journal 记录 4096：头 311（字节表六的表序 + D23 已定项 15 的新根段 + 2026-09-14 加的 fsid 与 MAC + 2026-09-24 加的记录标志与本次发布内序号）+ 点名项数组。
#[derive(Clone, PartialEq, Eq, Debug)]
struct JournalRecord {
    instance: InstanceGeneration,
    counter: JournalCounter,
    checkpoint_txg: CheckpointTxg,
    transaction: TransactionNumber,
    is_commit: bool,
    /// D23（journal 的角色与格式） 已定项 4：本次发布内序号，从 1 起；本实验每次发布只有一条记录，恒 1。
    ordinal_within_publish: u32,
    /// D23（journal 的角色与格式） 已定项 4 / 17：记录标志（位 0 = 本次发布末条），其余位恒 0。
    record_flags: u8,
    back_chain: u32,
    /// D23（journal 的角色与格式） 已定项 4（2026-09-14 用户定案）：系统配置 fsid 的低 8 字节，与单元头同口径（I-1.4（块头 fsid 一致））。
    fsid: u64,
    /// D23（journal 的角色与格式） 已定项 15 的新根段：崩在记录持久之后、根槽持久之前时由它重建那次发布的根。
    new_tree_table: NodePointer,
    new_mapping_root: NodePointer,
    new_tree_identifier_watermark: u64,
    new_rollback_floor: CheckpointTxg,
    named: Vec<NamedUnit>,
}

const JOURNAL_HEADER_CHECKSUM_OFFSET: usize = 4 + 2 + 1 + 1 + 4 + 4 + 10 + 8 + 12;
/// D23（journal 的角色与格式） 已定项 4：记录标志（位 0 = 本次发布末条）的偏移 = magic 4 + 类型 2 + 算法类型 1。
const JOURNAL_RECORD_FLAGS_OFFSET: usize = 4 + 2 + 1;

/// D23（journal 的角色与格式） 已定项 19 ②（2026-09-14 用户定案改写）：`previous_hash` =
/// CRC32C(**本实例内逻辑前一条**记录的 311 字节头，其中 `header_csum` 那 32 字节按零参与)；本实例第一条恒 0、不读盘。
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
        writer.assert_position(7, "记录标志偏移（已定项 4）");
        writer.put_u8(self.record_flags); // 记录标志：位 0 = 本次发布末条（已定项 4、17）
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
        writer.assert_position(87, "本次发布内序号偏移（已定项 4）");
        writer.put_u32(self.ordinal_within_publish);
        writer.put_u32(self.back_chain);
        let payload_checksum_offset = writer.position();
        assert_eq!(payload_checksum_offset, 95, "载荷校验和偏移（已定项 4）");
        writer.skip(4);
        let new_root_segment_start = writer.position();
        self.new_tree_table.write_to(&mut writer);
        self.new_mapping_root.write_to(&mut writer);
        writer.put_u64(self.new_tree_identifier_watermark);
        writer.put_u64(self.new_rollback_floor.0);
        assert_eq!(writer.position() - new_root_segment_start, JOURNAL_NEW_ROOT_SEGMENT_BYTES as usize, "新根段");
        // 2026-09-14 用户定案：新根段之后是 fsid 8，再之后是 MAC 16（第一版全 0 留位，D9（加密） 已定项 12 第 1 项）。
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
        reader.skip(1); // 算法类型
        let record_flags = reader.get_u8(); // 偏移 7：位 0 = 本次发布末条（已定项 4、17）；「当损坏」的判断在读者一侧（第五节 5.1）
        let record_length = reader.get_u32();
        let named_count = reader.get_u32() as usize;
        let instance = InstanceGeneration(reader.get_u32());
        let counter = JournalCounter(reader.get_six_byte_unsigned());
        let checkpoint_txg = CheckpointTxg(reader.get_u64());
        reader.skip(12 + WIDE_CHECKSUM_BYTES as usize);
        let transaction = TransactionNumber(reader.get_u64());
        let is_commit = reader.get_u8() == 1;
        let ordinal_within_publish = reader.get_u32(); // 偏移 87：本次发布内序号（已定项 4）
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
        Some(Self { instance, counter, checkpoint_txg, transaction, is_commit, ordinal_within_publish, record_flags, back_chain, fsid, new_tree_table, new_mapping_root, new_tree_identifier_watermark, new_rollback_floor, named })
    }
}

// ───────────────────────── 树表条目、inode、extent、分配、记账、映射 ─────────────────────────

/// D8（核心索引结构） 已定项 8（2026-09-14 用户定案重排）：树 ID 8 打头（= 码 2 条目的 key）+ 条目长度 2
/// + 树的种类 2 + flags 2 + 根指针 86 + previous_snapshot_txg 8 + 诞生 txg 8 + 头 ID 8 + 预留 76 = 200。
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
        writer.skip(76);
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
        if bytes[TREE_TABLE_ENTRY_BYTES as usize - 76..].iter().any(|&byte| byte != 0) {
            return Err(UnitError::Structure("树表条目预留 76 字节非零"));
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
        // C480（inode 记录的 blocks 怎么算全仓没有条款） 2026-09-23 用户定案：blocks = ⌈文件字节数 ÷ 512⌉
        // （逻辑长度的 512 字节块数，不表示分到的空间——不是 DATA_UNIT_BYTES / 512）。
        writer.put_u64(self.size.div_ceil(512));
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

/// extent 树下段叶记录 112：key (locality_id 8, inode 8, offset 8) + 指向码 1 的指针 88。
/// E142 第十五次跑起，第一个事务改走上段叶（`build_extent_upper_leaf_entry`）：只有一个数据单元的文件不建下段
/// （D8 已定项 14「只有一个数据单元的文件不建下段」），这两个函数留给下段路径（第一个事务走不到，标「附带」）。
#[allow(dead_code, reason = "下段叶格式，第一个事务这个装置走不到；留着给以后写下段路径用，不删")]
fn build_extent_record(inode: u64, offset: u64, pointer: DataPointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(EXTENT_LEAF_RECORD_BYTES as usize);
    writer.put_u64(0);
    writer.put_u64(inode);
    writer.put_u64(offset);
    pointer.write_to(&mut writer);
    writer.assert_position(EXTENT_LEAF_RECORD_BYTES, "extent 叶记录");
    writer.bytes
}

#[allow(dead_code, reason = "下段叶格式，第一个事务这个装置走不到；留着给以后写下段路径用，不删")]
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
    /// 已释放（跨度段最高位）：第一个事务里只有 mkfs 那片第 0 版树表单元——A 重写树表把它换下、进 defer 队列（D3 已定项 7），释放代 = A 的 txg。
    is_released: bool,
}

/// 分配记录跨度段的最高位 = 已释放（与 crates 的 `ALLOCATION_RECORD_RELEASED_FLAG` 同一位）。
const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;

impl AllocationRecord {
    fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(ALLOCATION_RECORD_BYTES as usize);
        writer.put_u32(self.device.0);
        writer.put_six_byte_unsigned(self.slot.0);
        writer.put_u16(if self.is_released { self.span_slots | ALLOCATION_RECORD_RELEASED_FLAG } else { self.span_slots });
        writer.put_u64(self.generation.0);
        writer.assert_position(ALLOCATION_RECORD_BYTES, "分配记录");
        writer.bytes
    }
    fn parse(bytes: &[u8]) -> Self {
        let mut reader = ByteReader::new(bytes);
        let device = DeviceIdentity(reader.get_u32());
        let slot = SlotNumber(reader.get_six_byte_unsigned());
        let span_field = reader.get_u16();
        let generation = CheckpointTxg(reader.get_u64());
        Self { device, slot, span_slots: span_field & !ALLOCATION_RECORD_RELEASED_FLAG, generation, is_released: span_field & ALLOCATION_RECORD_RELEASED_FLAG != 0 }
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
    /// 写者开关（E142 第十四次跑，重跑登记第五节 5.1）：臂 F（真实基线，`crates/` 今天的写法）恒 `0x01`；
    /// 臂 H（对照）恒 `0x00`。本实验每次发布只有一条记录，这条记录也就是它自己发布的末条，
    /// 已定项 17 因此要求它写 1——这个字段只用来在两条臂之间切换，不代表「这条记录是不是末条」有别的取法。
    last_of_publish_flag: u8,
}

impl PoolParameters {
    fn settled_two_devices() -> Self {
        Self {
            fsid: FIXED_FSID,
            device_count: 2,
            region_devices: [DeviceIdentity(RING_REGION_DEVICES[0]), DeviceIdentity(RING_REGION_DEVICES[1]), DeviceIdentity(RING_REGION_DEVICES[2])],
            barriers: BarrierPolicy::Settled,
            last_of_publish_flag: 0x01,
        }
    }
    fn control_one_device_no_barriers() -> Self {
        Self { fsid: FIXED_FSID, device_count: 1, region_devices: [DeviceIdentity(0), DeviceIdentity(0), DeviceIdentity(0)], barriers: BarrierPolicy::None, last_of_publish_flag: 0x01 }
    }
    /// 臂 H：把写者开关扳到 `0x00`（E142 第十四次跑第五节 5.1）；不改别的字段。
    fn with_last_of_publish_flag(mut self, flag: u8) -> Self {
        self.last_of_publish_flag = flag;
        self
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
    // D22（单元原子性怎么合成） 已定项 8 第 3 条（2026-09-23 用户定案改写，追平 C484（mkfs 不清根环，同 fsid 重来旧根还择得中））：
    // 根环三个区域与 journal 环都由 mkfs 经写零动作清，每块盘上四段（根环三个区域各一段 + journal 环一段）各一次调用，
    // 区域归属只定根种在哪块盘、不定清哪块盘——两块盘各清全部四段，8 步；段序列与步数以
    // `.claude/kb/layout/01-first-txn.md` 八「mkfs 种根」那一行为准（`12+1+1+1+4`、21 次操作、4114 个崩溃状态，
    // 种类 `[zero_fill×8,unit_write×4,barrier]|...`）；清零几段之间不加屏障。
    for device in parameters.devices() {
        for region_index in 0..RING_REGIONS {
            pool.record_zero_fill(device, ring_region_offset(region_index));
        }
        pool.record_zero_fill(device, DeviceOffset(JOURNAL_START_SLOT * SLOT_BYTES));
    }
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
        // C512 2026-09-23 用户定案：mkfs 第 0 代写全零。
        allocation_record_tree_root: NodePointer::empty_root(),
    };
    let root_slot = root.to_slot();
    // D22 已定项 8：第 0 代根种进全部区域，各自槽 0，FUA。
    for region in 0..RING_REGIONS {
        pool.write(parameters.region_devices[region as usize], ring_slot_offset(region, 0), &root_slot, StepKind::RootRecordFua);
    }
    // D22（单元原子性怎么合成） 已定项 16：每盘恒 2 个槽，世代号从 1 起；mkfs 把两个槽都种上世代号 1。
    for device in parameters.devices() {
        let system_configuration = SystemConfiguration {
            fsid: parameters.fsid,
            this_device: device,
            device_count: u32::try_from(parameters.device_count).expect("设备数"),
            slot_generation: SYSTEM_CONFIGURATION_GENERATION_AT_MKFS,
            region_devices: parameters.region_devices,
            journal_tail: 0,
            journal_instance: instance,
        };
        for slot_offset in SYSTEM_CONFIGURATION_SLOT_OFFSETS {
            pool.write(device, DeviceOffset(slot_offset), &system_configuration.to_slot(), StepKind::SystemConfigurationSlot);
        }
    }
    pool.barrier();
    (pool, MkfsOutput { root, instance_table_unit, tree_table_genesis_unit })
}

/// D23（journal 的角色与格式） 已定项 16：第一次可写挂载取 max(系统配置, 根环) + 1 = 1，
/// **并写进每一份系统配置（一次系统配置槽写，世代号 +1）之后才动单元**——所以它自成一段，排在暖机之前。
/// 段的收尾靠暖机第一次空发布开头那道屏障（D16 已定项 7 的形态，不另加屏障：这是最少屏障的写法）。
fn acquire_instance(pool: &mut RecordingPool, parameters: &PoolParameters) -> InstanceGeneration {
    let instance = InstanceGeneration(FIRST_INSTANCE_GENERATION);
    let slot_index = (SYSTEM_CONFIGURATION_GENERATION_AT_INSTANCE_ACQUISITION % 2) as usize;
    for device in parameters.devices() {
        let system_configuration = SystemConfiguration {
            fsid: parameters.fsid,
            this_device: device,
            device_count: u32::try_from(parameters.device_count).expect("设备数"),
            slot_generation: SYSTEM_CONFIGURATION_GENERATION_AT_INSTANCE_ACQUISITION,
            region_devices: parameters.region_devices,
            journal_tail: 0,
            journal_instance: instance,
        };
        pool.write(device, DeviceOffset(SYSTEM_CONFIGURATION_SLOT_OFFSETS[slot_index]), &system_configuration.to_slot(), StepKind::SystemConfigurationSlot);
    }
    instance
}

/// 分配记录树按位置寻址（D8（核心索引结构） 已定项 14）拆成的物理节点角色：两片叶（层级 0，一盘一片）、
/// 两个层级 1 节点（一盘一个，主几何下只需要这一层中间节点）、一个根（层级 2，单一节点、按盘分流）。
/// 这个装置的固定几何（两块 4 GiB 盘）只走得到这三层；`Internal` 带层级号是为了让第八节 G3 的形状函数
/// 能对更大的盘（如 256 GiB，根在层级 3）说话，即便这个枚举本身在这个场景里只用得到 `level == 1`。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
enum AllocationNodeRole {
    Leaf { device: u32 },
    Internal { device: u32, level: u32 },
    Root,
}

/// 第一个事务写出的十二个单元各自的身份。以前它是自由文本标签，靠 `match` 字符串取类与 key 宽，
/// 漏一个只能在运行期 panic；做成封闭枚举之后每一处 match 都穷举，漏一种编译不过。
/// 步骤种类（`StepKind`）说的是「这一步是哪一类写」，这个说的是「写的是哪个单元」——两件事分开。
/// E142 第十五次跑：分配记录树从裸的一个变体拆成 `Allocation(AllocationNodeRole)`，其余七个不变。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
enum TransactionUnit {
    Data,
    ExtentRoot,
    InodeLeaf,
    InodeRoot,
    Allocation(AllocationNodeRole),
    AccountingTree,
    MappingTree,
    TreeTable,
}

impl TransactionUnit {
    /// E142 第十一次跑量 1/3/4 用的可读标签（`tag()` 的 t1..t12 是既有产物格式，这里不改它）。
    /// 这个装置的场景只跑到两块盘（设备号 0 / 1），别的设备号在这里 panic——不是这一格没考虑到，
    /// 是这个实验的几何写死两盘，撞见别的设备号说明调用方传错了参数。
    fn descriptive_tag(self) -> &'static str {
        match self {
            TransactionUnit::Data => "data_unit",
            TransactionUnit::ExtentRoot => "extent_root",
            TransactionUnit::InodeLeaf => "inode_leaf",
            TransactionUnit::InodeRoot => "inode_root",
            TransactionUnit::Allocation(AllocationNodeRole::Leaf { device: 0 }) => "allocation_leaf_of_device_0",
            TransactionUnit::Allocation(AllocationNodeRole::Leaf { device: 1 }) => "allocation_leaf_of_device_1",
            TransactionUnit::Allocation(AllocationNodeRole::Leaf { device }) => panic!("这个装置只跑到两块盘：{device}"),
            TransactionUnit::Allocation(AllocationNodeRole::Internal { device: 0, .. }) => "allocation_internal_of_device_0",
            TransactionUnit::Allocation(AllocationNodeRole::Internal { device: 1, .. }) => "allocation_internal_of_device_1",
            TransactionUnit::Allocation(AllocationNodeRole::Internal { device, .. }) => panic!("这个装置只跑到两块盘：{device}"),
            TransactionUnit::Allocation(AllocationNodeRole::Root) => "allocation_root",
            TransactionUnit::AccountingTree => "accounting_root",
            TransactionUnit::MappingTree => "mapping_root",
            TransactionUnit::TreeTable => "tree_table",
        }
    }
    /// 字节表七里的步号，E142 第十五次跑起从 t1..t8 延伸到 t1..t12（第七节 B2）：
    /// t5/t6 两片叶、t7/t8 两个层级 1 节点、t9 根，t10..t12 顺延给记账/映射/树表。
    fn tag(self) -> &'static str {
        match self {
            TransactionUnit::Data => "t1",
            TransactionUnit::ExtentRoot => "t2",
            TransactionUnit::InodeLeaf => "t3",
            TransactionUnit::InodeRoot => "t4",
            TransactionUnit::Allocation(AllocationNodeRole::Leaf { device: 0 }) => "t5",
            TransactionUnit::Allocation(AllocationNodeRole::Leaf { device: 1 }) => "t6",
            TransactionUnit::Allocation(AllocationNodeRole::Leaf { device }) => panic!("这个装置只跑到两块盘：{device}"),
            TransactionUnit::Allocation(AllocationNodeRole::Internal { device: 0, .. }) => "t7",
            TransactionUnit::Allocation(AllocationNodeRole::Internal { device: 1, .. }) => "t8",
            TransactionUnit::Allocation(AllocationNodeRole::Internal { device, .. }) => panic!("这个装置只跑到两块盘：{device}"),
            TransactionUnit::Allocation(AllocationNodeRole::Root) => "t9",
            TransactionUnit::AccountingTree => "t10",
            TransactionUnit::MappingTree => "t11",
            TransactionUnit::TreeTable => "t12",
        }
    }
    /// D18（块里携带什么信息） 已定项 11 的类码，加上码 2 的 key 宽（码 1 与码 3 没有 key 区间，写 0）。
    /// D3（空间分配） 已定项 11：分配记录树 key 宽 10 对全部层级成立，不分叶或内部（位置 key 与分配记录 key 同构）。
    fn class_and_key_width(self) -> (u8, usize) {
        match self {
            TransactionUnit::Data => (UNIT_CLASS_DATA, 0),
            TransactionUnit::InodeLeaf => (UNIT_CLASS_PACKED, 0),
            TransactionUnit::ExtentRoot => (UNIT_CLASS_INDEX_NODE, 24),
            TransactionUnit::InodeRoot => (UNIT_CLASS_INDEX_NODE, 8),
            TransactionUnit::Allocation(_) => (UNIT_CLASS_INDEX_NODE, ALLOCATION_KEY_BYTES),
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
            TransactionUnit::Allocation(_) => TreeIdentifier(TREE_IDENTIFIER_ALLOCATION),
            TransactionUnit::AccountingTree => TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING),
            TransactionUnit::MappingTree => TreeIdentifier(TREE_IDENTIFIER_MAPPING),
            TransactionUnit::TreeTable => TreeIdentifier(TREE_IDENTIFIER_NONE),
        }
    }
}

// ───────────────────────── 分配记录树：按位置寻址的形状与编码（D8（核心索引结构） 已定项 14，第七节 B1/B3/B13） ─────────────────────────

/// 层级 L（0 = 叶）的一个节点罩多少槽：叶罩 W，往上每升一层乘一次扇出（D8 已定项 14「层级 L 的节点罩 W × 169^L 个槽」）。
fn allocation_record_tree_span_at_level(level: u32) -> u64 {
    ALLOCATION_RECORD_TREE_LEAF_SLOTS * ALLOCATION_RECORD_TREE_INTERNAL_FANOUT.pow(level)
}

/// 根层 R（D8 已定项 14）：取最小的 R ≥ 1，使 Σ_盘 ⌈盘上槽数 ÷ (W × 169^(R−1))⌉ ≤ 169。
/// `device_slots` 按 δ 甲读法传「设备字节 ÷ 16384」（`DEVICE_SLOTS`），第八节 G3 的 δ乙 变体在调用处另传单元区槽数。
fn allocation_record_tree_root_level(device_slots: &[u64]) -> u32 {
    let mut level = 1u32;
    loop {
        let span = allocation_record_tree_span_at_level(level - 1);
        let total: u64 = device_slots.iter().map(|&slots| slots.div_ceil(span)).sum();
        if total <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT {
            return level;
        }
        level += 1;
    }
}

/// 给定根层 R，一块盘在根节点里占几格（第七节 B3「root_cells_per_device」）。
fn allocation_record_tree_cells_per_device(slots: u64, root_level: u32) -> u64 {
    slots.div_ceil(allocation_record_tree_span_at_level(root_level - 1))
}

/// 第一个事务在给定几何上要写的分配记录树节点数（第七节 B13「first_txn_allocation_nodes」）：
/// 这个实验的全部记录都落在同一片叶（第一个事务的落点很少）⇒ 每盘一片叶、每盘每个中间层（层级 1..R−1）一个节点、一个根。
fn allocation_record_tree_first_transaction_node_count(device_count: u64, root_level: u32) -> u64 {
    device_count + device_count * u64::from(root_level - 1) + 1
}

/// 位置 key：与 `AllocationRecord::key_bytes()` 同一种编码（设备 4 + 槽号 6）——不是分配记录本身，
/// 是「这一段的起点」，供分配记录树内部 / 根节点的格 key 与 smallest_key / largest_key 共用。
fn allocation_position_key(device: u32, slot: u64) -> Vec<u8> {
    AllocationRecord { device: DeviceIdentity(device), slot: SlotNumber(slot), span_slots: 0, generation: CheckpointTxg(0), is_released: false }.key_bytes()
}

/// 分配记录树内部 / 根节点一条内部条目（D8 已定项 14 第 395 行，唯一写法：稀疏，96 字节）：
/// 只给有孩子的格建条目 = 10 字节位置 key（ι甲：这个孩子按位置规定罩的那一段的起点）+ `NodePointer` 86 字节；
/// 没有孩子的格不占条目（不再写 96 字节全 0 的空格）。
fn allocation_internal_cell(device: u32, slot: u64, pointer: NodePointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES as usize);
    writer.put(&allocation_position_key(device, slot));
    pointer.write_to(&mut writer);
    writer.bytes
}

/// 分配记录树内部 / 根节点一整层的格（层级 1 一个节点的 169 格，或根的 `cells_per_device * devices` 格）
/// 编成最终写进节点的条目序列（D8 已定项 14 第 395 行，唯一写法：稀疏，按 key 升序）。`slots` 是这一层全部
/// 位置，按位置升序给（位置升序本身就是 key 升序，ι甲：key = 位置起点），每项＝(设备, 这一格罩的起点位置,
/// 这一格的孩子指针（没有孩子就是 `None`，被这里过滤掉、不占条目）)。
fn assemble_allocation_cells(slots: &[(u32, u64, Option<NodePointer>)]) -> Vec<Vec<u8>> {
    slots.iter().filter_map(|&(device, position, child)| child.map(|pointer| allocation_internal_cell(device, position, pointer))).collect()
}

/// 分配记录树写出来的物理节点与它的根指针（这个装置的固定几何：两块盘，R 恒为 2）。
struct AllocationTreeBuild {
    /// 按 D3（空间分配） 已定项 10 ⑤ 的 bump 次序：每盘一片叶（设备升序）、每盘一个层级 1 节点（设备升序）、一个根。
    units: Vec<(SlotNumber, TransactionUnit, Vec<u8>)>,
    /// 根指针自己就带 `birth_sequence`（NodePointer 的字段），不用另存一份。
    root_pointer: NodePointer,
    leaf_pointers: Vec<(u32, NodePointer)>,
    internal_pointers: Vec<(u32, NodePointer)>,
}

/// 建出分配记录树的全部物理节点。这个装置的两条参数（两块 4 GiB 盘、一块 4 GiB 盘）根都在层级 2——
/// `assert_eq!` 钉住这条假设：变了要先看第五节 5.2 δ 那一格，不能悄悄按旧代码继续跑。
/// 每盘的记录全部落在同一片叶、落在这片叶所在的同一个层级 1 节点，也用 `assert_eq!` 钉住（第七节 B3）。
fn build_allocation_tree(parameters: &PoolParameters, records: &[AllocationRecord], txg: CheckpointTxg, instance: InstanceGeneration, sequences: &mut BirthSequenceAllocator) -> AllocationTreeBuild {
    let devices = parameters.devices();
    let device_slots: Vec<u64> = devices.iter().map(|_| DEVICE_SLOTS).collect();
    let root_level = allocation_record_tree_root_level(&device_slots);
    assert_eq!(root_level, 2, "这个装置的固定几何（4 GiB 盘）根恒在层级 2（第七节 B3）");
    let cells_per_device = allocation_record_tree_cells_per_device(DEVICE_SLOTS, root_level);
    let level_one_span = allocation_record_tree_span_at_level(root_level - 1);

    let mut units = Vec::new();
    let mut leaf_pointers = Vec::new();
    // 第一段：每盘一片叶（bump 次序：树内先叶后根、同层按 key 升序 ⇒ 设备升序）。
    for device in &devices {
        let device_records: Vec<AllocationRecord> = records.iter().filter(|record| record.device == *device).copied().collect();
        let leaf_indices: std::collections::BTreeSet<u64> = device_records.iter().map(|record| record.slot.0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS).collect();
        assert_eq!(leaf_indices.len(), 1, "这个装置的场景里一块盘的全部记录落在同一片叶（第七节 B3「leaves」）");
        let leaf_index = *leaf_indices.iter().next().expect("刚断言过恰好一个");
        let span_start = leaf_index * ALLOCATION_RECORD_TREE_LEAF_SLOTS;
        let smallest_key = allocation_position_key(device.0, span_start);
        let largest_key = allocation_position_key(device.0, span_start + ALLOCATION_RECORD_TREE_LEAF_SLOTS - 1);
        let mut sorted_records = device_records;
        sorted_records.sort_by_key(AllocationRecord::sort_key);
        let sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_ALLOCATION), txg, instance);
        let unit = build_index_node(
            TreeIdentifier(TREE_IDENTIFIER_ALLOCATION),
            0,
            ALLOCATION_KEY_BYTES,
            &smallest_key,
            &largest_key,
            txg,
            &parameters.fsid,
            instance,
            sequence,
            ALLOCATION_RECORD_BYTES as u16,
            &sorted_records.iter().map(AllocationRecord::to_bytes).collect::<Vec<_>>(),
        );
        let slot = SlotNumber(if device.0 == 0 { SLOT_ALLOCATION_LEAF_DEVICE_0 } else { SLOT_ALLOCATION_LEAF_DEVICE_1 });
        let pointer = NodePointer { head: PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_ALLOCATION), birth_txg: txg }, locations: parameters.location_entries(slot, &unit), instance, birth_sequence: sequence };
        units.push((slot, TransactionUnit::Allocation(AllocationNodeRole::Leaf { device: device.0 }), unit));
        leaf_pointers.push((device.0, pointer));
        let _ = leaf_index; // 供下一段算层级 1 的格号用（重新从 span_start 反推，省一个字段）
    }

    // 第二段：每盘一个层级 1 节点。
    let mut internal_pointers = Vec::new();
    let mut root_children: Vec<(u32, u64, NodePointer)> = Vec::new(); // (device, level_one_index, 指向层级 1 节点的指针)
    for (device, leaf_pointer) in &leaf_pointers {
        let device_records: Vec<&AllocationRecord> = records.iter().filter(|record| record.device.0 == *device).collect();
        let leaf_index = device_records[0].slot.0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS;
        assert!(device_records.iter().all(|record| record.slot.0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS == leaf_index), "同一盘的记录都在同一片叶");
        let level_one_index = leaf_index / ALLOCATION_RECORD_TREE_INTERNAL_FANOUT;
        let cell_within_level_one = leaf_index % ALLOCATION_RECORD_TREE_INTERNAL_FANOUT;
        // 5.2 α：这个层级 1 节点罩的 169 个位置，每个位置的起点槽 = 节点自己的起点 + 位置序号 × 叶宽。
        let level_one_slots: Vec<(u32, u64, Option<NodePointer>)> = (0..ALLOCATION_RECORD_TREE_INTERNAL_FANOUT)
            .map(|position| {
                let position_start = level_one_index * level_one_span + position * ALLOCATION_RECORD_TREE_LEAF_SLOTS;
                let child = if position == cell_within_level_one { Some(*leaf_pointer) } else { None };
                (*device, position_start, child)
            })
            .collect();
        let cells = assemble_allocation_cells(&level_one_slots);
        let smallest_key = allocation_position_key(*device, level_one_index * level_one_span);
        let largest_key = allocation_position_key(*device, level_one_index * level_one_span + level_one_span - 1);
        let sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_ALLOCATION), txg, instance);
        let unit = build_index_node(
            TreeIdentifier(TREE_IDENTIFIER_ALLOCATION),
            1,
            ALLOCATION_KEY_BYTES,
            &smallest_key,
            &largest_key,
            txg,
            &parameters.fsid,
            instance,
            sequence,
            ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES as u16,
            &cells,
        );
        let slot = SlotNumber(if *device == 0 { SLOT_ALLOCATION_INTERNAL_DEVICE_0 } else { SLOT_ALLOCATION_INTERNAL_DEVICE_1 });
        let pointer = NodePointer { head: PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_ALLOCATION), birth_txg: txg }, locations: parameters.location_entries(slot, &unit), instance, birth_sequence: sequence };
        units.push((slot, TransactionUnit::Allocation(AllocationNodeRole::Internal { device: *device, level: 1 }), unit));
        internal_pointers.push((*device, pointer));
        root_children.push((*device, level_one_index, internal_pointers.last().expect("刚 push 过").1));
    }

    // 第三段：根（单一节点，按盘分流，D8 已定项 14「根罩整个 key 空间、按盘分流」）。
    let root_children_by_position: std::collections::BTreeMap<(u32, u64), NodePointer> =
        root_children.iter().map(|(device, level_one_index, pointer)| ((*device, *level_one_index), *pointer)).collect();
    let mut root_slots: Vec<(u32, u64, Option<NodePointer>)> = Vec::new();
    for device in &devices {
        for level_one_index in 0..cells_per_device {
            let position_start = level_one_index * level_one_span;
            let child = root_children_by_position.get(&(device.0, level_one_index)).copied();
            root_slots.push((device.0, position_start, child));
        }
    }
    let root_cells = assemble_allocation_cells(&root_slots);
    // D8 已定项 14 第 395 行（唯一写法）：根罩整个 key 空间，smallest_key = (0, 0)，largest_key = (0xFFFFFFFF, 2^48 − 1)——
    // 与盘数、盘大小无关（第七节 A6）。
    let root_smallest_key = allocation_position_key(0, 0);
    let root_largest_key = allocation_position_key(u32::MAX, (1u64 << 48) - 1);
    let root_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_ALLOCATION), txg, instance);
    let root_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_ALLOCATION),
        u8::try_from(root_level).expect("根层级 1 字节装得下"),
        ALLOCATION_KEY_BYTES,
        &root_smallest_key,
        &root_largest_key,
        txg,
        &parameters.fsid,
        instance,
        root_sequence,
        ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES as u16,
        &root_cells,
    );
    let root_slot = SlotNumber(SLOT_ALLOCATION_ROOT);
    let root_pointer = NodePointer { head: PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_ALLOCATION), birth_txg: txg }, locations: parameters.location_entries(root_slot, &root_unit), instance, birth_sequence: root_sequence };
    units.push((root_slot, TransactionUnit::Allocation(AllocationNodeRole::Root), root_unit));

    AllocationTreeBuild { units, root_pointer, leaf_pointers, internal_pointers }
}

// ───────────────────────── extent 树上段叶（D8（核心索引结构） 已定项 14，ζ：根就是上段叶） ─────────────────────────

/// 标签字节（D8 已定项 14「extent 树」段）：0 全零占位（只有读者认、写者不写）、1 = 下段根指针、
/// 2 = 那一个数据单元的数据指针（内联）。第一个事务只有一个数据单元的文件 ⇒ 只写得到标签 2。
const EXTENT_UPPER_LEAF_ENTRY_TAG_NO_DATA_UNIT: u8 = 0;
const EXTENT_UPPER_LEAF_ENTRY_TAG_LOWER_SEGMENT_ROOT: u8 = 1;
const EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT: u8 = 2;

/// extent 树上段叶条目 113（D8 已定项 14）：key 24 `(locality 0, inode, 第三分量)` + 标签 1 + 载荷 88。
/// 第三分量按 5.2 η 甲恒 0（key 前 24 字节与旧的 `build_extent_record` 同一种布局，只是第三分量不再叫「offset」）。
fn build_extent_upper_leaf_entry(inode: u64, pointer: DataPointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES as usize);
    writer.put_u64(0);
    writer.put_u64(inode);
    writer.put_u64(0);
    writer.put_u8(EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT);
    pointer.write_to(&mut writer);
    writer.assert_position(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES, "extent 上段叶条目");
    writer.bytes
}

fn parse_extent_upper_leaf_entry(bytes: &[u8]) -> ([u8; 24], u8, DataPointer) {
    let mut reader = ByteReader::new(bytes);
    let key: [u8; 24] = reader.take(24).try_into().expect("切了 24 字节");
    let tag = reader.get_u8();
    (key, tag, DataPointer::read_from(&mut reader))
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

/// E142 第十四次跑（第三节 3.2 第 3399–3425 行）：给一个盘上绝对偏移，判它落在 w1 / w4 / t9 哪一条记录、记录内偏移是多少。
/// 三条记录都在 mkfs 之后紧接的三个 journal 计数器（1、2、3）上，与几何点无关，可以直接算，不用查 StructureCatalog。
fn journal_record_name_for_offset(offset: u64) -> Option<(&'static str, u64)> {
    for (label, counter) in [("w1", 1u64), ("w4", 2u64), ("t9", 3u64)] {
        let base = journal_record_offset(JournalCounter(counter)).0;
        if offset >= base && offset < base + JOURNAL_RECORD_BYTES {
            return Some((label, offset - base));
        }
    }
    None
}

/// H/F 相减、P3 全盘相减两处共用的字段标签（记录内偏移 → 字段名，第七节 A2 的偏移表）。
fn journal_header_field_tag(offset_in_record: u64) -> &'static str {
    match offset_in_record {
        JOURNAL_RECORD_FLAGS_OFFSET_U64 => "record_flags",
        46..=77 => "header_checksum",
        87..=90 => "ordinal_within_publish",
        91..=94 => "back_chain",
        _ => "other",
    }
}
const JOURNAL_RECORD_FLAGS_OFFSET_U64: u64 = JOURNAL_RECORD_FLAGS_OFFSET as u64;

/// 发布之后那次系统配置槽写的世代号与落点（D22 已定项 16）：取号那次是 2，之后每次发布 +1，槽 = 世代号 mod 2。
fn system_configuration_write_for_publish(checkpoint_txg: CheckpointTxg) -> (u64, usize) {
    let generation = checkpoint_txg.0 + SYSTEM_CONFIGURATION_GENERATION_AT_INSTANCE_ACQUISITION;
    (generation, (generation % 2) as usize)
}

/// 暖机（D16 已定项 8）：每次空发布照 D16 已定项 7 的顺序——屏障 → 空记录 → 屏障 → 根槽 FUA → 系统配置槽轮换；
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
            // 这次发布只有这一条记录（暖机的空发布不切分）：本次发布内序号恒 1（已定项 4），
            // 标志按写者开关写（臂 F 0x01、臂 H 0x00，E142 第十四次跑第五节 5.1）。
            ordinal_within_publish: 1,
            record_flags: parameters.last_of_publish_flag,
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
            // C512 2026-09-23 用户定案：零单元发布照抄上一版。
            allocation_record_tree_root: genesis.root.allocation_record_tree_root,
        };
        let (region, ring_slot) = ring_target_for_publish(txg);
        pool.write(parameters.region_devices[region as usize], ring_slot_offset(region, ring_slot), &root.to_slot(), StepKind::RootRecordFua);
        let (slot_generation, slot_index) = system_configuration_write_for_publish(txg);
        for device in parameters.devices() {
            let system_configuration = SystemConfiguration {
                fsid: parameters.fsid,
                this_device: device,
                device_count: u32::try_from(parameters.device_count).expect("设备数"),
                slot_generation,
                region_devices: parameters.region_devices,
                journal_tail: txg_number,
                journal_instance: instance,
            };
            pool.write(device, DeviceOffset(SYSTEM_CONFIGURATION_SLOT_OFFSETS[slot_index]), &system_configuration.to_slot(), StepKind::SystemConfigurationSlot);
        }
        roots.push(root);
    }
    (roots, previous_record_bytes)
}

#[allow(clippy::too_many_arguments, reason = "字段表就是这么多段，收成结构体只会多一层没人验的名字（build_index_node 同一处理由）")]
fn publish_first_file(
    pool: &mut RecordingPool,
    parameters: &PoolParameters,
    genesis: &MkfsOutput,
    file_bytes: &[u8],
    instance: InstanceGeneration,
    previous_record_bytes: Option<&[u8]>,
    change_count: u64,
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
    // E142 第十一次跑（2026-09-18）：改动计数从字面 1 改成调用方传入的 change_count（D8 已定项 6 偏移 88 的字段定义 = 最后一次改动所在发布的
    // checkpoint_txg；主口径传 FIRST_TRANSACTION_TXG，`crates/` 今天同口径——重跑登记 e142-r11-prereg.md 第三节）。
    let inode_record = InodeRecord { inode: FIRST_INODE_NUMBER, object_birth: txg, size: file_bytes.len() as u64, change_count };
    let inode_leaf_unit = build_packed_unit(inode_leaf_identity, INODE_RECORD_BYTES as u16, &[inode_record.to_bytes()], txg, fsid, write_order, inode_leaf_sequence);
    let inode_leaf_pointer = NodePointer {
        head: PointerHead { birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE), birth_txg: txg },
        locations: parameters.location_entries(SlotNumber(SLOT_INODE_LEAF), &inode_leaf_unit),
        instance,
        birth_sequence: inode_leaf_sequence,
    };

    // t2 extent 树上段叶（根即叶，5.2 ζ：只有一个 inode ⇒ 最低的、罩得住最大 inode 号的层级是 0）。
    // key 区间照 D18 已定项 2 射程写这片叶按位置罩的那一段（D8 已定项 14 第 401 行，唯一写法），
    // 不写叶里第一条与最后一条条目的 key：第 k 片（k = inode ÷ 143）罩 `[143k, 143k + 142]`，闭区间，第三分量 2^64 − 1。
    let extent_leaf_index = extent_upper_leaf_index(FIRST_INODE_NUMBER);
    let (extent_smallest_key, extent_largest_key) = extent_upper_leaf_positional_key_range(extent_leaf_index);
    let extent_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_EXTENT), txg, instance);
    let extent_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_EXTENT),
        0,
        24,
        &extent_smallest_key,
        &extent_largest_key,
        txg,
        fsid,
        instance,
        extent_sequence,
        EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES as u16,
        &[build_extent_upper_leaf_entry(FIRST_INODE_NUMBER, data_pointer)],
    );
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

    // t5..t9 分配记录树（按位置寻址，D8 已定项 14）：mkfs 的 m1 分配代 0；m2（第 0 版树表）被这次发布重写树表换下
    // ⇒ 记录改写成已释放、释放代 = 这次的 txg（D3 已定项 7，里程碑「第二个事务」步 5 逼出：不释放它，txg 0 的根离开候选集
    // 之后这一槽永远占着、I-3.1 红）；其余就是这次发布的 txg（「分配代」）；每盘各一条——含分配记录树自己的 5 个新节点
    // （D8 已定项 14 射程：这几个节点也占落点、也要记分配记录，不是只有旧的八个单元）。
    // deadlist 树 day-1 注册但没有根节点 ⇒ 不占落点、不写分配记录（字节表零那一节的 t5 那一行）。
    // 分配记录树两片叶 / 两个层级 1 节点只在两块盘时都建（build_allocation_tree 只给 `parameters.devices()`
    // 里真实存在的盘各建一份）——一块盘的对照臂（第七节 B10）没有「盘 1」那两个节点，`allocated` 跟着它走，
    // 不能不看 device_count 就把两块盘的槽都算成「有条分配记录」（那样会在没写过的槽上凭空记一条记录）。
    let mut allocated: Vec<(u64, u16, u64, bool)> = vec![
        (SLOT_INSTANCE_TABLE, 2, 0, false),
        (SLOT_TREE_TABLE_GENESIS, 1, FIRST_TRANSACTION_TXG, true),
        (SLOT_DATA_UNIT, 2, FIRST_TRANSACTION_TXG, false),
        (SLOT_EXTENT_ROOT, 1, FIRST_TRANSACTION_TXG, false),
        (SLOT_INODE_LEAF, 2, FIRST_TRANSACTION_TXG, false),
        (SLOT_INODE_ROOT, 1, FIRST_TRANSACTION_TXG, false),
        (SLOT_ALLOCATION_LEAF_DEVICE_0, 1, FIRST_TRANSACTION_TXG, false),
    ];
    if parameters.device_count > 1 {
        allocated.push((SLOT_ALLOCATION_LEAF_DEVICE_1, 1, FIRST_TRANSACTION_TXG, false));
    }
    allocated.push((SLOT_ALLOCATION_INTERNAL_DEVICE_0, 1, FIRST_TRANSACTION_TXG, false));
    if parameters.device_count > 1 {
        allocated.push((SLOT_ALLOCATION_INTERNAL_DEVICE_1, 1, FIRST_TRANSACTION_TXG, false));
    }
    allocated.push((SLOT_ALLOCATION_ROOT, 1, FIRST_TRANSACTION_TXG, false));
    allocated.push((SLOT_ACCOUNTING_ROOT, 1, FIRST_TRANSACTION_TXG, false));
    allocated.push((SLOT_MAPPING_ROOT, 1, FIRST_TRANSACTION_TXG, false));
    allocated.push((SLOT_TREE_TABLE_FIRST_PUBLISH, 1, FIRST_TRANSACTION_TXG, false));
    let mut allocation_records: Vec<AllocationRecord> = parameters
        .devices()
        .iter()
        .flat_map(|device| allocated.iter().map(move |(slot, span, generation, is_released)| AllocationRecord { device: *device, slot: SlotNumber(*slot), span_slots: *span, generation: CheckpointTxg(*generation), is_released: *is_released }))
        .collect();
    allocation_records.sort_by_key(AllocationRecord::sort_key);
    let allocation_tree = build_allocation_tree(parameters, &allocation_records, txg, instance, &mut sequences);
    // 分配记录树的物理节点数与门槛看齐（第七节 B13「first_txn_allocation_nodes」）；变了要先看 build_allocation_tree 的假设。
    assert_eq!(
        allocation_tree.units.len() as u64,
        allocation_record_tree_first_transaction_node_count(u64::try_from(parameters.device_count).expect("设备数"), 2),
        "分配记录树的物理节点数与形状函数算出来的不一致"
    );
    index_node_header_widths.push(("allocation", index_node_header_bytes(ALLOCATION_KEY_BYTES)));

    // t6 记账树（D5 已定项 8，2026-09-14 用户定案）：两盘时 **15 行**——
    // 带设备维的六项各两行（已分配 1、空闲 2、不可回收 3、defer 待释放 5、碎片度 runs 10、全空聚簇段数 11），
    // 池级三行（待删占用 4、已承诺预留 6、inode 号水位 12）。准入不等式要的四项 day-1 各写一行 0，
    // 「空 ≠ 0」那条防线因此保住：读不到行是坏账，读到 0 是真的 0。seq 一律 1（D8 已定项 10：直落叶）。
    let allocated_slots: u64 = allocated.iter().map(|(_, span, _, _)| u64::from(*span)).sum();
    let occupied: std::collections::BTreeSet<u64> = allocated.iter().flat_map(|(slot, span, _, _)| (0..u64::from(*span)).map(move |offset| slot + offset)).collect();
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
        // defer 待释放 = mkfs 那片第 0 版树表单元（1 槽）：A 重写树表把它换下（D3 已定项 7）。
        accounting_entries.push(per_device(STATISTIC_DEFER_QUEUE_BYTES, SLOT_BYTES));
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

    // 五棵树的根指针（t11 映射与 t12 树表都要引它们）
    let node_pointer = |tree: TreeIdentifier, slot: u64, unit: &[u8], sequence: BirthSequence| NodePointer {
        head: PointerHead { birth_tree: tree, birth_txg: txg },
        locations: parameters.location_entries(SlotNumber(slot), unit),
        instance,
        birth_sequence: sequence,
    };
    let extent_pointer = node_pointer(TreeIdentifier(TREE_IDENTIFIER_EXTENT), SLOT_EXTENT_ROOT, &extent_unit, extent_sequence);
    let inode_root_pointer = node_pointer(TreeIdentifier(TREE_IDENTIFIER_INODE), SLOT_INODE_ROOT, &inode_root_unit, inode_root_sequence);
    let allocation_pointer = allocation_tree.root_pointer;
    let accounting_pointer = node_pointer(TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING), SLOT_ACCOUNTING_ROOT, &accounting_unit, accounting_sequence);

    // t11 中央映射树：码 1 一条（数据单元）+ 码 2 / 码 3 九条（D19 已定项 8；映射树自己、树表、实例表豁免）——
    // 分配记录树自己的 5 个新节点也要各留一条映射条目，第七节 B7「mapping_entries=10」。
    let mut mapping_entries: Vec<(Vec<u8>, [LocationEntry; 2])> = vec![
        (mapping_key_for_data(data_pointer.head, data_pointer.write_order), data_pointer.locations),
        (mapping_key_for_node(UNIT_CLASS_INDEX_NODE, extent_pointer), extent_pointer.locations),
        (mapping_key_for_node(UNIT_CLASS_PACKED, inode_leaf_pointer), inode_leaf_pointer.locations),
        (mapping_key_for_node(UNIT_CLASS_INDEX_NODE, inode_root_pointer), inode_root_pointer.locations),
        (mapping_key_for_node(UNIT_CLASS_INDEX_NODE, accounting_pointer), accounting_pointer.locations),
    ];
    for (_, pointer) in allocation_tree.leaf_pointers.iter().chain(allocation_tree.internal_pointers.iter()) {
        mapping_entries.push((mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer), pointer.locations));
    }
    mapping_entries.push((mapping_key_for_node(UNIT_CLASS_INDEX_NODE, allocation_pointer), allocation_pointer.locations));
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
    // t5..t9：分配记录树的五个物理节点，bump 次序已经是「树内先叶后根、同层按设备升序」（build_allocation_tree）。
    units.extend(allocation_tree.units);
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

    // 第二段：journal 记录，点名 t1..t12 每个两盘；反向链照 D23 已定项 19 ② 罩前一条的整个头（header_csum 按零参与）。
    // 点名项的 key 尾段与 `units` 一一对应，次序就是 t1..t12（第五节 5.2 θ：点名照 `units` 的先后）；
    // unit_class 与 TransactionUnit 自报的那一个对账。
    let named_identities: Vec<(u8, TreeIdentifier, [u8; 10])> = {
        let mut list = vec![
            (UNIT_CLASS_DATA, data_pointer.head.birth_tree, data_key_tail(data_pointer.write_order)),
            (UNIT_CLASS_INDEX_NODE, extent_pointer.head.birth_tree, node_key_tail(extent_pointer.instance, extent_pointer.birth_sequence)),
            (UNIT_CLASS_PACKED, inode_leaf_pointer.head.birth_tree, node_key_tail(inode_leaf_pointer.instance, inode_leaf_pointer.birth_sequence)),
            (UNIT_CLASS_INDEX_NODE, inode_root_pointer.head.birth_tree, node_key_tail(inode_root_pointer.instance, inode_root_pointer.birth_sequence)),
        ];
        for (_, pointer) in allocation_tree.leaf_pointers.iter().chain(allocation_tree.internal_pointers.iter()) {
            list.push((UNIT_CLASS_INDEX_NODE, pointer.head.birth_tree, node_key_tail(pointer.instance, pointer.birth_sequence)));
        }
        list.push((UNIT_CLASS_INDEX_NODE, allocation_pointer.head.birth_tree, node_key_tail(allocation_pointer.instance, allocation_pointer.birth_sequence)));
        list.push((UNIT_CLASS_INDEX_NODE, accounting_pointer.head.birth_tree, node_key_tail(accounting_pointer.instance, accounting_pointer.birth_sequence)));
        list.push((UNIT_CLASS_INDEX_NODE, mapping_pointer.head.birth_tree, node_key_tail(mapping_pointer.instance, mapping_pointer.birth_sequence)));
        list.push((UNIT_CLASS_INDEX_NODE, tree_table_pointer.head.birth_tree, node_key_tail(tree_table_pointer.instance, tree_table_pointer.birth_sequence)));
        list
    };
    assert_eq!(named_identities.len(), units.len(), "点名项与写清单的单元数要一一对应");
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
        ordinal_within_publish: 1,
        record_flags: parameters.last_of_publish_flag,
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
        // C512 2026-09-23 用户定案：带文件的一版写全零（不是「树表 0 条、写过行」那一格）。
        allocation_record_tree_root: NodePointer::empty_root(),
    };
    let (region, ring_slot) = ring_target_for_publish(txg);
    pool.write(parameters.region_devices[region as usize], ring_slot_offset(region, ring_slot), &root.to_slot(), StepKind::RootRecordFua);

    // 根槽之后：系统配置槽轮换，世代号 5、落槽 1（D22 已定项 16），tail 前移到 jsn 3（D16 已定项 7 的系统配置注）。
    let (slot_generation, slot_index) = system_configuration_write_for_publish(txg);
    for device in parameters.devices() {
        let system_configuration = SystemConfiguration {
            fsid: parameters.fsid,
            this_device: device,
            device_count: u32::try_from(parameters.device_count).expect("设备数"),
            slot_generation,
            region_devices: parameters.region_devices,
            journal_tail: FIRST_TRANSACTION_TXG,
            journal_instance: instance,
        };
        pool.write(device, DeviceOffset(SYSTEM_CONFIGURATION_SLOT_OFFSETS[slot_index]), &system_configuration.to_slot(), StepKind::SystemConfigurationSlot);
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ReaderMode {
    /// 读者甲：第九、十次跑起的装置规则（`replay_journal`）：锚点按 (实例, txg) 与所选根相同的那条认；
    /// 无锚点时只认 txg = 根 txg + 1；不带提交标记即停；一条一条施加。新字段（标志、序号）照读、不解释。
    Primary,
    /// 读者乙：照 D23 已定项 4 / 17 / 14 第六条 / 已定项 14「这一版的失败处置」字面写（`replay_journal_clause`），
    /// 与读者甲各自独立实现、不共用代码（E142 第十四次跑第五节 5.1）——这是这一版真实基线的读法。
    Clause,
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

/// 读者乙 ①–④ 四类分支各走到几次（第五节 5.1、第八节 S5）。合法历史（未改写的层 0 状态）里全部恒 0——
/// 这几格只有 P6 的改写镜像才走得到（已定项 4 依据）。
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
struct ReaderClauseBranchCounts {
    /// ①标志字节位 0 之外有值 ⇒ 这条记录当损坏。
    corrupted_flag: u64,
    /// ①本次发布内序号为 0 ⇒ 这条记录当损坏。
    corrupted_ordinal_zero: u64,
    /// ②同一 (实例, txg) 带标志的记录多于一条 ⇒ 停下（已定项 14「这一版的失败处置」）。
    multiple_last_flag: u64,
    /// ③一次发布之内序号不是上一条 + 1 ⇒ 断。
    ordinal_gap: u64,
    /// ③锚点读得出 / 无锚点起链时，下一次发布首条序号不是 1 ⇒ 断在这一条。
    ordinal_not_one_at_publish_start: u64,
    /// ④同一 (实例, txg) 带标志的那条之后还有记录 ⇒ 断在带标志的那条、这次发布不施加。
    trailing_after_last_flag: u64,
}

impl ReaderClauseBranchCounts {
    /// Q142.9（够判后未跑）：`main()` 里唯一调它的 `name=header311_reader_branch_counts` 那一行这一段
    /// 跳过了层 0 主臂枚举，`total()` 暂时没有调用点；第二段要跑那条枚举时会重新用上，不删。
    #[allow(dead_code, reason = "Q142.9 附带，够判后未跑；第二段跑层 0 主臂枚举时恢复调用")]
    fn total(&self) -> u64 {
        self.corrupted_flag + self.corrupted_ordinal_zero + self.multiple_last_flag + self.ordinal_gap + self.ordinal_not_one_at_publish_start + self.trailing_after_last_flag
    }
    fn add(&mut self, other: &Self) {
        self.corrupted_flag += other.corrupted_flag;
        self.corrupted_ordinal_zero += other.corrupted_ordinal_zero;
        self.multiple_last_flag += other.multiple_last_flag;
        self.ordinal_gap += other.ordinal_gap;
        self.ordinal_not_one_at_publish_start += other.ordinal_not_one_at_publish_start;
        self.trailing_after_last_flag += other.trailing_after_last_flag;
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct RecoveryReport {
    outcome: RecoveryOutcome,
    journal: JournalScanReport,
    mapping_fallbacks: usize,
    /// 只在 `ReaderMode::Clause` 下非零；`Primary` 与 `Ignore` 恒是默认值（全 0、`fatal=false`）。
    clause_branches: ReaderClauseBranchCounts,
    clause_fatal: bool,
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

fn choose_system_configuration(reader: &dyn BlockReader) -> Result<SystemConfiguration, String> {
    let mut chosen: Option<SystemConfiguration> = None;
    for device_index in 0..reader.device_count() {
        let device = DeviceIdentity(u32::try_from(device_index).expect("设备数"));
        let mut best_on_device: Option<SystemConfiguration> = None;
        for slot_offset in SYSTEM_CONFIGURATION_SLOT_OFFSETS {
            let bytes = reader.read(device, DeviceOffset(slot_offset), SYSTEM_CONFIGURATION_SLOT_BYTES as usize);
            if let Some(candidate) = SystemConfiguration::parse_slot(&bytes) {
                if best_on_device.as_ref().is_none_or(|best| candidate.slot_generation > best.slot_generation) {
                    best_on_device = Some(candidate);
                }
            }
        }
        let Some(best_on_device) = best_on_device else {
            return Err(format!("盘 {device_index} 两个系统配置槽都无效"));
        };
        match &chosen {
            None => chosen = Some(best_on_device),
            Some(previous) => {
                if previous.fsid != best_on_device.fsid || previous.device_count != best_on_device.device_count {
                    return Err("两盘的系统配置 fsid 或设备数对不上".to_string());
                }
            }
        }
    }
    chosen.ok_or_else(|| "池里没有设备".to_string())
}

fn choose_root(reader: &dyn BlockReader, system_configuration: &SystemConfiguration) -> Option<RootRecord> {
    let mut best: Option<RootRecord> = None;
    for region in 0..RING_REGIONS {
        let device = system_configuration.region_devices[region as usize];
        if device.0 as usize >= reader.device_count() {
            continue;
        }
        for slot in 0..RING_SLOTS_PER_REGION {
            let bytes = reader.read(device, ring_slot_offset(region, slot), PHYSICAL_BLOCK_BYTES as usize);
            if let Some(candidate) = RootRecord::parse_slot(&bytes, &system_configuration.fsid) {
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
    // 前缀规则不跨实例边界（D23 已定项 14 第 1 条）：只有所选根自己那个实例的记录是候选，链从所选根覆盖的最后一条之后接；
    // 所选根是 mkfs 的第 0 代根时一条都不施加。
    let mut above: Vec<&JournalRecord> = records.values().filter(|record| record.instance == root.instance && (record.instance, record.checkpoint_txg) > water).collect();
    above.sort_by_key(|record| (record.instance, record.counter));
    report.above_water = above.len();
    // 链首接在所选根自己那条记录（同实例、checkpoint_txg 相等）之后；那条读不出时链首只认 checkpoint_txg = 根 txg + 1 的那条
    // （一次发布一条记录、txg 每次加一；与 crates 同一条规则，里程碑「第二个事务」步 3 三方第一轮攻方腿打中「无锚点时无条件接上」）。
    let root_own_record_counter = records.values().find(|record| record.instance == root.instance && record.checkpoint_txg == root.checkpoint_txg).map(|record| record.counter.0);
    let mut expected: Option<(InstanceGeneration, JournalCounter)> = root_own_record_counter.map(|counter| (root.instance, JournalCounter(counter + 1)));
    let chain_start_txg_without_anchor = CheckpointTxg(root.checkpoint_txg.0 + 1);
    for record in above.into_iter().take(usize::try_from(JOURNAL_IN_FLIGHT_RECORD_LIMIT).expect("在飞上限")) {
        if let Some(expected_key) = expected {
            if (record.instance, record.counter) != expected_key {
                break;
            }
        } else if record.checkpoint_txg != chain_start_txg_without_anchor {
            break;
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
                // C512 2026-09-23 用户定案：journal 新根段不带这一项（宽度不变），施加记录时照抄被施加那条根的这一项。
                allocation_record_tree_root: rebuilt.allocation_record_tree_root,
            };
        } else {
            report.verification_failed += 1;
            break;
        }
    }
    (report, rebuilt)
}

/// 读者乙（E142 第十四次跑第五节 5.1）：D23 已定项 4 / 17 / 14 第六条 / 已定项 14「这一版的失败处置」字面写，
/// 独立于读者甲（`replay_journal`），不共用代码。E142 的写路径每次发布恰一条记录（G18）：`above` 在自然
/// （未改写的）层 0 状态里至多 1 条，下面的多记录批处理只服务 P6 的改写镜像。
/// 返回 (扫描报告, 重建出来的根, 分支计数, 是否触发「这一版的失败处置」那个停机)。
fn replay_journal_clause(
    reader: &dyn BlockReader,
    root: &RootRecord,
    records: &BTreeMap<(InstanceGeneration, JournalCounter), JournalRecord>,
) -> (JournalScanReport, RootRecord, ReaderClauseBranchCounts, bool) {
    let mut branch = ReaderClauseBranchCounts::default();
    let mut report = JournalScanReport { valid_records: records.len(), ..JournalScanReport::default() };
    let mut rebuilt = *root;

    // ① 逐条先滤掉「当损坏」的记录：标志字节位 0 之外有值，或本次发布内序号为 0（已定项 4）。
    // 这两条判据与 `readable_record_count`（P6「读得出的记录条数」）共用同一对函数：改坏其中一个，
    // 两处观测（这里的候选集合、那边报出来的可读条数）会一起变，不会只影响其中一处。
    let mut usable: Vec<&JournalRecord> = Vec::new();
    for record in records.values() {
        if record_flag_byte_is_corrupted(record) {
            branch.corrupted_flag += 1;
            continue;
        }
        if record_ordinal_is_corrupted(record) {
            branch.corrupted_ordinal_zero += 1;
            continue;
        }
        usable.push(record);
    }
    usable.sort_by_key(|record| (record.instance, record.counter));

    let water = (root.instance, root.checkpoint_txg);
    let above: Vec<&JournalRecord> = usable.iter().filter(|record| record.instance == root.instance && (record.instance, record.checkpoint_txg) > water).copied().collect();
    report.above_water = above.len();

    // ② 锚点 = 所选根自己那次发布（同实例、同 txg）里带标志的记录；带标志的多于一条 ⇒ 停下（已定项 14「这一版的失败处置」）。
    let same_publish_as_root: Vec<&JournalRecord> = usable.iter().filter(|record| record.instance == root.instance && record.checkpoint_txg == root.checkpoint_txg).copied().collect();
    let flagged_at_root: Vec<&JournalRecord> = same_publish_as_root.iter().filter(|record| record.record_flags & 0x01 == 1).copied().collect();
    if flagged_at_root.len() > 1 {
        branch.multiple_last_flag += 1;
        return (report, rebuilt, branch, true);
    }
    let mut expected: Option<(InstanceGeneration, JournalCounter)> = flagged_at_root.first().map(|record| (root.instance, JournalCounter(record.counter.0 + 1)));
    let chain_start_txg_without_anchor = CheckpointTxg(root.checkpoint_txg.0 + 1);

    let mut index = 0usize;
    let in_flight_limit = usize::try_from(JOURNAL_IN_FLIGHT_RECORD_LIMIT).expect("在飞上限");
    while index < above.len() && index < in_flight_limit {
        let record = above[index];
        if let Some(expected_key) = expected {
            if (record.instance, record.counter) != expected_key {
                break;
            }
        } else if record.checkpoint_txg != chain_start_txg_without_anchor {
            break;
        }
        // ③ 这次发布的第一条（无论是锚点接上的还是「无锚点」起链的）序号必须是 1，否则断在这一条。
        if record.ordinal_within_publish != 1 {
            branch.ordinal_not_one_at_publish_start += 1;
            break;
        }
        // 收集这一次发布的全部记录，直到带标志的那一条（序号连续、counter 紧接）；候选用尽还没见到标志 ⇒ 这次发布还没写完，链断在这里。
        let mut batch_end = index;
        loop {
            if batch_end > index {
                let previous = above[batch_end - 1];
                let current = above[batch_end];
                if (current.instance, current.counter) != (previous.instance, JournalCounter(previous.counter.0 + 1)) {
                    break;
                }
                if current.ordinal_within_publish != previous.ordinal_within_publish + 1 {
                    branch.ordinal_gap += 1;
                    break;
                }
            }
            if above[batch_end].record_flags & 0x01 == 1 {
                break;
            }
            if batch_end + 1 >= above.len() {
                batch_end += 1; // 越出候选集合，下面按「没见到标志」处理
                break;
            }
            batch_end += 1;
        }
        if batch_end >= above.len() || above[batch_end].record_flags & 0x01 != 1 {
            break;
        }
        let flagged_record = above[batch_end];
        // ④ 同一 (实例, txg) 带标志的那条之后还有记录 ⇒ 断在带标志的那条，这次发布不施加。
        if batch_end + 1 < above.len() && above[batch_end + 1].instance == flagged_record.instance && above[batch_end + 1].checkpoint_txg == flagged_record.checkpoint_txg {
            branch.trailing_after_last_flag += 1;
            break;
        }
        // ④ 走到带标志的那一条且带提交标记才整体施加。
        if !flagged_record.is_commit {
            break;
        }
        let mut publish_verified = true;
        for batch_record in &above[index..=batch_end] {
            let mut record_verified = true;
            for named in &batch_record.named {
                let Some(unit_bytes) = unit_bytes_for_class(named.unit_class) else {
                    record_verified = false;
                    continue;
                };
                for location in &named.locations {
                    if location.device.0 as usize >= reader.device_count() {
                        continue;
                    }
                    let bytes = reader.read(location.device, location.slot.device_offset(), usize::try_from(unit_bytes).expect("单元字节数"));
                    if castagnoli_crc32(&bytes) != location.unit_checksum {
                        record_verified = false;
                    }
                }
            }
            if record_verified {
                report.verification_passed += 1;
                report.prefix_applied += 1;
            } else {
                report.verification_failed += 1;
                publish_verified = false;
            }
        }
        if !publish_verified {
            break;
        }
        rebuilt = RootRecord {
            fsid: rebuilt.fsid,
            instance: flagged_record.instance,
            checkpoint_txg: flagged_record.checkpoint_txg,
            tree_table: flagged_record.new_tree_table,
            tree_identifier_watermark: flagged_record.new_tree_identifier_watermark,
            rollback_floor: flagged_record.new_rollback_floor,
            instance_table: rebuilt.instance_table,
            mapping_root: flagged_record.new_mapping_root,
            allocation_record_tree_root: rebuilt.allocation_record_tree_root,
        };
        index = batch_end + 1;
        expected = Some((flagged_record.instance, JournalCounter(flagged_record.counter.0 + 1)));
    }
    (report, rebuilt, branch, false)
}

struct TreeRoots {
    extent: IndexNodeHeader,
    inode: IndexNodeHeader,
    accounting: IndexNodeHeader,
    mapping: IndexNodeHeader,
}

/// 分配记录树递归下探到叶（D8 已定项 14，按位置寻址）：每个节点核「header 里的 key 区间 == 它按位置规定罩的
/// 那一段」（D18 已定项 2 射程），不核「首末条目 key」——稠密内部节点的最后一格常是空格，M111 的会红检查
/// 就是防止这里退回码 2 树那种「区间 = 首末条目 key」的检查（那个检查对稠密内部节点会错判）。
#[allow(clippy::too_many_arguments, reason = "位置寻址要传全部几何上下文，收成结构体只会多一层没人验的名字")]
fn read_allocation_records(reader: &dyn BlockReader, tree: TreeIdentifier, pointer: &NodePointer, root: &RootRecord, expected_fsid: u64, device: u32, span_start: u64, span_length: u64, level: u32) -> Result<Vec<Vec<u8>>, String> {
    let bytes = read_unit_via_locations(reader, &pointer.locations, NODE_BYTES as usize)?;
    let node = parse_index_node(&bytes).map_err(|error| format!("分配记录树节点 {error:?}"))?;
    if node.tree != tree {
        return Err(format!("分配记录树节点头里写的树 ID 是 {}", node.tree.0));
    }
    if node.key_width != ALLOCATION_KEY_BYTES {
        return Err(format!("分配记录树节点自述 key 宽 {}，要 {ALLOCATION_KEY_BYTES}", node.key_width));
    }
    if node.birth_txg > root.checkpoint_txg || node.instance > root.instance {
        return Err("分配记录树节点诞生于根之后".to_string());
    }
    if node.fsid != expected_fsid {
        return Err("分配记录树节点 fsid 不符".to_string());
    }
    if node.birth_sequence != pointer.birth_sequence {
        return Err("分配记录树节点出生序号与指针不符".to_string());
    }
    if u32::from(node.level) != level {
        return Err(format!("分配记录树节点自述层级 {} 与位置期待的 {level} 不符", node.level));
    }
    let expected_smallest = allocation_position_key(device, span_start);
    let expected_largest = allocation_position_key(device, span_start + span_length - 1);
    if node.smallest_key != expected_smallest || node.largest_key != expected_largest {
        return Err("分配记录树节点的 key 区间与它按位置规定罩的那一段不符".to_string()); // D18 已定项 2 射程
    }
    if level == 0 {
        return Ok(node.entries);
    }
    let child_span = allocation_record_tree_span_at_level(level - 1);
    let mut collected = Vec::new();
    let mut previous_key: Option<Vec<u8>> = None;
    for cell in &node.entries {
        let key = &cell[..ALLOCATION_KEY_BYTES];
        if let Some(previous) = &previous_key {
            if key <= previous.as_slice() {
                return Err("分配记录树内部条目的 key 没有严格递增（D8 已定项 14 第 395 行「按 key 排」）".to_string());
            }
        }
        previous_key = Some(key.to_vec());
        let (child_device, child_span_start, child_pointer) = allocation_internal_cell_child_position(cell);
        if child_device != device {
            return Err(format!("分配记录树内部条目 key 的设备 {child_device} 与节点自己的设备 {device} 不符"));
        }
        if child_span_start < span_start || child_span_start >= span_start + span_length || (child_span_start - span_start) % child_span != 0 {
            return Err("分配记录树内部条目 key 的槽号不是这个节点段内某个孩子位置的起点（ι甲）".to_string());
        }
        collected.extend(read_allocation_records(reader, tree, &child_pointer, root, expected_fsid, device, child_span_start, child_span, level - 1)?);
    }
    Ok(collected)
}

/// 分配记录树内部条目：把 key 解析成 (设备, 这个孩子按位置罩的那一段的起点)（ι甲，D8 已定项 14 第 395 行）
/// 与它的子指针，供内部节点与根共用的读路径解码一条条目。
fn allocation_internal_cell_child_position(cell: &[u8]) -> (u32, u64, NodePointer) {
    let mut reader = ByteReader::new(cell);
    let device = reader.get_u32();
    let slot = reader.get_six_byte_unsigned();
    let pointer = NodePointer::read_from(&mut reader);
    (device, slot, pointer)
}

/// 分配记录树的根（单一节点，按盘分流，D8 已定项 14）：读根、按 β 甲核它的 key 区间、再按格分流到各盘的子树。
fn read_allocation_tree(reader: &dyn BlockReader, tree: TreeIdentifier, pointer: &NodePointer, root: &RootRecord, expected_fsid: u64, device_count: usize) -> Result<Vec<Vec<u8>>, String> {
    let bytes = read_unit_via_locations(reader, &pointer.locations, NODE_BYTES as usize)?;
    let node = parse_index_node(&bytes).map_err(|error| format!("分配记录树根 {error:?}"))?;
    if node.tree != tree {
        return Err(format!("分配记录树根头里写的树 ID 是 {}", node.tree.0));
    }
    if node.key_width != ALLOCATION_KEY_BYTES {
        return Err(format!("分配记录树根自述 key 宽 {}，要 {ALLOCATION_KEY_BYTES}", node.key_width));
    }
    if node.birth_txg > root.checkpoint_txg || node.instance > root.instance {
        return Err("分配记录树根诞生于根之后".to_string());
    }
    if node.fsid != expected_fsid {
        return Err("分配记录树根 fsid 不符".to_string());
    }
    if node.birth_sequence != pointer.birth_sequence {
        return Err("分配记录树根出生序号与指针不符".to_string());
    }
    let root_level = u32::from(node.level);
    if root_level == 0 {
        return Ok(node.entries); // 极小几何：根本身就是叶（这个装置的固定几何走不到，留通用性）
    }
    let cells_per_device = allocation_record_tree_cells_per_device(DEVICE_SLOTS, root_level);
    let level_one_span = allocation_record_tree_span_at_level(root_level - 1);
    // D8 已定项 14 第 395 行（唯一写法）：根罩整个 key 空间，与盘数、盘大小无关（第七节 A6）。
    let expected_smallest = allocation_position_key(0, 0);
    let expected_largest = allocation_position_key(u32::MAX, (1u64 << 48) - 1);
    if node.smallest_key != expected_smallest || node.largest_key != expected_largest {
        return Err("分配记录树根 key 区间不是整个 key 空间（D8 已定项 14 第 395 行）".to_string());
    }
    let mut collected = Vec::new();
    let mut previous_key: Option<Vec<u8>> = None;
    for cell in &node.entries {
        let key = &cell[..ALLOCATION_KEY_BYTES];
        if let Some(previous) = &previous_key {
            if key <= previous.as_slice() {
                return Err("分配记录树根条目的 key 没有严格递增（D8 已定项 14 第 395 行「按 key 排」）".to_string());
            }
        }
        previous_key = Some(key.to_vec());
        let (device, child_span_start, child_pointer) = allocation_internal_cell_child_position(cell);
        if device as usize >= device_count {
            return Err(format!("分配记录树根条目 key 的设备号 {device} 越出设备数范围"));
        }
        if child_span_start % level_one_span != 0 || child_span_start / level_one_span >= cells_per_device {
            return Err("分配记录树根条目 key 的槽号不是某个孩子位置的起点（ι甲）".to_string());
        }
        collected.extend(read_allocation_records(reader, tree, &child_pointer, root, expected_fsid, device, child_span_start, level_one_span, root_level - 1)?);
    }
    Ok(collected)
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

/// extent 树上段叶（层级 0，5.2 ζ：这个装置的几何只有一个 inode ⇒ 根就是叶）：D18 已定项 2 射程——
/// 按位置核 key 区间，不核「首末条目 key」（那是 `read_tree_root` 给码 2 btree 树用的通用检查）。
fn read_extent_upper_leaf(reader: &dyn BlockReader, tree: TreeIdentifier, pointer: &NodePointer, root: &RootRecord, expected_fsid: u64) -> Result<IndexNodeHeader, String> {
    let bytes = read_unit_via_locations(reader, &pointer.locations, NODE_BYTES as usize)?;
    let node = parse_index_node(&bytes).map_err(|error| format!("extent 树上段叶 {error:?}"))?;
    if node.tree != tree {
        return Err(format!("extent 树上段叶头里写的树 ID 是 {}", node.tree.0));
    }
    if node.key_width != 24 {
        return Err(format!("extent 树上段叶自述 key 宽 {}，要 24", node.key_width));
    }
    if node.birth_txg > root.checkpoint_txg || node.instance > root.instance {
        return Err("extent 树上段叶诞生于根之后".to_string());
    }
    if node.fsid != expected_fsid {
        return Err("extent 树上段叶 fsid 不符".to_string());
    }
    if node.birth_sequence != pointer.birth_sequence {
        return Err("extent 树上段叶出生序号与指针不符".to_string());
    }
    if node.level != 0 {
        return Err("extent 树这个装置的几何只有一个 inode，上段的根恒是层级 0 的叶（5.2 ζ）".to_string());
    }
    let first_entry = node.entries.first().ok_or_else(|| "extent 树上段叶没有条目".to_string())?;
    let inode = u64::from_le_bytes(first_entry[8..16].try_into().expect("key 前 24 字节里第二段是 inode 号"));
    let k = extent_upper_leaf_index(inode);
    let (expected_smallest, expected_largest) = extent_upper_leaf_positional_key_range(k);
    if node.smallest_key != expected_smallest || node.largest_key != expected_largest {
        return Err("extent 树上段叶 key 区间与它按位置规定罩的那一段不符".to_string()); // D18 已定项 2 射程
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
    let mut allocation_records: Option<Vec<Vec<u8>>> = None;
    let mut extent_root: Option<IndexNodeHeader> = None;
    for entry in &entries {
        if entry.tree.0 >= root.tree_identifier_watermark {
            return Err("树 ID 不低于水位".to_string()); // I-7.8
        }
        if entry.root.is_empty_root() {
            continue; // day-1 注册、还没有根的树（livelist、稀疏旁表）：没有单元可读
        }
        if entry.kind == TREE_KIND_ALLOCATION {
            // 按位置寻址（D8 已定项 14），不走通用的 `read_tree_root`：根不是唯一节点，要递归下探到叶。
            allocation_records = Some(read_allocation_tree(reader, entry.tree, &entry.root, root, expected_fsid, reader.device_count())?);
            continue;
        }
        if entry.kind == TREE_KIND_EXTENT {
            // 按位置寻址（D8 已定项 14），核区间不核首末条目 key（D18 已定项 2 射程）。
            extent_root = Some(read_extent_upper_leaf(reader, entry.tree, &entry.root, root, expected_fsid)?);
            continue;
        }
        by_kind.insert(entry.kind, read_tree_root(reader, entry.kind, entry.tree, &entry.root, root, expected_fsid)?);
    }
    let mut take = |kind: u16, name: &str| by_kind.remove(&kind).ok_or_else(|| format!("树表里没有{name}"));
    let roots = TreeRoots {
        extent: extent_root.ok_or_else(|| "树表里没有 extent 树".to_string())?,
        inode: take(TREE_KIND_INODE, "inode 树")?,
        accounting: take(TREE_KIND_ACCOUNTING, "记账树")?,
        // 中央映射树的根住根记录，不进树表（D19 已定项 11）。
        mapping: read_tree_root(reader, TREE_KIND_MAPPING, TreeIdentifier(TREE_IDENTIFIER_MAPPING), &root.mapping_root, root, expected_fsid)?,
    };
    let allocation_records = allocation_records.ok_or_else(|| "树表里没有分配记录树".to_string())?;
    // 每盘 (10 + 2 × 盘数) 条：6 个既有单元 + 分配记录树自己 (2 × 盘数 + 1) 个节点 + 记账 / 映射 / 树表
    // （两块盘时 10 + 4 = 14，一块盘时 10 + 2 = 12，第七节 B4、B10）。
    let expected_allocation_records = (10 + 2 * reader.device_count()) * reader.device_count();
    if allocation_records.len() != expected_allocation_records {
        return Err(format!("分配记录数 {} 不是 (10 + 2 × 盘数) × 盘数 = {expected_allocation_records}", allocation_records.len()));
    }
    // D5 已定项 8（2026-09-14 用户定案）：池级三行（待删占用、已承诺预留、inode 号水位）+ 每盘六行
    // （已分配字节、空闲字节、不可回收、defer 待释放、碎片度 runs、全空聚簇段数）⇒ 两盘 15 行。
    if roots.accounting.entries.len() != 3 + 6 * reader.device_count() {
        return Err(format!("记账条目数 {} 不是 3 + 6 × 盘数", roots.accounting.entries.len()));
    }
    // 6 个既有单元（数据、extent、inode 叶、inode 根、分配记录树根、记账）+ 分配记录树每盘一对（叶 + 层级 1 节点）；
    // 两块盘时 6 + 4 = 10（第七节 B7），一块盘时 6 + 2 = 8（第七节 B10 的对照臂）。
    let expected_mapping_entries = 6 + 2 * reader.device_count();
    if roots.mapping.entries.len() != expected_mapping_entries {
        return Err(format!("映射条目数 {} 不是 {expected_mapping_entries}", roots.mapping.entries.len()));
    }
    for record_bytes in &allocation_records {
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

    // extent 树：上段叶按 (locality 0, inode 1, 第三分量 0) 找条目（5.2 ζ：根就是上段叶）；标签 2 才是内联数据指针，
    // 解引用先按位置提示、校验和不对再查映射（D19 已定项 5）。
    let mut wanted_key = [0u8; 24];
    wanted_key[8..16].copy_from_slice(&FIRST_INODE_NUMBER.to_le_bytes());
    let mut data_unit: Option<(DataPointer, Vec<u8>)> = None;
    for record_bytes in &roots.extent.entries {
        let (key, tag, pointer) = parse_extent_upper_leaf_entry(record_bytes);
        if key != wanted_key {
            continue;
        }
        if tag != EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT {
            return Err(format!(
                "extent 上段叶条目标签 {tag} 不是内联数据指针（标签 {EXTENT_UPPER_LEAF_ENTRY_TAG_NO_DATA_UNIT} 没有单元、\
                 标签 {EXTENT_UPPER_LEAF_ENTRY_TAG_LOWER_SEGMENT_ROOT} 要走下段，这个装置的第一个事务走不到）"
            ));
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
fn recover(reader: &dyn BlockReader, policy: JournalPolicy, mode: ReaderMode) -> RecoveryReport {
    let mut mapping_fallbacks = 0;
    let system_configuration = match choose_system_configuration(reader) {
        Ok(system_configuration) => system_configuration,
        Err(reason) => {
            return RecoveryReport { outcome: RecoveryOutcome::Failed { root: None, reason }, journal: JournalScanReport::default(), mapping_fallbacks, clause_branches: ReaderClauseBranchCounts::default(), clause_fatal: false };
        }
    };
    let Some(root) = choose_root(reader, &system_configuration) else {
        return RecoveryReport {
            outcome: RecoveryOutcome::Failed { root: None, reason: "根环里一条合法根都没有".to_string() },
            journal: JournalScanReport::default(),
            mapping_fallbacks,
            clause_branches: ReaderClauseBranchCounts::default(),
            clause_fatal: false,
        };
    };
    // 报出去的 `root=` 恒是**所选**的那条根；施加记录之后走的是重建出来的根（D23 已定项 15）。
    let root_key = (root.instance, root.checkpoint_txg);
    let mut clause_branches = ReaderClauseBranchCounts::default();
    let mut clause_fatal = false;
    let (journal, effective_root) = match policy {
        JournalPolicy::Consult => match mode {
            ReaderMode::Primary => replay_journal(reader, &root, &scan_journal(reader, unit_fsid(&system_configuration.fsid))),
            ReaderMode::Clause => {
                let (journal, effective_root, branches, fatal) = replay_journal_clause(reader, &root, &scan_journal(reader, unit_fsid(&system_configuration.fsid)));
                clause_branches = branches;
                clause_fatal = fatal;
                (journal, effective_root)
            }
        },
        JournalPolicy::Ignore => (JournalScanReport::default(), root),
    };
    if clause_fatal {
        // 已定项 14「这一版的失败处置」：同一 (实例代号, checkpoint_txg) 带末条标志的记录多于一条，恢复在任何写之前停下
        // （`crates/singlefs-core/src/recovery.rs` 的 `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`）。
        return RecoveryReport {
            outcome: RecoveryOutcome::Failed { root: Some(root_key), reason: "root_publish_carries_more_than_one_last_record_flag_whose_anchor_is_undecided".to_string() },
            journal,
            mapping_fallbacks,
            clause_branches,
            clause_fatal,
        };
    }
    let outcome = match walk_to_file(reader, &effective_root, &system_configuration.fsid, &mut mapping_fallbacks) {
        Ok(Some(content)) => RecoveryOutcome::FileRead { root: root_key, content },
        Ok(None) => RecoveryOutcome::NoFile { root: root_key },
        Err(reason) => RecoveryOutcome::Failed { root: Some(root_key), reason },
    };
    RecoveryReport { outcome, journal, mapping_fallbacks, clause_branches, clause_fatal }
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
    /// S5（第八节）：读者乙在这一轮全部状态里走到 ①–④ 各分支的次数总和；`reader_mode` 不是 `Clause` 时恒 0。
    reader_branches: ReaderClauseBranchCounts,
}

fn oracle_violation(outcome: &RecoveryOutcome, root_persisted: bool, expected_content: &[u8]) -> Option<String> {
    match outcome {
        RecoveryOutcome::FileRead { content, .. } => (content != expected_content).then(|| "读回的内容不对".to_string()),
        RecoveryOutcome::NoFile { .. } => root_persisted.then(|| "根槽已持久而恢复到旧态".to_string()),
        RecoveryOutcome::Failed { reason, .. } => Some(format!("走读失败：{reason}")),
    }
}

fn evaluate_state(base: &Pool, writes: &[WriteRequest], persisted: Vec<bool>, root_index: usize, expected_content: &[u8], reader_mode: ReaderMode, tally: &mut Layer0Tally) {
    let root_persisted = persisted[root_index];
    let image = CrashImage { base, writes, persisted };
    let consulted = recover(&image, JournalPolicy::Consult, reader_mode);
    let ignored = recover(&image, JournalPolicy::Ignore, reader_mode);
    tally.reader_branches.add(&consulted.clause_branches);
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

fn enumerate_layer0(base: &Pool, writes: &[WriteRequest], segments: &[Vec<usize>], expected_content: &[u8], reader_mode: ReaderMode) -> Layer0Tally {
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
            evaluate_state(base, writes, persisted, root_index, expected_content, reader_mode, &mut tally);
        }
        for write_index in segment {
            persisted_before[*write_index] = true;
        }
    }
    evaluate_state(base, writes, persisted_before, root_index, expected_content, reader_mode, &mut tally);
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
        Probe { name: "system_configuration_slot_one_both_devices", flips: both(DeviceOffset(SYSTEM_CONFIGURATION_SLOT_OFFSETS[1]), 50) },
    ]
}

fn run_probe(full: &Pool, probe: &Probe) -> RecoveryReport {
    let mut damaged = full.clone();
    for (device, offset, byte_index) in &probe.flips {
        flip_byte(&mut damaged, *device, *offset, *byte_index);
    }
    recover(&damaged, JournalPolicy::Consult, ReaderMode::Clause)
}

// ───────────────────────── E142 第十一次跑：改动计数从 1 改成 3，逐字节相减 ─────────────────────────
// 重跑登记 research/prompts/e142-r11-prereg.md 第一节第 3 条：分母是两块盘的全部字节，不是只看那 8 个单元。
// 没写过的扇区两边都读 0，差不出字节，所以「两份镜像里写过的扇区并集」与「全部字节逐字节比对」等价（第 3 步跑之前已在这里写死这条推理）。

/// 量 1 的六元组，去掉「所属结构 + 结构内偏移」——那两列由 `classify_offset` 事后算，不是逐字节比对本身要用的。
#[derive(Clone, Debug)]
struct DiffSegment {
    device: u32,
    offset: u64,
    length: u64,
    old: Vec<u8>,
    new: Vec<u8>,
}

/// 两份镜像（同设备数）逐字节相减，合并相邻差异字节成段。
fn diff_pools(old: &Pool, new: &Pool) -> Vec<DiffSegment> {
    assert_eq!(old.devices.len(), new.devices.len(), "两份镜像设备数必须一致才能相减");
    let mut segments = Vec::new();
    let zero_sector = [0u8; PHYSICAL_BLOCK_BYTES as usize];
    for device_index in 0..old.devices.len() {
        let device = u32::try_from(device_index).expect("设备数");
        let mut sectors: std::collections::BTreeSet<u64> = old.devices[device_index].sectors.keys().copied().collect();
        sectors.extend(new.devices[device_index].sectors.keys().copied());
        let mut open: Option<(u64, Vec<u8>, Vec<u8>)> = None;
        for sector in sectors {
            let sector_offset = sector * PHYSICAL_BLOCK_BYTES;
            let old_sector = old.devices[device_index].sectors.get(&sector).unwrap_or(&zero_sector);
            let new_sector = new.devices[device_index].sectors.get(&sector).unwrap_or(&zero_sector);
            for byte_index in 0..PHYSICAL_BLOCK_BYTES as usize {
                let absolute = sector_offset + byte_index as u64;
                let old_byte = old_sector[byte_index];
                let new_byte = new_sector[byte_index];
                if old_byte == new_byte {
                    if let Some((start, old_bytes, new_bytes)) = open.take() {
                        segments.push(DiffSegment { device, offset: start, length: old_bytes.len() as u64, old: old_bytes, new: new_bytes });
                    }
                    continue;
                }
                let extends = matches!(&open, Some((start, old_bytes, _)) if *start + old_bytes.len() as u64 == absolute);
                if extends {
                    let (_, old_bytes, new_bytes) = open.as_mut().expect("刚判过 extends");
                    old_bytes.push(old_byte);
                    new_bytes.push(new_byte);
                } else {
                    if let Some((start, old_bytes, new_bytes)) = open.take() {
                        segments.push(DiffSegment { device, offset: start, length: old_bytes.len() as u64, old: old_bytes, new: new_bytes });
                    }
                    open = Some((absolute, vec![old_byte], vec![new_byte]));
                }
            }
        }
        if let Some((start, old_bytes, new_bytes)) = open.take() {
            segments.push(DiffSegment { device, offset: start, length: old_bytes.len() as u64, old: old_bytes, new: new_bytes });
        }
    }
    segments
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect::<Vec<_>>().join("")
}

/// 量 1/3/4 的「所属结构」候选表：第一个事务写到的 8 个单元、根记录、journal 记录（第一节第 3 条的区域清单，去掉系统配置——
/// 系统配置与改动计数无关，按第三节推导；它若意外出现在差异里，classify_offset 找不到候选、归到「未登记」而不是被这张表悄悄吃掉）。
struct StructureCatalog<'output> {
    units: &'output [(SlotNumber, TransactionUnit, Vec<u8>)],
    root_device: u32,
    root_offset: u64,
    journal_offset: u64,
    journal_length: u64,
}

fn build_structure_catalog(output: &TransactionOutput, root_device: u32, root_offset: u64) -> StructureCatalog<'_> {
    StructureCatalog { units: &output.units_by_slot, root_device, root_offset, journal_offset: journal_record_offset(output.record.counter).0, journal_length: output.record_bytes.len() as u64 }
}

/// 给一个 (device, offset) 找它落在哪个候选结构里；返回 (标签, 结构内偏移)。
fn classify_offset(catalog: &StructureCatalog, device: u32, offset: u64) -> Option<(&'static str, u64)> {
    for (slot, unit, bytes) in catalog.units {
        let base = slot.device_offset().0;
        if offset >= base && offset < base + bytes.len() as u64 {
            return Some((unit.descriptive_tag(), offset - base));
        }
    }
    if device == catalog.root_device && offset >= catalog.root_offset && offset < catalog.root_offset + PHYSICAL_BLOCK_BYTES {
        return Some(("root_record", offset - catalog.root_offset));
    }
    if offset >= catalog.journal_offset && offset < catalog.journal_offset + catalog.journal_length {
        return Some(("journal_record", offset - catalog.journal_offset));
    }
    None
}

/// NodePointer 内两条位置条目校验和的相对偏移（`NodePointer::write_to`：头 50 + 位置条目 14 × 2，校验和在条目内偏移 10）。
fn node_pointer_checksum_offsets(pointer_start: u64) -> [u64; 2] {
    [pointer_start + POINTER_HEAD_BYTES as u64 + 10, pointer_start + POINTER_HEAD_BYTES as u64 + LOC_ENTRY + 10]
}

fn overlaps(segment_offset: u64, segment_length: u64, target_offset: u64, target_length: u64) -> bool {
    segment_offset < target_offset + target_length && target_offset < segment_offset + segment_length
}

/// inode 记录字段表（D8 已定项 6，`decisions/08-核心索引结构.md:391-410`）：偏移 → 字段名。
fn inode_record_field_name(offset: u64) -> &'static str {
    match offset {
        0..=7 => "inode",
        8..=15 => "object_birth（出生代）",
        16..=23 => "locality_id",
        24..=39 => "mode/uid/gid/nlink",
        40..=63 => "size/blocks/rdev",
        64..=87 => "atime/mtime/ctime 秒",
        88..=95 => "改动计数",
        96..=107 => "atime/mtime/ctime 纳秒",
        108..=111 => "填充",
        112..=119 => "flags",
        _ => "预留",
    }
}

/// 量 3：给一个 (结构标签, 结构内偏移) 找「谁的校验和 / 指针罩到它」。返回 (是否归了因, 说明)。
/// 归不了因的（返回 false）按 F6 记一行 `name=gap`，不作废——重跑登记第十节 F6。
fn explain_unit_offset(unit: TransactionUnit, offset: u64) -> (bool, String) {
    let (class, key_width) = unit.class_and_key_width();
    if (10..42).contains(&offset) {
        return (true, "单元头校验和：seal_header_checksum 罩 [0,头末尾)，头内容变了就重算（D18 已定项 17）".to_string());
    }
    let payload_crc_offset: u64 = if class == UNIT_CLASS_PACKED { 89 } else { 76 + 2 * key_width as u64 };
    if (payload_crc_offset..payload_crc_offset + 4).contains(&offset) {
        return (true, format!("单元载荷 CRC：castagnoli_crc32 罩记录/条目区到单元末尾（D18 已定项 11 逐字，偏移 {payload_crc_offset}）"));
    }
    match unit {
        TransactionUnit::InodeLeaf => {
            let record_start = (PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES) as u64;
            if offset >= record_start && offset < record_start + INODE_RECORD_BYTES {
                let field_offset = offset - record_start;
                return (true, format!("inode 记录内偏移 {field_offset}：{}（D8 已定项 6）", inode_record_field_name(field_offset)));
            }
        }
        TransactionUnit::InodeRoot => {
            let entries_start = index_node_header_bytes(key_width) as u64 + NONCE_MAC_RESERVED_BYTES;
            if offset >= entries_start && offset < entries_start + INODE_INTERNAL_ENTRY {
                let entry_offset = offset - entries_start;
                let pointer_start = 34u64; // 分隔 key 8 + 子身份引用 26（build_inode_internal_entry）
                if entry_offset >= pointer_start && entry_offset < pointer_start + NODE_POINTER_BYTES {
                    let checks = node_pointer_checksum_offsets(pointer_start);
                    if (checks[0]..checks[0] + 4).contains(&entry_offset) {
                        return (true, "内部条目里子指针（指向 inode 叶）第 0 条位置条目单元校验和（D19 已定项 4）".to_string());
                    }
                    if (checks[1]..checks[1] + 4).contains(&entry_offset) {
                        return (true, "内部条目里子指针（指向 inode 叶）第 1 条位置条目单元校验和（D19 已定项 4）".to_string());
                    }
                    return (true, "内部条目里子指针其余字段（出生树 / 出生代号 / 实例代号 / 出生序号，不含校验和）".to_string());
                }
                return (true, format!("内部条目内偏移 {entry_offset}（分隔 key / 子身份引用段）"));
            }
        }
        TransactionUnit::MappingTree => {
            let entries_start = index_node_header_bytes(key_width) as u64 + NONCE_MAC_RESERVED_BYTES;
            if offset >= entries_start {
                let relative = offset - entries_start;
                let entry_index = relative / MAPPING_ENTRY_BYTES;
                let entry_offset = relative % MAPPING_ENTRY_BYTES;
                if entry_offset >= MAPPING_KEY_BYTES {
                    let location_offset = entry_offset - MAPPING_KEY_BYTES;
                    let which = u64::from(location_offset >= LOC_ENTRY);
                    let local = location_offset % LOC_ENTRY;
                    if (10..14).contains(&local) {
                        return (true, format!("映射树条目 {entry_index} 的第 {which} 条位置条目单元校验和（build_mapping_entry，D19 已定项 10）"));
                    }
                }
                return (true, format!("映射树条目 {entry_index} 内偏移 {entry_offset}（key 或位置条目非校验和字段）"));
            }
        }
        TransactionUnit::TreeTable => {
            let entries_start = index_node_header_bytes(key_width) as u64 + NONCE_MAC_RESERVED_BYTES;
            if offset >= entries_start {
                let relative = offset - entries_start;
                let entry_index = relative / TREE_TABLE_ENTRY_BYTES;
                let entry_offset = relative % TREE_TABLE_ENTRY_BYTES;
                let pointer_start = 14u64; // 树 ID 8 + 条目长度 2 + 种类 2 + flags 2（TreeTableEntry::to_bytes）
                if entry_offset >= pointer_start && entry_offset < pointer_start + NODE_POINTER_BYTES {
                    let local = entry_offset - pointer_start;
                    let checks = node_pointer_checksum_offsets(0);
                    if (checks[0]..checks[0] + 4).contains(&local) {
                        return (true, format!("树表条目 {entry_index} 的根指针第 0 条位置条目单元校验和（D19 已定项 4）"));
                    }
                    if (checks[1]..checks[1] + 4).contains(&local) {
                        return (true, format!("树表条目 {entry_index} 的根指针第 1 条位置条目单元校验和（D19 已定项 4）"));
                    }
                    return (true, format!("树表条目 {entry_index} 的根指针其余字段"));
                }
                return (true, format!("树表条目 {entry_index} 内偏移 {entry_offset}（树 ID / 种类 / 出生 txg / 头 ID 一类）"));
            }
        }
        _ => {}
    }
    (false, format!("单元内偏移 {offset}：不在头校验和 / 载荷 CRC / 已知记录条目区里，未登记"))
}

fn explain_root_record_offset(offset: u64) -> (bool, String) {
    match offset {
        0..=3 => (true, "root magic".to_string()),
        4..=19 => (true, "fsid".to_string()),
        20..=23 => (true, "flags 占位（恒 0）".to_string()),
        24..=27 => (true, "instance".to_string()),
        28..=35 => (true, "checkpoint_txg".to_string()),
        36..=121 => {
            let local = offset - 36;
            let checks = node_pointer_checksum_offsets(0);
            if (checks[0]..checks[0] + 4).contains(&local) {
                (true, "tree_table 指针第 0 条位置条目单元校验和".to_string())
            } else if (checks[1]..checks[1] + 4).contains(&local) {
                (true, "tree_table 指针第 1 条位置条目单元校验和".to_string())
            } else {
                (true, "tree_table 指针其余字段".to_string())
            }
        }
        122..=129 => (true, "tree_identifier_watermark".to_string()),
        130..=137 => (true, "rollback_floor".to_string()),
        138..=169 => (true, "根记录自证校验和（罩整个 512 槽含补齐、自身按 0 参与，D18 已定项 17）".to_string()),
        170..=255 => (true, "instance_table 指针（预期不随改动计数变；若变了记一行 gap）".to_string()),
        256..=341 => {
            let local = offset - 256;
            let checks = node_pointer_checksum_offsets(0);
            if (checks[0]..checks[0] + 4).contains(&local) {
                (true, "mapping_root 指针第 0 条位置条目单元校验和".to_string())
            } else if (checks[1]..checks[1] + 4).contains(&local) {
                (true, "mapping_root 指针第 1 条位置条目单元校验和".to_string())
            } else {
                (true, "mapping_root 指针其余字段".to_string())
            }
        }
        342..=427 => (
            true,
            "分配记录树根指针（C512（树表 0 条的一版上被换下的实例表记在哪没有条款） 2026-09-23 用户定案：这个装置的场景里恒 0，mkfs / 暖机 / 带文件的一版都不是「树表 0 条、写过行」那一格）".to_string(),
        ),
        428..=456 => (true, "算法类型 / nonce / MAC 留位（恒 0）".to_string()),
        _ => (false, format!("根记录内偏移 {offset}：落在 457 字节字段表之外的补齐区，未登记")),
    }
}

fn explain_journal_record_offset(units: &[(SlotNumber, TransactionUnit, Vec<u8>)], offset: u64) -> (bool, String) {
    // E142 第十四次跑：整张表按 D23 已定项 4 的 311 字段表重写（第三节 3.2：旧表把偏移 8–45 标错了 2 字节）；
    // 每个字段的起点由第七节 A2 的单测钉住。
    match offset {
        0..=3 => (true, "journal magic".to_string()),
        4..=5 => (true, "type".to_string()),
        6 => (true, "algorithm".to_string()),
        7 => (true, "record_flags（位 0 = 本次发布末条，已定项 4、17）".to_string()),
        8..=11 => (true, "record_length".to_string()),
        12..=15 => (true, "named_count".to_string()),
        16..=19 => (true, "instance（jsn 高位）".to_string()),
        20..=25 => (true, "counter（jsn 低位，six-byte）".to_string()),
        26..=33 => (true, "checkpoint_txg".to_string()),
        34..=45 => (true, "nonce 留位（恒 0）".to_string()),
        46..=77 => (true, "记录头校验和（罩整条记录 [0,4096) 含补齐、自身按 0 参与，D23 已定项 13）".to_string()),
        78..=85 => (true, "transaction".to_string()),
        86 => (true, "is_commit".to_string()),
        87..=90 => (true, "ordinal_within_publish（本次发布内序号，已定项 4）".to_string()),
        91..=94 => (true, "back_chain".to_string()),
        95..=98 => (true, "载荷校验和（罩点名项数组，D23 已定项 13）".to_string()),
        99..=184 => {
            let local = offset - 99;
            let checks = node_pointer_checksum_offsets(0);
            if (checks[0]..checks[0] + 4).contains(&local) {
                (true, "new_tree_table 指针第 0 条位置条目单元校验和".to_string())
            } else if (checks[1]..checks[1] + 4).contains(&local) {
                (true, "new_tree_table 指针第 1 条位置条目单元校验和".to_string())
            } else {
                (true, "new_tree_table 指针其余字段".to_string())
            }
        }
        185..=270 => {
            let local = offset - 185;
            let checks = node_pointer_checksum_offsets(0);
            if (checks[0]..checks[0] + 4).contains(&local) {
                (true, "new_mapping_root 指针第 0 条位置条目单元校验和".to_string())
            } else if (checks[1]..checks[1] + 4).contains(&local) {
                (true, "new_mapping_root 指针第 1 条位置条目单元校验和".to_string())
            } else {
                (true, "new_mapping_root 指针其余字段".to_string())
            }
        }
        271..=278 => (true, "new_tree_identifier_watermark".to_string()),
        279..=286 => (true, "new_rollback_floor".to_string()),
        287..=294 => (true, "fsid".to_string()),
        295..=310 => (true, "MAC 留位（恒 0）".to_string()),
        _ => {
            let relative = offset - JOURNAL_HEADER_BYTES;
            let entry_index = (relative / JOURNAL_NAMED_ENTRY_BYTES) as usize;
            let entry_offset = relative % JOURNAL_NAMED_ENTRY_BYTES;
            let tag = units.get(entry_index).map_or("?", |(_, unit, _)| unit.descriptive_tag());
            if (10..14).contains(&entry_offset) {
                return (true, format!("点名项 {entry_index}（{tag}）第 0 条位置条目单元校验和"));
            }
            if (24..28).contains(&entry_offset) {
                return (true, format!("点名项 {entry_index}（{tag}）第 1 条位置条目单元校验和"));
            }
            (true, format!("点名项 {entry_index}（{tag}）内偏移 {entry_offset}（非校验和字段）"))
        }
    }
}

/// 量 3 的统一入口：按结构标签分派到对应的 explain 函数。
fn explain_offset(catalog: &StructureCatalog, structure: &str, offset_in_structure: u64) -> (bool, String) {
    if structure == "root_record" {
        return explain_root_record_offset(offset_in_structure);
    }
    if structure == "journal_record" {
        return explain_journal_record_offset(catalog.units, offset_in_structure);
    }
    for (_, unit, _) in catalog.units {
        if unit.descriptive_tag() == structure {
            return explain_unit_offset(*unit, offset_in_structure);
        }
    }
    (false, format!("标签 {structure} 不在候选表里"))
}

/// 问题单第 1 行「够判条件」点名的六处（重跑登记第一节）：每处至少一段差异才够判。
#[derive(Clone, Copy, Default, Debug)]
struct ChangeCountBullets {
    offset_88_covered: bool,
    leaf_checksum_covered: bool,
    pointer_to_leaf_checksum_covered: bool,
    inode_root_checksum_covered: bool,
    tree_table_covered: bool,
    root_record_checksum_covered: bool,
}

impl ChangeCountBullets {
    fn all_covered(&self) -> bool {
        self.offset_88_covered && self.leaf_checksum_covered && self.pointer_to_leaf_checksum_covered && self.inode_root_checksum_covered && self.tree_table_covered && self.root_record_checksum_covered
    }
    fn record(&mut self, structure: &str, offset_in_structure: u64, length: u64) {
        match structure {
            "inode_leaf" => {
                let record_start = (PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES) as u64;
                if overlaps(offset_in_structure, length, record_start + 88, 8) {
                    self.offset_88_covered = true;
                }
                if overlaps(offset_in_structure, length, 10, 32) || overlaps(offset_in_structure, length, 89, 4) {
                    self.leaf_checksum_covered = true;
                }
            }
            "inode_root" => {
                // 结构内偏移 225 / 239（重跑登记第七节乙类，entries_start 131 + pointer_start 34 + {60,74}）。
                if overlaps(offset_in_structure, length, 225, 4) || overlaps(offset_in_structure, length, 239, 4) {
                    self.pointer_to_leaf_checksum_covered = true;
                }
                if overlaps(offset_in_structure, length, 10, 32) || overlaps(offset_in_structure, length, 92, 4) {
                    self.inode_root_checksum_covered = true;
                }
            }
            "tree_table" => self.tree_table_covered = true,
            "root_record" => {
                if overlaps(offset_in_structure, length, 138, 32) {
                    self.root_record_checksum_covered = true;
                }
            }
            _ => {}
        }
    }
}

// ───────────────────────── 装置自己补的取法：空白清单 ─────────────────────────

/// 每一条是仓里没有条款、装置不得不自己取一个值才写得出字节的地方。文字里不用空格，好让结果行按空格切段。
const GAPS: &[(&str, &str)] = &[
    ("G1", "已收口（2026-09-13 D8已定项11 + D18已定项16/18）：码2头=86+2×key宽，含29字节预留位115+2×key宽（inode与树表131、分配135、记账159、extent163、映射169）；跑出空白那天三处预想数（84、68+44、72/81/90三档）互不相等"),
    ("G2", "已收口（2026-09-14用户定案，C325还清）：key宽字段从81+2k挪到**偏移51**，它自己的偏移不依赖k⇒扫描期按[0,51)这段定长前缀读出k、一次定出头末端，I-2.4对码2从此直接判得了；装置的parse_index_node不再收调用方给的key宽"),
    ("G3", "已收口（2026-09-14用户定案改写D19已定项10）：映射key**一律27**——码2/码3的25字节末尾补零到27、条目一律55；2026-09-13那一版的「按类27/25不补齐、条目55/53」作废"),
    ("G4", "根记录字段序：D22已定项7的表与字节表七的表行序不同（水位、F、校验和、实例表指针四行的先后），两处都没写偏移；2026-09-14加的算法类型1+nonce12+MAC16同样只说「放在中央映射树根指针之后」、没写偏移；装置按D22的表序"),
    ("G5", "已收口（2026-09-13 D18已定项17）：32字节校验和字段里放CRC32C4字节+28字节零，自证结构罩整个槽含补齐、字段按零参与；跑出空白那天装置取的是SHA-256"),
    ("G6", "已收口（2026-09-13 D23已定项19②，2026-09-14改写口径，2026-09-24头宽改311）：previous_hash=CRC32C(**本实例内逻辑前一条**记录的311字节头，header_csum那32字节按零参与)，本实例写出的第一条恒0、不读盘；已定项17列的改动字节没提这一处，见G26"),
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
    ("G19", "mkfs种根的21次操作（19写+2屏障，段序列12+1+1+1+4、4114个崩溃状态，D22已定项8第3条2026-09-23用户定案改写之后含两块盘各清根环三区域与journal环共8步）不在层0枚举里：装置从mkfs之后的池起枚举（取号、暖机与事务），mkfs的崩溃状态没有任何东西判；段序列另发一行钉住，与layout/01-first-txn.md八「mkfs种根」那一行登记的真值一致"),
    ("G20", "已收口（2026-09-14用户定案，C321还清）：映射条目回到一宽55⇒码2头那个u16的条目宽字段够用，声明长度=条目数×条目宽原样成立；装置删掉了「条目宽0当变长哨兵」那套自创取法，parse_index_node对条目宽0一律判结构错"),
    ("G21", "取号那一步（D23已定项16：第一次可写挂载写每一份系统配置之后才动单元）不是根槽写路径，layout/01-first-txn八那张表罩不到它；屏障怎么放没有条款，装置按最少屏障取「不另加屏障，靠暖机第一次空发布开头那道屏障收段」⇒段序列独占一行[system_configuration_slot×2]"),
    ("G22", "**仍欠着**（C323，2026-09-14加注）：镜像大小（单元区的末端）全仓仍没有条款，D23已定项19③只定了「环≤设备容量÷4」⇒它给出容量的**下界**而不是值；装置按mkfs参数取4GiB（1GiB装不下默认768MiB的环）并在name=config里报出来，空闲字节3472670720、全空聚簇段数3310、runs4三个数都随它变"),
    ("G23", "已收口（2026-09-14用户定案，C324还清）：码3打包容器按「数据单元」那一档取落点（起点32768对齐、两槽都空）；连同D3已定项10⑤的bump次序（树ID升序、树内先叶后根、映射树倒数第二、树表最末）⇒t2 extent根拿50240、游标停在50241而t3要对齐⇒t3拿50242–50243，**槽50241空着**、runs因此从3变4"),
    ("G24", "树表条目的「头ID」（D5已定项9，2026-09-14用户定案）只说了inode树写自己、extent树写12、其余写0，没说**这个数从哪来**：第一版只有一个可写头，装置按「inode树的树ID就是头ID」写；多头之后头ID与树ID还是不是一回事，条款没答"),
    ("G25", "journal记录头2026-09-14加的fsid8与MAC16，条款只说fsid「与单元头同口径」（系统配置fsid的低8字节）、MAC第一版全0；**读者拿fsid做什么没写**——装置按I-1.4把fsid不符的记录整条丢掉（不进重放前缀），而「丢掉」与「判损坏断链」在条款里分不出来"),
    ("G26", "E142第十四次跑F1a：D23已定项17写标志改动「改第一个事务的字节（w1、w4、t9三条记录的这一字节与头部校验和）」，没列w4、t9的反向链；而已定项10「罩前一条的整个记录头」使反向链必然随头宽与标志字节的改动跟着变（w1是本实例第一条、反向链恒0，不受影响）——不作废，如实记这一处条款没提到"),
    ("G27", "5.2 ι：D8已定项14第395行只说内部条目「key10+子指针86、按key排」，没写这个key取「孩子按位置罩的段起点」还是「孩子子树里最小那条记录的key」；装置取ι甲（段起点），ι乙（记录最小key）登记成第二段的读法对照函数，主读法不建变体分支"),
    ("G28", "5.2 γ：D18已定项2射程与D8已定项14第385行的[k×W,(k+1)×W)都没写分配记录树叶/层级1节点区间的末端是闭区间还是半开——装置取γ甲（闭区间，D18定案句「[min_key,max_key]」与「最大key」更直接），γ乙（半开右端）登记成第二段的读法对照函数"),
    ("G29", "5.2 ε：D8已定项14第398/400行「标签0全零、只有读者认、写者不写」与「上段叶罩143个inode」两句字面上互相拉扯——装置取ε甲（稀疏，只写有文件的条目），ε乙（143格全写、空格标签0）登记成第二段的读法对照函数"),
    ("G30", "5.2 ζ：条款只说「上段按inode号的位置寻址」，没写「根的层级怎么算」；装置取「最低的、罩得住最大inode号的层级」（这个装置的几何inode1⇒层级0，根即叶），没有找到第二种写得出字节的读法，不登记变体"),
    ("G31", "5.2 θ：D23已定项17只说「一项一个单元」，没写点名项的次序；装置取「照单元的先后（数据单元在前，其后照bump次序）」，映射条目按D8已定项11的映射key排；这个装置的几何两种候选次序同值，不登记变体"),
    ("G32", "5.2 κ：条款没说层级1节点的段伸出盘外时区间末端截不截到盘末；这个装置的几何（两块4GiB盘）每盘只写层级1第0个节点、整段在盘内，走不到这一格，不登记读法"),
    ("F2", "D3已定项10⑤的落点表（t5『分配记录树根兼叶』落50245、t8树表落50248）与字节表零那一节写清单表的t5那一行都还是按位置寻址之前的布局，与D8已定项14『4GiB×2时根在第2层、树高3』字面对不上：条款自己两处说法不一致，不作废、不停机，这一次的写清单照D8已定项14与D3已定项10⑤的规则句（bump次序）写，两处旧表由书记员另行改"),
];

/// 整条路（mkfs → 取号 → 暖机两次 → 第一个事务）跑一遍，返回落盘后的整份镜像与这次发布的产出。
/// E142 第十一次跑用它跑出 M_A（change_count = FIRST_TRANSACTION_TXG）、M_B（真实基线，change_count = 1）与量 8 的两个几何取样点。
fn run_full_pipeline(parameters: &PoolParameters, change_count: u64, file_bytes: &[u8]) -> (Pool, TransactionOutput) {
    let (mut recording, genesis) = mkfs(parameters);
    let instance = acquire_instance(&mut recording, parameters);
    let (_, last_warm_up_record) = warm_up(&mut recording, parameters, &genesis, instance);
    let output = publish_first_file(&mut recording, parameters, &genesis, file_bytes, instance, last_warm_up_record.as_deref(), change_count);
    (recording.pool.clone(), output)
}

/// P6（E142 第十四次跑第五节 5.2）用：跑到「t9 之前（含 t9 两份）全部持久、t10 根槽与 t11 系统配置槽都没持久」
/// 那个定点状态 S*，返回 (暖机之后的池, 第一个事务的全部写请求, 这次发布的产出)。`s_star_persisted` 配它用：
/// 根槽 FUA 与系统配置槽两类写标 false，其余（8 个单元 × 2 盘 + journal 记录 × 2 盘）标 true。
fn s_star_writes(parameters: &PoolParameters, file_bytes: &[u8]) -> (Pool, Vec<WriteRequest>, TransactionOutput) {
    let (mut recording, genesis) = mkfs(parameters);
    let instance = acquire_instance(&mut recording, parameters);
    let (_, last_warm_up_record) = warm_up(&mut recording, parameters, &genesis, instance);
    let base = recording.pool.clone();
    let warm_up_operation_count = recording.operations.len();
    let output = publish_first_file(&mut recording, parameters, &genesis, file_bytes, instance, last_warm_up_record.as_deref(), FIRST_TRANSACTION_TXG);
    let (writes, _segments) = split_into_segments(&recording.operations[warm_up_operation_count..], false);
    (base, writes, output)
}

fn s_star_persisted(writes: &[WriteRequest]) -> Vec<bool> {
    writes.iter().map(|write| !matches!(write.kind, StepKind::RootRecordFua | StepKind::SystemConfigurationSlot)).collect()
}

/// 把 `writes` 里两份 t9 记录（两盘各一份 `StepKind::JournalRecord`）解析回 `JournalRecord`、按 `mutate` 改一个字段、
/// 重封头校验和（D23 已定项 13）再写回字节——P6 的「两份改写」都靠它，不手改字节偏移。
fn corrupt_t9_records(writes: &[WriteRequest], mutate: impl Fn(&mut JournalRecord)) -> Vec<WriteRequest> {
    writes
        .iter()
        .map(|write| {
            if write.kind == StepKind::JournalRecord {
                let mut record = JournalRecord::parse(&write.bytes).expect("t9 记录自检要过");
                mutate(&mut record);
                WriteRequest { bytes: record.to_bytes(), ..write.clone() }
            } else {
                write.clone()
            }
        })
        .collect()
}

/// P6「t9 之后在计数器 4 的槽里加一条同 txg 3 的记录」：两盘各一份，序号 2、标志 0x00、提交标记 1、
/// 反向链 = CRC32C(t9 的头)、不点名（重跑登记第五节 5.2 那一行）。
fn append_trailing_same_txg_record(writes: &[WriteRequest], t9: &JournalRecord, t9_bytes: &[u8], parameters: &PoolParameters) -> Vec<WriteRequest> {
    let extra = JournalRecord {
        instance: t9.instance,
        counter: JournalCounter(4),
        checkpoint_txg: t9.checkpoint_txg,
        transaction: TransactionNumber(0),
        is_commit: true,
        ordinal_within_publish: 2,
        record_flags: 0x00,
        back_chain: journal_back_chain(t9_bytes),
        fsid: t9.fsid,
        new_tree_table: t9.new_tree_table,
        new_mapping_root: t9.new_mapping_root,
        new_tree_identifier_watermark: t9.new_tree_identifier_watermark,
        new_rollback_floor: t9.new_rollback_floor,
        named: Vec::new(),
    };
    let extra_bytes = extra.to_bytes();
    let extra_offset = journal_record_offset(JournalCounter(4));
    let mut extended = writes.to_vec();
    for device in parameters.devices() {
        extended.push(WriteRequest { device, offset: extra_offset, bytes: extra_bytes.clone(), kind: StepKind::JournalRecord });
    }
    extended
}

/// 已定项 4 的①：标志字节位 0 之外有值 ⇒ 当损坏。`replay_journal_clause` 的候选过滤与
/// `readable_record_count` 共用这一个判据，不各写一份——各写一份会让改坏其中一处的变异
/// 只影响一个观测点，另一个观测点看不出来（M84 踩过这个坑）。
fn record_flag_byte_is_corrupted(record: &JournalRecord) -> bool {
    record.record_flags & !0x01 != 0
}

/// 已定项 4 的①：本次发布内序号为 0 ⇒ 当损坏。同上，与 `replay_journal_clause` 共用。
fn record_ordinal_is_corrupted(record: &JournalRecord) -> bool {
    record.ordinal_within_publish == 0
}

/// P6 读者读了标志与序号一格里，「读得出的记录条数」（按 jsn 去重，已定项 4 的①判「当损坏」的那两条）。
fn readable_record_count(records: &BTreeMap<(InstanceGeneration, JournalCounter), JournalRecord>) -> usize {
    records.values().filter(|record| !record_flag_byte_is_corrupted(record) && !record_ordinal_is_corrupted(record)).count()
}

fn p6_mutate_flag_0x03(record: &mut JournalRecord) {
    record.record_flags = 0x03;
}
fn p6_mutate_ordinal_0(record: &mut JournalRecord) {
    record.ordinal_within_publish = 0;
}
fn p6_mutate_flag_0x00(record: &mut JournalRecord) {
    record.record_flags = 0x00;
}
fn p6_mutate_ordinal_2(record: &mut JournalRecord) {
    record.ordinal_within_publish = 2;
}
fn p6_mutate_txg_2(record: &mut JournalRecord) {
    record.checkpoint_txg = CheckpointTxg(2);
}

/// `recover()` 的 `outcome.root` 恒是**所选**的那条根（施加记录之前），不是施加之后的——`recover` 函数上的注释
/// 与 `probes_behave_as_milestone_step_six_expects` 那条已有测试都是这个口径。P6 的预期表问的是「施加记录之后」
/// 的 (实例代号, checkpoint_txg)（重跑登记第五节 5.2 表头），S* 的 `above` 至多 1 条（t9），
/// 所以用 `journal.prefix_applied` 是否 ≥ 1 就等价于「重建出来的根有没有推进到 t9 那次发布」。
#[derive(Debug, Clone)]
struct P6Outcome {
    primary_prefix_applied: usize,
    primary_readable: usize,
    clause_prefix_applied: usize,
    clause_readable: usize,
    clause_fatal: bool,
}

/// P6（第五节 5.2）跑一格：在 S*（t9 之前含 t9 两份全部持久、t10 根槽与 t11 系统配置槽都没持久）上，
/// 可选地改写 t9（`mutate`）或在 t9 之后加一条同 txg 的记录（`append_trailing`），两种读者各跑一次。
fn run_p6_scenario(parameters: &PoolParameters, file_bytes: &[u8], mutate: Option<fn(&mut JournalRecord)>, append_trailing: bool) -> P6Outcome {
    let (base, base_writes, output) = s_star_writes(parameters, file_bytes);
    let mut writes = base_writes;
    if let Some(mutate) = mutate {
        writes = corrupt_t9_records(&writes, mutate);
    }
    if append_trailing {
        writes = append_trailing_same_txg_record(&writes, &output.record, &output.record_bytes, parameters);
    }
    let persisted = s_star_persisted(&writes);
    let image = CrashImage { base: &base, writes: &writes, persisted };
    let primary = recover(&image, JournalPolicy::Consult, ReaderMode::Primary);
    let clause = recover(&image, JournalPolicy::Consult, ReaderMode::Clause);
    // 读者甲不判标志/序号是否合法，「读得出」= 扫描到、校验和过的全部记录；读者乙额外按①滤掉当损坏的那些。
    let records = scan_journal(&image, unit_fsid(&parameters.fsid));
    P6Outcome {
        primary_prefix_applied: primary.journal.prefix_applied,
        primary_readable: records.len(),
        clause_prefix_applied: clause.journal.prefix_applied,
        clause_readable: readable_record_count(&records),
        clause_fatal: clause.clause_fatal,
    }
}

/// 差异段所属结构名去重（量 8 的「结构集合」）。
fn structure_name_set(differences: &[DiffSegment], catalog: &StructureCatalog) -> std::collections::BTreeSet<&'static str> {
    differences.iter().map(|segment| classify_offset(catalog, segment.device, segment.offset).map_or("unregistered", |(structure, _)| structure)).collect()
}

/// 差异段的完整位置集合（设备 + 结构 + 结构内偏移，不含值——量 8 的判据要求「不含值」）。
fn structure_position_set(differences: &[DiffSegment], catalog: &StructureCatalog) -> std::collections::BTreeSet<(u32, &'static str, u64)> {
    differences
        .iter()
        .map(|segment| {
            let (structure, offset) = classify_offset(catalog, segment.device, segment.offset).unwrap_or(("unregistered", segment.offset));
            (segment.device, structure, offset)
        })
        .collect()
}

/// 量 8 的判据：结构集合逐项相等（不是「相差 ≤ 1 段」的容差版——V6 变异测的就是这一句）。
fn structure_sets_match(left: &std::collections::BTreeSet<&str>, right: &std::collections::BTreeSet<&str>) -> bool {
    left == right
}

// ═════════ E142 第十五次跑步④（`research/prompts/e142-r15-prereg.md` 第六节）：Q142.1–Q142.8 的支撑函数 ═════════
// 窗口 = `transaction_operations`（暖机最后一次写之后到发布结束），与比对侧
// `crates/singlefs-harness/src/bin/e142_first_transaction_write_dump.rs` 的窗口取法同一个切点（P5）。
// 配对键是 (设备, 偏移, 长度)，不按名字配对（R3）；`region=`/`unit=` 只是给人看的标签，来自 `classify_offset`。

/// 窗口里模型自己的一次写：给 Q142.1、Q142.3、Q142.8 共用。
struct ModelWindowWrite {
    step: usize,
    region: &'static str,
    device: u32,
    offset: u64,
    bytes: Vec<u8>,
}

fn model_window_writes(catalog: &StructureCatalog, transaction_operations: &[RecordedOperation]) -> Vec<ModelWindowWrite> {
    let mut out = Vec::new();
    for (step, operation) in transaction_operations.iter().enumerate() {
        if let RecordedOperation::Write(write) = operation {
            let region = classify_offset(catalog, write.device.0, write.offset.0).map_or("unregistered", |(label, _)| label);
            out.push(ModelWindowWrite { step, region, device: write.device.0, offset: write.offset.0, bytes: write.bytes.clone() });
        }
    }
    out
}

/// 第八节 G4 用：跑一遍 mkfs → 取号 → 暖机 → 发布，返回窗口（暖机最后一次写之后到发布结束）里模型自己的
/// 写清单——与 `main()` 里对主几何跑的那一遍是同一条逻辑，只是几何参数不同（G4 传一盘的参数）。
fn model_window_writes_for_parameters(parameters: &PoolParameters, file_bytes: &[u8]) -> Vec<ModelWindowWrite> {
    let (mut recording, genesis) = mkfs(parameters);
    let instance = acquire_instance(&mut recording, parameters);
    let (_, last_warm_up_record) = warm_up(&mut recording, parameters, &genesis, instance);
    let warm_up_operation_count = recording.operations.len();
    let output = publish_first_file(&mut recording, parameters, &genesis, file_bytes, instance, last_warm_up_record.as_deref(), FIRST_TRANSACTION_TXG);
    let transaction_operations = &recording.operations[warm_up_operation_count..];
    let (root_region, root_slot) = ring_target_for_publish(CheckpointTxg(FIRST_TRANSACTION_TXG));
    let root_device = parameters.region_devices[root_region as usize].0;
    let root_offset = ring_slot_offset(root_region, root_slot).0;
    let catalog = build_structure_catalog(&output, root_device, root_offset);
    model_window_writes(&catalog, transaction_operations)
}

/// 从一份 `E7RESULT name=<marker> device=… offset=… length=… sha256=… [kind=…] [hexadecimal=…]` 格式的文本里
/// 抠出按 (设备, 偏移, 长度) 配对要用的字段。`crates/` 侧的导出与 arm O 的历史留存产物都是这个形状。
#[derive(Clone, Debug)]
struct ParsedWriteLine {
    device: u32,
    offset: u64,
    length: u64,
    sha256: String,
    hexadecimal: Option<String>,
}

fn parse_write_lines(text: &str, marker: &str) -> Vec<ParsedWriteLine> {
    text.lines()
        .filter(|line| line.contains(marker))
        .map(|line| {
            let fields = parse_result_line(line);
            ParsedWriteLine {
                device: fields.get("device").and_then(|value| value.parse().ok()).unwrap_or(0),
                offset: fields.get("offset").and_then(|value| value.parse().ok()).unwrap_or(0),
                length: fields.get("length").and_then(|value| value.parse().ok()).unwrap_or(0),
                sha256: fields.get("sha256").cloned().unwrap_or_default(),
                hexadecimal: fields.get("hexadecimal").cloned(),
            }
        })
        .collect()
}

/// Q142.1 的核心判定：sha256 相等 ⇒ 全等；不等就用整段十六进制定位第一处差异——抽出成独立函数是因为
/// `main()` 本身不可测（E142 第十一次跑修订第 1 条已经在 V1 上踩过一次这个教训），返回 (equal, first_diff_offset, mismatch_bytes)。
fn compare_paired_write(model_bytes: &[u8], model_sha256: &str, impl_line: &ParsedWriteLine) -> (bool, Option<u64>, Option<u64>) {
    if impl_line.sha256 == model_sha256 {
        return (true, None, Some(0));
    }
    let impl_bytes = impl_line.hexadecimal.as_deref().map(hex_decode).unwrap_or_default();
    let (first_diff_offset, mismatch_bytes) = byte_diff_summary(&impl_bytes, model_bytes);
    (false, first_diff_offset, mismatch_bytes)
}

/// Q142.1 配对：在 crates 导出的写清单里找与 (设备, 偏移, 长度) 精确相等的那一条，返回它的下标。
/// **不按名字配对**（R3）：按名字配对是旧「量 5」的失败模式——两块设备在同一偏移各写不同内容时，
/// 按名字配对会分不清是哪一块设备，第九节 M108 钉的就是这一条。
fn find_matching_impl_write(impl_writes: &[ParsedWriteLine], device: u32, offset: u64, length: u64) -> Option<usize> {
    impl_writes.iter().position(|line| line.device == device && line.offset == offset && line.length == length)
}

/// crates 一侧配不上模型任何一次写的下标，按原始次序——Q142.1 的「配不上的两边都报」，不能只从模型一侧遍历
/// （第九节 M109：比对器只从模型一侧遍历会漏掉「crates 多写了一次」这一类差异）。
fn unmatched_crates_indices(impl_matched: &[bool]) -> Vec<usize> {
    impl_matched.iter().enumerate().filter(|(_, matched)| !**matched).map(|(index, _)| index).collect()
}

/// extent 上段叶序号（D8 已定项 14 第 401 行，唯一写法：写路径与读路径与单测共用）：第 k 片
/// （从 0 起）= inode ÷ 143（整除）。M138 的会红检查（143→144）落在这里。
fn extent_upper_leaf_index(inode: u64) -> u64 {
    inode / EXTENT_TREE_UPPER_LEAF_INODES
}

/// extent 树上段叶的 key 区间（D8 已定项 14 第 401 行，唯一写法：写路径与读路径共用）：第 k 片
/// （从 0 起，k = inode ÷ 143）罩 `[143k, 143k + 142]`，闭区间，第三分量恒 2^64 − 1（第七节 A7）。
fn extent_upper_leaf_positional_key_range(k: u64) -> (Vec<u8>, Vec<u8>) {
    let first_inode_in_span = EXTENT_TREE_UPPER_LEAF_INODES * k;
    let last_inode_in_span = first_inode_in_span + EXTENT_TREE_UPPER_LEAF_INODES - 1;
    let mut smallest = [0u8; 24];
    smallest[8..16].copy_from_slice(&first_inode_in_span.to_le_bytes());
    let mut largest = [0u8; 24];
    largest[8..16].copy_from_slice(&last_inode_in_span.to_le_bytes());
    largest[16..24].copy_from_slice(&u64::MAX.to_le_bytes());
    (smallest.to_vec(), largest.to_vec())
}

/// P2（第五节 5.3）：比对器分得出一个字节——在臂 N16 自己的镜像上，四个点各翻一位，喂给同一条比对路
/// （`compare_paired_write`/`find_matching_impl_write`，与真的跟 crates 比时用的是同一对函数），
/// 必须恰好命中那一个区域、那一个偏移、且只翻中 1 字节。四个点：
/// (a) 盘 0 那一份「罩盘 0 的层级 1 节点」第 0 条条目（唯一写法：稀疏，只有这一条）子指针的第一个字节；
/// (b) 根 max_key 末字节；(c) extent 上段叶第一条条目的标签字节；(d) journal 记录第 5 个点名项首字节。
fn run_p2_positive_control(emitter: &mut Emitter, arm_label: &str, writes: &[ModelWindowWrite]) {
    let allocation_key_start = INDEX_NODE_KEY_WIDTH_OFFSET + 1 + ALLOCATION_KEY_BYTES;
    let extent_entries_start = index_node_header_bytes(24) + NONCE_MAC_RESERVED_BYTES as usize;
    let journal_named_item_5_offset = JOURNAL_HEADER_BYTES as usize + 4 * JOURNAL_NAMED_ENTRY_BYTES as usize;
    let level1_entries_start = index_node_header_bytes(ALLOCATION_KEY_BYTES) + NONCE_MAC_RESERVED_BYTES as usize;
    // D8 已定项 14 第 395 行（唯一写法）：内部节点只写有孩子的格，这个装置的场景每个层级 1 节点只有一个孩子
    // ⇒ 条目区里第 0 条（唯一一条）就是它，子指针紧跟在 10 字节 key 之后。
    let level1_pointer_first_byte = Some(level1_entries_start + ALLOCATION_KEY_BYTES);

    let points: [(&str, &str, u32, Option<usize>); 4] = [
        ("a_level1_pointer_first_byte", "allocation_internal_of_device_0", 0, level1_pointer_first_byte),
        ("b_root_largest_key_last_byte", "allocation_root", 0, Some(allocation_key_start + ALLOCATION_KEY_BYTES - 1)),
        ("c_extent_leaf_entry_tag_byte", "extent_root", 0, Some(extent_entries_start + 24)),
        ("d_journal_named_item_5_first_byte", "journal_record", 0, Some(journal_named_item_5_offset)),
    ];

    for (point_name, target_region, target_device, offset_in_region) in points {
        let Some(offset_in_region) = offset_in_region else {
            emit(emitter, &format!("name=positive_control_p2 arm={arm_label} point={point_name} region={target_region} device={target_device} matches_injection=false reason=no_nonempty_cell_found"));
            continue;
        };
        let synthetic: Vec<ParsedWriteLine> = writes
            .iter()
            .map(|write| {
                let mut bytes = write.bytes.clone();
                if write.region == target_region && write.device == target_device {
                    assert!(offset_in_region < bytes.len(), "P2 注入点必须落在区域内（{point_name}）");
                    bytes[offset_in_region] ^= 0xFF;
                }
                ParsedWriteLine { device: write.device, offset: write.offset, length: bytes.len() as u64, sha256: sha256_hex(&bytes), hexadecimal: Some(hex_bytes(&bytes)) }
            })
            .collect();
        let mut flagged: Vec<(String, u32, u64, u64)> = Vec::new();
        for write in writes {
            let sha = sha256_hex(&write.bytes);
            if let Some(index) = find_matching_impl_write(&synthetic, write.device, write.offset, write.bytes.len() as u64) {
                let (equal, first_diff_offset, mismatch_bytes) = compare_paired_write(&write.bytes, &sha, &synthetic[index]);
                if !equal {
                    flagged.push((write.region.to_string(), write.device, first_diff_offset.unwrap_or(u64::MAX), mismatch_bytes.unwrap_or(0)));
                }
            }
        }
        let matches_injection =
            flagged.len() == 1 && flagged[0].0 == target_region && flagged[0].1 == target_device && flagged[0].2 == offset_in_region as u64 && flagged[0].3 == 1;
        emit(emitter, &format!(
            "name=positive_control_p2 arm={arm_label} point={point_name} region={target_region} device={target_device} injected_offset={offset_in_region} regions_flagged={} matches_injection={matches_injection}",
            flagged.len()
        ));
    }
}

/// Q142.2：按 (是否屏障, 是否 FUA) 从一份写清单文本里重建段大小——切法与 `split_into_segments`
/// 的 `fua_is_boundary = true` 同一条规则：FUA 写自己关掉一段，屏障关掉它前面还没关的那一段。
fn window_segment_sizes_from_dump(text: &str) -> Vec<usize> {
    enum Step {
        Write { is_fua: bool },
        Barrier,
    }
    let mut steps: Vec<(usize, Step)> = Vec::new();
    for line in text.lines() {
        let fields = parse_result_line(line);
        let Some(step) = fields.get("step").and_then(|value| value.parse::<usize>().ok()) else {
            continue;
        };
        if line.contains("name=device_region_bytes ") {
            let is_fua = fields.get("kind").map(String::as_str) == Some("write_fua");
            steps.push((step, Step::Write { is_fua }));
        } else if line.contains("name=window_barrier ") {
            steps.push((step, Step::Barrier));
        }
    }
    steps.sort_by_key(|(step, _)| *step);
    let mut segments = Vec::new();
    let mut current = 0usize;
    for (_, kind) in steps {
        match kind {
            Step::Write { is_fua } => {
                current += 1;
                if is_fua {
                    segments.push(current);
                    current = 0;
                }
            }
            Step::Barrier => {
                if current > 0 {
                    segments.push(current);
                    current = 0;
                }
            }
        }
    }
    if current > 0 {
        segments.push(current);
    }
    segments
}

fn catalog_unit_at_offset<'a>(catalog: &StructureCatalog<'a>, offset: u64) -> Option<&'a (SlotNumber, TransactionUnit, Vec<u8>)> {
    catalog.units.iter().find(|(slot, _, _)| slot.device_offset().0 == offset)
}

/// Q142.3 区域级写清单一行：非码 2 的区域（数据单元、打包容器、journal 记录、根记录、系统配置）
/// key 相关字段写 `none`。`writer=transaction` 恒定——R2 的窗口本来就只有这一条路径的写。
fn write_list_row(catalog: &StructureCatalog, step: usize, region: &str, device: u32, offset: u64, bytes: &[u8]) -> String {
    let sha = sha256_hex(bytes);
    let Some((slot, unit, _)) = catalog_unit_at_offset(catalog, offset) else {
        return format!(
            "name=write_list_row step={step} writer=transaction unit={region} class=other bytes={} slot=none tree=none device={device} offset={offset} level=none entries=none entry_width=none declared_length=none min_key=none max_key=none birth_sequence=none sha256={sha}",
            bytes.len()
        );
    };
    let (class, _) = unit.class_and_key_width();
    let class_name = match class {
        UNIT_CLASS_DATA => "data",
        UNIT_CLASS_PACKED => "packed",
        UNIT_CLASS_INDEX_NODE => "index_node",
        _ => "other",
    };
    if class == UNIT_CLASS_INDEX_NODE {
        if let Ok(header) = parse_index_node(bytes) {
            let entry_width = header.entries.first().map_or(0, Vec::len);
            return format!(
                "name=write_list_row step={step} writer=transaction unit={region} class={class_name} bytes={} slot={} tree={} device={device} offset={offset} level={} entries={} entry_width={entry_width} declared_length={} min_key={} max_key={} birth_sequence={} sha256={sha}",
                bytes.len(), slot.0, header.tree.0, header.level, header.entries.len(), header.entries.len() * entry_width,
                hex_bytes(&header.smallest_key), hex_bytes(&header.largest_key), header.birth_sequence.0
            );
        }
    }
    format!(
        "name=write_list_row step={step} writer=transaction unit={region} class={class_name} bytes={} slot={} tree={} device={device} offset={offset} level=none entries=none entry_width=none declared_length=none min_key=none max_key=none birth_sequence=none sha256={sha}",
        bytes.len(), slot.0, unit.tree().0
    )
}

/// Q142.17 码 2 节点的字段级行：头部按固定偏移（D8（核心索引结构） 已定项 11、
/// D18（块里携带什么信息） 已定项 18）逐字段列出；条目区（唯一写法：稀疏，D8 已定项 14 第 395 行）
/// 每条都列，不再有空格；条目区之后到单元末尾的补齐区另出一行——字段偏移连续覆盖整个 16384 字节，不留缝。
fn code2_field_rows(unit_tag: &str, device: u32, bytes: &[u8], header: &IndexNodeHeader) -> Vec<String> {
    let key_width = header.key_width;
    let header_end = 86 + 2 * key_width;
    let entries_start = header_end + NONCE_MAC_RESERVED_BYTES as usize;
    let header_fields: [(&str, usize, usize); 19] = [
        ("magic", 0, 4),
        ("format_version", 4, 6),
        ("class", 6, 7),
        ("flags", 7, 8),
        ("declared_length", 8, 10),
        ("seal_header_checksum", 10, 42),
        ("tree_id", 42, 50),
        ("level", 50, 51),
        ("key_width", 51, 52),
        ("smallest_key", 52, 52 + key_width),
        ("largest_key", 52 + key_width, 52 + 2 * key_width),
        ("birth_txg", 52 + 2 * key_width, 60 + 2 * key_width),
        ("fsid", 60 + 2 * key_width, 68 + 2 * key_width),
        ("instance", 68 + 2 * key_width, 72 + 2 * key_width),
        ("birth_sequence", 72 + 2 * key_width, 76 + 2 * key_width),
        ("payload_crc", 76 + 2 * key_width, 80 + 2 * key_width),
        ("reserved", 80 + 2 * key_width, 82 + 2 * key_width),
        ("entry_count", 82 + 2 * key_width, 84 + 2 * key_width),
        ("entry_width", 84 + 2 * key_width, 86 + 2 * key_width),
    ];
    let mut out: Vec<String> = header_fields
        .iter()
        .map(|(field, start, end)| {
            format!(
                "name=field_row unit={unit_tag} device_of_copy={device} field={field} offset_in_unit={start} width={} value={} source=clause",
                end - start,
                hex_bytes(&bytes[*start..*end])
            )
        })
        .collect();
    out.push(format!(
        "name=field_row unit={unit_tag} device_of_copy={device} field=unit_reserved offset_in_unit={header_end} width={} value={} source=clause",
        entries_start - header_end,
        hex_bytes(&bytes[header_end..entries_start])
    ));

    let entry_width = header.entries.first().map_or(0, Vec::len);
    for (index, entry) in header.entries.iter().enumerate() {
        let offset_in_unit = entries_start + index * entry_width;
        let key_bytes = &entry[..key_width.min(entry.len())];
        out.push(format!(
            "name=field_row unit={unit_tag} device_of_copy={device} field=entry_{index}_key offset_in_unit={offset_in_unit} width={} value={} source=clause",
            key_bytes.len(),
            hex_bytes(key_bytes)
        ));
        if entry.len() > key_width {
            out.push(format!(
                "name=field_row unit={unit_tag} device_of_copy={device} field=entry_{index}_payload offset_in_unit={} width={} value={} source=clause",
                offset_in_unit + key_width,
                entry.len() - key_width,
                hex_bytes(&entry[key_width..])
            ));
        }
    }
    // 条目区之后到单元末尾的补齐区（D18 已定项 18：恒 0 且参与载荷 CRC）——Q142.17 要求字段偏移连续覆盖
    // 整个 16384 字节，这一行补上条目区结束到单元末尾这一段。
    let entries_end = entries_start + header.entries.len() * entry_width;
    if entries_end < NODE_BYTES as usize {
        out.push(format!(
            "name=field_row unit={unit_tag} device_of_copy={device} field=entries_padding offset_in_unit={entries_end} width={} value={} source=clause",
            NODE_BYTES as usize - entries_end,
            hex_bytes(&bytes[entries_end..NODE_BYTES as usize])
        ));
    }
    out
}

// ───────────────────────── main：发结果行 ─────────────────────────

fn emit(emitter: &mut Emitter, body: &str) {
    println!("{}", emitter.emit_raw(body));
}

fn main() {
    let mut emitter = Emitter::new();
    // 2026-09-18 F1 诊断报告（research/prompts/e142-r11-f1-diagnosis.md）核过：跑前登记第一节第 1 条要求「同 3000 字节内容」，
    // 这一格此前与 `crates/singlefs-harness/src/scenario.rs::first_file_content()` 的公式不一样，量 5 的 11 处不等全部可以追到这一处；
    // 主 agent 判定改装置这一侧对齐到 `index % 251`（crates 一侧被 QEMU 读回核对、三份用例与层 0 两条流钉着）。
    let file_bytes: Vec<u8> = (0..3000u32).map(|index| u8::try_from(index % 251).expect("小于 256")).collect();
    emit(&mut emitter, &format!(
        "name=config devices=2 physical_block_bytes={PHYSICAL_BLOCK_BYTES} file_bytes={} fsid=fixed device_bytes={DEVICE_BYTES} journal_ring_bytes={JOURNAL_RING_BYTES} journal_ring_slots={JOURNAL_RING_SLOTS} in_flight_limit={JOURNAL_IN_FLIGHT_RECORD_LIMIT} unit_area_start_slot={UNIT_AREA_START_SLOT} unit_area_slots={UNIT_AREA_SLOTS} cluster_segment_slots={CLUSTER_SEGMENT_SLOTS} open_cluster_segment_start={OPEN_CLUSTER_SEGMENT_START_SLOT}",
        file_bytes.len()
    ));

    // 判据 1：宽度对账。
    // 「预期」那一列逐格抄自 `.claude/kb/layout/01-first-txn.md` 零到七（2026-09-14 用户定案之后的那一版），
    // 「实测」是装置自己的常量算出来的。两列不等就记一笔 `width_mismatches`。
    let width_rows: [(&str, u64, u64); 30] = [
        ("pointer_head", 50, POINTER_HEAD_BYTES),
        ("data_unit_header", 105, DATA_UNIT_HEADER_BYTES),
        ("packed_unit_header", 107, PACKED_UNIT_HEADER_BYTES),
        ("unit_reserved", 29, NONCE_MAC_RESERVED_BYTES),
        ("data_unit_header_with_reserved", 134, DATA_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES),
        ("packed_unit_header_with_reserved", 136, PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES),
        ("root_record", 457, ROOT_RECORD_BYTES),
        ("tree_table_entry", 200, TREE_TABLE_ENTRY_BYTES),
        ("journal_header", 311, JOURNAL_HEADER_BYTES),
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
        ("system_configuration", 481, SYSTEM_CONFIGURATION_BYTES),
        ("system_configuration_slot", 4096, SYSTEM_CONFIGURATION_SLOT_BYTES),
        ("allocation_record_tree_internal_entry", 96, ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES),
        ("extent_tree_upper_leaf_entry", 113, EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES),
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
    let output = publish_first_file(&mut recording, &parameters, &genesis, &file_bytes, instance, last_warm_up_record.as_deref(), FIRST_TRANSACTION_TXG);
    let warm_up_operations = &recording.operations[acquisition_operation_count..warm_up_operation_count];
    emit(&mut emitter, &format!(
        "name=warm_up publishes={} writes={} barriers={} fua={} first_transaction_txg={FIRST_TRANSACTION_TXG} last_warm_up_root_txg={}",
        warm_up_roots.len(),
        warm_up_operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(_))).count(),
        warm_up_operations.iter().filter(|operation| matches!(operation, RecordedOperation::Barrier)).count(),
        warm_up_operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(write) if write.is_fua())).count(),
        warm_up_roots.last().map_or(0, |root| root.checkpoint_txg.0)
    ));
    // P5（第五节 5.3）：窗口之前最后一次暖机的 5 次写与窗口之前的写数，与 crates 导出的 `before_window_summary` 对——
    // 两边窗口切点是不是同一处（不对就是 S4，比的不是同一段）。
    let before_window_operations = &recording.operations[..warm_up_operation_count];
    let before_window_writes: Vec<&WriteRequest> = before_window_operations
        .iter()
        .filter_map(|operation| match operation {
            RecordedOperation::Write(write) => Some(write),
            RecordedOperation::Barrier => None,
        })
        .collect();
    let last_five: Vec<String> = before_window_writes
        .iter()
        .rev()
        .take(5)
        .rev()
        .map(|write| format!("device{}@{}+{}", write.device.0, write.offset.0, write.bytes.len()))
        .collect();
    emit(&mut emitter, &format!(
        "name=before_window_summary side=model operations={} writes={} last_five={}",
        before_window_operations.len(), before_window_writes.len(), last_five.join(",")
    ));
    // 写清单只数第一个事务；层 0 枚举吃 mkfs 之后的全部操作（取号 + 暖机两次空发布 + 第一个事务）
    let transaction_operations = &recording.operations[warm_up_operation_count..];
    let post_mkfs_operations = &recording.operations[mkfs_operation_count..];
    let acquisition_operations = &recording.operations[mkfs_operation_count..acquisition_operation_count];
    // 段序列登记表（layout/01-first-txn 八）的输入：每条路径单独切段、再加整条流。mkfs 那一行不进层 0 枚举（G19），但段序列钉在这里。
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
    let full_report = recover(&full, JournalPolicy::Consult, ReaderMode::Clause);
    let content_matches = matches!(&full_report.outcome, RecoveryOutcome::FileRead { content, .. } if *content == file_bytes);
    emit(&mut emitter, &format!(
        "name=recover_full outcome={} root={} content_matches={content_matches} valid_records={} above_water={} applied={} verification_passed={} mapping_fallbacks={}",
        outcome_kind(&full_report.outcome), outcome_root(&full_report.outcome), full_report.journal.valid_records, full_report.journal.above_water, full_report.journal.prefix_applied, full_report.journal.verification_passed, full_report.mapping_fallbacks
    ));

    // ═════════ E142 第十四次跑（重跑登记 research/prompts/e142-r14-prereg.md）：头 311 与末条标志 ═════════
    // Q142.1 逐写摘要：从 mkfs 起到第一个事务写完为止每一次写（这一点为止 `recording` 只录了
    // mkfs + 取号 + 暖机 + 第一个事务，后面别的管线不再写进这个 `recording`）。与臂 O 那份产物（草稿目录
    // `/tmp/claude-1000/e142-r14/arm-o/`，源码 sha256 ddeb68c5bb5512f85cef201657c67021725d021fdc6e58bdbb357b331576be32，
    // 即 git HEAD 上这份文件改动之前的样子）用同一段代码、按 P4 的脚本对齐比。
    for (index, operation) in recording.operations.iter().enumerate() {
        if let RecordedOperation::Write(write) = operation {
            emit(&mut emitter, &format!(
                "name=header311_write_digest index={index} kind={} device={} offset={} length={} sha256={}",
                write.kind.tag(), write.device.0, write.offset.0, write.bytes.len(), sha256_hex(&write.bytes)
            ));
        }
    }
    emit(&mut emitter, &format!(
        "name=header311_write_digest_summary writes={}",
        recording.operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(_))).count()
    ));

    // 臂 H（对照）：写者开关扳到 0x00，其余不变；与臂 F（本运行默认 parameters，flag=1）用同一份几何。
    let h_parameters = parameters.clone().with_last_of_publish_flag(0x00);
    let (h_image, _h_output) = run_full_pipeline(&h_parameters, FIRST_TRANSACTION_TXG, &file_bytes);
    let f_image_for_hf = full.clone();

    // Q142.2 / P2：H、F 逐字节相减，每一段按落在哪条记录（w1/w4/t9）、记录内哪个字段打标签。
    let hf_differences = diff_pools(&h_image, &f_image_for_hf);
    let mut hf_fields_by_record_device: BTreeMap<(&'static str, u32), Vec<String>> = BTreeMap::new();
    for segment in &hf_differences {
        let (record_name, offset_in_record) = journal_record_name_for_offset(segment.offset).unwrap_or(("unregistered", segment.offset));
        let field = journal_header_field_tag(offset_in_record);
        emit(&mut emitter, &format!(
            "name=header311_flag_diff device={} offset={} length={} old={} new={} record={record_name} field={field} offset_in_record={offset_in_record}",
            segment.device, segment.offset, segment.length, hex_bytes(&segment.old), hex_bytes(&segment.new)
        ));
        hf_fields_by_record_device.entry((record_name, segment.device)).or_default().push(field.to_string());
    }
    for record_name in ["w1", "w4", "t9"] {
        for device_index in 0..u32::try_from(parameters.device_count).expect("设备数") {
            let fields = hf_fields_by_record_device.get(&(record_name, device_index)).cloned().unwrap_or_default();
            emit(&mut emitter, &format!("name=header311_flag_diff_fields record={record_name} device={device_index} fields={}", if fields.is_empty() { "none".to_string() } else { fields.join(",") }));
        }
    }
    emit(&mut emitter, &format!("name=header311_flag_diff_summary segments={}", hf_differences.len()));

    // P3：全盘相减分得出一个字节——在各自镜像的副本上把设备 0 的 t9 记录偏移 87（本次发布内序号首字节）、
    // 偏移 7（记录标志）各翻一位，与未翻的同一臂镜像相减；O 那一列由 `name=positive_control_change_count` 两行管（第五节 P3）。
    let t9_base = journal_record_offset(JournalCounter(3)).0;
    for (arm_name, image) in [("H", &h_image), ("F", &f_image_for_hf)] {
        for (field_offset, field_tag) in [(87u64, "本次发布内序号"), (JOURNAL_RECORD_FLAGS_OFFSET_U64, "记录标志")] {
            let mut flipped = image.clone();
            flip_byte(&mut flipped, DeviceIdentity(0), DeviceOffset(t9_base), field_offset);
            let segments = diff_pools(image, &flipped);
            let matches_expected = segments.len() == 1 && segments[0].device == 0 && segments[0].length == 1 && segments[0].offset == t9_base + field_offset;
            emit(&mut emitter, &format!(
                "name=header311_p3 arm={arm_name} field=t9_{field_tag} segments={} device={} offset={} length={} matches_expected={matches_expected}",
                segments.len(), segments.first().map_or(0, |segment| segment.device), segments.first().map_or(0, |segment| segment.offset), segments.first().map_or(0, |segment| segment.length)
            ));
        }
    }

    // Q142.6 附带：H、F 两臂 w4、t9 的反向链值（不当答案，只与第四节的线索并列给主 agent 看）。
    for (arm_name, image) in [("H", &h_image), ("F", &f_image_for_hf)] {
        let records = scan_journal(image, unit_fsid(&parameters.fsid));
        for (label, counter) in [("w1", 1u64), ("w4", 2u64), ("t9", 3u64)] {
            if let Some(record) = records.get(&(InstanceGeneration(1), JournalCounter(counter))) {
                emit(&mut emitter, &format!(
                    "name=header311_record_bytes arm={arm_name} record={label} device=0 flag_byte={:#04x} ordinal_bytes={} back_chain={}",
                    record.record_flags, record.ordinal_within_publish, record.back_chain
                ));
            }
        }
    }

    // P6：读者读了标志与序号——四对 (H/F × 甲/乙) 都跑，另加 F 的六份改写（第五节 5.2）。
    let p6_rows: [(&str, Option<fn(&mut JournalRecord)>, bool); 8] = [
        ("F", None, false),
        ("H", None, false),
        ("F_flag_0x03", Some(p6_mutate_flag_0x03), false),
        ("F_ordinal_0", Some(p6_mutate_ordinal_0), false),
        ("F_flag_0x00", Some(p6_mutate_flag_0x00), false),
        ("F_ordinal_2", Some(p6_mutate_ordinal_2), false),
        ("F_trailing_same_txg", None, true),
        ("F_txg_2", Some(p6_mutate_txg_2), false),
    ];
    for (label, mutate, append_trailing) in p6_rows {
        let row_parameters = if label == "H" { h_parameters.clone() } else { parameters.clone() };
        let row = run_p6_scenario(&row_parameters, &file_bytes, mutate, append_trailing);
        emit(&mut emitter, &format!(
            "name=header311_p6 mirror={label} primary_prefix_applied={} primary_readable={} clause_prefix_applied={} clause_readable={} clause_fatal={}",
            row.primary_prefix_applied, row.primary_readable, row.clause_prefix_applied, row.clause_readable, row.clause_fatal
        ));
    }

    // 第八节 8.2 G1（方向相反：本实例的第一条就是 t9）：装置原判据 5 那条路，按臂 H、臂 F 各跑一次、相减。
    let g1_control_h = PoolParameters::control_one_device_no_barriers().with_last_of_publish_flag(0x00);
    let g1_control_f = PoolParameters::control_one_device_no_barriers().with_last_of_publish_flag(0x01);
    let (mut g1_recording_h, g1_genesis_h) = mkfs(&g1_control_h);
    let _ = publish_first_file(&mut g1_recording_h, &g1_control_h, &g1_genesis_h, &file_bytes, InstanceGeneration(FIRST_INSTANCE_GENERATION), None, FIRST_TRANSACTION_TXG);
    let (mut g1_recording_f, g1_genesis_f) = mkfs(&g1_control_f);
    let _ = publish_first_file(&mut g1_recording_f, &g1_control_f, &g1_genesis_f, &file_bytes, InstanceGeneration(FIRST_INSTANCE_GENERATION), None, FIRST_TRANSACTION_TXG);
    let g1_differences = diff_pools(&g1_recording_h.pool, &g1_recording_f.pool);
    let mut g1_fields: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for segment in &g1_differences {
        let (record_name, offset_in_record) = journal_record_name_for_offset(segment.offset).unwrap_or(("unregistered", segment.offset));
        g1_fields.insert(format!("{record_name}:{}", journal_header_field_tag(offset_in_record)));
    }
    let g1_has_back_chain = g1_fields.iter().any(|field| field.ends_with(":back_chain"));
    emit(&mut emitter, &format!(
        "name=header311_geometry sample=G1 record=t9 fields={} back_chain_present={g1_has_back_chain}",
        if g1_fields.is_empty() { "none".to_string() } else { g1_fields.iter().cloned().collect::<Vec<_>>().join(",") }
    ));

    // G2（盘数 2 → 1，带暖机）：run_full_pipeline 在一块盘的参数上，臂 H、臂 F 各跑一次、相减。
    let g2_h = PoolParameters::control_one_device_no_barriers().with_last_of_publish_flag(0x00);
    let g2_f = PoolParameters::control_one_device_no_barriers().with_last_of_publish_flag(0x01);
    let (g2_image_h, _) = run_full_pipeline(&g2_h, FIRST_TRANSACTION_TXG, &file_bytes);
    let (g2_image_f, _) = run_full_pipeline(&g2_f, FIRST_TRANSACTION_TXG, &file_bytes);
    let g2_differences = diff_pools(&g2_image_h, &g2_image_f);
    let mut g2_fields_by_record: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();
    for segment in &g2_differences {
        let (record_name, offset_in_record) = journal_record_name_for_offset(segment.offset).unwrap_or(("unregistered", segment.offset));
        g2_fields_by_record.entry(record_name).or_default().push(journal_header_field_tag(offset_in_record).to_string());
    }
    for record_name in ["w1", "w4", "t9"] {
        let fields = g2_fields_by_record.get(record_name).cloned().unwrap_or_default();
        emit(&mut emitter, &format!("name=header311_geometry sample=G2 record={record_name} fields={}", if fields.is_empty() { "none".to_string() } else { fields.join(",") }));
    }

    // ═════════ E142 第十一次跑（重跑登记 research/prompts/e142-r11-prereg.md）：改动计数从 1 改成 3，盘上还有哪些字节跟着变 ═════════
    // 真实基线 M_B（臂 B，旧口径写 1）：同一次进程里再跑一遍整条路，parameters / file_bytes 与 M_A 一字不差。
    let (image_change_count_old, output_change_count_old) = run_full_pipeline(&parameters, 1, &file_bytes);
    let image_change_count_new = full.clone();
    let (root_region, root_ring_slot) = ring_target_for_publish(CheckpointTxg(FIRST_TRANSACTION_TXG));
    let root_device = parameters.region_devices[root_region as usize].0;
    let root_offset = ring_slot_offset(root_region, root_ring_slot).0;
    let catalog = build_structure_catalog(&output, root_device, root_offset);
    // 量 1：M_A 与 M_B 在两块盘全部字节上逐字节相减（分母是全部字节——第一节第 3 条；见 diff_pools 的推理注释）。
    let differences = diff_pools(&image_change_count_old, &image_change_count_new);
    emit(&mut emitter, &format!("name=change_count_run_pair devices={} file_bytes={} old_change_count=1 new_change_count={FIRST_TRANSACTION_TXG} segments={}", parameters.device_count, file_bytes.len(), differences.len()));
    let mut bullets = ChangeCountBullets::default();
    let mut structures_found: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    let mut unexplained = 0u64;
    for (index, segment) in differences.iter().enumerate() {
        let (structure, offset_in_structure) = classify_offset(&catalog, segment.device, segment.offset).unwrap_or(("unregistered", segment.offset));
        emit(&mut emitter, &format!(
            "name=change_count_diff index={index} device={} offset={} length={} old={} new={} structure={structure} offset_in_structure={offset_in_structure}",
            segment.device, segment.offset, segment.length, hex_bytes(&segment.old), hex_bytes(&segment.new)
        ));
        structures_found.insert(structure);
        bullets.record(structure, offset_in_structure, segment.length);
        // 量 3：谁的校验和/指针罩到它；归不了因的按 F6 记一行 gap，不作废。
        let (explained, reason) = if structure == "unregistered" { (false, "落在候选表任何一个结构区间之外".to_string()) } else { explain_offset(&catalog, structure, offset_in_structure) };
        let reason = reason.replace(' ', "_");
        if explained {
            emit(&mut emitter, &format!("name=diff_explained index={index} structure={structure} offset_in_structure={offset_in_structure} reason={reason}"));
        } else {
            unexplained += 1;
            emit(&mut emitter, &format!("name=gap id=CC{index} text=diff段{index}未归因_structure={structure}_offset={offset_in_structure}_detail={reason}"));
        }
    }
    emit(&mut emitter, &format!("name=diff_explained_summary segments={} unexplained={unexplained}", differences.len()));
    // 量 4：差异段所属结构去重之后，减去问题单点名的六处对应的四个结构标签（inode_leaf/inode_root 各扛两条bullet）。
    let expected_structures: std::collections::BTreeSet<&str> = ["inode_leaf", "inode_root", "tree_table", "root_record"].into_iter().collect();
    let extra: Vec<&str> = structures_found.iter().filter(|structure| !expected_structures.contains(*structure)).copied().collect();
    emit(&mut emitter, &format!("name=extra_structures structures={} count={}", if extra.is_empty() { "none".to_string() } else { extra.join(",") }, extra.len()));
    emit(&mut emitter, &format!(
        "name=change_count_bullets offset_88_covered={} leaf_checksum_covered={} pointer_to_leaf_checksum_covered={} inode_root_checksum_covered={} tree_table_covered={} root_record_checksum_covered={} all_covered={}",
        bullets.offset_88_covered, bullets.leaf_checksum_covered, bullets.pointer_to_leaf_checksum_covered, bullets.inode_root_checksum_covered, bullets.tree_table_covered, bullets.root_record_checksum_covered, bullets.all_covered()
    ));

    // 量 2：偏移 88 那 8 字节的值本身，两份镜像 × 两块盘各一行。
    let leaf_slot_base = SlotNumber(SLOT_INODE_LEAF).device_offset();
    let record_field_offset: usize = (PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES) as usize + 88; // 224：记录区 136 起 + 记录内偏移 88
    for (image, label, expected_change_count) in [(&image_change_count_new, "M_A", FIRST_TRANSACTION_TXG), (&image_change_count_old, "M_B", 1u64)] {
        for device_index in 0..parameters.device_count {
            let device = DeviceIdentity(u32::try_from(device_index).expect("设备数"));
            let sector = image.read(device, leaf_slot_base, PHYSICAL_BLOCK_BYTES as usize);
            let field_bytes = &sector[record_field_offset..record_field_offset + 8];
            let matches_expected = field_bytes == expected_change_count.to_le_bytes().as_slice();
            emit(&mut emitter, &format!("name=change_count_value image={label} device={device_index} value={} matches_expected={matches_expected}", hex_bytes(field_bytes)));
        }
    }

    // 阳性对照 A / B（第五节）：inode 叶偏移 88 首字节注入一次已知改动，比对器必须精确报出注入点（且只报出注入点）。
    // 对照 C 要 crates/ 实装的镜像；experiment-runner 的写范围没有 crates/**（agent-write-scope.tsv 第 4 行只许 implementation-writer 写），
    // 这个二进制造不出 M_C，下面只跑 A / B，C 那一行如实标 blocked（交回报告里详列）。
    for (image, label) in [(&image_change_count_new, "A"), (&image_change_count_old, "B")] {
        let mut mutated = image.clone();
        for device_index in 0..parameters.device_count {
            flip_byte(&mut mutated, DeviceIdentity(u32::try_from(device_index).expect("设备数")), leaf_slot_base, 88);
        }
        let detected_segments = diff_pools(image, &mutated);
        let injection_points: std::collections::BTreeSet<(u32, u64)> = (0..parameters.device_count).map(|device_index| (u32::try_from(device_index).expect("设备数"), leaf_slot_base.0 + 88)).collect();
        let detected_points: std::collections::BTreeSet<(u32, u64)> = detected_segments.iter().map(|segment| (segment.device, segment.offset)).collect();
        let matches_injection = detected_points == injection_points;
        emit(&mut emitter, &format!("name=positive_control_change_count control={label} segments={} matches_injection={matches_injection}", detected_segments.len()));
    }
    emit(&mut emitter, "name=positive_control_change_count control=C segments=0 matches_injection=false reason=blocked_no_crates_write_scope");

    // ═════════ E142 第十五次跑步④：Q142.1–Q142.8（重跑登记第六节；参数 1 = crates 导出路径，参数 2 = arm O 参照路径，都可选） ═════════
    // R2/R3：窗口 = `transaction_operations`（暖机最后一次写之后到发布结束）；配对键 (设备, 偏移, 长度)，不按名字配对。
    let impl_snapshot_path = std::env::args().nth(1);
    let impl_dump_text = impl_snapshot_path
        .as_deref()
        .map(|path| std::fs::read_to_string(path).unwrap_or_else(|error| panic!("Q142.1：读不到 crates 导出 {path}：{error}")));
    let impl_writes: Vec<ParsedWriteLine> = impl_dump_text
        .as_deref()
        .map(|text| parse_write_lines(text, "name=device_region_bytes "))
        .unwrap_or_default();
    let mut impl_matched = vec![false; impl_writes.len()];

    let model_writes = model_window_writes(&catalog, transaction_operations);

    // P2（第五节 5.3）：比对器分得出一个字节——臂 N16 自己的镜像上翻四个点各一位，必须恰好命中。
    run_p2_positive_control(&mut emitter, "arm_n16", &model_writes);

    // Q142.1：逐区域比对（主读法 P）。
    let mut equal_count = 0u64;
    let mut unequal_count = 0u64;
    let mut unmatched_model = 0u64;
    for write in &model_writes {
        let sha = sha256_hex(&write.bytes);
        emit(&mut emitter, &format!(
            "name=device_region_bytes step={} region={} device={} offset={} length={} sha256={sha} hexadecimal={}",
            write.step, write.region, write.device, write.offset, write.bytes.len(), hex_bytes(&write.bytes)
        ));
        emit(&mut emitter, &write_list_row(&catalog, write.step, write.region, write.device, write.offset, &write.bytes));

        let length = write.bytes.len() as u64;
        let matched_index = find_matching_impl_write(&impl_writes, write.device, write.offset, length);
        let Some(index) = matched_index else {
            unmatched_model += 1;
            emit(&mut emitter, &format!("name=impl_bytes_unmatched side=model step={} region={} device={} offset={} length={length}", write.step, write.region, write.device, write.offset));
            continue;
        };
        impl_matched[index] = true;
        let impl_line = &impl_writes[index];
        let (equal, first_diff_offset, mismatch_bytes) = compare_paired_write(&write.bytes, &sha, impl_line);
        if equal {
            equal_count += 1;
        } else {
            unequal_count += 1;
        }
        let fields_text = if equal {
            "none".to_string()
        } else {
            let (_, description) = explain_offset(&catalog, write.region, first_diff_offset.unwrap_or(0));
            description.replace(' ', "_")
        };
        emit(&mut emitter, &format!(
            "name=impl_bytes_equal step={} region={} device={} offset={} length={length} matched=true equal={equal} first_diff_offset={} mismatch_bytes={} fields={fields_text}",
            write.step, write.region, write.device, write.offset,
            first_diff_offset.map_or("none".to_string(), |value| value.to_string()),
            mismatch_bytes.map_or("unknown".to_string(), |value| value.to_string())
        ));
    }
    for index in unmatched_crates_indices(&impl_matched) {
        let line = &impl_writes[index];
        emit(&mut emitter, &format!("name=impl_bytes_unmatched side=crates device={} offset={} length={}", line.device, line.offset, line.length));
    }
    let unmatched_crates = unmatched_crates_indices(&impl_matched).len() as u64;
    emit(&mut emitter, &format!(
        "name=impl_bytes_equal_summary model_regions={} crates_regions={} matched={} equal={equal_count} unequal={unequal_count} unmatched_model={unmatched_model} unmatched_crates={unmatched_crates} snapshot_given={}",
        model_writes.len(), impl_writes.len(), equal_count + unequal_count, impl_snapshot_path.is_some()
    ));
    emit(&mut emitter, &format!(
        "name=write_list_summary writes={} barriers={} fua={} allocation_records_per_device={} allocation_record_leaves={}",
        write_count, barrier_count, fua_count,
        catalog.units.iter().find_map(|(_, unit, bytes)| matches!(unit, TransactionUnit::Allocation(AllocationNodeRole::Leaf { .. })).then(|| parse_index_node(bytes).map(|header| header.entries.len()).unwrap_or(0))).unwrap_or(0),
        catalog.units.iter().filter(|(_, unit, _)| matches!(unit, TransactionUnit::Allocation(AllocationNodeRole::Leaf { .. }))).count()
    ));

    // Q142.12（字段级归因，第二段）只在 Q142.11 不等、主 agent 续派第二段时跑；这一段（第一段）到此为止，
    // 「不等」时第 4 行只报「不等」，字段级归因交给第二段。

    // Q142.13：段与先后——两边各自重建段大小，`fua_is_boundary = true`（与 `split_into_segments` 同一条切法）。
    let (_, model_transaction_segments) = split_into_segments(transaction_operations, true);
    let model_segment_sizes: Vec<usize> = model_transaction_segments.iter().map(Vec::len).collect();
    emit(&mut emitter, &format!(
        "name=window_segments side=model sizes={}",
        model_segment_sizes.iter().map(ToString::to_string).collect::<Vec<_>>().join("+")
    ));
    if let Some(text) = impl_dump_text.as_deref() {
        let crates_segment_sizes = window_segment_sizes_from_dump(text);
        emit(&mut emitter, &format!(
            "name=window_segments side=crates sizes={}",
            crates_segment_sizes.iter().map(ToString::to_string).collect::<Vec<_>>().join("+")
        ));
        let consistent = model_segment_sizes == crates_segment_sizes && unmatched_model == 0 && unmatched_crates == 0;
        emit(&mut emitter, &format!("name=window_segments_summary consistent={consistent}"));
    }

    // Q142.17：四种按位置寻址的结构（分配记录树的叶/内部节点/根、extent 树上段叶）的字段级行，每种代表一份（device_of_copy=0；
    // 两盘内容相同，另一份留给 P2 的翻位单独核）。
    let mut field_row_structures = 0u64;
    for (_, unit, bytes) in catalog.units {
        let is_new_structure = matches!(unit, TransactionUnit::Allocation(_) | TransactionUnit::ExtentRoot);
        if !is_new_structure {
            continue;
        }
        if let Ok(header) = parse_index_node(bytes) {
            field_row_structures += 1;
            for row in code2_field_rows(unit.descriptive_tag(), 0, bytes, &header) {
                emit(&mut emitter, &row);
            }
        }
    }
    emit(&mut emitter, &format!("name=field_row_summary structures={field_row_structures} policy=sparse_every_entry_plus_padding_row"));

    // Q142.19：臂 N15 → N16 哪些区域变了，按 (设备, 偏移, 长度) 配对（R3：不按区域名——节点数变了名字配不上）。
    // 第二个命令行参数这一次改指臂 N15 的产物（步 ① 现编现跑存下的那一份），不再是第十四次跑 arm O 的历史留存。
    if let Some(path) = std::env::args().nth(2) {
        let arm_n15_text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("Q142.19：读不到臂 N15 参照 {path}：{error}"));
        let arm_n15_writes = parse_write_lines(&arm_n15_text, "name=device_region_bytes ");
        let mut arm_n15_matched = vec![false; arm_n15_writes.len()];
        let mut matched_count = 0u64;
        let mut changed_count = 0u64;
        for write in &model_writes {
            let length = write.bytes.len() as u64;
            let Some(index) = find_matching_impl_write(&arm_n15_writes, write.device, write.offset, length) else {
                emit(&mut emitter, &format!("name=old_new_region_only_in role=N16 region={} device={} offset={} length={length}", write.region, write.device, write.offset));
                continue;
            };
            arm_n15_matched[index] = true;
            matched_count += 1;
            let sha = sha256_hex(&write.bytes);
            let changed = arm_n15_writes[index].sha256 != sha;
            if changed {
                changed_count += 1;
            }
            emit(&mut emitter, &format!("name=old_new_region region={} device={} changed={changed}", write.region, write.device));
        }
        for (index, matched) in arm_n15_matched.iter().enumerate() {
            if !matched {
                let line = &arm_n15_writes[index];
                emit(&mut emitter, &format!("name=old_new_region_only_in role=N15 device={} offset={} length={}", line.device, line.offset, line.length));
            }
        }
        emit(&mut emitter, &format!("name=old_new_region_summary matched={matched_count} changed={changed_count} unchanged={}", matched_count - changed_count));
    }

    // 第八节 G3：形状函数（纯函数）在五个几何点上各算一次，不跑整条写路（第七节 B13）。
    {
        let half_slot_device_bytes: u64 = 84 * allocation_record_tree_span_at_level(1) * SLOT_BYTES + SLOT_BYTES / 2;
        for (label, device_count, device_bytes) in [
            ("main", 2u64, DEVICE_BYTES),
            ("one_device", 1, DEVICE_BYTES),
            ("8GiB", 2, 8u64 << 30),
            ("256GiB", 2, 256u64 << 30),
            ("half_slot", 2, half_slot_device_bytes),
        ] {
            let slots = slots_of_device_bytes(device_bytes);
            let device_slots_vector = vec![slots; device_count as usize];
            let root_level = allocation_record_tree_root_level(&device_slots_vector);
            let cells_per_device = allocation_record_tree_cells_per_device(slots, root_level);
            let nodes = allocation_record_tree_first_transaction_node_count(device_count, root_level);
            emit(&mut emitter, &format!(
                "name=g3_shape point={label} device_count={device_count} device_bytes={device_bytes} slots={slots} root_level={root_level} root_cells_per_device={cells_per_device} first_txn_allocation_nodes={nodes}"
            ));
        }
    }

    // 第八节 G4：一盘，走整条写路，与 crates 的一盘导出比（第三个命令行参数，可选）。
    if let Some(path) = std::env::args().nth(3) {
        let one_device_dump_text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("G4：读不到一盘导出 {path}：{error}"));
        let g4_impl_writes = parse_write_lines(&one_device_dump_text, "name=device_region_bytes ");
        let g4_parameters = PoolParameters::control_one_device_no_barriers();
        let g4_writes = model_window_writes_for_parameters(&g4_parameters, &file_bytes);
        let mut g4_matched = vec![false; g4_impl_writes.len()];
        let mut g4_equal = 0u64;
        let mut g4_unequal = 0u64;
        for write in &g4_writes {
            let sha = sha256_hex(&write.bytes);
            let length = write.bytes.len() as u64;
            let Some(index) = find_matching_impl_write(&g4_impl_writes, write.device, write.offset, length) else {
                emit(&mut emitter, &format!("name=g4_bytes_unmatched side=model region={} device={} offset={} length={length}", write.region, write.device, write.offset));
                continue;
            };
            g4_matched[index] = true;
            let (equal, first_diff_offset, mismatch_bytes) = compare_paired_write(&write.bytes, &sha, &g4_impl_writes[index]);
            if equal {
                g4_equal += 1;
            } else {
                g4_unequal += 1;
            }
            emit(&mut emitter, &format!(
                "name=g4_bytes_equal region={} device={} offset={} length={length} equal={equal} first_diff_offset={} mismatch_bytes={}",
                write.region, write.device, write.offset,
                first_diff_offset.map_or("none".to_string(), |value| value.to_string()),
                mismatch_bytes.map_or("unknown".to_string(), |value| value.to_string())
            ));
        }
        for (index, matched) in g4_matched.iter().enumerate() {
            if !matched {
                let line = &g4_impl_writes[index];
                emit(&mut emitter, &format!("name=g4_bytes_unmatched side=crates device={} offset={} length={}", line.device, line.offset, line.length));
            }
        }
        emit(&mut emitter, &format!(
            "name=g4_bytes_equal_summary model_regions={} crates_regions={} equal={g4_equal} unequal={g4_unequal}",
            g4_writes.len(), g4_impl_writes.len()
        ));
    } else {
        emit(&mut emitter, "name=g4_bytes_equal_summary skipped=true reason=no_one_device_dump_path_given");
    }

    // 量 8：几何敏感性——取样点一（盘数 2→1）与取样点二（换一个值 1→4）。
    let one_device_parameters = PoolParameters::control_one_device_no_barriers();
    let (one_device_image_new, one_device_output_new) = run_full_pipeline(&one_device_parameters, FIRST_TRANSACTION_TXG, &file_bytes);
    let (one_device_image_old, _) = run_full_pipeline(&one_device_parameters, 1, &file_bytes);
    let (one_device_root_region, one_device_root_slot) = ring_target_for_publish(CheckpointTxg(FIRST_TRANSACTION_TXG));
    let one_device_root_device = one_device_parameters.region_devices[one_device_root_region as usize].0;
    let one_device_root_offset = ring_slot_offset(one_device_root_region, one_device_root_slot).0;
    let one_device_catalog = build_structure_catalog(&one_device_output_new, one_device_root_device, one_device_root_offset);
    let one_device_differences = diff_pools(&one_device_image_old, &one_device_image_new);
    let one_device_structures = structure_name_set(&one_device_differences, &one_device_catalog);
    let main_structures = structure_name_set(&differences, &catalog);
    let one_device_stable = structure_sets_match(&one_device_structures, &main_structures);
    emit(&mut emitter, &format!("name=geometry_sensitivity point=G1 devices={} segments={} structures={} stable={one_device_stable}", one_device_parameters.device_count, one_device_differences.len(), one_device_structures.len()));
    if !one_device_stable {
        emit(&mut emitter, &format!("name=geometry_sensitivity_detail point=G1 main_only={} sample_only={}", main_structures.difference(&one_device_structures).copied().collect::<Vec<_>>().join(","), one_device_structures.difference(&main_structures).copied().collect::<Vec<_>>().join(",")));
    }

    let (larger_change_count_image_new, larger_change_count_output_new) = run_full_pipeline(&parameters, 4, &file_bytes);
    let larger_change_count_catalog = build_structure_catalog(&larger_change_count_output_new, root_device, root_offset);
    let larger_change_count_differences = diff_pools(&image_change_count_old, &larger_change_count_image_new);
    let larger_change_count_positions = structure_position_set(&larger_change_count_differences, &larger_change_count_catalog);
    let main_positions = structure_position_set(&differences, &catalog);
    let larger_change_count_stable = larger_change_count_positions == main_positions;
    emit(&mut emitter, &format!("name=geometry_sensitivity point=G2 devices={} segments={} positions={} stable={larger_change_count_stable}", parameters.device_count, larger_change_count_differences.len(), larger_change_count_positions.len()));
    if !larger_change_count_stable {
        let main_only: Vec<String> = main_positions.difference(&larger_change_count_positions).map(|(device, structure, offset)| format!("{device}:{structure}:{offset}")).collect();
        let sample_only: Vec<String> = larger_change_count_positions.difference(&main_positions).map(|(device, structure, offset)| format!("{device}:{structure}:{offset}")).collect();
        emit(&mut emitter, &format!("name=geometry_sensitivity_detail point=G2 main_only={} sample_only={}", if main_only.is_empty() { "none".to_string() } else { main_only.join(",") }, if sample_only.is_empty() { "none".to_string() } else { sample_only.join(",") }));
    }
    // output_change_count_old 只用于量 2/量 8 的旁证；避免「构造了却没用」的 dead_code 告警。
    let _ = &output_change_count_old.record.counter;

    // 判据 4：层 0 主臂。基线是 mkfs 之后的池（事务的写全没持久那一态）。
    // Q142.9（重跑登记第六节：「第一段不跑层 0…`name=layer0`一行写`skipped=true`」）：新段序列下整轮枚举
    // 是 67108885 个状态（第七节 B8，单测 `layer0_state_count_is_67108885_with_zero_violations` 已挪出
    // `cargo test`、标 `#[ignore]`）；这一段只求第 1、2 行够判，不消费它，`main()` 这里同样跳过
    // `enumerate_layer0`，只报段大小与闭式（这两个是纯函数，便宜）。
    let (writes, segments) = split_into_segments(post_mkfs_operations, true);
    let segment_sizes: Vec<String> = segments.iter().map(|segment| segment.len().to_string()).collect();
    let closed_form = closed_form_state_count(&segments);
    let _ = &writes; // 只为算出 `segments`；枚举本身跳过，见上。
    emit(&mut emitter, &format!(
        "name=layer0 arm=settled_two_devices segments={} states=not_run closed_form={closed_form} skipped=true reason=附带_够判后未跑",
        segment_sizes.join("+")
    ));
    let (_, segments_fua_free) = split_into_segments(post_mkfs_operations, false);
    emit(&mut emitter, &format!("name=layer0_fua_not_boundary segments={} closed_form={}", segments_fua_free.iter().map(|segment| segment.len().to_string()).collect::<Vec<_>>().join("+"), closed_form_state_count(&segments_fua_free)));
    emit(&mut emitter, "name=journal_effect arm=settled_two_devices states=not_run differing_states=not_run skipped=true reason=附带_够判后未跑");
    // S5（第八节）：(臂 F, 读者乙) 的层 0 里，第五节 5.1 读者乙 ①–④ 任一分支的计数——依赖上面跳过的枚举，同样不跑。
    emit(&mut emitter, "name=header311_reader_branch_counts skipped=true reason=附带_够判后未跑（依赖层0主臂枚举）");

    // 判据 5：阳性对照——一块盘、不放屏障，oracle 必须分得出「根在而单元不在」。
    let control = PoolParameters::control_one_device_no_barriers();
    let (mut control_recording, control_genesis) = mkfs(&control);
    let control_mkfs_operation_count = control_recording.operations.len();
    let _ = publish_first_file(&mut control_recording, &control, &control_genesis, &file_bytes, InstanceGeneration(FIRST_INSTANCE_GENERATION), None, FIRST_TRANSACTION_TXG);
    let (control_base, _) = mkfs(&control);
    let (control_writes, control_segments) = split_into_segments(&control_recording.operations[control_mkfs_operation_count..], false);
    let control_closed_form = closed_form_state_count(&control_segments);
    let control_tally = enumerate_layer0(&control_base.pool, &control_writes, &control_segments, &file_bytes, ReaderMode::Clause);
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
        "name=instances mkfs_instance={MKFS_INSTANCE_GENERATION} first_writable_mount_instance={FIRST_INSTANCE_GENERATION} mkfs_system_configuration_generation={SYSTEM_CONFIGURATION_GENERATION_AT_MKFS} acquisition_system_configuration_generation={SYSTEM_CONFIGURATION_GENERATION_AT_INSTANCE_ACQUISITION} transaction_system_configuration_generation={} transaction_system_configuration_slot={}",
        system_configuration_write_for_publish(CheckpointTxg(FIRST_TRANSACTION_TXG)).0,
        system_configuration_write_for_publish(CheckpointTxg(FIRST_TRANSACTION_TXG)).1
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
    let system_configuration_slot = recording.pool.read(DeviceIdentity(0), DeviceOffset(SYSTEM_CONFIGURATION_SLOT_OFFSETS[0]), SYSTEM_CONFIGURATION_SLOT_BYTES as usize);
    let feature_bits = &system_configuration_slot[SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET..SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET + 3 * FEATURE_BITMAP_BYTES];
    let mut unknown_incompat_slot = system_configuration_slot.clone();
    unknown_incompat_slot[SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET] = 0x03;
    let mut no_layout_slot = system_configuration_slot.clone();
    no_layout_slot[SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET] = 0x00;
    emit(&mut emitter, &format!(
        "name=feature_bits incompat_byte0={:#04x} incompat_rest_zero={} compat_ro_zero={} compat_zero={} refuses_unknown_incompat={} refuses_missing_layout_bit={}",
        feature_bits[0],
        feature_bits[1..FEATURE_BITMAP_BYTES].iter().all(|byte| *byte == 0),
        feature_bits[FEATURE_BITMAP_BYTES..2 * FEATURE_BITMAP_BYTES].iter().all(|byte| *byte == 0),
        feature_bits[2 * FEATURE_BITMAP_BYTES..].iter().all(|byte| *byte == 0),
        !incompat_bits_are_mountable(&unknown_incompat_slot),
        !incompat_bits_are_mountable(&no_layout_slot),
    ));

    // Q142.9：层 0 主臂那两格（原判据 4）不跑，写 `not_run`；write_count / control 的门槛改按第七节
    // B5（29 写）、B10（8192 个状态）的新锚点算，不再钉旧布局的 21 / 2048（其余字段照报，不设门槛）。
    emit(&mut emitter, &format!(
        "name=verdict width_mismatches={width_mismatches} write_list_ok={} recover_full_ok={content_matches} layer0_states_ok=not_run layer0_violations=not_run control_states_ok={} control_violations_ok={} journal_differing_states=not_run",
        write_count == 29 && barrier_count == 2 && fua_count == 1,
        control_tally.states == control_closed_form && control_closed_form == 8192,
        control_tally.violations == control_expected_violations,
    ));
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_file() -> Vec<u8> {
        // 与 main() 里的 file_bytes 同一个公式（2026-09-18 对齐到 crates 的 `index % 251`，见 main() 里的注释）。
        (0..3000u32).map(|index| u8::try_from(index % 251).expect("小于 256")).collect()
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
        let output = publish_first_file(&mut recording, &parameters, &genesis, &sample_file(), instance, last_warm_up_record.as_deref(), FIRST_TRANSACTION_TXG);
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
    fn widths_equal_the_byte_table_and_the_root_record_is_457() {
        // 每一行左边是字节表零到七写死的绝对值，右边是装置自己算出来的——不是几个常量互相比。
        assert_eq!(ROOT_RECORD_BYTES, 457);
        assert_eq!(ROOT_CHECKSUM_OFFSET, 138, "自证校验和在根记录里的偏移");
        assert_eq!(ROOT_CHECKSUM_OFFSET as u64 + WIDE_CHECKSUM_BYTES + 3 * NODE_POINTER_BYTES + 1 + 12 + 16, ROOT_RECORD_BYTES);
        assert_eq!(POINTER_HEAD_BYTES, 50);
        assert_eq!(POINTER_HEAD_BYTES + 2 * LOC_ENTRY + 4 + 4, NODE_POINTER_BYTES);
        assert_eq!(POINTER_HEAD_BYTES + 2 * LOC_ENTRY + 10, DATA_POINTER_BYTES);
        assert_eq!(NODE_POINTER_BYTES, 86);
        assert_eq!(DATA_POINTER_BYTES, 88);
        assert_eq!(JOURNAL_HEADER_CHECKSUM_OFFSET, 46);
        assert_eq!(JOURNAL_HEADER_TEN_FIELD_BYTES, 78);
        assert_eq!(JOURNAL_HEADER_CHECKSUM_OFFSET as u64 + WIDE_CHECKSUM_BYTES, JOURNAL_HEADER_TEN_FIELD_BYTES);
        // 事务号 8 + 提交标记 1 + 本次发布内序号 4（已定项 4，2026-09-24 加）+ 反向链 4 + 载荷校验和 4 + 新根段 188 + fsid 8 + MAC 16 = 233；78 + 233 = 311。
        assert_eq!(JOURNAL_HEADER_TEN_FIELD_BYTES + 8 + 1 + 4 + 4 + 4 + JOURNAL_NEW_ROOT_SEGMENT_BYTES + 8 + 16, JOURNAL_HEADER_BYTES);
        assert_eq!(JOURNAL_HEADER_BYTES, 311);
        assert_eq!(JOURNAL_NEW_ROOT_SEGMENT_BYTES, 188);
        assert_eq!(JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES, 3785, "4096 − 311");
        assert_eq!((JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES) / JOURNAL_NAMED_ENTRY_BYTES, 67, "4096 的记录装 67 个点名项");
        // A2（第七节）：字段起点逐个钉住，记录标志 7、事务号 78、提交标记 86、本次发布内序号 87、反向链 91、
        // 载荷校验和 95、新根段 99、fsid 287、MAC 295、头末 311（D23 已定项 4）。
        assert_eq!(JOURNAL_RECORD_FLAGS_OFFSET, 7);
        assert_eq!(JOURNAL_HEADER_TEN_FIELD_BYTES, 78, "事务号起点");
        assert_eq!(JOURNAL_HEADER_TEN_FIELD_BYTES + 8, 86, "提交标记起点");
        assert_eq!(JOURNAL_HEADER_TEN_FIELD_BYTES + 8 + 1, 87, "本次发布内序号起点");
        assert_eq!(JOURNAL_HEADER_TEN_FIELD_BYTES + 8 + 1 + 4, 91, "反向链起点");
        assert_eq!(JOURNAL_HEADER_TEN_FIELD_BYTES + 8 + 1 + 4 + 4, 95, "载荷校验和起点");
        assert_eq!(JOURNAL_HEADER_TEN_FIELD_BYTES + 8 + 1 + 4 + 4 + 4, 99, "新根段起点");
        assert_eq!(99 + JOURNAL_NEW_ROOT_SEGMENT_BYTES, 287, "fsid 起点");
        assert_eq!(287 + 8, 295, "MAC 起点");
        assert_eq!(SYSTEM_CONFIGURATION_CHECKSUM_OFFSET, 155);
        assert_eq!(SYSTEM_CONFIGURATION_FSID_OFFSET, 102);
        assert_eq!(SYSTEM_CONFIGURATION_BYTES, 481);
        assert_eq!(SYSTEM_CONFIGURATION_SLOT_BYTES, 4096, "系统配置槽宽是格式常量（D22 已定项 2，2026-09-14 三方论证后按主 agent 推荐值）");
        assert_eq!(SYSTEM_CONFIGURATION_SLOT_BYTES - SYSTEM_CONFIGURATION_BYTES, 3615, "481 的系统配置在 4096 槽里余 3615");
        assert_eq!(SYSTEM_CONFIGURATION_SLOT_OFFSETS[1] - SYSTEM_CONFIGURATION_SLOT_OFFSETS[0], SYSTEM_CONFIGURATION_SLOT_BYTES, "槽距与槽宽同值 ⇒ 两个槽首尾相接、不重叠");
        assert_eq!(SYSTEM_CONFIGURATION_TAIL_OFFSET as u64 + 8 + 4, SYSTEM_CONFIGURATION_BYTES);
        assert_eq!(MAPPING_KEY_BYTES, 27);
        assert_eq!(MAPPING_KEY_BYTES + 2 * LOC_ENTRY, MAPPING_ENTRY_BYTES);
        assert_eq!(MAPPING_ENTRY_BYTES, 55, "映射条目一律 55（D19 已定项 10，2026-09-14 用户定案）");
        assert_eq!(MAPPING_KEY_NODE_SIGNIFICANT_BYTES + 2, MAPPING_KEY_BYTES, "码 2 / 码 3 的 key 末尾补两个零");
        assert_eq!(ALLOCATION_KEY_BYTES as u64 + 10, ALLOCATION_RECORD_BYTES);
        assert_eq!(24 + DATA_POINTER_BYTES, EXTENT_LEAF_RECORD_BYTES);
        assert_eq!(EXTENT_LEAF_RECORD_BYTES, 112);
        assert_eq!(8 + 26 + NODE_POINTER_BYTES, INODE_INTERNAL_ENTRY);
        assert_eq!(INODE_INTERNAL_ENTRY, 120);
        assert_eq!(TREE_TABLE_ENTRY_BYTES, 200);
        assert_eq!(8 + 2 + 2 + 2 + NODE_POINTER_BYTES + 8 + 8 + 8 + 76, TREE_TABLE_ENTRY_BYTES);
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
        assert_eq!((NODE_BYTES as usize - with_reserved(TREE_TABLE_KEY_WIDTH)) / TREE_TABLE_ENTRY_BYTES as usize, 81);
        // inode 树内部节点一层装几条：头 131、条目 120。
        assert_eq!((NODE_BYTES as usize - with_reserved(8)) / INODE_INTERNAL_ENTRY as usize, 135);
    }

    /// key 宽住偏移 51（D18 已定项 18，2026-09-14 用户定案）：解析器不用先知道这是哪棵树就找得到头末端。
    /// 这是 G2 收口那一格的会红检查——把 key 宽写进别的偏移，解析当场失败。
    #[test]
    fn the_key_width_field_sits_at_offset_51_so_the_header_locates_itself() {
        let BuiltPool { output, .. } = built_pool();
        // E142 第十五次跑：索引 0..12 是 t1..t12（第七节 B2）；分配记录树五个节点（索引 4..9）
        // 一律 key 宽 10（D3 已定项 11：位置 key 与分配记录 key 同构，不分叶或内部）。
        for (index, expected_key_width) in [(1usize, 24usize), (3, 8), (4, ALLOCATION_KEY_BYTES), (5, ALLOCATION_KEY_BYTES), (6, ALLOCATION_KEY_BYTES), (7, ALLOCATION_KEY_BYTES), (8, ALLOCATION_KEY_BYTES), (9, 22), (10, MAPPING_KEY_BYTES as usize), (11, TREE_TABLE_KEY_WIDTH)] {
            let unit = &output.units_by_slot[index].2;
            assert_eq!(usize::from(unit[INDEX_NODE_KEY_WIDTH_OFFSET]), expected_key_width, "偏移 51 那一字节就是 key 宽（索引 {index}）");
            let node = parse_index_node(unit).expect("自描述的 key 宽足以定出头末端");
            assert_eq!(node.key_width, expected_key_width);
        }
        assert_eq!(INDEX_NODE_KEY_WIDTH_OFFSET, 51);
        // 把 key 宽改成别的值：头末端跟着移，头校验和当场不过（索引 10 是映射树）。
        let mut tampered = output.units_by_slot[10].2.clone();
        tampered[INDEX_NODE_KEY_WIDTH_OFFSET] = 25;
        assert_eq!(parse_index_node(&tampered).unwrap_err(), UnitError::HeaderChecksum);
        // 自述一个大到让头越出这段字节的 key 宽：判结构错，不许下标越界。
        let mut absurd = output.units_by_slot[10].2.clone();
        absurd[INDEX_NODE_KEY_WIDTH_OFFSET] = 255;
        absurd.truncate(200);
        assert_eq!(parse_index_node(&absurd).unwrap_err(), UnitError::Structure("自述的 key 宽让头越出单元"));
    }

    #[test]
    /// E142 第十五次跑：29 次写、2 道屏障、1 道 FUA（第七节 B5「window_writes=29 segments=[24,2,1,2]」）——
    /// 12 个单元 × 2 盘 = 24（分配记录树从 1 个节点拆成 5 个之后 t1..t12），其余不变。
    fn transaction_issues_21_writes_2_barriers_1_fua_in_the_settled_order() {
        let BuiltPool { recording, output, warm_up_operation_count, .. } = built_pool();
        let operations = &recording.operations[warm_up_operation_count..];
        let steps: Vec<&'static str> = operations.iter().map(|operation| RecordedStepKind::of(operation).tag()).collect();
        assert_eq!(steps.iter().filter(|tag| **tag != "barrier").count(), 29);
        assert_eq!(steps.iter().filter(|tag| **tag == "barrier").count(), 2);
        assert_eq!(steps.iter().filter(|tag| **tag == "root_record_fua").count(), 1);
        assert_eq!(operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(write) if write.is_fua())).count(), 1, "FUA 只由步骤种类决定，只有根槽那一步是");
        assert_eq!(steps[24], "barrier");
        assert_eq!(steps[27], "barrier");
        assert_eq!(steps[28], "root_record_fua");
        assert_eq!(output.record.named.len(), 12, "t1..t12（第七节 B6「named=12」）");
        assert_eq!(output.record.named.iter().map(|named| named.locations.len()).sum::<usize>(), 24, "第七节 B6「location_entries=24」");
        // D3 已定项 10 ⑤（2026-09-14 用户定案）：t1 用户数据取不在开放聚簇段里的最低 32768 对齐空槽对；
        // t2..t12 从开放段 [50240, 50304) 按「树 ID 升序、树内先叶后根、映射树倒数第二、树表最末」bump（分配记录树内按
        // 「先叶后根、同层设备升序」），而 t3 是码 3 容器、要起点 32768 对齐 ⇒ 跳过 50241（第七节 B2）。
        let slots: Vec<u64> = output.units_by_slot.iter().map(|(slot, _, _)| slot.0).collect();
        assert_eq!(slots, vec![50180, 50240, 50242, 50244, 50245, 50246, 50247, 50248, 50249, 50250, 50251, 50252]);
        assert_eq!(SLOT_INODE_LEAF % (DATA_UNIT_BYTES / SLOT_BYTES), 0, "码 3 容器起点也 32768 对齐");
        assert!(!slots.contains(&SLOT_SKIPPED_BY_ALIGNMENT), "槽 50241 没人占");
        assert_eq!(SLOT_DATA_UNIT % (DATA_UNIT_BYTES / SLOT_BYTES), 0, "数据单元起点 32768 对齐");
        assert!(SLOT_DATA_UNIT < OPEN_CLUSTER_SEGMENT_START_SLOT, "用户数据不进开放聚簇段");
        assert_eq!(OPEN_CLUSTER_SEGMENT_START_SLOT % CLUSTER_SEGMENT_SLOTS, 0, "开放段 64 槽对齐");
        for slot in [
            SLOT_EXTENT_ROOT,
            SLOT_INODE_LEAF,
            SLOT_INODE_ROOT,
            SLOT_ALLOCATION_LEAF_DEVICE_0,
            SLOT_ALLOCATION_LEAF_DEVICE_1,
            SLOT_ALLOCATION_INTERNAL_DEVICE_0,
            SLOT_ALLOCATION_INTERNAL_DEVICE_1,
            SLOT_ALLOCATION_ROOT,
            SLOT_ACCOUNTING_ROOT,
            SLOT_MAPPING_ROOT,
            SLOT_TREE_TABLE_FIRST_PUBLISH,
        ] {
            assert!((OPEN_CLUSTER_SEGMENT_START_SLOT..OPEN_CLUSTER_SEGMENT_START_SLOT + CLUSTER_SEGMENT_SLOTS).contains(&slot), "提交内生块住开放聚簇段");
        }
    }

    #[test]
    fn mkfs_seeds_three_generation_zero_roots_readable_from_all_regions() {
        let parameters = PoolParameters::settled_two_devices();
        let (recording, genesis) = mkfs(&parameters);
        let system_configuration = choose_system_configuration(&recording.pool).expect("系统配置");
        let mut readable = 0;
        for region in 0..RING_REGIONS {
            let bytes = recording.pool.read(system_configuration.region_devices[region as usize], ring_slot_offset(region, 0), 512);
            if RootRecord::parse_slot(&bytes, &FIXED_FSID) == Some(genesis.root) {
                readable += 1;
            }
        }
        assert_eq!(readable, 3);
        assert_eq!(ring_region_offset(2).0, 7 << 20);
        // C512 2026-09-23 用户定案：mkfs 第 0 代写全零（这里直接查 mkfs 自己构造的根，不是靠 to_slot/parse_slot 往返——
        // 往返只证明「写的值读得回来」，证不了写的值就是全零，两者是不同的命题）。
        assert!(genesis.root.allocation_record_tree_root.is_empty_root(), "mkfs 第 0 代的分配记录树根指针全零");
        // D23 已定项 16：mkfs 写实例代号 0，单元写序 (0, 0)；树 ID 水位就是第一个要发的树 ID。
        assert_eq!(genesis.root.instance, InstanceGeneration(0));
        assert_eq!(genesis.root.tree_identifier_watermark, 11);
        assert_eq!(TREE_IDENTIFIER_WATERMARK_AFTER_PUBLISH, 19, "deadlist 取树 ID 18 ⇒ 发布后水位 19");
        assert_eq!(parse_packed_unit(&genesis.instance_table_unit).expect("实例表").write_order, WriteOrder { instance: InstanceGeneration(0), transaction: TransactionNumber(0) });
        // D22 已定项 16：mkfs 把每盘两个槽都种上世代号 1。
        for device_index in 0..2u32 {
            for slot_offset in SYSTEM_CONFIGURATION_SLOT_OFFSETS {
                let slot = recording.pool.read(DeviceIdentity(device_index), DeviceOffset(slot_offset), SYSTEM_CONFIGURATION_SLOT_BYTES as usize);
                let system_configuration = SystemConfiguration::parse_slot(&slot).expect("mkfs 的系统配置槽");
                assert_eq!(system_configuration.slot_generation, 1);
                assert_eq!(system_configuration.journal_instance, InstanceGeneration(0));
            }
        }
    }

    /// 系统配置 481（D22 已定项 9 + 已定项 15，2026-09-14 用户定案加四个字段）：每个字段按**绝对偏移**读一遍，
    /// 以及「整槽校验和罩整个 4096 槽」——改 481 之后那 3615 字节补齐里的任何一个字节，解析都要拒绝。
    #[test]
    fn system_configuration_is_481_bytes_and_the_slot_checksum_covers_all_4096() {
        let BuiltPool { recording, .. } = built_pool();
        let slot = recording.pool.read(DeviceIdentity(0), DeviceOffset(SYSTEM_CONFIGURATION_SLOT_OFFSETS[1]), SYSTEM_CONFIGURATION_SLOT_BYTES as usize);
        assert_eq!(slot.len(), 4096, "系统配置槽宽是格式常量 4096（D22 已定项 2，2026-09-14 三方论证后）");
        assert!(SystemConfiguration::parse_slot(&slot).is_some());
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
        assert!(slot[SYSTEM_CONFIGURATION_BYTES as usize..].iter().all(|byte| *byte == 0), "481 之后的 3615 字节补齐恒 0");
        let mut padded = slot.clone();
        padded[SYSTEM_CONFIGURATION_BYTES as usize] ^= 0x01;
        assert!(SystemConfiguration::parse_slot(&padded).is_none(), "整槽校验和罩到补齐区（D18 已定项 17）");
        // 算法标识写成未登记的码：这个槽不认（I-2.2）。
        let mut wrong_algorithm = slot.clone();
        wrong_algorithm[138] = 2;
        let digest = wide_checksum_with_field_zeroed(&wrong_algorithm, SYSTEM_CONFIGURATION_SLOT_BYTES as usize, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
        wrong_algorithm[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        assert!(SystemConfiguration::parse_slot(&wrong_algorithm).is_none(), "校验和算法标识不是 1 就不认这个槽");
    }

    #[test]
    fn unknown_incompat_bit_or_missing_ssd_line_bit_refuses_to_mount_but_compat_bits_do_not() {
        let parameters = PoolParameters::settled_two_devices();
        let (recording, _) = mkfs(&parameters);
        let slot = recording.pool.read(DeviceIdentity(0), DeviceOffset(SYSTEM_CONFIGURATION_SLOT_OFFSETS[0]), SYSTEM_CONFIGURATION_SLOT_BYTES as usize);
        assert_eq!(slot[SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET], INCOMPAT_FIRST_SSD_LINE_BIT, "mkfs 起 incompat 位 0 置 1（D15 已定项 4）");
        assert!(slot[SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET + 1..SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET + 3 * FEATURE_BITMAP_BYTES].iter().all(|byte| *byte == 0), "其余 95 字节全 0");
        assert!(SystemConfiguration::parse_slot(&slot).is_some(), "原样可挂");
        let reseal = |mut bytes: Vec<u8>, offset: usize, value: u8| {
            bytes[offset] = value;
            let digest = wide_checksum_with_field_zeroed(&bytes, SYSTEM_CONFIGURATION_SLOT_BYTES as usize, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
            bytes[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
            bytes
        };
        assert!(SystemConfiguration::parse_slot(&reseal(slot.clone(), SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET, 0x03)).is_none(), "incompat 位 1 没登记：不认识不许挂");
        assert_eq!(SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET, 6);
        assert!(SystemConfiguration::parse_slot(&reseal(slot.clone(), SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET + FEATURE_BITMAP_BYTES - 1, 0x80)).is_none(), "incompat 位 255 没登记：不认识不许挂");
        assert!(SystemConfiguration::parse_slot(&reseal(slot.clone(), SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET, 0x00)).is_none(), "没有布局身份：拒绝");
        assert!(SystemConfiguration::parse_slot(&reseal(slot.clone(), SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET + FEATURE_BITMAP_BYTES, 0x01)).is_some(), "compat_ro 位不认识只读挂，读者照样解析");
        assert!(SystemConfiguration::parse_slot(&reseal(slot, SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET + 2 * FEATURE_BITMAP_BYTES, 0x01)).is_some(), "compat 位不认识随便");
    }

    #[test]
    fn cold_start_reads_the_file_back_and_chooses_instance_one_txg_three() {
        let BuiltPool { recording, .. } = built_pool();
        let report = recover(&recording.pool, JournalPolicy::Consult, ReaderMode::Clause);
        assert_eq!(report.outcome, RecoveryOutcome::FileRead { root: (InstanceGeneration(1), CheckpointTxg(3)), content: sample_file() });
        assert_eq!(report.journal.valid_records, 3, "两条暖机空记录 + 事务记录");
        // 记录头 2026-09-14 带 fsid：把三条记录的 fsid 段全改成别的池，扫描一条都不认（I-1.4）。
        let mut alien = recording.pool.clone();
        for counter in 1..=3u64 {
            let offset = journal_record_offset(JournalCounter(counter));
            for device_index in 0..2u32 {
                let device = DeviceIdentity(device_index);
                let mut record = alien.read(device, offset, JOURNAL_RECORD_BYTES as usize);
                record[287..295].copy_from_slice(&0xDEAD_BEEF_u64.to_le_bytes()); // fsid 偏移 287（已定项 4，头 311 布局）
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

    /// E142 第十五次跑（第七节 B8）：分配记录树从 1 个单元拆成 5 个之后，事务段从 16 涨到 24 次写，
    /// 事务段与暖机第二次系统配置槽同段 ⇒ 那一段从 18 涨到 26；段序列与闭式改钉 B8，便宜（纯算术）。
    /// 整轮枚举（6710 万个状态）挪到下面 `#[ignore]` 的那条——Q142.9 附带，第一、二行都不消费它，够判后不跑。
    #[test]
    fn layer0_segment_sizes_and_closed_form_match_the_new_layout() {
        let BuiltPool { recording, mkfs_operation_count, .. } = built_pool();
        let (_, segments) = split_into_segments(&recording.operations[mkfs_operation_count..], true);
        assert_eq!(segments.iter().map(Vec::len).collect::<Vec<_>>(), vec![2, 2, 1, 2, 2, 1, 26, 2, 1, 2]);
        assert_eq!(closed_form_state_count(&segments), 67_108_885, "第七节 B8「layer0_states=67108885」");
    }

    #[test]
    #[ignore = "整轮枚举 67108885 个状态（第七节 B8），Q142.9 附带、够判后不跑（这一段没有它也够判第 1、2 行）；\
                第二段要验时手动 `cargo test -- --ignored layer0_state_count_is_67108885_with_zero_violations`"]
    fn layer0_state_count_is_67108885_with_zero_violations() {
        let BuiltPool { recording, mkfs_operation_count, .. } = built_pool();
        let (base, _) = mkfs(&PoolParameters::settled_two_devices());
        let (writes, segments) = split_into_segments(&recording.operations[mkfs_operation_count..], true);
        let tally = enumerate_layer0(&base.pool, &writes, &segments, &sample_file(), ReaderMode::Clause);
        assert_eq!(tally.states, 67_108_885);
        assert_eq!(tally.violations, 0, "{:?}", tally.first_violation);
    }

    /// FUA 不当边界（C313 已判掉的另一读法，只钉它给的数不同）：第七节 B9「new_segments=[2,2,3,2,27,2,3]
    /// fua_not_boundary_states=134217754」。
    #[test]
    fn fua_not_a_boundary_gives_134217754_states() {
        let BuiltPool { recording, mkfs_operation_count, .. } = built_pool();
        let (_, segments) = split_into_segments(&recording.operations[mkfs_operation_count..], false);
        assert_eq!(segments.iter().map(Vec::len).collect::<Vec<_>>(), vec![2, 2, 3, 2, 27, 2, 3]);
        assert_eq!(closed_form_state_count(&segments), 134_217_754);
    }

    /// 段序列登记表（layout/01-first-txn 八）的五行由这条钉住：mkfs 12+1+1+1+4（21 次操作、4114 个状态，
    /// D22（单元原子性怎么合成） 已定项 8 第 3 条 2026-09-23 用户定案改写：根环三个区域与 journal 环都由 mkfs
    /// 经写零动作清，每块盘四段各一次调用、两块盘共 8 步，段序列与步数以 `.claude/kb/layout/01-first-txn.md`
    /// 八「mkfs 种根」那一行为准——这个值与 `.claude/kb/checks-owed.md` C484（mkfs 不清根环，同 fsid 重来旧根还择得中）/
    /// C487（门禁 55 号拿活代码与不跟踪它的模型比） 登记的 `crates/` 真值一致）、取号 2、暖机 2+1+2+2+1+2、
    /// 事务 24+2+1+2、整条流 2+2+1+2+2+1+26+2+1+2（E142 第十五次跑，第七节 B5「segments=[24,2,1,2]」）。
    /// 改任何一条路径里屏障或 FUA 的位置都红——mkfs 那一行不进层 0 枚举，这里是它唯一的会红检查。
    ///
    /// D17（实现分层与第三方管道） 已定项 2 的结构等价类要的是「段边界位置 + 每段步骤种类集合」，
    /// 所以这里连每段的步骤种类多重集一起钉死，四条路径各钉一个**绝对值**（不是拿几条路径互相比）：
    /// 把某一步录成别的种类——例如系统配置槽写录成单元写——段边界与状态数一个都不变，只有这几行会红（C316 ②）。
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
        assert_eq!(mkfs_operations.len(), 21, "mkfs：19 次写（两块盘各清根环三区域与 journal 环共 8、m1/m2 各两盘、三个第 0 代根、两盘各两个系统配置槽）+ 2 道屏障");
        assert_eq!(sizes(mkfs_operations), vec![12, 1, 1, 1, 4]);
        assert_eq!(closed_form_state_count(&split_into_segments(mkfs_operations, true).1), 4114);
        assert_eq!(acquisition_operations.len(), 2, "取号：两盘各写一次系统配置槽，不另加屏障");
        assert_eq!(sizes(acquisition_operations), vec![2]);
        assert_eq!(sizes(warm_up_operations), vec![2, 1, 2, 2, 1, 2]);
        assert_eq!(sizes(transaction_operations), vec![24, 2, 1, 2]);
        assert_eq!(sizes(post_mkfs_operations), vec![2, 2, 1, 2, 2, 1, 26, 2, 1, 2]);

        assert_eq!(
            kinds(mkfs_operations),
            "[unit_write×4,zero_fill×8,barrier]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier]",
            "mkfs：两块盘各清根环三区域与 journal 环（共 8）与 m1/m2 两个单元各两盘同一段、三个第 0 代根各自 FUA 一段、两盘各两个系统配置槽收尾"
        );
        assert_eq!(kinds(acquisition_operations), "[system_configuration_slot×2]", "取号那一段只有两次系统配置槽写");
        assert_eq!(
            kinds(warm_up_operations),
            "[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[system_configuration_slot×2]",
            "暖机两次空发布：每次「空记录两盘 → 根 FUA → 系统配置槽两盘」，第一段前面还有那道开场屏障"
        );
        assert_eq!(
            kinds(transaction_operations),
            "[unit_write×24,barrier]|[journal_record×2,barrier]|[root_record_fua]|[system_configuration_slot×2]",
            "第一个事务：12 个单元各两盘（分配记录树拆成 5 个节点之后 t1..t12）→ journal 记录两盘 → 根 FUA → 系统配置槽两盘"
        );
        assert_eq!(
            kinds(post_mkfs_operations),
            "[system_configuration_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[system_configuration_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[unit_write×24,system_configuration_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[system_configuration_slot×2]",
            "整条流：取号那两次系统配置槽写自成一段（收段的是暖机第一次开头那道屏障），暖机第二次的两个系统配置槽与事务的 24 个单元写同一段 26 个写"
        );

        // 每一步都恰好落在一个段里：各段的步骤数加起来等于录到的操作数。
        for operations in [mkfs_operations, acquisition_operations, warm_up_operations, transaction_operations, post_mkfs_operations] {
            assert_eq!(segment_step_kinds(operations, true).iter().map(Vec::len).sum::<usize>(), operations.len());
        }
        // mkfs 之后的种类字母表仍是这五个：新的 zero_fill 只出现在 mkfs 自己的段里。
        let alphabet: std::collections::BTreeSet<&str> = segment_step_kinds(post_mkfs_operations, true).concat().iter().map(|kind| kind.tag()).collect();
        assert_eq!(alphabet.into_iter().collect::<Vec<_>>(), vec!["barrier", "journal_record", "root_record_fua", "system_configuration_slot", "unit_write"]);
        // 整条录制流（含 mkfs）的字母表五种改六种（D22 已定项 8 第 3 条 2026-09-23 用户定案改写，新增 zero_fill），
        // 与 layout/01-first-txn.md 八「mkfs 种根」那一行登记的真值（六种）同形。
        let global_alphabet: std::collections::BTreeSet<&str> = segment_step_kinds(&recording.operations, true).concat().iter().map(|kind| kind.tag()).collect();
        assert_eq!(
            global_alphabet.into_iter().collect::<Vec<_>>(),
            vec!["barrier", "journal_record", "root_record_fua", "system_configuration_slot", "unit_write", "zero_fill"],
            "步骤种类五种改六种（D22 已定项 8 第 3 条：mkfs 新增 zero_fill）"
        );
    }

    /// 暖机（D16 已定项 8）：两次空发布，根落区域 1 与区域 2（分住两块盘），jsn 1、2 不点名任何单元，第一个事务从 txg 3 起。
    #[test]
    fn warm_up_writes_two_empty_publishes_covering_both_devices() {
        let BuiltPool { recording, output, acquisition_operation_count, warm_up_operation_count, .. } = built_pool();
        let operations = &recording.operations[acquisition_operation_count..warm_up_operation_count];
        let writes = operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(_))).count();
        let fua_writes: Vec<&WriteRequest> = operations.iter().filter_map(|operation| match operation { RecordedOperation::Write(write) if write.is_fua() => Some(write), RecordedOperation::Write(_) | RecordedOperation::Barrier => None }).collect();
        assert_eq!(writes, 10, "每次空发布 2 条记录 + 1 个根 + 2 个系统配置槽");
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
        assert_eq!(system_configuration_write_for_publish(CheckpointTxg(1)), (3, 1));
        assert_eq!(system_configuration_write_for_publish(CheckpointTxg(2)), (4, 0));
        assert_eq!(system_configuration_write_for_publish(CheckpointTxg(3)), (5, 1));
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

    /// 第八节 G4 用：`model_window_writes_for_parameters` 取的窗口（暖机最后一次写之后到发布结束）
    /// 在一盘几何上恰 13 次写——这是「一次发布调用自己发出的写数」，与暖机走没走过无关，
    /// 与下一条测试（跳过暖机、mkfs 之后直接发布）数到的同一个 13 是同一个量的两种取法（第七节 B10）。
    /// M126 的会红检查落在这里：改了窗口起点会把这个数改成 12 或漏最后一次写；这条函数原来没有
    /// 任何单测覆盖（`model_window_writes_for_parameters` 是这次跑从 `run_variant_window_writes`
    /// 改来专供 G4 用的，第十五次跑冻结的单测钉的是旧函数，没有跟着搬），第十六次跑在这里补上（真盲区）。
    #[test]
    fn model_window_writes_for_parameters_has_thirteen_writes_on_the_one_device_geometry() {
        let parameters = PoolParameters::control_one_device_no_barriers();
        let writes = model_window_writes_for_parameters(&parameters, &sample_file());
        assert_eq!(writes.len(), 13, "一盘几何：暖机最后一次写之后到发布结束共 13 次写");
    }

    /// E142 第十五次跑（第七节 B10、B11）：一盘时分配记录树仍是叶 + 层级 1 + 根三个节点（B13「one_device
    /// first_txn_allocation_nodes=3」），加已有的 7 个单元 = 10 个单元、13 次写、2^13 = 8192 个状态。
    /// 这个装置的读者总是递归到叶收全部记录（`read_allocation_tree`），对应 B11 的
    /// 「读者整棵读分配记录树时」那一档：4096 − 4 = 4092。
    #[test]
    fn positive_control_without_barriers_has_4092_violations_out_of_8192() {
        let control = PoolParameters::control_one_device_no_barriers();
        let (mut recording, genesis) = mkfs(&control);
        let mkfs_operation_count = recording.operations.len();
        let _ = publish_first_file(&mut recording, &control, &genesis, &sample_file(), InstanceGeneration(FIRST_INSTANCE_GENERATION), None, FIRST_TRANSACTION_TXG);
        let (base, _) = mkfs(&control);
        let (writes, segments) = split_into_segments(&recording.operations[mkfs_operation_count..], false);
        assert_eq!(writes.len(), 13, "第七节 B10「control_writes=13」");
        let tally = enumerate_layer0(&base.pool, &writes, &segments, &sample_file(), ReaderMode::Clause);
        assert_eq!(tally.states, 8192, "第七节 B10「control_states=8192」");
        assert_eq!(tally.violations, 4092, "第七节 B11「violations_reader_reads_whole_allocation_tree=4092」");
        assert_eq!(tally.root_persisted_states, 4096);
    }

    /// E142 第十五次跑（第七节 B1–B3）：分配记录树五个节点的层级、罩的段、格——按位置寻址的落点表。
    /// M94（叶宽 812→806）、M95（扇出 169→168）、M97/M98（哪个常量该对哪个槽）、M100（γ乙 半开）、
    /// M101（层级 1 格号差一）、M102（α甲 下空格写 key，退回 α丙）的会红检查都落在这里。
    #[test]
    fn allocation_record_tree_five_nodes_match_the_position_addressed_layout() {
        let BuiltPool { output, .. } = built_pool();
        let leaf0 = parse_index_node(&output.units_by_slot[4].2).expect("罩盘 0 的叶");
        let leaf1 = parse_index_node(&output.units_by_slot[5].2).expect("罩盘 1 的叶");
        let internal0 = parse_index_node(&output.units_by_slot[6].2).expect("罩盘 0 的层级 1 节点");
        let internal1 = parse_index_node(&output.units_by_slot[7].2).expect("罩盘 1 的层级 1 节点");
        let root = parse_index_node(&output.units_by_slot[8].2).expect("根");

        // 叶（层级 0）：第七节 B3「leaves=[61] leaf_range=[(49532,50343)]」。
        for (leaf, device) in [(&leaf0, 0u32), (&leaf1, 1u32)] {
            assert_eq!(leaf.level, 0);
            assert_eq!(leaf.entries.len(), 14, "每盘 14 条记录都落在同一片叶（第七节 B4）");
            assert_eq!(leaf.smallest_key, allocation_position_key(device, 49532));
            assert_eq!(leaf.largest_key, allocation_position_key(device, 50343));
            for entry in &leaf.entries {
                assert_eq!(&entry[..4], &device.to_le_bytes(), "叶里每条记录都是这块盘自己的");
            }
        }

        // 层级 1（第七节 B3「l1_range=(0,137227)」）：D8 已定项 14 第 395 行唯一写法——稀疏，
        // 只写有孩子的格，这个装置的场景每盘只有一个孩子（cell_in_l1=61）。
        for (internal, device, leaf_slot) in [(&internal0, 0u32, SLOT_ALLOCATION_LEAF_DEVICE_0), (&internal1, 1u32, SLOT_ALLOCATION_LEAF_DEVICE_1)] {
            assert_eq!(internal.level, 1);
            assert_eq!(internal.entries.len(), 1, "唯一写法：稀疏，只有一条条目（第七节 B7「level1=96」＝1×96）");
            assert_eq!(internal.smallest_key, allocation_position_key(device, 0));
            assert_eq!(internal.largest_key, allocation_position_key(device, 137_227));
            let cell = &internal.entries[0];
            assert_eq!(&cell[..ALLOCATION_KEY_BYTES], &allocation_position_key(device, 49532)[..], "ι甲：条目 key = 孩子按位置罩的段起点");
            let mut reader = ByteReader::new(cell);
            reader.skip(ALLOCATION_KEY_BYTES);
            let pointer = NodePointer::read_from(&mut reader);
            assert_eq!(pointer.locations[0].slot, SlotNumber(leaf_slot), "唯一那条条目的子指针指向对应盘的叶");
        }

        // 根（层级 2，单一节点，按盘分流）：D8 已定项 14 第 395 行唯一写法——稀疏，两条条目（一盘一条），
        // smallest_key=(0,0)、largest_key=(0xFFFFFFFF,2^48−1)（第七节 A6，与盘数、盘大小无关）。
        assert_eq!(root.level, 2);
        assert_eq!(root.entries.len(), 2, "唯一写法：稀疏，一盘一条（第七节 B7「root=192」＝2×96）");
        assert_eq!(root.smallest_key, allocation_position_key(0, 0));
        assert_eq!(root.largest_key, allocation_position_key(u32::MAX, (1u64 << 48) - 1));
        for (cell_index, device, expected_slot) in [(0usize, 0u32, SLOT_ALLOCATION_INTERNAL_DEVICE_0), (1, 1u32, SLOT_ALLOCATION_INTERNAL_DEVICE_1)] {
            let cell = &root.entries[cell_index];
            assert_eq!(&cell[..ALLOCATION_KEY_BYTES], &allocation_position_key(device, 0)[..], "根条目 {cell_index} 的 key（ι甲：段起点 0）");
            let mut reader = ByteReader::new(cell);
            reader.skip(ALLOCATION_KEY_BYTES);
            let pointer = NodePointer::read_from(&mut reader);
            assert_eq!(pointer.locations[0].slot, SlotNumber(expected_slot), "根条目 {cell_index} 指向对应盘的层级 1 节点");
        }

        // 格式常量（第七节 B1，A1、A3）。
        assert_eq!(ALLOCATION_RECORD_TREE_LEAF_SLOTS, 812);
        assert_eq!(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES, 96);
        assert_eq!(ALLOCATION_RECORD_TREE_INTERNAL_FANOUT, 169);
        assert_eq!(EXTENT_TREE_UPPER_LEAF_INODES, 143);
        assert_eq!(EXTENT_TREE_INTERNAL_ENTRY_BYTES, 110);
        assert_eq!(EXTENT_TREE_INTERNAL_FANOUT, 147);
        assert_eq!(EXTENT_TREE_LOWER_LEAF_DATA_UNITS, 144);
    }

    /// D8 已定项 14 第 395 行（唯一写法）：`assemble_allocation_cells` 只给有孩子的格建条目，没有孩子的
    /// 格不占条目（M127/M128 的会红检查：改回稠密／稠密带 key 会让这条数变成 3）。
    #[test]
    fn assemble_allocation_cells_only_keeps_positions_with_a_child() {
        let pointer = NodePointer::empty_root();
        let slots = vec![(0u32, 100u64, None), (0u32, 200u64, Some(pointer)), (0u32, 300u64, None)];
        let cells = assemble_allocation_cells(&slots);
        assert_eq!(cells.len(), 1, "唯一写法：稀疏，只写有孩子的格");
        assert_eq!(cells[0], allocation_internal_cell(0, 200, pointer));
    }

    /// 5.2 γ/η（D18 已定项 2 射程、D8 已定项 14 第 401 行，唯一写法：写路径与读路径共用）：extent 上段叶
    /// 第 k 片的 key 区间，第七节 B12 锚点的四个 inode 点（1、142、143、286）。
    #[test]
    fn extent_upper_leaf_positional_key_range_matches_the_frozen_anchors() {
        for (inode, expected_k, expected_smallest_inode, expected_largest_inode) in
            [(1u64, 0u64, 0u64, 142u64), (142, 0, 0, 142), (143, 1, 143, 285), (286, 2, 286, 428)]
        {
            let k = extent_upper_leaf_index(inode);
            assert_eq!(k, expected_k, "inode={inode}");
            let (smallest, largest) = extent_upper_leaf_positional_key_range(k);
            let mut expected_smallest = [0u8; 24];
            expected_smallest[8..16].copy_from_slice(&expected_smallest_inode.to_le_bytes());
            let mut expected_largest = [0u8; 24];
            expected_largest[8..16].copy_from_slice(&expected_largest_inode.to_le_bytes());
            expected_largest[16..24].copy_from_slice(&u64::MAX.to_le_bytes());
            assert_eq!(smallest, expected_smallest.to_vec(), "inode={inode}：smallest_key");
            assert_eq!(largest, expected_largest.to_vec(), "inode={inode}：largest_key（第三分量恒 2^64−1，第七节 A7）");
        }
    }

    /// E142 第十五次跑（第七节 A3）：extent 树上段叶条目 113 = key 24 `(0, inode, 0)` + 标签 2（内联）+ 数据指针 88。
    /// M103（标签 2→1）、M104（max_key 的 inode 分量 142→143）、M105（丢标签字节，113→112）的会红检查落在这里。
    #[test]
    fn extent_upper_leaf_entry_is_113_bytes_with_an_inline_data_pointer_tag() {
        let BuiltPool { output, .. } = built_pool();
        let extent_node = parse_index_node(&output.units_by_slot[1].2).expect("extent 上段叶（根即叶）");
        assert_eq!(extent_node.level, 0, "5.2 ζ：只有一个 inode ⇒ 根就是上段叶");
        assert_eq!(extent_node.entries.len(), 1, "5.2 ε 甲：稀疏，只写有文件的 inode");
        let entry = &extent_node.entries[0];
        assert_eq!(entry.len(), 113, "第七节 A3「extent_tree_upper_leaf_entry=113」");
        let (key, tag, pointer) = parse_extent_upper_leaf_entry(entry);
        assert_eq!(&key[..8], &0u64.to_le_bytes(), "locality 恒 0");
        assert_eq!(&key[8..16], &FIRST_INODE_NUMBER.to_le_bytes());
        assert_eq!(&key[16..24], &0u64.to_le_bytes(), "条目自己的 key 第三分量恒 0（与节点头 largest_key 的第三分量 2^64−1 是两回事）");
        assert_eq!(tag, EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT, "只有一个数据单元的文件 ⇒ 标签 2（内联）");
        assert_eq!(pointer, output.data_pointer);
        // key 区间（D8 已定项 14 第 401 行，唯一写法：按位置罩的那一段，不是记录自己的 key）：
        // inode 1 ⇒ k = 0，闭区间 [0,142]，第三分量恒 2^64 − 1（第七节 A7、B12）。
        let (expected_smallest, expected_largest) = extent_upper_leaf_positional_key_range(0);
        assert_eq!(extent_node.smallest_key, expected_smallest);
        assert_eq!(extent_node.largest_key, expected_largest);
    }

    /// 第八节 G3「形状，纯函数」（第七节 B13）：分配记录树的根层级、根每盘格数、第一个事务节点数
    /// 在五个几何点上都用纯函数独立算一遍，不跑整条写路，入参一律是「每盘设备字节数」经 `slots_of_device_bytes`
    /// （D8 已定项 14 第 408 行，唯一写法）。M96（根层级门槛 ≤169→≤3）在主点（R=2→3）就翻；
    /// M142（盘上槽数向上取整）在主点不改输出（4 GiB 整除），只在 half_slot 点上把 R 从 2 顶到 3。
    #[test]
    fn allocation_record_tree_shape_matches_the_five_geometry_points() {
        assert_eq!(DEVICE_SLOTS, 262_144, "主点：4 GiB ÷ 16 KiB（δ 甲，经 slots_of_device_bytes，第七节 B3「device_slots」）");
        let half_slot_device_bytes: u64 = 84 * allocation_record_tree_span_at_level(1) * SLOT_BYTES + SLOT_BYTES / 2;
        let cases: [(&str, u64, u64, u32, u64, u64); 5] = [
            // (标签, 盘数, 每盘设备字节数, R, root_cells_per_device, first_txn_allocation_nodes)——第七节 B13。
            ("main", 2, DEVICE_BYTES, 2, 2, 5),
            ("one_device", 1, DEVICE_BYTES, 2, 2, 3),
            ("8GiB", 2, 8u64 << 30, 2, 4, 5),
            ("256GiB", 2, 256u64 << 30, 3, 1, 7),
            ("half_slot", 2, half_slot_device_bytes, 2, 84, 5),
        ];
        for (label, device_count, device_bytes, expected_root_level, expected_cells_per_device, expected_nodes) in cases {
            let slots = slots_of_device_bytes(device_bytes);
            let device_slots = vec![slots; device_count as usize];
            let root_level = allocation_record_tree_root_level(&device_slots);
            assert_eq!(root_level, expected_root_level, "{label}：R");
            let cells_per_device = allocation_record_tree_cells_per_device(slots, root_level);
            assert_eq!(cells_per_device, expected_cells_per_device, "{label}：root_cells_per_device");
            let nodes = allocation_record_tree_first_transaction_node_count(device_count, root_level);
            assert_eq!(nodes, expected_nodes, "{label}：first_txn_allocation_nodes");
        }
        // M142 的会红检查：half_slot 点上向上取整会把盘上槽数多算一槽，进而把 R 从 2 顶到 3（第七节 B13「R_if_ceil=3」）。
        let half_slot_slots_floor = slots_of_device_bytes(half_slot_device_bytes);
        let half_slot_slots_ceil = half_slot_device_bytes.div_ceil(SLOT_BYTES);
        assert_eq!(half_slot_slots_floor + 1, half_slot_slots_ceil, "half_slot 点：向上取整比向下取整多一槽（第 408 行「零头不算」）");
        assert_eq!(allocation_record_tree_root_level(&[half_slot_slots_floor; 2]), 2, "向下取整：R 仍是 2");
        assert_eq!(allocation_record_tree_root_level(&[half_slot_slots_ceil; 2]), 3, "M142 的会红检查：向上取整会把 R 顶到 3");
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
        // B8（第七节 7.2）：头 307 布局是 46+32+8+1+4=91；头 311 布局加本次发布内序号 4 字节 ⇒ 95。
        assert_eq!(JOURNAL_HEADER_CHECKSUM_OFFSET + WIDE_CHECKSUM_BYTES as usize + 8 + 1 + 4, 91, "头 307 布局的载荷校验和偏移（B8，仅作历史对照，不是这条记录的真实偏移）");
        let payload_checksum_offset = JOURNAL_HEADER_CHECKSUM_OFFSET + WIDE_CHECKSUM_BYTES as usize + 8 + 1 + 4 + 4;
        assert_eq!(payload_checksum_offset, 95, "载荷校验和字段的偏移（头 311 布局，已定项 4）");
        for (offset, what) in [
            (payload_checksum_offset, "载荷校验和字段"),
            (JOURNAL_HEADER_BYTES as usize + 3, "点名项数组"),
            (JOURNAL_RECORD_BYTES as usize - 1, "点名项之后的补齐区最后一字节"),
        ] {
            let mut record = output.record_bytes.clone();
            record[offset] ^= 0x01;
            assert!(JournalRecord::parse(&record).is_none(), "{what}落在 header_csum 覆盖内");
        }
        // 反向：只罩头 [0, 311) 的那个读法对补齐区那一字节说不出话——这一行证明上面第三条不是白抓的。
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
        assert!(matches!(by_name["system_configuration_slot_one_both_devices"].outcome, RecoveryOutcome::FileRead { .. }));
    }

    /// P6（A8，E142 第十四次跑第五节 5.2）：读者读了标志与序号——十六格预期逐句照条款推出来。
    /// S*：t9 之前（含 t9 两份）全部持久，t10 根槽与 t11 系统配置槽都没持久。
    #[test]
    fn p6_reader_outcomes_match_the_clause_table() {
        let f = PoolParameters::settled_two_devices();
        let h = f.clone().with_last_of_publish_flag(0x00);
        let file_bytes = sample_file();

        let row_f = run_p6_scenario(&f, &file_bytes, None, false);
        assert_eq!(row_f.primary_prefix_applied, 1, "读者甲：F 上 t9 施加 ⇒ 根推进到 (1,3)");
        assert_eq!(row_f.clause_prefix_applied, 1, "锚 w4（带标志）⇒ 接 t9；t9 带标志、提交标记 1 ⇒ 施加，根推进到 (1,3)");
        assert!(!row_f.clause_fatal);

        let row_h = run_p6_scenario(&h, &file_bytes, None, false);
        assert_eq!(row_h.primary_prefix_applied, 1, "读者甲不看标志，H 上 t9 照样施加 ⇒ (1,3)");
        assert_eq!(row_h.clause_prefix_applied, 0, "w4 不带标志 ⇒ 锚点读不出；t9 也不带标志 ⇒ 这次发布走不到末条，不施加，根停在 (1,2)");
        assert!(!row_h.clause_fatal);

        let row_flag_03 = run_p6_scenario(&f, &file_bytes, Some(p6_mutate_flag_0x03), false);
        assert_eq!(row_flag_03.primary_prefix_applied, 1);
        assert_eq!(row_flag_03.primary_readable, 3);
        assert_eq!(row_flag_03.clause_prefix_applied, 0, "位 0 之外有位为 1 ⇒ 当损坏（已定项 4），根停在 (1,2)");
        assert_eq!(row_flag_03.clause_readable, 2, "t9 当损坏，读得出的只剩 w1、w4");

        let row_ordinal_0 = run_p6_scenario(&f, &file_bytes, Some(p6_mutate_ordinal_0), false);
        assert_eq!(row_ordinal_0.primary_prefix_applied, 1);
        assert_eq!(row_ordinal_0.primary_readable, 3);
        assert_eq!(row_ordinal_0.clause_prefix_applied, 0, "序号 0 ⇒ 当损坏（已定项 4），根停在 (1,2)");
        assert_eq!(row_ordinal_0.clause_readable, 2);

        let row_flag_00 = run_p6_scenario(&f, &file_bytes, Some(p6_mutate_flag_0x00), false);
        assert_eq!(row_flag_00.primary_prefix_applied, 1);
        assert_eq!(row_flag_00.clause_prefix_applied, 0, "锚 w4 ⇒ 接 t9；t9 不带标志 ⇒ 不施加（已定项 17），根停在 (1,2)");

        let row_ordinal_2 = run_p6_scenario(&f, &file_bytes, Some(p6_mutate_ordinal_2), false);
        assert_eq!(row_ordinal_2.primary_prefix_applied, 1);
        assert_eq!(row_ordinal_2.clause_prefix_applied, 0, "锚点 w4 读得出、t9 是下一次发布的首条而序号不是 1 ⇒ 断在 t9（已定项 4），根停在 (1,2)");

        let row_trailing = run_p6_scenario(&f, &file_bytes, None, true);
        // 读者甲不看「带标志的那条之后还有没有记录」：t9（counter 3）施加之后，链继续接上新加的 counter 4 那条也施加 ⇒ 2 条。
        assert_eq!(row_trailing.primary_prefix_applied, 2, "读者甲逐条施加，t9 与新加的那条都施加，根仍推进到 txg 3");
        assert_eq!(row_trailing.clause_prefix_applied, 0, "t9 带标志而其后还有同 (1,3) 的记录 ⇒ 断在 t9、这次发布不施加（已定项 4），根停在 (1,2)");

        let row_txg_2 = run_p6_scenario(&f, &file_bytes, Some(p6_mutate_txg_2), false);
        assert_eq!(row_txg_2.primary_prefix_applied, 0, "读者甲锚到 w4，t9 的 (1,2) 不在水位之上 ⇒ 不施加，根停在 (1,2)");
        assert!(row_txg_2.clause_fatal, "同一 (实例, txg) 两条带末条标志 ⇒ 盘坏了、停下（已定项 14「这一版的失败处置」）");
    }

    /// M88 的会红检查：H / F 相减的每一段都归到 w1 / w4 / t9 之一，一段都不落进 unregistered
    /// （`journal_record_name_for_offset` 漏掉暖机记录时这条会红）。
    #[test]
    fn header311_flag_diff_segments_all_map_to_a_known_record() {
        let f = PoolParameters::settled_two_devices();
        let h = f.clone().with_last_of_publish_flag(0x00);
        let (h_image, _) = run_full_pipeline(&h, FIRST_TRANSACTION_TXG, &sample_file());
        let (f_image, _) = run_full_pipeline(&f, FIRST_TRANSACTION_TXG, &sample_file());
        let differences = diff_pools(&h_image, &f_image);
        assert!(!differences.is_empty(), "P1/P2 的旋钮读到了才谈得上这条检查");
        for segment in &differences {
            let (record_name, _) = journal_record_name_for_offset(segment.offset).expect("H/F 的差异只可能落在 w1/w4/t9 的记录头里");
            assert!(["w1", "w4", "t9"].contains(&record_name), "段落在 {record_name}，不是 w1/w4/t9 之一");
        }
    }

    /// 第八节判别力自证（M90）：主取样点 w1、G1 的 t9 都没有 [91,95) 反向链差异段——两条都是本实例第一条，
    /// 反向链两臂恒 0（已定项 10）。挪成「含」两条都必须由绿转红，否则这两格没有判别力（作废 V3）。
    #[test]
    fn header311_geometry_back_chain_absence_has_discriminating_power() {
        let f = PoolParameters::settled_two_devices();
        let h = f.clone().with_last_of_publish_flag(0x00);
        let (h_image, _) = run_full_pipeline(&h, FIRST_TRANSACTION_TXG, &sample_file());
        let (f_image, _) = run_full_pipeline(&f, FIRST_TRANSACTION_TXG, &sample_file());
        let main_differences = diff_pools(&h_image, &f_image);
        let has_back_chain = |differences: &[DiffSegment], record: &str| {
            differences.iter().any(|segment| {
                journal_record_name_for_offset(segment.offset).is_some_and(|(name, offset_in_record)| name == record && journal_header_field_tag(offset_in_record) == "back_chain")
            })
        };

        let g1_h = PoolParameters::control_one_device_no_barriers().with_last_of_publish_flag(0x00);
        let g1_f = PoolParameters::control_one_device_no_barriers().with_last_of_publish_flag(0x01);
        let (mut g1_recording_h, g1_genesis_h) = mkfs(&g1_h);
        let _ = publish_first_file(&mut g1_recording_h, &g1_h, &g1_genesis_h, &sample_file(), InstanceGeneration(FIRST_INSTANCE_GENERATION), None, FIRST_TRANSACTION_TXG);
        let (mut g1_recording_f, g1_genesis_f) = mkfs(&g1_f);
        let _ = publish_first_file(&mut g1_recording_f, &g1_f, &g1_genesis_f, &sample_file(), InstanceGeneration(FIRST_INSTANCE_GENERATION), None, FIRST_TRANSACTION_TXG);
        let g1_differences = diff_pools(&g1_recording_h.pool, &g1_recording_f.pool);

        assert!(!has_back_chain(&main_differences, "w1"), "主取样点：w1 是本实例第一条，反向链两臂都是 0");
        assert!(!has_back_chain(&g1_differences, "t9"), "G1：本实例第一条就是 t9，反向链两臂都是 0");
    }

    /// A7（第七节）的会红检查（M86）：反向链 = CRC32C(前一条记录的头 [0,311)，头校验和 32 字节按零参与)。
    /// 311 在这里写成字面量、逐字节手拼，不调用 `journal_back_chain`——两条路径独立算，
    /// 才分得出「算法本身错了」与「验证用的是同一个（可能同样错的）函数」。
    #[test]
    fn back_chain_matches_an_independently_computed_crc_over_311_bytes() {
        let BuiltPool { recording, output, .. } = built_pool();
        let records = scan_journal(&recording.pool, unit_fsid(&FIXED_FSID));
        let w4_bytes = records[&(InstanceGeneration(1), JournalCounter(2))].to_bytes();
        let mut header_311 = w4_bytes[..311].to_vec();
        assert_eq!(header_311.len(), 311, "A1：头 311 字节");
        header_311[46..46 + 32].fill(0); // 头校验和字段按零参与
        let independently_computed = castagnoli_crc32(&header_311);
        assert_eq!(output.record.back_chain, independently_computed, "t9 的反向链 = CRC32C(w4 的 311 字节头，头校验和按零参与)");
    }

    /// M87 的会红检查：`explain_journal_record_offset` 对偏移 87/91/95 的标签要按 D23 已定项 4
    /// 今天的字段表（本次发布内序号/反向链/载荷校验和），不是退回头 307 布局的旧标签。
    #[test]
    fn explain_journal_record_offset_labels_match_the_311_byte_field_table() {
        let (_, label) = explain_journal_record_offset(&[], 87);
        assert!(label.contains("ordinal_within_publish"), "偏移 87 是本次发布内序号，标签是「{label}」");
        let (_, label) = explain_journal_record_offset(&[], 91);
        assert!(label.contains("back_chain"), "偏移 91 是反向链，标签是「{label}」");
        let (_, label) = explain_journal_record_offset(&[], 95);
        assert!(label.contains("载荷校验和"), "偏移 95 是载荷校验和，标签是「{label}」");
    }

    /// M31 的会红检查：读者甲（`replay_journal`）施加记录时必须用记录里的新树表指针重建根，
    /// 不能沿用旧根的树表指针——沿用旧的会让施加之后的走读找不到刚发布的文件。
    #[test]
    fn primary_reader_rebuilds_the_tree_table_pointer_when_applying_a_record() {
        let BuiltPool { recording, .. } = built_pool();
        let parameters = PoolParameters::settled_two_devices();
        let probe = probes(&parameters).into_iter().find(|probe| probe.name == "newest_root_slot_one_byte").expect("探针在（第三节 probes()）");
        let mut damaged = recording.pool.clone();
        for (device, offset, byte_index) in &probe.flips {
            flip_byte(&mut damaged, *device, *offset, *byte_index);
        }
        let report = recover(&damaged, JournalPolicy::Consult, ReaderMode::Primary);
        assert!(
            matches!(&report.outcome, RecoveryOutcome::FileRead { root: (InstanceGeneration(1), CheckpointTxg(2)), content } if *content == sample_file()),
            "最新根槽坏了⇒择回暖机第 2 代根、施加 jsn 3 那条记录重建那次发布的根，用新树表指针才读得到文件：{:?}",
            report.outcome
        );
        assert_eq!(report.journal.prefix_applied, 1);
    }

    /// M67 的会红检查：D23 已定项 14 第 1 条「前缀规则不跨实例边界」——所选根是 mkfs 的第 0 代根
    /// （实例代号 0）时，别的实例（哪怕 checkpoint_txg 更大）的记录一条都不许施加。
    #[test]
    fn primary_reader_does_not_apply_records_from_a_newer_instance_onto_the_genesis_root() {
        let parameters = PoolParameters::settled_two_devices();
        let (recording, genesis) = mkfs(&parameters);
        let base = recording.pool.clone();
        let instance = InstanceGeneration(FIRST_INSTANCE_GENERATION);
        let w1 = JournalRecord {
            instance,
            counter: JournalCounter(1),
            checkpoint_txg: CheckpointTxg(1),
            transaction: TransactionNumber(0),
            is_commit: true,
            ordinal_within_publish: 1,
            record_flags: 0x01,
            back_chain: 0,
            fsid: unit_fsid(&parameters.fsid),
            new_tree_table: genesis.root.tree_table,
            new_mapping_root: genesis.root.mapping_root,
            new_tree_identifier_watermark: genesis.root.tree_identifier_watermark,
            new_rollback_floor: genesis.root.rollback_floor,
            named: Vec::new(),
        };
        let record_bytes = w1.to_bytes();
        let writes: Vec<WriteRequest> = parameters
            .devices()
            .into_iter()
            .map(|device| WriteRequest { device, offset: journal_record_offset(JournalCounter(1)), bytes: record_bytes.clone(), kind: StepKind::JournalRecord })
            .collect();
        let persisted = vec![true; writes.len()];
        let image = CrashImage { base: &base, writes: &writes, persisted };
        let report = recover(&image, JournalPolicy::Consult, ReaderMode::Primary);
        assert_eq!(outcome_root(&report.outcome), "0:0", "所选根是 mkfs 第 0 代根（实例 0），实例 1 的记录不跨实例边界，一条都不施加");
        assert_eq!(report.journal.prefix_applied, 0);
        assert_eq!(report.journal.above_water, 0, "前缀规则不跨实例边界：above_water 在过滤之后就该是 0");
    }

    /// 根记录 457（D22 已定项 7 的字段表，2026-09-14 用户定案在中央映射树根指针之后加算法类型 1 + nonce 12 + MAC 16；
    /// C512（树表 0 条的一版上被换下的实例表记在哪没有条款） 2026-09-23 用户定案在映射树根指针与那三段之间再插分配记录树根指针 86）：
    /// 自证校验和罩整个 512 槽（D18 已定项 17），末尾 29 字节第一版全 0，分配记录树根指针这个装置的场景里也恒 0。
    #[test]
    fn root_record_is_457_bytes_and_carries_the_mapping_tree_root() {
        let BuiltPool { output, .. } = built_pool();
        let slot = output.root.to_slot();
        assert_eq!(slot.len(), PHYSICAL_BLOCK_BYTES as usize);
        assert_eq!(RootRecord::parse_slot(&slot, &FIXED_FSID), Some(output.root));
        assert!(slot[ROOT_RECORD_BYTES as usize..].iter().all(|byte| *byte == 0), "457 之后的 55 字节补齐恒 0");
        let mut padded = slot.clone();
        padded[ROOT_RECORD_BYTES as usize] ^= 0x01;
        assert!(RootRecord::parse_slot(&padded, &FIXED_FSID).is_none(), "自证校验和罩到补齐区");
        // 分配记录树根指针这 86 字节这个装置的场景里恒 0（C512：只有「树表 0 条、写过行」的一版写非零，这个装置不建那一格）。
        assert!(slot[342..428].iter().all(|byte| *byte == 0), "分配记录树根指针这个装置的场景里恒 0");
        // 算法类型 / nonce / MAC 这 29 字节是第一版的留位，全 0；映射树根指针在它们之前隔着分配记录树根指针。
        assert!(slot[428..457].iter().all(|byte| *byte == 0), "算法类型 1 + nonce 12 + MAC 16 第一版全 0");
        let mut mapping_pointer_bytes = ByteReader::at(&slot, (ROOT_RECORD_BYTES - 2 * NODE_POINTER_BYTES - 1 - 12 - 16) as usize);
        assert_eq!(NodePointer::read_from(&mut mapping_pointer_bytes), output.root.mapping_root);
        assert_eq!(output.root.mapping_root.locations[0].slot, SlotNumber(SLOT_MAPPING_ROOT));
        let mut allocation_record_tree_root_bytes = ByteReader::at(&slot, (ROOT_RECORD_BYTES - NODE_POINTER_BYTES - 1 - 12 - 16) as usize);
        assert_eq!(NodePointer::read_from(&mut allocation_record_tree_root_bytes), output.root.allocation_record_tree_root);
    }

    /// journal 记录头 311（D23 已定项 15 的新根段 188 + 2026-09-14 加的 fsid 8 与 MAC 16 + 2026-09-24 加的
    /// 记录标志 1 与本次发布内序号 4）；点名项 56 的 key 尾段凑得出中央映射的 6 条 key（已定项 17 末句），一律 27 字节。
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
        // A4 / A5（第七节）：臂 F 三条记录偏移 7 是 0x01，偏移 87–90 是 01 00 00 00（本次发布内序号 1）。
        assert_eq!(parsed.record_flags, 0x01, "臂 F：本次发布末条标志（已定项 17）");
        assert_eq!(parsed.ordinal_within_publish, 1, "这次发布只有这一条（已定项 4）");
        assert_eq!(output.record_bytes[JOURNAL_RECORD_FLAGS_OFFSET], 0x01);
        assert_eq!(&output.record_bytes[87..91], &1u32.to_le_bytes(), "本次发布内序号小端 01 00 00 00");
        assert!(output.record_bytes[295..311].iter().all(|byte| *byte == 0), "记录头末尾 MAC 16 第一版全 0");
        let rebuilt: Vec<Vec<u8>> = parsed.named.iter().map(NamedUnit::mapping_key).collect();
        assert_eq!(rebuilt.len(), 12, "t1..t12（第七节 B6「named=12」）");
        assert_eq!(rebuilt.iter().filter(|key| key.len() == 27).count(), 12, "一律 27（D19 已定项 10，2026-09-14 用户定案）");
        for key in &output.mapping_keys {
            assert!(rebuilt.contains(key), "映射树里的 key 都凑得出来：{key:?}");
        }
    }

    /// 中央映射树：**10** 条条目（第七节 B7「mapping_entries=10」，分配记录树的五个新节点各占一条）——
    /// 一宽 55、key 一律 27（D19 已定项 10，2026-09-14 用户定案回定宽）；码 2 / 码 3 的 25 字节 key 末尾补两个零，
    /// 补零进 key 本身而不只是区间字段。
    #[test]
    fn mapping_tree_holds_one_entry_width_and_pads_short_keys_into_the_key_itself() {
        let BuiltPool { output, .. } = built_pool();
        let node = parse_index_node(&output.units_by_slot[10].2).expect("映射树根");
        assert_eq!(node.entries.len(), 10);
        assert!(node.entries.iter().all(|entry| entry.len() == 55), "条目一律 55");
        assert_eq!(node.key_width, 27, "头自述的 key 宽");
        assert_eq!(node.smallest_key.len(), 27);
        assert_eq!(node.largest_key.len(), 27);
        assert_eq!(&node.entries[0][..27], &node.smallest_key[..], "条目里 key 一律打头，区间就是首末两条的前 27 字节");
        assert_eq!(&node.entries[9][..27], &node.largest_key[..]);
        assert_eq!(node.entries[0][0], UNIT_CLASS_DATA, "最小那条是码 1（类标签 1 排在最前）");
        assert_eq!(&node.largest_key[25..], &[0, 0], "码 2 / 码 3 的 key 末尾补的那两个零");
        // 声明长度 = 条目数 × 条目宽，条目区从含预留位的头末尾（169）起。
        let declared_length = u16::from_le_bytes(output.units_by_slot[10].2[8..10].try_into().expect("切了 2 字节"));
        assert_eq!(declared_length, 550, "10 × 55（第七节 B7「mapping_declared=550」）");
        assert_eq!(index_node_header_bytes(27) + NONCE_MAC_RESERVED_BYTES as usize, 169);
    }

    /// 每个码 2 节点里的条目 key 严格递增（D8（核心索引结构） 已定项 11 的全序：逐字段无符号整数、自左向右）。
    /// 这是一条**跨结构**的绝对检查：任何一棵树把两条同 key 的条目写进同一个节点都会红，
    /// 不必等某一棵树自己那条断言想到这一格。D8 已定项 14 第 395 行（唯一写法）之后分配记录树的
    /// 内部 / 根节点也是稀疏表示、不再有空格，与其余码 2 树走同一条检查（M101、M102 的会红检查落在这里）。
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
        assert_eq!(checked_nodes, 10, "事务里十个码 2 节点：extent、inode 根、分配记录树五个节点、记账、映射、树表");
    }

    /// 条目宽字段写坏了，解析器要当场拒绝（D8（核心索引结构） 已定项 11：声明长度 = 条目数 × 条目宽）。
    /// 第七次跑的变异 M62 逼出来的：此前那条结构检查一条测试都没碰过，整段删掉也没人红。
    #[test]
    fn the_parser_refuses_an_entry_width_field_that_contradicts_the_declared_length() {
        let BuiltPool { output, .. } = built_pool();
        let mapping_unit = &output.units_by_slot[10].2;
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
        assert_eq!(distinct.len(), 10, "第七节 B7「mapping_entries=10」");
    }

    #[test]
    fn allocation_and_accounting_trees_carry_the_byte_table_numbers() {
        let BuiltPool { output, .. } = built_pool();
        assert_eq!(output.allocation_records.len(), 28, "每盘 14 条：m1 m2 与 t1–t12（deadlist 无节点、不占落点，第七节 B4）");
        assert_eq!(output.allocation_records.iter().filter(|record| record.generation == CheckpointTxg(0)).count(), 2, "mkfs 的 m1 分配代 0；m2 被 A 换下、释放代 3");
        assert_eq!(output.allocation_records.iter().filter(|record| record.generation == CheckpointTxg(3)).count(), 26, "A 的十二个落点各两盘 24 条，加 m2 那两条（已释放、释放代 3）");
        assert_eq!(output.allocation_records.iter().filter(|record| record.is_released).map(|record| record.slot.0).collect::<Vec<_>>(), vec![50178, 50178], "只有 m2 已释放");
        assert_eq!(output.allocation_records[0].key_bytes().len(), 10);
        let slots_per_device: Vec<u64> = output.allocation_records.iter().filter(|record| record.device == DeviceIdentity(0)).map(|record| record.slot.0).collect();
        assert_eq!(slots_per_device, vec![50176, 50178, 50180, 50240, 50242, 50244, 50245, 50246, 50247, 50248, 50249, 50250, 50251, 50252]);
        // D5 已定项 8（2026-09-14 用户定案）：两盘时 **15 行**，seq 一律 1、代一律 3——这一项不随分配记录树的节点数变。
        assert_eq!(output.accounting_entries.len(), 15);
        assert!(output.accounting_entries.iter().all(|entry| entry.sequence == 1 && entry.generation == CheckpointTxg(3)));
        let value_of = |statistic: u16| output.accounting_entries.iter().find(|entry| entry.statistic == statistic).expect("统计量").value;
        let rows_of = |statistic: u16| output.accounting_entries.iter().filter(|entry| entry.statistic == statistic).count();
        assert_eq!(value_of(STATISTIC_ALLOCATED_BYTES), 278_528, "每盘 17 槽 × 16384（第七节 B14「allocated_bytes」）");
        assert_eq!(value_of(STATISTIC_FREE_BYTES), 3_472_605_184, "(211968 − 17) × 16384（第七节 B14「free_bytes」）");
        assert_eq!(value_of(STATISTIC_ALLOCATED_BYTES) + value_of(STATISTIC_FREE_BYTES), UNIT_AREA_SLOTS * SLOT_BYTES);
        assert_eq!(value_of(STATISTIC_EMPTY_CLUSTER_SEGMENTS), 3310, "3312 个 64 槽段里两个被占");
        assert_eq!(value_of(STATISTIC_FRAGMENTATION_RUNS), 4, "空闲 run：[50179]、[50182, 50239]、[50241]、[50253, 单元区末]（第七节 B14）");
        assert_eq!(value_of(STATISTIC_INODE_WATERMARK), 2);
        // 准入不等式要的四项 day-1 各写一行：三项 0（读不到行是坏账，读到 0 是真的 0），defer 待释放 = 被 A 换下的 mkfs 树表 1 槽。
        for statistic in [STATISTIC_UNRECLAIMABLE_BYTES, STATISTIC_PENDING_DELETE_BYTES, STATISTIC_COMMITTED_RESERVATION_BYTES] {
            assert_eq!(value_of(statistic), 0);
        }
        assert_eq!(value_of(STATISTIC_DEFER_QUEUE_BYTES), SLOT_BYTES, "mkfs 那片第 0 版树表单元进 defer 队列");
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
        let node = parse_index_node(&output.units_by_slot[11].2).expect("树表单元");
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
        let declared_length = u16::from_le_bytes(output.units_by_slot[11].2[8..10].try_into().expect("切了 2 字节"));
        assert_eq!(declared_length, 1400, "7 × 200");
        // 预留 76 里塞一个非零字节：条目解析当场拒绝。
        let mut tampered = node.entries[0].clone();
        tampered[TREE_TABLE_ENTRY_BYTES as usize - 1] = 1;
        assert_eq!(TreeTableEntry::parse(&tampered).unwrap_err(), UnitError::Structure("树表条目预留 76 字节非零"));
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
        // C480（inode 记录的 blocks 怎么算全仓没有条款） 2026-09-23 用户定案：blocks = ⌈文件字节数 ÷ 512⌉ = ⌈3000 ÷ 512⌉ = 6
        // （逻辑长度的 512 字节块数，不表示分到的空间）；blocks 不在 InodeRecord 结构体里（它是写时算的一个派生量，
        // 不是从盘上读回来再参与判定的那种字段），这里直接按偏移 48 核对写出来的原始字节。
        assert_eq!(u64::from_le_bytes(leaf.records[0][48..56].try_into().expect("切了 8 字节")), 6, "blocks");
        let tree_table = parse_index_node(&genesis.tree_table_genesis_unit).expect("树表第 0 版");
        assert_eq!(tree_table.entries.len(), 0);
    }

    // ───────── E142 第十一次跑新增：量 1-4/8 与阳性对照 A/B 用到的算术模型的单测 ─────────

    #[test]
    fn inode_record_change_count_bytes_differ_only_in_the_low_byte_between_1_and_3() {
        let record_1 = InodeRecord { inode: 1, object_birth: CheckpointTxg(3), size: 3000, change_count: 1 }.to_bytes();
        let record_3 = InodeRecord { inode: 1, object_birth: CheckpointTxg(3), size: 3000, change_count: 3 }.to_bytes();
        let differing: Vec<usize> = (0..record_1.len()).filter(|&index| record_1[index] != record_3[index]).collect();
        assert_eq!(differing, vec![88], "改动计数 1→3 的 u64 小端表示只翻低位那一个字节（重跑登记第七节乙类）");
        assert_eq!(record_1[88], 1);
        assert_eq!(record_3[88], 3);
    }

    #[test]
    fn diff_pools_reports_a_single_byte_segment_when_only_one_byte_changes() {
        let mut old_sector = [0u8; PHYSICAL_BLOCK_BYTES as usize];
        old_sector[10] = 1;
        let mut new_sector = old_sector;
        new_sector[10] = 3;
        let old_pool = Pool { devices: vec![SparseDevice { sectors: std::collections::BTreeMap::from([(5u64, old_sector)]) }] };
        let new_pool = Pool { devices: vec![SparseDevice { sectors: std::collections::BTreeMap::from([(5u64, new_sector)]) }] };
        let differences = diff_pools(&old_pool, &new_pool);
        assert_eq!(differences.len(), 1);
        assert_eq!(differences[0].device, 0);
        assert_eq!(differences[0].offset, 5 * PHYSICAL_BLOCK_BYTES + 10);
        assert_eq!(differences[0].length, 1);
        assert_eq!(differences[0].old, vec![1]);
        assert_eq!(differences[0].new, vec![3]);
    }

    #[test]
    fn diff_pools_merges_adjacent_differing_bytes_and_keeps_separated_ones_apart() {
        let mut old_sector = [0u8; PHYSICAL_BLOCK_BYTES as usize];
        let mut new_sector = old_sector;
        new_sector[20] = 9;
        new_sector[21] = 9; // 相邻两字节都变 ⇒ 合并成一段，长度 2
        new_sector[100] = 9; // 隔得远的一字节 ⇒ 单独一段
        old_sector[300] = 5;
        new_sector[300] = 5; // 这个字节没变，不该出现在任何段里
        let old_pool = Pool { devices: vec![SparseDevice { sectors: std::collections::BTreeMap::from([(0u64, old_sector)]) }] };
        let new_pool = Pool { devices: vec![SparseDevice { sectors: std::collections::BTreeMap::from([(0u64, new_sector)]) }] };
        let mut differences = diff_pools(&old_pool, &new_pool);
        differences.sort_by_key(|segment| segment.offset);
        assert_eq!(differences.len(), 2, "20-21 合并成一段，100 单独一段");
        assert_eq!((differences[0].offset, differences[0].length), (20, 2));
        assert_eq!((differences[1].offset, differences[1].length), (100, 1));
    }

    #[test]
    fn diff_pools_detects_a_flipped_byte_at_the_exact_injected_position() {
        let BuiltPool { recording, .. } = built_pool();
        let base = recording.pool.clone();
        let mut mutated = base.clone();
        let leaf_base = SlotNumber(SLOT_INODE_LEAF).device_offset();
        flip_byte(&mut mutated, DeviceIdentity(0), leaf_base, 88);
        let differences = diff_pools(&base, &mutated);
        assert_eq!(differences.len(), 1, "阳性对照：注入一个字节只能测出一段");
        assert_eq!(differences[0].device, 0);
        assert_eq!(differences[0].offset, leaf_base.0 + 88);
        assert_eq!(differences[0].length, 1);
    }

    #[test]
    fn classify_offset_finds_the_inode_leaf_change_count_field_and_the_root_record_checksum() {
        let BuiltPool { output, .. } = built_pool();
        let parameters = PoolParameters::settled_two_devices();
        let (region, slot) = ring_target_for_publish(CheckpointTxg(FIRST_TRANSACTION_TXG));
        let root_device = parameters.region_devices[region as usize].0;
        let root_offset = ring_slot_offset(region, slot).0;
        let catalog = build_structure_catalog(&output, root_device, root_offset);
        let leaf_base = SlotNumber(SLOT_INODE_LEAF).device_offset().0;
        assert_eq!(classify_offset(&catalog, 0, leaf_base + 224), Some(("inode_leaf", 224)), "记录区 136 起 + 记录内偏移 88 = 224");
        assert_eq!(classify_offset(&catalog, 0, leaf_base), Some(("inode_leaf", 0)));
        assert_eq!(classify_offset(&catalog, root_device, root_offset + 138), Some(("root_record", 138)), "根记录自证校验和的结构内偏移");
        assert_eq!(classify_offset(&catalog, 0, 999_999_999_999), None, "远处的偏移落不进任何候选结构");
    }

    #[test]
    fn explain_unit_offset_finds_the_change_count_field_and_the_checksum_regions_but_not_the_common_prefix() {
        let record_start = (PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES) as u64;
        let (explained, reason) = explain_unit_offset(TransactionUnit::InodeLeaf, record_start + 88);
        assert!(explained);
        assert!(reason.contains("改动计数"), "reason={reason}");
        assert!(explain_unit_offset(TransactionUnit::InodeLeaf, 15).0, "偏移 15 落在头校验和 [10,42)");
        assert!(explain_unit_offset(TransactionUnit::InodeLeaf, 89).0, "偏移 89 是码 3 载荷 CRC");
        assert!(!explain_unit_offset(TransactionUnit::InodeLeaf, 5).0, "偏移 5 是共同前缀，不在头校验和/载荷CRC/记录区里，量 3 该记 gap");
    }

    #[test]
    fn explain_inode_root_offset_finds_the_two_location_checksums_that_point_at_the_inode_leaf() {
        let (explained_0, reason_0) = explain_unit_offset(TransactionUnit::InodeRoot, 225);
        let (explained_1, reason_1) = explain_unit_offset(TransactionUnit::InodeRoot, 239);
        assert!(explained_0 && reason_0.contains("位置条目单元校验和"), "reason_0={reason_0}");
        assert!(explained_1 && reason_1.contains("位置条目单元校验和"), "reason_1={reason_1}");
    }

    #[test]
    fn change_count_bullets_are_all_covered_only_when_all_six_offsets_are_hit() {
        let mut bullets = ChangeCountBullets::default();
        assert!(!bullets.all_covered());
        bullets.record("inode_leaf", 136 + 88, 1);
        bullets.record("inode_leaf", 15, 4);
        bullets.record("inode_root", 225, 4);
        bullets.record("inode_root", 92, 4);
        bullets.record("tree_table", 50, 4);
        assert!(!bullets.all_covered(), "根记录那一格还没打到，六处不能全过");
        bullets.record("root_record", 150, 4);
        assert!(bullets.all_covered());
    }

    #[test]
    fn structure_sets_differing_by_one_are_reported_unstable() {
        let left: std::collections::BTreeSet<&str> = ["inode_leaf", "inode_root"].into_iter().collect();
        let right: std::collections::BTreeSet<&str> = ["inode_leaf", "inode_root", "mapping_root"].into_iter().collect();
        assert!(!structure_sets_match(&left, &right), "量 8 判据是「相等」，不是「相差 ≤ 1 段」（V6 判别力自证）");
    }

    #[test]
    fn run_full_pipeline_writes_the_requested_change_count_into_the_inode_leaf_record() {
        // V1 变异（改动计数字面值改回 1）锚在 run_full_pipeline 内部的转发行：main() 自己那个独立调用点没有单测直接盖到，
        // M_A / M_B / G1 / G2 都经这个函数构造，锚在这里等价地测住了「装置真的按参数写出对应的 change_count」。
        for &value in &[1u64, FIRST_TRANSACTION_TXG, 4u64] {
            let (image, output) = run_full_pipeline(&PoolParameters::settled_two_devices(), value, &sample_file());
            let leaf = output.units_by_slot.iter().find(|(_, unit, _)| *unit == TransactionUnit::InodeLeaf).expect("t3 是 inode 叶");
            let parsed = parse_packed_unit(&leaf.2).expect("inode 叶自检要过");
            assert_eq!(InodeRecord::parse(&parsed.records[0]).expect("记录三段恒零要过").change_count, value);
            let record_field_offset = (PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES) as usize + 88;
            let sector = image.read(DeviceIdentity(0), leaf.0.device_offset(), PHYSICAL_BLOCK_BYTES as usize);
            assert_eq!(&sector[record_field_offset..record_field_offset + 8], value.to_le_bytes().as_slice(), "盘上的字节要与请求的 change_count 一致");
        }
    }

    #[test]
    fn run_full_pipeline_matches_the_manually_assembled_pool_for_the_same_change_count() {
        let BuiltPool { recording, .. } = built_pool();
        let (image, _) = run_full_pipeline(&PoolParameters::settled_two_devices(), FIRST_TRANSACTION_TXG, &sample_file());
        assert_eq!(recording.pool.devices.len(), image.devices.len());
        for device_index in 0..image.devices.len() {
            assert_eq!(recording.pool.devices[device_index].sectors, image.devices[device_index].sectors, "同参数同口径应逐字节相同");
        }
    }

    // ───────── E142 第十一次跑（量 5，crates 侧补上之后）新增：SHA-256 与 crates 快照解析的单测 ─────────

    #[test]
    fn sha256_matches_the_two_standard_test_vectors() {
        assert_eq!(sha256_hex(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }

    #[test]
    fn sha256_of_the_actual_root_record_bytes_matches_the_crates_side_reference_value() {
        // 这个十六进制串与 sha256 都是 2026-09-18 从 `cargo run -p singlefs-harness --bin first_transaction_region_bytes`
        // 的真实产出里摘的一行（`region=root_record device=0`），用来钉住「这份手写 SHA-256 与 crates 侧算的是同一个函数」，
        // 不是只对着标准测试向量自证。
        let hex = "534653525f5346532d453134322d303030312d000000000001000000030000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000048c40000000024acb30e0100000048c40000000024acb30e010000000000000013000000000000000000000000000000afc75d3e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c40000000021aa2fd40100000000c40000000021aa2fd40000000000000000000000000000000000000000000000000000000000000000000000000000000000000f0000000000000003000000000000000000000047c400000000bedfaead0100000047c400000000bedfaead01000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        assert_eq!(hex_decode(hex).len(), 512);
        assert_eq!(sha256_hex(&hex_decode(hex)), "b2c014f937018d4e11d74410302b319efa0feda62b6b7073446bcb64560dd51f");
    }

    #[test]
    fn hex_decode_round_trips_hex_bytes() {
        assert_eq!(hex_decode("00ff10"), vec![0x00, 0xff, 0x10]);
        assert_eq!(hex_decode(""), Vec::<u8>::new());
    }

    #[test]
    fn parse_result_line_reads_key_value_pairs_and_ignores_the_leading_marker() {
        let fields = parse_result_line("E7RESULT name=impl_region_bytes region=root_record device=0 offset=1052672 length=512 sha256=abc123");
        assert_eq!(fields.get("region").map(String::as_str), Some("root_record"));
        assert_eq!(fields.get("device").map(String::as_str), Some("0"));
        assert_eq!(fields.get("sha256").map(String::as_str), Some("abc123"));
        assert_eq!(fields.get("name").map(String::as_str), Some("impl_region_bytes"));
    }

    #[test]
    fn byte_diff_summary_finds_the_first_differing_byte_and_counts_the_rest() {
        assert_eq!(byte_diff_summary(&[1, 2, 3], &[1, 2, 3]), (None, Some(0)));
        assert_eq!(byte_diff_summary(&[1, 2, 3], &[1, 9, 9]), (Some(1), Some(2)));
        assert_eq!(byte_diff_summary(&[1, 2], &[1, 2, 3]), (Some(0), Some(3)), "长度不等直接报整段");
    }

    #[test]
    fn impl_bytes_equal_reports_true_when_a_synthetic_snapshot_line_has_the_same_sha256() {
        // 端到端钉一遍量 5 的关键逻辑：给一行手工拼的「crates 快照」，装置读出来的字节算出的 sha256 与快照里写的一样 ⇒ equal=true；
        // 不给参数（旧调用方式）不炸、只是找不到对应行。
        let device_bytes = b"hello region bytes".to_vec();
        let expected_sha256 = sha256_hex(&device_bytes);
        let snapshot_line = format!("E7RESULT name=impl_region_bytes region=root_record device=0 offset=0 length={} sha256={expected_sha256} hexadecimal_extent=whole_region hexadecimal={}", device_bytes.len(), hex_bytes(&device_bytes));
        let fields = parse_result_line(&snapshot_line);
        assert_eq!(fields.get("sha256").map(String::as_str), Some(expected_sha256.as_str()));
        assert_eq!(hex_decode(fields.get("hexadecimal").expect("hexadecimal 字段")), device_bytes);
    }

    // E142 第十五次跑步④：`compare_region`/`RegionComparison`（连同它们的 6 条单测、`snapshot_fields` 辅助函数）
    // 被 Q142.1 的新比对器取代——新比对器按 (设备, 偏移, 长度) 配对，crates 导出恒发整段十六进制（窗口里最大
    // 一次写 32768 字节，摆得出），不再需要「declared length 不等」「head_and_tail 抽样」这两种旧读法，
    // 直接删掉旧函数与旧测试（`code-discipline.md`「用不到的代码删掉」），不留驻留的死代码。

    // ───────── E142 第十五次跑步④：Q142.1 新比对器的单测（第九节 M108、M109 钉在这两条函数上） ─────────

    fn sample_impl_line(device: u32, offset: u64, length: u64, sha256: &str) -> ParsedWriteLine {
        let line = format!("E7RESULT name=device_region_bytes step=0 device={device} offset={offset} length={length} kind=write sha256={sha256} hexadecimal=00");
        let fields = parse_result_line(&line);
        ParsedWriteLine {
            device: fields.get("device").and_then(|value| value.parse().ok()).unwrap_or(0),
            offset: fields.get("offset").and_then(|value| value.parse().ok()).unwrap_or(0),
            length: fields.get("length").and_then(|value| value.parse().ok()).unwrap_or(0),
            sha256: fields.get("sha256").cloned().unwrap_or_default(),
            hexadecimal: fields.get("hexadecimal").cloned(),
        }
    }

    #[test]
    fn find_matching_impl_write_tells_two_devices_at_the_same_offset_apart() {
        // M108：按名字配对退回旧「量 5」的失败模式——两块设备在同一偏移各写不同内容，按名字配对分不清是哪一块。
        // 这里干脆没有名字字段，配对键就是 (设备, 偏移, 长度)；device=0 与 device=1 在同一偏移各有一条。
        let impl_writes = vec![
            sample_impl_line(0, 100, 10, "sha-for-device-0"),
            sample_impl_line(1, 100, 10, "sha-for-device-1"),
        ];
        assert_eq!(find_matching_impl_write(&impl_writes, 0, 100, 10), Some(0));
        assert_eq!(find_matching_impl_write(&impl_writes, 1, 100, 10), Some(1));
    }

    #[test]
    fn find_matching_impl_write_refuses_a_length_mismatch_at_the_same_device_and_offset() {
        let impl_writes = vec![sample_impl_line(0, 100, 10, "sha")];
        assert_eq!(find_matching_impl_write(&impl_writes, 0, 100, 20), None, "长度不等就是配不上，不是「差不多就算」");
    }

    #[test]
    fn find_matching_impl_write_reports_none_when_nothing_matches() {
        let impl_writes = vec![sample_impl_line(0, 100, 10, "sha")];
        assert_eq!(find_matching_impl_write(&impl_writes, 0, 999, 10), None);
    }

    #[test]
    fn unmatched_crates_indices_lists_every_index_the_model_never_claimed() {
        // M109：比对器只从模型一侧遍历会漏掉「crates 多写了一次」——这条函数就是「配不上的两边都报」里 crates 那一半。
        assert_eq!(unmatched_crates_indices(&[true, false, false, true]), vec![1, 2]);
    }

    #[test]
    fn unmatched_crates_indices_is_empty_when_every_impl_write_was_claimed() {
        assert_eq!(unmatched_crates_indices(&[true, true, true]), Vec::<usize>::new());
    }

    #[test]
    fn compare_paired_write_reports_equal_when_sha256_matches() {
        let model_bytes = b"same bytes on both sides".to_vec();
        let sha = sha256_hex(&model_bytes);
        let impl_line = sample_impl_line(0, 0, model_bytes.len() as u64, &sha);
        assert_eq!(compare_paired_write(&model_bytes, &sha, &impl_line), (true, None, Some(0)));
    }

    #[test]
    fn compare_paired_write_localizes_the_first_difference_when_sha256_differs() {
        let model_bytes = vec![1u8, 2, 3, 4];
        let impl_bytes = vec![1u8, 2, 9, 4];
        let mut impl_line = sample_impl_line(0, 0, 4, "different_from_model");
        impl_line.hexadecimal = Some(hex_bytes(&impl_bytes));
        assert_eq!(compare_paired_write(&model_bytes, &sha256_hex(&model_bytes), &impl_line), (false, Some(2), Some(1)));
    }
}
