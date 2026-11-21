//! 单元的头：共同前缀 42（D18（块里携带什么信息） 已定项 7），码 2 索引节点头（已定项 18），码 3 打包记录单元头（已定项 11）。
//!
//! 整份在 `verus!` 里，由 Verus 直接验（里程碑四出口 5「core 直验」编解码与记录格式这一簇）。三类单元的写者各交出一份写法规格
//! （[`specification_index_node_bytes`]、[`specification_packed_unit_bytes`]、[`specification_data_unit_bytes`]：先按字段表写成没封的那一份，再写载荷 CRC、
//! 再封头校验和，[`specification_sealed_unit`]），读者各交出一份读法规格（[`specification_parse_index_node`]、[`specification_parse_packed_unit`]、
//! [`specification_parse_data_unit`]），往返引理（[`lemma_index_node_round_trip`]、[`lemma_packed_unit_round_trip`]、
//! [`lemma_data_unit_round_trip`]）证写者写出来的单元读者读回同一份头与条目。CRC-32C 在 Verus 里是未解释函数
//! （`checksum::specification_crc32_castagnoli`，D13（验证路线） 已定项 14），往返只用到「同一段字节同一个数」。

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

mod data_unit;
mod index_node;
mod packed_unit;
pub use data_unit::*;
pub use index_node::*;
pub use packed_unit::*;

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
    pub open spec fn specification_bytes(self) -> Seq<u8> {
        little_endian_bytes(self.instance.0 as nat, 4) + little_endian_bytes(self.transaction as nat, 6)
    }

    /// 从 `source` 的 `position` 起读 10 字节解成的写序。
    pub open spec fn specification_decode_at(source: Seq<u8>, position: int) -> Self {
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
            source.subrange(position, position + 10) == self.specification_bytes(),
        ensures
            Self::specification_decode_at(source, position) == self,
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
pub open spec fn specification_unit_filesystem_identifier(filesystem_identifier: [u8; 16]) -> u64 {
    little_endian_value(filesystem_identifier@.subrange(0, 8)) as u64
}

/// 共同前缀 42 字节（[`write_common_prefix`] 的规格）：magic 4、格式版本 2、类标签 1、flags 1、声明长度 2、头校验和 32（写的时候是 0）。
pub open spec fn specification_common_prefix_bytes(unit_class: u8, declared_length: u16) -> Seq<u8> {
    UNIT_MAGIC@ + little_endian_bytes(UNIT_FORMAT_VERSION as nat, 2) + little_endian_bytes(unit_class as nat, 1)
        + little_endian_bytes(0, 1) + little_endian_bytes(declared_length as nat, 2) + zero_bytes(
        WIDE_CHECKSUM_BYTES as nat,
    )
}

/// 封了头校验和的单元（[`seal_header_checksum`] 的规格）：偏移 10 那 32 字节换成罩 `[0, header_end)`、自身按 0 算出来的字段。
pub open spec fn specification_header_checksum_sealed(bytes: Seq<u8>, header_end: int) -> Seq<u8> {
    bytes.subrange(0, HEADER_CHECKSUM_OFFSET as int) + specification_wide_checksum_field(
        bytes,
        header_end,
        HEADER_CHECKSUM_OFFSET as int,
    ) + bytes.subrange(HEADER_CHECKSUM_OFFSET + WIDE_CHECKSUM_BYTES, bytes.len() as int)
}

/// 写了载荷 CRC 的单元（[`seal_payload_checksum`] 的规格）：`payload_checksum_offset` 起 4 字节换成罩 `[header_end, 末尾)` 的 CRC-32C 小端。
pub open spec fn specification_payload_checksum_written(bytes: Seq<u8>, header_end: int, payload_checksum_offset: int) -> Seq<u8> {
    bytes.subrange(0, payload_checksum_offset) + little_endian_bytes(
        specification_crc32_castagnoli(bytes.subrange(header_end, bytes.len() as int)) as nat,
        4,
    ) + bytes.subrange(payload_checksum_offset + 4, bytes.len() as int)
}

/// 三类单元的写者都是先写载荷 CRC、再封头校验和（头校验和罩住载荷 CRC 那 4 字节）。
pub open spec fn specification_sealed_unit(unsealed: Seq<u8>, header_end: int, payload_checksum_offset: int) -> Seq<u8> {
    specification_header_checksum_sealed(specification_payload_checksum_written(unsealed, header_end, payload_checksum_offset), header_end)
}

/// 头校验和那一判（读者一侧）：偏移 10 那 32 字节等于罩 `[0, header_end)` 现算的字段。
pub open spec fn specification_header_checksum_holds(bytes: Seq<u8>, header_end: int) -> bool {
    bytes.subrange(HEADER_CHECKSUM_OFFSET as int, HEADER_CHECKSUM_OFFSET + WIDE_CHECKSUM_BYTES) == specification_wide_checksum_field(
        bytes,
        header_end,
        HEADER_CHECKSUM_OFFSET as int,
    )
}

/// 载荷 CRC 那一判（读者一侧，[`check_payload_checksum`] 的规格）。
pub open spec fn specification_payload_checksum_holds(bytes: Seq<u8>, header_end: int, payload_checksum_offset: int) -> bool {
    little_endian_value(bytes.subrange(payload_checksum_offset, payload_checksum_offset + 4)) == specification_crc32_castagnoli(
        bytes.subrange(header_end, bytes.len() as int),
    ) as nat
}

/// 类身份段之后那 29 字节预留位全 0（[`check_encryption_reserved_bytes_are_zero`] 的规格）。
pub open spec fn specification_encryption_reserved_bytes_are_zero(bytes: Seq<u8>, header_end: int) -> bool {
    bytes.subrange(header_end, header_end + NONCE_MAC_ALGORITHM_RESERVED_BYTES) == zero_bytes(
        NONCE_MAC_ALGORITHM_RESERVED_BYTES as nat,
    )
}

/// 定宽的几段依次接起来（记录区、条目区）：从末尾那一段往前剥，与写者逐段追加同一个次序。
pub open spec fn specification_concatenation(pieces: Seq<Seq<u8>>) -> Seq<u8>
    decreases pieces.len(),
{
    if pieces.len() == 0 {
        Seq::empty()
    } else {
        specification_concatenation(pieces.drop_last()) + pieces.last()
    }
}

/// 一串 `Vec<u8>` 各自的字节。
pub open spec fn specification_byte_sequences_of_vectors(vectors: Seq<Vec<u8>>) -> Seq<Seq<u8>> {
    vectors.map_values(|vector: Vec<u8>| vector@)
}

/// 每一段都恰好 `width` 字节。
pub open spec fn specification_every_piece_has_width(pieces: Seq<Seq<u8>>, width: nat) -> bool {
    forall|piece_index: int| 0 <= piece_index < pieces.len() ==> (#[trigger] pieces[piece_index]).len() == width
}

/// 从 `start` 起切 `count` 段、各 `width` 字节（读者切记录区、条目区的读法）。
pub open spec fn specification_pieces_at(bytes: Seq<u8>, start: int, count: nat, width: nat) -> Seq<Seq<u8>> {
    Seq::new(count, |piece_index: int| bytes.subrange(start + piece_index * width, start + (piece_index + 1) * width))
}

/// 定宽的 `n` 段接起来是 `n × width` 字节。
pub proof fn lemma_concatenation_of_pieces_of_one_width(pieces: Seq<Seq<u8>>, width: nat)
    requires
        specification_every_piece_has_width(pieces, width),
    ensures
        specification_concatenation(pieces).len() == pieces.len() * width,
    decreases pieces.len(),
{
    if pieces.len() > 0 {
        let shorter = pieces.drop_last();
        assert(specification_every_piece_has_width(shorter, width)) by {
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
        specification_every_piece_has_width(pieces, width),
        0 <= piece_index < pieces.len(),
    ensures
        specification_concatenation(pieces).subrange(piece_index * width, (piece_index + 1) * width) == pieces[piece_index],
    decreases pieces.len(),
{
    let shorter = pieces.drop_last();
    assert(specification_every_piece_has_width(shorter, width)) by {
        assert forall|shorter_index: int| 0 <= shorter_index < shorter.len() implies (#[trigger] shorter[shorter_index]).len()
            == width by {
            assert(shorter[shorter_index] == pieces[shorter_index]);
        }
    }
    lemma_concatenation_of_pieces_of_one_width(shorter, width);
    let whole = specification_concatenation(pieces);
    let front = specification_concatenation(shorter);
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
        specification_every_piece_has_width(pieces, width),
        0 <= start,
        start + pieces.len() * width <= bytes.len(),
        bytes.subrange(start, start + pieces.len() * width) == specification_concatenation(pieces),
    ensures
        specification_pieces_at(bytes, start, pieces.len(), width) == pieces,
{
    let cut = specification_pieces_at(bytes, start, pieces.len(), width);
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
        assert(cut[piece_index] =~= specification_concatenation(pieces).subrange(piece_index * width, (piece_index + 1) * width));
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
            let sealed = specification_sealed_unit(unsealed, header_end, payload_checksum_offset);
            &&& sealed.len() == unsealed.len()
            &&& forall|position: int|
                0 <= position < unsealed.len() && !(HEADER_CHECKSUM_OFFSET <= position < UNIT_COMMON_PREFIX_BYTES) && !(
                payload_checksum_offset <= position < payload_checksum_offset + 4) ==> #[trigger] sealed[position]
                    == unsealed[position]
            &&& specification_header_checksum_holds(sealed, header_end)
            &&& specification_payload_checksum_holds(sealed, header_end, payload_checksum_offset)
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
    let with_payload_checksum = specification_payload_checksum_written(unsealed, header_end, payload_checksum_offset);
    let crc = specification_crc32_castagnoli(unsealed.subrange(header_end, unsealed.len() as int));
    assert(with_payload_checksum.len() == unsealed.len());
    let field = specification_wide_checksum_field(with_payload_checksum, header_end, 10);
    assert(field.len() == 32);
    let sealed = specification_sealed_unit(unsealed, header_end, payload_checksum_offset);
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
    assert(specification_covered_with_the_field_zeroed(sealed, header_end, 10) =~= specification_covered_with_the_field_zeroed(
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
        identifier_in_unit_headers == specification_unit_filesystem_identifier(*filesystem_identifier),
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
        final(writer).written() == specification_common_prefix_bytes(unit_class, declared_length),
        final(writer).total_bytes() == old(writer).total_bytes(),
{
    writer.put(&UNIT_MAGIC);
    writer.put_u16(UNIT_FORMAT_VERSION);
    writer.put_u8(unit_class);
    writer.put_u8(0);
    writer.put_u16(declared_length);
    writer.skip(usize::try_from(WIDE_CHECKSUM_BYTES).expect("32"));
    assert(writer.written() =~= specification_common_prefix_bytes(unit_class, declared_length));
    writer.assert_position(UNIT_COMMON_PREFIX_BYTES, "共同前缀");
}

/// 头校验和覆盖 [0, 明文头末尾)，自身按 0 参与（I-2.4）。
///
/// # Panics
/// 头校验和字段不整个落在 `[0, header_end)` 里，或 `header_end` 过了 `bytes` 末尾（`requires` 排除的那一种）。
pub fn seal_header_checksum(bytes: &mut [u8], header_end: usize)
    requires
        specification_wide_checksum_field_fits(old(bytes)@, header_end as int, HEADER_CHECKSUM_OFFSET as int),
    ensures
        final(bytes)@ == specification_header_checksum_sealed(old(bytes)@, header_end as int),
{
    let digest = wide_checksum_with_field_zeroed(bytes, header_end, HEADER_CHECKSUM_OFFSET);
    bytes[HEADER_CHECKSUM_OFFSET..HEADER_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
    assert(bytes@ =~= specification_header_checksum_sealed(old(bytes)@, header_end as int));
}

/// 载荷 CRC 罩 `[header_end, 末尾)`，小端 4 字节写进 `payload_checksum_offset`：三类单元的写者共用这一步（原来各写一份
/// `copy_from_slice(&crc.to_le_bytes())`，`to_le_bytes` 没有 Verus 规格，经 `ByteWriter` 写成同样的 4 字节）。
fn seal_payload_checksum(bytes: &mut [u8], header_end: usize, payload_checksum_offset: usize)
    requires
        payload_checksum_offset + 4 <= header_end,
        header_end <= old(bytes)@.len(),
    ensures
        final(bytes)@ == specification_payload_checksum_written(
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
    assert(bytes@ =~= specification_payload_checksum_written(old(bytes)@, header_end as int, payload_checksum_offset as int));
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
    pub open spec fn specification_bytes(self) -> Seq<u8> {
        little_endian_bytes(self.birth_tree.0 as nat, 8) + little_endian_bytes(self.record_type as nat, 2)
            + little_endian_bytes(self.container as nat, 8) + little_endian_bytes(self.container_birth.0 as nat, 8)
    }

    /// 从 `source` 的 `position` 起读 26 字节解成的身份。
    pub open spec fn specification_decode_at(source: Seq<u8>, position: int) -> Self {
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
            source.subrange(position, position + 26) == self.specification_bytes(),
        ensures
            Self::specification_decode_at(source, position) == self,
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
        checked is Ok <==> specification_encryption_reserved_bytes_are_zero(bytes@, header_end as int),
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
pub open spec fn specification_check_common_prefix(bytes: Seq<u8>, expected_class: u8, header_end: int) -> Result<u16, UnitError> {
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
    } else if !specification_header_checksum_holds(bytes, header_end) {
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
        declared == specification_check_common_prefix(bytes@, expected_class, header_end as int),
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
        checked is Ok <==> specification_payload_checksum_holds(bytes@, header_end as int, payload_crc_offset as int),
        checked matches Err(error) ==> error == UnitError::PayloadChecksumMismatch,
{
    let mut reader = ByteReader::at(bytes, payload_crc_offset);
    if reader.get_u32() != crc32_castagnoli(&bytes[header_end..]) {
        return Err(UnitError::PayloadChecksumMismatch);
    }
    Ok(())
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
        specification_byte_sequences_of_vectors(pieces@) == specification_pieces_at(bytes@, start as int, count as nat, width as nat),
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
    assert(specification_byte_sequences_of_vectors(pieces@) =~= specification_pieces_at(bytes@, start as int, count as nat, width as nat));
    pieces
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
        source.subrange(0, 10) == specification_common_prefix_bytes(unit_class, declared_length).subrange(0, 10),
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
    assert(specification_common_prefix_bytes(unit_class, declared_length).subrange(0, 10) =~= magic + version + class + flags
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

} // verus!

#[cfg(test)]
mod tests {
    use super::*;

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
}
