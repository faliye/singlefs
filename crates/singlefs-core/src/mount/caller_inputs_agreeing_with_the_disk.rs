//! 调用方交进来的参数与盘表照盘上择到的系统配置核：设备身份重复、盘表与系统配置不一致的几项、
//! 探测到的物理块大小、盘上自记的本盘设备号，可写设备数的准入。

use super::*;

/// 交进来的每块盘自证过的系统配置槽（本池的 fsid、整槽校验和过，`recovery::verified_system_configuration_slots`，槽距取本池的固定结构槽距）
/// 里记的本盘设备号不是盘表给它的身份的各一项（一块盘上同一个号只报一次），按交进来的次序（D18（块里携带什么信息） 已定项 11
/// 「挂着之后收盘表的每个入口……核三样」的第二样）。可写挂载与挂着之后收盘表的入口经 [`device_table_disagreeing_with`] 调它，
/// 会话在每次用户改动之前调它（`mounted_session`；代码三方 m2-closeout-code-r2「盘体对调的盘表」：会话此前只比身份次序）。
/// 一份自证过的都没有的盘不在这里报。调用方先判过盘表里没有重复身份（按身份读盘，重复了读的是哪一块说不清）。读每块盘的两个系统配置槽，不写盘。
///
/// 循环：按交进来的盘逐块（轮数 = 盘数），每块读两槽；跨轮只往交回的清单里追加，没有提前出口。
pub(crate) fn own_device_numbers_on_disk_differing_from_the_identities_handed_in<
    Reader: PoolReader + ?Sized,
>(
    reader: &Reader,
    parameters_of_the_pool: &MakeFilesystemParameters,
) -> Vec<OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn> {
    let mut differing = Vec::new();
    for identity_handed_in in reader.device_identities() {
        let own_device_numbers_on_disk: BTreeSet<DeviceIdentity> =
            self_verified_system_configuration_slots_of_the_pool(
                reader,
                identity_handed_in,
                parameters_of_the_pool,
            )
            .into_iter()
            .map(|slot| slot.immutable.this_device)
            .collect();
        differing.extend(
            own_device_numbers_on_disk
                .into_iter()
                .filter(|own_device_number_on_disk| {
                    *own_device_number_on_disk != identity_handed_in
                })
                .map(|own_device_number_on_disk| {
                    OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn {
                        identity_handed_in,
                        own_device_number_on_disk,
                    }
                }),
        );
    }
    differing
}

/// 一块盘两个系统配置槽里本池自证过的那几份（整槽校验和过、fsid 与本池相同，`recovery::verified_system_configuration_slots`，
/// 槽距取本池的固定结构槽距）。一份都没有，这块盘就不「可见」（D18（块里携带什么信息） 已定项 11「可写挂载的顺序」：
/// 「可见」= 独占打开成功且系统配置读得通）：可写挂载取号之前的逐盘核（[`devices_without_the_selected_version`] 第一支）、
/// 挂着之后收盘表的入口（[`caller_inputs_agreeing_with_the_disk`]）与会话每次发布之前（`mounted_session`）判的是同一件事，都经这一个读法。
/// `parameters_of_the_pool` 是盘上择到的那份系统配置里的参数（[`parameters_of_the_system_configuration`]），不是调用方交进来的。
/// 读这块盘的两个系统配置槽，不写盘。
pub(super) fn self_verified_system_configuration_slots_of_the_pool<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    device: DeviceIdentity,
    parameters_of_the_pool: &MakeFilesystemParameters,
) -> Vec<crate::system_configuration::SystemConfiguration> {
    verified_system_configuration_slots(
        reader,
        device,
        u64::from(parameters_of_the_pool.geometry.fixed_structure_slot_spacing),
        &parameters_of_the_pool.filesystem_identifier,
    )
}

/// 交进来的盘里不「可见」的那几块（两个系统配置槽一份本池自证过的都没有，[`self_verified_system_configuration_slots_of_the_pool`]），
/// 按交进来的次序（代码三方 m2-closeout-code-r1 Z3-A 乙：挂着之后每个收盘表的入口都逐盘核，含会话发布；用户 2026-09-27 定）。
/// 挂着之后收盘表的入口在 [`caller_inputs_agreeing_with_the_disk`] 里调，会话在每次发布之前调（`mounted_session`）。
/// 读每块盘的两个系统配置槽，不写盘。
///
/// 循环：按交进来的盘逐块（轮数 = 盘数），每块读两槽；跨轮只往交回的清单里追加，没有提前出口。
pub(crate) fn devices_without_a_self_verified_system_configuration<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    parameters_of_the_pool: &MakeFilesystemParameters,
) -> Vec<DeviceIdentity> {
    reader
        .device_identities()
        .into_iter()
        .filter(|device| {
            self_verified_system_configuration_slots_of_the_pool(
                reader,
                *device,
                parameters_of_the_pool,
            )
            .is_empty()
        })
        .collect()
}

/// 交进来的盘表里有几个不同的设备身份：同一个身份交两次（两份都标成盘 0）只算一块盘。D2（RAID 条带策略） 已定项 13 的
/// 「可写设备数」数的是盘，不是盘表的项数（实二八交回第 2 问；调度记录 2026-09-24 第三节实现批次排法里实四那一项「数不同的设备身份」）。
/// 只看盘表里的身份，不读盘：池外的身份照样算一块，由取号之前的逐盘核拒（`devices_without_the_selected_version`）。
pub(super) fn distinct_device_identities_handed_in<Device>(
    devices: &[(DeviceIdentity, Device)],
) -> DeviceCount {
    DeviceCount(
        devices
            .iter()
            .map(|(identity, _)| *identity)
            .collect::<BTreeSet<DeviceIdentity>>()
            .len(),
    )
}

/// 交进来的盘表里重复的设备身份有就拒（[`MountError::DeviceIdentitiesHandedInMoreThanOnce`]）：只看盘表里的身份，不读盘、不写盘。
/// 可写挂载在「可写设备数 ≥ w 的下限」那一判之后、读第一块盘之前调它。
///
/// 循环：按盘表逐项数（轮数 = 盘表项数），跨轮只往每个身份的计数上加一，没有提前出口。
pub(super) fn refuse_device_identities_handed_in_more_than_once<Device>(
    devices: &[(DeviceIdentity, Device)],
) -> Result<(), CallerInputsDisagreeingWithTheDisk> {
    let mut times_handed_in_by_device: BTreeMap<DeviceIdentity, usize> = BTreeMap::new();
    for (identity, _) in devices {
        *times_handed_in_by_device.entry(*identity).or_insert(0) += 1;
    }
    let repeated: Vec<RepeatedDeviceIdentity> = times_handed_in_by_device
        .into_iter()
        .filter(|(_, times_handed_in)| *times_handed_in > 1)
        .map(|(device, times_handed_in)| RepeatedDeviceIdentity {
            device,
            times_handed_in,
        })
        .collect();
    if repeated.is_empty() {
        Ok(())
    } else {
        Err(CallerInputsDisagreeingWithTheDisk::DeviceIdentitiesHandedInMoreThanOnce { repeated })
    }
}

/// 盘上一份系统配置的系统不可变配置里、mkfs 参数那几项：fsid、根环逐区域设备身份与几何（D22（单元原子性怎么合成） 已定项 26 第一档：
/// mkfs 之后不可改）。写入口按它建。
pub(super) fn parameters_of_the_system_configuration(
    system_configuration: &crate::system_configuration::SystemConfiguration,
) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: system_configuration.immutable.filesystem_identifier,
        region_devices: system_configuration.immutable.region_devices,
        geometry: system_configuration.immutable.sizes,
    }
}

/// 交进来的盘表与盘上择到的那份系统配置不一致的几项（[`DeviceTableDisagreement`]）：盘数不是 `devs` 的一项，
/// 再按盘表次序、每块盘自证过的系统配置槽（本池的 fsid、整槽校验和过，`recovery::verified_system_configuration_slots`）里
/// 记的本盘设备号不是盘表给它的身份的各一项（一块盘上同一个号只报一次），再按盘表次序、探测到的 `physical_block_size` 不是系统配置记着的那个的各一项
/// （[`physical_block_sizes_probed_on_the_devices_differing_from_the_system_configuration`]）。读每块盘的两个系统配置槽，不写盘。
/// 调用方先判过盘表里没有重复身份（按身份读盘，重复了读的是哪一块说不清）。
///
/// 循环：按盘表逐块（轮数 = 盘表项数），每块读两槽；跨轮只往交回的清单里追加，没有提前出口。
pub(super) fn device_table_disagreeing_with<Device: BlockDevice>(
    devices: &[(DeviceIdentity, Device)],
    system_configuration: &crate::system_configuration::SystemConfiguration,
) -> Vec<DeviceTableDisagreement> {
    let devices_handed_in = DeviceCount(devices.len());
    let device_count_in_the_system_configuration = DeviceCount(
        usize::try_from(system_configuration.immutable.device_count)
            .expect("系统配置里的设备数是 4 字节，装得进 usize"),
    );
    let mut disagreements = Vec::new();
    if devices_handed_in != device_count_in_the_system_configuration {
        disagreements.push(
            DeviceTableDisagreement::DeviceCountDiffersFromTheSystemConfiguration {
                devices_handed_in,
                device_count_in_the_system_configuration,
            },
        );
    }
    disagreements.extend(
        own_device_numbers_on_disk_differing_from_the_identities_handed_in(
            devices,
            &parameters_of_the_system_configuration(system_configuration),
        )
        .into_iter()
        .map(
            |OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn {
                 identity_handed_in,
                 own_device_number_on_disk,
             }| {
                DeviceTableDisagreement::OwnDeviceNumberDiffersFromTheIdentityHandedIn {
                    identity_handed_in,
                    own_device_number_on_disk,
                }
            },
        ),
    );
    disagreements.extend(
        physical_block_sizes_probed_on_the_devices_differing_from_the_system_configuration(
            devices,
            PhysicalBlockSizeInBytes(system_configuration.immutable.sizes.physical_block_size),
        )
        .into_iter()
        .map(
            |PhysicalBlockSizeProbedOnADeviceDifferingFromTheSystemConfiguration {
                 identity_handed_in,
                 probed_physical_block_size,
                 physical_block_size_in_the_system_configuration,
             }| {
                DeviceTableDisagreement::PhysicalBlockSizeProbedOnTheDeviceDiffersFromTheSystemConfiguration {
                    identity_handed_in,
                    probed_physical_block_size,
                    physical_block_size_in_the_system_configuration,
                }
            },
        ),
    );
    disagreements
}

/// 按盘表次序逐块探测 `physical_block_size`，与择到的那份系统配置记着的 mkfs 时的值（`physical_block_size_in_the_system_configuration`）
/// 不等的各报一项（[`PhysicalBlockSizeProbedOnADeviceDifferingFromTheSystemConfiguration`]；D12（目标介质） 已定项 4，C1040）。
/// 可写挂载与挂着之后收盘表的入口经 [`device_table_disagreeing_with`] 调它，会话发布用户改动的入口
/// （`mounted_session::MountedSession::publish_user_change`）拿会话持着的那份参数里的值直接调它：两边是同一判。
/// 只调探测，不读盘、不写盘。
///
/// 循环：按盘表逐块（轮数 = 盘表项数），跨轮只往交回的清单里追加，没有提前出口。
pub(crate) fn physical_block_sizes_probed_on_the_devices_differing_from_the_system_configuration<
    Device: BlockDevice,
>(
    devices: &[(DeviceIdentity, Device)],
    physical_block_size_in_the_system_configuration: PhysicalBlockSizeInBytes,
) -> Vec<PhysicalBlockSizeProbedOnADeviceDifferingFromTheSystemConfiguration> {
    devices
        .iter()
        .filter_map(|(identity_handed_in, device)| {
            let probed_physical_block_size = device.probe_physical_block_size();
            (probed_physical_block_size != physical_block_size_in_the_system_configuration)
                .then_some(
                    PhysicalBlockSizeProbedOnADeviceDifferingFromTheSystemConfiguration {
                        identity_handed_in: *identity_handed_in,
                        probed_physical_block_size,
                        physical_block_size_in_the_system_configuration,
                    },
                )
        })
        .collect()
}

/// 写入口用的参数：盘上择到的那份系统配置里的 mkfs 参数（[`parameters_of_the_system_configuration`]）。调用方交进来的参数与它逐项比，
/// 交进来的盘表与它的设备数、每块盘的本盘设备号比（[`device_table_disagreeing_with`]），有一项不同就拒
/// （[`MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`]）。读每块盘的两个系统配置槽，不写盘。
/// 调用方先判过盘表里没有重复身份。
pub(super) fn parameters_of_the_selected_system_configuration_agreeing_with_the_caller_parameters_and_device_table<
    Device: BlockDevice,
>(
    caller_parameters: &MakeFilesystemParameters,
    devices: &[(DeviceIdentity, Device)],
    system_configuration: &crate::system_configuration::SystemConfiguration,
) -> Result<MakeFilesystemParameters, CallerInputsDisagreeingWithTheDisk> {
    let parameters_on_disk = parameters_of_the_system_configuration(system_configuration);
    // 两边都整个解构：参数或几何加了字段、这里没比，编译不过。
    let MakeFilesystemParameters {
        filesystem_identifier: caller_filesystem_identifier,
        region_devices: caller_region_devices,
        geometry:
            SystemImmutableSizes {
                physical_block_size: caller_physical_block_size,
                minimum_input_output_bytes: caller_minimum_input_output_bytes,
                fixed_structure_slot_spacing: caller_fixed_structure_slot_spacing,
                journal_ring_bytes: caller_journal_ring_bytes,
                root_ring_slots_per_region: caller_root_ring_slots_per_region,
            },
    } = caller_parameters;
    let MakeFilesystemParameters {
        filesystem_identifier: disk_filesystem_identifier,
        region_devices: disk_region_devices,
        geometry:
            SystemImmutableSizes {
                physical_block_size: disk_physical_block_size,
                minimum_input_output_bytes: disk_minimum_input_output_bytes,
                fixed_structure_slot_spacing: disk_fixed_structure_slot_spacing,
                journal_ring_bytes: disk_journal_ring_bytes,
                root_ring_slots_per_region: disk_root_ring_slots_per_region,
            },
    } = &parameters_on_disk;
    let disagreeing_fields: Vec<MakeFilesystemParameterField> = [
        (
            MakeFilesystemParameterField::FilesystemIdentifier,
            caller_filesystem_identifier == disk_filesystem_identifier,
        ),
        (
            MakeFilesystemParameterField::RegionDevices,
            caller_region_devices == disk_region_devices,
        ),
        (
            MakeFilesystemParameterField::PhysicalBlockSize,
            caller_physical_block_size == disk_physical_block_size,
        ),
        (
            MakeFilesystemParameterField::MinimumInputOutputBytes,
            caller_minimum_input_output_bytes == disk_minimum_input_output_bytes,
        ),
        (
            MakeFilesystemParameterField::FixedStructureSlotSpacing,
            caller_fixed_structure_slot_spacing == disk_fixed_structure_slot_spacing,
        ),
        (
            MakeFilesystemParameterField::JournalRingBytes,
            caller_journal_ring_bytes == disk_journal_ring_bytes,
        ),
        (
            MakeFilesystemParameterField::RootRingSlotsPerRegion,
            caller_root_ring_slots_per_region == disk_root_ring_slots_per_region,
        ),
    ]
    .into_iter()
    .filter(|(_, is_equal_on_both_sides)| !is_equal_on_both_sides)
    .map(|(field, _)| field)
    .collect();
    let disagreeing_device_table = device_table_disagreeing_with(devices, system_configuration);
    if disagreeing_fields.is_empty() && disagreeing_device_table.is_empty() {
        Ok(parameters_on_disk)
    } else {
        Err(
            CallerInputsDisagreeingWithTheDisk::ParametersOrDeviceTableDisagreeWithTheSelectedSystemConfiguration {
                disagreeing_fields,
                disagreeing_device_table,
            },
        )
    }
}

/// 挂着之后收调用方参数与盘表的入口（抬 F、准入抬 F、一次准入里再推一串、正常卸载）与管理员回退照可写挂载同一套核
/// （代码审阅第 17、18 条）：盘表里有身份交了不止一次就拒（不读盘）；择系统配置；调用方的参数逐项比，盘表比设备数与每块盘的本盘设备号，
/// 有一项不同就拒；这几项都对上之后，交进来的每块盘要「可见」——两个系统配置槽里至少一份本池自证过的，有一块没有就拒
/// （[`DeviceTableDisagreement::NoSelfVerifiedSystemConfigurationOnTheDeviceHandedIn`]，与可写挂载取号之前的逐盘核第一支同一判；
/// 代码三方 m2-closeout-code-r1 Z3-A 乙）。择系统配置那一步只要求读得出的那几份彼此对得上，空盘上一份都没有、择得出来，
/// 前三项也报不出它（本盘设备号是逐份自证槽比的，一份都没有就一条都不比），所以另判这一项。
/// 「可见」之后再核落后支：每块盘最新那份自证系统配置不落后于调用方持着的现行那一版（`current`）、或落后而现行那一版的单元都在、
/// 它的根住在这块盘上时那个根环槽里也是它（[`devices_behind_the_current_version`]；D18（块里携带什么信息） 已定项 11「挂着之后收盘表的每个入口……核三样」，用户 2026-09-27 定，
/// 代码三方 m2-closeout-code-r2「同池旧快照的盘经入口洗白」）。三样里的本盘设备号在前面盘表那一判里已比过（[`device_table_disagreeing_with`]）。
/// 只读盘，不写盘：每个入口在它的第一个写之前调它。落后支读到只坏一份的那几组经 `allocator` 的计数层进别的读那一份账
/// （[`PoolAllocator::reading_and_counting_the_other_unit_reads_with_one_copy_failing`]；D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）。
///
/// # Errors
/// 交进来的不对（[`CallerInputsDisagreeingWithTheDisk`] 的三种；不「可见」的盘报在第二种的 `disagreeing_device_table` 里，
/// 落后又缺单元的盘报在第三种里）；系统配置择不出来（别的池的盘换进来，两块盘上各有一份 fsid 不同的自证槽，报在这里）。
#[allow(
    clippy::ptr_arg,
    reason = "choose_system_configuration 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
pub(super) fn caller_inputs_agreeing_with_the_disk<Device: BlockDevice>(
    caller_parameters: &MakeFilesystemParameters,
    devices: &Vec<(DeviceIdentity, Device)>,
    allocator: &PoolAllocator,
    current: &TransactionOutput,
) -> Result<CallerInputsAgreeingWithTheDisk, CallerInputsCheckRefused> {
    refuse_device_identities_handed_in_more_than_once(devices)
        .map_err(CallerInputsCheckRefused::Disagreeing)?;
    let system_configuration = choose_system_configuration(devices)
        .map_err(CallerInputsCheckRefused::SystemConfigurationNotChosen)?;
    let parameters_of_the_pool =
        parameters_of_the_selected_system_configuration_agreeing_with_the_caller_parameters_and_device_table(
            caller_parameters,
            devices,
            &system_configuration,
        )
        .map_err(CallerInputsCheckRefused::Disagreeing)?;
    let devices_not_visible: Vec<DeviceTableDisagreement> =
        devices_without_a_self_verified_system_configuration(devices, &parameters_of_the_pool)
            .into_iter()
            .map(|identity_handed_in| {
                DeviceTableDisagreement::NoSelfVerifiedSystemConfigurationOnTheDeviceHandedIn {
                    identity_handed_in,
                }
            })
            .collect();
    if !devices_not_visible.is_empty() {
        return Err(CallerInputsCheckRefused::Disagreeing(
            CallerInputsDisagreeingWithTheDisk::ParametersOrDeviceTableDisagreeWithTheSelectedSystemConfiguration {
                disagreeing_fields: Vec::new(),
                disagreeing_device_table: devices_not_visible,
            },
        ));
    }
    // 落后支读到窗口之外只坏一份的那几组经计数层记进别的读那一份账（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」），
    // 下一次用户改动发成之后另发的那一批换下它们；这一核拒了、调用方把分配器换回进来时那一份，账照留（各份分配器拷贝共用）。
    let devices_behind = allocator
        .reading_and_counting_the_other_unit_reads_with_one_copy_failing(devices, |reader| {
            devices_behind_the_current_version(reader, &parameters_of_the_pool, current)
        });
    if !devices_behind.is_empty() {
        return Err(CallerInputsCheckRefused::Disagreeing(
            CallerInputsDisagreeingWithTheDisk::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                devices: devices_behind,
            },
        ));
    }
    Ok(CallerInputsAgreeingWithTheDisk {
        system_configuration,
        parameters_of_the_pool,
    })
}

/// 会话的推环入口（`mounted_session::MountedSession::push_the_root_ring_to_the_last_ring_slot_it_reaches_dropping_rollback_candidates_below_four_states`）
/// 收盘表时走的核：与抬 F 那一串的空发布收盘表同一套（[`caller_inputs_agreeing_with_the_disk`]），交回盘上择到的那份系统配置照它建的参数，
/// 写入口只用这一份。只读盘。
///
/// # Errors
/// 同 [`caller_inputs_agreeing_with_the_disk`]，装成 `MountError`（与抬 F 那一串交回的同一种）。
#[allow(
    clippy::ptr_arg,
    reason = "caller_inputs_agreeing_with_the_disk 走 &Vec：choose_system_configuration 要一个有大小的读者"
)]
pub(crate) fn parameters_of_the_pool_agreeing_with_the_caller_inputs<Device: BlockDevice>(
    caller_parameters: &MakeFilesystemParameters,
    devices: &Vec<(DeviceIdentity, Device)>,
    allocator: &PoolAllocator,
    current: &TransactionOutput,
) -> Result<MakeFilesystemParameters, MountError> {
    Ok(
        caller_inputs_agreeing_with_the_disk(caller_parameters, devices, allocator, current)?
            .parameters_of_the_pool,
    )
}

/// 可写挂载准入的第一条合取（D2（RAID 条带策略） 已定项 13「挂载准入」：可写设备数 ≥ w 的下限；D18（块里携带什么信息） 已定项 11
/// 「可写挂载的顺序」先判这一条）：交进来的不同设备身份数（[`distinct_device_identities_handed_in`]）低于 [`STRIPE_WIDTH_LOWER_BOUND`] 就拒。
/// 只比两个数，不读盘、不写盘。
/// 可写挂载在读第一块盘之前调它；之后到取号为止交进来的那份盘表只借给重建与写入口、长短不变，取号写的就是这里数过的这几块盘。
pub(super) fn admit_the_writable_device_count(
    devices_handed_in: DeviceCount,
) -> Result<(), MountError> {
    if devices_handed_in < STRIPE_WIDTH_LOWER_BOUND {
        return Err(
            MountError::WritableDeviceCountBelowTheStripeWidthLowerBound {
                devices_handed_in,
                stripe_width_lower_bound: STRIPE_WIDTH_LOWER_BOUND,
            },
        );
    }
    Ok(())
}
