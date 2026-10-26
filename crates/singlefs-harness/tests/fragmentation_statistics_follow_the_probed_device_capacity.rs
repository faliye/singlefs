//! C323（镜像大小全仓没有条款）checker 那一半的判别力：单元区的末端 = 设备容量（由设备探测得出，2026-09-14 用户定案不加字段），
//! I-3.16（碎片度与全空段统计对得上） 复算 runs 与全空聚簇段数时盘末取探测出来的容量，不取哪个固定档。
//! 同一条新池新建文件的路分别跑在两种盘大小上（4 GiB，与比它大三个整段的一档），两份合法镜像上 I-3.16 都真判、成立；
//! 4 GiB 那一份镜像盘上一个字节不改、只把探测出来的容量加长三个整段，记账的全空聚簇段数比按探测容量重算的少三，I-3.16 红
//! （I-5.2（空闲统计对得上） 跟着红：空闲 + 已分配 不再等于单元区）。

use std::collections::BTreeSet;

use singlefs_checker::image::{
    InvariantVerdict, FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_format::{
    CLUSTER_SEGMENT_SLOTS, SLOT_BYTES, TEST_IMAGE_DEFAULT_BYTES, UNIT_AREA_START_SLOT,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::scenario::{e142_parameters, run_new_pool_file_creation};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 4 GiB 的盘上单元区正好是整段：(262 144 − 50 176) 槽 = 3312 段，没有不成段的尾巴。
const FOUR_GIBIBYTE_DEVICE_BYTES: u64 = TEST_IMAGE_DEFAULT_BYTES;
/// 大的那一档比 4 GiB 多出的整段数。
const ADDED_CLUSTER_SEGMENTS: u64 = 3;
/// 4 GiB 加三个整段（3 × 64 槽 × 16 KiB = 3 MiB）：单元区 3315 段，同样没有尾巴。
const DEVICE_BYTES_THREE_CLUSTER_SEGMENTS_LARGER: u64 =
    FOUR_GIBIBYTE_DEVICE_BYTES + ADDED_CLUSTER_SEGMENTS * CLUSTER_SEGMENT_SLOTS * SLOT_BYTES;
/// 4 GiB 盘上新池新建文件那一版记账的全空聚簇段数（E142（新池新建文件的干跑） 第二十二次跑第二段：每盘 3310）。
const EMPTY_CLUSTER_SEGMENTS_RECORDED_ON_FOUR_GIBIBYTE_DEVICES: u64 = 3310;

/// 两块 `device_bytes` 大的内存盘上走 E142 那一条路（mkfs → 取号 → 暖机 → 新池新建文件），交回盘上字节，探测容量就是 `device_bytes`。
fn new_pool_file_creation_image_on_devices_of(device_bytes: u64) -> MemoryPool {
    let parameters = e142_parameters(512, 512);
    let stream = SharedStream::new();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0..2u32)
        .map(|device_number| {
            let identity = DeviceIdentity(device_number);
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    run_new_pool_file_creation(&parameters, &mut devices, &stream, |_point, _devices| {})
        .expect("新池新建文件在这两块内存盘上跑得通");
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: device_bytes,
    }
}

fn violated(verdicts: &[(&'static str, InvariantVerdict)]) -> Vec<(&'static str, String)> {
    verdicts
        .iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((*invariant, detail.clone())),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

fn verdict_among(
    verdicts: &[(&'static str, InvariantVerdict)],
    invariant: &str,
) -> InvariantVerdict {
    verdicts
        .iter()
        .find(|(candidate, _)| *candidate == invariant)
        .map(|(_, verdict)| verdict.clone())
        .expect("清单里有这一条")
}

/// 合法世界：4 GiB 的盘。I-3.16 真判、成立（不是不适用），整张镜像一条都不红。
#[test]
fn new_pool_file_creation_on_four_gibibyte_devices_holds_the_fragmentation_statistics() {
    let image = new_pool_file_creation_image_on_devices_of(FOUR_GIBIBYTE_DEVICE_BYTES);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        verdict_among(
            &verdicts,
            FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN
        ),
        InvariantVerdict::Holds,
        "4 GiB 的盘：I-3.16 真判、成立"
    );
    assert_eq!(violated(&verdicts), Vec::new(), "合法镜像一条都不红");
}

/// 合法世界：比 4 GiB 大三个整段的盘。写路径的记账照这块盘的容量记全空聚簇段数，checker 按探测出来的容量重算、对得上：
/// I-3.16 真判、成立，整张镜像一条都不红。checker 盘末取一个固定档（4 GiB）时这里误红。
#[test]
fn new_pool_file_creation_on_devices_three_cluster_segments_larger_holds_the_fragmentation_statistics(
) {
    let image =
        new_pool_file_creation_image_on_devices_of(DEVICE_BYTES_THREE_CLUSTER_SEGMENTS_LARGER);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        verdict_among(
            &verdicts,
            FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN
        ),
        InvariantVerdict::Holds,
        "大三个整段的盘：I-3.16 真判、成立"
    );
    assert_eq!(violated(&verdicts), Vec::new(), "合法镜像一条都不红");
}

/// 必红世界：4 GiB 盘上那份镜像，盘上一个字节不改，探测出来的容量加长三个整段。记账的全空聚簇段数还是 3310，按探测容量重算是 3313：
/// I-3.16 红、说明点名盘 0 的全空聚簇段数（runs 不变：尾巴上那一段空闲槽只是变长，盘 0 的 runs 先判、成立，第一处违例才落到全空段那一行）；
/// I-5.2 跟着红（空闲 + 已分配 不再等于加长之后的单元区），别的一条都不红。
#[test]
fn the_same_pool_bytes_on_devices_probed_three_cluster_segments_larger_redden_the_empty_segment_statistics(
) {
    let image = new_pool_file_creation_image_on_devices_of(FOUR_GIBIBYTE_DEVICE_BYTES);
    let probed_larger = MemoryPool {
        devices: image.devices.clone(),
        device_size_in_bytes: DEVICE_BYTES_THREE_CLUSTER_SEGMENTS_LARGER,
    };
    assert_eq!(
        probed_larger.devices, image.devices,
        "盘上一个字节都没改，变的只有探测出来的容量"
    );
    assert_eq!(
        (DEVICE_BYTES_THREE_CLUSTER_SEGMENTS_LARGER / SLOT_BYTES - UNIT_AREA_START_SLOT)
            % CLUSTER_SEGMENT_SLOTS,
        0,
        "加长之后的单元区仍是整段：多出来的正好是三个全空段"
    );
    let red = violated(&check_pool_image(&probed_larger));
    let red_names: BTreeSet<&str> = red.iter().map(|(invariant, _)| *invariant).collect();
    assert_eq!(
        red_names,
        BTreeSet::from([
            FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
            "I-5.2"
        ]),
        "只红 I-3.16 与连带的 I-5.2：{red:?}"
    );
    let recomputed_empty_segments =
        EMPTY_CLUSTER_SEGMENTS_RECORDED_ON_FOUR_GIBIBYTE_DEVICES + ADDED_CLUSTER_SEGMENTS;
    let fragmentation_detail = &red
        .iter()
        .find(|(invariant, _)| {
            *invariant == FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN
        })
        .expect("I-3.16 在红的里面")
        .1;
    assert_eq!(
        fragmentation_detail,
        &format!(
            "盘 0：记账的全空聚簇段数 {EMPTY_CLUSTER_SEGMENTS_RECORDED_ON_FOUR_GIBIBYTE_DEVICES}，全扫重算是 {recomputed_empty_segments}"
        ),
        "I-3.16 的第一处违例是盘 0 的全空聚簇段数：记账 3310，按探测容量重算 3313"
    );
}
