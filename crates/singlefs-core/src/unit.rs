//! 单元的头：共同前缀 42（D18（块里携带什么信息） 已定项 7），码 2 索引节点头（已定项 18），码 3 打包记录单元头（已定项 11）。
//!
//! 整份在 `verus!` 里，由 Verus 直接验（里程碑四出口 5「core 直验」编解码与记录格式这一簇）。三类单元的写者各交出一份写法规格
//! （[`spec_index_node_bytes`]、[`spec_packed_unit_bytes`]、[`spec_data_unit_bytes`]：先按字段表写成没封的那一份，再写载荷 CRC、
//! 再封头校验和，[`spec_sealed_unit`]），读者各交出一份读法规格（[`spec_parse_index_node`]、[`spec_parse_packed_unit`]、
//! [`spec_parse_data_unit`]），往返引理（[`lemma_index_node_round_trip`]、[`lemma_packed_unit_round_trip`]、
//! [`lemma_data_unit_round_trip`]）证写者写出来的单元读者读回同一份头与条目。CRC-32C 在 Verus 里是未解释函数
//! （`checksum::spec_crc32_castagnoli`，D13（验证路线） 已定项 14），往返只用到「同一段字节同一个数」。

use singlefs_format::{
    DATA_UNIT_BYTES, DATA_UNIT_HEADER_BYTES, INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE,
    INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT, NODE_BYTES, NONCE_MAC_ALGORITHM_RESERVED_BYTES,
    PACKED_UNIT_HEADER_BYTES, UNIT_COMMON_PREFIX_BYTES, WIDE_CHECKSUM_BYTES,
};

use crate::address::{CheckpointTxg, InstanceGeneration, TreeIdentifier};
// 规格与引理只在 Verus 编的时候有（平时的构建把它们擦掉），按名字引会编不过，所以整个模块引进来。
use crate::bytes::*;
// 规格与引理只在 Verus 编的时候有（平时的构建把它们擦掉），按名字引会编不过，所以整个模块引进来。
use crate::checksum::*;
use crate::pointer::BirthSequence;
use vstd::prelude::*;

verus! {

/// 单元的 magic「SFSU」（写成四个字节值：Verus 不认字节串字面量）。
pub const UNIT_MAGIC: [u8; 4] = [0x53, 0x46, 0x53, 0x55];
pub const UNIT_FORMAT_VERSION: u16 = 1;
/// 单元类型标签（D18（块里携带什么信息） 已定项 11 的登记表）。
pub const UNIT_CLASS_DATA: u8 = 1;
pub const UNIT_CLASS_INDEX_NODE: u8 = 2;
pub const UNIT_CLASS_PACKED: u8 = 3;
/// 打包记录类型（第二级登记表）：1 = 墓碑（D18（块里携带什么信息） 已定项 24），2 = inode 记录，3 = 退役记录，4 = 实例表。
pub const PACKED_TYPE_TOMBSTONE: u16 = 1;
pub const PACKED_TYPE_INODE: u16 = 2;
/// 退役记录（D18（块里携带什么信息） 已定项 11 登记的类型 3；记录宽 34第 2 条）。
pub const PACKED_TYPE_RETIREMENT: u16 = 3;
pub const PACKED_TYPE_INSTANCE_TABLE: u16 = 4;
/// 谱系记录（D18（块里携带什么信息） 已定项 24 第 5 条：不复用登记过的类型 3）。
pub const PACKED_TYPE_LINEAGE: u16 = 5;
/// 头校验和住共同前缀偏移 10。
pub const HEADER_CHECKSUM_OFFSET: usize = 10;

} // verus!

verus! {

/// 写序：实例代号 4 + 事务号低 48 位（D18（块里携带什么信息） 已定项 7）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WriteOrder {
    pub instance: InstanceGeneration,
    pub transaction: u64,
}

impl WriteOrder {
    /// 事务号装得进 6 字节：写者能把它写成盘上那 10 字节。
    pub open spec fn is_encodable(self) -> bool {
        self.transaction < 0x1_0000_0000_0000
    }

    /// 盘上 10 字节：实例代号 4 + 事务号低 6 字节，小端。
    pub open spec fn spec_bytes(self) -> Seq<u8> {
        little_endian_bytes(self.instance.0 as nat, 4) + little_endian_bytes(self.transaction as nat, 6)
    }

    /// 从 `source` 的 `position` 起读 10 字节解成的写序。
    pub open spec fn spec_decode_at(source: Seq<u8>, position: int) -> Self {
        WriteOrder {
            instance: InstanceGeneration(little_endian_value(source.subrange(position, position + 4)) as u32),
            transaction: little_endian_value(source.subrange(position + 4, position + 10)) as u64,
        }
    }

    /// 往返：一段字节里 `position` 起那 10 字节是这个写序写成的，就解回这个写序。
    pub proof fn lemma_decode_at_round_trip(self, source: Seq<u8>, position: int)
        requires
            self.is_encodable(),
            0 <= position,
            position + 10 <= source.len(),
            source.subrange(position, position + 10) == self.spec_bytes(),
        ensures
            Self::spec_decode_at(source, position) == self,
    {
        broadcast use group_little_endian_round_trips;
        lemma_little_endian_capacities();
        lemma_split_a_concatenated_range(
            source,
            position,
            little_endian_bytes(self.instance.0 as nat, 4),
            little_endian_bytes(self.transaction as nat, 6),
        );
    }
}

} // verus!

verus! {

broadcast use group_little_endian_round_trips;

// ===== 这一份文件用到的 singlefs-format 常量的规格 =====
// Verus 不认没进 verus! 的 crate 里的常量，所以被验区域用到的格式常量各写一条 assume_specification，交回它在 singlefs-format 里的值
// （写法照 verified_model.rs「singlefs-format 常量的规格」那一段；全 core 一个常量只写一条，别的文件要用就直接引）。每一条登记进
// 信任清单 .claude/gate.d/verus-trust-list.tsv，值写错由读 core 源码的差分测试判。
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::NODE_BYTES ] -> (node_bytes: u64) returns 16384u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::NONCE_MAC_ALGORITHM_RESERVED_BYTES ] -> (reserved_bytes: u64) returns 29u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE ] -> (header_bytes: u64) returns 86u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT ] -> (key_count: u64) returns 2u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::DATA_UNIT_BYTES ] -> (data_unit_bytes: u64) returns 32768u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::DATA_UNIT_HEADER_BYTES ] -> (header_bytes: u64) returns 105u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::UNIT_COMMON_PREFIX_BYTES ] -> (prefix_bytes: u64) returns 42u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::PACKED_UNIT_HEADER_BYTES ] -> (header_bytes: u64) returns 107u64;

fn reserved_bytes() -> (reserved: usize)
    ensures
        reserved == NONCE_MAC_ALGORITHM_RESERVED_BYTES,
{
    usize::try_from(NONCE_MAC_ALGORITHM_RESERVED_BYTES).expect("29")
}

// ===== 三类单元共用的规格 =====

/// 单元头里的 fsid（[`unit_filesystem_identifier`] 的规格）：系统配置 fsid 前 8 字节的小端值。
pub open spec fn spec_unit_filesystem_identifier(filesystem_identifier: [u8; 16]) -> u64 {
    little_endian_value(filesystem_identifier@.subrange(0, 8)) as u64
}

/// 共同前缀 42 字节（[`write_common_prefix`] 的规格）：magic 4、格式版本 2、类标签 1、flags 1、声明长度 2、头校验和 32（写的时候是 0）。
pub open spec fn spec_common_prefix_bytes(unit_class: u8, declared_length: u16) -> Seq<u8> {
    UNIT_MAGIC@ + little_endian_bytes(UNIT_FORMAT_VERSION as nat, 2) + little_endian_bytes(unit_class as nat, 1)
        + little_endian_bytes(0, 1) + little_endian_bytes(declared_length as nat, 2) + zero_bytes(
        WIDE_CHECKSUM_BYTES as nat,
    )
}

/// 封了头校验和的单元（[`seal_header_checksum`] 的规格）：偏移 10 那 32 字节换成罩 `[0, header_end)`、自身按 0 算出来的字段。
pub open spec fn spec_header_checksum_sealed(bytes: Seq<u8>, header_end: int) -> Seq<u8> {
    bytes.subrange(0, HEADER_CHECKSUM_OFFSET as int) + spec_wide_checksum_field(
        bytes,
        header_end,
        HEADER_CHECKSUM_OFFSET as int,
    ) + bytes.subrange(HEADER_CHECKSUM_OFFSET + WIDE_CHECKSUM_BYTES, bytes.len() as int)
}

/// 写了载荷 CRC 的单元（[`seal_payload_checksum`] 的规格）：`payload_checksum_offset` 起 4 字节换成罩 `[header_end, 末尾)` 的 CRC-32C 小端。
pub open spec fn spec_payload_checksum_written(bytes: Seq<u8>, header_end: int, payload_checksum_offset: int) -> Seq<u8> {
    bytes.subrange(0, payload_checksum_offset) + little_endian_bytes(
        spec_crc32_castagnoli(bytes.subrange(header_end, bytes.len() as int)) as nat,
        4,
    ) + bytes.subrange(payload_checksum_offset + 4, bytes.len() as int)
}

/// 三类单元的写者都是先写载荷 CRC、再封头校验和（头校验和罩住载荷 CRC 那 4 字节）。
pub open spec fn spec_sealed_unit(unsealed: Seq<u8>, header_end: int, payload_checksum_offset: int) -> Seq<u8> {
    spec_header_checksum_sealed(spec_payload_checksum_written(unsealed, header_end, payload_checksum_offset), header_end)
}

/// 头校验和那一判（读者一侧）：偏移 10 那 32 字节等于罩 `[0, header_end)` 现算的字段。
pub open spec fn spec_header_checksum_holds(bytes: Seq<u8>, header_end: int) -> bool {
    bytes.subrange(HEADER_CHECKSUM_OFFSET as int, HEADER_CHECKSUM_OFFSET + WIDE_CHECKSUM_BYTES) == spec_wide_checksum_field(
        bytes,
        header_end,
        HEADER_CHECKSUM_OFFSET as int,
    )
}

/// 载荷 CRC 那一判（读者一侧，[`check_payload_checksum`] 的规格）。
pub open spec fn spec_payload_checksum_holds(bytes: Seq<u8>, header_end: int, payload_checksum_offset: int) -> bool {
    little_endian_value(bytes.subrange(payload_checksum_offset, payload_checksum_offset + 4)) == spec_crc32_castagnoli(
        bytes.subrange(header_end, bytes.len() as int),
    ) as nat
}

/// 类身份段之后那 29 字节预留位全 0（[`check_encryption_reserved_bytes_are_zero`] 的规格）。
pub open spec fn spec_encryption_reserved_bytes_are_zero(bytes: Seq<u8>, header_end: int) -> bool {
    bytes.subrange(header_end, header_end + NONCE_MAC_ALGORITHM_RESERVED_BYTES) == zero_bytes(
        NONCE_MAC_ALGORITHM_RESERVED_BYTES as nat,
    )
}

/// 定宽的几段依次接起来（记录区、条目区）：从末尾那一段往前剥，与写者逐段追加同一个次序。
pub open spec fn spec_concatenation(pieces: Seq<Seq<u8>>) -> Seq<u8>
    decreases pieces.len(),
{
    if pieces.len() == 0 {
        Seq::empty()
    } else {
        spec_concatenation(pieces.drop_last()) + pieces.last()
    }
}

/// 一串 `Vec<u8>` 各自的字节。
pub open spec fn spec_byte_sequences_of_vectors(vectors: Seq<Vec<u8>>) -> Seq<Seq<u8>> {
    vectors.map_values(|vector: Vec<u8>| vector@)
}

/// 每一段都恰好 `width` 字节。
pub open spec fn spec_every_piece_has_width(pieces: Seq<Seq<u8>>, width: nat) -> bool {
    forall|piece_index: int| 0 <= piece_index < pieces.len() ==> (#[trigger] pieces[piece_index]).len() == width
}

/// 从 `start` 起切 `count` 段、各 `width` 字节（读者切记录区、条目区的读法）。
pub open spec fn spec_pieces_at(bytes: Seq<u8>, start: int, count: nat, width: nat) -> Seq<Seq<u8>> {
    Seq::new(count, |piece_index: int| bytes.subrange(start + piece_index * width, start + (piece_index + 1) * width))
}

/// 定宽的 `n` 段接起来是 `n × width` 字节。
pub proof fn lemma_concatenation_of_pieces_of_one_width(pieces: Seq<Seq<u8>>, width: nat)
    requires
        spec_every_piece_has_width(pieces, width),
    ensures
        spec_concatenation(pieces).len() == pieces.len() * width,
    decreases pieces.len(),
{
    if pieces.len() > 0 {
        let shorter = pieces.drop_last();
        assert(spec_every_piece_has_width(shorter, width)) by {
            assert forall|piece_index: int| 0 <= piece_index < shorter.len() implies (#[trigger] shorter[piece_index]).len()
                == width by {
                assert(shorter[piece_index] == pieces[piece_index]);
            }
        }
        lemma_concatenation_of_pieces_of_one_width(shorter, width);
        assert(pieces.last() == pieces[pieces.len() - 1]);
        assert((pieces.len() - 1) * width + width == pieces.len() * width) by (nonlinear_arith);
    }
}

/// 定宽的 `n` 段接起来，第 `piece_index` 段落在 `[piece_index × width, (piece_index + 1) × width)`。
pub proof fn lemma_piece_of_a_concatenation(pieces: Seq<Seq<u8>>, width: nat, piece_index: int)
    requires
        spec_every_piece_has_width(pieces, width),
        0 <= piece_index < pieces.len(),
    ensures
        spec_concatenation(pieces).subrange(piece_index * width, (piece_index + 1) * width) == pieces[piece_index],
    decreases pieces.len(),
{
    let shorter = pieces.drop_last();
    assert(spec_every_piece_has_width(shorter, width)) by {
        assert forall|shorter_index: int| 0 <= shorter_index < shorter.len() implies (#[trigger] shorter[shorter_index]).len()
            == width by {
            assert(shorter[shorter_index] == pieces[shorter_index]);
        }
    }
    lemma_concatenation_of_pieces_of_one_width(shorter, width);
    let whole = spec_concatenation(pieces);
    let front = spec_concatenation(shorter);
    assert(whole == front + pieces.last());
    if piece_index == pieces.len() - 1 {
        assert(pieces.last() == pieces[piece_index]);
        assert(piece_index * width == front.len());
        assert((piece_index + 1) * width == piece_index * width + width) by (nonlinear_arith);
        assert(whole.subrange(piece_index * width, (piece_index + 1) * width) =~= pieces[piece_index]);
    } else {
        lemma_piece_of_a_concatenation(shorter, width, piece_index);
        assert(shorter[piece_index] == pieces[piece_index]);
        assert((piece_index + 1) * width <= shorter.len() * width) by (nonlinear_arith)
            requires
                piece_index + 1 <= shorter.len(),
        ;
        assert(0 <= piece_index * width) by (nonlinear_arith)
            requires
                0 <= piece_index,
        ;
        assert((piece_index + 1) * width == piece_index * width + width) by (nonlinear_arith);
        assert(whole.subrange(piece_index * width, (piece_index + 1) * width) =~= front.subrange(
            piece_index * width,
            (piece_index + 1) * width,
        ));
    }
}

/// 读者切出来的定宽各段就是写者接起来的那几段：`[start, start + n × width)` 是那 `n` 段接起来的，就按段切回它们。
pub proof fn lemma_pieces_at_a_concatenation(bytes: Seq<u8>, start: int, pieces: Seq<Seq<u8>>, width: nat)
    requires
        spec_every_piece_has_width(pieces, width),
        0 <= start,
        start + pieces.len() * width <= bytes.len(),
        bytes.subrange(start, start + pieces.len() * width) == spec_concatenation(pieces),
    ensures
        spec_pieces_at(bytes, start, pieces.len(), width) == pieces,
{
    let cut = spec_pieces_at(bytes, start, pieces.len(), width);
    assert forall|piece_index: int| 0 <= piece_index < pieces.len() implies #[trigger] cut[piece_index] == pieces[piece_index] by {
        lemma_piece_of_a_concatenation(pieces, width, piece_index);
        assert((piece_index + 1) * width <= pieces.len() * width) by (nonlinear_arith)
            requires
                piece_index + 1 <= pieces.len(),
        ;
        assert(0 <= piece_index * width) by (nonlinear_arith)
            requires
                0 <= piece_index,
        ;
        assert((piece_index + 1) * width == piece_index * width + width) by (nonlinear_arith);
        assert(cut[piece_index] =~= spec_concatenation(pieces).subrange(piece_index * width, (piece_index + 1) * width));
    }
    assert(cut =~= pieces);
}

/// 封好的单元在两个校验和字段之外与没封的那一份逐字节相同，载荷 CRC 与头校验和两判都过：三类单元的往返引理都靠它。
pub proof fn lemma_sealed_unit_keeps_everything_outside_the_two_checksums(
    unsealed: Seq<u8>,
    header_end: int,
    payload_checksum_offset: int,
)
    requires
        UNIT_COMMON_PREFIX_BYTES <= payload_checksum_offset,
        payload_checksum_offset + 4 <= header_end,
        header_end <= unsealed.len(),
    ensures
        ({
            let sealed = spec_sealed_unit(unsealed, header_end, payload_checksum_offset);
            &&& sealed.len() == unsealed.len()
            &&& forall|position: int|
                0 <= position < unsealed.len() && !(HEADER_CHECKSUM_OFFSET <= position < UNIT_COMMON_PREFIX_BYTES) && !(
                payload_checksum_offset <= position < payload_checksum_offset + 4) ==> #[trigger] sealed[position]
                    == unsealed[position]
            &&& spec_header_checksum_holds(sealed, header_end)
            &&& spec_payload_checksum_holds(sealed, header_end, payload_checksum_offset)
            &&& sealed.subrange(header_end, sealed.len() as int) == unsealed.subrange(header_end, unsealed.len() as int)
            &&& forall|start: int, end: int|
                0 <= start <= end <= unsealed.len() && (end <= HEADER_CHECKSUM_OFFSET || UNIT_COMMON_PREFIX_BYTES <= start)
                    && (end <= payload_checksum_offset || payload_checksum_offset + 4 <= start) ==> #[trigger] sealed.subrange(
                    start,
                    end,
                ) == unsealed.subrange(start, end)
        }),
{
    lemma_little_endian_capacities();
    let with_payload_checksum = spec_payload_checksum_written(unsealed, header_end, payload_checksum_offset);
    let crc = spec_crc32_castagnoli(unsealed.subrange(header_end, unsealed.len() as int));
    assert(with_payload_checksum.len() == unsealed.len());
    let field = spec_wide_checksum_field(with_payload_checksum, header_end, 10);
    assert(field.len() == 32);
    let sealed = spec_sealed_unit(unsealed, header_end, payload_checksum_offset);
    assert(sealed == with_payload_checksum.subrange(0, 10) + field + with_payload_checksum.subrange(42, unsealed.len() as int));
    assert(sealed.len() == unsealed.len());
    assert forall|position: int|
        0 <= position < unsealed.len() && !(payload_checksum_offset <= position < payload_checksum_offset + 4)
            implies #[trigger] with_payload_checksum[position] == unsealed[position] by {
        if position < payload_checksum_offset {
            assert(with_payload_checksum[position] == unsealed.subrange(0, payload_checksum_offset)[position]);
        } else {
            assert(with_payload_checksum[position] == unsealed.subrange(
                payload_checksum_offset + 4,
                unsealed.len() as int,
            )[position - payload_checksum_offset - 4]);
        }
    }
    assert forall|position: int|
        0 <= position < unsealed.len() && !(10 <= position < 42) implies #[trigger] sealed[position]
            == with_payload_checksum[position] by {
        if position < 10 {
            assert(sealed[position] == with_payload_checksum.subrange(0, 10)[position]);
        } else {
            assert(sealed[position] == with_payload_checksum.subrange(42, unsealed.len() as int)[position - 42]);
        }
    }
    assert forall|position: int|
        0 <= position < unsealed.len() && !(10 <= position < 42) && !(payload_checksum_offset <= position
            < payload_checksum_offset + 4) implies #[trigger] sealed[position] == unsealed[position] by {
        assert(sealed[position] == with_payload_checksum[position]);
        assert(with_payload_checksum[position] == unsealed[position]);
    }
    // 罩住的那一截（字段按 0）两份相同，所以封进去的字段就是读者现算的字段。
    assert(spec_covered_with_the_field_zeroed(sealed, header_end, 10) =~= spec_covered_with_the_field_zeroed(
        with_payload_checksum,
        header_end,
        10,
    ));
    assert(sealed.subrange(10, 42) =~= field);
    // 载荷那一截三份相同，载荷 CRC 那 4 字节是写进去的那个数。
    assert(sealed.subrange(header_end, sealed.len() as int) =~= unsealed.subrange(header_end, unsealed.len() as int));
    assert(sealed.subrange(payload_checksum_offset, payload_checksum_offset + 4) =~= little_endian_bytes(crc as nat, 4)) by {
        assert forall|position: int| 0 <= position < 4 implies sealed.subrange(
            payload_checksum_offset,
            payload_checksum_offset + 4,
        )[position] == little_endian_bytes(crc as nat, 4)[position] by {
            assert(sealed[payload_checksum_offset + position] == with_payload_checksum[payload_checksum_offset + position]);
            assert(with_payload_checksum[payload_checksum_offset + position] == (unsealed.subrange(0, payload_checksum_offset)
                + little_endian_bytes(crc as nat, 4))[payload_checksum_offset + position]);
        }
    }    assert forall|start: int, end: int|
        0 <= start <= end <= unsealed.len() && (end <= 10 || 42 <= start) && (end <= payload_checksum_offset
            || payload_checksum_offset + 4 <= start) implies #[trigger] sealed.subrange(start, end) == unsealed.subrange(
        start,
        end,
    ) by {
        assert(sealed.subrange(start, end) =~= unsealed.subrange(start, end));
    }
}

// ===== 写者 =====

/// 单元头里的 fsid 是 8 字节：系统配置 fsid 的低 8 字节（D18（块里携带什么信息） 已定项 7）。
#[must_use]
pub fn unit_filesystem_identifier(filesystem_identifier: &[u8; 16]) -> (identifier_in_unit_headers: u64)
    ensures
        identifier_in_unit_headers == spec_unit_filesystem_identifier(*filesystem_identifier),
        little_endian_bytes(identifier_in_unit_headers as nat, 8) == filesystem_identifier@.subrange(0, 8),
{
    // u64::from_le_bytes 与切片转数组没有 Verus 规格：经 ByteReader 读前 8 字节的小端，同一个值。
    let mut reader = ByteReader::at(filesystem_identifier, 0);
    reader.get_u64()
}

/// # Panics
/// 写者不在缓冲起点，或装不下 42 字节（`requires` 排除的两种：三类单元的写者都是 `ByteWriter::new` 之后头一件写它）。
pub fn write_common_prefix(writer: &mut ByteWriter, unit_class: u8, declared_length: u16)
    requires
        old(writer).has_room_for(UNIT_COMMON_PREFIX_BYTES as nat),
        old(writer).written() == Seq::<u8>::empty(),
    ensures
        final(writer).is_well_formed(),
        final(writer).written() == spec_common_prefix_bytes(unit_class, declared_length),
        final(writer).total_bytes() == old(writer).total_bytes(),
{
    writer.put(&UNIT_MAGIC);
    writer.put_u16(UNIT_FORMAT_VERSION);
    writer.put_u8(unit_class);
    writer.put_u8(0);
    writer.put_u16(declared_length);
    writer.skip(usize::try_from(WIDE_CHECKSUM_BYTES).expect("32"));
    assert(writer.written() =~= spec_common_prefix_bytes(unit_class, declared_length));
    writer.assert_position(UNIT_COMMON_PREFIX_BYTES, "共同前缀");
}

/// 头校验和覆盖 [0, 明文头末尾)，自身按 0 参与（I-2.4）。
///
/// # Panics
/// 头校验和字段不整个落在 `[0, header_end)` 里，或 `header_end` 过了 `bytes` 末尾（`requires` 排除的那一种）。
pub fn seal_header_checksum(bytes: &mut [u8], header_end: usize)
    requires
        spec_wide_checksum_field_fits(old(bytes)@, header_end as int, HEADER_CHECKSUM_OFFSET as int),
    ensures
        final(bytes)@ == spec_header_checksum_sealed(old(bytes)@, header_end as int),
{
    let digest = wide_checksum_with_field_zeroed(bytes, header_end, HEADER_CHECKSUM_OFFSET);
    bytes[HEADER_CHECKSUM_OFFSET..HEADER_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
    assert(bytes@ =~= spec_header_checksum_sealed(old(bytes)@, header_end as int));
}

/// 载荷 CRC 罩 `[header_end, 末尾)`，小端 4 字节写进 `payload_checksum_offset`：三类单元的写者共用这一步（原来各写一份
/// `copy_from_slice(&crc.to_le_bytes())`，`to_le_bytes` 没有 Verus 规格，经 `ByteWriter` 写成同样的 4 字节）。
fn seal_payload_checksum(bytes: &mut [u8], header_end: usize, payload_checksum_offset: usize)
    requires
        payload_checksum_offset + 4 <= header_end,
        header_end <= old(bytes)@.len(),
    ensures
        final(bytes)@ == spec_payload_checksum_written(
            old(bytes)@,
            header_end as int,
            payload_checksum_offset as int,
        ),
{
    let payload_crc = crc32_castagnoli(&bytes[header_end..]);
    let mut checksum_writer = ByteWriter::new(4);
    checksum_writer.put_u32(payload_crc);
    let checksum_bytes = checksum_writer.into_bytes();
    proof {
        assert(checksum_bytes@ =~= little_endian_bytes(payload_crc as nat, 4));
    }
    bytes[payload_checksum_offset..payload_checksum_offset + 4].copy_from_slice(checksum_bytes.as_slice());
    assert(bytes@ =~= spec_payload_checksum_written(old(bytes)@, header_end as int, payload_checksum_offset as int));
}

// ===== 码 3 打包记录单元的身份 =====

/// 码 3 打包记录单元的四元组身份（出生树、打包记录类型、容器号、容器出生代）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackedIdentity {
    pub birth_tree: TreeIdentifier,
    pub record_type: u16,
    pub container: u64,
    pub container_birth: CheckpointTxg,
}

impl PackedIdentity {
    /// 盘上 26 字节：出生树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8，小端（退役记录与 inode 内部条目里的身份引用同序同宽）。
    pub open spec fn spec_bytes(self) -> Seq<u8> {
        little_endian_bytes(self.birth_tree.0 as nat, 8) + little_endian_bytes(self.record_type as nat, 2)
            + little_endian_bytes(self.container as nat, 8) + little_endian_bytes(self.container_birth.0 as nat, 8)
    }

    /// 从 `source` 的 `position` 起读 26 字节解成的身份。
    pub open spec fn spec_decode_at(source: Seq<u8>, position: int) -> Self {
        PackedIdentity {
            birth_tree: TreeIdentifier(little_endian_value(source.subrange(position, position + 8)) as u64),
            record_type: little_endian_value(source.subrange(position + 8, position + 10)) as u16,
            container: little_endian_value(source.subrange(position + 10, position + 18)) as u64,
            container_birth: CheckpointTxg(little_endian_value(source.subrange(position + 18, position + 26)) as u64),
        }
    }

    /// 往返：`position` 起那 26 字节是这个身份写成的，就解回它。
    pub proof fn lemma_decode_at_round_trip(self, source: Seq<u8>, position: int)
        requires
            0 <= position,
            position + 26 <= source.len(),
            source.subrange(position, position + 26) == self.spec_bytes(),
        ensures
            Self::spec_decode_at(source, position) == self,
    {
        broadcast use group_little_endian_round_trips;
        lemma_little_endian_capacities();
        let birth_tree = little_endian_bytes(self.birth_tree.0 as nat, 8);
        let record_type = little_endian_bytes(self.record_type as nat, 2);
        let container = little_endian_bytes(self.container as nat, 8);
        let container_birth = little_endian_bytes(self.container_birth.0 as nat, 8);
        lemma_split_a_concatenated_range(source, position, birth_tree + record_type + container, container_birth);
        lemma_split_a_concatenated_range(source, position, birth_tree + record_type, container);
        lemma_split_a_concatenated_range(source, position, birth_tree, record_type);
    }
}

// ===== 码 2 节点的头宽与容量 =====

/// 码 2 节点明文头末尾 `86 + 2 × key 宽`（[`index_node_plain_header_end`] 的式子）。
pub open spec fn spec_index_node_plain_header_end(key_width: int) -> int {
    INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE + INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT * key_width
}

/// 码 2 节点含预留位的头宽 `86 + 2 × key 宽 + 29`（[`index_node_header_bytes`] 的式子）。
pub open spec fn spec_index_node_header_bytes(key_width: int) -> int {
    spec_index_node_plain_header_end(key_width) + NONCE_MAC_ALGORITHM_RESERVED_BYTES
}

/// 一个码 2 节点装得下的定宽条目数 `(16384 − 含预留位的头宽) ÷ 条目宽`（[`index_node_entry_capacity`] 的式子）。
pub open spec fn spec_index_node_entry_capacity(key_width: int, entry_width: int) -> int {
    (NODE_BYTES - spec_index_node_header_bytes(key_width)) / entry_width
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
        header_end == spec_index_node_plain_header_end(key_width as int),
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
        header_bytes == spec_index_node_header_bytes(key_width as int),
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
        spec_index_node_header_bytes(key_width as int) <= NODE_BYTES,
    ensures
        capacity == spec_index_node_entry_capacity(key_width as int, entry_width as int),
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

// ===== 码 3 打包记录单元 =====

/// 码 3 单元没封的头连同预留位，`[0, 136)`，照写者的次序逐段接：共同前缀 42、类标签副本 1、身份 26（出生树 8 + 打包记录类型 2
/// + 容器号 8 + 容器出生代 8）、记录数 2、记录宽 2、诞生代号 8、fsid 8、载荷 CRC 4（写的时候是 0）、写序 10（实例代号 4 + 事务号 6）、
/// 出生序号 4、预留位 29。
pub open spec fn spec_packed_unit_unsealed_header(
    identity: PackedIdentity,
    record_count: nat,
    record_width: u16,
    birth_txg: CheckpointTxg,
    filesystem_identifier: [u8; 16],
    write_order: WriteOrder,
    birth_sequence: BirthSequence,
) -> Seq<u8> {
    spec_common_prefix_bytes(UNIT_CLASS_PACKED, (record_count * record_width) as u16) + little_endian_bytes(
        UNIT_CLASS_PACKED as nat,
        1,
    ) + little_endian_bytes(identity.birth_tree.0 as nat, 8) + little_endian_bytes(identity.record_type as nat, 2)
        + little_endian_bytes(identity.container as nat, 8) + little_endian_bytes(identity.container_birth.0 as nat, 8)
        + little_endian_bytes(record_count, 2) + little_endian_bytes(record_width as nat, 2) + little_endian_bytes(
        birth_txg.0 as nat,
        8,
    ) + little_endian_bytes(spec_unit_filesystem_identifier(filesystem_identifier) as nat, 8) + zero_bytes(4)
        + little_endian_bytes(write_order.instance.0 as nat, 4) + little_endian_bytes(write_order.transaction as nat, 6)
        + little_endian_bytes(birth_sequence.0 as nat, 4) + zero_bytes(NONCE_MAC_ALGORITHM_RESERVED_BYTES as nat)
}

/// 码 3 单元没封的那一份：共同前缀、类身份段、记录依次接在 136 之后、补 0 到 32768。
pub open spec fn spec_packed_unit_unsealed_bytes(
    identity: PackedIdentity,
    record_width: u16,
    records: Seq<Seq<u8>>,
    birth_txg: CheckpointTxg,
    filesystem_identifier: [u8; 16],
    write_order: WriteOrder,
    birth_sequence: BirthSequence,
) -> Seq<u8> {
    let header = spec_packed_unit_unsealed_header(
        identity,
        records.len(),
        record_width,
        birth_txg,
        filesystem_identifier,
        write_order,
        birth_sequence,
    );
    let record_area = spec_concatenation(records);
    header + record_area + zero_bytes((DATA_UNIT_BYTES - header.len() - record_area.len()) as nat)
}

/// [`build_packed_unit`] 写出的整个码 3 单元：载荷 CRC 罩 [107, 32768)、写在 89；头校验和罩 [0, 107)。
pub open spec fn spec_packed_unit_bytes(
    identity: PackedIdentity,
    record_width: u16,
    records: Seq<Seq<u8>>,
    birth_txg: CheckpointTxg,
    filesystem_identifier: [u8; 16],
    write_order: WriteOrder,
    birth_sequence: BirthSequence,
) -> Seq<u8> {
    spec_sealed_unit(
        spec_packed_unit_unsealed_bytes(
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
pub open spec fn spec_packed_unit_record_area_bytes() -> int {
    DATA_UNIT_BYTES - PACKED_UNIT_HEADER_BYTES - NONCE_MAC_ALGORITHM_RESERVED_BYTES
}

/// [`build_packed_unit`] 的前提，也就是 [`lemma_packed_unit_round_trip`] 的前提（往返不另加）：写序的事务号装得进 6 字节、
/// 记录宽不是 0、记录数装得进 2 字节、记录区装得进一个单元、每条记录都是记录宽。
/// 记录宽不是 0 与读者那一判（记录宽 0 而记录数不为 0 就拒）对齐：I-1.7 要记录宽等于登记表为那个打包记录类型登记的宽，登记的宽都不是 0。
pub open spec fn spec_packed_unit_writer_accepts(write_order: WriteOrder, record_width: u16, records: Seq<Seq<u8>>) -> bool {
    &&& write_order.is_encodable()
    &&& record_width > 0
    &&& records.len() <= u16::MAX
    &&& records.len() * record_width <= spec_packed_unit_record_area_bytes()
    &&& spec_every_piece_has_width(records, record_width as nat)
}

/// 码 3 打包记录单元 32768：类身份段 107 + 预留位 29，记录区从 136 起；载荷 CRC 罩 [107, 32768)。
///
/// # Panics
/// 记录数装不进 2 字节、记录区装不进一个单元、或有一条记录的宽与 `record_width` 不同（`requires` 排除的三种：写者先按容量切好记录再调这里）；
/// 写序的事务号装不进 6 字节（同上）。
///
/// `requires` 另要记录宽不是 0（[`spec_packed_unit_writer_accepts`]）：只约束经 Verus 验过的调用方，函数体不判它、不为它 panic；
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
        spec_packed_unit_writer_accepts(write_order, record_width, spec_byte_sequences_of_vectors(records@)),
    ensures
        unit@ == spec_packed_unit_bytes(
            identity,
            record_width,
            spec_byte_sequences_of_vectors(records@),
            birth_txg,
            *filesystem_identifier,
            write_order,
            birth_sequence,
        ),
{
    let ghost record_pieces = spec_byte_sequences_of_vectors(records@);
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
    assert(header == spec_packed_unit_unsealed_header(
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
            record_pieces == spec_byte_sequences_of_vectors(records@),
            record_index <= records@.len(),
            records@.len() * record_width <= spec_packed_unit_record_area_bytes(),
            spec_every_piece_has_width(record_pieces, record_width as nat),
            writer.written() == header + spec_concatenation(record_pieces.subrange(0, record_index as int)),
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
            assert(spec_every_piece_has_width(written_pieces, record_width as nat));
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
    assert(bytes@ =~= spec_packed_unit_unsealed_bytes(
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

// ===== 码 2 索引节点 =====

/// 码 2 节点没封的头连同预留位，`[0, 115 + 2k)`，照写者的次序逐段接：共同前缀 42、树 ID 8、层级 1、key 宽 1、key 区间 2k、诞生代号 8、
/// fsid 8、实例代号 4、出生序号 4、载荷 CRC 4（写的时候是 0）、预留 2、条目数 2、条目宽 2、预留位 29。
pub open spec fn spec_index_node_unsealed_header(
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
    spec_common_prefix_bytes(UNIT_CLASS_INDEX_NODE, (entry_count * entry_width) as u16) + little_endian_bytes(
        tree.0 as nat,
        8,
    ) + little_endian_bytes(level as nat, 1) + little_endian_bytes(smallest_key.len(), 1) + smallest_key + largest_key
        + little_endian_bytes(birth_txg.0 as nat, 8) + little_endian_bytes(
        spec_unit_filesystem_identifier(filesystem_identifier) as nat,
        8,
    ) + little_endian_bytes(instance.0 as nat, 4) + little_endian_bytes(birth_sequence.0 as nat, 4) + zero_bytes(4)
        + zero_bytes(2) + little_endian_bytes(entry_count, 2) + little_endian_bytes(entry_width as nat, 2) + zero_bytes(
        NONCE_MAC_ALGORITHM_RESERVED_BYTES as nat,
    )
}

/// 码 2 节点没封的那一份：共同前缀、类身份段、预留位 29、条目依次接起来、补 0 到 16384。
pub open spec fn spec_index_node_unsealed_bytes(
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
    let header = spec_index_node_unsealed_header(
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
    let entry_area = spec_concatenation(entries);
    header + entry_area + zero_bytes((NODE_BYTES - header.len() - entry_area.len()) as nat)
}

/// [`build_index_node`] 写出的整个码 2 节点：载荷 CRC 罩 [86 + 2k, 16384)、写在 76 + 2k；头校验和罩 [0, 86 + 2k)。
pub open spec fn spec_index_node_bytes(
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
    let header_end = spec_index_node_plain_header_end(smallest_key.len() as int);
    spec_sealed_unit(
        spec_index_node_unsealed_bytes(
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
pub open spec fn spec_index_node_writer_accepts(
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
    &&& entries.len() <= spec_index_node_entry_capacity(key_width, entry_width as int)
    &&& spec_every_piece_has_width(entries, entry_width as nat)
}

/// # Panics
/// 两端 key 不是 `key_width` 宽、`key_width` 装不进一个字节、条目宽是 0、条目多于一个节点装得下的、或有一条条目的宽不是 `entry_width`
/// 或窄于 key 宽（`requires` 排除的几种：写者先按 [`index_node_entry_capacity`] 判「装不下」并报错，不走到这里）。
///
/// `requires` 另要空节点的条目宽也不窄于 key 宽（[`spec_index_node_writer_accepts`]）：只约束经 Verus 验过的调用方，函数体只在有条目时
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
        spec_index_node_writer_accepts(
            key_width as int,
            smallest_key@,
            largest_key@,
            entry_width,
            spec_byte_sequences_of_vectors(entries@),
        ),
    ensures
        node@ == spec_index_node_bytes(
            tree,
            level,
            smallest_key@,
            largest_key@,
            birth_txg,
            *filesystem_identifier,
            instance,
            birth_sequence,
            entry_width,
            spec_byte_sequences_of_vectors(entries@),
        ),
{
    let ghost entry_pieces = spec_byte_sequences_of_vectors(entries@);
    proof {
        assert(entries@.len() * entry_width <= NODE_BYTES - spec_index_node_header_bytes(key_width as int)) by (nonlinear_arith)
            requires
                entries@.len() <= (NODE_BYTES - spec_index_node_header_bytes(key_width as int)) / (entry_width as int),
                entry_width > 0,
                spec_index_node_header_bytes(key_width as int) <= NODE_BYTES,
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
    assert(header == spec_index_node_unsealed_header(
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
            header.len() == spec_index_node_header_bytes(key_width as int),
            entry_pieces == spec_byte_sequences_of_vectors(entries@),
            entry_index <= entries@.len(),
            entries@.len() * entry_width <= NODE_BYTES - spec_index_node_header_bytes(key_width as int),
            key_width <= entry_width,
            spec_every_piece_has_width(entry_pieces, entry_width as nat),
            writer.written() == header + spec_concatenation(entry_pieces.subrange(0, entry_index as int)),
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
            assert(spec_every_piece_has_width(written_pieces, entry_width as nat));
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
    assert(bytes@ =~= spec_index_node_unsealed_bytes(
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
pub open spec fn spec_data_unit_unsealed_header(
    identity: DataUnitIdentity,
    birth_txg: CheckpointTxg,
    filesystem_identifier: [u8; 16],
    write_order: WriteOrder,
    payload_length: nat,
) -> Seq<u8> {
    spec_common_prefix_bytes(UNIT_CLASS_DATA, payload_length as u16) + little_endian_bytes(UNIT_CLASS_DATA as nat, 1)
        + little_endian_bytes(identity.tree.0 as nat, 8) + little_endian_bytes(identity.object as nat, 8)
        + little_endian_bytes(identity.object_birth.0 as nat, 8) + little_endian_bytes(identity.anchor_offset as nat, 8)
        + little_endian_bytes(birth_txg.0 as nat, 8) + little_endian_bytes(
        spec_unit_filesystem_identifier(filesystem_identifier) as nat,
        8,
    ) + little_endian_bytes(write_order.instance.0 as nat, 4) + little_endian_bytes(write_order.transaction as nat, 6)
        + zero_bytes(4) + zero_bytes(NONCE_MAC_ALGORITHM_RESERVED_BYTES as nat)
}

/// 码 1 单元没封的那一份：共同前缀、类身份段、载荷从 134 起、补 0 到 32768。
pub open spec fn spec_data_unit_unsealed_bytes(
    identity: DataUnitIdentity,
    birth_txg: CheckpointTxg,
    filesystem_identifier: [u8; 16],
    write_order: WriteOrder,
    payload: Seq<u8>,
) -> Seq<u8> {
    let header = spec_data_unit_unsealed_header(identity, birth_txg, filesystem_identifier, write_order, payload.len());
    header + payload + zero_bytes((DATA_UNIT_BYTES - header.len() - payload.len()) as nat)
}

/// [`build_data_unit`] 写出的整个码 1 单元：载荷 CRC 罩 [105, 32768)、写在 101；头校验和罩 [0, 105)。
pub open spec fn spec_data_unit_bytes(
    identity: DataUnitIdentity,
    birth_txg: CheckpointTxg,
    filesystem_identifier: [u8; 16],
    write_order: WriteOrder,
    payload: Seq<u8>,
) -> Seq<u8> {
    spec_sealed_unit(
        spec_data_unit_unsealed_bytes(identity, birth_txg, filesystem_identifier, write_order, payload),
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
        unit@ == spec_data_unit_bytes(identity, birth_txg, *filesystem_identifier, write_order, payload@),
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
    assert(bytes@ =~= spec_data_unit_unsealed_bytes(identity, birth_txg, *filesystem_identifier, write_order, payload@));
    seal_payload_checksum(bytes.as_mut_slice(), header_end, payload_crc_offset);
    seal_header_checksum(bytes.as_mut_slice(), header_end);
    bytes
}

// ===== 读者 =====

/// 数据单元的声明长度：共同前缀偏移 8 的那 2 字节。
///
/// # Panics
/// `unit` 短于 10 字节（`requires` 排除的那一种：调用方交的都是判过长度的整个单元）。
#[must_use]
pub fn declared_length(unit: &[u8]) -> (declared: u16)
    requires
        unit@.len() >= 10,
    ensures
        declared == little_endian_value(unit@.subrange(8, 10)),
{
    // u16::from_le_bytes 没有 Verus 规格：经 ByteReader 读偏移 8 的小端 2 字节，同一个值。
    let mut reader = ByteReader::at(unit, 8);
    reader.get_u16()
}

/// 读者判单元时能出的错，按调用方要做的决定分：读不到 / 头坏了 / 载荷坏了 / 结构字段自相矛盾。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitError {
    TooShort,
    BadMagic,
    UnknownFormatVersion,
    NonZeroFlags,
    WrongUnitClass {
        expected: u8,
        found: u8,
    },
    HeaderChecksumMismatch,
    PayloadChecksumMismatch,
    /// 类身份段之后那 29 字节 nonce / MAC / 算法类型预留位有非 0 字节：它们不在头校验和覆盖内（在载荷 CRC 覆盖内），
    /// 由「恒 0、读者遇到非 0 一律判该结构损坏」这条规则守（I-2.4（头校验和覆盖范围）；D18（块里携带什么信息） 已定项 16 / 已定项 17）。
    EncryptionReservedBytesNotZero,
    /// 头里的结构字段自相矛盾或越界：哪一样矛盾见 [`UnitStructureDefect`]。
    Structure(UnitStructureDefect),
}

/// 单元头里自相矛盾或越界的是哪一样结构字段（[`UnitError::Structure`] 带它）。封闭集合，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitStructureDefect {
    /// 码 2 节点：声明长度 ≠ 条目数 × 条目宽（[`parse_index_node`]）。
    IndexNodeDeclaredLengthIsNotEntryCountTimesEntryWidth,
    /// 码 2 节点：条目宽是 0、条目数不是 0（[`parse_index_node`]）——声明长度 0 过得了上一判，切出来的条目却一条都没有。
    IndexNodeEntryWidthZeroWithEntries,
    /// 码 2 节点：条目宽小于 key 宽（[`parse_index_node`]）。
    IndexNodeEntryWidthBelowTheKeyWidth,
    /// 码 2 节点：条目区越过节点末尾（[`parse_index_node`]）。
    IndexNodeEntryAreaPastTheNodeEnd,
    /// 码 3 单元：类身份段里的类标签副本不是 3（[`parse_packed_unit`]）。
    PackedUnitClassTagCopyIsNotThePackedClass,
    /// 码 3 单元：声明长度 ≠ 记录数 × 记录宽（[`parse_packed_unit`]）。
    PackedUnitDeclaredLengthIsNotRecordCountTimesRecordWidth,
    /// 码 3 单元：记录宽是 0、记录数不是 0（[`parse_packed_unit`]），与码 2 那一格同一个坏法。
    PackedUnitRecordWidthZeroWithRecords,
    /// 码 3 单元：记录区越过单元末尾（[`parse_packed_unit`]）。
    PackedUnitRecordAreaPastTheUnitEnd,
    /// 码 1 数据单元：类身份段里的类标签副本不是 1（[`parse_data_unit`]）。
    DataUnitClassTagCopyIsNotTheDataClass,
    /// 码 1 数据单元：声明长度越过单元末尾（[`parse_data_unit`]、[`data_unit_payload`]）。
    DataUnitDeclaredLengthPastTheUnitEnd,
    /// 码 1 数据单元：声明长度之后到单元末尾的补齐字节有非 0 的（[`data_unit_payload`]，I-2.3）。
    DataUnitPaddingBytesNotZero,
}

/// 类身份段之后那 29 字节预留位（从 `header_end` 起）全 0 才过（I-2.4（头校验和覆盖范围））：三类单元的读者共用这一判。
///
/// # Panics
/// 预留位过了 `bytes` 末尾（`requires` 排除的那一种：三类单元的读者先判过整个单元的长度）。
fn check_encryption_reserved_bytes_are_zero(
    bytes: &[u8],
    header_end: usize,
) -> (checked: Result<(), UnitError>)
    requires
        header_end + NONCE_MAC_ALGORITHM_RESERVED_BYTES <= bytes@.len(),
    ensures
        checked is Ok <==> spec_encryption_reserved_bytes_are_zero(bytes@, header_end as int),
        checked matches Err(error) ==> error == UnitError::EncryptionReservedBytesNotZero,
{
    assert(bytes@.len() == bytes.len());
    // .iter().any(…) 带闭包没有 Verus 规格：换成 every_byte_is_zero（逐字节的循环），同一个判法。
    if !every_byte_is_zero(&bytes[header_end..header_end + reserved_bytes()]) {
        return Err(UnitError::EncryptionReservedBytesNotZero);
    }
    Ok(())
}

/// 共同前缀四关与头校验和那一判（[`check_common_prefix`] 的规格）：都过了交回声明长度。
pub open spec fn spec_check_common_prefix(bytes: Seq<u8>, expected_class: u8, header_end: int) -> Result<u16, UnitError> {
    if bytes.len() < header_end {
        Err(UnitError::TooShort)
    } else if bytes.subrange(0, 4) != UNIT_MAGIC@ {
        Err(UnitError::BadMagic)
    } else if little_endian_value(bytes.subrange(4, 6)) != UNIT_FORMAT_VERSION {
        Err(UnitError::UnknownFormatVersion)
    } else if bytes[6] != expected_class {
        Err(UnitError::WrongUnitClass { expected: expected_class, found: bytes[6] })
    } else if bytes[7] != 0 {
        Err(UnitError::NonZeroFlags)
    } else if !spec_header_checksum_holds(bytes, header_end) {
        Err(UnitError::HeaderChecksumMismatch)
    } else {
        Ok(little_endian_value(bytes.subrange(8, 10)) as u16)
    }
}

/// 共同前缀四关（magic、版本、类标签、flags）+ 头校验和；返回声明长度。
fn check_common_prefix(
    bytes: &[u8],
    expected_class: u8,
    header_end: usize,
) -> (declared: Result<u16, UnitError>)
    requires
        UNIT_COMMON_PREFIX_BYTES <= header_end,
    ensures
        declared == spec_check_common_prefix(bytes@, expected_class, header_end as int),
{
    if bytes.len() < header_end {
        return Err(UnitError::TooShort);
    }
    if bytes[..4] != UNIT_MAGIC {
        proof {
            assert(bytes@.subrange(0, 4) != UNIT_MAGIC@);
        }
        return Err(UnitError::BadMagic);
    }
    assert(bytes@.subrange(0, 4) =~= UNIT_MAGIC@);
    // u16::from_le_bytes 没有 Verus 规格：经 ByteReader 读偏移 4 的小端 2 字节，同一个值。
    let mut version_reader = ByteReader::at(bytes, 4);
    if version_reader.get_u16() != UNIT_FORMAT_VERSION {
        return Err(UnitError::UnknownFormatVersion);
    }
    if bytes[6] != expected_class {
        return Err(UnitError::WrongUnitClass {
            expected: expected_class,
            found: bytes[6],
        });
    }
    if bytes[7] != 0 {
        return Err(UnitError::NonZeroFlags);
    }
    if !crate::checksum::wide_checksum_field_holds(bytes, header_end, HEADER_CHECKSUM_OFFSET) {
        return Err(UnitError::HeaderChecksumMismatch);
    }
    Ok(declared_length(bytes))
}

/// # Panics
/// 载荷 CRC 字段或 `header_end` 过了 `bytes` 末尾（`requires` 排除的那一种：三类单元的读者先判过整个单元的长度）。
fn check_payload_checksum(
    bytes: &[u8],
    header_end: usize,
    payload_crc_offset: usize,
) -> (checked: Result<(), UnitError>)
    requires
        payload_crc_offset + 4 <= bytes@.len(),
        header_end <= bytes@.len(),
    ensures
        checked is Ok <==> spec_payload_checksum_holds(bytes@, header_end as int, payload_crc_offset as int),
        checked matches Err(error) ==> error == UnitError::PayloadChecksumMismatch,
{
    let mut reader = ByteReader::at(bytes, payload_crc_offset);
    if reader.get_u32() != crc32_castagnoli(&bytes[header_end..]) {
        return Err(UnitError::PayloadChecksumMismatch);
    }
    Ok(())
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
    pub open spec fn spec_model(&self) -> IndexNodeHeaderModel {
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
            entries: spec_byte_sequences_of_vectors(self.entries@),
        }
    }
}

/// [`parse_index_node`] 的判法（交回的是 [`IndexNodeHeaderModel`]）。
pub open spec fn spec_parse_index_node(bytes: Seq<u8>) -> Result<IndexNodeHeaderModel, UnitError> {
    if bytes.len() != NODE_BYTES {
        Err(UnitError::TooShort)
    } else {
        let key_width = bytes[51] as int;
        let header_end = spec_index_node_plain_header_end(key_width);
        match spec_check_common_prefix(bytes, UNIT_CLASS_INDEX_NODE, header_end) {
            Err(error) => Err(error),
            Ok(declared_length) => {
                let entry_count = little_endian_value(bytes.subrange(header_end - 4, header_end - 2));
                let entry_width = little_endian_value(bytes.subrange(header_end - 2, header_end));
                let entries_start = header_end + NONCE_MAC_ALGORITHM_RESERVED_BYTES;
                if !spec_payload_checksum_holds(bytes, header_end, header_end - 10) {
                    Err(UnitError::PayloadChecksumMismatch)
                } else if !spec_encryption_reserved_bytes_are_zero(bytes, header_end) {
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
                            entries: spec_pieces_at(bytes, entries_start, entry_count, entry_width),
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
            Ok(header) => spec_parse_index_node(bytes@) == Ok::<IndexNodeHeaderModel, UnitError>(header.spec_model()),
            Err(error) => spec_parse_index_node(bytes@) == Err::<IndexNodeHeaderModel, UnitError>(error),
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

/// 从 `start` 起切 `count` 段、各 `width` 字节，各拷成一个 `Vec`（码 2 的条目区、码 3 的记录区）。
///
/// 原来写成 `chunks(width.max(1)).take(count).map(to_vec).collect()`（切的是 `[start, start + count × width)`）：
/// 宽不为 0 时恰好切出 `count` 段、各 `width` 字节；宽为 0 时调用方已判过段数也是 0，切出空的。迭代器与闭包没有 Verus 规格，
/// 换成同一个结果的下标循环。
fn pieces_of_one_width(bytes: &[u8], start: usize, count: usize, width: usize) -> (pieces: Vec<Vec<u8>>)
    requires
        start + count * width <= bytes@.len(),
    ensures
        spec_byte_sequences_of_vectors(pieces@) == spec_pieces_at(bytes@, start as int, count as nat, width as nat),
{
    assert(bytes@.len() == bytes.len());
    let mut pieces: Vec<Vec<u8>> = Vec::new();
    let mut piece_index: usize = 0;
    while piece_index < count
        invariant
            piece_index <= count,
            bytes@.len() == bytes.len(),
            start + count * width <= bytes@.len(),
            pieces@.len() == piece_index,
            forall|cut_index: int|
                0 <= cut_index < piece_index ==> (#[trigger] pieces@[cut_index])@ == bytes@.subrange(
                    start + cut_index * width,
                    start + (cut_index + 1) * width,
                ),
        decreases count - piece_index,
    {
        proof {
            assert(piece_index * width + width <= count * width) by (nonlinear_arith)
                requires
                    piece_index < count,
            ;
            assert((piece_index + 1) * width == piece_index * width + width) by (nonlinear_arith);
        }
        let piece_start = start + piece_index * width;
        pieces.push(copied_into_a_vector(&bytes[piece_start..piece_start + width]));
        piece_index += 1;
    }
    assert(spec_byte_sequences_of_vectors(pieces@) =~= spec_pieces_at(bytes@, start as int, count as nat, width as nat));
    pieces
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
    pub open spec fn spec_model(&self) -> PackedUnitHeaderModel {
        PackedUnitHeaderModel {
            identity: self.identity,
            birth_txg: self.birth_txg,
            filesystem_identifier: self.filesystem_identifier,
            write_order: self.write_order,
            birth_sequence: self.birth_sequence,
            record_width: self.record_width as nat,
            records: spec_byte_sequences_of_vectors(self.records@),
        }
    }
}

/// [`parse_packed_unit`] 的判法（交回的是 [`PackedUnitHeaderModel`]）。
pub open spec fn spec_parse_packed_unit(bytes: Seq<u8>) -> Result<PackedUnitHeaderModel, UnitError> {
    if bytes.len() != DATA_UNIT_BYTES {
        Err(UnitError::TooShort)
    } else {
        match spec_check_common_prefix(bytes, UNIT_CLASS_PACKED, PACKED_UNIT_HEADER_BYTES as int) {
            Err(error) => Err(error),
            Ok(declared_length) => {
                let record_count = little_endian_value(bytes.subrange(69, 71));
                let record_width = little_endian_value(bytes.subrange(71, 73));
                let records_start = PACKED_UNIT_HEADER_BYTES + NONCE_MAC_ALGORITHM_RESERVED_BYTES;
                if !spec_payload_checksum_holds(bytes, PACKED_UNIT_HEADER_BYTES as int, 89) {
                    Err(UnitError::PayloadChecksumMismatch)
                } else if !spec_encryption_reserved_bytes_are_zero(bytes, PACKED_UNIT_HEADER_BYTES as int) {
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
                            identity: PackedIdentity::spec_decode_at(bytes, 43),
                            birth_txg: CheckpointTxg(little_endian_value(bytes.subrange(73, 81)) as u64),
                            filesystem_identifier: little_endian_value(bytes.subrange(81, 89)) as u64,
                            write_order: WriteOrder::spec_decode_at(bytes, 93),
                            birth_sequence: BirthSequence(little_endian_value(bytes.subrange(103, 107)) as u32),
                            record_width,
                            records: spec_pieces_at(bytes, records_start, record_count, record_width),
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
            Ok(header) => spec_parse_packed_unit(bytes@) == Ok::<PackedUnitHeaderModel, UnitError>(header.spec_model()),
            Err(error) => spec_parse_packed_unit(bytes@) == Err::<PackedUnitHeaderModel, UnitError>(error),
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
pub open spec fn spec_parse_data_unit(bytes: Seq<u8>) -> Result<DataUnitHeader, UnitError> {
    if bytes.len() != DATA_UNIT_BYTES {
        Err(UnitError::TooShort)
    } else {
        match spec_check_common_prefix(bytes, UNIT_CLASS_DATA, DATA_UNIT_HEADER_BYTES as int) {
            Err(error) => Err(error),
            Ok(declared_length) => {
                if !spec_payload_checksum_holds(bytes, DATA_UNIT_HEADER_BYTES as int, 101) {
                    Err(UnitError::PayloadChecksumMismatch)
                } else if !spec_encryption_reserved_bytes_are_zero(bytes, DATA_UNIT_HEADER_BYTES as int) {
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
                            write_order: WriteOrder::spec_decode_at(bytes, 91),
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
        parsed == spec_parse_data_unit(bytes@),
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
pub open spec fn spec_data_unit_payload(bytes: Seq<u8>, declared_length: u16) -> Result<Seq<u8>, UnitError> {
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
            Ok(payload_bytes) => spec_data_unit_payload(bytes@, declared_length) == Ok::<Seq<u8>, UnitError>(
                payload_bytes@,
            ),
            Err(error) => spec_data_unit_payload(bytes@, declared_length) == Err::<Seq<u8>, UnitError>(error),
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

// ===== 往返：写者写出来的单元，读者读回同一份头与条目 =====

/// 1 字节的小端就是那个字节本身。
pub proof fn lemma_one_byte_little_endian(byte: u8)
    ensures
        little_endian_bytes(byte as nat, 1) == seq![byte],
{
    reveal_with_fuel(little_endian_bytes, 2);
    assert(little_endian_bytes(byte as nat, 1) =~= seq![byte]);
}

/// 共同前缀的前 10 字节（头校验和之前那一截）读回写进去的 magic、版本、类标签、flags 与声明长度。
pub proof fn lemma_common_prefix_reads_back(source: Seq<u8>, unit_class: u8, declared_length: u16)
    requires
        source.len() >= 10,
        source.subrange(0, 10) == spec_common_prefix_bytes(unit_class, declared_length).subrange(0, 10),
    ensures
        source.subrange(0, 4) == UNIT_MAGIC@,
        little_endian_value(source.subrange(4, 6)) == UNIT_FORMAT_VERSION,
        source[6] == unit_class,
        source[7] == 0,
        little_endian_value(source.subrange(8, 10)) == declared_length,
{
    lemma_little_endian_capacities();
    let magic = UNIT_MAGIC@;
    let version = little_endian_bytes(UNIT_FORMAT_VERSION as nat, 2);
    let class = little_endian_bytes(unit_class as nat, 1);
    let flags = little_endian_bytes(0, 1);
    let declared = little_endian_bytes(declared_length as nat, 2);
    assert(spec_common_prefix_bytes(unit_class, declared_length).subrange(0, 10) =~= magic + version + class + flags
        + declared);
    lemma_split_a_concatenated_range(source, 0, magic + version + class + flags, declared);
    lemma_split_a_concatenated_range(source, 0, magic + version + class, flags);
    lemma_split_a_concatenated_range(source, 0, magic + version, class);
    lemma_split_a_concatenated_range(source, 0, magic, version);
    lemma_one_byte_little_endian(unit_class);
    lemma_one_byte_little_endian(0);
    assert(source[6] == source.subrange(6, 7)[0]);
    assert(source[7] == source.subrange(7, 8)[0]);
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
            let unit = spec_data_unit_bytes(identity, birth_txg, filesystem_identifier, write_order, payload);
            &&& spec_parse_data_unit(unit) == Ok::<DataUnitHeader, UnitError>(
                DataUnitHeader {
                    identity,
                    birth_txg,
                    filesystem_identifier: spec_unit_filesystem_identifier(filesystem_identifier),
                    write_order,
                    declared_length: payload.len() as u16,
                },
            )
            &&& spec_data_unit_payload(unit, payload.len() as u16) == Ok::<Seq<u8>, UnitError>(payload)
        }),
{
    lemma_little_endian_capacities();
    let declared_length = payload.len() as u16;
    let header = spec_data_unit_unsealed_header(identity, birth_txg, filesystem_identifier, write_order, payload.len());
    let unsealed = spec_data_unit_unsealed_bytes(identity, birth_txg, filesystem_identifier, write_order, payload);
    let padding = zero_bytes((DATA_UNIT_BYTES - header.len() - payload.len()) as nat);
    assert(header.len() == 134);
    assert(unsealed.len() == 32768);
    lemma_sealed_unit_keeps_everything_outside_the_two_checksums(unsealed, 105, 101);
    let unit = spec_data_unit_bytes(identity, birth_txg, filesystem_identifier, write_order, payload);
    // 没封的那一份逐段剥开：头、载荷、补齐，再把头照写者的次序从末尾往前剥。
    assert(unsealed.subrange(0, unsealed.len() as int) =~= unsealed);
    lemma_split_a_concatenated_range(unsealed, 0, header + payload, padding);
    lemma_split_a_concatenated_range(unsealed, 0, header, payload);
    let prefix = spec_common_prefix_bytes(UNIT_CLASS_DATA, declared_length);
    let class = little_endian_bytes(UNIT_CLASS_DATA as nat, 1);
    let tree = little_endian_bytes(identity.tree.0 as nat, 8);
    let object = little_endian_bytes(identity.object as nat, 8);
    let object_birth = little_endian_bytes(identity.object_birth.0 as nat, 8);
    let anchor = little_endian_bytes(identity.anchor_offset as nat, 8);
    let birth = little_endian_bytes(birth_txg.0 as nat, 8);
    let fsid = little_endian_bytes(spec_unit_filesystem_identifier(filesystem_identifier) as nat, 8);
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
    assert(spec_unit_filesystem_identifier(filesystem_identifier) < 0x1_0000_0000_0000_0000);
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
        spec_packed_unit_writer_accepts(write_order, record_width, records),
    ensures
        spec_parse_packed_unit(
            spec_packed_unit_bytes(
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
                filesystem_identifier: spec_unit_filesystem_identifier(filesystem_identifier),
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
    let header = spec_packed_unit_unsealed_header(
        identity,
        record_count,
        record_width,
        birth_txg,
        filesystem_identifier,
        write_order,
        birth_sequence,
    );
    let record_area = spec_concatenation(records);
    let unsealed = spec_packed_unit_unsealed_bytes(
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
    let unit = spec_packed_unit_bytes(
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
    let prefix = spec_common_prefix_bytes(UNIT_CLASS_PACKED, declared_length);
    let class = little_endian_bytes(UNIT_CLASS_PACKED as nat, 1);
    let birth_tree = little_endian_bytes(identity.birth_tree.0 as nat, 8);
    let record_type = little_endian_bytes(identity.record_type as nat, 2);
    let container = little_endian_bytes(identity.container as nat, 8);
    let container_birth = little_endian_bytes(identity.container_birth.0 as nat, 8);
    let count = little_endian_bytes(record_count, 2);
    let width = little_endian_bytes(record_width as nat, 2);
    let birth = little_endian_bytes(birth_txg.0 as nat, 8);
    let fsid = little_endian_bytes(spec_unit_filesystem_identifier(filesystem_identifier) as nat, 8);
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
        entries.len() * entry_width <= NODE_BYTES - spec_index_node_header_bytes(smallest_key.len() as int),
        spec_every_piece_has_width(entries, entry_width as nat),
    ensures
        ({
            let key_width = smallest_key.len() as int;
            let header_end = 86 + 2 * key_width;
            let unsealed = spec_index_node_unsealed_bytes(
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
            &&& unsealed.subrange(0, 42) == spec_common_prefix_bytes(
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
                spec_unit_filesystem_identifier(filesystem_identifier) as nat,
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
            &&& unsealed.subrange(header_end + 29, header_end + 29 + entries.len() * entry_width) == spec_concatenation(
                entries,
            )
        }),
{
    lemma_little_endian_capacities();
    lemma_concatenation_of_pieces_of_one_width(entries, entry_width as nat);
    let entry_count = entries.len();
    let declared_length = (entry_count * entry_width) as u16;
    let header = spec_index_node_unsealed_header(
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
    let entry_area = spec_concatenation(entries);
    let unsealed = spec_index_node_unsealed_bytes(
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
    let prefix = spec_common_prefix_bytes(UNIT_CLASS_INDEX_NODE, declared_length);
    let tree_bytes = little_endian_bytes(tree.0 as nat, 8);
    let level_bytes = little_endian_bytes(level as nat, 1);
    let key_width_bytes = little_endian_bytes(smallest_key.len(), 1);
    let birth = little_endian_bytes(birth_txg.0 as nat, 8);
    let fsid = little_endian_bytes(spec_unit_filesystem_identifier(filesystem_identifier) as nat, 8);
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
        entries.len() * entry_width <= NODE_BYTES - spec_index_node_header_bytes(smallest_key.len() as int),
        spec_every_piece_has_width(entries, entry_width as nat),
    ensures
        ({
            let key_width = smallest_key.len() as int;
            let header_end = 86 + 2 * key_width;
            let node = spec_index_node_bytes(
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
            &&& node.subrange(0, 10) == spec_common_prefix_bytes(
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
                spec_unit_filesystem_identifier(filesystem_identifier) as nat,
                8,
            )
            &&& node.subrange(68 + 2 * key_width, 72 + 2 * key_width) == little_endian_bytes(instance.0 as nat, 4)
            &&& node.subrange(72 + 2 * key_width, 76 + 2 * key_width) == little_endian_bytes(birth_sequence.0 as nat, 4)
            &&& node.subrange(header_end - 4, header_end - 2) == little_endian_bytes(entries.len(), 2)
            &&& node.subrange(header_end - 2, header_end) == little_endian_bytes(entry_width as nat, 2)
            &&& spec_header_checksum_holds(node, header_end)
            &&& spec_payload_checksum_holds(node, header_end, header_end - 10)
            &&& node.subrange(header_end, header_end + 29) == zero_bytes(29)
            &&& node.subrange(header_end + 29, header_end + 29 + entries.len() * entry_width) == spec_concatenation(entries)
        }),
{
    let key_width = smallest_key.len() as int;
    let entry_count = entries.len();
    let header_end = 86 + 2 * key_width;
    let unsealed = spec_index_node_unsealed_bytes(
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
    let node = spec_index_node_bytes(
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
        spec_index_node_writer_accepts(smallest_key.len() as int, smallest_key, largest_key, entry_width, entries),
    ensures
        spec_parse_index_node(
            spec_index_node_bytes(
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
                filesystem_identifier: spec_unit_filesystem_identifier(filesystem_identifier),
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
    assert(entry_count * entry_width <= NODE_BYTES - spec_index_node_header_bytes(key_width)) by (nonlinear_arith)
        requires
            entry_count <= (NODE_BYTES - spec_index_node_header_bytes(key_width)) / (entry_width as int),
            entry_width > 0,
            spec_index_node_header_bytes(key_width) <= NODE_BYTES,
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
    let node = spec_index_node_bytes(
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
    assert(spec_check_common_prefix(node, UNIT_CLASS_INDEX_NODE, header_end) == Ok::<u16, UnitError>(declared_length));
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

    #[test]
    fn the_three_parsers_round_trip_their_builders_and_refuse_a_damaged_header() {
        let filesystem_identifier = [9u8; 16];
        let node = build_index_node(
            TreeIdentifier(singlefs_format::TREE_IDENTIFIER_EXTENT),
            0,
            24,
            &[1u8; 24],
            &[2u8; 24],
            CheckpointTxg(3),
            &filesystem_identifier,
            InstanceGeneration(1),
            BirthSequence(0),
            112,
            &[vec![1u8; 112], vec![2u8; 112]],
        );
        let header = parse_index_node(&node).expect("码 2");
        assert_eq!(
            (header.tree, header.key_width, header.entry_width),
            (
                TreeIdentifier(singlefs_format::TREE_IDENTIFIER_EXTENT),
                24,
                112
            )
        );
        assert_eq!(header.entries.len(), 2);
        assert_eq!(header.smallest_key, vec![1u8; 24]);
        let mut damaged = node.clone();
        damaged[60] ^= 1;
        assert_eq!(
            parse_index_node(&damaged),
            Err(UnitError::HeaderChecksumMismatch)
        );
        let mut wrong_class = node;
        wrong_class[6] = UNIT_CLASS_DATA;
        assert_eq!(
            parse_index_node(&wrong_class),
            Err(UnitError::WrongUnitClass {
                expected: 2,
                found: 1
            })
        );

        let identity = PackedIdentity {
            birth_tree: TreeIdentifier(singlefs_format::TREE_IDENTIFIER_INODE),
            record_type: PACKED_TYPE_INODE,
            container: 1,
            container_birth: CheckpointTxg(3),
        };
        let write_order = WriteOrder {
            instance: InstanceGeneration(1),
            transaction: 1,
        };
        let packed = build_packed_unit(
            identity,
            140,
            &[vec![7u8; 140]],
            CheckpointTxg(3),
            &filesystem_identifier,
            write_order,
            BirthSequence(0),
        );
        let packed_header = parse_packed_unit(&packed).expect("码 3");
        assert_eq!(
            (
                packed_header.identity,
                packed_header.record_width,
                packed_header.records.len()
            ),
            (identity, 140, 1)
        );
        assert_eq!(packed_header.write_order, write_order);

        let data = build_data_unit(
            DataUnitIdentity {
                tree: TreeIdentifier(singlefs_format::TREE_IDENTIFIER_EXTENT),
                object: 1,
                object_birth: CheckpointTxg(3),
                anchor_offset: 0,
            },
            CheckpointTxg(3),
            &filesystem_identifier,
            write_order,
            &[5u8; 3000],
        );
        let data_header = parse_data_unit(&data).expect("码 1");
        assert_eq!(data_header.declared_length, 3000);
        assert_eq!(
            data_unit_payload(&data, 3000).expect("载荷"),
            &[5u8; 3000][..]
        );
        let mut padded = data.clone();
        padded[32000] = 1;
        assert_eq!(
            parse_data_unit(&padded),
            Err(UnitError::PayloadChecksumMismatch),
            "补齐区在载荷 CRC 内"
        );
        let mut torn = data;
        torn[20000] ^= 1;
        assert_eq!(
            parse_data_unit(&torn),
            Err(UnitError::PayloadChecksumMismatch)
        );
    }

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
