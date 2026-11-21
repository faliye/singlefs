//! E142：新池新建文件的干跑——把 `.claude/kb/layout/01-first-txn.md` 那张字节表原样写成字节，
//! mkfs + 一个小文件 + 一次发布，两块 512 槽的盘；冷启动读回；按 D13（验证路线） 已定项 4 的层 0
//! 崩溃状态集合逐个恢复。目的是让代码把「字节表里哪几格写不出来、哪几条已定条款在字节层面互相顶着」逼出来：
//! 每一处装置不得不自己补的取法都报成一行 `gap`。跑前登记 `research/prompts/e142-preregistration.md`。
//!
//! 判 oracle 用 E77（发布的持久顺序） 判据 1：根槽已持久而恢复 ≠ 新态或走读失败即违例；根槽未持久时新旧两态都合法。
//! 纯确定性：fsid、时间戳都是固定参数，跑 N 遍逐字节一样。

use e7_index_bench::{usize_from_u64, Emitter};
use std::collections::BTreeMap;

mod allocation_tree_and_extent_leaves;
mod byte_types_writer_and_reader;
mod checksums_and_hex;
mod crash_segments_and_state_evaluation;
mod devices_pools_and_recorded_writes;
mod file_publication;
mod journal_and_record_formats;
mod journal_labels_warm_up_and_reservation;
mod journal_without_root_scenarios_and_window_writes;
mod layout_field_rows_and_anchors;
mod outcome_predictions_and_grouped_enumeration;
mod paired_write_comparison_and_write_list_rows;
mod pointers_and_unit_prefixes;
mod pool_parameters_and_mkfs;
mod recovery_and_journal_replay;
mod role_label_and_text_number_checks;
mod root_record_and_system_configuration;
mod single_publish_flow_probes_and_pool_diff;
mod stream_sections_and_verdict_fields;
mod structure_catalog_and_offset_explanations;
#[cfg(test)]
mod test_support_for_the_pool_and_streams;
mod unit_builders_and_parsers;

use self::allocation_tree_and_extent_leaves::*;
use self::byte_types_writer_and_reader::*;
use self::checksums_and_hex::*;
use self::crash_segments_and_state_evaluation::*;
use self::devices_pools_and_recorded_writes::*;
use self::file_publication::*;
use self::journal_and_record_formats::*;
use self::journal_labels_warm_up_and_reservation::*;
use self::journal_without_root_scenarios_and_window_writes::*;
use self::layout_field_rows_and_anchors::*;
use self::outcome_predictions_and_grouped_enumeration::*;
use self::paired_write_comparison_and_write_list_rows::*;
use self::pointers_and_unit_prefixes::*;
use self::pool_parameters_and_mkfs::*;
use self::recovery_and_journal_replay::*;
use self::role_label_and_text_number_checks::*;
use self::root_record_and_system_configuration::*;
use self::single_publish_flow_probes_and_pool_diff::*;
use self::stream_sections_and_verdict_fields::*;
use self::structure_catalog_and_offset_explanations::*;
#[cfg(test)]
use self::test_support_for_the_pool_and_streams::*;
use self::unit_builders_and_parsers::*;

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
/// D18（块里携带什么信息） 已定项 14 / 已定项 16：
/// nonce 12 + MAC 16 + 算法类型 1 = 29 字节预留位，紧接类身份段之后。
const NONCE_MAC_RESERVED_BYTES: u64 = 29;
/// D22（单元原子性怎么合成） 已定项 7 的字段表：magic 4 + fsid 16 + flags 4 + 实例代号 4 + checkpoint_txg 8
/// + 树表单元指针 86 + 树 ID 水位 8 + 回退下界 F 8 + 自证校验和 32 + 实例表单元指针 86
/// + 中央映射树根指针 86 + 分配记录树根指针 86（C512（树表 0 条的一版上被换下的实例表记在哪没有条款））
/// + 算法类型 1 + nonce 12 + MAC 16（末尾三段）。
const ROOT_RECORD_BYTES: u64 = 457;
/// D19（块指针的结构与宽度预算） 已定项 4：设备 4 + 16 KiB 槽号 6 + 密文校验和 4。
const LOC_ENTRY: u64 = 14; // naming-lint:external 名字由 kb 的 format-const 登记位定（D19 已定项 4），门禁按这个名字绑值
/// D19（块指针的结构与宽度预算） 已定项 7 / 已定项 11 的偏移表：MAC 16（偏移 0）+ nonce 12（16）+ 算法类型 1（28）
/// + 压缩算法码 1（29）+ 压后长度 2（30）+ extent 距起点偏移 2（32）+ 出生树 8（34）+ 出生 txg 8（42）。
///   头部共 50 字节，含压缩那两段。
const POINTER_HEAD_BYTES: u64 = 50;
/// D19（块指针的结构与宽度预算） 已定项 8：指向码 2 / 码 3 的指针 = 头部 + 位置条目 × 2 + 实例代号 4 + 出生序号 4。
const NODE_POINTER_BYTES: u64 = 86;
/// D19（块指针的结构与宽度预算） 已定项 8：指向码 1 的指针 = 头部 + 位置条目 × 2 + 写序 10。
const DATA_POINTER_BYTES: u64 = 88;
/// D8（核心索引结构） 已定项 8：树 ID 8 打头 + 条目长度 2 + 树的种类 2 + flags 2
/// + 根指针 86 + previous_snapshot_txg 8 + 诞生 txg 8 + 头 ID 8 + 预留 76（D8（核心索引结构） 已定项 8）。
const TREE_TABLE_ENTRY_BYTES: u64 = 200;
/// D8（核心索引结构） 已定项 6。
const INODE_RECORD_BYTES: u64 = 140;
/// D8（核心索引结构） 已定项 6：分隔 key 8 + 身份引用 26 + 子指针 86。
const INODE_INTERNAL_ENTRY: u64 = 120;
/// D8（核心索引结构） 已定项 3 + D19 已定项 8：key 24 + 指向码 1 的指针 88。
/// E142 第十五次跑起，新池新建文件不再写这个宽度（extent 树改按位置寻址、根即上段叶，见下），
/// 常量与它的 `name=width` 行留着标「附带」（下段叶条目，新池新建文件走不到）。
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
/// extent 树内部条目 110 = key 24 + 子指针 86。新池新建文件只有一个 inode，走不到内部节点（留给下段路径）。
#[allow(dead_code, reason = "上段内部节点，新池新建文件这个装置走不到；单测钉住数值，留着给以后写多 inode 路径用")]
const EXTENT_TREE_INTERNAL_ENTRY_BYTES: u64 = 110;
/// extent 树内部扇出 147 = (16384 − 163) ÷ 110。
#[allow(dead_code, reason = "上段内部节点，新池新建文件这个装置走不到；单测钉住数值，留着给以后写多 inode 路径用")]
const EXTENT_TREE_INTERNAL_FANOUT: u64 = 147;
/// extent 树下段叶罩 144 个数据单元；下段 key 的字节偏移 = 单元号 × 32634（新池新建文件走不到下段）。
#[allow(dead_code, reason = "下段叶，新池新建文件这个装置走不到；单测钉住数值，留着给以后写下段路径用")]
const EXTENT_TREE_LOWER_LEAF_DATA_UNITS: u64 = 144;
/// D3（空间分配） 已定项 7 + 已定项 11：key 10（设备 4 + 槽号 6）+ value 10（跨度 2 + 代 8）。
const ALLOCATION_RECORD_BYTES: u64 = 20;
/// D3（空间分配） 已定项 11：跨度段进 value ⇒ 分配记录树的 key 宽 10。
const ALLOCATION_KEY_BYTES: usize = 10;
/// D5（快照 / 空间记账机制） 已定项 5 + D8 已定项 7：key 22 + value 8 + seq 4。
const ACCOUNTING_ENTRY_BYTES: u64 = 34;
/// D19（块指针的结构与宽度预算） 已定项 6 / 已定项 10：中央映射的 key **一律 27**——
/// 码 1 天然 27（类标签 1 + 出生树 8 + 出生 txg 8 + 写序 10），码 2 / 码 3 的 25 字节末尾补零到 27。
const MAPPING_KEY_BYTES: u64 = 27;
/// 码 2 / 码 3 的映射 key 里有含义的那一段：类标签 1 + 出生树 8 + 出生 txg 8 + 实例代号 4 + 出生序号 4 = 25，其后两字节恒零。
const MAPPING_KEY_NODE_SIGNIFICANT_BYTES: u64 = 25;
/// 映射条目 = key 27 + value（位置条目 × 2）= **一律 55**（D19 已定项 6 / 已定项 10）。
const MAPPING_ENTRY_BYTES: u64 = 55;
/// D18（块里携带什么信息） 已定项 11：实例表一片记录宽 88（kind 1 + 有无下一片 1 + 位置指针 86 + 预留 0；
/// 记录宽 88，打包记录定长要求 kind 0 / kind 1 同宽）。
const INSTANCE_ROW_BYTES: u64 = 88;
/// D23（journal 的角色与格式） 已定项 12。
const JOURNAL_RECORD_BYTES: u64 = 4096;
/// 记录头（D23（journal 的角色与格式） 已定项 4 的字段表）：十个字段 78（`JOURNAL_HEADER_TEN_FIELD_BYTES`，
/// 含「记录标志」那一字节，位 0 = 本次发布末条，已定项 17） + 事务号 8 + 提交标记 1 + 本次发布内序号 4
/// + 反向链 4 + 载荷校验和 4 + 新根段 364 + fsid 8 + MAC 16 = 487（D23 已定项 4 节标题「头 487 字节」）。
///   4096 的记录装 (4096 − 487) / 56 = 64 个点名项（D23 已定项 17），512 扇区上余 25 字节（已定项 4）。
const JOURNAL_HEADER_BYTES: u64 = 487;
/// 十个字段那一段另登记成格式常量，好让门禁分得清它与整个头（D23 已定项 4）。
const JOURNAL_HEADER_TEN_FIELD_BYTES: u64 = 78;
/// D23（journal 的角色与格式） 已定项 15：新根段 = 树表单元指针 86 + 中央映射树根指针 86 + 树 ID 水位 8 + 回退下界 F 8
/// + 实例表单元指针 86 + 分配记录树根指针 86 + 卸载记号 4（七项，次序同此）。
const JOURNAL_NEW_ROOT_SEGMENT_BYTES: u64 = 364;
const _: () = assert!(JOURNAL_NEW_ROOT_SEGMENT_BYTES == 4 * NODE_POINTER_BYTES + 8 + 8 + 4);
/// D23（journal 的角色与格式） 已定项 4 口径的点名项宽度；构成无落点，装置按字节表六的预想构成写。
const JOURNAL_NAMED_ENTRY_BYTES: u64 = 56;
/// 系统配置字段表合计（D22（单元原子性怎么合成） 已定项 9 + 已定项 15）：含四个字段——
/// 自举头的写入者身份 20 与校验和算法标识 1、几何段的 mkfs 时 physical_block_size 4 与扩展点声明值 N 4，共 29；
/// 字段表末尾是回退下界 F 8 字节（住 [481, 489)），合计 489。
const SYSTEM_CONFIGURATION_BYTES: u64 = 489;
/// 头校验和 / 自证校验和的字段宽度（D18 已定项 7、D22 已定项 7、D23 已定项 4 同口径）。
/// 算法由 D18（块里携带什么信息） 已定项 17 定：字段里放 CRC32C 4 字节 + 28 字节零。
const WIDE_CHECKSUM_BYTES: u64 = 32;
const WIDE_CHECKSUM_CRC_BYTES: usize = 4;

/// 码 2 节点头里 key 区间之外的部分（D8（核心索引结构） 已定项 11 + D18（块里携带什么信息） 已定项 18 的偏移表）：
/// 共同前缀 42 + 树 ID 8 + 层级 1 + key 宽 1 + 诞生代号 8 + fsid 8 + 写序 4 + 出生序号 4 + 载荷 CRC 4
/// + 预留 2 + 条目数 2 + 条目宽 2 = 86；含 29 字节预留位的头 = 115 + 2 × key 宽。
///   key 宽住**偏移 51**——定位它不需要先知道 k。
const INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE: u64 = 86;
/// 码 2 头里 key 宽那一格的偏移（D18 已定项 18）：共同前缀 42 + 树 ID 8 + 层级 1 = 51。
const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;

/// 字节表零：系统配置槽 0 / 1 的设备内偏移。槽距 = 固定结构槽距 4096，与槽宽同值 ⇒ 两个槽首尾相接。
const SYSTEM_CONFIGURATION_SLOT_OFFSETS: [u64; 2] = [0, 4096];
/// D22（单元原子性怎么合成） 已定项 2 的槽宽那一格：
/// **系统配置槽宽是格式常量 4096**，不等于挂载时探测到的 `physical_block_size`。
/// 整槽校验和罩这 4096 字节含补齐（D18（块里携带什么信息） 已定项 17），489 字节的字段表在槽里余 3607。
/// ⚠️ 它只管系统配置：根槽仍按判定宽度 512 写（字节表七）。
const SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = 4096;
/// 字节表零：根环起点 1 MiB（槽 64）、P = 3、chunk = 1 MiB、每区 8 槽、槽距 4096（预想）。
const RING_START_OFFSET: u64 = 1 << 20;
const RING_PRIME_STEP: u64 = 3;
const RING_CHUNK_BYTES: u64 = 1 << 20;
const RING_REGIONS: u64 = 3;
const RING_SLOTS_PER_REGION: u64 = 8;
const RING_SLOT_SPACING: u64 = 4096;
/// D2（RAID 条带策略） 已定项 7：根环三个区域的设备归属第一版**写死** 0 / 1 / 0，不是 mkfs 参数。
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
/// 只剩主点的单测拿它钉 262 144；写路与读路按 `PoolParameters::device_slots`（第二十二次跑第二段起每盘字节数是参数）。
#[cfg(test)]
const DEVICE_SLOTS: u64 = slots_of_device_bytes(DEVICE_BYTES);
/// D5（快照 / 空间记账机制） 已定项 7：准入不等式里的「容量」= 单元区大小（起始槽号到盘尾），固定结构不在容量里。
/// 每盘字节数是参数（E142 第二十二次跑第二段 P8G），主点那一份常量 `UNIT_AREA_SLOTS` 由它在 `DEVICE_BYTES` 上取值。
const fn unit_area_slots_of_device_bytes(device_bytes: u64) -> u64 {
    (device_bytes - UNIT_AREA_START_SLOT * SLOT_BYTES) / SLOT_BYTES
}
const UNIT_AREA_SLOTS: u64 = unit_area_slots_of_device_bytes(DEVICE_BYTES);
/// E142 第二十二次跑第二段的第二个几何 P8G：两块盘、每盘 8 GiB（第二十二次登记第一节「第 2 行」的读法）。
const EIGHT_GIBIBYTE_POINT_DEVICE_BYTES: u64 = 8 << 30;
/// D3（空间分配） 已定项 10 ①：聚簇段段长 64 个 16 KiB 槽；开放段 = 单元区内最低的、64 槽对齐的全空段。
const CLUSTER_SEGMENT_SLOTS: u64 = 64;

/// 落点（D3（空间分配） 已定项 10）：m1 / m2 是 mkfs 的两个固定单元，从单元区起点起铺；
/// 用户数据 t1 取不在开放聚簇段里的、最低的 32768 对齐空槽对；t2–t8 是提交内生块，从开放聚簇段 bump。
const SLOT_INSTANCE_TABLE: u64 = 50176;
const SLOT_TREE_TABLE_GENESIS: u64 = 50178;
const SLOT_DATA_UNIT: u64 = 50180;
const OPEN_CLUSTER_SEGMENT_START_SLOT: u64 = 50240;
/// D3（空间分配） 已定项 10 ⑤：提交内生块从开放段 bump 的次序 =
/// 按树 ID 升序、树内先叶后根、中央映射树倒数第二、树表单元最末。游标从 50240 起：
/// t2 extent 根（树 11）拿 50240 ⇒ 游标 50241；t3 inode 叶是码 3 容器、按「数据单元」那一档要起点 32768 对齐
/// ⇒ 跳过 50241 拿 50242–50243；其后依次 bump。**槽 50241 因此空着**，它自己是一段空闲 run。
/// 分配记录树按位置寻址拆成 5 个节点（第七节 B2）：t5..t9 落 50245..50249，
/// 记账 / 映射 / 树表跟着顺延到 50250..50252。
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

/// D8（核心索引结构） 已定项 11：第一版树 ID 从 65536 起编（种类字段 2 字节取不到的最小值，
/// 与树的种类码错开）。树表单元与实例表单元的树 ID 段写 0（无归属，D5（快照 / 空间记账机制） 已定项 10 登记的保留值）。
const TREE_IDENTIFIER_NONE: u64 = 0;
const TREE_IDENTIFIER_EXTENT: u64 = 65536;
const TREE_IDENTIFIER_INODE: u64 = 65537;
const TREE_IDENTIFIER_ALLOCATION: u64 = 65538;
const TREE_IDENTIFIER_ACCOUNTING: u64 = 65539;
const TREE_IDENTIFIER_MAPPING: u64 = 65540;
/// D6 已定项 2：livelist 共享树 day-1 注册，新池新建文件根指针为零。
const TREE_IDENTIFIER_LIVELIST: u64 = 65541;
/// D5 已定项 6：稀疏旁表树 day-1 注册，新池新建文件根指针为零。
const TREE_IDENTIFIER_SHARE_COUNT: u64 = 65542;
/// D5 已定项 11：deadlist 树 day-1 注册进树表，新池新建文件根指针为零、一条条目都不写。
const TREE_IDENTIFIER_DEADLIST: u64 = 65543;
/// D18（块里携带什么信息） 已定项 24 第 6 条：`deleted_inodes` 树、墓碑树、谱系树、logged_ops 树 day-1 注册（D8 已定项 11 条文逐个给号），
/// 新池新建文件根指针为零。
const TREE_IDENTIFIER_DELETED_INODES: u64 = 65544;
const TREE_IDENTIFIER_TOMBSTONE: u64 = 65545;
const TREE_IDENTIFIER_LINEAGE: u64 = 65546;
const TREE_IDENTIFIER_LOGGED_OPERATIONS: u64 = 65547;
/// 退役树：D18（块里携带什么信息） 已定项 24 第 6 条同批 day-1 零根注册，树 ID 照 D8（核心索引结构） 已定项 11 条文（`layout/03`「树表退役树条目」那一行）。
const TREE_IDENTIFIER_RETIREMENT: u64 = 65548;
/// mkfs 那一刻一棵树都还没有，水位就是第一个要发的树 ID（D8 已定项 11「mkfs 时水位 65536」，单独一个常量，不借 extent 的号）；
/// 发布之后 = 本次发出的最高树 ID + 1（D8 已定项 8 ②，最后一棵退役树的号 + 1）。
const TREE_IDENTIFIER_WATERMARK_AT_MKFS: u64 = 65536;
const TREE_IDENTIFIER_WATERMARK_AFTER_PUBLISH: u64 = TREE_IDENTIFIER_RETIREMENT + 1;
/// 树的种类的码（D8（核心索引结构） 已定项 9 的登记表：0 无、1 extent、2 inode、3 分配记录、4 记账、
/// 5 中央映射、6 livelist 共享树、7 稀疏旁表、8 deadlist、9 `deleted_inodes`、10 墓碑树、11 谱系树、12 logged_ops、13 退役树；
/// 码 8 随 D5 已定项 11 进登记表，码 9..12 随 D18 已定项 24 第 6 条进登记表，码 13 随 D18 已定项 11 的打包记录类型 3 进登记表）。
const TREE_KIND_EXTENT: u16 = 1;
const TREE_KIND_INODE: u16 = 2;
const TREE_KIND_ALLOCATION: u16 = 3;
const TREE_KIND_ACCOUNTING: u16 = 4;
const TREE_KIND_MAPPING: u16 = 5;
const TREE_KIND_LIVELIST: u16 = 6;
const TREE_KIND_SHARE_COUNT: u16 = 7;
const TREE_KIND_DEADLIST: u16 = 8;
const TREE_KIND_DELETED_INODES: u16 = 9;
const TREE_KIND_TOMBSTONE: u16 = 10;
const TREE_KIND_LINEAGE: u16 = 11;
const TREE_KIND_LOGGED_OPERATIONS: u16 = 12;
const TREE_KIND_RETIREMENT: u16 = 13;

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
/// D5 已定项 8：准入不等式要的四个统计量 day-1 各写一行值 0，标签是 D5 已定项 4 那张表的行号。
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

/// D19（块指针的结构与宽度预算） 已定项 7 / 已定项 11：指针里压缩那两段的第一版取值。
/// 登记表住 D9（加密） 已定项 14：码 0 = 不压缩；码 0 时压后长度恒 0，非 0 判该指针损坏。
const POINTER_COMPRESSION_CODE_NONE: u8 = 0;
const POINTER_COMPRESSED_LENGTH_NONE: u16 = 0;

/// D5（快照 / 空间记账机制） 已定项 9：树表条目从 32 字节预留里切 8 字节存「头 ID」。
/// inode 树条目写自己（12）、extent 树条目写它服务的那个头（12），分配记录 / 记账 / livelist / 旁表 / deadlist 写 0。
const TREE_TABLE_HEAD_IDENTIFIER_NONE: u64 = 0;

/// D22（单元原子性怎么合成） 已定项 9：系统配置自举头的「写入者身份」=
/// 实现标识 16 字节 ASCII 零补齐 + 版本 4 字节；第一版写 `singlefs-rs` 与 1（I-1.5（系统配置记管道身份）、D17（实现分层与第三方管道） 债 3）。
const WRITER_IDENTITY_NAME: &[u8] = b"singlefs-rs";
const WRITER_IDENTITY_NAME_BYTES: usize = 16;
const WRITER_IDENTITY_VERSION: u32 = 1;
/// 同一条已定项的「校验和算法标识」登记表：0 无效、1 CRC-32C；第一版写 1（I-2.2（校验和算法全盘一致）、C309（32 字节校验和的算法没定））。
const CHECKSUM_ALGORITHM_CRC32_CASTAGNOLI: u8 = 1;
/// 几何段的两个字段：mkfs 时探测到的 physical_block_size（D13（验证路线） 已定项 5 的比对要它），
/// 与扩展点声明值 N（D21（权威态与派生态的分界） 已定项 8，第一版 0 字节）。
const MKFS_PHYSICAL_BLOCK_BYTES: u32 = 512;
const EXTENSION_POINT_DECLARED_BYTES: u32 = 0;

const UNIT_MAGIC: [u8; 4] = *b"SFSU";
const ROOT_MAGIC: [u8; 4] = *b"SFSR";
const SYSTEM_CONFIGURATION_MAGIC: [u8; 4] = *b"SFSB";
/// D15（格式冻结政策） 已定项 4 登记表位 1：第一条纯 SSD 布局线（系统配置带回退
/// 下界 F、根记录带卸载记号、没有回退见证与回退行）。写路径恒写的布局身份位，读者要求它在。
/// 位图小端、位 0 是第一个字节的最低位（D22 已定项 13）；位 0 空出、没有布局用它（`.claude/kb/feature-bits.md`）。
const INCOMPAT_LAYOUT_IDENTITY_BIT: u8 = 0x02;
/// incompat 位 2（见 `research/prompts/m4-trunc-r2-main-verification.md` 第六节）：
/// inode 记录带截断字段（偏移 120 起）。mkfs 起就置上；读者要求它在，不带位 2 的池一律拒。
const INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT: u8 = 0x04;
/// mkfs 与之后每一次写系统配置写进 incompat 位图第一个字节的值：位 1 加位 2（0x06）。
const WRITTEN_FIRST_INCOMPAT_BYTE: u8 = INCOMPAT_LAYOUT_IDENTITY_BIT | INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT;
/// 三张位图各 32 字节（256 位）紧跟 magic 4 + 版本 2，incompat 在前、compat_ro 居中、compat 在后（D15 已定项 1）。
const SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET: usize = 4 + 2;
const FEATURE_BITMAP_BYTES: usize = 32;
/// 读者认识的全部 incompat 位；多出任何一位就是「不认识不许挂」（fs-design 格式层判据）。第一个字节只收 0x06（位 1 加位 2）。
/// 没列在这里的位（含空出的位 0）读者一律按「不认识的位」拒（`.claude/kb/feature-bits.md`）。
const SUPPORTED_INCOMPAT_BITS: u8 = INCOMPAT_LAYOUT_IDENTITY_BIT | INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT;
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
/// D16 已定项 8（暖机取戊）：mkfs 之后第一次可写挂载先连推空发布，直到本实例写成的根覆盖两块盘；
/// 第一版几何区域 1 / 2 分住两块盘 ⇒ 两次（txg 1、2），新池新建文件从 txg 3 起。
const WARM_UP_EMPTY_PUBLISHES: u64 = 2;
const NEW_POOL_FILE_CREATION_TXG: u64 = 3;

/// 并行度取 `SINGLEFS_THREAD_CAP`（`scripts/hosts/capped.sh` 给「登记不到专属变量的实验装置」的通用兜底，
/// 见该脚本文件头注释；这份装置在 `scripts/experiment-apparatus/` 下，不是 `crates/singlefs-verification`，不在它专属
/// 变量表里）。没设或解析不出正整数就取登记的重活默认并行度（`.claude/heavy-thread-default`）。
fn parallel_thread_count() -> usize {
    default_heavy_threads(include_str!("../../../.claude/heavy-thread-default"))
}

include!("../heavy_threads_default/threads.rs");

// ───────────────────────── E142 第十七次跑：产物自检（问题单第 8 行，R18/R19/R20） ─────────────────────────
//
// 自检只读这一次真正打印出来的文本（下面的线程局部缓冲区），不读任何 Rust 变量本身——这样一条把某个
// 打印点的角色名或数字改回字面的变异（M156/M157），只要它真的改了打印出来的字节，自检就看得见；
// 反过来，只改了某个中间变量、打印点自己另有一套读法的改动，自检也看不见（这正是自检该有的边界：
// 它核的是产物，不是代码）。

thread_local! {
    static EMITTED_LINES: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn main() {
    // 准入（`.claude/gate.d/stage-inputs.tsv` 的 E142 一行，判法在 `scripts/gate/admission.py`）：
    // 输入自上次产物以来没变就不重跑（退 77）；放行时输入指纹打在产物最前面。
    for admission_header_line in e7_index_bench::admit_experiment_run_or_exit("E142") {
        println!("{admission_header_line}");
    }
    let mut emitter = Emitter::new();
    // 跑前登记第一节第 1 条要求「同 3000 字节内容」（F1 诊断报告 research/prompts/e142-r11-f1-diagnosis.md）：
    // 这一格与 `crates/singlefs-verification/src/scenario.rs::first_file_content()` 用同一个公式 `index % 251`，
    // crates 一侧被 QEMU 读回核对、三份用例与层 0 两条流钉着，装置这一侧对齐它。
    let file_bytes: Vec<u8> = (0..3000u32).map(|index| u8::try_from(index % 251).expect("小于 256")).collect();
    emit(&mut emitter, &format!(
        "name=config devices=2 physical_block_bytes={PHYSICAL_BLOCK_BYTES} file_bytes={} fsid=fixed device_bytes={DEVICE_BYTES} journal_ring_bytes={JOURNAL_RING_BYTES} journal_ring_slots={JOURNAL_RING_SLOTS} in_flight_limit={JOURNAL_IN_FLIGHT_RECORD_LIMIT} unit_area_start_slot={UNIT_AREA_START_SLOT} unit_area_slots={UNIT_AREA_SLOTS} cluster_segment_slots={CLUSTER_SEGMENT_SLOTS} open_cluster_segment_start={OPEN_CLUSTER_SEGMENT_START_SLOT}",
        file_bytes.len()
    ));

    // 判据 1：宽度对账。
    // 「预期」那一列逐格抄自 `.claude/kb/layout/01-first-txn.md` 零到七，
    // 「实测」是装置自己的常量算出来的。两列不等就记一笔 `width_mismatches`。
    let width_rows = width_rows();
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
    // PC2（第五节 5.2）与第二段的层 0 主臂枚举都要「mkfs 之后、事务的写全没持久那一态」当基线——
    // 在 acquire_instance / warm_up / publish_first_file 继续往 `recording.pool` 里真写字节之前先拷一份。
    let base_after_mkfs = recording.pool.clone();
    let instance = acquire_instance(&mut recording, &parameters);
    let acquisition_operation_count = recording.operations.len();
    let (warm_up_roots, last_warm_up_record) = warm_up(&mut recording, &parameters, &genesis, instance);
    let warm_up_operation_count = recording.operations.len();
    let output = publish_first_file(&mut recording, &parameters, &genesis, &file_bytes, instance, last_warm_up_record.as_deref(), NEW_POOL_FILE_CREATION_TXG);
    let warm_up_operations = &recording.operations[acquisition_operation_count..warm_up_operation_count];
    emit(&mut emitter, &format!(
        "name=warm_up publishes={} writes={} barriers={} fua={} new_pool_file_creation_txg={NEW_POOL_FILE_CREATION_TXG} last_warm_up_root_txg={}",
        warm_up_roots.len(),
        warm_up_operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(_))).count(),
        warm_up_operations.iter().filter(|operation| matches!(operation, RecordedOperation::Barrier(_))).count(),
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
            RecordedOperation::Barrier(_) => None,
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
    // 写清单只数新池新建文件；层 0 枚举吃 mkfs 之后的全部操作（取号 + 暖机两次空发布 + 新池新建文件）
    let transaction_operations = &recording.operations[warm_up_operation_count..];
    let post_mkfs_operations = &recording.operations[mkfs_operation_count..];
    let acquisition_operations = &recording.operations[mkfs_operation_count..acquisition_operation_count];
    // 段序列登记表（layout/01-first-txn 八）的输入：每条路径单独切段、再加整条流。mkfs 那一行不进层 0 枚举（G19），但段序列钉在这里。
    for (path_name, operations) in [("mkfs", &recording.operations[..mkfs_operation_count]), ("instance_acquisition", acquisition_operations), ("warm_up", warm_up_operations), ("transaction", transaction_operations), ("post_mkfs_stream", post_mkfs_operations)] {
        let (_, path_segments) = split_into_segments(operations, true);
        let sizes: Vec<String> = path_segments.iter().map(|segment| segment.len().to_string()).collect();
        emit(&mut emitter, &format!("name=segments path={path_name} operations={} segments={} closed_form={} kinds={}", operations.len(), sizes.join("+"), closed_form_state_count(&path_segments), format_segment_kinds(&segment_step_kinds(operations, true))));
    }
    // 第八节几何敏感性 G1B（E142 第二十五次跑臂 N25M 的 M③）：一块盘带屏障时 mkfs 那一段，「每块盘 5 步清零」在盘数另一侧的取样点。
    emit(&mut emitter, &segments_geometry_line_of_mkfs_on_one_device_with_barriers());
    // E142 第十九次跑臂 N19 的 ③：五条路径各一行三态（`name=segments ` 那五行的字段集合与次序不动，⑦）。
    for (path_name, earlier_operations, operations) in new_pool_file_creation_stream_paths(&recording.operations, mkfs_operation_count, acquisition_operation_count, warm_up_operation_count) {
        let summary = three_state_summary(earlier_operations, operations);
        emit(&mut emitter, &format!(
            "name=segments_three_state path={path_name} writes={} in_place_overwrites={} in_place_per_segment={} in_place_kinds={} closed_form_two_state={} closed_form_three_state={}",
            summary.writes, summary.in_place_overwrites, summary.in_place_per_segment, summary.in_place_kinds, summary.closed_form_two_state, summary.closed_form_three_state
        ));
    }
    // ⑥ G10a / G10b：主几何（⑧ 开）的录制流上，新池新建文件「最后一次 journal 记录写 → 根槽 FUA 写」之间吞掉一块盘那一步屏障。
    for (point, swallowed_device) in SWALLOWED_BARRIER_POINTS_WITH_TRAILING_BARRIER {
        let swallowed_transaction = swallow_one_device_barrier_step_before_the_root(transaction_operations, swallowed_device);
        let swallowed_post_mkfs: Vec<RecordedOperation> = recording.operations[mkfs_operation_count..warm_up_operation_count].iter().cloned().chain(swallowed_transaction.iter().cloned()).collect();
        let point_fields = format!("point={point} swallowed_device={}", swallowed_device.0);
        emit(&mut emitter, &segments_sensitivity_line(&point_fields, "transaction", &recording.operations[..warm_up_operation_count], &swallowed_transaction));
        emit(&mut emitter, &segments_sensitivity_line(&point_fields, "post_mkfs_stream", &recording.operations[..mkfs_operation_count], &swallowed_post_mkfs));
    }
    // ⑩ G11（⑧ 关，就是臂 N19，登记 R44）五条路径，与 G12a / G12b（G11 的流上照 G10 各吞一步屏障）。
    let stream_without_trailing_barrier = record_new_pool_file_creation_stream(&PoolParameters::settled_two_devices_without_trailing_publish_barrier(), &file_bytes);
    for (path_name, earlier_operations, operations) in new_pool_file_creation_stream_paths(&stream_without_trailing_barrier.operations, stream_without_trailing_barrier.mkfs_operation_count, stream_without_trailing_barrier.acquisition_operation_count, stream_without_trailing_barrier.warm_up_operation_count) {
        emit(&mut emitter, &segments_sensitivity_line("point=G11 trailing_barrier=off", path_name, earlier_operations, operations));
    }
    for (point, swallowed_device) in SWALLOWED_BARRIER_POINTS_WITHOUT_TRAILING_BARRIER {
        let swallowed_transaction = swallow_one_device_barrier_step_before_the_root(&stream_without_trailing_barrier.operations[stream_without_trailing_barrier.warm_up_operation_count..], swallowed_device);
        let swallowed_post_mkfs: Vec<RecordedOperation> = stream_without_trailing_barrier.operations[stream_without_trailing_barrier.mkfs_operation_count..stream_without_trailing_barrier.warm_up_operation_count].iter().cloned().chain(swallowed_transaction.iter().cloned()).collect();
        let point_fields = format!("point={point} trailing_barrier=off swallowed_device={}", swallowed_device.0);
        emit(&mut emitter, &segments_sensitivity_line(&point_fields, "transaction", &stream_without_trailing_barrier.operations[..stream_without_trailing_barrier.warm_up_operation_count], &swallowed_transaction));
        emit(&mut emitter, &segments_sensitivity_line(&point_fields, "post_mkfs_stream", &stream_without_trailing_barrier.operations[..stream_without_trailing_barrier.mkfs_operation_count], &swallowed_post_mkfs));
    }
    let write_count = transaction_operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(_))).count();
    let barrier_count = transaction_operations.iter().filter(|operation| matches!(operation, RecordedOperation::Barrier(_))).count();
    let fua_count = transaction_operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(write) if write.is_fua())).count();
    let instance_table = parse_packed_unit(&genesis.instance_table_unit).expect("mkfs 写出的实例表单元自检要过");
    let tree_table_genesis = parse_index_node(&genesis.tree_table_genesis_unit).expect("mkfs 写出的树表单元自检要过");
    emit(&mut emitter, &format!("name=mkfs_units instance_table_records={} instance_table_row_bytes={} tree_table_entries={} genesis_root_watermark={}", instance_table.records.len(), instance_table.record_width, tree_table_genesis.entries.len(), genesis.root.tree_identifier_watermark));
    emit(&mut emitter, &format!("name=root_record checkpoint_txg={} instance={} tree_identifier_watermark={} rollback_floor={} unmount_marker={} record_bytes={} back_chain={}", output.root.checkpoint_txg.0, output.root.instance.0, output.root.tree_identifier_watermark, output.root.rollback_floor.0, output.root.unmount_marker, output.record_bytes.len(), output.record.back_chain));
    let slots: Vec<String> = output.units_by_slot.iter().map(|(slot, unit_identity, unit)| format!("{}@{}x{}", unit_identity.tag(), slot.0, unit.len())).collect();
    emit(&mut emitter, &format!("name=write_list writes={write_count} barriers={barrier_count} fua={fua_count} named={} units={}", output.record.named.len(), slots.join(",")));
    // E142 第十七次跑（问题单第 8 行，R18）：这一条流的角色名定义表——单元的 t 号从写清单现数（`output.units_by_slot`），
    // journal 记录、根槽、系统配置槽顺延排在单元之后，不写字面。这是角色名自检（Q142.27）唯一的定义来源。
    let main_unit_count = output.units_by_slot.len();
    let journal_label = format!("t{}", main_unit_count + 1);
    let root_label = format!("t{}", main_unit_count + 2);
    let system_configuration_label = format!("t{}", main_unit_count + 3);
    emit(&mut emitter, &format!(
        "name=role_labels scope=main_transaction units={} journal_record={journal_label} root={root_label} system_configuration={system_configuration_label} warm_up_journal_records=w1,w4",
        output.units_by_slot.iter().map(|(_, unit, _)| unit.tag()).collect::<Vec<_>>().join(",")
    ));
    for (tree, width) in &output.index_node_header_widths {
        emit(&mut emitter, &format!("name=index_node_header tree={tree} header_bytes={width} with_reserved={}", width + usize_from_u64(NONCE_MAC_RESERVED_BYTES)));
    }

    // 判据 3：读回。
    let full = recording.pool.clone();
    let full_report = recover(&full, JournalPolicy::Consult, ReaderMode::Clause);
    let content_matches = matches!(&full_report.outcome, RecoveryOutcome::FileRead { content, .. } if *content == file_bytes);
    emit(&mut emitter, &format!(
        "name=recover_full outcome={} root={} content_matches={content_matches} valid_records={} above_water={} applied={} verification_passed={} mapping_fallbacks={}",
        outcome_kind(&full_report.outcome), outcome_root(&full_report.outcome), full_report.journal.valid_records, full_report.journal.above_water, full_report.journal.prefix_applied, full_report.journal.verification_passed, full_report.mapping_fallbacks
    ));

    // ═════════ E142 第十四次跑（重跑登记 research/prompts/e142-r14-prereg.md）：第十四次跑那一版的记录头与末条标志（今天的头见 `name=journal_header_*` 各行） ═════════
    // Q142.1 逐写摘要：从 mkfs 起到新池新建文件写完为止每一次写（这一点为止 `recording` 只录了
    // mkfs + 取号 + 暖机 + 新池新建文件，后面别的管线不再写进这个 `recording`）。与臂 O 那份产物（草稿目录
    // `/tmp/claude-1000/e142-r14/arm-o/`，源码 sha256 ddeb68c5bb5512f85cef201657c67021725d021fdc6e58bdbb357b331576be32，
    // 即 git HEAD 上这份文件改动之前的样子）用同一段代码、按 P4 的脚本对齐比。
    for (index, operation) in recording.operations.iter().enumerate() {
        if let RecordedOperation::Write(write) = operation {
            emit(&mut emitter, &format!(
                "name=journal_header_write_digest index={index} kind={} device={} offset={} length={} sha256={}",
                write.kind.tag(), write.device.0, write.offset.0, write.bytes.len(), sha256_hex(&write.bytes)
            ));
        }
    }
    emit(&mut emitter, &format!(
        "name=journal_header_write_digest_summary writes={}",
        recording.operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(_))).count()
    ));

    // 臂 H（对照）：写者开关扳到 0x00，其余不变；与臂 F（本运行默认 parameters，flag=1）用同一份几何。
    let flag_cleared_parameters = parameters.clone().with_last_of_publish_flag(0x00);
    let (flag_cleared_image, _flag_cleared_output) = run_full_pipeline(&flag_cleared_parameters, NEW_POOL_FILE_CREATION_TXG, &file_bytes);
    let flag_set_image_for_the_flag_difference = full.clone();

    // Q142.2 / P2：H、F 逐字节相减，每一段按落在哪条记录（w1/w4/本次事务自己那条）、记录内哪个字段打标签。
    let hf_differences = diff_pools(&flag_cleared_image, &flag_set_image_for_the_flag_difference);
    let mut hf_fields_by_record_device: BTreeMap<(String, u32), Vec<String>> = BTreeMap::new();
    for segment in &hf_differences {
        let (record_name, offset_in_record) = journal_record_name_for_offset(segment.offset, &journal_label).unwrap_or(("unregistered".to_string(), segment.offset));
        let field = journal_header_field_tag(offset_in_record);
        emit(&mut emitter, &format!(
            "name=journal_header_flag_diff device={} offset={} length={} old={} new={} record={record_name} field={field} offset_in_record={offset_in_record}",
            segment.device, segment.offset, segment.length, hex_bytes(&segment.old), hex_bytes(&segment.new)
        ));
        hf_fields_by_record_device.entry((record_name.clone(), segment.device)).or_default().push(field.to_string());
    }
    for record_name in ["w1".to_string(), "w4".to_string(), journal_label.clone()] {
        for device_index in 0..u32::try_from(parameters.device_count).expect("设备数") {
            let fields = hf_fields_by_record_device.get(&(record_name.clone(), device_index)).cloned().unwrap_or_default();
            emit(&mut emitter, &format!("name=journal_header_flag_diff_fields record={record_name} device={device_index} fields={}", if fields.is_empty() { "none".to_string() } else { fields.join(",") }));
        }
    }
    emit(&mut emitter, &format!("name=journal_header_flag_diff_summary segments={}", hf_differences.len()));

    // P3：全盘相减分得出一个字节——在各自镜像的副本上把设备 0 的本次事务那条记录偏移 87（本次发布内序号首字节）、
    // 偏移 7（记录标志）各翻一位，与未翻的同一臂镜像相减；O 那一列由 `name=positive_control_change_count` 两行管（第五节 P3）。
    let this_transaction_record_base = journal_record_offset(JournalCounter(3)).0;
    for (arm_name, image) in [("H", &flag_cleared_image), ("F", &flag_set_image_for_the_flag_difference)] {
        for (field_offset, field_tag) in [(87u64, "本次发布内序号"), (JOURNAL_RECORD_FLAGS_OFFSET_U64, "记录标志")] {
            let mut flipped = image.clone();
            flip_byte(&mut flipped, DeviceIdentity(0), DeviceOffset(this_transaction_record_base), field_offset);
            let segments = diff_pools(image, &flipped);
            let matches_expected = segments.len() == 1 && segments[0].device == 0 && segments[0].length == 1 && segments[0].offset == this_transaction_record_base + field_offset;
            emit(&mut emitter, &format!(
                "name=journal_header_p3 arm={arm_name} field={journal_label}_{field_tag} segments={} device={} offset={} length={} matches_expected={matches_expected}",
                segments.len(), segments.first().map_or(0, |segment| segment.device), segments.first().map_or(0, |segment| segment.offset), segments.first().map_or(0, |segment| segment.length)
            ));
        }
    }

    // Q142.6 附带：H、F 两臂 w4、本次事务那条记录的反向链值（不当答案，只与第四节的线索并列给主 agent 看）。
    for (arm_name, image) in [("H", &flag_cleared_image), ("F", &flag_set_image_for_the_flag_difference)] {
        let records = scan_journal(image, unit_fsid(&parameters.fsid));
        for (label, counter) in journal_header_record_labels(&journal_label) {
            if let Some(record) = records.get(&(InstanceGeneration(1), JournalCounter(counter))) {
                emit(&mut emitter, &format!(
                    "name=journal_header_record_bytes arm={arm_name} record={label} device=0 flag_byte={:#04x} ordinal_bytes={} back_chain={}",
                    record.record_flags, record.ordinal_within_publish, record.back_chain
                ));
            }
        }
    }

    // P6：读者读了标志与序号——四对 (H/F × 甲/乙) 都跑，另加 F 的六份改写（第五节 5.2）。
    #[allow(clippy::type_complexity, reason = "这个元组类型只在这一处出现，起类型别名只多一个只用一次的名字")]
    let journal_without_root_rows: [(&str, Option<fn(&mut JournalRecord)>, bool); 8] = [
        ("F", None, false),
        ("H", None, false),
        ("F_flag_0x03", Some(journal_without_root_mutate_flag_0x03), false),
        ("F_ordinal_0", Some(journal_without_root_mutate_ordinal_0), false),
        ("F_flag_0x00", Some(journal_without_root_mutate_flag_0x00), false),
        ("F_ordinal_2", Some(journal_without_root_mutate_ordinal_2), false),
        ("F_trailing_same_txg", None, true),
        ("F_txg_2", Some(journal_without_root_mutate_txg_2), false),
    ];
    for (label, mutate, append_trailing) in journal_without_root_rows {
        let row_parameters = if label == "H" { flag_cleared_parameters.clone() } else { parameters.clone() };
        let row = run_journal_without_root_scenario(&row_parameters, &file_bytes, mutate, append_trailing);
        emit(&mut emitter, &format!(
            "name=journal_header_p6 mirror={label} primary_prefix_applied={} primary_readable={} clause_prefix_applied={} clause_readable={} clause_fatal={}",
            row.primary_prefix_applied, row.primary_readable, row.clause_prefix_applied, row.clause_readable, row.clause_fatal
        ));
    }

    // 第八节 8.2 G1（方向相反：本实例的第一条就是这条流自己唯一的 journal 记录）：装置原判据 5 那条路，
    // 按臂 H、臂 F 各跑一次、相减。这条流没有暖机，唯一的 journal 记录落在计数器 1（"w1" 分支），
    // 不会走到 `journal_record_name_for_offset` 的第三分支——第二个参数按这条流自己的单元数算，图的是
    // 万一以后这条流也带了暖机，标签仍然对，不留一个永远用不上、字面写死的角色名。
    let unwarmed_single_device_flag_cleared_parameters = PoolParameters::control_one_device_no_barriers().with_last_of_publish_flag(0x00);
    let unwarmed_single_device_flag_set_parameters = PoolParameters::control_one_device_no_barriers().with_last_of_publish_flag(0x01);
    let (mut unwarmed_single_device_flag_cleared_recording, unwarmed_single_device_flag_cleared_genesis) = mkfs(&unwarmed_single_device_flag_cleared_parameters);
    let _ = publish_first_file(&mut unwarmed_single_device_flag_cleared_recording, &unwarmed_single_device_flag_cleared_parameters, &unwarmed_single_device_flag_cleared_genesis, &file_bytes, InstanceGeneration(FIRST_INSTANCE_GENERATION), None, NEW_POOL_FILE_CREATION_TXG);
    let (mut unwarmed_single_device_flag_set_recording, unwarmed_single_device_flag_set_genesis) = mkfs(&unwarmed_single_device_flag_set_parameters);
    let unwarmed_single_device_flag_set_output = publish_first_file(&mut unwarmed_single_device_flag_set_recording, &unwarmed_single_device_flag_set_parameters, &unwarmed_single_device_flag_set_genesis, &file_bytes, InstanceGeneration(FIRST_INSTANCE_GENERATION), None, NEW_POOL_FILE_CREATION_TXG);
    // 前缀用 "c"（control，不进 `[mawt]` 那个字母表）：这条流是另一套几何（一盘、没有暖机），
    // 它的单元数与主臂不同，用同一个 "t" 前缀会跟主臂 `name=role_labels` 的 t1..t12 撞号
    // （这个几何 10 个单元 ⇒ 若用 "t11" 会跟主臂 t11=MappingTree 撞在角色名自检里，见第十二节修订）。
    let unwarmed_single_device_journal_label = format!("c{}", unwarmed_single_device_flag_set_output.units_by_slot.len() + 1);
    let unwarmed_single_device_differences = diff_pools(&unwarmed_single_device_flag_cleared_recording.pool, &unwarmed_single_device_flag_set_recording.pool);
    let mut unwarmed_single_device_fields: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for segment in &unwarmed_single_device_differences {
        let (record_name, offset_in_record) = journal_record_name_for_offset(segment.offset, &unwarmed_single_device_journal_label).unwrap_or(("unregistered".to_string(), segment.offset));
        unwarmed_single_device_fields.insert(format!("{record_name}:{}", journal_header_field_tag(offset_in_record)));
    }
    let unwarmed_single_device_has_back_chain = unwarmed_single_device_fields.iter().any(|field| field.ends_with(":back_chain"));
    emit(&mut emitter, &format!(
        "name=journal_header_geometry sample=G1 record={unwarmed_single_device_journal_label} fields={} back_chain_present={unwarmed_single_device_has_back_chain}",
        if unwarmed_single_device_fields.is_empty() { "none".to_string() } else { unwarmed_single_device_fields.iter().cloned().collect::<Vec<_>>().join(",") }
    ));

    // G2（盘数 2 → 1，带暖机）：run_full_pipeline 在一块盘的参数上，臂 H、臂 F 各跑一次、相减。
    let warmed_single_device_flag_cleared_parameters = PoolParameters::control_one_device_no_barriers().with_last_of_publish_flag(0x00);
    let warmed_single_device_flag_set_parameters = PoolParameters::control_one_device_no_barriers().with_last_of_publish_flag(0x01);
    let (warmed_single_device_flag_cleared_image, _) = run_full_pipeline(&warmed_single_device_flag_cleared_parameters, NEW_POOL_FILE_CREATION_TXG, &file_bytes);
    let (warmed_single_device_flag_set_image, warmed_single_device_flag_set_output) = run_full_pipeline(&warmed_single_device_flag_set_parameters, NEW_POOL_FILE_CREATION_TXG, &file_bytes);
    let warmed_single_device_journal_label = format!("c{}", warmed_single_device_flag_set_output.units_by_slot.len() + 1);
    let warmed_single_device_differences = diff_pools(&warmed_single_device_flag_cleared_image, &warmed_single_device_flag_set_image);
    let mut warmed_single_device_fields_by_record: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for segment in &warmed_single_device_differences {
        let (record_name, offset_in_record) = journal_record_name_for_offset(segment.offset, &warmed_single_device_journal_label).unwrap_or(("unregistered".to_string(), segment.offset));
        warmed_single_device_fields_by_record.entry(record_name).or_default().push(journal_header_field_tag(offset_in_record).to_string());
    }
    for record_name in ["w1".to_string(), "w4".to_string(), warmed_single_device_journal_label] {
        let fields = warmed_single_device_fields_by_record.get(&record_name).cloned().unwrap_or_default();
        emit(&mut emitter, &format!("name=journal_header_geometry sample=G2 record={record_name} fields={}", if fields.is_empty() { "none".to_string() } else { fields.join(",") }));
    }

    // ═════════ E142 第十一次跑（重跑登记 research/prompts/e142-r11-prereg.md）：改动计数从 1 改成 3，盘上还有哪些字节跟着变 ═════════
    // 真实基线 M_B（臂 B，旧口径写 1）：同一次进程里再跑一遍整条路，parameters / file_bytes 与 M_A 一字不差。
    let (image_change_count_old, output_change_count_old) = run_full_pipeline(&parameters, 1, &file_bytes);
    let image_change_count_new = full.clone();
    let (root_region, root_ring_slot) = ring_target_for_publish(CheckpointTxg(NEW_POOL_FILE_CREATION_TXG));
    let root_device = parameters.region_devices[usize_from_u64(root_region)].0;
    let root_offset = ring_slot_offset(root_region, root_ring_slot).0;
    let catalog = build_structure_catalog(&output, root_device, root_offset);
    // 量 1：M_A 与 M_B 在两块盘全部字节上逐字节相减（分母是全部字节——第一节第 3 条；见 diff_pools 的推理注释）。
    let differences = diff_pools(&image_change_count_old, &image_change_count_new);
    emit(&mut emitter, &format!("name=change_count_run_pair devices={} file_bytes={} old_change_count=1 new_change_count={NEW_POOL_FILE_CREATION_TXG} segments={}", parameters.device_count, file_bytes.len(), differences.len()));
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
            // `index_value=`/`offset_value=` 让文本数字自检（R19/Q142.28）能在同一行现取来比：
            // `text=` 里嵌的 index、offset 两个数不是字面，是这一行本来就有的两个字段，绑定关系现成。
            emit(&mut emitter, &format!("name=gap id=CC{index} text=diff段{index}未归因_structure={structure}_offset={offset_in_structure}_detail={reason} index_value={index} offset_value={offset_in_structure}"));
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
    let record_field_offset: usize = usize_from_u64(PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES) + 88; // 224：记录区 136 起 + 记录内偏移 88
    for (image, label, expected_change_count) in [(&image_change_count_new, "M_A", NEW_POOL_FILE_CREATION_TXG), (&image_change_count_old, "M_B", 1u64)] {
        for device_index in 0..parameters.device_count {
            let device = DeviceIdentity(u32::try_from(device_index).expect("设备数"));
            let sector = image.read(device, leaf_slot_base, usize_from_u64(PHYSICAL_BLOCK_BYTES));
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
    run_comparator_byte_positive_control(&mut emitter, "arm_n16", &model_writes);

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
        let reference_arm_text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("Q142.19：读不到臂 N15 参照 {path}：{error}"));
        let reference_arm_writes = parse_write_lines(&reference_arm_text, "name=device_region_bytes ");
        let mut reference_arm_matched = vec![false; reference_arm_writes.len()];
        let mut matched_count = 0u64;
        let mut changed_count = 0u64;
        for write in &model_writes {
            let length = write.bytes.len() as u64;
            let Some(index) = find_matching_impl_write(&reference_arm_writes, write.device, write.offset, length) else {
                emit(&mut emitter, &format!("name=old_new_region_only_in role=N16 region={} device={} offset={} length={length}", write.region, write.device, write.offset));
                continue;
            };
            reference_arm_matched[index] = true;
            matched_count += 1;
            let sha = sha256_hex(&write.bytes);
            let changed = reference_arm_writes[index].sha256 != sha;
            if changed {
                changed_count += 1;
            }
            emit(&mut emitter, &format!("name=old_new_region region={} device={} changed={changed}", write.region, write.device));
        }
        for (index, matched) in reference_arm_matched.iter().enumerate() {
            if !matched {
                let line = &reference_arm_writes[index];
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
            let device_slots = slots_of_device_bytes(device_bytes);
            let device_slots_vector = vec![device_slots; usize_from_u64(device_count)];
            let root_level = allocation_record_tree_root_level(&device_slots_vector);
            let cells_per_device = allocation_record_tree_cells_per_device(device_slots, root_level);
            let nodes = allocation_record_tree_new_pool_file_creation_node_count(device_count, root_level);
            emit(&mut emitter, &format!(
                "name=g3_shape point={label} device_count={device_count} device_bytes={device_bytes} slots={device_slots} root_level={root_level} root_cells_per_device={cells_per_device} new_pool_file_creation_allocation_nodes={nodes}"
            ));
        }
    }

    // 第八节 G4：一盘，走整条写路，与 crates 的一盘导出比（第三个命令行参数，可选）。
    if let Some(path) = std::env::args().nth(3) {
        let one_device_dump_text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("G4：读不到一盘导出 {path}：{error}"));
        let one_device_dump_implementation_writes = parse_write_lines(&one_device_dump_text, "name=device_region_bytes ");
        let one_device_dump_parameters = PoolParameters::control_one_device_no_barriers();
        let one_device_dump_model_writes = model_window_writes_for_parameters(&one_device_dump_parameters, &file_bytes);
        let mut one_device_dump_matched = vec![false; one_device_dump_implementation_writes.len()];
        let mut one_device_dump_equal = 0u64;
        let mut one_device_dump_unequal = 0u64;
        for write in &one_device_dump_model_writes {
            let sha = sha256_hex(&write.bytes);
            let length = write.bytes.len() as u64;
            let Some(index) = find_matching_impl_write(&one_device_dump_implementation_writes, write.device, write.offset, length) else {
                emit(&mut emitter, &format!("name=g4_bytes_unmatched side=model region={} device={} offset={} length={length}", write.region, write.device, write.offset));
                continue;
            };
            one_device_dump_matched[index] = true;
            let (equal, first_diff_offset, mismatch_bytes) = compare_paired_write(&write.bytes, &sha, &one_device_dump_implementation_writes[index]);
            if equal {
                one_device_dump_equal += 1;
            } else {
                one_device_dump_unequal += 1;
            }
            emit(&mut emitter, &format!(
                "name=g4_bytes_equal region={} device={} offset={} length={length} equal={equal} first_diff_offset={} mismatch_bytes={}",
                write.region, write.device, write.offset,
                first_diff_offset.map_or("none".to_string(), |value| value.to_string()),
                mismatch_bytes.map_or("unknown".to_string(), |value| value.to_string())
            ));
        }
        for (index, matched) in one_device_dump_matched.iter().enumerate() {
            if !matched {
                let line = &one_device_dump_implementation_writes[index];
                emit(&mut emitter, &format!("name=g4_bytes_unmatched side=crates device={} offset={} length={}", line.device, line.offset, line.length));
            }
        }
        emit(&mut emitter, &format!(
            "name=g4_bytes_equal_summary model_regions={} crates_regions={} equal={one_device_dump_equal} unequal={one_device_dump_unequal}",
            one_device_dump_model_writes.len(), one_device_dump_implementation_writes.len()
        ));
    } else {
        emit(&mut emitter, "name=g4_bytes_equal_summary skipped=true reason=no_one_device_dump_path_given");
    }

    // 量 8：几何敏感性——取样点一（盘数 2→1）与取样点二（换一个值 1→4）。
    let one_device_parameters = PoolParameters::control_one_device_no_barriers();
    let (one_device_image_new, one_device_output_new) = run_full_pipeline(&one_device_parameters, NEW_POOL_FILE_CREATION_TXG, &file_bytes);
    let (one_device_image_old, _) = run_full_pipeline(&one_device_parameters, 1, &file_bytes);
    let (one_device_root_region, one_device_root_slot) = ring_target_for_publish(CheckpointTxg(NEW_POOL_FILE_CREATION_TXG));
    let one_device_root_device = one_device_parameters.region_devices[usize_from_u64(one_device_root_region)].0;
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

    // 判据 4：层 0 主臂。基线是 mkfs 之后的池（事务的写全没持久那一态）；整轮枚举 67108885 个状态（第七节 B8）。
    let (writes, segments) = split_into_segments(post_mkfs_operations, true);
    let segment_sizes: Vec<String> = segments.iter().map(|segment| segment.len().to_string()).collect();
    let closed_form = closed_form_state_count(&segments);
    let (_, segments_fua_free) = split_into_segments(post_mkfs_operations, false);
    emit(&mut emitter, &format!("name=layer0_fua_not_boundary segments={} closed_form={}", segments_fua_free.iter().map(|segment| segment.len().to_string()).collect::<Vec<_>>().join("+"), closed_form_state_count(&segments_fua_free)));

    let layer0_main_start = std::time::Instant::now();
    let layer0_main_threads = parallel_thread_count();
    let main_root_index = writes.iter().rposition(|write| write.kind == StepKind::RootRecordFua).expect("主臂写流里有根槽那一条");
    // 暖机两次空发布不写单元（D16（发布语义） 已定项 8「且不写单元」），整条主臂窗口里唯一会出现
    // `JournalRecord` 三次（w1、w4、t13 各一次发布），主事务自己那一次镜像最后 `device_count` 份就是它。
    let all_journal_indices: Vec<usize> = writes.iter().enumerate().filter(|(_, write)| write.kind == StepKind::JournalRecord).map(|(index, _)| index).collect();
    let main_journal_indices = all_journal_indices[all_journal_indices.len() - parameters.device_count..].to_vec();
    let main_dependency_groups = main_transaction_dependency_groups(&writes, parameters.device_count);
    let main_role_names = generic_role_names(&writes);
    let (main_tally, main_comparison, main_grouping) = enumerate_layer0_grouped(
        layer0_main_threads, &base_after_mkfs, &writes, &segments, &file_bytes, ReaderMode::Clause,
        main_root_index, &main_journal_indices, &main_dependency_groups, &main_role_names,
    );
    let publish_labels = ["txg1", "txg2", "txg3", "after_last_root", "every_write_persisted"];
    emit(&mut emitter, &format!(
        "name=layer0 arm=settled_two_devices segments={} states={} closed_form={closed_form} exhaustive={} states_by_publish={}",
        segment_sizes.join("+"), main_tally.states, main_tally.states == closed_form,
        format_publish_counts(&main_grouping, &publish_labels, |bucket| bucket.states)
    ));
    emit(&mut emitter, &format!(
        "name=layer0_violations violations={} first_violation={}",
        main_tally.violations, main_tally.first_violation.clone().unwrap_or_else(|| "none".to_string())
    ));
    emit(&mut emitter, &format!("name=layer0_violations_by_publish {}", format_publish_fields(&main_grouping, &publish_labels, |bucket| bucket.violations)));
    emit(&mut emitter, &format!(
        "name=journal_effect arm=settled_two_devices states={} differing_states={} differing_by_publish={}",
        main_tally.states, main_tally.journal_differing_states,
        format_publish_counts(&main_grouping, &["txg1", "txg2", "txg3"], |bucket| bucket.journal_differing_states)
    ));
    for (&(predicted, actual), count) in &main_comparison.outcome_matrix {
        if *count > 0 {
            emit(&mut emitter, &format!("name=layer0_outcome_matrix arm=settled_two_devices predicted={} actual={} count={count}", predicted.tag(), actual.tag()));
        }
    }
    emit(&mut emitter, &format!("name=layer0_outcome_matrix_summary off_diagonal={}", main_comparison.off_diagonal));
    // S5（第八节）：读者乙 ①–④ 各分支在整轮层 0 状态里走到几次；合法历史里恒 0。
    let reader_branches = main_tally.reader_branches;
    emit(&mut emitter, &format!(
        "name=journal_header_reader_branch_counts arm=settled_two_devices corrupted_flag={} corrupted_ordinal_zero={} multiple_last_flag={} ordinal_gap={} ordinal_not_one_at_publish_start={} trailing_after_last_flag={} total={}",
        reader_branches.corrupted_flag, reader_branches.corrupted_ordinal_zero, reader_branches.multiple_last_flag, reader_branches.ordinal_gap,
        reader_branches.ordinal_not_one_at_publish_start, reader_branches.trailing_after_last_flag, reader_branches.total()
    ));
    // Q142.37：判决行，每个字段只对应第六节 6.2 一个格的门槛，不合成一个（同 Q142.30 的做法）。
    // 判决字段本身从 `layer0_main_arm_verdict` 现算——这个函数被单测直接调用（不用真的跑一遍主臂
    // 枚举），吸取第十二节修订第 8 条的教训：内嵌在 `main()` 里的逻辑在 `cargo test` 下永远走不到。
    let main_arm_verdict = layer0_main_arm_verdict(&main_tally, closed_form, &main_comparison);
    emit(&mut emitter, &format!(
        "name=verdict_layer0 layer0_states_ok={} layer0_violations={} root_persisted_ok={} file_read_ok={} file_read_wrong_content_zero={} journal_differing_states={} journal_reading={} main_outcome_matrix_ok={}",
        main_arm_verdict.layer0_states_ok, main_arm_verdict.layer0_violations, main_arm_verdict.root_persisted_ok, main_arm_verdict.file_read_ok,
        main_arm_verdict.file_read_wrong_content_zero, main_tally.journal_differing_states, main_arm_verdict.journal_reading, main_arm_verdict.main_outcome_matrix_ok
    ));
    // 计时不在转发输出的循环里打：这里只在枚举真正跑完之后取一次挂钟差（`.claude/kb/vm-harness.md`）。
    emit(&mut emitter, &format!("name=layer0_main_arm_elapsed seconds={:.3} states={} threads={layer0_main_threads}", layer0_main_start.elapsed().as_secs_f64(), main_tally.states));
    emit(&mut emitter, "name=done_layer0_main");

    // 判据 5：阳性对照——一块盘、不放屏障，oracle 必须分得出「根在而单元不在」。
    // E142 第十七次跑（问题单第 6 行）：期望违例数不再写死 `− 8`，改由这一窗口的写清单现算三档（R10）；
    // 逐态比、结果类矩阵（Q142.23/24）与 G6、G7（Q142.25，第八节）、PC2（Q142.26，第五节 5.2）都在这里跑。
    let control_flow = build_single_publish_flow(&PoolParameters::control_one_device_no_barriers(), &file_bytes, false);
    let (control_expected_jia, control_expected_yi, control_expected_bing) = control_expected_violations_three_readings(&control_flow);
    let (control_tally, control_comparison) = compare_layer0_predictions(
        &control_flow.base, &control_flow.writes, &control_flow.segments, &file_bytes, ReaderMode::Clause,
        control_flow.journal_index, &control_flow.dependency_groups, &control_flow.role_names,
    );
    emit(&mut emitter, &format!(
        "name=layer0_control arm=one_device_no_barriers writes={} unit_writes={} states={} closed_form={} violations={} expected_violations={control_expected_jia} expected_basis=dependency_set_jia expected_violations_jia={control_expected_jia} expected_violations_yi={control_expected_yi} expected_violations_bing={control_expected_bing} root_persisted_states={} file_read={} no_file={} failed={} journal_differing={}",
        control_flow.writes.len(), control_flow.dependency_groups.len(),
        control_tally.states, control_flow.closed_form, control_tally.violations, control_tally.root_persisted_states, control_tally.file_read_states, control_tally.no_file_states, control_tally.failed_states, control_tally.journal_differing_states
    ));
    emit(&mut emitter, &format!(
        "name=layer0_control_state_comparison arm=one_device_no_barriers states={} predicted_violations={} agree={} missed={} false_alarm={} first_missed={} first_false_alarm={}",
        control_comparison.states, control_comparison.predicted_violations, control_comparison.agree, control_comparison.missed, control_comparison.false_alarm,
        control_comparison.first_missed.as_deref().unwrap_or("none"), control_comparison.first_false_alarm.as_deref().unwrap_or("none")
    ));
    for ((predicted, actual), count) in &control_comparison.outcome_matrix {
        emit(&mut emitter, &format!("name=layer0_control_outcome_matrix arm=one_device_no_barriers predicted={} actual={} count={count}", predicted.tag(), actual.tag()));
    }
    emit(&mut emitter, &format!("name=layer0_control_outcome_matrix_summary off_diagonal={}", control_comparison.off_diagonal));

    // 第八节 G6：同一个一盘几何，把 D16（发布语义） 已定项 7 的两道屏障放回去（FUA 不切段）——违例该从正数翻成 0。
    let barriers_restored_flow = build_single_publish_flow(&PoolParameters::one_device_with_barriers(), &file_bytes, false);
    let (barriers_restored_tally, barriers_restored_comparison) = compare_layer0_predictions(
        &barriers_restored_flow.base, &barriers_restored_flow.writes, &barriers_restored_flow.segments, &file_bytes, ReaderMode::Clause, barriers_restored_flow.journal_index, &barriers_restored_flow.dependency_groups, &barriers_restored_flow.role_names,
    );
    emit(&mut emitter, &format!(
        "name=layer0_sensitivity point=G6 barriers=settled fua_is_boundary=false segments={} states={} closed_form={} violations={} missed={} false_alarm={} file_read={} journal_differing={}",
        barriers_restored_flow.segments.iter().map(|segment| segment.len().to_string()).collect::<Vec<_>>().join("+"),
        barriers_restored_tally.states, barriers_restored_flow.closed_form, barriers_restored_tally.violations, barriers_restored_comparison.missed, barriers_restored_comparison.false_alarm, barriers_restored_tally.file_read_states, barriers_restored_tally.journal_differing_states
    ));

    // 第八节 G7：阳性对照的同一条写流，只把 FUA 当段边界（D13（验证路线） 已定项 4 字面）——状态数与违例都该翻。
    let control_flow_fua = build_single_publish_flow(&PoolParameters::control_one_device_no_barriers(), &file_bytes, true);
    let (fua_boundary_tally, fua_boundary_comparison) = compare_layer0_predictions(
        &control_flow_fua.base, &control_flow_fua.writes, &control_flow_fua.segments, &file_bytes, ReaderMode::Clause,
        control_flow_fua.journal_index, &control_flow_fua.dependency_groups, &control_flow_fua.role_names,
    );
    emit(&mut emitter, &format!(
        "name=layer0_sensitivity point=G7 barriers=none fua_is_boundary=true segments={} states={} closed_form={} violations={} missed={} false_alarm={} file_read={} journal_differing={}",
        control_flow_fua.segments.iter().map(|segment| segment.len().to_string()).collect::<Vec<_>>().join("+"),
        fua_boundary_tally.states, control_flow_fua.closed_form, fua_boundary_tally.violations, fua_boundary_comparison.missed, fua_boundary_comparison.false_alarm, fua_boundary_tally.file_read_states, fua_boundary_tally.journal_differing_states
    ));

    // PC2（第五节 5.2）：主臂几何上，全部持久、与逐个单元的两份都不持久（12 个单元、13 个状态）。
    // `base_after_mkfs` + `writes`（mkfs 之后到新池新建文件写完为止全部写）逐一持久/不持久，与 `enumerate_layer0`
    // 用的是同一种 `CrashImage` 构造法（`base` 是 mkfs 之后的池，不是 `full` 那样已经全部写完的池）。
    let pc2 = run_pc2(&base_after_mkfs, &writes, &output.units_by_slot, &file_bytes);
    for (label, tag, caught, outcome) in &pc2.per_unit {
        emit(&mut emitter, &format!("name=positive_control_main_geometry_unit label={label} unit={tag} caught={caught} outcome={outcome}"));
    }
    emit(&mut emitter, &format!(
        "name=positive_control_main_geometry states={} all_persisted_legal={} all_persisted_content_ok={} units={} caught={}",
        pc2.per_unit.len() + 1, pc2.all_persisted_legal, pc2.all_persisted_content_ok, pc2.per_unit.len(), pc2.caught
    ));

    // 判据 7：坏字节探针。
    for probe in probes(&parameters) {
        let report = run_probe(&full, &probe);
        let reason = match &report.outcome {
            RecoveryOutcome::Failed { reason, .. } => reason.replace(' ', "_"),
            RecoveryOutcome::NoFile { .. } | RecoveryOutcome::FileRead { .. } => "none".to_string(),
        };
        let report_content_matches = matches!(&report.outcome, RecoveryOutcome::FileRead { content, .. } if *content == file_bytes);
        emit(&mut emitter, &format!(
            "name=probe probe={} outcome={} root={} content_matches={report_content_matches} valid_records={} applied={} mapping_fallbacks={} reason={reason}",
            probe.name, outcome_kind(&report.outcome), outcome_root(&report.outcome), report.journal.valid_records, report.journal.prefix_applied, report.mapping_fallbacks
        ));
    }

    // 算术：码 1 映射 key 在同一事务写两个数据单元时相同；D16 的事务切分纪律下 1 MiB 写要几条记录。
    let first_key = mapping_key_for_data(output.data_pointer.head, output.data_pointer.write_order);
    let second_unit_same_transaction = mapping_key_for_data(output.data_pointer.head, output.data_pointer.write_order);
    let distinct_keys = if first_key == second_unit_same_transaction { 1 } else { 2 };
    emit(&mut emitter, &format!("name=mapping_key_collision same_transaction_data_units=2 distinct_keys={distinct_keys} mapping_entries_new_pool_file_creation={}", output.mapping_keys.len()));
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
        usize_from_u64(ALLOCATION_RECORD_BYTES) - ALLOCATION_KEY_BYTES,
        output.allocation_records.iter().filter(|record| record.generation == CheckpointTxg(0)).count()
    ));
    emit(&mut emitter, &format!(
        "name=instances mkfs_instance={MKFS_INSTANCE_GENERATION} first_writable_mount_instance={FIRST_INSTANCE_GENERATION} mkfs_system_configuration_generation={SYSTEM_CONFIGURATION_GENERATION_AT_MKFS} acquisition_system_configuration_generation={SYSTEM_CONFIGURATION_GENERATION_AT_INSTANCE_ACQUISITION} transaction_system_configuration_generation={} transaction_system_configuration_slot={}",
        system_configuration_write_for_publish(CheckpointTxg(NEW_POOL_FILE_CREATION_TXG)).0,
        system_configuration_write_for_publish(CheckpointTxg(NEW_POOL_FILE_CREATION_TXG)).1
    ));
    // 反向链（D23 已定项 19 ②）：环上计数器为 1 的那条恒 0，其余罩前一条的整个头。
    let records_on_disk = scan_journal(&full, unit_fsid(&FIXED_FSID));
    let back_chain_by_counter: Vec<String> = records_on_disk.values().map(|record| format!("{}:{}", record.counter.0, record.back_chain)).collect();
    emit(&mut emitter, &format!("name=back_chain records={} chains={}", records_on_disk.len(), back_chain_by_counter.join(",")));

    // 判据 8：空白清单。G22 的文本改成从这份产物自己的记账现算（问题单第 8 行、R19）：占位符替换成
    // `statistic_value` 现取的同一批数，另加三个绑定字段，让文本数字自检（Q142.28）在同一行找得到相等的兄弟字段。
    let free_bytes_per_device = statistic_value(STATISTIC_FREE_BYTES);
    let empty_cluster_segments_per_device = statistic_value(STATISTIC_EMPTY_CLUSTER_SEGMENTS);
    let fragmentation_runs_per_device = statistic_value(STATISTIC_FRAGMENTATION_RUNS);
    for (gap_identifier, text) in GAPS {
        if *gap_identifier == "G22" {
            let text = render_free_bytes_gap_text(text, free_bytes_per_device, empty_cluster_segments_per_device, fragmentation_runs_per_device);
            emit(&mut emitter, &format!(
                "name=gap id={gap_identifier} text={text} bound_free_bytes_per_device={free_bytes_per_device} bound_empty_cluster_segments_per_device={empty_cluster_segments_per_device} bound_fragmentation_runs_per_device={fragmentation_runs_per_device}"
            ));
        } else {
            emit(&mut emitter, &format!("name=gap id={gap_identifier} text={text}"));
        }
    }
    emit(&mut emitter, &format!("name=gaps count={}", GAPS.len()));

    // 判据 9（D15 已定项 4）：feature bits 的实际字节与「不认识不许挂」。
    let system_configuration_slot = recording.pool.read(DeviceIdentity(0), DeviceOffset(SYSTEM_CONFIGURATION_SLOT_OFFSETS[0]), usize_from_u64(SYSTEM_CONFIGURATION_SLOT_BYTES));
    let feature_bits = &system_configuration_slot[SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET..SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET + 3 * FEATURE_BITMAP_BYTES];
    let mut unknown_incompat_slot = system_configuration_slot.clone();
    // A28：0x0e = 布局身份位（0x02）、截断字段位（0x04）加没登记的位 3（0x08）——两个要求的位都在，但带着一个不认识的位，同样拒。
    unknown_incompat_slot[SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET] = WRITTEN_FIRST_INCOMPAT_BYTE | 0x08;
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

    // ═════════ E142 第二十一次跑（重跑登记 research/prompts/e142-r21-prereg.md 第七节 7.1 甲表，R56）：字段表 174 行 ═════════
    // 期望值是登记甲表的字面量，实测值从这条流写出的字节里按字面偏移解出来（见 `layout_field_rows`）；
    // 放在产物自检之前，好让角色名与文本数字自检也扫到这几行。
    let layout_rows = layout_field_rows(&full, &base_after_mkfs);
    for row in &layout_rows {
        let equal = layout_field_equal(&row.expected, &row.actual);
        emit(&mut emitter, &format!(
            "name=layout_field id={} record_or_unit={} field={} expected={} actual={} equal={equal} layout_line={}",
            row.row_identifier, row.record_or_unit, row.field, row.expected, row.actual, row.layout_line
        ));
    }
    let (layout_equal_rows, layout_unequal_rows) = layout_field_tally(&layout_rows);
    emit(&mut emitter, &format!("name=layout_field_summary rows={} equal={layout_equal_rows} unequal={layout_unequal_rows}", layout_rows.len()));
    let (layout_injected, layout_detected, layout_exact) = layout_field_positive_control(&layout_rows);
    emit(&mut emitter, &format!("name=layout_field_positive_control injected={layout_injected} detected={layout_detected} exact={layout_exact}"));

    // 判据 8 续（问题单第 8 行）：产物自检——角色名（R18/R20，Q142.27）、文本里的数（R19，Q142.28）、
    // 问题单点名的两处（R20，Q142.29）。只读这一次真正打印出来的行（`EMITTED_LINES`），不读 Rust 变量。
    let emitted_so_far: Vec<String> = EMITTED_LINES.with(|lines| lines.borrow().clone());
    let role_label_check = check_role_labels(&emitted_so_far);
    emit(&mut emitter, &format!(
        "name=self_check_role_labels defined={} used={} undefined={} contradictions={}",
        role_label_check.defined, role_label_check.used, role_label_check.undefined, role_label_check.contradictions
    ));
    for row in &role_label_check.contradiction_rows {
        emit(&mut emitter, &format!("name=self_check_role_label_contradiction {row}"));
    }
    for row in &role_label_check.undefined_rows {
        emit(&mut emitter, &format!("name=self_check_role_label_undefined {row}"));
    }

    let text_number_check = check_text_numbers(&emitted_so_far);
    emit(&mut emitter, &format!(
        "name=self_check_text_numbers text_fields={} numbers={} bound={} bound_mismatched={} literal_allowlisted={} unclassified={}",
        text_number_check.text_fields, text_number_check.numbers, text_number_check.bound, text_number_check.bound_mismatched, text_number_check.literal_allowlisted, text_number_check.unclassified
    ));
    for row in &text_number_check.mismatch_rows {
        emit(&mut emitter, &format!("name=self_check_text_number_mismatch {row}"));
    }
    for row in &text_number_check.unclassified_rows {
        emit(&mut emitter, &format!("name=self_check_text_number_unclassified {row}"));
    }

    // Q142.29：问题单点名的两处，各自从产物文本里现取来比（不读 Rust 变量），不依赖上面两个通用扫描器。
    let journal_header_record_label = find_field_value_excluding(&emitted_so_far, "journal_header_record_bytes", "record", &["w1", "w4"]).unwrap_or_else(|| "not_found".to_string());
    let write_list_journal_record_label = find_field_value_excluding(&emitted_so_far, "role_labels", "journal_record", &[]).unwrap_or_else(|| "not_found".to_string());
    let labels_equal = journal_header_record_label == write_list_journal_record_label;
    let free_bytes_gap_text = emitted_so_far
        .iter()
        .find(|line| {
            let fields = split_fields(line);
            fields.iter().any(|(field_key, field_value)| *field_key == "name" && *field_value == "gap") && fields.iter().any(|(field_key, field_value)| *field_key == "id" && *field_value == "G22")
        })
        .and_then(|line| split_fields(line).iter().find(|(field_key, _)| *field_key == "text").map(|(_, field_value)| (*field_value).to_string()))
        .unwrap_or_default();
    let free_bytes_in_the_gap_text = extract_number_after_marker(&free_bytes_gap_text, "空闲字节").unwrap_or(0);
    let accounting_free_bytes_per_device: u64 = find_field_value_excluding(&emitted_so_far, "accounting", "free_bytes_per_device", &[]).and_then(|value| value.parse().ok()).unwrap_or(0);
    let numbers_equal = free_bytes_in_the_gap_text == accounting_free_bytes_per_device;
    emit(&mut emitter, &format!(
        "name=self_check_named_instances journal_header_record_label={journal_header_record_label} write_list_journal_record_label={write_list_journal_record_label} labels_equal={labels_equal} g22_free_bytes={free_bytes_in_the_gap_text} accounting_free_bytes_per_device={accounting_free_bytes_per_device} numbers_equal={numbers_equal}"
    ));

    // R18 第五节 5.1 ④、第七节 A24–A27：模型自己核窗口两份系统配置槽与根槽的条款字段，
    // 喂 Q142.45 的 r18_system_configuration_fields_ok、r18_root_flags_ok
    // （输出字段名是登记写死的协议名，Rust 变量名按命名纪律另起，不必与它相同）。
    let (window_system_configuration_generation, window_system_configuration_slot_index) = system_configuration_write_for_publish(CheckpointTxg(NEW_POOL_FILE_CREATION_TXG));
    let window_system_configuration_slot_offset = SYSTEM_CONFIGURATION_SLOT_OFFSETS[window_system_configuration_slot_index];
    let mut window_system_configuration_clause_fields_ok = true;
    for device in parameters.devices() {
        let slot_bytes = recording.pool.read(device, DeviceOffset(window_system_configuration_slot_offset), usize_from_u64(SYSTEM_CONFIGURATION_SLOT_BYTES));
        let incompat_byte0_ok = slot_bytes[SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET] == WRITTEN_FIRST_INCOMPAT_BYTE;
        let feature_bits_rest_zero = slot_bytes[SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET + 1..SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET + 3 * FEATURE_BITMAP_BYTES]
            .iter()
            .all(|byte| *byte == 0);
        let rollback_floor_hex = hex_bytes(&slot_bytes[SYSTEM_CONFIGURATION_ROLLBACK_FLOOR_OFFSET..SYSTEM_CONFIGURATION_ROLLBACK_FLOOR_OFFSET + 8]);
        let padding_after_489_zero = slot_bytes[usize_from_u64(SYSTEM_CONFIGURATION_BYTES)..].iter().all(|byte| *byte == 0);
        let checksum_recomputes = wide_checksum_with_field_zeroed(&slot_bytes, usize_from_u64(SYSTEM_CONFIGURATION_SLOT_BYTES), SYSTEM_CONFIGURATION_CHECKSUM_OFFSET)
            == slot_bytes[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32];
        let field_ok = incompat_byte0_ok && feature_bits_rest_zero && rollback_floor_hex == "0000000000000000" && padding_after_489_zero && checksum_recomputes;
        window_system_configuration_clause_fields_ok &= field_ok;
        emit(&mut emitter, &format!(
            "name=r18_clause_fields region=system_configuration device={} incompat_byte0={:#04x} feature_bits_rest_zero={feature_bits_rest_zero} rollback_floor_hex={rollback_floor_hex} padding_after_489_zero={padding_after_489_zero} checksum_recomputes={checksum_recomputes} ok={field_ok}",
            device.0, slot_bytes[SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET],
        ));
    }
    let (window_root_region, window_root_slot) = ring_target_for_publish(CheckpointTxg(NEW_POOL_FILE_CREATION_TXG));
    let window_root_bytes = recording.pool.read(parameters.region_devices[usize_from_u64(window_root_region)], ring_slot_offset(window_root_region, window_root_slot), usize_from_u64(PHYSICAL_BLOCK_BYTES));
    let root_flags_hex = hex_bytes(&window_root_bytes[20..24]);
    let root_checksum_recomputes = wide_checksum_with_field_zeroed(&window_root_bytes, usize_from_u64(PHYSICAL_BLOCK_BYTES), ROOT_CHECKSUM_OFFSET)
        == window_root_bytes[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32];
    let window_root_record_clause_fields_ok = root_flags_hex == "00000000" && root_checksum_recomputes;
    emit(&mut emitter, &format!(
        "name=r18_clause_fields region=root_record device={} flags_hex={root_flags_hex} checksum_recomputes={root_checksum_recomputes} ok={window_root_record_clause_fields_ok}",
        parameters.region_devices[usize_from_u64(window_root_region)].0,
    ));

    // 第八节 G8（只在模型里）：把窗口里设备 0 的系统配置只换 F，与 F = 0 的那一份逐字节比、解析回来。
    let rollback_floor_sensitivity_baseline = SystemConfiguration {
        fsid: parameters.fsid,
        this_device: DeviceIdentity(0),
        device_count: u32::try_from(parameters.device_count).expect("设备数"),
        slot_generation: window_system_configuration_generation,
        region_devices: parameters.region_devices,
        journal_tail: NEW_POOL_FILE_CREATION_TXG,
        journal_instance: instance,
        rollback_floor: CheckpointTxg(0),
    };
    let rollback_floor_sensitivity_modified = SystemConfiguration {
        fsid: parameters.fsid,
        this_device: DeviceIdentity(0),
        device_count: u32::try_from(parameters.device_count).expect("设备数"),
        slot_generation: window_system_configuration_generation,
        region_devices: parameters.region_devices,
        journal_tail: NEW_POOL_FILE_CREATION_TXG,
        journal_instance: instance,
        rollback_floor: CheckpointTxg(0x0807_0605_0403_0201),
    };
    let rollback_floor_sensitivity_baseline_bytes = rollback_floor_sensitivity_baseline.to_slot();
    let rollback_floor_sensitivity_modified_bytes = rollback_floor_sensitivity_modified.to_slot();
    let rollback_floor_sensitivity_differing_offsets: Vec<usize> = (0..rollback_floor_sensitivity_baseline_bytes.len())
        .filter(|&index| rollback_floor_sensitivity_baseline_bytes[index] != rollback_floor_sensitivity_modified_bytes[index])
        .collect();
    let rollback_floor_sensitivity_expected_offsets: Vec<usize> = (155..159).chain(481..489).collect();
    let rollback_floor_sensitivity_bytes_hex = hex_bytes(&rollback_floor_sensitivity_modified_bytes[SYSTEM_CONFIGURATION_ROLLBACK_FLOOR_OFFSET..SYSTEM_CONFIGURATION_ROLLBACK_FLOOR_OFFSET + 8]);
    let rollback_floor_sensitivity_parse_round_trip = SystemConfiguration::parse_slot(&rollback_floor_sensitivity_modified_bytes)
        .map(|parsed| parsed.rollback_floor == rollback_floor_sensitivity_modified.rollback_floor)
        .unwrap_or(false);
    let rollback_floor_sensitivity_offsets_text: Vec<String> = rollback_floor_sensitivity_differing_offsets.iter().map(ToString::to_string).collect();
    emit(&mut emitter, &format!(
        "name=r18_sensitivity point=G8 field=rollback_floor value=0x0807060504030201 bytes_hex={rollback_floor_sensitivity_bytes_hex} differing_offsets={} parse_round_trip={rollback_floor_sensitivity_parse_round_trip}",
        rollback_floor_sensitivity_offsets_text.join(","),
    ));
    let rollback_floor_sensitivity_ok = rollback_floor_sensitivity_bytes_hex == "0102030405060708"
        && rollback_floor_sensitivity_differing_offsets == rollback_floor_sensitivity_expected_offsets
        && rollback_floor_sensitivity_parse_round_trip;

    // 第八节 G9（只在模型里）：把窗口里新池新建文件的根只把卸载记号置 1，与不置那一份逐字节比、解析回来；
    // 另把 flags 改成除位 0 之外的一位（0x02），重算自证校验和之后必须被拒。
    let unmount_marker_sensitivity_baseline = output.root;
    let mut unmount_marker_sensitivity_marked = unmount_marker_sensitivity_baseline;
    unmount_marker_sensitivity_marked.unmount_marker = true;
    let unmount_marker_sensitivity_baseline_bytes = unmount_marker_sensitivity_baseline.to_slot();
    let unmount_marker_sensitivity_marked_bytes = unmount_marker_sensitivity_marked.to_slot();
    let unmount_marker_sensitivity_differing_offsets: Vec<usize> = (0..unmount_marker_sensitivity_baseline_bytes.len())
        .filter(|&index| unmount_marker_sensitivity_baseline_bytes[index] != unmount_marker_sensitivity_marked_bytes[index])
        .collect();
    let unmount_marker_sensitivity_flags_hex = hex_bytes(&unmount_marker_sensitivity_marked_bytes[20..24]);
    let unmount_marker_sensitivity_parse_marker = RootRecord::parse_slot(&unmount_marker_sensitivity_marked_bytes, &parameters.fsid).map(|parsed| parsed.unmount_marker).unwrap_or(false);
    let mut unmount_marker_sensitivity_other_bit_bytes = unmount_marker_sensitivity_marked_bytes.clone();
    unmount_marker_sensitivity_other_bit_bytes[20..24].copy_from_slice(&0x0000_0002u32.to_le_bytes());
    let recomputed_root_checksum = wide_checksum_with_field_zeroed(&unmount_marker_sensitivity_other_bit_bytes, usize_from_u64(PHYSICAL_BLOCK_BYTES), ROOT_CHECKSUM_OFFSET);
    unmount_marker_sensitivity_other_bit_bytes[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&recomputed_root_checksum);
    let unmount_marker_sensitivity_other_bit_rejected = RootRecord::parse_slot(&unmount_marker_sensitivity_other_bit_bytes, &parameters.fsid).is_none();
    let unmount_marker_sensitivity_offsets_text: Vec<String> = unmount_marker_sensitivity_differing_offsets.iter().map(ToString::to_string).collect();
    emit(&mut emitter, &format!(
        "name=r18_sensitivity point=G9 field=unmount_marker flags_hex={unmount_marker_sensitivity_flags_hex} differing_offsets={} parse_marker={unmount_marker_sensitivity_parse_marker} other_bit_rejected={unmount_marker_sensitivity_other_bit_rejected}",
        unmount_marker_sensitivity_offsets_text.join(","),
    ));
    let unmount_marker_sensitivity_expected_offsets: Vec<usize> = std::iter::once(20).chain(138..142).collect();
    let unmount_marker_sensitivity_ok = unmount_marker_sensitivity_flags_hex == "01000000"
        && unmount_marker_sensitivity_differing_offsets.iter().all(|offset| unmount_marker_sensitivity_expected_offsets.contains(offset))
        && unmount_marker_sensitivity_differing_offsets.contains(&20)
        && unmount_marker_sensitivity_parse_marker
        && unmount_marker_sensitivity_other_bit_rejected;

    // 层 0 主臂那几格（判据 4）取上面整轮枚举的判决；write_count / control 的门槛按第七节
    // B5（29 写）、B10（8192 个状态）的新锚点算，不再钉旧布局的 21 / 2048（其余字段照报，不设门槛）。
    emit(&mut emitter, &format!(
        "name=verdict width_mismatches={width_mismatches} write_list_ok={} recover_full_ok={content_matches} layer0_states_ok={} layer0_violations={} control_states_ok={} control_violations_ok={} control_missed_zero={} control_false_alarm_zero={} control_outcome_matrix_ok={} g6_states_ok={} g6_violations_zero={} g7_states_ok={} g7_violations_zero={} positive_control_main_geometry_ok={} role_label_contradictions_zero={} role_labels_all_defined={} text_number_mismatches_zero={} text_numbers_all_classified={} named_instances_ok={} journal_differing_states={} r18_system_configuration_fields_ok={window_system_configuration_clause_fields_ok} r18_root_flags_ok={window_root_record_clause_fields_ok} r18_g8_ok={rollback_floor_sensitivity_ok} r18_g9_ok={unmount_marker_sensitivity_ok}",
        new_pool_file_creation_write_list_matches(write_count, barrier_count, fua_count, parameters.device_count),
        main_arm_verdict.layer0_states_ok,
        main_arm_verdict.layer0_violations,
        control_tally.states == control_flow.closed_form && control_flow.closed_form == 8192,
        control_tally.violations == control_expected_jia,
        control_comparison.missed == 0,
        control_comparison.false_alarm == 0,
        control_comparison.off_diagonal == 0,
        // 第七节 A17/A18 的字面（1028、2050）：`states = closed_form` 只核内部自洽，这两格另核跟登记的
        // 预言数对不对上——G7 对不上就是停机 S10（见第十二节修订：这个装置的 `split_into_segments` 与
        // `crates/singlefs-verification/src/segments.rs` 都把 FUA 写并进它前面没被屏障断开的那一段，
        // 不是登记 A18 假设的「FUA 恒自成一段」，两边一致、不作废）。
        barriers_restored_tally.states == barriers_restored_flow.closed_form && barriers_restored_flow.closed_form == 1028,
        barriers_restored_tally.violations == 0,
        fua_boundary_tally.states == control_flow_fua.closed_form && control_flow_fua.closed_form == 2050,
        fua_boundary_tally.violations == 0,
        pc2.caught == 12 && pc2.all_persisted_legal && pc2.all_persisted_content_ok,
        role_label_check.contradictions == 0,
        role_label_check.undefined == 0,
        text_number_check.bound_mismatched == 0,
        text_number_check.unclassified == 0,
        labels_equal && numbers_equal,
        main_tally.journal_differing_states,
    ));

    // ═════════ E142 第二十二次跑第二段（登记 5.4 ⑥，问题单第 2 行）：第二个几何 P8G，两块 8 GiB 盘 ═════════
    // 以下各行全部以 `name=p8g_` 起头，放在主点的 `name=verdict` 之后：把它们删掉，其余与第一段产物逐行相同（Q142.101）。
    let (p8g_rows, _) = p8g_section_rows(&file_bytes);
    for row in &p8g_rows {
        emit(&mut emitter, row);
    }
    let printed_main_verdict_line = emitted_so_far_verdict_line();
    let (control_equal, control_unequal) = compare_stream_verdict_fields_to_printed_verdict(&printed_main_verdict_line, &stream_section(&parameters, &file_bytes).verdict);
    emit(&mut emitter, &format!(
        "name=p8g_control_main_point fields={} equal={control_equal} unequal={} unequal_fields={}",
        control_equal + control_unequal.len(),
        control_unequal.len(),
        if control_unequal.is_empty() { "none".to_string() } else { control_unequal.join(",") }
    ));
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests;
