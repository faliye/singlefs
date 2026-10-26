//! C163（固定点各轮写序相同使平局破除落空）：I-1.8（归并后版本全序） 给码 3 的全序键是 (诞生代号, 实例代号, 事务号)，
//! 事务号留在末位做「同一 checkpoint 内的平局破除」；可 D18（块里携带什么信息） 已定项 7 的取号规则让同一 checkpoint 固定点的各轮取到同一个事务号
//! （D23（journal 的角色与格式） 已定项 7：事务号按实例计数、与 jsn 是两个计数，固定点迭代不产生新事务）——同一 checkpoint 里同一容器写出两版，
//! 两版的写序逐字节相同，全序键分不开它们。输入端由 D19（块指针的结构与宽度预算） 已定项 12（固定点里同一节点只写一次）与已定项 9（重写换出生序号）
//! 定案解掉：这样两版合法盘面上写不出来，池级 checker 见到就判 I-1.8 违例。
//!
//! 镜像的造法：新池新建文件之后的池上取一个码 3 容器（inode 叶容器），照抄一份放到单元区里一处没写过的槽：出生序号加一（重写换出生序号，
//! 归并键不同、各成一组）、载荷改一个字节（第二轮写出的另一版内容），重封——同一身份、同一 (诞生代号, 实例代号, 事务号) 的两版。
//! 对照那一份另把事务号加一（固定点各轮按轮序取号的样子）：全序键分得开，I-1.8 成立。
mod common;
mod common_tree_split;

use common::{build_pool, PoolImageGeometry};
use common_tree_split::reseal_unit_by_its_class;
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::unit::{UNIT_CLASS_PACKED, UNIT_MAGIC};
use singlefs_format::{DATA_UNIT_BYTES, SLOT_BYTES};
use singlefs_harness::memory_pool::{MemoryPool, SECTOR_BYTES};

/// 码 3 头里各段的偏移（D18（块里携带什么信息） 已定项 11 的字段表；池级 checker `ContentUnitClass::header_offsets` 同一份）：
/// 类标签 6，写序 93（实例代号 4 + 事务号 6），出生序号 103，头到 107 为止。
const UNIT_CLASS_TAG_OFFSET: usize = 6;
const PACKED_UNIT_TRANSACTION_OFFSET: usize = 97;
const PACKED_UNIT_TRANSACTION_BYTES: usize = 6;
const PACKED_UNIT_BIRTH_SEQUENCE_OFFSET: usize = 103;
const PACKED_UNIT_HEADER_BYTES: usize = 107;

/// 第二版改的那一个载荷字节：头之后第一个字节。
const PAYLOAD_BYTE_CHANGED_IN_THE_SECOND_VERSION: usize = PACKED_UNIT_HEADER_BYTES;

/// 照抄的那一份放在单元区里写过的最高槽之后这么多个槽（离开现有的单元，仍在盘里）。
const SLOTS_PAST_THE_HIGHEST_WRITTEN_SLOT: u64 = 64;

/// 第二版的写序怎么取。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WriteOrderOfTheSecondVersion {
    /// 现行取号规则：同一 checkpoint 固定点的各轮共享同一个事务号。
    SameTransactionAsTheFirstRound,
    /// 固定点各轮按轮序取号：第二轮的事务号比第一轮大一。
    TransactionOfTheNextRound,
}

fn packed_unit_bytes() -> usize {
    usize::try_from(DATA_UNIT_BYTES).expect("码 3 打包记录单元 32768 字节")
}

/// 盘 0 单元区里写过的第一个码 3 单元的槽号。
fn first_packed_unit_slot_on_device_zero(image: &MemoryPool) -> u64 {
    let unit_area_start = PoolImageGeometry::of_four_gibibyte_devices_under_the_common_parameters()
        .unit_area_start_slot()
        .0;
    let device_zero = image.devices.get(&DeviceIdentity(0)).expect("池里有盘 0");
    let written_slots: std::collections::BTreeSet<u64> = device_zero
        .written_sectors()
        .keys()
        .map(|sector| sector * SECTOR_BYTES / SLOT_BYTES)
        .filter(|slot| *slot >= unit_area_start)
        .collect();
    written_slots
        .into_iter()
        .find(|slot| {
            // 读按整扇区：头落在单元的第一个扇区里。
            let header = device_zero.read(
                DeviceOffsetInBytes(slot * SLOT_BYTES),
                usize::try_from(SECTOR_BYTES).expect("512"),
            );
            header[..4] == UNIT_MAGIC && header[UNIT_CLASS_TAG_OFFSET] == UNIT_CLASS_PACKED
        })
        .expect("新池新建文件之后单元区里有码 3 容器（inode 叶容器）")
}

/// 盘 0 单元区里写过的最高槽。
fn highest_written_slot_on_device_zero(image: &MemoryPool) -> u64 {
    image
        .devices
        .get(&DeviceIdentity(0))
        .expect("池里有盘 0")
        .written_sectors()
        .keys()
        .map(|sector| sector * SECTOR_BYTES / SLOT_BYTES)
        .max()
        .expect("盘 0 写过")
}

/// 新池新建文件之后的池上，把盘 0 第一个码 3 容器照抄一份当同一 checkpoint 里第二轮写出的那一版（出生序号加一、载荷改一个字节、写序按 `write_order`），
/// 放到写过的最高槽之后、重封。
fn image_with_a_second_version_of_one_container(
    tag: &str,
    write_order: WriteOrderOfTheSecondVersion,
) -> (MemoryPool, MemoryPool) {
    let pool = build_pool(tag);
    let image = pool.memory_pool();
    let source_slot = first_packed_unit_slot_on_device_zero(&image);
    let mut second_version = image
        .devices
        .get(&DeviceIdentity(0))
        .expect("池里有盘 0")
        .read(
            DeviceOffsetInBytes(source_slot * SLOT_BYTES),
            packed_unit_bytes(),
        );
    let birth_sequence = u32::from_le_bytes(
        second_version[PACKED_UNIT_BIRTH_SEQUENCE_OFFSET..PACKED_UNIT_BIRTH_SEQUENCE_OFFSET + 4]
            .try_into()
            .expect("出生序号 4 字节"),
    );
    second_version[PACKED_UNIT_BIRTH_SEQUENCE_OFFSET..PACKED_UNIT_BIRTH_SEQUENCE_OFFSET + 4]
        .copy_from_slice(&(birth_sequence + 1).to_le_bytes());
    second_version[PAYLOAD_BYTE_CHANGED_IN_THE_SECOND_VERSION] ^= 0xff;
    match write_order {
        WriteOrderOfTheSecondVersion::SameTransactionAsTheFirstRound => {}
        WriteOrderOfTheSecondVersion::TransactionOfTheNextRound => {
            let transaction_range = PACKED_UNIT_TRANSACTION_OFFSET
                ..PACKED_UNIT_TRANSACTION_OFFSET + PACKED_UNIT_TRANSACTION_BYTES;
            let mut transaction_bytes = [0u8; 8];
            transaction_bytes[..PACKED_UNIT_TRANSACTION_BYTES]
                .copy_from_slice(&second_version[transaction_range.clone()]);
            let next_round_transaction = u64::from_le_bytes(transaction_bytes) + 1;
            second_version[transaction_range].copy_from_slice(
                &next_round_transaction.to_le_bytes()[..PACKED_UNIT_TRANSACTION_BYTES],
            );
        }
    }
    reseal_unit_by_its_class(&mut second_version);
    let target_slot =
        highest_written_slot_on_device_zero(&image) + SLOTS_PAST_THE_HIGHEST_WRITTEN_SLOT;
    let mut planted = image.clone();
    planted
        .devices
        .get_mut(&DeviceIdentity(0))
        .expect("池里有盘 0")
        .write(
            DeviceOffsetInBytes(target_slot * SLOT_BYTES),
            &second_version,
        );
    (image, planted)
}

fn verdict_of_the_merged_version_total_order(image: &MemoryPool) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(invariant, _)| *invariant == "I-1.8")
        .expect("I-1.8 在池级 checker 的清单里")
        .1
}

/// 现行取号规则下同一 checkpoint 固定点两轮各写出一版同身份码 3 容器：两版写序逐字节相同，全序键 (诞生代号, 实例代号, 事务号) 分不开它们，
/// 池级 checker 判 I-1.8 第二句（同 key 的各组两两全序键不等）红。
/// 判别力：第二句恒判成立（`crates/mutations.tsv` 里「C163」那一行）时这一条红。
#[test]
fn two_versions_of_one_container_written_in_one_checkpoint_under_the_shared_transaction_number_redden_the_total_order(
) {
    let (image, planted) = image_with_a_second_version_of_one_container(
        "c163-shared-transaction",
        WriteOrderOfTheSecondVersion::SameTransactionAsTheFirstRound,
    );
    assert_eq!(
        verdict_of_the_merged_version_total_order(&image),
        InvariantVerdict::Holds,
        "对照：照抄之前 I-1.8 成立"
    );
    match verdict_of_the_merged_version_total_order(&planted) {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains("两两全序键不等这一条不成立"),
            "红在第二句（同 key 的两组全序键相同），不是第一句：{detail}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => panic!(
            "同一 checkpoint 两版同身份码 3 容器共享事务号，全序键分不开，I-1.8 该判违例；实际 {other:?}"
        ),
    }
}

/// 对照：第二轮按轮序取号（事务号加一）时全序键分得开两版，I-1.8 成立——平局破除只在事务号随轮次变时有判别力。
#[test]
fn two_versions_of_one_container_whose_rounds_take_their_own_transaction_numbers_keep_the_total_order(
) {
    let (_image, planted) = image_with_a_second_version_of_one_container(
        "c163-transaction-of-the-next-round",
        WriteOrderOfTheSecondVersion::TransactionOfTheNextRound,
    );
    assert_eq!(
        verdict_of_the_merged_version_total_order(&planted),
        InvariantVerdict::Holds,
        "事务号随轮次变：两版全序键不同，I-1.8 成立"
    );
}
