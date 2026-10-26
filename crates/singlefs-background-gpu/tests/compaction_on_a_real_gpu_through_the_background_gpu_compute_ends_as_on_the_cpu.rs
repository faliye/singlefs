//! 端到端：产品 GPU 包的实现（`background_gpu_compute::open_background_gpu_compute` 开出来的把手，背后是真卡上三段的两份 WGSL 核）递进核心层的可写挂载、
//! 选 `gpu`，推一趟整理到停；同一历史的另一个池选 `cpu` 推一趟。两趟发的每一版、录制流、盘上字节、停在哪、搬了几批、整理观测与 discard 逐项相同
//! （D26（后台整理与放置回收） 已定项 13「GPU 路径与 CPU 路径是两份实现……两份实现在测试里对拍」）。另钉住这份实现答的卡清单：开包时开着的本机全部独显，
//! 没点名时就是参与的卡；交来一组只点其中一张的设置之后参与的卡只剩它、卡清单不变（不重开），一批照常只在它上面算。
//! 池的搭建在 `common_pool_on_memory_devices`（同 harness 档 `build_pool` 那一串，盘在内存里）。
//! 要本机至少一张独立显卡：一张都没有时每条打一行 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>` 报「本次未跑」、返回成功；
//! 有卡而开不了判红。只在开了特性 `tests-on-a-real-gpu` 时编；用 GPU，经 `scripts/crash/gpu-quiesce-run.sh` 起，带 `--test-threads 1`。
mod common_pool_on_memory_devices;

use std::time::Duration;

use common_pool_on_memory_devices::pool_after_a_sequential_write_in_an_earlier_mount;
use singlefs_background_gpu::adapter_inventory::BackgroundGpuSelection;
use singlefs_background_gpu::background_gpu::{open_background_gpu, BackgroundGpuOpenRefused};
use singlefs_background_gpu::background_gpu_compute::open_background_gpu_compute;
use singlefs_background_gpu::checksum_kernels::{cpu_reference_outcome, UnitChecksumToVerify};
use singlefs_core::background_compute::{
    admit_background_compute_settings_with_the_gpu_compute,
    admit_gpu_settings_against_the_inventory, BackgroundComputeBackend, BackgroundComputePathTaken,
    BackgroundComputeSettings, BackgroundComputeSettingsRefused, BackgroundGpuComputeHandle,
    GpuDeviceIdentifier, GpuInventory, GpuMemoryInBytes,
};

/// 等 GPU 的上限：这几批在健康的卡上远用不了这么久。
const WAIT_LIMIT_OF_THE_GPU_TESTS: Duration = Duration::from_secs(60);

/// 选 `gpu`、用本机全部独显、不设显存上限。
fn gpu_settings_on_every_discrete_gpu() -> BackgroundComputeSettings {
    BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: None,
        background_gpu_memory_limit: None,
        background_gpu_wait_limit: WAIT_LIMIT_OF_THE_GPU_TESTS,
    }
}

fn report_not_run(test_name: &str) {
    println!(
        "NOT_RUN_NO_GPU_ADAPTER test={test_name} reason=本机的 Vulkan 适配器里一张独立显卡都没有"
    );
}

/// 照挂载那一组设置开卡、交回把手；一张独显都没有时打 `NOT_RUN_NO_GPU_ADAPTER` 那一行、交回 `None`。
fn gpu_compute_or_report_not_run(
    test_name: &str,
    settings: &BackgroundComputeSettings,
) -> Option<BackgroundGpuComputeHandle> {
    match open_background_gpu_compute(&BackgroundGpuSelection::of_the_settings(settings)) {
        Ok(opened) => Some(opened.gpu_compute),
        Err(BackgroundGpuOpenRefused::SettingsRefused(
            BackgroundComputeSettingsRefused::GpuNotPresent,
        )) => {
            report_not_run(test_name);
            None
        }
        Err(refused) => panic!("本机有独立显卡而开不了后台 GPU：{refused:?}"),
    }
}

#[test]
fn a_compaction_run_on_the_real_gpus_ends_exactly_as_the_cpu_path() {
    let settings = gpu_settings_on_every_discrete_gpu();
    let Some(gpu_compute) = gpu_compute_or_report_not_run(
        "a_compaction_run_on_the_real_gpus_ends_exactly_as_the_cpu_path",
        &settings,
    ) else {
        return;
    };
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
        let BackgroundComputePathTaken::Gpu(gpu_work) = batch.path_taken else {
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

/// 这份实现答的卡清单是本机全部独显与各自的显存总量（核心层挂载照它校验）；没点名时本机全部独显都参与（一张开不了就拒了挂载），它就是参与的卡、次序相同。
/// 点名参与的那张卡照收，点名不在本机的卡照「点名的卡不存在」拒。
#[test]
fn the_implementation_answers_the_participating_gpus_as_the_inventory_the_core_admits_against() {
    let test_name =
        "the_implementation_answers_the_participating_gpus_as_the_inventory_the_core_admits_against";
    let settings = gpu_settings_on_every_discrete_gpu();
    let background_gpu =
        match open_background_gpu(&BackgroundGpuSelection::of_the_settings(&settings)) {
            Ok(background_gpu) => background_gpu,
            Err(BackgroundGpuOpenRefused::SettingsRefused(
                BackgroundComputeSettingsRefused::GpuNotPresent,
            )) => {
                report_not_run(test_name);
                return;
            }
            Err(refused) => panic!("本机有独立显卡而开不了后台 GPU：{refused:?}"),
        };
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
    assert_eq!(inventory, participating, "卡清单就是参与的卡，次序相同");
    drop(background_gpu);
    let Some(gpu_compute) = gpu_compute_or_report_not_run(test_name, &settings) else {
        return;
    };
    let naming = |device: GpuDeviceIdentifier| BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices: Some(vec![device]),
        background_gpu_memory_limit: None,
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

/// 第 2、3 条与第 1 条「切换不重开」：开包时没点名，本机全部独显都参与；交来一组只点卡清单头一张、上限取它显存总量一半的设置之后，
/// 参与的卡只剩那一张、上限照给的，卡清单照旧是本机全部独显；之后一批校验和核对照常算完、与 CPU 参照逐条相同。
#[test]
fn settings_naming_one_card_narrow_the_participating_gpus_without_reopening_the_cards() {
    let test_name =
        "settings_naming_one_card_narrow_the_participating_gpus_without_reopening_the_cards";
    let settings = gpu_settings_on_every_discrete_gpu();
    let mut background_gpu =
        match open_background_gpu(&BackgroundGpuSelection::of_the_settings(&settings)) {
            Ok(background_gpu) => background_gpu,
            Err(BackgroundGpuOpenRefused::SettingsRefused(
                BackgroundComputeSettingsRefused::GpuNotPresent,
            )) => {
                report_not_run(test_name);
                return;
            }
            Err(refused) => panic!("本机有独立显卡而开不了后台 GPU：{refused:?}"),
        };
    let inventory_at_the_opening = background_gpu.devices_with_total_memory();
    let (named_card, total_memory) = inventory_at_the_opening[0].clone();
    let half_of_its_memory = GpuMemoryInBytes(total_memory.0 / 2);
    let narrowed = admit_gpu_settings_against_the_inventory(
        &BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: Some(vec![named_card.clone()]),
            background_gpu_memory_limit: Some(vec![half_of_its_memory]),
            background_gpu_wait_limit: WAIT_LIMIT_OF_THE_GPU_TESTS,
        },
        &background_gpu,
    )
    .expect("点名卡清单里的一张、上限是它显存总量的一半");
    background_gpu.use_the_settings_from_the_next_batch(&narrowed);
    let participating: Vec<(GpuDeviceIdentifier, Option<GpuMemoryInBytes>)> = background_gpu
        .participating_gpus()
        .into_iter()
        .map(|report| (report.identifier, report.memory_limit))
        .collect();
    assert_eq!(
        participating,
        vec![(named_card, Some(half_of_its_memory))],
        "参与的卡只剩点名的那一张，上限照给的"
    );
    assert_eq!(
        background_gpu.devices_with_total_memory(),
        inventory_at_the_opening,
        "卡清单照旧是本机全部独显：不重开"
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
