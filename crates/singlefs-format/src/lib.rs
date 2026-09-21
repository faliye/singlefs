//! 格式常量：第一个事务要写的每个宽度都在这里。
//!
//! 每个常量的值来自 `.claude/kb/layout/01-first-txn.md` 与各决策分项；带 `format-const:` 注的常量
//! 与 kb 里同名的 `<!-- format-const: NAME = N -->` 标记由门禁 27 号绑住，改值要三处一起动。
//! 值还是预想（等一次实测或一次定案）的常量，前一行写 `// placeholder: <分项或欠账编号（简称）> —— 为什么还是预想`，
//! 门禁「格式常量文件里的占位」数它们、并核每个占位都指到一条真实存在的分项或欠账。
//!
//! checker 与实现只共享这一个模块（D13（验证路线） 已定项 5），别的都各写一份。
#![forbid(unsafe_code)]

/// 落点粒度：一个 16 KiB 槽（D3（空间分配） 已定项 7；D19（块指针的结构与宽度预算） 已定项 4 的物理偏移就是槽号）。
pub const SLOT_BYTES: u64 = 16384;

/// 索引节点（码 2）恒 16 KiB（D8（核心索引结构） 已定项 2）。format-const: NODE_BYTES
pub const NODE_BYTES: u64 = 16384;

/// 数据单元（码 1）与打包记录单元（码 3）恒 32 KiB，含头（D4（校验和位置） 已定项 5）。format-const: DATA_UNIT_BYTES
pub const DATA_UNIT_BYTES: u64 = 32768;

/// 三类单元共同前缀：magic 4 + 格式版本 2 + 单元类型标签 1 + flags 1 + 声明长度 2 + 头校验和 32（D18（块里携带什么信息） 已定项 7）。
pub const UNIT_COMMON_PREFIX_BYTES: u64 = 42;

/// 32 字节校验和字段：CRC-32C 4 字节写在前 4 字节、其余 28 字节恒 0（D18（块里携带什么信息） 已定项 17）。
pub const WIDE_CHECKSUM_BYTES: u64 = 32;

/// 校验和字段里真正承载 CRC-32C 的那 4 字节。
pub const CHECKSUM_CRC32_CASTAGNOLI_BYTES: u64 = 4;

/// 单元明文头末尾之后的预留位：nonce 12 + MAC 16 + 算法类型 1（D18（块里携带什么信息） 已定项 14 / 已定项 16；算法类型 2026-09-14 用户定案）。
pub const NONCE_MAC_ALGORITHM_RESERVED_BYTES: u64 = 29;

/// 码 1 数据单元的类身份段末尾，也是头校验和的端点（D18（块里携带什么信息） 已定项 7 / 已定项 18）。format-const: DATA_UNIT_HEADER_BYTES
pub const DATA_UNIT_HEADER_BYTES: u64 = 105;

/// 码 1 数据单元含预留位的头：载荷从这里起（D18（块里携带什么信息） 已定项 16）。
pub const DATA_UNIT_PAYLOAD_OFFSET: u64 =
    DATA_UNIT_HEADER_BYTES + NONCE_MAC_ALGORITHM_RESERVED_BYTES;

/// 码 3 打包记录单元的类身份段末尾（D18（块里携带什么信息） 已定项 11）。format-const: PACKED_UNIT_HEADER_BYTES
pub const PACKED_UNIT_HEADER_BYTES: u64 = 107;

/// 码 3 含预留位的头：记录区从这里起（D18（块里携带什么信息） 已定项 16）。
pub const PACKED_UNIT_RECORDS_OFFSET: u64 =
    PACKED_UNIT_HEADER_BYTES + NONCE_MAC_ALGORITHM_RESERVED_BYTES;

/// 码 2 索引节点头里不含 key 区间的部分：共同前缀 42 + 树 ID 8 + 层级 1 + key 宽 1 + 诞生代号 8 + fsid 8 + 写序 4 + 出生序号 4 + 载荷 CRC 4 + 预留 2 + 条目数 2 + 条目宽 2（D8（核心索引结构） 已定项 11；D18（块里携带什么信息） 已定项 18；key 宽的位置 2026-09-14 用户定案）。
pub const INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE: u64 = 86;

/// 码 2 索引节点含 key 区间与预留位的头宽：`86 + 2 × key 宽 + 29`（D8（核心索引结构） 已定项 11；D18（块里携带什么信息） 已定项 16 / 已定项 18）。
pub const fn index_node_header_bytes(key_width_in_bytes: u64) -> u64 {
    INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE
        + 2 * key_width_in_bytes
        + NONCE_MAC_ALGORITHM_RESERVED_BYTES
}

/// 位置条目：设备身份 4 + 16 KiB 槽号 6 + 密文校验和 4（D19（块指针的结构与宽度预算） 已定项 4）。format-const: LOC_ENTRY
pub const LOC_ENTRY: u64 = 14; // naming-lint:external 名字由 kb 的 format-const 登记位定（D19 已定项 4），门禁 27 号按这个名字绑值

/// 每条指针带几份位置条目（D2（RAID 条带策略） 已定项 6：每次写至少 w = 2 列）。
pub const LOCATION_ENTRIES_PER_POINTER: u64 = 2;

/// 指针头部：MAC 16 + nonce 12 + 算法类型 1 + 压缩算法码 1 + 压后长度 2 + extent 偏移 2 + 出生树 8 + 出生 txg 8（D19（块指针的结构与宽度预算） 已定项 11；压缩两段 2026-09-14 用户定案）。
pub const POINTER_HEAD_BYTES: u64 = 50;

/// 指向码 1 的指针：头部 50 + 位置条目 14 × 2 + 写序 10（D19（块指针的结构与宽度预算） 已定项 7 / 已定项 8）。
pub const DATA_POINTER_BYTES: u64 =
    POINTER_HEAD_BYTES + LOC_ENTRY * LOCATION_ENTRIES_PER_POINTER + 10;

/// 指向码 2 / 码 3 的指针：头部 50 + 位置条目 14 × 2 + 实例代号 4 + 出生序号 4（D19（块指针的结构与宽度预算） 已定项 7 / 已定项 8）。
pub const NODE_POINTER_BYTES: u64 =
    POINTER_HEAD_BYTES + LOC_ENTRY * LOCATION_ENTRIES_PER_POINTER + 8;

/// 中央映射 key：类标签 1 + 出生树 8 + 出生 txg 8 + 写序 10，码 2 / 码 3 的 8 字节尾段补零到同宽（D19（块指针的结构与宽度预算） 已定项 6 / 已定项 10，2026-09-14 用户定案一宽）。
pub const MAPPING_KEY_BYTES: u64 = 27;

/// 中央映射条目：key 27 + 位置条目 14 × 2（D19（块指针的结构与宽度预算） 已定项 6）。
pub const MAPPING_ENTRY_BYTES: u64 = MAPPING_KEY_BYTES + LOC_ENTRY * LOCATION_ENTRIES_PER_POINTER;

/// extent 树叶记录 key：locality_id 8 + inode 8 + offset 8（D8（核心索引结构） 已定项 3）。
pub const EXTENT_KEY_BYTES: u64 = 24;

/// extent 树叶记录：key 24 + 数据指针 88（D19（块指针的结构与宽度预算） 已定项 7）。
pub const EXTENT_LEAF_RECORD_BYTES: u64 = EXTENT_KEY_BYTES + DATA_POINTER_BYTES;

/// inode 记录定长 140（D8（核心索引结构） 已定项 6）。format-const: INODE_RECORD_BYTES
pub const INODE_RECORD_BYTES: u64 = 140;

/// inode 树内部节点条目：分隔 key 8 + 身份引用 26 + 子指针 86（D8（核心索引结构） 已定项 6）。format-const: INODE_INTERNAL_ENTRY
pub const INODE_INTERNAL_ENTRY: u64 = 120;

/// 一个 32 KiB 码 3 容器装几条 inode 记录：(32768 − 136) ÷ 140。format-const: INODE_LEAF_RECORDS
pub const INODE_LEAF_RECORDS: u64 = 233;

/// 实例表记录定长 88（D18（块里携带什么信息） 已定项 11 打包记录类型 4）。format-const: INSTANCE_ROW_BYTES
pub const INSTANCE_ROW_BYTES: u64 = 88;

/// 实例表 kind 0 记录里的预留（88 − 22）（D18（块里携带什么信息） 已定项 11）。format-const: INSTANCE_ROW_RESERVED
pub const INSTANCE_ROW_RESERVED: u64 = 66;

/// 实例表一片装几条：(32768 − 136) ÷ 88。format-const: INSTANCE_TABLE_PAGE_RECORDS
pub const INSTANCE_TABLE_PAGE_RECORDS: u64 = 370;

/// 分配记录：key (设备 4, 槽号 6) + value (跨度 2, 代 8)（D3（空间分配） 已定项 7 / 已定项 11）。
pub const ALLOCATION_RECORD_KEY_BYTES: u64 = 10;
pub const ALLOCATION_RECORD_VALUE_BYTES: u64 = 10;
pub const ALLOCATION_RECORD_BYTES: u64 =
    ALLOCATION_RECORD_KEY_BYTES + ALLOCATION_RECORD_VALUE_BYTES;

/// 记账条目：key (标签 2, 树 ID 8, 设备 4, 代 8) + value 8 + seq 4（D5（快照 / 空间记账机制） 已定项 5；D8（核心索引结构） 已定项 7）。
pub const ACCOUNTING_KEY_BYTES: u64 = 22;
pub const ACCOUNTING_ENTRY_BYTES: u64 = ACCOUNTING_KEY_BYTES + 8 + 4;

/// 第一个事务这次发布写出的记账行数（D5（快照 / 空间记账机制） 已定项 8，2026-09-14 用户定案 15 行）。
pub const FIRST_TRANSACTION_ACCOUNTING_ROWS: u64 = 15;

/// 树表条目：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 + previous_snapshot_txg 8 + 诞生 txg 8 + 头 ID 8 + 预留 76（D8（核心索引结构） 已定项 8；重排与头 ID 2026-09-14 用户定案，加宽到 200 是 2026-09-16 用户定案）。format-const: TREE_TABLE_ENTRY_BYTES
pub const TREE_TABLE_ENTRY_BYTES: u64 = 200;

/// 第一个事务写出的树表条目数（extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist）。
pub const FIRST_TRANSACTION_TREE_TABLE_ENTRIES: u64 = 7;

/// 树 ID 从 11 起、与树的种类码错开（D8（核心索引结构） 已定项 11）。
pub const TREE_IDENTIFIER_EXTENT: u64 = 11;
pub const TREE_IDENTIFIER_INODE: u64 = 12;
pub const TREE_IDENTIFIER_ALLOCATION_RECORDS: u64 = 13;
pub const TREE_IDENTIFIER_ACCOUNTING: u64 = 14;
pub const TREE_IDENTIFIER_CENTRAL_MAPPING: u64 = 15;
pub const TREE_IDENTIFIER_LIVELIST: u64 = 16;
pub const TREE_IDENTIFIER_SPARSE_SIDE_TABLE: u64 = 17;
pub const TREE_IDENTIFIER_DEADLIST: u64 = 18;
/// 树 ID 水位：mkfs 写 11，第一次发布后写 19（D8（核心索引结构） 已定项 8 ②）。
pub const TREE_IDENTIFIER_WATERMARK_AT_MKFS: u64 = 11;
pub const TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH: u64 = 19;

/// 根记录：magic 4 + fsid 16 + flags 4 + 实例代号 4 + checkpoint_txg 8 + 树表指针 86 + 树 ID 水位 8 + 回退下界 F 8 + 自证校验和 32 + 实例表指针 86 + 中央映射树根指针 86 + 算法类型 1 + nonce 12 + MAC 16（D22（单元原子性怎么合成） 已定项 7；加密预留 2026-09-14 用户定案）。format-const: ROOT_RECORD_BYTES
pub const ROOT_RECORD_BYTES: u64 = 371;

/// journal 记录定长 4 KiB（D23（journal 的角色与格式） 已定项 12）。
pub const JOURNAL_RECORD_BYTES: u64 = 4096;

/// journal 记录头的十个基础字段（D23（journal 的角色与格式） 已定项 4）。format-const: JOURNAL_HEADER_TEN_FIELD_BYTES
pub const JOURNAL_HEADER_TEN_FIELD_BYTES: u64 = 78;

/// journal 记录头的新根段：树表指针 86 + 中央映射树根指针 86 + 树 ID 水位 8 + 回退下界 F 8（D23（journal 的角色与格式） 已定项 15）。
pub const JOURNAL_NEW_ROOT_SEGMENT_BYTES: u64 = NODE_POINTER_BYTES + NODE_POINTER_BYTES + 8 + 8;

/// journal 记录头：78 + 事务号 8 + 提交标记 1 + 反向链 4 + 载荷校验和 4 + 新根段 188 + fsid 8 + MAC 16（D23（journal 的角色与格式） 已定项 4 / 已定项 7 / 已定项 8 / 已定项 13 / 已定项 15；fsid 与 MAC 2026-09-14 用户定案）。format-const: JOURNAL_HEADER_BYTES
pub const JOURNAL_HEADER_BYTES: u64 = 307;

/// journal 点名项：位置条目 14 × 2 + 类标签 1 + 出生树 8 + 出生 txg 8 + key 尾段 10 + flags 1（D23（journal 的角色与格式） 已定项 17）。
pub const JOURNAL_NAMED_ENTRY_BYTES: u64 =
    LOC_ENTRY * LOCATION_ENTRIES_PER_POINTER + 1 + 8 + 8 + 10 + 1;

/// 一条 4 KiB 记录装几个点名项。
pub const JOURNAL_NAMED_ENTRIES_PER_RECORD: u64 =
    (JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES) / JOURNAL_NAMED_ENTRY_BYTES;

/// journal 环从槽 1024（16 MiB）起（D23（journal 的角色与格式） 已定项 2）。
pub const JOURNAL_RING_START_SLOT: u64 = 1024;

// placeholder: D23（journal 的角色与格式） 已定项 19 —— 环长是 mkfs 参数，默认 768 MiB，约束 环 ≤ 设备容量 / 4；第一版代码先按默认值
pub const JOURNAL_RING_DEFAULT_BYTES: u64 = 768 * 1024 * 1024;

/// 环几何的安全系数 F（I-8.1）。
pub const JOURNAL_SAFETY_FACTOR: u64 = 3;

/// 在飞记录数上限 = 环槽数 ÷ F，语义是重放前缀最多这么多条（D23（journal 的角色与格式） 已定项 18，2026-09-14 用户定案）。
pub const fn journal_in_flight_record_limit(ring_bytes: u64) -> u64 {
    ring_bytes / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR
}

/// 系统配置字段表合计（D22（单元原子性怎么合成） 已定项 9 / 已定项 15；2026-09-14 用户定案加四个字段）。format-const: SYSTEM_CONFIGURATION_BYTES
pub const SYSTEM_CONFIGURATION_BYTES: u64 = 481;

/// 系统配置每盘恒 2 个槽、槽 = 世代号 mod 2（D22（单元原子性怎么合成） 已定项 16）。
pub const SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE: u64 = 2;

/// 固定结构槽距的下限；槽距 = max(4096, mkfs 时探测的 io_min)（D2（RAID 条带策略） 已定项 19）。
pub const FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES: u64 = 4096;

/// 系统配置槽宽是格式常量 4096（D22（单元原子性怎么合成） 已定项 2，2026-09-14 三方第一轮 + 用户定案；此前是探测到的 physical_block_size）。format-const: SYSTEM_CONFIGURATION_SLOT_BYTES
pub const SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = 4096;

/// 根环参数：R 个区域、每区 S 槽、素数步长 P、chunk、基址（D22（单元原子性怎么合成） 已定项 2 / 已定项 16）。
pub const ROOT_RING_REGIONS: u64 = 3;
pub const ROOT_RING_SLOTS_PER_REGION: u64 = 8;
pub const ROOT_RING_PRIME_STEP: u64 = 3;
pub const ROOT_RING_CHUNK_BYTES: u64 = 1024 * 1024;
/// 根环起点是 16 KiB 槽号（1 MiB）。
pub const ROOT_RING_BASE_SLOT: u64 = 64;
/// 根环区域归属第一版写死 0 / 1 / 0（D2（RAID 条带策略） 已定项 7，2026-09-14 用户定案）。
pub const ROOT_RING_REGION_DEVICES: [u32; 3] = [0, 1, 0];

/// 单元区从槽 50176（784 MiB）起（D3（空间分配） 已定项 10）。
pub const UNIT_AREA_START_SLOT: u64 = 50176;

/// 聚簇段 64 槽（D3（空间分配） 已定项 10）。
pub const CLUSTER_SEGMENT_SLOTS: u64 = 64;

/// 第一个事务的 checkpoint_txg 是 3：两次暖机空发布之后（D16（发布语义） 已定项 6 / 已定项 8）。format-const: FIRST_TRANSACTION_TXG
pub const FIRST_TRANSACTION_TXG: u64 = 3;

/// 第一次可写挂载先推两次空发布（D16（发布语义） 已定项 8）。format-const: WARM_UP_EMPTY_PUBLISHES
pub const WARM_UP_EMPTY_PUBLISHES: u64 = 2;

// placeholder: C323（镜像大小全仓没有条款） —— 测试镜像 4 GiB 是里程碑步 0 的预想（由 环 ≤ 容量 / 4 逼出），用户说虚拟盘可扩到 200–500 GB
pub const TEST_IMAGE_DEFAULT_BYTES: u64 = 4 * 1024 * 1024 * 1024;

#[cfg(test)]
mod tests {
    use super::*;

    /// 字节表零到七的合计与这里的推导式必须一个数不差；写成加法、不写减法（减法在变异下会编译期溢出）。
    #[test]
    fn widths_match_the_first_transaction_byte_table() {
        assert_eq!(
            DATA_UNIT_PAYLOAD_OFFSET, 134,
            "码 1 载荷起点（D18 已定项 16 + 2026-09-14 算法类型）"
        );
        assert_eq!(PACKED_UNIT_RECORDS_OFFSET, 136, "码 3 记录区起点");
        assert_eq!(
            index_node_header_bytes(8),
            131,
            "inode 树根 / 树表单元的码 2 头"
        );
        assert_eq!(index_node_header_bytes(24), 163, "extent 树的码 2 头");
        assert_eq!(index_node_header_bytes(10), 135, "分配记录树的码 2 头");
        assert_eq!(index_node_header_bytes(22), 159, "记账树的码 2 头");
        assert_eq!(index_node_header_bytes(27), 169, "中央映射树的码 2 头");
        assert_eq!(DATA_POINTER_BYTES, 88, "指向码 1 的指针");
        assert_eq!(NODE_POINTER_BYTES, 86, "指向码 2 / 码 3 的指针");
        assert_eq!(MAPPING_ENTRY_BYTES, 55, "映射条目一宽");
        assert_eq!(EXTENT_LEAF_RECORD_BYTES, 112, "extent 叶记录");
        assert_eq!(
            INODE_INTERNAL_ENTRY,
            8 + 26 + NODE_POINTER_BYTES,
            "inode 内部条目 = 分隔 key 8 + 身份引用 26 + 子指针"
        );
        assert_eq!(
            INODE_LEAF_RECORDS,
            (DATA_UNIT_BYTES - PACKED_UNIT_RECORDS_OFFSET) / INODE_RECORD_BYTES,
            "一个容器装几条 inode 记录"
        );
        assert_eq!(
            INSTANCE_TABLE_PAGE_RECORDS,
            (DATA_UNIT_BYTES - PACKED_UNIT_RECORDS_OFFSET) / INSTANCE_ROW_BYTES,
            "实例表一片装几条"
        );
        assert_eq!(
            TREE_TABLE_ENTRY_BYTES,
            8 + 2 + 2 + 2 + NODE_POINTER_BYTES + 8 + 8 + 8 + 76,
            "树表条目九段"
        );
        assert_eq!(
            ROOT_RECORD_BYTES,
            4 + 16
                + 4
                + 4
                + 8
                + NODE_POINTER_BYTES
                + 8
                + 8
                + WIDE_CHECKSUM_BYTES
                + NODE_POINTER_BYTES
                + NODE_POINTER_BYTES
                + 1
                + 12
                + 16,
            "根记录十四段"
        );
        assert_eq!(
            JOURNAL_HEADER_BYTES,
            JOURNAL_HEADER_TEN_FIELD_BYTES
                + 8
                + 1
                + 4
                + 4
                + JOURNAL_NEW_ROOT_SEGMENT_BYTES
                + 8
                + 16,
            "journal 记录头七段"
        );
        assert_eq!(INODE_LEAF_RECORDS, 233, "一个容器装几条 inode 记录");
        assert_eq!(INSTANCE_TABLE_PAGE_RECORDS, 370, "实例表一片装几条");
        assert_eq!(ALLOCATION_RECORD_BYTES, 20, "分配记录");
        assert_eq!(ACCOUNTING_ENTRY_BYTES, 34, "记账条目");
        assert_eq!(TREE_TABLE_ENTRY_BYTES, 200, "树表条目");
        assert_eq!(ROOT_RECORD_BYTES, 371, "根记录");
        assert_eq!(JOURNAL_NEW_ROOT_SEGMENT_BYTES, 188, "journal 新根段");
        assert_eq!(JOURNAL_HEADER_BYTES, 307, "journal 记录头");
        assert_eq!(JOURNAL_NAMED_ENTRY_BYTES, 56, "journal 点名项");
        assert_eq!(JOURNAL_NAMED_ENTRIES_PER_RECORD, 67, "一条记录装几个点名项");
        assert_eq!(
            journal_in_flight_record_limit(JOURNAL_RING_DEFAULT_BYTES),
            65536,
            "在飞记录数上限 = 196608 ÷ 3"
        );
        assert_eq!(
            UNIT_AREA_START_SLOT,
            JOURNAL_RING_START_SLOT + JOURNAL_RING_DEFAULT_BYTES / SLOT_BYTES,
            "单元区紧接 journal 环之后"
        );
    }

    /// 系统配置的 481 与它所在的槽：槽宽预想 4096 时余 3615，装得下下一条 86 字节指针；按此前的 512 槽算只余 31、塞不进（D22 已定项 2 重开的理由之一）。
    #[test]
    fn system_configuration_fits_in_the_slot_with_room_for_one_more_pointer() {
        let slot_bytes = std::hint::black_box(SYSTEM_CONFIGURATION_SLOT_BYTES);
        assert!(SYSTEM_CONFIGURATION_BYTES <= slot_bytes);
        assert_eq!(
            slot_bytes,
            SYSTEM_CONFIGURATION_BYTES + 3615,
            "4096 槽余 3615"
        );
        assert!(
            SYSTEM_CONFIGURATION_BYTES + NODE_POINTER_BYTES <= slot_bytes,
            "下一条 86 字节指针装得下"
        );
        assert!(
            slot_bytes <= FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,
            "槽宽不超过槽距下限，两槽永不重叠"
        );
    }

    /// 树 ID 与树的种类码错开：种类码 0..8，树 ID 从 11 起，水位在最后一个之后。
    #[test]
    fn tree_identifiers_start_after_kind_codes_and_watermark_follows_the_last() {
        let identifiers = [
            TREE_IDENTIFIER_EXTENT,
            TREE_IDENTIFIER_INODE,
            TREE_IDENTIFIER_ALLOCATION_RECORDS,
            TREE_IDENTIFIER_ACCOUNTING,
            TREE_IDENTIFIER_CENTRAL_MAPPING,
            TREE_IDENTIFIER_LIVELIST,
            TREE_IDENTIFIER_SPARSE_SIDE_TABLE,
            TREE_IDENTIFIER_DEADLIST,
        ];
        for (position, identifier) in identifiers.iter().enumerate() {
            let expected = TREE_IDENTIFIER_WATERMARK_AT_MKFS
                + u64::try_from(position).expect("八棵树的序号装得进 u64");
            assert_eq!(*identifier, expected, "树 ID 连号");
        }
        assert_eq!(
            TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
            TREE_IDENTIFIER_DEADLIST + 1
        );
        assert_eq!(
            u64::try_from(ROOT_RING_REGION_DEVICES.len()).expect("三个区域"),
            ROOT_RING_REGIONS
        );
    }
}
