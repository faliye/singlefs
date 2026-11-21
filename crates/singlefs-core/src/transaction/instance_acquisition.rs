//! 取号：抬高的回收下限写进每块盘的系统配置、实例代号的取法、见证过的取号写与它的失败。

use super::*;

/// 取号全或无失败之后回卷写的结局（D18（块里携带什么信息） 已定项 11：先把已经写出的那几份回卷成旧代号，回卷不成才只读挂载）。
#[derive(Debug)]
pub enum AcquisitionRollback {
    /// 第一块盘的取号写就报错了：没有写出过带新号的系统配置，没有要回卷的。
    NothingWritten,
    /// 已写出的那几份都发了回卷写（写回旧代号）、设备都报了成功；回卷写之后没有屏障，返回时它们未必已持久。
    /// 这次挂载只读，盘上若还留着新号（回卷写没落盘也是这样），下一次取号跳过它。
    RolledBack,
    /// 回卷写或回卷写之前那道屏障报错：只读挂载。
    RollbackFailed(BlockDeviceError),
}

/// 取号没做成。拿不到实例代号，调用方就发不出任何带新号的写——那道屏障报错之后不许只重发屏障就继续
/// （D23（journal 的角色与格式） 已定项 16）。
#[derive(Debug)]
pub struct AcquisitionFailed {
    pub cause: BlockDeviceError,
    pub rollback: AcquisitionRollback,
}

/// 一次系统配置写带哪一个回退下界 F（D16（发布语义） 已定项 1「生效」与「抬 F 那一串」）。
/// 两个成员都在写之前按盘上现状现算整池的生效值（`recovery::effective_rollback_floor_rereading_unreadable_reads_once`）：一块盘两槽都读不出之后的那一次写，
/// 带的仍是整池的，不是这块盘自己读得出的（那样会把它写回 0，让它两槽里的最大值低于它上面的根，I-7.12（系统配置 F 不低于同盘根上的 F） 红）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RollbackFloorOfASystemConfigurationWrite {
    /// 取号、取号失败的回卷、每次发布末尾的轮换：带整池的 F 生效值（主 agent 2026-09-26 定，推的，被攻过零轮）。
    PoolEffectiveFloor,
    /// 抬 F 那一串第一次发布之前那一步：带新 F；新 F 低于整池生效值时带生效值（F 的生效值只增不减）。
    /// 调用方在这一步之前已经拒了低于生效值的新 F（`MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided`），
    /// 取大那一半只在两次读盘之间有瞬时读错时起作用。
    RaisedFloor(CheckpointTxg),
}

/// 抬 F 那一串先写系统配置那一步没做成（D16（发布语义） 已定项 1「抬 F 那一串」）。
#[derive(Debug)]
pub struct RaisedFloorSystemConfigurationWriteFailed {
    /// 报错之前已经写进新 F 的盘，按写的次序：它们系统配置里的新 F 留着、不回卷（F 的生效值只增不减）。
    /// 写之后那道屏障报错时是全部的盘（写都报了成功，只是没过屏障）；写之前那道屏障报错时是空的。
    pub devices_carrying_the_raised_floor: Vec<DeviceIdentity>,
    pub cause: BlockDeviceError,
    /// 这一步交给设备、设备报了成功的写（每块盘至多一次系统配置槽写）：它不属于任何一次发布，不进失败发布的账。
    pub writes: WritesByStructureKind,
}

/// 抬 F 那一串的第一步（D16（发布语义） 已定项 1「抬 F 那一串」，已定项 7「抬 F 那一串在它的第一次发布之前多一步」）：
/// 先过一道屏障（代码审阅第 19 条：这一写覆写较旧那一槽之前，另一槽里上一次轮换写的先持久），
/// 每块盘写一次系统配置槽、带新 F（[`RollbackFloorOfASystemConfigurationWrite::RaisedFloor`]），写完过一道屏障，之后才发第一条带新 F 的根。
/// tail 与实例代号取调用方给的——现行那一版末次发布轮换写的那一对，这一步只改 F 与世代号。这一步不属于任何一次发布。
/// 交回这一步的写账（每块盘一次系统配置槽写）。
///
/// # Errors
/// 写之前那道屏障、任一块盘的写或写之后那道屏障报错 ⇒ [`RaisedFloorSystemConfigurationWriteFailed`]：已写进的留着、不回卷，报出哪几块盘已经带上新 F
/// 与这一步已记的写；报错之后一个写都不再发。
pub fn write_the_raised_floor_into_every_system_configuration<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    raised_floor: CheckpointTxg,
    journal_tail: u64,
    journal_instance: InstanceGeneration,
) -> Result<WritesByStructureKind, RaisedFloorSystemConfigurationWriteFailed> {
    let writes_before_this_step = pool.writes_by_structure_kind.clone();
    let mut devices_carrying_the_raised_floor = Vec::new();
    // 发布末尾已有一道屏障（C577）：紧跟在同一个写入口的发布之后时下面这一道前面没有写，写入口不发它；留着它，不靠调用方前面是什么。
    // 第一个系统配置写之前一道池屏障（代码审阅第 19 条）：这一写覆写的是这块盘上较旧的那一槽，另一槽是上一次发布末尾轮换刚写的、
    // 还可能没持久；两槽轮换要的是「覆写旧槽时新槽已持久」，没有这一道，两次写同在一段里，崩溃时两槽可以一起撕坏。
    if let Err(cause) = pool.perform_commit_step(CommitStep::Barrier) {
        return Err(RaisedFloorSystemConfigurationWriteFailed {
            devices_carrying_the_raised_floor,
            cause,
            writes: pool
                .writes_by_structure_kind
                .since(&writes_before_this_step),
        });
    }
    for index in 0..pool.devices.len() {
        if let Err(cause) = pool.write_system_configuration_slot(
            index,
            journal_tail,
            journal_instance,
            RollbackFloorOfASystemConfigurationWrite::RaisedFloor(raised_floor),
        ) {
            return Err(RaisedFloorSystemConfigurationWriteFailed {
                devices_carrying_the_raised_floor,
                cause,
                writes: pool
                    .writes_by_structure_kind
                    .since(&writes_before_this_step),
            });
        }
        devices_carrying_the_raised_floor.push(pool.devices[index].0);
    }
    if let Err(cause) = pool.perform_commit_step(CommitStep::Barrier) {
        return Err(RaisedFloorSystemConfigurationWriteFailed {
            devices_carrying_the_raised_floor,
            cause,
            writes: pool
                .writes_by_structure_kind
                .since(&writes_before_this_step),
        });
    }
    Ok(pool
        .writes_by_structure_kind
        .since(&writes_before_this_step))
}

/// 每块盘两槽里全部自证过（fsid 与本池相同）的系统配置中最大的实例代号；一份都没有时是 mkfs 的 0。取号回卷时写回的旧号也是它。
fn highest_system_configuration_instance<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
) -> InstanceGeneration {
    let spacing = u64::from(pool.parameters.geometry.fixed_structure_slot_spacing);
    let filesystem_identifier = pool.parameters.filesystem_identifier;
    pool.devices
        .iter()
        .flat_map(|(identity, _)| {
            verified_system_configuration_slots(
                &*pool.devices,
                *identity,
                spacing,
                &filesystem_identifier,
            )
        })
        .map(|system_configuration| system_configuration.quantities.journal_instance)
        .max()
        .unwrap_or(MKFS_INSTANCE_GENERATION)
}

/// 判据 N-配置 的「没有这样的槽 ⇒ c_见证 = 0」（E158 第 3 次跑登记 `research/prompts/e158-r3-prereg.md`「`| **N-配置** |`」那一行），与 mkfs 写进每个槽的
/// tail 同值。取号时有一块盘两槽都读不出就拒（[`self_verified_system_configurations_of_every_device`]），所以取号读到的见证值只在
/// 盘表为空、一个槽都没有可读的时候取它。
const JOURNAL_TAIL_WITNESSED_BY_NO_SYSTEM_CONFIGURATION: u64 = 0;

/// 取号那一刻逐盘读到的两槽：盘表里每块盘两槽中本池自证过（整槽校验和过、fsid 与本池相同）的系统配置，按盘表次序、一块盘一项。
/// 只读盘、不写。两道拒，都在第一个取号写之前：
///
/// - 判不出（R2，`recovery::system_configuration_witness_is_undecidable`：有槽设备报读错，且没有一块盘两槽都自证过；C331（择根倒挂压过已确认的写）
///   改取 R2，用户 2026-10-03 定，零轮）：见证值可能缺了读错的那一槽，取不了值，与可写挂载判 N-配置 同一个判不出（D23（journal 的角色与格式） 已定项 14
///   「取号那一刻按同一个判不出（R2……）判」）。撕裂的系统配置轮换（读得出、自证不过）不拒：取号写重写坏的那一槽。
/// - 某块盘一份都读不出（C554 乙-配置续 Q1，主 agent 2026-09-27 定走「拒」）：这块盘不「可见」（D18（块里携带什么信息） 已定项 11
///   「可写挂载的顺序」：「可见」= 独占打开成功且系统配置读得通），与可写挂载取号之前的逐盘核（`mount.rs` 的
///   `devices_without_the_selected_version` 第一支，Z3-A 乙）同一判。R2 判得出的盘面上也有这一形（一块盘两槽都读不出、另一块两槽都自证过），
///   R2 之下这一道照留（条款没写它随取甲收严之后是否还单独成立，照取甲之前的「一份都没有就拒」留着）。
///
/// 例外（丙'，用户 2026-10-19 定 S4「开例外」、2026-10-20 再加两件闸，三方判决 `research/prompts/m3-close-sysconf-r3-main-verification.md` 第三节）：
/// `devices_rewritten_by_the_acquisition` 里的盘是可写挂载取号之前逐盘核过补写闸的两槽全废的盘（`mount.rs` 的 `devices_without_the_selected_version`），
/// 它们一份自证过的都没有不拒，交回空的一项，取号写两轮把它两槽补写成新号。别的调用方（mkfs 同一个进程那条流、用例）交空表，照旧拒。
///
/// # Errors
/// 判不出 ⇒ [`InstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness`]（带盘表次序第一块报读错的盘）；
/// 按盘表次序第一块一份自证过的都没有、又不在 `devices_rewritten_by_the_acquisition` 里的盘 ⇒
/// [`InstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness`]。
fn self_verified_system_configurations_of_every_device<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    devices_rewritten_by_the_acquisition: &[DeviceIdentity],
) -> Result<Vec<Vec<SystemConfiguration>>, InstanceAcquisitionFailed> {
    let spacing = u64::from(pool.parameters.geometry.fixed_structure_slot_spacing);
    let filesystem_identifier = pool.parameters.filesystem_identifier;
    let slots_of_every_device: Vec<SystemConfigurationSlotsOfOneDevice> = pool
        .devices
        .iter()
        .map(|(identity, _)| {
            system_configuration_slots_of_one_device_read_once(
                &*pool.devices,
                *identity,
                spacing,
                &filesystem_identifier,
            )
        })
        .collect();
    if system_configuration_witness_is_undecidable(&slots_of_every_device) {
        let first_device_reporting_a_read_error = slots_of_every_device
            .iter()
            .find(|slots_of_this_device| slots_of_this_device.slots_reporting_a_read_error > 0)
            .expect("判不出的前一半就是有一块盘报读错")
            .device;
        return Err(
            InstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
                first_device_reporting_a_read_error,
            },
        );
    }
    slots_of_every_device
        .into_iter()
        .map(|slots_of_this_device| {
            if slots_of_this_device.self_verified.is_empty() {
                if devices_rewritten_by_the_acquisition.contains(&slots_of_this_device.device) {
                    return Ok(slots_of_this_device.self_verified);
                }
                return Err(
                    InstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness {
                        device: slots_of_this_device.device,
                    },
                );
            }
            Ok(slots_of_this_device.self_verified)
        })
        .collect()
}

/// 取号那一刻的见证值 c_见证（C554 乙-配置续，用户 2026-09-27 定；定义取自 `research/prompts/e158-r3-prereg.md`
/// 「`| **N-配置续** |`」那一行与 `research/prompts/e158-r4-prereg.md`「N-配置 在 `replay_journal` 交回之后另读每块盘两个系统配置槽」那一条）：每块盘两槽里全部自证过的系统配置的 journal tail 的最大值，
/// 与可写挂载判 N-配置 同一取法（`mount.rs` 的 `newer_publish_witness`）。`slots_of_every_device` 是
/// [`self_verified_system_configurations_of_every_device`] 那一次读到的（每块盘至少一份）；盘表为空时
/// [`JOURNAL_TAIL_WITNESSED_BY_NO_SYSTEM_CONFIGURATION`]。
fn highest_journal_tail_of(slots_of_every_device: &[Vec<SystemConfiguration>]) -> u64 {
    slots_of_every_device
        .iter()
        .flatten()
        .map(|system_configuration| system_configuration.quantities.journal_tail)
        .max()
        .unwrap_or(JOURNAL_TAIL_WITNESSED_BY_NO_SYSTEM_CONFIGURATION)
}

/// 盘上最大的实例代号已是 `u32::MAX`，取号要的下一个号（它加一）装不下（代码审阅第 36 条）：实例代号是盘上读来的 4 字节
/// （系统配置槽与根记录里的），坏镜像、外来镜像才走得到。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstanceGenerationPastTheTopOfItsRange {
    pub highest_on_disk: InstanceGeneration,
}

/// 取号会取到的号 = max(每块盘两槽里全部自证过的槽的实例代号, 根环里全部根记录的实例代号) + 1（只读盘、不写）：可写挂载要在取号之前
/// 按它判这次要写的行区间（没有文件版本的一版上要写行就在任何写之前拒绝），`acquire_expected_instance` 写之前重算的也是它。
///
/// # Errors
/// 那个最大值已是 `u32::MAX` ⇒ [`InstanceGenerationPastTheTopOfItsRange`]（不 panic，代码审阅第 36 条）。
pub fn next_instance_generation_to_acquire<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
) -> Result<InstanceGeneration, InstanceGenerationPastTheTopOfItsRange> {
    let highest_root = highest_root_instance(
        &*pool.devices,
        &pool.parameters.region_devices,
        &pool.parameters.geometry,
        &pool.parameters.filesystem_identifier,
    )
    .unwrap_or(MKFS_INSTANCE_GENERATION);
    let highest_on_disk = highest_system_configuration_instance(pool).max(highest_root);
    highest_on_disk
        .0
        .checked_add(1)
        .map(InstanceGeneration)
        .ok_or(InstanceGenerationPastTheTopOfItsRange { highest_on_disk })
}

/// 同 [`next_instance_generation_to_acquire`]，号装不下就 panic：只给 `acquire_instance`（mkfs 同一个进程里那条流与用例）与
/// 用例直接比号用。可写挂载走 [`next_instance_generation_to_acquire`]，在任何写之前拒。
///
/// # Panics
/// 盘上最大的实例代号已是 `u32::MAX`。
#[must_use]
pub fn instance_generation_to_acquire<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
) -> InstanceGeneration {
    next_instance_generation_to_acquire(pool).expect(
        "盘上最大的实例代号小于 u32::MAX：mkfs 写 0、每次可写挂载加一；可写挂载走 next_instance_generation_to_acquire，装不下时在任何写之前拒",
    )
}

/// 取号（D23（journal 的角色与格式） 已定项 16、D18（块里携带什么信息） 已定项 11；2026-09-14 用户定案，
/// C322（取号那一步的屏障怎么放没有条款） 三轮三方）：新号 = max(每块盘两槽里全部自证过的槽的实例代号, 根环里全部根记录的
/// 实例代号) + 1；第一个取号写之前一道屏障（代码审阅第 19 条：取号写覆写较旧那一槽之前，另一槽里上一次轮换写的先持久）；
/// 取号写两轮（甲'，[`AcquisitionWriteRound`]）：每轮逐盘写一次系统配置槽（世代号逐盘 +1；tail 写取号那一刻的见证值，C554 乙-配置续），
/// 每轮之后一道屏障，第二轮那道屏障完成才把新号交出去，于是本实例的第一个非系统配置写一定排在它之后。首次挂载路径上第二轮那道就是暖机开场那道：
/// 暖机那道屏障前面没有写，写入口不再发第二次。写之前那道屏障报错 ⇒ 一个字节没写、取号失败；任一块盘的取号写或任一轮之后那道屏障报错
/// ⇒ 已写出的盘每块回卷一次成旧代号（取号之前全部自证过的槽中最大的实例代号；回卷写之前同样先过一道屏障），报失败。
/// ⚠️ D18（块里携带什么信息） 已定项 11 的「重试到 T_retry 用尽才算失败」这里没有做：
/// 块设备报的第一个错就判失败（写路径上的重试全仓都还没有）。
///
/// 前面没有可写挂载那一道逐盘核（mkfs 同一个进程里那条流、用例与装置在调）：取号那一刻读见证值时判不出（R2）或某块盘一份本池自证过的都没有，
/// 同样在第一个取号写之前拒（[`self_verified_system_configurations_of_every_device`] 的两道）。
///
/// # Errors
/// 判不出 ⇒ `SystemConfigurationWitnessUndecidableWhenReadingTheWitness`；某块盘一份自证过的系统配置都没有 ⇒
/// `DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness`（两样都一个字节没写）；
/// 屏障或取号写报错 ⇒ `Acquisition`（回卷的结局在里面）。
///
/// # Panics
/// 盘上最大的实例代号已是 `u32::MAX`（坏镜像、外来镜像：实例代号是盘上读来的 4 字节）：取号要的下一个号装不下
/// （[`instance_generation_to_acquire`]）。可写挂载走 `acquire_expected_instance`，同一格在任何写之前拒、不 panic。
pub fn acquire_instance<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
) -> Result<InstanceGeneration, InstanceAcquisitionFailed> {
    let instance = instance_generation_to_acquire(pool);
    write_acquired_instance(pool, instance)
}

/// 取号没做成（[`acquire_instance`]、取号的写那一半）。
#[derive(Debug)]
pub enum InstanceAcquisitionFailed {
    /// 取号那一刻读见证值时判不出（R2：有槽设备报读错，且没有一块盘两槽都自证过；C331（择根倒挂压过已确认的写） 改取 R2，
    /// 用户 2026-10-03 定）：一个字节都没写。`first_device_reporting_a_read_error` 是盘表次序第一块报读错的盘。
    /// 第一道屏障之前就判不出时连屏障都没发；屏障之后重读才判不出时发了那一道屏障、没有写。
    SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
        first_device_reporting_a_read_error: DeviceIdentity,
    },
    /// 取号那一刻读见证值时，`device` 两槽里一份本池自证过的系统配置都没有（C554 乙-配置续 Q1；D18（块里携带什么信息） 已定项 11「可见」）：
    /// 一个字节都没写。屏障前后哪一次读到的，同上一个成员。
    DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness {
        device: DeviceIdentity,
    },
    /// 屏障或取号写报错（回卷的结局在里面）。
    Acquisition(AcquisitionFailed),
}

impl From<AcquisitionFailed> for InstanceAcquisitionFailed {
    fn from(failure: AcquisitionFailed) -> Self {
        InstanceAcquisitionFailed::Acquisition(failure)
    }
}

/// 按调用方判定时算出的号取号没做成。
#[derive(Debug)]
pub enum ExpectedInstanceAcquisitionFailed {
    /// 写之前重算出的号与调用方判定时算出的号不同（两次读系统配置之间一次瞬时读错就够：读错的槽当作没有）：一个字节都没写。
    InstanceGenerationChangedBeforeWrite {
        expected: InstanceGeneration,
        recomputed: InstanceGeneration,
    },
    /// 写之前重算时盘上最大的实例代号已是 `u32::MAX`（两次读之间瞬时读错才会与判定时不同）：一个字节都没写。
    InstanceGenerationPastTheTopOfItsRange(InstanceGenerationPastTheTopOfItsRange),
    /// 同 [`InstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness`]：可写挂载读阶段的见证读判得出、
    /// 取号那一刻读见证值时判不出（两次读之间一次瞬时读错就够）。一个字节都没写。
    SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
        first_device_reporting_a_read_error: DeviceIdentity,
    },
    /// 同 [`InstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness`]：可写挂载取号之前的逐盘核读得出、
    /// 取号那一刻读见证值时这块盘一份自证过的系统配置都没有。一个字节都没写。
    DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness {
        device: DeviceIdentity,
    },
    Acquisition(AcquisitionFailed),
}

/// 取号，号用调用方判定时算出的那一个（`instance_generation_to_acquire`）：写之前再算一遍，对不上就不写、报错返回——可写挂载在取号之前
/// 按这个号判要写的行区间，号在判定与取号之间变了，判定就作废（m2-emptypool-nonempty-r1 云端攻方腿 Z2：两块盘第 2 次读系统配置槽 0
/// 各报一次瞬时读错，判定看到号 1 放行、取号读到号 2 写进两盘）。
///
/// # Errors
/// 重算的号不同 ⇒ `InstanceGenerationChangedBeforeWrite`；读见证值时判不出 ⇒ `SystemConfigurationWitnessUndecidableWhenReadingTheWitness`，
/// 某块盘一份自证过的都没有 ⇒ `DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness`；取号写或屏障报错 ⇒ `Acquisition`（与 `acquire_instance` 同）。
pub fn acquire_expected_instance<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    expected: InstanceGeneration,
) -> Result<InstanceGeneration, ExpectedInstanceAcquisitionFailed> {
    let witnessed = read_the_witness_before_the_expected_acquisition_writes(pool, expected, &[])?;
    write_the_witnessed_expected_acquisition(pool, witnessed)
}

/// [`acquire_expected_instance`] 的头一半：写之前重算要取的号、与 `expected` 比，再照 [`read_the_witness_before_the_acquisition_writes`]
/// 读上一个号、核见证值、发一道屏障、再读见证值。只读盘与发那一道屏障，一个字节都不写：可写挂载在这一半里让读穿到盘上
/// （`mount::acquire_then_publish_until_the_continuation`，C554 乙-配置续 Q1 的两次读要读盘）。
/// `devices_rewritten_by_the_acquisition` 是可写挂载核过补写闸的两槽全废的盘（丙'，[`self_verified_system_configurations_of_every_device`]）。
///
/// # Errors
/// 同 [`acquire_expected_instance`] 写之前的那几种（号变了、号到顶、判不出、某块盘一份自证过的都没有、那道屏障报错）。
pub(crate) fn read_the_witness_before_the_expected_acquisition_writes<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    expected: InstanceGeneration,
    devices_rewritten_by_the_acquisition: &[DeviceIdentity],
) -> Result<AcquisitionWitnessedBeforeItsWrites, ExpectedInstanceAcquisitionFailed> {
    let recomputed = next_instance_generation_to_acquire(pool)
        .map_err(ExpectedInstanceAcquisitionFailed::InstanceGenerationPastTheTopOfItsRange)?;
    if recomputed != expected {
        return Err(
            ExpectedInstanceAcquisitionFailed::InstanceGenerationChangedBeforeWrite {
                expected,
                recomputed,
            },
        );
    }
    read_the_witness_before_the_acquisition_writes(
        pool,
        recomputed,
        devices_rewritten_by_the_acquisition,
    )
    .map_err(expected_acquisition_failure_of)
}

/// [`acquire_expected_instance`] 的后一半：照头一半读到的写（[`write_the_witnessed_acquisition`]）。
///
/// # Errors
/// 取号写或写之后那道屏障报错 ⇒ `Acquisition`（回卷的结局在里面）。
pub(crate) fn write_the_witnessed_expected_acquisition<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    witnessed: AcquisitionWitnessedBeforeItsWrites,
) -> Result<InstanceGeneration, ExpectedInstanceAcquisitionFailed> {
    write_the_witnessed_acquisition(pool, witnessed).map_err(expected_acquisition_failure_of)
}

fn expected_acquisition_failure_of(
    failure: InstanceAcquisitionFailed,
) -> ExpectedInstanceAcquisitionFailed {
    match failure {
        InstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
            first_device_reporting_a_read_error,
        } => ExpectedInstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
            first_device_reporting_a_read_error,
        },
        InstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness {
            device,
        } => ExpectedInstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness {
            device,
        },
        InstanceAcquisitionFailed::Acquisition(acquisition) => {
            ExpectedInstanceAcquisitionFailed::Acquisition(acquisition)
        }
    }
}

/// 取号写之前读到的（[`read_the_witness_before_the_acquisition_writes`] 造，[`write_the_witnessed_acquisition`] 照它写）：
/// 要取的号、写之前盘上最大的实例代号（回卷写用）、屏障之后读到的见证值。字段私有：写那一半只收读那一半交出来的这一份。
#[derive(Debug)]
pub(crate) struct AcquisitionWitnessedBeforeItsWrites {
    pub(super) instance: InstanceGeneration,
    pub(super) previous_instance: InstanceGeneration,
    pub(super) witnessed_journal_tail: u64,
}

/// 取号的写那一半：照 [`read_the_witness_before_the_acquisition_writes`] 读、再照 [`write_the_witnessed_acquisition`] 写。
fn write_acquired_instance<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    instance: InstanceGeneration,
) -> Result<InstanceGeneration, InstanceAcquisitionFailed> {
    let witnessed = read_the_witness_before_the_acquisition_writes(pool, instance, &[])?;
    write_the_witnessed_acquisition(pool, witnessed)
}

/// 取号写之前那几次读：读上一个号，逐盘核一遍每块盘两槽读得出自证过的系统配置，一道屏障，再读一次每块盘两槽、取见证值
/// （[`self_verified_system_configurations_of_every_device`]、[`highest_journal_tail_of`]）。两次读里有一次判不出（R2）或有一块盘
/// 一份自证过的都没有（`devices_rewritten_by_the_acquisition` 里的盘除外，丙'）就报它；那道屏障报错就报失败。都是一个字节没写。
fn read_the_witness_before_the_acquisition_writes<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    instance: InstanceGeneration,
    devices_rewritten_by_the_acquisition: &[DeviceIdentity],
) -> Result<AcquisitionWitnessedBeforeItsWrites, InstanceAcquisitionFailed> {
    let previous_instance = highest_system_configuration_instance(pool);
    // 读见证值时判不出（R2）或有一块盘一份自证过的都没有就拒（C554 乙-配置续 Q1 第 1 条）：这一核放在第一道屏障之前，拒的时候连屏障都没发，录制流一步不多。
    self_verified_system_configurations_of_every_device(
        pool,
        devices_rewritten_by_the_acquisition,
    )?;
    // 第一个取号写之前一道池屏障（代码审阅第 19 条）：取号写覆写的是这块盘上较旧的那一槽，另一槽可能是上一次发布末尾轮换刚写的
    // （同一个进程里，或上一个进程退出之前）、还没持久；两槽轮换要的是「覆写旧槽时新槽已持久」。这道屏障在任何取号写之前报错，
    // 一个字节都没写，取号失败、没有要回卷的。首次挂载紧接在 mkfs 末尾那道屏障之后，这一道前面没有写：录制流里并掉，设备多收一次 FLUSH。
    if let Err(cause) = pool.perform_commit_step(CommitStep::Barrier) {
        return Err(AcquisitionFailed {
            cause,
            rollback: AcquisitionRollback::NothingWritten,
        }
        .into());
    }
    // 取号那一写带的 tail 是这一刻的见证值（C554 乙-配置续，用户 2026-09-27 定），不是 0：连着几次取号之后崩溃、中间没有
    // 发布轮换，两槽都换成了取号写，写 0 就把上一次轮换见证的那次发布抹掉，下一次可写挂载的 N-配置 判不出、会抛弃暂时读不出的最新根。
    // 第一道屏障之后、第一个取号写之前另读每块盘两槽一次（E158 第 4 次跑登记 `research/prompts/e158-r4-prereg.md`「N-配置 在 `replay_journal` 交回之后另读每块盘两个系统配置槽」那一条）；逐盘取号写与回卷写都用这一次读到的。
    // 屏障前那一核与这一次读之间又判不出或有一块盘一份都读不出，就不写（判定与写读同一份输入：写之前重算、对不上不写）。
    // mkfs 写的 tail 是 0，mkfs 之后第一次取号读到的就是 0：新池新建文件的字节不变。
    let witnessed_journal_tail =
        highest_journal_tail_of(&self_verified_system_configurations_of_every_device(
            pool,
            devices_rewritten_by_the_acquisition,
        )?);
    Ok(AcquisitionWitnessedBeforeItsWrites {
        instance,
        previous_instance,
        witnessed_journal_tail,
    })
}

/// 取号写的一轮（甲'，三方判决 `research/prompts/m3-close-sysconf-r3-main-verification.md` 第四节「落地」，用户 2026-10-19 定 S5「罩，取号写两轮」）：
/// 每轮逐盘写一次系统配置槽、之后一道池屏障。两轮按世代号逐盘 +1 写，第二轮落在第一轮没碰的那一槽，两轮都完成之后每块盘两槽都带新号——
/// 之后哪一槽持续坏，另一槽仍记着这个号，下一次取号跳过它（C555（两盘最新系统配置槽都坏时取号撞号））。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AcquisitionWriteRound {
    /// 覆写每块盘取号之前较旧的那一槽。
    OverTheOlderSlotOfEachDevice,
    /// 覆写每块盘取号之前最新的那一槽（第一轮之后它是较旧的那一槽）。
    OverTheSlotNewestBeforeTheAcquisition,
}

/// 取号写的两轮，按发的次序（[`AcquisitionWriteRound`]）。
const ACQUISITION_WRITE_ROUNDS: [AcquisitionWriteRound; 2] = [
    AcquisitionWriteRound::OverTheOlderSlotOfEachDevice,
    AcquisitionWriteRound::OverTheSlotNewestBeforeTheAcquisition,
];

/// 取号写（甲'，[`AcquisitionWriteRound`]）：两轮，每轮逐盘写一次系统配置槽（实例代号写新号、tail 写见证值），每轮之后一道池屏障，
/// 两轮都完成才交出新号。任一写或屏障报错就把已写出的盘回卷成旧代号（tail 仍写见证值）：传给回卷的盘表里每块盘只出现一次，
/// 回卷每块盘只写一次（第二轮判决 `research/prompts/m3-close-sysconf-r2-main-verification.md` 第二节 T1：逐份回卷会对同一块盘连写两槽、
/// 中间没有屏障，同段断电两槽一起坏，落进 C463 那一形）。
///
/// 循环：外层按两轮（轮数 2），内层按盘（轮数 = 盘数）；跨轮带着已写出的盘表，只追加没出现过的盘。提前出口两个：写报错、屏障报错，都回卷后返回。
fn write_the_witnessed_acquisition<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    witnessed: AcquisitionWitnessedBeforeItsWrites,
) -> Result<InstanceGeneration, InstanceAcquisitionFailed> {
    let AcquisitionWitnessedBeforeItsWrites {
        instance,
        previous_instance,
        witnessed_journal_tail,
    } = witnessed;
    let mut devices_written: Vec<usize> = Vec::new();
    for _round in ACQUISITION_WRITE_ROUNDS {
        for index in 0..pool.devices.len() {
            if let Err(cause) = pool.write_system_configuration_slot(
                index,
                witnessed_journal_tail,
                instance,
                RollbackFloorOfASystemConfigurationWrite::PoolEffectiveFloor,
            ) {
                return Err(pool.roll_back_acquisition(
                    &devices_written,
                    previous_instance,
                    witnessed_journal_tail,
                    cause,
                ));
            }
            if !devices_written.contains(&index) {
                devices_written.push(index);
            }
        }
        if let Err(cause) = pool.perform_commit_step(CommitStep::Barrier) {
            return Err(pool.roll_back_acquisition(
                &devices_written,
                previous_instance,
                witnessed_journal_tail,
                cause,
            ));
        }
    }
    Ok(instance)
}
