//! 端到端：产品 GPU 包的实现（`background_gpu_compute::open_background_gpu_compute` 开出来的把手，背后是真卡上三段的两份 WGSL 核）递进核心层的可写挂载、
//! 选 `gpu`，推一趟整理到停；同一历史的另一个池选 `cpu` 推一趟。两趟发的每一版、录制流、盘上字节、停在哪、搬了几批、整理观测与 discard 逐项相同
//! （D26（后台整理与放置回收） 已定项 13「GPU 路径与 CPU 路径是两份实现……两份实现在测试里对拍」）。另钉住这份实现答的卡清单：开包时开着的本机全部独显，
//! 点名交来的卡时参与的就是交来的那几张、次序照点名的次序；交来一组只点其中一张的设置之后参与的卡只剩它、卡清单不变（不重开），一批照常只在它上面算。
//! 池的搭建在 `common_pool_on_memory_devices`（同 harness 档 `build_pool` 那一串，盘在内存里）。
//! 开卡与挂载只点名服务经 `GPU_CARDS_BY_PRIORITY` 交给这个进程、属于这一台的那几张卡（`common_discrete_gpus_of_this_host`）：服务没交这一台的卡时每条打一行
//! `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>` 报「本次未跑」、返回成功；交了卡却对不上本机的独显、或开不了判红。源头没声明张数，服务分一张。
//! 只在开了特性 `tests-on-a-real-gpu` 时编；用 GPU，经 `task-server/task submit` 交（卡与清场由服务定），带 `--test-threads 1`。
//! 开卡与挂载给每张参与的卡的显存上限 1 GiB（七·3 验收，用户 2026-11-01 定；`common_discrete_gpus_of_this_host`）。
//! 七·3 验收（用户 2026-11-01 定）的另几格也在这里：没有卡时拒绝挂载（本进程照原样再起一次自己、把 Vulkan 驱动全藏起来，那一次里一张独显都看不到）；
//! 挂着时卡出错就报警、GC 停下（挂着切到等 GPU 的上限 1 纳秒：一批交出去就等超过上限、重交 3 次都超时，第 4 条「含重交 3 次都超时」那一种 GPU 出错）；
//! 停下之后占用超过阈值在 CPU 上补跑、结果与 CPU 模式相同，不超过时照旧停着。
mod common_discrete_gpus_of_this_host;
mod common_pool_on_memory_devices;

use std::os::unix::process::CommandExt;
use std::time::Duration;

use common_discrete_gpus_of_this_host::{
    discrete_gpus_on_this_host, handed_over_gpus_or_report_not_run,
    identifiers_of_the_handed_over_gpus, memory_limits_of_one_gibibyte_each, HandedOverGpu,
};
use common_pool_on_memory_devices::{
    parameters, pool_after_a_sequential_write_in_an_earlier_mount, PoolOnMemoryDevices,
};
use singlefs_background_gpu::adapter_inventory::BackgroundGpuSelection;
use singlefs_background_gpu::background_gpu::{
    open_background_gpu, BackgroundGpu, BackgroundGpuOpenRefused,
};
use singlefs_background_gpu::background_gpu_compute::open_background_gpu_compute;
use singlefs_background_gpu::checksum_kernels::{cpu_reference_outcome, UnitChecksumToVerify};
use singlefs_background_gpu::gpus_of_this_host::GpuMemoryLeakedInThisProcess;
use singlefs_core::background_compute::{
    admit_background_compute_settings_with_the_gpu_compute,
    admit_gpu_settings_against_the_inventory, AlarmOfABackgroundTaskStoppedAfterAComputeError,
    BackgroundComputeBackend, BackgroundComputePathTaken, BackgroundComputeSettings,
    BackgroundComputeSettingsRefused, BackgroundGcCpuCompensationThreshold,
    BackgroundGpuComputeHandle, BackgroundTask, BackgroundTaskAfterTheComputeError,
    GpuDeviceIdentifier, GpuInventory, GpuMemoryInBytes, ReasonTheBatchRanOnTheCpu,
    BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
};
use singlefs_core::compaction_plan::CompactionRunStop;
use singlefs_core::mount::{
    mount_writable_with_background_compute_settings, WritableMountWithBackgroundComputeRefused,
};
use singlefs_core::mounted_session::{CompactionRun, CompactionRunOutcome, MountedSession};

/// 等 GPU 的上限：这几批在健康的卡上远用不了这么久。
const WAIT_LIMIT_OF_THE_GPU_TESTS: Duration = Duration::from_secs(60);

/// 选 `gpu`、点名服务交来的那几张卡（照交来的次序）、每张卡的显存上限 1 GiB。
fn gpu_settings_on_the_handed_over_gpus(
    handed_over_gpus: &[HandedOverGpu],
) -> BackgroundComputeSettings {
    BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: Some(identifiers_of_the_handed_over_gpus(handed_over_gpus)),
        background_gpu_memory_limit: Some(memory_limits_of_one_gibibyte_each(
            handed_over_gpus.len(),
        )),
        background_gpu_wait_limit: WAIT_LIMIT_OF_THE_GPU_TESTS,
    }
}

/// 选 `gpu`、不点名（本机全部独显）、每张卡的显存上限 1 GiB：只给 Vulkan 驱动全藏起来的那一次用（一张独显都看不到，上限列表是空的，开卡照「GPU 不在」拒）。
fn gpu_settings_on_every_discrete_gpu() -> BackgroundComputeSettings {
    BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: None,
        background_gpu_memory_limit: Some(memory_limits_of_one_gibibyte_each(
            discrete_gpus_on_this_host(WAIT_LIMIT_OF_THE_GPU_TESTS).len(),
        )),
        background_gpu_wait_limit: WAIT_LIMIT_OF_THE_GPU_TESTS,
    }
}

/// 照挂载那一组设置（点名交来的卡）开卡、交回把手。卡是服务交来、对上了本机适配器的，开不了判红。
fn opened_gpu_compute(settings: &BackgroundComputeSettings) -> BackgroundGpuComputeHandle {
    match open_background_gpu_compute(
        &BackgroundGpuSelection::of_the_settings(settings),
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    ) {
        Ok(opened) => opened.gpu_compute,
        Err(refused) => {
            panic!("服务交来、对上了本机适配器的卡开不了后台 GPU：{refused:?}")
        }
    }
}

/// 照挂载那一组设置（点名交来的卡）开产品 GPU 包的后台 GPU。卡是服务交来、对上了本机适配器的，开不了判红。
fn opened_background_gpu_of_the_settings(settings: &BackgroundComputeSettings) -> BackgroundGpu {
    open_background_gpu(&BackgroundGpuSelection::of_the_settings(settings)).unwrap_or_else(
        |refused| panic!("服务交来、对上了本机适配器的卡开不了后台 GPU：{refused:?}"),
    )
}

#[test]
fn the_compaction_run_on_the_real_gpus_ends_exactly_as_the_cpu_path() {
    let Some(handed_over_gpus) = handed_over_gpus_or_report_not_run(
        "the_compaction_run_on_the_real_gpus_ends_exactly_as_the_cpu_path",
        WAIT_LIMIT_OF_THE_GPU_TESTS,
    ) else {
        return;
    };
    let settings = gpu_settings_on_the_handed_over_gpus(&handed_over_gpus);
    let gpu_compute = opened_gpu_compute(&settings);
    let history = pool_after_a_sequential_write_in_an_earlier_mount();
    let mut cpu_pool = history.copied_with_an_empty_stream();
    let mut gpu_pool = history.copied_with_an_empty_stream();
    drop(history);
    let mut cpu_session = cpu_pool
        .remounted_with_compaction_forced_to_start(BackgroundComputeSettings::PRODUCT, None);
    let cpu_run = cpu_session
        .run_compaction(&mut cpu_pool.devices)
        .expect("整理一趟");
    let mut gpu_session =
        gpu_pool.remounted_with_compaction_forced_to_start(settings, Some(gpu_compute));
    let gpu_run = gpu_session
        .run_compaction(&mut gpu_pool.devices)
        .expect("整理一趟");
    assert!(
        gpu_run.batches_published > 0,
        "这一趟签了意图、搬了批（不然这条用例什么都没比）：{:?}",
        gpu_run.outcome
    );
    assert_eq!(
        format!("{:?}", gpu_run.outcome),
        format!("{:?}", cpu_run.outcome),
        "停在同一条上"
    );
    assert_eq!(
        gpu_run.versions_published, cpu_run.versions_published,
        "发的每一版逐项相同"
    );
    assert_eq!(
        gpu_run.batches_published, cpu_run.batches_published,
        "搬了一样多批"
    );
    assert_eq!(
        format!("{:?}", gpu_run.discards),
        format!("{:?}", cpu_run.discards),
        "发布之后的 discard 相同"
    );
    assert_eq!(
        gpu_session.compaction.observations, cpu_session.compaction.observations,
        "整理观测逐项相同"
    );
    assert_eq!(
        gpu_pool.recorded_operations(),
        cpu_pool.recorded_operations(),
        "重挂之后的录制流逐步相同"
    );
    assert_eq!(
        gpu_pool.written_sectors_of_every_device(),
        cpu_pool.written_sectors_of_every_device(),
        "盘上逐字节相同"
    );
    let mut confirmed_on_the_gpus = 0;
    for batch in &gpu_run.background_compute_of_each_batch {
        let BackgroundComputePathTaken::Gpu(gpu_work) = &batch.path_taken else {
            panic!("gpu 那一趟每一批走 GPU：{batch:?}");
        };
        let entries = gpu_work.entries_of_the_three_segments();
        assert_eq!(
            entries.uncertain_this_round + entries.judged_by_the_cpu_reference,
            0,
            "健康的卡上两份核逐条相同：全部确认，{gpu_work:?}"
        );
        assert_eq!(
            gpu_work.units_whose_checksum_mismatched_on_the_gpu, 0,
            "健康的盘上整单元 CRC 都对得上：{gpu_work:?}"
        );
        confirmed_on_the_gpus += entries.confirmed_by_both_kernels;
    }
    assert!(confirmed_on_the_gpus > 0, "确认的条目数报出来了");
}

/// 这份实现答的卡清单是本机全部独显与各自的显存总量（核心层挂载照它校验）；点名交来的卡时参与的就是交来的那几张、次序照点名的次序，
/// 每张都在卡清单里（卡清单另含本机没点名的独显）。点名参与的那张卡照收，点名不在本机的卡照「点名的卡不存在」拒。
#[test]
fn the_implementation_answers_the_participating_gpus_as_the_inventory_the_core_admits_against() {
    let test_name =
        "the_implementation_answers_the_participating_gpus_as_the_inventory_the_core_admits_against";
    let Some(handed_over_gpus) =
        handed_over_gpus_or_report_not_run(test_name, WAIT_LIMIT_OF_THE_GPU_TESTS)
    else {
        return;
    };
    let settings = gpu_settings_on_the_handed_over_gpus(&handed_over_gpus);
    let background_gpu = opened_background_gpu_of_the_settings(&settings);
    let participating: Vec<GpuDeviceIdentifier> = background_gpu
        .participating_gpus()
        .into_iter()
        .map(|report| report.identifier)
        .collect();
    let inventory: Vec<GpuDeviceIdentifier> = background_gpu
        .devices_with_total_memory()
        .into_iter()
        .map(|(identifier, _total_memory)| identifier)
        .collect();
    assert_eq!(
        participating,
        identifiers_of_the_handed_over_gpus(&handed_over_gpus),
        "参与的卡就是点名的交来的卡，次序照点名的次序"
    );
    assert!(
        participating.iter().all(|card| inventory.contains(card)),
        "参与的卡都在卡清单里（卡清单是本机全部独显）：参与的 {participating:?}，卡清单 {inventory:?}"
    );
    drop(background_gpu);
    let gpu_compute = opened_gpu_compute(&settings);
    let naming = |device: GpuDeviceIdentifier| BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: Some(vec![device]),
        background_gpu_memory_limit: Some(memory_limits_of_one_gibibyte_each(1)),
        background_gpu_wait_limit: WAIT_LIMIT_OF_THE_GPU_TESTS,
    };
    assert!(
        admit_background_compute_settings_with_the_gpu_compute(
            naming(participating[0].clone()),
            Some(gpu_compute.clone()),
        )
        .is_ok(),
        "点名参与的那张卡：照收"
    );
    let card_not_on_this_host = GpuDeviceIdentifier::of_opaque_bytes(b"ffff:ff:ff.f".to_vec());
    assert_eq!(
        admit_background_compute_settings_with_the_gpu_compute(
            naming(card_not_on_this_host.clone()),
            Some(gpu_compute),
        )
        .map(|_admitted| ()),
        Err(
            BackgroundComputeSettingsRefused::NamedGpuDeviceNotInTheInventory {
                device: card_not_on_this_host
            }
        ),
        "点名不在本机的卡：拒"
    );
}

/// 第 2、3 条与第 1 条「切换不重开」：开包时点名服务交来的卡，它们都参与；交来一组只点交来的头一张、上限 1 GiB 的设置之后，
/// 参与的卡只剩那一张、上限照给的，卡清单照旧是开包时那一份（本机全部独显）；之后一批校验和核对照常算完、与 CPU 参照逐条相同。
#[test]
fn settings_naming_one_card_narrow_the_participating_gpus_without_reopening_the_cards() {
    let test_name =
        "settings_naming_one_card_narrow_the_participating_gpus_without_reopening_the_cards";
    let Some(handed_over_gpus) =
        handed_over_gpus_or_report_not_run(test_name, WAIT_LIMIT_OF_THE_GPU_TESTS)
    else {
        return;
    };
    let settings = gpu_settings_on_the_handed_over_gpus(&handed_over_gpus);
    let mut background_gpu = opened_background_gpu_of_the_settings(&settings);
    let inventory_at_the_opening = background_gpu.devices_with_total_memory();
    let named_card = handed_over_gpus[0].identifier();
    let memory_limit_of_the_named_card: Vec<GpuMemoryInBytes> =
        memory_limits_of_one_gibibyte_each(1);
    let narrowed = admit_gpu_settings_against_the_inventory(
        &BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: Some(vec![named_card.clone()]),
            background_gpu_memory_limit: Some(memory_limit_of_the_named_card.clone()),
            background_gpu_wait_limit: WAIT_LIMIT_OF_THE_GPU_TESTS,
        },
        &background_gpu,
    )
    .expect("点名卡清单里的一张、上限 1 GiB");
    background_gpu.use_the_settings_from_the_next_batch(&narrowed);
    let participating: Vec<(GpuDeviceIdentifier, Option<GpuMemoryInBytes>)> = background_gpu
        .participating_gpus()
        .into_iter()
        .map(|report| (report.identifier, report.memory_limit))
        .collect();
    assert_eq!(
        participating,
        vec![(named_card, memory_limit_of_the_named_card.first().copied())],
        "参与的卡只剩点名的那一张，上限照给的"
    );
    assert_eq!(
        background_gpu.devices_with_total_memory(),
        inventory_at_the_opening,
        "卡清单照旧是开包时那一份：不重开"
    );
    let unit_bytes: Vec<Vec<u8>> = (0_u8..8).map(|fill| vec![fill; 4096]).collect();
    let units: Vec<UnitChecksumToVerify<'_, usize>> = unit_bytes
        .iter()
        .enumerate()
        .map(|(position, bytes)| UnitChecksumToVerify {
            entry_key: position,
            unit_bytes: bytes,
            expected_crc32c: 0,
        })
        .collect();
    let outcomes = background_gpu
        .both_kernel_outcomes_of_the_batch(&units)
        .expect("点名的那一张卡上照常算完");
    let expected: Vec<_> = units
        .iter()
        .map(|unit| {
            let outcome = cpu_reference_outcome(unit.unit_bytes, unit.expected_crc32c);
            (outcome, outcome)
        })
        .collect();
    assert_eq!(outcomes, expected, "两份核都与 CPU 参照逐条相同");
}

/// 设着它时 [`with_every_vulkan_driver_hidden_a_mount_choosing_gpu_is_refused_before_any_disk_byte_changes`] 照「Vulkan 驱动全藏起来」那一边跑
/// （外面那一次起它的时候设）。
const VARIABLE_MARKING_THE_RUN_WITH_EVERY_VULKAN_DRIVER_HIDDEN: &str =
    "SINGLEFS_REAL_GPU_TEST_WITH_EVERY_VULKAN_DRIVER_HIDDEN";
/// 驱动全藏起来的那一次判过之后打的那一行的开头：外面那一次认它，认不到判红。
const LINE_OF_THE_RUN_WITH_EVERY_VULKAN_DRIVER_HIDDEN: &str =
    "MOUNT_REFUSED_WITH_EVERY_VULKAN_DRIVER_HIDDEN";
/// 等 GPU 的上限 1 纳秒：一段的每一处等（等错误范围交回、等提交做完、等回读映射好）都先看期限、再看做没做完，起算之后再读一次钟就过了期限
/// （`waiting_for_the_gpu::wait_for_completion_before_the_deadline`；推的，没量过两次读钟之间隔多久），这一段次次等超过上限、重交 3 次都超时，
/// 第 4 条「含重交 3 次都超时」那一种 GPU 出错。
const WAIT_LIMIT_EXCEEDED_BY_EVERY_SUBMISSION: Duration = Duration::from_nanos(1);
/// 补跑阈值 0%：池子里有一个字节已分配就超过。
const THRESHOLD_EXCEEDED_BY_ANY_ALLOCATED_BYTE: BackgroundGcCpuCompensationThreshold =
    BackgroundGcCpuCompensationThreshold {
        allocated_percent_of_the_capacity: 0,
    };

/// 驱动全藏起来的那一次里判的：照产品那一份判法数出 0 张独显；产品 GPU 包照挂载那一组设置开卡报「GPU 不在」；同一历史拷的池选 `gpu`、
/// 不递实现（开不出把手）的可写挂载拒（`GpuNotPresent`），盘上逐字节不变、一个改盘的动作都没录到。
fn the_mount_choosing_gpu_is_refused_with_no_discrete_gpu_visible() {
    let discrete_gpus = discrete_gpus_on_this_host(WAIT_LIMIT_OF_THE_GPU_TESTS);
    assert!(
        discrete_gpus.is_empty(),
        "Vulkan 驱动全藏起来之后一张独显都看不到：{discrete_gpus:?}"
    );
    let settings = gpu_settings_on_every_discrete_gpu();
    assert!(
        matches!(
            open_background_gpu_compute(
                &BackgroundGpuSelection::of_the_settings(&settings),
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            ),
            Err(BackgroundGpuOpenRefused::SettingsRefused(
                BackgroundComputeSettingsRefused::GpuNotPresent
            ))
        ),
        "产品 GPU 包开卡：GPU 不在"
    );
    let history = pool_after_a_sequential_write_in_an_earlier_mount();
    let mut pool = history.copied_with_an_empty_stream();
    let refused = mount_writable_with_background_compute_settings(
        &parameters(),
        &mut pool.devices,
        settings,
        None,
    );
    assert!(
        matches!(
            refused,
            Err(
                WritableMountWithBackgroundComputeRefused::BackgroundComputeSettings(
                    BackgroundComputeSettingsRefused::GpuNotPresent
                )
            )
        ),
        "选 gpu、没有卡也就没有实现：拒绝挂载（{:?}）",
        refused.as_ref().err()
    );
    assert_eq!(
        pool.written_sectors_of_every_device(),
        history.written_sectors_of_every_device(),
        "盘上逐字节不变"
    );
    assert!(
        pool.recorded_operations().is_empty(),
        "一个改盘的动作都没录到"
    );
}

/// 七·3 验收「没有卡时拒绝挂载」（用户 2026-11-01 定）：本进程照原样再起一次自己、只跑这一条，那一次里把 Vulkan 驱动全藏起来
/// （`VK_DRIVER_FILES` 与 `VK_ICD_FILENAMES` 指到一份不存在的驱动清单、不设 `VK_ADD_DRIVER_FILES`：Vulkan 加载器一个驱动都找不到，
/// wgpu 一个适配器都枚举不到，与本机没有独显时产品看到的一样；推的，读 Vulkan 加载器的环境变量约定），在那一次里判
/// [`the_mount_choosing_gpu_is_refused_with_no_discrete_gpu_visible`]。外面那一次认那一次退 0、打了判过的那一行。
/// 起的那一次放进它自己的进程组；它跑完才交回。本机有没有卡这一条都跑。
#[test]
fn with_every_vulkan_driver_hidden_a_mount_choosing_gpu_is_refused_before_any_disk_byte_changes() {
    let test_name =
        "with_every_vulkan_driver_hidden_a_mount_choosing_gpu_is_refused_before_any_disk_byte_changes";
    if std::env::var_os(VARIABLE_MARKING_THE_RUN_WITH_EVERY_VULKAN_DRIVER_HIDDEN).is_some() {
        the_mount_choosing_gpu_is_refused_with_no_discrete_gpu_visible();
        println!("{LINE_OF_THE_RUN_WITH_EVERY_VULKAN_DRIVER_HIDDEN} test={test_name}");
        return;
    }
    let driver_list_that_does_not_exist = std::env::temp_dir().join(format!(
        "singlefs-real-gpu-test-without-a-vulkan-driver-{}.json",
        std::process::id()
    ));
    assert!(
        !driver_list_that_does_not_exist.exists(),
        "指过去的驱动清单不在：{}",
        driver_list_that_does_not_exist.display()
    );
    let run_with_every_driver_hidden =
        std::process::Command::new(std::env::current_exe().expect("这个测试程序自己的路径"))
            .args([test_name, "--exact", "--nocapture", "--test-threads", "1"])
            .env(
                VARIABLE_MARKING_THE_RUN_WITH_EVERY_VULKAN_DRIVER_HIDDEN,
                "1",
            )
            .env(
                "VK_DRIVER_FILES",
                driver_list_that_does_not_exist.as_os_str(),
            )
            .env(
                "VK_ICD_FILENAMES",
                driver_list_that_does_not_exist.as_os_str(),
            )
            .env_remove("VK_ADD_DRIVER_FILES")
            .process_group(0)
            .output()
            .expect("起得来驱动全藏起来的那一次");
    let standard_output = String::from_utf8_lossy(&run_with_every_driver_hidden.stdout);
    assert!(
        run_with_every_driver_hidden.status.success()
            && standard_output.contains(&format!(
                "{LINE_OF_THE_RUN_WITH_EVERY_VULKAN_DRIVER_HIDDEN} test={test_name}"
            )),
        "驱动全藏起来的那一次退 0、判过：{:?}\n{standard_output}\n{}",
        run_with_every_driver_hidden.status,
        String::from_utf8_lossy(&run_with_every_driver_hidden.stderr)
    );
}

/// 下面三条共用的搭建推出来的东西。
struct GpuSessionAfterARunWhoseEveryWaitExceedsTheLimit {
    /// 另一次挂载里顺序写过第一个文件的那份历史（拷出 `cpu` 那一个池用）。
    history: PoolOnMemoryDevices,
    gpu_pool: PoolOnMemoryDevices,
    gpu_session: MountedSession,
    /// 服务交来的卡（点名：都参与），照交来的次序。
    participating_gpus: Vec<GpuDeviceIdentifier>,
    /// 等的上限切到 1 纳秒之后推的那一趟。
    run_on_the_gpus: CompactionRun,
    /// 推那一趟之前录制流里改盘的动作有几个。
    operations_recorded_before_the_run: usize,
}

/// 同一历史拷一个池选 `gpu`（服务交来的卡、每张 1 GiB、等的上限 60 秒）开卡、重挂、装上整理只供测试的开关（一批至多两个、起跑强制开着），
/// 挂着切到同一组卡与上限、等 GPU 的上限 [`WAIT_LIMIT_EXCEEDED_BY_EVERY_SUBMISSION`]，连同补跑阈值 `threshold`（从下一批起），推一趟整理。
/// 服务没交这一台的卡时打 `NOT_RUN_NO_GPU_ADAPTER` 那一行、交回 `None`。
fn gpu_session_after_a_run_whose_every_wait_exceeds_the_limit(
    test_name: &str,
    threshold: BackgroundGcCpuCompensationThreshold,
) -> Option<GpuSessionAfterARunWhoseEveryWaitExceedsTheLimit> {
    let handed_over_gpus =
        handed_over_gpus_or_report_not_run(test_name, WAIT_LIMIT_OF_THE_GPU_TESTS)?;
    let settings = gpu_settings_on_the_handed_over_gpus(&handed_over_gpus);
    let gpu_compute = opened_gpu_compute(&settings);
    let participating_gpus = identifiers_of_the_handed_over_gpus(&handed_over_gpus);
    let history = pool_after_a_sequential_write_in_an_earlier_mount();
    let mut gpu_pool = history.copied_with_an_empty_stream();
    let mut gpu_session = gpu_pool
        .remounted_with_compaction_forced_to_start(settings.clone(), Some(gpu_compute.clone()));
    gpu_session
        .switch_background_compute_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
            BackgroundComputeSettings {
                background_compute: settings.background_compute,
                background_gpu_devices: settings.background_gpu_devices,
                background_gpu_memory_limit: settings.background_gpu_memory_limit,
                background_gpu_wait_limit: WAIT_LIMIT_EXCEEDED_BY_EVERY_SUBMISSION,
            },
            Some(gpu_compute),
            threshold,
        )
        .expect("同一组卡与上限、等的上限大于 0、阈值不超过 100%：切换收下");
    let operations_recorded_before_the_run = gpu_pool.recorded_operations().len();
    let run_on_the_gpus = gpu_session
        .run_compaction(&mut gpu_pool.devices)
        .expect("整理一趟");
    Some(GpuSessionAfterARunWhoseEveryWaitExceedsTheLimit {
        history,
        gpu_pool,
        gpu_session,
        participating_gpus,
        run_on_the_gpus,
        operations_recorded_before_the_run,
    })
}

/// 那一趟在 GPU 上出错停下：一批放弃、整理停下、一版都没发。
fn is_stopped_after_a_compute_error_in_the_run(run: &CompactionRun) -> bool {
    matches!(
        run.outcome,
        CompactionRunOutcome::Stopped(
            CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
                task_after_the_error: BackgroundTaskAfterTheComputeError::LeftStopped
            }
        )
    ) && run.versions_published.is_empty()
}

/// 七·3 验收「挂着时卡出错就报警、GC 停下」（用户 2026-11-01 定）在真卡上：挂着切到等的上限 1 纳秒之后推一趟——一批在卡上等超过上限、
/// 重交 3 次都超时，这一批放弃、整理停下，一个改盘的动作都没发；这一趟就交回报警：整理、叫起 0 次（出错停下的那一趟不数）、补跑 0 批，
/// 错点名的卡至少一张、都是参与的卡；会话里整理停着。
#[test]
fn the_run_whose_every_wait_on_the_real_gpus_exceeds_the_limit_returns_an_alarm_and_stops_the_compaction(
) {
    let Some(stopped) = gpu_session_after_a_run_whose_every_wait_exceeds_the_limit(
        "the_run_whose_every_wait_on_the_real_gpus_exceeds_the_limit_returns_an_alarm_and_stops_the_compaction",
        BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
    ) else {
        return;
    };
    assert!(
        is_stopped_after_a_compute_error_in_the_run(&stopped.run_on_the_gpus),
        "卡上等超过上限、重交 3 次都超时：这一批放弃、整理停下、一版都没发（{:?}）",
        stopped.run_on_the_gpus.outcome
    );
    assert_eq!(
        stopped.gpu_pool.recorded_operations().len(),
        stopped.operations_recorded_before_the_run,
        "一个改盘的动作都没发"
    );
    let alarm: &AlarmOfABackgroundTaskStoppedAfterAComputeError = stopped
        .run_on_the_gpus
        .alarm_of_the_compaction_stopped_after_a_compute_error
        .as_ref()
        .expect("出错停下的那一趟就报警");
    assert_eq!(
        (
            alarm.task,
            alarm.wakes_while_stopped,
            alarm.batches_compensated_on_the_cpu
        ),
        (BackgroundTask::CompactionRelocation, 0, 0),
        "报的是整理、这一趟不数、还没补跑：{alarm:?}"
    );
    assert!(
        !alarm.gpu_devices_named_by_the_error.is_empty()
            && alarm
                .gpu_devices_named_by_the_error
                .iter()
                .all(|device| stopped.participating_gpus.contains(device)),
        "错点名的是参与的卡（参与的 {:?}）：{alarm:?}",
        stopped.participating_gpus
    );
    assert!(
        stopped
            .gpu_session
            .background_compute()
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation),
        "会话里整理停着"
    );
}

/// 七·3 验收「占用超过阈值时在 CPU 上补跑、结果与 CPU 模式相同」（用户 2026-11-01 定）在真卡上：切换时连同补跑阈值 0%，那一趟在 GPU 上出错停下之后
/// 再推一趟——每一批记成停着时在 CPU 上补跑，这一趟交回报警（叫起 1 次、补跑的批数就是这一趟的批数）；同一历史拷的另一个池选 `cpu` 推一趟：
/// 两趟停在同一条上、发的每一版逐项相同、搬了一样多批，录制流逐步相同、盘上逐字节相同（出错停下那一趟一个改盘的动作都没发）。
#[test]
fn the_compaction_stopped_on_the_real_gpus_compensates_on_the_cpu_above_the_threshold_exactly_as_the_cpu_mode(
) {
    let Some(mut stopped) = gpu_session_after_a_run_whose_every_wait_exceeds_the_limit(
        "the_compaction_stopped_on_the_real_gpus_compensates_on_the_cpu_above_the_threshold_exactly_as_the_cpu_mode",
        THRESHOLD_EXCEEDED_BY_ANY_ALLOCATED_BYTE,
    ) else {
        return;
    };
    assert!(
        is_stopped_after_a_compute_error_in_the_run(&stopped.run_on_the_gpus),
        "补跑之前先在 GPU 上出错停下（{:?}）",
        stopped.run_on_the_gpus.outcome
    );
    let compensating = stopped
        .gpu_session
        .run_compaction(&mut stopped.gpu_pool.devices)
        .expect("整理一趟");
    let mut cpu_pool = stopped.history.copied_with_an_empty_stream();
    let mut cpu_session = cpu_pool
        .remounted_with_compaction_forced_to_start(BackgroundComputeSettings::PRODUCT, None);
    let cpu_run = cpu_session
        .run_compaction(&mut cpu_pool.devices)
        .expect("整理一趟");
    assert!(
        compensating.batches_published > 0
            && compensating
                .background_compute_of_each_batch
                .iter()
                .all(|batch| matches!(
                    batch.path_taken,
                    BackgroundComputePathTaken::Cpu(
                        ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
                    )
                )),
        "停着而超过阈值 0%：搬了批，每一批都记成停着时在 CPU 上补跑（{:?}）",
        compensating.outcome
    );
    assert_eq!(
        compensating
            .alarm_of_the_compaction_stopped_after_a_compute_error
            .as_ref()
            .map(|alarm| (
                alarm.wakes_while_stopped,
                alarm.batches_compensated_on_the_cpu
            )),
        Some((
            1,
            u64::try_from(compensating.background_compute_of_each_batch.len()).expect("批数")
        )),
        "补跑的那一趟照报：停下之后头一次叫起、补跑的批数就是这一趟的批数"
    );
    assert_eq!(
        format!("{:?}", compensating.outcome),
        format!("{:?}", cpu_run.outcome),
        "停在同一条上"
    );
    assert_eq!(
        compensating.versions_published, cpu_run.versions_published,
        "发的每一版逐项相同"
    );
    assert_eq!(
        compensating.batches_published, cpu_run.batches_published,
        "搬了一样多批"
    );
    assert_eq!(
        stopped.gpu_pool.recorded_operations(),
        cpu_pool.recorded_operations(),
        "重挂之后的录制流逐步相同"
    );
    assert_eq!(
        stopped.gpu_pool.written_sectors_of_every_device(),
        cpu_pool.written_sectors_of_every_device(),
        "盘上逐字节相同"
    );
}

/// 七·3 验收「不超过时照旧停着」（用户 2026-11-01 定）在真卡上：补跑阈值取默认 85%（这个池每块盘的占用远低于 1%），那一趟在 GPU 上出错停下之后
/// 再推一趟——照旧停着、一批都没开、一个改盘的动作都没发，这一趟交回报警（叫起 1 次、补跑 0 批）。
#[test]
fn the_compaction_stopped_on_the_real_gpus_stays_stopped_while_not_above_the_threshold() {
    let Some(mut stopped) = gpu_session_after_a_run_whose_every_wait_exceeds_the_limit(
        "the_compaction_stopped_on_the_real_gpus_stays_stopped_while_not_above_the_threshold",
        BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
    ) else {
        return;
    };
    assert!(
        is_stopped_after_a_compute_error_in_the_run(&stopped.run_on_the_gpus),
        "先在 GPU 上出错停下（{:?}）",
        stopped.run_on_the_gpus.outcome
    );
    let operations_recorded_after_the_stop = stopped.gpu_pool.recorded_operations().len();
    let left_stopped = stopped
        .gpu_session
        .run_compaction(&mut stopped.gpu_pool.devices)
        .expect("整理一趟");
    assert!(
        matches!(
            left_stopped.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError
            )
        ) && left_stopped.versions_published.is_empty()
            && left_stopped.background_compute_of_each_batch.is_empty(),
        "不超过阈值：照旧停着、一批都没开（{:?}）",
        left_stopped.outcome
    );
    assert_eq!(
        stopped.gpu_pool.recorded_operations().len(),
        operations_recorded_after_the_stop,
        "一个改盘的动作都没发"
    );
    assert_eq!(
        left_stopped
            .alarm_of_the_compaction_stopped_after_a_compute_error
            .as_ref()
            .map(|alarm| (
                alarm.wakes_while_stopped,
                alarm.batches_compensated_on_the_cpu
            )),
        Some((1, 0)),
        "照旧停着的那一次叫起照报：停下之后头一次、没补跑"
    );
}
