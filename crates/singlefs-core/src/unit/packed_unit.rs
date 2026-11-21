//! 码 3 打包单元：写法与读法的规格、`build_packed_unit`、`parse_packed_unit` 与往返引理。

use super::*;

verus! {

broadcast use group_little_endian_round_trips;

// ===== 码 3 打包记录单元 =====

/// 码 3 单元没封的头连同预留位，`[0, 136)`，照写者的次序逐段接：共同前缀 42、类标签副本 1、身份 26（出生树 8 + 打包记录类型 2
/// + 容器号 8 + 容器出生代 8）、记录数 2、记录宽 2、诞生代号 8、fsid 8、载荷 CRC 4（写的时候是 0）、写序 10（实例代号 4 + 事务号 6）、
/// 出生序号 4、预留位 29。
pub open spec fn specification_packed_unit_unsealed_header(
    identity: PackedIdentity,
    record_count: nat,
    record_width: u16,
    birth_txg: CheckpointTxg,
    filesystem_identifier: [u8; 16],
    write_order: WriteOrder,
    birth_sequence: BirthSequence,
) -> Seq<u8> {
    specification_common_prefix_bytes(UNIT_CLASS_PACKED, (record_count * record_width) as u16) + little_endian_bytes(
        UNIT_CLASS_PACKED as nat,
        1,
    ) + little_endian_bytes(identity.birth_tree.0 as nat, 8) + little_endian_bytes(identity.record_type as nat, 2)
        + little_endian_bytes(identity.container as nat, 8) + little_endian_bytes(identity.container_birth.0 as nat, 8)
        + little_endian_bytes(record_count, 2) + little_endian_bytes(record_width as nat, 2) + little_endian_bytes(
        birth_txg.0 as nat,
        8,
    ) + little_endian_bytes(specification_unit_filesystem_identifier(filesystem_identifier) as nat, 8) + zero_bytes(4)
        + little_endian_bytes(write_order.instance.0 as nat, 4) + little_endian_bytes(write_order.transaction as nat, 6)
        + little_endian_bytes(birth_sequence.0 as nat, 4) + zero_bytes(NONCE_MAC_ALGORITHM_RESERVED_BYTES as nat)
}

/// 码 3 单元没封的那一份：共同前缀、类身份段、记录依次接在 136 之后、补 0 到 32768。
pub open spec fn specification_packed_unit_unsealed_bytes(
    identity: PackedIdentity,
    record_width: u16,
    records: Seq<Seq<u8>>,
    birth_txg: CheckpointTxg,
    filesystem_identifier: [u8; 16],
    write_order: WriteOrder,
    birth_sequence: BirthSequence,
) -> Seq<u8> {
    let header = specification_packed_unit_unsealed_header(
        identity,
        records.len(),
        record_width,
        birth_txg,
        filesystem_identifier,
        write_order,
        birth_sequence,
    );
    let record_area = specification_concatenation(records);
    header + record_area + zero_bytes((DATA_UNIT_BYTES - header.len() - record_area.len()) as nat)
}

/// [`build_packed_unit`] 写出的整个码 3 单元：载荷 CRC 罩 [107, 32768)、写在 89；头校验和罩 [0, 107)。
pub open spec fn specification_packed_unit_bytes(
    identity: PackedIdentity,
    record_width: u16,
    records: Seq<Seq<u8>>,
    birth_txg: CheckpointTxg,
    filesystem_identifier: [u8; 16],
    write_order: WriteOrder,
    birth_sequence: BirthSequence,
) -> Seq<u8> {
    specification_sealed_unit(
        specification_packed_unit_unsealed_bytes(
            identity,
            record_width,
            records,
            birth_txg,
            filesystem_identifier,
            write_order,
            birth_sequence,
        ),
        PACKED_UNIT_HEADER_BYTES as int,
        89,
    )
}

/// 一个码 3 单元的记录区装得下的字节：32768 − 头 107 − 预留位 29。
pub open spec fn specification_packed_unit_record_area_bytes() -> int {
    DATA_UNIT_BYTES - PACKED_UNIT_HEADER_BYTES - NONCE_MAC_ALGORITHM_RESERVED_BYTES
}

/// [`build_packed_unit`] 的前提，也就是 [`lemma_packed_unit_round_trip`] 的前提（往返不另加）：写序的事务号装得进 6 字节、
/// 记录宽不是 0、记录数装得进 2 字节、记录区装得进一个单元、每条记录都是记录宽。
/// 记录宽不是 0 与读者那一判（记录宽 0 而记录数不为 0 就拒）对齐：I-1.7 要记录宽等于登记表为那个打包记录类型登记的宽，登记的宽都不是 0。
pub open spec fn specification_packed_unit_writer_accepts(write_order: WriteOrder, record_width: u16, records: Seq<Seq<u8>>) -> bool {
    &&& write_order.is_encodable()
    &&& record_width > 0
    &&& records.len() <= u16::MAX
    &&& records.len() * record_width <= specification_packed_unit_record_area_bytes()
    &&& specification_every_piece_has_width(records, record_width as nat)
}

/// 码 3 打包记录单元 32768：类身份段 107 + 预留位 29，记录区从 136 起；载荷 CRC 罩 [107, 32768)。
///
/// # Panics
/// 记录数装不进 2 字节、记录区装不进一个单元、或有一条记录的宽与 `record_width` 不同（`requires` 排除的三种：写者先按容量切好记录再调这里）；
/// 写序的事务号装不进 6 字节（同上）。
///
/// `requires` 另要记录宽不是 0（[`specification_packed_unit_writer_accepts`]）：只约束经 Verus 验过的调用方，函数体不判它、不为它 panic；
/// 今天的调用方都在 `verus!` 之外，传的都是登记表里非 0 的记录宽。
#[must_use]
pub fn build_packed_unit(
    identity: PackedIdentity,
    record_width: u16,
    records: &[Vec<u8>],
    birth_txg: CheckpointTxg,
    filesystem_identifier: &[u8; 16],
    write_order: WriteOrder,
    birth_sequence: BirthSequence,
) -> (unit: Vec<u8>)
    requires
        specification_packed_unit_writer_accepts(write_order, record_width, specification_byte_sequences_of_vectors(records@)),
    ensures
        unit@ == specification_packed_unit_bytes(
            identity,
            record_width,
            specification_byte_sequences_of_vectors(records@),
            birth_txg,
            *filesystem_identifier,
            write_order,
            birth_sequence,
        ),
{
    let ghost record_pieces = specification_byte_sequences_of_vectors(records@);
    let header_end = usize::try_from(PACKED_UNIT_HEADER_BYTES).expect("107");
    let declared_length =
        u16::try_from(records.len() * usize::from(record_width)).expect("声明长度 2 字节");
    let mut writer = ByteWriter::new(usize::try_from(DATA_UNIT_BYTES).expect("32768"));
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
    writer.put_u64(unit_filesystem_identifier(filesystem_identifier));
    writer.assert_position(89, "码 3 载荷校验和");
    let payload_crc_offset = writer.position();
    writer.skip(4);
    writer.assert_position(93, "码 3 写序");
    writer.put_u32(write_order.instance.0);
    writer.put_six_byte_unsigned(write_order.transaction);
    writer.assert_position(103, "码 3 出生序号");
    writer.put_u32(birth_sequence.0);
    writer.assert_position(PACKED_UNIT_HEADER_BYTES, "码 3 头");
    writer.skip(reserved_bytes());
    let ghost header = writer.written();
    assert(header == specification_packed_unit_unsealed_header(
        identity,
        records@.len(),
        record_width,
        birth_txg,
        *filesystem_identifier,
        write_order,
        birth_sequence,
    ));
    // for 循环换成下标循环（Verus 对切片迭代器的规格不全），逐条核记录宽、逐条写，次序不变。
    let mut record_index: usize = 0;
    while record_index < records.len()
        invariant
            writer.is_well_formed(),
            writer.total_bytes() == DATA_UNIT_BYTES,
            header.len() == 136,
            record_pieces == specification_byte_sequences_of_vectors(records@),
            record_index <= records@.len(),
            records@.len() * record_width <= specification_packed_unit_record_area_bytes(),
            specification_every_piece_has_width(record_pieces, record_width as nat),
            writer.written() == header + specification_concatenation(record_pieces.subrange(0, record_index as int)),
        decreases records@.len() - record_index,
    {
        let record = &records[record_index];
        proof {
            assert(record_pieces[record_index as int] == record@);
            assert(record_pieces[record_index as int].len() == record_width);
        }
        // assert_eq! 展开成的 core::panicking::assert_failed 没有 Verus 规格，写成 assert! 加同一句话。
        assert!(
            record.len() == usize::from(record_width),
            "记录宽与头里写的不一致"
        );
        proof {
            let written_pieces = record_pieces.subrange(0, record_index as int);
            assert(specification_every_piece_has_width(written_pieces, record_width as nat));
            lemma_concatenation_of_pieces_of_one_width(written_pieces, record_width as nat);
            assert(record_index * record_width + record_width <= records@.len() * record_width) by (nonlinear_arith)
                requires
                    record_index < records@.len(),
            ;
            assert(record_pieces[record_index as int] == record@);
        }
        writer.put(record.as_slice());
        proof {
            let longer = record_pieces.subrange(0, record_index + 1);
            assert(longer.drop_last() =~= record_pieces.subrange(0, record_index as int));
            assert(longer.last() == record@);
        }
        record_index += 1;
    }
    proof {
        assert(record_pieces.subrange(0, records@.len() as int) =~= record_pieces);
        lemma_concatenation_of_pieces_of_one_width(record_pieces, record_width as nat);
    }
    let mut bytes = writer.into_bytes();
    assert(bytes@ =~= specification_packed_unit_unsealed_bytes(
        identity,
        record_width,
        record_pieces,
        birth_txg,
        *filesystem_identifier,
        write_order,
        birth_sequence,
    ));
    seal_payload_checksum(bytes.as_mut_slice(), header_end, payload_crc_offset);
    seal_header_checksum(bytes.as_mut_slice(), header_end);
    bytes
}

/// 码 3 打包记录单元解出来的头与记录。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackedUnitHeader {
    pub identity: PackedIdentity,
    pub birth_txg: CheckpointTxg,
    pub filesystem_identifier: u64,
    pub write_order: WriteOrder,
    pub birth_sequence: BirthSequence,
    pub record_width: usize,
    pub records: Vec<Vec<u8>>,
}

/// [`PackedUnitHeader`] 在规格里的样子：`Vec` 换成它们的字节。
pub struct PackedUnitHeaderModel {
    pub identity: PackedIdentity,
    pub birth_txg: CheckpointTxg,
    pub filesystem_identifier: u64,
    pub write_order: WriteOrder,
    pub birth_sequence: BirthSequence,
    pub record_width: nat,
    pub records: Seq<Seq<u8>>,
}

impl PackedUnitHeader {
    pub open spec fn specification_model(&self) -> PackedUnitHeaderModel {
        PackedUnitHeaderModel {
            identity: self.identity,
            birth_txg: self.birth_txg,
            filesystem_identifier: self.filesystem_identifier,
            write_order: self.write_order,
            birth_sequence: self.birth_sequence,
            record_width: self.record_width as nat,
            records: specification_byte_sequences_of_vectors(self.records@),
        }
    }
}

/// [`parse_packed_unit`] 的判法（交回的是 [`PackedUnitHeaderModel`]）。
pub open spec fn specification_parse_packed_unit(bytes: Seq<u8>) -> Result<PackedUnitHeaderModel, UnitError> {
    if bytes.len() != DATA_UNIT_BYTES {
        Err(UnitError::TooShort)
    } else {
        match specification_check_common_prefix(bytes, UNIT_CLASS_PACKED, PACKED_UNIT_HEADER_BYTES as int) {
            Err(error) => Err(error),
            Ok(declared_length) => {
                let record_count = little_endian_value(bytes.subrange(69, 71));
                let record_width = little_endian_value(bytes.subrange(71, 73));
                let records_start = PACKED_UNIT_HEADER_BYTES + NONCE_MAC_ALGORITHM_RESERVED_BYTES;
                if !specification_payload_checksum_holds(bytes, PACKED_UNIT_HEADER_BYTES as int, 89) {
                    Err(UnitError::PayloadChecksumMismatch)
                } else if !specification_encryption_reserved_bytes_are_zero(bytes, PACKED_UNIT_HEADER_BYTES as int) {
                    Err(UnitError::EncryptionReservedBytesNotZero)
                } else if little_endian_value(bytes.subrange(42, 43)) != UNIT_CLASS_PACKED {
                    Err(UnitError::Structure(UnitStructureDefect::PackedUnitClassTagCopyIsNotThePackedClass))
                } else if record_count * record_width != declared_length {
                    Err(UnitError::Structure(UnitStructureDefect::PackedUnitDeclaredLengthIsNotRecordCountTimesRecordWidth))
                } else if record_width == 0 && record_count != 0 {
                    Err(UnitError::Structure(UnitStructureDefect::PackedUnitRecordWidthZeroWithRecords))
                } else if records_start + declared_length > bytes.len() {
                    Err(UnitError::Structure(UnitStructureDefect::PackedUnitRecordAreaPastTheUnitEnd))
                } else {
                    Ok(
                        PackedUnitHeaderModel {
                            identity: PackedIdentity::specification_decode_at(bytes, 43),
                            birth_txg: CheckpointTxg(little_endian_value(bytes.subrange(73, 81)) as u64),
                            filesystem_identifier: little_endian_value(bytes.subrange(81, 89)) as u64,
                            write_order: WriteOrder::specification_decode_at(bytes, 93),
                            birth_sequence: BirthSequence(little_endian_value(bytes.subrange(103, 107)) as u32),
                            record_width,
                            records: specification_pieces_at(bytes, records_start, record_count, record_width),
                        },
                    )
                }
            },
        }
    }
}

pub fn parse_packed_unit(bytes: &[u8]) -> (parsed: Result<PackedUnitHeader, UnitError>)
    ensures
        match parsed {
            Ok(header) => specification_parse_packed_unit(bytes@) == Ok::<PackedUnitHeaderModel, UnitError>(header.specification_model()),
            Err(error) => specification_parse_packed_unit(bytes@) == Err::<PackedUnitHeaderModel, UnitError>(error),
        },
{
    if bytes.len() != usize::try_from(DATA_UNIT_BYTES).expect("32768") {
        return Err(UnitError::TooShort);
    }
    let header_end = usize::try_from(PACKED_UNIT_HEADER_BYTES).expect("107");
    let declared_length = usize::from(check_common_prefix(bytes, UNIT_CLASS_PACKED, header_end)?);
    check_payload_checksum(bytes, header_end, 89)?;
    check_encryption_reserved_bytes_are_zero(bytes, header_end)?;
    let mut reader = ByteReader::at(bytes, 42);
    if reader.get_u8() != UNIT_CLASS_PACKED {
        return Err(UnitError::Structure(
            UnitStructureDefect::PackedUnitClassTagCopyIsNotThePackedClass,
        ));
    }
    let identity = PackedIdentity {
        birth_tree: TreeIdentifier(reader.get_u64()),
        record_type: reader.get_u16(),
        container: reader.get_u64(),
        container_birth: CheckpointTxg(reader.get_u64()),
    };
    let record_count = usize::from(reader.get_u16());
    let record_width = usize::from(reader.get_u16());
    let birth_txg = CheckpointTxg(reader.get_u64());
    let filesystem_identifier = reader.get_u64();
    reader.skip(4);
    let write_order = WriteOrder {
        instance: InstanceGeneration(reader.get_u32()),
        transaction: reader.get_six_byte_unsigned(),
    };
    let birth_sequence = BirthSequence(reader.get_u32());
    // assert_eq! 展开成的 core::panicking::assert_failed 没有 Verus 规格，写成 assert! 加一句话。
    assert!(reader.position() == header_end, "码 3 头读到偏移 107");
    proof {
        assert(record_count * record_width <= 0xFFFF * 0xFFFF) by (nonlinear_arith)
            requires
                record_count <= 0xFFFF,
                record_width <= 0xFFFF,
        ;
    }
    if record_count * record_width != declared_length {
        return Err(UnitError::Structure(
            UnitStructureDefect::PackedUnitDeclaredLengthIsNotRecordCountTimesRecordWidth,
        ));
    }
    // 记录宽 0 而记录数不为 0：与码 2 那一格同一个坏法（代码审阅第 38 条）。
    if record_width == 0 && record_count != 0 {
        return Err(UnitError::Structure(
            UnitStructureDefect::PackedUnitRecordWidthZeroWithRecords,
        ));
    }
    let records_start = header_end + reserved_bytes();
    if records_start + declared_length > bytes.len() {
        return Err(UnitError::Structure(
            UnitStructureDefect::PackedUnitRecordAreaPastTheUnitEnd,
        ));
    }
    let records = pieces_of_one_width(bytes, records_start, record_count, record_width);
    Ok(PackedUnitHeader {
        identity,
        birth_txg,
        filesystem_identifier,
        write_order,
        birth_sequence,
        record_width,
        records,
    })
}

/// 码 3 打包记录单元往返：[`build_packed_unit`] 写出的单元，[`parse_packed_unit`] 读回同一份头与同样的记录。
/// 前提就是写者的前提：读者拒「记录宽 0 而记录数不为 0」，写者的前提里记录宽不是 0，所以往返不另加。
pub proof fn lemma_packed_unit_round_trip(
    identity: PackedIdentity,
    record_width: u16,
    records: Seq<Seq<u8>>,
    birth_txg: CheckpointTxg,
    filesystem_identifier: [u8; 16],
    write_order: WriteOrder,
    birth_sequence: BirthSequence,
)
    requires
        specification_packed_unit_writer_accepts(write_order, record_width, records),
    ensures
        specification_parse_packed_unit(
            specification_packed_unit_bytes(
                identity,
                record_width,
                records,
                birth_txg,
                filesystem_identifier,
                write_order,
                birth_sequence,
            ),
        ) == Ok::<PackedUnitHeaderModel, UnitError>(
            PackedUnitHeaderModel {
                identity,
                birth_txg,
                filesystem_identifier: specification_unit_filesystem_identifier(filesystem_identifier),
                write_order,
                birth_sequence,
                record_width: record_width as nat,
                records,
            },
        ),
{
    lemma_little_endian_capacities();
    lemma_concatenation_of_pieces_of_one_width(records, record_width as nat);
    let record_count = records.len();
    let declared_length = (record_count * record_width) as u16;
    let header = specification_packed_unit_unsealed_header(
        identity,
        record_count,
        record_width,
        birth_txg,
        filesystem_identifier,
        write_order,
        birth_sequence,
    );
    let record_area = specification_concatenation(records);
    let unsealed = specification_packed_unit_unsealed_bytes(
        identity,
        record_width,
        records,
        birth_txg,
        filesystem_identifier,
        write_order,
        birth_sequence,
    );
    let padding = zero_bytes((DATA_UNIT_BYTES - header.len() - record_area.len()) as nat);
    assert(header.len() == 136);
    assert(unsealed.len() == 32768);
    lemma_sealed_unit_keeps_everything_outside_the_two_checksums(unsealed, 107, 89);
    let unit = specification_packed_unit_bytes(
        identity,
        record_width,
        records,
        birth_txg,
        filesystem_identifier,
        write_order,
        birth_sequence,
    );
    assert(unsealed.subrange(0, unsealed.len() as int) =~= unsealed);
    lemma_split_a_concatenated_range(unsealed, 0, header + record_area, padding);
    lemma_split_a_concatenated_range(unsealed, 0, header, record_area);
    let prefix = specification_common_prefix_bytes(UNIT_CLASS_PACKED, declared_length);
    let class = little_endian_bytes(UNIT_CLASS_PACKED as nat, 1);
    let birth_tree = little_endian_bytes(identity.birth_tree.0 as nat, 8);
    let record_type = little_endian_bytes(identity.record_type as nat, 2);
    let container = little_endian_bytes(identity.container as nat, 8);
    let container_birth = little_endian_bytes(identity.container_birth.0 as nat, 8);
    let count = little_endian_bytes(record_count, 2);
    let width = little_endian_bytes(record_width as nat, 2);
    let birth = little_endian_bytes(birth_txg.0 as nat, 8);
    let fsid = little_endian_bytes(specification_unit_filesystem_identifier(filesystem_identifier) as nat, 8);
    let crc_field = zero_bytes(4);
    let instance = little_endian_bytes(write_order.instance.0 as nat, 4);
    let transaction = little_endian_bytes(write_order.transaction as nat, 6);
    let sequence = little_endian_bytes(birth_sequence.0 as nat, 4);
    let reserved = zero_bytes(29);
    let through_count = prefix + class + birth_tree + record_type + container + container_birth + count;
    let through_fsid = through_count + width + birth + fsid;
    lemma_split_a_concatenated_range(unsealed, 0, through_fsid + crc_field + instance + transaction + sequence, reserved);
    lemma_split_a_concatenated_range(unsealed, 0, through_fsid + crc_field + instance + transaction, sequence);
    lemma_split_a_concatenated_range(unsealed, 0, through_fsid + crc_field + instance, transaction);
    lemma_split_a_concatenated_range(unsealed, 0, through_fsid + crc_field, instance);
    lemma_split_a_concatenated_range(unsealed, 0, through_fsid, crc_field);
    lemma_split_a_concatenated_range(unsealed, 0, through_count + width + birth, fsid);
    lemma_split_a_concatenated_range(unsealed, 0, through_count + width, birth);
    lemma_split_a_concatenated_range(unsealed, 0, through_count, width);
    lemma_split_a_concatenated_range(unsealed, 0, prefix + class + birth_tree + record_type + container + container_birth, count);
    lemma_split_a_concatenated_range(unsealed, 0, prefix + class + birth_tree + record_type + container, container_birth);
    lemma_split_a_concatenated_range(unsealed, 0, prefix + class + birth_tree + record_type, container);
    lemma_split_a_concatenated_range(unsealed, 0, prefix + class + birth_tree, record_type);
    lemma_split_a_concatenated_range(unsealed, 0, prefix + class, birth_tree);
    lemma_split_a_concatenated_range(unsealed, 0, prefix, class);
    assert(unsealed.subrange(0, 10) =~= unsealed.subrange(0, 42).subrange(0, 10));
    assert(unit.subrange(0, 10) == unsealed.subrange(0, 10));
    lemma_common_prefix_reads_back(unit, UNIT_CLASS_PACKED, declared_length);
    assert(unit.subrange(107, 136) == unsealed.subrange(107, 136));
    assert(unit.subrange(42, 43) == unsealed.subrange(42, 43));
    assert(unit.subrange(43, 51) == unsealed.subrange(43, 51));
    assert(unit.subrange(51, 53) == unsealed.subrange(51, 53));
    assert(unit.subrange(53, 61) == unsealed.subrange(53, 61));
    assert(unit.subrange(61, 69) == unsealed.subrange(61, 69));
    assert(unit.subrange(69, 71) == unsealed.subrange(69, 71));
    assert(unit.subrange(71, 73) == unsealed.subrange(71, 73));
    assert(unit.subrange(73, 81) == unsealed.subrange(73, 81));
    assert(unit.subrange(81, 89) == unsealed.subrange(81, 89));
    assert(unit.subrange(93, 97) == unsealed.subrange(93, 97));
    assert(unit.subrange(97, 103) == unsealed.subrange(97, 103));
    assert(unit.subrange(103, 107) == unsealed.subrange(103, 107));
    assert(unit.subrange(136, 136 + record_count * record_width) == unsealed.subrange(
        136,
        136 + record_count * record_width,
    ));
    lemma_pieces_at_a_concatenation(unit, 136, records, record_width as nat);
}

} // verus!

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checksum::wide_checksum_field_holds;

    #[test]
    fn packed_unit_places_records_at_136_and_seals_both_checksums() {
        let identity = PackedIdentity {
            birth_tree: TreeIdentifier(0),
            record_type: PACKED_TYPE_INSTANCE_TABLE,
            container: 0,
            container_birth: CheckpointTxg(0),
        };
        let record = vec![0x5Au8; 88];
        let unit = build_packed_unit(
            identity,
            88,
            std::slice::from_ref(&record),
            CheckpointTxg(0),
            &[1u8; 16],
            WriteOrder {
                instance: InstanceGeneration(0),
                transaction: 0,
            },
            BirthSequence(0),
        );
        assert_eq!(unit.len(), 32768);
        assert_eq!(&unit[136..136 + 88], &record[..]);
        assert_eq!(u16::from_le_bytes([unit[69], unit[70]]), 1, "记录数");
        assert_eq!(u16::from_le_bytes([unit[71], unit[72]]), 88, "记录宽");
        assert!(wide_checksum_field_holds(
            &unit,
            107,
            HEADER_CHECKSUM_OFFSET
        ));
        assert_eq!(
            u32::from_le_bytes(unit[89..93].try_into().expect("4")),
            crc32_castagnoli(&unit[107..])
        );
    }
}
