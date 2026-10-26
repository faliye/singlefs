// GPU 判器：每个调用判一个崩溃状态，读 crash_judge_tables 摊出来的表（一张打包的 u32 表 + 目录），
// 按持久掩码给每个落点查版本，在版本字节上按 singlefs-core 的恢复路径解结构、择系统配置、择根、扫 journal 并重放、
// 走到第一个文件，两种 journal 政策各走一遍；再按 oracle 判七类违例、按记录核对器判两条判据。
// 结论：每个状态 VERDICT_WORDS 个 u32（布局见 crash_judge_gpu.rs 文件头；字号常量由 verdict_layout_wgsl_constants 生成、接在源码最前面）。判不了的状态置最高位并写原因码，交 CPU 判。

struct Dispatch {
    block_start_lo: u32,
    block_start_hi: u32,
    state_count: u32,
    scratch_words: u32,
};

@group(0) @binding(0) var<uniform> dispatch: Dispatch;
@group(0) @binding(1) var<storage, read> tables: array<u32>;
@group(0) @binding(2) var<storage, read_write> verdicts: array<u32>;
@group(0) @binding(3) var<storage, read_write> scratch: array<u32>;

const NONE: u32 = 0xFFFFFFFFu;
// 表数 TABLE_COUNT、表号 T_*（与 crash_judge_tables::JudgeTablesWords::every_table 同序）、段表宽 SEGMENT_WORDS 与段的展开方式码
// 由 crash_judge_gpu.rs 的 judge_table_layout_wgsl_constants 生成、接在源码最前面

const WRITE_WORDS: u32 = 12u;
const LOCATION_WORDS: u32 = 12u;
// 版本表每条的字数 VERSION_WORDS、组合掩码的字数 COMBINATION_WORDS_OF_A_VERSION、组合高位那几字的起点
// VERSION_WORD_COMBINATION_PAST_THE_SIXTY_FOURTH_WRITE 与持久掩码的字数 PERSISTED_MASK_WORDS 由 crash_judge_gpu.rs 的
// judge_table_layout_wgsl_constants 从 Rust 常量生成、接在源码最前面
// 版本表第 8 字（crash_judge_tables::VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES）：单元槽的一版，「前 16 KiB 后面接 16 KiB 全零」那 32 KiB 的
// CRC-32C 异或 16 KiB 全零的 CRC-32C。它再异或一段 16 KiB 的 CRC-32C，就是「这一版前 16 KiB 接那一段」32 KiB 的 CRC-32C（CRC 对异或线性）
const VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES: u32 = 8u;
// 版本表第 1 条全零单元（16 KiB，crash_judge_tables::ZERO_UNIT_VERSIONS）：拼读前一半是空槽时读它
const ZERO_UNIT_VERSION_OF_SIXTEEN_KIBIBYTES: u32 = 1u;
// zero_unit_version_over_a_slot_nothing_wrote 交的码（不是版本下标）：起点是空段的最后一槽、读两槽，后一半要 read_at 拼
const ZERO_SIXTEEN_KIBIBYTES_FOLLOWED_BY_THE_NEXT_SLOT: u32 = 0xFFFFFFFEu;
// 切片第 2 个检查字（c2）的位 5（版本表的标志位只用到位 4）：这一读跨过了落点的末尾（read_at 拼读），c1 是拼出来的整段 CRC
const SLICE_SPLICED_ACROSS_THE_END_OF_A_LOCATION: u32 = 32u;
const LATER_WORDS: u32 = 4u;
const PUBLISH_WORDS: u32 = 8u;
const EXPECTED_WORDS: u32 = 6u;

// 落点种类码
const LOCATION_SYSTEM_CONFIGURATION: u32 = 0u;
const LOCATION_ROOT_SLOT: u32 = 1u;
const LOCATION_JOURNAL_RECORD: u32 = 2u;
const LOCATION_UNIT_SLOT: u32 = 3u;
// 写的种类码
const WRITE_UNIT: u32 = 1u;
const WRITE_JOURNAL: u32 = 2u;
const WRITE_ROOT: u32 = 3u;

// 结论位（verdicts 第 0 字）
const BIT_CONSULTED_ORACLE: u32 = 0u;   // 位 0..6：七类
const BIT_IGNORED_ORACLE: u32 = 8u;     // 位 8..14
const BIT_ROOT_WITHOUT_RECORD: u32 = 16u;
const BIT_CLAIMED_STATE_MISSING_UNIT: u32 = 17u;
const BIT_UNDECIDABLE: u32 = 31u;
// oracle 七类（与 CrashPointReplayOracleViolationKind::EVERY_KIND 同序）
const ORACLE_NONE: u32 = NONE;
const ORACLE_NO_ROOT_CHOSEN: u32 = 0u;
const ORACLE_RECOVERED_TO_AN_OLDER_ROOT: u32 = 1u;
const ORACLE_READ_FAILED: u32 = 2u;
const ORACLE_NO_FILE_WHERE_AN_OLDER_ROOT_HAS_ONE: u32 = 3u;
const ORACLE_NO_FILE_WHERE_THIS_ROOT_HAS_ONE: u32 = 4u;
const ORACLE_FILE_READ_WHERE_THIS_ROOT_HAS_NONE: u32 = 5u;
const ORACLE_WRONG_CONTENT: u32 = 6u;
// 走读结局 OUTCOME_*（crash_judge_gpu.rs 的 RECOVERY_OUTCOME_CODE_*）与失败点码 FAILURE_POINT_*（gpu_failure_point_code）由 crash_judge_gpu.rs 生成
// 判不了的原因
const UNDECIDABLE_NO_VERSION: u32 = 1u;
const UNDECIDABLE_TOO_MANY_RECORDS: u32 = 2u;
const UNDECIDABLE_TOO_MANY_ENTRIES: u32 = 3u;
const UNDECIDABLE_TOO_DEEP: u32 = 4u;
const UNDECIDABLE_TOO_MANY_DEVICES: u32 = 5u;
const UNDECIDABLE_TOO_MANY_ROOTS: u32 = 6u;
// read_at 拼读那一读的整段 CRC 对得上位置条目（slice_crc_matches）：往下要读的字节跨过落点末尾，表里没有
const UNDECIDABLE_SPLICED_READ_MATCHES: u32 = 7u;
// 序号落进了一段不展开的段（那种段的序号区间是空的）：表与派活的序号对不上
const UNDECIDABLE_STATE_IN_A_SEGMENT_NOT_EXPANDED: u32 = 8u;
// 分片的段（C790）：片起点的各位加片内序号之后还有进位（越过段的域），或片内序号装不进 32 位：表与派活的序号对不上
const UNDECIDABLE_SLICE_PAST_ITS_DOMAIN: u32 = 9u;
// 读数据单元找下一块数据指针（find_the_next_block_of_data_pointers）时，从下段根往下读回的节点不是 extent 树那一遍核过的那一形
// （读不出、解不开、含这个单元的叶不在那里）：两遍读同一条指针、同一份表，应当走不到；走到了是内核自己的错，不往下判
const UNDECIDABLE_DATA_POINTER_BLOCK_NOT_FOUND_AGAIN: u32 = 10u;
// 恢复内核判退看三样读者的并集（crash_judge_recovery.wgsl 的 selected_version_under_the_three_readers）：只读挂载的打开、建上一版或沿链读实例表那一样读者
// 读到一张越过内核容量的表、没读到它的第一个失败，别的读者又都没有拒收内容——判退不退判不出来
const UNDECIDABLE_A_READER_OF_THE_FALL_BACK_UNION: u32 = 11u;

// 格式常量
const SLOT_SHIFT: u32 = 14u;                 // 16384 = 1 << 14
const NODE_BYTES: u32 = 16384u;
const DATA_UNIT_BYTES: u32 = 32768u;
const ROOT_RECORD_BYTES: u32 = 466u;
const JOURNAL_RECORD_BYTES: u32 = 4096u;
const JOURNAL_HEADER_BYTES: u32 = 496u;
const JOURNAL_NAMED_ENTRY_BYTES: u32 = 56u;
// 记录头里新根段的起点与宽（D23（journal 的角色与格式） 已定项 4 / 已定项 15：树表指针 86、映射根指针 86、树 ID 水位 8、F 8、
// 实例表指针 86、分配记录树根指针 86、卸载记号 4、回收门槛 t 8、钉槽 p 1），fsid 8 紧跟新根段（偏移 472），头到 496 为止
const JOURNAL_NEW_ROOT_SEGMENT_OFFSET: u32 = 99u;
const JOURNAL_NEW_ROOT_SEGMENT_BYTES: u32 = 373u;
// 新根段里后三段的段内偏移（walk.rs 的 NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER、NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER、
// NEW_ROOT_SEGMENT_UNMOUNT_MARKER）：施加一条记录取这三段（已定项 15），卸载记号只有位 0（root_record.rs 的 ROOT_RECORD_FLAG_UNMOUNT_MARKER）
const JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER: u32 = 188u;
const JOURNAL_NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER: u32 = 274u;
const JOURNAL_NEW_ROOT_SEGMENT_UNMOUNT_MARKER: u32 = 360u;
const JOURNAL_FILESYSTEM_IDENTIFIER_OFFSET: u32 = 472u;
// 根记录里回收门槛 t 8 字节与钉槽 p 1 字节的偏移（root_record.rs 的写者、lib.rs 的 ROOT_RECLAIM_THRESHOLD_OFFSET；C773）与「没有钉着的槽」那个值
// （singlefs_format::ROOT_RECORD_NO_RING_SLOT_PINNED）：I-7.18、I-7.19 与 I-3.1 钉带出来的那一截读它们
const ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET: u32 = 428u;
const ROOT_RECORD_PINNED_RING_SLOT_OFFSET: u32 = 436u;
const ROOT_RECORD_NO_RING_SLOT_PINNED: u32 = 255u;
// 新根段里回收门槛 t 与钉槽 p 的段内偏移（walk.rs 的 NEW_ROOT_SEGMENT_RECLAIM_THRESHOLD）：I-8.10 拿这 9 字节与同代根的 428..437 比
const JOURNAL_NEW_ROOT_SEGMENT_RECLAIM_THRESHOLD: u32 = 364u;
const JOURNAL_RING_START_BYTES: u32 = 16777216u;   // 1024 × 16384
const SYSTEM_CONFIGURATION_SLOT_BYTES: u32 = 4096u;
const ROOT_RING_REGION_STRIDE: u32 = 3145728u;     // P × chunk = 3 × 1 MiB
// 第一版的根环区域步长 P 与 chunk（singlefs_format::ROOT_RING_PRIME_STEP、ROOT_RING_CHUNK_BYTES）：系统配置偏移 363、367 那两个 4 字节字段
// 读者只收它们（D22（单元原子性怎么合成） 已定项 16 第 1 句，I-7.13）；区域间距照旧按 ROOT_RING_REGION_STRIDE 放
const ROOT_RING_PRIME_STEP: u32 = 3u;
const ROOT_RING_CHUNK_BYTES: u32 = 1048576u;
const_assert ROOT_RING_PRIME_STEP * ROOT_RING_CHUNK_BYTES == ROOT_RING_REGION_STRIDE;
const ROOT_RING_BASE_BYTES: u32 = 1048576u;        // 64 × 16384
const DATA_UNIT_PAYLOAD_CAPACITY: u32 = 32634u;
// 数据单元头里偏移 101 那份载荷区 CRC-32C 罩的字节数：单元 32768 减头 105（预留位 29 + 载荷容量 32634；crash_judge_tables 的 expected_version_chunks 同一个范围）
const DATA_UNIT_PAYLOAD_REGION_BYTES: u32 = 32663u;
// CRC-32C（Castagnoli）的生成多项式，反射位序（位 31 是 x 的 0 次）
const CRC32C_REFLECTED_POLYNOMIAL: u32 = 0x82F63B78u;
const EXTENT_UPPER_LEAF_INODES: u32 = 143u;
// extent 上段叶条目宽（key 24 + 标签 1 + 指针 88，D8（核心索引结构） 已定项 11 字段表；extent_tree.rs 的 EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES）
const EXTENT_UPPER_LEAF_ENTRY_BYTES: u32 = 113u;
const EXTENT_LOWER_LEAF_DATA_UNITS: u32 = 144u;
const EXTENT_INTERNAL_FANOUT: u32 = 147u;
const ALLOCATION_LEAF_SLOTS: u32 = 812u;
const ALLOCATION_INTERNAL_FANOUT: u32 = 169u;
const TREE_KIND_EXTENT: u32 = 1u;
const TREE_KIND_INODE: u32 = 2u;
const TREE_KIND_ALLOCATION: u32 = 3u;
const TREE_KIND_ACCOUNTING: u32 = 4u;
const TREE_KIND_LIVELIST: u32 = 6u;
const TREE_KIND_SPARSE_SIDE_TABLE: u32 = 7u;
const TREE_KIND_DEADLIST: u32 = 8u;
// 第一个文件版本那次零根注册的四棵树（recovery.rs 与 walk.rs 的 TREE_KIND_DELETED_INODES..TREE_KIND_LOGGED_OPERATIONS）
const TREE_KIND_DELETED_INODES: u32 = 9u;
const TREE_KIND_TOMBSTONE: u32 = 10u;
const TREE_KIND_LINEAGE: u32 = 11u;
const TREE_KIND_LOGGED_OPERATIONS: u32 = 12u;
// 退役树（recovery.rs 与 walk.rs 的 TREE_KIND_RETIREMENT，判决 m3-retire-r1 第三节第 1 条）：码 13，排在 logged_ops 之后
const TREE_KIND_RETIREMENT: u32 = 13u;
// 进树表的种类有几个（中央映射树 5 不进树表）：I-9.16 ② 按发号次序数这几个，见 tree_kind_in_the_issuing_order
const TREE_KINDS_OF_THE_TREE_TABLE: u32 = 12u;
// 只读快照的 inode 根与 extent 根（recovery.rs 与 walk.rs 的 TREE_KIND_SNAPSHOT_INODE_ROOT、TREE_KIND_SNAPSHOT_EXTENT_ROOT，判决
// `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1）：码 14、15，一个快照一对，一张树表里可以有多对，不进发号次序那十二种
const TREE_KIND_SNAPSHOT_INODE_ROOT: u32 = 14u;
const TREE_KIND_SNAPSHOT_EXTENT_ROOT: u32 = 15u;
// 树表条目 flags 2 字节的偏移（树 ID 8 + 条目长度 2 + 种类 2）与码 14 条目 flags 位 0「销毁中」（records.rs 的
// TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED，判决 S4）：码 14 上恰是这一位收下，别的位、别的种类上的这一位拒收（TreeTableEntry::parse_with_destruction_mark）
const TREE_TABLE_ENTRY_FLAGS_OFFSET: u32 = 12u;
const TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED: u32 = 1u;
// 树表条目里 previous_snapshot_txg、诞生 txg 8 字节的偏移（D8（核心索引结构） 已定项 8 的字段表：根指针 14..100 之后）
const TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET: u32 = 100u;
const TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET: u32 = 108u;
// 树表条目里头 ID 8 字节的偏移（诞生 txg 之后；walk.rs 与 recovery.rs 的 TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET）：恢复内核配快照条目、
// 池级 checker 两个内核判 I-9.27 与 I-9.16 ③ 都读它
const TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET: u32 = 116u;
// 第一版一张树表里至多几个只读快照（records.rs 的 SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION）：码 14 多于它，读树表的每一处都拒（UnitMalformed）
const SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION: u32 = 10u;
// 码 2 节点 key 的几种形态（field_width / field_count 与池级 checker 的 schema_field_width 共用的码）：0 记账 [2,8,4,8]、1 映射 [1,8,8,4,6]、
// 5 deadlist [8,8,1,8,8,4,6]（头的树 ID 8 + 纪元 8 + 中央映射 key 27，lib.rs 的 KEY_SCHEMA_DEADLIST；2、3、4 是池级 checker 那几种单字段与定宽形态）
const KEY_SCHEMA_DEADLIST: u32 = 5u;
// deadlist 树叶条目 43 字节里三段的偏移（walk.rs 的 DEADLIST_ENTRY_*_OFFSET；D5（快照 / 空间记账机制） 已定项 12）
const DEADLIST_ENTRY_BYTES: u32 = 43u;
const DEADLIST_ENTRY_HEAD_OFFSET: u32 = 0u;
const DEADLIST_ENTRY_EPOCH_OFFSET: u32 = 8u;
const DEADLIST_ENTRY_MAPPING_KEY_OFFSET: u32 = 16u;
// 6 logged_ops [2,8]（种类 2 + 签发 txg 8，lib.rs 的 KEY_SCHEMA_LOGGED_OPERATIONS；判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1）：
// 第四棵多层码 2 树，叶条目 44（key 10 + value 34，singlefs_format::LOGGED_OPERATIONS_ENTRY_BYTES）、内部条目 key + 子指针 86
const KEY_SCHEMA_LOGGED_OPERATIONS: u32 = 6u;
const LOGGED_OPERATIONS_KEY_BYTES: u32 = 10u;
const LOGGED_OPERATIONS_ENTRY_BYTES: u32 = 44u;
// deleted_inodes 根的 key 是 inode 号 8 字节；墓碑树、谱系树码 2 节点的 key 是容器号 8 字节（recovery.rs 的 key_width_for_kind）
const DELETED_INODES_KEY_WIDTH: u32 = 8u;
const CONTAINER_NUMBER_KEY_WIDTH: u32 = 8u;
// 墓碑树、谱系树码 2 节点的条目照 inode 树内部条目：分隔 key 8 + 身份引用 26 + 子指针 86
const CONTAINER_TREE_ENTRY_BYTES: u32 = 120u;
const CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET: u32 = 34u;
const FIRST_INODE_NUMBER: u32 = 1u;
// inode 记录里对象出生代（8 字节）、nlink（4 字节）、size（8 字节）的偏移（D8（核心索引结构） 已定项 6 字段表；恢复内核找第一个文件、
// 走读内核判记录、扫描内核的删除态扫描与墓碑那一遍都读这一份，walk.rs 的 INODE_RECORD_OBJECT_BIRTH_OFFSET、INODE_RECORD_NLINK_OFFSET、
// INODE_RECORD_SIZE_OFFSET）与删除中的 inode 的 nlink（records.rs 的 NLINK_OF_AN_INODE_BEING_DELETED）
const INODE_RECORD_OBJECT_BIRTH_OFFSET: u32 = 8u;
const INODE_RECORD_NLINK_OFFSET: u32 = 36u;
const INODE_RECORD_SIZE_OFFSET: u32 = 40u;
// inode 记录偏移 108 起的三段（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R1）：108–119 恒 0（填充 4、flags 8），
// 120 起截断字段 T（8 字节）与 N_w（8 字节），136–139 恒 0（预留 4）。恢复内核照核心层读者、走读内核照 walk.rs 的 I-9.7 读这几个数
const INODE_RECORD_RESERVED_BYTES_BEFORE_THE_TRUNCATION_FIELD_OFFSET: u32 = 108u;
const INODE_RECORD_RESERVED_BYTES_BEFORE_THE_TRUNCATION_FIELD_LENGTH: u32 = 12u;
const INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET: u32 = 120u;
const INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET: u32 = 128u;
const INODE_RECORD_RESERVED_BYTES_AFTER_THE_TRUNCATION_FIELD_OFFSET: u32 = 136u;
const INODE_RECORD_RESERVED_BYTES_AFTER_THE_TRUNCATION_FIELD_LENGTH: u32 = 4u;
// 系统配置 incompat 位图第一个字节里认得的两位（D15（格式冻结政策） 已定项 4）：位 1 布局身份（必须在）、位 2 inode 记录带截断字段
// （用户 2026-10-17 定新开，判决 m4-trunc-r2 第六节）；读者收 0x02 与 0x06，别的位一律当不认识
const INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT: u32 = 2u;
const INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT: u32 = 4u;
const INCOMPAT_BITS_THIS_READER_RECOGNIZES_IN_THE_FIRST_BYTE: u32 =
    INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT | INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT;
// 系统配置槽 incompat 位图那 32 字节（偏移 6 起）里这一版读者认得：第一个字节只带认得的位、布局身份那一位在，其余 31 字节全零
fn incompat_bits_are_recognized(base: u32) -> bool {
    let incompat0 = rd8(base, 6u);
    return (incompat0 & ~INCOMPAT_BITS_THIS_READER_RECOGNIZES_IN_THE_FIRST_BYTE) == 0u && all_zero(base, 7u, 31u)
        && (incompat0 & INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT) != 0u;
}
// 一槽带不带 incompat 位 2（只在 incompat_bits_are_recognized 之后读）
fn incompat_bits_carry_the_inode_record_truncation_field(base: u32) -> bool {
    return (rd8(base, 6u) & INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT) != 0u;
}
const NLINK_OF_AN_INODE_BEING_DELETED: u32 = 0u;
// records::InodeRecord::parse 的那几判（判决 m4-trunc-r2 R1）：偏移 108–119 与 136–139 两段恒零区非零、截断字段「T = 0 而 N_w ≠ 0」都判不可用
fn inode_record_parses_by_the_core_reader(base: u32, record_offset: u32) -> bool {
    if (!all_zero(base, record_offset + INODE_RECORD_RESERVED_BYTES_BEFORE_THE_TRUNCATION_FIELD_OFFSET,
            INODE_RECORD_RESERVED_BYTES_BEFORE_THE_TRUNCATION_FIELD_LENGTH)
        || !all_zero(base, record_offset + INODE_RECORD_RESERVED_BYTES_AFTER_THE_TRUNCATION_FIELD_OFFSET,
            INODE_RECORD_RESERVED_BYTES_AFTER_THE_TRUNCATION_FIELD_LENGTH)) {
        return false;
    }
    let truncation_field_txg = rd64(base, record_offset + INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET);
    let truncation_field_size = rd64(base, record_offset + INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET);
    return !is_zero64(truncation_field_txg) || is_zero64(truncation_field_size);
}
// records::InodeRecord::truncation_field_fits_the_pool：池带 incompat 位 2 时合法的字段都收，不带时 T 与 N_w 都要 0
fn inode_record_truncation_field_fits_the_pool(base: u32, record_offset: u32, carried: bool) -> bool {
    if (carried) {
        return true;
    }
    return all_zero(base, record_offset + INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET, 16u);
}
const MAGIC_SFSB: u32 = 0x42534653u;
const MAGIC_SFSR: u32 = 0x52534653u;
const MAGIC_SFSU: u32 = 0x55534653u;
const MAGIC_SFSJ: u32 = 0x4A534653u;

// 草稿区（每个状态一片，单位 u32）
// journal 记录表：环里自证过的记录按 (实例, 计数器) 去重收进来，恢复内核（scan_journal）与走读内核（逐盘、I-4.1 重放读的那一份 C_REPLAY_RECORDS）
// 都按它收、满了记判不了（原因 2）。环不绕回、不回收时条数随发布累计：墓碑容器那条流被判那两次发布落了之后 1636 条，
// 分配记录 2048 / 2050 条那两条流 864 / 865 条，所以从 512 抬到 2048。
// 再从 2048 抬到 4096（与分配记录那两张表抬到 8192 配套）：一版分配记录过 8192 的流要先写四千来个单元，一个单元一条 journal 记录，
// 8194 条那条流（顺序写 17 次各 200 个单元、再 201 个、再 144 个，被判那次缩回 1 个）盘 0 上 3749 条记录写（CPU 一侧量的），
// 2048 条时每个状态在扫环这一步就记判不了、碰不到分配记录那张表；装得下它的最小 2 的幂是 4096。
// 表挪到草稿区末尾（R_RECORDS，接在池级 checker 那几张表之后）：0 起原来那 12288 个字空着，其后的偏移不挪
const MAX_RECORDS: u32 = 4096u;
const RECORD_WORDS: u32 = 6u;
const MAX_ROOTS: u32 = 48u;
const R_ROOTS: u32 = 12288u;                                 // 96：版本、区域
// 中央映射树的叶（读一棵映射树时读到的码 2 叶节点）：一片一行，只记那片叶在表里的字节基；条目逐片从叶的字节里读（mapping_leaf_at），
// 不抄进表，映射有多少条条目都不判不了（用户 2026-10-19 定「按块分批读，不设上限」，与 R_DATA 那一段的数据指针同一个定案）。
// 行数受读树时见过的节点数限：恢复内核读一棵树至多 MAX_SEEN 个节点、走读内核一遍至多 C_MAX_VISITED 个单元，都不过这张表的行数（两处 const_assert）
const R_MAPPING_LEAVES: u32 = 12384u;                        // 3072：叶的字节基
const R_MAPPING_LEAF_ROWS: u32 = 3072u;
const_assert MAX_SEEN <= R_MAPPING_LEAF_ROWS;
const MAX_ACCOUNTING_ENTRIES: u32 = 512u;
const R_ACCOUNTING: u32 = 15456u;                            // 1536
// 恢复内核读一版分配记录树的那张表（allocation_count 条，满了记判不了、原因 3）：从 2048 抬到 4096 条，与扫描内核分配记录那张表
// （crash_judge_checker_common.wgsl 的 C_MAX_ALLOC_RECORDS）同宽。容器树旋钮压到一片 1 条、2 条的随机历史第 48、52 步起最新根的分配记录多于 2048 条
// （CPU 一侧数到第 59 步 2534 条、2342 条），两个内核在这里与站点 20 判不了。表本身挪到草稿区末尾（R_ALLOCATION，接在池级 checker 那几张表之后）：
// 16992 起原来那 12288 个字空着，其后的偏移不挪。
// 再从 4096 抬到 8192 条（与扫描内核那张同时抬，C894）：整理夹着回退那一组比重 40 步交判的 17 张镜像每版至多 4018 条，离 4096 只差 78；
// 同比重拉到 80 步那一组 45 张里 11 张、160 步那一组 105 张里 27 张多于 4096 条，至多 4728 条（CPU 一侧数的）。R_ROWS 与其后接在末尾的各表照偏移链往后挪 6 × 4096 = 24 576 字
const MAX_ALLOCATION_RECORDS: u32 = 8192u;
const ALLOCATION_RECORD_WORDS: u32 = 6u;
const MAX_SEEN: u32 = 512u;
const R_SEEN: u32 = 29280u;                                  // 1024
// 恢复内核读第一个文件的数据指针那一段（字节基、指针偏移，每条 2 字）：按块分批读，一块至多 DATA_POINTERS_PER_BLOCK 条，文件有多少个单元
// 都不判不了（用户 2026-10-19 定「按块分批读，不设上限」）。extent 树那一遍照旧走全树、核每一条指针、数单元，只把头一块记进来；
// walk_to_file 读数据单元读到这一块末尾时，从下段根往下找含下一个单元的那片叶，把那片叶里从它起的指针记成下一块（find_the_next_block_of_data_pointers）。
// 走读内核借这一段装由记录施加出来的版本清单（每版 1 字，至多 R_DATA_WORDS 版），扫描内核从这里读它
const R_DATA: u32 = 30304u;
const R_DATA_WORDS: u32 = 512u;
const DATA_POINTERS_PER_BLOCK: u32 = R_DATA_WORDS / 2u;
// 恢复内核读 extent 上段叶条目的那一段（字节基、条目偏移，每条 2 字）：按块分批读，一批至多 MAX_UPPER_ENTRIES 条，有数据的文件有多少个
// 都不判不了（用户 2026-10-19 定「按块分批读，不设上限」）；read_extent_tree_of_the_first_file 读完一批的下段再从上段根走一遍记下一批。
// 一批取 64 条（这一段装得下 256 条）：几十个文件就跨过一批的边界，对拍用例在扫描内核收单元的那张表（C_MAX_SCANNED）装得下的历史上就走得到下一批
const MAX_UPPER_ENTRIES: u32 = 64u;
const R_UPPER: u32 = 30816u;                                 // 512：字节基、偏移
// 恢复内核读一条根的实例表全部行的那张表（load_rows_of_the_instance_table_chain 与恢复落到的那一版那一读，满了记判不了、原因 3）：每行实例、txg lo、txg hi。
// 从 512 抬到 24576 行，与扫描内核读一条根自己实例表的那张表（C_R_MAX_ROWS）同宽：取号之后崩溃 65 × 369 次再挂载十二次的那个场景，
// 最新根的实例表 23997 行、66 片。表挪到草稿区末尾（R_ROWS）：31328 起原来那 1536 个字空着
const MAX_ROWS: u32 = 24576u;
// 共用那一半原地的表到这里为止，池级 checker 的表从这里起（crash_judge_checker_common.wgsl 的 C_REFERENCES0）
const SCRATCH_WORDS_REQUIRED: u32 = 32864u;
// 共用那一半挪到草稿区末尾的两张表：接在池级 checker 原有那几张表的末端（crash_judge_checker_common.wgsl 的 SCRATCH_END_BEFORE_THE_TABLES_GREW）之后，
// 池级 checker 抬上限之后挪过来的那几张表再接在它们后面（C_ALLOC_RECORDS 起）。恢复内核与两个池级 checker 内核共用每个状态的草稿区，
// 恢复内核写这两张、池级 checker 内核不碰它们
const R_ALLOCATION: u32 = 423864u;                           // 49152：盘、槽 lo/hi、跨度、代 lo/hi
const R_ROWS: u32 = R_ALLOCATION + ALLOCATION_RECORD_WORDS * MAX_ALLOCATION_RECORDS;   // 73728：实例、txg lo、txg hi
const END_OF_THE_RECOVERY_TABLES_MOVED_PAST_THE_CHECKER_TABLES: u32 = R_ROWS + 3u * MAX_ROWS;
// journal 记录表（MAX_RECORDS 抬到 4096 时挪过来）：接在池级 checker 那几张表的末端（crash_judge_checker_common.wgsl ⑮ 的末端，那里的 const_assert 核相等）之后，
// 走读内核 I-4.1 重放读的那一份环与 journal 组键两张跟着 MAX_RECORDS 长的表再接在它后面（同一份文件的 ⑯）。恢复内核与走读内核都写这一张
// （走读内核逐盘重收），扫描内核只读
const R_RECORDS: u32 = 1952255u;                             // 24576：每条 RECORD_WORDS 个字，两个内核各按自己的读法排
const END_OF_THE_JOURNAL_RECORD_TABLE_MOVED_PAST_THE_CHECKER_TABLES: u32 = R_RECORDS + RECORD_WORDS * MAX_RECORDS;

// 恢复内核判退时建上一版那一样读者（crash_judge_recovery.wgsl 的 rebuild_reader_kind，recovery::rebuild_version_with_its_snapshot_entries_judging_the_truncation_fields_as）
// 读 extent 树那一遍记下的每个有上段叶条目的文件，一个文件一行（FILE_ROW_WORDS 字）：inode 号 lo/hi、上段叶条目的字节基与偏移、标签、单元数，
// 再是 inode 树那一遍给它填的那条记录（找到没有、size lo/hi、对象出生代 lo/hi）。按 inode 号升序（上段按位置读回来的次序）。
// 放在共用那一半原地 16992 起空着的那一段（分配记录表挪走之后留下的，见 R_ALLOCATION），只有恢复内核的建上一版那一样读者写它；满了记判不了（原因 3）
const FILE_ROW_WORDS: u32 = 11u;
const MAX_FILES_OF_THE_EXTENT_TREE: u32 = 1024u;
const R_FILES: u32 = 16992u;                                 // 11264
const_assert R_FILES + FILE_ROW_WORDS * MAX_FILES_OF_THE_EXTENT_TREE <= R_SEEN;
// 读一棵 deadlist 树时读到的码 2 叶节点（一片一行，只记字节基，同 R_MAPPING_LEAVES）：建上一版那一样读者逐条解 deadlist 条目、拿它们的映射 key
// 查中央映射要它（recovery::deadlist_entries_parsed 与其后那一判）。行数受读树时见过的节点数限（MAX_SEEN）；放在 R_UPPER 之后空着的那一段
const R_DEADLIST_LEAVES: u32 = 31328u;                       // 512
const_assert R_UPPER + 2u * 256u <= R_DEADLIST_LEAVES;
const_assert R_DEADLIST_LEAVES + MAX_SEEN <= SCRATCH_WORDS_REQUIRED;

const MAX_DEVICES: u32 = 8u;
const MAX_FRAMES: u32 = 12u;

var<private> B: array<u32, TABLE_COUNT>;
var<private> persisted: array<u32, PERSISTED_MASK_WORDS>;
var<private> scratch_base: u32;
var<private> undecidable: u32;
var<private> record_count: u32;
// 看 journal 那一遍重放在哪一道停下（诊断：1 在飞上限、2 计数器对不上、3 无锚点首条不对、4 反向链断、5 首条序号不是 1、6 换了 txg、
// 7 序号不连续、8 提交标记没到就换事务、9 末条不带提交标记、10 末条之后同发布还有记录、11 点名单元验不过、12 施加了一版）
var<private> replay_stop_reason: u32;
var<private> root_count: u32;
// 中央映射：条目数（读映射树时数的全部叶条目）与 R_MAPPING_LEAVES 里记了几片叶
var<private> mapping_count: u32;
var<private> mapping_leaf_count: u32;
var<private> accounting_count: u32;
var<private> allocation_count: u32;
var<private> seen_count: u32;
// 第一个文件有几个数据单元（extent 树那一遍数的全部，不止记进 R_DATA 的那一块）
var<private> data_count: u32;
// 第一个文件之外各有数据单元的文件一共几个数据单元（extent 树那一遍数的；数映射条目要它，recovery 的 data_units_of_every_file）
var<private> data_units_of_the_files_other_than_the_first: u32;
// extent 树那一遍正在读的那个文件：读到几个单元、单元号是不是从 0 起连号（read_lower_segment 记，read_extent_tree_of_the_first_file 每条上段叶条目之前复位）
var<private> units_of_the_file_being_read: u32;
var<private> units_of_the_file_being_read_are_numbered_from_zero_without_a_gap: bool;
// 第一个文件那一条上段叶条目在哪（字节基、条目偏移；上段里没有它时字节基是 NONE）：上段叶条目按块分批记进 R_UPPER，读完 extent 树时
// R_UPPER 里是最后一批，walk_to_file 找第一个文件下一块数据指针时从这里取下段根
var<private> first_file_upper_entry_base: u32;
var<private> first_file_upper_entry_offset: u32;
// R_DATA 里此刻那一块：第一条是第一个文件的第几个单元、有几条
var<private> data_block_first: u32;
var<private> data_block_count: u32;
// extent 树那一遍读到的单元号是不是从 0 起连号（recovery::extents_of_the_first_file 的「没有洞」）：读的时候记、读完由调用方判
var<private> data_units_are_numbered_from_zero_without_a_gap: bool;
// 第一个文件在 extent 上段叶里那一条的标签（read_extent_tree_of_the_first_file 记下，上段里没有第一个文件那一条时是 NONE）：walk_to_file 按它核单元数、
// 读洞（标签 0、3、4 声明最后一个存在的单元之后到 size 是洞，core 的 TailOfAFile::HoleAfterTheLastDataUnit；判决 m4-trunc-r1 第三节 K5）
var<private> first_file_upper_leaf_tag: u32;
var<private> upper_count: u32;
var<private> row_count: u32;
// 给定状态表里这个状态的序号（crash_judge_tables::JudgeStateSource::GivenPersistedSets：第 k 个持久掩码、第 k 个恢复自称的那一版）；
// 按段展开的流不用它
var<private> given_state_ordinal: u32;
// 走读里一个读树、读节点的函数交回读不出、判不过时，它照 CPU 恢复那一处会报的 RecoveryFailure 成员记下失败点码（FAILURE_POINT_*，
// crash_judge_gpu.rs 的 gpu_failure_point_code）；调用它的那一处照它交 walk_failed。只在恢复内核的走读路径上写
var<private> walk_failure_point: u32;
// 中央映射树读不出时它的失败点（读得出是 FAILURE_POINT_NONE）：CPU 冷走读到第一次有提示读不出、要经映射回退时才读映射树
// （recovery 的 CentralMappingTreeReadOnFirstUse），走完各棵树、缺不缺树判完之后再要它一次；GPU 先读，读不出就记在这里、照那两个时点报
var<private> mapping_tree_failure_point: u32;
// 读一棵 deadlist 树时记进 R_DEADLIST_LEAVES 的叶有几片（read_code_two_tree 读 deadlist 那一支记）
var<private> deadlist_leaf_count: u32;
// 建上一版那一样读者读 extent 树时开着：read_extent_tree_of_the_first_file 把每个有上段叶条目的文件记进 R_FILES（files_of_the_extent_tree 行）
var<private> extent_tree_records_its_files: bool;
var<private> files_of_the_extent_tree: u32;
// 建上一版那一样读者读这一版的各棵树时开着（recovery::pointers_of_the_rebuilt_version_ascend_by_device 判的两遍，I-2.5）：读到的每一条指针
// （parse_node_pointer 全零的豁免、parse_data_pointer 不豁免）位置条目不按设备身份严格升序就记下；经 read_via_hint_then_mapping 读的每个进映射的单元，
// 映射里有它的 key 时，映射那两条位置条目不按设备身份严格升序也记下（重建之后指针换成映射位置再判的那一遍，transaction::take_the_mapping_positions_over_the_location_hints）。
// 判的地方在读者的最后，前面任何一步读不通都轮不到它
var<private> rebuild_judges_the_pointers_of_the_version: bool;
var<private> some_pointer_of_the_version_does_not_ascend_by_device: bool;
var<private> some_mapping_position_of_the_version_does_not_ascend_by_device: bool;

// ───────────────────────── 表与草稿的读写 ─────────────────────────

fn hdr(index: u32) -> u32 {
    return tables[B[T_HEADER] + index];
}
fn tab(table: u32, index: u32) -> u32 {
    return tables[B[table] + index];
}
// 一张表有几个字（目录里每张表的第二个字）
fn table_length(table: u32) -> u32 {
    return tables[table * 2u + 1u];
}
fn sw(index: u32) -> u32 {
    return scratch[scratch_base + index];
}
fn sws(index: u32, value: u32) {
    scratch[scratch_base + index] = value;
}

fn set_persisted(index: u32) {
    persisted[index >> 5u] = persisted[index >> 5u] | (1u << (index & 31u));
}
fn is_persisted(index: u32) -> bool {
    return (persisted[index >> 5u] & (1u << (index & 31u))) != 0u;
}

// 字节读：base 是 tables 里的绝对字下标，o 是字节偏移
fn rd8(base: u32, o: u32) -> u32 {
    let w = tables[base + (o >> 2u)];
    return (w >> ((o & 3u) << 3u)) & 0xFFu;
}
// 多字节读按字拼，不逐字节调 rd8：GPU 没有真正的函数调用，每个调用点展开一份被调函数，逐字节读让 rd32 一份展开成四份 rd8，
// 扫描内核里 rd8 展开近万份、占了展开预算的四成。拼法读的字与逐字节读的完全相同（跨字时才读下一个字），交回的值也相同
fn rd16(base: u32, o: u32) -> u32 {
    let word_index = base + (o >> 2u);
    let shift_in_bits = (o & 3u) << 3u;
    var value = tables[word_index] >> shift_in_bits;
    if ((o & 3u) == 3u) {
        value = value | (tables[word_index + 1u] << 8u);
    }
    return value & 0xFFFFu;
}
fn rd32(base: u32, o: u32) -> u32 {
    let word_index = base + (o >> 2u);
    let shift_in_bits = (o & 3u) << 3u;
    if (shift_in_bits == 0u) {
        return tables[word_index];
    }
    return (tables[word_index] >> shift_in_bits) | (tables[word_index + 1u] << (32u - shift_in_bits));
}
fn rd48(base: u32, o: u32) -> vec2<u32> {
    return vec2<u32>(rd32(base, o), rd16(base, o + 4u));
}
fn rd64(base: u32, o: u32) -> vec2<u32> {
    return vec2<u32>(rd32(base, o), rd32(base, o + 4u));
}
fn all_zero(base: u32, o: u32, n: u32) -> bool {
    for (var i = 0u; i < n; i = i + 1u) {
        if (rd8(base, o + i) != 0u) {
            return false;
        }
    }
    return true;
}
fn bytes_equal(base_a: u32, oa: u32, base_b: u32, ob: u32, n: u32) -> bool {
    for (var i = 0u; i < n; i = i + 1u) {
        if (rd8(base_a, oa + i) != rd8(base_b, ob + i)) {
            return false;
        }
    }
    return true;
}

// ───────────────────────── u64 ─────────────────────────

fn lt64(a: vec2<u32>, b: vec2<u32>) -> bool {
    return a.y < b.y || (a.y == b.y && a.x < b.x);
}
fn le64(a: vec2<u32>, b: vec2<u32>) -> bool {
    return !lt64(b, a);
}
fn eq64(a: vec2<u32>, b: vec2<u32>) -> bool {
    return a.x == b.x && a.y == b.y;
}
fn is_zero64(a: vec2<u32>) -> bool {
    return a.x == 0u && a.y == 0u;
}
fn u64_of(a: u32) -> vec2<u32> {
    return vec2<u32>(a, 0u);
}
fn add64(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    let lo = a.x + b.x;
    var hi = a.y + b.y;
    if (lo < a.x) {
        hi = hi + 1u;
    }
    return vec2<u32>(lo, hi);
}
fn add64_overflows(a: vec2<u32>, b: vec2<u32>) -> bool {
    let s = add64(a, b);
    return lt64(s, a);
}
fn sub64(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    let lo = a.x - b.x;
    var hi = a.y - b.y;
    if (a.x < b.x) {
        hi = hi - 1u;
    }
    return vec2<u32>(lo, hi);
}
// 左移 s 位，s 在 1..31
fn shl64(a: vec2<u32>, s: u32) -> vec2<u32> {
    if (s == 0u) {
        return a;
    }
    return vec2<u32>(a.x << s, (a.y << s) | (a.x >> (32u - s)));
}
fn shr64(a: vec2<u32>, s: u32) -> vec2<u32> {
    if (s == 0u) {
        return a;
    }
    return vec2<u32>((a.x >> s) | (a.y << (32u - s)), a.y >> s);
}
fn mul32x32(a: u32, b: u32) -> vec2<u32> {
    let a_lo = a & 0xFFFFu;
    let a_hi = a >> 16u;
    let b_lo = b & 0xFFFFu;
    let b_hi = b >> 16u;
    let p0 = a_lo * b_lo;
    let p1 = a_lo * b_hi;
    let p2 = a_hi * b_lo;
    let p3 = a_hi * b_hi;
    let mid = (p0 >> 16u) + (p1 & 0xFFFFu) + (p2 & 0xFFFFu);
    let lo = (p0 & 0xFFFFu) | (mid << 16u);
    let hi = p3 + (p1 >> 16u) + (p2 >> 16u) + (mid >> 16u);
    return vec2<u32>(lo, hi);
}
struct Product64 {
    value: vec2<u32>,
    overflow: bool,
};
// a × b（b 是 u32）；溢出置标志（饱和用）
fn mul64_32(a: vec2<u32>, b: u32) -> Product64 {
    let low = mul32x32(a.x, b);
    let high = mul32x32(a.y, b);
    var out: Product64;
    out.overflow = high.y != 0u;
    let sum = add64(vec2<u32>(low.x, low.y), vec2<u32>(0u, high.x));
    if (lt64(sum, low)) {
        out.overflow = true;
    }
    out.value = sum;
    return out;
}
fn mul64_32_saturating(a: vec2<u32>, b: u32) -> vec2<u32> {
    let p = mul64_32(a, b);
    if (p.overflow) {
        return vec2<u32>(NONE, NONE);
    }
    return p.value;
}
fn add64_saturating(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    if (add64_overflows(a, b)) {
        return vec2<u32>(NONE, NONE);
    }
    return add64(a, b);
}
fn sub64_saturating(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    if (lt64(a, b)) {
        return vec2<u32>(0u, 0u);
    }
    return sub64(a, b);
}
struct QuotientRemainder {
    quotient: vec2<u32>,
    remainder: vec2<u32>,
};
// 长除法；除数为 0 时商与余数都是 0（调用方保证除数非 0）
fn divmod64(a: vec2<u32>, d: vec2<u32>) -> QuotientRemainder {
    var out: QuotientRemainder;
    out.quotient = vec2<u32>(0u, 0u);
    out.remainder = vec2<u32>(0u, 0u);
    if (is_zero64(d)) {
        return out;
    }
    for (var i = 0u; i < 64u; i = i + 1u) {
        let bit_index = 63u - i;
        var bit = 0u;
        if (bit_index >= 32u) {
            bit = (a.y >> (bit_index - 32u)) & 1u;
        } else {
            bit = (a.x >> bit_index) & 1u;
        }
        out.remainder = shl64(out.remainder, 1u);
        out.remainder.x = out.remainder.x | bit;
        if (le64(d, out.remainder)) {
            out.remainder = sub64(out.remainder, d);
            if (bit_index >= 32u) {
                out.quotient.y = out.quotient.y | (1u << (bit_index - 32u));
            } else {
                out.quotient.x = out.quotient.x | (1u << bit_index);
            }
        }
    }
    return out;
}
struct QuotientAndSmallRemainder {
    quotient: vec2<u32>,
    remainder: u32,
};
// 64 位的数除以一个小于 65536 的除数：按 32、16、16 位三段做长除法，每段的被除数装得进 u32（上一段的余数小于除数、左移 16 位不越过 32 位）。
// 段内序号满 64 位时照它逐位拆（判决 m3-crash-enumeration-merge-r2 第二节 L1-7）；比 divmod64 的 64 轮少得多
fn divide_by_a_divisor_below_65536(value: vec2<u32>, divisor: u32) -> QuotientAndSmallRemainder {
    var out: QuotientAndSmallRemainder;
    let quotient_high = value.y / divisor;
    let upper = ((value.y % divisor) << 16u) | (value.x >> 16u);
    let lower = ((upper % divisor) << 16u) | (value.x & 0xFFFFu);
    out.quotient = vec2<u32>(((upper / divisor) << 16u) | (lower / divisor), quotient_high);
    out.remainder = lower % divisor;
    return out;
}
fn slot_to_bytes(slot: vec2<u32>) -> vec2<u32> {
    return shl64(slot, SLOT_SHIFT);
}

// ───────────────────────── 落点、版本、读 ─────────────────────────

fn location_word(location: u32, k: u32) -> u32 {
    return tab(T_LOCATIONS, location * LOCATION_WORDS + k);
}
fn version_of_location(location: u32) -> u32 {
    let writes_first = location_word(location, 5u);
    let writes_count = location_word(location, 6u);
    // 组合掩码 COMBINATION_WORDS_OF_A_VERSION 个字：第 i 次写在第 i / 32 字的第 i % 32 位（建表时核过一个落点的写不多于这几字装得下的）
    var combination: array<u32, COMBINATION_WORDS_OF_A_VERSION>;
    for (var word = 0u; word < COMBINATION_WORDS_OF_A_VERSION; word = word + 1u) {
        combination[word] = 0u;
    }
    for (var i = 0u; i < writes_count; i = i + 1u) {
        if (is_persisted(tab(T_LOCATION_WRITES, writes_first + i))) {
            combination[i >> 5u] = combination[i >> 5u] | (1u << (i & 31u));
        }
    }
    // 这个落点的版本在第 ⌈写数 ÷ 32⌉ 字往后都是 0，只比前面那几字
    let combination_words_in_use = (writes_count + 31u) / 32u;
    let versions_first = location_word(location, 7u);
    let versions_count = location_word(location, 8u);
    for (var i = 0u; i < versions_count; i = i + 1u) {
        let version_words = (versions_first + i) * VERSION_WORDS;
        var is_this_version = true;
        for (var word = 0u; word < combination_words_in_use; word = word + 1u) {
            if (tab(T_VERSIONS, version_words + version_word_of_the_combination_word(word)) != combination[word]) {
                is_this_version = false;
                break;
            }
        }
        if (is_this_version) {
            return versions_first + i;
        }
    }
    undecidable = UNDECIDABLE_NO_VERSION;
    return NONE;
}
// 版本表里组合掩码第 word 字在一条版本的第几字（crash_judge_tables::version_word_of_the_combination_word 同一个布局）：
// 第 0 字在第 0 字、第 1 字在第 7 字（64 位那时候的两个字，偏移不动），其余接在第 8 字之后
fn version_word_of_the_combination_word(word: u32) -> u32 {
    if (word == 0u) {
        return 0u;
    }
    if (word == 1u) {
        return 7u;
    }
    return VERSION_WORD_COMBINATION_PAST_THE_SIXTY_FOURTH_WRITE + word - 2u;
}
struct Slice {
    version: u32,
    base: u32,
    length: u32,
    c0: u32,
    c1: u32,
    c2: u32,
    c3: u32,
};
fn no_slice() -> Slice {
    var s: Slice;
    s.version = NONE;
    s.base = 0u;
    s.length = 0u;
    s.c0 = 0u;
    s.c1 = 0u;
    s.c2 = 0u;
    s.c3 = 0u;
    return s;
}
fn slice_of_version(version: u32, location: u32) -> Slice {
    var s: Slice;
    s.version = version;
    s.base = B[T_VERSION_BYTES] + tab(T_VERSIONS, version * VERSION_WORDS + 1u);
    s.length = location_word(location, 3u);
    s.c0 = tab(T_VERSIONS, version * VERSION_WORDS + 2u);
    s.c1 = tab(T_VERSIONS, version * VERSION_WORDS + 3u);
    s.c2 = tab(T_VERSIONS, version * VERSION_WORDS + 4u);
    s.c3 = tab(T_VERSIONS, version * VERSION_WORDS + 5u);
    return s;
}
// 读 (盘, 偏移, 长度)：起点是落点起点、长度不超过落点的才读得到；起点不是落点的，单元区里没写过的槽读出全零
// （zero_unit_version_over_a_slot_nothing_wrote）。切片的字节与检查字照版本表那一条（slice_of_version 同一个读法），长度按落点或全零单元。
// 空段的最后一槽按 32 KiB 读（拼读，判决 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第二节 S2 那一行与第三节第 1 条，第 32 批攻方 Z2；
// 正推说的「空槽加下一已写槽前 16 KiB」同一形）：CPU 的稀疏读者读出「16 KiB 全零 + 紧接着的下一个落点的前 16 KiB」。GPU 拿全零 16 KiB 那一条的第 8 字
// （VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES）异或下一个落点那一版的前 16 KiB 的 CRC，拼出整段的 CRC 放进 c1，c2 只留「有 32 KiB」与拼读两位
// （SLICE_SPLICED_ACROSS_THE_END_OF_A_LOCATION）：字节只有前 16 KiB 是这一读的，按位置条目比 CRC 对得上时交 CPU 判（slice_crc_matches）。
// 拼读切片后 16 KiB 没有字节（base 指的是全零 16 KiB 那一条版本字节，再往后不是下一个落点的字节），调用方只能比 CRC 或读前 16 KiB。
// 下一个落点查不到版本（undecidable 已记）、空段止于盘尾（CPU 越过盘尾读不到）的交 no_slice。「16 KiB 节点加下一槽」不用拼：单元槽落点本来就是
// 32 KiB 长（盘尾那一槽 16 KiB，CPU 读 32 KiB 越过盘尾），版本字节里后 16 KiB 就是叠上去的下一槽
fn read_at(device: u32, offset: vec2<u32>, length: u32) -> Slice {
    // 落点表按 (盘, 偏移) 升序二分，lo 落在排在 (device, offset) 之后的第一个落点上；lo − 1 是最后一个不排在它之后的落点，
    // 也就是最后一次走「lo = mid + 1」的那个 mid：它在不在这块盘上、起点是不是恰为 offset，在那一次记下
    var lo = 0u;
    var hi = hdr(6u);
    var before_is_on_this_device = false;
    var before_starts_at_the_offset = false;
    while (lo < hi) {
        let mid = (lo + hi) >> 1u;
        let d = location_word(mid, 0u);
        let o = vec2<u32>(location_word(mid, 1u), location_word(mid, 2u));
        if (d < device || (d == device && le64(o, offset))) {
            lo = mid + 1u;
            before_is_on_this_device = d == device;
            before_starts_at_the_offset = before_is_on_this_device && eq64(o, offset);
        } else {
            hi = mid;
        }
    }
    if (!before_is_on_this_device) {
        return no_slice();
    }
    let location = lo - 1u;
    var version = NONE;
    var unit_length = 0u;
    // 要查版本的落点：读的那个落点，或拼读时紧接着的下一个落点（lo）——一个调用点，内核展开不多一份查版本
    var location_to_look_up = NONE;
    var spliced = false;
    if (before_starts_at_the_offset) {
        unit_length = location_word(location, 3u);
        if (length <= unit_length) {
            location_to_look_up = location;
        }
    } else {
        version = zero_unit_version_over_a_slot_nothing_wrote(location, offset, length);
        unit_length = DATA_UNIT_BYTES >> (version & 1u);
        // 拼读的后一半是紧接着的下一个落点 lo（zero_unit_version_over_a_slot_nothing_wrote 交这个码时空段不止于盘尾，止于 lo 的起点）
        spliced = version == ZERO_SIXTEEN_KIBIBYTES_FOLLOWED_BY_THE_NEXT_SLOT;
        if (spliced) {
            location_to_look_up = lo;
        }
    }
    var version_looked_up = NONE;
    if (location_to_look_up != NONE) {
        version_looked_up = version_of_location(location_to_look_up);
    }
    if (before_starts_at_the_offset) {
        version = version_looked_up;
    } else if (spliced) {
        version = select(ZERO_UNIT_VERSION_OF_SIXTEEN_KIBIBYTES, NONE, version_looked_up == NONE);
    }
    if (version == NONE || version == ZERO_SIXTEEN_KIBIBYTES_FOLLOWED_BY_THE_NEXT_SLOT || (!spliced && length > unit_length)) {
        return no_slice();
    }
    var s: Slice;
    s.version = version;
    s.base = B[T_VERSION_BYTES] + tab(T_VERSIONS, version * VERSION_WORDS + 1u);
    s.length = unit_length;
    s.c0 = tab(T_VERSIONS, version * VERSION_WORDS + 2u);
    s.c1 = tab(T_VERSIONS, version * VERSION_WORDS + 3u);
    s.c2 = tab(T_VERSIONS, version * VERSION_WORDS + 4u);
    s.c3 = tab(T_VERSIONS, version * VERSION_WORDS + 5u);
    if (spliced) {
        s.length = DATA_UNIT_BYTES;
        s.c1 = tab(T_VERSIONS, version * VERSION_WORDS + VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES)
            ^ tab(T_VERSIONS, version_looked_up * VERSION_WORDS + 2u);
        s.c2 = 4u | SLICE_SPLICED_ACROSS_THE_END_OF_A_LOCATION;
        s.c3 = 0u;
    }
    return s;
}
// 单元区里一个没有落点的 16 KiB 槽（判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第三节第 2 条，攻方改法 F-零槽）：
// 基线与枚举写表里每次写碰过的槽都有落点，没有落点就是这条流的每个崩溃状态里都没写过，CPU 的稀疏读者（memory_pool 的 PoolReader）读出全零。
// before 是同一块盘上排在 offset 前面最近的落点：它的第 9、10 字是它之后、下一个落点之前在单元区里、整 16 KiB 在盘内、谁都没写过的槽 [第 9 字, 第 10 字)，
// 第 11 字是从哪一槽起整 32 KiB 不在盘内（crash_judge_tables::UnitSlotsNothingWroteAfterALocation）。读的起点按槽对齐、读的几槽都在那一段里，
// 交回版本表最前面那两条全零单元之一（crash_judge_tables::ZERO_UNIT_VERSIONS：第 0 条 32 KiB，盘尾只装得下 16 KiB 的槽交第 1 条 16 KiB）；
// 起点是空段的最后一槽、读两槽、整 32 KiB 在盘内的交 ZERO_SIXTEEN_KIBIBYTES_FOLLOWED_BY_THE_NEXT_SLOT（read_at 拼读）：那时空段止于下一个落点的起点
// （空段的终点取同一块盘上下一个落点的起点与盘尾里小的那个，crash_judge_tables::unit_slots_nothing_wrote_after_each_location；整 32 KiB 在盘内 ⇒ 下一槽不过盘尾），
// 下一个落点就是 before + 1、是单元槽（空段在单元区里，之后的落点都是单元槽）。别的交 NONE
fn zero_unit_version_over_a_slot_nothing_wrote(before: u32, offset: vec2<u32>, length: u32) -> u32 {
    // 起点不按槽对齐、或槽号装不进 32 位（偏移 ≥ 2⁴⁶，盘装不下）的不认：槽号 = 偏移 >> SLOT_SHIFT 装进 32 位要偏移 < 2^(32 + SLOT_SHIFT) = 2⁴⁶，
    // 也就是偏移的高 32 位 offset.y < 2^SLOT_SHIFT、offset.y >> SLOT_SHIFT 为 0，即下面守卫里的那一项（判决 `research/prompts/m3-batch32-code-r1-main-verification.md`
    // 第二节 K1 / K2 P4 回绕那一行：按 offset.y >> (32 − SLOT_SHIFT) 判时槽号在 [2³², 2³⁶) 回绕成低 32 位）
    let first_slot = (offset.x >> SLOT_SHIFT) | (offset.y << (32u - SLOT_SHIFT));
    let slots_read = (length + NODE_BYTES - 1u) >> SLOT_SHIFT;
    let first_slot_past_the_gap = location_word(before, 10u);
    // 读的几槽都在空段 [第 9 字, 第 10 字) 里：先判起点不越过空段的终点，再拿差比，不做会回绕的 first_slot + slots_read
    if (((offset.x & (NODE_BYTES - 1u)) | (offset.y >> SLOT_SHIFT)) != 0u || first_slot < location_word(before, 9u)
        || first_slot > first_slot_past_the_gap) {
        return NONE;
    }
    let slots_left_in_the_gap = first_slot_past_the_gap - first_slot;
    let whole_data_unit_fits = first_slot < location_word(before, 11u);
    if (slots_left_in_the_gap < slots_read) {
        return select(NONE, ZERO_SIXTEEN_KIBIBYTES_FOLLOWED_BY_THE_NEXT_SLOT, slots_left_in_the_gap == 1u && slots_read == 2u && whole_data_unit_fits);
    }
    return select(0u, 1u, !whole_data_unit_fits);
}
fn read_location(location: u32) -> Slice {
    let version = version_of_location(location);
    if (version == NONE) {
        return no_slice();
    }
    return slice_of_version(version, location);
}
fn device_is_in_the_pool(device: u32) -> bool {
    let count = hdr(3u);
    for (var i = 0u; i < count; i = i + 1u) {
        if (hdr(12u + i) == device) {
            return true;
        }
    }
    return false;
}
fn device_bytes() -> vec2<u32> {
    return vec2<u32>(hdr(4u), hdr(5u));
}

// ───────────────────────── 指针 ─────────────────────────

// 指针头部 extent 偏移那 2 字节在指针里的偏移（D19（块指针的结构与宽度预算） 已定项 7 的字段表：MAC 16、nonce 12、算法类型 1、压缩算法码 1、
// 压后长度 2 之后；`singlefs_checker::image` 的 POINTER_EXTENT_OFFSET_FIELD 是 32..34）。第一版指针指整个单元，这 2 字节恒 0
const POINTER_EXTENT_OFFSET_FIELD_OFFSET: u32 = 32u;
// 偏移 o 起那条指针头部的 extent 偏移为 0：恢复内核的读者只要这一判时不解整条指针
fn pointer_extent_offset_is_zero(base: u32, o: u32) -> bool {
    return rd16(base, o + POINTER_EXTENT_OFFSET_FIELD_OFFSET) == 0u;
}
// 偏移 o 起那条指针头部 extent 偏移之前的几段是第一版的取值（pointer.rs 的 PointerHeadAsRead::judged 在 extent 偏移之前判的那几段）：
// MAC 16 与 nonce 12 全 0、算法类型 0（不加密）、压缩算法码 0、压后长度 0。core 读走读跟随的指针（树表条目树根指针、inode 内部条目子指针、
// extent 上段叶两种指针、下段叶数据指针、extent 内部条目子指针、分配记录树内部条目子指针、实例表链指针）都经
// read_judging_the_encryption_compression_and_extent_offset_fields_from 一并判这几段与 extent 偏移，有一段不是第一版的取值就当那个结构损坏
fn pointer_encryption_and_compression_fields_are_the_first_version(base: u32, o: u32) -> bool {
    return all_zero(base, o, 28u) && rd8(base, o + 28u) == 0u && rd8(base, o + 29u) == 0u && rd16(base, o + 30u) == 0u;
}

struct Ptr {
    all_zero: bool,
    head_ok: bool,
    mac_nonce_zero: bool,
    extent_offset_zero: bool,
    birth_tree: vec2<u32>,
    birth_txg: vec2<u32>,
    device0: u32,
    slot0: vec2<u32>,
    sum0: u32,
    device1: u32,
    slot1: vec2<u32>,
    sum1: u32,
    instance: u32,
    tail: vec2<u32>,
};
fn parse_pointer_common(base: u32, o: u32) -> Ptr {
    var p: Ptr;
    p.all_zero = false;
    p.mac_nonce_zero = all_zero(base, o, 28u);
    p.head_ok = pointer_encryption_and_compression_fields_are_the_first_version(base, o);
    p.extent_offset_zero = pointer_extent_offset_is_zero(base, o);
    p.birth_tree = rd64(base, o + 34u);
    p.birth_txg = rd64(base, o + 42u);
    p.device0 = rd32(base, o + 50u);
    p.slot0 = rd48(base, o + 54u);
    p.sum0 = rd32(base, o + 60u);
    p.device1 = rd32(base, o + 64u);
    p.slot1 = rd48(base, o + 68u);
    p.sum1 = rd32(base, o + 74u);
    p.instance = rd32(base, o + 78u);
    p.tail = vec2<u32>(0u, 0u);
    return p;
}
fn parse_node_pointer(base: u32, o: u32) -> Ptr {
    var p = parse_pointer_common(base, o);
    p.tail = vec2<u32>(rd32(base, o + 82u), 0u);
    p.all_zero = all_zero(base, o, 86u);
    // NodePointer::location_entries_ascend_by_device：整条全零的空指针（empty_root）豁免
    if (rebuild_judges_the_pointers_of_the_version && !p.all_zero && p.device0 >= p.device1) {
        some_pointer_of_the_version_does_not_ascend_by_device = true;
    }
    return p;
}
fn parse_data_pointer(base: u32, o: u32) -> Ptr {
    var p = parse_pointer_common(base, o);
    p.tail = rd48(base, o + 82u);
    p.all_zero = all_zero(base, o, 88u);
    // pointer::location_entries_ascend_by_device：数据指针不豁免
    if (rebuild_judges_the_pointers_of_the_version && p.device0 >= p.device1) {
        some_pointer_of_the_version_does_not_ascend_by_device = true;
    }
    return p;
}
fn same_pointer(a: Ptr, b: Ptr) -> bool {
    return a.device0 == b.device0 && eq64(a.slot0, b.slot0) && a.sum0 == b.sum0
        && a.device1 == b.device1 && eq64(a.slot1, b.slot1) && a.sum1 == b.sum1
        && a.instance == b.instance && eq64(a.tail, b.tail)
        && eq64(a.birth_tree, b.birth_tree) && eq64(a.birth_txg, b.birth_txg);
}
// 位置条目里的整单元校验和对得上这一读的 CRC 吗
// 一读的 32 KiB 整段 CRC 对不对得上（只比 CRC、不往下读字节：拼读那一读拼出的 CRC 照比，不记判不了）
fn slice_whole_read_crc_matches(s: Slice, checksum: u32) -> bool {
    return (s.c2 & 4u) != 0u && s.c1 == checksum;
}
// 拼读（read_at）那一读拼出的整段 CRC 对得上：往下要读跨过落点末尾的那几个字节，表里没有，记判不了、交 CPU 判（判决
// `research/prompts/m3-batch33s-code-r1-main-verification.md` 第三节第 1 条「做不出的那一支记判不了、回落 CPU」）；对不上的照判对不上
fn slice_crc_matches(s: Slice, unit_bytes: u32, checksum: u32) -> bool {
    if (unit_bytes == NODE_BYTES) {
        return s.c0 == checksum;
    }
    if ((s.c2 & 4u) == 0u) {
        return false;
    }
    if (s.c1 == checksum && (s.c2 & SLICE_SPLICED_ACROSS_THE_END_OF_A_LOCATION) != 0u) {
        undecidable = UNDECIDABLE_SPLICED_READ_MATCHES;
        return false;
    }
    return s.c1 == checksum;
}
// 按两条位置条目读一个单元（recovery::read_unit_via_locations）
// 两条位置条目在一个循环里读（一个 read_at 调用点：read_at 被这个函数展开的份数减半，扫描内核才装得下拼读，判决
// `research/prompts/m3-batch33s-code-r1-main-verification.md` 第三节第 1 条「扫描内核余量只剩 17741，先量再加」）；次序与两次调用相同，第一条对得上就不读第二条
fn read_unit_via_locations(p: Ptr, unit_bytes: u32) -> Slice {
    for (var copy = 0u; copy < 2u; copy = copy + 1u) {
        let is_the_second = copy == 1u;
        let s = read_at(select(p.device0, p.device1, is_the_second), slot_to_bytes(select(p.slot0, p.slot1, is_the_second)), unit_bytes);
        if (s.version != NONE && slice_crc_matches(s, unit_bytes, select(p.sum0, p.sum1, is_the_second))) {
            return s;
        }
    }
    return no_slice();
}

// ───────────────────────── 系统配置 ─────────────────────────

struct SysConfig {
    kind: u32,          // 0 可择、1 自证不过、2 incompat 不认识、3 整池拒
    refusal: u32,
    fsid: vec4<u32>,
    this_device: u32,
    device_count: u32,
    generation: vec2<u32>,
    physical_block_size: u32,
    spacing: u32,
    slots_per_region: u32,
    region0: u32,
    region1: u32,
    region2: u32,
    ring_bytes: vec2<u32>,
    floor: vec2<u32>,
    unit_area_start_slot: vec2<u32>,
    // incompat 位 2：这一池的 inode 记录带截断字段（核心层 SystemImmutableConfiguration 的 inode_record_truncation_fields）
    carries_the_inode_record_truncation_field: bool,
};
fn empty_system_configuration(kind: u32, refusal: u32) -> SysConfig {
    var c: SysConfig;
    c.kind = kind;
    c.refusal = refusal;
    c.fsid = vec4<u32>(0u, 0u, 0u, 0u);
    c.this_device = 0u;
    c.device_count = 0u;
    c.generation = vec2<u32>(0u, 0u);
    c.physical_block_size = 0u;
    c.spacing = 0u;
    c.slots_per_region = 0u;
    c.region0 = 0u;
    c.region1 = 0u;
    c.region2 = 0u;
    c.ring_bytes = vec2<u32>(0u, 0u);
    c.floor = vec2<u32>(0u, 0u);
    c.unit_area_start_slot = vec2<u32>(0u, 0u);
    c.carries_the_inode_record_truncation_field = false;
    return c;
}
fn parse_system_configuration(s: Slice) -> SysConfig {
    if (s.version == NONE || s.length < SYSTEM_CONFIGURATION_SLOT_BYTES) {
        return empty_system_configuration(1u, 0u);
    }
    let base = s.base;
    if (rd32(base, 0u) != MAGIC_SFSB || s.c0 != 1u) {
        return empty_system_configuration(1u, 0u);
    }
    // recovery::mountable_slot_or_refusal 的 IncompatBitsNotRecognized 那一臂：格式版本不是这个读者写的那一版（1）时按格式版本整池拒
    // （拒收码 2，与下面读者收不收的池级值里格式版本那一判同一个码），是 1 才算 incompat 位图不认识（C663，用户 2026-10-20 定「格式版本先」）
    if (!incompat_bits_are_recognized(base)) {
        if (rd16(base, 4u) != 1u) {
            return empty_system_configuration(3u, 2u);
        }
        return empty_system_configuration(2u, 0u);
    }
    let slots_per_region = rd8(base, 362u);
    if (slots_per_region < 4u || slots_per_region > 16u) {
        return empty_system_configuration(3u, 1u);
    }
    // 读者收不收的池级值（recovery::system_configuration_values_this_reader_accepts）
    if (rd16(base, 4u) != 1u) {
        return empty_system_configuration(3u, 2u);
    }
    if (rd8(base, 219u) != 0u) {
        return empty_system_configuration(3u, 3u);
    }
    let spacing = rd32(base, 429u);
    let root_ring_base_slot = rd64(base, 371u);
    var largest_spacing = vec2<u32>(NONE, NONE);
    if (root_ring_base_slot.y < (1u << 18u)) {
        let base_bytes = slot_to_bytes(root_ring_base_slot);
        if (lt64(base_bytes, u64_of(SYSTEM_CONFIGURATION_SLOT_BYTES))) {
            return empty_system_configuration(3u, 4u);
        }
        largest_spacing = sub64(base_bytes, u64_of(SYSTEM_CONFIGURATION_SLOT_BYTES));
    }
    if (spacing < 4096u || lt64(largest_spacing, u64_of(spacing))) {
        return empty_system_configuration(3u, 4u);
    }
    let physical_block_size = rd32(base, 317u);
    if (physical_block_size < ROOT_RECORD_BYTES || physical_block_size > spacing) {
        return empty_system_configuration(3u, 5u);
    }
    if (!eq64(root_ring_base_slot, u64_of(64u))) {
        return empty_system_configuration(3u, 6u);
    }
    if (!eq64(rd64(base, 325u), u64_of(1024u))) {
        return empty_system_configuration(3u, 7u);
    }
    // 根环区域数 R（偏移 361 那 1 字节）不是第一版的 3：整池拒，排在 journal 环起点之后、环长之前（recovery::system_configuration_values_this_reader_accepts
    // 的 RootRingRegionCountNotTheFirstVersionValue，C663，用户 2026-10-20 定「R 读并整池拒」）。拒收码 14 只是新加的一个，前面那些码不挪
    if (rd8(base, 361u) != 3u) {
        return empty_system_configuration(3u, 14u);
    }
    // 根环区域步长 P（363）、chunk（367）不是第一版常量，或 S × 槽距大于 P × chunk：整池拒，排在 R 之后、环长之前
    // （recovery::root_ring_regions_fit_their_stride 的 RootRingPrimeStepNotTheFirstVersionValue、RootRingChunkBytesNotTheFirstVersionValue、
    // RootRingRegionLongerThanTheRegionStride；D22（单元原子性怎么合成） 已定项 16 第 1 句，用户 2026-10-22 定）。拒收码 15、16、17 是新加的，前面那些码不挪。
    // S × 槽距在 u32 里会溢出，按 槽距 ≤ ⌊P × chunk ÷ S⌋ 判（S 在 [4, 16] 里、不是 0，上面判过）：两种写法对整数等价
    if (rd32(base, 363u) != ROOT_RING_PRIME_STEP) {
        return empty_system_configuration(3u, 15u);
    }
    if (rd32(base, 367u) != ROOT_RING_CHUNK_BYTES) {
        return empty_system_configuration(3u, 16u);
    }
    if (spacing > ROOT_RING_REGION_STRIDE / slots_per_region) {
        return empty_system_configuration(3u, 17u);
    }
    let ring_bytes = rd64(base, 333u);
    let in_flight = divmod64(shr64(ring_bytes, 12u), u64_of(3u)).quotient;
    if (is_zero64(in_flight) || in_flight.y != 0u) {
        return empty_system_configuration(3u, 8u);
    }
    let ring_start = u64_of(JOURNAL_RING_START_BYTES);
    if (add64_overflows(ring_start, ring_bytes)) {
        return empty_system_configuration(3u, 8u);
    }
    let ring_end = add64(ring_start, ring_bytes);
    let unit_area_start_slot = rd64(base, 417u);
    if (unit_area_start_slot.y < (1u << 18u)) {
        if (lt64(slot_to_bytes(unit_area_start_slot), ring_end)) {
            return empty_system_configuration(3u, 8u);
        }
    }
    let slot_after_the_ring = add64(u64_of(1024u), shr64(add64(ring_bytes, u64_of(16383u)), SLOT_SHIFT));
    if (!eq64(unit_area_start_slot, slot_after_the_ring)) {
        return empty_system_configuration(3u, 9u);
    }
    if ((slot_after_the_ring.x & 63u) != 0u) {
        return empty_system_configuration(3u, 10u);
    }
    // I-8.1 的三条等式，放在最后、按这个次序（recovery::journal_ring_geometry_fields_recompute_from_the_ring_length）：
    // F（357）= 格式常量 3、在飞上限（345）= ⌊环长 ÷ 4096 ÷ 3⌋（上面算好的 in_flight，装得进 4 字节）、最坏占用（349）= 在飞上限 × 4096
    if (rd32(base, 357u) != 3u) {
        return empty_system_configuration(3u, 11u);
    }
    if (rd32(base, 345u) != in_flight.x) {
        return empty_system_configuration(3u, 12u);
    }
    if (!eq64(rd64(base, 349u), shl64(in_flight, 12u))) {
        return empty_system_configuration(3u, 13u);
    }
    var c = empty_system_configuration(0u, 0u);
    c.fsid = vec4<u32>(rd32(base, 102u), rd32(base, 106u), rd32(base, 110u), rd32(base, 114u));
    c.this_device = rd32(base, 139u);
    c.device_count = rd32(base, 143u);
    c.generation = rd64(base, 147u);
    c.physical_block_size = physical_block_size;
    c.spacing = spacing;
    c.slots_per_region = slots_per_region;
    c.region0 = rd32(base, 379u);
    c.region1 = rd32(base, 383u);
    c.region2 = rd32(base, 387u);
    c.ring_bytes = ring_bytes;
    c.floor = rd64(base, 481u);
    c.unit_area_start_slot = unit_area_start_slot;
    c.carries_the_inode_record_truncation_field = incompat_bits_carry_the_inode_record_truncation_field(base);
    return c;
}
// 全池没有一个槽 0 可择时逐档找槽 1
fn slot_one_by_trying_every_spacing(device: u32) -> SysConfig {
    var best = empty_system_configuration(1u, 0u);
    var have_best = false;
    var incompat = empty_system_configuration(1u, 0u);
    var have_incompat = false;
    let largest = ROOT_RING_BASE_BYTES - SYSTEM_CONFIGURATION_SLOT_BYTES;
    for (var candidate = 4096u; candidate <= largest; candidate = candidate + 512u) {
        let s = parse_system_configuration(read_at(device, u64_of(candidate), SYSTEM_CONFIGURATION_SLOT_BYTES));
        if (s.kind == 3u) {
            return s;
        }
        if (s.kind == 0u) {
            if (s.spacing != candidate) {
                continue;
            }
            if (!have_best || lt64(best.generation, s.generation)) {
                best = s;
                have_best = true;
            }
        } else if (s.kind == 2u && !have_incompat) {
            incompat = s;
            have_incompat = true;
        }
    }
    if (have_best) {
        return best;
    }
    if (have_incompat) {
        return incompat;
    }
    return empty_system_configuration(1u, 0u);
}
struct SysChoice {
    ok: bool,
    failure: u32,
    cfg: SysConfig,
};
fn choose_system_configuration() -> SysChoice {
    var out: SysChoice;
    out.ok = false;
    out.failure = 0u;
    out.cfg = empty_system_configuration(1u, 0u);
    let device_count = hdr(3u);
    if (device_count > MAX_DEVICES) {
        undecidable = UNDECIDABLE_TOO_MANY_DEVICES;
        return out;
    }
    var slot_zero: array<SysConfig, 8>;
    var first_spacing = NONE;
    for (var d = 0u; d < device_count; d = d + 1u) {
        let device = hdr(12u + d);
        slot_zero[d] = parse_system_configuration(read_at(device, u64_of(0u), SYSTEM_CONFIGURATION_SLOT_BYTES));
        if (slot_zero[d].kind == 3u) {
            out.failure = failure_point_of_a_refused_system_configuration(slot_zero[d]);
            return out;
        }
        if (slot_zero[d].kind == 0u && first_spacing == NONE) {
            first_spacing = slot_zero[d].spacing;
        }
    }
    var chosen_valid = false;
    var first_no_valid = false;
    var first_incompat = false;
    for (var d = 0u; d < device_count; d = d + 1u) {
        let device = hdr(12u + d);
        var spacing = first_spacing;
        if (slot_zero[d].kind == 0u) {
            spacing = slot_zero[d].spacing;
        }
        var slot_one: SysConfig;
        if (spacing != NONE) {
            slot_one = parse_system_configuration(read_at(device, u64_of(spacing), SYSTEM_CONFIGURATION_SLOT_BYTES));
        } else {
            slot_one = slot_one_by_trying_every_spacing(device);
        }
        if (slot_one.kind == 3u) {
            out.failure = failure_point_of_a_refused_system_configuration(slot_one);
            return out;
        }
        if (!first_incompat && (slot_zero[d].kind == 2u || slot_one.kind == 2u)) {
            first_incompat = true;
        }
        var best: SysConfig;
        if (slot_zero[d].kind != 0u && slot_one.kind != 0u) {
            first_no_valid = true;
            continue;
        } else if (slot_zero[d].kind == 0u && slot_one.kind != 0u) {
            best = slot_zero[d];
        } else if (slot_zero[d].kind != 0u && slot_one.kind == 0u) {
            best = slot_one;
        } else {
            if (lt64(slot_zero[d].generation, slot_one.generation)) {
                best = slot_one;
            } else {
                best = slot_zero[d];
            }
        }
        if (!chosen_valid) {
            out.cfg = best;
            chosen_valid = true;
        } else if (!(out.cfg.fsid.x == best.fsid.x && out.cfg.fsid.y == best.fsid.y && out.cfg.fsid.z == best.fsid.z && out.cfg.fsid.w == best.fsid.w)
            || out.cfg.device_count != best.device_count) {
            out.failure = FAILURE_POINT_SYSTEM_CONFIGURATIONS_DISAGREE;
            return out;
        }
    }
    if (chosen_valid) {
        out.ok = true;
        return out;
    }
    // recovery::choose_system_configuration_naming_devices_without_a_mountable_slot 最后那个 match：有一槽 incompat 不认识报它，
    // 否则有盘两槽都废报没有有效系统配置，两样都没有只在池里一块盘都没有时
    if (first_incompat) {
        out.failure = FAILURE_POINT_SYSTEM_CONFIGURATION_INCOMPAT_BITS_NOT_RECOGNIZED;
    } else if (first_no_valid) {
        out.failure = FAILURE_POINT_NO_VALID_SYSTEM_CONFIGURATION;
    } else {
        out.failure = FAILURE_POINT_NO_DEVICE_IN_THE_POOL;
    }
    return out;
}
// recovery::mountable_slot_or_refusal：整池拒的槽里每区槽数 S 越界（parse_system_configuration 的拒收码 1）报 RootRingSlotsPerRegionOutOfRange，
// 别的拒收码是读者不收的池级值（system_configuration_values_this_reader_accepts）
fn failure_point_of_a_refused_system_configuration(refused: SysConfig) -> u32 {
    if (refused.refusal == 1u) {
        return FAILURE_POINT_ROOT_RING_SLOTS_PER_REGION_OUT_OF_RANGE;
    }
    return FAILURE_POINT_SYSTEM_CONFIGURATION_VALUE_REFUSED;
}

// ───────────────────────── 根 ─────────────────────────

struct RootFields {
    valid: bool,
    instance: u32,
    txg: vec2<u32>,
    watermark: vec2<u32>,
    floor: vec2<u32>,
    unmount: bool,
    tree_table: Ptr,
    instance_table: Ptr,
    mapping_root: Ptr,
    allocation_root: Ptr,
};
fn zero_pointer() -> Ptr {
    var p: Ptr;
    p.all_zero = true;
    p.head_ok = true;
    p.mac_nonce_zero = true;
    p.extent_offset_zero = true;
    p.birth_tree = vec2<u32>(0u, 0u);
    p.birth_txg = vec2<u32>(0u, 0u);
    p.device0 = 0u;
    p.slot0 = vec2<u32>(0u, 0u);
    p.sum0 = 0u;
    p.device1 = 0u;
    p.slot1 = vec2<u32>(0u, 0u);
    p.sum1 = 0u;
    p.instance = 0u;
    p.tail = vec2<u32>(0u, 0u);
    return p;
}
fn invalid_root() -> RootFields {
    var r: RootFields;
    r.valid = false;
    r.instance = 0u;
    r.txg = vec2<u32>(0u, 0u);
    r.watermark = vec2<u32>(0u, 0u);
    r.floor = vec2<u32>(0u, 0u);
    r.unmount = false;
    r.tree_table = zero_pointer();
    r.instance_table = zero_pointer();
    r.mapping_root = zero_pointer();
    r.allocation_root = zero_pointer();
    return r;
}
fn fsid_matches(base: u32, o: u32, fsid: vec4<u32>) -> bool {
    return rd32(base, o) == fsid.x && rd32(base, o + 4u) == fsid.y && rd32(base, o + 8u) == fsid.z && rd32(base, o + 12u) == fsid.w;
}
fn parse_root(s: Slice, fsid: vec4<u32>) -> RootFields {
    if (s.version == NONE || s.length < ROOT_RECORD_BYTES) {
        return invalid_root();
    }
    let base = s.base;
    if (rd32(base, 0u) != MAGIC_SFSR || s.c0 != 1u || !fsid_matches(base, 4u, fsid)) {
        return invalid_root();
    }
    let flags = rd32(base, 20u);
    if ((flags & 0xFFFFFFFEu) != 0u) {
        return invalid_root();
    }
    var r: RootFields;
    r.valid = true;
    r.unmount = (flags & 1u) != 0u;
    r.instance = rd32(base, 24u);
    r.txg = rd64(base, 28u);
    r.tree_table = parse_node_pointer(base, 36u);
    r.watermark = rd64(base, 122u);
    r.floor = rd64(base, 130u);
    r.instance_table = parse_node_pointer(base, 170u);
    r.mapping_root = parse_node_pointer(base, 256u);
    r.allocation_root = parse_node_pointer(base, 342u);
    return r;
}
// 根记录里一条指针的头部是第一版的取值：pointer.rs 的 PointerHeadAsRead::judged 判的五段全过——MAC 16 与 nonce 12 全 0、算法类型 0、
// 压缩算法码 0、压后长度 0（Ptr 的 head_ok），extent 偏移 0（Ptr 的 extent_offset_zero）
fn root_record_pointer_head_is_the_first_version(pointer: Ptr) -> bool {
    return pointer.head_ok && pointer.extent_offset_zero;
}
// root_record.rs 的 parse_slot：树表、实例表、中央映射树、分配记录树四条指针都经
// NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from 读，任一条头部有一段不是第一版的取值，这条根记录损坏、
// 与自证不过同一个处置，择根时不可择。只在恢复择根时判：池级 checker 的根环读法（image::valid_roots，走读内核 checker_collect_roots）不判它，
// 那里红 I-2.4 的是走读跟随这几条指针的那几处
fn root_record_pointer_heads_are_all_the_first_version(r: RootFields) -> bool {
    return root_record_pointer_head_is_the_first_version(r.tree_table)
        && root_record_pointer_head_is_the_first_version(r.instance_table)
        && root_record_pointer_head_is_the_first_version(r.mapping_root)
        && root_record_pointer_head_is_the_first_version(r.allocation_root);
}
fn region_device(cfg: SysConfig, region: u32) -> u32 {
    if (region == 0u) {
        return cfg.region0;
    }
    if (region == 1u) {
        return cfg.region1;
    }
    return cfg.region2;
}
struct RootChoice {
    found: bool,
    root: RootFields,
};
// 三个区域全部槽逐个验自证（连同四条指针头部的五段），取 (txg, 实例) 最大的；每条自证过的根记进草稿（版本、区域）
// 读出来的根住在 (region, slot) 合不合落点公式（root_ring::root_sits_in_the_ring_slot_the_formula_gives，C743）：第 0 代根（txg 0）
// 照 mkfs 种根的槽另认（三个区域各自的槽 0），别的根只住区域 txg mod 3、槽 (txg div 3) mod S；不合的照这一槽读坏记（错位读）
fn root_sits_in_the_ring_slot_the_formula_gives(txg: vec2<u32>, region: u32, slot: u32, slots_per_region: u32) -> bool {
    if (is_zero64(txg)) {
        return slot == 0u;
    }
    let by_region = divide_by_a_divisor_below_65536(txg, 3u);
    let by_slot = divide_by_a_divisor_below_65536(by_region.quotient, slots_per_region);
    return by_region.remainder == region && by_slot.remainder == slot;
}

fn choose_root(cfg: SysConfig) -> RootChoice {
    var out: RootChoice;
    out.found = false;
    out.root = invalid_root();
    root_count = 0u;
    for (var region = 0u; region < 3u; region = region + 1u) {
        let device = region_device(cfg, region);
        for (var slot = 0u; slot < cfg.slots_per_region; slot = slot + 1u) {
            let offset = u64_of(ROOT_RING_BASE_BYTES + region * ROOT_RING_REGION_STRIDE + slot * cfg.spacing);
            let s = read_at(device, offset, cfg.physical_block_size);
            let candidate = parse_root(s, cfg.fsid);
            if (!candidate.valid || !root_record_pointer_heads_are_all_the_first_version(candidate)) {
                continue;
            }
            if (!root_sits_in_the_ring_slot_the_formula_gives(candidate.txg, region, slot, cfg.slots_per_region)) {
                continue;
            }
            if (root_count < MAX_ROOTS) {
                sws(R_ROOTS + root_count * 2u, s.version);
                sws(R_ROOTS + root_count * 2u + 1u, region);
                root_count = root_count + 1u;
            } else {
                undecidable = UNDECIDABLE_TOO_MANY_ROOTS;
            }
            if (!out.found || lt64(out.root.txg, candidate.txg)
                || (eq64(out.root.txg, candidate.txg) && out.root.instance < candidate.instance)) {
                out.root = candidate;
                out.found = true;
            }
        }
    }
    return out;
}

// ───────────────────────── journal ─────────────────────────

struct Rec {
    valid: bool,
    base: u32,
    instance: u32,
    counter: vec2<u32>,
    txg: vec2<u32>,
    transaction: vec2<u32>,
    commit: bool,
    ordinal: u32,
    last: bool,
    back_chain: u32,
    named_count: u32,
};
fn invalid_record() -> Rec {
    var r: Rec;
    r.valid = false;
    r.base = 0u;
    r.instance = 0u;
    r.counter = vec2<u32>(0u, 0u);
    r.txg = vec2<u32>(0u, 0u);
    r.transaction = vec2<u32>(0u, 0u);
    r.commit = false;
    r.ordinal = 0u;
    r.last = false;
    r.back_chain = 0u;
    r.named_count = 0u;
    return r;
}
// 记录新根段里一条指针的头（偏移 o 起 34 字节）是第一版的取值：head_ok 判的几段，加 extent 偏移（指针偏移 32 的 2 字节）为 0
fn record_pointer_head_is_the_first_version(base: u32, o: u32) -> bool {
    return parse_node_pointer(base, o).head_ok && rd16(base, o + 32u) == 0u;
}
// journal::JournalRecord::parse：magic、类型、整条校验和、标志、长度、提交标记、序号、新根段四条指针头部、卸载记号位 0 之外为 0、fsid、载荷校验和
fn parse_record(s: Slice, fsid_low: vec2<u32>) -> Rec {
    if (s.version == NONE || s.length < JOURNAL_RECORD_BYTES) {
        return invalid_record();
    }
    let base = s.base;
    if (rd32(base, 0u) != MAGIC_SFSJ || s.c0 != 1u || rd16(base, 4u) != 1u) {
        return invalid_record();
    }
    let flags = rd8(base, 7u);
    if (flags > 1u) {
        return invalid_record();
    }
    if (rd32(base, 8u) != JOURNAL_RECORD_BYTES) {
        return invalid_record();
    }
    let commit = rd8(base, 86u);
    if (commit > 1u) {
        return invalid_record();
    }
    let ordinal = rd32(base, 87u);
    if (ordinal == 0u) {
        return invalid_record();
    }
    // 新根段四条指针各按 NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from 判头：
    // MAC / nonce、算法类型、压缩码、压缩长度、extent 偏移都是第一版的取值
    if (!record_pointer_head_is_the_first_version(base, 99u) || !record_pointer_head_is_the_first_version(base, 185u)) {
        return invalid_record();
    }
    if (!record_pointer_head_is_the_first_version(base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER)
        || !record_pointer_head_is_the_first_version(base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER)) {
        return invalid_record();
    }
    // UnmountMarker::of_unmount_marker_field_of_a_new_root_segment：位 0 之外有位为 1 当这条记录损坏
    if ((rd32(base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_UNMOUNT_MARKER) & 0xFFFFFFFEu) != 0u) {
        return invalid_record();
    }
    if (!eq64(rd64(base, JOURNAL_FILESYSTEM_IDENTIFIER_OFFSET), fsid_low)) {
        return invalid_record();
    }
    if (s.c1 != 1u) {
        return invalid_record();
    }
    var r: Rec;
    r.valid = true;
    r.base = base;
    r.instance = rd32(base, 16u);
    r.counter = rd48(base, 20u);
    r.txg = rd64(base, 26u);
    r.transaction = rd64(base, 78u);
    r.commit = commit == 1u;
    r.ordinal = ordinal;
    r.last = flags == 1u;
    r.back_chain = rd32(base, 91u);
    r.named_count = rd32(base, 12u);
    return r;
}
fn record_at(index: u32) -> Rec {
    let version = sw(R_RECORDS + index * RECORD_WORDS);
    var r: Rec;
    r.valid = true;
    r.base = B[T_VERSION_BYTES] + tab(T_VERSIONS, version * VERSION_WORDS + 1u);
    r.instance = sw(R_RECORDS + index * RECORD_WORDS + 1u);
    r.counter = vec2<u32>(sw(R_RECORDS + index * RECORD_WORDS + 2u), sw(R_RECORDS + index * RECORD_WORDS + 3u));
    r.txg = vec2<u32>(sw(R_RECORDS + index * RECORD_WORDS + 4u), sw(R_RECORDS + index * RECORD_WORDS + 5u));
    r.transaction = rd64(r.base, 78u);
    r.commit = rd8(r.base, 86u) == 1u;
    r.ordinal = rd32(r.base, 87u);
    r.last = (rd8(r.base, 7u) & 1u) == 1u;
    r.back_chain = rd32(r.base, 91u);
    r.named_count = rd32(r.base, 12u);
    return r;
}
fn record_version_at(index: u32) -> u32 {
    return sw(R_RECORDS + index * RECORD_WORDS);
}
fn record_key_less(instance_a: u32, counter_a: vec2<u32>, instance_b: u32, counter_b: vec2<u32>) -> bool {
    return instance_a < instance_b || (instance_a == instance_b && lt64(counter_a, counter_b));
}
// 记录表按 (实例代号, 计数器) 严格升序（只经 insert_record 写）：第一个键不小于 (instance, counter) 的位置，都小交回 record_count。
// 二分：每一轮 [low, high) 之前的键都小于它、从 high 起的都不小于它，至多 ⌈log2(MAX_RECORDS + 1)⌉ = 13 轮
fn record_position_not_below(instance: u32, counter: vec2<u32>) -> u32 {
    var low = 0u;
    var high = record_count;
    while (low < high) {
        let middle = (low + high) / 2u;
        let middle_instance = sw(R_RECORDS + middle * RECORD_WORDS + 1u);
        let middle_counter = vec2<u32>(sw(R_RECORDS + middle * RECORD_WORDS + 2u), sw(R_RECORDS + middle * RECORD_WORDS + 3u));
        if (record_key_less(middle_instance, middle_counter, instance, counter)) {
            low = middle + 1u;
        } else {
            high = middle;
        }
    }
    return low;
}
// 第 position 条的键是不是 (instance, counter)：position 可以是 record_count（表尾之后，不是）
fn record_key_at_is(position: u32, instance: u32, counter: vec2<u32>) -> bool {
    return position < record_count && sw(R_RECORDS + position * RECORD_WORDS + 1u) == instance
        && sw(R_RECORDS + position * RECORD_WORDS + 2u) == counter.x && sw(R_RECORDS + position * RECORD_WORDS + 3u) == counter.y;
}
// 按 (实例代号, 计数器) 排序插入；已有同键的不换（BTreeMap::or_insert：先读到的留下）。插入点用二分找（环按偏移收、计数器升序到达时就是表尾，不挪）
fn insert_record(version: u32, r: Rec) {
    let position = record_position_not_below(r.instance, r.counter);
    if (record_key_at_is(position, r.instance, r.counter)) {
        return;
    }
    if (record_count >= MAX_RECORDS) {
        undecidable = UNDECIDABLE_TOO_MANY_RECORDS;
        return;
    }
    var i = record_count;
    while (i > position) {
        for (var k = 0u; k < RECORD_WORDS; k = k + 1u) {
            sws(R_RECORDS + i * RECORD_WORDS + k, sw(R_RECORDS + (i - 1u) * RECORD_WORDS + k));
        }
        i = i - 1u;
    }
    sws(R_RECORDS + position * RECORD_WORDS, version);
    sws(R_RECORDS + position * RECORD_WORDS + 1u, r.instance);
    sws(R_RECORDS + position * RECORD_WORDS + 2u, r.counter.x);
    sws(R_RECORDS + position * RECORD_WORDS + 3u, r.counter.y);
    sws(R_RECORDS + position * RECORD_WORDS + 4u, r.txg.x);
    sws(R_RECORDS + position * RECORD_WORDS + 5u, r.txg.y);
    record_count = record_count + 1u;
}
// recovery::scan_journal：每块盘按次序、环里每个记录槽按偏移升序
fn scan_journal(cfg: SysConfig) {
    record_count = 0u;
    let fsid_low = vec2<u32>(cfg.fsid.x, cfg.fsid.y);
    let device_count = hdr(3u);
    let location_count = hdr(6u);
    for (var d = 0u; d < device_count; d = d + 1u) {
        let device = hdr(12u + d);
        for (var l = 0u; l < location_count; l = l + 1u) {
            if (location_word(l, 0u) != device || location_word(l, 4u) != LOCATION_JOURNAL_RECORD) {
                continue;
            }
            let s = read_location(l);
            let r = parse_record(s, fsid_low);
            if (r.valid) {
                insert_record(s.version, r);
            }
        }
    }
}
fn find_record(instance: u32, counter: vec2<u32>) -> u32 {
    let position = record_position_not_below(instance, counter);
    if (record_key_at_is(position, instance, counter)) {
        return position;
    }
    return NONE;
}
// 两份记录解出来相等（磁盘上另一份与扫环时收下的那一条比）：比解析读的那几段
fn records_parse_equal(base_a: u32, base_b: u32) -> bool {
    if (base_a == base_b) {
        return true;
    }
    if (!bytes_equal(base_a, 4u, base_b, 4u, 34u)) {
        return false;
    }
    if (!bytes_equal(base_a, 78u, base_b, 78u, 21u)) {
        return false;
    }
    if (!bytes_equal(base_a, JOURNAL_NEW_ROOT_SEGMENT_OFFSET, base_b, JOURNAL_NEW_ROOT_SEGMENT_OFFSET, JOURNAL_NEW_ROOT_SEGMENT_BYTES + 8u)) {
        return false;
    }
    let named = rd32(base_a, 12u);
    return bytes_equal(base_a, JOURNAL_HEADER_BYTES, base_b, JOURNAL_HEADER_BYTES, named * JOURNAL_NAMED_ENTRY_BYTES);
}
// 反向链判定：0 成立、1 断、2 前一条不在盘上
fn judge_back_chain(r: Rec, ring_bytes: vec2<u32>) -> u32 {
    if (is_zero64(r.counter) || eq64(r.counter, u64_of(1u))) {
        return 2u;
    }
    let previous_counter = sub64(r.counter, u64_of(1u));
    let previous_index = find_record(r.instance, previous_counter);
    if (previous_index == NONE) {
        return 2u;
    }
    let previous_version = record_version_at(previous_index);
    let previous_base = B[T_VERSION_BYTES] + tab(T_VERSIONS, previous_version * VERSION_WORDS + 1u);
    let ring_slots = shr64(ring_bytes, 12u);
    let slot_in_ring = divmod64(sub64(previous_counter, u64_of(1u)), ring_slots).remainder;
    let offset = add64(u64_of(JOURNAL_RING_START_BYTES), shl64(slot_in_ring, 12u));
    let device_count = hdr(3u);
    let fsid_low = rd64(previous_base, JOURNAL_FILESYSTEM_IDENTIFIER_OFFSET);
    for (var d = 0u; d < device_count; d = d + 1u) {
        let s = read_at(hdr(12u + d), offset, JOURNAL_RECORD_BYTES);
        if (s.version == NONE) {
            continue;
        }
        let candidate = parse_record(s, fsid_low);
        if (!candidate.valid || !records_parse_equal(s.base, previous_base)) {
            continue;
        }
        if (s.c2 == r.back_chain) {
            return 0u;
        }
        return 1u;
    }
    return 2u;
}
fn readable_record_of_the_same_publish_follows(r: Rec, index: u32) -> bool {
    var i = index + 1u;
    while (i < record_count) {
        let instance = sw(R_RECORDS + i * RECORD_WORDS + 1u);
        if (instance != r.instance) {
            break;
        }
        let txg = vec2<u32>(sw(R_RECORDS + i * RECORD_WORDS + 4u), sw(R_RECORDS + i * RECORD_WORDS + 5u));
        if (lt64(r.txg, txg)) {
            break;
        }
        if (eq64(txg, r.txg)) {
            return true;
        }
        i = i + 1u;
    }
    return false;
}
fn named_units_verified(r: Rec) -> bool {
    for (var i = 0u; i < r.named_count; i = i + 1u) {
        let o = JOURNAL_HEADER_BYTES + i * JOURNAL_NAMED_ENTRY_BYTES;
        let unit_class = rd8(r.base, o + 28u);
        var unit_bytes = 0u;
        if (unit_class == 1u || unit_class == 3u) {
            unit_bytes = DATA_UNIT_BYTES;
        } else if (unit_class == 2u) {
            unit_bytes = NODE_BYTES;
        } else {
            return false;
        }
        let s0 = read_at(rd32(r.base, o), slot_to_bytes(rd48(r.base, o + 4u)), unit_bytes);
        if (s0.version == NONE || !slice_crc_matches(s0, unit_bytes, rd32(r.base, o + 10u))) {
            return false;
        }
        let s1 = read_at(rd32(r.base, o + 14u), slot_to_bytes(rd48(r.base, o + 18u)), unit_bytes);
        if (s1.version == NONE || !slice_crc_matches(s1, unit_bytes, rd32(r.base, o + 24u))) {
            return false;
        }
    }
    return true;
}
struct Replay {
    ok: bool,
    failure: u32,
    root: RootFields,
};
// recovery::replay_journal（Consult：验点名单元）
fn replay_journal(chosen: RootFields, ring_bytes: vec2<u32>) -> Replay {
    var out: Replay;
    out.ok = false;
    out.failure = 0u;
    out.root = chosen;
    // 锚点：所选根那次发布里带末条标志的那一条
    var anchor_count = 0u;
    var anchor_counter = vec2<u32>(0u, 0u);
    for (var i = 0u; i < record_count; i = i + 1u) {
        let r = record_at(i);
        if (r.instance == chosen.instance && eq64(r.txg, chosen.txg) && r.last) {
            anchor_count = anchor_count + 1u;
            anchor_counter = r.counter;
        }
    }
    if (anchor_count > 1u) {
        out.failure = FAILURE_POINT_ROOT_PUBLISH_CARRIES_MORE_THAN_ONE_LAST_RECORD_FLAG_WHOSE_ANCHOR_IS_UNDECIDED;
        return out;
    }
    let has_anchor = anchor_count == 1u;
    // 所选根的 txg 没有后继（singlefs_replay_prefix 的 ChosenRootCheckpointTxgHasNoSuccessor）：recovery::replay_journal 报 UnitMalformed
    if (chosen.txg.x == NONE && chosen.txg.y == NONE) {
        out.failure = FAILURE_POINT_UNIT_MALFORMED;
        return out;
    }
    let chain_start_txg = add64(chosen.txg, u64_of(1u));
    var expected_counter = add64(anchor_counter, u64_of(1u));
    var have_expected = has_anchor;
    let limit64 = divmod64(shr64(ring_bytes, 12u), u64_of(3u)).quotient;
    var limit = limit64.x;
    if (limit64.y != 0u) {
        limit = NONE;
    }
    var taken = 0u;
    var open_count = 0u;
    var previous = invalid_record();
    var rebuilt = chosen;
    replay_stop_reason = 0u;
    for (var i = 0u; i < record_count; i = i + 1u) {
        let r = record_at(i);
        if (r.instance != chosen.instance || !lt64(chosen.txg, r.txg)) {
            continue;
        }
        if (taken >= limit) {
            replay_stop_reason = 1u;
            break;
        }
        taken = taken + 1u;
        if (have_expected) {
            if (!eq64(r.counter, expected_counter)) {
                replay_stop_reason = 2u;
                break;
            }
        } else if (!eq64(r.txg, chain_start_txg) || r.ordinal != 1u) {
            replay_stop_reason = 3u;
            break;
        }
        if (judge_back_chain(r, ring_bytes) == 1u) {
            replay_stop_reason = 4u;
            break;
        }
        if (open_count == 0u && r.ordinal != 1u) {
            replay_stop_reason = 5u;
            break;
        }
        if (open_count > 0u) {
            if (!eq64(r.txg, previous.txg)) {
                replay_stop_reason = 6u;
                break;
            }
            if (r.ordinal != previous.ordinal + 1u) {
                replay_stop_reason = 7u;
                break;
            }
            if (!previous.commit && !eq64(r.transaction, previous.transaction)) {
                replay_stop_reason = 8u;
                break;
            }
        }
        expected_counter = add64(r.counter, u64_of(1u));
        have_expected = true;
        if (!r.commit && r.last) {
            replay_stop_reason = 9u;
            break;
        }
        if (r.last && readable_record_of_the_same_publish_follows(r, i)) {
            replay_stop_reason = 10u;
            break;
        }
        if (!named_units_verified(r)) {
            replay_stop_reason = 11u;
            break;
        }
        open_count = open_count + 1u;
        previous = r;
        if (!r.last) {
            continue;
        }
        open_count = 0u;
        // 施加末条：所选根的七个字段换成记录新根段里的（recovery::replay_journal，已定项 15，用户 2026-10-02「回到甲」）；fsid 照所选根
        var next = rebuilt;
        next.valid = true;
        next.unmount = (rd32(r.base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_UNMOUNT_MARKER) & 1u) != 0u;
        next.instance = r.instance;
        next.txg = r.txg;
        next.tree_table = parse_node_pointer(r.base, 99u);
        next.watermark = rd64(r.base, 271u);
        next.floor = rd64(r.base, 279u);
        next.instance_table = parse_node_pointer(r.base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER);
        next.mapping_root = parse_node_pointer(r.base, 185u);
        next.allocation_root = parse_node_pointer(r.base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER);
        rebuilt = next;
        replay_stop_reason = 12u;
    }
    out.ok = true;
    out.root = rebuilt;
    return out;
}

// ───────────────────────── 单元解析 ─────────────────────────

struct Node {
    ok: bool,
    base: u32,
    tree: vec2<u32>,
    level: u32,
    key_width: u32,
    smallest_offset: u32,
    largest_offset: u32,
    birth_txg: vec2<u32>,
    fsid: vec2<u32>,
    instance: u32,
    birth_sequence: u32,
    entry_count: u32,
    entry_width: u32,
    entries_start: u32,
};
fn bad_node() -> Node {
    var n: Node;
    n.ok = false;
    n.base = 0u;
    n.tree = vec2<u32>(0u, 0u);
    n.level = 0u;
    n.key_width = 0u;
    n.smallest_offset = 0u;
    n.largest_offset = 0u;
    n.birth_txg = vec2<u32>(0u, 0u);
    n.fsid = vec2<u32>(0u, 0u);
    n.instance = 0u;
    n.birth_sequence = 0u;
    n.entry_count = 0u;
    n.entry_width = 0u;
    n.entries_start = 0u;
    return n;
}
// 码 2 节点头里条目那一段（unit::parse_index_node 的头布局）：key 宽在偏移 51，头到 86 + 2 × key 宽为止，条数与条目宽是头的最后 4 字节，
// 条目从头之后 29 字节起。parse_index_node 与中央映射叶表（mapping_leaf_at）都照这一处读
struct IndexNodeEntries {
    key_width: u32,
    entry_count: u32,
    entry_width: u32,
    entries_start: u32,
};
fn index_node_entries_of_the_header(base: u32) -> IndexNodeEntries {
    let key_width = rd8(base, 51u);
    let header_end = 86u + 2u * key_width;
    return IndexNodeEntries(key_width, rd16(base, header_end - 4u), rd16(base, header_end - 2u), header_end + 29u);
}
// unit::parse_index_node（读 16 KiB）
fn parse_index_node(s: Slice) -> Node {
    if (s.version == NONE || s.length < NODE_BYTES) {
        return bad_node();
    }
    let base = s.base;
    if (rd32(base, 0u) != MAGIC_SFSU || rd16(base, 4u) != 1u || rd8(base, 6u) != 2u || rd8(base, 7u) != 0u) {
        return bad_node();
    }
    if ((s.c2 & 11u) != 11u) {
        return bad_node();
    }
    let entries = index_node_entries_of_the_header(base);
    let key_width = entries.key_width;
    let declared_length = rd16(base, 8u);
    let entry_count = entries.entry_count;
    let entry_width = entries.entry_width;
    if (entry_count * entry_width != declared_length) {
        return bad_node();
    }
    if (entry_width == 0u && entry_count != 0u) {
        return bad_node();
    }
    if (entry_width < key_width) {
        return bad_node();
    }
    let entries_start = entries.entries_start;
    if (entries_start + declared_length > NODE_BYTES) {
        return bad_node();
    }
    var n: Node;
    n.ok = true;
    n.base = base;
    n.tree = rd64(base, 42u);
    n.level = rd8(base, 50u);
    n.key_width = key_width;
    n.smallest_offset = 52u;
    n.largest_offset = 52u + key_width;
    n.birth_txg = rd64(base, 52u + 2u * key_width);
    n.fsid = rd64(base, 60u + 2u * key_width);
    n.instance = rd32(base, 68u + 2u * key_width);
    n.birth_sequence = rd32(base, 72u + 2u * key_width);
    n.entry_count = entry_count;
    n.entry_width = entry_width;
    n.entries_start = entries_start;
    return n;
}
fn entry_offset(n: Node, index: u32) -> u32 {
    return n.entries_start + index * n.entry_width;
}
struct Packed {
    ok: bool,
    base: u32,
    birth_tree: vec2<u32>,
    record_type: u32,
    container: vec2<u32>,
    container_birth: vec2<u32>,
    record_count: u32,
    record_width: u32,
    birth_txg: vec2<u32>,
    fsid: vec2<u32>,
    instance: u32,
    transaction: vec2<u32>,
    birth_sequence: u32,
};
fn bad_packed() -> Packed {
    var p: Packed;
    p.ok = false;
    p.base = 0u;
    p.birth_tree = vec2<u32>(0u, 0u);
    p.record_type = 0u;
    p.container = vec2<u32>(0u, 0u);
    p.container_birth = vec2<u32>(0u, 0u);
    p.record_count = 0u;
    p.record_width = 0u;
    p.birth_txg = vec2<u32>(0u, 0u);
    p.fsid = vec2<u32>(0u, 0u);
    p.instance = 0u;
    p.transaction = vec2<u32>(0u, 0u);
    p.birth_sequence = 0u;
    return p;
}
// unit::parse_packed_unit（读 32 KiB）
fn parse_packed_unit(s: Slice) -> Packed {
    if (s.version == NONE || s.length < DATA_UNIT_BYTES) {
        return bad_packed();
    }
    let base = s.base;
    if (rd32(base, 0u) != MAGIC_SFSU || rd16(base, 4u) != 1u || rd8(base, 6u) != 3u || rd8(base, 7u) != 0u) {
        return bad_packed();
    }
    if ((s.c2 & 11u) != 11u || rd8(base, 42u) != 3u) {
        return bad_packed();
    }
    let declared_length = rd16(base, 8u);
    let record_count = rd16(base, 69u);
    let record_width = rd16(base, 71u);
    if (record_count * record_width != declared_length) {
        return bad_packed();
    }
    if (record_width == 0u && record_count != 0u) {
        return bad_packed();
    }
    if (136u + declared_length > DATA_UNIT_BYTES) {
        return bad_packed();
    }
    var p: Packed;
    p.ok = true;
    p.base = base;
    p.birth_tree = rd64(base, 43u);
    p.record_type = rd16(base, 51u);
    p.container = rd64(base, 53u);
    p.container_birth = rd64(base, 61u);
    p.record_count = record_count;
    p.record_width = record_width;
    p.birth_txg = rd64(base, 73u);
    p.fsid = rd64(base, 81u);
    p.instance = rd32(base, 93u);
    p.transaction = rd48(base, 97u);
    p.birth_sequence = rd32(base, 103u);
    return p;
}
fn packed_record_offset(p: Packed, index: u32) -> u32 {
    return 136u + index * p.record_width;
}
struct DataUnit {
    ok: bool,
    base: u32,
    tree: vec2<u32>,
    object: vec2<u32>,
    object_birth: vec2<u32>,
    anchor_offset: vec2<u32>,
    birth_txg: vec2<u32>,
    fsid: vec2<u32>,
    instance: u32,
    transaction: vec2<u32>,
    declared_length: u32,
    padding_zero: bool,
    payload_crc: u32,
};
fn bad_data_unit() -> DataUnit {
    var d: DataUnit;
    d.ok = false;
    d.base = 0u;
    d.tree = vec2<u32>(0u, 0u);
    d.object = vec2<u32>(0u, 0u);
    d.object_birth = vec2<u32>(0u, 0u);
    d.anchor_offset = vec2<u32>(0u, 0u);
    d.birth_txg = vec2<u32>(0u, 0u);
    d.fsid = vec2<u32>(0u, 0u);
    d.instance = 0u;
    d.transaction = vec2<u32>(0u, 0u);
    d.declared_length = 0u;
    d.padding_zero = false;
    d.payload_crc = 0u;
    return d;
}
// unit::parse_data_unit（读 32 KiB）
fn parse_data_unit(s: Slice) -> DataUnit {
    if (s.version == NONE || s.length < DATA_UNIT_BYTES) {
        return bad_data_unit();
    }
    let base = s.base;
    if (rd32(base, 0u) != MAGIC_SFSU || rd16(base, 4u) != 1u || rd8(base, 6u) != 1u || rd8(base, 7u) != 0u) {
        return bad_data_unit();
    }
    if ((s.c2 & 11u) != 11u || rd8(base, 42u) != 1u) {
        return bad_data_unit();
    }
    let declared_length = rd16(base, 8u);
    if (134u + declared_length > DATA_UNIT_BYTES) {
        return bad_data_unit();
    }
    var d: DataUnit;
    d.ok = true;
    d.base = base;
    d.tree = rd64(base, 43u);
    d.object = rd64(base, 51u);
    d.object_birth = rd64(base, 59u);
    d.anchor_offset = rd64(base, 67u);
    d.birth_txg = rd64(base, 75u);
    d.fsid = rd64(base, 83u);
    d.instance = rd32(base, 91u);
    d.transaction = rd48(base, 95u);
    d.declared_length = declared_length;
    d.padding_zero = (s.c2 & 16u) != 0u;
    d.payload_crc = rd32(base, 101u);
    return d;
}

// 两个多项式在 CRC-32C 的生成多项式下相乘取余（反射位序，位 31 是 x 的 0 次）：逐位看乘数，被乘数每过一位乘一次 x。循环恰好 32 圈、没有提前出口
fn crc32c_product_modulo_the_polynomial(multiplier: u32, multiplicand: u32) -> u32 {
    var product = 0u;
    var multiplicand_times_x_to_the_bit = multiplicand;
    for (var bit = 0u; bit < 32u; bit = bit + 1u) {
        if ((multiplier & (0x80000000u >> bit)) != 0u) {
            product = product ^ multiplicand_times_x_to_the_bit;
        }
        let shifted_out_a_one = (multiplicand_times_x_to_the_bit & 1u) != 0u;
        multiplicand_times_x_to_the_bit = multiplicand_times_x_to_the_bit >> 1u;
        if (shifted_out_a_one) {
            multiplicand_times_x_to_the_bit = multiplicand_times_x_to_the_bit ^ CRC32C_REFLECTED_POLYNOMIAL;
        }
    }
    return product;
}
// byte_count 个 0 字节的 CRC-32C（初值与末尾异或都是全 1）：寄存器读进 n 个 0 字节等于乘 x 的 8n 次再取余，x 的 8n 次按 n 的二进制位平方连乘，
// 不逐字节走（洞里一个单元的载荷区三万多字节，每个洞的位置逐字节走太慢）。循环圈数是 byte_count 的二进制位数（至多 32）
fn crc32c_of_zero_bytes(byte_count: u32) -> u32 {
    var x_to_the_eight_times_this_power_of_two = 0x00800000u;
    var x_to_the_eight_times_the_bytes_seen = 0x80000000u;
    var bytes_left = byte_count;
    loop {
        if (bytes_left == 0u) {
            break;
        }
        if ((bytes_left & 1u) != 0u) {
            x_to_the_eight_times_the_bytes_seen = crc32c_product_modulo_the_polynomial(x_to_the_eight_times_this_power_of_two, x_to_the_eight_times_the_bytes_seen);
        }
        x_to_the_eight_times_this_power_of_two = crc32c_product_modulo_the_polynomial(x_to_the_eight_times_this_power_of_two, x_to_the_eight_times_this_power_of_two);
        bytes_left = bytes_left >> 1u;
    }
    return ~crc32c_product_modulo_the_polynomial(x_to_the_eight_times_the_bytes_seen, 0xFFFFFFFFu);
}

// ───────────────────────── 中央映射 ─────────────────────────

struct Lookup {
    found: bool,
    failed: bool,
    p: Ptr,
};
// 中央映射叶表（R_MAPPING_LEAVES）第 leaf_index 行那片叶：字节基与头里条目那一段
struct MappingLeaf {
    base: u32,
    entries: IndexNodeEntries,
};
fn mapping_leaf_at(leaf_index: u32) -> MappingLeaf {
    let base = sw(R_MAPPING_LEAVES + leaf_index);
    return MappingLeaf(base, index_node_entries_of_the_header(base));
}
// 在映射叶条目里找一个 27 字节 key（逐片叶、片里逐条，次序同读树时记叶的次序）：同 key 多条取最后一条；条目窄于 55 判红
// （一片叶里条目同宽：按片判与逐条判、遇到第一条窄的就判红相同，读树时零条目的叶不收）
fn mapping_lookup(key_base: u32, key_offset: u32, key_class: u32, key_tail: vec2<u32>, key_tail_is_transaction: bool) -> Lookup {
    var out: Lookup;
    out.found = false;
    out.failed = false;
    out.p = zero_pointer();
    for (var leaf_index = 0u; leaf_index < mapping_leaf_count; leaf_index = leaf_index + 1u) {
        let leaf = mapping_leaf_at(leaf_index);
        if (leaf.entries.entry_width < 55u) {
            out.failed = true;
            return out;
        }
        let base = leaf.base;
        for (var entry_index = 0u; entry_index < leaf.entries.entry_count; entry_index = entry_index + 1u) {
            let o = leaf.entries.entries_start + entry_index * leaf.entries.entry_width;
            // key：类标签 1 + 出生树 8 + 出生 txg 8 + 实例 4 + 尾段 6
            if (rd8(base, o) != key_class) {
                continue;
            }
            if (!bytes_equal(base, o + 1u, key_base, key_offset + 34u, 16u)) {
                continue;
            }
            if (rd32(base, o + 17u) != rd32(key_base, key_offset + 78u)) {
                continue;
            }
            var tail_matches = false;
            if (key_tail_is_transaction) {
                tail_matches = eq64(rd48(base, o + 21u), key_tail);
            } else {
                tail_matches = rd32(base, o + 21u) == key_tail.x && rd16(base, o + 25u) == 0u;
            }
            if (!tail_matches) {
                continue;
            }
            var p = zero_pointer();
            p.all_zero = false;
            p.device0 = rd32(base, o + 27u);
            p.slot0 = rd48(base, o + 31u);
            p.sum0 = rd32(base, o + 37u);
            p.device1 = rd32(base, o + 41u);
            p.slot1 = rd48(base, o + 45u);
            p.sum1 = rd32(base, o + 51u);
            out.p = p;
            out.found = true;
        }
    }
    return out;
}
struct MappedRead {
    s: Slice,
    failed: bool,
};
// 按位置提示读，读不出经中央映射回退（recovery::read_mapped_tree_node_via_hint_then_central_mapping / read_data_unit_via_hint_then_central_mapping）
// pointer_base / pointer_offset：这条指针在字节里的位置（映射 key 从它的出生树、出生 txg、实例、尾段拼）
fn read_via_hint_then_mapping(p: Ptr, pointer_base: u32, pointer_offset: u32, unit_class: u32, unit_bytes: u32) -> MappedRead {
    var out: MappedRead;
    out.failed = false;
    // 建上一版那一样读者读的每个进映射的单元：映射里有它的 key 时，重建之后它的指针换成映射那两条位置条目，再按 I-2.5 判一遍
    // （transaction::take_the_mapping_positions_over_the_location_hints 之后的 pointers_of_the_rebuilt_version_ascend_by_device）。
    // 查映射本身判红（映射树读不出、条目窄于字段表）的这里不记：那一判在读者里先于这一判报
    if (rebuild_judges_the_pointers_of_the_version && mapping_tree_failure_point == FAILURE_POINT_NONE) {
        let position = mapping_lookup(pointer_base, pointer_offset, unit_class, p.tail, unit_class == 1u);
        if (!position.failed && position.found && position.p.device0 >= position.p.device1) {
            some_mapping_position_of_the_version_does_not_ascend_by_device = true;
        }
    }
    out.s = read_unit_via_locations(p, unit_bytes);
    if (out.s.version != NONE) {
        return out;
    }
    // 要经映射回退：映射树读不出就报它读不出的那一处；映射条目窄于字段表报 EntryWidthDiffersFromItsFieldTable（central_mapping_locations_among_entries）；
    // 映射里没有报 MappingMiss；映射落点也读不出报 MappingStillUnreadable
    out.failed = true;
    if (mapping_tree_failure_point != FAILURE_POINT_NONE) {
        walk_failure_point = mapping_tree_failure_point;
        return out;
    }
    let looked = mapping_lookup(pointer_base, pointer_offset, unit_class, p.tail, unit_class == 1u);
    if (looked.failed) {
        walk_failure_point = FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE;
        return out;
    }
    if (!looked.found) {
        walk_failure_point = FAILURE_POINT_MAPPING_MISS;
        return out;
    }
    out.s = read_unit_via_locations(looked.p, unit_bytes);
    if (out.s.version == NONE) {
        walk_failure_point = FAILURE_POINT_MAPPING_STILL_UNREADABLE;
        return out;
    }
    out.failed = false;
    return out;
}

// ───────────────────────── 走读用的判定 ─────────────────────────

fn root_header_checks(n: Node, tree: vec2<u32>, key_width: u32, p: Ptr, root: RootFields, fsid_low: vec2<u32>) -> bool {
    if (!eq64(n.tree, tree)) {
        return false;
    }
    if (n.key_width != key_width) {
        return false;
    }
    if (lt64(root.txg, n.birth_txg) || n.instance > root.instance) {
        return false;
    }
    if (!eq64(n.fsid, fsid_low)) {
        return false;
    }
    if (n.birth_sequence != p.tail.x) {
        return false;
    }
    return true;
}
// recovery::tree_root_checked_against_its_pointer
// 判不过时记失败点：解不开 UnitMalformed，头与指针、层级 0 的 key 区间对不上 InvariantViolated。层级 ≥ 1 的节点头里的 key 区间是子树覆盖区间
// （D18 已定项 2），由读孩子的一方判（inode 树根读完叶容器判 inode_tree_root_key_range_is_the_subtree_coverage，容器树在 read_container_tree_index_nodes 里判）
fn tree_root_checked(n: Node, tree: vec2<u32>, key_width: u32, p: Ptr, root: RootFields, fsid_low: vec2<u32>) -> bool {
    if (!n.ok) {
        walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
        return false;
    }
    walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
    if (!root_header_checks(n, tree, key_width, p, root, fsid_low)) {
        return false;
    }
    if (n.level == 0u && !key_range_is_tight_over_the_entries(n, key_width)) {
        return false;
    }
    return true;
}
// recovery::key_range_is_tight_over_the_entries：头里的 key 区间是首末两条条目的 key（零条目的节点不判）
fn key_range_is_tight_over_the_entries(n: Node, key_width: u32) -> bool {
    if (n.entry_count == 0u) {
        return true;
    }
    return bytes_equal(n.base, entry_offset(n, 0u), n.base, n.smallest_offset, key_width)
        && bytes_equal(n.base, entry_offset(n, n.entry_count - 1u), n.base, n.largest_offset, key_width);
}
// recovery::inode_tree_root_key_range_is_the_subtree_coverage 的累加那一半：读回来的记录号里最小、最大的两个（have 为 false 是还一条都没有）
struct InodeNumbersReadBack {
    have: bool,
    smallest: vec2<u32>,
    largest: vec2<u32>,
};
fn inode_numbers_read_back_with(range: InodeNumbersReadBack, inode: vec2<u32>) -> InodeNumbersReadBack {
    var out = range;
    if (!range.have) {
        out.have = true;
        out.smallest = inode;
        out.largest = inode;
        return out;
    }
    if (lt64(inode, range.smallest)) { out.smallest = inode; }
    if (lt64(range.largest, inode)) { out.largest = inode; }
    return out;
}
// recovery::inode_tree_root_key_range_is_the_subtree_coverage：inode 树根头里的 key 区间 = [读回来的最小记录号, 最大记录号]；
// 零条目的根与一条记录都没读回来的不判。不成立是 InvariantViolated（调用方照它的读者种类折算）
fn inode_tree_root_key_range_is_the_subtree_coverage(inode_root: Node, read_back: InodeNumbersReadBack) -> bool {
    if (inode_root.entry_count == 0u || !read_back.have) {
        return true;
    }
    return eq64(rd64(inode_root.base, inode_root.smallest_offset), read_back.smallest)
        && eq64(rd64(inode_root.base, inode_root.largest_offset), read_back.largest);
}
// 见过的单元（按第一条位置条目的槽号）：重复交回 false
fn seen_insert(slot: vec2<u32>) -> bool {
    for (var i = 0u; i < seen_count; i = i + 1u) {
        if (sw(R_SEEN + i * 2u) == slot.x && sw(R_SEEN + i * 2u + 1u) == slot.y) {
            return false;
        }
    }
    if (seen_count >= MAX_SEEN) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;
        return false;
    }
    sws(R_SEEN + seen_count * 2u, slot.x);
    sws(R_SEEN + seen_count * 2u + 1u, slot.y);
    seen_count = seen_count + 1u;
    return true;
}

// ───────────────────────── 多层码 2 树（记账、中央映射）─────────────────────────

// key 按字段比：field_widths 0 = 记账 [2,8,4,8]、1 = 映射 [1,8,8,4,6]、5 = deadlist [8,8,1,8,8,4,6]（头 8、纪元 8，后五段是中央映射 key）、
// 6 = logged_ops [2,8]
fn field_width(schema: u32, field: u32) -> u32 {
    if (schema == 0u) {
        if (field == 0u) { return 2u; }
        if (field == 1u) { return 8u; }
        if (field == 2u) { return 4u; }
        return 8u;
    }
    if (schema == KEY_SCHEMA_LOGGED_OPERATIONS) {
        return select(8u, 2u, field == 0u);
    }
    // 映射那五段 [1,8,8,4,6] 从第 field_in_the_mapping_key 段数起：映射 key 从第 0 字段起，deadlist 的 key 前面多头与纪元两段 8 字节
    var field_in_the_mapping_key = field;
    if (schema == KEY_SCHEMA_DEADLIST) {
        if (field < 2u) { return 8u; }
        field_in_the_mapping_key = field - 2u;
    }
    if (field_in_the_mapping_key == 0u) { return 1u; }
    if (field_in_the_mapping_key == 1u) { return 8u; }
    if (field_in_the_mapping_key == 2u) { return 8u; }
    if (field_in_the_mapping_key == 3u) { return 4u; }
    return 6u;
}
fn field_count(schema: u32) -> u32 {
    if (schema == 0u) {
        return 4u;
    }
    if (schema == KEY_SCHEMA_DEADLIST) {
        return 7u;
    }
    if (schema == KEY_SCHEMA_LOGGED_OPERATIONS) {
        return 2u;
    }
    return 5u;
}
// 一种 key 形态的宽：各字段宽之和（记账 22、映射 27、deadlist 43）
fn key_width_of_the_schema(schema: u32) -> u32 {
    var width = 0u;
    for (var field = 0u; field < field_count(schema); field = field + 1u) {
        width = width + field_width(schema, field);
    }
    return width;
}
fn read_field(base: u32, o: u32, width: u32) -> vec2<u32> {
    var v = vec2<u32>(0u, 0u);
    for (var i = 0u; i < width; i = i + 1u) {
        let b = rd8(base, o + i);
        if (i < 4u) {
            v.x = v.x | (b << (i * 8u));
        } else {
            v.y = v.y | (b << ((i - 4u) * 8u));
        }
    }
    return v;
}
// -1 / 0 / 1 → 0 / 1 / 2
fn compare_keys(schema: u32, base_a: u32, oa: u32, base_b: u32, ob: u32) -> u32 {
    var o1 = oa;
    var o2 = ob;
    let n = field_count(schema);
    for (var f = 0u; f < n; f = f + 1u) {
        let w = field_width(schema, f);
        let a = read_field(base_a, o1, w);
        let b = read_field(base_b, o2, w);
        if (lt64(a, b)) {
            return 0u;
        }
        if (lt64(b, a)) {
            return 2u;
        }
        o1 = o1 + w;
        o2 = o2 + w;
    }
    return 1u;
}
struct Code2Frame {
    n: Node,
    child_index: u32,
    first_child_smallest_base: u32,
    first_child_smallest_offset: u32,
    previous_child_largest_base: u32,
    previous_child_largest_offset: u32,
    has_previous_child: bool,
    separator_base: u32,
    separator_offset: u32,
    previous_separator_offset: u32,
    has_previous_separator: bool,
    expected_level: u32,
    is_root: bool,
};
var<private> code2_frames: array<Code2Frame, 12>;

struct TreeRead {
    ok: bool,
    node_count: u32,
    entry_count: u32,
};
fn failed_tree_read() -> TreeRead {
    var t: TreeRead;
    t.ok = false;
    t.node_count = 0u;
    t.entry_count = 0u;
    return t;
}
// code_two_tree::read_code_two_tree（EveryHeaderAgainstItsReference）：schema 0 记账（提示读不出经映射回退）、1 映射（只按位置条目读）、
// 5 deadlist（同记账：提示读不出经映射回退；recovery.rs walk_to_file 的 deadlist 树那一支，判决 `research/prompts/m4-snapshot-r1-main-verification.md`
// 第三节 S3）。记账的叶条目写进 R_ACCOUNTING（字节基、条目偏移、条目宽）；映射的叶一片一行记进 R_MAPPING_LEAVES（条目留在叶的字节里，条数不设上限）；
// deadlist 的叶条目只数不记（冷走读只拿它的条数数映射条目，recovery.rs 的 mapping_entries_kept_for_the_snapshots），不占那两张表、不受记账那张表的条数上限
fn read_code_two_tree(root_pointer: Ptr, root_pointer_base: u32, root_pointer_offset: u32, tree: vec2<u32>, schema: u32, root: RootFields, fsid_low: vec2<u32>) -> TreeRead {
    var out = failed_tree_read();
    seen_count = 0u;
    let kw = key_width_of_the_schema(schema);
    let records_each_leaf_entry = schema == 0u;
    if (schema == 1u) {
        mapping_count = 0u;
        mapping_leaf_count = 0u;
    } else if (records_each_leaf_entry) {
        accounting_count = 0u;
    } else if (schema == KEY_SCHEMA_DEADLIST) {
        deadlist_leaf_count = 0u;
    }
    var entries = 0u;
    var nodes = 0u;
    var depth = 0u;
    // 压根
    var pending_pointer = root_pointer;
    var pending_pointer_base = root_pointer_base;
    var pending_pointer_offset = root_pointer_offset;
    var pending_expected_level = NONE;
    var pending_is_root = true;
    var enter = true;
    loop {
        if (enter) {
            // 进一个节点
            // 失败点（code_two_tree::read_node_and_its_subtree）：同一个节点两次引用、头与引用对不上、层级、空节点、key 次序与区间都是
            // InvariantViolated；映射树的节点只按位置条目读，读不出 UnitUnreadable；解不开 UnitMalformed；内部条目窄 EntryWidthDiffersFromItsFieldTable；
            // 子指针头部 UnitMalformed；记账树的节点读不出照 read_via_hint_then_mapping 记的
            if (!seen_insert(pending_pointer.slot0)) {
                walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
                return out;
            }
            var s: Slice;
            if (schema == 1u) {
                s = read_unit_via_locations(pending_pointer, NODE_BYTES);
                if (s.version == NONE) {
                    walk_failure_point = FAILURE_POINT_UNIT_UNREADABLE;
                    return out;
                }
            } else {
                let m = read_via_hint_then_mapping(pending_pointer, pending_pointer_base, pending_pointer_offset, 2u, NODE_BYTES);
                if (m.failed) {
                    return out;
                }
                s = m.s;
            }
            let n = parse_index_node(s);
            if (!n.ok) {
                walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                return out;
            }
            walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
            if (!root_header_checks(n, tree, kw, pending_pointer, root, fsid_low)) {
                return out;
            }
            if (n.level == 0u && n.entry_count > 0u) {
                if (!bytes_equal(n.base, entry_offset(n, 0u), n.base, n.smallest_offset, kw)
                    || !bytes_equal(n.base, entry_offset(n, n.entry_count - 1u), n.base, n.largest_offset, kw)) {
                    return out;
                }
            }
            if (pending_expected_level != NONE && n.level != pending_expected_level) {
                return out;
            }
            if (n.entry_count == 0u) {
                return out;
            }
            nodes = nodes + 1u;
            if (n.level == 0u) {
                // 叶：条目按 key 严格递增，记条目（映射记叶：这片叶已经进了见过的节点表，行数不过 MAX_SEEN）
                if (schema == 1u) {
                    sws(R_MAPPING_LEAVES + mapping_leaf_count, n.base);
                    mapping_leaf_count = mapping_leaf_count + 1u;
                } else if (schema == KEY_SCHEMA_DEADLIST) {
                    sws(R_DEADLIST_LEAVES + deadlist_leaf_count, n.base);
                    deadlist_leaf_count = deadlist_leaf_count + 1u;
                }
                for (var e = 0u; e < n.entry_count; e = e + 1u) {
                    if (e > 0u && compare_keys(schema, n.base, entry_offset(n, e - 1u), n.base, entry_offset(n, e)) != 0u) {
                        return out;
                    }
                    if (records_each_leaf_entry) {
                        if (entries >= MAX_ACCOUNTING_ENTRIES) {
                            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;
                            return out;
                        }
                        sws(R_ACCOUNTING + entries * 3u, n.base);
                        sws(R_ACCOUNTING + entries * 3u + 1u, entry_offset(n, e));
                        sws(R_ACCOUNTING + entries * 3u + 2u, n.entry_width);
                    }
                    entries = entries + 1u;
                }
                // 把叶交给父节点当孩子
                if (depth == 0u) {
                    out.ok = true;
                    break;
                }
                let parent = depth - 1u;
                // 分隔 key ≤ 孩子最小 key
                if (compare_keys(schema, code2_frames[parent].separator_base, code2_frames[parent].separator_offset, n.base, n.smallest_offset) == 2u) {
                    return out;
                }
                if (code2_frames[parent].child_index == 0u) {
                    code2_frames[parent].first_child_smallest_base = n.base;
                    code2_frames[parent].first_child_smallest_offset = n.smallest_offset;
                }
                code2_frames[parent].previous_child_largest_base = n.base;
                code2_frames[parent].previous_child_largest_offset = n.largest_offset;
                code2_frames[parent].has_previous_child = true;
                code2_frames[parent].child_index = code2_frames[parent].child_index + 1u;
                enter = false;
                continue;
            }
            // 内部条目宽不等于 key 宽 + 86 就拒，宽了的同样拒（code_two_tree::read_code_two_tree 读内部节点那一判与 parse_internal_entry，
            // C780，用户 2026-10-18 定「宽于字段表也拒」）；失败点沿用 EntryWidthDiffersFromItsFieldTable 那一个码，与 core 报的成员相同
            if (n.entry_width != kw + 86u) {
                walk_failure_point = FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE;
                return out;
            }
            if (depth >= MAX_FRAMES) {
                undecidable = UNDECIDABLE_TOO_DEEP;
                return out;
            }
            code2_frames[depth].n = n;
            code2_frames[depth].child_index = 0u;
            code2_frames[depth].has_previous_child = false;
            code2_frames[depth].has_previous_separator = false;
            code2_frames[depth].expected_level = pending_expected_level;
            code2_frames[depth].is_root = pending_is_root;
            depth = depth + 1u;
            enter = false;
            continue;
        }
        // 栈顶内部节点：下一个孩子，或收尾
        let top = depth - 1u;
        let n = code2_frames[top].n;
        if (code2_frames[top].child_index >= n.entry_count) {
            // 区间 = 第一个孩子的最小 key 到最后一个孩子的最大 key
            if (!bytes_equal(code2_frames[top].first_child_smallest_base, code2_frames[top].first_child_smallest_offset, n.base, n.smallest_offset, kw)
                || !bytes_equal(code2_frames[top].previous_child_largest_base, code2_frames[top].previous_child_largest_offset, n.base, n.largest_offset, kw)) {
                return out;
            }
            depth = depth - 1u;
            if (depth == 0u) {
                out.ok = true;
                break;
            }
            let parent = depth - 1u;
            if (compare_keys(schema, code2_frames[parent].separator_base, code2_frames[parent].separator_offset, n.base, n.smallest_offset) == 2u) {
                return out;
            }
            if (code2_frames[parent].child_index == 0u) {
                code2_frames[parent].first_child_smallest_base = n.base;
                code2_frames[parent].first_child_smallest_offset = n.smallest_offset;
            }
            code2_frames[parent].previous_child_largest_base = n.base;
            code2_frames[parent].previous_child_largest_offset = n.largest_offset;
            code2_frames[parent].has_previous_child = true;
            code2_frames[parent].child_index = code2_frames[parent].child_index + 1u;
            continue;
        }
        let e = code2_frames[top].child_index;
        let separator_offset = entry_offset(n, e);
        let child_pointer = parse_node_pointer(n.base, separator_offset + kw);
        // code_two_tree::parse_internal_entry：子指针头部加密、压缩那几段与 extent 偏移都要是第一版的取值
        if (!child_pointer.head_ok || !child_pointer.extent_offset_zero) {
            walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
            return out;
        }
        walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
        if (code2_frames[top].has_previous_separator) {
            if (compare_keys(schema, n.base, separator_offset, n.base, code2_frames[top].previous_separator_offset) != 2u) {
                return out;
            }
            if (compare_keys(schema, n.base, separator_offset, code2_frames[top].previous_child_largest_base, code2_frames[top].previous_child_largest_offset) != 2u) {
                return out;
            }
        }
        code2_frames[top].separator_base = n.base;
        code2_frames[top].separator_offset = separator_offset;
        code2_frames[top].previous_separator_offset = separator_offset;
        code2_frames[top].has_previous_separator = true;
        pending_pointer = child_pointer;
        pending_pointer_base = n.base;
        pending_pointer_offset = separator_offset + kw;
        pending_expected_level = n.level - 1u;
        pending_is_root = false;
        enter = true;
    }
    out.node_count = nodes;
    out.entry_count = entries;
    if (schema == 1u) {
        mapping_count = entries;
    } else if (records_each_leaf_entry) {
        accounting_count = entries;
    }
    return out;
}

// ───────────────────────── 分配记录树 ─────────────────────────

fn allocation_span(level: u32) -> vec2<u32> {
    var span = u64_of(ALLOCATION_LEAF_SLOTS);
    for (var i = 0u; i < level; i = i + 1u) {
        span = mul64_32_saturating(span, ALLOCATION_INTERNAL_FANOUT);
    }
    return span;
}
fn allocation_root_level() -> u32 {
    let slots = shr64(device_bytes(), SLOT_SHIFT);
    let device_count = hdr(3u);
    for (var level = 1u; level < 256u; level = level + 1u) {
        let child_span = allocation_span(level - 1u);
        var cells = vec2<u32>(0u, 0u);
        for (var d = 0u; d < device_count; d = d + 1u) {
            let q = divmod64(add64_saturating(slots, sub64(child_span, u64_of(1u))), child_span).quotient;
            cells = add64_saturating(cells, q);
        }
        if (le64(cells, u64_of(ALLOCATION_INTERNAL_FANOUT))) {
            return level;
        }
    }
    return 255u;
}
// 根之下的节点位置：level、device、index；根 level = root_level、device = NONE
struct AllocationNode {
    is_root: bool,
    level: u32,
    device: u32,
    index: vec2<u32>,
};
fn allocation_key_bytes_equal(base: u32, o: u32, device: u32, slot: vec2<u32>) -> bool {
    return rd32(base, o) == device && eq64(rd48(base, o + 4u), slot);
}
struct AllocationFrame {
    n: Node,
    node: AllocationNode,
    child_index: u32,
    has_previous_child: bool,
    previous_child_device: u32,
    previous_child_index: vec2<u32>,
};
var<private> allocation_frames: array<AllocationFrame, 12>;

fn allocation_first_slot(level: u32, index: vec2<u32>) -> vec2<u32> {
    let span = allocation_span(level);
    let p = mul64_32(index, span.x);
    var first = p.value;
    if (p.overflow || span.y != 0u) {
        first = vec2<u32>(NONE, NONE);
    }
    // 槽号最大 2^48 − 1
    let largest = vec2<u32>(NONE, 0xFFFFu);
    if (lt64(largest, first)) {
        first = largest;
    }
    return first;
}
fn allocation_last_slot(level: u32, index: vec2<u32>) -> vec2<u32> {
    let span = allocation_span(level);
    let first = allocation_first_slot(level, index);
    var last = add64_saturating(first, sub64(span, u64_of(1u)));
    let largest = vec2<u32>(NONE, 0xFFFFu);
    if (lt64(largest, last)) {
        last = largest;
    }
    return last;
}
struct AllocationRead {
    ok: bool,
    node_count: u32,
    record_count: u32,
};
// allocation_record_tree::read_allocation_record_tree（EveryHeaderAgainstItsReference，提示读不出经映射回退）；记录写进 R_ALLOCATION
fn read_allocation_tree(root_pointer: Ptr, root_pointer_base: u32, root_pointer_offset: u32, tree: vec2<u32>, root: RootFields, fsid_low: vec2<u32>) -> AllocationRead {
    var out: AllocationRead;
    out.ok = false;
    out.node_count = 0u;
    out.record_count = 0u;
    seen_count = 0u;
    allocation_count = 0u;
    let root_level = allocation_root_level();
    var nodes = 0u;
    var depth = 0u;
    var pending_pointer = root_pointer;
    var pending_pointer_base = root_pointer_base;
    var pending_pointer_offset = root_pointer_offset;
    var pending_node: AllocationNode;
    pending_node.is_root = true;
    pending_node.level = root_level;
    pending_node.device = NONE;
    pending_node.index = vec2<u32>(0u, 0u);
    var enter = true;
    loop {
        if (enter) {
            // 失败点（allocation_record_tree::read_node_and_its_subtree）：两次引用、头与引用、层级、key 区间、空节点、叶里记录次序、
            // 内部条目点名的孩子与次序是 InvariantViolated；解不开 UnitMalformed；条目窄 EntryWidthDiffersFromItsFieldTable；叶里记录不在叶罩的那一段
            // AllocationRecordOutsideThePoolGeometry；子指针头部 UnitMalformed；读不出照 read_via_hint_then_mapping 记的
            if (!seen_insert(pending_pointer.slot0)) {
                walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
                return out;
            }
            let m = read_via_hint_then_mapping(pending_pointer, pending_pointer_base, pending_pointer_offset, 2u, NODE_BYTES);
            if (m.failed) {
                return out;
            }
            let n = parse_index_node(m.s);
            if (!n.ok) {
                walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                return out;
            }
            walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
            if (!root_header_checks(n, tree, 10u, pending_pointer, root, fsid_low)) {
                return out;
            }
            if (n.level != pending_node.level) {
                return out;
            }
            // key 区间是它的位置规定罩的那一段
            if (pending_node.is_root) {
                if (!all_zero(n.base, n.smallest_offset, 10u)) {
                    return out;
                }
                for (var i = 0u; i < 10u; i = i + 1u) {
                    if (rd8(n.base, n.largest_offset + i) != 0xFFu) {
                        return out;
                    }
                }
            } else {
                let first = allocation_first_slot(pending_node.level, pending_node.index);
                let last = allocation_last_slot(pending_node.level, pending_node.index);
                if (!allocation_key_bytes_equal(n.base, n.smallest_offset, pending_node.device, first)
                    || !allocation_key_bytes_equal(n.base, n.largest_offset, pending_node.device, last)) {
                    return out;
                }
            }
            if (n.entry_count == 0u) {
                return out;
            }
            nodes = nodes + 1u;
            if (n.level == 0u) {
                // 分配记录宽不等于 20 就拒，宽了的同样拒（allocation_record_tree.rs 逐条 AllocationRecord::parse，C867，I-1.10 写「等于」）
                if (n.entry_width != 20u) {
                    walk_failure_point = FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE;
                    return out;
                }
                let leaf_last = allocation_last_slot(0u, pending_node.index);
                let first_record_of_the_leaf = allocation_count;
                for (var e = 0u; e < n.entry_count; e = e + 1u) {
                    let o = entry_offset(n, e);
                    let device = rd32(n.base, o);
                    let slot = rd48(n.base, o + 4u);
                    let span_field = rd16(n.base, o + 10u);
                    // 隔离记号与已释放标志同时置（0x4000 与 0x8000）：allocator::AllocationRecord::parse 拒成
                    // QuarantineMarkOnAReleasedRecord，分配记录树读者报 UnitMalformed（D19（块指针的结构与宽度预算） 已定项 5），先于落叶判
                    if ((span_field & 0xC000u) == 0xC000u) {
                        walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                        return out;
                    }
                    // 跨度取去掉两个标志位的低位；记号恢复用不上（各盘那几条比的是 (槽, 跨度, 代, 已释放)，
                    // recovery::allocation_records_are_one_per_device），存进表时去掉，下面几处照旧按 0x7FFF 取跨度
                    let span = span_field & 0x3FFFu;
                    let generation = rd64(n.base, o + 12u);
                    // 落在这片叶里：同盘、叶序号 = 槽 / 812、末槽不越过叶末槽
                    let leaf_index = divmod64(slot, u64_of(ALLOCATION_LEAF_SLOTS)).quotient;
                    if (pending_node.is_root || device != pending_node.device || !eq64(leaf_index, pending_node.index)) {
                        walk_failure_point = FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY;
                        return out;
                    }
                    if (span == 0u || lt64(leaf_last, add64(slot, u64_of(span - 1u)))) {
                        walk_failure_point = FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY;
                        return out;
                    }
                    if (allocation_count >= MAX_ALLOCATION_RECORDS) {
                        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;
                        return out;
                    }
                    sws(R_ALLOCATION + allocation_count * ALLOCATION_RECORD_WORDS, device);
                    sws(R_ALLOCATION + allocation_count * ALLOCATION_RECORD_WORDS + 1u, slot.x);
                    sws(R_ALLOCATION + allocation_count * ALLOCATION_RECORD_WORDS + 2u, slot.y);
                    sws(R_ALLOCATION + allocation_count * ALLOCATION_RECORD_WORDS + 3u, span_field & 0xBFFFu);
                    sws(R_ALLOCATION + allocation_count * ALLOCATION_RECORD_WORDS + 4u, generation.x);
                    sws(R_ALLOCATION + allocation_count * ALLOCATION_RECORD_WORDS + 5u, generation.y);
                    allocation_count = allocation_count + 1u;
                }
                // 叶里的记录按 key 严格递增：CPU 逐条解完、判完落在叶里之后才判（read_node_and_its_subtree 叶那一支的 keys_ascend）
                for (var r = first_record_of_the_leaf + 1u; r < allocation_count; r = r + 1u) {
                    let previous_device = sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS);
                    let previous_slot = vec2<u32>(sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS + 1u), sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS + 2u));
                    let device = sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS);
                    let slot = vec2<u32>(sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS + 1u), sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS + 2u));
                    if (!(previous_device < device || (previous_device == device && lt64(previous_slot, slot)))) {
                        walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
                        return out;
                    }
                }
                if (depth == 0u) {
                    out.ok = true;
                    break;
                }
                allocation_frames[depth - 1u].child_index = allocation_frames[depth - 1u].child_index + 1u;
                enter = false;
                continue;
            }
            // 分配记录树内部条目宽不等于 96 就拒，宽了的同样拒（allocation_record_tree.rs 那一判，C867）
            if (n.entry_width != 96u) {
                walk_failure_point = FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE;
                return out;
            }
            if (depth >= MAX_FRAMES) {
                undecidable = UNDECIDABLE_TOO_DEEP;
                return out;
            }
            allocation_frames[depth].n = n;
            allocation_frames[depth].node = pending_node;
            allocation_frames[depth].child_index = 0u;
            allocation_frames[depth].has_previous_child = false;
            depth = depth + 1u;
            enter = false;
            continue;
        }
        let top = depth - 1u;
        let n = allocation_frames[top].n;
        let parent = allocation_frames[top].node;
        if (allocation_frames[top].child_index >= n.entry_count) {
            depth = depth - 1u;
            if (depth == 0u) {
                out.ok = true;
                break;
            }
            allocation_frames[depth - 1u].child_index = allocation_frames[depth - 1u].child_index + 1u;
            continue;
        }
        let e = allocation_frames[top].child_index;
        let o = entry_offset(n, e);
        // 孩子由条目 key 点名：槽落在孩子那一层的格点上、盘在池里、孩子的父就是这个节点
        let child_level = parent.level - 1u;
        let device = rd32(n.base, o);
        let slot = rd48(n.base, o + 4u);
        let child_span = allocation_span(child_level);
        let qr = divmod64(slot, child_span);
        walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
        if (!is_zero64(qr.remainder) || !device_is_in_the_pool(device)) {
            return out;
        }
        var child: AllocationNode;
        child.is_root = false;
        child.level = child_level;
        child.device = device;
        child.index = qr.quotient;
        // parent_of(child) == parent
        if (child_level + 1u == root_level) {
            if (!parent.is_root) {
                return out;
            }
        } else {
            if (parent.is_root) {
                return out;
            }
            let parent_index = divmod64(child.index, u64_of(ALLOCATION_INTERNAL_FANOUT)).quotient;
            if (parent.device != device || !eq64(parent_index, parent.index)) {
                return out;
            }
        }
        if (allocation_frames[top].has_previous_child) {
            let previous_device = allocation_frames[top].previous_child_device;
            let previous_index = allocation_frames[top].previous_child_index;
            if (!(previous_device < device || (previous_device == device && lt64(previous_index, child.index)))) {
                return out;
            }
        }
        allocation_frames[top].has_previous_child = true;
        allocation_frames[top].previous_child_device = device;
        allocation_frames[top].previous_child_index = child.index;
        pending_pointer = parse_node_pointer(n.base, o + 10u);
        // allocation_record_tree 读内部条目的子指针：头部加密、压缩那几段不是第一版的取值，这个节点损坏
        if (!pending_pointer.head_ok) {
            walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
            return out;
        }
        // allocation_record_tree 读内部条目的子指针：头部 extent 偏移非 0，这个节点损坏
        if (!pending_pointer.extent_offset_zero) {
            walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
            return out;
        }
        pending_pointer_base = n.base;
        pending_pointer_offset = o + 10u;
        pending_node = child;
        enter = true;
    }
    out.node_count = nodes;
    out.record_count = allocation_count;
    return out;
}

// ───────────────────────── extent 树 ─────────────────────────

fn extent_upper_span(level: u32) -> vec2<u32> {
    var span = u64_of(EXTENT_UPPER_LEAF_INODES);
    for (var i = 0u; i < level; i = i + 1u) {
        span = mul64_32_saturating(span, EXTENT_INTERNAL_FANOUT);
    }
    return span;
}
fn extent_lower_span(level: u32) -> vec2<u32> {
    var span = u64_of(EXTENT_LOWER_LEAF_DATA_UNITS);
    for (var i = 0u; i < level; i = i + 1u) {
        span = mul64_32_saturating(span, EXTENT_INTERNAL_FANOUT);
    }
    return span;
}
fn mul64_64_saturating(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    if (b.y != 0u) {
        if (is_zero64(a)) {
            return a;
        }
        return vec2<u32>(NONE, NONE);
    }
    return mul64_32_saturating(a, b.x);
}
// extent key 的三段
fn extent_key_locality(base: u32, o: u32) -> vec2<u32> { return rd64(base, o); }
fn extent_key_inode(base: u32, o: u32) -> vec2<u32> { return rd64(base, o + 8u); }
fn extent_key_offset(base: u32, o: u32) -> vec2<u32> { return rd64(base, o + 16u); }
fn extent_key_equals(base: u32, o: u32, inode: vec2<u32>, offset: vec2<u32>) -> bool {
    return is_zero64(rd64(base, o)) && eq64(rd64(base, o + 8u), inode) && eq64(rd64(base, o + 16u), offset);
}
// extent_tree 的 internal_entries：一个内部节点的每条子指针（条目 key 24 之后）头部加密、压缩那几段是第一版的取值、extent 偏移为 0
// （NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from）；
// core 在往下读第一个孩子之前就把整个节点的条目解完，有一条不是这个节点就损坏
fn extent_internal_child_pointer_heads_are_the_first_version(n: Node) -> bool {
    for (var e = 0u; e < n.entry_count; e = e + 1u) {
        if (!pointer_encryption_and_compression_fields_are_the_first_version(n.base, entry_offset(n, e) + 24u)) {
            return false;
        }
        if (!pointer_extent_offset_is_zero(n.base, entry_offset(n, e) + 24u)) {
            return false;
        }
    }
    return true;
}
struct ExtentFrame {
    n: Node,
    level: u32,
    index: vec2<u32>,
    child_index: u32,
    has_previous_child: bool,
    previous_child_index: vec2<u32>,
};
var<private> extent_frames: array<ExtentFrame, 12>;

struct ExtentRead {
    ok: bool,
    node_count: u32,
    // 上段叶条目一共几条（read_upper_segment 交回；记进 R_UPPER 的只是从 batch_first 起的那一批）
    upper_entry_total: u32,
};
// 判一个 extent 树节点（extent_tree::ExtentTreeReading::read_and_judge_node，Every）；is_the_root_of_the_tree 说它是不是整棵树的根
// （上段的根，树表条目指着它；下段的根不算）
fn judge_extent_node(n: Node, tree: vec2<u32>, p: Ptr, root: RootFields, fsid_low: vec2<u32>, expected_level: u32, is_the_root_of_the_tree: bool,
    range_inode_first: vec2<u32>, range_offset_first: vec2<u32>, range_inode_last: vec2<u32>, range_offset_last: vec2<u32>) -> bool {
    // 失败点：解不开 UnitMalformed；头与引用、层级、key 区间、空节点 InvariantViolated（extent_tree::read_and_judge_node）
    if (!n.ok) {
        walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
        return false;
    }
    walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
    if (!root_header_checks(n, tree, 24u, p, root, fsid_low)) {
        return false;
    }
    if (n.level != expected_level) {
        return false;
    }
    if (!extent_key_equals(n.base, n.smallest_offset, range_inode_first, range_offset_first)
        || !extent_key_equals(n.base, n.largest_offset, range_inode_last, range_offset_last)) {
        return false;
    }
    // 零条目只许上段的根兼叶：第一个文件删完之后的零条目根（D8（核心索引结构） 已定项 16 第 5 条），与 read_and_judge_node 同一判
    if (n.entry_count == 0u && !(is_the_root_of_the_tree && expected_level == 0u)) {
        return false;
    }
    return true;
}
// 上段：按走读次序第 batch_first 条起的叶条目写进 R_UPPER（字节基、条目偏移），至多 MAX_UPPER_ENTRIES 条，upper_count 是这一批记了几条。
// 按块分批读、不设上限（用户 2026-10-19 定，与 R_DATA 那一段同一个定案）：is_the_judging_pass 那一遍（batch_first 是 0）照 extent_tree::read_upper_segment
// 逐节点判、记见过、数全部叶条目，记满一批之后的条目照判、不记；后面几批是同一棵已判过的上段从根再走一遍，只按次序记下一批，不再记见过
// （同一个读法、同一份字节，判得过的一遍读不出别的结局）
fn read_upper_segment(root_pointer: Ptr, root_pointer_base: u32, root_pointer_offset: u32, tree: vec2<u32>, root: RootFields, fsid_low: vec2<u32>,
    is_the_judging_pass: bool, batch_first: u32) -> ExtentRead {
    var out: ExtentRead;
    out.ok = false;
    out.node_count = 0u;
    out.upper_entry_total = 0u;
    upper_count = 0u;
    var entry_ordinal = 0u;
    var nodes = 0u;
    var depth = 0u;
    var pending_pointer = root_pointer;
    var pending_pointer_base = root_pointer_base;
    var pending_pointer_offset = root_pointer_offset;
    var pending_level = NONE;
    var pending_index = vec2<u32>(0u, 0u);
    var enter = true;
    loop {
        if (enter) {
            // 段的根先读、先解出层级，再记「见过」（extent_tree::read_upper_segment / read_lower_segment 先 read_node、parse_index_node，
            // read_and_judge_node 才插 units_seen）；段里别的节点先记见过再读。记下一批的那几遍不记见过（判的那一遍记过了）
            let is_the_root_of_the_segment = depth == 0u;
            if (is_the_judging_pass && !is_the_root_of_the_segment && !seen_insert(pending_pointer.slot0)) {
                walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
                return out;
            }
            let m = read_via_hint_then_mapping(pending_pointer, pending_pointer_base, pending_pointer_offset, 2u, NODE_BYTES);
            if (m.failed) {
                return out;
            }
            let n = parse_index_node(m.s);
            if (!n.ok) {
                walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                return out;
            }
            if (is_the_judging_pass && is_the_root_of_the_segment && !seen_insert(pending_pointer.slot0)) {
                walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
                return out;
            }
            var level = pending_level;
            if (level == NONE) {
                level = n.level;
            }
            let span = extent_upper_span(level);
            let first_inode = mul64_64_saturating(pending_index, span);
            let last_inode = add64_saturating(first_inode, sub64(span, u64_of(1u)));
            if (!judge_extent_node(n, tree, pending_pointer, root, fsid_low, level, depth == 0u, first_inode, vec2<u32>(0u, 0u), last_inode, vec2<u32>(NONE, NONE))) {
                return out;
            }
            nodes = nodes + 1u;
            if (level == 0u) {
                // 条目宽只在有条目时核：extent_tree.rs 的 upper_leaf_entries 逐条 ExtentUpperLeafEntry::parse 时才核宽，
                // 零条目的上段根（第一个文件删完之后）条目宽是什么都不拒；有条目时宽不等于 113 就拒，宽了的同样拒（ExtentUpperLeafEntry::parse，I-1.10 写「等于」）
                if (n.entry_count > 0u && n.entry_width != EXTENT_UPPER_LEAF_ENTRY_BYTES) {
                    walk_failure_point = FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE;
                    return out;
                }
                var previous_inode = vec2<u32>(0u, 0u);
                for (var e = 0u; e < n.entry_count; e = e + 1u) {
                    let o = entry_offset(n, e);
                    // 失败点（extent_tree::upper_leaf_entries）：标签不认识、字段表写零的字节不是零、不在叶罩的那一段、次序 InvariantViolated；
                    // 指针头部 UnitMalformed
                    walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
                    if (!is_zero64(extent_key_locality(n.base, o)) || !is_zero64(extent_key_offset(n.base, o))) {
                        return out;
                    }
                    let inode = extent_key_inode(n.base, o);
                    let tag = rd8(n.base, o + 24u);
                    if (tag == 0u) {
                        if (!all_zero(n.base, o + 25u, 88u)) {
                            return out;
                        }
                    } else if (tag == 1u || tag == 3u) {
                        // 标签 3 的载荷与标签 1 同一个字段表（ExtentUpperLeafEntry::parse 换成没有洞的标签照同一段解，判决 m4-trunc-r1 第三节 K5），
                        // 下面几判同 1；4 同 2
                        if (!all_zero(n.base, o + 25u + 86u, 2u)) {
                            return out;
                        }
                        // ExtentUpperLeafEntry::parse：下段根指针头部加密、压缩那几段不是第一版的取值，这条条目损坏
                        if (!pointer_encryption_and_compression_fields_are_the_first_version(n.base, o + 25u)) {
                            walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                            return out;
                        }
                        // ExtentUpperLeafEntry::parse：下段根指针头部 extent 偏移非 0，这条条目损坏
                        if (!pointer_extent_offset_is_zero(n.base, o + 25u)) {
                            walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                            return out;
                        }
                    } else if (tag == 2u || tag == 4u) {
                        // 同上：内联的数据指针（加密、压缩那几段与 extent 偏移各判一次）
                        if (!pointer_encryption_and_compression_fields_are_the_first_version(n.base, o + 25u)) {
                            walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                            return out;
                        }
                        // 同上：内联的数据指针
                        if (!pointer_extent_offset_is_zero(n.base, o + 25u)) {
                            walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                            return out;
                        }
                    } else {
                        return out;
                    }
                    if (lt64(inode, first_inode) || lt64(last_inode, inode)) {
                        return out;
                    }
                    if (e > 0u && !lt64(previous_inode, inode)) {
                        return out;
                    }
                    previous_inode = inode;
                    // 这一批从第 batch_first 条起、至多 MAX_UPPER_ENTRIES 条；这一批之外的照判、照数，不记
                    if (entry_ordinal >= batch_first && upper_count < MAX_UPPER_ENTRIES) {
                        sws(R_UPPER + upper_count * 2u, n.base);
                        sws(R_UPPER + upper_count * 2u + 1u, o);
                        upper_count = upper_count + 1u;
                    }
                    entry_ordinal = entry_ordinal + 1u;
                }
                if (depth == 0u) {
                    out.ok = true;
                    break;
                }
                extent_frames[depth - 1u].child_index = extent_frames[depth - 1u].child_index + 1u;
                enter = false;
                continue;
            }
            // extent_tree::internal_entries：条目宽不等于 110（窄了宽了都算，C867）EntryWidthDiffersFromItsFieldTable，再逐条判子指针头部 UnitMalformed，都在读任何一个孩子之前
            if (n.entry_width != 110u) {
                walk_failure_point = FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE;
                return out;
            }
            if (!extent_internal_child_pointer_heads_are_the_first_version(n)) {
                walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                return out;
            }
            if (depth >= MAX_FRAMES) {
                undecidable = UNDECIDABLE_TOO_DEEP;
                return out;
            }
            extent_frames[depth].n = n;
            extent_frames[depth].level = level;
            extent_frames[depth].index = pending_index;
            extent_frames[depth].child_index = 0u;
            extent_frames[depth].has_previous_child = false;
            depth = depth + 1u;
            enter = false;
            continue;
        }
        let top = depth - 1u;
        let n = extent_frames[top].n;
        if (extent_frames[top].child_index >= n.entry_count) {
            depth = depth - 1u;
            if (depth == 0u) {
                out.ok = true;
                break;
            }
            extent_frames[depth - 1u].child_index = extent_frames[depth - 1u].child_index + 1u;
            continue;
        }
        let e = extent_frames[top].child_index;
        let o = entry_offset(n, e);
        let child_level = extent_frames[top].level - 1u;
        let child_span = extent_upper_span(child_level);
        let first_inode = extent_key_inode(n.base, o);
        walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
        if (!is_zero64(extent_key_locality(n.base, o)) || !is_zero64(extent_key_offset(n.base, o))) {
            return out;
        }
        let qr = divmod64(first_inode, child_span);
        if (!is_zero64(qr.remainder)) {
            return out;
        }
        let child_index = qr.quotient;
        // child.parent() == parent：index / 147
        let parent_index = divmod64(child_index, u64_of(EXTENT_INTERNAL_FANOUT)).quotient;
        if (!eq64(parent_index, extent_frames[top].index)) {
            return out;
        }
        if (extent_frames[top].has_previous_child && !lt64(extent_frames[top].previous_child_index, child_index)) {
            return out;
        }
        extent_frames[top].has_previous_child = true;
        extent_frames[top].previous_child_index = child_index;
        pending_pointer = parse_node_pointer(n.base, o + 24u);
        pending_pointer_base = n.base;
        pending_pointer_offset = o + 24u;
        pending_level = child_level;
        pending_index = child_index;
        enter = true;
    }
    out.node_count = nodes;
    out.upper_entry_total = entry_ordinal;
    return out;
}
// 下段：数一个文件的每一个数据单元（units_of_the_file_being_read），是第一个文件的下段时头一块的数据指针写进 R_DATA（字节基、指针偏移），
// 按单元号升序；单元号要从 0 起连号由调用方判（读到的单元号与它在这个文件里的位置不同就记进
// units_of_the_file_being_read_are_numbered_from_zero_without_a_gap）
fn read_lower_segment(root_pointer: Ptr, root_pointer_base: u32, root_pointer_offset: u32, inode: vec2<u32>, tree: vec2<u32>, root: RootFields, fsid_low: vec2<u32>,
    is_the_first_file: bool) -> ExtentRead {
    var out: ExtentRead;
    out.ok = false;
    out.node_count = 0u;
    out.upper_entry_total = 0u;
    var nodes = 0u;
    var depth = 0u;
    var pending_pointer = root_pointer;
    var pending_pointer_base = root_pointer_base;
    var pending_pointer_offset = root_pointer_offset;
    var pending_level = NONE;
    var pending_index = vec2<u32>(0u, 0u);
    var enter = true;
    let payload = u64_of(DATA_UNIT_PAYLOAD_CAPACITY);
    loop {
        if (enter) {
            // 段的根先读、先解出层级，再记「见过」（extent_tree::read_upper_segment / read_lower_segment 先 read_node、parse_index_node，
            // read_and_judge_node 才插 units_seen）；段里别的节点先记见过再读
            let is_the_root_of_the_segment = depth == 0u;
            if (!is_the_root_of_the_segment && !seen_insert(pending_pointer.slot0)) {
                walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
                return out;
            }
            let m = read_via_hint_then_mapping(pending_pointer, pending_pointer_base, pending_pointer_offset, 2u, NODE_BYTES);
            if (m.failed) {
                return out;
            }
            let n = parse_index_node(m.s);
            if (!n.ok) {
                walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                return out;
            }
            if (is_the_root_of_the_segment && !seen_insert(pending_pointer.slot0)) {
                walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
                return out;
            }
            var level = pending_level;
            if (level == NONE) {
                level = n.level;
            }
            let span = extent_lower_span(level);
            let first_unit = mul64_64_saturating(pending_index, span);
            let last_unit = add64_saturating(first_unit, sub64(span, u64_of(1u)));
            let first_offset = mul64_64_saturating(first_unit, payload);
            let last_offset = sub64_saturating(mul64_64_saturating(add64_saturating(last_unit, u64_of(1u)), payload), u64_of(1u));
            if (!judge_extent_node(n, tree, pending_pointer, root, fsid_low, level, false, inode, first_offset, inode, last_offset)) {
                return out;
            }
            nodes = nodes + 1u;
            if (level == 0u) {
                // extent 叶记录宽不等于 112 就拒，宽了的同样拒（records.rs parse_extent_record，C867）
                if (n.entry_width != 112u) {
                    walk_failure_point = FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE;
                    return out;
                }
                var previous_unit = vec2<u32>(0u, 0u);
                for (var e = 0u; e < n.entry_count; e = e + 1u) {
                    let o = entry_offset(n, e);
                    // records::parse_extent_record 先判数据指针头部（UnitMalformed），read_lower_node_and_its_subtree 再判 key、区间与次序（InvariantViolated）
                    // records::parse_extent_record：数据指针头部加密、压缩那几段不是第一版的取值，这条叶记录损坏
                    if (!pointer_encryption_and_compression_fields_are_the_first_version(n.base, o + 24u)) {
                        walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                        return out;
                    }
                    // records::parse_extent_record：数据指针头部 extent 偏移非 0，这条叶记录损坏
                    if (!pointer_extent_offset_is_zero(n.base, o + 24u)) {
                        walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                        return out;
                    }
                    walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
                    let offset = extent_key_offset(n.base, o);
                    let qr = divmod64(offset, payload);
                    if (!is_zero64(extent_key_locality(n.base, o)) || !eq64(extent_key_inode(n.base, o), inode) || !is_zero64(qr.remainder)) {
                        return out;
                    }
                    let unit = qr.quotient;
                    if (lt64(unit, first_unit) || lt64(last_unit, unit)) {
                        return out;
                    }
                    if (e > 0u && !lt64(previous_unit, unit)) {
                        return out;
                    }
                    previous_unit = unit;
                    if (is_the_first_file && units_of_the_file_being_read < DATA_POINTERS_PER_BLOCK) {
                        sws(R_DATA + units_of_the_file_being_read * 2u, n.base);
                        sws(R_DATA + units_of_the_file_being_read * 2u + 1u, o + 24u);
                    }
                    // 单元号要等于它在整个文件里的位置（extents_of_the_first_file 的「没有洞」，第一个文件与别的文件同一条）：这里记下来，调用方判
                    if (!eq64(unit, u64_of(units_of_the_file_being_read))) {
                        units_of_the_file_being_read_are_numbered_from_zero_without_a_gap = false;
                    }
                    units_of_the_file_being_read = units_of_the_file_being_read + 1u;
                }
                if (depth == 0u) {
                    out.ok = true;
                    break;
                }
                extent_frames[depth - 1u].child_index = extent_frames[depth - 1u].child_index + 1u;
                enter = false;
                continue;
            }
            // extent_tree::internal_entries：条目宽不等于 110（窄了宽了都算，C867）EntryWidthDiffersFromItsFieldTable，再逐条判子指针头部 UnitMalformed，都在读任何一个孩子之前
            if (n.entry_width != 110u) {
                walk_failure_point = FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE;
                return out;
            }
            if (!extent_internal_child_pointer_heads_are_the_first_version(n)) {
                walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                return out;
            }
            if (depth >= MAX_FRAMES) {
                undecidable = UNDECIDABLE_TOO_DEEP;
                return out;
            }
            extent_frames[depth].n = n;
            extent_frames[depth].level = level;
            extent_frames[depth].index = pending_index;
            extent_frames[depth].child_index = 0u;
            extent_frames[depth].has_previous_child = false;
            depth = depth + 1u;
            enter = false;
            continue;
        }
        let top = depth - 1u;
        let n = extent_frames[top].n;
        if (extent_frames[top].child_index >= n.entry_count) {
            depth = depth - 1u;
            if (depth == 0u) {
                out.ok = true;
                break;
            }
            extent_frames[depth - 1u].child_index = extent_frames[depth - 1u].child_index + 1u;
            continue;
        }
        let e = extent_frames[top].child_index;
        let o = entry_offset(n, e);
        let level = extent_frames[top].level;
        let child_span = extent_lower_span(level - 1u);
        let child_offset_span = mul64_64_saturating(child_span, payload);
        let offset = extent_key_offset(n.base, o);
        walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
        if (!is_zero64(extent_key_locality(n.base, o)) || !eq64(extent_key_inode(n.base, o), inode)) {
            return out;
        }
        let qr = divmod64(offset, child_offset_span);
        if (!is_zero64(qr.remainder)) {
            return out;
        }
        let child_index = qr.quotient;
        // child.data_unit_range().0 / span(level) == index
        let child_first_unit = mul64_64_saturating(child_index, child_span);
        let parent_index = divmod64(child_first_unit, extent_lower_span(level)).quotient;
        if (!eq64(parent_index, extent_frames[top].index)) {
            return out;
        }
        if (extent_frames[top].has_previous_child && !lt64(extent_frames[top].previous_child_index, child_index)) {
            return out;
        }
        extent_frames[top].has_previous_child = true;
        extent_frames[top].previous_child_index = child_index;
        pending_pointer = parse_node_pointer(n.base, o + 24u);
        pending_pointer_base = n.base;
        pending_pointer_offset = o + 24u;
        pending_level = level - 1u;
        pending_index = child_index;
        enter = true;
    }
    out.node_count = nodes;
    return out;
}
// 整棵 extent 树 + extents_of_the_first_file：上段整段（按块分批记叶条目），再按每条叶条目读那个文件的下段。第一个文件之外有数据单元的文件
// （按 inode 号写数据写出来的）与 CPU 恢复同一套收法（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M6）：
// 至少一个单元、单元从 0 起连号、尾部没有洞（标签 1、2）；它们的单元数并进映射条目数（recovery 的 data_units_of_every_file），
// 冷走读不读它们的数据单元、不找它们的记录（recovery.rs walk_to_file 同）
fn read_extent_tree_of_the_first_file(root_pointer: Ptr, root_pointer_base: u32, root_pointer_offset: u32, tree: vec2<u32>, root: RootFields, fsid_low: vec2<u32>) -> ExtentRead {
    var out: ExtentRead;
    out.ok = false;
    out.node_count = 0u;
    out.upper_entry_total = 0u;
    seen_count = 0u;
    data_count = 0u;
    files_of_the_extent_tree = 0u;
    data_units_of_the_files_other_than_the_first = 0u;
    data_units_are_numbered_from_zero_without_a_gap = true;
    first_file_upper_leaf_tag = NONE;
    first_file_upper_entry_base = NONE;
    first_file_upper_entry_offset = 0u;
    // 第一个文件之外的文件有一个没有单元、单元有洞或尾部有洞（recovery::extents_of_the_first_file 在读完整棵树之后才判，读的失败点先报）
    var files_other_than_the_first_are_well_formed = true;
    var nodes = 0u;
    var upper_entry_total = 0u;
    var batch_first = 0u;
    // 迭代上界：每一批至少记一条叶条目（记不到就判不了、交回），批数不过叶条目数 ÷ MAX_UPPER_ENTRIES 向上取整；跨轮带着的是下一批从第几条起。
    // 上段与下段各一个调用点（GPU 程序把每个调用点展开成一份拷贝）
    loop {
        let is_the_judging_pass = batch_first == 0u;
        let upper = read_upper_segment(root_pointer, root_pointer_base, root_pointer_offset, tree, root, fsid_low, is_the_judging_pass, batch_first);
        if (!upper.ok) {
            return out;
        }
        if (is_the_judging_pass) {
            nodes = upper.node_count;
            upper_entry_total = upper.upper_entry_total;
        }
        if (upper_count == 0u && batch_first < upper_entry_total) {
            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;
            return out;
        }
        for (var i = 0u; i < upper_count; i = i + 1u) {
            let base = sw(R_UPPER + i * 2u);
            let o = sw(R_UPPER + i * 2u + 1u);
            let inode = extent_key_inode(base, o);
            let tag = rd8(base, o + 24u);
            let is_the_first_file = eq64(inode, u64_of(FIRST_INODE_NUMBER));
            // extent_tree::read_extent_tree：标签 1、3 读下段，2、4 是内联的那一个单元（3、4 之后到 size 是洞，判决 m4-trunc-r1 第三节 K5）；标签 0 一个单元都没有
            let names_a_lower_segment_root = tag == 1u || tag == 3u;
            let names_an_inline_data_unit = tag == 2u || tag == 4u;
            units_of_the_file_being_read = 0u;
            units_of_the_file_being_read_are_numbered_from_zero_without_a_gap = true;
            if (names_a_lower_segment_root) {
                let lower_root = parse_node_pointer(base, o + 25u);
                let lower = read_lower_segment(lower_root, base, o + 25u, inode, tree, root, fsid_low, is_the_first_file);
                if (!lower.ok) {
                    return out;
                }
                nodes = nodes + lower.node_count;
            } else if (names_an_inline_data_unit) {
                if (is_the_first_file) {
                    sws(R_DATA, base);
                    sws(R_DATA + 1u, o + 25u);
                }
                units_of_the_file_being_read = 1u;
            }
            // 建上一版那一样读者要每个文件的单元：上段叶条目在哪、标签、单元数记一行（按 inode 号升序，下段读完才记，单元数是数全的）
            if (extent_tree_records_its_files) {
                if (files_of_the_extent_tree >= MAX_FILES_OF_THE_EXTENT_TREE) {
                    undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;
                    return out;
                }
                let row = R_FILES + files_of_the_extent_tree * FILE_ROW_WORDS;
                sws(row, inode.x);
                sws(row + 1u, inode.y);
                sws(row + 2u, base);
                sws(row + 3u, o);
                sws(row + 4u, tag);
                sws(row + 5u, units_of_the_file_being_read);
                sws(row + 6u, 0u);
                files_of_the_extent_tree = files_of_the_extent_tree + 1u;
            }
            if (is_the_first_file) {
                first_file_upper_leaf_tag = tag;
                first_file_upper_entry_base = base;
                first_file_upper_entry_offset = o;
                data_count = units_of_the_file_being_read;
                data_units_are_numbered_from_zero_without_a_gap = units_of_the_file_being_read_are_numbered_from_zero_without_a_gap;
            } else {
                data_units_of_the_files_other_than_the_first = data_units_of_the_files_other_than_the_first + units_of_the_file_being_read;
                // 第一个文件之外的文件与第一个文件同一读法（判决 m4-multi-inode-size-r1 第二节 S2，recovery::extents_of_the_first_file）：
                // 标签 0 一个单元都没有（粘住的标签 0）、标签 3、4 尾部是洞，都收；只拒单元中间有洞（从 0 起不连号）
                if (!units_of_the_file_being_read_are_numbered_from_zero_without_a_gap) {
                    files_other_than_the_first_are_well_formed = false;
                }
            }
        }
        batch_first = batch_first + upper_count;
        if (batch_first >= upper_entry_total) {
            break;
        }
    }
    data_block_first = 0u;
    data_block_count = min(data_count, DATA_POINTERS_PER_BLOCK);
    // recovery::extents_of_the_first_file：哪个文件的单元中间有洞都是 UnitMalformed
    walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
    if (!files_other_than_the_first_are_well_formed) {
        return out;
    }
    // 上段里没有第一个文件那一条：第一个文件删完之后的零条目根（D8（核心索引结构） 已定项 16 第 5 条），或删完之后只有别的文件有条目，
    // 第一个文件没有数据单元、没有下段，读得出（同 recovery.rs extents_of_the_first_file 那一支）；结局由后面 inode 树里第一个文件那条记录定
    if (first_file_upper_leaf_tag == NONE) {
        out.ok = true;
        out.node_count = nodes;
        return out;
    }
    // 标签 0 配 size > 0 是整个文件都是洞（判决 m4-trunc-r1 第三节 K5、第六节 S1），一个单元都没有也读得出；别的标签至少一个单元
    if (data_count == 0u && first_file_upper_leaf_tag != 0u) {
        return out;
    }
    if (!data_units_are_numbered_from_zero_without_a_gap) {
        return out;
    }
    out.ok = true;
    out.node_count = nodes;
    return out;
}
// 找下一块数据指针时往下走的一步（walk_to_file 读了下段里一个节点交给它）：is_the_block_found 为真时 R_DATA 换成了下一块；
// 否则 pointer_base / pointer_offset 是下一层要读的那条子指针在哪
struct StepTowardTheNextBlockOfDataPointers {
    is_the_block_found: bool,
    pointer_base: u32,
    pointer_offset: u32,
};
// 第一个文件的第 unit 个单元在下段的哪片叶里：节点 read 是从下段根往下读回的一层。内部节点取子区间起点不超过 unit 的最后一条条目往下；
// 叶里从 unit 那一条起、到叶末尾的数据指针记成下一块（叶不过 EXTENT_LOWER_LEAF_DATA_UNITS 条，装得进一块）。
// 这几层在 extent 树那一遍都读过、核过（read_lower_segment：键升序、子区间对得上父区间、单元从 0 起连号、没有零条目的下段节点），
// 这里不再判，只认路；读回的不是那一形记判不了（UNDECIDABLE_DATA_POINTER_BLOCK_NOT_FOUND_AGAIN）
fn find_the_next_block_of_data_pointers(read: MappedRead, unit: u32) -> StepTowardTheNextBlockOfDataPointers {
    var step: StepTowardTheNextBlockOfDataPointers;
    step.is_the_block_found = false;
    step.pointer_base = NONE;
    step.pointer_offset = 0u;
    let payload = u64_of(DATA_UNIT_PAYLOAD_CAPACITY);
    let n = parse_index_node(read.s);
    if (read.failed || !n.ok || n.entry_count == 0u) {
        undecidable = UNDECIDABLE_DATA_POINTER_BLOCK_NOT_FOUND_AGAIN;
        return step;
    }
    if (n.level > 0u) {
        var chosen = 0u;
        for (var e = 1u; e < n.entry_count; e = e + 1u) {
            let first_unit_of_the_child = divmod64(extent_key_offset(n.base, entry_offset(n, e)), payload).quotient;
            if (lt64(u64_of(unit), first_unit_of_the_child)) {
                break;
            }
            chosen = e;
        }
        step.pointer_base = n.base;
        step.pointer_offset = entry_offset(n, chosen) + 24u;
        return step;
    }
    let first_unit_of_the_leaf = divmod64(extent_key_offset(n.base, entry_offset(n, 0u)), payload).quotient;
    let entry_of_the_unit = unit - first_unit_of_the_leaf.x;
    if (first_unit_of_the_leaf.y != 0u || first_unit_of_the_leaf.x > unit || entry_of_the_unit >= n.entry_count) {
        undecidable = UNDECIDABLE_DATA_POINTER_BLOCK_NOT_FOUND_AGAIN;
        return step;
    }
    data_block_first = unit;
    data_block_count = min(n.entry_count - entry_of_the_unit, DATA_POINTERS_PER_BLOCK);
    for (var k = 0u; k < data_block_count; k = k + 1u) {
        sws(R_DATA + k * 2u, n.base);
        sws(R_DATA + k * 2u + 1u, entry_offset(n, entry_of_the_unit + k) + 24u);
    }
    step.is_the_block_found = true;
    return step;
}

// ───────────────────────── 实例表 ─────────────────────────

// InstanceTablePage::parse 的每一片：身份 (0, 4, 片号, 0)、行 kind 0 且 flags 0、末条是合法链指针记录；交回 0 最后一片、1 有下一片（写进 next）、2 坏
struct PageParse {
    verdict: u32,
    next: Ptr,
};
fn parse_instance_table_page(p: Packed, page_index: vec2<u32>) -> PageParse {
    var out: PageParse;
    out.verdict = 2u;
    out.next = zero_pointer();
    if (!p.ok) {
        return out;
    }
    if (!is_zero64(p.birth_tree) || p.record_type != 4u || !eq64(p.container, page_index) || !is_zero64(p.container_birth)) {
        return out;
    }
    if (p.record_count == 0u || p.record_width != 88u) {
        return out;
    }
    let last = packed_record_offset(p, p.record_count - 1u);
    if (rd8(p.base, last) != 1u) {
        return out;
    }
    let flag = rd8(p.base, last + 1u);
    let pointer_zero = all_zero(p.base, last + 2u, 86u);
    if (flag == 0u) {
        if (!pointer_zero) {
            return out;
        }
        out.verdict = 0u;
    } else if (flag == 1u) {
        // InstanceTableChainRecord::parse（走读那一种判）：链指针头部加密、压缩那几段不是第一版的取值，或 extent 偏移非 0，这一片损坏
        if (pointer_zero || !pointer_encryption_and_compression_fields_are_the_first_version(p.base, last + 2u)) {
            return out;
        }
        if (!pointer_extent_offset_is_zero(p.base, last + 2u)) {
            return out;
        }
        out.verdict = 1u;
        out.next = parse_node_pointer(p.base, last + 2u);
    } else {
        return out;
    }
    for (var r = 0u; r + 1u < p.record_count; r = r + 1u) {
        let o = packed_record_offset(p, r);
        if (rd8(p.base, o) != 0u || rd8(p.base, o + 21u) != 0u) {
            out.verdict = 2u;
            return out;
        }
    }
    return out;
}
// recovery::read_instance_table_chain：沿链读整张表；true = 读得出解得开
fn instance_table_chain_readable(root: RootFields) -> bool {
    var pointer = root.instance_table;
    var page_index = vec2<u32>(0u, 0u);
    for (var page = 0u; page < 256u; page = page + 1u) {
        // recovery::read_instance_table_chain：一片读不出 UnitUnreadable、解不开 UnitMalformed
        let s = read_unit_via_locations(pointer, DATA_UNIT_BYTES);
        if (s.version == NONE) {
            walk_failure_point = FAILURE_POINT_UNIT_UNREADABLE;
            return false;
        }
        let parsed = parse_instance_table_page(parse_packed_unit(s), page_index);
        if (parsed.verdict == 2u) {
            walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
            return false;
        }
        if (parsed.verdict == 0u) {
            return true;
        }
        pointer = parsed.next;
        page_index = add64(page_index, u64_of(1u));
    }
    undecidable = UNDECIDABLE_TOO_DEEP;
    return false;
}

// ───────────────────────── 走到第一个文件 ─────────────────────────

struct WalkResult {
    outcome: u32,
    failure: u32,
    content_matches: bool,
};
fn walk_failed(code: u32) -> WalkResult {
    var w: WalkResult;
    w.outcome = OUTCOME_FAILED;
    w.failure = code;
    w.content_matches = false;
    return w;
}
fn walk_no_file() -> WalkResult {
    var w: WalkResult;
    w.outcome = OUTCOME_NO_FILE;
    w.failure = 0u;
    w.content_matches = false;
    return w;
}
fn tree_kind_key_width(kind: u32) -> u32 {
    if (kind == TREE_KIND_EXTENT) { return 24u; }
    if (kind == TREE_KIND_INODE) { return 8u; }
    if (kind == TREE_KIND_ALLOCATION) { return 10u; }
    if (kind == TREE_KIND_ACCOUNTING) { return 22u; }
    if (kind == TREE_KIND_DELETED_INODES) { return DELETED_INODES_KEY_WIDTH; }
    // logged_ops 根（recovery.rs key_width_for_kind 的码 12：种类 2 + 签发 txg 8，判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1）
    if (kind == TREE_KIND_LOGGED_OPERATIONS) { return LOGGED_OPERATIONS_KEY_BYTES; }
    if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE || kind == TREE_KIND_RETIREMENT) { return CONTAINER_NUMBER_KEY_WIDTH; }
    return NONE;
}
// 进树表的第 position 个种类（发号次序，从 0 数）：1、2、3、4，跳过不进树表的 5，再 6..13
fn tree_kind_in_the_issuing_order(position: u32) -> u32 {
    if (position < 4u) {
        return position + 1u;
    }
    return position + 2u;
}
struct ContainerTreeFrame {
    n: Node,
    next_entry: u32,
};
var<private> container_tree_frames: array<ContainerTreeFrame, 12>;
struct ContainerTreeRead {
    ok: bool,
    mapped_units: u32,
};
// recovery::read_container_tree_index_nodes（墓碑树、谱系树）：从根往下读码 2 节点（提示读不出经映射回退），每个节点照树根同一组核自描述
// （tree_root_checked：树 ID、key 宽 8、出生身份、fsid、key 区间与首末条目相符）；根层级至少 1、每个节点非空；每条条目宽不窄于 120、
// 子指针头部是第一版的取值；层级 ℓ > 1 的条目身份引用全 0、指一个码 2 孩子：层级 ℓ − 1、首 key = 条目的分隔 key；层级 1 的条目指容器，只数不读。
// 交回进映射的单元数：每个码 2 节点（根在内）加每片容器。深度超过 container_tree_frames 的 12 层判不了。
// 只有一处读节点（根与孩子都经 pending 那一组进去）：每个调用点各展开一份。
// container_record_type 不是 NONE 时（建上一版那一样读者读整棵容器树，recovery::read_container_tree）层级 1 的条目连容器一起读：提示读不出经映射回退
// （码 3），解开、核容器头的身份与条目里的身份引用逐字相等、类型 = container_record_type、出生树 = 这棵树、容器号 = 分隔 key、记录宽 = container_record_width、
// 非空（不成立 UnitMalformed；记录宽等于字段表宽之后逐条解记录不会再不成）。CPU 先读完全部码 2 节点再按叶序读容器：调用方先照 NONE 走一遍、
// 过了再照记录类型走一遍（码 2 节点那几判在头一遍都过了，第二遍只会停在容器上）
fn read_container_tree_index_nodes(root_pointer: Ptr, root_pointer_base: u32, root_pointer_offset: u32, tree: vec2<u32>, root: RootFields, fsid_low: vec2<u32>,
    container_record_type: u32, container_record_width: u32) -> ContainerTreeRead {
    var out: ContainerTreeRead;
    out.ok = false;
    out.mapped_units = 0u;
    var nodes = 0u;
    var containers = 0u;
    var depth = 0u;
    var pending_pointer = root_pointer;
    var pending_pointer_base = root_pointer_base;
    var pending_pointer_offset = root_pointer_offset;
    var pending_separator_base = 0u;
    var pending_separator_offset = 0u;
    // 交进来的孩子是不是父节点的第一条、最后一条条目（判父节点的子树覆盖区间要它们）
    var pending_is_the_first_entry = false;
    var pending_is_the_last_entry = false;
    var enter = true;
    loop {
        if (enter) {
            enter = false;
            let m = read_via_hint_then_mapping(pending_pointer, pending_pointer_base, pending_pointer_offset, 2u, NODE_BYTES);
            if (m.failed) {
                return out;
            }
            let n = parse_index_node(m.s);
            if (!tree_root_checked(n, tree, CONTAINER_NUMBER_KEY_WIDTH, pending_pointer, root, fsid_low)) {
                return out;
            }
            // recovery::container_tree_node_of_level_one_is_tight_over_its_entries：层级 1 的节点（孩子是容器、key 是容器号）区间贴紧首末两条条目，
            // 与 tree_root_checked 对叶的那一判同一个失败点
            walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
            if (n.level == 1u && !key_range_is_tight_over_the_entries(n, CONTAINER_NUMBER_KEY_WIDTH)) {
                return out;
            }
            // recovery::read_container_tree_index_nodes：根层级 0 或空、孩子层级不合父层级减一、空、首 key 不是分隔 key、
            // 类型段 0 的身份引用不全 0 都是 UnitMalformed；条目窄 EntryWidthDiffersFromItsFieldTable；子指针头部 UnitMalformed
            walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
            if (n.entry_count == 0u) {
                return out;
            }
            if (depth == 0u) {
                if (n.level == 0u) {
                    return out;
                }
            } else {
                let parent_level = container_tree_frames[depth - 1u].n.level;
                if (n.level + 1u != parent_level
                    || !bytes_equal(n.base, n.smallest_offset, pending_separator_base, pending_separator_offset, CONTAINER_NUMBER_KEY_WIDTH)) {
                    return out;
                }
                // 父节点（层级 ≥ 2）头里的 key 区间是子树覆盖区间（D18 已定项 2）：第一个孩子头里的最小 key = 父头里的最小 key，
                // 最后一个孩子头里的最大 key = 父头里的最大 key（InvariantViolated）
                let parent = container_tree_frames[depth - 1u].n;
                if ((pending_is_the_first_entry && !bytes_equal(parent.base, parent.smallest_offset, n.base, n.smallest_offset, CONTAINER_NUMBER_KEY_WIDTH))
                    || (pending_is_the_last_entry && !bytes_equal(parent.base, parent.largest_offset, n.base, n.largest_offset, CONTAINER_NUMBER_KEY_WIDTH))) {
                    walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
                    return out;
                }
            }
            if (depth >= MAX_FRAMES) {
                undecidable = UNDECIDABLE_TOO_DEEP;
                return out;
            }
            container_tree_frames[depth].n = n;
            container_tree_frames[depth].next_entry = 0u;
            depth = depth + 1u;
            nodes = nodes + 1u;
            continue;
        }
        if (depth == 0u) {
            break;
        }
        let top = depth - 1u;
        let n = container_tree_frames[top].n;
        let e = container_tree_frames[top].next_entry;
        if (e >= n.entry_count) {
            depth = depth - 1u;
            continue;
        }
        container_tree_frames[top].next_entry = e + 1u;
        // inode 树内部条目宽不等于 120 就拒，宽了的同样拒（records.rs parse_inode_internal_entry，C867）
        if (n.entry_width != CONTAINER_TREE_ENTRY_BYTES) {
            walk_failure_point = FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE;
            return out;
        }
        let o = entry_offset(n, e);
        walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
        if (!record_pointer_head_is_the_first_version(n.base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET)) {
            return out;
        }
        if (n.level == 1u) {
            containers = containers + 1u;
            if (container_record_type != NONE) {
                let container_pointer = parse_node_pointer(n.base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET);
                let container_read = read_via_hint_then_mapping(container_pointer, n.base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET, 3u, DATA_UNIT_BYTES);
                if (container_read.failed) {
                    return out;
                }
                let container = parse_packed_unit(container_read.s);
                walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                if (!container.ok) {
                    return out;
                }
                // 条目里的身份引用：出生树 8、打包记录类型 2、容器号 8、容器出生代 8（偏移 8 起）；分隔 key 在偏移 0
                let identity_matches_the_entry = eq64(container.birth_tree, rd64(n.base, o + 8u)) && container.record_type == rd16(n.base, o + 16u)
                    && eq64(container.container, rd64(n.base, o + 18u)) && eq64(container.container_birth, rd64(n.base, o + 26u));
                if (!identity_matches_the_entry || container.record_type != container_record_type || !eq64(container.birth_tree, tree)
                    || !eq64(container.container, rd64(n.base, o)) || container.record_width != container_record_width || container.record_count == 0u) {
                    return out;
                }
            }
            continue;
        }
        // 类型段 0：身份引用（出生树 8、类型 2、容器号 8、容器出生代 8）全 0
        if (!all_zero(n.base, o + 8u, 26u)) {
            return out;
        }
        pending_pointer = parse_node_pointer(n.base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET);
        pending_pointer_base = n.base;
        pending_pointer_offset = o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET;
        pending_separator_base = n.base;
        pending_separator_offset = o;
        pending_is_the_first_entry = e == 0u;
        pending_is_the_last_entry = e + 1u == n.entry_count;
        enter = true;
    }
    out.ok = true;
    out.mapped_units = nodes + containers;
    return out;
}
fn expected_version_index(instance: u32, txg: vec2<u32>) -> u32 {
    let count = hdr(8u);
    for (var i = 0u; i < count; i = i + 1u) {
        if (tab(T_EXPECTED, i * EXPECTED_WORDS) == instance
            && tab(T_EXPECTED, i * EXPECTED_WORDS + 1u) == txg.x
            && tab(T_EXPECTED, i * EXPECTED_WORDS + 2u) == txg.y) {
            return i;
        }
    }
    return NONE;
}
// recovery::walk_to_file_in_the_unit_area_starting_at
// recovery::snapshot_tree_table_entries_paired_one_to_one（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1 与第七节岔路 1）：
// 码 14 与码 15 条数相等；按盘上次序每条码 15 的头 ID 恰是一条还没被前面的码 15 配上的码 14 的树 ID；码 14 至多 SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION 条；
// 每条码 14 的头 ID 等于表里第一条码 2 条目的树 ID（表里没有码 2 也不成立）。没有快照条目的表恒成立。四样不成立都是 UnitMalformed，先后不改结论。
// 条目宽调用方已判过是 200；一张树表的条目装不过 128 条（16 KiB 的码 2 节点、200 字节的条目），配上没有记在四个字的位图里
fn snapshot_tree_table_entries_are_paired_one_to_one(tree_table: Node) -> bool {
    var snapshot_inode_roots = 0u;
    var snapshot_extent_roots = 0u;
    var live_inode_tree = vec2<u32>(0u, 0u);
    var has_a_live_inode_tree = false;
    for (var e = 0u; e < tree_table.entry_count; e = e + 1u) {
        let o = entry_offset(tree_table, e);
        let kind = rd16(tree_table.base, o + 10u);
        if (kind == TREE_KIND_SNAPSHOT_INODE_ROOT) {
            snapshot_inode_roots = snapshot_inode_roots + 1u;
        } else if (kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT) {
            snapshot_extent_roots = snapshot_extent_roots + 1u;
        } else if (kind == TREE_KIND_INODE && !has_a_live_inode_tree) {
            live_inode_tree = rd64(tree_table.base, o);
            has_a_live_inode_tree = true;
        }
    }
    if (snapshot_inode_roots != snapshot_extent_roots) {
        return false;
    }
    var paired: array<u32, 4>;
    for (var word = 0u; word < 4u; word = word + 1u) {
        paired[word] = 0u;
    }
    for (var e = 0u; e < tree_table.entry_count; e = e + 1u) {
        let o = entry_offset(tree_table, e);
        if (rd16(tree_table.base, o + 10u) != TREE_KIND_SNAPSHOT_EXTENT_ROOT) {
            continue;
        }
        let head = rd64(tree_table.base, o + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
        var paired_position = NONE;
        for (var candidate = 0u; candidate < tree_table.entry_count && candidate < 128u; candidate = candidate + 1u) {
            let candidate_o = entry_offset(tree_table, candidate);
            if (rd16(tree_table.base, candidate_o + 10u) == TREE_KIND_SNAPSHOT_INODE_ROOT && eq64(rd64(tree_table.base, candidate_o), head)
                && (paired[candidate / 32u] & (1u << (candidate % 32u))) == 0u) {
                paired_position = candidate;
                break;
            }
        }
        if (paired_position == NONE) {
            return false;
        }
        paired[paired_position / 32u] = paired[paired_position / 32u] | (1u << (paired_position % 32u));
    }
    if (snapshot_inode_roots > SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION) {
        return false;
    }
    for (var e = 0u; e < tree_table.entry_count; e = e + 1u) {
        let o = entry_offset(tree_table, e);
        if (rd16(tree_table.base, o + 10u) == TREE_KIND_SNAPSHOT_INODE_ROOT
            && (!has_a_live_inode_tree || !eq64(rd64(tree_table.base, o + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET), live_inode_tree))) {
            return false;
        }
    }
    return true;
}
// 读回来的分配记录（R_ALLOCATION）照 recovery 那三判逐条判：allocation_records_fit_the_pool_geometry、
// allocation_records_are_one_placement_per_device_on_every_device、allocation_record_generations_and_spans_are_judged，次序照它们；交回第一处判不过的失败点码，
// 都过交 FAILURE_POINT_NONE。冷走读与建上一版各调一处
fn allocation_records_judged_against_the_pool_and_the_root(root: RootFields, cfg: SysConfig) -> u32 {
    // 分配记录落在池几何里：盘在池里、槽不低于单元区起点、跨度不越过单元区末尾、同盘不相交（记录按 (盘, 槽) 升序，相邻比即可）
    // recovery::allocation_records_fit_the_pool_geometry 逐条：盘不在池里、槽在单元区起点之下 AllocationRecordOutsideThePoolGeometry，
    // 盘末尾在单元区起点之前 DeviceEndsBeforeTheUnitAreaStart（placement_lies_in_the_unit_area_of_its_device 判到这一条时才判），
    // 跨度越过单元区末尾、同盘两条罩住同一个槽 AllocationRecordOutsideThePoolGeometry
    let device_slots = shr64(device_bytes(), SLOT_SHIFT);
    for (var r = 0u; r < allocation_count; r = r + 1u) {
        let device = sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS);
        let slot = vec2<u32>(sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS + 1u), sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS + 2u));
        let span = sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS + 3u) & 0x7FFFu;
        if (!device_is_in_the_pool(device)) {
            return FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY;
        }
        if (lt64(slot, cfg.unit_area_start_slot)) {
            return FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY;
        }
        if (lt64(device_slots, cfg.unit_area_start_slot)) {
            return FAILURE_POINT_DEVICE_ENDS_BEFORE_THE_UNIT_AREA_START;
        }
        if (lt64(device_slots, add64(slot, u64_of(span)))) {
            return FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY;
        }
        if (r > 0u) {
            let previous_device = sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS);
            let previous_slot = vec2<u32>(sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS + 1u), sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS + 2u));
            let previous_span = sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS + 3u) & 0x7FFFu;
            if (previous_device == device && lt64(slot, add64(previous_slot, u64_of(previous_span)))) {
                return FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY;
            }
        }
    }
    // 每个落点每盘各一条：各盘的 (槽, 跨度, 代, 已释放) 集合相同、同盘槽号唯一（升序已判过）、不少于 10 个落点
    let device_count = hdr(3u);
    var first_device_start = NONE;
    var first_device_count = 0u;
    var devices_with_records = 0u;
    var r = 0u;
    while (r < allocation_count) {
        let device = sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS);
        var end = r;
        while (end < allocation_count && sw(R_ALLOCATION + end * ALLOCATION_RECORD_WORDS) == device) {
            end = end + 1u;
        }
        devices_with_records = devices_with_records + 1u;
        if (first_device_start == NONE) {
            first_device_start = r;
            first_device_count = end - r;
        } else {
            if (end - r != first_device_count) {
                return FAILURE_POINT_INVARIANT_VIOLATED;
            }
            for (var i = 0u; i < first_device_count; i = i + 1u) {
                let a = R_ALLOCATION + (first_device_start + i) * ALLOCATION_RECORD_WORDS;
                let b = R_ALLOCATION + (r + i) * ALLOCATION_RECORD_WORDS;
                if (sw(a + 1u) != sw(b + 1u) || sw(a + 2u) != sw(b + 2u) || sw(a + 3u) != sw(b + 3u) || sw(a + 4u) != sw(b + 4u) || sw(a + 5u) != sw(b + 5u)) {
                    return FAILURE_POINT_INVARIANT_VIOLATED;
                }
            }
        }
        r = end;
    }
    if (devices_with_records != device_count || first_device_count < 10u) {
        return FAILURE_POINT_INVARIANT_VIOLATED;
    }
    // 分配记录跨度非 0、代不晚于根
    for (var r2 = 0u; r2 < allocation_count; r2 = r2 + 1u) {
        let span = sw(R_ALLOCATION + r2 * ALLOCATION_RECORD_WORDS + 3u) & 0x7FFFu;
        let generation = vec2<u32>(sw(R_ALLOCATION + r2 * ALLOCATION_RECORD_WORDS + 4u), sw(R_ALLOCATION + r2 * ALLOCATION_RECORD_WORDS + 5u));
        if (span == 0u || lt64(root.txg, generation)) {
            return FAILURE_POINT_INVARIANT_VIOLATED;
        }
    }
    return FAILURE_POINT_NONE;
}
// 读回来的记账条目（R_ACCOUNTING）照 recovery 那三判：every_accounting_entry_is_as_wide_as_its_field_table、
// accounting_entry_parsed_and_judged_against_the_root、accounting_entry_count_is_three_plus_six_per_device，次序照它们；交回第一处判不过的失败点码，
// 都过交 FAILURE_POINT_NONE。冷走读与建上一版各调一处
fn accounting_entries_judged_against_the_root(root: RootFields) -> u32 {
    let device_count = hdr(3u);
    // 记账条目：先逐条判宽 = 34，窄了宽了都拒（recovery::every_accounting_entry_is_as_wide_as_its_field_table，EntryWidthDiffersFromItsFieldTable；
    // records::AccountingEntry::parse 不等于 34 交 None，C780，用户 2026-10-18 定「宽于字段表也拒」），
    // 再逐条判代不晚于根、seq ≠ 0（accounting_entry_parsed_and_judged_against_the_root，InvariantViolated）；
    // 次序同 recovery::walk_to_file_in_the_unit_area_starting_at：分配记录的代与跨度、记账条目宽、逐条判记账条目，之后才数记账条目与映射条目
    for (var a = 0u; a < accounting_count; a = a + 1u) {
        if (sw(R_ACCOUNTING + a * 3u + 2u) != 34u) {
            return FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE;
        }
    }
    for (var a = 0u; a < accounting_count; a = a + 1u) {
        let base = sw(R_ACCOUNTING + a * 3u);
        let o = sw(R_ACCOUNTING + a * 3u + 1u);
        if (lt64(root.txg, rd64(base, o + 14u)) || rd32(base, o + 30u) == 0u) {
            return FAILURE_POINT_INVARIANT_VIOLATED;
        }
    }
    // 记账条目数 3 + 6 × 盘数，inode 号水位的台阶行不算（recovery.rs accounting_entry_count_is_three_plus_six_per_device：
    // 标签 12、设备段 0xFFFF_FFFF、代段早于根的 txg 的行；条目宽不等于 34 的解不开、照算，与 AccountingEntry::parse 交 None 那一支相同；
    // 走到这里的条目都过了上面判宽那一道，宽都是 34）
    var accounting_entries_other_than_the_inode_number_watermark_steps = 0u;
    for (var a = 0u; a < accounting_count; a = a + 1u) {
        let base = sw(R_ACCOUNTING + a * 3u);
        let o = sw(R_ACCOUNTING + a * 3u + 1u);
        let width = sw(R_ACCOUNTING + a * 3u + 2u);
        let is_an_inode_number_watermark_step = width == 34u && rd16(base, o) == 12u && rd32(base, o + 10u) == NONE
            && lt64(rd64(base, o + 14u), root.txg);
        if (!is_an_inode_number_watermark_step) {
            accounting_entries_other_than_the_inode_number_watermark_steps = accounting_entries_other_than_the_inode_number_watermark_steps + 1u;
        }
    }
    if (accounting_entries_other_than_the_inode_number_watermark_steps != 3u + 6u * device_count) {
        return FAILURE_POINT_INVARIANT_VIOLATED;
    }
    return FAILURE_POINT_NONE;
}
// recovery::tree_table_entries_each_kind_at_most_once（连同它判的 snapshot_tree_table_entries_paired_one_to_one）：条目宽 200、每条照
// TreeTableEntry::parse_with_destruction_mark 解得开（条目长度、flags、预留 76、根指针头部加密压缩那几段与 extent 偏移）、码 14 / 15 之外每种至多一条、
// 快照条目一一配对。不成立交 false：冷走读、建上一版报 UnitMalformed，只读挂载打开报 TreeTableEntryMalformed 或同一个 UnitMalformed，都是内容拒收。
// 走读、只读挂载打开、建上一版三样读者各调一处
fn tree_table_entries_each_kind_at_most_once(tree_table: Node) -> bool {
    if (tree_table.entry_width != 200u) {
        return false;
    }
    var kinds_seen = 0u;
    for (var e = 0u; e < tree_table.entry_count; e = e + 1u) {
        let o = entry_offset(tree_table, e);
        // TreeTableEntry::parse_with_destruction_mark：flags 是 0，或码 14 条目上恰是位 0（销毁中）；别的位、别的种类上的位 0 拒收
        let flags = rd16(tree_table.base, o + TREE_TABLE_ENTRY_FLAGS_OFFSET);
        let flags_are_accepted = flags == 0u
            || (flags == TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED && rd16(tree_table.base, o + 10u) == TREE_KIND_SNAPSHOT_INODE_ROOT);
        if (rd16(tree_table.base, o + 8u) != 200u || !flags_are_accepted || !all_zero(tree_table.base, o + 124u, 76u)) {
            return false;
        }
        // TreeTableEntry::parse：树根指针头部加密、压缩那几段不是第一版的取值，这条条目损坏
        if (!pointer_encryption_and_compression_fields_are_the_first_version(tree_table.base, o + 14u)) {
            return false;
        }
        // TreeTableEntry::parse：树根指针头部 extent 偏移非 0，这条条目损坏（与条目长度、flags、预留 76 同在读任何一棵树之前拒）
        if (!pointer_extent_offset_is_zero(tree_table.base, o + 14u)) {
            return false;
        }
        let kind = rd16(tree_table.base, o + 10u);
        // 码 14、15 两种快照条目一个快照一对、可以有多条：不进「每种至多一条」，下面按头 ID 一一配对判
        if (kind == TREE_KIND_SNAPSHOT_INODE_ROOT || kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT) {
            continue;
        }
        // tree_table_entries_each_kind_at_most_once 的 kinds_seen 是整个 u16 的集合：种类 < 32 的记进位图，
        // 种类 ≥ 32 的（第一版写者写不出、坏盘上才有）逐条与前面各条比（树表条目数不超过一个码 2 节点装得下的，与走读内核 ③ 的两两比同一量级）。
        // 走读内核的 ③（crash_judge_checker.wgsl 判 I-9.16 那一段）同样比整个 u16，两个内核在种类 ≥ 32 的重复上都拦得到
        if (kind < 32u) {
            if ((kinds_seen & (1u << kind)) != 0u) {
                return false;
            }
            kinds_seen = kinds_seen | (1u << kind);
        } else {
            for (var earlier = 0u; earlier < e; earlier = earlier + 1u) {
                if (rd16(tree_table.base, entry_offset(tree_table, earlier) + 10u) == kind) {
                    return false;
                }
            }
        }
    }
    if (!snapshot_tree_table_entries_are_paired_one_to_one(tree_table)) {
        return false;
    }
    return true;
}
// recovery::tree_table_entries_ascend_by_tree_identifier_and_in_the_issuing_order（I-9.16 ① ②）：不成立交 false（InvariantViolated）
fn tree_table_entries_ascend_by_tree_identifier_and_in_the_issuing_order(tree_table: Node) -> bool {
    // I-9.16 ①：树 ID 严格升序
    for (var e = 1u; e < tree_table.entry_count; e = e + 1u) {
        let previous = rd64(tree_table.base, entry_offset(tree_table, e - 1u));
        let current = rd64(tree_table.base, entry_offset(tree_table, e));
        if (!lt64(previous, current)) {
            return false;
        }
    }
    // I-9.16 ②：按发号次序 1,2,3,4,6,7,8,9,10,11,12,13（recovery.rs 的 TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER）相邻两棵严格升序
    var previous_tree = vec2<u32>(0u, 0u);
    var have_previous = false;
    for (var k = 0u; k < TREE_KINDS_OF_THE_TREE_TABLE; k = k + 1u) {
        let kind = tree_kind_in_the_issuing_order(k);
        for (var e = 0u; e < tree_table.entry_count; e = e + 1u) {
            let o = entry_offset(tree_table, e);
            if (rd16(tree_table.base, o + 10u) == kind) {
                let tree = rd64(tree_table.base, o);
                if (have_previous && !lt64(previous_tree, tree)) {
                    return false;
                }
                previous_tree = tree;
                have_previous = true;
                break;
            }
        }
    }
    return true;
}
fn walk_to_file(root: RootFields, cfg: SysConfig, expected: u32) -> WalkResult {
    let fsid_low = vec2<u32>(cfg.fsid.x, cfg.fsid.y);
    // 实例表：根直接持有第 0 片
    let instance_table_slice = read_unit_via_locations(root.instance_table, DATA_UNIT_BYTES);
    if (instance_table_slice.version == NONE) {
        return walk_failed(FAILURE_POINT_UNIT_UNREADABLE);
    }
    let instance_table = parse_packed_unit(instance_table_slice);
    if (!instance_table.ok) {
        return walk_failed(FAILURE_POINT_UNIT_MALFORMED);
    }
    if (instance_table.record_type != 4u || instance_table.record_count == 0u) {
        return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
    }
    let first_page = parse_instance_table_page(instance_table, vec2<u32>(0u, 0u));
    if (first_page.verdict == 2u) {
        return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
    }
    if (first_page.verdict == 1u) {
        if (!instance_table_chain_readable(root)) {
            return walk_failed(walk_failure_point);
        }
    }
    // 树表
    let tree_table_slice = read_unit_via_locations(root.tree_table, NODE_BYTES);
    if (tree_table_slice.version == NONE) {
        return walk_failed(FAILURE_POINT_UNIT_UNREADABLE);
    }
    let tree_table = parse_index_node(tree_table_slice);
    if (!tree_table.ok) {
        return walk_failed(FAILURE_POINT_UNIT_MALFORMED);
    }
    if (tree_table.key_width != 8u) {
        return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
    }
    if (tree_table.entry_count == 0u) {
        return walk_no_file();
    }
    // 条目逐条解开、每种至多一条
    if (!tree_table_entries_each_kind_at_most_once(tree_table)) {
        return walk_failed(FAILURE_POINT_UNIT_MALFORMED);
    }
    if (!tree_table_entries_ascend_by_tree_identifier_and_in_the_issuing_order(tree_table)) {
        return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
    }
    // 中央映射树：走读到 TreeRoots 那一步总要读它；先读也罢（读得出读不出与次序无关，见 crash_judge_gpu.rs 的说明）
    // 映射树读不出不在这里报：CPU 到第一次有提示读不出、要经映射回退时才读它（那一刻报它读不出的那一处），走完各棵树、缺不缺树判完之后再要它一次
    // （recovery 的 CentralMappingTreeReadOnFirstUse::locations_of_key 与 into_tree）；读不出的那一处记在 mapping_tree_failure_point
    mapping_tree_failure_point = FAILURE_POINT_NONE;
    let mapping = read_code_two_tree(root.mapping_root, 0u, 0u, root.mapping_root.birth_tree, 1u, root, fsid_low);
    if (!mapping.ok) {
        if (undecidable != 0u) {
            return walk_failed(FAILURE_POINT_NONE);
        }
        mapping_tree_failure_point = walk_failure_point;
    }
    var have_extent = false;
    var have_inode = false;
    var have_allocation = false;
    var have_accounting = false;
    var extent_tree = vec2<u32>(0u, 0u);
    var extent_nodes = 0u;
    var allocation_nodes = 0u;
    var accounting_nodes = 0u;
    var deadlist_nodes = 0u;
    var deadlist_entries = 0u;
    var have_deleted_inodes = false;
    var have_logged_operations = false;
    var container_tree_units = 0u;
    var inode_root = bad_node();
    var inode_root_pointer = zero_pointer();
    for (var e = 0u; e < tree_table.entry_count; e = e + 1u) {
        let o = entry_offset(tree_table, e);
        let tree = rd64(tree_table.base, o);
        // I-7.8：树 ID 低于水位
        if (!lt64(tree, root.watermark)) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
        let kind = rd16(tree_table.base, o + 10u);
        // 只读快照的两种条目：冷走读读的是活头那一版的文件，跳过它们（recovery.rs walk_to_file 在判过树 ID 水位之后 continue）
        if (kind == TREE_KIND_SNAPSHOT_INODE_ROOT || kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT) {
            continue;
        }
        let pointer = parse_node_pointer(tree_table.base, o + 14u);
        if (pointer.all_zero) {
            continue;
        }
        if (kind == TREE_KIND_ALLOCATION) {
            let read = read_allocation_tree(pointer, tree_table.base, o + 14u, tree, root, fsid_low);
            if (!read.ok) {
                return walk_failed(walk_failure_point);
            }
            allocation_nodes = read.node_count;
            have_allocation = true;
            continue;
        }
        if (kind == TREE_KIND_EXTENT) {
            let read = read_extent_tree_of_the_first_file(pointer, tree_table.base, o + 14u, tree, root, fsid_low);
            if (!read.ok) {
                return walk_failed(walk_failure_point);
            }
            extent_nodes = read.node_count;
            extent_tree = tree;
            have_extent = true;
            continue;
        }
        // 记账树与 deadlist 树（第三棵多层码 2 树，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）：同一个读法整棵读回来，
        // 一个调用点（GPU 程序把每个调用点展开成一份拷贝）；deadlist 的节点与叶条目各占映射条目（条目留存的那几条，判决 S3 的 α），数映射条目时要它们
        if (kind == TREE_KIND_ACCOUNTING || kind == TREE_KIND_DEADLIST) {
            let read = read_code_two_tree(pointer, tree_table.base, o + 14u, tree, select(0u, KEY_SCHEMA_DEADLIST, kind == TREE_KIND_DEADLIST), root, fsid_low);
            if (!read.ok) {
                return walk_failed(walk_failure_point);
            }
            if (kind == TREE_KIND_DEADLIST) {
                deadlist_nodes = read.node_count;
                deadlist_entries = read.entry_count;
                continue;
            }
            accounting_nodes = read.node_count;
            have_accounting = true;
            continue;
        }
        // 墓碑树、谱系树与退役树：从根往下读码 2 节点、数进映射的节点与容器，不读容器（它们的内容不影响读回的文件）
        if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE || kind == TREE_KIND_RETIREMENT) {
            let read = read_container_tree_index_nodes(pointer, tree_table.base, o + 14u, tree, root, fsid_low, NONE, 0u);
            if (!read.ok) {
                return walk_failed(walk_failure_point);
            }
            container_tree_units = container_tree_units + read.mapped_units;
            continue;
        }
        let key_width = tree_kind_key_width(kind);
        if (key_width == NONE) {
            return walk_failed(FAILURE_POINT_UNIT_MALFORMED);
        }
        // inode 树根、deleted_inodes 根与 logged_ops 根（进映射的树根：提示读不出经映射回退，再核自描述；logged_ops 根只数进映射条目数，
        // 条目不在冷走读里解，recovery.rs walk_to_file 同）
        let m = read_via_hint_then_mapping(pointer, tree_table.base, o + 14u, 2u, NODE_BYTES);
        if (m.failed) {
            return walk_failed(walk_failure_point);
        }
        let n = parse_index_node(m.s);
        if (!tree_root_checked(n, tree, key_width, pointer, root, fsid_low)) {
            return walk_failed(walk_failure_point);
        }
        if (kind == TREE_KIND_INODE) {
            inode_root = n;
            inode_root_pointer = pointer;
            have_inode = true;
        }
        if (kind == TREE_KIND_DELETED_INODES) {
            have_deleted_inodes = true;
        }
        if (kind == TREE_KIND_LOGGED_OPERATIONS) {
            have_logged_operations = true;
        }
    }
    if (!have_extent || !have_inode || !have_allocation || !have_accounting) {
        return walk_failed(FAILURE_POINT_UNIT_MALFORMED);
    }
    if (mapping_tree_failure_point != FAILURE_POINT_NONE) {
        return walk_failed(mapping_tree_failure_point);
    }
    let allocation_failure = allocation_records_judged_against_the_pool_and_the_root(root, cfg);
    if (allocation_failure != FAILURE_POINT_NONE) {
        return walk_failed(allocation_failure);
    }
    let accounting_failure = accounting_entries_judged_against_the_root(root);
    if (accounting_failure != FAILURE_POINT_NONE) {
        return walk_failed(accounting_failure);
    }
    // 映射条目数 = 1 + extent 节点 + 分配记录树节点 + 记账树节点 + deadlist 树节点 + deadlist 条目数 + inode 叶容器数 + 数据单元数
    // + 删除那两棵树与谱系树的节点（recovery.rs 的 deletion_tree_nodes：deleted_inodes 根有就 1、logged_ops 根有就 1，墓碑树、谱系树各自每个码 2 节点加每片容器；
    // deadlist 那两项是 mapping_entry_count_is_one_per_mapped_unit 的 deadlist_tree_nodes 与 mapping_entries_kept_for_the_snapshots）
    let deletion_tree_nodes = select(0u, 1u, have_deleted_inodes) + select(0u, 1u, have_logged_operations) + container_tree_units;
    // 数据单元数是每个文件的（第一个文件与第一个文件之外各文件，recovery 的 data_units_of_every_file）
    if (mapping_count != 1u + extent_nodes + allocation_nodes + accounting_nodes + deadlist_nodes + deadlist_entries + inode_root.entry_count + data_count
        + data_units_of_the_files_other_than_the_first + deletion_tree_nodes) {
        return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
    }
    // inode 树：根层级 1，每条内部条目 → 叶容器 → 记录
    if (inode_root.level != 1u) {
        return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
    }
    // 条目宽只在有条目时核：recovery.rs 逐条 inode_internal_entry_parsed_and_judged 时才核宽，零条目的根（unlink 删掉树里最后一条记录之后）
    // 条目宽是什么都不拒；有条目时宽不等于 120 就拒，宽了的同样拒（C867）
    if (inode_root.entry_count > 0u && inode_root.entry_width != CONTAINER_TREE_ENTRY_BYTES) {
        return walk_failed(FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE);
    }
    var found_inode = false;
    var inode_nlink = 0u;
    var inode_size = vec2<u32>(0u, 0u);
    var inode_object_birth = vec2<u32>(0u, 0u);
    var inode_numbers_read_back = InodeNumbersReadBack(false, vec2<u32>(0u, 0u), vec2<u32>(0u, 0u));
    for (var e = 0u; e < inode_root.entry_count; e = e + 1u) {
        let o = entry_offset(inode_root, e);
        let separator = rd64(inode_root.base, o);
        let identity_birth_tree = rd64(inode_root.base, o + 8u);
        let identity_type = rd16(inode_root.base, o + 16u);
        let identity_container = rd64(inode_root.base, o + 18u);
        let identity_birth = rd64(inode_root.base, o + 26u);
        // recovery::inode_internal_entry_parsed_and_judged：先解条目（子指针头部 UnitMalformed），再判类型段（InvariantViolated）
        let child = parse_node_pointer(inode_root.base, o + 34u);
        // records::parse_inode_internal_entry_or_refusal：子指针头部加密、压缩那几段不是第一版的取值，这条内部条目损坏
        if (!child.head_ok) {
            return walk_failed(FAILURE_POINT_UNIT_MALFORMED);
        }
        // records::parse_inode_internal_entry_or_refusal：子指针头部 extent 偏移非 0，这条内部条目损坏
        if (!child.extent_offset_zero) {
            return walk_failed(FAILURE_POINT_UNIT_MALFORMED);
        }
        if (identity_type != 2u) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
        let m = read_via_hint_then_mapping(child, inode_root.base, o + 34u, 3u, DATA_UNIT_BYTES);
        if (m.failed) {
            return walk_failed(walk_failure_point);
        }
        let leaf = parse_packed_unit(m.s);
        if (!leaf.ok) {
            return walk_failed(FAILURE_POINT_UNIT_MALFORMED);
        }
        if (!eq64(leaf.birth_tree, identity_birth_tree) || leaf.record_type != identity_type || !eq64(leaf.container, identity_container)
            || !eq64(leaf.container_birth, identity_birth) || leaf.record_width != 140u || !eq64(leaf.fsid, fsid_low)) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
        if (!eq64(child.birth_tree, identity_birth_tree) || leaf.birth_sequence != child.tail.x) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
        if (lt64(root.txg, leaf.birth_txg) || leaf.instance != child.instance) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
        for (var rec = 0u; rec < leaf.record_count; rec = rec + 1u) {
            let ro = packed_record_offset(leaf, rec);
            if (!inode_record_parses_by_the_core_reader(leaf.base, ro)) {
                return walk_failed(FAILURE_POINT_UNIT_MALFORMED);
            }
            let inode = rd64(leaf.base, ro);
            if (lt64(inode, separator) || lt64(inode, identity_container)) {
                return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
            }
            // records::InodeRecord::truncation_field_fits_the_pool：不带 incompat 位 2 的池里截断字段必须全零（recovery 的 I-9.7 那一判）
            if (!inode_record_truncation_field_fits_the_pool(leaf.base, ro, cfg.carries_the_inode_record_truncation_field)) {
                return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
            }
            inode_numbers_read_back = inode_numbers_read_back_with(inode_numbers_read_back, inode);
            if (eq64(inode, u64_of(FIRST_INODE_NUMBER))) {
                found_inode = true;
                inode_object_birth = rd64(leaf.base, ro + INODE_RECORD_OBJECT_BIRTH_OFFSET);
                inode_nlink = rd32(leaf.base, ro + INODE_RECORD_NLINK_OFFSET);
                inode_size = rd64(leaf.base, ro + INODE_RECORD_SIZE_OFFSET);
            }
        }
    }
    // inode 树根头里的 key 区间是子树覆盖区间（recovery.rs walk_to_file：读完叶容器、取第一个文件那条记录之前判）
    if (!inode_tree_root_key_range_is_the_subtree_coverage(inode_root, inode_numbers_read_back)) {
        return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
    }
    if (!found_inode) {
        return walk_no_file();
    }
    // 删除中的 inode（nlink = 0）：从意图那次发布起冷走读答「没有文件」，与 recovery.rs 冷走读同一判、同一处（取到第一个文件那条记录之后、
    // 核单元数之前；D8（核心索引结构） 已定项 16 第 4 条）
    if (inode_nlink == NLINK_OF_AN_INODE_BEING_DELETED) {
        return walk_no_file();
    }
    // 数据单元数按上段叶条目的标签对 size（core 冷走读的 data_unit_count_matches_the_inode_size，即 write_request_split::data_units_read_within_the_file_size）：
    // size 按净荷容量算出的单元数是 max(1, ⌈size ÷ 32634⌉)；标签 1、2 存在的单元不少于它，多出来的是截断中间态里 size 之外还挂着的单元，
    // 不读、不判声明长度（判决 m4-trunc-r1 第三节 K2）；标签 3、4、0 少于它，存在的单元之后到 size 是洞、读出 0、不读盘（K5）
    let payload = u64_of(DATA_UNIT_PAYLOAD_CAPACITY);
    var units_implied_by_the_size = divmod64(add64_saturating(inode_size, sub64(payload, u64_of(1u))), payload).quotient;
    if (is_zero64(units_implied_by_the_size)) {
        units_implied_by_the_size = u64_of(1u);
    }
    let tail_is_a_hole = first_file_upper_leaf_tag == 0u || first_file_upper_leaf_tag == 3u || first_file_upper_leaf_tag == 4u;
    if (tail_is_a_hole) {
        if (!lt64(u64_of(data_count), units_implied_by_the_size)) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
    } else if (lt64(u64_of(data_count), units_implied_by_the_size)) {
        return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
    }
    // size 之内要读、要核的单元数：有洞时是存在的全部，没有洞时是 size 算出的那几个（不多于 data_count，装得进 32 位）
    let data_units_within_the_size = select(units_implied_by_the_size.x, data_count, tail_is_a_hole);
    var content_matches = true;
    var chunk_first = 0u;
    var chunk_count = 0u;
    if (expected != NONE) {
        chunk_first = tab(T_EXPECTED, expected * EXPECTED_WORDS + 3u);
        chunk_count = tab(T_EXPECTED, expected * EXPECTED_WORDS + 4u);
        // 期望那一版的内容按写侧的切法切成 max(1, ⌈内容长度 ÷ 净荷⌉) 个单元（crash_judge_tables 的 expected_version_chunks），读出的文件是 size 那么长：
        // 两边单元数不同，内容就不同
        if (!eq64(u64_of(chunk_count), units_implied_by_the_size)) {
            content_matches = false;
        }
    }
    // 数据指针按块读（R_DATA 里是 data_block_first 起的 data_block_count 条）：读到这一块末尾，从下段根一层层往下找含下一个单元的那片叶，
    // 每一步读一个节点；找下一块时此刻要读的那条节点指针在哪（字节基、偏移），不在找是 NONE
    var next_block_pointer_base = NONE;
    var next_block_pointer_offset = 0u;
    var i = 0u;
    // 迭代上界：每个单元读一次，加上每找一块往下读的层数（每块至少一个单元，层数不过 MAX_FRAMES，内部节点每一步往下一层）。
    // 跨轮带着的：i 只增，找块时 next_block_pointer_* 一层层往下、找到就回 NONE；data_block_first ≤ i < data_block_first + data_block_count 时读单元
    loop {
        if (i >= data_units_within_the_size) {
            break;
        }
        let finding_the_next_block = i >= data_block_first + data_block_count;
        if (finding_the_next_block && next_block_pointer_base == NONE) {
            // 从下段根起：第一个文件那一条上段叶条目偏移 25 起是下段根指针（一块装不下的文件有下段，内联的只有一个单元）
            next_block_pointer_base = first_file_upper_entry_base;
            next_block_pointer_offset = first_file_upper_entry_offset + 25u;
        }
        var base = next_block_pointer_base;
        var o = next_block_pointer_offset;
        var pointer: Ptr;
        var unit_class = 2u;
        var unit_bytes = NODE_BYTES;
        if (finding_the_next_block) {
            pointer = parse_node_pointer(base, o);
        } else {
            base = sw(R_DATA + (i - data_block_first) * 2u);
            o = sw(R_DATA + (i - data_block_first) * 2u + 1u);
            pointer = parse_data_pointer(base, o);
            unit_class = 1u;
            unit_bytes = DATA_UNIT_BYTES;
        }
        // 读下段节点与读数据单元同一个调用点（GPU 程序把每个调用点展开成一份拷贝）
        let m = read_via_hint_then_mapping(pointer, base, o, unit_class, unit_bytes);
        if (finding_the_next_block) {
            let step = find_the_next_block_of_data_pointers(m, i);
            if (undecidable != 0u) {
                return walk_failed(FAILURE_POINT_NONE);
            }
            next_block_pointer_base = step.pointer_base;
            next_block_pointer_offset = step.pointer_offset;
            if (step.is_the_block_found) {
                next_block_pointer_base = NONE;
            }
            continue;
        }
        if (m.failed) {
            return walk_failed(walk_failure_point);
        }
        let unit = parse_data_unit(m.s);
        if (!unit.ok) {
            return walk_failed(FAILURE_POINT_UNIT_MALFORMED);
        }
        let first_file_byte = mul64_64_saturating(u64_of(i), payload);
        if (!eq64(unit.tree, extent_tree) || !eq64(unit.object, u64_of(FIRST_INODE_NUMBER)) || !eq64(unit.anchor_offset, first_file_byte)) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
        if (!eq64(unit.object_birth, inode_object_birth)) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
        if (!eq64(unit.birth_txg, pointer.birth_txg) || unit.instance != pointer.instance || !eq64(unit.transaction, pointer.tail) || !eq64(unit.fsid, fsid_low)) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
        var expected_length = sub64_saturating(inode_size, first_file_byte);
        if (lt64(payload, expected_length)) {
            expected_length = payload;
        }
        if (!eq64(u64_of(unit.declared_length), expected_length)) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
        if (!unit.padding_zero) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
        if (expected != NONE && i < chunk_count) {
            if (tab(T_CHUNKS, (chunk_first + i) * 2u) != unit.payload_crc || tab(T_CHUNKS, (chunk_first + i) * 2u + 1u) != unit.declared_length) {
                content_matches = false;
            }
        }
        i = i + 1u;
    }
    // 洞（存在的单元之后到 size）不读盘，读出 0（core 的 content_padded_with_the_hole_up_to_the_size）。每个洞的位置照期望那一版的那个单元比：
    // 声明长度是 size 切到这一段的长度，载荷区 CRC 是内核现算的全零那一份——载荷区罩预留位、载荷与补到单元末尾的 0，洞的载荷全是 0，
    // 所以每个洞的位置都是同一个数（判决 m4-trunc-r1 第六节 S5：判表不加「这一块全零」位、不加从镜像算出的列，期望只来自版本表）。
    // 只比到期望那一版的单元数为止：两边单元数不同上面已判内容不同，再往后比不出别的，也不让一个很大的 size 拖长这一圈
    if (expected != NONE && data_units_within_the_size < chunk_count) {
        let payload_region_crc_of_a_hole = crc32c_of_zero_bytes(DATA_UNIT_PAYLOAD_REGION_BYTES);
        for (var position = data_units_within_the_size; position < chunk_count; position = position + 1u) {
            let first_file_byte = mul64_64_saturating(u64_of(position), payload);
            var length_of_the_hole_here = sub64_saturating(inode_size, first_file_byte);
            if (lt64(payload, length_of_the_hole_here)) {
                length_of_the_hole_here = payload;
            }
            if (tab(T_CHUNKS, (chunk_first + position) * 2u) != payload_region_crc_of_a_hole
                || !eq64(u64_of(tab(T_CHUNKS, (chunk_first + position) * 2u + 1u)), length_of_the_hole_here)) {
                content_matches = false;
            }
        }
    }
    var w: WalkResult;
    w.outcome = OUTCOME_FILE_READ;
    w.failure = 0u;
    w.content_matches = content_matches;
    return w;
}


// ───────────────────────── 每个状态的开场：表的目录、草稿区、持久掩码 ─────────────────────────

fn begin_state(local: u32) {
    for (var t = 0u; t < TABLE_COUNT; t = t + 1u) {
        B[t] = tables[t * 2u];
    }
    scratch_base = local * dispatch.scratch_words;
    undecidable = 0u;
    record_count = 0u;
    root_count = 0u;
    mapping_count = 0u;
    mapping_leaf_count = 0u;
    accounting_count = 0u;
    allocation_count = 0u;
    seen_count = 0u;
    data_count = 0u;
    data_units_of_the_files_other_than_the_first = 0u;
    data_block_first = 0u;
    data_block_count = 0u;
    upper_count = 0u;
    row_count = 0u;
    deadlist_leaf_count = 0u;
    extent_tree_records_its_files = false;
    files_of_the_extent_tree = 0u;
    rebuild_judges_the_pointers_of_the_version = false;
    some_pointer_of_the_version_does_not_ascend_by_device = false;
    some_mapping_position_of_the_version_does_not_ascend_by_device = false;
    var ordinal_lo = dispatch.block_start_lo + local;
    var ordinal_hi = dispatch.block_start_hi;
    if (ordinal_lo < dispatch.block_start_lo) {
        ordinal_hi = ordinal_hi + 1u;
    }
    for (var word = 0u; word < PERSISTED_MASK_WORDS; word = word + 1u) {
        persisted[word] = 0u;
    }
    // 给定状态表（crash_judge_tables::JudgeStateSource::GivenPersistedSets）：这张表不空，第 k 个状态就是表里第 k 个持久掩码
    // （每个 ⌈枚举写数 ÷ 32⌉ 个字），撕裂镜像与重放已在掩码里，不再按段拆
    given_state_ordinal = ordinal_lo;
    if (table_length(T_GIVEN_PERSISTED_SETS) != 0u) {
        let words_per_state = (hdr(1u) + 31u) / 32u;
        for (var word = 0u; word < words_per_state; word = word + 1u) {
            persisted[word] = tab(T_GIVEN_PERSISTED_SETS, ordinal_lo * words_per_state + word);
        }
        return;
    }
    // 序号 → 持久掩码（与 crash_facts::persisted_writes_of_state_by_the_segment_expansions 同一套数字次序）
    let segment_count = hdr(2u);
    var segment_of_state = segment_count;
    for (var s = 0u; s < segment_count; s = s + 1u) {
        let end = vec2<u32>(tab(T_SEGMENTS, s * SEGMENT_WORDS + 4u), tab(T_SEGMENTS, s * SEGMENT_WORDS + 5u));
        if (lt64(vec2<u32>(ordinal_lo, ordinal_hi), end)) {
            segment_of_state = s;
            break;
        }
    }
    // 之前的段整段持久：数位那一组与整组落的单元写那一组合起来就是段里全部的写
    for (var s = 0u; s < segment_of_state; s = s + 1u) {
        let digit_first = tab(T_SEGMENTS, s * SEGMENT_WORDS);
        let digit_count = tab(T_SEGMENTS, s * SEGMENT_WORDS + 1u);
        for (var k = 0u; k < digit_count; k = k + 1u) {
            set_persisted(tab(T_SEGMENT_WRITES, digit_first + k));
        }
        let group_first = tab(T_SEGMENTS, s * SEGMENT_WORDS + 7u);
        let group_count = tab(T_SEGMENTS, s * SEGMENT_WORDS + 8u);
        for (var k = 0u; k < group_count; k = k + 1u) {
            set_persisted(tab(T_SEGMENT_WRITES, group_first + k));
        }
    }
    var torn: array<u32, PERSISTED_MASK_WORDS>;
    for (var word = 0u; word < PERSISTED_MASK_WORDS; word = word + 1u) {
        torn[word] = 0u;
    }
    if (segment_of_state < segment_count) {
        let s = segment_of_state;
        // 段内序号满 64 位（判决 m3-crash-enumeration-merge-r2 第二节 L1-7：此前只用低 32 位，33–64 写的段拆错）
        var remaining = sub64(vec2<u32>(ordinal_lo, ordinal_hi), vec2<u32>(tab(T_SEGMENTS, s * SEGMENT_WORDS + 2u), tab(T_SEGMENTS, s * SEGMENT_WORDS + 3u)));
        let expansion = tab(T_SEGMENTS, s * SEGMENT_WORDS + 6u);
        // 不展开的段序号区间是空的，落不到这里；落到了就是表与序号对不上，判不了
        if (expansion == SEGMENT_EXPANSION_CODE_NOT_EXPANDED) {
            undecidable = UNDECIDABLE_STATE_IN_A_SEGMENT_NOT_EXPANDED;
        } else {
            let digit_first = tab(T_SEGMENTS, s * SEGMENT_WORDS);
            let digit_count = tab(T_SEGMENTS, s * SEGMENT_WORDS + 1u);
            let group_first = tab(T_SEGMENTS, s * SEGMENT_WORDS + 7u);
            let group_count = tab(T_SEGMENTS, s * SEGMENT_WORDS + 8u);
            let slice_digit_first = tab(T_SEGMENTS, s * SEGMENT_WORDS + 9u);
            let slice_digit_count = tab(T_SEGMENTS, s * SEGMENT_WORDS + 10u);
            // 分片的段（C790）：段内编号 = 片起点 + 片内序号。片起点拆成的各位（段写表里从 slice_digit_first 起，单元写那一组那一位在前）
            // 加上片内序号、逐位进位，每一步只用 u32（片长不超过 2^32，片内序号装得进低 32 位；crash::add_to_the_mixed_radix_digits 同一个算法）
            if (slice_digit_count > 0u) {
                if (remaining.y != 0u) {
                    undecidable = UNDECIDABLE_SLICE_PAST_ITS_DOMAIN;
                } else {
                    var carry = remaining.x;
                    var position = 0u;
                    if (group_count > 0u) {
                        let sum = tab(T_SEGMENT_WRITES, slice_digit_first) + carry % 2u;
                        carry = carry / 2u + sum / 2u;
                        if (sum % 2u == 1u) {
                            for (var k = 0u; k < group_count; k = k + 1u) {
                                set_persisted(tab(T_SEGMENT_WRITES, group_first + k));
                            }
                        }
                        position = 1u;
                    }
                    for (var k = 0u; k < digit_count; k = k + 1u) {
                        let write = tab(T_SEGMENT_WRITES, digit_first + k);
                        let choices = tab(T_RECORDED, write * 4u);
                        let sum = tab(T_SEGMENT_WRITES, slice_digit_first + position + k) + carry % choices;
                        carry = carry / choices + sum / choices;
                        let digit = sum % choices;
                        if (digit == choices - 1u) {
                            set_persisted(write);
                        } else if (digit == 1u) {
                            torn[write >> 5u] = torn[write >> 5u] | (1u << (write & 31u));
                        }
                    }
                    if (carry != 0u) {
                        undecidable = UNDECIDABLE_SLICE_PAST_ITS_DOMAIN;
                    }
                }
            } else {
                // 甲二有单元写：段内序号的最低一位是「单元写全落」，其余拆到原地写上（crash::CrashStatePlan 同一次序）
                if (group_count > 0u) {
                    if ((remaining.x & 1u) == 1u) {
                        for (var k = 0u; k < group_count; k = k + 1u) {
                            set_persisted(tab(T_SEGMENT_WRITES, group_first + k));
                        }
                    }
                    remaining = shr64(remaining, 1u);
                }
                for (var k = 0u; k < digit_count; k = k + 1u) {
                    let write = tab(T_SEGMENT_WRITES, digit_first + k);
                    let choices = tab(T_RECORDED, write * 4u);
                    let divided = divide_by_a_divisor_below_65536(remaining, choices);
                    let digit = divided.remainder;
                    remaining = divided.quotient;
                    if (digit == choices - 1u) {
                        set_persisted(write);
                    } else if (digit == 1u) {
                        torn[write >> 5u] = torn[write >> 5u] | (1u << (write & 31u));
                    }
                }
            }
        }
    }
    let recorded_count = hdr(0u);
    for (var w = 0u; w < recorded_count; w = w + 1u) {
        if ((torn[w >> 5u] & (1u << (w & 31u))) == 0u) {
            continue;
        }
        let torn_index = tab(T_RECORDED, w * 4u + 1u);
        if (torn_index == NONE) {
            continue;
        }
        set_persisted(torn_index);
        let replay_first = tab(T_RECORDED, w * 4u + 2u);
        let replay_count = tab(T_RECORDED, w * 4u + 3u);
        for (var r = 0u; r < replay_count; r = r + 1u) {
            let replay_index = tab(T_REPLAYS, (replay_first + r) * 2u);
            let replayed = tab(T_REPLAYS, (replay_first + r) * 2u + 1u);
            if (is_persisted(replayed)) {
                set_persisted(replay_index);
            }
        }
    }
}
