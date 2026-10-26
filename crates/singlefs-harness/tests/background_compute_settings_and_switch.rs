//! 后台任务的计算放在哪（D26（后台整理与放置回收） 已定项 13 第 1–4 条）：一组挂载参数（`background_compute`、`background_gpu_devices`、
//! `background_gpu_memory_limit`）的校验逐种拒绝与全合法；可写挂载给了配错的设置在读任何一块盘之前拒、不降级到 `cpu`；
//! 挂着切换从下一批开始生效、一批之内不换，切换被拒原设置照旧；选了 `gpu` 的一批看得出请求 `gpu`、三段经递进来的那一份 GPU 实现算。
//! GPU 实现挂载时由调用方递进来（同一已定项「挂载时由调用方把它的实现递进去」）：选 `gpu` 而没给实现照 GPU 不在拒挂载、拒切换；
//! 卡清单由实现答；挂着切到 `gpu` 不另给实现时沿用挂载递进来的那一份。用的假实现在 `common_background_gpu_compute`。
//! 等 GPU 结果的上限 `background_gpu_wait_limit`（用户 2026-10-26 定「设超时，超时算出错」）：选 `gpu` 时 0 拒（校验、挂载、挂着切换各一条），
//! 选 `cpu` 时照收不用，切换之后下一批起生效。
//! 只校验的入口（`admit_background_compute_settings`）用一份假清单（[`GpuInventoryForTests`]），两张卡：「card-eight」8 GiB、「card-sixteen」16 GiB，按这个次序；
//! 假实现答的是同样两张卡。
//! 要整理推出几批的用例照 `compaction_batch_relocating_a_user_visible_root_is_non_empty.rs` 的做法：池按 `common::build_pool` 起，
//! 可写挂载，强制选第一个文件那次发布的提交内生块所在的段 50240、一批一个，显式入口推到停。
mod common;
mod common_background_gpu_compute;

use std::time::Duration;

use common::{build_pool, disk_snapshot, parameters};
use common_background_gpu_compute::shared_cpu_backed_gpu_compute;
use singlefs_core::address::{DeviceIdentity, SlotNumber};
use singlefs_core::background_compute::{
    admit_background_compute_settings, BackgroundComputeBackend, BackgroundComputeOfOneBatch,
    BackgroundComputeOfThisMount, BackgroundComputePathTaken, BackgroundComputeSettings,
    BackgroundComputeSettingsRefused, BackgroundTask, GpuComputedSegment, GpuDeviceIdentifier,
    GpuEntryCounts, GpuInventory, GpuMemoryInBytes, ReasonTheBatchRanOnTheCpu,
    BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
};
use singlefs_core::compaction_plan::{
    CompactionBatchSize, CompactionBatchUnits, CompactionSwitches,
};
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::mount::{
    mount_writable, mount_writable_with_background_compute_settings,
    WritableMountWithBackgroundComputeRefused,
};
use singlefs_core::mounted_session::{CompactionRunOutcome, MountedSession};

/// 第一个文件那次发布的提交内生块落在这一段（默认几何，D3（空间分配） 已定项 10 ⑤ 的开放段 [50240, 50304)）。
const SEGMENT_OF_THE_FIRST_PUBLISH: u64 = 50240;
const EIGHT_GIBIBYTES: u64 = 8 << 30;
const SIXTEEN_GIBIBYTES: u64 = 16 << 30;
/// 等 GPU 结果的上限的产品默认，逐项写出来比（`background_compute::BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT`，推的、待量）。
const GPU_WAIT_LIMIT_OF_THE_PRODUCT_IN_SECONDS: u64 = 10;
/// 切换用例换上的那个上限：与默认不同的一个数（用例取的）。
const GPU_WAIT_LIMIT_SWITCHED_TO_IN_MILLISECONDS: u64 = 2500;

/// 假的本机卡清单：交回构造时给的那几张卡，按给的次序。
struct GpuInventoryForTests {
    devices: Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)>,
}

impl GpuInventory for GpuInventoryForTests {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        self.devices.clone()
    }
}

fn card_named(name: &str) -> GpuDeviceIdentifier {
    GpuDeviceIdentifier::of_opaque_bytes(name.as_bytes().to_vec())
}

/// 两张卡：「card-eight」8 GiB 在前、「card-sixteen」16 GiB 在后。
fn inventory_of_an_eight_and_a_sixteen_gibibyte_card() -> GpuInventoryForTests {
    GpuInventoryForTests {
        devices: vec![
            (card_named("card-eight"), GpuMemoryInBytes(EIGHT_GIBIBYTES)),
            (
                card_named("card-sixteen"),
                GpuMemoryInBytes(SIXTEEN_GIBIBYTES),
            ),
        ],
    }
}

fn gpu_settings(
    background_gpu_devices: Option<Vec<GpuDeviceIdentifier>>,
    background_gpu_memory_limit: Option<Vec<GpuMemoryInBytes>>,
) -> BackgroundComputeSettings {
    BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Gpu,
        background_gpu_devices,
        background_gpu_memory_limit,
        background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
    }
}

/// D26（后台整理与放置回收） 已定项 13 第 1–3 条的产品默认，逐项写出来比：`cpu`，卡与上限都不给，等 GPU 的上限 10 秒。
fn cpu_with_no_card_and_no_limit() -> BackgroundComputeSettings {
    BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Cpu,
        background_gpu_devices: None,
        background_gpu_memory_limit: None,
        background_gpu_wait_limit: Duration::from_secs(GPU_WAIT_LIMIT_OF_THE_PRODUCT_IN_SECONDS),
    }
}

/// [`admissible_gpu_settings_naming_both_cards`] 换掉等 GPU 结果的上限。
fn admissible_gpu_settings_naming_both_cards_waiting_at_most(
    background_gpu_wait_limit: Duration,
) -> BackgroundComputeSettings {
    let BackgroundComputeSettings {
        background_compute,
        background_gpu_devices,
        background_gpu_memory_limit,
        background_gpu_wait_limit: _,
    } = admissible_gpu_settings_naming_both_cards();
    BackgroundComputeSettings {
        background_compute,
        background_gpu_devices,
        background_gpu_memory_limit,
        background_gpu_wait_limit,
    }
}

/// 两张卡都点名、上限各取那张卡的显存总量（等于总量不算超过）：一组全合法的 `gpu` 设置。
fn admissible_gpu_settings_naming_both_cards() -> BackgroundComputeSettings {
    gpu_settings(
        Some(vec![card_named("card-sixteen"), card_named("card-eight")]),
        Some(vec![
            GpuMemoryInBytes(SIXTEEN_GIBIBYTES),
            GpuMemoryInBytes(EIGHT_GIBIBYTES),
        ]),
    )
}

/// 会话装上整理只供测试的开关：强制选段 50240、一批一个，其余照产品值。
fn force_one_unit_batches_on_the_segment_of_the_first_publish(session: &mut MountedSession) {
    let region = CompactionRegion::of_one_cluster_segment(
        DeviceIdentity(0),
        SlotNumber(SEGMENT_OF_THE_FIRST_PUBLISH),
    )
    .expect("64 的倍数");
    let CompactionSwitches {
        batch_size: _,
        forced_region: _,
        run_forcing,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    } = CompactionSwitches::PRODUCT;
    session
        .allocator
        .set_compaction_switches(CompactionSwitches {
            batch_size: CompactionBatchSize::ForcedForTests(
                CompactionBatchUnits::new(1).expect("至少 1"),
            ),
            forced_region: Some(region),
            run_forcing,
            region_exclusion,
            destination_policy,
            release_checksum_mismatch_after_issue,
            empty_cluster_segments_floor,
            relocation_publish_order,
        });
}

fn requested_backends_of(batches: &[BackgroundComputeOfOneBatch]) -> Vec<BackgroundComputeBackend> {
    batches
        .iter()
        .map(|batch| batch.requested_backend)
        .collect()
}

#[test]
fn gpu_without_a_gpu_inventory_is_refused_as_gpu_not_present() {
    let refused = admit_background_compute_settings(gpu_settings(None, None), None);
    assert_eq!(
        refused,
        Err(BackgroundComputeSettingsRefused::GpuNotPresent),
        "选了 gpu 而没有卡清单：GPU 不在，拒"
    );
}

#[test]
fn gpu_with_an_inventory_listing_no_card_and_no_card_named_is_refused_as_gpu_not_present() {
    let inventory_without_a_card = GpuInventoryForTests {
        devices: Vec::new(),
    };
    let refused = admit_background_compute_settings(
        gpu_settings(None, None),
        Some(&inventory_without_a_card),
    );
    assert_eq!(
        refused,
        Err(BackgroundComputeSettingsRefused::GpuNotPresent),
        "卡清单一张卡都没有、又没点名卡：参与的卡是本机全部显卡，一张都没有，GPU 不在，拒"
    );
}

#[test]
fn gpu_naming_a_card_missing_from_the_inventory_is_refused() {
    let inventory = inventory_of_an_eight_and_a_sixteen_gibibyte_card();
    let refused = admit_background_compute_settings(
        gpu_settings(
            Some(vec![card_named("card-eight"), card_named("card-missing")]),
            None,
        ),
        Some(&inventory),
    );
    assert_eq!(
        refused,
        Err(
            BackgroundComputeSettingsRefused::NamedGpuDeviceNotInTheInventory {
                device: card_named("card-missing"),
            }
        ),
        "点名的「card-missing」不在卡清单里：拒，并点名是哪一张"
    );
}

#[test]
fn gpu_memory_limits_fewer_than_the_named_cards_are_refused() {
    let inventory = inventory_of_an_eight_and_a_sixteen_gibibyte_card();
    let refused = admit_background_compute_settings(
        gpu_settings(
            Some(vec![card_named("card-eight"), card_named("card-sixteen")]),
            Some(vec![GpuMemoryInBytes(EIGHT_GIBIBYTES)]),
        ),
        Some(&inventory),
    );
    assert_eq!(
        refused,
        Err(
            BackgroundComputeSettingsRefused::MemoryLimitCountDifferentFromTheParticipatingDevices {
                memory_limits_given: 1,
                participating_devices: 2,
            }
        ),
        "点了两张卡、上限只给一个：不补齐，拒"
    );
}

#[test]
fn gpu_memory_limits_fewer_than_every_card_of_the_host_when_no_card_is_named_are_refused() {
    let inventory = inventory_of_an_eight_and_a_sixteen_gibibyte_card();
    let refused = admit_background_compute_settings(
        gpu_settings(None, Some(vec![GpuMemoryInBytes(EIGHT_GIBIBYTES)])),
        Some(&inventory),
    );
    assert_eq!(
        refused,
        Err(
            BackgroundComputeSettingsRefused::MemoryLimitCountDifferentFromTheParticipatingDevices {
                memory_limits_given: 1,
                participating_devices: 2,
            }
        ),
        "没点名卡：上限对本机全部显卡（两张），只给一个，拒"
    );
}

#[test]
fn gpu_memory_limit_above_the_total_memory_of_its_card_is_refused() {
    let inventory = inventory_of_an_eight_and_a_sixteen_gibibyte_card();
    let refused = admit_background_compute_settings(
        gpu_settings(
            Some(vec![card_named("card-sixteen"), card_named("card-eight")]),
            Some(vec![
                GpuMemoryInBytes(SIXTEEN_GIBIBYTES),
                GpuMemoryInBytes(EIGHT_GIBIBYTES + 1),
            ]),
        ),
        Some(&inventory),
    );
    assert_eq!(
        refused,
        Err(
            BackgroundComputeSettingsRefused::MemoryLimitAboveTheTotalMemoryOfTheDevice {
                device: card_named("card-eight"),
                memory_limit: GpuMemoryInBytes(EIGHT_GIBIBYTES + 1),
                total_memory: GpuMemoryInBytes(EIGHT_GIBIBYTES),
            }
        ),
        "上限按点名的次序一张卡一个：第二个数给的是 card-eight，比它的 8 GiB 多一个字节，拒"
    );
}

#[test]
fn gpu_naming_an_empty_card_list_is_refused_as_undecided() {
    let inventory = inventory_of_an_eight_and_a_sixteen_gibibyte_card();
    let refused =
        admit_background_compute_settings(gpu_settings(Some(Vec::new()), None), Some(&inventory));
    assert_eq!(
        refused,
        Err(BackgroundComputeSettingsRefused::EmptyGpuDeviceListWhoseMeaningIsUndecided),
        "卡列表给了而一张都没点：条款没写，第一版拒"
    );
}

#[test]
fn gpu_naming_the_same_card_twice_is_refused_as_undecided() {
    let inventory = inventory_of_an_eight_and_a_sixteen_gibibyte_card();
    let refused = admit_background_compute_settings(
        gpu_settings(
            Some(vec![card_named("card-eight"), card_named("card-eight")]),
            None,
        ),
        Some(&inventory),
    );
    assert_eq!(
        refused,
        Err(
            BackgroundComputeSettingsRefused::GpuDeviceNamedMoreThanOnceWhoseMeaningIsUndecided {
                device: card_named("card-eight"),
            }
        ),
        "同一张卡点了两次：条款没写，第一版拒，并点名是哪一张"
    );
}

#[test]
fn gpu_settings_naming_both_cards_with_limits_equal_to_their_total_memory_are_admitted() {
    let inventory = inventory_of_an_eight_and_a_sixteen_gibibyte_card();
    let admitted = admit_background_compute_settings(
        admissible_gpu_settings_naming_both_cards(),
        Some(&inventory),
    )
    .expect("两张卡都在清单里、上限个数等于卡数、每个上限等于那张卡的显存总量：全合法");
    assert_eq!(
        admitted,
        admissible_gpu_settings_naming_both_cards(),
        "校验过的那一组就是交进去的那一组，一项不改"
    );
}

#[test]
fn cpu_settings_are_admitted_without_checking_the_cards_or_the_limits() {
    let cpu_settings_with_cards_and_limits_that_would_not_pass = BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Cpu,
        background_gpu_devices: Some(vec![card_named("card-missing")]),
        background_gpu_memory_limit: Some(vec![
            GpuMemoryInBytes(u64::MAX),
            GpuMemoryInBytes(u64::MAX),
        ]),
        background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
    };
    let admitted = admit_background_compute_settings(
        cpu_settings_with_cards_and_limits_that_would_not_pass.clone(),
        None,
    )
    .expect("选 cpu 时卡与上限照收、不校验，没有卡清单也收");
    assert_eq!(
        admitted, cpu_settings_with_cards_and_limits_that_would_not_pass,
        "照收的就是交进去的那一组"
    );
}

#[test]
fn writable_mount_with_gpu_settings_and_no_gpu_is_refused_before_any_disk_byte_changes() {
    let mut pool = build_pool("background-compute-mount-refused-without-a-gpu");
    let mut devices = pool.reopen_recorded();
    let before_the_mount = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let refused = mount_writable_with_background_compute_settings(
        &parameters(),
        &mut devices,
        gpu_settings(None, None),
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
        "选了 gpu 而没给 GPU 实现：GPU 不在，拒绝挂载，不退回 cpu 挂上：{refused:?}"
    );
    pool.devices = Some(devices);
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_mount,
        "设置被拒在读任何一块盘之前：系统配置槽、根环里自证过的根、录制流的步数都与挂载之前逐项相同"
    );
}

#[test]
fn writable_mount_with_admitted_gpu_settings_hands_them_to_the_session_in_effect() {
    let mut pool = build_pool("background-compute-mount-with-admitted-gpu-settings");
    let mut devices = pool.reopen_recorded();
    let (_gpu_compute_for_tests, gpu_compute) = shared_cpu_backed_gpu_compute();
    let mounted = mount_writable_with_background_compute_settings(
        &parameters(),
        &mut devices,
        admissible_gpu_settings_naming_both_cards(),
        Some(gpu_compute.clone()),
    )
    .expect("全合法的 gpu 设置：挂载照常做成");
    let (_, session) = MountedSession::of_the_mount(mounted);
    pool.devices = Some(devices);
    assert_eq!(
        session.background_compute().in_effect().settings(),
        &admissible_gpu_settings_naming_both_cards(),
        "挂载给的那一组装进会话、此刻生效"
    );
    assert_eq!(
        session.background_compute().requested_from_the_next_batch(),
        None,
        "挂载那一刻没有待生效的切换"
    );
}

#[test]
fn writable_mount_without_background_compute_settings_takes_the_product_cpu_settings() {
    let mut pool = build_pool("background-compute-mount-without-settings");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    pool.devices = Some(devices);
    assert_eq!(
        session.background_compute().in_effect().settings(),
        &cpu_with_no_card_and_no_limit(),
        "现有的可写挂载入口等价于挂载时给产品默认：cpu，卡与上限都不给"
    );
    assert_eq!(
        session.background_compute().requested_from_the_next_batch(),
        None,
        "挂载那一刻没有待生效的切换"
    );
}

#[test]
fn a_switch_requested_in_the_middle_of_a_batch_leaves_that_batch_on_its_settings_and_takes_effect_from_the_next_batch(
) {
    let mut pool = build_pool("background-compute-switch-in-the-middle-of-a-batch");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    force_one_unit_batches_on_the_segment_of_the_first_publish(&mut session);
    let (_gpu_compute_for_tests, gpu_compute) = shared_cpu_backed_gpu_compute();
    let mut batches_that_reached_the_hook: u64 = 0;
    let mut request_gpu_in_the_middle_of_the_first_batch =
        |background_compute: &mut BackgroundComputeOfThisMount| {
            batches_that_reached_the_hook += 1;
            if batches_that_reached_the_hook == 1 {
                background_compute
                    .request_switch_from_the_next_batch(
                        admissible_gpu_settings_naming_both_cards(),
                        Some(gpu_compute.clone()),
                    )
                    .expect("全合法的 gpu 设置：切换收下，记成待生效");
            }
        };
    let run = session
        .run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch(
            &mut devices,
            &mut request_gpu_in_the_middle_of_the_first_batch,
        )
        .expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        matches!(run.outcome, CompactionRunOutcome::Stopped(_)),
        "这一趟照常停，不是一次发布没发成：{:?}",
        run.outcome
    );
    let requested_backends = requested_backends_of(&run.background_compute_of_each_batch);
    assert!(
        requested_backends.len() >= 2,
        "签发一次、至少搬一批：至少两批（这一趟 {requested_backends:?}）"
    );
    assert_eq!(
        requested_backends.first(),
        Some(&BackgroundComputeBackend::Cpu),
        "请求切换的那一批开头读到的是 cpu：这一批照旧（这一趟 {requested_backends:?}）"
    );
    assert!(
        requested_backends[1..]
            .iter()
            .all(|backend| *backend == BackgroundComputeBackend::Gpu),
        "下一批开头换上 gpu，之后每一批都是 gpu（这一趟 {requested_backends:?}）"
    );
    assert_eq!(
        session.background_compute().in_effect().settings(),
        &admissible_gpu_settings_naming_both_cards(),
        "这一趟之后生效的是切换到的那一组"
    );
    assert_eq!(
        session.background_compute().requested_from_the_next_batch(),
        None,
        "待生效的那一组已在下一批开头换上"
    );
}

#[test]
fn a_refused_switch_keeps_the_settings_in_effect_for_the_next_batches() {
    let mut pool = build_pool("background-compute-refused-switch");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    force_one_unit_batches_on_the_segment_of_the_first_publish(&mut session);
    let refused =
        session.switch_background_compute_from_the_next_batch(gpu_settings(None, None), None);
    assert_eq!(
        refused,
        Err(BackgroundComputeSettingsRefused::GpuNotPresent),
        "切到 gpu 而 GPU 不在：拒绝切换"
    );
    assert_eq!(
        session.background_compute().in_effect().settings(),
        &cpu_with_no_card_and_no_limit(),
        "切换被拒：生效的仍是挂载时那一组 cpu"
    );
    assert_eq!(
        session.background_compute().requested_from_the_next_batch(),
        None,
        "切换被拒：没有记成待生效"
    );
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        !run.background_compute_of_each_batch.is_empty(),
        "强制选段、一批一个：这一趟至少推一批"
    );
    for batch in &run.background_compute_of_each_batch {
        assert_eq!(
            *batch,
            BackgroundComputeOfOneBatch {
                task: BackgroundTask::CompactionRelocation,
                requested_backend: BackgroundComputeBackend::Cpu,
                path_taken: BackgroundComputePathTaken::Cpu(
                    ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings
                ),
                compute_error_that_abandoned_the_batch: None,
            },
            "被拒的切换不生效：每一批请求的都是 cpu、照设置走 CPU"
        );
    }
}

/// 选 `gpu` 的一批（D26（后台整理与放置回收） 已定项 13 第 1、8 条）：请求的是 gpu，三段经递进来的那一份实现算，强制选段时不打分，
/// 乙、甲两段的条目数照这一批扫到的记录记下，不注入故障时全都确认、没有不确定、没有交 CPU 判。
#[test]
fn a_batch_under_gpu_settings_records_the_gpu_request_and_runs_its_segments_on_the_gpu_compute() {
    let mut pool = build_pool("background-compute-gpu-batch-runs-on-the-gpu-compute");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    force_one_unit_batches_on_the_segment_of_the_first_publish(&mut session);
    let (gpu_compute_kept_by_the_test, gpu_compute) = shared_cpu_backed_gpu_compute();
    session
        .switch_background_compute_from_the_next_batch(
            admissible_gpu_settings_naming_both_cards(),
            Some(gpu_compute.clone()),
        )
        .expect("全合法的 gpu 设置：切换收下");
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        matches!(run.outcome, CompactionRunOutcome::Stopped(_)),
        "这一趟照常停，不是一次发布没发成：{:?}",
        run.outcome
    );
    assert!(
        !run.background_compute_of_each_batch.is_empty(),
        "强制选段、一批一个：这一趟至少推一批"
    );
    let mut region_records_classified_in_the_run: u64 = 0;
    for batch in &run.background_compute_of_each_batch {
        assert_eq!(
            (batch.task, batch.requested_backend),
            (
                BackgroundTask::CompactionRelocation,
                BackgroundComputeBackend::Gpu
            ),
            "整理搬迁的一批、请求的是 gpu"
        );
        let BackgroundComputePathTaken::Gpu(gpu_work) = batch.path_taken else {
            panic!("请求 gpu 的一批走 GPU：{batch:?}");
        };
        assert_eq!(
            gpu_work.live_slots_per_cluster_segment,
            GpuEntryCounts::default(),
            "强制选段：不打分，丙段一条都不交"
        );
        assert_eq!(
            gpu_work
                .entries_of_the_three_segments()
                .uncertain_this_round
                + gpu_work
                    .entries_of_the_three_segments()
                    .judged_by_the_cpu_reference,
            0,
            "不注入故障：全都确认，{gpu_work:?}"
        );
        region_records_classified_in_the_run += gpu_work
            .region_record_classification
            .confirmed_by_both_kernels;
    }
    assert!(
        region_records_classified_in_the_run > 0,
        "对 R 扫描时 R 里读到的记录交乙段分类，确认的条目数报出来了（R 搬空之后那几批扫不到记录、报 0）"
    );
    let gpu_compute_for_tests = gpu_compute_kept_by_the_test
        .lock()
        .expect("假实现没在别处 panic 过");
    assert!(
        gpu_compute_for_tests.entries_handed_in_to(GpuComputedSegment::RegionRecordClassification)
            > 0
            && gpu_compute_for_tests
                .entries_handed_in_to(GpuComputedSegment::UnitChecksumVerification)
                > 0,
        "乙、甲两段都经递进来的实现算了：{:?}",
        gpu_compute_for_tests.entries_handed_in
    );
}

/// 挂载时选 `cpu`、递进来一份实现（D26 已定项 13「挂载时由调用方把它的实现递进去」）：挂着切到 `gpu` 时不另给实现，沿用挂载递进来的那一份，切换收下。
#[test]
fn a_switch_to_gpu_without_an_implementation_reuses_the_one_handed_in_at_the_mount() {
    let mut pool = build_pool("background-compute-switch-reuses-the-gpu-compute-of-the-mount");
    let mut devices = pool.reopen_recorded();
    let (_gpu_compute_for_tests, gpu_compute) = shared_cpu_backed_gpu_compute();
    let mounted = mount_writable_with_background_compute_settings(
        &parameters(),
        &mut devices,
        cpu_with_no_card_and_no_limit(),
        Some(gpu_compute.clone()),
    )
    .expect("cpu 设置连同一份实现：挂载照常做成");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    pool.devices = Some(devices);
    session
        .switch_background_compute_from_the_next_batch(
            admissible_gpu_settings_naming_both_cards(),
            None,
        )
        .expect("挂载递进来过实现：切到 gpu 不另给也照它校验、收下");
    assert_eq!(
        session
            .background_compute()
            .requested_from_the_next_batch()
            .and_then(|requested| requested.gpu_compute()),
        Some(&gpu_compute),
        "待生效的那一组带的就是挂载递进来的那一份实现"
    );
}

/// 选 `gpu` 而点名的卡不在递进来的那一份实现答的卡清单里（第 4 条「点名的卡不存在」）：挂载照实现的卡清单校验、拒，在读任何一块盘之前。
#[test]
fn writable_mount_with_gpu_settings_naming_a_card_the_gpu_compute_does_not_list_is_refused_before_any_disk_byte_changes(
) {
    let mut pool = build_pool("background-compute-mount-naming-a-card-the-gpu-compute-lacks");
    let mut devices = pool.reopen_recorded();
    let before_the_mount = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let (_gpu_compute_for_tests, gpu_compute) = shared_cpu_backed_gpu_compute();
    let refused = mount_writable_with_background_compute_settings(
        &parameters(),
        &mut devices,
        gpu_settings(Some(vec![card_named("card-missing")]), None),
        Some(gpu_compute),
    );
    assert!(
        matches!(
            &refused,
            Err(
                WritableMountWithBackgroundComputeRefused::BackgroundComputeSettings(
                    BackgroundComputeSettingsRefused::NamedGpuDeviceNotInTheInventory { device }
                )
            ) if *device == card_named("card-missing")
        ),
        "点名的卡不在实现的卡清单里：拒绝挂载：{refused:?}"
    );
    pool.devices = Some(devices);
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_mount,
        "设置被拒在读任何一块盘之前"
    );
}

#[test]
fn gpu_settings_with_a_gpu_wait_limit_of_zero_are_refused() {
    let inventory = inventory_of_an_eight_and_a_sixteen_gibibyte_card();
    let refused = admit_background_compute_settings(
        admissible_gpu_settings_naming_both_cards_waiting_at_most(Duration::ZERO),
        Some(&inventory),
    );
    assert_eq!(
        refused,
        Err(BackgroundComputeSettingsRefused::GpuWaitLimitOfZero),
        "卡与显存上限都合法、等 GPU 的上限是 0：一批交出去就算超时，拒"
    );
}

#[test]
fn gpu_settings_with_the_smallest_gpu_wait_limit_above_zero_are_admitted_with_it() {
    let inventory = inventory_of_an_eight_and_a_sixteen_gibibyte_card();
    let admitted = admit_background_compute_settings(
        admissible_gpu_settings_naming_both_cards_waiting_at_most(Duration::from_nanos(1)),
        Some(&inventory),
    )
    .expect("等 GPU 的上限 1 纳秒不是 0：照收");
    assert_eq!(
        admitted.background_gpu_wait_limit,
        Duration::from_nanos(1),
        "校验过的那一组带着交进去的上限"
    );
}

#[test]
fn cpu_settings_with_a_gpu_wait_limit_of_zero_are_admitted_and_keep_it_unused() {
    let cpu_settings_with_a_wait_limit_of_zero = BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Cpu,
        background_gpu_devices: None,
        background_gpu_memory_limit: None,
        background_gpu_wait_limit: Duration::ZERO,
    };
    let admitted =
        admit_background_compute_settings(cpu_settings_with_a_wait_limit_of_zero.clone(), None)
            .expect("选 cpu 时等 GPU 的上限照收、不用，0 也收");
    assert_eq!(
        admitted, cpu_settings_with_a_wait_limit_of_zero,
        "照收的就是交进去的那一组"
    );
}

#[test]
fn writable_mount_with_gpu_settings_whose_gpu_wait_limit_is_zero_is_refused_before_any_disk_byte_changes(
) {
    let mut pool = build_pool("background-compute-mount-refused-for-a-gpu-wait-limit-of-zero");
    let mut devices = pool.reopen_recorded();
    let before_the_mount = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let (_gpu_compute_for_tests, gpu_compute) = shared_cpu_backed_gpu_compute();
    let refused = mount_writable_with_background_compute_settings(
        &parameters(),
        &mut devices,
        admissible_gpu_settings_naming_both_cards_waiting_at_most(Duration::ZERO),
        Some(gpu_compute.clone()),
    );
    assert!(
        matches!(
            refused,
            Err(
                WritableMountWithBackgroundComputeRefused::BackgroundComputeSettings(
                    BackgroundComputeSettingsRefused::GpuWaitLimitOfZero
                )
            )
        ),
        "等 GPU 的上限是 0：拒绝挂载，不退回 cpu 挂上：{refused:?}"
    );
    pool.devices = Some(devices);
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before_the_mount,
        "设置被拒在读任何一块盘之前：系统配置槽、根环里自证过的根、录制流的步数都与挂载之前逐项相同"
    );
}

#[test]
fn a_switch_to_gpu_settings_whose_gpu_wait_limit_is_zero_is_refused_and_keeps_the_settings_in_effect(
) {
    let mut pool = build_pool("background-compute-switch-refused-for-a-gpu-wait-limit-of-zero");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    pool.devices = Some(devices);
    let (_gpu_compute_for_tests, gpu_compute) = shared_cpu_backed_gpu_compute();
    let refused = session.switch_background_compute_from_the_next_batch(
        admissible_gpu_settings_naming_both_cards_waiting_at_most(Duration::ZERO),
        Some(gpu_compute.clone()),
    );
    assert_eq!(
        refused,
        Err(BackgroundComputeSettingsRefused::GpuWaitLimitOfZero),
        "切到等 GPU 的上限是 0 的 gpu 设置：拒绝切换"
    );
    assert_eq!(
        session.background_compute().in_effect().settings(),
        &cpu_with_no_card_and_no_limit(),
        "切换被拒：生效的仍是挂载时那一组"
    );
    assert_eq!(
        session.background_compute().requested_from_the_next_batch(),
        None,
        "切换被拒：没有记成待生效"
    );
}

#[test]
fn a_switch_of_the_gpu_wait_limit_requested_in_the_middle_of_a_batch_takes_effect_from_the_next_batch(
) {
    let mut pool = build_pool("background-compute-switch-of-the-gpu-wait-limit");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    force_one_unit_batches_on_the_segment_of_the_first_publish(&mut session);
    let (_gpu_compute_for_tests, gpu_compute) = shared_cpu_backed_gpu_compute();
    let switched_to = Duration::from_millis(GPU_WAIT_LIMIT_SWITCHED_TO_IN_MILLISECONDS);
    let mut gpu_wait_limit_in_effect_in_each_batch: Vec<Duration> = Vec::new();
    let mut request_a_new_gpu_wait_limit_in_the_middle_of_the_first_batch =
        |background_compute: &mut BackgroundComputeOfThisMount| {
            if gpu_wait_limit_in_effect_in_each_batch.is_empty() {
                background_compute
                    .request_switch_from_the_next_batch(
                        admissible_gpu_settings_naming_both_cards_waiting_at_most(switched_to),
                        Some(gpu_compute.clone()),
                    )
                    .expect("全合法的 gpu 设置：切换收下，记成待生效");
            }
            gpu_wait_limit_in_effect_in_each_batch.push(
                background_compute
                    .in_effect()
                    .settings()
                    .background_gpu_wait_limit,
            );
        };
    let run = session
        .run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch(
            &mut devices,
            &mut request_a_new_gpu_wait_limit_in_the_middle_of_the_first_batch,
        )
        .expect("整理一趟");
    pool.devices = Some(devices);
    assert!(
        matches!(run.outcome, CompactionRunOutcome::Stopped(_)),
        "这一趟照常停，不是一次发布没发成：{:?}",
        run.outcome
    );
    assert!(
        gpu_wait_limit_in_effect_in_each_batch.len() >= 2,
        "签发一次、至少搬一批：至少两批（这一趟 {gpu_wait_limit_in_effect_in_each_batch:?}）"
    );
    assert_eq!(
        gpu_wait_limit_in_effect_in_each_batch.first(),
        Some(&Duration::from_secs(
            GPU_WAIT_LIMIT_OF_THE_PRODUCT_IN_SECONDS
        )),
        "请求切换的那一批照旧是挂载时的上限（这一趟 {gpu_wait_limit_in_effect_in_each_batch:?}）"
    );
    assert!(
        gpu_wait_limit_in_effect_in_each_batch[1..]
            .iter()
            .all(|wait_limit| *wait_limit == switched_to),
        "下一批开头换上切换到的上限，之后每一批都是它（这一趟 {gpu_wait_limit_in_effect_in_each_batch:?}）"
    );
}
