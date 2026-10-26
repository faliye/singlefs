//! 码 1 全序键（I-1.8（归并后版本全序） ②，C464（I-1.8 对码 1 的全序键写窄了） 用户 2026-10-26 定「① 加诞生代号打头」）的两张镜像，
//! harness 档的池级 checker 用例与 checker 档的 GPU 对拍各用一份（`crates/singlefs-harness/tests/merged_version_total_order_of_data_units_across_two_checkpoints.rs`、
//! `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_data_unit_total_order_keys.rs`）。
//!
//! 底子：新池新建文件（txg 3，对象出生代 3）之后实例 1 覆盖写一版（txg 4）：遍历方向那一版数据单元诞生代号 4、对象出生代 3。
//! 照抄那一版的两份副本（两盘同槽），放到两盘写过的最高槽之后一处没写过的同一个槽，重封载荷 CRC 与头校验和：
//! - 「一个事务跨两个 checkpoint」那一张：诞生代号改成 3（上一个 checkpoint 写下的那一版）、写序不动、载荷改一个字节——同一个对象 key，
//!   两组写序逐字节相同、诞生代号不同，两版都是合法的已发布版本（欠账 C464 那一行「怎么拦」的镜像）。
//! - 「真重号」那一张：诞生代号、写序都不动、载荷改一个字节——对象 key、诞生代号、写序三样都相同，归并键与遍历方向那一版相同、并成一组，
//!   载荷不同。码 1 的归并键是对象 key、诞生代号、fsid、写序四段，扫描项的 fsid 都是本池的，三样相同而归并键不同的两组造不出来：
//!   真重号落在 ① 那一半（同一组的成员载荷相同），不在 ②。
//!
//! 用这一份的测试文件同时声明 `mod common;`。
#![allow(
    dead_code,
    reason = "harness 档用例与 checker 档对拍各自只用到其中一部分"
)]

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::transaction::TransactionOutput;
use singlefs_core::unit::seal_header_checksum;
use singlefs_format::{DATA_UNIT_BYTES, SLOT_BYTES};
use singlefs_harness::memory_pool::{MemoryPool, SECTOR_BYTES};

use crate::common::{build_pool, overwrite_in_process_as_instance_one_and_advance_the_pool};

/// 码 1 头里各段的偏移（D18（块里携带什么信息） 已定项 7 的字段表；池级 checker `ContentUnitClass::header_offsets` 同一份）：
/// 诞生代号 75（8 字节）、写序 91（实例代号 4 + 事务号 6）、载荷 CRC 101，头到 105 为止。
pub const DATA_UNIT_BIRTH_TXG_OFFSET: usize = 75;
pub const DATA_UNIT_WRITE_ORDER_OFFSET: usize = 91;
pub const DATA_UNIT_WRITE_ORDER_BYTES: usize = 10;
const DATA_UNIT_PAYLOAD_CRC_OFFSET: usize = 101;
const DATA_UNIT_HEADER_BYTES: usize = 105;

/// 照抄的那一版改的那一个载荷字节：头之后第一个字节（另一版内容）。
const PAYLOAD_BYTE_CHANGED_IN_THE_COPY: usize = DATA_UNIT_HEADER_BYTES;

/// 照抄的那一版放在两盘写过的最高槽之后这么多个槽（离开现有的单元，仍在盘里）。
const SLOTS_PAST_THE_HIGHEST_WRITTEN_SLOT: u64 = 64;

/// 覆盖写那一版的内容：与第一个文件不同、装得进一个数据单元。
pub const CONTENT_OF_THE_OVERWRITE: &[u8] =
    b"the first file overwritten once, so that its data unit is born after its object";

/// 照抄的那一版的诞生代号怎么取。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BirthTxgOfTheCopy {
    /// 比遍历方向那一版早一个 checkpoint：同一个事务在上一个 checkpoint 里写下的那一版（写序不动）。
    OneCheckpointEarlier,
    /// 与遍历方向那一版相同：对象 key、诞生代号、写序三样都相同的真重号。
    SameAsTheWalkedVersion,
}

/// 两张镜像之一，连同搭建自证要的几样。
pub struct ImageWithASecondVersionOfTheDataUnit {
    /// 照抄之前的底子（覆盖写之后）。
    pub base: MemoryPool,
    /// 照抄的那一版放进去之后。
    pub planted: MemoryPool,
    /// 遍历方向那一版数据单元的整单元字节（盘 0 那一份）。
    pub walked_version: Vec<u8>,
    /// 照抄的那一版（重封之后）。
    pub copied_version: Vec<u8>,
    /// 照抄的那一版落在两盘的哪个槽。
    pub copied_slot: u64,
}

fn data_unit_bytes() -> usize {
    usize::try_from(DATA_UNIT_BYTES).expect("码 1 数据单元 32768 字节")
}

/// 按码 1 重封：先算载荷 CRC，再算头校验和。
fn reseal_data_unit(unit: &mut [u8]) {
    let payload_crc = crc32_castagnoli(&unit[DATA_UNIT_HEADER_BYTES..]);
    unit[DATA_UNIT_PAYLOAD_CRC_OFFSET..DATA_UNIT_PAYLOAD_CRC_OFFSET + 4]
        .copy_from_slice(&payload_crc.to_le_bytes());
    seal_header_checksum(unit, DATA_UNIT_HEADER_BYTES);
}

/// 一块盘写过的最高槽。
fn highest_written_slot(image: &MemoryPool, device: DeviceIdentity) -> u64 {
    image
        .devices
        .get(&device)
        .expect("池里有这块盘")
        .written_sectors()
        .keys()
        .map(|sector| sector * SECTOR_BYTES / SLOT_BYTES)
        .max()
        .expect("这块盘写过")
}

/// 一个码 1 头里的诞生代号。
#[must_use]
pub fn birth_txg_of(unit: &[u8]) -> u64 {
    u64::from_le_bytes(
        unit[DATA_UNIT_BIRTH_TXG_OFFSET..DATA_UNIT_BIRTH_TXG_OFFSET + 8]
            .try_into()
            .expect("诞生代号 8 字节"),
    )
}

/// 一个码 1 头里的写序那 10 字节。
#[must_use]
pub fn write_order_of(unit: &[u8]) -> Vec<u8> {
    unit[DATA_UNIT_WRITE_ORDER_OFFSET..DATA_UNIT_WRITE_ORDER_OFFSET + DATA_UNIT_WRITE_ORDER_BYTES]
        .to_vec()
}

/// 新池新建文件、实例 1 覆盖写一版之后的池上，把遍历方向那一版数据单元照抄一份（诞生代号按 `birth_txg_of_the_copy`、载荷改一个字节），
/// 两盘同槽各放一份、重封。
#[must_use]
pub fn image_with_a_second_version_of_the_data_unit(
    tag: &str,
    birth_txg_of_the_copy: BirthTxgOfTheCopy,
) -> ImageWithASecondVersionOfTheDataUnit {
    let mut pool = build_pool(tag);
    let overwrite: TransactionOutput = overwrite_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        CONTENT_OF_THE_OVERWRITE,
    );
    let base = pool.memory_pool();
    let [pointer] = <[_; 1]>::try_from(overwrite.data_pointers.clone())
        .expect("覆盖写那一版装得进一个数据单元，一条数据指针");
    let walked_location = pointer.locations[0];
    let walked_version = base
        .devices
        .get(&walked_location.device)
        .expect("池里有那块盘")
        .read(walked_location.slot.to_device_offset(), data_unit_bytes());
    let mut copied_version = walked_version.clone();
    match birth_txg_of_the_copy {
        BirthTxgOfTheCopy::OneCheckpointEarlier => {
            let earlier_birth_txg = birth_txg_of(&walked_version)
                .checked_sub(1)
                .expect("覆盖写那一版诞生在 txg 4，早一个 checkpoint 是 3");
            copied_version[DATA_UNIT_BIRTH_TXG_OFFSET..DATA_UNIT_BIRTH_TXG_OFFSET + 8]
                .copy_from_slice(&earlier_birth_txg.to_le_bytes());
        }
        BirthTxgOfTheCopy::SameAsTheWalkedVersion => {}
    }
    copied_version[PAYLOAD_BYTE_CHANGED_IN_THE_COPY] ^= 0xff;
    reseal_data_unit(&mut copied_version);
    let copied_slot = pointer
        .locations
        .iter()
        .map(|location| highest_written_slot(&base, location.device))
        .max()
        .expect("两条位置条目")
        + SLOTS_PAST_THE_HIGHEST_WRITTEN_SLOT;
    let mut planted = base.clone();
    for location in pointer.locations {
        planted
            .devices
            .get_mut(&location.device)
            .expect("池里有那块盘")
            .write(
                DeviceOffsetInBytes(copied_slot * SLOT_BYTES),
                &copied_version,
            );
    }
    ImageWithASecondVersionOfTheDataUnit {
        base,
        planted,
        walked_version,
        copied_version,
        copied_slot,
    }
}
