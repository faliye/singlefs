//! 块指针（D19（块指针的结构与宽度预算） 已定项 7 / 已定项 8 / 已定项 11）：
//! 头部 50（MAC 16 / nonce 12 / 算法类型 1 / 压缩算法码 1 / 压后长度 2 / extent 偏移 2 / 出生树 8 / 出生 txg 8）
//! + 位置条目 14 × 2（按设备身份升序，I-2.5）+ key 尾段：指向码 2 / 码 3 的带实例代号 4 + 出生序号 4（86）。
//!
//! 整份在 `verus!` 里，由 Verus 直接验（里程碑四出口 5「core 直验」编解码与记录格式这一簇）。每一种指针与位置条目各有
//! 一份写法的规格（`spec_bytes`）、一份读法的规格（`spec_decode_at`：从一段字节的某个位置起读出来的样子）与一条往返引理
//! （`lemma_decode_at_round_trip`：那几个字节是它写成的，就读回它）；写的方法交出的正是写法的规格，读的方法交出的正是读法的规格。

use singlefs_format::{DATA_POINTER_BYTES, LOC_ENTRY, NODE_POINTER_BYTES, POINTER_HEAD_BYTES};
use vstd::prelude::*;

use crate::address::{
    CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber, TreeIdentifier,
};
// 规格与引理只在 Verus 编的时候有（平时的构建把它们擦掉），按名字引会编不过，所以整个模块引进来。
use crate::bytes::*;
use crate::unit::WriteOrder;

verus! {

broadcast use group_little_endian_round_trips;

// ===== 这一份文件用到的 singlefs-format 常量的规格（写法同 unit.rs 头上那一段）=====
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::LOC_ENTRY ] -> (location_entry_bytes: u64) returns 14u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::POINTER_HEAD_BYTES ] -> (pointer_head_bytes: u64) returns 50u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::NODE_POINTER_BYTES ] -> (node_pointer_bytes: u64) returns 86u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::DATA_POINTER_BYTES ] -> (data_pointer_bytes: u64) returns 88u64;

/// 位置条目：设备身份 4 + 16 KiB 槽号 6 + 密文校验和 4（加密关时是整单元 CRC-32C）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocationEntry {
    pub device: DeviceIdentity,
    pub slot: SlotNumber,
    pub unit_checksum: u32,
}

impl LocationEntry {
    /// 槽号装得进 6 字节：写者能把它写成盘上那 14 字节。
    pub open spec fn is_encodable(self) -> bool {
        self.slot.0 < 0x1_0000_0000_0000
    }

    /// 盘上 14 字节。
    pub open spec fn spec_bytes(self) -> Seq<u8> {
        little_endian_bytes(self.device.0 as nat, 4) + little_endian_bytes(self.slot.0 as nat, 6)
            + little_endian_bytes(self.unit_checksum as nat, 4)
    }

    /// 从 `source` 的 `position` 起读 14 字节解成的位置条目。
    pub open spec fn spec_decode_at(source: Seq<u8>, position: int) -> Self {
        LocationEntry {
            device: DeviceIdentity(little_endian_value(source.subrange(position, position + 4)) as u32),
            slot: SlotNumber(little_endian_value(source.subrange(position + 4, position + 10)) as u64),
            unit_checksum: little_endian_value(source.subrange(position + 10, position + 14)) as u32,
        }
    }

    /// 往返：`position` 起那 14 字节是这条位置条目写成的，就解回它。
    pub proof fn lemma_decode_at_round_trip(self, source: Seq<u8>, position: int)
        requires
            self.is_encodable(),
            0 <= position,
            position + 14 <= source.len(),
            source.subrange(position, position + 14) == self.spec_bytes(),
        ensures
            Self::spec_decode_at(source, position) == self,
    {
        lemma_little_endian_capacities();
        let device_bytes = little_endian_bytes(self.device.0 as nat, 4);
        let slot_bytes = little_endian_bytes(self.slot.0 as nat, 6);
        let checksum_bytes = little_endian_bytes(self.unit_checksum as nat, 4);
        lemma_split_a_concatenated_range(source, position, device_bytes + slot_bytes, checksum_bytes);
        lemma_split_a_concatenated_range(source, position, device_bytes, slot_bytes);
    }

    /// # Panics
    /// 写过缓冲末尾或槽号装不进 6 字节（`requires` 排除的两种）。
    pub fn write_to(&self, writer: &mut ByteWriter)
        requires
            old(writer).has_room_for(LOC_ENTRY as nat),
            self.is_encodable(),
        ensures
            final(writer).is_well_formed(),
            final(writer).written() == old(writer).written() + self.spec_bytes(),
            final(writer).total_bytes() == old(writer).total_bytes(),
    {
        let start = writer.position();
        writer.put_u32(self.device.0);
        writer.put_six_byte_unsigned(self.slot.0);
        writer.put_u32(self.unit_checksum);
        assert(writer.written() =~= old(writer).written() + self.spec_bytes());
        assert!(
            writer.position() - start == usize::try_from(LOC_ENTRY).expect("14"),
            "位置条目写了 14 字节"
        );
    }

    pub fn read_from(reader: &mut ByteReader<'_>) -> (entry: Self)
        requires
            old(reader).has_left(LOC_ENTRY as nat),
        ensures
            final(reader).advanced_by(old(reader), LOC_ENTRY as nat),
            entry == Self::spec_decode_at(old(reader).source(), old(reader).spec_position() as int),
            entry.is_encodable(),
    {
        Self {
            device: DeviceIdentity(reader.get_u32()),
            slot: SlotNumber(reader.get_six_byte_unsigned()),
            unit_checksum: reader.get_u32(),
        }
    }
}

/// 两条位置条目的写法：照数组次序接起来，28 字节。
pub open spec fn spec_location_entries_bytes(locations: [LocationEntry; 2]) -> Seq<u8> {
    locations[0].spec_bytes() + locations[1].spec_bytes()
}

/// 两条位置条目的槽号都装得进 6 字节。
pub open spec fn location_entries_are_encodable(locations: [LocationEntry; 2]) -> bool {
    locations[0].is_encodable() && locations[1].is_encodable()
}

/// 两条位置条目按设备身份严格升序（I-2.5）。
pub open spec fn spec_location_entries_ascend_by_device(locations: [LocationEntry; 2]) -> bool {
    locations[0].device.0 < locations[1].device.0
}

/// 一条盘上指针里两条位置条目的槽号不等。**这是一份坏镜像，不是我们的不变量被破坏**：位置条目里的槽号是盘上的
/// 6 字节，可以是任何值；D2（RAID 条带策略） 已定项 10 要的是「每个副本一条位置条目」，而「两条都落在同一个槽」
/// 是第一版的布局约定（一个单元整个落在一列上、两盘各一份）——盘上写着两个不同的槽，说明这一版不是这个实现写出来的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocationEntriesOnDifferentSlots {
    pub devices: [DeviceIdentity; 2],
    pub slots: [SlotNumber; 2],
}

/// 一条盘上指针的两条位置条目共用的那个槽号（第一版两盘同槽）：不等就交回错误，由调用方在动任何状态之前拒绝。
/// 读路径上每一处「拿盘上指针当落点用」都走这一处判，不各写各的（`code-discipline.md`「重复要生成，不许手抄」）。
///
/// # Errors
/// 两条位置条目的槽号不等 ⇒ [`LocationEntriesOnDifferentSlots`]。一份两条位置条目不同槽的镜像不会把可写挂载打 panic。
pub fn slot_shared_by_both_location_entries(
    locations: &[LocationEntry; 2],
) -> (shared: Result<SlotNumber, LocationEntriesOnDifferentSlots>)
    ensures
        shared matches Ok(slot) ==> slot == locations[0].slot && slot == locations[1].slot,
        shared is Err <==> locations[0].slot != locations[1].slot,
{
    if locations[0].slot.0 == locations[1].slot.0 {
        Ok(locations[0].slot)
    } else {
        Err(LocationEntriesOnDifferentSlots {
            devices: [locations[0].device, locations[1].device],
            slots: [locations[0].slot, locations[1].slot],
        })
    }
}

/// 一条盘上指针的两条位置条目不按设备身份严格升序（I-2.5（位置条目按设备身份升序）：相同或逆序一律判损坏）。
/// 例外只有一个：整条全零的码 2 / 码 3 指针（[`NodePointer::empty_root`]，「注册了但还没有根」，两条位置条目都是盘 0）整条豁免，
/// 读者（[`NodePointer::location_entries_ascend_by_device`]）与写者（[`NodePointer::write_to`]）同一个豁免；数据指针没有全零写法，不豁免。
/// **这是一份坏镜像，不是我们的不变量被破坏**：写者（[`NodePointer::write_to`] / [`DataPointer::write_to`]）按严格升序写、在那里断言，
/// 而照抄上一版的发布会把盘上读来的指针原样写回去（树表条目、多层码 2 树的内部条目、根记录）——不在读进来的地方判，
/// 坏镜像就走到写者的断言上（C476（盘上内容可控时走得到的 panic 有 21 处） 普查「缺口」那张表 `pointer.rs` 那一行，实测坐实）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocationEntriesNotAscendingByDevice {
    pub devices: [DeviceIdentity; 2],
}

/// 两条位置条目按设备身份严格升序（I-2.5）。
///
/// # Errors
/// 相同或逆序 ⇒ [`LocationEntriesNotAscendingByDevice`]。
pub fn location_entries_ascend_by_device(
    locations: &[LocationEntry; 2],
) -> (ascending: Result<(), LocationEntriesNotAscendingByDevice>)
    ensures
        ascending is Ok <==> spec_location_entries_ascend_by_device(*locations),
{
    if locations[0].device.0 < locations[1].device.0 {
        Ok(())
    } else {
        Err(LocationEntriesNotAscendingByDevice {
            devices: [locations[0].device, locations[1].device],
        })
    }
}

/// 出生序号：同一棵树在同一个 checkpoint 里每写出一个码 2 / 码 3 单元加 1（D19（块指针的结构与宽度预算） 已定项 9）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BirthSequence(pub u32);

/// 指针头部 50 字节：第一版 MAC / nonce 全 0、算法类型 0、压缩算法码 0、压后长度 0、extent 偏移 0。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerHead {
    pub birth_tree: TreeIdentifier,
    pub birth_txg: CheckpointTxg,
}

/// 指针头部里为加密与压缩留的那几段（D19（块指针的结构与宽度预算） 已定项 3 / 已定项 7）读到的不是第一版写的取值。
/// 这几段是**盘上读来的字节**、不是我们的不变量：读者判到就交回它，由调用方把那条指针所在的结构当损坏处置，不拿这条指针去解引用。
/// 第一版的取值：加密不进第一个可运行版本（D9（加密） 已定项 10），写者把 MAC 16、nonce 12、算法类型 1 写 0；
/// 压缩算法码登记表第一版只有码 0「不压缩」，码 0 时压后长度恒 0（D9（加密） 已定项 14）；extent 偏移那 2 字节第一版写 0
/// （第一版指针指整个单元），读到非 0 同 MAC / nonce 判该指针所在的结构损坏（C581（extent 偏移非 0 没有判据），
/// `research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行：I-2.4 扩到指针头部的 extent 偏移）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerHeadFieldOutsideTheFirstVersion {
    /// MAC 16 或 nonce 12 里有非 0 字节。D19（块指针的结构与宽度预算） 已定项 3 只写了「不加密的卷把它们留成空位」，
    /// 读到非 0 怎么处置条款没写；这里按 I-2.4（头校验和覆盖范围） 给单元头那 29 字节定的「恒 0、读者遇到非 0 一律判该结构损坏」同一个读法。
    MacOrNonceNotZero,
    /// 算法类型不是 0（未加密）：1、2 是登记过的加密算法（D22（单元原子性怎么合成） 已定项 17 的登记表），第一版不支持加密
    /// （D9（加密） 已定项 10），别的码不认识——都判这条指针无法解引用（D9（加密） 已定项 14 末句：与压缩算法码同一条政策）。
    AlgorithmTypeNotUnencrypted { algorithm_type: u8 },
    /// 压缩算法码不是 0：登记表第一版只有码 0（D9（加密） 已定项 14），不认识的码判这条指针无法解引用。
    CompressionAlgorithmCodeNotRecognized { compression_algorithm_code: u8 },
    /// 压缩算法码是 0 而压后长度非 0：判这条指针损坏（D9（加密） 已定项 14「码 0 时它恒 0，非 0 判该指针损坏」）。
    CompressedLengthNotZeroWithoutCompression { compressed_length: u16 },
    /// extent 偏移不是 0（[`PointerHeadExtentOffsetNotZero`]）：不判的话读者从单元头读起，而写者要说的是从偏移处读——静默读错内容。
    ExtentOffsetNotZero(PointerHeadExtentOffsetNotZero),
}

/// 指针头部第一版之外的取值落在哪一组字段上：读者按它挑报给调用方的那一句（`what`），两组各一句，诊断时分得开。
/// 两组的处置相同（判该指针所在的结构损坏，不拿这条指针去解引用）。封闭集合，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerHeadFieldGroup {
    /// MAC / nonce、算法类型、压缩算法码与压后长度（D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14）。
    EncryptionOrCompression,
    /// extent 偏移（C581（extent 偏移非 0 没有判据））。
    ExtentOffset,
}

impl PointerHeadFieldOutsideTheFirstVersion {
    /// 这一处落在哪一组字段上。
    #[must_use]
    pub fn field_group(self) -> (group: PointerHeadFieldGroup)
        ensures
            group == PointerHeadFieldGroup::ExtentOffset <==> self is ExtentOffsetNotZero,
    {
        match self {
            PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero
            | PointerHeadFieldOutsideTheFirstVersion::AlgorithmTypeNotUnencrypted { .. }
            | PointerHeadFieldOutsideTheFirstVersion::CompressionAlgorithmCodeNotRecognized {
                ..
            }
            | PointerHeadFieldOutsideTheFirstVersion::CompressedLengthNotZeroWithoutCompression {
                ..
            } => PointerHeadFieldGroup::EncryptionOrCompression,
            PointerHeadFieldOutsideTheFirstVersion::ExtentOffsetNotZero(_) => {
                PointerHeadFieldGroup::ExtentOffset
            }
        }
    }
}

/// 读盘上指针的读者判不判头部里第一版之外的取值（加密、压缩那几段与 extent 偏移）。封闭集合，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerHeadFieldsJudgement {
    /// 走读与挂载的读者：判，有一段不是第一版的取值就交回 [`PointerHeadFieldOutsideTheFirstVersion`]，调用方判该指针所在的结构损坏
    /// （D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14、C581（extent 偏移非 0 没有判据））。
    JudgedAsTheStructureIsCorruptOutsideTheFirstVersion,
    /// 影子账认一条被抛弃根引用着哪些单元的读者：不判，指针里的位置条目照样认。影子账要的是多隔离、不漏隔离：
    /// 判了就少认一个单元，被抛弃根的那一个不隔离、回退之后可能被发出去。
    NotJudgedByTheShadowLedger,
}

/// 指针头部 extent 偏移那 2 字节（头部偏移 32）读到非 0。第一版写者恒写 0（[`PointerHead::write_to`]），读者遇到非 0
/// 判该指针所在的结构损坏，同 MAC / nonce（C581（extent 偏移非 0 没有判据））。**这是盘上读来的字节、不是我们的不变量**。
/// 带着读到的值。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerHeadExtentOffsetNotZero {
    pub extent_offset: u16,
}

/// 指针头部按字段表读出来的样子，加密与压缩那几段与 extent 偏移还没判：[`PointerHead::read_from`] 与
/// [`PointerHead::read_judging_the_encryption_compression_and_extent_offset_fields_from`] 共用这一份读法，字段表的偏移只写一处。
struct PointerHeadAsRead {
    is_every_mac_and_nonce_byte_zero: bool,
    algorithm_type: u8,
    compression_algorithm_code: u8,
    compressed_length: u16,
    extent_offset: u16,
    head: PointerHead,
}

impl PointerHeadAsRead {
    /// 从 `source` 的 `position` 起读 50 字节、按字段表切出来的样子。
    spec fn spec_decode_at(source: Seq<u8>, position: int) -> Self {
        PointerHeadAsRead {
            is_every_mac_and_nonce_byte_zero: source.subrange(position, position + 28) == zero_bytes(28),
            algorithm_type: little_endian_value(source.subrange(position + 28, position + 29)) as u8,
            compression_algorithm_code: little_endian_value(source.subrange(position + 29, position + 30)) as u8,
            compressed_length: little_endian_value(source.subrange(position + 30, position + 32)) as u16,
            extent_offset: little_endian_value(source.subrange(position + 32, position + 34)) as u16,
            head: PointerHead {
                birth_tree: TreeIdentifier(little_endian_value(source.subrange(position + 34, position + 42)) as u64),
                birth_txg: CheckpointTxg(little_endian_value(source.subrange(position + 42, position + 50)) as u64),
            },
        }
    }

    /// [`Self::judged`] 的判法：按 MAC / nonce、算法类型、压缩算法码、压后长度、extent 偏移的次序，第一处不是第一版取值的那一段报出来。
    spec fn spec_judged(self) -> Result<PointerHead, PointerHeadFieldOutsideTheFirstVersion> {
        if !self.is_every_mac_and_nonce_byte_zero {
            Err(PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero)
        } else if self.algorithm_type != 0 {
            Err(PointerHeadFieldOutsideTheFirstVersion::AlgorithmTypeNotUnencrypted {
                algorithm_type: self.algorithm_type,
            })
        } else if self.compression_algorithm_code != 0 {
            Err(PointerHeadFieldOutsideTheFirstVersion::CompressionAlgorithmCodeNotRecognized {
                compression_algorithm_code: self.compression_algorithm_code,
            })
        } else if self.compressed_length != 0 {
            Err(PointerHeadFieldOutsideTheFirstVersion::CompressedLengthNotZeroWithoutCompression {
                compressed_length: self.compressed_length,
            })
        } else if self.extent_offset != 0 {
            Err(PointerHeadFieldOutsideTheFirstVersion::ExtentOffsetNotZero(
                PointerHeadExtentOffsetNotZero { extent_offset: self.extent_offset },
            ))
        } else {
            Ok(self.head)
        }
    }

    fn read_from(reader: &mut ByteReader<'_>) -> (head_as_read: Self)
        requires
            old(reader).has_left(POINTER_HEAD_BYTES as nat),
        ensures
            final(reader).advanced_by(old(reader), POINTER_HEAD_BYTES as nat),
            head_as_read == Self::spec_decode_at(old(reader).source(), old(reader).spec_position() as int),
    {
        let is_every_mac_and_nonce_byte_zero = every_byte_is_zero(reader.take(16 + 12));
        let algorithm_type = reader.get_u8();
        let compression_algorithm_code = reader.get_u8();
        let compressed_length = reader.get_u16();
        let extent_offset = reader.get_u16();
        Self {
            is_every_mac_and_nonce_byte_zero,
            algorithm_type,
            compression_algorithm_code,
            compressed_length,
            extent_offset,
            head: PointerHead {
                birth_tree: TreeIdentifier(reader.get_u64()),
                birth_txg: CheckpointTxg(reader.get_u64()),
            },
        }
    }

    /// 加密与压缩那几段按第一版的取值判，再判 extent 偏移（[`PointerHeadFieldOutsideTheFirstVersion`] 各成员的出处）。
    fn judged(self) -> (judged: Result<PointerHead, PointerHeadFieldOutsideTheFirstVersion>)
        ensures
            judged == self.spec_judged(),
    {
        if !self.is_every_mac_and_nonce_byte_zero {
            return Err(PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero);
        }
        if self.algorithm_type != 0 {
            return Err(
                PointerHeadFieldOutsideTheFirstVersion::AlgorithmTypeNotUnencrypted {
                    algorithm_type: self.algorithm_type,
                },
            );
        }
        if self.compression_algorithm_code != 0 {
            return Err(
                PointerHeadFieldOutsideTheFirstVersion::CompressionAlgorithmCodeNotRecognized {
                    compression_algorithm_code: self.compression_algorithm_code,
                },
            );
        }
        if self.compressed_length != 0 {
            return Err(
                PointerHeadFieldOutsideTheFirstVersion::CompressedLengthNotZeroWithoutCompression {
                    compressed_length: self.compressed_length,
                },
            );
        }
        if self.extent_offset != 0 {
            return Err(PointerHeadFieldOutsideTheFirstVersion::ExtentOffsetNotZero(
                PointerHeadExtentOffsetNotZero {
                    extent_offset: self.extent_offset,
                },
            ));
        }
        Ok(self.head)
    }
}

impl PointerHead {
    /// 盘上 50 字节：MAC 与 nonce 28 字节 0、算法类型 0、压缩算法码 0、压后长度 0、extent 偏移 0，再接出生树与出生 txg。
    pub open spec fn spec_bytes(self) -> Seq<u8> {
        zero_bytes(28) + little_endian_bytes(0, 1) + little_endian_bytes(0, 1) + little_endian_bytes(0, 2)
            + little_endian_bytes(0, 2) + little_endian_bytes(self.birth_tree.0 as nat, 8)
            + little_endian_bytes(self.birth_txg.0 as nat, 8)
    }

    /// 从 `source` 的 `position` 起读 50 字节解成的头部（加密与压缩那几段不判）。
    pub open spec fn spec_decode_at(source: Seq<u8>, position: int) -> Self {
        PointerHead {
            birth_tree: TreeIdentifier(little_endian_value(source.subrange(position + 34, position + 42)) as u64),
            birth_txg: CheckpointTxg(little_endian_value(source.subrange(position + 42, position + 50)) as u64),
        }
    }

    /// 同 [`Self::spec_decode_at`]，加密、压缩与 extent 偏移那几段按第一版的取值判：按 MAC / nonce、算法类型、压缩算法码、
    /// 压后长度、extent 偏移的次序，第一处不是第一版取值的那一段报出来。
    pub open spec fn spec_read_judging_at(
        source: Seq<u8>,
        position: int,
    ) -> Result<Self, PointerHeadFieldOutsideTheFirstVersion> {
        let algorithm_type = little_endian_value(source.subrange(position + 28, position + 29)) as u8;
        let compression_algorithm_code = little_endian_value(source.subrange(position + 29, position + 30)) as u8;
        let compressed_length = little_endian_value(source.subrange(position + 30, position + 32)) as u16;
        let extent_offset = little_endian_value(source.subrange(position + 32, position + 34)) as u16;
        if source.subrange(position, position + 28) != zero_bytes(28) {
            Err(PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero)
        } else if algorithm_type != 0 {
            Err(PointerHeadFieldOutsideTheFirstVersion::AlgorithmTypeNotUnencrypted { algorithm_type })
        } else if compression_algorithm_code != 0 {
            Err(PointerHeadFieldOutsideTheFirstVersion::CompressionAlgorithmCodeNotRecognized {
                compression_algorithm_code,
            })
        } else if compressed_length != 0 {
            Err(PointerHeadFieldOutsideTheFirstVersion::CompressedLengthNotZeroWithoutCompression {
                compressed_length,
            })
        } else if extent_offset != 0 {
            Err(PointerHeadFieldOutsideTheFirstVersion::ExtentOffsetNotZero(
                PointerHeadExtentOffsetNotZero { extent_offset },
            ))
        } else {
            Ok(Self::spec_decode_at(source, position))
        }
    }

    /// 往返：`position` 起那 50 字节是这个头部写成的，就解回它，判第一版之外的取值时也判得过。
    pub proof fn lemma_decode_at_round_trip(self, source: Seq<u8>, position: int)
        requires
            0 <= position,
            position + 50 <= source.len(),
            source.subrange(position, position + 50) == self.spec_bytes(),
        ensures
            Self::spec_decode_at(source, position) == self,
            Self::spec_read_judging_at(source, position) == Ok::<Self, PointerHeadFieldOutsideTheFirstVersion>(self),
    {
        lemma_little_endian_capacities();
        let mac_and_nonce = zero_bytes(28);
        let algorithm_type = little_endian_bytes(0, 1);
        let compression_algorithm_code = little_endian_bytes(0, 1);
        let compressed_length = little_endian_bytes(0, 2);
        let extent_offset = little_endian_bytes(0, 2);
        let birth_tree = little_endian_bytes(self.birth_tree.0 as nat, 8);
        let birth_txg = little_endian_bytes(self.birth_txg.0 as nat, 8);
        let through_extent_offset = mac_and_nonce + algorithm_type + compression_algorithm_code + compressed_length
            + extent_offset;
        lemma_split_a_concatenated_range(source, position, through_extent_offset + birth_tree, birth_txg);
        lemma_split_a_concatenated_range(source, position, through_extent_offset, birth_tree);
        lemma_split_a_concatenated_range(
            source,
            position,
            mac_and_nonce + algorithm_type + compression_algorithm_code + compressed_length,
            extent_offset,
        );
        lemma_split_a_concatenated_range(
            source,
            position,
            mac_and_nonce + algorithm_type + compression_algorithm_code,
            compressed_length,
        );
        lemma_split_a_concatenated_range(source, position, mac_and_nonce + algorithm_type, compression_algorithm_code);
        lemma_split_a_concatenated_range(source, position, mac_and_nonce, algorithm_type);
    }

    /// # Panics
    /// 写过缓冲末尾（`requires` 排除的那一种）。
    pub fn write_to(&self, writer: &mut ByteWriter)
        requires
            old(writer).has_room_for(POINTER_HEAD_BYTES as nat),
        ensures
            final(writer).is_well_formed(),
            final(writer).written() == old(writer).written() + self.spec_bytes(),
            final(writer).total_bytes() == old(writer).total_bytes(),
    {
        let start = writer.position();
        writer.skip(16 + 12);
        writer.put_u8(0);
        writer.put_u8(0);
        writer.put_u16(0);
        writer.put_u16(0);
        assert!(writer.position() - start == 34, "出生树落在指针头部偏移 34");
        writer.put_u64(self.birth_tree.0);
        writer.put_u64(self.birth_txg.0);
        assert(writer.written() =~= old(writer).written() + self.spec_bytes());
        assert!(
            writer.position() - start == usize::try_from(POINTER_HEAD_BYTES).expect("50"),
            "指针头部写了 50 字节"
        );
    }

    /// 读头部，加密与压缩那几段与 extent 偏移都不判。core 读盘上指针的读者一律不用它（走
    /// [`Self::read_judging_the_encryption_compression_and_extent_offset_fields_from`]；影子账的读者经
    /// [`NodePointer::read_with_head_fields_judgement_from`] 不判）；留给测试与造坏盘的装置读自己刚写出、要改的那几个字节。
    pub fn read_from(reader: &mut ByteReader<'_>) -> (head: Self)
        requires
            old(reader).has_left(POINTER_HEAD_BYTES as nat),
        ensures
            final(reader).advanced_by(old(reader), POINTER_HEAD_BYTES as nat),
            head == Self::spec_decode_at(old(reader).source(), old(reader).spec_position() as int),
    {
        PointerHeadAsRead::read_from(reader).head
    }

    /// 读头部，加密与压缩那几段与 extent 偏移按第一版的取值判。
    ///
    /// # Errors
    /// 那几段有一段不是第一版的取值 ⇒ [`PointerHeadFieldOutsideTheFirstVersion`]。
    pub fn read_judging_the_encryption_compression_and_extent_offset_fields_from(
        reader: &mut ByteReader<'_>,
    ) -> (head: Result<Self, PointerHeadFieldOutsideTheFirstVersion>)
        requires
            old(reader).has_left(POINTER_HEAD_BYTES as nat),
        ensures
            final(reader).advanced_by(old(reader), POINTER_HEAD_BYTES as nat),
            head == Self::spec_read_judging_at(old(reader).source(), old(reader).spec_position() as int),
    {
        PointerHeadAsRead::read_from(reader).judged()
    }
}

/// 指向码 2 / 码 3 的指针 86。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodePointer {
    pub head: PointerHead,
    pub locations: [LocationEntry; 2],
    pub instance: InstanceGeneration,
    pub birth_sequence: BirthSequence,
}

impl NodePointer {
    /// 整条全 0 的指针（[`Self::empty_root`] 交回的那一条）。
    pub open spec fn spec_empty_root() -> Self {
        let zero = LocationEntry { device: DeviceIdentity(0), slot: SlotNumber(0), unit_checksum: 0 };
        NodePointer {
            head: PointerHead { birth_tree: TreeIdentifier(0), birth_txg: CheckpointTxg(0) },
            locations: [zero, zero],
            instance: InstanceGeneration(0),
            birth_sequence: BirthSequence(0),
        }
    }

    /// 两条位置条目的槽号都装得进 6 字节。
    pub open spec fn is_encodable(self) -> bool {
        location_entries_are_encodable(self.locations)
    }

    /// 写者肯写的位置条目次序：按设备身份严格升序，或整条全零（I-2.5 对全零指针整体豁免）。
    pub open spec fn has_location_entries_the_writer_accepts(self) -> bool {
        self == Self::spec_empty_root() || spec_location_entries_ascend_by_device(self.locations)
    }

    /// 盘上 86 字节：头部 50 + 两条位置条目 28 + 实例代号 4 + 出生序号 4。
    pub open spec fn spec_bytes(self) -> Seq<u8> {
        self.head.spec_bytes() + spec_location_entries_bytes(self.locations) + little_endian_bytes(
            self.instance.0 as nat,
            4,
        ) + little_endian_bytes(self.birth_sequence.0 as nat, 4)
    }

    /// 从 `source` 的 `position` 起读 86 字节解成的指针（头部加密与压缩那几段不判）。
    pub open spec fn spec_decode_at(source: Seq<u8>, position: int) -> Self {
        NodePointer {
            head: PointerHead::spec_decode_at(source, position),
            locations: [
                LocationEntry::spec_decode_at(source, position + 50),
                LocationEntry::spec_decode_at(source, position + 64),
            ],
            instance: InstanceGeneration(little_endian_value(source.subrange(position + 78, position + 82)) as u32),
            birth_sequence: BirthSequence(little_endian_value(source.subrange(position + 82, position + 86)) as u32),
        }
    }

    /// 同 [`Self::spec_decode_at`]，头部加密、压缩与 extent 偏移那几段按第一版的取值判。
    pub open spec fn spec_read_judging_at(
        source: Seq<u8>,
        position: int,
    ) -> Result<Self, PointerHeadFieldOutsideTheFirstVersion> {
        match PointerHead::spec_read_judging_at(source, position) {
            Ok(_head) => Ok(Self::spec_decode_at(source, position)),
            Err(refusal) => Err(refusal),
        }
    }

    /// 往返：`position` 起那 86 字节是这条指针写成的，就解回它，判头部第一版之外的取值时也判得过。
    pub proof fn lemma_decode_at_round_trip(self, source: Seq<u8>, position: int)
        requires
            self.is_encodable(),
            0 <= position,
            position + 86 <= source.len(),
            source.subrange(position, position + 86) == self.spec_bytes(),
        ensures
            Self::spec_decode_at(source, position) == self,
            Self::spec_read_judging_at(source, position) == Ok::<Self, PointerHeadFieldOutsideTheFirstVersion>(self),
    {
        lemma_little_endian_capacities();
        let head_bytes = self.head.spec_bytes();
        let locations_bytes = spec_location_entries_bytes(self.locations);
        let instance_bytes = little_endian_bytes(self.instance.0 as nat, 4);
        let birth_sequence_bytes = little_endian_bytes(self.birth_sequence.0 as nat, 4);
        lemma_split_a_concatenated_range(
            source,
            position,
            head_bytes + locations_bytes + instance_bytes,
            birth_sequence_bytes,
        );
        lemma_split_a_concatenated_range(source, position, head_bytes + locations_bytes, instance_bytes);
        lemma_split_a_concatenated_range(source, position, head_bytes, locations_bytes);
        lemma_split_a_concatenated_range(
            source,
            position + 50,
            self.locations[0].spec_bytes(),
            self.locations[1].spec_bytes(),
        );
        self.head.lemma_decode_at_round_trip(source, position);
        self.locations[0].lemma_decode_at_round_trip(source, position + 50);
        self.locations[1].lemma_decode_at_round_trip(source, position + 64);
        assert(Self::spec_decode_at(source, position).locations =~= self.locations);
    }

    /// 没有根的树：整条指针全 0（I-2.5 对全零指针整体豁免升序判定）。
    #[must_use]
    pub const fn empty_root() -> (root: Self)
        ensures
            root == Self::spec_empty_root(),
    {
        let zero = LocationEntry {
            device: DeviceIdentity(0),
            slot: SlotNumber(0),
            unit_checksum: 0,
        };
        Self {
            head: PointerHead {
                birth_tree: TreeIdentifier(0),
                birth_txg: CheckpointTxg(0),
            },
            locations: [zero, zero],
            instance: InstanceGeneration(0),
            birth_sequence: BirthSequence(0),
        }
    }

    /// 这条指针是不是整条全 0（逐字段比，与 `*self == Self::empty_root()` 同一个判法）。
    fn is_the_empty_root(&self) -> (is_empty: bool)
        ensures
            is_empty == (*self == Self::spec_empty_root()),
    {
        let is_empty = self.head.birth_tree.0 == 0 && self.head.birth_txg.0 == 0
            && self.locations[0].device.0 == 0 && self.locations[0].slot.0 == 0
            && self.locations[0].unit_checksum == 0 && self.locations[1].device.0 == 0
            && self.locations[1].slot.0 == 0 && self.locations[1].unit_checksum == 0
            && self.instance.0 == 0 && self.birth_sequence.0 == 0;
        proof {
            if is_empty {
                assert(self.locations =~= Self::spec_empty_root().locations);
            }
        }
        is_empty
    }

    /// 两条位置条目按设备身份严格升序（I-2.5）；整条全零的指针（「注册了但还没有根」）整条豁免。
    ///
    /// # Errors
    /// 不是全零、而两条位置条目相同或逆序 ⇒ [`LocationEntriesNotAscendingByDevice`]。
    pub fn location_entries_ascend_by_device(
        &self,
    ) -> (ascending: Result<(), LocationEntriesNotAscendingByDevice>)
        ensures
            ascending is Ok <==> self.has_location_entries_the_writer_accepts(),
    {
        if self.is_the_empty_root() {
            return Ok(());
        }
        location_entries_ascend_by_device(&self.locations)
    }

    /// # Panics
    /// 写过缓冲末尾、槽号装不进 6 字节，或位置条目不按设备身份严格升序又不是全零指针（`requires` 排除的三种）。
    pub fn write_to(&self, writer: &mut ByteWriter)
        requires
            old(writer).has_room_for(NODE_POINTER_BYTES as nat),
            self.is_encodable(),
            self.has_location_entries_the_writer_accepts(),
        ensures
            final(writer).is_well_formed(),
            final(writer).written() == old(writer).written() + self.spec_bytes(),
            final(writer).total_bytes() == old(writer).total_bytes(),
    {
        let start = writer.position();
        self.head.write_to(writer);
        assert!(
            self.location_entries_ascend_by_device().is_ok(),
            "位置条目按设备身份严格升序（I-2.5），整条全零的空指针（empty_root）豁免：盘上读来、要照抄写回的指针在 recovery::rebuild_version 判过"
        );
        self.locations[0].write_to(writer);
        self.locations[1].write_to(writer);
        writer.put_u32(self.instance.0);
        writer.put_u32(self.birth_sequence.0);
        assert(writer.written() =~= old(writer).written() + self.spec_bytes());
        assert!(
            writer.position() - start == usize::try_from(NODE_POINTER_BYTES).expect("86"),
            "码 2 / 码 3 指针写了 86 字节"
        );
    }

    pub fn read_from(reader: &mut ByteReader<'_>) -> (pointer: Self)
        requires
            old(reader).has_left(NODE_POINTER_BYTES as nat),
        ensures
            final(reader).advanced_by(old(reader), NODE_POINTER_BYTES as nat),
            pointer == Self::spec_decode_at(old(reader).source(), old(reader).spec_position() as int),
            pointer.is_encodable(),
    {
        let head = PointerHead::read_from(reader);
        let pointer = Self::read_after_the_head(head, reader);
        assert(pointer.locations =~= Self::spec_decode_at(
            old(reader).source(),
            old(reader).spec_position() as int,
        ).locations);
        pointer
    }

    /// 同 [`Self::read_from`]，头部里加密、压缩与 extent 偏移那几段按第一版的取值判（[`PointerHead::read_judging_the_encryption_compression_and_extent_offset_fields_from`]）。
    ///
    /// # Errors
    /// 那几段有一段不是第一版的取值 ⇒ [`PointerHeadFieldOutsideTheFirstVersion`]。
    pub fn read_judging_the_encryption_compression_and_extent_offset_fields_from(
        reader: &mut ByteReader<'_>,
    ) -> (pointer: Result<Self, PointerHeadFieldOutsideTheFirstVersion>)
        requires
            old(reader).has_left(NODE_POINTER_BYTES as nat),
        ensures
            pointer == Self::spec_read_judging_at(old(reader).source(), old(reader).spec_position() as int),
            pointer is Ok ==> final(reader).advanced_by(old(reader), NODE_POINTER_BYTES as nat),
            pointer matches Ok(read_pointer) ==> read_pointer.is_encodable(),
    {
        let head =
            PointerHead::read_judging_the_encryption_compression_and_extent_offset_fields_from(
                reader,
            )?;
        let pointer = Self::read_after_the_head(head, reader);
        assert(pointer.locations =~= Self::spec_decode_at(
            old(reader).source(),
            old(reader).spec_position() as int,
        ).locations);
        Ok(pointer)
    }

    /// 读一条子指针，判不判头部第一版之外的取值由读者定（[`PointerHeadFieldsJudgement`]）：走读的读者判，影子账的读者不判。
    ///
    /// # Errors
    /// `judgement` 是判的那一种、而头部有一段不是第一版的取值 ⇒ [`PointerHeadFieldOutsideTheFirstVersion`]；不判的那一种恒 `Ok`。
    pub fn read_with_head_fields_judgement_from(
        reader: &mut ByteReader<'_>,
        judgement: PointerHeadFieldsJudgement,
    ) -> (pointer: Result<Self, PointerHeadFieldOutsideTheFirstVersion>)
        requires
            old(reader).has_left(NODE_POINTER_BYTES as nat),
        ensures
            pointer == match judgement {
                PointerHeadFieldsJudgement::JudgedAsTheStructureIsCorruptOutsideTheFirstVersion => Self::spec_read_judging_at(
                    old(reader).source(),
                    old(reader).spec_position() as int,
                ),
                PointerHeadFieldsJudgement::NotJudgedByTheShadowLedger => Ok(
                    Self::spec_decode_at(old(reader).source(), old(reader).spec_position() as int),
                ),
            },
            pointer is Ok ==> final(reader).advanced_by(old(reader), NODE_POINTER_BYTES as nat),
            pointer matches Ok(read_pointer) ==> read_pointer.is_encodable(),
    {
        match judgement {
            PointerHeadFieldsJudgement::JudgedAsTheStructureIsCorruptOutsideTheFirstVersion => {
                Self::read_judging_the_encryption_compression_and_extent_offset_fields_from(reader)
            }
            PointerHeadFieldsJudgement::NotJudgedByTheShadowLedger => Ok(Self::read_from(reader)),
        }
    }

    fn read_after_the_head(head: PointerHead, reader: &mut ByteReader<'_>) -> (pointer: Self)
        requires
            old(reader).has_left((NODE_POINTER_BYTES - POINTER_HEAD_BYTES) as nat),
        ensures
            final(reader).advanced_by(old(reader), (NODE_POINTER_BYTES - POINTER_HEAD_BYTES) as nat),
            pointer.head == head,
            pointer.locations[0] == LocationEntry::spec_decode_at(old(reader).source(), old(reader).spec_position() as int),
            pointer.locations[1] == LocationEntry::spec_decode_at(old(reader).source(), (old(reader).spec_position() + 14) as int),
            pointer.instance.0 == little_endian_value(old(reader).source().subrange(
                (old(reader).spec_position() + 28) as int,
                (old(reader).spec_position() + 32) as int,
            )),
            pointer.birth_sequence.0 == little_endian_value(old(reader).source().subrange(
                (old(reader).spec_position() + 32) as int,
                (old(reader).spec_position() + 36) as int,
            )),
            pointer.is_encodable(),
    {
        let locations = [
            LocationEntry::read_from(reader),
            LocationEntry::read_from(reader),
        ];
        Self {
            head,
            locations,
            instance: InstanceGeneration(reader.get_u32()),
            birth_sequence: BirthSequence(reader.get_u32()),
        }
    }
}

/// 指向码 1 数据单元的指针 88：头部 50 + 位置条目 14 × 2 + 写序 10（D19（块指针的结构与宽度预算） 已定项 8）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataPointer {
    pub head: PointerHead,
    pub locations: [LocationEntry; 2],
    pub write_order: WriteOrder,
}

impl DataPointer {
    /// 两条位置条目的槽号与写序的事务号都装得进 6 字节。
    pub open spec fn is_encodable(self) -> bool {
        location_entries_are_encodable(self.locations) && self.write_order.is_encodable()
    }

    /// 盘上 88 字节：头部 50 + 两条位置条目 28 + 写序 10。
    pub open spec fn spec_bytes(self) -> Seq<u8> {
        self.head.spec_bytes() + spec_location_entries_bytes(self.locations) + self.write_order.spec_bytes()
    }

    /// 从 `source` 的 `position` 起读 88 字节解成的指针（头部加密与压缩那几段不判）。
    pub open spec fn spec_decode_at(source: Seq<u8>, position: int) -> Self {
        DataPointer {
            head: PointerHead::spec_decode_at(source, position),
            locations: [
                LocationEntry::spec_decode_at(source, position + 50),
                LocationEntry::spec_decode_at(source, position + 64),
            ],
            write_order: WriteOrder::spec_decode_at(source, position + 78),
        }
    }

    /// 同 [`Self::spec_decode_at`]，头部加密、压缩与 extent 偏移那几段按第一版的取值判。
    pub open spec fn spec_read_judging_at(
        source: Seq<u8>,
        position: int,
    ) -> Result<Self, PointerHeadFieldOutsideTheFirstVersion> {
        match PointerHead::spec_read_judging_at(source, position) {
            Ok(_head) => Ok(Self::spec_decode_at(source, position)),
            Err(refusal) => Err(refusal),
        }
    }

    /// 往返：`position` 起那 88 字节是这条指针写成的，就解回它，判头部第一版之外的取值时也判得过。
    pub proof fn lemma_decode_at_round_trip(self, source: Seq<u8>, position: int)
        requires
            self.is_encodable(),
            0 <= position,
            position + 88 <= source.len(),
            source.subrange(position, position + 88) == self.spec_bytes(),
        ensures
            Self::spec_decode_at(source, position) == self,
            Self::spec_read_judging_at(source, position) == Ok::<Self, PointerHeadFieldOutsideTheFirstVersion>(self),
    {
        let head_bytes = self.head.spec_bytes();
        let locations_bytes = spec_location_entries_bytes(self.locations);
        lemma_split_a_concatenated_range(source, position, head_bytes + locations_bytes, self.write_order.spec_bytes());
        lemma_split_a_concatenated_range(source, position, head_bytes, locations_bytes);
        lemma_split_a_concatenated_range(
            source,
            position + 50,
            self.locations[0].spec_bytes(),
            self.locations[1].spec_bytes(),
        );
        self.head.lemma_decode_at_round_trip(source, position);
        self.locations[0].lemma_decode_at_round_trip(source, position + 50);
        self.locations[1].lemma_decode_at_round_trip(source, position + 64);
        self.write_order.lemma_decode_at_round_trip(source, position + 78);
        assert(Self::spec_decode_at(source, position).locations =~= self.locations);
    }

    /// # Panics
    /// 写过缓冲末尾、槽号或事务号装不进 6 字节，或位置条目不按设备身份严格升序（`requires` 排除的三种；数据指针没有全零豁免）。
    pub fn write_to(&self, writer: &mut ByteWriter)
        requires
            old(writer).has_room_for(DATA_POINTER_BYTES as nat),
            self.is_encodable(),
            spec_location_entries_ascend_by_device(self.locations),
        ensures
            final(writer).is_well_formed(),
            final(writer).written() == old(writer).written() + self.spec_bytes(),
            final(writer).total_bytes() == old(writer).total_bytes(),
    {
        let start = writer.position();
        self.head.write_to(writer);
        assert!(
            location_entries_ascend_by_device(&self.locations).is_ok(),
            "数据指针的位置条目按设备身份严格升序（I-2.5），没有全零豁免：盘上读来、要照抄写回的指针在 recovery::rebuild_version 判过"
        );
        self.locations[0].write_to(writer);
        self.locations[1].write_to(writer);
        writer.put_u32(self.write_order.instance.0);
        writer.put_six_byte_unsigned(self.write_order.transaction);
        assert(writer.written() =~= old(writer).written() + self.spec_bytes());
        assert!(
            writer.position() - start == usize::try_from(DATA_POINTER_BYTES).expect("88"),
            "数据指针写了 88 字节"
        );
    }

    pub fn read_from(reader: &mut ByteReader<'_>) -> (pointer: Self)
        requires
            old(reader).has_left(DATA_POINTER_BYTES as nat),
        ensures
            final(reader).advanced_by(old(reader), DATA_POINTER_BYTES as nat),
            pointer == Self::spec_decode_at(old(reader).source(), old(reader).spec_position() as int),
            pointer.is_encodable(),
    {
        let head = PointerHead::read_from(reader);
        let pointer = Self::read_after_the_head(head, reader);
        assert(pointer.locations =~= Self::spec_decode_at(
            old(reader).source(),
            old(reader).spec_position() as int,
        ).locations);
        pointer
    }

    /// 同 [`Self::read_from`]，头部里加密、压缩与 extent 偏移那几段按第一版的取值判（[`PointerHead::read_judging_the_encryption_compression_and_extent_offset_fields_from`]）。
    ///
    /// # Errors
    /// 那几段有一段不是第一版的取值 ⇒ [`PointerHeadFieldOutsideTheFirstVersion`]。
    pub fn read_judging_the_encryption_compression_and_extent_offset_fields_from(
        reader: &mut ByteReader<'_>,
    ) -> (pointer: Result<Self, PointerHeadFieldOutsideTheFirstVersion>)
        requires
            old(reader).has_left(DATA_POINTER_BYTES as nat),
        ensures
            pointer == Self::spec_read_judging_at(old(reader).source(), old(reader).spec_position() as int),
            pointer is Ok ==> final(reader).advanced_by(old(reader), DATA_POINTER_BYTES as nat),
            pointer matches Ok(read_pointer) ==> read_pointer.is_encodable(),
    {
        let head =
            PointerHead::read_judging_the_encryption_compression_and_extent_offset_fields_from(
                reader,
            )?;
        let pointer = Self::read_after_the_head(head, reader);
        assert(pointer.locations =~= Self::spec_decode_at(
            old(reader).source(),
            old(reader).spec_position() as int,
        ).locations);
        Ok(pointer)
    }

    fn read_after_the_head(head: PointerHead, reader: &mut ByteReader<'_>) -> (pointer: Self)
        requires
            old(reader).has_left((DATA_POINTER_BYTES - POINTER_HEAD_BYTES) as nat),
        ensures
            final(reader).advanced_by(old(reader), (DATA_POINTER_BYTES - POINTER_HEAD_BYTES) as nat),
            pointer.head == head,
            pointer.locations[0] == LocationEntry::spec_decode_at(old(reader).source(), old(reader).spec_position() as int),
            pointer.locations[1] == LocationEntry::spec_decode_at(old(reader).source(), (old(reader).spec_position() + 14) as int),
            pointer.write_order == WriteOrder::spec_decode_at(old(reader).source(), (old(reader).spec_position() + 28) as int),
            pointer.is_encodable(),
    {
        let locations = [
            LocationEntry::read_from(reader),
            LocationEntry::read_from(reader),
        ];
        let instance = InstanceGeneration(reader.get_u32());
        let transaction = reader.get_six_byte_unsigned();
        Self {
            head,
            locations,
            write_order: WriteOrder {
                instance,
                transaction,
            },
        }
    }
}

} // verus!

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_pointer_is_86_bytes_and_round_trips() {
        let pointer = NodePointer {
            head: PointerHead {
                birth_tree: TreeIdentifier(0),
                birth_txg: CheckpointTxg(0),
            },
            locations: [
                LocationEntry {
                    device: DeviceIdentity(0),
                    slot: SlotNumber(50178),
                    unit_checksum: 0xAB,
                },
                LocationEntry {
                    device: DeviceIdentity(1),
                    slot: SlotNumber(50178),
                    unit_checksum: 0xAB,
                },
            ],
            instance: InstanceGeneration(0),
            birth_sequence: BirthSequence(1),
        };
        let mut writer = ByteWriter::new(86);
        pointer.write_to(&mut writer);
        let bytes = writer.into_bytes();
        assert_eq!(bytes[34], 0, "出生树 0");
        assert_eq!(
            &bytes[50..54],
            &0u32.to_le_bytes(),
            "第一条位置条目的设备身份"
        );
        assert_eq!(
            &bytes[54..60],
            &50178u64.to_le_bytes()[..6],
            "槽号 6 字节小端"
        );
        assert_eq!(
            NodePointer::read_from(&mut ByteReader::at(&bytes, 0)),
            pointer
        );
        assert_eq!(NodePointer::empty_root().locations[1].slot, SlotNumber(0));
    }

    /// 两条位置条目都在盘 1 的一条非空码 2 / 码 3 指针：两条设备身份相同，不是全零的空指针，写出时断言（I-2.5 严格升序）。
    #[test]
    #[should_panic(
        expected = "位置条目按设备身份严格升序（I-2.5），整条全零的空指针（empty_root）豁免"
    )]
    fn a_non_empty_node_pointer_with_both_location_entries_on_one_device_is_refused_by_the_writer()
    {
        let on_device_one = LocationEntry {
            device: DeviceIdentity(1),
            slot: SlotNumber(50178),
            unit_checksum: 0xAB,
        };
        let pointer = NodePointer {
            head: PointerHead {
                birth_tree: TreeIdentifier(3),
                birth_txg: CheckpointTxg(7),
            },
            locations: [on_device_one, on_device_one],
            instance: InstanceGeneration(1),
            birth_sequence: BirthSequence(1),
        };
        let mut writer = ByteWriter::new(86);
        pointer.write_to(&mut writer);
    }

    /// 整条全零的空指针（`empty_root`，mkfs 写进映射树根与分配记录树根的那一形）两条位置条目都是盘 0：照写，写出来 86 字节全 0。
    #[test]
    fn the_all_zero_empty_root_is_written_although_both_location_entries_name_device_zero() {
        let mut writer = ByteWriter::new(86);
        NodePointer::empty_root().write_to(&mut writer);
        assert!(
            writer.into_bytes().iter().all(|byte| *byte == 0),
            "空指针写出来整条全 0"
        );
    }

    /// 两条位置条目都在盘 0 的数据指针：数据指针没有全零豁免，写出时断言。
    #[test]
    #[should_panic(expected = "数据指针的位置条目按设备身份严格升序（I-2.5），没有全零豁免")]
    fn a_data_pointer_with_both_location_entries_on_one_device_is_refused_by_the_writer() {
        let on_device_zero = LocationEntry {
            device: DeviceIdentity(0),
            slot: SlotNumber(50200),
            unit_checksum: 0xCD,
        };
        let pointer = DataPointer {
            head: PointerHead {
                birth_tree: TreeIdentifier(3),
                birth_txg: CheckpointTxg(7),
            },
            locations: [on_device_zero, on_device_zero],
            write_order: WriteOrder {
                instance: InstanceGeneration(1),
                transaction: 4,
            },
        };
        let mut writer = ByteWriter::new(88);
        pointer.write_to(&mut writer);
    }
}
