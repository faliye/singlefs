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
// 系统配置 incompat 位图第一个字节这一版布局的那一个（D15（格式冻结政策） 已定项 4）：位 1 布局身份与位 2 inode 记录带截断字段都带
// （位 2 新开，判决 m4-trunc-r2 第六节；是布局身份的一部分），第一个字节是别的值一律当不认识
const INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT: u32 = 2u;
const INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT: u32 = 4u;
const INCOMPAT_FIRST_BYTE_OF_THIS_LAYOUT: u32 =
    INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT | INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT;
// 系统配置槽 incompat 位图那 32 字节（偏移 6 起）里这一版读者认得：第一个字节恰好是这一版布局的那一个，其余 31 字节全零
fn incompat_bits_are_recognized(base: u32) -> bool {
    return rd8(base, 6u) == INCOMPAT_FIRST_BYTE_OF_THIS_LAYOUT && all_zero(base, 7u, 31u);
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
// 不抄进表，映射有多少条条目都不判不了（「按块分批读，不设上限」，与 R_DATA 那一段的数据指针同一规则）。
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
// 都不判不了（「按块分批读，不设上限」）。extent 树那一遍照旧走全树、核每一条指针、数单元，只把头一块记进来；
// walk_to_file 读数据单元读到这一块末尾时，从下段根往下找含下一个单元的那片叶，把那片叶里从它起的指针记成下一块（find_the_next_block_of_data_pointers）。
// 走读内核借这一段装由记录施加出来的版本清单（每版 1 字，至多 R_DATA_WORDS 版），扫描内核从这里读它
const R_DATA: u32 = 30304u;
const R_DATA_WORDS: u32 = 512u;
const DATA_POINTERS_PER_BLOCK: u32 = R_DATA_WORDS / 2u;
// 恢复内核读 extent 上段叶条目的那一段（字节基、条目偏移，每条 2 字）：按块分批读，一批至多 MAX_UPPER_ENTRIES 条，有数据的文件有多少个
// 都不判不了（「按块分批读，不设上限」）；read_extent_tree_of_the_first_file 读完一批的下段再从上段根走一遍记下一批。
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

// 恢复内核判退时建上一版那一样读者（crash_judge_recovery.wgsl 的 rebuild_reader_kind，recovery::rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at）
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
// R_MAPPING_LEAVES 里记的叶有一片条目窄于 55（read_code_two_tree 读映射树时开头清掉、逐片记；查映射先看它，判红不必逐片扫叶表）。
// 只有 mapping_lookup 读它，它只在一次读通的映射树之后调，那一次读树开头已清过，begin_state 不另清
var<private> some_mapping_leaf_has_entries_narrower_than_a_mapping_entry: bool;
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
// 建上一版那一样读者读的进映射的单元里有一个的 key 在映射里查不到（经 read_via_hint_then_mapping 记下；查映射本身判红的不记）：
// recovery::central_mapping_keys_are_those_of_the_mapped_units_and_the_entries_kept_for_the_snapshots 那一判的 GPU 一侧
var<private> some_mapped_unit_key_is_missing_from_the_central_mapping: bool;

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
// 空段的最后一槽按 32 KiB 读（拼读，判决 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第二节 S2 那一行与第三节第 1 条，攻方 Z2；
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
    // （拒收码 2，与下面读者收不收的池级值里格式版本那一判同一个码），是 1 才算 incompat 位图不认识（C663「格式版本先」）
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
    // 的 RootRingRegionCountNotTheFirstVersionValue，C663「R 读并整池拒」）
    if (rd8(base, 361u) != 3u) {
        return empty_system_configuration(3u, 14u);
    }
    // 根环区域步长 P（363）、chunk（367）不是第一版常量，或 S × 槽距大于 P × chunk：整池拒，排在 R 之后、环长之前
    // （recovery::root_ring_regions_fit_their_stride 的 RootRingPrimeStepNotTheFirstVersionValue、RootRingChunkBytesNotTheFirstVersionValue、
    // RootRingRegionLongerThanTheRegionStride；D22（单元原子性怎么合成） 已定项 16 第 1 句）。
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
        // 施加末条：所选根的七个字段换成记录新根段里的（recovery::replay_journal，已定项 15「回到甲」）；fsid 照所选根
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
