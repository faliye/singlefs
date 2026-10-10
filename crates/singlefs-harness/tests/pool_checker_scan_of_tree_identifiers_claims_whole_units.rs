//! 池级 checker 的 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 扫描方向按认领规则跳单元（D18（块里携带什么信息） 已定项 7，
//! 用户 2026-10-03 定按整单元跳：码 1、码 3 两槽，码 2 一槽；判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md`
//! 第二节 S2 第一行，改法 F4）。合法历史 mkfs → 可写挂载 → 第一个文件，文件内容由用户给：内容里恰在槽边界上带一个头校验和自洽的码 2 伪头，
//! 它落在那个数据单元的第二槽，被数据单元认领，不数进「出现过的」树 ID。四形照攻方原型
//! `research/prompts/m3-sweep-and-tree-id-r1-opus-model/sweep-proto/src/bin/fake_header_tree_id.rs` 的 `high`、`foreign`、`control`、`inode`
//! 四臂另写一份（D13（验证路线） 已定项 5：用例不与原型共用代码）；一份坏镜像：没被任何单元认领的槽上一个真孤儿节点，树 ID 高于水位，照旧红。

use std::collections::BTreeMap;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::mount_writable;
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{publish_first_file, FirstFile, PoolWriter};
use singlefs_core::unit::{seal_header_checksum, UNIT_CLASS_INDEX_NODE, UNIT_MAGIC};
use singlefs_format::{
    DATA_UNIT_PAYLOAD_OFFSET, JOURNAL_RING_DEFAULT_BYTES, SLOT_BYTES, UNIT_AREA_START_SLOT,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

/// 这几份池的 fsid。
const FILESYSTEM_IDENTIFIER_OF_THE_CLAIM_POOLS: [u8; 16] = *b"CLAIM-WHOLE-UNIT";
/// 伪头改成的「别的池」的 fsid 低 8 字节（攻方 `foreign` 臂那个值）。
const FOREIGN_FILESYSTEM_IDENTIFIER_LOW: u64 = 0x0123_4567_89ab_cdef;
/// 每块盘 4 GiB 内存稀疏盘。
const DEVICE_BYTES_OF_THE_CLAIM_POOLS: u64 = 4 << 30;
/// 每区槽数 S = 4。
const SLOTS_PER_REGION_OF_THE_CLAIM_POOLS: u64 = 4;
/// 写入时间。
const WRITE_TIME_SECONDS_OF_THE_CLAIM_POOLS: u64 = 1_788_000_000;
/// 第一个文件的长度：一个数据单元，内容跨过它的第二槽起点（单元偏移 16384）。
const FILE_BYTES_REACHING_THE_SECOND_SLOT: usize = 20_000;
/// 伪头里的树 ID：2⁴⁰，远高于这几份池的水位（mkfs 水位 65536 起，第一个文件发到 65548）。
const FORGED_TREE_IDENTIFIER: u64 = 1 << 40;
/// 伪头里的诞生代号：1，不超过根环里最大的 checkpoint_txg（扫描方向第一道排除挡不住它）。
const FORGED_BIRTH_TXG: u64 = 1;
/// 码 2 头：树 ID 在 42、key 宽在 51；诞生代号 52 + 2k、fsid 60 + 2k、写序实例 68 + 2k、明文头末尾 86 + 2k
/// （D18（块里携带什么信息） 已定项 18 的偏移表）。
const INDEX_NODE_TREE_IDENTIFIER_OFFSET: usize = 42;
const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;
const INDEX_NODE_BIRTH_TXG_OFFSET_BEFORE_THE_KEYS: usize = 52;
const INDEX_NODE_FILESYSTEM_IDENTIFIER_OFFSET_BEFORE_THE_KEYS: usize = 60;
const INDEX_NODE_WRITE_ORDER_INSTANCE_OFFSET_BEFORE_THE_KEYS: usize = 68;
const INDEX_NODE_PLAIN_HEADER_END_BEFORE_THE_KEYS: usize = 86;
/// 内存稀疏盘按整扇区写。
const SECTOR_BYTES: usize = 512;
/// 找模板时每个槽读多少：码 2 头最宽 86 + 2 × 255 = 596 字节，一个 4096 罩得住。
const BYTES_READ_FOR_A_UNIT_HEADER: usize = 4096;
/// 真孤儿节点放在单元区起点之后这么多槽：这几份池写过的单元都在单元区开头几十槽里，那里之外的槽没有单元认领。
const SLOTS_PAST_THE_UNIT_AREA_START_OF_THE_STRAY_NODE: u64 = 100_000;

fn parameters_of_the_claim_pools() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER_OF_THE_CLAIM_POOLS,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                SLOTS_PER_REGION_OF_THE_CLAIM_POOLS,
            )
            .expect("S = 4 在 4..16 里"),
        },
    }
}

fn memory_pool_of(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: DEVICE_BYTES_OF_THE_CLAIM_POOLS,
    }
}

/// 用户给的文件内容：按位置取的字节，不含任何单元头。
fn file_content_of(length: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index + 10) % 251).expect("小于 251"))
        .collect()
}

/// 伪头放进文件内容的哪里。
#[derive(Clone, Copy, Debug)]
enum ForgedHeaderPlacement {
    /// 内容偏移 = 单元偏移 16384 − 载荷起点：恰在数据单元第二槽的起点上。
    OnTheSecondSlotOfTheFileUnit,
    /// 内容偏移 0：在数据单元第一槽里、不在槽边界上。
    AtTheStartOfTheContent,
}

impl ForgedHeaderPlacement {
    fn content_offset(self) -> usize {
        match self {
            ForgedHeaderPlacement::OnTheSecondSlotOfTheFileUnit => {
                usize::try_from(SLOT_BYTES - DATA_UNIT_PAYLOAD_OFFSET).expect("小于一个槽")
            }
            ForgedHeaderPlacement::AtTheStartOfTheContent => 0,
        }
    }
}

/// 伪头带哪个池的 fsid。
#[derive(Clone, Copy, Debug)]
enum ForgedHeaderFilesystem {
    ThisPool,
    AnotherPool,
}

/// mkfs 并可写挂载之后的池（第一个文件还没写）、挂载取到的实例。
struct MountedClaimPool {
    devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    mounted: singlefs_core::mount::Mounted,
}

fn mounted_claim_pool() -> MountedClaimPool {
    let parameters = parameters_of_the_claim_pools();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [0u32, 1]
        .into_iter()
        .map(|device| {
            (
                DeviceIdentity(device),
                SparseBlockDevice::new(
                    DEVICE_BYTES_OF_THE_CLAIM_POOLS,
                    PhysicalBlockSizeInBytes(512),
                ),
            )
        })
        .collect();
    make_filesystem(&parameters, &mut devices).expect("两块 4 GiB 内存盘上 mkfs 做得成");
    let mounted = mount_writable(&parameters, &mut devices).expect("只做过 mkfs 的池可写挂载");
    MountedClaimPool { devices, mounted }
}

/// 盘 0 单元区里第一个码 2 节点的明文头（挂载之后盘上有实例表节点这类真节点），拿来当伪头的模板。
fn header_of_the_first_index_node_on_the_first_device(image: &MemoryPool) -> Vec<u8> {
    let device = &image.devices[&DeviceIdentity(0)];
    let sectors_per_slot = SLOT_BYTES / 512;
    let mut written_slots: Vec<u64> = device
        .written_sectors()
        .keys()
        .map(|sector| sector / sectors_per_slot)
        .filter(|slot| *slot >= UNIT_AREA_START_SLOT)
        .collect();
    written_slots.dedup();
    written_slots
        .into_iter()
        .map(|slot| {
            device.read(
                DeviceOffsetInBytes(slot * SLOT_BYTES),
                BYTES_READ_FOR_A_UNIT_HEADER,
            )
        })
        .find(|head| head[..4] == UNIT_MAGIC && head[6] == UNIT_CLASS_INDEX_NODE)
        .expect("挂载之后盘 0 单元区里有码 2 节点")
}

/// 照模板改出一个码 2 伪头：树 ID 2⁴⁰、诞生代号 1、写序实例 = 这次挂载的实例，fsid 按 `filesystem` 取，重封头校验和。交回明文头的字节。
fn forged_index_node_header(
    template: &[u8],
    instance: InstanceGeneration,
    filesystem: ForgedHeaderFilesystem,
) -> Vec<u8> {
    let key_span = 2 * usize::from(template[INDEX_NODE_KEY_WIDTH_OFFSET]);
    let header_end = INDEX_NODE_PLAIN_HEADER_END_BEFORE_THE_KEYS + key_span;
    let mut forged = template[..header_end].to_vec();
    forged[INDEX_NODE_TREE_IDENTIFIER_OFFSET..INDEX_NODE_TREE_IDENTIFIER_OFFSET + 8]
        .copy_from_slice(&FORGED_TREE_IDENTIFIER.to_le_bytes());
    let birth_txg = INDEX_NODE_BIRTH_TXG_OFFSET_BEFORE_THE_KEYS + key_span;
    forged[birth_txg..birth_txg + 8].copy_from_slice(&FORGED_BIRTH_TXG.to_le_bytes());
    let write_order_instance = INDEX_NODE_WRITE_ORDER_INSTANCE_OFFSET_BEFORE_THE_KEYS + key_span;
    forged[write_order_instance..write_order_instance + 4]
        .copy_from_slice(&instance.0.to_le_bytes());
    match filesystem {
        ForgedHeaderFilesystem::ThisPool => {}
        ForgedHeaderFilesystem::AnotherPool => {
            let filesystem_identifier =
                INDEX_NODE_FILESYSTEM_IDENTIFIER_OFFSET_BEFORE_THE_KEYS + key_span;
            forged[filesystem_identifier..filesystem_identifier + 8]
                .copy_from_slice(&FOREIGN_FILESYSTEM_IDENTIFIER_LOW.to_le_bytes());
        }
    }
    seal_header_checksum(&mut forged, header_end);
    forged
}

/// 写完第一个文件之后的镜像：`forged` 给了就把伪头放进文件内容的 `placement` 那里（内容是用户的，什么字节都合法）。
fn image_after_a_first_file_carrying(
    forged: Option<(ForgedHeaderPlacement, ForgedHeaderFilesystem)>,
) -> MemoryPool {
    let parameters = parameters_of_the_claim_pools();
    let MountedClaimPool {
        mut devices,
        mut mounted,
    } = mounted_claim_pool();
    let instance = mounted.output.instance;
    let mut content = file_content_of(FILE_BYTES_REACHING_THE_SECOND_SLOT);
    if let Some((placement, filesystem)) = forged {
        let template =
            header_of_the_first_index_node_on_the_first_device(&memory_pool_of(&devices));
        let forged_header = forged_index_node_header(&template, instance, filesystem);
        let at = placement.content_offset();
        content[at..at + forged_header.len()].copy_from_slice(&forged_header);
    }
    {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut mounted.allocator,
            mounted.current.root(),
            FirstFile {
                content: &content,
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_CLAIM_POOLS,
            },
            instance,
            mounted.current.record_bytes(),
        )
        .expect("第一个文件");
    }
    memory_pool_of(&devices)
}

fn violated_invariants(image: &MemoryPool) -> BTreeMap<&'static str, String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

fn tree_identifier_watermark_verdict(image: &MemoryPool) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(invariant, _)| *invariant == "I-7.8")
        .expect("池级 checker 的清单里有 I-7.8")
        .1
}

/// 攻方 `high` 臂：伪头（本池 fsid、本实例写序、诞生代号 1）恰在数据单元第二槽起点上。数据单元在它的起点槽上被认领、跳两槽，
/// 伪头不参与认领：I-7.8 真被评估过且成立，一条违例都没有（改法 F4 之前这一形 I-7.8 红：水位 65548、「出现过的」最大树 ID 2⁴⁰）。
#[test]
fn a_node_header_forged_on_the_second_slot_of_a_file_unit_is_claimed_by_that_unit_and_the_watermark_invariant_holds(
) {
    let image = image_after_a_first_file_carrying(Some((
        ForgedHeaderPlacement::OnTheSecondSlotOfTheFileUnit,
        ForgedHeaderFilesystem::ThisPool,
    )));
    assert_eq!(
        tree_identifier_watermark_verdict(&image),
        InvariantVerdict::Holds,
        "数据单元第二槽上的伪头被数据单元认领，不数进出现过的树 ID"
    );
    assert_eq!(
        violated_invariants(&image),
        BTreeMap::new(),
        "合法历史（用户内容里带伪头）上一条都不许红"
    );
}

/// 攻方 `foreign` 臂：同一个伪头，fsid 改成别的池的。fsid 不是本池的码 2 节点扫描方向照旧数（I-7.8 那一行的读法），
/// 所以挡住它的只有认领规则：I-7.8 成立，一条违例都没有。
#[test]
fn a_node_header_carrying_another_pools_identifier_forged_on_the_second_slot_of_a_file_unit_is_claimed_and_the_watermark_invariant_holds(
) {
    let image = image_after_a_first_file_carrying(Some((
        ForgedHeaderPlacement::OnTheSecondSlotOfTheFileUnit,
        ForgedHeaderFilesystem::AnotherPool,
    )));
    assert_eq!(
        tree_identifier_watermark_verdict(&image),
        InvariantVerdict::Holds,
        "别的池 fsid 的伪头同样被数据单元认领"
    );
    assert_eq!(
        violated_invariants(&image),
        BTreeMap::new(),
        "合法历史（用户内容里带别的池的伪头）上一条都不许红"
    );
}

/// 攻方 `control` 臂：同长内容、不带伪头：I-7.8 成立，一条违例都没有。
#[test]
fn a_file_reaching_the_second_slot_of_its_unit_without_a_forged_header_holds_the_watermark_invariant(
) {
    let image = image_after_a_first_file_carrying(None);
    assert_eq!(
        tree_identifier_watermark_verdict(&image),
        InvariantVerdict::Holds,
        "不带伪头的对照"
    );
    assert_eq!(
        violated_invariants(&image),
        BTreeMap::new(),
        "不带伪头的对照上一条都不许红"
    );
}

/// 攻方 `inode` 臂：伪头放在内容偏移 0（数据单元第一槽里、不在槽边界上），扫描方向按槽起点读头，读不到它：I-7.8 成立，一条违例都没有。
#[test]
fn a_node_header_forged_off_the_slot_boundaries_inside_a_file_unit_is_not_seen_and_the_watermark_invariant_holds(
) {
    let image = image_after_a_first_file_carrying(Some((
        ForgedHeaderPlacement::AtTheStartOfTheContent,
        ForgedHeaderFilesystem::ThisPool,
    )));
    assert_eq!(
        tree_identifier_watermark_verdict(&image),
        InvariantVerdict::Holds,
        "不在槽边界上的伪头扫描方向读不到"
    );
    assert_eq!(
        violated_invariants(&image),
        BTreeMap::new(),
        "伪头不在槽边界上时一条都不许红"
    );
}

/// 坏镜像：不带伪头的那一份上，把同一个伪头写到单元区里一个没有任何单元认领的槽（离写过的单元远远的）的起点：它是一个真孤儿码 2 节点，
/// 树 ID 2⁴⁰ 高于根环水位，诞生代号 1 不超过根环最大 txg、写序实例没有实例表行排除它 ⇒ 认领规则不挡它，I-7.8 照旧红、只红它。
#[test]
fn a_stray_index_node_on_a_slot_no_unit_claims_with_a_tree_identifier_above_the_watermark_reddens_only_the_watermark_invariant(
) {
    let mut image = image_after_a_first_file_carrying(None);
    let template = header_of_the_first_index_node_on_the_first_device(&image);
    let instance = InstanceGeneration(1);
    let mut stray_node =
        forged_index_node_header(&template, instance, ForgedHeaderFilesystem::ThisPool);
    stray_node.resize(stray_node.len().next_multiple_of(SECTOR_BYTES), 0);
    let stray_slot = UNIT_AREA_START_SLOT + SLOTS_PAST_THE_UNIT_AREA_START_OF_THE_STRAY_NODE;
    image
        .devices
        .get_mut(&DeviceIdentity(0))
        .expect("盘 0")
        .write(DeviceOffsetInBytes(stray_slot * SLOT_BYTES), &stray_node);
    let violated = violated_invariants(&image);
    assert_eq!(
        violated.keys().copied().collect::<Vec<&str>>(),
        ["I-7.8"],
        "没被认领的槽上的真孤儿节点只红 I-7.8：{violated:?}"
    );
    assert!(
        violated["I-7.8"].contains(&FORGED_TREE_IDENTIFIER.to_string()),
        "红在树 ID 2⁴⁰ 上：{}",
        violated["I-7.8"]
    );
}
