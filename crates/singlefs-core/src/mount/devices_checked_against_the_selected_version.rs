//! 取号之前的逐盘核与挂着之后收盘表入口的落后支：每块盘带不带所选那一版（系统配置、单元、根），
//! 两槽全废的盘过不过补写闸，落后于那一版的盘缺的是什么。

use super::*;

impl SelectedVersionCheckedOnEachDevice<'_> {
    pub(super) fn version(&self) -> RollbackTarget {
        RollbackTarget {
            instance: self.root.instance,
            checkpoint_txg: self.root.checkpoint_txg,
        }
    }
}

/// 逐盘核读一个系统配置槽那一刻的结论（改法丙，代码轮 `research/prompts/m3-close-b19-code-r3-main-verification.md` 第三节，用户 2026-10-29 选）。
/// 封闭集合，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SystemConfigurationSlotConclusionWhenChecked {
    /// 读得出，读的一方没报自证不过：本池自证过的，或 magic 与校验和都过而 fsid 是别的池、布局位不认识、每区槽数越界的
    /// （`recovery::system_configuration_slots_of_one_device_read_once` 对这几样都不报读坏，只进内存那一趟取号读它们也不进读坏账）。
    ReadWithoutFailingSelfVerification,
    /// 读得出、读的一方报了自证不过（撕裂、清零）。
    ReadFailingSelfVerification,
    /// 设备报读错。
    ReportingAReadError,
}

impl<Reader: PoolReader + ?Sized> PoolReader
    for SystemConfigurationSlotReadsRecordedByTheCheck<'_, Reader>
{
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.reader.device_identities()
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.reader.device_size_in_bytes(device)
    }
    fn physical_block_size_probed_on_the_device(
        &self,
        device: DeviceIdentity,
    ) -> Option<PhysicalBlockSizeInBytes> {
        self.reader.physical_block_size_probed_on_the_device(device)
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        let bytes = self.reader.read(device, offset, length);
        let conclusion = match bytes {
            Some(_) => {
                SystemConfigurationSlotConclusionWhenChecked::ReadWithoutFailingSelfVerification
            }
            None => SystemConfigurationSlotConclusionWhenChecked::ReportingAReadError,
        };
        self.slots_read
            .borrow_mut()
            .push(SystemConfigurationSlotAsChecked {
                device,
                offset,
                length_in_bytes: u64::try_from(length).expect("一个系统配置槽的读长装得进 u64"),
                conclusion,
            });
        bytes
    }
    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.reader
            .journal_record_offsets_hint(device, ring_start, ring_bytes)
    }
    fn reread_after_a_bad_read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        self.reader.reread_after_a_bad_read(device, offset, length)
    }
    fn keep_the_self_verified_reread(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
    ) {
        self.reader
            .keep_the_self_verified_reread(device, offset, bytes);
    }
    fn note_that_the_bytes_read_self_verify(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
    ) {
        self.reader
            .note_that_the_bytes_read_self_verify(device, offset, bytes);
    }
    fn note_that_the_bytes_read_fail_self_verification(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length_in_bytes: usize,
    ) {
        self.reader.note_that_the_bytes_read_fail_self_verification(
            device,
            offset,
            length_in_bytes,
        );
        let length_in_bytes =
            u64::try_from(length_in_bytes).expect("一个系统配置槽的读长装得进 u64");
        let mut slots_read = self.slots_read.borrow_mut();
        let slot_read = slots_read
            .iter_mut()
            .rev()
            .find(|slot| {
                slot.device == device
                    && slot.offset == offset
                    && slot.length_in_bytes == length_in_bytes
            })
            .expect("读的一方报自证不过的那一读刚经这一层读过（system_configuration_slots_of_one_device_read_once 读完一槽当场判）");
        slot_read.conclusion =
            SystemConfigurationSlotConclusionWhenChecked::ReadFailingSelfVerification;
    }
}

/// 两槽一份本池自证过的系统配置都没有的盘为什么没被取号补写（丙'，C874（补写那一支进不了崩溃测试、运行时看不出），判决
/// `research/prompts/m3-close-b19-core-r1-main-verification.md` 第二节 R4「补写闸没过的原因进挂载错误成员的载荷」）：闸前那一判（两槽里有一槽设备报读错）
/// 或补写闸五件里第一件没过的（[`RewriteGateNotMet`]）。拒因的成员表不加成员，原因装在
/// [`SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition`] 的载荷里。封闭集合，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceNotRewrittenByTheAcquisition {
    /// 闸前：两槽里有一槽设备报读错（读错的槽里装的是什么看不见、可能是一份更新的见证，不补写，S-辨同一个口径）。
    SystemConfigurationSlotReportsAReadError,
    /// 两槽都读得出、都自证不过，补写闸有一件没过。
    RewriteGateNotMet(RewriteGateNotMet),
}

/// 两槽全废的盘过不了丙' 补写闸的是哪一件（[`rewrite_gate_not_met_by_a_device_without_any_self_verified_system_configuration`]）。
/// 调用方拒成 [`SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition`]、装在它的载荷里报出来
/// （[`DeviceNotRewrittenByTheAcquisition::RewriteGateNotMet`]）。封闭集合，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RewriteGateNotMet {
    /// 闸一：所选那一版的单元清单是空的，或有一个在这块盘上读不出、字节与验得过的那一份不同，或没有验得过的一份可比。
    UnitsOfTheSelectedVersionNotAllOnTheDevice,
    /// 闸五：这块盘的根环或 journal 环里有一槽设备报读错（读错的槽里可能藏着更新的根或记录，看不见不当没有）。
    RingSlotOnTheDeviceReportsAReadError,
    /// 闸三：这块盘的根环或 journal 环里有比所选那一版新的根或记录（按择根的序：checkpoint_txg 为主、实例代号破平局）。
    RootOrRecordNewerThanTheSelectedVersionInItsRings,
    /// 闸四：这块盘单元区里有写序实例大于所选那一版实例的单元（同 fsid 的克隆盘在克隆那边只写了单元就崩的那一形）。
    UnitWrittenByALaterInstanceInItsUnitArea,
    /// 闸二：所选那一版的根写进了根环、住在这块盘上，那一槽读一次重读一次都不是它（落后支缺根那一半的同一判）。
    RootOfTheSelectedVersionNotInItsRingSlot,
}

/// 可写挂载准入的逐盘核（D2（RAID 条带策略） 已定项 13、D18（块里携带什么信息） 已定项 11）：交进来的每块盘带不带所选那一版，
/// 交回不带的那几块与各自缺的（[`SelectedVersionLackingOnDevice`]），连同两槽全废而过了补写闸的盘（丙'）。只读盘，不写。
///
/// 每块盘先看它两槽里本池自证过的系统配置。一份都没有：照丙'（用户 2026-10-19 定 S4「开例外，四件闸」，2026-10-20 再加单元区扫描与环槽读错两件；
/// 三方判决 `research/prompts/m3-close-sysconf-r3-main-verification.md` 第二节 U4 与第三节）过补写闸
/// （[`rewrite_gate_not_met_by_a_device_without_any_self_verified_system_configuration`]），都过就记进补写的盘、交给取号补写；有一件不过交 `Err((那块盘, 那一件))`，
/// 由调用方按 C331（择根倒挂压过已确认的写） 取甲的拒因报（用户 2026-09-28 定：这一支并进甲的拒因）。
/// 有自证过的：世代号最大的那一份按落后支那一判（[`what_a_device_behind_the_version_lacks`]，与挂着之后收盘表的入口同一判）。
/// 每块盘两槽读的那一刻各是什么结论一并交回（[`SystemConfigurationSlotAsChecked`]，改法丙）：读两槽的读者外面套一层
/// [`SystemConfigurationSlotReadsRecordedByTheCheck`]，判法照旧是 `system_configuration_slots_of_one_device_read_once` 那一份，这里不另读、不另判。
///
/// 循环：外层按交进来的盘（轮数 = 盘数），内层在落后支与补写闸里；跨轮只往交回的三张表里追加（槽的结论在读完两槽、判这块盘之前就追加）。
/// 外层的提前出口两个：两槽全废又过不了补写闸（整个核交 `Err`）、不落后或落后而不缺（这块盘判完了、看下一块，不记）。
///
/// # Errors
/// 某块盘两槽里一份本池自证过的系统配置都读不出、又没被补写 ⇒ `Err((那块盘, 为什么没补写))`：两槽里有一槽设备报读错，或补写闸第一件没过的。
pub(super) fn devices_without_the_selected_version<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    selected_version: SelectedVersionCheckedOnEachDevice<'_>,
) -> Result<DevicesCheckedBeforeTheAcquisition, (DeviceIdentity, DeviceNotRewrittenByTheAcquisition)>
{
    let parameters_of_the_pool = parameters_of_the_system_configuration(system_configuration);
    let mut devices_without = Vec::new();
    let mut devices_rewritten_by_the_acquisition = Vec::new();
    let mut system_configuration_slots_as_checked = Vec::new();
    for device in reader.device_identities() {
        let slot_reads_recorded = SystemConfigurationSlotReadsRecordedByTheCheck {
            reader,
            slots_read: RefCell::new(Vec::new()),
        };
        let SystemConfigurationSlotsOfOneDevice {
            self_verified,
            slots_reporting_a_read_error,
            ..
        } = system_configuration_slots_of_one_device_read_once(
            &slot_reads_recorded,
            device,
            u64::from(parameters_of_the_pool.geometry.fixed_structure_slot_spacing),
            &parameters_of_the_pool.filesystem_identifier,
        );
        system_configuration_slots_as_checked.extend(slot_reads_recorded.slots_read.into_inner());
        let newest_self_verified = self_verified
            .into_iter()
            .max_by_key(|slot| slot.quantities.slot_generation);
        let Some(newest_self_verified) = newest_self_verified else {
            // 两槽「全废」只认这一次读得出、自证不过的（撕裂、清零、别的池的）：设备报读错的槽里装的是什么看不见、可能是一份更新的见证，
            // 与 S-辨（一槽撕裂与设备读不出分开）同一个口径，不补写，照今天拒（读阶段判完之后系统配置又读不出）。
            if slots_reporting_a_read_error > 0 {
                return Err((
                    device,
                    DeviceNotRewrittenByTheAcquisition::SystemConfigurationSlotReportsAReadError,
                ));
            }
            match rewrite_gate_not_met_by_a_device_without_any_self_verified_system_configuration(
                reader,
                system_configuration,
                &parameters_of_the_pool,
                device,
                selected_version,
            ) {
                None => {
                    devices_rewritten_by_the_acquisition.push(device);
                    continue;
                }
                Some(gate_not_met) => {
                    return Err((
                        device,
                        DeviceNotRewrittenByTheAcquisition::RewriteGateNotMet(gate_not_met),
                    ));
                }
            }
        };
        let Some((newest_system_configuration_journal_tail, lacking)) =
            what_a_device_behind_the_version_lacks(
                reader,
                &parameters_of_the_pool,
                device,
                &newest_self_verified,
                VersionJudgedBehindOrNot {
                    version: selected_version.version(),
                    journal_position: selected_version.journal_position,
                    units: selected_version.units,
                    root_in_the_ring: selected_version.root_in_the_ring,
                },
            )
        else {
            continue;
        };
        let lacking = match lacking {
            VersionLackingOnADeviceBehindIt::UnitsMissing(units_missing) => {
                SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndMissingItsUnits {
                    newest_system_configuration_journal_tail,
                    units_missing,
                }
            }
            VersionLackingOnADeviceBehindIt::RootNotInItsRingSlot(root_not_in_its_ring_slot) => {
                SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndItsRootNotInItsRingSlot {
                    newest_system_configuration_journal_tail,
                    root_not_in_its_ring_slot,
                }
            }
        };
        devices_without.push(DeviceWithoutTheSelectedVersion { device, lacking });
    }
    Ok(DevicesCheckedBeforeTheAcquisition {
        devices_without,
        devices_rewritten_by_the_acquisition,
        system_configuration_slots_as_checked,
    })
}

/// 丙' 补写闸（[`devices_without_the_selected_version`]）：两槽一份本池自证过的系统配置都没有的盘 `device`，五件都过交回 `None`、取号补写它；
/// 按下面的次序判，第一件不过的交回（[`RewriteGateNotMet`]）。第六件「补写失败按甲' 每盘回卷一次」在取号写里（`transaction::write_the_witnessed_acquisition`）。
/// 调用方只拿两槽都读得出、都自证不过的盘来过闸（两槽「全废」；有一槽设备报读错的不在这里判，[`devices_without_the_selected_version`]）。
///
/// 1. 本盘带所选那一版的全部单元：清单非空、每一个在这块盘上的落点读一次、字节与验得过的那一份相同（没有验得过的一份可比的单元算不带）。
/// 2. 一遍读这块盘上的根环槽与 journal 环槽（[`rings_of_the_device_against_the_selected_version`]）：有一槽设备报读错就不补写（闸五，零轮改法，用户 2026-10-20 采）；
///    有比所选那一版新的根或记录就不补写（闸三）。
/// 3. 扫这块盘整个单元区（`system_configuration_repair::write_order_and_birth_in_the_unit_area_of_the_device`）：有写序实例大于所选那一版实例的单元就不补写（闸四，零轮改法，用户 2026-10-20 采）。
/// 4. 根槽那一半不缺（闸二）：同落后支缺根那一半（[`root_of_the_version_not_in_its_ring_slot_on_the_device`]，读一次、不是它重读一次）。
///    排在最后：它是唯一会重读的一件，前几件不过就不必读。
///
/// 这块盘是谁只由调用方的盘表说了算（两槽全废，本盘设备号读不到）：别的盘都按盘表核过本盘设备号。只读盘：单元读一次，两个环这块盘上的槽各读一次，
/// 单元区每个槽读一次（不全 0 的再读一次 32 KiB），根槽至多两次。
fn rewrite_gate_not_met_by_a_device_without_any_self_verified_system_configuration<
    Reader: PoolReader + ?Sized,
>(
    reader: &Reader,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    parameters_of_the_pool: &MakeFilesystemParameters,
    device: DeviceIdentity,
    selected_version: SelectedVersionCheckedOnEachDevice<'_>,
) -> Option<RewriteGateNotMet> {
    let every_unit_of_the_version_is_on_the_device = !selected_version.units.is_empty()
        && selected_version.units.iter().all(|unit| {
            !unit.bytes.is_empty()
                && reader
                    .read(device, unit.slot.to_device_offset(), unit.bytes.len())
                    .is_some_and(|bytes_on_this_device| bytes_on_this_device[..] == unit.bytes[..])
        });
    if !every_unit_of_the_version_is_on_the_device {
        return Some(RewriteGateNotMet::UnitsOfTheSelectedVersionNotAllOnTheDevice);
    }
    match rings_of_the_device_against_the_selected_version(
        reader,
        system_configuration,
        device,
        selected_version.root,
    ) {
        RingsOfTheDeviceAgainstTheSelectedVersion::SomeSlotReportsAReadError => {
            return Some(RewriteGateNotMet::RingSlotOnTheDeviceReportsAReadError);
        }
        RingsOfTheDeviceAgainstTheSelectedVersion::HoldARootOrRecordNewerThanTheSelectedVersion => {
            return Some(RewriteGateNotMet::RootOrRecordNewerThanTheSelectedVersionInItsRings);
        }
        RingsOfTheDeviceAgainstTheSelectedVersion::NothingNewerAndEverySlotRead => {}
    }
    let unit_area =
        crate::system_configuration_repair::write_order_and_birth_in_the_unit_area_of_the_device(
            reader,
            device,
            unit_area_start_of_the_chosen_system_configuration(system_configuration).slot(),
            &parameters_of_the_pool.filesystem_identifier,
        );
    if unit_area
        .highest_write_order_instance
        .is_some_and(|instance| instance > selected_version.root.instance)
    {
        return Some(RewriteGateNotMet::UnitWrittenByALaterInstanceInItsUnitArea);
    }
    if root_of_the_version_not_in_its_ring_slot_on_the_device(
        reader,
        parameters_of_the_pool,
        device,
        selected_version.root_in_the_ring,
    )
    .is_some()
    {
        return Some(RewriteGateNotMet::RootOfTheSelectedVersionNotInItsRingSlot);
    }
    None
}

/// 一块盘的根环槽与 journal 环槽对所选那一版是什么样（[`rings_of_the_device_against_the_selected_version`]）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RingsOfTheDeviceAgainstTheSelectedVersion {
    /// 有一槽设备报读错（先于下一种报：读错的槽里可能藏着更新的）。
    SomeSlotReportsAReadError,
    /// 每槽都读成，有一条根或记录比所选那一版新。
    HoldARootOrRecordNewerThanTheSelectedVersion,
    /// 每槽都读成，没有比所选那一版新的。
    NothingNewerAndEverySlotRead,
}

/// 读这块盘上的根环槽（区域归属落在它上面的那几个区域的全部槽）与 journal 环槽（读者给得出提示就按提示，给不出读整环，与扫环同一个取法），
/// 每槽读一次：设备报读错的记下；自证过的根、解得开的记录按择根的序（checkpoint_txg 为主、实例代号破平局）比所选那一版的根，大的记下
/// （丙' 闸三与闸五；「比所选那一版新的根与记录」是这一份读法，零轮）。读坏而自证不过、解不开的槽不算读错，也分不出它从没写过，只能接受。
///
/// 循环：先按根环槽（轮数 = R × S），再按 journal 环槽（轮数 = 提示给的槽数或环槽数）；跨轮只置两个记号，没有提前出口。
fn rings_of_the_device_against_the_selected_version<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    device: DeviceIdentity,
    selected_version_root: &RootRecord,
) -> RingsOfTheDeviceAgainstTheSelectedVersion {
    let immutable = &system_configuration.immutable;
    let selected_version_in_the_order_of_the_root_choice = (
        selected_version_root.checkpoint_txg,
        selected_version_root.instance,
    );
    let mut some_slot_reports_a_read_error = false;
    let mut holds_something_newer = false;
    for ring_slot in every_root_ring_slot(&immutable.sizes) {
        let device_of_the_region =
            immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号小于 3")];
        if device_of_the_region != device {
            continue;
        }
        match read_root_ring_slot(
            reader,
            &immutable.region_devices,
            &immutable.sizes,
            &immutable.filesystem_identifier,
            ring_slot,
        ) {
            RootRingSlotReading::SelfVerified(root) => {
                if (root.checkpoint_txg, root.instance)
                    > selected_version_in_the_order_of_the_root_choice
                {
                    holds_something_newer = true;
                }
            }
            RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => {
                some_slot_reports_a_read_error = true;
            }
            RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified) => {}
        }
    }
    let ring_start = crate::address::DeviceOffsetInBytes(
        singlefs_format::JOURNAL_RING_START_SLOT * singlefs_format::SLOT_BYTES,
    );
    let ring_bytes = immutable.sizes.journal_ring_bytes;
    let record_bytes =
        usize::try_from(singlefs_format::JOURNAL_RECORD_BYTES).expect("一条记录 4096 字节");
    let ring_bytes_inside_the_device = reader
        .device_size_in_bytes(device)
        .map_or(0, |device_bytes| {
            ring_bytes.min(device_bytes.saturating_sub(ring_start.0))
        });
    let record_offsets = reader
        .journal_record_offsets_hint(device, ring_start, ring_bytes)
        .unwrap_or_else(|| {
            (0..ring_bytes_inside_the_device / singlefs_format::JOURNAL_RECORD_BYTES)
                .map(|record_index| {
                    crate::address::DeviceOffsetInBytes(
                        ring_start.0 + record_index * singlefs_format::JOURNAL_RECORD_BYTES,
                    )
                })
                .collect()
        });
    let filesystem_identifier_in_records =
        crate::unit::unit_filesystem_identifier(&immutable.filesystem_identifier);
    for record_offset in record_offsets {
        match reader.read(device, record_offset, record_bytes) {
            None => some_slot_reports_a_read_error = true,
            Some(bytes) => {
                if crate::journal::JournalRecord::parse(&bytes, filesystem_identifier_in_records)
                    .is_some_and(|record| {
                        (record.checkpoint_txg, record.instance)
                            > selected_version_in_the_order_of_the_root_choice
                    })
                {
                    holds_something_newer = true;
                }
            }
        }
    }
    match (some_slot_reports_a_read_error, holds_something_newer) {
        (true, _) => RingsOfTheDeviceAgainstTheSelectedVersion::SomeSlotReportsAReadError,
        (false, true) => {
            RingsOfTheDeviceAgainstTheSelectedVersion::HoldARootOrRecordNewerThanTheSelectedVersion
        }
        (false, false) => RingsOfTheDeviceAgainstTheSelectedVersion::NothingNewerAndEverySlotRead,
    }
}

/// 读阶段的见证读判完之后，逐盘核或取号那一遍又读到判不出（R2）或某块盘一份自证过的系统配置都没有（两次读之间的瞬时读错就够）：
/// 拒可写，报 [`MountError::SystemConfigurationUnreadableAfterTheReadStage`]，带读阶段往下走用的那一遍读数与 `unreadable`。
/// 不并进读阶段判据 N-配置 那一拒（C664（挂载拒绝合法复算把两种拒因报成一个））。一个字节都没写。
pub(super) fn refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(
    read_stage: ReadStageSettled,
    unreadable: SystemConfigurationUnreadableWhen,
) -> MountError {
    let settled = match read_stage {
        ReadStageSettled::OnTheFirstRead { first_read } => first_read,
        ReadStageSettled::OnTheOneReread { reread, .. } => reread,
    };
    MountError::SystemConfigurationUnreadableAfterTheReadStage(Box::new(
        SystemConfigurationUnreadableAfterTheReadStage {
            read_stage: settled,
            unreadable,
        },
    ))
}

/// 落后支要判的那一版（[`what_a_device_behind_the_version_lacks`]）：它的 (实例代号, checkpoint_txg)、那次发布末条记录的 jsn、它的单元、它的根写没写进根环。
#[derive(Clone, Copy)]
struct VersionJudgedBehindOrNot<'version> {
    version: RollbackTarget,
    journal_position: JournalSequenceNumber,
    units: &'version [PublishedUnit],
    root_in_the_ring: RootOfTheVersionInTheRing,
}

/// 落后支缺单元那一半（[`what_a_device_behind_the_version_lacks`] 先判它）：`newest_self_verified` 是这块盘两槽里本池自证过、世代号最大的那一份系统配置。
/// 这一半算不算落后（乙'，三方判决 `research/prompts/m3-close-sysconf-r3-main-verification.md` 第二节 U3 与第四节「落地」，C593（落后且缺一份单元的盘被永久拒可写））：
/// 它的 (实例代号, journal tail) 不落后于那一版那次发布末条记录的 jsn，或这块盘自己的 journal 环槽里有那一版的末条记录
/// （[`last_record_of_the_version_in_its_ring_slot_on_the_device`]），就不算落后、交回 `None`（不读它的单元）。
/// 算落后的再逐个读 `units` 在这块盘上的落点，读不出或字节与验得过的那一份不同的，只有落在 (tail, 那一版] 那个窗口里记录点名的落点上才算缺
/// （D18（块里携带什么信息） 已定项 11）：单元写在记录之前那道屏障之前、轮换写在屏障之后，
/// 这块盘收过 tail 及之前每次发布的单元，窗口之外缺的是落盘之后坏了的一份。窗口取不到（[`slots_named_by_the_records_after_the_tail`] 交回 `None`）时全部算缺。
/// 算缺的一个都没有也交回 `None`（只落后、单元都在：根落盘之后、轮换之前崩，或只剩一槽自证、那一槽停在上一次轮换；或只有窗口之外坏了的那几份）；有才交回 (那份的 tail, 缺的单元)。
/// 窗口之外坏了的那几份各按读到只坏一份报给读者（[`PoolReader::note_that_a_unit_was_read_with_one_copy_failing`]，D4（校验和位置） 已定项 4）：
/// 第二条位置条目上那一份只有这里看得见，读单元读到第一条就停。挂载与会话改动的入口经数只坏一份的那一层交读者，记进别的读那一份账，下一次用户改动发成之后另发的那一批换下它。
/// 字节是空的单元（提示与映射都读不出的数据单元，D19（块指针的结构与宽度预算） 已定项 5）没有验得过的一份可比，跳过。只读盘。
///
/// 循环：按 `units`（轮数 = 这一版的单元数）筛一遍，再按筛出来的单元报（轮数不超过这一版的单元数），跨轮只往清单里追加，没有提前出口。
fn units_of_the_version_missing_on_a_device_behind_it<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    parameters_of_the_pool: &MakeFilesystemParameters,
    device: DeviceIdentity,
    newest_self_verified: &crate::system_configuration::SystemConfiguration,
    version: VersionJudgedBehindOrNot<'_>,
) -> Option<(JournalSequenceNumber, Vec<(TransactionUnit, SlotNumber)>)> {
    let newest_system_configuration_journal_tail = JournalSequenceNumber {
        instance_generation: newest_self_verified.quantities.journal_instance,
        counter: newest_self_verified.quantities.journal_tail,
    };
    let version_journal_position = version.journal_position;
    if newest_system_configuration_journal_tail >= version_journal_position {
        return None;
    }
    if last_record_of_the_version_in_its_ring_slot_on_the_device(
        reader,
        parameters_of_the_pool,
        device,
        version.version,
        version_journal_position,
    ) {
        return None;
    }
    let units_differing_on_this_device: Vec<&PublishedUnit> = version
        .units
        .iter()
        .filter(|unit| !unit.bytes.is_empty())
        .filter(|unit| {
            !reader
                .read(device, unit.slot.to_device_offset(), unit.bytes.len())
                .is_some_and(|bytes_on_this_device| bytes_on_this_device[..] == unit.bytes[..])
        })
        .collect();
    let slots_named_after_the_tail = slots_named_by_the_records_after_the_tail(
        reader,
        parameters_of_the_pool,
        newest_system_configuration_journal_tail,
        version_journal_position,
    );
    let (units_counted_as_missing, units_with_only_this_copy_damaged): (
        Vec<&PublishedUnit>,
        Vec<&PublishedUnit>,
    ) = units_differing_on_this_device
        .into_iter()
        .partition(|unit| {
            slots_named_after_the_tail
                .as_ref()
                .is_none_or(|slots_named| slots_named.contains(&unit.slot))
        });
    let identities_of_the_devices = reader.device_identities();
    for unit in units_with_only_this_copy_damaged {
        reader.note_that_a_unit_was_read_with_one_copy_failing(
            &crate::make_filesystem::location_entries(
                &identities_of_the_devices,
                unit.slot,
                &unit.bytes,
            ),
        );
    }
    let units_missing: Vec<(TransactionUnit, SlotNumber)> = units_counted_as_missing
        .into_iter()
        .map(|unit| (unit.identity, unit.slot))
        .collect();
    (!units_missing.is_empty()).then_some((newest_system_configuration_journal_tail, units_missing))
}

/// (这块盘的 tail, 那一版那次发布末条记录] 那个窗口里每条记录点名的槽（第一版两盘同槽，D2（RAID 条带策略） 已定项 10，点名的槽就是这块盘上的落点）。
/// 交回 `None` 的两种（调用方全部算缺）：tail 的实例代号不是那一版的（跨实例的窗口不放行）；窗口里有一个计数器在哪块盘的环槽里都解不出这个计数器的记录
/// （读不出、自证不过，或环转过一圈已被别的计数器盖掉：窗口比环长时第一个计数器就走这一支，读的槽数因此不超过环的槽数乘盘数）。
/// 窗口里的记录按计数器认、实例代号不限：tail 与那一版同实例时，夹在中间的计数器可以住着上一个实例没来得及轮换的那条记录（取号写的 tail 是取号那一刻的见证值，新实例的计数器接在环里最大的计数器后面）。
/// 同一个计数器几块盘上各有一条（分叉的盘）时取并集，多点名只多算缺、不多放行。调用方保证 tail 小于那一版的 jsn。只读盘。
///
/// 循环：外层按窗口里的计数器（轮数 = 末条计数器 − tail 的计数器），内层按盘（轮数 = 盘数）；跨轮只往槽的清单里追加；提前出口一个：某个计数器哪块盘上都解不出，整个交回 `None`。
fn slots_named_by_the_records_after_the_tail<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    parameters_of_the_pool: &MakeFilesystemParameters,
    tail: JournalSequenceNumber,
    version_journal_position: JournalSequenceNumber,
) -> Option<Vec<SlotNumber>> {
    if tail.instance_generation != version_journal_position.instance_generation {
        return None;
    }
    let record_bytes =
        usize::try_from(singlefs_format::JOURNAL_RECORD_BYTES).expect("一条记录 4096 字节");
    let filesystem_identifier_in_records =
        crate::unit::unit_filesystem_identifier(&parameters_of_the_pool.filesystem_identifier);
    let identities_of_the_devices = reader.device_identities();
    let mut slots_named = Vec::new();
    for counter in (tail.counter + 1)..=version_journal_position.counter {
        let offset = crate::journal::record_offset(
            counter,
            parameters_of_the_pool.geometry.journal_ring_bytes,
        );
        let mut is_the_counter_found = false;
        for holding_device in &identities_of_the_devices {
            let record_of_the_counter = reader
                .read(*holding_device, offset, record_bytes)
                .and_then(|bytes| {
                    crate::journal::JournalRecord::parse(&bytes, filesystem_identifier_in_records)
                })
                .filter(|record| record.counter == counter);
            if let Some(record) = record_of_the_counter {
                is_the_counter_found = true;
                slots_named.extend(
                    record
                        .named
                        .iter()
                        .flat_map(|named| named.locations)
                        .map(|location| location.slot),
                );
            }
        }
        if !is_the_counter_found {
            return None;
        }
    }
    Some(slots_named)
}

/// 这块盘自己的 journal 环里、那一版末条记录该在的那一槽（`journal::record_offset` 按计数器算）读一次，解得开、带末条标志、
/// (实例代号, 计数器) 是 `version_journal_position`、(实例代号, checkpoint_txg) 是 `version` 的，交回 true（乙'）。
/// 计数器是 0（那一版没有记录：环里一条记录都没有）的不读、交回 false。只读盘：至多读一槽，不重读。
fn last_record_of_the_version_in_its_ring_slot_on_the_device<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    parameters_of_the_pool: &MakeFilesystemParameters,
    device: DeviceIdentity,
    version: RollbackTarget,
    version_journal_position: JournalSequenceNumber,
) -> bool {
    if version_journal_position.counter < crate::journal::FIRST_JOURNAL_COUNTER {
        return false;
    }
    let offset = crate::journal::record_offset(
        version_journal_position.counter,
        parameters_of_the_pool.geometry.journal_ring_bytes,
    );
    let record_bytes =
        usize::try_from(singlefs_format::JOURNAL_RECORD_BYTES).expect("一条记录 4096 字节");
    reader
        .read(device, offset, record_bytes)
        .and_then(|bytes| {
            crate::journal::JournalRecord::parse(
                &bytes,
                crate::unit::unit_filesystem_identifier(
                    &parameters_of_the_pool.filesystem_identifier,
                ),
            )
        })
        .is_some_and(|record| {
            record.place_in_publish
                == crate::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish
                && record.instance == version_journal_position.instance_generation
                && record.counter == version_journal_position.counter
                && record.instance == version.instance
                && record.checkpoint_txg == version.checkpoint_txg
        })
}

/// 落后支的那一判（可写挂载取号之前逐盘核 [`devices_without_the_selected_version`] 与挂着之后收盘表的入口 [`devices_behind_the_current_version`]
/// 共用这一份；D18（块里携带什么信息） 已定项 11 落后支，C585（会话发布放进发布途中取下的旧快照） 用户定改法 C）：缺那一版的单元
/// （[`units_of_the_version_missing_on_a_device_behind_it`]：那一半的落后按乙' 判），**或**这块盘最新那份自证系统配置
/// （`newest_self_verified`）的 (实例代号, journal tail) 落后于那一版那次发布末条记录的 jsn、单元都在而那一版的根写在这块盘上、那个根环槽里不是它
/// （[`root_of_the_version_not_in_its_ring_slot_on_the_device`]；根那一半仍按 tail 判，乙' 不动它），才交回 (那份的 tail, 缺的是什么)；
/// 别的交回 `None`（只落后、单元都在、根槽里是它：根落盘之后、轮换之前崩；只剩一槽自证、那一槽停在上一次轮换）。只读盘。
fn what_a_device_behind_the_version_lacks<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    parameters_of_the_pool: &MakeFilesystemParameters,
    device: DeviceIdentity,
    newest_self_verified: &crate::system_configuration::SystemConfiguration,
    version: VersionJudgedBehindOrNot<'_>,
) -> Option<(JournalSequenceNumber, VersionLackingOnADeviceBehindIt)> {
    if let Some((newest_system_configuration_journal_tail, units_missing)) =
        units_of_the_version_missing_on_a_device_behind_it(
            reader,
            parameters_of_the_pool,
            device,
            newest_self_verified,
            version,
        )
    {
        return Some((
            newest_system_configuration_journal_tail,
            VersionLackingOnADeviceBehindIt::UnitsMissing(units_missing),
        ));
    }
    let newest_system_configuration_journal_tail = JournalSequenceNumber {
        instance_generation: newest_self_verified.quantities.journal_instance,
        counter: newest_self_verified.quantities.journal_tail,
    };
    let is_behind_the_version = newest_system_configuration_journal_tail < version.journal_position;
    if !is_behind_the_version {
        return None;
    }
    let root_of_the_version = version.root_in_the_ring;
    let root_not_in_its_ring_slot = root_of_the_version_not_in_its_ring_slot_on_the_device(
        reader,
        parameters_of_the_pool,
        device,
        root_of_the_version,
    );
    root_not_in_its_ring_slot.map(|root_not_in_its_ring_slot| {
        (
            newest_system_configuration_journal_tail,
            VersionLackingOnADeviceBehindIt::RootNotInItsRingSlot(root_not_in_its_ring_slot),
        )
    })
}

/// 落后支缺根那一半：那一版的根写进了根环（`root_of_the_version`），它该在的根环槽（`root_ring::target_for_publish` 按 txg 算）
/// 住在 `device` 这块盘上，读那一槽一次，不是它（读坏、或自证得过而是另一条根）就重读一次，仍不是它才交回两次读到的
/// （D16（发布语义） 已定项 1「根槽这一次读坏」：知道住着根的槽重读一次仍坏才算；挂着之后现行那一版的根槽是这个进程写过、FUA 返回过的）。
/// 根没写进根环、或那一槽不在这块盘上的交回 `None`，不读盘。只读盘：至多读那一槽两次。
fn root_of_the_version_not_in_its_ring_slot_on_the_device<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    parameters_of_the_pool: &MakeFilesystemParameters,
    device: DeviceIdentity,
    root_of_the_version: RootOfTheVersionInTheRing,
) -> Option<VersionRootNotInItsRingSlot> {
    let (instance, checkpoint_txg) = match root_of_the_version {
        RootOfTheVersionInTheRing::Written {
            instance,
            checkpoint_txg,
        } => (instance, checkpoint_txg),
        RootOfTheVersionInTheRing::NotWritten => return None,
    };
    let ring_slot = target_for_publish(
        checkpoint_txg,
        parameters_of_the_pool.geometry.root_ring_slots_per_region,
    );
    let device_of_the_ring_slot = parameters_of_the_pool.region_devices
        [usize::try_from(ring_slot.region).expect("区域号是 txg 对区域数取模，小于 3")];
    if device_of_the_ring_slot != device {
        return None;
    }
    let reading_other_than_the_version_root = || match read_root_ring_slot(
        reader,
        &parameters_of_the_pool.region_devices,
        &parameters_of_the_pool.geometry,
        &parameters_of_the_pool.filesystem_identifier,
        ring_slot,
    ) {
        RootRingSlotReading::SelfVerified(root)
            if root.instance == instance && root.checkpoint_txg == checkpoint_txg =>
        {
            None
        }
        RootRingSlotReading::SelfVerified(another_root) => {
            Some(RingSlotReadingOtherThanTheVersionRoot::AnotherRoot {
                instance: another_root.instance,
                checkpoint_txg: another_root.checkpoint_txg,
            })
        }
        RootRingSlotReading::Bad(bad_reading) => {
            Some(RingSlotReadingOtherThanTheVersionRoot::Bad(bad_reading))
        }
    };
    let first_reading = reading_other_than_the_version_root()?;
    let reread = reading_other_than_the_version_root()?;
    Some(VersionRootNotInItsRingSlot {
        ring_slot,
        first_reading,
        reread,
    })
}

/// 挂着之后收盘表的入口在「可见」之后逐盘核的落后支（D18（块里携带什么信息） 已定项 11「挂着之后收盘表的每个入口……核三样」的第三样，
/// 用户 2026-09-27 定；代码三方 m2-closeout-code-r2「同池旧快照的盘经入口洗白」）：交进来的每块盘，两槽里本池自证过、世代号最大的那一份
/// 系统配置落后于调用方持着的现行那一版（`current` 那次发布的末条记录），又缺它的单元、或它的根写在这块盘上而那个根环槽里不是它，
/// 就记下（[`DeviceBehindTheCurrentVersion`]）；判法与可写挂载取号之前逐盘核的落后支同一份（[`what_a_device_behind_the_version_lacks`]），
/// 「所选那一版」换成现行那一版，它的根恒写进了根环（每次发布在根槽 FUA 返回之后才换 `current`）。
/// 活的会话里每次发布在系统配置轮换落盘之后才返回（`transaction::CommitStep::RotateSystemConfigurationSlots`），现行那一版的盘都跟得上，
/// 只有被换成旧快照的盘落后；只剩一槽自证、那一槽停在上一次轮换的盘落后而单元都在、根槽里是它，不记（不误拒）。
/// 旧快照取在一次发布途中（单元与记录已写、根槽 FUA 还没写）、那次发布的根又写在这块盘上的，单元都在，靠根那一半拒（C585（会话发布放进发布途中取下的旧快照））。
/// 一份自证过的都没有的盘不在这里判（「可见」那一判先拒了它，[`devices_without_a_self_verified_system_configuration`]）。
/// 按交进来的次序交回。只读盘：每块盘读两个系统配置槽，落后的那几块再读现行那一版每个单元的落点，单元都在的再读至多两次它的根槽。
///
/// 循环：按交进来的盘逐块（轮数 = 盘数），跨轮只往交回的清单里追加；外层的提前出口一个：一份自证过的都没有（「可见」那一判的事，看下一块）。
pub(crate) fn devices_behind_the_current_version<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    parameters_of_the_pool: &MakeFilesystemParameters,
    current: &TransactionOutput,
) -> Vec<DeviceBehindTheCurrentVersion> {
    let current_version_journal_position = JournalSequenceNumber {
        instance_generation: current.record.instance,
        counter: current.record.counter,
    };
    let mut devices_behind = Vec::new();
    for device in reader.device_identities() {
        let Some(newest_self_verified) = self_verified_system_configuration_slots_of_the_pool(
            reader,
            device,
            parameters_of_the_pool,
        )
        .into_iter()
        .max_by_key(|slot| slot.quantities.slot_generation) else {
            continue;
        };
        if let Some((newest_system_configuration_journal_tail, lacking)) =
            what_a_device_behind_the_version_lacks(
                reader,
                parameters_of_the_pool,
                device,
                &newest_self_verified,
                VersionJudgedBehindOrNot {
                    version: RollbackTarget {
                        instance: current.root.instance,
                        checkpoint_txg: current.root.checkpoint_txg,
                    },
                    journal_position: current_version_journal_position,
                    units: &current.units,
                    root_in_the_ring: RootOfTheVersionInTheRing::Written {
                        instance: current.root.instance,
                        checkpoint_txg: current.root.checkpoint_txg,
                    },
                },
            )
        {
            devices_behind.push(DeviceBehindTheCurrentVersion {
                device,
                newest_system_configuration_journal_tail,
                current_version_journal_position,
                lacking,
            });
        }
    }
    devices_behind
}
