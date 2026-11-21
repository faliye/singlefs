//! 日志记录、树表项、inode 记录、extent 记录、分配记录、记账项、映射项与实例表链记录的拼装与解析。

use e7_index_bench::{u16_from_u64, u32_from_u64, usize_from_u64};
use super::*;

/// journal 记录 4096：头 487（字节表六的表序 + D23 已定项 15 的新根段七项 + fsid 与 MAC + 记录标志与本次发布内序号）+ 点名项数组。
#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) struct JournalRecord {
    pub(super) instance: InstanceGeneration,
    pub(super) counter: JournalCounter,
    pub(super) checkpoint_txg: CheckpointTxg,
    pub(super) transaction: TransactionNumber,
    pub(super) is_commit: bool,
    /// D23（journal 的角色与格式） 已定项 4：本次发布内序号，从 1 起；本实验每次发布只有一条记录，恒 1。
    pub(super) ordinal_within_publish: u32,
    /// D23（journal 的角色与格式） 已定项 4 / 17：记录标志（位 0 = 本次发布末条），其余位恒 0。
    pub(super) record_flags: u8,
    pub(super) back_chain: u32,
    /// D23（journal 的角色与格式） 已定项 4：系统配置 fsid 的低 8 字节，与单元头同口径（I-1.4（块头 fsid 一致））。
    pub(super) fsid: u64,
    /// D23（journal 的角色与格式） 已定项 15 的新根段：崩在记录持久之后、根槽持久之前时由它重建那次发布的根。
    pub(super) new_tree_table: NodePointer,
    pub(super) new_mapping_root: NodePointer,
    pub(super) new_tree_identifier_watermark: u64,
    pub(super) new_rollback_floor: CheckpointTxg,
    /// 新根段后三项（D23 已定项 15）：实例表单元指针、分配记录树根指针、卸载记号——
    /// 取这次发布写的那条根的同名字段；恢复施加记录时三项取记录里的、不照抄所选根。
    pub(super) new_instance_table: NodePointer,
    pub(super) new_allocation_record_tree_root: NodePointer,
    pub(super) new_unmount_marker: bool,
    pub(super) named: Vec<NamedUnit>,
}

pub(super) const JOURNAL_HEADER_CHECKSUM_OFFSET: usize = 4 + 2 + 1 + 1 + 4 + 4 + 10 + 8 + 12;
/// D23（journal 的角色与格式） 已定项 4：记录标志（位 0 = 本次发布末条）的偏移 = magic 4 + 类型 2 + 算法类型 1。
pub(super) const JOURNAL_RECORD_FLAGS_OFFSET: usize = 4 + 2 + 1;

/// D23（journal 的角色与格式） 已定项 19 ②：`previous_hash` =
/// CRC32C(**本实例内逻辑前一条**记录的 487 字节头，其中 `header_csum` 那 32 字节按零参与)；本实例第一条恒 0、不读盘。
pub(super) fn journal_back_chain(previous_record_bytes: &[u8]) -> u32 {
    let mut header = previous_record_bytes[..usize_from_u64(JOURNAL_HEADER_BYTES)].to_vec();
    header[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + usize_from_u64(WIDE_CHECKSUM_BYTES)].fill(0);
    castagnoli_crc32(&header)
}

impl JournalRecord {
    pub(super) fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(usize_from_u64(JOURNAL_RECORD_BYTES));
        writer.put(&JOURNAL_MAGIC);
        writer.put_u16(1); // 类型：普通记录
        writer.put_u8(0); // 算法类型
        writer.assert_position(7, "记录标志偏移（已定项 4）");
        writer.put_u8(self.record_flags); // 记录标志：位 0 = 本次发布末条（已定项 4、17）
        writer.put_u32(u32_from_u64(JOURNAL_RECORD_BYTES));
        writer.put_u32(u32::try_from(self.named.len()).expect("点名项数 4 字节"));
        writer.put_u32(self.instance.0);
        writer.put_six_byte_unsigned(self.counter.0);
        writer.put_u64(self.checkpoint_txg.0);
        writer.skip(12); // nonce
        writer.assert_position(JOURNAL_HEADER_CHECKSUM_OFFSET as u64, "记录头校验和");
        writer.skip(usize_from_u64(WIDE_CHECKSUM_BYTES));
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
        // D23 已定项 15 新根段的后三项（次序：实例表单元指针、分配记录树根指针、卸载记号）；卸载记号 4 字节，
        // 编码照根记录 flags（位 0 = 正常卸载那一串写的，其余位恒 0，D22 已定项 7；kb 空白，推的）。
        self.new_instance_table.write_to(&mut writer);
        self.new_allocation_record_tree_root.write_to(&mut writer);
        writer.put_u32(if self.new_unmount_marker { 0x0000_0001 } else { 0 });
        assert_eq!(writer.position() - new_root_segment_start, usize_from_u64(JOURNAL_NEW_ROOT_SEGMENT_BYTES), "新根段");
        // 新根段之后是 fsid 8，再之后是 MAC 16（第一版全 0 留位，D9（加密） 已定项 12 第 1 项）。
        writer.put_u64(self.fsid);
        writer.skip(16);
        writer.assert_position(JOURNAL_HEADER_BYTES, "记录头");
        for named in &self.named {
            named.write_to(&mut writer);
        }
        let payload_end = writer.position();
        let mut bytes = writer.bytes;
        // 载荷校验和仍罩点名项数组（D23 已定项 13 的字段保留），它自己住头里、落在头校验和覆盖内。
        let payload_checksum = castagnoli_crc32(&bytes[usize_from_u64(JOURNAL_HEADER_BYTES)..payload_end]);
        bytes[payload_checksum_offset..payload_checksum_offset + 4].copy_from_slice(&payload_checksum.to_le_bytes());
        // D23 已定项 13：`header_csum` 覆盖**整条记录 [0, 4096) 含补齐**、自身按零参与。
        let digest = wide_checksum_with_field_zeroed(&bytes, usize_from_u64(JOURNAL_RECORD_BYTES), JOURNAL_HEADER_CHECKSUM_OFFSET);
        bytes[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        bytes
    }

    pub(super) fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes[..4] != JOURNAL_MAGIC {
            return None;
        }
        if wide_checksum_with_field_zeroed(bytes, usize_from_u64(JOURNAL_RECORD_BYTES), JOURNAL_HEADER_CHECKSUM_OFFSET)
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
        reader.skip(12 + usize_from_u64(WIDE_CHECKSUM_BYTES));
        let transaction = TransactionNumber(reader.get_u64());
        let is_commit = reader.get_u8() == 1;
        let ordinal_within_publish = reader.get_u32(); // 偏移 87：本次发布内序号（已定项 4）
        let back_chain = reader.get_u32();
        let payload_checksum = reader.get_u32();
        let new_tree_table = NodePointer::read_from(&mut reader);
        let new_mapping_root = NodePointer::read_from(&mut reader);
        let new_tree_identifier_watermark = reader.get_u64();
        let new_rollback_floor = CheckpointTxg(reader.get_u64());
        let new_instance_table = NodePointer::read_from(&mut reader);
        let new_allocation_record_tree_root = NodePointer::read_from(&mut reader);
        let unmount_marker_bits = reader.get_u32();
        if unmount_marker_bits & !0x0000_0001 != 0 {
            return None; // 卸载记号位 0 之外有位为 1 ⇒ 当损坏（D23 已定项 15；编码照根记录 flags，kb 空白，推的）
        }
        let new_unmount_marker = unmount_marker_bits & 0x0000_0001 != 0;
        let fsid = reader.get_u64();
        reader.skip(16); // MAC：第一版留位
        let payload_end = usize_from_u64(JOURNAL_HEADER_BYTES) + named_count * usize_from_u64(JOURNAL_NAMED_ENTRY_BYTES);
        if record_length as usize != bytes.len() || payload_end > bytes.len() {
            return None;
        }
        if castagnoli_crc32(&bytes[usize_from_u64(JOURNAL_HEADER_BYTES)..payload_end]) != payload_checksum {
            return None;
        }
        let named = (0..named_count).map(|_| NamedUnit::read_from(&mut reader)).collect();
        Some(Self {
            instance,
            counter,
            checkpoint_txg,
            transaction,
            is_commit,
            ordinal_within_publish,
            record_flags,
            back_chain,
            fsid,
            new_tree_table,
            new_mapping_root,
            new_tree_identifier_watermark,
            new_rollback_floor,
            new_instance_table,
            new_allocation_record_tree_root,
            new_unmount_marker,
            named,
        })
    }
}

// ───────────────────────── 树表条目、inode、extent、分配、记账、映射 ─────────────────────────

/// D8（核心索引结构） 已定项 8：树 ID 8 打头（= 码 2 条目的 key）+ 条目长度 2
/// + 树的种类 2 + flags 2 + 根指针 86 + previous_snapshot_txg 8 + 诞生 txg 8 + 头 ID 8 + 预留 76 = 200。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct TreeTableEntry {
    pub(super) kind: u16,
    pub(super) tree: TreeIdentifier,
    pub(super) root: NodePointer,
    pub(super) birth_txg: CheckpointTxg,
    /// D5（快照 / 空间记账机制） 已定项 9：这棵树归哪个可写头；无归属写 0。
    pub(super) head_identifier: u64,
}

impl TreeTableEntry {
    #[allow(clippy::wrong_self_convention, reason = "`to_bytes` 在 `.iter().map(类型::to_bytes)` 里按引用传入，改成按值要处处加 `copied`")]
    pub(super) fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(usize_from_u64(TREE_TABLE_ENTRY_BYTES));
        writer.put_u64(self.tree.0);
        writer.put_u16(u16_from_u64(TREE_TABLE_ENTRY_BYTES));
        writer.put_u16(self.kind);
        writer.put_u16(0);
        self.root.write_to(&mut writer);
        writer.put_u64(0); // previous_snapshot_txg：0 = 不适用（新池新建文件没有前驱快照）
        writer.put_u64(self.birth_txg.0);
        writer.put_u64(self.head_identifier);
        writer.skip(76);
        writer.assert_position(TREE_TABLE_ENTRY_BYTES, "树表条目");
        writer.bytes
    }
    pub(super) fn parse(bytes: &[u8]) -> Result<Self, UnitError> {
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
        if bytes[usize_from_u64(TREE_TABLE_ENTRY_BYTES) - 76..].iter().any(|&byte| byte != 0) {
            return Err(UnitError::Structure("树表条目预留 76 字节非零"));
        }
        Ok(Self { kind, tree, root, birth_txg, head_identifier })
    }
}

/// 每棵树的 key 宽（码 2 头里的 key 区间与自述 key 宽都随它走）。
pub(super) fn key_width_for_kind(kind: u16) -> Option<usize> {
    match kind {
        TREE_KIND_EXTENT => Some(24),
        TREE_KIND_INODE => Some(8),
        TREE_KIND_ALLOCATION => Some(ALLOCATION_KEY_BYTES),
        TREE_KIND_ACCOUNTING => Some(22),
        TREE_KIND_MAPPING => Some(usize_from_u64(MAPPING_KEY_BYTES)),
        // livelist、稀疏旁表、deadlist 三棵 day-1 注册但新池新建文件里根指针为零 ⇒ 没有节点要解，key 宽这一格还没有条款。
        _ => None,
    }
}
/// 树表单元自己按 key = 树 ID（8 字节）走（预想）。
pub(super) const TREE_TABLE_KEY_WIDTH: usize = 8;

/// inode 树内部节点条目 120（D8 已定项 6）：分隔 key 8 + 身份引用 26（出生树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8）+ 子指针 86。
pub(super) fn build_inode_internal_entry(separator_key: u64, child_identity: PackedIdentity, child: NodePointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(usize_from_u64(INODE_INTERNAL_ENTRY));
    writer.put_u64(separator_key);
    writer.put_u64(child_identity.birth_tree.0);
    writer.put_u16(child_identity.record_type);
    writer.put_u64(child_identity.container);
    writer.put_u64(child_identity.container_birth.0);
    child.write_to(&mut writer);
    writer.assert_position(INODE_INTERNAL_ENTRY, "inode 内部条目");
    writer.bytes
}

pub(super) fn parse_inode_internal_entry(bytes: &[u8]) -> (u64, PackedIdentity, NodePointer) {
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
pub(super) struct InodeRecord {
    pub(super) inode: u64,
    pub(super) object_birth: CheckpointTxg,
    pub(super) size: u64,
    pub(super) change_count: u64,
}

impl InodeRecord {
    pub(super) fn to_bytes(self) -> Vec<u8> {
        let mut writer = ByteWriter::new(usize_from_u64(INODE_RECORD_BYTES));
        writer.put_u64(self.inode);
        writer.put_u64(self.object_birth.0);
        writer.put_u64(0); // locality_id：第一版无父目录取 0
        writer.put_u32(0o100644); // mode
        writer.put_u32(0); // uid
        writer.put_u32(0); // gid
        writer.put_u32(1); // nlink
        writer.assert_position(40, "size");
        writer.put_u64(self.size);
        // C480（inode 记录的 blocks 怎么算全仓没有条款）：blocks = ⌈文件字节数 ÷ 512⌉
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
    pub(super) fn parse(bytes: &[u8]) -> Result<Self, UnitError> {
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
/// E142 第十五次跑起，新池新建文件改走上段叶（`build_extent_upper_leaf_entry`）：只有一个数据单元的文件不建下段
/// （D8 已定项 14「只有一个数据单元的文件不建下段」），这两个函数留给下段路径（新池新建文件走不到，标「附带」）。
#[allow(dead_code, reason = "下段叶格式，新池新建文件这个装置走不到；留着给以后写下段路径用，不删")]
fn build_extent_record(inode: u64, offset: u64, pointer: DataPointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(usize_from_u64(EXTENT_LEAF_RECORD_BYTES));
    writer.put_u64(0);
    writer.put_u64(inode);
    writer.put_u64(offset);
    pointer.write_to(&mut writer);
    writer.assert_position(EXTENT_LEAF_RECORD_BYTES, "extent 叶记录");
    writer.bytes
}

#[allow(dead_code, reason = "下段叶格式，新池新建文件这个装置走不到；留着给以后写下段路径用，不删")]
fn parse_extent_record(bytes: &[u8]) -> ([u8; 24], DataPointer) {
    let mut reader = ByteReader::new(bytes);
    let key: [u8; 24] = reader.take(24).try_into().expect("切了 24 字节");
    (key, DataPointer::read_from(&mut reader))
}

/// 分配记录 20（D3 已定项 7 + 已定项 11）：key = 设备 4 + 槽号 6；value = 跨度 2（最高位 = 已释放）+ 分配代 8。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct AllocationRecord {
    pub(super) device: DeviceIdentity,
    pub(super) slot: SlotNumber,
    pub(super) span_slots: u16,
    pub(super) generation: CheckpointTxg,
    /// 已释放（跨度段最高位）：新池新建文件里只有 mkfs 那片第 0 版树表单元——A 重写树表把它换下、进 defer 队列（D3 已定项 7），释放代 = A 的 txg。
    pub(super) is_released: bool,
}

/// 分配记录跨度段的最高位 = 已释放（与 crates 的 `ALLOCATION_RECORD_RELEASED_FLAG` 同一位）。
const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;

impl AllocationRecord {
    #[allow(clippy::wrong_self_convention, reason = "`to_bytes` 在 `.iter().map(类型::to_bytes)` 里按引用传入，改成按值要处处加 `copied`")]
    pub(super) fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(usize_from_u64(ALLOCATION_RECORD_BYTES));
        writer.put_u32(self.device.0);
        writer.put_six_byte_unsigned(self.slot.0);
        writer.put_u16(if self.is_released { self.span_slots | ALLOCATION_RECORD_RELEASED_FLAG } else { self.span_slots });
        writer.put_u64(self.generation.0);
        writer.assert_position(ALLOCATION_RECORD_BYTES, "分配记录");
        writer.bytes
    }
    pub(super) fn parse(bytes: &[u8]) -> Self {
        let mut reader = ByteReader::new(bytes);
        let device = DeviceIdentity(reader.get_u32());
        let slot = SlotNumber(reader.get_six_byte_unsigned());
        let span_field = reader.get_u16();
        let generation = CheckpointTxg(reader.get_u64());
        Self { device, slot, span_slots: span_field & !ALLOCATION_RECORD_RELEASED_FLAG, generation, is_released: span_field & ALLOCATION_RECORD_RELEASED_FLAG != 0 }
    }
    pub(super) fn key_bytes(&self) -> Vec<u8> {
        self.to_bytes()[..ALLOCATION_KEY_BYTES].to_vec()
    }
    /// D8（核心索引结构） 已定项 11：key 的全序按逐字段无符号整数、字段自左向右比较——小端存储不构成 memcmp 序。
    pub(super) fn sort_key(&self) -> (u32, u64) {
        (self.device.0, self.slot.0)
    }
}

/// 记账条目 34（D5 已定项 5 + D8 已定项 7）：key (标签 2, 树 ID 8, 设备 4, 代 8) + value 8 + seq 4。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct AccountingEntry {
    pub(super) statistic: u16,
    pub(super) tree: TreeIdentifier,
    pub(super) device: DeviceIdentity,
    pub(super) generation: CheckpointTxg,
    pub(super) value: u64,
    pub(super) sequence: u32,
}

impl AccountingEntry {
    #[allow(clippy::wrong_self_convention, reason = "`to_bytes` 在 `.iter().map(类型::to_bytes)` 里按引用传入，改成按值要处处加 `copied`")]
    pub(super) fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(usize_from_u64(ACCOUNTING_ENTRY_BYTES));
        writer.put_u16(self.statistic);
        writer.put_u64(self.tree.0);
        writer.put_u32(self.device.0);
        writer.put_u64(self.generation.0);
        writer.put_u64(self.value);
        writer.put_u32(self.sequence);
        writer.assert_position(ACCOUNTING_ENTRY_BYTES, "记账条目");
        writer.bytes
    }
    pub(super) fn parse(bytes: &[u8]) -> Self {
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
    pub(super) fn key_bytes(&self) -> Vec<u8> {
        self.to_bytes()[..22].to_vec()
    }
    /// key 的全序按逐字段无符号整数比较（D8（核心索引结构） 已定项 11）：标签 → 树 ID → 设备 → 代。
    pub(super) fn sort_key(&self) -> (u16, u64, u32, u64) {
        (self.statistic, self.tree.0, self.device.0, self.generation.0)
    }
}

/// 中央映射 key（D19 已定项 6 / 已定项 10）：**一律 27**——
/// 码 1 = 类标签 1 + 出生树 8 + 出生 txg 8 + 写序 10 天然就是 27；
/// 码 2 / 码 3 = 类标签 1 + 出生树 8 + 出生 txg 8 + 实例代号 4 + 出生序号 4 = 25，**末尾补零到 27**。
pub(super) fn mapping_key_for_data(head: PointerHead, write_order: WriteOrder) -> Vec<u8> {
    let mut writer = ByteWriter::new(usize_from_u64(MAPPING_KEY_BYTES));
    writer.put_u8(UNIT_CLASS_DATA);
    writer.put_u64(head.birth_tree.0);
    writer.put_u64(head.birth_txg.0);
    writer.put_u32(write_order.instance.0);
    writer.put_six_byte_unsigned(write_order.transaction.0);
    writer.assert_position(MAPPING_KEY_BYTES, "码 1 映射 key");
    writer.bytes
}

pub(super) fn mapping_key_for_node(unit_class: u8, pointer: NodePointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(usize_from_u64(MAPPING_KEY_BYTES));
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
pub(super) fn mapping_key_sort_key(key: &[u8]) -> (u8, u64, u64, u32, u64) {
    let mut reader = ByteReader::new(key);
    let unit_class = reader.get_u8();
    let birth_tree = reader.get_u64();
    let birth_txg = reader.get_u64();
    let instance = reader.get_u32();
    let tail = if unit_class == UNIT_CLASS_DATA { reader.get_six_byte_unsigned() } else { u64::from(reader.get_u32()) };
    (unit_class, birth_tree, birth_txg, instance, tail)
}

pub(super) fn build_mapping_entry(key: &[u8], locations: [LocationEntry; 2]) -> Vec<u8> {
    assert_eq!(key.len() as u64, MAPPING_KEY_BYTES, "映射 key 一律 27（D19 已定项 10，2026-09-14 用户定案）");
    let mut writer = ByteWriter::new(usize_from_u64(MAPPING_ENTRY_BYTES));
    writer.put(key);
    for location in &locations {
        location.write_to(&mut writer);
    }
    writer.assert_position(MAPPING_ENTRY_BYTES, "映射条目");
    writer.bytes
}

pub(super) fn parse_mapping_entry(bytes: &[u8]) -> (Vec<u8>, [LocationEntry; 2]) {
    let mut reader = ByteReader::new(bytes);
    let key = reader.take(usize_from_u64(MAPPING_KEY_BYTES)).to_vec();
    (key, [LocationEntry::read_from(&mut reader), LocationEntry::read_from(&mut reader)])
}

/// 实例表 kind = 1 链指针记录（D18 已定项 11）：kind 1 | 有无下一片 1 | 位置指针 86（无下一片时清零）= 88。
/// 位置指针 86 占掉了预留，记录宽 88——**这条记录没有余量了**，
/// 指针再宽一个字节 `INSTANCE_ROW_BYTES` 就要跟着涨（打包记录定长，kind 0 与 kind 1 必须同宽）。
pub(super) fn build_instance_table_chain_record() -> Vec<u8> {
    let mut writer = ByteWriter::new(usize_from_u64(INSTANCE_ROW_BYTES));
    writer.put_u8(1);
    writer.put_u8(0);
    writer.skip(usize_from_u64(NODE_POINTER_BYTES));
    writer.assert_position(INSTANCE_ROW_BYTES, "实例表链指针记录");
    writer.bytes
}

#[cfg(test)]
#[path = "journal_and_record_formats_tests.rs"]
mod tests;
