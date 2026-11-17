//! 端到端：产品 GPU 包的实现（`background_gpu_compute::open_background_gpu_compute` 开出来的把手，背后是真卡上三段的两份 WGSL 核）递进核心层的可写挂载、
//! 选 `gpu`，推一趟整理到停；同一历史的另一个池选 `cpu` 推一趟。两趟发的每一版、录制流、盘上字节、停在哪、搬了几批、整理观测与 discard 逐项相同
//! （D26（后台整理与放置回收） 已定项 13「GPU 路径与 CPU 路径是两份实现……两份实现在测试里对拍」）。另钉住这份实现答的卡清单：开包时开着的本机全部独显，
//! 点名交来的卡时参与的就是交来的那几张、次序照点名的次序；交来一组只点其中一张的设置之后参与的卡只剩它、卡清单不变（不重开），一批照常只在它上面算。
//! 池的搭建在 `common_pool_on_memory_devices`（同 验证包 `build_pool` 那一串，盘在内存里）。
//! 开卡与挂载只点名服务经 `GPU_CARDS_BY_PRIORITY` 交给这个进程、属于这一台的那几张卡（`common_discrete_gpus_of_this_host`）：服务没交这一台的卡时每条打一行
//! `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>` 报「本次未跑」、返回成功；交了卡却对不上本机的独显、或开不了判红。源头没声明张数，服务分一张。
//! 只在开了特性 `tests-on-a-real-gpu` 时编；用 GPU，经 `task-server/task submit` 交（卡与清场由服务定），带 `--test-threads 1`。
//! 开卡与挂载给每张参与的卡的显存上限 1 GiB（`common_discrete_gpus_of_this_host`）。
//! 七·3 验收的另几格也在这里：没有卡时拒绝挂载（本进程照原样再起一次自己、把 Vulkan 驱动全藏起来，那一次里一张独显都看不到）；
//! 挂着时卡出错就报警、GC 停下（挂着切到等 GPU 的上限 1 纳秒：一批交出去就等超过上限、重交 3 次都超时，第 4 条「含重交 3 次都超时」那一种 GPU 出错）；
//! 停下之后占用超过阈值在 CPU 上补跑、结果与 CPU 模式相同，不超过时照旧停着。
//! 要卡的每一条分两段、经判定库交接（`common_history_pool_through_the_library`、`common_pairing_input_through_the_library`）：从那份历史起步的几条把历史先过库，
//! 交一批校验和的那一条把那一批单元先过库，查卡清单的那一条把那张不在本机的卡的名字连同 CPU 上写下的拒法先过库；只录那一趟到此返回、不开卡。
//! 判段的对拍结论按键入库复用（读回输入的摘要、参照一侧与这条用例 GPU 一侧的判法单元键），库里有两边一致的不开卡；要现判的从库里读回再开卡。
//! 每条用例的持卡判入口是 `judged_on_the_cards_that_<用例名>`（登记在 `src/gpu_side_units_entries.rs`）。
//! Vulkan 驱动全藏起来的那一条不要卡，不分段。
mod common_discrete_gpus_of_this_host;
mod common_history_pool_through_the_library;
mod common_pairing_input_through_the_library;
mod common_pool_on_memory_devices;

use std::os::unix::process::CommandExt;
use std::time::Duration;

use common_discrete_gpus_of_this_host::{
    discrete_gpus_on_this_host, handed_over_gpus_or_report_not_run,
    identifiers_of_the_handed_over_gpus, memory_limits_of_one_gibibyte_each, HandedOverGpu,
    HandedOverGpusSeenThroughTheInstance,
};
use common_history_pool_through_the_library::{
    assert_the_pieces_equal_the_reference,
    history_pool_concluded_on_the_cards_through_the_pairing_library,
    history_pool_of_the_input_read_back, reference_bytes_of_the_pieces,
};
use common_pairing_input_through_the_library::{
    append_count, append_length_prefixed_bytes, append_number,
    judgement_of_the_assertions_on_the_cards,
    pairing_concluded_on_the_cards_through_the_pairing_library,
    pairing_input_bytes_of_the_written_sectors, written_sectors_of_the_pairing_input_bytes,
    ComparisonOnTheCards, PairingInputBeingRead,
};
use common_pool_on_memory_devices::{
    parameters, pool_after_a_sequential_write_in_an_earlier_mount, PoolOnMemoryDevices,
};
use singlefs_background_gpu::adapter_inventory::BackgroundGpuSelection;
use singlefs_background_gpu::background_gpu::{
    open_background_gpu, BackgroundGpu, BackgroundGpuOpenRefused, OpeningsNotReturnedInThisProcess,
    VulkanInstanceOfThisProcess,
};
use singlefs_background_gpu::background_gpu_compute::open_background_gpu_compute;
use singlefs_background_gpu::checksum_kernels::{
    cpu_reference_outcome, ChecksumKernelOutcome, UnitChecksumToVerify,
};
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
use singlefs_verification::crash_library::PairingJudged;

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

/// 选 `gpu`、不点名（`vulkan_instance` 上看得到的本机全部独显）、每张卡的显存上限 1 GiB：只给 Vulkan 驱动全藏起来的那一次用
/// （一张独显都看不到，上限列表是空的，开卡照「GPU 不在」拒）。
fn gpu_settings_on_every_discrete_gpu(
    vulkan_instance: &VulkanInstanceOfThisProcess,
) -> BackgroundComputeSettings {
    BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: None,
        background_gpu_memory_limit: Some(memory_limits_of_one_gibibyte_each(
            discrete_gpus_on_this_host(vulkan_instance, WAIT_LIMIT_OF_THE_GPU_TESTS).len(),
        )),
        background_gpu_wait_limit: WAIT_LIMIT_OF_THE_GPU_TESTS,
    }
}

/// 照挂载那一组设置（点名交来的卡）在 `vulkan_instance` 上开卡、交回把手。卡是服务交来、对上了本机适配器的，开不了判红。
fn opened_gpu_compute(
    vulkan_instance: &VulkanInstanceOfThisProcess,
    settings: &BackgroundComputeSettings,
) -> BackgroundGpuComputeHandle {
    match open_background_gpu_compute(
        &BackgroundGpuSelection::of_the_settings(settings),
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        &OpeningsNotReturnedInThisProcess::with_none_not_returned(),
        vulkan_instance,
    ) {
        Ok(opened) => opened.gpu_compute,
        Err(refused) => {
            panic!("服务交来、对上了本机适配器的卡开不了后台 GPU：{refused:?}")
        }
    }
}

/// 照挂载那一组设置（点名交来的卡）在 `vulkan_instance` 上开产品 GPU 包的后台 GPU。卡是服务交来、对上了本机适配器的，开不了判红。
fn opened_background_gpu_of_the_settings(
    vulkan_instance: &VulkanInstanceOfThisProcess,
    settings: &BackgroundComputeSettings,
) -> BackgroundGpu {
    open_background_gpu(
        &BackgroundGpuSelection::of_the_settings(settings),
        vulkan_instance,
    )
    .unwrap_or_else(|refused| panic!("服务交来、对上了本机适配器的卡开不了后台 GPU：{refused:?}"))
}

/// 参照单元登记表里选 `cpu` 那一趟整理那一组的名字（`src/reference_units_entries.rs`）。
const REFERENCE_UNITS_GROUP_OF_THE_COMPACTION_RUN: &str = "cpu_path_of_the_compaction_run";
/// 参照单元登记表里点名不在本机的卡那一条在 CPU 上写下拒法那一组的名字（`src/reference_units_entries.rs`）。
const REFERENCE_UNITS_GROUP_OF_THE_CARD_NOT_ON_THIS_HOST: &str =
    "cpu_refusal_of_the_card_not_on_this_host";

const PIECE_OUTCOME: &str = "停在哪一条上";
const PIECE_VERSIONS_PUBLISHED: &str = "发的每一版";
const PIECE_BATCHES_PUBLISHED: &str = "搬了几批";
const PIECE_DISCARDS: &str = "发布之后的 discard";
const PIECE_OBSERVATIONS: &str = "整理观测";
const PIECE_RECORDED_OPERATIONS: &str = "重挂之后的录制流";
const PIECE_WRITTEN_SECTORS: &str = "盘上的扇区";

/// 一趟整理留下的、两边逐项比的那几样，各编成字节：前六样是 `{:?}` 的文字，末一样是每块盘写过的扇区。
fn pieces_of_the_compaction_run(
    run: &CompactionRun,
    session: &MountedSession,
    pool: &PoolOnMemoryDevices,
) -> Vec<(&'static str, Vec<u8>)> {
    vec![
        (PIECE_OUTCOME, format!("{:?}", run.outcome).into_bytes()),
        (
            PIECE_VERSIONS_PUBLISHED,
            format!("{:?}", run.versions_published).into_bytes(),
        ),
        (
            PIECE_BATCHES_PUBLISHED,
            format!("{:?}", run.batches_published).into_bytes(),
        ),
        (PIECE_DISCARDS, format!("{:?}", run.discards).into_bytes()),
        (
            PIECE_OBSERVATIONS,
            format!("{:?}", session.compaction.observations).into_bytes(),
        ),
        (
            PIECE_RECORDED_OPERATIONS,
            format!("{:?}", pool.recorded_operations()).into_bytes(),
        ),
        (
            PIECE_WRITTEN_SECTORS,
            pairing_input_bytes_of_the_written_sectors(&pool.written_sectors_of_every_device()),
        ),
    ]
}

/// 整理对拍参照一侧的入口（登记在 `src/reference_units_entries.rs`）：照对拍输入里的扇区搭回那份历史，选 `cpu` 重挂、推一趟整理到停，
/// 留下的几样编成字节。只录那一段调它。
fn reference_bytes_of_the_compaction_run_on_the_cpu(input_bytes: &[u8]) -> Vec<u8> {
    let mut cpu_pool = PoolOnMemoryDevices::of_the_written_sectors(
        written_sectors_of_the_pairing_input_bytes(input_bytes),
    );
    let mut cpu_session = cpu_pool
        .remounted_with_compaction_forced_to_start(BackgroundComputeSettings::PRODUCT, None);
    let cpu_run = cpu_session
        .run_compaction(&mut cpu_pool.devices)
        .expect("整理一趟");
    reference_bytes_of_the_pieces(&pieces_of_the_compaction_run(
        &cpu_run,
        &cpu_session,
        &cpu_pool,
    ))
}

/// 交一批校验和那一条参照一侧的入口（登记在 `src/reference_units_entries.rs`）：对拍输入里每个单元照期望值 0 的 CPU 参照结局，编成字节
/// （条数，逐条是算出的 CRC 与对不对得上，都是 8 字节小端）。只录那一段调它。
fn reference_bytes_of_the_units_narrowed_to_one_card(input_bytes: &[u8]) -> Vec<u8> {
    let mut input = PairingInputBeingRead::from_the_start_of(input_bytes);
    let unit_count = input.take_count();
    let mut reference_bytes = Vec::new();
    append_count(&mut reference_bytes, unit_count);
    for _unit in 0..unit_count {
        let outcome = cpu_reference_outcome(input.take_length_prefixed_bytes(), 0);
        append_number(&mut reference_bytes, u64::from(outcome.computed_crc32c));
        append_number(
            &mut reference_bytes,
            u64::from(outcome.matches_the_expected_crc32c),
        );
    }
    input.finish();
    reference_bytes
}

#[test]
fn the_compaction_run_on_the_real_gpus_ends_exactly_as_the_cpu_path() {
    history_pool_concluded_on_the_cards_through_the_pairing_library(
        "the_compaction_run_on_the_real_gpus_ends_exactly_as_the_cpu_path",
        REFERENCE_UNITS_GROUP_OF_THE_COMPACTION_RUN,
        reference_bytes_of_the_compaction_run_on_the_cpu,
        judged_on_the_cards_that_the_compaction_run_on_the_real_gpus_ends_exactly_as_the_cpu_path,
    );
}

/// 持卡判入口（登记在 `src/gpu_side_units_entries.rs`）：照读回的扇区搭池、选 `gpu` 点名交来的卡推一趟整理到停，
/// 留下的几样逐样与库里读回的选 `cpu` 那一趟比，每一批走 GPU、全部确认。
fn judged_on_the_cards_that_the_compaction_run_on_the_real_gpus_ends_exactly_as_the_cpu_path(
    test_name: &str,
    input_bytes: &[u8],
    reference_bytes: &[u8],
) -> PairingJudged {
    judgement_of_the_assertions_on_the_cards(test_name, || {
        let Some(HandedOverGpusSeenThroughTheInstance {
            vulkan_instance,
            gpus: handed_over_gpus,
        }) = handed_over_gpus_or_report_not_run(test_name, WAIT_LIMIT_OF_THE_GPU_TESTS)
        else {
            return ComparisonOnTheCards::NotRunOnTheseCards;
        };
        let settings = gpu_settings_on_the_handed_over_gpus(&handed_over_gpus);
        let gpu_compute = opened_gpu_compute(&vulkan_instance, &settings);
        let mut gpu_pool = history_pool_of_the_input_read_back(input_bytes);
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
        // 选 cpu 的那一趟在只录那一段跑过、留下的几样在库里：停在哪、发的每一版、搬了几批、discard、整理观测、录制流、盘上的扇区逐样相同
        assert_the_pieces_equal_the_reference(
            &pieces_of_the_compaction_run(&gpu_run, &gpu_session, &gpu_pool),
            reference_bytes,
            &[
                PIECE_OUTCOME,
                PIECE_VERSIONS_PUBLISHED,
                PIECE_BATCHES_PUBLISHED,
                PIECE_DISCARDS,
                PIECE_OBSERVATIONS,
                PIECE_RECORDED_OPERATIONS,
                PIECE_WRITTEN_SECTORS,
            ],
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
        ComparisonOnTheCards::EveryComparisonHeld
    })
}

/// 这份实现答的卡清单是本机全部独显与各自的显存总量（核心层挂载照它校验）；点名交来的卡时参与的就是交来的那几张、次序照点名的次序，
/// 每张都在卡清单里（卡清单另含本机没点名的独显）。点名参与的那张卡照收，点名不在本机的卡照「点名的卡不存在」拒。
#[test]
fn the_implementation_answers_the_participating_gpus_as_the_inventory_the_core_admits_against() {
    // 喂给核心层的输入只有一样：那张不在本机的卡的名字；参照一侧在 CPU 上照它写下核心层该怎么拒。两样过判定库，只录那一趟到此返回、不开卡。
    pairing_concluded_on_the_cards_through_the_pairing_library(
        "the_implementation_answers_the_participating_gpus_as_the_inventory_the_core_admits_against",
        REFERENCE_UNITS_GROUP_OF_THE_CARD_NOT_ON_THIS_HOST,
        || b"ffff:ff:ff.f".to_vec(),
        reference_bytes_of_the_refusal_of_the_card_not_on_this_host,
        judged_on_the_cards_that_the_implementation_answers_the_participating_gpus_as_the_inventory_the_core_admits_against,
    );
}

/// 点名不在本机的卡那一条参照一侧的入口（登记在 `src/reference_units_entries.rs`）：对拍输入是那张卡的名字，在 CPU 上写下核心层点名它时该交回的拒法
/// （`{:?}` 的文字）。只录那一段调它。
fn reference_bytes_of_the_refusal_of_the_card_not_on_this_host(input_bytes: &[u8]) -> Vec<u8> {
    let refusal: Result<(), BackgroundComputeSettingsRefused> = Err(
        BackgroundComputeSettingsRefused::NamedGpuDeviceNotInTheInventory {
            device: GpuDeviceIdentifier::of_opaque_bytes(input_bytes.to_vec()),
        },
    );
    format!("{refusal:?}").into_bytes()
}

/// 持卡判入口（登记在 `src/gpu_side_units_entries.rs`）：点名交来的卡开包，参与的就是交来的那几张、都在卡清单里；点名参与的那张照收，
/// 点名读回的那张不在本机的卡，核心层交回的拒法与读回的参照（CPU 上写下的那一句）相同。
fn judged_on_the_cards_that_the_implementation_answers_the_participating_gpus_as_the_inventory_the_core_admits_against(
    test_name: &str,
    input_bytes: &[u8],
    reference_bytes: &[u8],
) -> PairingJudged {
    judgement_of_the_assertions_on_the_cards(test_name, || {
        let Some(HandedOverGpusSeenThroughTheInstance {
            vulkan_instance,
            gpus: handed_over_gpus,
        }) = handed_over_gpus_or_report_not_run(test_name, WAIT_LIMIT_OF_THE_GPU_TESTS)
        else {
            return ComparisonOnTheCards::NotRunOnTheseCards;
        };
        let settings = gpu_settings_on_the_handed_over_gpus(&handed_over_gpus);
        let background_gpu = opened_background_gpu_of_the_settings(&vulkan_instance, &settings);
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
        let gpu_compute = opened_gpu_compute(&vulkan_instance, &settings);
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
        let refusal_of_the_card_not_on_this_host =
            admit_background_compute_settings_with_the_gpu_compute(
                naming(GpuDeviceIdentifier::of_opaque_bytes(input_bytes.to_vec())),
                Some(gpu_compute),
            )
            .map(|_admitted| ());
        assert_eq!(
            format!("{refusal_of_the_card_not_on_this_host:?}"),
            String::from_utf8_lossy(reference_bytes),
            "点名不在本机的卡：拒，拒法与 CPU 上写下的那一句相同"
        );
        ComparisonOnTheCards::EveryComparisonHeld
    })
}

/// 第 2、3 条与第 1 条「切换不重开」：开包时点名服务交来的卡，它们都参与；交来一组只点交来的头一张、上限 1 GiB 的设置之后，
/// 参与的卡只剩那一张、上限照给的，卡清单照旧是开包时那一份（本机全部独显）；之后一批校验和核对照常算完、与 CPU 参照逐条相同。
#[test]
fn settings_naming_one_card_narrow_the_participating_gpus_without_reopening_the_cards() {
    pairing_concluded_on_the_cards_through_the_pairing_library(
        "settings_naming_one_card_narrow_the_participating_gpus_without_reopening_the_cards",
        REFERENCE_UNITS_GROUP_OF_THE_COMPACTION_RUN,
        || {
            let mut input_bytes = Vec::new();
            append_count(&mut input_bytes, 8);
            for fill in 0_u8..8 {
                append_length_prefixed_bytes(&mut input_bytes, &vec![fill; 4096]);
            }
            input_bytes
        },
        reference_bytes_of_the_units_narrowed_to_one_card,
        judged_on_the_cards_that_settings_naming_one_card_narrow_the_participating_gpus_without_reopening_the_cards,
    );
}

/// 持卡判入口（登记在 `src/gpu_side_units_entries.rs`）：点名交来的卡开包，换成只点头一张的设置之后参与的只剩它、卡清单不变，
/// 读回的那一批照期望值 0 交、两份核逐条与读回的 CPU 参照比。
fn judged_on_the_cards_that_settings_naming_one_card_narrow_the_participating_gpus_without_reopening_the_cards(
    test_name: &str,
    input_bytes: &[u8],
    reference_bytes: &[u8],
) -> PairingJudged {
    judgement_of_the_assertions_on_the_cards(test_name, || {
        let unit_bytes: Vec<Vec<u8>> = {
            let mut input = PairingInputBeingRead::from_the_start_of(input_bytes);
            let unit_count = input.take_count();
            let unit_bytes = (0..unit_count)
                .map(|_unit| input.take_length_prefixed_bytes().to_vec())
                .collect();
            input.finish();
            unit_bytes
        };
        let Some(HandedOverGpusSeenThroughTheInstance {
            vulkan_instance,
            gpus: handed_over_gpus,
        }) = handed_over_gpus_or_report_not_run(test_name, WAIT_LIMIT_OF_THE_GPU_TESTS)
        else {
            return ComparisonOnTheCards::NotRunOnTheseCards;
        };
        let settings = gpu_settings_on_the_handed_over_gpus(&handed_over_gpus);
        let mut background_gpu = opened_background_gpu_of_the_settings(&vulkan_instance, &settings);
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
        let expected: Vec<_> = {
            let mut reference = PairingInputBeingRead::from_the_start_of(reference_bytes);
            let outcome_count = reference.take_count();
            let expected = (0..outcome_count)
                .map(|_outcome| {
                    let outcome = ChecksumKernelOutcome {
                        computed_crc32c: u32::try_from(reference.take_number())
                            .expect("写下的 CRC 装得进 u32"),
                        matches_the_expected_crc32c: reference.take_number() == 1,
                    };
                    (outcome, outcome)
                })
                .collect();
            reference.finish();
            expected
        };
        assert_eq!(outcomes, expected, "两份核都与 CPU 参照逐条相同");
        ComparisonOnTheCards::EveryComparisonHeld
    })
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
    let vulkan_instance = VulkanInstanceOfThisProcess::created_for_this_process();
    let discrete_gpus = discrete_gpus_on_this_host(&vulkan_instance, WAIT_LIMIT_OF_THE_GPU_TESTS);
    assert!(
        discrete_gpus.is_empty(),
        "Vulkan 驱动全藏起来之后一张独显都看不到：{discrete_gpus:?}"
    );
    let settings = gpu_settings_on_every_discrete_gpu(&vulkan_instance);
    assert!(
        matches!(
            open_background_gpu_compute(
                &BackgroundGpuSelection::of_the_settings(&settings),
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
                &OpeningsNotReturnedInThisProcess::with_none_not_returned(),
                &vulkan_instance,
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

/// 七·3 验收「没有卡时拒绝挂载」：本进程照原样再起一次自己、只跑这一条，那一次里把 Vulkan 驱动全藏起来
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
    gpu_pool: PoolOnMemoryDevices,
    gpu_session: MountedSession,
    /// 服务交来的卡（点名：都参与），照交来的次序。
    participating_gpus: Vec<GpuDeviceIdentifier>,
    /// 等的上限切到 1 纳秒之后推的那一趟。
    run_on_the_gpus: CompactionRun,
    /// 推那一趟之前录制流里改盘的动作有几个。
    operations_recorded_before_the_run: usize,
}

/// 照读回的扇区（`input_bytes`）搭回那份历史、拷一个池选 `gpu`（服务交来的卡、每张 1 GiB、等的上限 60 秒）开卡、重挂、装上整理只供测试的开关
/// （一批至多两个、起跑强制开着），挂着切到同一组卡与上限、等 GPU 的上限 [`WAIT_LIMIT_EXCEEDED_BY_EVERY_SUBMISSION`]，连同补跑阈值 `threshold`
/// （从下一批起），推一趟整理。下面三条的持卡判入口调它。服务没交这一台的卡时打 `NOT_RUN_NO_GPU_ADAPTER` 那一行、交回 `None`。
fn gpu_session_after_a_run_whose_every_wait_exceeds_the_limit(
    test_name: &str,
    threshold: BackgroundGcCpuCompensationThreshold,
    input_bytes: &[u8],
) -> Option<GpuSessionAfterARunWhoseEveryWaitExceedsTheLimit> {
    let history = history_pool_of_the_input_read_back(input_bytes);
    let HandedOverGpusSeenThroughTheInstance {
        vulkan_instance,
        gpus: handed_over_gpus,
    } = handed_over_gpus_or_report_not_run(test_name, WAIT_LIMIT_OF_THE_GPU_TESTS)?;
    let settings = gpu_settings_on_the_handed_over_gpus(&handed_over_gpus);
    let gpu_compute = opened_gpu_compute(&vulkan_instance, &settings);
    let participating_gpus = identifiers_of_the_handed_over_gpus(&handed_over_gpus);
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

/// 七·3 验收「挂着时卡出错就报警、GC 停下」在真卡上：挂着切到等的上限 1 纳秒之后推一趟——一批在卡上等超过上限、
/// 重交 3 次都超时，这一批放弃、整理停下，一个改盘的动作都没发；这一趟就交回报警：整理、叫起 0 次（出错停下的那一趟不数）、补跑 0 批，
/// 错点名的卡至少一张、都是参与的卡；会话里整理停着。
#[test]
fn the_run_whose_every_wait_on_the_real_gpus_exceeds_the_limit_returns_an_alarm_and_stops_the_compaction(
) {
    history_pool_concluded_on_the_cards_through_the_pairing_library(
        "the_run_whose_every_wait_on_the_real_gpus_exceeds_the_limit_returns_an_alarm_and_stops_the_compaction",
        REFERENCE_UNITS_GROUP_OF_THE_COMPACTION_RUN,
        reference_bytes_of_the_compaction_run_on_the_cpu,
        judged_on_the_cards_that_the_run_whose_every_wait_on_the_real_gpus_exceeds_the_limit_returns_an_alarm_and_stops_the_compaction,
    );
}

/// 持卡判入口（登记在 `src/gpu_side_units_entries.rs`）：照读回的历史搭池、挂着切到等的上限 1 纳秒推一趟：这一批放弃、整理停下、报警。
fn judged_on_the_cards_that_the_run_whose_every_wait_on_the_real_gpus_exceeds_the_limit_returns_an_alarm_and_stops_the_compaction(
    test_name: &str,
    input_bytes: &[u8],
    _reference_bytes_not_compared_by_this_test: &[u8],
) -> PairingJudged {
    judgement_of_the_assertions_on_the_cards(test_name, || {
        let Some(stopped) = gpu_session_after_a_run_whose_every_wait_exceeds_the_limit(
            test_name,
            BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
            input_bytes,
        ) else {
            return ComparisonOnTheCards::NotRunOnTheseCards;
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
        ComparisonOnTheCards::EveryComparisonHeld
    })
}

/// 七·3 验收「占用超过阈值时在 CPU 上补跑、结果与 CPU 模式相同」在真卡上：切换时连同补跑阈值 0%，那一趟在 GPU 上出错停下之后
/// 再推一趟——每一批记成停着时在 CPU 上补跑，这一趟交回报警（叫起 1 次、补跑的批数就是这一趟的批数）；同一历史拷的另一个池选 `cpu` 推一趟：
/// 两趟停在同一条上、发的每一版逐项相同、搬了一样多批，录制流逐步相同、盘上逐字节相同（出错停下那一趟一个改盘的动作都没发）。
#[test]
fn the_compaction_stopped_on_the_real_gpus_compensates_on_the_cpu_above_the_threshold_exactly_as_the_cpu_mode(
) {
    history_pool_concluded_on_the_cards_through_the_pairing_library(
        "the_compaction_stopped_on_the_real_gpus_compensates_on_the_cpu_above_the_threshold_exactly_as_the_cpu_mode",
        REFERENCE_UNITS_GROUP_OF_THE_COMPACTION_RUN,
        reference_bytes_of_the_compaction_run_on_the_cpu,
        judged_on_the_cards_that_the_compaction_stopped_on_the_real_gpus_compensates_on_the_cpu_above_the_threshold_exactly_as_the_cpu_mode,
    );
}

/// 持卡判入口（登记在 `src/gpu_side_units_entries.rs`）：照读回的历史搭池、出错停下之后阈值 0% 再推一趟：每一批在 CPU 上补跑、报警，留下的几样逐样与库里读回的选 `cpu` 那一趟比。
fn judged_on_the_cards_that_the_compaction_stopped_on_the_real_gpus_compensates_on_the_cpu_above_the_threshold_exactly_as_the_cpu_mode(
    test_name: &str,
    input_bytes: &[u8],
    reference_bytes: &[u8],
) -> PairingJudged {
    judgement_of_the_assertions_on_the_cards(test_name, || {
        let Some(mut stopped) = gpu_session_after_a_run_whose_every_wait_exceeds_the_limit(
            test_name,
            THRESHOLD_EXCEEDED_BY_ANY_ALLOCATED_BYTE,
            input_bytes,
        ) else {
            return ComparisonOnTheCards::NotRunOnTheseCards;
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
        assert!(
            compensating.batches_published > 0
                && compensating
                    .background_compute_of_each_batch
                    .iter()
                    .all(|batch| matches!(
                        batch.path_taken,
                        BackgroundComputePathTaken::Cpu(
                            ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(
                                _
                            )
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
        // 选 cpu 的那一趟在只录那一段跑过、留下的几样在库里：停在哪、发的每一版、搬了几批、录制流、盘上的扇区逐样相同
        assert_the_pieces_equal_the_reference(
            &pieces_of_the_compaction_run(&compensating, &stopped.gpu_session, &stopped.gpu_pool),
            reference_bytes,
            &[
                PIECE_OUTCOME,
                PIECE_VERSIONS_PUBLISHED,
                PIECE_BATCHES_PUBLISHED,
                PIECE_RECORDED_OPERATIONS,
                PIECE_WRITTEN_SECTORS,
            ],
        );
        ComparisonOnTheCards::EveryComparisonHeld
    })
}

/// 七·3 验收「不超过时照旧停着」在真卡上：补跑阈值取默认 85%（这个池每块盘的占用远低于 1%），那一趟在 GPU 上出错停下之后
/// 再推一趟——照旧停着、一批都没开、一个改盘的动作都没发，这一趟交回报警（叫起 1 次、补跑 0 批）。
#[test]
fn the_compaction_stopped_on_the_real_gpus_stays_stopped_while_not_above_the_threshold() {
    history_pool_concluded_on_the_cards_through_the_pairing_library(
        "the_compaction_stopped_on_the_real_gpus_stays_stopped_while_not_above_the_threshold",
        REFERENCE_UNITS_GROUP_OF_THE_COMPACTION_RUN,
        reference_bytes_of_the_compaction_run_on_the_cpu,
        judged_on_the_cards_that_the_compaction_stopped_on_the_real_gpus_stays_stopped_while_not_above_the_threshold,
    );
}

/// 持卡判入口（登记在 `src/gpu_side_units_entries.rs`）：照读回的历史搭池、出错停下之后阈值取默认再推一趟：照旧停着、报警。
fn judged_on_the_cards_that_the_compaction_stopped_on_the_real_gpus_stays_stopped_while_not_above_the_threshold(
    test_name: &str,
    input_bytes: &[u8],
    _reference_bytes_not_compared_by_this_test: &[u8],
) -> PairingJudged {
    judgement_of_the_assertions_on_the_cards(test_name, || {
        let Some(mut stopped) = gpu_session_after_a_run_whose_every_wait_exceeds_the_limit(
            test_name,
            BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
            input_bytes,
        ) else {
            return ComparisonOnTheCards::NotRunOnTheseCards;
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
        ComparisonOnTheCards::EveryComparisonHeld
    })
}
