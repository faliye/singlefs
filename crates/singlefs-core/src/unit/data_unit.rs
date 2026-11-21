//! 数据单元（单元类 1）：写法与读法的规格、`build_data_unit`、`parse_data_unit`、`data_unit_payload` 与往返引理。

use super::*;

verus! {

broadcast use group_little_endian_round_trips;

// ===== 码 1 数据单元 =====

/// 码 1 数据单元的五元组身份（D18（块里携带什么信息） 已定项 3：类标签副本 1 + 树 ID 8 + 对象 ID 8 + 对象出生代 8 + 锚点偏移 8）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataUnitIdentity {
    pub tree: TreeIdentifier,
    pub object: u64,
    pub object_birth: CheckpointTxg,
    pub anchor_offset: u64,
}

/// 一个数据单元装得下多少字节载荷：32768 − 头 105 − 预留 29 = 32634。写者在装单元之前拿它判「装不下」并报错，不走到断言。
#[must_use]
pub fn data_unit_payload_capacity() -> (capacity: usize)
    ensures
        capacity == DATA_UNIT_BYTES - DATA_UNIT_HEADER_BYTES - NONCE_MAC_ALGORITHM_RESERVED_BYTES,
{
    usize::try_from(DATA_UNIT_BYTES).expect("32768")
        - usize::try_from(DATA_UNIT_HEADER_BYTES).expect("105")
        - reserved_bytes()
}

/// 码 1 单元没封的头连同预留位，`[0, 134)`，照写者的次序逐段接：共同前缀 42、类标签副本 1、树 ID 8、对象 ID 8、对象出生代 8、锚点 8、
/// 诞生代号 8、fsid 8、写序 10（实例代号 4 + 事务号 6）、载荷 CRC 4（写的时候是 0）、预留位 29。
pub open spec fn specification_data_unit_unsealed_header(
    identity: DataUnitIdentity,
    birth_txg: CheckpointTxg,
    filesystem_identifier: [u8; 16],
    write_order: WriteOrder,
    payload_length: nat,
) -> Seq<u8> {
    specification_common_prefix_bytes(UNIT_CLASS_DATA, payload_length as u16) + little_endian_bytes(UNIT_CLASS_DATA as nat, 1)
        + little_endian_bytes(identity.tree.0 as nat, 8) + little_endian_bytes(identity.object as nat, 8)
        + little_endian_bytes(identity.object_birth.0 as nat, 8) + little_endian_bytes(identity.anchor_offset as nat, 8)
        + little_endian_bytes(birth_txg.0 as nat, 8) + little_endian_bytes(
        specification_unit_filesystem_identifier(filesystem_identifier) as nat,
        8,
    ) + little_endian_bytes(write_order.instance.0 as nat, 4) + little_endian_bytes(write_order.transaction as nat, 6)
        + zero_bytes(4) + zero_bytes(NONCE_MAC_ALGORITHM_RESERVED_BYTES as nat)
}

/// 码 1 单元没封的那一份：共同前缀、类身份段、载荷从 134 起、补 0 到 32768。
pub open spec fn specification_data_unit_unsealed_bytes(
    identity: DataUnitIdentity,
    birth_txg: CheckpointTxg,
    filesystem_identifier: [u8; 16],
    write_order: WriteOrder,
    payload: Seq<u8>,
) -> Seq<u8> {
    let header = specification_data_unit_unsealed_header(identity, birth_txg, filesystem_identifier, write_order, payload.len());
    header + payload + zero_bytes((DATA_UNIT_BYTES - header.len() - payload.len()) as nat)
}

/// [`build_data_unit`] 写出的整个码 1 单元：载荷 CRC 罩 [105, 32768)、写在 101；头校验和罩 [0, 105)。
pub open spec fn specification_data_unit_bytes(
    identity: DataUnitIdentity,
    birth_txg: CheckpointTxg,
    filesystem_identifier: [u8; 16],
    write_order: WriteOrder,
    payload: Seq<u8>,
) -> Seq<u8> {
    specification_sealed_unit(
        specification_data_unit_unsealed_bytes(identity, birth_txg, filesystem_identifier, write_order, payload),
        DATA_UNIT_HEADER_BYTES as int,
        101,
    )
}

/// 码 1 数据单元 32768：类身份段 105（偏移 42 标签副本 / 43 树 ID / 51 对象 ID / 59 对象出生代 / 67 锚点 / 75 诞生代号 / 83 fsid / 91 写序 / 101 载荷 CRC）
/// + 预留位 29，载荷从 134 起、声明长度之后补 0；载荷 CRC 罩 [105, 32768)，头校验和罩 [0, 105)。
///
/// # Panics
/// 载荷装不进一个单元，或写序的事务号装不进 6 字节（`requires` 排除的两种：写者先按 [`data_unit_payload_capacity`] 判「装不下」并报错）。
#[must_use]
pub fn build_data_unit(
    identity: DataUnitIdentity,
    birth_txg: CheckpointTxg,
    filesystem_identifier: &[u8; 16],
    write_order: WriteOrder,
    payload: &[u8],
) -> (unit: Vec<u8>)
    requires
        payload@.len() <= DATA_UNIT_BYTES - DATA_UNIT_HEADER_BYTES - NONCE_MAC_ALGORITHM_RESERVED_BYTES,
        write_order.is_encodable(),
    ensures
        unit@ == specification_data_unit_bytes(identity, birth_txg, *filesystem_identifier, write_order, payload@),
{
    let header_end = usize::try_from(DATA_UNIT_HEADER_BYTES).expect("105");
    let unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    assert!(
        payload.len() <= data_unit_payload_capacity(),
        "载荷装不进一个单元：写者要先按 data_unit_payload_capacity 判、报错，不许走到这里"
    );
    let mut writer = ByteWriter::new(unit_bytes);
    write_common_prefix(
        &mut writer,
        UNIT_CLASS_DATA,
        u16::try_from(payload.len()).expect("声明长度 2 字节"),
    );
    writer.put_u8(UNIT_CLASS_DATA);
    writer.put_u64(identity.tree.0);
    writer.put_u64(identity.object);
    writer.put_u64(identity.object_birth.0);
    writer.put_u64(identity.anchor_offset);
    writer.assert_position(75, "数据单元诞生代号");
    writer.put_u64(birth_txg.0);
    writer.put_u64(unit_filesystem_identifier(filesystem_identifier));
    writer.assert_position(91, "数据单元写序");
    writer.put_u32(write_order.instance.0);
    writer.put_six_byte_unsigned(write_order.transaction);
    writer.assert_position(101, "数据单元载荷 CRC");
    let payload_crc_offset = writer.position();
    writer.skip(4);
    writer.assert_position(DATA_UNIT_HEADER_BYTES, "数据单元头");
    writer.skip(reserved_bytes());
    writer.put(payload);
    let mut bytes = writer.into_bytes();
    assert(bytes@ =~= specification_data_unit_unsealed_bytes(identity, birth_txg, *filesystem_identifier, write_order, payload@));
    seal_payload_checksum(bytes.as_mut_slice(), header_end, payload_crc_offset);
    seal_header_checksum(bytes.as_mut_slice(), header_end);
    bytes
}

/// 码 1 数据单元解出来的头。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataUnitHeader {
    pub identity: DataUnitIdentity,
    pub birth_txg: CheckpointTxg,
    pub filesystem_identifier: u64,
    pub write_order: WriteOrder,
    pub declared_length: u16,
}

/// [`parse_data_unit`] 的判法。
pub open spec fn specification_parse_data_unit(bytes: Seq<u8>) -> Result<DataUnitHeader, UnitError> {
    if bytes.len() != DATA_UNIT_BYTES {
        Err(UnitError::TooShort)
    } else {
        match specification_check_common_prefix(bytes, UNIT_CLASS_DATA, DATA_UNIT_HEADER_BYTES as int) {
            Err(error) => Err(error),
            Ok(declared_length) => {
                if !specification_payload_checksum_holds(bytes, DATA_UNIT_HEADER_BYTES as int, 101) {
                    Err(UnitError::PayloadChecksumMismatch)
                } else if !specification_encryption_reserved_bytes_are_zero(bytes, DATA_UNIT_HEADER_BYTES as int) {
                    Err(UnitError::EncryptionReservedBytesNotZero)
                } else if little_endian_value(bytes.subrange(42, 43)) != UNIT_CLASS_DATA {
                    Err(UnitError::Structure(UnitStructureDefect::DataUnitClassTagCopyIsNotTheDataClass))
                } else if DATA_UNIT_HEADER_BYTES + NONCE_MAC_ALGORITHM_RESERVED_BYTES + declared_length > bytes.len() {
                    Err(UnitError::Structure(UnitStructureDefect::DataUnitDeclaredLengthPastTheUnitEnd))
                } else {
                    Ok(
                        DataUnitHeader {
                            identity: DataUnitIdentity {
                                tree: TreeIdentifier(little_endian_value(bytes.subrange(43, 51)) as u64),
                                object: little_endian_value(bytes.subrange(51, 59)) as u64,
                                object_birth: CheckpointTxg(little_endian_value(bytes.subrange(59, 67)) as u64),
                                anchor_offset: little_endian_value(bytes.subrange(67, 75)) as u64,
                            },
                            birth_txg: CheckpointTxg(little_endian_value(bytes.subrange(75, 83)) as u64),
                            filesystem_identifier: little_endian_value(bytes.subrange(83, 91)) as u64,
                            write_order: WriteOrder::specification_decode_at(bytes, 91),
                            declared_length,
                        },
                    )
                }
            },
        }
    }
}

pub fn parse_data_unit(bytes: &[u8]) -> (parsed: Result<DataUnitHeader, UnitError>)
    ensures
        parsed == specification_parse_data_unit(bytes@),
{
    if bytes.len() != usize::try_from(DATA_UNIT_BYTES).expect("32768") {
        return Err(UnitError::TooShort);
    }
    let header_end = usize::try_from(DATA_UNIT_HEADER_BYTES).expect("105");
    let declared_length = check_common_prefix(bytes, UNIT_CLASS_DATA, header_end)?;
    check_payload_checksum(bytes, header_end, 101)?;
    check_encryption_reserved_bytes_are_zero(bytes, header_end)?;
    let mut reader = ByteReader::at(bytes, 42);
    if reader.get_u8() != UNIT_CLASS_DATA {
        return Err(UnitError::Structure(
            UnitStructureDefect::DataUnitClassTagCopyIsNotTheDataClass,
        ));
    }
    let identity = DataUnitIdentity {
        tree: TreeIdentifier(reader.get_u64()),
        object: reader.get_u64(),
        object_birth: CheckpointTxg(reader.get_u64()),
        anchor_offset: reader.get_u64(),
    };
    let birth_txg = CheckpointTxg(reader.get_u64());
    let filesystem_identifier = reader.get_u64();
    let write_order = WriteOrder {
        instance: InstanceGeneration(reader.get_u32()),
        transaction: reader.get_six_byte_unsigned(),
    };
    let payload_start = header_end + reserved_bytes();
    if payload_start + usize::from(declared_length) > bytes.len() {
        return Err(UnitError::Structure(
            UnitStructureDefect::DataUnitDeclaredLengthPastTheUnitEnd,
        ));
    }
    Ok(DataUnitHeader {
        identity,
        birth_txg,
        filesystem_identifier,
        write_order,
        declared_length,
    })
}

/// [`data_unit_payload`] 的判法：载荷末尾越界、补齐有非 0 的报结构错，否则交回 `[134, 134 + 声明长度)`。
pub open spec fn specification_data_unit_payload(bytes: Seq<u8>, declared_length: u16) -> Result<Seq<u8>, UnitError> {
    let payload_start = DATA_UNIT_HEADER_BYTES + NONCE_MAC_ALGORITHM_RESERVED_BYTES;
    let payload_end = payload_start + declared_length;
    if payload_end > bytes.len() {
        Err(UnitError::Structure(UnitStructureDefect::DataUnitDeclaredLengthPastTheUnitEnd))
    } else if bytes.subrange(payload_end, bytes.len() as int) != zero_bytes((bytes.len() - payload_end) as nat) {
        Err(UnitError::Structure(UnitStructureDefect::DataUnitPaddingBytesNotZero))
    } else {
        Ok(bytes.subrange(payload_start, payload_end))
    }
}

/// 载荷从 134 起、长声明长度；之后到单元末尾必须全 0（I-2.3），不是就报结构错。
///
/// # Errors
/// 载荷末尾（134 + 声明长度）越过 `bytes` 末尾 ⇒ [`UnitError::Structure`]（[`UnitStructureDefect::DataUnitDeclaredLengthPastTheUnitEnd`]）：声明长度是盘上读来的，
/// 两字节最大 65535、比一个单元的载荷容量大，这里自己判、不越界切片——今天的调用方都先经 [`parse_data_unit`] 核过这一格，
/// 这一判让本函数不靠调用方的那一步也不 panic（坏的是盘上数据，按「能恢复的写进类型」报错，不断言）。
/// 补齐字节非零 ⇒ [`UnitError::Structure`]（[`UnitStructureDefect::DataUnitPaddingBytesNotZero`]）。
pub fn data_unit_payload(bytes: &[u8], declared_length: u16) -> (payload: Result<&[u8], UnitError>)
    ensures
        match payload {
            Ok(payload_bytes) => specification_data_unit_payload(bytes@, declared_length) == Ok::<Seq<u8>, UnitError>(
                payload_bytes@,
            ),
            Err(error) => specification_data_unit_payload(bytes@, declared_length) == Err::<Seq<u8>, UnitError>(error),
        },
{
    let payload_start = usize::try_from(DATA_UNIT_HEADER_BYTES).expect("105") + reserved_bytes();
    let payload_end = payload_start + usize::from(declared_length);
    if payload_end > bytes.len() {
        return Err(UnitError::Structure(
            UnitStructureDefect::DataUnitDeclaredLengthPastTheUnitEnd,
        ));
    }
    // .iter().any(…) 带闭包没有 Verus 规格：换成 every_byte_is_zero（逐字节的循环），同一个判法。
    if !every_byte_is_zero(&bytes[payload_end..]) {
        return Err(UnitError::Structure(
            UnitStructureDefect::DataUnitPaddingBytesNotZero,
        ));
    }
    Ok(&bytes[payload_start..payload_end])
}

/// 码 1 数据单元往返：[`build_data_unit`] 写出的单元，[`parse_data_unit`] 读回同一份头，[`data_unit_payload`] 读回同一段载荷。
pub proof fn lemma_data_unit_round_trip(
    identity: DataUnitIdentity,
    birth_txg: CheckpointTxg,
    filesystem_identifier: [u8; 16],
    write_order: WriteOrder,
    payload: Seq<u8>,
)
    requires
        payload.len() <= DATA_UNIT_BYTES - DATA_UNIT_HEADER_BYTES - NONCE_MAC_ALGORITHM_RESERVED_BYTES,
        write_order.is_encodable(),
    ensures
        ({
            let unit = specification_data_unit_bytes(identity, birth_txg, filesystem_identifier, write_order, payload);
            &&& specification_parse_data_unit(unit) == Ok::<DataUnitHeader, UnitError>(
                DataUnitHeader {
                    identity,
                    birth_txg,
                    filesystem_identifier: specification_unit_filesystem_identifier(filesystem_identifier),
                    write_order,
                    declared_length: payload.len() as u16,
                },
            )
            &&& specification_data_unit_payload(unit, payload.len() as u16) == Ok::<Seq<u8>, UnitError>(payload)
        }),
{
    lemma_little_endian_capacities();
    let declared_length = payload.len() as u16;
    let header = specification_data_unit_unsealed_header(identity, birth_txg, filesystem_identifier, write_order, payload.len());
    let unsealed = specification_data_unit_unsealed_bytes(identity, birth_txg, filesystem_identifier, write_order, payload);
    let padding = zero_bytes((DATA_UNIT_BYTES - header.len() - payload.len()) as nat);
    assert(header.len() == 134);
    assert(unsealed.len() == 32768);
    lemma_sealed_unit_keeps_everything_outside_the_two_checksums(unsealed, 105, 101);
    let unit = specification_data_unit_bytes(identity, birth_txg, filesystem_identifier, write_order, payload);
    // 没封的那一份逐段剥开：头、载荷、补齐，再把头照写者的次序从末尾往前剥。
    assert(unsealed.subrange(0, unsealed.len() as int) =~= unsealed);
    lemma_split_a_concatenated_range(unsealed, 0, header + payload, padding);
    lemma_split_a_concatenated_range(unsealed, 0, header, payload);
    let prefix = specification_common_prefix_bytes(UNIT_CLASS_DATA, declared_length);
    let class = little_endian_bytes(UNIT_CLASS_DATA as nat, 1);
    let tree = little_endian_bytes(identity.tree.0 as nat, 8);
    let object = little_endian_bytes(identity.object as nat, 8);
    let object_birth = little_endian_bytes(identity.object_birth.0 as nat, 8);
    let anchor = little_endian_bytes(identity.anchor_offset as nat, 8);
    let birth = little_endian_bytes(birth_txg.0 as nat, 8);
    let fsid = little_endian_bytes(specification_unit_filesystem_identifier(filesystem_identifier) as nat, 8);
    let instance = little_endian_bytes(write_order.instance.0 as nat, 4);
    let transaction = little_endian_bytes(write_order.transaction as nat, 6);
    let crc_field = zero_bytes(4);
    let reserved = zero_bytes(29);
    let through_transaction = prefix + class + tree + object + object_birth + anchor + birth + fsid + instance + transaction;
    lemma_split_a_concatenated_range(unsealed, 0, through_transaction + crc_field, reserved);
    lemma_split_a_concatenated_range(unsealed, 0, through_transaction, crc_field);
    lemma_split_a_concatenated_range(unsealed, 0, prefix + class + tree + object + object_birth + anchor + birth + fsid + instance, transaction);
    lemma_split_a_concatenated_range(unsealed, 0, prefix + class + tree + object + object_birth + anchor + birth + fsid, instance);
    lemma_split_a_concatenated_range(unsealed, 0, prefix + class + tree + object + object_birth + anchor + birth, fsid);
    lemma_split_a_concatenated_range(unsealed, 0, prefix + class + tree + object + object_birth + anchor, birth);
    lemma_split_a_concatenated_range(unsealed, 0, prefix + class + tree + object + object_birth, anchor);
    lemma_split_a_concatenated_range(unsealed, 0, prefix + class + tree + object, object_birth);
    lemma_split_a_concatenated_range(unsealed, 0, prefix + class + tree, object);
    lemma_split_a_concatenated_range(unsealed, 0, prefix + class, tree);
    lemma_split_a_concatenated_range(unsealed, 0, prefix, class);
    // 前 10 字节（头校验和之前那一截）在封好的单元里没变。
    assert(unsealed.subrange(0, 10) =~= unsealed.subrange(0, 42).subrange(0, 10));
    assert(unit.subrange(0, 10) == unsealed.subrange(0, 10));
    lemma_common_prefix_reads_back(unit, UNIT_CLASS_DATA, declared_length);
    // 预留位与补齐在封好的单元里还是 0。
    assert(unit.subrange(105, 134) == unsealed.subrange(105, 134));
    assert(unit.subrange(134 + payload.len() as int, 32768) == unsealed.subrange(134 + payload.len() as int, 32768));
    assert(unit.subrange(134, 134 + payload.len() as int) == unsealed.subrange(134, 134 + payload.len() as int));
    // 类标签副本那一字节。
    assert(unit.subrange(42, 43) == unsealed.subrange(42, 43));
    // 身份、诞生代号、fsid、写序各段。
    assert(unit.subrange(43, 51) == unsealed.subrange(43, 51));
    assert(unit.subrange(51, 59) == unsealed.subrange(51, 59));
    assert(unit.subrange(59, 67) == unsealed.subrange(59, 67));
    assert(unit.subrange(67, 75) == unsealed.subrange(67, 75));
    assert(unit.subrange(75, 83) == unsealed.subrange(75, 83));
    assert(unit.subrange(83, 91) == unsealed.subrange(83, 91));
    assert(unit.subrange(91, 95) == unsealed.subrange(91, 95));
    assert(unit.subrange(95, 101) == unsealed.subrange(95, 101));
    assert(specification_unit_filesystem_identifier(filesystem_identifier) < 0x1_0000_0000_0000_0000);
}

} // verus!

#[cfg(test)]
mod tests {
    use super::*;

    /// 外部审核 ⑥：声明长度越过单元末尾（134 + 40000 > 32768）时 `data_unit_payload` 自己报结构错，不在切片上越界 panic；
    /// 不经 `parse_data_unit` 直接调它。
    #[test]
    fn data_unit_payload_with_a_declared_length_past_the_unit_end_is_a_structure_error() {
        let unit = build_data_unit(
            DataUnitIdentity {
                tree: TreeIdentifier(singlefs_format::TREE_IDENTIFIER_EXTENT),
                object: 1,
                object_birth: CheckpointTxg(3),
                anchor_offset: 0,
            },
            CheckpointTxg(3),
            &[9u8; 16],
            WriteOrder {
                instance: InstanceGeneration(1),
                transaction: 1,
            },
            &[5u8; 100],
        );
        assert_eq!(
            data_unit_payload(&unit, 40_000),
            Err(UnitError::Structure(
                UnitStructureDefect::DataUnitDeclaredLengthPastTheUnitEnd,
            )),
            "声明长度 40000：载荷末尾越过 32768 字节的单元"
        );
    }

    /// 声明长度恰好等于载荷容量（32634，载荷末尾就是单元末尾）：越界那一判不拒，补齐区是空的，交回整段载荷。
    #[test]
    fn data_unit_payload_with_a_declared_length_reaching_the_unit_end_exactly_is_the_whole_payload()
    {
        let capacity = data_unit_payload_capacity();
        let unit = build_data_unit(
            DataUnitIdentity {
                tree: TreeIdentifier(singlefs_format::TREE_IDENTIFIER_EXTENT),
                object: 1,
                object_birth: CheckpointTxg(3),
                anchor_offset: 0,
            },
            CheckpointTxg(3),
            &[9u8; 16],
            WriteOrder {
                instance: InstanceGeneration(1),
                transaction: 1,
            },
            &vec![6u8; capacity],
        );
        assert_eq!(
            data_unit_payload(&unit, u16::try_from(capacity).expect("32634 装得进两字节"))
                .map(<[u8]>::len),
            Ok(capacity),
            "载荷末尾等于单元末尾"
        );
    }
}
