import os

from edit_lib import edit

MT ="crates/singlefs-core/src/mount.rs"
edit(MT, """    CallerParametersDisagreeWithTheSelectedSystemConfiguration {
        disagreeing_fields: Vec<MakeFilesystemParameterField>,
        disagreeing_device_table: Vec<DeviceTableDisagreement>,
    },
    /// 盘上读来的一个编号已到它那一格的顶""", """    CallerParametersDisagreeWithTheSelectedSystemConfiguration {
        disagreeing_fields: Vec<MakeFilesystemParameterField>,
        disagreeing_device_table: Vec<DeviceTableDisagreement>,
    },
    /// 挂着之后收盘表的入口（抬 F、准入抬 F、一次准入里再推一串、正常卸载）在参数、盘表与「可见」都核过之后，交进来的这几块盘落后于
    /// 现行那一版、又缺它的单元（[`DeviceBehindTheCurrentVersion`]，与可写挂载取号之前逐盘核的落后支同一判，「所选那一版」换成调用方持着的
    /// 现行那一版；D18（块里携带什么信息） 已定项 11「挂着之后收盘表的每个入口……核三样」，用户 2026-09-27 定）。不核这一判时，
    /// 同池一份旧快照的盘经这些入口照常收到本池的写、系统配置被轮换到「跟得上」，之后可写挂载的落后支就认不出它
    /// （代码三方 m2-closeout-code-r2「同池旧快照的盘经入口洗白」）。只读过每块盘的两个系统配置槽与落后那几块盘上现行那一版单元的落点，
    /// 在任何写之前返回（抬 F 与卸载在动分配器之前）：盘上逐字节不变。调用方要做的是换回这个池现行的盘，不是改参数或盘表的次序。
    DevicesBehindTheCurrentVersionAndMissingItsUnits {
        devices: Vec<DeviceBehindTheCurrentVersion>,
    },
    /// 盘上读来的一个编号已到它那一格的顶""")
edit(MT, """    ParametersOrDeviceTableDisagreeWithTheSelectedSystemConfiguration {
        disagreeing_fields: Vec<MakeFilesystemParameterField>,
        disagreeing_device_table: Vec<DeviceTableDisagreement>,
    },
}

impl From<CallerInputsDisagreeingWithTheDisk> for MountError {""", """    ParametersOrDeviceTableDisagreeWithTheSelectedSystemConfiguration {
        disagreeing_fields: Vec<MakeFilesystemParameterField>,
        disagreeing_device_table: Vec<DeviceTableDisagreement>,
    },
    /// 同 `MountError::DevicesBehindTheCurrentVersionAndMissingItsUnits`：清单不空。
    DevicesBehindTheCurrentVersionAndMissingItsUnits {
        devices: Vec<DeviceBehindTheCurrentVersion>,
    },
}

impl From<CallerInputsDisagreeingWithTheDisk> for MountError {""")
edit(MT, """            } => MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration {
                disagreeing_fields,
                disagreeing_device_table,
            },
        }
    }
}""", """            } => MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration {
                disagreeing_fields,
                disagreeing_device_table,
            },
            CallerInputsDisagreeingWithTheDisk::DevicesBehindTheCurrentVersionAndMissingItsUnits {
                devices,
            } => MountError::DevicesBehindTheCurrentVersionAndMissingItsUnits { devices },
        }
    }
}""")
edit(MT, """    NoSelfVerifiedSystemConfigurationOnTheDeviceHandedIn { identity_handed_in: DeviceIdentity },
}
""", """    NoSelfVerifiedSystemConfigurationOnTheDeviceHandedIn { identity_handed_in: DeviceIdentity },
}

/// 盘表把一块盘标成 `identity_handed_in`，它一份自证过的系统配置槽（本池的 fsid、整槽校验和过）里记的本盘设备号是
/// `own_device_number_on_disk`（[`own_device_numbers_on_disk_differing_from_the_identities_handed_in`]）。可写挂载与挂着之后收盘表的入口
/// 报成 [`DeviceTableDisagreement::OwnDeviceNumberDiffersFromTheIdentityHandedIn`]，会话报成
/// `mounted_session::UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable`（代码三方 m2-closeout-code-r2「盘体对调的盘表」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn {
    pub identity_handed_in: DeviceIdentity,
    pub own_device_number_on_disk: DeviceIdentity,
}

/// 挂着之后收盘表的入口在「可见」之后逐盘核出的一块落后于现行那一版、又缺它的单元的盘（[`devices_behind_the_current_version`]）：
/// 它两槽里自证过的系统配置世代号最大那一份的 (实例代号, journal tail) 是 `newest_system_configuration_journal_tail`，小于调用方持着的
/// 现行那一版那次发布末条记录的 jsn（`current_version_journal_position`），而现行那一版有单元在这块盘上它的落点读不出、或字节与这个进程
/// 验得过的那一份不同（`units_missing`，按角色与落点列出，次序同现行那一版的单元清单）。与可写挂载取号之前逐盘核的
/// [`SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndMissingItsUnits`] 同一判（[`units_of_the_version_missing_on_a_device_behind_it`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceBehindTheCurrentVersion {
    pub device: DeviceIdentity,
    pub newest_system_configuration_journal_tail: JournalSequenceNumber,
    pub current_version_journal_position: JournalSequenceNumber,
    pub units_missing: Vec<(TransactionUnit, SlotNumber)>,
}
""")
edit(MT, """        let newest_system_configuration_journal_tail = JournalSequenceNumber {
            instance_generation: newest_self_verified.quantities.journal_instance,
            counter: newest_self_verified.quantities.journal_tail,
        };
        if newest_system_configuration_journal_tail >= selected_version_journal_position {
            continue;
        }
        let units_missing: Vec<(TransactionUnit, SlotNumber)> = units
            .iter()
            .filter(|unit| !unit.bytes.is_empty())
            .filter(|unit| {
                !reader
                    .read(device, unit.slot.to_device_offset(), unit.bytes.len())
                    .is_some_and(|bytes_on_this_device| bytes_on_this_device == unit.bytes)
            })
            .map(|unit| (unit.identity, unit.slot))
            .collect();
        if !units_missing.is_empty() {
            devices_without.push(DeviceWithoutTheSelectedVersion {
                device,
                lacking:
                    SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndMissingItsUnits {
                        newest_system_configuration_journal_tail,
                        units_missing,
                    },
            });
        }
    }
    devices_without
}
""", open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "03_y2_helpers.rs.txt"), encoding="utf-8").read().rstrip("\n") + "\n")
edit(MT, """    let slot_spacing_in_bytes = u64::from(
        system_configuration
            .immutable
            .sizes
            .fixed_structure_slot_spacing,
    );
    for (identity_handed_in, _) in devices {
        let own_device_numbers_on_disk: BTreeSet<DeviceIdentity> =
            verified_system_configuration_slots(
                devices,
                *identity_handed_in,
                slot_spacing_in_bytes,
                &system_configuration.immutable.filesystem_identifier,
            )
            .into_iter()
            .map(|slot| slot.immutable.this_device)
            .collect();
        disagreements.extend(
            own_device_numbers_on_disk
                .into_iter()
                .filter(|own_device_number_on_disk| own_device_number_on_disk != identity_handed_in)
                .map(|own_device_number_on_disk| {
                    DeviceTableDisagreement::OwnDeviceNumberDiffersFromTheIdentityHandedIn {
                        identity_handed_in: *identity_handed_in,
                        own_device_number_on_disk,
                    }
                }),
        );
    }
    disagreements
}""", """    disagreements.extend(
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
    disagreements
}""")
edit(MT, """/// 前三项也报不出它（本盘设备号是逐份自证槽比的，一份都没有就一条都不比），所以另判这一项。
/// 只读盘，不写盘：每个入口在它的第一个写之前调它。
///
/// # Errors
/// 交进来的不对（[`CallerInputsDisagreeingWithTheDisk`] 的两种；不「可见」的盘报在第二种的 `disagreeing_device_table` 里）；
/// 系统配置择不出来（别的池的盘换进来，两块盘上各有一份 fsid 不同的自证槽，报在这里）。""", """/// 前三项也报不出它（本盘设备号是逐份自证槽比的，一份都没有就一条都不比），所以另判这一项。
/// 「可见」之后再核落后支：每块盘最新那份自证系统配置不落后于调用方持着的现行那一版（`current`）、或落后而现行那一版的单元都在
/// （[`devices_behind_the_current_version`]；D18（块里携带什么信息） 已定项 11「挂着之后收盘表的每个入口……核三样」，用户 2026-09-27 定，
/// 代码三方 m2-closeout-code-r2「同池旧快照的盘经入口洗白」）。三样里的本盘设备号在前面盘表那一判里已比过（[`device_table_disagreeing_with`]）。
/// 只读盘，不写盘：每个入口在它的第一个写之前调它。
///
/// # Errors
/// 交进来的不对（[`CallerInputsDisagreeingWithTheDisk`] 的三种；不「可见」的盘报在第二种的 `disagreeing_device_table` 里，
/// 落后又缺单元的盘报在第三种里）；系统配置择不出来（别的池的盘换进来，两块盘上各有一份 fsid 不同的自证槽，报在这里）。""")
edit(MT, """fn caller_inputs_agreeing_with_the_disk<Device: BlockDevice>(
    caller_parameters: &MakeFilesystemParameters,
    devices: &Vec<(DeviceIdentity, Device)>,
) -> Result<CallerInputsAgreeingWithTheDisk, CallerInputsCheckRefused> {""", """fn caller_inputs_agreeing_with_the_disk<Device: BlockDevice>(
    caller_parameters: &MakeFilesystemParameters,
    devices: &Vec<(DeviceIdentity, Device)>,
    current: &TransactionOutput,
) -> Result<CallerInputsAgreeingWithTheDisk, CallerInputsCheckRefused> {""")
edit(MT, """                disagreeing_device_table: devices_not_visible,
            },
        ));
    }
    Ok(CallerInputsAgreeingWithTheDisk {""", """                disagreeing_device_table: devices_not_visible,
            },
        ));
    }
    let devices_behind =
        devices_behind_the_current_version(devices, &parameters_of_the_pool, current);
    if !devices_behind.is_empty() {
        return Err(CallerInputsCheckRefused::Disagreeing(
            CallerInputsDisagreeingWithTheDisk::DevicesBehindTheCurrentVersionAndMissingItsUnits {
                devices: devices_behind,
            },
        ));
    }
    Ok(CallerInputsAgreeingWithTheDisk {""")
edit(MT, "caller_inputs_agreeing_with_the_disk(parameters, devices)", "caller_inputs_agreeing_with_the_disk(parameters, devices, current)", count=4)
edit(MT, """        | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
        // txg 到顶不是空间不够；""", """        | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
        | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnits { .. }
        // txg 到顶不是空间不够；""")
edit(MT, """/// 盘表里有身份交了不止一次（`DeviceIdentitiesHandedInMoreThanOnce`）、调用方的参数或盘表（盘数与 `devs`、每块盘的本盘设备号）
/// 与盘上择到的系统配置不一致（`CallerParametersDisagreeWithTheSelectedSystemConfiguration`），都在回收、影子账与任何写之前；""", """/// 盘表里有身份交了不止一次（`DeviceIdentitiesHandedInMoreThanOnce`）、调用方的参数或盘表（盘数与 `devs`、每块盘的本盘设备号）
/// 与盘上择到的系统配置不一致、或有盘不「可见」（`CallerParametersDisagreeWithTheSelectedSystemConfiguration`）、有盘落后于现行那一版又缺它的单元
/// （`DevicesBehindTheCurrentVersionAndMissingItsUnits`），都在回收、影子账与任何写之前；""")
edit(MT, """    /// （D22（单元原子性怎么合成） 已定项 26 第一档）。在任何写之前（重复身份在任何读之前）拒，盘上逐字节不变。
    CallerInputsDisagreeWithTheDisk(CallerInputsDisagreeingWithTheDisk),""", """    /// （D22（单元原子性怎么合成） 已定项 26 第一档）。有盘不「可见」、或落后于现行那一版又缺它的单元（D18（块里携带什么信息） 已定项 11
    /// 「挂着之后收盘表的每个入口……核三样」）也报在这里。在任何写之前（重复身份在任何读之前）拒，盘上逐字节不变。
    CallerInputsDisagreeWithTheDisk(CallerInputsDisagreeingWithTheDisk),""")

MS = "crates/singlefs-core/src/mounted_session.rs"
edit(MS, """//! 对不上在任何读写之前拒（实审 A1b Q2、Q3）；身份对得上，还要每块盘「可见」（两个系统配置槽里至少一份本池自证过的），
//! 有一块不可见就在任何写之前拒（代码三方 m2-closeout-code-r1 Z3-A 乙：只比身份时，换上去的一块空盘照样收到本池的写）。""", """//! 对不上在任何读写之前拒（实审 A1b Q2、Q3）；身份对得上，还要每块盘「可见」（两个系统配置槽里至少一份本池自证过的），
//! 有一块不可见就在任何写之前拒（代码三方 m2-closeout-code-r1 Z3-A 乙：只比身份时，换上去的一块空盘照样收到本池的写）；
//! 「可见」之后再核每块盘自证槽里的本盘设备号是盘表给它的身份、这块盘不落后于现行那一版又缺它的单元（D18（块里携带什么信息） 已定项 11
//! 「挂着之后收盘表的每个入口……核三样」，用户 2026-09-27 定；代码三方 m2-closeout-code-r2「盘体对调的盘表」「同池旧快照的盘经入口洗白」）。""")
edit(MS, """use crate::mount::{
    devices_without_a_self_verified_system_configuration,
    push_one_floor_raise_within_the_admission_budget, FloorRaisePushedWithinTheAdmissionBudget,
    FloorRaiseStop, MountError, MountOutput, Mounted, ParametersAndDeviceTableOfTheMount,
    RaisedFloor, ShadowLedger,
};""", """use crate::mount::{
    devices_behind_the_current_version, devices_without_a_self_verified_system_configuration,
    own_device_numbers_on_disk_differing_from_the_identities_handed_in,
    push_one_floor_raise_within_the_admission_budget, DeviceBehindTheCurrentVersion,
    FloorRaisePushedWithinTheAdmissionBudget, FloorRaiseStop, MountError, MountOutput, Mounted,
    OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn, ParametersAndDeviceTableOfTheMount,
    RaisedFloor, ShadowLedger,
};""")
edit(MS, """    DevicesWithoutASelfVerifiedSystemConfiguration { devices: Vec<DeviceIdentity> },
    /// 发布被拒、不是空间不够的那两种""", """    DevicesWithoutASelfVerifiedSystemConfiguration { devices: Vec<DeviceIdentity> },
    /// 盘表的身份与这次挂载核过的那一份相同、每块盘都「可见」，这几块盘（按交进来的次序）自证过的系统配置槽里记的本盘设备号却不是
    /// 盘表给它的身份——盘体对调了、或插错了位置（与可写挂载、挂着之后收盘表的入口同一判，
    /// `mount::own_device_numbers_on_disk_differing_from_the_identities_handed_in`）。只比身份时，发布照样做成、系统配置轮换把错的本盘设备号
    /// 写进两块盘、根写到不归它的那块盘上，之后这个池可写挂不上（代码三方 m2-closeout-code-r2「盘体对调的盘表」，用户 2026-09-27 定）。
    /// 读过每块盘的两个系统配置槽，一个写都没发：盘上逐字节不变、会话不动。调用方要做的是把盘体放回各自的身份再发。
    OwnDeviceNumbersDifferFromTheDeviceTable {
        disagreements: Vec<OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn>,
    },
    /// 盘表的身份对得上、每块盘都「可见」、本盘设备号也对得上，这几块盘（按交进来的次序）却落后于会话的现行那一版、又缺它的单元：
    /// 同池一份旧快照被换了上去（与可写挂载取号之前逐盘核的落后支、挂着之后收盘表的入口同一判，`mount::devices_behind_the_current_version`）。
    /// 不核这一判时发布照样做成、旧快照的系统配置被轮换到「跟得上」，之后可写挂载的落后支就认不出它、池级 checker 红 I-2.1
    /// （代码三方 m2-closeout-code-r2「同池旧快照的盘经入口洗白」，用户 2026-09-27 定）。只剩一槽自证、那一槽停在上一次轮换的盘
    /// 落后而单元都在，不在这里。读过每块盘的两个系统配置槽与落后那几块上现行那一版单元的落点，一个写都没发：盘上逐字节不变、会话不动。
    /// 调用方要做的是换回这个池现行的盘。
    DevicesBehindTheCurrentVersionAndMissingItsUnits {
        devices: Vec<DeviceBehindTheCurrentVersion>,
    },
    /// 发布被拒、不是空间不够的那两种""")
edit(MS, """    /// 交进来的盘表先与它逐项比（实审 A1b Q2、Q3），再读每块盘的两个系统配置槽核它「可见」（Z3-A 乙）。
    /// 「可见」在进循环之前核一次：一次调用里盘表借着、换不了盘；循环里推的每一串抬 F 在它自己的第一个写之前再核一遍
    /// （`mount::push_one_floor_raise_within_the_admission_budget` 经 `caller_inputs_agreeing_with_the_disk`），
    /// 所以这次调用发出的每一个写之前都核过。""", """    /// 交进来的盘表先与它逐项比（实审 A1b Q2、Q3），再读每块盘的两个系统配置槽核它「可见」（Z3-A 乙），再核本盘设备号与落后支
    /// （D18（块里携带什么信息） 已定项 11「挂着之后收盘表的每个入口……核三样」：「每次发布」按「每次用户改动」读）。
    /// 三样在进循环之前核一次：一次调用里盘表借着、换不了盘；循环里推的每一串抬 F 在它自己的第一个写之前再核一遍三样
    /// （`mount::push_one_floor_raise_within_the_admission_budget` 经 `caller_inputs_agreeing_with_the_disk`），
    /// 所以这次调用发出的每一个写之前都核过。""")
edit(MS, """    /// # Errors
    /// [`UserChangeRefused`] 的六种。""", """    /// # Errors
    /// [`UserChangeRefused`] 的八种。""")
edit(MS, """        if !devices_not_visible.is_empty() {
            return Err(
                UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration {
                    devices: devices_not_visible,
                },
            );
        }""", """        if !devices_not_visible.is_empty() {
            return Err(
                UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration {
                    devices: devices_not_visible,
                },
            );
        }
        let own_device_numbers_differing =
            own_device_numbers_on_disk_differing_from_the_identities_handed_in(
                devices.as_slice(),
                parameters,
            );
        if !own_device_numbers_differing.is_empty() {
            return Err(UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable {
                disagreements: own_device_numbers_differing,
            });
        }
        let devices_behind =
            devices_behind_the_current_version(devices.as_slice(), parameters, current_file_version);
        if !devices_behind.is_empty() {
            return Err(
                UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnits {
                    devices: devices_behind,
                },
            );
        }""")
print("03_y2 done")
