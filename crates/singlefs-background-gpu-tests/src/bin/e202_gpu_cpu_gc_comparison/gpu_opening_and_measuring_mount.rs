//! 一次跑的开卡与量挂载：Vulkan 实例、进程那一张开卡表、开卡与挂载。

use super::*;

// ---- 一次跑：开卡、挂载、起线程 ----

/// 量挂载那一刻开出来的 GPU：交挂载的把手、多判包装的记录（多判的 GPU 臂才有），与卸载之后泄漏了哪几张卡。
pub(super) struct GpuOfTheRun {
    pub(super) gpu_compute: Option<BackgroundGpuComputeHandle>,
    pub(super) rejudge_log: Option<SharedRejudgeLog>,
    pub(super) gpus_leaked_when_dropped: Option<GpusLeakedWhenTheBackgroundGpuIsDropped>,
}

/// GPU 臂开卡用的那一份进程级 Vulkan 实例（取卡时建的，[`cards_seen_by_the_command`]，对卡与开卡都用它）；这条命令没有它（服务交的卡里没有这一台的）交回 `Err`。
pub(super) fn vulkan_instance_for_the_gpu_arm(
    vulkan_instance: Option<&VulkanInstanceOfThisProcess>,
) -> Result<&VulkanInstanceOfThisProcess, String> {
    vulkan_instance
        .ok_or_else(|| "GPU 臂要开卡，这条命令却没建进程级 Vulkan 实例（没枚举独显）".to_string())
}

/// 这个进程里各张卡上在开、到点没回来的开卡那一张表（D26（后台整理与放置回收） 已定项 13 第 4 条：表在进程级，各次开卡共用；
/// 前一次挂住的卡，之后的开卡同样不再起，出路只有重启进程）。装置不经产品的进程开卡器开卡、手里没有开卡器持有的那一张，
/// 所以在这里放一份：整个进程只建一次，装置里的每一处开卡都取这一份（[`openings_not_returned_in_this_process`]）。
static OPENINGS_NOT_RETURNED_IN_THIS_PROCESS: std::sync::OnceLock<
    OpeningsNotReturnedInThisProcess,
> = std::sync::OnceLock::new();

/// 装置每一处开卡交给产品开卡入口的那一张表：头一次问的时候建，之后每次交同一张（[`OPENINGS_NOT_RETURNED_IN_THIS_PROCESS`]）。
pub(super) fn openings_not_returned_in_this_process() -> &'static OpeningsNotReturnedInThisProcess {
    OPENINGS_NOT_RETURNED_IN_THIS_PROCESS
        .get_or_init(OpeningsNotReturnedInThisProcess::with_none_not_returned)
}

/// 照这条臂开卡（挂载与开卡都在第一次 GC 调用之前做完、在窗口外：登记第一节「不含挂载那一刻量代价」），在 `vulkan_instance` 上开。
pub(super) fn open_the_gpu_of_the_arm(
    arm: LockstepArm,
    settings: &BackgroundComputeSettings,
    memory_leaked: &GpuMemoryLeakedInThisProcess,
    vulkan_instance: Option<&VulkanInstanceOfThisProcess>,
) -> Result<GpuOfTheRun, String> {
    if !arm.compute.uses_the_gpu() {
        return Ok(GpuOfTheRun {
            gpu_compute: None,
            rejudge_log: None,
            gpus_leaked_when_dropped: None,
        });
    }
    let vulkan_instance = vulkan_instance_for_the_gpu_arm(vulkan_instance)?;
    let selection = BackgroundGpuSelection::of_the_settings(settings);
    match arm.rejudging {
        RejudgingOfTheArm::NotRejudged => {
            let opened = open_background_gpu_compute(
                &selection,
                memory_leaked,
                openings_not_returned_in_this_process(),
                vulkan_instance,
            )
            .map_err(|refused| format!("开后台 GPU：{refused:?}"))?;
            Ok(GpuOfTheRun {
                gpu_compute: Some(opened.gpu_compute),
                rejudge_log: None,
                gpus_leaked_when_dropped: Some(opened.gpus_leaked_when_dropped),
            })
        }
        RejudgingOfTheArm::EveryConfirmedEntryRejudgedOnTheCpu => {
            let background_gpu = open_background_gpu_with_the_memory_leaked_in_this_process(
                &selection,
                memory_leaked,
                openings_not_returned_in_this_process(),
                vulkan_instance,
                |_gpus_to_open| {},
            )
            .map_err(|refused| format!("开后台 GPU：{refused:?}"))?;
            let gpus_leaked_when_dropped = background_gpu.gpus_leaked_when_dropped();
            let rejudge_log: SharedRejudgeLog = Arc::new(Mutex::new(RejudgeLog::default()));
            let rejudged = GpuComputeRejudgedOnTheCpu {
                inner: BackgroundGpuComputeOnTheParticipatingGpus::with_no_entry_uncertain(
                    background_gpu,
                ),
                log: Arc::clone(&rejudge_log),
            };
            Ok(GpuOfTheRun {
                gpu_compute: Some(BackgroundGpuComputeHandle::of_an_implementation(rejudged)),
                rejudge_log: Some(rejudge_log),
                gpus_leaked_when_dropped: Some(gpus_leaked_when_dropped),
            })
        }
    }
}

/// 从起步镜像拷一份、照设置可写挂载、装上开关（登记第五节「量挂载」）。
pub(super) fn measuring_mount(
    image: &StartingImage,
    settings: BackgroundComputeSettings,
    gpu_compute: Option<BackgroundGpuComputeHandle>,
    batch_units: usize,
) -> Result<(MountedSession, PoolDevices, SharedRecordingDigest), String> {
    let (mut devices, recording) = copied_pool_devices(&image.devices);
    let mounted = mount_writable_with_background_compute_settings(
        &make_filesystem_parameters_of_the_pool(),
        &mut devices,
        settings,
        gpu_compute,
    )
    .map_err(|refused| format!("量挂载：{refused:?}"))?;
    let (_mount_output, mut session) = MountedSession::of_the_mount(mounted);
    session
        .allocator
        .set_compaction_switches(compaction_switches_of_the_measuring_mount(batch_units));
    Ok((session, devices, recording))
}

#[cfg(test)]
mod tests;
