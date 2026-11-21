//! 可写挂载的入口：[`mount_writable`] 与它带只供测试的开关、后台计算设置、整理参数的各个变体，
//! 择系统配置与全部开关一处收的那一条。

use super::*;

/// 可写挂载：恢复 → 重建上一版与分配器 → 取号 → 写行发布 → 暖机到本实例的根覆盖每块盘。只做过 mkfs 的池（所选根的树表 0 条、
/// 环里没有记录）同样走这条路：取号 1、不写行、零单元的发布推到本实例的根覆盖每块盘，第一个文件版本接在 `current` 后面。
///
/// # Errors
/// 交进来的盘数低于 w 的下限（`WritableDeviceCountBelowTheStripeWidthLowerBound`，在读任何一块盘之前）、
/// 恢复失败、所选根下有文件而环里一条记录都没有、实例表沿链读不出或解不开、上一版树表 0 条而取号之前的空间准入不过
/// （`SpaceAdmissionRefusedBeforeAcquisition`；上一版带文件的不在这里报，写行之后推抬 F 再判，结局在 `MountOutput::space_admission`）、
/// 取号之前的预演报错、取号之后那一串在分配器的拷贝上取不到落点
/// （`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`）、有盘不带所选那一版（空盘，或停在旧状态：
/// `WritableMountRefusedByDevicesWithoutTheSelectedVersion`，那样的池只许只读挂载）、取号失败、发布失败。
/// 盘表里有设备身份交了不止一次（`DeviceIdentitiesHandedInMoreThanOnce`，在读任何一块盘之前）、调用方的参数或盘表（盘数与 `devs`、
/// 每块盘的本盘设备号）与盘上择到的系统配置不一致（`CallerParametersDisagreeWithTheSelectedSystemConfiguration`，在任何写之前）。
/// 取号之前准入不够、写行与暖机之后推抬 F 时，抬 F 自己报的、不是空间不够的错装进 `FloorRaiseFailedAfterTheMountsPublishes` 交回
/// （块设备错这一类不是「推满仍不够」；那时实例已取、行已写，写行、暖机与之前推成的那几串已落盘，它们的写账随错交出）；
/// 盘上读来的实例代号或 txg 已到顶、下一个号装不下（`SequenceNumberPastTheTopOfItsRange`，取号之前）；
/// 抬 F 那一串自己也取不到落点、被空间准入拒的，是推满仍不够，挂载照样做成（`MountSpaceAdmission::StillShortAfterTheFloorRaises`）。
/// 挂载续做里推满仍放不下同样不报错：挂载照样做成、队列留着（`MountOutput::deletion_queue_continuation_outcome`）；续做里的一步没发成、
/// 不是空间不够才报 `Publish`，续做里为空间推的抬 F 自己报了不是空间不够的错报 `FloorRaiseFailedAfterTheMountsPublishes`
/// （[`continue_the_deletion_queue_until_it_is_empty`]）。
pub fn mount_writable<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
) -> Result<Mounted, MountError> {
    mount_writable_with_space_admission(parameters, devices, SpaceAdmission::JudgedByTheFormula)
}

/// 同 [`mount_writable`]，判不判空间准入由调用方给（只供测试的开关 `SpaceAdmission`；产品路径走 `mount_writable`，恒判）。
/// 开关装在交回的分配器上，这次挂载之后的发布照它判不判。
///
/// # Errors
/// 同 [`mount_writable`]；`SkippedByTheTestOnlySwitch` 时不报 `SpaceAdmissionRefusedBeforeAcquisition`。
pub fn mount_writable_with_space_admission<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    space_admission: SpaceAdmission,
) -> Result<Mounted, MountError> {
    mount_writable_with_test_only_switches(parameters, devices, space_admission, ShadowLedger::On)
}

/// 同 [`mount_writable_with_space_admission`]，影子账那一臂也由调用方给（只供测试的开关 [`ShadowLedger`]，`.claude/rules/fs-design.md`
/// 五条硬要求第 2 条：C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 那一格的必红靠它强制进入——被抛弃的根由崩溃恢复造出）。
/// 产品路径走 [`mount_writable`]，恒 `On`。
///
/// # Errors
/// 同 [`mount_writable_with_space_admission`]。
pub fn mount_writable_with_test_only_switches<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    space_admission: SpaceAdmission,
    shadow_ledger: ShadowLedger,
) -> Result<Mounted, MountError> {
    mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        parameters,
        devices,
        space_admission,
        shadow_ledger,
        BeforeTheOneReread::RereadImmediately,
    )
}

/// 同 [`mount_writable_with_test_only_switches`]，「重读一次」之前做什么也由调用方给（[`BeforeTheOneReread`]：只供测试的钩子，
/// 用例在里面撤掉暂时的读故障；产品路径走 [`mount_writable`]，立即重读）。
///
/// # Errors
/// 同 [`mount_writable_with_test_only_switches`]；读到的样子里有更新的东西读不出、重读一次仍读不出 ⇒
/// `NewerStateStillUnreadableAfterOneReread`（取号之前，盘上逐字节不变）。
pub fn mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread<
    Device: BlockDevice,
>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    space_admission: SpaceAdmission,
    shadow_ledger: ShadowLedger,
    before_the_one_reread: BeforeTheOneReread<'_>,
) -> Result<Mounted, MountError> {
    mount_writable_with_every_test_only_switch(
        parameters,
        devices,
        space_admission,
        shadow_ledger,
        DeletionSwitches::PRODUCT,
        CompactionSwitches::PRODUCT,
        before_the_one_reread,
        InstanceTableSweepConfiguration::PRODUCT,
        AdmittedBackgroundComputeSettings::PRODUCT,
    )
}

/// 同 [`mount_writable_with_test_only_switches`]，删除那几次发布的两个只供测试的开关（每批几个数据单元、一片墓碑容器装满几条，
/// `unlink::DeletionSwitches`，`.claude/rules/fs-design.md` 五条硬要求第 2 条）也由调用方给：装在交回的分配器上，挂载续做与这次挂载之后的
/// 会话照它（与空间准入开关同一个做法）。产品路径走 [`mount_writable`]，恒 `DeletionSwitches::PRODUCT`。
///
/// # Errors
/// 同 [`mount_writable_with_test_only_switches`]。
pub fn mount_writable_with_test_only_switches_and_deletion_switches<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    space_admission: SpaceAdmission,
    shadow_ledger: ShadowLedger,
    deletion_switches: DeletionSwitches,
) -> Result<Mounted, MountError> {
    mount_writable_with_every_test_only_switch(
        parameters,
        devices,
        space_admission,
        shadow_ledger,
        deletion_switches,
        CompactionSwitches::PRODUCT,
        BeforeTheOneReread::RereadImmediately,
        InstanceTableSweepConfiguration::PRODUCT,
        AdmittedBackgroundComputeSettings::PRODUCT,
    )
}

/// 同 [`mount_writable_with_test_only_switches`]，整理的只供测试的开关（`compaction_plan::CompactionSwitches`，判决 G6）也由调用方给：
/// 装在交回的分配器上，挂载续做盘上那一条未完成的整理意图（[`MountOutput::compaction_continuation`]）与这次挂载之后的会话照它。
/// 产品路径走 [`mount_writable`]，恒 `CompactionSwitches::PRODUCT`（批大小取默认）。
///
/// # Errors
/// 同 [`mount_writable_with_test_only_switches`]：续做自己不报错（提示类意图不挡挂载，D8（核心索引结构） 已定项 5），走到哪儿记进交回值。
pub fn mount_writable_with_compaction_switches<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    compaction_switches: CompactionSwitches,
) -> Result<Mounted, MountError> {
    mount_writable_with_every_test_only_switch(
        parameters,
        devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        DeletionSwitches::PRODUCT,
        compaction_switches,
        BeforeTheOneReread::RereadImmediately,
        InstanceTableSweepConfiguration::PRODUCT,
        AdmittedBackgroundComputeSettings::PRODUCT,
    )
}

/// 同 [`mount_writable_with_space_admission`]，整理的只供测试的开关也由调用方给（随机历史的执行器要它：挂载里不续做整理、下一趟整理接着推盘上那一条，
/// 逐版比那一串里没有续做整理的发布）。
///
/// # Errors
/// 同 [`mount_writable_with_space_admission`]。
pub fn mount_writable_with_space_admission_and_compaction_switches<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    space_admission: SpaceAdmission,
    compaction_switches: CompactionSwitches,
) -> Result<Mounted, MountError> {
    mount_writable_with_every_test_only_switch(
        parameters,
        devices,
        space_admission,
        ShadowLedger::On,
        DeletionSwitches::PRODUCT,
        compaction_switches,
        BeforeTheOneReread::RereadImmediately,
        InstanceTableSweepConfiguration::PRODUCT,
        AdmittedBackgroundComputeSettings::PRODUCT,
    )
}

/// 同 [`mount_writable_with_test_only_switches`]，交回之前那一判实例表清扫的开关与阈值也由调用方给
/// （D18（块里携带什么信息） 已定项 11「回收」一条：开关与阈值是配置项，阈值默认 365 行，用户 2026-10-29 定，C690；
/// [`crate::instance_table_sweep::InstanceTableSweepConfiguration`]）。产品路径走 [`mount_writable`]，取
/// `InstanceTableSweepConfiguration::PRODUCT`。
///
/// # Errors
/// 同 [`mount_writable_with_test_only_switches`]：清扫自己不报错，它走到哪儿记在 `MountOutput::instance_table_sweep`。
pub fn mount_writable_with_instance_table_sweep_configuration<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    space_admission: SpaceAdmission,
    shadow_ledger: ShadowLedger,
    instance_table_sweep: InstanceTableSweepConfiguration,
) -> Result<Mounted, MountError> {
    mount_writable_with_every_test_only_switch(
        parameters,
        devices,
        space_admission,
        shadow_ledger,
        DeletionSwitches::PRODUCT,
        CompactionSwitches::PRODUCT,
        BeforeTheOneReread::RereadImmediately,
        instance_table_sweep,
        AdmittedBackgroundComputeSettings::PRODUCT,
    )
}

/// 带后台计算设置的可写挂载（[`mount_writable_with_background_compute_settings`]）没做成：设置被拒，或挂载自己没做成。
/// 与 [`MountError`] 分开报：设置那一拒在读任何一块盘之前，调用方要改的是挂载参数，不是池。
#[derive(Debug)]
pub enum WritableMountWithBackgroundComputeRefused {
    /// D26（后台整理与放置回收） 已定项 13 第 4 条：配错或 GPU 用不上，拒绝挂载，不降级到 `cpu`。在读任何一块盘之前拒，盘上逐字节不变。
    BackgroundComputeSettings(BackgroundComputeSettingsRefused),
    /// 设置校验过了，挂载自己没做成：同 [`mount_writable`] 的错。
    Mount(MountError),
}

/// 同 [`mount_writable`]，这次挂载的后台计算设置由调用方给（D26（后台整理与放置回收） 已定项 13 第 1–4 条：三个挂载参数只有一组、
/// 管全部后台任务），连同可选的一份 GPU 实现（同一已定项「挂载时由调用方把它的实现递进去」；产品 GPU 包实现
/// `background_compute::BackgroundGpuCompute`，卡清单也由它答）。先照第 4 条校验
/// （`background_compute::admit_background_compute_settings_with_the_gpu_compute`；选 `gpu` 而 `gpu_compute` 是 `None` 就是 GPU 不在），
/// 不过就在读任何一块盘之前拒绝挂载，不退回 `cpu`；过了就照 [`mount_writable`] 挂，设置连同实现装进交回的那份参数与盘表
/// （[`ParametersAndDeviceTableOfTheMount::background_compute_of_this_mount`]），挂载续做与之后的会话照它。
/// [`mount_writable`] 与别的可写挂载入口都等价于在这里给 `BackgroundComputeSettings::PRODUCT`（`cpu`）、不给实现。
/// GC 停着时在 CPU 上补跑的阈值取默认（[`mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold`]）。
///
/// # Errors
/// 设置被拒 ⇒ [`WritableMountWithBackgroundComputeRefused::BackgroundComputeSettings`]；
/// 挂载自己的错 ⇒ [`WritableMountWithBackgroundComputeRefused::Mount`]，同 [`mount_writable`]。
pub fn mount_writable_with_background_compute_settings<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    background_compute_settings: BackgroundComputeSettings,
    gpu_compute: Option<BackgroundGpuComputeHandle>,
) -> Result<Mounted, WritableMountWithBackgroundComputeRefused> {
    mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold(
        parameters,
        devices,
        background_compute_settings,
        gpu_compute,
        BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
    )
}

/// 同 [`mount_writable_with_background_compute_settings`]，另给挂载参数 `background_gc_cpu_compensation_threshold`
/// （GC 因一批计算出错停着时，池子已分配占总容量超过它就在 CPU 上补跑；用户 2026-10-28 定案，默认 85%）。它与后台计算设置同一套校验：
/// 超过 100% 就在读任何一块盘之前拒绝挂载（`BackgroundComputeSettingsRefused::GcCpuCompensationThresholdAboveTheWholeCapacity`），盘上逐字节不变。
///
/// # Errors
/// 同 [`mount_writable_with_background_compute_settings`]，设置被拒多一种阈值越界。
pub fn mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold<
    Device: BlockDevice,
>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    background_compute_settings: BackgroundComputeSettings,
    gpu_compute: Option<BackgroundGpuComputeHandle>,
    gc_cpu_compensation_threshold: BackgroundGcCpuCompensationThreshold,
) -> Result<Mounted, WritableMountWithBackgroundComputeRefused> {
    let background_compute = match admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold(
        background_compute_settings,
        gpu_compute,
        gc_cpu_compensation_threshold,
    ) {
        Ok(admitted) => admitted,
        Err(refused) => {
            return Err(
                WritableMountWithBackgroundComputeRefused::BackgroundComputeSettings(refused),
            )
        }
    };
    mount_writable_with_every_test_only_switch(
        parameters,
        devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        DeletionSwitches::PRODUCT,
        CompactionSwitches::PRODUCT,
        BeforeTheOneReread::RereadImmediately,
        InstanceTableSweepConfiguration::PRODUCT,
        background_compute,
    )
    .map_err(WritableMountWithBackgroundComputeRefused::Mount)
}

/// 同 [`mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold`]，另给整理的两样挂载参数：批大小与三条水位线
/// （[`CompactionMountParameters`]；D26（后台整理与放置回收） 已定项 1，不存盘）。挂载续做盘上那一条未完成的整理意图照这个批大小推。
/// 挂载就起周期线程的运行库（`singlefs_mount_runtime`）走这一个入口。
///
/// # Errors
/// 同 [`mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold`]。
pub fn mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_parameters<
    Device: BlockDevice,
>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    background_compute_settings: BackgroundComputeSettings,
    gpu_compute: Option<BackgroundGpuComputeHandle>,
    gc_cpu_compensation_threshold: BackgroundGcCpuCompensationThreshold,
    compaction_parameters: CompactionMountParameters,
) -> Result<Mounted, WritableMountWithBackgroundComputeRefused> {
    let CompactionSwitches {
        batch_size: _,
        forced_region,
        run_forcing,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    } = CompactionSwitches::PRODUCT;
    let mut mounted =
        mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_switches(
            parameters,
            devices,
            background_compute_settings,
            gpu_compute,
            gc_cpu_compensation_threshold,
            CompactionSwitches {
                batch_size: CompactionBatchSize::OfTheMount(compaction_parameters.batch_units),
                forced_region,
                run_forcing,
                region_exclusion,
                destination_policy,
                release_checksum_mismatch_after_issue,
                empty_cluster_segments_floor,
                relocation_publish_order,
            },
        )?;
    mounted.allocator.compaction_mut().watermarks = compaction_parameters.watermarks;
    Ok(mounted)
}

/// 同 [`mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold`]，整理的只供测试的开关
/// （`compaction_plan::CompactionSwitches`，判决 G6）也由调用方给（[`mount_writable_with_compaction_switches`] 那一种做法）：装在交回的分配器上，
/// 挂载续做盘上那一条未完成的整理意图照它与这次挂载给的后台计算设置推，这次挂载之后的会话照它。可写挂载续做里一批在 GPU 上出错停下那一种
/// 由它强制进入（`.claude/rules/fs-design.md` 五条硬要求第 2 条；判决 `research/prompts/m4-gpugc-code-r2-main-verification.md` 四之一）：
/// 产品路径的入口只有批大小可调（[`mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_parameters`]）。
/// 与别的只供测试的开关一样在发布构建里够得着（C921（只供测试的开关在发布构建里够得着））。
///
/// # Errors
/// 同 [`mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold`]。
pub fn mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_switches<
    Device: BlockDevice,
>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    background_compute_settings: BackgroundComputeSettings,
    gpu_compute: Option<BackgroundGpuComputeHandle>,
    gc_cpu_compensation_threshold: BackgroundGcCpuCompensationThreshold,
    compaction_switches: CompactionSwitches,
) -> Result<Mounted, WritableMountWithBackgroundComputeRefused> {
    let background_compute =
        admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold(
            background_compute_settings,
            gpu_compute,
            gc_cpu_compensation_threshold,
        )
        .map_err(WritableMountWithBackgroundComputeRefused::BackgroundComputeSettings)?;
    mount_writable_with_every_test_only_switch(
        parameters,
        devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        DeletionSwitches::PRODUCT,
        compaction_switches,
        BeforeTheOneReread::RereadImmediately,
        InstanceTableSweepConfiguration::PRODUCT,
        background_compute,
    )
    .map_err(WritableMountWithBackgroundComputeRefused::Mount)
}

/// 可写挂载择系统配置（S3，C707）：先按调用方给的本池 fsid 筛（[`choose_system_configuration_of_the_pool`]：自述别的 fsid 的槽当自证不过）；
/// 筛完一块盘可择的都没有（`NoValidSystemConfiguration`：盘上没有一槽是调用方说的这个池的）才不筛再择一次，择到的 fsid 与调用方的不同，
/// 由后面的参数核报成 [`MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`]（调用方交错了 fsid 那一形照旧报）。
/// 只读挂载与恢复带本池 fsid 的入口用同一个筛法（`recovery::choose_system_configuration_of_the_pool_naming_devices_without_a_mountable_slot`），
/// 不走不筛再择那一步：它们没有参数核，再择一次就读到调用方没点名的那个池。
///
/// # Errors
/// 筛过之后择系统配置报的错（不是一槽都没有的那一种）；不筛那一次报的错。
#[allow(
    clippy::ptr_arg,
    reason = "choose_system_configuration 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn system_configuration_of_the_pool_named_by_the_caller<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    filesystem_identifier_handed_in: &[u8; 16],
) -> Result<crate::system_configuration::SystemConfiguration, RecoveryFailure> {
    let among_the_slots_of_the_pool =
        choose_system_configuration_of_the_pool(devices, filesystem_identifier_handed_in);
    if matches!(
        among_the_slots_of_the_pool,
        Err(RecoveryFailure::NoValidSystemConfiguration { .. })
    ) {
        return choose_system_configuration(devices);
    }
    among_the_slots_of_the_pool
}

/// 可写挂载的全部只供测试的开关一处收：空间准入、影子账、删除开关、整理开关、「重读一次」之前的钩子、
/// 实例表清扫的配置；连同这次挂载给的后台计算设置（校验在调用方，[`mount_writable_with_background_compute_settings`]）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样只供测试的开关或挂载的输入，不成一个概念"
)]
fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    space_admission: SpaceAdmission,
    shadow_ledger: ShadowLedger,
    deletion_switches: DeletionSwitches,
    compaction_switches: CompactionSwitches,
    mut before_the_one_reread: BeforeTheOneReread<'_>,
    instance_table_sweep: InstanceTableSweepConfiguration,
    background_compute: AdmittedBackgroundComputeSettings,
) -> Result<Mounted, MountError> {
    admit_the_writable_device_count(distinct_device_identities_handed_in(devices))?;
    refuse_device_identities_handed_in_more_than_once(devices)?;
    let system_configuration = system_configuration_of_the_pool_named_by_the_caller(
        devices,
        &parameters.filesystem_identifier,
    )?;
    // 写入口按盘上择到的这一份建，不按调用方的（代码审阅第 17 条）；调用方的参数、盘表的盘数与每块盘的本盘设备号与它不一致
    // 就在任何写之前拒。
    let parameters_of_the_pool =
        parameters_of_the_selected_system_configuration_agreeing_with_the_caller_parameters_and_device_table(
            parameters,
            devices,
            &system_configuration,
        )?;
    // 单元区起点按择到的系统配置的环长现算（与偏移 417 那 8 字节相等，择的时候判过）：建空闲图、判分配记录与被抛弃根的落点都按它。
    let unit_area_start = unit_area_start_of_the_chosen_system_configuration(&system_configuration);
    // 每块盘都到得了单元区起点（代码审阅第 35 条）：重建分配器按盘的字节数建空闲图（`DeviceFreeMap::with_unit_area_start`），盘比单元区起点
    // 还短时那里的减法下溢；在读根环之前拒，盘上逐字节不变。
    every_device_reaches_the_unit_area_start(&*devices, unit_area_start)?;
    // 读阶段：择根、扫 journal、重放；系统配置见证过比所选那一版新的发布就重读一遍，仍判真就在这里拒（C554 乙）。
    let (
        ReadStage {
            chosen_root,
            self_verified_roots_read: roots_read_by_the_read_stage,
            records,
            journal,
            effective_root,
            against_the_witness: _,
            publishes_missing_records_whose_named_units_do_not_verify: _,
        },
        read_stage_settled,
    ) = read_stage_settled_at_most_on_the_one_reread(
        &*devices,
        &system_configuration,
        &mut before_the_one_reread,
    )?;
    let own_record = record_standing_for_the_version(&records, &effective_root);
    let selected_version_journal_position =
        journal_position_of_the_selected_version(&effective_root, own_record.as_ref());
    // 判退不看建上一版通没通，看三样读者的并集（C755，用户 2026-10-20 定「三句都补」；C866 改法甲：并集里有内容拒收不退），与恢复、只读挂载
    // 同一份（`recovery::fall_back_judged_on_the_three_readers`；F2 乙，用户 2026-10-07 定「三处都做」）。退过去不重放：所选根与所选那一版都换成它，
    // 扫环报告里施加的那一截清零（新实例写的上一个实例那一行 W 取 0，与落在一条根上、一条记录都没施加时相同）。
    // 判法、建退到那一版要的那条记录、F1 乙再判（`F1_AFTER_F2`）同用触发之后重扫的那一遍（D23（journal 的角色与格式） 已定项 14，用户 2026-10-20 定）。
    // 退过去的那一版建不出来，留在所选的那一版：成不成照建所选那一版。之后的一切（取号之前逐盘核、写行、影子账与 C）都按落到的那一版。
    // 退到哪一版照 `F2_NEWEST`：[见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根（判法在 `recovery` 那个函数里）。
    // 建上一版（退到的那一版与所选的那一版两处）按位置条目读单元，经一层数「只坏一份」的读（D4（校验和位置） 已定项 4「读到只坏一份时计数」）；
    // 读法与报给盘的几样原样交给底下这串盘，数出来的挂载交回之前记进这次挂载的状态
    // （`CompactionOfThisMount::count_unit_reads_with_one_copy_failing_while_mounting`）。判退的三样读者不经它：它们读的是建上一版接着要读的同几个单元，
    // 判完不留字节，经它就把同一份坏副本多数几遍。
    let devices_counting_unit_reads_with_one_copy_failing =
        PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*devices);
    // 判退之后 `chosen_root` 换成退到的那一条；新实例第一个 txg 的下界、树 ID 水位要盖住读阶段择到的根（`first_txg_of_new_instance`、
    // `tree_identifier_watermark_of_the_ring`：择根取环里最新的，读阶段读到的根都不超过它），C 并的较新可读根要盖住读阶段读到的全部根
    // （`rebuilt_allocator`：判退退掉的那一条与它和退到的那一版之间读阶段读得出的那几条，之后几遍读它们的根槽读坏时照样在回退候选集里），
    // 判抛弃用读阶段择到的那条根的实例表（[`instance_table_of_the_newest_root_read_at_most_twice`]：判退之后也用它，与判退本身滤候选用的同一张）。
    let root_chosen_by_the_read_stage = chosen_root;
    let fall_back_rebuilt = match fall_back_judged_on_the_three_readers(
        &*devices,
        &system_configuration,
        &chosen_root,
        &effective_root,
        &records,
    ) {
        None => None,
        Some(fall_back) => {
            let version_fallen_back_to = fall_back.version_fallen_back_to;
            let record_standing_for_the_version_fallen_back_to = record_standing_for_the_version(
                &fall_back.records_rescanned_after_the_trigger,
                &version_fallen_back_to,
            );
            let journal_position_of_the_version_fallen_back_to =
                journal_position_of_the_selected_version(
                    &version_fallen_back_to,
                    record_standing_for_the_version_fallen_back_to.as_ref(),
                );
            rebuild_previous_version(
                &devices_counting_unit_reads_with_one_copy_failing,
                unit_area_start,
                &version_fallen_back_to,
                record_standing_for_the_version_fallen_back_to,
            )
            .ok()
            .map(|previous| {
                (
                    version_fallen_back_to,
                    journal_position_of_the_version_fallen_back_to,
                    previous,
                    fall_back.records_rescanned_after_the_trigger,
                )
            })
        }
    };
    let (chosen_root, effective_root, journal, selected_version_journal_position, previous) =
        match fall_back_rebuilt {
            Some((
                version_fallen_back_to,
                journal_position_of_the_version_fallen_back_to,
                previous,
                records_rescanned_after_the_trigger,
            )) => {
                // F1 乙按退到的那一版的水位再判一次（`F1_AFTER_F2`）：读阶段判 F1 时水位是退之前的所选那一版。用触发之后重扫的那一遍
                // （C754「再判之前重扫环」与 C755「同用触发之后那一遍」）：读阶段扫环时一时读不出的槽，这一次可能读得出，
                // 拿读阶段那一份判会把一次瞬时读错当成缺口。
                refuse_publishes_above_the_version_fallen_back_to_missing_records_whose_named_units_do_not_verify(
                    &*devices,
                    &system_configuration,
                    &records_rescanned_after_the_trigger,
                    &version_fallen_back_to,
                    &mut before_the_one_reread,
                )?;
                (
                    version_fallen_back_to,
                    version_fallen_back_to,
                    JournalScanReport {
                        valid_records: journal.valid_records,
                        above_water: journal.above_water,
                        prefix_applied: 0,
                        verification_passed: journal.verification_passed,
                        verification_failed: journal.verification_failed,
                        maximum_applied_transaction: 0,
                    },
                    journal_position_of_the_version_fallen_back_to,
                    previous,
                )
            }
            None => {
                let previous = rebuild_previous_version(
                    &devices_counting_unit_reads_with_one_copy_failing,
                    unit_area_start,
                    &effective_root,
                    own_record,
                )?;
                (
                    chosen_root,
                    effective_root,
                    journal,
                    selected_version_journal_position,
                    previous,
                )
            }
        };
    let unit_reads_with_one_copy_failing_while_building_on_the_version =
        devices_counting_unit_reads_with_one_copy_failing.unit_reads_with_one_copy_failing();
    // 读到只坏一份的那几个单元的位置条目随建出来的那一版交下去（`PreviousVersion` 带着），挂载交回之前按挂载之后那一版认成映射 key。
    let mut previous = previous;
    previous.record_the_units_read_with_one_copy_failing_while_building_on_it(
        devices_counting_unit_reads_with_one_copy_failing
            .locations_of_the_units_read_with_one_copy_failing(),
    );
    // 环里没有记录时从 1 起（D23（journal 的角色与格式） 已定项 14 第 3 条：计数器全池接着走）。
    let next_counter = records
        .values()
        .map(|record| record.counter)
        .max()
        .unwrap_or(0)
        + 1;
    let first_txg = first_txg_of_new_instance(
        devices,
        &system_configuration,
        &records,
        &root_chosen_by_the_read_stage,
    )?;
    let tree_identifier_watermark_of_the_ring = tree_identifier_watermark_of_the_ring(
        devices,
        &system_configuration,
        &records,
        &effective_root,
        &root_chosen_by_the_read_stage,
    );
    let rebuilt = rebuilt_allocator(
        devices,
        &system_configuration,
        &previous,
        first_txg,
        &records,
        &effective_root,
        &root_chosen_by_the_read_stage,
        &roots_read_by_the_read_stage,
        shadow_ledger,
        &mut before_the_one_reread,
    );
    let RebuiltAllocator {
        mut allocator,
        effective_floor,
        abandoned_roots_unreadable,
        instance_table_of_the_newest_root,
        root_ring_of_the_rebuilt_allocator,
    } = rebuilt?;
    allocator.set_deletion_switches(deletion_switches);
    allocator.set_compaction_switches(compaction_switches);
    // 重建时读回活意图、恢复 R 排他（D26（后台整理与放置回收） 已定项 1 第 3 条：意图住 logged_ops，分配器读得到）：在写行、暖机、抬 F 与任何
    // 续做之前装上，这次挂载里每一次发布的新落点都不进 R（暂停是会话那一侧的事，挂载里不暂停）。
    if let Some((_issued_txg, intent)) = previous
        .version_of_trees()
        .and_then(|version| live_compaction_intent_of_the_entries(&version.logged_operations))
    {
        allocator.compaction_mut().issue(intent);
    }
    let previous_row = PreviousInstanceRow {
        instance: effective_root.instance,
        selected_root_txg: effective_root.checkpoint_txg,
        applied_transaction_high_water: journal.maximum_applied_transaction,
    };
    establish_instance(
        &parameters_of_the_pool,
        devices,
        allocator,
        InstanceStart {
            chosen_root,
            effective_root,
            journal,
            previous,
            previous_row,
            first_txg,
            next_counter,
            tree_identifier_watermark_of_the_ring,
            effective_floor,
            abandoned_roots_unreadable,
            shadow_ledger,
            system_configuration,
            space_admission,
            selected_version_journal_position,
            rereads: RereadsOfThisMount {
                read_stage: read_stage_settled,
                instance_table_of_the_newest_root,
                root_ring_of_the_rebuilt_allocator,
            },
            instance_table_sweep,
            background_compute,
            unit_reads_with_one_copy_failing_while_building_on_the_version,
        },
    )
}
