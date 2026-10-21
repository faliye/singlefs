//! 落后支缺单元那一半（乙'，三方判决 `research/prompts/m3-close-sysconf-r3-main-verification.md` 第二节 U3 与第四节「落地」；C593（落后且缺一份单元的盘被永久拒可写））：
//! 本盘环槽里有所选那一版的末条记录就不算落后，根那一半不动。两条用例都在单元区 384 槽那一档的两块内存小盘上，录一次覆盖写 P
//! （第一个文件之后先覆盖写一次，P 是 txg 5，根落区域 2 的盘 0；造法在 `tests/common_unwitnessed_publish/mod.rs`）。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    plain_devices_on, record_one_publish_after_overwrites, PublishRecordedAsR,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{
    mount_writable_with_space_admission, MountError, SelectedVersionLackingOnDevice,
};
use singlefs_harness::memory_pool::MemoryPool;

const OVERWRITES_BEFORE_P: u64 = 1;

/// C593 那一形：P 的单元、记录与根都落了、两块盘的系统配置轮换都没落（一次正常崩溃），盘 1 上 P 写的一个单元那一份再翻坏一个字节（单故障）。
/// 两块盘最新那份系统配置都落后于 P 的末条记录；盘 1 又缺一份 P 的单元。乙' 之下盘 1 自己的 journal 环槽里有 P 的末条记录，
/// 不算落后，可写挂载建在 P 上做成。判别力：删掉「本盘有末条记录就不算落后」那一判（变异「乙' 落后支缺单元那一半不看本盘的末条记录
/// （C593 那一形照旧永久拒可写）」）时盘 1 落后又缺单元，拒成 `WritableMountRefusedByDevicesWithoutTheSelectedVersion`，这一条红。
#[test]
fn a_device_behind_by_the_lost_rotation_missing_one_unit_copy_but_holding_the_last_record_mounts_writable(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_P, PublishRecordedAsR::Overwrite);
    assert_eq!(
        recorded.key,
        (InstanceGeneration(1), CheckpointTxg(5)),
        "第一个文件 txg 3、覆盖写一次 txg 4，P 是 txg 5"
    );
    let mut crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let unit_copy_on_device_one = recorded
        .unit_writes()
        .into_iter()
        .find(|write| write.device == DeviceIdentity(1))
        .expect("P 写了单元，每块盘一份");
    crash_image.flip_byte(DeviceIdentity(1), unit_copy_on_device_one.offset, 600);
    let mut devices = plain_devices_on(&crash_image);
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("盘 1 落后于 P 的系统配置轮换、缺一份 P 的单元，而它的 journal 环槽里有 P 的末条记录：不算落后，可写挂载做成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.key,
        "可写挂载建在 P 上（P 的根落了盘）"
    );
}

/// 发布单元写到一半时取下的快照盘：池在两块盘上做完 P（P 的根在盘 0），再把盘 1 换成 P 的单元写在盘 1 上还差最后一次时取下的快照
/// （记录与根都没落）。快照盘最新那份系统配置落后于 P、缺 P 的单元，环槽里也没有 P 的末条记录：乙' 之下照旧拒在任何写之前、
/// 点名盘 1 缺单元（第三轮攻方 U3 那一格，三种读法同拒）。「缺的份数不到 2 不算缺」单用时这一形放行（第二轮判决 T3）。
#[test]
fn a_device_snapshotted_halfway_through_the_unit_writes_of_the_version_is_still_refused_before_any_write(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_P, PublishRecordedAsR::Overwrite);
    let unit_writes_on_device_one: Vec<_> = recorded
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.device == DeviceIdentity(1))
        .collect();
    let (last_unit_write_index_on_device_one, _) = unit_writes_on_device_one
        .iter()
        .rev()
        .find(|(_, write)| write.kind == singlefs_harness::segments::StepKind::UnitWrite)
        .copied()
        .expect("P 在盘 1 上写了单元");
    let mut snapshot = recorded.base.clone();
    let writes_on_device_one_before_its_last_unit_write: Vec<_> = recorded.writes
        [..last_unit_write_index_on_device_one]
        .iter()
        .filter(|write| write.device == DeviceIdentity(1))
        .cloned()
        .collect();
    assert!(
        writes_on_device_one_before_its_last_unit_write
            .iter()
            .all(|write| write.kind == singlefs_harness::segments::StepKind::UnitWrite),
        "快照取在盘 1 的单元写落了一部分、记录与根都没落的时刻"
    );
    snapshot.apply_writes(&writes_on_device_one_before_its_last_unit_write);
    let mut image = recorded.image_with_the_whole_publish_persisted();
    image.devices.insert(
        DeviceIdentity(1),
        snapshot.devices[&DeviceIdentity(1)].clone(),
    );
    let before = image.clone();
    let mut devices = plain_devices_on(&image);
    let refusal = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect_err("快照盘落后于 P、缺 P 的单元、环槽里没有 P 的末条记录：拒可写");
    let MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion {
        devices: lacking, ..
    } = refusal
    else {
        panic!("该报 WritableMountRefusedByDevicesWithoutTheSelectedVersion，实际 {refusal:?}");
    };
    assert_eq!(lacking.len(), 1, "只有快照盘不带 P：{lacking:?}");
    assert_eq!(lacking[0].device, DeviceIdentity(1));
    assert!(
        matches!(
            lacking[0].lacking,
            SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndMissingItsUnits { .. }
        ),
        "盘 1 落后又缺 P 的单元：{lacking:?}"
    );
    let after = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: image.device_size_in_bytes,
    };
    assert_eq!(after, before, "拒在任何写之前：两块盘逐字节不变");
}
