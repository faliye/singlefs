//! 索引节点（单元类 2）：头宽与条目容量、写法与读法的规格、`build_index_node`、`parse_index_node` 与它们的往返引理。

use super::*;

verus! {

broadcast use group_little_endian_round_trips;

// ===== 码 2 节点的头宽与容量 =====

/// 码 2 节点明文头末尾 `86 + 2 × key 宽`（[`index_node_plain_header_end`] 的式子）。
pub open spec fn specification_index_node_plain_header_end(key_width: int) -> int {
    INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE + INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT * key_width
}

/// 码 2 节点含预留位的头宽 `86 + 2 × key 宽 + 29`（[`index_node_header_bytes`] 的式子）。
pub open spec fn specification_index_node_header_bytes(key_width: int) -> int {
    specification_index_node_plain_header_end(key_width) + NONCE_MAC_ALGORITHM_RESERVED_BYTES
}

/// 一个码 2 节点装得下的定宽条目数 `(16384 − 含预留位的头宽) ÷ 条目宽`（[`index_node_entry_capacity`] 的式子）。
pub open spec fn specification_index_node_entry_capacity(key_width: int, entry_width: int) -> int {
    (NODE_BYTES - specification_index_node_header_bytes(key_width)) / entry_width
}

/// 码 2 索引节点 16384（D18（块里携带什么信息） 已定项 18 的偏移表，key 宽住 51）：
/// 42 树 ID / 50 层级 / 51 key 宽 / 52 key 区间 2k / 52+2k 诞生代号 / 60+2k fsid / 68+2k 写序 4 / 72+2k 出生序号 /
/// 76+2k 载荷 CRC / 80+2k 预留 2 / 82+2k 条目数 / 84+2k 条目宽 / 86+2k 预留位 29 / 115+2k 条目区。
/// 头校验和罩 [0, 86+2k)，载荷 CRC 罩 [86+2k, 16384)；条目定宽、key 打头；声明长度 = 条目数 × 条目宽。
///
/// 这里算的是明文头末尾 `86 + 2 × key 宽`：头校验和罩 [0, 这里)、载荷 CRC 从这里罩起。
/// `requires` 的上界只为排除乘法与加法越过 `usize`：调用方交的 key 宽是格式常量或盘上那一个字节，都远小于一个节点。
fn index_node_plain_header_end(key_width: usize) -> (header_end: usize)
    requires
        key_width <= NODE_BYTES,
    ensures
        header_end == specification_index_node_plain_header_end(key_width as int),
{
    let key_range_bytes = usize::try_from(INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT)
        .expect("key 区间两个 key")
        * key_width;
    usize::try_from(INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE).expect("86") + key_range_bytes
}

/// 码 2 索引节点含 key 区间与预留位的头宽，也是条目区的起点：明文头 `86 + 2 × key 宽` 再加预留位 29
/// （D8（核心索引结构） 已定项 11；D18（块里携带什么信息） 已定项 16 / 已定项 18）。
///
/// core 自己的一份式子：checker（`singlefs_checker::index_node_header_bytes`）与理想模型（`singlefs_verification::model::index_node_header_bytes`）
/// 各写各的、互不调用（D13（验证路线） 已定项 5：`singlefs-format` 只放标量），
/// `crates/singlefs-verification/tests/index_node_header_width_computed_three_ways_agrees_for_every_key_width.rs` 在 key 宽的全部取值上把三份连起来比。
///
/// `requires` 同 [`index_node_plain_header_end`]：只排除越过 `usize`，不越过时照式子交回。
#[must_use]
pub fn index_node_header_bytes(key_width: usize) -> (header_bytes: usize)
    requires
        key_width <= NODE_BYTES,
    ensures
        header_bytes == specification_index_node_header_bytes(key_width as int),
{
    index_node_plain_header_end(key_width) + reserved_bytes()
}

/// 一个码 2 节点装得下多少条定宽条目：(16384 − 头 − 预留) / 条目宽。写者在装节点之前拿它判「装不下」并报错，不走到断言。
///
/// # Panics
/// 条目宽是 0（没有「装几条」可算），或 key 宽大到含预留位的头宽过一个节点（16384 减头宽会下溢）。调用方交的都是格式常量
/// （各种树的 key 宽与条目宽），盘上读来的宽不进这里；两种输入都是调用方的 bug。
///
/// `requires` 就是上面两种 panic 的反面：经 Verus 验过的调用方证得出它们，没经 Verus 验的调用方违反时照旧在断言上 panic。
#[must_use]
pub fn index_node_entry_capacity(key_width: usize, entry_width: usize) -> (capacity: usize)
    requires
        entry_width > 0,
        specification_index_node_header_bytes(key_width as int) <= NODE_BYTES,
    ensures
        capacity == specification_index_node_entry_capacity(key_width as int, entry_width as int),
{
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    assert!(
        entry_width > 0,
        "条目宽是 0：index_node_entry_capacity 只收格式常量里的条目宽"
    );
    assert!(
        key_width <= node_bytes && index_node_header_bytes(key_width) <= node_bytes,
        "key 宽 {key_width} 的码 2 节点头宽过一个节点：index_node_entry_capacity 只收格式常量里的 key 宽"
    );
    let entries_start = index_node_header_bytes(key_width);
    (node_bytes - entries_start) / entry_width
}

// ===== 码 2 索引节点 =====

/// 码 2 节点没封的头连同预留位，`[0, 115 + 2k)`，照写者的次序逐段接：共同前缀 42、树 ID 8、层级 1、key 宽 1、key 区间 2k、诞生代号 8、
/// fsid 8、实例代号 4、出生序号 4、载荷 CRC 4（写的时候是 0）、预留 2、条目数 2、条目宽 2、预留位 29。
pub open spec fn specification_index_node_unsealed_header(
    tree: TreeIdentifier,
    level: u8,
    smallest_key: Seq<u8>,
    largest_key: Seq<u8>,
    birth_txg: CheckpointTxg,
    filesystem_identifier: [u8; 16],
    instance: InstanceGeneration,
    birth_sequence: BirthSequence,
    entry_count: nat,
    entry_width: u16,
) -> Seq<u8> {
    specification_common_prefix_bytes(UNIT_CLASS_INDEX_NODE, (entry_count * entry_width) as u16) + little_endian_bytes(
        tree.0 as nat,
        8,
    ) + little_endian_bytes(level as nat, 1) + little_endian_bytes(smallest_key.len(), 1) + smallest_key + largest_key
        + little_endian_bytes(birth_txg.0 as nat, 8) + little_endian_bytes(
        specification_unit_filesystem_identifier(filesystem_identifier) as nat,
        8,
    ) + little_endian_bytes(instance.0 as nat, 4) + little_endian_bytes(birth_sequence.0 as nat, 4) + zero_bytes(4)
        + zero_bytes(2) + little_endian_bytes(entry_count, 2) + little_endian_bytes(entry_width as nat, 2) + zero_bytes(
        NONCE_MAC_ALGORITHM_RESERVED_BYTES as nat,
    )
}

/// 码 2 节点没封的那一份：共同前缀、类身份段、预留位 29、条目依次接起来、补 0 到 16384。
pub open spec fn specification_index_node_unsealed_bytes(
    tree: TreeIdentifier,
    level: u8,
    smallest_key: Seq<u8>,
    largest_key: Seq<u8>,
    birth_txg: CheckpointTxg,
    filesystem_identifier: [u8; 16],
    instance: InstanceGeneration,
    birth_sequence: BirthSequence,
    entry_width: u16,
    entries: Seq<Seq<u8>>,
) -> Seq<u8> {
    let header = specification_index_node_unsealed_header(
        tree,
        level,
        smallest_key,
        largest_key,
        birth_txg,
        filesystem_identifier,
        instance,
        birth_sequence,
        entries.len(),
        entry_width,
    );
    let entry_area = specification_concatenation(entries);
    header + entry_area + zero_bytes((NODE_BYTES - header.len() - entry_area.len()) as nat)
}

/// [`build_index_node`] 写出的整个码 2 节点：载荷 CRC 罩 [86 + 2k, 16384)、写在 76 + 2k；头校验和罩 [0, 86 + 2k)。
pub open spec fn specification_index_node_bytes(
    tree: TreeIdentifier,
    level: u8,
    smallest_key: Seq<u8>,
    largest_key: Seq<u8>,
    birth_txg: CheckpointTxg,
    filesystem_identifier: [u8; 16],
    instance: InstanceGeneration,
    birth_sequence: BirthSequence,
    entry_width: u16,
    entries: Seq<Seq<u8>>,
) -> Seq<u8> {
    let header_end = specification_index_node_plain_header_end(smallest_key.len() as int);
    specification_sealed_unit(
        specification_index_node_unsealed_bytes(
            tree,
            level,
            smallest_key,
            largest_key,
            birth_txg,
            filesystem_identifier,
            instance,
            birth_sequence,
            entry_width,
            entries,
        ),
        header_end,
        header_end - 10,
    )
}

/// [`build_index_node`] 的前提，也就是 [`lemma_index_node_round_trip`] 的前提（往返不另加）：两端 key 都是 key 宽、key 宽装得进一个字节、
/// 条目宽不是 0、条目宽不窄于 key 宽、条目不多于一个节点装得下的、每条条目都是条目宽。
/// 条目宽不窄于 key 宽不论有没有条目都要，与读者那一判（空节点也判）对齐：D8（核心索引结构） 已定项 11 要条目里 key 一律是前 key 宽个字节，
/// I-1.10 要条目宽等于那棵树登记的条目字段表宽度，登记的宽都不窄于那棵树的 key 宽。
pub open spec fn specification_index_node_writer_accepts(
    key_width: int,
    smallest_key: Seq<u8>,
    largest_key: Seq<u8>,
    entry_width: u16,
    entries: Seq<Seq<u8>>,
) -> bool {
    &&& smallest_key.len() == key_width
    &&& largest_key.len() == key_width
    &&& key_width <= u8::MAX
    &&& entry_width > 0
    &&& key_width <= entry_width
    &&& entries.len() <= specification_index_node_entry_capacity(key_width, entry_width as int)
    &&& specification_every_piece_has_width(entries, entry_width as nat)
}

/// # Panics
/// 两端 key 不是 `key_width` 宽、`key_width` 装不进一个字节、条目宽是 0、条目多于一个节点装得下的、或有一条条目的宽不是 `entry_width`
/// 或窄于 key 宽（`requires` 排除的几种：写者先按 [`index_node_entry_capacity`] 判「装不下」并报错，不走到这里）。
///
/// `requires` 另要空节点的条目宽也不窄于 key 宽（[`specification_index_node_writer_accepts`]）：只约束经 Verus 验过的调用方，函数体只在有条目时
/// 判它（逐条的那一句断言），空节点不为它 panic；今天的调用方都在 `verus!` 之外，传的 key 宽与条目宽都是登记表里的格式常量。
#[allow(
    clippy::too_many_arguments,
    reason = "字段表就是这么多段，收成结构体只会多一层没人验的名字"
)]
#[must_use]
pub fn build_index_node(
    tree: TreeIdentifier,
    level: u8,
    key_width: usize,
    smallest_key: &[u8],
    largest_key: &[u8],
    birth_txg: CheckpointTxg,
    filesystem_identifier: &[u8; 16],
    instance: InstanceGeneration,
    birth_sequence: BirthSequence,
    entry_width: u16,
    entries: &[Vec<u8>],
) -> (node: Vec<u8>)
    requires
        specification_index_node_writer_accepts(
            key_width as int,
            smallest_key@,
            largest_key@,
            entry_width,
            specification_byte_sequences_of_vectors(entries@),
        ),
    ensures
        node@ == specification_index_node_bytes(
            tree,
            level,
            smallest_key@,
            largest_key@,
            birth_txg,
            *filesystem_identifier,
            instance,
            birth_sequence,
            entry_width,
            specification_byte_sequences_of_vectors(entries@),
        ),
{
    let ghost entry_pieces = specification_byte_sequences_of_vectors(entries@);
    proof {
        assert(entries@.len() * entry_width <= NODE_BYTES - specification_index_node_header_bytes(key_width as int)) by (nonlinear_arith)
            requires
                entries@.len() <= (NODE_BYTES - specification_index_node_header_bytes(key_width as int)) / (entry_width as int),
                entry_width > 0,
                specification_index_node_header_bytes(key_width as int) <= NODE_BYTES,
        ;
    }
    // assert_eq! 展开成的 core::panicking::assert_failed 没有 Verus 规格，写成 assert! 加同一句话。
    assert!(smallest_key.len() == key_width, "最小 key 是 key 宽个字节");
    assert!(largest_key.len() == key_width, "最大 key 是 key 宽个字节");
    let header_end = index_node_plain_header_end(key_width);
    let declared_length = entries.len() * usize::from(entry_width);
    assert!(
        entries.len() <= index_node_entry_capacity(key_width, usize::from(entry_width)),
        "条目装不进一个节点：写者要先按 index_node_entry_capacity 判、报错，不许走到这里"
    );
    let mut writer = ByteWriter::new(usize::try_from(NODE_BYTES).expect("16384"));
    write_common_prefix(
        &mut writer,
        UNIT_CLASS_INDEX_NODE,
        u16::try_from(declared_length).expect("声明长度 2 字节"),
    );
    writer.put_u64(tree.0);
    writer.put_u8(level);
    writer.assert_position(51, "码 2 的 key 宽字段");
    writer.put_u8(u8::try_from(key_width).expect("key 宽 1 字节"));
    writer.put(smallest_key);
    writer.put(largest_key);
    writer.put_u64(birth_txg.0);
    writer.put_u64(unit_filesystem_identifier(filesystem_identifier));
    writer.put_u32(instance.0);
    writer.put_u32(birth_sequence.0);
    let payload_crc_offset = writer.position();
    writer.skip(4);
    writer.skip(2);
    writer.put_u16(u16::try_from(entries.len()).expect("条目数 2 字节"));
    writer.put_u16(entry_width);
    writer.assert_position(u64::try_from(header_end).expect("头末尾"), "码 2 头");
    writer.skip(reserved_bytes());
    let ghost header = writer.written();
    assert(header == specification_index_node_unsealed_header(
        tree,
        level,
        smallest_key@,
        largest_key@,
        birth_txg,
        *filesystem_identifier,
        instance,
        birth_sequence,
        entries@.len(),
        entry_width,
    ));
    // for 循环换成下标循环（Verus 对切片迭代器的规格不全），逐条核条目宽与 key 宽、逐条写，次序不变。
    let mut entry_index: usize = 0;
    while entry_index < entries.len()
        invariant
            writer.is_well_formed(),
            writer.total_bytes() == NODE_BYTES,
            header.len() == specification_index_node_header_bytes(key_width as int),
            entry_pieces == specification_byte_sequences_of_vectors(entries@),
            entry_index <= entries@.len(),
            entries@.len() * entry_width <= NODE_BYTES - specification_index_node_header_bytes(key_width as int),
            key_width <= entry_width,
            specification_every_piece_has_width(entry_pieces, entry_width as nat),
            writer.written() == header + specification_concatenation(entry_pieces.subrange(0, entry_index as int)),
        decreases entries@.len() - entry_index,
    {
        let entry = &entries[entry_index];
        proof {
            assert(entry_pieces[entry_index as int] == entry@);
            assert(entry_pieces[entry_index as int].len() == entry_width);
        }
        // assert_eq! 展开成的 core::panicking::assert_failed 没有 Verus 规格，写成 assert! 加同一句话。
        assert!(
            entry.len() == usize::from(entry_width),
            "条目宽与头里写的不一致"
        );
        assert!(
            entry.len() >= key_width,
            "条目里 key 一律是前 key 宽 个字节"
        );
        proof {
            let written_pieces = entry_pieces.subrange(0, entry_index as int);
            assert(specification_every_piece_has_width(written_pieces, entry_width as nat));
            lemma_concatenation_of_pieces_of_one_width(written_pieces, entry_width as nat);
            assert(entry_index * entry_width + entry_width <= entries@.len() * entry_width) by (nonlinear_arith)
                requires
                    entry_index < entries@.len(),
            ;
            assert(entry_pieces[entry_index as int] == entry@);
        }
        writer.put(entry.as_slice());
        proof {
            let longer = entry_pieces.subrange(0, entry_index + 1);
            assert(longer.drop_last() =~= entry_pieces.subrange(0, entry_index as int));
            assert(longer.last() == entry@);
        }
        entry_index += 1;
    }
    proof {
        assert(entry_pieces.subrange(0, entries@.len() as int) =~= entry_pieces);
        lemma_concatenation_of_pieces_of_one_width(entry_pieces, entry_width as nat);
    }
    let mut bytes = writer.into_bytes();
    assert(bytes@ =~= specification_index_node_unsealed_bytes(
        tree,
        level,
        smallest_key@,
        largest_key@,
        birth_txg,
        *filesystem_identifier,
        instance,
        birth_sequence,
        entry_width,
        entry_pieces,
    ));
    seal_payload_checksum(bytes.as_mut_slice(), header_end, payload_crc_offset);
    seal_header_checksum(bytes.as_mut_slice(), header_end);
    bytes
}

/// 码 2 索引节点解出来的头与条目。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexNodeHeader {
    pub tree: TreeIdentifier,
    pub level: u8,
    pub key_width: usize,
    pub smallest_key: Vec<u8>,
    pub largest_key: Vec<u8>,
    pub birth_txg: CheckpointTxg,
    pub filesystem_identifier: u64,
    pub instance: InstanceGeneration,
    pub birth_sequence: BirthSequence,
    pub entry_width: usize,
    pub entries: Vec<Vec<u8>>,
}

/// [`IndexNodeHeader`] 在规格里的样子：`Vec` 换成它们的字节。
pub struct IndexNodeHeaderModel {
    pub tree: TreeIdentifier,
    pub level: u8,
    pub key_width: nat,
    pub smallest_key: Seq<u8>,
    pub largest_key: Seq<u8>,
    pub birth_txg: CheckpointTxg,
    pub filesystem_identifier: u64,
    pub instance: InstanceGeneration,
    pub birth_sequence: BirthSequence,
    pub entry_width: nat,
    pub entries: Seq<Seq<u8>>,
}

impl IndexNodeHeader {
    pub open spec fn specification_model(&self) -> IndexNodeHeaderModel {
        IndexNodeHeaderModel {
            tree: self.tree,
            level: self.level,
            key_width: self.key_width as nat,
            smallest_key: self.smallest_key@,
            largest_key: self.largest_key@,
            birth_txg: self.birth_txg,
            filesystem_identifier: self.filesystem_identifier,
            instance: self.instance,
            birth_sequence: self.birth_sequence,
            entry_width: self.entry_width as nat,
            entries: specification_byte_sequences_of_vectors(self.entries@),
        }
    }
}

/// [`parse_index_node`] 的判法（交回的是 [`IndexNodeHeaderModel`]）。
pub open spec fn specification_parse_index_node(bytes: Seq<u8>) -> Result<IndexNodeHeaderModel, UnitError> {
    if bytes.len() != NODE_BYTES {
        Err(UnitError::TooShort)
    } else {
        let key_width = bytes[51] as int;
        let header_end = specification_index_node_plain_header_end(key_width);
        match specification_check_common_prefix(bytes, UNIT_CLASS_INDEX_NODE, header_end) {
            Err(error) => Err(error),
            Ok(declared_length) => {
                let entry_count = little_endian_value(bytes.subrange(header_end - 4, header_end - 2));
                let entry_width = little_endian_value(bytes.subrange(header_end - 2, header_end));
                let entries_start = header_end + NONCE_MAC_ALGORITHM_RESERVED_BYTES;
                if !specification_payload_checksum_holds(bytes, header_end, header_end - 10) {
                    Err(UnitError::PayloadChecksumMismatch)
                } else if !specification_encryption_reserved_bytes_are_zero(bytes, header_end) {
                    Err(UnitError::EncryptionReservedBytesNotZero)
                } else if entry_count * entry_width != declared_length {
                    Err(UnitError::Structure(UnitStructureDefect::IndexNodeDeclaredLengthIsNotEntryCountTimesEntryWidth))
                } else if entry_width == 0 && entry_count != 0 {
                    Err(UnitError::Structure(UnitStructureDefect::IndexNodeEntryWidthZeroWithEntries))
                } else if entry_width < key_width {
                    Err(UnitError::Structure(UnitStructureDefect::IndexNodeEntryWidthBelowTheKeyWidth))
                } else if entries_start + declared_length > bytes.len() {
                    Err(UnitError::Structure(UnitStructureDefect::IndexNodeEntryAreaPastTheNodeEnd))
                } else {
                    Ok(
                        IndexNodeHeaderModel {
                            tree: TreeIdentifier(little_endian_value(bytes.subrange(42, 50)) as u64),
                            level: little_endian_value(bytes.subrange(50, 51)) as u8,
                            key_width: key_width as nat,
                            smallest_key: bytes.subrange(52, 52 + key_width),
                            largest_key: bytes.subrange(52 + key_width, 52 + 2 * key_width),
                            birth_txg: CheckpointTxg(
                                little_endian_value(bytes.subrange(52 + 2 * key_width, 60 + 2 * key_width)) as u64,
                            ),
                            filesystem_identifier: little_endian_value(
                                bytes.subrange(60 + 2 * key_width, 68 + 2 * key_width),
                            ) as u64,
                            instance: InstanceGeneration(
                                little_endian_value(bytes.subrange(68 + 2 * key_width, 72 + 2 * key_width)) as u32,
                            ),
                            birth_sequence: BirthSequence(
                                little_endian_value(bytes.subrange(72 + 2 * key_width, 76 + 2 * key_width)) as u32,
                            ),
                            entry_width,
                            entries: specification_pieces_at(bytes, entries_start, entry_count, entry_width),
                        },
                    )
                }
            },
        }
    }
}

/// 解一个码 2 节点：key 宽从偏移 51 自述、头校验和罩 [0, 86+2k)、载荷 CRC 罩 [86+2k, 16384)、声明长度 = 条目数 × 条目宽。
pub fn parse_index_node(bytes: &[u8]) -> (parsed: Result<IndexNodeHeader, UnitError>)
    ensures
        match parsed {
            Ok(header) => specification_parse_index_node(bytes@) == Ok::<IndexNodeHeaderModel, UnitError>(header.specification_model()),
            Err(error) => specification_parse_index_node(bytes@) == Err::<IndexNodeHeaderModel, UnitError>(error),
        },
{
    if bytes.len() != usize::try_from(NODE_BYTES).expect("16384") {
        return Err(UnitError::TooShort);
    }
    let key_width = usize::from(bytes[51]);
    let header_end = index_node_plain_header_end(key_width);
    let declared_length = usize::from(check_common_prefix(
        bytes,
        UNIT_CLASS_INDEX_NODE,
        header_end,
    )?);
    check_payload_checksum(bytes, header_end, header_end - 10)?;
    check_encryption_reserved_bytes_are_zero(bytes, header_end)?;
    let mut reader = ByteReader::at(bytes, 42);
    let tree = TreeIdentifier(reader.get_u64());
    let level = reader.get_u8();
    reader.skip(1);
    // to_vec 没有 Verus 规格：copied_into_a_vector 拷成同样的一份。
    let smallest_key = copied_into_a_vector(reader.take(key_width));
    let largest_key = copied_into_a_vector(reader.take(key_width));
    let birth_txg = CheckpointTxg(reader.get_u64());
    let filesystem_identifier = reader.get_u64();
    let instance = InstanceGeneration(reader.get_u32());
    let birth_sequence = BirthSequence(reader.get_u32());
    reader.skip(4 + 2);
    let entry_count = usize::from(reader.get_u16());
    let entry_width = usize::from(reader.get_u16());
    // assert_eq! 展开成的 core::panicking::assert_failed 没有 Verus 规格，写成 assert! 加一句话。
    assert!(reader.position() == header_end, "码 2 头读到明文头末尾");
    proof {
        assert(entry_count * entry_width <= 0xFFFF * 0xFFFF) by (nonlinear_arith)
            requires
                entry_count <= 0xFFFF,
                entry_width <= 0xFFFF,
        ;
    }
    if entry_count * entry_width != declared_length {
        return Err(UnitError::Structure(
            UnitStructureDefect::IndexNodeDeclaredLengthIsNotEntryCountTimesEntryWidth,
        ));
    }
    // 条目宽 0 而条目数不为 0：声明长度 0 过得了上一判，切出来的条目却一条都没有，读者看到的条数与头里写的不符（代码审阅第 38 条）。
    if entry_width == 0 && entry_count != 0 {
        return Err(UnitError::Structure(
            UnitStructureDefect::IndexNodeEntryWidthZeroWithEntries,
        ));
    }
    if entry_width < key_width {
        return Err(UnitError::Structure(
            UnitStructureDefect::IndexNodeEntryWidthBelowTheKeyWidth,
        ));
    }
    let entries_start = header_end + reserved_bytes();
    if entries_start + declared_length > bytes.len() {
        return Err(UnitError::Structure(
            UnitStructureDefect::IndexNodeEntryAreaPastTheNodeEnd,
        ));
    }
    let entries = pieces_of_one_width(bytes, entries_start, entry_count, entry_width);
    Ok(IndexNodeHeader {
        tree,
        level,
        key_width,
        smallest_key,
        largest_key,
        birth_txg,
        filesystem_identifier,
        instance,
        birth_sequence,
        entry_width,
        entries,
    })
}

/// 码 2 节点没封的那一份逐段剥开（[`lemma_index_node_round_trip`] 拆出来的前一半）：照写者的次序，每个字段落在字段表上的那一截。
pub proof fn lemma_index_node_unsealed_fields(
    tree: TreeIdentifier,
    level: u8,
    smallest_key: Seq<u8>,
    largest_key: Seq<u8>,
    birth_txg: CheckpointTxg,
    filesystem_identifier: [u8; 16],
    instance: InstanceGeneration,
    birth_sequence: BirthSequence,
    entry_width: u16,
    entries: Seq<Seq<u8>>,
)
    requires
        largest_key.len() == smallest_key.len(),
        smallest_key.len() <= u8::MAX,
        entries.len() * entry_width <= NODE_BYTES - specification_index_node_header_bytes(smallest_key.len() as int),
        specification_every_piece_has_width(entries, entry_width as nat),
    ensures
        ({
            let key_width = smallest_key.len() as int;
            let header_end = 86 + 2 * key_width;
            let unsealed = specification_index_node_unsealed_bytes(
                tree,
                level,
                smallest_key,
                largest_key,
                birth_txg,
                filesystem_identifier,
                instance,
                birth_sequence,
                entry_width,
                entries,
            );
            &&& unsealed.len() == NODE_BYTES
            &&& unsealed.subrange(0, 42) == specification_common_prefix_bytes(
                UNIT_CLASS_INDEX_NODE,
                (entries.len() * entry_width) as u16,
            )
            &&& unsealed.subrange(42, 50) == little_endian_bytes(tree.0 as nat, 8)
            &&& unsealed.subrange(50, 51) == little_endian_bytes(level as nat, 1)
            &&& unsealed.subrange(51, 52) == little_endian_bytes(smallest_key.len(), 1)
            &&& unsealed.subrange(52, 52 + key_width) == smallest_key
            &&& unsealed.subrange(52 + key_width, 52 + 2 * key_width) == largest_key
            &&& unsealed.subrange(52 + 2 * key_width, 60 + 2 * key_width) == little_endian_bytes(birth_txg.0 as nat, 8)
            &&& unsealed.subrange(60 + 2 * key_width, 68 + 2 * key_width) == little_endian_bytes(
                specification_unit_filesystem_identifier(filesystem_identifier) as nat,
                8,
            )
            &&& unsealed.subrange(68 + 2 * key_width, 72 + 2 * key_width) == little_endian_bytes(instance.0 as nat, 4)
            &&& unsealed.subrange(72 + 2 * key_width, 76 + 2 * key_width) == little_endian_bytes(
                birth_sequence.0 as nat,
                4,
            )
            &&& unsealed.subrange(header_end - 4, header_end - 2) == little_endian_bytes(entries.len(), 2)
            &&& unsealed.subrange(header_end - 2, header_end) == little_endian_bytes(entry_width as nat, 2)
            &&& unsealed.subrange(header_end, header_end + 29) == zero_bytes(29)
            &&& unsealed.subrange(header_end + 29, header_end + 29 + entries.len() * entry_width) == specification_concatenation(
                entries,
            )
        }),
{
    lemma_little_endian_capacities();
    lemma_concatenation_of_pieces_of_one_width(entries, entry_width as nat);
    let entry_count = entries.len();
    let declared_length = (entry_count * entry_width) as u16;
    let header = specification_index_node_unsealed_header(
        tree,
        level,
        smallest_key,
        largest_key,
        birth_txg,
        filesystem_identifier,
        instance,
        birth_sequence,
        entry_count,
        entry_width,
    );
    let entry_area = specification_concatenation(entries);
    let unsealed = specification_index_node_unsealed_bytes(
        tree,
        level,
        smallest_key,
        largest_key,
        birth_txg,
        filesystem_identifier,
        instance,
        birth_sequence,
        entry_width,
        entries,
    );
    let padding = zero_bytes((NODE_BYTES - header.len() - entry_area.len()) as nat);
    assert(unsealed.subrange(0, unsealed.len() as int) =~= unsealed);
    lemma_split_a_concatenated_range(unsealed, 0, header + entry_area, padding);
    lemma_split_a_concatenated_range(unsealed, 0, header, entry_area);
    let prefix = specification_common_prefix_bytes(UNIT_CLASS_INDEX_NODE, declared_length);
    let tree_bytes = little_endian_bytes(tree.0 as nat, 8);
    let level_bytes = little_endian_bytes(level as nat, 1);
    let key_width_bytes = little_endian_bytes(smallest_key.len(), 1);
    let birth = little_endian_bytes(birth_txg.0 as nat, 8);
    let fsid = little_endian_bytes(specification_unit_filesystem_identifier(filesystem_identifier) as nat, 8);
    let instance_bytes = little_endian_bytes(instance.0 as nat, 4);
    let sequence = little_endian_bytes(birth_sequence.0 as nat, 4);
    let crc_field = zero_bytes(4);
    let gap = zero_bytes(2);
    let count = little_endian_bytes(entry_count, 2);
    let width = little_endian_bytes(entry_width as nat, 2);
    let reserved = zero_bytes(29);
    let through_keys = prefix + tree_bytes + level_bytes + key_width_bytes + smallest_key + largest_key;
    let through_sequence = through_keys + birth + fsid + instance_bytes + sequence;
    lemma_split_a_concatenated_range(unsealed, 0, through_sequence + crc_field + gap + count + width, reserved);
    lemma_split_a_concatenated_range(unsealed, 0, through_sequence + crc_field + gap + count, width);
    lemma_split_a_concatenated_range(unsealed, 0, through_sequence + crc_field + gap, count);
    lemma_split_a_concatenated_range(unsealed, 0, through_sequence + crc_field, gap);
    lemma_split_a_concatenated_range(unsealed, 0, through_sequence, crc_field);
    lemma_split_a_concatenated_range(unsealed, 0, through_keys + birth + fsid + instance_bytes, sequence);
    lemma_split_a_concatenated_range(unsealed, 0, through_keys + birth + fsid, instance_bytes);
    lemma_split_a_concatenated_range(unsealed, 0, through_keys + birth, fsid);
    lemma_split_a_concatenated_range(unsealed, 0, through_keys, birth);
    lemma_split_a_concatenated_range(unsealed, 0, prefix + tree_bytes + level_bytes + key_width_bytes + smallest_key, largest_key);
    lemma_split_a_concatenated_range(unsealed, 0, prefix + tree_bytes + level_bytes + key_width_bytes, smallest_key);
    lemma_split_a_concatenated_range(unsealed, 0, prefix + tree_bytes + level_bytes, key_width_bytes);
    lemma_split_a_concatenated_range(unsealed, 0, prefix + tree_bytes, level_bytes);
    lemma_split_a_concatenated_range(unsealed, 0, prefix, tree_bytes);
}

/// 码 2 节点封好之后各字段还在字段表上的那一截，两个校验和两判都过（[`lemma_index_node_round_trip`] 拆出来的中间一段）。
pub proof fn lemma_index_node_sealed_fields(
    tree: TreeIdentifier,
    level: u8,
    smallest_key: Seq<u8>,
    largest_key: Seq<u8>,
    birth_txg: CheckpointTxg,
    filesystem_identifier: [u8; 16],
    instance: InstanceGeneration,
    birth_sequence: BirthSequence,
    entry_width: u16,
    entries: Seq<Seq<u8>>,
)
    requires
        largest_key.len() == smallest_key.len(),
        smallest_key.len() <= u8::MAX,
        entries.len() * entry_width <= NODE_BYTES - specification_index_node_header_bytes(smallest_key.len() as int),
        specification_every_piece_has_width(entries, entry_width as nat),
    ensures
        ({
            let key_width = smallest_key.len() as int;
            let header_end = 86 + 2 * key_width;
            let node = specification_index_node_bytes(
                tree,
                level,
                smallest_key,
                largest_key,
                birth_txg,
                filesystem_identifier,
                instance,
                birth_sequence,
                entry_width,
                entries,
            );
            &&& node.len() == NODE_BYTES
            &&& node.subrange(0, 10) == specification_common_prefix_bytes(
                UNIT_CLASS_INDEX_NODE,
                (entries.len() * entry_width) as u16,
            ).subrange(0, 10)
            &&& node[51] as int == key_width
            &&& node.subrange(42, 50) == little_endian_bytes(tree.0 as nat, 8)
            &&& node.subrange(50, 51) == little_endian_bytes(level as nat, 1)
            &&& node.subrange(52, 52 + key_width) == smallest_key
            &&& node.subrange(52 + key_width, 52 + 2 * key_width) == largest_key
            &&& node.subrange(52 + 2 * key_width, 60 + 2 * key_width) == little_endian_bytes(birth_txg.0 as nat, 8)
            &&& node.subrange(60 + 2 * key_width, 68 + 2 * key_width) == little_endian_bytes(
                specification_unit_filesystem_identifier(filesystem_identifier) as nat,
                8,
            )
            &&& node.subrange(68 + 2 * key_width, 72 + 2 * key_width) == little_endian_bytes(instance.0 as nat, 4)
            &&& node.subrange(72 + 2 * key_width, 76 + 2 * key_width) == little_endian_bytes(birth_sequence.0 as nat, 4)
            &&& node.subrange(header_end - 4, header_end - 2) == little_endian_bytes(entries.len(), 2)
            &&& node.subrange(header_end - 2, header_end) == little_endian_bytes(entry_width as nat, 2)
            &&& specification_header_checksum_holds(node, header_end)
            &&& specification_payload_checksum_holds(node, header_end, header_end - 10)
            &&& node.subrange(header_end, header_end + 29) == zero_bytes(29)
            &&& node.subrange(header_end + 29, header_end + 29 + entries.len() * entry_width) == specification_concatenation(entries)
        }),
{
    let key_width = smallest_key.len() as int;
    let entry_count = entries.len();
    let header_end = 86 + 2 * key_width;
    let unsealed = specification_index_node_unsealed_bytes(
        tree,
        level,
        smallest_key,
        largest_key,
        birth_txg,
        filesystem_identifier,
        instance,
        birth_sequence,
        entry_width,
        entries,
    );
    lemma_index_node_unsealed_fields(
        tree,
        level,
        smallest_key,
        largest_key,
        birth_txg,
        filesystem_identifier,
        instance,
        birth_sequence,
        entry_width,
        entries,
    );
    lemma_sealed_unit_keeps_everything_outside_the_two_checksums(unsealed, header_end, header_end - 10);
    let node = specification_index_node_bytes(
        tree,
        level,
        smallest_key,
        largest_key,
        birth_txg,
        filesystem_identifier,
        instance,
        birth_sequence,
        entry_width,
        entries,
    );
    assert(unsealed.subrange(0, 10) =~= unsealed.subrange(0, 42).subrange(0, 10));
    assert(node.subrange(0, 10) == unsealed.subrange(0, 10));
    assert(node.subrange(51, 52) == unsealed.subrange(51, 52));
    lemma_one_byte_little_endian(smallest_key.len() as u8);
    assert(node[51] == node.subrange(51, 52)[0]);
    assert(node.subrange(header_end, header_end + 29) == unsealed.subrange(header_end, header_end + 29));
    assert(node.subrange(42, 50) == unsealed.subrange(42, 50));
    assert(node.subrange(50, 51) == unsealed.subrange(50, 51));
    assert(node.subrange(52, 52 + key_width) == unsealed.subrange(52, 52 + key_width));
    assert(node.subrange(52 + key_width, 52 + 2 * key_width) == unsealed.subrange(52 + key_width, 52 + 2 * key_width));
    assert(node.subrange(52 + 2 * key_width, 60 + 2 * key_width) == unsealed.subrange(52 + 2 * key_width, 60 + 2 * key_width));
    assert(node.subrange(60 + 2 * key_width, 68 + 2 * key_width) == unsealed.subrange(60 + 2 * key_width, 68 + 2 * key_width));
    assert(node.subrange(68 + 2 * key_width, 72 + 2 * key_width) == unsealed.subrange(68 + 2 * key_width, 72 + 2 * key_width));
    assert(node.subrange(72 + 2 * key_width, 76 + 2 * key_width) == unsealed.subrange(72 + 2 * key_width, 76 + 2 * key_width));
    assert(node.subrange(header_end - 4, header_end - 2) == unsealed.subrange(header_end - 4, header_end - 2));
    assert(node.subrange(header_end - 2, header_end) == unsealed.subrange(header_end - 2, header_end));
    assert(node.subrange(header_end + 29, header_end + 29 + entry_count * entry_width) == unsealed.subrange(
        header_end + 29,
        header_end + 29 + entry_count * entry_width,
    ));
}

/// 码 2 索引节点往返：[`build_index_node`] 写出的节点，[`parse_index_node`] 读回同一份头与同样的条目。
/// 前提就是写者的前提：读者对空节点也判条目宽不窄于 key 宽，写者的前提里不论有没有条目都要它，所以往返不另加。
pub proof fn lemma_index_node_round_trip(
    tree: TreeIdentifier,
    level: u8,
    smallest_key: Seq<u8>,
    largest_key: Seq<u8>,
    birth_txg: CheckpointTxg,
    filesystem_identifier: [u8; 16],
    instance: InstanceGeneration,
    birth_sequence: BirthSequence,
    entry_width: u16,
    entries: Seq<Seq<u8>>,
)
    requires
        specification_index_node_writer_accepts(smallest_key.len() as int, smallest_key, largest_key, entry_width, entries),
    ensures
        specification_parse_index_node(
            specification_index_node_bytes(
                tree,
                level,
                smallest_key,
                largest_key,
                birth_txg,
                filesystem_identifier,
                instance,
                birth_sequence,
                entry_width,
                entries,
            ),
        ) == Ok::<IndexNodeHeaderModel, UnitError>(
            IndexNodeHeaderModel {
                tree,
                level,
                key_width: smallest_key.len(),
                smallest_key,
                largest_key,
                birth_txg,
                filesystem_identifier: specification_unit_filesystem_identifier(filesystem_identifier),
                instance,
                birth_sequence,
                entry_width: entry_width as nat,
                entries,
            },
        ),
{
    lemma_little_endian_capacities();
    let key_width = smallest_key.len() as int;
    let entry_count = entries.len();
    assert(entry_count * entry_width <= NODE_BYTES - specification_index_node_header_bytes(key_width)) by (nonlinear_arith)
        requires
            entry_count <= (NODE_BYTES - specification_index_node_header_bytes(key_width)) / (entry_width as int),
            entry_width > 0,
            specification_index_node_header_bytes(key_width) <= NODE_BYTES,
    ;
    let declared_length = (entry_count * entry_width) as u16;
    let header_end = 86 + 2 * key_width;
    lemma_index_node_sealed_fields(
        tree,
        level,
        smallest_key,
        largest_key,
        birth_txg,
        filesystem_identifier,
        instance,
        birth_sequence,
        entry_width,
        entries,
    );
    let node = specification_index_node_bytes(
        tree,
        level,
        smallest_key,
        largest_key,
        birth_txg,
        filesystem_identifier,
        instance,
        birth_sequence,
        entry_width,
        entries,
    );
    lemma_common_prefix_reads_back(node, UNIT_CLASS_INDEX_NODE, declared_length);
    lemma_pieces_at_a_concatenation(node, header_end + 29, entries, entry_width as nat);
    assert(specification_check_common_prefix(node, UNIT_CLASS_INDEX_NODE, header_end) == Ok::<u16, UnitError>(declared_length));
    assert(little_endian_value(node.subrange(header_end - 4, header_end - 2)) == entry_count);
    assert(little_endian_value(node.subrange(header_end - 2, header_end)) == entry_width);
}

} // verus!

#[cfg(test)]
mod tests {
    use super::*;

    use crate::checksum::wide_checksum_field_holds;

    /// 条目宽 0：在开头那一句断言上 panic，不走到除法上除零（外部代码审查第二条）。
    #[test]
    #[should_panic(expected = "条目宽是 0")]
    fn index_node_entry_capacity_of_a_zero_entry_width_panics_on_its_precondition() {
        let _ = index_node_entry_capacity(8, 0);
    }

    /// key 宽 8135：含预留位的头宽 86 + 2 × 8135 + 29 = 16385，宽过一个节点；在断言上 panic，不走到 16384 减头宽的下溢上。
    #[test]
    #[should_panic(expected = "的码 2 节点头宽过一个节点")]
    fn index_node_entry_capacity_of_a_key_width_whose_header_overruns_the_node_panics_on_its_precondition(
    ) {
        let _ = index_node_entry_capacity(8135, 8);
    }

    /// key 宽 8134：头宽 86 + 2 × 8134 + 29 = 16383，还装得下一个节点；条目宽 2 时一条都装不下，交 0（写者据此报「装不下」），不 panic。
    /// 头宽的那一判是「不宽过节点」，不是「留得下一条」。
    #[test]
    fn index_node_entry_capacity_of_the_widest_key_whose_header_fits_the_node_is_zero_when_no_entry_fits(
    ) {
        assert_eq!(
            index_node_header_bytes(8134),
            16383,
            "头宽 86 + 2 × 8134 + 29"
        );
        assert_eq!(
            index_node_entry_capacity(8134, 2),
            0,
            "16384 − 16383 = 1 字节，条目宽 2 一条都装不下"
        );
    }

    #[test]
    fn empty_tree_table_node_has_key_width_at_51_and_a_sealed_header() {
        let filesystem_identifier = [9u8; 16];
        let node = build_index_node(
            TreeIdentifier(0),
            0,
            8,
            &[0u8; 8],
            &[0u8; 8],
            CheckpointTxg(0),
            &filesystem_identifier,
            InstanceGeneration(0),
            BirthSequence(1),
            200,
            &[],
        );
        assert_eq!(node.len(), 16384);
        assert_eq!(&node[..4], b"SFSU");
        assert_eq!(node[6], UNIT_CLASS_INDEX_NODE);
        assert_eq!(node[51], 8, "key 宽住偏移 51");
        assert_eq!(
            u16::from_le_bytes([node[8], node[9]]),
            0,
            "0 条条目 ⇒ 声明长度 0"
        );
        assert_eq!(
            u16::from_le_bytes([node[84 + 16], node[85 + 16]]),
            200,
            "条目宽在 84 + 2k"
        );
        assert!(wide_checksum_field_holds(
            &node,
            86 + 16,
            HEADER_CHECKSUM_OFFSET
        ));
        let payload_crc = u32::from_le_bytes(node[76 + 16..80 + 16].try_into().expect("4"));
        assert_eq!(
            payload_crc,
            crc32_castagnoli(&node[86 + 16..]),
            "载荷 CRC 从明文头末尾起、含预留位"
        );
    }
}
