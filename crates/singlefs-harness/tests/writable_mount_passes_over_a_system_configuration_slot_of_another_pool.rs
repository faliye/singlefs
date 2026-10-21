//! 可写挂载择系统配置先筛掉 fsid 不是本池的槽、按自证不过处置（S3，C707（系统配置判不出里 fsid 不符落哪一边没写）；三方判决
//! `research/prompts/m3-close-sysconf-r3-main-verification.md` 第二节 U5 S3 那一行，用户 2026-10-19 定「先按 fsid 筛，算自证不过」）。
//! 单元区 384 槽那一档的两块内存小盘（`tests/common_admission_pool/mod.rs`）。
mod common_admission_pool;

use common_admission_pool::{
    device_bytes_of_the_384_slot_small_devices, parameters_of_the_384_slot_small_devices,
    plain_device, plain_devices_on, PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::mount_writable_with_space_admission;
use singlefs_core::recovery::{choose_system_configuration, RecoveryFailure};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;

/// 第一个文件之后覆盖写一次（txg 4，返回成功：确认过），盘 0 最新那一槽换成一份别的池的系统配置（同一个世代号，fsid 改一个字节、
/// 整槽校验和重封：读得出、自证得过，只是 fsid 不是本池的；单故障）。不按 fsid 筛时两块盘择到的 fsid 对不上、整池报
/// `SystemConfigurationsDisagree`；筛了之后盘 0 择它的另一槽，可写挂载建在确认过的 txg 4 上做成，不丢这次写。
/// 判别力：删掉筛 fsid 那一判（变异「S3 择系统配置不按本池 fsid 筛（别的池的槽整池报对不上）」）时可写挂载拒成
/// `Recovery(SystemConfigurationsDisagree)`，这一条红。
#[test]
fn the_newest_slot_of_one_device_replaced_by_a_slot_of_another_pool_is_passed_over_and_the_confirmed_write_is_kept(
) {
    let parameters = parameters_of_the_384_slot_small_devices();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters.clone(),
        device_bytes_of_the_384_slot_small_devices(),
        plain_device,
    );
    pool.overwrite(OVERWRITE_BYTES)
        .expect("384 槽小盘上第一个文件之后覆盖写一次落得下");
    let confirmed = *pool.session().current.root();
    assert_eq!(
        (confirmed.instance, confirmed.checkpoint_txg),
        (InstanceGeneration(1), CheckpointTxg(4)),
        "覆盖写那一版是 txg 4，返回了成功"
    );
    let mut image = pool.image();
    let spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let device_zero = image
        .devices
        .get_mut(&DeviceIdentity(0))
        .expect("池里有盘 0");
    let (newest_offset, mut newest) = [0, 1]
        .into_iter()
        .map(|slot_index| {
            let offset = DeviceOffsetInBytes(slot_index * spacing);
            let parsed = SystemConfiguration::parse_slot(&device_zero.read(offset, slot_bytes))
                .expect("覆盖写之后盘 0 两槽都自证");
            (offset, parsed)
        })
        .max_by_key(|(_, parsed)| parsed.quantities.slot_generation)
        .expect("两槽");
    newest.immutable.filesystem_identifier[15] ^= 0xff;
    device_zero.write(newest_offset, &newest.to_slot());
    assert_eq!(
        choose_system_configuration(&image),
        Err(RecoveryFailure::SystemConfigurationsDisagree),
        "前提：不按 fsid 筛时盘 0 择到别的池那一槽、与盘 1 的 fsid 对不上"
    );
    let mut devices = plain_devices_on(&image);
    let mounted = mount_writable_with_space_admission(
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("别的池的槽当自证不过、择盘 0 的另一槽：可写挂载做成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        (confirmed.instance, confirmed.checkpoint_txg),
        "可写挂载建在确认过的那一版上，不丢那次覆盖写"
    );
}
